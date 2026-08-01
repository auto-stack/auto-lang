# Plan 382：枚举变体 `#[from]` 属性 — a2r 原生错误转换生成（G2 非 workaround 方案）

> **Status**: 设计已定稿，未实施（2026-08-01）。
> **来源**: auto-ai Plan 014 遗留语义缺口 G2 —— `AgentError::Client(#[from])` 在 a2r
> 转译下无 `#[from]` 发射，`?` 无法把 `ClientError` 自动转成 `AgentError`（E0277），
> agent.at 用显式 `is Ok/Err` 映射规避。
> **影响仓库**: `auto-lang`（`parser.rs` 变体属性解析 + `ast.rs` EnumItem + `rust.rs`
> 枚举 codegen）→ 转译产物 `auto-ai-agent`（error.at / agent.at）。
> **风险**: 中 —— 触碰枚举语法解析 + codegen，但有现有枚举派生（`#[derive(Debug)]`）
> 与 golden 测试可参照；不引入新依赖。
> **承接**: auto-ai Plan 014（G1/G3 已修复合并，G2 留此计划）；auto-lang Plan 380
> （a2r Rust 互操作补全，struct_init 空参回归 `815c1234` 已修）。

---

## 0. 现状事实（已核实）

- `.at` 源：`error.at` 声明 `pub enum AgentError { ... Client(ClientError) ... }`，
  自带 `message()` 方法（`is self` 分支手写各变体格式串）。
- 生成物：`rust/src/error.rs` 中 AgentError 仅 `#[derive(Debug)]` —— **无 From 转换、
  无 Display、无 `std::error::Error` impl**；ToolError 是 `#[derive(Clone, Debug, ...)]`
  + 手写 `message()`。
- Workaround（agent.at:313-319）：`let resp = is self.client.complete(req).await {
  Ok(r) -> r, Err(e) -> return Err(AgentError.Client(e)) }` —— 替代 `.?`。
- Rust 参考版（auto-ai crates/auto-ai-agent/src/error.rs）：`#[derive(Debug, Error)]`
  + `#[error("...")]` + `Client(#[from] ClientError)` / `Tool(#[from] ToolError)`，
  由 thiserror 生成 Display 与 From。
- 关键机制事实：**`?` 只要求 `impl From<ClientError> for AgentError`**，与 thiserror 无关。
- Auto 语法：枚举变体目前**无属性位**（parser.rs parse_enum_body ~4648 直接把当前
  token 当变体名，`#` 会解析失败）。

## 1. 目标

让 `let resp = self.client.complete(req).await.?` 直接编译通过（消除 E0277），
删除 agent.at 的显式映射 workaround；同时给转译错误类型补齐真实
`std::error::Error` 语义（Display + From），**零新依赖、格式串单一事实源**。

## 2. 方案对比（长期视角）

| 维度 | A：`#[from]` → From impl | B：thiserror 集成 |
|---|---|---|
| `?` 转换（唯一硬需求） | ✅ | ✅ |
| 真实 `Error` + Display（生态互操作） | ❌（需另做） | ✅ |
| 格式串单一事实源 | ✅（无格式串） | ❌ `#[error]` 与 message() 双维护 |
| 依赖 | 无 | 需 thiserror |
| 多后端（TS/Py/JS） | ✅ 语义属性后端无关 | ❌ 纯 Rust 机制 |

**结论**：长期终点是 **B 的语义 + A 的机制** —— `#[from]` 作为后端无关的语义属性，
Rust 后端的 From / Display / Error 全部由 a2r 原生生成，不背 thiserror 依赖、
不留重复格式串。

## 3. 设计

### 3.1 Auto 语法：变体属性

```auto
pub enum AgentError {
    /// Propagated from the Layer-2 LLM client.
    #[from]
    Client(ClientError)
    /// Propagated from a tool invocation.
    #[from]
    Tool(ToolError)
    ToolNotFound(str)
    ...
}
```

