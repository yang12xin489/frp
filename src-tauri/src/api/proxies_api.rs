use crate::domain::proxy::Proxy;
use crate::domain::types::ProxyType;
use crate::{services::config_service as svc, state::AppState};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn save_proxy(app: AppHandle, state: State<AppState>, proxy: Proxy) -> Result<(), String> {
    svc::loaded_from_store(&app, &state)?;
    {
        let mut g = state.write();
        let list = &mut g.config.proxies;
        if let Some(idx) = list
            .iter()
            .position(|p| p.name == proxy.name && p.type_ == proxy.type_)
        {
            list[idx] = proxy;
        } else {
            list.push(proxy);
        }
    }
    svc::save_now(&app, &state)?;
    Ok(())
}

#[tauri::command]
pub fn remove_proxy(
    app: AppHandle,
    state: State<AppState>,
    name: String,
    type_: ProxyType,
) -> Result<bool, String> {
    svc::loaded_from_store(&app, &state)?;
    let mut removed = false;
    {
        let mut g = state.write();
        let before = g.config.proxies.len();
        g.config
            .proxies
            .retain(|p| !(p.name == name && p.type_ == type_));
        removed = g.config.proxies.len() != before;
    }
    let _ = svc::save_now(&app, &state);
    Ok(removed)
}

#[tauri::command]
pub fn load_proxies(app: AppHandle, state: State<AppState>) -> Result<Vec<Proxy>, String> {
    svc::loaded_from_store(&app, &state)?;
    Ok(state.read().config.proxies.clone())
}

#[tauri::command]
pub fn get_proxy(
    app: AppHandle,
    state: State<AppState>,
    name: String,
    type_: ProxyType,
) -> Result<Option<Proxy>, String> {
    svc::loaded_from_store(&app, &state)?;
    Ok(state
        .read()
        .config
        .proxies
        .iter()
        .find(|p| p.name == name && p.type_ == type_)
        .cloned())
}
