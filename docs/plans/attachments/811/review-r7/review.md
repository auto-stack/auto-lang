# PLAN-811 独立复审 R7

2026-10-19（Asia/Shanghai） · stage: review · review_scope: final · **outcome: needs_fix**

受审 `04f7446fdc1d3aed9c44dba6efcc9f661ea6a31d`，Plan revision 1。上轮正控造载缺口已闭：核心模块44/44、三次独立正控全绿。**新增未满负控把“8项任务”误当作“最多7槽排队”**，遗漏接收线程尚未取首项的合法状态；受控调度直接使现存负控失败。当前仅此测试修复回归，生产代码没有新增行为变化。

## 1. 受审基线及证据复用

- 原唯一树 `D:/autostack/.wt/lang-811/auto-lang`，branch `codex/plan-811-dev`；base `ab08955c241f6a824091c53d3b02894ca17e193b`。新提交只有core `image_pipeline.rs`测试模块3个hunk，+170/-21；累计17个实施提交。
- auto-down兄弟 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，双方入场clean。受审配置/规范/Plan/SD/fix-r6/work原始收据在 [baseline.json.txt](baseline.json.txt) 冻结；结束稳定性见 [final-baseline.json.txt](final-baseline.json.txt)。
- reviewer_identity：当前用户请求的Codex复审上下文，本会话未实施；具体模型/会话ID未暴露。使用 `/auto-plan:review`，未创建agent/新树。
- [production-equivalence.json.txt](production-equivalence.json.txt)：`#[cfg(test)] mod tests`之前全部生产源码逐字等价（LF规范化SHA均`ce9cf376e1bdff2621be1b21878ed3dbac89eb8851e0945f8a6fb682f57b0762`）；其他26项实施/config摘要、依赖及canonical输入均与R6一致，复用的R6 typed消费者exe hash也一致。
- 因此明确复用R6已过的VM7/7、Man3/3、真实HTTP13/13、七个真实窗口及原在途归零/重复关闭/open/scan/Weak等生产证据；work六收据原SHA仍为2ffb87755，按生产源码/依赖/config等价复用，未篡改成当前SHA，也不冒记本轮重新驱动消费者。
- 实际构建/检查均在原树，主检出仅附件/Plan簿记；未改实施、canonical/live ledger、合入/归档/清树，未跑整档t/tv/tu/tt/tb/taa/tf或docs_gen。

## 2. 上轮正控缺口已闭

F811-R2-02的R6余量（固定12次offer、可能只接受8项，不能保证running≥9的满态前提）**CLOSED**：

1. 当前正控在24项预算与20秒墙钟内有界补入，失败offer重试，观测到原门槛running>8和backlog≥8后停止；没有降低阈值或删除满通道断言。
2. 失败路径报告accepted/rejected/峰值和末采样；从满状态shutdown后验证registry终态、pending0、自有线程0、backlog0。原超时/失败清理控制随模块一起重放通过。
3. 独立 `cargo test --offline -p auto-media-core --lib -- --test-threads=1`：**44/44，0 ignored，72.16s，warning0**。随后直接运行同一已绑定test binary的正控三次：**3/3**，4.59s/4.59s/4.51s。
4. core `cargo check --offline -p auto-media-core --tests` 与scoped rustfmt均exit0，warning0。见 [verification.json.txt](verification.json.txt) 与各stdout/stderr。

以上关闭旧正控例和原计数/退出/清理路径；新负控仍有下节修复回归，整体不能最终pass。work自述的16次正控压力绿属于work报告，本轮独立执行数如上，不混加计数。

## 3. F811-R7-01 · P2 · 新未满负控的输入不能保证未满

**AC-03 / V-03 / T-01、T-07、T-08，test-only修复回归。**

`full_channel_oracle_negative_control_below_saturation_stays_not_full`在 `image_pipeline.rs:3319` 设置item_budget=CAPACITY=8；`:3388–3394`却断言peak_backlog<CAPACITY，理由是“正在变换者总会占一项”。这只有在resize线程**已经取走至少一项**时才成立；fixture没有建立该前提。

若resize线程暂时没有被调度，decode仍可向sync_channel(8)成功发送全部8项。此时running=8仍满足总任务数上界，但backlog可合法等于8。**R6的“8项不可能达到running≥9”诊断仍成立，不能把它外推成“8项永不排满8槽”。**

