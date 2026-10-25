# PLAN-827 验收证据绑定矩阵（R5 闭包包，2026-10-25）

规则（review-r5 "Reproducible closing package"）：每行 = V/子案例 → 精确测试/实际入口 →
正/负断言 → 输入/工具/运行时版本 → 留存 raw → scope 选择映射。继承测试的同一性核验：
本计划唯一树与 D798/D828 交付在同一合入基面（base 369e1c349，实现/依赖/配置/输入未变，
R5 绑定时这些文件未改；R6 修复在 AV 帧槽位 reserve 边界新增最小探针，执行器相关证据以 4a069ce7f 定向测试重新核验），故按"实现、依赖、配置与输入身份未变"显式复用。

## V-06 工件准入（scope=admission）

| 子案例 | 精确入口 | 断言（正/负） | 版本/raw | scope 映射 |
|---|---|---|---|---|
| 好 hash 坏 ABC（自洽帧坏内层） | `abc_semantics_tier::abc_admission_tampered_inner_never_executes`（继承） | 负：篡改内层永不执行 | 同基面 | admission → admission-inherited-abc（必需成员） |
| 截断/尾随/坏 wire | `abc_semantics_tier::abc_codec_rejects_truncation_and_trailing`（继承） | 负：decode 拒 | 同基面 | admission-inherited-abc |
| 手工 Decoded/伪身份 | `cache_tier::av_cache_wrong_identity_rejected` + `av_cache_valid_payload_under_wrong_body_key_rejected`（继承） | 负：身份/键不匹配拒 | 同基面 | admission-inherited-cache |
| 帧坏内层（读侧） | `cache_tier::av_cache_corrupt_frame_rejected`（继承） | 负：envelope 校验拒 | 同基面 | admission-inherited-cache |
| 伪 producer/kind（native 面） | `native_admission_tier::native_admission_identity_axes_rejected`（继承） | 负：身份轴拒 | 同基面 | admission-inherited-native |
| 合法单函数完整模块（正控） | `abc_semantics_tier::abc_admission_only_verify_produces_executable_handle` + `native_admission_tier::native_admission_real_coff_positive`（继承）；**827 薄重放** `av_source_827_semantics::av_source_827_consume_gate_corrupt_abc_never_executes` | 正：verify→execute=7/0；负：corrupt bytes decode 必败 | AP-01 冻结 SHA；raw=admission-thin-consume-gate.log | admission-inherited-* + admission-thin-consume-gate |

## V-08 当前授权/身份/root/lease/坐标（scope=cache）

| 子案例 | 精确入口 | 断言 | scope 映射 |
|---|---|---|---|
| **撤销当前授权阻断缓存代码调度** | **827 薄重放** `av_source_827_trace_grant_revocation_blocks_execution` | 负：caps=[] 时 source.undefined 诊断且 av_execute=0（授权参与键，无陈旧调度）；正：带授权运行 ok | cache-thin-grant-span |
| 恢复期当前授权活体 | `cache_tier::av_cache_restore_requires_current_trace_grant`（继承） | 负/正：无活授权拒、有则过 | cache-inherited-lifecycle |
| span 坐标重绑（位置变化） | **827 薄重放** `av_source_827_position_shift_rebinds_trap_span_coordinates` + `cache_tier::av_cache_position_drift_keeps_trap_span`（继承） | 负例正用：trap span start 恰平移 +2、长度/函数稳定（真实新坐标，非"非空"即可） | cache-thin-grant-span / cache-inherited-lifecycle |
| root/lease 隔离与释放 | `cache_tier::av_cache_gc_and_lease`、`av_cache_consumption_leases_cover_siblings_and_release`、`av_cache_snapshot_manifest_lease_survives_position_edit_gc`（继承） | 正/负：lease 覆盖/释放/GC 存活 | cache-inherited-lifecycle |
| 依赖失效/命中生命周期 | `cache_tier::av_cache_hit_lifecycle`、`av_cache_dependency_invalidation`（继承） | 正/负：恰改单元失效 | cache-inherited-lifecycle |
| 跨根身份 | 827 既有 `av_source_827_cross_project_content_addressed_reuse` + 同键异内容 Conflict（cache 档） | 正：同内容复用仍执行；负：异内容 Conflict | cache |

