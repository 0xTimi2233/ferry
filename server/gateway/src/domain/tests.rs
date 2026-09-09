//! 领域模型单测
//!
//! 断言走 assert 宏，取值走 `?` 传播，不出现 unwrap 与 expect。

use chrono::{Duration, Utc};

use crate::domain::aggregates::*;
use crate::domain::errors::*;
use crate::domain::values::*;

/// 测试用返回类型，任意领域错误都能被 `?` 传播
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn provider(name: &str) -> Result<Provider, InvalidValue> {
    Provider::new(name)
}

fn model(name: &str) -> Result<UpstreamModelId, InvalidValue> {
    UpstreamModelId::new(name)
}

fn alias_name(name: &str) -> Result<AliasName, InvalidValue> {
    AliasName::new(name)
}

fn secret(raw: &str) -> Result<Secret, InvalidValue> {
    Secret::new(raw)
}

fn group_id() -> GroupId {
    GroupId::new("g-deepseek")
}

fn api_key(name: &str) -> Result<Credential, Box<dyn std::error::Error>> {
    let (credential, _) = Credential::register_api_key(
        CredentialId::new("c1"),
        name,
        provider("DeepSeek")?,
        secret("sk-test-0001")?,
    )?;
    Ok(credential)
}

fn subscription(expires_in_hours: i64) -> Result<Credential, Box<dyn std::error::Error>> {
    let (credential, _) = Credential::register_subscription(
        CredentialId::new("c2"),
        "chatgpt-plus",
        provider("OpenAI")?,
        secret("access-token")?,
        secret("refresh-token")?,
        Utc::now() + Duration::hours(expires_in_hours),
        Some("acct-1".to_string()),
    )?;
    Ok(credential)
}

fn target(id: &str, group: GroupId, model_name: &str) -> Result<AliasTarget, InvalidValue> {
    Ok(AliasTarget::new(
        AliasTargetId::new(id),
        group,
        model(model_name)?,
        Protocol::OpenAiChat,
    ))
}

#[test]
fn should_register_api_key_credential_as_ready() -> TestResult {
    let (credential, event) = Credential::register_api_key(
        CredentialId::new("c1"),
        "deepseek-main",
        provider("DeepSeek")?,
        secret("sk-test-0001")?,
    )?;

    assert!(credential.is_available());
    assert_eq!(credential.masked_secret(), "sk-t••••••0001");
    assert_eq!(credential.group(), &group_id());
    assert_eq!(event.name(), "CredentialRegistered");
    Ok(())
}

#[test]
fn should_reject_blank_credential_name() -> TestResult {
    let result = Credential::register_api_key(
        CredentialId::new("c1"),
        "   ",
        provider("DeepSeek")?,
        secret("sk-test-0001")?,
    );

    assert!(matches!(result, Err(CredentialError::Invalid(_))));
    Ok(())
}

#[test]
fn should_reject_subscription_for_unsupported_upstream() -> TestResult {
    let result = Credential::register_subscription(
        CredentialId::new("c1"),
        "deepseek-main",
        provider("DeepSeek")?,
        secret("access")?,
        secret("refresh")?,
        Utc::now() + Duration::hours(1),
        None,
    );

    assert!(matches!(
        result,
        Err(CredentialError::UpstreamLacksSubscription(_))
    ));
    Ok(())
}

#[test]
fn should_keep_only_models_offered_by_upstream() -> TestResult {
    let mut credential = api_key("deepseek-main")?;
    credential.record_offered_models(vec![
        model("deepseek-chat")?,
        model("deepseek-reasoner")?,
        model("deepseek-coder")?,
    ])?;

    credential.update_kept_models(vec![model("deepseek-chat")?, model("deepseek-coder")?])?;

    assert_eq!(credential.kept_models().len(), 2);
    Ok(())
}

#[test]
fn should_reject_empty_model_selection() -> TestResult {
    let mut credential = api_key("deepseek-main")?;
    credential.record_offered_models(vec![model("deepseek-chat")?])?;

    let result = credential.update_kept_models(vec![]);

    assert_eq!(result, Err(CredentialError::EmptySelection));
    Ok(())
}

#[test]
fn should_reject_model_not_offered_by_upstream() -> TestResult {
    let mut credential = api_key("deepseek-main")?;
    credential.record_offered_models(vec![model("deepseek-chat")?])?;

    let result = credential.update_kept_models(vec![model("gpt-5")?]);

    assert_eq!(
        result,
        Err(CredentialError::ModelNotOffered("gpt-5".to_string()))
    );
    Ok(())
}

#[test]
fn should_drop_kept_models_no_longer_offered() -> TestResult {
    let mut credential = api_key("deepseek-main")?;
    credential.record_offered_models(vec![model("deepseek-chat")?, model("deepseek-coder")?])?;
    credential.update_kept_models(vec![model("deepseek-chat")?, model("deepseek-coder")?])?;

    credential.record_offered_models(vec![model("deepseek-chat")?])?;

    assert_eq!(credential.kept_models(), &[model("deepseek-chat")?]);
    Ok(())
}

