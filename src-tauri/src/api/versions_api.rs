use crate::domain::active_frp::ActiveFrp;
use crate::domain::version::FrpVersion;
use crate::services::version_service;
use crate::state::{AppState, FrpcProcState};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn get_versions(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<FrpVersion>, String> {
    Ok(version_service::get_versions(&app, &state).await?)
}

#[tauri::command]
pub fn get_active_version(
    _app: AppHandle,
    state: State<AppState>,
) -> Result<Option<ActiveFrp>, String> {
    Ok(version_service::get_active(&state))
}

#[tauri::command]
pub async fn activate_version(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    version_service::activate(&app, &state, &name).map_err(Into::into)
}

#[tauri::command]
pub async fn deactivate_version(
    app: AppHandle,
    state: State<'_, AppState>,
    proc_state: State<'_, FrpcProcState>,
    name: String,
) -> Result<(), String> {
    version_service::deactivate(&app, &state, &proc_state, &name).map_err(Into::into)
}

#[tauri::command]
pub async fn delete_version(
    app: AppHandle,
    state: State<'_, AppState>,
    proc_state: State<'_, FrpcProcState>,
    name: String,
) -> Result<(), String> {
    version_service::delete(&app, &state, &proc_state, &name).map_err(Into::into)
}

#[tauri::command]
pub async fn download_version(app: AppHandle, name: String, url: String) -> Result<(), String> {
    version_service::download(&app, &name, &url)
        .await
        .map_err(Into::into)
}
