# Plan 448: AutoUI 语法改进——msg 声明去名 + 事件内联 lambda 简写（滚动收集）

> **状态**: 🟢 A/B1/B2 已合并 master（merge `7f4ed335c`）；C 已合并 master
> （merge `f5ad59bae`）；D 已合并 master（merge `89aa2aa31`）；
> **I 已登记未实施**（2026-09-02 三轮走查 028 后批次：038/041-044/p 系列，
> 见 §8 + §6 三轮补充）；C/D/… 继续按示例走查追加
> **来源**: examples/ui/002-counter/src/front/app.at 示例走查（文件注释区的"简写版"）
> **基线**: master bcb6e139b（实施于 9e330123f 分叉的 worktree）
> **性质**: **滚动收集计划**——逐个 UI 示例走查，收集 AutoUI 语法改进需求追加为
> 需求 C/D/…；每条独立实施、独立验证、独立勾销，不必一次做完。

## 验证结果（2026-09-02 二轮，worktree plan-448 @ 278ea1cc3，条目 D + 016-028 走查）

- 新增测试：vue `test_style_array_and_class_safe_concat`（数组 join 双
  形态/class-safe 拼接/ref 拼接保 :style）+ VM `plan448_style_array_
  and_concat_resolution`（真实链路 join/空段跳过/状态翻转/拼接修复）。
- 全量：默认 lib 3343 绿 0 失败；ui-iced lib 失败 17 = master 预存
  flaky 簇（renderer desktop/osconfig daemon/session——master 孤立跑
  反挂更多 6>2，环境敏感非本改动）；gallery_golden 绿；vue_capabilities
  77+5 同 master；docs_gen 4 绿。
- 示例：024 regen `:class` join 形态 + vue-tsc 错误签名与 master 逐条
  相同；018 解析/抽取过（S002 预存校验拦截，master 同错）。
- 走查覆盖：016/018（app+5 页）/019/020/021/022/023（6 页）/024/025/028
  ——H 登记（computed 短板三源证据）、F 升级（025 反证能力在）、E 根因
  （025 watch 规避注记）、016 msg-Init 观察入 §6。

## 验证结果（2026-09-02，worktree plan-448 @ ff3bfef3c，条目 C）

- 新增测试四路 + e2e 双实证全绿（明细见 §3.3）：parser 铸名单测、vue 折叠
  纯净断言、rust 枚举/注入断言、vm 写回链路；MCP e2e 005（写回+副作用）
  与 010（纯裸 value）。
- 全量：默认 lib 3342 绿 0 失败；ui-iced lib 4374 绿（17 失败 = master
  预存/flaky 逐名重合：strips_tags、counter_loopback、renderer desktop
  簇、d8_toggle_dark_mode、style_migration_probe、p054 icon flaky）；
  gallery_golden 绿（Vue 产物逐字节不变）；vue_capabilities 77+5 同 master；
  docs_gen 4 绿；test-trans 失败集与 master 逐名对照重合（a2r 预存簇 +
  ring_caps 双侧孤立绿 flaky）。
- 示例构建：005/010 `auto build`（Vue）全绿；013/017 失败与 master
  同签名（预存）；459 `--example ui_dual_app`（ui-iced）编译绿。

## 验证结果（2026-08-25，worktree plan-448）
- A（commit 5e43662bf）：`cargo test -p auto-lang --lib` 3181 绿；
  vue_capabilities 72 / docs_gen 4 / gallery_golden 1（基线重生成）/
  ui_snapshots 3（路径归一化后重生成）/ schema_drift 1 / component_registry 7 绿；
  auto-man 6 绿；doctest 4 失败与 bench E0601 均为 master 预存（逐一对齐确认）。
- B2（commit ddc028455）：`--features ui-iced` `ui::` 组 474 绿；全量 lib
  （ui-iced）3678 绿；新增 vm_bridge 复合赋值全链路测试。
- B1：全量 lib 3185（默认）/ 3683（ui-iced）绿；vue_capabilities 72、
  gallery_golden 1、ui_snapshots 3、docs_gen 4、auto-man 229 绿
  （auto-man `test_resolve_at_adapter` 与 `benchmark_downcast_performance`
  为并行竞态/计时型 flaky，单跑稳定绿，与本计划无关）；002-counter 改简写版后
  `auto build` regen 成功，产物 App.vue：`@click="__evt_onclick_N"` + 三个
  minted 函数体（`count.value ±= 1;`），无尾部 emit，vue-tsc --noEmit exit 0，
  vite build 绿。
