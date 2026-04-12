# Plan 159: AutoCode — Coding Agent 设计与实现

**日期**: 2026-04-08
**状态**: 设计完成，待实现
**目标**: 用 Rust 构建一个类 Claude Code 的 Coding Agent 原型，后续移植到 AutoLang

---

## 1. 目标与范围

### 目标
构建一个名为 **AutoCode** 的 Coding Agent，能够：
1. 与 LLM API（Claude / OpenAI / OpenAI 兼容）进行流式通信
2. 管理对话上下文（消息历史、上下文压缩）
3. 执行基本工具（Bash、文件读/写/编辑、Grep 搜索）
4. 在交互式 REPL 中完成 Prompt → 代码生成的完整循环

### 范围
- **第一版**: Rust 原型，独立仓库，功能完整的 MVP
- **后续**: 移植到 AutoLang，作为 auto-lang workspace 中的新 crate

### 参考架构
- **Claude Code** (`D:\github\claude-code`): TypeScript 实现，核心参考
- **claw-code** (`D:\github\claw-code`): Rust 实现，直接参考其 crate 结构和 API 设计

---

## 2. 架构设计

### 2.1 仓库结构

Rust 原型位于独立仓库 `auto-code-rs`：

```
D:\autostack\auto-code-rs\
├── Cargo.toml                    # Workspace 根
├── crates/
│   ├── ac-api/                   # LLM API 通信层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # 公共 API 导出
│   │       ├── types.rs          # Message, ContentBlock, ToolDefinition, StreamEvent
│   │       ├── anthropic.rs      # Claude Messages API (SSE streaming)
│   │       └── openai.rs         # OpenAI Chat Completions API (SSE streaming)
│   │
│   ├── ac-tools/                 # 工具定义与执行
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Tool trait + ToolRegistry
│   │       ├── bash.rs           # Bash 命令执行
│   │       ├── file_read.rs      # 文件读取
│   │       ├── file_write.rs     # 文件写入
│   │       ├── file_edit.rs      # 文件编辑（字符串替换）
│   │       └── search.rs         # Grep 搜索
│   │
│   ├── ac-runtime/               # Agent 运行时
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # 公共 API 导出
│   │       ├── agent.rs          # Agentic loop（核心循环）
│   │       ├── context.rs        # 消息历史、上下文压缩
│   │       ├── permission.rs     # 权限检查（Allow / Ask / ReadOnly）
│   │       └── session.rs        # 会话持久化（JSONL）
│   │
│   └── ac-cli/                   # CLI 入口
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs           # CLI 参数解析 + 入口
│           └── repl.rs           # 交互式 REPL 循环
```

AutoLang 移植版本（后续，在 auto-lang workspace 中）：

```
crates/
├── auto-code-api/      # LLM API 模块（.at 代码 + Rust FFI）
├── auto-code-tools/    # 工具模块
├── auto-code-core/     # Agent 运行时
└── auto-code-cli/      # CLI 入口
```

### 2.2 Crate 依赖关系

```
ac-cli → ac-runtime → ac-api
                    → ac-tools
```

- `ac-api`: 无内部依赖，依赖 `reqwest`, `tokio`, `serde_json`, `futures`
- `ac-tools`: 无内部依赖，依赖 `serde_json`, `regex`, `glob`
- `ac-runtime`: 依赖 `ac-api`, `ac-tools`
- `ac-cli`: 依赖 `ac-runtime`

---

## 3. 核心类型定义

### 3.1 API 类型（`ac-api/src/types.rs`）

```rust
use serde::{Deserialize, Serialize};

/// 消息角色
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// 内容块
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },

    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(default)]
        is_error: bool,
    },
}

/// 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

/// 工具定义（发送给 LLM 的 schema）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value, // JSON Schema
}

/// API 请求
#[derive(Debug, Clone)]
pub struct ApiRequest {
    pub model: String,
    pub system: Option<String>,
    pub messages: Vec<Message>,
    pub tools: Vec<ToolDefinition>,
    pub max_tokens: u32,
    pub stream: bool,
}

/// API 用量
#[derive(Debug, Clone, Default)]
pub struct ApiUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// 停止原因
#[derive(Debug, Clone, PartialEq)]
pub enum StopReason {
    EndTurn,      // 正常结束
    ToolUse,      // 需要执行工具
    MaxTokens,    // 达到 token 上限
    StopSequence, // 遇到停止序列
}

/// SSE 流事件
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// 增量文本输出
    TextDelta { text: String },
    /// 工具调用开始
    ToolUseBegin { id: String, name: String },
    /// 工具参数增量（JSON 字符串片段）
    ToolUseDelta { id: String, input_delta: String },
    /// 流结束
    Done { stop_reason: StopReason, usage: ApiUsage },
    /// 错误
    Error { message: String },
}

/// API 完整响应（非流式 fallback）
#[derive(Debug, Clone)]
pub struct ApiResponse {
    pub content: Vec<ContentBlock>,
    pub stop_reason: StopReason,
    pub usage: ApiUsage,
}
```

### 3.2 工具类型（`ac-tools/src/lib.rs`）

```rust
use serde_json::Value;

/// 工具错误
#[derive(Debug)]
pub struct ToolError(pub String);

/// 工具 trait
pub trait Tool: Send + Sync {
    /// 工具名称（与 API 中的 name 对应）
    fn name(&self) -> &str;

    /// 工具描述（发送给 LLM）
    fn description(&self) -> &str;

    /// 输入 JSON Schema
    fn input_schema(&self) -> Value;

    /// 执行工具
    fn execute(&self, input: Value) -> Result<String, ToolError>;

    /// 是否只读（不修改文件系统）
    fn is_read_only(&self) -> bool {
        false
    }
}

/// 工具注册表
pub struct ToolRegistry {
    tools: std::collections::HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self { /* ... */ }

    /// 注册工具
    pub fn register(&mut self, tool: Box<dyn Tool>) { /* ... */ }

    /// 获取工具
    pub fn get(&self, name: &str) -> Option<&dyn Tool> { /* ... */ }

    /// 获取所有工具定义（用于 API 请求）
    pub fn definitions(&self) -> Vec<ToolDefinition> { /* ... */ }

    /// 执行工具
    pub fn execute(&self, name: &str, input: Value) -> Result<String, ToolError> { /* ... */ }
}
```

### 3.3 权限类型（`ac-runtime/src/permission.rs`）

```rust
/// 权限模式
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionMode {
    Allow,      // 允许所有操作
    Ask,        // 每次询问用户
    ReadOnly,   // 只允许读操作
}

/// 权限决策
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionDecision {
    Allow,
    Deny { reason: String },
}

/// 权限检查
pub struct PermissionPolicy {
    mode: PermissionMode,
}

impl PermissionPolicy {
    /// 检查工具是否被允许执行
    pub fn check(&self, tool_name: &str, tool: &dyn Tool) -> PermissionDecision {
        match self.mode {
            PermissionMode::Allow => PermissionDecision::Allow,
            PermissionMode::ReadOnly => {
                if tool.is_read_only() {
                    PermissionDecision::Allow
                } else {
                    PermissionDecision::Deny {
                        reason: format!("{} blocked in read-only mode", tool_name),
                    }
                }
            }
            PermissionMode::Ask => PermissionDecision::Allow, // CLI 层负责实际询问
        }
    }
}
```

---

## 4. API 通信层设计

### 4.1 LLM Client Trait

```rust
use futures::Stream;
use std::pin::Pin;

/// LLM 客户端抽象
pub trait LlmClient: Send + Sync {
    /// 流式请求
    fn stream(
        &self,
        request: ApiRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = StreamEvent> + Send>>>;

    /// 非流式请求（fallback）
    fn complete(&self, request: ApiRequest) -> Result<ApiResponse>;
}
```

### 4.2 Anthropic 适配器

```rust
pub struct AnthropicClient {
    api_key: String,
    http: reqwest::Client,
    base_url: String, // 默认 "https://api.anthropic.com"
}

impl AnthropicClient {
    pub fn new(api_key: String) -> Self { /* ... */ }
}

impl LlmClient for AnthropicClient {
    fn stream(&self, request: ApiRequest) -> Result<Pin<Box<dyn Stream<Item = StreamEvent> + Send>>> {
        // 1. 构建请求体：
        //    POST /v1/messages
        //    Headers: x-api-key, anthropic-version: 2023-06-01, content-type: application/json
        //    Body: { model, messages, system, tools, max_tokens, stream: true }
        //
        // 2. 发送 HTTP 请求，获取 SSE 响应流
        //
        // 3. 解析 SSE 事件：
        //    - message_start → 初始化
        //    - content_block_start(index, type=text/tool_use) → TextDelta 或 ToolUseBegin
        //    - content_block_delta(index, delta) → 追加文本或工具参数
        //    - content_block_stop → 完成一个内容块
        //    - message_delta(stop_reason) → Done
    }

    fn complete(&self, request: ApiRequest) -> Result<ApiResponse> {
        // 同上但不启用 stream，直接返回完整响应
    }
}
```

