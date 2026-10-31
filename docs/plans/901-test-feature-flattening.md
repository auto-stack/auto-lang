---
plan_id: PLAN-901
status: executing
feature_name: Design52-B语言测试feature拍平与跨仓消费接线
author: [Codex]
created_at: 2026-10-31
updated_at: 2026-10-31
plan_revision: 1
current_step: 1
total_steps: 8
supersedes_spec_components: [docs/specs/auto-test-case/project.md, docs/specs/auto-test-case/architecture.md, docs/specs/auto-test-case/build-storage-lifecycle.md, docs/specs/auto-lang/project.md]
new_spec_components: [docs/specs/auto-test-case/test-feature-flattening.md]
touched_goals: []
affects: [auto-lang, auto-lang-core, auto-test-case, test-infrastructure, ui-sdk-delivery-consumer, web-test-consumer]
depends_on: [PLAN-898, PLAN-900]
---

# [PLAN-901] Design52 B：语言测试feature拍平

## 0. 变更摘要

在900交付后的实际语言仓中，删除七个语言测试feature及其选择职责：test-trans、test-cases、test-ace、test-book、test-aavm、test-http-e2e、test-case-batch；测试代码在同一Cargo测试编译形态可见，运行范围由固定组、nextest过滤、原ignore/平台/环境/资源条件决定。测试工具bin不再编出“需启用feature”的占位版本；本地TestCase协议/执行和增量编译行为不改变。

本合同主责auto-lang；auto-ui只做SDK发行投影/受影响消费适配，auto-web只做语言HTTP测试feature退出的接线适配，auto-man只修实际依赖已删除语言feature的消费者（若无则只验证、不改）。不实施Design52 D，不搬第二遍900已迁内容，不以“feature全部删掉”为由破坏普通语言测试不构建ACE的隔离。

当前drafting，仅立项和只读盘点；未建实施树、未编译/运行测试、未改产品/测试/用户技能，未联系其他会话或创建agent。用户要求”暂不实施”，本合同不构成实施授权。
（2026-10-31 更新：用户以 /goal 明确授权整链实施（work→review→merge）；T-01 已建树并冻结基线，见 §9。）


## 1. 目标

- G1：语言facade/Core及当前活跃工具/固定组不再用七个测试feature选择编译形态；auto-test-case按真实被调用路径改为必要依赖。
- G2：daily、具名完整组、all、显式ACE及AAVM手动入口保原职责，逐用例与内部check义务无未解释丢失；新可见测试不会自动进入旧日常范围。
- G3：auto-case、ace-certify、test-case-pilot默认构建具备原enabled行为，普通VM/transpiler输出、父侧裁判、trace与批处理观察不变。
- G4：真实语言构建、固定组、外部SDK生成/发行消费者保持可用；B交付完整，D接受明确版本与剩余UI职责。
- G5：用实际Cargo编译单元/feature身份和成本证据证明测试feature分叉退出，不把test harness或dev依赖的合法新增误报为重复编译；保留Design52的120秒增量风险决策门。

非目标：UI ui-iced拍平、UI默认后端改造/渲染测试重排；ACE独立workspace内部的test-case-batch拍平；改变code-editor/python/mem-profile/streaming-http/build-examples行为；改变debug/opt/jobs来凑收益；执行全ACE/AG性能套件；修复全部既存红；单仓无Web checkout问题900-N7；Cargo全局共享target/Kache/缓存清理专项；远端发布。

## 2. 架构方案

编译与执行分开：Cargo正常编译语言库与所请求的测试目标；fixed-tests选择组并合并等配置调用；nextest在已编译可见集合中选测试；原prepare_tools只为显式ACE消费者准备独立worker。删除feature不能让编译期、静态初始化、普通语言运行或普通测试选择隐式触发ACE worker构建。

保留#[cfg(test)]、平台条件、真实可选依赖/产品能力门和原ignore。#[cfg(all(test, feature="test-..."))]只去feature部分；#[cfg(any(test, feature="test-http-e2e"))]涉及跨crate HTTP helper，要按其实际外部消费者提供必要可见性，不能粗改为cfg(test)导致Web dev依赖调用不到，也不能把测试执行搬入产品初始化。

### B/D及其他owner交接

