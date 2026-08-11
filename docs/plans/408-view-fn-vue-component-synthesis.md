# Plan 408: view fn → 独立 Vue 组件合成（a2vue codegen 扩展）

> **状态**: 📋 计划（2026-08-10 登记，未实施）。承接 auto-musk Plan 023（`view fn → 独立组件 codegen`）的转译器侧立项。
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

## 5. Plan 053 移植发现的 codegen 技术债（2026-08-11 登记，新 phases）

> **来源**: auto-shell **Plan 053**（ash-gui-auto 对齐 vue 原版）实施过程中发现。
> 这些项都需在 **auto-lang** 侧修复，阻塞 Plan 053 的某些里程碑或影响 codegen 开发体验。
> 作为本计划的新 phases（P5-x），独立于 Task 1-4 实施。

### P5-1 ✅ 已验证不存在（2026-08-11 复核，误判）

**原假设**: 改 `.at` 后 codegen 用 ui-cache.json 旧缓存生成旧产物。

**复核实验**: 改 `prompt_bar.at` 的 placeholder（content 改）+ ghost span style
（view 结构改），**不删** ui-cache.json 重新 codegen，PromptBar.vue **正确更新**。
UICache 的 `is_dirty`（`ui_cache.rs:82`）正确比较 `hash_string(&content)`，
缓存失效机制正常。

**真因**: Plan 053 M3 的"删缓存才工作"实际是 **stack widget 不被 codegen 识别**
（P5-7）+ 操作时序混淆，非缓存问题。UICache（`vue.rs:2594` is_dirty + hash_string）
机制正确，**无需修复**。

**优先级**: ❌ 无需实施（误判）。

### P5-2 🟡 auto clean target.rs panic

**问题**: `auto clean` 在 `crates/auto-man/src/target.rs:285` panic:
`Invalid target kind: 'root'. Valid options are: app, lib, bag, dep, device, test`。
导致无法用 auto clean 清理产物缓存（用户需手动删 `.auto/` 或 `gen/`）。

**位置**: `crates/auto-man/src/target.rs:285`。

**修复**: 处理 `'root'` target kind（跳过或视为合法）。

### P5-3 ✅ 已完成（2026-08-11）

**问题**: view fn body 里的表达式（如 `block_body.at` 的 `field.1.Text.to_float()`）
**不走 method map**，原样输出 `.to_float()`。这是 Plan 053 M1（字符串方法映射补全）
未覆盖的**第三条方法映射路径**（另两条已覆盖：`vue.rs expr_to_js` 模板/computed、
`ts_adapter` handler body）。

**位置**: view fn 表达式生成处（`aura/extract.rs` 或 `vue.rs` view fn 处理，需定位）。

**修复**: view fn 表达式生成时复用既有 method map（`to_float`→`parseFloat` 等）。

**解锁**: Plan 053 B4（MemoryInfo Progress 数值兜底）。

**证据**: Plan 053 M6 试过 `value: field.1.Text.to_float()`，产物 BlockBody.vue 报
`Property 'to_float' does not exist on type 'string'`。

**实施**（2026-08-11，branch `auto-shell` commit 见 git log）:
- `vue.rs` 抽出共享方法映射表 `map_method_to_js`（Plan 053 M1 全表），接入三条
  view 表达式路径:
  1. **bound 位置** `expr_to_vue_bound_value` 的 `Expr::Call`（此前 Dot 方法调用
     原样输出）—— 覆盖 `block_body.at` 的 `value: field.1.Text.to_float()`（B4 直接依赖）
  2. **文本插值位置** `expr_to_vue_text_raw` 的 `Expr::Call`（此前仅
     to_string/len/contains 特判）
  3. **view if 条件** `convert_condition`（字符串 token 替换，覆盖 spaced/紧凑形式，
     含无参方法 to_lower/to_upper/trim_left/trim_right/is_empty）
- `expr_to_js` 的旧内联 match 收敛到 `map_method_to_js` 单源（行为不变）。
- 加单测 `test_plan053_p53_view_expr_method_mapping` 覆盖 bound + if 条件两路。
- 验证：`field.1.Text.to_float()` 产物 `Number(parseFloat(field[1].Text))`，
  vue-tsc BlockBody 4 个 TS2339 → 0；vite build 通过。**Plan 053 B4 随之完成**。
- 遗留：view 表达式路径不做 `contains` 的 R010 receiver 类型门控（与 ts_adapter 对齐，
  现行为与修复前一致，`expr_to_js` 的门控保留）。

### P5-4 🟢 纯 module fn 文件不被 codegen

