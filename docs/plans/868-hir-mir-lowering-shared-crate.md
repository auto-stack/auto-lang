---
plan_id: PLAN-868
status: execution_done
feature_name: HIR→MIR lowering 出库为共享 crate
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-mir/hir-lowering-boundary.md
touched_goals: [GOAL-002]

affects: [auto-mir, auto-av, auto-ac, auto-test-case]
depends_on: []
current_step: 5
total_steps: 8
---

# [PLAN-868] HIR→MIR lowering 出库为共享 crate

## 0. 变更摘要

**代码事实**：HIR→MIR 的 lowering 现在住在测试探针宿主里——`experimental/ac-av-probe/probe/src/hir_adapter.rs`（约 106 KB），而 `auto-av-probe` 被另外两处跨 crate 依赖：

- `experimental/ac-mir-native/src/driver.rs` 调 `auto_av_probe::hir_adapter::lower_checked(...)`（mn 的原生对拍路径）；
- `experimental/ac-cte-probe/probe/Cargo.toml` 依赖 `auto-av-probe`（CTE 消费者）。

也就是说：**一个名字叫 "probe" 的测试宿主，目前是 ACE 生产语义管线（HIR→MIR）的唯一提供者。** 这既是"旧 HIR 直达 → HIR→MIR 单源"迁移（[Design43 §2](../design/43-ac-av-typed-mir-and-abc.md)）的第一个硬前置，也是 862/863/864/865 四份计划的 lowering 改动必须面对的落点问题——先改后搬会产生跨计划 merge 冲突与证据重绑定。

本Plan只做三件事：**搬移**（lowering 出库）、**依赖倒置**（三个消费者共用一份实现）、**版本身份**（lowering 版本进入报告轴）。**不改任何 lowering 语义**。

## 1. 目标

- 建立独立共享 crate（建议 `experimental/auto-mir-lower`），持有 `HIR → Typed MIR` lowering 的唯一实现，供 `probe`、`ac-mn`、`ac-cte-probe` 三处消费；probe 内实现**删除**，不留双份/shim。
- 保持依赖方向红线：`auto-av-core`（执行核心）**不得**依赖 `ac-core`（前端）；`auto-lang`（产品）继续零 Cranelift/零实验栈依赖。
- 给 lowering 一个稳定的**契约版本**（`MIR_LOWERING_VERSION`）并入报告身份轴，使"lowering 变了"与"crate 版本变了"可区分。
- 搬移前后**行为等价**：同一批语料/suite 在搬前搬后逐 check 的 `status` 与 `actual` 完全一致（含 av / mn / cte 三族与 852 的 ace-m1）。
- 为后续迁移（mn 扩 data/owned、消费面切换、旧 lowering 退役）提供唯一写面；862/863/864/865 在此之后改 lowering 即落在终局位置。

**非目标**：不新增/修改任何 MIR op 或 ABC 语义；不扩 mn 的 native 面（关掉 `outside the mn scalar surface` 属后续迁移计划）；不改 recipe/`autolang.case`/suite schema、不改 `ace.m1.cert/1` 键集；不把 collections/closures/dynamic（typed-only 面）拉进来；不动 ac-core 前端/verify/descriptor；不做性能优化；不顺带清理 `probe/src/native_reference.rs` 等其他宿主职责。

## 2. 架构方案

```text
ac-core（AST/HIR/verify/descriptor）
        │  HIR 类型
        ▼
   auto-mir-lower  ──▶ auto-av-core（Typed MIR 模型/verifier/ABC）
        ▲    ▲    ▲
        │    │    └── ac-cte-probe（CTE 消费者）
        │    └─────── ac-mn（MIR→CLIF 原生对拍）
        └──────────── ac-av-probe/probe（worker、会话、cache 宿主）
```

