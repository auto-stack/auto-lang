---
plan_id: PLAN-871
status: drafting
feature_name: AC native 消费面切换与旧 HIR 直达 lowering 退役
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-ac/native-backend-and-retirement.md
touched_goals: [GOAL-002]

affects: [auto-ac, auto-mir, auto-test-case]
depends_on: [PLAN-868, PLAN-870]
current_step: 0
total_steps: 9
---

# [PLAN-871] AC native 消费面切换与旧 HIR 直达 lowering 退役

## 0. 变更摘要

[PLAN-868](868-hir-mir-lowering-shared-crate.md) 让 HIR→MIR 单源、[PLAN-870](870-ac-mn-record-text-native-expansion.md) 让 mn 覆盖 record/text/text-record。本Plan收尾迁移：**把 native 的消费面按 profile 逐个切到 mn，然后删除旧 `ac-core` HIR→CLIF lowering**，使"共享 MIR 是唯一语义源"从设计变成事实。

三条硬约束（本Plan的核心判断）：

1. **切换点不能在 `ac-core`**：`ac-mn` 依赖 `ac-core`（前端/HIR），若 `ac-core` 反向调用 `ac-mn` 即成环。切换只能发生在 **driver 层**（`ac-case-worker` / 新的 native CLI wrapper）。
2. **逐 profile 切，不做整批**：scalar → record → text-record；每个 profile 先 shadow（双后端同跑、父侧比对）再默认。
3. **退役才有"单源"**：只要旧 lowering 还在，就仍有两条从 HIR 出发的路径；删除是验收项，不是清理可选项。

## 1. 目标

- driver 层可显式选择 native 后端（legacy / mn），默认在对应 profile 差分绿后切 mn；切换过程可观测、可回退（profile 粒度）。
- 消费者清单全部切换并验证：`ac` 协议适配器（v2）、`auto-ac-prototype` 的 `build`/`run` 消费面、7 个 `verify-ac-*` 脚本。
- 旧 lowering 与只服务它的代码**删除**（非注释、非 `#[deprecated]` 保留），旧金样/脚本分支/Spec 说明同步收口。
- 报告与缓存身份可解释：backend 选择 + lowering 版本进入报告轴；旧 HIR 键的产物显式失效或迁移，**不得同一逻辑产物两套键**。
- 退役后 `cargo tree` 与源码引用证明无残留消费者；AV 保持独立 oracle，验证能力不下降。

**非目标**：新增语言能力；改 AV 执行/协议 schema（除必要身份轴）；扩展 mn 到 collections/closures/Dyn（属后续）；性能目标；产品 CLI（`auto`）默认引擎切换（G7 范畴）；增量发布；native loader/HMR（I08）。

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
| 默认切换顺序 | scalar → record → text-record | 与 870 的差分门一一对应；每步单独验收 |
| 旧路径终局 | 删除 | 保留会使"单源"名不副实，且双路径无法再判"是否还有消费者" |
| 兼容承诺 | `auto-ac-prototype` 为实验 CLI，允许参数/子命令调整，但必须显式声明（无静默行为变化） | 它不是产品 CLI（产品是 `auto`） |

## 3. 技术栈

- 切换点：`experimental/ac-case-worker/src/**`（`acworker/1` 协议，承载 v2 `ac` 适配器）、新 `experimental/ac-core` 之外的 native CLI（拟 `experimental/ac-mir-native/src/bin/ac-native.rs`，T-02 定名）。
- 旧 lowering 退役面：`experimental/ac-core/src/native.rs`（`lower_object`/`lower_text_object` 等）、`text_resources.rs`（若仅服务 lowering，T-01 核实）、`main.rs` 的 `build`/`run` 子命令。
- 保留：`link.rs`（链接设施）、`verify.rs`、`descriptor.rs`、`source/*`（前端与 HIR 是单源）。
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

## 5. 详细设计

### 5.1 前置、写面与顺序

