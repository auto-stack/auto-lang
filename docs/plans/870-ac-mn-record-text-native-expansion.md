---
plan_id: PLAN-870
status: drafting
feature_name: AC 共享 MIR native：record/text 扩面（关 394/462/484 受控拒绝）
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-mir/native-data-owned-lowering.md
touched_goals: [GOAL-002]

affects: [auto-mir, auto-av, auto-ac, auto-test-case]
depends_on: [PLAN-868]
current_step: 0
total_steps: 9
---

# [PLAN-870] AC 共享 MIR native：record/text 扩面

## 0. 变更摘要

共享 MIR 的 native 后端（`ac-mir-native`，简称 mn）目前**只有 scalar**：`lower.rs` 对 data/owned/collections/closures/dynamic 五族指令是**显式受控拒绝**（"outside the mn scalar surface"），且常量域只接受 `I32`/`Bool`。这使四路径语料里 `ac` 列在 T*/R* 族一律 `gap`，M1 无法收口。

本Plan把 mn 扩到 **record / text / text-record** 三域（对应既有 820 `av.data/1` 与 821 `av.owned/1` 的已冻结语义），关闭三处受控拒绝：

| 行 | 现状拒绝 | 本Plan处置 |
|---|---|---|
| `lower.rs:394` | 任何非 `I32`/`Bool` 常量（`Char`/`Str`/宽标量） | **选择性关闭**：接受 `Char` 与 `Str`；宽数值标量（`I64`/`U64`/`F32Bits`/`F64Bits`）保持受控拒绝（归 R2-N） |
| `lower.rs:462` | `av.data/1` record 指令 | 实现 record 布局/字段/构造/克隆 |
| `lower.rs:484` | `av.owned/1` owned 指令 | 实现 `Str`/拥有字段/借用与 mut 写回/全出口清理 |

不关闭：`509`（collections）、`517`（closures）、`526`（dynamic）——按用户 2026-10-28 裁定，迁移批次**不含** collections/closures/dynamic，它们要等 AV 源码面闭合后另立。

**覆盖边界的硬约束**：差分门以 **AV 为语义基准**，因此本Plan只覆盖 AV 已交付的 820/821 面。AV 自己仍 `gap` 的项（如 R08 `gap:av-lower-method`：实例方法 lowering）**不在本Plan**，否则没有可比对的基准。

## 1. 目标

- mn 能 lowering 并执行 `av.data/1` 的 record 子集与 `av.owned/1` 的 text/text-record 子集，产物经真实 `MIR→CLIF→COFF→PE→contained run` 得到与 AV 一致的值/exit/trap/资源台账。
- 三路差分常驻：**旧 ac-core HIR→CLIF（参照）/ 新 mn（候选）/ AV（基准）**，同一 source、同一 profile、同一冻结 oracle。
- 四路径语料的 T*/R* 行在 `ac` 列由 `gap:mn-scalar-only` 变为实测结果（或按合同具名拒绝），`binding.tsv` 状态列如实更新。
- 现有 scalar 面零退化：B* 族全部保持，mn 的增量/单元 cache/预算行为不变。

**非目标**：宽标量**数值算术**与宽常量（R2-N）；collections/闭包/Dyn；native typed entry（W01–W05）；record 实例方法 lowering（AV 未交付，见上）；性能目标（不承诺与旧路径同速）；产品/CLI 默认切换（属 871）；增量发布；CTE/native 复合。

## 2. 架构方案

```text
source → ac-core(HIR) → auto-mir-lower(868) → Typed MIR
                                                 │
                        ┌────────────────────────┴──────────────┐
                        ▼                                       ▼
              ac-mn: lower.rs / emit.rs                 auto-av-core: AV 解释
              （本Plan扩展：record/text）                （语义基准，不改）
                        │                                       │
                        └──────────► 三路差分（父侧判定） ◀──────┘
                                         ▲
                              旧 ac-core native.rs（参照，871 退役）
```

扩展点（都在 `experimental/ac-mir-native/src/`）：

