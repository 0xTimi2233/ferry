//! 发起模型调用

/// 入站数据
#[derive(Debug, Clone)]
pub struct InvokeModel {
    pub alias: String,
    pub protocol: crate::domain::values::Protocol,
    pub stream: bool,
    pub body: Vec<u8>,
}
