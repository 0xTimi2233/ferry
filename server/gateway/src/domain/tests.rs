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
    ]);

    credential.update_kept_models(vec![model("deepseek-chat")?, model("deepseek-coder")?])?;

    assert_eq!(credential.kept_models().len(), 2);
    Ok(())
}

#[test]
fn should_reject_empty_model_selection() -> TestResult {
    let mut credential = api_key("deepseek-main")?;
    credential.record_offered_models(vec![model("deepseek-chat")?]);

    let result = credential.update_kept_models(vec![]);

    assert_eq!(result, Err(CredentialError::EmptySelection));
    Ok(())
}

#[test]
fn should_reject_model_not_offered_by_upstream() -> TestResult {
    let mut credential = api_key("deepseek-main")?;
    credential.record_offered_models(vec![model("deepseek-chat")?]);

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
    credential.record_offered_models(vec![model("deepseek-chat")?, model("deepseek-coder")?]);
    credential.update_kept_models(vec![model("deepseek-chat")?, model("deepseek-coder")?])?;

    credential.record_offered_models(vec![model("deepseek-chat")?]);

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
fn should_create_alias_with_first_reference() -> TestResult {
    let reference = UpstreamRef::new(
        UpstreamRefId::new("r1"),
        CredentialId::new("c1"),
        model("deepseek-chat")?,
        Protocol::OpenAiChat,
    );
    let alias = ModelAlias::create(
        alias_name("deepseek-chat")?,
        SelectionStrategy::RoundRobin,
        reference,
    );

    assert_eq!(alias.refs().len(), 1);
    assert_eq!(alias.refs()[0].protocol(), Protocol::OpenAiChat);
    assert!(alias.references(&CredentialId::new("c1")));
    Ok(())
}

#[test]
fn should_add_second_reference_and_tune_weights() -> TestResult {
    let mut alias = ModelAlias::create(
        alias_name("deepseek-chat")?,
        SelectionStrategy::RoundRobin,
        UpstreamRef::new(
            UpstreamRefId::new("r1"),
            CredentialId::new("c1"),
            model("deepseek-chat")?,
            Protocol::OpenAiChat,
        ),
    );
    let mut second = UpstreamRef::new(
        UpstreamRefId::new("r2"),
        CredentialId::new("c3"),
        model("deepseek-chat")?,
        Protocol::OpenAiChat,
    );
    second.tune(Weight::new(3), Priority::new(2)?);

    alias.add_ref(second);

    assert_eq!(alias.refs().len(), 2);
    assert_eq!(alias.refs()[1].weight().value(), 3);
    assert_eq!(alias.refs()[1].priority().value(), 2);
    Ok(())
}

#[test]
fn should_remove_all_references_of_deleted_credential() -> TestResult {
    let mut alias = ModelAlias::create(
        alias_name("deepseek-chat")?,
        SelectionStrategy::RoundRobin,
        UpstreamRef::new(
            UpstreamRefId::new("r1"),
            CredentialId::new("c1"),
            model("deepseek-chat")?,
            Protocol::OpenAiChat,
        ),
    );
    alias.add_ref(UpstreamRef::new(
        UpstreamRefId::new("r2"),
        CredentialId::new("c3"),
        model("deepseek-chat")?,
        Protocol::OpenAiChat,
    ));

    let empty = alias.remove_credential_refs(&CredentialId::new("c1"));

    assert!(!empty);
    assert_eq!(alias.refs().len(), 1);
    assert!(!alias.references(&CredentialId::new("c1")));
    Ok(())
}

#[test]
fn should_report_alias_empty_after_last_reference_removed() -> TestResult {
    let mut alias = ModelAlias::create(
        alias_name("solo-alias")?,
        SelectionStrategy::FillFirst,
        UpstreamRef::new(
            UpstreamRefId::new("r1"),
            CredentialId::new("c3"),
            model("deepseek-chat")?,
            Protocol::OpenAiChat,
        ),
    );

    let empty = alias.remove_credential_refs(&CredentialId::new("c3"));

    assert!(empty);
    Ok(())
}

#[test]
fn should_reject_reference_to_missing_credential() -> TestResult {
    let result = ensure_alias_can_reference(None, &CredentialId::new("credential-9999"));

    assert!(matches!(result, Err(CatalogError::CredentialNotFound(_))));
    Ok(())
}

#[test]
fn should_reject_reference_to_disabled_credential() -> TestResult {
    let mut credential = api_key("openai-key-2")?;
    credential.disable();

    let result = ensure_alias_can_reference(Some(&credential), credential.id());

    assert!(matches!(
        result,
        Err(CatalogError::CredentialUnavailable(_))
    ));
    Ok(())
}

#[test]
fn should_publish_usage_event_when_recorded() -> TestResult {
    let (record, event) = UsageRecord::record(
        "req_1",
        CredentialId::new("c1"),
        alias_name("deepseek-chat")?,
        TokenUsage {
            input: 100,
            output: 20,
            ..TokenUsage::default()
        },
        Money::from_micro_usd(1_500),
        true,
        Utc::now(),
    );

    assert_eq!(record.tokens().total(), 120);
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
