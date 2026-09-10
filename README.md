# ferry

个人自用的 AI 网关。统一管理上游凭证与 AI 订阅，让 Codex、Pi Agent 这类编码客户端只接一个入口，按模型名调度到不同上游。

## 能力

- 凭证管理：密钥凭证与订阅凭证，支持订阅授权、令牌刷新与模型清单同步
- 模型目录：以别名聚合账号组与上游模型，客户端只见别名
- 模型调用：按别名调度到可用凭证，支持流式响应与失败重试
- 计量与设置：用量汇总、请求日志、访问密钥与服务端设置

## 文档

| 路径 | 内容 |
|---|---|
| [docs/product/vision.md](docs/product/vision.md) | 产品愿景与目标 |
| [docs/architecture/overview.md](docs/architecture/overview.md) | 系统架构全景 |
| [docs/design/ui.md](docs/design/ui.md) | 视觉规范与设计令牌 |
| [docs/design/ux.md](docs/design/ux.md) | 交互规范 |
| [docs/adr/](docs/adr/) | 架构决策记录 |
| [context.md](context.md) | 限界上下文的统一语言 |
| [contracts/](contracts/) | 接口契约与业务验收契约 |
| [server/gateway/](server/gateway/) | 网关上下文实现 |

## 开发

工具链统一走 just。

```bash
just ci        # 格式、静态检查、测试、依赖审计
just check     # 仅格式检查
just lint      # 仅静态检查
just test      # 仅测试
just audit     # 仅依赖与合规审计
```

接口形状的单一真源是 `contracts/proto/`，构建时由 `server/contracts` 的 `build.rs` 生成 Rust 类型，Rust 侧不需要 `protoc`。业务行为契约以 `.feature` 文件承载，执行载体由支撑切片落地，统一入口为 `just test-contracts`。
