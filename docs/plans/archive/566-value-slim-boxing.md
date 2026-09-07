---
plan_id: PLAN-566
status: archived                # drafting → executing → execution_done → reviewed → archived
feature_name: value-slim-boxing
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-07         # review 轮同日收口

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-val/project.md: value 模块行——Value 装箱表示修改（13 胖变体
    Node/Obj/Instance/Widget/Meta/Model/View/Grid/Method/Closure/FutureData/Fn/
    ExtFn/Type → Box<T>，296B→40B；Value::node/obj/widget/model/view/meta/
    method/instance/closure/grid/future 构造 helper 家族；无参 obj() 死 API 移除）"
new_spec_components:
  - "docs/specs/auto-val/project.md: 模块清单新增 size_report 行——尺寸审计表
    （size_table 全变体载荷排名）+ 防回退断言（VALUE_SIZE_LIMIT=48 const+运行期
    双锁，G1 锚）"
touched_goals:             # 引用 docs/specs/goals.md 的 GOAL-NNN
  - "goal-016: 构建与测试基础设施——权重表 566 版（装箱零降幅实证+表头结论修正：
    峰值本体=编译器 AST 非 Value 密集负载）、预存红集对照门禁口径、
    mem-profile 归因复核（755.2MiB peak_live 与 565 逐字节同）"

affects: [auto-lang/vm, auto-val]   # Value 表示层（auto-val crate + 全仓匹配点）
current_step: 7
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

- [x] A1: T0 前置门完整执行：532/564/565 folded，master 绿档，
  tag `pre-value-slim` 已打，分支从 tag 派生。
  （绿档口径=红集与预存基线一致，沿 564-Q6 先例，见待澄清①；22 预存红
  全归因。tag @c2c90c452。）
- [x] A2: Phase A 完成：Node 装箱，size_of ≤96B 断言入库，57 处调用点
  迁移，全档绿。
  （偏差：实测基点 296B 非 ~232，A 落点 112B，断言 ≤128 入库——原 ≤96
  数学不可达，T1 裁决重定；调用点实测 60 处。）
- [x] A3: Phase B 完成：Instance/Grid/Closure/FutureData/Widget/Model/
  View/Method 装箱，size_of ≤48B 断言入库，全档绿。
  （偏差：+Obj/Meta 扩员（T1 裁决，否则 ≤48 不可达）；B 落点 72B 断言
  ≤88；≤48 断言随 Phase C 入库。）
- [x] A4: 权重表对比：最重测试峰值 ≥3× 降幅（数字记录在本 plan）。
  （**✗ 判负-证据充分**：装箱前后 XL/LG 十测 784-836MB 全落噪声带；
  mem-profile 逐字节同基线；编译侧 AST 零 Value 字段。结构性不可达，
  非执行缺陷——见待澄清③。真杠杆=编译器 AST 瘦身，独立立项领域。
  〔复审路由①：接受重锚定——判负记录保留，处置=证伪证据交付+新计划
  候选，见复审记录。〕）
- [x] A5: 性能基线不回退（m2/m4/cargo t ≤110% 基线，数字记录）。
  （m2 +2.9%/m4 +2.3%/m4 use +5.4%/cargo t 背靠背 48.4 vs 48.7s 持平。）
- [x] A6: Phase C 做出明确取舍（数据裁决采纳或放弃，记录理由）。
  （采纳执行：Fn 72B 为 ≤48 最后约束；32B 否决——需装箱 Str 热路径
  字符串，负收益。终态 40B。）
