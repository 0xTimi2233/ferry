//! 管理访问密钥与服务端设置

/// 入站数据
#[derive(Debug, Clone, Default)]
pub struct ManageSettings {
    pub retention_days: Option<u32>,
    pub session_affinity: Option<bool>,
    pub rotate_access_key: bool,
}
