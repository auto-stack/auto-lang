---
plan_id: PLAN-897
status: drafting
feature_name: experimental构建存储减量与缓存生命周期
plan_revision: 1
author: [Codex]
created_at: 2026-10-31
updated_at: 2026-10-31
current_step: 0
total_steps: 8
supersedes_spec_components: [docs/specs/auto-test-case/module-selection.md, docs/specs/repository-boundaries/build-cost-baseline.md]
new_spec_components: [docs/specs/auto-test-case/build-storage-lifecycle.md]
touched_goals: []
affects: [test-infrastructure, experimental-workspaces]
depends_on: [PLAN-895]
---

# [PLAN-897] experimental构建存储减量与缓存生命周期

## 0. 变更摘要

用户发现主检出及890/894各有数十GB experimental。只读核查确认：每树受管源码约10MB，几乎全部体积来自各独立Cargo workspace的target，三树唯一文件数据合计125.06GiB，其中唯一PDB约72.92GiB。不是Git worktree复制了30GB源码。将普通开发/测试调试产物减量、同树兼容构建输出复用及闲置缓存安全回收作为一个实施包；先冻结实际消费者与有界可行性结果，再改配置。不拆编译器模块、不把活动target跨树硬共享。

本轮仅起草；未修改构建配置、未删除缓存、未运行项目测试或建立实施树。

## 1. 目标

- G1：普通experimental开发/测试不默认生成完整PDB；保留显式完整调试入口，正式release/性能合同与原oracle不变。
- G2：同一worktree内兼容依赖减少重复构建目录；现有regular/batch、profile、平台、features及来源身份的必要隔离保持。先证明再切路径。
- G3：有可预览、可审计的闲置构建缓存回收与worktree收尾规则；不把target整树转存为计划证据，当前890/894不受本计划操作。
- G4：用相同完整受影响构建/测试闭包报告前后唯一文件数据、PDB/增量/其余分项及构建成本；不把删光缓存后的零占用宣称为稳态收益。

非目标：跨worktree共用可写target、junction/symlink或对活动产物硬链接去重；源码/测试/样本削减；改Golden、资源/性能门；重构root成员；改变Windows计量禁用wrapper策略；扩展AG/UI/Man外仓套件。既有plan-artifacts最近5个规则不机械套用于活动构建缓存。

## 2. 架构方案

保持独立workspace源边界与每个worktree的可写隔离；优先减掉普通dev/test全量调试信息，再在同树内评估兼容target命名空间整合。允许保留确有不兼容输入的命名空间，但须逐项说明身份差异，不能继续无解释的每workspace重复。

T-01先做有界实际闭包探针：选AV+MN两个共享依赖明显且有运行时shim的workspace，核debug/test低调试与显式调试的正确性、源码/config/profile身份和路径消费者。T-01形成明确路径/profile决策，后续任务才能执行。若原承诺不可兑现或要改变性能/诊断合同，回new明确修改，不默许弱化。

跨树编译复用继续仅使用既有有界sccache条件；它不能去掉本地target，而且Windows正式计量明确禁用RUSTC_WRAPPER/RUSTC_WORKSPACE_WRAPPER。不能把“开sccache”当本问题已解决，也不能为腾盘停共享daemon或改计量背景。

## 3. 技术栈

现有Cargo profiles/target-dir、Python test-governance、原生Windows Job、SHA256输入/工具身份；现有worktree/guard机制。新增清理入口的具体文件名由T-01确认，候选为scripts/test-governance/build_storage.py（新）。不引入新的全局编译产物服务。

## 4. 需求分析与背景调查

### 授权与边界

2026-10-31用户指出experimental在主检出、890/894重复占用、累计约120GB，延续其磁盘长期不可持续的诉求。此次已授权/执行的是本地只读核查与拟定修复合同；本r1构建profile/入口改动尚待用户确认。不得把之前对plan-artifacts的删除授权扩大为删除活动构建目录。

主检出按仓规仅计划簿记。本轮无build/test/清理；实施拟唯一树D:/autostack/.wt/lang-897/auto-lang，分支plan-897-dev。890/894仍在途；893/896可能涉及同一prepare/identity/选择入口，须按实际符号取写面，不把这些整Plan设为逻辑前置，不替他方停进程或清树。

