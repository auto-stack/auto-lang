# Plan 408: view fn → 独立 Vue 组件合成（a2vue codegen 扩展）

> **状态**: ✅ P1 已实施（2026-08-10）。同文件 `component fn` → 独立 Vue SFC 合成已落地，golden 验证通过，既有 view fn 内联行为零破坏。承接 auto-musk Plan 023（`view fn → 独立组件 codegen`）的转译器侧立项。
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
