# Plan 382：枚举变体 `#[from]` 属性 — a2r 原生错误转换生成（G2 非 workaround 方案）

> **Status**: 已实施并合并 master（2026-08-01，worktree `plan-382-error-from-attr`，
> 提交 `9a5a0f73`，合并 `5198299b`）；全量 retranspile + cargo check 0 错误。
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

---

## 实施记录（2026-08-01，已合并）

- parser.rs `parse_enum_body`：变体名前接受 `#[ident]`（可带括号参数），存入
  `EnumItem.attrs`（新增字段，4 处构造点补齐）。
- rust.rs `enum_decl`：`#[from]` + 单载荷变体 → `impl From<Payload> for Enum`
  （`?` 只需 From，不依赖 thiserror）。
- rust.rs `ext_decl`：已知枚举的 inherent `message()` → 合成
  `impl Display`（委托 message()）+ `impl std::error::Error`；影响 ToolError /
  AgentError / ClientError / ConfigError 4 个错误类型（与参考版 thiserror 对齐）。
- error.at：`AgentError.Client(ClientError)` / `Tool(ToolError)` 加 `#[from]`；
  agent.at 删除显式 `is Ok/Err` 映射，恢复 `.await.?`（E0277 消失）。
- golden：`test/a2r/16_interop/004_variant_from_attr`（.at + .expected.rs，
  test entry `test_16_interop_004_variant_from_attr`）。
- 验证：全量 retranspile → cargo check **0 错误**；生成 diff 仅 agent.rs + error.rs
  （其余逐字节不变）。
- 注：`cargo test --features test-trans` 当前因 master 预存在的 E0063
  （`ui/vm_bridge.rs` 测试里 `AuraWidget` 缺 `exposes` 字段）无法整库编译运行，
  与本次改动无关，golden 通过"自包含文件 transpile_rust 输出 == expected.rs"保证。

---

## 附录 A：a2r golden 套件 69 失败调研与修复（2026-08-01，✅ 已完成）

> E0063 修复（AuraWidget `exposes`，见 `9b286848`）后 `--features test-trans`
> 整库首次可编译运行，暴露 **69 个 a2r golden 失败**（220 通过）。
> 全部失败在本次调研中分类完毕：**12 个是真转译器 bug，57 个是过时 golden**
> （expected.rs 未随转译器演进再生成）。无一与 Plan 382 的 `#[from]`/
> Display/Error 特性相关。

### A.1 真 bug（12 个）：`!T` 同步 Result 被 async 化 ⚠️

**症状**：`fn safe_divide(a int, b int) !int`（Plan 204 定义 `!T` = **同步**
`Result<T, Box<dyn Error>>`，调用用 `.?`）被转译成 `async fn` + 调用点自动加
`.await` → `main` 变 `async fn main()`（无运行时则不可运行）。

**根因**：parser.rs:9538 `!T` 解析为 `Type::Result(Box::new(T))`，与 `~Result`
（async）共用同一 AST 形态；而 `is_async_fn`（rust.rs:8012）与 `type_is_async`
对 `Type::Result(_)` 一律判 async。该判定是 `d269a92d`（plan-013 G2+G3 时代）
为 `~Result` 方法加的，**误伤同步 `!T`**。长期潜伏未被发现：auto-ai 语料不用
`!T`（用 `Result<A,B>` 同步 / `~Result` 异步），且 golden 套件一直被 E0063
挡着跑不起来。

**受影响测试（12）**：`09_option_result/002`、`034`，cookbook
`algorithms/007,011`、`errors/001,004`、`file/005,006,007,009,011,012`。

**修复**（已实施，`c6849779` 合并 `ac27914f`）：`~T` 解析为
`GenericInstance("Future")`（parser Tilde 分支），`~Result` 由 Future 检查独立
捕获 async——因此从 `is_async_fn` / `type_is_async` / spec async 判定（共 5 处）
移除 `Type::Result(_)` 即可：同步 `!T` 恢复 sync（`fn` + 调用点 `?`），`~Result`
不受影响（auto-ai 语料无 `!T`，全量重生成与提交版逐字节一致）。

### A.2 过时 golden（57 个，合法演进）→ ✅ 已重生成

| 子类 | 数量 | 演进内容 | 处理 |
|---|---|---|---|
| 结构体字段 `pub` | ~32 | 转译器现在给 struct 字段发射 `pub` | 重生成 ✅ |
| 语句位 is-match 补 `;` | ~15 | Plan 380：`is` 语句臂以值表达式结尾时补 `;` | 重生成 ✅ |
| 杂项 | ~10 | `HashMap.get(&"a")` 自动借用、`use a2r_std` 路径、`a2r_std::env::set` 桥接、shared var 临时变量、元组结构体 `Counter(10)`（bd4c475e）、`0 as u32`、`char_at`→i32 码点语义（Plan 347，2026-07-23） | 逐条审阅后重生成 ✅ |

**处理**：test runner 在断言失败时已写 `<case>.wrong.rs`（当前输出），
逐条审阅确认合法后 `wrong.rs → expected.rs`，共 **59 个**（含 12 个 async 测试中
叠加了其它 stale 差异的 2 个）。003_field_attrs 已按此修复（`9b286848`）。

### A.3 修复方案与验证 → ✅ 全部完成

1. **修 A.1**（`c6849779`）：`Type::Result` 从 5 处 async 判定移除 → 12 个测试
   恢复 sync（对照各自 expected.rs 即为正确输出）。
2. **重生成 A.2**（同提交）：59 个过时 golden 审阅后重生成。
3. 验证：`cargo test --features test-trans --lib a2r_tests` → **289 通过 / 0 失败**
   （原 220/69）；auto-ai-agent 全量 retranspile 输出与提交版逐字节一致。
4. 备注：该套件此前多年未跑（E0063 编译挡板 + 无 CI），建议纳入常规回归
   （后续计划）。
