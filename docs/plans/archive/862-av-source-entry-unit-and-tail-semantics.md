---
plan_id: PLAN-862
status: archived
feature_name: AV源码入口、Unit与尾值语义（R2-E/SEM-05）
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 3

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-av/project.md
  - docs/specs/auto-test-case/ace-m1-conformance.md
  - docs/specs/auto-mir/overview.md
new_spec_components:
  - docs/specs/auto-ac/source-entry-unit.md
  - docs/specs/auto-av/entry-and-unit-profile.md
touched_goals: [GOAL-002]   # 解释器/VM 快周转：热重载与真实源码执行底座

affects: [auto-av, auto-mir, auto-ac, auto-test-case]
current_step: 9
total_steps: 9
---

# [PLAN-862] AV源码入口、Unit与尾值语义

## 0. 变更摘要

本Plan是M1核心骨架成型后的第一批源码扩面切片之一，领取[AV源码与替换路线](../../design/strategy/av-source-programs-and-avm-replacement-roadmap.md) **R2-E**中“入口/Unit/尾值/退出通道”部分（`G2a` 前门），并冻结能力矩阵的 **SEM-05**：

- 交付新的具名源码 profile 与执行 profile，使真实Auto程序可以 `fn main() void`、隐式尾值、单文件脚本体进入AV执行，并把“guest返回值 / Unit / 语言trap / 资源失败 / infra”呈现为**互不替代的独立通道**。
- 不改动 `auto.source.scalar-i32.draft` r1 与 `av.core-i32/1`（profile id 1 / AVAB 语义）的既有字节与语义；新域独立版本、双向拒绝。
- 本Plan **不含** 控制台/print（PLAN-863）、集合（864）、闭包（865）、在线热重载（866）。
- **并入项（2026-10-28 用户裁定）**：本Plan同时承担 **M1 分母重绑定**（把对账表里"从未真跑过"的格子实测定级），不另立计划。口径与 103 行逐行映射见[对等口径决策件](../reports/ace-m1-scope-decision-2026-10-28.md)（已裁定：`na:m1` 三类接受、`diff:accepted` 四族接受、a2r 双轴拆分接受、a2r 名解析计真缺口）。

本轮只起草合同：不建实施树、不跑构建/语料、不启动其他会话，也不改任何在途Plan的合同或进度。

## 1. 目标

- 冻结一套**入口选择与结果通道**语义矩阵：合法入口形态、唯一性判定、尾值/显式return、Unit返回值表示、退出码映射、诊断码，全部可观察、可反证。
- 让真实源码经 `词法/语法解析 → 绑定/推导 → Checked HIR → Typed MIR → ABC → 受检验证 → AV执行` 全链得到 typed 结果，不靠宿主硬编码按文件名/函数名挑结果。
- 为紧随其后的 863/864/865 提供公共前门：它们都要求“真实程序能进入、能返回、能判定”，不再各自发明入口规则。
- 保持 `D798`/`D828`/`D820`–`D826`/`D827`/`D852` 已交付合同不变；本Plan对 852 分母的动作限于：**追加入口/Unit 新行** + **按已裁定口径重绑既有 `probe` 单元格**（不改任何 expected/oracle，不改 12 列 schema）。

成功标准：入口与结果通道可用独立oracle区分（改结果必红、错入口必拒、错通道必分），旧 profile 字节/行为零漂移。

## 2. 架构方案

按[Design43 §2/§4/§12.1](../../design/43-ac-av-typed-mir-and-abc.md)的三层表示（Typed MIR → portable ABC → 私有LoadedProgram）扩一个**独立profile**，而不是在冻结 profile 上加字段：

| 层 | 本Plan动作 | 不变的边界 |
|---|---|---|
| 源码前端（ac-core `source/`） | 新 `SourceProfile` 变体 + 入口/Unit/尾值语法与推导 | scalar/core/text/record 四域支持集合不扩大、不自动获得新能力 |
| HIR 身份 | 新 descriptor + 精确可失败映射（未注册三元组整体拒绝） | 旧 descriptor 字节零改动 |
| Typed MIR | 新增 unit 值类型与 entry 声明类别（entry kind/signature 属于语义） | 旧 MIR schema/语义版本不变 |
| ABC | 新 profile id 与独立 schema/语义轴（建议 `ABC_PROFILE_*` 下一个值，具体号 T-02 冻结） | AVAB/AVAD v1–v5 字节语义不改 |
| AV 执行 | 结果通道：`Success{int}` / `Success{unit}` / `Trap` / `Resource` / `Cancelled` / `Infra` 保持现有枚举语义，只新增合法载荷形态 | 现有 `ExecOutcome` 分类不被“Unit当0”或“trap当Err”替代 |
| 消费者 | 既有 828 会话/受检安装/缓存接口复用，不新造平行会话 | `resident-function-install` 的安装/寿命规则 |

入口属于**模块级语义**：单函数不等于 unit，kind/entry/imports 的往返与准入必须由 wire 与 verifier 一致复核（[Design43 §12.1](../../design/43-ac-av-typed-mir-and-abc.md)）。因此本Plan的 profile 扩展必须同时给出：类别字段、签名等价、拒绝码，以及在恢复路径（新进程/重验）中的同判定。

## 3. 技术栈

- 源码前端与HIR：`experimental/ac-core`（`src/source/{lexer,parser,resolve,typecheck,adapter,ast}.rs`、`src/hir.rs`、`src/descriptor.rs`、`src/verify.rs`），沿用既有 `compile_*_source` 适配器形态。
- MIR/ABC/AV：`experimental/ac-av-probe/core`（`mir.rs`、`mir_verify.rs`、`abc.rs`、`abc_verify.rs`、`av.rs`、`contract.rs`、`program_snapshot.rs`）与 `probe`（`session.rs`、`driver.rs`、`formal_worker.rs`），以及D868后的`experimental/auto-mir-lower/src/**`。
- 语料与判定：`crates/auto-lang/test/cases/ace-m1/`（`src/*.at`、`binding.tsv`、case 描述文件）、`crates/auto-lang/src/tests/ace_m1_tests.rs`、`scripts/verify-ace-m1-852.ps1`。
- 若需新脚本，拟建 `scripts/verify-av-862.ps1`（Scope=targeted/compat/entry/acceptance），测试前缀 `av_entry_862`；**当前不存在、未运行**，由 T-05 建立后再写实际命令。

唯一实施树：`D:/autostack/.wt/lang-862/auto-lang`，分支 `plan-862-dev`（Plan 529 分组布局）。主检出只保留计划簿记。

## 4. 需求分析与背景调查

用户 2026-10-28 授权：以 Design43 与ACE路线文档为基准，为826之后的ACE路线规划后继计划；本轮只规划，不实施、不建树、不跑门禁。已读并作为依据的文档：Design43、Design44、[ACE总入口 §0/§0.4](../../design/strategy/auto-native-backend-evolution.md)、[AV源码与替换路线](../../design/strategy/av-source-programs-and-avm-replacement-roadmap.md)（G0–G8、R2-*）、[能力矩阵](../../design/strategy/av-language-capability-matrix.md)（C-06/C-29/SEM-05）、[版本/里程碑合同](../reports/ace-milestones-version-boundary-2026-10-24.md)（M1–M4）、[852底座与能力验证合同](../reports/ace-852-baseline-and-capability-validation-2026-10-26.md)。

现状事实（以current Spec与代码位置为据，非运行结果）：

- 源码侧已有四个具名 profile：`auto.source.{core-i32,scalar-i32,text,record,text-record}.draft`（[project](../../specs/auto-ac/project.md)、[source-scalar-i32](../../specs/auto-ac/source-scalar-i32.md)、[source-record](../../specs/auto-ac/source-record.md)、[source-text](../../specs/auto-ac/source-text.md)、[source-text-record](../../specs/auto-ac/source-text-record.md)）。入口契约是**零参数、返回 i32 的已声明函数**（[auto-ac/project](../../specs/auto-ac/project.md) 入口节），**没有 Unit 值、没有隐式尾返回、没有顶层脚本**（能力矩阵 C-06，852 binding `gap:mn-scalar-only` 与 G2a 缺口行）。
- AV 侧入口/结果同样是标量：`ExecOutcome::Success{value}` 承载值，`Trap/Resource/Cancelled/Infra` 分态；`av.core-i32/1` 的 AVAB 语义、`av.data|owned|collections|dynamic|closures` 的 AVAD v1–v5 均按各自 profile 冻结（[auto-av/project](../../specs/auto-av/project.md)）。
- 852 已交付四路径一致性语料与父侧判定（[ace-m1-conformance](../../specs/auto-test-case/ace-m1-conformance.md)）。T*/R*/L*/V*/K* 行的源码层缺口带owner登记，其中**入口/输出形态**属于横向前门，不是某个类型族的私有缺口。

