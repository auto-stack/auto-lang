---
plan_id: PLAN-1108
status: drafting
feature_name: PLAN-1098 后续批量回归门恢复
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-20
plan_revision: 1
depends_on: [PLAN-1098, PLAN-1056, PLAN-1045]
supersedes_spec_components: [docs/specs/auto-test-case/module-selection.md, docs/specs/auto-test-case/ace-regular-test-budget.md]
new_spec_components: []
touched_goals: []
affects: [auto-test-case, auto-lang-tests, auto-ac]
current_step: 0
total_steps: 7
---

# [PLAN-1108] PLAN-1098 后续批量回归门恢复

## 0. 变更摘要

PLAN-1098 已归档，准备与执行分离、工具产物身份绑定、慢例分流已落主。后续两次到期批量回归均出现 28 PASS、6 FAIL、2 N/A。先恢复批量门，处理配置作用域和治理测试问题，并重验已有修复的 Playground 编译腿。本稿是实施前规划，实测任务均待执行。

## 1. 目标

- 恢复 aavm、auto-lang、auto-lang-core 的真实发现和运行，两个 AAC 组继续各自最多四项并发。
- 恢复 infra-meter 对泄漏、回收和工具 pin 的判定，以及 infra-fixed 对缺准备错误的独立验证。保留生产租约互斥。
- 重验当前树的 auto-playground。交付后在主检出单实例复验正常 cargo t all，原六个失败组逐一取得真实终态。
- 对齐日常职责组与显式组预算口径。新增 item ≤10 秒，ACE 完整日常职责组 ≤120 秒，60 秒为目标。

## 2. 架构方案

复用固定选择、现有 nextest 配置、准备收据及运行租约。将 AAC 专用并发规则限定在各自有效的配置或筛选条件内。T-01 核对当前 nextest 可用的配置能力。若需要组专属文件，需证明普通准备、archive 和 prepared run 可使用等价配置，同时保留通用资源规则。

实际批量失败发生在 nextest list。显式准备另有 nextest archive 路径。两条路径都要验证。AAC 各自的并发限制、选择和准备身份都须实际核验。不得通过扩大 Cargo 包图或关闭有效配置检查解决未匹配错误。

## 3. 技术栈

Python 固定驱动与治理测试、Cargo/nextest、Windows Job 和现有租约。实施树 D:/autostack/.wt/lang-1108/auto-lang，分支 plan-1108-dev，材料 D:/autostack/plan-artifacts/plan-1108/。准备和测试前检查其他会话的测试、构建及租约。若忙，等待并通知用户，等待与工作时间分列。周期批量复验在主检出单实例，Linux/WSL 按现行裁定排除。

## 4. 需求分析与背景调查

用户授权基于 1098 分析和规划。取材基面 5089d3de01d5f028fa6c545698662d15ab73c983。参照[总览](../specs/overview.md)、[固定组](../specs/auto-test-case/module-selection.md)、[预算](../specs/auto-test-case/ace-regular-test-budget.md)、[工具链认证](../specs/auto-test-case/ace-toolchain-certification.md)和[已知债务](KNOWN-DEBT-AND-RISKS.md)。1098 合并 8a9f2816d，归档 b067fc060，归档内任务已标完成。frontmatter 的 current_step: 0 与该收据不一致，此处据交付与验证记录判断。[1098 归档](archive/1098-ace-daily-regression-purpose-and-cost.md)

### 4.1 1098 最终验收

冻结 905208386 的 §5.28 记录 ace-host 42.8s、ace-av 47.2s、ace-native 41.3s、ace-a2x 58.7s。显式 ace-toolchain 56/56 PASS，61.0s。ace-workspace 225.6s、ace-host-suites 40.2s、ace-perf 577.6s，三者均 FAIL。143 是各行失败事件之和，含聚合组重复执行。归档的 24 行达 120 秒预算，其中含显式 toolchain。按最终行值只有这一行超过 60 秒，正文所写 22 行 ≤60s 与表值不一致。下一轮按真实选择对齐分母并保留历史记录。

