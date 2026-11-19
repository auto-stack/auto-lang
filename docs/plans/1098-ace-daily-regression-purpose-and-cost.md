---
plan_id: PLAN-1098
status: executing
feature_name: ACE日常回归用途与成本重整
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19

plan_revision: 1
depends_on: []
supersedes_spec_components: [docs/specs/auto-test-case/module-selection.md, docs/specs/auto-test-case/ace-regular-test-budget.md, docs/specs/auto-test-case/ace-cost-attribution.md, docs/specs/auto-test-case/ace-perf-cost-and-reproduction.md, docs/specs/auto-test-case/test-purpose-and-admission.md]
new_spec_components: [docs/specs/auto-test-case/ace-toolchain-certification.md]
touched_goals: []

affects: [auto-test-case, auto-ac, auto-av, auto-syntax]
current_step: 0
total_steps: 11
---

# [PLAN-1098] ACE日常回归用途与成本重整

## 0. 变更摘要

承接2026-11-19的ACE Rust测试调查，治理普通正确性测试中的成本与用途混杂：正式性能测试漏归属、测试体补构建/release路径、低效夹具、观察型探针、跨工具链全链认证，以及宿主与语料入口的重复执行。**本稿只规划；没有执行本计划的测试、迁移、分流或入口切换。**

范围以真实ACE职责组为入口，已有47项AV慢例逐项列入§5.3；不将4,464个Rust测试全部改写成AutoDown。公共语言源码与预期适合文档化；非法IR、协议负控、缓存身份、资源/回滚状态继续由真实Rust测试保护。目标是快速发现特性间副作用，文件格式不是日常准入依据。

八阶段、11任务：冻结用途/成本 → 准备与运行分离 → 性能归属和夹具优化 → 探针固化 → 工具链分流和宿主消重 → 预算检查和逐组验收 → 独立复审 → 合入沉淀。各阶段保原义务映射，不以删断言、缩冻结规模、批量ignore或拆一分钟碎组达标。

## 1. 目标

1. 对每条被修改、替换、合并或移出的回归义务记录实际旧身份、独立oracle、适用条件、新入口及关闭证据。内部单测和必要真实接缝继续保护未来修改。
2. 日常执行消费明确准备的debug/test产物；缺失/陈旧产物明确失败，不自动补Cargo/依赖构建、release构建和外部Rust/C认证编译。真实Auto前端、MIR/ABC、AV及native codegen/link本体仍可作为被测功能执行并计时。
3. 完整普通职责组目标≤60秒、验收上限≤120秒。本计划新增的每个原生test/文档check≤10.000秒；已知47项中继续留日常的完整原生test也优化到≤10秒，不能只看内部某小段。
4. 正式采样/冻结压力归`ace-perf`；外部工具链认证有独立显式入口；临时探针转有效断言或逐项具名退出日常。普通正确性慢例优先优化，不能按耗时改称性能测试。

非目标：动态叶目录/消费者图、全仓自动挑测试、重新实施1066、批量重写公共AutoDown作者、修改auto-musk技能、启动WSL/Linux/VM、实现1017已推迟的CLIF/binder/closure产品算法优化、给每个计划附加全ACE/全PERF门。

## 2. 架构方案

复用固定组、`scripts/fixed-tests.py`、`scripts/test-governance/prepare_tools.py`、nextest和`ace_cost_report.py`。用途审查是可追溯的维护结论，不变成另一套执行注册表。

| 运行面 | 选择与职责 |
|---|---|
| 普通完整职责组 | 快速正确性、非法输入、小规模资源边界、生命周期、协议负控、必要真实接缝；只消费已准备产物 |
| `ace-perf`（已有） | `::performance::`、原串行profile/平台/ignore/样本/尺度/阈值，保正式计量合同 |
| `ace-toolchain`（拟新增） | 显式固定组，`all=false`，不被`ace`/日常/`all`隐含选择；真实rustc/C编译、多端完整compile/run及其负控；依赖仍先debug准备 |
| research（逐项裁决） | 具名手工入口或转为有oracle的日常例；不充日常覆盖 |
| Linux（1061/1063所有权） | Windows零原生Linux执行、零WSL；本包保持隔离，不代签原生Linux验收 |

`ace-toolchain`尚不存在，由T-06交付；不吞并已有`a2r-compile`的250项legacy专项。原各后端源码、provider、输出、产物、条件、reference与证书义务保全；AV成功或Rust emit金样不能代签真compile/run。真实修改触及对应生成器/ABI/工具链时，必要消费者验证仍须显式选该组，不成为每Plan固定收尾。

拟在同一固定驱动增加`--prepare-only`、`--run-prepared <manifest>`：前者准备所选完整调用的工具/测试二进制，后者只验身份并运行。T-01须核安装版nextest的实际二进制/归档复用接口；不能假定某个`--no-build`选项存在。不建第二runner、逐叶新鲜度或自动依赖图。

交付后普通`cargo t <ACE组>`/`cargo t ace`默认也只能消费本树匹配的准备收据；找不到时返回具名PREPARATION_REQUIRED并给显式准备命令，**不能只增加可选run-prepared而让原日常入口继续lazy build**。显式传manifest可消除候选歧义。非ACE组原选择/执行行为不因本包改变；混合调用须在ACE准备失败时停止相应调用并保真实失败，不用其他组成功冲销。兼容命令的行为变更须在SD-01/说明中写明。

## 3. 技术栈与工作位置

Rust既有ACE测试、Python驱动/成本报告、nextest原结构化终态及Windows Job观测。常规准备debug/test增量；四个no_std运行时保必要panic-abort的`runtime-dev`，不改成release或破坏链接ABI。

未来唯一实施树`D:/autostack/.wt/lang-1098/auto-lang`、分支`plan-1098-dev`，本轮不创建。兄弟依赖用同组真实worktree，不建链接。材料唯一scope=`D:/autostack/plan-artifacts/plan-1098/`，开工登记owner/用途/路径/可删缓存；代码、编译与测试仅在实施树，计划簿记留主检出。纯规划临时分析在系统临时目录。

## 4. 需求分析与背景调查

### 4.1 授权、规范和并行边界

用户已授权新建详细计划，**先只规划**；未授权实施、跑新成本基线、切组或合入。`stage:new pass`只是规划合同自检，不是开工许可。

**2026-11-19T14:43 授权更新（goal 授予，覆盖上述"仅规划"状态）**：用户在本仓新会话直接下达 `/goal 完成计划1098`（goal id `goal-b63fa353-…`，objective `完成计划1098`，max rounds 256）。按 `auto-plan-goal` 规约，该 goal 即整链授权：在 T-01 指定树内实施、独立复审、限额内返修、以及在最终 `pass` 绑定当前 revision/提交后合入沉淀。授权**不**包含：改变目标/验收/阈值/兼容承诺/仓范围/预算、绕过 worktree 与证据政策、无最终 pass 强行合入。计划范围、AC-01…12、§5 逐行义务与 §7 验收均不变。

已读`docs/specs/overview.md`及下述固定组/预算/成本/性能/用途Specs。1017、1060、1066已归档，通过当前Specs继承约束，不重开旧计划；1060的21组/71个case入口、1017的94项S0为历史分母，不冒当前枚举。

| 并行/前序计划 | 衔接与边界 |
|---|---|
| 1061/1063 Linux | 不等Linux环境即可做Windows部分；不重复建设Linux设施。若普通组仍触发WSL，须在运行前拦住并协调1061最小Windows隔离切片；解决前该组未执行/裁定排除，不跑WSL后称达标，不阻无关组 |
| 1078、1079–1096、1068 | 共享AutoDown作者和多后端选择正在规划。本包可先治理成本；公共语言语料消费已交接口或登记具体迁移owner，不另建后端作者副本，不假称拟议fixture/adapter已实现 |
| 1072等在途产品计划 | T-01重冻源码/工具/config，协调同文件写窗，不借其在途产物/目录作本包证据，不撤其产品回归达标 |
| 1017后继产品优化 | 不提前实施已推迟算法改造。确无等价测试优化仍无法达预算时，具名记录未完成与最小修订，不自动放宽预算或移PERF |

### 4.2 已观测分母、成本和来源

调查基面`54ed8a4ca6aa28bea169b7d665c012b0a5ff3dd5`；当时ACE源码/固定驱动无工作区修改。其他会话的文档WIP与根目录临时`.rs`不在本包写集/测试分母。

静态扫描已跟踪`experimental/ace`：1,012个Rust文件，651个含测试，31包共4,464个`#[test]`声明（av 1,319/core 905/native 359/abi 312/a2x-worker 290/conformance 198）。排除注释/字符串伪声明；宏/cfg/target/ignore仍需真实发现。**声明数不是实际执行数，也不表示已逐例裁决用途。**

当前`domain=ace`为27个nextest/2个Python/73个case组。27中含外部`ase-m0`和显式PERF，剩25个ACE常规组含聚合`ace-workspace`。本包单组预算基线为**24个非聚合常规组**，另列聚合成本；不混用1066历史76全仓case组与当前73个ACE域组。

11月19日14:10已有收据，只作成本排序，不作本包当前通过证据：

| 记录 | 观测 | 边界 |
|---|---|---|
| lang-1072 `run-0xx9ddgf` | ace-source 919执行，整命令6.730秒，PASS | 大量内部回归可很快；非本包通过 |
| `run-8iqbutj5` | source+av总964.829秒：工具构建796.700、发现/准备80.117、执行86.972、残差1.040 | 本次工具准备主导，不能推论每轮相同；残差不冒已测清理 |
| 该次AV | 1,428执行，运行80.966秒，47项>10秒，2项失败 | 并行item时间不求和当整墙；不是PASS |

原失败身份：`host_console::av_console_entry_selection_cannot_bypass_main_policy`、`host_console::av_console_owned_front_door_enforces_console_entry_only`。T-01核归属；首红/重放分列，本包新红必须修复，外部旧红具名移交，不改expected/ignore掩盖，不用成本绿代功能绿。

run根=`D:/autostack/.wt/lang-1072/auto-lang/target/test-groups/`；`run-8iqbutj5` source_state=`3376df0c13ea20907aedeb2ee03bd99fd7f60ea489faaeccca8fe0102cb90c2f`；config=`090c0e71021f8cd283c2142a1456bc263c5229236d7ac48e37b01b9139635c68`。两份results.json SHA256按上述顺序为`c3a5907f0c92da261b55fa943b6e72d95d12c8e9e690fc5827eb595eec27357d`、`bdee19a19f2dcc3c1df779284a0c4d4fa329fd3c8612ae3688c50e97b34aed8f`。他计划清理路径后T-01仍须从实际源码/新run重冻，不能依赖材料永久留存。

### 4.3 来源指纹与已证差异

| 源码/规范 | SHA256（规划时） |
|---|---|
| `docs/specs/auto-test-case/module-selection.md` | `134dcebc4b7456fd1d8f3ef2a7c216247efcb13c7c5bc32009975d5ee514288b` |
| `docs/specs/auto-test-case/ace-regular-test-budget.md` | `942077bcdda21052be7c817a2e4afdf83d3c36ae062db14358a8fa6bc845d464` |
| `docs/specs/auto-test-case/ace-cost-attribution.md` | `d5c19ea912dfd5f2b0488b907ecc4418d6ccfcd77916b13ca5b1e167e9520b93` |
| `docs/specs/auto-test-case/ace-perf-cost-and-reproduction.md` | `10c02ac2bf21e67005720c661ec8fe958355bf0f39173de42ce7494009f5ae66` |
| `docs/specs/auto-test-case/test-purpose-and-admission.md` | `c08df670e8e21ccf8daf2516f798aa50fa293da1ae452762a0dd3bb5d0c8fe76` |
| `docs/design/testing/ace-feature-organized-autodown-corpus.md` | `9c02d5b393e6c55b24de9ce86dc19cf83f6ffbff50a1d5e94289710654d9ae50` |
| `scripts/fixed-tests.py` | `379df21c3414fe47f194aa2ff621b5a50afbf7ff014e7b960e7b45d277d31d78` |
| `scripts/test-governance/prepare_tools.py` | `e043fd2c5068f193c2bbce306d66af3e7e65300f84bedff0ab1cc9b6ce17f5bc` |
| `scripts/test-governance/toolchains.json` | `177710e13656ddd4f2d2ba1a25662916f23a062c15a9b639df85fb5131b764ba` |

已证：`mod perf`与`performance::`过滤不符；native缺库补Cargo；Rust lowering缺env补release；MIR夹具7万次线性intern；部分探针两分支都打印即绿。需纠正调查推断：core/common的runtime-release helper目前只查到同文件内部引用，不可宣称已导致日常构建；现有prepare_tools已有同身份复用，796.7秒不代表每轮重复冷构建；固定驱动已有完整配置key的OR去重，不能再按组名猜重复。

T-01优先只读既有收据/配置/源码。确需新发现时仅显式debug/test准备，不走旧自动release工具准备；可比的“优化前”夹具/用例计时在T-02准备边界落地后，以同debug配置重放旧夹具形成，不能拿旧release工具时间冒debug A/B收益。

## 5. 详细设计

### 5.1 用途与义务守恒

对所有活跃包/target/模块家族作用途核查；对本包全部改动行、47项、实施新发现>10秒项、补构建路径与无oracle路径逐个记录。其余未逐例人工评审的行记“家族核查、原态保留”，不充逐例日常准入结论。

