# DEBTS — 债务与风险台账

> 单一账本两种类型：`风险/绕道`（archive-plan 归档审查登记——已完成工作内部的
> 绕道/风险）；`延期`（finish-plan 登记——因硬根因延期未做的任务）。
> 行格式见下；新增条目理由落在提交信息，不写本文件注释。

| Plan | Type | Category | Severity | Description | Root Cause | Reference | Logged |
|------|------|----------|----------|-------------|------------|-----------|--------|
| 450 | 风险/绕道 | 未来增强 | 📋 | `VueGenerator.uses_autodown`/`with_uses_autodown` 无生产调用方（死代码）：R003 Info 提示（AutoDownEditor 模板用而 main.ts 疑缺 CSS 导入）永不发射；生产门在 auto-man `parse_npm_deps` → `generate_main_ts` 注入，已接线。 | 为 Info 级提示穿透 ui_build_* 全签名不值当（plan-450 批次四裁定）；接线待面板组件化或 R003 升级为硬校验时一并做 | `crates/auto-lang/src/ui_gen/vue.rs:824`（with_uses_autodown 零 caller） | 2026-08-26 |
| 435 | 延期 | 未来增强 | 📋 | kitchen-sink demo 页的 playwright 视觉回归 spec 未搭（页面已生成并路由，`/kitchen-sink` 可访问）。 | gallery 应用无 playwright 测试基建（无 spec 目录/无 runner 接线），原计划 P5 即标注"待基建"；属独立基建任务非本计划收口点 | `examples/widgets-gallery/src/front/pages/kitchen-sink.at`（生成器 `ui_gen/docs_gen.rs generate_kitchen_sink`） | 2026-08-26 |
| 435 | 延期 | 已知限制 | 🟢 | D10 长尾 props 覆盖：~44% web_component 元素零 props 声明（死表退役后 schema 为唯一源，长尾仅 tag+import）。 | props 回填依赖 gallery 页手写表与人工核对，机制已建（P5b-1 三源优先级 rs > gallery 回填 > 空），渐进积累无一次性收口点 | §6.4 承诺差距表 D10；`ui_gen/docs_gen.rs scan_gallery_props` | 2026-08-26 |
| 438 | 风险/绕道 | 已知限制 | 🟢 | "刷新间隔调节"实为 speedDiv 分频 workaround：`.Tick` 的 setInterval 周期在解析期固化为常量（model 的 `interval` 字段被 extract.rs 消费后从 state_vars 移除），运行时无法真正调节定时器频率——025-dashboard 以 250ms 基准 + 1/4/10 分频达成三档。 | 根治需 interval 保持 state ref + `watch(interval)` 重启定时器，涉及 012/024/025 语义面（interval 从"魔法字段"变普通状态），收益有限（分频法在 UI 层不可感知差异）；按 M1-fix 先例如有第三个消费方再立项 | `crates/auto-lang/src/aura/extract.rs:667`（interval 消费点）；`examples/ui/025-dashboard/src/front/app.at`（speedDiv 分频） | 2026-08-27 |
| 041 | 风险/绕道 | 已知限制 | 🟢 | VM 难块显式降级：Mermaid（源码面板+web-only 标签，resvg 无布局引擎）/MathBlock（mono+$$，KaTeX web-only）/QueryBlock（未求值标签，求值归宿主）——不再静默段落化。 | 布局/求值引擎缺失是平台事实；显式标签即契约 | `crates/auto-lang/src/ui/autodown_render.rs`（T7 三臂）；计划 041 豁免表 #4-6 | 2026-09-03 |
| 041 | 风险/绕道 | 已知限制 | 🟢 | Details 点击折叠回路未接：VM v1 状态源=open attr（loading 强制展开/final 收起），点击→消息→状态→重渲染的消息通道归宿主。 | VM 只读臂无内部消息通道（View 树 onclick 需宿主 M）；随滚动同步契约计划接线 | `crates/auto-lang/src/ui/autodown_render.rs`（Details 臂）；计划 041 豁免表 #8 | 2026-09-03 |
| 041 | 风险/绕道 | 已知限制 | 🟢 | parser 组件指令 argstr 多参扫描仅首参可靠（$callout(type:"x") 有效，title 第二参丢失；open 须首参）。 | autodown-core a2r 发射既有行为（argValueAt 扫描器），本计划不改解析器 | `autodown/packages/core/rust/src/markdown_parser.rs` argValueAt；计划 041 豁免表 #9 | 2026-09-03 |
| 041 | 延期 | 已知限制 | 🟢 | 流式增量 v1 只在 View 装配层（结构键 diff+未变块复用+尾块重同步），解析仍全文档 reparse；untracked convert_element 臂维持全量渲染（无 path 键）；StreamCache 注册表无容量上限。 | 尾窗重解析为正交优化；tracked 主路径已增量；LRU 随滚动同步计划 | `crates/auto-lang/src/ui/autodown_render.rs` StreamCache；`aura_view_builder.rs` autodown_stream_registry；计划 041 豁免表 #10/#12 | 2026-09-03 |
| 041 | 风险/绕道 | 已知限制 | 🟡 | fence 编辑壳 mono 族用 Family::Monospace（code_editor 为 Windows CJK tofu 已改 Consolas named family）——fence 中文注释有 tofu 风险；另 hljs 主题明暗态在缓冲建立时定格，运行时切主题不刷新。 | autodown_editor core 禁 iced 分层下最小实现；T11 联测观察，对齐 code_editor Consolas 方案留债候选 | `crates/auto-lang/src/ui/autodown_editor/core.rs` mono_family/new_leaf_buffer；计划 041 豁免表 #11 | 2026-09-03 |
| 041 | 风险/绕道 | 已知限制 | 🟢 | 代码块 chrome 精修（复制/折叠按钮、语言下拉）与 CustomScrollbar/原生滚动条差异豁免——VM fence header v1 仅语言标签。 | vue 侧 039 已精修；VM 等交互控件批次（滚动同步契约计划） | `crates/auto-lang/src/ui/autodown_blocks.rs` FENCE_CHROME；计划 041 豁免表 #1/#2 | 2026-09-03 |
