# PLAN-830 work r4 — review R2 findings F-830-R2-01..06 修复证据

2026-10-22。同一实施树 `D:/autostack/.wt/lang-830/auto-lang`（`plan-830-dev`）。
本目录所有 JSON/trace 都由下方命令在**修复后代码**上重跑得到，不是文案复用。

## 基线

- 基面 `ab56c32ec4cb28e01fbcca74e9861164ffad43f0`（autographics-dev）。
- review R2 冻结 `dbaea29e122d2fa2a18d18ce3f09444583098efd`；本修复提交
  **`e8d551a2b`**（`agp_commit=e8d551a2bb1a`，见各 report `fingerprints`）。
- AGP `Cargo.lock` sha256 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`
  （未变）；依赖仍为 stock iced 0.14.0 / iced_widget 0.14.2 / wgpu 27.0.1。
- 计划 `plan_revision: 3`，状态 `executing → execution_done`（8/10，T-04..T-08 重开后再完成）。

## 结果总表

| 证据 | 命令 | 结果 |
|---|---|---|
| AGP 全量 | `cargo test --locked` | **151/151**（[明细](agp-tests.txt)），其中 `surface_contract` 29/29、`surface_review_r2` 5/5 |
| 类型/格式 | `cargo check --locked`、`cargo fmt --all -- --check` | 零 warning、格式通过（[check](check.txt)、[fmt](fmt.txt)） |
| review 原始 holdout（未改一字） | [replay-holdouts.ps1](replay-holdouts.ps1) | **4/4 通过**（[输出](holdouts.txt)） |
| 场景 oracle 敏感性 | [replay-scenario-oracle.ps1](replay-scenario-oracle.ps1) | 合法对照 exit=0（15/15）；路由 local 变异 `[9999,9999]` → **exit=1，8 项门禁红**（[trace](scenario-mutation.txt)、[失败项](scenario-corrupt-failures.txt)） |
| V-02 离屏 | release `--case surface-offscreen` | 6/6 |
| V-02/03/04 + S-01 | release `--case surface-compose` | 17/17 |
| V-03/04/05 输入 | release `--case surface-input` | **18/18**（含焦点迁移/真实 caret 锚点门禁） |
| V-03/04/06 场景 | release `--case surface-scenarios` | **15/15**（含 `s02_route_oracle_rejects_corrupt_local`） |
| V-07/08/09 生命周期 | release `--case surface-lifecycle` | **15/15**（含三类型设备归属、resize drop 守恒） |
| V-05 真实 OS 中文 IME | release `--case surface-ime-os` | **14/14**：真实 caret、caret 随输入移动、锚点在投影 caret 矩形内、CJK commit（本轮 `爱彼此你好你好`） |
| 829 兼容 | `ag2d-breakout` / `ag3d-squash` / `iced-shared-device` | 全部 exit=0 |
| 独立消费/重定位 | `--case independent-consumer` | pass，A→B **29 字段 0 差异**、根生产 0 collateral |

每个 case 的门禁清单与 report 指纹见 [gates.txt](gates.txt)；原始报告在各子目录
`report.json`（release profile）。

## 六项 finding → 永久用例/门禁

| finding | 永久回归（测试入口） | 正式门禁 |
|---|---|---|
| R2-01 活体内容不刷新消费者 | `tests/surface_review_r2.rs::iced_message_update_refreshes_every_composed_consumer`（双消费者 + 静态帧/显式 dirty 对照） | S-01/S-02 真实 message→像素路径保持绿 |
| R2-02 焦点迁移不释放旧控件 | `tests/surface_review_r2.rs::focus_migration_unfocuses_the_old_iced_control` | `surface-input::focus_migration_releases_the_old_control`（两真实 text_input、A→B、preedit 后切换、blur） |
| R2-03 锚点固定坐标 | `tests/surface_review_r2.rs::ime_anchor_follows_the_real_caret` | `surface-ime-os::{real_caret_reported_by_the_widget,caret_follows_typing_and_scroll,anchor_tracks_the_active_mount,anchor_region_is_the_caret_not_the_plane,anchor_inside_projected_caret_region}` |
| R2-04 外设备 3D 绕过 | `tests/surface_review_r2.rs::every_adapter_checks_its_real_device`（2D/iced/3D + raw 不可验证 + 同设备正控） | `surface-lifecycle::neg_second_device_caught` |
| R2-05 resize 替换不计 drop | `tests/surface_review_r2.rs::target_accounting_balances_every_exit` | `surface-lifecycle::resize_replacement_is_counted_as_a_drop`（no-op 不虚增） |
| R2-06 场景门禁绕过路由输出 | `surface-scenarios::{s02_route_oracle_rejects_corrupt_local,s02_*_route_round_trip,s03_*}` + 外部 9999 变异 | 变异下整体 exit=1（8 项红） |

## 复现

```powershell
# 1) AGP 全量 + 契约 + 新永久回归
cargo test --manifest-path experimental/autographics-prototype/Cargo.toml --locked

# 2) review 原始四项 holdout（源码未改）
powershell -File docs/plans/attachments/830/work-r4/replay-holdouts.ps1

# 3) 场景 oracle 故障敏感性（control vs corrupted local）
powershell -File docs/plans/attachments/830/work-r4/replay-scenario-oracle.ps1

# 4) 六个 surface case（release）
$P = experimental/autographics-prototype/target/release/graphics-probe.exe
foreach ($c in 'surface-offscreen','surface-compose','surface-input','surface-scenarios','surface-lifecycle','surface-ime-os') {
  & $P --case $c --replay canonical --report scratch/830/work-r4/$c
}
```

## 边界

- 本轮未跑根仓 `cargo t/tv/tt/tb/tu/taa/docs_gen`（仅 experimental 变化不触发）；
  merge-owned 全套门禁在最终 review pass 后由 merge 在新冻结提交上执行。
- 场景 oracle 的旧 R2 冻结副本（`review-r2/scenario-source.rs`）**故意保持冻结**；
  它对新 oracle 不再有证据效力（本目录 `scenario-source.rs` sha256
  `fc07862c…`，见 `scenario-source.sha256.txt`）。本次未改写入 review 附件。
- 候选窗逐像素模板比对仍不在合同内；`gpu_timing_available=false` 不伪造。