| 类型 | 已核实际例子（`experimental/ace/`前缀） | 保留的长期义务 |
|---|---|---|
| Auto行为/诊断 | `crates/core/tests/cases/scalar_source.rs::{scalar_positive_corpus_compiles,scalar_negative_matrix}` | 9普通正例+2能力例、19诊断输入的profile/domain/capability/code/span；公共输入与内部断言分工，不因Rust载体误删 |
| 非法IR/坏包 | `crates/av-core/src/compact_abc.rs::av_wire_corrupt_sections_and_opcode_rejected` | 截断/重复/重叠/缺段/非法opcode受控拒绝，保持Rust直接造坏包 |
| 生命周期 | `crates/av-core/src/av.rs::{av_borrow_rejects_expired_frame_and_slot_generation,av_orphaned_env_cleanup_releases_in_reverse_order}` | 过期frame/slot代际与最后owner释放/释放顺序，小规模日常保留 |
| 缓存/事务 | `crates/native/src/typed_session.rs::{audit1032_ntx_02_batch_failure_is_atomic,audit1032_ntx_02_signature_change_pulls_real_dependents}` | 失败保旧与真实依赖拉取，不能只看最终值 |
| 恢复/租约 | `crates/source-query/src/modules/persistence.rs::candidate_cancellation_and_actual_root_publication_failure_preserve_old_recovery`；`crates/artifacts/tests/cases/leased_publication.rs::leased_put_and_owned_root_retirement_preserve_readers_and_other_projects` | 取消/发布失败后下一次恢复、旧reader/其他project根不误删、最后释放后回收；留真实小store |
| 框架/协议 | `tests/conformance/tests/console.rs`三流、版本stale、Skip、NUL测试 | 错预期必须判失败、缺轴不能绿、真实运输不能全换mock |
| 工具链认证 | §5.3 C类和`host/rust_lowering.rs`认证两例 | 真compile/run、独立reference/oracle、provider/证书，显式专项保全 |
| 正式性能/开发探针 | §5.2 P类/O类 | 原正式采样尺度不缩；观察型转独立oracle或有据research，不按名字批量退役 |

逐义务记录：审计ID、旧package/target/full_test或Case/check、源码、原选择/平台/ignore、输入闭包/逻辑路径/入口/能力/预算、独立预期与断言角色、消费者、新用途/入口、拆分合并映射、负控、完整前后item时间/来源、owner/状态。一个test含多个角色时逐角色守恒，canonical执行数另列。材料表只放本包scope，耐久关键对照折回本计划/Spec，不建立自动执行侧表。

### 5.2 已确认问题逐项合同

下表路径除特别说明均以`experimental/ace/`为根，执行按完整原生身份而非行号绑定。

| ID | 实际路径/符号 | 拟优化/分流 | 必须保留的义务与关闭条件 |
|---|---|---|---|
| P-01 | `crates/syntax-core/tests/cases/perf.rs::perf_fresh_parse_and_incremental_and_restart_query` | `tests/conformance.rs`的`mod perf`改真正`performance::`选择面 | N=32、3预热、8,000声明、fresh/增量/回退/重启场景和原读数；普通0选择、PERF唯一选择 |
| P-02 | 同文件`perf_core_vs_legacy_parser_reference` | 同上 | 6,000函数、CST/legacy两链、32样本与P50/P95/P99，不只留下一个parser |
| B-01 | `crates/native/tests/common/mod.rs::trace_support_lib`及`common/collections.rs`库helper | 缺库Cargo分支改消费预建manifest | 真实trace/collections ABI、capability、hash、link/run与错库拒绝；缺/坏产物不得补构建 |
| B-02 | `crates/case-worker/tests/cases/backend.rs::trace_support_library_travels_to_the_mn_link` | 缺库Cargo移显式准备，保原测试 | exit=12、stderr=`ba`的参数求值顺序，真实支持库进link；缺库是infra失败 |
| B-03 | `tests/conformance/tests/host/rust_lowering.rs::{a2r_hir_first_case_green,maintenance_build_worker}` | 删缺`AC_A2X_HIR_WORKER`时release构建，统一预建 | 5项daily语料原输出/缺口判决、worker身份；无env不回落旧release/相对路径 |
| B-04 | `crates/core/tests/common/mod.rs::{text_runtime_lib,text_runtime_pair_copy,text_runtime_sha}` | T-01核可达，无消费者则清死helper，有则读预建runtime-dev | 有消费者时保runtime契约/23符号/hash/sidecar变异；死代码清理不冒实测收益 |
| B-05 | `scripts/test-governance/toolchains.json`的`av/batch-av/av-regular/ac/mn/mn-collections/cte/abi/sema/a2x`当前profile=release，`text-runtime/data-runtime`当前runtime-release | 普通常规准备改debug/runtime-dev并核所有消费路径；case-parent/trace-runtime/execution-fixture原合理profile保留。正式PERF原工具profile单独绑定用途，不和日常收据串用 | 不只禁止测试体release却让显式日常准备继续release；worker协议、静态库ABI/sidecar/hash与批次feature保持。普通/正式用途profile差异进入收据身份及输出路径，不改原PERF输入/样本/执行profile |
| F-01 | `crates/av/tests/cases/mir_tier.rs::mir_admission_module_tables_over_budget_is_resource` | 夹具线性构造7万常量，避逐次线性intern，不改产品语义 | 7万个不同I32、模块表门及精确Resource(TableEntries)；合法邻例和关闭资源门负控；不能缩成7千 |
| F-02 | `crates/av/tests/cases/sched_m2_968.rs::ok`全部调用者 | 一次run同时核exit/stdout；必要确定性角色另保最小双run | compile→lower→verify→ABC→scheduler真实链；所有原exit/stdout/拒绝；双run角色比较真实两次outcome/流，不能删后冒已验 |
| O-01 | `crates/av/tests/cases/p946_t01_probe.rs::p946_empty_remove_contract_observed` | 按当前List空remove合同固定精确trap/value/code，去两分支打印即绿 | 错结果必红，预期来自独立合同，不自动录制当前值 |
| O-02 | `crates/av/tests/cases/probe972.rs::probe972_t02_str_surface` | 六子场景各有值/诊断oracle，公共语料衔接1078 | str等/拼接、char比较/码点、slice/len_bytes、char_at均独立判决，不只不panic |
| O-03 | 同文件`probe972_p1b2_closure_param_frozen_rejection` | 核闭包参数现行支持；仍拒则精确阶段/code，已支持转正例 | 不能任意Err即绿，不能旧缺口阻挡已批准能力 |
| O-04 | 同文件`probe972_p1c_suffix_list_public_frozen_rejection` | 核List公共参数suffix装配合同，同上 | 成功绑定执行或指定诊断；语法错误不能冒原准入拒绝 |
| O-05 | `p946_t01_probe.rs::p946_list_argument_moves_single_module` | 保返回22断言，额外cap观察段强化或具名移research | 不因有打印删掉整个有效例；值与cap隔离角色分别可追溯 |
| H-01 | `tests/conformance/tests/console.rs::console_real_corpus_and_stale_identity` | 对照`case-ace-m1-console-863`15项；完整语言语料保case组，宿主留最小真实身份见证 | 15项语言义务不丢；av-profile/mir-lowering轴、fresh→改轴stale真实worker观察；不能伪造结果替代 |
| H-02 | 同文件`{console_three_oracle_axes_fail_independently_and_missing_axes_error,console_skip_remains_skip_and_never_creates_a_green_observation}` | 比较层能复用纯输入的复用，真实运输保最小例 | stdout/stderr/exit各错独立Fail并指向本轴；三种缺轴Error；Skip无native/绿结果，原6变异角色不丢。至少一条真实worker响应同时携带三轴，强制错/漏运输轴也必须被parent拒绝；不得只在假响应里证明比较器正确 |
| H-03 | 同文件`console_nul_bytes_survive_worker_transport_and_parent_judgement` | 保最小真实跨进程例 | NUL原始字节跨transport/parent保持，不能用mock代签 |
| H-04 | `tests/conformance/tests/host/rust_lowering.rs::{a2r_hir_tampered_golden_is_red,a2r_hir_missing_worker_is_infra_error,a2r_hir_worker_credential_pin}` | 便宜emit/坏golden/缺工具/身份负控保普通 | 错golden真红、缺worker infra、身份正确；不改Skip/准备成功 |
| H-05 | 同文件`{a2r_hir_certify_leg_all_cases_pass,a2r_hir_certify_negative_controls_reject}` | 显式`ace-toolchain` | family每个actual/reference真实执行、cert_sha256、原120秒/leg、错oracle14→15和错reference拒绝全保；旧认证预算不是新增例豁免 |
| H-06 | `scripts/fixed-tests.py::invocations`，host/console/rust-lowering/batch | 复用完整key的OR并集，只合兼容准备 | host+console同key不重复；host/rust-lowering工具key不同不强合，batch feature不同独立；零选择不PASS |

### 5.3 已知47项AV慢例

历史完整test墙钟见下，统一原生前缀`ace-av::conformance$`、源码`experimental/ace/crates/av/tests/cases/<模块>.rs`。每行内部子角色由T-01展开，不把47行当全部oracle数量。

处置代码：**C**=日常保AV/内部廉价接缝，外部compile/run保原输入/多端/独立oracle移显式`ace-toolchain`，不进PERF；**D**=日常正确性留原组，优化重复装配/I/O/准备，不改语义/失败义务；**R**=资源/身份/恢复反控留日常优先修夹具，原冻结规模不减，只有原合同确属正式采样/压力者可据证移PERF，普通大正确性不新建慢例收容组；**Q**=先区分计时/尺度实验与正确性，只有正式采样/冻结压力归PERF，不能因慢改用途；**L**=Linux目标绑定行，核1061/1063所有权，当前bind/cold link不含Linux运行，不能冒原生LinuxPASS。

