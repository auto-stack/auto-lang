# PLAN-828 T-09 覆盖清单（r3 合同回写：case → 真实入口 → 判据）

日期：2026-10-22（r3 回写，替代 2026-10-20 r2 版）｜ 基线：`plan-828-dev` @
`f7d1091b3`（T-08 战役冻结）+ 本批 T-07 完整轴/T-09 收口提交 ｜ 验证命令：
`cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test <target> -- --test-threads=1`

本文件按 r3 合同（§5.6–5.8、§6 矩阵）把每个 V/C case 映射到**实际测试入口、
oracle 与提交版本**；r2 版的"开放项 3（V-08 双线内存）"已由本批关闭（见 V-08 行），
"开放项 2（body 事实消费者）"已由 T-03/B1-C03 机器关闭（见 V-03 行）。测试总数是
选择面证据，不是逐 case PASS 的替代；每个入口的原始 raw 见对应测试输出。

## 1. V-case 映射

| V-case | 真实入口（文件::测试） | oracle | 版本 |
|---|---|---|---|
| V-01 仅一函数变化 | `function_incremental_828::t01_v01_single_body_edit_full_path_incremental`（双函数首例：值/11轴计数/安装集/身份）；`::t05_generations_and_stable_ids_across_revisions`；`contract::contract_828_hotpath_real_session_edit_is_single_body_window`（生产者窗口门）；`::t07_v07_512_revisions_bounded_history_stable_watermark`（冻结负载 512 修订逐修订结构门）；`t08::t08_formal_campaign_v08_dual_gate_30_revisions`（正式战役结构门） | 独立手算期望；未改函数零阶段/零整模块；安装集恰为改函 | 3e60af2c8→T-05/06→f7d1091b3 |
| V-02 来源/语法 | `function_incremental_828::t02_frozen_load_projection_parity`、`::t02_syntax_error_candidate_rejected_keeps_old`、`::t07_trap_span_maps_through_line_col`；span/mint 全矩阵=`function_incremental_828_r1`（86 测：r2a..r22a 编辑矩阵+sidecar 恢复族）；`scalar_projection_` 差分（ac-syntax，6 测） | 与 ac-core scalar parser AST+span 全等对拍；语法错误拒绝保旧版；trap span 随编辑映射 | 85cbebea4→3090f2326 |
| V-03 精确依赖 | `function_incremental_828::t03_body_credentials_are_context_bound`、`::t03_signature_change_is_non_fast_path_and_keeps_old`；body 事实查询核心=`b1::t03_body_fact_query_dependency_invalidation`（产物归属模型全矩阵）+`b1::t03_r2_reread_does_not_refresh_old_dependency`/`t03_r2_history_retains_its_own_body_dependencies`；intrinsic 事实=`r1::r2a_credential_invalidates_when_referenced_intrinsic_changes`、`r1::r3a_credential_tracks_each_intrinsic_reference_not_just_symbol_set`、`r1::r3a_credential_tracks_effect_fact_of_consulted_intrinsic`；只读接口 caller=V-01 计数轴 | 凭证绑实际环境/owner/域；已读事实变化→消费者失效，重读不刷新旧凭据；未读不失效 | 14c929a5d→599cc15af→a66d9d9e5 |
| V-04 信任 | `function_incremental_828::t04_unit_admission_negatives`、`::t01_v04_rejected_candidate_keeps_current_version`；A07 五门准入=`b1::t03_a07_artifact_admission_controls`（10 模式逐门拒）+`b1::t03_a07_roundtrip_plain_and_mixed_intrinsic_adoption`+`function_incremental_828::t03_a07_fn_hir_tampered_row_is_skipped_not_trusted`；链接门=`b2::link_foreign_owner_same_name_is_refused`、`b2::link_unresolved_called_import_refuses_before_dispatch`、`b2::execution_real_link_rejections_and_unit_cannot_execute`、`contract::contract_828_dispatch_real_link_rejections` | 准入前 dispatch/install=0；坏 payload 拒于对应门；unit 不可直接执行；旧版保持 | 14c929a5d→cfa67eece |
| V-05 版本可见性 | `function_incremental_828::t06_v05_deterministic_barrier_keeps_inflight_chain_on_its_snapshot`；三入口一致=`contract::contract_828_dispatch_default_three_entries_unit_consistency`；`b2::execution_pinned_self_and_mutual_recursion_keep_old_trace_and_payload`；`r1::r6a_undo_and_paused_snapshot_keep_their_distinct_source_maps` | 步数屏障（非 sleep）；旧链整链旧 callee/旧值；新顶层取新；三入口同派发 | 168b6b5658→cfa67eece |
| V-06 失败/迟到 | `function_incremental_828::t06_v06_stale_candidate_never_overwrites_newer_result`；事务矩阵=`contract::contract_828_transaction_*`（8 测：取消/提交/stale 基面/foreign/后置 verify 拒/close/公开 recheck 发布失效/失败 recheck）；`t07::t07_c05_candidate_pair_dies_at_consumption_not_before`；分段预算=`b2::link_repeated_finite_resume_budget_makes_progress`/`link_zero_resume_budget_pauses_without_progress`；r23a 关闭面=`review-r23a/close-controls.rs`（6）+`close-adjacent.rs`（3）经 `run-controls.ps1` 原样重放 | 失败/迟到/取消不覆盖当前根；候选在消费点死亡；fuel/trap 退出释放 | 0b612fda1→27aaa34d5→cfa67eece→cccb09b1f |
| V-07 恢复/回收 | 单元恢复=`function_incremental_828::t07_restore_readmits_and_miss_falls_back`；sidecar 适用性/修复=r1 r18a..r22a（缺行/坏 v2/错函数/重复指令治愈矩阵）；manifest 消费门=`t07::t07_f11_manifest_matched_absent_and_rejected_paths`；目录严格性/required 边=`t07::t07_r2_manifest_strictness_and_required_references`、`t07::t07_r2_superseded_units_reclaimed_and_user_row_survives_gc`；迁移=`function_incremental_828_mig`（9 测：legacy 全路径/缺块自愈/重建闭包双指纹/幂等重开）；**完整存储轴=`function_incremental_828::t07_v07_full_storage_axis_512_edits_gc_restart_sentinel`（冻结负载 512 真实编辑+changed-only 持久化+周期真实 GC 水位有界+两次中途 warm restart 全单元重验+pin 保活/释放死亡+用户哨兵逐字节幸存）**；驻留水位=`t07::t07_f09_resident_inventory_bounded_watermark_and_close_release`；pin/退役回收=`t07::t07_v07_old_snapshot_pin_hold_then_release_reclaims`、`t07::t07_v07_retired_history_dies_unpinned_and_survives_pin`；哨兵浸泡=`t07::t07_v07_user_sentinel_row_untouched_by_soak_and_restore`；**HIR soft root 有界**（发布新 cv 退役旧 root/内容回访重挂/restore 按**行存在**置归属追踪器——与 untrusted HIR 可采纳性分轴，B3-C01：合法 span 拒绝恢复不漏退役，计数 `hir_soft_root_retired`/`hir_soft_root_rerooted`） | 恢复重验（无信任落盘）；持有时旧代码可用、释放即回收；水位收敛不随编辑数增长；每轮值独立正确 | d93a9e373→3090f2326→cccb09b1f→本批 |
| V-08 双门 | 进程内=`function_incremental_828::t08_v08_formal_p95_and_structural_dual_gate`；**正式战役=`t08::t08_formal_campaign_v08_dual_gate_30_revisions`：真实常驻 worker+IPC+父侧 judge，计时窗=编辑入队→父侧**全部校验完成**（B3-C04 终点后置），n=30，P95<1000ms 且全对且结构门绿**；结构门含 B3 三门：修订进度（revision==req_id）/确切安装目标（`expected_install`）/未改函数派发载荷身份稳定（`session::dispatched_identities`，worker 逐发布上报，B3-C02/C03——stale 修订/错目标/真实深复制执行副本各自使同一总门红）；完整观测（逐样本计数差分/诊断/身份/相位）与 raw 经 `JudgeReport::write_evidence` 落盘留存（成功失败同留，B3-C04）；初始化 `init_ms` 单列；**双线预算判决（T-07 收口，同测试内硬门）**：A 线=冻结负载前端编译瞬态隔离测量（`av_probe_worker frontend-attribution`，798 §5.9 同口径）≤224MiB；B 线=worker 采样峰值−A 归因 ≤128MiB（缺归因整峰计 B） | 双门：粒度红不能被时间绿掩盖；发布绑定当前修订与确切目标；复制执行必红；双线均为硬门断言；实测见 interface-decision T-08/T-09/B3 修复节 | f7d1091b3→B3 修复批 |
| V-09 反控 | 门负控=`t08::t08_judge_gate_negatives_each_axis_red`（零门/错期望/流中断各自独立红）+`function_incremental_828::t08_v09_gate_negatives_redden_independently`；旧管线负控=`contract::contract_828_hotpath_old_synth_pipeline_same_window_is_red`、`contract_t01r::t01_review_real_old_pipeline_must_invalidate_same_structural_gate`；副本负控=`contract::contract_828_dispatch_payload_identity_axes`（强制复制未改执行 payload 同门必红）+`b2::execution_changed_slots_pinned_callee_chain_and_real_copy_negative`；坏工件负控=V-04 行 A07/链接门 | 每轴独立可红；counter 绑真实入口并以身份/trace 复核 | 8be89bf5d→f7d1091b3 |

