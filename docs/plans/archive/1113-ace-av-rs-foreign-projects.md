---
plan_id: PLAN-1113
status: archived
completion_kind: delivered
feature_name: "ACE AV/RS 普通工程与 Rust FFI"
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-21
plan_revision: 2
current_step: 7
total_steps: 7
depends_on: [PLAN-967, PLAN-978, PLAN-1043, PLAN-1078, PLAN-1093]
supersedes_spec_components: [docs/specs/ace-migration/m2-host.md, docs/specs/ace-migration/m2-build.md]
new_spec_components: []
touched_goals: [GOAL-002, GOAL-003]
affects: [auto-av, auto-abi, ace-m2-build]
---

# [PLAN-1113] ACE AV/RS 普通工程与 Rust FFI

## 0. 变更摘要

Rust 路线第一批。交 `IF-ACE-AV-RS-PROJECT/1`，供 1114 用同一个普通工程入口装配标准库。保留 967、1043 与 1093 已验切片的真实范围与身份。 各批依赖和交接见 [Design 60](../../design/60-ace-rust-route.md)。

2026-11-21 r2：用户选择“授权在 1113 补齐”。T-03 接管普通 AV 为 V-02 缺少的公共 Rust owned 资源、回调和完成端口接口。V-01–03 与 AC-01–05 的原验收、仓集合、已完成任务和历史证据保留。

## 1. 目标

普通 Auto 工程通过受检 `use.rs`/`dep` 调用真实 Rust 库，并能从预编译工件运行。覆盖已在 967、1093 承诺的 Rust 类型、资源、回调和完成端口。1093 已交的公共接面为本包输入，V-02 所需的 Rust 公共接面缺口由 1113 补齐。本包负责 Rust 工程、分发和生产入口的接线。

## 2. 架构方案

普通工程源与锁定依赖 → 1078 普通源码入口及共享 Checked HIR/MIR/ABC → 967 的 importer、生成 shim、ABI 准入与 `ForeignSession` → 唯一 AV → 已声明的 Rust provider。用 978 `M2Build`、`ac-build` 处理构建、SDK 与重定位。Auto 业务在 AV 执行，Rust 仅执行显式导入的 provider。

## 3. 技术栈

`crates/auto-bindgen/src/rust_import.rs`、`experimental/ace/crates/abi/src/rust_provider.rs`、`experimental/ace/crates/av-core/src/foreign_exec.rs`、`experimental/ace/crates/m2-build`、AutoDown。

## 4. 需求分析与背景调查

[967 规范](../../specs/ace-migration/m2-host.md)记录 UTF-8 及资源/帧/ABI 已验边界。[1043](../../specs/stdlib/design/context-source-assembly.md)和 [f0_load1043](../../../experimental/ace/crates/abi/tests/cases/f0_load1043.rs) 证明有限来源装配。[978](978-ace-m2-build.md)有五目标构建/SDK 基础。[1093](1093-av-parity-foreign-language-surface.md)已交普通 AV 的预构建 Rust/C provider 路径，消费范围见 §10。

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

### T-03 r2 公共生命周期接口

| V-02 义务 | 生产与执行接面 | 关闭条件 |
|---|---|---|
| Owned | Rust 注解和真实类型 → 967 importer/shim → 939 受检资源声明 → 当前 `ForeignSession` 的 provider/代次/析构与 DLL 寿命。使用既有资源形状，运行期资源不进入 ABC 或 SDK。 | 普通项目的真实 Rust provider 记录显式关闭及会话错误/关闭触发的最后一次析构；双关闭、撤权、跨会话和旧代零派发。 |
| 回调 | 939 的受检回调签名和 noescape 规则 → shared lowering → ABC → AV 调用借用的回调目标，保留普通 Auto 函数值和闭包的约束。 | 独立核 Auto 回调、Rust 调用顺序及返回值；错误签名/授权或超期不可进入 provider，受控 trap/panic 后会话可用。 |
| 完成端口 | 967 `CompletionPorts` 与 `ForeignSession` 同代；close/rebind/cancel 与提交串行判定。Rust 线程/回调持有的可调用地址和数据保持到生产方停用，之后才能释放 DLL 和回调状态。 | 同一个注册表观察真实同步/异步路径、合法结果唯一提交，以及取消/关闭后的迟到丢弃与一次 release；从工程和仓外 SDK 运行 V-02。 |

实施输入为 `plan-1113-dev@9a4a43f6e`。先修完成端口状态和会话所有权，再增加受检 Rust 形状与 AV 执行。当前 `cancel` 重入会重复发布 Released、旧代端口留在注册表且 close 时无释放事件，新增状态反例以原 967 的完成规则验证。生命周期依赖已交 C ABI/资源/端口合同；工程的公开形状、期望和独立 trace 随实际实现逐项冻结。

### 规范增量

