---
plan_id: PLAN-566
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: value-slim-boxing
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm, auto-val]   # Value 表示层（auto-val crate + 全仓匹配点）
current_step: 0
total_steps: 7
---

# [PLAN-566] Value 瘦身（胖变体装箱）：~232B → 32-48B

## 变更摘要

564 家族终环（564 遏制与度量 → 565 快赢 → **tag 锚点 → 566 装箱**）。
把 `auto_val::Value` 巨型 enum（~40 变体）中的胖结构体变体装箱
（`Node(Node)` → `Node(Box<Node>)` 等），使 `size_of::<Value>()` 从
~232B 降至 32-48B（5-7×），`Vec<Value>` 密集负载（VM 堆/ListValue/
引擎栈）同比例缩水——这是把单测试 ~800MB 峰值压进百兆区间的最大杠杆。

**用户裁定的风险隔离设计（2026-09-05）**：本改动涉及面大（实测 Node 57 处 +
Instance 103 处 + 单点变体 6 处），**必须在其余全部修改合入、master 提交并打
tag 之后单独执行**，保证随时可整体回退：

- 前置硬门（T0）：532/564/565 全部 folded，master 干净绿档，打 tag
  **`pre-value-slim`**；分支从该 tag 派生。
- 分阶段提交（每阶段一个 commit，单独可 revert）；任一阶段验收不过即停，
  master 不吸收半成品。
- 回退路径：`git revert`（单阶段）或整体回到 `pre-value-slim`（全量）。

## 目标

- G1: `size_of::<auto_val::Value>()` ≤48B（静态断言测试锁死，防回退）。
- G2: 564 权重表对比：最重测试峰值降 ≥3×（基线 ~800MB → 目标 ≤270MB，
      叠加 565 成果后预期进百兆区间）。
- G3: 全档测试绿（t/tf/tv 含 aavm2 全套），性能基线不回退（m2/m4 独跑
      时长与 cargo t 总时长 ≤110% 基线）。
- G4: 零语义变化：.at 语言层、字节码、序列化、金样 dump 全不变
      （装箱是纯 Rust 内部表示变化）。

## 架构方案

```
变更核心（auto-val/src/value.rs:147 Value enum）:
  Phase A  Node(Node) → Node(Box<Node>)          [57 处调用点,~232B→~90B]
  Phase B  Instance/Widget/Model/View/Grid/
           Method/Closure/FutureData 装箱         [~110 处,~90B→~48B]
  Phase C  (可选,数据裁决) Fn/ExtFn/Type 装箱     [→~32B,收益递减]

护航:
  - size_of 静态断言测试（G1）入库,永久锁死
  - 每阶段: cargo check 全仓 → cargo t → tv 档 aavm2 → 564 权重表单点复测
  - 565 P0 的编译侧/执行侧占比数据 → 裁决 Phase B/C 的取舍
```

## 需求分析与背景调查

（2026-09-05 本会话全仓实测取证，566 立项依据）

- **尺寸推算**（静态，P0 数据先行确认）：`Node`(~232B：3×AutoStr+usize+
  Args(Vec)+Obj(IndexMap~64B)+Kids(IndexMap~72B)) 是撑大 Value 的唯一鲸鱼；
  次大 Instance(Type+Obj)~88B；Widget/Grid/Closure 仅 48-56B；
  Model/View 仅 24B。
- **改动面实测**：`Value::Node(` 57 处（auto-atom parser/atom.rs、auto-gen
  data.rs、auto-lang ast.rs/atom_helpers.rs、macros 测试）；
  `Value::Instance(` 103 处（vm builder/collections/io/list/storage）；
  Widget/Model/View/Grid/Method/Future 各 **1 处**（即 enum 定义本身）。
- **安全性实测**：`Value` 无 `repr(C)`/无 `transmute`/不穿 FFI 布局边界；
  AutoUI Rust crates（aura/ui/ui_gen/a2ui）对胖变体直接构造/匹配 = **0 处**
  ——波及面收敛在 auto-atom/auto-gen/auto-lang vm 三处。
