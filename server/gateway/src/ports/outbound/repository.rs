//! 写端口

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::aggregates::{Alias, Credential, CredentialGroup, Settings, UsageRecord};
use crate::domain::values::{
    AliasName, CredentialId, GroupId, PendingAuthorization, Secret, UpstreamModelId,
};

use super::PortError;

/// 凭证仓储：按标识与名称查凭证，保存为整体替换
#[async_trait]
pub trait CredentialRepository: Send + Sync {
    async fn find(&self, id: &CredentialId) -> Result<Option<Credential>, PortError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Credential>, PortError>;
    async fn list_all(&self) -> Result<Vec<Credential>, PortError>;
    async fn save(&self, credential: &Credential) -> Result<(), PortError>;
    async fn delete(&self, id: &CredentialId) -> Result<(), PortError>;
}

/// 别名仓储：按名称查别名，`count_targets_to` 供账号组移除前判断引用
#[async_trait]
pub trait AliasRepository: Send + Sync {
    async fn find(&self, name: &AliasName) -> Result<Option<Alias>, PortError>;
    async fn list_all(&self) -> Result<Vec<Alias>, PortError>;
    async fn save(&self, alias: &Alias) -> Result<(), PortError>;
    async fn delete(&self, name: &AliasName) -> Result<(), PortError>;
    /// 统计指向该账号组的别名目标数量，供别名侧在账号组被移除后清理
    async fn count_targets_to(&self, group: &GroupId) -> Result<u64, PortError>;
}

/// 账号组仓储：按上游标识存取，组内凭证清空时删除
#[async_trait]
pub trait CredentialGroupRepository: Send + Sync {
    async fn find(&self, id: &GroupId) -> Result<Option<CredentialGroup>, PortError>;
    async fn list_all(&self) -> Result<Vec<CredentialGroup>, PortError>;
    async fn save(&self, group: &CredentialGroup) -> Result<(), PortError>;
    async fn delete(&self, id: &GroupId) -> Result<(), PortError>;
}

/// 会话与凭证的绑定
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionBinding {
    pub session_id: String,
    pub credential_id: CredentialId,
    pub expires_at: DateTime<Utc>,
}

/// 会话绑定仓储：同一会话至多一个绑定，删除幂等
#[async_trait]
pub trait SessionBindingRepository: Send + Sync {
    async fn find(&self, session_id: &str) -> Result<Option<SessionBinding>, PortError>;
    async fn save(&self, binding: &SessionBinding) -> Result<(), PortError>;
    async fn remove(&self, session_id: &str) -> Result<(), PortError>;
}

/// 用量仓储：只追加不修改
#[async_trait]
pub trait UsageRepository: Send + Sync {
    async fn append(&self, record: &UsageRecord) -> Result<(), PortError>;
}

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn load(&self) -> Result<Option<Settings>, PortError>;
    async fn save(&self, settings: &Settings) -> Result<(), PortError>;
}

/// 访问密钥校验：哈希与校验语义由适配器实现，明文不落盘
pub trait AccessKeyVerifier: Send + Sync {
    /// 生成新密钥明文，仅返回一次
    fn generate(&self) -> String;
    fn hash(&self, plaintext: &str) -> Result<String, PortError>;
    fn verify(&self, plaintext: &str, hash: &str) -> Result<bool, PortError>;
}

/// 待授权仓储：`take` 原子取出并消费，同一状态标识只能成功一次
#[async_trait]
pub trait PendingAuthorizationRepository: Send + Sync {
    async fn save(&self, pending: &PendingAuthorization) -> Result<(), PortError>;
    /// 取出并消费，同一状态标识只能成功一次
    async fn take(&self, state: &str) -> Result<Option<PendingAuthorization>, PortError>;
}

/// 账号组的轮询位置，由外部存储持有，保证重启与并发下轮换不丢位
#[async_trait]
pub trait SelectionCursorRepository: Send + Sync {
    /// 原子推进指定账号组的轮询位置并返回本次应使用的下标，下标必小于 `len`
    async fn advance(&self, group: &GroupId, len: usize) -> Result<usize, PortError>;
}

/// 一次上游调用的分层超时，四者各自独立，不用单一 deadline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpstreamTimeouts {
    /// 建立连接的上限
    pub connect_seconds: u64,
    /// 等待首字节的上限
    pub first_byte_seconds: u64,
    /// 两次读取之间的空闲上限
    pub read_seconds: u64,
    /// 整次调用的上限
    pub total_seconds: u64,
}

/// 一次上游调用的入参，明文密钥只在此处出现
#[derive(Debug, Clone)]
pub struct UpstreamCall {
    pub provider: crate::domain::values::Provider,
    pub secret: Secret,
    pub protocol: crate::domain::values::Protocol,
    pub upstream_model: UpstreamModelId,
    pub timeouts: UpstreamTimeouts,
    pub body: Vec<u8>,
    pub stream: bool,
}

/// 上游响应
#[derive(Debug, Clone)]
pub struct UpstreamResponse {
    pub status: u16,
    pub body: Vec<u8>,
    /// 上游给出的重试等待秒数
    pub retry_after_seconds: Option<u64>,
    /// 配额耗尽且不可重试
    pub quota_exhausted: bool,
    /// 上游水位头，原样透传给客户端
    pub rate_limit_headers: Vec<(String, String)>,
}

/// 换取的订阅令牌
#[derive(Debug, Clone)]
pub struct ExchangedToken {
    pub access_token: Secret,
    pub refresh_token: Secret,
    pub expires_at: DateTime<Utc>,
    pub account: Option<String>,
}

/// 上游客户端：按 `UpstreamCall.provider` 选择端点，`invoke` 不自行重试，重试由调度层决定
#[async_trait]
pub trait UpstreamClient: Send + Sync {
    async fn fetch_models(&self, call: UpstreamCall) -> Result<Vec<UpstreamModelId>, PortError>;

    async fn invoke(&self, call: UpstreamCall) -> Result<UpstreamResponse, PortError>;

    /// 由授权码换取令牌
    async fn exchange_authorization_code(
        &self,
        provider: &str,
        code: &str,
        code_verifier: &str,
    ) -> Result<ExchangedToken, PortError>;

    async fn refresh_token(
        &self,
        credential_id: &CredentialId,
        refresh_token: Secret,
    ) -> Result<ExchangedToken, PortError>;
}

/// 信封加密的密文：数据密钥被包裹，密文与包裹后的密钥分开存放
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedSecret {
    pub wrapped_key: String,
    pub nonce: String,
    pub ciphertext: String,
}

/// 凭据加解密：`seal` 为每次调用生成新的数据密钥，`open` 用包裹的密钥解出明文
pub trait SecretCipher: Send + Sync {
    fn seal(&self, plaintext: &str) -> Result<SealedSecret, PortError>;
    fn open(&self, sealed: &SealedSecret) -> Result<Secret, PortError>;
}
