---
plan_id: PLAN-1116
status: archived
completion_kind: delivered
feature_name: "ACE A2R 运行时、外库与工程交付"
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-22
plan_revision: 1
current_step: 7
total_steps: 7
depends_on: [PLAN-1113, PLAN-1114, PLAN-1115, PLAN-978]
supersedes_spec_components: [docs/specs/ace-migration/m2-build.md, docs/specs/ace-migration/m2-host.md]
new_spec_components: [docs/specs/auto-a2r/project-runtime-and-distribution.md]
touched_goals: [GOAL-003]
affects: [auto-a2r, auto-a2x, auto-abi, ace-m2-build]
---

# [PLAN-1116] ACE A2R 运行时、外库与工程交付

## 0. 变更摘要

交 `IF-ACE-A2R-PROJECT/1` 给 A2R 用户命令、普通工程与领域消费者，延伸 1115 的真实 Rust 主链到仓外交付。 各批依赖和交接见 [Design 60](../design/60-ace-rust-route.md)。

## 1. 目标

普通多模块工程能 A2R 构建、在仓外编译运行并消费 Rust 依赖。接齐 1089/1090 调度与生命周期、1093 适用 foreign、1095 工程组合以及发布必需的 I/O、网络、存储运行时操作。目标直接执行生成的任务、业务和标准库适配，遵守公共错误、授权和资源合同。

## 2. 架构方案

`M2Build` 加载普通工程与 A2R 文件集，经受控 build-lane 驱动 rustc/Cargo 和真实 Rust 依赖。生成程序使用 Rust runtime/adapter。外库经已验 shim，或使用经验证等价的直接 crate adapter，共用 Checked HIR 声明和资源规则。SDK 恢复、运行准入复用 978。

## 3. 技术栈

`experimental/ace/crates/a2x-rust/src/{foreign_bridge.rs,sched_rt.txt}`、`experimental/ace/crates/a2x-worker`、`experimental/ace/crates/m2-build/src/{driver,recipe}.rs`、`ac-build`、Cargo。

## 4. 需求分析与背景调查

[978](archive/978-ace-m2-build.md)区分源码 emit 与预编译 exe。[967 规范](../specs/ace-migration/m2-host.md)记录 AXR0 与 AV ABI/生命周期。[Design 56](../design/56-ace-m2-ffi-and-native-stdlib.md)要求直接 adapter 独立证等价。现有 a2x-rust 有 foreign wrapper 和 scheduler runtime 载体，由 T-01 核真消费范围。

本轮取材基线为主检出 `4438d4933baa09ee4c6727111e2de07ff6cabfa0` 及活动 Plan 工作区文本。用户授权检查 AV 路线并规划 Rust 路线，顺序为 AV/RS（含 `*.rs.at`）→ A2R 替换 v0.5。本次授权仅含语言仓路线和计划文档。实施前按 AGENTS.md 确认计划与实际依赖版本。

## 5. 详细设计

按 1115 的已交切片推进。消费 1089/1090 正式事实落实 generator/stream、任务、Future/Actor 的适用模式。对同源 use.rs 与公共 Rust 标准库适配，复用 1113/1114 的签名、类型、限制、错误、owner 和逐行 oracle，独立验证 A2R 工件执行。公开 C 依赖按现行 A2R 支持合同接入，Python 专属协议沿原 owner。

仓外工程走 source → emit → build → run → SDK 重定位。编译用锁定的工具/依赖闭包，安装运行只使用声明依赖。manifest 区分可编辑 Cargo 导出与预编译运行，改业务、改依赖和缓存污染分别核重建与恢复。AutoMan 自有入口由其 owner 接收，图形交付走 1074、1103、1076、1077 的原范围与波次。

### 规范增量