- `#[from]` 是第一个被支持的变体属性；属性解析为通用机制（`#[ident]` 列表），
  后续 `#[error("...")]`、`#[transparent]` 等可扩展。
- 语义后端无关：每个后端按各自约定发射转换（Rust → `impl From`；Python → `__class_getitem__`/异常链等按需）。

### 3.2 Rust 后端生成（a2r 原生，非 thiserror）

```rust
#[derive(Debug)]
pub enum AgentError {
    Client(ClientError),
    Tool(ToolError),
    ...
}

// 从 #[from] 变体生成 —— 让 `?` 生效
impl From<ClientError> for AgentError {
    fn from(e: ClientError) -> Self { AgentError::Client(e) }
}
impl From<ToolError> for AgentError {
    fn from(e: ToolError) -> Self { AgentError::Tool(e) }
}

// 从 message() 的 is-self 分支合成 Display + Error —— 单一事实源
impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}
impl std::error::Error for AgentError {}
```

- Display 委托 message()：`.at` 里 message() 的 `f"client error: ${e}"` 等格式串
  是唯一事实源，不引入 `#[error]` 双写。
- 条件发射：仅当枚举变体带 `#[from]`（或存在 message()）时生成对应 impl，避免
  对无转换语义的枚举过度生成。
- **不做**自动推断（如"单载荷变体一律 From"）：`ToolNotFound(str)` /
  `MaxTurnsExceeded(uint)` 不该获得 `impl From<String>/From<u32>`（污染 + 可能撞
  冲突 impl）。显式 `#[from]` 只对 Client/Tool 生效，与参考版语义一致。

### 3.3 实施面

| 位置 | 改动 |
|---|---|
| `crates/auto-lang/src/parser.rs`（parse_enum_body ~4648） | 变体名前接受 `#[ident]`（可多个），存入 EnumItem |
| `crates/auto-lang/src/ast.rs`（EnumItem） | 加 `attrs: Vec<Name>` |
| `crates/auto-lang/src/trans/rust.rs`（枚举 codegen） | `from` 属性 → 发射 `impl From<Payload> for Enum`；存在 message() → 合成 Display + Error；其它属性原样透传 |
| `crates/auto-ai-agent/src/error.at` | `Client(ClientError)` / `Tool(ToolError)` 加 `#[from]` |
| `crates/auto-ai-agent/src/agent.at` | workaround 段塌缩为 `.await.?` |
| auto-ai-agent Cargo.toml | **无改动**（不引入 thiserror） |

### 3.4 验证

1. golden：`test/a2r/` 下新增枚举变体属性用例（`#[from]` → From impl；message()
   → Display/Error 合成），transpiler 单测。
2. 全量 `crates/auto-ai-agent/retranspile.sh check` → **0 错误**。
3. agent.at 中 `?` 生效、显式映射删除后行为不变（错误仍映射到 `AgentError.Client`）。
4. 回归：内置角色 / driver / 其它枚举（ToolError、DriveOutcome、PipelineEvent）
   生成物不受影响。

## 4. 非目标 / 后续

- **不引入 thiserror 依赖**；仅当转译产物需要深度 thiserror 生态特性（`#[source]`
  链、`#[transparent]`）时再评估，且优先以属性形式扩展 a2r 原生生成。
- 其它后端（TS/Python/JS）的 `#[from]` 转换映射不在本计划范围，属性语义先定。
- 泛型错误载荷 / `#[from]` 与 `?` 在 async 链中的组合用法后续按需补用例。

## 5. 实施顺序

1. parser + ast：变体属性解析（`#[ident]` 列表，向后兼容无属性变体）
2. rust.rs：`#[from]` → From impl 发射（`?` 立刻生效，最小闭环）
3. rust.rs：message() → Display + Error 合成（补齐 B 的语义）
4. error.at / agent.at 落地 + workaround 删除
5. 验证（3.4）→ worktree 合回 master（沿用 plan-380/381 worktree 流程）
