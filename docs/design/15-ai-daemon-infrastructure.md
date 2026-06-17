# 15 - AI Daemon 基础设施（AutoOS 共享 LLM Harness）

## Status

**Designed** — 架构设计完成,待实施(Plan 325)。

## 关系

- **上位**: [00-intro.md](00-intro.md) AutoOS 设计文档体系;本文是第 15 章。
- **依赖**: [13-networking.md](13-networking.md) HTTP Server 能力;[08-ui-systems.md](08-ui-systems.md) AutoUI App 模型。
- **被引用**: Plan 291 Phase 3(AI 集成)、Plan 322(Ash AI 模式 F3 stub)。
- **实施计划**: [Plan 325](../plans/325-ai-daemon-infrastructure.md)。

---

## 1. 问题与定位

### 1.1 场景

AutoOS 中所有应用(Ash、AutoForge、UI 编辑器、未来的所有 AutoUI App)都需要 LLM 能力:
- Ash: F3 AI 模式(自然语言→命令)。
- AutoForge: Agent 模式(代码生成、Spec 驱动)。
- UI 编辑器: AI 辅助布局/样式。
- 未来: 每个应用都可以是 Agent。

### 1.2 约束

LLM API 的并发限制是**全局的**——例如智谱 AI 给 4 个并发,不是每个 App 4 个,而是所有 App **加起来** 4 个。如果每个 App 独立管理并发,会导致超限、被拒、或浪费配额。

### 1.3 定位

AI Daemon(`aillmd`)是 AutoOS 的**共享基础设施**,类比:
- Wayland compositor 管理稀缺的 GPU 帧缓冲 → aillmd 管理稀缺的 LLM 并发槽。
- NetworkManager 管理网络连接 → aillmd 管理 API 连接池。
- ssh-agent 管理密钥 → aillmd 管理 API Key vault。

所有 AutoOS 应用通过统一的 `auto-llm-client` crate 接入,不直接持有密钥或管理并发。

---

## 2. 架构总览

```
┌──────────────────────────────────────────────────────────────┐
│                    aillmd (daemon)                           │
│                                                              │
│  HTTP over Unix socket / Named pipe                         │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ POST /v1/chat/completions  → 并发仲裁 → 上游 LLM API    │ │
│  │ GET  /v1/status            → 并发池 / 队列状态          │ │
│  │ POST /v1/admin/switch      → 切换模型                   │ │
│  │ GET  /v1/usage             → 成本 / token 用量          │ │
│  │ SSE  streaming response    → 流式补全                   │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Concurrency  │  │ Model Router │  │ Request Queue    │  │
│  │ Pools        │  │ (fallback,   │  │ (priority:       │  │
│  │ (per model)  │  │  cost-aware) │  │  Interactive >   │  │
│  │ Semaphore(N)│  │              │  │  Background)     │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Key Vault    │  │ Cost Tracker │  │ MCP Management   │  │
│  │ (per provider)│  │ (per App)    │  │ Interface        │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└───────┬──────────────────┬──────────────────┬───────────────┘
        │                  │                  │
   ┌────┴─────┐       ┌────┴─────┐       ┌────┴─────┐
   │   Ash    │       │  Forge   │       │ UI Edit  │  ... 未来所有 App
   │ (F3 AI)  │       │ (Agent)  │       │ (Agent)  │
   └──────────┘       └──────────┘       └──────────┘
    auto-llm-client (共享 Rust crate,所有 App 链接它)
```

---

## 3. 通信协议

### 3.1 传输层:HTTP over Unix socket

**选择理由**:

| 考量 | HTTP over Unix socket | 裸 TCP HTTP | 自定义 IPC |
|---|---|---|---|
| 与上游 LLM API 一致 | ✅ 都是 HTTP | ✅ | ❌ |
| 流式补全(SSE) | ✅ HTTP 原生 | ✅ | ❌ 需自建 |
| curl 调试 | ✅ `--unix-socket` | ✅ | ❌ |
| 端口冲突 | ✅ 无(socket 文件) | ❌ | ✅ |
| 安全 | ✅ 文件权限 | ⚠️ localhost 裸奔 | ✅ |
| 延迟 | ✅ 低(无 TCP 栈) | ⚠️ TCP 握手 | ✅ 最低 |
| 未来跨机器 | ⬜ 切 TCP 即可 | ✅ | ⬜ |

**Docker 模型**:`docker.sock` 上跑 HTTP 协议。我们的 `aillmd.sock` 同理。

- **Unix**: `/run/aillmd.sock` 或 `~/.config/autoos/aillmd.sock`
- **Windows**: `\\.\pipe\aillmd`
- `auto-llm-client` crate 封装平台差异,App 无感知。

### 3.2 API 端点

| 方法 | 路径 | 用途 | 流式 |
|---|---|---|---|
| POST | `/v1/chat/completions` | 聊天补全(OpenAI 兼容格式) | SSE |
| POST | `/v1/embeddings` | 向量嵌入 | 否 |
| GET | `/v1/status` | 并发池状态、队列长度 | 否 |
| GET | `/v1/models` | 可用模型列表 | 否 |
| POST | `/v1/admin/switch` | 切换主模型 | 否 |
| POST | `/v1/admin/keys/:provider` | 添加 API Key | 否 |
| GET | `/v1/usage` | Token 用量统计(per App) | 否 |