**Anthropic 消息格式映射：**

```json
// User message
{ "role": "user", "content": [{ "type": "text", "text": "..." }] }

// Assistant message with tool use
{
  "role": "assistant",
  "content": [
    { "type": "text", "text": "让我查看文件" },
    { "type": "tool_use", "id": "toolu_xxx", "name": "Read", "input": { "file_path": "..." } }
  ]
}

// Tool result (作为 user message 的一部分)
{
  "role": "user",
  "content": [
    { "type": "tool_result", "tool_use_id": "toolu_xxx", "content": "文件内容..." }
  ]
}
```

### 4.3 OpenAI 适配器

```rust
pub struct OpenAiClient {
    api_key: String,
    http: reqwest::Client,
    base_url: String, // 默认 "https://api.openai.com/v1"
}

impl OpenAiClient {
    pub fn new(api_key: String) -> Self { /* ... */ }

    /// 设置自定义 base URL（用于 OpenAI 兼容 API）
    pub fn with_base_url(mut self, url: String) -> Self { /* ... */ }
}

impl LlmClient for OpenAiClient {
    fn stream(&self, request: ApiRequest) -> Result<Pin<Box<dyn Stream<Item = StreamEvent> + Send>>> {
        // 1. 构建请求体：
        //    POST /chat/completions
        //    Headers: Authorization: Bearer <key>, content-type: application/json
        //    Body: { model, messages, tools (function format), max_tokens, stream: true }
        //
        // 2. 解析 SSE：data: { choices: [{ delta: { content/tool_calls } }] }
        //    - data: [DONE] → Done
    }
}
```

**OpenAI 消息格式映射：**

```json
// User message
{ "role": "user", "content": "..." }

// Assistant message with tool call
{
  "role": "assistant",
  "content": "让我查看文件",
  "tool_calls": [{
    "id": "call_xxx",
    "type": "function",
    "function": { "name": "Read", "arguments": "{\"file_path\":\"...\"}" }
  }]
}

// Tool result
{ "role": "tool", "tool_call_id": "call_xxx", "content": "文件内容..." }
```

---

## 5. 工具实现细节

### 5.1 Bash 工具

```rust
pub struct BashTool;

impl Tool for BashTool {
    fn name(&self) -> &str { "Bash" }
    fn description(&self) -> &str { "Execute a bash command and return stdout/stderr" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "The bash command to run" },
                "timeout": { "type": "integer", "description": "Timeout in seconds (default 120)" }
            },
            "required": ["command"]
        })
    }
    fn execute(&self, input: Value) -> Result<String, ToolError> {
        let command = input["command"].as_str().ok_or(ToolError("missing command".into()))?;
        let timeout = input["timeout"].as_u64().unwrap_or(120);

        // 使用 std::process::Command
        // 1. 在 shell 中执行: cmd /C on Windows, bash -c on Unix
        // 2. 捕获 stdout + stderr
        // 3. 设置超时
        // 4. 返回合并的输出（或错误信息）
    }
    fn is_read_only(&self) -> bool { false }
}
```

### 5.2 Read 工具

```rust
pub struct ReadTool;

impl Tool for ReadTool {
    fn name(&self) -> &str { "Read" }
    fn description(&self) -> &str { "Read a file with line numbers" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string", "description": "Path to the file" },
                "offset": { "type": "integer", "description": "Starting line (0-based)" },
                "limit": { "type": "integer", "description": "Max lines to read (default 2000)" }
            },
            "required": ["file_path"]
        })
    }
    fn execute(&self, input: Value) -> Result<String, ToolError> {
        // 1. 读取文件内容
        // 2. 按行分割
        // 3. 应用 offset/limit
        // 4. 添加行号前缀 "  1 | ..."
        // 5. 返回格式化文本
    }
    fn is_read_only(&self) -> bool { true }
}
```

### 5.3 Write 工具

```rust
pub struct WriteTool;

impl Tool for WriteTool {
    fn name(&self) -> &str { "Write" }
    fn description(&self) -> &str { "Create or overwrite a file" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string", "description": "Path to write to" },
                "content": { "type": "string", "description": "File content" }
            },
            "required": ["file_path", "content"]
        })
    }
    fn execute(&self, input: Value) -> Result<String, ToolError> {
        // 1. 创建必要的父目录
        // 2. 写入内容
        // 3. 返回确认消息
    }
    fn is_read_only(&self) -> bool { false }
}
```

### 5.4 Edit 工具

```rust
pub struct EditTool;

impl Tool for EditTool {
    fn name(&self) -> &str { "Edit" }
    fn description(&self) -> &str { "Replace exact string in a file" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string", "description": "Path to the file" },
                "old_string": { "type": "string", "description": "Exact string to find (must be unique)" },
                "new_string": { "type": "string", "description": "Replacement string" }
            },
            "required": ["file_path", "old_string", "new_string"]
        })
    }
    fn execute(&self, input: Value) -> Result<String, ToolError> {
        // 1. 读取文件
        // 2. 查找 old_string（必须恰好出现一次）
        // 3. 替换为 new_string
        // 4. 写回文件
        // 5. 返回确认消息
    }
    fn is_read_only(&self) -> bool { false }
}
```

### 5.5 Grep 工具

```rust
pub struct GrepTool;

impl Tool for GrepTool {
    fn name(&self) -> &str { "Grep" }
    fn description(&self) -> &str { "Search file contents with regex pattern" }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "Regex pattern" },
                "path": { "type": "string", "description": "Directory or file to search (default .)" },
                "glob": { "type": "string", "description": "File glob filter (e.g. *.rs)" }
            },
            "required": ["pattern"]
        })
    }
    fn execute(&self, input: Value) -> Result<String, ToolError> {
        // 1. 使用 regex crate 编译 pattern
        // 2. 遍历目录（使用 walkdir 或 glob crate）
        // 3. 过滤文件类型（glob 参数）
        // 4. 逐文件搜索匹配行
        // 5. 格式化输出: "file.rs:42: matching line content"
        // 6. 限制输出行数（最多 250 行）
    }
    fn is_read_only(&self) -> bool { true }
}
```

---

## 6. Agent 运行时设计

### 6.1 Agentic Loop

```rust
pub struct Agent {
    messages: Vec<Message>,
    client: Box<dyn LlmClient>,
    tools: ToolRegistry,
    permission: PermissionPolicy,
    system_prompt: String,
}

impl Agent {
    pub fn new(client: Box<dyn LlmClient>, tools: ToolRegistry, permission: PermissionMode) -> Self { /* ... */ }

    /// 处理用户输入，运行 agentic loop
    pub async fn run_turn(&mut self, user_input: String) -> Result<TurnResult> {
        // 1. 追加 user message
        self.messages.push(Message {
            role: Role::User,
            content: vec![ContentBlock::Text { text: user_input }],
        });

        loop {
            // 2. 构建 API 请求
            let request = ApiRequest {
                model: self.model.clone(),
                system: Some(self.system_prompt.clone()),
                messages: self.messages.clone(),
                tools: self.tools.definitions(),
                max_tokens: 8192,
                stream: true,
            };

            // 3. 流式调用 LLM
            let mut stream = self.client.stream(request)?;
            let mut assistant_content: Vec<ContentBlock> = Vec::new();
            let mut current_tool_id = String::new();
            let mut current_tool_name = String::new();
            let mut current_tool_input = String::new();

            while let Some(event) = stream.next().await {
                match event {
                    StreamEvent::TextDelta { text } => {
                        // 实时打印到终端
                        print!("{}", text);
                        // 收集到当前文本块
                    }
                    StreamEvent::ToolUseBegin { id, name } => {
                        println!(); // 换行
                        current_tool_id = id;
                        current_tool_name = name;
                        current_tool_input.clear();
                    }
                    StreamEvent::ToolUseDelta { id, input_delta } => {
                        current_tool_input.push_str(&input_delta);
                    }
                    StreamEvent::Done { stop_reason, usage } => {
                        // 收集 assistant message
                        // 根据 stop_reason 决定是否继续
                        if stop_reason != StopReason::ToolUse {
                            // 没有工具调用，循环结束
                            return Ok(TurnResult { stop_reason, usage });
                        }
                    }
                    StreamEvent::Error { message } => {
                        return Err(AgentError::ApiError(message));
                    }
                }
            }

            // 4. 追加 assistant message
            self.messages.push(Message {
                role: Role::Assistant,
                content: assistant_content.clone(),
            });

            // 5. 执行工具调用
            for block in &assistant_content {
                if let ContentBlock::ToolUse { id, name, input } = block {
                    let result = match self.tools.get(name) {
                        Some(tool) => {
                            // 权限检查
                            match self.permission.check(name, tool) {
                                PermissionDecision::Allow => {
                                    match tool.execute(input.clone()) {
                                        Ok(output) => (output, false),
                                        Err(e) => (e.0, true),
                                    }
                                }
                                PermissionDecision::Deny { reason } => (reason, true),
                            }
                        }
                        None => (format!("Unknown tool: {}", name), true),
                    };

                    // 追加 tool result
                    self.messages.push(Message {
                        role: Role::User,
                        content: vec![ContentBlock::ToolResult {
                            tool_use_id: id.clone(),
                            content: result.0,
                            is_error: result.1,
                        }],
                    });
                }
            }

            // 回到循环顶部，再次调用 LLM
        }
    }
}
```

