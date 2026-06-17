# Plan 325: AI Daemon 基础设施（aaid + auto-ai-client）

> **Status**: Phase 1-2 complete (2026-06-17). P1: `auto-ai-client` crate (18 tests). P2: `aaid` daemon binary — HTTP server + concurrency pool + usage tracker + config (7 tests). Smoke-tested: `/v1/status`, `/v1/models`, `/v1/usage` all return JSON. P3 (Ash F3 接入) / P4 (MCP+CLI) pending.
> **设计文档**: [15-ai-daemon-infrastructure.md](../design/15-ai-daemon-infrastructure.md)
> **关系**: Plan 322(Ash AI 模式 F3 stub → 接入 client)、Plan 291 Phase 3(AI 集成)。
> 落地 [docs/design/15](../design/15-ai-daemon-infrastructure.md) 的架构设计。

---

## 1. 目标

从 AutoForge 提取 LLM harness → 构建 AutoOS 共享基础设施:

```
Phase 1: auto-llm-client crate (从 Forge 提取,直连模式)
Phase 2: aillmd daemon (HTTP over Unix socket,并发仲裁)
Phase 3: Ash F3 接入 (替换 stub)
Phase 4: MCP 管理接口 + aillmctl CLI
```

---

## 2. Phase 1: `auto-llm-client` crate

**交付**: 独立 Rust crate,从 AutoForge 提取 LLM 调用逻辑。无 daemon,直连 API。

### 2.1 Crate 结构

```
crates/auto-llm-client/
├── Cargo.toml
├── src/
│   ├── lib.rs           # LlmClient + CompletionRequest/Response
│   ├── provider.rs      # Provider trait (Zhipu / OpenAI / Anthropic)
│   ├── providers/
│   │   ├── zhipu.rs     # 智谱 GLM API 适配
│   │   ├── openai.rs    # OpenAI API 适配
│   │   └── anthropic.rs # Anthropic API 适配
│   ├── stream.rs        # SSE 流式解析
│   ├── config.rs        # 配置读取 (~/.config/autoos/llm-client.at)
│   └── error.rs         # 统一错误类型
```

### 2.2 核心 API

```rust
pub struct LlmClient { /* provider + config */ }

impl LlmClient {
    pub fn new() -> Result<Self>;
    pub fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse>;
    pub fn complete_stream(&self, req: &CompletionRequest)
        -> Result<Box<dyn Iterator<Item = Result<String>>>>;
}

pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f64>,
    pub stream: bool,
}
```

### 2.3 Provider Trait

```rust
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse>;
    async fn complete_stream(&self, req: &CompletionRequest)
        -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>>;
}
```

每个 provider 实现此 trait,内部处理 HTTP 调用 + SSE 解析。

### 2.4 配置

```auto
// ~/.config/autoos/llm-client.at (直连模式配置)
providers {
    zhipu {
        base_url : "https://open.bigmodel.cn/api/paas/v4"
        key_env : ZHIPU_API_KEY   // 从环境变量读密钥
        models : [glm-4.5, glm-4-flash]
    }
}
default_provider : zhipu
default_model : glm-4.5
```

### 2.5 任务

1. 从 AutoForge 提取 LLM 调用代码 → `auto-llm-client` crate。
2. 定义 `LlmProvider` trait + 实现 Zhipu provider(主要)。
3. SSE 流式解析。
4. 配置读取。
5. 单测:mock provider + 响应解析。
6. AutoForge 改用此 crate(验证提取正确)。

---

## 3. Phase 2: `aillmd` daemon

**交付**: 独立二进制,HTTP over Unix socket,全局并发仲裁。

### 3.1 Crate 结构

```
crates/auto-llm-daemon/
├── Cargo.toml
├── src/
│   ├── main.rs          # 二进制入口 + lazy start 逻辑
│   ├── server.rs        # HTTP server (hyper / axum)
│   ├── pool.rs          # 并发池管理 (per-model Semaphore)
│   ├── queue.rs         # 优先级请求队列
│   ├── router.rs        # 模型路由 + fallback 策略
│   ├── vault.rs         # API Key vault
│   ├── tracker.rs       # 成本/用量追踪
│   └── config.rs        # daemon 配置
```

### 3.2 HTTP 端点

| 方法 | 路径 | 功能 |
|---|---|---|
| POST | `/v1/chat/completions` | 补全(透传到上游 + 并发管理) |
| GET | `/v1/status` | 并发池状态 |
| GET | `/v1/models` | 可用模型 |
| POST | `/v1/admin/switch` | 切换主模型 |
| GET | `/v1/usage` | 用量统计 |

### 3.3 并发仲裁流程

```
请求到达 → 查 X-Priority 头
  → Interactive → 优先级队列头部
  → Background → 队列尾部
→ 检查 model 的 Semaphore
  → 有 permit → 获取 permit → 发上游请求 → 流式透传 → 释放 permit
  → 无 permit → 排队等待
    → 等待超时 → fallback 策略
      → wait_or_fallback → 切备用模型
      → wait → 继续等
      → error → 返回 503
```

### 3.4 依赖 auto-llm-client