#[test]
fn should_disable_and_enable_credential() -> TestResult {
    let mut credential = api_key("deepseek-main")?;

    let disabled = credential.disable();
    assert!(!credential.is_available());
    assert_eq!(disabled.name(), "CredentialHealthChanged");

    credential.enable();
    assert!(credential.is_available());
    Ok(())
}

#[test]
fn should_clear_cooldown_when_disabled() -> TestResult {
    let mut credential = api_key("openai-key-2")?;
    credential.mark_cooling("429 速率限制", Utc::now() + Duration::minutes(1));
    assert!(!credential.is_available());

    credential.disable();

    assert!(matches!(credential.health(), HealthStatus::Disabled));
    Ok(())
}

#[test]
fn should_refresh_subscription_token_and_stay_ready() -> TestResult {
    let mut credential = subscription(4)?;
    let expires_at = Utc::now() + Duration::hours(8);

    let event = credential.apply_refresh(secret("new-access")?, secret("new-refresh")?, expires_at);

    assert!(credential.is_available());
    assert_eq!(credential.next_refresh_at(), Some(expires_at));
    assert_eq!(event.name(), "CredentialHealthChanged");
    Ok(())
}

#[test]
fn should_postpone_refresh_when_access_token_still_valid() -> TestResult {
    let mut credential = subscription(1)?;
    let before = credential.next_refresh_at();

    let event = credential.on_refresh_failed(Utc::now(), Utc::now() + Duration::minutes(5));

    assert!(event.is_none());
    assert!(credential.is_available());
    assert_ne!(credential.next_refresh_at(), before);
    Ok(())
}

#[test]
fn should_mark_failed_when_refresh_fails_and_token_expired() -> TestResult {
    let mut credential = subscription(-1)?;

    let event = credential.on_refresh_failed(Utc::now(), Utc::now() + Duration::minutes(5));

    assert!(event.is_some());
    assert!(credential.health().is_failed());
    assert_eq!(credential.next_refresh_at(), None);
    Ok(())
}

#[test]
fn should_reject_fetch_when_credential_unavailable() -> TestResult {
    let mut credential = api_key("openai-key-2")?;
    credential.mark_cooling("429", Utc::now() + Duration::minutes(1));

    assert!(credential.ensure_available_for_fetch().is_err());
    Ok(())
}

#[test]
fn should_reject_duplicated_credential_name() -> TestResult {
    let credential = api_key("deepseek-main")?;

    assert_eq!(
        credential.ensure_name_available(true),
        Err(CredentialError::Duplicated("deepseek-main".to_string()))
    );
    assert!(credential.ensure_name_available(false).is_ok());
    Ok(())
}

#[test]
fn should_reject_offered_models_that_empty_kept_set() -> TestResult {
    let mut credential = api_key("deepseek-main")?;
    credential.record_offered_models(vec![model("deepseek-chat")?])?;
    credential.update_kept_models(vec![model("deepseek-chat")?])?;

    let result = credential.record_offered_models(vec![model("deepseek-reasoner")?]);

    assert_eq!(result, Err(CredentialError::EmptySelection));
    assert_eq!(credential.kept_models(), &[model("deepseek-chat")?]);
    Ok(())
}

#[test]
fn should_charge_cost_only_for_api_key_credentials() -> TestResult {
    let key = api_key("deepseek-main")?;
    let sub = subscription(4)?;

    assert!(key.kind().charges_cost());
    assert!(!sub.kind().charges_cost());
    Ok(())
}

#[test]
fn should_consume_pending_authorization_once() -> TestResult {
    let pending = PendingAuthorization {
        state: "state-0001".to_string(),
        code_verifier: "verifier".to_string(),
        provider: "OpenAI".to_string(),
        expires_at: Utc::now() + Duration::minutes(10),
    };

    assert!(pending.clone().consume("state-0001", Utc::now()).is_ok());
    assert_eq!(
        pending.clone().consume("state-9999", Utc::now()),
        Err(AuthorizationError::Invalid)
    );

    let expired = PendingAuthorization {
        expires_at: Utc::now() - Duration::minutes(1),
        ..pending
    };
    assert_eq!(
        expired.consume("state-0001", Utc::now()),
        Err(AuthorizationError::Expired)
    );
    Ok(())
}

#[test]
fn should_recover_from_cooldown_after_recover_at() -> TestResult {
    let mut credential = api_key("deepseek-main")?;
    let recover_at = Utc::now() + Duration::minutes(1);
    credential.mark_cooling("429", recover_at);

    assert!(credential.recover_from_cooldown(Utc::now()).is_none());

    let event = credential.recover_from_cooldown(recover_at + Duration::seconds(1));

    assert!(event.is_some());
    assert!(credential.is_available());
    Ok(())
}

#[test]
fn should_derive_group_id_from_provider() -> TestResult {
    assert_eq!(
        GroupId::for_provider(&provider("DeepSeek")?).as_str(),
        "g-deepseek"
    );
    assert_eq!(
        GroupId::for_provider(&provider("OpenAI")?).as_str(),
        "g-openai"
    );
    Ok(())
}

