@module-credential @use_case-refresh_subscription_token
Feature: 刷新订阅令牌
  作为网关系统
  我想要在订阅令牌过期前自动刷新
  以便于凭证持续可用而无需人工干预

  Background:
    Given 网关已启动

  @wip @credential-refresh_subscription_token_refreshed
  Scenario: 在到期前刷新令牌
    Given 订阅凭证 "chatgpt-plus" 的访问令牌将在 4 小时内过期
    And 其刷新令牌有效
    When 自动刷新任务处理该凭证
    Then 访问令牌与刷新令牌被更新
    And 凭证健康状态保持可用
    And 发布领域事件 CredentialHealthChanged

  @wip @credential-refresh_subscription_token_failed
  Scenario: 刷新失败且令牌已过期时标记不可用
    Given 订阅凭证 "chatgpt-plus" 的访问令牌已过期
    And 其刷新令牌已失效
    When 自动刷新任务处理该凭证
    Then 凭证健康状态变为失效且原因记录为刷新失败
    And 发布领域事件 CredentialHealthChanged

  @wip @credential-refresh_subscription_token_rejected
  Scenario: 刷新令牌已失效但访问令牌仍有效时保持可用
    Given 订阅凭证 "chatgpt-plus" 的访问令牌将在 30 分钟后过期
    And 其刷新令牌已失效
    When 自动刷新任务处理该凭证
    Then 凭证健康状态保持可用
    And 下次刷新时间被推迟
