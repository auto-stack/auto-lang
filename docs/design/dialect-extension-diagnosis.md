# 语法扩展 / 方言（Dialect）体系诊断与改进方案

> 状态：诊断 + 改进方案，待评审
> 范围：`crates/auto-lang` 的 Parser / AST / AURA / VM / Transpiler
> 核心结论：**当前并没有"两套独立解析器"，但存在可清晰定位的结构性问题。**
> 改进按"三轴"分别处理，避免把不同性质的问题混入"方言"一个概念。

---

## 0. 背景与误解澄清

最初提出的问题假设："AutoUI 的解析器（AURA）与 Auto 本身的解析器是独立两套，AST 分别是 `AuraStmt/AuraExpr` 与 `Stmt/Expr`，导致大量重复。"

**经代码核查，该假设不成立。** 真实情况是：

```
.at 源码
   │  （唯一的 Lexer + Parser）
   ▼
┌─────────────────────────────────────────────┐
│ 基础 Auto Parser（parser.rs，13214 行）       │
│   - is_contextual_keyword() 按场景启用关键字   │
│   - widget/model/view/on/msg 均在此解析        │
│   - 产出统一的 Stmt / Expr（基础 AST）         │
└─────────────────────────────────────────────┘
   │
   ▼  统一的基础 AST（如 Stmt::WidgetDecl）
┌─────────────────────────────────────────────┐
│ AURA Extractor（aura/extract.rs）            │
│   - extract_widget_from_decl(&WidgetDecl)     │  ← AST → IR 转换
│   - extract_expr/extract_stmt（镜像式，见 §1.3）│  ← 不重新解析文本
└─────────────────────────────────────────────┘
   │
   ▼  AuraWidget IR（供代码生成与 VM 渲染）
```

证据要点：
- `aura/extract.rs:13` 直接 `use crate::ast::{Expr, Stmt, ...}`，输入是**已解析好的基础 AST**，不是源文本。
- 整个 `aura/` 与 `a2ui/` 目录里没有任何 `Lexer` / `tokenize` / `TokenKind` 的使用——AURA 不做词法分析。
- `AuraExpr`（`aura/types.rs:580`）、`AuraStmt`（`aura/types.rs:723`）是**中间表示（IR）类型**，不是语法树。

所以 `widget/model/view/on/msg` 的解析早已是统一的。本文聚焦的是**另外的真实问题**，并给出一套分轴处理的改进方案。

---

## 1. 真实问题诊断

### 1.1 方言派发逻辑硬编码、散落，且没有正式的扩展抽象

"方言"的载体是三处零散的硬编码：

| 位置 | 内容 | 缺陷 |
|---|---|---|
| `session.rs:27-38` | `enum Scenario { Core, UI, Shell }` | `Shell` 形同虚设，parser 从不查询；新增方言要改基础枚举 |
| `parser.rs:414-419` | `is_contextual_keyword()` 把 5 个 UI 关键字写死 | 每加一个方言都要改这个函数 |
| `parser.rs:3789-3810` | `parse_stmt_inner` 的 `TokenKind::Ident` 分支里 match 关键字字符串 | 派发表硬编码，无法在 crate 外扩展 |

**休眠的基础设施**：`Parser` 已经有一个看起来是为扩展设计的字段——
```rust
// parser.rs:145-147
pub trait BlockParser {
    fn parse(&self, parser: &mut Parser) -> AutoResult<Body>;
}
// parser.rs:186
pub special_blocks: HashMap<AutoStr, Box<dyn BlockParser>>;
```
但：
- `add_special_block()`（`parser.rs:297`）**全局零调用**——没有任何代码注册过 block。
- 唯一派发点（`parser.rs:9477`）只对**节点实例化体** `Name(args){...}` 生效，**不参与语句级派发**。
- `special_block()`（`parser.rs:9852`）里 `remove`+`insert` 的借检查舞蹈说明这套机制被设计出来后从未被实际使用打磨过。

结论：扩展点是存在的，但既不完整（只覆盖节点体）、也从未启用。方言派发目前**完全靠硬编码**。

### 1.2 基础 `Stmt` 枚举随方言膨胀

`Stmt`（`ast.rs:183`）已有约 35 个变体，其中方言相关的：
- UI：`WidgetDecl`、`MsgDecl`、`ModelBlock`、`ViewBlock`
- Godot：`SceneDecl`
- 通用但偏运行时的：`OnEvents`

后果：**每个下游消费者（typeck / trans / vm / interpreter）都被迫处理这些变体**，即便运行在 `Core` 场景，这些 UI 变体永远不会出现。随着方言增多（Shell 真正落地、未来更多后端），这一压力只会加剧。

这正是"Rust 的 Enum 是静态的、没法动态扩展"在工程上真正的代价——不是出在 parser，而是出在**所有 AST 消费者**。

### 1.3 AURA IR 镜像层冗余且易漂移

`AuraExpr` / `AuraStmt` 手工复刻了基础 `Expr` / `Stmt` 的一个**很小的子集**：

- `extract_expr`（`extract.rs:96`）覆盖：字面量、`Bina`/`Unary`、`Dot`/`Call`、`Array`/`Object`、`Closure`、`FStr`、`Index`、单分支 `If`；其余一律 `Err(UnsupportedExpr)`（`:350`）。
- `extract_stmt`（`extract.rs:388`）**只认两种形状**：`Store`（→`Assign`）和 `Expr(Call(Dot(Ident,_)))`（→`MethodCall`）。其余一律 `Err(UnsupportedStmt)`（`:425`）。

风险：
- 基础 `Expr` 演进（加变体、改形状）时，`AuraExpr` **不会自动跟进**，编译器也不报警（`_ =>` 通配吃掉一切）。这是"两套语义由 AI 分别维护、悄悄分叉"的温床。
- §3 将证明：**这套镜像层在两条执行路径上都没有被真正消费**——VM 用 `LogicPayload::AstStmts`（基础 `Stmt`），transpiler 用 `trans/`。它是纯粹的冗余中间层。

附带卫生问题：`ExtractError::InvalidStateRef` 与 `MissingField`（`extract.rs:52-67`）声明了却**从未构造**——dead variant。

---

## 2. 派发现状中的隐藏不一致（重构前必须知晓）

探查中发现的、相关的小坑：

