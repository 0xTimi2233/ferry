//! 聚合根

use chrono::{DateTime, Utc};

use crate::domain::errors::{CatalogError, CredentialError, SettingsError};
use crate::domain::events::DomainEvent;
use crate::domain::values::{
    AliasName, CredentialId, HealthStatus, InvalidValue, Money, Priority, Protocol, Provider,
    Secret, SelectionStrategy, TokenUsage, UpstreamModelId, UpstreamRefId, Weight,
};

/// 凭证的凭据形态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialKind {
    /// 长期有效的密钥
    ApiKey {
        /// 密钥明文
        secret: Secret,
    },
    /// 会过期并需要刷新的订阅凭据
    Subscription {
        /// 访问令牌
        access_token: Secret,
        /// 刷新令牌
        refresh_token: Secret,
        /// 过期时刻
        expires_at: DateTime<Utc>,
        /// 账号标识
        account: Option<String>,
    },
}

impl CredentialKind {
    /// 是否订阅型
    pub fn is_subscription(&self) -> bool {
        matches!(self, Self::Subscription { .. })
    }
}

/// 凭证聚合根
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credential {
    id: CredentialId,
    name: String,
    provider: Provider,
    kind: CredentialKind,
    priority: Priority,
    weight: Weight,
    health: HealthStatus,
    offered_models: Vec<UpstreamModelId>,
    kept_models: Vec<UpstreamModelId>,
    next_refresh_at: Option<DateTime<Utc>>,
}

impl Credential {
    /// 注册一个密钥凭证
    pub fn register_api_key(
        id: CredentialId,
        name: impl Into<String>,
        provider: Provider,
        secret: Secret,
    ) -> Result<(Self, DomainEvent), CredentialError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(CredentialError::Invalid(InvalidValue::Blank("名称")));
        }
        let event = DomainEvent::CredentialRegistered {
            id: id.clone(),
            name: name.clone(),
            provider: provider.as_str().to_string(),
        };
        Ok((
            Self {
                id,
                name,
                provider,
                kind: CredentialKind::ApiKey { secret },
                priority: Priority::default(),
                weight: Weight::default(),
                health: HealthStatus::Ready,
                offered_models: Vec::new(),
                kept_models: Vec::new(),
                next_refresh_at: None,
            },
            event,
        ))
    }

    /// 登记一个订阅凭证
    pub fn register_subscription(
        id: CredentialId,
        name: impl Into<String>,
        provider: Provider,
        access_token: Secret,
        refresh_token: Secret,
        expires_at: DateTime<Utc>,
        account: Option<String>,
    ) -> Result<(Self, DomainEvent), CredentialError> {
        if !provider.supports_subscription() {
            return Err(CredentialError::UpstreamLacksSubscription(
                provider.as_str().to_string(),
            ));
        }
        let name = name.into();
        if name.trim().is_empty() {
            return Err(CredentialError::Invalid(InvalidValue::Blank("名称")));
        }
        let event = DomainEvent::CredentialRegistered {
            id: id.clone(),
            name: name.clone(),
            provider: provider.as_str().to_string(),
        };
        Ok((
            Self {
                id,
                name,
                provider,
                kind: CredentialKind::Subscription {
                    access_token,
                    refresh_token,
                    expires_at,
                    account,
                },
                priority: Priority::default(),
                weight: Weight::default(),
                health: HealthStatus::Ready,
                offered_models: Vec::new(),
                kept_models: Vec::new(),
                next_refresh_at: Some(expires_at),
            },
            event,
        ))
    }

    /// 标识
    pub fn id(&self) -> &CredentialId {
        &self.id
    }

    /// 名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 上游
    pub fn provider(&self) -> &Provider {
        &self.provider
    }

    /// 凭据形态
    pub fn kind(&self) -> &CredentialKind {
        &self.kind
    }

    /// 健康状态
    pub fn health(&self) -> &HealthStatus {
        &self.health
    }

    /// 优先级
    pub fn priority(&self) -> Priority {
        self.priority
    }

    /// 权重
    pub fn weight(&self) -> Weight {
        self.weight
    }

    /// 上游声明的模型清单
    pub fn offered_models(&self) -> &[UpstreamModelId] {
        &self.offered_models
    }

    /// 已保留的模型
    pub fn kept_models(&self) -> &[UpstreamModelId] {
        &self.kept_models
    }

    /// 下次刷新时刻
    pub fn next_refresh_at(&self) -> Option<DateTime<Utc>> {
        self.next_refresh_at
    }

    /// 是否可被调度选取
    pub fn is_available(&self) -> bool {
        self.health.is_available()
    }

    /// 密钥掩码
    pub fn masked_secret(&self) -> String {
        match &self.kind {
            CredentialKind::ApiKey { secret } => secret.mask(),
            CredentialKind::Subscription { .. } => "oauth •••• 已授权".to_string(),
        }
    }

    /// 记录上游声明的模型清单，已保留的模型自动标为选中
    pub fn record_offered_models(&mut self, models: Vec<UpstreamModelId>) {
        self.kept_models.retain(|m| models.contains(m));
        self.offered_models = models;
    }

    /// 更新保留的模型
    pub fn update_kept_models(
        &mut self,
        selected: Vec<UpstreamModelId>,
    ) -> Result<(), CredentialError> {
        if selected.is_empty() {
            return Err(CredentialError::EmptySelection);
        }
        for model in &selected {
            if !self.offered_models.contains(model) {
                return Err(CredentialError::ModelNotOffered(model.as_str().to_string()));
            }
        }
        self.kept_models = selected;
        Ok(())
    }

    /// 拉取模型前的前置校验
    pub fn ensure_available_for_fetch(&self) -> Result<(), CredentialError> {
        if self.is_available() {
            Ok(())
        } else {
            Err(CredentialError::NotAvailable(self.name.clone()))
        }
    }

    /// 禁用
    pub fn disable(&mut self) -> DomainEvent {
        self.health = HealthStatus::Disabled;
        DomainEvent::CredentialHealthChanged {
            id: self.id.clone(),
            status: self.health.clone(),
        }
    }

    /// 启用
    pub fn enable(&mut self) -> DomainEvent {
        self.health = HealthStatus::Ready;
        DomainEvent::CredentialHealthChanged {
            id: self.id.clone(),
            status: self.health.clone(),
        }
    }

    /// 标记冷却
    pub fn mark_cooling(
        &mut self,
        reason: impl Into<String>,
        recover_at: DateTime<Utc>,
    ) -> DomainEvent {
        self.health = HealthStatus::Cooling {
            reason: reason.into(),
            recover_at,
        };
        DomainEvent::CredentialHealthChanged {
            id: self.id.clone(),
            status: self.health.clone(),
        }
    }

    /// 标记失效
    pub fn mark_failed(&mut self, reason: impl Into<String>) -> DomainEvent {
        self.health = HealthStatus::Failed {
            reason: reason.into(),
        };
        DomainEvent::CredentialHealthChanged {
            id: self.id.clone(),
            status: self.health.clone(),
        }
    }

    /// 刷新订阅令牌，返回是否发生变更
    pub fn apply_refresh(
        &mut self,
        access_token: Secret,
        refresh_token: Secret,
        expires_at: DateTime<Utc>,
    ) -> DomainEvent {
        if let CredentialKind::Subscription {
            access_token: current_access,
            refresh_token: current_refresh,
            expires_at: current_expiry,
            ..
        } = &mut self.kind
        {
            *current_access = access_token;
            *current_refresh = refresh_token;
            *current_expiry = expires_at;
        }
        self.health = HealthStatus::Ready;
        self.next_refresh_at = Some(expires_at);
        DomainEvent::CredentialHealthChanged {
            id: self.id.clone(),
            status: self.health.clone(),
        }
    }

    /// 刷新失败后的处理，访问令牌仍有效时推迟下次刷新
    pub fn on_refresh_failed(
        &mut self,
        now: DateTime<Utc>,
        retry_at: DateTime<Utc>,
    ) -> Option<DomainEvent> {
        if let CredentialKind::Subscription { expires_at, .. } = &self.kind
            && *expires_at > now
        {
            self.next_refresh_at = Some(retry_at.min(*expires_at));
            return None;
        }
        self.next_refresh_at = None;
        Some(self.mark_failed("刷新失败"))
    }
}

