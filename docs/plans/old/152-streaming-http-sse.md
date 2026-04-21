# Plan 152: 流式 HTTP 与 SSE 解析

**Status**: Completed
**Created**: 2026-04-01
**Priority**: High (Phase 1 LLM API 阻塞性依赖)
**Completed**: 2026-04-01

## 目标

实现流式 HTTP 客户端和 Server-Sent Events (SSE) 解析能力，为 LLM API 流式调用提供基础设施支持。

---

## 背景与动机

### 为什么需要流式 HTTP？

传统的 HTTP 请求是**请求-响应**模式：
1. 发送请求
2. 等待完整响应
3. 一次性接收所有数据

对于 LLM API 来说，这意味着：
- ❌ 用户需要等待整个响应生成完成
- ❌ 无法实现"打字机效果"的实时输出
- ❌ 无法处理超长响应（内存占用高）

**流式 HTTP** 解决了这些问题：
- ✅ 数据分块传输，实时显示
- ✅ 降低内存占用
- ✅ 更好的用户体验

### Server-Sent Events (SSE)

SSE 是流式 HTTP 的一种标准格式，被广泛用于：
- LLM API 流式响应（Anthropic、OpenAI）
- 实时事件推送
- 服务器状态更新

**SSE 格式**：
```
data: {"content": "Hello", "delta": "Hello"}

data: {"content": " World", "delta": " World"}

data: [DONE]
```

---

## 核心设计

### 架构层次

```
┌─────────────────────────────────────────────────────────────┐
│                    AutoLang 应用层                         │
│  chat_stream(provider, messages).await                       │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│              流式 HTTP 客户端 (Plan 152)                     │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ HTTPStream 类型                                     │   │
│  │ - async fn request(url) ~HTTPStream                  │   │
│  │ - async fn next() ~StreamEvent                       │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ SSE 解析器                                            │   │
│  │ - fn parse_sse(chunk) ~[]SSEEvent                   │   │
│  │ - 事件缓冲处理                                       │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│                    stdlib/auto/http.at                     │
│  (扩展现有 HTTP 客户端，添加流式支持)                        │
└─────────────────────────────────────────────────────────────┘
```

---

## 实施计划

### Phase 1: HTTPStream 类型 (2天)

#### 1.1 Auto 标准库扩展

```auto
/// stdlib/auto/http_stream.at

/// HTTP 流式响应类型
type HTTPStream

/// 流式事件
type StreamEvent

/// 创建流式 HTTP GET 请求
#[pub]
fn http_get_stream(url str) ~HTTPStream

/// 创建流式 HTTP POST 请求
#[pub]
fn http_post_stream(url str, body str) ~HTTPStream

/// HTTPStream 方法

/// 读取下一个事件（异步）
#[pub]
fn HTTPStream.next(self HTTPStream) ~StreamEvent

/// 检查流是否结束
#[pub]
fn HTTPStream.is_done(self HTTPStream) int

/// 关闭流
#[pub]
fn HTTPStream.close(self HTTPStream) void
```

#### 1.2 Rust 实现 (VM FFI)

