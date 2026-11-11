---
plan_id: PLAN-1014
status: archived
feature_name: 固定回归六红基线与测试归属及执行租约有界修复
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12

plan_revision: 1
depends_on: [PLAN-1001, PLAN-1005]
supersedes_spec_components: [docs/specs/auto-test-case/build-storage-lifecycle.md, docs/specs/auto-test-case/ace-test-policy.md, docs/specs/auto-macros/project.md, docs/specs/auto-media-core/project.md, docs/specs/stdlib/project.md]
new_spec_components: [docs/specs/auto-test-case/fixed-regression-baseline-and-ownership.md]
touched_goals: [GOAL-002]

affects: [auto-test-case, auto-macros, auto-media-core, stdlib]
current_step: 12
total_steps: 12
---

# [PLAN-1014] 固定回归六红基线与测试归属及执行租约有界修复

## 0. 变更摘要

以`cargo t all` covered1005的六个known-red组为封闭修复集：测试归属/工具准备三项，宏接口、sys natives、媒体fixture三项；同时承接同执行链债务D-1001-R1/R3，并核对D-1001-R2五个ACE旧红的当前关闭状态。一个Plan、四条明确修复分支，统一按真实回归基线验收，不扩成全DEBTS清仓。

1005/1001保持归档；1013保持原合同，不往其中塞基线修复。**本包分组/runner/tool配置应先落地，再由1013冻结最终来源及采证**；1013可先开发，不依赖其先完成本包，避免循环依赖。本包不替代1013的facts准入修复或ACE M1签章。

本轮仅规划：drafting r1、0/12。只新建Plan和取号簿记；未施工、建worktree、跑回归、改DEBTS/Specs/1013、打tag或启动其他会话。

## 1. 目标

1. 消除六个具名旧红根因：当前语言仓合法固定组可发现/执行，不再引用迁出crate，不隐式构建ACE；迁出的测试义务有真实owner入口，不能删组即冒覆盖通过。
2. 日常legacy与显式ACE接线职责分明。核心ACE tests从legacy选择中排除后，须由既有显式ACE职责组完整承接；不能关守卫、全组改ACE或用ignore/NA掩红。
3. pilot按真实regular/batch工具配置准备，独立于主检出旧target/PATH；保持default-off拒绝batch的语义与原真实进程/计量断言。
4. fixed-tests正常/异常收尾及已确认死亡owner恢复正确，活owner、未知身份、权限不可判定与竞争替换不得被误清理；仅操作本树自有租约，不停其他会话进程。
5. 宏、系统函数、fixture修复保持产品语义/原oracle。六红逐项闭合、受影响组有效、最终主检出一次正式`cargo t all`全量门通过，并诚实列原条件NA和测试转移。
6. 相关债务逐项去重/闭账；D-1001-R2五项逐个有当前复证或明确未闭owner/下一动作，不能因1000归档就自动closed。发现仍影响M1 required的项，交1013签章前处理并保持阻断。

非目标：修全部历史898/896债、改C/Rust FFI/新标准库/M2范围、调试器/PATH全机治理、缓存大清理、外仓产品改造、修改1013/1005合同、放宽M1分母/阈值。本包仅语言仓代码与规范；auto-ui仅核已迁LSP来源/既有入口，不修改其源码/计划或冒称其完整产品测试已过。

## 2. 架构方案

分支A：当前owner/固定模块职责 → 原test-groups静态选择 → 显式prepare_tools → 原runner/父oracle → 完整原组与转移对账。分支B：`value!`展开 → 当前`Value::Obj(Box<Obj>)`构造契约 → 原消费语义。分支C：`libs::sys`采样 → VM native/值映射 → 真实Auto脚本断言。分支D：七份固定图片字节/PROVENANCE → 新增manifest → 原fixture_manifest格式/解码/orientation断言。

先冻结六红复现与原测试/模块归属，再做四分支修复和共用租约；T-09汇集闭账/ACE五项核对/1013交接，T-10独立复审，T-11冻结落地和一轮主检出正式全量门，T-12规范归档清理。不新增自动发现目录/叶ID执行系统或新总裁判。

## 3. 技术栈

现Python fixed-tests/prepare_tools/unittest、TOML固定组/JSON工具声明、Rust proc-macro/Value、旧VM sysinfo/native桥、serde_json/image及Git冻结证据。未来唯一树`D:/autostack/.wt/lang-1014/auto-lang`、branch=`codex/plan-1014-dev`，work/内查/独立review顺序共用，不另建review树或junction。本轮未创建。

材料唯一scope=`D:/autostack/plan-artifacts/plan-1014/`，开工登记owner/用途/在途状态；所有探针/日志/转储/外置target副本在此，正常构建在本树target。主检出正式batch原件沿其唯一batch scope登记，不造第二份副本，收尾摘要折入本Plan/正式收据后按政策删除。纯规划临时核对用系统TEMP。

## 4. 需求分析与背景调查

用户在只读分析后授权“根据你的分析新建一个修复计划”。继承当前本仓只规划、不施工、不启动其他会话的边界；本轮不改既有计划或canonical。未指定模型/工期/资源预算。后续执行需实际work指令，不因创建本Plan自动开始。

调研HEAD=`1169f135940e38e6a567826ce406785134e356e3`，branch=master。收据`docs/plans/.last-batch-regression.json`：covered_commit=`90e66211b057d2e159f5e32639347e2e56763893`、last_covered_plan_id=1005、37 invocations、2917真实执行、3原条件NA（aavm/a2r-actor-tests/auto-vm）、verdict=known-reds-only、新红0。scope已删，因此本包以当前代码重放，不能假称还能读取旧raw。该收据不是全绿，也不替代显式ACE/M1正式终验。

读取overview后以当前代码/模块Spec校正其旧仓库地图：`auto-lsp`实际位于`D:/autostack/auto-ui/crates/auto-lsp`（manifest已只读核存在），语言仓仍有同名固定组。当前未发现承接本六红修复的在途Plan；971的LSP工作是M2功能，不承担本基线修复。1001 R1/R3与900/901债合并关联，不重复算两份缺陷。

### 4.1 有界问题账