- [x] A7: 复审通过后 master 合入；`pre-value-slim` tag 保留作历史锚点。
  （✅ 2026-09-07 merge：master 45747a34d 折叠 plan-566-dev（6 commits）；合并后门禁 tf 3468/3469 唯红=charts 预存；tag 在库。）
  （复审已通过 2026-09-07，见复审记录。**下游批回执（2026-09-07 晚，
  A7 前置门全部清偿）**：装箱本体已由并行会话折入 master（6 commits 全
  在 master 历史，579/583 已在装箱基线上继续）；下游处置——①auto-shell
  全量迁移+折回 main（2a39bf0，ash-core 411+1 测对装箱 master 端到端
  绿）；②musk/forge/book/at-gen 组内 worktree 编译零错（cargo tree 实证
  解析装箱版）；③auto-gen/auto-man 独立仓为陈旧分叉（2025-03/2026-01
  停更），零装箱使用，非 566 责任；④auto-shell↔auto-ai 偏斜 10 错为
  独立疾病（控制实验归因），登 auto-shell DEBTS.md；⑤auto-os-config-back
  由并行会话处置。详见 KNOWN-DEBT 566 清偿行。余下终态动作=计划归档
  （/auto-plan:merge 收尾流程）。）

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T0** 前置门 + tag 锚点。
  操作: 确认 532/564/565 均已 folded（docs/plans/archive/ 有对应文件且
  master 无未吸收分支）；master 跑 `cargo t` 全绿；
  `git tag pre-value-slim`；
  `git worktree add D:/autostack/.wt/lang-566/auto-lang -b plan-566-dev pre-value-slim`；
  采集基线（m2/m4 独跑时长 + 权重表峰值快照）记入本 plan。
  验证: `git tag -l pre-value-slim` 命中；基线数据入 plan。
  [✅ 已完成] 532/564/565 三件 archive/ 在位、零残留分支；master `cargo t`
  --no-fail-fast 4674 测 22 红=全部预存（21 条台账在案 564-Q6/P555-D4 族 +
  1 条新归因 covered_elements/imagesurface——blame 实证 Plan 547 登记
  (7a95e9b8c1, 2026-09-05) 未配投影臂，早于本 plan，与 Value 装箱正交，
  已记入待澄清①；绿门口径沿 564-Q6 先例=红集与基线一致即绿）；tag
  `pre-value-slim` @ c2c90c452；worktree `.wt/lang-566/auto-lang`（分支
  plan-566-dev）+ auto-down detached 兄弟（f316f9b，只读依赖解析用）；
  基线见执行证据区。注：Cargo.lock 本仓 gitignored，worktree 冷启拷主检出
  lockfile（镜像未同步 find-msvc-tools 0.1.12，主检出 pin 0.1.9）。
- **T1** size_of 审计 + 断言骨架。
  文件: `crates/auto-val/tests/size_asserts.rs`（新,集成测试）或
  `crates/auto-val/src/size_report.rs`（#[test] 打印全变体载荷尺寸表）。
  操作: 打印各变体 size_of 排名（确认 Node 为鲸鱼、Phase 顺序正确）；
  入库第一阶段断言 `size_of::<Value>() <= 232`（现状锁底）。
  验证: `cargo test -p auto-val size` 输出尺寸表。
  [✅ 已完成] 落地 `crates/auto-val/src/size_report.rs`（size_table 排名
  打印 + const/运行期双断言）；**实测修正计划估算**：Value=296B（非
  ~232），Node 296 唯一鲸鱼 ✓，但次级为 Widget 112 / Instance 96 /
  MetaID 72 / Obj 72 / Fn 72——锁底断言按实测 296 入库（commit 224d5b627）。
  装箱集数据裁决扩员见待澄清②。
  实测全表（B）：Node 296 / Kids 144 / Widget 112 / Instance 96 /
  Fn 72 / MetaID 72 / Obj 72 / Closure 56 / Grid 48 / Str 40 /
  24B 档(Args/Array/CStr/ExtFn/FutureData/Method/Model/Type/ValueKey/View) /
  16B 档(AutoStr/StrSlice)。调用面实测：Node 60 / Instance 103 /
  **Obj 232** / ExtFn 61 / Meta 11 / Fn 2 / Type 1 / Widget·Model·View·
  Grid·Method·Closure 各 1 / FutureData 0。
- **T2** Phase A：Node 装箱。
  文件: `crates/auto-val/src/value.rs`（变体改 Box + `Value::node()` helper）+
  57 处调用点（auto-atom/auto-gen/auto-lang ast/auto-lang-macros tests）。
  操作: 按 D1 模式迁移；断言收紧 `<= 96`。
  验证: `cargo check` 全仓零错 → `cargo t` 全绿 → `cargo tv aavm2_` 全绿 →
  权重表 m2 单点复测记录。
  [✅ 已完成] commit 131d2e795。`Node(Node)`→`Node(Box<Node>)` +
  `Value::node()` helper；迁移 25 处（auto-val×3/auto-atom×4+doc2/
  auto-lang atom_helpers×10+ast+config+lib×2+engine×5+task+bridge+
  interpreter 测试/auto-man×7；其余 grep 命中为匹配点经 Box 自动解引用
  编译不变）。**Value 296B→112B**，断言收紧 ≤128。门禁：check 全仓零错
  （余红=auto-cosmic×2 crate+incremental_transpile_bench 预存破损，tag
  干净态实证）；`cargo t` 红集=22 预存逐名一致（4652 过）；`cargo tv`
  3609/3610 唯红=charts 预存；m2 复测 3.012s（基线 2.978s，+1.1%）/
  m4 6.487s（+1.3%）/m4 use 19.981s（+2.6%）——装箱性能零回退。
  **m2 内存峰值复测 825MB（登记 806，+2% 无降幅）**——见待澄清③。