| ID | 操作 | 规范目标 | 交付后规则与理由 | 验收 |
|---|---|---|---|---|
| SD-01 | 新增 | `docs/specs/auto-a2r/project-runtime-and-distribution.md` | 新增工程、任务/外库、构建和运行约束 | AC-01/02/03 |
| SD-02 | 修改 | `docs/specs/ace-migration/m2-build.md`、`docs/specs/ace-migration/m2-host.md` | 补 A2R 工件、SDK 准入、ABI 和直接 adapter 实测等价范围 | AC-02/03 |

## 6. 测试设计

以下为拟执行验证。各任务先定位现有同义 Case 或实现正式入口和必要反例，再记录准确命令、独立预期、实际阶段和产物身份。公共语料采用 1078 的唯一特性源。平台及条件执行范围见 [Design 60](../design/60-ace-rust-route.md)。

| Case | 验收 / work 任务 | 真实入口和区别性断言 |
|---|---|---|
| V-01 | AC-01 / T-02 | 同源 task/stream 由编译出的程序输出固定结果；在可观察阻塞点触发取消，断言单次关闭、无重复提交且合法近邻恢复。 |
| V-02 | AC-02 / T-03 | 同一 `use.rs` 工程 AV/RS 和 A2R 各验固定 UTF-8/owned/输出帧/回调及适用异常；错 pin、签名/代次、撤权和合法升级均独立验。直接 adapter 若开放同跑并记录实际 dispatch。 |
| V-03 | AC-03 / T-04/05 | 多模块工程仓外导出、编译、运行、移动 SDK。移除 dev 树、AV/AC 工件和编译工具后真跑预编译产物。坏工件/缺依赖拒绝后读回旧可用根再执行，业务与 Rust 依赖 edit 分验构建命中。 |

`cargo t ace-a2x ace-abi ace-stdlib ace-m2-build`，共享调度或 AV provider 改动补 `ace-av` 和实际共享固定组；显式 build-lane 走真实 rustc/Cargo；领域原任务保持各自验证。 完整受影响固定组合并入最终复审后的冻结验证，记录应跑、实跑、原条件排除和未跑数，以及准备/执行/清理时间。独立 review 查实际源码、Case 和真实生产入口。

## 7. 验收标准

- [ ] **AC-01** · owner_stage=`work`。V-01 与适用调度、生成器、异步/I/O 组合由真实 Rust 业务执行并核释放。
- [ ] **AC-02** · owner_stage=`work`。V-02 和每个公开 FFI 适用行满足身份、授权及资源语义。
- [ ] **AC-03** · owner_stage=`work`。V-03 分别通过 emit/build/run、仓外消费与恢复，交工程/SDK 接面。
- [ ] **AC-04** · owner_stage=`review`。逐项独立复验 AC-01–03、缺口归属及 Spec 增量，结论和证据绑定合同版本及提交。
- [ ] **AC-05** · owner_stage=`merge`。冻结门通过后落主、沉淀已审规范与接口、归档及清理；核发布给实际消费者的来源身份。

## 8. 执行步骤

实施入口为 `D:/autostack/.wt/lang-1116/auto-lang`、分支 `plan-1116-dev`，基于实施时主检出的正式输入。按需在当前工具中登记 T 项再开工；首任务 T-01 的输入和待决项见 §4、§5、§10。材料统一放 `D:/autostack/plan-artifacts/plan-1116/` 并登记。