| ID | 操作 | 规范目标 | 交付后规则与理由 | 验收 |
|---|---|---|---|---|
| SD-01 | 修改 | `docs/specs/ace-migration/m2-host.md` | 补已验的普通 AV Rust 工程入口与会话状态 | AC-01/02 |
| SD-02 | 修改 | `docs/specs/ace-migration/m2-build.md` | 补 AV/RS 构建与 SDK 的输入身份和运行约束 | AC-03 |

## 6. 测试设计

以下为拟执行验证。各任务先定位现有同义 Case 或实现正式入口和必要反例，再记录准确命令、独立预期、实际阶段和产物身份。公共语料采用 1078 的唯一特性源。平台及条件执行范围见 [Design 60](../../design/60-ace-rust-route.md)。

| Case | 验收 / work 任务 | 真实入口和区别性断言 |
|---|---|---|
| V-01 | AC-01 / T-02 | 普通 Auto 项目调用真实 Rust `f0_text`，`A中😀\0` 长度为 9、空串为 0；改 provider 返回规则后同一 Auto 源观察到确切新值与新工件身份。 |
| V-02 | AC-02 / T-03 | 普通 AV 工程真实 Rust owned/回调/完成端口覆盖：合法会话独立核 trace 与析构恰一次；撤权/错代次调用进入 Rust 的次数保持 0；取消后迟到完成不重复提交，panic 返回受控错误且会话恢复按合同可复现。 |
| V-03 | AC-03 / T-04 | `ac-build` 通过普通工程构建，移动 SDK、屏蔽开发树与编译工具后真实运行；篡改 ABI/工件/缺依赖具名失败且旧可用根可恢复，验证首先读回旧工件再执行。 |

`cargo t ace-abi ace-av ace-m2-build case-av-language-parity`；若改共享语法、绑定或 IR，加受影响的 `ace-source`、`ace-semantics`、`ace-lowering`，若改 FFI 公共提取器，补其 `auto-bindgen` 固定组。 完整受影响固定组合并入最终复审后的冻结验证，记录应跑、实跑、原条件排除和未跑数，以及准备/执行/清理时间。独立 review 查实际源码、Case 和真实生产入口。

## 7. 验收标准

- [x] **AC-01** · owner_stage=`work`。正式接入普通工程和真实 Rust shim，V-01 的返回值、来源与缓存身份均可独立核对。
- [x] **AC-02** · owner_stage=`work`。适用 ABI/资源/回调行完成，V-02 的成功、拒绝、关闭与恢复有真实调用观测。
- [x] **AC-03** · owner_stage=`work`。V-03 从仓外预编译 SDK 真跑并验证构建缓存与失败恢复；导出可检查的 AV/RS 接面。
- [x] **AC-04** · owner_stage=`review`。逐项独立复验 AC-01–03、缺口归属及 Spec 增量，结论和证据绑定合同版本及提交。
- [x] **AC-05** · owner_stage=`merge`。冻结门通过后落主、沉淀已审规范与接口、归档及清理；核发布给实际消费者的来源身份。

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

### r2 定向验证记录（Windows/MSVC）

| 范围 | 实际结果 | 准备 / 执行 |
|---|---|---|
| `ace-av-core` 定向两项 completion ports 测试，`557324321` | 2/2，通过。重复 cancel、close 后旧代端口的迟到丢弃与 release 恰一次；其余 70 个唯一用例未选。新增 `closing_foreign_session_releases_port_once_and_rejects_late_completion` 0.012 s。 | dev/test 准备 11.987 s，两条 exact 命令 0.071 / 0.012 s。 |
| `ace-abi` conformance 的 `f0_t13::t13_future_ports_bind_the_session_generation`、`f0_async::` | 1/1 + 2/2，通过。前者改用实际 `ForeignSession` 所持注册表，后两项仍从真实 Rust 线程进宿主回调。 | dev/test 准备 22.992 s，命令 0.137 / 0.443 s。 |
| `ac-build` owned resource 项目与仓外 SDK，`626d6201b` | 6/6，通过。真实 Rust `Box<Resource>` 生成 C shim，显式关闭、会话退出和 panic 各触发一次原方析构，双关闭具名拒绝；搬移 SDK、项目及构建根不可见后真实运行得 9，析构一次。 | 第一构建准备 24.561 s。合用 Rust 绑定器初次编译错用旧变量名（4.454 s，exit 101），修正后 `cargo build --locked -j 2 -p ace-m2-build --bin ac-build --bin av-worker -p ace-av` exit 0、13.397 s。六条 CLI 命令共 1.127 s，各 0.027–0.583 s。 |
| `av-worker case-serve` 使用预构建 owned provider | 1/1，通过。共享普通 Auto 入口载入 prepared descriptor、真实 DLL；得 9，原方析构一次。 | 复用前述准备，执行 0.145 s。 |
| V-01 旧 F0 回放，`626d6201b` | 8/8，通过，保留 9、空串、变体、锁及缓存结果。 | 复用前述准备，8 条命令共 2.373 s。 |

