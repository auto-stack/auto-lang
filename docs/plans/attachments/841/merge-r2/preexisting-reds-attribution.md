# Phase A2 ui-t 两红的预存归因（M841 已知红族）

2026-10-27，merge r2 Phase A2 `ui-t`（UI 4903bef）：2360 运行 / 2358 过 / 2 败 / 886 skip；
plan492 金丝雀（M841-GATE-05 修复对象）**全档负载下通过**。

## 红集（仅两枚）

1. `auto-ui-tools vue::plan593_index_css_golden_tests::index_css_values_match_p1_baseline`
   — vue.rs:13148 `canonical 归一后 index.css 值对漂移`：实际值缺 `success/warning/info/error`
   四键（金样含、发射已无）。
2. `auto-ui-tools rust_ui::tests::merged_api_client_crud_fallback_for_uncovered_endpoints`
   — rust_ui.rs:7111 `GET no-param fallback = list-all over API_DATA` 断言失败（实际走
   api_impl 直调生成路径）。

## 归因证据

- **同因复现于 pre-841 主检出配对**（lang master `7e498e4cc` + UI master `b4fe000`，
  即 850 批量档时点的仓配对，main checkout 实测 2026-10-27）：
  - `index_css_values_match_p1_baseline` 红于 vue.rs:13102，同一断言、同一缺失四键集；
  - `merged_api_client_crud_fallback_for_uncovered_endpoints` 红于 rust_ui.rs:7065
    （行号位移源于 841 在该文件的增量，测试逻辑同源），同一断言消息。
  → 两红在无 841 分支代码的配对上以相同签名复现——**非 PLAN-841 引入**。
- **计划内既有记录**：T-08 收口（v07-v08-closeout.md §5，2026-10-26）已记
  `auto-ui-tools --lib` 宽面 184 过/2 败=同两枚，明确定性"本计划未改这些生产路径或
  golden，不借改写无关 golden 取得全绿，留独立基线债复核"。
- 850 批量档（2026-10-26T00:02Z，同仓配对）ui-full 3073 全绿的记录与此实测冲突，
  提示两测试对仓/环境状态敏感（生成物/伴生文件类输入）；以**当前同代码同配对复现**
  为准归因预存。独立基线债复核归后续批量/债务流程，不由 841 承载。

## 门禁处置

- 不改选择面/断言/时限；ui-t 门=FAIL 集**精确等于**上述已知对（子集匹配且无新红）
  才视为通过；任何其他红=门失败。fail-fast 取消的 71 项为该两红位置的确定性下游，
  与 plan-832 merge 先例（"ti/ui-t 归因预存红与 10-25 同集"）同口径记录。
- ui-tu/ui-ti/lang 各档零容忍。

## 日志

- 分支全档：`a2-ui-t.log.txt`
- 主检配对定向：`preexisting-reds-main-checkout.log.txt`

## 附：plan492 金丝雀恢复段 flake（M841-KNOWN-FLAKE-01）

同一冻结提交（UI 4903bef）整档 ui-t 4 跑：A2 过 / **A3 红**（:460 "repaired import must
restore the healthy package" 36≠44）/ a4 过 / a5 过；隔离跑与 2 并发（与 plan437 gallery 同跑
6 轮）恒绿。A3 失败运行内三个 parked Init（Area/Bar/Line）全部 slice-resumed to completion、
无任何 `[VM-INIT] ... FAILED`/resume FAILED 日志、DonutChart 无失败痕迹——所有 Init 终态
Done 而 dump 少 8 个 MouseArea（≈一图带面），指向渲染/重播物化面的负载敏感非确定性，
**无产品失败证据**；金丝雀核心负控语义（GATE-04 准入拒绝/Init 失败隔离、GATE-05 相位
记账）在全部 4 跑与定向中一致通过。:482 相位断言在 4 跑中全绿（GATE-05 修复生效）。

处置：不阻断合入；登记 KNOWN-DEBT-AND-RISKS.md（M841-KNOWN-FLAKE-01，1/4 整档频率，
需带仪表整档复现根因；后续批量回归命中时按债项路由）。日志：a2/a3/a4/a5-ui-t.log.txt。
