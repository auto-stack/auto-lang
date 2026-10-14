# PLAN-742 r2 修复（work 第三轮）证据包

绑定：worktree `D:/autostack/.wt/lang-742/auto-lang`，分支 `plan-742-dev`，
base = r2 审核面 `6d93cfdb7`；修复提交 `8c9b2d3d9`。
auto CLI（提交态重建）SHA256
`5e5b146ca7f25914411844362defb40a4cf19d002f79173751a4d2f8ae5a8d20`。
auto-down 组内兄弟 detached @ c39ac7f9（与 r1/r2 记录一致）。

## 内容

- `replay-r2-generated-results.json`：审核 r2 replay.ps1 -Build + 
  execute-generated.ps1 于提交态 `8c9b2d3d9` 重放——13 case 全部
  compile_exit=0；对照 REVIEW.md 期望逐项一致：五个字面量（control/
  list/module/brace/escape 含 CJK+转义引号复合串、borrow-marker
  `"crate::repository::take(value)"`）值保持；module_control/scope_shadow
  模块可达（"ok"）；void_control/void_branch 编译运行通过（StatusCode）；
  option_variable `1:ok / -1:not_found`（Some/None 双臂）；file_control/
  file_module `"readme.txt"`。
- `notes-v1-battery-summary.txt`：T-3 联动（提交态 CLI 再生成后端
  generation.json 2026-10-14T01:40:08 → cargo build → run_v1_battery.sh
  fails=0）。
- `s10-driven-transcript.log`：**实驱 S10**——setup 隔离目录 → 建
  s10-final（request_id 契约）→ taskkill /F（kill -9）→ 确认下线 →
  重启（NOTES_TEST_MODE=1）→ setup 同目录 → GET 存活（COUNT 1，HAS True，
  STATE active）。

## 后处理链盘点（R2-01 诊断的完整清单）

| 阶段 | 消费者 | 字面量保护 |
|---|---|---|
| 体发射（rust.rs） | split_glued_statements / lower_list_constructs | r1 已掩码化 |
| 体行内（api_gen） | rewrite_inline_runtime_names | 自带字符串状态（r1 前已有） |
| 整文件（api.rs） | qualify_a2r_std | **本轮掩码化**（replace_in_code_regions） |
| 整文件（api.rs/db.rs） | borrow_string_module_call_args | **本轮掩码化+调用形态** |
| companion/db.rs | strip_collection_new | **本轮掩码化** |
| db.rs 遗留 | MutexGuard 正则 / append_tostring_for_str_fields / append_clone_for_borrowed_fields | **同类暴露登记为债务候选**（Plan 399 遗产，db.at 业务串可匹配这些正则形态；非 742 引入面） |

## 门禁

- `cargo test -p auto-man --lib api_gen`：**47 passed / 1 ignored**（三连跑稳定；
  新增 test_handler_body_r2_matrix_compile_oracle——11 值编译+运行矩阵）。
- `cargo tt --no-fail-fast`：**5649/5649**；中途满池轮红 = 预存负载 flake 家族
  （ffi_dual_018 / plan730_commit_target_matrix / plan484_024），逐名定向复跑
  全绿。
- rustfmt --check 触面干净；742 新增 dead-code 零。

## 边界（如实记录）

- Vue 面：pnpm install 两轮网络失败（repo.huaweicloud.com 镜像
  UND_ERR_DESTROYED）——gen/front/vue 前端未起。
- VM 面：Notes VM autotest 需有头桌面 MCP 环境（localhost:9247），
  本会话无头不可运行。
- Upload 入口的限定化为源码级接线（与 File 同构共享 helper），无独立
  Upload companion 实编反例（审核亦仅要求源码检查+明示待查）。
- `.take` 是 .at 语言级移动语义糖（`x.take`→`x`，rust.rs:3363）——点调用
  名为 take 的模块函数按语言设计不可达，oracle 用 measure 名。
