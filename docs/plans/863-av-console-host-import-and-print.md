---
plan_id: PLAN-863
status: reviewed
feature_name: AV类型化宿主导入与控制台输出（R2-E/print）
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-30
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-test-case/architecture.md
  - docs/specs/auto-test-case/ace-m1-conformance.md
  - docs/specs/auto-av/project.md
  - docs/specs/auto-mir/hir-lowering-boundary.md
new_spec_components:
  - docs/specs/auto-av/host-import-and-console.md
  - docs/specs/auto-ac/source-console-output.md
touched_goals: [GOAL-002]

affects: [auto-av, auto-mir, auto-ac, auto-test-case]
current_step: 6
total_steps: 8
---

# [PLAN-863] AV类型化宿主导入与控制台输出

## 0. 变更摘要

本Plan领取[AV源码与替换路线](../design/strategy/av-source-programs-and-avm-replacement-roadmap.md) **R2-E** 的“typed console/普通CLI结果”部分，并交付能力矩阵 **C-29** 的首个真实能力面：让真实Auto源码可以 `print`/`eprint` 并把输出作为**可独立判决的观察轴**，而不靠退出码猜测程序行为。

同时本Plan建立ACE的第一类**类型化宿主导入（host import）**合同：ABC 模块级声明 + 能力授权 + 受检派发 + 有界预算，供后续 FS/argv/env/stdlib（R3-H）与 FFI（R6-F）复用同一形状。当前 ABC 的 import 只承载闭包单元链接（`CalleeRef::Import` 在 AV 派发中不可执行），宿主导入是新增类别，不是旧 import 的放宽。

本Plan不含：FS/网络/进程/argv/env/time（属 R3-H）、f-string/完整格式化（C-25 另议）、stdin 交互、UI/服务入口、async。

## 1. 目标

- 源码 `print`/`eprint` 以**具名能力**进入：未授权、未知符号、签名不符在准入前拒绝，零副作用可观察。
- 输出成为父侧可断言的**精确字节观察轴**（有上限、有阶段、可反控），与 guest 返回值/trap/资源失败分列。
- 格式化最小而明确：`int`（十进制、无前导+、负号前置）、`bool`（`true`/`false`）、`str`（原字节，含 UTF-8 与非 ASCII）；每调用一条行终结符；不做宽度/对齐/插值。
- 宿主导入的形状版本化、可扩展且**不把宿主指针/句柄写进portable产物**；跨进程恢复后重新准入，不恢复信任。
- 不改 `av.core-i32/1` 与 AVAD v1–v5：新 profile + 新 ABC section + 新语义轴。

## 2. 架构方案

按[Design43 §4 宿主与互操作](../design/43-ac-av-typed-mir-and-abc.md)与 §12.1 三层表示：

| 层 | 本Plan动作 | 边界 |
|---|---|---|
| 源码前端 | 新 `SourceProfile`（含 862 的入口/Unit 语义）+ `print`/`eprint` 调用形式、参数类型检查、诊断 | 不引入模块/use（R3-M）、不引入 f-string |
| HIR/MIR | 宿主声明作为显式引用（symbol + 签名 + 能力），MIR 中可区分 host call 与 guest call | 不把宿主调用降级成 guest import |
| ABC | 新增 host-import section（类别/符号/签名/能力/所需资源），与闭包 import 分区；版本号新 | 旧 session/closure import 语义零改动 |
| 准入 | verifier：符号已声明、签名等价、能力在授权集内、未知/越权在派发前拒绝 | 不允许“先跑再撞墙” |
| AV 执行 | host call 派发经受限接口：fuel/取消/输出上限；宿主调用不得回调 guest 重入 | 不作内存隔离承诺 |
| 宿主实现 | probe 侧 console provider（有界缓冲，超限 = 具名资源失败，非截断成功） | 不接线产品 CLI/SDK 默认路径 |

写面不含产品默认切换：本Plan只在实验 workspace 与测试语料内交付，不改变 `auto run` 的默认引擎（G7 才谈默认）。

## 3. 技术栈

- 源码：`experimental/ac-core/src/source/*`、`src/{hir,descriptor,verify}.rs`。
- MIR/ABC/AV：`experimental/ac-av-probe/core/src/{mir,mir_verify,abc,abc_verify,av,contract}.rs`。
- 消费者：`experimental/ac-av-probe/probe/src/{hir_adapter,session,driver,formal_worker}.rs`、新 `console_host.rs`（拟建）。
- T-04 实际消费者边界：共享 lowering 为 `experimental/auto-mir-lower/src/hir_adapter.rs`；whole-module console 宿主/持久化入口为 `probe/src/console_host.rs`，不启用旧 session import。`experimental/ac-mir-native/src/{fingerprint,lower}.rs` 仅补共享 enum 的受控拒绝臂（不增加 native console 支持）；shared lowering version=3，对应父侧 `crates/auto-lang-core/src/test_cases/{ac_av,ac_mn,ac_cte}.rs` 三处身份镜像同升。
- 观察协议：`docs/specs/auto-test-case/architecture.md` 已规定父侧 stdout 读取带界与 stderr 排空/杀树；本Plan在此之上定义**断言轴**，不改既有 infra 行为。语料：`crates/auto-lang/test/cases/ace-m1/`。
- 已实现并实跑 `scripts/verify-av-863.ps1`（Scope=targeted/compat/console/acceptance），测试前缀 `av_console_863`；最终证据与命令见 T-05 交接摘要。

唯一实施树 `D:/autostack/.wt/lang-863/auto-lang`，分支 `plan-863-dev`。

## 4. 需求分析与背景调查

