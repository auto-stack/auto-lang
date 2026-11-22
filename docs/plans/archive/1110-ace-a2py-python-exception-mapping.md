---
plan_id: PLAN-1110
status: archived
feature_name: ACE A2PY 的 Python 异常映射与同源位置验收
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-22
plan_revision: 2
depends_on: [PLAN-1107, PLAN-1094]
supersedes_spec_components:
  - docs/specs/auto-a2x/python-ecosystem-integration.md
  - docs/specs/auto-a2x/python-stdlib-readiness.md
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace/a2x-python, ace/a2x-worker, auto-test-case]
current_step: 6
total_steps: 6
---

# [PLAN-1110] ACE A2PY Python 异常映射与同源位置验收

## 0. 变更摘要

2026-11-20，用户要求为 A2PY 的 Python 异常映射单独立项。1094 交付 AV 的可捕获错误语义和共享 Auto 用例。1110 在 1107 的原生 Python 路径上实现映射，以同源用例验收 AV 和 A2PY 的异常捕获及源码位置。本次交付文档，实施和运行验证待后续阶段。

r2 复查补齐未捕获 Case、双端异常类型和位置的独立断言，以及同一源从作者文件到执行入口的绑定。1094 r5 给出 AV 独立交付条件。

## 1. 目标

- 受检 `use.py` 操作抛出的普通 Python 异常进入 Auto 错误语义。存在处理分支时执行 `catch`，未捕获时报告语言错误，两种结果都保留原 Auto 操作的位置。
- 每个必验 Case 的 AV 和 A2PY 检查读取同一逻辑路径、同一份源码字节，各自真实执行并对照独立预期。
- 范围为现有 Python 操作的异常、捕获、诊断及对应的测试入口。通用 AV 脚本和对象协议由 1094 负责。跨函数展开、完整 `with` 协议、容器跨界和平台扩面沿各自计划交付。

## 2. 架构方案

`use.py` 源码 → ACE 共享前门的受检操作事实、Auto 错误语义和原始 span → 现有 A2PY 生成器 → 原生 Python 执行 → Auto handler 或结构化未捕获诊断。同源 AV 检查走 1094 正式交付的普通源码链。A2PY 在受检操作执行边界构造语言错误，沿现有 helper、runner 和父侧比较消费实际结果。

## 3. 技术栈

`ace-a2x-core`、`ace-a2x-python`、`ace-a2x-worker`、AV Python 会话及 AutoDown TestCase。首轮沿 1107 已验 Windows x64 和 CPython 3.14.2，用 Python 标准库触发异常。现有 PyTorch 用例作为回归保留。

## 4. 需求分析与背景调查

本轮授权为 auto-lang 内的文档立项和交接说明。静态阅读基线 `5a76167cc64d945637d90aa0d28fd7ef260b7302` 与相关模块，测试均待后续实施。[Specs 总览](../specs/overview.md)

| 来源 | 观察与差额 |
|---|---|
| [1107 r2](archive/1107-ace-python-av-a2py-integration.md)与[Python 生态规范](../specs/auto-a2x/python-ecosystem-integration.md) | 1107 已归档。同源执行与 978 工程/SDK 已交，已验未捕获场景在 AV 为 `ForeignStatus`，A2PY 为非零退出及 Python 异常类。 |
| [1094](archive/1094-av-parity-script-and-python-protocols.md)与[控制流规范](../specs/ace-migration/m2-control-effects.md) | 1094 已按 AV-only 归档，交付 `IF-AV-PARITY-SCRIPT` 及两条共享异常 Case。1110 消费其正式错误分类与位置。 |
| [py_ops.rs](../../experimental/ace/crates/core/src/py_ops.rs)、[lower.rs](../../experimental/ace/crates/a2x-python/src/lower.rs)、[runtime.rs](../../experimental/ace/crates/a2x-python/src/runtime.rs) | `PyOpFact.span` 记录操作表达式范围。A2PY lowering 消费受检事实，helper 直接执行 Python 协议。 |
| [print.rs](../../experimental/ace/crates/a2x-python/src/print.rs) | `SourceMapEntry` 把声明范围映射到生成行区间。区分同一函数里的失败表达式需要核验操作级位置。 |
| [AV worker](../../experimental/ace/crates/av/src/language_source_worker.rs) | AV 通用 worker 用 `probe.at` 编译并省略 `Error` / `Trap` 的 span。1094 负责 AV 观察接线。 |
| [A2PY TestCase adapter](../../crates/auto-lang-core/src/test_cases/ac_a2py_hir.rs)、[A2X worker](../../experimental/ace/crates/a2x-worker/src/bin/a2x-worker.rs) | stdlib adapter 当前只收 `Syntax::Auto`，以 `source.at` 暂存。现有 `emit_py` 按普通 core/scalar/record/text/collections profile 编译，`run_py` 返回实际退出码和双流。1110 负责接通 `main.as` / `auto-script` 的受检 Python 编译和异常结果往返，保留作者源身份。 |
| [共享语料规范](../specs/auto-test-case/feature-organized-corpus.md) | 作者文件按语言特性组织，一份 `### Auto` 供多个目标检查读取，AV 视图只选 AV。 |

