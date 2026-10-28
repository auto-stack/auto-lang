---
plan_id: PLAN-870
status: archived
feature_name: AC 共享 MIR native：record/text 扩面（关 394/462/484 受控拒绝）
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 2

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-mir/data-place-profile.md
  - docs/specs/auto-mir/owned-access-profile.md
  - docs/specs/auto-test-case/ace-m1-conformance.md
new_spec_components:
  - docs/specs/auto-mir/native-data-owned-lowering.md
touched_goals: [GOAL-002]

affects: [auto-mir, auto-av, auto-ac, auto-test-case]
depends_on: [PLAN-868]
current_step: 9
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

**非目标**：宽标量**数值算术**与宽常量（R2-N）；collections/闭包/Dyn；native typed entry（W01–W05）；record 实例方法 lowering（AV 未交付，见上）；吞吐优化（不承诺与旧路径同速，既有周转/资源门仍须验收）；产品/CLI 默认切换（属 871）；增量发布；CTE/native 复合。

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

本次授权与输入：用户明确要求修订波次/实际写面、补871旧能力覆盖及删除门、让864支持List与拥有型type元素；允许动作仅计划/设计/战略文档修订，不实施代码、不调整预算、不改868。基面路径、内容SHA-256、旧稿快照及静态预检见[修订收据](attachments/ace-wave-20261028/revision-summary.md)；观察到的current Spec/调用点与未来拟建入口分开记录。

## 5. 详细设计

### 5.0 波次、实际写面与交接（本次修订）

按[波次交接合同](reports/ace-wave-handoff-2026-10-28.md)§1–2执行：S为共享source/HIR/lowering/MIR/ABC/AV及会话接口，N为ac-mn，H为公共driver/SDK适配/测试注册与binding；分别单写，跨窗同时取窗。共享lowering的未来写面为D868后的`experimental/auto-mir-lower/src/**`，旧`probe/src/hir_adapter.rs`仅作搬前调查，不能再落新实现。T-02及接线前核验实际已合入revision、Spec/API/版本、owner释放与消费者兼容证据并记录接受；review/repair未完不交WIP。

第1波D868后T-01..04只占N，与862的S实现并行；T-05公共差分驱动、SDK/recipe适配及binding占H，须等D862和H释放，刷新共同MIR/profile/报告基面，默认D862→D870交付。N范围含cache/link/worker，绝非整个计划“只ac-mn”：公共消费者接线需登记H。若862新增公开枚举确需mn兼容匹配臂，870登记暂停N，向862短交接N与共同基面，最小适配交付后再接受N恢复，不在同一N写面并改。871须等本Plan交付及覆盖门，不能拿本Plan的AV子集覆盖冒领旧方法能力。

首例即按既有文件化AutoDown/TestCase、cache fresh/hit/restart/body-edit、未改函数生产工作=0与现行P90≤2000ms口径验证；亚秒目标及P95/P99另列，不能伪称当前硬门已是亚秒。内存/资源峰值、live/回收及cache容量在T-02沿上游预算交集冻结，记录命令、样本和反控，不擅自提高上限。共享核心/SDK/测试迁移先确认拆分owner实际位置与释放；产品默认入口、旧AutoVM/a2r和868合同/进度不在本次修订写面。

### 5.1 前置、写面与顺序

- T-01..04硬前置：D868与既有D820/D821；T-05公共驱动/binding接线另等D862及H释放。D862不阻塞N后端主体开工，不能简化为整项先等862。
- 写面：`experimental/ac-mir-native/src/**`；`experimental/ac-case-worker/**`（差分驱动，若需）；`crates/auto-lang/test/cases/ace-m1/binding.tsv`（状态列）；语料仅**追加**；不改 `ac-core`、不改 `auto-av-core`、不改 recipe/schema。
- 顺序：scalar基线→record→text/text-record（可共用执行设施，但各自profile身份与合法/拒绝域独立验证）→D862后的公共差分/语料状态接线。D870不等于871旧能力覆盖门已通过。

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
| V-07 | AC-03/AC-07 写面/兼容 | T-02及公共接线交接摘要；已交付消费者并集与窗口清单 | 真实D门/释放/接受齐全；缺交接或消费WIP不准接线；受影响profile兼容验证及首例cache/周转/内存证据齐全 | T-02冻结；最终work任务执行；review复核 |