| 决定 | 内容 | 理由 |
|---|---|---|
| 新 crate 而非并入 `auto-av-core` | lowering 依赖 ac-core 的 HIR 类型，而执行核心必须保持"无前端依赖" | 若并入，`auto-av-core` 会被 HIR 类型污染，任何宿主/运行时消费执行核心都得带上编译器前端 |
| probe 保留宿主职责 | worker、`case-serve`、会话、cache、观察 | 这些是测试基础设施，不该进共享 lowering |
| 一次性删除旧实现 | 不做 re-export shim、不留 `#[deprecated]` 副本 | 双份实现会立刻产生漂移，且旧路径无法判定是否还在被用 |
| 版本轴进报告、不进 cache 键 | 搬移不改输出，故 store/cache 键不变；只加报告轴 | 无谓失效会作废已交付 store 证据；语义真变时再按合同升版本 |

## 3. 技术栈

- 新 crate 建议路径 `experimental/auto-mir-lower/`（独立 `Cargo.toml`；`experimental/` 不是 workspace，各树独立解析）。
- 依赖：`auto-ac-prototype`（ac-core：HIR/诊断）、`auto-av-core`（MIR 模型）；**不依赖** Cranelift / serde（若消费者需要序列化由其自行加）。
- 消费方改动：`experimental/ac-av-probe/probe`、`experimental/ac-mir-native`、`experimental/ac-cte-probe/probe` 的 `Cargo.toml` + 调用点。
- 身份轴：报告 `toolchain` 侧新增 `mir-lowering` 条目（T-02 冻结键名与取值）。
- 门禁：三消费者 `cargo check`/定向 `cargo test`；正式档与语料对照在 merge 执行。拟建 `scripts/verify-mir-lowering-868.ps1`（Scope=equivalence/deps/identity/acceptance），**当前不存在、未运行**。

唯一实施树：`D:/autostack/.wt/lang-868/auto-lang`，分支 `plan-868-dev`。

## 4. 需求分析与背景调查

用户 2026-10-28 确认方向：把旧的 HIR 直达迁移到 HIR→MIR，使 MIR→Native 成为共享路线；并指定**先单独建立本计划**做 lowering 出库。本轮只规划，不建树、不跑构建、不改实现。

已核实的代码事实（逐条可复核）：

| 事实 | 位置 |
|---|---|
| lowering 实现位于测试宿主 | `experimental/ac-av-probe/probe/src/hir_adapter.rs`（106 KB；`lower_checked`/`lower_checked_data`/`lower_checked_owned` 等） |
| mn 通过 probe 消费 lowering | `experimental/ac-mir-native/src/driver.rs`：`auto_av_probe::hir_adapter::lower_checked(...)`；`Cargo.toml` 依赖 `auto-av-probe` |
| CTE 消费者同样依赖 probe | `experimental/ac-cte-probe/probe/Cargo.toml`：`auto-av-probe = { path = "../../ac-av-probe/probe" }` |
| 执行核心当前无前端依赖 | `experimental/ac-av-probe/core/Cargo.toml`：description 明确 "Pure std — no auto-lang/UI/man/Cranelift" |
| 实验栈在根 workspace 之外 | 根 `Cargo.toml` 的 `exclude` 含 `"experimental"`；产品构建图零 Cranelift |
| 设计方向已冻结 | [Design43 §2](../design/43-ac-av-typed-mir-and-abc.md)："未来共享 MIR 接线应另立迁移 Plan，以 native/AV 对拍验证后逐步替换" |

现状风险（本Plan要消除的）：

1. 生产语义由测试宿主提供，边界不清——任何人改 worker/会话都可能顺手动到 lowering。
2. 862（入口/Unit）、863（宿主导入）、864（集合/变体）、865（闭包）都需要改 lowering；若不先出库，四处改动都落在 probe，之后搬移会产生大范围冲突与证据重绑定。
3. 无版本身份：无法回答"这次 native/AV 结果不同，是不是 lowering 变了"。

## 5. 详细设计

### 5.1 前置、写面与顺序

