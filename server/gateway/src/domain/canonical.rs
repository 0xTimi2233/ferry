//! 统一表示
//!
//! 基准协议各有各的字段与语义，入站与出站都先翻译成本模块的表达再跨边界，领域因此不感知
//! 任何上游协议的字段。翻译由适配器完成，「我们表达不了」的失败归 [`TranslationError`]，
//! 上游报文读不进来走的是上游失败。

use crate::domain::values::{Protocol, TokenUsage};

/// 消息角色
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// 消息内容块
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentBlock {
    Text(String),
    Image {
        media_type: String,
        data: Vec<u8>,
    },
    ToolCall {
        id: String,
        name: String,
        arguments: String,
    },
    ToolResult {
        call_id: String,
        content: String,
    },
}

/// 一条消息，按顺序承载若干内容块
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub role: Role,
    pub blocks: Vec<ContentBlock>,
}

impl Message {
    pub fn text(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            blocks: vec![ContentBlock::Text(content.into())],
        }
    }
}

/// 工具定义，参数结构以 JSON Schema 原文承载，不做二次建模
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub schema: String,
}

/// 采样参数，比例类取值以千分比表达，避免浮点
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Sampling {
    pub temperature_milli: Option<u16>,
    pub top_p_milli: Option<u16>,
    pub max_output_tokens: Option<u32>,
    pub stop: Vec<String>,
}

/// 一次模型调用的统一请求
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalRequest {
    pub messages: Vec<Message>,
    pub tools: Vec<ToolDefinition>,
    pub sampling: Sampling,
    /// 客户端声明的流式意图。端口的方法选择必须与它一致，
    /// 即 `stream` 为真时只能走 `invoke_stream`。
    pub stream: bool,
}

/// 结束原因
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCall,
    Filtered,
    Failed,
}

/// 一次模型调用的统一响应
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalResponse {
    pub message: Message,
    pub finish: FinishReason,
    pub usage: TokenUsage,
}

/// 一次模型调用的用例入参：路由信封加统一表示
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvokeCommand {
    /// 客户端请求的模型名，决定路由到哪个别名
    pub alias: String,
    /// 客户端使用的协议，决定响应如何翻译回客户端
    pub protocol: Protocol,
    /// 会话标识，缺省表示本次调用不参与会话粘性
    pub session_id: Option<String>,
    pub request: CanonicalRequest,
}

/// 流式响应的单个事件，逐块转发给客户端
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalEvent {
    /// 增量内容。`index` 标识所属内容块，同一块的增量按序到达；
    /// 首片携带完整标识，后续片只带增量文本。
    Delta { index: u32, block: ContentBlock },
    /// 结束，携带结束原因与累计用量
    Finished {
        finish: FinishReason,
        usage: TokenUsage,
    },
    /// 流内错误，发完即结束流。已发生的用量一并带出，供落账侧记账
    Failed { reason: String, usage: TokenUsage },
}

/// 统一表示与某一侧协议之间「表达不了」的失败。两个变体都不表示上游报文读不进来：
/// 统一表示送成上游协议时目标协议不支持某项能力，报 `UnsupportedByTarget`，此时还没向上游发起请求；
/// 统一表示回译成客户端协议时客户端协议表达不了响应内容，报 `UnsupportedResponseByTarget`。
/// 上游报文无法解析成统一表示不算翻译失败，它走上游失败的通道，错误里的上游字段因此有值。
/// 两个变体最终都由 `UseCaseError::Translation` 映射到客户端可见的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationError {
    /// 目标协议表达不了统一表示里要送出的能力
    UnsupportedByTarget { feature: String, protocol: Protocol },
    /// 上游响应无法回译为客户端协议
    UnsupportedResponseByTarget { feature: String, protocol: Protocol },
}

impl std::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedByTarget { feature, protocol } => {
                write!(f, "目标协议 {} 不支持{feature}", protocol.label())
            }
            Self::UnsupportedResponseByTarget { feature, protocol } => {
                write!(f, "上游响应无法回译为{}：{feature}", protocol.label())
            }
        }
    }
}

impl std::error::Error for TranslationError {}
