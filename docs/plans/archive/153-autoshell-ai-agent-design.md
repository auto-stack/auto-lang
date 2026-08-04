# Plan 153: AutoShell AI Agent 设计与实施

**Status**: Draft
**Created**: 2026-04-01
**Priority**: High
**Related**: [Plan 152: 流式 HTTP 与 SSE 解析](./152-streaming-http-sse.md) (Phase 1 阻塞性依赖)

---

# 第一部分：设计

## 参考仓库分析总结

### 已扫描的参考仓库

| 仓库 | 路径 | 关键发现 |
|------|------|----------|
| **Open Agent SDK** | D:\github\open-agent-sdk | ✅ 完整扫描 - 进程内 Agent 引擎，26+ 内置工具，完整 MCP 支持 |
| **Claude Code 原版** | D:\github\claude-code | 🔄 部分扫描 - Agent 系统架构、工具编排、MCP 协议 |
| **Claude Code Rust 版** | D:\github\claw-code | ⏸️ 待深入 - TypeScript→Rust 转译经验 |
| **OpenCode** | D:\github\opencode | ⏸️ 待深入 - VSCode 集成架构 |
| **NanoClaw/SafeClaw** | D:\github\nanoclaw | ⏸️ 待深入 - 安全增强机制 |

---

## 核心架构发现

### 1. Open Agent SDK 核心架构

```
用户代码 (createAgent/query)
    ↓
Agent (高层 API)
    ↓
QueryEngine (核心引擎)
    ↓
query() (查询执行)
    ↓
┌──────────┬──────────┬──────────┐
│ 工具系统  │ API客户端 │ MCP客户端 │
│ (26+工具) │          │          │
└──────────┴──────────┴──────────┘
```

**关键组件**：

1. **Agent 类** (`src/agent.ts`)
   - `query(prompt)` - 流式响应
   - `prompt(prompt)` - 阻塞响应
   - `getMessages()` - 会话历史
   - `clear()` - 重置会话

2. **QueryEngine** (`src/QueryEngine.ts`)
   - `submitMessage()` - 主查询循环
   - 消息管理、会话状态、使用量追踪

3. **工具系统** (`src/tools/`)
   - 26 个内置工具
   - Zod schema 验证
   - 并发安全检测
   - 权限系统

4. **MCP 客户端** (`src/services/mcp/`)
   - stdio/SSE/HTTP 传输
   - 工具/资源/提示词加载
   - 自动重连

### 2. Claude Code 工具编排机制

**关键发现**：`toolOrchestration.ts` 实现了智能的工具调用批处理：

```typescript
// 工具调用分批逻辑
function partitionToolCalls(toolUseMessages, toolUseContext): Batch[] {
    // 1. 检查每个工具的 isConcurrencySafe()
    // 2. 将并发安全的工具归为一批
    // 3. 非并发安全的工具串行执行
}
```

**批处理策略**：
- 只读工具（Read、Grep、Glob）→ 并发执行
- 写入工具（Write、Edit、Bash）→ 串行执行
- 自动检测工具的并发安全性

### 3. 工具接口设计

```typescript
type Tool<Input = AnyObject, Output = unknown> = {
    // 基本信息
    name: string
    description: string

    // Schema 定义
    inputSchema: z.Schema<Input>
    inputJSONSchema?: ToolInputJSONSchema

    // 核心方法
    call(args, context, canUseTool, onProgress): Promise<ToolResult<Output>>

    // 权限和验证
    checkPermissions(input, context): Promise<PermissionResult>
    validateInput?(input, context): Promise<ValidationResult>

    // 工具特性
    isConcurrencySafe(input): boolean
    isReadOnly(input): boolean
    isDestructive?(input): boolean

    // UI 渲染
    renderToolUseMessage(input, options): ReactNode
    renderToolResultMessage(content, options): ReactNode
}
```

---

## AutoShell AI Agent 设计方案

### 设计原则

基于参考仓库分析和 Auto 语言特性，我们确定以下设计原则：

1. **原生 Auto 实现** - 利用 Auto Task/Msg 系统，不是简单的移植
2. **与 AutoShell 深度集成** - 复用现有的命令执行、管道系统
3. **Token 效率优先** - 针对 Auto 语法优化的 Prompt 模板
4. **多提供商支持** - 抽象 LLM 接口，支持 Claude、OpenAI、本地模型
5. **可扩展工具系统** - 简单的工具注册机制，支持 MCP

---

## Agent 粒度决策：混合模式

### Auto 生命周期层次

```
Immortal (超越程序生命周期)
    ↓
Process (程序生命周期，全局变量)
    ↓
Auto (GC/RC 管理)
    ↓
Task (任务完成生命周期) ← 当前已实现
    ↓
Scope (函数/块作用域)
```

### Agent 粒度设计原则

基于 **混合模式**，不同类型的 Agent 使用不同的粒度：

| Agent 类型 | 生命周期 | 粒度 | 隔离级别 | 典型用途 |
|-----------|---------|------|----------|----------|
| **请求型 Agent** | 秒级 | Task | 共享内存 | 单次 LLM 调用、工具执行 |
| **会话型 Agent** | 分钟-小时级 | Task | 共享内存 | 用户对话、上下文维持 |
| **协调型 Agent** | 长期运行 | Task | 共享内存 | 任务分发、结果聚合 |
| **隔离型 Agent** | 小时-天级 | Process | 独立进程 | 沙箱执行、高安全性需求 |

### 粒度选择标准

