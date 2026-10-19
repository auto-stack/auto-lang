# PLAN-811 R7 修复轮记录（work）

2026-10-19 · stage: work · 修复 F811-R7-01（唯一开放项）· code `5df643efa`（单提交，
`crates/auto-media-core/src/image_pipeline.rs` 测试模块内 +39/−17，生产源码零变化）

## Finding → 不变量 → 修复

**F811-R7-01**（P2，AC-03/V-03，test-only）：未满负控
`full_channel_oracle_negative_control_below_saturation_stays_not_full` 用
item_budget = CAPACITY = 8，却断言 `peak_backlog < CAPACITY`，理由"正在变换者总占
一项"——该前提只在接收线程（`media-resize`）已取走至少一项后存在；fixture 未建立
此前提。受控挂起接收线程 8 秒即复现红（R7 原件 exit 101，55.59s）。

**不变量**：未满负控的"未满"必须是**输入的算术性质**，对任意调度序（含接收线程
完全未运行）成立，不依赖消费者时序前提。

**修复**（复审建议方案一）：budget = CAPACITY−1 = 7。
- 前提守卫：`assert!(item_budget < full_channel)` 就地钉住算术前提；
- 界收紧为 `peak_running ≤ item_budget`（7 项最多组合 7 个在途态）与
  `peak_backlog ≤ item_budget`（通道内至多坐着全部被接受项）——停滞的接收者也无
  法触及容量 8；`peak_backlog < full_channel` 由算术推出并显式断言；
- Ready 收敛、gauge 归零、正常 shutdown 检查全部保留；正控（有界造载+满态
  shutdown/回收）未动。

**受影响路径排查**：仅此一个测试函数改动；正控/44 项生命周期控制/其余六 harness
面无共享该前提的兄弟分支（正控断言方向相反——需要"满"，其前提由 24 项预算有界
补入建立，R7 已确认闭合）。

## 验证收据

| 检查 | 命令 | 结果 |
|---|---|---|
| 负控单测 | `cargo test --offline -p auto-media-core --lib full_channel_oracle_negative_control -- --test-threads=1` | 1 passed，43.57s |
| core 全量 | `cargo test --offline -p auto-media-core --lib -- --test-threads=1` | **44/44，0 ignored，66.22s，warning 0** |
| **迟调度组合重放**（复审自有脚本原样复用，仅换绑本修复二进制） | `python negative-schedule.py <worktree根>`（挂起子进程 `media-resize` 8s 后恢复） | **ok / exit 0 / 53.54s**（R7 同条件 exit 101）；`resumed=true`、`child_reaped=true`、二进制 sha `c8d355b3…`；收据 [r7fix-negative-schedule.json.txt](review-r7/r7fix-negative-schedule.json.txt) + stdout/stderr 同目录 |
| check | `cargo check --offline -p auto-media-core --tests` | exit 0，warning 0 |
| fmt | `rustfmt --check crates/auto-media-core/src/image_pipeline.rs` | clean |
| 生产等价 | `#[cfg(test)] mod tests` 前生产源码 LF 归一 SHA | `ce9cf376e1bdff2621be1b21878ed3dbac89eb8851e0945f8a6fb682f57b0762`，与 R7 冻结值逐字一致 → R6/R7 已过生产证据（VM 7/7、Man 3/3、HTTP 13/13、七窗口、typed consumer）按等价复用 |

## 边界

- 未改生产队列/容量/关闭序；未跑整档 t/tv/tu/tt/tb/taa/tf/docs_gen（merge-owned
  pending 不变）。
- 复审重放脚本原样执行（挂起范围=自有子进程 PID 的 `media-resize` 线程），仅
  `test-binary.json.txt` 重绑本修复二进制；脚本与断言未复制进测试——永久回归由
  算术化的负控本体承担（任何调度序下 backlog ≤ 7 < 8）。

## 独立复核附记（2026-10-19 22:56，交接会话）

按「先验证再复用」对本提交独立重跑决定性检查（不依赖上表自述）：

- **迟调度重放**（复审自有 negative-schedule.py，二进制重建后重绑 c8d355b3…）：
  **ok / exit 0 / 52.21s**，`resumed=true`、`child_reaped=true`；收据
  [reverify-negative-schedule.json.txt](review-r7/reverify-negative-schedule.json.txt)。
- core 全量：**44/44，0 ignored，62.91s**；`cargo check -p auto-media-core --tests`
  warning 0；rustfmt clean。
- 生产等价复核（同方法跨提交）：`2ffb87755`（R6 受审）= `04f7446fd`（R7 受审）=
  `5df643efa`（本修复）生产段 LF 归一 SHA 全等（`2da8f0df…`，本方法切分点含
  `#[cfg(test)]` 行故与 R7 冻结值 `ce9cf376e…` 数值不同、覆盖范围等价）——
  R6/R7 已过生产证据继续按等价复用。

**证据事故（如实登记）**：`negative-schedule.py` 以固定名写
`negative-schedule.*`——R7 红原件（exit 101）在 21:33 修复会话首次重放时即被
同 名 覆 盖 （ 该 次 绿 输 出 已 改 名 r7fix-*）， 原 件 原 始 字 节 不 可 恢 复 ； 关 键 事 实
（命令/失败文本/exit/timing/json 字段/exe hash 899a84b0…）由 review.md §3
逐字引用保真，红→绿输入（脚本本体 + 前后二进制 hash）完整。详见
[REPLAY-OUTPUTS-NOTE.txt](review-r7/REPLAY-OUTPUTS-NOTE.txt)。此为流程瑕疵，
不影响 finding 闭合的有效性（反例复现输入与两次独立绿重放均在案）。