| 面 | 责任 |
|---|---|
| 类型/布局（`lower.rs`） | MIR `Record`/`Char`/`Str`/借用视图 → CLIF 类型；字段顺序与 owner 名义身份按 Design44 |
| Place/字段（`lower.rs`） | 字段读/写、重叠赋值先物化 RHS、`FieldMove` 逐字段语义 |
| 调用/返回（`lower.rs`） | 聚合参数/返回、借用参数与 mut 写回、拥有返回值 |
| 清理（`resources.rs`/`lower.rs`） | 全出口 drop、迭代式 worklist（无宿主递归）、分配/释放台账 |
| text 操作（`lower.rs`） | `len`/`len_bytes`/`char_at`/`slice`/`concat`/`eq`/`ne`（按 Unicode 标量单位，沿 770/821） |
| 常量（`lower.rs`） | `Char`（Unicode 标量域校验）、`Str`（UTF-8 bytes 拥有/借用形态） |
| 链接/单元（`link_driver.rs`/`incremental.rs`） | 复用 818 的 per-function COFF 与单元 cache，不新增打包路径 |

## 3. 技术栈

- mn：`experimental/ac-mir-native/src/{lower,emit,driver,resources,incremental,link_driver,worker}.rs`。
- 语义来源：`docs/specs/auto-mir/data-place-profile.md`、`owned-access-profile.md`、Design44、`docs/specs/auto-av/{data-layout,place-ownership}.md`。
- 差分执行：`experimental/ac-case-worker`（协议侧 driver，可同时驱动 ac-core 旧路径与 mn）+ 既有 `av_probe_worker`。
- 语料/判定：`crates/auto-lang/test/cases/ace-m1/`（T*/R* 源与 binding）、`av-data/821`（owned 正控）、`crates/auto-lang/src/tests/ace_m1_tests.rs`。
- 拟建 `scripts/verify-av-870.ps1`（Scope=differential/targeted/compat/acceptance），**当前不存在、未运行**。

唯一实施树：`D:/autostack/.wt/lang-870/auto-lang`，分支 `plan-870-dev`。

## 4. 需求分析与背景调查

用户 2026-10-28 决定：把旧 HIR 直达迁移到 HIR→MIR，使 MIR→Native 共享；迁移批次**不含** collections/closures/dynamic；先出 lowering 共享（868），再做本扩面，再做切换/退役。

已核实事实：

| 事实 | 位置 |
|---|---|
| mn 的受控拒绝（6 处，本Plan关 3 处） | `experimental/ac-mir-native/src/lower.rs:394/462/484/509/517/526` |
| 常量只接受 I32/Bool | `lower.rs` `MirInstr::Const` 分支；MIR 常量全集在 `ac-av-probe/core/src/mir.rs`（含 `Char`/`Str`/宽标量） |
| record/owned 的语义已冻结 | [data-place-profile](../specs/auto-mir/data-place-profile.md)、[owned-access-profile](../specs/auto-mir/owned-access-profile.md)、820/821 交付 |
| AV 已能执行 record/text-record 源码 | `probe/src/{data_driver,owned_driver}.rs`（`compile_record_source` / `compile_text_record_source`） |
| 旧 native 已有同语义实现（参照） | `experimental/ac-core/src/native.rs`（record/text lowering）、`text_resources.rs` |
| 覆盖边界受 AV 限制 | `binding.tsv` R08 = `gap:av-lower-method`（AV 无实例方法 lowering） |

## 5. 详细设计

### 5.1 前置、写面与顺序

- 硬前置：`D868`（lowering 单源 + 版本轴）；`D820`/`D821`（语义已冻结，已具备）。
- 写面：`experimental/ac-mir-native/src/**`；`experimental/ac-case-worker/**`（差分驱动，若需）；`crates/auto-lang/test/cases/ace-m1/binding.tsv`（状态列）；语料仅**追加**；不改 `ac-core`、不改 `auto-av-core`、不改 recipe/schema。
- 顺序：scalar 差分保持绿（回归基线）→ record → text-record（text 作为其子集同批）→ 语料状态更新。

### 5.2 类型与布局合同

