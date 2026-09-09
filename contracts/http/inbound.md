# 入站承载面

所有入站请求走同一套约定，切片只声明自己的路径与方法。

## 通用约定

| 项 | 约定 |
|---|---|
| 鉴权 | 请求头 `Authorization: Bearer <访问密钥>`，校验失败返回 401 |
| 会话标识 | 请求头 `X-Session-Id`，缺省时按请求体内容哈希派生 |
| 请求 ID | 请求头 `X-Request-Id`，缺省时由网关生成，响应回传同名头 |
| 响应结构 | 成功直接返回协议原生结构；失败返回 `{ "error": { "code", "message", "upstream" } }` |
| 上游水位 | 上游的 `retry-after` 与 `x-ratelimit-*` 原样透传 |

## 协议端点

| 方法 | 路径 | 入站协议 |
|---|---|---|
| POST | `/v1/chat/completions` | OpenAI Chat Completions |
| POST | `/v1/responses` | OpenAI Responses |
| POST | `/v1/messages` | Anthropic Messages |
| POST | `/v1beta/models/{model}:generateContent` | Gemini |

## 管理端点

| 方法 | 路径 | 用例 |
|---|---|---|
| GET | `/admin/credentials` | list_credentials |
| GET | `/admin/credentials/{id}` | get_credential |
| POST | `/admin/credentials/api-key` | register_api_key_credential |
| POST | `/admin/credentials/subscription/start` | start_subscription_authorization |
| POST | `/admin/credentials/subscription/complete` | complete_subscription_authorization |
| POST | `/admin/credentials/{id}/refresh` | refresh_subscription_token |
| GET | `/admin/credentials/{id}/models` | fetch_upstream_models |
| PUT | `/admin/credentials/{id}/models` | update_credential_models |
| PATCH | `/admin/credentials/{id}/health` | change_credential_health |
| DELETE | `/admin/credentials/{id}` | delete_credential |
| PATCH | `/admin/credential-groups/{id}` | configure_credential_group |
| GET | `/admin/aliases` | list_aliases |
| POST | `/admin/aliases` | create_alias |
| POST | `/admin/aliases/{name}/targets` | create_alias 追加目标 |
| DELETE | `/admin/aliases/{name}` | delete_alias |
| GET | `/admin/usage` | query_usage_summary |
| GET | `/admin/logs` | query_request_logs |
| GET | `/admin/settings` | manage_settings |
| PATCH | `/admin/settings` | manage_settings |

## 错误码映射

| 用例错误 | HTTP 状态码 | 附带头 |
|---|---|---|
| InvalidInput | 400 | — |
| Unauthorized | 401 | — |
| NotFound | 404 | — |
| ConcurrencyLimited | 429 | `Retry-After` 为变体给出的秒数 |
| AllCredentialsUnavailable | 503 | `Retry-After` 为 `recover_at` 距当前的秒数，为 None 时不带该头 |
| Domain，其余业务规则拒绝 | 409 | — |
| Port | 502 | — |