1. **`view` 的 token 不一致。** `TokenKind::View`（`token.rs:111`）是 **Core 语言的参数模式关键字**（`fn foo(view x int)`），lexer 对源码里的 `view` **永远**产出 `TokenKind::View`（`token.rs:366`）。而 `parse_stmt_inner:3802` 处对顶层 `view { ... }` 块的派发位于 `TokenKind::Ident` 分支内——**该分支对 `view` 永远不可达**。当前顶层 `view` 块只能在 `widget` 体内通过 `parse_widget_decl` 的 `.text` 字符串匹配（`:10056`）工作。**顶层 `view {}` 在 UI 场景实际是个 bug**。
2. **`on` 的双路径。** 行首 `on` → `TokenKind::On`（`lexer.rs:731`，always-on，命中 `parse_stmt_inner:3816`）；行中 `on` → `TokenKind::Ident`（命中 UI-gated 字符串派发 `:3803`）。两条路径并存，语义易混。
3. **`scene`（Godot）绕过一切场景判断。** `looks_like_scene_decl()` 在 `TokenKind::Ident` 分支里**先于** `is_contextual_keyword` 判断（`:3789`），任何场景下都生效。方言门控不统一的又一例证。
4. **场景选择完全靠调用方程序性设置**，无 CLI flag、无文件扩展映射。`.at` 一统天下，UI 模式仅由 `ui_gen/api.rs`、`auto-man/{vue,rust_ui}.rs` 等生成器调用 `CompilerSession::ui()` 触发。

---

## 3. 执行路径分析：UI VM 与原生 AutoVM 的关系

> 本节回应"前端 VM 模式下的解析逻辑，与原生 AutoVM 直接解析 Auto 代码的流程，是独立的还是一致的"。

### 3.1 结论：UI VM 路径与原生路径**共用**同一套 Codegen + AutoVM

`AuraWidget` IR 内部直接携带**基础 AST**（`aura/types.rs:562`）：

```rust
pub enum LogicPayload {
    AstStmts(Vec<crate::ast::Stmt>),   // ← 基础 Stmt，不是 AuraStmt
    ...
}
```

因此 UI VM 模式的完整链路是：

```
.at UI 源码 → Parser(UI scenario) → Code/Stmt
   → extract_widget_from_decl → AuraWidget（handler 体仍是基础 Stmt）
   → synthesize_widget_module:
       1. 改写 .count → __state.count
       2. 包装成 fn handler_<W>_<E>(__state: <W>_State)
       3. 加 type <W>_State { ... }
       4. ★ 调用与原生同一个 vm::codegen::Codegen ★
   → Module(ABC) → Linker → VirtualFlash → AutoVM（同一个）
   → VmBridge::call_handler 派发；AuraViewBuilder::read_state 取值渲染
```

差异仅在于 handler 喂给 `Codegen` 之前多了一步"合成"（`ui/handler_codegen.rs`），之后的编译/链接/执行全部复用原生 VM。

| 维度 | 原生非 UI 路径 | UI VM 路径 |
|---|---|---|
| Parser / AST | 共用（`Code`/`Stmt`/`Expr`） | 共用 |
| 字节码编译器 | `vm::codegen::Codegen` | **同一个** |
| 字节码格式 | ABC（`OpCode`） | **同一种** |
| 链接器 / 执行引擎 | `Linker` / `AutoVM` | **同一个** |
| 唯一差异 | `Stmt` 直接编译 | 先把 `on{}` 体改写+包装成 `fn handler_*(__state)`，再编译 |

关键证据：
- `ui/handler_codegen.rs:345-352` `synthesize_widget_module` 内部 `Codegen::new()` —— 即原生路径的编译器。
- `ui/vm_bridge.rs:170-194` `Linker::link()` + `AutoVM::new()` —— 与原生 `lib.rs:681-740` 相同。
- `ui/vm_bridge.rs:1-33` 文档注释明确：handlers 由 "genuine VM `Codegen` (the same compiler the non-UI `run()` path uses)" 编译。
- `vm/codegen.rs` 中 `compile_stmt` 对 `Stmt::WidgetDecl/ModelBlock/ViewBlock/MsgDecl` **零 match 臂**——原生路径遇 UI 节点会静默跳过。

### 3.2 反差：transpiler 路径才是真正独立的

从同一个 `AuraWidget` IR 出发有两条分叉：

```
AuraWidget ─┬─► UI VM：synthesize → Codegen → AutoVM（共用原生 VM）   ← 一致
            │
            └─► Transpiler：VueGenerator/JetGenerator/ArkGenerator/RustGenerator
                 handler 体经 ts_adapter → trans::typescript
                 产出 JS/Kotlin/ArkTS/Rust 源码                          ← 独立（无 VM）
```

`ui_gen/{vue,jet,ark,rust}.rs` 各自实现 `BackendGenerator::generate(&AuraWidget) -> String`，**完全绕过 VM**。这条路径里 handler 体走 `trans/` 下的通用 AST 转译器（如 `TypeScriptTrans`），不经过 `Codegen`/`AutoVM`。

---

## 4. 三轴分析：把"统一"拆成三个正交问题

> 本节是改进方案的**概念基础**。它修正了一个常见误区：把所有"统一"诉求都塞进"方言"这一个概念里。
> 实际上只有**轴 A 是真正的方言**；轴 B 是消费方/目标；轴 C 是数据形态。混为一谈会让"方言"被过度使用。

| 轴 | 含义 | 例子 | 解决手段 |
|---|---|---|---|
| **A. 语法方言** | 这段代码**容许出现什么**（关键字、语句类型） | UI 的 `widget/view/on`；Core 的 `fn/task` | Dialect trait（解析层） |
| **B. 执行/转译后端** | 这段代码**到哪里去** | VM 执行 vs 转译成 JS/Rust/C | Codegen+AutoVM vs `trans/` |
| **C. 程序形态** | 这段代码**整体是什么形状** | 顺序脚本（`Code`）vs 响应式组件（widget） | 是否需要"结构化容器"IR |

### 4.1 轴 B（VM 后端）：已经统一 90%

§3 已证明：UI handler 经 synthesize 后喂给的就是**原生那个 `Codegen`**，编译/链接/执行全部走原生 `AutoVM`。所以"VM 模式下 AutoScript 和 AutoUI 用同一套 VM 逻辑"——**这已经是现状**。fn/闭包/控制流等共享特性没有任何重复处理。

唯一还能再统一的，是 AURA IR 在 VM 路径上的"中转"角色：当前 `WidgetDecl → AuraWidget → synthesize → Codegen`，而 AuraWidget 在这条路上几乎是个透传层（handler 体本来就是基础 `Stmt`）。方言体系可让它变成 `WidgetDecl →（UiDialect 负责 synthesize）→ Codegen`，即 **VM 路径绕过 AuraWidget**。

### 4.2 轴 B（转译后端）：统一发生在叶子层，不在容器层

要分两层看：