SEM-05 的冲突（[能力矩阵 §3](../../design/strategy/av-language-capability-matrix.md)）：旧源码有顶层脚本/Unit main/尾值和显示格式；新profile是零参 i32 + 显式 return。两者不能由一个“差不多”的映射合并——入口选择、脚本执行顺序、Unit/guest结果/OS exit/显示必须分别冻结，且不得按文件名挑结果（`SEM-05`）。本Plan只冻结**入口与结果通道**；显示/格式化属 863。

本次授权与输入：用户明确要求修订波次/实际写面、补871旧能力覆盖及删除门、让864支持List与拥有型type元素；允许动作仅计划/设计/战略文档修订，不实施代码、不调整预算、不改868。基面路径、内容SHA-256、旧稿快照及静态预检见[修订收据](../attachments/ace-wave-20261028/revision-summary.md)；观察到的current Spec/调用点与未来拟建入口分开记录。

## 5. 详细设计

### 5.0 波次、实际写面与交接（本次修订）

按[波次交接合同](../reports/ace-wave-handoff-2026-10-28.md)§1–2执行：S为共享source/HIR/lowering/MIR/ABC/AV及会话接口，N为ac-mn，H为公共driver/SDK适配/测试注册与binding；分别单写，跨窗同时取窗。共享lowering的未来写面为D868后的`experimental/auto-mir-lower/src/**`，旧`probe/src/hir_adapter.rs`仅作搬前调查，不能再落新实现。T-02及接线前核验实际已合入revision、Spec/API/版本、owner释放与消费者兼容证据并记录接受；review/repair未完不交WIP。

第1波本Plan占S并按需占H；D868前T-01/T-02仅只读调查/合同准备，T-03代码实施必须等D868与S释放。870 T-01..04可在N并行；本Plan先完成Unit/入口与分母重绑定交付，释放H后870 T-05再接公共差分驱动/binding。新增MIR类型必须同步检查既有mn消费者：不支持的新profile受控拒绝，旧profile仍可编译/执行，不把编译适配当native Unit能力。

首例即按既有文件化AutoDown/TestCase、cache fresh/hit/restart/body-edit、未改函数生产工作=0与现行P90≤2000ms口径验证；亚秒目标及P95/P99另列，不能伪称当前硬门已是亚秒。内存/资源峰值、live/回收及cache容量在T-02沿上游预算交集冻结，记录命令、样本和反控，不擅自提高上限。共享核心/SDK/测试迁移先确认拆分owner实际位置与释放；产品默认入口、旧AutoVM/a2r和868合同/进度不在本次修订写面。

若新增公开枚举使mn必须修改匹配臂，T-02登记N短窗，由870暂停并交接；862在同时具备S/N窗口时完成最小拒绝/兼容适配，释放N后870恢复。不得把编译失败留给并行owner，也不得暗改870的N文件。

### 5.1 前置、入口与写面

硬前置：`D798`、`D828`、`D852`（已具备）、**`D868`（HIR→MIR lowering 出库）**——本Plan要改 MIR 入口/Unit 的 lowering，必须落在共享 crate 而非 probe 宿主；`D868` 未交付前只做只读核查与合同冻结（T-02 可先做）。本Plan不等待 863–866。写面：

- `experimental/ac-core/src/source/{ast,lexer,parser,resolve,typecheck,adapter}.rs` + `src/{hir,descriptor,verify}.rs` + 新 schema `*.atom`（新域）。
- `experimental/auto-mir-lower/src/**` + `experimental/ac-av-probe/core/src/{mir,mir_verify,abc,abc_verify,av,contract}.rs` + `probe/src/{session,driver,formal_worker}.rs`。
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

判定仍在父侧（worker 永不见 expected），`binding.tsv` **只追加行或翻转状态列**、不改 12 列 schema；新 case 与 oracle 由父侧独立裁决。

### 5.5 M1 分母重绑定（并入项，2026-10-28 用户裁定）

**要解决的问题**：`binding.tsv` 里 24 个 `probe` 与 27 个 `wait:*` 从未真实执行过（`wait` 指向的 822 未获授权新增 T-11），所以"M1 干净行数"一直是估算。本Plan用**同一次语料运行**把它们定级。

范围与边界：

- **只定级，不补能力**：能跑的如实写 `ok`；仍缺能力的写 `gap:<owner>`（按决策件 §7 的 owner 表）；契约已备但尚未实跑的不许写 `ok`。
- **不改 expected/oracle**：任何"跑出来跟冻结期望不一致"的结果 → 记录差异并停手，不许改期望凑绿（这是既有纪律）。
- **不改 12 列 schema**：新 token（`ok+cert-gap`、`na:m1(rule)`、`diff:accepted(rule)`）写在状态列；`na` 必须带规则 ID（NA-1..3），新增 `na` 需决策依据。
- **不加分母、不删分母**：只翻转既有单元格状态 + 追加本Plan的入口/Unit 行。
- **前置**：`D868`（lowering 版本身份进报告，否则重绑定证据一落地就被下次迁移判 stale）；`wait:*` 的定级需要其指向能力已交付的，按实际交付判断（未交付就转 `gap:<owner>`，不硬跑）。
- **产出**：`attachments/862/rebind/` 下次级证据摘要（命令、退出码、逐族结果、hash）+ `binding.tsv` 状态更新 + 一句结论："clean = N/103（实测）"。

**不做的**：不把 `na` 当垃圾桶（决策件 §5 四条反滥用规则在 review 时逐条核）；不顺手修 a2r/旧引擎；不重跑已判 `ok` 的行去"求一致"。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/source-entry-unit.md | 无该域 → 新源码 profile：入口形态/Unit/尾值/诊断矩阵/拒绝面。**T-07 F-862-R1-02/F-04 增补**：① 结果位以外写 `void`（参数/局部）命名 `source.unit-placement`（可达并已测；entry-unit 面没有 `type` 声明，字段条款由 HIR/MIR 层守护）；② 入口矩阵另有两个码 `source.entry-order`（脚本序列后不得再有 item）与 `source.entry-result`（入口结果位规则），须一并入册；③ Unit 的 source/HIR/MIR 三层拒绝面与各自可达性 | 真实程序可进入 | AC-01/02 |
| SD-02 | add | docs/specs/auto-av/entry-and-unit-profile.md | 无该域 → 新执行 profile：unit类型/entry kind/结果通道/恢复准入 | 结果可判定 | AC-02/03 |
| SD-03 | modify | docs/specs/auto-av/project.md | 追加新 profile 条目，旧 profile 边界保持 | 模块可见 | AC-03 |
| SD-04 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 追加入口/Unit case 行（U01–U10）与判定轴（`av_channel` = value/unit；`unit` 与 `exit` 互斥），不改既有行语义 | 证据落点 | AC-05 |
| SD-05 | modify | docs/specs/auto-mir/overview.md | 追加 unit 结果类型与 entry 类别的 MIR 侧约束（`mir.unit-placement`：Unit slot 仅作 unit 入口的 return 操作数；`mir.entry-kind` 双向一致） | IR 合同 | AC-02/03 |
| SD-06 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 追加五态词表与 `na:m1(rule)`/`diff:accepted(rule)`/`ok+cert-gap` 判定（实现决策件 SD-01/02）；**T-07 F-862-R1-01/F-04 增补**：① 新规则 **NA-4**——入口/Unit 是 AV-only 执行 profile，native/legacy 不要求判它（2026-10-28 用户复审期间裁定；重新进入条件：有计划冻结 native/legacy 对该 profile 的支持时逐行改回 `gap:<plan>`）；② 本轮新增 owner token `a2r-golden`/`scenario-tooling`/`av-text-move`/`av-text-nul`/`av-mut-param`/`P864`；③ `probe`/`wait:*` 已清零、clean 以实测 20/73 记；不改 12 列 schema | 重绑定需可判口径 | AC-06 |

