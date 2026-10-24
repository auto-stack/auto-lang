# PLAN-817 acceptance matrix（T-08 交付；V-01..V-12 映射于 F-817-R12-01 恢复）

**恢复说明**：修复轮 R7–R11 期间本文件被逐轮改写为仅含修复行，T-08/T-11 的
V-01..V-12 全量映射丢失（R12 复审定为 F-817-R12-01）。本版恢复全量映射：每行绑定
AC/task、实际入口（具名测试）、修正后的 Scope 命令与可持久化证据状态；修复轮历史
原文保留在文末历史节。R12 审查时点快照见主检出
[submitted-acceptance-matrix.md](../../plans/attachments/817/review-r12/submitted-acceptance-matrix.md)。

**证据状态图例**：
- 新跑@`4309f49de`＝当前冻结树上 R11-fix 轮或 R12 复审实际执行；
- 复用＝测试/生产逻辑、依赖、lock、配置未变，末次验证提交随行标注（R12 复审复用裁定在案）；
- merge-owned pending＝统一门禁，最终 review 通过后由 merge 在冻结提交执行、合入前置。

**证据提交索引**（plan-817-dev）：T-02 `b249767d0` / T-03 `ef76a0b4a` / T-04
`6d5e44ac3` / T-05 `6c92decfb` / T-06 `fa4f39c25` / T-07 `5b8abb004` / T-08
`f744c4cd2` / T-11 `4288147ff` / R6-fix `d640fbce4` / R7-fix `2552597ff` /
R8-fix `cb0ba5231` / R9-fix `e55b59cae` / R10-fix `b29cb2348` / R11-fix `4309f49de`。

命令：`powershell -File scripts/verify-ac-cte-817.ps1 -Scope <scope>`。