**使用 Task 粒度** (默认):
- ✅ 需要与 AutoShell 共享状态（变量、文件句柄）
- ✅ 快速创建/销毁（毫秒级）
- ✅ 大量并发（数百个 Agent）
- ✅ 轻量级消息传递

**使用 Process 粒度**:
- ✅ 需要完全隔离（独立内存、独立运行时）
- ✅ 执行不可信代码（沙箱）
- ✅ 崩溃恢复（一个 Agent 崩溃不影响其他）
- ✅ 独立资源限制（CPU、内存）

---

## 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                     AutoShell REPL                          │
│  (/ask 命令、/agent 命令、自然语言输入)                      │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│                   AutoAgent Task                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  消息处理循环 (on 块)                                 │  │
│  │  - Ask(prompt, context) → LLM 调用                   │  │
│  │  - ToolCall(name, args) → 工具执行                   │  │
│  │  - SubAgentComplete(result) → 结果聚合               │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
┌───────▼──────┐  ┌─────▼─────┐  ┌──────▼──────┐
│ LLM Provider │  │ 工具系统   │  │ MCP 客户端  │
│              │  │           │  │             │
│ - Anthropic  │  │ ShellTool │  │ stdio/SSE   │
│ - OpenAI     │  │ FileTool  │  │ HTTP        │
│ - Local      │  │ GitTool   │  │ 工具代理    │
└──────────────┘  └───────────┘  └─────────────┘
```

### 核心 Auto Task 定义 (Task 粒度 Agent)

```auto
// Agent 消息协议
enum AgentMsg {
    // 用户请求
    Ask(prompt string, context Context)

    // LLM 响应
    Response(content string, tool_calls []ToolCall)

    // 工具调用
    ToolCall(name string, args object) ~ToolResult

    // 子 Agent 完成
    SubAgentComplete(task_id string, result object)

    // 错误
    Error(err string)

    // 进程间通信（Process 粒度 Agent）
    ForwardToProcess(target_process_id string, msg bytes)
    ProcessMessage(source_process_id string, msg bytes)
}

// 主 Agent Task
#[single]
task AutoAgent {
    // 私有状态
    config mut = AgentConfig{}
    messages mut = []Message
    tools mut = []Tool
    subagents mut = map[string]Handle<Agent>]

    fn start() ! {
        // 初始化工具
        self.tools = register_builtin_tools()

        // 初始化 LLM 客户端
        self.llm_client = LLMClient.new(self.config)
    }

    on(ctx) {
        Ask(prompt, ctx) => {
            // 1. 构建消息历史
            let messages = self.build_messages(prompt, ctx)

            // 2. 调用 LLM (流式)
            let stream = self.llm_client.chat_completion_stream(messages).await.

            // 3. 处理流式响应
            mut response = ""
            for chunk in stream {
                match chunk.type {
                    "content_block_delta" => {
                        response += chunk.delta.text
                        // 实时输出到终端
                        print(chunk.delta.text)
                    }
                    "content_block_start" => {
                        if chunk.content_block.type == "tool_use" {
                            // 提取工具调用
                            let tool_call = chunk.content_block
                            // 执行工具调用
                            let result = self.execute_tool(tool_call.name, tool_call.input).await.
                            // 添加到消息历史
                            self.messages.push(Message.tool_result(tool_call.id, result))
                        }
                    }
                }
            }

            ctx.reply(Response(response, []))
        }

        ToolCall(name, args) => {
            let result = self.execute_tool(name, args).await.
            ctx.reply(result)
        }
    }
}
```

### LLM Provider 抽象

```auto
// LLM Provider 接口
spec LLMProvider {
    fn chat_completion(messages []Message) ~Response
    fn chat_completion_stream(messages []Message) ~Stream
    fn supports_streaming() bool
}

// Anthropic Provider
task AnthropicProvider implements LLMProvider {
    api_key mut = ""
    base_url mut = "https://api.anthropic.com"

    on(ctx) {
        Request(messages) => {
            let response = http.post(f"{self.base_url}/v1/messages")
                .header("x-api-key", self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .body(json! {
                    model: "claude-3-5-sonnet-20241022",
                    messages: messages,
                    stream: true
                })
                .await.?

            ctx.reply(Response.from_http(response))
        }
    }
}

// OpenAI Provider
task OpenAIProvider implements LLMProvider {
    api_key mut = ""
    base_url mut = "https://api.openai.com"

    on(ctx) {
        Request(messages) => {
            let response = http.post(f"{self.base_url}/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .body(json! {
                    model: "gpt-4o",
                    messages: messages,
                    stream: true
                })
                .await.?

            ctx.reply(Response.from_http(response))
        }
    }
}

// 本地模型 Provider (Ollama)
task LocalLLMProvider implements LLMProvider {
    base_url mut = "http://localhost:11434"

    on(ctx) {
        Request(messages) => {
            let response = http.post(f"{self.base_url}/api/chat")
                .body(json! {
                    model: "llama3.2",
                    messages: messages,
                    stream: true
                })
                .await.?

            ctx.reply(Response.from_http(response))
        }
    }
}
```

### 工具系统设计

```auto
// 工具接口
spec Tool {
    fn name() string
    fn description() string
    fn input_schema() Schema
    fn is_read_only() bool
    fn is_concurrency_safe() bool
    fn execute(args object, context Context) ~ToolResult
}