### 6.2 Context 压缩

```rust
pub struct ContextManager {
    max_tokens: u32,          // 上下文 token 上限（默认 100K）
    recent_turns: usize,      // 保留最近 N 轮（默认 10）
}

impl ContextManager {
    /// 估算当前消息的 token 数（简单估算：字符数 / 4）
    fn estimate_tokens(messages: &[Message]) -> u32 { /* ... */ }

    /// 如果接近上限，压缩消息
    fn maybe_compact(&self, messages: &mut Vec<Message>, client: &dyn LlmClient) -> Result<()> {
        let tokens = Self::estimate_tokens(messages);
        if tokens < self.max_tokens {
            return Ok(());
        }

        // 1. 保留 system message
        // 2. 保留最近 N 轮对话
        // 3. 对更早的消息，调用 LLM 生成摘要
        // 4. 用摘要替换旧消息
    }
}
```

### 6.3 会话持久化

```rust
pub struct Session {
    path: PathBuf,
    messages: Vec<Message>,
}

impl Session {
    /// 创建新会话
    pub fn new(workspace: &Path) -> Self {
        let hash = short_hash(workspace);
        let dir = home_dir().join(".autocode/sessions").join(&hash);
        create_dir_all(&dir).ok();
        let path = dir.join(format!("{}.jsonl", timestamp()));
        Self { path, messages: Vec::new() }
    }

    /// 追加消息（JSONL 格式，每行一条）
    pub fn append(&self, message: &Message) -> Result<()> {
        let line = serde_json::to_string(message)? + "\n";
        let mut file = OpenOptions::new().create(true).append(true).open(&self.path)?;
        file.write_all(line.as_bytes())?;
        Ok(())
    }

    /// 从文件恢复会话
    pub fn load(path: &Path) -> Result<Self> { /* ... */ }
}
```

---

## 7. CLI 设计

### 7.1 参数解析

```rust
/// CLI 参数
struct Cli {
    /// 单次 prompt（非交互模式）
    #[arg(short = 'p', long)]
    prompt: Option<String>,

    /// 模型名称
    #[arg(short, long)]
    model: Option<String>,

    /// API key（默认从环境变量读取）
    #[arg(long)]
    api_key: Option<String>,

    /// API 提供商: anthropic / openai
    #[arg(long, default_value = "anthropic")]
    provider: String,

    /// OpenAI 兼容 base URL
    #[arg(long)]
    base_url: Option<String>,

    /// 权限模式: allow / ask / read-only
    #[arg(long, default_value = "ask")]
    permission: String,

    /// 允许所有操作（等同于 --permission allow）
    #[arg(long)]
    allow: bool,

    /// 只读模式
    #[arg(long)]
    read_only: bool,
}
```

### 7.2 REPL 循环

```rust
fn run_repl(mut agent: Agent, session: Session) -> Result<()> {
    println!("AutoCode v0.1.0");
    println!("输入 prompt 开始对话，输入 :quit 退出\n");

    loop {
        // 1. 读取用户输入
        let input = readline("> ")?;
        let input = input.trim();

        if input.is_empty() { continue; }
        if input == ":quit" || input == ":q" { break; }
        if input == ":help" { /* 打印帮助 */ continue; }
        if input.starts_with(":") { /* 处理其他命令 */ continue; }

        // 2. 运行 agent turn
        match agent.run_turn(input.to_string()).await {
            Ok(result) => {
                println!(); // 换行
                session.append_all(agent.new_messages())?;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
```

---

## 8. System Prompt

AutoCode 使用以下 system prompt 指导 LLM 行为：

```
你是 AutoCode，一个编码助手 Agent。你可以通过工具帮助用户完成编程任务。

## 工具使用规则
- 使用 Read 工具查看文件内容，不要猜测
- 使用 Edit 工具进行精确修改，不要重写整个文件
- 使用 Bash 运行命令（构建、测试、git 操作等）
- 使用 Grep 搜索代码

## 工作流程
1. 先理解用户需求
2. 阅读相关文件了解现状
3. 制定修改方案
4. 执行修改
5. 运行测试验证

## 代码风格
- 遵循项目现有的代码风格
- 不添加不必要的注释或文档
- 保持修改最小化
```

---

## 9. 参考项目

本计划的实现以以下两个项目为参考蓝本：

### 9.1 Claude Code（TypeScript 原版）
- **仓库**: `D:\github\claude-code`
- **语言**: TypeScript，运行于 Bun
- **核心架构要点**:
  - `src/services/api/claude.ts` (~3400 行): API 通信核心，`queryModelWithStreaming()` 使用 AsyncGenerator 流式返回
  - `src/tools/` 目录: 每个工具一个文件夹，`Tool.ts` 定义 Tool interface（Zod schema + `call()` + `checkPermissions()`）
  - `src/QueryEngine.ts` (~1300 行): 会话管理，`mutableMessages` 数组跟踪对话
  - `src/query.ts` (~1700 行): Agentic loop，`while(true)` 循环中调用 API → 收集 tool_use → 执行工具 → 继续循环
  - `src/main.tsx` (~4600 行): CLI 入口，Commander.js 解析 50+ 选项
  - `src/types/message.ts`: 消息类型定义（User/Assistant/System/ToolResult）
- **设计模式**: AsyncGenerator 全栈、Zod 校验、React/Ink 终端 UI、多 provider 支持

### 9.2 claw-code（Rust 重写版）
- **仓库**: `D:\github\claw-code`
- **语言**: Rust，Tokio 异步运行时
- **核心架构要点**:
  - 9 个 crate 的 workspace: `api/`, `commands/`, `runtime/`, `tools/`, `telemetry/`, `plugins/`, `compat-harness/`, `mock-anthropic-service/`, `rusty-claude-cli/`
  - `crates/api/src/providers/anthropic.rs`: Anthropic SSE 流式实现，`reqwest` + 自定义 `SseParser`
  - `crates/runtime/src/conversation.rs`: Agentic loop，`run_turn()` 方法实现 prompt → API → tool → 循环
  - `crates/runtime/src/session.rs`: JSONL 会话持久化，`ConversationMessage` + `ContentBlock` 枚举
  - `crates/runtime/src/permissions.rs`: `PermissionPolicy` + rule-based 权限检查
  - `crates/tools/src/lib.rs`: `GlobalToolRegistry` + 40+ 工具实现
  - `crates/api/src/sse.rs`: SSE 解析器（处理 chunked 响应）
- **设计模式**: Trait 抽象（`ApiClient`, `ToolExecutor`, `PermissionPrompter`）、JSONL 增量写入、Provider enum 分发

### 9.3 参考对照关系

| AutoCode 组件 | Claude Code 参考 | claw-code 参考 | 优先参考 |
|--------------|-----------------|----------------|---------|
| API 通信 | `src/services/api/claude.ts` | `crates/api/src/providers/` | **claw-code**（同为 Rust） |
| SSE 解析 | SDK 内置 | `crates/api/src/sse.rs` | **claw-code** |
| 消息类型 | `src/types/message.ts` | `crates/api/src/types.rs` | **claw-code** |
| Agentic Loop | `src/query.ts` | `crates/runtime/src/conversation.rs` | **两者结合** |
| 工具系统 | `src/Tool.ts` + `src/tools/` | `crates/tools/src/lib.rs` | **claw-code**（trait 模式） |
| 权限系统 | `src/Tool.ts` checkPermissions | `crates/runtime/src/permissions.rs` | **claw-code** |
| 会话持久化 | JSON | `crates/runtime/src/session.rs` (JSONL) | **claw-code** |
| CLI 入口 | `src/main.tsx` | `crates/rusty-claude-cli/src/main.rs` | **claw-code** |
| Context 压缩 | `compact()` + `microCompact()` | `compact_session()` | **两者结合** |
| System Prompt | `src/services/api/` | `crates/runtime/src/prompt.rs` | **Claude Code**（更完整） |

---

## 10. 实现阶段

> **每个 Phase 开始前的工作流**:
>
> ```
> 1. 深入阅读两个参考项目中对应模块的源码
> 2. 对比分析两者的实现策略差异
> 3. 提炼出适合 AutoCode 的最佳实践
> 4. 更新本计划中该 Phase 的详细实施步骤
> 5. 开始编码实现
> ```

---

### Phase 1: API 通信层（`ac-api`）

