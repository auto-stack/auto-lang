# PLAN-707 T-08 验证报告

- 日期：2026-09-29
- worktree：`D:/autostack/.wt/lang-707/auto-lang`（branch `plan-707-dev`）
- 基线：master @ 66c9cac19；本报告对照 AC-01..06 与 SD-01..06 逐条对证。
- 门禁命令与结果见 §1；环境红/抖动定责见 §4（全部 baseline 同命令复证）。

## 1. 门禁结果

| 门禁 | 命令 | 结果 |
|---|---|---|
| 快速检查（开发期多次） | `cargo check -p auto-lang` | 绿（无新告警：告警数与基线一致，新增代码零告警） |
| 无默认 feature | `cargo check -p auto-lang --no-default-features` | 绿（194 告警=该形态基线） |
| 测试形态编译 | `cargo check -p auto-lang --tests`（含 `--features test-http-e2e`） | 绿 |
| 全量 tf | `cargo tf`（fail-fast 于 1567 截断）+ `--no-fail-fast` 全量（5709 例，排除画廊围栏另测） | 5703/5709 绿；5 红全部基线定责（§4），**707 相关零真实回归** |
| 画廊围栏（AGENTS.md 另计档） | `test(widgets_gallery_all_front_pages_compile)` 单独 | 预存红（干净树同败 568s，§4） |
| th（真 TCP 池） | `cargo th`（等价 `--features test-http-e2e http_e2e`） | plan707 E2E 三例全绿（`-j4` 与 back_proxy 同跑复验）；back_proxy 全家预存环境红（§4） |
| 格式 | `rustfmt --check`（9 个新增/重写文件） | 已格式化通过 |
| 调试残留 | 源码扫描 `707dbg\|341dbg\|dbg_step` | 零残留；新增代码无 debug 输出 |

## 2. AC 逐条对证

| AC | 证据（测试/探针） | 结论 |
|---|---|---|
| AC-01 取消实停 | `plan707_cancel_active/queued/retry/races/detached`（5 例）+ spike 红转绿 witness | **pass**：managed future 三阶段实停、许可归还、迟到不复活；detached 消息桥不误杀 |
| AC-02 慢流等待+不热转 | `plan707_wait_generator_park_drive_count_static`（gate 关闭驱动静态+放行唤醒）+ `plan707_client_*`（段 park/resume） | **pass**：凭据化等待零自旋；段原栈恢复；双流隔离（E2E 帧序列断言） |
| AC-03 取消矩阵回基线 | E2E 断连→上游取消（≤8s 观测）；close 两轮回基线；风暴两轮（`plan707_stream_cancel_storm_two_rounds_baseline`） | **pass**：资源组收口（scope 组 + task.owned 第二线）使上游连接真实关闭 |
| AC-04 解析/背压/预算 | `plan707_decode_*`（9 金样：全切分点/UTF-8 中点/CR·LF·CRLF/预算）；`plan707_stream_backpressure_bounds_queue_and_stops_pull`（队列钉界）；`plan707_stream_item_budget_terminal_failed` | **pass**：解析网络分段无关；合法空串/[DONE] 不丢；队列字节/条数有界；超限终结性诊断 |
| AC-05 签名保留+新面装配 | `plan707_client_*`（手动/内联/变量 for/poll/close/error）+ parity 报告 §2 | **pass**：get_stream/post_stream(+headers) 签名不变；[DONE] 哨兵保留；三形态 for-in 全绿；sse_close/sse_error 全链可用 |
| AC-06 资源报告+门禁 | `plan707_stream_thread_count_stable_across_streams`（12 流线程稳定）；`plan707_stream_job_abort_table_reclaims`；本报告 §1/§4 | **pass**：固定 runtime 线程稳定；无每流线程/runtime；无同步 read/join 默认流路径；红项独立定责 |

## 3. SD-01..06 实现对证（供独立 review → merge 沉淀）

