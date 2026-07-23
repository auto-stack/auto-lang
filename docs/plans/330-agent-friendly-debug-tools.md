# Plan 330：Coding Agent 可用的 AutoUI 调试工具链

> 原编号 342；2026-07-23 因编号冲突改为 330（原号保留给 342-block-tier-phase-a-package-foundation）

> **For Claude:** 当前 AutoUI 的 MCP 服务（`autoui_snapshot`/`autoui_vtree`/`autoui_state` 等）是为人类开发者设计的可视化工具——需要 GUI 窗口运行、输出 AURA/Atom 格式、依赖视觉截图。Coding Agent（如 Claude）无法有效使用它们：① 缺少 CLI 入口；② 输出不标准（非 JSON）；③ 无 headless 模式；④ 不能诊查 VM 内部状态（heap_objects、globals、call_stack）。本计划设计一套 Agent 友好的 CLI 调试工具。

## 1. 当前 MCP 工具分析

| 工具 | 输出格式 | Agent 可用性 | 问题 |
|------|---------|-------------|------|
| `autoui_snapshot` | AURA text | ❌ | 非标准格式，难以解析 |
| `autoui_vtree` | Atom text | ❌ | 需可视化理解 |
| `autoui_state` | JSON-like | ⚠️ | 部分可用，但需 GUI 运行 |
| `autoui_screenshot` | PNG | ❌ | Agent 无法视觉理解 |
| `autoui_action`（press/type） | 文本 | ❌ | 需要 GUI 交互 |

**核心问题**：所有工具都需要**运行中的 iced GUI**，且输出格式不适合 Agent 解析。

## 2. 目标：Agent 友好的 CLI 调试工具

### 2.1 设计原则

1. **CLI 可调用**：`auto debug <command>`，无需 GUI
2. **JSON 输出**：所有输出为 `--json` 格式
3. **可复现**：相同的输入产生相同的输出
4. **正交**：每个命令做一件事
5. **分层**：从高层（widget state）到低层（VM heap）

### 2.2 CLI 命令设计

```
auto debug state <widget-name>              # 当前 widget model state (JSON)
auto debug state <widget-name> --field=notes # 特定字段
auto debug vtree <widget-name> --json        # VTree 结构（JSON，含节点 id/类型/bbox）
auto debug handler <widget> <event> [args]   # 触发 handler 并返回 state 变化（diff）
auto debug globals                           # VM 全局变量 dump (JSON)
auto debug heap-objects                      # VM heap_objects dump (JSON)
auto debug symbols                           # VM 符号表 dump (JSON)
auto debug eval <widget> <expr>              # 在 widget 上下文中求值表达式
```

### 2.3 示例用法

```bash
# Agent 排查 015-notes 的 Delete 问题
$ auto debug state App --field=notes
{"notes": [{"id":0,"title":"Welcome","body":"...","time":"Just now"}, ...], "length":3}

$ auto debug handler App DeleteNote
# 触发 .DeleteNote handler
{"before": {"notes": 3, "active_index": 0}, "after": {"notes": 2, "active_index": 0}}

$ auto debug globals
{"notes": "<ListData<Value> id=4000000 len=2>", "nextid": 4}

$ auto debug eval App ".notes.len()"
2
```

## 3. 实现方案

### Phase 1 — CLI 入口（最小可用）

**文件**：`crates/auto/src/debug.rs`（新增）

在 `auto` CLI 中添加 `debug` 子命令：

```rust
#[derive(Subcommand)]
enum DebugCommands {
    /// Dump widget model state
    State {
        widget_name: String,
        #[arg(long)]
        field: Option<String>,
    },
    /// Dump VM globals
    Globals,
    /// Dump VM symbols
    Symbols,
    /// Run a handler and show state diff
    Handler {
        widget_name: String,
        event: String,
        #[arg(long)]
        args: Vec<String>,
    },
}
```

每个命令编译并运行 widget（单帧），提取数据，打印 JSON。

### Phase 2 — VM 内省工具

**文件**：`crates/auto-lang/src/vm/introspection.rs`（新增）

新增 VM 方法：

```rust
impl AutoVM {
    /// Dump all globals as JSON
    pub fn dump_globals_json(&self) -> serde_json::Value { ... }

    /// Dump heap_objects summary (id, type_tag, size)
    pub fn dump_heap_summary(&self) -> serde_json::Value { ... }

    /// Dump a specific heap object by id
    pub fn dump_heap_object(&self, id: u64) -> serde_json::Value { ... }

    /// Dump symbols (exports) as JSON
    pub fn dump_symbols(&self) -> serde_json::Value { ... }
}
```

### Phase 3 — 嵌入 VM 诊断到 call_fn_by_name

**文件**：`crates/auto-lang/src/vm/engine.rs`

在 `call_fn_by_name` 中添加 **可选诊断**（由环境变量控制）：

```rust
if std::env::var("AUTO_VM_TRACE").is_ok() {
    eprintln!("[VM_TRACE] fn={} args={:?} steps={} depth={}",
        fn_name, n_args, steps, task.call_stack.len());
}
```

当 handler 执行超过阈值时自动报告：
```rust
if steps > 10_000 && call_depth > 50 {
    eprintln!("[VM_WARN] Possible recursion: fn={} depth={}", fn_name, call_depth);
}
```

### Phase 4 — MCP 增强（可选）

将 CLI `debug` 命令暴露为 MCP 工具，供外部 Agent 调用：

```
// MCP 工具
"name": "auto_cli",
"description": "Run an auto CLI command and return the output",
"arguments": {"command": "debug state App --field=notes"}
```

## 4. 验收标准

- [ ] `auto debug state App --json` 返回 015-notes 的 model state（含 notes 列表）
- [ ] `auto debug globals --json` 返回全局变量内容
- [ ] `auto debug handler App DeleteNote` 触发 handler 并返回 state diff
- [ ] `AUTO_VM_TRACE=1 auto run -r vm` 输出 VM 执行追踪
- [ ] Agent 可以用这些工具独立排查 015-notes 问题（无需人工点击 UI）

## 5. 与现有工具的互补

| 现有工具 | 新 CLI 工具 | Agent 场景 |
|----------|------------|-----------|
| autoui_snapshot（GUI） | `debug state` | 查 model 数据 |
| autoui_vtree（GUI） | `debug vtree --json` | 查渲染树结构 |
| 手动点击 UI | `debug handler` | 触发事件看效果 |
| 无 | `debug globals` | 查 VM 全局变量 |
| 无 | `debug symbols` | 查符号表冲突 |
| 无 | `AUTO_VM_TRACE` | 追踪 VM 执行 |

## 6. 与 Plan 341（调试方法论）的关系

Plan 341 总结的"最有效的 3 个诊断"（VM 测试框架、逐层 eprintln、type_tag）——本计划把这三个诊断**工具化、CLI 化、JSON 化**，让 Agent 可以自动使用。