- **硬前置：无**（不依赖 862–866，也不依赖在途 844/861/867）。反过来，**862/863/864/865 应把本Plan的 `D868` 作为 lowering 写面的硬前置**；866（在线重载）不受影响。
- 写面：新 crate 全部；`ac-av-probe/probe`（删实现、改调用）；`ac-mir-native`（改依赖/调用）；`ac-cte-probe/probe`（改依赖/调用）；三处 `Cargo.toml`；报告身份轴所在文件（`auto-lang-core/src/test_cases/{stale,ac,ac_av,ac_mn,ac_cte}.rs` 中实际持有 toolchain 身份者）。
- 不写：`auto-av-core`（保持零前端依赖）、`ac-core`、`auto-lang` 产品代码、recipe/schema、语料内容。
- 单写窗口：probe 与 mn 同改属共享核心窗口；与 864/865 的 MIR 写面按"先满足 D868、再取窗"交接，不持窗等上游。

### 5.2 搬移策略（纯搬移，禁止混合改造）

- 先搬再收敛：函数签名、模块路径、语义一律保持；消费者以最小改动指向新 crate。**不在本Plan做**命名统一（如 `lower_checked_data`/`lower_checked_owned` 改名）、不重构内部结构、不调优。
- 删除而非转发：probe 内实现删除；若编译期发现仍有调用点，逐个改指新 crate，不保留 shim。
- 边界核实（T-01）：`hir_adapter` 与 `probe/src/{session,fn_hir_codec,data_driver,owned_driver}` 的耦合点——若 lowering 依赖 probe 的会话/缓存类型，需要把这些依赖**倒置**（传入参数/回调）或把纯 lowering 部分切出来；这一步的实际切面由 T-01 定，T-02 冻结。

### 5.3 身份轴与证据影响

- 新增 `MIR_LOWERING_VERSION`（语义=lowering 契约版本；初值 1），由共享 crate 导出，消费者原样上报。
- 报告轴：在报告 `toolchain` 身份中追加条目（键名 T-02 冻结，建议 `mir-lowering`）。
- **收据不动**：`ace.m1.cert/1` 为类型化精确键集 schema，852 的 Rust/PowerShell 双消费者共享校验；本Plan不新增收据字段（否则会误伤已交付认证证据）。旧**报告**因轴新增而判 STALE 属预期，T-02 决定是否把该轴设为"可选上报"以降噪。
- cache/store：本Plan不改任何键（输出不变）；`MIR_LOWERING_VERSION` 只进报告。语义真变时另立版本迁移。

### 5.4 测试框架接入（既有 TestCase 协议，零改动）

- recipe、case/suite schema、语料内容、判定面全部不变；suite 仍由既有 `crates/auto-lang/src/tests/*_tests.rs` 调 `run_suite`。
- 等价性判定方式：同一 suite 在搬前/搬后各跑一次，比对报告中**逐 check 的 `status` 与 `actual`**（剔除时间戳/绝对路径/临时目录等易变字段；字段集由 T-02 冻结）。
- 若搬移期间发现某 check 状态变化，一律视为**回归**，不得以"重构导致"豁免。

### 5.5 风险

| 风险 | 处置 |
|---|---|
| 循环依赖（probe→lower→probe） | 前置核实：lowering 不得反向依赖 probe 的任何宿主类型；需要者以参数/回调倒置 |
| 搬移中夹带改造导致等价门失效 | 5.2 明确禁止；T-06 review 逐项核对"只搬不改" |
| 旧报告/收据被误判失效 | 收据不动；报告 STALE 按预期重跑；T-02 明确声明 |
| 三消费者各自出现本地副本 | T-04 加依赖方向守卫（`cargo tree`/清单断言）；review 复核 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-mir/hir-lowering-boundary.md | 无 → lowering 的模块边界、唯一实现位置、依赖方向、版本身份 | 生产语义不再由测试宿主提供 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-mir/overview.md | 追加 lowering 归属与版本轴，MIR 合同语义不变 | 可见性 | AC-02/04 |
| SD-03 | modify | docs/specs/auto-av/project.md | 追加"probe 不再是 lowering 提供者"的边界说明 | 事实纠正 | AC-01 |
| SD-04 | modify | docs/specs/auto-test-case/architecture.md | 追加报告身份轴（lowering 版本），判分与收据规则不变 | 可判定 | AC-04 |

