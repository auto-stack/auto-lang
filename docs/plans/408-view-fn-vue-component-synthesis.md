# Plan 408: view fn → 独立 Vue 组件合成（a2vue codegen 扩展）

> **状态**: ✅ **P1–P10 全部完成并合并 master**。P1（同文件 SFC 合成）+ P2（跨文件复用）+ P3（computed）+ P4（emit+model）+ P5（use{fn}）+ P6（prop 绑定）+ P7（动态索引）+ P8（table 原生）+ P9（computed 三元）+ P10（prop-as-handler）。§7 缺陷表 5/1+2/4/6/7/3 全部修复（缺陷 8 已由 gap 44 修复）。**仍开放**：slot（§8.1，结构性改动）、§7.7 类型收窄、pages 写盘（KNOWN-DEBT）、auto-musk 试点（§6.3，跨仓库）。承接 auto-musk Plan 023（`view fn → 独立组件 codegen`）的转译器侧立项。
> **前置**: Plan 374（已完成，Rust 模式 view fn fragment 内联展开——本计划在 Vue 路径的"内联已有、独立合成缺失"基础上扩展）；Plan 367（codegen 质量改进，view fn 顺带提及）。
> **仓库**: **auto-lang**（`crates/auto-lang/src/ui_gen/vue.rs` + `aura/extract.rs` + `ast`）；auto-musk 为验证方（023 的逃生舱渐进原生化）。
> **目标**: 让 a2vue codegen 支持把 `.at` 的 `view fn` **合成为独立 Vue 组件（SFC）**——不仅内联展开（现状），还可被多个 widget 复用、成为 `.at` 单一真源组件，替代逃生舱 `.vue`。

---

## 0. 背景与现状调研（2026-08-10 实测）

### 0.1 view fn 的两级能力

调研确认 `view fn` 在 a2vue 路径的现状分两级：

**✅ 已有 — 内联展开**（374 的 Rust 侧修复已移植到 Vue 路径）：
- `ui_gen/api.rs:405-411`：`generate_component_from_file` 注册 `ViewFragmentDecl` → `register_view_fragment`
- `ui_gen/vue.rs` 测试（`:11780-11842`）验证：`view fn RenderT` 被内联展开为 AuraNode 树，生成的 SFC 中 `RenderT` 调用处是展开后的元素（`assert!(!sfc.contains("import RenderT"))`）
- 语义：`view fn X(...)` 调用 = 宏式展开（复制片段树到调用点），**不产生独立组件/SFC**

**❌ 缺失 — 独立组件合成**（本计划的缺口）：
- vue.rs 无 `view fn → 独立 SFC` 生成路径：片段只存在于调用点内联，无法被**多个 widget 跨文件复用**
- 无法用 `use { component: X }` 引用一个 `view fn`（use 块只认 widget/逃生舱文件）
- 无法让 `view fn` 成为 `.at` 单一真源组件（逃生舱 `.vue` 才能被复用）

### 0.2 现状的证据链

| 探针 | 位置 | 结论 |
|---|---|---|
| `ViewFragmentDecl` 注册 | `ui_gen/api.rs:406-411` | Vue 生产路径已注册片段 |
| 内联展开测试 | `ui_gen/vue.rs:11780-11842` | 验证内联路径（非组件导入） |
| "合成独立 SFC" 代码 | `ui_gen/vue.rs` 全文件 grep | 无（仅 inline 相关） |
| KNOWN-DEBT 记录 | `docs/plans/KNOWN-DEBT-AND-RISKS.md` | 无 view fn 条目 |
| auto-musk 侧登记 | auto-musk `docs/plans/023-*.md` §3.1 | 依赖本能力（共用组件收敛） |

### 0.3 为什么需要独立合成（auto-musk 用例）

auto-musk 023 §3.1（2026-08-10 登记）：三视图二级导航 + 内容标题栏收敛为共用组件。当前是"各自独立组件 + CSS 统一（`inject_styles.ts` 统一规则 + `!important` 兜底）"——要消掉漂移风险、样式单一真源，需要：

```auto
// 期望：一个 view fn 定义，三处复用
view fn NavSidebarHeader(title str) { ... }
view fn ContentHeader(title str, ...) { ... }
// ChatsView / SpecsView / WikiView 各自以 .at 声明复用
```

这要求 `view fn` 能独立成组件（SFC + props + slot），而当前只有内联展开。

---

## 1. 方案

### Task 1: `view fn` 独立 SFC 生成（核心）

**目标**: `.at` 文件中的顶层 `view fn X(...)` 生成独立 Vue SFC（`X.vue`），供同文件及其他文件 widget 以 `<X .../>` 引用。

**改动点**:
1. **`ui_gen/api.rs` / `generate_component_from_file`**: 除 `WidgetDecl` 外，也把 `ViewFragmentDecl` 作为组件生成源——为每个片段调用 `VueGenerator::generate`（片段视图树 → SFC），输出 `components/<Name>.vue`。
2. **`ui_gen/vue.rs`**: 片段生成 SFC 时，把 fragment 参数映射为 `defineProps`；返回类型为 `AuraWidget`（view = 片段树）。
3. **引用解析**: 某 widget 的 view 树中调用 `X(...)` 时，若 X 是**本文件已合成的独立片段**（而非仅内联候选），生成 `<X :a="..."/>` 引用而非内联展开；同时保留"单文件内联"作为降级路径（见 §3）。
4. **`aura/extract.rs`**: `register_view_fragment` 现有机制保留；新增"片段 → AuraWidget"提取辅助（镜像 `extract_widget_from_decl`，处理 fragment 参数为 props）。

**跨文件复用**: `from_workspace`（`auto-man/src/vue.rs:1315+`）扫描 front_dir 时，若 widget 的 `use { component: X }` 命中某文件的 `view fn X`，则复用该片段生成的 SFC（ext 复制机制扩展：`view fn` 生成的 SFC 视为 ext 组件，随 use 声明复制）。

### Task 2: `view fn` 与 slot / 子组件差异

三视图共用 `NavSidebar` 需要列表骨架注入差异（聊天=session 列表、规范=section 导航、知识库=双树）。两种路径：
- **方案 A（slot）**: 片段支持 `slot: "list"` 占位，各视图以 slot children 传入差异部分（需 codegen 支持片段内 slot 透传）；
- **方案 B（props + 条件渲染）**: 片段接收 `variant`/`items` props，内部 `if variant == "chats" {...}` 分支（复用既有条件渲染 codegen）。

