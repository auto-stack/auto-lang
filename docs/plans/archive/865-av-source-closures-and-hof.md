---
plan_id: PLAN-865
status: archived
completion_kind: delivered
feature_name: AV闭包源码语法与间接调用（R2-L 闭包）
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-29
plan_revision: 2

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-av/closure-environments.md
  - docs/specs/auto-mir/closure-capture-profile.md
  - docs/specs/auto-test-case/ace-m1-conformance.md
new_spec_components:
  - docs/specs/auto-ac/source-closures.md
touched_goals: [GOAL-002]

affects: [auto-av, auto-mir, auto-ac, auto-test-case]
current_step: 9
total_steps: 9
---

# [PLAN-865] AV闭包源码语法与间接调用（R2-L 闭包）

## 0. 变更摘要

PLAN-823 已交付执行侧 `av.closures/1`（profile id 6、AVAD v5、TY_CLOSURE tag 17）：typed owned 闭包环境、move/显式 clone 逃逸捕获、静态可证非逃逸同步借用、受检间接调用与 env/frame 身份，并明确排除**源码闭包语法/前端推导**（[auto-av/project](../../specs/auto-av/project.md)、[closure-environments](../../specs/auto-av/closure-environments.md)）。

852 语料把 K02/K05 登记为 `diff-legacy-grammar(parse)`：两条 legacy 路径**都不能解析 move 闭包语法**，owner 写为 `823-future-source`。本Plan承接该缺口：为闭包建立源码域（语法/捕获分析/间接调用/逃逸诊断），使 K01–K06 从 typed 证据变为**真实源码执行**。

不新增执行语义：closure 的 env 布局、捕获模式、CLONE/pin/escape 规则沿用 823 冻结合同。

明确不含：共享可变捕获与循环引用（C-19，归 R5-O）、async/生成器、跨线程、native 闭包 ABI、持久闭包、以及 `map/filter/reduce` 等 HOF 库面（见 §10，需库/方法派发裁定，另立切片）。

## 1. 目标

- 源码可写 `(x int) => { … }`、`move (x int) => { … }`、零参/多参 lambda，并可作为值传递、赋值、返回、按名调用。
- 捕获分析在编译期完成：move/owned 捕获、显式 clone、静态可证非逃逸借用；逃逸/跨生命周期使用**编译期具名拒绝**，不靠运行期兜底。
- 间接调用经 823 `CalleeRef` 形状进 ABC 并可执行；env 声明与链接期等价检查保持（声明漂移精确拒绝）。
- 冻结 **SEM-04**：Copy 捕获 / owned 捕获 / 显式 clone 的可观察差异与首批拒绝面，禁止用深 clone 静默改变共享效果。
- K01–K06 原源全链执行或诚实拒绝（带 owner），并把 `823-future-source` 关闭为实际交付。

## 2. 架构方案

| 层 | 本Plan动作 | 边界 |
|---|---|---|
| 源码前端 | 新 `SourceProfile`（含 862 入口、864 集合面）+ lambda/捕获/调用语法与逃逸分析 | 不引入共享可变捕获、async、方法 HOF |
| HIR | 新 descriptor；闭包表达式携带 env 声明（与 823 一致） | 不改旧 descriptor |
| MIR | 复用 823 `CREATE_CLOSURE`/间接调用与 env 字段；捕获按值/按引用模式由分析决定 | 不新增闭包执行指令 |
| 准入 | env 声明与 `fn:` 绑定等价检查在 link 期；捕获模式/逃逸在 verify 期 | 不允许“先执行再看” |
| 会话 | 扩 823 `ClosuresUpdateSession` 到源码前端（828 形状） | 身份/代际/授权规则不变 |

## 3. 技术栈

- 源码：`experimental/ac-core/src/source/*`、`src/{hir,descriptor,verify}.rs`。
- lowering写面：`experimental/auto-mir-lower/src/**`；会话接线`experimental/ac-av-probe/probe/src/{closures_session,session}.rs`，消费822/823执行合同；核心`core/src/{mir,closures,abc,abc_verify,av}.rs`若需触及须登记S及不变语义证明。
- 语料：`crates/auto-lang/test/cases/ace-m1/src/K0*.at`、`binding.tsv`、`crates/auto-lang/src/tests/ace_m1_tests.rs`。
- 拟建 `scripts/verify-av-865.ps1`（Scope=targeted/compat/closures/acceptance），前缀 `av_closures_865`；当前不存在。

唯一实施树 `D:/autostack/.wt/lang-865/auto-lang`，分支 `plan-865-dev`。

## 4. 需求分析与背景调查

用户 2026-10-28 授权同上（只规划）。事实基础：

- [closure-environments](../../specs/auto-av/closure-environments.md) 与 [closure-capture-profile](../../specs/auto-mir/closure-capture-profile.md)：owned env、move/显式 clone、静态非逃逸 borrow、间接调用、env/frame 身份与 A23 代码层恢复已冻结；源码语法/推导/共享 RC/async 为显式非目标。
- [resident-function-install §823](../../specs/auto-av/resident-function-install.md)：闭包 body 自带 `env` 声明，`CREATE_CLOSURE.callee` 走 `CalleeRef`；link 期 `fn:` 绑定做 env 等价检查，声明漂移精确拒绝。
- 852 binding：K01（move 捕获）typed 行 `ok:av-closures-823`；K02/K05 两条 legacy 路径 `diff-legacy-grammar(parse)`，注记“owner 823 freezes source syntax”；K03/K04/K06 已有 typed 证据与借用/逃逸负控。
- 语料 `src/K01.at`：`let f = move (n int) => { return base + n }`；`src/K02.at` 与 `src/K05.at` 使用同一 move-closure 形态。
- SEM-04：823 首批为 move/显式 clone 的 owned 闭包；旧调用面含隐式捕获/可变共享与 alias → 必须按用例裁定，不能用深 clone 统一。

本次授权与输入：用户明确要求修订波次/实际写面、补871旧能力覆盖及删除门、让864支持List与拥有型type元素；允许动作仅计划/设计/战略文档修订，不实施代码、不调整预算、不改868。基面路径、内容SHA-256、旧稿快照及静态预检见[修订收据](../attachments/ace-wave-20261028/revision-summary.md)；观察到的current Spec/调用点与未来拟建入口分开记录。

2026-10-29 goal 授权：用户通过 `/goal 完成计划865` 授权按本计划 r2 完整交付，包括在专用 worktree 实施、独立复审、最多三轮有依据修复及合并归档；该授权不改变目标、验收标准、阈值、仓库范围或预算。此前“仅计划修订”的范围已由本次明确交付授权取代。

## 5. 详细设计

### 5.0 波次、实际写面与交接（本次修订）

按[波次交接合同](../reports/ace-wave-handoff-2026-10-28.md)§1–2执行：S为共享source/HIR/lowering/MIR/ABC/AV及会话接口，N为ac-mn，H为公共driver/SDK适配/测试注册与binding；分别单写，跨窗同时取窗。共享lowering的未来写面为D868后的`experimental/auto-mir-lower/src/**`，旧`probe/src/hir_adapter.rs`仅作搬前调查，不能再落新实现。T-02及接线前核验实际已合入revision、Spec/API/版本、owner释放与消费者兼容证据并记录接受；review/repair未完不交WIP。

第3波在D864等真实前置满足后，S默认866释放后接本Plan，再释放给863；866→本Plan→863是写面排序，不新增D866语义依赖。共享source/lowering/closures会话和H绑定均按窗交接；可在先期只读准备独立捕获分析测试材料，不提前注册公共定义或复制上游WIP。D864交付的List<Str>/List<拥有型type>仍须按823合法捕获类型范围逐项核验，不把集合支持自动推成所有闭包组合。

首例即按既有文件化AutoDown/TestCase、cache fresh/hit/restart/body-edit、未改函数生产工作=0与现行P90≤2000ms口径验证；亚秒目标及P95/P99另列，不能伪称当前硬门已是亚秒。内存/资源峰值、live/回收及cache容量在T-02沿上游预算交集冻结，记录命令、样本和反控，不擅自提高上限。共享核心/SDK/测试迁移先确认拆分owner实际位置与释放；产品默认入口、旧AutoVM/a2r和868合同/进度不在本次修订写面。

### 5.1 前置与写面

硬前置：`D862`（入口）、`D864`（集合源码，供捕获 owned 值与 HOF 后续切片）、`D823`（已具备）、**`D868`（HIR→MIR lowering 出库）**——闭包捕获/lowering 落共享 crate。写面见 §3。

### 5.2 源码语义冻结（T-02）

`../attachments/865/work/closures-source-contract.md` 冻结：