### 已观察事实（2026-10-31，文件数据而非NTFS分配簇）

| 检出 | experimental唯一数据 | 其中唯一PDB | target外数据/受管源码 |
|---|---:|---:|---:|
| 主检出 | 48.59GiB | 27.24GiB | 10,440,311B，803受管文件 |
| lang-890 | 32.03GiB | 19.00GiB | 10,189,621B，772受管文件 |
| lang-894 | 44.45GiB | 26.68GiB | 10,293,869B，778受管文件 |

三树全局按device/inode去重为134,286,219,955B；PDB唯一合计78,299,488,256B。按路径相加总量更大（同树内部已有硬链接），不能把路径和当物理节省。os-051/ui-003的只读依赖树各仅约9MB且没有target，佐证Git建树不带构建缓存。

原始核查只在本地：C:\Users\zhaop\AppData\Local\Temp\experimental-storage-audit-20261031.json；SHA256=3d141db63d318417fd59c8ff2491e601f6b30236168b71d70a4d4c5dcc191ac8。未把原始清单/构建证据纳入Git。

### 当前依据与实际路径

- 代码基面观察433efce4e5f5a9e36dfde9c48ecb9a825f212d81；work开工须重核，不能消费890/894未交付WIP。
- docs/specs/auto-test-case/module-selection.md：895顶级/叶模块、原子守恒、实际消费者闭包、来源/profile/env身份；Windows计量禁用wrapper，准备/执行与有界Job保持。
- docs/specs/repository-boundaries/build-cost-baseline.md与已归档873：先实测、不因本问题拆root crate；既有无共享缓存计量与日常sccache态分列。
- .cargo/config.toml（SHA前缀11c09ea176623298）没有全局target-dir。scripts/new-wt-group.sh只git worktree add，不复制target。
- experimental/ac-av-probe/Cargo.toml（43c8cd77a8cdc52e）、experimental/ac-mir-native/Cargo.toml（e1e58592e563edd2）等是独立workspace；修改根Cargo.toml不能冒充已配置这些工程。
- scripts/test-governance/toolchains.json（97154ba0e3046d29）已将运行worker放到本树target/test-tools/{regular,batch}；scripts/test-governance/prepare_tools.py（037f41da96a5dc62）显式传--target-dir并绑定二进制hash。不能忽略这部分已合并的复用，也不能让路径重构抹掉batch隔离。
- Rust模块发现/单测及手工构建仍可在各experimental workspace下生成debug/test target；module_command.py（7be4ddc0e9effc11）与实际discover/run路径由T-01复核。
- 有硬编码shim消费者：ac-av-probe/probe/src/text_runtime.rs::trace_support_lib的test-support/target/debug/ac_trace_support.lib；ac-mir-native/src/collections_runtime.rs的显式target-dir；scripts/verify-ac-741.ps1、verify-ac-data-resources-774.ps1及对应真实消费者。路径改变须逐一承接，不能只把工具build跑绿。
- target中同时存在验证收据/探针产物（如ac-verify-receipts）及CACHEDIR.TAG标记的构建中间物；按目录名全部删除不成立。

## 5. 详细设计

### 下一任务入口与实现约束

T-01可先只读核现行Specs/895实际交付及890/894/893/896写面；真正编译探针使用897自己的树和受控窗口。冻结表列每个workspace的普通/调试/正式profile、target来源、必需lib/worker消费者、不可回收证据及正在运行/预留测量状态。

普通dev/test的候选为关闭完整调试信息；具体Cargo配置以T-01实验证明后冻结。保留显式全调试路径，不修改现行正式release/benchmark输入。profile、target命名空间、compiler flags、effective env变化参与构建及测试身份；旧绿不能无条件复用为新profile证明。

同树复用仅在目标triple、profile、features、锁与源码闭包兼容时成立，保原regular/batch隔离。由Cargo负责中间物指纹；实际发布的worker/lib在消费前仍绑定来源和hash，另一构建不得静默替换当前执行物。若shim需要兼容出口，可发布来源可核的固定副本；禁止链接进worktree和无来源复制。

