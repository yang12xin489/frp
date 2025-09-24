use super::types::AuthType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Auth {
    pub method: AuthType,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebServer {
    pub addr: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Switches {
    pub auth: bool,
    pub web_server: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FrpcConfig {
    pub server_addr: String,
    pub server_port: u16,
    pub auth: Auth,
    pub web_server: WebServer,
    pub proxies: Vec<crate::domain::proxy::Proxy>,
    pub switch: Switches,
}