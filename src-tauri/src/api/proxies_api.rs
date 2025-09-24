use crate::domain::proxy::Proxy;
use crate::{services::config_service as svc, state::AppState};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn load_proxies(state: State<AppState>) -> Result<Vec<Proxy>, String> {
    Ok(state.read().config.proxies.clone())
}

#[tauri::command]
pub fn get_proxy(state: State<AppState>, name: String) -> Result<Option<Proxy>, String> {
    Ok(state
        .read()
        .config
        .proxies
        .iter()
        .find(|p| p.name == name)
        .cloned())
}

#[tauri::command]
pub fn save_proxy(app: AppHandle, state: State<AppState>, proxy: Proxy) -> Result<(), String> {
    {
        let mut g = state.write();
        let list = &mut g.config.proxies;
        if let Some(idx) = list.iter().position(|p| p.id == proxy.id) {
            list[idx] = proxy;
        } else {
            list.push(proxy);
        }
    }
    svc::save_now(&app, &state)?;
    Ok(())
}

#[tauri::command]
pub fn remove_proxy(app: AppHandle, state: State<AppState>, id: String) -> Result<bool, String> {
    let removed = {
        let mut g = state.write();
        let before = g.config.proxies.len();
        g.config.proxies.retain(|p| !(p.id == id));
        g.config.proxies.len() != before
    };

    svc::save_now(&app, &state)?;
    Ok(removed)
}