- 遗留（另行 housekeeping）：rust-workspace/counter 副本为 a2rust-ui 旧产物
  （本就滞后于现行 codegen），无标准 regen 命令，未刷新。

## 审计补漏（2026-08-25 第二轮，实施后全面自查）

首轮实施后逐条对照计划审计，发现并修复/补齐：

0. **VM(iced) 模式点击无效（真 bug，第三轮修复——铸名提前到 parser）**：
   iced 运行时经 `run_file_dynamic_ui_inner →
   DynamicComponent::with_registry_and_imports_from_decls → VmBridge::new_from_decls`
   从 **AST 声明（decl.on）** 合成 VM handler 函数，而首轮 desugar 只注入到
   提取后的 AuraWidget——视图树带 `__evt_*` 名字但 VM 里没有对应函数，
   `call_handler_for` 的 HandlerNotFound 被静默吞掉（仅 ASH_DEBUG_VM_LOG 可见）。
   定位过程：库级复刻（extract/dispatch/view 全通）→ 插桩 update → MCP
   autoui_action press 实证消息到达且 handler 识别正确但 count 不变 → 追到
   decl-based 合成分裂。修复：**铸名移到 parser**
   （`mint_inline_event_handlers`，widget 与 component fn 两个构造点），同时
   改写事件引用、追加 OnHandler 到 decl.on、追加 MsgVariant 到 decl.messages
   ——提取路径与 decl 合成路径从此同源。fragment 体内联 lambda 仍在提取期铸名
   但改用 `__evtf_` 前缀（避免与 parser 铸名冲突；VM 路径不支持为已知 v1 边界，
   Vue/Rust 路径经注入仍可用）。新增 `test_inline_lambda_event_decl_based_synthesis`
   （真实应用路径）+ 真源冒烟；MCP e2e 实证 count 0→3→2。
1. **view fn 片段内联 lambda 静默丢失（真 bug，已修）**：desugar 预pass只走
   `decl.view`，而 view fn 片段在 `extract_view_node` 内部从 thread-local
   注册表展开（Element/Component 两个调用点），完全绕过预pass——实证失败形态
   为 onclick 空字符串、handler 体丢弃且无任何报错。修复：铸名上下文改为
   thread-local `INLINE_MINTS`（与 `VIEW_FRAGMENTS` 同型，每 widget 重置），
   两个展开点对展开后节点补跑 `collect_inline_events`；新增回归测试
   `test_extract_inline_lambda_in_view_fn_fragment`。注意：片段参数替换不进入
   inline 体（与旧字符串 handler 同限制），体应引用 widget state。
2. **examples/blocks 的 README 迁移遗漏（已补）**：首轮文档迁移只扫了 docs/
   与 website/，漏 examples/**/README.md、blocks/**/README.md（~30 文件）；
   002-counter README 整体重写为简写版形态（含"何时改用显式 msg/on"指引）。
3. **regen 冒烟补齐**：003-converter、002-counter 构建干净（正向通过）；
   013-todo / 015-notes / 011-calculator 构建失败系 master 预存
   （R006/R007 v-for :key 校验、eval_expr 缺导入），与 master 的 auto.exe
   逐字节同错——非本计划引入，建议另行立项修复。
4. **ark/jet 次要后端补用例**：新增 from-source 内联 lambda 代码生成测试
   （parse→extract→generate），铸名 `__evt_onclick_1` 在 Kotlin/ArkTS 产物中
   直通验证。

第二轮验证：全量 lib 3188（默认）/ 3686（ui-iced）绿；vue_capabilities 72、
gallery_golden 1、ui_snapshots 3、auto-man 229 绿。

### 仍开放的遗留（明示）

- 带参 inline lambda `(e) => {…}` 提取期显式报错（v1 范围，Rust 枚举需具体
  payload 类型）。
- B2 方案 (b)（vm/codegen 原生 Dot 左侧复合赋值）递延——UI handler 之外的
  一般 VM 代码 `obj.field += 1` 仍报错。
- Rust 内嵌测试源码仍用旧写法 `msg Msg`（有意保留作兼容路径覆盖；关闭兼容
  窗口时统一迁移）。
- atom IR 去 name 键的仓外消费者假设未验证（仓内无消费者已确认）。
- parser 测试未钉住：多语句 lambda 体、表达式体 `() => expr`、`onclick.stop:`
  修饰符与内联组合（解析路径存在，无测试）。
