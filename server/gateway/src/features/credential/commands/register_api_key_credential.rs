//! 注册密钥凭证

/// 入站数据
#[derive(Debug, Clone)]
pub struct RegisterApiKeyCredential {
    pub name: String,
    pub provider: String,
    pub secret: String,
}
