//! 聚合根

use chrono::{DateTime, Utc};

use crate::domain::errors::{CatalogError, CredentialError, SettingsError};
use crate::domain::events::DomainEvent;
use crate::domain::values::{
    AliasName, AliasTargetId, CredentialId, GroupId, HealthStatus, InvalidValue, Money, Priority,
    Protocol, Provider, Secret, SelectionStrategy, SupportedUpstreams, TokenUsage, UpstreamModelId,
    Weight,
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

    /// 订阅额度不单独计价，金额只按密钥凭证累计
    pub fn charges_cost(&self) -> bool {
        matches!(self, Self::ApiKey { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credential {
    id: CredentialId,
    group: GroupId,
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
        if !SupportedUpstreams::contains(&provider) {
            return Err(CredentialError::UpstreamUnsupported(
                provider.as_str().to_string(),
            ));
        }
        let group = GroupId::for_provider(&provider);
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
            group,
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

    #[allow(clippy::too_many_arguments)]
    pub fn register_subscription(
        id: CredentialId,
        name: impl Into<String>,
        provider: Provider,
        access_token: Secret,
        refresh_token: Secret,
        expires_at: DateTime<Utc>,
        account: Option<String>,
    ) -> Result<(Self, DomainEvent), CredentialError> {
        if !SupportedUpstreams::contains(&provider) {
            return Err(CredentialError::UpstreamUnsupported(
                provider.as_str().to_string(),
            ));
        }
        if !provider.supports_subscription() {
            return Err(CredentialError::UpstreamLacksSubscription(
                provider.as_str().to_string(),
            ));
        }
        let group = GroupId::for_provider(&provider);
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
            group,
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

    pub fn group(&self) -> &GroupId {
        &self.group
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

    /// 上游清单变化时，已保留但不再提供的模型被剔除；清空则拒绝并保持原集合
    pub fn record_offered_models(
        &mut self,
        models: Vec<UpstreamModelId>,
    ) -> Result<(), CredentialError> {
        let retained: Vec<UpstreamModelId> = self
            .kept_models
            .iter()
            .filter(|m| models.contains(m))
            .cloned()
            .collect();
        if !self.kept_models.is_empty() && retained.is_empty() {
            return Err(CredentialError::EmptySelection);
        }
        self.kept_models = retained;
        self.offered_models = models;
        Ok(())
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

    /// 名称唯一性需要全量视野，由用例层查重后交给聚合定音
    pub fn ensure_name_available(&self, taken: bool) -> Result<(), CredentialError> {
        if taken {
            return Err(CredentialError::Duplicated(self.name.clone()));
        }
        Ok(())
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

    /// 冷却到期后恢复可用，返回事件
    pub fn recover_from_cooldown(&mut self, now: DateTime<Utc>) -> Option<DomainEvent> {
        let recovered = self.health.recovered_after_cooldown(now);
        if matches!(recovered, HealthStatus::Ready) && !self.health.is_available() {
            self.health = recovered;
            return Some(self.health_changed_event());
        }
        None
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

    /// 删除凭证，返回供订阅方清理引用的事件
    pub fn delete(self) -> DomainEvent {
        DomainEvent::CredentialDeleted { id: self.id }
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
pub struct AliasTarget {
    id: AliasTargetId,
    group: GroupId,
    upstream_model: UpstreamModelId,
    protocol: Protocol,
    weight: Weight,
    priority: Priority,
}

impl AliasTarget {
    pub fn new(
        id: AliasTargetId,
        group: GroupId,
        upstream_model: UpstreamModelId,
        protocol: Protocol,
    ) -> Self {
        Self {
            id,
            group,
            upstream_model,
            protocol,
            weight: Weight::default(),
            priority: Priority::default(),
        }
    }

    pub fn id(&self) -> &AliasTargetId {
        &self.id
    }

    pub fn group(&self) -> &GroupId {
        &self.group
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

/// 同一上游的一组凭证，是调度的单位
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialGroup {
    id: GroupId,
    provider: Provider,
    strategy: SelectionStrategy,
    max_concurrency: u32,
}

impl CredentialGroup {
    /// 组由上游唯一确定，标识与上游绑定
    pub fn for_provider(provider: Provider) -> Self {
        Self {
            id: GroupId::for_provider(&provider),
            provider,
            strategy: SelectionStrategy::default(),
            max_concurrency: 4,
        }
    }

    pub fn max_concurrency(&self) -> u32 {
        self.max_concurrency
    }

    pub fn set_max_concurrency(&mut self, value: u32) {
        self.max_concurrency = value.max(1);
    }

    pub fn id(&self) -> &GroupId {
        &self.id
    }

    pub fn provider(&self) -> &Provider {
        &self.provider
    }

    pub fn strategy(&self) -> SelectionStrategy {
        self.strategy
    }

    pub fn set_strategy(&mut self, strategy: SelectionStrategy) {
        self.strategy = strategy;
    }

    /// 组内已无凭证时移除，返回供订阅方清理别名目标的事件
    pub fn remove(self) -> DomainEvent {
        DomainEvent::CredentialGroupRemoved { id: self.id }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alias {
    name: AliasName,
    targets: Vec<AliasTarget>,
}

impl Alias {
    pub fn create(name: AliasName, first: AliasTarget) -> Self {
        Self {
            name,
            targets: vec![first],
        }
    }

    pub fn name(&self) -> &AliasName {
        &self.name
    }

    pub fn targets(&self) -> &[AliasTarget] {
        &self.targets
    }

    pub fn add_target(&mut self, target: AliasTarget) {
        self.targets.push(target);
    }

    /// 按优先级与权重挑选目标：先取优先级最高的目标集合，再按权重在其中挑选。
    /// `roll` 为 [0, 1) 内的取值，由调用方给出以便可重现；权重为零的目标不参与轮询。
    pub fn select_target(&self, roll: f64) -> Option<&AliasTarget> {
        let best = self
            .targets
            .iter()
            .filter(|t| t.weight().is_participating())
            .min_by_key(|t| t.priority().value())?;
        let priority = best.priority().value();
        let pool: Vec<&AliasTarget> = self
            .targets
            .iter()
            .filter(|t| t.priority().value() == priority && t.weight().is_participating())
            .collect();
        let total: u64 = pool.iter().map(|t| u64::from(t.weight().value())).sum();
        if total == 0 {
            return None;
        }
        let mut point = (roll.clamp(0.0, 1.0) * total as f64) as u64;
        for target in pool {
            let weight = u64::from(target.weight().value());
            if point < weight {
                return Some(target);
            }
            point -= weight;
        }
        None
    }

    pub fn references_group(&self, group: &GroupId) -> bool {
        self.targets.iter().any(|t| t.group() == group)
    }

    /// 返回移除后是否已无目标，由调用方决定是否删除别名
    pub fn remove_group_targets(&mut self, group: &GroupId) -> bool {
        self.targets.retain(|t| t.group() != group);
        self.targets.is_empty()
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
    latency_ms: u64,
    failure_reason: Option<String>,
    affinity_hit: bool,
    at: DateTime<Utc>,
}

/// 一次调用的记账入参
#[derive(Debug, Clone)]
pub struct UsageEntry {
    pub request_id: String,
    pub credential_id: CredentialId,
    pub alias: AliasName,
    pub tokens: TokenUsage,
    pub cost: Money,
    pub succeeded: bool,
    pub latency_ms: u64,
    pub failure_reason: Option<String>,
    pub affinity_hit: bool,
    pub at: DateTime<Utc>,
}

impl UsageRecord {
    pub fn record(entry: UsageEntry) -> (Self, DomainEvent) {
        let record = Self {
            request_id: entry.request_id,
            credential_id: entry.credential_id.clone(),
            alias: entry.alias.clone(),
            tokens: entry.tokens,
            cost: entry.cost,
            succeeded: entry.succeeded,
            latency_ms: entry.latency_ms,
            failure_reason: entry.failure_reason.clone(),
            affinity_hit: entry.affinity_hit,
            at: entry.at,
        };
        let event = DomainEvent::UsageRecorded {
            credential_id: entry.credential_id,
            alias: entry.alias.as_str().to_string(),
            tokens: entry.tokens,
            cost: entry.cost,
            succeeded: entry.succeeded,
            latency_ms: entry.latency_ms,
            failure_reason: entry.failure_reason.clone(),
            affinity_hit: entry.affinity_hit,
        };
        (record, event)
    }

    pub fn latency_ms(&self) -> u64 {
        self.latency_ms
    }

    pub fn failure_reason(&self) -> Option<&str> {
        self.failure_reason.as_deref()
    }

    pub fn affinity_hit(&self) -> bool {
        self.affinity_hit
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
    group: Option<&CredentialGroup>,
    group_id: &GroupId,
) -> Result<(), CatalogError> {
    match group {
        None => Err(CatalogError::GroupNotFound(group_id.as_str().to_string())),
        Some(_) => Ok(()),
    }
}
