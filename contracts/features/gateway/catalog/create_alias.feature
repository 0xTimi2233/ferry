@module-catalog @use_case-create_alias
Feature: 创建别名
  作为网关使用者
  我想要把一个模型名绑定到若干账号组与上游模型
  以便于客户端按模型名请求时网关能挑选账号组

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @catalog-create_alias_created
  Scenario: 创建带单个目标的别名
    Given 不存在名为 "deepseek-chat" 的别名
    And 存在账号组 "g-deepseek"，上游为 "DeepSeek"
    When 提交别名 "deepseek-chat"、账号组 "g-deepseek" 与上游模型 "deepseek-chat"
    Then 别名 "deepseek-chat" 被创建且包含一个目标
    And 该目标的协议标记为 OpenAI Chat Completions

  @wip @catalog-create_alias_multiple_targets
  Scenario: 同一别名绑定多个账号组
    Given 别名 "deepseek-chat" 已存在且包含一个目标
    And 存在账号组 "g-openai"
    When 为别名追加账号组 "g-openai" 与上游模型 "gpt-5"
    Then 该别名包含两个目标
    And 两个目标的权重与优先级可分别设置

  @wip @catalog-create_alias_target_missing
  Scenario: 追加目标时别名不存在
    Given 不存在名为 "no-such-alias" 的别名
    When 为该别名追加账号组 "g-deepseek" 与上游模型 "deepseek-chat"
    Then 追加被拒绝且返回别名不存在
    And 不创建新的别名

  @wip @catalog-create_alias_duplicated
  Scenario: 别名已存在时拒绝创建
    Given 别名 "deepseek-chat" 已存在
    When 提交别名 "deepseek-chat"
    Then 创建被拒绝且返回别名已存在
    And 不创建新的别名

  @wip @catalog-create_alias_group_missing
  Scenario: 引用的账号组不存在时拒绝
    Given 不存在标识为 "g-missing" 的账号组
    When 提交别名 "orphan-alias" 并引用该账号组
    Then 创建被拒绝且返回账号组不存在
    And 不创建新的别名
