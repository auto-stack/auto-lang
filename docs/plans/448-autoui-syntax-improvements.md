# Plan 448: AutoUI 语法改进——msg 声明去名 + 事件内联 lambda 简写（滚动收集）

> **状态**: 🟦 已立项待执行（2026-08-25 立项；需求 A/B 已完成代码级调研，
> 调研结论与行号以基线 bcb6e139b 为准，实施时如有漂移按符号名重定位）
> **来源**: examples/ui/002-counter/src/front/app.at 示例走查（文件注释区的"简写版"）
> **基线**: master bcb6e139b
> **性质**: **滚动收集计划**——逐个 UI 示例走查，收集 AutoUI 语法改进需求追加为
> 需求 C/D/…；每条独立实施、独立验证、独立勾销，不必一次做完。

## §0 条目总览

| 条目 | 一句话 | 状态 |
|---|---|---|
| A | `msg Msg {…}` 去掉无用的名字 → `msg {…}` | 待实施 |
| B | `onclick: () => {…}` 内联 lambda 简写（含 B2：VM 路径复合赋值修复） | 待实施 |
| C+ | 后续从其他示例收集（占位） | — |

条目 A/B 相互独立可分别实施；B 实施后简单 widget 可完全不写 msg/on
（002-counter 目标形态即如此）。

---

## §1 需求 A：msg 声明去名（`msg Msg { Inc, Dec, Reset }` → `msg { Inc, Dec, Reset }`）

### A.1 动机与证据

`Msg` 名字**无任何语义用途**（2026-08-25 全仓库审计结论）：

- 事件查找永远取尾段：`.Inc` / `Msg::Inc` / `App.Dec` 在所有消费方收敛为裸名
  `Inc`/`Dec`，中间限定名从不参与查找——
  `ui/handler_codegen.rs:519`（bare_handler_name）、`ui/vm_bridge.rs:1394`
  （extract_handler_name）、`ui_gen/vue.rs:11893`（pattern_to_handler_name）、
  `ui_gen/rust.rs:3683`（extract_variant_name）、`aura/extract.rs:329-381`。
- 各后端枚举名不依赖它：Rust 用 `format!("{}Msg", widget_name)` 派生
  （`ui_gen/rust.rs:274` current_msg_name）；Kotlin/ArkTS 硬编码 `Msg`
  （`ui_gen/jet/generator.rs:338`、`ui_gen/ark/state.rs:340`）。
- 它仅有的存在感是"被打印/被复制"：AST `MsgDecl.name`、`AuraMessage.name`
  复制（`aura/extract.rs:759`）、atom 序列化（`aura/atom.rs:78`）、Display。
- 铁证：`crates/auto-lang/tests/fixtures/pkg_app.at:4` 写的是 `msg M { Go }`，
  产物照样叫 `Msg`/`PkgAppMsg`——名字已是死参数。

去名后与 widget 其他无名的体关键字（`model` / `view` / `on` / `props`）形态统一。

### A.2 方案

1. **Parser 名字改可选**（`parser.rs:13103-13183` parse_msg_decl_inner）：
   - 新正字法 `msg { variants }`——`expect_ident("msg")` 后若 cur 是 `{` 直接进体；
   - 兼容窗口内仍接受旧写法 `msg Name { … }`（读掉名字 token，**丢弃**，不发警告；
     仓库外用户代码不受打断）。
2. **AST 去 name 字段**：`ast/ui.rs:279-286` `MsgDecl { name, variants }` →
   `{ variants }`；同步 `ast.rs:308` Display、`ast.rs:1183-1185` to_node、
   `ast.rs:1260` source 三处打印。
3. **Aura/atom 层**：`aura/types.rs:702-708` `AuraMessage` 删 `name` 字段；
   `aura/extract.rs:757-759` 不再复制；`aura/atom.rs:78-79` atom 输出去掉该键
   （仓库内无消费者、无测试断言——若有外部工具读 atom 需先确认，见 A.5 风险）。
4. **死代码清理**：`ui_gen/shared/state.rs:120-122` `add_message`（唯一调用方是
   同文件测试）连带删除。
5. **语料迁移**：全仓库 175 处 `msg Msg {`/`msg M {` 机械替换为 `msg {`——
   分布：examples/ 112 文件（ui 42、capability-tests 38、根目录散例 12、
   widgets-gallery 15）、stdlib/aura/widgets/ 22、crates/auto-lang/test/ 8、
   tests/fixtures/ 3、blocks/ 6。排除 gen/ 生成物与 website/.vitepress/dist 构建产物。
