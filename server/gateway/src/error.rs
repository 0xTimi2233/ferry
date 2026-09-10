//! 应用层错误分档
//!
//! 切片把领域错误与出站端口错误映射到本模块的枚举，入站适配器据此产出响应。
//! 每个领域错误与端口错误都必须显式映射，不得用通配臂收尾：新增变体时编译器应当报错，
//! 而不是静默降级为一个语义更弱的分档。

use crate::domain::canonical::TranslationError;
use crate::domain::errors::{CatalogError, CredentialError, RelayError, SettingsError};
use crate::ports::outbound::PortError;

/// 一个用例的处理结果
pub type UseCaseResult<T> = Result<T, UseCaseError>;

/// 出错实体的类型，用于生成对外错误文本
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entity {
    Alias,
    CredentialGroup,
    Credential,
    Model,
}

impl Entity {
    fn label(self) -> &'static str {
        match self {
            Self::Alias => "别名",
            Self::CredentialGroup => "账号组",
            Self::Credential => "凭证",
            Self::Model => "模型",
        }
    }
}

/// 用例失败，按入站适配器可映射的语义分档
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UseCaseError {
    /// 入站输入不合法
    InvalidInput(String),
    /// 业务规则拒绝
    Domain(String),
    /// 并发达到上限，属短时可重试
    ConcurrencyLimited { retry_after_seconds: u64 },
    /// 全部凭证不可用，恢复时间取其中最晚
    AllCredentialsUnavailable {
        attempted: Vec<String>,
        recover_at: Option<chrono::DateTime<chrono::Utc>>,
    },
    /// 资源不存在
    NotFound { entity: Entity, name: String },
    /// 未授权
    Unauthorized,
    /// 上游拒绝了本次调用。provider 为空表示端口层给不出上游名称，此时详情在 detail。
    UpstreamFailed {
        provider: String,
        status: Option<u16>,
        detail: String,
    },
    /// 统一表示与客户端协议或上游协议之间无法互译
    Translation(String),
    /// 网关自身失效，与上游无关
    Internal(String),
}

impl UseCaseError {
    pub fn not_found(entity: Entity, name: impl Into<String>) -> Self {
        Self::NotFound {
            entity,
            name: name.into(),
        }
    }

    pub fn upstream_failed(provider: impl Into<String>, status: u16) -> Self {
        Self::UpstreamFailed {
            provider: provider.into(),
            status: Some(status),
            detail: String::new(),
        }
    }
}

impl std::fmt::Display for UseCaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(reason) => write!(f, "输入不合法：{reason}"),
            Self::Domain(reason) => write!(f, "业务规则拒绝：{reason}"),
            Self::ConcurrencyLimited { .. } => f.write_str("并发已满"),
            Self::AllCredentialsUnavailable { attempted, .. } => {
                write!(f, "全部凭证不可用，已尝试 {}", attempted.join("、"))
            }
            Self::NotFound { entity, name } => write!(f, "{} {name} 不存在", entity.label()),
            Self::Unauthorized => f.write_str("未授权"),
            Self::UpstreamFailed {
                provider, detail, ..
            } if provider.is_empty() => {
                write!(f, "上游调用失败：{detail}")
            }
            Self::UpstreamFailed {
                provider,
                status: Some(status),
                ..
            } => write!(f, "上游 {provider} 返回 {status}"),
            Self::UpstreamFailed { provider, .. } => write!(f, "上游 {provider} 调用失败"),
            Self::Translation(reason) => write!(f, "协议转换失败：{reason}"),
            Self::Internal(reason) => write!(f, "网关内部失败：{reason}"),
        }
    }
}

impl std::error::Error for UseCaseError {}

impl From<PortError> for UseCaseError {
    fn from(value: PortError) -> Self {
        match value {
            PortError::Storage(reason) => Self::Internal(format!("存储失败：{reason}")),
            PortError::Cipher(reason) => Self::Internal(format!("加解密失败：{reason}")),
            PortError::Upstream(detail) => Self::UpstreamFailed {
                provider: String::new(),
                status: None,
                detail,
            },
        }
    }
}

impl From<TranslationError> for UseCaseError {
    fn from(value: TranslationError) -> Self {
        Self::Translation(value.to_string())
    }
}

impl From<CredentialError> for UseCaseError {
    fn from(value: CredentialError) -> Self {
        use CredentialError as E;
        match value {
            E::Duplicated(name) => Self::Domain(format!("凭证 {name} 已存在")),
            E::UpstreamUnsupported(name) => Self::InvalidInput(format!("上游 {name} 不受支持")),
            E::UpstreamLacksSubscription(name) => {
                Self::Domain(format!("上游 {name} 不支持订阅授权"))
            }
            E::AuthorizationExpired => Self::Domain("授权已过期".to_owned()),
            E::AuthorizationInvalid => Self::InvalidInput("授权状态无效".to_owned()),
            E::NotAvailable(name) => Self::Domain(format!("凭证 {name} 当前不可用")),
            E::NotFound(name) => Self::not_found(Entity::Credential, name),
            E::ModelNotOffered(name) => Self::Domain(format!("模型 {name} 不在上游清单中")),
            E::EmptySelection => Self::InvalidInput("至少保留一个模型".to_owned()),
            E::Invalid(inner) => Self::InvalidInput(inner.to_string()),
        }
    }
}

impl From<CatalogError> for UseCaseError {
    fn from(value: CatalogError) -> Self {
        use CatalogError as E;
        match value {
            E::Duplicated(name) => Self::Domain(format!("别名 {name} 已存在")),
            E::NotFound(name) => Self::not_found(Entity::Alias, name),
            E::GroupNotFound(id) => Self::not_found(Entity::CredentialGroup, id),
            E::CredentialUnavailable(name) => Self::Domain(format!("凭证 {name} 不可用")),
            E::Invalid(inner) => Self::InvalidInput(inner.to_string()),
        }
    }
}

impl From<RelayError> for UseCaseError {
    fn from(value: RelayError) -> Self {
        match value {
            RelayError::AliasNotFound(name) => Self::not_found(Entity::Model, name),
            RelayError::Unauthorized => Self::Unauthorized,
            RelayError::AllUnavailable {
                attempted,
                recover_at,
            } => Self::AllCredentialsUnavailable {
                attempted,
                recover_at,
            },
            RelayError::UpstreamFailed { provider, status } => {
                Self::upstream_failed(provider, status)
            }
        }
    }
}

impl From<SettingsError> for UseCaseError {
    fn from(value: SettingsError) -> Self {
        use SettingsError as E;
        match value {
            E::RetentionInvalid(value) => Self::InvalidInput(format!("保留期 {value} 取值非法")),
            E::Invalid(inner) => Self::InvalidInput(inner.to_string()),
        }
    }
}
