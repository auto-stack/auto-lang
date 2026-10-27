# PLAN-817 R2 overall readiness and repair review — needs_fix

2026-10-23；review_scope=final；r3；独立于实现的当前Codex审查上下文。worktree D:/autostack/.wt/lang-817/auto-lang，branch plan-817-dev。HEAD仍为f11c6dbc72ac397cfb38c8fe912bf2ae412c42c3，base d46e03e8a4986f3d5fed5bd09665c2aeebf0340d。实际检查对象为未提交修复，见 inspected-wip.patch/status.txt；不能将其测试结果授予旧HEAD。检查中bench.rs从无修改变为新增68行，明确仍有外部写入，未冻结。本轮仅记录审查，不提交他方WIP或修改实现。

## 上轮闭环核查

| finding | 当前证据 | 结论 |
|---|---|---|
| F-817-R1-01 | snapshot测试8/8通过，覆盖原2/12与混合callee/arity、DAG、trap | 原例已修；新冷bool反例失败，完整不变量未关闭 |
| F-817-R1-02 | CteDriver已有resident；release 30修订测试1/1通过，16.74s | partial：测试末尾把main当前generation与自身比较；保存的h0/h114/sum代际被丢弃，无逐修订独立值断言/未改函数完整阶段与IO观测。forced rebuild测试只要求counter>=1且结果Ok，无正式gate非零退出。未满足V-12四子case全部要求 |
| F-817-R1-03 | bench仍直接query_step、小sum_deck、restart new driver；新增big_deck当时未使用，编译dead_code告警 | 未关闭；正式runner端到端、真实新进程、规定负载/原始证据缺失 |
| F-817-R1-04 | 检查期间才加入samples/is_finite前置检查 | WIP，尚无提交/重建后的完整非法输入反控证据，不复用旧结论或假称已关闭 |
| F-817-R1-05 | Meta→AV定向2/2通过；新增meta块+正式case WIP | 主路径改善；规范SD-01仍称record面AV拒绝，新增meta语义/能力准入与schema文档尚需全面重审，完整关闭pending |

## 新发现

**F-817-R2-01 / P1 / AC-02,03,11 / T-03,T-11：冷snapshot首调用bool回归。** `SnapshotLane::warm`无条件wrapper_source(...,false)，尽管driver已计算is_bool，warm未接收；生成i32 wrapper直接return bool。通过重建当前debug worker真实case-serve复现：first-bool.atomdata → first-bool-request.json → first-bool-result.json。flag(3)应返回bool:true并回填main.g，实际cte.callee.frontend、expected i32 found bool、ct_execute=0。源码SHA256 av_driver=C968FE8CDF9EC2D7790DED5689ECCB71526AFBB8981BEA9E7B5C5A9E6C42BD1F，driver=B34F9E7843EA0A688D33B86136DAC356E456C3808F9C0AB1B2DE4A66CFA2BA70。已有mixed测试从i32起步，遮蔽冷bool分支。关闭oracle：首bool真/假、i32→bool→i32、不同arity、失败后重试及cache miss/hit全部正确，合法bool不得被分类成语言错误。

重放（新store避免残留）：在主检出 `Get-Content docs/plans/attachments/817/review-r2/first-bool-request.json -Raw | & D:/autostack/.wt/lang-817/auto-lang/experimental/ac-cte-probe/target/debug/cte_probe_worker.exe case-serve`。同输入exec closure作为合法对照应保持通过，修复不得扩大语言支持范围。

**F-817-R2-02 / P2 / AC-08,09 / T-08：交接/健康/规范未收口。** 修改未提交，Plan仍executing 5/11、没有本轮work收据；acceptance-matrix/spec-delta/turnaround报告仍旧。定向编译有fresh_warm未用/赋值未读、测试p95遮蔽未用（期间新增big_deck dead_code）。清理真实告警并刷新AC/V/SD映射和固定提交，暂停实现写入后方可最终审查。不能用新SD缩减已同意合同。

## 本轮命令和结果

在既有plan worktree执行，未运行整档t/tv/tt/tb/tu/taa：
- cargo test --manifest-path experimental/ac-cte-probe/Cargo.toml --offline --test cte_snapshot_tests cte_snapshot_ -- --skip baseline_30：8 passed，0 failed，1 filtered。
- cargo test --manifest-path experimental/ac-cte-probe/Cargo.toml --offline --test cte_schedule_tests cte_schedule_meta_argument：2 passed，0 failed，16 filtered。
- cargo test --manifest-path experimental/ac-cte-probe/Cargo.toml --offline --release --test cte_snapshot_tests cte_snapshot_resident_lane_baseline_30_revisions -- --nocapture：1 passed，0 failed，8 filtered，16.74s。
- cargo build --manifest-path experimental/ac-cte-probe/Cargo.toml --offline --bin cte_probe_worker：成功，有2 warnings；随后独立bool反例失败（合法程序被拒）。

重建worker后运行，避免R1旧binary问题。release测试自身绿不弥补vacuuous identity断言或缺失门禁。没有把任意改动提交到旧HEAD名下；spec-delta冻结SHA256 FC18F2937DC1959E6153319FAB7358BC13F84467504D7D5A659ABAB02CED52CB。

## AC与路线

AC-02/03/06/08/11 fail/partial；AC-01/04/05/07 pending完整冻结复核（本轮不重跑未变缓存/GC/规模重项，因为必做修复仍未完成且代码漂移，不能授予整体pass）；AC-09不通过；AC-10为merge-owned pending。新meta输入的资源/解码/权限负控亦须在最终提交补齐审查。依据review技能的readiness门，不在已知未完work上继续一轮完整最终验收。

维持executing，T-03/T-07/T-08/T-11开放，current_step=5。下一步work关闭R1剩余问题和R2-01/02，完善诊断及覆盖调整记录（已两次非pass），提交/冻结并刷新主检出交接，再最终review。merge-owned裸cargo t+tv+probe消费面门禁pending，最终review通过后由merge阶段执行；本轮未合入/归档/发布canonical Spec或ledger。