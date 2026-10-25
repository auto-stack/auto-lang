# PLAN-820 r6 R9 — needs_fix (new zero-value regression; T-06 only)

2026-10-25；review_scope: final / affected repair + legitimate canonical boundaries。本聊天Codex，独立于实施上下文，未启动代理。唯一树D:/autostack/.wt/lang-820/auto-lang，plan-820-dev；受审HEAD8e0e0ef337991c1e1726847b8c6c70aaccf26a6c；repair base7308acad627df367c7a79e43212aec4f9078b790；原base37e51253689631ffa61a9af58041c5f6646a7569。前后HEAD/clean一致；diff-check与定向ac_av.rs rustfmt check通过。

入审主检出4774ee7914b6be9a34e81435431fb0e5f0f927a4（晚于用户交接89a278c25）；Plan入审SHA256 F5BD009C78D7D3F48CEA805EB1B565314FA51F72FFA2F0FAD54E45DDA77AF473；SD SHA256 8349379826CE33F71A587A0CD58B08829C62E25DE29AF84B2D34F59E6AB621F0。Plan实际executing7/9，按完成声明入审。

## F820-R8-01 CLOSED

原7组真实父侧故障worker控制全部原样独立重放：[runner](judge-replay.rs)、[受控worker](response-worker.rs)、[运行器](run-replay.ps1)、[输出](result.txt)。core双形态Fail/Error；data bool/0、bool/1合法形状Pass；bool/2、bool/00以及有匹配exit42的weird/bad均在oracle之前Error。准入不再取决于是否声明exit，此原finding不重开。

独立生产judge_tests 3/3、0 ignored，[输出](judge-unit-result.txt)；wrong-bits含f32 NaN反控1/1、0 ignored、0.64s，[输出](bits-result.txt)。未重跑mass、turnaround、workspace全套或验收聚合。

## F820-R9-01 P1 — 新规范编码校验拒绝合法i32零结果（AC-01/06, T-06, V-09/10）

既有规则：data支持i32（含0）并保留其合法结果；canonical_typed(I32(0))正常产生不补零的十六进制字符串"0"。禁止前导补零不等于禁止零值。

**真实guest/真实worker/真实parent的正控边界矩阵**：[输入](real-int-replay.rs)、[编译运行器](run-real-int.ps1)、[原始输出](real-int-result.txt)。此运行器不设置AC_AV_CASE_WORKER，public run_suite走真实data recipe worker；四个source均为支持的record源码 `type P{x int}`、main构造P{x:V}再返回p.x，expect为success+exit=V。

| V | 实际判定 |
|---|---|
| -1 | Pass |
| 0 | **Error**：`malformed typed report on step run: typed bits 0 are not canonical for kind i32` |
| 1 | Pass |
| 42 | Pass |

这是实际合法source失败，不是故障worker模拟产值。旧R8故障控制仍单独记录在result.txt，二者不混淆。

**已建立原因**：本轮ac_av.rs::typed_shape_violation将i32判定写成 `(1..=8).contains(&t.bits.len()) && !t.bits.starts_with('0')`，未为单字符"0"保留合法分支。正常worker在每个data success均附typed，因此即使旧value通道为Some(0)、exit期望也为0，oracle之前仍必然被拒。新增单测合法i32只覆盖"2a"而没有0，非法前导零测试通过掩盖了这一合法边界回归。

**最小关闭oracle**：允许零的唯一规范编码"0"，保持"00"/"02a"等补零编码非绿；定向覆盖实际guest -1/0/1/42（按机制可补MIN/MAX）正例，并保持R8七组合及typed shape反控不回退。不要跳过整个i32/typed校验来让零过；data入口scope与oracle无关性继续保持。修复只需受影响结果准入与边界测试，无须完整性能矩阵或整档。

## 状态与后续

回executing6/9，仅T-06重开；T-02及旧findings关闭保留。AC-06仍partial（合法data零结果回归），AC-07未过、AC-08 pending。V-12/SD最终逐目标对账及兼容性能证据适用性未完成，当前已知回归下未继续完整final聚合；不把执行方单测总数当独立pass。既有turnaround红/环境解释保留，本轮没有重试、改阈值或独立宣告豁免。

原树work补本条具名根因/合法零边界，提交后定向复审与未审项。未改实现/canonical/liveledger、未合入/归档。merge-owned裸cargo t+tv+独立workspace完整有限合同档仍pending。
