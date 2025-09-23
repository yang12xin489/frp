use crate::state::{AppState, FrpcProcState};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, State, WindowEvent};

mod errors;
mod events;
mod state;
mod domain {
    pub mod active_frp;
    pub mod config;
    pub mod progress_payload;
    pub mod proxy;
    pub mod types;
    pub mod version;
}
mod infra {
    pub mod archive;
    pub mod http;
    pub mod paths;
    pub mod store;
}
pub mod services {
    pub mod config_service;
    pub mod runner;
    pub mod version_service;
    pub mod watchdog_service;
}
mod api {
    pub mod config_api;
    pub mod proxies_api;
    pub mod runner_api;
    pub mod settings_api;
    pub mod versions_api;
}

#[cfg(target_os = "macos")]
use runtime::ActivationPolicy;

fn kill_child_if_any(st: &State<'_, FrpcProcState>) {
    if let Ok(mut g) = st.child.lock() {
        if let Some(ch) = g.as_mut() {
            // 强制兜底一把（不会重复触发 close 事件没关系）
            let _ = ch.kill();
            let _ = ch.wait();
        }
        *g = None;
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close(); // 不退出应用
                let _ = window.hide(); // 仅隐藏窗口

                #[cfg(target_os = "macos")]
                {
                    let _ = window
                        .app_handle()
                        .set_activation_policy(ActivationPolicy::Accessory);
                }

                // —— Windows/Linux: 从任务栏移除
                #[cfg(not(target_os = "macos"))]
                {
                    let _ = window.set_skip_taskbar(true);
                }
            }
        })
        .setup(|app| {
            let state: State<AppState> = app.handle().state();
            services::config_service::loaded_from_store(&app.handle(), &state)?;
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }

            let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        #[cfg(target_os = "macos")]
                        {
                            let _ = app.set_activation_policy(ActivationPolicy::Regular);
                        }

                        if let Some(win) = app.get_webview_window("main") {
                            #[allow(unused_must_use)]
                            {
                                #[cfg(not(target_os = "macos"))]
                                win.set_skip_taskbar(false); // Windows/Linux 恢复任务栏

                                win.show();
                                win.unminimize();
                                win.set_focus();
                            }
                        }
                    }
                    "quit" => {
                        if let Some(st) = app.try_state::<FrpcProcState>() {
                            kill_child_if_any(&st);
                        }
                        app.exit(0)
                    }
                    _ => {}
                })
                .build(app)?;

            let mut exe = std::env::current_exe().expect("current_exe");
            exe.set_file_name(if cfg!(windows) {
                "frp-client-watchdog.exe"
            } else {
                "frp-client-watchdog"
            });
            let mut child = std::process::Command::new(&exe)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map_err(|e| {
                    anyhow::anyhow!("spawn reaper failed: {e} (path: {})", exe.display())
                })?;

            if let Some(tx) = child.stdin.take() {
                *app.state::<FrpcProcState>().watchdog.lock().unwrap() = Some(tx);
            }
            Ok(())
        })
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .manage(FrpcProcState::default())
        .invoke_handler(tauri::generate_handler![
            api::config_api::load_config,
            api::config_api::save_server,
            api::config_api::export_toml,
            api::config_api::export_toml_to_file,
            api::config_api::save_now,
            api::proxies_api::save_proxy,
            api::proxies_api::remove_proxy,
            api::proxies_api::load_proxies,
            api::proxies_api::get_proxy,
            api::versions_api::get_versions,
            api::versions_api::get_active_version,
            api::versions_api::activate_version,
            api::versions_api::deactivate_version,
            api::versions_api::delete_version,
            api::versions_api::download_version,
            api::runner_api::start_frpc,
            api::runner_api::stop_frpc,
            api::runner_api::frpc_status,
            api::settings_api::set_setting,
            api::settings_api::get_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