#[test]
fn should_classify_retryable_status_codes() -> TestResult {
    assert!(RetryPolicy::is_retryable_status(429));
    assert!(RetryPolicy::is_retryable_status(503));
    assert!(!RetryPolicy::is_retryable_status(400));
    assert_eq!(RetryPolicy::default().max_rounds, 3);
    Ok(())
}

#[test]
fn should_publish_credential_deleted_event() -> TestResult {
    let credential = api_key("deepseek-backup")?;

    let event = credential.delete();

    assert_eq!(event.name(), "CredentialDeleted");
    Ok(())
}

#[test]
fn should_group_credentials_by_provider_with_default_strategy() -> TestResult {
    let mut group = CredentialGroup::for_provider(provider("DeepSeek")?);

    assert_eq!(group.provider().as_str(), "DeepSeek");
    assert_eq!(group.strategy(), SelectionStrategy::RoundRobin);

    group.set_strategy(SelectionStrategy::Weighted);

    assert_eq!(group.strategy(), SelectionStrategy::Weighted);
    Ok(())
}

#[test]
fn should_create_alias_with_first_target() -> TestResult {
    let alias = Alias::create(
        alias_name("deepseek-chat")?,
        target("t1", group_id(), "deepseek-chat")?,
    );

    assert_eq!(alias.targets().len(), 1);
    assert_eq!(alias.targets()[0].protocol(), Protocol::OpenAiChat);
    assert!(alias.references_group(&group_id()));
    Ok(())
}

#[test]
fn should_add_second_target_and_tune_weights() -> TestResult {
    let mut alias = Alias::create(
        alias_name("deepseek-chat")?,
        target("t1", group_id(), "deepseek-chat")?,
    );
    let mut second = target("t2", GroupId::new("g-openai"), "gpt-5")?;
    second.tune(Weight::new(3), Priority::new(2)?);

    alias.add_target(second);

    assert_eq!(alias.targets().len(), 2);
    assert_eq!(alias.targets()[1].weight().value(), 3);
    assert_eq!(alias.targets()[1].priority().value(), 2);
    Ok(())
}

#[test]
fn should_remove_all_targets_of_removed_group() -> TestResult {
    let mut alias = Alias::create(
        alias_name("deepseek-chat")?,
        target("t1", group_id(), "deepseek-chat")?,
    );
    alias.add_target(target("t2", GroupId::new("g-openai"), "gpt-5")?);

    let empty = alias.remove_group_targets(&group_id());

    assert!(!empty);
    assert_eq!(alias.targets().len(), 1);
    assert!(!alias.references_group(&group_id()));
    Ok(())
}

#[test]
fn should_report_alias_empty_after_last_target_removed() -> TestResult {
    let mut alias = Alias::create(
        alias_name("solo-alias")?,
        target("t1", group_id(), "deepseek-chat")?,
    );

    let empty = alias.remove_group_targets(&group_id());

    assert!(empty);
    Ok(())
}

#[test]
fn should_reject_reference_to_missing_group() -> TestResult {
    let result = ensure_alias_can_reference(None, &GroupId::new("g-missing"));

    assert!(matches!(result, Err(CatalogError::GroupNotFound(_))));
    Ok(())
}

#[test]
fn should_accept_reference_to_existing_group() -> TestResult {
    let group = CredentialGroup::for_provider(provider("DeepSeek")?);

    let result = ensure_alias_can_reference(Some(&group), group.id());

    assert!(result.is_ok());
    Ok(())
}

#[test]
fn should_publish_usage_event_when_recorded() -> TestResult {
    let (record, event) = UsageRecord::record(UsageEntry {
        request_id: "req_1".to_string(),
        credential_id: CredentialId::new("c1"),
        alias: alias_name("deepseek-chat")?,
        tokens: TokenUsage {
            input: 100,
            output: 20,
            ..TokenUsage::default()
        },
        cost: Money::from_micro_usd(1_500),
        succeeded: true,
        latency_ms: 900,
        failure_reason: None,
        affinity_hit: true,
        at: Utc::now(),
    });

    assert_eq!(record.tokens().total(), 120);
    assert_eq!(record.latency_ms(), 900);
    assert!(record.affinity_hit());
    assert_eq!(event.name(), "UsageRecorded");
    Ok(())
}

#[test]
fn should_reject_invalid_retention_days() -> TestResult {
    let mut settings = Settings::initialize("0.0.0.0:8080", "hash");

    let result = settings.update_retention(0);

    assert!(matches!(result, Err(SettingsError::RetentionInvalid(_))));
    assert_eq!(settings.retention_days(), 90);
    Ok(())
}

#[test]
fn should_update_retention_days() -> TestResult {
    let mut settings = Settings::initialize("0.0.0.0:8080", "hash");

    settings.update_retention(180)?;

    assert_eq!(settings.retention_days(), 180);
    assert_eq!(settings.granularity(), Granularity::Daily);
    Ok(())
}
