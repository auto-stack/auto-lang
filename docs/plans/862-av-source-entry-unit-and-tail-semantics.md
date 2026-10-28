---
plan_id: PLAN-862
status: drafting
feature_name: AV源码入口、Unit与尾值语义（R2-E/SEM-05）
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-ac/source-entry-unit.md
  - docs/specs/auto-av/entry-and-unit-profile.md
touched_goals: [GOAL-002]   # 解释器/VM 快周转：热重载与真实源码执行底座

affects: [auto-av, auto-mir, auto-ac, auto-test-case]
current_step: 0
total_steps: 8
---

# [PLAN-862] AV源码入口、Unit与尾值语义

## 0. 变更摘要

本Plan是M1核心骨架成型后的第一批源码扩面切片之一，领取[AV源码与替换路线](../design/strategy/av-source-programs-and-avm-replacement-roadmap.md) **R2-E**中“入口/Unit/尾值/退出通道”部分（`G2a` 前门），并冻结能力矩阵的 **SEM-05**：

- 交付新的具名源码 profile 与执行 profile，使真实Auto程序可以 `fn main() void`、隐式尾值、单文件脚本体进入AV执行，并把“guest返回值 / Unit / 语言trap / 资源失败 / infra”呈现为**互不替代的独立通道**。
- 不改动 `auto.source.scalar-i32.draft` r1 与 `av.core-i32/1`（profile id 1 / AVAB 语义）的既有字节与语义；新域独立版本、双向拒绝。
- 本Plan **不含** 控制台/print（PLAN-863）、集合（864）、闭包（865）、在线热重载（866）。

本轮只起草合同：不建实施树、不跑构建/语料、不启动其他会话，也不改任何在途Plan的合同或进度。

## 1. 目标

- 冻结一套**入口选择与结果通道**语义矩阵：合法入口形态、唯一性判定、尾值/显式return、Unit返回值表示、退出码映射、诊断码，全部可观察、可反证。
- 让真实源码经 `词法/语法解析 → 绑定/推导 → Checked HIR → Typed MIR → ABC → 受检验证 → AV执行` 全链得到 typed 结果，不靠宿主硬编码按文件名/函数名挑结果。
- 为紧随其后的 863/864/865 提供公共前门：它们都要求“真实程序能进入、能返回、能判定”，不再各自发明入口规则。
- 保持 `D798`/`D828`/`D820`–`D826`/`D827`/`D852` 已交付合同不变；本Plan不改 852 分母既有行，只**追加**入口/Unit 相关case行。

成功标准：入口与结果通道可用独立oracle区分（改结果必红、错入口必拒、错通道必分），旧 profile 字节/行为零漂移。

## 2. 架构方案

按[Design43 §2/§4/§12.1](../design/43-ac-av-typed-mir-and-abc.md)的三层表示（Typed MIR → portable ABC → 私有LoadedProgram）扩一个**独立profile**，而不是在冻结 profile 上加字段：

| 层 | 本Plan动作 | 不变的边界 |
|---|---|---|
| 源码前端（ac-core `source/`） | 新 `SourceProfile` 变体 + 入口/Unit/尾值语法与推导 | scalar/core/text/record 四域支持集合不扩大、不自动获得新能力 |
| HIR 身份 | 新 descriptor + 精确可失败映射（未注册三元组整体拒绝） | 旧 descriptor 字节零改动 |
| Typed MIR | 新增 unit 值类型与 entry 声明类别（entry kind/signature 属于语义） | 旧 MIR schema/语义版本不变 |
| ABC | 新 profile id 与独立 schema/语义轴（建议 `ABC_PROFILE_*` 下一个值，具体号 T-02 冻结） | AVAB/AVAD v1–v5 字节语义不改 |
| AV 执行 | 结果通道：`Success{int}` / `Success{unit}` / `Trap` / `Resource` / `Cancelled` / `Infra` 保持现有枚举语义，只新增合法载荷形态 | 现有 `ExecOutcome` 分类不被“Unit当0”或“trap当Err”替代 |
| 消费者 | 既有 828 会话/受检安装/缓存接口复用，不新造平行会话 | `resident-function-install` 的安装/寿命规则 |