| ID | 实际原生项 / 源码定位 | 历史秒 | 保留的回归义务 | 方案 |
|---|---|---:|---|---|
| S-01 | `bstr_1064::bstr_1064_v_d09_file_read_bytes_cross_end`（`bstr_1064.rs:968`） | 69.958 | 空/二进制/缺文件read_bytes的Option与字节内容、AV/A2R/A2PY独立结果 | C |
| S-02 | `kv_zip_1064::v03_zip_plan897_and_btypes_av`（`kv_zip_1064.rs:618`） | 51.688 | ZIP原资产、ASCII/CJK/动态deflate与BTYPE=00内容、读取预算边界 | D |
| S-03 | `bstr_1064::bstr_1064_v_codec_five_ends`（`bstr_1064.rs:696`） | 43.781 | 原codec合法/非法输入在AV/native/C/Rust/Python的各自结果 | C |
| S-04 | `resources_tier::av_turnaround_six_scenarios_smoke`（`resources_tier.rs:806`） | 42.960 | 六场景、原3样本、真实hit/miss、冲突诊断、body edit不同修订身份 | Q |
| S-05 | `bstr_1064::bstr_1064_t01_longest_chain_probe`（`bstr_1064.rs:73`） | 41.758 | 1024/4096/8192成功与16384 InstrSteps资源结果、原每段10秒门 | R |
| S-06 | `stdlib_asm_972::stdlib_asm_972_cross_end_gaps_and_a2r_math`（`stdlib_asm_972.rs:288`） | 38.324 | 装配缺口判决及真实A2R数学行为，不只生成文件 | C |
| S-07 | `kv_zip_1064::v03_kv_a2r_roundtrip_and_budget_4096`（`kv_zip_1064.rs:494`） | 33.775 | A2R KV roundtrip、4096边界、原最终ACE-KV/1磁盘字节 | C |
| S-08 | `kv_zip_1064::v03_zip_a2r_roundtrip_and_budget`（`kv_zip_1064.rs:866`） | 33.414 | A2R ZIP合法内容/预算及真实编译执行 | C |
| S-09 | `kv_zip_1064::v03_zip_a2py_roundtrip_and_budget`（`kv_zip_1064.rs:871`） | 33.149 | Python ZIP合法内容/预算及真实执行 | C |
| S-10 | `kv_zip_1064::v03_kv_a2py_roundtrip_and_budget_4096`（`kv_zip_1064.rs:499`） | 31.159 | Python KV roundtrip、4096边界及磁盘字节 | C |
| S-11 | `mir_tier::mir_admission_module_tables_over_budget_is_resource`（`mir_tier.rs:91`） | 30.228 | 7万不同常量、精确TableEntries资源拒绝，保合法邻例 | R |
| S-12 | `stdlib_sys_973::sys973_a2r_real_rustc_leg`（`stdlib_sys_973.rs:608`） | 28.293 | 原sys系统接口源码的真实rustc编译/运行与独立结果 | C |
| S-13 | `kv_zip_1064::v03_kv_wire_oracle_and_reopen_av`（`kv_zip_1064.rs:221`） | 26.976 | 独立ACE-KV/1字节、run=100/reopen=200、清空后头字节、无临时残留 | D |
| S-14 | `stdlib_sys_973::sys973_a2py_real_python_leg`（`stdlib_sys_973.rs:701`） | 26.588 | 原sys系统接口源码的Python真实运行与独立结果 | C |
| S-15 | `bstr_1064::bstr_1064_v01_decode_utf8_and_region`（`bstr_1064.rs:300`） | 26.544 | UTF8/region原陷阱和值100、512/1100字节多块预算，原内部10秒门 | R |
| S-16 | `sys_d01_d06_1064::v02_d01_d06_a2r_cross_end`（`sys_d01_d06_1064.rs:446`） | 21.394 | 原D01-D06文件/目录/路径接口A2R真实执行及各独立预期 | C |
| S-17 | `v08_surface_972::v08_surface_manifest_and_five_end_exercise`（`v08_surface_972.rs:348`） | 19.869 | 公开表面manifest和五端行为=62、各真实产物/编译/执行 | C |
| S-18 | `sys_d01_d06_1064::v02_d01_d06_a2py_cross_end`（`sys_d01_d06_1064.rs:485`） | 19.865 | 原D01-D06接口Python真实执行及独立预期 | C |
| S-19 | `json_975::json975_char_to_str_a2x_ends`（`json_975.rs:303`） | 19.384 | char到Str和JSON相关各目标结果、实际目标生成执行 | C |
| S-20 | `module_packages_960::packages_960_four_ends_run_public_str_and_recover`（`module_packages_960.rs:145`） | 18.905 | 跨包public Str四端与失败恢复后的首次消费 | C |
| S-21 | `stdlib_core_945::stdlib_core_945_four_ends_run_math_and_text_slices`（`stdlib_core_945.rs:207`） | 17.565 | 四端math/text slices各原输入与独立结果 | C |
| S-22 | `av_source_incremental::av_source_incremental_detector_negative_controls`（`av_source_incremental.rs:549`） | 15.021 | 正常fast edit、强制rewarm值仍对但身份红、同hash换Arc检测、原findings>=100 | R |
| S-23 | `plan966_ct_product::p966_ct_generated_declarations_change_end_results`（`plan966_ct_product.rs:265`） | 14.412 | CT生成声明变化真正影响各后端结果，不能缓存旧生成体 | C |
| S-24 | `plan966_ct_product::p966_ct_statements_and_macro_run_on_av_native_rust_c_py`（`plan966_ct_product.rs:138`） | 14.396 | CT statements/macro在原五端真实运行和独立预期 | C |
| S-25 | `kv_zip_1064::v03_kv_corrupt_io_budget_and_atomic_preserve_av`（`kv_zip_1064.rs:310`） | 14.356 | 损坏/IO/预算错误精确判决、原文件原子保留与恢复 | D |
| S-26 | `bstr_1064::bstr_1064_t01_illegal_scalar_neighbor`（`bstr_1064.rs:203`） | 14.096 | 0xD800与0x110000非法码点均CharInvalid trap，不能任意错误代签 | D |
| S-27 | `stdlib_sys_973::sys973_v03_process_spawn_capture_timeout_exit`（`stdlib_sys_973.rs:426`） | 13.817 | 真实spawn/capture/timeout/exit与进程收尾，时限不放宽 | D |
| S-28 | `stdlib_sys_973::sys973_t08_seek_permissions_mtime`（`stdlib_sys_973.rs:478`） | 13.779 | 真实seek/权限/mtime OS结果，保持平台条件 | D |
| S-29 | `stdlib_assembly_1043::stdlib_assembly_1043_selection_runs_on_av_native_rust_and_c`（`stdlib_assembly_1043.rs:157`） | 13.484 | stdlib选择与AV/native/Rust/C真实链和各目标结果 | C |
| S-30 | `stdlib_asm_972::stdlib_asm_972_hash_domains_av`（`stdlib_asm_972.rs:929`） | 12.967 | codepoint/byte hash、空/ASCII/Unicode/NUL原hash值与域区分=1111 | D |
| S-31 | `sys_d01_d06_1064::v02_d01_d02_file_text_range_and_dir_names_av`（`sys_d01_d06_1064.rs:78`） | 12.843 | 文件text范围/目录名精确内容与边界，不仅无错误 | D |
| S-32 | `console_coverage_1041::vcoverage_1041_same_source_runs_on_a2r_c_and_a2py_with_provider_identity`（`console_coverage_1041.rs:146`） | 12.394 | 同源A2R/C/Python各运行结果与provider身份 | C |
| S-33 | `stdlib_sys_973::sys973_path_algorithms`（`stdlib_sys_973.rs:396`） | 12.359 | 原路径算法正反例，保精确字符串/平台边界 | D |
| S-34 | `stdlib_sys_973::sys973_env_set_remove_real_process`（`stdlib_sys_973.rs:306`） | 12.343 | 真实进程环境设置/删除及原隔离与恢复 | D |
| S-35 | `kv_zip_1064::v03_zip_negative_corrupt_bomb_and_preserve_av`（`kv_zip_1064.rs:725`） | 12.279 | 坏ZIP/解压炸弹/预算分类与原文件保留，不能错误当空成功 | D |
| S-36 | `stdlib_sys_973::sys973_time_real_clock_bands`（`stdlib_sys_973.rs:367`） | 12.217 | 真实时钟原范围关系/单调性，禁止增sleep或放宽带宽达标 | D |
| S-37 | `b1_slice_975::json975_char_to_str_b1_slice_oracles`（`b1_slice_975.rs:11`） | 12.204 | 原B1切片/char到Str独立oracle，不能与跨端大例互相代签 | D |
| S-38 | `stdlib_sys_973::sys973_dirs_and_replace`（`stdlib_sys_973.rs:331`） | 12.133 | 真实目录与替换/失败保留语义、清理 | D |
| S-39 | `stdlib_sys_973::sys973_t09_target_selection_linux_binds`（`stdlib_sys_973.rs:522`） | 12.064 | Linux目标sysplat_lnx绑定/冷link；原生运行NOT_RUN owner1063，不称Windows Linux执行通过 | L |
| S-40 | `stdlib_sys_973::sys973_t10_rand_real_entropy`（`stdlib_sys_973.rs:560`） | 11.979 | 真实安全熵长度/字符集/两次差异，不能固定假随机替代 | D |
| S-41 | `stdlib_sys_973::sys973_v04_short_read_and_fault_injection_no_false_success`（`stdlib_sys_973.rs:235`） | 11.864 | 短读与故障注入不误报成功、精确错误与恢复 | D |
| S-42 | `module_sources_950::modules_950_user_shadow_consumer_restore_first_edit_and_next_read`（`module_sources_950.rs:424`） | 11.844 | user shadow消费恢复后首次编辑和下一次读，真实身份/依赖 | R |
| S-43 | `stdlib_sys_973::sys973_v05_clock_rand_oracles`（`stdlib_sys_973.rs:266`） | 11.498 | clock/rand原可辨负控，合法值和退化值不能同绿 | D |
| S-44 | `stdlib_sys_973::sys973_v03_process_exit_outcome`（`stdlib_sys_973.rs:451`） | 11.431 | 真实子进程退出outcome及收尾，不只parent exit0 | D |
| S-45 | `module_sources_950::modules_950_qualified_consumer_restore_first_edit_and_next_read`（`module_sources_950.rs:420`） | 10.970 | qualified消费恢复后首次编辑/下一读、真实依赖与身份 | R |
| S-46 | `stdlib_core_945::stdlib_core_945_clone_allocation_fault_is_a_typed_resource_trap`（`stdlib_core_945.rs:440`） | 10.860 | clone分配故障必须typed Resource trap，原释放/恢复断言不丢 | D |
| S-47 | `v10_layers_972::v10_core_alloc_layers_mn`（`v10_layers_972.rs:202`） | 10.706 | CORE零alloc/attempt/live且exit1111121，ALLOC正常平衡与预算/释放原反控 | R |

以上是拟处置，不是已切换。继续日常行完整test≤10秒；分流保原完整义务/原条件。S-05原内部每个target_bytes≤10秒，但整test历史41.758秒，新门看整test。S-22原`findings.len()>=100`及同hash换resident对象的指针/代际反控是已有结构/规模义务；日常小例不能把阈值改≥1代签旧例，原≥100反控仍须完整执行。普通正确性在保持原角色后仍不能满足预算，本项保持未完成，不自动分流。

### 5.4 准备、并发与入口切换

1. **显式准备**生成所选完整配置的发现清单、测试二进制/必要归档、worker/runtime、工具/内容hash、有效环境、平台/features/profile/source_state及成本。复用现有prepare_tools收据，不用exe存在/mtime代身份。Case输出准入需要的case-parent也先准备，`run_case`不得暗补。
2. **只运行**时缺manifest/错树/缺库、源/config/toolchain/profile/features/env/产物或sidecar变化，测试体前具名准备错误；零lazy build/下载/release/旧exe回落。Auto编译器被测本体的parse/bind/emit/link允许并计时；外部rustc/C认证只在显式toolchain被测动作里允许，其依赖仍不得体内Cargo构建。
3. 指纹沿用固定调用与实际输入身份；宁可明确要求重准备，不删轴提升命中。跨次证据复用须同身份/覆盖，无跨case可变session/store共享，不借其他计划target。
4. 准备/联合测试前核agent协调信息、现有`execution_lease()`及真实Cargo/rustc/nextest/worker活动。其他会话忙则等待并通知，记录等待；不杀未知进程，不把自己已持有的准备→运行所有权误判他人。跨worktree互斥需要时扩展已有设施，不造第二调度平台。等待单列，不隐藏在组计时外不汇报。
5. 当前`cargo t`薄驱动经`cargo run`启动，首次会编译。严格无编译执行须先准备launcher，必要时用同一已准备launcher或`python scripts/fixed-tests.py`进入同一实现。不能声称别名冷启动也零编译；所有准备成本可见。
6. 切换按“冻结旧义务→新路径负控敏感→同输入对照→切固定组/脚本/聚合→真实入口验收→撤重复活入口”。普通PERF差集恰为P-01/P-02；C类保每目标动作/条件；console的15项语料和附加宿主断言各有owner。旧→新身份映射覆盖全部角色，少目标/错目标/emit冒run不能绿。
7. 同key OR并集沿用原实现；不同feature/profile/env/tools不按同名test强合。切换失败恢复该阶段旧实现/选择，不改M1历史证据，不留双活作者/第二默认runner。Windows不触发WSL/Linux腿；S-39仅绑定事实单列，原生Linux仍由1061/1063验收。

### 5.5 T-01 冻结基线（r1 首轮，只读核查，未执行任何组）

实施树 `D:/autostack/.wt/lang-1098/auto-lang`、分支 `plan-1098-dev`，基面 `d20cf3f87`（建树时 master；建树后 master 被并发会话推进到 `37379e6d4`，合入前重同步）。材料 scope `D:/autostack/plan-artifacts/plan-1098/`（空，仅登记）。本轮**未**构建/未跑组：开始时刻机器上有其他会话的活跃 `cargo/rustc/nextest`（14:44–14:45 启动），按 §5.4-4 不并发启动准备/运行。主检出代码路径（`crates/ scripts/ experimental/ test/ .cargo/ Cargo*`）无 WIP；仅 docs/ 与其他会话的 `packages/blueprints/**` 删除，非本包写集。

| # | 冻结项 | 实测（树/配置身份） | 结论 |
|---|---|---|---|
| K-01 | ACE 域固定组分母 | `scripts/test-groups.toml`（`090c0e71021f8cd2…`）166 组，`domain=ace` **102**：nextest **27** / python **2**（`infra-pilot-ace`、`plan1066-s-doc-controls`）/ case **73** | 与 §4.2 完全一致；24 个非聚合常规组 = 27 − `ase-m0`（外部）− `ace-perf`（显式 PERF）− `ace-workspace`（聚合） |
| K-02 | P-01/P-02 泄漏路径 | `crates/syntax-core/tests/conformance.rs:14` `mod perf;`；普通 `ace-syntax` 表达式 `(all()) and not test(/(^|::)performance::/)`；`ace-perf` 表达式 `test(/(^|::)performance::/)`、`timeout_seconds=14400` | 结构性确认：`perf::perf_fresh_parse…` / `perf::perf_core_vs_legacy…`（`cases/perf.rs:41,175`）**既不匹配 PERF 选择、也不被普通组排除**→ 现留在日常组。真实发现差集（32 样本/8,000 声明等正文不动）由 T-03 构建后发现，不在本轮声明 |
| K-03 | B-05 工具 profile 表 | `scripts/test-governance/toolchains.json`（`177710e13656ddd4…`）15 个工具：`av,batch-av,av-regular,ac,mn,mn-collections,cte,abi,sema,a2x`=`release`；`text-runtime,data-runtime`=`runtime-release`；`trace-runtime`=`runtime-dev`；`case-parent,execution-fixture`=`debug` | 与 §5.2 B-05 逐项一致；普通/正式 profile 分离仍待 T-02 实现 |
| K-04 | B-03 测试体补 release | `experimental/ace/tests/conformance/tests/host/rust_lowering.rs:50-56`（无 `AC_A2X_HIR_WORKER` 时"build the release worker now"）、`:202` `--release`、`:228-231` 回落 `experimental/ace/target/release` | 确认存在；`a2r_hir_missing_worker_is_infra_error:112` 是该路径的负控，T-02 保留其"缺工具=infra 失败"义务 |
| K-05 | F-01 夹具形态 | `crates/av/tests/cases/mir_tier.rs:97` `for i in 0..70_000i32`，`:130` 断言 `ResourceKind::TableEntries` | 确认 7 万不同常量 + 精确资源门；优化点=避免逐次线性 intern，尺度不缩 |
| K-06 | O-01 双分支打印即绿 | `crates/av/tests/cases/p946_t01_probe.rs:118-130` `Ok(out)=>println!` / `Err(e)=>println!`，无断言 | 确认无 oracle；O-05（`:275` 起）另需独立核对打印/断言比例 |
| K-07 | H-01…H-03 宿主文件 | `experimental/ace/tests/conformance/tests/console.rs`：`console_real_corpus_and_stale_identity:33`、`console_three_oracle_axes_fail_independently_and_missing_axes_error:86`、`console_skip_remains_skip_and_never_creates_a_green_observation:134`、`console_nul_bytes_survive_worker_transport_and_parent_judgement:150` | 行项存在。**新增发现**：同文件 `:197 console_resource_exhaustion_is_error_with_raw_prefix_evidence` 不在 §5.2 H-02 列，T-07 逐行展开时须纳入（缺轴/原始前缀义务） |
| K-08 | §4.2 两个旧红归属 | 实测位于 `experimental/ace/crates/av/tests/cases/host_console.rs:12`、`:23`（package `ace-av`），不在 `tests/conformance` | 归属为 `ace-av` 组；完整 nextest 身份待真实发现（T-01 后续）确认，不按名字代签 |
| K-09 | nextest 预构建接口 | `cargo-nextest 0.9.138`（x86_64-pc-windows-msvc）：**无** `--no-build`；`list`/`run` 均支持 `--archive-file`、`--archive-format`、`--binaries-metadata`、`--workspace-remap` | T-02 的 prepare/run 分离接口**定为 archive 握手**（准备产出 archive 清单+身份，运行只消费它），不假定不存在的参数；零编译由 T-02 负控实证 |
| K-10 | 并发互斥既有设施 | `scripts/fixed-tests.py:372-446` `execution_lease()`：`target/test-groups/active` 原子 mkdir + `owner.json{pid,root}` + 死锁回收（PLAN-1014 B-07） | V-13 扩展现有租约，不建第二调度平台；等待单列 |
| K-11 | 47 行源码存在性 | 抽查 `bstr_1064.rs`、`mir_tier.rs`、`probe972.rs`、`v10_layers_972.rs` 均在 `crates/av/tests/cases/` | 存在性通过；逐行角色/预期展开仍待 T-01 后续轮次（不得以声明数代执行数） |

