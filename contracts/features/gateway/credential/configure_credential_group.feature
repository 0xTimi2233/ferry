@module-credential @use_case-configure_credential_group
Feature: 配置账号组
  作为网关使用者
  我想要调整某个上游账号组的选凭证策略
  以便于控制同上游多个凭证的挑选方式

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @credential-configure_credential_group_updated
  Scenario: 更新账号组的选凭证策略
    Given 账号组 "g-deepseek" 的策略为轮询
    When 提交策略为加权
    Then 账号组 "g-deepseek" 的策略变为加权

  @wip @credential-configure_credential_group_missing
  Scenario: 账号组不存在时拒绝
    Given 不存在标识为 "g-missing" 的账号组
    When 提交策略为轮询
    Then 更新被拒绝且返回账号组不存在