范围内 0 忽略。发现及清理未独立计时。日志在计划唯一暂存区，SHA-256：`r2-owned-shared.log` `0b38f6cff0247f4939b851968175575744b54ebbc6f1c78e64aad4336943fc69`；`r2-owned-worker.log` `6d7731025d0ce3db067633cbdf982713d7064a0989460f702f302e762db99592`；`r2-f0-replay.log` `26d872cf4389bb541ed4f91e0a66c2ca2170bd416f1573706f5eb52f678d3ab6`；`r2-ports-abi-result.log` `a07231ae021ef4793881f915fc5b83e4aae95850fc91a3fab0aa15be122be79`。V-02 仍需从项目跑 Auto 回调、Rust 同步/异步完成端口，端口宿主回归只证明宿主层行为。

### r2 冻结前复跑（Windows/MSVC）

代码 `391aa3b5348621c0cacc55fef46e12814eb2d0e4`，基面 `eb04da7b2806ea727ec7d8f09f3b709af476ce0d`。保留 1094 的 Python 错误 observer 与展开分支。基面最后一笔为 1094 归档文档，`ce8ba4fbc..d3faf6bc2` 对 `eb04da7b2..391aa3b53` 的五笔 patch 经 `git range-diff` 核对逐笔相等。

| 门或用例 | 原始执行结果 | 时间 |
|---|---|---|
| V-01，真实 `ac-build build --target av` | 8/8，0 排除。文本 9/0、provider 改版 10、锁定 itoa 11。缓存分层以及授权、缺锁、失配锁拒绝均通过。 | 各命令 0.009–0.764 s |
| V-02，真实 owned 项目和 SDK | 6/6，0 排除。显式 close、会话 drop 与 panic 清理恰一次，双关闭具名拒绝，仓外 SDK 返回 9。第二会话读第一会话的 token 被拒绝，callee delta=0。原会话仍读得 8 且最终析构一次。 | 各 CLI 0.025–0.781 s。跨会话与闭包恢复合计 0.058 s |
| V-02，捕获闭包与负控 | 4/4 CLI，0 排除。Rust 实际回调依序得到 13、15，工程与移位 SDK 均返回 28。错误签名拒绝，无新增 trace。再以真 DLL 触发 Auto 除零 trap，同一会话恢复 28。 | 各 CLI 0.025–0.714 s |
| V-02，同步/异步端口 | 工程和移位 SDK 返回 17，Rust witness 分别记录独立线程，端口值为 5、12。真实 provider 进入后台后等受控信号；先取消并关闭会话，再令 Rust 完成 12，精确观察 Released 一次和 LateDropped 一次。撤权、预取消、旧代零新增 dispatch。后台 panic 受控，同会话重新得 17。 | 3/3 CLI 各 0.082–0.776 s，2/2 诊断 0.063/0.052 s |
| V-03，真实 `ac-build sdk/run` | 12/12，0 排除。移动 SDK 后屏蔽开发树与工具仍可运行。坏 ABI、工件内容、缺 DLL、缺声明、错授权和失配 pin 被拒，重新核验后从旧 recipe 真跑。旧 recipe SHA-256 `299060e93387325d0f7bf6d2d1e0bcd3d1d670d7c62f523068841cff1d7738b3`。 | 各命令 0.011–0.699 s |
| 新增 `cancelled_rust_worker_keeps_its_image_and_drops_its_late_result_once` | 1/1，通过。已进入 worker 后取消，session Drop 时映像仍在，worker 返回后只见一个迟到事件。 | dev/test 准备 9.93 s，机内用例 0.01 s |
| 新增 `p1113_project_and_relocated_sdk_run_captured_callback_and_completion_ports` | 1/1，通过。build/pack/run 组合独立核捕获值和不同 Rust 线程，工程及删除开发缓存后的 SDK 均为 45。 | dev/test 准备 11.46 s，实测用例含打包和清理 2.85 s |

冻结 CLI 准备 `cargo build --locked -j 2 -p ace-m2-build --bin ac-build -p ace-av --bin av-worker` 56.83 s，exit 0。仓外探针链接 1.71/1.48 s，exit 0。首次单测编译缺新增中断枚举分支，修复后两条通过。初次端口探针入口误设为 `panic`，改回 `main` 后重放通过。发现和清理未单独计时。Windows 按裁定排除 Linux/WSL。

