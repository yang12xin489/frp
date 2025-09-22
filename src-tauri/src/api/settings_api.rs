use crate::{services::config_service as svc, state::AppState};
use serde_json::Value;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn set_setting(
    app: AppHandle,
    state: State<AppState>,
    key: String,
    value: Value,
) -> Result<bool, String> {
    svc::loaded_from_store(&app, &state)?;
    {
        let mut g = state.write();
        g.settings.insert(key, value);
    }
    svc::save_now(&app, &state)?;
    Ok(true)
}

#[tauri::command]
pub fn get_setting(
    app: AppHandle,
    state: State<AppState>,
    key: String,
) -> Result<Option<Value>, String> {
    svc::loaded_from_store(&app, &state)?;
    Ok(state.read().settings.get(&key).cloned())
}
