# 转发

收下客户端请求，解析入站协议，转换为内部统一表示，再构造出站请求发往上游，并把响应转换回客户端协议的形态。

## 术语

**转发**:
一次请求从入站协议到出站协议的完整处理
等价: Relay
避免: 代理, 透传

**基准协议**:
Ferry 直接实现的四套线上协议，分别是 OpenAI Chat Completions、OpenAI Responses、Anthropic Messages 与 Gemini
等价: BaseProtocol
避免: 标准协议, 原生协议

**内部统一表示**:
Ferry 领域内部表达一次模型调用的语言
等价: CanonicalRequest
避免: 中间格式, 标准格式