## 6. 测试设计

| V | AC/不变量 | 实际/拟建入口 | 可区分错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 行为等价 | 搬前/搬后各跑 ace-m1 base suite + av-data(821/822/823/824) + av-abc(825/826) + ac-cte + ac-mn 各族 | 逐 check `status`+`actual` 完全一致；任一变化即回归 | T-04；不剔除任何 check |
| V-02 | AC-02 三消费者 | probe worker `case-serve`、mn driver、cte probe 各跑一组真实路径 | 三者均经新 crate 执行；`cargo tree` 证明无本地副本 | T-03/04 |
| V-03 | AC-03 依赖方向 | `auto-av-core` 不得依赖 `ac-core`；`auto-lang` 不得依赖实验栈 | 依赖清单断言；被破坏即红 | T-04 |
| V-04 | AC-04 身份轴 | 报告含 lowering 版本；旧报告 STALE；收据键集不变 | 轴缺失/漂移非绿；852 收据双消费者仍通过 | T-05 |
| V-05 | AC-05 反控 | 故意改一个 lowering 分支/常量（本地注入） | 语料必红 → 证明等价门有判别力；恢复后必绿 | T-05；注入缝仅测试用 |

定向命令（T-05 建立后填实际）：三消费者 `cargo check --locked` + 定向 `cargo test`；`powershell -File scripts/verify-mir-lowering-868.ps1 -Scope <equivalence|deps|identity|acceptance>`；完整 `cargo t` 归 merge。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 搬移前后全部既有语料逐 check 状态/actual 一致（无豁免）；V-01 |
| AC-02 | work | 单一共享实现；probe/mn/cte 三消费者皆经它执行，无副本/shim；V-02 |
| AC-03 | work | 依赖方向红线成立（core 无前端依赖、产品无实验栈依赖）；V-03 |
| AC-04 | work | lowering 版本轴进入报告；收据键集不变；旧报告 STALE 行为符合声明；V-04 |
| AC-05 | work | 反控可证等价门有判别力（改 lowering 必红）；V-05 |
| AC-06 | review | 独立核"只搬不改"、切面合理、无行为夹带、warnings/fmt；merge 门 pending |
| AC-07 | merge | 冻结提交裸 `cargo t` + 受触面档（VM/编译器→`tv`）通过 |
| AC-08 | merge | SD-01..04 沉淀、ledger、归档、guard 清理；并记录 862/863/864/865 的 lowering 写面已改指新 crate |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 无 | 核实切面：`hir_adapter` 与 probe 宿主类型的耦合、三消费者调用点、报告身份轴落点；产出 `attachments/868/work/baseline.md` | AC-01/02 | [x] 2026-10-28：`attachments/868/work/baseline.md`（work 树 452320fd5 实测）。核心事实：hir_adapter 零宿主耦合（仅 auto_ac_prototype+auto_av_core import，无 crate::/super::）→ 纯搬移成立，倒置预案不触发；接口面 9 个 pub 项；33 文件/61 引用点（probe src 9 / probe tests 17 / bins 2 / mn 4 / cte 1 / lib.rs 1）；身份轴落点 ac_av/ac_cte/ac_mn 的 identity+axes 函数与 stale.rs applicable 清单；ace 收据 schema 自包含不受轴影响；mn 无 stale 专属分支（落 sema fallback，T-04 实测） |
| T-02 | work | T-01 | 冻结边界与身份：crate 名/路径、公开 API 清单、报告轴键名与取值、等价比对字段集、收据不动的声明 | AC-02/04 | [x] 2026-10-28：冻结于 `attachments/868/work/baseline.md` §8。crate=`experimental/auto-mir-lower`（lib `auto_mir_lower`）；API=9 项原样+`MIR_LOWERING_VERSION: &str = "1"`；轴键 `mir-lowering` 必报（三消费者 identity+报告）；stale applicable 仅 av/cte 分支追加；比对字段集=逐 result 的 case/check/adapter/action/tier/status/mismatches/detail/identity/artifacts(logical,sha256,bytes)/native{exit_code,cache_mode,cache_hits,counters,gaps}，剔除 duration_ms/stage_spans/native.toolchain/artifacts.stored；收据键集零改动 |
| T-03 | work | T-02 | 建 crate + 搬 lowering（纯搬移）+ 三消费者改指 + 删旧实现（编译通过） | AC-02 | [x] 2026-10-28：commit `aaf22f279`（work 树 plan-868-dev）。git mv 100% rename（2627 行零改动）；`experimental/auto-mir-lower`（deps=ac-core+av-core）+ 根级 re-export；33 文件 61 引用全部改指（probe src `crate::hir_adapter::`→`auto_mir_lower::`、tests/bins/mn/cte 同构）+ lib.rs 模块删除 + 三 Cargo.toml 加依赖。编译：auto-mir-lower/ac-av-probe(ac-av-probe all-targets 8s)/ac-cte-probe 全绿；ac-mn lib+bins+触面测试绿（codegen_tier/transaction_tier 的 MirFunction.env 报错为 master 预存红：PLAN-823/824 加字段未更新 mn 测试夹具，两文件不在本 diff 内，记 KNOWN-DEBT） |
| T-04 | work | T-03 | 等价性运行与依赖守卫（V-01/02/03），记录搬前基线报告供比对 | AC-01/02/03 | [x] 2026-10-28：搬前基线 11 族（ac-av/av-data-821~824/av-abc-825/826/av-source-827/ace-m1/ac-cte/ac-mn）151/151 PASS；搬后同 11 族 run exit 全 0，`compare-reports.py` 冻结字段集比对 **11/11 族逐 check 全等**（status/actual/identity/artifacts sha/native counters 除 ledger_peak_* 等，attachments/868/work/{baseline,post}-reports/，本地证据）。依赖守卫：av-core 依赖清单空；auto-mir-lower 仅 ac-core+av-core；experimental 全域 `pub fn lower_checked(` 定义唯一；根 workspace 零 experimental path 依赖；`cargo tree -i auto-mir-lower` 三消费者均解析且无反向边 |
| T-05 | work | T-04 | 身份轴 + 反控注入 + 脚本 `verify-mir-lowering-868.ps1`（V-04/05），提交代码与证据 | AC-04/05 | [x] 2026-10-28：commit `89c551ee1`。轴：`EXPECTED_MIR_LOWERING_VERSION="1"` 镜像于 ac_av/ac_cte/ac_mn，入 identity/axes/双 mn 收据/NativeRecord；stale.rs av/cte applicable 追加。实测：新报告行轴含 `mir-lowering:1`（ac-av 18/18 绿）；旧（无轴）报告判 `native.mir-lowering missing (now 1)` STALE（7 行）；mn 收据带轴；ace 收据键集双消费者零改动。反控：注入 mark_a 1→3 于共享 crate → ac-mn 必红（run-first exit 期望 0 实得 1）、av 不敏感（marks 只进 trace 序，记入案）→ 还原必绿。脚本三档实测 PASS：deps（5 守卫）/identity（11 族重跑全绿+4 断言）/equivalence（对搬前基线 11/11 EQ）。scoped 定向测试（45 测批）45/45 绿 |
| T-06 | review | T-05 | 独立最终review（重点：只搬不改、切面、轴语义）；merge 门 pending | AC-06 | [ ] |
| T-07 | merge | T-06 pass | 冻结提交裸 `cargo t` + `tv` | AC-07 | [ ] |
| T-08 | merge | T-07 | Spec/ledger/归档/guard；回写 862–865 写面并解除其 lowering 前置 | AC-08 | [ ] |

