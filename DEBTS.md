# DEBTS — 债务与风险台账

> 单一账本两种类型：`风险/绕道`（archive-plan 归档审查登记——已完成工作内部的
> 绕道/风险）；`延期`（finish-plan 登记——因硬根因延期未做的任务）。
> 行格式见下；新增条目理由落在提交信息，不写本文件注释。

| Plan | Type | Category | Severity | Description | Root Cause | Reference | Logged |
|------|------|----------|----------|-------------|------------|-----------|--------|
| 450 | 风险/绕道 | 未来增强 | 📋 | `VueGenerator.uses_autodown`/`with_uses_autodown` 无生产调用方（死代码）：R003 Info 提示（AutoDownEditor 模板用而 main.ts 疑缺 CSS 导入）永不发射；生产门在 auto-man `parse_npm_deps` → `generate_main_ts` 注入，已接线。 | 为 Info 级提示穿透 ui_build_* 全签名不值当（plan-450 批次四裁定）；接线待面板组件化或 R003 升级为硬校验时一并做 | `crates/auto-lang/src/ui_gen/vue.rs:824`（with_uses_autodown 零 caller） | 2026-08-26 |
| 435 | 延期 | 未来增强 | 📋 | kitchen-sink demo 页的 playwright 视觉回归 spec 未搭（页面已生成并路由，`/kitchen-sink` 可访问）。 | gallery 应用无 playwright 测试基建（无 spec 目录/无 runner 接线），原计划 P5 即标注"待基建"；属独立基建任务非本计划收口点 | `examples/widgets-gallery/src/front/pages/kitchen-sink.at`（生成器 `ui_gen/docs_gen.rs generate_kitchen_sink`） | 2026-08-26 |
| 435 | 延期 | 已知限制 | 🟢 | D10 长尾 props 覆盖：~44% web_component 元素零 props 声明（死表退役后 schema 为唯一源，长尾仅 tag+import）。 | props 回填依赖 gallery 页手写表与人工核对，机制已建（P5b-1 三源优先级 rs > gallery 回填 > 空），渐进积累无一次性收口点 | §6.4 承诺差距表 D10；`ui_gen/docs_gen.rs scan_gallery_props` | 2026-08-26 |