> **开始前: 对比分析两个参考项目的 API 通信层**
>
> 需要深入阅读并对比：
> - Claude Code: `src/services/api/claude.ts` — 理解消息格式构建、流式处理、错误重试
> - claw-code: `crates/api/src/providers/anthropic.rs`, `crates/api/src/providers/openai_compat.rs`, `crates/api/src/sse.rs` — 理解 Rust 中的 SSE 解析、请求构建、provider 抽象
> - claw-code: `crates/api/src/types.rs` — 理解 Rust 类型定义方式
> - claw-code: `crates/api/src/client.rs` — 理解 provider enum 分发模式
> - Claude Code: `src/utils/model/providers.js` — 理解 provider 检测逻辑
>
> 对比重点：
> 1. SSE 解析：claw-code 用自定义 parser vs Claude Code 用 SDK 内置 — 我们的实现策略
> 2. 消息格式转换：Anthropic vs OpenAI 的消息结构差异，如何统一抽象
> 3. 流式输出处理：content_block_start/delta/stop 事件映射
> 4. 错误重试策略：两者的重试逻辑差异
> 5. Auth 管理：x-api-key vs Bearer token vs OAuth

**目标**: 能流式调用 Claude 和 OpenAI API，正确解析 SSE 响应

**交付物**:
- `ac-api` crate，包含 `LlmClient` trait 和两个实现
- 单元测试：mock SSE 数据解析
- 集成测试：真实 API 调用（需要 API key）

**详细步骤**:

| # | 步骤 | 产出文件 | 参考 |
|---|------|---------|------|
| 1.1 | 创建 `D:\autostack\auto-code-rs` 目录 | 目录结构 | claw-code workspace |
| 1.2 | 创建 workspace `Cargo.toml`，声明 members | `Cargo.toml` | claw-code `rust/Cargo.toml` |
| 1.3 | 创建 `crates/ac-api/` crate 骨架 | `crates/ac-api/Cargo.toml`, `src/lib.rs` | claw-code `crates/api/` |
| 1.4 | 定义核心类型 | `src/types.rs` — `Role`, `ContentBlock`, `Message`, `ToolDefinition`, `ApiRequest`, `ApiResponse`, `StreamEvent`, `StopReason`, `ApiUsage` | claw-code `types.rs` |
| 1.5 | 定义 `LlmClient` trait | `src/client.rs` — `stream()` + `complete()` 方法 | claw-code `ApiClient` trait |
| 1.6 | 实现 SSE 解析器 | `src/sse.rs` — 解析 `event:` / `data:` 行，处理 chunked 传输，返回 `Vec<SseEvent>` | claw-code `crates/api/src/sse.rs` |
| 1.7 | 实现 SSE 流适配器 | `src/sse_stream.rs` — 将 `reqwest` response body bytes 转为 `Stream<Item = StreamEvent>` | claw-code `providers/anthropic.rs` 的流处理 |
| 1.8 | 实现 `AnthropicClient` | `src/anthropic.rs` — 请求构建（messages → Anthropic JSON 格式）+ SSE 解析 + 错误处理 | claw-code `providers/anthropic.rs` + Claude Code `claude.ts` |
| 1.9 | 实现 `OpenAiClient` | `src/openai.rs` — 请求构建（messages → OpenAI JSON 格式）+ SSE 解析 + 可配置 base_url | claw-code `providers/openai_compat.rs` |
| 1.10 | 实现消息格式转换 | `src/message_format.rs` — `Message` → Anthropic format / OpenAI format 的双向转换 | 两个项目的消息格式差异 |
| 1.11 | SSE 解析单元测试 | `src/sse.rs` 测试 — 用硬编码的 SSE 数据验证解析正确性 | claw-code `sse.rs` tests |
| 1.12 | API 集成测试（可选，需 API key） | `tests/integration.rs` — 真实调用 Claude API，验证流式输出 | - |

**验证标准**:
- [ ] `cargo build -p ac-api` 编译通过
- [ ] SSE 解析器能正确处理 Anthropic 和 OpenAI 格式的 mock 数据
- [ ] `StreamEvent` 枚举正确映射两种 API 的所有事件类型
- [ ] （可选）真实 API 调用能流式返回文本

---

### Phase 2: 工具系统 + Bash（`ac-tools`）

> **开始前: 对比分析两个参考项目的工具系统**
>
> 需要深入阅读并对比：
> - Claude Code: `src/Tool.ts` — 理解 Tool interface 设计（Zod schema、call 签名、checkPermissions）
> - Claude Code: `src/tools/` — 具体工具实现（特别是 `BashTool/`, `FileTool/`）
> - Claude Code: `src/tools.ts` — 工具注册、过滤、组装逻辑
> - claw-code: `crates/tools/src/lib.rs` — 理解 `ToolSpec` 和 `GlobalToolRegistry`
> - claw-code: `crates/runtime/src/bash.rs` — Bash 工具的具体 Rust 实现
> - claw-code: `crates/runtime/src/file_ops.rs` — 文件操作的 Rust 实现
>
> 对比重点：
> 1. Tool trait 设计：claw-code 用 `ToolSpec` struct + match 分发 vs Claude Code 用 object interface — 哪种更适合我们
> 2. JSON Schema 生成：如何从 Rust 结构生成工具的 input_schema
> 3. Bash 安全验证：Claude Code 有 18 个子模块做 bash 命令安全检查 vs claw-code 的简化版 — MVP 用简化版
> 4. 错误处理：工具执行失败时如何返回给 LLM（is_error 标记）
> 5. 工具描述：如何编写有效的工具 description 让 LLM 正确使用

**目标**: Tool trait + ToolRegistry + Bash 工具

**交付物**:
- `ac-tools` crate
- `Tool` trait 和 `ToolRegistry`
- `BashTool` 实现（命令执行 + stdout/stderr 捕获）

**详细步骤**:

| # | 步骤 | 产出文件 | 参考 |
|---|------|---------|------|
| 2.1 | 创建 `crates/ac-tools/` crate | `Cargo.toml`, `src/lib.rs` | claw-code `crates/tools/` |
| 2.2 | 定义 `Tool` trait | `src/lib.rs` — `name()`, `description()`, `input_schema()`, `execute()`, `is_read_only()` | Claude Code `Tool.ts` + claw-code `ToolSpec` |
| 2.3 | 定义 `ToolError` 和 `ToolResult` | `src/lib.rs` — 错误类型 | claw-code `ToolError` |
| 2.4 | 实现 `ToolRegistry` | `src/registry.rs` — `register()`, `get()`, `definitions()`, `execute()` | claw-code `GlobalToolRegistry` |
| 2.5 | 实现 `BashTool` | `src/bash.rs` — 跨平台命令执行（Windows: `cmd /C`, Unix: `sh -c`），捕获 stdout + stderr，设置超时 | claw-code `runtime/src/bash.rs` + Claude Code `BashTool/` |
| 2.6 | Bash 工具安全基础 | `src/bash.rs` — 基本危险命令检测（`rm -rf /` 等） | Claude Code 18 个 bash 子模块（简化版） |
| 2.7 | 工具 JSON Schema 定义 | `src/bash.rs` 内嵌 schema — `{ command: string, timeout?: int }` | Claude Code Zod schema 模式 |
| 2.8 | 单元测试 | `src/bash.rs` tests — 测试 `echo hello`, `exit 1`, 超时等场景 | claw-code bash tests |

**验证标准**:
- [ ] `cargo build -p ac-tools` 编译通过
- [ ] `BashTool` 能执行命令并返回 stdout/stderr
- [ ] `ToolRegistry` 能注册和查找工具
- [ ] `definitions()` 返回正确的 JSON Schema 数组

---

### Phase 3: Agent 运行时 + 文件工具（`ac-runtime`）

> **开始前: 对比分析两个参考项目的 Agent 循环**
>
> 需要深入阅读并对比：
> - Claude Code: `src/query.ts` (~1700 行) — 理解 agentic loop 的完整实现：while(true) 循环、消息收集、工具执行、终止条件
> - Claude Code: `src/QueryEngine.ts` — 理解会话管理、消息追加、usage 跟踪
> - claw-code: `crates/runtime/src/conversation.rs` — 理解 Rust 中的 `run_turn()` 实现
> - claw-code: `crates/runtime/src/permissions.rs` — 理解 `PermissionPolicy` 和 rule-based 检查
> - Claude Code: `src/tools/FileReadTool/`, `src/tools/FileEditTool/`, `src/tools/FileWriteTool/` — 文件工具的实现细节
> - claw-code: `crates/runtime/src/file_ops.rs` — 文件操作的 Rust 实现
>
> 对比重点：
> 1. Agentic loop 结构：Claude Code 用 AsyncGenerator yield 事件 vs claw-code 用 `run_turn()` 返回 `TurnSummary` — 选择哪种
> 2. 工具调用收集：如何在流式响应中累积 tool_use 块（content_block_start/delta/stop 的状态机）
> 3. 文件编辑的精确匹配：`old_string` 必须唯一匹配的实现策略
> 4. 权限集成点：在 agentic loop 的哪个位置检查权限
> 5. Usage 跟踪：如何累计 token 用量

