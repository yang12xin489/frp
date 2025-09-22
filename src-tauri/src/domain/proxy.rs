use super::types::{DomainType, ProxyType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HttpSwitch {
    pub domain: DomainType,
    pub auth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proxy {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: ProxyType,
    #[serde(default)]
    pub local_ip: String,
    #[serde(default)]
    pub local_port: u16,
    #[serde(default)]
    pub subdomain: String,
    #[serde(default)]
    pub custom_domains: Vec<String>,
    #[serde(default)]
    pub locations: Vec<String>,
    #[serde(default)]
    pub http_user: String,
    #[serde(default)]
    pub http_password: String,
    #[serde(default)]
    pub switch: HttpSwitch,
}

impl Default for Proxy {
    fn default() -> Self {
        Self {
            name: String::new(),
            type_: ProxyType::Http,
            local_ip: String::new(),
            local_port: 0,
            subdomain: String::new(),
            custom_domains: vec![],
            locations: vec![],
            http_user: String::new(),
            http_password: String::new(),
            switch: HttpSwitch::default(),
        }
    }
}