**叶子层（单个语句/表达式 → 目标语言）：已经统一。**
- 脚本转译器 `a2r/a2c/a2py/a2ts` 住在 `trans/`（`rust.rs`/`c.rs`/`python.rs`/`typescript.rs`）。
- UI 转译器 a2vue 的 handler 体，是通过 `ui_gen/ts_adapter.rs` **包装同一个 `trans::typescript::TypeScriptTrans`** 来转译的。
- "把 `count = count + 1` 翻译成目标语言"这件事，UI 和脚本用的是**同一套 `trans/` 机器**，没有重复。✅

**容器层（整个程序的结构 → 目标框架的骨架）：不应强统一。**
- `a2r` 输入是 `Code`（顺序语句），输出 Rust 模块——做"逐语句翻译 + 模块组装"。
- `a2vue` 输入是 `AuraWidget`（props/state/view 树/handlers），输出 Vue SFC——做"view 树→template、model→reactive、on→event handler"的**框架映射**。

这两者不是同一种工作。SFC 的 `<template>`、Compose 的 `@Composable` 是"组件"才有的概念，顺序脚本里没有。强行用一个"方言"统一，等于让脚本转译器去理解 widget 结构、让 widget 转译器去理解 main 函数——只会制造耦合。

> **结论：a2vue 和 a2r 不是"同一种东西的两个方言"，而是"针对两种不同程序形态的两个消费方"。** 真正可统一、且已经统一的，是它们共用的叶子翻译器 `trans/`。

### 4.3 轴 C（程序形态）：AURA IR 合理存在的部分

要把"AURA IR"拆成两半，区别对待：

| AURA IR 的内容 | 评价 |
|---|---|
| **组件结构**：`AuraWidget{props, state, view树, handlers, messages}` | ✅ **合理且必要**。widget 是脚本里没有的形态，需要结构化容器。对 transpiler 和 VM renderer（`AuraViewBuilder`）都有用。 |
| **表达式/语句镜像**：`AuraExpr`/`AuraStmt`/`AuraBinOp`/... | ❌ **冗余且有害**。VM 不用（走 `LogicPayload::AstStmts`），transpiler 不用（走 `trans/`），它只是历史遗留的平行语义层。 |

所以"消灭 AURA IR"这个口号应修正为：**消灭 `AuraExpr`/`AuraStmt` 镜像，保留 `AuraWidget` 组件结构**。

### 4.4 综合定位

> **方言（Dialect）只解决"解析时允许什么语法"（轴 A）。VM 共享靠复用 `Codegen`+`AutoVM`（轴 B，已实现）。转译器共享靠复用 `trans/` 叶子层（轴 B，已实现）。组件结构保留 `AuraWidget`（轴 C，正当）。唯一要删的，是夹在中间、谁都绕着走的 `AuraExpr`/`AuraStmt` 镜像。**

---

## 5. 设计目标

1. **可扩展（轴 A）**：新增一种方言（关键字集合 + 语句/块解析）时，不必修改 `is_contextual_keyword`、`parse_stmt_inner`、`Scenario` 枚举等核心代码——在自己的模块/crate 里实现一个 trait 即可注册。
2. **不爆炸（轴 A/C 边界）**：方言专属节点仍可作为基础 `Stmt` 变体存在（它们确实是语法），但解析派发不再硬编码。本方案**不**急于引入 `Stmt::Ext`（路线 B），先观察一个版本周期。
3. **不漂移（轴 C）**：IR 层不应手工镜像基础 `Expr`/`Stmt` 的语义；`AuraWidget` 只承载组件结构，handler 直接持有基础 `Stmt`，子集约束通过校验器实现。
4. **VM 路径去中转（轴 B）**：UI handler 经方言 synthesize 后直接进共享 `Codegen`，绕过 AuraWidget 中转（AuraWidget 仅服务于 transpiler + renderer）。
5. **渐进可迁移**：现有 UI 解析代码（`parse_widget_decl` 等约 1500 行）应能逐步迁入新机制，迁移期间二者可共存。
6. **行为不变**：迁移完成后，现有 `.at` UI 源码的解析结果与今天完全一致（§2 列举的不一致按"修 bug"单独决策，不在本体系里偷偷改语义）。

---

## 6. 改进方案

按三轴分别给出方案。三者**正交、可独立推进**，但建议按 §7 的顺序落地。

### 6.1【轴 A】`Dialect` trait —— 正式化方言派发

**核心思想**：把"哪些关键字归我管 + 看到关键字怎么解析"抽象成 trait，按场景在 `Parser` 上注册。

```rust
// 新增：crates/auto-lang/src/dialect.rs

use crate::ast::Stmt;
use crate::parser::Parser;
use crate::session::CompilerSession;
use crate::error::AutoResult;

/// 一个方言：在某个场景下生效的一组关键字与语句解析器。
///
/// 设计原则：
/// - 方言只管"解析时允许什么语法"（轴 A），不管执行/转译（轴 B）或程序形态（轴 C）。
/// - 方言解析出的节点仍是基础 `Stmt` 的合法变体（如 `Stmt::WidgetDecl`），
///   保证下游消费者（typeck/trans/vm）类型签名不变。
pub trait Dialect: Send + Sync {
    /// 该方言是否在当前 session 下生效。
    fn matches(&self, session: &CompilerSession) -> bool;

    /// 该方言接管的语句起始关键字（仅作为语句起始、且在语句位置时被查询）。
    /// 返回的关键字在所属场景下应被视为"上下文关键字"而非普通标识符。
    fn keywords(&self) -> &'static [&'static str];

    /// 命中某个关键字时调用。
    /// - 返回 `Ok(Some(stmt))`：本方言已处理，产出 stmt。
    /// - 返回 `Ok(None)`：关键字虽在列表里但本次不归我管（让下一个方言/默认路径处理）。
    /// - 返回 `Err(_)`：报错。
    fn try_parse_stmt(&self, parser: &mut Parser, keyword: &str)
        -> AutoResult<Option<Stmt>>;
}
```

`Parser` 增加方言表与派发入口：

```rust
// parser.rs —— Parser 结构新增字段
pub dialects: Vec<Box<dyn Dialect>>,   // 构造时按 session 装配，见 build_dialects()

/// 尝试用已注册的方言解析当前语句起始标识符。
/// 在 parse_stmt_inner 的 TokenKind::Ident 分支首位置调用。
fn try_dialect_stmt(&mut self, ident: &str) -> AutoResult<Option<Stmt>> {
    for d in &self.dialects {
        if !d.keywords().contains(&ident) { continue; }
        match d.try_parse_stmt(self, ident)? {
            Some(stmt) => return Ok(Some(stmt)),
            None => continue,   // 让下一个方言有机会
        }
    }
    Ok(None)
}

/// 按 session 装配方言表。集中所有方言的注册，便于一眼看清"当前有哪些方言"。
fn build_dialects(session: &CompilerSession) -> Vec<Box<dyn Dialect>> {
    let mut v: Vec<Box<dyn Dialect>> = Vec::new();
    // 顺序即优先级；先注册先匹配。
    if session.scenario == crate::session::Scenario::UI {
        v.push(Box::new(crate::dialect::ui::UiDialect));
    }
    // 未来：Shell、Godot 等在此注册
    v
}
```