**建议先做方案 B**（复用现有 `if` 分支 codegen，不动 slot 机制），slot 支持作为后续。

### Task 3: 逃生舱降级兼容

- `view fn` 现有"内联展开"路径**保留**为默认（零破坏）；
- 独立合成通过显式标记启用（如 `view fn` 前加 `@component` 注解，或 `use { component }` 引用片段时自动切换为合成）；
- 生成工程仍可 `use { component: X from "xxx.vue" }` 逃生舱（向后兼容）。

### Task 4: 验证 — a2vue golden + auto-musk 试点

1. **auto-lang 侧**: 新增 a2vue golden 用例（仿 `test_a2vue` 基建，Plan 022 Phase 3 建立）：`view fn Card` + widget 引用 → 期望产物含 `<Card>` 组件 import + props 绑定（vs 现有 inline golden 用例对比）。
2. **单测**: `cargo test -p auto-lang` 全绿（含新增 golden）；既有 `test_view_fragment_inline`（`vue.rs:11780`）保持。
3. **auto-musk 试点**（023 §3.1 的先行验证）: 用一个简单场景（如 UserMessage / 单个二级导航 header）以 `.at` view fn 复用替换逃生舱，验证三视图共用组件可落地。

---

## 2. 实施顺序

1. **Task 1**（独立 SFC 生成）— 核心，先支持"同文件片段独立成组件"
2. **Task 4**（golden + 试点）— 与 Task 1 并行验证
3. **Task 2**（slot/props 差异）— 支撑三视图共用 NavSidebar
4. **Task 3**（降级兼容）— 收尾，确保零破坏

---

## 3. 风险与降级

| 风险 | 级别 | 降级 |
|---|---|---|
| 独立合成破坏既有内联行为 | 🟡 | 保留内联为默认；合成显式标记启用（Task 3） |
| 跨文件片段复用复杂（ext 复制/解析顺序） | 🔴 | 先做同文件合成；跨文件复用走 `use { component }` 引用已合成 SFC |
| slot 机制成本高 | 🟡 | 方案 B（props + 条件渲染）先行 |
| 片段参数 → defineProps 类型映射 | 🟡 | 参数一律 `any`（对齐现 api.ts `any` 策略），类型严格度后续迭代 |
| 与 auto-musk 023 协调（跨仓库） | 🟢 | 023 已登记依赖；本计划完成 P1-P2 后回填 023 状态 |

---

## 4. 与既有计划的关系

- **Plan 374**（已归档）: Rust 模式 view fn fragment 内联展开——本计划的"内联已有"基础；Rust 侧如需独立合成可同思路扩展（本计划先聚焦 Vue）。
- **Plan 367**（codegen 质量改进）: view fn 顺带提及；本计划是其在 Vue 组件化方向的深化。
- **auto-musk Plan 023**（`docs/plans/023-view-fn-component-codegen.md`）: 本计划是其转译器侧立项（P5 共用组件收敛的直接依赖）。
- **KNOWN-DEBT**: 本计划完成后，若部分场景仍需逃生舱（如 useStreamingDocument 增量 JSON），登记残留。

---

## 5. 实施记录（2026-08-10，P1）

**落地范围**: Task 1（同文件合成核心）+ Task 3（降级兼容，设计自带）+ Task 4（golden 验证）。Task 2（slot）经评估由"方案 B（props+条件渲染）"免费获得——component fn 接收 variant/items props，内部 `if` 分支复用既有条件渲染 codegen，无需新机制。

**触发机制**: 新增 `component fn` 关键字（与 `view fn` 形成清晰二分），而非注解。`view fn` 语义完全不变（内联默认），`component fn` 显式 opt-in 独立合成。

**改动清单**:
- `ast/ui.rs`: `ViewFragmentDecl` 加 `is_component: bool`。
- `dialect/ui.rs` + `parser.rs`: 新增 `component fn` 关键字路径；`parse_fragment_decl_body_tail(is_component)` 共享 view fn / component fn 的参数+body 解析。
- `aura/extract.rs`: `extract_view_node` 命中片段后按 `is_component` 分流——true 返回 `AuraNode::Component`（组件引用），false 走原内联；新增 `extract_widget_from_fragment`（params→props + body→view_tree）、`fragment_param_type`、`fragment_to_component_node` 辅助。
- `ui_gen/api.rs`: `generate_component_from_file` 收集 `is_component` 片段成 AuraWidget（生成独立 SFC）+ 名字并入 sub_widgets（同文件 widget 可 `<Name/>` 引用）。
- 测试: `test/a2vue/007_component_fn/`（input.at + App.expected.vue + Card.expected.vue）+ `test_a2vue_component_fn`（走完整生产路径 `generate_component_from_file`）。

**验证**: `cargo test -p auto-lang --lib "ui_gen::vue"` 181 全绿（含 7 个 a2vue golden + view fn 内联测试）；零回归（既有失败 dstr/route/ark/vm 在 master 基线同样失败，与本项目无关）。

**残留（后续迭代）**:
- 跨文件复用（`use { component }` 引用他文件 component fn + ext 复制）——需联动 auto-man `copy_ext_files`，§3 标 🔴，未做。
- legacy 入口（`ui_build_shadcn_with_widgets`）不支持 component fn 合成——见 KNOWN-DEBT。
- auto-musk 023 试点验证（§Task 4 第 3 点）——待 auto-musk 侧以 `.at` component fn 替换逃生舱。

---

## 6. P2 残留解决方案（2026-08-10 立项）

P1 落地后对三个残留做了实测复核，结论与立项时不同——尤其残留 2 已随 P1 顺带解决。下面是逐个的根因、方案与范围。

### 6.1 残留 2：legacy 入口 —— 已随 P1 解决（仅文档过期）

**复核结论**: 经实测，`ui_build_shadcn_with_widgets` / `ui_build_shadcn_with_widgets_and_stores` / `ui_build_shadcn_with_sub_widgets(_and_stores)` 以及 auto-man 的 `compile_at_to_vue(_with_sub_widgets)` **全部是 `generate_component_from_file` 的薄包装**（lib.rs:4552 / vue.rs:3052）。P1 改的是后者本体（api.rs:428-438 收集 component fn），能力自动透传到所有调用方。

证据：lib.rs:4563 写盘循环已遍历 `result.all_widget_codes`——component fn SFC 在 `output.is_some()` 时会写盘。