本轮仍**未**关闭任何 T-01 出口项：逐行（P/B/F/O/H + 47 行）义务映射、真实发现差集、两个旧红的重放判定、Case 消费者组核对尚未完成，且未构建。下一步仍在 T-01 内（先等并发会话空闲再取代表基线）。

### 5.6 T-01 第二轮：真实重放与逐行静态核查（2026-11-19，树 `d20cf3f87`，debug）

**重放（本轮唯一执行）**：worktree `experimental/ace` 内 `cargo nextest run -p ace-av --test conformance -E 'test(host_console::)' --no-fail-fast --status-level all`。观测（冷 debug，本树首次构建）：**准备/编译 61s**、**执行 0.280s**（21 例 19 绿 2 红）、整命令 **62.7s**、`exit=100`。原始日志 `D:/autostack/plan-artifacts/plan-1098/t01-av-console-replay/run.log`。该 1:218 的准备/执行比正是 §2 要拆开的两段，不是"测试慢"。

两个旧红（§4.2）**首红复现**，完整身份与实测：

| 完整 nextest 身份 | 结果 | 失败点 | 根因（实证） |
|---|---|---|---|
| `ace-av::conformance host_console::av_console_entry_selection_cannot_bypass_main_policy` | FAIL 0.092s | `host_console.rs:18` `.err().unwrap()` 取到 `Ok` | `ace_av::console_host::compile("…d_helper")` 不再拒 |
| `ace-av::conformance host_console::av_console_owned_front_door_enforces_console_entry_only` | FAIL 0.049s | `host_console.rs:28` `.unwrap_err()` 取到 `Ok`（已降级出 `MirModule{profile:"av.host-console/1", name:"d_helper"}`） | `lower_checked_owned(…, "d_helper", 1)` 不再拒 |

**判定：不是产品回归，是陈旧 oracle。** `3a9d9c1fb`（2026-11-18，PLAN-982）"the console lane admits named entries (d_fn) …entry shape stays gated by verify" 已把入口面从"只许 d_main"改为"具名入口 + verify 守形状"；`crates/lowering/src/hir_adapter.rs:1535-1575` 现在只为"模块内查不到该名"报 `lower.entry`。两个 863 测试仍断言旧策略。按仓库规则**不得单方面改预期**，故具名移交 `D-1098-01`（见 `docs/plans/KNOWN-DEBT-AND-RISKS.md`），不充 PASS、不 `#[ignore]`、不用成本绿代功能绿。

**逐行静态核查新增结论（§5.2/§5.3）**：

- **B-01**：缺库即 `cargo build --locked --profile runtime-dev`，`crates/native/tests/common/mod.rs:19-40`（target-dir `../../target`）；collections 侧同型但 `--profile runtime-release`，`crates/native/tests/common/collections.rs:31-46`。
- **B-02**：`crates/case-worker/tests/cases/backend.rs:204-223` **另抄了一份**同样的"缺库即 cargo build"——同一反模式实测 **4 处**（native common ×2、case-worker ×1、core common ×1），不是 2 处；`exit=12`/stderr`ba` 的参数求值顺序义务在 :224-229，保留。
- **B-04**：`crates/core/tests/common/mod.rs:293-350` 的 `text_runtime_lib/text_runtime_pair_copy/text_runtime_sha` 全仓**无测试消费者**（其余命中均为产品符号 `ace_av::text_runtime::*`）→ 死 helper；清理不提"实测收益"。
- **C 类判据实证**：`crates/av/tests/cases/console_coverage_1041.rs:98 run_rust_stdout` 真跑 `rustc --edition 2021` 并执行产物；`:128 run_c_stdout` 走 `ace_a2x_worker::crun::compile_c_fileset`（真 C 编译）。同类 helper 在 `plan966_ct_product.rs:71`、`stdlib_assembly_1043.rs:90`、`module_packages_960.rs:79,132`、`stdlib_core_945.rs:190`、`module_sources_950.rs:60` 各自复制 → C 类"多端"义务集中在少数 helper，可按端映射进 `ace-toolchain`。
- **P-01/P-02 原条件**：`cases/perf.rs` `N=32`、3 预热、`build_doc(8000)`；P-02 6,000 函数、CST vs legacy 双链、P50/P95/P99——与 §5.2 完全一致，不得缩。
- **H-04/H-05**：便宜日常例 `a2r_hir_first_case_green`（5 注册例）+ `a2r_hir_tampered_golden_is_red`；其 release worker 维护构建在 `rust_lowering.rs:56-58`（B-03）。certify 族逐例 PASS + `cert_sha256` + 每 leg `<120_000ms`（:326-360），负控 14→15 与错 reference（:366-379）。
- **47 行静态分档（候选，非最终处置）**：47 行全文按"是否触外部链 helper"分档，产物 `D:/autostack/plan-artifacts/plan-1098/t01-47-rows-static-triage.md`（SHA256 `314539e1…0f68`）。候选：C 触外部链 21 行、D/R 纯 AV 内部 20 行、thin-wrapper 待展开 5 行、L Linux 绑定 1 行（S-39）。**分档只看文本标记，不等于逐行处置结论**；每行仍须展开角色/独立 oracle/条件/消费者后才能勾 T-01。

**T-01 仍未闭**：47 行逐项展开（角色/预期/条件/消费者）、24 组真实发现分母与 >10s 项、`ace-toolchain` 的目标清单与 Case 消费者核对、两个旧红的 owner 答复（D-1098-01）。

### 5.7 T-01 第三轮：47 行结构化展开（只读，未跑组）

**方法**：对 §5.3 全部 47 行按完整原生身份定位 `#[test]` 函数体，用**跳过字符串/raw-string/注释的花括号匹配**取体（内嵌 Auto 源码自身含 `fn`，简单按 `\nfn ` 截断会把体算成 3–15 行），再提取 oracle 机制（`expect_i32`/`assert*`/`unwrap_err`）、本地助手链（深度≤3）、端助手（`run_py_fileset`/`resolve_python`/`run_rust_stdout`/`run_c_stdout`/`compile_c_fileset`/`run_a2r`/`run_a2py`/`native_exit`/`native_value`/`c_stdout`/`rust_stdout`/`run_av`/`mn_run`/`run_case`）与时间门。产物：`D:/autostack/plan-artifacts/plan-1098/t01-47-rows-expanded.md`（SHA256 `a9250286…`）+ 抽取脚本 `extract_rows.py`（`d986d9e7…`）。

**结构性结论（供 T-03…T-07 记账，不是处置结论）**：

1. **47 行不是 47 段独立逻辑**：多数是 3–15 行 `#[test]` 外壳，真实多端逻辑在同文件本地助手里；且**一个助手服务多行**（`kv_zip_1064::run_kv_cross_backend_leg(…)` 同时服务 S-07 kv-a2r 与 S-10 kv-a2py；`run_zip_cross_backend_leg` 同时服务 S-08/S-09）。分流必须按"助手 × 端参数"记账，否则同一义务会被记两次或整段漏掉。
2. **oracle 不是"打印即绿"**：D/R 类普遍用 `expect_i32(out, N, tag)` 精确值（S-33 期望 11、S-27 期望 42、S-28/S-36/S-38/S-40 同型）或 `assert_eq!`；至今唯一确认的"双分支打印即绿"仍是 §5.2 O-01。据此，S 行不得以"探针"名义批量退役。
3. **端助手高度集中**：`run_py_fileset`/`resolve_python`、`run_rust_stdout`、`compile_c_fileset`、`native_exit`、`c_stdout`/`rust_stdout`——这些正是 `ace-toolchain` 的接缝；T-06 应按"助手×端"切，而不是按测试名。
4. **抽表局限（须人工确认）**：直接 `Command::new(RUSTC|cl|python)` 的行不被"端助手"列捕获（实证：S-12 `stdlib_sys_973.rs:666,681` 直接起 rustc；S-01 的 A2R 端同理），所以"端助手列为空"**不等于**无外部链。
5. **既有时间门**：仅 S-02/S-25 带 `Duration::from_secs(8)` 的 AV wall 预算（`kv_zip_1064.rs:99`），历史慢点在准备/IO 而非断言；`resources_tier`（S-04）另有 `from_secs(5)`/`120ms` 观测门。

**T-01 未闭项（本轮推进但未完成）**：24 组真实发现分母与 >10s 项（本轮机器被并发会话占用，见 §9）；`ace-toolchain` 目标清单（静态已定：`-p ace-av --test conformance` 承载 S 行，`-p ace-conformance --test conformance` 承载 H-05/host 行）；Case 消费者四套件静态存在（`ace-document-console.suite.ad`、`ace-document-base.suite.ad`、`ace-m1/ace-m1-tr-native.suite.ad`、`ace-m1/typed-native-891/typed-native-891.suite.ad`），未做 inventory 计数。

### 5.8 T-01 第四段：24 组包/target/feature 差集静态冻结（只读）

按 `scripts/test-groups.toml`（`090c0e71…`）与 `experimental/ace/Cargo.toml` 成员清单逐组解析（脚本 `extract_groups.py`，产物 `t01-24-groups-targets.md`）。24 个非聚合 ACE nextest 组的真实形状：

| 形状 | 组 | 说明 |
|---|---|---|
| 单 test 二进制 `-p X --test conformance`（无 `--lib`） | `ace-source-syntax-927`(core)、`ace-python`(abi)、`ace-host`/`ace-console`/`ace-batch`/`ace-rust-lowering`(conformance)、`ace-stdlib`(stdlib-conformance) | 7 组只跑一个 test target；`ace-python` 另有 `-E test(p1026)` |
| `--lib --tests` 多包 | `ace-source`、`ace-syntax`(4 包)、`ace-semantics`(4 包)、`ace-artifacts`、`ace-av`(2 包)、`ace-native-worker`、`ace-native`、`ace-abi`、`ace-comptime`(4 包)、`ace-lowering`、`ace-foreign-contract`、`ace-a2x`(6 包)、`ace-runtime`(2 包) | 13 组含 lib 单测 + 全部 test target |
| `--lib --tests` 多 test target | `ace-m2-build` | 8 个 test target（b0_closure/compat_selection/coverage_lanes/identity_cache/load_selection/restore_recovery/sdk_pack/…），组名易被误读为单例 |
| 无 test target（仅 lib） | `ace-a2c-abi`（`ace-a2x-c`）、`ace-adapters`（`auto-lang-core --features code-editor`）、`ace-facade`（`auto-lang-ace-host`） | 后两者跑**根 workspace**，不是 `experimental/ace`；`ace-a2x-c` 无 tests 目录 |
| 共享同一 test 二进制 `-p ace-conformance --test conformance` | `ace-host`、`ace-console`、`ace-batch`、`ace-rust-lowering` | 只靠 `--features`/`tools`/`expression` 区分 |

**H-06（同 key OR 并集）静态实证**：`ace-host` 与 `ace-console` 的 9 元 key（runner/workspace/args/domain/tools/file/suite/respect_default_filter/timeout）**完全相同**（tools 同为 12 项、args 同），只有 expression 不同（host=`all()`，console=`^console::`）→ `invocations()` 必须合并为一次调用（现有实现如此，实测配置可证）。`ace-batch` 多 `--features test-case-batch`、`ace-rust-lowering` tools 仅 `a2x` → 两者 key 不同，**不得**强合。这三条与 §5.2 H-06/§6.1 V-10 一致，且可纯静态复核。

