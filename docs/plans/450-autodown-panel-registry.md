# Plan 450：AutoDown 面板 widget 登记（019 批次一，跨仓互链）

> 状态：**执行中（2026-08-26 立项）**。上游互链：auto-down 仓
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
   （Codeblock 走 413 Rich span 关键字高亮经验）。
2. codegen 臂重定向：AutoDownEditor spec 的 vue 消费确认经 uses_autodown 门
   （pac.at npm_deps）；ark TextArea / jet OutlinedTextField 移动端降级不变。
3. palette_map.at a2r 发射 + 对拍：auto-down 本仓侧职责（autodown-core crate 就位）。

## 验收（批次一）

- `cargo check -p auto-lang` 绿；
- registry 测试：新 widget 可解析（lookup by name/alias）；
- auto-down 侧 palette map regen 零漂移 + 测试绿 + 对齐表一致。