- rust-workspace/counter 副本未刷新（a2rust-ui 旧产物，无标准 regen 命令）。
- 013/015/011 的预存构建失败（R006/R007/eval_expr，master 同错，非本计划）。

## §0 条目总览

| 条目 | 一句话 | 状态 |
|---|---|---|
| A | `msg Msg {…}` 去掉无用的名字 → `msg {…}` | ✅ 已实施 |
| B | `onclick: () => {…}` 内联 lambda 简写（含 B2：VM 路径复合赋值修复） | ✅ 已实施 |
| C | 裸 `value: .field` 两向绑定——输入框免 msg/on 三件套 | ✅ 已实施（已合并） |
| D | style 组合能力——`style: ["基座", if … {…} else {…}]` 数组形态 + class-safe 拼接 | ✅ 已实施（本节 2026-09-02） |
| E | str 充当 bool/状态机（根因：watch 无 immediate 的规避 + 作者惯性） | 📋 观察（§6） |
| F | 手工展开的标量列表族（能力已存在，纯语料迁移） | 📋 观察（§6） |
| G | registry 组件输入协议与裸 value 语义不一致 | 📋 观察（§6） |
| H | computed 块体/派生态短板——已裁决分 H1/H2/H3 三阶段（H3 独立立项） | 🟡 H1/H2 实施中（§7） |
| I | grid `cols:`/`gap:` 动态值——038 三份 48 行棋盘块的存在原因 | 📋 已登记（§8） |

条目 A/B 相互独立可分别实施；B 实施后简单 widget 可完全不写 msg/on
（002-counter 目标形态即如此）；C 实施后表单输入框亦可完全不写
msg/on（005-login 等目标形态即如此）。

### 实施补遗（与原方案的差异）

- **A**：`ui_gen/shared/state.rs` 的 `add_message` 未删——它属于 Vue
  StateAnalyzer 的 `MessageDef`（独立结构），与 `MsgDecl.name` 无关；
  顺带修复：ui_snapshots 快照路径归一化为 examples/ 相对路径（此前嵌入主仓
  绝对路径，任何 worktree 运行必挂）；auto CLI cmd_ui 打印改列 variants；
  gallery_vue_golden 基线随 .at 迁移重生成。
- **B1**：事件值以 `(` 开头时**无需 lookahead**——旧语法没有任何以 `(` 开头的
  值形态，直接分流进 `parse_closure`；带参 lambda `(e) => {…}` 在提取期显式
  报错（Rust 枚举需要具体 payload 类型，v1 不支持）；Vue defineEmits 保留
  合成 variant 声明（类型正确、无副作用），仅豁免尾部 `emit('__evt_*')`。
- **B2**：desugar 条件覆盖 Dot LHS **与裸 state 字段 Ident**（后者经 Phase-1
  重写同样变 Dot LHS，调研时未意识到）；toast 重写既有手工 Asn+Add 展开与
  desugar 同型，互为印证。

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

## §3 需求 C：裸 `value:` 两向绑定（输入框免 msg/on 三件套）✅ 已实施

### C.1 动机与证据（2026-09-02 第二轮走查）

第二轮按示例走查（003/005/010/011/012/013/015/017/022/0459）发现**最大存量
样板**：每个表单输入框需要三件套——msg 变体 + `oninput: .XChanged` 绑定 +
`.XChanged -> { .x = .x }` handler。全仓同名自赋 `.x = .x` 共 8 处
（005×2、010×3、013×1、017×1、459×1，另 015 一处走 registry 协议），
连带 9 个 msg 变体与 9 个视图绑定。

关键机制事实（走查确证）：

- **`.x = .x` 是空转仪式**：VM/iced 路径 `input_state_map` 在 handler 运行
  **前**写回（`ui/dynamic.rs` `on_with_input_for`）；原生 Rust 路径
  `input_fields` 注入 `self.field = last_input_text()`（`ui_gen/rust.rs`
  `scan_input_fields`）；Vue 路径 v-model 折叠。三路的 `.x = .x` 均为 no-op。
- **裸 `value:` 只有 Vue 双向**：vue.rs 在无 handler 时也折 v-model
  （PLAN-037 T5）；但 iced 的 `on_change` 仅在存在 handler 时接线——裸
  `value:` 在 VM 模式打字**永不落盘**。这是真实能力缺口，非纯样板问题。

### C.2 方案（实施于 parser 铸名层，与 B 同点）