入口属于**模块级语义**：单函数不等于 unit，kind/entry/imports 的往返与准入必须由 wire 与 verifier 一致复核（[Design43 §12.1](../design/43-ac-av-typed-mir-and-abc.md)）。因此本Plan的 profile 扩展必须同时给出：类别字段、签名等价、拒绝码，以及在恢复路径（新进程/重验）中的同判定。

## 3. 技术栈

- 源码前端与HIR：`experimental/ac-core`（`src/source/{lexer,parser,resolve,typecheck,adapter,ast}.rs`、`src/hir.rs`、`src/descriptor.rs`、`src/verify.rs`），沿用既有 `compile_*_source` 适配器形态。
- MIR/ABC/AV：`experimental/ac-av-probe/core`（`mir.rs`、`mir_verify.rs`、`abc.rs`、`abc_verify.rs`、`av.rs`、`contract.rs`、`program_snapshot.rs`）与 `probe`（`hir_adapter.rs`、`session.rs`、`driver.rs`、`formal_worker.rs`）。
- 语料与判定：`crates/auto-lang/test/cases/ace-m1/`（`src/*.at`、`binding.tsv`、case 描述文件）、`crates/auto-lang/src/tests/ace_m1_tests.rs`、`scripts/verify-ace-m1-852.ps1`。
- 若需新脚本，拟建 `scripts/verify-av-862.ps1`（Scope=targeted/compat/entry/acceptance），测试前缀 `av_entry_862`；**当前不存在、未运行**，由 T-05 建立后再写实际命令。

唯一实施树：`D:/autostack/.wt/lang-862/auto-lang`，分支 `plan-862-dev`（Plan 529 分组布局）。主检出只保留计划簿记。

## 4. 需求分析与背景调查

用户 2026-10-28 授权：以 Design43 与ACE路线文档为基准，为826之后的ACE路线规划后继计划；本轮只规划，不实施、不建树、不跑门禁。已读并作为依据的文档：Design43、Design44、[ACE总入口 §0/§0.4](../design/strategy/auto-native-backend-evolution.md)、[AV源码与替换路线](../design/strategy/av-source-programs-and-avm-replacement-roadmap.md)（G0–G8、R2-*）、[能力矩阵](../design/strategy/av-language-capability-matrix.md)（C-06/C-29/SEM-05）、[版本/里程碑合同](reports/ace-milestones-version-boundary-2026-10-24.md)（M1–M4）、[852底座与能力验证合同](reports/ace-852-baseline-and-capability-validation-2026-10-26.md)。

现状事实（以current Spec与代码位置为据，非运行结果）：

- 源码侧已有四个具名 profile：`auto.source.{core-i32,scalar-i32,text,record,text-record}.draft`（[project](../specs/auto-ac/project.md)、[source-scalar-i32](../specs/auto-ac/source-scalar-i32.md)、[source-record](../specs/auto-ac/source-record.md)、[source-text](../specs/auto-ac/source-text.md)、[source-text-record](../specs/auto-ac/source-text-record.md)）。入口契约是**零参数、返回 i32 的已声明函数**（[auto-ac/project](../specs/auto-ac/project.md) 入口节），**没有 Unit 值、没有隐式尾返回、没有顶层脚本**（能力矩阵 C-06，852 binding `gap:mn-scalar-only` 与 G2a 缺口行）。
- AV 侧入口/结果同样是标量：`ExecOutcome::Success{value}` 承载值，`Trap/Resource/Cancelled/Infra` 分态；`av.core-i32/1` 的 AVAB 语义、`av.data|owned|collections|dynamic|closures` 的 AVAD v1–v5 均按各自 profile 冻结（[auto-av/project](../specs/auto-av/project.md)）。
- 852 已交付四路径一致性语料与父侧判定（[ace-m1-conformance](../specs/auto-test-case/ace-m1-conformance.md)）。T*/R*/L*/V*/K* 行的源码层缺口带owner登记，其中**入口/输出形态**属于横向前门，不是某个类型族的私有缺口。

