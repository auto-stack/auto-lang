# PLAN-817 R4 final review — needs_fix

2026-10-23。Plan r3（主检出合同），review_scope=final，当前独立于实施的Codex上下文。worktree D:/autostack/.wt/lang-817/auto-lang，branch plan-817-dev，冻结提交 f2221f50d984a2cf638c6b8c5894247d44818750，base d46e03e8a4986f3d5fed5bd09665c2aeebf0340d。起止HEAD未变/树clean；worktree内Plan是旧r2，仅参考主检出r3。读取了R3修复收据的根因与覆盖调整，并据实际代码/运行核查。

## R3四阻断核查

| R3 finding | 本轮结论 |
|---|---|
| R3-01 memory | partial，仍阻断：实际采样对象是父进程，不是worker/进程树；managed不参与门。见R4-01 |
| R3-02 resident P95 | 本轮关闭：release snapshot13/13（含30修订，3.33s整测试族）通过；resident P95已进入>=失败判断；新正式报告97.16ms，时间与结构检测均有实现 |
| R3-03 IPC/window/load | partial，真实stream IPC及规定负载已接，但准备/父判仍在计时窗外。见R4-02 |
| R3-04 oracle/degrade | 退化检测部分关闭：普通re_delta触发红、禁用对照取消该红，5个turnaround定向测试全过；oracle只核最后一步且HIR非空，缺真实变异控制，仍阻断。见R4-03 |

## Findings

### F-817-R4-01 P1：内存门绑定错进程、漏managed/不可用采样（AC-05/06，T-05/07/08）

bench.rs spawn_process_samples/spawn_warm_samples 启动的 PeakSampler 在父进程；mempeek.rs 明确读取current_process_memory，没有按PID采子进程或对进程树同时归集。worker wire含worker_private_bytes_peak/step peak，但父判完全未读取。于是scenario_peaks.body-edit/meta-edit等是parent内存，子worker即便超384MiB仍不可见；TREE_GATE也仅比较父进程sampler，无法证明树≤512MiB。

提交report本身worker_private_bytes_peak=477859840（455.72MiB），>384MiB，但per-scenario片段<=384，最终target_met=true：同一个父worker的高水位被按512MiB树阈值判，其他采样空窗未覆盖。报告已冻结submitted-turnaround.jsonl（SHA256 B401CA1411862E9A386B692BE2AF90D39AE9D4E6FB386CE080E430769B549F74）。不据此断言子worker当前实际超标；确认的是门不检查所声称对象。

此外measure_built.ok未检查managed_target_met，wire to_wire未传该字段，gate旁注称已检查但代码没有；0/缺失采样也会被当作低于上限。关闭oracle：真实worker PID峰值、同时存活树峰值、managed cap同轮入正式gate，缺证据拒绝绿；在真实child瞬态分配spike/managed拒绝反控下同门红，正常正控绿，raw能核采样对象及峰值。

### F-817-R4-02 P1：编辑硬门仍截掉准备与完整父判（AC-06，T-07/08）

bench.rs body/meta提前构造decks/expected/steps整数组，spawn_warm_samples在for内先拿预造deck，再Instant::now；构造源/recipe未入窗。更直接：recv后立刻 `let mut wall=t.elapsed()`，随后才admit/提取cache/hash等，父侧准入比较未计时。与修复收据“完整父判在窗内”不符。IPC和基线输入已改善，不否认。

关闭oracle：样本生成起到实际正式父判结束单墙钟；逐样本源hash/字节/值匹配。注入准备或父判慢步骤应使同一P95门红，不能只给wall人工加延迟。保持规定负载与30不同修订。

### F-817-R4-03 P1：所谓完整oracle仍漏HIR/来源/前序步骤，未以变异证明（AC-06/07/09，T-07/08）