- **引擎热路径**：engine.rs 对这些变体的匹配多为判别分支（装箱零代价，
  如 engine.rs:824）；字段访问型（803/1317/2701 等）仅增一次解引用，
  且按值传递从 ~232B 拷贝降为 ~32B 拷贝，移动密集路径净收益为正。
- **性能风险点**：construct/match 高频路径（auto-atom parser 建 Node 流、
  vm storage/collections 的 Instance 流）多一次堆分配/解引用——以 G3 基线
  护栏量化，超 110% 即停下分析。

## 详细设计

### D1: 分阶段装箱与改动模式

- 统一构造 helper 先行：`impl Value { pub fn node(n: Node) -> Self { Self::Node(Box::new(n)) } ... }`
  ——调用点尽量改走 helper，收敛 diff 并集中未来调整点。
- 构造点：`Value::Node(x)` → `Value::node(x)`（或 `Value::Node(Box::new(x))`）。
- 匹配点：`if let Value::Node(n)` 的**字段访问**（`n.name`）经 Box 自动
  解引用编译不变；**显式解构**（`let Node{..} = n`）需 `*n` / `n.as_ref()`
  / `let Node{..} = *n`——编译器逐个引路，无静默漏改可能。
- Clone 语义等价（Box<T>: Clone 即深克隆，与现状同阶）；地址稳定性反而
  改善（Value 移动不再搬运 Node 载荷）。

### D2: 静态断言与基线护栏

- `auto-val` 新增测试 `size_asserts.rs`：
  `const _: () = assert!(size_of::<Value>() <= 48);`（随阶段收紧 90→48→32）。
- 基线采集（T0 时点，565 成果已在内）：m2/m4/static_diff 独跑时长 + 564
  权重表峰值快照，写入本 plan 执行证据区作为 G3 对照。

### D3: 回退与门禁设计

- tag `pre-value-slim` 为唯一回退锚点；分支 `plan-566-dev` 从 tag 派生。
- 每阶段（A/B/C）一个 commit，commit message 标注阶段与新 size_of 实测值。
- 阶段门禁：`cargo check` 全仓零错 → `cargo t` 全绿 →
  `cargo tv aavm2_` 全绿（nextest 受 564 组限流）→ 权重表该阶段复测。
- 任一门禁红：当场修复或 revert 该阶段 commit，不带病进入下一阶段。

## 测试设计

- 静态：size_of 断言（G1）逐阶段收紧。
- 行为：全档回归（t/tf/tv）+ aavm2 语料断言原样全绿（G4 的实证）。
- 性能：m2/m4 独跑时长 + `cargo t` 总时长对比基线 ≤110%（G3）。
- 内存：564 权重表 Phase A/B 后复测，峰值降幅 ≥3×（G2）。

## 验收标准

- [ ] A1: T0 前置门完整执行：532/564/565 folded，master 绿档，
      tag `pre-value-slim` 已打，分支从 tag 派生。
- [ ] A2: Phase A 完成：Node 装箱，size_of ≤96B 断言入库，57 处调用点
      迁移，全档绿。
- [ ] A3: Phase B 完成：Instance/Grid/Closure/FutureData/Widget/Model/
      View/Method 装箱，size_of ≤48B 断言入库，全档绿。
- [ ] A4: 权重表对比：最重测试峰值 ≥3× 降幅（数字记录在本 plan）。
- [ ] A5: 性能基线不回退（m2/m4/cargo t ≤110% 基线，数字记录）。
- [ ] A6: Phase C 做出明确取舍（数据裁决采纳或放弃，记录理由）。
- [ ] A7: 复审通过后 master 合入；`pre-value-slim` tag 保留作历史锚点。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T0** 前置门 + tag 锚点。
  操作: 确认 532/564/565 均已 folded（docs/plans/archive/ 有对应文件且
  master 无未吸收分支）；master 跑 `cargo t` 全绿；
  `git tag pre-value-slim`；
  `git worktree add D:/autostack/.wt/lang-566/auto-lang -b plan-566-dev pre-value-slim`；
  采集基线（m2/m4 独跑时长 + 权重表峰值快照）记入本 plan。
  验证: `git tag -l pre-value-slim` 命中；基线数据入 plan。
