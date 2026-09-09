@module-credential @use_case-delete_credential
Feature: 删除凭证
  作为网关使用者
  我想要删除不再使用的凭证
  以便于凭证清单保持整洁

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @credential-delete_credential_deleted
  Scenario: 删除凭证
    Given 存在凭证 "deepseek-backup"
    When 删除该凭证
    Then 凭证不再出现在凭证列表中
    And 发布领域事件 CredentialDeleted 供账号池清理引用

  @wip @credential-delete_credential_missing
  Scenario: 凭证不存在时拒绝删除
    Given 不存在标识为 "credential-9999" 的凭证
    When 删除该凭证
    Then 删除被拒绝且返回凭证不存在
    And 不发布领域事件
