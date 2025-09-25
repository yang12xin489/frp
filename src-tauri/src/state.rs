use crate::domain::config::FrpcConfig;
use serde_json::{Map, Value};
use std::process::Child;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct Inner {
    pub config: FrpcConfig,
    pub settings: Map<String, Value>,
}

#[derive(Clone, Default)]
pub struct AppState(pub Arc<RwLock<Inner>>);

impl AppState {
    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, Inner> {
        self.0.read().unwrap()
    }
    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, Inner> {
        self.0.write().unwrap()
    }
}

use crate::services::local_proxy::ProxySpec;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::process::CommandChild;
use tokio::task::JoinHandle;

#[derive(Default)]
pub struct FrpcProcState {
    pub child: Arc<Mutex<Option<Child>>>,
    pub watchdog: Arc<Mutex<Option<CommandChild>>>,
    pub proxy_specs: Arc<Mutex<Vec<ProxySpec>>>,
    pub shim_tasks: Mutex<Vec<JoinHandle<()>>>,
}

pub fn notify_watchdog(app: &AppHandle, msg: String) -> Result<(), std::io::Error> {
    let state: tauri::State<'_, FrpcProcState> = app.state();
    {
        let mut guard = state.watchdog.lock().unwrap();
        if let Some(child) = guard.as_mut() {
            let _ = child.write((msg + "\n").as_bytes());
        }
    }
    Ok(())
}