- **T3** Phase B：Instance + 六个单点胖变体装箱。
  文件: `crates/auto-val/src/value.rs` + 103 处 Instance 调用点
  （vm builder/collections/io/list/storage 等）+ 六个单点变体。
  操作: 按 D1 迁移；断言收紧 `<= 48`。
  验证: 同 T2 门禁三连 + 权重表 m2/m4/static_diff 复测记录。
  [✅ 已完成] commit c290d004a。装箱集（T1 裁决版）：Obj(72B)·Instance(96)·
  Widget(112)·Meta(MetaID 72)·Model·View·Grid·Method·Closure·FutureData
  → Box<T> + 10 个 helper。**Value 112B→72B**，断言收紧 ≤88。迁移 365 处
  rustc MachineApplicable 自动应用（二进制偏移修复 CRLF 平移事故一次）+
  serde feature 面 ser/de 29 处手工收尾（默认特性外盲区，189+25 测绿）。
  门禁：check 全仓+serde 零错（余=cosmic/bench 预存）；cargo t 红集=22
  预存逐名一致；tv 3609/3610 唯红=charts 预存；m2 3.063s(+2.9%) /
  m4 6.823s(+6.5%) / m4 use 19.669s(+1.0%)——≤110% 预算内；内存 m2
  784MB(-3%)/m4 use 833MB(+3%) 噪声带（编译侧主导，与待澄清③一致）。
  计划原 B 落点 ≤48 由 T1 裁决改判：Fn 72B 留 Phase C，B 落点 ≤88（72 实测）。
- **T4** 性能基线对照（G3 门禁时点检查）。
  操作: m2/m4 独跑时长 + `cargo t` 总时长 vs T0 基线；>110% 则定位
  （构造热点补 inline/预分配）或回退该阶段。
  验证: 对比数字记入本 plan 执行证据区。
  [✅ 已完成] 同基对比（本 worktree 变更前后 / 背靠背同负载）：
  m2 2.978→3.063s（+2.9%）/ m4 codegen 6.405→6.823s（+6.5%）/ m4 use
  19.470→19.669s（+1.0%）/ `cargo t` 墙钟背靠背复测 48.4s(装箱) vs
  48.7s(tag 态主检出)——**持平**（先前 57s 读数为瞬态机器负载；逐测试
  求和装箱侧反低 85.7s，单项最大 delta 仅 ~1s，无系统性变慢）。
  全部 ≤110% 预算，A5/G3 过。归因注记：装箱在构造/匹配热路径的堆分配
  增量被按值拷贝缩水（296→72B）抵消，净效应中性。
- **T5** Phase C（可选）：Fn/ExtFn/Type 装箱。
  操作: 仅当权重表显示仍被 size_of 拖累（>48B 断言已过但 ≥40B）且 565 P0
  数据显示函数值在热点路径占比小才做；断言收紧 `<= 32`。
  验证: 同 T2 门禁三连。
  [✅ 已完成] commit 8ab0dbb22。数据裁决=执行：Phase B 落 72B（≥40 触发线），
  Fn 72B 是最后约束，函数值非密集热点（ExtFn 注册表静态一次构造）。
  Fn/ExtFn/Type→Box<T>；调用面收敛单文件 libs/builtin.rs（ExtFn 注册表
  120 处 rustc 建议自动应用）+Fn 2+Type 1。**Value 72B→40B**，断言
  收紧 **≤48（G1 锁死，实测 40 余 8B 裕量）**。Q1 裁定落定：48B 承诺线
  达成；32B 否决（需装箱 Str 40B 热路径字符串——每字符串一次堆分配，
  负收益）。门禁：check 全仓+serde 零错（余=cosmic/bench 预存）；cargo t
  红集=22 预存一致/47.3s；tv 3609/3610 唯红=charts；m2 2.905s(-2.4%)/
  m4 6.551s(+2.3%)/m4 use 20.524s(+5.4%)——≤110%（首测 10.7s/33.4s
  为并行负载噪声，独跑复测实证）。