**设计原则**: `/v1/chat/completions` 兼容 OpenAI API 格式。App 切换「直连 LLM API」和「走 daemon」只需改 base URL。

### 3.3 请求头

```
POST /v1/chat/completions HTTP/1.1
Host: localhost
Content-Type: application/json
X-App-Name: ash          # 哪个 App 发的(用于成本追踪)
X-Priority: interactive   # interactive | background
X-Stream: true            # 是否流式

{"model":"glm-4.5","messages":[...],"stream":true}
```

### 3.4 流式响应(SSE)

```
HTTP/1.1 200 OK
Content-Type: text/event-stream

data: {"choices":[{"delta":{"content":"Hello"}}]}

data: {"choices":[{"delta":{"content":" world"}}]}

data: [DONE]
```

与上游 LLM API 的 SSE 格式完全一致——daemon 透传,只加并发管理。

---

## 4. 并发仲裁

### 4.1 全局并发池

每个 model provider 一个 `Semaphore`:

```rust
struct ConcurrencyPool {
    pools: HashMap<Provider, Arc<Semaphore>>,
    // zhipu: Semaphore(4), openai: Semaphore(10), ...
}
```

请求到达时:
1. 从 pool 获取 permit(异步等待,不阻塞)。
2. 发请求到上游 LLM API。
3. 响应完成(或流结束)→ 释放 permit。

### 4.2 优先级队列

当并发满时,请求排队。交互式(Ash)优先于后台(Forge 批处理):

```rust
enum Priority {
    Interactive,  // Ash F3、UI 辅助 — 用户在等
    Background,   // Forge 后台生成 — 可以慢
}
```

队列用 `priority_queue`:Interactive 请求插队,Background 按 FIFO。

### 4.3 模型路由(fallback)

```
请求 glm-4.5 → 并发满(4/4 occupied)
  → 等待? (priority high → 等 < 2s)
  → 超时 → 自动切 fallback model (openai/gpt-4o)
  → 或返回 503 (由 App 决定是否重试)
```

策略可配置:

```auto
// ~/.config/autoos/ai-daemon.at
models {
    primary {
        provider : zhipu
        model : glm-4.5
        max_concurrency : 4
    }
    fallback {
        provider : openai
        model : gpt-4o
        trigger : timeout   // timeout | error | always
        timeout_ms : 2000
    }
    on_overflow : wait_or_fallback   // wait | fallback | error
}
```

---

## 5. Key Vault

### 5.1 密钥集中管理

所有 API Key 存在 daemon 的配置文件中,App **永远不接触密钥**:

```auto
// ~/.config/autoos/ai-daemon.at
keys {
    zhipu : "xxxx-xxxx-xxxx"
    openai : "sk-xxxx"
}
```

App 发请求时不需要 Authorization header——daemon 注入。

### 5.2 安全

- 配置文件权限 `0600`(仅 owner 可读)。
- 未来:用 OS keyring(macOS Keychain / Windows Credential Manager)存储。
- App 与 daemon 之间通过 Unix socket 文件权限隔离(只有同用户可连)。

---

## 6. Cost Tracker

### 6.1 Per-App 用量

每次请求记录:
```json
{
    "app": "ash",
    "model": "glm-4.5",
    "prompt_tokens": 150,
    "completion_tokens": 80,
    "cost_cny": 0.002,
    "timestamp": "2026-06-17T12:00:00Z"
}
```

### 6.2 查询

```
GET /v1/usage?app=ash&since=2026-06-01
→ { "total_tokens": 45000, "total_cost_cny": 1.23, "requests": 120 }
```

### 6.3 预算限制(可选)

```auto
budgets {
    ash { daily_tokens : 50000 }
    forge { daily_tokens : 500000 }
}
```

超预算 → daemon 返回 429 → App 可以提示用户或切模型。

---

## 7. MCP 管理接口

Daemon 自身也是一个 **MCP server**,暴露管理工具供其他 Agent 查询/操作:

| MCP Tool | 用途 |
|---|---|
| `llm_status` | 查并发池状态(哪些满、哪些有空) |
| `llm_models` | 列可用模型 |
| `llm_switch` | 切换主模型 |
| `llm_usage` | 查用量 |
| `llm_queue` | 查队列状态 |

**注意**: MCP 用于**管理**(query/control),LLM 补全走**专用 HTTP 端点**(效率更高)。这与 MCP 的定位一致——MCP 是工具暴露,不是数据通道。

---

## 8. Daemon 生命周期

### 8.1 Lazy Start(ssh-agent 模型)

```
1. App 启动 → auto-llm-client 尝试连 aillmd.sock
2. 连不上 → client fork+exec: aillmd
3. Daemon 启动 → 读配置 → 绑定 socket
4. Client 重连成功
5. Daemon 持续运行(空闲 10 分钟后自动退出)
```

### 8.2 未来:Socket Activation