// Shell 命令工具
task ShellTool implements Tool {
    fn name() string { "bash" }

    fn description() string {
        "Execute shell commands in the current directory"
    }

    fn input_schema() Schema {
        Schema.object! {
            "command": Schema.string().describe("Shell command to execute")
            "timeout": Schema.int().optional().describe("Timeout in seconds")
        }
    }

    fn is_read_only() bool { false }

    fn is_concurrency_safe() bool { false }

    fn execute(args object, context Context) ~ToolResult {
        let cmd = args["command"].as_str()
        let timeout = args.get("timeout").as_int_or(30)

        // 使用 AutoShell 的命令执行
        let result = shell_exec_with_timeout(cmd, timeout).await.

        ToolResult.success(result)
    }
}

// 文件读取工具
task FileReadTool implements Tool {
    fn name() string { "read_file" }

    fn description() string {
        "Read the contents of a file"
    }

    fn input_schema() Schema {
        Schema.object! {
            "path": Schema.string().describe("File path to read")
            "offset": Schema.int().optional().describe("Byte offset to start reading")
            "limit": Schema.int().optional().describe("Maximum bytes to read")
        }
    }

    fn is_read_only() bool { true }

    fn is_concurrency_safe() bool { true }

    fn execute(args object, context Context) ~ToolResult {
        let path = args["path"].as_str()
        let offset = args.get("offset").as_int_or(0)
        let limit = args.get("limit").as_int_or(-1)

        let content = fs.read(path, offset, limit).await.

        ToolResult.success(content)
    }
}

// 工具注册表
task ToolRegistry {
    tools mut = map[string]Tool

    fn start() ! {
        // 注册内置工具
        self.register(ShellTool{})
        self.register(FileReadTool{})
        self.register(FileWriteTool{})
        self.register(GrepTool{})
        self.register(GlobTool{})
    }

    on(ctx) {
        Register(tool) => {
            self.tools[tool.name()] = tool
        }

        Get(name) => {
            ctx.reply(self.tools.get(name))
        }

        List() => {
            ctx.reply(self.tools.values())
        }
    }
}
```

### MCP 客户端实现

```auto
// MCP 连接类型
enum MCPTransport {
    Stdio { command: string, args: []string }
    SSE { url: string }
    HTTP { url: string }
}

// MCP 服务器连接
task MCPServerConnection {
    transport mut = MCPTransport
    client mut = MCPClient

    fn start() ! {
        match self.transport {
            Stdio { command, args } => {
                // 启动子进程
                let process = spawn_process(command, args)
                self.client = MCPClient.new_stdio(process.stdin, process.stdout)
            }
            SSE { url } => {
                self.client = MCPClient.new_sse(url)
            }
            HTTP { url } => {
                self.client = MCPClient.new_http(url)
            }
        }

        // 初始化握手
        self.client.initialize().await.
    }

    on(ctx) {
        ListTools() => {
            let tools = self.client.list_tools().await.
            ctx.reply(tools)
        }

        CallTool(name, args) => {
            let result = self.client.call_tool(name, args).await.
            ctx.reply(result)
        }

        ListResources() => {
            let resources = self.client.list_resources().await.
            ctx.reply(resources)
        }

        ReadResource(uri) => {
            let content = self.client.read_resource(uri).await.
            ctx.reply(content)
        }
    }
}

// MCP 工具包装器
task MCPTool implements Tool {
    server_name mut = ""
    tool_name mut = ""
    connection mut = Handle<MCPServerConnection>

    fn name() string {
        f"mcp.{self.server_name}.{self.tool_name}"
    }

    fn description() string {
        // 从 MCP schema 获取
        self.connection.ask(GetToolSchema(self.tool_name)).await.
            .description
    }

    fn input_schema() Schema {
        self.connection.ask(GetToolSchema(self.tool_name)).await.
            .input_schema
    }

    fn is_read_only() bool {
        // 尝试从 schema 推断
        true
    }

    fn is_concurrency_safe() bool {
        // 尝试从 schema 推断
        true
    }

    fn execute(args object, context Context) ~ToolResult {
        let result = self.connection.ask(CallTool(self.tool_name, args)).await.
        ToolResult.from_mcp(result)
    }
}
```

### 多 Agent 协作

```auto
// 协调 Agent (Task 粒度)
task CoordinatorAgent {
    workers mut = map[string]Handle<WorkerAgent>]

    on(ctx) {
        Task(description) => {
            // 1. 分解任务
            let subtasks = self.decompose_task(description)

            // 2. 为每个子任务创建 Worker Agent
            mut results = []object
            for subtask in subtasks {
                let worker = WorkerAgent.spawn()
                worker.ask(Start(subtask)).await.
                let result = worker.ask(Join()).await.
                results.push(result)
            }

            // 3. 聚合结果
            let final = self.aggregate_results(results)
            ctx.reply(Complete(final))
        }
    }
}

// Worker Agent
task WorkerAgent {
    task mut = Task
    status mut = "idle"
    result mut = null

    on(ctx) {
        Start(task) => {
            self.task = task
            self.status = "running"

            // 执行任务...
            let result = self.execute_task(task).await.
            self.result = result
            self.status = "completed"
        }

        Join() => {
            ctx.reply(self.result)
        }

        Status() => {
            ctx.reply(self.status)
        }
    }
}

// Worker Agent (支持 Task 和 Process 两种粒度)
enum WorkerGranularity {
    Task,   // 轻量级，共享内存
    Process // 重量级，独立进程
}

task WorkerAgent {
    task mut = Task
    granularity mut = WorkerGranularity::Task
    process_id mut = null  // Process 粒度时的进程 ID

    on(ctx) {
        Start(task, gran) => {
            self.task = task
            self.granularity = gran

            match gran {
                WorkerGranularity::Task => {
                    // 直接执行
                    let result = self.execute_task(task).await.
                    self.result = result
                }
                WorkerGranularity::Process => {
                    // 启动独立进程
                    let process_id = spawn_agent_process(task)
                    self.process_id = process_id
                    // 等待进程完成
                    let result = wait_for_process(process_id).await.
                    self.result = result
                }
            }
            self.status = "completed"
        }

        Join() => {
            ctx.reply(self.result)
        }

        Status() => {
            ctx.reply(self.status)
        }
    }
}

