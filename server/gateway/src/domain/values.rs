//! 值对象

use serde::{Deserialize, Serialize};

/// 凭证标识
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CredentialId(String);

impl CredentialId {
    /// 由字符串构造
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    /// 取字符串
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 模型别名
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AliasName(String);

impl AliasName {
    /// 由字符串构造，空白名称不合法
    pub fn new(raw: impl Into<String>) -> Result<Self, InvalidValue> {
        let raw = raw.into();
        if raw.trim().is_empty() {
            return Err(InvalidValue::Blank("模型别名"));
        }
        Ok(Self(raw))
    }

    /// 取字符串
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 上游标识
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Provider(String);

impl Provider {
    /// 由字符串构造，空白名称不合法
    pub fn new(raw: impl Into<String>) -> Result<Self, InvalidValue> {
        let raw = raw.into();
        if raw.trim().is_empty() {
            return Err(InvalidValue::Blank("上游"));
        }
        Ok(Self(raw))
    }

    /// 取字符串
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 是否支持订阅授权
    pub fn supports_subscription(&self) -> bool {
        matches!(self.as_str(), "OpenAI" | "Anthropic")
    }

    /// 该上游的默认协议
    pub fn default_protocol(&self) -> Protocol {
        match self.as_str() {
            "Anthropic" => Protocol::AnthropicMessages,
            "Gemini" => Protocol::Gemini,
            _ => Protocol::OpenAiChat,
        }
    }
}

/// 基准协议
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    /// OpenAI Chat Completions
    OpenAiChat,
    /// OpenAI Responses
    OpenAiResponses,
    /// Anthropic Messages
    AnthropicMessages,
    /// Gemini
    Gemini,
}

/// 上游模型标识
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UpstreamModelId(String);

impl UpstreamModelId {
    /// 由字符串构造，空白名称不合法
    pub fn new(raw: impl Into<String>) -> Result<Self, InvalidValue> {
        let raw = raw.into();
        if raw.trim().is_empty() {
            return Err(InvalidValue::Blank("上游模型"));
        }
        Ok(Self(raw))
    }

    /// 取字符串
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 上游引用标识
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UpstreamRefId(String);

impl UpstreamRefId {
    /// 由字符串构造
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    /// 取字符串
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 优先级，数值越小越优先
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Priority(u16);

impl Priority {
    /// 由数值构造，零不合法
    pub fn new(raw: u16) -> Result<Self, InvalidValue> {
        if raw == 0 {
            return Err(InvalidValue::OutOfRange("优先级"));
        }
        Ok(Self(raw))
    }

    /// 取数值
    pub fn value(&self) -> u16 {
        self.0
    }
}

impl Default for Priority {
    fn default() -> Self {
        Self(1)
    }
}

/// 权重，零表示不参与轮询
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weight(u32);

impl Weight {
    /// 由数值构造
    pub fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// 取数值
    pub fn value(&self) -> u32 {
        self.0
    }

    /// 是否参与轮询
    pub fn is_participating(&self) -> bool {
        self.0 > 0
    }
}

impl Default for Weight {
    fn default() -> Self {
        Self(1)
    }
}

/// 密钥明文，只在校验与写入时短暂持有
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Secret(String);

impl Secret {
    /// 由字符串构造，空白不合法
    pub fn new(raw: impl Into<String>) -> Result<Self, InvalidValue> {
        let raw = raw.into();
        if raw.trim().is_empty() {
            return Err(InvalidValue::Blank("密钥"));
        }
        Ok(Self(raw))
    }

    /// 取明文
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// 生成掩码，保留首尾各四位
    pub fn mask(&self) -> String {
        let chars: Vec<char> = self.0.chars().collect();
        if chars.len() <= 8 {
            return "•".repeat(chars.len());
        }
        let head: String = chars.iter().take(4).collect();
        let tail: String = chars.iter().skip(chars.len() - 4).collect();
        format!("{head}••••••{tail}")
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<secret>")
    }
}

/// 健康状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// 可用
    Ready,
    /// 冷却中
    Cooling {
        /// 冷却原因
        reason: String,
        /// 预计恢复时间
        recover_at: chrono::DateTime<chrono::Utc>,
    },
    /// 已禁用
    Disabled,
    /// 失效，需要人工处理
    Failed {
        /// 失效原因
        reason: String,
    },
}

impl HealthStatus {
    /// 是否可被调度选取
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Ready)
    }

    /// 是否为失效
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }
}

/// 选择策略
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionStrategy {
    /// 轮询
    #[default]
    RoundRobin,
    /// 加权轮询
    Weighted,
    /// 固定优先
    FillFirst,
}

/// token 用量
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUsage {
    /// 输入 token
    pub input: u64,
    /// 输出 token
    pub output: u64,
    /// 缓存命中读取
    pub cache_read: u64,
    /// 缓存写入
    pub cache_write: u64,
}

impl TokenUsage {
    /// 总 token
    pub fn total(&self) -> u64 {
        self.input + self.output + self.cache_read + self.cache_write
    }
}

/// 金额，以微美元为单位避免浮点误差
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    /// 微美元
    pub micro_usd: i64,
}

impl Money {
    /// 由微美元构造
    pub fn from_micro_usd(micro_usd: i64) -> Self {
        Self { micro_usd }
    }

    /// 零金额
    pub fn zero() -> Self {
        Self { micro_usd: 0 }
    }

    /// 累加
    pub fn add(&self, other: Self) -> Self {
        Self {
            micro_usd: self.micro_usd + other.micro_usd,
        }
    }
}

/// 值不合法
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidValue {
    /// 必填项为空白
    Blank(&'static str),
    /// 取值超出允许范围
    OutOfRange(&'static str),
}

impl std::fmt::Display for InvalidValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blank(field) => write!(f, "{field}不能为空"),
            Self::OutOfRange(field) => write!(f, "{field}取值非法"),
        }
    }
}

impl std::error::Error for InvalidValue {}
