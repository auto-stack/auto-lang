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
| `provider.rs` | `provider.at` | ✅ 已完成 |
| `loader.rs` | `loader.at` | ✅ 已完成（桥接 auto_atom/auto_val） |
| `validate.rs` | `validate.at` | ✅ 已完成 |
| `lib.rs` | `lib.at` | ✅ 已完成 |

**阶段 1 全部完成**。验收：provider.at 通过全部 6 个对标 Rust `#[test]`
的行为测试；tier/wire/provider 在 AutoVM 干净运行；loader/validate/lib 因
依赖 `use.rust` 桥接类型，AutoVM 不解析（a2r-first，见下），但 a2r 能翻译
出结构正确的 Rust。

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
   - `use.rust` 导入的本地类型被误加 crate 前缀（如本地 `TierRouting`
     被译成 `auto_atom::TierRouting`）；
   - `&iter()` 借用迭代器译法不对（`for x in &node.kids_iter()` 应为
     `for x in node.kids_iter()`）；
   - 每次有 `unbalanced parentheses (depth: 1)` 假警告（输出实际合法）。
   验收以 **AutoVM 运行（纯 Auto 文件）+ 行为冒烟测试 + a2r 结构正确** 为准
   （wire.at / provider.at 已通过全部对标 Rust `#[test]` 的用例）。

5. **`routes` / `route` 是保留关键字，不能做字段名**
   Auto 把 `route`、`routes` 保留给 routing/navigation。用作字段名时
   lexer 把它当关键字 token，报 `Expected term, got Routes`。loader.at 的
   `TierRouting.routes` 改名为 `entries`。**移植前先查保留字表。**

6. **Auto VM 的 Map 没有 iteration API**
   `Map<K,V>` / `HashMap` 只有 `set/get/contains/remove/size`，没有
   `keys()/values()/entries()/iter()`，且 `for k,v in map` 静默产出 0 项。
   凡需遍历 map 的地方（validate、loader 的 providers 表），改用一个
   **并行的 `List<str>` 键表**（如 `provider_names`、`tier_names`），
   `for name in keys { map.get(name) }`。这与 Rust 原版的
   `for (_, p) in &map` 等价，但要多维护一个键表字段。

7. **`use.rust` 桥接的文件是 a2r-first（AutoVM 不解析）**
   loader.at 用 `dep auto_atom` + `use.rust auto_atom` 桥接 Rust 的
   AtomParser/Node/Kid/Value。这些类型对纯 AutoVM 解释器是未知的，所以
   loader/validate/lib 在 AutoVM 里跑不起来（报
   `Unknown enum variant: Atom.Node` 等）。它们的价值在 a2r→Rust 路径：
   翻译后的代码在 cargo 下行为与原版一致。**未来 Auto 自举出原生 Atom
   解析器后，去掉 `use.rust` 行、换成原生调用即可，公开 API 不变。**
   （决策来源：用户明确选择"先用桥接模式，记录问题，未来自举后替换"。）

### 验证命令

```bash
# AutoVM 运行（应无错，打印 0 或无输出）
./target/release/auto.exe crates/ai-config/src/<file>.at

# 翻译为 Rust（生成 <file>.a2r.rs）
./target/release/auto.exe trans --path crates/ai-config/src/<file>.at rust
```

## 阶段 2：auto-ai-client

### 文件清单
| Rust | Auto | 状态 |
|---|---|---|
| `error.rs` | `error.at` | ✅ 已完成 |
| `daemon.rs` | `daemon.at` | ✅ 已完成 |
| `lib.rs` | `lib.at` | ✅ 已完成（a2r-first） |

**阶段 2 全部完成**。3 个文件，~535 行 Rust → ~430 行 Auto。

`lib.at` 是 **a2r-first**：HTTP + JSON + SSE 流式依赖 `reqwest`/`serde_json`/
`futures`，Auto VM 的 `json.encode[T]`/`json.decode[T]` 泛型分发在纯解释器
里有 bug（返回指针值），所以 `complete`/`complete_stream` 的真实行为走 a2r→
Rust 路径。已验证可翻译，且 `fn ... ~Result<T,E>` 正确译为
`pub async fn ... -> Result<T,E>`（plan 的 async 决策落地）。

纯 VM 可验证的部分（均已通过行为测试）：
- error.at：3 个错误变体的 message 格式化。
- daemon.at：daemon_url() 默认值 + `$AAID_URL` 覆盖 + default_daemon_url()。
- lib.at 的 SseBuffer：跨 chunk 重组、`[DONE]` 过滤、finish() 尾部 flush。

### 阶段 2 新踩坑（追加到上面的清单）

8. **`pub const` 不支持**
   `pub const X str = "..."` 报 `Expected infix operator, got const`。
   模块级 `const X = ...`（无 pub、无类型注解）可以。需要公开的常量改用
   **公开函数**返回（如 `default_daemon_url()`）。

9. **`json.encode[T]` / `json.decode[T]` 在 VM 里不可靠**
   泛型 JSON 编解码在纯 VM 下返回指针/ID 值（如 `"4000000"`）而非真正
   序列化结果，且 `json.decode[T](...)` 会 panic（"Dynamic call not
   supported"）。依赖 JSON 编解码的文件标为 **a2r-first**（翻译后由
   serde_json 提供真实行为）。纯 VM 里只能用 `json.parse(s) → JsonValue`
   + 手动字段提取。

10. **`use <stdlib_module>` 会触发 stdlib 解析错误**
    `use http` / `use fs` / `use json` 等会加载对应 stdlib `.at` 文件，
    其中某些声明（如 `pub fn JsonValue.as_int(self JsonValue) int;`）让
    VM 解析器报 `Expected term, got Newline`。但这些模块的函数**全局可
    用，无需 import**（`http.request(...)`、`fs.exists(...)`、
    `json.parse(...)` 直接调）。**规则：不要写 `use <stdlib>`，直接用。**

11. **跨文件用户模块在独立 VM 运行里不可见**
    `use daemon` / `use loader` 等用户 crate 内的模块，在单独 `auto a.at`
    运行时报 `Module not found` / `Undefined variable`。这是 VM 的模块
    解析限制（只认注册过的 stdlib/auto 模块），不影响 a2r 翻译（译为
    `use crate::daemon`）。行为测试时需把被测模块的类型/函数内联到同一
    文件，或仅测不依赖跨文件的部分。

## 阶段 3：auto-ai-agent
（阶段 2 验收后展开）
