---
plan_id: PLAN-1113
status: executing
feature_name: "ACE AV/RS 普通工程与 Rust FFI"
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-20
plan_revision: 1
current_step: 1
total_steps: 7
depends_on: [PLAN-967, PLAN-978, PLAN-1043, PLAN-1078]
supersedes_spec_components: [docs/specs/ace-migration/m2-host.md, docs/specs/ace-migration/m2-build.md]
new_spec_components: []
touched_goals: [GOAL-002, GOAL-003]
affects: [auto-av, auto-abi, ace-m2-build]
---

# [PLAN-1113] ACE AV/RS 普通工程与 Rust FFI

## 0. 变更摘要

Rust 路线第一批。交 `IF-ACE-AV-RS-PROJECT/1`，供 1114 用同一个普通工程入口装配标准库。保留 967、1043 与 1093 已验切片的真实范围与身份。 各批依赖和交接见 [Design 60](../design/60-ace-rust-route.md)。

## 1. 目标

普通 Auto 工程通过受检 `use.rs`/`dep` 调用真实 Rust 库，并能从预编译工件运行。覆盖已在 967、1093 承诺的 Rust 类型、资源、回调和完成端口。实验首例可复用有限来源，正式交付逐项消费 1093 所需的正式语言切片。公共 FFI 语法与资格核验归 1093，本包负责 Rust 工程、分发和生产入口的接线。

## 2. 架构方案

普通工程源与锁定依赖 → 1078 普通源码入口及共享 Checked HIR/MIR/ABC → 967 的 importer、生成 shim、ABI 准入与 `ForeignSession` → 唯一 AV → 已声明的 Rust provider。用 978 `M2Build`、`ac-build` 处理构建、SDK 与重定位。Auto 业务在 AV 执行，Rust 仅执行显式导入的 provider。

## 3. 技术栈

`crates/auto-bindgen/src/rust_import.rs`、`experimental/ace/crates/abi/src/rust_provider.rs`、`experimental/ace/crates/av-core/src/foreign_exec.rs`、`experimental/ace/crates/m2-build`、AutoDown。

## 4. 需求分析与背景调查

[967 规范](../specs/ace-migration/m2-host.md)记录 UTF-8 及资源/帧/ABI 已验边界。[1043](../specs/stdlib/design/context-source-assembly.md)和 [f0_load1043](../../experimental/ace/crates/abi/tests/cases/f0_load1043.rs) 证明有限来源装配。[978](archive/978-ace-m2-build.md)有五目标构建/SDK 基础。[1093](1093-av-parity-foreign-language-surface.md)的公共 Rust/C 用例和回调接面仍待其正式切片。

本轮取材基线为主检出 `4438d4933baa09ee4c6727111e2de07ff6cabfa0` 及活动 Plan 工作区文本。用户授权检查 AV 路线并规划 Rust 路线，顺序为 AV/RS（含 `*.rs.at`）→ A2R 替换 v0.5。本次授权仅含语言仓路线和计划文档。实施前按 AGENTS.md 确认计划与实际依赖版本。

2026-11-20 用户通过 `/goal 完成计划1113` 授权本合同范围内的执行、独立复审、有界修复与合入归档。实施基面 `auto-lang@c6544ee5ebf0b400860676bb09289b8e4be8ee44`，同组只读依赖 `auto-down@72b097006f34db3a871449f10e213820d51abef6`。

### T-01 接口对照（Windows/MSVC）

