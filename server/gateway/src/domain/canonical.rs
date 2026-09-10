//! 统一表示
//!
//! 基准协议各有各的字段与语义，入站与出站都先翻译成本模块的表达再跨边界，领域因此不感知
//! 任何上游协议的字段。翻译由适配器完成，翻译不了的部分由适配器报 [`TranslationError`]。

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
    /// 增量内容
    Delta(ContentBlock),
    /// 结束，携带结束原因与累计用量
    Finished {
        finish: FinishReason,
        usage: TokenUsage,
    },
    /// 流内错误，发完即结束流
    Failed { reason: String },
}

/// 统一表示到底层协议或从底层协议翻译失败
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationError {
    /// 目标协议不支持该能力
    UnsupportedByTarget { feature: String, protocol: Protocol },
}

impl std::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedByTarget { feature, protocol } => {
                write!(f, "目标协议 {protocol:?} 不支持{feature}")
            }
        }
    }
}

impl std::error::Error for TranslationError {}
