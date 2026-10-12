# PLAN-727 T-07 报告：三方 parity（VM / a2r / 原生 Rust）

- 阶段：work（T-07）
- 代码基线：worktree `D:/autostack/.wt/lang-727/auto-lang`，分支 `plan-727-dev`
  （T-01..T-06 提交链之上）。
- 性质：文件传输新面/legacy/builder 在 VM、a2r 转译产物、原生 Rust 三方的
  实际执行对拍证据。**不是状态字符串对拍**——下载对拍目标文件字节、上传
  对拍 wire（方法/头/multipart 字段/filename/体）、失败对拍原目标字节与
  staging 清理。

## 1. 执行矩阵与证据

| # | 矩阵项 | VM 腿 | a2r 腿 | 原生 Rust 腿 | 结果 |
|---|---|---|---|---|---|
| 1 | 下载成功（收据 + 目标替换 + 无 staging 遗留） | `plan727_vm_transfer_download_wait_success`（Auto 源 run_with_capture，stdout 收据 + 文件字节） | `http_e2e_plan727_sync_transfer_matrix`（转译→cargo build/run，`receipt: {"kind":"success"` + 文件字节） | `plan727_download_full_commit_replaces_target`（内核测试 200KiB 多块 + 目录清点） | 三方全绿 |
| 2 | 404/失败保留原目标 | 同上收据面 | `http_e2e_plan727_missing_preserves_target`（404 + `PRECIOUS` 字节） | `plan727_download_http_failure_preserves_target`（404 保留 + staging 清理） | 三方全绿 |
| 3 | 坏 options 终结（请求不发出） | `plan727_vm_transfer_bad_options_terminal`（死端口 + Failed(options)） | golden 形态（options 串直传） | `plan727_bad_options_terminal_no_request`（status null + 终态单次交付） | 三方全绿 |
| 4 | 取消词汇（cancel/wait/error） | `plan727_vm_transfer_cancel_observable` | 002 golden（async cancel + wait） | `plan727_cancel_mid_transfer_preserves_target_and_cleans_staging` + commit-gate 迟到取消不回滚 | 三方全绿 |
| 5 | 上传 multipart wire（fields/filename/体） | legacy `test_multipart_file_functional`（plan349，流式迁移后仍绿） | `http_e2e_plan727_builder_multipart_wire`（binary 1000B 全内容 + `name="note"` + `filename`）+ sync 矩阵 wire | `plan727_upload_multipart_wire_and_text_fields` + raw `Content-Length` 精确长度 | 三方全绿 |
| 6 | 缺失文件/读失败不假成功 | — | — | `plan727_upload_missing_file_fails_not_success`、`plan727_upload_dir_rejected`（请求未发出）、builder `erroring_part` 流（测试内联） | 原生全绿 |
| 7 | 续传/If-Range/416/200 重启 | legacy `test_http_download_resume`（plan349 严格续传迁移后绿） | legacy 发射探针（`plan727_probe_legacy_file_helper_emission_unchanged`） | `plan727_resume_206_appends_via_staging`、`plan727_resume_200_full_restart_never_appends`、`it_if_range_etag_match_appends_mismatch_restarts`、`it_416_preserves_target` | 三方全绿 |
| 8 | >10MiB 流式（12MiB，FNV-1a hash 对拍） | — | — | `it_large_download_12mib_streaming`（32KiB 滴发 + hash 一致） | 原生绿 |
| 9 | 未知 total（chunked）进度/收据 | — | — | `it_chunked_unknown_total_progress_null`（`"total":null`/`"percent":null`） | 原生绿 |
| 10 | 零字节/offset=0 | — | — | `plan727_zero_byte_download_succeeds`、`plan727_zero_byte_resume_requires_exact_empty_target` | 原生绿 |
| 11 | 超预算（max_bytes） | — | — | `plan727_budget_exceeded_preserves_target`（原目标 + staging 清理） | 原生绿 |
| 12 | 同目标冲突仲裁 | — | — | `plan727_same_target_conflict`（conflict 终态） | 原生绿 |
| 13 | 进度迭代器迁移（producer） | `plan727_progress_relay_data_then_done`（relay 泵：进度/收据 JSON + Done + staging 落盘；消费端 for-in 为既有机制） | — | — | VM 腿绿 |

## 2. 命令与退出码（本报告数据源）

```text
cargo test -p a2r-std --lib http::                      # 46/46 绿
cargo test -p a2r-std --test http_transfer -- --test-threads=1   # 6/6 绿（串行矩阵）
cargo test -p a2r-std                                    # lib 65 + 集成全绿，0 失败
cargo test -p auto-lang --lib plan727                    # 8/8 绿（golden 2 + 探针 2 + VM 3 + relay 1）
cargo test -p auto-lang --lib --features test-http-e2e plan727 -- --test-threads=1  # 8/8 绿（含 4 e2e 编译运行）
cargo test -p auto-lang --lib plan349                    # 16/16 绿（legacy 形状回归）
```

## 3. legacy 发射/形状差异（冻结，不悄悄重定）

- a2r `http.download/upload/download_resume -> u32`：发射面逐字节不变
  （probe 断言含 `!contains("transfer_download")`）；执行迁核心后语义纠正：
  文件错误 → 0（旧代码 copy 失败仍返回 HTTP 状态 = 假成功，已修）。
- VM `http.download/download_resume -> bool`：bool 形状保留；执行从
  每调用线程+join 改为 yield/park（对 Auto 程序可见形态不变）。
- VM `http.upload -> Response`：Response 句柄形状保留；无响应失败 = 内部
  错误句柄（旧伪 500 语义延续）。
- 旧 a2r golden `17_rust_std/011`（u32→i64 int 样本）保留；发射探针另证
  u64 offset 字面量（4294967296）不截断。VM legacy resume 的 i32 offset
  ABI 事实保留（64 位路径在新面 options）。

## 4. 已知边界（与决策报告一致）

- 无 validator 的续传不承诺远端版本一致（If-Range 严格对齐例外见报告 §3）。
- 上传重试默认 0（非幂等 POST 不自动重放）；显式 retries 的重试复核
  len/mtime，不符 → source_changed。
- multipart MIME 为扩展名最小推断表（对齐 Part::file 常见行为），未列
  扩展 = application/octet-stream。
- 服务端文件上传路由/Range 响应、API/IPC 契约统一、CPU 抢占、断电持久性
  不在本计划交付集。