`parse_stmt_inner` 的 `TokenKind::Ident` 分支收敛为：

```rust
TokenKind::Ident => {
    let ident = self.cur.text.as_str();
    // 1) 先问方言表（取代原 is_contextual_keyword 硬编码）
    if let Some(stmt) = self.try_dialect_stmt(ident)? {
        return stmt;   // 注意：这里改成返回 Stmt 而非控制流穿透
    }
    // 2) scene（Godot）—— 当前 always-on，可后续也纳入方言；迁移期保持原样
    if self.looks_like_scene_decl() {
        return self.parse_scene_decl()?;
    }
    // 3) 默认：节点/调用语句
    self.parse_node_or_call_stmt()?
}
```

**UI 方言迁入此处**（`parse_widget_decl` 等方法保持原样，只是改由 trait 派发）：

```rust
// crates/auto-lang/src/dialect/ui.rs
use crate::ast::Stmt;
use crate::dialect::Dialect;
use crate::error::AutoResult;
use crate::parser::Parser;
use crate::session::{CompilerSession, Scenario};

pub struct UiDialect;

impl Dialect for UiDialect {
    fn matches(&self, s: &CompilerSession) -> bool { s.scenario == Scenario::UI }

    fn keywords(&self) -> &'static [&'static str] {
        &["widget", "model", "view", "msg", "on"]
    }

    fn try_parse_stmt(&self, p: &mut Parser, kw: &str) -> AutoResult<Option<Stmt>> {
        let stmt = match kw {
            "widget" => p.parse_widget_decl()?,
            "msg"    => p.parse_msg_decl()?,
            "model"  => p.parse_model_block()?,
            // 注意：view 在顶层有 token 不可达 bug（见 §2.1），迁移时单独修：
            //   让 UiDialect 在 UI 场景对 view 关键字按文本派发，绕过 TokenKind::View。
            "view"   => p.parse_view_block()?,
            "on"     => Stmt::OnEvents(p.parse_on_events()?),
            _ => return Ok(None),
        };
        Ok(Some(stmt))
    }
}
```

**为什么用 trait object（`Vec<Box<dyn Dialect>>`）而非泛型**：parser 已经 13000+ 行、字段众多，给 `Parser` 加泛参会传染整个代码库。trait object 的运行时开销（一次动态派发/语句）可忽略。

**与 `BlockParser`/`special_blocks` 的关系**：现有 `BlockParser` 只覆盖节点体、且从未启用。建议**删除 `special_blocks`**，把"特殊块"概念统一进 `Dialect`（若某方言需要接管节点体，由该方言在 `try_parse_stmt` 内部自行处理）。这能消除两套并行的扩展抽象。

**收益**：
- 新增方言 = 新增一个实现 `Dialect` 的类型 + 在 `build_dialects` 里加一行，**核心 parser 零改动**。
- `is_contextual_keyword`、`Scenario` 枚举不再需要每方言改一次（`Scenario` 可保留为"会话标签"，方言自己判断 `matches`）。
- Shell 场景（当前 vestigial）可以直接以一个 `ShellDialect` 的形式真正落地。

> ⚠️ **本轴不解决"基础 `Stmt` 膨胀"**：UI 节点仍是基础 `Stmt` 变体。这是有意的——引入 `Stmt::Ext(Box<dyn DialectStmt>)` 会牺牲 exhaustive match、影响 typeck/trans/vm 全链路，代价远高于本轴。建议先落地本方案，观察一个版本周期再决定是否启动路线 B（见 §8）。

---

### 6.2【轴 B / VM】消除 VM 路径上的 AuraWidget 中转

**现状**：VM 路径是 `WidgetDecl → extract_widget_from_decl → AuraWidget → synthesize_widget_module → Codegen`。AuraWidget 在这条路上几乎透传（handler 体本就是基础 `Stmt`）。

**目标**：让 VM 路径直接 `WidgetDecl → synthesize → Codegen`，绕过 AuraWidget。AuraWidget 只保留给 transpiler 和 renderer。

**做法**：把 `synthesize_widget_module`（`ui/handler_codegen.rs:345`）的输入从 `&AuraWidget` 改为直接接收从 `WidgetDecl` 提取出的必要信息（handler 列表、state 定义、imports），不再要求一个完整 AuraWidget。或更简洁地：新增一个 `synthesize_from_decl(&WidgetDecl)` 直接走 `WidgetDecl` 的基础 AST 子树（`OnBlock`、`ModelBlock` 等本来就是基础 `Stmt` 的载荷），跳过 `extract_widget_from_decl`。

```rust
// ui/handler_codegen.rs —— 新增直接入口
/// VM 路径专用：从基础 AST 的 WidgetDecl 直接合成 VM 模块，不经 AuraWidget。
pub fn synthesize_from_decl(
    decl: &WidgetDecl,
    child_widgets: &[WidgetDecl],
    import_stmts: &[Stmt],
    import_aliases: &HashMap<AutoStr, AutoStr>,
    api_over_http: bool,
) -> SynthResult<(Module, GenericRegistry)> {
    // 直接遍历 decl.on_blocks / decl.model_block 等基础 AST 子树
    // 复用现有的 rewrite_state_refs_stmts / synthesize_handler_fn / synthesize_state_type
    // 最后调用同一个 Codegen
    ...
}
```

**收益**：VM 路径少一次 IR 往返，且与"消除 AuraExpr/AuraStmt 镜像"（§6.4）解耦——即便 AuraWidget 仍在，VM 也不再依赖它。

**注意**：`extract_widget_from_decl` 仍被 transpiler 和 renderer 使用，**不删**。本步只是给 VM 多开一条直达通道。

#### PR-3 实施记录：先拆分结构（C），后去中转（A）

> 调研（§11 审计）显示 VM 去中转触及 4 条耦合线，比预想复杂。因此 PR-3 拆为两步：
> - **PR-3（本步，已完成）**：拆分 AuraWidget 结构，显式化逻辑/视图依赖边界。纯重构，零行为变更。
> - **PR-3b（后续）**：在清晰边界上做完全去中转。

**PR-3 做了什么**：
- 新增 `WidgetLogicRef<'_>` / `WidgetViewRef<'_>` 零拷贝引用视图（`aura/types.rs`）。
- `AuraWidget::logic()` / `view_data()` 两个方法暴露拆分边界。
- `synthesize_widget_module` 和 `DynamicComponent::with_registry_and_imports` 加结构化注释，显式标注逻辑/视图两部分。
- AuraWidget 保留不删，所有现有消费路径不变。