同一普通回调工程实例的 Auto 源身份 `main.at:public:73f481bdfef2e558`，Rust 源 `e7e70ec526195a796ce5a2e8cf468dd9d8ed481e6bb4834194fe6cc2cf7898e3`，shim `f956a3b904ce604d8ac3abcfcf892c628f31a77384a1a9abe1015ba1feda8952`，DLL SHA-256 `b139b78888aaba255b89e0204a27e8b70bd290baa39cf43060bbb4fed45b0908`。完整 Runtime/SDK ABI 和路径见 SD-01/SD-02。原始日志均在 `D:/autostack/plan-artifacts/plan-1113/`，按顺序为 `r2-frozen-v01.log` `d30e0e54298429d7082710566445424001f83ac8bc7418c9590b49af778ea6f4`，`r2-frozen-owned.log` `16a7c54291688b4091ae73c1c1f2ca943efee74ca68c6801e99ae7512067b77d`，`r2-frozen-callback.log` `b584ddac634aa057c6252d19159a0a9d150538c4e4c9b56b8f714f1c22b2d7c1`，`r2-frozen-completion.log` `93997e0964163048527d09e741417499c9ac2b1dfb234c294973fe68147d26cd`，`r2-frozen-extra.log` `386b73e0c4abb911710bc57eaa0c5c972e7bfbb26fbddb035ebb0ccb18684b26`，`r2-frozen-project-race.log` `a48a851f9a15865069e86ea8b3c89d171c5988f9e9095f20c27de4c81986a719`，`r2-frozen-sdk-race.log` `497d375bb0d6badc25a44e663b07da462bedfeb642388e0561b84117d3efde21`，`r2-frozen-v03.log` `25b30ac3dbb0b96ee7cd20ef593ff3c9e37d98a0037875f2c49a31aa17e117ee`。

## 8. 执行步骤

实施入口为 `D:/autostack/.wt/lang-1113/auto-lang`、分支 `plan-1113-dev`，基于实施时主检出的正式输入。按需在当前工具中登记 T 项再开工；首任务 T-01 的输入和待决项见 §4、§5、§10。材料统一放 `D:/autostack/plan-artifacts/plan-1113/` 并登记。

- [x] T-01 建立 967 与 1093 接口对照
  - owner_stage=`work`。核 967/1043/978 正式实例、1093 公共语法各所需接口和现有测试；标明 Rust types、关键版本、正反例与交接子面，选定普通入口的合法试点（AC-01）。
  - §4 已完成生产路径与观测点审计，基面 `c6544ee5e`。试点结果归 T-02，正式 AC 未关闭。
- [x] T-02 接通普通 AV Rust 工程
  - owner_stage=`work`。在 AV runner 与 978 复用 importer/binder/shim；V-01 走真实 `use.rs` 工程、记录验前与验后 dispatch（AC-01）。
  - 提交 `e516cf379`。复用 1093 受检编译、ABC 准入与 967 会话/生成 shim。实际项目以 `project.json` 的 `rust_provider.source` 相对路径、显式 foreign caps 和可选 `rust_provider.cargo_lock` 装配。Cargo 依赖按既有 provider 注解消费，带依赖的冷构建以 `--offline --locked` 执行。
- [x] T-03 接入资源与回调生命周期
  - owner_stage=`work`。消费 967/939/1093 的正式资源、回调和端口切片，在本计划补齐 §5 r2 的公开 Rust 声明、shim 和 AV 执行，将 V-02 绑定受检普通工程与 SDK（AC-02）。
  - `207d823a9`、`85f6376e4`、`391aa3b53` 补齐 owned、代次绑定、闭包与两种真实完成端口。工程和仓外 SDK 的正反例与会话恢复见 §7。
- [x] T-04 交付仓外 SDK
  - owner_stage=`work`。接 978 SDK、授权、缓存/恢复；V-03 验工具缺席和错工件反例（AC-03）。
  - `9a4a43f6e` 随 SDK 打包受检 ABC、provider 绑定描述及真实 DLL。V-03 实际结果见 §7。总接口身份与 AC-03 随 T-05 冻结。
- [x] T-05 冻结 AV/RS 工程接口
  - owner_stage=`work`。核完整组与所需公开 Case，记录 ISA/ABI、示例、源与 artifact 身份，准备 `IF-ACE-AV-RS-PROJECT/1`（AC-01–03）。
  - 在 SD-01/SD-02 冻结入口与运行合同。固定组安排为 `auto-bindgen ace-source ace-semantics ace-lowering ace-av ace-abi ace-m2-build case-av-language-parity`，共享受检 MIR 另覆 `ace-native` 消费者。最终门归 T-07。
- [x] T-06 独立复审
  - owner_stage=`review`。核 AC-01–04，回放关键正反例、实际入口和结果，记录遗漏、延期及替代实现，绑定 revision 和提交。
- [x] T-07 验证冻结交付与归档
  - owner_stage=`merge`。执行冻结受影响组，核 AC-05，落主并交接，折叠证据后按 guard 清理并完成路径对账。

## 9. 复审记录