| 决策项 | 要求 |
|---|---|
| 语法 | `(params) => expr` 与 `(params) => { stmts }`；`move` 前缀位置；零参/多参；作为表达式出现在 let/赋值/实参/return |
| 捕获 | 捕获哪些自由变量由实际使用决定；`move` 捕获为 owned 转移（原变量 move 后不可再用，须具名诊断）；非 move 捕获只允许**静态可证非逃逸同步借用**；其余在编译期拒绝 |
| clone | 显式 `.clone()` 产生独立 env（可观察独立性）；不得自动插入 clone 改变共享效果 |
| 逃逸 | 返回闭包/存字段/跨调用保存的诊断规则；借用捕获逃逸=拒绝（对应 K04） |
| 调用 | 直接调用 lambda 值与通过变量间接调用；元数/类型检查；高阶参数（函数作实参/返回）的首批形态 |
| 环境身份 | env 声明与 823 逐位一致；捕获顺序/字段命名规则固定，避免同一源在不同实现下 env 漂移 |
| 增量 | body 编辑/捕获集合变化/env 漂移三类编辑的失效判定（env 漂移必须精确失效，不是整备） |
| 拒绝面 | 共享可变捕获/循环引用/async/跨线程/持久闭包 → 具名 unsupported |

### 5.3 分析与 lowering

- 捕获分析在 typecheck 后、HIR 前完成（或按 ac-core 现有阶段确定，T-01 核实），输出捕获列表与模式；不可在 lowering 期补猜。
- lowering：`CREATE_CLOSURE` + env 初始化（按 823 字段序）+ 间接调用；禁止把闭包降成“带隐藏参数的普通函数”绕过检测（会产生不同可观察性）。
- 逃逸/借用：保守拒绝优先；误拒是允许的（可后续放宽），误放是缺陷。
- 会话：closures 源码会话消费 828 单元接口；env 漂移走精确失效（823 V-13 四子case 形状）。

### 5.4 测试框架接入（既有 TestCase 协议，不新造框架）

K 族目前走 typed fixture（`ac-av/closures-run-v1` + `av.entry: "closures:<id>"`，父侧 `CLOSURES_CASES`），而 852 把 K02/K05 的 legacy 侧登记为 `diff-legacy-grammar(parse)`。本Plan接入**真实源码**：

- case 形态：追加 source 行（复用 `ace-m1/src/K0*.at`），av check 走源码入口（`d_main`）；legacy avm/a2r 的 `diff-*` 行**保持如实差异**，不因新源码路径可跑就改写为绿。
- recipe：新注册 `ac-av/closures-source-run-v1`（三缝 append-only + worker 源码域分支），不改 `closures-run-v1` 的 fixture 语义。
- 期望落点：`expect.exit` / 错误码进 case；env 漂移与寿命类断言（父侧表）沿用 823 `judge_*` 形状扩展，声明即判。
- `binding.tsv` 只改状态/备注，schema 不变。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/source-closures.md | 无 → 闭包源码域：语法/捕获/逃逸/调用/诊断/SEM-04 | 源码闭环 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-av/closure-environments.md | 追加 source 绑定与实际覆盖，typed-only 边界改写 | 事实纠正 | AC-02/05 |
| SD-03 | modify | docs/specs/auto-mir/closure-capture-profile.md | 追加源码捕获分析到 MIR 的映射约束 | IR 合同 | AC-03 |
| SD-04 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | K 行重绑定与实测映射 | 证据落点 | AC-05 |
| SD-05 | modify | docs/design/strategy/av-language-capability-matrix.md | before: C-18 AV源码仅“865已认领/待源码”，C-11仅有823已声明闭包签名； after: C-18记录865已交付源码lambda/捕获/间接调用且保留共享可变/async等拒绝，C-11记录函数类型闭包参数/返回与间接调用；默认/变参、具名函数项值和集合方法HOF仍未交付 | 路线事实 | AC-05 |

## 6. 测试设计

| V | AC/不变量 | 实际/拟建入口 | 可区分错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 语法/调用 | K01–K06 原源；零参/多参/直接与间接调用正例 | 手算 exit 值；env 字段序漂移/元数错必红 | T-03 |
| V-02 | AC-02 捕获/逃逸 | move 后读取（K/源级）、借用逃逸（K04）、共享可变/async 拒绝、clone 独立性 | 编译期具名拒绝（非运行期）；clone 独立 oracle 精确 | T-03/04 |
| V-03 | AC-03 环境一致性 | env 声明漂移、捕获集合改变、签名改变 | 漂移精确失效；旧调用保活；不得整备 | T-04 |
| V-04 | AC-04 资源/寿命 | 创建者退出后调用、旧帧/旧代回收、失败候选 | 无泄漏/无悬挂；Weak oracle 释放 | T-04 |
| V-05 | AC-05 语料绑定 | K 行 + 独立 oracle + 分母审计 | SKIP 不计绿；legacy `diff-*` 如实保留 | T-05 |
| V-06 | AC-04/AC-06 写面/兼容 | T-02及公共接线交接摘要；已交付消费者并集与窗口清单 | 真实D门/释放/接受齐全；缺交接或消费WIP不准接线；受影响profile兼容验证及首例cache/周转/内存证据齐全 | T-02冻结；最终work任务执行；review复核 |

定向命令：两独立 workspace 定向 `cargo test`；`scripts/verify-av-865.ps1 -Scope <targeted|compat|closures|acceptance>`；完整档归 merge。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 闭包源码语法/调用按冻结件成立，K 源全链执行；V-01 |
| AC-02 | work | 捕获/逃逸/clone 语义与拒绝面成立（编译期拒绝），SEM-04 落 Spec；V-02 |
| AC-03 | work | env 声明与 link 等价检查保持，env 漂移精确失效；V-03 |
| AC-04 | work | 创建者退出/旧代回收/失败候选资源正确；V-04 ；首例cache/文件化测试、周转/内存预算及兼容证据齐全，V-06 |
| AC-05 | work | 852 K 行与独立 oracle 落地，`823-future-source` 关闭；V-05 |
| AC-06 | review | 独立核 AC/V/SD、捕获分析是否在正确阶段、profile 6 零漂移、warnings/fmt；核V-06交接与首例指标 |
| AC-07 | merge | 冻结提交裸 `cargo t` + `tv` 通过 |
| AC-08 | merge | Spec/矩阵/ledger/归档/guard 清理完成 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | D862、D864、D823 | **[x]** 基线：D862/D864/D868 已归档；HIR 已有 `TypeKind::Function(FnSig)`，但 source AST/VT/parser 无 lambda 或闭包类型；MIR 已有 `CreateClosure`/`CallClosure` 与 CalleeRef，`ClosuresUpdateSession::apply_revision` 可消费闭包 MIR；K01–K06 原源示例存在，K02/K05 使用 move lambda；MIR List/Variant 元素域允许标量、Str、Record、Variant，不允许嵌套 List/Fn/borrow view。823 owned env/capture 及 link 等价合同沿用。写树 `D:/autostack/.wt/lang-865/auto-lang`，base `5af8e0cbe3f5a5a4971d5142abb4fb53b0f983b4`，auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` | AC-01/02 | [x] |
| T-02 | work | T-01 | **[x]** 冻结语法、`fn(...)->...` 高阶签名、Copy/owned/clone/borrow SEM-04、捕获顺序、823 env/link 规则、D864 List 交集、具名拒绝面及 V-06 计量口径；交接沿 §5.0，未提高预算。合同 [closures-source-contract.md](../attachments/865/work/closures-source-contract.md)，SHA-256 `3b13327d8c6ebc68f42b7b643e169f32fdcaf6f59268dfdb303aa03fa0ae2d00`，提交 `0275805f0` | AC-01..05 | [x] |
| T-03 | work | T-02 | **[x]** `experimental/ac-core/src/source/{parser,resolve,typecheck,closure_capture}.rs`、HIR/descriptor/schema 接入 lambda、函数类型、捕获与具名拒绝；修复 R865-F01/F02；surface 9 tests pass，源会话零参/多参/直接调用与元数、签名、async负例通过；R865-M01：`ac-semantics` 限定适配器显式拒绝函数类型/间接调用/闭包表达式，保留旧 `VT::Function` 分类 | AC-01/02 | [x] |
| T-04 | work | T-03 | **[x]** `lower_function_closures_unit` 接入 `experimental/auto-mir-lower/src/hir_adapter.rs`；源码会话走 823 `ClosuresUpdateSession`，env 漂移保留旧 creator，release/live 与分配平衡在 K 源父侧检查；lowering 2 项、会话/捕获 5 项曾通过；修复 R865-F03；捕获集与签名源编辑分别只重编 creator 与 closure，untouched 单元保持缓存 ；R865-F05已闭合：拥有型闭包由 make 返回后传入 apply 的 fn 参数并在参数体内间接调用，session结果42且释放平衡 | AC-03/04 | [x] |
| T-05 | work | T-04 | **[x]** 新增 K01–K06 源码 TestCase、父侧 oracle、`binding.tsv` AV 状态及 `scripts/verify-av-865.ps1`；K02/K05 legacy diff 保持，K06 标注 `gap:R5-O`；执行 V-06 file cache、restart、body-edit、P90/热集和 823 兼容验证，原始证据仅本地；受影响 AC-01..03 已定向重跑；K01-K06 与 V-06 证据沿用已通过记录 ；依赖复跑 K01-K06 file-backed TestCase：8 checks 全 PASS、coverage gaps=0 | AC-05/04 | [x] |
| T-06 | review | T-05 | **[x]** final artifact-based review pass on `7491be94`; AC-07/08 remain merge-owned closeout gates | AC-06 | [x] |
| T-07 | merge | T-06 pass | **[x]** 冻结代码/Spec 提交 fe2923ba9 + ledger projection SHA-256 718255cc08a50ec2af79597be78cdefb78adab19925a9af67016cdc2af73f722：裸 `cargo t` 3207/3207 PASS（673 skipped，1 slow），`cargo tv` 163/163 PASS（2061 skipped）；此前超时项空闲定向复跑 1/1 PASS | AC-07 | [x] |
| T-08 | merge | T-07 | **[x]** Spec/矩阵/索引与 ledger v139→v140 读回一致；106 条历史兼容修复；补充既有 823/852 索引；新增 6 条 865 条目；归档链接 13/13 有效；Windows guard clean | AC-08 | [x] |
| T-09 | merge | T-08 | **[x]** PLAN-863 保持 drafting，以 delivery baseline 5fa3133fa6401904b0760986ecb2e83b0534cbd4 接续；HOF（map/filter/reduce）留给独立 R3-H/stdlib 切片，C-19 共享可变捕获仍由 R5-O 裁定 | AC-08 | [x] |

T-07 merge gate receipt (2026-10-29): exact commands `cargo t` and `cargo tv` passed on the reviewed code/spec content plus the read-back-validated v140 ledger projection; filtered replay of ac_av_probe_mixed_recipe_stale_identity_is_order_independent passed 1/1. Full results: 3207/3207 (673 skipped; 1 slow) and 163/163 (2061 skipped). No implementation files changed after these gates; at that checkpoint T-08/T-09 remained merge-owned and are now completed in the archive checkpoint below.

## 9. 复审记录

`stage: new | plan_id: PLAN-865 | plan_revision: 2 | outcome: pass（修订草案静态预检；实施仍等真实D门/写面释放） | next: D864等前置后T-01/T-02，S按866→865交接`

2026-10-28 r1→r2（用户授权修订波次/写面，864纳入拥有型List、871补旧能力与删除门）：仅修订执行合同，不实施。原status/current_step/total_steps、任务勾选与历史记录保留；D868合同和进度不变。V-06补交接/消费者兼容与首例cache、文件化测试、周转及内存核验；详细依据与静态预检见[后继批次排序报告](../reports/ace-next-batch-2026-10-28.md)和[PLAN-865波次静态校验收据](../attachments/ace-wave-20261028/static-validation.json)。本记录只说明文档修订，未执行AC、性能测试或独立实现review，merge-owned门禁仍待实施后完成。

2026-10-29 work：T-03/T-04/T-05 已完成；执行提交 `7d708e36f51e75686f77a28e50734220935ea4f0`（parent `f49c28054c1c1d09b29a68a457672fdbab4ec383`），实现树 `D:/autostack/.wt/lang-865/auto-lang`，branch `plan-865-dev`；依赖 revision：auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`、测试解析所需 auto-ui `0261ae9dacf786beb6dceb630be6af267f242625`（均未改动）。

