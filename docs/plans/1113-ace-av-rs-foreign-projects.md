---
plan_id: PLAN-1113
status: executing
feature_name: "ACE AV/RS 普通工程与 Rust FFI"
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-21
plan_revision: 1
current_step: 3
total_steps: 7
depends_on: [PLAN-967, PLAN-978, PLAN-1043, PLAN-1078, PLAN-1093]
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

[967 规范](../specs/ace-migration/m2-host.md)记录 UTF-8 及资源/帧/ABI 已验边界。[1043](../specs/stdlib/design/context-source-assembly.md)和 [f0_load1043](../../experimental/ace/crates/abi/tests/cases/f0_load1043.rs) 证明有限来源装配。[978](archive/978-ace-m2-build.md)有五目标构建/SDK 基础。[1093](archive/1093-av-parity-foreign-language-surface.md)已交普通 AV 的预构建 Rust/C provider 路径，消费范围见 §10。

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

- [x] **AC-01** · owner_stage=`work`。正式接入普通工程和真实 Rust shim，V-01 的返回值、来源与缓存身份均可独立核对。
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

### 恢复 work 验证记录（Windows/MSVC）

| 实跑范围 | 实际结果 | 准备 / 执行 |
|---|---|---|
| V-01 官方 CLI，`ac-build build --project <dir> --target av --root <dir>` | 8/8、0 排除。9、0、9→10 和锁定 itoa 返回 11，同 Auto/provider 分层缓存，缺授权、缺锁、失配锁具名拒绝。两份 Auto 源来自 1093 的 canonical Case。 | 全量准备 44.753 s，后续变更增量 7.705 s。最终 8 条命令共 2.341 s（各 0.009–0.709 s）。 |
| V-03 CLI SDK 预编译和仓外搬移 | 12/12、0 排除。搬移后先 `verify`，隔离项目/构建根且工具不在 PATH，运行真实 ABC/DLL 得 9，工具派发计数空。错误授权、ABI、DLL 哈希、缺 DLL、未声明 PE 依赖、错 pin 分别拒绝。失败更新保留旧闭包，读回原 recipe 后再执行得 9。 | 与 V-01 共用增量编译。最终 12 条命令共 1.070 s（各 0.013–0.559 s）。 |
| V-02 部分诊断，用 V-03 同一 SDK 的真实 DLL/ABC | 同一个 967 会话执行得 9，拒权、预取消、关闭后旧代次零新增派发，受控 panic 后同一会话恢复得 9。真实 Rust panic hook 写 stderr，进程 exit 0。 | 枚举编译产物 0.387 s。首尝试少 Windows native link path，补用 Cargo 已记录的路径后探针编译 0.831 s。执行 0.134 s。 |
| 978 原回归，`ace-m2-build` 的 `sdk_pack` / `restore_recovery` 两测试目标 | 8/8 / 2/2，通过，0 忽略。 | `cargo test --locked -j 2 -p ace-m2-build --test sdk_pack --test restore_recovery --no-run` 50.913 s，两个准备好的目标串行分别 1.564 / 0.877 s。 |
| 967 原对照，`ace-abi` conformance 的 `rust_f0::f0_session_lifecycle_success_teardown_unauthorized_rebind` / `f0_async::` | 1/1 / 2/2 通过，0 忽略。前者观察真实会话重绑定，后者经宿主 trampoline 观察发布和迟到丢弃。其余 213 个唯一用例未选。 | `cargo test --locked -j 2 -p ace-abi --test conformance --no-run` 6.602 s，命令墙钟 1.082 / 0.451 s。 |

