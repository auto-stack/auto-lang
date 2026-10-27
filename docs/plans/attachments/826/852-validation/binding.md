# PLAN-826 T-11 / V-14 — 852 统一能力验证 binding

日期：2026-10-28。树：`D:/autostack/.wt/lang-826/auto-lang`，分支 `plan-826-dev`，
基面 `01b3eb8cc`+本任务提交。D852=archive/852（r2 闭环）已交付；D798/D825/D828 均已归档。

## 逐行映射（语义ID → 实际起点 → V子例 → 正式入口 → 独立 oracle → 证据）

| 语义ID | 实际起点 | V子例 | 正式入口 | 独立 oracle | 证据 |
|---|---|---|---|---|---|
| source B01/B04/B09（正控 14/42/120） | `crates/auto-lang/test/cases/ace-m1/src/B{01,04,09}.at` 真实源码→HIR→typed MIR | V-14 | `av_wire_826_baseline::…sources_match…over_both_wires`（legacy AVAB + compact AVAC 双策略） | 852 冻结期望表（非本仓解释器自证）；optimized==reference 双路径互证 | 测试绿；loaded_pc_lookups>0 / reference=0 |
| source B13/B14（故障 IntOverflow/DivByZero） | 同上 | V-14 | 同上 | 冻结 trap 期望 | 测试绿（双 wire 均按 trap 分类，非 infra） |
| I01 稳定 body 编辑 30 修订/未改函数零工作 | D828 战役（273575B/115 函数）+ compact 会话 | V-12 子例1/2 | `verify-av-826.ps1 -Scope incremental`（t08_formal_campaign_v08_dual_gate_30_revisions + av_wire_826_incremental 5 例） | 战役双门（结构+P95<1s）；compact 会话 builds/reuses/六阶段计数 | 全绿（战役 12.8s） |
| I04 类型/布局变化失效 | 标量子集：签名变化（整型签名域） | V-12 子例3 | `av_wire_826_incremental_signature_reject_then_exact_invalidation` | 热编辑拒绝+精确失效（helper+main 重建、tail 恒等） | 绿；**复合类型/字段布局编辑=范围外**（标量 wire 无此语义，owner=源码类型系统线，关闭条件=复合类型源码支持交付） |
| I05 改回历史内容 | 内容命中（等价编辑） | V-12 子例2 | `av_wire_826_incremental_content_hit_reuses_every_index` | builds=0/reuses=3、对象恒等 | 绿 |
| I06 fresh/hot/重启/跨工程标签 | compact driver 会话+持久行 | V-06/V-12 恢复轴 | `av_wire_826_compat_edit_session…` + `av_wire_826_incremental_persisted_rows…` | 精确命中计数（冷0/编辑1/位置2）；restore_units_admitted=3（同策略）/0（跨代冷回退） | 绿 |
| I07 旧调用暂停→新版→恢复旧 | compact 快照分步/恢复 | V-14 | `av_wire_826_baseline_resume_old_call_after_new_version` | 旧值3/新值30，恢复后旧链完成且 loaded_pc_lookups>0 | 绿 |
| I10 512修订/churn+GC 有界 | 执行器资源档（wire 无关层） | V-07 | `-Scope baseline` 内 resources_tier `av_resources_32mib_product_random_read_and_churn` | 32MiB 受管/水位界 | 绿（compact 执行预算不变已在 V-07/T-04 证） |
| I11 单函数编辑周转 P95 | D828 战役 | V-12 子例1 | 同 I01 入口 | 战役 P95 门 | 绿 |
| I12 灵敏度负控 | 父侧 judge + 错 oracle | V-10/V-14 | `av_wire_826_parent_sensitivity_controls_non_green`（AV826_JUDGE_SABOTAGE 四轴+错 exit） | 各负控必非绿 | 绿 |
| data/owned 旧 codec 路由不退化 | 已交付 AVAD 族 | V-11/V-09 | `-Scope compat`（av_data_820/av_owned_821/av_collections_822 cache_compat 5+7+8） | 各族自有期望 | 绿 |

## 体积/装载成本（原始数字，冻结五源基线）

`AV826-BASELINE raw-bytes compact=917 legacy=1739 ratio=0.527`（同语义模块、
两代 wire 的 raw 总长；compact 较 legacy 小 47.3%）。分段/压缩/loaded 成本与
WASM/V8 对标归 T-06 bench（四必需入口），本表只记 raw 基线。

## 保留的 M1 缺口（不冒领）

- 复合类型/字段布局源码编辑（I04 完整轴）：标量 wire 语义外，owner=源码类型系统线。
- source/native/AJ 面全部不在本计划（852 边界原文保留）。
- a2r 日常 emit+金样复用有效认证（零 Rust 重编译）——未在本计划触发，沿用 852/798 既有认证。
