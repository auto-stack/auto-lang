# Plan 013: auto-ai → Auto 语言移植

> **状态**：已批准，实施中
> **仓库**：auto-lang（Auto 代码）+ auto-ai（Rust 原版参考）
> **前置**：Auto Language Spec v0.2, auto-lang-creator skill

## 目标

将 auto-ai 的 3 个核心 Rust crate 用 Auto 语言复刻，放到 `auto-lang/crates/` 下。
Auto 代码必须能通过 AutoVM 运行，也能通过 a2r 翻译成 Rust，行为与原版一致。

## 移植范围

| Rust crate | Auto crate | 代码量 | 可移植性 |
|---|---|---|---|
| `ai-config` | `crates/ai-config/src/*.at` | ~1220 行 | 高 |
| `auto-ai-client` | `crates/auto-ai-client/src/*.at` | ~478 行 | 中 |
| `auto-ai-agent` | `crates/auto-ai-agent/src/*.at` | ~6898 行 | 中 |

不移植：`auto-ai-daemon`（axum）、`auto-ai-cli`（ratatui TUI）。

## 关键架构决策

1. **spec 即 dyn Trait**：Auto 的 `spec` 自动做动态分发，不需要 `dyn` 关键字。
   `Arc<dyn Client>` → `Arc(Client)`。
2. **serde**：用 stdlib 的 `json.encode[T]` / `json.decode[T]` 或 `use.rust serde_json` 桥接。
3. **async**：`async fn` → `fn ... ~T`。
4. **.at 解析**：桥接 `auto_atom`（已有 Auto 生态）。

## 阶段 1：ai-config

### 文件清单
| Rust | Auto | 状态 |
|---|---|---|
| `tier.rs` | `tier.at` | ✅ 已完成 |
| `wire.rs` | `wire.at` | ✅ 已完成 |
| `provider.rs` | `provider.at` | 待做 |
| `loader.rs` | `loader.at` | 待做 |
| `validate.rs` | `validate.at` | 待做 |
| `lib.rs` | `lib.at` | 待做 |

### 验收标准
- AutoVM 能运行 parse_name / resolve_key / resolve_model_id
- a2r 能翻译回 Rust 通过 cargo check

### 移植踩坑记录（wire.at 阶段发现，后续文件必读）

经实际验证（auto.exe v0.4.0 + a2r），以下为 AutoVM/a2r 的现实约束，非
spec 文档所载，移植时**必须遵守**：

1. **构造函数尾表达式必须用 `return`**
   在 `static fn` / 普通函数里，把 `Type(...)` 或 `Variant(...)` 作为函数
   最后一条**不加 return 的尾表达式**时，AutoVM 报诡异的
   `field type mismatch`（"field `id` expects type `str`, found `str`"——
   类型相同却报不匹配）。`tier.at` 原本因此坏掉，已修：所有返回构造体的
   函数体改为 `return Type(...)`。**规则：凡函数返回一个构造体调用，一律
   显式 `return`。**

2. **带字段的 enum 变体用 tuple 变体 + 位置解构**
   AutoVM 尚未实现 struct-style 变体的 `is` 解构（codegen panic:
   `not implemented: Expression StructPattern`）。`wire.rs` 的
   `ContentBlock::{Text{text}, ToolUse{id,name,input}, ToolResult{...}}`
   改写为元组变体 `Text(str)` / `ToolUse(str,str,JsonValue)` /
   `ToolResult(str,str,bool)`，用 `ContentBlock.Text(t)` 构造、
   `ContentBlock.Text(t) ->` 位置解构。字段顺序对齐 Rust struct 字段顺序。

3. **不要写 `use json`**
   stdlib 的 `json.at` 含 `pub fn JsonValue.as_int(self JsonValue) int;`
   这类声明，VM 解析时报 `Expected term, got Newline`。但 `JsonValue`
   类型与 `json.parse(...)` 函数**全局可用，无需 import**。直接用即可。

4. **a2r → Rust 的已知差距（非移植错误，属 a2r 待完善）**
   `auto trans ... rust` 生成的代码有以下问题，当前不阻塞 Auto 侧验收，
   但"通过 cargo check"这一条尚达不到：
   - enum 缺 `Eq`/`Ord` derive，却被用到带 `Eq,Ord` 的 struct 上；
   - 返回 `&self` 的 String 字段时漏 `.clone()`（E0507）；
   - 每次有 `unbalanced parentheses (depth: 1)` 假警告（输出实际合法）。
   验收以 **AutoVM 运行 + 行为冒烟测试** 为准（wire.at 已通过全部 8 个
   对标 Rust `#[test]` 的用例）。

### 验证命令

```bash
# AutoVM 运行（应无错，打印 0 或无输出）
./target/release/auto.exe crates/ai-config/src/<file>.at

# 翻译为 Rust（生成 <file>.a2r.rs）
./target/release/auto.exe trans --path crates/ai-config/src/<file>.at rust
```

## 阶段 2：auto-ai-client
（阶段 1 验收后展开）

## 阶段 3：auto-ai-agent
（阶段 2 验收后展开）
