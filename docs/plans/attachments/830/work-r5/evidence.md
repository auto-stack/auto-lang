# PLAN-830 work r5 — review R3 findings F-830-R3-01..03 修复证据

2026-10-22。同一实施树 `D:/autostack/.wt/lang-830/auto-lang`（`plan-830-dev`）。
所有 JSON/trace 均由下方命令在**修复后代码**上重跑，不是文案复用。

## 基线

- 基面 `ab56c32ec4cb28e01fbcca74e9861164ffad43f0`（autographics-dev）。
- review R3 冻结 `5aec0a63429d0a4cd3253751b195e8f74ddc2976`；本修复提交
  **`5cd03b861`**（`agp_commit=5cd03b861832`，见各 report `fingerprints`）。
- AGP `Cargo.lock` sha256 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`
  （未变）；依赖仍为 stock iced 0.14.0 / iced_widget 0.14.2 / wgpu 27.0.1。
- 计划 `plan_revision: 3`，状态 `executing → execution_done`（8/10，T-05..T-08 重开后再完成）。

## 结果总表

| 证据 | 命令 | 结果 |
|---|---|---|
| AGP 全量 | `cargo test --locked` | **157/157**（[明细](agp-tests.txt)）：`surface_contract` 29、`surface_review_r2` 5、`surface_review_r3` 6 |
| 类型/格式 | `cargo check --locked`、`cargo fmt --all -- --check` | 零 warning、格式通过（[check](check.txt)、[fmt](fmt.txt)） |
| review R3 组合反例（未改一字） | review-r3 的 `replay.ps1` | **3/3 通过**：分帧对照、合批 A→B（widgets=(false,true)、blur=(false,false)）、foreign focus 拒绝后旧焦点保持 (true,false)（[输出](r3-replay.txt)） |
| 原 R2 四项 holdout（未改一字） | `attachments/830/work-r4/replay-holdouts.ps1` | **4/4 通过**（[输出](r2-holdouts.txt)） |
| 场景 oracle 反控 | 同上 | 合法对照 15/15 exit=0；9999 变异 8 门红 exit=1 |
| V-02 离屏 | release `--case surface-offscreen` | 6/6 |
| V-02/03/04 + S-01 | release `--case surface-compose` | 17/17 |
| V-03/04/05 输入 | release `--case surface-input` | 18/18 |
| V-03/04/06 场景 | release `--case surface-scenarios` | 15/15 |
| V-07/08/09 生命周期/计量 | release `--case surface-lifecycle` | **19/19**（新增上传/内存/平面计数器门禁） |
| V-05 真实 OS 中文 IME | release `--case surface-ime-os` | 14/14（真实 caret 与 CJK commit） |
| 829 兼容 | `ag2d-breakout` / `ag3d-squash` / `iced-shared-device` | 全部 exit=0 |
| 独立消费/重定位 | `--case independent-consumer` | pass，A→B **29 字段 0 差异**、根生产 0 collateral |

每个 case 的门禁清单与指纹见 [gates.txt](gates.txt)。

## 三项 finding → 修复与永久证据

| finding | 根因/修复 | 永久回归 | 正式门禁 |
|---|---|---|---|
| **R3-01** 合批输入双焦点 | runtime 从 HashMap 里挑一个 `ButtonPressed` 当目标，但同帧每个事件列表都会跑、各自重新聚焦；HashMap 首项不是跨面交付次序。修为：router 终态写入 `CompositionGraph::note_routed_focus`，runtime 每帧以它作为**唯一真实编辑目标**，非目标 iced 面在绘制前清焦点/组合态；事件仍投递（先前点击动作不丢） | `tests/surface_review_r3.rs::{two_clicks_queued_before_one_frame_keep_only_last_focus, reversed_batch_keeps_the_router_target, earlier_batched_click_still_acts, separate_frame_focus_transfer_is_a_valid_control}` | `surface-input::focus_migration_releases_the_old_control`；review R3 combinations 3/3 |
| **R3-02** 拒绝请求有副作用 | 公开 focus 先释放其他全部 iced 焦点、再查目标；非法 id 时旧焦点已丢。修为：先校验（存在/是 iced/本 runtime 代际），再迁移；拒绝不动当前焦点与 IME，成功后更新 router 终态 | `tests/surface_review_r3.rs::{rejected_foreign_focus_keeps_current_edit_state, rejected_stale_and_non_iced_focus_keep_current_state}` | 同上（review combinations 第 3 例） |
| **R3-03** V-09 上传/内存证据 | 组合管线（planar 128B/次、quad 160B/次）真实 `write_buffer` 记账；`upload_stats()` 汇总内容适配器；`owned_target_bytes()`；进程 RSS 采样；GPU/驱动驻留具名不可用 | `surface-lifecycle` 三项新门禁 | `uploads_measured_on_the_composition_path`、`upload_counters_cover_the_content_path`、`plane_upload_counter_is_live`、`memory_facts_reported_and_separated` |

### V-09 本轮实测（release，`surface-lifecycle` report）

- 上传（60 帧循环前后快照）：组合路径 `66` 次 / `10560` 字节 =
  `quad_composites(66) × 160B + plane_composites(0) × 128B`；全路径
  `84` 次 / `11616` 字节（含内容适配器）。排除项具名：iced 内部上传、
  纹理/bind group 创建、驱动侧拷贝。
- 平面计数器活控制：另建一个真实 3D 平面 composite → `2` 次 / `288` 字节
  （128B 平面 uniform + mesh 内容上传），证明两条组合管线都被计量。
- 内存：进程工作集 `242.8 → 243.1 MB`（OS 驻留，非归属内存）；
  owned 目标 `4` 个 = `1,511,424` 字节（Σ`w*h*4`）；GPU 内存
  `available=false` + 具名原因（wgpu 27 无可移植查询，DXGI 属 adapter 专属未实现）；
  `driver_residency` 单独标注未测量。

## 复现

```powershell
# 1) AGP 全量（157）
cargo test --manifest-path experimental/autographics-prototype/Cargo.toml --locked

# 2) review R3 的原始组合反例 + 场景反控（源码未改）
powershell -File docs/plans/attachments/830/review-r3/replay.ps1

# 3) 六个 surface case（release）
$P = experimental/autographics-prototype/target/release/graphics-probe.exe
foreach ($c in 'surface-offscreen','surface-compose','surface-input','surface-scenarios','surface-lifecycle','surface-ime-os') {
  & $P --case $c --replay canonical --report scratch/830/work-r5/$c
}
```

## 边界

- 本轮未跑根仓 `cargo t/tv/tt/tb/tu/taa/docs_gen`（仅 experimental 变化不触发）；
  merge-owned 全套在最终 review pass 后由 merge 在新冻结提交执行。
- 场景源本轮未改动（sha256 仍为 `fc07862c…`），review-r3 的冻结副本继续有效。
- iced 内部上传与驱动驻留内存不可从公开 API 取得，报告中具名而非估算。