**问题**: 只含 module fn（无 widget/store）的文件被 codegen 跳过（warning:
`No widget or store declarations found in input file`）。无法建 `lib/` 工具模块。

**位置**: codegen 的文件扫描/模块识别。

**修复**: 支持纯 module fn 文件（生成 lib 工具模块供 import，或允许 module fn 独立导出）。

**workaround**: module fn 放进 widget/store 文件内（Plan 053 abbreviate 改内联）。

**优先级**: 🟢 低——有 workaround，影响代码组织整洁度。

### P5-5 🟡 textarea 加入 user_class_skip_elements（解锁 Plan 053 M4）

**问题**: `vue.rs` 的 `user_class_skip_elements` 含 `"input"` 不含 `"textarea"`，
导致 textarea 被强加默认 `border rounded px-2 py-1`，无法做透明多行输入框。

**位置**: `crates/auto-lang/src/ui_gen/vue.rs:4909-4918`（`user_class_skip_elements`）。

**修复**: 列表加入 `"textarea"`。

**解锁**: Plan 053 M4（多行续行检测）的 input→textarea 切换。

### P5-6 🟡 input handler debounce codegen 注入（解锁 Plan 053 M5）

**问题**: `.at` 完全无定时器（`native_catalog.rs` 全表无 setTimeout/setInterval/debounce），
补全 debounce 无法在 `.at` 层实现（`auto.time.sleep_ms` 是阻塞 sleep，会卡 UI）。

**位置**: `vue.rs` input handler 生成处。

**修复**: input handler body 含 `complete(` 调用时，自动包一层 `setTimeout` debounce
（80ms + 序列号丢弃过期结果，逻辑照搬 vue 原版 `PromptBar.vue:60-84`）。

**解锁**: Plan 053 M5（补全 debounce）。

### P5-7 🔴 widget module fn + use 导入 + 复杂 handler 三重限制（解锁 Plan 053 M3）

**问题（三连，Plan 053 M3 实测全被阻塞）**:
1. **widget 文件的顶层 module fn 不被 codegen**（P5-4 扩展）：不只纯 module fn
   文件,widget 文件（prompt_bar.at）顶层的 module fn（tokenize）也不生成;
   只有 store 文件的 module fn 生成。
2. **use 不能导入 module fn**：`use shell_store: tokenize_input` 被误解为导入
   store（产物 `import { usetokenize_inputStore }`）。use 只认 store/widget/component。
3. **widget handler 的复杂 body 不生成 function**：DoTokenize handler（tokenize
   状态机:struct 数组 + push + 嵌套 for/break + 7 分支 if/else if 链,~80 行）
   静默未生成 function（同文件其他简单 handler 正常生成）→ 产物 TS2304
   Cannot find name 'DoTokenize'。

**位置**:
- module fn 扫描:codegen 文件处理（widget vs store 的 module fn 区别）
- use 解析:use 的 item 类型识别（store/widget vs module fn）
- handler 生成:`vue.rs` / `ts_adapter` 对复杂 handler body（struct 数组 push +
  嵌套循环 + 长 if/else if）的生成路径

**修复**:
- widget 文件也生成顶层 module fn;或支持 module fn 跨文件 use 导入
- handler 生成支持复杂 body（struct 数组 + push + 嵌套循环 + 长 if/else if）

**解锁**: Plan 053 M3（输入框语法高亮）。tokenize 逻辑已保留在 prompt_bar.at
为死代码（DoTokenize handler）,待本项修复后连接 view overlay。

**证据**: PromptBar.vue 生成了所有简单 handler function,唯独 DoTokenize 缺失。

---

## 6. P5 phases 实施优先级建议

1. **P5-7**（widget module fn + 复杂 handler）—— 解锁 Plan 053 M3，且是 codegen 表达能力的关键扩展 ✅ 已完成
2. **P5-3**（view fn 方法映射）—— 解锁 Plan 053 B4，且是方法映射一致性的补全 ✅ 已完成（2026-08-11，见 §5）
3. **P5-5**（textarea）+ **P5-6**（debounce）—— 随 Plan 053 M4/M5 推进时实施
4. **P5-2**（auto clean panic）—— 真问题但影响小（手动删可绕过）
5. **P5-4**（纯 module fn 文件）—— 低优先，有 workaround（P5-7 部分覆盖）

> **P5-1（ui-cache 缓存失效）经 2026-08-11 复核为误判，已移除**（见 §5 P5-1）：
> UICache 的 is_dirty + hash_string 机制正常，改 .at 后产物正确更新。
> M3 的"删缓存才工作"症状实际是 P5-7（stack widget 不被识别），非缓存。
