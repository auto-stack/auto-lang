# PLAN-802 final review R4 — needs_fix

2026-10-17 / plan_revision=1 / review_scope=final。当前 Codex 会话独立于实施
上下文，沿用前轮审查上下文；未创建agent，未修实现，未合入。

## 基线、复用与本轮核查

- 原树 `D:/autostack/.wt/lang-802/auto-lang`，branch `plan-802-dev`。
- HEAD `b32d4df7d3a4a43746a22ec4b8d41d1856dc9ab2`；base `1dd123361a75aa4b683376dcb2049299122724c9`。
- sibling auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- main入场 `fe09d5e5e9efaa44d9832f84b7d9d7da85ef4e74`。
- 原树 status空；diff-check通过。HEAD/依赖与R3完全相同，无Rust或测试配置变化。
- 当前二进制SHA256 `D916FF74D4FE7B2D2D1CFBE1B101F41C847E0C29CA7A1EF51981066CB47158F8`，与新work实机JSON一致。
- 本轮只改验收driver/证据/簿记。明确复用R3独立consumer7/7、定向27/27、fmt PASS及R2 no-ui metadata：源码提交、依赖、配置与相应测试输入未变；本轮未重新运行这些Rust检查，未跑整档t/tv/tf。

本轮读取当前Plan、R3修复诊断/闭合表、新driver及完整实机JSON；重放
新五态 `--selftest`（PASS：0.5s通过，2.5s迟到/tick停滞/错误上界/缺Reader
均正确拒绝；review-r4-selftest.log.txt）。另独立补足有查询耗时的计时
反控，复现同一开放finding，见下。

输入SHA256：

| 输入 | SHA256 |
|---|---|
| Plan入场 | 1C04FB41AC8DBE55617682F550DA6D4AD73C56B9A7F97DDEB052A8A8F1052419 |
| v06_driver.py | 83B551A9C106520CDBC9B3257AF24B8D051B089D5FC5394B34A0AA28E4594B11 |
| v06-evidence.json.txt | F3FDBE977C803DFD8B3664DE9F68C6B2F16618751EB6CF7615D480E086C4E410 |
| docs/specs/auto-lang/vm/overview.md | 9D792387AB8822D21AED0DEBA5818DCF17BE71A5AA8BE0D9F5242DC3307F0E91 |
| docs/specs/auto-lang/ui/overview.md | F7C54E4ED71BB3598F829F06FE0B97C772ACFE0827B0D81505640B4426A1A160 |
| docs/specs/auto-lang/ui/design/code-editor-session-endpoints.md | 2225A35E4C39E7FC69C57E2177FA728FA0FF4A68C4CF9AAE6B175187C39C52C3 |

## F802-R3-01 — 仍OPEN，P2；有查询耗时的超限病例仍误放行

影响 AC-03/04/05、V06、T05/06。沿用同一finding，不另立新要求或重复
打开已闭合格式/几何问题。

确有修正：通道tick间隔不再进入预算；tick/offset来自同一响应；t0前移
到输入之前；端点在循环内采样；tick推进成为必验；原2.5s零查询耗时
病例被拒绝。诊断覆盖了R3的具体路径，但未闭合完整“≤2s”不变量。

仍使用 `budget=2.0+u` 判成功。u虽已改为观察迭代耗时，仍是增加可通过
窗口：测量不确定区间跨过合同阈值时应不能认证，不能一律判pass。
新的selftest mock查询近乎零耗时，不能检出该放行窗口。

独立反例使用**原run_oracle**及新MockClient，只令每次Reader查询与
press回执各消耗0.25s（虚拟时钟，不操作窗口），注入0.5s有效对照及
2.2s实际迟到。后者trace在0.5/1.0/1.5/2.0s仍旧值，2.5s首见新值，
tick105→109；u=0.5s≤0.6s抖动门，预算2.5s，最终全部必验PASS、
rec.ok_all=True。有效对照亦通过，证明反例不是整体驱动损坏导致。

复现：`python docs/plans/attachments/802/review-r4-timing-controls.py`。
脚本/原始日志在本目录；脚本exit0表示成功复现误放行，JSON分别记录
expected_pass=False与actual_pass=True。mock只测试oracle，**不证明实际
编辑器延迟2.2s**。

新work实机样本Reader和端点首见均1.273s，本身处于2s内，是有效进展；
本轮未自行重复窗口运行。不能用此正向样本替代对超限病例的拒绝能力。

修正方向：按冻结≤2s合同判定。观察到新值的保守上界在2s内可pass；若
观测/误差区间跨越2s，判不确定/不可认证并补验，勿直接增加成功门限。
保留单查询耗时/抖动的独立诊断，以及0.5s有效、原2.5s、此2.2s有耗时
负控。改善观测或取得相关事件时间戳后再给出必要证据，不要求修改
冻结合同，也不要求为此修改原Rust业务实现。

## 其余验收、SD与路由

F802-R2-02格式保持CLOSED；错误clamp与缺Reader旧例保持闭合；
F802-R1-01/R2-01完整V06时序部分仍受R3-01限制。

程序化路径现已分相说明入队/窗口/端点观察，保留renderer同drain先
note_controller_offset再安排滚动的源证据，不再将异步trace冒充亚帧
测量；本轮不再把该诚实边界列为新finding。

AC01/02维持pass（源码与R3验证条件未变），AC03/04 partial，AC05 fail，
AC06 pending。SD冻结提案沿用R2报告三条精确目标与文字、R3的规范输入
和GOAL-003/007，涉及原2s时序的完成声明尚不可发布；本轮未改canonical
Specs或ledger。没有新增Spec组件/语义合同修订。

needs_fix，executing4/8，T05/T06重开；T01..04保留完成。下一步原树work
闭合判定区间问题并更新闭合表，再送审。merge-owned裸cargo t+tv、实际
新增触面档、Spec/ledger/索引/归档/wt-guard清理保持pending，未合入交付。