/// 一个上游引用
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpstreamRef {
    id: UpstreamRefId,
    credential_id: CredentialId,
    upstream_model: UpstreamModelId,
    protocol: Protocol,
    weight: Weight,
    priority: Priority,
}

impl UpstreamRef {
    /// 构造一个上游引用
    pub fn new(
        id: UpstreamRefId,
        credential_id: CredentialId,
        upstream_model: UpstreamModelId,
        protocol: Protocol,
    ) -> Self {
        Self {
            id,
            credential_id,
            upstream_model,
            protocol,
            weight: Weight::default(),
            priority: Priority::default(),
        }
    }

    /// 标识
    pub fn id(&self) -> &UpstreamRefId {
        &self.id
    }

    /// 引用的凭证
    pub fn credential_id(&self) -> &CredentialId {
        &self.credential_id
    }

    /// 上游模型
    pub fn upstream_model(&self) -> &UpstreamModelId {
        &self.upstream_model
    }

    /// 协议
    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    /// 权重
    pub fn weight(&self) -> Weight {
        self.weight
    }

    /// 优先级
    pub fn priority(&self) -> Priority {
        self.priority
    }

    /// 设置权重与优先级
    pub fn tune(&mut self, weight: Weight, priority: Priority) {
        self.weight = weight;
        self.priority = priority;
    }
}

/// 模型别名聚合根
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelAlias {
    name: AliasName,
    strategy: SelectionStrategy,
    refs: Vec<UpstreamRef>,
}

