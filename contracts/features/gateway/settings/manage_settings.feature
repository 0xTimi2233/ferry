@module-settings @use_case-manage_settings
Feature: 管理访问密钥与服务端设置
  作为网关使用者
  我想要管理访问密钥与服务端参数
  以便于控制谁能接入以及数据如何保留

  Background:
    Given 网关已启动

  @wip @settings-manage_settings_query
  Scenario: 查询当前设置
    Given 网关已完成初始化
    When 查询设置
    Then 返回监听地址、访问密钥掩码、统计粒度与保留期
    And 返回会话粘性是否开启

  @wip @settings-manage_settings_update
  Scenario: 更新服务端设置
    Given 统计粒度为按天，保留期为 90 天
    When 提交保留期为 180 天
    Then 保留期变为 180 天
    And 其余设置不变

  @wip @settings-manage_settings_rotate_key
  Scenario: 重新生成访问密钥
    Given 已存在一个访问密钥
    When 重新生成访问密钥
    Then 响应中的新密钥字段非空且长度为 32 个字符
    And 再次查询设置时该字段为空
    And 旧密钥立即失效

  @wip @settings-manage_settings_affinity
  Scenario: 切换会话粘性
    Given 会话粘性已开启
    When 提交会话粘性为关闭
    Then 会话粘性变为关闭
    And 其余设置不变

  @wip @settings-manage_settings_retention_invalid
  Scenario: 保留期取值非法时拒绝
    Given 统计粒度为按天
    When 提交保留期为 "0 天"
    Then 更新被拒绝且返回保留期取值非法
    And 设置不变