| ID | 原债务/旧红 | 当前事实与待实证边界 | 关闭owner |
|---|---|---|---|
| B-01 | PLAN-900/901，auto-lsp stale group | test-groups仍用本仓`-p auto-lsp`，crate已迁auto-ui | T-02/03，真实owner映射与本仓失效入口退役 |
| B-02 | PLAN-900/901、D-1001-R1，core ACE泄漏 | daily/core排除表达式仅列部分旧前缀；守卫仍检查所有选中ACE前缀；1001曾计81项，不把旧81当当前固定分母 | T-02/03，实际发现的模块/身份转移守恒 |
| B-03 | PLAN-900/901，infra-pilot-cli | 当前legacy/tools=[]，测试真实需要pilot/ACE；default_off硬读`target/test-tools/regular/release/av-worker.exe`；隔离旧报9/10不是完整闭合 | T-04，显式source-bound准备与职责拆分 |
| B-04 | D-896-BASELINE-01/900-N3/898-N5同一宏红 | `value!`生成`Value::Obj(obj)`，当前变体要求Box；具体四个E0308须重编复证 | T-06，宏契约修复/原完整组 |
| B-05 | 898-N5及sys_natives_vm fix-forward | 真实脚本含采样范围、processes记录字段、kill(0)、disk等；尚未证明哪条语义失败，不预设根因 | T-07，实际失败断言定位与最小修复 |
| B-06 | PLAN-007，fixture_manifest | 七fixture和PROVENANCE在库，manifest缺；原测试要求7条、文件类型和orientation=6 | T-08，补真实固定清单不改断言 |
| B-07 | D-1001-R3，执行租约残留 | 当前execution_lease已有try/finally，但mkdir/owner写入在try前；旧“无finally”描述不能代当前根因；硬退出也可能留目录 | T-02/05，真实失败/恢复路径复证与互斥保护 |
| A-01 | D-1001-R2，ACE五项旧红仍开放 | 1000归档而债表仍open，可能已修未闭账；不能直接把5项扩成新的产品任务 | T-01/09，逐项复证/去重/明确承接，required未闭则阻M1 |

### 4.2 源码/指纹与具体矩阵

下列为本轮SHA256；实施前核实际HEAD/源变化，不能照搬旧结果：

| 路径 | SHA256 |
|---|---|
| `docs/plans/.last-batch-regression.json` | `7afc76b3bd37f30b5cf4b60e2547411bb8b87ca111b1128844062896832dc3cb` |
| `scripts/fixed-tests.py` | `1b3705113062a086a2c9a8dfb77a97e061dbfe1a9d6c8aba586f1347a71bf213` |
| `scripts/test-groups.toml` | `b2f6f6b9203efe576ac2b0795f218338b05512e1d203f50b1ba7606bdc4f7453` |
| `scripts/test-governance/prepare_tools.py` | `37346de6dd60db8f408bd5d5a4123f0574ad447be8ec14ae7465f21f7b1e30fe` |
| `scripts/tests/test_case_pilot_cli.py` | `f86ebfb1363564e6d6858c51e22d669272993883c7e854cfc59d65029daf972f` |
| `crates/auto-macros/src/lib.rs` | `60cdb06ff54f7d7e81308e6c0706840e7699872a323d279760017d3b945ae750` |
| `crates/auto-lang/tests/sys_natives_vm_tests.rs` | `317198464ee77d58a9d7abe83661537b809275fb101bdea529a3be0bb5816984` |
| `crates/auto-media-core/src/image_pipeline.rs` | `316995da3f994c5f825cf90024882d499ff9f306a34781c410bd906e59d116aa` |

| owner | 现存路径/符号 | 拟改/核验范围及consumer |
|---|---|---|
| T-01..03 | `scripts/test-groups.toml`；`scripts/fixed-tests.py::configuration/select/invocations/enumerate_rust/run_rust` | 迁出组、daily/core模块职责和显式ace-adapters承接；实际list对账，不靠缩过滤通过 |
| T-04 | `scripts/tests/test_case_pilot_cli.py::PilotRoundAccounting`；`scripts/test-governance/prepare_tools.py::prepare`；`toolchains.json` | pilot/regular/batch声明、env及输出路径；case-parent现含test-case-pilot，av/batch-av已分target；必要regular配置/拆分测试文件标拟新增，T-02定稿 |
| T-05 | `scripts/fixed-tests.py::execution_lease/main/execute_plan`；`governance_io.py::machine_gate`；`build_storage.py::process_alive/inspect` | 优先复用现锁/身份设施；只修自有执行租约及必要消费，保缓存清理器活租约保护，不借机缓存清理 |
| T-06 | `crates/auto-macros/src/lib.rs`；`tests/value_macro_tests.rs`；`crates/auto-val/src/value.rs::Value::Obj/Value::obj` | 修宏展开接当前helper/装箱契约，默认不改公共Value布局；核嵌套/插值/单次求值 |
| T-07 | `crates/auto-lang/tests/sys_natives_vm_tests.rs`；`crates/auto-lang-core/src/libs/sys.rs`；`src/vm/native/natives_sys.rs` | 实测定位采样/native ABI/记录字段/旧VM接线，沿真实owner最小修，不把浮动采样常量化 |
| T-08 | `crates/auto-media-core/src/image_pipeline.rs::tests::fixture_manifest`；`tests/fixtures/PROVENANCE.md`及七原件；拟新增同目录`manifest.json` | 原件固定清单/格式/方向；不引UI依赖或测试时再生原件 |
| T-01/09 | `experimental/ace/crates/a2x-worker/tests/cases/contract_certification.rs`；`experimental/ace/tests/conformance/tests/host/{rust_lowering,semantics_protocol,source}.rs` | D-1001-R2五具名测试当前重放，归属ace-a2x/ace-host；不是直接宣称全ACE绿 |
| T-09..12 | `docs/plans/KNOWN-DEBT-AND-RISKS.md`、正式batch收据、本Plan、1013来源交接、下列SD | 债务逐项关闭/当前证据/主线全量门与1013冻结输入；本轮不改这些旧文件 |

## 5. 详细设计

### 5.1 阶段合同与执行入口

| 阶段 | 真实输入→输出/owner | 关闭条件 |
|---|---|---|
| S0：T-01/02 | 当前代码/旧收据→六红复现、当前完整模块选择、ACE五项复证、租约路径小探针 | 固定问题边界与输入身份；归属/prepare/租约机制决定记本Plan后依赖实现才开工 |
| S1：T-03..05 | owner矩阵→固定组/工具声明/原runner与租约 | legacy零ACE、转移义务守恒、pilot干净环境真实运行；活/未知owner保护、死owner可恢复 |
| S2：T-06..08 | 当前宏/Value、sys/native、七fixture→原产品oracle | 三类根因修复、完整职责组有效，原断言不弱化 |
| S3：T-09 | 全分支当前事实→冻结候选/债务去重/1013交接 | 七个B全部闭合，A-01五项各有当前裁定；work任务9/9完整才execution_done |
| S4：T-10 | 同树独立上下文→AC/V/SD及遗漏/健康复核 | 独立pass才reviewed；任何本包红/必要oracle缺失不得仅记债通过 |
| S5：T-11/12 | 冻结H→主线C/一次cargo t all→规范/收据/归档清理 | 全量门真实绿、原NA诚实、转移对账、1013冻结身份可用；实际清理结束才12/12 delivered |

### 5.2 测试归属与工具准备

T-02基于真实nextest/unittest发现冻结`旧group/模块→当前owner group→实际命名身份/原条件→验证入口`。仅此次迁移用一次性转移表，不建永久叶目录/动态影响图。legacy选择中排除的每个ACE模块须由显式ACE职责组完整覆盖，保ignored/平台/压力条件；新未知ACE前缀仍应触发守卫。不得放宽守卫、把整个core/daily标ACE、将全部旧义务统一all=false或NA来复绿。