- **T1** size_of 审计 + 断言骨架。
  文件: `crates/auto-val/tests/size_asserts.rs`（新,集成测试）或
  `crates/auto-val/src/size_report.rs`（#[test] 打印全变体载荷尺寸表）。
  操作: 打印各变体 size_of 排名（确认 Node 为鲸鱼、Phase 顺序正确）；
  入库第一阶段断言 `size_of::<Value>() <= 232`（现状锁底）。
  验证: `cargo test -p auto-val size` 输出尺寸表。
- **T2** Phase A：Node 装箱。
  文件: `crates/auto-val/src/value.rs`（变体改 Box + `Value::node()` helper）+
  57 处调用点（auto-atom/auto-gen/auto-lang ast/auto-lang-macros tests）。
  操作: 按 D1 模式迁移；断言收紧 `<= 96`。
  验证: `cargo check` 全仓零错 → `cargo t` 全绿 → `cargo tv aavm2_` 全绿 →
  权重表 m2 单点复测记录。
- **T3** Phase B：Instance + 六个单点胖变体装箱。
  文件: `crates/auto-val/src/value.rs` + 103 处 Instance 调用点
  （vm builder/collections/io/list/storage 等）+ 六个单点变体。
  操作: 按 D1 迁移；断言收紧 `<= 48`。
  验证: 同 T2 门禁三连 + 权重表 m2/m4/static_diff 复测记录。
- **T4** 性能基线对照（G3 门禁时点检查）。
  操作: m2/m4 独跑时长 + `cargo t` 总时长 vs T0 基线；>110% 则定位
  （构造热点补 inline/预分配）或回退该阶段。
  验证: 对比数字记入本 plan 执行证据区。
- **T5** Phase C（可选）：Fn/ExtFn/Type 装箱。
  操作: 仅当权重表显示仍被 size_of 拖累（>48B 断言已过但 ≥40B）且 565 P0
  数据显示函数值在热点路径占比小才做；断言收紧 `<= 32`。
  验证: 同 T2 门禁三连。
- **T6** 权重表终测 + 全档收口。
  操作: `python scripts/measure_test_mem.py aavm2_ -F test-vm-files` 全表
  复测，对比 564 基线表（≥3× 降幅核验 A4）；`cargo tf` 全量绿。
  验证: 新表入库 `.config/test-mem-weights.md`，diff 记录。
- **T7** 复审 + 合入。
  操作: `/auto-plan:review` 全清单；合入 master（Conventional Commit）；
  tag `pre-value-slim` 保留。
  验证: master `cargo t` 绿；archive 归档。

## 复审记录

## 待澄清事项

- **Q1（断言档位）**: 终态断言定 48B 还是 32B——取决于 Phase C 取舍
  （T5 数据裁决）；建议 48B 为承诺线、32B 为尽力线。
- **Q2（Obj/Kids 内部 IndexMap 二级瘦身）**: Node 内部的 Obj/Kids 仍是
  IndexMap（~64/72B）——装箱 Node 后它们只在堆上，不再是 Value 尺寸瓶颈；
  是否进一步 Box<IndexMap> 由 Phase B 后权重表决定，默认不做（收益递减）。
- **Q3（与 auto-val 语义演进的关系）**: ValueData/ValueID（Universe 间接层）
  已是既定方向，装箱与它正交；若后续 Value 载荷迁往 Universe，装箱仍不
  冲突（Box 内搬家）。
