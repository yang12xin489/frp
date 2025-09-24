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

    // 拿到配置
    let cfg = state.read().config.clone();

    // 1) 先把整个配置转成 serde_json::Value
    let mut root = serde_json::to_value(&cfg)?;


    // 2) 处理 proxies：只保留启用的，并移除不需要的键
    if let Some(arr) = obj.get_mut("proxies").and_then(|x| x.as_array_mut()) {
        arr.retain(|p| p.get("enable").and_then(|b| b.as_bool()) == Some(true));

        for p in arr {
            if let Some(m) = p.as_object_mut() {
                m.remove("id");
                m.remove("enable");
                m.remove("switch");
            }
        }
        // 如果过滤后为空就删掉整个键（可选）
        if arr.is_empty() {
            obj.remove("proxies");
        }
    }

    // 3) 根据 switch 删掉 auth / webServer（注意 key 是驼峰：webServer）
    if let Some(sw) = obj.get("switch").and_then(|v| v.as_object()) {
        if sw.get("auth").and_then(|v| v.as_bool()) != Some(true) {
            obj.remove("auth");
        }
        if sw.get("webServer").and_then(|v| v.as_bool()) != Some(true) {
            obj.remove("webServer");
        }
    }

    // 如果最终文件不需要写出 switch，本行放开
    // obj.remove("switch");

    // 4) 直接把 root（就是 serde_json::Value）序列化为 TOML 字符串
    // serde 的 toml 库支持对任意 Serialize 做 to_string
    Ok(toml::to_string_pretty(&root)?)
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
