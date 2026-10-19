# PLAN-811 R6 修复记录（work）

2026-10-19 · stage: work · 修复 F811-R2-02 满通道 fixture 前置条件（R6 唯一 OPEN finding）

## 0. 修复身份

- worktree: `D:/autostack/.wt/lang-811/auto-lang`，branch `codex/plan-811-dev`
- base（受审提交）: `2ffb87755e482fb39f01123d28e3f82fa3e373c1`；本修复单提交 `04f7446fd`（累计 17 个实施提交）
- 依赖: auto-down 兄弟 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`（零源改，与 R1–R6 一致）
- 触面: 仅 `crates/auto-media-core/src/image_pipeline.rs` 的 `mod tests`（3 个 hunk 全在测试模块内，+170/−21；**生产代码零变化，lib 产物不变**，auto-lang/Man 消费者无行为影响——故 plan811 七测/Man 三测无需重跑，理由为二进制等价而非回避）

## 1. Finding → 不变量 → 闭合行

`F811-R2-02（R6 余量）满通道造载前置条件 → 满通道验收不得依赖提交突发与 worker 首次 pop 的调度竞争 → image_pipeline.rs::shutdown_with_real_decode_backlog_completes_within_external_deadline 造载段（原 :3203–3246）`

**结构诊断（与 R6 报告§3一致）**：入口 `MediaPriorityQueue::CAPACITY=8`，同优先级满队即拒（`push` 在 `len>=CAPACITY && work.priority<=lowest` 时返回 false）。原 fixture 用固定 12 次紧循环 submit：合法调度下若 2 个 decode worker 在突发期间（µs 级）未 pop 任何项，则仅 8 项被接受；而 8 个被接受任务的结构上界 = running 峰值 8（2 decoding + 1 transforming + 5 通道等待）、backlog 峰值 7（通道 8 槽满 + 1 正在变换需第 9 项），**恒不可达**断言的 `running≥9 && backlog≥8`。增大轮询时长不能创造已被入口拒绝的任务（R6 原文）。

**复现尝试（诚实记录）**：在受审基线 `2ffb87755` 上三次定向尝试未再触发红——(a) 8 个无绑定并行副本全绿；(b) PowerShell `Start-Process -ProcessorAffinity` 因 PS 5.1 无此参数失败；(c) `cmd start /AFFINITY 3` 将 6 副本钉到同 2 核仍 6/6 绿（各 ~4.8s）。窗口为测试启动瞬间的调度竞争，本机 20 核难以确定性重放；绑定证据取 R6 存档原件 `review-r6/core-tests.stdout.txt`（`panicked ... sampled running peak 8`）与上述算术结构证明，不以本地未复现否认 finding。

**修复（R6 处方「有界补入任务」路线，未新增生产调度协议/未扩大图像规模/未提高资源上限）**：

1. **正控改为有界补入**：循环内先采样（running/backlog 峰值），未观测到满（`peak_running>8 && peak_backlog>=8`）则补入一个新 distinct fingerprint 任务（预算 24 项、墙钟 20s、每次轮询一次 offer、被拒 offer 同索引重试——入口满=上游 worker 正忙，槽会打开）；观测到满即停止补入。失败诊断按 R6 要求报告 accepted/rejected_offers/两个峰值/最后采样值。阈值保持原语义（`running>8`≡原 `>=9`、`backlog>=8`=CAPACITY），未删除满队列断言、未把 running 阈值降到 8。
2. **新增未满判别负控** `full_channel_oracle_negative_control_below_saturation_stays_not_full`：同 payload/同仪表/同轮询，预算恰好 8 项——按物品数算术确定性地 running≤8、backlog≤7（正在变换者恒占一项），断言整个运行期间读数 NOT-full，且 8 项全部经正常消费路径收敛到 Ready、仪表归零（不靠 shutdown 捷径）。证明正控阈值是真实判别而非恒绿横幅。
3. **满态关闭回收补强**：正控在 shutdown 后除原 `pending_count==0`/`is_shutdown` 外，增断言 `is_closed`、`worker_count()==0`（全部自有线程 join）、`resize_backlog()==0`（真终态仪表）——即 R6 要求的「从该满状态 shutdown 完成/资源回收」。

**当前结果**：见 §2。**剩余缺口**：无（F811-R2-02 整体闭合；R1–R5 全部 finding 此前已闭且本轮重放未回归）。

## 2. 验证证据（本提交 `04f7446fd` 上实跑）

| 检查 | 结果 |
|---|---|
| 正控单跑 | 1/1 绿，4.65s |
| 负控单跑（含正常收敛） | 1/1 绿，27.35s（8 项全部 decode+resize 完成，量级符合大图串行预期） |
| 钉核压力：6 副本 /AFFINITY 3（2 核） | 6/6 绿，各 ~4.8s |
| 并行压力：8 副本无绑定 | 8/8 绿，各 ~6–7s |
| **R6 首红场景复刻**：全模块 `cargo test --offline -p auto-media-core --lib -- --test-threads=1` + 后台 `cargo check -p auto-lang --tests` 编译负载 | **44/44 绿，74.13s**（43→44 为新增负控；旧有闭控制——在途关闭归零×3、重复关闭完成语义、gauge lifecycle、scan reap、准入/open 竞态等——全部在同一串行面内重放通过） |
| `cargo check --offline -p auto-media-core --tests` | exit 0，warning 0 |
| `rustfmt --edition 2021 --check`（触面文件） | exit 0 |

新正控 fixture 累计 16 次绿（单跑1 + 钉核6 + 并行8 + 模块面1），覆盖空闲、CPU 饥饿（24+ 线程挤 2 核）、并行竞争与「模块串行+编译负载」四种形态；不以重跑取绿——失败路径带完整前提诊断（预算内未观测到满时即红并报告 accepted/峰值/末采样）。

## 3. 时序与门禁声明

按 2026-10-14 裁定：本轮仅修复 finding + 相关反例/模块测试 + 必要 check，未运行整档 `t/tv/tt/tb/tu/taa/tf`、docs_gen。test-only 改动使 auto-lang/Man 消费面二进制等价，plan811 七测/Man 三测/HTTP 13/六 harness 收据维持 R6 绑定不变。merge-owned 门禁保持 pending：最终审查通过后在冻结提交统一裸 `cargo t` + `cargo tv`（VM 触面）+ 777 strict 复核；tf 仅主检出到期批量。