- 硬前置：`D868`（lowering 单源）、`D870`（record/text 差分绿）、`D852`（语料与判定）。
- 写面：`ac-case-worker`、新 native CLI、`ac-core`（删除旧 lowering 与其调用点）、`verify-ac-*.ps1` 脚本、报告身份轴所在文件、Spec/矩阵。
- 顺序（严格）：inventory → shadow 门（scalar）→ 切 scalar → shadow 门（record/text-record）→ 切全部 → 删除旧 lowering → 脚本/金样/Spec 收口。

### 5.2 消费者清单与切换矩阵（T-01 冻结）

| 消费者 | 现状 | 切换后 | 验证 |
|---|---|---|---|
| v2 `ac` 适配器（`ac/check|build|run`） | worker 调 ac-core lowering | worker 调 mn（经 driver 选择） | `ac-native` 语料全绿 + 报告身份更新 |
| `auto-ac-prototype` CLI `build/run` | 直接 lowering | 改由 native CLI 承担（或保留为"前端/检查"角色） | 逐子命令行为对照表 |
| `verify-ac-{741,745,767,770,771,772,782}.ps1` | 断言旧 CLI 行为 | 改指向新后端；旧口径作为差分参照保留一版 | 每脚本一份"旧断言→新断言"映射 |
| `ace-m1` 的 `ac` 列 | mn（scalar 时部分 gap） | mn 覆盖 record/text 后为实测 | binding 状态列 |
| 783 artifact 层的 HIR 键条目 | 旧路径产物 | 显式失效/迁移（不并存双键） | 恢复/命中用例 |

### 5.3 shadow 门与切换判据

- **shadow**：同一输入在同一进程内跑两后端（driver 层链接两者），父侧比对值/trap/诊断/资源台账；**差异一律红**，不得以"后端不同"豁免。
- 切换判据（逐 profile）：shadow 在冻结语料上连续绿（次数由 T-02 冻结）+ 无未分类差异 + 报告身份轴正确 → 默认切 mn。
- **回退**：保留 profile 级 legacy 开关至删除前；切换后红则回退该 profile，其他 profile 不受影响。

### 5.4 退役清单与顺序

按依赖顺序删除，每步先证明"无消费者"：

1. 协议/CLI 调用点 → 改指 mn（编译期证明）。
2. `ac-core` 内旧 lowering 入口与其独占辅助（`native.rs` 相关臂、`text_resources.rs` 若独占）删除。
3. 旧金样/快照/脚本分支归档（保留历史收据，不保留执行路径）。
4. Spec/矩阵/账本回写；`auto-ac/project.md` 的"原生 lowering"表述改为"消费 mn"。
5. 残留检查：`cargo tree`、源码 grep（`lower_object`/`lower_text_object`/旧二进制名）、脚本引用、文档链接。

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
| SD-01 | add | docs/specs/auto-ac/native-backend-and-retirement.md | 无 → backend 选择/切换判据/退役判据/兼容声明 | 单源的工程收口 | AC-01/02/05 |
| SD-02 | modify | docs/specs/auto-ac/project.md | "AC 原生 lowering"改为"消费共享 MIR lowering"；旧入口退役说明 | 事实纠正 | AC-05 |
| SD-03 | modify | docs/specs/auto-mir/overview.md | 追加 native consumer 的默认后端与身份轴 | 可见性 | AC-04 |
| SD-04 | modify | docs/specs/auto-test-case/architecture.md | 追加 backend 身份轴（不触收据键集） | 可判定 | AC-04 |
| SD-05 | retire | docs/specs/auto-ac/source-{core-i32,scalar-i32,text,record,text-record}.md 中"native lowering 实现位置"表述 | 保留源码域语义，删去"由 ac-core native 生成"的旧归属 | 归属迁移 | AC-05 |

## 6. 测试设计

