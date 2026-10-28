---
plan_id: PLAN-865
status: drafting
feature_name: AV闭包源码语法与间接调用（R2-L 闭包）
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-ac/source-closures.md
touched_goals: [GOAL-002]

affects: [auto-av, auto-mir, auto-ac, auto-test-case]
current_step: 0
total_steps: 9
---

# [PLAN-865] AV闭包源码语法与间接调用（R2-L 闭包）

## 0. 变更摘要

PLAN-823 已交付执行侧 `av.closures/1`（profile id 6、AVAD v5、TY_CLOSURE tag 17）：typed owned 闭包环境、move/显式 clone 逃逸捕获、静态可证非逃逸同步借用、受检间接调用与 env/frame 身份，并明确排除**源码闭包语法/前端推导**（[auto-av/project](../specs/auto-av/project.md)、[closure-environments](../specs/auto-av/closure-environments.md)）。

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
- lowering/执行：`experimental/ac-av-probe/core/src/{mir,closures,abc,abc_verify,av}.rs`、`probe/src/{hir_adapter,closures_session,session}.rs`。
- 语料：`crates/auto-lang/test/cases/ace-m1/src/K0*.at`、`binding.tsv`、`crates/auto-lang/src/tests/ace_m1_tests.rs`。
- 拟建 `scripts/verify-av-865.ps1`（Scope=targeted/compat/closures/acceptance），前缀 `av_closures_865`；当前不存在。

唯一实施树 `D:/autostack/.wt/lang-865/auto-lang`，分支 `plan-865-dev`。

## 4. 需求分析与背景调查

用户 2026-10-28 授权同上（只规划）。事实基础：

- [closure-environments](../specs/auto-av/closure-environments.md) 与 [closure-capture-profile](../specs/auto-mir/closure-capture-profile.md)：owned env、move/显式 clone、静态非逃逸 borrow、间接调用、env/frame 身份与 A23 代码层恢复已冻结；源码语法/推导/共享 RC/async 为显式非目标。
- [resident-function-install §823](../specs/auto-av/resident-function-install.md)：闭包 body 自带 `env` 声明，`CREATE_CLOSURE.callee` 走 `CalleeRef`；link 期 `fn:` 绑定做 env 等价检查，声明漂移精确拒绝。
- 852 binding：K01（move 捕获）typed 行 `ok:av-closures-823`；K02/K05 两条 legacy 路径 `diff-legacy-grammar(parse)`，注记“owner 823 freezes source syntax”；K03/K04/K06 已有 typed 证据与借用/逃逸负控。
- 语料 `src/K01.at`：`let f = move (n int) => { return base + n }`；`src/K02.at` 与 `src/K05.at` 使用同一 move-closure 形态。
- SEM-04：823 首批为 move/显式 clone 的 owned 闭包；旧调用面含隐式捕获/可变共享与 alias → 必须按用例裁定，不能用深 clone 统一。

## 5. 详细设计

### 5.1 前置与写面

硬前置：`D862`（入口）、`D864`（集合源码，供捕获 owned 值与 HOF 后续切片）、`D823`（已具备）。写面见 §3。

### 5.2 源码语义冻结（T-02）

`attachments/865/work/closures-source-contract.md` 冻结：

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
| SD-05 | modify | docs/specs/av-language-capability-matrix.md（design/strategy） | C-18/C-11 源码列更新 | 路线事实 | AC-05 |

## 6. 测试设计

