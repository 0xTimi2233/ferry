//! 聚合根

use chrono::{DateTime, Utc};

use crate::domain::errors::{CatalogError, CredentialError, SettingsError};
use crate::domain::events::DomainEvent;
use crate::domain::values::{
    AliasName, CredentialId, HealthStatus, InvalidValue, Money, Priority, Protocol, Provider,
    Secret, SelectionStrategy, TokenUsage, UpstreamModelId, UpstreamRefId, Weight,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialKind {
    ApiKey {
        secret: Secret,
    },
    Subscription {
        access_token: Secret,
        refresh_token: Secret,
        expires_at: DateTime<Utc>,
        account: Option<String>,
    },
}

impl CredentialKind {
    pub fn is_subscription(&self) -> bool {
        matches!(self, Self::Subscription { .. })
    }
}

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
        let credential = Self {
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
        };
        Ok((credential, event))
    }

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
        let credential = Self {
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
        };
        Ok((credential, event))
    }

    pub fn id(&self) -> &CredentialId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn provider(&self) -> &Provider {
        &self.provider
    }

    pub fn kind(&self) -> &CredentialKind {
        &self.kind
    }

    pub fn health(&self) -> &HealthStatus {
        &self.health
    }

    pub fn priority(&self) -> Priority {
        self.priority
    }

    pub fn weight(&self) -> Weight {
        self.weight
    }

    pub fn offered_models(&self) -> &[UpstreamModelId] {
        &self.offered_models
    }

    pub fn kept_models(&self) -> &[UpstreamModelId] {
        &self.kept_models
    }

    pub fn next_refresh_at(&self) -> Option<DateTime<Utc>> {
        self.next_refresh_at
    }

    pub fn is_available(&self) -> bool {
        self.health.is_available()
    }

    pub fn masked_secret(&self) -> String {
        match &self.kind {
            CredentialKind::ApiKey { secret } => secret.mask(),
            CredentialKind::Subscription { .. } => "oauth •••• 已授权".to_string(),
        }
    }

    /// 上游清单变化时，已保留但不再提供的模型被剔除
    pub fn record_offered_models(&mut self, models: Vec<UpstreamModelId>) {
        self.kept_models.retain(|m| models.contains(m));
        self.offered_models = models;
    }

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

    pub fn ensure_available_for_fetch(&self) -> Result<(), CredentialError> {
        if self.is_available() {
            Ok(())
        } else {
            Err(CredentialError::NotAvailable(self.name.clone()))
        }
    }

    pub fn disable(&mut self) -> DomainEvent {
        self.health = HealthStatus::Disabled;
        self.health_changed_event()
    }

    pub fn enable(&mut self) -> DomainEvent {
        self.health = HealthStatus::Ready;
        self.health_changed_event()
    }

    pub fn mark_cooling(
        &mut self,
        reason: impl Into<String>,
        recover_at: DateTime<Utc>,
    ) -> DomainEvent {
        self.health = HealthStatus::Cooling {
            reason: reason.into(),
            recover_at,
        };
        self.health_changed_event()
    }

    pub fn mark_failed(&mut self, reason: impl Into<String>) -> DomainEvent {
        self.health = HealthStatus::Failed {
            reason: reason.into(),
        };
        self.health_changed_event()
    }

    fn health_changed_event(&self) -> DomainEvent {
        DomainEvent::CredentialHealthChanged {
            id: self.id.clone(),
            status: self.health.clone(),
        }
    }

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
        self.health_changed_event()
    }

    /// 访问令牌仍有效时只推迟下次刷新，已过期才标记失效
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

    pub fn id(&self) -> &UpstreamRefId {
        &self.id
    }

    pub fn credential_id(&self) -> &CredentialId {
        &self.credential_id
    }

    pub fn upstream_model(&self) -> &UpstreamModelId {
        &self.upstream_model
    }

    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    pub fn weight(&self) -> Weight {
        self.weight
    }

    pub fn priority(&self) -> Priority {
        self.priority
    }

    pub fn tune(&mut self, weight: Weight, priority: Priority) {
        self.weight = weight;
        self.priority = priority;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelAlias {
    name: AliasName,
    strategy: SelectionStrategy,
    refs: Vec<UpstreamRef>,
}

impl ModelAlias {
    pub fn create(name: AliasName, strategy: SelectionStrategy, first: UpstreamRef) -> Self {
        Self {
            name,
            strategy,
            refs: vec![first],
        }
    }

    pub fn name(&self) -> &AliasName {
        &self.name
    }

    pub fn strategy(&self) -> SelectionStrategy {
        self.strategy
    }

    pub fn refs(&self) -> &[UpstreamRef] {
        &self.refs
    }

    pub fn add_ref(&mut self, reference: UpstreamRef) {
        self.refs.push(reference);
    }

    pub fn references(&self, credential_id: &CredentialId) -> bool {
        self.refs.iter().any(|r| r.credential_id() == credential_id)
    }

    /// 返回移除后是否已无引用，由调用方决定是否删除别名
    pub fn remove_credential_refs(&mut self, credential_id: &CredentialId) -> bool {
        self.refs.retain(|r| r.credential_id() != credential_id);
        self.refs.is_empty()
    }
}

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
    #[allow(clippy::too_many_arguments)]
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

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn credential_id(&self) -> &CredentialId {
        &self.credential_id
    }

    pub fn alias(&self) -> &AliasName {
        &self.alias
    }

    pub fn tokens(&self) -> TokenUsage {
        self.tokens
    }

    pub fn cost(&self) -> Money {
        self.cost
    }

    pub fn succeeded(&self) -> bool {
        self.succeeded
    }

    pub fn at(&self) -> DateTime<Utc> {
        self.at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Granularity {
    Daily,
    Hourly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    listen: String,
    access_key_hash: String,
    granularity: Granularity,
    retention_days: u32,
    session_affinity: bool,
}

impl Settings {
    pub fn initialize(listen: impl Into<String>, access_key_hash: impl Into<String>) -> Self {
        Self {
            listen: listen.into(),
            access_key_hash: access_key_hash.into(),
            granularity: Granularity::Daily,
            retention_days: 90,
            session_affinity: true,
        }
    }

    pub fn listen(&self) -> &str {
        &self.listen
    }

    pub fn access_key_hash(&self) -> &str {
        &self.access_key_hash
    }

    pub fn granularity(&self) -> Granularity {
        self.granularity
    }

    pub fn retention_days(&self) -> u32 {
        self.retention_days
    }

    pub fn session_affinity(&self) -> bool {
        self.session_affinity
    }

    pub fn update_retention(&mut self, days: u32) -> Result<(), SettingsError> {
        if days == 0 {
            return Err(SettingsError::RetentionInvalid(format!("{days} 天")));
        }
        self.retention_days = days;
        Ok(())
    }

    pub fn update_session_affinity(&mut self, enabled: bool) {
        self.session_affinity = enabled;
    }

    pub fn rotate_access_key(&mut self, new_hash: impl Into<String>) {
        self.access_key_hash = new_hash.into();
    }
}

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