### 5.9 T-01 出口项：准备/运行接口草案（已核 CLI 事实，实现属 T-02）

**已核事实（`cargo-nextest 0.9.138`，本机实测 help）**：无 `--no-build`；存在 `cargo nextest archive --archive-file <PATH>`（构建 test 二进制并打成 `.tar.zst`），`list`/`run` 均支持 `--archive-file`、`--workspace-remap`、`--binaries-metadata`。→ 准备/运行分离**走 archive 握手**，不假定不存在的参数。

**接口草案（T-02 按此实现，实际 manifest 名在 T-02 钉住）**：

- `scripts/fixed-tests.py <groups> --prepare-only --out <dir>`：对 `invocations()` 合并后的**每个 key**（runner/workspace/args/domain/tools/file/suite/timeout 的并集成员）执行一次 `cargo nextest archive --archive-file <dir>/archive-<n>.tar.zst`，选择面沿用该 key 的 `-p/--lib/--tests/--test/--features/--target-dir`（不含 `-E`，表达式属运行期）；工具仍走现有 `prepare_tools.prepare()`（receipt 复用已按 source_state+rustc+tool config+target 身份校验并逐件 re-hash）。产出 `prepared.json`：`schema`、`config_sha256`、`source_state`、rustc marker、每个 key 的 archive path+sha256、工具 receipts、生成时间与 driver 版本。
- `<同命令> --run-prepared <prepared.json>`：先逐项校验 identity（source_state、config、toolchain、archive sha256、工具 receipts），任一漂移 → **具名准备错误，零回落、零补构建**；通过后 `cargo nextest run --archive-file <a> --workspace-remap <checkout> --ignore-default-filter -E <并集表达式> --no-fail-fast --no-tests fail`（`-E`/`--run-ignored`/`respect_default_filter` 与 archive 的组合由 T-02 负控实证，不预设）。
- 日常入口按 §2：`cargo t <ACE组>` 未找到本树匹配 prepared.json 时返回 `PREPARATION_REQUIRED` + 显式准备命令；显式 `--manifest` 消除候选歧义。

**可复用面（决定 archive 数量，不是新注册表）**：`ace-host` 与 `ace-console` 同 key → 同一 archive；`ace-rust-lowering` 与 host 同包同 features（仅 `-E`/tools 不同）→ 可复用同一 archive；`ace-batch` 因 `--features test-case-batch` 独立 archive；`ace-facade`/`ace-adapters` 在根 workspace → 各自独立 archive；其余组各自一个。

**T-02 必须负控的风险面**：archive 重映射后仍依赖 `CARGO_MANIFEST_DIR`/仓库相对路径的用例（如 S-12 直接起 rustc、B-01 的 trace-runtime 构建路径、`scratch()` 临时目录）须在真实链上证明行为不变；否则该组退回"显式准备但运行期直取 test 二进制"的等价形态，并在 SD-01 写明。

**实证（本轮已完成，`ace-lowering` 真实链）**：

```
cargo nextest archive -p ace-lowering --lib --tests --archive-file <scope>/archive-spike/ace-lowering.tar.zst
  → exit 0，18.75s（含编译；产出 4,563,639 B，tar.zst，sha256 ed97df0d…）
cargo nextest run --archive-file <…>.tar.zst --workspace-remap <worktree>/experimental/ace \
  --archive-format tar-zst --no-fail-fast --no-tests fail
  → exit 0，**9/9 PASS，0.34s**（零构建）
cargo nextest run … --ignore-default-filter -E 'test(closures_lowering::)' …
  → exit 0，**3/3 PASS，0.39s**，6 skipped（表达式在 archive 模式下生效）
```

即 §2/§5.4 要的"准备（可观测、可复用）→ 只运行（零编译）"在真实 nextest 上成立；准备/运行比 18.75s : 0.34s。

### 5.10 T-01 真实发现分母（第一批，`--discover-only`，不执行 body、不准备 release 工具）

| 组 | 选择 / 排除 | 二进制数 | 观测 | 材料 |
|---|---:|---:|---|---|
| `ace-av` | **1,428 / 0** | 8（`ace-av::conformance` 1,277、`ace-av-core` 69、`ace-av-core::conformance` 48、`ace-av` 17、`ace-av-core::foreign_resources` 11、`ace-av::bin/av-worker` 3、`ace-av-core::p982_config_super_put` 2、`ace-av-core::p982_console_stdin_contract` 1） | 墙 **56.378s**（含构建），CPU 190.2s，峰值 Job 3.50GB；`source_state 6371cd89…` | `discover-ace-av/results.json` |
| `ace-lowering` | 9 / 0 | 2 | 三组合计 50.95s | `discover-small-3/results.json` |
| `ace-artifacts` | 43 / 0 | 3（`ace-artifacts` 19 / `::conformance` 21 / `::store_resources` 3） | 同上 | 同上 |
| `ace-semantics` | 295 / 0 | 19（其中 `ace-source-query` 104、`ace-semantics-store::conformance` 65） | 同上 | 同上 |

历史收据（§4.2）的 `ace-av` 1,428 执行数与本次**发现**数一致 → 分母稳定；本轮只发现不执行，`>10s` 项仍须 T-02/T-09 在只运行形态下重测。

**第二批（`ace-conformance` 共享二进制的 4 组，本轮实测）**：

| 组 | 选择 / 排除 | 观测 |
|---|---:|---|
| `ace-host` + `ace-console` | **171 / 0**（**合并为 1 次调用**，见 §5.8 H-06） | 4 组整批墙 **199.06s**（含 conformance target 的两种 feature 构建） |
| `ace-batch` | 12 / 0 | 独立调用（`--features test-case-batch`，tools=2） |
| `ace-rust-lowering` | 6 / 0 | 独立调用（tools 仅 `a2x`） |

**H-06 由静态推断升级为实测**：日志逐字显示 `Running ace-host, ace-console`（一次），`results` 只有 3 条 = 4 组；合并表达式为 `(all() and not performance::) or (^console:: and not performance::)`。另证 `ace-console` 的选择面是 `ace-host` 的真子集（`all()` 覆盖），即两者的日常义务在合并调用里同时满足，但**组预算仍须按组各记**（§6.2 不变）。

**第三批（其余 10 个中等组，`discover-batch3/`，整批墙 212.33s）**：

| 组 | 选择/排除 | 二进制数 |
|---|---:|---:|
| `ace-source-syntax-927` | 13 / 0 | 1（`ace-core`） |
| `ace-source` | **904 / 0** | 1（`ace-core`） |
| `ace-syntax` | 159 / 0 | 4（syntax-runtime 52 / grammar 45 / core 31 / artifacts 31） |
| `ace-abi` | 314 / 0 | 1 |
| `ace-comptime` | 136 / 0 | 2（comptime 85 / source-cte 51） |
| `ace-stdlib` | 2 / 0 | 1 |
| `ace-runtime` | 84 / 0 | 2（data-conformance 64 / text-conformance 20） |
| `ace-foreign-contract` | 56 / 0 | 1 |
| `ace-python` | 10 / 0 | 1（`ace-abi`，`-E test(p1026)`） |
| `ace-a2c-abi` | 9 / 0 | 1（`ace-a2x-c` lib 单测；确认"只有 lib"但有 9 例） |
| **小计** | **1,687 / 0** | — |

已发现合计（5 批）：`ace-av` 1,428 + 4 组 347 + conformance 4 组 189 + 本批 1,687 = **3,651 例**。

**第四批（重内存 4 组，`discover-heavy4/`，整批墙 39.55s）**：

| 组 | 选择/排除 | 说明 |
|---|---:|---|
| `ace-native` | **329 / 0** | conformance 258 / sched_m2_1067 28 / typed_entry_tier 18 / lib 16 / bin mn-collections-worker 5 / typed_entry_incremental 4 |
| `ace-a2x` | **328 / 0** | a2x-worker::conformance 272 / a2x-ts 20 / bin a2x-worker 8 / 其余多目标 |
| `ace-native-worker` | 43 / 0 | conformance 39 / bin 4 |
| `ace-m2-build` | 30 / 0 | 7 个 test target（sdk_pack 8 / load_selection 6 / compat 5 / identity 4 / b0 3 / coverage 2 / restore 2） |
| **小计** | **730 / 0** | — |

**累计已发现 4,381 例 / 22 个非聚合组**；余 `ace-facade`、`ace-adapters` 两个根 workspace 组，聚合 `ace-workspace` 与 `ace-perf` 另窗。

### 5.13 T-02 前置核查：B-05 的**真实消费面**（比计划原假设大得多）

对 `experimental/ace` 全量 Rust 源码静态扫描（`target/release|runtime-release`）：

- **37 处硬编码引用，分布在 20 个文件**；其中 **11 处会实际触发构建**（字面量 `"runtime-release"`）：`tests/text-runtime/src/lib.rs`、`tests/data-runtime/src/lib.rs`、`crates/native/src/{collections_runtime,sched_runtime,typed_runtime}.rs`、`crates/native/tests/cases/collections_resources_tier.rs`、`crates/native/tests/common/collections.rs`、`crates/core/tests/common/mod.rs`、`crates/av/src/text_runtime.rs`、`crates/abi/src/source_foreign.rs`。
- **5 处是"lib 级 lazy builder"**（不是测试体）：`crates/av/src/text_runtime.rs:13 text_runtime_lib()`、`crates/abi/src/source_foreign.rs:~1475`、`crates/native/src/collections_runtime.rs:233`、`crates/native/src/sched_runtime.rs:292`、`crates/native/src/typed_runtime.rs:228`。它们被产品路径调用（`native/{console,driver,worker}.rs` 等），在测试里表现为**运行期 cargo build**。
- **`collections`/`sched`/`typed` 三个 runtime 在 `toolchains.json` 里没有任何条目** → 今天即使 `--prepare-only` 也无法预建，"零编译运行"在这些组上不可能成立。计划原文只列了 B-01/B-02/B-04 三处测试体补构建，**低估了 5 处 lib 级构建 + 3 个缺失工具条目**。

**B-05 的可行改法（T-02 分步，不改产品语义）**：

1. `toolchains.json` 新增 `collections-runtime`/`sched-runtime`/`typed-runtime` 条目（profile 按 §3 的 no_std 裁定先维持现状并在 SD-01 记录），`text/data` 由 `runtime-release` 改 `runtime-dev`（与 `trace-runtime` 一致）。
2. 5 个 lib builder 改为 **env 优先**：测试准备注入（如 `ACE_TEXT_RUNTIME_LIB` 族）时读取并校验存在性/哈希，**缺失即具名 infra 失败、不回落构建**；env 未设（产品/开发直用）保持现有 lazy build——"日常零编译"由 env 保证，产品行为不变。
3. 硬编码 `target/release/<worker>` 的测试（`a2x-worker/tests/cases/{a2c_cert,a2c_coll_cert,a2c_rt_cert,cert_coll_904,cert_rt_902}.rs`、`av/tests/cases/{av_collections_resources,av_dynamic_resources}.rs`）改 **env 优先 + 明确失败**，禁止回落 release 路径。
4. 最后才翻 `av/batch-av/av-regular/ac/mn/mn-collections/cte/abi/sema/a2x` 的 profile 为 debug，并用真实组验证（`ace-abi`/`ace-batch`/`ace-native` 各一次 prepare→run，比较 debug worker 的 item 时间是否仍满足 ≤10s）。

**已排除的误判**：`av_collections_resources.rs:814` / `av_dynamic_resources.rs:723` 的 `target/release` 不是工具准备问题而是**测试自己找 worker**，归第 3 步一并处理。

**关键细节（读实现后修正上表第 2 步的做法）**：这些 lib builder 不是"每次必建"，而是**内容绑定 + receipt 缓存**：`typed_runtime::prepare()` 用 `files/configs/rustc -vV/相关 env` 算 `build_key`，若 `<lib>.identity.json` 的 `build_key` 与 ABI 匹配且 `content_hash` 相符则**零 cargo 调用**；`override_library` 只改路径，**不跳过**身份计算与构建判定（`typed_runtime.rs:107-268`）。因此：

- 仅"补 toolchains 条目 + cargo build"**不足以**让运行期零编译——运行期第一次调用仍会因缺 `identity.json` 再跑一次 cargo。
- 正确做法是让**准备阶段执行 builder 本身**（一次），把 stamp 写出来。已有先例：`mn-collections-worker` 启动即调 `collections_runtime::prepare(None)`（`crates/native/src/bin/mn-collections-worker.rs:258,381`）。
- 拟新增最小 `runtime-warm` bin（每宿主 crate 一个：`ace-native` 调 collections/sched/typed，`ace-av` 调 text/trace，`ace-abi` 调 text），并在 `toolchains.json` 加可选 `"warm": [...]` 字段、`prepare_tools.prepare()` 在构建成功后**执行一次 warm 命令**（计入准备时间与 receipt 身份）。这样"运行期零编译"由准备阶段负责，产品侧的 lazy build 与身份校验保持不变。

### 5.11 T-01 第五段：O 类探针的独立预期（按 Spec 钉住，不录当前输出）

