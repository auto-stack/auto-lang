# 会话生命周期

## 范围

`mcp/session_manager.rs:SessionManager`——AI agent 会话的创建、复用、
重置、删除与源码累积；以及它与底层 `AutovmReplSession` 的关系。

## 原则

- session-per-agent 隔离（architecture.md ADR-02）：每个会话一份独立
  `AutovmReplSession`，会话间状态零共享。
- 与人类 REPL 共用同一核心（plan-265 的 dual-mode VM）：MCP 不另造 VM，
  `VmSession` 只是 `AutovmReplSession` 加生命周期元数据。
- 源码历史是会话的唯一持久状态：`source_history` 同时服务 `auto_snapshot`
  （导出）与 `auto_patch`（重建）。

## 细节

### 数据结构

```rust
pub struct SessionManager { sessions: HashMap<String, VmSession> }
struct VmSession {
    session: AutovmReplSession,   // 持久 VM 核心
    created_at: Instant,          // 元数据，暂未读（#[allow(dead_code)]）
    last_active: Instant,         // GC 依据
    sandbox: bool,                // 仅存不生效（#[allow(dead_code)]）
    source_history: Vec<String>,  // 成功执行的源码段，按序累积
}
```

### 会话 ID

`ses_%04x%04x`：全局 `AtomicU64` 计数器（从 1 起）的低 16 位 +
`SystemTime` 亚秒纳数的低 16 位。进程内唯一，不防猜测——配合 ADR-02，
sandbox 语义缺失意味着任何拿到 ID 的调用方都能操作该会话。

### 生命周期操作

| 操作 | 入口 | 行为 |
|---|---|---|
| 创建 | `create(sandbox)` | 新建 `AutovmReplSession`，记录时间戳，返回 id |
| 取用 | `get(id)` | 返回 `&mut AutovmReplSession`，顺手刷新 `last_active` |
| 重置 | `reset(id)` | 调 `session.reset()` 清空 VM 状态；**不清** `source_history` |
| 删除 | `delete(id)` | 从 map 移除 |
| GC | `cleanup_expired(max_idle)` | retain `last_active` 超龄会话 |
| 累积 | `append_source(id, code)` | `auto_evaluate` 成功后把整段 code 入队 |
| 导出 | `get_source(id)` | `source_history.join("\n\n")` |
| 重建 | `rebuild_with_source(id, src)` | 换全新 `AutovmReplSession`，history 重置为单元素 `[src]` |

### 不变量

- 只有 `auto_evaluate` 成功（`session.run` 返回 `Ok`）才 `append_source`；
  失败代码不进历史——snapshot/patch 永远基于可运行前缀。
- `rebuild_with_source` 之后 history 恰好一段，即当前完整源码；
  patch 与 snapshot 因此读同一份真源（ADR-03）。
- `get`/`append_source`/`reset`/`rebuild_with_source` 都刷新
  `last_active`，活跃会话不会被 GC 误杀。

### GC 现状

`cleanup_expired` 只被 `autovm_daemon` 调用（daemon.rs:214，按
`timeout_secs` 周期触发）。`McpServer::run` 的 stdio 循环没有 GC 调用，
stdio 模式下会话数量只增不减（ADR-02 的负面后果）。

## 显式非目标

- 跨进程/跨 CLI 调用共享会话——那是 `autovm_daemon`（plan-269）的职责，
  本模块只做进程内 map。
- 并发执行：`SessionManager` 无锁，依赖调用方单线程（MCP 主循环 /
  daemon 事件循环）；AutoVM 本身单线程。
- sandbox 策略执行：标志位已留，I/O/网络限制未实现（plan-265 Phase 4，
  未落地）。

> 来源: `crates/auto-lang/src/mcp/session_manager.rs`、`crates/auto-lang/src/autovm_daemon.rs`、`docs/plans/old/265-autovm-mcp-server.md`、`docs/plans/old/269-autovm-daemon-cli.md`