| V | AC / task | 内容与关键断言 | 实际入口（具名测试/实跑） | Scope | 证据状态 |
|---|---|---|---|---|---|
| V-01 | AC-01/02 · T-02/T-03 | 真实闭环 sum42/bool 分支翻转、HIR 常量原位、native 独立 expected（exit 42 / 2→1） | cte_first_sum42_real_av_and_backfill、cte_first_bool_branch_flips_native_exit、cte_first_store_restore_new_process、cte_first_av_forbidden_counter_control、cte_first_store_hit_skips_av_same_process、cte_first_cross_project_hit（族 6）+ 正式语料 query-sum42/bool | first-case | 复用：族全量末次@`2552597ff`（R7-fix 轮受影响面 60 测含 first）；R8+ 仅触恢复读/审计链，共享恢复面由 cache/freeze 新跑@`4309f49de` 覆盖 |
| V-02 | AC-02 · T-03 | 未走 trap 潜伏、已走溢出诊断具名 callee+调用来源、helper 链、执行事实先于 consumer 裁决 | cte_schedule_trap_latent_in_unexecuted_branch、cte_schedule_trap_executed_diag_names_callee、cte_schedule_trap_helper_chain_names_requested_callee、cte_schedule_execution_precedes_consumer_not_completed（族 19） | schedule | 复用：族全量末次@`2552597ff`；调度/trap/meta 面自 R7 后零改动 |
| V-03 | AC-03 · T-03 | 查询环全链定位（小/三节点）、合法 DAG 单飞不误拒、普通递归合法、fuel/资源分态、取消隔离且不发布 | cte_schedule_query_cycle_small_located、cte_schedule_query_cycle_long_located、cte_schedule_legal_dag_single_flight、cte_schedule_normal_recursion_is_not_a_cycle、cte_schedule_fuel_recursion_resource_class、cte_schedule_cancel_before_request_isolated、cte_schedule_cancel_midflight_publishes_nothing（族 19 含节点/深度双上限两侧） | schedule | 复用：同上@`2552597ff` |
| V-04 | AC-02/04 · T-02/T-04 | 模型结果不冒领执行、五类坏块受控拒（非 miss）、未知 callee/caps 在 AV 前 ct_execute=0、错 producer 语法隔离 | cte_admit_model_results_stay_in_model_domain、cte_admit_wrong_producer_envelope_rejected、cte_admit_unknown_capability_zero_av、cte_admit_trace_capability_denied_zero_av、cte_admit_wrong_expected_type、cte_admit_unknown_owner、cte_admit_duplicate_binding、cte_admit_placeholder_drift_rejected、cte_admit_legal_control_passes（族 9）+ freeze_bad_blocks_all_controlled_rejections、freeze_unknown_callee_language_diag_zero_av | admission+freeze | admit 复用@`2552597ff`；freeze 新跑@`4309f49de`（7/7，含坏块/同键冲突保旧/取消零发布） |
| V-05 | AC-04 · T-02/T-04 | fresh→hit→真新进程/跨工程命中、AV 禁调反控（miss 受控失败、hit 照常 ct_execute=0） | cte_first_store_hit_skips_av_same_process、cte_first_store_restore_new_process、cte_first_cross_project_hit、cte_first_av_forbidden_counter_control + restart 语料（ac_cte_probe 轴） | first-case+cache | first 复用@`2552597ff`；恢复/hit 机制新跑@`4309f49de`（cache 6/6 + freeze 7/7，R8–R12 四轮恢复链改动的每轮定向复验 + R12 复审 3 项） |
| V-06 | AC-04 · T-04 | helper body/签名编辑失效+旧条目完好可回滚、无关面按声明粒度复用、caps 重键、meta 负依赖入身份、hit 不缓存 consumer 裁决 | cache_helper_body_edit_invalidates_and_old_entry_intact、cache_helper_signature_edit_invalidates、cache_unrelated_body_edit_reuses_under_declared_granularity、cache_caps_change_invalidates、cache_meta_negative_dependency_rekeys_identity、cache_hit_never_caches_the_consumer_verdict（族 6） | cache | 新跑@`4309f49de`（6/6） |
| V-07 | AC-04/05 · T-04/T-05 | 坏块受控拒、lease 保活跨 GC、跨工程 root 独立、用户哨兵逐轮字节不变、真实水位物理删除+收敛 | freeze_bad_blocks_all_controlled_rejections、freeze_same_key_conflict_keeps_old_commit、freeze_failed_and_cancelled_steps_publish_nothing、freeze_lease_keeps_entry_alive_across_gc、freeze_cross_project_roots_independent、freeze_user_sentinel_survives_every_pass（族 7）+ resources_real_watermark_gc_physical_delete_and_converge | freeze+resources | freeze 新跑@`4309f49de`（7/7）；GC 水位规模测复用@`2552597ff`（该面零改动；merge 期 resources scope——本轮修正后已真含此测——串行全族兜底） |
| V-08 | AC-05 · T-05 | ≥32MiB 真实页+512 不同输入修订、台账预置先于分配+spike 真红、深输入/wire 界、10s 累计墙钟、384/512MiB worker/树门、编解码预算（R6–R11 六轮资源修复） | resources_thirty_two_mib_pages_and_512_distinct_revisions、resources_worker_peak_under_hard_gates、resources_budget_within_gates_and_pre_reserved、resources_spike_control_is_really_red、resources_deep_input_bound、resources_wire_input_bounded、cte_resources_request_wall_real_ten_seconds + cte_resources_{encode,restore}_reserve_before_alloc_order、cte_resources_oversized_restore_rejected_before_load、cte_resources_corrupt_blob_read_bounded、cte_resources_bounded_read_capacity_exact、cte_resources_probe_sql_error_not_miss、cte_resources_decode_audit_bounded、cte_resources_decode_audit_escaped_scratch_bounded、cte_resources_allocator_gate_sensitivity、cte_resources_request_deadline_fast_control（族 18） | resources | cte_resources_* 12 控制项新跑@`4309f49de`（逐项；含 R8–R11 全部反控与 decode 审计两测，R12 复审复跑 3 项 1/1）；规模三重奏（32MiB/512/GC）与 worker 树门复用@`2552597ff`（重族沿 R11/R12 裁定不重跑；merge 期修正后 resources scope 串行全族执行） |
| V-09 | AC-06 · T-07 | 六场景×30、两编辑硬门 P95<1000ms（正式 release）、raw 逐行 hash/值/PID 绑定、阈值 0/坏样本红、spike/重算/树内存/采样缺失四反控、resident 退化门 | cte_turnaround_six_scenarios_hard_gates_pass、cte_turnaround_gate_ms_zero_control_is_red、cte_turnaround_injected_spike_control_is_red、cte_turnaround_malformed_inputs_all_red、cte_turnaround_resident_degrade_gate_and_control、cte_turnaround_wire_mutations_rejected_no_panic、cte_turnaround_stream_mutations_share_admission、cte_turnaround_child_memory_spike_control_is_red、cte_turnaround_tree_gate_aggregates_concurrent_children、cte_turnaround_managed_budget_control_is_red、cte_turnaround_sampling_unavailable_control_is_red（族 11） | turnaround | 复用：族全量末次@`d640fbce4`（R6-fix 轮 89 测含 turnaround 11）；正式 30 样本报告冻结于 R7-fix 轮（turnaround.jsonl：body 274.4/meta 14.2/resident 234.4ms、树峰 408.8MiB）；bench 面自 R7 后零改动（R12 复审"性能门沿原匹配证据保留"裁定在案）；merge 期 acceptance scope 串行全族执行 |
| V-10 | AC-07 · T-06 | 未知 recipe lint 拒（列注册集）、坏 worker 工具身份=ERROR 行不绿、restart 分块真新进程共享 store、edit 恰一次、stale cte 轴族漂移即红、隔离态与语言 diag 分流 | 官测正式语料 6 case（`cargo test -p auto-lang --features test-cases --lib ac_cte_probe -- --test-threads=1`）+ cte_adapter_protocol_echo_admission、cte_adapter_restart_chunks_share_store_across_processes、cte_adapter_worker_edits_apply_exactly_once（族 3）+ auto-case lint | autodown | 复用：官测 6 case 末次全绿@`9fe341e61`（R2-fix 轮）+ adapter 3 族全量；注册表/语料/协议面自 R2 后零改动（R8 依赖侧 783 追加两 bounded API 为纯追加，收据在案） |
| V-11 | AC-08 · T-08 | 只读消费零 diff（consumed crates）、注册 append-only（master 表比对）、worktree 无 reparse point、覆盖矩阵自检 | verify -Scope closeout（git diff 审计+注册表语义比对+reparse 扫描+矩阵打印） | closeout | merge-owned pending：最终 review 通过后在冻结提交复跑（T-08 原跑@`f744c4cd2`；其后实现历经修复轮演进，closeout 属收尾边界审计） |
| V-12 | AC-11 · T-11 | 四子case ①生产观测：单 session 30 实参修订仅 wrapper 重编译、helper/无关函数零重编译且代际不变、逐值手算 ②身份：车道键隔离、各自命中 ③依赖：deck 级 helper body 失效+session 级仅重编该函数、wrapper 代际不变 ④失败控制：坏编辑受控拒+最后好版本存活、强制冷路径退化计数恰 1 而 30 编辑平坦 | ①cte_snapshot_production_observation_incremental ②cte_snapshot_identity_lanes_do_not_share_cache ③cte_snapshot_dependency_invalidates_and_scopes_recompiles ④cte_snapshot_failure_control_edits_and_degrade_counter；配套：cte_snapshot_retarget_switches_callees_correctly、cte_snapshot_duplicate_callee_and_dag_positive_control、cte_snapshot_failing_call_leaves_no_bad_publication、cte_snapshot_resident_lane_baseline_30_revisions、cte_snapshot_forced_rebuild_control_nonzero、cte_snapshot_cold_first_call_bool_true_and_false、cte_snapshot_i32_bool_i32_lane_transitions、cte_snapshot_cold_bool_miss_hit_and_retry_after_failure、cte_snapshot_cold_bool_legal_language_surface_unchanged（族 13） | snapshot | 复用：族全量末次@`2552597ff`（R7-fix 轮受影响面含 snapshot）；车道逻辑自 R7 后零改动（R8+ 恢复读/审计链经 cache/freeze 新跑@`4309f49de` 覆盖共享机制；R12 复审"T-11 实现/结构证据保留"裁定在案） |

