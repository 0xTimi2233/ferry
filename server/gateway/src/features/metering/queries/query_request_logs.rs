//! 查询请求日志

/// 入站数据
#[derive(Debug, Clone, Default)]
pub struct QueryRequestLogs {
    pub alias: Option<String>,
    pub credential: Option<String>,
    pub only_failed: bool,
}
