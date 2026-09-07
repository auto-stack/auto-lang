---
plan_id: PLAN-588
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: Stage B P-4——VM 债族修复（CALL 内联实参形态跨迭代退化）
author: [zhaopuming, ZCode]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [reports/P588-1, reviews/P588-2]
touched_goals: [GOAL-003]             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm]       # 受影响的 specs 路径
current_step: 6
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

（D1 定罪回填。）**根因**：`file.read_text(p)` 等模块式 native 调用走实例
路径——codegen 对未入静态白名单的模块 Ident 推 `const.i32 0` 占位 receiver
（codegen.rs:8819 matches! 列表缺 `file`），shim 只弹实参不消费占位 →
每调用净漏 +1 槽且垫入表达式栈：`a = a + f(file.read_text(p))` 的 ADD 弹
[占位 0, 结果] 而非 [a, 结果]——首轮「正确」纯属 a=0 巧合，此后累加值
永不参与（AUTO_VM_TRACE_OPS ip/sp 序列定罪：迭代 1 loop 入口 sp=4 →
迭代 2 sp=5，逐轮 +1；callee 帧内 bp/sp 几何完全正常）。**修复**=沿
Plan 437 §0.6.H③ math 先例白名单 += `"file"`（单行 + 注记）；语义零变化。
其余未列入 8819 白名单的 stdlib 模块（char/conv/async/image/io/log/net/
path/process/sched/term/test/time 等）同族潜在泄漏未实证，登记观察债不
扩面（证据驱动再加）。

## 测试设计

Category B：`cargo tv` 主档（纯 .at 语料 golden）；修复语料 3+ 形态；
`cargo tf` 折叠前。零 aavm 触发（不改 auto/lib）。

## 验收标准

- [x] v1/v4 稳定失败形态逐轮正确（v4 33775/67550/101325；v1 9250）。
- [x] m16b inline == nested == 319890（修复前 nested=1805）。
- [x] corpus 回归锚 3 形态入档金样绿（99_p588debt）。
- [x] tv 唯红=charts 预存；tf 2619/2620 唯红 charts 预存（基线一致）。
- [x] KNOWN-DEBT P584-D3 残留①结案注记（残留② JsonValue 另案）。

## 执行步骤

1. [✅ 已完成] **D1 定罪**：AUTO_VM_TRACE_OPS 全指令 ip/sp/bp 追踪
   （worktree scratch/v4_trace.log）——占位槽逐轮泄漏实锤，callee 帧正常，
   病灶在 caller 表达式栈（详见设计节）。
2. [✅ 已完成] **D2 单点修**：codegen.rs:8819 白名单 += "file"（沿 math
   先例 + PLAN-588 注记）。
3. [✅ 已完成] **D3 回归锚**：99_p588debt 三形态（001 内联累积
   direct==via-let 金样、002 字符串跨 fn 返回、003 三 callee 操作数形态
   ×4 累积），`cargo tv p588debt` 3/3 绿。
4. [✅ 已完成] **复现器对账**：v4 逐轮累积 33775/67550/101325；v1 全量
   9250（=185×50）；m16b inline==nested **319890**（修复前 1805）。
5. [✅ 已完成] **门档**：tv 全量唯红=charts 预存；tf 2619/2620 与基线
   一致（charts 预存）。
6. [✅ 已完成] **收口**：P584-D3 残留①结案 + P588-1/2 沉淀 + vm/plans.md
   588 行 + INDEX 重生 + 归档。

## 复审记录

（2026-09-07 复审，verify-don't-trust。）

| # | 验收项 | 结果 | 证据 |
|---|---|---|---|
| C1 | 逐轮正确 | PASS | v4 三轮累积 33775/67550/101325；v1 9250=185×50 |
| C2 | m16b 对账 | PASS | inline==nested==319890（修复前 1805/319890） |
| C3 | corpus 锚 | PASS | 99_p588debt 三形态金样绿（direct==via-let 不变量） |
| C4 | 门档 | PASS | tv 唯红 charts 预存；tf 2619/2620 基线一致 |
| C5 | 簿记 | PASS | P584-D3①结案；P588-1/2 沉淀；vm/plans.md 588 行 |

遗漏/延后扫描：①其余 stdlib 模块（char/conv/time 等 ~13 个）未入 8819
白名单的同族潜在泄漏**未实证**——登记 P588-D1 观察债（证据驱动再加，
不扩本批面）；②583 残留②（JsonValue 元素 str 分派 None）属另案不在
本批；③修复单行语义零变化（白名单=跳过占位 receiver 压栈，math 先例
同型）。健康：零新增警告；trace 为既有 env 门控设施非残留。

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
