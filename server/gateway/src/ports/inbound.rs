//! 入站端口
//!
//! 入站适配器只负责把协议负载翻译成 Command 或 Query，再由处理器执行；
//! 处理器签名即入站端口的契约，切片只实现自己的处理器。

use std::future::Future;

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

/// 入站端口：一个用例一个实现，依赖通过构造注入
pub trait UseCase<In>: Send + Sync {
    type Out;

    fn execute(&self, input: In) -> impl Future<Output = UseCaseResult<Self::Out>> + Send;
}