| 面 | B本合同交付 | D或其他owner后续 |
|---|---|---|
| 语言facade/Core | 七测试feature退役、调用与依赖闭合、工具bin有效默认形态、固定组选择守恒 | D不得恢复语言测试feature以修UI构建 |
| auto-ui/crates/auto-ui-tools/src/rust_ui.rs | DeliveryProjectionSpec及PROJECTION_AUTO_LANG/CORE接受新生产清单，SDK包包含常驻TestCase所需真实依赖；旧已交付形态是否仍支持，沿现有合同核并保兼容/漂移拒绝/幂等 | D负责UI自身依赖/feature，不承担B造成的发行破损 |
| auto-ui-runtime | 核消费Core公开HTTP helper和迁入tests，必要最小接线 | ui-iced/default/各UI测试feature（包括UI自己的test-http-e2e）及运行选择重构归D；不得由B顺手删除 |
| auto-web | 适配其Cargo.toml中Core/test-http-e2e转发与dev依赖；外部Web同名feature若仍被Man/发行入口使用，保域内入口但移除无效语言转发；真实HTTP消费与原owner执行合同保留 | Web算法/新服务能力不变；不重开890迁移 |
| auto-man/auto-api-gen | auto-api-gen语言仓接线和四仓入口核；Man仅必要语言feature消费者适配，否则只读/验证 | Man自己的test-http-e2e如属Web域，保持其既有合同；不做CLI结构重构 |
| experimental/ace | 仅语言侧桥接与case-parent构建命令退出语言test-*；必要定向消费者验证 | ACE内部4处固定组--features和worker配置保留；后续若要拍平ACE单独确认合同，不交给UI D |
| UI014/007 | 提供B正式提交、SDK依赖/投影规则、测试入口变化 | CLI/Vue/Aura/renderer内部结构归014；UI主题/整棵examples/ui归位按007和900实际差额重核 |

本计划唯一正文在auto-lang；实施使用D:/autostack/.wt/lang-901/{auto-lang,auto-ui,auto-web}，语言分支plan-901-dev，UI/Web参与仓各一专用分支plan-901-ui-dev/plan-901-web-dev。必要auto-down detached兄弟沿现有解析规则，auto-man仅确有改动时补一树并登记。所有修改、构建、测试在对应专用树，不在主检出实现；独立复审顺序使用同一权威树，不复制另一实施树。禁止任何junction/symlink。

## 3. 技术栈

沿用Rust/Cargo/nextest、固定组scripts/fixed-tests.py与test-groups.toml、原TestCase/prepare_tools和原跨仓发行夹具；Python标准库做一次性清点/比较。不得新建动态叶目录、影响图、测试runner或并行计量框架；不改现有contained Job、deadline、原子oracle和重内存限流。

## 4. 需求分析与背景调查

### 4.1 授权与当前知识

用户本轮明确授权在auto-lang为B建立正式计划，核900后范围及auto-ui影响、B/D边界，暂不实施。当前仅在主检出创建本计划/中央号源簿记及Design52进度更新；不创建worktree、不执行构建测试、不接管891/899/Reader等会话。计划确认后才可进入work。

已读docs/specs/overview.md、auto-test-case/{project,architecture,build-storage-lifecycle}.md、auto-lang/project.md、Design52及900最终收据。Spec中部分旧test-ace隔离、test-cases optional与tm档说明滞后于最新用户仓规，B必须沉淀真实变化，不批量改无关历史。来源冲突以本次明确目标及原测试义务为准，不把旧描述当已完成证据。

900已archived10/10、513/513迁移对账、树/分支清理；其收据同时保留：非独立上下文复审的局限、冷构建体积未测、旧发布二进制未重建、900-N1..N7预存债和周期批量DUE。B接受其已交付边界，不冒称替900完成这些未实测项，也不拿900旧结果认证B。

### 4.2 本次只读实物基线（实施T-01重新冻结）

语言HEAD b55889dffe970fb059ed841b9ff30e2e55f74ea9；UI 4e9acd52596c69f380447ef0608ef256439802e4；Web c48924f3381e775b8435c3989e739df347570f2c；Man cbbe6573e2d40e3237f52718dd02a3cd065d5cbd。主检出已有他方docs WIP，均保留。

