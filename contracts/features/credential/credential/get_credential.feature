@module-credential @use_case-get_credential
Feature: 查询凭证详情
  作为网关使用者
  我想要查看单个凭证的完整信息
  以便于确认配置是否正确

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @credential-get_credential_found
  Scenario: 查询存在的凭证
    Given 存在凭证 "deepseek-main"
    When 查询该凭证详情
    Then 返回名称、类型、上游、保留模型清单与健康状态
    And 密钥以掩码形式返回，不包含明文

  @wip @credential-get_credential_missing
  Scenario: 查询不存在的凭证
    Given 不存在标识为 "credential-9999" 的凭证
    When 查询该凭证详情
    Then 查询失败且返回凭证不存在