## 6. 测试设计

实施自查 → 独立最终review → merge 门，唯一树、不分配新agent；完整档在最终review通过后由merge执行。

| V | AC/不变量 | 实际/拟建入口与输入 | 可区分错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 入口矩阵 | 新语料：int/Unit/脚本三形态各正例；双入口、带参、int入口尾值、Unit返回带值、缺返回 | 每格具名诊断码；合法/非法集合互不扩大；变异任一格必红 | T-03；仅入口面 |
| V-02 | AC-02 结果通道 | 同一程序分别产出 int、Unit、trap、resource、cancel；含未执行分支与未调用函数 | 通道可区分（Unit≠0；trap≠值；resource≠infra）；把 Unit 当 0 的变异必红 | T-04；预算/取消沿用 |
| V-03 | AC-03 旧域保持 | 旧 scalar/core 语料重跑 + 新域文档喂旧 verify 入口、旧域文档喂新入口 | 旧 profile 字节/结果零漂移；双向 `verify.domain`/`verify.profile-op` 拒绝 | T-04 |
| V-04 | AC-04 全链与恢复 | 真实 `.at` 文件 → 前端 → MIR → ABC → AV；新进程 decode/verify 后重跑；错 kind/双入口 wire 手术 | 入口 kind 与签名一致性在恢复后同判定；坏输入受控拒绝 | T-04；正式入口由 T-05 建立 |
| V-05 | AC-05 语料绑定 | 852 语料追加入口/Unit 行 + 独立 oracle（手算/规格），父侧判定 | 改 expected 必红、SKIP 不计绿、分母不缩 | T-05 |
| V-06 | AC-06 分母重绑定 | 24 个 `probe` + 27 个 `wait:*` 逐格实跑；`na`/`gap` 取值对照决策件规则 | 每格有真实执行证据或具名 owner；`na` 必带规则 ID；SKIP/未跑计入阻塞；`clean` 数字与实际逐格统计一致 | T-06；不改 expected |
| V-07 | AC-04/AC-07 写面/兼容 | T-02及公共接线交接摘要；已交付消费者并集与窗口清单 | 真实D门/释放/接受齐全；缺交接或消费WIP不准接线；受影响profile兼容验证及首例cache/周转/内存证据齐全 | T-02冻结；最终work任务执行；review复核 |

定向命令（T-05 建立后填写实际结果）：`cargo check -p auto-lang`；`experimental/ac-core` 与 `experimental/ac-av-probe` 两独立 workspace 的定向 `cargo test`；`powershell -File scripts/verify-av-862.ps1 -Scope <targeted|compat|entry|acceptance>`。完整 `cargo t`/`tv` 与 852 acceptance 全量属 merge 门，不在work阶段反复跑。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 入口矩阵按冻结合同成立：三形态正例全过，非法面全部具名拒绝；V-01 |
| AC-02 | work | Unit/int/trap/resource/cancel/infra 通道可区分且无冒充；V-02 |
| AC-03 | work | 旧 profile 字节/语义零漂移，新域与旧域双向拒绝；V-03 |
| AC-04 | work | 真实源码全链 + 新进程恢复后同判定 + 坏输入拒绝；V-04 ；首例cache/文件化测试、周转/内存预算及兼容证据齐全，V-07 |
| AC-05 | work | 852 语料追加行与独立 oracle 落地，分母不缩、SKIP 不计绿；V-05 |
| AC-06 | work | 24+27 个未跑格逐格定级：证据/owner 齐全，`na` 带规则 ID，`clean` 为实测且与逐格统计一致；V-06 |
| AC-07 | review | 独立核 work 的 AC/V/SD、入口冻结件与实际数据流、`na` 反滥用四条、warnings/fmt；merge 门 pending ；核V-07交接与首例指标 |
| AC-08 | merge | 冻结提交上跑裸 `cargo t` + 触面档（VM/编译器→`tv`）；失败不豁免 |
| AC-09 | merge | Spec 沉淀（SD-01..06）、ledger、归档、worktree guard 清理实际完成 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 无 | 核实现状：读 ac-core 四 profile 的真实入口/类型规则、ac-av 入口 kind/结果通道、852 语料行；产出 `attachments/862/work/baseline.md`（文件+行号+未验证假设） | AC-01 | [x] |
| T-02 | work | T-01 | 冻结 `entry-contract.md`：入口矩阵/Unit表示/结果通道/退出映射/profile 三元组；冲突项列用户决定；冻结fresh/hit/restart/body-edit、周转/内存口径与反控 | AC-01/02 | [x] |
| T-03 | work | T-02、D868、S释放/接受 | 实现源码域：lexer/parser/resolve/typecheck/adapter + descriptor + 正反诊断，定向测试 V-01（**T-03a 前端半 + T-03b 发射/HIR 半均已交付**，见 §9） | AC-01 | [x] |
| T-04 | work | T-03 | 实现共享lowering/MIR/ABC/AV：unit、entry kind、结果通道、恢复；V-02/03/04；核既有mn消费者受控拒绝新域、旧域保持，记录交接（**已交付**，见 §9） | AC-02/03/04 | [x] |
| T-05 | work | T-04 | 语料/脚本/绑定：追加 ace-m1 行、独立 oracle、`verify-av-862.ps1`，跑定向面并提交代码及证据摘要（原始证据仅本地）；**T-07 R1 已修**（F-862-R1-02/03/05/06/07；修复轮 `e062edf4e`，关闭证据见 §9） | AC-05 | [x] |
| T-06 | work | T-05、D868 | M1 分母重绑定：实跑 24 个 `probe` + 27 个 `wait:*` 逐格定级、按决策件写状态、产出证据摘要与实测 `clean` 数；不改 expected/schema；执行V-07首例cache/文件化/周转/内存与兼容验证；**T-07 R1 已修**（F-862-R1-01/04/08；NA-4 已入 SD-06+决策件 §12，内存腿已实测） | AC-06/04 | [x] |
| T-07 | review | T-06 | 独立最终review，核 AC/V/SD 与实际入口、`na` 反滥用；merge 门 pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交上裸 `cargo t` + `tv`，失败归因不豁免；**已执行**（`0e62d8c4c` 重基基线）：`cargo t --no-fail-fast` **3199/3199**、`cargo tv` **163/163**；fail-fast 轮的唯一红为 867 预存红 `launcher_pickcat`（master 已由 867 T-12 修复）+ 4 个并行负载超时（隔离全绿），最终全量无红 | AC-08 | [x] |
| T-09 | merge | T-08 | Spec/ledger 沉淀、归档、worktree guard clean 清树；**已完成**：ledger v119 `61a5b904f`、归档 `2012b33dc`、三棵 worktree guard clean 后移除、分支与组目录删除（见 §9 cleaned） | AC-09 | [x] |

## 9. 复审记录

2026-10-28 **merge T-08/T-09（delivered；archived；cleaned）**：`stage: merge | plan_id: PLAN-862 | plan_revision: 3 | outcome: pass | delivery_commit: 0e62d8c4cd33dd5b29a074690107601dc57dbe07 | base_commit: 13057957c6363c87f63bb2e2edc2d36ad3ad773a | rebase_map: d116ac82e→2a7bf0352→0e62d8c4c（reviewed e062edf4e→9d7604858→e9bca9472；两跳 git range-diff 均全等） | dependency_revisions: auto-down c3464d0、auto-ui f489c7b（只读，未写） | ledger: v118→v119（guarded offline projection；备份 tmp/862/specs.before.json，sha256 5005dfd5700e5d3e77df507448c2eae84d78910f431a8cb2b7b1393c01cd0e76） | archive: docs/plans/archive/862-av-source-entry-unit-and-tail-semantics.md | next: 无（archived 终态）`。

