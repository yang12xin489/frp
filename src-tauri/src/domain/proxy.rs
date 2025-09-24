use super::types::DomainType;
use serde::{Deserialize, Deserializer, Serialize};
use std::ops::{Deref, DerefMut};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HttpSwitch {
    pub domain: DomainType,
    pub auth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Proxy {
    Http(HttpProxy),
    Https(HttpsProxy),
}

impl Proxy {
    pub fn common(&self) -> &ProxyCommon {
        match self {
            Proxy::Http(p) => &p.common,
            Proxy::Https(p) => &p.common,
        }
    }
    pub fn common_mut(&mut self) -> &mut ProxyCommon {
        match self {
            Proxy::Http(p) => &mut p.common,
            Proxy::Https(p) => &mut p.common,
        }
    }
}

impl Deref for Proxy {
    type Target = ProxyCommon;
    fn deref(&self) -> &Self::Target {
        self.common()
    }
}
impl DerefMut for Proxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.common_mut()
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProxyExport {
    Http(HttpProxyExport),
    Https(HttpsProxyExport),
}

// 生成默认 id
fn gen_id() -> String {
    Uuid::new_v4().to_string()
}

// 如果是空串或只包含空白，则用默认 id；
// 如果字段缺失，依赖 #[serde(default = "gen_id")] 兜底。
fn empty_string_as_default_id<'de, D>(de: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    // 这里用 Option<String> 来兼容“字段存在/缺失”两种情况
    let opt = Option::<String>::deserialize(de)?;
    Ok(match opt {
        Some(s) if !s.trim().is_empty() => s,
        _ => gen_id(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyCommon {
    #[serde(default = "gen_id", deserialize_with = "empty_string_as_default_id")]
    pub id: String,
    pub name: String,
    pub enable: bool,
    #[serde(rename = "localIP", default)]
    pub local_ip: String,
    #[serde(rename = "localPort", default)]
    pub local_port: u16,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyCommonExport {
    name: String,
    local_ip: String,
    local_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HttpProxy {
    #[serde(flatten)]
    pub common: ProxyCommon,
    pub subdomain: String,
    pub custom_domains: Vec<String>,
    pub locations: Vec<String>,
    pub http_user: String,
    pub http_password: String,
    pub switch: HttpSwitch,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpProxyExport {
    #[serde(flatten)]
    pub common: ProxyCommonExport,
    pub subdomain: String,
    pub custom_domains: Vec<String>,
    pub locations: Vec<String>,
    pub http_user: String,
    pub http_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HttpsProxy {
    #[serde(flatten)]
    pub common: ProxyCommon,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpsProxyExport {
    #[serde(flatten)]
    common: ProxyCommonExport,
}

pub fn to_proxy_export(p: &Proxy) -> Option<ProxyExport> {
    use crate::domain::proxy::Proxy as P;
    match p {
        P::Http(h) => Some(ProxyExport::Http(HttpProxyExport {
            common: ProxyCommonExport {
                name: h.common.name.clone(),
                local_ip: h.common.local_ip.clone(),
                local_port: h.common.local_port,
            },
            subdomain: h.subdomain.clone(),
            custom_domains: h.custom_domains.clone(),
            locations: h.locations.clone(),
            http_user: h.http_user.clone(),
            http_password: h.http_password.clone(),
        })),
        P::Https(h) => Some(ProxyExport::Https(HttpsProxyExport {
            common: ProxyCommonExport {
                name: h.common.name.clone(),
                local_ip: h.common.local_ip.clone(),
                local_port: h.common.local_port,
            },
        })),
    }
}
