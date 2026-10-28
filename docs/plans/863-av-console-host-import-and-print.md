---
plan_id: PLAN-863
status: drafting
feature_name: AV类型化宿主导入与控制台输出（R2-E/print）
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-av/host-import-and-console.md
  - docs/specs/auto-ac/source-console-output.md
touched_goals: [GOAL-002]

affects: [auto-av, auto-mir, auto-ac, auto-test-case]
current_step: 0
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

用户 2026-10-28 授权同上（只规划）。事实基础：

- [auto-av/project](../specs/auto-av/project.md)：798 交付 Checked HIR→Typed MIR→ABC→私有 LoadedProgram→Rust AV；ABC 有 `MirImport{env,symbol}`，仅用于闭包单元链接（[resident-function-install](../specs/auto-av/resident-function-install.md)：`fn:<name>` 绑定/import 闭包门）。
- `experimental/ac-av-probe/core/src/av.rs` 的 `CalleeRef::Import(_)` 在派发路径返回不可执行（注释：pending unit linkage / unresolved import = infra），`abc_verify.rs` 明确“no Call may target an import”。→ 宿主导入是**新类别**，不是现有字段的复用。
- 能力矩阵 C-29：`print/stdin/stderr、普通CLI结果` 当前为“—：trace 不是 print”，归 G2/G4 + R2-E/R4-P；C-30（FS/argv/env/进程）与 C-31（JSON/regex 等）仍缺，本Plan不认领。
- 852 判定协议：成功=exit 值、diag 分级、infra 非语言判定；`binding.tsv`（124 行 = 63 corpus + 40 scenario + 注释）以 i32 exit 为唯一值观察。→ 输出需要**新增观察轴**，否则 print 只能靠副作用外的间接推断，不可反控。
- 旧源码打印语义（[能力矩阵 §3 SEM-05/SEM-06](../design/strategy/av-language-capability-matrix.md)、C-25）：显示格式历史存在差异，`trace` 不是 print；旧 bug 不自动成为新规范。

## 5. 详细设计

### 5.1 前置与写面

硬前置：`D862`（入口/Unit/结果通道）。若 862 未交付，本Plan保持 drafting（不消费 WIP）。写面见 §3；另：`crates/auto-lang/src/tests/ace_m1_tests.rs` 的追加断言、`docs/specs/auto-test-case/architecture.md`（仅追加断言轴章节，不改既有 infra 规则）。

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
| T-01 | work | D862 | 核实 ABC/MIR import 现状、AV 派发缝、test-case 观察轴能力；产出 `attachments/863/work/baseline.md` | AC-01/02 | [ ] |
| T-02 | work | T-01 | 冻结合同（§5.2 全表）与版本三元组 | AC-01..05 | [ ] |
| T-03 | work | T-02 | 源码前端 print/eprint + 诊断；V-01 | AC-01 | [ ] |
| T-04 | work | T-03 | ABC host-import section + verifier + AV 派发 + provider；V-02/03/04 | AC-02/03/04 | [ ] |
| T-05 | work | T-04 | 观察轴 + 语料/oracle/脚本；V-05，提交代码/证据 | AC-05 | [ ] |
| T-06 | review | T-05 | 独立最终review；merge 门 pending | AC-06 | [ ] |
| T-07 | merge | T-06 pass | 冻结提交裸 `cargo t` + `tv` | AC-07 | [ ] |
| T-08 | merge | T-07 | Spec/ledger/归档/guard 清树 | AC-08 | [ ] |

## 9. 复审记录

2026-10-28 new r1：`stage: new | plan_id: PLAN-863 | plan_revision: 1 | outcome: pass（等待 D862 实施前置，合同可预检） | next: D862 交付后 work T-01`。全部 AC/V/T 未执行；profile 号/符号名/能力名/上限值均为建议值，T-02 冻结。未建树、未跑构建、未改在途Plan。批次排序依据见[后继批次规划](reports/ace-next-batch-2026-10-28.md)。

## 10. 待澄清事项

- 首批是否同时交付 stderr（`eprint`）与 stdout，还是只 stdout。默认两者同批（同一派发形状，边际成本小），如用户要求收窄可只留 stdout。
- `print` 是否作为**关键字**还是**预导入符号**（可被用户函数遮蔽/拒绝遮蔽）。默认：保留字不可作名字，且不允许用户重定义（与既有“用户函数不能被 intrinsic 劫持”规则一致，T-01 核实）。
- 输出上限的具体数值（每行/总字节）需 T-02 按 852/825/826 既有资源预算口径给出，不由本草案编造。