| V | AC/不变量 | 实际/拟建入口 | 可区分错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 语法/调用 | K01–K06 原源；零参/多参/直接与间接调用正例 | 手算 exit 值；env 字段序漂移/元数错必红 | T-03 |
| V-02 | AC-02 捕获/逃逸 | move 后读取（K/源级）、借用逃逸（K04）、共享可变/async 拒绝、clone 独立性 | 编译期具名拒绝（非运行期）；clone 独立 oracle 精确 | T-03/04 |
| V-03 | AC-03 环境一致性 | env 声明漂移、捕获集合改变、签名改变 | 漂移精确失效；旧调用保活；不得整备 | T-04 |
| V-04 | AC-04 资源/寿命 | 创建者退出后调用、旧帧/旧代回收、失败候选 | 无泄漏/无悬挂；Weak oracle 释放 | T-04 |
| V-05 | AC-05 语料绑定 | K 行 + 独立 oracle + 分母审计 | SKIP 不计绿；legacy `diff-*` 如实保留 | T-05 |

定向命令：两独立 workspace 定向 `cargo test`；`scripts/verify-av-865.ps1 -Scope <targeted|compat|closures|acceptance>`；完整档归 merge。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 闭包源码语法/调用按冻结件成立，K 源全链执行；V-01 |
| AC-02 | work | 捕获/逃逸/clone 语义与拒绝面成立（编译期拒绝），SEM-04 落 Spec；V-02 |
| AC-03 | work | env 声明与 link 等价检查保持，env 漂移精确失效；V-03 |
| AC-04 | work | 创建者退出/旧代回收/失败候选资源正确；V-04 |
| AC-05 | work | 852 K 行与独立 oracle 落地，`823-future-source` 关闭；V-05 |
| AC-06 | review | 独立核 AC/V/SD、捕获分析是否在正确阶段、profile 6 零漂移、warnings/fmt |
| AC-07 | merge | 冻结提交裸 `cargo t` + `tv` 通过 |
| AC-08 | merge | Spec/矩阵/ledger/归档/guard 清理完成 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | D862、D864、D823 | 核实 823 env 合同/会话接口、ac-core 解析阶段、K 源实际语法；产出 baseline | AC-01/02 | [ ] |
| T-02 | work | T-01 | 冻结 §5.2 合同（含 SEM-04 与拒绝面） | AC-01..05 | [ ] |
| T-03 | work | T-02 | 前端：lambda/捕获/逃逸诊断/descriptor；V-01/02 | AC-01/02 | [ ] |
| T-04 | work | T-03 | lowering + 源码会话（env 漂移/寿命）；V-03/04 | AC-03/04 | [ ] |
| T-05 | work | T-04 | 语料绑定/oracle/脚本/矩阵；V-05，提交代码与证据 | AC-05 | [ ] |
| T-06 | review | T-05 | 独立最终review；merge 门 pending | AC-06 | [ ] |
| T-07 | merge | T-06 pass | 冻结提交裸 `cargo t` + `tv` | AC-07 | [ ] |
| T-08 | merge | T-07 | Spec/矩阵/ledger/归档/guard | AC-08 | [ ] |
| T-09 | merge | T-08 | 交接 866 与后续 HOF 切片的实际基面与限制 | AC-08 | [ ] |

## 9. 复审记录

2026-10-28 new r1：`stage: new | plan_id: PLAN-865 | plan_revision: 1 | outcome: pass（等 D862/D864，合同可预检） | next: D864 后 work T-01`。全部 AC/V/T 未执行；语法/捕获规则/诊断码待 T-02 冻结；未建树、未跑构建、未改在途Plan。批次排序依据见[后继批次规划](reports/ace-next-batch-2026-10-28.md)。

## 10. 待澄清事项

- `map/filter/reduce` 等 HOF 是否属于本切片：默认**不属于**。它们需要 List 方法派发/库装配的语义裁定（822 方法面未含），硬塞会把“库能力”伪装成“闭包语法”。建议后续以 R3-H/stdlib 切片认领，并在此之前不做半成品。
- 共享可变捕获（C-19）若旧消费者实际依赖，不能永久排除：本Plan只登记为 unsupported + owner，须在 R5-O 设计后决定。
- 闭包能否作为 `record` 字段/List 元素：默认不含（组合面另立），如首批需要须扩 T-02 与资源预算。