验证：`powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-av-865.ps1 -Scope acceptance` exit 0；source surface 8、MIR lowering 2、parent oracle 1、冻结 `closures-run-v1` compat 1、source/session 5、K01–K06 文件 suite 1 均通过。K suite 8 checks 全 PASS、coverage gaps=0；file cache fresh=2 miss/2 writes，跨 worker restart=2 hits/0 writes，body edit 只提交/verify/encode/admit 1 个 closure unit。20 个真实 TestCase worker 样本：P90 request wall `271.3219 ms`（门槛 2000 ms），最大进程峰值 `1,413,120 bytes`，最大 current+history 热集 `4,933 bytes`（限额 8 MiB）。原始本地证据 `../attachments/865/work/source-cache-metrics.json`，SHA-256 `0029b818b947eb0eaa17695916c12e44d74984a67ff72bc25a26d414b5ec2591`；不入库。worker release build 通过；现有 `owned_session.rs` dead-code warning 不在本次改动内，留 AC-06 复核基线确认。

handoff：`stage: work | plan_id: PLAN-865 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-865/auto-lang | branch: plan-865-dev | base_commit: f49c28054c1c1d09b29a68a457672fdbab4ec383 | code_commit: 7d708e36f51e75686f77a28e50734220935ea4f0 | dependency_revisions: auto-down=c3464d0a0d36e4f8a7988b0c0c26448fc52361b4, auto-ui=0261ae9dacf786beb6dceb630be6af267f242625 | task_ids: T-03,T-04,T-05 | evidence: verify-av-865.ps1 acceptance exit=0; P90=271.3219ms/20 samples; hot peak=4933B; local evidence SHA-256 above | blockers: none | next_responsible_role: review`。Work-owned AC-01..05/V-01..06 已交付；K06 共享可变捕获按原合同诚实拒绝并保留 owner R5-O，HOF 仍排除。AC-06 独立复核与 merge-owned `cargo t`/`tv`、Spec/ledger/归档/guard 均未完成，分别归 review/merge。

2026-10-28 new r1：`stage: new | plan_id: PLAN-865 | plan_revision: 1 | outcome: pass（等 D862/D864，合同可预检） | next: D864 后 work T-01`。全部 AC/V/T 未执行；语法/捕获规则/诊断码待 T-02 冻结；未建树、未跑构建、未改在途Plan。批次排序依据见[后继批次规划](../reports/ace-next-batch-2026-10-28.md)。

stage: goal | plan_id: PLAN-865 | plan_revision: r2 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-865/auto-lang | commit: 7d708e36f51e75686f77a28e50734220935ea4f0 | next: review

2026-10-29 final review：按提交代码、断言、工作合同及当前 Spec 输入重建结论；本复审与实现处于同一会话/上下文，不声称独立。冻结的 §5 规范增量表（UTF-8，844 bytes）SHA-256 6fddd2de4a83ab66b00ded12cb0340744fa1d6f00071cd43478eaf6054603d08。输入：T-02 contract 3b13327d8c6ebc68f42b7b643e169f32fdcaf6f59268dfdb303aa03fa0ae2d00；closure-environments c93267790454ec719f4ddd7595386b1e1cca036688696b45caaa620c63e5340f；MIR closure profile 9b8ac7d6a70da6c5ce27c98a829b3ae20a74345abba1fa3fd2e2f307df4d8dea9；ACE M1 Spec（main 当前版含 PLAN-887）6fc7d890335ccb0b4a58533edbbb8e5711763766ef2c0c77f8c783bd7a4f0b32；capability matrix（main 有无关用户 WIP，C-18 行仍是修订前状态）ae4bbc1e573f725ce4797e28efbca46a15dfd4bdb368dbe06faeaf0480cf508；SD-01 目标当前不存在。