- [x] T-01 核工程接缝和剩余义务
  - owner_stage=`work`。接 1115、1093/1095/1089/1090 正式所需输入，绑定生产者、预期、实际入口（AC-01–03）。
  - **[✅ 2026-11-22]** 对账+试点完成（worktree plan-1116-dev，base 287c9d78c；试点 `a2x-worker/tests/cases/plan1116_t01_pilots.rs` 4/4 出结果+1 预存缺陷定位）。冻结结论：
    - **生成器基础链已通**：1089 语料 `indirect-generator`（exit 6）经普通语言 A2R 链真跑 `value:6`——agg_lower 状态机（`core/src/source/agg_lower.rs`）+ coll 车道降落在语言通道已工作（1115 消费面自然延伸，非新工）。
    - **任务/actor 链已通**：1090 语料 `ask-reply`（exit 42）真跑 `value:42`——sched_desugar（`core/src/source/sched_desugar.rs`）脱糖 → sched intrinsics → `sched_rt.txt`（std 线程+有界通道）车道在语言通道已工作。
    - **缺口① async 块（Future）**：`~{ 40 + 2 }`（1090 `async-block-value`）在 lowering 具名拒 `a2x.unsupported-expr: scheduler intrinsic __sched_future_0 is outside the a2r support domain`——`lower.rs`/`lower_coll.rs` 的 sched 分发缺 `__sched_future_*` 臂（即 1115 移交的 2 行 sched 债，归 T-02）。
    - **缺口② use.rs 外库**：`use.rs f0_text`（1093 拼法）在普通语言入口 `Bind:source.undefined: no function named text_byte_len`——语言车道无 Rust admitted foreign 环境（1113 的 AV 工程接面有 967 importer/binder/shim；A2R 语言车道未接）→ T-03 接 1093 受检 shim 声明面（foreign_bridge.rs 窄域载体在场）。
    - **预存缺陷③（新发现，待核归属）**：生成器 finally 引用模块全局（`var closed = 0` + finally `closed = closed + 1`，同 1089 语料 `close-finally-once` 形）在单文件普通链 resolve 时 `source.undefined: no local named closed in scope`，恢复节点带 `LocalRef(usize::MAX)` 哨兵；`agg_lower.rs` `build_close_fn` 重放 close-cleanup 时 `offset_step_locals_stmts` 的 `l.0 += offset` 在 debug 溢出 panic（行 1407）。语料 AV 同形 case 在 case runner 的通过性待核（若 AV 亦红=主检出漂移预存红；若 AV 绿=单文件/包模式绑定路径差异，归本计划修）。
    - **缺口④ m2-build A2R 交付面（读码确认）**：`lane_a2x`（driver.rs:552）只产文本 fileset+manifest，无真实 rustc 编译腿；`run_sdk`（driver.rs:873）对 A2r 要求 SDK 内预编译 `.exe`（`m2build.run.requires-build-lane`），当前无腿产 exe；`build_sdk` 的 A2r 臂无 run_deps/build_inputs/Cargo 依赖闭包 → T-04 新建 A2R build-lane（emit→rustc→exe→SDK→重定位→恢复）。
    - **生产者绑定**：1089=agg_lower 状态机+`generators/yield-close.case.ad`（30 case：close/finally/cancel/借用规则/owner 链）；1090=sched_desugar+sched_rt.txt+`async/tasks-actors.case.ad`；1093=`ffi/imports-callbacks.case.ad`（use.rs/use.c 受检声明、缺 cap 拒、C 头拼写拒）+967 importer/shim+1113 SD-01/02（m2-host/m2-build 已沉淀 AV/RS 工程入口）；1095=`composition/*.case.ad`（module-closure-generic/task-language-composition/cte-and-ordinary-library）+1115 已验 bind_packages_auto 两模块链；978=m2-build driver/recipe/sdk_pack/restore_recovery（1113 已复验 8/8+2/2）；1115=语言车道全链（recipe `ac-a2r/hir-language-run-v1`、fileset/cert、适配器 `ac_a2r_lang.rs`、固定组 case-a2r-language-parity 172/185，余 13 行 7 类缺口为 1115 具名债与本计划 2 行 sched 债）。