## 2. r3 代表性补充矩阵映射（C01/C02/C05）

| Case | 真实入口 | 版本 |
|---|---|---|
| C01-01 | `contract::contract_828_frontend_single_body_matches_cold_compile`+`contract_828_hotpath_real_session_edit_is_single_body_window`+V-01 计数轴；负控=旧 synth 管线同窗红（见 V-09 行） | 3e60af2c8→85cbebea4 |
| C01-02 | `r1`（86：bool/短路/命名实参/条件for/递归/槽数/长度变化矩阵）+`scalar_projection_` 6+`t02_frozen_load_projection_parity`；授权正负控=`r1::r3a_credential_tracks_*` | 85cbebea4 |
| C01-03 | `b1::t03_body_fact_query_dependency_invalidation`（完整矩阵：已读失效/重验/remint 恢复/未读不失效/只读接口轴）+`r1::r3a_*` | 14c929a5d→599cc15af |
| C01-04 | `contract::contract_828_frontend_rejects_erroneous_body_and_restores_env`+`function_incremental_828::t02_syntax_error_candidate_rejected_keeps_old`+`::t01_v04_rejected_candidate_keeps_current_version` | 3e60af2c8 |
| C01-05 | `contract::contract_828_frontend_caller_entry_untouched_control`+`contract_t01r::t01_review_one_emission_must_not_regenerate_unchanged_body_documents`+`contract_828_frontend_rejects_signature_drift` | 23a6b35a0→85cbebea4 |
| C02-01 | `contract::contract_828_dispatch_payload_identity_axes`+`contract_828_dispatch_default_three_entries_unit_consistency`+`function_incremental_828::t05_generations_and_stable_ids_across_revisions` | 168b6b5658 |
| C02-02 | `b2::execution_heterogeneous_unit_tables_named_arguments_all_slot_orders`+`b2::link_reordered_units_use_their_own_span_maps`+`b2::projection_distinct_admitted_functions_sharing_producer_slot_cannot_cross_maps`（共享生产槽反例）+`function_incremental_828::t04_content_addressed_reuse_hits_previous_revision` | cfa67eece→3090f2326 |
| C02-03 | `b2::execution_pinned_self_and_mutual_recursion_keep_old_trace_and_payload`+`contract::contract_828_dispatch_default_three_entries_unit_consistency` | 168b6b5658 |
| C02-04 | session.rs 组件内 `component_link_failure_keeps_all_installed_state_then_accepts_good_candidate`（真实生产 commit 全状态图像）+`b2::link_foreign_owner_same_name_is_refused`/`link_unresolved_called_import_refuses_before_dispatch` | cfa67eece |
| C02-05 | `contract::contract_828_dispatch_payload_identity_axes`（副本负控）+`b2::execution_changed_slots_pinned_callee_chain_and_real_copy_negative` | 168b6b5658→a66d9d9e5 |
| C05-01..04 | r23a `close-controls.rs` 6 + `close-adjacent.rs` 3（`run-controls.ps1` 原样重放；等价表示调整按 D-03 记录）+`t07::t07_c05_candidate_pair_dies_at_consumption_not_before`+`contract::contract_828_transaction_close_releases_owned_entries` | 47894c2e6（r23a） |

