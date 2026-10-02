# PLAN-727 验证报告：AC/SD 逐项实证绑定

- 阶段：work（T-08；execution_done 交接）
- 代码 revision：worktree `D:/autostack/.wt/lang-727/auto-lang`，分支
  `plan-727-dev`，HEAD `8ba5199d2` + 工作区收尾（警告差分归零/探针清理）。
  基线 `b34852532`（master @ 2026-10-02）。提交链：`b1638a5e1`（T-01..04 核心）、
  `38454a3bf`（T-05 VM 桥）、`0a5cec371`（T-06 a2r 发射）、`8ba5199d2`
  （T-07 矩阵）。
- 依赖：auto-down 兄弟 worktree（detached @ `895f8d0`，只读解析用）。
- 725/726 干扰核对：两计划均在各自 worktree 未合入 master，与本触面无交叠。

## 1. AC 逐项绑定

| AC | 通过条件 | 实证（测试/探针 → 代码 revision） | 状态 |
|---|---|---|---|
| AC-01 | 新公共 FileTransfer、multipart 声明/注册/类型/两 facade 映射齐备；新面 VM/a2r 结果一致；legacy 差异不悄悄重定 | http.at/http.vm.at 声明 + catalog 9930-9935 双表（`38454a3bf`）；plan727 golden sync/async（`0a5cec371`）；`http_e2e_plan727_sync_transfer_matrix`（VM/a2r 同源 .at 双腿执行，`8ba5199d2`）；`plan727_probe_legacy_file_helper_emission_unchanged`（`!contains transfer_download`）；`test_native_ids_no_collision` 绿 | ✅ |
| AC-02 | 无整文件 Vec/阻塞网络/每任务线程/runtime；async 等待让出；应用内存守独立预算；排队取消/总期限覆盖准入 | 核心管道 = 64KiB 块 × 有界(2)（`crates/a2r-std/src/http/transfer.rs`，`b1638a5e1`）；12MiB 流式 hash 对拍 `it_large_download_12mib_streaming`；排队取消/期限 `plan727_execute_queued_*`（T-02 闭合，gate 不开复现）；`it_queue_saturation_rejects_then_recovers`（配额隔离 + 回基线）；资源公式见 727-transfer-resources §1 | ✅ |
| AC-03 | 仅完整成功提交；HTTP/读写/flush/替换/超限/取消失败保留原目标；同目标冲突有规则；Windows 替换不先删旧文件 | `plan727_download_http_failure_preserves_target`（404）、`plan727_write_failure_injection_preserves_target`（注入）、`plan727_budget_exceeded_preserves_target`（超限）、`plan727_cancel_mid_transfer_*`（取消）——均校验原目标字节 + staging 清零；`plan727_same_target_conflict`（conflict 终态）；Windows rename=REPLACE_EXISTING 语义（仓库既有记载 state_file.rs:10 + t01-probe 持读句柄替换实测，`b1638a5e1` 报告 §6） | ✅ |
| AC-04 | Range/本地长度/实际长度严格验证；200 完整重启；416/坏范围保旧；validator/无 validator 承诺清楚；64 位位置正确 | `plan727_resume_206_appends_via_staging`（Content-Range 起点/长度/总长严格）、`plan727_resume_200_full_restart_never_appends`、`it_if_range_etag_match_appends_mismatch_restarts`（If-Range 失配→200 重启）、`it_416_preserves_target`、`plan727_resume_offset_mismatch_preserves_target`；弱 ETag 拒绝 + validator 单字段（options 严格解析测试）；64 位经 options u64 + `plan727_probe_legacy_offset_u64_emission`（2^32 字面量） | ✅ |
| AC-05 | cancel/Drop/scope/shutdown 停止网络生产与后续文件修改；在途 FS 收口后许可/句柄回基线；进度有界、终态不丢单次、慢消费不挂 | `plan727_cancel_mid_transfer_*`（网络停 + 收口 + 清理）；`plan727_commit_gate_late_cancel_does_not_rollback`（提交点竞态冻结）；scope 有类型组 `transfer_resources` + `finalize_scope` 组收口（`38454a3bf`）；终态单次交付 `plan727_bad_options_terminal_no_request` 进度面；`transfer_wait_async` future Drop = 结构化取消（guard）；`plan727_progress_relay_data_then_done`（进度合并/终态不丢） | ✅ |
| AC-06 | raw/multipart/legacy/builder 与三方实际执行正确；缺失文件/重试读失败不省略不假成功；门禁无新增红，SD 有实证 | raw `plan727_upload_raw_octet_stream`（Content-Length 精确）；multipart wire 三方（parity 报告 §1 #5）；缺文件/目录拒绝（请求不发出）+ builder `erroring_part` 流；重试重开复核（`it_upload_retry_replay_bounded_termination` + SourceChanged 路径）；门禁差分见 §2；parity/资源两报告（`8ba5199d2`） | ✅ |