**AC 绑定**：AC-01=V-01/05/10、AC-02=V-01/02/04、AC-03=V-02/03、AC-04=V-04..07、
AC-05=V-07/08、AC-06=V-09、AC-07=V-10、AC-08=V-11+本矩阵完整性（F-817-R12-01 本轮收口）、
AC-09=review（T-09）、AC-10=merge（T-10）、AC-11=V-12（SD-06）。

## 脚本接线修正（R12-01 整理中发现，随本轮收口）

T-08 交付的 scope 过滤串与实际测试名不匹配，自交付起静默漏跑：
- cache/freeze scope 用 `cte_cache_`/`cte_freeze_` 名过滤，而两族测试名为
  `cache_*`/`freeze_*`——**各选中 0 个测试（空转绿）**；
- resources scope 的 `cte_resources_` 过滤漏掉 6 个以 `resources_*` 命名的
  规模/门测试（32MiB 页与 512 修订、真实水位 GC、worker 峰值、预算预置、spike
  反控、深输入界）；
- acceptance scope 的 workspace 全量步骤并行执行，resources 二进制内进程级
  allocator 窗口会被并行线程污染（R11-fix 轮实测并行 10 项红 6 项）。

修正：cache/freeze/resources 改 `--test <binary>` 精确选择（resources 保持串行
并纳入全族 18 测）、workspace 全量步骤 `-- --test-threads=1`。接线验证（`-- --list`
计数，零执行）：cache=6、freeze=7、resources=18；其余 scope 过滤串实配
（first=6、admit=9、schedule=19、turnaround=11、snapshot=13）。测试本体与既有
证据不受影响——历轮收据均以直跑/`--test` 选择执行，漏跑仅存在于脚本聚合入口；
修正后 merge 期 scope 才首次真正全族执行。

