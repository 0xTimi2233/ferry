//! 发起模型调用
//!
//! 入参与出参由领域统一表示承载，见 ADR 0001，不在契约里重复定义。

pub use crate::domain::canonical::{
    CanonicalEvent, CanonicalRequest, CanonicalResponse, InvokeCommand, TranslationError,
};
