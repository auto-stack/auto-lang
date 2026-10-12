# PLAN-725 下游回执建议文（SD-04——Q-3 fallback 形）

> 跨仓零触碰纪律：auto-edit 仓本件零改动（其工作树另有并行会话 WIP 在
> 途——本文件为 022 T-06 Q-3 先例同款 fallback：回执内容+登记建议文随
> 附，由 auto-edit 侧自行落档）。合并后 auto-edit 侧应将下列内容并入
> `docs/upstream/2026-09-m4-perf-unblock-supply.md` §8。

## 供⑫ 清偿回执（a2r trans print 拼接形）

- **勘定修正**：当前 master（96c876cea+）print 拼接形发射已是
  `println!("{}", format!(...))`（首参字面量合法形）——供料档实录的
  `println!(format!(...))` 缺口在旧二进制（0.4.2+b50dce902）形态，现势
  源不复发。上游处置=金样锁定（`test/a2r/04_strings/010_print_concat`
  ——三形态：字面量+`.str()` 拼接/链式拼接/变量拼接）。
- **下游复验指引**：探针标记行 `print("BENCH ..." + x.str())` 形可直接
  重埋（regen 应 exit 0）。

## 供⑬ 清偿回执（a2r frame fn i32 收窄）

- **上游处置**：`a2r_std::frame::{begin_ms,present_ms}` 返回收窄 i64→i32
  （saturating——SD-B §3b 值域口径）；编译级证明=plan716 探针
  `frame_timestamps_a2r_std_parity` 扩展（`let _: i32 = begin_ms()` +
  i32 域内与源 i64 逐值对拍）。
- **下游复验指引**：探针字段可回到 `int`（i32）承接（此前下游适配的
  i64 字段承接形仍兼容 VM 轨；a2r 轨 i64 字段直赋会反向 E0308——建议
  随本清偿把探针字段改回 int 形，regen exit 0 即回执）。
- **注意**：plain-fn 车道裸 `int`=i64（.at plain-fn 惯例）——frame fn
  消费在 plain fn 内需显式 `var x i64` 承接时加宽（`i64 变量承接形已在
  案`——SD-B §3b 措辞）。

## 帧管线件回执节（管线优化件——下游重判指引）

- **上游交付**：帧管线增量更新（SD-01 契约+SD-02 上游阶梯谱节）——
  单帧单建/载荷静态判定/元素级 O(1) 高度上报/put-then-take 勘定。上游
  双态谱：5KB segsum P50 11.68→5.30ms、100KB 24.04→5.88ms、1MB
  126.76→8.07ms（尺寸缩放清零；MCP 驱动协议同 stage_frame）。
- **下游重判口径**：帧档（bench.py stage_frame）复跑——type_latency
  P95 ≤1 帧（1000/面板 Hz）/scroll_fps ≥面板率×0.9 两行判定。上游谱
  不含 S5（layout+draw 残差）与重 handler 负载（下游 SrcChanged 探针
  臂体）——真判定以下游补跑为准（不冒领）。
- **工作量预期**：改后上游键入帧 segsum ~5-8ms（60Hz 预算 16.7ms），
  下游判定带余量；若下游谱仍超界，优先查 S5 残差（首帧 shaping 债——
  editor-kernel「现状限制」在册）与 handler 体成本（下游件域）。
- **budgets 联动**：两行 validity 解锁后 budgets.json 状态位随下游件
  更新。