清理默认预览，输出实际解析路径、种类、预计回收数据、保护原因。只操作明确本仓构建缓存/自有闲置根，拒绝reparse逃逸、tracked源码、未知目录及活动/计量预留。单纯没有cargo进程不代表两阶段测量已结束。target内必要证据先核复制及读取方；已有计划的来源绑定/首次失败记录不覆盖。实际清理与准备/执行采用互斥协调，失败/取消准确留收据，不强杀别的任务、不“清理成功”掩盖半删。

新worktree不预复制全target；结束按所属仓guard回收，不把可重建产物搬到plan-artifacts绕过收尾。主检出的周期清理只在其owner确认的空闲窗口执行；不能借本计划自动删除890/894缓存。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/build-storage-lifecycle.md | 未定义→普通/完整调试/正式构建分级、命名空间、来源身份与缓存/证据生命周期 | 避免每树重复数十GB及误删 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-test-case/module-selection.md | 现有模块守恒→补profile/路径变动的准备与实际消费者验证、保Windows Job与计量禁wrapper规则 | 不能靠缓存减量削弱选面或身份 | AC-02/03/05 |
| SD-03 | modify | docs/specs/repository-boundaries/build-cost-baseline.md | 既有873基线保持→另附本问题有界前后存储/成本与适用输入，不改历史数字或member结论 | 避免把清空缓存冒充收益 | AC-01/06 |

## 6. 测试设计

下表均为拟执行，不是已过证据。实现与内部自检同一897树；最终review独立上下文、同一树冻结SHA。未指定模型或新聊天/agent授权。

| case | AC / 不变量 | 实际入口/输入 | 必需观测/区分反例 | 边界与owner |
|---|---|---|---|---|
| V-01 | AC-01/06，真实存储 | 相同完整选面冷准备→执行→温态，原/候选普通profile | 源与case/特性分母相同，列unique/PDB/incremental/其余与总成本；不能删除产物后单测目录 | T-01/06，单轮不冒P90 |
| V-02 | AC-02，诊断能力 | 普通dev/test与显式完整调试两入口 | 普通不产出完整PDB；调试入口生成与PE匹配的符号；错误输入仍给原诊断/退出，既有实际case oracle不变 | T-02，正式release不改 |
| V-03 | AC-03，发布/来源 | AV/MN/shim实际消费者，regular/batch及两个不同源码/feature构建 | 正确配置实际运行一致；另一配置/错库或错worker不可被按原身份接受；输入改变旧收据失效 | T-03/05，不新增跨树共享 |
| V-04 | AC-04，生命周期 | 活动编译、两阶段计量预留、空闲缓存、含收据target、链接/越界/未知/只读共享文件 | 活动/预留/未迁必要证据保护；空闲可重建项真回收；链接不穿透、不chmod污染外部；中途失败不报全清 | T-04，有真实负控与合法清理 |
| V-05 | AC-05，正式模块守恒 | 原cargo tm选面/消费者及受影响完整并集，含必要shim | 新旧发现/选择身份守恒，编译/执行/通过分列，不能只用list-only或父包装exit0；Windows Job/资源条件保持 | T-05/07/08，未执行条件明确 |

门禁：先由变更路径文件通过cargo tm --changes <paths> --select-only --out <本地目录>确定完整模块/消费者并集，再运行实际选面；新清理/配置路由控制要有真实消费者。修改所有workspace profile将扩大实际受影响闭包，明确列出，不因磁盘优化抽样冒全绿。正式收益矩阵仍是独立里程碑，不新增三轮全目录或降低样本。895模块用途建议不构成本计划默认准入修改授权。

## 7. 验收标准