| 实际面 | 观察与剩余工作 |
|---|---|
| facade Cargo.toml | 七test-*均在，default=[]；auto-test-case仍optional；src/bin三工具用cfg main/stub，不是清单required-features；4个example的build-examples是另一个门，保留 |
| Core Cargo.toml | test-cases/test-ace/test-case-batch/test-http-e2e仍在；auto-test-case optional；default仍with-file-history |
| cfg门 | 对crates下Rust逐行匹配cfg/cfg_attr含feature=test-*，26文件/124处（含重复行与组合门，非124个独立义务）；原“24文件/~115处”不再作完成分母 |
| 固定组 | tomllib只读解析100组；30个组带--features，其中26根语言workspace、4独立ACE（ace-av/ace-batch/ace-workspace/ace-performance）；不能把30处全部删掉 |
| 工具准备 | scripts/test-governance/toolchains.json的case-parent仍传六个语言测试feature；AV worker自己的batch feature另属ACE，不能误删 |
| daily/广组 | daily表达式all()，靠旧feature隐藏部分转译/书籍/HTTP/ACE桥接；integration与auto-lang/Core广组也all()。拍平后必须补固定静态选择边界，不能以参数删除自证范围没变 |
| 产品侧调用 | Core entry.rs的trace调用、a2r/a2c/a2ts旧enabled/disabled双臂、ac_av的parent_observations/test_batch均受门控；双臂需选择已约定enabled路径并保持结果/诊断/副作用 |
| auto-ui发行 | rust_ui.rs约4059–4110按旧test-cases feature行及optional auto-test-case裁剪；约8060–8400有相关正反/幂等测试。变为常驻依赖后只删行会导致SDK缺包或投影漂移 |
| HTTP跨仓 | auto-web的feature与dev-dep显式请求Core/test-http-e2e；auto-api-gen与Man仍转发Web域同名feature。必须先接收helper合同再删除语言转发 |
| dead feature余项 | facade仍有with-file-history/use-evaluator，Core有with-file-history；crates Rust cfg搜索无该两名命中。A未删掉的静态余项归B核全消费者后退出，rustyline真实with-file-history依赖不删 |
| 编译配置 | 根dev debug=line-tables-only，依赖debug=false；不得归因给B或偷偷再改。code-editor为行为门，ACE桥接组仍请求它，不能当空开关抹掉 |

来源SHA256：facade清单9F0463E0779FF637FB116DD53F799DDB861E9549861899EE59785546F40791D3；Core清单E9C8FB7F3DB1A024890FE9FF5AACA40E59A46D50B3D87AE609BD5DE230A2E239；test-groups.toml 6C1FA9B2540413983E4502C1EE4E8E05603315A8E61872ACCE507BEF37FC34AD；toolchains.json 45C270E8BE53F2B5E8BAE898F9C724C830BE4B66E790B97CC0247ECE2CFF1BDB；根Cargo.toml 722B102258415FEE932397E6C5123D91BD92FB4244F7D9C2B55195C58562E923。

以上未运行cargo metadata/tree/list/测试；不把静态解析100组当用例数或执行成功。auto-ui实验frozen SDK中的旧门是固定历史输入，不能全仓替换；仅当前生产投影和受影响消费者迁移，历史快照保持原版本身份。

## 5. 详细设计

### 5.1 编译恒定与用例职责

T-01在原基线记录默认/全七feature联合、逐组的编译可见ID和实际选中ID、原ignore/条件；只为本迁移做一次性差额账，原始JSON放计划暂存区，不生成运行时目录。T-03将七feature去除后与原最大可见并集对账：同名测试/原子子check/样本/oracle不丢，原default不选的测试不得无授权进入daily。名字必要改动须具名旧新映射。

新暴露的ACE桥接测试只能由原显式ACE组选择，语言all仍不准备/构建ACE worker；新可见HTTP仍归http/原完整职责组，网络串行/端口/TEMP/Job原约束保留；AAVM/AA2R原ignore与手动--ignored/T3条件不变，不能对正常VM/A2R批量加ignore。新可见body可编译不等于默认执行。

