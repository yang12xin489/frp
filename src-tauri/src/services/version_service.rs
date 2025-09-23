use crate::domain::active_frp::{ActiveFrp, SETTINGS_ACTIVE_KEY};
use crate::domain::progress_payload::ProgressPayload;
use crate::domain::version::{FrpVersion, SETTINGS_VERSIONS_KEY};
use crate::events::{EVT_ACTIVATING_STATUS, EVT_DOWNLOAD_PROGRESS};
use crate::infra::archive::{extract_archive_to, find_executable_recursively, frpc_name};
use crate::infra::paths::unpack_dir_for;
use crate::services::config_service::save_now;
use crate::services::runner;
use crate::state::{AppState, FrpcProcState};
use crate::{
    errors::Result,
    infra::{
        http::{client, USER_AGENT},
        paths::get_download_dir,
    },
};
use futures_util::StreamExt;
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::io::AsyncWriteExt;
use tokio::time::{self, Duration as TokioDuration, MissedTickBehavior};

#[derive(Debug, Deserialize)]
struct GhAsset {
    id: u64,
    name: String,
    size: u64,
    browser_download_url: String,
    download_count: Option<u64>,
    created_at: String,
}
#[derive(Debug, Deserialize)]
struct GhRelease {
    name: String,
    assets: Option<Vec<GhAsset>>,
}

fn update_frp_version(
    app: &AppHandle,
    state: &AppState,
    name: &str,
    exist: Option<bool>,
    active: Option<bool>,
) -> Result<()> {
    let r = state.read();
    let mut versions: Vec<FrpVersion> = r
        .settings
        .get(SETTINGS_VERSIONS_KEY)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    drop(r); // 释放读锁

    if let Some(idx) = versions.iter().position(|v| v.name == name) {
        if let Some(e) = exist {
            versions[idx].exist = e;
        }
        if let Some(a) = active {
            if a {
                for (i, v) in versions.iter_mut().enumerate() {
                    v.active = i == idx;
                }
            } else {
                versions[idx].active = false;
            }
        }
    }

    {
        let mut w = state.write();
        w.settings.insert(
            SETTINGS_VERSIONS_KEY.to_string(),
            serde_json::to_value(&versions)
                .map_err(|e| crate::errors::AppError::Other(e.to_string()))?,
        );
    }
    save_now(app, state)?;
    Ok(())
}

fn format_size(n: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    if n == 0 {
        return "0 B".into();
    }
    let i = ((n as f64).log2() / 10.0).floor() as usize;
    let idx = i.min(UNITS.len() - 1);
    let val = (n as f64) / (1024u64.pow(idx as u32) as f64);
    let frac = if idx == 0 {
        0
    } else if val < 10.0 {
        2
    } else if val < 100.0 {
        1
    } else {
        0
    };
    format!("{:.*} {}", frac, val, UNITS[idx])
}

fn pick_asset_by_name(name: &str) -> bool {
    if !name.to_lowercase().contains("frp") {
        return false;
    }
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    match (os, arch) {
        ("macos", "aarch64") => Regex::new("(?i)darwin_arm64").unwrap().is_match(name),
        ("macos", _) => Regex::new("(?i)darwin_amd64").unwrap().is_match(name),
        ("windows", "aarch64") => Regex::new("(?i)windows_arm64").unwrap().is_match(name),
        ("windows", _) => Regex::new("(?i)windows_amd64").unwrap().is_match(name),
        ("linux", "aarch64") => Regex::new("(?i)linux_arm64").unwrap().is_match(name),
        ("linux", _) => Regex::new("(?i)linux_amd64").unwrap().is_match(name),
        _ => false,
    }
}