- **prepared**：冻结基线 `reviewed e9bca9472`（= e062edf4e 重基后）→ 规范增量按 SD-01..06 落在 worktree（5 个 docs/specs 目标：新增 `auto-ac/source-entry-unit.md`、`auto-av/entry-and-unit-profile.md`；改 auto-ac/auto-av/auto-mir/auto-test-case 的 project/overview/plans 索引），形成 docs-only 后继 `0e62d8c4c`；`docs/specs/INDEX.md` 由 `scripts/spec-index.py` 复算为无变化（内容逐行相同）。实现/依赖未变（`git diff e062edf4e..0e62d8c4c --name-only` 仅 docs/specs）。
- **landed**：master `git merge --ff-only plan-862-dev` → tip **0e62d8c4c**（无 merge commit）。期间其他会话两次推进 master（867/878：仅 docs/specs + `.autoos/specs.json` + docs，**零代码文件**，`git diff 8ae51d6af 51415cd8e --name-only | grep -v ^docs/` 仅 `.autoos/specs.json`），故两跳重基不改变代码基线；两跳 `git range-diff` 全等（7/7 提交 `=`）。
- **merge 门（在重基后冻结基线上实跑）**：`cargo t --no-fail-fast` → **3199/3199 passed, 0 failed**（此前 fail-fast 轮出现的 1 红 = 867 的 `plan503_tests::launcher_rewrite::launcher_pickcat_filters_results`（F-867-NEWRED-01，master 上已由 867 T-12 `ec1d19a10` 修复）与 4 个 120s 超时（ac_cte_probe 3 + ac_av_probe 1；隔离重跑 5/5 与 1/1 全绿 = 并行负载超时，非回归），最终全量轮无红）；`cargo tv` → **163/163 passed**。基线身份：代码树 = 8ae51d6af + 本计划 7 提交（878 仅文档/ledger，不改代码）。
- **ledger_refreshed**：`D:/autostack/auto-lang/.autoos/specs.json` v118→**v119**；guarded offline projection（Regime B；先备份、后手术式插入、读回校验）：designs +2（`P862-DESIGN-SOURCE-ENTRY-UNIT`、`P862-DESIGN-ENTRY-UNIT-PROFILE`）、tests +1（`P862-TESTS`）、reviews +1（`P862-REVIEW-R2`）、reports +1（`P862-MERGE`）；并按"复用既有当前知识项"更新 `P852-DESIGN-CONFORMANCE` 内容（不复制目标文件）；读回校验：version 118→119、section id 不变、无重复/缺失 id、其余条目逐项 byte-identical；`git diff --numstat` = 130 insertions / 2 deletions；提交 `61a5b904f`，投影后文件 sha256 `17cec18aa1685a5bd36a6a7d0c73b6e76837a560dd74c403f83fcad0cac4923f`。**通道说明**：本会话无 spec 工具/HTTP store（Regime B），且在 worktree 内改 ledger 会以 v115 旧基覆盖 878 的 v118 投影，故投影在主检出 master 上以单通道执行并记录（备份 `tmp/862/specs.before.json`，sha256 见下）。
- **archived**：计划移入 `docs/plans/archive/`，frontmatter `status: archived`；链接按 archive 位置重写（`../`→`../../`、`x/`→`../x/`）并在真实父目录逐条校验；T-01..T-09 勾闭（cleanup 见下条 `cleaned`）。
- **cleaned**：移除前三棵 worktree 均过 wt-guard（本会话 bash 为 WSL，按 /mnt/d/... 调用并置 MSYS_NO_PATHCONV=1）：uto-lang/uto-down/uto-ui 全部 **clean**（无 reparse point）；随后 git worktree remove 三棵、git branch -d plan-862-dev（was 0e62d8c4c，已合入）、删除空组目录 D:/autostack/.wt/lang-862。中间产物：D:/autostack/plan-artifacts/plan-862 不存在（无可回收）；worktree 内 	arget/（约 15GB）随 worktree 一并移除；组内无 scratch-*/evidence 检出。终态核验：master tip 2012b33dc，git worktree list 无 862，git branch --list 'plan-862*' 为空，组目录不存在，delivery  e62d8c4c 为 master 祖先。
- **production artifacts**：本计划改动全在 `experimental/**`（ac-core/ac-av-probe/auto-mir-lower 等独立 workspace）与 `auto-test-case`+`auto-lang-core` 的 test-cases 门内测试面（`test_cases::ac_av`）——产品 `auto` 默认二进制、UI bundle 与 release 产物不链接这些路径，**无生产制品需要重建**；`target/debug/auto.exe` 语义不受影响。
- **batch regression due check**：`.last-batch-regression.json` 覆盖 860/`covered_commit 5613541d9`/2026-10-28T09:33Z；本计划 id%5=2 非触发；其后归档的 867/876/877/878 均非 5 倍数（865 未落地），收据龄 ~3h < 48h → **不 due**，未跑 regress。
- **遗留**：D-862-R1-01（852 验证脚本装载器预存缺陷，`KNOWN-DEBT-AND-RISKS.md`）、决策件附录无规则 ID `na:m1` 行的全表回填待裁定（B2）、a2r 新金样（a2r 线）——均已在计划内登记，不阻塞本次交付。

2026-10-28 **re-review T-07（R2，final pass）**：`stage: review | review_scope: final（re-review） | reviewer_identity/limitations: 同一实施会话复审，无独立外部/异模型上下文；结论由修复 diff（e062edf4e）、关闭行与在冻结提交上重跑的门重建 | plan_id: PLAN-862 | plan_revision: 3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-862/auto-lang | branch: plan-862-dev | reviewed_commit: e062edf4e53db111a08fb2a3a511acaf5299996b（树 clean） | base_commit: 13057957c6363c87f63bb2e2edc2d36ad3ad773a | dependency_revisions: auto-down c3464d0、auto-ui f489c7b（只读） | spec_inputs: 计划 R1 基线 sha256 332e7afa…（R2 仅改 §5 SD-01/04/05/06 文本与 §9 记录，随本轮簿记提交冻结）、决策件新增 §12、entry-contract b364c10b…、ace-m1-conformance 976dfd60…（仍未改 canonical specs） | acceptance_results: AC-01 pass、AC-02 pass、AC-03 pass（双向身份拒绝已测）、AC-04 pass（内存腿已实测）、AC-05 pass、AC-06 pass（NA-4 已裁定并落 SD-06/决策件） | pending_closeout_gates: T-08 merge 裸 cargo t + tv（diff 触及前端/MIR/ABC/AV）；T-09 Spec/ledger/归档/清树；na 分布须在 852/862 acceptance 报告单列 | findings: F-862-R1-01..08 全部 CLOSED（无新增 finding） | next: merge T-08`。

- **修复复现与关闭（本轮实际执行）**：① F-02 的红/绿对照——修复前同一输入返回 `source.unsupported`（R1 探针 `tmp/862/probe/`），修复后返回冻结码 `source.unit-placement`（新增 2 条负例 + 1 条 MIR 负例，`entry_unit_862` 26 passed、`av_entry_862` 13 passed）；② F-03 两条身份负例实跑绿（`verify.domain` / `mir.schema-profile`）；③ F-07 的 4 条准入负例落库（`contract_decode` 23 passed，含 `unit`+exit 0/1、未知 profile、非 core recipe）；④ F-08 采样：entry-unit 331,776–557,056 B、scalar 控制 45,056 B（`--test-threads=1 --nocapture`）；⑤ F-06 `rustfmt --check` 9 文件 exit 0；⑥ F-01/F-04 文本落地（SD-01/04/05/06 + 决策件 §12，NA-4 三条防扩散）。
- **共享面回归（parser.rs 被所有 ac-core 源面共用，按受影响模块扩面）**：`experimental/ac-core` 全量 **146 passed / 0 failed**（13 个测试二进制）；`cargo test -p auto-test-case` 全量 **91 passed / 0 failed**。862 门在 `e062edf4e` 上重跑：`targeted`（13+23+9+26+13）、`compat`（97 例/26 a2r，scalar 身份轴不变）、`entry`（12/12 + `unit→value` 变异恰好红 U02/U04）、`acceptance`（**clean 20/73、probe 0、wait:* 0、NA-4=30**）。
- **未被本轮修复改变（再核）**：`binding.tsv` expected/oracle 与 12 列 schema、既有 case/源、852 脚本、`docs/specs/**`；`void` 在 scalar/record/text 面仍 `source.unsupported`（既有测试 + 新增断言双重守护）。
- **保留边界（非 finding，交 merge/后续）**：B2 决策件附录无规则 ID 的 `na:m1`（C01/C02/I01–I11/G02–G08）仍待单独裁定；D-862-R1-01（852 验证脚本装载器预存缺陷）仍在 `KNOWN-DEBT-AND-RISKS.md`；a2r 新金样归 a2r 线。