1. **Parser 新 pass `mint_bare_input_sync`**（`mint_inline_event_handlers`
   内、inline 铸名之后）：`input`/`textarea` Element 且 `value:` 为直连
   state 字段（`Dot(Ident("self"|"."), field)` 或裸 `Ident`，镜像 rust.rs
   `is_direct_self` 规则）且无 oninput/onchange/input/change 事件时，
   铸 `oninput: .__bind_<Widget>_oninput_<n>`（空体）+ msg 变体 + on-handler。
2. **`__bind_` 前缀**（区别于用户内联 lambda 的 `__evt_`）标记自动同步铸名；
   **织入 widget 名**防 registry 级 `input_state_map`（仅按 handler 名索引）
   跨兄弟子组件同名冲撞——否则两个子组件各铸 `__bind_oninput_1`，
   first-wins 会让第二个绑定静默失效。
3. **Vue 四点抑制**（v-model 已实现同步语义，铸名是纯噪音）：
   `@input` attr 两处发射点（含 Plan 399 Phase 12 的 side-effect 分支）、
   空函数（原会发 `// TODO` 桩）、defineEmits 条目（含 `has_emit` 置位
   条件化）、尾部 emit 过滤。抑制后 Vue 产物与旧裸 value 形态**逐字节同**
   （gallery_golden 基线零变化实证）。
4. **语料迁移**：005/010/013/017/459 删三件套（8 处自赋 + 9 变体 + 9 绑定）；
   005 的清错副作用（`if .email != "" { .email_error = "" }`）以 B 的内联
   lambda 保留；**015 不迁**（搜索走 registry `nav` 组件的
   search_value/onsearch 协议 + payload 消费 store.Search，非本样板）；
   015-settings 类多级点（`.store.me.image`）天然不铸（单向维持）。

### C.3 测试（四路 + e2e 双实证）

- parser 单测 `test_bare_value_input_mints_oninput_sync`：铸名/守卫
  （显式 oninput 抑制、多级点不铸、无 value 不铸）/空体 handler/变体登记。
- vue 单测 `test_bare_value_input_folds_to_vmodel_without_mint_noise`：
  v-model 折叠 + `__bind_`/`@input`/`defineEmits`/`TODO` 四零断言 +
  显式路径回归（@input 保留）。
- rust 单测 `test_bare_value_input_rust_codegen`：枚举变体 + match 臂 +
  `last_input_text()` 注入。
- vm 单测 `plan448_bare_value_input_vm_writeback`：真实 parse→decl→
  DynamicComponent 链，input_state_map 映射 + `on_with_input` 落盘。
- **MCP e2e**（`auto run -r vm`）：005 输入邮箱→Sign In→仅密码错误出现
  （写回 + 内联副作用双证）；010 纯裸 value 输入 "Zed"→快照 `value: "Zed"`
  （handler 显示 `.App.__bind_oninput_1`）。
- 全量回归：默认 lib 3342 绿；ui-iced lib 4374 绿（17 失败与 master
  预存/flaky 集完全重合）；gallery_golden/ui_snapshots 基线不变；
  vue_capabilities 77+5（同 master 预存）；docs_gen 4 绿；test-trans
  失败集与 master 逐名对照重合（a2r 预存簇）。

### C.4 边界与不做

- 只覆盖 Element 形态 `input`/`textarea`；registry 组件（ComboboxInput/
  FormControl 等）不铸——它们有自己的 value/change 协议。
- view-fn 片段内的裸 value 输入不铸（沿 B 的 v1 边界，`__evtf_` 同理）。
- 多级点 value（`.store.me.image`）保持单向（写回三路都只认直连字段）。
- checkbox/select/slider 等其他表单控件的 `checked:`/值协议不在本轮
  （未走查到样板痛点，待后续示例收集）。
- 与 B 的 `__evt_oninput_N` 一样，同 widget 内联 lambda 铸名跨兄弟子组件
  仍有理论冲撞面（registry 索引机制限制）——本轮只对新铸名织入 widget 名
  消解，`__evt_` 族维持现状（记入 C.5）。

### C.5 遗留与风险

- `__evt_*`（B 族铸名）未织 widget 名——registry `input_state_map` 若被
  两个兄弟子组件的同名内联 lambda 触发同题；低概率（用户显式命名惯例
  分散），留待实际案例再修。
- 迁移后 013/017 的 vue-tsc 构建失败为 master 预存（错误签名逐条对照
  相同），非本条引入；013 的 store TS2345 与 017 的 timer/回调类型错
  属既有债务（另案）。

---

## §4/§5 需求 D：style 组合能力 ✅ 已实施（2026-09-02，worktree @ 278ea1cc3）

