use super::types::AuthType;
use crate::domain::proxy::{to_proxy_export, ProxyExport};
use crate::state::FrpcProcState;
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct WebServerExport {
    addr: String,
    port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrpcConfigExport {
    server_addr: String,
    server_port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth: Option<Auth>,
    web_server: WebServerExport,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    proxies: Vec<ProxyExport>,
}

impl FrpcConfig {
    pub fn to_export(&self, proc_state: &FrpcProcState) -> FrpcConfigExport {
        let proxies = self
            .proxies
            .iter()
            .filter(|p| p.enable)
            .filter_map(|m| to_proxy_export(m, proc_state)) // -> Option<ProxyExport>
            .collect();

        let web_server = WebServerExport {
            addr: self.web_server.addr.clone(),
            port: self.web_server.port,
            user: self
                .switch
                .web_server
                .then(|| self.web_server.user.clone())
                .filter(|s| !s.is_empty()),
            password: self
                .switch
                .web_server
                .then(|| self.web_server.password.clone())
                .filter(|s| !s.is_empty()),
        };

        FrpcConfigExport {
            server_addr: self.server_addr.clone(),
            server_port: self.server_port,
            auth: self.switch.auth.then(|| self.auth.clone()),
            proxies,
            web_server,
        }
    }
}