// Process 粒度 Agent 的进程间通信
task ProcessAgentBridge {
    // 跨进程消息传递
    on(ctx) {
        ForwardToProcess(target_process_id, msg_bytes) => {
            // 通过 IPC 发送消息到目标进程
            ipc_send(target_process_id, msg_bytes).await.
        }

        ProcessMessage(source_process_id, msg_bytes) => {
            // 从其他进程接收消息
            let msg = deserialize_message(msg_bytes)
            // 处理消息...
        }
    }
}
```

---

## Process 粒度 Agent 设计

### 进程隔离模型

```auto
// Process Agent 定义（需要特殊语法支持）
#[process]  // 标注为进程级 Agent
process IsolatedAgent {
    // 进程私有状态（不共享）
    config mut = AgentConfig{}

    fn start() ! {
        // 进程启动时的初始化
        // 独立的 AutoVM 实例
        // 独立的内存空间
    }

    on(ctx) {
        // 消息处理循环
        // 通过 IPC 接收消息
    }

    fn stop() ! {
        // 进程关闭时的清理
    }
}
```

### 进程间通信 (IPC)

```auto
// IPC 消息格式
enum IPCMessage {
    Request {
        id: string
        method: string
        payload: bytes
    }
    Response {
        id: string
        payload: bytes
    }
    Error {
        id: string
        error: string
    }
}

// IPC 传输层
enum IPCTransport {
    Stdio,      // 标准输入输出（类似 MCP stdio）
    SharedMemory,  // 共享内存
    UnixSocket,     // Unix 域套接字
    NamedPipe,      // 命名管道 (Windows)
    TCP         // TCP 套接字（跨机器）
}

