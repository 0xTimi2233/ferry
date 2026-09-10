@module-catalog @use_case-delete_alias
Feature: 删除别名
  作为网关使用者
  我想要删除不再使用的别名
  以便于模型清单保持整洁

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @catalog-delete_alias_deleted
  Scenario: 删除存在的别名
    Given 别名 "deepseek-chat" 已存在
    When 删除该别名
    Then 该别名不再出现在别名列表中

  @wip @catalog-delete_alias_missing
  Scenario: 别名不存在时拒绝删除
    Given 不存在名为 "no-such-alias" 的别名
    When 删除该别名
    Then 删除被拒绝且返回别名不存在

  @wip @catalog-delete_alias_cleanup_targets
  Scenario: 账号组移除后清理其目标
    Given 别名 "deepseek-chat" 指向账号组 "g-deepseek"
    When 收到领域事件 CredentialGroupRemoved 指向 "g-deepseek"
    Then 该别名不再包含指向该账号组的目标

  @wip @catalog-delete_alias_last_target
  Scenario: 目标被清空后别名一并移除
    Given 别名 "solo-alias" 只指向账号组 "g-deepseek"
    When 收到领域事件 CredentialGroupRemoved 指向 "g-deepseek"
    Then 别名 "solo-alias" 被移除