§5.2 的 O 行原来只有"要独立 oracle"的原则；本节把**预期来源**钉住（执行仍属 T-05）：

| ID | 现状（实测代码） | 独立预期来源与内容 |
|---|---|---|
| O-01 `p946_empty_remove_contract_observed` | `p946_t01_probe.rs:118-130` 两分支只 `println!`，无条件断言 | `docs/specs/stdlib/design/ace-collections.md:17`：**空 `remove` 裸形 = trap `list-index`**（`pop` 空=None 只是包装层合同）→ 断言 Trap 且 code=`list-index`，合法邻例（非空 remove、空 pop=None）各自独立断言 |
| O-02 `probe972_t02_str_surface` | `probe972.rs:409-449` 六子场景只 `eprintln!` | 语言/Unicode 事实（非实现回读）：`str_eq`→1、`str_concat`→4、`char_codepoint_cmp`→1、`char_codepoint`→97、`str_slice_len_bytes`→1（"A" 单字节）、`str_char_at`→20013（'中' U+4E2D） |
| O-03 `probe972_p1b2_closure_param_frozen_rejection` | `:237-263` 只断言 `Err`（`Ok` 即 panic）+ 打印 code | **无 spec 声明的精确 code** → 不得把打印值录成 oracle；T-05 必须二选一：转正例（已支持）或由 owner 钉住阶段+code |
| O-04 `probe972_p1c_suffix_list_public_frozen_rejection` | `:269-286` 同上只打印 | Spec 已声明：packages 面在 binder 层拒 `source.module-symbol`（`docs/specs/stdlib/design/ace-collections.md:33`、`docs/specs/auto-ac/source-collections-variants.md:137-138`）→ 断言该码 |
| O-05 `p946_list_argument_moves_single_module` | `p946_t01_probe.rs:275+`，已有返回值 22 断言 + cap 观察打印 | 保留 `==22` 断言；cap 观察段要么给独立 oracle（cap 值/事件），要么具名移 research，不留在日常当装饰 |

即：O-01/O-02/O-04 的预期可完全从既有 Spec/Unicode 事实写出，**不需要**先跑一次再抄；O-03 缺合同须由 T-05 决策。

### 5.12 T-02 首刀：准备/运行分离已实现并实测（2026-11-19，树 `d20cf3f87` + `197ac9e5d`）

`scripts/fixed-tests.py` 新增：

- `--prepare-only --out <dir>`：对 `invocations()` 合并后的每个 key，先走现有 `prepare_tools.prepare()`（工具 receipts 复用），再 `cargo nextest archive` 构建该 key 的 test 二进制归档；产出 `<dir>/prepared.json`（`schema: fixed-test-prepared/1`、`config_sha256`、`source_state`、rustc marker、每 key 的 archive path/sha256/bytes、工具 identity 与 artifact 哈希、`prepared_seconds`）。
- `--run-prepared <prepared.json> --out <dir>`：先逐项校验（配置 sha、`checkout_state()`、rustc marker、每个工具的 `tool_identity`、每个工具 artifact 的 sha256、archive 的 sha256/存在性），任一不符 → `PREPARATION_REQUIRED: …` 且 **exit 2、零构建、零执行**；通过后 `cargo nextest list --archive-file …` + `cargo nextest run --archive-file … --workspace-remap … --archive-format tar-zst --ignore-default-filter -E <expr>`，终态校验沿用原 `enumerate_rust`/`terminal_results`。

**实测（`ace-lowering`，本树）**：

| 步骤 | 命令 | 结果 |
|---|---|---|
| 准备 | `--prepare-only --out t02/prepare-lowering` | exit 0，**1.19s**（目标已建），manifest+`archive-0.tar.zst` |
| 只运行 | `--run-prepared … --out t02/run-lowering` | **exit 0，PASS 9/9，wall 0.721s**（整命令 1.32s），零构建 |
| 负控·源码漂移 | 改 `scripts/fixed-tests.py` 后再 run-prepared | `PREPARATION_REQUIRED: source/input state changed since preparation`，exit 2 |
| 负控·archive 篡改 | manifest sha256 置零 | `PREPARATION_REQUIRED: archive changed since preparation: …`，exit 2 |
| 负控·archive 缺失 | 指向不存在路径 | `PREPARATION_REQUIRED: archive missing: …`，exit 2 |

源码漂移负控是**真实发生**的：实现该接口时改了 driver 本身，`checkout_state()` 覆盖 `scripts/`，于是旧 manifest 立即失效——即 `--run-prepared` 绑定的正是本树源码身份，不是"exe 存在即用"。T-02 剩余：`cargo t` 默认入口在缺 manifest 时返回 `PREPARATION_REQUIRED`（§2 的入口切换）、工具 profile 改 debug/runtime-dev（B-05）、B-01…B-04 去测试体补构建、`test_fixed_groups.py`/`test_prepare_tools.py` 增治理负控。

**T-02 第二刀（同日续，代码 `946c38909`）：普通入口切换 + 治理负控**

- **规范位置**：`--prepare-only` 无 `--out` 时落到规范位 `target/test-groups/prepared/<checkout_state[:16]>/`；普通 `cargo t <ACE组>`（非 select-only/discover-only）现在先 `find_matching_manifest()`：在规范位按 mtime 倒序找**校验通过**且**覆盖全部所请求 ACE 组**的 manifest；找不到 → `PREPARATION_REQUIRED: no verified prepared manifest covers …; run: python scripts/fixed-tests.py … --prepare-only`，**exit 2、零构建零执行**；找到 → 以该 manifest 的 archive 运行（混合调用只对 ACE 组走准备件，非 ACE 组行为不变）。
- **实测**：无准备时 `python scripts/fixed-tests.py ace-lowering` → 0.11s 内 `PREPARATION_REQUIRED`（附可直接复制的准备命令）；`--prepare-only` → 1.19s 写到规范位；再有准备时直接跑 → `PASS 9/9`，整命令 1.6s，零构建。
- **治理负控（`scripts/tests/test_fixed_groups.py::PreparedRunGate`，5 例新增）**：合法 manifest 通过；config/source/toolchain 三类漂移各自具名失败；archive 篡改/缺失具名失败；`find_matching_manifest` 跳过无效 manifest 且要求"全覆盖"（只覆盖部分组时返回 None）；普通 ACE 入口在无准备时**不调用 `execute_plan`** 即抛 `PREPARATION_REQUIRED` 且消息含 `--prepare-only`。全套 `test_fixed_groups.py` **77/77 OK**（7.46s），`test_prepare_tools.py` 8/8 OK。
- **顺带修正既有红**：`ace-python` 的 `expression` 原本只是 `test(p1026)`（缺 performance 排除，`test_ace_module_groups_exclude_formal_performance` 在 master 上就是红的——已单独在 master 复现确认非本包引入）；现在去掉 args 里重复的 `-E test(p1026)`，expression 改为 `(test(p1026)) and not test(/(^|::)performance::/)`。**重发现验证选择面不变（仍 10 例）**，非放宽或掩红。
- **兼容面（须写入 SD-01）**：门覆盖**全部 `domain=ace` 的 nextest 组**，含外部 `ase-m0`（`experimental/ase-m0` 独立 workspace）与 `ace-perf`；二者从此同样要求先准备。**非 ACE 组与 `--ignored`/`--test-name` 显式重放路径不受影响**。命令行兼容：`cargo t <ACE组>` 的旧行为（自动构建后跑）不再可用，须先 `--prepare-only`——这是 §2 明示的意图改变，须在说明中写明。

### 规范增量

| delta_id | add/modify/retire | docs/specs目标 | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-test-case/module-selection.md` | 固定组/OR并集→补准备/执行和显式toolchain，精确用途选择，不建叶图 | 分层保可达 | AC-01/02/03/07 |
| SD-02 | modify | `docs/specs/auto-test-case/ace-regular-test-budget.md` | 历史21组→重冻真实完整组，补新增item≤10秒，组120/60不变 | 防成本再积累 | AC-08/09/10 |
| SD-03 | modify | `docs/specs/auto-test-case/ace-cost-attribution.md` | 原只读报告→扩新增例预算/阶段/首红/复用证据，不建新meter | 同一成本口径 | AC-02/08/09 |
| SD-04 | modify | `docs/specs/auto-test-case/ace-perf-cost-and-reproduction.md` | 原PERF集→两例S_new/成本显列，原样本/串行/3600秒合同保持 | 修漏归属 | AC-03/09 |
| SD-05 | modify | `docs/specs/auto-test-case/test-purpose-and-admission.md` | 四用途建议→补ACE逐义务处置/探针准入和守恒 | 去留有据 | AC-01/04/05/06 |
| SD-06 | add | `docs/specs/auto-test-case/ace-toolchain-certification.md`（新） | 无专门ACE认证入口→显式debug认证/受影响消费者必选/原family负控成本 | 编译认证保全 | AC-07/09 |

## 6. 测试设计

### 6.1 可区分的语义与工作流负控

全部为拟实施验证、当前未运行。owner_stage=work，T-10独立review重核；先确认变异实际落到被消费的输入/状态。纯计时边界可用小收据fixture，不用假PASS证明产品行为。

| ID | AC/任务 | 实际输入/入口和变异 | 必须观察的oracle | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/T-01 | 旧→新义务映射删S-01 A2R角色、改一个ignore/平台条件、重复同角色 | 守恒核查分别报缺失/条件漂移/重复，不靠总数凑平 | 全部处置行；不代语义运行 |
| V-02 | AC-02/T-02 | 合法prepare后run-prepared；分别删worker/lib、改sidecar/hash/source/config/features/env/profile | 正常控制执行；坏轴测试体前准备失败、零补编译；重准备后首次执行正确 | 不缺工具→Skip/旧release回落 |
| V-03 | AC-02/T-02 | B-01/02/03实际helper调用，缺库/缺env；B-05全部普通工具准备；观测完整子进程树 | 执行段零Cargo依赖构建/--release；普通显式准备也零release，正常B-02仍exit12、stderr ba；正式PERF独立profile/hash控制不串用 | Auto编译器本体不误禁，不能只检test body |
| V-04 | AC-03/T-03 | 真实普通/PERF发现；临时注册变异恢复mod perf或双挂载 | 普通泄漏/专项漏项/重复均红；合法两例唯一PERF、32样本/输入不变 | grep不代发现 |
| V-05 | AC-04/T-04 | F-01线性夹具7万不同常量、合法小模块；控制性关闭/错改TableEntries门 | 大输入精确Resource(TableEntries)，小输入接受，故障实现被抓住 | 原尺度/门不缩；不能先死在无关门 |
| V-06 | AC-04/T-04 | F-02全部旧源码/预期；分别改错exit/stdout，确定性见证真实双run | 两轴分别红；双run角色比较两次真实结果/流 | 单run不能代确定性 |
| V-07 | AC-05/T-05 | O-01错误值/trap，O-02六场景逐个错预期，O-03/04改成无关syntax/type错误 | 正常控制绿，各错误值/code/阶段明确红 | 预期先按Spec/独立oracle确认，不自动录制 |
| V-08 | AC-06/T-07 | console真实worker；各错stdout/stderr/exit、删三轴、改版本轴、NUL实传、Skip | 各轴Fail且mismatch准确，缺轴Error，fresh→stale，Skip无native，NUL字节不截断 | 框架原meta-oracle保留 |
| V-09 | AC-06/07/T-06/07 | H-05原family、错oracle14→15/错reference；C类各目标错expected或仅emit | 真compile/run有结果/身份/证书；错oracle/reference拒绝，少目标/错目标/emit冒run阻认证 | 原target/source/provider不换mini例 |
| V-10 | AC-06/T-07 | invocations实际host+console，同名但tools不同的host+rust-lowering，batch feature | 同key一次执行各义务引用，不同key不强合，零选择不PASS | 不按test名判配置相同 |
| V-11 | AC-04/06/T-04/07 | 真实fast edit、强制整前端rewarm、同hash换Arc；恢复后首次修改/读 | 正常编辑绿；值仍对而全重算/换对象红；恢复首改/首读和依赖正确 | 原≥100反控完整保全，小例不代签 |
| V-12 | AC-08/T-08 | 报告fixture新增item=10.000/10.001秒，缺终态/STALE/截断组，完整组=120.000/120.001秒 | 边界严格；超10新例review失败，陈旧/缺终态不可复用，超120组红，不取整 | 不真sleep10秒，新增治理例自身≤10秒 |
| V-13 | AC-02/10/T-02/08 | 临时域模拟有owner租约与第二会话准备/union，合法释放再试 | 忙时不启动build/test且通知、单列等待，释放后可进入，不误杀未知活动 | 不干扰真实他agent |
| V-14 | AC-07/10/T-06/09 | Windows普通/聚合/toolchain的选择/准备/运行检查Linux/WSL，S-39核bind-only | 零WSL/原生Linux腿；归1061/1063，bind结果不冒运行PASS | 不要求Linux环境，不按名称批量关平台中立算法 |

### 6.2 耗时口径与新用例准入

- 用现有固定driver的全新`--out`、原命令meter、nextest完整item `exec_time`、Case原生`duration_ms`；扩展原`ace_cost_report.py --mode regular`，不新建第二meter/统一三warm或P90矩阵。并行item时间不能相加当墙。
- 分列等待其他会话`T_wait`、显式编译/工具准备`T_cold`、发现/准入、运行、父判定/复制/清理、整命令墙。没单独观测的阶段记UNOBSERVED及边界，不填0；测试体补构建不能藏进cold排除。
- 完整职责组`T_warm`含manifest校验、必要发现/准入、运行、父判定/复制/清理，≤120.000秒、目标60。`ace-workspace`/`ace`聚合另列选择和真实墙，不冒也应≤120，不代24个单职责预算。
- 新增item看完整setup/body/teardown≤10.000秒，包多个子场景看整体；工具链/治理新负控也适用。迁移原大例原角色和原预算保留，单列“迁移原例”；新拆出的日常小例仍受10秒门。
- 无计时/终态、身份无法对应、失败后只选快子集均不通过。47行逐项关闭：日常完整item≤10；分流原义务/原条件有效通过；待定位不能算关闭。
- 新增差集由本包实际base与当前完整发现/Case检查清单对照，配合旧→新义务映射区分真正新增、改名和原例迁移；不能靠手写“旧例”标签豁免10秒，也不在执行时扫描全仓构建动态目录。新增test包多个case或子check时同时核其完整原生item和适用原生check时间。
- 历史成本只排序；前后比较说明源码变化与等价断言。最终证据绑定本包冻结源码/依赖/config/input/tool/observer，同身份覆盖不变可复用，不为提交重复全轮。首红、未跑、环境归因、隔离重放分别保留。
- P-01/P-02在原串行PERF profile下实际完整运行，新增S_new单列。原集合不变部分可按身份合法复用，复核1017逐模块求和的≤3600秒合同。syntax-core新增模块原预算没有覆盖时单列实际成本/合计，不挪用其他模块预算掩盖；确实越界保未完成并提出修复，不自行改上限。该专项不附加每次review/merge。

### 6.3 计划命令、职责组与评审职责

全部未执行。未来`$PlanOut = 'D:/autostack/plan-artifacts/plan-1098'`；每次输出新目录。T-02选项是拟新增接口，T-01核后写回真实命令，不能当已有能力运行。

```powershell
# 已有，只读预览，不证明真实发现/通过
python scripts/fixed-tests.py ace-source ace-av ace-host ace-console --select-only