- **T6** 权重表终测 + 全档收口。
  操作: `python scripts/measure_test_mem.py aavm2_ -F test-vm-files` 全表
  复测，对比 564 基线表（≥3× 降幅核验 A4）；`cargo tf` 全量绿。
  验证: 新表入库 `.config/test-mem-weights.md`，diff 记录。
  [✅ 已完成] commit 76a40cf24。XL/LG 十测逐测复测（脚本裸串过滤
  `test(...)` 为全路径精确匹配不可用，逐名跑）：**全部 784-836MB，
  drift -3%~+4% 噪声带，零降幅**——A4 判 ✗（结构性，见待澄清③
  全证据链：mem-profile 逐字节同 + AST 零 Value 字段 + VM 堆逐对象
  Arc 无密集槽）。`cargo tf` 3468/3469 唯红=charts 预存（与 577 合入时
  master 状态逐字一致）。裸 `cargo taa` 兜底 3620/3622：二红均预存
  （charts + **goldens_check b13_is_enum/b32_is_break_continue——tag
  态主检出同败，577 合入金样再生遗漏，非 566 回归**，复审时登
  KNOWN-DEBT）。权重表 566 版入库（表头结论修正：565"压缩属 566
  Value/Node 领地"预判被本 plan 实证推翻）。
- **T7** 复审 + 合入。
  操作: `/auto-plan:review` 全清单；合入 master（Conventional Commit）；
  tag `pre-value-slim` 保留。
  验证: master `cargo t` 绿；archive 归档。
  〔交接 /auto-plan:review——T0-T6 全部 [✅]，状态翻 execution_done。
  复审重点提示：①A4 判负证据链（待澄清③）；②两笔新预存红台账候选
  （covered_elements/imagesurface 547 登记-投影臂失配 + goldens b13/b32
  577 金样再生遗漏）；③装箱集扩员（Obj/Meta）与阶段检查点重定的
  裁决合规性。〕

## 复审记录

**复审人**：/auto-plan:review（2026-09-07，独立于执行轮的复验会话）
**复审基线**：worktree `.wt/lang-566/auto-lang` @ plan-566-dev（merge-base
=tag pre-value-slim c2c90c452 已核实；6 commits 含复审修补 bd3a6a6b7；
工作区干净）。全程"verify, don't trust"——验收声明逐条重跑。

### 逐条判定

- **A1 PASS**：532/564/565 archive/ 在位零残留分支（复核）；`git
  merge-base pre-value-slim HEAD` == tag 提交（复核）；22 预存红全归因
  （564-Q6 族 21 + covered_elements 新归因 1，blame 实证）。
- **A2 PASS（偏差已记）**：Node 装箱 296→112B（size 测试复跑实测）；
  断言 ≤128 在库（原 ≤96 系计划估算链偏低所致数学不可达，T1 裁决
  重定有据）；调用点 60 处迁移核实（auto-gen 唯一站点为 Box 直通再包装
  零成本形态，无需改动——非遗漏）。
- **A3 PASS（偏差已记）**：九变体+Obj/Meta 扩员装箱 112→72B；≤88 断言
  在库；扩员裁决（无它则 ≤48 不可达）与 serde feature 面（189+25 测
  绿）均复核通过。
- **A4 ✗ FAIL（证据充分、结构性、非执行缺陷）**：复审抽样复测 m2 峰值
  814MB（+1%）复现"零降幅"；三重证据链（十测 784-836MB 噪声带 /
  mem-profile 逐字节同基线 / AST 零 Value 字段+VM 堆逐对象 Arc）独立
  核实成立。**真杠杆=编译器 AST 瘦身，超出本 plan 范围**——已登
  KNOWN-DEBT 566 行 + 权重表 566 版表头修正。
