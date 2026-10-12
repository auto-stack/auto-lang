# PLAN-762 图表初始化批量失败归因（2026-10-11）

来源：[761 批量回归](761-batch-regression.md) tt 腿单发红——`gallery_chart_components_render_geometry`
（3.426s，`/bar-chart` 缺 `h6`；日志 `handler_BarChart_Init parked / CpuRunnable`），tf 同提交
6.435s 通过、tb 通过。代码基线 `5314cf0f18774acc7628237fabbca13cf868362c`。

## 固定条件（本分诊 worktree）

| 项 | 值 |
|---|---|
| auto-lang | worktree `D:/autostack/.wt/lang-762/auto-lang`，branch `plan-762-dev`，基面 master `11031c556`（RUST_MIN_STACK=16MB 仓库默认） |
| auto-down 兄弟 | 组内 detached `895f8d0`（依赖仓只读） |
| gallery 源 | `D:/autostack/auto-os` 主检出 `0532b8fbc`（解析序：无组内兄弟 → 主检出）；`widgets-gallery/src/front/app.at` blob `c111cb60228bd2d17c438776a4ced710b3454048`，最后触及 `73d02b50f`（Plan 717），working tree clean |
| feature 对照 | default（=t 档 ui-iced）与 test-trans（=tt 档）均在本 worktree 实测 |

761 的 tf/tt 两腿同会话、同检出、同 gallery 文件——**跨仓资产漂移候选由构造排除**；
两腿代码同为 `5314cf0f1`（窗口 vm_bridge 仅注释差异）——**代码差异候选由构造排除**。

## 失败机制（代码路径 + 确定性复现）

四路由测试的驱动 helper `ui/dynamic.rs::drive_scheduler_to_quiescence`：

```rust
let progressed =
    init.dispatched > 0 || init.completed > 0 || cpu.completed > 0 || io.completed > 0;
...
if !progressed { break; }   // "在途但无驱动手段（I/O 未就绪）——退出"
```

- `progressed` 只认**派发/完成**，不计 `cpu.slices_run > 0`（**续跑了片但尚未完成**）。
- `resume_cpu_slices` 每调用对每个 continuation 只跑一片（快照纪律，让出放回队尾同轮不重拾）；
  因此单次 helper 调用对每个 handler 至多推进"派发片 + 1 续跑片"。
- 片预算（D-2 冻结+R-1 校准）：**1M 指令 / 4ms 墙钟**（每 64 步查钟），正常由墙钟收口。
  图表 Init（band scale + 逐柱子路径字符串合成，内存分配密集）在轻载下 1-2 片内完成；
  **并行负载下缓存/带宽压力显著降低每片实际完成指令数，同一 Init 需 ≥3 片**。
- 第 3 片需求出现的那轮：无派发（FIFO 依赖序：InFlight 段未终态不派后继 demand）、
  无完成、无 I/O → `progressed=false` → `busy=true` → **早退**。断言时
  `handler_BarChart_Init` 仍 parked/CpuRunnable，几何未算，`h6` 缺失——与 761 tt 实录签名一致。

**生产路径无此缺陷**：生产驱动点 `poll_frame_pump` 的 `progressed` 判据更丰富
（含 `slices_run` 无关的 failed/silent_done 面），且帧通知泵每帧重入——让出的 continuation
下一帧继续推进；早退面只存在于"一次性驱动到静默"的测试载体语义里。

**确定性复现**（机制级，与负载无关，T-02 落为 `plan437_child_init_tests.rs` 机制钉子）：
构造必跨 ≥3 片的 Init（250k 次自旋迭代，实测 ~16.7 VM 指令/迭代 ≈ 4.2M 步 >
2×1M 指令帽，< 10M 累计护栏），单次 `drive_scheduler_to_quiescence(10_000)` 后续体
确定性仍在途（指令帽绑定，与机器速度无关：快机每片恰 1M 指令封顶，慢机墙钟帽只会
让片更短、片数更多）；完成态截止时间驱动循环随后收敛，自旋计数几何（M 250040 260）
落地为完成证据。

## 受控负载对照（有界）

| 实验 | 形态 | 结果 |
|---|---|---|
| 隔离基线（default feature） | 单测 nextest 单实例，轻载 | PASS 3.907s（1/1） |
| 均匀 CPU 超订 | 24 × python 忙旋（20 逻辑核）下重复 15 次 | **15/15 PASS**——纯 CPU 旋不构成 761 负载形态（无缓存/带宽压力，每片实际指令数未降），诚实负证据 |
| 内存带宽压力 + 全 tt 档 | 6 × 256MB 循环写 hog + `cargo tt --no-fail-fast`（未修复代码） | gallery **PASS** 7.913s（正常 ~6.4s）；全档 5613 过 / 1 超时（plan738 assembly 探针 120s 慢杀，761 已记其 83-110s 暖缓存尾测——本实验加压所致环境伪红，非目标面） |
| 修复后负载镜像 | 24 × CPU 旋下重复 15 次（T-02 完成态驱动落地后） | **15/15 PASS**——同负载同绿，完成态驱动不引入回归 |

两轮有界负载实验（15×CPU 旋、1×tt+内存 hog）均未自然再现单发红——761 的触发是
特定并发混合（他会话 tb + 本会话 tt）下的尾事件，有界尝试不复刻其精确形态。裁定因此
不依赖自然红复现，落在：代码路径分析（判据空洞可读证）+ 761 实录签名与早退预测态
逐点吻合（parked/CpuRunnable + marker 缺失）+ **确定性机制复现**（下节钉子，
指令帽绑定、与负载无关）。不以"负载实验通过"销案——同理不以 tf 一次通过销案。

## 裁定

- **边界 = harness**：缺陷在测试载体 helper 的退出判据（`progressed` 未计片让出），
  不在生产调度语义（park/resume/片纪律/帧泵均按 PLAN-711 契约正确工作）、不在跨仓资产。
- 处置（T-02，PLAN-759 T-08 plan502 完成态驱动同 precedent）：四路由测试改
  **完成态 + 有界截止时间驱动**（无排队 demand / 无在途 CPU / 无 I/O 凭据才放行断言；
  超时红带 pending/CPU/I/O 诊断），marker 原样不放宽；补机制钉子（上节确定性复现）。
- **遗留（D-762-1，KNOWN-DEBT 登记）**：`drive_scheduler_to_quiescence` 本体判据未修
  （本计划授权面=测试文件，生产 helper 改动按计划约定"先修订确认"）。16 个调用点共享该
  早退暴露面；plan502 已自带外层完成态循环（先例），其余调用点的多片 Init 在并行负载下
  理论可复现同类红。修复方向（供后续计划）：`progressed` 增计 `cpu.slices_run > 0` 与
  `silent_done_transitions > 0`（对齐 `poll_frame_pump` 判据族），I/O 未就绪退出臂保持不变；
  届时同步更新本计划机制钉子的"单调用早退"复现断言。