| 合同 | 已交输入与现有观测点 | 本计划接线与上游交接 |
|---|---|---|
| AC-01 / V-01，借用 UTF-8 | 967 `f0_text.rs` 的注解经 `auto_bindgen::rust_import` 生成 C-compatible shim。窄先例 `open_project_session` 从工程目录取 provider，经 `compile_text_record_product`、owned MIR/ABC、`verify_abc_foreign` 进入唯一 AV。`rust_f0` 有 9/0、直接 ABI oracle、真实 callee 计数、缺文件与坏签名、同入口恢复。 | 1078 `Auto` grammar 已识别 `use.rs`，但 `compile_language_source` 未注入 Rust `AdmittedForeign`，`language_source_worker::run_chain` 走无 foreign table 的 ABC 准入和 `execute_sched`。1093 r3 T-02 交普通 `Auto` 的声明、签名/授权与受检调用闭包。1113 用 `f0_text` 做窄试点，正式用同一 Auto 源证明 provider 源码改动后结果和产物身份同时改变。 |
| AC-02 / V-02，ABI 与生命周期 | 967 `f0_rich` 有借用 UTF-8/i32 混合参数、bool、fallible；`String`/ `Vec<i32>`/ `Result`/泛型受原 importer 拒绝。可写输出与资源柄沿 `m2-host.md` 的隔离帧和原 owner，不能从这些拒绝推断 owned 资源均拒绝。`ForeignSession` 管表代次与撤销，`rust_f0` 有 panic/恢复/拒绝计数；`f0_async` 的 Rust 线程经宿主 trampoline 回送，`f0_t13` 有 port 代次及 COFF callback 限定。 | 1093 r3 T-02/T-03 交普通工程的资源/闭包回调及同步、异步完成端口的受检表示和 lowering/dispatch。旧 host trampoline 仅证明旧端口机制。1113 的 V-02 要在实际工程观察同 owner 析构恰一次、撤权或跨代拒绝零 dispatch、取消后的迟到事件无第二次提交及 panic 恢复。 |
| AC-03 / V-03，构建和 SDK | 1043 `open_assembled_rs_provider_session` 消费已选 `.rs.at`，按注解 `dep` 选构建器。`f0_t14` 的 `BuildIdentity` 区分 provider、生成 shim 和工具身份。978 `ProjectBuildIdentity`、`ProviderSelection` 与 SDK 验包可复用。 | 978 当前 `lane_av` 只产 AV 观测收据，`run_sdk` AV 分支读 SDK 中 Auto 源并运行普通 Driver。其通用 provider 文件选择未进入 AV 绑定/调度。1113 接入 1093 受检产物与 967 会话时才能绑定 SDK 的 provider/ABI/授权，实际读回已封装旧产物复跑。 |

试点采用冻结 `experimental/ace/crates/abi/rustside/f0_text.rs`（SHA-256 `e21cee0317080a401bfd97031add2791be13975e8dfc6f2ae7569979f365ce2a`），独立区分 UTF-8 字节长度与 C 字符串终止长度，并复用 967 同一源的精确 +1 provider 变更。正式公共 Case 沿 1093 的特性作者位置与 1078 的 suite；本轮对照未改现有 Case 或引入第二入口。

## 5. 详细设计

沿用导入器权威，由注解/受检依赖生成声明和 shim。绑定阶段核目标 ABI、签名、pin、版本与授权，调用时核会话中的 owner、代次与许可。授权和句柄由运行会话持有；新进程重验，关闭撤销并交原 provider 析构。保留 967 对借用、owned、长度文本、可写输出、回调、完成/取消和 panic 的适用约束。暂缺的合法组合按 1093 或本包实际归属具名挂账，Rust ABI 边界以 C-compatible shim 为准。

复用 978 身份轴，将编译工具身份、运行兼容和授权分开。改 Auto body 保命中不变的 provider；改实际依赖内容重新产出并显式选入；重新打开 SDK 在任何 dispatch 前核工件与声明。允许已有批准的显式加载形式，正式分发验证所支持形式各自的真实闭包。最小试点沿 1078/967/978 正式基础先做，依赖语言宽面的工作按 1093 的所需切片接续。

T-02 由现有 `use.rs` + `f0_text` 先验证普通项目入口。T-04 用既有 `ac-build build --project <dir> --target av --root <dir>`、`ac-build sdk`、`ac-build run --sdk <dir> --target av` 入口复验分层边界。闭包里的工件和授权由实际工程声明供给，使用现有参数与准入机制承载。

### 规范增量

| ID | 操作 | 规范目标 | 交付后规则与理由 | 验收 |
|---|---|---|---|---|
| SD-01 | 修改 | `docs/specs/ace-migration/m2-host.md` | 补已验的普通 AV Rust 工程入口与会话状态 | AC-01/02 |
| SD-02 | 修改 | `docs/specs/ace-migration/m2-build.md` | 补 AV/RS 构建与 SDK 的输入身份和运行约束 | AC-03 |