用户 2026-10-28 授权同上（只规划）；2026-10-30 `/goal 完成计划863` 授权本计划执行、独立复审、合同内有界修复、合入、Spec/ledger 沉淀、归档与 guard 清树，后续阶段继承授权。事实基础：

- [auto-av/project](../specs/auto-av/project.md)：798 交付 Checked HIR→Typed MIR→ABC→私有 LoadedProgram→Rust AV；ABC 有 `MirImport{env,symbol}`，仅用于闭包单元链接（[resident-function-install](../specs/auto-av/resident-function-install.md)：`fn:<name>` 绑定/import 闭包门）。
- `experimental/ac-av-probe/core/src/av.rs` 的 `CalleeRef::Import(_)` 在派发路径返回不可执行（注释：pending unit linkage / unresolved import = infra），`abc_verify.rs` 明确“no Call may target an import”。→ 宿主导入是**新类别**，不是现有字段的复用。
- 能力矩阵 C-29：`print/stdin/stderr、普通CLI结果` 当前为“—：trace 不是 print”，归 G2/G4 + R2-E/R4-P；C-30（FS/argv/env/进程）与 C-31（JSON/regex 等）仍缺，本Plan不认领。
- 852 判定协议：成功=exit 值、diag 分级、infra 非语言判定；`binding.tsv`（124 行 = 63 corpus + 40 scenario + 注释）以 i32 exit 为唯一值观察。→ 输出需要**新增观察轴**，否则 print 只能靠副作用外的间接推断，不可反控。
- 旧源码打印语义（[能力矩阵 §3 SEM-05/SEM-06](../design/strategy/av-language-capability-matrix.md)、C-25）：显示格式历史存在差异，`trace` 不是 print；旧 bug 不自动成为新规范。

## 5. 详细设计

### 5.1 前置与写面

硬前置：`D862`（入口/Unit/结果通道）与 **`D868`（lowering 出库）**——`print`/`eprint` 的源码 lowering 落在共享 crate。两者未交付时保持 drafting（只做只读核查，不消费 WIP）。写面见 §3；另：`crates/auto-lang/src/tests/ace_m1_tests.rs` 的追加断言、`docs/specs/auto-test-case/architecture.md`（仅追加断言轴章节，不改既有 infra 规则）。

### 5.2 合同冻结（T-02 决策件）

T-02 产出 `attachments/863/work/host-console-contract.md`，冻结：

| 决策项 | 要求 |
|---|---|
| 调用形式 | `print(<expr>)`、`eprint(<expr>)`；expr 限 `int`/`bool`/`str`/`char`（是否含 `Str` 拥有值按 821 既有转换规则，T-01 核实）；拒绝多参/无参/其它类型 |
| 格式化 | int 十进制（`-` 前缀、无 `+`、无前导零、MIN 正确）；bool `true`/`false`；str/char 原字节或码点 UTF-8；每调用一个 `\n`；无空格/宽度/插值 |
| 能力 | 具名能力（建议 `console.stdout` / `console.stderr`），授权集来自宿主；未授权=准入拒绝（零输出、零派发计数） |
| 符号 | 稳定符号名与签名版本（如 `auto.console.print.v1`），符号+签名+能力三元组进 ABC 索引 |
| 预算 | 每次调用与整次运行两级输出上限；超限=具名 resource 失败（不截断成功）；输出计入内存/IO 预算 |
| 取消 | 取消后进行中的 host call 立即失败并可观察；已写出的字节不被回滚（明确写“已发生副作用不撤销”） |
| 观察轴 | 父侧断言 `stdout`（字节精确/行序列）、`stderr`、`exit`、`trap/resource` 分列；断言缺失≠通过 |
| 版本 | 新执行 profile 三元组（suggest `av.host-console/1`、profile id 8、wire 新世代）与源码 profile 名由 T-02 写定 |
| 隔离声明 | 明确“同进程原生调用不提供内存隔离”，不作为不可信插件沙箱（沿 Design43 §5） |

### 5.3 准入与执行细节

- ABC host-import 表与闭包 import 表分区，各自计数与校验；两者不得互相冒充（把 host 符号写成闭包 import 必须拒绝，反之亦然）。
- verifier：未知符号/签名不符/能力越权/重复声明 → 各自具名码；**在任何派发前**拒绝。
- AV：host call 计入 fuel 与调用深度；宿主返回类型化结果（成功/资源失败/取消/宿主错误），宿主错误不得伪装成 guest trap。
- provider：console 有界缓冲 + 上限检查；写失败（宿主 I/O）为 infra，与语言结果分列。
- store/恢复：host 声明与授权事实进入 envelope/身份轴；恢复后重新准入；授权撤回后旧产物不得继续执行该能力（沿用 [822 授权预检](../specs/auto-av/resident-function-install.md)形状）。

### 5.4 测试框架接入（协议层的真实改动点）

输出断言不是“给某个 case 加字段”那么简单：当前 `crates/auto-test-case/src/contract.rs::channel_supported` 对 `Adapter::Probe` 只开放 `probe` 与 `exit`，**declared-but-unobservable = admission ERROR**（不允许解析后忽略）。因此本Plan在协议侧必须显式做三件事（T-02 冻结、T-05 落地）：

1. `channel_supported(Adapter::Probe, "stdout"|"stderr")` 置真，并在 `decode_expect` 的 probe 已知通道集追加同名项（旧 case 不声明即行为不变）。
2. 父侧准入新增“捕获流”校验：按 case 声明做精确字节/上限比较，复用 `compare_stream_field` 同源入口（不另写比较器）；recipe 不产出而 case 声明 = ERROR，不静默当空。
3. worker 只回传原始流字节与截断标记，**PASS 判定仍全在父侧**；超限是 resource/infra，不是“截断即通过”。