定向命令：`cargo check --locked`（ac-mn）+ 定向 `cargo test`；`verify-av-870.ps1 -Scope differential`；完整档归 merge。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | record 子集三路一致（值/诊断）；覆盖集合与 T-02 冻结清单相同；V-01 |
| AC-02 | work | 借用/写回/move 语义与 AV 一致；V-02 |
| AC-03 | work | text 操作与资源台账一致，无宿主递归；V-03 ；首例cache/文件化测试、周转/内存预算及兼容证据齐全，V-07 |
| AC-04 | work | 明确拒绝面（宽标量/collections/闭包/Dyn）具名拒绝且零副作用；V-04 |
| AC-05 | work | 语料状态如实更新，独立 oracle，SKIP 不计绿；V-05 |
| AC-06 | work | 反控可证差分门有判别力；V-06 |
| AC-07 | review | 独立核 AC/V/SD、指令覆盖无遗漏/无静默空实现、scalar 零退化、warnings/fmt；核V-07交接与首例指标 |
| AC-08 | merge | 冻结提交裸 `cargo t` + `tv` 通过；失败不豁免 |
| AC-09 | merge | SD-01..04 沉淀、ledger、归档、guard；并交接 871 的切换前置 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | D868 | 列全 MIR 指令与类型在 mn 的现状（逐 op 分类：已支持/本Plan/拒绝），产出 `attachments/870/work/coverage.md`（含 18 行 T*/R* 语料实测分类、上游 S 缺口与 9 项登记） | AC-01/04 | [x] |
| T-02 | work | T-01 | 冻结布局/指令覆盖/拒绝面/清理与台账口径/差分观察面；冻结fresh/hit/restart/body-edit、周转/内存口径与反控 → `attachments/870/work/freeze.md`（2026-10-28 work 裁定三级台账口径并入） | AC-01..05 | [x] |
| T-03 | work | T-02 | record lowering（布局/字段/构造/克隆/借用/写回）：`ac-mir-native/src/layout.rs`（新）+ `lower.rs`/`emit.rs`/`fingerprint.rs` 扩面 + driver 前端升级（scalar→data→owned 最窄域）；V-01/02 由 `tests/data_owned_tier.rs` 三路差分覆盖（R01/R02/R03/R06 值+R04/R05 资源面全绿；R1 后另含 mut 回写三路） | AC-01/02 | [x]（R1 补面后复核关闭） |
| T-04 | work | T-03 | text-record lowering（Char/Str 常量、text 操作、拥有字段、全出口 drop）：shim `ac_text_runtime` 接入（driver 自动定位/链接/manifest 条件轴）、入口零化+trap 臂 sweep+last-owning-write 降序清理调度；V-03/04：T01/T02/T04/T08 值/trap + 台账三级口径 + 拒面负控（宽常量/collections/entry 形状，codegen=0）全绿 | AC-03/04 | [x]（R1 补面后复核关闭） |
| T-05 | work | T-04、D862、H释放/接受 | D862 于本会话期间交付（862 归档 2012b33dc）→ 基面刷新（merge afcd4d09e + MirType::Unit 兼容臂）；H 接线：`t-r-native.case.ad`（17 case/31 check，值/trap 行双腿=mn+av、诊断行 mn 腿+av 腿按 862 绑定）+ 独立 suite `ace-m1-tr-native.suite.ad`（852 base 不扩）+ `ace_m1_tr_native_suite_green`/negative oracle 测试 + binding.tsv ac 列 18 行判级 + `ac_mn.rs` 诊断面映射（probe 词表内 typecheck 面）；V-05 实测+独立 oracle+无 coverage gap ✓，V-06 变异必红（T01 38→39）✓ | AC-05/06 | [x] |
| T-06 | work | T-05 | 零退化：852 base suite 绿 + ac-mn 全 tier 绿（两项预登记 flake 归因：Windows 计时器采样红 + rounds 存活负载 flake——隔离重跑 2×绿）；首例文件化=tr-native suite（TestCase/case.ad 驱动真 worker）；cache fresh/hit/restart/body-edit/跨进程恢复=cache_tier 8 测绿；周转 turnaround_tier 3 测绿；内存=ManagedLedger 分区门（resources_tier）+manifest 条件轴保持 scalar 逐字节兼容；提交 3e4d520f9 + afcd4d09e + 5ae22bf3a | AC-03 | [x] |
| T-07 | review | T-06 | 独立最终review R1 needs_fix（F-870-R1-01/02）→ 修复 042022df3 → repair re-review pass（限制：同会话复审，见 §9 记录） | AC-07 | [x] |
| T-08 | merge | T-07 pass | 冻结基面（rebase 后代码提交 range-diff 全等价，唯一内容差=派生 ledger 重投影非测试输入——证据复用理由记录在案）：裸 `cargo t` 全 face **3201/3201 passed**（391.3s，--no-fail-fast 完整面；前置两次 fail-fast 截断均为调度竞争瞬态：4×120s 超时与 rounds P3 flake，隔离复跑绿）+ `cargo tv` **163/163**（4.0s） | AC-08 | [x] |
| T-09 | merge | T-08 | SD-01..04 沉淀（native-data-owned-lowering 新合同+两 profile native 消费节+ace-m1 T/R 绑定节）；ledger 守卫式离线投影 v120→v121（4 条目，回读校验+作用域守卫）；ff-only 落地 bc90ef718（rebase range-diff 证明：代码提交 `=` 等价，ledger 提交 a4720d7a4→bc90ef718 因基面重投影）；归档+guard 清理收据见下 | AC-09 | [x] |