Phase 2(AutoOS 成熟后):
- OS 在 boot 时创建 socket 文件(不启动 daemon)。
- 第一个连接到来 → OS 自动拉起 daemon(systemd socket activation)。
- 无竞态,无「daemon not ready」窗口。

### 8.3 配置热加载

Daemon 监听配置文件变化(inotify / ReadDirectoryChanges):
- 修改 `ai-daemon.at` → 自动重新加载模型/密钥/预算配置。
- 不需要重启 daemon。

---

## 9. `auto-llm-client` Crate

所有 App 链接的客户端 crate。

### 9.1 API

```rust
pub struct LlmClient {
    // 自动发现 daemon 或降级为直连
}

impl LlmClient {
    /// 创建客户端(自动发现 daemon)。
    pub fn new() -> Self;

    /// 发送补全请求(阻塞)。
    pub fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse>;

    /// 发送流式补全(返回迭代器)。
    pub fn complete_stream(&self, req: CompletionRequest)
        -> Result<Box<dyn Iterator<Item = Result<Delta>>>>;

    /// 查询 daemon 状态。
    pub fn status(&self) -> Result<DaemonStatus>;
}

pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub priority: Priority,
    pub stream: bool,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f64>,
}
```

### 9.2 降级模式

```rust
impl LlmClient {
    pub fn new() -> Self {
        if let Some(socket) = discover_daemon() {
            Self::Daemon(socket)  // 优先走 daemon
        } else {
            Self::Direct {        // 降级:直连 LLM API
                keys: load_keys_from_env_or_config(),
                concurrency: Semaphore::new(1),  // 保守并发
            }
        }
    }
}
```

App 代码不变——无论 daemon 是否在运行,`client.complete()` 都能用。

---

## 10. 配置文件 Schema

```auto
// ~/.config/autoos/ai-daemon.at

daemon {
    socket : "~/.config/autoos/aillmd.sock"
    idle_timeout_min : 10
    log_level : info
}

models {
    primary {
        provider : zhipu
        model : glm-4.5
        base_url : "https://open.bigmodel.cn/api/paas/v4"
        max_concurrency : 4
    }
    fallback {
        provider : openai
        model : gpt-4o
        base_url : "https://api.openai.com/v1"
        trigger : timeout
        timeout_ms : 2000
    }
    on_overflow : wait_or_fallback
}

keys {
    zhipu : "xxxx"
    openai : "sk-xxxx"
}

budgets {
    ash { daily_tokens : 50000 }
    forge { daily_tokens : 500000 }
}
```

---

## 11. 迁移路径

### Phase 1: 从 AutoForge 提取 → `auto-llm-client` crate

- 从 AutoForge 内部提取 LLM harness 逻辑 → 独立 crate `auto-llm-client`。
- AutoForge 改用此 crate(行为不变,只是代码搬位置)。
- 此时还没有 daemon,client 直接调 API。

### Phase 2: 新增 `aillmd` daemon

- 实现 daemon 二进制(HTTP over Unix socket)。
- `auto-llm-client` 优先连 daemon,降级为直连。
- Ash F3 AI 模式接入 client → 替换 stub。

### Phase 3: MCP 管理接口 + 配置 App

- Daemon 暴露 MCP server(llm_status / llm_switch / ...)。
- AutoUI 配置 App(可视化并发池 / 成本 / 模型切换)。

### Phase 4: AutoOS 集成

- Socket activation(boot 时自动创建 socket)。
- 所有 AutoUI App 默认依赖 `auto-llm-client`。
- 成本预算强制执行。

---

## 12. CLI 工具 (`aillmctl`)

```bash
aillmctl status              # 并发池状态
aillmctl models              # 可用模型
aillmctl switch glm-4.5      # 切主模型
aillmctl keys add zhipu xxx  # 加密钥
aillmctl usage --app ash     # 查用量
aillmctl queue               # 查等待队列
```

等价于 `systemctl` / `docker` / `redis-cli` —— daemon 的管理面板。

---

## 13. 与 MCP 生态的关系

```
┌─────────────────────────────────────────────┐
│              AutoOS App Space               │
│                                             │
│  ┌──────┐  ┌──────┐  ┌──────┐              │
│  │ Ash  │  │Forge │  │ UI   │              │
│  └──┬───┘  └──┬───┘  └──┬───┘              │
│     │         │         │                   │
│     │  MCP    │  MCP    │  MCP   (互查状态)  │
│     │  Server │  Server │  Server           │
│     └────┬────┴────┬────┘                   │
│          │         │                        │
│     ┌────┴─────────┴────┐                   │
│     │  aillmd            │                   │
│     │  (LLM 补全: HTTP)  │                   │
│     │  (管理接口: MCP)   │                   │
│     └───────────────────┘                   │
└─────────────────────────────────────────────┘
```

- **App ↔ Daemon LLM 补全**: 专用 HTTP(效率)。
- **App ↔ App 互操作**: MCP(查状态、调用工具)。
- **App → Daemon 管理**: MCP(llm_status 等)。
- **Daemon → LLM API**: HTTP+SSE(上游)。

三条通道各司其职,不混用。
