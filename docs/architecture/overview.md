# 系统架构全景

限界上下文划分的模块化单体，内部按 CQRS 垂直切片组织，切片自带入站适配器与用例处理器，入站端口以处理器签名隐式表达；领域层为 DDD 战术模式的充血模型；出站依赖通过端口适配器倒置，端口按读写分离。

## 架构拓扑

```mermaid
flowchart TB
    subgraph 客户端
        C1[Codex CLI]
        C2[Pi Agent]
        C3[其他 OpenAI 兼容客户端]
    end

    subgraph 单二进制
        ACC[访问<br/>密钥校验]
        subgraph 协议适配
            IN[入站协议解析]
            CANON[内部统一表示]
            OUT[出站协议构造]
        end
        CAT[模型目录]
        ROUTE[调度]
        CRED[凭证]
        METER[计量]
        WEB[管理界面<br/>内嵌静态资源]
    end

    subgraph 上游
        U1[DeepSeek]
        U2[OpenAI 订阅]
    end

    C1 --> ACC
    C2 --> ACC
    C3 --> ACC
    ACC --> IN
    IN --> CANON
    CANON --> OUT
    CANON --> CAT
    CAT --> ROUTE
    ROUTE --> CRED
    OUT --> U1
    OUT --> U2
    OUT -.用量事件.-> METER
    CRED -.健康变更.-> ROUTE
    WEB --> CRED
    WEB --> CAT
    WEB --> METER
```

协议适配是唯一接触线上协议的地方，入站协议与出站协议都先翻译成内部统一表示再向外，领域模型不感知任何上游协议的字段。

## 技术栈选型

- 运行时与语言：Rust 1.98，单二进制交付，跨 Linux、macOS、Windows
- HTTP 主干：axum + hyper + tower-http + reqwest + tokio。axum 类型隔离在薄适配层，业务逻辑不直接依赖，以应对其 0.9 的破坏性变更
- 持久化：SQLite，WAL 模式，sqlx 访问，迁移脚本随仓库版本化
- 凭据加密：信封加密。每条凭据用随机数据密钥加密，数据密钥再由主密钥包裹后落盘，主密钥取自操作系统凭据库；端口以 `SealedSecret` 承载包裹后的密钥与密文
- 可观测：tracing 做结构化日志，metrics 做指标导出
- 配置：文件承载，SIGHUP 触发重载，arc-swap 原子替换，保留 last-known-good 快照
- 前端：React 19 + Vite 8 + Tailwind v4 + shadcn/ui + TanStack Query + TanStack Table + recharts，构建产物由 rust-embed 嵌入
- 工具链：cargo 构建，just 统一入口，Biome 管前端，cargo-deny 管依赖

## 基础设施

### 入站承载面

入站请求走同一套约定，切片只声明自己的路径与方法。

| 项 | 约定 | 归属 |
|---|---|---|
| 鉴权 | 请求头 `Authorization: Bearer <访问密钥>`，校验失败返回 401 | 组装根中间件 |
| 会话标识 | 请求头 `X-Session-Id`，缺省时按请求体内容哈希派生 | 组装根中间件 |
| 请求 ID | 请求头 `X-Request-Id`，缺省时由网关生成，响应回传同名头 | 组装根中间件 |
| 错误结构 | 失败返回 `{ "error": { "code", "message", "upstream" } }` | 组装根中间件，由 `UseCaseError` 分档映射 |
| 上游水位 | 上游的 `Retry-After` 与 `x-ratelimit-*` 原样透传 | 组装根中间件 |

鉴权校验不是领域行为。ferry 为单用户单密钥，无角色与权限矩阵，因此不划入任何用例；访问密钥的生成、轮换与持久化是领域行为，落在 `manage_settings` 用例。出现多用户、角色、权限矩阵或租户隔离时，才拆分独立的身份上下文。

接口形状与路径的单一真源是 `contracts/proto/gateway/v1/`，`server/contracts/` 以 `protox` 与 `prost-build` 生成 Rust 类型并输出到 `OUT_DIR`。成功态统一返回 200，失败状态码由 `UseCaseError` 分档与 HTTP 的统一映射表决定，两张表都在组装根。

### 运行时关注点

- 错误处理：领域错误按可恢复性分级，对外统一为协议规定的错误结构，不把上游原始错误直接透传给客户端；上游错误在诊断信息里保留来源
- 日志与追踪：每次请求注入请求 ID 与会话 ID，凭证与密钥在日志中脱敏，流式响应记录首字节延迟与总时长
- 超时：分层设置，连接、首字节、单次读、整体上限各自独立，不用单一 deadline，四者以 `UpstreamCall.timeouts` 表达
- 限流与冷却：在途并发用信号量按凭证与账号组两级限制；上游 429 作为配额水位信号处理，读取 `Retry-After` 与配额头，区分可重试的短窗口限流与不可重试的硬配额耗尽，冷却阶梯按窗口去重并叠加抖动，上游水位头经 `UpstreamResponse.rate_limit_headers` 透传给客户端
- 流式响应的配额耗尽：响应头提交前缓冲握手事件，发现终局拒绝时返回可重试的 HTTP 错误；已提交后走流内错误事件，槽位释放绑定请求取消
- 优雅关闭：收到终止信号后停止接受新请求，等待在途流式响应结束或超时

## 部署形态

单个二进制，前台进程，监听地址可配。数据文件与配置放在同一台机器上，不跨网络访问数据库。管理界面静态资源在 release 构建期嵌入，debug 期从文件系统读取。

## 用例编排

`invoke_model` 在需要选定账号时调用 `select_credential`，两者各自实现、各自验收，前者对后者注入假实现即可自证。
