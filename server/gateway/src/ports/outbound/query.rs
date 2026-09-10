//! 读端口
//!
//! 端口抽象的是外部依赖的调用方式，不是对外形状。视图类型由契约定义，端口只回传契约类型
//! 与外部存储自有类型；标识类入参使用领域值对象，在边界处完成校验。

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use contracts::gateway::v1::{
    AliasView, CredentialGroupView, CredentialView, Granularity, Money, RequestLogView,
    SettingsView, UsageBucketView,
};

use super::PortError;
use crate::domain::values::CredentialId;

/// 凭证读模型的筛选条件
#[derive(Debug, Clone, Default)]
pub struct CredentialFilter {
    pub provider: Option<String>,
}

/// 请求日志的筛选条件
#[derive(Debug, Clone, Default)]
pub struct RequestLogFilter {
    pub alias: Option<String>,
    pub credential: Option<String>,
    pub only_failed: bool,
}

/// 按凭证聚合的折算金额，由外部存储直接给出
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialCostView {
    pub credential_id: CredentialId,
    pub cost: Money,
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

/// 请求日志读模型：按时间倒序，支持按别名、凭证与失败筛选
#[async_trait]
pub trait RequestLogQuery: Send + Sync {
    async fn list(&self, filter: RequestLogFilter) -> Result<Vec<RequestLogView>, PortError>;
}

#[async_trait]
pub trait SettingsQuery: Send + Sync {
    async fn current(&self) -> Result<Option<SettingsView>, PortError>;
}
