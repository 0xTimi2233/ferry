//! 值对象

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CredentialId(String);

impl CredentialId {
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AliasName(String);

impl AliasName {
    pub fn new(raw: impl Into<String>) -> Result<Self, InvalidValue> {
        let raw = raw.into();
        if raw.trim().is_empty() {
            return Err(InvalidValue::Blank("别名"));
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Provider(String);

impl Provider {
    pub fn new(raw: impl Into<String>) -> Result<Self, InvalidValue> {
        let raw = raw.into();
        if raw.trim().is_empty() {
            return Err(InvalidValue::Blank("上游"));
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn supports_subscription(&self) -> bool {
        matches!(self.as_str(), "OpenAI" | "Anthropic")
    }

    pub fn default_protocol(&self) -> Protocol {
        match self.as_str() {
            "Anthropic" => Protocol::AnthropicMessages,
            "Gemini" => Protocol::Gemini,
            _ => Protocol::OpenAiChat,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    OpenAiChat,
    OpenAiResponses,
    AnthropicMessages,
    Gemini,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UpstreamModelId(String);

impl UpstreamModelId {
    pub fn new(raw: impl Into<String>) -> Result<Self, InvalidValue> {
        let raw = raw.into();
        if raw.trim().is_empty() {
            return Err(InvalidValue::Blank("上游模型"));
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AliasTargetId(String);

impl AliasTargetId {
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GroupId(String);

impl GroupId {
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 数值越小越优先
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Priority(u16);

impl Priority {
    pub fn new(raw: u16) -> Result<Self, InvalidValue> {
        if raw == 0 {
            return Err(InvalidValue::OutOfRange("优先级"));
        }
        Ok(Self(raw))
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

impl Default for Priority {
    fn default() -> Self {
        Self(1)
    }
}

/// 零表示不参与轮询
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weight(u32);

impl Weight {
    pub fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub fn value(&self) -> u32 {
        self.0
    }

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
    pub fn new(raw: impl Into<String>) -> Result<Self, InvalidValue> {
        let raw = raw.into();
        if raw.trim().is_empty() {
            return Err(InvalidValue::Blank("密钥"));
        }
        Ok(Self(raw))
    }

    pub fn expose(&self) -> &str {
        &self.0
    }

    /// 保留首尾各四位，其余以圆点替代
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Ready,
    Cooling {
        reason: String,
        recover_at: chrono::DateTime<chrono::Utc>,
    },
    Disabled,
    Failed {
        reason: String,
    },
}

impl HealthStatus {
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Ready)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionStrategy {
    #[default]
    RoundRobin,
    Weighted,
    FillFirst,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

impl TokenUsage {
    pub fn total(&self) -> u64 {
        self.input + self.output + self.cache_read + self.cache_write
    }
}

/// 以微美元为单位，避免浮点累加误差
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    pub micro_usd: i64,
}

impl Money {
    pub fn from_micro_usd(micro_usd: i64) -> Self {
        Self { micro_usd }
    }

    pub fn zero() -> Self {
        Self { micro_usd: 0 }
    }

    pub fn add(&self, other: Self) -> Self {
        Self {
            micro_usd: self.micro_usd + other.micro_usd,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidValue {
    Blank(&'static str),
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