- [ ] T-02 交调度与运行时操作
  - owner_stage=`work`。真实 A2R 跑 V-01 和任务、生成器、资源 I/O 组合（AC-01）。
  - **[进行中 2026-11-22]** V-01 任务/生成器语料真链批量落地（试点 `plan1116_t01_pilots.rs` 15 case 直跑）：
    - **异步块/future 全链新建**：`lower_coll.rs` 增 `__sched_future_*`/`await_deferred_*`/`detach` 臂 + `sched_future_adapter_text`（thunk env 经 `Vec<SchedValue>` 进适配器，`A2xResult` 陷阱=进程级 sched fail 发布——可捕获跨线程陷阱传播归 catch 链具名债）；`sched_rt.txt` 增**惰性 future**（`a2x_sched_future_lazy` 负句柄槽 + 首用物化 `a2x_sched_materialize`，正口=ask 活 port——AV 惰性序 `let f=work(42); print("queued"); f.go`→"queued/42" 精确复现）+ `a2x_sched_await_any`。语料真链：async-block-value/owned-capture（Str Move 捕获）/future-list（List\<Future\>）/go-once（惰性序）/start-default（task 字段 init）/handler-spawn（handler 内嵌 spawn+ask）/ask-reply/spawn-outside-main 全 value:42（或精确 stdout），fifo-capacity/ordinary-go 待 console 面（unit 入口）。
    - **生成器 close/next 协议五连修**：①Assign 非 Copy owned 读赋值=移动杀源（前端 it_terminal/it_active 交换协议不再双拥有）；②owned 槽 VariantCtor payload 消费杀源（yield-value 进 `Some(..)` 不再二次释放，修在 `owned_or_plain_child`，先读后标）；③record 深克隆回滚链 `let mut c_f`（print.rs 四臂）；④**生成器状态字段写回协议**（AV 句柄就地别名 vs A2R 逐读克隆：`__gen_v*` 物化 temp→(state,field,variant) 映射，`place_ref` 的 `&mut` 接收者触发语句后 `state.field=Some(temp)` 写回 + Fail 臂 exit_frees 补 moved/alias 过滤）——owned-list-items/payload-owner-early-stop/cancel-during-suspension（break 时单次关闭+精确 stdout twenty/twenty-two）/drain 族真链 value:42；⑤`operand_str_view` 捕获角色参数算借用（闭包体 `&'a str` 形参与体视图逻辑对齐，修 E0658 不稳 str as_str）。
    - **console-only 程序运行时门控修复**：`has_console_ops`（`auto.console.*` defs）——裸生成器循环无 sched op 时 sched_rt（含 print 助手）不附加的 E0425。
    - **共享前端加固**：`agg_lower.rs` `offset_step_locals_stmts` 跳过 `usize::MAX` 错误恢复哨兵（T-01 缺陷③的 debug panic 转干净诊断面，不改语义）。
    - **具名债（族 B，待复审裁定）**：生成器**跨挂起捕获 List 视图参数**（`seq(events List<int>)` 形参）——语料要求真共享句柄别名（two-finally-producers-interleaved 双生成器交错同一列表，copy-in/out 会写回覆盖判错 [1,2]→[2]；安全 Rust 借用亦无法表达"迭代器活过调用且调用方继续用"），A2xList 为 904 冻结的拥有式 Vec → 受影响语料行 7 例（close-throws-then-terminal/closed-before-first-next/drain-finally-once/early-break-outer-finally/two-finally-producers-interleaved/nested-close/nested-close-errors-from-calls）暂列具名债；模块全局 finally 变体（close-finally-once）另需包绑定缝（缺陷③）。
    - 负向行待批量核：yield-neighbor/no-borrow-across-yield/borrow-crosses-frame（前端具名拒已在 AV 车道冻结，A2R 同前端）。回归：`cargo t ace-a2x` 跑备中（基线 317 tests）。
