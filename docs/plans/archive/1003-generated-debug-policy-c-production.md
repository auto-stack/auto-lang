---
plan_id: PLAN-1003
status: archived
feature_name: 生成工程调试策略C生产默认与迁移回退落地
author: [Codex]
created_at: 2026-11-10
updated_at: 2026-11-10
plan_revision: 1
current_step: 7
total_steps: 10
depends_on: [PLAN-1001, PLAN-910, PLAN-930]
supersedes_spec_components: [docs/specs/repository-boundaries/generated-cargo-build-lifecycle.md, docs/specs/auto-test-case/build-storage-lifecycle.md]
new_spec_components: [docs/specs/repository-boundaries/generated-debug-policy-production.md]
touched_goals: [GOAL-003, GOAL-016]
affects: [auto-generation-core, generated-delivery, debug-policy, repository-boundaries]
---

# PLAN-1003：生成工程调试策略C生产落地

## 0. 变更摘要

承接已归档[1001](archive/1001-incremental-cache-lifecycle-and-generated-debug-policy-validation.md)和用户“采用C”裁定，交付生成工程的正式默认策略、有效配置/身份、迁移与完整调试回退；不是再次研究A/B/C。统一受管入口默认成员`line-tables-only`、非成员依赖`debug=false`，不关闭incremental，不改变优化/断言/溢出/panic/release语义。

本轮只建立正式r1草案，0/10。没有实施、构建、GUI、安装/部署、缓存清理、worktree、材料scope、提交或tag动作。后续[OS068](../../../auto-os/docs/plans/068-windows-rq-remaining-fixes-and-full-acceptance.md)最终重建/性能冻结消费本Plan正式交付；[989](989-split-refactor-joint-delivery-and-tag-readiness.md)消费生产策略及真实收益/成本，不以1001研究数字代本包交付。

## 1. 目标

- G1：语言中立生成核心拥有一个受管C默认定义，UI/Web/Man开发入口和适用独立交付入口一致消费；用户显式配置的优先级及诊断可核。
- G2：C进入实际Cargo编译单元及发布/运行产物，重复ensure不改相同字节或误重编；配置改变不得复用陈旧发布件。
- G3：有作用域明确的完整调试恢复方式；迁移可恢复、并存容量可测、旧非增量变体清理受闲置/owner/lease保护，不自动清全局target或共享用户缓存。
- G4：002-counter、013-todo、auto-chat、auto-database四真实入口、成员行号、业务与必要direct/RQ冒烟及移根消费成立；给同源收益/构建成本和已知调试代价。

非目标：集中workspace/统一锁、全局关incremental、改sccache容量、修改语言/ACE所有workspace profile、降低测试/画质、全生态部署、修全部存量红、实现Web独立导出新渠道、重跑全RQ矩阵或ACE/AGE迁移。D-1001-R1..R6按适用性单列；只认领D-1001-R7，不因本包归档清其他债。

## 2. 架构方案

配置决策→中立受管生成入口→有效profile/来源收据→Cargo实际单元→成员发布件/hash→公开App/服务入口→独立业务与行号读回。共用生成策略不让语言核心反依赖UI/OS。

| 依赖/启动点 | 明确条件 |
|---|---|
| 1001/910/930 | 已正式归档；T-01核实际delivery/Spec，1001 C研究与采用决定不是已部署默认 |
| T-01配置决定 | 比较受管根manifest或受管config两种有限候选，核Cargo发现链、覆盖优先级、交付成员集合和真实调用点；选定单一owner后才T-02/03实现，不先假设研究scope配置可直接复制生产 |
| OS068 | 不构成本包前置；其清单/有界修复可先，最终冻结/重建/性能待本包正式交付，防止循环依赖 |
| 外部入口红 | 先以冻结当前生产基线确认源/工具是否偏斜；修本包引入回归。P1/vendor等已有渠道缺口保持具名阻塞适用腿，回实际owner正式交付，不缩四入口/移根义务 |

主实施树拟`D:/autostack/.wt/lang-1003/auto-lang`，分支`codex/plan-1003-dev`；需要改UI/Man/Web时同组各一权威树/独立提交，顺序work→独立上下文review→merge。产品只用隔离源副本/合成数据；不改用户真实安装和主检出缓存。禁止junction/symlink；本轮未建树。

## 3. 技术栈

现有Rust/toml生成、Cargo JSON单元/实际rustc调用、Python固定组/生成交付driver、Windows dbghelp行号oracle及已有UI/Web验证。不存在的生产策略接口/控制测试由本包创建，T-01冻结实际名称；不预装调试器或新常驻服务。

## 4. 需求分析与背景调查

授权：2026-11-10用户要求为“调试策略C生产落地”和“Windows完整RQ剩余修复与验收”分别立正式计划、同步989及路线，暂不实施。仅文档/独占取号；不推定研究/回收/机器操作授权，无指定模型、代理或数值资源预算。
2026-11-10 goal追加授权（/auto-plan-goal）：用户以`/goal 完成计划1003`授权全链执行——work（含建树`D:/autostack/.wt/lang-1003/auto-lang`，分支`codex/plan-1003-dev`，base=df190c1f5）、review、有界修复与merge；目标/验收/兼容承诺/范围/预算不变。材料区`D:/autostack/plan-artifacts/plan-1003/`已登记。四位发现/读取/交接已由1001所引用auto-musk ccc7629修复；当前完整分配1003，不截成100/003，执行T-01重核实际入口。

