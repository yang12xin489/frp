use crate::state::{AppState, FrpcProcState};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn start_frpc(
    app: AppHandle,
    frpc_proc_state: State<'_, FrpcProcState>,
    state: State<'_, AppState>,
) -> Result<u32, String> {
    crate::services::runner::start(&app, &state, &frpc_proc_state).await
}

#[tauri::command]
pub async fn stop_frpc(app: AppHandle, state: State<'_, FrpcProcState>) -> Result<(), String> {
    crate::services::runner::stop(&app, &state)
}

#[tauri::command]
pub async fn frpc_status(state: State<'_, FrpcProcState>) -> Result<bool, String> {
    crate::services::runner::is_running(&state)
}