6. **内嵌源码的 Rust 测试同步**（约 12 文件）：`AuraMessage { name: … }` 字面量
   10 处（`ui_gen/rust.rs` 测试 ×7、`ui_gen/ark/state.rs:594`、
   `ui_gen/ark/generator.rs:2156`、`aura/types.rs:1128`）+ `ast/ui.rs:912`
   MsgDecl 字面量 + parser.rs 内联 msg 测试（15628 起 ~15 处，多数只断言
   variants 无需改）。
7. **文档**：`ui_gen/docs_gen.rs:227` 硬编码输出 `msg Msg { Go }` 改为
   `msg { Go }`；docs/ 下 32 个提及文件更新（dist 构建产物除外）。

### A.3 测试

- parser 单测：`msg { A, B }` 解析出 MsgDecl{variants:[A,B]}；旧写法
  `msg Msg { A }` 仍解析成功且 name 被丢弃。
- atom/golden 涉及 msg 的输出若断言了 name 键则更新。
- 基线：`cargo test -p auto-lang --lib` 全绿；`--test vue_capabilities`、
  a2vue/a2ark golden 全绿（`test/a2ark/020_msg_enum` 期望产物本身是生成器
  硬编码 `enum Msg`，不受影响）。
- 迁移后全量 repossample 编译冒烟：examples/ui 至少 002/013/015 三例 regen 无 diff。

### A.4 边界与不做

- 不改事件绑定/查找语法本身（`.Dec`、`App.Dec` 等尾段解析规则原样）。
- 不动 MsgVariant 的 payload/quoted 语义。
- 旧写法 `msg Name {…}` 保留兼容窗口，不设删除时点（另行 housekeeping）。

### A.5 风险

- atom 输出去掉 name 键是**外部可见格式变化**：仓库内无消费者，若有仓外工具
  读 atom IR 需确认；保守替代方案是 atom 层恒输出 `"Msg"` 常量。
- 175 处替换是纯机械操作但量大，靠"替换后全量测试"兜底。

---

## §2 需求 B：事件内联 lambda 简写（`onclick: () => {…}`）

### B.1 目标形态（002-counter 简写版，实施后该文件即此形态）

```
widget App {
    model { var count int = 0 }
    view {
        center {
            text `Counter: ${.count}`
            row {
                button "-"    { onclick: () => {.count -= 1} }
                button "Reset"{ onclick: () => {.count = 0} }
                button "+"    { onclick: () => {.count += 1} }
            }
        }
    }
}
```

简单回调不再经历"msg 声明 → onclick 发事件 → on 收事件"三段式；编译器铸
匿名事件名并自动合成 on 处理器（desugar）。msg/on 块在简单 widget 里可完全省略。

### B.2 现状与前置能力

- **closure 语法已存在**（Plan 060）：`Expr::Closure`，`parser.rs:4374`
  parse_closure 支持 `() => {语句块}`（空参 4386、块体 4429-4431）；TS 侧转译
  现成（`ts_adapter.rs:1546`、块表达式 1566）。
- **缺口在事件绑定**：`ViewEvent { name, handler: String, params }`
  （`ast/ui.rs:556`）的 handler 是纯字符串；`parse_event_handler`
  （`parser.rs:14648`）只接受 `.Name` / `Name` / `.Name(args)`。今天写
  `onclick: () => {…}` 会把 `(` 当 handler 文本吃掉随后语法错误。
- 属性值入口两个：花括号形态 `parser.rs:14123`、圆括号形态 `13963`，都要接。

### B.3 方案

**B1 解析 + desugar：**

1. `ViewEvent` 增字段 `inline: Option<Vec<Stmt>>`（lambda 体语句；破坏面最小，
   ViewEvent 消费方仅 parser / aura::extract / vue.rs 测试 / plan408_tests）。
2. `parse_event_handler`（或其两个调用点）值以 `(` 开头时做 `) =>` lookahead
   （复用 `parser.rs:1979-1995` 现成的零参 closure 探测），命中走 `parse_closure()`
   取 `Expr::Block(body)` 存入 inline；同时支持带参 `(e) => {…}`（事件对象参数）。
3. **desugar 单点放在 `aura/extract.rs` extract_widget_from_decl**（view 树抽取处
   850/962 构造 AuraEvent 之前）：为每个 inline 事件铸唯一名——沿用 `__` 前缀
   约定（参照 `__stream_sse_*`，parser.rs:15013），如 `__evt_onclick_1`（每 widget
   计数器）——handler 写 `.__evt_onclick_1`；并：
   - 向 `AuraWidget.handlers` 注入 `LogicPayload::AstStmts(lambda体)`；
   - 向 `AuraWidget.messages` 补无参合成 variant（widget 没有 msg 声明时，
     枚举仅由合成 variant 构成，照常生成）。
   单点覆盖全部后端（Vue/Rust/Ark/Kotlin/a2ui/VM），AST 保持纯净。