# T-02拟新增：准备与无编译执行，实际manifest名在T-01/02钉住
python scripts/fixed-tests.py ace-source --prepare-only --out "$PlanOut/prepare-source"
python scripts/fixed-tests.py ace-source --run-prepared "$PlanOut/prepare-source/prepared.json" --out "$PlanOut/run-source"

# 已有报告器，真实run须完整单职责组
python scripts/test-governance/ace_cost_report.py --mode regular --run "$PlanOut/run-source" --out "$PlanOut/cost-source.md" --json-out "$PlanOut/cost-source.json"

# T-06拟新增显式认证组，依赖先准备；原外部编译本身是其被测义务
python scripts/fixed-tests.py ace-toolchain --prepare-only --out "$PlanOut/prepare-toolchain"
python scripts/fixed-tests.py ace-toolchain --run-prepared "$PlanOut/prepare-toolchain/prepared.json" --out "$PlanOut/toolchain"

# 原单例正式重放方式；T-03在计划更新准确迁名
cargo t ace-perf --test-name performance::perf_fresh_parse_and_incremental_and_restart_query --out "$PlanOut/perf-syntax-incremental"
cargo t ace-perf --test-name performance::perf_core_vs_legacy_parser_reference --out "$PlanOut/perf-syntax-reference"
```

24个现有非聚合常规组：`ace-source-syntax-927`、`ace-source`、`ace-syntax`、`ace-semantics`、`ace-artifacts`、`ace-m2-build`、`ace-av`、`ace-native-worker`、`ace-native`、`ace-abi`、`ace-comptime`、`ace-lowering`、`ace-python`、`ace-foreign-contract`、`ace-a2c-abi`、`ace-host`、`ace-batch`、`ace-console`、`ace-a2x`、`ace-stdlib`、`ace-facade`、`ace-adapters`、`ace-runtime`、`ace-rust-lowering`。T-01处理真实新增/迁名delta，不据历史数删组。

治理组：`infra-fixed`、`infra-ace-cost-report`、`infra-ace-policy`，必要定向`python -m unittest discover -s scripts/tests -p test_prepare_tools.py`。真实Case消费者已核具体组：`case-ace-m1-console-863`→`crates/auto-lang/test/cases/ace-document-console.suite.ad`；`case-ace-m1-base`→`ace-document-base.suite.ad`（AV/多检查）；`case-ace-m1-tr-native`→`ace-m1/ace-m1-tr-native.suite.ad`（真实native接缝）；`case-ace-m1-typed-native-891`→`ace-m1/typed-native-891/typed-native-891.suite.ad`（typed native）。T-01复核当前工具与实际动作；若本包改变的worker/ABI不由这些组覆盖，追加具名消费者，不能占位验收。

work迭代跑定向负控+受影响完整组；T-09冻结24个单组预算，可复用本次同身份完整证据。最终union由真实diff确定，预计包含`ace-source ace-syntax ace-av ace-native-worker ace-native ace-host ace-console ace-rust-lowering ace-batch`、治理和Case消费者；共享准备协议变更时补所有真实消费者。列明联合组名/选择理由/时间，不只报告“Group union”。同key去重，PERF/认证另窗；不在work/review/merge反复叠跑24组全轮，只重测因修复失效范围。

不以`cargo t all`替代本包验收，周期批量回归独立到期处理。work实现和内部自查；review在同一树独立上下文，暂停work写入、核AC且不边审边修；merge只做冻结必要并集与沉淀清理。未获用户额外委派，不自动开agent/chat。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 真实发现冻结入口/条件/用途；§5.2、47行及新慢例/补构建/弱oracle全部有逐角色去向；V-01缺行/漂移/重复红，声明数不冒执行数 |
| AC-02 | work | 显式prepare后普通组消费同身份产物；缺失/陈旧/错env/lib/sidecar在体前错误且0补构建/0release，真实进程/收据证明，V-02/03/13 |
| AC-03 | work | P-01/P-02普通各0、PERF各1；原32样本/输入/场景/统计不减，V-04；其余普通项无误失选 |
| AC-04 | work | F-01原7万/精确资源门，F-02全部原exit/stdout/确定性，R类身份/恢复/owner断言保留；V-05/06/11敏感，留日常行≤10秒 |
| AC-05 | work | O-01…05当前独立oracle或有据research去向；V-07错值/code/阶段红，不打印即绿或按probe名批量删 |
| AC-06 | work | H-01…06职责映射完整，15语料/版本/NUL/三轴/Skip及配置隔离有效，V-08/10/11；同key不重复，不同key不误合 |
| AC-07 | work | ace-toolchain实际显式注册，全部C/H-05原目标compile/run/reference/cert负控可达；V-09/14。真实工具链触面必选；Linux不冒PASS |
| AC-08 | work | 原报告器新增完整item≤10秒门，V-12严格边界/缺终态/STALE/截断组敏感，自身新增例≤10秒；不新建runner/meter |
| AC-09 | work | 当前24完整非聚合常规组或重冻delta的T_warm均≤120秒，目标60；47行逐项关账、新增item≤10；分流原例/条件通过，PERF增量和原3600秒合同可核，聚合墙另列 |
| AC-10 | work | 执行前并发检查/等待通知，报告实际模块、列举/执行/条件排除/失败、准备/发现/运行/清理/整墙、新增内容/逐项时间；V-13/14。首红/未跑/重放分列，零WSL |
| AC-11 | review | 冻结提交独立核AC-01…10/SD-01…06/实际diff、遗漏/未批准延后/陈旧证据/告警/格式/debug输出；新增>10、组越界、必验未跑不得pass，绑定review SHA |
| AC-12 | merge | 通过review后冻结必要并集、Spec沉淀、摘要折回/归档、所有输出目录对账；材料全删或具名持有，wt-guard clean后清树；实际关闭后勾完 |

功能红与成本绿不互相覆盖。未关闭本包AC或缺必要oracle不能execution_done；外部既有红具名保留，review按原规则判影响，不跳整组或宣称全组PASS。

## 8. 执行步骤

| 任务 | 阶段 / owner_stage | 依赖 | 修改面、交付和出口 | 验证 |
|---|---|---|---|---|
| [ ] T-01 | I / work | 用户后续授权实施 | 建唯一树/scope；重冻24职责+聚合/PERF/Case、包target/module/source差集；展开全部问题/47行的角色、原条件、预期、消费者；核nextest接口/Linux归属/旧红/实际工具需求，把准确Case组与探针预期写回计划 | AC-01、V-01；仅必要发现/代表基线，先查并发。出口为可执行准备接口和逐行决定，非另一份泛调研 |
| [ ] T-02 | II / work | T-01 | `scripts/fixed-tests.py`、`scripts/test-governance/{prepare_tools.py,toolchains.json}`、必要薄驱动、B-01…05；显式prepare/run、身份/互斥、普通debug/runtime-dev增量和正式PERF profile隔离，去helper补构建/release；复用`test_fixed_groups.py`/`test_prepare_tools.py` | AC-02/10，V-02/03/13；真实source/worker/native/Case代表链，不只mock |
| [ ] T-03 | III / work | T-01/02 | `syntax-core/tests/conformance.rs`、`cases/perf.rs`及必要选择治理；两例接performance、原正文统计合同保全 | AC-03、V-04；真实发现差集+两例原PERF完整运行/成本 |
| [ ] T-04 | III / work | T-01/02 | `av/tests/cases/{mir_tier.rs,sched_m2_968.rs}`及D/R候选夹具；线性常量、单run多轴、等价准备复用，不改产品语义 | AC-04、V-05/06/11；受影响完整ace-av及下游，原尺度/恢复状态保留 |
| [ ] T-05 | IV / work | T-01/02 | `p946_t01_probe.rs`、`probe972.rs`及挂载；O-01…05固化当前合同，无长期目的角色逐项具名research/退休并留原依据 | AC-05、V-07；各错预期真红、合法绿、新增例≤10秒 |
| [ ] T-06 | V / work | T-01/02 | 47表C/Q/L、`host/rust_lowering.rs`认证部分、`test-groups.toml`/必要target；显式ace-toolchain保原各端/故障，Q正式压力原样PERF；Linux交1061/1063，绑定事实单列 | AC-07、V-09/14；旧新同输入每端对照、专项完整真实运行，不仅emit |
| [ ] T-07 | V / work | T-01/02/06 | `conformance/tests/console.rs`、host协议、`invocations`和Case消费者；H-01…06核实后消重，保真实运输/版本/负控和batch隔离 | AC-06、V-08/10/11；完整host/console/rust-lowering/batch及15项case组；语义union可去重，预算须单组 |
| [ ] T-08 | VI / work | T-01/02，最终接T-03…07 | 扩原`ace_cost_report.py`、`scripts/tests/test_ace_cost_report.py`：新增差集/≤10秒、原组预算/终态/STALE检查；会话报告，版本兼容；不另建meter | AC-08/10、V-12/13；治理完整组+prepare-tools定向，缺失时间不伪造 |
| [ ] T-09 | VI / work | T-03…08 | 冻结实现，24单组有效预算+47行关账、新增内容/时间、工具链/PERF增量及负控关闭，提交SD提案/Spec增量和源码/依赖/config/input结果身份 | AC-01…10；可复用同身份完整证据，超支/缺口不勾完成 |
| [ ] T-10 | VII / review | T-09 | 同树独立上下文复审，暂停work写入，逐AC/SD/改动义务核真实输入消费者、负控敏感性和完整成本，必要定向复验并绑定review SHA | AC-11；needs_fix/needs_replan回work，不能边修边签 |
| [ ] T-11 | VIII / merge | T-10通过、合入授权 | 冻结必要并集，沉淀SD-01…06、归档/输出对账/摘要折回，删本包材料与守卫后清树 | AC-12；未来merge任务不阻T-09交execution_done，不提前勾 |

下一就绪任务只有T-01；准备接口、探针独立预期、逐角色分流未完成前不跳到大范围删例/切组。阶段出口可审查；先落地阶段须独立phase review与冻结门，不提前称全包完成，不另建第二实施树。

完成汇报必须填写（实测，不只说“Group union”）：

| 模块/实际命令/源码身份 | 列举/执行/条件排除/失败 | 等待 | 冷准备/工具构建 | 发现/准入 | 运行 | 父判定/复制清理 | 整命令 | 证据复用来源/理由 |
|---|---|---|---|---|---|---|---|---|
| 待实施 | 未运行 | — | — | — | — | 未观测须注明 | — | — |

新增例另列`新身份/内容/独立预期/完整item时间/10秒判定/AC`；移入专项原例另列`旧→新身份/原尺度/oracle/实际时间/条件`，不冒新增快例。merge收尾对账全部实际输出目录（含外盘/消费者根），摘要与hash折回计划，plan-artifacts整scope删除或按具名未决持有登记，wt-guard clean后清理树/分支。归档不是清理已完成证明。

## 9. 复审记录

2026-11-19：`stage: new`，PLAN-1098 r1，`outcome: pass`（仅规划合同自检），`next: 等待用户授权work后T-01`。实现进度0/11。已纳入本次P/B/F/O/H发现与47项历史AV慢例，所有实施负控/成本验收及独立复审未执行。

本轮仅修改本计划与取号簿记，未改测试、配置、Specs；未创建实施树，未启动新的编译/发现/测试，未合入归档。旧收据的PASS/FAIL不是本轮测试结果。

2026-11-19（goal round 1，r1）：`stage: work`，T-01 **进行中**（未 pass）。执行内容：goal 授权入 §4.1；建唯一实施树 `D:/autostack/.wt/lang-1098/auto-lang`（`plan-1098-dev`，基面 `d20cf3f87`，主检出 master 建树后被并发会话推进到 `37379e6d4`）；建材料 scope `D:/autostack/plan-artifacts/plan-1098/`（空，已在 `.retention/policy.json` 登记 in-flight，无构建中间产物）；完成 §5.5 K-01…K-11 只读冻结（分母 102 ACE 组逐类核实、P-01/P-02 泄漏结构性确认、B-03/B-05/F-01/O-01/H-01…03 源码实证、nextest 0.9.138 archive 接口、既有 `execution_lease()` 互斥、47 行源码存在性）。**未**构建、**未**跑任何组：开工时机器上有其他会话活跃的 cargo/nextest（14:44–14:45 启动），按 §5.4-4 等待，不并发启动准备/运行。主检出代码路径无 WIP（仅 docs/ 与他会话 `packages/blueprints/**` 删除，非本包写集）。T-01 出口未闭：逐行（P/B/F/O/H + 47 行）义务映射、真实发现差集、两个旧红重放判定、Case 消费者组核对仍待后续轮次。

`stage: goal | plan_id: PLAN-1098 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: executing | outcome: pass（仅路由：drafting→work 已开启，T-01 未完成） | worktree: D:/autostack/.wt/lang-1098/auto-lang | commit: ccbe165f3（计划文档）/ d20cf3f87（树基面） | next: work（继续 T-01）`

2026-11-19（goal round 2，r1）：`stage: work`，T-01 继续（未 pass）。执行内容：①真实重放 `ace-av` 的 `host_console::` 21 例（冷 debug，准备 61s / 执行 0.280s / 整命令 62.7s / exit=100，19 绿 2 红），两个旧红**首红复现**并定位为 982 具名入口裁定后的**陈旧 oracle**（非产品回归），具名移交 D-1098-01；②逐行静态核查补齐 B-01/B-02/B-04（含"同一缺库补构建反模式 4 处"、B-04 死 helper 无消费者）、C 类真 rustc/真 C 编译 helper 实证、P-01/P-02 原参数、H-04/H-05 预算与负控；③47 行候选分档落材料 scope（`t01-47-rows-static-triage.md`，sha256 `314539e1…0f68`）。机器无并发构建（开工前后均已核对）。仍未闭：47 行逐项展开、24 组真实发现分母与 >10s 项、toolchain 目标清单/Case 消费者核对。

`stage: goal | plan_id: PLAN-1098 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass（进行中：重放与静态核查落地，T-01 出口未闭） | worktree: D:/autostack/.wt/lang-1098/auto-lang | commit: 计划文档见下一条 docs(plan1098) 提交；树 d20cf3f87 | next: work（T-01 续）`

2026-11-19（goal round 3，r1）：`stage: work`，T-01 继续（未 pass）。执行内容：47 行结构化展开（§5.7）——花括号感知取体（内嵌 Auto 源码不再截断）、oracle 机制/本地助手链/端助手/时间门逐行提取，产物 `t01-47-rows-expanded.md`（`a9250286…`）+ 脚本 `extract_rows.py`（`d986d9e7…`）；四条结构性结论（47 行多为外壳、助手跨行共享、端助手集中、raw `Command` 不被命名覆盖）与既有时间门实证。**未跑任何组**：开工时另一会话 `python D:\autostack\.wt\lang-1078\auto-lang\scripts\fixed-tests.py case-av-language-parity` 正在 `cargo build -p ace-av --release`（14:50 起），按 §5.4-4 等待、未并发启动准备/运行；Case 四套件仅做存在性静态核对（未 inventory）。未闭：24 组真实发现分母与 >10s 项、toolchain 目标清单确认、Case inventory。

`stage: goal | plan_id: PLAN-1098 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: pass（进行中：47 行结构化展开落地并留局限；T-01 出口未闭） | worktree: D:/autostack/.wt/lang-1098/auto-lang | commit: 计划文档见本次 docs(plan1098) 提交；树 d20cf3f87 | next: work（T-01 续：等机器空闲后取真实发现分母）`

同日续（round 3 末段）：24 组包/target/feature 差集静态冻结（§5.8，脚本 `extract_groups.py`，产物 `t01-24-groups-targets.md`）；静态实证 H-06——`ace-host`/`ace-console` 的 9 元 key 完全相同必须合并一次调用，`ace-batch`（features）与 `ace-rust-lowering`（tools）key 不同不得强合；另发现 `ace-m2-build` 实为 8 个 test target、`ace-a2c-abi` 只有 lib、`ace-facade`/`ace-adapters` 跑根 workspace（易被误读为 ACE workspace 单例）。仍未跑组。

2026-11-19（goal round 4，r1）：`stage: work`，T-01 继续（未 pass）。执行内容：①**准备/运行接口实证**（§5.9）——`cargo nextest archive -p ace-lowering --lib --tests` 18.75s 产出 4.56MB tar.zst；`cargo nextest run --archive-file … --workspace-remap …` **9/9 PASS 0.34s 零构建**；再加 `--ignore-default-filter -E 'test(closures_lowering::)'` → 3/3 PASS 0.39s（表达式在 archive 模式生效）。prepare/run 分离在真实 nextest 上成立。②**真实发现分母第一批**（§5.10）：`ace-av` **1,428/0**（8 二进制，墙 56.378s，峰值 Job 3.50GB）、`ace-lowering` 9、`ace-artifacts` 43、`ace-semantics` 295；`ace-av` 发现数与此前执行的 1,428 一致。③材料：`discover-ace-av/`、`discover-small-3/`、`archive-spike/`。仍受并发会话（lang-1078 `case-av-language-parity`，14:50 起持续）限速：`ace-host`/`ace-console`/`ace-batch`/`ace-rust-lowering`（共享 `ace-conformance` target）等其余组未发现。

`stage: goal | plan_id: PLAN-1098 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: pass（进行中：archive 接口实证 + 四组真实分母；T-01 未闭） | worktree: D:/autostack/.wt/lang-1098/auto-lang | commit: 本轮 docs(plan1098) 提交；树 d20cf3f87 | next: work（T-01 续：其余组发现 + 收口）`

2026-11-19（goal round 5，r1）：`stage: work`，T-01 继续（未 pass）。执行内容：①第三批 **10 组真实发现分母**（§5.10）：`ace-source` 904、`ace-abi` 314、`ace-syntax` 159、`ace-comptime` 136、`ace-runtime` 84、`ace-foreign-contract` 56、`ace-source-syntax-927` 13、`ace-python` 10、`ace-a2c-abi` 9（lib 单测）、`ace-stdlib` 2，小计 1,687/0，整批墙 212.33s；五批累计已发现 **3,651** 例。②**O 类独立预期钉住**（§5.11）：O-01 依 `ace-collections.md:17` 钉为 Trap `list-index`；O-02 六场景按 Unicode/算术事实钉出精确值（1/4/1/97/1/20013）；O-04 依 packages 面 `source.module-symbol` 声明；O-03 无 spec 声明 → 交 T-05 决策（不得录打印值）；O-05 保留 22 断言 + cap 段须独立 oracle 或具名移 research。未闭：`ace-native`/`ace-native-worker`/`ace-a2x`/`ace-m2-build`（8 target）/`ace-facade`/`ace-adapters`（根 workspace）、聚合 `ace-workspace`、Case inventory。

`stage: goal | plan_id: PLAN-1098 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: pass（进行中：10 组分母 + O 类预期钉住；T-01 未闭） | worktree: D:/autostack/.wt/lang-1098/auto-lang | commit: 本轮 docs(plan1098) 提交；树 d20cf3f87 | next: work（T-01 收口：重内存组发现 + Case inventory）`

同日续（round 5 末段）：T-02 **首刀实现**（§5.12，代码提交 `197ac9e5d`）——`--prepare-only`/`--run-prepared` archive 握手落地并实测：准备 1.19s → 只运行 PASS 9/9 wall 0.721s 零构建；三条负控（源码漂移/archive 篡改/archive 缺失）全部具名 `PREPARATION_REQUIRED` exit 2、零执行。其中"源码漂移"负控是真实事件：改 driver 自身后旧 manifest 立即失效。T-02 未完成项照旧：默认入口切换、B-05 profile、B-01…B-04、治理负控。

`stage: goal | plan_id: PLAN-1098 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: pass（T-02 首刀落地并有负控；T-01/T-02 均未闭） | worktree: D:/autostack/.wt/lang-1098/auto-lang | commit: 计划 197ac9e5d（T-02 代码）/ 本轮 docs(plan1098) | next: work（T-02 续 + T-01 收口）`

2026-11-19（goal round 6，r1）：`stage: work`，T-02 第二刀（§5.12 续）。执行内容：①普通 ACE 入口切换——`--prepare-only` 无 `--out` 落规范位 `target/test-groups/prepared/<source_state[:16]>/`；默认 `cargo t <ACE组>` 先找**校验通过且全覆盖**的 manifest，缺失即 `PREPARATION_REQUIRED`（exit 2、零构建），实测 0.11s 拒绝 / 有准备时 `PASS 9/9` 整命令 1.6s 零构建。②治理负控 `PreparedRunGate` 5 例（config/source/toolchain 漂移、archive 篡改/缺失、find 跳过无效且要求全覆盖、无准备不调 execute_plan）；`test_fixed_groups.py` **77/77 OK**、`test_prepare_tools.py` 8/8 OK。③修正 master 既有红：`ace-python` expression 缺 performance 排除（已在 master 复现确认非本包引入），去重复 `-E` 并补排除，重发现仍选 10 例。代码提交 `946c38909`。T-02 剩余：B-05 工具 profile（debug/runtime-dev）、B-01…B-04 去体内补构建、工具链认证组（T-06）。

`stage: goal | plan_id: PLAN-1098 | plan_revision: r1 | round: 6 | from: work | to: work | status: executing | outcome: pass（入口切换+治理负控落地；T-02 未闭） | worktree: D:/autostack/.wt/lang-1098/auto-lang | commit: 946c38909 / 本轮 docs(plan1098) | next: work（T-02 续：B-05/B-01…04）`

2026-11-19（goal round 7，r1）：`stage: work`。执行内容：①第四批重内存组真实发现（§5.10，`discover-heavy4/`，整批 39.55s）：`ace-native` 329、`ace-a2x` 328、`ace-native-worker` 43、`ace-m2-build` 30，小计 730；**累计 4,381 例 / 22 个非聚合组**（余 facade/adapters 两个根 workspace 组 + 聚合/PERF）。②**B-05 真实消费面核查**（§5.13）：全量扫描得 37 处 `target/release|runtime-release` 引用 / 20 文件，其中 11 处会触发构建，**5 处是 lib 级 lazy builder**（`av/text_runtime`、`abi/source_foreign`、`native/{collections,sched,typed}_runtime`），且 `collections/sched/typed` 三个 runtime **无 toolchains 条目**（今天无法预建）——计划原假设的"测试体补构建 3 处"明显低估。据此给出 B-05 四步改法（补工具条目 → lib builder env 优先且缺省回落仅限非测试 → hardcode 测试改 env → 最后翻 profile 并真实验证）。本轮未改实现代码。

`stage: goal | plan_id: PLAN-1098 | plan_revision: r1 | round: 7 | from: work | to: work | status: executing | outcome: pass（T-01 分母 22/24 组、B-05 消费面核查完成；T-02 未闭） | worktree: D:/autostack/.wt/lang-1098/auto-lang | commit: 本轮 docs(plan1098) | next: work（T-02 B-05 第 1/2 步：补工具条目 + lib builder env 优先）`

## 10. 待澄清事项

无阻止提交本稿的用户决策。以下由T-01有界核查，不据此推迟全包或重新请求同范围许可：

1. 安装版nextest/薄驱动的预构建消费接口，出口是T-02可实施命令和零编译证明，不假定未核参数。
2. O-01…04当前Spec及独立预期：区分已支持/仍拒绝并记录精确值或阶段/code，不能自动录制当前输出。合同矛盾时只冻结该行并提出具体行为决策。
3. C/Q/L每行角色、冻结尺度与Linux归属先查真实代码。无等价分流或日常120/10预算不可达时，具名保未完成和最小修订，不自行降门。
4. 24组/47项为规划分母；并行计划delta须补对照，不能删旧行。新的产品语义、外仓实施或预算改变需用户决定。
5. **D-1098-01（具名未决，已登记 KNOWN-DEBT）**：`ace-av::conformance host_console::av_console_entry_selection_cannot_bypass_main_policy` 与 `…_owned_front_door_enforces_console_entry_only` 两例按 863 旧策略断言 `lower.entry`，已被 PLAN-982 `3a9d9c1fb`（具名入口 `d_fn`，形状由 verify 守）废弃；2026-11-19 于本树 debug 重放两例均红（§5.6）。**不得由本包单方面改预期**：需 982 线 owner 或用户裁定"改写成新合同的独立 oracle"还是"恢复旧策略"，本包按外部既有红具名保留、不 ignore、不充 PASS。关闭条件见债务条目。
