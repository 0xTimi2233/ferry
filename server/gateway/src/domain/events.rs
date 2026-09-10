//! 领域事件

use crate::domain::values::{CredentialId, GroupId, HealthStatus, Money, TokenUsage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    CredentialRegistered {
        id: CredentialId,
        name: String,
        provider: String,
    },
    CredentialHealthChanged {
        id: CredentialId,
        status: HealthStatus,
    },
    CredentialDeleted {
        id: CredentialId,
    },
    CredentialGroupRemoved {
        id: GroupId,
    },
    UsageRecorded {
        credential_id: CredentialId,
        alias: String,
        tokens: TokenUsage,
        /// 未定价时为空，不计价凭证记零
        cost: Option<Money>,
        succeeded: bool,
        latency_ms: u64,
        /// 失败原因，成功时为 None
        failure_reason: Option<String>,
        affinity_hit: bool,
    },
}

impl DomainEvent {
    pub fn name(&self) -> &'static str {
        match self {
            Self::CredentialRegistered { .. } => "CredentialRegistered",
            Self::CredentialHealthChanged { .. } => "CredentialHealthChanged",
            Self::CredentialDeleted { .. } => "CredentialDeleted",
            Self::CredentialGroupRemoved { .. } => "CredentialGroupRemoved",
            Self::UsageRecorded { .. } => "UsageRecorded",
        }
    }
}