保留固定组职责边界和配置差异；去掉测试feature后等配置组才合并一次nextest，不为合并命令粗合资源配置。若静态过滤不足以表达身份，T-01给出具体差额与有限表达式方案，不重建tm系统。保留现有code-editor等真实能力差异，不能为了“单一”隐式给所有测试开code-editor。

### 5.2 依赖、trace与工具bin

Core/facade按实际依赖改auto-test-case为常驻；test_cases模块和必要API常驻，原cfg(test)单测仍仅测试构建。trace与batch观察的启用路径必须保持线程/会话隔离及Drop恢复，无显式执行时不spawn worker、不写测试报告；普通语言返回值/诊断不能因默认trace可见而改变。工具bin默认执行真实enabled路径，缺参数/坏输入仍正确报错；移除的是feature缺席stub，不是错误处理。

HTTP helper跨crate由Web测试消费，不能仅留cfg(test)；选已有必要公开helper或中立支撑，不复制第二套网络裁判/服务实现，不扩大生产自动装配。UI发行需同时承接常驻auto-test-case及vendored parser真实依赖闭包；不因旧投影习惯把正在使用的依赖剥掉。

### 5.3 成本、不变量与风险门

Design52原“cargo build→cargo t all零新增rlib”不能直接当所有目标的绝对判据：test harness、dev依赖、平台目标和code-editor真实行为feature可合法产生不同单元。T-01按同包/同工具链/同profile/triple/同真实能力/相同env，冻结可比较的正常库单元，单列test-only目标及必要新增依赖；无证据不得把实际重复判成合法变体。

正式验收须证明：七测试feature不再出现在语言编译身份；完成所需test/dev目标准备后，切换语言固定组不会仅因测试选择新增另一份同角色库rlib；同配置调用实际合并；真实code-editor差异保留且具名。T-01若发现还需额外全仓profile或code-editor语义变更才能满足原绝对断言，先形成Design52差额决策，不能静默降低目标或顺带实现；本计划承诺的是测试feature导致的分叉归零，Design52整体零新增目标不由B单独宣布完成。

在相同配置、相同包/测试编译目标、清楚的冷/热状态下对照拍平前后编译及日常edit-rebuild。记录准备/冷编译/发现/运行/清理/整命令、用例列出/实际执行/原排除数。保原debug/优化/样本/工具链，不设强制百分比收益。因AAVM/HTTP测试体常编译使可比日常准备或增量重建额外超过120秒时，先提交原因与保留单一test-aavm分叉等选项供用户重议，不自行采用回退、不删测试凑门；冷态总耗时本身超过120秒不等于额外成本超限。

### 5.4 接收顺序、证据与回退

先准备UI/Web接收面的新旧清单/最小helper适配，再移除语言feature；冻结跨仓SHA组合及受影响发行/工具读者。合入窗口串行，B不能合入一个令当前UI/Web/Man清单引用不存在feature的组合。回退用同一组各仓提交逆序恢复，原target只属缓存不能当兼容来源；不回退900已交付零UI反向边。

实施开工创建并登记D:/autostack/plan-artifacts/plan-901/（owner=auto-lang PLAN-901，跨仓子目录区分），计划以外所有探针、对账、日志、发行包、外部消费副本和输出统一在该scope；正常Cargo target留各专用worktree。现有runner若硬编码项目内输出，调用时显式指定外置路径；确需改设施才能遵守本规则的最小接线归本计划，不默认授权另建证据系统。docs/plans/attachments禁止任何内容入库。merge将必要摘要折叠本计划，逐项对账所有输出路径后删整个scope与worktree；仅具名未解决问题可按owner/关闭条件持有。无实施的本次规划不创建scope。旧最近5个策略不再适用。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/test-feature-flattening.md（新） | 多语言测试feature→常驻编译/运行选择、身份对账与成本边界、B/D交接 | 唯一新合同 | AC-01..06 |
| SD-02 | modify | docs/specs/auto-test-case/{project,architecture,build-storage-lifecycle}.md | test-cases/test-ace编译门→依赖常驻、显式组隔离；独立ACE workspace/profile保持 | 与896/898衔接 | AC-01/02/03/05 |
| SD-03 | modify | docs/specs/auto-lang/project.md | 测试feature转发→稳定语言构建及工具入口；仅触及本次合同 | 保900零UI边 | AC-01/04 |
| SD-04 | modify | auto-ui现有SDK发行规范对应章节（T-01按docs/specs实际定位，不虚构路径） | 旧optional裁剪→常驻TestCase闭包及新旧投影支持边界 | 不是D的UI后端拍平 | AC-04 |
| SD-05 | modify | docs/specs/repository-boundaries/web-four-repository-delivery.md（若T-01核有现行语言feature合同则列入最终元数据） | 更新HTTP helper/测试转发接收组合，既有owner验证不降级 | 仅实际差额 | AC-04/07 |

