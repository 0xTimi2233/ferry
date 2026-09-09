@module-catalog @use_case-create_model_alias
Feature: 创建模型别名
  作为网关使用者
  我想要把一个模型名绑定到若干上游引用
  以便于客户端按模型名请求时网关能挑选上游

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @catalog-create_model_alias_created
  Scenario: 创建带单个上游引用的别名
    Given 不存在名为 "deepseek-chat" 的模型别名
    And 存在可用凭证 "deepseek-main"，上游为 "DeepSeek"
    When 提交别名 "deepseek-chat"、策略 "轮询"、凭证 "deepseek-main" 与上游模型 "deepseek-chat"
    Then 模型别名 "deepseek-chat" 被创建且包含一个上游引用
    And 该引用的协议标记为 OpenAI Chat Completions

  @wip @catalog-create_model_alias_multiple_refs
  Scenario: 同一别名绑定多个上游引用
    Given 模型别名 "deepseek-chat" 已存在且包含一个上游引用
    And 存在可用凭证 "deepseek-backup"
    When 为别名追加凭证 "deepseek-backup" 与上游模型 "deepseek-chat"
    Then 该别名包含两个上游引用
    And 两个引用的权重与优先级可分别设置

  @wip @catalog-create_model_alias_duplicated
  Scenario: 别名已存在时拒绝创建
    Given 模型别名 "deepseek-chat" 已存在
    When 提交别名 "deepseek-chat"
    Then 创建被拒绝且返回别名已存在
    And 不创建新的别名

  @wip @catalog-create_model_alias_credential_missing
  Scenario: 引用的凭证不存在时拒绝
    Given 不存在标识为 "credential-9999" 的凭证
    When 提交别名 "orphan-alias" 并引用该凭证
    Then 创建被拒绝且返回凭证不存在
    And 不创建新的别名

  @wip @catalog-create_model_alias_credential_disabled
  Scenario: 引用的凭证已禁用时拒绝
    Given 凭证 "openai-key-2" 健康状态为已禁用
    When 提交别名 "disabled-alias" 并引用该凭证
    Then 创建被拒绝且返回凭证不可用