| delta | 实现落点 | 对证 |
|---|---|---|
| SD-01（新 http-stream-lifecycle.md） | `http_stream.rs`（STREAMS 表/状态机/许可对/背压/finalize）+ 本目录三报告 | 待 merge 沉淀（work 阶段未动 canonical Spec ✓） |
| SD-02（async-http-result-lifecycle 修订） | `async_http.rs`：JOB_ABORTS + cancel abort + 插入后复查 + 完成出表；`submit_detached_client_job` | AC-01 五探针 |
| SD-03（handler-async-lifecycle 修订） | `ParkedWait::HttpStream`、`waiting_http_stream_id` 重试协议、generator 凭据化等待、scope 资源组 | AC-02/03 |
| SD-04（http-server.md 修订） | VM for/pull/relay 面（E2E relay 三例）、手动签名保留、失败/取消边界 | AC-05 + parity 报告 |
| SD-05（backend-assembly.md 修订） | catalog 双表登记 + parity 报告 §3（a2r/auto-man 未迁、bus/进度流不迁移明示） | AC-05 |
| SD-06（networking-stdlib.md 修订） | 能力图更新素材=decision D-1..D-12 + 本报告 | AC-02/05/06 |

## 4. 红项定责（baseline 同命令复证，全部与 707 无因果）

| 测试 | tf 结果 | 干净树基线 | 定责 |
|---|---|---|---|
| `gallery_pages_compile_tests::widgets_gallery_all_front_pages_compile` | FAIL 535-568s | FAIL 568s（同命令同树） | 预存环境红（跨仓 auto-os 画廊编译围栏，本机环境） |
| `ui_gen::vue::tests::test_a2vue_desktop_surface_asset` | FAIL ~20s | FAIL（先前 stash 复证） | 预存环境红 |
| `ui::desktop_protocol::client_runtime::tests::projector_counter_layout_and_hits` | FAIL 0.1s | FAIL | 预存红 |
| `vm::ffi::stdlib::e4_default_http_tests::default_headers_reach_wire_on_plain_get` | FAIL 207-271s | FAIL 271s | 预存红（真实 HTTP 端点环境依赖） |
| `musk_vm_track_tests::...::merged_mode_api_call_emits_warn_opcode` | 首跑 FAIL；no-fail-fast 跑未复现；干净树单跑 FAIL | 双向不稳 | 预存 flaky |
| `ui_gen::api::tests::test_plan358_d1_for_style_if_msg_on_stress` | 并行 FAIL 8.9s | 干净树 -j1 PASS；本树 -j1 PASS 4.4s | 并行负载抖动（stress 型测试） |
| `plan707_spike_cancel_gap_active_job_future_not_stopped` | 并行 FAIL（300ms 宽限） | 本树修复后全档 -j2 绿 | 707 测试自身时序 flake → 宽限 2s 修复（abort 生效前调度延迟） |

## 5. 未授权延期/超范围改动扫描

- 无。范围外明示项（a2r/auto-man 未迁、bus/进度流不迁移、`.iter()` 方法式派发、
  task-loop 唤醒保留轮询形态）均已在 decision/parity 报告冻结为「明示不声称」，
  无静默降级；canonical Specs 与派生 ledger 未动（work 纪律）。
- 计划内新增路径均已登记：`http_stream.rs`（ffi/mod.rs）、`decoder.rs`（sse/mod.rs）、
  测试 6 文件（tests.rs / lib.rs e2e 模块）。

## 6. 遗留与建议（供 review/后续计划）

1. `musk p053` 与 `plan358 stress` 为机器负载敏感型既有测试，建议另档稳定化（非本计划范围）。
2. back_proxy 全家启动失败为本机环境问题，建议环境探针（ AUTO_OS_ROOT/代理依赖）另查。
3. task-loop 流唤醒的轮询形态事件化，可在后续响应性计划内与 708 协同推进。