### 4.2 批量回归失败的原始证据

主检出 target/test-groups 下 run-amzo64ib 用时 439.038s，run-753_hof3 用时 179.068s。两次均为 36 组，28 PASS / 6 FAIL / 2 N/A。后者 results.json SHA256 为 eaedf0fd77d3443fac18a92ad82ad5d7f00515a32245fb1a4fb2b05dbf3f1507，source_state 为 03344bf382251e452a9cc6650b92f6a15195b8c54744a119daf6c6aaf1d0b556。[现有批量收据](.last-batch-regression.json)

| ID | 实际组/用例 | 原始结果与原因 |
|---|---|---|
| G-01 | aavm、auto-lang、auto-lang-core | 最新批量目录 0/9/10，discovery.command.json 的 nextest list exit 96。discovery.stderr 记录 package(ace-aac-support-conformance) 和 package(ace-aac-host-conformance) 在 root 包图无匹配。早一批只有 support 规则。源自 1056/1045 加入 .config/nextest-full.toml 的规则 |
| G-02 | infra-meter / test_preparation_rejects_leak_despite_existing_artifact_then_accepts_clean_build | scripts/tests/test_native_meter.py 以 tools/release/tool.exe 校验 FIXTURE_WORKER。1098 的 prepare_tools.pin_artifacts 改为按身份固定的副本，正控制仍期待旧路径 |
| G-03 | infra-fixed / test_ordinary_ace_entry_requires_preparation_before_any_run | scripts/tests/test_fixed_groups.py mock 了 manifest 查找，却通过 fixed.main 共用批量根的活租约。收到 lease held，无法抵达 PREPARATION_REQUIRED |
| G-04 | auto-playground | 两批 23/discovery.stderr 都是 nextest list 下的 Cargo exit 101。auto-lang-ace-host/src/project.rs 的 render_type 漏掉 TypeRef::Inferred（E0004）。该组没有指定 nextest-full.toml。64e518f52 已在主线补上渲染为下划线的分支，待实跑 |

固定组 SHA256 8bb5c62196c720dbd4d78c3f15246b7add4ff1ed834d4ae69b4912dcf39f295a，nextest-full SHA256 6c56945a398a5fd575c9ee9df2ca3567ae2d26419a901ccbd7f1d40f5f7c815a，fixed-tests.py SHA256 eef2d86c1274732f84fe72aca0f0f01c9344a34707958049169ea615d8700ef1。T-01 以届时源码与收据重绑。

### 4.3 后续排序

| 顺序 | 事项 | 具体范围与出口 |
|---|---|---|
| 1 | 本计划恢复周期门 | G-01…03 修复及 G-04 重验，恢复可靠的发现、运行和收据 |
| 2 | ace-batch 的 AutoDown 消费（D-1098-02） | 12 个测试中 8 红。tests/conformance/tests/batch.rs 的 ac_av::test_batch::pilot::load 仍读取旧围栏，closures-source-865.suite.ad 已 include 文档式文件。复用正式加载和规范化路径，保 15 例的 K1/K3/K10/tail 对照、逐例 oracle、超时与回收。以批处理对照独立运行、错 oracle/错 scope 判红及 12 项真实终态为出口。这是下一份 ACE 正确性计划的首选 |
| 3 | 正确性和正式门红项 | D-1098-01 两条 console 旧预期交 982 入口合同 owner，核具名入口正例和 verify 错形状/签名拒绝。D-1037-01、1060-R3/R8、972 pin、source_foreign authority 按当前身份与原因归组。前三个新 PERF 红族 closures_fn_912*、closures_recovery_913*、mir_native_incremental_series_formal 按原 profile 空闲定向重放，保首次失败 |
| 4 | 剩余准备与用例成本 | 先更新同机同输入基线，按组墙和实际原因排序。核 abi/a2x/store-gc 及 text/data runtime 的 release 准备是否有时限、ABI 或材料签章要求。1098 的 a2x debug 超时证据早于 30 秒探针调整，适合用具名编译腿做小范围 A/B。原有慢例受组预算约束，新增例继续用 10 秒硬门 |
| 专用环境 | 1061 / 1063 | 原生 Linux VM 分别负责入口隔离和标准库验收。Windows bind-only 结果保留其绑定角色 |

