@module-metering @use_case-query_usage_summary
Feature: 查询用量汇总
  作为网关使用者
  我想要查看按天聚合的用量
  以便于了解消耗与趋势

  Background:
    Given 网关已启动且访问密钥已配置

  @wip @metering-query_usage_summary_by_day
  Scenario: 查询按天聚合的用量
    Given 最近七天存在用量记录
    When 查询近七天的用量汇总
    Then 返回每天一档的记录，包含请求次数、输入 token、输出 token 与折算金额

  @wip @metering-query_usage_summary_by_metric
  Scenario: 切换统计指标
    Given 最近七天存在用量记录
    When 以指标 "折算金额" 查询汇总
    Then 返回的每档记录以金额为主值

  @wip @metering-query_usage_summary_empty
  Scenario: 区间内没有记录
    Given 指定区间内不存在用量记录
    When 查询该区间的用量汇总
    Then 返回空区间，各天数值为零

  @wip @metering-query_usage_summary_excludes_subscription
  Scenario: 订阅凭证不计入金额
    Given 存在一条来自订阅凭证的用量记录
    When 查询用量汇总
    Then 该记录的金额计为零，token 用量正常计入

  @wip @metering-query_usage_summary_unpriced
  Scenario: 未定价的记录不计入金额合计
    Given 存在一条来自未定价密钥凭证的用量记录
    When 查询用量汇总
    Then 该档的折算金额缺省，表示合计不完整
    And token 用量正常计入