**目标**: Agentic loop + Read/Write/Edit 工具 + 权限系统

**交付物**:
- `ac-runtime` crate
- Agent struct 和 agentic loop
- ReadTool, WriteTool, EditTool（在 `ac-tools` 中）
- 权限系统

**详细步骤**:

| # | 步骤 | 产出文件 | 参考 |
|---|------|---------|------|
| 3.1 | 创建 `crates/ac-runtime/` crate | `Cargo.toml`, `src/lib.rs` | claw-code `crates/runtime/` |
| 3.2 | 实现 `ReadTool` | `ac-tools/src/file_read.rs` — 读取文件、按行号格式化、offset/limit 分页 | Claude Code `FileReadTool/` + claw-code `file_ops.rs` |
| 3.3 | 实现 `WriteTool` | `ac-tools/src/file_write.rs` — 写入文件、自动创建父目录 | Claude Code `FileWriteTool/` + claw-code `file_ops.rs` |
| 3.4 | 实现 `EditTool` | `ac-tools/src/file_edit.rs` — 精确字符串替换（old_string 唯一匹配校验） | Claude Code `FileEditTool/` |
| 3.5 | 定义 `PermissionMode` 和 `PermissionPolicy` | `src/permission.rs` — Allow/Ask/ReadOnly 三种模式 + `check()` 方法 | claw-code `permissions.rs` |
| 3.6 | 定义 `Agent` struct | `src/agent.rs` — 持有 messages, client, tools, permission, system_prompt | claw-code `ConversationRuntime` |
| 3.7 | 实现 `Agent::run_turn()` — 消息追加 | `src/agent.rs` — 将 user input 追加为 User message | claw-code `run_turn()` 开头部分 |
| 3.8 | 实现 `Agent::run_turn()` — API 调用 | `src/agent.rs` — 构建 ApiRequest，调用 `client.stream()`，处理 StreamEvent | claw-code `run_turn()` API 调用部分 |
| 3.9 | 实现 `Agent::run_turn()` — 工具执行 | `src/agent.rs` — 解析 tool_use 块，执行工具，追加 ToolResult | claw-code `run_turn()` 工具执行部分 |
| 3.10 | 实现 `Agent::run_turn()` — 循环控制 | `src/agent.rs` — stop_reason 判断（ToolUse → 继续，EndTurn → 退出） | Claude Code `query.ts` 的 while(true) 循环 |
| 3.11 | 实现流式文本输出回调 | `src/agent.rs` — TextDelta 事件实时打印到 stdout | Claude Code 的 yield message 模式 |
| 3.12 | 集成测试 — Mock LLM Client | `src/agent.rs` tests — 用 mock client 测试 agentic loop 的各种场景（有/无工具、多轮工具、错误恢复） | claw-code `mock-anthropic-service/` |

**验证标准**:
- [ ] `cargo build -p ac-runtime` 编译通过
- [ ] `ReadTool` 能读取文件并返回带行号的内容
- [ ] `WriteTool` 能创建/覆写文件
- [ ] `EditTool` 能精确替换字符串（包括唯一性校验）
- [ ] Mock 测试：agent 能完成一轮 "用户提问 → LLM 回答" 循环
- [ ] Mock 测试：agent 能完成 "用户提问 → LLM 调用工具 → 工具结果 → LLM 继续回答" 循环
- [ ] Mock 测试：权限拒绝时正确返回错误给 LLM

---

### Phase 4: CLI + REPL（`ac-cli`）

> **开始前: 对比分析两个参考项目的 CLI 和 REPL**
>
> 需要深入阅读并对比：
> - Claude Code: `src/entrypoints/cli.tsx` + `src/main.tsx` — CLI 参数解析、两种模式（REPL vs -p）
> - claw-code: `crates/rusty-claude-cli/src/main.rs` — Rust CLI 入口、`CliAction` enum 分发
> - claw-code: `crates/runtime/src/config.rs` — 配置管理
> - Claude Code: REPL 交互逻辑（在 `main.tsx` 中的交互循环）
>
> 对比重点：
> 1. CLI 参数设计：两者的参数差异，我们需要哪些（精简版）
> 2. Provider 选择逻辑：如何根据参数选择 Anthropic vs OpenAI
> 3. API Key 管理：环境变量 vs 命令行参数 vs 配置文件
> 4. REPL 体验：输入提示、Ctrl+C 中断、多行输入
> 5. 初始化流程：创建 client → 创建 agent → 进入循环

**目标**: 完整可用的命令行工具

**交付物**:
- `ac-cli` binary crate
- CLI 参数解析
- 交互式 REPL
- 环境变量配置

**详细步骤**:

| # | 步骤 | 产出文件 | 参考 |
|---|------|---------|------|
| 4.1 | 创建 `crates/ac-cli/` binary crate | `Cargo.toml`, `src/main.rs` | claw-code `rusty-claude-cli/` |
| 4.2 | CLI 参数解析 | `src/main.rs` — clap derive 模式定义 `Cli` struct（prompt, model, api_key, provider, base_url, permission, allow, read_only） | claw-code `parse_args()` |
| 4.3 | Provider 初始化逻辑 | `src/main.rs` — 根据 `--provider` 参数创建 `AnthropicClient` 或 `OpenAiClient`，从环境变量或参数获取 API key | claw-code `ProviderClient` enum |
| 4.4 | Agent 初始化 | `src/main.rs` — 创建 `ToolRegistry`、注册所有工具、创建 `PermissionPolicy`、创建 `Agent` | claw-code `ConversationRuntime::new()` |
| 4.5 | 实现 REPL 循环 | `src/repl.rs` — `run_repl()` 函数：打印欢迎信息 → 读取输入 → 处理命令（`:quit`, `:help`）→ 调用 `agent.run_turn()` → 显示结果 | Claude Code `launchRepl()` |
| 4.6 | 实现单次 prompt 模式 | `src/main.rs` — `-p` 参数时调用 `agent.run_turn()` 然后退出 | claw-code `CliAction::Prompt` |
| 4.7 | 流式输出集成 | `src/repl.rs` — 将 agent 的 TextDelta 事件实时打印（不需要等整个响应完成） | Claude Code 的流式输出体验 |
| 4.8 | 错误处理和用户反馈 | `src/repl.rs` — API 错误、工具错误的友好显示 | claw-code 错误处理 |
| 4.9 | Ctrl+C / 中断处理 | `src/repl.rs` — 优雅处理 SIGINT，中断当前 LLM 调用但不退出 REPL | Claude Code 的中断行为 |
| 4.10 | 端到端测试 | 手动测试 — 启动 REPL，输入 prompt，验证 LLM 响应和工具调用 | - |

**验证标准**:
- [ ] `cargo build -p ac-cli` 编译通过，生成 `autocode` binary
- [ ] `autocode` 启动后显示欢迎信息并等待输入
- [ ] `autocode -p "hello"` 单次模式正常工作
- [ ] `autocode --provider anthropic` / `--provider openai` 切换正常
- [ ] `autocode --allow` / `--read-only` 权限模式切换正常
- [ ] REPL 中输入 prompt 能获得流式 LLM 响应
- [ ] LLM 调用 Bash 工具能执行命令并返回结果
- [ ] LLM 使用 Read/Write/Edit 工具能操作文件
- [ ] Ctrl+C 能中断当前操作但不退出

---

### Phase 5: Grep + Context 压缩 + 持久化

> **开始前: 对比分析两个参考项目的搜索、压缩和持久化**
>
> 需要深入阅读并对比：
> - Claude Code: `src/tools/GrepTool/` + `src/tools/GlobTool/` — Grep 和 Glob 工具的实现
> - claw-code: `crates/tools/src/` 中的搜索工具 — Rust 版搜索实现
> - Claude Code: `src/query.ts` 中的 `compact()` 和 `microCompact()` — 两种压缩策略
> - claw-code: `crates/runtime/src/compact.rs` — Rust 版压缩实现
> - claw-code: `crates/runtime/src/session.rs` — JSONL 会话持久化
> - Claude Code: `src/services/api/` 中的 system prompt 构建
> - claw-code: `crates/runtime/src/prompt.rs` — system prompt 模板
>
> 对比重点：
> 1. Grep 实现策略：直接用 regex crate 还是调用外部 rg 命令
> 2. 压缩策略：Claude Code 的双策略（compact + micro-compact）vs claw-code 的单一 compact — MVP 用哪种
> 3. 压缩触发时机：token 阈值如何确定，何时调用压缩
> 4. 会话持久化格式：JSONL 的行格式、文件命名、会话恢复
> 5. System prompt 构建：如何包含工具说明、项目上下文

**目标**: 搜索能力、长对话支持、会话持久化