2026-10-28 **work T-07 修复轮（F-862-R1-01..08 全部关闭）**：`stage: work | plan_id: PLAN-862 | plan_revision: 3 | outcome: 完成（T-05/T-06 重新勾闭，下一步 re-review） | worktree_path: D:/autostack/.wt/lang-862/auto-lang | branch: plan-862-dev | base_commit: 13057957c6363c87f63bb2e2edc2d36ad3ad773a | code_commit: e062edf4e（修复轮；其上 0e9c08189 / ea644f8e8 / 8cda9958a 与 327037b06） | dependency_revisions: auto-down c3464d0、auto-ui f489c7b（只读，未写） | task_ids: T-05, T-06 | findings: F-862-R1-01..08 CLOSED | next: review T-07（re-review）`。

| Finding | 修复（文件） | 关闭证据（命令 → 结果） |
|---|---|---|
| F-01 NA-4 未裁定 | 用户 2026-10-28 复审中批准；写入 SD-06 与[决策件 §12](../reports/ace-m1-scope-decision-2026-10-28.md)（适用/依据/重新进入条件/防扩散三条） | 文档一致性复核；`-Scope acceptance` 的 `na` 分布仍 `NA-4=30` |
| F-02 `source.unit-placement` 不可达 | `experimental/ac-core/src/source/parser.rs`：entry-unit 域 `param_type`/`decl_ann` 接受 `void` 词（scalar 等仍 `source.unsupported`）；字段条款（entry-unit 面无 `type` 声明）由 HIR/MIR 层守护，MIR 侧补负例 | `entry_unit_862` **26 passed**（param/local 两条新负例 + `verify.domain` 跨域一条）；`av_entry_862` **13 passed**（`mir.unit-placement` 新负例） |
| F-03 AC-03 第二方向 | 新增两条身份负例：scalar 文档 → `verify_entry_unit` = `verify.domain`；`AVAU` 字节 + legacy owner schema = `mir.schema-profile`（受控拒绝，非重解释） | 同上（26 / 13 全绿） |
| F-04 规范增量不完整 | 计划 §5 SD-01/04/05/06 增补：`source.entry-order`/`source.entry-result`、Unit 三层放置可达性、U01–U10 与 `av_channel` 轴、`mir.unit-placement`/`mir.entry-kind`、NA-4、六个新 owner token、`probe`/`wait:*` 清零与实测 clean | 计划 §5 表已更新（merge 沉淀） |
| F-05 注释与实现不符 | `contract.rs` 注释改为"unit 通道没有任何 exit oracle（含 0）" | `contract_decode` 的 `unit+exit` 负例（0 与 1）覆盖 |
| F-06 未 rustfmt | 对 9 个触及文件执行 `rustfmt` | `rustfmt --check`（9 文件）exit 0 |
| F-07 准入负例无持久测试 | `crates/auto-test-case/tests/contract_decode.rs` 新增 4 条（profile+channel 正例、`unit`+`exit` 0/1 拒绝、未知 profile、非 core recipe 带 profile） | `cargo test -p auto-test-case --test contract_decode` → **23 passed** |
| F-08 V-07 内存腿 | `av_entry_862` 新增 windows-only 进程峰值采样（`mempeek` 1ms、各取两次 warm 最大值、256MiB 粗门 + 实测打印） | `--test-threads=1 --nocapture`：entry-unit **331,776–557,056 B**（0.32–0.53 MiB）、scalar 控制组 **45,056 B**；远低于 128MiB worker 预算 |

- **修复轮整体门（均在 `e062edf4e` 上，原始证据本地 `tmp/862/verify-acceptance-r2/`）**：`-Scope targeted` → 合同解码 13 / 准入负例 23 / 父侧判定 9 / 入口矩阵 26 / probe 首例 13 全绿；`-Scope compat` → 旧 97 例（26 a2r）绿且 scalar 身份轴未变；`-Scope entry` → 12/12 绿、`unit→value` 变异恰好红 U02/U04；`-Scope acceptance` → **clean = 20/73、probe 0、wait:* 0、NA-4=30**。
- **未变（本轮再核）**：`binding.tsv` 的 expected/oracle 与 12 列 schema、既有 case/源、852 脚本、`docs/specs/**` 均零改动；未跑整档 `cargo t`/`tv`（merge 门 T-08）。
- **复审注意**：F-02 的修法是"让冻结码可达"（推荐路径），未改任何既有 profile 的接受面（scalar/record/text 对 `void` 仍 `source.unsupported`，由 `void_is_not_part_of_the_other_profiles` 与新增断言双重守护）；F-03 的 wire 负例实际命中 `mir.schema-profile`（entry-unit 自有 owner schema 在 profile 门先拒），已在测试内写明原因。

2026-10-28 **review T-07（final，needs_fix）**：`stage: review | review_scope: final | reviewer_identity/limitations: 本会话即 T-01..T-06 实施会话，无独立外部/异模型复审上下文；结论由冻结提交 diff、原始回执复算与独立探针重建，不采信 work 总结 | plan_id: PLAN-862 | plan_revision: 3 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-862/auto-lang | branch: plan-862-dev | reviewed_commit: 0e9c08189eeff8327f9b573b0620f84d4a12c1bd（树 clean） | base_commit: 13057957c6363c87f63bb2e2edc2d36ad3ad773a（master 复审时为 a27435014，merge 需调和） | dependency_revisions: auto-down c3464d0（只读）、auto-ui f489c7b（只读，本计划新增组内检出） | spec_inputs: 计划 sha256 332e7afa…、entry-contract b364c10b…、对等口径决策件 8ef5876d…、ace-m1-conformance 976dfd60…（本计划未改 docs/specs） | acceptance_results: AC-01 pass、AC-02 pass（resource/cancel/infra 腿见边界 B1）、AC-03 partial（F-03）、AC-04 partial（F-08）、AC-05 pass、AC-06 partial（F-01，NA-4 已由用户当轮裁定） | pending_closeout_gates: T-08 merge 裸 cargo t + tv（diff 触及 MIR/ABC/AV 与前端）；T-09 Spec/ledger/归档/清树 | findings: F-862-R1-01..08（OPEN）| evidence: 见下 | next: work（T-05/T-06 重开）`。

