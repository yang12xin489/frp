use crate::state::{AppState, FrpcProcState};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, RunEvent, State, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_shell::ShellExt;

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
}
mod api {
    pub mod config_api;
    pub mod proxies_api;
    pub mod runner_api;
    pub mod settings_api;
    pub mod versions_api;
}
#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;

fn kill_child_if_any(st: &State<'_, FrpcProcState>) {
    if let Ok(mut g) = st.child.lock() {
        if let Some(ch) = g.as_mut() {
            let _ = ch.kill();
            let _ = ch.wait();
        }
        *g = None;
    }
}

fn show_window(app: &tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);

    if let Some(win) = app.get_webview_window("main") {
        #[cfg(not(target_os = "macos"))]
        let _ = win.set_skip_taskbar(false); // 之前隐藏过任务栏的话
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    } else {
        if let Ok(win) =
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("")
                .visible(false)
                .center()
                .build()
        {
            #[cfg(not(target_os = "macos"))]
            let _ = win.set_skip_taskbar(false);
            let _ = win.show();
            let _ = win.set_focus();
        }
    }
}

fn hide_window(win: &tauri::window::Window) {
    let _ = win.hide();

    #[cfg(target_os = "macos")]
    {
        let _ = win
            .app_handle()
            .set_activation_policy(ActivationPolicy::Accessory);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = win.set_skip_taskbar(true);
    }
}

static ALLOW_EXIT: AtomicBool = AtomicBool::new(false);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                hide_window(window)
            }
        })
        .setup(|app| {
            let state: State<AppState> = app.handle().state();
            services::config_service::loaded_from_store(&app.handle(), &state)?;

            let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click { button, .. } if button == MouseButton::Left => {
                        show_window(tray.app_handle());
                    }
                    _ => {}
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_window(app),
                    "quit" => {
                        if let Some(st) = app.try_state::<FrpcProcState>() {
                            kill_child_if_any(&st);
                        }
                        ALLOW_EXIT.store(true, Ordering::SeqCst);
                        app.exit(0)
                    }
                    _ => {}
                })
                .build(app)?;

            let watchdog_command = app.app_handle().shell().sidecar("frpc-watchdog").unwrap();
            let (_rx, child) = watchdog_command.spawn().expect("Failed to spawn watchdog");
            *app.state::<FrpcProcState>().watchdog.lock().unwrap() = Some(child);
            Ok(())
        })
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .manage(FrpcProcState::default())
        .invoke_handler(tauri::generate_handler![
            api::config_api::load_config,
            api::config_api::save_server,
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
        .build(tauri::generate_context!())
        .expect("failed to build frpc app")
        .run(|_app, event| match event {
            RunEvent::ExitRequested { api, .. } => {
                if !ALLOW_EXIT.load(Ordering::SeqCst) {
                    api.prevent_exit();
                }
            }
            _ => {}
        });
}
