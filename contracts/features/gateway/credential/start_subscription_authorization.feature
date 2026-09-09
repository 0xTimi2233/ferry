@module-credential @use_case-start_subscription_authorization
Feature: 发起订阅授权
  作为网关使用者
  我想要为订阅账号发起授权
  以便于网关能用我的订阅额度调用上游

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @credential-start_subscription_authorization_started
  Scenario: 发起一次订阅授权
    Given 上游 "OpenAI" 支持订阅授权
    When 发起授权请求
    Then 返回一个授权地址
    And 待授权状态被保存，状态标识与校验码成对存在
    And 待授权状态带有有效期

  @wip @credential-start_subscription_authorization_unknown_upstream
  Scenario: 上游不支持订阅授权时拒绝
    Given 上游 "DeepSeek" 不支持订阅授权
    When 发起授权请求
    Then 请求被拒绝且返回该上游不支持订阅授权
    And 不保存待授权状态