**交付物**:
- GrepTool（regex 搜索）
- ContextManager（消息压缩）
- Session 持久化（JSONL）
- System prompt

**详细步骤**:

| # | 步骤 | 产出文件 | 参考 |
|---|------|---------|------|
| 5.1 | 实现 `GrepTool` | `ac-tools/src/search.rs` — regex 搜索 + glob 过滤 + 目录遍历 + 行数限制 | Claude Code `GrepTool/` + claw-code search tools |
| 5.2 | 实现 token 估算 | `ac-runtime/src/context.rs` — 简单字符估算（chars / 3 或 chars / 4） | claw-code token 计数 |
| 5.3 | 实现 `ContextManager` | `ac-runtime/src/context.rs` — `maybe_compact()` 方法：超过阈值时压缩旧消息为摘要 | claw-code `compact_session()` + Claude Code `compact()` |
| 5.4 | 实现压缩逻辑 — LLM 摘要 | `ac-runtime/src/context.rs` — 调用 LLM 对旧消息生成摘要，替换为一条 system message | Claude Code 的摘要生成 |
| 5.5 | 将 ContextManager 集成到 Agent | `ac-runtime/src/agent.rs` — 在 `run_turn()` 的循环顶部调用 `maybe_compact()` | claw-code 的 auto-compaction |
| 5.6 | 定义 Session 格式 | `ac-runtime/src/session.rs` — `SessionMessage` 序列化格式（role + content + timestamp） | claw-code `session.rs` JSONL 格式 |
| 5.7 | 实现 `Session::append()` | `ac-runtime/src/session.rs` — 增量写入 JSONL | claw-code 增量写入 |
| 5.8 | 实现 `Session::load()` | `ac-runtime/src/session.rs` — 从 JSONL 恢复消息历史 | claw-code 会话恢复 |
| 5.9 | 实现 session 路径管理 | `ac-runtime/src/session.rs` — `~/.autocode/sessions/<workspace-hash>/` 目录结构 | claw-code 会话路径 |
| 5.10 | 编写 System Prompt | `ac-runtime/src/prompt.rs` — 工具使用规则、工作流程、代码风格指导 | Claude Code system prompt + claw-code `prompt.rs` |
| 5.11 | 将 Session 集成到 CLI | `ac-cli/src/repl.rs` — REPL 启动时创建/恢复 Session，每轮结束后持久化 | claw-code 会话管理 |
| 5.12 | 长对话端到端测试 | 手动测试 — 进行 10+ 轮对话，验证压缩触发、会话恢复 | - |

**验证标准**:
- [ ] `GrepTool` 能搜索目录中的文件内容
- [ ] `GrepTool` 支持 glob 过滤（如 `*.rs`）
- [ ] 长对话能自动触发压缩，不会超出上下文
- [ ] 会话关闭后重新打开能恢复历史
- [ ] System prompt 正确引导 LLM 使用工具

---

### Phase 6: AutoLang 移植（双轨路径）

> **策略**: VM FFI 桥接 + a2r 转译器增强并行推进
>
> - **轨道 A (VM FFI)**: AutoLang 代码通过 VM FFI 调用 Rust 实现的 agent 功能，立即可用
> - **轨道 B (a2r 增强)**: 分阶段补齐 a2r 转译器的语言特性支持，逐步实现「纯 .at 代码转译为 Rust」
>
> 两条轨道互相促进：VM FFI 提供即时可用的运行时能力，a2r 增强提供长期编译路径。

#### Phase 6A: VM FFI 桥接（已完成 FFI 部分）

**目标**: 通过 VM FFI 让 AutoLang 代码可以驱动 AutoCode 的全部功能

**已完成** (2026-04-08):

| # | FFI 函数 | NATIVE_ID | 状态 |
|---|---------|-----------|------|
| 6A.1 | `Process.spawn_with_output` | 1305 | ✅ 完成 |
| 6A.2 | `http_post_stream_with_headers` | 2255 | ✅ 完成 |
| 6A.3 | `Regex.is_match` | 2400 | ✅ 完成 |
| 6A.4 | `Regex.find_all` | 2401 | ✅ 完成 |
| 6A.5 | `File.walk` | 1010 | ✅ 完成 |
| 6A.6 | `File.append_text` | 1011 | ✅ 完成 |
| 6A.7 | `File.read_lines` | 1012 | ✅ 完成 |

**待完成（VM FFI 应用层）**:

| # | 步骤 | 产出文件 | 说明 |
|---|------|---------|------|
| 6A.8 | 创建 auto-code crate 骨架 | `crates/auto-code/Cargo.toml`, `src/lib.rs` | 空壳，导出 .at 模块 |
| 6A.9 | 移植 API 通信层 | `crates/auto-code/src/llm_client.at` | 用 http_post_stream_with_headers 调 LLM |
| 6A.10 | 移植 Anthropic 适配器 | `crates/auto-code/src/anthropic.at` | Claude API 格式 |
| 6A.11 | 移植 OpenAI 适配器 | `crates/auto-code/src/openai.at` | OpenAI API 格式 |
| 6A.12 | 移植工具系统 | `crates/auto-code/src/tools.at` | Bash/Read/Write/Edit/Grep |
| 6A.13 | 移植 Agent 循环 | `crates/auto-code/src/agent.at` | Agentic loop |
| 6A.14 | 移植 CLI + REPL | `crates/auto-code/src/repl.at` | 交互式 REPL |
| 6A.15 | 功能验证 | 测试 | 对比 Rust 原型功能 |

#### Phase 6B: a2r 转译器增强

**目标**: 补齐 a2r 转译器语言特性，使得 AutoLang 代码可以转译为与 Rust 原型等价的 Rust 代码

> 详细的特性差距分析见 **附录 F**。

**Phase 6B-1: Option/Result + HashMap + 方法链** (覆盖 ~60% 需求)

| # | 特性 | 当前状态 | 需要做什么 | 影响的 AutoCode 模块 |
|---|------|---------|-----------|-------------------|
| 6B-1.1 | Option 构造器转译 | ✅ **已完成** | `Some(expr)` → `Some(expr)`, `None` → `None` | 所有模块 |
| 6B-1.2 | Result 构造器转译 | ✅ **已完成** | `Ok(expr)` → `Ok(expr)`, `Err(expr)` → `Err(expr)` | 所有模块 |
| 6B-1.3 | Option 模式匹配转译 | ✅ **已完成** | `OptionPattern/OptionUncover` 转译完成 | Agent 循环、工具执行 |
| 6B-1.4 | Result 模式匹配转译 | ✅ **已完成** | `ResultPattern/ResultUncover` 转译完成 | API 响应处理 |
| 6B-1.5 | HashMap 类型 | ✅ **已完成** (Plan 160) | `Map<K, V>` 类型，a2r→`HashMap<K,V>`，a2c→`map_K_V*` | 工具注册表、消息历史 |
| 6B-1.6 | HashMap 字面量 | ⏸️ 类型注解已支持 | `{k: v}` 在有 Map 注解时仍输出结构体语法，需后续添加 HashMap::from 转译 | JSON 构造 |
| 6B-1.7 | 方法链支持 | 单方法支持 | 连续 `.method()` 调用转译正确 | 所有模块 |
| 6B-1.8 | `is` 语句完整 match arm | 只处理第一个语句 | 支持 match arm body 完整语句块 | Agent 循环、流解析 |

**Phase 6B-2: async fn + trait object + derive** (覆盖 ~85% 需求)

| # | 特性 | 当前状态 | 需要做什么 | 影响的 AutoCode 模块 |
|---|------|---------|-----------|-------------------|
| 6B-2.1 | `async fn` 声明 | ✅ test 134 | 返回 `~T` 自动 → `async fn`，unwrap 返回类型 | Agent 循环、API 调用 |
| 6B-2.2 | trait object `dyn` | ✅ test 031 | `spec` 作类型 → `Box<dyn Trait>`，`[]Spec` → `Vec<Box<dyn>>` | 工具注册表 |
| 6B-2.3 | derive 宏属性 | ✅ test 135 | `#[derive(Debug, Clone)]` 透传到 Rust | 所有核心类型 |
| 6B-2.4 | serde 标签枚举 | ✅ (2.3 统一实现) | `#[serde(tag = "type")]` 属性透传 | API 类型系统 |
| 6B-2.5 | 自定义属性透传 | ✅ (2.3 统一实现) | `#[serde(...)]` / `#[tokio::main]` 等直接透传 | API 层、CLI |
| 6B-2.6 | 外部 crate use | ✅ test 133 | `use.rust crate::module` → `use crate::module;` | 所有模块 |
| 6B-2.7 | `impl From<A> for B` | ⏸️ deferred | `ext From for B { fn from(a A) B }` → `impl From<A> for B` | 错误转换链 |

**Phase 6B-3: 完整模块系统 + 泛型约束** (覆盖 ~95% 需求)

