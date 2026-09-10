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
    And 发布领域事件 CredentialDeleted

  @wip @credential-delete_credential_group_kept
  Scenario: 组内仍有其它凭证时保留账号组
    Given 账号组 "g-deepseek" 包含凭证 "deepseek-main" 与 "deepseek-backup"
    When 删除凭证 "deepseek-backup"
    Then 账号组 "g-deepseek" 仍然存在且只包含 "deepseek-main"
    And 不发布领域事件 CredentialGroupRemoved

  @wip @credential-delete_credential_group_removed
  Scenario: 组内最后一个凭证被删除时移除账号组
    Given 账号组 "g-deepseek" 只包含凭证 "deepseek-backup"
    When 删除凭证 "deepseek-backup"
    Then 账号组 "g-deepseek" 被移除
    And 发布领域事件 CredentialGroupRemoved

  @wip @credential-delete_credential_missing
  Scenario: 凭证不存在时拒绝删除
    Given 不存在标识为 "credential-9999" 的凭证
    When 删除该凭证
    Then 删除被拒绝且返回凭证不存在
    And 不发布领域事件