## 6. 测试设计

以下均未执行。实现/内部核查由work负责；独立终审由review在同一实施树、独立上下文逐条重建，不能把实施者自查标独立通过。本规划不创建agent。必要独立角色在执行阶段按review技能安排，用户未指定模型。

| case_id / owner task | AC | 输入→入口 | 可区分错误实现的结果与边界 |
|---|---|---|---|
| V-01 / T-01/03 | AC-01/02 | 原各feature联合及逐固定组枚举→新无测试feature枚举 | 逐ID/ignore/原子义务映射；default/具名/all分别对账，新增仅编译可见不冒称默认多执行合法；不改变金样/样本 |
| V-02 / T-03/05 | AC-02/03 | cargo t daily、cargo t parser vm a2r a2c a2ts corpus http（完整受影响职责）；广组由实际触面合并 | 旧执行/排除集合对应，HTTP原串行条件；普通daily/parser零ACE worker准备；显式ace-adapters仍接原工具；错误新增选测/0选择不能绿 |
| V-03 / T-02/05 | AC-01/03 | cargo build -p auto-lang --bins；三bin --help及原合法/坏参数；legacy TestCase真实VM/a2r消费 | 无feature缺席stub；正确完成/诊断/报告/exit；trace/batch观察开关改编译方式但值/错误分类/父侧裁判不变；工具未被调用时无额外worker |
| V-04 / T-02/05 | AC-03 | 原ac_av batch/parent_observations合法、坏envelope、取消/reap和作用域恢复控制；显式cargo t ace-adapters及必要ace-batch | 原并发/Job/预算/样本保持；父侧拒绝伪PASS、作用域Drop恢复，不因常驻观察串会话；不跑整个ACE性能档 |
| V-05 / T-04/05 | AC-04 | auto-ui-tools rust_ui投影命名单测与既有发行/SDK消费者；新正常清单、支持旧清单、坏/重复依赖行及再次交付 | 新SDK实际fresh/locked生成构建可消费；依赖不漏、投影幂等、坏清单仍拒绝；不能只改测试expected通过；固定frozen SDK历史不重写 |
| V-06 / T-04/05 | AC-04 | 原Web HTTP host/helper测试及auto-api-gen/Man最小受影响消费；从专用树调用原owner入口 | Cargo不请求已删语言feature，真实JSON/file/upload/关闭与对应helper行为不变；不以改到主检出或旧exe回退冒绿，不重跑所有四仓应用 |
| V-07 / T-01/05 | AC-05 | 同环境cargo build JSON编译单元→准备必要test/dev目标→不同语言固定组构建；可比编辑一次 | 正常库单元/feature/hash无测试选择分叉；合法test/dev/真实能力单列；父command0不代替child结束；额外>120秒触发重议 |
| V-08 / T-03/05 | AC-02/06 | cargo t aavm枚举及原--ignored手动入口可达性，T3条件/重内存组静态+命名控制 | 原AAVM/AA2R继续ignore、正常VM/A2R不误关，XL/LG/MD与heavy_gate保持；不运行长塔凑验收 |

最终冻结候选并集：语言完整职责auto-lang、auto-lang-core、auto-test-case、auto-api-gen，加设施infra-fixed/infra-meter/infra-pilot-cli及必要integration；编译枚举与具名组已证明同义部分按不变输入复用，不机械堆叠全仓轮次。ACE为真实共享桥接触面，明确跑ace-adapters及受影响ace-batch/原case suite，T-01具名选择；独立ACE性能/全部产品未改不追加全线。auto-ui/auto-web按本次受影响投影/HTTP测试族与发行消费验证，原owner入口保留，不拿语言组代替。若独立复审发现触面扩大，具名补必要完整组，不静默编全部experimental。