## 5. 详细设计

### 5.1 职责与依赖

| owner | 交付与消费 |
|---|---|
| 1107（已交底座） | 原生 A2PY Python 操作、单次求值、运行和已有异常/工程回归。 |
| 1094（上游、AV-only） | 交付捕获和未捕获两条 Case、实际 AV check、类型与位置的复验入口和来源指纹。T-06 冻结证据，T-08 正式落主交付 `IF-AV-PARITY-SCRIPT` 的 Python 错误切片。[交接表](archive/1094-av-parity-script-and-python-protocols.md#python-错误切片交接) |
| 1110（A2PY 消费方） | 消费共同语义和真实 AV 基线。在现有 A2PY 实现映射、诊断和位置传递，承接 A2PY check 与双端验收及位置反例。 |

方向为 1107、1094 → 1110。T-01 可先核 1107 的已交入口。T-02..04 消费正式合入并有适用证据的 1094 错误切片，可接收单独落主的切片收据。仅发现 A2PY 接线、目标结果协议或测试工具存在缺口时，1110 自行收口。AV 已交合同或证据不足时，1110 的相应验收等待上游修复。1094 依其自身 AV 断言合入，A2PY 检查、工具和 1110 的评审均由 1110 阶段选择。

### 5.2 映射与位置

- 从受检 Python 操作构造符合 1094 合同的 Auto 错误。两条必验 Case 都比较实际 Python 类型 `ValueError` 与 Auto 语义类别 `value`。保留约定的消息信息，AV provider 数字和 A2PY 本地表示按语义归一。既有 runner 的 `lang.error` 指示运行结果大类，异常细分类字段或等价实际观察另行断言。
- receiver 与各实参按源序各求值一次。嵌套操作在内层失败时保留内层位置。捕获取 handler 实际处理的错误、处理次数与后续入口值。未捕获取真实运行终态的结构化语言错误，A2PY 同时核实际进程非零及无成功标记。两种观察都绑定同次执行，父侧以独立 oracle 核类型与位置。
- 位置按 Case 逻辑路径和原始源 UTF-8 半开字节范围比较，行列按项目约定换算。显式核 runtime 错误的位置能回到受检操作，先校验逻辑路径、语法模式和源 SHA256，再对位置。暂存路径与生成文件位置仅作内部信息。`source.at` 作为磁盘暂存名时，仍须以原 `main.as` 和 `auto-script` 进入共享前门。
- 授权后实际发生的 Python 语言异常进入映射。解释器或必需包准入失败、宿主授权、资源/预算、取消和运行框架错误维持各自路径。进程级中断及退出沿运行器合同。控制流与资源释放沿现有实现。
- T-01 核实前门预处理后的 span 可追溯性，在直接携带操作位置和扩展现有 source map 之间定最小可行方案。持久化的位置元数据与实际源码/产物身份绑定。只在实际协议变更处更新版本并验证缓存失效。

### 规范增量

| delta_id | 操作 | 目标 | 规则变化 | 原因 | 验收 |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-a2x/python-ecosystem-integration.md` | 追加已验的捕获、未捕获、原始位置及支持范围 | 描述两端已验行为 | AC-01/02/03 |
| SD-02 | modify | `docs/specs/auto-a2x/python-stdlib-readiness.md` | 追加真实使用的异常结果、源码身份及 runner 观察合同 | 使执行消费者可核验位置 | AC-02/03 |

AV 规范归 1094 沉淀。本轮的规范增量均为提案。

## 6. 测试设计

共享锚点为 `av.parity.1094.python-error-catch` 和 `av.parity.1094.python-error-uncaught`，由 1094 写入同一作者文件 `crates/auto-lang/test/cases/language/script/python-context.case.ad`。1110 从正式作者读取两个 Case，在各自 Case 内加 A2PY check。捕获和未捕获是两个独立场景，每个场景的 AV 与 A2PY 共用同一段 `### Auto`。[1094 用例与独立预期](archive/1094-av-parity-script-and-python-protocols.md#不变量入口与观察)

每个必验 Case 采用一段源和两个执行 check。AV 用 1094 接通的 `ac-av/language-source-run-v1` 路径，A2PY 走 `ac-a2py/hir-stdlib-run-v1` / `run-py` 所在的既有文件集和真实解释器路径，脚本准入及结果接线在本计划核定。父侧断言实际输入的路径和 SHA256 相同，依实际通道核对程序值、捕获事件、未捕获错误及位置。预期来自共享语言合同与独立 Python 标准库参考。

| 用例 | 验收与执行 owner | 输入与区分性判定 |
|---|---|---|
| V-01 捕获及成功对照 | AC-01/02，T-02/T-04 | 捕获 Case 两端入口值为 `42`，handler 一次。由捕获观察核对 `ValueError`、`value` 和 `main.as` 内层 `int("abc")` 的原范围（1094 LF 草案 `[51,61)`）。成功对照 `int("7")` 返回 `7`，捕获次数为零。 |
| V-02 未捕获与位置变化 | AC-01/02，T-03/T-04 | 未捕获 Case 的两端运行终态均为 Auto 语言错误，零捕获，无成功值，类型为 `ValueError`、Auto 类别为 `value`，原位置为 `main.as` 内层 `int("abc")`（LF 草案 `[45,55)`）。A2PY 进程非零。偏移原源位置的同源对照重验新范围，整函数、helper、旧坐标及外层 `.to(int)` 均令断言失败。 |
| V-03 分类与求值 | AC-01/03，T-02/T-04 | 以 1107 的空列表 `pop()` 为 `IndexError` / `index` 分类对照，嵌套失败以计数或副作用核对求值次数和内层位置。缺解释器、权限拒绝及运行限制走各自具名结果，用户 catch 触发次数为零。 |
| V-04 真实入口与回归 | AC-02/03/04，T-03/T-04 | 每条 Case 两端各自真正运行。捕获和未捕获各做错类型、错范围、缺观察负控，另核 emit 无执行观察时失败。重验普通 Auto 错误、1107 Python 未捕获和训练、A2PY 原生运行及受影响的 SDK 消费。 |

拟用 `cargo t ace-a2x ace-python case-av-language-parity`，改 adapter 加 `ace-adapters`，按实际共享修改补 `ace-source ace-semantics ace-lowering ace-artifacts ace-m2-build` 中的相关组。AV corpus 组是 AV 视图。T-01 核定具名 A2PY check 选择和双端位置断言的实际命令。Windows 只跑适用腿，Linux/WSL 归专用 Linux 环境。

## 7. 验收标准

- [x] AC-01（owner_stage=work）V-01..03 两端捕获和未捕获的类型均符合独立预期，实际 handler 与终态分别验证。正常对照捕获次数为零，嵌套副作用符合源序。
- [x] AC-02（owner_stage=work）V-01/V-02/V-04 两端捕获及未捕获都有实际错误携带的原始逻辑路径与失败操作范围。同一次执行的源指纹、类型、结果和位置均能核对，错误或缺失的观察令验收失败。
- [x] AC-03（owner_stage=work）V-03/V-04 各类失败沿对应结果路径，1107 既有场景及普通 Auto 错误回归通过。
- [x] AC-04（owner_stage=work）V-04 从同一个作者源取得真实 AV 与 A2PY 结果，两端输入指纹一致，AV 视图可独立选择 AV。
- [x] AC-05（owner_stage=review）独立复核 AC-01..04、SD-01..02、接口和负控，将结果绑定实际 revision 和提交。
- [x] AC-06（owner_stage=merge）冻结门通过，沉淀已验规范与合入收据。按仓库流程归档和清理。

## 8. 执行步骤

实施前在可用的 TODO 工具登记任务 ID。专属树 `D:/autostack/.wt/lang-1110/auto-lang`，分支 `plan-1110-dev`。开工登记唯一材料目录 `D:/autostack/plan-artifacts/plan-1110/`。

- [x] T-01 核对接口与位置
  - owner_stage: work。先以 1107 核对受检 span、声明 map、真实 runner 和 `.as` 准入。1094 接口落主后用锚点查证 AV 实际行为与 A2PY 各环节的接入差额，先确定最小方案，再由 T-02..04 完成捕获、位置与双端验收。
  - 关闭条件为记录两条已交 AV check 的正式入口和预期、源身份、坐标、准确命令与观察形态，以及 A2PY 接线决策。保留原 `.as`、`auto-script` 与源码字节。关联 AC-01..04。
- [x] T-02 实现 A2PY 捕获映射
  - owner_stage: work。前置 T-01 及正式 AV 错误切片。沿现有受检前门和 A2PY `lower.rs`、`runtime.rs` 接 Python 操作与 Auto handler，先以真实 runner 核 V-01..03 的捕获、未捕获分类及求值。位置和双端验收由 T-03/04 关闭。关联 AC-01/03。
- [x] T-03 接通源码位置与诊断
  - owner_stage: work。前置 T-02。沿现有 `print.rs`、`run-py` 与 `ac_a2py_hir.rs` 接通同源路径、语法模式及实际类型和位置。用 V-01/V-02 核对捕获和未捕获观察，跑 V-04 的缺失及错误观察负控。关联 AC-02/03。
- [x] T-04 完成同源双端验收
  - owner_stage: work。前置 T-02/03。复用 1094 的 canonical Case、AV check 和适用独立预期。仅启用本计划承接且实际通过的 A2PY check，新增对照沿一源多 check 规则登记。
  - 关闭条件为 V-01..04 双端真实结果及受影响职责组证据。关联 AC-01..04。
- [x] T-05 独立复审
  - owner_stage: review。前置 T-04。对冻结提交重建验收映射，核实原始 span、真实运行和规范增量。关联 AC-05。
- [x] T-06 合入归档
  - owner_stage: merge。前置 T-05 pass。按审定并集和仓库清理规则收尾。移除专属树前用 `pwsh -NoProfile -File D:/autostack/wt-guard.ps1 <worktree>` 核对。关联 AC-06。

## 9. 复审记录

2026-11-22，`stage: merge | plan_id: PLAN-1110 | plan_revision: r2 | outcome: pass（delivered）| consolidation_receipt: PLAN-1110:r2`。

- `prepared`：复审基线 `ece92f65b`（r2 final pass）；SD-01/SD-02 在专属树落稿并提交 `fd828c232`（docs-only 后代）；rebase 到 master（`ca7c8e07e`）零冲突，补丁等价核对——本计划触碰的全部文件字节一致，差异仅为 master 自身 1116/1117/1053 文件；旧→新映射 `ece92f65b→d4e658d14`、`fd828c232→729bf91e5`。
- 冻结门（merge 所属，专属树）：准备 `python scripts/fixed-tests.py <6组> --prepare-only`（14m44s，6 invocation）+ `cargo t --run-prepared`（3m49s）。首轮 case-a2py-language-parity PASS（4/4）、ace-python PASS（10）、ace-m2-build PASS（45）、ace-a2x FAIL、ace-adapters FAIL（1）、case-av-language-parity FAIL（2）。
- **门禁修复（return-to-work）**：`ace-a2x` 组 `boundaries::manifest_face_only_sanctioned_dependencies` 红——本计划给 `ace-a2x-python`/`ace-a2x-worker` 新增的 `tempfile` 依赖不在 V-13 白名单（工作阶段只跑定向 nextest 二进制，未跑整组，故漏网）。修复 `9c9135825`：按 1116 `auto-bindgen`/`base64` 先例把 `tempfile` 以 PLAN-1110 注释加入许可清单；boundaries 5/5 复绿，`ace-a2x` 整组重准备（6.3s）+ 重跑 PASS（349，1m45s）。修复仅触碰 `a2x-worker/tests/cases/boundaries.rs`，其余 5 组输入未变，首轮 PASS 证据保持有效。
- **既有红（非本计划所致，具名排除）**：① `case-av-language-parity` 的 `av.parity.1093.explicit-c-library`/`c-header-spelling-rejected` —— `parity1093_c` C provider 源未入库（1093 既有债务，PLAN-1125 已立项），`ACE_AV_C_PROVIDER_DIR` 未设时 prepare 具名跳过、用例判 FAIL；master 主检出同条件复现同样 2 红（收据 plan-artifacts 已核，`master-av-group/results.json`，529 执行 2 红同 ID 同 detail）。② `ace-adapters` 的 `abi::source_protocol_tests::source_parent_oracle_failure_rejudges...` —— master 主检出 nextest 复现红（120s 超时），专属树内 8.4s 断言红，双端均红，本计划 diff 不触 abi/source-protocol 面。
- `landed`：wt-guard clean → `git merge --ff-only plan-1110-dev`，master tip = `9c9135825`（delivery commit），无 merge 提交，线性历史。
- `canonical_verified`：`docs/specs/auto-a2x/python-ecosystem-integration.md`（8115B，sha256 前缀 1d8b98d6d954d7f0，§2/§3/§5 更新 + §6 新增）、`docs/specs/auto-a2x/python-stdlib-readiness.md`（9233B，sha256 前缀 9580e6174bb57414，1107 附录第 3 条修订 + PLAN-1110 附录新增），均经 `729bf91e5` 落主，PLAN-1110 溯源在文。
- `archived`：`git mv` 至 `docs/plans/archive/`，status: archived，AC-06/T-06 勾选，current_step 6/6，completion_kind: delivered。
- merge 会话计时：定向复跑 3 测试 0.602s（warm）；boundaries 复跑 0.163s；6 组准备 14m44s + 门禁 3m49s；ace-a2x 重准备 6.3s + 重跑 1m45s；master 对照准备 2m41s + 运行 2m44s；auto-case 主检出重建 27.3s。资源等待：让位其他会话 `cargo t ace` 约 4.7 分钟。新增用例无超 10s（见 work 记录）。
- `cleaned`：见下条收据。

2026-11-22，`stage: review | review_scope: final | reviewer_identity: ZCode 会话（独立于实施会话 Codex；结论从工件重建，非转述执行方摘要）| plan_id: PLAN-1110 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1110/auto-lang | branch: plan-1110-dev | reviewed_commit: ece92f65b548635f44b8c9deb6aaea0e520676d2 | base_commit: b2a616c779aaafb1c97029787c88ef4343392785 | dependency_revisions: 1094@ce8ba4fbc,1107@535b34a01（均为祖先）| spec_inputs: docs/specs/auto-a2x/python-ecosystem-integration.md@r1107, docs/specs/auto-a2x/python-stdlib-readiness.md@r1107 | acceptance_results: AC-01 pass, AC-02 pass, AC-03 pass, AC-04 pass | pending_closeout_gates: 冻结并集 `cargo t case-a2py-language-parity case-av-language-parity ace-adapters ace-a2x ace-python ace-m2-build`（merge 所属）| findings: 0 blocking | evidence: 见下 | next: merge (T-06)`。

独立核验要点（全部对照冻结提交 `ece92f65b`，树 clean）：

- **基线与身份**：worktree 无未提交改动；单实现提交叠 base `b2a616c77`；收据 `fixed-a2py-debug-run/results.json` SHA256 实测 `f22d17cac26971a99050fceb05b8b9048e852a6f52eea757047fceb8195c0ef8` 与记录一致，内含 4 条 A2PY check PASS（1057/1290/1366/1314 ms，仅选中行执行，882 项矩阵其余 NOT_RUN 具名）。
- **复跑（本会话新增运行）**：`cargo nextest run --locked --offline -p ace-a2x-python --test py_native_run_1107 -p ace-m2-build --test py_project_1107 -E 'test(p1110)' --test-threads 1` → 3 PASS（caught 0.154s / uncaught 0.142s / 缓存前缀位移 0.304s；warm 构建 0.48s，无新例超 10s）。scratch 限定 `plan-artifacts/plan-1110/review-native/`。
- **span 独立验证**：从作者文件字节直接重算——catch `int("abc")`@51..61、uncaught@45..55、nested-index `items.pop()`@81..92，与 oracle 完全一致（非自证：native 测试硬编码 51/45 并按 `int("abc")` 10 字节推 end）。
- **同源双端（AC-04）**：4 份 paired report 原始工件显示每 Case 的 av/a2py 共享同一 source SHA256（fe536c29…/c85fe9c2…/3f98fc39…/f9895004…），AV 视图经既有 `--check av` 独立选择。
- **负控**：result-*-type/position/missing 共 6 组原始 report，错类型/错位置 → A2PY FAIL 各 1 mismatch；缺观察 → 双端 FAIL（av 3 / a2py 2-3 mismatch）；与记录一致。
- **身份绑定核查**：`oracle` 哈希只覆盖期望块（catch-av 与 nested-index-av 期望块等价故同哈希，属正常）；区分性 error-type/kind/span 断言在 options 侧，由 `contract = fingerprint(CaseSpec)` 覆盖——改任一 span/kind 均使旧 PASS 失效，绑定成立。`{kind}=4` 为既有语言错误 catch 通道常量（与 a2x-rust PLAN-1115 约定一致）。
- **健康检查**：本提交零新增编译警告（强制重编 4 个触碰 crate 核对）；ace-core/ace-lowering/ace-av/ace-native 及 a2x-worker 既有警告均为 base 存量（`compile_language_source` 未用 import 在 base 已存在），diff 无遗留调试输出（测试内 eprintln 为计时输出，惯例）。
- **规范增量**：SD-01/SD-02 目标文档存在，现行文本带"尚未映射/未捕获=退出码+异常类"未交标记，增量描述与实现一一对应（python-script 准入、logical_path/语法/SHA256 身份回验、≤4KiB 独占创建观察文件、escaped/handled 计数、a2x-store 位置上下文缓存、m2-build python helper 哈希轴）。均为提案，落主归 merge。

2026-11-22，`stage: work | plan_id: PLAN-1110 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1110/auto-lang | branch: plan-1110-dev | base_commit: b2a616c77 | code_commit: ece92f65b548635f44b8c9deb6aaea0e520676d2 | dependency_revisions: 1094@ce8ba4fbc,1107@535b34a01 | task_ids: T-01..04 | blockers: none | next_responsible_role: review (T-05)`。

实现依据受检 `PyOpFact` 和 `LoopUnwind`。生成程序在实际 Python 操作周围保留错误类别和字节范围，Auto handler 仍消费已有 kind/message 控制流。runner 由独立有界观察传递实际类型、是否逃逸和捕获次数。父侧以同次执行的逻辑路径、模式、源码指纹及结果判定，AV 捕获检查也从已有 `language_error.span` 断言范围。缓存对含操作的位置来源附加目标片段上下文，工程构建身份加入实际 Python helper 内容哈希。阶段验证按 Windows x64 / CPython 3.14.2。

| 验证入口 | 实际结果与范围 | 实测耗时 |
|---|---|---|
| `auto-case module-case` 从 canonical language suite 运行捕获、未捕获、嵌套错误和正常对照 | 8 PASS，4 个作者源各 2 个目标。AV 为 549/525/581/542 ms，A2PY 为 901/963/1071/1014 ms。原类型、位置及入口值分别实断。 | 每次含发现与清理 3.382/3.382/3.556/3.627 s。首次普通 `run --case` 的整套汇总退出 1，选中行全 PASS，随后以 `module-case` 的单 Case gate 复验退出 0。 |
| `cargo t case-a2py-language-parity --prepare-only` ＋ `cargo t --run-prepared` | 4 PASS，实际只选择 A2PY check。捕获/未捕获/嵌套错误/正常分别 1057/1290/1366/1314 ms。root worker 和解释器均已绑定实际结果。 | debug 准备 48.531 s、子运行 7.092 s、总运行 9.820 s，发现/清理未单独计量。首次按 target 选择走到旧 release 准备，已中止并改用已支持的 check 选择，正式收据只用 debug。 |
| `cargo nextest run --locked --offline -p ace-a2x-python --test py_native_run_1107 -p ace-m2-build --test py_project_1107 --test-threads 1` | 16 PASS。1107 训练、原生异常、身份值流、SDK 重定位/缺包、纯 Auto 工程回归。新增原生捕获与未捕获分别 0.201/0.204 s，新增缓存前缀移动并真实执行 0.542 s。 | 构建 46.03 s，发现未单独计量，nextest 执行汇总 41.825 s，清理包含在测试内。15.377 s 的 SDK 重定位是原有用例。 |
| 外部 scope 内 `auto-case module-case` 错类型/错位/缺观察控制 | 捕获与未捕获各三条，6 条均在同一 A2PY oracle 给预期 FAIL。真实 worker 拒绝 emit-only；缺解释器、缺授权和 Auto 除零沿各自路径。 | 单条含发现与回收 1.458–1.504 s。缺解释器 0.036 s，缺授权 0.216 s，普通 Auto 除零 0.284 s。 |

新增语料检查及 Rust 用例均在 10 秒以内。未开始独立复审。冻结完整职责组合并门由后续阶段执行，按受影响的 `case-a2py-language-parity`、`case-av-language-parity`、`ace-adapters`、`ace-a2x`、`ace-python`、`ace-m2-build` 确认选择。结果收据 `D:/autostack/plan-artifacts/plan-1110/fixed-a2py-debug-run/results.json`，SHA256 `f22d17cac26971a99050fceb05b8b9048e852a6f52eea757047fceb8195c0ef8`。固定组短运行目录 `tmcase-_wyyfnz1` 已回收，专属树与材料范围留给后续 review/merge。

2026-11-22 开工。用户授权实施 PLAN-1110。base `b2a616c77`，专属树 `D:/autostack/.wt/lang-1110/auto-lang`，分支 `plan-1110-dev`，auto-down 兄弟树 `456c86a`。1094 的 `ce8ba4fbc` 和 1107 的 `535b34a01` 都在基线祖先中。使用 1094 唯一作者的两条异常源，A2PY 消费 `compile_language_source_with_foreign`、受检 `PyOpFact.span` 和 HIR `LoopUnwind`。先用真实 Python 运行核类型与字节范围，再接 worker/Case 的身份及观察。AV 捕获用例当前有实际 handler 次数，其已有 `language_error.span` 观察需要接入父侧断言。材料目录登记在 `D:/autostack/plan-artifacts/plan-1110/scope.json`。

2026-11-20，`stage: new`，`PLAN-1110 r1`，`status: drafting`。计划负责 A2PY，双端闭环消费 1094 的正式 AV 切片。`outcome: pass` 表示文档合同可交接。下一阶段入口为实施授权后的 T-01，依赖接口的工作随对应切片就绪进入。

静态核对通过：编号和任务结构、文档链接、与 1094 候选锚点的源码一致性、现有固定组、计划依赖无环。实现和行为验收待后续阶段。

2026-11-20 r2：静态复查与 1094 r5 的交接。V-01/V-02 明确两条共享 Case，补齐类型和原位置断言。V-04、AC-01/02 覆盖错误或缺失观察。A2PY 入口与证据消费由本包负责，1094 的 AV 交付独立。

## 10. 待澄清事项

T-01 按 1094 正式交付确认消息格式、错误类型/位置的实际观测入口，以及 A2PY 原始 span 映射和共享脚本准入方式。异常类型、语义类别、源身份和两种执行结果的验收要求见 §5/6。若正式接口有差额，按 §5.1 的归属处理。