recipe 侧：`print`/`eprint` 是新增宿主导入类别，优先新注册 `ac-av/console-run-v1`（三缝 append-only，与既有 recipe 互不复用 id），worker 域分支承载 console provider；不用“给 core-run 偷偷塞宿主调用”的方式改动其已冻结语义。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/host-import-and-console.md | 无 → AVHC/1、profile 8、HOST 声明节/0x35 HostCall；console/1 类型化符号/能力/64KiB+1MiB 预算/派发/具名错误/同步重入拒绝/重新授权恢复；whole-module surface，console function-unit 受控拒绝 mir.host-unit；旧 profile 不变 | 首个宿主能力 | AC-02/03/04 |
| SD-02 | add | docs/specs/auto-ac/source-console-output.md | 无 → print/eprint 语法、格式化、诊断 | 真实程序可输出 | AC-01 |
| SD-03 | modify | docs/specs/auto-test-case/architecture.md | 追加输出断言轴（stdout/stderr 精确字节+上限），infra 规则不变 | 可判决 | AC-05 |
| SD-04 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 追加 console 行与轴映射 | 证据落点 | AC-05 |
| SD-05 | modify | docs/specs/auto-av/project.md | 追加新 profile 与旧 profile 边界 | 模块可见 | AC-03 |
| SD-06 | modify | docs/specs/auto-mir/hir-lowering-boundary.md | current lowering version 2→3，记录 863 HostConsole/host table/call 与三消费者镜像；公开面追加 lower_checked_console，保留历史 2/881 与 1/868 | SD-01 已实现版本的 canonical 同步；R2 知识增量最终化，非合同扩面 | AC-06/08 |

## 6. 测试设计

| V | AC/不变量 | 实际/拟建入口 | 可区分错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 源码面 | 新语料：int/bool/str/char 打印、UTF-8、负数/MIN、连续多行；错型/多参/无参 | 精确字节序列；错型必须在编译期拒绝而非运行期空输出 | T-03 |
| V-02 | AC-02 准入 | 未授权/未知符号/签名不符/重复声明；host 符号伪装成闭包 import | 派发计数=0、零输出；变异必红 | T-04 |
| V-03 | AC-03 预算/取消/失败 | 输出超上限（两次越界形态）、取消中途、宿主写失败 | 超限=resource（非截断成功）、取消可观察、写失败=infra | T-04 |
| V-04 | AC-04 恢复/授权 | 新进程 decode/verify 后执行；授权撤回后的旧产物 | 恢复后重新准入；撤回后拒绝且零输出 | T-04 |
| V-05 | AC-05 观察轴与语料 | 父侧 stdout/stderr/exit 三轴语料 + 缺轴/错轴负控 + 852 行追加 | 缺断言不计绿；错字节必红；SKIP 不计绿 | T-05 |

定向命令（T-05 建立后填实际）：两独立 workspace 的 `cargo test`（core/probe）+ `powershell -File scripts/verify-av-863.ps1 -Scope <targeted|compat|console|acceptance>`；完整 `cargo t`+`tv` 归 merge。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | print/eprint 源码面与格式化按冻结件成立，错型编译期拒绝；V-01 |
| AC-02 | work | 宿主导入准入（符号/签名/能力）在任何派发前成立，host/closure import 互不冒充；V-02 |
| AC-03 | work | 输出上限/取消/宿主失败三类可区分，无截断成功；V-03 |
| AC-04 | work | 新进程恢复后重新准入，授权撤回后拒绝且零输出；V-04 |
| AC-05 | work | 父侧三轴断言与 852 行落地，独立 oracle，缺轴/SKIP 不计绿；V-05 |
| AC-06 | review | 独立核 AC/V/SD、实际派发与准入数据流、无宿主指针入产物、warnings/fmt |
| AC-07 | merge | 冻结提交裸 `cargo t` + `tv` 通过；失败不豁免 |
| AC-08 | merge | Spec 沉淀（SD-01..05）、ledger、归档、guard 清理完成 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | D862 | 核实 ABC/MIR import、AV 派发、test-case 父侧路径；`attachments/863/work/baseline.md`，提交 b1d8b68b2 | AC-01/02 | [x] |
| T-02 | work | T-01 | `attachments/863/work/host-console-contract.md` 冻结 §5.2 与三元组、预算、V-01..05 关闭条件；提交 b1d8b68b2；语义实现未执行 | AC-01..05 | [x] |
| T-03 | work | T-02 | 源码/HIR 新域、print/eprint 类型化声明/能力/诊断、借用文本、入口效果语句；b8f133cfa；8 新测 + 69 兼容定向测试通过。V-01 精确运行字节待 T-04 | AC-01 | [x] |
| T-04 | work | T-03 | 563eec65e + 6583d21ad 基础实现；3ff90f15e 修复 F-863-R1-01/02/03；20 runtime + 1 artifact 身份测试通过，含修复前失败反控；见 repair-r1-summary.md | AC-02/03/04 | [x] |
| T-05 | work | T-04 | 84f1d2054 父侧/语料证据保持有效；3ff90f15e 修正 F-863-R1-04 候选 SD-01/02，保留 R1 parent 11/11 与 AC-05 证据；无合同内遗留 | AC-05 | [x] |
| T-06 | review | T-05 | R2 final pass，3ff90f15e；同会话 artifact-based review（无独立模型/会话），F01..04 关闭；merge 门 pending；见 review-r2.md | AC-06 | [x] |
| T-07 | merge | T-06 pass | 冻结提交裸 `cargo t` + `tv` | AC-07 | [ ] |
| T-08 | merge | T-07 | Spec/ledger/归档/guard 清树 | AC-08 | [ ] |

## 9. 复审记录

