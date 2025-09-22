use std::sync::Arc;
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Result as StoreResult, Store, StoreExt};

pub const CONFIG_KEY: &str = "config";
pub const SETTINGS_KEY: &str = "settings";
pub const LOADED_FLAG_KEY: &str = "__loaded_flag__";

pub fn store(app: &AppHandle, file: &str) -> StoreResult<Arc<Store<Wry>>> {
    app.store(file)
}