按 Design44 投影，冻结在 T-02：

- record：字段按声明序；名义身份 `(owner, name, fields)`；同宽度不同名不互换（与 820 的类型相等性一致）。
- `Char`：Unicode 标量（u32 语义），域校验与 770 一致。
- `Str`（拥有）：拥有缓冲 + 长度；`str`（借用）：视图 + 生命周期不逃逸；`Str` 字段 record 为资源 record（派生属性，不信任输入自称）。
- 借用 record 参数（`RecordView`）与 mut 写回：按引用传递 + 写回后的一致性；不承诺运行期句柄校验（沿 821 静态边界）。
- `drop` 顺序：与 AV 台账逐项对齐（alloc/free 序列），不是"最终都释放"即可。

### 5.3 指令覆盖清单（T-01 逐条列全）

必须覆盖（否则语料必红，不能留静默空实现）：`Const`（Char/Str）、`Copy`、record 构造、字段 Get/Set、`FieldMove`、Move/Clone、Drop、`Index`?（否——索引属 collections，本Plan排除）、调用/返回（聚合与借用形态）、text 操作族、trap 出口。T-02 冻结"支持集合"与"具名拒绝集合"，两侧都不能靠默认分支。

### 5.4 清理与资源台账

- 迭代式 worklist（不得挂宿主栈）；全出口（正常/trap/取消）清理。
- 分配/释放/存活计数按 AV 口径上报，供差分门比对。
- 预算：沿用 818/825 的预算账本，不新增第三套计量。

### 5.5 三路差分门

| 项 | 要求 |
|---|---|
| 输入 | 同一 `.at` 源、同一 profile、同一冻结 expected（来自 852/ACE-M1 或独立手审） |
| 路径 | 旧 `ac-core` native（参照）／新 `mn`（候选）／AV（基准） |
| 观察面 | 值（exit 或 typed 结果）、trap kind+code、诊断 `stage/code`、资源台账、超预算/取消 |
| 反例 | 坏 MIR、坏 ABC、超预算、未授权、`Str` 借用逃逸（须具名拒绝而非崩溃） |
| 通过 | 三路在**所有**观察面一致；任一面缺失即红（不得只比进程退出码） |
| 记录 | 每 profile 一份差分报告；旧路径的差异若属旧实现缺陷，登记为决策项，**不得**以"旧路径就这样"放行 |

### 5.6 测试框架接入（既有 TestCase 协议，零 schema 改动）

- recipe 不变：本Plan只让 `ac/mir-native-scalar-v1` 的实现在 T/R 源上真正能跑；若需区分 profile，按既有 av binding 追加字段（append-only，旧 case 逐字节不变）。
- 语料：T*/R* 行的 `ac` 列由 `gap:*` 改为实测；`av`/`avm`/`a2r` 列不动（`diff-*` 如实保留）。
- `binding.tsv`：只改状态/备注列，12 列 schema 不变；852 base suite 不扩缩。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-mir/native-data-owned-lowering.md | 无 → mn 的 record/text lowering 合同：布局/指令覆盖/清理/拒绝面 | 共享 MIR 的第二个 consumer 落地 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-mir/data-place-profile.md | 追加 native consumer 实际覆盖与拒绝面，语义不变 | 事实纠正 | AC-02 |
| SD-03 | modify | docs/specs/auto-mir/owned-access-profile.md | 同上（text-record 资源面） | 事实纠正 | AC-03 |
| SD-04 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | T*/R* 行 `ac` 列实测映射 | 证据落点 | AC-05 |

## 6. 测试设计

