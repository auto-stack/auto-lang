---
plan_id: PLAN-588
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: Stage B P-4——VM 债族修复（CALL 内联实参形态跨迭代退化）
author: [zhaopuming, ZCode]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm]       # 受影响的 specs 路径
current_step: 1
total_steps: 6
---

# [PLAN-588] Stage B P-4——VM 债族修复：CALL 内联实参形态跨迭代退化（583 残留债①）

## 变更摘要

583 台账残留债①「CALL 结果内联作算术操作数/字符串跨 fn 返回特定形态静默
归 0」（KNOWN-DEBT P584-D3；shell.at 为该形态高密度用户，P-5 搬迁回归前
修复比踩雷便宜）。起草期已独立复现并定界：

- **稳定失败形态**（scratch/p583/v1/v4）：`total = total + f(file.read_text(p))`
  ——native 结果**内联作 fn 实参** + CALL 结果**内联作算术操作数**，循环
  第 2 轮起 CALL 返回值在消费点丢失（首轮正确）；伴随输出流丢印/跨块串值
  → 栈漂移迹象。
- **已排除**：被调方实参存活（callee 自报 `in:1336` 全程正确）；let 绑定
  中转（`let t = read_text(p)` 再传）完全正常；常量实参/纯字符串操作 callee
  单独正常；池 dedup/freelist 日志（P419_POOL_LOG）无 rc 异常。
- **嫌疑面**：CALL_NAT 死区结算 × 内联实参栈几何的交互（sp_before/after
  错位释放）；或 CALL/RET 帧扫描 rc_release_slot_range 对内联临时槽的
  误释放——待引擎级 sp 追踪定罪。

门档：Category B（VM 引擎局部）——`cargo tv` 主档 + 复现语料回归锚；
折叠前 `cargo tf`。worktree=lang-588。

## 目标

1. 稳定失败形态修复：内联 native 实参 + CALL 算术操作数形态，循环内逐轮
   正确（v1/v4 判据）；m12/m16 复现器（存活文件版）逐轮对账。
2. 回归语料入 corpus（test/vm/99_p588debt/ 内联实参族 ≥3 形态）。
3. 全量零回归：`cargo tv` + 折叠前 `cargo tf` 与基线一致（charts 预存豁免）。
4. KNOWN-DEBT P584-D3 残留①结案（残留② JsonValue str 分派不在本批）。

## 架构方案

| 件 | 落点 | 说明 |
|---|---|---|
| D1 定罪 | engine.rs CALL_NAT 死区结算/RET 帧扫描 + sp 追踪 | 临时 trace 定位漂移点（证据留 scratch/p588/） |
| D2 修复 | 按定罪单点修 | 栈几何/rc 记账配平；不改语义 |
| D3 回归锚 | test/vm/99_p588debt/{001_inline_arg_accum,002_str_return_fn,003_m16_shape} | 金样 .expected.out |

## 需求分析与背景调查

（2026-09-07 探针全档 scratch/p583/{v1..v4,probe_min*}；关键判据：）
- v4：callee print("in:"+len) 全程 1336，外层 a 恒 1336 → 返回值消费点丢失。
- v3（let t 中转）同形全对 → 触发面=内联实参的栈几何而非字符串池内容。
- v2：不同 callee（split/find/loop）皆触发 + 丢印 → 漂移累积非内容相关。
- 583 台账 m12/m16 原文件引用已随 P-1 迁移失配，复现器以存活文件适配
  （scratch/p583/m16b.adapt.at：inline 319890 vs nested 1805）。

## 详细设计

（D1 定罪后回填——预期修复面：CALL_NAT 死区结算区间或 RET 帧扫描区间
对「shim 推栈结果落位与死区区间重叠/邻接」的处理；或 CALL 实参消费的
stake 标记（PLAN-062 T12 影子份额）在内联临时槽上的缺位。）

## 测试设计

Category B：`cargo tv` 主档（纯 .at 语料 golden）；修复语料 3+ 形态；
`cargo tf` 折叠前。零 aavm 触发（不改 auto/lib）。

## 验收标准

- [ ] v1/v4 稳定失败形态逐轮正确（探针判据复跑）。
- [ ] m16b 适配复现器 inline == nested（逐字节对账）。
- [ ] corpus 回归锚 ≥3 形态入档，`cargo tv` 绿。
- [ ] `cargo tv` 全量与基线一致；折叠前 `cargo tf` 唯红=charts 预存。
- [ ] KNOWN-DEBT P584-D3 残留①结案注记。

## 执行步骤

1. [ ] D1 引擎级 sp/rc 追踪定罪（证据 scratch/p588/）。
2. [ ] D2 单点修复。
3. [ ] D3 corpus 回归锚 ≥3 形态 + tv 绿。
4. [ ] 复现器复跑对账（v1/v4/m16b）。
5. [ ] 门档：tv + 折叠前 tf。
6. [ ] 收口：P584-D3① 结案 + specs 沉淀 + 归档。

## 复审记录

（待 /auto-plan:review 填写。）

## 待澄清事项

（无——583 台账已定性为债非设计；修复以栈几何/rc 配平为唯一出口，
语义零变化。）

## 变更摘要

## 目标

## 架构方案

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

## 详细设计

## 测试设计

## 验收标准

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

## 复审记录

## 待澄清事项