// IPC 客户端
task IPCClient {
    transport mut = IPCTransport::Stdio
    connection mut = Connection

    fn connect(address string) ! {
        match self.transport {
            IPCTransport::Stdio => {
                // 使用 stdin/stdout
            }
            IPCTransport::UnixSocket => {
                self.connection = connect_unix_socket(address).await.
            }
            IPCTransport::TCP => {
                self.connection = connect_tcp(address).await.
            }
        }
    }

    fn send(msg IPCMessage) ! {
        let bytes = serialize(msg)
        self.connection.write(bytes).await.
    }

    fn receive() ~IPCMessage {
        let bytes = self.connection.read().await.
        deserialize(bytes)
    }
}
```

### 进程生命周期管理

```auto
// 进程注册表
task ProcessRegistry {
    processes mut = map[string]ProcessInfo]

    on(ctx) {
        Spawn(config) => {
            let process_id = generate_id()

            // 启动新进程
            match config.transport {
                IPCTransport::Stdio => {
                    let child = spawn_process_with_stdio(
                        config.executable,
                        config.args
                    )
                    self.processes[process_id] = ProcessInfo {
                        id: process_id,
                        pid: child.id,
                        transport: config.transport,
                        status: "running"
                    }
                }
                // ... 其他传输方式
            }

            ctx.reply(process_id)
        }

        Stop(process_id) => {
            let info = self.processes[process_id]
            kill_process(info.pid)
            info.status = "stopped"
            ctx.reply(OK)
        }

        List() => {
            ctx.reply(self.processes.values())
        }
    }
}
```

---

## Agent 类型设计

### 1. 请求型 Agent (Request Agent)

**粒度**: Task
**生命周期**: 秒级
**用途**: 单次 LLM 调用、简单工具执行

```auto
task RequestAgent {
    on(ctx) {
        Execute(request) => {
            let result = self.process_request(request).await.
            ctx.reply(result)
            // 任务完成后自动销毁
        }
    }
}
```

### 2. 会话型 Agent (Session Agent)

**粒度**: Task
**生命周期**: 分钟-小时级
**用途**: 用户对话、上下文维持

```auto
task SessionAgent {
    session_id mut = ""
    messages mut = []Message
    context mut = Context

    on(ctx) {
        Start(session_id) => {
            self.session_id = session_id
            // 加载历史消息
            self.messages = load_session_history(session_id)
        }

        Chat(user_msg) => {
            // 添加用户消息
            self.messages.push(user_msg)

            // 调用 LLM
            let response = self.llm_client.chat(self.messages).await.

            // 添加助手响应
            self.messages.push(response)

            // 持久化会话
            save_session_history(self.session_id, self.messages)

            ctx.reply(response)
        }

        Clear() => {
            self.messages = []
            ctx.reply(OK)
        }
    }
}
```

### 3. 协调型 Agent (Coordinator Agent)

**粒度**: Task
**生命周期**: 长期运行
**用途**: 任务分发、结果聚合

```auto
task CoordinatorAgent {
    workers mut = map[string]Handle<WorkerAgent>]
    task_queue mut = []Task
    results mut = map[string]Result]

    on(ctx) {
        Submit(task) => {
            // 分解任务
            let subtasks = self.decompose(task)

            // 分发给 Worker
            for subtask in subtasks {
                let worker = self.get_or_spawn_worker()
                worker.ask(Execute(subtask)).await.
            }

            ctx.reply(TaskAccepted)
        }

        Complete(task_id, result) => {
            self.results[task_id] = result

            // 检查是否所有子任务完成
            if self.all_tasks_completed() {
                let final = self.aggregate_results()
                ctx.reply(AllTasksComplete(final))
            }
        }
    }
}
```

### 4. 工作型 Agent (Worker Agent)

**粒度**: Task 或 Process
**生命周期**: 按需创建/销毁
**用途**: 执行具体工具调用

```auto
task WorkerAgent {
    task mut = null
    status mut = "idle"

    on(ctx) {
        Execute(task) => {
            self.task = task
            self.status = "running"

            // 执行任务...
            let result = self.execute_task(task).await.

            self.status = "idle"
            ctx.reply(TaskComplete(result))
        }
    }
}
```

### 5. 隔离型 Agent (Sandbox Agent)

**粒度**: Process
**生命周期**: 按需创建/销毁
**用途**: 沙箱执行、高安全性需求

```auto
#[process]
process SandboxAgent {
    // 独立进程，完全隔离
    on(ctx) {
        ExecuteUntrusted(code) => {
            // 在沙箱中执行代码
            let result = sandbox_execute(code).await.
            ctx.reply(result)
        }
    }
}
```

---

## 粒度切换策略

### 自动升级规则

```auto
fn decide_granularity(agent_config AgentConfig) WorkerGranularity {
    // 1. 检查是否需要隔离
    if agent_config.requires_sandbox {
        return WorkerGranularity::Process
    }

    // 2. 检查资源需求
    if agent_config.max_memory > 1GB || agent_config.max_cpu > 2 {
        return WorkerGranularity::Process
    }

    // 3. 检查任务类型
    match agent_config.task_type {
        TaskType::UntrustedExecution => WorkerGranularity::Process
        TaskType::NormalProcessing => WorkerGranularity::Task
    }

    // 默认使用 Task 粒度
    WorkerGranularity::Task
}
```

---

## 实施计划概览

| Phase | 内容 | 预计时间 | 依赖 |
|-------|------|----------|------|
| **Phase 1** | LLM API 基础 | 9.5天 | Plan 152 |
| **Phase 2** | Agent Task 基础 | 2-3周 | Phase 1 |
| **Phase 3** | 工具系统完善 | 2-3周 | Phase 2 |
| **Phase 4** | MCP 客户端 | 2-3周 | Phase 2 |
| **Phase 5** | 多 Agent 协作 | 3-4周 | Phase 2 |
| **Phase 6** | AutoShell 集成 | 2-3周 | Phase 2 |
| **Phase 7** | Process 粒度 Agent | 3-4周 | IPC 基础设施 |
| **Phase 8** | 粒度自动切换 | 2-3周 | Phase 7 |

---

# 第二部分：Phase 1 实施计划

## 状态更新：`~T` 已实现

**Date**: 2026-04-01

### 重要发现：`~T` Future 类型已实现！

经过检查，以下计划已经**全部完成**：

| 计划 | 状态 | 说明 |
|------|------|------|
| **Plan 121** | ✅ 完成 | Task/Msg 基础系统 |
| **Plan 124** | ✅ 完成 | `~T` Future 类型，`.await`，`ask/reply` |
| **Plan 125** | ✅ 完成 | 多态路由、消息上下文 |
| **Plan 126** | ✅ 完成 | `.go` Worker Pool |
| **Plan 127** | ✅ 完成 | TaskSystem VM 执行 |
| **Plan 128** | ✅ 完成 | 调度器消息分发 |

**这意味着**：Phase 1 可以直接实施**异步流式版本**，无需降级到同步阻塞调用！

### 更新的实施策略

#### 原计划（已过时）

```
Phase 1A: 同步阻塞版本（降级方案）
    ↓
Phase 1B: 等待 Plan 124 完成后再实施异步版本
```

#### 新策略（推荐）

```
直接实施异步流式 LLM API
    ├── 利用 Plan 124 的 ~T 类型
    ├── 利用 Plan 121 的 Task/Msg 系统
    └── 利用 Plan 126 的 .go 并发
```

### 更新的时间估算

| 方案 | 工作量 | 依赖 |
|------|--------|------|
| **原 Phase 1A + 1B** | 16天 | Plan 124（待完成） |
| **新方案：直接异步** | **9.5天** | Plan 124 ✅ |

**节省时间**：约 6.5 天

---

## Phase 1: LLM API 基础 - 详细实施

**目标**: 实现基础的 LLM API 调用能力

**依赖**: [Plan 152: 流式 HTTP 与 SSE 解析](./152-streaming-http-sse.md) (阻塞依赖)

### Auto 基础设施现状分析

#### ✅ 已有基础设施

| 模块 | 文件 | 功能 | 状态 |
|------|------|------|------|
| **HTTP 客户端** | `stdlib/auto/http.at` | `http.get()`, `http.post()`, `RequestBuilder` | ✅ 可用 |
| **TCP 网络** | `stdlib/auto/net.at` | `tcp_bind()`, `tcp_connect()` | ✅ 可用 |
| **JSON 序列化** | `stdlib/auto/json.at` | `encode()`, `decode()`, `parse()` | ✅ 可用 |
| **URL 处理** | `stdlib/auto/url.at` | `encode()`, `decode()`, `parse()` | ✅ 可用 |
| **Process 基础** | `stdlib/auto/process.at` | `spawn()`, `args()`, `current_dir()` | ✅ 可用 |
| **Async 基础** | `stdlib/auto/async.at` | `spawn()`, `channel()`, `sleep()` | ✅ 可用 |
| **`~T` Future** | Plan 124 ✅ | `~T` 类型, `.await`, `ask/reply` | ✅ **已完成** |
| **Task/Msg** | Plan 121 ✅ | `task`, `spawn`, `send`, `TaskSystem.start()` | ✅ **已完成** |
| **`.go` 并发** | Plan 126 ✅ | `.go` 后缀操作符, Worker Pool | ✅ **已完成** |

#### ⚠️ 部分实现/需要扩展

| 模块 | 文件 | 当前状态 | 需要扩展 |
|------|------|----------|----------|
| **HTTP 客户端** | `http.at` | 同步阻塞调用 | 需要流式响应支持 |
| **Process** | `process.at` | 简单 `spawn()` | 需要进程间通信 (IPC) |

---

## 模块结构

```
crates/auto-shell/src/llm/
├── mod.rs              # 模块导出
├── types.rs            # 核心类型定义
├── provider.rs         # Provider trait 定义
├── anthropic.rs        # Anthropic API 实现
├── openai.rs           # OpenAI API 实现
├── local.rs            # 本地模型 (Ollama) 实现
└── error.rs            # 错误类型
```

---

## 核心类型定义

```rust
// crates/auto-shell/src/llm/types.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LLM 消息角色
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// LLM 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