**方案（纯文档清理，零代码风险）**:
1. 移除 `KNOWN-DEBT-AND-RISKS.md` 中 Plan 408 那条"legacy 入口不支持"的过期条目。
2. 加一条 e2e 验证测试：用 legacy 入口（`ui_build_shadcn_with_widgets_and_stores` 带 output）编译含 component fn 的 .at，断言产物含 `components/<Name>.vue`，防止未来回归。
3. 修订本节表述（残留 2 标记为已解决）。

### 6.2 残留 1：跨文件复用 —— 真实缺口，本 P2 核心

**根因（三层缺口，实测）**:

1. **parser**: `use { component: X }` 的 `from` 是强制的（parser.rs:11533 `expect_ident("from")`），无 `from` 语法直接报错。跨文件引用他文件 component fn SFC 没有声明语法。
2. **codegen**: 即便 parser 放开，`known_sub_widgets` 的 import 路径硬编码 `@/components/{Name}.vue`（vue.rs:1860），与 component fn SFC 的写盘位置一致——**这一层无需改**，只要 SFC 确实写到那。
3. **auto-man 写盘**: `from_workspace` 的写盘循环对 component fn 处理不统一：
   - front_dir 直文件路径（vue.rs:1557-1588）：**正确**，按 `widget.name` 写盘。
   - app.at 路径（vue.rs:1447）：`widget.name.to_lowercase()` —— 大小写 bug（`MyCard` 写成 `mycard.vue`，与 import `@/components/MyCard.vue` 不符）。
   - pages/ 路径（vue.rs:1507-1508）：只取 `widgets.first()`，component fn SFC 被丢弃。
   - Phase 1 预扫描（vue.rs:1392）调 `ui_build_shadcn_with_widgets`（丢弃 `all_widget_codes`），但他文件 component fn 名字**确实**会经此路径进 `sub_widget_names`（因为 Phase 1 遍历 `result.widgets`，component fn 在其中）。

**方案（auto-lang + auto-man 双侧，最小改动）**:

auto-lang 侧（3 处）:
1. `parser.rs:11533` — `from` 改为可选。无 `from` 时 `ExtImport.path = ""`，语义=引用同项目已合成的 component fn SFC。
2. `vue.rs register_ext_imports:1385` — Component 分支：`imp.path.is_empty()` 时生成 `import {sym} from '@/components/{sym}.vue'` + 注册 `ext_tag_keys`，**不**触发文件复制。
3. `aura/extract.rs` 或 AST — `ExtImport.path` 空串即代表"项目内 component fn"，无需新字段。

auto-man 侧（3 处）:
1. `collect_ext_import_files:1100` — 跳过 `path.is_empty()` 的 component（否则进 ext_file_set → copy_ext_files 去复制不存在的文件而报错）。
2. `from_workspace` app.at 路径（vue.rs:1447）— 去掉 `.to_lowercase()`，与 import 大小写对齐。
3. `from_workspace` pages/ 路径（vue.rs:1507）— 改为遍历 `all_widget_codes` 写盘（需让 lib.rs 的 `ui_build_shadcn_with_widgets_and_stores` 返回它，或新增入口）。

**MVP 范围（本 P2 先做）**: 两文件都在 front_dir 直文件层即可跑通——auto-lang parser + register_ext_imports + collect 跳过空 path；auto-man 写盘修正（app.at lowercase + pages 丢弃）。pages 写盘修正依赖 `all_widget_codes` 透传，单独评估。

**风险**: 跨仓库（auto-man），需协同测试。watch/增量路径（vue.rs:2604 `compile_at_to_vue` 丢弃 all_widget_codes）有同类 bug，但 watch 是独立路径，本 P2 先不强改。

**实施记录（P2 实测后修正）**:

实测推翻了立项时对三处 auto-man 缺口的部分预判：

auto-lang 侧（全部完成）:
1. ✅ `parser.rs` — `from` 改为可选（仅 `component` kind 允许无 from，fn/composable 仍强制）。空 path = 项目内 component fn 引用。
2. ✅ `vue.rs register_ext_imports` — 空 path 分支生成 `import {sym} from '@/components/{sym}.vue'` + 注册 ext tag。
3. ✅ 测试 `test_use_component_no_from_resolves_to_components` 验证。

auto-man 侧（实测后范围缩小）:
1. ✅ `collect_ext_import_files` — **实测无需改**：`is_local_ext_path("")` 返回 false，空 path 自动不进 ext_file_set，不会触发 copy_ext_files 报错。
2. ✅ app.at i>0 分支（vue.rs:1430）+ front_dir 直文件分支（vue.rs:1562）— 真正缺口是重新生成 SFC 时 `VueGenerator::new_shadcn()` **未带 sub_widgets**，导致同文件/跨文件 component fn 引用解析失败。已修正为 `.with_sub_widgets(sub_widget_names.clone())`（sub_widget_names 含跨文件 component fn 名，Phase 1 收集）。
3. ⚠️ app.at lowercase（vue.rs:1447）— **实测不影响**：写盘循环对非 pages 路径用第四元组 `widget_name`（原样），不读第二元组 `name`（lowercase）。
4. ⏳ pages 路径 component fn SFC 写盘 — 仍残留（见 KNOWN-DEBT），罕见场景，修复需改 lib.rs 公开 API。

**结论**: 跨文件复用的 front_dir 直文件 + app.at 路径已打通（含跨文件 component fn 名收集）。pages 路径作为残留。

### 6.3 残留 3：auto-musk 试点 —— 本轮不做（需先扩展 component fn 能力）

**复核结论（实测后修正）**: auto-musk 当前 0 处使用 component fn / view fn。原以为 `AgentAvatar.vue` 是最简首试候选，但精读源码后发现它含 **computed（`color`/`bgColor`/`textColor`/`initials`/`title` 五个 computed，含 `professionColors` 字典 + char hash fallback）**，超出当前 component fn 能力（P1 的 `extract_widget_from_fragment` 无 computed/字典/动态 style 对象绑定）。

**决策**: 本轮（P2）不在 auto-musk 做试点。理由：
1. 强行替换 AgentAvatar 要么退化（去掉颜色逻辑，组件失去意义），要么需先给 component fn 加 `computed` 字段（Plan 408 Task 2 范畴）——那是更大的设计决策，不该在试点里仓促做。
2. auto-musk 是独立仓库（`D:/autostack/auto-musk`），不在当前 plan-408 worktree 边界内。
3. 转译器层能力（P1 同文件合成 + P2 跨文件复用 + legacy 验证）已就绪并有测试覆盖。

