# PLAN-839:r1 独立 R2 启动取消修复复审

阶段结论：**F-839-R1-01/P2 已闭合，未发现该修复的新回归。整 Plan 最终验收尚未开始/通过。** 本次是用户明确要求的修复后独立定向调查；主 Plan 仍 executing、T08 原生 consumer 必需门待闭合，不能提前授予 execution_done/reviewed。主 Plan 和债务表由父代理单写。

身份：Codex 独立代理 `/root/review839`，独立于实施上下文；未另建树/新代理，模型及会话 ID 未暴露，不编造。使用 `C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md` 与 verification-contract.md，已读主检出 AGENTS/Plan 和受审树实际实现。代码写入冻结，进入和收尾 `git status --short` 均空。

受审唯一树 `D:/autostack/.wt/lang-839/auto-lang`，分支 `plan-839-dev`，修复提交 `e44644e1fa03c126801caecfe84a8b91054ee09c`，修复前 `e58828ca64b7e26ce984faf850fb3b1d0887517f`，整体基面 `37fd1a9cfe22d3d3471202ad4a8dbfe2103f9a5c`。SDK `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`/680 文件，278 source 输入；全部 Plan、规范、两锁、源身份、探针及技能指纹见 [阶段收据](phase-receipt.json.txt)。未运行 GUI 原生门、完整 driver/AGP 或 root 任一整档。

## 独立闭合证据

| 规则与路径 | 独立核验 / actual oracle | 结果 |
|---|---|---|
| 原始四边沿；AC04/05/06、T04/06 | 原 R1 `probe/src/lib.rs` 内容原封复制（原始 SHA 保留），仅在末尾添加新测试模块。真实 Host start→失焦/resize-DPI/离窗/遮挡→完整 Auto+GPU frame。每项输出 Paused/tick0/保留具体原因，三次 20ms 后台帧均 actual Auto tick0；原3合法控制同过 | pass，原例闭合 |
| 初始化已获得 candidate、尚未 accept 时取消；同一完整事务边界 | 新 `cancellation_after_initial_candidate_clears_new_input_and_stop_discards`。Start→全部 Auto queries/候选→连续取消和新增 held/V→accept 仅 Paused/tick0、清输入/clock；显式 Resume/Step 在 Starting 拒绝。另一分支 Stop 丢 candidate，accept 报无帧，owner 释放，Editor 指纹不变 | pass，全不变量边界补足 |
| Starting→Paused→Step inflight→Running inflight；AC04/06 | 新 `canceled_start_step_inflight_pause_then_resume_never_leaks_start_latch`。neutral Step 恰 tick1、无位移/V；Step inflight 时 Resume/重复 Step 拒绝；Running inflight Pause 等实际 tick2 完成再 Paused；repeat 不复活 held，显式 Resume+新非repeat才 actual Auto tick3/x=1 | pass，兄弟路径与新回归控制 |
| 初始化晚 scalar 故障、Stop、fresh Run；AC03/06 | 新 `canceled_initialization_late_scalar_fault_then_fresh_run_uses_new_identity`。源码仅在最后 queried `field(f==11)` 真 DivisionByZero；取消仍使整笔 Faulted/无last-good/no pending/VM弱owner释放，Resume/Step拒绝；Stop 后新 Session/RunId 正常 Running/tick0/原初始文档 | pass，失败不误变 Paused/泄漏 |
| 30s 初始化与后续 2s 完整事务；AC04/06 | 通读三个私有期限测试真实断言并独立重跑。取消保持 Starting 到完整帧；模拟3s的初始化合法（大于runtime2s而小于init30s）；31s初始化在 output 前与 candidate 后 accept 均 Faulted/owner释放/no publication/latch清；取消后 Step 的3s仍按2s失败 | pass，期限未放宽 |
| 永久实现/Host接线 | `Play::pause` 的 Starting 分支只 latch+clear；`accept` 在完整验证/prepare期限之后取走 latch，进入 Paused，否则正常 Running；`fail/stop` 清 latch。Host窗口四边沿仍调用同一 pause，Starting 输入 down 不转给 Play，accept 再 clear，Resume/Step状态准入保持。永久 Host 四边沿测试含真实model/2D GPU恢复和Editor identity，失败控制含 profile/prepare 与fresh Run；实际断言已逐项审查 | pass，源码与永久覆盖相符 |

精确命令均在 [收据](phase-receipt.json.txt)，原始日志：[7项客户端](holdouts.log.txt)、[3项期限](deadlines.log.txt)。测试 10 项不同 Rust case，0 failed/ignored；原四边沿包含在一个case内，不重复加总。新客户端锁严格等于 driver 所有 name/version/source/checksum，加唯一 review package；`--locked --offline` 实际通过。首次追加模块命令误用了相对路径，工具返回明确错误；随后在正确目录追加后实际执行，未改原样例输入/产品源码。

## 机制判断与边界

修复根因正确：原 `pause` 没有处理 Starting，`accept` 无条件启动 Running。当前把暂停意图与公开 Starting 状态分开，避免提前使用 Running/Pausing 的2s期限；完整 frame准入和GPU准备仍是发布条件。`pause_after_start` 只影响首次 accept，在 Stop/Fault 和首次成功 accept 清除。没有靠删除取消规则、延长 runtime期限、跳输出、Rust代替行为或调低负控来过门。

这关闭了合同内缺陷；不扩展同步 compile/native GPU 可抢占或新平台承诺。原真实 GUI正反门由工作阶段另行执行，本阶段没有将 Host 定向调用冒充 OS事件回放。父代理所记录 consumer 的多次 GUI失败保留为未完成门：监测日志确证一次非probe物理鼠标移动/前台切换及产品 `Paused/auto-Pause:cursor left/tick0`；较早额外对象点击的具体来源没有独立证据，不予推断，缺报告/panic也不记产品通过。

SD01/03取消规则与当前代码/测试一致，冻结副本 [spec-delta-reviewed.md](spec-delta-reviewed.md)。SD02/04与完整AC01..08的最终审查需在 work就绪后正式记录；本阶段不修改canonical/ledger，不批准合入。

下一责任：work补 T08 consumer必需门/身份一致性/可复跑readiness，主 Plan转 execution_done；同一独立上下文再完成最终AC01..08/SD01..04验收。最终pass只转reviewed，原完整merge-owned矩阵、实际autographics-dev落点/规范沉淀/归档guard清理/到期检查始终 pending。
