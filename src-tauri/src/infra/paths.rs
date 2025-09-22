use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub const STORE_FILE: &str = "frpc.json";
pub const CONFIG_TOML_FILE: &str = "frpc.toml";
pub const DOWNLOAD_ROOT: &str = "downloads";

pub const UNPACK_ROOT: &str = "unpacked";

pub fn app_config_dir(app: &AppHandle) -> PathBuf {
    app.path().app_config_dir().expect("app_config_dir")
}
pub fn app_data_dir(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().expect("app_data_dir")
}

pub fn get_download_dir(app: &AppHandle) -> std::io::Result<PathBuf> {
    let dir = app_data_dir(app).join(DOWNLOAD_ROOT);
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

pub fn archive_stem(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".tar.gz") {
        return name[..name.len() - ".tar.gz".len()].to_string();
    }
    if lower.ends_with(".tgz") {
        return name[..name.len() - ".tgz".len()].to_string();
    }
    if let Some(idx) = name.rfind('.') {
        return name[..idx].to_string();
    }
    name.to_string()
}

pub fn unpack_dir_for(app: &AppHandle, name: &str) -> PathBuf {
    app_data_dir(app)
        .join(DOWNLOAD_ROOT)
        .join(archive_stem(name))
}