pub async fn get_versions(app: &AppHandle, state: &AppState) -> Result<Vec<FrpVersion>> {
    let cached: Option<Vec<FrpVersion>> = {
        let r = state.read();
        r.settings
            .get(SETTINGS_VERSIONS_KEY)
            .and_then(|v| serde_json::from_value::<Vec<FrpVersion>>(v.clone()).ok())
    };

    if let Some(v) = cached.as_ref().filter(|v| !v.is_empty()) {
        return Ok(v.clone());
    }

    let url = "https://api.github.com/repos/fatedier/frp/releases?per_page=50";
    let resp = client()
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await
        .map_err(|e| crate::errors::AppError::Other(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(crate::errors::AppError::Other(format!(
            "GitHub API error: {}",
            resp.status()
        )));
    }

    let releases: Vec<GhRelease> = resp
        .json()
        .await
        .map_err(|e| crate::errors::AppError::Other(e.to_string()))?;

    let base = get_download_dir(&app)?;
    let mut versions: Vec<FrpVersion> = Vec::new();

    for rel in releases {
        let version_str = rel.name; // 或者 rel.tag_name，看你结构体定义
        if let Some(assets) = rel.assets {
            if let Some(asset) = assets.into_iter().find(|a| pick_asset_by_name(&a.name)) {
                let real_path = base.join(&asset.name);
                versions.push(FrpVersion {
                    id: asset.id,
                    name: asset.name.clone(),
                    size: format_size(asset.size),
                    version: version_str.clone(),
                    created_at: asset.created_at.chars().take(10).collect(),
                    count: asset.download_count.unwrap_or(0),
                    url: asset.browser_download_url.clone(),
                    exist: real_path.exists(),
                    active: get_active(state).map_or(false, |a| a.name == asset.name),
                });
            }
        }
    }

    {
        let mut w = state.write();
        w.settings.insert(
            SETTINGS_VERSIONS_KEY.to_string(),
            serde_json::to_value(&versions)
                .map_err(|e| crate::errors::AppError::Other(e.to_string()))?,
        );
    }
    save_now(&app, &state)?;

    Ok(versions)
}

pub fn get_active(state: &AppState) -> Option<ActiveFrp> {
    let g = state.read();
    g.settings
        .get(SETTINGS_ACTIVE_KEY)
        .and_then(|v| serde_json::from_value::<ActiveFrp>(v.clone()).ok())
}

pub fn set_active(app: &AppHandle, state: &AppState, active_version: &ActiveFrp) -> Result<()> {
    {
        let mut w = state.write();
        w.settings.insert(
            SETTINGS_ACTIVE_KEY.into(),
            serde_json::to_value(active_version)?,
        );
    }
    save_now(app, state)?;
    Ok(())
}

pub fn activate(app: &AppHandle, state: &AppState, name: &str) -> Result<()> {
    // 1) 确认压缩包存在
    let downloads_dir = get_download_dir(app)?;
    let archive = downloads_dir.join(name);
    if !archive.exists() {
        return Err(crate::errors::AppError::Other(format!(
            "archive not found: {}",
            archive.display()
        ))
        .into());
    }

    // 2) 清理
    let unpack_dir = unpack_dir_for(app, name);
    if unpack_dir.exists() {
        std::fs::remove_dir_all(&unpack_dir)?;
    }

    // 3) 解压到解压目录
    extract_archive_to(&archive, &downloads_dir)
        .map_err(|e| crate::errors::AppError::Other(e.to_string()))?;

    // 4) 查找可执行文件
    let exe_name = frpc_name();
    let exe = find_executable_recursively(&unpack_dir, exe_name).ok_or_else(|| {
        crate::errors::AppError::Other(format!(
            "executable '{}' not found under {}",
            exe_name,
            unpack_dir.display()
        ))
    })?;
    let exe_abs = exe.canonicalize()?;
    // 5) 写入激活记录
    let active_version = ActiveFrp {
        name: name.to_string(),
        archive_path: archive.canonicalize()?.to_string_lossy().into_owned(),
        unpack_dir: unpack_dir.canonicalize()?.to_string_lossy().into_owned(),
        exe_path: exe_abs.to_string_lossy().into_owned(),
        activated_at: chrono::Utc::now().to_rfc3339(),
    };

    // 5) 写入激活记录
    set_active(app, state, &active_version)?;
    update_frp_version(app, state, name, None, Option::from(true))?;
    let _ = app
        .clone()
        .emit(EVT_ACTIVATING_STATUS, json!({ "status": false }));

    Ok(())
}

pub fn clear_active_if_matches(app: &AppHandle, state: &AppState, name: &str) -> Result<()> {
    let mut need_clear = false;
    {
        let g = state.read();
        if let Some(cur) = g
            .settings
            .get(SETTINGS_ACTIVE_KEY)
            .and_then(|v| serde_json::from_value::<FrpVersion>(v.clone()).ok())
        {
            if cur.name == name {
                need_clear = true;
            }
        }
    }
    if need_clear {
        {
            let mut g = state.write();
            g.settings.remove(SETTINGS_ACTIVE_KEY);
        }
        save_now(app, state)?;
    }
    Ok(())
}

fn stop_if_target_active(
    app: &AppHandle,
    state: &AppState,
    proc_state: &State<FrpcProcState>,
    name: &str,
) {
    if let Some(active) = get_active(state) {
        if active.name == name {
            let running = runner::is_running(proc_state).unwrap_or_else(|_e| false);
            if running {
                let _ = runner::stop(app, proc_state);
            }
        }
    }
}

pub fn deactivate(
    app: &AppHandle,
    state: &AppState,
    proc_state: &State<FrpcProcState>,
    name: &str,
) -> Result<()> {
    stop_if_target_active(app, state, proc_state, name);

    // 如果当前激活的是它，先清空记录（调用者也可在外层先停进程）
    clear_active_if_matches(app, state, name)?;

    // 删解压目录
    let unpack_dir = unpack_dir_for(app, name);
    if unpack_dir.exists() {
        std::fs::remove_dir_all(&unpack_dir)?;
    }

    update_frp_version(app, state, name, None, Option::from(false))?;

    Ok(())
}

pub fn delete(
    app: &AppHandle,
    state: &AppState,
    proc_state: &State<FrpcProcState>,
    name: &str,
) -> Result<()> {
    stop_if_target_active(app, state, proc_state, name);

    // 如果当前激活的是它，先清空记录（调用者也可在外层先停进程）
    clear_active_if_matches(app, state, name)?;

    // 删压缩包
    let downloads = get_download_dir(app)?;
    let archive = downloads.join(name);
    if archive.exists() {
        std::fs::remove_file(&archive)?;
    }

    // 删解压目录
    let unpack_dir = unpack_dir_for(app, name);
    if unpack_dir.exists() {
        std::fs::remove_dir_all(&unpack_dir)?;
    }

    update_frp_version(app, state, name, Option::from(false), Option::from(false))?;

    Ok(())
}

pub async fn download(app: &AppHandle, state: &AppState, name: &str, url: &str) -> Result<()> {
    let dir = get_download_dir(app)?;
    let target = dir.join(name);

    // 已存在：直接发 100 并返回
    if target.exists() {
        let _ = app.emit(
            EVT_DOWNLOAD_PROGRESS,
            ProgressPayload {
                name: name.to_string(),
                progress: 100,
            },
        );
        update_frp_version(app, state, name, Option::from(true), Option::from(false))?;
        return Ok(());
    }

    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(crate::errors::AppError::Other(format!("HTTP {}", resp.status())).into());
    }

    let total = resp
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let mut file = tokio::fs::File::create(&target).await?;
    let mut stream = resp.bytes_stream();

    // === 固定频率上报所需的共享状态 ===
    let received = Arc::new(AtomicU64::new(0));
    let running = Arc::new(AtomicBool::new(true));
    let mut reporter: Option<tokio::task::JoinHandle<()>> = None;

    // 有总长度时，开启固定频率 reporter（100ms 一次）
    if total > 0 {
        let app_cloned = app.clone();
        let name_s = name.to_string();
        let rec = Arc::clone(&received);
        let run = Arc::clone(&running);

        const REPORT_INTERVAL_MS: u64 = 50; // 固定频率（可调）

        reporter = Some(tokio::spawn(async move {
            let mut last_pct: u8 = 0;
            let _ = app_cloned.emit(
                EVT_DOWNLOAD_PROGRESS,
                ProgressPayload {
                    name: name_s.clone(),
                    progress: 0,
                },
            );

            let mut ticker = time::interval(TokioDuration::from_millis(REPORT_INTERVAL_MS));
            ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

            loop {
                ticker.tick().await;
                if !run.load(Ordering::Relaxed) {
                    break;
                }

                let r = rec.load(Ordering::Relaxed);
                let pct = ((r as f64 / total as f64) * 100.0).floor() as u8;

                if pct != last_pct {
                    last_pct = pct;
                    let _ = app_cloned.emit(
                        EVT_DOWNLOAD_PROGRESS,
                        ProgressPayload {
                            name: name_s.clone(),
                            progress: pct as u32,
                        },
                    );
                }
                if pct >= 100 {
                    break;
                }
            }

            // 防止竞态，最后再补一次
            let r = rec.load(Ordering::Relaxed);
            let pct = ((r as f64 / total as f64) * 100.0).floor() as u8;
            if pct < 100 {
                let _ = app_cloned.emit(
                    EVT_DOWNLOAD_PROGRESS,
                    ProgressPayload {
                        name: name_s.clone(),
                        progress: pct as u32,
                    },
                );
            }
        }));
    } else {
        // 无 Content-Length：无法计算百分比；这里不启 reporter，
        // 由前端用“processing/不确定进度”展示更合理（或你另行设计心跳事件）
    }

    // === 下载循环：只负责累加字节 ===
    let mut acc: u64 = 0;
    while let Some(chunk_res) = stream.next().await {
        let bytes = chunk_res?;
        file.write_all(&bytes).await?;
        acc += bytes.len() as u64;

        if total > 0 {
            received.store(acc, Ordering::Relaxed);
        }
    }

    // 收尾：确保 reporter 看见 100%
    if total > 0 {
        received.store(total, Ordering::Relaxed);
    }
    running.store(false, Ordering::Relaxed);

    if let Some(h) = reporter {
        let _ = h.await;
    }

    // 一定发最终 100
    let _ = app.emit(
        EVT_DOWNLOAD_PROGRESS,
        ProgressPayload {
            name: name.to_string(),
            progress: 100,
        },
    );

    update_frp_version(app, state, name, Option::from(true), Option::from(false))?;

    Ok(())
}