1045 所记 v05 真实工具链腿入显式认证组仍归 AAC owner。954 的路径夹具和 stdin 就绪问题由其原计划处理。两者均有各自的验证范围。

## 5. 详细设计

| 事项 | 修改面与必须保留的义务 |
|---|---|
| G-01 配置 | .config/nextest-full.toml、scripts/test-groups.toml，必要时固定驱动的 nextest_args、archive_args、build_archive、run_rust、verify_prepared。实际验证普通发现、archive、prepared list/run 的有效配置和旧收据失效。ace-aac、ace-aac-host 各自最多 4，其他原使用的资源规则及 ace-perf 串行继续生效。若配置需要传到 prepared 段，纳入已有身份校验 |
| G-02 pin | scripts/tests/test_native_meter.py 验证 env、工具行和收据引用同一身份域内的 pin，字节和 SHA256 与合法构件相同。旧路径已存在且有后代泄漏时仍失败，记录回收且不产生成功准备。干净重跑后共享路径改写不影响有效 pin，真实 pin 缺失或被改写应拒绝复用 |
| G-03 租约 | scripts/tests/test_fixed_groups.py 让实际缺准备测试使用独立临时根的租约。父进程持有批量根租约时，子测试仍得到 PREPARATION_REQUIRED 和正确准备指引，且未调用准备/执行。保同根第二请求拒绝、owner 文档完整、合法 prepared 入口正对照 |
| G-04 复验 | 冻结 64e518f52 与当前渲染入口来源，实跑 auto-playground 后取真实终态。新失败按具体阶段和 owner 归属 |
| 记录 | 新收据附旧误归因的原始诊断依据。核实际默认/聚合选择，不以 1098 表中混进的显式 ace-toolchain 推导日常分母，ase-m0 按真实入口归属单列。持有债务直到实测关闭 |

### 规范增量

| delta_id | 类型 | 目标 | 变更与验收 |
|---|---|---|---|
| SD-01 | modify | docs/specs/auto-test-case/module-selection.md | 组专属资源配置的作用域、准备/运行有效配置一致及身份校验。以固定选择对齐日常与显式角色。AC-01/04 |
| SD-02 | modify | docs/specs/auto-test-case/ace-regular-test-budget.md | 对齐真实分母，历史混合表注明范围，显式组及聚合另报。60/120/10 秒规则照现有裁定。AC-04 |

## 6. 测试设计

以下均为待实施验证。负控配置和产物变异在实施树或独立临时材料里进行。owner_stage 为 work，V-06 归 merge。

| ID | 入口与变异 | 可观察结果 |
|---|---|---|
| V-01 | 旧配置下定向 nextest list，修后三组真实发现和执行。该族至少一组额外走显式 prepare-only → run-prepared | 旧配置报告无匹配，修后保原有真实身份或条件 N/A。archive 可实际运行，无执行段补构建 |
| V-02 | 两个 AAC 固定组分别 prepared 真跑，检查有效并发。临时去掉一条限额，另用旧配置收据测试失效 | 正常各至多四并发且原测试执行。漏限额能被检出，错身份收据被拒。其他共享规则用实际有效配置或受影响代表组核验 |
| V-03 | G-02 已有共享构件时启动真实 leaky helper，再启动 clean helper。分别改写共享构件和实际 pin | 第一段拒绝并回收，不发布成功准备。第二段 env/收据同指 pin 且真验字节。共享路径变化可复用，pin 变化拒绝 |
| V-04 | 持有真实根租约，临时根跑 G-03，再模拟真实同根申请。合法收据作正控制 | 临时根到缺准备分支，未进入准备/执行。同根申请仍拒绝，原租约完整。合法收据能读并进入执行 |
| V-05 | 实跑 auto-playground 固定组，核 Cargo 发现日志和测试结果 | 原 E0004 消失，组真实执行。任何其他错误具名区分 |
| V-06 | 主检出 cargo t all 配合旧结果逐组对照 | 旧失败六组均完成其原合规选择与终态。全批结果据实，其他失败留实际阶段与责任，不记通过 |