spawn_warm_samples admit只核最后step outcome、language_class、values非空/HIR非空、values.values().next()==expected及echo；没有检查具体绑定键、HIR常量=expected、consumer类型、source_hash、step数量/前序query与edited结果。spawn_process_samples也只取last并任意找edited，未完整核对各步。合法shell+正确values但错误HIR/来源仍可ok=true。代码中没有坏actual/diag/错HIR变异反控接到这些准入函数；现有延迟和结构注入不能证明语义oracle。

关闭oracle：正式父判使用逐步骤/绑定expected，核对原始来源hash、HIR/类型和值，所有required字段和协议身份；对wrong value/HIR、diag、错来源/绑定、缺步骤等真实变异，assert同一gate失败；合法控制保留。退化检测三态本轮已过，不重开该已修部分。

### F-817-R4-04 P1：owner-only优化删除合法native入口（AC-01/02，T-03/08）

新core/consumer.rs backfill_and_verify仅extract resolved owner函数，若reduced可编译+patch_and_verify就返回，未保留req.entry及调用闭包；entry验证在稍后driver.run_native进行，此时不再fallback。复现输入owner-entry.atomdata（同目录request/result）：sum=20+22；consume(){let ct=0;return ct}；main(){return consume()}；bind consume.ct=q；entry main；默认native on。预期回填consume.ct=42并由main返回42，实际真实worker报cte.request.entry: entry.not-found d_main（ct_execute=1）。加native off对照同源成功（cache hit、value/HIR=42），见owner-native-off-control.json。绑定合同不要求owner必须entry，原实现完整bundle保留main；新路径损失支持面。

运行使用本次cargo test构建的release worker；额外cargo build因turnaround仍持有exe出现Windows sharing violation，未把这条构建失败当产品缺陷。复现worker实现与测试构建同冻结提交；case-serve返回生产wire结果。关闭oracle：owner!=entry、多个owner、entry调用被回填owner、普通owner==entry、回退路径/类型错/错误绑定均通过对应正负控，独立native出口=42；优化不得重parse CT结果或默默删除受验模块。

## 验证记录

- cargo test --manifest-path experimental/ac-cte-probe/Cargo.toml --offline --release --test cte_snapshot_tests --test cte_schedule_tests：snapshot13/13、schedule19/19通过；基线30修订通过（整snapshot族3.33s）。
- cargo test --manifest-path experimental/ac-cte-probe/Cargo.toml --offline --release --test cte_turnaround_tests：5/5通过，84.10s。此绿不覆盖上述未断言的内存对象/时间边界/oracle。
- owner-entry-request通过case-serve：合法程序入口被删，diag（原始JSON已存）；native off控制ok。
- 广域cargo fmt --all --check失败，触及兄弟auto-down生成源及既有坏example（ui_container.rs未闭合字串），不归因为本计划；仅作环境/基线限制，未修改其内容。随后对本轮四个主要修改文件做rustfmt定向检查。
- 未执行整档cargo t/tv/tt/tb/tu/taa，未合入/归档，未改实现/Spec/ledger。

## Baseline/Spec/路由

规范增量冻结spec-delta-frozen.md SHA256 9C6A34DF33DF0547A6CD037A6CE45ED5580D10D4144CD016CE128318E3B1C5F8；SD-02/06不得沉淀未满足的内存/窗口保证，owner-only语义需先修复。依赖为冻结树内783/796/798/828及lock，未消费其他在途核心实现。

AC-01/02因R4-04 fail/partial；AC-05/06因R4-01/02/03 partial；AC-11驻留时间与结构本轮通过，完整规范/回归仍受新consumer影响；AC-03调度定向通过；AC-04/07其余完整缓存/正式runner/GC等pending，不推定整体通过；AC-08/09未完成；AC-10与merge-owned裸cargo t+tv+消费面门禁pending。发现修复引入的确定性回归后返回work，不重跑全部重资源验收来制造总数。下一轮需在同树提交根因、覆盖调整及关闭证据后复审。
定向rustfmt结果：exit1，consumer.rs:422的一处format调用需排版；其余该次检查文件未报差异。此为附加健康项，不代替四个语义/验收阻断。