已有900-N1..N7、898-N5以同身份/同输入证据核对；初红、环境归因、修复和重放分列。新触发缺陷若本次常驻路径所必需须修或重议，不能一律推给预存债；无关预存红具名登记不伪全绿。周期cargo t all DUE仍主检出单实例另行跟踪，不作为B每步全局门。

## 7. 验收标准

- [ ] AC-01（owner_stage: work）：七语言测试feature和其当前生产/测试/工具消费者引用退出；cfg(test)/真实能力门保留，三bin默认真实enabled行为；dead feature余项核清后处理，清单/源码搜索及真实构建均证实。
- [ ] AC-02（work）：V-01/02/08逐ID/子check和原条件守恒；daily不擅扩、语言all不隐式准备ACE，AAVM手动/ignore及重内存纪律保持；新增可见/原排除/真实执行分列。
- [ ] AC-03（work）：trace/批处理观察及ACE桥接正反/隔离/回收验证通过，普通legacy入口独立可用，无错误PASS或未调用即spawn。
- [ ] AC-04（work）：UI新SDK投影/依赖/实际生成发行消费、Web最小helper接线与API/Man实际受影响消费通过；无已删feature引用，支持矩阵与D交接版本明确，900零UI反向边不回归。
- [ ] AC-05（work）：测试feature导致的编译形态分叉归零有真实单元证据；同义成本/空间观察分列，120秒额外成本门已核，不能借改配置/减少测试过门。
- [ ] AC-06（owner_stage: review）：独立上下文逐AC/遗漏/延期/workaround与健康审查通过，规范增量及必要组选择核实；已知警告/红按来源处置，新增未处理警告为0。
- [ ] AC-07（owner_stage: merge）：所有实际修改仓以冻结review版本及有效证据串行落地，canonical知识沉淀，不写specs.json；B→D版本/剩余范围与回退记录齐备；归档、guard清树与统一scope路径对账/删除完成。

## 8. 执行步骤