- **独立复核（本轮实际执行，均在 reviewed_commit 上）**：① 逐行比对 `binding.tsv`：103 → 113 行、无删除、**expected 列 103 行零改动**、无 `src/*.at` 与既有 case 文件改动（仅 binding + 新目录）；② 独立复算五态：corpus 73 行、green 单元格 137、**clean = 20/73**（与 acceptance 打印一致，与"50/103"投影的差异=未执行格保留旧 token，见 work 记录）；③ 抽样对齐原始回执：T01/av-owned PASS、R04/av-owned PASS 且 av-data FAIL、V01/av-core `source.lex`、B17/ac `mn.unsupported.frontend`、B18/avm infra ERROR、R07/a2r emit 拒绝、E01/E02 无源码行（未跑）→ 与表内 token 一致；④ 在冻结提交上复跑判别力反控：`unit`→`value` 变异恰好红 U02/U04，entry suite 12/12；⑤ V-07 周转由 3 份原始报告重算 n=36/P90=362ms/median 225/max 619，三次均 12 PASS 0 SKIP；⑥ 合同负例探针（本地产物，未入库）：`av_channel:"unit"`+`exit:0` 在装载期被拒（文本与实现一致），但见 F-07；⑦ 触及 crate `cargo check` 无新增告警；`rustfmt --check` 见 F-06；无新增 `dbg!/println!/TODO`（worker 的 `eprintln!` 沿用既有协议错风格）；⑧ `docs/specs/**` 零改动；852 脚本零改动。
- **F-862-R1-01（high，AC-06 / 反滥用规则 2、3）OPEN**：T-06 为 U01–U10 的 ac/avm/a2r 共 30 格新增 `na:m1(NA-4)`，而计划 §5.5 与决策件 §5 规则 2 都禁止计划自行新增 `na` 规则、规则 3 规定 na 增长无决策记录即 merge 门失败。**用户 2026-10-28 复审中裁定：批准 NA-4**（入口/Unit 为 AV-only profile，native/legacy 不要求判它；重新进入条件=有计划冻结 native/legacy 对该 profile 的支持时逐行改回 `gap:<plan>`）。修复：把该裁定写入 SD-06 正文与决策件附记（merge 沉淀时落 canonical），并在 T-09 的 852 acceptance 报告单列 `na` 分布；U 行 token 保持不变。
- **F-862-R1-02（medium，AC-01/AC-02、SD-01/SD-05）OPEN**：冻结码 `source.unit-placement` **不可达**——参数/局部/字段位置的 `unit` 在 parser 即被 `source.unsupported` 拒绝（`is_type_word`/`type_word` 未含 unit，`member_type` 同），resolve 的两处 `source.unit-placement` 分支无输入可达；`verify.unit-placement`/`mir.unit-placement` 只在手工构造文档/MIR 时可达且**无测试**。T-03 §9 把"Unit 放置"列为已通过证据，与事实不符。复现：`fn helper(u unit) int { return 0 }` + `fn main() int { return 0 }` 与 `let a unit = 1`，经 entry-unit profile 实跑均返回 `source.unsupported`（本地产物 `tmp/862/probe/`）。修复二选一并记录：(a) 推荐——在 entry-unit 域让 `param_type/decl_ann/member_type/type_word` 接受 `unit` 类型词，使冻结码可达，并在 ac-core 补 3 条负例（param/local/field）；(b) 若维持 parser 拒绝，须把 SD-01 写成"结果位以外 `unit` = `source.unsupported`"并 retire `source.unit-placement`（合同修订，超出本计划授权，需回用户）。
- **F-862-R1-03（medium，AC-03）OPEN**：双向身份拒绝只有单向证据——旧 profile 拒新语法有测试（`void_is_not_part_of_the_other_profiles`、`expression_statements_stay_rejected_on_the_scalar_surface`、U02 `/av-scalar-rejects`）✓；**新域侧**（旧 scalar HIR 文档喂 `verify_entry_unit`、AVAB/AVAD 字节喂 entry-unit decode）无直接测试，只有 entry-unit magic/kind 篡改测试。修复：加两条身份负例（domain 失配 + wire 家族失配）或引证并补齐既有位置。
- **F-862-R1-04（medium，SD-04/SD-06）OPEN**：规范增量不完整——SD-06 未写 NA-4（用户已裁定）、未写本轮新增 owner token（`a2r-golden`/`scenario-tooling`/`av-text-move`/`av-text-nul`/`av-mut-param`/`P864`）与 `probe`/`wait:*` 清零口径；SD-01 未收录实现中新增但不在 T-02 §8 冻结清单的两个码（`source.entry-order`、`source.entry-result`）。按现 delta 沉淀会让 canonical spec 与 `binding.tsv` 不一致。
- **F-862-R1-05（low）OPEN**：`crates/auto-test-case/src/contract.rs` `av_channel` 上方注释称"非零 `exit` 与 `unit` 同格矛盾"，实现拒绝**任何**与 `unit` 并存的 `exit`（含 0，已复现）。修复：改注释为"unit 没有 exit oracle"（保持严格性）。
- **F-862-R1-06（low）OPEN**：`crates/auto-lang-core/src/test_cases/ac_av.rs` 两处 `probe_toolchain_axes(...)` 调用超宽，`rustfmt --check` 报 diff（其余触及文件干净）。修复：`cargo fmt` 后重跑 ac_av 定向测试。
- **F-862-R1-07（low）OPEN**：新增准入负例无持久测试——`av_channel:"unit"`+`exit` 的装载期拒绝、未知 `profile`、非 core recipe 带 `profile`，本轮仅以临时探针验证。修复：落成合同层单测（3 条）。
- **F-862-R1-08（medium，AC-04/V-07）OPEN**：V-07 内存腿只有模块预算计数（`peak_staging=6195B`/`peak_hot=2906B`/`slot=120B`），**未记录进程峰值样本与旧 profile 反控**；T-02 §7 口径要求"记录命令、样本与反控"。修复：用既有 `mempeek` 采样一次 entry-unit 请求的进程峰值并同命令采样 scalar 控制组，或按 (a) 记录明确的边界决定与理由。
- **边界（非 finding，已核）**：
  - B1：V-02 的 resource/cancel/infra 腿在本 profile **无新增触发面**（entry-unit 面为 scalar 子集，Unit 无分配载荷，probe 配方不暴露 limit/cancel 旋钮）；这些是 AV 执行器既有通道，由既有 core 测试守护，本计划只新增"int/Unit/trap 三态互不替代"的证据（含 Unit≠`I32(0)` 与 trap 测试）。
  - B2：决策件附录对 C01/C02/I01–I11/G02–G08 写无规则 ID 的 `na:m1`，与其 §5 规则 1 冲突；全表词表回填须先裁定这些行的规则归属（候选 NA-5/NA-6）——记为待决策，本轮不做（避免制造违规 token）。故 M1 clean 以实测 20/73 记，不与 50/103 投影比较。
  - B3：`ok:av-closures-823`/`ok:av-data-typed` 属 823/824 的历史 ok 前缀 token，acceptance 以 `^ok` 计入绿；本计划未改其语义。
- **D-862-R1-01（debt，范围外，已登记 KNOWN-DEBT）**：PLAN-852 的 `scripts/verify-ace-m1-852.ps1` Load-Binding 用 `-split "`t", -1`，在 pwsh 7.6.9 下第三操作数按 options 解析，返回 1 段 → 其 `contract`/`acceptance` scope 无法运行（本计划未改该脚本，git diff 为空）。本计划 `verify-av-862.ps1` 已用 `.Split([char]9)` 覆盖同一完整性面（12 列/唯一 id/owner 注记/`na` 规则 ID）。852 已归档，修复需独立 L0 worktree。
- **未复核（留给 merge/后续）**：整档 `cargo t`/`tv`（T-08）、Spec/ledger 沉淀与归档（T-09）、a2r 新金样、以及 decision-appendix 全表回填（B2）。

2026-10-28 **work T-05 + T-06（语料/脚本/绑定 + M1 分母重绑定）**：`stage: work | plan_id: PLAN-862 | plan_revision: 3 | outcome: 完成（T-01..T-06 全部完成；T-07 review / T-08/T-09 merge 未开始） | worktree: D:/autostack/.wt/lang-862/auto-lang | branch: plan-862-dev | base: 13057957c | code_commit: 8cda9958a（T-05）+ ea644f8e8（T-06）+ 0e9c08189（验证脚本装载器修正），其上 327037b06 为 T-03b/T-04 | deps: auto-down detached c3464d0 只读；**新增组内只读兄弟检出** `D:/autostack/.wt/lang-862/auto-ui`（detached f489c7b，根 crate 路径依赖 `../../../auto-ui` 所需，非本计划写面，merge 清理时一并 guard/移除） | task_ids: T-05, T-06 | next: review T-07`。