SEM-05 的冲突（[能力矩阵 §3](../design/strategy/av-language-capability-matrix.md)）：旧源码有顶层脚本/Unit main/尾值和显示格式；新profile是零参 i32 + 显式 return。两者不能由一个“差不多”的映射合并——入口选择、脚本执行顺序、Unit/guest结果/OS exit/显示必须分别冻结，且不得按文件名挑结果（`SEM-05`）。本Plan只冻结**入口与结果通道**；显示/格式化属 863。

## 5. 详细设计

### 5.1 前置、入口与写面

硬前置：`D798`、`D828`、`D852`（已具备）；本Plan不等待 863–866。写面：

- `experimental/ac-core/src/source/{ast,lexer,parser,resolve,typecheck,adapter}.rs` + `src/{hir,descriptor,verify}.rs` + 新 schema `*.atom`（新域）。
- `experimental/ac-av-probe/core/src/{mir,mir_verify,abc,abc_verify,av,contract}.rs` + `probe/src/{hir_adapter,session,driver,formal_worker}.rs`。
- 语料/脚本：`crates/auto-lang/test/cases/ace-m1/**`（仅追加行与文件）、`scripts/verify-av-862.ps1`、`crates/auto-lang/src/tests/ace_m1_tests.rs`。
- 不动：旧 AutoVM/a2r/产品CLI/SDK默认入口、852 binding schema 12 列、既有 profile 的 descriptor/语义轴。

### 5.2 入口语义冻结（T-02 决策件）

T-02 必须产出 `attachments/862/work/entry-contract.md`，逐项冻结（未冻结不得进入 T-03）：

| 决策项 | 本Plan要求 |
|---|---|
| 合法入口形态 | `fn main() int`（显式return，等价现状）、`fn main() void`（无返回值）、单文件顶层语句序列（尾值表达式为结果）；三者互斥歧义时按明确规则拒绝，不静默择一 |
| 唯一性 | 每模块恰一个入口候选；多重 main/`main` 与非入口同名/重载形态 → 具名诊断，不按首个匹配 |
| 参数 | 入口零参数；带参数入口拒绝（argv 属 863 之后的宿主能力，不在本Plan） |
| 返回 | int 入口要求全路径返回（沿既有规则）；Unit 入口禁止 `return <value>`；尾值只在脚本/Unit语境生效，int 入口尾值不隐式转换 |
| 结果通道 | guest int → 值通道；Unit → 值与零占位**语义上不相等**（不得把 Unit 当 0 参与比较/算符）；trap/resource/cancel/infra 各自独立码 |
| 退出映射 | OS exit 由消费端决定：脚本/CLI 约定 int 原样、Unit=0、trap/resource 具名非零、infra 独立；该映射属于**消费者合同**，不能写进 AV 语义冒充语言结果 |
| 静态合法性 | 未执行分支、未调用函数同样检查（沿 scalar profile 纪律）；入口类型错误在 verify 期报，不落到运行期 |
| 版本 | 新源码 profile 与执行 profile 的 id/schema/语义版本三元组由 T-02 冻结并写入 SD；不复用旧号 |

### 5.3 MIR/ABC/AV 合同

- unit 值：MIR 增加 unit 结果类型（无运行时宽度）；调用/返回/link 的类型等价必须区分 `unit` 与 `i32`，禁止零宽冒充。
- entry 类别：ABC 模块头保留 entry kind（int/unit/script）+ 入口绑定；verifier 复核签名一致（kind-less 或错 kind 输入受控拒绝，不允许 fallback）。
- 执行：`execute_program` 在 Unit 入口成功时返回 typed unit 结果；`Success{int}` 与 `Success{unit}` 在消费端可区分；fuel/取消/资源预算不变。
- 恢复：新进程 decode/verify 后重跑同一入口判定；缓存/存储的 envelope 身份必须包含 profile/语义版本（不得以“字节可解析”替代准入）。
- 失败面：错 kind、双入口、签名不符、Unit 与值混用各自独立错误码；不允许“先执行合法前缀再报成功”。

