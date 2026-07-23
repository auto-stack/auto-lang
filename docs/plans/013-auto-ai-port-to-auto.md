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
| `tier.rs` | `tier.at` | ⚠️ 已移植，有缺陷待修（见下） |
| `wire.rs` | `wire.at` | ⚠️ 已移植，有已知限制 |
| `provider.rs` | `provider.at` | ⚠️ 已移植，有边界差异 |
| `loader.rs` | `loader.at` | ⚠️ 已移植，有缺陷待修 |
| `validate.rs` | `validate.at` | ⚠️ 已移植，错误格式略有差异 |
| `lib.rs` | `lib.at` | ⚠️ 已移植，re-export 未落实 |

阶段 1 文件均已移植且 AutoVM 能解析，但逐文件对照 Rust 源码审计发现若干
**真实行为缺陷**（详见[已知不足](#已知不足与补救路径)），需修复后方能称
"完成"。provider.at 通过了全部 6 个对标 Rust `#[test]` 的行为测试；
tier/wire/provider 在 AutoVM 干净运行；loader/validate/lib 因依赖
`use.rust` 桥接类型，AutoVM 不解析（a2r-first）。

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
| `error.rs` | `error.at` | ⚠️ 已移植，缺 `From<reqwest::Error>` 转换 |
| `daemon.rs` | `daemon.at` | ⚠️ 已移植，Windows 检测/stdio 有差异 |
| `lib.rs` | `lib.at` | ⚠️ 已移植（a2r-first），有缺陷待修 |

阶段 2 三文件均已移植（~535 行 Rust → ~430 行 Auto），但审计发现若干
**真实缺陷**（详见[已知不足](#已知不足与补救路径)），需修复。

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

### 文件清单与进度
| Rust | Auto | 状态 |
|---|---|---|
| `error.rs` | `error.at` | ✅ 已移植 |
| `role_def.rs` | `role_def.at` | ✅ 已移植（Role spec） |
| `relay.rs` | `relay.at` | ✅ 已移植（RelayTarget spec） |
| `tool.rs` | `tool.at` | ✅ 已移植（Tool spec + ToolRegistry） |
| `memory.rs` | `memory.at` | ✅ 已移植（含配对感知 trim） |
| `validate.rs` | `validate.at` | ✅ 已移植 |
| `lib.rs` | `lib.at` | ⏳ 待做 |
| `roles.rs` (395) | `roles.at` | ⏳ 待做 |
| `skill.rs` (476) | `skill.at` | ⏳ 待做 |
| `role_def` 之上的 15 个 `builtin_roles/*` | `builtin_roles/*.at` | ⏳ 待做 |
| `config/role_config.rs` (712) | `config/role_config.at` | ⏳ 待做 |
| `workflow.rs` (1181) | `workflow.at` | ⏳ 待做（标记 deprecated） |
| `workflow_validator.rs` (192) | `workflow_validator.at` | ⏳ 待做 |
| `orchestration/*` (5 文件, ~1487) | `orchestration/*.at` | ⏳ 待做 |

**阶段 3 仅完成 6/26 文件**（基础层：错误、Role/Tool spec、memory、validate）。
核心的 ReAct 循环（`agent.rs` 918 行）、workflow 引擎、orchestration
pipeline/driver 均未开始。

## 已知不足与补救路径

> 2026-07-23 逐文件对照 Rust 源码审计得出。分为 **A. 真实移植错误**（Auto
> 代码 bug，可修复）与 **B. VM/a2r 平台限制**（需平台侧改进，当前记档）。

### A. 真实移植错误（必须修复）

| # | 文件 | 缺陷 | Rust 行为 | Auto 当前行为 | 严重度 |
|---|---|---|---|---|---|
| A1 | `tier.at` | `resolve_model_id` 缺 nearest-tier 回退 | 精确匹配→否则最近的"≥tier"→否则最高"<tier" | 仅精确匹配，否则 None | **高**（3/4 Rust 测试会失败） |
| A2 | `tier.at` | 缺 `all_tiers()` 函数 | 返回 5 个 tier 的有序数组 | 完全缺失 | 中（lib.at 还引用了它） |
| A3 | `tier.at` | `ModelDefinition.new` 的 name 默认值 | `name = ""`（空） | `name = id` | 中（影响 display） |
| A4 | `loader.at` | `auth_required` 丢了 `unwrap_or(true)` | 默认 true（缺 key 则 fail-fast） | 赋了 `?bool` 给 `bool`，默认 false | **高（安全相关）** |
| A5 | `loader.at` | `parse_tier` 默认值 | `unwrap_or_default()`=**Min**（注释自相矛盾说 Mid） | 显式 Mid | 中（忠实了注释而非代码） |
| A6 | `client/lib.at` | 缺 `Default` for AiClient | `with_url(daemon_url())`（honors $AAID_URL） | 无 | 中（公开 API 缺失） |
| A7 | `client/lib.at` | 缺 wire 类型 re-export | `pub use ai_config::{Message,...}` | 仅 `use error/daemon` | 中（公开 API 缺失） |
| A8 | `client/lib.at` | HTTP 传输错误未映射为 ClientError | `.map_err(ClientError::from)?` | `.send()` 无错误包装 | 中（错误不可表示） |
| A9 | `client/lib.at` | `complete()` 重复读 body | 错误分支读 text，成功分支用 json | 两个分支都 `body_bytes()` | 低-中（若消费式读取则第二次为空） |
| A10 | `client/error.at` | 缺 `From<reqwest::Error>` | `.into()` / `?` 依赖此转换 | 无 | 中（A8 的根因之一） |
| A11 | `ai-config/lib.at` | re-export 是注释非代码 | `pub use ...` | `//` 注释罗列 | 中（A2 使其中一处还引用了不存在的符号） |
| A12 | 全部文件 | 未移植任何 Rust 测试 | 37+ 测试（ai-config）+ 客户端测试 | 0 个 `#[test]` 移植 | 中（行为靠手测，无回归保护） |

### B. VM/a2r 平台限制（记档，待平台改进）

| # | 限制 | 影响 | 触及文件 |
|---|---|---|---|
| B1 | a2r 译出的 Rust **过不了 cargo check** | enum 缺 Eq/Ord derive、返回 String 字段漏 .clone()、本地类型被误加 crate 前缀、`&iter()` 借用错误 | 全部经 a2r 的文件 |
| B2 | `ContentBlock` 用 tuple 变体 → 丢失 `serde(tag="type")` 线上判别符 | client↔daemon 序列化格式不一致（wire 模块的核心目的） | `wire.at` |
| B3 | AutoVM 不能解构 struct-style enum 变体 | B2 的根因 | `wire.at` |
| B4 | `json.encode/decode[T]` 在 VM 返回垃圾值 / panic | JSON 编解码只能走 a2r | client、agent |
| B5 | VM Map 无 keys()/entries()/iter() | 需并行键表 workaround | loader、validate、tool |
| B6 | 跨文件用户模块在独立 VM 运行不可见 | 只能 a2r 或内联测试 | 全部多文件 crate |
| B7 | `pub const` 不支持 | 用公开函数替代 | daemon |
| B8 | `use <stdlib>` 触发 stdlib 解析错误 | 全局直接调用，不 import | 全部 |
| B9 | `daemon.at` Windows 检测用 `env OS` 而非 `cfg!(windows)` | 标准环境正确，精简环境脆弱 | daemon |
| B10 | `daemon.at` `spawn` 未设 stdio=null | 守护进程可能继承父进程 stdio | daemon |

### 补救优先级

1. **先修 A 类真实错误**（A1/A4 安全与正确性优先）——本计划后续提交。
2. B 类中，**B1（a2r 过 cargo check）** 是验收标准的硬阻塞，需 a2r 侧改进；
   待 auto-lang 的 a2r 修好对应问题后回归验证。
3. B2/B3（ContentBlock 序列化）影响线上互操作，优先级高但受 B3 阻塞——
   需 AutoVM 实现 struct 变体解构后才能用 struct 变体还原 tag。
4. 阶段 3 剩余 20 文件在 A 类修复后再继续，避免在新文件里重复同类错误。
