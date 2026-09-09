@module-credential @use_case-update_credential_models
Feature: 选择保留的模型
  作为网关使用者
  我想要指定凭证保留哪些模型
  以便于只暴露需要的能力

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @credential-update_credential_models_updated
  Scenario: 更新凭证保留的模型
    Given 凭证 "deepseek-main" 当前保留 "deepseek-chat" 与 "deepseek-reasoner"
    When 提交保留清单 "deepseek-chat" 与 "deepseek-coder"
    Then 凭证保留的模型变为 "deepseek-chat" 与 "deepseek-coder"

  @wip @credential-update_credential_models_empty
  Scenario: 保留清单为空时拒绝
    Given 凭证 "deepseek-main" 当前保留两个模型
    When 提交空的保留清单
    Then 更新被拒绝且返回至少保留一个模型
    And 凭证保留的模型不变

  @wip @credential-update_credential_models_not_offered
  Scenario: 提交上游未提供的模型时拒绝
    Given 凭证 "deepseek-main" 的上游未提供 "gpt-5"
    When 提交保留清单 "gpt-5"
    Then 更新被拒绝且返回模型不在上游清单中
    And 凭证保留的模型不变