/// LLM 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub model: String,
    pub usage: Usage,
    pub stop_reason: Option<String>,
}

/// Token 使用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// LLM 配置
#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
            base_url: None,
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
        }
    }
}
```

---

## Provider Trait

```rust
// crates/auto-shell/src/llm/provider.rs

use async_trait::async_trait;
use crate::llm::types::{LLMConfig, LLMResponse, Message};

/// 异步 LLM Provider trait
#[async_trait]
pub trait AsyncLLMProvider: Send + Sync {
    /// 异步发送聊天请求，返回流式响应
    async fn chat_stream(&self, messages: &[Message]) -> impl Stream<Item = StreamEvent>;

    /// 异步发送聊天请求（阻塞版本）
    async fn chat(&self, messages: &[Message]) -> LLMResponse;

    /// 获取 Provider 名称
    fn name(&self) -> &str;

    /// 检查配置是否有效
    fn validate_config(&self) -> Result<(), LLMError>;
}

/// 流式事件
pub enum StreamEvent {
    ContentDelta(String),
    ToolUse(ToolUseCall),
    Done,
}

/// LLM 错误类型
#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Authentication failed: invalid API key")]
    AuthenticationFailed,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}
```

---

## Anthropic Provider 实现

```rust
// crates/auto-shell/src/llm/anthropic.rs

use crate::llm::provider::{LLMError, AsyncLLMProvider, StreamEvent};
use crate::llm::types::{LLMConfig, LLMResponse, Message, Usage};
use reqwest::Client;
use futures::stream::{Stream, StreamExt};

pub struct AnthropicProvider {
    config: LLMConfig,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(config: LLMConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    fn base_url(&self) -> String {
        self.config.base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com")
    }
}

#[async_trait]
impl AsyncLLMProvider for AnthropicProvider {
    async fn chat_stream(&self, messages: &[Message]) -> impl Stream<Item = StreamEvent> {
        let response = self.client
            .post(format!("{}/v1/messages", self.base_url()))
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&json!({
                "model": self.config.model,
                "messages": messages,
                "stream": true,
                "max_tokens": self.config.max_tokens.unwrap_or(4096)
            }))
            .send()
            .await
            .expect("Failed to send request");

        // 使用 Plan 152 的 SSE 解析器
        let stream = parse_sse_stream(response).await;

        stream
    }

    async fn chat(&self, messages: &[Message]) -> LLMResponse {
        let mut content = String::new();
        let mut stream = self.chat_stream(messages).await;

        while let Some(event) = stream.next().await {
            match event {
                StreamEvent::ContentDelta(text) => {
                    content.push_str(&text);
                }
                StreamEvent::Done => break,
                _ => {}
            }
        }

        LLMResponse {
            content,
            model: self.config.model.clone(),
            usage: Usage {
                input_tokens: 0,
                output_tokens: 0,
            },
            stop_reason: Some("end_turn".to_string()),
        }
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    fn validate_config(&self) -> Result<(), LLMError> {
        if self.config.api_key.is_empty() {
            return Err(LLMError::AuthenticationFailed);
        }
        Ok(())
    }
}
```

---

## Provider 工厂

```rust
// crates/auto-shell/src/llm/mod.rs

use crate::llm::provider::{LLMError, AsyncLLMProvider};
use crate::llm::types::LLMConfig;
use std::sync::Arc;

pub mod anthropic;
pub mod local;
pub mod openai;
pub mod provider;
pub mod types;
pub mod error;

pub use provider::{AsyncLLMProvider, LLMError, StreamEvent};
pub use types::{LLMConfig, LLMResponse, Message, MessageRole, Usage};

/// Provider 类型
#[derive(Debug, Clone, Copy)]
pub enum ProviderType {
    Anthropic,
    OpenAI,
    Local,
}

/// 创建 Provider
pub fn create_provider(provider_type: ProviderType, config: LLMConfig) -> Arc<dyn AsyncLLMProvider> {
    match provider_type {
        ProviderType::Anthropic => Arc::new(anthropic::AnthropicProvider::new(config)),
        ProviderType::OpenAI => Arc::new(openai::OpenAIProvider::new(config)),
        ProviderType::Local => Arc::new(local::LocalLLMProvider::new(config)),
    }
}

/// 从环境变量自动检测 Provider
pub fn detect_provider_from_env() -> (ProviderType, LLMConfig) {
    // 优先级: OpenAI > Anthropic > Local
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        let config = LLMConfig {
            api_key: key,
            base_url: std::env::var("OPENAI_BASE_URL").ok(),
            model: std::env::var("OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-4o".to_string()),
            ..Default::default()
        };
        return (ProviderType::OpenAI, config);
    }

    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        let config = LLMConfig {
            api_key: key,
            base_url: std::env::var("ANTHROPIC_BASE_URL").ok(),
            model: std::env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string()),
            ..Default::default()
        };
        return (ProviderType::Anthropic, config);
    }

    // 默认使用本地模型
    let config = LLMConfig {
        api_key: "local".to_string(),
        base_url: std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string()),
        model: std::env::var("OLLAMA_MODEL")
            .unwrap_or_else(|_| "llama3.2".to_string()),
        ..Default::default()
    };
    (ProviderType::Local, config)
}
```

---

## Auto 标准库扩展

```auto
/// stdlib/auto/llm.at

