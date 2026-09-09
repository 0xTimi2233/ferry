@module-catalog @use_case-delete_model_alias
Feature: 删除模型别名
  作为网关使用者
  我想要删除不再使用的模型别名
  以便于模型清单保持整洁

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @catalog-delete_model_alias_deleted
  Scenario: 删除存在的别名
    Given 模型别名 "deepseek-chat" 已存在
    When 删除该别名
    Then 该别名不再出现在别名列表中

  @wip @catalog-delete_model_alias_missing
  Scenario: 别名不存在时拒绝删除
    Given 不存在名为 "no-such-alias" 的模型别名
    When 删除该别名
    Then 删除被拒绝且返回别名不存在

  @wip @catalog-delete_model_alias_cleanup_references
  Scenario: 凭证被删除后清理其引用
    Given 模型别名 "deepseek-chat" 引用凭证 "deepseek-backup"
    And 凭证 "deepseek-backup" 已被删除
    When 收到领域事件 CredentialDeleted
    Then 该别名不再包含指向该凭证的引用

  @wip @catalog-delete_model_alias_last_reference
  Scenario: 引用被清空后别名一并移除
    Given 模型别名 "solo-alias" 只引用凭证 "deepseek-backup"
    And 凭证 "deepseek-backup" 已被删除
    When 收到领域事件 CredentialDeleted
    Then 模型别名 "solo-alias" 被移除