已从`docs/specs/overview.md`导航到[研究与裁定](../specs/repository-boundaries/generated-debug-policy-study.md)、[生成生命周期](../specs/repository-boundaries/generated-cargo-build-lifecycle.md)、[缓存生命周期](../specs/auto-test-case/build-storage-lifecycle.md)及生成交付规范。总体地图旧，源码/专页优先。设计见[Design53 §9](../design/53-independent-build-and-generated-cargo-governance.md#9-调试策略c生产落地target-state2026-11-10)。

| 已核来源/路径 | 事实及拟触面 |
|---|---|
| `rust_workspace.rs::try_ensure_workspace_for` | 每次ensure重写受管根内容（write_if_changed）；当前无C根profile。末尾写`.cargo/config.toml`，不能在用户工程随意注入全局祖先配置 |
| `generated_build.rs::GeneratedBuildContext::{resolve,cargo_env,write_context_file}` | context键当前基于producer+flags，未显式包含debug策略；必须证明新旧策略/有效覆盖的artifact/freshness隔离，不只看Cargo能重编 |
| `generated_build.rs`发布/收据与`rust_workspace.rs::workspace_lock_identity` | 保成员身份、共享target/栈flags、lease、发布hash/输入快照与内容新鲜度；策略加入哪层身份由T-01给实际数据流和反例 |
| UI `crates/auto-ui-tools/src/rust_ui.rs` | 普通生成消费中立根；交付根生成目前提取producer `[profile.dev]`，未因此保证package覆盖一起带出；vendored SDK crate可能是workspace成员，须按真实成员集判C，不把全部path依赖等同非成员 |
| `scripts/test-groups.toml` | 已有auto-generation-core、generated-delivery-controls、infra-fixed；UI/Man/Web保持各自owner门 |
| `test/fixtures/modularity/generated-ui-consumer/driver.py` | 真实UI生成/移根/工具闭包入口，不新造第二套导出或用单测冒真实消费者 |

规划HEAD语言`df190c1f5c28e4895f0f97ac7b346f8fa0e28237`，1001正式delivery=`762941d5b`；未提交文档不代表被测冻结组合。关键SHA256：研究Spec=`974c985eb9307837c7ceafcd2162066bc739fcf9c1491eb854f4c105724632f9`；生命周期=`10cab2b159f9899244acbddf2f7705703b2617a4a5b8260e6a56fd114d0f658b`；rust_workspace=`2a58165f169c0ceea023f16f9dd8bbcd8fc4837d41b7d7f5495691a5190165a2`；generated_build=`1782b9a812866bcae884d6f14e8b1fff2a69fbf6fc2310b55b2534de026fe890`；UI rust_ui=`89fb028c88a917ce9d75029483fdb11ac1f1d74f17c5520796c147f999b0b052`。实施时重核。

研究11.71→5.13GiB、迁移共存17.01GiB/回退16.00GiB仅作方法/容量输入，不是生产节省承诺。1001原始材料已删，不恢复旧scope；当前最终候选重新取证，保其同会话审查、未跑UI窗口/调试器等限制。

## 5. 详细设计

### 5.1 默认、覆盖与交付

1. 仅受管生成工程采用C；成员与非成员依赖以实际Cargo成员表/JSON编译单元确定，包括path/host/build依赖。成员保行号、非成员无debug、incremental仍有效。UI dev是首要作用面；四入口中Web release应保持实际有效配置/行为，不人为制造收益。
2. T-01冻结配置优先级：受管默认最低，用户显式合法覆盖保留并记录来源/有效值，未知或冲突拒绝/具名诊断，不能静默覆盖用户或给override臂签“采用C”。完整debug恢复必须覆盖成员和依赖，不能只设根debug=2而遗留package override=0。有效覆盖记录进入发布/新鲜度身份，多个并存策略不能串用旧exe。
3. 正常生成、Man公开build/run/service、安装SDK与UI独立delivery均核策略传递。交付成员集合可能变化，逐类列其有效值；不为省字节篡改vendor/member集合。保原优化、栈、linker、feature/lock、增量、release及主题资源合同。
4. 无变更ensure与暖重跑须保持受管文件bytes/mtime及收据稳定；策略/工具改变才按现有规则准确失效。失败、取消、坏配置不得覆盖last-good发布件；恢复后可真实运行，不能只改JSON标签。

### 5.2 迁移与空间账

新策略允许Cargo构建新变体；不原地覆盖活跃上下文或删除旧deps/发布件。T-04给按owner/lease/idle预览、显式执行及回退的受支持步骤，优先保留旧变体直至新件行为/行号验证成功。1001工具仅清incremental，不能借它清deps/PDB旧变体；若需整根治理只能沿现有精确根守卫在已证明闲置时做，禁止自动广扫/全局cargo clean。不为收尾强迫用户即时清旧缓存。

记录无清理共存峰、保护/暂留量、允许回收后的稳态和重建代价；无法安全清理则残留如实入总账，不能移出scope隐藏。full-debug恢复及再回C各有实际成功/失败保旧对照。目标容量/停止线由T-01按可用盘与既有研究容量冻结，用户未给数值，不臆定通过。

### 5.3 验证边界与989交接

冻结同最终源码/锁/工具/输入/工作集与缓存阶段的原默认对照和生产C，只改变debug策略；两臂使用同一canonical项目路径/成员名串行复演、各自隔离缓存，不能把member/path差额归因配置；按卷/file-ID去重allocated，生成/target/发布/残留与采样峰分账，保误差/回长/冷暖编辑耗时。四入口不可偷换空项目；UI两例实际窗口动作/状态与Web真实HTTP/数据读回，成员报错调用点行号可解析。额外调试器能力仅在实际可用时验证，不再承诺依赖帧调试；不以该已接受代价放弃成员行号。

生产C必须真实产生可归因净空间收益，保持已有行为和正式成本预算；不照抄研究−51%，不把清用户旧缓存计收益。若新组合不支持正收益或缺完整计量，保持相关AC未通过并修复/提明确修约，不把研究结论当生产PASS。本Plan只做必要direct/两Host冒烟，完整六格/全产品/性能最终门归OS068，避免重复。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/repository-boundaries/generated-cargo-build-lifecycle.md | 仅target/flags身份→经验证的C策略默认/有效覆盖、发布新鲜度及兼容边界 | 不串用陈旧产物 | AC-01/02/03 |
| SD-02 | add | docs/specs/repository-boundaries/generated-debug-policy-production.md（新） | 研究采用建议→实际生产入口、成员集合、迁移回退、字节/时间/行号/行为证据与限制 | 与1001研究分层 | AC-01～06 |
| SD-03 | modify | docs/specs/auto-test-case/build-storage-lifecycle.md | 已有增量治理→注明旧debug非增量残留与受保护迁移/显式回收边界 | 不扩大清理器权限 | AC-04/06 |

本轮不改canonical。D-1001-R7只在正式交付/来源读回后关闭；其他债依实际适用性保留。OS068/989收到完整delivery、Spec/策略ID/工具/产物hash、有效配置、复用/失效集与可重建摘要。

## 6. 测试设计

全部拟执行。每行owner_stage=work，独立review核实际oracle，不能从勾数推PASS。

| V-ID | 不变量/AC | 合法与负控 | 实际入口/可观察关闭条件 |
|---|---|---|---|
| V-01 | AC-01 默认作用域/优先级 | 默认C、显式完整debug、用户局部覆盖、未知/冲突、delivery成员分类 | common生成入口及新增既有文件内控制；Cargo JSON/编译调用证明实际debug值/成员集，release及外部workspace不受改写 |
| V-02 | AC-02/03 身份/幂等 | 同ensure、C→完整debug→C、同名旧exe/伪造策略/失败构建 | context/收据/发布读回，变配置准确失效且稳定重跑不改字节/无故重编；错误产物不能被发现为fresh，失败保旧可运行 |
| V-03 | AC-03/05 四真实入口 | 002/013窗口点击/编辑、chat/db HTTP及读回、旧工具与新工具错配 | 正式auto build/run/service入口；实际进程hash、业务状态、成员行号栈；Web release有效配置不变，不把旧PATH工具冒当前 |
| V-04 | AC-04 迁移/回退安全 | 活lease/他人target/持有拒绝、已确认闲置正控、坏配置/中断 | 受支持recipe＋现有守卫；前后源码/锁/last-good/持有hash相同；迁移/full-debug恢复再回C真实构建/运行，残留不隐去 |
| V-05 | AC-05 SDK/delivery | 正常移根locked构建、缺策略/篡改配置、开发路径被移除 | 930真实driver与UI现delivery入口；无主检出fallback，可搬动config与成员有效值成立；已有缺生产渠道不以N/A越过适用验收 |
| V-06 | AC-06 收益与成本 | 同最终源原默认/C，硬链接重数、遗漏旧变体、跨版本/缓存阶段混减反例 | 1001方法＋本包真实工作集计量，净收益超过适用误差，冷/暖编辑/迁移峰/回退成本及预算齐，失真拒绝 |
| V-07 | AC-03/05 RQ消费者 | direct、standalone-RQ、desktop-RQ各最小真实动作，错模式/旧Host | 现UI/OS公开入口，生产C工具/产物身份贯通，不以attach冒业务；只本包受影响烟测，全矩阵归OS068 |
| V-08 | AC-06 交付真实性 | 只有研究Spec/有号无delivery/头计数/已删scope反例 | 从提交/Spec/Plan摘要读回配置与实证，错源/缺行为不通过，持久摘要足够不恢复旧材料 |

永久控制优先落`crates/auto-generation-core/src/{rust_workspace,generated_build}.rs`现有测试及`scripts/tests/test_generated_delivery_controls.py`，UI侧沿`rust_ui.rs`既有生成/delivery测试；拟新控制由T-02/03实现，不冒现存用例。语言最终按真实触面跑完整`cargo t auto-generation-core generated-delivery-controls`；只有修改治理脚本才加`infra-fixed`，不机械跑全语言/ACE/AG。已有固定组来自源码静态读取，开工核`cargo t --list`。UI生成器/Man/Web真实消费沿各owner入口，T-01冻结准确命令和输出；旧基线红与新红/环境重放分列，不用父exit0遮子失败。work自查、独立上下文final review同树顺序，不自行开代理/聊天。冻结身份未变证据复用，不因merge再跑全部矩阵。

## 7. 验收标准

| AC | owner_stage | 必须真实满足 |
|---|---|---|
| AC-01 | work | T-01配置优先级/成员归属明确，默认C在实际单元成立，release与非受管workspace不回退，V-01正负控齐 |
| AC-02 | work | 策略/有效覆盖参与准确新鲜度和发布身份，同名陈旧件拒绝；幂等、切换、坏配置/失败保旧及lease隔离通过V-02 |
| AC-03 | work | 四公开入口、成员行号、正常/错配工具、必要direct/RQ行为成立；V-02/03/07不是只看构建或attach |
| AC-04 | work | 有限迁移/回退/旧变体处置真实可重复；用户/活动/持有保护，字节/共存峰/恢复成本齐，V-04通过 |
| AC-05 | work | SDK/delivery策略可搬动且按实际成员集有效，真实移根构建/行为通过；V-03/05/07齐，不静默删除适用渠道 |
| AC-06 | work | V-06/08给同源净收益、误差及正式成本预算/结果；不回算1001清理/研究收益；SD与OS068/989交接可核，缺净收益/预算仍未通过 |
| AC-07 | review | 冻结revision逐AC/V/SD独立复核，遗漏/延后/workaround及旧债适用性清楚，零未处理新增warning/fmt/debugprint |
| AC-08 | merge | 正式代码/Spec/跨仓引用落地读回，D-1001-R7据证关闭；所有输出折摘要、guard及材料清理/具名持有齐，交OS068/989；不推定用户全局安装已部署 |

## 8. 执行步骤

| 完成 | ID | owner_stage | 依赖 | 动作/文件/预期 | 验收 |
|---|---|---|---|---|---|
| [x] | T-01 | work | 后续执行授权；1001/910/930 | 冻结Spec/源码/四入口/调用图/override/成员分类/预算和容量线，最多两种配置落点探针，选单一owner；下游据有效决定实施 | V-01/06准备；AC-01/06 |
|      |     |      | T-01 冻结记录（2026-11-10，goal授权）：**基线**五源SHA256全部复验一致（rust_ui.rs 勘误=auto-ui仓`crates/auto-ui-tools/src/`，master=8d289105；auto-lang base=df190c1f5）。**探针**（plan-artifacts/plan-1003/t01/，cargo 1.98.0/rustc 1.98.0，sccache在位）：①受管根manifest（虚拟根`[profile.dev] debug="line-tables-only"`+`[profile.dev.package."*"] debug=false`）→成员单元`-C debuginfo=line-tables-only`、非成员path依赖无debuginfo且`-C strip=debuginfo`、两单元`-C incremental=`保留、零警告；②cwd祖先`.cargo/config.toml`携带`[profile]`同样生效（=1001研究机制可解释）。**决定=受管根manifest**（单一owner=`try_ensure_workspace_for`）：cargo config发现锚cwd，受管spawn cwd=ws生效但工程目录`--manifest-path`手工运行发现不到ws config（F-910-R1-01/905同源教训）；manifest落点cwd无关、受管/手工一致。**调用图**：UI=auto-ui-tools`rust_ui::resolve_managed_front`→core(Ui)；Web=auto-web facade→core(Service)×4处；Man=http_service→auto_web facade；交付根=`generate_delivery_root_manifest`（现提取producer`[profile.dev]`段、`package."*"`未带出=修正点）；发现=`auto-ui-compiler::app_registry:249`→`discover_front_exe`（需策略感知）。**override接口**：env`AUTO_GENERATED_DEBUG_POLICY`∈{缺省=c受管默认,`c`/`default`,`full`}，未知值具名错误且失败在改写前；cargo原生`CARGO_PROFILE_DEV_DEBUG`=用户局部覆盖——policy缺省时保留+记录（标签`c+user:...`+身份隔离），显式`c`时冲突拒绝；`full`=不写任何`[profile.dev]`段（纯cargo默认dev：成员+依赖debug=2、host-only延迟关闭、增量开；`"*"`无env覆盖形式→必须由生成器省段）。段落定义归core单一来源，dev ws根与delivery根一致消费，release永不写。**身份层**：默认c不进context_id（公式不变→Web release缓存/既有根保持、dev单元cargo指纹原地迁移）；非默认有效策略（full/用户局部覆盖）作第4键进context_id→独立根+lease+并存隔离；context.json加性增`debug_policy`(+source)（schema串v1不变，旧根无字段=legacy）；`FrontReceipt`增`debug_policy`，freshness与`discover_front_exe`增策略(+exe哈希)校验。**成员分类**：dev ws成员=生成应用(line-tables)、producer/三方=非成员(false)；delivery根成员=app+28 vendored crates(line-tables)、registry依赖(false)——按实际成员集。**预算/容量线**（规则非比例）：迁移实测前free(D:)≥预测共存峰+30G；free(D:)<30G停止报告不自动清；本机D: avail=264G；计量沿1001口径（分配字节去重、卷/file-ID、生成/target/发布/残留分账）。 |  |
| [x] | T-02 | work | T-01 | common rust_workspace/generated_build实现受管C策略、身份/新鲜度/完整debug恢复与负控，不触用户全局config | V-01/02；AC-01/02 |
|      |      |      | T-02 证据（commit 0c79e3300）：`generated_build.rs` 增 `DEBUG_POLICY_ENV`/`resolve_debug_policy`/`DebugPolicySelection`/`managed_profile_toml`（C=两表、full=仅marker）、context第4键分层（默认c哈希输入与旧三段逐字节一致——测试复算旧公式对照）、context.json加性`debug_policy(+source)`、`FrontReceipt.debug_policy`+`debug_policy_matches`+`discover_front_exe`策略门控与exe哈希校验；`rust_workspace.rs` ensure在任何写入前解析策略（未知/冲突具名失败、旧manifest字节保持）、profile块入受管根manifest、re-ensure字节/mtime稳定。控制测试：`debug_policy_resolution_matrix`（含unknown/conflict负控、partial值键隔离）、`context_debug_policy_identity_layers`、`workspace_debug_policy_default_full_and_negative_controls`（子进程探针default/full/partial/unknown/conflict×幂等×ctx根隔离）、`front_receipt_policy_gates_freshness_and_discovery`（legacy缺字段/异策略/换名同exe负控+匹配正控）。`cargo test -p auto-generation-core --lib` 36/36绿；auto-api-gen `cargo check`过（warning为auto-lang-core预存，非本计划引入）；web-shared-generation-probe fixture断言为deps键解析，不受增段影响。未触用户全局config；`.cargo/config.toml`仍仅target-dir+栈flag。真实Cargo单元/JSON级V-01/02证据由T-05/T-06冻结组合补齐。 |  |
| [x] | T-03 | work | T-02 | UI rust_ui delivery、Web/Man实际调用链接同一策略，包内配置与SDK成员集合正确，旧flags/源owner保留 | V-01/03/05；AC-01/03/05 |
|      |      |      | T-03 证据（auto-ui 树 commit 17220173，组=`D:/autostack/.wt/lang-1003/{auto-lang@0c79e3300, auto-ui@17220173, auto-web@2d69841(detached), auto-down@58ea33f(detached)}`，组内 path 解析即开即用）：①交付根 manifest 改消费核心单一策略定义（`managed_profile_toml`，C两表随包/成员=app+28 vendored=line-tables、registry依赖=false、块静态可搬动；full=仅marker；不再抄取producer根`[profile.dev]`——旧抄取丢`package."*"`段）；策略在任何staging写入前解析（未知/冲突具名失败）；delivery-manifest.json加性`debug_policy{label,source}`。②FrontReceipt接线：写入`context.debug_policy`；`front_needs_regeneration`增`debug_policy_matches`门（legacy无字段/异策略永不fresh）。③**认领D-942-P1受限修复**（§10.3预见）：`DELIVERY_VENDOR_TREES`补`experimental/ace/crates/semantics-contract`同形镜像（PLAN-944引入的非optional叶子依赖；预存生产回归=原根/移根locked 101红），closure单测复绿；完整关闭条件（930 driver V01/V02+导出链）由T-05实测绑定。④Web侧=组内auto-web钉master作为auto-ui-tools依赖实际编译（对新core过编译=接线验证）；Man经auto_web facade零fork（T-05真实入口验证）。⑤定性：rust_ui::tests串行61绿+closure复绿；`test_shell_pack_lib_freshness`与auto-ui-compiler两codegen fixture红=基线存量（主检出示同红）；警告6条=基线同数零新增。 |  |
| [x] | T-04 | work | T-02/03 | 实测scope旧→C→完整debug→C，给lease/失败/空间峰与受限残留回收recipe，不在主检出清缓存 | V-02/04；AC-02/04 |
|      |      |      | T-04 证据（2026-11-10～11，scope=`plan-artifacts/plan-1003/t04`，工具=组内重建 auto@e0c70c74/auto-ui-tool@e38325c9，sccache经RUSTC_WRAPPER=""禁用，离线ctx8复算双双命中：full=83ce3fe3/默认c=f2353a92）：**A臂(原默认=full策略+显式钉默认根)** 冷编595单元/3m05s/闭包10.28GiB（分配字节去重，ws根manifest无profile段）；**迁移A→C(默认策略原地)** 595单元重建/2m08s/共存峰**14.12GiB**（旧A rlib/pdb残留），manifest翻转C两表，receipt/context.json→`f2353a92`+policy`c`，发布件哈希A≠C（a78bc1fc→c33909e9）；**实弹单元flag证据**（touch producer源+受管env手动`cargo build -v`）：成员`counter_9679bf44`=`-C debuginfo=line-tables-only`、5个producer path依赖=`-C strip=debuginfo`(debug=false)、全部`inc`(增量保留)，暖touch传播仅7单元（无故重编=0）；**C→full回退** 隔离新根83ce3fe3冷编595/179s/10.28GiB（与A逐字节同构），默认根与C发布件不动；**full→C回切** 2.5s/0重编（双根双暖=即时回退）；**幂等正控** 重跑0重编、receipt/ws-manifest字节稳定；**负控**：bad value与显式c+CARGO_PROFILE_DEV_DEBUG=2经真实入口具名失败且三受管文件哈希不变；**真中断**（winpid树杀@458/595单元处）last-good发布件/receipt/manifest全保，partial根留死pid lease（stale语义）；**恢复构建**0重编；**用户局部覆盖经真实入口**：CARGO_PROFILE_DEV_DEBUG=2→新隔离根1c97f098+receipt标签`c+user:...`（附带验证）。**治理守卫**：活lease→`live build/run lease holds this root`拒绝零删除；闲置preview报reclaimable 15.97GB/21174文件；apply在**PLAN-1000会话ace测试编译在场**（4×cargo+nextest，无显式target不可证foreign）被正确推迟（维护窗口语义）。**挂起腿闭合（2026-11-11）**：轮询等待他会话编译波间空档（未杀任何他会话进程），第2次轮询命中静默，守卫apply自行复核后通过——`{"removed": …generated-consumers\f2353a92, "unique_bytes_removed_from_scope": 20226435219, "file_data_bytes_reclaimed": 19993974931}`（整根≈18.84 GiB，含A残留+C+Web release）；删根后成员`bin/debug`发布件完好（sha256仍=收据）；回收后默认策略重建 counter+todo 596单元/126.4s，稳态去重闭包 5,512,992,433 B=**5.134 GiB**（残留清零，与T-06干净C臂5.08–5.13一致），收据ctx f2353a92/policy c。说明：T-04前段各臂用的是alias修复前的工具（auto@e0c70c74/auto-ui-tool@e38325c9），alias修复不改变cargo单元；最终工具下的迁移/回退数字以T-06为准。 |  |
| [x] | T-05 | work | T-03/04 | 四真实入口/成员行号/SDK移根及两Host必要烟测；源码/进程/真实读回，保旧红和环境缺口 | V-03/05/07；AC-03/05 |
|      |      |      | T-05 进行中证据（2026-11-11；组=auto-lang@ed9b596df/auto-ui@a63ab8bb/auto-web@2d69841/auto-man@347c29f/auto-down@58ea33f；release工具由组内重建；材料`plan-artifacts/plan-1003/t05`）。**UI两入口（真实窗口进程，MCP经`AUTOUI_MCP_PORT`）**：002-counter 按键`+,+,-`→state `count 0→1`；013-todo（front+back成员，merged in-process）输入→state `input:"plan1003 smoke item"`、条目checkbox toggle→`store.active_count 3→4`；两例运行进程映像=成员`bin/debug`发布件且sha256=收据`exe_sha256`、收据policy=`c`。限制：MCP截图两例均超时（`iced thread may not be responding`），窗口腿证据=状态读回非截图；013-todo的`onenter`提交经MCP键盘Enter未触发，未作为证据。**Web两入口（git archive隔离源副本 chat@5cdca0f/database@dee81e0，`auto service --server rust`，release）**：chat full+钉默认根冷编437单元/就绪≈250s，`/__auto/health/live`→200 `{"state":"ready"}`、`/api/contacts`→200 `[]`；切默认C→**0重编**/≈5s就绪/行为同；database 同根仅1单元/≈20s，`/api/db/columns`→405，切默认C→0重编/405保持；ws根manifest含C两表而release零重编=Web有效配置不变。**成员行号（dbghelp oracle，t05/line-oracle）——发现并修复真实缺口**：发布exe的RSDS记录pdb名=`counter_9679bf44.pdb`（相对名），PLAN-910发布流程改名为`counter.pdb`→dbghelp exe目录回退找不到，**原样发布对仅解析163个静态库导出符号、0成员符号**（策略C“保留成员行号”对用户实际运行的发布件原不成立；既有行为副作用，非本包引入）。修复=core `publish_artifact`另以cargo报告名硬链接pdb别名（commit b73ac5011，含“删后重建=新inode”判别性刷新测试）；修后**未改名原样发布对**解析：成员符号12866/12866带行号，其中成员自有源28个（`counter_9679bf44::App::new -> …\002-counter-9679bf44\src\main.rs:300`等），依赖`auto_ui_runtime` 18609/18687无行号（裁定已知代价）。**错配工具负控**：剥除收据`debug_policy`字段（旧工具产物形态）→真实入口日志走`Regenerating Rust UI code`分支并重写收据（无变更幂等对照该行=0）。**792/930 delivery driver（V-05）**：首轮DRIVER_EXIT=1——`V01-fresh-resolve`已PASS（D-942-P1的metadata 101处）但`V01-real-build`/`V06-theme-relocated-locked-build`编译红=**D-942-P1隐藏第二层**：auto-lang-core自PLAN-943(2026-11-05)起`include_str!`越界引用`stdlib/contracts/{operations,compatibility}.toml`，交付包未带（此前被101挡住）；全28交付crate字面量`include_*!`越界目标扫描（t05/sweep_root_inputs.py）未覆盖项恰此两项，补入`DELIVERY_ROOT_INPUTS`（auto-ui a63ab8bb）。**二轮（冻结于上述提交）DRIVER_EXIT=0，43项全PASS**：V01 fresh resolve+`cargo build --locked`(131s)、V02 A→B移根locked(131s，本地target无共享回退，越界path=无)、V03a–j负控、V04 dev保持/模板逐字节同源、V05a–h随包工具、V06主题/真实运行/profile矩阵；jet/ark两格为driver自定义环境门控（本机无gradle/hvigor），非本包。**移根包B策略可搬动（无开发根env）**：根manifest携C两表、`delivery-manifest.json`记`debug_policy{label:c,source:managed-default}`；B内强制重编`cargo build -v --locked`实际flag：app`uiminimal`与vendored成员(auto_val/auto_lang_core/auto_ui_runtime)=`line-tables-only`+增量，registry`blake3`与exclude的vendor树`ace_semantics_contract`=`strip=debuginfo`（按实际成员集，区别于dev ws中producer为非成员）。**测试卫生如实记**：b73ac5011提交命令以`grep … &&`串接，在一次35过1败的运行后仍提交；该红在并行构建负载下复现1/15，失败者=既有`shared_generation::…reader_dynamic_json_nested_return_and_list_reuse_are_admitted`（temp fixture生产者发现命中他fixture缺`stdlib/auto`），与本包无涉，其余40+次全绿；另将本包策略矩阵测试改纯函数注入（ed9b596df）消除进程内非法env值泄漏隐患。**缺策略对照（oracle判别力）**：B根manifest去掉两表后重编543单元，成员与registry依赖全为`debuginfo=2`（另52为cargo默认延迟关闭的host依赖）；manifest已还原。**V-07 两Host烟测（发布件`counter.exe` sha256=4b3c58ba=收据，policy c，ctx f2353a92）**：①direct=上述MCP腿。②**standalone-RQ**：组内`auto rqhost --pipe plan1003-rqhost`（私有管道，不碰默认`autodesk-rqhost`）+`auto run -r rust -q`，客户端参数`--autodesk-render=queue --autodesk-incubate --autodesk-broker=plan1003-rqhost`；rust编译轨`-q`走`run_native_client`无MCP共位（仅VM轨有），故动作=向rqhost原生窗口投递鼠标消息+窗口截图读回：`+`三击→窗口`Counter: 0→3`、host修订1→4；空白区点击对照修订4→4。③**desktop-RQ**：组内补auto-os@253deb9（detached）构建`ui-desktop`（链接组内核心），隔离=`AUTO_OS_ROOT`指向无`apps.manifest`目录（注册表2项，不回退主检出）、独立`AUTO_VM_STORAGE_FILE`、`AUTO_DESKTOP_RQHOST=1`私有端点、`AUTOUI_ACCEPTANCE=1`验收通道；`autoui_desktop{bus:"launch\u001f002-counter"}`→host日志`launch_app(outproc-native) 002-counter <- …\ws\002-counter-9679bf44\bin\debug\counter.exe`、adopt OK、子进程映像sha256=收据；`handler Inc`×3→桌面内窗口`Counter: 3`、合成帧1→4。**负控/对照**：(a)同一桌面`AUTO_GENERATED_DEBUG_POLICY=full`→`launch_app(inproc) 002-counter`且无原生进程（c收据发布件不被复用=策略身份贯通桌面产物发现）；(b)旧Host：PATH旧`auto`(0.1.0+plan-890…g03bb65e, sha 54124fd4)作rqhost接入新客户端→接纳、两击`Counter: 2`（协议兼容，无拒绝）；(c)`-q`且管道无host→客户端自动孵化rqhost并出首帧（设计行为，非失败模式，不计作负控）；(d)桌面窗口对OS级投递点击无响应、验收指针对shell卡片`hit=None`（设计：live-render像素身份守卫，须走验收通道）。**过程偏差如实记**：首次桌面启动未隔离，注册表并入主检出`apps.manifest`的30项并尝试起017-chat服务，且因PATH旧`auto`与本scope ws混树出现`auto-web-contract` native links冲突后退出（本人隔离env造成的混合，非产品缺陷）；一次清理按命令行子串匹配Stop-Process误杀了本次调用自身shell，已改为按进程名+路径精确清理，事后核对无残留进程。**限制**：MCP截图在direct腿均超时；桌面RQ下counter第三个主题按钮渲染为灰块（与本包无关的保真观察，未追）；desktop腿只验证002-counter。 |  |
| [x] | T-06 | work | T-05 | 同最终输入原默认/C计量，核净收益/成本与误差；不达则修复或明确提修约不假绿 | V-06；AC-06 |
|      |      |      | T-06 预声明（2026-11-11，计量前写入；用户未给数值预算，下列为work阶段工程判定线，非用户批准阈值，独立review可否决）：工作集=002-counter+013-todo（两UI dev入口，同canonical项目路径/成员名），两臂各自隔离ws与显式target根（`plan-artifacts/plan-1003/t06/{ws,root}-{A,C,C2}`），串行、`RUSTC_WRAPPER=""`、同一冻结工具/源码/锁，只变策略（A=`full`即cargo默认dev，C=缺省）。口径=分配字节按卷file-ID去重（t04/measure.py），target根与生成ws分账。**判定线**：①净空间收益 `A−C > max(1 GiB, 10×E)`，E=重复冷臂C2与C的闭包差；②冷编 `C ≤ 1.05×A`；③编辑重编 `C ≤ 1.10×A`；④暖无变更重跑两臂0重编；⑤容量线沿T-01（计量后free(D:)≥30G）。任一不达=AC-06不通过，如实记录不改线。已知噪声源：他会话编译并行在场（不可控），时间项按单次记录不做统计声明。 |  |
|      |      |      | T-06 结果（2026-11-11；工具 auto/auto-ui-tool=组内 auto-lang@ed9b596df、auto-ui@a63ab8bb 重建的release；材料`plan-artifacts/plan-1003/t06`）。**A臂（full=cargo默认dev）**：冷编 counter 595单元/148.3s + todo 1单元/19.9s；target根 12,319,413,557 B（deps 5.690/incremental 3.980/顶层pdb 1.467 GiB）；编辑重编 10.0s/1单元、还原 9.5s；暖无变更 0重编/1.3s；编辑后去重闭包 12,558,564,650 B=11.696 GiB。**C臂（缺省）**：冷编 595/119.9s + 1/8.7s；冷态去重闭包 5,451,405,684 B=**5.077 GiB**（根 4.928：deps 2.776/incremental 1.659/顶层pdb 0.187；ws独占 0.149）；编辑 5.1s、还原 5.1s；暖 0重编/1.4s；编辑后闭包 5.127 GiB；收据 ctx f2353a92/policy c。**C2重复冷臂**：595/118.8s + 1/8.5s，闭包 5,451,408,773 B → **E=3,089 B**。**口径纠偏**：ws内发布pdb是target根pdb的硬链接，root与ws分别计量再相加会重复计数（A 13.105 vs 去重≈11.638 GiB；C 5.264 vs 5.077），该朴素差值 7.84 GiB 高估收益 1.28 GiB，弃用；A冷态闭包为推导值（冷态根实测 + 编辑后实测的ws独占 177,096,631 B，ws在两时点仅差 0.7 MB），同相位实测对照用编辑后态 11.696→5.127 GiB。**判定线核对**：①冷态 A−C≈7.045e9 B=6.56 GiB（−56%；编辑后态实测 6.569 GiB），> max(1 GiB,10×E) ✓；②冷编 128.6s/168.2s=0.76 ✓；③编辑 5.1/10.0=0.51 ✓；④两臂暖 0重编 ✓；⑤计量后 free(D:)=78G ✓。**迁移/回退成本（最终工具，在root-A上原地 A→C→full→C，带3s采样）**：迁移重建 596单元/124.2s，target根采样瞬态峰 16.018 GiB，稳态共存闭包 14.975 GiB（=旧变体残留≈9.85 GiB，未回收前不计收益）；同根原地回退full 28.4s/2单元；再回C 10.6s/2单元。**设计代价如实记**：产品缺省下`full`走隔离新根，首次回退=冷编（T-04实测 179s、约 10.3–11.5 GiB 新根）而非上述原地 28s——以隔离换取策略不混用与整根可回收。**行号对照（原样发布对）**：A/C成员自有源均解析`…\src\main.rs:300`（28/28）；依赖`auto_ui_runtime` A臂 9230符号仅454无行号、pdb 751 MiB，C臂 18609/18687无行号、pdb 95 MiB。**失真反例**：把原地迁移后的根当作“C”会得到 14.98>11.64 的负收益（遗漏旧变体）；冷态与编辑后态混减、root+ws朴素相加均已排除。**限制**：单次计量、他会话编译并行在场（时间项有噪声，不作统计声明）；A冷态闭包含一项推导；未计Web（release不受影响，T-05已证0重编）；净收益不含任何清理用户旧缓存所得。 |  |
| [x] | T-07 | work | T-06 | 冻结受影响完整组及真实消费者并集，逐AC/SD/债务核验，准备OS068/989交付和复用/失效集 | V-08；AC-01～06 |
|      |      |      | T-07 证据（2026-11-11）。**冻结身份**：auto-lang `codex/plan-1003-dev`@6a1443627（0c79e3300 策略核心 → b73ac5011 pdb别名 → ed9b596df 测试纯函数化 → ca0f1521a 规范增量 → 6a1443627 rustfmt），auto-ui `codex/plan-1003-dev`@0c873457（17220173 → a63ab8bb → 0c873457 rustfmt），两树工作区干净；组内只读兄弟 auto-web@2d69841、auto-man@347c29f、auto-down@58ea33f、auto-os@253deb9（均detached，零改动）。**固定组（最终HEAD）**：`cargo t auto-generation-core generated-delivery-controls` gate_passed=true/115s——auto-generation-core executed 37/selected 37/excluded 0（含本包4个控制用例），generated-delivery-controls executed 15。未改治理脚本故不加infra-fixed。**UI owner侧**：auto-ui@0c873457 `rust_ui::tests` 串行 62过/1红/1ignored，红=`test_shell_pack_lib_freshness`（基线存量，主检出同红）。**卫生**：auto-generation-core两文件rustfmt零差异（基线即干净）；rust_ui.rs fmt差异=基线80处零新增；rust_ui.rs警告6条=基线同数；产品代码无新增调试打印（子进程探针`PLAN1003_CTX=`为测试协议输出）。**证据复用说明**：T-05/T-06的工具二进制与driver二轮基于ed9b596df/a63ab8bb；其后仅规范文档与rustfmt提交（无语义变更），固定组与目标测试已在最终HEAD重跑，真实入口证据按“身份未变”复用，独立review可要求重放。**AC自核**：AC-01 默认C在真实单元成立（dev ws、移根包两类成员集各有flag证据）、release不写、未知/冲突/局部覆盖矩阵齐；AC-02 策略进收据/发现身份、非默认进ctx根、幂等/切换/坏配置/真中断保旧/lease拒绝；AC-03 四入口+成员行号（含发布件pdb别名修复）+direct/standalone-RQ/desktop-RQ真实动作与负控；AC-04 迁移/回退/整根回收正控/共存峰/回收后稳态；AC-05 driver 43项全绿+移根包有效值+缺策略对照；AC-06 预声明判定线全达、失真反例、SD三项已在树。**规范增量（已提交于worktree，待merge落主）**：SD-01 modify generated-cargo-build-lifecycle.md；SD-02 add generated-debug-policy-production.md；SD-03 modify build-storage-lifecycle.md。frontmatter spec元数据与此一致。**债务处置建议（review/merge裁定）**：D-1001-R7 可据本包交付关闭；D-942-P1 关闭条件（vendor该crate、driver V01/V02与导出链复绿）已满足且含隐藏第二层修复；D-942-P2（Web无交付导出入口）保持开放=本包非目标。**新观察到的候选项（均非本包引入，未修）**：①`shared_generation::…reader_dynamic_json_nested_return_and_list_reuse_are_admitted` 在并行负载下偶发红（1/15，temp fixture互扰）；②auto-ui `test_shell_pack_lib_freshness` 入库物过期；③auto-ui-compiler 两个codegen fixture红；④rust直连臂MCP截图超时、013-todo `onenter` 经MCP键盘Enter未触发；⑤桌面RQ下counter第三主题按钮渲染为灰块；⑥rust编译轨`-q`无MCP共位（VM轨有）。**未做/限制**：未向`scripts/tests/test_generated_delivery_controls.py`增控制（永久控制落在两crate的Rust测试）；安装SDK producer形态的交付未单独实测（driver用开发producer）；013-todo未进desktop腿；计量为单次。**OS068/989交接**：消费上述两提交与SD-02页；策略ID=`c`(managed-default)，覆盖接口`AUTO_GENERATED_DEBUG_POLICY`；默认策略ctx8不变→既有release缓存可复用，dev单元首次构建原地重编且旧变体残留须整根守卫回收；`full`/局部覆盖走独立根；发布目录新增`<bin>_<id8>.pdb`别名；收据新增`debug_policy`（旧收据一次性判stale重建）；全RQ六格/性能最终门仍归OS068，本包只做了受影响烟测。 |  |
| [x] | T-08 | review | T-01～07无必需缺口 | 同一权威树独立上下文final review，绑定revision/源码/依赖和SD；不因采纳决定豁免生产证据 | AC-07 |
| [x] | T-09 | merge | T-08 pass | 各owner代码与Spec正式落主、身份读回；更新R7和下游引用，不创建tag/推送用户发布 | AC-08落地 |
| [ ] | T-10 | merge | T-09 | 折命令/关键数/哈希入本Plan，逐全部实际输出/持有/活动进程对账，guard clean后移树/材料，最终归档 | AC-08收尾 |

唯一材料拟`D:/autostack/plan-artifacts/plan-1003/`，按仓/用途分，开工登记。源码副本/target/SDK/安装/日志/截图都在此或本worktree正规target，禁止散落输出；merge折必要摘要入本文件后删scope，未知/用户持有不动。正式包/清单若仍待接管，按具名owner/理由/关闭条件持有，不盲删。未来执行T-01，当前全部未执行。

## 9. 复审记录

2026-11-10 / stage:new / PLAN-1003 / r1 / outcome:blocked（仅规划；执行授权/配置落点与预算在T-01收敛）/ drafting0/10。D-1001-R7已认领但未关闭。源码/Spec/重叠/作用域自检不是独立review；后续先定向设计复核，再由用户启动实施。无运行PASS、无新agent/聊天。

规划静态自检：11章，T/AC/V/SD=10/8/8/3，全未执行；当前链接/表格/围栏/空白检查及正式依赖闭包87份无环通过，语义SHA256=`3c03f5e2747cd5c3f31aeb8c1c6235a7f4d4834d02dc6647fa65f40dcb439bdd`（UTF-8/LF，§0～8与§10，排除§9）。仅合同结构/来源自检，非独立设计复审或运行PASS。

## 9 附：work 交接记录（goal 链）

`stage: work | PLAN-1003 | r1 | outcome: pass | worktree: D:/autostack/.wt/lang-1003/auto-lang（branch codex/plan-1003-dev @6a1443627）+ D:/autostack/.wt/lang-1003/auto-ui（codex/plan-1003-dev @0c873457）；只读兄弟 auto-web@2d69841/auto-man@347c29f/auto-down@58ea33f/auto-os@253deb9（detached） | base: auto-lang df190c1f5 / auto-ui 8d289105 | 代码提交：lang 0c79e3300→b73ac5011→ed9b596df→ca0f1521a→6a1443627；ui 17220173→a63ab8bb→0c873457 | 任务：T-01..T-07 完成（7/10，余 T-08/09/10 为 review/merge） | 证据：本节各行+§8行内；固定组 gate_passed（37+15）；792 driver 二轮 43/43；T-06 预声明判定线全达 | 阻塞：无（T-04 闲置回收正控已于编译波间空档完成）`

**越界与副作用披露（2026-11-11，收尾路径对账登记）**：
1. **主检出缓存被写入**：792/930 driver V06 腿（随包工具、无 producer 钉定、scratch 位于 `D:/autostack` 下→上溯兄弟发现命中主检出）于 01:23–01:25 向 `D:/autostack/auto-lang/target/generated-consumers/f3f98a9e` 新增 **4.642 GiB/9,677 文件**（C 变体 dev 单元+v06 成员 uplift 件；该根既有 5.093 GiB 文件全部未动，总量至 ~9.7 GiB），`context.json` 被 rewrite 为含 `debug_policy:c`（加性字段）。成因=driver 既有环境发现行为叠加本包新默认；与计划§2"不改主检出缓存"承诺相悖，如实记录。处置：**不自动清理**（整根属用户；删除须走守卫 preview/apply 且会连既有 5.09 GiB 一并回收=用户下次构建冷编）；建议用户在维护窗口用 `python -I scripts/test-governance/build_storage.py <该根>` 预览后决定。同类复跑规避法：driver `--scratch` 置于 `D:/autostack` 之外，或为 V06 腿补 producer 钉定（回 OS068/driver owner 决定，不属本包改动）。
2. **auto-ui 主检出 target 被写入**：两次基线对照 `cargo test -p auto-ui-tools/--lib`（于 `/d/autostack/auto-ui`）产生的常规构建产物（非破坏）。
3. 会话过程偏差均已行内记录：grep串联导致一次红测试后提交（已定性为既有用例负载flake并加固）；一次按命令行子串的 Stop-Process 误杀自身shell；桌面首启未隔离并入主检出 apps.manifest（重试已隔离）。

`stage: goal | PLAN-1003 | r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: 同上 | commit: 6a1443627/0c873457 | next: review`

**复审 R1（2026-11-11）**
`stage: review | review_scope: final | reviewer: 实施会话本身（同模型同上下文）——**独立性不成立**：计划§6要求独立上下文且不自行开代理/聊天；用户于goal续行指示本会话继续复审，按约如实记为限制；结论由产物与复审者自选反例重建，不引用执行摘要 | PLAN-1003 | r1 | outcome: **needs_fix** | worktree: D:/autostack/.wt/lang-1003/{auto-lang@6a14436273e9cf33bda09a3e6af28c5d6157846b, auto-ui@0c873457154be50fcbe75e16e1bd9ccc9fbdbd79}（codex/plan-1003-dev，两树干净）| base: df190c1f5 / 8d289105 | 依赖: auto-web@2d69841 auto-man@347c29f(仅Cargo.lock构建漂移，不属交付) auto-down@58ea33f auto-os@253deb9 | spec_inputs: SD-01/02/03 @ca0f1521a | next: work`

复审自选反例与结果（材料 `plan-artifacts/plan-1003/review/`，探针=链接组内核心的 `discover-probe`，调用真实 `discover_front_exe`）：
- **F-1003-R1-01（P1，AC-02/V-02“伪造策略”，原需求缺口）**：`discover_front_exe` 只比对策略标签与exe哈希，**不比对 `context_id`**。复现：`full` 策略真实构建 counter（收据 ctx `83ce3fe3`、标签 `full`）→(a) policy=full 发现=Some（正控）；(b) 默认策略、原收据=None（正控）；(c) **仅把收据标签改为 `c`**、默认策略（当前 ctx `f2353a92`）→ **Some(…counter.exe)**——`full` 构建件被当作 `c` 可用件返回。影响面：auto-ui `app_registry::published_native_exe`→桌面原生 launch 判定。work 自核与已提交规范文本（SD-01§4、SD-02§3“伪造标签均不发现”）及 `debug_policy_matches` 文档注释与实现不符；work 单测未覆盖“异ctx+同标签”组合。兄弟路径 `front_needs_regeneration` 已同时比对 ctx 与标签（不受影响）。关闭oracle：永久测试——非默认策略上下文的收据仅改标签→发现None，合法对照仍Some；修后重放本条(a)(b)(c)与T-05桌面正/负控。
- **F-1003-R1-02（P2，AC-06“保持已有行为与成本预算”，新增成本未入账）**：发现路径由“读收据+is_file”变为“完整上下文解析+全文件sha256”，实测 80.9 ms/次（79.75 MB exe，20次均值）；运行期调用点=注册表扫描 `entry_for_dir`（每个带发布件的应用每次扫描一次）。work 未计量未披露。另：发现现依赖 producer 可解析（解析失败→None→静默降级inproc）——**假设级**，本机未能构造失败。关闭条件：在≥10个带发布件应用的真实注册表扫描上给出增量延迟并入账/写入规范，或改为等效但廉价的校验且保持本包全部负控。
- **F-1003-R1-03（P3，AC-05/V-08 记录准确性）**：带 `CARGO_PROFILE_DEV_DEBUG=2` 生成真实交付包（fixture proj）→ `delivery-manifest.json` 记 `label: c+user:CARGO_PROFILE_DEV_DEBUG`，但交付根内容为纯C两表且消费侧不读环境——标签不描述包内容。关闭：清单标签只反映随包静态策略（或另列生成期环境），补断言。
- **F-1003-R1-04（P2，流程/收尾；计划§2“不改主检出缓存”）**：work 已披露的主检出根 `f3f98a9e` 被写入 4.642 GiB。无法由代码修复；须由 merge 收尾路径对账承载，并保留“用户决定是否回收”的具名持有；driver V06 腿的 producer 钉定/默认 scratch 位置属 driver owner（候选债）。不阻断代码复审，但 merge 不得标为已清理。

复核通过的独立检查：默认 ctx8 与旧公式一致——除单测复算外，主检出既有 legacy 根 `f3f98a9e` 被新工具默认策略解析为同一 id（真实连续性）；非默认策略根（`full` 83ce3fe3、局部覆盖 1c97f098/c7867221）均被守卫 preview 准入（SD-03 成立）；UI拆分后端与前端同走 `cargo_managed_build`（dev、每次过cargo，无陈旧复用）；Web 服务四份日志均 `Finished release`；无消费者枚举成员bin目录（pdb别名安全）；producer 根 `[profile.dev]` 仅 debug 键（交付块替换内容等价）；auto-generation-core 在 HEAD 零警告；固定组结果文件核对=37/37+15、gate_passed；GOAL-003/016 真实存在，新规范页无需人工索引登记。

验收映射：AC-01 pass；AC-02 **fail**（R1-01）；AC-03 pass（限制：直连臂截图超时；桌面腿在R1-01修后需重放）；AC-04 pass；AC-05 **partial**（R1-03）；AC-06 **partial**（R1-02 成本未入账；规范文本一处与实现不符）；AC-07 本轮=needs_fix。重开：T-02（发现身份）、T-03（交付标签）、T-07（冻结/规范文本/受影响重放）；T-01/04/05/06 保留（T-05 的桌面正负控与发现相关证据列为修后受影响重放，不整体重跑）。merge-owned 待办不变：最终冻结并集、SD落主、D-1001-R7/D-942-P1关闭、R1-04路径对账。

`stage: goal | PLAN-1003 | r1 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: 同上 | commit: 6a1443627/0c873457 | next: work（修复循环 1/3）`

**修复循环 1（2026-11-11，复审R1 needs_fix → work）**
`stage: work | PLAN-1003 | r1 | outcome: pass | worktree: 同一权威树（lang 11e1f1fd5 / ui 0d37a997，均干净）| 本轮提交：lang 1d3549c8d(F-R1-01 发现增收ctx一致性) → 348835611(F-R1-02 发现拆分身份/字节) → df82d5ecc(规范文本更正) → 11e1f1fd5(选择序与校验分离 context_identity)；ui 2d937091(F-R1-03 交付清单静态标签) → 0d37a997(注册表分类改身份判定) | next: review R2`

逐 finding 闭合行：
- **R1-01 → 修复**：不变量=“可发现 ⇔ 收据身份(ctx8+策略标签)==当前有效身份 且（执行级）字节=记录哈希”。修复：`discover_front_exe_with` 增 `receipt.context_id == context_id`；永久测试=异ctx+仅改标签→None（含 identity 级）+full 下正控 Some；**变异验证**（禁用比对→测试红）。最终提交真机重放：full 构建+仅改标签→`None`，full 下正控→`Some`，恢复 c 后收据回 `f2353a92/c`。
- **R1-02 → 修复+计量**：拆分 `discover_front_exe_identity`（分类，不读字节）/`discover_front_exe`（执行，+sha256）；auto-ui `entry_for_dir` 改 `native_exe_present`。真机10应用真实 `scan_apps` 计量：811.7ms→247.8ms→**2.0ms**（无收据基线1.6ms，+0.04ms/应用）；policy=full 下同扫描 10/10 拒绝且耗时不变。残余成本=执行级每次启动 81ms（一次，可忽略）；遗留边界（既有、未改）：910 前约定路径回退不在身份规则内，已写入 SD-02。
- **R1-03 → 修复**：`delivery_debug_policy_record` 纯函数——label 只记随包静态块（c/full），生成期 `CARGO_PROFILE_DEV_DEBUG` 单列 `generation_env_override_not_carried`；三态永久测试；真机重放（@2d937091 工具）标签=`c`/`managed-default`/override列示；其后 core 提交不触及交付装配，证据按身份链复用。
- **R1-04 → 不可代码修复**：主检出根 f3f98a9e 增写 4.642GiB 维持原状；转 merge 收尾路径对账+用户决定持有；不因合入标记为已清理。

受影响重放与冻结门（最终 HEAD）：伪造标签探针（上）；桌面默认→`outproc-native` 发布件/Inc×2→帧1→3、full→`inproc` 无原生进程；`cargo t auto-generation-core generated-delivery-controls`=gate PASS 37/37+15（131.7s）；auto-ui `rust_ui::tests` 串行 63过/1基线红（shell_pack，主检出示同红）/1ignored；auto-ui-compiler 1157过/2基线红（两codegen fixture，主检出示同红）；fmt=各文件基线差零新增、core零差异、core crate零警告（auto-lang-core 依赖131条为主检出既有）。

`stage: goal | PLAN-1003 | r1 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: 同上 | commit: 11e1f1fd5/0d37a997 | next: review R2`

**复审 R2（2026-11-11，受影响项复核）**
`stage: review | review_scope: final（repair re-review）| reviewer: 仍为实施会话（独立性不足的同一限制，见R1）| PLAN-1003 | r1 | outcome: **pass** | worktree: lang@11e1f1fd5 / ui@0d37a997（均干净）| base: df190c1f5 / 8d289105 | spec_inputs: SD-01/02/03 @df82d5ecc（随分支合入）`

复核范围=R1三 findings 的闭合 + 其重放真实性：①R1-01：最终提交真机判别重放（full 构建+仅改标签→None；full 正控 Some；恢复后 c）；永久测试经变异验证（禁用比对→红）。②R1-02：10应用真实 scan_apps 计量 2.0ms（基线1.6ms，修复前811.7ms）；policy=full 下同扫描全拒；残余=执行级单次81ms+既有约定路径回退边界（已写入规范）。③R1-03：真机交付重放+三态永久测试；后续提交不触交付装配（身份链复用成立）。④R1-04：保持用户持有、转 merge 路径对账，未静默闭合。冻结门在最终 HEAD 重跑：lang 两组 gate PASS（37/37+15）；auto-ui 63过/1基线红、compiler 1157过/2基线红（三红均主检出基线同红）；fmt/警告基线零新增。规范终稿通读：与实现一致，无执行日记化；frontmatter spec 元数据与落盘一致。**限制（不变）**：同会话复审；直连臂截图超时（证据=状态读回）；计量单次；安装SDK形态未实测（D-1003-C7）；T-06判定线为work阶段自定。merge-owned：最终冻结并集（已在最终HEAD跑过，merge 复核身份即可复用）、SD 落主核验、D-1001-R7/D-942-P1 关闭、R1-04 路径对账、归档与清理。

`stage: goal | PLAN-1003 | r1 | round: 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: 同上 | commit: 11e1f1fd5/0d37a997 | next: merge`

**merge 收据（2026-11-11）** `stage: merge | PLAN-1003:r1`
- prepared：reviewed 基线=复审R2 pass 绑定 lang 11e1f1fd5 / ui 0d37a997；SD-01/02/03 冻结于 lang 分支（df82d5ecc）。
- landed：auto-lang master `df190c1f5→11e1f1fd5`（基未动，rebase no-op，`--ff-only`，df190c1f5..HEAD 零合并提交）；auto-ui master `8d289105→006798ac（他会话plan-079文档3提交）→rebase 5提交→393ea17c`，旧→新映射 17220173→370f6343、a63ab8bb→10192725、0c873457→a1b7449b、2d937091→96735289、0d37a997→393ea17c，`git range-diff` 5/5 全等（安全重写证明）；上游新增为零代码触面（docs-only），全部验证证据按身份有效。主检出冒烟 `cargo test -p auto-generation-core --lib`=36/36。
- canonical_verified（落主提交内 sha256）：`docs/specs/repository-boundaries/generated-cargo-build-lifecycle.md`=123e3e423ba1db04977f15239551fe9bdf67e0995c7b641029da3ac2f61eb9a9（SD-01）；`docs/specs/repository-boundaries/generated-debug-policy-production.md`=531d154f1b71b787423fc591989386248e05ad0819415b9110e72a4afce913f0（SD-02新页）；`docs/specs/auto-test-case/build-storage-lifecycle.md`=57984327da582c9fd7527a7b3fef34e4a5f3d46e6befb30c047e499609f66b52（SD-03）。frontmatter spec 元数据与落盘一致。债务：D-1001-R7、D-942-P1（含隐藏第二层）按证据关闭于 KNOWN-DEBT。
- 债权/债务裁定：D-942-P2 维持开放（本包非目标）；D-1003-C1..C8 候选登记开放；R1-04（主检出 f3f98a9e 增写 4.642 GiB）= 用户持有项，registry 记录，未经用户决定不清理。
- cleaned：见下续记（worktree/材料/对账）。

## 10. 待澄清事项

1. T-01决定实际生产配置落点与用户覆盖/完整debug恢复接口；已批准采用C，不再重复问是否采用，不将研究scope祖先config当通用生产机制。
2. 收益/冷暖/峰值预算须据当前机器/冻结工作集预先明确，不能把研究比例写成已获准阈值；无预算/无收益不签生产目标完成。
3. 适用SDK/delivery若命中旧P1等独立生产缺口，归真实owner有限修复并正式绑定；不把C落地变成整个交付系统重构，也不以该旧红免除原适用验收。
4. OS068最终窗口等本包正式交付；本包不反向等OS068或989、ACE M2、AGE M2、Linux发行版。UI068 Reader与OS067 FeaturePack只按实际共享接口协调，不增无关全包前置。
