# PLAN-820 r6 R6 — needs_fix (formal typed-result oracle incomplete)

2026-10-25；review_scope: final / affected repair + outstanding cache/formal-result acceptance。reviewer：本聊天Codex，独立于实施上下文，未启动代理。唯一树 D:/autostack/.wt/lang-820/auto-lang；plan-820-dev；受审HEAD7c93283423f23b52020e7e98c7bd235eeb9bd51a；repair base affb907bd8657d32a027453268cb209450b6c73b；original base37e51253689631ffa61a9af58041c5f6646a7569。前后HEAD/clean一致、diff-check绿。

入审主检出153d1dff5c1b54ef40b5ada91e10c0330d3390c2（晚于用户交接a07430028）；Plan SHA256 6245BCD84B0EB9A025ACBE822198C5313D622E1193EC682326D135B23F281C38；SD draft SHA256 4FFC08BC60A3BEABCE86B63F2E4B6F581E8204CD24ED0926047124D201F3FB69。实际Plan executing7/9，按完成声明入审，不改语义revision。

## F820-R5-01 CLOSED

独立 `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_data_820_incremental av_data_820_v13_caps_`，2 passed/0 ignored/0.02s；定向data_session/incremental格式与diff-check绿。

另原样重编R5审查输入：[caps-replay.rs](caps-replay.rs)、[运行器](run-caps.ps1)、[输出](caps-result.txt)。新增trace授权后热编辑mark_a成功，当前结果1，冷对照1；撤销被使用的授权受控拒绝、旧会话仍1。新增回归亦覆盖成功撤销后无权热编辑拒绝。源码确认候选env用新caps parse/resolve，失败恢复oldcaps，全部成功后env/code/snapshot同发布点更新。前轮事务/布局/计量/验收守卫/格式等关闭证据保留，未触发全量重放。

## 补完V-08“好hash坏内层”的独立正反控

[输入](cache-inner-replay.rs)、[运行器](run-cache-inner.ps1)、[输出](cache-inner-result.txt)。通过公开ManagedStore.put_artifact在A20发布真实artifact：store内容hash有效、信封身份与checksum有效、payload AVAD内类型自环。get_artifact成功取得该row；真正调用restore_data，得到 `re-verify: abc verify: mir.type-cycle`，execution_seq增量0。不是破坏信封checksum后调用unwrap的浅层控制，也没有伪造测试总数。独立补证了当前恢复的内层验证与执行前拒绝；此例未发现实现缺陷。

## F820-R6-01 P1 — 正式data recipe尚无typed result/位型父侧判定通道（AC-01/06，T-02/06，V-03/10，SD-05）

**既有规则**：计划§5.2要求首例的typed宽标量位型往返与正式AutoDown父侧判定可达；contract-decision §10（229..231行）明确父侧Success宽标量逐位bits断言，§11/§12 V-10要求wrong-bits反控总体非零/ERROR。不是要求新增宽标量源码语法，typed fixture仍是已批准起点。

**实际路径/原因**：

- av_probe_worker.rs:429..447 data recipe分支只把source交给request_data(record source front door)，Success用 `value.as_i32()` 变成JSON value；没有typed MIR fixture请求/结果分支，也没有六标量tag/rawbits。
- 父侧ac_av.rs:430 `StepOut.value: Option<i32>`，deny_unknown_fields；run_data_check复用此判定面。无法携带u64::MAX、i64::MIN或f32/f64位型，也丢失合法bool结果。
- 已提交V-10四测只有wrong-exit、wrong-diag、未注册recipe、worker exit/extra-frame/wrong-hash故障。没有正式wrong-bits、位型合法正控；in-process assert_eq/to_bits不等于父侧独立判定。

**真实可复现输入**：[formal-bool-replay.ps1](formal-bool-replay.ps1)、[请求](bool-request.json.txt)、[worker原始响应](bool-result.txt)。调用受审树实际worker `case-serve`、正式recipe ac-av/data-run-v1，输入支持的record源码（含P{x:42}）但main返回bool true。响应：

```json
{"op":"run","outcome":"success","value":null,"trap":null,"diag":null}
```

源类型/结果在进入父侧前已经丢失。不是把bool新增为源码能力；当前record域和data entry原本支持bool。这里只声称worker实际输出丢值，并由父侧类型定义证明无逐位判定通道，未伪称完整parent suite对此反例实际误绿。宽标量未有源码语法，所以不能用无效宽标量source probe冒称该请求已到达typed fixture生产路径；缺口正是该正式入口尚未交付。

**关闭oracle/边界**：在当前真实owner处实现最小、明确版本/身份的data typed fixture入口与typed结果编码（不扩产品默认或源码语法），父侧独立比较tag和精确rawbits；至少覆盖冻结u64::MAX/i64::MIN、负NaN payload/-0.0及f32样本、现有bool/i32，合法正控与只改bit/类型tag的错控都经正式recipe父侧实际判断；错误工具身份/漏case/skip保持非绿，旧core-i32协议与消费语义保持。修正SD-05及合同清单的case→入口→oracle绑定。若现有owner协议确实不能完成，按原计划回new显式修订并取得批准；不能继续把scalar-exit recipe当作宽标量父侧门已完成。

## 其余审面/状态

R5两授权测试修复不改变cache/codec执行核心，本轮复用前轮已绑定的语义/缓存普通正例、资源峰值和布局拒绝证据；新增上述真实内层恢复反例。V-10工具故障已有wrong-hash框架，但typed bits子项确定缺失。V-12精确cap两侧及全部Spec目标当前版本/冲突对账仍需最终补核，不能靠历史case总数标pass。SD-07已描述caps跨层发布，当前未发现R5原错误；SD-05的scalar-only当前描述与已批准父侧typed判定义务不一致，不能按当前草案沉淀为降级后合同。未发布canonical/ledger。

AC-09的R5 finding关闭，其他已闭finding均不重开；AC-01/06仍partial（F-R6-01），AC-07未过，AC-08 pending。回executing5/9：重开T-02/T-06，T-09保持完成。先回work补本条既有合同缺口、具名根因与端到端正反控，再提交复审受影响及未审面。未改实现、未整档/真实acceptance聚合、未合入。merge-owned裸cargo t+tv+独立workspace完整有限合同档仍pending。
