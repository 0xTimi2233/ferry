//! 出站端口
//!
//! 写侧经聚合根，读侧直接取投影，两侧接口分开。

pub mod query;
pub mod repository;

pub use query::*;
pub use repository::*;

/// 出站依赖的失败原因
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortError {
    /// 存储不可用
    Storage(String),
    /// 上游请求失败
    Upstream(String),
    /// 加解密失败
    Cipher(String),
}

impl std::fmt::Display for PortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(reason) => write!(f, "存储失败：{reason}"),
            Self::Upstream(reason) => write!(f, "上游失败：{reason}"),
            Self::Cipher(reason) => write!(f, "加解密失败：{reason}"),
        }
    }
}

impl std::error::Error for PortError {}
