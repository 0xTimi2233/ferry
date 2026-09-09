@module-credential @use_case-complete_subscription_authorization
Feature: 完成订阅授权
  作为网关使用者
  我想要在浏览器授权后让网关接手
  以便于订阅凭证被自动登记

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @credential-complete_subscription_authorization_completed
  Scenario: 用有效授权码完成授权
    Given 存在未过期的待授权状态，状态标识为 "state-0001"
    When 上游回调携带状态标识 "state-0001" 与授权码 "code-0001"
    Then 网关向上游换取访问令牌与刷新令牌
    And 创建订阅凭证且健康状态为可用
    And 待授权状态被消费，再次使用同一状态标识会被拒绝
    And 发布领域事件 CredentialRegistered

  @wip @credential-complete_subscription_authorization_expired
  Scenario: 待授权状态已过期时拒绝
    Given 待授权状态 "state-0002" 已过期
    When 上游回调携带状态标识 "state-0002" 与授权码 "code-0002"
    Then 请求被拒绝且返回授权已过期
    And 不创建订阅凭证

  @wip @credential-complete_subscription_authorization_mismatched
  Scenario: 状态标识不存在时拒绝
    Given 不存在状态标识为 "state-9999" 的待授权状态
    When 上游回调携带状态标识 "state-9999" 与授权码 "code-9999"
    Then 请求被拒绝且返回授权状态无效
    And 不创建订阅凭证