| V | AC/不变量 | 实际/拟建入口 | 可区分错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 shadow 门 | scalar / record / text-record 三组冻结语料，双后端同跑 | 值/trap/诊断/台账全一致；任一差异必红 | T-03 |
| V-02 | AC-02 逐 profile 切换 | 切换后 `ac-native` 与 `ace-m1` 的 `ac` 列 | 切后仍绿；回退开关可用且可观察 | T-04 |
| V-03 | AC-03 消费者全切 | v2 适配器、native CLI、7 个脚本 | 无调用旧 lowering（trace/编译期断言） | T-05 |
| V-04 | AC-04 身份/cache | 报告轴、旧 HIR 键产物、跨进程恢复 | 混键被拒绝；恢复重准入；旧报告 STALE 符合声明 | T-05 |
| V-05 | AC-05 退役无残留 | 删除后全量构建 + grep + cargo tree + 脚本 | 任何残留引用即红；构建不得依赖已删符号 | T-06 |

定向命令：`cargo check --locked`（三 workspace）+ 定向 `cargo test` + 脚本三 scope；完整档归 merge。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | shadow 门在三个 profile 上连续绿（次数按 T-02），差异零豁免；V-01 |
| AC-02 | work | 逐 profile 默认切换成立且可回退，切换状态可观察；V-02 |
| AC-03 | work | 全部消费者切到 mn（协议/CLI/脚本），无旧 lowering 调用；V-03 |
| AC-04 | work | backend/lowering 身份轴正确，旧键产物不并存，恢复重准入；V-04 |
| AC-05 | work | 旧 lowering 与独占辅助已删除，Spec/归属回写；无残留引用；V-05 |
| AC-06 | work | 反控可证差分/`ac` 列有判别力；V-01 |
| AC-07 | review | 独立核 AC/V/SD、删除范围（不该删的没删/该删的没留）、无行为夹带、warnings/fmt |
| AC-08 | merge | 冻结提交裸 `cargo t` + `tv` 通过；失败不豁免 |
| AC-09 | merge | SD-01..05 沉淀、矩阵/ledger、归档、guard；并记录"单源"已成立 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | D868、D870 | 消费者与删除面 inventory（协议/CLI/脚本/文档/缓存键），产出 `attachments/871/work/inventory.md` | AC-03/05 | [ ] |
| T-02 | work | T-01 | 冻结切换矩阵、开关名与语义、shadow 判据、收据/身份声明、退役顺序 | AC-01..05 | [ ] |
| T-03 | work | T-02 | shadow 双后端运行与差异分类（scalar 先行）；V-01 | AC-01 | [ ] |
| T-04 | work | T-03 | scalar 默认切换 + 回退验证；再 record/text-record；V-02 | AC-02 | [ ] |
| T-05 | work | T-04 | 消费者全切（协议/CLI/7 脚本）+ 身份/cache 处置；V-03/04 | AC-03/04 | [ ] |
| T-06 | work | T-05 | 删除旧 lowering 与独占辅助、归档旧金样/分支、残留检查；V-05，提交代码/证据 | AC-05 | [ ] |
| T-07 | review | T-06 | 独立最终review；merge 门 pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交裸 `cargo t` + `tv` | AC-08 | [ ] |
| T-09 | merge | T-08 | Spec/矩阵/ledger/归档/guard；声明单源成立并交接后续（collections 等） | AC-09 | [ ] |

## 9. 复审记录

2026-10-28 new r1：`stage: new | plan_id: PLAN-871 | plan_revision: 1 | outcome: pass（等 D868/D870，合同可预检） | next: D870 后 work T-01`。全部 AC/V/T 未执行；开关名、shadow 连续绿次数、收据声明、脚本映射表由 T-01/T-02 冻结。本轮未建树、未跑构建、未改任何实现。

## 10. 待澄清事项

- native CLI 的归属：并入 `ac-mir-native` 的 bin，还是新建 `experimental/ac-driver`。默认前者（避免多一层 crate）；若 T-01 发现 CLI 需要同时依赖协议层，再改为独立 wrapper。
- `auto-ac-prototype` 的 `build`/`run` 是"保留为前端/检查命令"还是"整体迁移到 native CLI"：默认**迁移**，`auto-ac-prototype` 只留前端/检查，但这属于实验 CLI 的兼容变化，需在 T-02 明确声明。
- 收据是否需要表达 backend：默认**不需要**（收据只覆盖 a2r 认证面），若 review 认为需要，另立收据版本，不在本Plan顺手扩键集。
