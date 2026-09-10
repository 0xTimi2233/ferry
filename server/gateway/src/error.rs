//! 应用层错误分档
//!
//! 切片把领域错误与出站端口错误映射到本模块的枚举，入站适配器据此产出响应。
//! 每个领域错误与端口错误都必须显式映射，不得用通配臂收尾：新增变体时编译器应当报错，
//! 而不是静默降级为一个语义更弱的分档。

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
    /// 上游拒绝了本次调用，携带上游名称供对外错误体使用
    UpstreamFailed { provider: String, status: u16 },
    /// 出站依赖失败
    Port(PortError),
}

impl UseCaseError {
    pub fn not_found(entity: Entity, name: impl Into<String>) -> Self {
        Self::NotFound {
            entity,
            name: name.into(),
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
            Self::UpstreamFailed { provider, status } => {
                write!(f, "上游 {provider} 返回 {status}")
            }
            Self::Port(inner) => inner.fmt(f),
        }
    }
}

impl std::error::Error for UseCaseError {}

impl From<PortError> for UseCaseError {
    fn from(value: PortError) -> Self {
        use PortError as E;
        let carried = match &value {
            E::Storage(reason) => E::Storage(reason.clone()),
            E::Upstream(reason) => E::Upstream(reason.clone()),
            E::Cipher(reason) => E::Cipher(reason.clone()),
        };
        Self::Port(carried)
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
                Self::UpstreamFailed { provider, status }
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