**两个硬约束（漏掉即无声失效）：**

- Rust 后端对不在 msg 枚举里的 handler **静默跳过 match 臂**
  （`ui_gen/rust.rs:1163-1168`）→ 必须补合成 variant，否则无任何报错直接失效。
- Vue 生成的 handler 函数尾部自动 `emit('Name')`（`ui_gen/vue.rs` ~2248-2300，
  defineEmits 受 msg variants 约束 2068-2095）→ 合成名不在 defineEmits 里，
  strict TS 下会报错；对 `__evt_*` 合成 handler 豁免尾部 emit。

**B2 顺带修复：VM 路径 Dot 左侧复合赋值（`.count += 1` 的唯一缺口）**

复合赋值支持现状分路径：

| 路径 | `.count += 1` | 依据 |
|---|---|---|
| Vue/TS、ArkTS、Kotlin、a2r 转译、ui_gen Rust 生成 | ✅ | `ui_gen/rust.rs:4481` 专门处理；012-stopwatch 原生产物即 `self.elapsed += 10`；Vue 有测试锁定 |
| iced/GPUI/headless VM 路径 | ❌ | `.count` 重写为 `__state.count`（`ui/handler_codegen.rs:98`）后是 Dot 表达式；`vm/codegen.rs:5897` 复合赋值只接受 Ident 左侧，`:5969-5974` 直接报错，且**整个 widget handler 合成失败**（`ui/vm_bridge.rs:218`） |

修法二选一，**推荐 (a)**：

- (a) `ui/handler_codegen.rs` 重写层 desugar：`x.f op= e` → `x.f = x.f op e`
  （普通 `=` 的 Dot 左侧 SET_FIELD 路径已存在，vm/codegen.rs:6184 起）。改动小、
  不碰 VM 字节码语义；B2 与 B1 解耦，可先行单独落地。
- (b) `vm/codegen.rs` 复合赋值分支支持 Dot 左侧（GET_FIELD + 运算 + SET_FIELD）。
  更通用但动 VM 核心，留作后续债务项。

### B.4 测试

1. parser 单测：三种内联形态（零参/带参/多语句体）解析出 inline 体；
   旧语法 `.Dec` / `.Dec(arg)` 回归不受 lookahead 影响。
2. `ui_gen/vue.rs` 单测：inline 事件生成 `@click="__evt_onclick_1"` +
   `function __evt_onclick_1()`，**无尾部 emit**、不进 defineEmits。
3. `ui_gen/rust.rs` 单测：合成 variant 进 `AppMsg` 枚举 + match 臂 +
   `on_click(\|_| AppMsg::__evt_onclick_1)`；无 msg 声明的 widget 枚举仅含合成项。
4. VM（B2 后）：vm_bridge 测试——handler 体含 `.count += 1` 的 widget 合成成功
   且语义等于 `.count = .count + 1`（现有 `ui/vm_bridge.rs:2189` 测试手构造的
   desugar 形态可改为直接写 `+=`）。
5. 002-counter 改为简写版形态后：a2vue golden 更新 + regen 冒烟（Vue 与
   rust-workspace/counter 原生构建两条路）。

### B.5 边界与不做

- 同一 lambda 出现多次不去重（每处独立 handler），行为正确仅产物略多。
- 事件修饰符 `onclick.stop: () => {…}` 组合要保证修饰符解析（14627，
  在值之前）与内联值互不干扰；本轮只保证不报错、语义不变。
- `.Name(args)` 与 `(...) =>` 的 lookahead 边界：老语法 `.Dec(...)` 必须零回归
  （现有 plan408_tests 等锁定）。
- 不做跨 widget 提升复用、不做 lambda 捕获参数以外的事件元数据透传。

### B.6 风险

- ark/jet/kotlin/a2ui 次要后端消费 AuraEvent 字符串名，合成名 `__evt_*` 的映射
  需各跑一轮回归（它们不识别 `__` 前缀的语义，但作为普通名应直通）。
- Vue/TS strict 下合成 handler 的类型签名（事件参数 `any`）与现有 handler 保持一致。
- 每 widget 的合成计数器需在增量编译/regen 间稳定（同一输入同名输出，否则
  golden diff 噪声）——计数按 view 树深度优先出现顺序分配即可稳定。

---

## §3 后续条目（占位）

后续按示例走查追加需求 C/D/…，格式沿用 §1/§2（动机与证据 → 方案 → 测试 →
边界 → 风险），并在 §0 总览表登记。
