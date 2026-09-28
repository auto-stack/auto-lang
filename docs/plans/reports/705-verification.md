# PLAN-705 T-08 验证报告（verification）

时点：T-01..07 完成（branch `plan-705-dev`，T-07 后 commit `71aa3d71d`）。
基线：`025fb192c`（2026-09-28）。本报告逐 AC 汇总证据；门禁命令与分类
结论在案，独立复审（`/auto-plan:review`）另阶段执行。

## 1. 门禁结果

| 门禁 | 命令 | 结果 |
|---|---|---|
| 快速编译（双形态） | `cargo check -p auto-lang`（default）/ `--no-default-features` | **0 error** 双形态 |
| HTTP 门禁 | `cargo th`（=nextest `--features test-http-e2e http_e2e`） | **40/56 pass**；16 红全部定性（见 §3） |
| 全量门禁 | `cargo tf --no-fail-fast`（全量 5832 项收口） | **5819 pass / 13 failed**——10 项 base 预存红（base worktree `025fb192c` 逐项复证）+ 3 项 dep/ffi_dual oracle 冷构建争用偶发（双树隔离复跑全绿），见 §2 |
| 定向族 | plan705 / plan702 / plan326 / client 矩阵 / 广谱 | 11/11、7/7、79/79、63/63、522/522（T-06 parity 报告） |
| 新增告警 | cargo check 输出 grep | 本计划新增文件（async_http.rs/plan705 探针）零告警；总告警数为基线噪声（base 同源） |
| 格式 | rustfmt --check | 本仓无 rustfmt.toml，base 即非 rustfmt 形（base engine.rs 同样 DIFF）——按仓库既有风格书写，无新增风格违规 |

## 2. cargo tf 红项定责（13 项，base 逐项复证）

全量收口：`cargo tf --no-fail-fast` = **5832 项中 5819 pass**（35 slow XL
系正常档位内）。13 红分两类：

**A. base 预存红（10 项）**——base worktree（`025fb192c`，零本计划改动）
隔离复跑同项皆红：
`musk_vm_track_p053_1_widget_computed::{passthrough_survives_reeval,
store_arg_helper_chain}`、`p053_4_merged_api_warning`、
`p053_6_widget_obj_arg_field_read_in_helper`、
`p054_t1::icon_component_child_renders_in_button_content_subtree`、
`p054_t4::icon_component_class_prop_carries_ml_auto_and_tint`、
`plan606_029_photo_gallery_thumbnails`、
`desktop_protocol::projector_counter_layout_and_hits`、
`ui_gen::api::test_plan358_d1_for_style_if_msg_on_stress`、
`ui_gen::vue::test_a2vue_desktop_surface_asset`。
本机预存红族（AGENTS cargo t "7 预存红"同源）；与 705 diff 无因果
（codegen 仅追加元数据发布，零发射面变化）。

**B. oracle 冷构建争用偶发（3 项，非回归）**：
`dep_parity_018_dep_fields`、`ffi_dual_018_dep_fields`、
`ffi_dual_019_dep_layout_invariants`——tf 进程内同族 dep 测试并行触发
嵌套 cargo 冷构建的锁争用（首个 tf 跑中 018 曾滞 40min 同根）；base
与本树**隔离复跑全绿**（本树 3/3 PASS，base 3/3 PASS）。资源归因同
资源报告 §1（test 窗口 × 冷 oracle 构建），非断言失败。

## 3. cargo th 16 红定责

- 15 × `back_proxy_tests::http_e2e_back_proxy_*`：socket bind
  `Os { code: 10013, PermissionDenied }`（Windows 端口排除段，环境级）。
  base 复证同样失败（T-04 复审记录；AGENTS th 行"≥3 环境相关红"同源族）。
- 1 × `plan326::e2e_concurrent_sse`：j 并行下端口压测敏感，隔离复跑绿
  （两次复证）。

## 4. 验收标准逐条证据

