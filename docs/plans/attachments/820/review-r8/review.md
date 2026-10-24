# PLAN-820 r6 R8 — needs_fix (only T-06 reopened)

2026-10-25；review_scope: final / affected repair + typed response sibling controls。本聊天Codex，独立于实施上下文，未启动代理。唯一树D:/autostack/.wt/lang-820/auto-lang；branch plan-820-dev；reviewed_commit7308acad627df367c7a79e43212aec4f9078b790；repair base278b1dce0e94cbe7e5e68a921e5ee569abc88314；原base37e51253689631ffa61a9af58041c5f6646a7569。前后HEAD/clean一致、diff-check绿。

主检出入审HEAD cf215283b59cc090ba9d75beb7030b81bb6cb7ea；Plan SHA256 0290BBE0DBE15A616FFF9BC1655B235CA673B58C4E1234D0CD793693D568755A；SD SHA256 6BC05B8259900587FE074F0605A50BA13D578F10F3655BDFE31C908F502088AB。Plan实际executing7/9（当前主检出），按完成声明再审。

## R7两项独立CLOSED

- **F820-R7-01 CLOSED**：原故障worker双形态通过真实run_suite再次重放，core value=null无typed→Fail/missing，附typed bool/1→Error/protocol violation。源码准入typed_on_non_data与judge family隔离均已接入生产路径。正常core/data协议此前的正控不作废。
- **F820-R7-02 CLOSED**：worker/父侧独立表、正式case均加入f32-nan-payload/7f80ad00；独立根仓wrong_bits_oracle_fails（包含wrong-bits-f32-nan）1/1、0 ignored、0.56s，[输出](bits-result.txt)；独立corpus正控1/1、0 ignored、3.48s，[输出](corpus-result.txt)。因此不再重开T-02。
- 新judge_tests独立3/3（0 ignored），[输出](judge-unit-result.txt)。定向ac_av/root tests/typed_fixture三文件rustfmt check通过。没有重放mass/turnaround；work收据所记P95=1064.5ms红及环境说明原样保留，本轮没有独立证明环境归因或把它改记性能pass/豁免，后续final证据仍须核其适用性。

## F820-R8-01 P2 — 新typed形状校验允许非法bool，并可被exit oracle完全绕过（T-06/AC-06/V-10）

既有规则：contract-decision §3 bool只有0/1；typed结果采用规范tag/rawbits（bool/i32不补零）；R7关闭要求包含data malformed typed兄弟路径，不能让不合法worker报告满足父侧判定。

**真正父侧端到端反例**：[runner](judge-replay.rs)、[故障worker](response-worker.rs)、[编译运行器](run-replay.ps1)、[输出](result.txt)。与R7相同受控AC_AV_CASE_WORKER接缝，正确echo nonce/request_hash、source hash与相应recipe/profile身份，仅按表改变响应value/typed字段。执行public `auto_lang_core::test_cases::run_suite`；不是对私有校验器的复制，也不把故障worker当guest正确性证据。

| 输入响应 / oracle | 期望判定 | 真实结果 |
|---|---|---|
| core无value、无typed / success | 非绿 | Fail（已修合法负控） |
| core无value、typed bool/1 / success | 非绿 | Error（R7修复关闭） |
| data无value、typed bool/0 / success | 可绿 | Pass（合法形状对照） |
| data无value、typed bool/1 / success | 可绿 | Pass（合法形状对照） |
| data无value、typed bool/2 / success | 非绿：bool非0/1 | **Pass** |
| data无value、typed bool/00 / success | 非绿：非规范编码 | **Pass** |
| data value42、typed weird/bad / success+exit42 | 非绿：未知tag | **Pass** |

注意success-only未声明真假期望，所以0/1两合法形状均可绿；没有擅自给普通bool source增加固定语义oracle。错误在非法bool编码/未知tag也被接纳。

**已建立原因**：ac_av.rs:1196把bool与i32一起按1..=8位hex验证，未限bool恰好"0"/"1"，也没有拒绝其前导零；:1237先进入Some(expect.exit)分支，仅比step.value，typed_shape_violation只放在:1269的“无exit/stdout/stderr oracle”分支。改变oracle是否含exit就改变是否校验同一不可信typed报告，导致未知tag绕过。该finding是本次新增形状校验机制的未闭兄弟面；不重新声称旧core问题未修。

**最小关闭oracle**：所有data响应中存在的typed报告均在与oracle无关的准入/形状步骤校验；bool只允许规范0/1，i32保持已定义的规范hex范围，宽标量保持固定宽度。非法kind/bits不应因value存在、exit匹配或选择其他oracle被接受；保留core协议隔离、data的0/1和宽标量合法正控。对本表7种组合及相关oracle分支做定向验证，错误报告总体必须非绿；不需要更改源码支持面或跑整档/全性能矩阵。

## 范围与状态

本轮变动不影响此前授信/cache/布局/授权机制，原关闭证据保留；R7已闭两finding均不重开。AC-06仍partial（新父侧gate缺口），AC-07未过、AC-08 pending；其他未审V-12精确边界/SD逐目标Spec对账与兼容性能适用性保持pending，不用当前单测总数冒领整Plan pass。

回executing6/9，**仅T-06重开**；T-02保持本轮修复完成。原树work修结果协议形状与oracle分支的统一准入，更新具名根因/组合关闭表，提交后定向复审。未改实现/canonical/ledger，未合入；未跑整档/acceptance聚合。merge-owned裸cargo t+tv+独立workspace完整有限合同档仍pending。
