# Plan 450：AutoDown 面板 widget 登记（019 批次一，跨仓互链）

> 状态：**完成（2026-08-26；批次一 registry 登记 → 批次二 vue 调查 → 批次三
> iced 面板映射 → 批次四 codegen 臂确认 → P3 schema 声明面收口 57b55e8d9；
> 对拍落点在 auto-down 侧 plan 019 批次五 77a2eaf）**。上游互链：auto-down 仓
> [docs/plans/archive/019-rust-platform.md](../../../../auto-down/docs/plans/archive/019-rust-platform.md)
> （若未归档则在 plans/ 下）与 `packages/engine/PANEL-ALIGNMENT.md`——本计划即其对齐表
> "registry 待登记" 空位的 auto-lang 侧落地。

## 背景

auto-down 已完成 017（渲染统一：palette_map.at 面板词汇单源 + 注册表渲染器）与
018（编辑内核替换）。其面板词汇（Text/H1..H6/Separator/Codeblock/Quote/List/Table/
Callout/Details/MathBlock/Mermaid/Query/Embed）中 Text/Separator/Mermaid 已在
registry，其余 10 个为空位。rust 端（iced 渲染、编辑壳）消费这些面板需要 registry
登记 + codegen 臂。

## 本批次（批次一）：registry 登记

- `registry.rs` register_document_widgets()：登记 Heading（primary_prop=level，
  对齐 palette H1..H6）、Codeblock（language）、Quote、List（ordered）、Table、
  Callout（kind）、Details（summary）、MathBlock（source）、Query（query）、
  Embed（target）——WidgetCategory::Display，别名 snake_case。
- 双仓对齐：auto-down 侧 palette_map.at 的空 registry 字段回填上述名字，
  PANEL-ALIGNMENT.md 状态列更新。

## 后续批次（019 主线，另立或续本计划）

- vue backend 映射：面板 ↔ engine 面板渲染器组件名（对齐表三列补全）。
- iced backend：面板 → iced widget 映射 + Codeblock 富 span（413 经验）。
- codegen 臂重定向：AutoDownEditor spec 消费 engine 出口。
- crate 对拍：palette_map.at a2r 发射 + 对拍脚本。

## 批次二调查结论（2026-08-26，本会话完成）

- vue backend 的默认映射不走 WidgetSpec.backends，而走 **schema.meta 的 vue 元数据**
  （registry.rs apply_default_vue_mappings：schema canonical 折叠键 → component/import/npm）。
  markdown → MarkdownRender 的 import 现指 @autodown/vue——该包是 engine 的 re-export shim
  （auto-down 017），**当前仍正确**；待 020 退役 shim 时统一切 @autodown/engine。
- golden 链路：test/a2vue/002_markdown（input.at → expected.vue）锁定 markdown 臂行为。
- 面板 widgets 的 vue component 映射**暂不登记**：engine 侧面板渲染器尚非具名 Vue 组件
  （builtin-panels 是函数）；待面板组件化后经 schema.meta 补映射，避免假组件名进 codegen。

## 后续批次的准确落点（省去考古）

1. iced backend：面板 → iced widget 映射，落 ui_gen/rust.rs 渲染臂 + renderer.rs
   （Codeblock 走 413 Rich span 关键字高亮经验）。——**批次三完成（见下）**
2. codegen 臂重定向：AutoDownEditor spec 的 vue 消费确认经 uses_autodown 门
   （pac.at npm_deps）；ark TextArea / jet OutlinedTextField 移动端降级不变。
   ——**批次四确认完成（见下）**
3. palette_map.at a2r 发射 + 对拍：auto-down 本仓侧职责（autodown-core crate 就位）。

## 批次四：codegen 臂重定向确认（2026-08-26，本会话完成）

- **vue 消费链完整**：schema/aura.at `vue: { component: "AutoDownEditor",
  import: "@autodown/editor" }` → vue.rs generate_shadcn_imports 发射
  `import { AutoDownEditor } from '@autodown/editor'` → 该包现为 re-export shim
  （`export * from '@autodown/engine/editor'`，plan 017），engine 0.4.0 仍以
  `AutoDownEditor` 名义出口 `EngineEditor.vue`——链路现役正确；020 退役 shim 时
  统一切 `@autodown/engine`（同批次二 markdown 臂裁定）。
