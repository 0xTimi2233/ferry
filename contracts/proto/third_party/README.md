# 第三方 proto

本目录存放契约依赖的上游 proto 定义，来自 [googleapis/googleapis](https://github.com/googleapis/googleapis)，许可为 Apache License 2.0。

| 文件 | 用途 |
|---|---|
| `google/api/http.proto` | `HttpRule` 与路径模板文法 |
| `google/api/annotations.proto` | `google.api.http` 方法选项，用于声明管理端点路由 |

生成链在 `server/contracts/build.rs` 中把本目录作为 include 根，因此 Rust 侧不需要安装 `protoc`，也不需要 `googleapis` 的其余文件。升级时直接替换文件内容，并重新运行 `just ci`。