```rust
// crates/auto-lang/src/vm/ffi/stdlib.rs

/// HTTPStream 值 - 包装底层的 tokio stream
pub enum VmValue {
    // ... 现有变体
    
    // Plan 152: 流式 HTTP
    HTTPStream(Rc<RefCell<HttpStreamInner>>),
}

pub struct HttpStreamInner {
    reader: tokio_util::io::StreamReader<reqwest::Upgraded>,
    buffer: Vec<u8>,
    done: bool,
}

/// NATIVE_HTTP_GET_STREAM (2401)
#[no_mangle]
pub extern "C" fn fn http_get_stream(
    task: &mut AutoTask,
    _vm: &AutoVM,
) -> Result<VmValue, VMError> {
    // 1. 从栈获取 URL 参数
    let url_str = task.stack.pop_string()?;
    
    // 2. 解析 URL
    let url = reqwest::Url::parse(&url_str)
        .map_err(|e| VMError::RuntimeError(format!("Invalid URL: {}", e)))?;
    
    // 3. 创建异步 HTTP 客户端
    task.runtime.block_on(async {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await
            .map_err(|e| VMError::RuntimeError(format!("HTTP error: {}", e)))?;
        
        // 4. 升级到流式连接
        let upgraded = response.upgrade().await
            .map_err(|e| VMError::RuntimeError(format!("Upgrade error: {}", e)))?;
        
        let reader = tokio_util::io::StreamReader::new(upgraded);
        
        Ok(VmValue::HTTPStream(Rc::new(RefCell::new(HttpStreamInner {
            reader,
            buffer: Vec::new(),
            done: false,
        })))
    })
}

/// NATIVE_HTTP_STREAM_NEXT (2402)
#[no_mangle]
pub extern "C" fn http_stream_next(
    task: &mut AutoTask,
    _vm: &AutoVM,
) -> Result<VmValue, VMError> {
    // 1. 从栈获取 HTTPStream 参数
    let stream_rc = task.stack.pop::<Rc<RefCell<HttpStreamInner>>>()?;
    let mut stream = stream_rc.borrow_mut();
    
    // 2. 读取下一个 SSE 事件
    task.runtime.block_on(async {
        let mut reader = &mut stream.reader;
        
        // 读取一行（SSE 格式：`data: {...}\n\n`）
        let mut line = String::new();
        reader.read_line(&mut line).await
            .map_err(|e| VMError::RuntimeError(format!("Read error: {}", e)))?;
        
        // 3. 解析 SSE 事件
        if line.starts_with("data: ") {
            let json_str = &line[6..]; // 跳过 "data: "
            
            // 检查结束标记
            if json_str == "[DONE]" {
                stream.done = true;
                return Ok(VmValue::String("done".to_string()));
            }
            
            // 解析 JSON
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                let content = json["content"].as_str().unwrap_or("");
                return Ok(VmValue::String(content.to_string()));
            }
        }
        
        // 空行表示事件结束
        Ok(VmValue::String(""))
    })
}
```

### Phase 2: SSE 解析器 (2天)

#### 2.1 Auto 标准库扩展

```auto
/// stdlib/auto/sse.at

/// SSE 事件类型
type SSEEvent

/// 创建 SSE 事件
#[pub]
fn sse_event(data str, event str, id str) SSEEvent

/// SSE 事件方法

/// 获取事件数据
#[pub]
fn SSEEvent.data(self SSEEvent) str

/// 获取事件类型
#[pub]
fn SSEEvent.event(self SSEEvent) str

/// 获取事件 ID
#[pub]
fn SSEEvent.id(self SSEEvent) str

/// 解析 SSE 文本块
#[pub]
fn parse_sse(chunk str) ~[]SSEEvent

/// 流式 HTTP 请求构建器
type HTTPStreamRequest

/// 设置请求头
#[pub]
fn HTTPStreamRequest.header(self HTTPStreamRequest, key str, value str) HTTPStreamRequest

/// 设置请求体
#[pub]
fn HTTPStreamRequest.body(self HTTPStreamRequest, body str) HTTPStreamRequest

/// 发送请求（返回流式响应）
#[pub]
fn HTTPStreamRequest.send(self HTTPStreamRequest) ~HTTPStream
```

#### 2.2 SSE 解析实现