## 3. 批次复审 finding → 永久断言映射

| finding | 永久断言（同输入同 oracle 收编） |
|---|---|
| B1-C01..C05（R1 五项） | `b1::t03_a07_artifact_admission_controls` / `t03_a07_roundtrip_plain_and_mixed_intrinsic_adoption` / `t03_body_fact_query_dependency_invalidation` / `t03_close_releases_cold_bundle_owner` + codec v2 稳定 id 表 |
| B1-R2-01 / C03 / R2-02（R2 三项） | `b1::t03_r2_assignment_target_change_is_new_content_not_old_code` / `t03_r2_assignment_value_change_positive` / `t03_r2_reread_does_not_refresh_old_dependency` / `t03_r2_history_retains_its_own_body_dependencies` / `t03_r2_semantic_wrong_target_or_entry_not_adopted`（can3→can4 全字段内容身份） |
| B1-R3-01（can4 moved/overflow 字段） | `b1::t03_r3_call_binding_moved_distinguished_at_restore` + `t03_r3_dependencies_cancel_and_late_do_not_refresh_history` |
| B2-C01..C04（第二批四项） | `b2::link_*` 8 件 + `execution_*` 4 件 + session.rs `component_link_failure_*` |
| B2-C03 残项 / T07-R2-01 / T07-R2-02（第二批 R2 三项） | `b2::projection_distinct_admitted_functions_sharing_producer_slot_cannot_cross_maps` + `t07::t07_r2_manifest_strictness_and_required_references` / `t07_r2_superseded_units_reclaimed_and_user_row_survives_gc` |
| T07-R3-01（manifest 键身份，R3/R4 两轮） | `function_incremental_828_mig` 9 测（format=2 双指纹：body fp + 规范化 required 闭包 fp） |
| B3-C01（未采纳 A07 漏退役旧 root） | restore 按**行存在**置 `persisted_hir_cv` 归属（与采纳分轴）；复审 `hir-roots-controls.rs` 两控制收编语义（位置变化例 root 有界、正控保持） |
| B3-C02/C03（judge 未绑修订/目标；复制执行门缺失） | 父侧 judge 三门（revision==req_id/`expected_install` 精确匹配/未改函数派发身份稳定）；复审 `judge-controls.rs` stale/install/copy 三红控 + clean/whole/wrongvalue 三绿控 |
| B3-C04（计时终点/raw 丢弃） | 终点后置至全部父侧校验后；`JudgeReport::write_evidence`（report+逐样本 jsonl 落盘）；`init_ms` 单列；样本保留 worker 相位/计数差分/诊断/身份 |
| B3-C05（文档失实） | successor（resident_weak 签名/dispatched_identities/restore 归属语义）、SD-01（hit/miss 真实顺序：document→hit/miss→miss 才 verify）、coverage（b1=11 实际选择/raw 归类）按最终代码回写 |