stage: review | review_scope: final | reviewer_identity/limitations: same implementation context; independent review not claimed | plan_id: PLAN-865 | plan_revision: r2 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-865/auto-lang | branch: plan-865-dev | reviewed_commit: 7d708e36f51e75686f77a28e50734220935ea4f0 | base_commit: f49c28054c1c1d09b29a68a457672fdbab4ec383 | dependency_revisions: auto-down=c3464d0a0d36e4f8a7988b0c0c26448fc52361b4, auto-ui=0261ae9dacf786beb6dceb630be6af267f242625 | spec_inputs: T-02-contract=3b13327d8c6ebc68f42b7b643e169f32fdcaf6f59268dfdb303aa03fa0ae2d00; SD-01=absent; SD-02=c93267790454ec719f4ddd7595386b1e1cca036688696b45caaa620c63e5340f; SD-03=9b8ac7d6a70da6c5ce27c98a829b3ae20a74345abba1fa3fd2e2f307df4d8dea9; SD-04=6fc7d890335ccb0b4a58533edbbb8e5711763766ef2c0c77f8c783bd7a4f0b32; SD-05=ae4bbc1e573f725ce4797e28efbca46a15dfd4bdb368dbe06faeaf0480cf508; frozen_SD_delta_SHA256=6fddd2de4a83ab66b00ded12cb0340744fa1d6f00071cd43478eaf6054603d08 | acceptance_results: AC-01 partial (V-01, R865-F01); AC-02 partial (V-02, R865-F02); AC-03 partial (V-03, R865-F03); AC-04 pass (K02 creator-exit source run, successful-session live=0/allocs=frees, env-drift refusal preserves installed creator; existing 823 typed lifecycle coverage); AC-05 pass (K01-K06 file suite, 8 checks/0 coverage gaps, legacy diff rows preserved); AC-06 needs_fix | pending_closeout_gates: AC-07 bare cargo t + tv; AC-08 canonical Specs/ledger/archive/guard | findings: R865-F01 (AC-01,T-03,V-01): current committed source tests parse zero-arg/direct lambda forms but do not compile and execute them; executable source examples use one-parameter closures only; no multi-parameter successful call or wrong-arity negative is asserted. Add source-to-session direct/zero/multi-parameter positives and an arity negative with diagnostic oracle. R865-F02 (AC-02,T-03/T-04,V-02): K05 returns 42 and checks release balance but cannot distinguish a deep-cloned closure env from an aliased env; typed 823 clone oracle exists but the new source path has no distinguishing clone assertion. Add a source-level clone oracle whose result changes if env is shared, a closure function-signature mismatch negative with a named diagnostic, and an async-closure source negative with the frozen named unsupported-closure diagnostic. R865-F03 (AC-03,T-04,V-03): source-session tests cover body-only edit and hand-mutated one-field env type drift; no source edit changes the capture set or closure signature and asserts precise changed-unit invalidation/old revision behavior. Add those source-session edits and expected unit/oracle checks. | evidence: clean implementation HEAD 7d708e36f51e75686f77a28e50734220935ea4f0; verify-av-865.ps1 -Scope acceptance previously recorded exit=0 on that commit; checks mapped against closures_865_surface.rs, closures_865_lowering.rs, av_closures_865_source.rs, K01-K06 and current Spec hashes above; source tests shown are missing the V-01/V-02/V-03 oracles stated in §6/T-02. Main checkout contains unrelated dirty documentation/user WIP; it is preserved and was not edited except this Plan. | next: work T-03/T-04/T-05

stage: goal | plan_id: PLAN-865 | plan_revision: r2 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-865/auto-lang | commit: 7d708e36f51e75686f77a28e50734220935ea4f0 | next: review

stage: goal | plan_id: PLAN-865 | plan_revision: r2 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-865/auto-lang | commit: 7d708e36f51e75686f77a28e50734220935ea4f0 | next: work


2026-10-29 repair round 3 (R865-F01..03): commit eda2512ffc39ea474d3cffea24b4af9de3110b5d (parent 7d708e36f51e75686f77a28e50734220935ea4f0) closes all three review findings. F01 adds source-to-session zero-parameter, multi-parameter and direct-lambda executions plus wrong-arity rejection. F02 adds an observable independent-list clone oracle, signature-mismatch rejection and a named async-closure rejection. F03 changes the closure capture set and signature through source edits, asserts exactly the creator and closure units are rebuilt while d_untouched is retained, then verifies a rejected candidate leaves the prior valid source runnable. Lowering now includes imports only for call/create-closure references, preventing unrelated function signatures and envs from changing the unit fingerprint.

Focused verification on the repair head: closures_865_surface 9/9; closures_865_lowering 3/3; av_closures_865_source 8/8; the filtered capture/signature invalidation case 1/1. rustfmt --check --edition 2021 on all five changed Rust files and git diff --check pass. Package-wide probe cargo fmt --check still reports pre-existing formatting diffs in unrelated files; those files were not changed. The pre-existing owned_session.rs dead-code warning remains outside this repair.

F865-F01 | outcome: closed | evidence: source zero/multi/direct execution and arity diagnostic in av_closures_865_source.rs; signature/async diagnostics in closures_865_surface.rs.
F865-F02 | outcome: closed | evidence: clone independence returns 11 (aliased env would return 12); mismatched closure signature and async source diagnostics asserted.
F865-F03 | outcome: closed | evidence: capture-set and signature source edits each rebuild only d_main and the closure unit; d_untouched remains cached and the prior valid revision remains executable after rejection.

stage: work | plan_id: PLAN-865 | plan_revision: r2 | round: 3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-865/auto-lang | branch: plan-865-dev | base_commit: f49c28054c1c1d09b29a68a457672fdbab4ec383 | code_commit: eda2512ffc39ea474d3cffea24b4af9de3110b5d | repairs: R865-F01,R865-F02,R865-F03 | verification: surface=9/9; lowering=3/3; source=8/8; incremental-filter=1/1; changed-files-rustfmt=pass; diff-check=pass | blockers: none | next_responsible_role: review

stage: goal | plan_id: PLAN-865 | plan_revision: r2 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-865/auto-lang | commit: eda2512ffc39ea474d3cffea24b4af9de3110b5d | next: review
2026-10-29 final re-review: reviewed commit eda2512ffc39ea474d3cffea24b4af9de3110b5d against base f49c28054c1c1d09b29a68a457672fdbab4ec383. This is the same thread/context as implementation; independence is not claimed. The prior R865-F01/F02/F03 closures are confirmed by source tests and focused replay. Current module Spec inputs remain unchanged: closure-environments c93267790454ec719f4ddd7595386b1e1cca036688696b45caaa620c63e5340f; closure-capture-profile 9b8ac7d6a70da6c5ce27c98a829b3ae20a74345abba1fa3fd2e2f307df4d8dea9; ACE M1 6fc7d890335ccb0b4a58533edbbb8e5711763766ef2c0c77f8c783bd7a4f0b32; capability matrix ae4bbc1e573f725ce4797e28efbca46a15dfd4bbd368dbe06faeaf0480cf508; T-02 contract 3b13327d8c6ebc68f42b7b643e169f32fdcaf6f59268dfdb303aa03fa0ae2d00. SD-01 remains absent; current extracted delta table is 831 UTF-8 bytes, SHA-256 060075a5ddc491809c0064c10cb6a1930a81393fc3992c3271981cedbfa1aa69.

Acceptance replay: AC-01 pass (surface 9/9; source/session 8/8, including executable zero/multi/direct calls and arity negative); AC-02 pass (source clone oracle returns 11, aliased env would return 12; named signature and async diagnostics); AC-03 pass (capture-set and signature edits rebuild exactly creator+closure, leave d_untouched cached, and a rejected candidate leaves the old valid source runnable; env-drift control still passes); AC-04 pass (source/session release oracle and file-backed K suite pass); AC-05 pass (verify-av-865.ps1 -Scope closures exit 0; source/session 8 passed; K suite 1 passed with 8 rows and zero coverage gaps; fresh=2 misses/2 writes, restart=2 hits/0 writes, body edit=1 unit; 20-sample P90=260.4487ms, max peak=1,101,824 bytes, max hot set=4,483 bytes; local evidence ../attachments/865/work/source-cache-metrics.json SHA-256 1a52d883c8ae2f5c302eb3a86153cafc1f8bbede112d4d497ebd353d0122890d). AC-06 needs_fix: rustfmt --check --edition 2021 over the 28 Rust files changed from base to reviewed HEAD checked 28 files and failed 7. The committed PLAN-865 additions in ac_av.rs, test_cases/mod.rs, ace_m1_tests.rs, av_probe_worker.rs, closures_865_support.rs, closures_session.rs, and lib.rs contain formatting diffs. This violates the existing AC-06/AGENTS clean-formatting review gate. Formatting is the established cause; no semantic regression was observed. Unrelated package-wide formatting differences exist in untouched files and are outside this finding. A pre-existing owned_session.rs dead-code warning is unchanged and not introduced by this Plan. AC-07 (bare cargo t + tv) and AC-08 (canonical Specs, ledger, archive, guard) remain merge-owned pending.

R865-F04 | type: original requirement gap | severity: P2 | acceptance/task: AC-06/T-04/T-05 | command: rustfmt --check --edition 2021 on Rust paths in base..eda2512ffc39ea474d3cffea24b4af9de3110b5d | expected: all Plan-modified Rust files pass formatting | observed: 7 of 28 checks fail with diffs in the added T-04/T-05 implementation and harness files, including module ordering in lib.rs | cause: committed Plan additions were not formatted consistently | correction: format only the changed Rust files on the Plan branch, inspect the resulting diff for formatting-only scope, and rerun the changed-file formatter check; leave unrelated baseline files untouched | controls: affected source tests and file-backed K suite pass; git diff --check base..HEAD passes | closure: closed in repair round 5; seven flagged files formatted, all 28 Plan-modified Rust paths pass rustfmt, and base..HEAD diff-check passes.

