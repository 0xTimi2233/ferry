//! 读端口

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::aggregates::Granularity;
use crate::domain::values::{AliasName, CredentialId, HealthStatus, Money, Protocol, TokenUsage};

use super::PortError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialView {
    pub id: CredentialId,
    pub name: String,
    pub kind: CredentialKindView,
    pub provider: String,
    pub group: String,
    pub kept_model_count: usize,
    pub masked_secret: String,
    pub kept_models: Vec<String>,
    pub health: HealthStatus,
    pub month_cost: Option<Money>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialKindView {
    ApiKey,
    Subscription,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasTargetView {
    pub group: String,
    pub upstream: String,
    pub upstream_model: String,
    pub protocol: Protocol,
    pub weight: u32,
    pub priority: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasView {
    pub name: AliasName,
    pub targets: Vec<AliasTargetView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialGroupView {
    pub id: String,
    pub provider: String,
    pub strategy: String,
    pub credential_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageBucketView {
    pub day: String,
    pub requests: u64,
    pub tokens: TokenUsage,
    pub cost: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestLogView {
    pub at: DateTime<Utc>,
    pub alias: String,
    pub credential_name: String,
    pub tokens: TokenUsage,
    pub latency_ms: u64,
    pub succeeded: bool,
    pub failure_reason: Option<String>,
    /// 本次调用是否命中会话粘性
    pub affinity_hit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsView {
    pub listen: String,
    pub masked_access_key: String,
    pub granularity: Granularity,
    pub retention_days: u32,
    pub session_affinity: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CredentialFilter {
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RequestLogFilter {
    pub alias: Option<String>,
    pub credential: Option<String>,
    pub only_failed: bool,
}

/// 凭证读模型：列表按上游筛选，详情按标识取单条，密钥只以掩码返回
#[async_trait]
pub trait CredentialQuery: Send + Sync {
    async fn list(&self, filter: CredentialFilter) -> Result<Vec<CredentialView>, PortError>;
    async fn get(&self, id: &CredentialId) -> Result<Option<CredentialView>, PortError>;
}

/// 别名读模型：返回别名及其目标的完整投影
#[async_trait]
pub trait AliasQuery: Send + Sync {
    async fn list(&self) -> Result<Vec<AliasView>, PortError>;
}

#[async_trait]
pub trait CredentialGroupQuery: Send + Sync {
    async fn list(&self) -> Result<Vec<CredentialGroupView>, PortError>;
}

/// 用量读模型：按时间粒度聚合，或按凭证聚合折算金额
#[async_trait]
pub trait UsageQuery: Send + Sync {
    async fn buckets(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        granularity: Granularity,
    ) -> Result<Vec<UsageBucketView>, PortError>;

    async fn cost_by_credential(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<CredentialCostView>, PortError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialCostView {
    pub credential_id: CredentialId,
    pub cost: Money,
}

/// 请求日志读模型：按时间倒序，支持按别名、凭证与失败筛选
#[async_trait]
pub trait RequestLogQuery: Send + Sync {
    async fn list(&self, filter: RequestLogFilter) -> Result<Vec<RequestLogView>, PortError>;
}

#[async_trait]
pub trait SettingsQuery: Send + Sync {
    async fn current(&self) -> Result<Option<SettingsView>, PortError>;
}