**后续路径**: auto-musk 应用层替换作为独立后续任务，在 auto-musk 仓库推进。根据试点反馈决定是否扩展 component fn 能力（computed / 字典 / emit / slot）。三视图共用 header 收敛（023 §3.1）需 emit + slot，依赖 Task 2，排期更靠后。

### 6.4 P3：component fn computed 支持（2026-08-11 落地）

**动机**: P2 §6.3 指出 AgentAvatar 试点受阻于 component fn 无 computed。复核代码发现 computed 的**基础设施全在**——`AuraWidget.computed` 字段、VueGenerator 的 `const x = computed(() => ...)` 生成（vue.rs:1967）、widget 的 `computed { }` 块解析（parser.rs:12332）——只是 P1 的 `extract_widget_from_fragment` 把它留空了。本 P3 接通这一层。

**改动**（3 处，全部复用现有基础设施）:
- `ast/ui.rs`: `ViewFragmentDecl` 加 `computed: Option<ComputedBlock>`（仅 component fn；view fn 恒 None）。
- `parser.rs parse_fragment_decl_body_tail`: `is_component` 且 cursor 在 `computed` 时调 `parse_computed_block_inner`（复用 widget 的解析，位于 params 之后、view body 之前，与 widget 的 computed→view 顺序一致）。
- `aura/extract.rs extract_widget_from_fragment`: `frag.computed` → `Vec<AuraComputed>`（镜像 `extract_widget_from_decl` 的 computed 处理）。

**语法**:
```auto
component fn Badge(text: str) {
    computed {
        label => .text
        upper => .text
    }
    span { text .label }
}
```
→ 生成 `const label = computed(() => ...)` + template `{{ label }}`。

**验证**: `test_component_fn_with_computed`——断言 Badge SFC 含 `const label = computed(...)` + `{{ label }}`；回归断言 view fn 带 computed 会解析失败（computed 仅 component fn 合法）。auto-lang vue 模块 181 + plan408 4 + plan367 6 全绿，零回归。

**AgentAvatar 试点评估更新**: computed 能力已补，但 AgentAvatar 的 `professionColors` 对象字典字面量 + `charCodeAt` hash fallback 仍是 `.at` 语言层缺口（非 component fn 范畴）。AgentAvatar 的颜色映射仍需逃生舱 helper fn（`use { fn: professionColor }`），但组件骨架（props/computed/模板）可用 component fn 表达——属部分替换。完整试点仍待 auto-musk 侧推进。

## 7. P4：component fn codegen 缺陷修复（2026-08-11 立项，来自 auto-musk 探针）

> **来源**: auto-musk Plan 023 §3.2 component fn 能力探针（`tmp/probe-component-fn/`，4 场景实测）。P1-P3 的核心机制（合成/props/条件渲染/computed/跨文件复用）验证可用，但暴露 4 个 codegen 缺陷——其中缺陷 4（prop 作 handler）是 023 §3.1 共用组件收敛的硬阻塞，缺陷 1-3 影响产物质量。本 P4 逐个修复。

### 7.1 缺陷 1+2：同文件 component fn 调用点的 prop 绑定（字面量 + `self.` 前缀）

**现象**（探针 A，同文件 `Card` 被 `App` 引用）:
```
.at:   Card(title: .heading, active: true)
       Card(title: "second", active: false)
产物:  <Card :title=" self .heading" :active="{{ true }}" />
       <Card :title="second" :active="{{ false }}" />
```
- bool 字面量 → `:active="{{ true }}"`（双花括号，TS 语法错）
- str 字面量 → `:title="second"`（未引号，被当变量引用）
- 变量 prop → `:title=" self .heading"`（`self` 未定义 + 多余空格）

**根因**（**两条 prop 绑定路径不一致**）:

component fn 调用 tag 在 AuraNode 提取后是 `AuraNode::Component`，但 **Phase 1 单文件编译时 `known_sub_widgets` 为空**（vue.rs:15513 注释自证："Phase 1 front files compile WITHOUT known_sub_widgets"）。因此同文件 component fn 调用 tag 命中 `map_tag` 的 **PascalCase fallback → 普通元素路径**（vue.rs:3258 的 `is_known_sub_widget` 分支进不去），该路径用 **`expr_to_vue_text`**（文本模式）渲染 prop 值：
- `Expr::Bool` 经 `convert_template_to_vue` 包成 `{{ true }}`（vue.rs Str 分支的 mustache 逻辑泄漏到非文本场景）
- `Expr::Str("second")` 走文本分支返回 `second`（丢引号）
- `Expr::Dot(self, heading)` 在文本模式保留 `self.`（文本模式的 Dot 分支未剥离 self，而绑定模式 `expr_to_vue_bound_value` 会剥离）

而**跨文件复用**（探针 C）走 `ext_components` → `is_external_component` → vue.rs:3258 分支，用 **`expr_to_vue_bound_value`**（绑定模式，正确：bool→`true`、str→`'second'`、Dot→剥离 self）。这就是探针 C 干净、探针 A 中招的差异。

**解决办法**（两选一）:

- **方案 A（推荐，根治）**: Phase 1 编译时把**本文件 `component fn` 名**注入 `known_sub_widgets`。`generate_component_from_file`（api.rs）已收集 component fn 成 sub_widgets（P1 改动），但单 widget 编译入口（`VueGenerator::new`）的 `known_sub_widgets` 未带上同文件 component fn 名。修 `generate_component_from_file`：为每个 widget 的编译传入"同文件所有 component fn 名"作为 `known_sub_widgets`。这样同文件调用进 vue.rs:3258 分支，用正确的 `expr_to_vue_bound_value`。
- **方案 B（兜底）**: 普通 PascalCase fallback 路径（vue.rs:3258 之外）的 prop 绑定也从 `expr_to_vue_text` 换成 `expr_to_vue_bound_value`。改动面更大、影响所有 PascalCase fallback 元素（含 `use` 引用的逃生舱），回归风险高。

**建议方案 A**——精准、低回归，且符合"component fn 是本文件 sub_widget"的语义。

**验证**: 扩展 `test/a2vue/007_component_fn/input.at` 增加 bool/str 字面量 prop + 变量 prop 调用，断言 App.expected.vue 含 `:active="true"` / `:title="'second'"` / `:title="heading"`（无 self、无双花括号）。新增 `test_component_fn_literal_props` 单测。