stage: review | review_scope: final | reviewer_identity/limitations: same thread/context as implementation; independence not claimed; artifact-based re-review | plan_id: PLAN-865 | plan_revision: r2 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-865/auto-lang | branch: plan-865-dev | reviewed_commit: eda2512ffc39ea474d3cffea24b4af9de3110b5d | base_commit: f49c28054c1c1d09b29a68a457672fdbab4ec383 | dependency_revisions: auto-down=c3464d0a0d36e4f8a7988b0c0c26448fc52361b4, auto-ui=0261ae9dacf786beb6dceb630be6af267f242625 | acceptance_results: AC-01 pass; AC-02 pass; AC-03 pass; AC-04 pass; AC-05 pass; AC-06 needs_fix R865-F04 | pending_closeout_gates: AC-07 bare cargo t + tv; AC-08 canonical Specs/ledger/archive/guard | findings: R865-F04 (P2, AC-06/T-04/T-05, original requirement gap: committed Plan-modified Rust files are not rustfmt-clean) | evidence: verify-av-865.ps1 -Scope closures exit=0; surface=9/9; lowering=3/3; source/session=8/8; K suite=1/1; changed Rust formatting check=28 checked/7 failed; diff-check=pass; source-cache-metrics.json local-only SHA-256 1a52d883c8ae2f5c302eb3a86153cafc1f8bbede112d4d497ebd353d0122890d | next: work T-04/T-05

stage: goal | plan_id: PLAN-865 | plan_revision: r2 | round: 4 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-865/auto-lang | commit: eda2512ffc39ea474d3cffea24b4af9de3110b5d | next: work
2026-10-29 repair round 5 (R865-F04): commit 7e21ef7cafe73e243c6a0800636215cb1774afea (parent eda2512ffc39ea474d3cffea24b4af9de3110b5d). Applied rustfmt only to the seven flagged files; inspection confirmed the diff contains formatting changes only. On base..HEAD, rustfmt --check --edition 2021 --config skip_children=true checked all 28 Plan-modified Rust files and passed; git diff --check also passed. The worktree is clean. No semantic tests were rerun because this repair changed formatting only; round 4 targeted execution evidence remains applicable.

R865-F04 | outcome: closed | evidence: rustfmt --check --edition 2021 --config skip_children=true on all 28 Rust files changed from base to commit 7e21ef7cafe73e243c6a0800636215cb1774afea passes; git diff --check base..HEAD passes; only the seven offending Plan files were formatted, leaving package-wide pre-existing formatting differences in untouched files unchanged.

stage: work | plan_id: PLAN-865 | plan_revision: r2 | round: 5 | outcome: pass | worktree_path: D:/autostack/.wt/lang-865/auto-lang | branch: plan-865-dev | base_commit: f49c28054c1c1d09b29a68a457672fdbab4ec383 | code_commit: 7e21ef7cafe73e243c6a0800636215cb1774afea | repairs: R865-F04 | verification: changed-Rust-files-rustfmt=28/28 pass; diff-check=pass; worktree=clean | blockers: none | next_responsible_role: review

stage: goal | plan_id: PLAN-865 | plan_revision: r2 | round: 5 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-865/auto-lang | commit: 7e21ef7cafe73e243c6a0800636215cb1774afea | next: review
R865-F05 | type: original requirement gap | severity: P2 | acceptance/task: AC-01/T-04/V-01 (T-05 dependency reopened) | command: inspect closures_865_surface.rs, av_closures_865_source.rs, and K01-K06; rg -n 'fn apply|apply\(' across the Plan source tests | expected: a closure value is passed to a function-typed parameter, invoked through CALL_CLOSURE, and its result is asserted end-to-end | observed: the surface test compiles apply(f fn(int)->int, value int) but its main returns a constant without calling apply; source-session tests exercise returned closures, not closure arguments; K01-K06 contain no function-typed callback argument invocation | cause: required HOF-parameter runtime oracle is absent; product behavior is unverified, not established as failing | correction: add a source-to-session positive that passes a closure value to a named function with a function-typed parameter and asserts its runtime result; fix any failure, then replay the affected closure source/session and dependent K suite checks | controls: function-type declarations and indirect-call body compile; named-function closure returns K02/K05 and direct/zero/multi-parameter calls remain verified. | closure: closed in repair round 7; source/session callback invocation returns 42 with zero live values and balanced allocations.

2026-10-29 final re-review (repair head 7e21ef7cafe73e243c6a0800636215cb1774afea): same thread/context as implementation; independence not claimed. The round-5 change is limited to rustfmt output in seven files, with no semantic repair changes; rustfmt --check --edition 2021 --config skip_children=true over all 28 Rust paths in base..HEAD passes, and git diff --check passes. Retained evidence for unchanged implementation: AC-02 pass (source clone independence oracle, signature and async diagnostics); AC-03 pass (capture-set/signature invalidation and old-revision control); AC-04 pass (release/live balance and creator-exit source case); AC-05 pass (K01-K06 file suite, 8 checks/0 gaps, fresh/restart/body-edit metrics). AC-01 needs_fix R865-F05: §5.2 contract includes closures as higher-order parameters, but no source/session execution passes a closure to a function-typed parameter; the apply compile test only declares the body and does not call it. AC-06 pass for this review: current module Specs are unchanged; the current capability-matrix hash is recorded below and its C-11/C-18 rows are unchanged by concurrent edits; frozen delta SHA-256 060075a5ddc491809c0064c10cb6a1930a81393fc3992c3271981cedbfa1aa69. Changed-Rust formatting is 28/28; no debug/todo/unimplemented marker was added. The worker release build recorded earlier had only the pre-existing dead-code warning in unchanged owned_session.rs; it remains explicitly noted in the Plan and is outside this repair. V-06 evidence remains the local source-cache-metrics.json SHA-256 1a52d883c8ae2f5c302eb3a86153cafc1f8bbede112d4d497ebd353d0122890d. Current inputs: T-02 contract 3b13327d8c6ebc68f42b7b643e169f32fdcaf6f59268dfdb303aa0fa0ae2d00; closure-environments c93267790454ec719f4ddd7595386b1e1cca036688696b45caaa620c63e5340f; closure-capture-profile 9b8ac7d6a70da6c5ce27c98a829b3ae20a74345abba1fa3fd2e2f307df4d8dea9; ACE M1 6fc7d890335ccb0b4a58533edbbb8e5711763766ef2c0c77f8c783bd7a4f0b32; capability matrix current main content ae4bbcb1e573f725ce4797e28efbca46a15dfd4bb368dbe06faeaf0480cf508; SD-01 target absent. Main matrix has unrelated user WIP in C-06/C-15/C-16/C-17/C-24/C-29; C-11/C-18 are untouched, so defer a narrow hunk reconciliation to merge and preserve the WIP. AC-07 bare cargo t + tv and AC-08 canonical Specs/ledger/archive/guard remain merge-owned pending. Finding R865-F05 is an original requirement gap: missing end-to-end evidence for an explicit contract feature. No unsupported deferral is accepted.

stage: review | review_scope: final | reviewer_identity/limitations: same thread/context as implementation; artifact-based re-review, independence not claimed | plan_id: PLAN-865 | plan_revision: r2 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-865/auto-lang | branch: plan-865-dev | reviewed_commit: 7e21ef7cafe73e243c6a0800636215cb1774afea | base_commit: f49c28054c1c1d09b29a68a457672fdbab4ec383 | dependency_revisions: auto-down=c3464d0a0d36e4f8a7988b0c0c26448fc52361b4, auto-ui=0261ae9dacf786beb6dceb630be6af267f242625 | spec_inputs: T-02-contract=3b13327d8c6ebc68f42b7b643e169f32fdcaf6f59268dfdb303aa0fa0ae2d00; SD-01=absent; SD-02=c93267790454ec719f4ddd7595386b1e1cca036688696b45caaa620c63e5340f; SD-03=9b8ac7d6a70da6c5ce27c98a829b3ae20a74345abba1fa3fd2e2f307df4d8dea9; SD-04=6fc7d890335ccb0b4a58533edbbb8e5711763766ef2c0c77f8c783bd7a4f0b32; SD-05=ae4bbcb1e573f725ce4797e28efbca46a15dfd4bb368dbe06faeaf0480cf508; frozen_SD_delta_SHA256=060075a5ddc491809c0064c10cb6a1930a81393fc3992c3271981cedbfa1aa69 | acceptance_results: AC-01 needs_fix R865-F05; AC-02 pass; AC-03 pass; AC-04 pass; AC-05 pass; AC-06 pass with pre-existing warning noted | pending_closeout_gates: AC-07 bare cargo t + tv; AC-08 canonical Specs/ledger/archive/guard | findings: R865-F05 (P2, AC-01/T-04/V-01, original requirement gap: no end-to-end invocation of a closure passed to a function-typed parameter) | evidence: rustfmt changed-Rust=28/28 pass; diff-check=pass; closures_865_surface.rs apply declaration compiles but main returns constant; av_closures_865_source.rs tests closure returns and direct/zero/multi calls but no closure argument; K01-K06 have no callback argument case; source contract §5.2 requires higher-order parameters | next: work T-04/T-05

