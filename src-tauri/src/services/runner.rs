use crate::{
    events::{EVT_CLOSE, EVT_LOG_ERROR, EVT_LOG_STDERR, EVT_LOG_STDOUT},
    state::FrpcProcState,
};
use serde::Serialize;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone, Debug)]
pub struct ClosePayload {
    pub code: Option<i32>,
}

use crate::state::notify_watchdog;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn start(
    app: &AppHandle,
    proc_state: &FrpcProcState,
    exe_path: &str,
    cfg_path: &str,
) -> Result<u32, String> {
    // 防重复
    {
        let g = proc_state.child.lock().map_err(|e| e.to_string())?;
        if g.is_some() {
            return Err("frpc is already running".into());
        }
    }

    // 构建命令
    let mut cmd = Command::new(exe_path);
    cmd.arg("-c")
        .arg(cfg_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Windows: 隐藏控制台窗口
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn frpc failed: {e} (exe: {exe_path}, cfg: {cfg_path})"))?;
    let pid = child.id();

    let _ = notify_watchdog(app, format!("SET PID {pid}").into());

    // 拿到输出管道
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // 放入全局状态
    {
        let mut g = proc_state.child.lock().map_err(|e| e.to_string())?;
        *g = Some(child);
    }

    // stdout → 事件
    if let Some(out) = stdout {
        let app2 = app.clone();
        thread::spawn(move || {
            let reader = BufReader::new(out);
            for line in reader.lines() {
                match line {
                    Ok(s) => {
                        let _ = app2.emit(EVT_LOG_STDOUT, s);
                    }
                    Err(e) => {
                        let _ = app2.emit(EVT_LOG_ERROR, format!("read stdout error: {e}"));
                        break;
                    }
                }
            }
        });
    }

    // stderr → 事件
    if let Some(err) = stderr {
        let app2 = app.clone();
        thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines() {
                match line {
                    Ok(s) => {
                        let _ = app2.emit(EVT_LOG_STDERR, s);
                    }
                    Err(e) => {
                        let _ = app2.emit(EVT_LOG_ERROR, format!("read stderr error: {e}"));
                        break;
                    }
                }
            }
        });
    }

    // 退出监控线程：子进程退出→清空句柄并发 EVT_CLOSE
    let app_close = app.clone();
    let child_arc = proc_state.child.clone();
    thread::spawn(move || loop {
        let status_opt = {
            let mut guard = child_arc.lock().expect("poisoned");
            if let Some(ch) = guard.as_mut() {
                match ch.try_wait() {
                    Ok(st) => st,
                    Err(e) => {
                        let _ = app_close.emit(EVT_LOG_ERROR, format!("try_wait error: {e}"));
                        None
                    }
                }
            } else {
                break; // 已被 stop() 清空
            }
        };

        if let Some(status) = status_opt {
            // 清空句柄、发送 close 事件
            let mut guard = child_arc.lock().expect("poisoned");
            *guard = None;
            let code = status.code();
            let _ = app_close.emit(EVT_CLOSE, ClosePayload { code });
            break;
        }

        thread::sleep(Duration::from_millis(200));
    });

    Ok(pid)
}

pub fn stop(app: &AppHandle, proc_state: &FrpcProcState) -> Result<(), String> {
    let mut g = proc_state.child.lock().map_err(|e| e.to_string())?;
    if let Some(ch) = g.as_mut() {
        ch.kill().map_err(|e| format!("kill frpc failed: {e}"))?;
    }
    let _ = notify_watchdog(app, "CLEAR".into());
    Ok(())
}

pub fn is_running(proc_state: &FrpcProcState) -> Result<bool, String> {
    let g = proc_state.child.lock().map_err(|e| e.to_string())?;
    Ok(g.is_some())
}