### 5.4 与 863/864/865 的接缝

本Plan只发布“能被判定地进入与退出”。863 在其上加宿主导入（print），864/865 在其上加类型族语法。三者均消费本Plan的 `SourceProfile` 选择与结果通道，不另立入口规则；若 863 发现结果通道不足以承载 stdout 断言，只能扩**观察轴**，不得改本Plan已冻结的 guest 结果语义。

### 5.4 测试框架接入（既有 TestCase 协议，不新造框架）

语料仍是**文件化 case**：在 `crates/auto-lang/test/cases/` 追加（拟 `ace-m1` 新行 + 族 `av-entry-862`），保持“一份 source × 多 check”形态（同源可同时挂 ac-native / av / autovm / a2r）。协议侧只允许两处动作：

- **recipe 绑定**：入口/Unit 改变的是源码 profile 与 entry 类别。优先在既有 `ac-av/core-run-v1` 的 av binding **追加源 profile 选择字段**（append-only：旧 case 不写该字段时行为逐字节不变）；若该字段无法向后兼容，则按既有三缝新注册 `ac-av/entry-run-v1`（`REGISTERED_PROBE_RECIPES` + step 词表 / 父侧 `test_cases/mod.rs` 分派 / worker `case-serve` 域分支）。
- **观察通道**：本Plan只用既有 `exit`（值通道）与 `probe`（结构化步骤面）；**不新增 stdout/stderr**（归 863），也不改 `channel_supported` 表。

判定仍在父侧（worker 永不见 expected），`binding.tsv` 只追加行、不改 12 列 schema；新 case 与 oracle 由父侧独立裁决。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/source-entry-unit.md | 无该域 → 新源码 profile：入口形态/Unit/尾值/诊断矩阵/拒绝面 | 真实程序可进入 | AC-01/02 |
| SD-02 | add | docs/specs/auto-av/entry-and-unit-profile.md | 无该域 → 新执行 profile：unit类型/entry kind/结果通道/恢复准入 | 结果可判定 | AC-02/03 |
| SD-03 | modify | docs/specs/auto-av/project.md | 追加新 profile 条目，旧 profile 边界保持 | 模块可见 | AC-03 |
| SD-04 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 追加入口/Unit case 行与判定轴，不改既有行语义 | 证据落点 | AC-05 |
| SD-05 | modify | docs/specs/auto-mir/overview.md | 追加 unit 结果类型与 entry 类别的 MIR 侧约束 | IR 合同 | AC-02/03 |

## 6. 测试设计

实施自查 → 独立最终review → merge 门，唯一树、不分配新agent；完整档在最终review通过后由merge执行。

| V | AC/不变量 | 实际/拟建入口与输入 | 可区分错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 入口矩阵 | 新语料：int/Unit/脚本三形态各正例；双入口、带参、int入口尾值、Unit返回带值、缺返回 | 每格具名诊断码；合法/非法集合互不扩大；变异任一格必红 | T-03；仅入口面 |
| V-02 | AC-02 结果通道 | 同一程序分别产出 int、Unit、trap、resource、cancel；含未执行分支与未调用函数 | 通道可区分（Unit≠0；trap≠值；resource≠infra）；把 Unit 当 0 的变异必红 | T-04；预算/取消沿用 |
| V-03 | AC-03 旧域保持 | 旧 scalar/core 语料重跑 + 新域文档喂旧 verify 入口、旧域文档喂新入口 | 旧 profile 字节/结果零漂移；双向 `verify.domain`/`verify.profile-op` 拒绝 | T-04 |
| V-04 | AC-04 全链与恢复 | 真实 `.at` 文件 → 前端 → MIR → ABC → AV；新进程 decode/verify 后重跑；错 kind/双入口 wire 手术 | 入口 kind 与签名一致性在恢复后同判定；坏输入受控拒绝 | T-04；正式入口由 T-05 建立 |
| V-05 | AC-05 语料绑定 | 852 语料追加入口/Unit 行 + 独立 oracle（手算/规格），父侧判定 | 改 expected 必红、SKIP 不计绿、分母不缩 | T-05 |

