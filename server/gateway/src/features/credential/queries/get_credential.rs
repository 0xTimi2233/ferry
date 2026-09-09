//! 查询凭证详情

/// 入站数据
#[derive(Debug, Clone)]
pub struct GetCredential {
    pub credential_id: String,
}