```rust
// crates/auto-lang/src/sse/parser.rs

use std::io::{BufRead, BufReader};

/// SSE 事件
#[derive(Debug, Clone)]
pub struct SSEEvent {
    pub id: Option<String>,
    pub event: Option<String>,
    pub data: String,
    pub retry: Option<u32>,
}

/// SSE 解析错误
#[derive(Debug, thiserror::Error)]
pub enum SSEError {
    #[error("Invalid SSE format: {0}")]
    InvalidFormat(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// 解析 SSE 文本块
pub fn parse_sse_chunk(chunk: &str) -> Result<Vec<SSEEvent>, SSEError> {
    let mut events = Vec::new();
    let mut current_event = SSEEvent {
        id: None,
        event: None,
        data: String::new(),
        retry: None,
    };
    
    for line in chunk.lines() {
        if line.is_empty() {
            // 空行表示事件结束
            if !current_event.data.is_empty() {
                events.push(current_event.clone());
                current_event.data = String::new();
            }
            continue;
        }
        
        if let Some(rest) = line.strip_prefix("data:") {
            current_event.data.push_str(rest.trim());
            continue;
        }
        
        if let Some(rest) = line.strip_prefix("event:") {
            current_event.event = Some(rest.trim().to_string());
            continue;
        }
        
        if let Some(rest) = line.strip_prefix("id:") {
            current_event.id = Some(rest.trim().to_string());
            continue;
        }
        
        if let Some(rest) = line.strip_prefix("retry:") {
            current_event.retry = rest.trim().parse().ok();
            continue;
        }
    }
    
    // 最后一个事件
    if !current_event.data.is_empty() {
        events.push(current_event);
    }
    
    Ok(events)
}

/// SSE 流解析器
pub struct SSEParser<R: BufRead> {
    reader: R,
    buffer: String,
}

impl<R: BufRead> SSEParser<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: String::new(),
        }
    }
    
    /// 解析下一个事件
    pub fn next_event(&mut self) -> Result<Option<SSEEvent>, SSEError> {
        loop {
            // 检查 buffer 中是否有完整事件
            if let Some(events) = self.parse_buffer() {
                return Ok(Some(events));
            }
            
            // 读取更多数据
            let mut chunk = String::new();
            let n = self.reader.read_line(&mut chunk)?;
            if n == 0 {
                return Ok(None); // EOF
            }
            
            self.buffer.push_str(&chunk);
        }
    }
    
    fn parse_buffer(&mut self) -> Result<Option<Vec<SSEEvent>>, SSEError> {
        // 查找双换行（事件分隔符）
        while let Some(pos) = self.buffer.find("\n\n") {
            let chunk = &self.buffer[..pos];
            self.buffer = self.buffer[pos + 2..].to_string();
            
            if !chunk.trim().is_empty() {
                return Ok(Some(parse_sse_chunk(chunk)?));
            }
        }
        
        Ok(None)
    }
}
```

### Phase 3: LLM Provider 集成 (1.5天)

#### 3.1 Anthropic 流式 API

```rust
// crates/auto-shell/src/llm/anthropic_stream.rs

use crate::llm::types::{Message, MessageRole};
use crate::sse::parser::{SSEEvent, parse_sse_chunk};
use futures::stream::Stream;

pub struct AnthropicStreamProvider {
    config: LLMConfig,
    client: reqwest::Client,
}

impl AnthropicStreamProvider {
    pub async fn chat_stream(
        &self,
        messages: &[Message],
    ) -> impl Stream<Item = Result<StreamEvent, reqwest::Error>> {
        let request = AnthropicStreamRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            stream: true,
        };
        
        let response = self.client
            .post(format!("{}/v1/messages", self.base_url()))
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .expect("Failed to send request");
        
        let upgraded = response
            .upgrade()
            .await
            .expect("Failed to upgrade");
        
        let reader = tokio_util::io::StreamReader::new(upgraded);
        
        // 创建 SSE 流
        futures::stream::unfold(reader, move |mut reader| async move {
            let mut line = String::new();
            reader.read_line(&mut line).await.ok()?;
            
            if line.starts_with("data: ") {
                let json_str = &line[6..];
                
                if json_str == "[DONE]" {
                    return Some((StreamEvent::Done, None));
                }
                
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(content_block) = json.get("delta") {
                        if let Some(text) = content_block.get("text") {
                            if let Some(text_str) = text.as_str() {
                                return Some((
                                    StreamEvent::ContentDelta(text_str.to_string()),
                                    Some(reader),
                                ));
                            }
                        }
                    }
                    
                    if let Some(content_block) = json.get("content_block") {
                        if let Some(content_block_start) = content_block.get("content_block") {
                            if let Some(text) = content_block_start.get("text") {
                                if let Some(text_str) = text.as_str() {
                                    return Some((
                                        StreamEvent::ContentDelta(text_str.to_string()),
                                        Some(reader),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            
            Some((StreamEvent::Heartbeat, Some(reader)))
        })
    }
    
    fn base_url(&self) -> String {
        self.config.base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com")
    }
}

/// 流式事件
#[derive(Debug, Clone)]
pub enum StreamEvent {
    ContentDelta(String),
    ToolUse(ToolUseCall),
    Done,
    Heartbeat,
}
```

#### 3.2 Auto FFI 绑定

```auto
/// stdlib/auto/llm.at (扩展)

/// 流式聊天请求
/// 返回 ~HTTPStream
#[pub]
fn chat_stream_anthropic(
    provider LLMProvider,
    messages []Message
) ~HTTPStream

/// 从流中读取下一个内容增量
/// 返回 (content str, done int)
#[pub]
fn http_stream_read_content(stream HTTPStream) ~str

/// 检查流是否完成
#[pub]
fn http_stream_is_done(stream HTTPStream) int
```

