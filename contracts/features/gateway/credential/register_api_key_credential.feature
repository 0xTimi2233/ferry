@module-credential @use_case-register_api_key_credential
Feature: 注册密钥凭证
  作为网关使用者
  我想要注册一个上游密钥凭证
  以便于网关能代表我调用该上游

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @credential-register_api_key_credential_created
  Scenario: 注册一个可用的密钥凭证
    Given 不存在名为 "deepseek-main" 的凭证
    When 提交名称 "deepseek-main"、上游 "DeepSeek" 与密钥 "sk-test-0001"
    Then 凭证 "deepseek-main" 被创建且健康状态为可用
    And 凭证自动归入该上游的账号组，组不存在时一并创建
    And 凭证的密钥以密文形式存储，读取凭证详情时只返回掩码
    And 发布领域事件 CredentialRegistered

  @wip @credential-register_api_key_credential_duplicated
  Scenario: 名称重复时拒绝注册
    Given 已存在名为 "deepseek-main" 的凭证
    When 提交名称 "deepseek-main"、上游 "DeepSeek" 与密钥 "sk-test-0002"
    Then 注册被拒绝且返回名称已存在
    And 不创建新的凭证
    And 不发布领域事件

  @wip @credential-register_api_key_credential_blank
  Scenario: 名称或密钥为空时拒绝注册
    Given 不存在名为 "" 的凭证
    When 提交名称 ""、上游 "DeepSeek" 与密钥 "sk-test-0003"
    Then 注册被拒绝且返回名称不能为空
    And 不创建新的凭证

  @wip @credential-register_api_key_credential_group_removed
  Scenario: 组内最后一个凭证被删除时移除账号组
    Given 账号组 "g-deepseek" 只包含凭证 "deepseek-backup"
    When 删除凭证 "deepseek-backup"
    Then 账号组 "g-deepseek" 被移除
    And 发布领域事件 CredentialGroupRemoved 供别名清理目标

  @wip @credential-register_api_key_credential_unknown_upstream
  Scenario: 上游不在支持范围内时拒绝注册
    Given 上游 "UnknownVendor" 未被支持
    When 提交名称 "unknown-main"、上游 "UnknownVendor" 与密钥 "sk-test-0004"
    Then 注册被拒绝且返回上游不受支持
    And 不创建新的凭证
