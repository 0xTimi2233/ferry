//! 拉取上游模型清单

/// 入站数据
#[derive(Debug, Clone)]
pub struct FetchUpstreamModels {
    pub credential_id: String,
}

/// 出站数据
#[derive(Debug, Clone)]
pub struct OfferedModels {
    pub models: Vec<String>,
    pub kept: Vec<String>,
}