### Phase 4: 测试与文档 (1.5天)

#### 4.1 单元测试

```rust
// crates/auto-shell/src/sse/tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sse_parsing() {
        let sse_text = r#"
data: {"content": "Hello"}
data: {"content": " World"}

data: [DONE]
"#;
        
        let events = parse_sse_chunk(sse_text).unwrap();
        assert_eq!(events.len(), 2);
        
        assert_eq!(events[0].data, r#"{"content": "Hello"}"#);
        assert_eq!(events[1].data, r#"{"content": " World"}"#);
    }
    
    #[test]
    fn test_sse_with_retry() {
        let sse_text = r#"
id: 123
retry: 5000
event: message
data: Test message

"#;
        
        let events = parse_sse_chunk(sse_text).unwrap();
        assert_eq!(events[0].id, Some("123".to_string()));
        assert_eq!(events[0].retry, Some(5000));
        assert_eq!(events[0].event, Some("message".to_string()));
    }
    
    #[tokio::test]
    async fn test_http_stream() {
        // 测试实际的 HTTP 流式请求
        // 需要一个测试 SSE 服务器
    }
}
```

#### 4.2 集成测试

```auto
// 测试用例: test_llm_stream.at

use llm: { anthropic, chat_stream, user_message }
use http_stream: { read_content, is_done }

task StreamTestAgent {
    on(ctx) {
        Test => {
            let provider = anthropic("sk-ant-test", "claude-3-5-sonnet-20241022")
            let messages = [user_message("Say 'Hello' in one word.")]
            
            let stream = chat_stream(provider, messages).await.
            
            mut full_content = ""
            while !is_done(stream) {
                let chunk = read_content(stream).await.
                full_content += chunk
            }
            
            ctx.reply(full_content)
        }
    }
}
```

---

## 依赖关系

### 输入依赖

| 计划 | 描述 | 状态 |
|------|------|------|
| Plan 121 | Task/Msg 基础系统 | ✅ 完成 |
| Plan 124 | `~T` Future 类型 | ✅ 完成 |
| Plan 126 | `.go` 并发 | ✅ 完成 |

### 输出依赖

| 计划 | 描述 | 阶塞性 |
|------|------|--------|
| **Plan 153: Phase 1 LLM API** | LLM API 基础实现 | 🔴 阻塞 |
| Phase 2: Agent Task | Agent Task 实现 | 🟡 部分（需要流式） |

---

## 文件清单

### 新增文件

```
crates/auto-lang/src/sse/
├── mod.rs              # 模块导出 (~50 lines)
├── parser.rs           # SSE 解析器 (~200 lines)
├── types.rs            # SSE 类型定义 (~50 lines)
└── tests.rs            # 测试 (~100 lines)

crates/auto-shell/src/llm/
├── stream.rs           # 流式 Provider 实现 (~300 lines)
└── tests.rs            # 集成测试 (~100 lines)

stdlib/auto/
├── http_stream.at      # HTTP 流式客户端 (~150 lines)
└── sse.at              # SSE 解析器 (~100 lines)

test/
└── llm_stream.at       # 集成测试 (~50 lines)
```

### 修改文件

```
crates/auto-lang/Cargo.toml      # 添加 futures, tokio-util 依赖
crates/auto-shell/Cargo.toml    # 添加流式相关依赖
crates/auto-lang/src/vm/ffi/stdlib.rs  # 添加 HTTP_STREAM opcodes
```

---

## Cargo.toml 依赖

### auto-lang

```toml
[dependencies]
# 现有依赖...

# Plan 152: 流式 HTTP
tokio-util = { version = "0.7", features = ["io"] }
futures = "0.3"
```

### auto-shell

```toml
[dependencies]
# 现有依赖...

# Plan 152: 流式 LLM
tokio-util = { version = "0.7", features = ["io"] }
futures = "0.3"
```

---

## 时间估算

| Phase | 任务 | 工作量 | 依赖 |
|-------|------|--------|------|
| **Phase 1** | HTTPStream 类型 | 2天 | 无 |
| **Phase 2** | SSE 解析器 | 2天 | Phase 1 |
| **Phase 3** | LLM Provider 集成 | 1.5天 | Phase 1 + 2 |
| **Phase 4** | 测试与文档 | 1.5天 | Phase 1-3 |
| **总计** | | **7天** | 无 |

