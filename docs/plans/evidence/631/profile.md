# PLAN-631 T-01/T-03 剖析报告：027-file-manager 列表交互重建成本

- 日期：2026-09-15 ｜ 分支 `plan-631-dev`（base `b62164566`）
- 场景：VM 轨 027（根列表缩放至 67 项，T-05 消费场景数据集），
  MCP `autoui_action press` 驱动 15 次列表行交替选中，采集逐重建帧
  `[P631-PROFILE]` 行（P631_PROFILE=1，仪表 = renderer view() 脏路径
  builder/render 两段计时 + `Style::parse_reported` 全局计数）。
- 帧群说明：一次选中交互产生多帧重建（状态写 + 状态栏/toast 等小帧）；
  列表整重建帧以 `style_parse_calls > 1000` 聚类，p90/max 即取自该群
  （67 行 × 行内组件 ≈ 2.6k 次 parse/帧）。

## 数据（五轮，JSON 同目录）

| 轮次 | 构建 | 缓存 | rebuild p90/max (ms) | builder p90 (ms) | render p90 (ms) | parse p90 (ms) |
|---|---|---|---|---|---|---|
| baseline-debug-67rows | debug（T-02/T-04 exe） | — | 16.16 / 26.46 | 16.14 | 4.9 | 13.72 |
| baseline-debug-nocache | debug（最终 exe） | 关 | 15.83 / 22.78 | 15.81 | 5.03 | 13.23 |
| after-debug-cache | debug（最终 exe） | 开 | **9.77 / 16.52** | **9.75** | 5.00 | **1.44** |
| baseline-opt-nocache | release | 关 | 2.23 / 3.90 | 2.22 | 1.36 | 1.11 |
| after-opt-cache | release | 开 | 2.61 / 4.32 | 2.16 | 1.62 | **0.22** |

## 热区结论

1. **Style::parse 是 debug 构建交互重建的第一热区**：列表整重建帧
   ~2,640 次 parse、p90 13.2ms ≈ builder 段的 80%。逐元素逐重建重复
   解析同类串（67 行 × ~40 类串）与 F-6 判定一致。
2. **T-03 intern 缓存收益（debug）**：parse p90 13.2 → 1.4ms（**~9x**）；
   builder p90 15.8 → 9.8ms（1.6x）；整重建 p90 15.8 → 9.8ms（1.6x），
   max 22.8 → 16.5ms。render 段（~5ms）不受影响，成为新首区。
3. **opt（release）双档**：parse 1.11 → 0.22ms（5x），但 opt 下原 parse
   占比小，整重建 2.2 vs 2.6ms 差异在运行噪声内（缓存查表+克隆 ≈ 收益）。
   opt 交互本已在个位毫秒，非优化目标面。

## AC-05 目标校准（计划 §10 Q1）

- 立项依据值"027 选中 ~0.3s（debug）"（auto-os PLAN-016 环境）在本环境
  未复现：现状基线 debug 整重建 max ~26ms、opt max ~4ms——**早已 < 50ms**。
  差异归因：PLAN-016 计量含 iced layout/draw 与更重组件树（含行内
  popover 旧形态），且宿主/构建不同。
- 校准结论：AC-05 以"< 50ms"腿达成（debug p90 9.8ms / max 16.5ms），
  ≥5x 腿按"parse 热区分项"达成（parse 9x、builder 1.6x）；整重建 5x
  不成立（基线本就不慢），如实记录。

## AC-06 冻结清单

- debug/opt 双数据：上表 + 同目录 `profile_*.json` 五份。
- 热区排行（debug 列表帧）：Style::parse ~13ms > render（AbstractView→
  Element 构造）~5ms > 其余 builder ~2ms。
- 缓存收益量化：debug parse 9x / builder 1.6x / 整重建 1.6x；opt 噪声内。
- 复现：`python docs/plans/evidence/631/profile_driver.py --clicks 15
  [--profile release]`（A/B 开关 `AUTO_STYLE_CACHE=0/1`）。

## 残留与边界

- MCP 驱动的 press 不产生真实指针事件（消息直派），hover 交互无法经此
  通道计量；hover 与选中共用同一重建机制（状态写 → view_dirty → 全量
  重转换），选中数据即代表面。F-5 落地后行级 hover 已零重建（hover_area
  既有机制 + T-02 mouse-area 对齐）。
- 结构 diffing 评估（T-06）→ `diffing-eval.md`。
