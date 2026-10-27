# PLAN-817 R5 final review — needs_fix

2026-10-23，主检出Plan r3；review_scope=final；当前独立于实施的Codex审查上下文。同树D:/autostack/.wt/lang-817/auto-lang，plan-817-dev，冻结实现eb8f9bc03312e4ba69c91e9e7018bf88bc763dfb，diff base d46e03e8a4986f3d5fed5bd09665c2aeebf0340d（本轮修复对比f2221f50d）。起止树clean、HEAD不变。阅读R4根因/覆盖调整收据后实际核代码与运行；没有改实现。

## 四项关闭核查

| R4 | 结果 |
|---|---|
| 01 内存 | partial，warm改读child峰值/顶层managed为实质改进；tree仍是parent-only，spawn_process仍parent-only，零峰值未拒绝。未关闭 |
| 02 计时窗口 | 暖IPC路径关闭：Instant在DeckKind.materialize之前，停表已移到逐步父判后；本轮未再重提旧窗口缺陷。restart/worker-edit准备仍不在窗内，属全场景剩余问题 |
| 03 结果核对 | partial，暖IPC已核op/数量/hash及最终HIR常量，但绑定键/类型/前序数值仍漏；非流式spawn仍为浅oracle，没有实际错响应变异测试。未关闭 |
| 04 owner入口 | 原反例关闭：重建release worker重放原owner-entry输入，consume.ct=42/HIR=42/native exit=42。closure提取加入entry及传递call链；native实际运行已证明旧例，不据此声称所有词法/调用组合已覆盖 |

## Findings

### F-817-R5-01 P1：R4-01内存门未覆盖完整进程路径（AC-05/06；T-05/07/08）

bench::spawn_warm_samples已读取child worker_private_bytes_peak并核managed_target_met，认可修复。然而：
1. spawn_process_samples仍在父进程启动PeakSampler，根本未消费child峰值/managed；restart与worker-edit仍可漏child超标。
2. summary的TREE_GATE仍比较run_bench父进程sampler.stop()；mempeek读取current_process_memory，未做父子同时存活内存归集。parent与warm worker各未超单进程门也不能证明树<=512MiB。
3. warm分支对wk缺失只是不更新worker_peak；0或缺失在全部样本里会生成0峰值且不因不可用而失败；summary只判断>cap，未判断0/缺失。
4. 内存spike/不可用采样控制未进入正式bench回归（本轮测试文件零diff；已有spike是人为延迟，不是child内存瞬态）。

冻结报告仍worker_private_bytes_peak=477478912（455.36MiB）但target_met=true；报告中的tree命名没有对应tree采样实现。这里不推定child实际超标，只确认缺少能证明/拒绝超标的闸门。

关闭oracle：每种进程场景的真实child峰值与managed均入门，进程树按同时存活进程采样/归集；0/缺失采样判不可用而不绿。真实child瞬态分配超384MiB、树超512MiB、managed超32MiB、采样失败各使同一门红；正常正控绿，raw注明PID/采样范围。

### F-817-R5-02 P1：缩小Meta负载替代原验收（AC-06/08；T-07/08）

bench::big_meta_deck从规定基线改为10个紧凑helper，且实际查询仅call scale(n)，这些helper未被scale调用；新注释称256KiB只属于body-edit。此为未经授权的合同缩减：主Plan§5.4与contract-decision§8（T-01冻结）规定256KiB有意义callee/consumer、含Meta/DAG，六场景各30样本；此前明确要求两编辑硬门按规定负载。收据将meta P95 14.2ms作为闭合证明不成立。

关闭oracle：恢复Meta门规定的有效source/consumer/callee工作量、字段/负依赖编辑与30修订、完整IPC父判时间/内存同轮门；若确需缩减，先由new显式修订/获授权，不能仅换注释或把新限制写进规范。

### F-817-R5-03 P1：结果oracle剩余漏洞、缺真实变异证据（AC-06/07/09；T-07/08）

暖admission用values.values().next()/hir.values().next()，不核具体binding key、consumer_types；只在最后query核数值/HIR，之前query仅检查非空，即错误binding或错误前序值仍能绿。最后step values/hir缺失时先记steps_ok=false又unwrap，可能panic；步数不符仍索引want_step_defs[si]，多步可能越界。需要受控拒绝并保存完整失败报告。

非流式spawn_process_samples仍只看last+any edited、HIR非空和值相等；source_hash/step数量/前序/managed/绑定类型未核。现有turnaround测试只有normal、延迟、非法参数、结构退化，不含收据声称的坏actual/HIR/来源变异。原finding要求证明实际变异经同一gate传播，不接受源码里若干比较替代。

关闭oracle：逐step与预期binding、类型、值、HIR、来源、edited恰一次及身份相符；warm/spawn共享或分别证明同等准入。错key、错type、首query错值、diag、缺HIR、多/少step、错hash等实际wire变异稳定拒绝且gate非零（无panic吞报告），合法响应保持绿。保留错响应原始bytes及expected。

## 实际验证

- cargo test --manifest-path experimental/ac-cte-probe/Cargo.toml --offline --release --test cte_snapshot_tests --test cte_schedule_tests：snapshot13/13、schedule19/19通过；30修订基线通过，snapshot总3.55s。
- cargo build --manifest-path experimental/ac-cte-probe/Cargo.toml --offline --release --bin cte_probe_worker：成功，无告警。
- 重放review-r4/owner-entry-request，改store_root至本轮目录，case-serve实际exit=42，见owner-entry-result.json。新worker wire managed=true、ct_execute=1。
- rustfmt --edition 2021 --check consumer.rs/bench.rs/worker.rs：通过。
- 没有跑整档t/tv/tt/tb/tu/taa。已确认必需门仍缺，故不以再跑未变benchmark正控代替缺失反控；本轮未重跑完整资源/GC/native矩阵，保持pending。

冻结Spec delta SHA256 9C6A34DF33DF0547A6CD037A6CE45ED5580D10D4144CD016CE128318E3B1C5F8（仍未随本轮新语义刷新），报告SHA256 67E1F1D841898C7808E0FD5B11A9224D20AE456A91355A7E44B9E69CEC656C31，均复制到本目录。只读依赖783/796/798/828取该冻结树/lock，没有消费他方在途代码。

AC-01/02旧owner反例本轮通过；AC-03调度定向通过；AC-05/06/08/09因上述合同内问题不通过/partial；AC-04/07及整体AC-11其余完整验收pending（此前驻留时间与结构证据保留，不重开已关闭具体缺陷）。AC-10与merge-owned裸cargo t+tv+消费面pending。整体不能pass。维持executing；T-03对owner修复可完成，T-05/07/08/11待整体收口，完成5/11。下轮须给实际关闭oracle证据及根因/覆盖调整，不能用总测试数代替。未改实现/Spec/ledger，未合入/归档。