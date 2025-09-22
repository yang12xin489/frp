use super::types::AuthType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auth {
    pub method: AuthType,
    pub token: String,
}
impl Default for Auth {
    fn default() -> Self {
        Self {
            method: AuthType::Token,
            token: String::new(),
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrpcConfig {
    pub server_addr: String,
    pub server_port: u16,
    pub auth: Auth,
    pub web_server: WebServer,
    pub proxies: Vec<crate::domain::proxy::Proxy>,
    pub switch: Switches,
}

impl Default for FrpcConfig {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1".into(),
            server_port: 7000,
            auth: Auth::default(),
            web_server: WebServer {
                addr: "127.0.0.1".into(),
                port: 7400,
                user: String::new(),
                password: String::new(),
            },
            proxies: vec![],
            r#switch: Switches {
                auth: false,
                web_server: false,
            },
        }
    }
}
