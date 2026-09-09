//! 入站端口
//!
//! 入站适配器只负责把协议负载翻译成 Command 或 Query，再由处理器执行；
//! 处理器签名即入站端口的契约，切片只实现自己的处理器。

use std::future::Future;

use crate::domain::errors::{CatalogError, CredentialError, RelayError, SettingsError};
use crate::ports::outbound::PortError;

/// 一个用例的处理结果
pub type UseCaseResult<T> = Result<T, UseCaseError>;

/// 用例失败，区分入站输入非法、领域规则拒绝与出站依赖失败
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UseCaseError {
    /// 入站输入不合法
    InvalidInput(String),
    /// 领域规则拒绝
    Domain(String),
    /// 资源不存在
    NotFound(String),
    /// 未授权
    Unauthorized,
    /// 出站依赖失败
    Port(PortError),
}

impl std::fmt::Display for UseCaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(reason) => write!(f, "输入不合法：{reason}"),
            Self::Domain(reason) => write!(f, "业务规则拒绝：{reason}"),
            Self::NotFound(name) => write!(f, "{name} 不存在"),
            Self::Unauthorized => f.write_str("未授权"),
            Self::Port(inner) => inner.fmt(f),
        }
    }
}

impl std::error::Error for UseCaseError {}

impl From<PortError> for UseCaseError {
    fn from(value: PortError) -> Self {
        Self::Port(value)
    }
}

impl From<CredentialError> for UseCaseError {
    fn from(value: CredentialError) -> Self {
        use CredentialError as E;
        match value {
            E::NotFound(name) => Self::NotFound(format!("凭证 {name}")),
            E::Duplicated(name) => Self::Domain(format!("凭证 {name} 已存在")),
            E::Invalid(inner) => Self::InvalidInput(inner.to_string()),
            other => Self::Domain(other.to_string()),
        }
    }
}

impl From<CatalogError> for UseCaseError {
    fn from(value: CatalogError) -> Self {
        use CatalogError as E;
        match value {
            E::NotFound(name) | E::GroupNotFound(name) => Self::NotFound(name),
            E::Duplicated(name) => Self::Domain(format!("别名 {name} 已存在")),
            E::Invalid(inner) => Self::InvalidInput(inner.to_string()),
            other => Self::Domain(other.to_string()),
        }
    }
}

impl From<RelayError> for UseCaseError {
    fn from(value: RelayError) -> Self {
        match value {
            RelayError::Unauthorized => Self::Unauthorized,
            RelayError::AliasNotFound(name) => Self::NotFound(format!("模型 {name}")),
            other => Self::Domain(other.to_string()),
        }
    }
}

impl From<SettingsError> for UseCaseError {
    fn from(value: SettingsError) -> Self {
        match value {
            SettingsError::Invalid(inner) => Self::InvalidInput(inner.to_string()),
            other => Self::Domain(other.to_string()),
        }
    }
}

/// 入站端口：一个用例一个实现，依赖通过构造注入
pub trait UseCase<In>: Send + Sync {
    type Out;

    fn execute(&self, input: In) -> impl Future<Output = UseCaseResult<Self::Out>> + Send;
}