/// LLM 模块 - 大语言模型 API 集成
///
/// 提供统一的异步 LLM API，支持多个提供商

/// LLM Provider 类型
type LLMProvider

/// 创建 Anthropic Provider
#[pub]
fn anthropic(api_key str, model str) LLMProvider

/// 创建 OpenAI Provider
#[pub]
fn openai(api_key str, model str) LLMProvider

/// 创建本地模型 Provider (Ollama)
#[pub]
fn local(base_url str, model str) LLMProvider

/// 异步发送聊天请求
/// 返回流式响应 ~Response
#[pub]
fn chat_stream(provider LLMProvider, messages []Message) ~Response

/// 异步发送聊天请求（阻塞等待完整响应）
#[pub]
fn chat(provider LLMProvider, messages []Message) ~Response

/// LLM 消息类型
type Message

/// 创建用户消息
#[pub]
fn user_message(content str) Message

/// 创建系统消息
#[pub]
fn system_message(content str) Message

/// 创建助手消息
#[pub]
fn assistant_message(content str) Message

/// LLM 响应类型
type Response

/// 获取响应内容
#[pub]
fn Response.content(self Response) str

/// 获取使用量信息
#[pub]
fn Response.usage(self Response) Usage

/// 使用量类型
type Usage

/// 输入 Token 数量
#[pub]
fn Usage.input_tokens(self Usage) int

/// 输出 Token 数量
#[pub]
fn Usage.output_tokens(self Usage) int

/// 流式事件类型
type StreamEvent

/// 内容增量事件
#[pub]
fn is_content_delta(event StreamEvent) int

/// 工具调用事件
#[pub]
fn is_tool_use(event StreamEvent) int

/// 完成事件
#[pub]
fn is_done(event StreamEvent) int
```

---

## 示例用法

```auto
// 在 Auto 代码中使用异步 LLM
use llm: { anthropic, chat_stream, user_message, Response }
use async: { TaskSystem }

task ChatAgent {
    on(ctx) {
        Ask(question) => {
            let provider = anthropic("sk-ant-xxx", "claude-3-5-sonnet-20241022")

            let messages = [user_message(question)]

            // 异步流式调用
            let response = chat_stream(provider, messages).await.

            // 处理流式响应
            ctx.reply(Response(response.content()))
        }
    }
}

fn main() ! {
    let agent = ChatAgent.spawn()

    agent.ask("What is the capital of France?")

    TaskSystem.start()
}
```

---

## 实施任务

| 任务 | 工作量 | 依赖 |
|------|--------|------|
| 核心类型定义 | 0.5天 | 无 |
| Provider trait | 0.5天 | 无 |
| Anthropic 实现 | 1.5天 | Plan 152 |
| OpenAI 实现 | 1.5天 | Plan 152 |
| 本地模型实现 | 1天 | Plan 152 |
| Auto FFI 绑定 | 1天 | Provider 完成 |
| 测试 | 1天 | 以上全部 |
| 文档 | 0.5天 | 以上全部 |
| **Phase 1 小计** | **7.5天** | Plan 152 |

加上 Plan 152 的 7 天，**总计 14.5 天**（约 3 周）

---

## 验收标准

- [ ] `cargo build -p auto-shell` 编译成功
- [ ] `cargo test -p auto-shell -- llm` 所有测试通过
- [ ] 可以调用 Anthropic API 并获得响应
- [ ] 可以调用 OpenAI API 并获得响应
- [ ] 可以调用 Ollama 本地模型并获得响应
- [ ] 支持流式输出
- [ ] 错误处理正确（认证失败、网络错误等）
- [ ] 从环境变量自动检测 Provider

---

## 文件清单

### 新增文件

```
crates/auto-shell/src/llm/
├── mod.rs              # 模块导出 (~100 lines)
├── types.rs            # 核心类型定义 (~150 lines)
├── provider.rs         # Provider trait (~100 lines)
├── anthropic.rs        # Anthropic API (~200 lines)
├── openai.rs           # OpenAI API (~180 lines)
├── local.rs            # 本地模型 (~150 lines)
├── error.rs            # 错误类型 (~30 lines)
└── tests.rs            # 测试 (~100 lines)

stdlib/auto/llm.at      # Auto 标准库接口 (~100 lines)
```

### 修改文件

```
crates/auto-shell/Cargo.toml    # 添加依赖
crates/auto-shell/src/lib.rs    # 导出 llm 模块
```

---

## Cargo.toml 依赖

```toml
# crates/auto-shell/Cargo.toml

[dependencies]
# 现有依赖...
auto-lang = { path = "../crates/auto-lang" }
auto-val = { path = "../crates/auto-val" }