auto-lsp本仓失效注册退役须明确“迁出”及auto-ui现存manifest/既有可执行owner入口；不把本仓不再运行解释为LSP测试通过，也不要求语言all构建外仓。本包不改auto-ui；若现owner缺入口，具名阻断转移验收并正式交接，不能空填路径通过。

pilot按职责拆：纯Python/legacy检查留无ACE准备路径；真正pilot/ACE检查接显式ACE组，normal/default-off和batch工具以现source-bound prepare机制分别构建/钉hash/env，准备与运行分账。同一套件混regular/batch不能依赖一个可变全局WORKER/PATH或未声明旧exe。T-02决定最小拆文件/类或固定组声明，新增路径/名字在实施时注册。原default-off：independent真实通过，group模式明确拒绝、NOT_RUN，不fallback regular冒batch；保实际PID/worker_starts/计量等原断言。

### 5.3 租约与产品修复边界

租约状态：无owner→原子独占获取→写已核owner→运行→本owner释放；遇存量租约先验证本树归属/进程身份/锁，活或不可判定保持拒绝，确认死亡后在互斥与重验下恢复。含owner写入失败、guard/prepare异常、cleanup失败、owned helper硬退出以及下一次真实获取的闭环。不得只根据mtime/PID格式认死，不按名字杀进程，不删除未知目录。PID复用/竞争更换不能清新owner；旧格式无法证明身份要保守拒绝并给具名恢复信息。现已有finally，T-02先定位实际断点，不规定重复加finally就是修复。缓存治理consumer仍应拒活fixed-test lease，不进入真正删除缓存流程做正例。

宏修复优先接`Value::obj`等当前构造合同，不反改Value布局来迁就旧展开。保对象键值、插值、嵌套与表达式单次求值；原ignore带有未支持语法原因，本包不以顺手unignore扩解析面，也不新增ignore掩编译红。

sys先重放定位具体失败断言/值类型、采样时序与映射；当前源不证明CPU/进程/kill哪项为根因。修支持域内最小接线/表示/采样错误，合法浮动数据按原语义断言，保`kill(0)==false`且不操作真实用户进程。若原断言与平台合同矛盾，须给现规范/系统行为证据修正契约与正反例，不能为了绿放宽成永真、清空数据或固定返回值；涉及承诺变更先needs_replan。

fixture补七条原件真实manifest，文件名/类型/orientation与源PROVENANCE和字节核验相符，不改图片、不测试时镜像UI。原7条/魔数/损坏检测/alpha/orientation断言保留；拟新增manifest内容须review核来源，不靠“JSON能读”通过。

### 5.4 债务、ACE五项与1013交接

必须关闭B-01..07；B-04的三处历史债引用同一修复证据，不把D-896整条内其他crate顺便标closed。只处理本六红和R1/R3，其他D-1001-R4/R5/R6或已closed R7不纳入。任何新发现先具名分诊，不自动全债扩包。

A-01五项：`certify_dry_run_b01`、`a2r_hir_first_case_green`、`a2r_hir_worker_credential_pin`、`abi_recipe_binding_exclusivity`、`av_source_inventory_denominators_replayable`。每项记录当前源/工具/测试身份、重放结果、既有关闭依据或未闭根因、M1 required关联与实际owner。已修未闭账补当前关闭记录；仍红不能用旧“归档为未观测”给M1放行。本Plan的A-01完成是可信分诊/交接，不自动承担五项全部产品修复；命中1013既有准入/来源范围则交其work，否则给具名有界后继需求与owner，并明确M1仍阻断，不能写回已归档1000要求执行。若A-01问题直接导致本Plan七B或最终全量门不通过，则是本包阻断，不能仅交接后宣称本包完成。

无循环依赖：本包不依赖1013完成；交付本包代码/完整C、分组/runner/tool/pins的最终hash及转移表后，1013重新核来源、冻结采证。若1013已提前采证，受本包变更影响的观察/R必须按其合同失效重采，不能仅改hash续用。不修改1013文件或重开1005；本包不创建ACE tag，不把全量门绿当M1签章。

### 规范增量