### 可复现的实际原测试失败

[negative-schedule.py](negative-schedule.py) 直接启动当前test binary的原负控（没有复制算法、修改测试/实现或增加图像尺寸）。仅按这个**自有子进程PID**查找`media-resize`线程，延迟其调度8秒后恢复；其他线程照常运行。恢复后等待原测试完成，finally恢复/关闭线程句柄并回收该子进程；不触碰其他应用或系统线程。延迟短于原fixture自身30/60秒前置/收敛预算。

实际结果：

```text
test ...full_channel_oracle_negative_control_below_saturation_stays_not_full ... FAILED
test result: FAILED. 0 passed; 1 failed; 43 filtered out; finished in 55.59s
panicked at image_pipeline.rs:3390:9:
not-full control: backlog peak 8 must stay below the channel capacity 8
(the resizing item always occupies one of the accepted items)
```

原件 [negative-schedule.stdout.txt](negative-schedule.stdout.txt)、[negative-schedule.stderr.txt](negative-schedule.stderr.txt)、[negative-schedule.json.txt](negative-schedule.json.txt)。同hash exe为`899a84b0d78345f8dc393b6ad320c76d6eb504728233a511d56dac6350c735d4`；记录resumed=true、exit101、child_reaped=true。

失败点位于“全部票据正常Ready”和running≤8断言之后：工作已正常收敛，红的是错误的backlog<8断言，**不是解码/关闭失败或资源泄漏的证据**。普通44项首跑该用例为绿；本轮不声称常态必红，受控延迟用于检验新算术前提的普遍有效性。

### 所需修复

二选一即可，不需要改变生产队列/容量或重做已通过正控：

- 使用严格小于容量的负控预算，例如CAPACITY−1（7项），相应核对accepted和running上界，保留正常Ready收敛/归零检查；此时即使接收线程尚未运行也确实不可能占满8槽。
- 或在8项负控中先受控确证接收线程已取走首项，建立“至少一项不在通道内”的前提，再断言backlog≤7。

补新负控及接收线程迟调度组合，保留正控满态shutdown/回收与原44项里的已闭生命周期控制。不能仅增加sleep期待resize先取走、删掉负控断言，或用一次普通绿代替正确的未满输入。当前负控构造的逻辑不足与本轮R6正控已修是两个不同事实。

## 4. AC、规范增量与下一步

| AC | R7结果 | 依据 |
|---|---|---|
| AC-01 | pass | 生产源码/依赖/config等价，复用R6精确库、VM/core接线与轻闭包证据 |
| AC-02 | pass | 关闭准入/open/重复shutdown/归零等旧例无生产变化；生命周期定向随44项通过 |
| AC-03 | partial | 正控有界造载与从满态回收已闭；新增未满负控输入不充分，受控调度失败 |
| AC-04 | pass | 复用R6七窗口/三形态和真实HTTP13；输入/生产实现与依赖/config未变 |
| AC-05 | pass | 发行/生成/manifest/777/792相关面未变，指定R6证据复用；最终合流门pending |
| AC-06 | fail | 普通44/44与check/fmt健康过，但新负控有可复现错误断言 |
| AC-07 | pending | merge-owned |

SD冻结 [spec-delta-input.md](spec-delta-input.md)，SHA `6d6c937b752dbfb0f6152f2cc4a9c72ff25c1cd49a63d16707830a2cdd7b4618`；六个目标、五supersedes/一new及GOAL-003/007有效。当前809管理规则源/effect adapter分工按Plan SD05调和，旧handoff“未迁”只作历史原件，不回写canonical。未改变语义合同、Plan revision、目标或验收尺度。

Plan → **executing 5/11**：保留T02..T06，重开T01/T07/T08，T09未通过。旧F811-R2-02正控余量关闭，其余旧findings保持关闭；当前仅**F811-R7-01**开放，工作范围是修正新负控的有效前置条件。原树提交后做影响复审。

merge-owned仍pending：最终pass后的冻结裸cargo t+tv、最终并集/777/受影响发行、canonical/ledger沉淀、归档及guard清理。本轮不授予合入或交付完成状态。