### D.1 动机与证据

- `style: if .dark_mode { "A" } else { "B" }` 全仓 **143 处 / 13 文件**
  （006/008/009/010/011/013/015/018/024×4）；016-028 批次走查补充两类极证：
  - 018-settings 三对字号按钮——每对 A/B 仅差 2 个类
    （`bg-primary text-primary-foreground` vs `border … hover:bg-accent`），
    基座 `px-4 py-2 rounded-lg text-*` 完全重复；
  - 024-charts 四组件 ×4 图例行——基座全同，仅差 `opacity-40` 一个类。
- 拼接形态 `"基座" + if {…} else {…}` 语法层可编译，但两处断链：
  Vue 侧 Plan 043 H5 分类把非字面量/非 if 表达式一律发 `:style`（内联
  CSS 语义——Tailwind 类串被浏览器当无效 CSS 丢弃）；VM 侧
  `resolve_expr_to_string_with` 无 `Bina` 臂——一律空串兜底，拼接 style
  在 iced 轨**静默消失**（均 2026-09-02 scratch 实证）。

### D.2 方案（裁定 (b) 数组形态为主语法 + (a) 结构判定为辅）

1. **主语法 `style: [部分, …]`**（数组=显式类段列表，无歧义）：
   - VM 求值器（`ui/aura_view_builder.rs` `resolve_expr_to_string_with`）
     增 `Expr::Array` 臂：各段递归求值后**单空格 join，空段跳过**（条件段
     可能为 ""，`["gap-1", if .v { "" } else { "opacity-40" }]` 不产生
     双空格）；元素支持 Str/If/Ident/Dot 引用（数组形态即类语义，引用
     放行——016 `class: cell.style` 同诉求）。
   - Vue **三发射点**各增 Array 臂，统一产
     `:class="['字面量', (三元), 绑定].filter(Boolean).join(' ')"`：
     `extract_classes`（普通元素）、`push_style_class`（shadcn/registry
     路径）、`layout_dynamic_style_attr`（布局原语 row/col——Plan 458 的
     If→:class 同型扩展；第三点为首验现场：024 图例行走此路径，前两点
     覆盖不到）。
2. **辅助：class-safe 结构判定**——手工拼接 `"基座" + if {…} else {…}`
   在两发射点经 `is_class_safe_concat`（Str/Str-if 之 `+` 递归闭包）判
   定后也落 `:class`；**含引用的拼接维持 H5 的 `:style`**（`"color: rgb("
   + r` 与 `"a" + .cls` 结构不可分，引用段请用数组形态）。VM 侧 `Bina(Add)`
   臂顺带修复（字符串拼接求值，非仅类用途）。
3. **语料迁移**（19 处）：018-settings 3 对（基座提取进数组首段）+
   024-charts 四组件 16 处（`["items-center gap-N", if .vis { "" } else
   { "opacity-40" }]`）。010 的数十处**不迁**——其 A/B 两串多处不同，
   属语义色令牌迁移范畴（512/515 线），数组形态无收益。

### D.3 测试

- vue 单测 `test_style_array_and_class_safe_concat`：数组 join（含
  text/row 圆括号两形态与布局路径双命中）、字面量段引号、引用段进
  join、class-safe 拼接→`:class`、ref 拼接保 `:style`（H5 语义不回归）。
- VM 单测 `plan448_style_array_and_concat_resolution`：真实
  parse→extract→VmBridge→AuraViewBuilder 链，三行断言——数组空段跳过、
  字面量数组 join、拼接随状态翻转（on/off 两态）。
- 024 regen：产物 `:class="[…].filter(Boolean).join(' ')"` 形态；vue-tsc
  错误签名与 master **逐条相同**（50 错同分布，预存簇零新增）。
- 018 regen：被预存 S002（theme-toggle 未知元素，master 同错）挡在
  生成后校验——解析/抽取已过（校验为后置），形态由单测同款源覆盖。

### D.4 边界与不做

- `class:` prop 的数组形态未做（v1 只 style；Vue 原生 `:class="[…]"`
  可直通，VM 侧引用走既有机制）。
- 原生 Rust（a2r）路径 style 仅认字面量——If 形态今天同样不支持，
  数组继承该预存边界。
- ark/jet/kotlin 次要后端：style 以字符串消费，数组形态经各自 expr
  求值器——未加臂，等同 If 形态的现状边界（未回归验证）。
- view-fn 片段内的数组 style 未单独验证（沿 B/C 的 v1 片段边界）。