发现/清理未单独计时，CLI 时间含各命令的实际处理，固定组并集未运行。Linux/WSL 按裁定排除。本轮无入库新增测试。最终 CLI 与诊断日志位于 `D:/autostack/plan-artifacts/plan-1113/`。SHA-256 分别为 `v01-frozen.log`=`21974dd94b3c25ad32caf3ea20a8e6078bed8bbbb60cc70541394eb1dbb0eb39`，`v03-frozen.log`=`2164d959749b58f5ea4f77af9c31e9cc704906a764132b1f2415f39d4c547081`，`v02-partial.log`=`653b97ac0aafedae81f06651d52e262998962ff7c1dad889536bc33f27b6f276`。恢复验证前后原 recipe SHA-256 一致（`5485de6f5a1d9723637092b8a87335f6b0d62e5725a8c5bff2547704385b0b52`）。V-02 完整资源与回调链待 §10 裁定。

## 8. 执行步骤

实施入口为 `D:/autostack/.wt/lang-1113/auto-lang`、分支 `plan-1113-dev`，基于实施时主检出的正式输入。按需在当前工具中登记 T 项再开工；首任务 T-01 的输入和待决项见 §4、§5、§10。材料统一放 `D:/autostack/plan-artifacts/plan-1113/` 并登记。

- [x] T-01 建立 967 与 1093 接口对照
  - owner_stage=`work`。核 967/1043/978 正式实例、1093 公共语法各所需接口和现有测试；标明 Rust types、关键版本、正反例与交接子面，选定普通入口的合法试点（AC-01）。
  - §4 已完成生产路径与观测点审计，基面 `c6544ee5e`。试点结果归 T-02，正式 AC 未关闭。
- [x] T-02 接通普通 AV Rust 工程
  - owner_stage=`work`。在 AV runner 与 978 复用 importer/binder/shim；V-01 走真实 `use.rs` 工程、记录验前与验后 dispatch（AC-01）。
  - 提交 `e516cf379`。复用 1093 受检编译、ABC 准入与 967 会话/生成 shim。实际项目以 `project.json` 的 `rust_provider.source` 相对路径、显式 foreign caps 和可选 `rust_provider.cargo_lock` 装配。Cargo 依赖按既有 provider 注解消费，带依赖的冷构建以 `--offline --locked` 执行。
- [ ] T-03 接入资源与回调生命周期
  - owner_stage=`work`。消费 1093 正式资源/回调切片，绑定 V-02 到受检普通工程，区分哪些老窄面可复用以及必须新增的组合（AC-02）。
  - `9a4a43f6e` 实作可重用的执行前授权检查，已用真实工程工件验证同会话 panic 恢复和零派发负控。完整 V-02 仍待 B-1113-02。