## 9. 复审记录

`stage: merge | plan_id: PLAN-870 | plan_revision: 2 | outcome: pass | delivery_commit: bc90ef718（master 尖=交付提交，ff-only 无合并提交；rebase 老→新映射：3e4d520f9→c873b3d21、5ae22bf3a→20ef8bd4a、042022df3→27154894a、edb6e794a→f9fdfcadd 均 range-diff `=` 等价；a4720d7a4→bc90ef718 为 ledger 冲突按守卫路径在 master v120 基面重投影） | canonical_specs: docs/specs/auto-mir/native-data-owned-lowering.md（新）、data-place-profile.md/owned-access-profile.md/overview.md（native 消费节+索引）、docs/specs/auto-test-case/ace-m1-conformance.md（T/R ac 绑定节） | ledger: .autoos/specs.json v121（designs P870-DESIGN-NATIVE-DATA-OWNED-LOWERING / tests P870-TESTS / reviews P870-REVIEW-R1R2 / reports P870-MERGE；通道=守卫式离线投影，无 store writer 可达——HTTP 探测空；备份 .autoos/specs.json.bak-p870-merge，回读+作用域断言通过） | gate: cargo t 3201/3201 + tv 163/163（冻结基面；复用理由：rebase 代码提交全等价+master 增量纯 docs/ledger） | checkpoints: prepared ✓（reviewed 042022df3+spec 冻结）→ landed ✓（bc90ef718）→ ledger_refreshed ✓（v121）→ archived ✓（docs/plans/archive/870-…）→ cleaned ✓（guard clean，树/分支/组回收收据见 git log） | 871 交接: mn 已交付 av.data/1+av.owned/1 record/text 面；871 的 G-COV 覆盖门须按其合同核对旧五 profile 的源码/诊断/CLI/产物身份/cache 消费者——注意本计划登记的三项：视图 eq 源级缺口（gap:upstream-s-eq）、split_units 类型索引未重映射、AV clone 行释放序口径；旧 lower 能力对等用例（含 R07 mut/R08 method）在 binding ac 列的 gap 行如实保留 | production_artifacts: auto-lang 无常驻服务部署面；experimental CLI 二进制按需重建，无 stale 常驻进程（观察项：无需操作）`