## 2. 触面门禁记录（worktree 串行；红逐名与 master 同档对照）

| 门禁 | 结果 | 红差分（逐名 vs master 同命令） |
|---|---|---|
| `cargo test -p a2r-std` | ✅ 全绿（lib 65 + tests/http_client 7 + tests/http_transfer 6 串行） | 无红 |
| 裸 `cargo t`（--no-fail-fast） | 红 ∈ master 预存集 | 同名同族：musk_vm_track×6、plan707_wait（本 worktree 单跑绿=负载抖动）、plan606_gallery_029、projector、desktop_bus/surface、ash_leak、schema_drift×2、kitchen_sink——master 同命令逐名复现（plan502/plan707_cancel 双侧单跑均绿，为漂移成员） |
| `cargo tv` | ✅ 162/162 | 无红 |
| `cargo tt`（--no-fail-fast） | 12 红 = master 逐名相同（musk×6/plan707_wait/gallery/desktop×2/projector/ash） | 零新增 |
| `cargo th`（--no-fail-fast） | 64/66；**http_e2e_plan727_* 4/4 全绿** | 2 红：back_proxy_real_routes_corpora_data_face（master 同名红）；plan707_relay_frames_timed（已知时序 flake=4f123a50e 族，master 单跑亦红、worktree 单跑绿） |
| `cargo tb`（补跑：触 stdlib .at） | 13 红 vs master 11+1timeout | 新名 book_ch03_08/06_05/06_08/09_02 → **P727-F1**（见 §3，基线提交全新 worktree 同红，非本计划引入）；plan502_m3 双侧单跑均绿 |
| 警告差分 | a2r-std 0；auto-lang 382 = master 382 | 零新增（TransferEntry 保活字段以 `#[allow(dead_code)]`+注解显式化） |
| aavm 触发条件 | 零触发（`auto/lib/*.at`、`test/vm/aavm2/**`、`parity/**`、aavm2 测试基建均未触） | 不跑 taa（按 §AAVM 映射） |
| ui_gen/book 生成器 | 未触 `ui_gen/**`；tb 补跑已覆盖 book 面 | 不跑 tu |

## 3. 发现与边界

- **P727-F1（预存，非本计划引入）**：`book_listing_tests::book_ch03_08/ch06_05/ch06_08/ch09_02` 在**基线提交 `b34852532` 的全新 worktree 同样失败**（已用一次性基线 worktree 复现并清理）；根因是 book 仓 `rust/listings` 的 is 臂仍用 `0 =>` 旧语法，现行解析器只接受 `0 ->`（语料 `test/vm/04_control_flow/006_is_stmt` 即新语法）。主检出单跑通过属其 target 增量构建的本地假象（`cargo clean -p auto-lang` 后复测仍 PASS，未再深究其增量状态；判定依据以基线全新 worktree 复现为准）。修复归属 book 仓（另案/L0），本计划不动他仓。
- plan707 client/stream 族（t 档 11 例）在 master 主检出同过滤即红（注册表跨测试污染族），T-05 改动前基线同红——与本计划无因果。
- 队列语义在 T-07 实测中纠偏：在途总量上限 = queue_capacity(16)（队列许可提交即取），非"4 活跃 + 16 排队"；决策报告 §7 与代码一致，测试已按实测语义固化。
- staging 清理失败保留可追踪路径与错误（不宣称零遗留）；断电持久性不承诺（决策 §9）。

## 4. SD 沉淀提案状态

SD-01..SD-06 的实证材料齐备（决策/parity/资源三报告 + golden + 门禁日志），
canonical Specs 的落盘按流程留给 `/auto-plan:review` 验证后由 merge 执行：
SD-01 新建 `docs/specs/stdlib/design/http-file-transfer.md`（契约 =
727-transfer-decision.md §1-§7 冻结面）；SD-02..SD-06 修 a2r-std/stdlib/
lowering/stream-lifecycle/networking 各 Spec 的过时表述（"文件 helper 走
ureq"→共享传输核心；"a2r 未迁移"→724 现状 + 727 文件传输）。

## 5. 兼容与限制声明（复审关注面）

- legacy VM resume 的 i32 offset ABI 保留（新面为 64 位路径）。
- 新面收据/进度 JSON 是跨后台契约，字段集冻结于决策报告 §2；后续加字段
  须走 Spec delta。
- 上传 MIME 为最小扩展名推断表；未列扩展 = octet-stream（与 legacy
  Part::file 推断的常见集一致，exotic 扩展存在 wire 差异，已在 parity
  报告 §4 登记）。
- 丢弃 `transfer_wait_async` future = 取消传输（结构化取消，与内核
  execute 同语义）；progress 观察永不取消。