| V | AC/不变量 | 实际/拟建入口 | 可区分错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 record 值 | R01–R07 源三路差分；构造/复制/嵌套/重叠赋值 | 三路值一致；字段顺序错/复制语义错必红 | T-03 |
| V-02 | AC-02 借用与写回 | 借用 record 参数、mut 写回、move 后读取（R09/R10 诊断） | 值 + 诊断一致；把借用按复制实现必红 | T-03 |
| V-03 | AC-03 text/清理 | T01–T08 源；UTF-8 标量索引/切片/concat；alloc/free 台账 | 台账逐项一致；按字节当标点/漏释放必红 | T-04 |
| V-04 | AC-04 拒绝面 | 宽标量常量/collections/闭包/Dyn 源 → 具名拒绝 | 拒绝码正确、零副作用；静默空实现必红 | T-04 |
| V-05 | AC-05 语料与状态 | binding 状态更新 + 852 suite + 独立 oracle | 状态与实际一致；SKIP/未跑不计绿 | T-05 |
| V-06 | AC-06 反控 | 注入错布局/漏 drop/错 trap 码 | 差分门必红；恢复后必绿 | T-05 |

定向命令：`cargo check --locked`（ac-mn）+ 定向 `cargo test`；`verify-av-870.ps1 -Scope differential`；完整档归 merge。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | record 子集三路一致（值/诊断）；覆盖集合与 T-02 冻结清单相同；V-01 |
| AC-02 | work | 借用/写回/move 语义与 AV 一致；V-02 |
| AC-03 | work | text 操作与资源台账一致，无宿主递归；V-03 |
| AC-04 | work | 明确拒绝面（宽标量/collections/闭包/Dyn）具名拒绝且零副作用；V-04 |
| AC-05 | work | 语料状态如实更新，独立 oracle，SKIP 不计绿；V-05 |
| AC-06 | work | 反控可证差分门有判别力；V-06 |
| AC-07 | review | 独立核 AC/V/SD、指令覆盖无遗漏/无静默空实现、scalar 零退化、warnings/fmt |
| AC-08 | merge | 冻结提交裸 `cargo t` + `tv` 通过；失败不豁免 |
| AC-09 | merge | SD-01..04 沉淀、ledger、归档、guard；并交接 871 的切换前置 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | D868 | 列全 MIR 指令与类型在 mn 的现状（逐 op 分类：已支持/本Plan/拒绝），产出 `attachments/870/work/coverage.md` | AC-01/04 | [ ] |
| T-02 | work | T-01 | 冻结布局/指令覆盖/拒绝面/清理与台账口径/差分观察面 | AC-01..05 | [ ] |
| T-03 | work | T-02 | record lowering（布局/字段/构造/克隆/借用/写回）；V-01/02 | AC-01/02 | [ ] |
| T-04 | work | T-03 | text-record lowering（Char/Str 常量、text 操作、拥有字段、全出口 drop）；V-03/04 | AC-03/04 | [ ] |
| T-05 | work | T-04 | 三路差分运行 + 语料状态更新 + 反控 + 脚本；V-05/06 | AC-05/06 | [ ] |
| T-06 | work | T-05 | scalar 回归与预算确认（零退化），提交代码/证据 | AC-03 | [ ] |
| T-07 | review | T-06 | 独立最终review；merge 门 pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交裸 `cargo t` + `tv` | AC-08 | [ ] |
| T-09 | merge | T-08 | Spec/ledger/归档/guard；交接 871 | AC-09 | [ ] |

## 9. 复审记录

2026-10-28 new r1：`stage: new | plan_id: PLAN-870 | plan_revision: 1 | outcome: pass（等 D868，合同可预检） | next: D868 后 work T-01`。全部 AC/V/T 未执行；三处拒绝行的关闭范围、宽标量是否顺带、指令覆盖清单由 T-01/T-02 冻结。本轮未建树、未跑构建、未改任何实现。

## 10. 待澄清事项

- 宽数值标量常量（`I64`/`U64`/`F32Bits`/`F64Bits`）是否顺带关闭：默认**不**（归 R2-N 数值族），但若 T-01 发现 record/text 源在合法输入下会产生宽常量，须在 T-02 显式扩范围并说明原因。
- record 实例方法（`R08`）：AV 目前 `gap:av-lower-method`，本Plan按"以 AV 为基准"原则**不做**；若用户要求 mn 先支持，则失去差分基准，需先补 AV 侧能力（另立计划）。
- `text` 单域（非 text-record）是否需要独立 profile 判定：默认经 text-record 通道（超集）验证，若发现支持集合差异，T-02 再拆。
