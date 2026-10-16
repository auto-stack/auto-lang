# PLAN-802 R1 — readiness review

2026-10-17 / review_scope: final / outcome: needs_fix / plan_revision: 1。
本会话独立于实施上下文；未创建其他 agent。本轮按 review 技能的 readiness
规则止于必验缺口核查，不声称完成最终 AC/SD 全矩阵复审。

## 基线与范围

- worktree: `D:/autostack/.wt/lang-802/auto-lang`，branch `plan-802-dev`。
- 注册 HEAD: `4029dd705a1f3d4e39ddbffbb6c5f426c402c2a6`。
- base: `1dd123361a75aa4b683376dcb2049299122724c9`。
- 主检出: `99cb54c719869c88b18b8d033ffafa47cb498f75`。
- 依赖 auto-down: work 收据声明 `c39ac7f`；本轮未独立确认完整 SHA。
- Plan 入场 SHA256: `E14F07FD62717529832D94BE3F27AF321C7C70077056628BC833ABB16ED26049`。
- Spec SHA256：vm/overview `9D792387AB8822D21AED0DEBA5818DCF17BE71A5AA8BE0D9F5242DC3307F0E91`；ui/overview `F7C54E4ED71BB3598F829F06FE0B97C772ACFE0827B0D81505640B4426A1A160`；ui/design/code-editor-session-endpoints `2225A35E4C39E7FC69C57E2177FA728FA0FF4A68C4CF9AAE6B175187C39C52C3`（均位于 docs/specs/auto-lang/）。

从 main 读取当前 Plan、T01 矩阵和 T06 收据，从原实施树读取 adapter、
plan802 测试；核对 worktree 注册和改动清单。工作树 Git status 未成功：
初次报告 must be run in a work tree，显式 -C 后 Permission denied。
因此不声称实现树 clean，也不将注册 HEAD 当作全部当前文件已提交的证明。
本轮没有运行测试或修改实现，未授予任何可供合入复用的测试 pass。

## F802-R1-01 — P2，合同内必验 oracle 未完成

影响 AC-03 / V06 / T-05，并使 T-06 的 work-complete 送审声明尚不成立。

Plan §6 V06 明确要求真实 renderer drain、可见偏移/clamp 和 701 收敛；
并明确“必要实机未跑是缺口，不能标 pass”。T06 收据 §三（43–48 行）
明确承认 live iced 消费链未跑，却把补验写成“merge 前若环境允许”。
该项是 work-owned 验收，不能降成可选收尾。

独立源检查：`crates/auto-lang/src/tests/plan802_code_editor_provider.rs`
456 行 `editor_scroll_projection_chain` 在 479 行 drain 后，直接调用
`note_controller_state` 人工回填偏移（注释亦称“模拟 note-back”）。
它验证真 controller 组件，但不执行窗口的 operation::scroll_to、真实
ScrollStateReader 回填或用户滚动后的 ≤2s 收敛，不能补上这条 oracle。
此 finding 不声称实际滚动实现已经损坏，而是必需验收证据尚未交付。

修正：原树 work 补齐 autoui-verifier 实机验证，记录冻结提交、feature、
示例/输入、实际窗口操作、程序化滚动可见结果、clamp、用户滚动后读回及
≤2s 计时证据，保留未绑定反控。如环境确实不可用，保持此项开放并记录
阻塞条件；不得以组件测试或 merge-owned 门禁代替。合同未被批准修改。

## AC / SD / 路由

- AC-01/02/04：本轮未完成独立重放，pending，不继承 work 测试数为独立 pass。
- AC-03：partial，V06 窗口链证据缺失。
- AC-05：fail readiness；完整独立 AC/SD 与健康检查待 work 就绪后执行。
- AC-06：pending merge，裸 cargo t + cargo tv、实际新增触面档、规范发布、ledger、归档及 guard 清理均未执行。
- SD-01/02/03：保留 Plan §5 提案；本轮未批准发布，未修改 canonical Specs/ledger。

冻结 SD 快照（本轮仅保存现有提案，不授 pass）：SD-01 修改 vm/overview，
editor 直引 UI → 22 项 owned provider、缺席矩阵/装配/寿命；SD-02 修改
ui/overview，记录 adapter 单一 registry、宿主装配和其他族仍未迁；SD-03
修改 ui/design/code-editor-session-endpoints，provider 后 VM/a2r 同源且
单位/IO/收敛保持。目标前缀均为 `docs/specs/auto-lang/`，对应 AC 依次
01/02/03、03/04、02/03；后续完整审查仍须核对具体规范文字。

状态维持 executing，重开 T-05/T-06，current_step=4。T-01..04 保留 work
进度标记，不表示本轮独立验收通过。next: 原树 work 闭合 F802-R1-01，
重新确认提交与实现洁净度，补齐送审包后复审。没有合入或交付。
