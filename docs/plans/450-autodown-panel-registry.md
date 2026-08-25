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

## 验收（批次一）

- `cargo check -p auto-lang` 绿；
- registry 测试：新 widget 可解析（lookup by name/alias）；
- auto-down 侧 palette map regen 零漂移 + 测试绿 + 对齐表一致。
