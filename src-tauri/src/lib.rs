use crate::state::{AppState, FrpcProcState};
use tauri::Manager;

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
mod services {
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
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                //window.close_devtools();
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