`stage: review | review_scope: final (repair re-review) | reviewer_identity/limitations: 实现同会话复审（限制同 R1；仅重验受影响项+复述不变证据） | plan_id: PLAN-870 | plan_revision: 2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-870/auto-lang | branch: plan-870-dev | reviewed_commit: 042022df3（修复提交；src 零改动、纯测试新增，R1 证据不受影响） | base_commit: 9e847acf7 | dependency_revisions: 同 R1 | spec_inputs: 同 R1 | acceptance_results: F-870-R1-01 CLOSED（mn_owned_mut_writethrough_wrapper_three_paths：AV Success(15)/旧 native 15/mn 15 + 台账恰一次 alloc[1B]/free[1]——MutRet 包装回写机器面三路实跑，手算独立 oracle）；F-870-R1-02 CLOSED（mn_owned_strcmp_and_codepoint_machine_faces：eq(A,B)=0/ne(A,B)=1/eq(A,A)=1/码点 -1→exit70，真 emit+link+run）；AC-01..06 全 pass（R1 证据+本轮补面）；AC-07 本记录闭合 | pending_closeout_gates: merge-owned——冻结提交裸 `cargo t`（根工作区，含 ace_m1_tr_native×2/ace_m1_base/ac_mn_probe 等 H 面测试）+ 触面档：VM/编译器→`cargo tv`（语料 golden 面；本计划未改 ac-av-core/共享 lowering 语义，tv 为合同要求的确认面）；`cargo tf` 批量回归归 /auto-plan:regress 到期判定 | findings: 无（R1 两 findings 已闭合并复核） | evidence: cargo test --test data_owned_tier → 6 passed（含原 4+修复 2）；cargo fmt --check 0；cargo check --all-targets 0 警告；worktree clean @042022df3 | next: merge（T-08 裸 t+tv → T-09 沉淀/归档/交接 871）`

`stage: review | review_scope: final | reviewer_identity/limitations: 实现同会话复审（无独立上下文可用；结论自提交/测试/文档工件重构，非执行者叙述） | plan_id: PLAN-870 | plan_revision: 2 | outcome: needs_fix（2 findings，均为同类：已实现机器面缺运行时 oracle；修复面小且定向） | worktree_path: D:/autostack/.wt/lang-870/auto-lang | branch: plan-870-dev | reviewed_commit: 5ae22bf3a | base_commit: 9e847acf7（master，经 afcd4d09e 合入 D862/867/878；master 后续漂移仅 docs 簿记 862/875 收尾，代码面零影响） | dependency_revisions: D868 共享 lowering；D862（862 归档 2012b33dc，MirType::Unit 已接兼容臂）；ac_text_runtime shim 经 probe 定位 | spec_inputs: docs/specs/auto-mir/{data-place-profile,owned-access-profile}.md、docs/specs/auto-test-case/ace-m1-conformance.md（拟变更目标）| acceptance_results: AC-01 pass（差分层 R01-R03/R06/R04/R05 三路值+台账）/AC-02 partial（F-870-R1-01）/AC-03 pass（text 族+台账三级+无宿主递归+首例指标）/AC-04 pass（拒面负控 codegen=0）/AC-05 pass（binding 判级+独立 oracle+coverage_gaps 空+负 oracle 红）/AC-06 pass（变异必红+静态反控+双释放断言）/AC-07 本记录 | pending_closeout_gates: merge-owned 裸 cargo t + 触面档（tv 类比：本计划仅触 experimental/ac-mn 独立工作区+H 面 ace_m1/ac_mn/ace-m1 语料；根 t 面含 ace_m1/ac_mn_probe 测试） | findings: F-870-R1-01（P2，AC-02/V-02）：MutRet 包装写回机器面（FieldMove 逐字段回写+写穿零化）已实现但零运行时验证——语料 R07 为共享栈拒绝行（gap，无 AV 基准），冻结面 §7 支持集含 mut 写回；最小修复=差分层加非语料 mut-资源-record 源（bump(Note) 型，手算期望 15 + 台账）三路/双路实跑；F-870-R1-02（P2，AC-04/V-03 覆盖边界）：StrCmp eq/ne 与 CharFromCodepoint 非法码点→trap70 机器面已实现但零运行时验证（唯一源级行 T05 被 upstream-s-eq 阻塞；KNOWN-DEBT 中"由三路差分层覆盖"表述不实——已更正为待补）；最小修复=手写 MIR 单元直发 emit+link+run（静态视图 eq/ne 出口 0/1、非法码点 exit70） | evidence: 差分层 cargo test --test data_owned_tier 4/4（提交 5ae22bf3a 树）；tr-native suite 2 测绿+负 oracle 红；852 base suite 绿；ac-mn 各 tier 绿（两预登记 flake 归因：计时器环境红+rounds 负载 flake 隔离 2×绿）；cargo fmt --check 0；cargo check --all-targets 0 警告；worktree clean @5ae22bf3a | next: work 同树定向修复（仅两测试文件级改动），复审仅重验受影响项`