## 6. 测试设计

以下为拟执行验证。各任务先定位现有同义 Case 或实现正式入口和必要反例，再记录准确命令、独立预期、实际阶段和产物身份。公共语料采用 1078 的唯一特性源。平台及条件执行范围见 [Design 60](../design/60-ace-rust-route.md)。

| Case | 验收 / work 任务 | 真实入口和区别性断言 |
|---|---|---|
| V-01 | AC-01 / T-02 | 普通 Auto 项目调用真实 Rust `f0_text`，`A中😀\0` 长度为 9、空串为 0；改 provider 返回规则后同一 Auto 源观察到确切新值与新工件身份。 |
| V-02 | AC-02 / T-03 | 普通 AV 工程真实 Rust owned/回调/完成端口覆盖：合法会话独立核 trace 与析构恰一次；撤权/错代次调用进入 Rust 的次数保持 0；取消后迟到完成不重复提交，panic 返回受控错误且会话恢复按合同可复现。 |
| V-03 | AC-03 / T-04 | `ac-build` 通过普通工程构建，移动 SDK、屏蔽开发树与编译工具后真实运行；篡改 ABI/工件/缺依赖具名失败且旧可用根可恢复，验证首先读回旧工件再执行。 |

`cargo t ace-abi ace-av ace-m2-build case-av-language-parity`；若改共享语法、绑定或 IR，加受影响的 `ace-source`、`ace-semantics`、`ace-lowering`，若改 FFI 公共提取器，补其 `auto-bindgen` 固定组。 完整受影响固定组合并入最终复审后的冻结验证，记录应跑、实跑、原条件排除和未跑数，以及准备/执行/清理时间。独立 review 查实际源码、Case 和真实生产入口。

## 7. 验收标准

- [ ] **AC-01** · owner_stage=`work`。正式接入普通工程和真实 Rust shim，V-01 的返回值、来源与缓存身份均可独立核对。
- [ ] **AC-02** · owner_stage=`work`。适用 ABI/资源/回调行完成，V-02 的成功、拒绝、关闭与恢复有真实调用观测。
- [ ] **AC-03** · owner_stage=`work`。V-03 从仓外预编译 SDK 真跑并验证构建缓存与失败恢复；导出可检查的 AV/RS 接面。
- [ ] **AC-04** · owner_stage=`review`。逐项独立复验 AC-01–03、缺口归属及 Spec 增量，结论和证据绑定合同版本及提交。
- [ ] **AC-05** · owner_stage=`merge`。冻结门通过后落主、沉淀已审规范与接口、归档及清理；核发布给实际消费者的来源身份。

### 验证记录 · T-02 窄入口试点

Windows/MSVC，`rustc 1.98.0 (88d9e12ae)`，基面 `c6544ee5e`。在专属树 ACE workspace 以 `cargo test --locked -j 2 -p ace-abi --test conformance --no-run` 准备 test-profile 可执行文件，再逐项以 `--exact <完整 case ID> --test-threads=1 --nocapture` 运行既有显式构建/集成用例。首次准备 0.096 s、exit 101（环境的 sccache 禁用 incremental），仅此命令关闭 wrapper 后重试通过，122.744 s、exit 0。相同基面有既有编译告警，日志留存。可执行文件 SHA-256 `0ed11e6b6169af411b74e8b677f3b3b482ab105da2cf135fb410b5870ba6837b`。

| 现有 case | 实际观测 | 命令总墙钟 |
|---|---|---:|
| `rust_f0::f0_av_calls_real_rust_provider_end_to_end` | 1/1，UTF-8 含 NUL 得 9，空串得 0，直接 ABI oracle 得 9，callee=3、拒绝=0 | 0.532 s |
| `rust_f0::f0_project_import_runs_rejects_and_recovers` | 1/1，工程文件驱动 9，坏签名和缺 provider 具名拒绝，恢复原文件后同入口再次得 9 | 0.491 s |
| `rust_f0::f0_provider_identity_rebinds_and_auto_body_edits_do_not` | 1/1，Auto body 改动保持声明身份，同一 `abc` 源在 Rust body +1 后由 3 改为 4，provider digest 改变且签名稳定 | 0.773 s |
| `f0_t14::t14_precise_invalidation_and_reuse` | 1/1，原键命中、provider/shim/工具输入变化失效，删除产物不命中、恢复后命中 | 0.785 s |