2026-10-28 new r1：`stage: new | plan_id: PLAN-863 | plan_revision: 1 | outcome: pass（等待 D862 实施前置，合同可预检） | next: D862 交付后 work T-01`。全部 AC/V/T 未执行；profile 号/符号名/能力名/上限值均为建议值，T-02 冻结。未建树、未跑构建、未改在途Plan。批次排序依据见[后继批次规划](reports/ace-next-batch-2026-10-28.md)。

2026-10-30 work（T-01/T-02 完成，实施仍在执行）：`stage: work | plan_id: PLAN-863 | plan_revision: 1 | outcome: progress（非 work pass，不进入 execution_done） | worktree_path: D:/autostack/.wt/lang-863/auto-lang | branch: plan-863-dev | base_commit: eda05a86f2f6bd29d3b70479917360d64a908a66 | code_commit: b1d8b68b2（仅基线与合同文档） | dependency_revisions: D862 delivery 0e62d8c4c、D868 delivery 7754c4e4c，均已在 master 历史 | task_ids: T-01,T-02 | blockers: none | next_responsible_role: work T-03`。

- 授权：用户 `/goal 完成计划863`，目标工具已有同名 active goal，沿用而未新建；全链授权见 §4。
- 主检出仅有其它会话文档 WIP（design/plan/report），无代码 WIP；保持原样，禁止纳入 863 提交。
- 唯一树已创建并核实 `git rev-parse --show-toplevel` 与 `git branch --show-current`；实现未写入主检出。
- 静态核查：旧 import 无宿主派发；EntryUnit 不接纳 text，需新增独立源码/HIR 域；profile id 8 未分配；父侧真实实现为 `crates/auto-lang-core/src/test_cases/ac_av.rs`，共享比较器 `ac.rs::compare_stream_field`，流上限 1MiB。旧 `probe/src/hir_adapter.rs` 已删除，lowering 写面改用 `experimental/auto-mir-lower/src/hir_adapter.rs`。
- T-02 采用 §10 默认项：stdout/stderr 同批、print/eprint 为新 profile 保留字；每调用 64KiB（含 LF）、整次运行双流合计 1MiB（含 LF），不超过父侧既有每流 1MiB 边界。完整合同在 worktree 的 `docs/plans/attachments/863/work/host-console-contract.md`，随 b1d8b68b2 可恢复。尚未执行语义测试；AC-01..08 均未宣称通过；T-03..08 未完成。

2026-10-30 goal round 1 checkpoint：`stage: goal | plan_id: PLAN-863 | plan_revision: 1 | round: 1 | from: work | to: work | status: executing | outcome: pass（仅 T-01/T-02 检查点；不是 work-complete） | worktree: D:/autostack/.wt/lang-863/auto-lang | commit: b1d8b68b2e044bdc0e6f4b989da7494c8ef3437a | next: work T-03`。合同 SHA256 `f2c791a6e330c3d6eccad1eba2b02fec615282963dc0947a6965dc87718714ec`；基线摘要 SHA256 `559dfe119b4d8abbfa4ab891c9e42a0e635eaefa894953743308b72972d54d0d`（路径均为上述 worktree 下 `docs/plans/attachments/863/work/`）。`git status --short` 在实施树为空；提交钩子通过。未执行构建/测试，未进入 review/merge，目标保持 active。

2026-10-30 work T-03：`stage: work | plan_id: PLAN-863 | plan_revision: 1 | outcome: progress（非 work-complete） | worktree_path: D:/autostack/.wt/lang-863/auto-lang | branch: plan-863-dev | base_commit: eda05a86f | code_commit: b8f133cfa | checkpoint_commit: 2f309713b942c942983b443c8775cbbc6ed12cff | task_ids: T-03 | blockers: none | next_responsible_role: work T-04`。

- 实现真实源码→独立 HIR 域→受检声明数据流：`SourceProfile::HostConsole` / `HirDomain::HostConsole`，descriptor `host-console.atom`，`compile_host_console_source` / `verify_host_console`；typed intrinsic 携带精确签名、稳定符号、observable effect 与 requires，lowering 后续必须转为独立 host reference，尚未启用旧 import 执行。
- `print/eprint` 一位置参数；i32/bool/str/Str/char；Str 转只读视图；Unit 不可存储/转整型；保留名绑定拒绝。输出效果语句不被误判成脚本尾值。text ownership/resource walk 复用现有实现但保持新域身份；其它 profile 保留既有身份与接受面。
- 定向检查：ac-core `host_console_863` **8/8**；`entry_unit_862` **26/26**；`scalar_source` **7/7**；`text_source` **10/10**；`source_frontend` **14/14**；`source_hir` **12/12**；独立用例总数 **77**（最终 entry/console 重跑 34 例不重复计数），0 failed/ignored。覆盖 HIR 重绑定/符号签名变异/缺能力声明/跨域拒绝及 dead-source 参数错/Unit 不可转换等反例。ac-core 类型检查无告警；fmt/diff-check 通过。
- probe 全 tests 类型检查首次发现 `function_incremental_828_contract` 缺 HostConsole 与预存 Closures match arms；已分别接真实 verifier，随后该受影响测试与 probe lib 类型检查 exit 0。未改其它 881/821/owned_session 文件，既有告警已在摘要逐项登记；review 时核对。
- 本地摘要（已提交的 md）`D:/autostack/.wt/lang-863/auto-lang/docs/plans/attachments/863/work/frontend-summary.md`，SHA256 `d526fbfb30bc614b3534e1ee81c351b08bb98bf944701634d07c945c0d4a6023`。AC-01 的精确运行输出仍待 T-04；AC-02..05 实现未完成。未运行整档 t/tv/tf/docs_gen，未送审或合入。