定向命令（T-05 建立后填写实际结果）：`cargo check -p auto-lang`；`experimental/ac-core` 与 `experimental/ac-av-probe` 两独立 workspace 的定向 `cargo test`；`powershell -File scripts/verify-av-862.ps1 -Scope <targeted|compat|entry|acceptance>`。完整 `cargo t`/`tv` 与 852 acceptance 全量属 merge 门，不在work阶段反复跑。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 入口矩阵按冻结合同成立：三形态正例全过，非法面全部具名拒绝；V-01 |
| AC-02 | work | Unit/int/trap/resource/cancel/infra 通道可区分且无冒充；V-02 |
| AC-03 | work | 旧 profile 字节/语义零漂移，新域与旧域双向拒绝；V-03 |
| AC-04 | work | 真实源码全链 + 新进程恢复后同判定 + 坏输入拒绝；V-04 |
| AC-05 | work | 852 语料追加行与独立 oracle 落地，分母不缩、SKIP 不计绿；V-05 |
| AC-06 | review | 独立核 work 的 AC/V/SD、入口冻结件与实际数据流、warnings/fmt；merge 门 pending |
| AC-07 | merge | 冻结提交上跑裸 `cargo t` + 触面档（VM/编译器→`tv`）；失败不豁免 |
| AC-08 | merge | Spec 沉淀（SD-01..05）、ledger、归档、worktree guard 清理实际完成 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 无 | 核实现状：读 ac-core 四 profile 的真实入口/类型规则、ac-av 入口 kind/结果通道、852 语料行；产出 `attachments/862/work/baseline.md`（文件+行号+未验证假设） | AC-01 | [ ] |
| T-02 | work | T-01 | 冻结 `entry-contract.md`：入口矩阵/Unit表示/结果通道/退出映射/profile 三元组；冲突项列用户决定 | AC-01/02 | [ ] |
| T-03 | work | T-02 | 实现源码域：lexer/parser/resolve/typecheck/adapter + descriptor + 正反诊断，定向测试 V-01 | AC-01 | [ ] |
| T-04 | work | T-03 | 实现 MIR/ABC/AV：unit 类型、entry kind、结果通道、verifier 与恢复准入，测试 V-02/03/04 | AC-02/03/04 | [ ] |
| T-05 | work | T-04 | 语料/脚本/绑定：追加 ace-m1 行、独立 oracle、`verify-av-862.ps1`，跑定向面并提交代码与证据 | AC-05 | [ ] |
| T-06 | review | T-05 | 独立最终review，核 AC/V/SD 与实际入口；merge 门 pending | AC-06 | [ ] |
| T-07 | merge | T-06 pass | 冻结提交上裸 `cargo t` + `tv`，失败归因不豁免 | AC-07 | [ ] |
| T-08 | merge | T-07 | Spec/ledger 沉淀、归档、worktree guard clean 清树 | AC-08 | [ ] |

## 9. 复审记录

2026-10-28 new r1：`stage: new | plan_id: PLAN-862 | plan_revision: 1 | outcome: pass（可在既有授权内进入 work；T-02 若需改变已批准语义则回用户决定） | next: work T-01→T-02`。全部 AC/V/T 未执行；profile 号、schema 名、诊断码均为建议值，T-02 冻结后写回。本轮未建实施树、未跑任何构建/语料，未改在途Plan与既有Spec。批次排序依据见[后继批次规划](reports/ace-next-batch-2026-10-28.md)。

## 10. 待澄清事项

- SEM-05 三形态（int main / Unit main / 顶层脚本）是否**同时**进入首批，还是先交付前两种、脚本形态另立切片。若用户要求脚本形态同批，须确认“脚本执行顺序与副作用可见点”的oracle由谁裁定。
- 显示/格式化（旧源码的打印格式）是否需要在 863 之前先定；本Plan默认把全部显示面推给 863。
- 是否要求本 profile 同时进入 AC native 路径（`gap:ac-composite-native`/`mn` 一侧）。默认**不要求**：native 复合扩面另立计划，本Plan只交付 AV 通道。