命令例为 cargo t aavm auto-lang auto-lang-core auto-playground infra-meter infra-fixed --out <新目录>。AAC 每组执行 python scripts/fixed-tests.py <ace-aac 或 ace-aac-host> --prepare-only --out <准备目录>，再执行 python scripts/fixed-tests.py --run-prepared <准备目录>/prepared.json --out <执行目录>。同身份的有效完整组收据可复用。记录六个原失败组和 AAC 消费者的准备、发现、运行、清理、等待及命令墙。汇报新增用例的内容、期望和完整 item 时间。普通测试体零隐式编译，准备使用常规 debug 增量，遇签章材料固定 profile 时按现有合同具名处理。

## 7. 验收标准

| ID | owner_stage | 通过条件 |
|---|---|---|
| AC-01 | work | G-01 三个 root 组和两个 AAC 组的实际收据齐全。V-01/02 区分旧错误、限额丢失与配置失效，原选择和适用条件守恒 |
| AC-02 | work | infra-meter 完整组通过，V-03 区分真实泄漏、回收、正确 pin、错误 pin |
| AC-03 | work | infra-fixed 完整组通过，V-04 区分缺准备、租约碰撞和合法准备 |
| AC-04 | work | Playground 复验与历史归因对账完整。记录适用的模块时间、终态、原条件和新增 item≤10 秒。预算分母与显式组选择一致 |
| AC-05 | review | 同树独立核 AC-01…04、源码/有效配置/收据、遗漏和边界，绑定冻结提交，按需重放敏感负控 |
| AC-06 | merge | 落主后以一次正常 cargo t all 取得当前周期收据，原六组分别闭环。其余新错误按真实阶段具名归属。仅已验证债务可关闭，Spec、归档和守卫清理有收据 |

## 8. 执行步骤

- [ ] T-01 重冻六组和配置实际路径
  - owner_stage work。后续授权实施后开始，核源码、配置、依赖、nextest 版本和 G-04 的修复来源。为 V-01/02 写下普通、archive 和 archive 运行的实际接口与身份边界。AC-01/04。
- [ ] T-02 限定 AAC 资源规则作用域
  - owner_stage work，依赖 T-01。修 G-01，复用现有配置机制，运行 V-01/02。AC-01。
- [ ] T-03 恢复 meter 的 pin 与回收验证
  - owner_stage work，依赖 T-01。修 G-02，运行完整 infra-meter 与 V-03。AC-02。
- [ ] T-04 隔离缺准备测试的租约
  - owner_stage work，依赖 T-01。修 G-03，运行完整 infra-fixed 与 V-04。AC-03。
- [ ] T-05 验证六组及受影响消费者
  - owner_stage work，依赖 T-02…04。实跑既有 G-04 修复与其余原失败组，确认两个 AAC 的真实结果，补逐模块耗时、归因和 SD。AC-01…04。
- [ ] T-06 独立复审
  - owner_stage review，依赖 T-05。核有效配置、原选择、pin 与租约负控、分母和证据身份。AC-05。
- [ ] T-07 主检出周期复验与归档
  - owner_stage merge，依赖 T-06 通过及合入授权。V-06 归主检出周期机制，复用同身份且覆盖完整的有效收据。更新债务、规范和归档，按守卫清理。AC-06。

## 9. 复审记录

2026-11-20，stage=new，PLAN-1108 r1，outcome=pass（规划自检），next=T-01（实施授权后）。使用两份历史运行原始日志、配置、实际用例及现有主线修复形成合同。进度 0/7，本次验证为文档静态核对。

## 10. 待澄清事项

规划阶段无阻塞。T-01 验证安装版 nextest 的配置与 archive 消费接口。T-05 核 Playground 当前结果与源码漂移。后续 ace-batch、红项归属和工具 profile 优化各按§4.3 排程。