| ID / owner_stage | 前置 | 实物路径/动作与交付 | 关闭证据 |
|---|---|---|---|
| T-01 / work | 用户明确批准本合同实施 | 唯一lang-901组建树；复核§4全部来源/消费者；登记plan-artifacts/plan-901；一次性旧新可见/选择/条件基线，冻结精确组/跨仓清单与成本测法；解决§5.3单元判据，不改产品语义 | [✅ 已完成] 组 D:/autostack/.wt/lang-901/{auto-lang@afcbd7084(plan-901-dev), auto-down@7c4d5cb(detached), auto-ui@4e9acd52(plan-901-ui-dev), auto-web@c48924f(plan-901-web-dev)}；§4.2 全部复核吻合（两清单/test-groups/toolchains/根toml sha256 逐项一致，124门/26文件，100组=26根+4ACE，case-parent 6feature）；旧基线枚举 `cargo t <26根组> --discover-only` 全 DISCOVERED（6m13s，daily 2650sel/101excl，21组同配置合并 3354sel/745excl，aavm/auto-lang/auto-lang-core/ace-adapters 各自 DISCOVERED）+ 七feature联合枚举 13 suites/4282 cases；冻结文档与逐门清单 plan-artifacts/plan-901/{t01_freeze.md,t01_baseline.json,t01_baseline_discovery/,t01_union7_enumeration.json}；成本测法冻结于 t01_freeze.md §4（同包/同profile/同triple 单元 dump 对照+组切换增量+edit-rebuild 120s 门）；新增触面：experimental/ace/tests/conformance 依赖 core 四feature、auto-test-case→vendor/autodown-test-parser 入SDK闭包 |
| T-02 / work | T-01 | crates/auto-lang{,-core}/{Cargo.toml,src/lib.rs,src/entry.rs,src/test_cases/**}及src/bin三工具：常驻依赖/必要helper/trace/batch调用，移除stub与反向cfg旧路径；先准备跨仓接收补丁 | V-03/04，AC-01/03；中间状态不冒可交付 |
| T-03 / work | T-02 | crates/auto-lang/src/tests.rs及盘点26文件；test-groups.toml、toolchains.json、固定组治理测试、活跃PS/Python/脚本调用者：移除语言测试feature选择，补固定运行边界；仅核实后的dead feature退出 | V-01/02/08，AC-01/02；独立ACE内部4处feature及真实能力不误删 |
| T-04 / work | T-02/03 | auto-ui rust_ui.rs投影/相应测试与发行清单；auto-web清单/HTTP helper消费者、语言auto-api-gen与实际Man消费者最小适配；写B/D接收表和回退组合 | V-05/06，AC-04；不实现D，不改frozen历史版本 |
| T-05 / work | T-03/04 | 冻结组合跑受影响完整组/必要ACE桥接和跨仓消费；同义Cargo单元与额外成本测量；更新仅本次活跃文档/仓规test-ace编译隔离说法并拟SD | V-01..08结果，AC-01..05；超成本门先回报重议，不自改目标 |
| T-06 / review | T-05 | 同树独立上下文复审全部验收、强反例/守恒/延期/健康/SD与输入身份 | AC-06；实现者自查不冒独立终审 |
| T-07 / merge | T-06 pass | 冻结提交跑必要组并集或复用同输入有效证据；各实际修改仓按接收顺序合入，canonical SD01..05真实沉淀；记录D剩余合同入口 | AC-07落地部分；不写旧ledger/index，不宣称D或Cargo全治理完成 |
| T-08 / merge | T-07 | 摘要折叠本计划；按最新仓规所有输出目录对账、scope安全删除、逐树wt-guard clean后移除树/分支/组；归档终态 | AC-07收尾部分；未解决问题才具名持有，未知不盲删 |

下一执行入口T-01，唯一必要启动决定是用户明确实施本合同；当前只交付draft，0/8。已观察的跨仓兼容改动在B合同内，不另起重复UI/Web迁移计划。若某必要消费者确需超出§2最小接线的产品改造，先给证据与合同修订，不默默延期或归给D。

## 9. 复审记录

2026-10-31 / stage: new / PLAN-901 r1 / outcome: pass（规划合同自查，非实施或独立终审PASS） / 授权=仅立项；已核900交付与当前源码、100固定组/30带feature分属26+4、26文件124门、UI投影/Web接线及真实能力差异；T01..08覆盖AC01..07、SD01..05，V01..08未执行。next: 用户确认实施后work T-01；无建树/测试/代码提交/push。本轮仅本计划、号源和Design52进度簿记，保全他方WIP。

2026-10-31 / stage: work / PLAN-901 r1 / T-01 / outcome: pass / worktree: D:/autostack/.wt/lang-901/auto-lang @afcbd7084 (plan-901-dev) + auto-ui @4e9acd52 (plan-901-ui-dev) + auto-web @c48924f (plan-901-web-dev) + auto-down detached @7c4d5cb / 证据: plan-artifacts/plan-901/（t01_freeze.md 冻结文档、t01_baseline.json 逐门清单、t01_baseline_discovery/ 26组旧基线枚举全 DISCOVERED、t01_union7_enumeration.json 联合枚举4282、t01_baseline_recon.py 脚本）；登记 .retention/policy.json auto-lang/PLAN-901 / 主检出预检: 他方docs WIP保留；未跟踪 test/fixtures/reader-import-budget/ 属他方会话，未触碰 / next: T-02 语言仓常驻化

## 10. 待澄清事项

1. Design52全局“零新增rlib”与Cargo正常test/dev及code-editor差异：T-01以真实单元分类形成可执行判据，不能把所有新增归罪测试feature或用合法差异掩盖重复；如原设计要求改变真实能力或profile，需另行决策。本包不擅改产品语义。
2. UI发行仍支持哪些旧生产/installed清单、TestCase常驻后是否需要扩大受管依赖闭包，由T-01沿当前投影正反测试/正式发行合同冻结，T-04必须实际消费验证，不留给D补破损。
3. 若>120秒额外成本，用户决定坚持常驻、保留单一AAVM分叉或其他有据方案；无此结果前不预留豁免，不把普通慢测试秒数当编译增量。
4. auto-web-contract跨仓路径/900-N7、旧二进制与周期回归、Design52 D/ACE独立workspace拍平以及完整Cargo生成缓存问题仍独立跟踪，不随B自动关闭。
