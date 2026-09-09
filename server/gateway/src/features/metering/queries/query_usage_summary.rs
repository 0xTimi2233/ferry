//! 查询用量汇总

/// 统计指标
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageMetric {
    Requests,
    Tokens,
    Cost,
}

/// 入站数据
#[derive(Debug, Clone)]
pub struct QueryUsageSummary {
    pub days: u32,
    pub metric: UsageMetric,
}
