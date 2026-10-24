# PLAN-820 r6 R7 — needs_fix

2026-10-25；review_scope: final / affected repair + shared-judge compatibility。本聊天Codex，独立于实施上下文，未启动代理。唯一树D:/autostack/.wt/lang-820/auto-lang，plan-820-dev；reviewed_commit278b1dce0e94cbe7e5e68a921e5ee569abc88314；repair base7c93283423f23b52020e7e98c7bd235eeb9bd51a；原base37e51253689631ffa61a9af58041c5f6646a7569。入审与结论前HEAD不变、实现clean，diff-check通过。

主检出入审HEAD f62b813e280b91ffdc9399005c7bbcf07002d176（晚于用户交接簿记6024c6ff4）。Plan r6实际executing7/9，按完成声明再审。本轮Plan/SD入审文件保持原内容至结论回写，未更改canonical/ledger。

## R6 修复验收

正式typed通道已实现：worker从真实AV产值生成{kind,bits}，父侧另持常量表，不拿worker自报expected作oracle；data recipe typed:<id>复用现有入口字符串，七种冻结id真实走MIR→ABC准入→执行。共享judge的所有影响需受下面兼容性反控约束。

独立根仓定向 `cargo test -p auto-lang --features test-cases --lib ac_av_data_probe -- --test-threads=1`：7 passed/0 ignored，1.91s，包括wrong-bits/wrong-type-tag/unknown fixture、旧wrong-result/diag/skip/worker故障；[完整日志](v10.log.txt)。独立 `... ac_av_probe_corpus_executes_green -- --test-threads=1`：1 passed/0 ignored，3.98s，[完整日志](corpus.log.txt)，含新增typed/bool正例。四触面typed_fixture/worker/ac_av/root tests定向rustfmt check和diff-check均通过。根仓仍有旧42 warnings，未把它们当本次新回归。

因此F820-R6-01“没有通道”的根因已消除；冻结f32特殊位型覆盖仍有F-R7-02余量。旧R1..R5各关闭问题实现未改变，保留证据；不重复全workspace/全性能矩阵。

## F820-R7-01 P1 — data typed存在性规则放宽了core-i32旧消费语义

既有规则：计划§5.1.6 / §5.3、AC-06/V-09要求旧core-i32入口与ExecOutcome消费语义不放宽，本轮typed协议增量只属于data。

实际更改：ac_av.rs的共享StepOut新增typed字段（所有recipe解码均接受），共享judge_step成功无exit oracle分支把旧 `step.value.is_none()` 改成 `step.value.is_none() && step.typed.is_none()`，没有按RecipeFamily隔离。于是core成功响应原本没有i32值会失败，带data bool typed对象后却通过。

**独立端到端反例**：[负控worker](response-worker.rs)、[真实父侧runner](judge-replay.rs)、[编译运行器](run-replay.ps1)、[原始结果](result.txt)。调用真实public `auto_lang_core::test_cases::run_suite`、正式`ac-av/core-run-v1`；源为合法int main返回42、期望completion success。测试用AC_AV_CASE_WORKER指向本次审查临时worker（公开测试接缝），该worker按请求真实echo nonce/request_hash、正确source hash、core身份，只故意损坏返回值：

```text
include_typed=0: Ok([(Fail, [Mismatch { observation: "exit", kind: "missing", ... }])])
include_typed=1: Ok([(Pass, [])])
```

两次value均null，第二次仅新增typed bool/1。这是fault-worker的父侧准入/判定负控，不声称执行了真实guest，也不拿伪造输出证明产品行为；真实judge在其应拒绝的core非i32结果上报绿，即本finding。临时exe位于TEMP，未替换生产worker、未修改worktree实现。

原因是family-specific扩展进入common presence rule；core身份版本/source/hash门均未因多一个typed字段拒绝。**关闭oracle**：core/i32成功仍必须有旧value渠道，data的typed存在性/位型判断按family隔离；同时pin上述两core负控均非绿、正常core正控和data bool/typed正控仍绿。审查共享StepOut/judge的兄弟路径（非data malformed typed等）并验证字段/值形状，别只特殊拦截本例bool tag。修复影响面是共享parent verdict，不要求全档，定向旧recipe正反控+data相关正反控即可。

## F820-R7-02 P2 — 冻结f32 NaN payload被普通1.0样本替代（原F-R6-01覆盖余量）

contract-decision §3仍冻结 `0x7F80_AD00` 的f32 NaN payload；V-03/10正式父侧位型判定及R6关闭要求明确包含冻结f32样本。当前worker TYPED_FIXTURES和父侧DATA_TYPED_FIXTURES只有`f32-one/3f800000`，case与manifest同样只有这个样本；1.0不是冻结NaN输入。in-process旧fixture有0x7f80ad00，但未到正式worker/parent这条新协议路径，不替代其NaN payload oracle。

关闭oracle：在独立父表/实际worker输入/正式case与必需ID绑定中补冻结0x7f80ad00并确认实际值逐位不变；至少其一bit错误在真正父侧判红。保留1.0可作额外合法对照，不需要删现有正例。不得悄悄改原冻结样本或把普通float测试当NaN覆盖。没有发现该NaN当前AV管线实现会出错，本finding是承诺的正式验收输入被替换。

## AC / SD / 下一步

AC-01/06 partial（typed通道建立、原7正3负通过，但兼容回归/冻结样本未闭）；AC-09沿前轮修复证据保留；AC-07未过、AC-08 merge pending。SD-05的“父侧存在性认value或typed”必须约束data family，SD的冻结样本应与合同一致；其他SD最终逐目标Spec版本/祖先对账及V-12精确cap两侧尚pending，不用执行方总数冒领独立pass。本次已知共享verdict回归后停止全final聚合。

回executing5/9，T-02/T-06重开（typed冻结样本/共享判定和验证），其他已完成work项与旧finding关闭保留。work补本轮具名根因/跨family组合覆盖后提交，复审只回放受影响与未审项。未改实现、未整档/acceptance聚合、未合入。merge-owned裸cargo t+tv+独立workspace完整有限合同档仍pending。
