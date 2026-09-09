//! 启用或禁用凭证

/// 入站数据
#[derive(Debug, Clone)]
pub struct ChangeCredentialHealth {
    pub credential_id: String,
    pub enabled: bool,
}