stage: goal | plan_id: PLAN-865 | plan_revision: r2 | round: 6 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-865/auto-lang | commit: 7e21ef7cafe73e243c6a0800636215cb1774afea | next: work
R865-F05 | outcome: closed | evidence: av_closures_865_passes_closure_to_function_typed_parameter passes through the source compiler, unit lowerer, ClosuresUpdateSession, and CALL_CLOSURE parameter call; result=42; run_source asserts live_after_release=0, allocs=frees, gate pass, and clean session close.

2026-10-29 repair round 7 (R865-F05): commit 7fea66b0e151e78c13a9f19f91253b49ed89476a (parent 7e21ef7cafe73e243c6a0800636215cb1774afea). Added a source/session test that returns an owned closure from make, passes it as a function-typed argument to apply, invokes it inside apply, and observes 42. This proves the previously missing higher-order parameter path; no implementation code change was required.

Verification: exact callback test exit=0, 1/1; av_closures_865_source exit=0, 9/9; ace_m1_closures_865_source_suite_green with test-cases feature exit=0, 1/1 (8 K checks, 0 coverage gaps); rustfmt --check --edition 2021 --config skip_children=true on 28 Plan-modified Rust paths passes; git diff --check base..HEAD passes; worktree clean. The targeted builds emitted existing warnings in unchanged auto-av-probe owned_session.rs and unrelated auto-lang test files; no warning came from the added test.

stage: work | plan_id: PLAN-865 | plan_revision: r2 | round: 7 | outcome: pass | worktree_path: D:/autostack/.wt/lang-865/auto-lang | branch: plan-865-dev | base_commit: f49c28054c1c1d09b29a68a457672fdbab4ec383 | code_commit: 7fea66b0e151e78c13a9f19f91253b49ed89476a | repairs: R865-F05 | verification: callback=1/1; source-session=9/9; K-file-suite=1/1 (8 checks, 0 gaps); changed-Rust-rustfmt=28/28 pass; diff-check=pass; worktree=clean | blockers: none | next_responsible_role: review

stage: goal | plan_id: PLAN-865 | plan_revision: r2 | round: 7 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-865/auto-lang | commit: 7fea66b0e151e78c13a9f19f91253b49ed89476a | next: review
2026-10-29 final review round 8: same thread/context as implementation; artifact-based re-review, independent review not claimed. Baseline reconciliation: T-01 records 5af8e0cbe3f5a5a4971d5142abb4fb53b0f983b4 and git merge-base(master, plan-865-dev) confirms 5af8e0c; earlier receipts used f49c28054c1c1d09b29a68a457672fdbab4ec383 as an incremental handoff boundary. This verdict covers the full 5af8e0c..7fea66b0e151e78c13a9f19f91253b49ed89476a Plan diff (39 files, 32 Rust). All 32 Rust paths pass rustfmt --check --edition 2021 --config skip_children=true; git diff --check passes. The full source/session suite rerun on the reviewed head passes 9/9, including the R865-F05 callback path and prior accepted controls. The round-7 K01-K06 file-backed TestCase run passes 1/1 with 8 checks and 0 coverage gaps. The current local-only V-06 evidence refreshed by that gate is source-cache-metrics.json, 20 samples, P90=355.1596 ms, max process peak=1,015,808 bytes, max hot set=4,483 bytes; SHA-256 1c55fb2f3af4d4bb3609b9f84f5c84753875b173fc30612571d119411032cbc8. It meets P90 <= 2000 ms and hot set <= 8 MiB; do not commit the JSON. R865-F01..F05 are closed; no open work-owned findings remain. Added eprintln! lines are error-path diagnostics, not debug tracing; no dbg!/todo!/unimplemented! marker is added. Existing compiler warnings in unchanged owned_session.rs and unrelated auto-lang test sources are noted as baseline; no warning was introduced by the callback test.

Spec delta review: SD-05 before/after wording was clarified in the Plan without changing scope or acceptance. Frozen delta SHA-256 is 7617baabf74ea1b7881442f75a2a81fdf8c64a2419a0fd381c9b48ed7203d093. Current inputs: T-02 contract 3b13327d8c6ebc68f42b7b643e169f32fdcaf6f59268dfdb303aa0fa0ae2d00; closure-environments c93267790454ec719f4ddd7595386b1e1cca036688696b45caaa620c63e5340f; closure-capture-profile 9b8ac7d6a70da6c5ce27c98a829b3ae20a74345abba1fa3fd2e2f307df4d8dea9; ACE M1 6fc7d890335ccb0b4a58533edbbb8e5711763766ef2c0c77f8c783bd7a4f0b32; capability matrix current main content ae4bbcb1e573f725ce4797e28efbca46a15dfd4bb368dbe06faeaf0480cf508; SD-01 target remains absent for merge to add. GOAL-002 exists. The main matrix has unrelated user WIP in C-06/C-15/C-16/C-17/C-24/C-29, while C-11/C-18 remain unchanged; merge must apply only those two planned hunks and preserve the WIP. From the true base, Plan code paths and main code paths do not overlap. auto-down and auto-ui consumer worktrees remain pinned and clean at c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 and 0261ae9dacf786beb6dceb630be6af267f242625. AC-07 bare cargo t + tv and AC-08 canonical Specs/ledger/archive/guard remain merge-owned pending.

stage: review | review_scope: final | reviewer_identity/limitations: same thread/context as implementation; artifact-based re-review; independence not claimed | plan_id: PLAN-865 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-865/auto-lang | branch: plan-865-dev | reviewed_commit: 7fea66b0e151e78c13a9f19f91253b49ed89476a | base_commit: 5af8e0cbe3f5a5a4971d5142abb4fb53b0f983b4 | dependency_revisions: auto-down=c3464d0a0d36e4f8a7988b0c0c26448fc52361b4, auto-ui=0261ae9dacf786beb6dceb630be6af267f242625 | spec_inputs: T-02-contract=3b13327d8c6ebc68f42b7b643e169f32fdcaf6f59268dfdb303aa0fa0ae2d00; SD-01=absent; SD-02=c93267790454ec719f4ddd7595386b1e1cca036688696b45caaa620c63e5340f; SD-03=9b8ac7d6a70da6c5ce27c98a829b3ae20a74345abba1fa3fd2e2f307df4d8dea9; SD-04=6fc7d890335ccb0b4a58533edbbb8e5711763766ef2c0c77f8c783bd7a4f0b32; SD-05=ae4bbcb1e573f725ce4797e28efbca46a15dfd4bb368dbe06faeaf0480cf508; frozen_SD_delta_SHA256=7617baabf74ea1b7881442f75a2a81fdf8c64a2419a0fd381c9b48ed7203d093 | acceptance_results: AC-01 pass; AC-02 pass; AC-03 pass; AC-04 pass; AC-05 pass; AC-06 pass | pending_closeout_gates: AC-07 bare cargo t + tv; AC-08 canonical Specs/ledger/archive/guard | findings: none open; R865-F01,F02,F03,F04,F05 closed | evidence: source/session=9/9 (including callback result 42 and release oracle); K suite=1/1 (8 checks, 0 gaps); V-06 P90=355.1596ms/20 samples, max peak=1,015,808 bytes, hot set=4,483 bytes, local hash 1c55fb2f3af4d4bb3609b9f84f5c84753875b173fc30612571d119411032cbc8; changed-Rust=32/32 rustfmt pass; full-base diff-check pass | next: merge

stage: goal | plan_id: PLAN-865 | plan_revision: r2 | round: 8 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-865/auto-lang | commit: 7fea66b0e151e78c13a9f19f91253b49ed89476a | next: merge
2026-10-29 PLAN-865:r2 merge preflight and gate result (round 9). Prepared baseline: final review pass is for revision r2, commit 7fea66b0e151e78c13a9f19f91253b49ed89476a, true base 5af8e0cbe3f5a5a4971d5142abb4fb53b0f983b4, dependencies auto-down=c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 and auto-ui=0261ae9dacf786beb6dceb630be6af267f242625. Default branch master was 2fdee4b0c817aa3e2effb331f0f8b57b3ef56d95. Rebased clean plan-865-dev onto master to 835fb9abf714a52adbe657a4f1e83bd70bce648e. `git range-diff` reports all 9 commits patch-equivalent; old→new: 0275805f0eb3→2dd302a9b724; cae23a9765b2→a869c8585f89; fc30d2d4ec5a→20f43bc62e33; 0b3bce3fc6cc→93df6bc3dfa8; f49c28054c1c→2662d17eb740; 7d708e36f51e→4bccce4b5627; eda2512ffc39→477e2301ae77; 7e21ef7cafe7→74ba84c34883; 7fea66b0e151→835fb9abf714. The current branch has 39 changed paths from the true base; master has 50, with zero overlapping `crates/` or `experimental/` paths. No source or Spec conflict was introduced by rebase.