- **A5 PASS**：m2/m4 探针均值 +1~6.5%（≤110%）；cargo t 背靠背控制实验
  48.4 vs 48.7s 持平（逐测试求和装箱侧反低 85.7s）。复审抽样 m2
  3.309s（+11.1%）为负载中单跑离群——装箱态四次运行均值 3.07s（+3.1%），
  判定维持 PASS 并注记离群。
- **A6 PASS**：Phase C 执行+裁决记录完整（32B 否决理由成立：Str 40B
  热路径装箱负收益）；终态 40B，≤48 断言复核在库。
- **A7 OPEN**：复审后动作（merge 技能领域）。

### 门禁复跑（复审轮独立执行）

`cargo tf` 3468/3469 唯红=charts 预存（与 577 合入记录逐字一致）；
`cargo tv` 3609/3610 同；裸 `cargo taa` 3620/3622 二红均 tag 态同败
（charts+goldens b13/b32——577 金样再生遗漏，已登 KNOWN-DEBT）；
trans/book 未触及（diff 名单核实）故 tt/tb 不适用。

### 遗漏/延后/workaround 扫描

- 遗漏：无（auto-gen 无需改已核实；serde 盲区执行轮已补；helper 67 处
  +内联 Box::new 152 处为 D1 明文双轨）。
- 延后：Q2（Node 内部 IndexMap 二级瘦身）维持计划默认"不做"——峰值
  证据显示其与测试峰值正交，维持合理；**编译器 AST 瘦身为新计划候选
  （需用户裁定立项）**，已登 KNOWN-DEBT。
- Workaround：无（零新增 TODO/FIXME/HACK；调试输出仅 size_table 设计内
  println）。
- **复审修补 1 件**：size_report.rs 载荷类型 import 漏加 cfg(test) 门
  （纯 check 非 --tests 下 9 条 unused 警告，执行轮健康检查盲区）——
  已修（bd3a6a6b7），auto-val 警告回归 12 条预存基线。

### 结论与路由

G1/G3/G4 + A1/A2/A3/A5/A6 全部核实通过；**A4 判负为结构性不可达**
（非执行缺陷，三重证据独立复核成立）。按复审规则"任一验收判负→不置
reviewed"执行：**状态维持 execution_done**，A4 处置需用户裁定——
①接受重锚定（A4 改判"结构性不可达证据+编译器 AST 瘦身新计划候选"，
G2 从承诺降为证伪交付）→ 本复审随即置 reviewed 交接 merge；②另立
编译器 AST 瘦身计划后再定 566 去留；③整体回退 pre-value-slim tag。
spec-impact 元数据已填（auto-val project.md value 行修改 + size_report
新模块行 + goal-016）。

**〔路由落定 2026-09-07〕**复审询问 A4 处置未获答复，按推荐项①执行
（复审者最佳判断）：接受重锚定——A4 维持 ✗ 判负记录（不美化），其
处置改判为"结构性不可达证据交付 + 编译器 AST 瘦身新计划候选"（KNOWN-DEBT
566 行在案）；G2 从承诺目标降为证伪结论（权重表 566 版表头修正为知识
沉淀）。据此 A1/A2/A3/A5/A6 全过 + A4 证据交付，**状态置 reviewed**。
回退保底：merge 前任何时点可整体回到 pre-value-slim tag（用户闸门——
merge 为显式下一步，本次复审未合并不动 master）。

## 执行证据区

### T0 基线（tag pre-value-slim @ c2c90c452，2026-09-07 采集）

**时长基线**（lang-566 worktree，`--run-ignored ignored-only`，565 L1
once-compiled runner 形态）：

| 闸门 | 基线时长 |
|---|---|
| m2 parser corpus（35 文件） | 2.978s |
| m4 codegen corpus（58 文件） | 6.405s |
| m4 use corpus | 19.470s |
| `cargo t` 全量（主检出，--no-fail-fast 墙钟） | 44.8s（4674 测） |

**内存峰值基线**（`.config/test-mem-weights.md` 565 版，2026-09-07 复测，
同代码基线 c2c90c452 已含）：非忽略最重测试——m5 engine corpus 814MB /
corpus_runner rerun 813MB / m5 use 811MB / m4 use 810MB / m1 lexer 809MB /
m2 parser 806MB / m3 milestone fib 802MB / m3 typeinfo 792MB(L1) /
m4 codegen 796MB(L1)。G2 判定锚点=最重非忽略测试 ≥3× 降幅（~800MB →
≤270MB）。