- [x] T-04 交付仓外 SDK
  - owner_stage=`work`。接 978 SDK、授权、缓存/恢复；V-03 验工具缺席和错工件反例（AC-03）。
  - `9a4a43f6e` 随 SDK 打包受检 ABC、provider 绑定描述及真实 DLL。V-03 实际结果见 §7。总接口身份与 AC-03 随 T-05 冻结。
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
- `stage: goal | plan_id: PLAN-1113 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1113/auto-lang | commit: c6544ee5ebf0b400860676bb09289b8e4be8ee44 | next: user`。主检出 `ba6b079af` 复核 PLAN-1093 仍为 r3 drafting 0/8，未找到实施分支或正式交付；§10 无新裁定。B-1113-01 连续三轮，目标进入 blocked，计划和两棵干净 worktree 保留以待 §10 解锁。未启动新构建或测试。
- 2026-11-20，`stage: new | plan_id: PLAN-1113 | plan_revision: r1 | outcome: blocked | next: 接收 PLAN-1093 T-02 的 Rust 切片后恢复 T-02`。按用户要求将正文已有的 PLAN-1093 依赖同步到 `depends_on`，按各 Plan 前置关系在 §10 补交付顺序及恢复条件。r1 合同和阶段验收沿用原记录。
- 2026-11-21 依赖复核：`git merge-base --is-ancestor` 确认 1093 r3 的交付 `300d8e585` 和合入收据 `a2249c685` 均在 master `dd5f3eff5` 上。核当前源码、公开 Case 与 1093 §9 实跑收据，T-02 可接续，V-02 生命周期需逐项补证，详见 §10。1113 专属树仍为 `c6544ee5e`，本轮为静态复核。
- 2026-11-21 恢复 work：专属树 FF 至 `dd5f3eff56176b00cae2c5c8ce916cfed8c2f9f6`，同组 auto-down 保持 `72b097006f34db3a871449f10e213820d51abef6`。T-02 commit `e516cf379`。在 ACE workspace 编译 `cargo build --locked -j 2 -p ace-m2-build --bin ac-build`（44.753 s，exit 0）；真实仓外项目走 `ac-build build --project <dir> --target av --root <dir>`，输入读取 1093 的唯一公开 Auto 源。8 项集成检查全通过，0 排除。冷构建 9（0.883 s）、同输入工程命中（0.119 s）、空串 0 且复用同一 DLL SHA（0.181 s）、同一 Auto 源改 Rust 结果 10 且 DLL SHA 变化（0.745 s）、缺授权拒绝（0.011 s）、带依赖缺锁拒绝（0.011 s）、实际锁定 itoa 产 11 且记录锁 SHA（0.770 s）、失配锁拒绝（0.323 s）。运行/准备及清理由该显式集成入口承担，内部发现/清理未分计。摘要 `D:/autostack/plan-artifacts/plan-1113/v01-first.log` SHA-256 `b0af2ef9e64a9cb58ba6375cbd21e5a8eccf49c81e07b67964f642ae301720d6`。格式化后 `cargo check --locked -j 2 -p ace-m2-build --all-targets`（37.966 s，exit 0，既有依赖告警）；无新增入库测试。冻结固定组待后续复审/合入。
- 2026-11-21，`stage: work | plan_id: PLAN-1113 | plan_revision: r1 | outcome: blocked | worktree_path: D:/autostack/.wt/lang-1113/auto-lang | branch: plan-1113-dev | base_commit: dd5f3eff5 | code_commit: 9a4a43f6e | dependency_revisions: auto-down@72b097006f34db3a871449f10e213820d51abef6; 1093@300d8e585 | task_ids: T-02/T-04 complete; T-03 partial | evidence: §7 | blockers: B-1113-02 | next_responsible_role: 1113 work after producer/scope decision`。Spec 拟增量放实施树，最终接口待 T-05。工作树干净，未合入。
- `stage: goal | plan_id: PLAN-1113 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1113/auto-lang | commit: 9a4a43f6e | next: user`。恢复后首轮核到新阻塞 B-1113-02。前一阻塞的普通 Rust 入口切片已被消费。
- `stage: goal | plan_id: PLAN-1113 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-1113/auto-lang | commit: 9a4a43f6e | next: user`。B-1113-02 连续第 2 轮。主检出 `b73e4f0c8` 无公共 Rust ABI 或普通 AV 回调补齐提交，§10 无范围裁定。前轮的 T-02/T-04 已验证提交保留，T-03 仍未齐，本轮未启动构建或测试。

## 10. 待澄清事项

- **B-1113-01，1093 已交付可供 T-02 消费的切片。** r3 已归档，评审交付提交为 `300d8e585`。恢复时先将同一专属树快进到含该提交的主检出，复用 1093 的普通 AV 前门与预构建 provider 准入，接通实际工程声明和 978 的构建/分发。V-02 仍按原合同关闭。若 landed API 无法表达必要的资源或回调组合，按缺口确认生产者与授权。

### 1093 前置交付记录

表为 2026-11-20 按 `depends_on` 排出的交付顺序，括号记录当时待交前置。2026-11-21 核对表内计划均已归档。1078–1082、967、978、1043、1064 也已归档，按适用接口复用。

