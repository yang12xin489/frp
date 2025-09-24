use crate::domain::config::FrpcConfig;
use crate::{services::config_service as svc, state::AppState};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn load_config(state: State<AppState>) -> Result<FrpcConfig, String> {
    Ok(state.read().config.clone())
}

#[tauri::command]
pub fn save_server(
    app: AppHandle,
    state: State<AppState>,
    partial: FrpcConfig,
) -> Result<(), String> {
    svc::save_server_config(&app, &state, partial);
    Ok(())
}

#[tauri::command]
pub fn save_now(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    svc::save_now(&app, &state).map_err(Into::into)
}
