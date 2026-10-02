# PLAN-731 下游重判解锁回执（v0.1-M4.2/M4 完全体位点预告）

> 承 725 `downstream-handoff.md` 同款（669 模式：上游出谱与回执，判定归
> 下游重判件——判定口径 frozen ③ 零变更）。消费方=auto-edit 帧两行重判
> 收口件（v0.1-M4.2 位点；M4.1 tag 后余题清单头位承接）。

## 1. 上游改后谱（重判依据面）

| 档.相位 | s5 P50 debug | s5 P50 release | 滚动尾帧 P95 debug | fold_scan |
|---|---|---|---|---|
| 1mb.type | 7.06 → **0.81ms** | 1.16 → **0.36ms** | — | 6.29ms → ~1µs |
| 1mb.scroll | 7.01 → 0.79ms | 1.25 → 0.42ms | 38.66 → **1.49ms** | 6.91ms → ~1µs |
| 1mb.typenl（022/024 协议形） | 6.22 → 0.80ms | 1.08 → 0.34ms | — | 5.52ms → ~1µs |

- S1-S4 segsum 零回退（±2% 噪声带）；视觉 golden 三形 0.00%（frozen ①②）。
- 双态谱在档：`ladder-{baseline,after}.jsonl`（debug）+
  `ladder-{baseline,after}-release.jsonl`（release）；
  复跑=`python docs/plans/evidence/731/ladder.py --label <x> --out <p>`。

## 2. 尾帧归因状态（024 遗题）

- 022/024 P95 95-112ms 尾帧在**上游 041-auto-edit 例不复现**（debug+
  release、type/scroll/typenl 全相位，最大 s5 P95=2.35ms 改前）——尾帧
  为下游 app 特有形态（候选：页面组件树规模/autodown 块编辑器多
  CodeEditor 嵌布/装载后首帧行）。
- **本件插桩已随 master 下沉**：`[P725-FRAME]` 行新增
  `s5_layout_us/s5_shaping_us/s5_draw_us`+`[P725-ARMS]` 臂钻取
  （ce_draw_block/ce_gutter/ce_fold_scan/ce_text_runs）——下游重判件
  bench 复跑时**尾帧可直接分段归因**（无需上游再埋点）。
- 泵读数注记：distinct present 消费滞后于实际呈现（配对面——见
  SD-01 §4b）；下游 fps 判定若经帧泵通道读，需按 022 原口径
  （distinct present 计数）与实况节奏对账。

## 3. 重判件预告（消费清单）

1. 下游 bench 帧档复跑（022 协议原样：autoui_type 换行连发
   cursor-follow+EnumDisplaySettings@60Hz 面板率读回）。
2. budgets 帧两行转绿判定（P95 ≤16.7ms / scroll ≥面板×0.9）；
   对比表滚动行 022 首判+024 重判+731 后三列并陈。
3. 尾帧若仍超界：读 s5 三子段+臂钻取定位（上游归因通道已备）；
   段内面（如 autodown 块编辑器多实例嵌布）归下游件另立。
4. anchors --verify+render --check 收口（024 同款）。

## 4. 判定口径（frozen ③——原样重申）

- type_latency=帧内 P95 ≤1 帧（16.7ms@60Hz）；scroll_fps=distinct
  present ≥面板×0.9。本件零重定义（只压时长）。