| 顺序 | 已归档计划（原待交前置） |
|---:|---|
| 1 | [1083 名义类型、枚举与模式](archive/1083-av-parity-records-variants-and-patterns.md) |
| 2 | [1084 动态值与 null](archive/1084-av-parity-dynamic-objects-and-null.md)（1083）、[1085 模块与全局初始化](archive/1085-av-parity-modules-and-global-state.md)（1083） |
| 3 | [1086 借用、move 与资源值](archive/1086-av-parity-borrows-moves-and-storage.md)（1083、1085） |
| 4 | [1087 错误与控制流展开](archive/1087-av-parity-errors-and-unwinding.md)（1083、1084、1085、1086）、[1088 泛型、spec 与方法决议](archive/1088-av-parity-generics-specs-and-methods.md)（1083、1085、1086） |
| 5 | [1089 生成器与迭代生命周期](archive/1089-av-parity-generators-and-stream-lifetimes.md)（1083、1086、1087）、[1091 编译期执行与宏](archive/1091-av-parity-comptime-and-macros.md)（1085、1088） |
| 6 | [1090 任务、异步块与 Actor](archive/1090-av-parity-tasks-async-and-actors.md)（1083、1085、1086、1087、1089）、[1092 标注、元数据与反射](archive/1092-av-parity-attributes-and-metadata.md)（1088、1091） |
| 7 | [1093 受检 FFI 与回调](archive/1093-av-parity-foreign-language-surface.md)（1086、1087、1090、1092） |

### 1113 分步恢复条件

| 1113 工作 | 本轮核对 | 恢复与验收 |
|---|---|---|
| T-02 / V-01 | **可接续。** 1093 的 `run_foreign_chain` 加载调用方提供的 prepared descriptor 与 DLL，核工件身份，经 `compile_language_source_with_foreign` 和 AV 真派发。§9 记录 UTF-8 9、空串 0、provider 变更 9→10 且 DLL SHA 改变、缺 cap 具名拒绝。 | 1113 接通普通工程的 `use.rs`/`dep` 与显式授权、构建和缓存身份，在实际工程重做 V-01。 |
| T-03 / V-02 | **仍需逐项核实。** 1093 证明了 panic 被 shim 限制在 C ABI 边界内，以及缺 cap、缺闭包等拒绝；新普通 AV 路径按请求加载 stub 表。1093 复审将 owned/callback/outbytes 资源链归 967 owner，现有公开 Case 未覆盖普通工程会话的完整回调和同步/异步完成端口。 | 沿 967 会话 owner 核实际工程的代次、关闭、析构恰一次、撤权零派发、取消后迟到完成及 panic 后恢复；具名记录需要上游补出的受检表示或执行接缝。 |

每次交接记录落主 revision、源码与产物/ABI/授权身份及实际 Case。1113 可按所需切片逐步恢复，T-04 的 SDK 验收仍以实际工程产物闭包和 V-03 为准，T-05 在 AC-01–03 具备实际结果后冻结接口。

- **B-1113-02，公共 Rust ABI 与普通 AV callback 缺口。** 已实现的工程持续使用 967 importer、会话与 1093 普通前门。当前 [Rust importer 的形状](../../crates/auto-bindgen/src/rust_import.rs) 仍只接入 ViewUtf8、OutBytes、I32、Bool 与 FrameCall 等已定义形状，owned provider 资源柄及回调/异步签名无法生成受检 Rust 声明与 shim。[AV ForeignCall 路径](../../experimental/ace/crates/av-core/src/av.rs) 对非空 callback lanes 明确拒绝 `foreign.callback-unsupported`。[967 完成端口对照](../../experimental/ace/crates/abi/tests/cases/f0_async.rs) 的两例借真实 DLL 与宿主 trampoline 运行，§7 已重放，通过的端口证据适用于宿主层。1093 [复审收据](archive/1093-av-parity-foreign-language-surface.md) 也把 owned/callback/outbytes 资源链归回 967 原边界。解锁需生产出普通 Rust 工程可表达的受检 owned 资源/析构、闭包回调和同步/异步端口 ABI 及 AV 执行接缝，由上游后续计划交付，或经用户明确授权将公共接口补齐移入 1113。然后在项目/SDK 的真实 provider 上验证析构恰一次、回调轨迹及迟到完成不重复提交；V-02 保持原验收。
