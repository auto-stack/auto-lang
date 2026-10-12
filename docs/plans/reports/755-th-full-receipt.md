# PLAN-755 完整串行 th 全档对账收据（2026-10-10）

- 计划：PLAN-755 T-05 / AC-05（输入=751 收据 16 项未过名单逐名对账）。
- 执行环境：worktree `D:/autostack/.wt/lang-755/auto-lang`（branch `plan-755-dev`），
  base `d6e4819ba` + `327069006`（T-02 夹具有界化）+ `00e7f712c`（T-01/T-03 契约补齐）；
  组内 auto-down 兄弟 detached@`895f8d0`（依赖只读）。
- 命令：`cargo nextest run -p auto-lang --lib --features test-http-e2e http_e2e
  --no-fail-fast --jobs 1`（= th 别名语义 + no-fail-fast + 串行单实例，
  `.cargo/config.toml` th 注释纪律；与 751 收据同款 `--jobs 1`）。
- 结果：**102 selected / 102 run / 101 passed / 1 failed / 0 timed out，总 186.390s**
  （751 基线：86 passed / 2 failed / 14 timed out，1962.969s——超时死重清零，实跑面 3.1 分钟）。

## 16 项旧未过逐名对账

| 751 收据 # | 测试 | 751 基线 | 本轮 | 分诊 |
|---|---|---|---|---|
| 1 | back_proxy `http_e2e_back_proxy_real_routes_corpora_data_face` | FAIL 0.138s | FAIL 0.142s | **在册基线红维持**（同形：018 chapters `:id` 参数 400≠200 corpora 断言；755 边界=不恶化，未恶化 ✓） |
| 2 | plan730 `http_e2e_plan730_client730_server_interop` | FAIL 109.988s | **PASS** | 服务真实启动后 upload 互操作链通过（旧失败=契约错误被吞后的 ~110s 内部期限断言） |
| 3-15 | plan730 13 项（legacy_bind_failure_cleans / legacy_middleware_reject_no_write / legacy_write_failure_500 / vm_auth_before_storage / vm_chunked_upload / vm_disconnect_cleans_staging / vm_multipart_upload_commit_download / vm_plain_endpoint_untouched / vm_quota_and_health_under_load / vm_quoted_boundary_wire / vm_raw_upload_and_business_reject / vm_slow_upload_survives_handler_deadline / vm_wire_caps_and_malformed） | TIMEOUT 120s ×13 | **全部 PASS**（0.6-38s 实跑；vm_slow_upload_survives_handler_deadline 38.2s 为真慢速上传语义） | timeout 清零：服务真实 bind + wire 断言生效 |
| 16 | plan326 `e2e_a_redirect_302_with_location` | TIMEOUT 120.033s | **PASS 0.6s** | 302 + Location + follow 断言通过 |

**旧 86 绿零回归**（101 = 86 + 15 全绿）。

## 附加观测

- **并行态 flake（知情登记，非 755 缺陷）**：`--no-fail-fast` 并行跑（未按 serial 纪律）时
  `vm_quoted_boundary_wire` 曾 FAIL 0.140s；隔离单跑 PASS 0.629s、串行全档 PASS——为
  并行进程间交互（th 注释 serial only 的既有约束，与 e2e_ports_unique 守卫的历史撞号
  同类）。收据以串行跑为准。
- 契约走查/merge 守卫（`plan755_http_contract_tests` 2 测）在 th 滤串外，随日常档/T-06 验证。

## 结论

- **14 项 timeout 清零、interop 转绿、back_proxy 维持在册基线红不恶化、86 绿零回归**——
  AC-05 四项口径全部满足。
- HTTP e2e 面（除在册 back_proxy 基线红外）全绿；751-R2 遗留的"挂死族"以实证机制
  （契约错误被吞 × 拒绝连接慢速爬行）闭合，见 [755-startup-diagnosis.md](755-startup-diagnosis.md)。
