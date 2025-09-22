
pub const SETTINGS_VERSIONS_KEY: &str = "frp_versions";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrpVersion {
    pub id: u64,
    pub name: String,
    pub size: String,
    pub version: String,
    pub created_at: String,
    pub count: u64,
    pub url: String,
    pub exist: bool,
    pub active: bool,
}
