//! 选择保留的模型

/// 入站数据
#[derive(Debug, Clone)]
pub struct UpdateCredentialModels {
    pub credential_id: String,
    pub models: Vec<String>,
}