- `stage: merge | plan_id: PLAN-1113 | plan_revision: r2 | outcome: pass | delivery_commit: cf38bfcd7178e9f4bc4c66f02fe215aa06cf1c3b | archive_commit: 37cec7007 | completion_kind: delivered | cleanup: verified`。交付 `IF-ACE-AV-RS-PROJECT/1`，AC-01–05、T-01–07 全闭合。规范与仓外冒烟按下面的记录验证。
- `PLAN-1113:r2 | cleaned | holds: none`。逐项对账了项目和 SDK 输出根、移位副本、冻结构建和执行日志、C provider 源副本、库和探针、临时目录及两棵 worktree 的 Cargo 输出。计划材料均在 `D:/autostack/plan-artifacts/plan-1113/` 或专属 worktree 内。专属 auto-lang 树（109803 个文件，31,464,730,423 字节）和只读依赖 auto-down 树（1294 个文件，26,841,766 字节）均 clean 且通过 PowerShell reparse 扫描，已移除。验证空组目录、`plan-1113-dev` 分支及两个 Git worktree 登记均已移除。证据折叠至本计划后，已删除登记过的完整材料目录（11050 个文件，1,917,445,228 字节），核路径已不存在，登记为合入清理完毕，无持有。清理分步执行，整体耗时未单独计时。
- 周期批量回归读取 1109 的收据，关闭时 10.14 小时，无新增命中整五触发的计划，当前未到期。

- `PLAN-1113:r2 | archived | completion_kind: delivered | archive: docs/plans/archive/1113-ace-av-rs-foreign-projects.md | cleanup: pending`。实际归档路径、T-01–07 唯一计数、19 条本地链接、1093 前置、落地主提交和两份规范均已核对。T-07 的工作树及证据清理由下一收尾检查完成。

### T-07 冻结固定组

`PLAN-1113:r2 | prepared | reviewed/delivery_commit: cf38bfcd7178e9f4bc4c66f02fe215aa06cf1c3b | source_state: 2787a095516af8e0aa434052ed136935d9d0555332327e36009e9233e5687f47`。基面 `eb04da7b2`，同组依赖 `auto-down@72b097006f34db3a871449f10e213820d51abef6`。固定组配置 SHA-256 `69165441f188d5dde0a17a1291a0a5e2b0748f778104a7e2b027632a76611fbf`，nextest full 配置 `6c56945a398a5fd575c9ee9df2ca3567ae2d26419a901ccbd7f1d40f5f7c815a`。沿用复审冻结的 SD-01/02 内容，根锁 SHA-256 `6e49cacd70ac2df80a30514b628e75dae93ab98bc600d2724458535d975d1a07`。

| 固定组 | 实跑 | 首跑通过 / 未通过 | nextest 跳过或 Case SKIP | 发现 / 运行时间 |
|---|---:|---:|---:|---|
| auto-bindgen | 10 | 10 / 0 | 0 | 0.31 / 0.30 s |
| ace-source | 977 | 972 / 5 | 1 | 0.48 / 10.33 s |
| ace-semantics | 295 | 295 / 0 | 10 | 2.84 / 7.53 s |
| ace-lowering | 9 | 9 / 0 | 0 | 0.27 / 0.39 s |
| ace-av | 1576 | 1562 / 14 | 23 | 1.73 / 48.19 s |
| ace-abi | 328 | 300 / 28 | 8 | 2.32 / 46.72 s |
| ace-m2-build | 40 | 40 / 0 | 0 | 2.46 / 7.01 s |
| ace-native | 330 | 325 / 5 | 19 | 1.59 / 43.78 s |
| case-av-language-parity | 522 | 522 / 0（补齐准备后） | 162 | 64.30 s 运行，清单发现未单计 |

原生八组显式 `cargo t <groups> --prepare-only --debug-tools --out <scope>` 首次从冷工具准备 830.42 s，锁变更后 30.22 s，按新 `prepared.json` 执行 178.69 s，exit 1，3565 实跑且无待跑项。3 个本计划新增 Rust 用例均通过，关闭会话端口 0.0145 s，取消后的 worker 租约及迟到结果 0.0589 s，工程和仓外 SDK 捕获回调/端口/错误线程字段 1.0277 s。两种语言/Rust selector 合用 `--debug-tools` 的试探在参数校验 exit 2，0 运行，随后分开配置。

AV 首跑的 `bstr_1064::bstr_1064_v01_large_64k_a2r` 用时 12.819 s。复用同一归档和工具配置，`--test-threads 1` 串行重放 1/1，通过 1.527 s，整命令 2.358 s。首次重放缺 nextest JSON 开关，exit 95，未执行，补 `NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1` 后运行。有效原生结果 3514 通过和 51 个基面失败。ace-source 5 条 `text_record_source`、ace-av 13 条、ace-abi 28 条 `source_foreign` 对照 1093/1094 已归档收据；ace-native 5 条对照 1055 的具名基面失败。首红和原有债务归属保留，未发现本包引入的新失败。

