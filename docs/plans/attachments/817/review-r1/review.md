# PLAN-817 final review R1 — needs_fix

2026-10-23，r3，独立于实施的当前审查上下文。实现 f11c6dbc72ac397cfb38c8fe912bf2ae412c42c3，base d46e03e8a4986f3d5fed5bd09665c2aeebf0340d，同 plan-817-dev 工作树。没有修改实现。读取主检出 Plan，直接检查代码及反例；不依据执行者 PASS 作结论。

## Findings

- **F-817-R1-01 / P1 / T-03,T-11 / AC-02,03,11：snapshot 第二个 callee 被错误执行。** av_driver.rs::retarget 仅替换实参 span，未替换 callee；callee 参数只用于随后 rfind，且 is_bool 保留首调用类型。request.json / two-callees.atomdata 是生产 case-serve 输入：first(a)=a+1、second(a)=a+10，q1=first(1)、q2=second(2)。预期 x=2,y=12，实际 outcome=ok、ct_execute=2、x=2,y=3，错误值还回填 HIR（见 two-callees-result.json）。原因已由代码与运行双证实。关闭 oracle：同请求不同 callee、不同返回类型及参数数目，合法 DAG/重复同 callee 正控，结果/缓存/consumer 均正确；失败不得遗留错误发布。
- **F-817-R1-02 / P1 / T-11,T-08 / AC-11：正式查询跨编辑没有驻留增量会话。** driver.rs::query_step_inner 每次声明局部 lane=None，再 SnapshotLane::warm；CteDriver 不持有 lane。故每个 helper 编辑导致 miss 后重新整模块 warm，不是 §5/T-11 的跨请求未改对象/代际保留。cte_snapshot_tests.rs 将 driver 缓存测试与独立手动 SnapshotLane 的增量测试分开，未覆盖实际 consumer 路径。V-12(1) 仅几个小函数+30次改实参，无273575B/115函数基线、P95和完整阶段/IO观测；(4) whole_module_events 初始化为1而非向正式 gate 注入退化并要求非零退出。关闭 oracle：正式 recipe/consumer 连续真实 body edit，重放规定基线与30不同修订，证明未改对象/代际与生产阶段/IO=0，真实整模块重建/重装载及换对象反控必须使 gate 非零。
- **F-817-R1-03 / P1 / T-07,T-08 / AC-06：性能测量不是约定端到端/负载。** bench.rs 使用两个小函数；restart 只 new CteDriver，无新进程；measure 直接 query_step，排除准备/序列化、IPC与父侧准入/比较。meta 编辑仅 field_count，不是 Meta→AV 实参执行。故报告4ms/1ms不足以证明256KiB/100函数负载两编辑端到端门。关闭 oracle：正式 runner、规定有效负载、每场景30样本；restart PID不同；单墙钟覆盖准备至父判，原始值/计数/hash/同轮内存可复算，GC真实并发。
- **F-817-R1-04 / P2 / T-07,T-08 / AC-06：空样本门禁假绿。** `cte_probe_worker.exe bench-turnaround --samples 0` 实测 exit=0、六场景samples=0、P95=0、target_met=true，记录 zero-samples.json。bench.rs percentile(empty)=0、all(empty)=true，无数量检查。V-09明定空/负/NaN反控。NaN阈值被CLI接受为null，比较本身不拒绝；本轮NaN运行因临时store权限导致值失败，不能声称复现NaN单独假绿。关闭 oracle：0/不足30/非法/负/NaN/Inf输入在同一正式门拒绝且退出非零，正常30样本及慢样本控制保留；严格P95<阈值（当前仅>判错）。
- **F-817-R1-05 / P1 / T-03,T-08 / AC-02,03,08：Meta结果作为真实AV实参的约定路径被删除。** schedule.rs:155显式拒绝所有record-profile Call，core profile又无record；work收据主动承认源级组合不可达，却在spec-delta SD-01把拒绝写成新合同。原§5.1规定FieldCount/HasField结果作为真实AV调用实参；未见授权缩减。独立meta查询或manifest级单测不关闭此要求。关闭 oracle：通过实际recipe运行FieldCount/HasField→Call→consumer，修改字段集/缺席字段后实际执行、结果与精确依赖失效正确；或先取得显式合同修订再重审，不能仅改SD。

## Evidence and limits

重放：在主检出运行 `Get-Content docs/plans/attachments/817/review-r1/request.json -Raw | & D:/autostack/.wt/lang-817/auto-lang/experimental/ac-cte-probe/target/release/cte_probe_worker.exe case-serve`。首次已持久化错误冻结值，重放可为hit，依然错误；用不同store_root可重放miss。worker binary 来自现有 release 构建，本轮未重建，故代码解释同时独立核对冻结提交；后续修复须重新构建。首次TEMP store运行遇os error 5，作为环境失败剔除，换本仓可写store后成功复现错值。零样本输出已持久化。没有运行整档cargo t/tv/tt/tb/tu/taa。

发现必做未完路径后返回work，未宣称完成其余全部验收/健康检查/峰值/GC独立复核。AC-01/04/05/07 pending进一步复核；AC-02/03/06/08/11 fail/partial；AC-09未过；AC-10 merge-owned pending。Spec delta冻结见同目录spec-delta-frozen.md，SHA256 FC18F2937DC1959E6153319FAB7358BC13F84467504D7D5A659ABAB02CED52CB。SD-01/06受上述finding影响不通过；touched_goals=[]说明未补齐，后续核清实际goal映射或无影响理由。

实现树HEAD再次核对未漂移；git status受当前沙箱访问限制无法确认clean，不能授予pass。依赖复用冻结提交所含783/796/798/828；未完整重核全部外部依赖工作树，后续review仍需完成。修复后提交同树、重放反例、再最终review；merge-owned cargo t+tv+消费面定向保持pending，不合入/归档。