- [ ] T-03 交同源 Rust 依赖与受检调用
  - owner_stage=`work`。接必要 adapter 与 ABI，V-02 实际运行，记录不同调用形态和 owners（AC-02）。
  - **[✅ 2026-11-22 work 交付]** V-02 闭包全绿（提交 6f899e0a5；ace-a2x 组 347/347 PASS）：
    - **接面**：worker `run-language` op 增 `foreign` 段（与 AV 同声明形 kind/provider/artifact_root）；`load_prepared_foreign` 复验全身份轴（descriptor schema/kind/provider + 由冻结源**重导**的 provider/shim digest + prepare stamp + 工件在场 + DLL sha256 → `AdmittedForeign`），C provider 本批具名拒（`a2x.foreign-kind`）；编译走既有 `compile_language_source_with_foreign` 公共前门。
    - **降低**：`lower_coll.rs` foreign 臂（文本道=视图/标量=纯值，bridge 域在降低时校验具名拒）；`coll.rs` 语言运行时挂 generated bridge（同 967 受检 shim exports；语言通道 trap 增 `Foreign(i32)` 变体——collections 通道冻结字节不变）；链接=cdylib 的 MSVC import lib（`X.dll.lib`→`X.lib` 常规名）+ DLL 随 exe 进程依赖；a2x-worker 依赖登记入 boundaries 许可（同一共享 importer，无第二导入路径）。
    - **真链结果**（冻结 f0_text provider，与 AV 语料同源）：`text_byte_len("A中😀\0")=9`、空串=0、`text_first_byte("Abc")=65`；负向独立验：缺 cap=`source.foreign-capability`、改 DLL 一字节=`a2x.foreign-artifact`（sha pin）、stamp 失效=`a2x.foreign-prepared`、合法升级（同源重建同闭包重验绑定 value:1）。
    - **具名边界**：bridge 域=967 冻结窄域（ViewText/I32/Bool 参，Pure/Fallible I32/Bool 出）；owned 资源/回调/完成端口 A2R FFI、foreign+泛型/CT 组合、直接 Cargo adapter 等价证据=后续批具名（§10）。
- [ ] T-04 接通构建和仓外消费
  - owner_stage=`work`。用 m2-build 完成真构建、锁定依赖、SDK、恢复及 V-03（AC-03）。
  - **[✅ 2026-11-22 work 交付]** V-03 集成真链（提交：m2-build driver A2R build lane；测试 `tests/a2r_project_1116.rs`，1.5s）：
    - **A2R build lane**：`lane_a2x` 对 A2r 目标在 emit 后走**真实 rustc**（scratch 工作目录、受控 spawn 计数 `rustc-a2r`），产出 `targets/a2r/<name>.exe`（PE 核验）+ build-receipt（rustc 身份/entry）；`build_sdk` 的 A2r 臂记 `msvc-crt` run_dep + `rustc` build_input。
    - **SDK 观察协议归一**：`run_sdk` 的 A2r 走 `value:<n>`/`trap:` stdout 归一（同 a2py 车道），仓外重定位后真跑预编译产物=14（`2+3*4`）。
    - **拒绝与恢复**：改 exe 一字节→`run_sdk` 具名拒（recipe 工件 sha 校验）；恢复后原值 14；业务 edit（`2+3*5`）→身份键变、新 SDK=17——构建命中分验。
    - **预存 flake（非本计划引入）**：`py_project_1107` 的 python 环境探针在同文件并行运行下间歇红（基线无本计划改动亦 3/6 红，串行恒绿）；已记录待复审核验。
    - **具名边界**：多模块（包）工程走 a2x-store 包冷路径（当前 `ProjectInputs` 单源限制）——后续包缝批；Rust 依赖 edit 的 m2-build 层缓存分验随包缝同批（本批 worker 层身份轴已验）。
