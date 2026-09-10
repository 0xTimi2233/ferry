@module-relay @use_case-select_credential
Feature: 选定凭证
  作为网关系统
  我想要为一次请求选定可用凭证
  以便于调用能落到合适的账号上

  Background:
    Given 网关已启动

  @wip @relay-select_credential_selected
  Scenario: 按账号组策略选定凭证
    Given 别名 "deepseek-chat" 指向账号组 "g-deepseek"，组策略为轮询
    And 该账号组有两个可用凭证
    When 为一次请求选定凭证
    Then 返回其中一个可用凭证
    And 连续两次选定会轮流返回不同凭证

  @wip @relay-select_credential_skip_cooling
  Scenario: 跳过冷却中的凭证
    Given 账号组 "g-deepseek" 的一个凭证处于冷却，另一个可用
    When 为一次请求选定凭证
    Then 返回可用凭证而非冷却凭证

  @wip @relay-select_credential_cooldown_recovered
  Scenario: 冷却到期后凭证重新可选
    Given 账号组 "g-deepseek" 的唯一凭证处于冷却且恢复时间已过
    When 为一次请求选定凭证
    Then 返回该凭证
    And 凭证健康状态回到可用

  @wip @relay-select_credential_paths
  Scenario: 别名有多个目标时按优先级与权重选定
    Given 别名 "deepseek-chat" 指向账号组 "g-deepseek" 与 "g-deepseek-backup"
    And 前者优先级为 1、权重为 1，后者优先级为 2、权重为 100
    When 为一次请求选定凭证
    Then 选定落在优先级更高的 "g-deepseek" 上
    And 权重更高的低优先级目标不会被选中

  @wip @relay-select_credential_session_affinity
  Scenario: 同一会话固定使用同一凭证
    Given 会话粘性已开启
    And 会话 "s-1" 已绑定到凭证 "deepseek-main"
    When 会话 "s-1" 再次请求
    Then 返回凭证 "deepseek-main"
    And 标记本次为命中会话粘性

  @wip @relay-select_credential_affinity_fallback
  Scenario: 绑定凭证不可用时重建绑定
    Given 会话粘性已开启
    And 会话 "s-1" 绑定到凭证 "deepseek-main"，该凭证变为冷却
    When 会话 "s-1" 再次请求
    Then 返回另一个可用凭证
    And 会话 "s-1" 的绑定更新为新凭证

  @wip @relay-select_credential_concurrency_limited
  Scenario: 并发达到上限时快速失败
    Given 账号组 "g-deepseek" 的在途并发上限为 2，已有两次调用在途
    When 为一次请求选定凭证
    Then 选定被拒绝且返回并发已满

  @wip @relay-select_credential_all_unavailable
  Scenario: 全部凭证不可用
    Given 别名 "deepseek-chat" 的全部凭证均处于冷却
    And 各凭证的恢复时间分别为 T1 与 T2，其中 T1 早于 T2
    When 为一次请求选定凭证
    Then 选定被拒绝且错误信息包含尝试过的凭证名称清单
    And 错误信息里的恢复时间取其中最晚的 T2