impl ModelAlias {
    /// 创建别名并携带首个上游引用
    pub fn create(name: AliasName, strategy: SelectionStrategy, first: UpstreamRef) -> Self {
        Self {
            name,
            strategy,
            refs: vec![first],
        }
    }

    /// 名称
    pub fn name(&self) -> &AliasName {
        &self.name
    }

    /// 选择策略
    pub fn strategy(&self) -> SelectionStrategy {
        self.strategy
    }

    /// 上游引用
    pub fn refs(&self) -> &[UpstreamRef] {
        &self.refs
    }

    /// 追加一个上游引用
    pub fn add_ref(&mut self, reference: UpstreamRef) {
        self.refs.push(reference);
    }

    /// 是否引用了该凭证
    pub fn references(&self, credential_id: &CredentialId) -> bool {
        self.refs.iter().any(|r| r.credential_id() == credential_id)
    }

    /// 移除某个凭证的全部引用，返回是否已无引用
    pub fn remove_credential_refs(&mut self, credential_id: &CredentialId) -> bool {
        self.refs.retain(|r| r.credential_id() != credential_id);
        self.refs.is_empty()
    }
}

/// 用量记录聚合根
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageRecord {
    request_id: String,
    credential_id: CredentialId,
    alias: AliasName,
    tokens: TokenUsage,
    cost: Money,
    succeeded: bool,
    at: DateTime<Utc>,
}

impl UsageRecord {
    /// 记录一次调用
    pub fn record(
        request_id: impl Into<String>,
        credential_id: CredentialId,
        alias: AliasName,
        tokens: TokenUsage,
        cost: Money,
        succeeded: bool,
        at: DateTime<Utc>,
    ) -> (Self, DomainEvent) {
        let record = Self {
            request_id: request_id.into(),
            credential_id: credential_id.clone(),
            alias: alias.clone(),
            tokens,
            cost,
            succeeded,
            at,
        };
        let event = DomainEvent::UsageRecorded {
            credential_id,
            alias: alias.as_str().to_string(),
            tokens,
            cost,
            succeeded,
        };
        (record, event)
    }

    /// 请求标识
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// 凭证标识
    pub fn credential_id(&self) -> &CredentialId {
        &self.credential_id
    }

    /// 别名
    pub fn alias(&self) -> &AliasName {
        &self.alias
    }

    /// token 用量
    pub fn tokens(&self) -> TokenUsage {
        self.tokens
    }

    /// 折算金额
    pub fn cost(&self) -> Money {
        self.cost
    }

    /// 是否成功
    pub fn succeeded(&self) -> bool {
        self.succeeded
    }

    /// 发生时刻
    pub fn at(&self) -> DateTime<Utc> {
        self.at
    }
}

/// 统计粒度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Granularity {
    /// 按天
    Daily,
    /// 按小时
    Hourly,
}

/// 设置聚合根
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    listen: String,
    access_key_hash: String,
    granularity: Granularity,
    retention_days: u32,
    session_affinity: bool,
}

impl Settings {
    /// 初始化设置
    pub fn initialize(listen: impl Into<String>, access_key_hash: impl Into<String>) -> Self {
        Self {
            listen: listen.into(),
            access_key_hash: access_key_hash.into(),
            granularity: Granularity::Daily,
            retention_days: 90,
            session_affinity: true,
        }
    }

    /// 监听地址
    pub fn listen(&self) -> &str {
        &self.listen
    }

    /// 访问密钥哈希
    pub fn access_key_hash(&self) -> &str {
        &self.access_key_hash
    }

    /// 统计粒度
    pub fn granularity(&self) -> Granularity {
        self.granularity
    }

    /// 保留期
    pub fn retention_days(&self) -> u32 {
        self.retention_days
    }

    /// 会话粘性是否开启
    pub fn session_affinity(&self) -> bool {
        self.session_affinity
    }

    /// 更新保留期
    pub fn update_retention(&mut self, days: u32) -> Result<(), SettingsError> {
        if days == 0 {
            return Err(SettingsError::RetentionInvalid(format!("{days} 天")));
        }
        self.retention_days = days;
        Ok(())
    }

    /// 更新会话粘性
    pub fn update_session_affinity(&mut self, enabled: bool) {
        self.session_affinity = enabled;
    }

    /// 轮换访问密钥
    pub fn rotate_access_key(&mut self, new_hash: impl Into<String>) {
        self.access_key_hash = new_hash.into();
    }
}

/// 别名与凭证引用之间的约束检查
pub fn ensure_alias_can_reference(
    credential: Option<&Credential>,
    credential_id: &CredentialId,
) -> Result<(), CatalogError> {
    match credential {
        None => Err(CatalogError::CredentialNotFound(
            credential_id.as_str().to_string(),
        )),
        Some(c) if !c.is_available() => {
            Err(CatalogError::CredentialUnavailable(c.name().to_string()))
        }
        Some(_) => Ok(()),
    }
}
