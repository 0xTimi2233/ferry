//! 配置账号组

/// 入站数据
#[derive(Debug, Clone)]
pub struct ConfigureCredentialGroup {
    pub group_id: String,
    pub strategy: String,
}