## V-09 资源（scope=resources）

| 子案例 | 精确入口 | 断言 | raw/scope |
|---|---|---|---|
| cap-at/超1（输入轴） | **827 薄重放** `av_source_827_input_cap_at_and_over_one` | 正：恰 4MiB 过前门并编译=0；负：+1B=admission/av.resource.input-size 且 dispatch=0 | resources-thin-cap-pair |
| cap-at/超1（表/嵌套轴） | `resources_tier::av_resources_over_table_budget_is_resource`、`av_resources_front_door_nesting_boundary`（继承） | 正/负对 | resources-inherited-finite |
| 无限循环/cancel/资源态 | 827 既有 `av_source_827_resource_budget_and_cancel_axes` | InstrSteps 资源态/Cancelled（host 决策）/dispatch=0 | resources-thin-cap-pair |
| churn/清理（32MiB 信封内） | `resources_tier::av_resources_32mib_product_random_read_and_churn`（继承） | 预算内清理与水位 | resources-inherited-finite |
| 双线内存硬门 + 两路径 P95 | `av_probe_worker bench-turnaround --samples 30 --gate-ms 1000`（validate_and_gate 对 frontend_transient/worker_private 双线 hard_gate_failures→exit5）+ `t08_formal_campaign_v08_dual_gate_30_revisions`（正式会话时间门） | P95<1000 且双线绿；gate-0 必红反控；**R7 F-R7-02：worker 先从冻结 workspace 构建（cargo build --release -p auto-av-probe --bin av_probe_worker），SHA256+head+probe-ws 脏项入 per-run 绑定收据署名两 raw，陈旧/篡改/缺失绑定负控先于战役必红+当前正控** | raw=bench-*.jsonl+resources-worker-binding-*.json（失败轮保留）；当前绑定过门轮 20261025-091045（worker 0A50E300E1B4BC97…、head=10c1456e2、single-body-edit P95=598.0/signature-conflict P95=24.8）；失败轮 085917 归因并发编译撞车，样本+收据保留 |

注：828 战役测试内内存峰值为报告项（其注释明示）；**内存硬门在 bench-turnaround 的
validate_and_gate**（同 scope 必跑、失败 exit 5）——本行即"actual dual-line hard-memory/formal
two-path evidence"到当前基面的绑定。故障第 n 次分配注入现已在 R6 闭包，见下节实际分配/清理反控，不再保留“无注入缝”的未完成项。

## V-13 依赖/授权/生命周期/raw（scope=incremental）

| 子案例 | 精确入口 | 断言 | raw/scope |
|---|---|---|---|
| 依赖：helper body 精确失效 | 827 `av_source_827_incremental_two_fn_local_updates`（**R5 增身份断言**） | gcd 编辑：恰 d_gcd 安装、main 阶段全零**且 main (ptr,generation) 不动**（接口只读依赖） | incremental |
| 授权：撤销当前授权（会话再入） | **R5 增断言** `av_source_827_incremental_precise_refusals` 尾 | 负：warmup(trace 源, caps=[]) 必 Err（与"新增未授权文本"不同输入） | incremental |
| 授权：新增未授权文本 | 同测试 mark_a 块 | 负：拒绝且修订保留 | incremental |
| 旧帧/值/drop 生命周期 | `function_incremental_828_contract::{contract_828_transaction_sequential_cancel_then_commit, contract_828_transaction_close_releases_owned_entries, contract_828_transaction_prepare_does_not_publish_and_drop_is_inert}`（继承） | cancel 保基/close 释放/prepare 不发布 | incremental-inherited-lifecycle |
| 逐修订结构 raw | **R5 增持久化、R7 F-R7-01 升级** `av_source_827_incremental_d828_baseline_30_structure_and_p95` | 每修订 {changed_fn,install_set,value,wall_ms,identity_hold} **断言前落盘**；unchanged before/after 缺对必拒（具体 (ptr,generation) 对，非 null）；最终 structure/P95 verdict 先于断言写入；失败运行证据不被后续运行覆盖 | **每运行文件 incremental-raw-<run_id>.json（不可变证据）+incremental-raw.json（last-completed 指针，含 run_id）**；shape 负控 `av_source_827_incremental_raw_shape_rejects_missing_baseline`（null before/缺 after/短对/零修订全拒）+失败留档负控 `av_source_827_incremental_failing_verdict_still_writes_raw`（negatives/ 留档，不动真实运行文件）均接 incremental 必需清单 |

