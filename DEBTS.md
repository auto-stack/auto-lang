# DEBTS — 债务与风险台账

> 单一账本两种类型：`风险/绕道`（archive-plan 归档审查登记——已完成工作内部的
> 绕道/风险）；`延期`（finish-plan 登记——因硬根因延期未做的任务）。
> 行格式见下；新增条目理由落在提交信息，不写本文件注释。

| Plan | Type | Category | Severity | Description | Root Cause | Reference | Logged |
|------|------|----------|----------|-------------|------------|-----------|--------|
| 450 | 风险/绕道 | 未来增强 | 📋 | `VueGenerator.uses_autodown`/`with_uses_autodown` 无生产调用方（死代码）：R003 Info 提示（AutoDownEditor 模板用而 main.ts 疑缺 CSS 导入）永不发射；生产门在 auto-man `parse_npm_deps` → `generate_main_ts` 注入，已接线。 | 为 Info 级提示穿透 ui_build_* 全签名不值当（plan-450 批次四裁定）；接线待面板组件化或 R003 升级为硬校验时一并做 | `crates/auto-lang/src/ui_gen/vue.rs:824`（with_uses_autodown 零 caller） | 2026-08-26 |
