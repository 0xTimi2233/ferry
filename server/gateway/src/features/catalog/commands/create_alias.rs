//! 创建别名

/// 入站数据
#[derive(Debug, Clone)]
pub struct CreateAlias {
    pub name: String,
    pub group_id: String,
    pub upstream_model: String,
    pub weight: u32,
    pub priority: u16,
}

/// 入站数据：为已存在的别名追加目标
#[derive(Debug, Clone)]
pub struct AppendAliasTarget {
    pub name: String,
    pub group_id: String,
    pub upstream_model: String,
    pub weight: u32,
    pub priority: u16,
}