Case 首次执行 66.06 s 返回 520 PASS、162 SKIP、2 FAIL，`explicit-c-library` 和 `c-header-spelling-rejected` 均是 `foreign.prepared`。复核 C provider 准备器需要 `ACE_AV_C_PROVIDER_DIR`，在材料目录复制 1094 入库的 `parity1093_c.c/.h/_impl.c` 并独立复核 SHA。Case 重新准备 3.24 s，完整执行 69.82 s / exit 0，两条用例分别 0.944/0.975 s 且均通过。Windows 的 Linux ELF、Linux console、WSL 按裁定排除，归专用 Linux 环境。等待含 1049 的语法组及 PLAN-114 的 native audit，末轮实际等待 37.96 s，清理时间另记。

材料目录中原生准备清单 SHA-256 `f43b00ad66855e288eb2ca9c3cad273ee64850fc84a5d93a07a2acc25afa9e54`，原生结果 `f59a4360e074b1db993035dfc2d4f5ec406d414620c9c95d6c3c34feeb86e1c2`，串行重放 `e2266868eb4bf3853b2ac0591adba00597e5691d621ef7881ab20f40ffe6bb18`，Case 首跑结果 `72362a87bf77be99b83fab656a541463e831b7ef1558f16c49be5fe30d23ce19`，Case 最终结果 `f1235ed662f242cafa433346df18b91d772cb4b167005893c49e594b6c04e35e`。

- `PLAN-1113:r2 | landed | default: master | delivery_commit: cf38bfcd7178e9f4bc4c66f02fe215aa06cf1c3b | merge: ff-only`。从 `eb04da7b2` 快进，主检出确认 HEAD 与已审交付提交相同。合入前计划树 `wt-guard` 为 clean。
- `PLAN-1113:r2 | canonical_verified | SD-01/SD-02 | main@cf38bfcd7`。主检出的 `docs/specs/ace-migration/m2-host.md` SHA-256 `12c0e1bed1450d36886f74d1f0ebbeba04a11542000909ddeef43b1d8ae90665`，`docs/specs/ace-migration/m2-build.md` SHA-256 `7b20fed5a9456fc2c081776bd4415a58311769545e7a53fc9c82b08cc70464a0`，与独立复审确认的规范增量一致。主检出调用已冻结的 `ac-build run --sdk <relocated> --target av --grant <grant>`，屏蔽编译工具后，标量 SDK 返回 9（31 ms），异步 SDK 返回 17（30 ms）。首次漏传标量包的授权，具名拒绝且零派发，补传授权后通过。两份成功收据 SHA-256 `9825b26e936de1185d6b117c071bc37602590b850f46fa3db81e5023f2ea844e`、`a5fa36b2f2e88578e3e5f5ba1df0137e3dc3e8f856d4995e922867132faae72e`。


- `stage: review | review_scope: final | plan_id: PLAN-1113 | plan_revision: r2 | outcome: pass | reviewed_commit: cf38bfcd7178e9f4bc4c66f02fe215aa06cf1c3b | base_commit: eb04da7b2806ea727ec7d8f09f3b709af476ce0d | acceptance_results: AC-01..04 pass | pending_closeout_gates: T-07 | next: merge`。首次 debug/test 准备 830.42 s 后，执行入口在 0.67 s 以 exit 2 拒绝已变更的源码指纹。首轮执行 0 条 Rust 用例。归因是根 Cargo.lock 缺 `ace-abi → ace-av` 这条与 Cargo.toml 一致的边，nextest 解析时已自动补写。提交 `cf38bfcd7` 仅固化该边，无版本、工具或实验 workspace 锁变化；回查之前准备实际使用的锁内容相同。SDK、AV 及两份规范内容同前次复审，原 AC-01–04 证据继续有效。首轮语言 Case 只准备 16.06 s，因 PLAN-1049 的 `ace-aac-syntax` 构建占用，执行暂缓。新冻结提交重新生成准备清单并执行固定组。