| # | 特性 | 当前状态 | 需要做什么 |
|---|------|---------|-----------|
| 6B-3.1 | 泛型约束 `where T: Trait` | 无 | `fn foo<T>(x T) where T: Clone` |
| 6B-3.2 | Cargo.toml 自动生成 | 无 | 分析 use 语句，生成正确的 Cargo.toml |
| 6B-3.3 | 动态值类型 `Value` | 无 | `type Value` → `serde_json::Value` 或自定义 enum |
| 6B-3.4 | 静态常量 `const` | 无 | `const NAME: str = "..."` → `const NAME: &str = "..."` |
| 6B-3.5 | 多文件模块系统 | 单文件 | `use module::function` 真正解析为多文件 |

**Phase 6B-4: 深度特性差距（auto-code-rs 原型对比）** (覆盖 ~100% 需求)

> 以下分析基于 `auto-code-rs/` 原型全部源码（4 crate, 5325 行）与 a2r 转译器能力的逐行对比。
> 更新日期: 2026-04-12

**严重阻碍（Blocking）— 必须实现才能转译核心代码**:

| # | 功能 | auto-code-rs 示例 | 当前 a2r 状态 | 建议优先级 |
|---|------|-------------------|--------------|-----------|
| 6B-4.1 | **per-field serde 属性** | `#[serde(rename = "role")] content: String` | 仅支持 type 级 derive | P0 |
| 6B-4.2 | **`pub` 可见性** | `pub struct`, `pub fn`, `pub enum` | 所有输出都是私有的 | P0 |
| 6B-4.3 | **关联函数（无 self）** | `fn new() -> Self` 在 `impl Type` 块中 | 方法强制加 `&self` 参数 | P0 |
| 6B-4.4 | **impl Trait for Type（外部 trait）** | `impl Display for Message`, `impl Clone for Session` | 仅支持 spec（Auto 自定义 trait） | P0 |
| 6B-4.5 | **struct 解构匹配** | `Message::User { content } => ...` | 匹配模式不支持字段绑定 | P1 |
| 6B-4.6 | **`serde_json::json!` 宏** | `json!({"role": "user", "content": msg})` | 无宏展开支持 | P1 |
| 6B-4.7 | **`&mut self` 方法** | `fn push(&mut self, msg: Message)` | 方法总是 `&self` | P1 |
| 6B-4.8 | **`#[tokio::main]`** | 异步 main 函数 | 无属性透传到 main | P1 |
| 6B-4.9 | **`impl Into<String>` 参数** | `fn new(base_url: impl Into<String>)` | 无泛型约束语法 | P2 |
| 6B-4.10 | **方法链中的复杂闭包** | `.map(\|r\| r.content).collect::<Vec<_>>()` | 闭包类型推断有限 | P2 |

**中等阻碍 — 影响功能完整性**:

| # | 功能 | 说明 | 建议优先级 |
|---|------|------|-----------|
| 6B-4.11 | **`String` vs `&str` 精确区分** | Rust 中两者不可互换，当前 a2r 统一用 String | P2 |
| 6B-4.12 | **生命周期标注** | `&'a str`, `struct Session<'a>` | P3 |
| 6B-4.13 | **enum variant 构造 + 方法链** | `Message::system(content).with_model(model)` | P2 |
| 6B-4.14 | **`Box::new()` / `Arc::new()`** | 智能指针包装 | P2（部分已有 Box<dyn Spec>） |
| 6B-4.15 | **`Result<T>` 错误处理链** | `.map_err()?`, `anyhow::Result` | P2 |
| 6B-4.16 | **多文件模块系统** | `mod types; mod anthropic;` | P3（同 6B-3.5） |
| 6B-4.17 | **Cargo.toml 生成** | 依赖管理 | P3（同 6B-3.2） |
| 6B-4.18 | **`const` 常量** | `const API_URL: &str = "...";` | P3（同 6B-3.4） |
| 6B-4.19 | **`static` 变量** | `static CLIENT: Lazy<Client>` | P2（部分已有 global_vars） |
| 6B-4.20 | **泛型约束 `where T: Trait`** | `where T: Serialize + Clone` | P3（同 6B-3.1） |

**实施优先级建议**:

```
第一批（核心结构支持）— 解锁 ~70% 代码转译:
  1. 6B-4.2  pub 可见性         — 最简单，影响面最广
  2. 6B-4.3  关联函数（无 self）  — struct 构造器必需
  3. 6B-4.7  &mut self 方法     — 可变方法必需
  4. 6B-4.1  per-field 属性     — serde 集成必需

第二批（trait 系统）— 解锁 ~85% 代码转译:
  5. 6B-4.4  impl ExternalTrait for Type — Display/Clone 等 std trait
  6. 6B-3.1  泛型约束 where T: Trait

第三批（高级特性）— 解锁 ~95% 代码转译:
  7. 6B-4.5  struct 解构匹配
  8. 6B-4.10 复杂闭包/方法链
  9. 6B-3.5  多文件模块系统
```

**验证标准**:
- [ ] 所有新增 FFI 函数通过单元测试（✅ 已完成）
- [ ] AutoLang VM 版本能启动 REPL 并连接 LLM API（Phase 6A 待完成）
- [x] a2r 转译器支持 Option/Result 构造和匹配（Phase 6B-1）✅ test 130
- [x] a2r 转译器支持 HashMap 类型（Phase 6B-1）✅ Plan 160 完成
- [x] a2r 转译器支持 async fn 和 derive 宏（Phase 6B-2）✅ tests 133-135
- [ ] 纯 .at 代码转译为 Rust 后功能与 Rust 原型对等（Phase 6B-3）

### ac-api
```toml
[dependencies]
reqwest = { version = "0.12", features = ["stream", "json"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
futures = "0.3"
thiserror = "2"
```

### ac-tools
```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
regex = "1"
glob = "0.3"
walkdir = "2"
thiserror = "2"
```

### ac-runtime
```toml
[dependencies]
ac-api = { path = "../ac-api" }
ac-tools = { path = "../ac-tools" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
```

### ac-cli
```toml
[dependencies]
ac-runtime = { path = "../ac-runtime" }
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
dirs = "6"
```

---

## 12. 成功标准

### MVP（Phase 1-4）
- [ ] 能在 REPL 中输入 prompt，获得 LLM 响应
- [ ] LLM 能调用 Bash 工具执行命令
- [ ] LLM 能使用 Read/Write/Edit 工具操作文件
- [ ] 支持 Claude 和 OpenAI 两个 provider
- [ ] 权限系统工作（Ask 模式下提示用户确认）
- [ ] 流式输出实时显示

### 完整版（Phase 1-5）
- [ ] Grep 搜索工具工作
- [ ] 长对话不超出上下文（自动压缩）
- [ ] 会话可持久化和恢复
- [ ] System prompt 正确引导 LLM 行为

### AutoLang 版（Phase 6）
- [ ] VM FFI 版本：所有 7 个 FFI 函数通过测试（✅ 已完成）
- [ ] VM FFI 版本：AutoLang 代码能驱动 Agent 循环（Phase 6A 待完成）
- [ ] a2r Phase 6B-1：Option/Result 构造和匹配转译通过测试
- [x] a2r Phase 6B-1：HashMap 类型转译通过测试（128_map_type + 129_map_func）✅
- [ ] a2r Phase 6B-2：async fn + derive 宏转译通过测试
- [ ] a2r Phase 6B-3：纯 .at 代码转译为 Rust 后功能与原型对等

---

## 13. 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| SSE 解析复杂 | API 通信不稳定 | 先用非流式验证逻辑，再加流式 |
| Token 估算不准 | 上下文压缩过早或过晚 | 用保守估算（字符/3），后续优化 |
| 跨平台 Bash | Windows 兼容问题 | 使用 `cmd /C` 作为 fallback |
| LLM 输出格式不稳定 | 工具调用解析失败 | 容错处理，返回错误给 LLM 重试 |
| AutoLang a2r 特性不足 | 无法纯转译 | VM FFI 路径兜底，a2r 分阶段补齐 |
| HashMap 缺失 | 数据结构表达受限 | 短期用 JSON 字符串 + FFI 处理 |
| serde derive 缺失 | API 类型无法自动序列化 | 手写序列化或用 JSON FFI |
| async fn 支持不完整 | Agent 循环无法转译 | VM FFI 路径用 Task 系统替代 |

---

## 附录 D: 与参考实现的对比

| 特性 | Claude Code (TS) | claw-code (Rust) | AutoCode (Phase 1-5) |
|------|-----------------|-------------------|----------------------|
| 语言 | TypeScript/Bun | Rust/Tokio | Rust/Tokio |
| 架构 | React/Ink UI | Trait-based | Trait-based（参考 claw-code） |
| API 提供商 | Anthropic + Bedrock + Vertex | Anthropic + OpenAI + xAI | Anthropic + OpenAI + 兼容 |
| 工具数量 | 20+ | 40+ | 5 |
| SSE 解析 | SDK 内置 | 自定义 parser | 自定义 parser |
| 上下文压缩 | compact + micro-compact | compact_session | 简单摘要压缩 |
| 权限系统 | 5 种模式 + deny rules | 5 种模式 + rules | 3 种模式 |
| 会话持久化 | JSON | JSONL | JSONL |
| Hook 系统 | 完整 | 完整 | 无（MVP 不需要） |
| MCP | 完整 | 完整 | 无（MVP 不需要） |