**AuraWidget 消费方分类**（PR-3 调研确认）：

| 消费方 | 用哪部分 | 具体字段 |
|---|---|---|
| `synthesize_widget_module` | 逻辑 | name, state_vars, handlers, lifecycle, messages, handler_params |
| `VmBridge` state 初始化 | 逻辑 | name, state_vars[].initial |
| `DynamicComponent` 组装 | 视图 + 少量逻辑 | view_tree, name, tick_interval, span_map, key_bindings |
| `AuraViewBuilder`（根） | 视图 | view_tree（作为参数传入，不存 struct） |
| `AuraViewBuilder`（子 widget） | 视图 + 逻辑 | child 的 view_tree + state_vars（经 registry） |
| Transpiler（vue/jet/ark/rust） | 全部 | 逻辑 + 视图 |

#### PR-3b 路线图：VM 完全去中转的四条线

前置条件：PR-3（拆分）已完成，依赖边界清晰。以下四线可独立推进：

**1. synthesize 直读 WidgetDecl**
- 新增 `synthesize_from_decl(&WidgetDecl)` 读 `decl.model.fields` / `decl.on.handlers`
- handler 体从 `OnHandler.body.stmts`（基础 `Stmt`）直接取，不经 `LogicPayload`
- state 类型从 `ModelField` 构建，不经 `AuraStateDef`
- 辅助函数（`synthesize_state_type` / `synthesize_handler_fn`）签名改为接受 decl 或提取出的标量

**2. state 初始化改用基础 Expr 求值**
- 当前 `eval_aura_expr_to_value`（`vm_bridge.rs:204`）消费 `AuraExpr`
- 改为 `eval_ast_expr_to_value` 消费基础 `Expr`（`ModelField.init`）
- 或复用现有 `Expr→AuraExpr` 转换（`extract_expr`）作为过渡

**3. view 树独立提取**
- 新增 `extract_view_tree_from_decl(&WidgetDecl) -> (AuraNode, span_map)`
- 复用 `extract_view_block` + `assign_node_ids`，不经完整 AuraWidget
- `DynamicComponent` 的 `view_template` / `input_state_map` 从此获取

**4. 子 widget registry 去 AuraWidget 化**
- `WidgetRegistry` 当前存 `AuraWidget`
- 改为存 `WidgetDecl` 或 `ViewMetadata`（含 view_tree + state field names）
- `render_child_widget`（`aura_view_builder.rs:1192`）随之调整

**收益**：VM 路径（`run_file_dynamic_ui`）完全不经 AuraWidget，直接 `WidgetDecl → synthesize → Codegen` + `WidgetDecl → view tree`。AuraWidget 退化为仅供 transpiler 使用的 transpiler-only IR。

**风险**：四线中第 4 线（registry）最复杂，因为它改变 `WidgetRegistry` 的公共类型。建议第 4 线最后做，或评估是否值得做（若 transpiler 仍需 AuraWidget，registry 可能需要同时持有两种形式）。

---

### 6.3【轴 B / 转译】确认并巩固 `trans/` 叶子层的统一地位

这一块**已经统一**，本方案只做"确认 + 巩固"，不改架构：

1. **审计**：确认所有 UI 转译器（vue/jet/ark/rust）的 handler 体翻译都经 `ui_gen/ts_adapter.rs` → `trans::typescript`（或对应的 `trans::*`），不存在第二个手写表达式翻译器。若发现遗漏（如 ark 直接手写翻译某构造），收口回 `trans/`。
2. **文档化**：在 `trans/mod.rs` 顶部写明"`trans/` 是所有转译路径（脚本 a2r/a2c/a2py 与 UI a2vue/a2jet/a2ark）共享的唯一叶子翻译层"，避免后人再分支。
3. **不做**：不强求 `a2vue` 和 `a2r` 共享"容器组装"逻辑——它们的程序形态不同（轴 C），各自组装是正当的。

---

### 6.4【轴 C】瘦身 AuraWidget：`AuraExpr`/`AuraStmt` 镜像的处理

> ⚠️ **本节已根据前置审计结果修订。** 审计推翻了"镜像层无消费方"的原始预期（见 §11 审计附录）。本节给出修订后的方案。

#### 审计结论（详见 §11）

`AuraExpr`/`AuraStmt` **并非无消费方**，而是被 ~11 个文件、~12 个消费函数重度使用，分两类：

| 消费类型 | 具体 | 说明 |
|---|---|---|
| **transpiler 代码生成** | `ui_gen/vue.rs`、`ui_gen/rust.rs`、`ui_gen/jet/generator.rs`、`ui_gen/ark/state.rs` 的 `expr_to_js/kotlin/arkts/rust`、`stmt_to_*`、`bin_op_to_*` | view 树/props/state-default 等表达式槽消费 `AuraExpr` |
| **运行时求值/渲染** | `ui/vm_bridge.rs:752`、`ui/state_migration.rs:160`、`ui/aura_view_builder.rs:1965`、`ui/aura_snapshot_builder.rs:345` | 动态求值 AuraExpr 到 Value |

原诊断说"transpiler 走 `trans/`"只对了一半：**handler 体**确实走 `trans/`（经 `ts_adapter`），但 **view/prop/state-default 等表达式槽**消费的是 `AuraExpr`，不经过 `trans/`。

**因此 `AuraExpr`/`AuraStmt` 不能直接删除。** 它们是 transpiler + renderer 路径的输入契约。

#### 修订后的方案：分两档处理

**第一档：`AuraStmt` —— 可优先消除（消费面窄）**

`AuraStmt` 的消费方仅 4 个生成器的 `stmt_to_*` 函数，且 handler 体在 VM 路径已用 `LogicPayload::AstStmts`（基础 `Stmt`）。可把 transpiler 的 `stmt_to_*` 改为消费基础 `Stmt`（4 处），删除 `AuraStmt` + `extract_stmt`。这一档改动可控。

**第二档：`AuraExpr` —— 需要先建替代层，不能裸删（消费面宽）**

`AuraExpr` 被 4 个生成器 + 4 个运行时求值器重度消费，裸删会拆掉整条代码生成与渲染管道。可选策略：

| 策略 | 做法 | 评价 |
|---|---|---|
| **B-1（保守）** | `AuraWidget` 的 handler 槽改持基础 `Stmt`（消除 `AuraStmt`），但 **保留 `AuraExpr` 作为 view/prop 表达式槽类型** | ✅ 低风险、消除 handler 镜像漂移；❌ view/prop 仍有镜像 |
| **B-2（彻底）** | view/prop 表达式槽也改持基础 `Expr`，4 个生成器的 `expr_to_*` + 4 个求值器改为消费 `Expr`（经 `trans/` 或直接 match） | ✅ 彻底消除镜像；❌ 改动大（~12 个函数重写），属高风险大改 |
| **B-3（渐进）** | 先做 B-1，再给 `AuraExpr` 加一个"从基础 `Expr` 透明派生"的桥（让 extract 不再手工逐变体翻译，而是薄包装），最后择机做 B-2 | ✅ 风险递进；每步可独立验证 |

