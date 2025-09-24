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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HttpsProxy {
    #[serde(flatten)]
    pub common: ProxyCommon,
    pub subdomain: String,
    pub custom_domains: Vec<String>,
    pub locations: Vec<String>,
    pub http_user: String,
    pub http_password: String,
    pub switch: HttpSwitch,
}