| ID | owner_stage | 可观察完成条件 |
|---|---|---|
| AC-01 | work | 三树来源/目标缓存/证据/实际消费者清单冻结；只读事实、假设和保护窗口区分，核查口径可重建 |
| AC-02 | work | 普通experimental dev/test不默认生成完整PDB，显式完整调试真实可用；正式release/性能档、原错误诊断及适用行为保持 |
| AC-03 | work | 同树兼容命名空间复用已实证；必要不兼容分区逐项具名；所有实际worker/lib消费者正确，错来源/配置拒绝且身份随输入失效 |
| AC-04 | work | 清理有真实预览/执行/失败收据，只清自有明确闲置缓存；活动/预留测量/必要证据/源码/外部链接受保护；不改890/894现场 |
| AC-05 | work | 受影响完整模块及消费者/集成闭包守恒、真实执行通过；未执行/旧红正确披露；Windows Job及计量wrapper策略不弱化 |
| AC-06 | work | 同闭包稳态前后文件数据/PDB/增量/缓存及准备/执行成本报告完成，减少幅度为实测而非承诺；原输入/正式性能合同不改 |
| AC-07 | review | 冻结SHA独立逐AC/V/SD复审、源码/配置/行为/生命周期反例和遗漏/workaround扫描通过，无未闭必需finding |
| AC-08 | merge | 冻结并集门、实际合入、SD文档沉淀/读回与归档、fresh guard/树与分支收尾、可重建缓存不转存证据；报告真实释放量 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 路径/交付与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 用户确认r1、895交付与实际符号写窗 | 核现行树/清单及原source/env指纹；同897树做AV+MN+shim有界可行性，冻结profile/namespace/cleanup ownership决策与V-01基线 | AC-01/02/03/06 |
| [ ] | T-02 | work | T-01 | experimental各workspace普通dev/test配置、显式完整调试入口；候选脚本路径T-01具名；V-02与原适用诊断正负 | AC-02 |
| [ ] | T-03 | work | T-01/02 | prepare_tools.py/toolchains.json、实际discover/run与硬编码shim消费者的target/profile身份承接；V-03、regular/batch和新鲜绑定控制 | AC-03/05 |
| [ ] | T-04 | work | T-01、清理入口实际owner约定 | 新清理入口/生命周期规则及worktree收尾接线；V-04真实合法/拒绝/失败，不触在途890/894；仅owner空闲窗口回收主检出 | AC-04 |
| [ ] | T-05 | work | T-02/03/04 | 源路径选cargo tm完整模块/消费者，跑实际入口与V-05/负控；修齐后冻结 | AC-05 |
| [ ] | T-06 | work | T-05 | 同闭包完整单轮稳态存储/成本对照，V-01/SD-01..03与全部AC自检；正式性能档保持；execution_done | AC-01..06 |
| [ ] | T-07 | review | T-06冻结 | 独立同树逐AC/V/SD/健康与遗漏审计，必要修复回同树定向验闭，不重复无关全档 | AC-07 |
| [ ] | T-08 | merge | T-07 pass | 冻结受影响完整并集与到期批次职责；实际合入/SD读回/归档/guard清树，不转存target；实际回收统计 | AC-08 |

## 9. 复审记录

2026-10-31 stage:new | PLAN-897 r1 | outcome:blocked（仅指implementation handoff等待用户确认r1，不是技术负结果） | authorization:用户磁盘调查诉求；本轮只读核查/计划簿记，未扩大为活动缓存删除或构建配置实施 | observations:三树125.06GiB、PDB72.92GiB、源码约10MB/树；当前存在890/894活动构建/测试 | checks:来源/路径/清单/选测Spec静态核对，未build/test/删除/建实施树 | next:用户确认r1后work从T-01起；T-01先定有界可行性与具体路由，不以简单全局target-dir代替身份/隔离设计。

## 10. 待澄清事项

- 待用户确认本r1实施合同。仓规L1要求“present for confirmation before executing”；这次不是对之前plan-artifacts删除授权重复确认。
- 无需用户决定的实现细节：T-01在现有规则内选择具体profile/同树目标目录与清理CLI；需要改变原正式性能/诊断/来源合同才回new说明差异。
- 本r1不承诺未经实测的GiB硬上限或速度增益。先保证普通全PDB默认关闭、可安全回收、同覆盖真实对照；若用户另有总盘预算再显式修订，不把此前证据目录10GiB预算套到构建缓存。
