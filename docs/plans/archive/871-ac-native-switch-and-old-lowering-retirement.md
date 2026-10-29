---
plan_id: PLAN-871
status: archived
feature_name: AC native 消费面切换与旧 HIR 直达 lowering 退役
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-29
plan_revision: 4

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-ac/project.md
  - docs/specs/auto-mir/overview.md
  - docs/specs/auto-test-case/architecture.md
  - docs/specs/auto-ac/source-core-i32.md
  - docs/specs/auto-ac/source-scalar-i32.md
  - docs/specs/auto-ac/source-text.md
  - docs/specs/auto-ac/source-record.md
  - docs/specs/auto-ac/source-text-record.md
new_spec_components:
  - docs/specs/auto-ac/native-backend-and-retirement.md
touched_goals: [GOAL-002]

affects: [auto-ac, auto-mir, auto-test-case]
depends_on: [PLAN-868, PLAN-870]
current_step: 9
total_steps: 9
---

# [PLAN-871] AC native 消费面切换与旧 HIR 直达 lowering 退役

## 0. 变更摘要

[PLAN-868](868-hir-mir-lowering-shared-crate.md) 让 HIR→MIR 单源、[PLAN-870](870-ac-mn-record-text-native-expansion.md) 让 mn 覆盖 record/text/text-record。本Plan收尾迁移：**把 native 的消费面按 profile 逐个切到 mn，然后删除旧 `ac-core` HIR→CLIF lowering**，使"共享 MIR 是唯一语义源"从设计变成事实。

三条硬约束（本Plan的核心判断）：

1. **切换点不能在 `ac-core`**：`ac-mn` 依赖 `ac-core`（前端/HIR），若 `ac-core` 反向调用 `ac-mn` 即成环。切换只能发生在 **driver 层**（`ac-case-worker` / 新的 native CLI wrapper）。
2. **逐 profile 切，不做整批**：core → scalar → text → record → text-record；五个profile分别库存、过覆盖门、shadow后再默认（可以复用底层设施，不能合并身份）。
3. **退役才有"单源"**：只要旧 lowering 还在，就仍有两条从 HIR 出发的路径；删除是验收项，不是清理可选项。

## 1. 目标

- driver 层可显式选择 native 后端（legacy / mn），默认在对应 profile 差分绿后切 mn；切换过程可观测、可回退（profile 粒度）。
- 消费者清单全部切换并验证：`ac` 协议适配器（v2）、`auto-ac-prototype` 的 `build`/`run` 消费面、7 个 `verify-ac-*` 脚本。
- 旧 lowering 与只服务它的代码**删除**（非注释、非 `#[deprecated]` 保留），旧金样/脚本分支/Spec 说明同步收口。
- 报告与缓存身份可解释：backend 选择 + lowering 版本进入报告轴；旧 HIR 键的产物显式失效或迁移，**不得同一逻辑产物两套键**。
- 退役后 `cargo tree` 与源码引用证明无残留消费者；AV 保持独立 oracle，验证能力不下降。

**非目标**：新增语言能力；改 AV 执行/协议 schema（除必要身份轴）；扩展 mn 到 collections/closures/Dyn（属后续）；吞吐优化（既有周转/资源门仍须验收）；产品 CLI（`auto`）默认引擎切换（G7 范畴）；增量发布；native loader/HMR（I08）。

## 2. 架构方案

```text
              ┌──────────────── ac-core ────────────────┐
source ─────▶ │ 前端 · Checked HIR · verify · 链接设施   │   （不再拥有 native lowering）
              └──────────────────────────────────────────┘
                                   │ HIR
                                   ▼
                          auto-mir-lower (868)
                                   │ MIR
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
              ac-mn (lower/emit)            auto-av-core (AV)
                    │                             │
                    └──────┬──────────────────────┘
                           ▼
                 driver 层（切换点）：
                 ac-case-worker（协议侧）/ ac-native CLI
                           │
              legacy 开关（迁移期）→ ac-core::native 旧 lowering（871 删除）
```

