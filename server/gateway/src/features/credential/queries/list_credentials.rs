//! 查询凭证列表

/// 入站数据
#[derive(Debug, Clone, Default)]
pub struct ListCredentials {
    pub provider: Option<String>,
}