> **2026-08-11 P6 落地**（实际根因修正 + 修复）：实测发现根因与上述分析略有出入。P2 的 `all_sub_widgets` 合并（api.rs:446）确实已让 Card 进 `known_sub_widgets`，import 也正确生成。但 `fragment_to_component_node`（extract.rs）把 component fn 调用产出为 `AuraNode::Component`，而 `node_to_html` 的 **AuraNode::Component 专门分支**（vue.rs:3916）用 `prop_to_attr_value`（文本模式）渲染 prop——`prop_to_attr_value` 对 Bool/Str 走 `expr_to_vue_text`（包双花括号、丢引号、保留 self），对 Dot 也用文本模式（不剥离 self）。
>
> **修复**：将 `AuraNode::Component` 分支（vue.rs:3939）的 prop 渲染从 `prop_to_attr_value` 换成 `expr_to_vue_bound_value`（绑定模式：bool→true、str→'second'、Dot→剥离 self）。只改这一处调用，不动 `prop_to_attr_value` 本体（它还被 category-section 等 3 处用，避免回归）。007 golden App.expected.vue 更新（`:title="heading"` 替代旧的 `:title=" self .heading"`）。新增 `test_component_fn_literal_props` 覆盖 bool/str/变量三种 prop。vue 181 + plan408 7 全绿，零回归。

### 7.2 缺陷 3：computed `if` 表达式的多余 IIFE 包装

**现象**（探针 D）:
```
.at:   computed { label => if .count > 0 { "有" } else { "无" } }
产物:  const label = computed<any>(() => (() => { if (props.count > 0) { return '有'; } else { return '无'; } })())
```
能跑（IIFE 立即执行返回值），但多余一层 `(() => {...})()`，且类型推断退化为 `any`。

**根因**: computed 表达式转译时，`if` 表达式被 `Expr::If` → TS 语句块（`{ return ...; }`）而非 TS 条件表达式（`cond ? a : b`）。widget 的 computed 走同一路径（vue.rs:1967 附近），故这是**既有 codegen 的 if-as-expression 缺口**，非 component fn 特有——只是 component fn 试点最先暴露。

**解决办法**: `Expr::If` 在**表达式上下文**（computed/prop 绑定，非语句上下文）转译为三元 `cond ? then : else`。需区分上下文——语句上下文（handler body）保留 `{ if ... }`，表达式上下文用三元。

**验证**: `test_component_fn_with_computed` 扩展——断言 `const label = computed<string>(() => props.count > 0 ? '有' : '无')`（三元 + 类型推断）。低优先级（能跑，仅质量）。

> **2026-08-11 P9 落地**（缺陷 3）：已修复。`expr_to_js` 的 `Expr::If` 分支（vue.rs:5535）原先一律用 IIFE `(() => { if ... return ... })()`。改为：当**每个分支 body 都是单表达式语句**（`Stmt::Expr`）时，转成嵌套三元 `cond ? then : (cond2 ? then2 : else)`；多语句分支 fallback 到 IIFE（功能正确，复杂场景仍需 block+return）。
>
> **修复点**：新增 `single_body_expr_js` 辅助（vue.rs:5250）——body.stmts 长度 1 且为 `Stmt::Expr` 时返回表达式的 JS 形式，否则 None。`Expr::If` 分支据此选择三元或 IIFE 路径。既有测试 `test_computed_if_chain_transpiles_to_iife` 更新为 `test_computed_if_chain_transpiles_to_ternary`（期望嵌套三元，不再期望 IIFE）。新增 `test_computed_if_emits_ternary`（component fn computed if → 三元，无 IIFE）。vue 186 + plan408 10 + plan367 6 全绿，零回归。**类型推断仍为 `<any>`**（`expr_to_ts_type` 对 Expr::If 未实现，属独立改进，不影响功能）。

### 7.3 缺陷 4：component fn 内部 button onclick 调用 prop 作 handler（emit 缺口）

**现象**（探针 B，023 §3.1 共用组件收敛的硬阻塞）:
```
.at:   component fn NavItem(label: str, onselect: msg) {
           button { onclick: onselect, text .label }
       }
产物 NavItem.vue:
       const props = defineProps<{ label: string; onselect: any }>()
       function ononselect(): void {     // ← prop 未被当可调用引用
           // TODO: handler not defined in on-block
       }
       <button @click="ononselect">
```
父组件侧透传正确（`onselect: .Clicked` → `@select="Clicked"` ✅），但**子组件内部**把 `onclick: onselect` 的 `onselect`（一个 prop 引用）当成未定义的本地 handler，生成空函数。

**根因**: `extract_view_node` 提取 `button { onclick: onselect }` 时，`onselect` 作为 handler 引用进入 `AuraEvent.handler`。Vue 生成阶段（vue.rs:2353）发现 `onselect` 不在 widget 的 `on { }` 块定义的 handler 集合里，于是生成 `// TODO: handler not defined in on-block` 空函数。**component fn 没有识别"handler 名 == prop 名"的情况**——即 prop 作为可调用事件回调。

这等价于**缺 emit**：子组件应通过 `defineEmits` 声明事件、内部 `$emit('select')`，而非调用本地空函数。当前 `extract_widget_from_fragment` 的 `messages: Vec::new()`（硬编码空）使得 component fn 无 emit 声明能力。

**解决办法**（三选一，按复杂度）:

- **方案 A（emit 完整支持，根治）**: `extract_widget_from_fragment` 接入 msg 块——component fn 声明 `msg Msg { Select }`，生成 `defineEmits<{ Select: [] }>()`；内部 `onclick` 绑定到 `$emit('Select')`。**这是 408 Task 2 的核心**，需 parser + extract + vue 三层改动。彻底解决 §3.1。
- **方案 B（prop-as-callback，轻量）**: component fn 内部识别"handler 名命中 prop 名"时，把 `onclick` 绑定为 `props.<propname>()`（`<button @click="props.onselect()">`）。不改 msg/emit 机制，复用现有 prop 透传。props 类型 `msg` → `() => void`（而非 `any`）。**最小改动**，覆盖 §3.1 的"传入回调"场景（但非标准 Vue emit，父组件需用 `:onselect` 而非 `@select`）。
- **方案 C（登记，暂不修）**: 维持现状，§3.1 继续阻塞，component fn 仅用于纯展示组件。

**建议**: **方案 B 先行**（解锁 §3.1 的最常见模式——共用 header 传入 click 回调），方案 A（完整 emit）作为 Task 2 正式落地。方案 B 与 A 不冲突——A 落地后 B 的 prop-as-callback 仍可作为轻量替代保留。

