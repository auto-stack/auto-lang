---
plan_id: PLAN-599
status: drafting                # drafting → executing → execution_done → reviewed → archived
feature_name: a2r-lang-capabilities
author: [ZCode]
created_at: 2026-09-09
updated_at: 2026-09-09
plan_revision: 1
current_step: 0
total_steps: 8

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/trans rust(a2r), auto-lang parser]   # 详见 §5 规范增量
---

# [PLAN-599] a2r-lang-capabilities：语言能力四项——外来泛型字段 / 外来 trait impl / trait 对象 / 裸线程+阻塞 io

> **来源**：auto-term `docs/designs/004-engine-autoization-roadmap.md` §5④
> （能力缺口账 §2 表的四项；002 缺口 1.1 扩展）。
> **轨道声明**：本计划在 **Q3 轨**（a2r 转译表达力——Auto 源书写面，
> trans/rust.rs + parser）执行，与 **PLAN-596**（Q2 轨：430 dep 运行期
> 绑定管线的 trait/泛型/回调 shim 生成）概念相邻、码路无交：596 改
> shim-metadata/classify/marshaller，本计划改 parser 类型面 + a2r 发射。
> 两计划可并行，互不依赖。

## 0. 变更摘要

补齐引擎 Auto 化（004 P-L 相位）所需的四项 a2r 语言能力，每项
「语法裁定 → a2r 发射 → 快照 + rustc 实编」，终局以 **term.rs 子集
真身 round-trip** 收口（四能力同一语料全命中，行为对拍 Rust oracle）：

1. **外来泛型类型字段**：type 块/参数/局部可声明 `Term<ChannelListener>`
   形态（类型名经 `use.rs` 导入锚定，a2r 原样透传泛型应用）；
2. **外来 trait impl**：`ext ChannelListener for EventListener`——既有
   `ext X for Trait` 语法扩展接受外来 trait 名（emit
   `impl EventListener for ChannelListener`）；
3. **trait 对象**：字段/参数/局部可声明 `Box<dyn MasterPty + Send>`
   拼写（a2r 透传；自动解包/解引用语化不属本计划——那在 37 债轨道）；
4. **裸线程 + 阻塞 io**：reader 线程形态可表达（`std::thread` 透传 +
   move 闭包 vs a2r-std `spawn_blocking` 包装，T-01 spike 二选一）。

## 1. 目标

- **G1-G4**：四能力各自有独立语料（快照 + `rustc --edition 2024`
  独立实编通过并运行正确——006 S2 口径，不信快照文本）；
- **G5**：capstone——手写 Auto 版 TermSession 子集（ChannelListener +
  Dimensions impl + `Term<..>` 字段 + 快照方法），a2r 产物独立实编，
  与手写 Rust oracle 黑盒输出逐字节等价（S2 spike
  `auto-term spikes/autoize-roundtrip` 的四能力扩展版）。

### 非目标

- 不动 Q2 轨（dep/use.rust 运行期绑定管线，PLAN-596 领地）；
- 不做智能指针自动解包/解引用语化（a2r 遗留 37 债家族，另行销账）；
- 不迁移 pty.rs/term.rs 真身（004 P-S 相位事，本计划只让语言「能」）；
- 不做 004⑥（C 通道 a2r 后端）与⑤（cdylib 导出发射）。

## 2. 架构方案

统一策略：**外来名透传 + 最小语法面**。类型名/ trait 名已可经
`use.rs` 透传导入（rust.rs:21977-22027 原生 use + Cargo 依赖渲染），
未知类型名静默透传（002 F6）是既有事实通道——本计划将其**正式化**
（显式支持 + 未解析名告警面），并为三处语法位放行泛型应用/dyn 拼写：

| 能力 | 语法位 | a2r 发射 |
|---|---|---|
| 外来泛型字段 | type 块字段/参数/局部：`Name<Arg>` | 原样 `Name<Arg>` |
| 外来 trait impl | `ext Self for ForeignTrait`（方法块既有） | `impl ForeignTrait for Self` |
| trait 对象 | 类型位：`Box<dyn Trait + Send>` 等拼写 | 原样透传 |
| 裸线程 | T-01 裁定：A=闭包加 move 发射/`std::thread` 透传；B=a2r-std `task::spawn_blocking` 包装 | 按裁定 |

## 3. 技术栈

parser.rs（类型/ ext 语法面）、trans/rust.rs（发射）、a2r 快照设施
（`test/a2r/`）、rustc 实编门（S2 口照）、（若 T-01 选 B）a2r-std。

## 4. 需求分析与背景调查

- **授权**：用户 2026-09-09 指示就 004 §5③④「各自做出计划文件」——
  本计划为立项起草（drafting），**执行待后续 /auto-plan:work 指令**；
- 002 Q3 能力面在案：trait 定义/impl（自家）/泛型/模式匹配/闭包/actor
  全部可 emit；`use.rs` 透传原生链接外来 crate 通道存在；
- 002 缺口 1.1：type 块只认 Auto 类型——外来泛型字段不可声明（本计划
  能力①正面清偿）；F6：未知类型名静默透传（本计划正式化+告警）；
- S2 实证先例：`enum Damage` + GridSize + TermSession 子集 round-trip
  通过 rustc 实编并运行（spikes/autoize-roundtrip/NOTES.md）；
- 37 债家族中「智能指针自动解包族/ownership-闭包耦合族」与本计划
  能力③④相邻但不同物（债=已写 Auto 的编译失败；本计划=新语法面）。

