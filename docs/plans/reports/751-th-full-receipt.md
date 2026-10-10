# PLAN-751 完整串行 th 验收收据（2026-10-10）

- 计划：PLAN-751 T-05 / AC-05（承接 PLAN-738 R9 移交的 HTTP 全档尾项：45/102 → 补齐 102/102）。
- 执行环境：worktree `D:/autostack/.wt/lang-751/auto-lang`（分支 plan-751-dev）。
  **被测 auto-lang 代码 = master `0884add2a` 的 auto-lang 面**——PLAN-751 的实现提交
  （`f16de5a19` auto-man、`fe3d78937` specs）不触及 auto-lang，且 th 档 `-p auto-lang`
  不编译 auto-man（auto-lang 不依赖 auto-man），th 结果与 PLAN-751 diff 零因果。
- 命令：`cargo th --jobs 1 --no-fail-fast`（= `nextest run -p auto-lang --lib
  --features test-http-e2e http_e2e`，串行单实例——http_e2e 族固定端口，
  `.cargo/config.toml` th 注释要求）。
- 结果：**102 selected / 102 run**（R9 未跑的 57 项全部补跑；R9 已跑 45 项本轮
  全部重跑，不沿用旧结论），**86 passed / 2 failed / 14 timed out**，
  总时长 1962.969s（≈32.7 分钟）。nextest slow-timeout（60s 警告 ×2 → 120s
  TERMINATING）收尸，无无限挂死。

## 逐名分诊（16 项未过）

| # | 测试 | 本轮 | 分诊 |
|---|---|---|---|
| 1 | `back_proxy_tests::http_e2e_back_proxy_real_routes_corpora_data_face` | FAIL 0.138s | **在册基线红**（R9 45 项中的 2 红之一，同名同形；400/200 corpora 数据面基线） |
| 2 | `plan730_http_upload_tests::http_e2e::http_e2e_plan730_client730_server_interop` | FAIL 109.988s | **确定性失败**（R9 同形：109.655s，固定端口 18980 upload 传输失败；两轮耗时几乎一致=内部 ~110s 期限后断言失败，非负载） |
| 3-15 | `plan730_http_upload_tests::http_e2e::http_e2e_plan730_{legacy_bind_failure_cleans, legacy_middleware_reject_no_write, legacy_write_failure_500, vm_auth_before_storage, vm_chunked_upload, vm_disconnect_cleans_staging, vm_multipart_upload_commit_download, vm_plain_endpoint_untouched, vm_quota_and_health_under_load, vm_quoted_boundary_wire, vm_raw_upload_and_business_reject, vm_slow_upload_survives_handler_deadline, vm_wire_caps_and_malformed}` | TIMEOUT 120s ×13 | **plan730 上传 e2e 整族确定性挂死**（见下「挂死形态」）；R9 停跑点 `legacy_bind_failure_cleans`（>60s 停）即本族首个，本轮证明族级而非单项 |
| 16 | `vm::ffi::http_server::plan326_tests::http_e2e::e2e_a_redirect_302_with_location` | TIMEOUT 120.033s | **确定性挂死**（非 flake：隔离单跑复现 TIMEOUT 120.025s）；同文件 39 个兄弟用例全绿（0.6s 级），独此一项挂——与 plan730 族共享 `vm::ffi::http_server` 的 Response 族返回路径（`http.response_redirect` vs `http.upload_*`） |

**挂死形态**（隔离复现，单测单进程 `--jobs 1`）：测试起线程 `crate::run(&code)` 跑
VM HTTP 服务，stderr 仅打印 `warning: 'nil' is deprecated`（编译期已过）后静默；
测试侧 `TcpStream` 可建连但服务端不响应——`read_response` 的 60s read_timeout
×2 次阻塞读累计 ≈120s 被 nextest TERMINATING。挂死不随机器负载变化（同条件下
plan326 兄弟用例 0.6s 全绿、plan729 12MiB 文件矩阵全绿）。

## 归因边界（本轮不修实现，登记后续）

- **非 PLAN-751 引入**：见上「执行环境」；亦非 R9/R10 修复引入的回归面。
- **红移窗口**：plan734 T-08 收据 th 99/101 绿（`ae9b7a7e2`，2 预存）→ PLAN-738
  系列之后转红。http_server.rs 最后触碰 = PLAN-738 R2（`bade1845c`/树快照
  `dd9ffe35b`，2026-10-09 15:24）；其后 VM/stdlib 面还有 `19973b089`（merge
  修复轮：register_stdlib_ffi 无契约重注册按 merge 撤销语义——**http 族契约
  注册语义**）与 `41b4d4be5`（R9）。R2 收据自述「th 余 back_proxy master 预存
  +次序 flake 复跑绿」与本轮确定性族挂死矛盾——该收据的 th 面可能非全档或
  当时未触发；**精确根因需专项 bisect**（候选锚点：`dd9ffe35b` post-tree、
  `19973b089`）。
- 后续动作：另立修复计划做挂死根因 bisect + 修复 + 本收据 16 项复验；已登记
  KNOWN-DEBT。**本收据不宣称 HTTP 面全绿**（86/102 过）。

## 复跑矩阵

| 验证 | 结果 |
|---|---|
| 全档 102 项串行（本轮主跑） | 86 pass / 2 fail / 14 timeout |
| `http_e2e_plan730_vm_plain_endpoint_untouched` 隔离单跑（--no-capture） | TIMEOUT 120.028s（确定性） |
| `e2e_a_redirect_302_with_location` 隔离单跑 | TIMEOUT 120.025s（确定性） |
| R9 承接 45 项交叉对照 | 43 项两轮一致绿；back_proxy、interop 两轮一致红（同形） |

next：T-06 复审合并后，挂死族修复走新的后续计划（候选：`751-th-full-receipt.md`
本收据即其输入）。
