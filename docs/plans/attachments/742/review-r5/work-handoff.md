# PLAN-742 r4 修复（work 第五轮）证据包

绑定：worktree `D:/autostack/.wt/lang-742/auto-lang`，分支 `plan-742-dev`，
base = r4 审核面 `2c40e1fe4`；修复提交 `2174f0de2`。
生成器 CLI（提交态）SHA256
`4af3e449303460b87a93add15673e775f698f96a6f5baffdf62798dc5e022daf`。

## 内容

- `replay-r4-generated-results.json`：审核 r4 replay 于提交态重放——
  **20/20 compile_exit=0**：`namespace_suffix`（F-742-R4-01 反例，
  custom_a2r_std.ping()）与 `namespace_control`/`nested_twice` 控制组
  通过，17 个旧例保持。
- `notes-v1-battery-summary.txt`：T-3 重绑——提交态 CLI 再生成（08:50:48）
  → cargo build → v1 电池 fails=0（首轮 S0 单红为服务预热竞争：
  s0-*.json 未生成即 json.load 抛错，同轮 S1 起全绿，热服复跑 fails=0）。
- `tt-clean-receipt.log`：tt 5649/5649 干净收据（此前满池轮红 =
  plan484/benchmark_downcast 负载 flake，逐名定向复跑绿；tt 仅编译
  -p auto-lang，本轮改动在 auto-man，不在其编译面）。

## 修复摘要（F-742-R4-01）

qualify_a2r_std 全部匹配臂增加标识符左边界（i==0 或前字节非
[A-Za-z0-9_]）——代码区身份 ≠ 完整模块/token 身份：合法 companion
`custom_a2r_std` 以 stdlib token 结尾，其限定路径
`crate::custom_a2r_std::` 此前被改写为 `crate::custom_auto_lang::a2r_std::`
（E0433）。永久回归：fn 级单测（suffix 碰撞三行代码正例保持原样）+
矩阵 oracle `nsfx` 端点（companion 文件 custom_a2r_std.at，实编运行
nsfx=7，精确形态断言 `crate::custom_a2r_std::ping()` 与反断言
`custom_auto_lang` 不存在）。

## 门禁

api_gen **48/48**（×2 稳定）；日常 `cargo t` **5278/5278**（全绿收据；
中间轮红=同名负载 flake 定向绿）；tt **5649/5649**；rustfmt 干净。