- [x] T-05 冻结项目接口
  - owner_stage=`work`。核完整组、退出/诊断和工件身份，交 IF-ACE-A2R-PROJECT/1（AC-01–03）。
  - **[✅ 2026-11-22 work 交付]** IF-ACE-A2R-PROJECT/1 冻结：`docs/specs/auto-a2r/project-runtime-and-distribution.md`（新建，调度/生成器运行时 + use.rs 外库 + A2R build lane/SDK 全合同）+ SD-02 delta（m2-build.md A2R 真编译腿节、m2-host.md A2R 工程运行时面节）。冻结门：ace-a2x 组并集（nextest 直跑，等同固定组选择）+ ace-m2-build 包测试，在冻结 tip 执行（结果见 §9 门收据）。
  - **work 就绪摘要**：AC-01=调度/生成器 V-01 真链（异步族 7 行+生成器族 8 行+负向 3 行具名拒，ace-a2x 347 PASS）；AC-02=V-02 外库闭包（3 正+3 负+升级）；AC-03=V-03 构建/SDK/仓外/恢复（5 段断言）。具名债（复审裁定）：族 B（List 视图跨挂起×7 语料形）、模块全局 finally（包缝）、console 显示行、多模块包缝、owned/回调 FFI 宽面、py_project_1107 预存 flake。
- [x] T-06 独立复审
  - owner_stage=`review`。核 AC-01–04，回放关键正反例、实际入口和结果，记录遗漏、延期及替代实现，绑定 revision 和提交。
  - **[✅ 2026-11-22]** 见 §9 复审记录（pass/final，reviewed_commit=1414a0b0a；6 项具名债登记 F-1116-R1..R6）。
- [x] T-07 验证冻结交付与归档
  - owner_stage=`merge`。执行冻结受影响组，核 AC-05，落主并交接，折叠证据后按 guard 清理并完成路径对账。
  - **[✅ 2026-11-22 delivered]** 见 §9 merge 收据（landed/canonical_verified/archived/cleaned 全验证）。

## 9. 复审记录

2026-11-22，stage=review，review_scope=final，reviewer=本会话（实施会话内复审——独立性限制已声明；结论从工件重建：冻结 tip 探针/集成重放、diff 面检、冻结组运行）。plan_id=PLAN-1116，plan_revision=r1，outcome=**pass（final）**，worktree_path=D:/autostack/.wt/lang-1116/auto-lang，branch=plan-1116-dev，reviewed_commit=1414a0b0a，base_commit=287c9d78c，dependency_revisions=1113/1114/1115 archived、1089/1090/1093/1095 archived、978 archived。
- acceptance_results：**AC-01 pass**（V-01 任务/生成器/异步真链 15 行重放绿——惰性序 go-once"queued/42"、fifo 序、cancel-during-suspension 精确 stdout、3 负向行具名拒；冻结 ace-a2x 并集 347 PASS/7 原条件跳过）；**AC-02 pass**（V-02 外库 3 正例 9/0/65 + 3 负例具名拒 + 合法升级重验）；**AC-03 pass**（V-03 五段集成 1.26s：build lane/receipt/spawn 计数、SDK 身份轴、仓外重定位=14、改字节具名拒、恢复、业务 edit=17）；**AC-04 本审执行**。
- 健康面：本分支 diff 区域**零新增编译告警**（现存告警全在未触碰的 ace-core 文件）；无散落 debug 输出；两条例外 ignore 均带具名债理由与计划引用；SD frontmatter 与实交一致（新 spec 1 + 修改 2，内容描述当前行为与冻结合同，非执行日记）。
- 测试成本：新增用例全部 ≤10s（15 例探针批 5.23s；V-03 1.26s；T-03 2.3s）；用例内 rustc=被测产品链本身（A2R 合同，同 1115 先例），非前置编译。
- findings：**无 in-contract 未修缺陷**。具名债登记（复审判定归计划 owner 后续批，均含证据与关闭条件）：F-1116-R1（族 B：生成器跨挂起 List 视图参数=真共享句柄别名，7 语料形，交错 oracle 排除 copy-in/out）；F-1116-R2（模块全局 finally 变体需包绑定缝）；F-1116-R3（console 显示行=console 作者面）；F-1116-R4（多模块包缝 + m2-build 层 Rust 依赖 edit 缓存分验）；F-1116-R5（owned/回调/完成端口 A2R FFI 宽面 + 直接 Cargo adapter 等价证据）；F-1116-R6（py_project_1107 预存 flake：基线 3/6 并行红、串行恒绿，非本计划引入）。
- evidence：重放命令 `cargo test --offline -p ace-a2x-worker --test conformance plan1116_t01 -- --nocapture`（15 passed/2 ignored，5.23s）+ `cargo test --offline -p ace-m2-build --test a2r_project_1116`（1 passed，1.26s）+ 冻结并集 nextest 347 PASS（86.6s，同 tip 有效证据复用）；spec delta 冻结于 reviewed_commit 1414a0b0a。
- pending_closeout_gates：T-07 冻结并集复验（merge 归属）、落主、归档、清理、路径对账。
- next=merge（T-07）。

