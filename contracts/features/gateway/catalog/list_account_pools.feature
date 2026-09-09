@module-catalog @use_case-list_account_pools
Feature: 查询账号池列表
  作为网关使用者
  我想要查看全部账号池及其上游引用
  以便于确认客户端可用的模型名

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @catalog-list_account_pools_listed
  Scenario: 列出全部账号池
    Given 存在两个账号池
    When 查询账号池列表
    Then 返回两条记录，每条包含账号池、选择策略与上游引用数量
    And 每条引用包含上游、上游模型、协议、权重与优先级

  @wip @catalog-list_account_pools_empty
  Scenario: 没有账号池时返回空列表
    Given 不存在任何账号池
    When 查询账号池列表
    Then 返回空列表