**验证**: 探针 B 复现为单测 `test_component_fn_prop_as_handler`——`component fn NavItem(label, onselect: msg)` 内部 `button { onclick: onselect }` → NavItem.vue 含 `@click="props.onselect()"`（方案 B）或 `@click="$emit('Select')"` + `defineEmits`（方案 A）。

> **2026-08-11 P10 落地**（方案 B）：已实施。新增 `try_callback_prop_attr` 辅助（vue.rs:9844）——当 handler 是裸标识符且命中 `prop_names` 时，返回 `@event="props.<name>(...)"`。在两条事件绑定路径（普通元素 :3686、shadcn 组件 :9185）的 `handler_to_function_call` 前短路调用，跳过 `used_handlers.insert`（避免空 stub）。P4 的 emit 声明侧（`.Variant` 引用 + defineEmits）与本节方案 B 互补：自包含事件用 msg/emit，父注入回调用 prop-as-handler。测试 `test_component_fn_prop_as_handler` 覆盖。**§3.1 共用组件收敛（NavItem 模式）解锁**。

### 7.4 缺陷 5：component fn 不支持 `use { fn }`（fn 引入）

**现象**（探针 E2，auto-musk 023 P3 试点 UserMessage）:
```
.at:   component fn UserMessage(content: str) {
           computed { html => renderMentions(.content) }
           div { html: .html }
       }
产物:  const html = computed<any>(() => renderMentions(props.content))   ← 标识符原样输出
       // 缺 import { renderMentions } from '...'                         ← TS2304 Cannot find name
```
computed 内引用的逃生舱 fn 标识符被原样保留，但生成的 SFC **不带 import 语句**。

**根因**: `parse_fragment_decl_body_tail`（parser.rs）只解析 `params + computed + view body`——**不解析 `use { }` 块**；`extract_widget_from_fragment` 的 `ext_imports: Vec::new()`（硬编码空）+ `api_imports: Vec::new()`。component fn 无任何 import 来源，故 computed/视图里引用的外部 fn 全部变成悬空标识符。

**这是 P3 试点的硬阻塞**：auto-musk 的纯展示逃生舱几乎都依赖逃生舱 fn（UserMessage→renderMentions、RawPreview→rawFileUrl/loadRawFileText、StreamingRenderer→useStreamingDocument）。无 fn 引入能力，这些组件的原生化都无法生成有效 SFC。

**解决办法**:
- **方案 A（use 块支持）**: `parse_fragment_decl_body_tail` 在 computed 之后、view body 之前可选解析 `use { }` 块（复用 widget 的 `parse_use_block`）；`extract_widget_from_fragment` 把 use 声明的 fn/composable 填入 `ext_imports`/`api_imports`，Vue 生成时输出对应 import。**完整方案**，component fn 可引入任意逃生舱 fn/composable。
- **方案 B（仅 fn，最小）**: component fn 不加 use 块语法，但允许在**调用方 widget** 的 use 块声明 fn 后，codegen 自动给同项目 component fn SFC 补 import。语义模糊（跨文件作用域泄漏），不推荐。

**建议方案 A**——component fn 作为独立 SFC 宿主，自带 use 块是自洽的语义，且复用 widget 现有解析。

**验证**: 探针 E2 复现——`component fn UserMessage(content) { use { fn: renderMentions from "..." }; computed { html => renderMentions(.content) } ... }` → UserMessage.vue 含 `import { renderMentions } from '...'` + computed 调用。

> **2026-08-11 P5 落地**（方案 A）：已实施。`ViewFragmentDecl` 加 `ext_imports: Vec<ExtImport>`；`parse_fragment_decl_body_tail` 在 params 之后、computed 之前解析可选 `use { }` 块（复用 `parse_widget_use_block_inner`，支持多块）；`extract_widget_from_fragment` 填 `frag.ext_imports.clone()`（替代硬编码 `Vec::new()`）。codegen 零改动——`register_ext_imports`（vue.rs:1354）已遍历 `widget.ext_imports` 生成 import。语法顺序：params → **use** → computed → msg → model → on → view body。测试 `test_component_fn_with_use_fn` 覆盖。**UserMessage P3 试点解封**。

### 7.5 缺陷 6+7：动态索引 + 原生 table 标签映射（P3 试点 StreamingTable 暴露）

**现象**（探针 F，auto-musk 023 P3 候选 StreamingTable）:
```
.at:   td { text .row[.col] }            // 行对象按列名取值
产物:  <td><span>{{ row }}</span><div>{{ col }}</div><div /></td>   ← 完全错位

.at:   table { thead { tr { th {...} } } ... }   // 原生 HTML table
产物:  import { Table } from '@/components/ui/table'  ← 被映射成 shadcn Table
       <Table :key="..."><thead class="bg-muted/50">...
```

**根因**:
- **缺陷 6（动态索引 `.row[.col]`）**: `.at` 的 `text` 节点对 `Expr::Index(Expr::Dot(...), ident)` 这类"对象按动态键取值"表达式解析/生成不完整——`row[col]` 被拆成多个节点。这是 view 树表达式提取的缺口（非 component fn 特有，但 component fn 试点最先暴露）。
- **缺陷 7（table 标签映射）**: `map_tag` 把 `table/thead/tbody/tr/th/td` 一律映射到 shadcn `Table` 组件族（vue.rs shadcn 注册表）。逃生舱 StreamingTable.vue 用原生 `<table>`，原生化时无法表达"保持原生 HTML 标签"——需 `native` 标记或 force_native 机制（现仅 checkbox/input/button/textarea 有 force_native）。

**解决办法**:
- **缺陷 6**: 修 view 树 `Expr::Index` 的 text 节点生成——`row[col]` → `{{ row[col] }}`（单 mustache，对象按动态键取值）。需在 `expr_to_vue_text`/`expr_to_vue_bound_value` 的 Index 分支确认动态键（Ident）场景。
- **缺陷 7**: 扩展 `force_native_elements`（vue.rs:3235）纳入 `table/thead/tbody/tr/th/td`，或加 `native` 前缀/属性让用户显式声明"此标签不映射"。

**优先级**: 🟡 中——StreamingTable 是 P3 候选之一，但不是唯一路径；缺陷 5（fn import）修复后可先原生化不依赖动态索引的组件。缺陷 6 影响所有"对象按键取值"场景，范围更广，应优先于 7。

