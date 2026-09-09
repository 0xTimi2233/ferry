//! 创建别名

/// 入站数据
#[derive(Debug, Clone)]
pub struct CreateAlias {
    pub name: String,
    pub strategy: String,
    pub credential_id: String,
    pub upstream_model: String,
    pub weight: u32,
    pub priority: u16,
}