**建议**：PR-5 先做 B-1（消 `AuraStmt`，handler 改持基础 `Stmt`）。`AuraExpr` 的彻底消除（B-2/B-3）作为**独立的后续议题**，不强制纳入本轮迁移——它涉及面太广，应单独立项评估。

#### 第一档（消 `AuraStmt`）的目标结构

```rust
// aura/types.rs
pub struct EventHandler {
    pub key: Key,
    pub body: Vec<crate::ast::Stmt>,         // ← 直接持基础 Stmt，不再用 AuraStmt
    pub span: Span,
}

// AuraExpr 暂时保留（第二档待议）：
pub enum AuraExpr { /* 保持现状，服务 view/prop 槽 + 4 个运行时求值器 */ }
```

#### 子集约束（handler 体）改为校验器

```rust
// aura/validate.rs
use crate::ast::Stmt;

/// 校验 handler 体只使用 UI 运行时支持的语句子集。
/// 替代 extract_stmt 的 UnsupportedStmt 报错。
pub fn validate_handler_stmt(stmt: &Stmt) -> Result<(), UnsupportedKind> {
    match stmt {
        Stmt::Store(_) => Ok(()),
        Stmt::Expr(e) => validate_call_shape(e),
        other => Err(UnsupportedKind::stmt(other)),
    }
}
```

#### 第一档删除/迁移清单
- 删除：`AuraStmt`、`AuraUpdateOp`、`extract_stmt`（`aura/types.rs:723`、`:750`、`aura/extract.rs:388`）。
- 迁移：4 个生成器的 `stmt_to_js/kotlin/arkts/rust` 改为消费基础 `Stmt`（`ui_gen/{vue,rust,jet/generator,ark/state}.rs`）。
- 保留：`AuraExpr`/`AuraBinOp`/`AuraUnaryOp`/`extract_expr`（第二档待议）。
- 保留：`LogicPayload::AstStmts`（VM 路径已用基础 `Stmt`，无需改）。

**收益（第一档）**：
- handler 体的语句语义不再有镜像层——基础 `Stmt` 演进时自动覆盖，消除 `_ =>` 通配漂移。
- handler 体在 VM 路径与 transpiler 路径**首次统一**为同一类型（基础 `Stmt`）。
- `AuraExpr` 的 view/prop 镜像留待后续单独议，不阻塞本轮迁移。

---

### 6.5 审计附录：`special_blocks` 死代码确认

> 审计确认 `BlockParser`/`special_blocks` 为纯死代码，PR-1 可安全删除。

| 项 | 位置 | 审计结论 |
|---|---|---|
| `BlockParser` trait | `parser.rs:145-147` | **零实现**（全工作区无 `impl BlockParser`） |
| `special_blocks` 字段 | `parser.rs:186` | 3 个构造器均初始化为空 HashMap |
| `add_special_block` | `parser.rs:297-299` | **零调用**（仅定义存在） |
| `special_block` 派发 | `parser.rs:9852-9867` | 唯一调用点 `:9478` 的 `contains_key` 永远 false → 不可达 |
| 测试引用 | — | 零 |
| 公开 API | `pub trait`/`pub field`/`pub fn` | 技术上是 breaking change，但工作区内无下游消费者 |

**删除清单（9 处，全在 `parser.rs`）**：`:145-148`（trait）、`:186`（字段）、`:254`/`:321`/`:367`（3 个构造器 init 行）、`:297-300`（方法）、`:9852-9867`（方法）、`:9477-9483`（派发点简化为直接 `parse_node_body`）。

**注意**：`lexer.rs:745` 的 `identifier_or_special_block` 是**同名但无关**的 lexer 方法，**不要删**。

---

## 7. 迁移路线（建议分阶段 PR）

每一步都应保持现有测试全绿；UI 解析的黄金文件测试（若存在）是关键回归保障。

| 阶段 | 轴 | 内容 | 风险 | 可独立合入 |
|---|---|---|---|---|
| **PR-1 基建** | A | 新增 `Dialect` trait、`Parser::dialects`/`try_dialect_stmt`/`build_dialects`；**暂不迁移任何现有逻辑**。加单元测试验证空方言表行为不变。删除休眠的 `BlockParser`/`special_blocks`。 | 低 | ✅ |
| **PR-2 迁移 UI 派发** | A | 把 `widget/model/view/msg/on` 的语句派发迁入 `UiDialect`；删除 `is_contextual_keyword`；顺手修 §2.1 的 `view` token 不可达 bug（让 `UiDialect` 对 view 按文本派发）。 | 中（触及 §2 不一致） | ✅ |
| **PR-3 VM 去中转** | B/VM | 新增 `synthesize_from_decl(&WidgetDecl)`；`run_file_dynamic_ui` 改用它，VM 路径不再构造完整 AuraWidget。`AuraViewBuilder` 若依赖 AuraWidget，改由 `synthesize_from_decl` 同时返回必要的 view 元数据。 | 中 | ✅ |
| **PR-4 巩固 trans 统一** | B/转译 | 审计 vue/jet/ark/rust handler 体翻译是否都走 `trans/`；收口遗漏；在 `trans/mod.rs` 文档化统一地位。 | 低 | ✅ |
| **PR-5 消除镜像** | C | 前置：grep `AuraStmt`/`AuraExpr` 所有消费方，确认主路径无消费者（形成审计记录）。然后删除镜像类型 + extract 函数；新增 `aura/validate.rs`；迁移有消费的残留。 | 高（触及所有 UI 后端，建议拆子 PR：先删 `AuraStmt`，再删 `AuraExpr`） | ✅ |

---

## 8. 关于"基础 `Stmt` 膨胀"（路线 B）的判断

`Dialect`（轴 A）不阻止 `Stmt` 继续增长。若未来方言节点变多到下游消费者难以忍受，再考虑给 `Stmt` 开扩展口（如 `Stmt::Ext(Box<dyn DialectStmt>)`）。但那会牺牲 exhaustive match，且影响 typeck/trans/vm 全链路，**代价远高于轴 A**。

建议：**先做 §6 的三轴改进，观察一个版本周期。** 若 `Stmt` 膨胀确成痛点，再启动路线 B。届时方言体系已就位，扩展口的实验成本更低（方言自带节点类型，只需一个统一的 `Ext` 装载口）。

