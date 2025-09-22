use crate::domain::config::FrpcConfig;
use serde_json::{Map, Value};
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

// 进程句柄状态
use std::process::Child;
use std::sync::Mutex;

#[derive(Default)]
pub struct FrpcProcState {
    pub child: Arc<Mutex<Option<Child>>>,
}
