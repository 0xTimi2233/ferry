@module-catalog @use_case-delete_account_pool
Feature: 删除账号池
  作为网关使用者
  我想要删除不再使用的账号池
  以便于模型清单保持整洁

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @catalog-delete_account_pool_deleted
  Scenario: 删除存在的账号池
    Given 账号池 "deepseek-chat" 已存在
    When 删除该账号池
    Then 该账号池不再出现在账号池列表中

  @wip @catalog-delete_account_pool_missing
  Scenario: 账号池不存在时拒绝删除
    Given 不存在名为 "no-such-pool" 的账号池
    When 删除该账号池
    Then 删除被拒绝且返回账号池不存在

  @wip @catalog-delete_account_pool_cleanup_references
  Scenario: 凭证被删除后清理其引用
    Given 账号池 "deepseek-chat" 引用凭证 "deepseek-backup"
    When 收到领域事件 CredentialDeleted 指向 "deepseek-backup"
    Then 该账号池不再包含指向该凭证的引用

  @wip @catalog-delete_account_pool_last_reference
  Scenario: 引用被清空后账号池一并移除
    Given 账号池 "solo-pool" 只引用凭证 "deepseek-backup"
    When 收到领域事件 CredentialDeleted 指向 "deepseek-backup"
    Then 账号池 "solo-pool" 被移除