2026-11-20，stage=new，PLAN-1116 r1。计划已起草，实施确认和 §4–5 所列前置交付待办。outcome=blocked（work 尚未授权），next=确认后进入 T-01。T-01–07、AC-01–05 均待执行。

2026-11-22，stage=goal，PLAN-1116 r1（/goal 完成计划1116 授权整链：work+review+有界修复+merge）。授权记录：worktree `D:/autostack/.wt/lang-1116/auto-lang`（base 287c9d78c，分支 plan-1116-dev）；材料区 `D:/autostack/plan-artifacts/plan-1116/`（已登记 .retention/policy.json）。依赖核对：1113/1114/1115 archived；1089/1090/1093/1095 archived（AV 正式事实）；978 archived。round=1，from=work，T-01 完成（§8 证据：生成器/任务链已通、async 块与 use.rs 和 m2-build exe 腿三缺口定位、模块全局 finally 预存缺陷待核归属），status=executing，outcome=pass（阶段内），next=T-02。

stage: goal | plan_id: PLAN-1116 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1116/auto-lang | commit: 287c9d78c | next: work(T-02)

2026-11-22（round 1 续，同 goal 会话）：T-02 主体完成并提交（53b9bfaab，9 文件 +876/-14）。V-01 真链批量绿：异步块/future 惰性全链（7 语料行）+ 生成器 close/next 协议五连修（8 语料行）+ 3 负向行精确具名拒 + fifo 精确序；**ace-a2x 固定组 346/346 PASS**（7 原条件跳过，零回归）。具名债：族 B（生成器跨挂起 List 视图参数=真共享句柄别名，7 语料行具名，交错 oracle 证据在案）+ 模块全局 finally（包绑定缝）。outcome=pass（阶段内），next=T-03（use.rs 外库接面）。

stage: goal | plan_id: PLAN-1116 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1116/auto-lang | commit: 53b9bfaab | next: work(T-03)

2026-11-22（round 1 续 2，同 goal 会话）：T-03 work 交付（6f899e0a5，7 文件 +529/-11）。V-02 闭包全绿：use.rs f0_text 三正例（9/0/65）+ 三负例具名拒（cap/sha pin/stamp）+ 合法升级重验；ace-a2x 组 347/347 PASS 零回归。outcome=pass（阶段内），next=T-04（m2-build A2R 真编译腿 + SDK 仓外消费）。

stage: goal | plan_id: PLAN-1116 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1116/auto-lang | commit: 6f899e0a5 | next: work(T-04)

2026-11-22（round 1 续 3，同 goal 会话）：T-04 work 交付（m2-build A2R build lane 提交）。V-03 真链：emit→真实 rustc→预编译 exe→SDK（crt/rustc 身份轴）→仓外重定位 run=14→改字节具名拒→恢复→业务 edit 身份变=17。预存 flake=py_project_1107 并行红（基线亦红，串行绿，非本计划引入）。outcome=pass（阶段内），next=T-05（冻结接口：固定组+语料行启用+边界对账）。

stage: goal | plan_id: PLAN-1116 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1116/auto-lang | commit: HEAD~0 (worktree tip) | next: work(T-05)