仅拟议，review定稿、merge沉淀；本轮不改canonical。

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-test-case/fixed-regression-baseline-and-ownership.md`（拟新增） | 固定组当前owner、迁出/ACE模块转移、六红闭合、完整门结果与条件NA口径 | 防删组/缩coverage冒绿 | AC-01/02/03/08/10 |
| SD-02 | modify | `docs/specs/auto-test-case/build-storage-lifecycle.md` | 补执行租约获取/异常/死owner恢复/未知保护及活lease consumer语义 | R3闭环，保活动数据 | AC-04/07 |
| SD-03 | modify | `docs/specs/auto-test-case/ace-test-policy.md` | 补本包ACE接线/pilot regular/batch显式prepare与转移导航，不改原13项策略分母 | legacy隔离与真实工具一致 | AC-02/03/08 |
| SD-04 | modify | `docs/specs/auto-macros/project.md` | 校正现Spec只提注册宏的陈旧描述，沉淀实际value!当前构造/求值契约 | Box构造修复 | AC-05 |
| SD-05 | modify | `docs/specs/auto-media-core/project.md` | 明确七fixture固定manifest/来源/验证，与UI live输入分开 | 补缺文件而非改oracle | AC-06 |
| SD-06 | modify | `docs/specs/stdlib/project.md` | 按实际根因补sys native既有支持域/类型与采样观察说明，不新增系统API | 真实语义闭红 | AC-07 |

## 6. 测试设计

均为待实施，不是本轮执行结果。原输入/反例由T-01固化，新增永久断言由对应task实现并接原loader；已有fixture/原义务不为本包缩减。

| Case | AC | 输入/真实入口 | 可观察oracle/区分性断言 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-01 | 当前六红分别经原固定组/具名case；收据与源码比对 | 记录原失败形状/原因，已自然转绿也须说明当前证据；编译失败不算用例已执行 | T-01；六项 |
| V-02 | AC-02 | 当前nextest选中集合，legacy与ace-adapters及其他实际owner转移表 | 每个迁出模块有显式执行归属，缺一项测试即失败；原ignore/平台条件守恒，非永久叶系统 | T-02/03；所有实际转移项 |
| V-03 | AC-02 | 人为向legacy注入真实ACE测试前缀；合法普通legacy对照 | 守卫在执行/工具准备前拒绝，实际legacy组零ACE worker；未知新增前缀仍被抓 | T-03；原run_rust/prepare |
| V-04 | AC-02/10 | stale本仓LSP注册→退役/owner入口映射与静态选择 | 本仓不再调用不存在package；auto-ui真实manifest/既有入口可定位，不能转成NA/PASS冒已跑 | T-02/03；迁出非跨仓产品验收 |
| V-05 | AC-03 | 无旧target/PATH依赖的pilot准备→现真实CLI independent/group；regular与batch分别测试 | 声明工具按源准备/hash/env可核；default-off group拒绝且NOT_RUN、batch真执行，原PID/计量oracle保留 | T-04；显式ACE专属树target |
| V-06 | AC-03 | 缺/错regular或batch工具、env指向另一形态/旧hash；合法当前工具对照 | 准备/身份阶段拒绝，无mock/fallback/skip冒绿；不因机器碰巧有exe通过 | T-04；工具输入和消耗 |
| V-07 | AC-04 | 本owner正常、guard/prepare异常、owner写入失败、cleanup异常 | 本owner释放或具名非零残留，保原失败诊断；下一实际获取不被陈旧目录永远阻塞 | T-02/05；全部租约阶段 |
| V-08 | AC-04 | 自有helper活/硬退出、PID复用/未知/AccessDenied、并发竞争替换租约 | 确认死亡才互斥重验恢复；活/未知/新owner字节不动；恢复后下一运行真实成功；不杀外部进程 | T-05；同树租约 |
| V-09 | AC-04 | build_storage现consumer遇活fixed lease、合法已清租约 | 活时仍拒，清后只读inspect准入；不需实际删除缓存证明本修复 | T-05；缓存治理消费 |
| V-10 | AC-05 | value!对象/嵌套/插值/单次副作用及原测试；对应consumer | 四E0308消失，值/键/嵌套正确且表达式只求值一次，完整组通过；不改Value布局/ignore压红 | T-06；当前宏支持域 |
| V-11 | AC-07 | 原sys真实Auto脚本、失败最小输入、同类型合法/非法控制 | 明确断言实际值/阶段与根因；原完整功能通过，类型/字段/采样约束不永真，kill0不杀真进程 | T-07；原支持平台 |
| V-12 | AC-06 | 原七fixture+新manifest→原fixture_manifest；临时错误条目/方向负控 | 完整七项/格式/损坏/alpha/EXIF正例；缺/错条目原oracle能拒绝；图片字节不变 | T-08；永久oracle原有则复用不造镜像测试 |
| V-13 | AC-08 | A-01五具名ACE测试在当前源/工具下重放并映射M1 | 五项各有真实当前结果/身份，绿才closed；红有owner/required关联和阻断，1000归档不代通过 | T-01/09；五项不扩为全ACE清债 |
| V-14 | AC-08/09 | 冻结完整受影响组并集、旧负控及一项移除转移/工具/租约保护的受控坏机制 | 组/子项真实终态；相应控制对该坏机制敏感，恢复后绿；独立review核AC/SD/遗漏 | T-09/10；实际受影响路径 |
| V-15 | AC-10 | 最终主检出一次cargo t all与显式转移ACE组证据、旧六红逐项闭账 | 全量gate真/exit0、六红无残留，原NA有依据；新invocation/执行数量逐项解释，不硬凑旧2917 | T-11；正式batch，不混M1终验 |
| V-16 | AC-10 | C/分组/runner/tool最终hash→1013；归档与所有输出路径清点/清理 | 交接可定位、受影响旧证据失效规则明确，实际清理后12/12，无未知路径误删或假raw可读声明 | T-09/11/12；签前衔接/交付 |

验证入口与执行责任：

- 治理完整组：`cargo t infra-fixed infra-pilot-cli infra-ace-policy`；pilot拆到新显式组后使用T-02定稿的真实组名，旧组纯legacy部分仍完整测。新增组是拟实现，不能提前当现API。ACE接线完整consumer：`cargo t ace-adapters ace-batch`及实际转移owner组；配置/loader变更还补`infra-ace-m1-arch`等真实消费，不以普通legacy组通过替ACE。
- 产品完整组：`cargo t auto-macros auto-val auto-media-core auto-lang auto-lang-core vm stdlib daily`，同配置组一次并集调度，不重复机械刷；若无Value实现变更，auto-val作为宏必要consumer验证理由记明。宏/媒体/系统分别定向迭代，冻结后取完整并集；共享VM/Value触面扩大时补真实下游consumer。
- A-01定向原入口：`cargo t ace-a2x --test-name certify_dry_run_b01`；另四项分别`cargo t ace-host --test-name <完整具名测试>`，T-01先核实际发现身份。若执行揭示相关产品需改则先分诊，不借诊断任务自动扩大实现。`ace-host`/`ace-a2x`等现组名均已静态核配置。
- 本包是明确基线治理任务，要求**一次最终主检出单实例`cargo t all`**，由T-11按batch范式读取当前真实结果、写收据；不在worktree跑这个正式batch，也不要求每阶段重复全仓。此前只在唯一树跑上述受影响完整组。正式batch等待既有测量/重构建窗口结束，保重内存/Job/超时/输出原预算，不停其他会话。
- 发现/实际执行/原条件NA/ignore分别计，准备冷构建/运行/清理/总时分别列；父exit0不代子项，known-reds-only不当全绿。全量门若出现本包外新红，先隔离三角化，保完整红收据、具名阻断；未经范围决定不扩修，也不改NA/收据压绿。

角色：work实现与自查T-01..09；T-10独立上下文同树复核AC/V/SD、遗漏/债务与健康，记录真实身份、HEAD和指纹，实现停写，review不顺手修代码。用户未指派reviewer/模型，本轮不启动其他会话/代理；后续自查不能代独立复审。needs_fix回同树work，实质变更使受影响冻结证据失效。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 | 核验/任务 |
|---|---|---|---|
| AC-01 | work | 六红当前输入/失败形状、原owner/模块/工具/源码身份冻结，旧收据不冒当前执行 | V-01；T-01/02 |
| AC-02 | work | LSP stale注册退役且owner交接真实；daily/core合法legacy通过、全部迁移ACE义务有显式组，守卫不弱化/无无据NA | V-02..04；T-03 |
| AC-03 | work | pilot真实normal/default-off/batch按声明准备，源/hash/env及各原断言成立，干净状态不依赖旧exe/PATH | V-05/06；T-04 |
| AC-04 | work | 正常/异常释放与死owner恢复闭环，活/未知/竞争新owner不动；缓存consumer仍保护活租约 | V-07..09；T-05 |
| AC-05 | work | 宏四E0308根因关闭，原语义/单次求值/完整组绿，Value布局与原ignore条件不偷改 | V-10；T-06 |
| AC-06 | work | 真实七fixture清单补齐，原格式/解码/alpha/orientation oracle与完整媒体组绿、字节/来源可核 | V-12；T-08 |
| AC-07 | work | sys实际失败根因/值类型定位，原脚本及完整相关组绿，未以恒真/假数据/无据平台豁免掩红 | V-11；T-07 |
| AC-08 | work | 七B逐项关闭/债务去重；五ACE旧红各有当前裁定，未闭者owner/required关联/下一动作真实，1013交接无循环 | V-13/14/16；T-09 |
| AC-09 | review | 独立核AC-01..08、适用V/全部SD/遗漏/健康，完整受影响并集/区分性负控有效；无本包未闭项或越权延后 | V-14；T-10 |
| AC-10 | merge | 冻结H→主线C对账、一次正式all绿、NA与转移诚实，Spec/当前收据/1013来源交接完成；归档和实际路径清理后12/12 | V-04/15/16；T-11/12 |

本包完成只证明六红基线和指定执行链债务关闭；ACE M1仍须1013及所有required真实闭合、独立R/终验/TAG门满足。本包全量绿不能反向证明facts来源可信，也不自动创建`v0.6-ACE-M1`。

## 8. 执行步骤

唯一权威任务表，均未执行；12项=work9+review1+merge2。开工先按§3核仓/实际树/branch/HEAD/依赖与材料登记，计划簿记留主检出。

| Task | owner_stage | 依赖 | 操作/实际文件与输出 | 验证与AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 1001/1005交付 | 重放六红、固化来源/模块及收据；定向复证A-01五ACE旧红，保新红/环境/绿分别归因 | V-01/13，原六组与§6定向命令；AC-01 | [x] |
| T-02 | work | T-01 | 真实list转移矩阵，pilot两工具端到端小探针、租约实际异常/死owner前探；确定最小机制/新路径/组名并写本Plan | V-02/04/05/07/08，小探针不是全组通过；决策齐再开对应实现；AC-01..04 | [x] |
| T-03 | work | T-02 | 修test-groups及必要loader：LSP迁出、daily/core与显式ACE职责；保守卫/原条件，转移原测试不丢 | V-02..04，infra-fixed+daily/core+转移ACE完整组；AC-02 | [x] |
| T-04 | work | T-02/03 | pilot按职责拆分/注册、toolchains和prepare_tools最小接线，regular/batch source-bound工具/env真实消费 | V-05/06，现/新pilot完整组和必要ACE consumer；AC-03 | [x] |
| T-05 | work | T-02 | 修execution_lease真实失败/原子恢复，复用已核互斥/身份设施，核build_storage保护 | V-07..09，infra-fixed完整组+真实owned helper闭环；AC-04 | [x] |
| T-06 | work | T-01 | 修宏展开当前构造契约，复用/补原永久对象/插值/单次求值断言 | V-10，cargo t auto-macros auto-val及真实consumer；AC-05 | [x] |
| T-07 | work | T-01/03 | 定位sys实际失败值/断言/桥接，再做最小产品修复和区分性原支持域控制；完整core组验收须待T-03归属修复 | V-11，sys具名迭代后auto-lang/auto-lang-core/vm/stdlib完整并集；AC-07 | [x] |
| T-08 | work | T-01 | 核七fixture/PROVENANCE/源清单，新增真实manifest，保图片字节及原断言 | V-12，cargo t auto-media-core及真实必要consumer；AC-06 | [x] |
| T-09 | work | T-01..08 | 冻结H/受影响完整组证据，七B债务闭账去重、A-01五项当前裁定/阻断及owner，准备SD与1013最终输入清单 | V-13/14/16，AC-08；七B未闭不进execution_done，九work齐才可交review | [x] |
| T-10 | review | T-09，独立上下文 | 同树停写，从实际代码/原件核全部AC/适用V/SD、转移守恒/独立oracle、遗漏/健康与债务，绑定最终H | V-14，AC-09；merge未来项只审合同，不预勾已执行；needs_fix回work | [x] |
| T-11 | merge | T-10 pass | 冻结完整职责组并集有效证据对账，落地C/沉淀SD-01..06；主检出单实例一次正式cargo t all，收据按真实结果更新，记录1013 config/runner/tool输入身份 | V-15/16，AC-10落地；all红不标完成，不重写历史covered1005收据冒当时全绿 | [x] |
| T-12 | merge | T-11 | 折叠原件/反控/全量门摘要到Plan；归档、核所有实际输出/活动进程/链接/具名持有，wt-guard clean后清树/scope并刷新retention/终态 | V-16，AC-10实际清理；未知归属不盲删，真实12/12才delivered | [x] |

下一依赖就绪任务T-01；本轮不启动。T-02为既有批准语义/预算内有界调查，不是重复许可；支持域/验收阈值/外仓写入需实质改变时才另行裁定。

## 9. 复审记录

2026-11-12：stage: new；plan_id: PLAN-1014；plan_revision: 1；outcome: pass（规划可交接，非实现/独立复审已过）；next: work（本轮不执行）。新增T-01..12、AC-01..10、V-01..16、SD-01..06；七B修复与A-01五项分诊各有owner/关闭边界，和1013明确先配置落地、后冻结采证，无互相等待。

编号通过本仓new-plan.sh在master取1014；取号前互斥核active/archive/旧archived最大1013、counter1014，推进1015。技能旧三位限制不适用于当前已支持多位编号的仓流程，不截号。本会话仅计划簿记，不提交他人已在WIP中的counter历史改动或其他文档。

规划静态核对通过：§0..10齐、12任务定义唯一且均未勾、owner_stage=work9/review1/merge2，10AC/16V/6SD连续且有映射，八份来源指纹、16个既有职责组名与1013文档hash均核实。规划期间其他工作分配了1015，counter已推进1016，未覆盖或回退；也观察到1007/DEBTS的并行文档变化与其他新文件，本会话未改写/清理这些现场。未修改源码/测试/canonical/1005/1013。所有实施/产品测试/独立review/正式all/归档清理仍pending，本次静态核对不计task完成。


## 9.1 执行记录（work r1，2026-11-12）

### 身份与指纹

- 树：`D:/autostack/.wt/lang-1014/auto-lang`，branch=`codex/plan-1014-dev`，base=`1169f1359`（=调研HEAD）。代码提交 `eb278ca79`（实现单元，16 files +430/−197），簿记提交 `c601d2420`（债务闭账）。
- §4.2 八份指纹实施前复核：**8/8 MATCH**（按计划表逐项 SHA256；首轮脚本转录两处笔误误报，解析计划表直比后全对）。
- 材料 scope `D:/autostack/plan-artifacts/plan-1014/`（owner/用途/在途已登记 `.retention/policy.json`）：probe-sys-cpu（sysinfo NaN 排除探针）、probe-vm-sys（VM 同入口逐步标记复刻）、probe-lease.py（租约 7 态探针）、extract_diff.py+a01-2-diff.txt（A-01 项2 差分摘录）、fixed-groups-cases.txt（case-file 旁路）。

### T-01 六红重放（全部复现，根因逐项实证）

| 红 | 当前形状（命令→结果） | 根因（实证） |
|---|---|---|
| auto-lsp | `cargo t auto-lsp`→FAIL「package ID specification `auto-lsp` did not match」 | crate 已迁 auto-ui（`D:/autostack/auto-ui/crates/auto-lsp` manifest+tests 实核），本仓注册失效（B-01） |
| auto-macros | `cargo t auto-macros`→FAIL 4×E0308（discovery/build failed） | `handle_interpolated_value` 生成 `Value::Obj(obj)`，Plan 566 后变体要求 `Box<Obj>`；helper `Value::obj`（value.rs:215）在位未用（B-04） |
| auto-lang（sys） | `cargo t auto-lang --test-name test_sys_natives_vm`→FAIL「Assertion failed at line 49」 | 行号含前导空行→line49=`assert(p0.name.len()>0)`；探针隔离：`p0.name`="auto.exe" 而 `p0.name.len()`=**0**。根因＝VM `ARRAY_LEN`（engine.rs:2139-2170）对 GET_FIELD 新入池 rc==1 串**先 rc_release 后读**→pool_free 清槽读空；与已修 ADD 同族（engine.rs:3192 先例+回归）。子代理独立上下文以 AUTO_VM_TRACE_OPS/P419_POOL_LOG 活证据定案（B-05） |
| auto-lang-core | `cargo t auto-lang-core`→FAIL「ACE host tests leaked into a legacy group」 | 枚举式排除漏 7 模块 58 例（abi::source_protocol_tests 2、ac_av::resource_915_tests 10、ac_av::resource_events::tests 12、ac_av::worker_json_tests 4、ac_mn::refusal_tests 8、ac_mn::resource_events::tests 7、ac_mn::source_diag_tests 15）；**新事实：daily（all=false 未入 @1005 批）与 integration 同表达式同漏**——修复面扩为三组（B-02） |
| auto-media-core | `cargo t auto-media-core`→54 例仅 `image_pipeline::tests::fixture_manifest` 红 | `manifest.json` 缺失；7 fixture+PROVENANCE 在库且与 auto-ui 源 7/7 字节一致（SHA256 核验）；根因＝根 `.gitignore` `*.json` 母规则吞掉迁移清单（PLAN-892 金样同缺陷类）（B-06） |
| infra-pilot-cli | 主检出@1005＝3失败+1错误；冷树复放＝**PASS 但仅选 2/10** | `PilotCLI` 8 例 `skipUnless(EXE/WORKER存在)` 依环境残留 exe——冷树整类静默 skip（覆盖蒸发），主检出则工具未声明而红；组 `tools=[]` 而测试真实需要 pilot/ACE 工具（B-03） |

### T-02 转移矩阵与机制决策

- **转移表（发现法冻结）**：8 模块（上表 7+`ac_a2r_hir::path_domain_tests` 3 例）→ owner=`ace-adapters`（其表达式 `test_cases::(ac|abi|sema|ace_)` 正向选择同族）。守恒验证：ace-adapters `--discover-only` 84 例/17 模块，8 模块全 COVERED；实跑 `cargo t ace-adapters`=**84/84 PASS**。
- **auto-lsp 退役映射**：owner 入口=auto-ui 仓既有测试装配（crates/auto-lsp/tests/{integration,lsp_protocol}_test.rs 实核），本仓 `-p auto-lsp` 组删除；不声称本仓 LSP 测试通过，也不要求 all 构建外仓（V-04）。
- **pilot 工具决策**：`av-regular` 新工具（ace-av 无 feature、release、target=`target/test-tools/regular`）物化 default-off 原路径；`batch-av`（test-case-batch、target-batch）供 group/combined；`case-parent` 供 pilot exe；新 ACE 域组 `infra-pilot-ace`（all=false，与 ace-host 同为显式选择——受 test_all_is_explicit_language_only 约束不进 `cargo t all`）。
- **租约机制决策**：复用 `build_storage.process_alive`（tasklist 活性、unknown→alive）+ rename 原子互斥 + 隔离区重验；不新建锁设施。
- **V-03 守卫敏感复控**：伪造 legacy 组选 `test_cases::ac_mn::refusal_tests::` → 守卫拒绝（执行前）；合法对照 `infer::` 104 例 PASS。修复前 daily/auto-lang-core 守卫实跳即天然正例。

### T-03..T-08 实现与验证（全部在 eb278ca79）

- **T-03**：daily/integration/auto-lang-core 三处族式排除 `test_cases::(ac|abi::|sema::|ace_)`（daily 保留 tests:: 原排除，语义不变）；auto-lsp 组退役留注释。验证：`cargo t auto-lang-core`=**1602/1602**（守卫过后仅剩两个**隐藏存量红**，见下）；`cargo t daily`=2654 例、`cargo t integration`=3321 例，各余 1-2 例**环境归因**红（见下）；`cargo t infra-fixed`=60/60、`infra-ace-policy`=11/11。
- **守卫揭示的两个 pristine master 存量红**（主检出具名复放同红实证，非本包引入；均在 stdlib 触面=affects 声明域内修复）：
  1. `plan884_provider_input_coverage`：`trans/rust/{byte_lists,res_obs}.rs` 落库未入 Vm provider 指纹闭包→补登记（manifest.rs）。
  2. `test_substr_clamps_to_char_boundaries`：三 oracle 冲突（本测试钉 a2r_std 向内钳制值 vs 995 h4 步进重建 oracle 钉 VM 双向回退 vs a2r-std 自有测试 `partial_characters_negative_and_overflow_do_not_panic` 钉向内值）。终裁：VM 保持 995 双向回退（h4/真实丢字修复背景）、a2r_std 保持向内（其自有 oracle）、parity 测试改钉 VM 实际值并文档化分叉域（边界对齐输入两后端严格同值；字符内输入 VM 保整字/转译侧向内）。中间方案（曾同步 a2r_std 双向回退）被 a2r-std 自有测试否定后回退，过程留 strings_json.rs 注释。
- **T-04**：拆分 `test_case_pilot_cli.py`（纯 Python 2 例留 legacy）/`test_case_pilot_ace.py`（PilotCLI 8 例）；`toolchains.json` 增 `av-regular`；新组 `infra-pilot-ace`（tools=case-parent/batch-av/av-regular）。验证：`cargo t infra-pilot-ace`=**8/8 执行 0 skip**（default-off regular worker 落 `target/test-tools/regular/release/av-worker.exe` sha256=60c631d9…、batch=85c45439… 两形态可区分；group 模式真跑、independent 真跑）；`cargo t infra-pilot-cli`=2/2。
- **T-05**：`execution_lease` 重写（原子获取/owner 写入入 try+回滚/finally 具名清理失败/`_reclaim_dead_lease` 本树+确认死亡+rename 互斥+隔离区重验/活-外树-不可读-未知内容保守拒）；`build_storage` 消费者改验 pid 活性。验证：probe-lease.py **7/7 PASS**（正常、死恢复、活拒、外树拒、不可读闭、未知内容拒且不删、写失败回滚）；消费者探针：活租约→`lease is active (pid)` 拒、死租约→越过租约门（后续 evidence-mix 正交守卫按设计拒）。
- **T-06**：宏一行修复+注释。验证：`cargo t auto-macros`=**13/13**（4×E0308 消失，插值/嵌套/单次求值原断言全保）。
- **T-07**：ARRAY_LEN 先读后放+2 例引擎回归（rc1 读前值、CJK 字符数）。验证：`cargo t auto-lang --test-name test_sys_natives_vm`=PASS；回归 2 例执行绿；`cargo t a2r-std`=115/115、vm+stdlib=1046/1046（a2r_std/下游触面）。
- **T-08**：manifest.json 从 auto-ui 源补齐（哈希 b09a719c… 双侧一致）+.gitignore 豁免行。验证：`cargo t auto-media-core`=**54/54**。
- **M1 准入钉链刷新**（infra-ace-m1-arch 真实消费，按其准入设计）：`original-budget-contracts.json` 的 fixed-tests.py 源钉→新值；`ace_m1_budget_contracts.py` CONTRACT_SHA→d0f48415…；`registry.json` budget 身份钉同步。`cargo t infra-ace-m1-arch` 全绿（含运行时篡改负控）。

### 冻结并集（V-14 输入）

`cargo t auto-macros auto-val auto-media-core auto-lang auto-lang-core vm stdlib daily infra-ace-m1-arch`（worktree，eb278ca79）：auto-macros 13 ✓、auto-val 150 ✓、auto-media-core 54 ✓、auto-lang-core 1602 ✓、vm+stdlib 1046 ✓、infra-ace-m1-arch ✓；auto-lang 1751 例与 daily 2654 例各余环境归因红：`plan702_ui_path_busy_wait_retired_gate`（读 `../../../auto-ui` 跨仓路径，worktree 组无兄弟→扫 0 文件；**主检出具名复放 PASS**——正式门在主检出，不受影响）与 `plan707_spike_cancel_gap_active_job_future_not_stopped`（负载时序敏感；**两树隔离复放均 PASS**，按既有时序归因纪律记录）。

### A-01 五项当前裁定（V-13；HEAD 树内定向重放）

| 项 | 组 | 当前结果 | 裁定 |
|---|---|---|---|
| certify_dry_run_b01 | ace-a2x | **PASS** | 已修未闭账→随本收据闭（D-1001-R2 更新） |
| a2r_hir_first_case_green | ace-host | RED | a2x-emit 产物字节差（a2x_runtime.rs@423/manifest.json@477，trap-enum 区；摘录 plan-artifacts a01-2-diff.txt） |
| a2r_hir_worker_credential_pin | ace-host | RED | rust_lowering.rs:423 "matching pin passes" 断言失败 |
| abi_recipe_binding_exclusivity | ace-host | RED | ContractError unknown ABI leg "z" for abi/interop-v1 |
| av_source_inventory_denominators_replayable | ace-host | RED | AP-16 `examples/ui/015-notes/...app.at` scan-err——路径**仅存于 auto-ui 仓**（双仓实核），语言单仓结构性不可能绿＝跨仓清单路径设计债 |

四红均 M1 required 关联、维持阻断，按 §5.4 交 **1013**（facts 准入/来源冻结范围）；不回写已归档 1000。ace-host 为显式组（不进 `cargo t all`），不构成本包全量门阻断。

### 1013 交接输入清单（V-16 前半）

- C=eb278ca79（实现）/c601d2420（簿记），base=1169f1359，branch=codex/plan-1014-dev。
- 分组/runner/tool 最终身份：`scripts/test-groups.toml`（154 组：auto-lsp 退役、infra-pilot-ace 新增、daily/integration/auto-lang-core 族式排除）、`scripts/fixed-tests.py`（租约+守卫不变）、`scripts/test-governance/toolchains.json`（+av-regular）。冻结 SHA256 由 1013 按其合同自取（本包不预填其采证）。
- 受本包变更影响的 1013 侧旧观察/R 失效面：fixed-tests.py 钉（budget contract/registry 已随包刷新至 eb278ca79 值）；daily/integration/auto-lang-core 选择集变化（族式排除）；ace-adapters 承接表；infra-pilot-ace 新组。1013 冻结前须按其合同重采。
- A-01 四红 triage 摘录（上表）+ D-1001-R2 更新条目。

### work 收据

stage: work | plan_id: PLAN-1014 | plan_revision: 1 | outcome: pass | worktree: D:/autostack/.wt/lang-1014/auto-lang | branch: codex/plan-1014-dev | base_commit: 1169f1359 | code_commit: eb278ca79 (+docs c601d2420) | dependency_revisions: 无跨仓依赖变更（manifest 副本源=auto-ui PLAN-007 迁移件，只读核验） | task_ids: T-01..T-09（9/9） | evidence: 上表+冻结并集数字；六红逐项闭（auto-lsp 退役/auto-macros 13/13/sys 具名绿/auto-lang-core 1602/media 54/pilot 8+2）；两隐藏存量红修复（plan884 登记、substr 三 oracle 收敛）；A-01 1 绿 4 红 triage 交 1013 | blockers: 无 work 侧未决；merge 侧待办=主检出正式 `cargo t all`（T-11）、归档清理（T-12） | next_responsible_role: review（T-10 独立上下文同树）


### review 收据（T-10，2026-11-12）

stage: review | review_scope: final | reviewer_identity: 独立子代理上下文（agent_4f6be2de，本会话派发；非实现者，同树只读） | plan_id: PLAN-1014 | plan_revision: 1 | outcome: **pass** | worktree: D:/autostack/.wt/lang-1014/auto-lang | branch: codex/plan-1014-dev | reviewed_commit: eb278ca79（代码）+ c601d2420（簿记） | base_commit: 1169f1359 | dependency_revisions: 无 | spec_inputs: §5 规范增量 SD-01..06（六目标文件实核存在；SD-01 add 正确未建） | acceptance_results: AC-01..AC-08 全 pass（逐项独立复放：auto-macros 13/13、auto-media-core 54/54、sys 具名 PASS、引擎回归 2 例、infra-pilot-cli 2 + infra-fixed 60、infra-pilot-ace 8/8 且 regular/batch 工具 SHA256 前缀 60c631d9/85c45439 与收据一致、probe-lease 复跑 7/7、转移守恒 84 例/17 模块含全部 8 模块逐计数=61 例、钉链三处一致、A-01 红形状抽验一致） | pending_closeout_gates: T-11 主检出单实例正式 `cargo t all`+SD-01..06 沉淀（含 F-1014-R1-03 的 SD-06 措辞校正）、T-12 归档清理 | findings: F-1014-R1-01（LOW，engine.rs:3252 新 assert_eq! 被当前 cargo fmt 标记——style_edition 漂移，仓内操作性格式化器 rustfmt --edition 2021 下干净，5 处 master 存量同标；接受不改基线）、F-1014-R1-02（INFO，a2r-std/str.rs:97 陈旧注释与分叉契约文档不一致——存量未触，后继刷新）、F-1014-R1-03（INFO，SD-06 文案以 sys 采样为中心而实根因是 VM ARRAY_LEN——T-11 沉淀时以 VM 先读后放为主导）、F-1014-R1-04（INFO，租约窄窗竞态可留空 active 目录需手清——fail-closed 无数据风险，契约内） | evidence: 复审报告全文经子代理返回；健康检查=131 条 lib warning 全为存量（触面区域零新增）、无散落调试打印、diff 精确 16 文件+簿记、无 docs/specs 改动、无 1013/1005/1001 文件改动、无新 tag、worktree clean | next: merge（T-11/T-12）


### merge 收据（PLAN-1014:r1，进行中——已验检查点）

- prepared：reviewed_commit=eb278ca79（pass，F-1014-R1-01..04 非阻断）；canonical Spec diff=SD-01..06（SD-06 措辞按 F-1014-R1-03 以 VM 根因主导）；delivery candidate=82fc90f86（eb278ca79 的 docs-only 后代，实现/依赖零改动，delta 逐项=reviewed 冻结表）。
- landed：master `git merge --ff-only codex/plan-1014-dev` → tip=**82fc90f86**，线性 1169f1359→82fc90f86（3 commits，无 merge commit；rebase no-op——master 未动）。主检出并行文档 WIP（1015/1006 簿记+design docs，85 项）经 stash/pop 无损保留，零冲突。
- canonical_verified（82fc90f86）：docs/specs/auto-test-case/fixed-regression-baseline-and-ownership.md=088f23c34a3bf817…（add）、build-storage-lifecycle.md=72b4ad7a82c630bb…、ace-test-policy.md=9b977b100b544f59…、auto-macros/project.md=de52daac152a0ff5…、auto-media-core/project.md=f29678e8ebe83a70…、stdlib/project.md=0390cd46e282c65d…（六路径在交付提交上，hash=git show 实测）。
- T-11 正式全量门：主检出单实例 `cargo t all` 运行中（租约空闲、零竞争 cargo/nextest 进程核实后启动）。


### T-11 正式全量门收据（主检出，2026-11-12）

- 首跑（run-nzdol092）：36 调用/4524 执行/3 原条件NA；红2=plan707（负载flake，隔离复放绿——落地提交上两度隔离绿）+plan988（slow-timeout 121.3s 终止）。热缓存重跑（run-dey_vny4）：**36/4524/3NA，唯一余红=plan988 预算边界**；plan707 绿（flake 印证）。
- plan988 定性：**功能绿**——直跑串行 `cargo test -p auto-generation-core --test plan988_byte_consumers … -- --nocapture --test-threads=1`＝119.25s 1 passed 0 failed（全子例断言 PASS）。预算边界（nextest 60s/120s）为门既有配置，本包按合同"保…超时…原预算"不擅改；具名交owner＝D-1014-P988-BUDGET（KNOWN-DEBT 在途条目+本收据）。@1005 同预算下曾过线，本门总负载随 auto-lang-core 1602 例解封增大。
- 执行数 2917→4524 逐项解释：auto-lang-core 守卫修复解封 1602 例（本包 B-02 直接产出）+@1005 后各计划（1006..1013 落地窗）组内自然增长；3 原条件NA 不变（aavm/a2r-actor-tests/auto-vm）。六红全绿于本门（auto-lsp 退役、macros 13/13、sys 绿、core 1602 绿、media 54/54、pilot 拆分组绿）。零新回归归因于 1006..1014 落地窗。
- 收据：`docs/plans/.last-batch-regression.json` 更新为 covered1014@82fc90f86、verdict=known-reds-only（单一分诊红=plan988 预算边界，含直跑功能绿证据与 owner 交接）；不重写 @1005 历史收据。批量收据提交＝receipt commit。
- 1013 config/runner/tool 输入身份：C=82fc90f86；test-groups.toml/fixed-tests.py/toolchains.json 以该提交为冻结对象（1013 按其合同自取 hash 重采，见 §9.1 交接清单）。


### T-12 清理与终态收据（2026-11-12）

- archived：`docs/plans/archive/1014-fixed-regression-baseline-and-owned-execution-repair.md`（git mv 因源未跟踪改普通 move+add，提交 0d817f247；status=archived；docs 已入默认分支——非磁盘假归档）。
- cleaned：wt-guard clean（无 reparse point）→ `git worktree remove D:/autostack/.wt/lang-1014/auto-lang` → `git branch -d codex/plan-1014-dev`（was 82fc90f86）→ 组目录 `D:/autostack/.wt/lang-1014` 已删；构建中间产物随树消亡未回收；`D:/autostack/plan-artifacts/plan-1014/` 整scope删除（探针/日志/差分摘录已折叠为 §9.1/T-11 摘要）；`.retention/policy.json` 刷新（deleted-at-merge，无具名持有——plan988 预算边界为 KNOWN-DEBT 在途条目 D-1014-P988-BUDGET+批量收据记录，非文件持有）。
- 收尾路径对账：本计划命令实际输出的目录＝worktree target（随树删）、主检出 target（正常构建目录，批量收据 run-dey_vny4 在其 test-groups 下）、系统 TEMP 日志（/tmp/*.log，瞬态）、plan-artifacts scope（已删）。无第二证据目录、无其他盘符输出、无链接。主检出并行 WIP（1015/1006 簿记+design docs）原样保留未纳入本包提交。
- 落地检查（stale-artifact 观察项）：语言仓无运行时服务/发行产物须随merge重建；PATH 工具（auto.exe 等）与主检出 target 内二进制由既有 D-1001-R5 工具刷新债跟踪，本次未新增部署面。批量门已在落地提交上真实执行（covered1014）。
- 最终账：**12/12 delivered**（work9+review1+merge2）；六红全闭、七B全闭、A-01 1绿4红triage交1013、SD-01..06 沉淀于 82fc90f86。

### merge 终收据

stage: merge | plan_id: PLAN-1014 | plan_revision: r1 | outcome: **pass** | delivery_commit: 82fc90f86（ff-only 线性落地）| receipts: 批量收据 0b80224b1、归档 0d817f247 | canonical Spec paths: docs/specs/auto-test-case/fixed-regression-baseline-and-ownership.md（add, 088f23c34a3bf817）、build-storage-lifecycle.md（72b4ad7a82c630bb）、ace-test-policy.md（9b977b100b544f59）、auto-macros/project.md（de52daac152a0ff5）、auto-media-core/project.md（f29678e8ebe83a70）、stdlib/project.md（0390cd46e282c65d）| archive: docs/plans/archive/1014-*.md | cleanup: worktree/branch/group/scope 全清，registry 刷新 | pending merge-owned gates: 无（正式批量门已执行并出具 known-reds-only 收据，plan988 预算边界为具名债 D-1014-P988-BUDGET，非未决门）

## 10. 待澄清事项

无阻塞本轮规划的用户决定。T-02负责pilot最小拆分与租约恢复机制的真实前探；T-07根因未定、A-01旧五红当前状态未重放，均有明确任务/关闭oracle，不先猜实现。若发现本六红之外需扩功能/平台/跨仓改动，具名needs_replan/后继，不自动扩DEBTS范围。独立review角色在实际交接时核实，本轮不启动其他会话。