### D.5 遗留与风险

- 手工拼接 + 引用的类语义仍不可表达（结构不可分）——文档口径：引用段
  用数组形态；`is_class_safe_concat` 的判定窄（Str/If 闭包），误伤面
  为零但覆盖面有限。
- 024 图例在 VM 快照/vtree 不打印 class 字段（打印策略），VM 侧正确性
  由求值器单测锁定；如需 e2e 视觉级验证待快照通道补 class 输出。

---

## §6 走查观察（未立项，防止丢失；2026-09-02 第二轮走查更新）

- **E：str 充当 bool/状态机**——012 `running str="false"`、020
  `is_playing str="Playing"`、021 `view_mode str="list"`、019 死
  `CategoryChanged -> { .chip1 = .chip1 }`、020 死 `SeekChanged ->
  { .progress_val = .progress_val }`。**根因半明**：025 注记证实其一为
  watch 门控规避（"vue.rs 生成 watch(running) 无 immediate，需要一次
  变更才启动定时器"）；其余为作者惯性。若立项，方向是 watch immediate
  语义 + 语料 bool 卫生，非新语法。
- **F：手工展开的标量列表族（升级：能力已存在，纯语料迁移）**——010
  faq1-5（10 var）、012 lap1-3、016 星期头 7 行、019 chip1-5 +
  vid1-6（24 var）、020 track1-5/artist1-5（10 var）、021 blog1-3
  （15 var）、024 visBr0-3/legendColor0-3（编号族）。**025 反证能力在**：
  `procs = [{…}]` 内联数组字面量 + `for` 完全可用——F 是历史语料债，
  非语法缺口；随下次触碰各示例顺手迁移（连带清死 msg/handler）。
- **G：registry 组件的输入协议**——015 `nav(search: true,
  search_value:, onsearch:)`、042 `oninput: ."update:modelValue"`：
  registry 组件两向协议与裸 value 语义不一致，高频再现时可立
  "registry 组件统一 bind 协议"条目。
- **016 msg 缺 Init**——`msg { PrevMonth, … }` 而 on 块有 `.Init`：
  VM/Vue 路径无碍（handler 按名合成），原生 Rust 路径枚举外 handler
  静默跳臂（rust.rs 既有约束）；可选"on-handler 无 msg 变体编译期
  警告"lint 条目，与 013（msg 有 Init）形态不一，属语料卫生。

### 三轮走查补充（2026-09-02，028 之后批次：038/041-044/p 系列）

- **C 迁移补遗清单**（条目 C 落地后仍存的三件套/空体残例，均小件）：
  - 043-clipboard-bridge `.DraftChanged -> { }`——空体三件套，**非测试
    载体，可干净迁移**（裸 `value: .draft`）。
  - p507 `oninput: .Rename`（三件套）+ 同文件裸 `textarea (value: .name)`
    （C 后已自动双向）——tier-coverage 载体，迁移无风险。
  - 042-two-inputs-child `.UserChanged/.PassChanged`——注释标明为
    Plan 483 musk 缺陷**镜像载体**（v-model 语义 marker），迁移前需
    裁定是否保留原形态。
  - p493 `oninput: .Dummy`（空体三件套）、p051 `oninput: .SetB($event)`
    （payload 形态）——PLAN-493/051 复验器具，形状即测试对象，**不动**。
- **legacy `msg Msg {`/`msg B {` 命名形态**残存于 042/043/p493/p515——
  A 兼容窗口内的语料卫生（关闭窗口时统一迁移，见 §1 A.4）。
- **041 头注自认 vm 组件边界三缺口**（Plan 449 调查结论，README「vm
  组件边界」）：vm 下组件子树对快照不可见、回调 props 使组件整体退化、
  view fn 片段的参数化条件不求值——VM 能力债非语法项，高频受阻时另立
  （与 G 的 registry 协议条目可能合流）。
- **038 `oncontextmenu.prevent: .Flag(...)`**——事件修饰符 + payload
  实参组合在两后端工作正常（走查确认，非缺口）。

---

## §7 需求 H：computed 块体/派生态短板（已登记未实施，2026-09-02 走查收集）

### H.1 动机与证据（016-028 批次）

- **016-calendar**：PrevMonth/NextMonth/Today/SelectDay 四个 handler 各自
  重复 `.month_label = month_name(.month); .days = build_month_grid(.year,
  .month, .today, .selected_date)` 重算链——`days`/`month_label` 事实上是
  `.year/.month/.today/.selected_date` 的派生态。