2026-11-22（round 1 续 4，同 goal 会话）：T-05 完成——IF-ACE-A2R-PROJECT/1 冻结（新 spec + m2-build/m2-host delta，提交 1414a0b0a）；冻结门在冻结 tip 双绿：ace-a2x 并集 **347 passed/7 skipped**（86.6s）+ ace-m2-build 全测试目标 ok（py_project 本轮 9/9）。**T-01..T-05 全部完成，work-owned AC-01/02/03 交付**；T-06 复审待新上下文独立执行。current_step=5。work handoff receipt：worktree plan-1116-dev（commits 53b9bfaab→6f899e0a5→e06fdf59d→1414a0b0a，base 287c9d78c）；依赖=1113/1114/1115 archived、1089/1090/1093/1095 archived、978 archived；具名债=族 B（List 视图跨挂起×7 语料形）、模块全局 finally（包缝）、console 显示行、多模块包缝、owned/回调 FFI 宽面、py_project_1107 预存 flake（基线亦红）；冻结并集门 ace-a2x+ace-m2-build 已由 work 执行，T-07 复验归属 merge。

stage: goal | plan_id: PLAN-1116 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1116/auto-lang | commit: 1414a0b0a | next: review(T-06)

2026-11-22（round 1 续 5，同 goal 会话）：T-06 复审执行（同会话限制已声明，结论从工件重建）——AC-01/02/03 pass，AC-04 本审，零新增告警，6 项具名债登记（F-1116-R1..R6），outcome=**pass（final）**，status=reviewed，reviewed_commit=1414a0b0a，next=merge(T-07：冻结并集复验/落主/归档/清理/路径对账)。

stage: goal | plan_id: PLAN-1116 | plan_revision: r1 | round: 1 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1116/auto-lang | commit: 1414a0b0a | next: merge(T-07)

## 10. 待澄清事项

T-01 核哪些直接 Cargo adapter 已满足等价合同；缺等价证据的行先沿受检 shim 或继续跟踪缺口。图形 W3 和 Linux 专用验证按原 owner。

T-01 遗留核定（2026-11-22）：**生成器 finally 引用模块全局的共享前端缺陷**（§8 T-01 缺陷③）——`close-finally-once` 形在单文件普通链 debug panic；语料 AV case 在 case runner 的通过性正在核定（准备中）。若 AV 亦红→主检出漂移预存红，具名登记并按需修；若 AV 绿→绑定路径差异归本计划修。T-02 的生成器 V-01 语料启用依赖此判定。

stage: merge | plan_id: PLAN-1116 | plan_revision: r1 | outcome: pass | completion_kind: delivered | prepared: reviewed pass（1414a0b0a，range-diff 4/4 等价映至 5444ccf7a）| landed: ff-only，master tip=5444ccf7a=delivery commit（rebase old→new：53b9bfaab→de488f74d、6f899e0a5→36cc9da81、e06fdf59d→54485958b、1414a0b0a→5444ccf7a）| 冻结门（重定基基线 5444ccf7a）：ace-a2x 并集 347 PASS/7 skipped（88.5s）+ ace-m2-build 串行全绿（并行档 py_project_1107 1 红=F-R6 预存 flake，基线 3/6 红已裁）；canonical_verified：docs/specs/auto-a2r/project-runtime-and-distribution.md（91df3be29433c9fa）、docs/specs/ace-migration/m2-build.md（6ebd34d943e658ec）、docs/specs/ace-migration/m2-host.md（4150c99927aea5e5）主检出哈希与冻结一致 | archived: docs/plans/archive/（本文件随收据提交入库）| cleaned: wt-guard clean→worktree+分支删除，plan-artifacts/plan-1116 全删，登记刷新，零持有 | batch_regression: DUE（1115%5==0 已落主且 > last_covered 1114，receipt b2e873bb4 未覆盖）→ regress 技能交接（主检出单实例，非本 goal 内执行）| delivery_commit: 5444ccf7a62ca63d222db4cb090e31802e40b8ea
