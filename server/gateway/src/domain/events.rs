//! 领域事件

use crate::domain::values::{CredentialId, HealthStatus, Money, TokenUsage};

/// 领域内发布的状态变更
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    /// 凭证已登记
    CredentialRegistered {
        /// 凭证标识
        id: CredentialId,
        /// 凭证名称
        name: String,
        /// 上游
        provider: String,
    },
    /// 凭证健康状态已变更
    CredentialHealthChanged {
        /// 凭证标识
        id: CredentialId,
        /// 变更后的状态
        status: HealthStatus,
    },
    /// 凭证已删除
    CredentialDeleted {
        /// 凭证标识
        id: CredentialId,
    },
    /// 一次调用的用量已记录
    UsageRecorded {
        /// 凭证标识
        credential_id: CredentialId,
        /// 模型别名
        alias: String,
        /// token 用量
        tokens: TokenUsage,
        /// 折算金额
        cost: Money,
        /// 上游是否成功
        succeeded: bool,
    },
}

impl DomainEvent {
    /// 事件名，用于日志与订阅匹配
    pub fn name(&self) -> &'static str {
        match self {
            Self::CredentialRegistered { .. } => "CredentialRegistered",
            Self::CredentialHealthChanged { .. } => "CredentialHealthChanged",
            Self::CredentialDeleted { .. } => "CredentialDeleted",
            Self::UsageRecorded { .. } => "UsageRecorded",
        }
    }
}
