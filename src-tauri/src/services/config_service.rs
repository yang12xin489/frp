use crate::domain::config::FrpcConfig;
use crate::state::AppState;
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

pub fn export_toml_string(state: &AppState) -> Result<String> {
    use serde_json::Value as JV;
    let cfg = state.read().config.clone();

    let method_str = serde_json::to_value(&cfg.auth.method)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let mut root = serde_json::Map::new();
    root.insert("serverAddr".into(), JV::String(cfg.server_addr));
    root.insert("serverPort".into(), JV::Number(cfg.server_port.into()));

    let mut web_map = serde_json::Map::new();
    web_map.insert("addr".into(), JV::String(cfg.web_server.addr));
    web_map.insert("port".into(), JV::Number(cfg.web_server.port.into()));
    if cfg.switch.web_server {
        if !cfg.web_server.user.is_empty() {
            web_map.insert("user".into(), JV::String(cfg.web_server.user));
        }
        if !cfg.web_server.password.is_empty() {
            web_map.insert("password".into(), JV::String(cfg.web_server.password));
        }
    }
    root.insert("webServer".into(), JV::Object(web_map));

    if cfg.switch.auth {
        let mut a_map = serde_json::Map::new();
        a_map.insert("method".into(), JV::String(method_str));
        a_map.insert("token".into(), JV::String(cfg.auth.token));
        root.insert("auth".into(), JV::Object(a_map));
    }

    if !cfg.proxies.is_empty() {
        let arr: Vec<JV> = cfg
            .proxies
            .iter()
            .filter(|p| p.enable)
            .map(|p| {
                let mut v = serde_json::to_value(p).unwrap_or(JV::Null);
                if let JV::Object(ref mut map) = v {
                    map.remove("id");
                    map.remove("enable");
                    map.remove("switch");
                }
                v
            })
            .collect();
        root.insert("proxies".into(), JV::Array(arr));
    }

    let value = JV::Object(root);
    Ok(toml::to_string(&value)?)
}

pub fn export_toml_to_file(app: &AppHandle, state: &AppState) -> Result<String> {
    let toml = export_toml_string(state)?;
    let dir = app_config_dir(app);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(CONFIG_TOML_FILE);
    std::fs::write(&path, toml)?;
    Ok(path.display().to_string())
}

pub fn save_server_config(app: &AppHandle, state: &AppState, partial: FrpcConfig) -> Result<()> {
    let mut g = state.write();
    let old = &mut g.config;
    if !partial.server_addr.is_empty() {
        old.server_addr = partial.server_addr;
    }
    old.server_port = partial.server_port;
    old.auth = partial.auth;
    old.switch = partial.switch;
    old.web_server = partial.web_server;
    save_now(app, state)?;
    Ok(())
}
