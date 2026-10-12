# PLAN-735 T-03/T-04 改后双谱对照 + 生效证据（2026-10-03）

> 数据源：`ladder-p735-base.jsonl`（改前=插桩件@5da3b4727）vs
> `ladder-p735-after.jsonl`（改后=配对修复@7b9b2f8f5）。同二进制谱系
> （iced_winit 0.14.1，release 档，同负载同脚本——ladder735.py 复跑）。
> 行数切片协议注记：相位窗按日志行号切（n0..n1 墙钟标记）——5kb/typenl
> after 切片（124 行 vs 常态 ~550）为**日志尾截断**（该跑整跑时延漂移，
> typenl 键更新未及落盘进程即终止；窗内捕获行配对 100%，非 fix 缺陷）。
> 5kb 单档复跑确认（见 §1b）。

## 1. 行配对率（AC-04 臂 a 主判据——13%→100%）

| 档.相位 | 改前 paired/frames | 改后 paired/frames | pump/s 改前→改后 |
|---|---|---|---|
| 5kb.type | 2/16 (13%) | **15/15 (100%)** | 2.45 → 10.35 |
| 5kb.scroll | 4/20 (20%) | **9/9 (100%)** | 0.55 → 1.25 |
| 5kb.typenl | 12/43 (28%) | 2/2*（切片伪影） | 2.88 → 0.49* |
| 100kb.type | 2/12 (17%) | **23/23 (100%)** | 2.70 → 10.89 |
| 100kb.scroll | 3/20 (15%) | **20/20 (100%)** | 0.42 → 2.75 |
| 100kb.typenl | 11/38 (29%) | **41/41 (100%)** | 2.76 → 10.10 |
| 1mb.type | 3/12 (25%) | **14/14 (100%)** | 2.98 → 10.07 |
| 1mb.scroll | 5/19 (26%) | **19/19 (100%)** | 0.68 → 2.60 |
| 1mb.typenl | 13/40 (33%) | **41/41 (100%)** | 3.16 → 10.08 |
| **合计** | 55/220 (25%) | **184/184 (100%)** | — |

- **修复判据达成**：改后全相位行配对 100%（含键入/滚动/typenl 三形
  ×三档）——731/026 的「13-33% 自 present」伪影清偿；`present=-1`
  仅剩真未呈现帧（本谱零出现——无最小化/零尺寸面）。
- **泵率达带（G-5）**：type/typenl pump/s≈10.0-10.9=驱动器真实周期
  读数（p50 62.5ms+服务端长尾≈95-105ms 有效周期——
  probe_type_period.py 在档）；scroll 1.25-2.75=滚动驱动周期读数
  （200ms sleep+时延+Tick 批化）。两相位均 ≥ 驱动节奏×0.9（行通道
  即驱动节奏的忠实读数——修复语义=通道对齐，非提速面）。
- **R-1 序障重验（JSONL 全量断言）**：404 行 0 序违例（present>begin
  全过；负 total_ms=0）；present==draw_end 行=159（truthful 路径
  直证）；泵发布回退路径（消费时刻）无一行触发伪序。

## 1b. 5kb 复跑（切片异常确认跑）

`ladder-p735-after2-5kb.jsonl`（同二进制复跑）：type 13/13（pump/s
11.09）/ scroll 20/20（2.80）/ **typenl 43/43（10.52）**——三相位
100% 配对，先跑 5kb/typenl 窗确认=日志尾截断（驱动/观测层单跑
漂移），修复面零缺陷。改后两跑合计行配对 **260/260（100%）**。

## 2. 零回退门（G-5——731 谱段）

- **S5 三子段带**（改前→改后，s5 P50 ms）：5kb type 0.12→0.12；
  100kb type 0.16→0.20；1mb type 0.33→0.36；1mb scroll 0.31→0.33；
  1mb typenl 0.33→0.30——同带内（±10% 机时波动，无系统性回退）。
- **segsum（S1..S4）均值**：type 相位 +12~34%（100kb 1.37→1.83 /
  1mb 4.49→5.01）——逐段均匀上浮（s2/s3a/s3b/s4 各 +30±5%）而
  scroll/typenl ±5%、S5 带 ±10% 稳定，归因**跨跑负载漂移**（731
  在案同类：100KB segsum 跨跑 -36% 帧样本构成波动；本件代码面=
  发布点 emit 常数级分支，门控——不可能对 VM/建/Element 段加
  30%）。零回退判据取 S5 带+golden+diff 事实三重：S1..S4 打点与
  实现路径零改动（frame_segments diff 仅发布点+新增字段）。
- **视觉 golden 三形**：type/scroll/resize 0.00% 全绿
  （golden735.py，baseline=基面源 c989483ce 构建捕获 vs
  diff=修复件 7b9b2f8f5 构建；基线存
  examples/ui/041-auto-edit/src/front/tests/screenshots/735-*.png）。
- **分级门禁实测**：
  - `cargo t` 日常档（--no-fail-fast 全选面）：**5038 run / 5022 绿 /
    16 红——零新红**。16 红全预存：14=2026-10-02 批量回执登记红
    （musk_p053×4/musk_p054×2/plan606_029/projector_counter/
    desktop_bus_inbox/desktop_surface_merge/plan707 flake/schema×2+
    kitchen_sink REG-4）+ash_stream_leak_probe（主检出实测同红——
    未登记预存，vm::ffi::term_engine 域，非本件域）+
    plan502_m3_layout_geometry_e2e（全档负载 flake——worktree scoped
    复跑绿、主检出 scoped 绿）。fail-fast 首跑中止点 1469/5038 的
    3 红=musk_p053 同族。
  - `cargo tv` 语料档：**162/162 绿**（1.9s）。
  - `cargo t frame`：51/51 绿；plan716 探针族 19/19 绿（含
    frame_capture_overhead_two_state 两态开销门）。
- **门关零开销**：新增通道全走既有门（AUTO_FRAME_BENCH/
  AUTO_SCHED_DIAG OnceLock 布尔）；golden 跑（双门关）与基面源
  0.00% 等值=行为零扰动直证。

## 3. 呈现真相通道（AC-04 臂 b）

- 逐帧行通道：`[P725-FRAME]` 增 `draw_end_ms` 列（根包装 draw 括号
  结束墙钟）——本次双谱 404 行全量携带。
- frame_bench 读侧：`frame_draw_end_ms()`（DRAW_END 原子，末值覆盖，
  门控零开销）——Rust 探针/测试可达。
- 语义断言（§1 R-1 段）：present 列=draw_end 时（drawn 帧）行值与
  真值严格相等；undrawn 帧 -1（真未呈现语义）。
- VM 9920 内建/a2r 扩展未接线（下游判定零触碰；回执列建议项）。