2026-10-30 goal round 2 checkpoint：`stage: goal | plan_id: PLAN-863 | plan_revision: 1 | round: 2 | from: work | to: work | status: executing | outcome: pass（仅 T-03 源码/HIR 检查点；非 work-complete） | worktree: D:/autostack/.wt/lang-863/auto-lang | commit: 2f309713b942c942983b443c8775cbbc6ed12cff | next: work T-04`。上一轮为 progress（T-01/T-02 文档提交）；本轮为 progress（源码/HIR 实现提交与 77 项定向测试证据）。实施树 clean，授权不变，无待用户决策。

2026-10-30 work T-04：`stage: work | plan_id: PLAN-863 | plan_revision: 1 | outcome: progress（非 work-complete） | worktree_path: D:/autostack/.wt/lang-863/auto-lang | branch: plan-863-dev | base_commit: eda05a86f | code_commit: 6583d21ad（其前实施 563eec65e） | checkpoint_commit: f5874901b79764b7ea93f8e82a23f573d346e4b6 | task_ids: T-04 | blockers: none | next_responsible_role: work T-05`。

- 真实路径：新源码/HIR 域→共享 `lower_checked_console`→独立 `MirHostImport`+`HostCall`→verifier→AVHC/1（profile 8、HOST section、opcode 0x35）→decode/deep admission→每次执行 current grants→AV→有界 `ConsoleCapture`。未复用或放宽 guest import，未把 print 伪装成 trace/guest 返回整数。provider/native 地址、句柄与 grant credential 不进入 portable 内容。
- V-01..04：独立精确字节 oracle（i32 MIN/负数/0、bool、UTF-8/NUL/换行、char、Str/str）、guest 调用顺序/文本投影/既有副作用保留；11 类真实 wire 声明/引用/类型变异；未授权（含不可达声明）零派发、泛用 executor 无 provider 拒绝；64KiB/call 与 1MiB 双流合计的等界/越界；fuel/depth/memory 与输出上限分辨；host 内置取消/IO 失败、guest owner 正常释放；同步重入所有 guest dispatch 拒绝并能正常恢复后续执行；managed-store 新进程恢复及同一 loaded credential 撤权后零输出/派发。
- 最终 console 定向集 **17/17 passed**，exit 0；本轮另有 **55** 个受影响 core/entry/data/owned/compact/native/指令合同检查通过（计数与命令逐项见摘要，不重复计入调试重跑）。已有 `av_data_820_semantics` 指令表已知版本 pin 从 52 正常推进至 53，先观测红后补 HOST_CALL 行覆盖。共享 lowering 变更引起全局身份从 2→3，三个父侧镜像同步；父侧真实 worker/断言集成及根类型检查由 T-05 一并完成。
- whole-module AVHC artifact 是本次新增执行面；`mir.host-unit` 显式拒绝未引入的 console function-unit linking，既有 session/closure 语义不变；native 仅为共享 enum 扩展补受控拒绝，非支持扩面。此为实际 producer/consumer 路径澄清，未缩减 AC 或把要求延后。
- 证据摘要：`D:/autostack/.wt/lang-863/auto-lang/docs/plans/attachments/863/work/runtime-summary.md`（随 f5874901b 固定）；原始日志同目录 `runtime-verification.log` **本地保留未提交**，SHA256 `b923bcb7c2547e9eac42623a1c8d5cb98ab166bc161be5de141cba52ed8a8cc2`。probe 既有 4 处告警已逐项记为 baseline，无新增告警；core/lowering 类型检查、所改文件 rustfmt 与 diff-check 通过。
- AC-01（运行字节）与 AC-02..04 已有直接 work 证据；AC-05 尚未交付。未跑整档 t/tv/tf/docs_gen，未进入 final review/merge。

2026-10-30 goal round 3 checkpoint：`stage: goal | plan_id: PLAN-863 | plan_revision: 1 | round: 3 | from: work | to: work | status: executing | outcome: pass（仅 T-04 runtime 检查点；非 work-complete） | worktree: D:/autostack/.wt/lang-863/auto-lang | commit: f5874901b79764b7ea93f8e82a23f573d346e4b6 | next: work T-05`。上一轮为 progress（源码/HIR 实现）；本轮为 progress（AV/portable/授权/预算/恢复实现及真实反控证据）。实施树 clean，无用户决策阻塞；继续新 recipe、父侧 stdout/stderr/exit 观察轴、语料与脚本。

2026-10-30 work T-05 最终交接：`stage: work | plan_id: PLAN-863 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-863/auto-lang | branch: plan-863-dev | base_commit: eda05a86f2f6bd29d3b70479917360d64a908a66 | code_commit: 84f1d20546bfc683b9abbbecf22e6006be3b9145 | task_ids: T-05（T-01..05 全部 work-complete） | blockers: none | next_responsible_role: final review T-06`。