- **uses_autodown 门的真实落点**：生产门 = auto-man `parse_npm_deps`（pac.at）
  → `generate_main_ts` 自动注入 `@autodown/editor/style.css`（三调用点
  write_project_files / generate / regenerate_source_files 均已接线）。
  `VueGenerator.uses_autodown`（R003 Info 提示的开关）无生产调用方——
  `with_uses_autodown` 零 caller，且 `validate_sfc` 本身不在生产路径。
  裁定：不接线（为 Info 级提示穿透 ui_build_* 全签名不值当），记为已知死代码。
- **移动端降级不漂移**：registry AutoDownEditor backends ark → TextArea、
  jet → OutlinedTextField（androidx.compose.material3）未动，新增
  `test_autodown_editor_mobile_backends_stable` 回归守卫锁死。

## 批次三：iced backend 面板映射（2026-08-26，本会话完成）

**实现**（分解为既有 View 变体——Plan 319 单臂规则，不新增 View 变体，
renderer.rs 零改动；Codeblock 的 Rich span 高亮沿用 442 的
`StyleClass::CodeLang` → `highlight_code` 路径，无需新做）：

- VM 链路（iced 主消费路径）`ui/aura_view_builder.rs` `convert_element` 兜底区
  七面板臂：heading（level 钳位 1..6 → h1..h6 同源样式）、quote/blockquote
  （border-l 容器）、callout（kind→tint 五档配色 + title 头）、details
  （→`View::Accordion` 单项默认展开，对齐表"可对齐 Accordion 族"裁定；
  VM 降级无 toggle 回写）、math_block/query_block/embed_block（注册位面板
  → 可见源码/引用文本，不再静默丢内容）。tracked 链路经既有委托复用。
- a2r 链路 `ui_gen/rust.rs` `generate_view_tree` 同 tag 族特例：字面量
  level/kind 静态选样式，动态表达式发射全臂 match（生成代码保持纯表达式）；
  details 发射 `auto_lang::ui::view::AccordionItem::new(..).with_children(..)
  .with_expanded(true)`。tag 族此前落 `tag_to_view_fn` 的 `_ => "col"`
  fallback——内容静默丢弃，现已消除。

**裁定**：registry 不登记 iced BackendMapping（与批次二 vue 裁定同理——
无消费者，假组件名进 codegen 是噪音）；映射的实现落点即渲染行为本身。
schema 面板 elements **后补登记**（57b55e8d9，schema_drift P3 收口）：
7 元素声明面不带 vue meta（vue 组件映射仍等面板组件化，同批次二），
Query/Embed 两 PascalCase 名由生成器自旋 P3 存根（nav-destination/
swiper 先例）；if-arm 实现形态走 rs_not_in_vb/render baseline 通道
（041a 原生元素同款）。

**验证**：VM 5 测（heading 样式/钳位、quote 结构、callout tint、details
Accordion、注册位三面板内容可见）+ a2r 3 测（heading 静态/dynamic match、
quote/callout、details/AccordionItem/embed 发射串）；
`cargo test -p auto-lang --features ui-iced,code-editor --lib` 全绿。

## 验收（批次一）

- `cargo check -p auto-lang` 绿；
- registry 测试：新 widget 可解析（lookup by name/alias）；
- auto-down 侧 palette map regen 零漂移 + 测试绿 + 对齐表一致。

## finish-plan 复审（2026-08-26）

七项任务逐一对码重验全部 pass：批次一 registry（registry.rs:839，
测试 2155/2188 绿）；批次一双仓对齐（palette_map.at registry 字段 +
PANEL-ALIGNMENT.md，de9ba43）；批次二/四为调查与确认性批次（结论在册，
ark/jet 守卫测试绿）；批次三 iced 映射（aura_view_builder.rs:2082-2195
七臂 + rust.rs:2042+ 特例，VM 8 测 + a2r 3 测绿）；P3 schema 收口
（schema.rs:596-657 七元素，fence 裸跑绿，baseline +14 有意漂移）；
落点 3 对拍（auto-down 77a2eaf，crate 7 测 + engine 254 测绿，金标 16 行
panelOfBlock 投影）。验证命令全部重跑复现绿。已知遗留（裁定在册，非未竟）：
`VueGenerator.uses_autodown`/`with_uses_autodown` 无生产调用方（批次四
裁定不接线，为 Info 级提示穿透 ui_build_* 全签名不值当）；vue 面板组件
映射待面板组件化（批次二裁定）。分类：A（全部完成）→ 归档。