实跑 4/4，0 忽略，4 条 exact 命令总墙钟 2.581 s（既有 libtest 报告执行窗合计 2.43 s，含用例内部 provider 构建，发现及清理未单独计时）。同二进制其余 212 个唯一 case 未选中。此轮无新增入库用例，无固定组或 Linux/WSL 运行。Windows CRLF 使试点包装初报 `one_case=False`、退出 1，从原始日志按 CRLF 重核每条确有 1 个通过，四个被测进程均退出 0。复核摘要 `D:/autostack/plan-artifacts/plan-1113/abi-pilot-recount.results` SHA-256 `e4c395b11c3af7687baacbd95a5a0cc1acadcd3c04f301a33fbb41478b8e1a7`。证据范围为 967 窄入口，T-02 和 AC-01 仍待正式接线。

### 验证记录 · 1078 普通入口对照

同树以 `cargo build --locked -j 2 -p ace-av --bin av-worker` 准备 dev worker（exit 0，45.319 s，SHA-256 `93e37040c0ed66fbdf42cf7170b42f044bffae473233d30b5067c2b7d479d4d5`）。用 `av-worker case-serve` 向 `ac-av/language-source-run-v1` 送真实 Auto 文本，`entry=d_main`，cap `ac.source.language.av/v1`。

| 诊断探针 | 实际观测 | 首次 / 重放墙钟 |
|---|---|---:|
| 普通入口常量正控 | `fn main() int { return 9 }` 产 value 9 | 0.132 / 0.046 s |
| `use.rs f0_text`，调用 `text_byte_len("A中😀\0")` | 传输 exit 0，子项 `outcome=diag`，`stage=bind`，`code=source.undefined`，不存在 `text_byte_len`。当前 request 没有可传入工程/锁定 provider 的字段，源码核实 resolver 无 admitted Rust 环境，未到 ABC/执行 | 0.033 / 0.035 s |

诊断探针实跑 2/2 各两次，0 排除；首次包装以 `resolve` 匹配诊断阶段而退出 1，改为检查实际 `bind/source.undefined` 后复跑符合预期（包装 exit 0，worker 仍拒绝 Rust 导入）。探针执行窗含进程启动和 JSON 响应，发现/清理未单独计时；无新增长期 Case。复跑摘要 `D:/autostack/plan-artifacts/plan-1113/av-worker-pilot-replay.results` SHA-256 `173e79589444c4322673779d7636cf693d82174d64e02b783903acfba7687e4e`。并行构建占用检查见同 scope 的 `pilot-occupancy.log`，仅执行了 `-j 2` 定向准备；固定组待冻结候选。

## 8. 执行步骤

实施入口为 `D:/autostack/.wt/lang-1113/auto-lang`、分支 `plan-1113-dev`，基于实施时主检出的正式输入。按需在当前工具中登记 T 项再开工；首任务 T-01 的输入和待决项见 §4、§5、§10。材料统一放 `D:/autostack/plan-artifacts/plan-1113/` 并登记。

- [x] T-01 建立 967 与 1093 接口对照
  - owner_stage=`work`。核 967/1043/978 正式实例、1093 公共语法各所需接口和现有测试；标明 Rust types、关键版本、正反例与交接子面，选定普通入口的合法试点（AC-01）。
  - §4 已完成生产路径与观测点审计，基面 `c6544ee5e`。试点结果归 T-02，正式 AC 未关闭。
- [ ] T-02 接通普通 AV Rust 工程
  - owner_stage=`work`。在 AV runner 与 978 复用 importer/binder/shim；V-01 走真实 `use.rs` 工程、记录验前与验后 dispatch（AC-01）。