- **025-dashboard 注记自认**："几何/排序为 handler 内联重算（模块级 fn
  不进 vue SFC，437 §0.6.E-3），与 024-charts 同款已知重复模式"——
  `cpuLine/cpuArea/…` 六个几何串在每个数据 handler 里重算。
- **018 双页 NOTE 自认**："the vue codegen does not emit a `return` for a
  block-bodied computed, so it would yield undefined"——作者被迫把查找
  结果存 model 字段而非 computed。**待验证**：vue.rs:8260 的 Plan 043
  M5 #2 声称发 `{ stmts; return tail; }`（复用 transpile_handler_body），
  与 018 注记矛盾——实施时先裁决注释与实现孰新。

### H.2 裁决记录（2026-09-02 探针实证）+ 分阶段方案

**裁决结论：018 注记正确，M5#2 注释与实现脱节；两侧各断一处且修法不同。**

- **Vue 侧（一行修）**：`expr_to_js` 的 `Expr::Block` 臂注释声称产物
  `{ stmts; return tail; }`，但调用的是 `transpile_handler_body`（原样
  渲染语句，**从不生成 return**）；真正实现"尾表达式语句转
  `return e;`"的 `transpile_body_as_return`（Plan 043 H1 为 IIFE 建）
  在同文件从未被用上。探针实证（`doubled => { let base = .w * 2;
  base + 1 }`）产物为 `computed<any>(() => { let base…; base + 1; })`
  ——尾语句是裸表达式，**恒 undefined**。次要发现：块体形态类型推断
  落 `computed<any>`（`expr_to_ts_type` 无 Block 臂）。
- **VM 侧（中等工程）**：`eval_computed → resolve_expr_to_value` 的
  匹配**没有 `Expr::Block` 臂**——探针实证块体 computed 求值直接
  `None`（渲染为空）。不能照抄一行修：VM 求值器是纯表达式求值器，
  块体的 `let` 作用域/语句序列需要真执行语义。
- **第三条腿（修好块体也解锁不了 016/025）**：016 的
  `days = build_month_grid(…)` 是**单表达式** computed（今天语法就支持），
  真实障碍是 437 §0.6.E-3——模块级 helper fn 不进生成的 vue SFC，
  computed 内调用在 Vue 侧无函数可调。

**Phase 切分：**

- **H1（Vue 块体，小）**：Block 臂换 `transpile_body_as_return`（其
  边界语义现成：尾语句非表达式时维持 undefined，有文档）；
  `expr_to_ts_type` 补 Block 臂（委托尾表达式推断，消除 `<any>`）；
  vue 单测断言 `return` 与类型。018 两条 NOTE 形态即可改写验证。
- **H2（VM 块体，中）**：复用 handler 编译机制——为块体 computed
  合成 `fn __computed_<Widget>_<name>(__state AppState)`（体=块语句 +
  `rewrite_state_refs` + 尾表达式语句转 `Stmt::Return`），随
  `synthesize_widget_module`（及 decl 孪生路径）进同一 VM 模块；
  `VmBridge::call_computed_fn`（state 经 `rc_push_id` 压栈，结果解码
  复用 `call_vm_fn` 尾部的字符串解码 + 堆引用 retain 语义）；
  `eval_computed` 的 Block 分支走该调用（fn 缺失时回落 None 保持
  旧降级）。单测：parse→bridge→eval 真链路 + 状态翻转重求值。
- **H3（helper fn 进 SFC，大——独立立项，不入 448）**：016/025 的
  重算消除依赖此件；涉及 use 导入 fn 的提取/发射策略与
  PLAN-051 C3 的 VM 侧 fn 别名机制对齐，建议另开计划。
- **边界**：尾语句非表达式（if/let 收尾）时两侧维持 undefined/None
  （文档化语义，与 `transpile_body_as_return` 一致）；computed 内
  API 调用（018 实际形态）本就不适合 computed（异步），不在范围。

### H.3 方向草图（原登记，保留对照）

- (a) computed 多语句体 return 语义修复 —— 已裁决并入 H1/H2。
- (b) 模块级 helper fn 进 vue SFC（437 §0.6.E-3 的债）—— H3。
- (c) `watch … immediate` 语义（顺带治 E 的 025 规避）——未裁决，仍留。
- 与 store 模式（013/015/018 的 use xxx_store）的关系需先裁定：派生态
  归 computed 还是归 store 方法。

### H.3 测试（预置）

- vue：块体 computed 产出非 undefined；016 形态（多依赖派生）改写后
  四 handler 缩为单赋值。
