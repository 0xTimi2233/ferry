@module-catalog @use_case-list_aliases
Feature: 查询别名列表
  作为网关使用者
  我想要查看全部模型别名及其上游引用
  以便于确认客户端可用的模型名

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @catalog-list_aliases_listed
  Scenario: 列出全部别名
    Given 存在两个模型别名
    When 查询别名列表
    Then 返回两条记录，每条包含别名、选择策略与上游引用数量
    And 每条引用包含上游、上游模型、协议、权重与优先级

  @wip @catalog-list_aliases_empty
  Scenario: 没有别名时返回空列表
    Given 不存在任何模型别名
    When 查询别名列表
    Then 返回空列表