## 修复轮历史（R7–R11 原文保留）

| 轮 | 修复内容（原文） | 面 | 验证 |
|---|---|---|---|
| R7 修复 | **F-01（恢复预留非实际分配上界）**：恢复链四段有序——零分配大小探测（经公共 `ManagedStore::db()` 只读 SQL `chunks.size`，framed=48B 头+载荷）在**读取/重组上限处**先行拒绝超界载体（加载器零调用）；载入预留=loader 真实瞬态足迹（framed Vec+unframed 拷贝=framed+payload）；加载长度须与探测一致（内容寻址漂移=受控 infra 拒）；decode Hot 预留保守 2× 载荷（serde 结构/容量开销+同时存活，撤销"JSON 文本≥结构"假设）。R7 反例重放：真实执行提取 identity→新 store 经公共 put 植入 2MiB→驱动请求**读取前受控拒**（frozen_load_attempts=0、无 load 预留、未发布、probe 数字与植入真值对照）；正常 hit 正控保持 | resources | cte_resources_ 12（新增 oversized_restore_rejected_before_load）；R6 各反控维持 |
| R8 修复 | **F-01（db 长度不能限定真实 blob 读取）**：783 追加式有界读取 API（依赖单写适配：`read_chunk_gated_bounded`/`get_artifact_bounded`——File::open 后句柄 metadata 预检（真实长度>额度=ResourceExhausted 零内容分配拒绝）+ `Read::take` 硬界读取（metadata/read 间漂移/文件替换/增长恒有界），CAS 准入保留）；817 恢复链改 `load_frozen_payload_bounded(cap=framed+payload 预留额)`。allocator 级反例重放：512B 合法载荷（预留 1072B）→blob 文件替换 2MiB 垃圾（db 元数据不动）→驱动请求在物理读取界受控拒，**观测窗口最大单次分配 <512KiB**（2MiB 从未读入；全局 allocator wrapper 记录）；界内正控同路 max≤512KiB。**F-02（probe 吞 SQL 错）**：`.optional()` 只把"确无行"当 miss；SQL 执行/IO/schema 错误传播为受控 infra 拒（driver 不再借此启动 AV 重算/发布）——反例：驱动自身连接上 rename chunks 表→请求 Diag("size probe failed: no such table")且 ct_execute=0/未发布；空 store 真 miss 正控正常执行 | resources | cte_resources_ 14（新增 corrupt_blob_read_bounded+probe_sql_error_not_miss）；受影响面 restore/oversized/encode/deadline+cache/freeze 17 项全绿；meta-edit 计时漂移经干净 R7 树同机 A/B 归因环境非代码（1234 vs 757ms，hot 反而更快），turnaround.jsonl 保持 R7 冻结报告 |
| R9 修复 | **F-01（take 不限 Vec 扩容/共存少算）**：783 bounded 读改**精确容量**——metadata 预检后 `vec![0u8; real_len]` + `read_exact`（单次内容分配==已验证文件长，read_to_end 倍增扩容消除；metadata 后缩短=UnexpectedEof 受控 Io、增长=截断帧由 CAS 拒）；driver 恢复链**分账预留**——frame 读取额=framed 探测值（loader cap，任何超探测帧长的文件在读界拒）与 payload 副本额（frame−48，覆盖 get_artifact_bounded 内 to_vec 与 frame 共存峰值 2×frame−48）各自独立、互不复用；事件流 frame-read/payload-copy 分笔。allocator 观测升级**共存存活峰值**（alloc−dealloc 水位）：9 尺寸（500..2056 跨 2^n 步进）×4 态扫描——合法（单次≤frame+杂项、共存≤两笔授权额）、framed+1（读界拒、零内容分配）、同长垃圾（读满 framed、单次==framed 后 CAS 拒）、缩短（受控拒）；driver 全链 2MiB 旧例/512B 正控（单次+共存双指标）维持 | resources | cte_resources_ 15（新增 bounded_read_capacity_exact；corrupt/restore/oversized/encode/deadline 适配新事件名全绿）；unused_mut 清零 |
| R10 修复 | **F-01（2× payload 非 decode 分配上界）**：decode 额度改**零分配结构审计**——JSON 容器计量（字符串内容 S×2 + 数组元素 A×最宽槽 3×24B×Vec 倍增系数 2+32B 粒度 + 固定项，类型布局锚定）；非法/超深 JSON 在审计处受控拒（from_slice 前置）；复审反例重放（真实 identity+公共 put 植入 closure 空串）：N=100000 审计 17.6MB>Hot 8MiB 在预留处拒（窗口无 decode 级分配）、N=10000 审计 1.76MB 放行且窗口实际分配≤审计值（旧 2× 额度下 393216B 单次 vs 62292B 授权）、N=0 正常 hit。**F-02（encode 精确长度≠容量）**：serialize 改 count→with_capacity→to_writer（容量==授权，增长链 2048>1163 根除）；encode 测试加容量断言（capacity==count+窗口单次≤授权）+增长型 encoder 红控。**F-03（观测器漏检）**：窗口 API 改 begin/readout 分离（读不重置、基线取自 begin 时水位）；INCIDENTAL 8192→2048；sweep 加 100_000B 内容主导档；新增敏感性红控（旧 take+read_to_end 复刻在同一 blob 上共存足迹>授权阈值——检查器对旧算法必红、新算法绿） | resources | cte_resources_ 17（新增 decode_audit_bounded+gate_sensitivity，encode 加容量断言/红控）；受影响面 13 项全绿；hot 场景审计冒烟 63.5ms 正常 |
| R11 修复 | **F-01（审计 2S 未覆盖转义 scratch 与 owned String 共存）**：字符串额度模型 2×S → **S + 2×T**——S=字符串内容字节总和（serde_core StringVisitor to_owned 使全部 owned 副本同时存活于解码结果，decoded≤encoded）；T=最大单 token 内容（serde_json 1.0.154 `Deserializer::scratch` 为**全文档共享单 Vec**、逐 token clear 保容量、Vec 倍增封顶 2×单 token；**转义字符串**（SliceRead::parse_str_bytes→Reference::Copied）与**超长数字字面量**（parse_long_integer/parse_decimal 数字入同一 scratch）同为 scratch token——同一共享缓冲一项覆盖两分支，非转义串走 Borrowed 零拷贝）；scratch 与全部 owned 同时存活的稳态最坏=S+2T 恰为授权值，增长期 realloc 瞬态（prev-owned+1.5×cap≤(S−T)+3T）亦在界内。R11 反例重放（红相在冻结基线 b29cb2348 复现后转绿）：三档 escaped/plain 配对（source_fp=x·N+\n+x·N vs 同解码长度 plain；N=32768/65536/131072）decode-only 窗口共存峰（197387/393995/787211B）≤新授权；旧 2S 字符串项+16KiB 松弛被超越（敏感性红控——回退 2S 必红）；10 万位数字字面量（exec_steps 拼接）审计≥2×位数且 decode 窗口界内；审计自身零分配；驱动默认 cap 下外源 envelope 到达 schema 受控拒（ct_execute=0/不发布）。R10 空串数组/正常信封/encode/观测器各控维持 | resources | cte_resources_ 18（新增 decode_audit_escaped_scratch_bounded）；受影响面 23 项全绿（resources 定向 10 逐项+cache 6+freeze 7） |