## R7 证据闭包（2026-10-25，冻结 10c1456e2+ebced8cd5）

- **F-827-R7-01（V13 持久化身份/失败 raw）**：根因=identities_before 裸名键 vs 查询 `d_main` 前缀键（30 行 before 全 null）+raw 仅在 P95 断言后一次性写固定名。修复/重放见上表 V-13 行；评审 replay-evidence.ps1 复放 null_before=0 exit0。
- **F-827-R7-02（V09 bench 二进制未绑当前源码）**：根因=resources 档直接执行 target/release 旧 exe（depinfo 缺 frame_allocation.rs，写于 R6 前）。修复/重放见上表 V-09 行；selftest 增绑定校验五控制（外部二进制/篡改收据/缺失 exe/空哈希红+精确绿）。R6 执行器变更后的 30 样本资源证据已在新绑定 worker 上再生（091045 轮）。
- **R6 core 写窗对账**：frame_allocation.rs 写入 ac-av-probe/core 系用户明确指令的最小故障探针；最终写面含 crates/auto-lang **计划内测试面**（§3 既定 corpus+根门测试+模块注册），生产 crates 源码与 D798 公共 API 零触碰。


## R6 故障注入闭包（2026-10-25）

- **故障第 n 次分配注入已实现**：4a069ce7f 的 `core/src/frame_allocation.rs`，AV 根帧/调用帧实际槽位 `try_reserve_exact` 前触发。`FrameAllocationProbe` 线程局部作用域指定第 n 次；未启用时不注入。统计 attempts/successful/released/live_bytes/peak_bytes；槽位 Vec 随 Frame 实际 Drop 后回收记账。
- V09 必需成员：`av_source_827_nth_frame_allocation_failure_cleans_owned_slots`（n=1/2/3失败、n=4未到达正控、零frames/slots预算attempt=0、callee预算先拒、合法trace前缀保留）；`av_source_827_frame_allocation_pause_cancel_trap_cleanup`（跨暂停恢复n=2失败、暂停pin存活/取消后Weak消失、丢弃未完成链、trap/fuel退出全部释放）。两测试已绑定 resources-thin-cap-pair 必需ID。
- 真实反控：关闭注入分支的受控变异使n=1测试失败exit101；恢复后两测试通过。证据 `[work-r6](work-r6/readiness.md)`。
- 覆盖真实帧槽位储存分配，不替换全局allocator，不宣称控制parser/store/诊断Vec所有进程分配。旧root/store租约证据仍由继承生命周期成员负责；本次新增的是执行槽位及暂停快照拥有链的故障清理证据，原合同不删项、不归后续owner。
## 受控失败传播

每组必需成员经脚本必需 ID 精确绑定（R3 语义：缺失/改名/重复/真跳过必红）；移除任一继承行
（如 `abc_admission_tampered_inner_never_executes`）→ 对应 scope exit 1 → acceptance exit 1。
现有合法正控保持绿（受影响 scope 本轮实放）。