> **2026-08-11 P7 落地**（缺陷 6）：已修复。根因实测——`text .row[.col]` 走 `has_dot_primary` 分支（parser.rs:12761），调 `dot_item()` 只解析 `.row`，**`[.col]` 残留在 token 流**里被当成后续 children/props，导致拆成 3 个错位节点（`<span>{{ row }}</span><div>{{ col }}</div><div />`）。
>
> **修复**：`dot_item()`（parser.rs:2039）在链式 Dot 解析后、赋值检查前，加 `[...]` 索引循环——每次 `LSquare` → `parse_expr` 解析索引 → `RSquare`，包装 `Expr::Index(lhs, index)`。这样 `.row[.col]` 保持单节点 → 产物 `{{ row[col] }}`。修复点是通用 `dot_item`，对所有 view 表达式（不止 component fn）生效。测试 `test_dynamic_index_dot_field` 覆盖。vue 186 + plan408 8 + plan367 6 全绿，零回归。缺陷 7（table 标签映射）仍待做。

> **2026-08-11 P8 落地**（缺陷 7）：已修复。实测发现 `map_tag` 的 shadcn 分支（vue.rs:4649 `shadcn_component_name`）把小写 `table` 映射成 `import { Table } from '@/components/ui/table'` + `<Table>`（thead/tbody/tr/th/td 子标签已是原生，只有 `table` 被映射）。
>
> **修复**：`map_tag`（vue.rs:4649）shadcn 查询前，加 table 核心标签排除——`table/thead/tbody/tfoot/tr/th/td` 直接返回原 tag（保持原生 HTML）。PascalCase `Table` 仍可通过 sub-widget/ext-component 路径走 shadcn 组件（二分清晰：小写=原生，PascalCase=shadcn）。**`col`/`colgroup`/`caption` 故意排除**——`col` 与 Auto layout 的 `col`（→ div/flex）撞名，加入会破坏既有 layout 用法（test_shadcn_map_tag 回归验证）。测试 `test_native_table_not_mapped_to_shadcn` 覆盖。vue 186 + plan408 9 + plan367 6 全绿，零回归。

### 7.6 P4 实施顺序与优先级（2026-08-11 P3 试点后修订）

| 缺陷 | 优先级 | 理由 | 方案 |
|---|---|---|---|
| 缺陷 | 优先级 | 理由 | 方案 | 状态 |
|---|---|---|---|---|
| **5**（component fn 无 use/fn 引入） | 🔴 高 | **P3 试点硬阻塞**——所有依赖逃生舱 fn 的纯展示组件（UserMessage/RawPreview/StreamingRenderer）无法生成有效 SFC | 7.4 方案 A | ✅ P5 |
| **1+2**（同文件 prop 绑定） | 🔴 高 | 阻塞同文件 component fn 试点；方案 A 改动小、低回归 | 7.1 方案 A | ✅ P6 |
| **4**（prop 作 handler / emit） | 🔴 高 | 023 §3.1 共用组件收敛硬阻塞；方案 B 最小化解锁 | 7.3 方案 B（prop-as-handler）+ 方案 A（emit，P4） | ✅ P4（emit）+ P10（prop-as-handler） |
| **6**（动态索引 `row[col]`） | 🟡 中 | 影响所有"对象按键取值"场景；StreamingTable 等表格类组件需要 | 7.5 缺陷 6 | ✅ P7 |
| **7**（table 标签映射） | 🟡 中 | 仅影响原生 table/iframe 等需保持 HTML 标签的组件 | 7.5 缺陷 7 | ✅ P8 |
| **3**（computed IIFE） | 🟢 低 | 能跑，仅质量；不阻塞试点 | 7.2 | ✅ P9 |
| **8**（computed 互引 `.value`） | 🔴 高 | 阻塞派生状态组件 | 7.8 | ✅ gap 44（06ba08ac，2026-08-07） |

**P4–P10 验收**: auto-musk 探针 A/B/D/E2/F/G 重跑后产物 TS 全绿（§7.7 类型收窄除外，见下）；新增 a2vue golden/单测覆盖字面量 prop、prop-as-handler、computed 三元、fn import、动态索引、cross-computed `.value`、原生 table。auto-lang `cargo test -p auto-lang` 零回归。

### 7.7 与 auto-musk 023 的闭环（2026-08-11 P3 试点后修订）

P3 试点实测结论：**当前不可行**——所有纯展示候选都被 P4 缺陷阻塞：
- UserMessage → 缺陷 5（fn import）
- StreamingTable → 缺陷 6（动态索引）+ 7（table 映射）
- RawPreview → 缺陷 5（fn import）+ 生命周期/正则（超 component fn 范畴）
- ChatMessage → 链式依赖（UserMessage + StreamingRenderer 均未原生化）

**P4 修复后路径**:
- **缺陷 5 + 1+2 落地** → UserMessage 可原生化（P3 首试解封）
- **缺陷 4 落地** → 023 §3.1 共用组件收敛解锁
- **缺陷 6+7 落地** → StreamingTable 可原生化
- auto-musk 侧 023 §3.2/§3.3 已登记探针结论与候选，待 P4 对应缺陷落地后按序推进。

### 7.8 缺陷 8：computed 互相引用时未 unwrap `.value`（P3 续 ErrandCard 暴露）

**现象**（auto-musk 023 P3 续 ErrandCard，2026-08-11 探针 G + 真实迁移）:
```auto
computed {
    errandStatus => getErrandState(.errands, .tc)   // computed A
    hasState => .errandStatus != None               // computed B 引用 A
    status => if .hasState { .errandStatus.status } else { "running" }  // C 引用 A+B
}
```
产物：
```ts
const errandStatus = computed<any>(() => getErrandState(props.errands, props.tc))
const hasState = computed<boolean>(() => errandStatus.value != null)           // ✅ 有 .value
const status = computed<any>(() => { if (hasState.value) { return errandStatus.status; } ... })  // ❌ errandStatus.status 缺 .value
```
TS 报错：`Property 'status' does not type 'ComputedRef<any>'`（`errandStatus` 是 ComputedRef，脚本里访问字段必须先 `.value`）。

**根因**: computed codegen（vue.rs computed→`const x = computed(() => <expr>)`）对 `<expr>` 中**其他 computed 引用的 unwrap 不一致**——比较/传参位置（`errandStatus != null`）正确加 `.value`，但**字段访问位置**（`errandStatus.status`）漏加。这是缺陷 3（IIFE）的同源问题：computed 表达式的 codegen 未统一处理"标识符是 ComputedRef 时需 `.value`"。