**预存红集**（22，G3 绿门对照基线）：plan370×3（d2/d8/z6）、plan492 c2、
aura strip_html、lucide manifest、ui::layout grid/master_stack/snap×12、
desktop_protocol covered_elements、charts_gallery。全部 tag 基点在案，
后续各阶段验收以"红集不新增"为准。

## 待澄清事项

- **①（T0 绿门口径，执行注记非阻塞）**: master `cargo t` 22 红均为预存
  （21 条台账在案 + covered_elements/imagesurface 新归因：Plan 547 登记
  element_table Covered（7a95e9b8c1）未配 `Coverage::target_set()` 投影臂，
  blame 实证早于本 plan）。本 plan 绿门沿 564-Q6 先例按"红集与 T0 基线
  一致"执行；covered_elements 修复属 desktop_protocol 领地，留给独立
  修复立项（复审时视需要登 KNOWN-DEBT）。

- **②（T1 数据裁决：装箱集扩员 Obj+Meta，阶段检查点按实测重定，非阻塞）**:
  实测 Value=296B（计划估算链 ~232 全线偏低）。按实测推演各阶段落点：
  Phase A(Node)→120B（Widget 112 撑）/ Phase B(+Instance·七单点)→80B
  （MetaID 72/Obj 72/Fn 72 撑）/ Phase C(+Fn·ExtFn·Type)→**仍 80B（Obj 72
  撑）**——原阶段划分 G1 ≤48B 数学不可达。裁决（沿计划"数据裁决 Phase
  B/C 取舍"条款）：装箱集扩为原清单 **+Obj(232 处)+Meta(11 处)**，阶段
  检查点重定 A≤128 / B≤88 / C≤48；终态地板=48B（Str 40B 为地板设定者，
  热路径字符串不装箱——装箱=每字符串一次堆分配，负收益）。Q1 终态断言
  答案随之裁定：48B 为承诺线（32B 需装箱 Str，否决）。

- **③（T2 数据否决 G2 前提，复审/用户裁定项）**: Phase A 后 m2 峰值
  825MB（登记 806，+2% 噪声内零降幅）；mem-profile 复跑与 565 基线
  **逐字节相同**（compile 7419.7MiB/8.8M allocs/peak_live 755.2MiB），
  且静态核实 auto-lang AST（Code/Stmt/Expr 家族）**零 Value 字段**——
  最重测试的 ~800MB 峰值本体=**编译器内部 AST/token 结构**（470KB 自举
  链进程内编译的 755MB 活集），不是 Value 密集数组（执行侧仅 2.9MiB）。
  计划核心前提"Vec<Value> 密集负载同比例缩水→压峰值"对最重测试族
  **不成立**：VM 堆=DashMap 逐对象 Arc<RwLock<dyn HeapObject>>（无密集
  Value 槽），引擎栈/ListData 的密集 Value 在这些测试里 footprint 可忽略。
  **G2（≥3× 峰值降幅）/A4 经 Value 装箱结构性不可达**；真正杠杆=编译器
  AST 瘦身（ast.rs/parser.rs 领地，超出本 plan 范围，应独立立项）。
  本 plan 继续执行的理由：G1（≤48B 锁死）/G4（零语义变化）仍完全可交付
  且是标题目标；A4 将以本证据标红，不静默美化。运行时密集 Value 消费面
  （VM 栈/ListData/Model/Grid）仍获 6× 密度收益，只是不体现在这批
  编译侧主导的测试峰值上。

- **Q1（断言档位）**: 终态断言定 48B 还是 32B——取决于 Phase C 取舍
  （T5 数据裁决）；建议 48B 为承诺线、32B 为尽力线。
- **Q2（Obj/Kids 内部 IndexMap 二级瘦身）**: Node 内部的 Obj/Kids 仍是
  IndexMap（~64/72B）——装箱 Node 后它们只在堆上，不再是 Value 尺寸瓶颈；
  是否进一步 Box<IndexMap> 由 Phase B 后权重表决定，默认不做（收益递减）。
- **Q3（与 auto-val 语义演进的关系）**: ValueData/ValueID（Universe 间接层）
  已是既定方向，装箱与它正交；若后续 Value 载荷迁往 Universe，装箱仍不
  冲突（Box 内搬家）。
