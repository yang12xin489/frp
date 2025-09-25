use crate::domain::config::FrpcConfig;
use crate::state::{AppState, FrpcProcState};
use crate::{
    errors::Result,
    infra::{
        paths::{app_config_dir, CONFIG_TOML_FILE, STORE_FILE},
        store::{store, CONFIG_KEY, LOADED_FLAG_KEY, SETTINGS_KEY},
    },
};
use serde_json::Value;
use tauri::AppHandle;

pub fn loaded_from_store(app: &AppHandle, state: &AppState) -> Result<()> {
    // 先用读锁看是否已加载
    if state
        .read()
        .settings
        .get(LOADED_FLAG_KEY)
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return Ok(());
    }

    // 未加载：先做 IO（不持锁）
    let st = store(app, STORE_FILE).map_err(|e| crate::errors::AppError::Store(e.to_string()))?;
    let cfg_val: Option<Value> = st.get(CONFIG_KEY);
    let cfg: FrpcConfig = cfg_val
        .map(|json| serde_json::from_value(json).unwrap_or_default())
        .unwrap_or_default();

    let settings_obj = st
        .get(SETTINGS_KEY)
        .and_then(|v: Value| v.as_object().cloned())
        .unwrap_or_default();

    // 再拿写锁，只做内存赋值与打标
    {
        let mut g = state.write();
        g.config = cfg;
        g.settings = settings_obj;
        g.settings.insert(LOADED_FLAG_KEY.into(), Value::Bool(true));
    }
    Ok(())
}

pub fn save_now(app: &AppHandle, state: &AppState) -> Result<()> {
    let st = store(app, STORE_FILE).map_err(|e| crate::errors::AppError::Store(e.to_string()))?;
    let g = state.read();
    st.set(CONFIG_KEY, serde_json::to_value(&g.config)?);
    st.set(SETTINGS_KEY, Value::Object(g.settings.clone()));
    st.save()
        .map_err(|e| crate::errors::AppError::Store(e.to_string()))?;
    Ok(())
}

pub fn export_toml_to_file(
    app: &AppHandle,
    state: &AppState,
    proc_state: &FrpcProcState,
) -> Result<String> {
    let cfg = state.read().config.clone();
    let dto = cfg.to_export(proc_state);
    let toml_str = toml::to_string_pretty(&dto)?;
    let dir = app_config_dir(app);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(CONFIG_TOML_FILE);
    std::fs::write(&path, toml_str)?;
    Ok(path.display().to_string())
}

pub fn save_server_config(
    app: &AppHandle,
    state: &AppState,
    frpc_config: FrpcConfig,
) -> Result<()> {
    {
        let mut g = state.write();
        let old_proxies = std::mem::take(&mut g.config.proxies);
        g.config = FrpcConfig {
            proxies: old_proxies,
            ..frpc_config
        };
    };
    save_now(app, state)?;
    Ok(())
}
