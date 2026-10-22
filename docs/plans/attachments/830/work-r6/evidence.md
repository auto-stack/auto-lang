# PLAN-830 work r6 — review R4 findings F-830-R4-01..02 修复证据

2026-10-22。同一实施树 `D:/autostack/.wt/lang-830/auto-lang`（`plan-830-dev`）。
所有 JSON/trace 均由下方命令在**修复后代码**上重跑。

## 基线

- 基面 `ab56c32ec4cb28e01fbcca74e9861164ffad43f0`（autographics-dev）。
- review R4 冻结 `5cd03b861832b9de49e2f04637a7fce130b1e7dd`；本修复提交
  **`764e87559`**（`agp_commit=764e87559cff`）。
- AGP `Cargo.lock` sha256 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`
  （未变）；stock iced 0.14 / iced_widget 0.14.2 / wgpu 27.0.1。
- 计划 `plan_revision: 3`，状态 `executing → execution_done`（8/10）。

## 结果总表

| 证据 | 命令 | 结果 |
|---|---|---|
| AGP 全量 | `cargo test --locked` | **162/162**（[明细](agp-tests.txt)）：contract 29、r2 5、r3 6、r4 5 |
| 类型/格式 | `cargo check --locked`、`cargo fmt --all -- --check` | 零 warning、格式通过（[check](check.txt)、[fmt](fmt.txt)） |
| review R4 焦点出口（未改一字） | [replay-focus-exits.ps1](replay-focus-exits.ps1) | **6/6 通过**（[输出](focus-exits.txt)）：取消先于 pending、正确 revalidate 卸载、取消后 flush 控制、A→B 合批、foreign 拒绝 |
| review R3 组合反例 + 场景反控 | review-r3 `replay.ps1` | 3/3；场景合法 15/15 exit=0、9999 变异 8 门红 exit=1（[输出](r3-replay.txt)） |
| 原 R2 四项 holdout | work-r4 `replay-holdouts.ps1` | **4/4**（[输出](r2-holdouts.txt)） |
| V-02 离屏 | release `--case surface-offscreen` | 6/6 |
| V-02/03/04 + S-01 | release `--case surface-compose` | 17/17 |
| V-03/04/05 输入 | release `--case surface-input` | 18/18 |
| V-03/04/06 场景 | release `--case surface-scenarios` | 15/15 |
| V-07/08/09 生命周期/计量 | release `--case surface-lifecycle` | **23/23**（新增分管线精确/总量守恒/内容精确/省略计数负控） |
| V-05 真实 OS 中文 IME | release `--case surface-ime-os` | 14/14 |
| 829 兼容 | `ag2d-breakout` / `ag3d-squash` / `iced-shared-device` | 全部 exit=0 |
| 独立消费/重定位 | `--case independent-consumer` | pass，A→B 29 字段 0 差异、根生产 0 collateral |

门禁清单与指纹见 [gates.txt](gates.txt)。

## 两项 finding → 修复与永久证据

| finding | 根因/修复 | 永久回归 | 正式门禁 |
|---|---|---|---|
| **R4-01** 取消先于 pending 渲染时焦点复活 | 宿主在 `about_to_wait` 才 flush；失焦/revalidate 在路由之后、渲染之前时，runtime 把“无目标”当“允许所有面聚焦”。修为 router 目标**三态** `Unset/Target/Cancelled`：`Cancelled` 下所有 iced 面在绘制前清焦点/组合态（事件仍投递），直到新的合法 press/公开 focus 记录 `Target` | `tests/surface_review_r4.rs` 5 例：取消先于 flush、revalidate 卸载、取消后 flush 对照、取消后新点击、取消后公开 focus | review R4 `focus-exits` 6/6；`surface-input`/`surface-ime-os` 生命周期门禁保持 |
| **R4-02** 上传门禁可被其他管线顶替 | 原平面活控只看“合计 ≥128B”（quad 160B 可冒充）；内容汇总量只用 `>=`（漏汇总仍过）。修为：`plane_uploads()/portal_quad_uploads()/content_uploads()` 分管线读取；门禁改为**分管线精确**（改写次数与字节分别 == composite 数 ×128/160）、**总量严格等于三项之和**、内容适配器 **64B 精确**活控；并加 V-08 钩子负控（真实写入保留、只跳过记账） | `surface-lifecycle` 6 项上传门禁 + `neg_plane_upload_omission_caught` / `neg_content_upload_omission_caught` | 省略平面计账 → 平面门红；省略内容汇总 → 两个总量门红；恢复后控制块重新为绿（见下） |

### 本轮实测（release，`surface-lifecycle`）

- `quad_upload_counter_matches_composites`：66 次 / 10560B == 66 composites ×160。
- `upload_total_is_the_sum_of_its_sources`：84 / 11616 B == plane(0)+quad(66/10560)+content(18/1056)。
- `content_upload_counter_is_exact`：新建空 canvas 一次内容穿透 = 1 次 / 64B。
- `plane_upload_counter_matches_composites`：平面控制块 1 次 / 128B == 1 plane composite。
- `neg_plane_upload_omission_caught`：省略后 plane 记录 0/0（composites=1）→ predicate 红；恢复后 1 次/128B 绿。
- `neg_content_upload_omission_caught`：省略后 total=0/0、parts=4/512 → 红；恢复后绿。
- 内存（不变）：进程 RSS 231.1→231.3MB、owned 4 目标=1,511,424B、GPU 内存 `available=false` 具名原因、`driver_residency` 单列。

## 复现

```powershell
cargo test --manifest-path experimental/autographics-prototype/Cargo.toml --locked
powershell -File docs/plans/attachments/830/work-r6/replay-focus-exits.ps1
powershell -File docs/plans/attachments/830/review-r3/replay.ps1
$P = experimental/autographics-prototype/target/release/graphics-probe.exe
foreach ($c in 'surface-offscreen','surface-compose','surface-input','surface-scenarios','surface-lifecycle','surface-ime-os') {
  & $P --case $c --replay canonical --report scratch/830/work-r6/$c
}
```

## 边界与如实记录

- 本轮未跑根仓 `cargo t/tv/tt/tb/tu/taa/docs_gen`；merge-owned 全套在最终 review pass 后执行。
- **环境 flake（如实保留）**：debug 阶段一次 `surface-offscreen`（运行在 R3 冻结
  `5cd03b861`，非本轮改动）因 OS 输入/前台未就绪失败，重跑通过；release 证据取
  通过的一次。失败门禁摘要见 [offscreen-flake.txt](offscreen-flake.txt)。与 review
  记录的 `iced-shared-device` 首轮 OS 点击超时同类，属注入环境，不是产品断言。
- review R4 的 `upload-mutation.rs` 冻结的是 `5cd03b861` 的源；修复后代码已内置
  等价负控（V-08 钩子），新一次 review 应从 `764e87559` 重新冻结源与断言再重建变异。
- iced 内部上传与驱动驻留内存不可从公开 API 取得，报告中具名而非估算。
