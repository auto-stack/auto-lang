# PLAN-742 r5 修复（work 第六轮）证据包

绑定：worktree `D:/autostack/.wt/lang-742/auto-lang`，分支 `plan-742-dev`，
base = r5 审核面 `2174f0de2`；修复提交 `8d59f5e88`。
生成器 CLI（提交态）SHA256
`cc57d2c90b084c4d5a2028250052725b24b92e8211439496e3146d9d43675341`。

## 内容

- `replay-r5-generated-results.json`：审核 r5 replay 于提交态重放——
  全部 case compile_exit=0：`compare_less`（F-742-R5-01 反例，
  repository.choose(1 < 2, s)）通过，`compare_greater`/`compare_control`/
  `multiline_literal` 控制组通过，20 个旧例保持。
- `notes-v1-battery-summary.txt`：T-3 重绑——提交态 CLI 再生成（api.rs
  2026-10-14T10:04；本次 run 因 pnpm 前端失败路径 generation.json 未落盘，
  以 api.rs mtime + CLI SHA 绑定）→ cargo build → **提交态脚本版本**
  （`git show HEAD:run_v1_battery.sh`，S0–S19）fails=0。
  注：工作树脚本含另一会话未提交的 S20 块（+181 行，mtime 09:35）——
  其内联 `$(python -c ...open('$OUT/...'))` 在 git-bash 下 /tmp 不经 MSYS
  参数转换导致 LID 提取为空、S20 全红；属该会话 WIP 自身的路径缺陷，
  非 742 框架面，未纳入本证据、未触碰其工作树。

## 修复摘要（F-742-R5-01）

split_top_level_args 的 `<` 仅在**紧贴标识符**时计入泛型深度（发射器
从不给泛型留空格）；带空格的比较操作数（`1 < 2`、`a <= b`）不再吞掉
实参逗号——`repository.choose(1 < 2, s)` 正确切分，第二 &str 参数恢复
借用。闭合侧保持饱和递减（比较 `>` 只出现在泛型深度已为 0 处）。
永久回归：fn 级单测（比较/泛型/字符串逗号 + 嵌套 Vec<Vec<T>>）+
矩阵 cmp_args/cmp_control 端点（真实 choose(bool, str) companion，
实编运行 2/2，精确形态断言 `choose(1 < 2, &s)`）。

## 门禁

api_gen **49/49**（×2 稳定）；tt **5649/5649**（干净收据；中间单红=
已知 plan484 满池负载 flake 定向绿）；rustfmt 干净。
