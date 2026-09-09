//! 领域事件

use crate::domain::values::{CredentialId, HealthStatus, Money, TokenUsage};

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
    UsageRecorded {
        credential_id: CredentialId,
        pool: String,
        tokens: TokenUsage,
        cost: Money,
        succeeded: bool,
    },
}

impl DomainEvent {
    pub fn name(&self) -> &'static str {
        match self {
            Self::CredentialRegistered { .. } => "CredentialRegistered",
            Self::CredentialHealthChanged { .. } => "CredentialHealthChanged",
            Self::CredentialDeleted { .. } => "CredentialDeleted",
            Self::UsageRecorded { .. } => "UsageRecorded",
        }
    }
}