---

## 验收标准

### 功能验收

- [ ] `http_get_stream()` 可以发起流式 HTTP 请求
- [ ] `http_stream_next()` 可以逐个读取 SSE 事件
- [ ] `parse_sse()` 可以正确解析 SSE 格式
- [ ] 支持 Anthropic 流式 API
- [ ] 支持OpenAI 流式 API
- [ ] 错误处理正确（连接断开、解析错误等）

### 测试验收

```bash
# 单元测试
cargo test -p auto-lang -- sse
cargo test -p auto-shell -- llm_stream

# 集成测试（需要 API key）
ANTHROPIC_API_KEY=sk-ant-xxx cargo test --test llm_stream_anthropic

# 本地测试
# 启动测试 SSE 服务器
cargo run --bin test_sse_server
# 在另一个终端运行测试
cargo test -- test_sse_integration
```

---

## 参考资料

- [MDN - Server-Sent Events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events)
- [HTTP/2 流式响应](https://httpwg.github.io/specs/rfc7540.html)
- [Anthropic Streaming API](https://docs.anthropic.com/en/api/messages-streaming)
- [OpenAI Streaming API](https://platform.openai.com/docs/api-reference/chat/create#chat/create-stream)
- [tokio-util StreamReader](https://docs.rs/tokio-util/latest/tokio_util/io/struct.StreamReader.html)

---

## 实施总结

**完成日期**: 2026-04-01

### 已实现功能

✅ **Phase 1: HTTPStream 类型**
- 添加了 `tokio-util`, `futures`, `reqwest` 依赖
- 创建了 `HTTP_STREAMS` 线程本地存储
- 实现了 `HttpStreamData` 结构体
- 添加了流式 HTTP opcode（2240-2244）
- 实现了 5 个流式 HTTP FFI 函数

✅ **Phase 2: SSE 解析器**
- 创建了完整的 SSE 解析模块 (`crates/auto-lang/src/sse/`)
- 实现了 `SSEEvent` 类型和相关方法
- 实现了 `parse_sse_chunk()` 函数
- 实现了 `SSEParser<R>` 流式解析器
- 10 个单元测试全部通过

✅ **Auto 标准库**
- `stdlib/auto/http_stream.at` - 流式 HTTP 客户端接口
- `stdlib/auto/sse.at` - SSE 解析器接口
- `stdlib/auto/llm.at` - LLM API 基础类型定义

### 创建的文件

```
crates/auto-lang/src/sse/
├── mod.rs              # 模块导出
├── parser.rs           # SSE 解析器 (~280 lines)
└── types.rs            # SSE 类型定义 (~70 lines)

stdlib/auto/
├── http_stream.at      # 流式 HTTP 客户端 (~150 lines)
├── sse.at              # SSE 解析器 (~130 lines)
└── llm.at              # LLM API 基础 (~120 lines)
```

### 修改的文件

```
crates/auto-lang/
├── Cargo.toml          # 添加依赖
├── src/lib.rs          # 导出 sse 模块
└── src/vm/ffi/stdlib.rs # 添加流式 HTTP FFI (~150 lines 新增)
```

### 测试结果

```bash
cargo test -p auto-lang sse
# 所有 10 个 SSE 测试通过 ✅

cargo build -p auto-lang
# 编译成功 ✅
```

### 后续工作

为 Plan 153 (AutoShell AI Agent) 提供了基础支持：
- ✅ 流式 HTTP 基础设施
- ✅ SSE 解析能力
- ✅ LLM 消息类型定义
- ⏸️ 完整的 LLM Provider 实现（Plan 153 Phase 1）
- ⏸️ 工具调用系统（Plan 153 Phase 3）

### 问题修复

**Tokio Runtime Panic** (已修复 ✅)

在实施过程中发现的 tokio 1.49 nested runtime panic 已在同一次提交中修复：

- **问题**：`VmInterpreter` 包含自己的 tokio runtime，导致嵌套 runtime
- **修复**：移除 VmInterpreter 中的 runtime，改用全局 runtime
- **提交**：`fix(runtime): resolve tokio 1.49 nested runtime panic`