---

## 9. 关键代码坐标索引

| 关注点 | 位置 |
|---|---|
| 基础 AST `Stmt` / `Expr` | `crates/auto-lang/src/ast.rs:183` / `:307` |
| `Parser` 结构 | `crates/auto-lang/src/parser.rs:178` |
| 入口 `parse` / `parse_stmt` / `parse_expr` | `parser.rs:1108` / `:3552` / `:1428` |
| `parse_stmt_inner` Ident 分支 | `parser.rs:3789-3810` |
| `is_contextual_keyword` | `parser.rs:414` |
| `is_ui_scenario` | `parser.rs:405` |
| `Scenario` 枚举 | `session.rs:27-38` |
| `BlockParser` trait / `special_blocks` / 派发 | `parser.rs:145` / `:186` / `:9477` / `:9852` |
| `parse_widget_decl` 及其内部派发 | `parser.rs:10015` / `:10056` |
| UI AST 类型 | `crates/auto-lang/src/ast/ui.rs` |
| `AuraWidget` / `LogicPayload::AstStmts` | `aura/types.rs:52` / `:562` |
| `AuraExpr` / `AuraStmt`（待删） | `aura/types.rs:580` / `:723` |
| `extract_expr` / `extract_stmt`（待删） | `aura/extract.rs:96` / `:388` |
| `extract_widget_from_decl`（保留） | `aura/extract.rs:750` |
| `ExtractError`（含 dead variants） | `aura/extract.rs:52` |
| VM 共享：`synthesize_widget_module` | `ui/handler_codegen.rs:345` |
| VM 共享：`Codegen::new()` | `ui/handler_codegen.rs:352` |
| VM 共享：`VmBridge` / `AutoVM` | `ui/vm_bridge.rs:103` / `vm/engine.rs:228` |
| 原生 VM 入口 `execute_autovm_with_path` | `lib.rs:547`（parse→codegen→link→vm） |
| Transpiler 统一叶子层 | `trans/{javascript,typescript,python,c,rust}.rs` + `ui_gen/ts_adapter.rs` |
| UI 后端生成器 | `ui_gen/{vue,jet,ark,rust}.rs` |
| `BackendGenerator` trait | `ui_gen/mod.rs:74` |
| `TokenKind::View` / `::On` | `token.rs:111` / `:125` |
| lexer `header_keyword`（`on` 行首） | `lexer.rs:731` |

---

## 10. 已决策清单

> 以下决策已在评审中确认，作为后续 PR 实施的既定前提。

### 决策 1：`view` 顶层块 bug —— ✅ PR-2 一并修
在 UI 方言迁入 `UiDialect` 时，让 `UiDialect` 对 `view` 关键字按**文本**派发（绕过 `TokenKind::View` 的 token 冲突），使 UI 场景下顶层 `view { }` 块真正可解析。这是明确 bug 修复，不改变 `view` 作为 Core 参数模式关键字（`fn foo(view x int)`）的既有语义——二者通过"语句位置 vs 参数位置"天然区分。

### 决策 2：`on` 双路径 —— ✅ 收归 `UiDialect` 单一派发
行首 `on` 仍可由 lexer 产出 `TokenKind::On`（保持 lex 行为不变），但**解析派发逻辑统一进 `UiDialect`**：`TokenKind::On` 分支也委托给方言处理。消除"两条派发路径并存"的散落。

### 决策 3：休眠的 `special_blocks` —— ✅ PR-1 删除
`BlockParser` trait + `Parser::special_blocks` 字段 + `add_special_block()` + `special_block()` 全局零调用，确认为死基础设施，PR-1 删除。未来"特殊块/节点体扩展"需求由 `Dialect` 统一承担（方言在 `try_parse_stmt` 内部自行处理节点体）。

### 决策 4：IR 镜像消除范围 —— ⚠️ 已据审计结果修订（分两档）
**原始决议**（评审时）：`AuraExpr`/`AuraStmt` 若审计确认无核心消费方则全删。

**审计结果**（§11）：**前提不成立。** `AuraStmt` 消费面窄（4 个生成器 `stmt_to_*`），可优先消除；但 `AuraExpr` 被 4 个生成器 + 4 个运行时求值器重度消费（~12 个函数），**不能裸删**。

**修订后的决议**（见 §6.4）：
- **第一档（PR-5 范围）**：消除 `AuraStmt`，handler 槽改持基础 `Stmt`，4 个生成器 `stmt_to_*` 迁移到基础 `Stmt`。`AuraExpr` 暂时保留。
- **第二档（独立后续议题，不纳入本轮迁移）**：`AuraExpr` 的彻底消除（view/prop 槽改持基础 `Expr`，迁移 ~12 个消费函数）作为单独议题评估，采用渐进策略 B-3。

### 决策 5：PR-3 view 元数据旁路 —— ✅ 采用候选方案，到 PR-3 详设
`synthesize_from_decl` 返回 `(Module, GenericRegistry, ViewMetadata)` 三元组，`ViewMetadata` 承载 `AuraViewBuilder` 所需的 view 树信息，使 VM 路径绕过 AuraWidget 的同时不丢失渲染所需元数据。具体结构在 PR-3 设计时确定。

### 决策 6：场景配置 —— ✅ 引入 `--scene` CLI flag + `.au` 后缀

分三部分落地（独立于 PR-1~5，可作为并行 PR-6）：

#### (a) `--scene` CLI flag（优先级覆盖）
新增 `--scene <core|ui|shell>` CLI 选项。优先级：

```
CLI --scene  >  pac.at 的 scene 字段  >  默认(Core)
```

CLI 显式指定时覆盖 `pac.at` 配置，否则沿用 `pac.at`（现状：`scene: "ui"` 字段），再否则默认 `Core`。向后完全兼容。

#### (b) `.au` 后缀 —— 弱提示 + 默认推断
- **语义**：`.au` = "Auto UI" 源码。文件后缀为 `.au` 时，**默认推断**为 UI 场景（等价于自动设 `--scene ui`），但可被 `--scene` / `pac.at` 覆盖。
- **不破坏混合文件**：`.at` 保持现状，仍可含 widget（场景由配置决定）。`.au` 不做强约束（不要求文件必须含 widget，也不禁止 `.at` 含 widget）。
- **与 `.vm.at` 的关系**：`.vm.at` 是 AutoVM 上下文文件（`compile.rs:1269`），是独立维度的后缀约定，不受 `.au` 影响。未来若有"UI 的 VM 上下文文件"，按现有插前缀规则生成 `.vm.au`。

#### (c) 模块解析的连带改动（引入 `.au` 必须处理）
`compile.rs:1121` 的 `extensions = [".at", ".auto"]` 决定了 `use`/`import` 时按哪些后缀查找模块文件。引入 `.au` 需要同步更新查找逻辑：