- **T-05 交付**：合同层 append-only 两项——`ProbeAvSpec.profile`（值仅 `entry-unit`，缺省=scalar 逐字节不变）与 `ExpectSpec.av_channel`（`value`/`unit`；`unit` 与 `exit` 同格声明被拒，禁止把 Unit 折成 0），`ResolvedOracle` 同步参与 oracle 身份；worker 接受 `profile` 并上报 `channel` 与 `av.entry-unit/1` 身份；父侧判定绑定 expected core 版本（`av.entry-unit/1` vs `av.core-i32/1`）、receipt/toolchain 增 `av-profile=7`+`av-source-profile=entry-unit`、scalar 行禁止携带该轴。语料：`crates/auto-lang/test/cases/ace-m1/entry-862/`（U01–U10 + suite，独立手算 oracle：42/unit/42/unit/trap/5 类入口诊断）；脚本 `scripts/verify-av-862.ps1`（`targeted`/`compat`/`entry`/`acceptance`，targeted 对零用例过滤设下限）。**顺带修正**：真实语料暴露重复 `main` 先撞通用 `source.duplicate`，按 T-02 §2.2 在 entry-unit 域让位给 `source.entry-duplicate`（其他名字/profile 规则不变）。
- **T-05 验证（本提交上，原始证据本地）**：`-Scope entry` → **12/12 绿**（逐行断言 channel/exit/toolchain 三元组；判别力反控：把 U02/U04 的 `unit` 期望改成 `value` 后**恰好这两行红**）；`-Scope compat` → 旧 852 base suite **97 绿（26 a2r）**、每个 probe 检查仍报 `av.core-i32/1`（零漂移）；`-Scope targeted` → 合同解码 13、父侧判定 9、ac-core 入口矩阵 24、probe 首例 10，全绿（每步都有"过滤零用例即失败"的下限）。
- **T-06 交付**：24 `probe` + 27 `wait:*` 逐格真实执行（28 个有源码的行 × 四路径 + record/text-record 两个 AV recipe；期望全部取冻结 oracle，诊断/陷阱格用哨兵码读回实际码；**未改 expected/oracle/case 源**），结果写回 `binding.tsv` 单元格与证据注记（35 行改动 + 新增 U01–U10 行）。**probe 24→0、wait:* 27→0**；corpus 实测 **clean = 20/73**。关键翻转：T01/T02/T04/T06/T07/T08 与 R01–R06/R09/R10 的 av 列由未跑变为 `ok`（record/text 源码经 `ac-av/data|owned-run-v1` 真跑：38/27/128512/借用逃逸拒绝/非法标量拒绝/越界 trap；19/9/23/51/42/21/名义错型拒绝/move 后读拒绝）；V01–V07 归 `gap:P864`；R07 `gap:av-mut-param`、R08 `gap:av-lower-method`（真实执行复证）；9 个 a2r 格实测"发射成功但无冻结金样"→ `gap:a2r-golden`（R07 为 `gap:a2r-untyped-emit`）；B17–B19 的 ac 三格与 B18 avm 为 `diff:pending`（新引擎/旧引擎与 oracle 的真差异，未套旧侧裁定），B17/B19 avm 按 R-C 落 `diff:accepted(R-C)`；无源码的 scenario 行（E01/E02/R01/R04/R06/I04/I09）记 `gap:scenario-tooling`（trace/events/layout 观测通道未交付）。
- **T-06 V-07 计量**：文件化 TestCase 12 检查 ×3 轮全绿；fresh `cache_hit_abc=0/cache_miss_abc=1/mir_units_persisted=1/av_execute=1`；hit（同 store）`cache_hit_abc≥1/mir_build=0` 且 AV 仍执行；restart 真新进程 decode+verify+重跑 Unit 通道不变；body-edit 新增测试 `av_entry_862_body_edit_rebuilds_only_the_edited_unit`（值 7→8、未改 callee 命中、`cache_miss_abc=1`）；周转 36 个端到端样本 **P90=362ms**（median 225 / max 619，硬门 ≤2000ms）；内存记模块预算 `peak_staging=6195B/peak_hot=2906B/slot=120B`，**进程峰值采样未重跑**（沿用 825/828 预算门，本 profile 不新增分配通道）。证据摘要（本文件随提交入库）：[t05-t06-evidence.md](../attachments/862/work/t05-t06-evidence.md)。
- **两项须 review 裁定的口径**：① U 行的三条非 AV 路径记 `na:m1(NA-4)`——NA-4 是本计划为"AV-only 入口 profile"新增的规则（决策件只冻结了 NA-1..NA-3），按反滥用规则 2 需 review 明确接受，否则 U 行只能记 gap（会虚增 M1 负债）；② 决策件附录对 C01/C02/I01–I11/G02–G08 写无规则 ID 的 `na:m1`，与其自身 §5 规则 1 冲突，故本轮**未做全表词表回填**（回填须先裁定这些行的规则归属）。因此实测 clean=20/73 低于决策件的 50/103 投影：差异是记账口径（未执行格保留 `emit-ok-run-unverified`/`diff-legacy-*`/`snapshot-only-cert-gap`/`n/a` 等旧 token），不是执行退化。
- **未执行**：整档 `cargo t`/`tv`/`tu`（merge 门 T-08）、`docs_gen`、Spec 沉淀与归档（T-09）、a2r 新金样冻结（归 a2r 线）。
- **交接注意**：主检出仍带其他会话的未提交 docs WIP（代码零 WIP，仅 §§簿记编辑）；本计划的簿记提交需与其他 WIP 分开暂存。实施树 clean（两提交后无未跟踪业务文件；`tmp/862/**` 为本地证据，未入库）。

2026-10-28 **work T-03b + T-04（入口/Unit 全链贯通）**：`stage: work | plan_id: PLAN-862 | plan_revision: 3 | outcome: 部分完成（T-01..T-04 完成，T-05/T-06 未开始） | worktree: D:/autostack/.wt/lang-862/auto-lang | branch: plan-862-dev | base: 13057957c | code_commit: 327037b06（T-03b/T-04；其上 aa2cf334e 为 T-01/T-02/T-03a） | deps: 无变更 | next: work T-05 → T-06`。

- **T-03 完成（T-03a+T-03b）**：源码域与 HIR 半均落地。T-03a 见下条记录；T-03b 新增 `schema/entry-unit.atom`（`auto.hir.entry-unit.draft`/1/`entry-unit-draft`）并注册进 `bind_document`，`unit` 仅在该域作内建 TypeUse 词、`ExprKind::Unit` 为无载荷的 unit 值（schema `expr.unit` 分支 + binder + 规范写器）；新增 `verify_entry_unit`（scalar 结构规则 + `verify.unit-placement` 结果位唯一 + `verify.unit-expr` 单元值配对）；adapter 真正发射 `result: unit`（bareword，无 `t_unit` 表项）、Unit 函数体的终结 `return` 带 unit 值，脚本 i32 尾值由 resolve 改写为该 entry 的 return（下游只见一种形状）；T-03a 的临时发射闸门已删除。native/text 资源/CLI/artifact codec/semantics adapter/CST adapter 全部给出显式臂：Unit **没有 native lowering**（AV-only），其余消费者受控拒绝。
- **T-04 完成**：lowering 接受 `HirDomain::EntryUnit` 并产出 `av.entry-unit/1` 模块（owner schema `auto.mir.av-entry-unit/1`），unit 结果类型落地，`ExprKind::Unit` 降为零宽 temp（无指令），entry kind 由入口签名推导；MIR 增 `MirType::Unit`（wire tag 18，仅该家族）/`EntryKind`/`MirModule::entry_kind`，`verify_mir` 增 profile 准入、`mir.unit-placement`（Unit slot 仅作 Unit 入口的 return 操作数）、`mir.entry-kind`（kind 与签名**双向**一致，kind-less 拒绝）、零宽 slot 按宽度免初始化（不是放宽初始化分析）；ABC 增**新 magic 家族 `AVAU`**（wire v1 / profile id 7 / `auto.abc.av-entry-unit/1`）与头部 entry-kind 字节，TY_UNIT 与 kind 字节在 encode/decode 双向家族门控，AVAB 与 AVAD v1–v5 互不可读；AV 增 `Value::Unit`（真值，绝不等同 `I32(0)`），`typed_zero` 为 Unit slot 铸造该值，入口 return 交回 `Success{value: Value::Unit}`，portable(Dyn) 编码受控拒绝，closure/snapshot/cache 合并路径携带 profile+kind，probe 增 `RequestProfile`/`set_profile` 与 `EntryUnitAbc` wire 代（独立 cache 命名空间、owner schema 映射到 MIR 身份，request 与 worker restore 两侧一致）、`value_unit()`/`channel()` 使通道可观察。
- **验证（本轮实际执行，均在本提交上）**：`experimental/ac-core` `cargo test --test entry_unit_862` → **24 passed**（三形态入口 + 入口矩阵 9 类诊断 + Unit 放置 + 静态全链）；`experimental/ac-av-probe` `cargo test --test av_entry_862` → **9 passed**（int/unit/script 通道、Unit≠`I32(0)`、trap 独立通道、**篡改 kind 字节被 `mir.entry-kind` 拒绝**、scalar profile 控制组、store fresh→hit、**真新进程 `av_probe_worker restore-run` 复跑 UNIT 通道**）。旧域回归：ac-core `scalar_source/source_frontend/source_hir/scalar_hir/text_source/record_source/text_record_source/text_binding` → **113 passed**；probe `av_source_827_{semantics,first_case,incremental}/mir_tier/abc_semantics_tier` → **49 passed**；`auto-av-core` → **58 passed**。各 experimental 消费方 workspace（ac-case-worker/ac-cte-probe/ac-semantics/ac-artifact-probe/auto-abi-probe/ac-syntax/ac-mir-native）`cargo check --all-targets` 通过，唯 `ac-mir-native/tests/codegen_tier.rs` 因 PLAN-818 之后 `MirFunction/MirImport` 增 `env` 字段**本就编译红**（与本次无关，N 窗口归 870/871）。**未执行**：整档 `cargo t`/`tv`/`tu`（merge 门）、852 语料、`verify-av-862.ps1`（T-05）、M1 分母重绑定（T-06）。
- **AC/V 证据映射**：AC-01 ← V-01（三形态 + 具名拒绝，测试与 `entry-contract.md` §2 逐条对应）；AC-02 ← V-02（Unit/trap 通道可区分，Unit≠0 有正反两面）；AC-03 ← V-03（旧五 profile 前端与 scalar 执行路径 113+49 例零漂移；新域/旧域双向身份拒绝由 descriptor/verify/wire 家族门控测试覆盖）；AC-04 ← V-04（真实源码全链 + 篡改 kind 拒绝 + 新进程恢复；V-07 的首例**计量数字**仍待 T-06 补齐）。
- **未勾选项**：T-05（语料/`verify-av-862.ps1`/binding 追加）、T-06（24 `probe` + 27 `wait:*` 逐格重绑定 + V-07 首例 cache/文件化/周转/内存数字）、T-07 review、T-08/T-09 merge 保持 `[ ]`；AC-05/06 因此仍为 open。
- **交接注意**：主检出仍带其他会话的未提交 docs WIP；本计划的簿记编辑与其同处 master 工作树，仍未提交以免误带。运行 827 测试会改写 `docs/reports/plan827/*.json`，本轮已 `git checkout --` 复原并删除 pid 后缀临时产物，工作树现为 clean。