Frozen Spec delta is SD-01..SD-05, SHA-256 7617baabf74ea1b7881442f75a2a81fdf8c64a2419a0fd381c9b48ed7203d093. Current Spec inputs still match the final review: closure-environments c93267790454ec719f4ddd7595386b1e1cca036688696b45caaa620c63e5340f; closure-capture-profile 9b8ac7d6a70da6c5ce27c98a829b3ae20a74345abba1fa3fd2e2f307df4d8dea9; ACE M1 6fc7d890335ccb0b4a58533edbbb8e5711763766ef2c0c77f8c783bd7a4f0b32; capability matrix working-copy WIP ae4bbcb1e573f725ce4797e28efbca46a15dfd4bb368dbe06faeaf0480cf508; SD-01 source-closures target absent. `.autoos/specs.json` is present at main with SHA-256 c520695d9e090458f8d80b2407b5a431dda95ef446a5fbd4cb79adbfbf009d80; no ledger operation was attempted. Main checkout WIP remains untouched. No canonical Spec, matrix, ledger, or delivery commit has been written.

AC-07 on the rebased candidate, using native Windows PowerShell: `cargo t` exit 1; summary 98 passed, 1 failed, 19 timed out, 673 skipped (3,089/3,207 not run after failure). Failing test: `tests::ac_cte_probe_tests::cte_probe_corpus_executes_green`; nested compiler output reports non-exhaustive matches in `experimental/ac-semantics/core/src/adapter.rs` for the new `TypeRef::Function`, `ExprKind::CallValue`, and `ExprKind::Closure` variants. This is a real compatibility compile error requiring a work-stage correction. A separate PLAN-002 Cargo test/build process was active during this gate; timeout attribution is uncertain and the full gate must be rerun after correction. `cargo tv` exit 0: 163 passed, 2,061 skipped. Thus AC-07 is not passed; do not land.

R865-M01 | type: merge gate finding | severity: P2 | acceptance: AC-07 / source AST consumer compatibility | evidence: Windows `cargo t` failure on `835fb9abf714a52adbe657a4f1e83bd70bce648e`; commit `7491be94bd20746f711ba7267e995642ec66e12e` maps TypeRef::Function, ExprKind::CallValue, ExprKind::Closure to explicit unsupported nodes and preserves VT::Function in legacy verdicts; final re-review on the repaired commit passes | required action: none; frozen merge gates remain pending | status: closed.

Consolidation checkpoints: prepared=baseline/delta/source hashes recorded above; landed=pending; ledger_refreshed=pending; archived=pending; cleaned=pending. The goal has already used its three work↔review repair cycles (R865-F01..F03, R865-F04, R865-F05); this fourth repair/re-review cycle exceeds the /auto-plan-goal cap. Plan status returns to `executing`, `current_step: 3`; T-07 remains unchecked. No landing, ledger write, archive, or worktree removal occurred. Resume only after the user explicitly authorizes an additional bounded repair/re-review cycle.

stage: merge | plan_id: PLAN-865 | plan_revision: r2 | outcome: needs_fix | reviewed_commit: 7fea66b0e151e78c13a9f19f91253b49ed89476a | rebased_candidate: 835fb9abf714a52adbe657a4f1e83bd70bce648e | range_diff: 9/9 patch-equivalent | AC-07: cargo t failed (98 pass/1 fail/19 timeout, concurrent Plan-002 build observed); cargo tv passed (163/163) | prepared: verified | landed: no | ledger_refreshed: no | archived: no | cleaned: no | next: user

stage: goal | plan_id: PLAN-865 | plan_revision: r2 | round: 9 | from: merge | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-865/auto-lang | commit: 835fb9abf714a52adbe657a4f1e83bd70bce648e | next: user


2026-10-29 repair round 10 (R865-M01): The user explicitly authorized continuation beyond the prior repair-cycle cap, within the existing Plan scope and acceptance criteria. On rebased candidate `835fb9abf714a52adbe657a4f1e83bd70bce648e`, commit `7491be94bd20746f711ba7267e995642ec66e12e` explicitly projects function types, indirect calls, and closure expressions to unsupported nodes in the limited `ac-semantics` adapter, while retaining `VT::Function` in the legacy verdict mapper. This keeps that adapter fail-closed and restores exhaustive matching after the source AST expansion.

Verification from native Windows PowerShell: `cargo test --manifest-path experimental/ac-semantics/core/Cargo.toml --features core-ast --lib closure_ast_forms_remain_explicitly_unsupported_in_limited_sema_adapter` exit 0 (1 passed); `cargo test --manifest-path crates/auto-lang/Cargo.toml --lib --features test-cases cte_probe_corpus_executes_green` exit 0 (1 passed); `rustfmt --edition 2021 --check --config skip_children=true experimental/ac-semantics/core/src/adapter.rs` pass; `git diff --check` pass. Only `experimental/ac-semantics/core/src/adapter.rs` changed in the repair commit. Full `cargo t`/`cargo tv` remain frozen merge gates, pending fresh review.

R865-M01 | outcome: fixed-awaiting-review | invariant: every new source-AST type/expression variant consumed by the limited adapter must be classified explicitly; function forms remain unsupported there, and legacy verdicts preserve the function type name | permanent oracle: `closure_ast_forms_remain_explicitly_unsupported_in_limited_sema_adapter` checks all three projections, unsupported diagnostics, scalar accepted control, and legacy `VT::Function`; `cte_probe_corpus_executes_green` covers the compiler integration path | result: focused unit 1/1 and integration 1/1 pass; final review pending on `7491be94bd20746f711ba7267e995642ec66e12e`.

handoff: `stage: work | plan_id: PLAN-865 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-865/auto-lang | branch: plan-865-dev | base_commit: 835fb9abf714a52adbe657a4f1e83bd70bce648e | code_commit: 7491be94bd20746f711ba7267e995642ec66e12e | repairs: R865-M01 | verification: adapter=1/1; CTE-integration=1/1; rustfmt=pass; diff-check=pass; worktree=clean | blockers: none | next_responsible_role: review`。

stage: goal | plan_id: PLAN-865 | plan_revision: r2 | round: 10 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-865/auto-lang | commit: 7491be94bd20746f711ba7267e995642ec66e12e | next: review


2026-10-29 final re-review round 11: artifact-based review of r2 at `7491be94bd20746f711ba7267e995642ec66e12e` (`plan-865-dev`), diff base current `master` `2fdee4b0c817aa3e2effb331f0f8b57b3ef56d95`. Same primary thread as implementation; an independent read-only review context also returned pass. The prior reviewed commit `7fea66b0e151e78c13a9f19f91253b49ed89476a` was rebased to `835fb9abf714a52adbe657a4f1e83bd70bce648e`; `git range-diff` confirms 9/9 commits patch-equivalent. Repair commit `7491be94` has only 98 additions in `experimental/ac-semantics/core/src/adapter.rs`; current code worktree is clean at the recorded Windows path. auto-down and auto-ui dependency worktrees are clean at `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` and `0261ae9dacf786beb6dceb630be6af267f242625`.

R865-M01 closure: the limited adapter explicitly represents function-type annotations, indirect calls, and closure expressions as unsupported; the legacy verdict retains function parameters as `function`. The permanent unit oracle verifies all three projections, fail-closed diagnostics, a supported scalar control, and the old-pipeline function-type path. Replayed `cargo test --manifest-path experimental/ac-semantics/core/Cargo.toml --features core-ast --lib closure_ast_forms_remain_explicitly_unsupported_in_limited_sema_adapter` exit 0 (1/1); replayed `cargo test --manifest-path crates/auto-lang/Cargo.toml --lib --features test-cases cte_probe_corpus_executes_green` exit 0 (1/1), covering the actual compiler/CTE integration path. `rustfmt --edition 2021 --check --config skip_children=true` passes on all 33 Rust paths in `master..HEAD`; `git diff --check master..HEAD` passes. No full-tier `cargo t`/`cargo tv` run was made in review; those remain merge-owned.

Acceptance map: AC-01 pass; AC-02 pass; AC-03 pass; AC-04 pass; AC-05 pass; AC-06 pass. Prior review evidence for AC-01..05 remains applicable because the nine rebased commits are patch-equivalent and the repair touches only the limited sema adapter; source/session callback result remains 42, K01-K06 remain 8 checks/0 coverage gaps, and V-06 remains within its recorded P90/hot-set budgets. AC-06: approved T-02 contract SHA-256 `3b13327d8c6ebc68f42b7b643e169f32fdcaf6f59268dfdb303aa03fa0ae2d00`; current SD-02/03/04 inputs match `c93267790454ec719f4ddd7595386b1e1cca036688696b45caaa620c63e5340f`, `9b8ac7d6a70da6c5ce27c98a829b3ae20a74345abba1fa3fd2e2f307df4d8dea9`, `6fc7d890335ccb0b4a58533edbbb8e5711763766ef2f0c77f8c783bd7a4f0b32`; capability matrix input remains `ae4bbc1e573f725ce4797e28efbca46a15dfd4bb368dbe06faeaf0480cf508`; SD-01 target is absent as planned; frozen SD-01..05 delta hash remains `7617baabf74ea1b7881442f75a2a81fdf8c64a2419a0fd381c9b48ed7203d093`; GOAL-002 exists. No canonical Spec or ledger edit is in the implementation diff. Accepted scope edges C-19/R5-O and C-11/C-18 HOF/default/variadic/function-item gaps are recorded as nonblocking candidates `865-N1/N2` in `KNOWN-DEBT-AND-RISKS.md`.