## 5. 详细设计

### 各能力语料蓝图（快照目录拟名）

- `25_foreign_types/001_foreign_generic_field`：use.rs 导入 +
  `type Holder { term FakeTerm<MyListener> }` + 构造/读取；
- `25_foreign_types/002_foreign_trait_impl`：`ext MyListener for
  ForeignTrait { mut fn send_event(...) }`（方法块形态按既有 ext 发射
  规则，trait 名换透传名）；
- `25_foreign_types/003_trait_object`：`type Session { master
  Box<dyn ForeignMaster + Send> }` + 参数位；
- `25_foreign_types/004_bare_thread`：按 T-01 裁定形态，阻塞 read 用
  小文件 read 近似（真 PTY 读取留 capstone 用链接真 crate）；
- capstone（T-06）：TermSession 子集 Auto 源 + 手写 Rust oracle
  （fixture 对，黑盒断言 stdout 全等）。

### T-01（bounded）线程通道选型 spike

A（std::thread 透传 + move 闭包）：验证 a2r 闭包捕获发射可加 `move`、
外来路径 `std::thread::spawn` 调用可透传；B（a2r-std spawn_blocking
包装）：tokio 依赖面（a2r-std 已有 task 模块）。判据：产物是否零
运行时依赖（A 可零、B 必链 a2r-std）+ 与 VM 语义往返（B 近 VM actor）。
结论回填 §2 表。

### 规范增量

| delta_id | 操作 | docs/specs/... target | before/after rule | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | docs/a2r-transpiler-guide.md | Implementation Status 增四能力；类型映射表补外来名透传口径（含 F6 正式化与告警） | 002 缺口 1.1 清偿 | AC-01..04 |
| SD-02 | modify | auto-lang/trans/overview.md | a2r 能力面清单补：外来泛型字段/外来 trait impl/trait 对象拼写/裸线程通道（按 T-01 裁定） | 当前态记录 | AC-01..05 |
| SD-03 | modify | auto-lang/trans/design/*.md（按实际归属，执行时定位） | ext-for-外来-trait 与类型位 dyn 拼写的语法/发射规则 | 语法面正式化 | AC-02/03 |

## 6. 测试设计

- 每能力：a2r 快照（.expected.rs）+ **rustc 实编运行**（S2 口径：
  独立 rustc --edition 2024，非仅文本比对）；
- capstone：黑盒对拍（stdout 逐字节等价），oracle 为手写 Rust 真身
  子集；
- 回归护栏：a2r 快照 24 组全绿；cargo tt/tf 档基线两例固有失败不变。

## 7. 验收标准

- **AC-01**：外来泛型字段语料：快照绿 + 独立实编运行正确；
- **AC-02**：外来 trait impl 语料：同上，且 emit 形态为
  `impl ForeignTrait for Self`（快照断言）；
- **AC-03**：trait 对象语料：同上（Box<dyn ..> 拼写原样出现在产物）；
- **AC-04**：裸线程语料：同上（通道形态按 T-01 裁定记录在案）；
- **AC-05**：capstone：TermSession 子集 Auto 源 → a2r → rustc 实编 →
  与 Rust oracle 黑盒输出全等；四能力在该语料全命中（逐项指出）；
- **AC-06**：a2r 快照 24 组 + tt/tf 档零回归（基线两例不变）；guide
  与 004 §5④ 回执落账。

## 8. 执行步骤

- [ ] **T-01**（bounded investigation）线程通道选型 spike（A/B 判据
  §5），结论回填 §2 表与 SD-02；
- [ ] **T-02** 能力①外来泛型类型字段：parser 放行 + a2r 透传 +
  未解析名告警面 + 语料（快照+实编）；
- [ ] **T-03** 能力②外来 trait impl：ext-for 语法面 + 发射 + 语料；
- [ ] **T-04** 能力③trait 对象拼写：类型位放行 + 透传 + 语料；
- [ ] **T-05** 能力④裸线程+阻塞 io：按 T-01 落地 + 语料；
- [ ] **T-06** capstone：term.rs 子集 Auto 版 + Rust oracle + 黑盒
  对拍 runner（语料入 test/a2r/，oracle 入 fixture）；
- [ ] **T-07** guide/overview 落稿 + 004 §5④ 回执 + DEBTS #10 增 599 条；
- [ ] **T-08** 收口门禁：a2r 快照全量 + cargo tt + cargo tf（基线两例
  不变）+ bindgen/a2c 套件不回归。

依赖：T-02..T-04 相互独立可并行；T-05←T-01；T-06←T-02/03/04（+05 若
  子集含线程；TermSession 子集默认不含线程，线程命中走 004 语料）；
  T-07/08 收口。

## 9. 复审记录

stage: new | PLAN-599 | rev 1 | outcome: pass | next: work
（起草即绪：能力面/语法位/透传通道均经 002 与 rust.rs 实码核位；
与 PLAN-596 边界显式；执行授权待用户指令。）

## 10. 待澄清事项

- T-01 线程通道 A/B：授权执行时按 §5 判据自裁并记录（倾向 A——零
  运行时依赖与引擎场景更配，但以 spike 实证为准）；
- capstone 是否含线程命中：默认不含（TermSession 子集无线程面），
  若用户要求四能力全在 capstone 命中，扩展子集含 reader 线程桩——
  执行时按成本自裁记录。
