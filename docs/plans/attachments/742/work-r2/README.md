# PLAN-742 r1 修复（work 第二轮）证据包

绑定：worktree `D:/autostack/.wt/lang-742/auto-lang`，分支 `plan-742-dev`，
base = 审核入口 HEAD `c151935f1`（reviewed_commit 一致）；修复提交见计划 §9。
auto-down 兄弟（组内路径依赖）：detached @ c39ac7f9（与审核记录的
dependency revision 相同）。

## 内容

- `replay-generated-results.json`：审核 replay.ps1 -Build 在修复后工作树
  重放的四用例结果——compile_exit 全 0，actual 与 REVIEW.md 预期字面值
  逐字节一致（control/"ordinary message"、list_literal/"List<int>.new([])"、
  module_literal/"repository.filename"、brace_literal/"} return secret"）；
  D7/D9 运行时控制同轮 PASS（replay 尾部输出）。
- `notes-v1-battery-summary.txt`：T-3 联动重验——工作树 CLI 再生成
  auto-notes 后端（generation.json 2026-10-13T15:05）+ cargo build 通过 +
  `tests/probe/run_v1_battery.sh` 全绿（fails=0，36 断言 ✓，含 S10 崩溃
  恢复/S15 修复/S19 混态；NOTES_TEST_MODE=1 + 隔离 NOTES_DATA_DIR，
  NOTES_SEED 未设）。

## 门禁

- `cargo test -p auto-man --lib api_gen`：46 passed / 1 ignored（三连跑稳定；
  新增 test_handler_body_string_survival_and_compile_oracle = 真生成→rustc
  实编→运行→返回值对拍 oracle）。
- `cargo tt --no-fail-fast`：5649/5649 passed（首跑 fail-fast 红为 ffi_dual
  满池负载 flake——批量回执已两轮在案建议登记 KNOWN-DEBT）。
- rustfmt --check 触面两文件干净。
- dead-code 告警（F-742-R1-04）三项清零：NL / wrap_inline_return 旧包装 /
  borrow_string_db_call_args_old 已删除。
- auto-man 全 lib：master 基线同款 3 预存红（merged_api_client_crud_fallback /
  test_shell_pack_lib_freshness / plan593_index_css），本轮零新增红；
  并行态偶发 receipt 竞争根因=generation_consumer_identity 未持
  a2r_env_lock 即 set_var（已补锁，bp-root-env-lock 先例形态）。
