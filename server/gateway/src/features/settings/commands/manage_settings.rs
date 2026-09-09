//! 管理访问密钥与服务端设置

/// 入站数据
#[derive(Debug, Clone, Default)]
pub struct ManageSettings {
    pub retention_days: Option<u32>,
    pub session_affinity: Option<bool>,
    pub rotate_access_key: bool,
}

/// 出站数据
#[derive(Debug, Clone)]
pub struct SettingsOutcome {
    pub listen: String,
    pub masked_access_key: String,
    pub granularity: String,
    pub retention_days: u32,
    pub session_affinity: bool,
    /// 轮换时返回的新密钥明文，仅此一次
    pub rotated_access_key: Option<String>,
}