## 4. 旧消费者兼容证据（T-09 定向回归）

- 828 面：`function_incremental_828` 21（含三件 512：soak/旗舰/完整存储轴，日常重放显式
  skip、里程碑全跑）、`_r1` 86、`_contract` 22、`_contract_t01r` 4、`_b1` 11、`_b2` 14、
  `_t07` 8、`_t08` 4、`_mig` 9、probe lib 组件 1。
- 798 面（整模块入口/LayerSpec/A09 身份原样保留的回归）：`cache_tier` 20、`mir_tier` 8、
  `abc_semantics_tier` 15、`binding_tier` 4、`vertical` 3、`native_admission_tier` 11、
  `native_trace_parity` 1、`case_binding` 5、auto-av-core 22。
- r23a 关闭面：`close-controls` 6 + `close-adjacent` 3 原样重放。

## 5. 开放项登记（非 required skip）

1. **V-02 错误恢复树 span 全等对拍**：r3 合同面为拒绝语义（语法错误候选拒绝、
   旧版可用、修好后出新结果——C01-04 已闭）；错误恢复树的逐 span 全等非本计划
   验收项（首版无错误恢复消费方），语言错误恢复落地时按后继接缝补轴。
2. **跨计划既有债（非 828 引入）**：`shim-metadata` bin 4 个 dead-code 告警为
   master 既有（PLAN-596 面，828 分支零触及，本批仅清 R3 复审点名的 shim doc
   注释告警）；登记 KNOWN-DEBT 候选，归 shim owner。