Worktree note for merge: the physical checkout and per-worktree metadata are verified through explicit Windows `GIT_DIR`/`GIT_WORK_TREE`; `.git` and reverse `gitdir` records contain `/mnt/d/...`, so native `git worktree list` labels it prunable. This is a metadata/cleanup issue, not a code finding; AC-08 must normalize/handle it before guard, removal, and branch cleanup. Main checkout code paths are clean; unrelated documentation WIP remains preserved.

2026-10-29 metadata correction: the preceding note records the earlier Linux-style pointers, not the current state. Native Windows Git repaired the Plan worktree registration; `.git` now points to `D:/autostack/auto-lang/.git/worktrees/auto-lang3`, the reverse `gitdir` points to `D:/autostack/.wt/lang-865/auto-lang/.git`, and `git worktree list --porcelain` recognizes the checkout. The repair overwrote the old pointer, so its original writer cannot be identified from the remaining files. Reading metadata under WSL is read-only and does not create or rewrite these paths; a `/mnt/d/...` pointer is consistent with registration while Git saw the worktree through that Linux path namespace, but the exact operation is unknown.

stage: review | review_scope: final | reviewer_identity/limitations: artifact-based review in implementation thread plus independent read-only review context; no model/session identity asserted | plan_id: PLAN-865 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-865/auto-lang | branch: plan-865-dev | reviewed_commit: 7491be94bd20746f711ba7267e995642ec66e12e | base_commit: 2fdee4b0c817aa3e2effb331f0f8b57b3ef56d95 | dependency_revisions: auto-down=c3464d0a0d36e4f8a7988b0c0c26448fc52361b4, auto-ui=0261ae9dacf786beb6dceb630be6af267f242625 | spec_inputs: T-02-contract=3b13327d8c6ebc68f42b7b643e169f32fdcaf6f59268dfdb303aa03fa0ae2d00; SD-01=absent; SD-02=c93267790454ec719f4ddd7595386b1e1cca036688696b45caaa620c63e5340f; SD-03=9b8ac7d6a70da6c5ce27c98a829b3ae20a74345abba1fa3fd2e2f307df4d8dea9; SD-04=6fc7d890335ccb0b4a58533edbbb8e5711763766ef2f0c77f8c783bd7a4f0b32; SD-05=ae4bbc1e573f725ce4797e28efbca46a15dfd4bb368dbe06faeaf0480cf508; frozen_SD_delta_SHA256=7617baabf74ea1b7881442f75a2a81fdf8c64a2419a0fd381c9b48ed7203d093 | acceptance_results: AC-01 pass; AC-02 pass; AC-03 pass; AC-04 pass; AC-05 pass; AC-06 pass | pending_closeout_gates: AC-07 bare cargo t + tv; AC-08 canonical Specs/ledger/archive/guard/worktree cleanup | findings: none open; R865-M01 closed | evidence: range-diff=9/9 patch-equivalent; adapter=1/1; CTE-integration=1/1; changed-Rust=33/33 rustfmt pass; full-base diff-check pass; independent read-only review pass | next: merge

stage: goal | plan_id: PLAN-865 | plan_revision: r2 | round: 11 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-865/auto-lang | commit: 7491be94bd20746f711ba7267e995642ec66e12e | next: merge

## 10. 待澄清事项

- `map/filter/reduce` 等 HOF 是否属于本切片：默认**不属于**。它们需要 List 方法派发/库装配的语义裁定（822 方法面未含），硬塞会把“库能力”伪装成“闭包语法”。建议后续以 R3-H/stdlib 切片认领，并在此之前不做半成品。
- 共享可变捕获（C-19）若旧消费者实际依赖，不能永久排除：本Plan只登记为 unsupported + owner，须在 R5-O 设计后决定。
- 闭包能否作为 `record` 字段/List 元素：默认不含（组合面另立），如首批需要须扩 T-02 与资源预算。

2026-10-28 D868 前置解除：PLAN-868 已交付落 master（delivery 7754c4e4c）——HIR→MIR lowering 唯一实现为 `experimental/auto-mir-lower/src/**`（纯搬移等价已证，旧 `probe/src/hir_adapter.rs` 已删除），本计划 D868 硬前置视为满足；lowering 相关 T 任务的写面以该 crate 为准（边界合同见 [docs/specs/auto-mir/hir-lowering-boundary.md](../../specs/auto-mir/hir-lowering-boundary.md)）。


Archive checkpoint (2026-10-29): fast-forwarded master from 674871e2a3bcc1a9cf594ac7446f3c1d1129ecbb to delivery baseline 5fa3133fa6401904b0760986ecb2e83b0534cbd4; ledger v139→v140 read-back passed at SHA-256 718255cc08a50ec2af79597be78cdefb78adab19925a9af67016cdc2af73f722; status=archived, completion_kind=delivered.

Cleanup checkpoint (2026-10-29): `D:/autostack/.wt/lang-865/auto-lang` passed the Windows `wt-guard.sh` check (no reparse points), was removed without force, and `plan-865-dev` was deleted at `5fa3133fa6401904b0760986ecb2e83b0534cbd4`. The Plan worktree's 16.3 GB Cargo cache (verified `CACHEDIR.TAG`) was reclaimed with the checkout; `D:/autostack/plan-artifacts/plan-865` did not exist. The `D:/autostack/.wt/lang-865` group remains because it contains sibling `auto-down` and `auto-ui` trees; they are outside this Plan's ownership and were preserved.

Landing artifact observation (2026-10-29): auto-lang is a library/test workspace, with no backend daemon binary or `gen/front/vue/dist` bundle; the Vue bundle path is also absent in auto-man and auto-ui. PLAN-865 changed no dependency repository. No release `auto.exe` exists in auto-man; its debug `target/debug/auto.exe` is dated 2026-10-29 21:38:01 +08:00, before the 22:14:29 +08:00 code delivery, and belongs to a separate live VM probe. It was left running and was not treated as a production release or restarted.


Evidence retention note (2026-10-29): the 20-sample local source-cache metrics file was last written at 22:38:47 during the full gate window. The preserved current copy at [source-cache-metrics.json](../attachments/865/work/source-cache-metrics.json) has SHA-256 cc7cbc1885c1bb5f3196a46ea8817d9f64323dbd4854acf29a107e8ed3bb452e and reports P90=735.0567 ms, max process peak=1,716,224 bytes, and hot set=4,483 bytes. Earlier work and review receipts record historical snapshots with hashes 0029b818b947eb0eaa17695916c12e44d74984a67ff72bc25a26d414b5ec2591, 1a52d883c8ae2f5c302eb3a86153cafc1f8bbede112d4d497ebd353d0122890d, and 1c55fb2f3af4d4bb3609b9f84f5c84753875b173fc30612571d119411032cbc8. The workspace no longer contains the raw blob matching the first hash; the summary values remain in their original receipts, while this current copy is preserved separately.


批量回归收尾（2026-10-29，测试 HEAD 16555e450783bde1f9ad81ce494dc0e3c0e92988）：完成的分段为 language 3203/3203、ui-full 3120/3120、tt 3578/3578。首轮 mir_native_rounds_failure_liveness_and_caps 在并发 DB/Vue 进程期间触发已登记的 PLAN-868 P3 3 秒墙负载敏感红；隔离重放 1/1 通过，随后安静窗口的整段 tt 也全绿。tb 选中 3093 项，其中 3092 通过；book_ch09_06 与相邻书仓 golden 更新重叠（用例报错 23:46:23，golden 写入时间 23:46:26），更新后的定向重放 1/1 通过，书仓 WIP 保留原样。用户明确要求不再重跑整批，并授权剩余结果按通过收口；因此记录为用户接受通过。批次驱动因 tb 的首次退出码 100 未继续执行 man-t、man-tcore、man-tc、man-ti、ui-tu、ui-ti、ui-tn，这些腿在回归收据中标为用户接受，未声称实际执行。随后用户中断的一次重启尝试已停止；受保护的 10 份报告均按 SHA-256 恢复。测试时 HEAD 已包含 AutoGraphics 规格/账本提交，这些提交未改测试代码。详见 docs/plans/.last-batch-regression.json 与外部日志目录 D:/autostack/full-regression-receipts/1793288263077079500。