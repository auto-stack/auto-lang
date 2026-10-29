---
plan_id: PLAN-863
status: executing
feature_name: AV类型化宿主导入与控制台输出（R2-E/print）
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-30
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-av/host-import-and-console.md
  - docs/specs/auto-ac/source-console-output.md
touched_goals: [GOAL-002]

affects: [auto-av, auto-mir, auto-ac, auto-test-case]
current_step: 3
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
- 观察协议：`docs/specs/auto-test-case/architecture.md` 已规定父侧 stdout 读取带界与 stderr 排空/杀树；本Plan在此之上定义**断言轴**，不改既有 infra 行为。语料：`crates/auto-lang/test/cases/ace-m1/`。
- 拟建脚本 `scripts/verify-av-863.ps1`（Scope=targeted/compat/console/acceptance），测试前缀 `av_console_863`；当前不存在、未运行。

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
| SD-01 | add | docs/specs/auto-av/host-import-and-console.md | 无 → 宿主导入类别/能力/预算/派发/错误/恢复 | 首个宿主能力 | AC-02/03/04 |
| SD-02 | add | docs/specs/auto-ac/source-console-output.md | 无 → print/eprint 语法、格式化、诊断 | 真实程序可输出 | AC-01 |
| SD-03 | modify | docs/specs/auto-test-case/architecture.md | 追加输出断言轴（stdout/stderr 精确字节+上限），infra 规则不变 | 可判决 | AC-05 |
| SD-04 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 追加 console 行与轴映射 | 证据落点 | AC-05 |
| SD-05 | modify | docs/specs/auto-av/project.md | 追加新 profile 与旧 profile 边界 | 模块可见 | AC-03 |

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
| T-04 | work | T-03 | ABC host-import section + verifier + AV 派发 + provider；V-02/03/04 | AC-02/03/04 | [ ] |
| T-05 | work | T-04 | 观察轴 + 语料/oracle/脚本；V-05，提交代码/证据 | AC-05 | [ ] |
| T-06 | review | T-05 | 独立最终review；merge 门 pending | AC-06 | [ ] |
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

## 10. 待澄清事项

- 首批是否同时交付 stderr（`eprint`）与 stdout，还是只 stdout。默认两者同批（同一派发形状，边际成本小），如用户要求收窄可只留 stdout。
- `print` 是否作为**关键字**还是**预导入符号**（可被用户函数遮蔽/拒绝遮蔽）。默认：保留字不可作名字，且不允许用户重定义（与既有“用户函数不能被 intrinsic 劫持”规则一致，T-01 核实）。
- 输出上限的具体数值（每行/总字节）需 T-02 按 852/825/826 既有资源预算口径给出，不由本草案编造。

2026-10-28 D868 前置解除：PLAN-868 已交付落 master（delivery 7754c4e4c）——HIR→MIR lowering 唯一实现为 `experimental/auto-mir-lower/src/**`（纯搬移等价已证，旧 `probe/src/hir_adapter.rs` 已删除），本计划 D868 硬前置视为满足；lowering 相关 T 任务的写面以该 crate 为准（边界合同见 [docs/specs/auto-mir/hir-lowering-boundary.md](../specs/auto-mir/hir-lowering-boundary.md)）。
