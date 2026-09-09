@module-credential @use_case-change_credential_health
Feature: 启用或禁用凭证
  作为网关使用者
  我想要手动启停某个凭证
  以便于在排查问题时不删除凭证也能停止使用

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @credential-change_credential_health_disabled
  Scenario: 禁用可用凭证
    Given 凭证 "deepseek-main" 健康状态为可用
    When 禁用该凭证
    Then 凭证健康状态变为已禁用
    And 调度不再选取该凭证
    And 发布领域事件 CredentialHealthChanged

  @wip @credential-change_credential_health_enabled
  Scenario: 启用已禁用凭证
    Given 凭证 "deepseek-main" 健康状态为已禁用
    When 启用该凭证
    Then 凭证健康状态变为可用
    And 发布领域事件 CredentialHealthChanged

  @wip @credential-change_credential_health_cooling
  Scenario: 冷却中的凭证被禁用
    Given 凭证 "openai-key-2" 处于冷却状态
    When 禁用该凭证
    Then 凭证健康状态变为已禁用
    And 冷却状态被清除
