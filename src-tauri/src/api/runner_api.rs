use crate::state::FrpcProcState;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn start_frpc(
    app: AppHandle,
    state: State<'_, FrpcProcState>,
    exe_path: String,
    cfg_path: String,
) -> Result<u32, String> {
    crate::services::runner::start(&app, &state, &exe_path, &cfg_path)
}

#[tauri::command]
pub async fn stop_frpc(state: State<'_, FrpcProcState>) -> Result<(), String> {
    crate::services::runner::stop(&state)
}

#[tauri::command]
pub async fn frpc_status(state: State<'_, FrpcProcState>) -> Result<bool, String> {
    crate::services::runner::is_running(&state)
}