# 新增 LLM 相关依赖
reqwest = { version = "0.12", features = ["json", "stream"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
async-trait = "0.1"
thiserror = "1.0"
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"
```

---

# 第三部分：后续 Phases

## Phase 2: Agent Task 基础 (2-3周)

**目标**: 实现基础的 Agent Task

**任务**:
- [ ] AutoAgent Task 定义
- [ ] 消息协议设计
- [ ] 基础工具注册系统
- [ ] Shell 命令工具
- [ ] 文件操作工具（Read/Write）
- [ ] 工具执行编排（参考 Claude Code 的批处理逻辑）

**关键文件**:
- `crates/auto-shell/src/agent/mod.rs`
- `crates/auto-shell/src/agent/task.rs`
- `crates/auto-shell/src/agent/messages.rs`
- `crates/auto-shell/src/tools/mod.rs`
- `crates/auto-shell/src/tools/shell.rs`
- `crates/auto-shell/src/tools/file.rs`

## Phase 3: 工具系统完善 (2-3周)

**目标**: 完善工具系统，添加更多工具

**任务**:
- [ ] Grep 工具
- [ ] Glob 工具
- [ ] Git 工具
- [ ] 工具权限系统
- [ ] 工具并发安全检测
- [ ] 工具进度报告

**关键文件**:
- `crates/auto-shell/src/tools/grep.rs`
- `crates/auto-shell/src/tools/glob.rs`
- `crates/auto-shell/src/tools/git.rs`
- `crates/auto-shell/src/tools/permissions.rs`

## Phase 4: MCP 客户端 (2-3周)

**目标**: 实现 MCP 协议支持

**任务**:
- [ ] MCP 协议实现
- [ ] stdio 传输
- [ ] SSE 传输
- [ ] HTTP 传输
- [ ] MCP 工具包装器
- [ ] MCP 资源读取

**关键文件**:
- `crates/auto-shell/src/mcp/mod.rs`
- `crates/auto-shell/src/mcp/protocol.rs`
- `crates/auto-shell/src/mcp/transport.rs`
- `crates/auto-shell/src/mcp/tool.rs`

## Phase 5: 多 Agent 协作 (3-4周)

**目标**: 实现多 Agent 协作能力

**任务**:
- [ ] CoordinatorAgent
- [ ] WorkerAgent
- [ ] 子 Agent 生命周期管理
- [ ] 结果聚合机制
- [ ] 错误传播与恢复

**关键文件**:
- `crates/auto-shell/src/agent/coordinator.rs`
- `crates/auto-shell/src/agent/worker.rs`

## Phase 6: AutoShell 集成 (2-3周)

**目标**: 将 Agent 集成到 AutoShell REPL

**任务**:
- [ ] `/ask` 命令
- [ ] `/agent` 命令
- [ ] 上下文感知（当前目录、Git 状态）
- [ ] 历史对话记忆
- [ ] 流式输出到终端
- [ ] 配置文件支持

**关键文件**:
- `crates/auto-shell/src/cmd/ask.rs`
- `crates/auto-shell/src/cmd/agent.rs`
- `crates/auto-shell/src/agent/context.rs`

## Phase 7: Process 粒度 Agent (3-4周)

**目标**: 实现 Process 粒度的 Agent 支持

**依赖**: 需要先创建 [Plan 1XX: IPC 基础设施](./1XX-ipc-infrastructure.md)

**任务**:
- [ ] `#[process]` 语法支持
- [ ] 进程启动和管理
- [ ] IPC 传输层实现（stdio、UnixSocket、TCP）
- [ ] 进程间消息序列化/反序列化
- [ ] 进程注册表
- [ ] 进程生命周期管理
- [ ] 沙箱执行环境

**关键文件**:
- `crates/auto-lang/src/parser/process.rs`
- `crates/auto-shell/src/agent/process.rs`
- `crates/auto-shell/src/ipc/mod.rs`
- `crates/auto-shell/src/ipc/transport.rs`
- `crates/auto-shell/src/ipc/stdio.rs`
- `crates/auto-shell/src/ipc/unix.rs`
- `crates/auto-shell/src/ipc/tcp.rs`

## Phase 8: 粒度自动切换 (2-3周)

**目标**: 实现智能的粒度选择机制

**任务**:
- [ ] 粒度决策引擎
- [ ] 资源需求检测
- [ ] 安全需求检测
- [ ] 动态粒度升级/降级
- [ ] 性能监控和优化

**关键文件**:
- `crates/auto-shell/src/agent/granularity.rs`
- `crates/auto-shell/src/agent/monitor.rs`

---

# 参考资料

## 参考仓库

- [Open Agent SDK](D:\github\open-agent-sdk) - 完整的 Agent SDK 实现
- [Claude Code](D:\github\claude-code) - 官方 Claude Code 实现
- [Auto Task/Msg 设计](../design/task-msg.md) - Auto 并发系统设计
- [AI-Native 设计](../design/ai-native.md) - Auto 作为 AI 原生语言的愿景

## API 文档

- [Anthropic Messages API](https://docs.anthropic.com/en/api/messages)
- [OpenAI Chat API](https://platform.openai.com/docs/api-reference/chat)
- [Ollama API](https://github.com/ollama/ollama/blob/main/docs/api.md)

## 相关计划

- [Plan 121: Task/Msg 基础系统](./121-task-msg-system.md)
- [Plan 124: async-future-await](./124-async-future-await.md)
- [Plan 126: 微观并发](./126-phase4-micro-concurrency.md)
- [Plan 128: 调度器消息分发](./128-scheduler-message-dispatch.md)
- [Plan 152: 流式 HTTP 与 SSE 解析](./152-streaming-http-sse.md)
