@module-catalog @use_case-create_account_pool
Feature: 创建账号池
  作为网关使用者
  我想要把一个模型名绑定到若干上游引用
  以便于客户端按模型名请求时网关能挑选上游

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @catalog-create_account_pool_created
  Scenario: 创建带单个上游引用的账号池
    Given 不存在名为 "deepseek-chat" 的账号池
    And 存在可用凭证 "deepseek-main"，上游为 "DeepSeek"
    When 提交账号池 "deepseek-chat"、策略 "轮询"、凭证 "deepseek-main" 与上游模型 "deepseek-chat"
    Then 账号池 "deepseek-chat" 被创建且包含一个上游引用
    And 该引用的协议标记为 OpenAI Chat Completions

  @wip @catalog-create_account_pool_multiple_refs
  Scenario: 同一账号池绑定多个上游引用
    Given 账号池 "deepseek-chat" 已存在且包含一个上游引用
    And 存在可用凭证 "deepseek-backup"
    When 为账号池追加凭证 "deepseek-backup" 与上游模型 "deepseek-chat"
    Then 该账号池包含两个上游引用
    And 两个引用的权重与优先级可分别设置

  @wip @catalog-create_account_pool_duplicated
  Scenario: 账号池已存在时拒绝创建
    Given 账号池 "deepseek-chat" 已存在
    When 提交账号池 "deepseek-chat"
    Then 创建被拒绝且返回账号池已存在
    And 不创建新的账号池

  @wip @catalog-create_account_pool_credential_missing
  Scenario: 引用的凭证不存在时拒绝
    Given 不存在标识为 "credential-9999" 的凭证
    When 提交账号池 "orphan-pool" 并引用该凭证
    Then 创建被拒绝且返回凭证不存在
    And 不创建新的账号池

  @wip @catalog-create_account_pool_credential_disabled
  Scenario: 引用的凭证已禁用时拒绝
    Given 凭证 "openai-key-2" 健康状态为已禁用
    When 提交账号池 "disabled-pool" 并引用该凭证
    Then 创建被拒绝且返回凭证不可用
