@module-credential @use_case-fetch_upstream_models
Feature: 拉取上游模型清单
  作为网关使用者
  我想要从上游拉取可用模型清单
  以便于选择要暴露给客户端的模型

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @credential-fetch_upstream_models_fetched
  Scenario: 从上游拉取模型清单
    Given 存在可用的凭证 "deepseek-main"，上游为 "DeepSeek"
    When 对该凭证发起模型拉取
    Then 返回上游声明的模型清单
    And 凭证上已保留的模型标记为选中

  @wip @credential-fetch_upstream_models_unavailable
  Scenario: 凭证不可用时拉取失败
    Given 凭证 "openai-key-2" 处于冷却状态
    When 对该凭证发起模型拉取
    Then 拉取被拒绝且返回凭证当前不可用
    And 不修改已保留的模型

  @wip @credential-fetch_upstream_models_upstream_error
  Scenario: 上游返回错误时保留原有清单
    Given 存在可用的凭证 "deepseek-main"
    And 上游 "DeepSeek" 返回服务不可用
    When 对该凭证发起模型拉取
    Then 拉取失败且错误信息里的上游字段为 "DeepSeek"
    And 凭证已保留的模型不变