| 决定 | 内容 | 理由 |
|---|---|---|
| 切换点位置 | `ac-case-worker` + 独立 native CLI wrapper | 避免 `ac-core ↔ ac-mn` 环；协议侧本来就是后端选择层 |
| 开关形态 | env/feature `AC_NATIVE_BACKEND={legacy,mn}`（名字 T-02 冻结） | 迁移期需可回退；不得进入产品默认路径 |
| 默认切换顺序 | core → scalar → text → record → text-record，每个具名profile独立过G-COV与shadow | 不以三组测试或text-record超集代替旧五域覆盖 |
| 旧路径终局 | 删除 | 保留会使"单源"名不副实，且双路径无法再判"是否还有消费者" |
| 兼容承诺 | `auto-ac-prototype` 为实验 CLI，允许参数/子命令调整，但必须显式声明（无静默行为变化） | 它不是产品 CLI（产品是 `auto`） |

## 3. 技术栈

- 切换点：`experimental/ac-case-worker/src/**`（`acworker/1` 协议，承载 v2 `ac` 适配器）、新 `experimental/ac-core` 之外的 native CLI（拟 `experimental/ac-mir-native/src/bin/ac-native.rs`，T-02 定名）。
- 旧 lowering 退役面：`experimental/ac-core/src/native.rs`（`lower_object`/`lower_text_object` 等）、旧native独占辅助（须证明独占）、`main.rs` 的 `build`/`run` 子命令。
- 保留：link.rs、verify.rs、descriptor.rs、source/*；特别保留`text_resources.rs`的verify层分析与`TextResourcePlan`生产/消费接口。verify.rs仍有实际调用，禁止将其整文件当旧lowering删除。
- 语料/判定：`crates/auto-lang/test/cases/ace-m1/`、`av-data/821`、`crates/auto-lang/src/tests/{ace_m1_tests,ac_case_tests}.rs`。
- 拟建 `scripts/verify-av-871.ps1`（Scope=shadow/switch/retire/compat/acceptance），**当前不存在、未运行**。

唯一实施树：`D:/autostack/.wt/lang-871/auto-lang`，分支 `plan-871-dev`。

## 4. 需求分析与背景调查

用户 2026-10-28 决定把旧 HIR 直达迁移到共享 MIR 路线；[Design49 §6](../design/49-native-single-mir-lowering.md) 已冻结逐 profile 切换/退役判据。本Plan是该设计的第三段（切换 + 退役）。

已核实事实：

| 事实 | 位置 |
|---|---|
| 旧 lowering 只有两个对象级入口 | `experimental/ac-core/src/native.rs`：`lower_object`、`lower_text_object`（+ 常量/包装器/链接复用） |
| 旧 lowering 的消费者是协议侧与 CLI | `experimental/ac-case-worker`（v2 `ac` adapter 的 worker）、`auto-ac-prototype` 的 `build`/`run` |
| 切换会成环的证据 | `experimental/ac-mir-native/Cargo.toml` 依赖 `auto-ac-prototype`；反向调用不可行 |
| mn 已有单元/增量/链接设施可复用 | `ac-mir-native/src/{incremental,link_driver,emit,worker}.rs`（818 交付） |
| 收据不动的边界 | `docs/specs/auto-test-case/ace-m1-conformance.md` + `ace-certify`（`ace.m1.cert/1` 精确键集） |

迁移期风险（本Plan必须处置）：报告/缓存身份混用（HIR 键 vs MIR 键）、trap/exit 映射不一致（旧 `ExitProcess(70)` 与 AV typed trap 码的语义对应）、`bool` 规范化、`Str` 布局与 drop 深度。

本次授权与输入：用户明确要求修订波次/实际写面、补871旧能力覆盖及删除门、让864支持List与拥有型type元素；允许动作仅计划/设计/战略文档修订，不实施代码、不调整预算、不改868。基面路径、内容SHA-256、旧稿快照及静态预检见[修订收据](attachments/ace-wave-20261028/revision-summary.md)；观察到的current Spec/调用点与未来拟建入口分开记录。

## 5. 详细设计

### 5.0 波次、实际写面与交接（本次修订）

按[波次交接合同](reports/ace-wave-handoff-2026-10-28.md)§1–2执行：S为共享source/HIR/lowering/MIR/ABC/AV及会话接口，N为ac-mn，H为公共driver/SDK适配/测试注册与binding；分别单写，跨窗同时取窗。共享lowering的未来写面为D868后的`experimental/auto-mir-lower/src/**`，旧`probe/src/hir_adapter.rs`仅作搬前调查，不能再落新实现。T-02及接线前核验实际已合入revision、Spec/API/版本、owner释放与消费者兼容证据并记录接受；review/repair未完不交WIP。

第2波可准备库存/shadow与独立driver/CLI；N、H按实际接线取窗。ac-core删除属于S，优先在864/866释放后、865接线前插入；G-COV或G-DELETE未就绪则立即让窗，不阻塞865/863继续功能线，后续释放点再接。D870只解锁已覆盖子集shadow，D871最终交付还必须满足本Plan新增的五profile旧能力覆盖/删除门，不承诺第2波直接完成。

2026-10-29 方法producer为[PLAN-881](881-shared-record-method-lowering.md) r2，静态缺口见[调查报告](reports/ace-method-lowering-gap-2026-10-29.md)。默认公共接线插序为864→866→881→本Plan删除窗（门齐才取）→865→863；提前插881须owner登记释放/接受，规划排序本身不授予窗口。本Plan可继续只读库存/独立准备，S/N/H修改不得与881并写；881未交付或其他覆盖未齐时让出删除窗，不阻塞后续功能线。

首例即按既有文件化AutoDown/TestCase、cache fresh/hit/restart/body-edit、未改函数生产工作=0与现行P90≤2000ms口径验证；亚秒目标及P95/P99另列，不能伪称当前硬门已是亚秒。内存/资源峰值、live/回收及cache容量在T-02沿上游预算交集冻结，记录命令、样本和反控，不擅自提高上限。共享核心/SDK/测试迁移先确认拆分owner实际位置与释放；产品默认入口、旧AutoVM/a2r和868合同/进度不在本次修订写面。

### 5.1 前置、写面与顺序

- 硬前置：`D868`（lowering 单源）、`D870`（record/text 差分绿）、`D852`（语料与判定）。
- 写面：`ac-case-worker`、新 native CLI、`ac-core`（删除旧 lowering 与其调用点）、`verify-ac-*.ps1` 脚本、报告身份轴所在文件、Spec/矩阵。
- 顺序（严格）：五profile能力/消费者inventory→逐profile G-COV→逐profile shadow→已过门profile切换→全部消费者迁移→全局G-DELETE→删除旧lowering→脚本/金样/Spec收口。D870不覆盖的方法行未交付即阻塞对应切换和最终删除。

### 5.2 消费者清单与切换矩阵（T-01 冻结）

| 消费者 | 现状 | 切换后 | 验证 |
|---|---|---|---|
| v2 `ac` 适配器（`ac/check|build|run`） | worker 调 ac-core lowering | worker 调 mn（经 driver 选择） | `ac-native` 语料全绿 + 报告身份更新 |
| `auto-ac-prototype` CLI `build/run` | 直接 lowering | 改由 native CLI 承担（或保留为"前端/检查"角色） | 逐子命令行为对照表 |
| `verify-ac-{741,745,767,770,771,772,782}.ps1` | 断言旧 CLI 行为 | 改指向新后端；旧口径作为差分参照保留一版 | 每脚本一份"旧断言→新断言"映射 |
| `ace-m1` 的 `ac` 列 | mn（scalar 时部分 gap） | mn 覆盖 record/text 后为实测 | binding 状态列 |
| 783 artifact 层的 HIR 键条目 | 旧路径产物 | 显式失效/迁移（不并存双键） | 恢复/命中用例 |

### 5.2.1 G-COV旧能力覆盖门（T-01/T-02冻结）

库存必须逐行覆盖core/scalar/text/record/text-record五个profile的current Spec与旧永久测试、合法及拒绝集合、CLI/协议/脚本/cache消费者；每行绑定旧证据、共享lowering+mn覆盖用例、独立expected、实测、producer/consumer与阻塞owner。必含静态/只读实例方法、mut/move/clone、别名安全字段赋值、Unicode、借用与全出口资源清理。text身份不能折叠为text-record。

D870只覆盖AV现有820/821子集，旧record静态/只读实例方法由PLAN-881认领（尚未交付；M-01..06涵盖整模块、真方法单体前端、参数模式/身份/候选事务与恢复）。涉及方法的profile必须接受D881的原R08=42、771/782原方法与V-09..13/P-01..06资源/模式/增量证据、逐域正反方法库存后才可过对应G-COV行；不将D881新增为无关profile的统一启动门。871登记缺口及验收oracle，不能用NA/unsupported/改expected/删旧用例使G-COV变绿，也不能在迁移中夹带语言实现。通用view eq/ne、非方法类型链接/资源边界继续独立列owner，D881不自动关闭这些行。未补齐时可继续库存/独立准备，未覆盖profile不切默认，最终删除及D871保持阻塞。

### 5.3 shadow 门与切换判据

- **shadow**：同一输入在同一进程内跑两后端（driver 层链接两者），父侧比对值/trap/诊断/资源台账；**差异一律红**，不得以"后端不同"豁免。
- 切换判据（逐profile）：该profile的G-COV逐行齐全 + shadow在冻结语料上连续绿（次数T-02冻结）+ 无未分类差异 + 身份轴正确 → 默认切mn；不能只比较新路径已支持的交集。
- **回退**：保留 profile 级 legacy 开关至删除前；切换后红则回退该 profile，其他 profile 不受影响。

### 5.4 G-DELETE删除门与退役顺序

删除前同时具备：五profile G-COV与shadow全过、全部CLI/协议/7脚本/cache消费者迁移、旧键显式失效与恢复重验、删除符号/依赖清单和保留代码白名单。注入缺失方法/消费者行必须阻止切换或删除并保持旧可用路径；局部切换不算D871。

1. T-05完成消费者迁移及身份/cache验证，保存冻结旧语义/差分摘要。
2. 取S+所需N/H窗口；优先864/866释放后插入，未就绪让窗，按后续释放点重排。
3. 只删除`native.rs`中HIR直达lowering及可证明独占的辅助。保留`text_resources.rs`的verify层`TextResourcePlan`分析/接口及verify.rs调用；若抽离仅native部分，逐符号证明无共享消费者，不做整文件删除。
4. 旧执行分支/金样按迁移映射收口，保留历史摘要；current Spec保留旧源码语义，只迁实现归属。
5. 编译/依赖与引用检查证明无旧lowering残留消费者；保留分析仍有真实调用并通过正反资源用例。各五profile支持/拒绝集合不缩小，才可声明单源完成。

### 5.5 身份、cache 与证据

- 报告：新增 backend 选择 + lowering 版本轴（868 已引入版本轴，本Plan加 backend 维度）；旧报告 STALE 属预期。
- 收据：`ace.m1.cert/1` 键集**不动**；若不新增字段无法表达 backend，则按"收据只证明 a2r 认证/与 backend 无关"的既有边界处理，T-02 显式声明。
- cache：旧 HIR 键产物显式失效（版本化）或迁移；禁止同一逻辑单元在两种键下并存；跨进程恢复仍走 decode+verify 重准入。

### 5.6 测试框架接入（既有 TestCase 协议，零 schema 改动）

- recipe 不变；`ac` 适配器后端在 worker 内切换，case 文件不感知（这正是"同一 case 判不同后端"的既有形态）。
- `ace-m1` 的 `ac` 列在切换后应保持绿（不得因后端切换而改期望值）；若必须改，属语义变化 → 回 870/Design49 裁定，不在本Plan顺手改绿。
- 反控：故意让 mn 与 AV 分歧（注入缝）→ 差分/`ac` 列必红。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/native-backend-and-retirement.md | 无 → 五profile G-COV/shadow/默认切换、G-DELETE、失败保持旧路径及兼容声明 | 单源收口且不丢旧能力 | AC-01/02/05/10 |
| SD-02 | modify | docs/specs/auto-ac/project.md | "AC 原生 lowering"改为"消费共享 MIR lowering"；旧入口退役说明 | 事实纠正 | AC-05 |
| SD-03 | modify | docs/specs/auto-mir/overview.md | 追加 native consumer 的默认后端与身份轴 | 可见性 | AC-04 |
| SD-04 | modify | docs/specs/auto-test-case/architecture.md | 追加 backend 身份轴（不触收据键集） | 可判定 | AC-04 |
| SD-05 | retire | docs/specs/auto-ac/source-{core-i32,scalar-i32,text,record,text-record}.md 中"native lowering 实现位置"表述 | 保留源码域语义，删去"由 ac-core native 生成"的旧归属 | 归属迁移 | AC-05 |

## 6. 测试设计

| V | AC/不变量 | 实际/拟建入口 | 可区分错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 shadow门 | core/scalar/text/record/text-record五个具名profile，各按完整G-COV清单三路及独立oracle判定 | 值/trap/诊断/台账一致；缺旧方法或只测交集必红，不合并profile身份 | T-03 |
| V-02 | AC-02 逐 profile 切换 | 切换后 `ac-native` 与 `ace-m1` 的 `ac` 列 | 切后仍绿；回退开关可用且可观察 | T-04 |
| V-03 | AC-03 消费者全切 | v2 适配器、native CLI、7 个脚本 | 无调用旧 lowering（trace/编译期断言） | T-05 |
| V-04 | AC-04 身份/cache | 报告轴、旧 HIR 键产物、跨进程恢复 | 混键被拒绝；恢复重准入；旧报告 STALE 符合声明 | T-05 |
| V-05 | AC-05 退役无残留 | 删除后全量构建 + grep + cargo tree + 脚本 | 任何残留引用即红；构建不得依赖已删符号 | T-06 |
| V-06 | AC-10 G-COV | 五profile×旧特性/消费者库存，含静态/只读方法、mut/move/clone、Unicode/alias/drop及CLI/cache | 每行有原Spec/测试、共享lowering+mn用例与独立预期/实测；注入缺方法或消费者阻塞切换，拒绝合法例不算覆盖 | T-01/02/03/04 |
| V-07 | AC-05/10 G-DELETE | 缺覆盖/旧消费者/旧键恢复反控；资源分析保留白名单与verify真实调用 | 门失败不删除旧可用路径；门通过后无旧lowering，但TextResourcePlan分析及verify调用仍有效，资源正反例成立 | T-05/06 |
| V-08 | AC-04/AC-07 写面/兼容 | T-02及公共接线交接摘要；已交付消费者并集与窗口清单 | 真实D门/释放/接受齐全；缺交接或消费WIP不准接线；受影响profile兼容验证及首例cache/周转/内存证据齐全 | T-02冻结；最终work任务执行；review复核 |

定向命令：`cargo check --locked`（三 workspace）+ 定向 `cargo test` + 脚本三 scope；完整档归 merge。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 五个具名profile各按完整G-COV库存shadow连续绿（次数T-02冻结），不以交集代替、不豁免缺能力；V-01/06 |
| AC-02 | work | 逐 profile 默认切换成立且可回退，切换状态可观察；V-02 |
| AC-03 | work | 全部消费者切到 mn（协议/CLI/脚本），无旧 lowering 调用；V-03 |
| AC-04 | work | backend/lowering 身份轴正确，旧键产物不并存，恢复重准入；V-04 ；首例cache/文件化测试、周转/内存预算及兼容证据齐全，V-08 |
| AC-05 | work | G-DELETE通过后旧lowering/独占辅助删除，无残留消费者；TextResourcePlan及verify分析保留有效，旧语义不缩；V-05/07 |
| AC-06 | work | 反控可证差分/`ac` 列有判别力；V-01 |
| AC-07 | review | 独立核 AC/V/SD、删除范围（不该删的没删/该删的没留）、无行为夹带、warnings/fmt；核V-08交接与首例指标 |
| AC-08 | merge | 冻结提交裸 `cargo t` + `tv` 通过；失败不豁免 |
| AC-09 | merge | SD-01..05沉淀、矩阵/ledger/归档/guard完成；G-COV/G-DELETE及消费者证据齐全后记录单源成立 |
| AC-10 | work | 五profile G-COV及G-DELETE清单齐全；方法缺口有producer/阻塞owner且须实际交付后才过门，失败保持旧可用路径；V-06/07 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | D868、D870（正式shadow/切换前）；只读库存可提前 | 五profile旧能力/消费者/删除/保留面inventory → [inventory-t01](attachments/871/work/inventory-t01.md)（C1–C9 消费者、删除/保留白名单、五profile行组、gap 登记、身份/cache 现状与 868 脚本版本钉遗留） | AC-03/05/10 | [x] |
| T-02 | work | T-01 | 冻结逐profile G-COV、shadow、切换及全局G-DELETE、资源分析保留白名单、身份/cache和S/N/H交接；冻结fresh/hit/restart/body-edit、周转/内存口径与反控 → [freeze-t02](attachments/871/work/freeze-t02.md)（F1 开关/切换点、F2 shadow 判据与 2 连绿、F3 G-COV、F4–F7） | AC-01..05/10 | [x] |
| T-03 | work | T-02、D868、D870、对应窗口释放/接受 | 五profile按G-COV库存shadow与差异分类，缺方法行保持阻塞；V-01/06 → [shadow-t03](attachments/871/work/shadow-t03.md)：`shadow_tier_871` 47 行五profile双腿+独立oracle，两轮连续 6/6 绿（48.02s/34.97s，exit 0），唯一差异面=释放批内序（F2 已冻非语义），反控只红变体行 | AC-01/10 | [x] |
| T-04 | work | T-03、逐profile G-COV通过 | core/scalar/text/record/text-record分别默认切换与回退；未覆盖旧合法能力不切该profile；V-02/06 → [switch-t04](attachments/871/work/switch-t04.md)：`backend.rs` 五profile默认表=mn + env逐profile回退（每请求读取）、`ac-native` CLI（build/build-source/backend）、worker core\|scalar 走 mn（含 store/HIR atom/trace lib）；实测 6+5+7 绿、shadow 重构后 6 绿、`--features test-cases` ace_m1 7绿/1既有ignored；后置=cache_t05_ 6红归T-05、报告轴/stale归T-05 | AC-02/10 | [x] |
| T-05 | work | T-04、H/N释放/接受 | 全消费者/CLI/7脚本迁移及身份/cache；删除门反控；V-03/04/07；执行V-08首例cache/文件化/周转/内存与兼容验证 → [identity-cache-t05](attachments/871/work/identity-cache-t05.md)：mn `CacheMode`(fresh/read-only)+报告词汇、`ac-native-backend` 轴与按行 profile 的 stale 镜像（真实 worker 漂移守卫）、5 脚本 build 腿切 ac-native（745/741/767/771 全绿，5.1+7 双跑）、脚本暴露的 4 个 mn 能力缺口当日闭合（view-record clone/纯 id 方法/前向 t_MutRet/owned 字段读）+ ctor 操作数快照缺口闭合；worker cache/backend 15 绿、报告/stale 与 ace_m1 绿；782 整脚本复跑与 868 identity scope 待补（不影响 D-门） | AC-03/04/10/06 | [x] |
| T-06 | work | T-05、G-DELETE通过、S及所需N/H释放/接受 | 删除旧lowering及独占辅助，保留TextResourcePlan/verify分析；V-05/07；证据本地、摘要提交 → [retire-t06](attachments/871/work/retire-t06.md)：`native.rs`(4652行)/CLI build面/783 vertical(node+perf+driver)/worker legacy路径/av-probe 旧参照全部删除，cranelift 从 ac-core 依赖树消失；`capabilities_check`+`runtime_contract` 移入 lib；报告轴对 legacy 返回 `retired:legacy`（旧报告必 STALE）；`scripts/verify-ac-871-retire.ps1` 8/8 通过（含"重引入旧符号必编译失败"反控）；741/745/767 脚本复跑绿；已知缩面（ac-core 旧 PE 套件、783 store 面测试）登记待 T-07 裁定 | AC-05/10 | [x] |
| T-07 | review | T-06 | 独立最终review；merge 门 pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交裸 `cargo t` + `tv` | AC-08 | [ ] |
| T-09 | merge | T-08 | Spec/矩阵/ledger/归档/guard；声明单源成立并交接后续（collections 等） | AC-09 | [ ] |

## 9. 复审记录

2026-10-29 `stage: review | review_scope: final | reviewer_identity: 实施会话内自审（无独立 reviewer 会话/模型可用；限制已声明：结论由命令收据、提交差异与 Spec 输入重建，未采信执行者摘要） | plan_id: PLAN-871 | plan_revision: 4 | outcome: pass（final；merge-owned 门禁 pending） | worktree: D:/autostack/.wt/lang-871/auto-lang | branch: plan-871-dev | base_commit: 25bcae36f（master）| reviewed_commit: 见 §10（冻结提交，合并前记录）| dependency_revisions: D852/D864/D866/D868/D870/D881 全部 archived（881=db0773130，MIR_LOWERING_VERSION=2）；ac-ui 兄弟树 detached@3864e1a（父仓构型） | spec_inputs: docs/specs/auto-ac/{project,mir-native-scalar,mir-native-cache,source-*}.md、docs/specs/auto-mir/overview.md、docs/specs/auto-test-case/architecture.md（工作期未改，§7 规范增量待 merge 落盘） | acceptance_results: AC-01 pass（T-03 五 profile 双腿两连绿 + 退役后单腿冻结 oracle `shadow_tier_871` 6/6；旧腿差分证据按 T-02 F2 两态存档）；AC-02 pass（默认表 mn 五 profile，legacy 显式退役拒绝，解析状态可观察）；AC-03 pass（worker/ac-native/741/745/767/771/770 脚本面迁移；882 无 build 腿；旧 CLI 面删除）；AC-04 pass（`ac-native-backend` 轴 linked 必报、按行 profile 重算、legacy→`retired:legacy` 必 STALE、mn CacheMode fresh/read-only、跨键不命中用例）；AC-05 pass（`verify-ac-871-retire.ps1` 8/8：退出文件/grep 零引用/cargo tree 无 cranelift/三 workspace check/保留面/**注入旧符号必编译失败的删除门反控**）；AC-06 pass（shadow 变异 oracle 只红该行 + 删除门反控）；AC-07 pass（本记录）；AC-08/AC-09 pending（merge-owned：裸 `cargo t`+`tv`、Spec/矩阵/ledger/归档）；AC-10 pass（五 profile 行齐；脚本迁移暴露的 4+1 处 mn 能力缺口当日闭合并加回归，残留 gap 有 owner 与 oracle） | pending_closeout_gates: 裸 `cargo t` + `tv`（merge）、SD-01..05 落盘、`.autoos/specs.json`+spec-index、矩阵/ledger、归档/guard | findings: F-871-R1（低/覆盖债，非 in-contract 缺陷）：随旧 lowering 退役整删的 ac-core 套件（text_review_r2..r10/deep_regression/text_resources/text_record_resources）与 783 store 面套件，其 verify-only 断言与 store 语义由 hir_verify/text_binding/*_hir/*_source、mn cache/transaction/resources/incremental 与 shadow/gap 覆盖；如后续要求逐行恢复，按 T-01 §7 从 `3aa5560b2~1` 取回 verify 腿再删 native 腿 → 归 KNOWN-DEBT；F-871-R2（环境）：pwsh7→5.1 嵌套链的 `Get-FileHash` 模块自动加载偶发失败，5.1 顶层执行稳定 → 复跑宿主约定；F-871-R3（脚本）：782 的 scope 断言在 871 树需 `AUTO_LANG_PLAN_TREE=871` 显式前缀，770 的 per-plan scope 步骤按 782 既有约定"唯一失败步骤"计；F-871-R4（环境）：并发运行两个 771 会导致 ac-core `--lib` 瞬时失败（target/临时目录竞争），串行复跑绿 | evidence: docs/plans/attachments/871/work/{inventory-t01,freeze-t02,shadow-t03,switch-t04,identity-cache-t05,retire-t06}.md + `scripts/verify-ac-871-retire.ps1` + 各脚本 receipt（local） | next: merge（T-08/T-09）`

2026-10-29 `stage: new | plan_id: PLAN-871 | plan_revision: 4 | outcome: pass（881 r2规划补核的交接同步，未实施） | changed: §5.0/§5.2.1方法producer修订与M-06/V-09..13/P-01..06证据引用 | next: 同原G-COV/G-DELETE；未改变AC/T/进度、不提前获得窗口`。用户授权本轮复核规划；r3/r2历史保留如下。

2026-10-29 `stage: new | plan_id: PLAN-871 | plan_revision: 3 | outcome: pass（方法producer及写面交接修订的静态预检；未实施） | changed: §5.0/§5.2.1，方法owner由待立项改PLAN-881；保留AC-01..10/T-01..09、status/current_step及全部历史进度 | next: 只读库存可提前；方法profile的G-COV等D881实交证据，全局G-DELETE仍独立核验`。用户本轮仅授权规划；未修改864、未取得任何实施窗口，旧r2快照保留如下。

`stage: new | plan_id: PLAN-871 | plan_revision: 2 | outcome: pass（修订草案静态预检；实施仍等真实D门/写面释放） | next: 旧能力库存可只读先做，正式shadow等D868/D870，切换/删除逐门执行`

2026-10-28 r1→r2（用户授权修订波次/写面，864纳入拥有型List、871补旧能力与删除门）：仅修订执行合同，不实施。原status/current_step/total_steps、任务勾选与历史记录保留；D868合同和进度不变。V-08补交接/消费者兼容与首例cache、文件化测试、周转及内存核验；详细依据与静态预检见[波次合同](reports/ace-wave-handoff-2026-10-28.md)和[修订收据](attachments/ace-wave-20261028/revision-summary.md)。本记录只说明文档修订，未执行AC、性能测试或独立实现review，merge-owned门禁仍待实施后完成。

2026-10-29 work 开工授权：用户以 `/goal 完成计划871`（/auto-plan-goal 链）授权整个链路——在 `D:/autostack/.wt/lang-871/auto-lang`（分支 `plan-871-dev`，基面 master@25bcae36f）内实施 T-01..T-06、独立 review、限额内修复与 merge。硬前置已核：D852/D868/D870/D864/D866/D881 全部 archived（含方法 producer 881 已交付 db0773130）；§5.0 插序 864→866→881 均已落地，删除窗空闲可取。主检出 WIP 仅为 docs 路线图/计划簿记，无代码路径 WIP。

2026-10-28 new r1：`stage: new | plan_id: PLAN-871 | plan_revision: 1 | outcome: pass（等 D868/D870，合同可预检） | next: D870 后 work T-01`。全部 AC/V/T 未执行；开关名、shadow 连续绿次数、收据声明、脚本映射表由 T-01/T-02 冻结。本轮未建树、未跑构建、未改任何实现。

## 10. 合并记录（merge）

2026-10-29 `stage: merge | plan_id: PLAN-871 | plan_revision: 4 | outcome: pass | delivery_commit: 87dd2b6ad | reviewed_commit: a2ae5f856（落地前 `88a0da594` 的 rebase 等价副本；`git range-diff 25bcae36f..plan-871-dev@{1} master..plan-871-dev` 14/14 `=`，仅 base 前移）| merge_form: ff-only（主检出 master dc2820437→87dd2b6ad）| gates_on_frozen_delivery: 裸 `cargo t` = 3204/3204 passed（673 skipped，452.7s）；`cargo tv` = 163/163 passed（3.3s，2060 skipped）；`scripts/verify-ac-871-retire.ps1` 8/8；脚本 741(11)/745(11 步 74 例)/767(13 步 45 例)/770(13 步，per-plan scope 按 782 约定归因)/771(23 步 23 例)/782(整体绿，后继树 opt-in)；worker/ac-mn/ac-core 套件绿 | canonical_delta: SD-01 `auto-ac/native-backend-and-retirement.md`（新增）、SD-02 `auto-ac/project.md`、SD-03 `auto-mir/overview.md`、SD-04 `auto-test-case/architecture.md`、SD-05 `auto-ac/source-{core-i32,scalar-i32,text,record,text-record}.md` + `auto-ac/plans.md` 行 | ledger: `.autoos/specs.json` v134→v135（P871-1 designs / P871-2 tests / P871-3 reviews / P871-4 reports；守卫仅允许版本+4 项变化），`scripts/spec-index.py` 重生成 INDEX | archive: `docs/plans/archive/871-ac-native-switch-and-old-lowering-retirement.md`（status: archived）| cleanup: 见 §11 | 备注：主检出存在与本 Plan 无关的既有 docs WIP（其他会话），merge 未触碰；evidence 摘要仅入库 md（依据 2026-10-28 证据政策）`

## 11. 清理记录（cleanup）

2026-10-29 清理完成：`bash /mnt/d/autostack/wt-guard.sh`（Git Bash 中 D: = /mnt/d）对 `D:/autostack/.wt/lang-871/auto-lang` 与 `.../auto-ui` 均输出 **clean（无 reparse point）**；随后 `git worktree remove` 删除 auto-lang/auto-ui 两个 worktree、`git branch -d plan-871-dev`（was 87dd2b6ad）、`auto-down` 兄弟 detached worktree 同样 guard-clean 后移除，组目录 `D:/autostack/.wt/lang-871` 已删除（rmdir/空目录校验）。主检出 `git worktree list` 仅余他 Plan 树；本 Plan 的交付提交/归档/ledger 均已入库。

## 12. 待澄清事项

- native CLI 的归属：并入 `ac-mir-native` 的 bin，还是新建 `experimental/ac-driver`。默认前者（避免多一层 crate）；若 T-01 发现 CLI 需要同时依赖协议层，再改为独立 wrapper。
- `auto-ac-prototype` 的 `build`/`run` 是"保留为前端/检查命令"还是"整体迁移到 native CLI"：默认**迁移**，`auto-ac-prototype` 只留前端/检查，但这属于实验 CLI 的兼容变化，需在 T-02 明确声明。
- 收据是否需要表达 backend：默认**不需要**（收据只覆盖 a2r 认证面），若 review 认为需要，另立收据版本，不在本Plan顺手扩键集。
