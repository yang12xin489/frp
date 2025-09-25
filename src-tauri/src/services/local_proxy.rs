use crate::state::FrpcProcState;
use serde_json::json;
use std::{
    collections::HashMap,
    io::{Error, ErrorKind, Result},
    net::SocketAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tauri::{AppHandle, Emitter};
use tokio::task::JoinHandle;
use tokio::{
    io,
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{TcpListener, TcpStream},
    time::{interval, MissedTickBehavior},
};

pub struct ProxySpec {
    pub id: String,
    pub listener: TcpListener,
    pub target: SocketAddr,
}

#[derive(Clone, Default)]
pub struct ProxyStats {
    up_total: Arc<AtomicU64>,
    down_total: Arc<AtomicU64>,
}

impl ProxyStats {
    #[inline]
    fn new() -> Self {
        Self::default()
    }
}

/// 只在读侧做计数，避免双计数
pub struct CountRead<T> {
    inner: T,
    counter: Arc<AtomicU64>,
}
impl<T> CountRead<T> {
    #[inline]
    pub fn new(inner: T, counter: Arc<AtomicU64>) -> Self {
        Self { inner, counter }
    }
}
impl<T: AsyncRead + Unpin> AsyncRead for CountRead<T> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> std::task::Poll<Result<()>> {
        let before = buf.filled().len();
        let r = std::pin::Pin::new(&mut self.inner).poll_read(cx, buf);
        if let std::task::Poll::Ready(Ok(())) = &r {
            let now = buf.filled().len();
            if now > before {
                self.counter
                    .fetch_add((now - before) as u64, Ordering::Relaxed);
            }
        }
        r
    }
}
impl<T: AsyncWrite + Unpin> AsyncWrite for CountRead<T> {
    #[inline]
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        data: &[u8],
    ) -> std::task::Poll<Result<usize>> {
        std::pin::Pin::new(&mut self.inner).poll_write(cx, data)
    }
    #[inline]
    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_flush(cx)
    }
    #[inline]
    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

/// ===================== 启动 shim + 采样 =====================

pub async fn run_tcp_shim(app: AppHandle, proc_state: &FrpcProcState) -> Result<()> {
    {
        let mut g = proc_state
            .shim_tasks
            .lock()
            .map_err(|_| Error::new(ErrorKind::Other, "lock shim_tasks"))?;
        for h in g.drain(..) {
            h.abort(); // 立即取消任务；监听 socket 会被 drop，从而释放端口
        }
    }
    // 取出 specs 所有权（短锁，不跨 await）
    let mut specs: Vec<ProxySpec> = {
        let mut g = proc_state
            .proxy_specs
            .lock()
            .map_err(|_| Error::new(ErrorKind::Other, "lock proxy_specs"))?;
        std::mem::take(&mut *g)
    };
    if specs.is_empty() {
        eprintln!("[shim] no proxy specs");
        return Ok(());
    }

    let mut new_handles: Vec<JoinHandle<()>> = Vec::with_capacity(specs.len() + 1);
    // 记录 (id, stats) 供采样线程使用；同时为每个 spec 启动监听任务
    let mut view: Vec<(String, Arc<ProxyStats>)> = Vec::with_capacity(specs.len());

    for spec in specs.drain(..) {
        let stats = Arc::new(ProxyStats::new());
        view.push((spec.id.clone(), stats.clone()));
        let app2 = app.clone();
        let id_for_log = spec.id.clone();
        let handle = tokio::spawn(async move {
            if let Err(e) = serve_one_proxy(app2, spec, stats).await {
                eprintln!("[shim:{}] serve error: {e}", id_for_log);
            }
        });
        new_handles.push(handle);
    }

    // 固定 200ms 上报：仅读原子计数，无锁
    let sampler = tokio::spawn({
        let app = app.clone();
        let view = Arc::new(view);
        async move {
            const MS: u64 = 1000;
            let dt = MS as f64 / 1000.0;
            let mut tick = interval(Duration::from_millis(MS));
            tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

            let mut last: HashMap<&str, (u64, u64)> = HashMap::with_capacity(view.len());

            loop {
                tick.tick().await;

                let mut payload = Vec::with_capacity(view.len());
                for (id, st) in view.iter() {
                    let up = st.up_total.load(Ordering::Relaxed);
                    let down = st.down_total.load(Ordering::Relaxed);
                    let (lu, ld) = last.get::<str>(id.as_str()).copied().unwrap_or((up, down));
                    last.insert(id.as_str(), (up, down));

                    payload.push(json!({
                        "proxy": id,
                        "up_bps":   (up - lu) as f64 / dt,
                        "down_bps": (down - ld) as f64 / dt,
                        "up_total": up,
                        "down_total": down
                    }));
                }

                let _ = app.emit("frp:traffic", json!(payload));
            }
        }
    });

    new_handles.push(sampler);

    {
        let mut g = proc_state
            .shim_tasks
            .lock()
            .map_err(|_| Error::new(ErrorKind::Other, "lock shim_tasks"))?;
        *g = new_handles;
    }

    Ok(())
}

/// ===================== 稳定转发：copy_bidirectional =====================

#[inline]
fn set_socket_opts(s: &TcpStream) {
    let _ = s.set_nodelay(true);
}

pub async fn serve_one_proxy(
    _app: AppHandle,
    spec: ProxySpec,
    stats: Arc<ProxyStats>,
) -> Result<()> {
    let ProxySpec {
        id,
        listener,
        target,
    } = spec;
    eprintln!("[shim:{id}] listen {}", listener.local_addr()?);
    loop {
        let (cli, peer) = listener.accept().await?;
        let up = stats.up_total.clone();
        let down = stats.down_total.clone();
        let id__ = id.clone();
        tokio::spawn(async move {
            match handle_conn(cli, target, up, down).await {
                Ok(()) => { /* 正常结束 */ }
                Err(e) => eprintln!("[shim:{id__}] conn {peer} error: {e}"),
            }
        });
    }
}

async fn handle_conn(
    cli: TcpStream,
    target: SocketAddr,
    up_counter: Arc<AtomicU64>,
    down_counter: Arc<AtomicU64>,
) -> Result<()> {
    let cli = cli;
    let svr = TcpStream::connect(target).await?;

    set_socket_opts(&cli);
    set_socket_opts(&svr);

    // 读侧包一层实现计数（client->server 计入 up，server->client 计入 down）
    let mut cli_r = CountRead::new(cli, up_counter);
    let mut svr_r = CountRead::new(svr, down_counter);

    let _ = io::copy_bidirectional(&mut cli_r, &mut svr_r).await?;
    Ok(())
}
