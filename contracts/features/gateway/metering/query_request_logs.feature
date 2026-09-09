@module-metering @use_case-query_request_logs
Feature: 查询请求日志
  作为网关使用者
  我想要查看请求记录
  以便于排查失败原因

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @metering-query_request_logs_listed
  Scenario: 查询最近的请求记录
    Given 存在若干请求记录
    When 查询请求日志
    Then 按时间倒序返回记录，每条包含时间、账号池、凭证、输入 token、输出 token、耗时与结果
    And 失败的记录附带失败原因

  @wip @metering-query_request_logs_filtered
  Scenario: 按账号池或凭证筛选
    Given 存在涉及两个账号池的请求记录
    When 按账号池 "deepseek-chat" 筛选
    Then 只返回该账号池的记录

  @wip @metering-query_request_logs_only_failed
  Scenario: 只看失败记录
    Given 存在成功与失败的请求记录
    When 以只看失败筛选
    Then 只返回失败的记录

  @wip @metering-query_request_logs_empty
  Scenario: 没有记录时返回空列表
    Given 不存在任何请求记录
    When 查询请求日志
    Then 返回空列表