**影响范围**: 🔴 **严重**——任何"computed 引用其他 computed"的组件都中招。这是有派生状态组件的常见模式（ErrandCard/TaskPlanCard/GenericToolCard 等几乎所有非平凡卡片）。UserMessage 没中招是因为它只有一个 computed（不引用其他 computed）。

**关联现象（类型收窄缺口）**: 尝试用"每个 computed 独立调 fn"绕过缺陷 8 时，触发另一缺口——`.at` 的 `if getErrandState(...) != None { getErrandState(...).field }` 编译为 TS 后，`if (getErrandState(...) != null) { getErrandState(...).field }` 报 `Object is possibly null`（TS2531）。两次独立调用 getErrandState，TS 不收窄第二次的类型。**根因**：.at 的 `!= None` 分支未利用条件收窄后续同表达式调用的类型。这影响所有"fn 返回 Option/可能 null + if 判 None + 后续访问字段"模式（比缺陷 8 更普遍）。建议与缺陷 8 一并修：要么 codegen 对 `if X != None { X.field }` 生成 `?.` 或临时变量收窄，要么 .at 层引入 `?.` 操作符（缺陷 6 同源）。

**解决办法**: computed codegen 在生成 `<expr>` 时，对所有"解析为同 widget computed 名"的标识符引用，统一在**字段访问/方法调用**位置也注入 `.value`（当前只在不跟随 `.` 的位置注入）。需修 vue.rs 的 computed 表达式生成（与缺陷 3 的 `if→三元` 一并处理更经济）。

**验证**: 探针扩展——`component fn { computed { a => ...; b => .a.field } }` → `b = computed(() => a.value.field)`。ErrandCard 真实迁移作 e2e。

**优先级**: 🔴 高（与缺陷 3 合并修复）——阻塞几乎所有有派生状态的组件原生化。

> **2026-08-11 核查结论（文档滞后）**：缺陷 8 实际**已修复**——commit `06ba08ac`（Plan 012 Batch A gap 44，2026-08-07）比本节文档早 4 天落地。`expr_to_js` 的 `Expr::Dot` 分支递归转译 object 子树，命中 `computed_names` 时注入 `.value`（vue.rs:5355），故 `b => .a.field` → `a.value.field`（字段访问位置正确 unwrap）。P10 新增 `test_computed_references_computed_unwraps_value` 固化此行为。
>
> **窄残留**：多语句 computed body（IIFE fallback 路径，vue.rs:5530/5572）走 `ts_adapter::transpile_handler_body`，构造 `AuraTsContext` 时**未传 computed_names**——多语句 computed body 内引用另一个 computed 仍会漏 `.value`。单表达式 computed（含 P9 三元化的 if/else）全部正确。窄场景，后续补 `AuraTsContext.with_computed` 即可。

---

## 8. P4 实施记录：component fn emit + model（2026-08-11 落地）

> 本节是已落地的实施记录，对应 §7.3 缺陷4（emit）的 emit 部分 + model（本地状态）。§7 系列的其余缺陷（1+2/3/5/6/7）仍待逐个落地。

### 8.1 emit + model（对应 §7.3 缺陷4 的 emit 侧 + 本地状态）

**动机**: auto-musk 023 §3.1 的 NavSidebar 共用组件需要 msg/emit（ToggleCollapse 折叠交互）+ model（本地折叠状态）。P3 补了 computed，本节补 emit + model——两者都是 P3 的同构重复（纯接通，复用 widget 现有基础设施）。slot 需结构性改动，留作 P5。

**改动**（3 处，全部复用现有基础设施，codegen 零改动）:
- `ast/ui.rs`: `ViewFragmentDecl` 加 `messages: Vec<MsgDecl>` + `model: Option<ModelBlock>` + `on: Option<OnBlock>`（仅 component fn；view fn 恒空/None）。
- `parser.rs parse_fragment_decl_body_tail`: computed 之后、view body 之前，加 msg（循环，支持多块）+ model + on 解析（仅 `is_component`）。model 字段在 enter_scope 后 bind 到 infer_ctx（和 params 一起），让 on handler body 的 `.collapsed` 通过符号检查。on 在 model 字段 bind 之后解析。
- `aura/extract.rs extract_widget_from_fragment`: 填 `messages`（复用 extract_msg_decl）、`state_vars`（复用 extract_model_fields）、`handlers`/`handler_params`（复用 extract_on_block）、含 .Tick interval 提取 + .Init/.Destroy lifecycle 抽取（完整镜像 extract_widget_from_decl）。

**语法**（params → computed → msg → model → on → view body）:
```auto
component fn CollapseBtn(label: str) {
    msg Msg { ToggleCollapse }
    model { var collapsed bool = false }
    on { .ToggleCollapse -> { .collapsed = !.collapsed } }
    button { text .label onclick: .ToggleCollapse }
}
```
→ 生成 `defineEmits<{ ToggleCollapse: [] }>()` + `const collapsed = ref<boolean>(false)` + handler `collapsed.value = !collapsed.value; emit('ToggleCollapse')`；父侧 `<CollapseBtn ... @togglecollapse="Bump" />`。

**验证**: `test_component_fn_with_emit`——CollapseBtn SFC 含 defineEmits + ref + handler mutate + emit；App SFC 含 `@togglecollapse` 事件绑定；回归断言 view fn 带 msg 块解析失败（msg 仅 component fn 合法）。auto-lang vue 模块 181 + plan408 5 + plan367 6 全绿，零回归（21 个既有失败 dstr/route/ark/vm 与本项目无关）。

**Codegen 零改动验证**: `generate_sfc` 的 messages→emit_events（:1563）、defineEmits（:2044）、handler auto-emit（:2183）、state_vars→ref（:1917）全基于 AuraWidget 字段工作；父侧 `fragment_to_component_node` 已透传 events → `sub_widget_event_to_vue` 转 `@Event`。extract 填进去即自动产出。

**NavSidebar 试点评估更新**: emit + model 已补，NavSidebar 的核心交互（折叠状态 + 向上抛事件）可落地。**仍缺 slot**（列表骨架由各视图注入）——slot 需给 `AuraNode::Component` 加 children 字段 + 重写 node_to_html Component 分支（当前自闭合），结构性改动，作 P5。三视图共用 NavSidebar 的完整收敛（§3.1）需 P5 slot 落地后推进。
