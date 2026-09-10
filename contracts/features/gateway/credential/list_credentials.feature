@module-credential @use_case-list_credentials
Feature: 查询凭证列表
  作为网关使用者
  我想要查看全部凭证及其状态
  以便于了解哪些可用哪些异常

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @credential-list_credentials_listed
  Scenario: 列出全部凭证
    Given 存在三个凭证，其中一个处于冷却状态
    When 查询凭证列表
    Then 返回三条记录，每条包含名称、类型、上游、所属账号组、保留模型数量与健康状态
    And 处于冷却状态的记录附带冷却原因
    And 不返回密钥明文

  @wip @credential-list_credentials_groups
  Scenario: 列出账号组
    Given 存在两个账号组，分别属于 "DeepSeek" 与 "OpenAI"
    When 查询凭证列表
    Then 同时返回账号组清单，每个账号组包含标识、上游、策略与凭证数量

  @wip @credential-list_credentials_empty
  Scenario: 没有凭证时返回空列表
    Given 不存在任何凭证
    When 查询凭证列表
    Then 返回空列表

  @wip @credential-list_credentials_filtered
  Scenario: 按上游筛选
    Given 存在两家上游的凭证
    When 按上游 "DeepSeek" 筛选
    Then 只返回该上游的凭证

  @wip @credential-list_credentials_filtered_empty
  Scenario: 按上游筛选无匹配时返回空列表
    Given 只存在上游 "OpenAI" 的凭证
    When 按上游 "DeepSeek" 筛选
    Then 返回空列表
