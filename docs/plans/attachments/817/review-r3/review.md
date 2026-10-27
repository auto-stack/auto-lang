# PLAN-817 R3 final review — needs_fix

2026-10-23，r3；独立于实施的当前Codex上下文。冻结实现9fe341e61fdb8743ee1d89988c3fd8c90d48f1aa，base d46e03e8a4986f3d5fed5bd09665c2aeebf0340d，plan-817-dev，D:/autostack/.wt/lang-817/auto-lang；开始和结束均clean/HEAD不变。主检出work收据已读取（d652fa234后续master有他方簿记），未改其余WIP。不是未提交代码审查。

## 修复确认

重建release worker，重放R2 first-bool-request（改用本轮store），实际main.g=bool:true、HIR=0<1、ct_execute=1，见bool-result.json。冷bool/转换/缓存等snapshot12个定向测试通过，Meta/调度19个通过；非法参数samples=0和gate=NaN分别返回target_met=false（exit2）。重建无编译warning。驻留代际检查已改独立比较，meta负控已补；不再重提其旧缺陷。Spec delta已刷新并冻结，SHA256 45B598E40D49C6B4BEFF5CA8E7A64240285AC256B8EC5A01E867184BA5CD36D2。

## 尚未关闭的发现

### F-817-R3-01 P1：同轮内存硬门超标仍给target_met=true（AC-05/06，T-05/07/08）

提交的docs/reports/plan817/turnaround.jsonl最终行：worker_private_bytes_peak=669405184（638.39MiB），超过384MiB worker门402653184，也超过512MiB树门536870912；却hard_gate_failures=[]、target_met=true。冻结原报告见submitted-turnaround.jsonl，源SHA256 B5A176DF8D0443DBA96E524CCD94A2EDB2AD0F9947F37E41918C322A54D9F9CC。

bench.rs:503/518只记录worker_peak，后续失败列表没有内存校验；也无真实进程树同时峰值归集。多个场景driver共存可能影响峰值，具体分配原因待work定位；已确认的是实际同进程采样超标和gate漏判，不先断言具体泄漏根因。合同§3/§5.4规定内存与时间同轮验、超标不可target_met=true。

关闭oracle：正式测量窗worker≤384MiB/整个请求进程树≤512MiB、受管≤32MiB，任何超标/采样不可用都不得绿；实际spike输入使同一正式gate失败，正常输入通过，raw报告与判定一致。不能从峰值任意扣前端或其他仍存活对象。

### F-817-R3-02 P1：驻留30修订时间验收失败，bench又漏判该场景（AC-11，T-11/07）

独立命令（同worktree）：cargo test --manifest-path experimental/ac-cte-probe/Cargo.toml --offline --release --test cte_snapshot_tests cte_snapshot_resident_lane_baseline_30_revisions -- --exact --nocapture。

首次连同schedule/snapshot定向执行：19 schedule通过，12 snapshot通过，baseline失败P95=1231ms（27.34s）。避免重建worker的并发影响，单独再跑上方命令仍失败：P95=1096ms，exit101，1 failed，28.18s。机器负载可影响具体数字，不以此推定特定算法根因；本机当前required gate确实未通过。

更直接的冻结证据：提交报告resident-edit.p95_ms=1527.9816却target_met=true。bench.rs:543只对body-edit/meta-edit比较阈值，resident-edit仅检查重整计数/值，不查P95。关闭oracle：在规定参考负载及环境说明下30不同修订结构与P95<1000ms双门均过；同一正式gate对resident慢样本失败，不只在单独单测中断言；保留全部raw失败样本。

### F-817-R3-03 P1：原R1-03端到端窗口/负载仍未关闭（AC-06/07，T-07/08）

bench.rs body-edit走measure→drv.query_step（deck构建在measure外），meta-edit也直接query_step；两硬门没有IPC/父侧准入比较。spawn_process_samples并非正式AutoDown父判入口，且计时wall在解析JSON/echo检查之前结束（665行）。restart和worker-edit分别在296/409行仍传sum_deck(0)，即两函数小输入；worker-edit每轮都是相同0→1改动，未逐轮新版本/hash。已有115函数输入只在进程内其他场景，不覆盖声明的“全部场景规定负载”。raw没有value字段，和work收据“raw行携带pid/value”不符。

关闭oracle：规定有效负载在真实正式runner/harness中，计时从准备/序列化到父判结束，两编辑硬门用该窗口；restart真新进程、每轮编辑新hash，场景负载/PID/值/计数/raw均可核查。增加旁路小输入IPC场景不能替代原硬门。

### F-817-R3-04 P1：benchmark oracle及退化反控仍可假绿/假红（AC-06/09/11，T-07/08/11）

spawn_process_samples的ok只看进程成功、schema/nonce/hash及steps数量，完全不检查step outcome、values、consumer/HIR、编辑结果。故合法协议形状中的diag或错值仍是ok=true；measure_built只验证值非空，body/resident基值又取自被测AV，无独立完整expected。row输出无实际值，无法从raw复算。全量“all_values_ok=true”不是这些语义路径的证明。

退化控制：bench.rs:567分支在inject_rebuild=true时无论是否探测到重整都会push失败（re_delta==0另push，之后又无条件push控制红），其自身exit!=0不能证明“同一gate”检测了重整。测试额外要求计数>0有所帮助，但仍未证明gate真实失败传播；控制标记不能制造红来代替闸门。

关闭oracle：对真实父判输入施加坏actual/diag/错误值/错误hash等变异，同一语义判定必须非零；独立闭式完整数值+consumer检查，raw记录实际值。退化由真实检测器值触发普通gate红；保留强制输入但禁掉检测器的控制应暴露测试失败，不能因inject标志本身恒红。无关合法输入保持通过。

## AC、规范与后续

AC-02/03相关冷bool与meta定向改善已验证；AC-05/06/11因上述缺陷fail/partial，AC-08规范不得沉淀尚未满足的资源/时间/端到端保证；AC-09未通过。AC-01/04/07剩余完整验收与未重跑资源/GC部分pending；本轮发现required gate失败后未用整档测试掩盖未完成work。旧缓存/GC证据保留，非宣称新版本整体通过。AC-10及裸cargo t+tv+probe消费面门禁merge-owned pending。未执行整档t/tv/tt/tb/tu/taa，未改实现/canonical Spec/ledger，未合入或归档。

当前收据声称“全部闭合”与固定报告及代码不符。返回同树work：T-05/T-07/T-08/T-11打开；T-03针对冷bool/Meta定向修复可完成。完成任务5/11。此前已两轮失败，下一交接须提供根因诊断和验证覆盖调整（特别是实际门禁窗口与失败传播），提交后再复审。