- VM：computed 随依赖变更重求值（快照断言 days/month_label）。

---

## §8 需求 I：grid `cols:`/`gap:` 动态值 ✅ 已实施（2026-09-02，worktree @ 4c23f5902）

### I.1 动机与证据

- **038-minesweeper 整个棋盘块 ×3 份**（各约 48 行，beginner/intermediate/
  expert 三档），唯一差异是 `cols: 9/16/30`——仓内注释自证根因：
  "vue codegen 的 cols 只接受字面量（→ grid-cols-N Tailwind class）；
  VM 的 extract_u16 也只接受字面量（→ View::Grid{cols}）"。
- 限制点核实（2026-09-02）：vue.rs `"grid"` 臂走 `extract_int_value`
  （仅 `Expr::Int`）产 `grid-cols-N` 静态类；VM `extract_u16`（仅
  `Expr::Int`）产 `View::Grid { cols }`——动态表达式两侧都静默丢弃
  （cols 缺省）。
- 025/0507 等用 `grid (cols: 3.0, …)` 字面量正常——缺口只在"列数随
  state 变化"场景（响应式表格/棋盘/看板）。

### I.2 方案（实施收口）

1. **VM 侧**：新增 `eval_u16_prop` 助手，接入 `convert_grid` 与
   `convert_grid_tracked_ctx` 双胞胎的提取链（literal `extract_u16` →
   "columns" 别名 → **表达式求值**）；非字面量表达式经
   `resolve_expr_to_value` 重建期求值（state 变更触发重建，与 style prop
   同周期）；求值失败/越界落调用方默认（cols 1 / gap 0），**不钳垃圾**。
2. **Vue 侧**：`generate_shadcn_attrs` 的 `"grid"` 臂（真实构建路径——
   首验发现 plain 生成器不走此臂，见 I.4）——字面量保静态类零回归；
   动态 cols 发内联
   `:style="'grid-template-columns: repeat(' + (expr) + ', minmax(0, 1fr))'"`
   （动态 Tailwind 类名不被 JIT 收录故走内联样式，与 `grid-cols-N`
   展开式逐字等价）；gap 同型发 `'gap: ' + (expr) + 'px'`；已有
   `:style` 属性防御性合并。
3. **语料**：038-minesweeper 三份棋盘块合一（`cols: .store.cols`——
   store 本就随难度维护 `cols` 态），净删 ~96 行；原仓内注释记载的
   双后端字面量限制随迁移失效，替换为动态 cols 说明注记。

### I.3 测试

- vue 单测 `test_grid_dynamic_cols_gap_inline_style`（shadcn 生成器）：
  字面量保 `grid-cols-7` 静态类；动态发 repeat + px 双内联断言。
- VM 单测 `plan448_grid_dynamic_cols_resolution`（convert_grid 直测）：
  动态随状态 3→16（038 难度切换形态）/字面量 7 不变/不可解析表达式
  落 1 列兜底。
- **e2e**：038 VM 模式（MCP snapshot）难度切换 9→16→30 列，
  棋盘格数 81/256/480 逐档断言。
- 038 Vue regen：产物为单 grid + 内联 repeat 绑定；vue-tsc 错误签名
  与 master **逐条相同**（store 文件预存簇 4 错零新增）。

### I.4 边界与不做

- **plain 生成器路径不覆盖**：`VueGenerator::new()`（非 shadcn）的 grid
  走 extract_classes + 通用透传——`cols` 落成死属性 `:cols="7"`（预存
  行为，字面量时代即如此，本条不扩大战场；真实 `auto build` 走 shadcn
  路径已覆盖）。
- `columns` 别名的动态求值未接（罕见形态，literal 别名保留）。
- 求值失败的兜底裁定为**落默认**（cols 1/gap 0）而非 clamp——错误可
  观察（棋盘塌成单列）优于静默钳出错误列数。

### I.5 遗留与风险

- 动态 cols 的求值每重建一次发生（无缓存）——038 形态（81-480 cell
  重建）实测无感知开销；更大的 board 若出现性能问题再议 memo。
- vue-tsc 对 `store.cols` 的 `number | undefined` 报错属 store composable
  预存簇（038 master 同错），与本条无关。

---

## §9 后续条目

继续按示例走查追加需求 J/K/…（观察项见 §6，H 见 §7，I 见 §8），格式沿用
§1/§2（动机与证据 → 方案 → 测试 → 边界 → 风险），并在 §0 总览表登记。