- [ ] T-03 接入资源与回调生命周期
  - owner_stage=`work`。消费 1093 正式资源/回调切片，绑定 V-02 到受检普通工程，区分哪些老窄面可复用以及必须新增的组合（AC-02）。
- [ ] T-04 交付仓外 SDK
  - owner_stage=`work`。接 978 SDK、授权、缓存/恢复；V-03 验工具缺席和错工件反例（AC-03）。
- [ ] T-05 冻结 AV/RS 工程接口
  - owner_stage=`work`。核完整组与所需公开 Case，记录 ISA/ABI、示例、源与 artifact 身份，准备 `IF-ACE-AV-RS-PROJECT/1`（AC-01–03）。
- [ ] T-06 独立复审
  - owner_stage=`review`。核 AC-01–04，回放关键正反例、实际入口和结果，记录遗漏、延期及替代实现，绑定 revision 和提交。
- [ ] T-07 验证冻结交付与归档
  - owner_stage=`merge`。执行冻结受影响组，核 AC-05，落主并交接，折叠证据后按 guard 清理并完成路径对账。

## 9. 复审记录

2026-11-20，stage=new，PLAN-1113 r1。计划已起草，实施确认与所需正式接口核对待办。outcome=blocked（work 尚未授权），next=确认后进入 T-01。T-01–07、AC-01–05 均待执行。

- 2026-11-20，`stage: work | plan_id: PLAN-1113 | plan_revision: r1 | outcome: blocked | worktree_path: D:/autostack/.wt/lang-1113/auto-lang | branch: plan-1113-dev | base_commit: c6544ee5ebf0b400860676bb09289b8e4be8ee44 | code_commit: c6544ee5ebf0b400860676bb09289b8e4be8ee44 | dependency_revisions: auto-down@72b097006f34db3a871449f10e213820d51abef6 | task_ids: T-01 complete; T-02 pilot only | evidence: §4/§7 | blockers: §10 | next_responsible_role: PLAN-1093 producer, then PLAN-1113 work`。两棵树干净。编译输出在专属树 `experimental/ace/target/`，日志、探针及 TEMP/TMP 均在已登记 scope。无交付提交或 Spec 沉淀。
- `stage: goal | plan_id: PLAN-1113 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1113/auto-lang | commit: c6544ee5ebf0b400860676bb09289b8e4be8ee44 | next: user`。用户的 `/goal 完成计划1113` 授权本计划完整 work→review→merge；授权与前置输入对照见 §4。依赖缺口见 §10。
- `stage: goal | plan_id: PLAN-1113 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-1113/auto-lang | commit: c6544ee5ebf0b400860676bb09289b8e4be8ee44 | next: user`。B-1113-01 连续第 2 轮。主检出 `56bccc41a` 时 1093 仍为 r3 drafting 0/8，无 1093 实施分支或已落主切片，§10 无新裁定。1113 及依赖树干净，停在 T-02，未启动新构建或测试。

## 10. 待澄清事项

T-01 核 1093 哪些行已能正式消费，确认待交切片及 owner。若现有接面无法表达 required 回调/资源组合，由相应生产者补出，保留该行验收。

- **B-1113-01，外部前置，owner PLAN-1093。** 主检出 PLAN-1093 r3 为 `drafting`、0/8，暂无已审落主的普通工程 Rust 切片。当前代码及真实普通入口对照见 §4/§7。解锁条件是 PLAN-1093 T-02 交付可消费的 `Auto` 受检 Rust 绑定/调用、授权与源/产物版本，T-03 交付适用资源、回调与完成端口的普通 AV 受检路径。先收到 T-02 可恢复 T-02；T-03 完成后才能闭合 V-02。每个交接须含落主 revision、源码/ABI/授权身份和对应实际 Case。交接后在同一专属树更新正式输入，接续 T-02–05。T-04 与 V-03 绑定实际工程产物闭包，T-05 冻结接口依赖前述真实结果；T-06/07 尚未进入。PLAN-1113 授权范围按现合同消费 1093，若要改由 1113 实现公共接面，须先裁定生产者和计划范围。