- **建议规则**：`use foo` / `import foo` 查找顺序为 `[foo.au, foo.at, foo.auto]`（UI 优先），命中即止。
- **理由**：UI 组件文件（`.au`）作为模块被引用时，应优先命中；同时保持对纯 `.at` 模块的向后兼容。
- **场景一致性校验**（可选增强）：若 `--scene=core` 的文件引用了 `.au` 模块，给出 warning（提醒场景不匹配），但不阻止——避免过度约束。

#### 落地建议
决策 6 作为独立 PR-6，与 PR-1~5 解耦。建议在 PR-2（`UiDialect` 就位）之后实施，这样 `.au` 的场景推断可直接走方言体系，而非临时硬编码。

---

## 11. 审计附录（PR-1 前置）

> 三路并行审计的结论数据，作为 PR-1~5 计划的事实基础。

### 11.1 `AuraExpr`/`AuraStmt` 消费方审计

**关键结论**：这两个类型**不是**无消费方，而是被重度消费。工作区为 `crates/auto-lang/`，其他 crate（auto-man/auto-vm/auto-lsp/...）零引用。

| 类型 | 定义 | 构造点 | 消费点（(c) 类） |
|---|---|---|---|
| `AuraExpr` | `aura/types.rs:580` | `extract.rs:96`、`a2ui/import.rs:566` + 大量 test/state-init | **~12 个函数跨 11 文件**（见下） |
| `AuraStmt` | `aura/types.rs:723` | `extract.rs:388` | **4 个生成器 `stmt_to_*`** + 4 个 vue 分析函数 |
| `AuraBinOp` | `aura/types.rs:689` | `extract.rs:355` | 5 个生成器 `bin_op_to_*` + `aura_view_builder` + `atom.rs` |
| `AuraUnaryOp` | `aura/types.rs:712` | `extract.rs:375` | 4 个生成器 + `vm_bridge` 运行时 + `atom.rs` |
| `AuraUpdateOp` | `aura/types.rs:750` | `extract_stmt` 内部 | 4 个生成器 `stmt_to_*` |

**`AuraExpr` 的 12 个消费函数**（按类别）：

| 类别 | 函数 | 位置 |
|---|---|---|
| Vue 生成 | `expr_to_js`/`expr_to_auto_string`/`expr_to_ts_type`/`expr_to_vue_text`/`bin_op_to_js` 等 | `ui_gen/vue.rs:2890,3449,3617,3822,4149,...` |
| Rust 生成 | `expr_to_rust`/`stmt_to_rust`/`expr_to_json_value`/`bin_op_to_rust` | `ui_gen/rust.rs:3196,3228,3272,3445` |
| Jet 生成 | `expr_to_kotlin`/`stmt_to_kotlin`/`binop_to_kotlin` | `ui_gen/jet/generator.rs:743,936,973` |
| Ark 生成 | `expr_to_arkts`/`stmt_to_arkts`/`expr_to_modifier` | `ui_gen/ark/state.rs:464,492`、`ark/generator.rs:1450,1769` |
| VM 运行时求值 | `eval_aura_expr_to_value` | `ui/vm_bridge.rs:752` |
| 状态迁移 | `eval_default` | `ui/state_migration.rs:160` |
| 动态渲染 | `resolve_expr_to_value`/`resolve_expr_to_string` | `ui/aura_view_builder.rs:1900,1965` |
| 快照序列化 | `eval_expr`/`eval_expr_bool` | `ui/aura_snapshot_builder.rs:345,366` |
| atom 序列化 | `serialize_expr` | `aura/atom.rs:304` |
| a2ui 桥 | `import_value`/`export_expr` | `a2ui/import.rs:566`、`a2ui/export.rs` |

**修正了原诊断的一个错误认知**：transpiler 的 **handler 体**确实走 `trans/`（经 `ts_adapter`），但 **view/prop/state-default 表达式槽**消费的是 `AuraExpr`，不经过 `trans/`。这是 §6.4 分两档处理的依据。

### 11.2 `special_blocks` 死代码审计

确认纯死代码。数据流可证明为惰性：`add_special_block` 零调用 → `special_blocks` 恒为空 HashMap → `contains_key`（`:9478`）恒 false → `special_block`（`:9852`）不可达 → `BlockParser` 零实现。

删除清单见 §6.5。注意 `lexer.rs:745 identifier_or_special_block` 是同名无关的 lexer 方法，不删。

### 11.3 派发链审计

**UI 关键字派发链的唯一入口**：`parse_stmt_inner` 的 `TokenKind::Ident` 分支（`parser.rs:3789-3810`）。

| 项 | 调用者数 | 位置 |
|---|---|---|
| `is_contextual_keyword` | **1** | `parser.rs:3796`（Ident 分支内） |
| `is_ui_scenario` | **1** | `parser.rs:415`（`is_contextual_keyword` 内） |
| `TokenKind::On` 分支 | 独立 arm | `parser.rs:3816`（always-on，与 Ident 内的 UI-gated `"on"` 并存） |

5 个 UI 解析函数签名：

| 函数 | 返回 | 备注 |
|---|---|---|
| `parse_widget_decl` / `parse_msg_decl` / `parse_model_block` / `parse_view_block` | `AutoResult<Stmt>` | 直接返回 Stmt，方言可直接调用 |
| `parse_on_events` | `AutoResult<OnEvents>` | ⚠️ 返回 `OnEvents` 非 `Stmt`，方言需自己包 `Stmt::OnEvents(...)` |

**构造器/session 约束**：3 个构造器（`new`/`new_with_note`/`new_with_note_and_first_token`）均默认 `CompilerSession::default()`（=Core），真实 session 由 `with_session()`（`parser.rs:400`）后置注入。因此 `build_dialects(session)` **不能在 `Parser::new` 里跑**，必须：
- 方案①：在 `with_session` 内部构建方言表；或
- 方案②：在 `try_dialect_stmt` 首次调用时惰性构建（snapshot `self.session`）。

`CompilerSession` 是 `Clone`/owned-by-value，`session` 字段是 `pub`，故 `build_dialects` 可自由取值。

### 11.4 `pac.at` 场景配置流向

auto-lang 核心 crate **不解析 `pac.at`**。scenario 在核心层有两个注入点：
- `lib.rs:1904-1920`：模块路径启发式（`back/` → Core，否则 UI）。
- `lib.rs:3356-3369`：`scenario: &str` 参数显式传入。

`pac.at` 的 `scene: "ui"` 解析在**上层 crate**（CLI/项目层，非 auto-lang）。决策 6 的 `--scene` flag 实现也应在那一层，最终经 `lib.rs:3356` 的入口注入核心 crate。