## 9. 复审记录

2026-10-28 new r1：`stage: new | plan_id: PLAN-868 | plan_revision: 1 | outcome: pass（无硬前置，可在授权下进入 work） | next: work T-01`。全部 AC/V/T 未执行；crate 名/报告轴键名/等价比对字段集由 T-02 冻结。本轮未建树、未跑构建、未改任何实现与其他在途 Plan（844/861/867 不受影响，且与本 Plan 无共享写面）。

2026-10-28 work r1（execution_done 交接）：
`stage: work | plan_id: PLAN-868 | plan_revision: 1 | outcome: pass（work-owned 全完成） |
worktree: D:/autostack/.wt/lang-868/auto-lang | branch: plan-868-dev | base: 452320fd5 |
code_commits: aaf22f279（T-03 纯搬移）+ 89c551ee1（T-05 轴+脚本） |
dependency_revisions: auto-ui 7095f53（组内 detached 兄弟）、auto-down c3464d0（同）——仅构建解析用，非代码依赖 |
task_ids: T-01..T-05 | evidence: 见 §8 各行 + attachments/868/work/baseline.md（§6/§8/§9 含 151/151 双腿、11/11 冻结字段集全等、反控红绿、三档 verify 脚本 PASS） |
blockers: 无 | next_responsible_role: review（T-06）`

