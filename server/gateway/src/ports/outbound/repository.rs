//! 写端口

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::aggregates::{Alias, Credential, CredentialGroup, Settings, UsageRecord};
use crate::domain::values::{
    AliasName, CredentialId, GroupId, PendingAuthorization, Secret, UpstreamModelId,
};

use super::PortError;

#[async_trait]
pub trait CredentialRepository: Send + Sync {
    async fn find(&self, id: &CredentialId) -> Result<Option<Credential>, PortError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Credential>, PortError>;
    async fn list_all(&self) -> Result<Vec<Credential>, PortError>;
    async fn save(&self, credential: &Credential) -> Result<(), PortError>;
    async fn delete(&self, id: &CredentialId) -> Result<(), PortError>;
}

#[async_trait]
pub trait AliasRepository: Send + Sync {
    async fn find(&self, name: &AliasName) -> Result<Option<Alias>, PortError>;
    async fn list_all(&self) -> Result<Vec<Alias>, PortError>;
    async fn save(&self, alias: &Alias) -> Result<(), PortError>;
    async fn delete(&self, name: &AliasName) -> Result<(), PortError>;
    async fn count_targets_to(&self, group: &GroupId) -> Result<u64, PortError>;
}

#[async_trait]
pub trait CredentialGroupRepository: Send + Sync {
    async fn find(&self, id: &GroupId) -> Result<Option<CredentialGroup>, PortError>;
    async fn list_all(&self) -> Result<Vec<CredentialGroup>, PortError>;
    async fn save(&self, group: &CredentialGroup) -> Result<(), PortError>;
}

#[async_trait]
pub trait UsageRepository: Send + Sync {
    async fn append(&self, record: &UsageRecord) -> Result<(), PortError>;
}

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn load(&self) -> Result<Option<Settings>, PortError>;
    async fn save(&self, settings: &Settings) -> Result<(), PortError>;
}

#[async_trait]
pub trait PendingAuthorizationRepository: Send + Sync {
    async fn save(&self, pending: &PendingAuthorization) -> Result<(), PortError>;
    /// 取出并消费，同一状态标识只能成功一次
    async fn take(&self, state: &str) -> Result<Option<PendingAuthorization>, PortError>;
}

/// 一次上游调用的入参
#[derive(Debug, Clone)]
pub struct UpstreamRequest {
    pub protocol: crate::domain::values::Protocol,
    pub upstream_model: UpstreamModelId,
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
}

/// 换取的订阅令牌
#[derive(Debug, Clone)]
pub struct ExchangedToken {
    pub access_token: Secret,
    pub refresh_token: Secret,
    pub expires_at: DateTime<Utc>,
    pub account: Option<String>,
}

#[async_trait]
pub trait UpstreamClient: Send + Sync {
    async fn fetch_models(
        &self,
        credential: &Credential,
    ) -> Result<Vec<UpstreamModelId>, PortError>;

    async fn invoke(
        &self,
        credential: &Credential,
        request: UpstreamRequest,
    ) -> Result<UpstreamResponse, PortError>;

    /// 由授权码换取令牌
    async fn exchange_authorization_code(
        &self,
        provider: &str,
        code: &str,
        code_verifier: &str,
    ) -> Result<ExchangedToken, PortError>;

    async fn refresh_token(&self, credential: &Credential) -> Result<ExchangedToken, PortError>;
}

/// 凭据加解密，主密钥来自信封加密
pub trait SecretCipher: Send + Sync {
    fn encrypt(&self, plaintext: &str) -> Result<String, PortError>;
    fn decrypt(&self, ciphertext: &str) -> Result<String, PortError>;
}