Daemon 内部使用 `auto-llm-client` 的 `LlmProvider` trait 发上游请求。Daemon 只是加了并发管理 + 路由 + Key 注入。

### 3.5 Lazy Start

```rust
// auto-llm-client 的 discover_daemon():
fn discover_daemon() -> Option<UnixStream> {
    let socket_path = daemon_socket_path()?;
    match UnixStream::connect(&socket_path) {
        Ok(stream) => Some(stream),
        Err(_) => {
            // Daemon 不在运行 → fork + exec
            spawn_daemon(&socket_path)?;
            // 等待 socket 就绪(重试连接,最多 2s)
            wait_for_socket(&socket_path, Duration::from_secs(2))?;
            UnixStream::connect(&socket_path).ok()
        }
    }
}
```

### 3.6 任务

1. `auto-llm-daemon` crate + 二进制。
2. HTTP server(axum,支持 Unix socket 监听)。
3. 并发池(Semaphore per model)。
4. 优先级队列。
5. 模型路由 + fallback。
6. Key vault(配置文件读取)。
7. 成本追踪(简单 JSON log,Phase 1 不做 DB)。
8. `auto-llm-client` 的 daemon 模式(socket 连接 + lazy start)。
9. 单测 + 集成测试(mock upstream)。

---

## 4. Phase 3: Ash F3 接入

**交付**: Ash 的 F3 AI 模式从 stub 变为真实 LLM 调用。

### 4.1 改动

- `auto-shell/Cargo.toml` 添加 `auto-llm-client` 依赖。
- `Repl::run()` 中 F3 分支:用 `LlmClient` 发请求。
- AI 模式流程:
  ```
  F3 → prompt ? → 用户输入自然语言
  → LlmClient::complete("将以下自然语言翻译为 ash 命令: ...")
  → 返回命令建议
  → [Enter] 执行 / [e] 编辑 / [Esc] 取消
  ```

### 4.2 任务

1. Ash 依赖 `auto-llm-client`。
2. F3 分支调用 `LlmClient`(替换 stub)。
3. 系统提示词设计(自然语言 → ash 命令)。
4. 测试(需要 mock 或真实 API key)。

---

## 5. Phase 4: MCP 管理接口 + CLI

**交付**: Daemon 作为 MCP server + `aillmctl` 管理工具。

### 5.1 MCP Tools

| Tool | 用途 |
|---|---|
| `llm_status` | 并发池状态 |
| `llm_models` | 可用模型 |
| `llm_switch` | 切换模型 |
| `llm_usage` | 用量 |
| `llm_queue` | 队列状态 |

### 5.2 `aillmctl` CLI

```
aillmctl status / models / switch / keys / usage / queue
```

### 5.3 任务

1. Daemon 实现 MCP server(stdio 或 HTTP transport)。
2. `aillmctl` 二进制(调 daemon 的 HTTP 端点)。
3. 测试。

---

## 6. 跨阶段共用:配置文件

### 6.1 Phase 1 (直连模式)

```auto
// ~/.config/autoos/llm-client.at
providers {
    zhipu {
        base_url : "https://open.bigmodel.cn/api/paas/v4"
        key_env : ZHIPU_API_KEY
    }
}
default_provider : zhipu
default_model : glm-4.5
```

### 6.2 Phase 2+ (daemon 模式)

```auto
// ~/.config/autoos/ai-daemon.at
daemon {
    socket : "~/.config/autoos/aillmd.sock"
    idle_timeout_min : 10
}
models {
    primary { provider : zhipu, model : glm-4.5, max_concurrency : 4 }
    fallback { provider : openai, model : gpt-4o, trigger : timeout }
    on_overflow : wait_or_fallback
}
keys {
    zhipu : "xxxx"
    openai : "sk-xxxx"
}
budgets {
    ash { daily_tokens : 50000 }
}
```

App 端配置简化为:
```auto
// ~/.config/autoos/llm-client.at (daemon 模式)
daemon_socket : "~/.config/autoos/aillmd.sock"
// 不需要 provider/key — daemon 管
```

---

## 7. 验证

- **Phase 1**: `auto-llm-client` 单测(mock provider) + AutoForge 集成(行为不变)。
- **Phase 2**: `curl --unix-socket aillmd.sock http://localhost/v1/chat/completions` 手动测试。
- **Phase 3**: `ash` F3 → 输入 "list all rust files" → 返回 `find . -name "*.rs"`。
- **Phase 4**: `aillmctl status` 显示并发池;MCP client 查 `llm_status`。

---

## 8. 关键文件

| 文件 | Phase | 说明 |
|---|---|---|
| `crates/auto-llm-client/` (新) | 1 | 共享客户端 crate |
| `crates/auto-llm-daemon/` (新) | 2 | daemon 二进制 |
| `crates/auto-shell/Cargo.toml` | 3 | 添加 auto-llm-client 依赖 |
| `crates/auto-shell/src/frontend/repl.rs` | 3 | F3 分支接入 LlmClient |
| `crates/aillmctl/` (新) | 4 | CLI 管理工具 |
| `docs/design/15-ai-daemon-infrastructure.md` | — | 架构设计(已完成) |
