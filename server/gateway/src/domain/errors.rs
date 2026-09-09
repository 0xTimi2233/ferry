//! 领域错误

use crate::domain::values::InvalidValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialError {
    Duplicated(String),
    UpstreamUnsupported(String),
    UpstreamLacksSubscription(String),
    AuthorizationExpired,
    AuthorizationInvalid,
    NotAvailable(String),
    Referenced(String),
    NotFound(String),
    ModelNotOffered(String),
    EmptySelection,
    Invalid(InvalidValue),
}

impl std::fmt::Display for CredentialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Duplicated(name) => write!(f, "凭证 {name} 已存在"),
            Self::UpstreamUnsupported(name) => write!(f, "上游 {name} 不受支持"),
            Self::UpstreamLacksSubscription(name) => write!(f, "上游 {name} 不支持订阅授权"),
            Self::AuthorizationExpired => f.write_str("授权已过期"),
            Self::AuthorizationInvalid => f.write_str("授权状态无效"),
            Self::NotAvailable(name) => write!(f, "凭证 {name} 当前不可用"),
            Self::Referenced(name) => write!(f, "凭证 {name} 仍被模型别名引用"),
            Self::NotFound(name) => write!(f, "凭证 {name} 不存在"),
            Self::ModelNotOffered(name) => write!(f, "模型 {name} 不在上游清单中"),
            Self::EmptySelection => f.write_str("至少保留一个模型"),
            Self::Invalid(inner) => inner.fmt(f),
        }
    }
}

impl std::error::Error for CredentialError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogError {
    Duplicated(String),
    NotFound(String),
    CredentialNotFound(String),
    CredentialUnavailable(String),
    Invalid(InvalidValue),
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Duplicated(name) => write!(f, "模型别名 {name} 已存在"),
            Self::NotFound(name) => write!(f, "模型别名 {name} 不存在"),
            Self::CredentialNotFound(id) => write!(f, "凭证 {id} 不存在"),
            Self::CredentialUnavailable(name) => write!(f, "凭证 {name} 不可用"),
            Self::Invalid(inner) => inner.fmt(f),
        }
    }
}

impl std::error::Error for CatalogError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayError {
    AliasNotFound(String),
    Unauthorized,
    AllUnavailable {
        attempted: Vec<String>,
        recover_at: Option<chrono::DateTime<chrono::Utc>>,
    },
    UpstreamFailed {
        provider: String,
        status: u16,
    },
}

impl std::fmt::Display for RelayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AliasNotFound(name) => write!(f, "模型 {name} 不存在"),
            Self::Unauthorized => f.write_str("未授权"),
            Self::AllUnavailable { attempted, .. } => {
                write!(f, "全部凭证不可用，已尝试 {}", attempted.join("、"))
            }
            Self::UpstreamFailed { provider, status } => {
                write!(f, "上游 {provider} 返回 {status}")
            }
        }
    }
}

impl std::error::Error for RelayError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    RetentionInvalid(String),
    Invalid(InvalidValue),
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RetentionInvalid(value) => write!(f, "保留期 {value} 取值非法"),
            Self::Invalid(inner) => inner.fmt(f),
        }
    }
}

impl std::error::Error for SettingsError {}