| AC | 证据 |
|---|---|
| AC-01 等待交还 owner / 多请求并存 / ~T 最终值 / 反例 | `plan705_e2e_upstream_gate_health_two_parked`（gate park 期间 health 200）、`plan705_e2e_async_return_final_value_and_int_counterexample`（~int 150 / int 240 反例）、`plan705_engine_closure_segment_parks_and_resumes`（closure 段）、`plan705_e2e_chained_await_and_error_recovery`（链式+失败恢复）、`plan705_gate_default_callgraph_no_busy_wait`（默认调用图零忙等）；决策报告 §2 入口矩阵 |
| AC-02 单次终结 / 迟到不复活 / 不丢唤醒 | `plan705_spike_cancel_and_late_completion_single_finalization`（协议族）、`p027_*` 两项（兼容面）、`plan705_spike_notify_owner_loop_parks_resumes_without_polling`（enable→check→await 零丢唤醒）、风暴探针逐轮基线 |
| AC-03 无每请求线程 / 限额生效 | `plan705_client_thread_count_stable_under_load`（线程数差 ≤2）、`plan705_client_queue_full_terminal_error_no_thread_fallback`、`plan705_client_body_budget_enforced`、`plan705_client_total_deadline_terminal_error`；资源报告 §2 限额表 |
| AC-04 生命期有界 / 失效不执行 / 取消回收 / 半关闭 | `plan705_e2e_deadline_cancels_parked_and_reclaims`、`plan705_e2e_invalid_scope_skips_handler_dispatch`（零 handler 任务）、`plan705_e2e_shutdown_cancels_parked_side_effect_kept`（副作用保留=非回滚契约）、`plan705_e2e_half_close_still_responds`、`plan705_e2e_cancel_storm_reclaims_deterministically` |
| AC-05 API/SSE/UI 无回退 + 非事务语义 | plan326 79/79（SSE/015/017/023 parity/关闭）、plan702 7/7（engine/UI 段）、client 矩阵 63/63、SD-01..04 源码对证（§6 规范增量 → 沉淀归 review/merge） |
| AC-06 门禁/健康扫描/独立复审证据 | §1 门禁表 + §2/§3 定责（base 复证）+ 三份报告（decision/parity/resource-lifecycle）+ 本报告 |

## 5. 延期 / workaround 扫描

- **无隐瞒 fallback**：默认 HTTP 调用图的同步忙等恰 1 处且为 legacy 串行
  server（决策报告 §2 明示保留，gate 锁定）；引擎忙等臂保留给 legacy
  调用图（702 同款边界）。
- **超范围顺手修复（在案）**：`intercept_error` 跨帧展开——既有引擎缺口
  （深帧 try 经 park/resume 后 catch 不生效；plan702 单帧面未覆盖，705
  服务端面暴露后修复；`plan705_engine_deep_frame_try_recovery` 回归在
  册，base 无此行为）。
- **明示延期**（§非目标对应）：CPU 时间片/抢占、全量 task actor、外部
  SSE/HTTPStream 异步等待、a2r 客户端、TLS/HTTP2/WS——支持矩阵（parity
  报告 §3）逐项标出剩余能力；`__axum:` 全链路 fixture 依赖 musk sibling
  （plan442 ignore 形态），705 以合成 fn-ref closure 覆盖同一段入口。
- **已知边界**：`__axum:` 路由的 `~T` 元数据门依赖 exports 反查成功；
  反查失败按同步处理（保守方向，记录于代码注释）。
- **环境事件（在案）**：并行 plan-706 会话（WSL git）prune 本 worktree
  元数据一次（T-02 复审记录；已重建，后续提交前重验链接）。

## 6. Spec delta 就绪度（SD-01..04 → review/merge 沉淀）

- SD-01（新增 `http-handler-async-lifecycle` spec）：协议形状已实现并有
  探针证据；canonical spec 文件由 review/merge 阶段按 §5 规范增量表
  沉淀（本报告 §4 即源码对证索引）。
- SD-02（async-http-result-lifecycle 修订）：单次终结/取消不复活/有界
  client 已落地；"JSON 队满可 spawn"旧行为移除（终结性错误）。
- SD-03（http-server §8.1 修订）：park/resume、总上限、失效跳过、503/
  关闭/SSE 边界已落地；CPU 非目标明示。
- SD-04（networking-stdlib 阶段标注）：C1 已验证；actor/mailbox、外部
  流、a2r 仍独立未迁（支持矩阵明示）。