- **AC-01/02/03/04/05 全部 work-owned 达成**，证据逐条在 §8 与 baseline.md。
- **merge-owned 门禁 pending**（AC-07/08）：冻结提交裸 `cargo t` + `cargo tv`；SD-01..04 沉淀、ledger、归档、guard 清理、862–865 写面回写。
- 发现并归因的三项**预存**问题（均非本 diff 引入，记 KNOWN-DEBT 候选，本 Plan 不修）：
  1. stale.rs applicable 与 av 族报告轴脱节（`av-core-version` 等多轴漂移；data/owned 族与 mn 行落 sema fallback 分支；cte 报告行缺 native.toolchain）——搬前基线即全红，本 Plan 仅按声明追加 `mir-lowering`。
  2. ac-mir-native `codegen_tier`/`transaction_tier` 构造 `MirFunction` 缺 `env` 字段（PLAN-823/824 加字段未同步 mn 测试夹具）——`cargo check --all-targets` 在 master 同样红。
  3. `mir_native_rounds_failure_liveness_and_caps` 为 3 秒墙负载敏感 flake：本会话 57 测并发批 3 红、45 测同型批全绿（本树/基面 experimental/主检出三处对照）+ 隔离 2 绿 + 故障路径不经过搬移代码（协议前 hijack）；归因=并行重载下 3s 墙超时，与本 diff 无因果。
- 测试运行记录（本次 work 全量）：三棵 experimental `cargo check`（触面）；11 族 suite×3 腿（搬前/搬后/轴后，各 151 checks 全 PASS）；verify 脚本 deps/identity/equivalence 三档；scoped 45 测批×3 树对照 + 57 测批 1 次（含上述 flake 归因）。统一回归门禁（裸 cargo t/tv）**pending，owner=merge**。

## 10. 待澄清事项

- ~~crate 名与位置~~：T-02 冻结 `experimental/auto-mir-lower`（已落地）。
- ~~身份轴必报 vs 可选~~：T-02 冻结必报（已落地；旧报告 STALE 判定实测符合声明）。预存 stale applicable 脱节问题升 KNOWN-DEBT（见 §9），后续修复归 stale 机制维护计划。
- ~~深度耦合倒置~~：T-01 证实零宿主耦合，纯搬移成立，无需倒置。