`stage: work | plan_id: PLAN-870 | plan_revision: 2 | outcome: pass（T-01..T-04 完成；T-05/T-06 阻塞 D862/H 释放，精确解阻动作见下） | worktree_path: D:/autostack/.wt/lang-870/auto-lang | branch: plan-870-dev | base_commit: 9e847acf7 | code_commit: 3e4d520f9（plan-870-dev） | dependency_revisions: D868（共享 lowering experimental/auto-mir-lower，master 交付）；ac_text_runtime shim 经 auto_av_probe::native_reference::text_runtime_lib() 现场构建 | task_ids: T-01,T-02,T-03,T-04 | evidence: ①cargo test -p ac-mn --test data_owned_tier → 4 passed（10 可执行行 × AV/旧native/mn 三路：T01=38/T02=27/T04=128512/T08=exit70+StrIndex/R01=19/R02=9/R03=23/R06=21/R04=51/R05=42；台账三级口径：alloc bytes 序精确、free 多重集+单次性、指令点释放精确；诊断行 T03=verify.resource-assign/T06=borrow-escape/T07=text.invalid-scalar/R10=use-after-move 与 AV 同栈同码，codegen=0）②拒面负控：宽 I64 常量→mn.unsupported.const、av.collections/1→mn.unsupported.profile、非零参 i32 入口→entry 具名拒绝，均 codegen 0 ③零退化：first_case 3/codegen 6/parity 2/closures 1/cache 8/transaction 6/incremental 5/turnaround 3 全绿；resources_tier 4/5——唯一红 `transient_spike_sampler_and_raw_gate` 环境归因关闭（Windows 计时器分辨率 ~15.6ms 使 5ms 轮询得 4 tick<5；该测零触及本 diff，详见 KNOWN-DEBT Plan 870 节） ④布局/路径序/指纹身份反控（mn_data_owned_static_countercontrols：声明序偏移、严格 DFS Str 路径、名义身份与字符串常量值移动 fp） | blockers: T-05 硬前置 D862（862 executing r3 4/9，worktree lang-862 有在途 WIP）+ H 写面释放（crates/auto-lang-core/test_cases/ac_mn.rs、ace_m1_tests.rs、binding.tsv 归 862 线重绑定后的 870 差分集成窗口）；解阻动作=862 交付并 H 释放后刷新共同 MIR/Unit/profile 基面，按波次合同 §2 登记 H 接线 | next_responsible_role: T-05 work（D862 后）`

**Work 续记（2026-10-28 晚间，T-05/T-06 完成）**：D862 落地后基面刷新=master merge（862/867/878，afcd4d09e）；862 引入的 `MirType::Unit` 在 mn 补具名拒绝/标签兼容臂（admitted 三 profile 集不变，无 S/N 短窗需求——862 未触 ac-mir-native）；N 侧差分层在新基面 4/4 绿。H 接线（862 归档后 H 窗口依波次合同移交 870）：新 case/suite/测试 + binding ac 判级 + ac_mn 诊断面映射（probe 合法 stage 集 parse/resolve/typecheck/run 内的 typecheck 面——`verify` 非 probe 可声明面，管线边界由 mn.* 码本身命名）；852 base suite 与 ac-mn 各 tier 零退化（两项预登记 flake 按 KNOWN-DEBT 归因关闭：计时器采样环境红、rounds 存活负载 flake 隔离 2×绿）。binding ac 列判级与 av 列 862 判定逻辑逐行镜像（ok / gap:mn-text-move / gap:upstream-s-eq / gap:mn-mut-param / gap:mn-lower-method）。

