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
pub struct UpstreamRefView {
    pub upstream: String,
    pub upstream_model: String,
    pub protocol: Protocol,
    pub credential_name: String,
    pub weight: u32,
    pub priority: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasView {
    pub name: AliasName,
    pub strategy: String,
    pub refs: Vec<UpstreamRefView>,
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
    pub keyword: Option<String>,
    pub only_failed: bool,
}

#[async_trait]
pub trait CredentialQuery: Send + Sync {
    async fn list(&self, filter: CredentialFilter) -> Result<Vec<CredentialView>, PortError>;
    async fn get(&self, id: &CredentialId) -> Result<Option<CredentialView>, PortError>;
}

#[async_trait]
pub trait AliasQuery: Send + Sync {
    async fn list(&self) -> Result<Vec<AliasView>, PortError>;
}

#[async_trait]
pub trait UsageQuery: Send + Sync {
    async fn buckets(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        granularity: Granularity,
    ) -> Result<Vec<UsageBucketView>, PortError>;
}

#[async_trait]
pub trait RequestLogQuery: Send + Sync {
    async fn list(&self, filter: RequestLogFilter) -> Result<Vec<RequestLogView>, PortError>;
}

#[async_trait]
pub trait SettingsQuery: Send + Sync {
    async fn current(&self) -> Result<Option<SettingsView>, PortError>;
}
