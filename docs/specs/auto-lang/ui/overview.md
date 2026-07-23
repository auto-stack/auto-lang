# ui（AURA / UI 引擎）

> **Status**: partial（骨架待蒸馏，Phase 1）

## 职责

AURA UI DSL 的实现与代码生成：ui/ 运行时（app/component/event_router/gpui/headless）、
ui_gen/ 目标生成（ark/jet/block/kotlin/ts）、a2ui/、aura/ schema 处理。
与 `stdlib/aura/`、`schema/aura.at`、`packages/widgets` 共同构成 UI 生态。

## 关键入口

- `crates/auto-lang/src/ui/`、`ui_gen/`、`a2ui/`、`aura/`
- `schema/aura.at`（AURA schema 定义）

## 蒸馏来源（Phase 1）

- `docs/design/08-ui-systems.md`
- `docs/plan-reports/`（UI 相关主题）
- `examples/ui/001–025` 教学序列
