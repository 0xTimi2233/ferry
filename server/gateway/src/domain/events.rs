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
        cost: Money,
        succeeded: bool,
        latency_ms: u64,
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
