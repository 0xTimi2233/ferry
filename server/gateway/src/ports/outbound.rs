//! 出站端口
//!
//! 写侧经聚合根，读侧直接取投影，两侧接口分开。

pub mod events;
pub mod query;
pub mod repository;

pub use events::*;
pub use query::*;
pub use repository::*;

/// 出站依赖的失败原因
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortError {
    /// 存储不可用
    Storage(String),
    /// 上游请求失败，带上已拿到的响应状态码
    Upstream { status: u16, detail: String },
    /// 加解密失败
    Cipher(String),
}

impl std::fmt::Display for PortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(reason) => write!(f, "存储失败：{reason}"),
            Self::Upstream { status, detail } => write!(f, "上游返回 {status}：{detail}"),
            Self::Cipher(reason) => write!(f, "加解密失败：{reason}"),
        }
    }
}

impl std::error::Error for PortError {}
