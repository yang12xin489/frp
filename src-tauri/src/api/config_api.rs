use crate::domain::config::FrpcConfig;
use crate::{services::config_service as svc, state::AppState};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn load_config(app: AppHandle, state: State<AppState>) -> Result<FrpcConfig, String> {
    svc::loaded_from_store(&app, &state)?;
    Ok(state.read().config.clone())
}

#[tauri::command]
pub fn save_server(
    app: AppHandle,
    state: State<AppState>,
    partial: FrpcConfig,
) -> Result<(), String> {
    svc::loaded_from_store(&app, &state)?;
    svc::update_server_config(&state, partial);
    svc::save_now(&app, &state)?;
    Ok(())
}

#[tauri::command]
pub fn export_toml(app: AppHandle, state: State<AppState>) -> Result<String, String> {
    svc::loaded_from_store(&app, &state)?;
    svc::export_toml_string(&state).map_err(Into::into)
}

#[tauri::command]
pub fn export_toml_to_file(app: AppHandle, state: State<AppState>) -> Result<String, String> {
    svc::loaded_from_store(&app, &state)?;
    svc::export_toml_to_file(&app, &state).map_err(Into::into)
}

#[tauri::command]
pub fn save_now(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    svc::save_now(&app, &state).map_err(Into::into)
}