- `stage: review | review_scope: final | reviewer: same session, distinct artifact/code audit | plan_id: PLAN-1113 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1113/auto-lang | branch: plan-1113-dev | reviewed_commit: 378427ec2445285e0b44d4cda328dfbcc5715b15 | base_commit: eb04da7b2806ea727ec7d8f09f3b709af476ce0d | dependency_revisions: auto-down@72b097006f34db3a871449f10e213820d51abef6; 1093@300d8e585 | acceptance_results: AC-01..04 pass | pending_closeout_gates: T-07 | findings: F-1113-R2-01 closed | next: merge`。
  - AC-01 核 project.json 授权 → 967 importer → 1093 AV 前门 → Rust 真派发及 9/0/10/11/cache。AC-02 查 noescape、闭包验证与原预算、ABI 边界 panic、单次端口提交和 worker 原映像租约。§7 的除零恢复、跨会话句柄、真实取消后关闭及迟到轨迹均通过。修复没有改变 AV/ABI 执行体，复用这些证据。
  - AC-03 复验 SDK 12/12、0 排除，各步 0.013–0.587 s，旧 recipe SHA-256 `34876f1e745e379cb82b0975524f324f365278aeacae742f401284a379b19ae6`。修复在同一兼容判断中核实含异步 provider 的 SDK 声明 `multi`，真跑 17，改单字段为 `single` 具名拒绝。同步/异步 mode 进入对应符号的兼容摘要，旧无完成端口的事实摘要沿用。新增组合用例再跑 1/1，dev/test 准备 58.03 s，用例 0.76 s；新增端口状态用例复用 0.01 s 结果。显式 `ace-m2-build` 工程构建验证不在默认日常组。发现/清理未单计。
  - SD-01 `docs/specs/ace-migration/m2-host.md` SHA-256 `12c0e1bed1450d36886f74d1f0ebbeBA04a11542000909ddeef43b1d8ae90665`；SD-02 `docs/specs/ace-migration/m2-build.md` SHA-256 `7b20fed5a9456fc2c081776bd4415a58311769545e7a53fc9c82b08cc70464a0`。前置与当前目标 ID 已核，未发现未批准延期或替代实现。计划记录保留历史缺口和既有已知红，待冻结门逐项对照。源树 clean、`git diff --check` 通过、无新增编译告警。
  - 本计划材料目录日志 `r2-sdk-threading-review.log` SHA-256 `bfd9bf00347fda32198240ad3ee4a9a25dd9d4421cb27cb40537fe1d05069d7f`，`r2-review-async-sdk.log` `84bb5a322186d8e30279190dc916d48d0058114263d77518e9f0bfb99583d75a`，`r2-review-scalar-v03.log` `fe4dc0bdff94419e5f6a4d3dc623ba35ab8e7aaea20a9fa2a385750d7b7a3f23`。

- `stage: review | review_scope: final | reviewer: same session, rebuilt from plan and artifacts | plan_id: PLAN-1113 | plan_revision: r2 | outcome: needs_fix | reviewed_commit: 391aa3b5348621c0cacc55fef46e12814eb2d0e4 | base_commit: eb04da7b2 | findings: F-1113-R2-01 | next: work`。AC-01/02 保留有效证据。AC-03/T-05：`rust_project::offer` 总写 `threading=single`，实际 V-02 中 Rust 前台和后台的线程 ID 不同。SDK 对外的线程兼容身份失实。修复保留原无异步 provider 的 `single`，以异步 SDK 实际运行和改单字段导致 `m2build.compat.threading` 的负控闭合。待重验 SD-02。

- `stage: work | plan_id: PLAN-1113 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1113/auto-lang | branch: plan-1113-dev | base_commit: eb04da7b2806ea727ec7d8f09f3b709af476ce0d | code_commit: 391aa3b5348621c0cacc55fef46e12814eb2d0e4 | dependency_revisions: auto-down@72b097006f34db3a871449f10e213820d51abef6; 1093@300d8e585 | task_ids: T-01..05 | evidence: §7 r2 冻结前复跑 | blockers: 无 | next_responsible_role: review`。

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
- `stage: goal | plan_id: PLAN-1113 | plan_revision: r1 | round: 6 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1113/auto-lang | commit: 9a4a43f6e | next: user`。B-1113-02 在恢复后的第 3 轮复现。主检出 `5c195e72e` 的 Rust importer、普通 AV 回调和端口接面无新交付，§10 无范围裁定。保留干净专属树与 3/7 进度，完整 V-02、冻结、复审及归档待 §10 解锁。本轮未启动构建或测试。
- 2026-11-21，`stage: new | plan_id: PLAN-1113 | plan_revision: r2 | outcome: pass | next: work T-03`。用户授权本计划补齐 V-02 所需的公共 Rust 资源、回调及完成端口，按 §5 r2 与 967/939/1093 既有契约实施。T-03 生产范围扩为本仓公共接面，AC-01–05 保留，历史 r1 签名按输入变化补足验证。专属树 `plan-1113-dev@9a4a43f6e`；同组依赖 `auto-down@72b097006f34db3a871449f10e213820d51abef6`。
- 2026-11-21 r2 实施检查点。 `557324321` 将端口代次与 session 绑定、cancel/close 释放改为幂等；`626d6201b` 增 `owned_open`/`owned_borrow`/`owned_close` 与 `owned=Type`，复用 939 的 Handle 声明和受检整数句柄。AV 以当前 provider、资源种类与代次验证 token，关闭调用原方，代码租约与表同寿命；`ac-build` 与普通 prepared worker 共用 Rust 导出绑定器。§7 记录实际工程和 SDK 验证。T-03 与 AC-02 保持待完成，next=work（普通 AV 回调及生产端口接线、V-02 全链）。宿主目标仍是前轮的 blocked 状态，只有客户端可重新激活自动轮次。

## 10. 待澄清事项