- AC-01..04：前序 frontend/runtime 摘要与本轮 8 源码 + 18 runtime 定向复跑；AC-05：真实父侧新 console **15/15 PASS**、旧 scalar/data **18/18 PASS**、旧 entry/Unit **12/12 PASS**；contract **25**、parent admission **2**、parent integration **5** 全通过。均 exit 0，不把跨命令重复语料累计成独立测试数。
- 父侧独立断言 stdout/stderr 精确 bytes 与 exit/Unit/trap；必需双流+结果/错误 oracle，缺轴直接 admission ERROR，错任一轴 FAIL。required capture 字段、schema/lowering、canonical base64、present/nontruncated、双流合计 1MiB、互斥结果面均受检；resource/cancelled/infra 是 ERROR，保留真实 raw prefix。新 recipe 限 d_main/单 run/无 session/profile override；lower helper 入口绕过负控已补。
- V-05 含真实 wrong-stdout/wrong-stderr/wrong-exit、missing-axis、actual SKIP、NUL 字节差异、超限前 prefix、report Fresh 与改 profile→Stale；worker 不接收 oracle/PASS。O01..O15 新 AV 列按实测置 `ok:av-host-console-source-863`；peer native/legacy/a2r 的新 profile parity 保留显式 gap，不新增 na，不改旧 oracle/分母，不宣称 M1 完整绿。
- 可复现命令：预建 `cargo build --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --bin av_probe_worker`；`cargo build -p auto-lang --no-default-features --features test-cases --bin auto-case`；设 `AC_AV_CASE_WORKER=D:/autostack/.wt/lang-863/auto-lang/experimental/ac-av-probe/target/debug/av_probe_worker.exe` 后执行 `./scripts/verify-av-863.ps1 -Scope acceptance -OutDir docs/plans/attachments/863/work/final-acceptance`，targeted/compat/console/acceptance 均 PASS。暂存时清理新 case EOF 空行并统一新语料 LF 后，重跑 `-Scope console -OutDir docs/plans/attachments/863/work/final-acceptance/console`，15 PASS + 5 parent tests，exit 0；旧输入/实现未变，旧兼容证据复用。
- 本轮根 `cargo check -p auto-test-case`、`cargo check -p auto-lang-core --features test-cases` 通过；修改 Rust 文件 rustfmt、最终 staged diff-check、提交钩子通过。根 core 149 / CLI 3 / UI compiler 39 及 probe 既有告警归 baseline；新增实现零新增告警。未混入全仓 warning/fmt 清理。
- 根 path 依赖只读 sibling：auto-ui `2349d0ec99b3aa21c3f64975965bc105bf6e469c`、auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`，路径分别为 `D:/autostack/.wt/lang-863/auto-ui`、`.../auto-down`，均 clean detached、零实现改动，收尾 guard 清理。主检出其他文档 WIP、888 等在途树均未触碰。
- 已提交摘要：`D:/autostack/.wt/lang-863/auto-lang/docs/plans/attachments/863/work/parent-summary.md`，SHA256 `be7707111a9466cd0e217521cd4bf517926dec3acb9517327c99551a6dfd8cf2`；候选 SD-01..05 全文 `proposed-spec-delta.md`，SHA256 `7ca3d8ea44b91be90125ad8785908eaf71fe28227516b2133610bbf3546d3787`。原始日志/报告仅本地忽略，不入库：同目录 `final-acceptance-summary.log` SHA256 `593fe36fec36ca28ae3b9d66e08d99e61fdbe73a87afe5f71081a4ff718bb8cd`；最终 LF 语料报告 `final-acceptance/console/console/report.json` SHA256 `3029507d10cec2be6d56f53455f42a79f3647b934996bf9ee7e4a1fa3298a7e3`；其他逐文件 hash 见摘要。
- readiness：work-owned AC-01..05 完成；无已知 open in-contract finding、必需覆盖缺口或待决合同项。AC-06 独立最终代码/验收/Spec 审查待执行；merge-owned AC-07 裸 `cargo t` + `cargo tv` **pending**；AC-08 canonical Specs/ledger/归档/guard 清树 **pending**。本轮未执行 t/tv/tf/docs_gen，不宣称交付。

2026-10-30 goal round 4：`stage: goal | plan_id: PLAN-863 | plan_revision: 1 | round: 4 | from: work | to: review | status: execution_done | outcome: pass（work 阶段） | worktree: D:/autostack/.wt/lang-863/auto-lang | commit: 84f1d20546bfc683b9abbbecf22e6006be3b9145 | next: final review T-06`。实施树 clean，所有授权 work 任务已提交；目标保持 active，待复审与合入交付。

2026-10-30 final review R1：`stage: review | review_scope: final | reviewer_identity: Codex（同实施会话，独立 artifact-based review 步骤；无独立模型/会话上下文，未指定其他 final reviewer） | plan_id: PLAN-863 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-863/auto-lang | branch: plan-863-dev | reviewed_commit: 84f1d20546bfc683b9abbbecf22e6006be3b9145 | base_commit: eda05a86f2f6bd29d3b70479917360d64a908a66 | dependency_revisions: auto-ui 2349d0ec99b3aa21c3f64975965bc105bf6e469c; auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4（均 clean detached） | next: work 修复 R1`。

- 完整 findings/AC/V/SD/Spec 输入 hash 与最小关闭 oracle 见 [R1 审查报告](attachments/863/review-r1.md)。实施代码在 review 全程冻结，入/出 HEAD 均 84f1d2054、clean；本轮只写本地忽略探针与主检出审查/Plan/债务记录。未发布 canonical Spec 或 ledger。
- F-863-R1-01（P2，AC-01/T-04）：`lower_checked_console(...,d_helper)` 拒绝，但新接纳 HostConsole 的公开 `lower_checked_owned` 绕过固定 main；真实 compile→MIR→ABC→verify→AV 输出 **99\n / I32(11)**，合法 main 控制组 **7\n / I32(42)**。须在所有接受该域的公开 source lowering 路径统一约束，保留旧 text/owned 的原入口选择能力。
- F-863-R1-02（P2，AC-03/T-04）：capture 已有 3 bytes、memory cap=2、output cap=1MiB，实际 ConsoleOutput（root/dispatch=0），应 SlotBytes；现有测试仅测 retained==memorycap 之后的 root 相加分支，漏掉 strict-greater 前置分支。关闭需 memory-only/output-only/合法复用三控制。
- F-863-R1-03（P2，AC-04/T-04）：`print(1)` reference snapshot **14081312560767487292→14081312560767487293**，wire sidecar 仍旧值，却 load/persist/restore/执行全部成功（stdout `[49,10]`、Unit）。期望 source identity 必须对 portable payload 实核，不能只把 caller 标签放进 key/envelope；保留真实 fresh-process/current-grant 控制。
- F-863-R1-04（P2，AC-06/SD-01/02/T-05）：候选 Spec 把 generic provider 拒绝扩大到所有 profile8（实际仅非空 host table），把 `int` main 的尾值限制扩大到所有 main（Unit main `print(1);42` 仍是 Unit，非隐式返回）。按已存在行为/862规范澄清，未要求扩改旧语法或新增执行限制。proposed delta 冻结 SHA256 `7ca3d8ea44b91be90125ad8785908eaf71fe28227516b2133610bbf3546d3787`。
- 审查实跑：source **8**、runtime **18** 全过；额外真实 parent driver **11** 反控：合法 PASS/错字节 FAIL；truncated/missing/malformed base64/lowering2/absent stream/Unit-with-payload/resource/cancelled/infra 均 ERROR。不是只调用私有 helper。owned Unicode slice 控制输出 **世界\n / I32(7)**；`return print(1)` 受 source.unit-return-value 拒绝。44 个变更 Rust 文件 rustfmt 与全 diff-check 通过，既有告警已核 baseline。
- AC-01 partial/F01；AC-02 pass；AC-03 partial/F02；AC-04 partial/F03；AC-05 pass；AC-06 needs_fix/F04+实现阻塞；AC-07/08 merge-owned **pending**。无未审查高风险域留下，但不把通过的旧案例当作 F01..03 的关闭证据。新增 peer parity gap 记 D-863-NB-01，绝不替代这些合同内修复。
- 本地 raw replay prefix：`D:/autostack/.wt/lang-863/auto-lang/docs/plans/attachments/863/review-r1/`。`probe/src/main.rs` SHA256 `4c5b9fc8e83070399a27504c20e88bcbd1b4a342072c33e4bc9d3696cb954b36`；`probe.log` SHA256 `c81516df47bb27346585940239b09ee012aef848ae3b00905722488786279de0`；`parent-faults.log` SHA256 `35e46f7e9e70b47d04619c2a7064196ccfb6f09b821761d4b274ad5de7fae5fd`。命令 `cargo run --manifest-path docs/plans/attachments/863/review-r1/probe/Cargo.toml --target-dir experimental/ac-av-probe/target --bin p863-review-r1`，exit0表示观察程序完成，**不表示不变量通过**。原始探针/日志/存储均本地忽略未暂存；报告内保留数值与可复现输入，供清树后审计。

2026-10-30 goal round 5：`stage: goal | plan_id: PLAN-863 | plan_revision: 1 | round: 5 | from: review | to: work | status: executing | outcome: needs_fix | commit: 84f1d20546bfc683b9abbbecf22e6006be3b9145 | next: 修复 F-863-R1-01..04 后定向验证/提交/再复审`。前一轮分类 progress，本轮以新增可复现证据推进；失败 final review 1 次，自动 repair cycle 1。重新打开 T-04 与依赖 T-05，current_step=3；未改 AC/合同阈值，目标 active，无外部阻塞。未跑整档 t/tv/tf/docs_gen，禁止合入。

2026-10-30 work R1 repair：`stage: work | plan_id: PLAN-863 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-863/auto-lang | branch: plan-863-dev | base_commit: 84f1d20546bfc683b9abbbecf22e6006be3b9145 | code_commit: 3ff90f15eb51401aab2b84dfcb9ee1e0b21953fd | dependency_revisions: auto-ui 2349d0ec99b3aa21c3f64975965bc105bf6e469c; auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4（clean detached） | task_ids: T-04,T-05 | blockers: none | next_responsible_role: final review T-06`。

- finding→invariant→适用路径/状态→永久断言→红绿证据与剩余边界见实施树已提交 `docs/plans/attachments/863/work/repair-r1-summary.md`。未改变 r1 的目标/AC/范围；同一次自动修复 cycle 1，final review 失败累计 1 次，当前尚未再次 review。
- 原始 R1 探针重放复现三个实现反例；新回归各在修复前失败（exit101），修复后 `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test host_console_863` **20/20**，`--lib av_console_863_snapshot_identity_checked` **1/1**（均 exit0 / 0 ignored）。包括公开 owned helper 拒绝与旧 text helper 可选、main/script/Unit 合法控制、retained memory-only/output-only/both/合法复用、逐函数 sidecar 和 reference 篡改在 load/persist/run/restore 全路径拒绝 `host.identity`；零派发/零输出，合法 fresh-process/恢复/当前授权保留。
- 本地 raw 路径 `D:/autostack/.wt/lang-863/auto-lang/docs/plans/attachments/863/review-r1/`：`repair-runtime.log` SHA256 `3a4863d3a88b7fe428f39a72596563661d5dd8ba0339b19f13eab21014647923`；`repair-snapshot.log` `c0e2df76f28749242e9d2a58bef23f08f104157232c63edca28d57014abd8ec8`；原探针 `pre-repair.log` `d87d72287e0b840b92acf8ddd94a56b88acd5515d1704746901e561420c79bb7`。三个 red log hash 与命令详见摘要；非 md 证据未暂存。
- F04 修正为“非空 host 表需要 provider”和“int main 禁止隐式返回；Unit main 尾值丢弃”，未扩大旧语法限制；候选 Spec delta SHA256 `a449f02667884c8ceae3324e4bbb1052480692dc415c484a1ad2c982699af813`。新增 snapshot 全函数一致性/资源类别及双超限 output 优先的精确文字。
- readiness：AC-01..05 work-owned 完成，无 open in-contract finding、必需覆盖缺口或待决合同项；AC-05 源码/父侧/语料/配置与依赖未变，复用 R1 source 8、兼容定向 69、parent 故障控制 11 及既有 15 行语料。受影响 runtime 同族扩展检查因共用改动的 lowering/admission/memory 路径而执行；未重跑其它全档。4 个改动 Rust 文件 rustfmt check / diff-check exit0；编译由定向测试覆盖，无新增告警。实现树 clean，实施写入冻结等待 final review。
- AC-06/T-06 待独立 artifact-based 最终复审；AC-07 merge-owned 裸 `cargo t` + `tv` **pending**；AC-08 canonical Specs/ledger/归档/guard 清理 **pending**。未运行 t/tv/tf/docs_gen；未改 canonical Specs/ledger、未合入。主检出其他会话文档 WIP 原样保留。

2026-10-30 goal round 6：`stage: goal | plan_id: PLAN-863 | plan_revision: 1 | round: 6 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-863/auto-lang | commit: 3ff90f15eb51401aab2b84dfcb9ee1e0b21953fd | next: review`。R1 四项工作侧修复已提交、已定向验证；下一轮复审冻结提交，目标 active。

2026-10-30 final review R2：`stage: review | review_scope: final | reviewer_identity: Codex（同实施会话 artifact-based review，无独立模型/会话上下文，未指定其他 final reviewer） | plan_id: PLAN-863 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-863/auto-lang | branch: plan-863-dev | reviewed_commit: 3ff90f15eb51401aab2b84dfcb9ee1e0b21953fd | base_commit: eda05a86f2f6bd29d3b70479917360d64a908a66 | dependency_revisions: auto-ui 2349d0ec99b3aa21c3f64975965bc105bf6e469c; auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4（clean detached） | findings: R1-01..04 closed; no new blocking findings | next: merge`。

- 完整闭环/AC/V/SD/复用边界见 [R2 审查报告](attachments/863/review-r2.md)。原始公开 API 探针复查：console/owned helper 皆 lower.entry，main 皆 7\n/I32(42)；retained memory-only→SlotBytes 且 root/dispatch=0；reference snapshot 改值后的 load/persist/run 与**修复前已写入的 forged store restore**皆 host.identity/零输出。Unit-tail→Unit/1\n、return-effect→source.unit-return-value、no-host generic→I32(42)；旧 owned slice→世界\n/I32(7)。三项永久回归定向复跑各 1 pass / 0 ignored，exit0；逐函数 sidecar 篡改、四准入边界及合法控制均实核。44 个 Rust 文件 rustfmt + 全 diff-check exit0，无新增告警/延期/workaround。
- 实施入/出 HEAD 同为 3ff90f15e 且 clean；代码、依赖、配置、原始语料身份稳定，复用 repair 20 runtime+1 artifact 和 R1 未受影响 source/parent/compat 证据；未重跑整档。canonical 6 输入 SHA256 与 R1 一致。候选 SD-01..05 hash 仍 `a449f02667884c8ceae3324e4bbb1052480692dc415c484a1ad2c982699af813`；R2 报告补充明确的 SD-06 canonical lowering version 2→3 同步文字与 supersedes 路径（原 SD-01 已描述/实现的版本，不改语义合同）。未发布 canonical Specs/ledger。
- 本地 raw prefix `D:/autostack/.wt/lang-863/auto-lang/docs/plans/attachments/863/review-r2/`：`public-replay.log` SHA256 `12bcb8fbec07cc61881680835ced0fda1f1564a349144fc96b6327ff4b33f005`；`entry.log` `ec591afa0d4f905dd048622c3cef583df57c54abd2007f8333703e5e3aaea5c9`；`budget.log` `0f5e150f3a7a02cd27581cdaf5f12cdc4626437de8f556efb54591b89f545448`；`snapshot.log` `c8386f19c3a086269ea99997e602d114853e8b7ab25e4da051e82e98fb9cbcd3`。命令/探针 hash 与实际值已保存在报告；所有 raw 仅本地忽略，未入库。
- AC-01..06 pass，T-01..06 完成（current_step=6/8）；AC-07 裸 cargo t + cargo tv **merge-owned pending**；AC-08 Spec/ledger/archive/guard/landing **merge-owned pending**。未跑 t/tv/tf/docs_gen；门禁成功前不得合入或宣称交付。D-863-NB-01 保留显式 peer parity gap，不替代任何合同内要求。

2026-10-30 goal round 7：`stage: goal | plan_id: PLAN-863 | plan_revision: 1 | round: 7 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-863/auto-lang | commit: 3ff90f15eb51401aab2b84dfcb9ee1e0b21953fd | next: merge`。上一轮 progress；本轮四 finding 关闭与最终审查 pass，final review 累计 2（1 fail/1 pass），repair cycle 1，目标 active。

## 10. 待澄清事项

- 首批是否同时交付 stderr（`eprint`）与 stdout，还是只 stdout。默认两者同批（同一派发形状，边际成本小），如用户要求收窄可只留 stdout。
- `print` 是否作为**关键字**还是**预导入符号**（可被用户函数遮蔽/拒绝遮蔽）。默认：保留字不可作名字，且不允许用户重定义（与既有“用户函数不能被 intrinsic 劫持”规则一致，T-01 核实）。
- 输出上限的具体数值（每行/总字节）需 T-02 按 852/825/826 既有资源预算口径给出，不由本草案编造。

2026-10-28 D868 前置解除：PLAN-868 已交付落 master（delivery 7754c4e4c）——HIR→MIR lowering 唯一实现为 `experimental/auto-mir-lower/src/**`（纯搬移等价已证，旧 `probe/src/hir_adapter.rs` 已删除），本计划 D868 硬前置视为满足；lowering 相关 T 任务的写面以该 crate 为准（边界合同见 [docs/specs/auto-mir/hir-lowering-boundary.md](../specs/auto-mir/hir-lowering-boundary.md)）。