## 附录 E: AutoLang VM FFI 差距分析（已完成）

| 能力 | 原有状态 | 现在状态 | NATIVE_ID |
|------|---------|---------|-----------|
| HTTP 客户端 | ✅ 有 | ✅ 有 | 2230-2255 |
| SSE 解析 | ✅ 有 | ✅ 有 | 2245-2250 |
| JSON 处理 | ✅ 有 | ✅ 有 | 1900-1917 |
| 文件读写 | ✅ 有 | ✅ 有 | 1000-1009 |
| 进程执行（仅 exit code） | ✅ 有 | ✅ 有 | 1304 |
| 进程执行（stdout/stderr） | ❌ 无 | ✅ **已新增** | 1305 |
| HTTP streaming + 自定义 headers | ❌ 无 | ✅ **已新增** | 2255 |
| Regex 搜索 | ❌ 无 | ✅ **已新增** | 2400-2401 |
| 目录递归遍历 | ❌ 无 | ✅ **已新增** | 1010 |
| 文件追加写入 | ❌ 无 | ✅ **已新增** | 1011 |
| 文件按行读取 | ❌ 无 | ✅ **已新增** | 1012 |
| 环境变量 | ✅ 有 | ✅ 有 | 1100-1102 |
| 异步/并发 | ✅ 有 | ✅ 有 | 2300-2311 |
| 字符串处理 | ✅ 丰富 | ✅ 有 | 1500-1509 |
| URL 处理 | ✅ 有 | ✅ 有 | 2000-2015 |

## 附录 F: a2r 转译器特性差距分析

> 本附录记录 a2r（Auto-to-Rust）转译器需要补齐的语言特性，以支持将 AutoCode 的 .at 代码
> 转译为与 Rust 原型 (`auto-code-rs/`) 功能等价的 Rust 代码。
>
> 分析基于：
> - AutoCode Rust 原型所有 crate 的功能需求
> - a2r 转译器现有实现 (`crates/auto-lang/src/trans/rust.rs`)
> - 现有 a2r 测试用例 (`crates/auto-lang/test/a2r/`)

### F.1 优先级 P0：Option/Result 构造与匹配 ✅ 已完成

**当前状态**: ✅ a2r 转译器已支持所有 Option/Result 表达式的转译。

**已完成**:
- ✅ `Some(expr)` → `Some(expr)`
- ✅ `None` → `None`
- ✅ `Ok(expr)` → `Ok(expr)`
- ✅ `Err(expr)` → `Err(expr)`
- ✅ `OptionPattern` → `Some(binding)` / `None` (在 is 分支中)
- ✅ `ResultPattern` → `Ok(binding)` / `Err(binding)` (在 is 分支中)
- ✅ `OptionUncover` → 提取绑定变量
- ✅ `ResultUncover` → 提取绑定变量
- ✅ a2r 测试 `130_option_construct` 通过

**待完成**:
- ⏸️ `impl From<A> for B` 自动转换（需要 a2r 支持外部 crate use）

### F.2 优先级 P0：HashMap/Map 类型 ✅ 已完成（Plan 160）

**当前状态**: ✅ `Type::Map(K, V)` 已添加到 AST。a2r 转译为 `HashMap<K, V>`。

**已完成**:
- ✅ `Type::Map(Box<Type>, Box<Type>)` 变体
- ✅ 类型转译: `Map<K, V>` → `std::collections::HashMap<K, V>` (a2r)
- ✅ 类型转译: `Map<K, V>` → `map_K_V*` (a2c), `Record<K, V>` (a2ts), `dict` (a2py)
- ✅ 14 个文件改动，2544 测试通过

**待完成**:
- ⏸️ 字面量: `{k: v, ...}` → `HashMap::from([(k, v), ...])`（需要类型上下文传递）
- ⏸️ 方法映射: `map.get(key)`, `map.insert(k, v)` 等

### F.3 优先级 P1：方法链完整性

**当前状态**: 单个方法调用已支持。连续调用如 `.iter().map(|x| x * 2).collect()` 
需要验证。AutoCode 大量使用函数式管道:
- `.iter().filter_map().collect()`
- `.lines().enumerate().collect()`
- `.chars().take(n).collect()`

**需要验证/修复**:
- 确保 `expr.method1().method2()` 链式调用正确转译
- 确保 `|param| body` 闭包作为方法参数时正确转译
- 确保 `.collect::<Vec<T>>()` turbofish 语法支持（或类型推断）

**影响的 AutoCode 模块**: 所有模块的数据处理

### F.4 优先级 P1：`is` 语句完整 match arm

**当前状态**: `is_stmt` 只处理 body 的第一个语句，复杂 match arm body 以 `/* TODO */` 占位。
AutoCode 的 SSE 解析和工具调用解析需要完整的 match arm body。

**需要实现**: `is_stmt` 支持完整的语句块作为 match arm body

**影响的 AutoCode 模块**: Agent 循环（StreamEvent 匹配）、工具执行

### F.5 优先级 P1：`async fn` 声明

**当前状态**: `~{ body }` → `async { body }` 已支持，但 `async fn` 声明不支持。
AutoCode 的核心 Agent 循环是 `async fn run_turn()`。

**需要实现**: `#[async] fn name() ReturnType { body }` → `async fn name() -> ReturnType { body }`

**影响的 AutoCode 模块**: Agent 循环、API 客户端

### F.6 优先级 P2：derive 宏与属性透传

**当前状态**: 无 derive 宏支持。AutoCode 的核心类型（ApiRequest, StreamEvent 等）全部
依赖 `#[derive(Debug, Clone, Serialize, Deserialize)]`。标签枚举依赖 `#[serde(tag = "type")]`。

**需要实现**:
- `#[derive(Debug, Clone)]` 属性透传
- `#[serde(tag = "type")]` 属性透传
- `#[serde(rename_all = "snake_case")]` 属性透传
- 通用 `#[...]` 属性直接透传到 Rust 输出

**影响的 AutoCode 模块**: API 类型系统（types.rs 等价物）

### F.7 优先级 P2：trait object 完整支持

**当前状态**: `spec` 生成 `dyn Name`，但 `Box<dyn Spec>` 模式不完整。
AutoCode 的工具注册表用 `HashMap<String, Box<dyn Tool>>` 存储异构工具实例。

**需要实现**: `Box<dyn SpecName>` 真正可用的转译

**影响的 AutoCode 模块**: 工具注册表 (ToolRegistry)

### F.8 优先级 P2：外部 crate 引用

**当前状态**: `use` 语句只做简单字符串替换（`auto.xxx` → `crate::xxx`）。
AutoCode 需要 `use reqwest::blocking::Client`, `use serde_json::Value` 等。

**需要实现**: `use rust:reqwest::blocking::Client` → `use reqwest::blocking::Client`

**影响的 AutoCode 模块**: 所有模块

### F.9 优先级 P3：泛型约束

**当前状态**: 泛型参数有，但无 `where T: Trait` 约束。

**需要实现**: `fn foo<T>(x T) where T: Clone + Debug` → `fn foo<T: Clone + Debug>(x: T)`

### F.10 优先级 P3：动态值类型

**当前状态**: 无。AutoCode 处理 LLM API 返回的动态 JSON，需要 `serde_json::Value`。

**需要实现**: `type Value` → `serde_json::Value` 或自定义 tag union

### F.11 优先级 P3：Cargo.toml 自动生成

**当前状态**: 无。转译后的 Rust 代码需要正确的 Cargo.toml。

**需要实现**: 分析 .at 代码中的 `use rust:xxx` 语句，自动生成 Cargo.toml 依赖

### F.12 优先级 P3：静态常量

**当前状态**: 无 `const` 声明。

**需要实现**: `const NAME str = "..."` → `const NAME: &str = "..."`

### F.13 a2r 已支持的关键特性（无需修改）

以下特性 a2r 已经支持，不需要额外工作：
- struct/type 定义（含泛型、继承、组合、委托）
- enum 定义（Scalar / Homogeneous / Heterogeneous）
- tag/ADT 定义
- spec/trait 定义
- ext/impl 块
- type alias
- 闭包/lambda
- 借用系统（view/mut/take/move）
- 指针操作
- f-string → format!()
- null coalescing (??)
- 错误传播 (.? → ?)
- async block (~{} → async {})
- .await / .go
- List<T> → Vec<T>
- Option<T>/Result<T> 类型映射
- 全局变量 → Lazy<Mutex<T>>
- print() → println!() 智能转译
