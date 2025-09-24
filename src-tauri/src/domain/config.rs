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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FrpcConfigFrom<'a> {
    server_addr: &'a str,
    server_port: u16,

    #[serde(skip_serializing_if = "Option::is_none")]
    auth: Option<&'a Auth>,

    #[serde(skip_serializing_if = "Option::is_none")]
    web_server: Option<&'a WebServer>,

    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    proxies: &'a [crate::domain::proxy::Proxy],

    #[serde(skip_serializing)]
    switch: &'a Switches,
}

impl FrpcConfig {
    pub fn from(&self) -> FrpcConfigFrom<'_> {
        FrpcConfigFrom {
            server_addr: &self.server_addr,
            server_port: self.server_port,
            auth: self.switch.auth.then(|| &self.auth),
            web_server: self.switch.web_server.then(|| &self.web_server),
            proxies: &self.proxies,
            switch: &self.switch,
        }
    }
}