2026-10-28 **work T-01/T-02 + T-03a（前端半）**：`stage: work | plan_id: PLAN-862 | plan_revision: 3 | outcome: 部分完成（T-01/T-02 完成，T-03 仅前端半；T-04..T-06 未开始） | worktree: D:/autostack/.wt/lang-862/auto-lang | branch: plan-862-dev | base: 13057957c | code_commit: aa2cf334e | deps: 无变更（auto-down detached c3464d0 未被写入） | next: work T-03b → T-04`。

- **T-01 完成**：`attachments/862/work/baseline.md`（现状表格：源码前端/HIR/lowering/MIR-ABC-AV/probe/852 语料，逐条带文件:行号，并单列未验证假设）。基线核查另有一次 `cargo check --workspace --all-targets`（`experimental/ac-av-probe`，exit 0，33.21s）。
- **T-02 完成**：`attachments/862/work/entry-contract.md` 冻结入口三形态、唯一性/形态互斥诊断表、Unit 跨层表示、结果通道、entry kind、退出码映射（消费者合同）、fresh/hit/restart/body-edit 计量与反控、profile 三元组（`auto.source.entry-unit.draft` / `auto.hir.entry-unit.draft`+`entry-unit-draft` / `auto.mir.av-entry-unit/1` / `auto.abc.av-entry-unit/1`+`AVAU` v1 / profile id 7 / `av.entry-unit/1` / producer `plan862/av-entry-unit`）。§9 记录了对计划 §5.3 措辞的两处收窄（wire kind 取 2 值 int/unit；退出码留在消费者），两者都不放宽 AC-01/AC-02。
- **T-03a 已交付（T-03 未完成）**：源码域前端半落地——`SourceProfile::EntryUnit` + `auto.source.entry-unit.draft`、`TypeWord::Void`/`VT::Unit`、`Stmt::ExprStmt`（脚本尾值）、顶层语句序列→合成 `main`、入口矩阵 9 类具名诊断（`source.entry-{missing,duplicate,ambiguous,params,top-return,tail,tail-type,result,order}`、`source.unit-return-value`）。**未做**：Atom/HIR 发射（`entry-unit.atom` schema + descriptor/verify + adapter 发射，T-03b）——该路径以 `source.unsupported` 受控拒绝、显式命名 T-03b，不做静默 fallback。
- **验证（本轮实际执行）**：`experimental/ac-core` `cargo test --test entry_unit_862` → **21 passed / 0 failed**；旧面回归 `cargo test --test {scalar_source,source_frontend,source_hir,text_source,record_source,text_record_source,text_binding}` → **97 passed / 0 failed**（旧五 profile 前端零漂移，AC-03 第一方向）；`experimental/ac-av-probe` `cargo check --workspace --all-targets` → exit 0；消费方 `experimental/{ac-artifact-probe,ac-syntax,auto-mir-lower}` 随该检查编译通过。**未执行**：整档 `cargo t`/`tv`/`tu`、852 语料、`verify-ace-m1-852.ps1`（均属 T-05/T-06 或 merge 门）。
- **未勾选项**：T-03..T-09 保持 `[ ]`；AC-01 只有前端半证据，不构成 AC-01 通过；AC-08/AC-09 属 merge，未触碰。
- **交接注意**：主检出当前带其他会话的未提交 docs WIP（54 文件，含 roadmap/design/reports），本计划的簿记编辑与该 WIP 同处 master 工作树，未提交以免误带；实施代码在 `plan-862-dev` 上单独提交。

`stage: new | plan_id: PLAN-862 | plan_revision: 3 | outcome: pass（修订草案静态预检；实施仍等真实D门/写面释放） | next: T-01/T-02只读准备，D868后T-03`

2026-10-28 r2→r3（用户授权修订波次/写面，864纳入拥有型List、871补旧能力与删除门）：仅修订执行合同，不实施。原status/current_step/total_steps、任务勾选与历史记录保留；D868合同和进度不变。V-07补交接/消费者兼容与首例cache、文件化测试、周转及内存核验；详细依据与静态预检见[波次合同](../reports/ace-wave-handoff-2026-10-28.md)和[修订收据](../attachments/ace-wave-20261028/revision-summary.md)。本记录只说明文档修订，未执行AC、性能测试或独立实现review，merge-owned门禁仍待实施后完成。

2026-10-28 new r1：`stage: new | plan_id: PLAN-862 | plan_revision: 1 | outcome: pass（可在既有授权内进入 work；T-02 若需改变已批准语义则回用户决定） | next: work T-01→T-02`。全部 AC/V/T 未执行；profile 号、schema 名、诊断码均为建议值，T-02 冻结后写回。本轮未建实施树、未跑任何构建/语料，未改在途Plan与既有Spec。批次排序依据见[后继批次规划](../reports/ace-next-batch-2026-10-28.md)。

2026-10-28 **r1→r2（并入 M1 分母重绑定）**：`stage: new | plan_id: PLAN-862 | plan_revision: 2 | outcome: pass | next: work T-01`。变更原因：用户 2026-10-28 裁定"M1 分母重绑定不另立计划，并入下一个要做的计划"；对账口径由[对等口径决策件](../reports/ace-m1-scope-decision-2026-10-28.md)定稿（4/4 采纳推荐项）。合同变更：新增 §5.5 边界、SD-06、V-06、AC-06、T-06；因本Plan仍为 drafting、`current_step: 0`、无任何已执行证据，原 review/merge/归档任务与 AC 顺延为 T-07/T-08/T-09 与 AC-07/08/09（无历史证据被失效）。范围未扩大：仍不含 863–866 的能力，重绑定只翻转状态列、不改 expected/oracle/schema。

## 10. 待澄清事项

- SEM-05 三形态（int main / Unit main / 顶层脚本）是否**同时**进入首批，还是先交付前两种、脚本形态另立切片。若用户要求脚本形态同批，须确认“脚本执行顺序与副作用可见点”的oracle由谁裁定。
- 显示/格式化（旧源码的打印格式）是否需要在 863 之前先定；本Plan默认把全部显示面推给 863。
- 是否要求本 profile 同时进入 AC native 路径（`gap:ac-composite-native`/`mn` 一侧）。默认**不要求**：native 复合扩面另立计划，本Plan只交付 AV 通道。

2026-10-28 D868 前置解除：PLAN-868 已交付落 master（delivery 7754c4e4c）——HIR→MIR lowering 唯一实现为 `experimental/auto-mir-lower/src/**`（纯搬移等价已证，旧 `probe/src/hir_adapter.rs` 已删除），本计划 D868 硬前置视为满足；lowering 相关 T 任务的写面以该 crate 为准（边界合同见 [docs/specs/auto-mir/hir-lowering-boundary.md](../../specs/auto-mir/hir-lowering-boundary.md)）。