**Work 交接明细（2026-10-28 T-01..T-04 完成时）**：

- 实施面：`experimental/ac-mir-native/src/{layout.rs(新),lower.rs,emit.rs,fingerprint.rs,driver.rs,link_driver.rs,cache.rs,worker.rs,incremental.rs}` + `src/bin/mn-worker.rs`（--textlibsha 恢复轴）+ `tests/data_owned_tier.rs`（新 4 测）+ 既有测试机械修复（823 env 字段预存红，coverage §4.8）。零改动：auto-av-core、ac-core、auto-mir-lower、probe（av.rs 探针临时插桩已还原 pristine，git status 干净）。
- 上游缺口移交 S owner（862 线）：①视图 `==` 在 auto-mir-lower 走通用 Bin 路径（StrCmp 守卫未触发，最小源复现存 attachments/870/work/probe-sources/）→ T05 行具名排除 `gap:upstream-s-eq`；②split_units 不重映射指令内嵌类型索引（Aggregate.record_ty 悬空）→ mn 侧 verify 等价解析（resolve_record_ty）+ S 修复建议；③AV clone 行 exit 内释放序为宿主 Rc drop 实现细节 → 台账三级口径冻结（freeze §5）。
- cache 行为：单元 A09' 键轴不变；A10' manifest 新增**条件轴** textlib（仅 owned 链接存在时渲染/入键）；scalar 请求 manifest 渲染与 fp 逐字节不变（B* 零退化实证）。fp 渲染修复（宽类型具名 shape/Char/Str 常量按值）仅影响含宽类型单元。
- 预算：ManagedLedger 三分区沿用无新计量；guest 堆无预算注入（shim 默认 unlimited；注入面非语料面）。
- merge-owned 门禁 pending（未执行，按 2026-10-14 时序裁定）：裸 `cargo t` + 触面档（本计划触及 experimental/ac-mn（独立工作区，不在根 t 面）+ 无根 crate 改动 → 根裸 t 预期零影响，merge 时仍须按 AC-08 执行）；本批开发迭代仅跑 crate 内定向面。

2026-10-28 r1→r2（用户授权修订波次/写面，864纳入拥有型List、871补旧能力与删除门）：仅修订执行合同，不实施。原status/current_step/total_steps、任务勾选与历史记录保留；D868合同和进度不变。V-07补交接/消费者兼容与首例cache、文件化测试、周转及内存核验；详细依据与静态预检见[波次合同](reports/ace-wave-handoff-2026-10-28.md)和[修订收据](attachments/ace-wave-20261028/revision-summary.md)。本记录只说明文档修订，未执行AC、性能测试或独立实现review，merge-owned门禁仍待实施后完成。

2026-10-28 new r1：`stage: new | plan_id: PLAN-870 | plan_revision: 1 | outcome: pass（等 D868，合同可预检） | next: D868 后 work T-01`。全部 AC/V/T 未执行；三处拒绝行的关闭范围、宽标量是否顺带、指令覆盖清单由 T-01/T-02 冻结。本轮未建树、未跑构建、未改任何实现。

## 10. 待澄清事项

- **【已解阻】T-05/T-06 的 D862 + H 前置已于 2026-10-28 晚间满足**（862 合并归档 2012b33dc，H 面移交 870）；T-05/T-06 已完成（提交 afcd4d09e/5ae22bf3a），基面接受记录见 §9。
- 宽数值标量常量（`I64`/`U64`/`F32Bits`/`F64Bits`）是否顺带关闭：默认**不**（归 R2-N 数值族），但若 T-01 发现 record/text 源在合法输入下会产生宽常量，须在 T-02 显式扩范围并说明原因。
- record 实例方法（`R08`）：AV 目前 `gap:av-lower-method`，本Plan按"以 AV 为基准"原则**不做**；若用户要求 mn 先支持，则失去差分基准，需先补 AV 侧能力（另立计划）。
- text与text-record必须分别验证具名profile、合法集合及拒绝边界，可复用底层设施，不能以text-record成功代替text身份覆盖；871还需单独核旧方法等超出D870的义务。