- **B-1113-02 已关闭。** 用户授权由 PLAN-1113 补齐 V-02 所需的公共接面。owned 资源、捕获闭包和同步/异步完成端口已从真实工程及仓外 SDK 验收，见 §7。

- **B-1113-01，1093 已交付可供 T-02 消费的切片。** r3 已归档，评审交付提交为 `300d8e585`。恢复时先将同一专属树快进到含该提交的主检出，复用 1093 的普通 AV 前门与预构建 provider 准入，接通实际工程声明和 978 的构建/分发。V-02 仍按原合同关闭。若 landed API 无法表达必要的资源或回调组合，按缺口确认生产者与授权。

### 1093 前置交付记录

表为 2026-11-20 按 `depends_on` 排出的交付顺序，括号记录当时待交前置。2026-11-21 核对表内计划均已归档。1078–1082、967、978、1043、1064 也已归档，按适用接口复用。

| 顺序 | 已归档计划（原待交前置） |
|---:|---|
| 1 | [1083 名义类型、枚举与模式](1083-av-parity-records-variants-and-patterns.md) |
| 2 | [1084 动态值与 null](1084-av-parity-dynamic-objects-and-null.md)（1083）、[1085 模块与全局初始化](1085-av-parity-modules-and-global-state.md)（1083） |
| 3 | [1086 借用、move 与资源值](1086-av-parity-borrows-moves-and-storage.md)（1083、1085） |
| 4 | [1087 错误与控制流展开](1087-av-parity-errors-and-unwinding.md)（1083、1084、1085、1086）、[1088 泛型、spec 与方法决议](1088-av-parity-generics-specs-and-methods.md)（1083、1085、1086） |
| 5 | [1089 生成器与迭代生命周期](1089-av-parity-generators-and-stream-lifetimes.md)（1083、1086、1087）、[1091 编译期执行与宏](1091-av-parity-comptime-and-macros.md)（1085、1088） |
| 6 | [1090 任务、异步块与 Actor](1090-av-parity-tasks-async-and-actors.md)（1083、1085、1086、1087、1089）、[1092 标注、元数据与反射](1092-av-parity-attributes-and-metadata.md)（1088、1091） |
| 7 | [1093 受检 FFI 与回调](1093-av-parity-foreign-language-surface.md)（1086、1087、1090、1092） |

### 1113 分步恢复条件

下表记录当时的恢复判断。T-02 至 T-05 的最终执行与验收见 §7–9。

| 1113 工作 | 本轮核对 | 恢复与验收 |
|---|---|---|
| T-02 / V-01 | **可接续。** 1093 的 `run_foreign_chain` 加载调用方提供的 prepared descriptor 与 DLL，核工件身份，经 `compile_language_source_with_foreign` 和 AV 真派发。§9 记录 UTF-8 9、空串 0、provider 变更 9→10 且 DLL SHA 改变、缺 cap 具名拒绝。 | 1113 接通普通工程的 `use.rs`/`dep` 与显式授权、构建和缓存身份，在实际工程重做 V-01。 |
| T-03 / V-02 | **仍需逐项核实。** 1093 证明了 panic 被 shim 限制在 C ABI 边界内，以及缺 cap、缺闭包等拒绝；新普通 AV 路径按请求加载 stub 表。1093 复审将 owned/callback/outbytes 资源链归 967 owner，现有公开 Case 未覆盖普通工程会话的完整回调和同步/异步完成端口。 | 沿 967 会话 owner 核实际工程的代次、关闭、析构恰一次、撤权零派发、取消后迟到完成及 panic 后恢复；具名记录需要上游补出的受检表示或执行接缝。 |

每次交接记录落主 revision、源码与产物/ABI/授权身份及实际 Case。1113 可按所需切片逐步恢复，T-04 的 SDK 验收仍以实际工程产物闭包和 V-03 为准，T-05 在 AC-01–03 具备实际结果后冻结接口。

- **B-1113-02 的 r1 观察与 r2 后续。** 原 1093 [复审收据](1093-av-parity-foreign-language-surface.md) 将 owned/callback/outbytes 资源链归回 967 原边界。r2 已接 owned 真实工程和 SDK，余下从 Rust 注解引入回调与端口声明，接通 [共享调用 lowering](../../../experimental/ace/crates/lowering/src/hir_adapter.rs) 和 [AV ForeignCall](../../../experimental/ace/crates/av-core/src/av.rs)。普通 Auto 的函数值使用闭包；旧 939 路径的 CallbackRef 是 named target。执行接线须延续既有 noescape 约束并允许受检 Auto 回调返回，外部 provider 的重入入口保持原拒绝。967 的 [宿主端口对照](../../../experimental/ace/crates/abi/tests/cases/f0_async.rs) 复用同一规则表，正式 V-02 还须从真实工程和 SDK 发起同步/异步完成、取消并观察真实迟到提交。
