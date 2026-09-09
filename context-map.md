# 上下文地图

## 上下文清单

- [访问](./server/access/context.md) — 校验访问密钥，拒绝未授权请求
- [转发](./server/relay/context.md) — 解析入站协议，构造出站请求，回传响应
- [模型目录](./server/catalog/context.md) — 维护模型别名与上游引用的对应关系
- [调度](./server/routing/context.md) — 选取凭证，处理重试与会话粘性
- [凭证](./server/credential/context.md) — 管理上游凭证的生命周期与健康状态
- [计量](./server/metering/context.md) — 记录用量并折算成本

## 关系模式

- **访问 [U] → 转发 [D]**
  - **命令**：校验通过后放行请求
- **转发 [U] → 模型目录 [D, ACL]**
  - **查询**：解析模型别名，取得上游引用清单
- **转发 [U] → 调度 [D, ACL]**
  - **查询**：为某个上游引用选取凭证
- **凭证 [U, PL] → 调度 [D, ACL]**
  - **事件**：发布 HealthChanged，调度订阅后调整可用性
- **转发 [U, PL] → 计量 [D, ACL]**
  - **事件**：发布 UsageRecorded，计量订阅后落账

标记说明：U=上游，D=下游，PL=发布者语言，ACL=防腐层
