//! 领域错误

use crate::domain::values::InvalidValue;

/// 凭证领域的失败原因
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialError {
    /// 同名凭证已存在
    Duplicated(String),
    /// 上游不在支持范围内
    UpstreamUnsupported(String),
    /// 上游不支持订阅授权
    UpstreamLacksSubscription(String),
    /// 待授权状态已过期
    AuthorizationExpired,
    /// 待授权状态不存在或已被消费
    AuthorizationInvalid,
    /// 凭证当前不可用
    NotAvailable(String),
    /// 凭证仍被模型别名引用
    Referenced(String),
    /// 凭证不存在
    NotFound(String),
    /// 待保留的模型不在上游清单中
    ModelNotOffered(String),
    /// 保留清单为空
    EmptySelection,
    /// 取值不合法
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

/// 模型目录领域的失败原因
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogError {
    /// 同名别名已存在
    Duplicated(String),
    /// 别名不存在
    NotFound(String),
    /// 引用的凭证不存在
    CredentialNotFound(String),
    /// 引用的凭证不可用
    CredentialUnavailable(String),
    /// 取值不合法
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

/// 转发与调度领域的失败原因
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayError {
    /// 模型别名不存在
    AliasNotFound(String),
    /// 访问密钥无效
    Unauthorized,
    /// 全部凭证不可用
    AllUnavailable {
        /// 尝试过的凭证名称
        attempted: Vec<String>,
        /// 预计恢复时间
        recover_at: Option<chrono::DateTime<chrono::Utc>>,
    },
    /// 上游返回错误
    UpstreamFailed {
        /// 上游名称
        provider: String,
        /// 状态码
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

/// 设置领域的失败原因
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    /// 保留期取值非法
    RetentionInvalid(String),
    /// 取值不合法
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
