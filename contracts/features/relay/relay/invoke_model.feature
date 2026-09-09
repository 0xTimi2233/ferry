@module-relay @use_case-invoke_model
Feature: 发起模型调用
  作为网关使用者
  我想要让客户端通过网关调用模型
  以便于统一使用手上的凭证与订阅

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @relay-invoke_model_succeeded
  Scenario: 非流式调用成功
    Given 模型别名 "deepseek-chat" 存在且包含可用凭证
    When 客户端以访问密钥请求该别名，且不要求流式响应
    Then 返回上游的响应且已转换为客户端协议的形态
    And 发布领域事件 UsageRecorded，包含凭证、别名与 token 用量

  @wip @relay-invoke_model_streamed
  Scenario: 流式调用成功
    Given 模型别名 "gpt-5-codex" 存在且包含可用凭证
    When 客户端以访问密钥请求该别名，且要求流式响应
    Then 网关逐块转发上游事件直到流结束
    And 流结束后发布领域事件 UsageRecorded

  @wip @relay-invoke_model_alias_missing
  Scenario: 别名不存在时拒绝
    Given 不存在名为 "no-such-alias" 的模型别名
    When 客户端以访问密钥请求该别名
    Then 请求被拒绝且返回模型不存在
    And 不向上游发起请求

  @wip @relay-invoke_model_access_denied
  Scenario: 访问密钥无效时拒绝
    Given 客户端携带无效的访问密钥
    When 客户端请求模型别名 "deepseek-chat"
    Then 请求被拒绝且返回未授权
    And 不向上游发起请求

  @wip @relay-invoke_model_failover
  Scenario: 首个凭证失败时切换凭证
    Given 模型别名 "deepseek-chat" 包含凭证 "openai-key-2" 与 "deepseek-main"
    And 凭证 "openai-key-2" 返回限流且可重试
    When 客户端请求该别名
    Then 网关改用 "deepseek-main" 完成本次调用
    And 凭证 "openai-key-2" 进入冷却且记录原因

  @wip @relay-invoke_model_all_unavailable
  Scenario: 全部凭证不可用时返回统一错误
    Given 模型别名 "deepseek-chat" 的全部凭证均处于冷却状态
    When 客户端请求该别名
    Then 请求被拒绝且错误信息包含尝试过的凭证清单
    And 错误信息包含预计恢复时间

  @wip @relay-invoke_model_hard_quota
  Scenario: 凭证硬配额耗尽时不再重试
    Given 模型别名 "deepseek-chat" 包含凭证 "deepseek-main"
    And 该凭证返回配额耗尽且无重试时间
    When 客户端请求该别名
    Then 请求被拒绝且凭证标记为失效
    And 不对该凭证继续重试
