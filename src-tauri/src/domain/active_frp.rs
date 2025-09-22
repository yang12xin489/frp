use serde::{Deserialize, Serialize};

pub const SETTINGS_ACTIVE_KEY: &str = "frp_active_version";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveFrp {
    // 资产文件名（archive）
    pub name: String,
    // 压缩包绝对路径
    pub archive_path: String,
    // 解压目录绝对路径
    pub unpack_dir: String,
    // frpc 可执行文件绝对路径
    pub exe_path: String,
    // ISO-8601 时间戳
    pub activated_at: String,
}
