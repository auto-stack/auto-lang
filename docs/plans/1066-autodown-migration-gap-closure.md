---
plan_id: PLAN-1066
status: executing
feature_name: AutoDown迁移补漏与1062收口纠正
author: [Codex]
created_at: 2026-11-16
updated_at: 2026-11-17
plan_revision: 4
current_step: 9
total_steps: 13
depends_on: [PLAN-1062]
supersedes_spec_components: [docs/specs/auto-test-case/document-migration-closure.md, docs/specs/auto-test-case/architecture.md, docs/specs/auto-test-case/document-cases.md, docs/specs/auto-test-case/governance-and-inventory.md, docs/specs/auto-test-case/module-selection.md]
new_spec_components: []
touched_goals: []
affects: [auto-test-case, auto-lang-core/test_runner, auto-lang-core/entry, auto-lang/tests]
---

# [PLAN-1066] AutoDown迁移补漏与1062收口纠正

## 0. 变更摘要

将1062交付后查出的四类残留在一个补漏计划内关闭：S调度器旧围栏，N原生目录发现/执行入口，P两个a2r工程与全语料rustc门，C conformance生成器。先列实际输入、消费者和独立判据，再迁移/切入口，最后对当前活跃入口重新发现。不能以“文档在盘”代“实际读文档”。

本次授权只有规划和1062收口文档纠正；不编译、不执行产品测试、不迁语料、不切入口、不建实施树。1062保留archived/20 of 20及原交付收据；新增事后更正，不倒填历史PASS或重开原计划。

| 轴 | 本轮静态盘点 | 迁移与验收分母 |
|---|---|---|
| S | 当前76个runner=case组；75组仅含文档作者文件，scheduler仍旧围栏 | S01～S11，11例/11检查，8成功+3诊断；其余75组原身份保持 |
| N | 4个公开Test.run_*_dir方法、5个实际根；旧发现算法选1050项 | VM496、a2r250、a2c118、a2ts85、cookbook101；1161个文件oracle引用；目录项不冒Rust注册数 |
| P | 2个特殊工程+1个全语料编译门 | 2工程/5个Auto源，原16+3=19个显式断言；编译门1个Rust test、250个子输入（与N/a2r重合） |
| C | bootstrap闭集30；独立固定conformance验证33 | 30个生成候选不算30个测试PASS；33验证原身份/oracle保持，普通生成器注册应退出 |
| 关闭口径 | 旧文本仍在盘；部分B helper已正确读doc | 旧文本可保冻结历史；所有本包活跃路径的源/oracle载荷、发现资格都不得依赖旧sidecar |

**r2规划复审补充（2026-11-16，未实施）**：N的1161是现存文件oracle引用；另有9个缺失header槽位仍判空，故原判决角色共1170。N0028/N0030的现有doc实际漏了返回值oracle，须补回，不可视为已无损迁移。五根还有78份不属N1050的文档，选择资格必须保边界。N相邻消费者中的VM cache 506项/8个脚本标记、cookbook VM 108个包装绑定（Windows声明77 active/30 ignore/1 cfg_attr ignore；6个目录多源）也有旧文件选择依赖，纳T-03/08/09；这些和1050有重叠，不相加冒唯一case数。C同时关闭普通包装的旁路生成/仓内wrong输出，不只删bootstrap函数。

P在r2同类扫描还确认3个原ignored手动guard直接复制6份旧Rust金样（M表），一并切到既有doc产物载荷并保手动条件；6输入均重叠于N/a2r250，不加到250冒新分母。

这些轴存在重叠，禁止相加冒唯一代码case总数。N表逐1050项指向既有doc，P编译250引用其中a2r行；C的33不等于bootstrap30。补漏不是重跑全部ACE/全仓回归，也不重做1062已有效迁移的全部语料。

2026-11-16 r3阶段裁定：A只关闭判据/集合/可信执行与正常路径零作者写入的正确性切片；完整S/N/P/C载体迁移与250编译campaign明确B。原13任务/0完成、1050/1170等集合与全部27原负控保留，新增4个A片段负控；不是把未修补漏改成PASS，也不重开1062。

## 1. 目标

1. 新旧输入、原比较语义、完整原子结果、消费者身份及条件守恒；普通作者修改doc即改变实际执行输入/预期。
2. N发现与执行均以doc为权威，doc-only仍可发现并运行；移除或污染旧文本不会改变结果，缺doc时有旧文本也必须拒绝。
3. P工程的全部Auto模块与19项原期望集中到两份doc，受管物化后调用原项目转译器；不得将真实多文件入口再豁免为support。
4. 全语料rustc义务仍真实执行，日常入口没有隐式rustc/cargo/linker。编译准备显式、debug/test、源绑定；专项和普通正确性各记。
5. C普通验证无写金样副作用；显式proposal仅写指定scope，保30输入/33验证与独立oracle。
6. 最终重新扫描全部当前case组/目录消费者/公开shims/硬编码工程/生成器。新增活跃项纳同一计划，不拿冻结快照子集全绿证明“全部”。

边界：不重写VM/转译器/调度器语义、不改原正式压力规模/原ledger规则、不把正确性用例挪到ace-perf。Rust实现算法单元测试、真正SDK资产、外仓book/UI/Man/auto-term不因本计划全转文档；逐路径具名，不能把本包源/expected改称support避迁。Linux/WSL不在Windows运行，1061仍为环境隔离owner，不是本包Windows实施前置。

## 2. 架构方案

AutoDown作者 → 共享只读文档输入视图（源/虚拟文件/目标/原比较器/选择资格/身份） → 原发现函数/原Rust包装/原Test.run_*_dir shim → 同一VM或转译器 → 原独立oracle/实际报告。

优先复用auto-test-case author/CaseSpec/Payload及core的legacy_doc_for_trans与既有output admission；统一必要的doc读取，禁止复制一套后端/发现引擎。公开方法名和目录参数保留；目录发现不再以.at/expected存在性作入选条件。特殊工程把doc载荷物化到受管目录再调原transpile_rust_project；编译器原生产文件API仍可读产品工程文件，本计划只关闭测试作者源侧载。

## 3. 技术栈与位置

- 现有：crates/auto-test-case/src/{author,contract,document,resolve,compare}.rs；crates/auto-lang-core/src/{test_runner,entry,legacy_doc_for_trans}.rs；vm/ffi/stdlib/sys_env.rs；test_cases/{inventory,execution_trace,materialize}.rs。
- 包装：crates/auto-lang/src/tests/{legacy_doc,a2r_tests,conformance_tests,vm_file_tests,a2c_tests,a2ts_tests,cookbook_vm_tests}.rs；core/src/trans/{rust,python}.rs；test/runners/{vm_tests,a2r_tests,a2c_tests,a2ts_tests}.at。
- 治理：scripts/test-groups.toml、scripts/test-governance/{document_case_migration,input_inventory,wrapper_bindings,prepare_tools}.py及input-contract.json；auto-case现有CLI及proposal/保护根设施。
- 新文件仅提案：scheduler.document.case.ad、两个main.project.case.ad、所需小型正负控；编译准备/验收适配先复用现有工具与receipt，T-01/02决定准确共享模块，不建第二缓存或动态目录。
- 唯一未来实施树 D:/autostack/.wt/lang-1066/auto-lang；分支codex/plan-1066-dev（若repo正式脚本要求plan-1066-dev则登记实际名）。实施/内部检查/独立review在同树顺序进行，不创建review专属树。
- 材料唯一 D:/autostack/plan-artifacts/plan-1066/，开工登记owner、用途、在途状态。准备/物化/实际/proposal均显式输出scope子域；正常target在专属树，外置target仅scope并登记可删除。纯规划临时材料在系统临时目录，不入仓；merge摘要入Plan后清整个scope并对账所有实际输出路径。

## 4. 需求分析与背景调查

### 4.1 授权、来源和事实

用户2026-11-16明确要求建立包含四类残留的补漏计划，列实际用例、入口切换、负控和验收，同时纠正1062收口记录，先只规划。未授权实现/测试/入口切换/发布。规划基面HEAD=2cdb28fb353e615619b533f6a69c72496362a9ec；并行其他计划WIP保持原样。

来源：[Spec总览](../specs/overview.md)、[文档合同](../specs/auto-test-case/document-cases.md)、[架构](../specs/auto-test-case/architecture.md)、[模块选择](../specs/auto-test-case/module-selection.md)、[治理](../specs/auto-test-case/governance-and-inventory.md)、[收口Spec](../specs/auto-test-case/document-migration-closure.md)、[1062归档](archive/1062-remaining-code-cases-to-autodown.md)。以实际代码核查，不以archived/checked证明全量关闭。

- scheduler提交cad341817于2026-11-15 14:12形成11例，已经是1062交付42a1b19d1的祖先。因此不仅是交付后新增；1062最终增量扫描/“全部75组”表述漏掉了当时已进入主线的第76组。
- N的entry.rs四个run_*_file_test_impl仍读case.source_file和expected路径；sys_env.rs四个shim仍调用这些函数；不是仅保留的死代码。test_runner.rs发现函数也依赖旧文本存在性。
- a2r_tests的两个project测试仍从真实main.at进入；a2r_rustc_real_compile_gate无ignore，250输入里直接读旧源并spawn rustc。不能把它与已ignore的单例compile_smoke混淆。
- conformance_bootstrap仍普通#[test]，generate_expected仍read input.at并write expected_output.txt；错误仅打印。它不是独立oracle验证，也没有“仅proposal可写”保护。
- 常规B包装多数已正确切doc，1050目录选中项对应doc本轮全部存在；问题是部分消费者/发现仍绕过。已迁移的正常通道必须复验守恒，不能重做格式回退。

本次N计数由逐句对应当前Rust目录选择规则的只读文件盘点得到，不执行原生discover/body、不调用会触发prepare的input_inventory.discover_inputs。四族五根共1050项；T-01须以同源实际发现/注册复核，不能把静态脚本当最终运行证据。旧input-contract.json仅保早期floor（1044条：vm494/a2r246/a2c118/a2ts85/cookbook101，VM cache504），不等于当前1050/506；不覆盖旧floor换绿，扩充当前权威doc义务时保原subset证明。

r2复审基面HEAD=aa257fa1425c31fab48bd544cbd116b2fdce7dcd。只读辅助围栏/字节抽取核1050源和1161现存oracle角色，发现两项doc未承载原result；C入口还对9个不存在的header路径使用unwrap_or_default，不能因盘点只数现存文件而漏该空值判断。该辅助抽取不是生产decoder/runtime执行证据，work仍须用同源author/实际入口复核。新增相邻范围来源为vm_file_tests.rs的is_script/只读inventory、cookbook_vm_tests.rs的.at选择/GENERATE_EXPECTED，以及conformance_tests.rs的.wrong.out；属于原四类中的发现/消费者及生成器旁路，不另开第五个迁移计划。

### 4.2 既存债务与本包边界

D-P3-A2X-DRIFT（54对旧金样漂移）、mn温存环境红、cb_file_png CWD竞争不由格式迁移认证通过；首次红/旧基线/排除/NOT_RUN分列。KNOWN-DEBT另登记cache-roundtrip文档缺status: executed，本包共享author/投影回归须观察并保留该具名红，不将未处理项充PASS；独立语义债不借补漏计划顺手改语言。若查明新消费者丢原metadata导致该类漂移，则按本包输入视图合同修复并记录准确对照。

本轮授权与来源（2026-11-16）：用户只要求重冻985/A验收合同、核1024及审计后继和1066的A/B边界，明确任务/关闭条件并更新A顺序；不启动实施，不新增Plan。读取主线107b23d0f69c6ad11d036dc37bed0b688493db30、当前Spec导航/相关模块合同、1024 F01..35/S01..13、1027/1028归档收据、1029..1038和1066当前合同及实际源码消费者。静态风险不是已经复现的bug；A行须真实修复后验证或可复核充分反证，未执行保持pending，不能以本轮分类当PASS。原资源/平台/性能/ignore与UI三波边界保持。

实施授权（2026-11-17）：用户以 /goal "完成计划1066/A" 授权 A 切片端到端交付——在唯一实施树 D:/autostack/.wt/lang-1066/auto-lang（分支按本仓规约 plan-1066-dev）执行 T-01/02/03/07/08/09/10/11 的 A 子面、NC-A1..4、T-12[A] 独立 phase 复核与 T-13[A] 正式 fold（IF-1066-A-CORRECTNESS，保树予 B）。材料 scope D:/autostack/plan-artifacts/plan-1066/。不改本 Plan 目标/验收阈值/兼容承诺/仓范围/预算；B 载体迁移与 250 编译 campaign 不在本次授权内，保原义务与负控不删。

## 5. 详细设计

### A-EXIT-20261116：A必须的正确性与B载体迁移分账（本轮当前）

现存N0028/N0030文档确实只有vm-stdout，旧result分别是4字节`succ`和2字节`10`、均无尾LF；本轮只读重核旧oracle与doc hash匹配r2，未修改。九项原缺H文件的角色仍是**空字节header比较**，不能改成无检查。1050输入/1161文件引用＋9空H=1170判决角色是不同层次的量；cache506/B108/E78/P250/C30/33/S11有重叠，不能相加冒唯一case或PASS。

| 切片 / 原任务与AC | A出口前必须关闭 / 精确oracle | B保留义务/关闭条件 |
|---|---|---|
| A-DEN / T-01/02、09/10/11子面；AC-01/07/08 | 冻结S11、N1050/1170角色、E78、cache506/8脚本、B108/6目录实选、P2工程5源19断言/P250/M3-6输入、C30生成与33验证的**原义务和当前消费者**；逐键映射A/B/平台/ignore/actual来源，缺项/合并同名族/空H变缺席须红。A全部required正常验证仍按985真实来源执行，不以仅数目录签覆盖。 | 全入口doc-only迁移、完整v2治理/trace与零活跃旧载荷扫描在T-03/08/09/10的B部分完成，原集合/条件/新增合法doc选择不减；E78不得误入N。 |
| A-ORACLE / T-03、10/11子面；AC-01/02 | N0028/N0030补原vm-result和final-newline=false，A实际采用的doc/ACE adapter同一次运行独立采stdout/result；只错result或删通道必须红。九空H角色显式保零字节及header轴，A适用C控制不能因不存在.expected.h跳过。A若用旧入口，必须绑定真实未改变原oracle/选择身份，不能借旧入口绿声称新doc已完整。 | 所有native目录/VM cache/cookbook切doc及原NC-21/22/24全路径在T-03/08/09/10 B关；两个原result的缺口本身不得留B。 |
| A-NOWRITE / T-07、03/08、10/11子面；AC-06/07/08 | A验收实际会执行的普通路径禁自动生成/覆盖oracle及仓内wrong输出；复用既有scope输出/关闭bootstrap普通写入或选择显式只读验证路径，作者树hash＋新增文件/实际写路径核实，错oracle仍真实红。原33独立conformance验证与原检查保持；generator/proposal是作者操作，不算语言验证PASS。不能靠运行后回滚作者文件掩盖写入。 | T-07完整显式proposal读30 doc/只写scope、所有普通bootstrap/GENERATE_EXPECTED旁路退出，以及NC-16..18/25全局回查仍B；A防护不等于这些已迁完。 |
| A-EVIDENCE / T-02/08/09/10/11子面；AC-02/07/08 | A每个实际check记录source/oracle/选择资格/虚拟path/工具/producer/actual与独立判决；同族/多源/脚本原lower位置与原条件保持。源/hash不同、只stdout冒双通道、假的父PASS、空选择、缺工具/NOT_RUN不能签A。旧载体或doc路径按当前实际版本标识，不能拿旧路径结果当doc-only证据。 | 完整v2库存/trace/文档唯一路由的T-08/09及NC-19/23/24/26 B关闭，保显式历史v1和兼容规则；本轮允许载体迁移时序后移，不允许A执行语义/判据或证据失真后移。 |
| B-FORMAT / T-04/05、T-03/08/09余量 | **不作为A前置**：scheduler S11旧围栏→doc、两工程5源/19期望项目doc、N/caches/cookbook全部文档唯一路由与治理v2；A期间原检查/规模/条件实际保留且有可信来源。 | M2A-08后完整doc-only/污染/脚本/路径/多源/漏项负控及全部原actual通过，T-12独立review、T-13交付；不能因“格式在B”永久保fallback。 |
| B-COMPILE / T-06 | **不增加A的隐式250项Rust编译门**；原250输入/原test及ledger/known-broken/obsolete/external-dep/ignore/平台条件和M3手动guard不删；已有非图形A2R必要正确性证据仍由原能力/985提供。 | 在B的A2R工作窗口/最终候选前按r2显式prepare→250真实编译/分类＋M3/6独立判据→只读receipt验证，NC-13..15/26/27齐；不拿UI第三波的一个例子替250，也不把250每个UI计划附跑。 |

A阶段交接原T-01/02/03/07/08/09/10/11中的上述子面→独立T-12[A] phase复核→T-13[A]按既有fold正式交**IF-1066-A-CORRECTNESS**（拟标识，非新Plan/状态）；13原任务整项只在原完整范围齐后勾。唯一树保留/resync做B，不能按A切片假归档。985/A只消费这份正确性切片与真实A检查来源，不等本Plan13/13，也不等全1050已转新载体。T-09/10/11/12/13 B最终仍核全部原范围，不删除旧NC/AC。

旧r2中“当前全入口零旧载荷”是本Plan完整B交付谓词，不再反置为A必需；原独立oracle、格式语义和正常全量发现的最终要求不减。此处只改时序；未来实施A小修与B迁移可共享机制，但不能为一个A缺result强行先实施所有API/CLI/proposal改造。

A阶段允许保留的是旧测试载体/输入接口，不是旧引擎冒新引擎：A语言/库/FFI支持的通过必须由真实ACE producer/worker执行、按原独立oracle判定；旧AutoVM/AST A2R仅可作为冻结来源/具名对照，不能计ACE通过。T-03[A]/T-08[A]须在每条actual中记录engine/backend身份，选择旧入口时也须沿985原adapter实际dispatch ACE；若接口尚不支持，保pending并由该A子面补接，不能用旧运行绿或格式延期豁免。

### 5.1 四类输入和切换合同

| 类别 | 原作者/入口 → 新作者/入口 | 不变的判决 | 切换前必须完成 |
|---|---|---|---|
| S | scheduler.suite.ad include scheduler.case.ad → 同目录scheduler.document.case.ad | 原11个case/check ID、recipe、d_main、成功exit/双流和3个stage/code | 11项v2无损投影、真实旧新actual、负控；再改唯一include，扫描全部当前组 |
| N | discover_* + entry::run_*_file_test + sys_env/Test.run_*_dir → 同名公开入口读共享doc视图 | 原排序/命名/目录参数/空集错误、按family比较器、bootstrap/path规则、每项失败数 | 1050项源/oracle/资格/命名表；四shim真实调用；doc-only与污染控制 |
| P-project | test_14_modules_005_multi_file / test_16_interop_021_path_dep → 两份main.project.case.ad | 5源、entry/module名/源目录语义、原16+3期望 | 文档源闭包/原期望降低、受管物化、原转译器实际输出；停读原项目源 |
| P-compile | a2r_rustc_real_compile_gate直接读旧源/循环rustc → 显式准备+250项真实编译收据+验证 | 原test身份、250子键、known-broken/obsolete/external-dep/新失败规则 | 原子源绑定、工具/参数/结果分类、默认入口不启动编译、显式入口不掉子项 |
| C | 普通bootstrap/test生成函数写仓内oracle → 显式conformance proposal | 30原输入；33验证及原Unicode trim独立比较不变 | 源读doc、候选只写scope、正常测试零作者写入、33原验证+负控 |

A已迁75组的完整源/oracle身份不可因本包无故漂移；scheduler recipe加入注册author白名单时保entry/caps/成功失败互斥、双流空值与缺席差异。原suite ID、fixture_root、group表达式/条件保持；新文件按原目录放，不能改公共fixture根绕过路径保护。

### 5.2 N：实际目录范围、选择资格与原oracle

根均为crates/auto-lang/test/<family>；cookbook通过Test.run_a2r_dir，另外四族用各自同名方法。test/runners保四文件/五个测试函数及方法调用；不把全部1050项塞成一个新增日常Rust测试。

| family | 当前输入项 | 原文件oracle观测数（非assert宏数） |
|---|---:|---|
| vm | 506 | .error=14, .out=295, .result=189 + B 期登记 10 个 doc-only 项（其中 `017_sub_semantics` 的 stdout 角色由 R2 修复补入） |
| a2r | 250 | .rs=250 |
| a2c | 118 | .c=118, .h=109 |
| a2ts | 85 | .ts=85 |
| cookbook | 101 | .rs=101 |

VM有同时判stdout/result的项，C可同时判C/H，故1161文件oracle引用不等于1050个逻辑检查；每项仍有原执行成功/错误分支义务。N附录逐项列family/id/doc/原oracle角色/源与doc hash；T-01补逐条原子判据、旧发现ordinal、source语法/path/依赖及实际条件。

#### r2补齐的实际oracle与身份合同

| 已定位输入 | 原判据/现文档缺口 | 本包必须交付 |
|---|---|---|
| N0028 vm/04_control_flow/004_if_with_bool | stdout="I won!\n"；result="succ"（4字节，hex 73756363，无尾LF）；doc只有vm-stdout | 补check-id vm-result及final-newline: false；同一次VM运行采集并独立比对两通道，不能只补标题不接执行 |
| N0030 vm/04_control_flow/006_is_stmt | stdout="Here is 10!\n"；result="10"（2字节，hex 3130，无尾LF）；doc只有vm-stdout | 同上；故意只错result、stdout保持正确时，原生入口和VM缓存包装均须红 |
| N0792～N0799及N0801（九项，详N表） | a2c/10_collections的003_array_declaration、004_array_mutation、005_array_index_read、006_array_copy、007_array_slice、008_array_nested、009_array_zero_size、010_array_loop、012_list_iter没有.expected.h；旧入口仍将其读成空字节并比较actual_header | 显式保零字节header判据；不是“不检查header”。9项+109现存H=118个H判决，加118个C=236个C族角色；其余角色不变，总1170 |

两个result的独立旧oracle SHA-256分别为05cc1caeac2bff87d2eee9914de06f0a11025170c51563839a2049cc475692d9、4a44dc15364204a80fe80e9039455cc1608281820fe2b24f1e5233ade6af1dd5。补doc时source/stdout原字节保持，不能从新doc反推或生成这两个原期望；vm_file_tests缓存取vm-result也必须实际取得新值。缺少任何一个原通道或将其当smoke，是输入视图错误，不能以两侧都PASS冒等价。

原身份需分消费者保持：VM/C/TS的FileTestCase.name为category/case；a2r/cookbook目录入口的A2rTestCase.name为suite_name/category/case（suite_name分别a2r/cookbook）。原native Rust/TS/C转译module参数取虚拟源stem，而a2r_rustc_real_compile_gate原来传完整case.name（如a2r/01_basics/001_hello）；二者不能为了共用视图强行归一。旧path字段可保为虚拟标签，禁止再被当成当前可读取的作者文件；结果命名、ledger键和trace均核原身份。

78个非N文档为VM10/a2r34（含C附录33个conformance）/C2/cookbook32；详附录E，TS0。它们可能属于另一包装、smoke或更深目录，不能一概叫“未注册”或停用，但不能因新版遍历所有.case.ad而加入原N1050。P的两份未来project文档也不能误入单源目录执行。反过来，N新增合法doc不得因没有旧.at而丢失：资格由本次冻结的消费者合同和文档声明确定，不能永远只准旧1050名单。

1. 四个discover函数以doc为唯一正常载荷/资格来源，可用dir+stem的惯例；不能继续要求原.at/.as/.expected.*存在，也不能缺doc时静默跳过原已登记义务。保1050旧入选项；VM cache另506项，其中10个无oracle或旧发现排除项不能因有doc被擅自升级为目录验证。显式空目录保持原no tests错误；已登记doc缺失须具名报缺，不冒合法空集或更少用例。
2. 优先从doc checks/虚拟source/注册family映射推导原发现资格；原smoke、空字符串、缺席与同目录不同消费者不能混淆。确有歧义时补有限、显式legacy-consumers元属性及author合同，T-02先以全部1050+额外10项证明无损，不新建与文档内容竞争的case作者索引。治理冻结清单仅用于独立守恒和缺失检测，不代载荷。
3. family+目录ID+check/消费通道作为义务键，不能只按doc case ID去重（不同族可能同ID）。cookbook同doc的VM面不能用于a2r判据，C/TS legacy文档即使暂用Rust产物节承载文本也不得误派a2r；消费端按原family和明确artifact逻辑路径取值。
4. VM保原stdout/result逐字节、error文件“存在性=期望任意错误”、.at/.as硬化通道及99_bootstrap库前置；不能改成无path内联运行绕过门，不能读取旧源码作bootstrap正文。原Auto标准库实现可作具名support，case自身不能列support。
5. a2r/cookbook保原源目录依赖、module名称、Rust bytes；C保原C/H缺席或空串与错误分支原规则；TS保bytes。原侧车文件名仅是虚拟标签/历史映射，不是当前读文件授权。wrong产物及trace/报告也须写显式scope，不能继续写仓内.wrong.*。
6. API可加文档视图字段或内部新描述符，原Test.run_*_dir方法/目录参数仍可用。全仓搜FileTestCase/A2rTestCase/A2cTestCase/A2tsTestCase构造和字段消费者，库外可用性先T-01列source兼容选择；不能靠留下旧生产执行fallback躲过接线。
7. discovery/inventory/trace以当前doc的载荷、行、oracle hash记身份。scripts/test-governance/input_inventory.py现重读旧source/oracle验证的正常分支一并修正；旧v1/历史命令显式保兼容，floor的subset守恒不弱化，不直接重写JSON去适配漏项。离线receipt须能重读actual，不信自报PASS。

8. 相邻VM缓存的506个cache ID和8个.as脚本资格以doc syntax/path及明确consumer合同确定，不能再用!at.is_file() && as.is_file()判断；doc-only中.as缺席时仍必须走原s2s lower。inventory_vm_cache_read_only的正常新协议分支也不能打开旧源来证明缓存身份。
   各consumer原lower位置保持：VM缓存包装原来显式调用lower_source，原生目录入口原来传虚拟源path到run_with_capture_and_path；不能把“脚本模式”简化成给所有消费者统一再lower一次。模式控制除最终值还须核实际路由/传入VM的前后源hash，防止未lower但该简单样本偶然同值。
9. cookbook VM的108个原包装绑定见附录B，名称和Windows77/31选择保持；6个多源目录必须T-01先用原同源Rust读取/真实收据冻结实际选中的stem/doc，不可凭排序、随意选第一doc、把一个包装扩成所有候选。当前候选旧源中12个没有doc，不代表它们已成为活跃义务；若T-01证明旧实选恰为缺doc候选，必须在本Plan补文档并对照，不准擅换到已有doc。正常选择不依赖旧.at是否在盘。
10. cookbook保原CWD相关语义及既有ignore条件，案例自身Auto文本不是data fixture。需要文件/图片等真实运行数据时逐项登记support、hash及路径，物化到scope中的等价运行目录，保持相对路径和恢复行为；本包不声称顺带解决所有原CWD并发债。源/oracle/wrong/proposal不得写作者树。

#### 正常治理与历史查询的版本边界（T-08）

拟加auto-case legacy-inventory --schema-version 2，schema=test-legacy-inventory/2；VM只读helper以AUTO_TEST_INVENTORY_SCHEMA=2输出test-vm-cache-inventory/2。当前发现、input_inventory、attach/verify_inputs、wrapper_bindings、固定组source_bindings/实际trace全部显式使用v2并核schema，读取doc重建source/oracle/空值/选择身份。原legacy-inventory无版本或显式1、原VM helper默认v1保为显式历史查询；它们不进入当前固定组/候选验收正常路径，不能在v2失败后回退v1。

v2给出doc路径/内容hash/section或载荷位置、虚拟源路径/syntax、消费者族/原键、每通道存在性/bytes/空header判据及原子actual指向。旧source_path等别名可作为历史标签参加原association/floor映射，但正常check_files必须从doc载荷核hash，不得去打开这个别名指向的.at/expected。旧契约JSON不删除重建；显式旧键→当前doc origin的有损/遗漏映射必须拒绝。原manifest构建若指向固定旧Git对象可显式查询旧v1；当前candidate必须v2。T-02落实字段/兼容机械细节，不再把“schema保持”理解为正常路径继续验旧文件。

### 5.3 P：多文件工程与真实编译门

两份新作者：crates/auto-lang/test/a2r/14_modules/005_multi_file/main.project.case.ad（id=legacy.a2r.project.14_modules.005_multi_file）和 crates/auto-lang/test/a2r/16_interop/021_path_dep/main.project.case.ad（id=legacy.a2r.project.16_interop.021_path_dep），均check-id=project；原Rust test ID保留。其源/历史golden全清单：

| 原路径 | 角色 | SHA-256 |
|---|---|---|
| crates/auto-lang/test/a2r/14_modules/005_multi_file/api/handlers.at | 源载荷 | 28bd451e1379924c38c7d54569d023eaf701676da0d89b0b8a43f282f24fb3cd |
| crates/auto-lang/test/a2r/14_modules/005_multi_file/api/handlers.expected.rs | 历史golden；不增加为新bytes断言 | d0ce353f0a4e8d17416056977a5f186b5e2e4c8f2dcfa5d162a175ac52effb29 |
| crates/auto-lang/test/a2r/14_modules/005_multi_file/api/mod.at | 源载荷 | 8b3199da2a8c9767110fa2c0b4bc86a405cb3213c47297b6f76d14e7a72f6a2a |
| crates/auto-lang/test/a2r/14_modules/005_multi_file/api/mod.expected.rs | 历史golden；不增加为新bytes断言 | 9a413bdcc3c333d1688f4a687e7f56905c944af60bea570007f69cd945c38f93 |
| crates/auto-lang/test/a2r/14_modules/005_multi_file/db.at | 源载荷 | 0fe7dbd6794922b53652157f9a3fd650217ffc73d4f4e7080fb2f9c207acc74a |
| crates/auto-lang/test/a2r/14_modules/005_multi_file/db.expected.rs | 历史golden；不增加为新bytes断言 | de221bb8b2895e1171f13199fdc0a92220b288ca819c9b90b0326bb60206cc3a |
| crates/auto-lang/test/a2r/14_modules/005_multi_file/main.at | 源载荷 | 8434a96fbfaa4096e4e4916d9f38c7925a12236013010688909c55a9f4dee61e |
| crates/auto-lang/test/a2r/14_modules/005_multi_file/main.expected.rs | 历史golden；不增加为新bytes断言 | ac16699b63a32eece74d7a2122df1e1df53c9c438fdc8e87bf5b3173498e50fc |
| crates/auto-lang/test/a2r/16_interop/021_path_dep/main.at | 源载荷 | ce5cf8cb6e2c99f247b4585c117c63db31f3b30842514d5bfe1400cf06cd1874 |

005含main.at、db.at、api/mod.at、api/handlers.at四Auto源；021含main.at一个源。四份005旧.expected.rs仅历史，不把未被原测试逐字节判断的golden升级为新必过oracle，也不能拿它们覆盖实际输出。源码/预期格式为Auto主源+四级附加虚拟文件、Rust产物节有限exists/contains断言（现author未支持exists/contains，T-05负责补齐并与bytes互斥/权限有限）。原19个显式断言如下：

| 断言ID | 工程 | 产物 | 原判据 → 文档判据 |
|---|---|---|---|
| P01 | 005_multi_file | main.rs | exists |
| P02 | 005_multi_file | db.rs | exists |
| P03 | 005_multi_file | api/mod.rs | exists |
| P04 | 005_multi_file | api/handlers.rs | exists |
| P05 | 005_multi_file | main.rs | contains: mod db; |
| P06 | 005_multi_file | main.rs | contains: mod api; |
| P07 | 005_multi_file | main.rs | contains: fn main() |
| P08 | 005_multi_file | api/mod.rs | contains: pub mod handlers; |
| P09 | 005_multi_file | db.rs | contains: struct Connection |
| P10 | 005_multi_file | db.rs | contains: fn connect() |
| P11 | 005_multi_file | api/handlers.rs | contains: use crate::db::*; |
| P12 | 005_multi_file | api/handlers.rs | contains: fn handle_request |
| P13 | 005_multi_file | Cargo.toml | exists |
| P14 | 005_multi_file | Cargo.toml | contains: [package] |
| P15 | 005_multi_file | Cargo.toml | contains: name = "app-005_multi_file" |
| P16 | 005_multi_file | Cargo.toml | contains: edition = "2021" |
| D01 | 021_path_dep | Cargo.toml | contains: auto_cosmic_dbus = { path =  |
| D02 | 021_path_dep | Cargo.toml | contains: serde = { version = "1", features = ["derive"] } |
| D03 | 021_path_dep | Cargo.toml | contains: log = "*" |

包含匹配仍大小写敏感/逐文本，不能全局trim或改成任意输出非空。D01只要求前缀，不捏造依赖实际路径值；取得Cargo.toml是原隐含存在要求，不冒新增第4条显式assert。多个contains归同一次原项目转译，不重复执行每个断言。

拟新增产物匹配属性仅为exists: true和非空字符串数组contains；未知键/空匹配集合/重复项按明确合同定位拒绝，不允许任意表达式。它们与同产物的全文bytes/lf金样形式互斥，旧decode的接受/拒绝和v2完整投影必须保持。每个声明期望单独记录实际文件存在性或具体substring判决，不能把16个期望压成仅“有输出”。005的完整目标产物节如下（Auto四文件部分见上方源清单；此语法待T-05接通）：

~~~~~markdown
### Rust
check-id: "project"

#### main.rs
exists: true
contains: ["mod db;", "mod api;", "fn main()"]

#### db.rs
exists: true
contains: ["struct Connection", "fn connect()"]

#### api/mod.rs
exists: true
contains: ["pub mod handlers;"]

#### api/handlers.rs
exists: true
contains: ["use crate::db::*;", "fn handle_request"]

#### Cargo.toml
exists: true
contains: ["[package]", "name = \"app-005_multi_file\"", "edition = \"2021\""]
~~~~~

物化必须保持entry=main.at、相对模块路径、项目目录basename（影响app-005_multi_file）及源目录依赖含义；先拒绝绝对/..逃逸、重复/冲突/reparse，再写受管目录，关闭时清理。原导入逻辑/库API不重写；测试不能继续从旧main.at进入，也不能物化主源后仍向旧api/db目录import。合法缺省、缺文件、模块循环按原生产诊断/失败保持。

编译门：现有250个a2r输入与N表a2r行完全同域，不另加250新样本。原compile_gate_known_broken.txt作为受控ledger支持资产可保（本轮36条非注释项，T-01冻结完整字节/每项及原适用性）；逐子项比较compiled/external-dep-skip/known-broken/unexpected-failure/obsolete-ledger。只接受原E0432/E0433/E0463外依赖分类，“新错误+外依赖错误”仍失败；原ledger项变绿必须报obsolete，不能消失。缺工具、空子集、失败spawn不是PASS。

计划新增显式固定组a2r-compile（未实现）：负责现有test身份的读收据验证，不纳daily/a2r/all自动选择；其250项准备编译是显式专项，保独立可调用入口和ledger义务，不挪ace-perf。原a2r_rustc_real_compile_gate保名称并标准确ignore理由，默认/普通组/裸cargo test不会执行；显式按下一段命名重放，必须通过实际nextest注册/选择证明。显式准备复用prepare_tools的源/工具/配置绑定设施，先调原转译器生成产品，逐项rustc --emit=metadata（debug/test、不release），记录输入/产物/工具hash、命令/exit/stderr/分类与原真实终态。验证阶段仅读匹配收据/实际产物；缺/陈旧收据报准备缺失，禁止lazy build。不得退化为预填250 PASS或只判receipt总exit。

r2固定组准入必须同时核无参数cargo t、a2r、all、根lib的广义选择和裸cargo test适用条件；仅从a2r表达式排除不够。新a2r-compile必须all=false且不在default/普通set中，expression精确选tests::a2r_tests::a2r_rustc_real_compile_gate，tools=[]/domain=legacy（当前fixed-tests禁止legacy组挂ACE tools）。显式命令固定为cargo t a2r-compile --ignored --test-name tests::a2r_tests::a2r_rustc_real_compile_gate；现driver要求--ignored搭配命名--test-name，不能把不带参数的cargo t a2r-compile当有效重放。准备在独立阶段，不能通过自动tool声明把250编译附回日常；缺显式准备/零选择必须失败。原子编译分类由原始进程exit/stderr/产物及ledger重算，假PASS或仅重算receipt局部hash仍须因原子证据不符被拒绝。

#### P相邻手动编译守卫（r2补齐，T-06）

三个函数都在crates/auto-lang/src/tests/a2r_tests.rs中，原ignore保持，**不是**把它们列作历史回放就允许继续复制.expected.rs。它们编译的是doc里的Rust golden产物；全语料250门编译的是从doc Auto源实际转译得到的新产品，两者原判据不同，不互相替代。

| M | 原完整test（前缀tests::a2r_tests::） | 原输入 → 当前doc载荷 | 必须保持的独立判决 |
|---|---|---|---|
| M01 | a2r_compile_smoke_str_param_borrow | N0530的str_param_borrow.expected.rs → 同doc的Rust产物 | rustc --crate-type=lib --emit=metadata真实成功 |
| M02 | a2r_foreign_shape_compile_run | N0716～N0719四份.expected.rs → 四份doc的Rust产物 | 同原stub/--extern真实编译与4个实际程序：成功exit且stdout分别contains generic_field_ok  42、trait_impl_ok  3、trait_object_ok、bare_thread_ok  41（空格按原字节） |
| M03 | a2r_capstone_term_subset_parity | N0720 term_session_subset.expected.rs → 同doc的Rust产物 | 产品与独立oracle.rs共用同一fake_core rlib，双方实际成功运行且stdout逐字节相等，不以同一产品冒参考程序 |

这6个doc产物先由显式准备物化/编译到scope，原ignored验证体只核新鲜收据并按原规则运行准备好的guest，禁止缺物料时临时rustc或复制旧expected。确切手动入口为cargo t a2r --ignored --test-name tests::a2r_tests::<M表函数>（逐名，非默认全ignored）。每个M原test名字/ignore及内部子输入保留，准备/执行分账。原固定系统临时目录auto_a2r_599_gate/auto_a2r_599_capstone等不得再共用，均改为本scope中本次调用的独占子域，禁止顺手删除其他会话产物。

允许具名保留的独立参考/工具fixture仅以下四份（路径前缀crates/auto-lang/test/a2r/25_foreign_types/）；它们不是Auto主源或普通.expected金样，保原字节/hash/owner=M02/M03，不能扩成通配豁免：

| fixture | 用途 | SHA-256 |
|---|---|---|
| 001_foreign_generic_field/stub_fake_term.rs | M02受控foreign stub | 0f94efcb3b2eff0a32cc5a531768c483bcbf60f0f98675d9c450fe425d414845 |
| 002_foreign_trait_impl/stub_foreign_shape.rs | M02原共享foreign_shape stub | 1fc7864d774c4bbaabb8ecfa1efb929dd19c45187b2cbf930172693504ddffdc |
| 005_term_session_subset/stub_fake_core.rs | M03产品与参考共用的mock库 | 7eef28a346d14f507df7b7960ccf0fca865944985aeb1bc2b5f982f113c6ac62 |
| 005_term_session_subset/oracle.rs | M03独立手写参考程序，保其stdout oracle角色 | 4353b51f6a9a75c1b591c53abf573f4c8298e1b44c4f9968a0f23f1c9a4c569a |

### 5.4 C：30生成闭集与33独立验证

原conformance_bootstrap循环30输入，全部包含在33固定验证中；附录C逐项列。040/042/043只保独立验证，不顺手扩大生成闭集。原普通生成器没有独立assert，其退出普通注册是删副作用入口，不是删除30个验证；注册对账应显示“生成器-1，33验证不减，新控制另列”。

拟新增单一入口auto-case conformance-proposal --root <repo> --out <scope> [--case <固定ID>]（尚未实现，T-02定共享函数接口、T-07交付CLI）。现proposal仅收artifact mismatches，不支持此30 VM result，不能直接声称现命令已覆盖或同时再建另一生成器。复用protected_roots/audit_output_dir，不接任意shell/任意后端命令；读doc源，按原VM真实输出生成候选，每例单独hash/diff/错误，生成不是验证绿。

必须显式out且位于允许scope；在执行VM前拒绝仓内作者根、旧expected、路径逃逸/reparse/别名目标，写失败非零。候选含旧doc/source/oracle身份、新观测/差异和生成入口，不能覆盖doc/expected或自动接受。正常cargo t corpus及33验证零作者写入；移除普通bootstrap注册及其旧read/write代码路径，文档/注释/run帮助同步。手动生成错误不再仅打印后exit0；30例其中一例执行错误必须在总结果中保该子项失败，原33独立验证始终按doc原oracle和Unicode trim判决。

r2明确同时处理正常包装的失败/旁路输出：run_conformance_test在mismatch时写.wrong.out，cookbook VM在mismatch时写wrong.out且GENERATE_EXPECTED可在缺oracle时写expected.out，a2r/TS/C等也有.wrong.*。这些都不能因仅核已有doc/expected的hash而漏掉新增文件；T-03/07/08将诊断输出放scope，正常成功与失败路径都对账作者树的文件集合/内容（包括原本不存在的wrong文件）。遗留GENERATE_EXPECTED不得授权普通包装写仓内；若保兼容请求，须清晰拒绝并指向显式候选流程，不得静默扩大conformance的30闭集。原无oracle smoke仍是smoke，不能把自动生成当独立期望。proposal部分成功允许保scope候选，但总报告需明确failed/NOT_RUN，不能把部分30项充完整成功。

### 5.5 S：完整11项对照与示例

全部case共用旧recipe ac-av/scheduler-source-run-v1、entry d_main、默认单run，原check ID av-scheduler保留；3负例不新增stdout/stderr判据。新文档名scheduler.document.case.ad，首例源正文如下，其余源按表逐字节内联：

| ID | case ID（新旧） | 旧check → 新check | 旧source（suite目录内） | 旧expect → 新expect | 原source SHA-256 |
|---|---|---|---|---|---|
| S01 | ace.m2.p968.message-handler | av-scheduler → 同名 | src/message_handler.at | success; exit=0; stdout="got one\n"; stderr="" | 5ad3f3a430ec5c978bdc5c2efc5bd5e3138024e249270234a1bc9c1ac97069d8 |
| S02 | ace.m2.p968.spawn-with-state | av-scheduler → 同名 | src/spawn_with_state.at | success; exit=0; stdout="mycounter: 15\n"; stderr="" | bb6c0112854b2b0fdcde8cb2d7b136738e7a8ebbb0476337bb17c4eb4654af68 |
| S03 | ace.m2.p968.mailbox-fifo | av-scheduler → 同名 | src/mailbox_fifo.at | success; exit=0; stdout="got two\ngot one\n"; stderr="" | 5dc347e7f7172c46b7128d8daf0d10f2bcd92cf7425f3ce9c32e986032ec0255 |
| S04 | ace.m2.p968.ask-reply | av-scheduler → 同名 | src/ask_reply.at | success; exit=0; stdout="reply:\n42\n"; stderr="" | ab529b296da44c394fb1e73c9bc9277431dec0414bc7b4a9e7469c0bac9b67bc |
| S05 | ace.m2.p968.go-join | av-scheduler → 同名 | src/go_join.at | success; exit=0; stdout="go:\n7\n"; stderr="" | 2b456b8bb94630c765994639cd98294e371c2590896b1e201bd71c14a3d0d31f |
| S06 | ace.m2.p968.backpressure | av-scheduler → 同名 | src/backpressure.at | success; exit=0; stdout="1\n2\n3\n"; stderr="" | 20dba76331127510412f885c2ea569ad01c20d49f40cefcb1ddde23d447de5fa |
| S07 | ace.m2.p968.stop-hook | av-scheduler → 同名 | src/stop_hook.at | success; exit=0; stdout="one\nstopped\n"; stderr="" | c220267f8346fadd7179243ee3a7f46613e1fc836a3ad17c7bee533b81546c58 |
| S08 | ace.m2.p968.virtual-timer | av-scheduler → 同名 | src/virtual_timer.at | success; exit=0; stdout="main\nlater\n"; stderr="" | 0e70e49ce1a5ff08f834e112ca11f9622121c7b4524c854cd9bf0cced30761a4 |
| S09 | ace.m2.p968.neg-unknown-task | av-scheduler → 同名 | src/neg_unknown_task.at | resolve/source.sched-task | 47e397b96f51ae63402f13ea486eac77e3baa0c79202a5f110ba3fa9e6dfa9e1 |
| S10 | ace.m2.p968.neg-malformed-task | av-scheduler → 同名 | src/neg_malformed_task.at | parse/source.unsupported | 29a3efe1c9d4d83ab822d591020c8df717e2283bde3b906e9dac0dc8918b991d |
| S11 | ace.m2.p968.neg-send-after-stop | av-scheduler → 同名 | src/neg_send_after_stop.at | run/lang.error | b0c6912da7e73371b464ec7b2e3da0e172946b27b89f222f988850195bc43f71 |

~~~~~markdown
## Case: message-handler
id: "ace.m2.p968.message-handler"
tags: ["ace-m2", "scheduler", "plan-968"]
links: ["PLAN-968"]

### Auto
path: "main.at"

~~~~auto
task Echo {
    fn start() ! {
    }
    on {
        1 -> {
            print("got one")
        }
        2 -> {
            print("got two")
        }
    }
}

fn main() {
    let h = Task.spawn("Echo", 16)
    h.send(1)
}
~~~~

### Probe av-scheduler
recipe: "ac-av/scheduler-source-run-v1"
entry: "d_main"
exit: 0

#### stdout
~~~~text
got one
~~~~

#### stderr
~~~~text
~~~~
~~~~~

示例是目标作者格式，recipe白名单及相应Probe双流规则是否已接通须T-04核定/实现；此处不是已执行证据。11项full v2与原件比较，不只看PASS；成功输出换行、空stderr、负例stage resolve/parse/run及code以原表为准，不能全部改成任意错误。suite ID/group ID保；新include实际选择11项，旧围栏留历史且不进入普通入口。

### 5.6 负控先行、入口切换与防再漏

各阶段：从固定旧Git对象+当前doc和真实消费者冻结 → 候选doc-only/共享视图 → 语义与真实actual/负控 → 切正常入口 → 正常入口重核。不能只改fixture路径或让对照器自生成期望。

最终不把组数写死76作为唯一准入：从test-groups.toml重新发现全部runner=case并导出每组origin.format，任何新旧围栏残留具名失败；新增组不在旧清单也必须报增量。覆盖N五根、四个公开shim、P两个原测试及编译子项、C正常/显式写入两入口；再搜普通读取、构造、调用和写入。历史/实现support/external分别记录路径+owner+用途，去掉通配的“source-dir都豁免”。

### 规范增量

| delta_id | add/modify/retire | target | before → after / 理由 | AC |
|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-test-case/document-migration-closure.md | 原过度收口已于本轮更正；未来按本包真实门补齐当前全部入口、准确数轴和保留边界 | AC-01/07/10 |
| SD-02 | modify | docs/specs/auto-test-case/architecture.md | 已更正未交付的TestRunner断言；实施后记共享doc视图、发现/公开shim/物化与显式准备 | AC-02/04/05 |
| SD-03 | modify | docs/specs/auto-test-case/document-cases.md | scheduler recipe及项目exists/contains有限匹配、源闭包/consumer资格实际合同；未知值定位拒绝 | AC-02/03/04 |
| SD-04 | modify | docs/specs/auto-test-case/governance-and-inventory.md | 旧floor/sidecar身份 → 当前doc源/oracle/选择资格、真实原子actual与历史显式分派 | AC-01/02/07 |
| SD-05 | modify | docs/specs/auto-test-case/module-selection.md | 普通a2r可触发全量rustc → 显式a2r-compile prepare/verify、不进日常，全量原义务保 | AC-05/08 |

本轮只更正Spec中已证伪的“全量已交”事实，不提前写新接口已交付；其余SD实施后由review/merge沉淀。1062归档前置更正、§9追加事后核查和KNOWN-DEBT登记属已授权簿记，保archived/r1/原勾选及原收据；不把本包计划当修复完成。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-A-PHASE | modify | docs/specs/auto-test-case/document-migration-closure.md、document-cases.md与module-selection.md（按实际交付） | 本包全量迁移一次关闭→A正确性正式切片/原载体真实版本与B完整doc-only/v2/proposal/编译范围分别标记，原oracle/条件不减 | 用户重冻A出口；不把规划当已迁移事实 | AC-A-CORRECTNESS、原AC-01..10 |

### 附录N：全部1050目录输入 → 现有doc与原oracle

N0001～N1050为 A 期冻结快照行号；**B 期增量 10 项**（vm `08_strings/017_sub_semantics`、`10_types/033_array_len_user_type`、`17_cffi/{001,002,003}`、`18_ffi/{047,055}`、`99_bootstrap/{066,067,068}`，逐项 doc/source 由 `plan1066_a_obligations.json` 的 `n_rows` 记 1060 行承载）已登记进冻结分母，故当前分母为 **1060 行/1171 角色**；family/id为规范义务键。doc是唯一目标作者，原source由同目录原stem.at（VM必要时.as）定位；hash前16位仅快速定位，固定基面Git对象和T-01完整SHA-256为正式依据。原oracle文件名只列角色，不许可当前读取。编译250就是其中family=a2r全部行。选择资格/缺doc拒绝/补元属性属于T-02，不以文件恰好在盘代接线完成。

| 行 | family/id | 目标既有doc（repo相对） | 原oracle文件角色 | 原source hash16 | doc hash16 |
|---|---|---|---|---|---|
| N0001 | vm / 01_basics/001_hello | crates/auto-lang/test/vm/01_basics/001_hello/hello.case.ad | hello.expected.out | 05264bcb8e5dd5ef | b1067d3daf908bb7 |
| N0002 | vm / 01_basics/002_arithmetic | crates/auto-lang/test/vm/01_basics/002_arithmetic/arithmetic.case.ad | arithmetic.expected.result | cb898749d76e51fd | f57f4c3cf24d98d2 |
| N0003 | vm / 01_basics/003_str_upper | crates/auto-lang/test/vm/01_basics/003_str_upper/str_upper.case.ad | str_upper.expected.out | bb7e9b36490c36dc | 4a354ee91aba7d9e |
| N0004 | vm / 01_basics/004_uint | crates/auto-lang/test/vm/01_basics/004_uint/uint.case.ad | uint.expected.result | ab713e0e9887c7c3 | ac812869170a144c |
| N0005 | vm / 01_basics/005_byte | crates/auto-lang/test/vm/01_basics/005_byte/byte.case.ad | byte.expected.result | 76950bc1ad4c3bc7 | dca091be97189bab |
| N0006 | vm / 01_basics/006_unary | crates/auto-lang/test/vm/01_basics/006_unary/unary.case.ad | unary.expected.result | c85ac4c786ef7977 | 561fcebd7cb2d16a |
| N0007 | vm / 01_basics/007_group | crates/auto-lang/test/vm/01_basics/007_group/group.case.ad | group.expected.result | 5a7e0aea55b602ab | 3869e5051b199eb2 |
| N0008 | vm / 01_basics/008_comp | crates/auto-lang/test/vm/01_basics/008_comp/comp.case.ad | comp.expected.result | eb7854518012998e | b14ffa30eb6782b0 |
| N0009 | vm / 01_basics/009_comp_false | crates/auto-lang/test/vm/01_basics/009_comp_false/comp_false.case.ad | comp_false.expected.result | 6084373450aaf5ab | 85d830cbe8ecf1ed |
| N0010 | vm / 01_basics/010_eq | crates/auto-lang/test/vm/01_basics/010_eq/eq.case.ad | eq.expected.result | ab019fe046c39bac | 8dbd61a8ecacdff4 |
| N0011 | vm / 01_basics/011_eq_false | crates/auto-lang/test/vm/01_basics/011_eq_false/eq_false.case.ad | eq_false.expected.result | 885bb242c8b6591a | 3ff286b41d2e9ebc |
| N0012 | vm / 02_bit_ops/001_binary_literal | crates/auto-lang/test/vm/02_bit_ops/001_binary_literal/binary_literal.case.ad | binary_literal.expected.out | 2561595db7addd63 | 14434b0f5ae6351e |
| N0013 | vm / 02_bit_ops/002_bitwise_ops | crates/auto-lang/test/vm/02_bit_ops/002_bitwise_ops/bitwise_ops.case.ad | bitwise_ops.expected.out | 1fbc84a88c766fdc | 0c79e892ba86b871 |
| N0014 | vm / 02_bit_ops/003_bit_scan | crates/auto-lang/test/vm/02_bit_ops/003_bit_scan/bit_scan.case.ad | bit_scan.expected.out | 1a912baf5dbc2bb7 | 577438529a52d311 |
| N0015 | vm / 02_bit_ops/004_not_flip | crates/auto-lang/test/vm/02_bit_ops/004_not_flip/not_flip.case.ad | not_flip.expected.out | b8c961ae3c524e55 | 85db491410d169f0 |
| N0016 | vm / 02_bit_ops/005_bitfield | crates/auto-lang/test/vm/02_bit_ops/005_bitfield/bitfield.case.ad | bitfield.expected.out | b6da23bcd1a6d5ca | 36acc9884786183d |
| N0017 | vm / 03_variables/001_var | crates/auto-lang/test/vm/03_variables/001_var/var.case.ad | var.expected.result | 54fa549661bf19ac | 8e186f6e911806dd |
| N0018 | vm / 03_variables/002_var_assign | crates/auto-lang/test/vm/03_variables/002_var_assign/var_assign.case.ad | var_assign.expected.result | 73b7853b53075b92 | 92f17eca0343d51d |
| N0019 | vm / 03_variables/003_var_mut | crates/auto-lang/test/vm/03_variables/003_var_mut/var_mut.case.ad | var_mut.expected.result | dbdae452b612c5b1 | 8a12acac876defa9 |
| N0020 | vm / 03_variables/004_var_if | crates/auto-lang/test/vm/03_variables/004_var_if/var_if.case.ad | var_if.expected.result | d56983cbf659db7d | daf458dea2de07b7 |
| N0021 | vm / 03_variables/005_let_binding | crates/auto-lang/test/vm/03_variables/005_let_binding/let_binding.case.ad | let_binding.expected.result | b03a4d22b93ac8fe | 50e9f7e88fc96ccc |
| N0022 | vm / 03_variables/006_let_asn_error | crates/auto-lang/test/vm/03_variables/006_let_asn_error/let_asn_error.case.ad | let_asn_error.expected.error | d47b5a561d4c99d4 | f1485af4deb1fc22 |
| N0023 | vm / 03_variables/007_var_reassignment | crates/auto-lang/test/vm/03_variables/007_var_reassignment/var_reassignment.case.ad | var_reassignment.expected.result | 64c2e6a87e1694b3 | 12626fbdb57f31c0 |
| N0024 | vm / 03_variables/008_simple_block | crates/auto-lang/test/vm/03_variables/008_simple_block/simple_block.case.ad | simple_block.expected.result | 1c67dcb56fc5f10c | b53129d1b2d5b7ae |
| N0025 | vm / 04_control_flow/001_if_true | crates/auto-lang/test/vm/04_control_flow/001_if_true/if_true.case.ad | if_true.expected.result | 587973e8e529a86c | 9566c27509d9d072 |
| N0026 | vm / 04_control_flow/002_if_false | crates/auto-lang/test/vm/04_control_flow/002_if_false/if_false.case.ad | if_false.expected.result | 6db98c22a4fce13e | c27e1dbb711ee950 |
| N0027 | vm / 04_control_flow/003_if_else_if | crates/auto-lang/test/vm/04_control_flow/003_if_else_if/if_else_if.case.ad | if_else_if.expected.result | 0817fdf4933e9e8c | 0b1b0292a734a8c8 |
| N0028 | vm / 04_control_flow/004_if_with_bool | crates/auto-lang/test/vm/04_control_flow/004_if_with_bool/if_with_bool.case.ad | if_with_bool.expected.out,if_with_bool.expected.result | 1a90bcf9e0ef67fb | e43b6438f8986dd2 |
| N0029 | vm / 04_control_flow/005_if_in_array | crates/auto-lang/test/vm/04_control_flow/005_if_in_array/if_in_array.case.ad | if_in_array.expected.result | 7042944e1e15e407 | 6d2e851076cbae82 |
| N0030 | vm / 04_control_flow/006_is_stmt | crates/auto-lang/test/vm/04_control_flow/006_is_stmt/is_stmt.case.ad | is_stmt.expected.out,is_stmt.expected.result | 4e6459b6dee0f7ed | 8989e8db3b45f17c |
| N0031 | vm / 04_control_flow/007_asn_upper | crates/auto-lang/test/vm/04_control_flow/007_asn_upper/asn_upper.case.ad | asn_upper.expected.result | f7c0b5e551f8c860 | af1649629fc3b9d1 |
| N0032 | vm / 04_control_flow/008_is_or_pattern | crates/auto-lang/test/vm/04_control_flow/008_is_or_pattern/is_or_pattern.case.ad | is_or_pattern.expected.out | 8cf1f651ed61b53e | 13ebbd3f9553a126 |
| N0033 | vm / 04_control_flow/009_is_or_groups | crates/auto-lang/test/vm/04_control_flow/009_is_or_groups/is_or_groups.case.ad | is_or_groups.expected.out | 0c518f98f0474242 | 590d9bbf0491050a |
| N0034 | vm / 04_control_flow/010_as_int_to_float | crates/auto-lang/test/vm/04_control_flow/010_as_int_to_float/as_int_to_float.case.ad | as_int_to_float.expected.out | ca5085f6fa4972c5 | 6576acb6112e88c4 |
| N0035 | vm / 04_control_flow/011_as_float_to_int | crates/auto-lang/test/vm/04_control_flow/011_as_float_to_int/as_float_to_int.case.ad | as_float_to_int.expected.out | 247c59ce0605489b | a155e923d36af183 |
| N0036 | vm / 04_control_flow/012_to_str | crates/auto-lang/test/vm/04_control_flow/012_to_str/to_str.case.ad | to_str.expected.out | 42e5752b52fb454e | 9fb0e8022ceb06d5 |
| N0037 | vm / 04_control_flow/013_to_int_from_str | crates/auto-lang/test/vm/04_control_flow/013_to_int_from_str/to_int_from_str.case.ad | to_int_from_str.expected.out | 14fc3182a29ec436 | 049339b2bdb75dbe |
| N0038 | vm / 05_loops/001_for_range | crates/auto-lang/test/vm/05_loops/001_for_range/for_range.case.ad | for_range.expected.result | 9bb94333e43913f6 | ea13174d4c0d9c99 |
| N0039 | vm / 05_loops/002_range_inclusive | crates/auto-lang/test/vm/05_loops/002_range_inclusive/range_inclusive.case.ad | range_inclusive.expected.result | 21e5610639a27e0d | 5d3baa3ccca7120d |
| N0040 | vm / 05_loops/003_range_literal | crates/auto-lang/test/vm/05_loops/003_range_literal/range_literal.case.ad | range_literal.expected.result | 5907d097038f69b0 | 5d934431de4c837f |
| N0041 | vm / 05_loops/004_for_each_object | crates/auto-lang/test/vm/05_loops/004_for_each_object/for_each_object.case.ad | for_each_object.expected.out | 65876fb4ddcae36e | abe3581a42fe0099 |
| N0042 | vm / 05_loops/005_loop_keyword | crates/auto-lang/test/vm/05_loops/005_loop_keyword/loop_keyword.case.ad | loop_keyword.expected.out | 9dc4371747b3458a | 8dbedc7058cd34d3 |
| N0043 | vm / 06_arrays/001_array_literal | crates/auto-lang/test/vm/06_arrays/001_array_literal/array_literal.case.ad | array_literal.expected.result | 67358ed8fd6cd4d8 | f8fbcb7f6903e066 |
| N0044 | vm / 06_arrays/002_array_index | crates/auto-lang/test/vm/06_arrays/002_array_index/array_index.case.ad | array_index.expected.result | f8ade450b6a26957 | eae4503618569072 |
| N0045 | vm / 06_arrays/003_array_update | crates/auto-lang/test/vm/06_arrays/003_array_update/array_update.case.ad | array_update.expected.result | 32eb7d7b60f74a0f | 7b7c2c760e1de4da |
| N0046 | vm / 06_arrays/004_array_of_objects | crates/auto-lang/test/vm/06_arrays/004_array_of_objects/array_of_objects.case.ad | array_of_objects.expected.result | 1fb3ffcf3df89e31 | 6786f9b2a4a3fc8d |
| N0047 | vm / 06_arrays/005_array_multiple_mutations | crates/auto-lang/test/vm/06_arrays/005_array_multiple_mutations/array_multiple_mutations.case.ad | array_multiple_mutations.expected.result | 8f91ab8405f23a83 | 5c54888ccee3c177 |
| N0048 | vm / 06_arrays/010_tuple | crates/auto-lang/test/vm/06_arrays/010_tuple/tuple.case.ad | tuple.expected.out | b19679d0adf96af3 | 71a8690be9112b06 |
| N0049 | vm / 07_objects/001_object_field | crates/auto-lang/test/vm/07_objects/001_object_field/object_field.case.ad | object_field.expected.result | 071b77a1634fb9e1 | 3d86ede697d73c40 |
| N0050 | vm / 07_objects/002_object_field_int_key | crates/auto-lang/test/vm/07_objects/002_object_field_int_key/object_field_int_key.case.ad | object_field_int_key.expected.result | f9fb088355a9c02f | 6bd2e7e9a7c89913 |
| N0051 | vm / 07_objects/003_object_field_bool_key | crates/auto-lang/test/vm/07_objects/003_object_field_bool_key/object_field_bool_key.case.ad | object_field_bool_key.expected.result | 59f1ee35113e3569 | 8815c02ccd645562 |
| N0052 | vm / 07_objects/004_obj_set | crates/auto-lang/test/vm/07_objects/004_obj_set/obj_set.case.ad | obj_set.expected.result | 22a33c25b284a8a8 | e5d7b6cebf07825d |
| N0053 | vm / 07_objects/005_nested_object | crates/auto-lang/test/vm/07_objects/005_nested_object/nested_object.case.ad | nested_object.expected.result | f20a0c750ff58361 | d692138be5a3bae8 |
| N0054 | vm / 07_objects/006_inline_map | crates/auto-lang/test/vm/07_objects/006_inline_map/inline_map.case.ad | inline_map.expected.out | ea2bc4c7d1b20979 | 7ca2cf9456cedfa2 |
| N0055 | vm / 07_objects/006_nested_object_y | crates/auto-lang/test/vm/07_objects/006_nested_object_y/nested_object_y.case.ad | nested_object_y.expected.result | e99996ec0bc66893 | afb64a4689fb9a58 |
| N0056 | vm / 07_objects/007_json | crates/auto-lang/test/vm/07_objects/007_json/json.case.ad | json.expected.result | 278785eed5fcc549 | d9967cba45faed9a |
| N0057 | vm / 07_objects/008_last_block_or_object | crates/auto-lang/test/vm/07_objects/008_last_block_or_object/last_block_or_object.case.ad | last_block_or_object.expected.result | dd28b1d7c7088519 | f0e3873bc219d6b6 |
| N0058 | vm / 07_objects/009_multiple_field_mutations | crates/auto-lang/test/vm/07_objects/009_multiple_field_mutations/multiple_field_mutations.case.ad | multiple_field_mutations.expected.result | 35581fd5d0df3230 | 91d9d9284965d7ca |
| N0059 | vm / 07_objects/010_map_bracket_write | crates/auto-lang/test/vm/07_objects/010_map_bracket_write/map_bracket_write.case.ad | map_bracket_write.expected.result | 9742083e77934ae0 | 328afff7d2293e19 |
| N0060 | vm / 08_strings/001_fstr | crates/auto-lang/test/vm/08_strings/001_fstr/fstr.case.ad | fstr.expected.result | 8a24e08febf2dfa0 | ee0d274f938ebf9f |
| N0061 | vm / 08_strings/002_fstr_expr | crates/auto-lang/test/vm/08_strings/002_fstr_expr/fstr_expr.case.ad | fstr_expr.expected.result | 719d1de07d2771df | 8db4b8a7bfd53e0d |
| N0062 | vm / 08_strings/003_str_index | crates/auto-lang/test/vm/08_strings/003_str_index/str_index.case.ad | str_index.expected.result | da6e9f91aca822a5 | 306e4c5af7a9341e |
| N0063 | vm / 08_strings/004_int_to_str | crates/auto-lang/test/vm/08_strings/004_int_to_str/int_to_str.case.ad | int_to_str.expected.result | ddce512d5fb47a8d | c6576db2817046b6 |
| N0064 | vm / 08_strings/005_str_import_basic | crates/auto-lang/test/vm/08_strings/005_str_import_basic/str_import_basic.case.ad | str_import_basic.expected.out | 8fadaf2810be65b6 | d143833844d97a96 |
| N0065 | vm / 08_strings/006_str_import_case | crates/auto-lang/test/vm/08_strings/006_str_import_case/str_import_case.case.ad | str_import_case.expected.out | de41b6b955f050c8 | e39d423796f64431 |
| N0066 | vm / 08_strings/007_str_import_search | crates/auto-lang/test/vm/08_strings/007_str_import_search/str_import_search.case.ad | str_import_search.expected.out | 92a427c0d2a122ce | c9bc3abe2e3435e3 |
| N0067 | vm / 08_strings/008_str_import_transform | crates/auto-lang/test/vm/08_strings/008_str_import_transform/str_import_transform.case.ad | str_import_transform.expected.out | f67d62f04767b00d | 9b839d443fae12e7 |
| N0068 | vm / 08_strings/009_char_at | crates/auto-lang/test/vm/08_strings/009_char_at/char_at.case.ad | char_at.expected.out | 9b7904e2badbf85f | a1ac42c4250c0509 |
| N0069 | vm / 08_strings/010_str_ext_import | crates/auto-lang/test/vm/08_strings/010_str_ext_import/str_ext_import.case.ad | str_ext_import.expected.out | 0fb0afb5468a47b3 | 88a2adda5258aca1 |
| N0070 | vm / 08_strings/011_vm_stub_panic | crates/auto-lang/test/vm/08_strings/011_vm_stub_panic/vm_stub_panic.case.ad | vm_stub_panic.expected.error | 50609109128c3b2a | 46e4ea0a9aea46af |
| N0071 | vm / 08_strings/012_str_eq | crates/auto-lang/test/vm/08_strings/012_str_eq/str_eq.case.ad | str_eq.expected.out | 770de10635637265 | cf20b48777bf0465 |
| N0072 | vm / 08_strings/013_str_param | crates/auto-lang/test/vm/08_strings/013_str_param/str_param.case.ad | str_param.expected.out | 0acbd96dd12b833e | de5353f21079db80 |
| N0073 | vm / 08_strings/014_raw_str | crates/auto-lang/test/vm/08_strings/014_raw_str/raw_str.case.ad | raw_str.expected.out | 1e23c4920eb3263f | 294a720811d8d85a |
| N0074 | vm / 08_strings/015_multi_fstr | crates/auto-lang/test/vm/08_strings/015_multi_fstr/multi_fstr.case.ad | multi_fstr.expected.out | f4d4c3b0ea8a3d51 | 953e13bb2943ac51 |
| N0075 | vm / 08_strings/016_heaprc_string_pool | crates/auto-lang/test/vm/08_strings/016_heaprc_string_pool/heaprc_string_pool.case.ad | heaprc_string_pool.expected.result | 71f73b6fdc37b94d | d1e5a9407ed08335 |
| N0076 | vm / 08_strings/018_loop_concat_churn | crates/auto-lang/test/vm/08_strings/018_loop_concat_churn/loop_concat_churn.case.ad | loop_concat_churn.expected.out | 97df2b3702111ae3 | 5b14869a83472747 |
| N0077 | vm / 09_functions/001_fn_simple | crates/auto-lang/test/vm/09_functions/001_fn_simple/fn_simple.case.ad | fn_simple.expected.result | 991542e3f89bec92 | 3003be71e06cd3de |
| N0078 | vm / 09_functions/002_fn_named_args | crates/auto-lang/test/vm/09_functions/002_fn_named_args/fn_named_args.case.ad | fn_named_args.expected.result | daf9ffa2c5b16dca | a44435fd2df153ef |
| N0079 | vm / 09_functions/003_fn_multiple | crates/auto-lang/test/vm/09_functions/003_fn_multiple/fn_multiple.case.ad | fn_multiple.expected.result | 5004f2655fce1493 | b55ad62b3e57e3b0 |
| N0080 | vm / 09_functions/004_fn_nested | crates/auto-lang/test/vm/09_functions/004_fn_nested/fn_nested.case.ad | fn_nested.expected.result | 2feea8e5be85dd9a | ef3e2742964f35f2 |
| N0081 | vm / 09_functions/005_fn_in_expr | crates/auto-lang/test/vm/09_functions/005_fn_in_expr/fn_in_expr.case.ad | fn_in_expr.expected.result | 84c136a1d8223f36 | c66d142a29e90f33 |
| N0082 | vm / 09_functions/006_fn_local_var | crates/auto-lang/test/vm/09_functions/006_fn_local_var/fn_local_var.case.ad | fn_local_var.expected.result | 48c93b775ba6e08f | 7bf94abbcb12673c |
| N0083 | vm / 09_functions/007_closure | crates/auto-lang/test/vm/09_functions/007_closure/closure.case.ad | closure.expected.result | 68970e9cd3036d61 | 726d3505317d6ac6 |
| N0084 | vm / 09_functions/008_closure_typed | crates/auto-lang/test/vm/09_functions/008_closure_typed/closure_typed.case.ad | closure_typed.expected.result | 53ecbdabda3781d0 | 440651b9337e80d3 |
| N0085 | vm / 09_functions/009_forward_decl | crates/auto-lang/test/vm/09_functions/009_forward_decl/forward_decl.case.ad | forward_decl.expected.result | 4c7d0aeccc7db0e0 | ed852c1e296ac424 |
| N0086 | vm / 09_functions/010_closure_hof_map | crates/auto-lang/test/vm/09_functions/010_closure_hof_map/closure_hof_map.case.ad | closure_hof_map.expected.out | 3eae97cefbe4c838 | 789d2b50777321ee |
| N0087 | vm / 09_functions/011_list_map_filter | crates/auto-lang/test/vm/09_functions/011_list_map_filter/list_map_filter.case.ad | list_map_filter.expected.out | 4d9863a960288a0c | 30f7af8320f37858 |
| N0088 | vm / 09_functions/012_list_reduce_find_any_all | crates/auto-lang/test/vm/09_functions/012_list_reduce_find_any_all/list_reduce_find_any_all.case.ad | list_reduce_find_any_all.expected.out | 6c49addc909fb55e | 46dcf76bb611ad3f |
| N0089 | vm / 09_functions/013_closure_capture_hof | crates/auto-lang/test/vm/09_functions/013_closure_capture_hof/closure_capture_hof.case.ad | closure_capture_hof.expected.out | 3b3eb1e9ecdbeb6a | 782d9cc0967a650c |
| N0090 | vm / 09_functions/014_list_for_each_edge | crates/auto-lang/test/vm/09_functions/014_list_for_each_edge/list_for_each_edge.case.ad | list_for_each_edge.expected.out | 5d22092bf3fed916 | 78077738f3ba4746 |
| N0091 | vm / 09_functions/015_list_chain_pipeline | crates/auto-lang/test/vm/09_functions/015_list_chain_pipeline/list_chain_pipeline.case.ad | list_chain_pipeline.expected.out | e3996eb2a86e80a9 | c4e8d64ddbae037d |
| N0092 | vm / 09_functions/016_enum_multi_destruct | crates/auto-lang/test/vm/09_functions/016_enum_multi_destruct/enum_multi_destruct.case.ad | enum_multi_destruct.expected.out | 8b88e9f4b8671df1 | d189db6629f2f600 |
| N0093 | vm / 09_functions/017_enum_named_construct | crates/auto-lang/test/vm/09_functions/017_enum_named_construct/enum_named_construct.case.ad | enum_named_construct.expected.out | dcf716f2cce66f3e | 181b3e9148fe6634 |
| N0094 | vm / 09_functions/018_enum_destruct_edge | crates/auto-lang/test/vm/09_functions/018_enum_destruct_edge/enum_destruct_edge.case.ad | enum_destruct_edge.expected.out | 21a1db3545de9769 | 1bc4ddc0b9b628f7 |
| N0095 | vm / 09_functions/019_enum_tuple_payload | crates/auto-lang/test/vm/09_functions/019_enum_tuple_payload/enum_tuple_payload.case.ad | enum_tuple_payload.expected.out | 98a495395098dbe1 | 2148971b03f2ce31 |
| N0096 | vm / 09_functions/020_fn_named_args_reordered | crates/auto-lang/test/vm/09_functions/020_fn_named_args_reordered/fn_named_args_reordered.case.ad | fn_named_args_reordered.expected.result | 644c77a659ca93a9 | 35817c956f16ea4c |
| N0097 | vm / 09_functions/021_fn_named_ctor_reordered | crates/auto-lang/test/vm/09_functions/021_fn_named_ctor_reordered/fn_named_ctor_reordered.case.ad | fn_named_ctor_reordered.expected.result | 9f2574a0d11286f8 | ac248b4d972e2f87 |
| N0098 | vm / 10_types/001_type_compose | crates/auto-lang/test/vm/10_types/001_type_compose/type_compose.case.ad | type_compose.expected.error | 3cb68ccab4bbeb24 | 7625d00147b5ff3f |
| N0099 | vm / 10_types/002_int_enum | crates/auto-lang/test/vm/10_types/002_int_enum/int_enum.case.ad | int_enum.expected.result | 94474c90f2272086 | 171e8b0da5f12388 |
| N0100 | vm / 10_types/003_generic_instantiation | crates/auto-lang/test/vm/10_types/003_generic_instantiation/generic_instantiation.case.ad | generic_instantiation.expected.error | a100945db9dba841 | e26c5fa2505fd078 |
| N0101 | vm / 10_types/004_generic_field_x | crates/auto-lang/test/vm/10_types/004_generic_field_x/generic_field_x.case.ad | generic_field_x.expected.error | 6fdb60b1c9ad8ebe | 040fde5016c655e0 |
| N0102 | vm / 10_types/005_generic_field_y | crates/auto-lang/test/vm/10_types/005_generic_field_y/generic_field_y.case.ad | generic_field_y.expected.error | 3667c4a78d128814 | 4fc3505f95ed8ca8 |
| N0103 | vm / 10_types/006_field_addition | crates/auto-lang/test/vm/10_types/006_field_addition/field_addition.case.ad | field_addition.expected.result | e09551caef1ead4f | 6cf6890d8051f6a9 |
| N0104 | vm / 10_types/007_type_instance_prop | crates/auto-lang/test/vm/10_types/007_type_instance_prop/type_instance_prop.case.ad | type_instance_prop.expected.result | 0c2c40d392bf8f5c | 93600e65130010d6 |
| N0105 | vm / 10_types/008_nested_type_instance | crates/auto-lang/test/vm/10_types/008_nested_type_instance/nested_type_instance.case.ad | nested_type_instance.expected.result | c706b646d1536ad7 | 18ef5cbc3cec56e2 |
| N0106 | vm / 10_types/009_access_fields_in_method | crates/auto-lang/test/vm/10_types/009_access_fields_in_method/access_fields_in_method.case.ad | access_fields_in_method.expected.error | 3187cf371d71bf5e | 14194d03bc3aa4fa |
| N0107 | vm / 10_types/010_ext_method | crates/auto-lang/test/vm/10_types/010_ext_method/ext_method.case.ad | ext_method.expected.out | 3389140d9eb41be9 | e4337304a1b55d7d |
| N0108 | vm / 10_types/011_enum_is_match | crates/auto-lang/test/vm/10_types/011_enum_is_match/enum_is_match.case.ad | enum_is_match.expected.out | e2fa247e2846d812 | 2487dbf027b1461d |
| N0109 | vm / 10_types/011_enum_multi_field | crates/auto-lang/test/vm/10_types/011_enum_multi_field/enum_multi_field.case.ad | enum_multi_field.expected.out | 82d2e153c812ddc8 | e20187838d057930 |
| N0110 | vm / 10_types/012_enum_dot_match | crates/auto-lang/test/vm/10_types/012_enum_dot_match/enum_dot_match.case.ad | enum_dot_match.expected.out | e2d36c059a865009 | 7f9fcda462e56fbd |
| N0111 | vm / 10_types/013_fn_return_obj | crates/auto-lang/test/vm/10_types/013_fn_return_obj/fn_return_obj.case.ad | fn_return_obj.expected.out | c080671a8b62c57a | 4129bd4237d60e62 |
| N0112 | vm / 10_types/034_struct_str_field | crates/auto-lang/test/vm/10_types/034_struct_str_field/struct_str_field.case.ad | struct_str_field.expected.out | 8f5315be9abc1cb3 | bcc0a2a6136de0e7 |
| N0113 | vm / 10_types/035_struct_from_fn | crates/auto-lang/test/vm/10_types/035_struct_from_fn/struct_from_fn.case.ad | struct_from_fn.expected.out | 66af79d5edcf0ab5 | 7da9e0712d27c296 |
| N0114 | vm / 10_types/036_struct_in_array | crates/auto-lang/test/vm/10_types/036_struct_in_array/struct_in_array.case.ad | struct_in_array.expected.out | 17dc78ad57203554 | bc847bd2ee879c36 |
| N0115 | vm / 11_compound_ops/001_add_eq | crates/auto-lang/test/vm/11_compound_ops/001_add_eq/add_eq.case.ad | add_eq.expected.result | 3bab699cc03b9ddd | f5db71c5d3fbde90 |
| N0116 | vm / 11_compound_ops/002_sub_eq | crates/auto-lang/test/vm/11_compound_ops/002_sub_eq/sub_eq.case.ad | sub_eq.expected.result | 86865ec19fab9d0b | bf54d4f6664e50d1 |
| N0117 | vm / 11_compound_ops/003_mul_eq | crates/auto-lang/test/vm/11_compound_ops/003_mul_eq/mul_eq.case.ad | mul_eq.expected.result | e65ba4f7dbbbd3f9 | b1d478dccdfa4171 |
| N0118 | vm / 11_compound_ops/004_div_eq | crates/auto-lang/test/vm/11_compound_ops/004_div_eq/div_eq.case.ad | div_eq.expected.result | edc5f742dc1bc1a8 | 4518c747947444df |
| N0119 | vm / 11_compound_ops/005_chained | crates/auto-lang/test/vm/11_compound_ops/005_chained/chained.case.ad | chained.expected.result | 53788e3072862ee4 | 42c1c981f84f6349 |
| N0120 | vm / 11_compound_ops/006_div_eq_oneline | crates/auto-lang/test/vm/11_compound_ops/006_div_eq_oneline/div_eq_oneline.case.ad | div_eq_oneline.expected.result | 33c6daab47c0f2f7 | 398950261125bda3 |
| N0121 | vm / 12_type_coercion/001_int_plus_float | crates/auto-lang/test/vm/12_type_coercion/001_int_plus_float/int_plus_float.case.ad | int_plus_float.expected.result | bd4094ac997e8159 | 60cfbfd6ff4fc42a |
| N0122 | vm / 12_type_coercion/002_float_plus_int | crates/auto-lang/test/vm/12_type_coercion/002_float_plus_int/float_plus_int.case.ad | float_plus_int.expected.result | dc1bfad4be07b8b7 | a39b30e189f2b57f |
| N0123 | vm / 12_type_coercion/003_int_times_float | crates/auto-lang/test/vm/12_type_coercion/003_int_times_float/int_times_float.case.ad | int_times_float.expected.result | 3d4a75d656aa3f46 | da21af94fabbb32e |
| N0124 | vm / 12_type_coercion/004_float_times_int | crates/auto-lang/test/vm/12_type_coercion/004_float_times_int/float_times_int.case.ad | float_times_int.expected.result | 6fa9fc8458e6e288 | f738a3069e7a0b64 |
| N0125 | vm / 12_type_coercion/005_complex | crates/auto-lang/test/vm/12_type_coercion/005_complex/complex.case.ad | complex.expected.result | 2cf866b581b796f3 | 191ae775f98a1d5a |
| N0126 | vm / 12_type_coercion/006_with_variable | crates/auto-lang/test/vm/12_type_coercion/006_with_variable/with_variable.case.ad | with_variable.expected.result | 0a167035b36c699c | 5a0985e9bdd2df19 |
| N0127 | vm / 13_collections/001_hashmap_new | crates/auto-lang/test/vm/13_collections/001_hashmap_new/hashmap_new.case.ad | hashmap_new.expected.result | 3ecd98999966872c | 2fb348f9211bbef3 |
| N0128 | vm / 13_collections/002_hashmap_insert_str | crates/auto-lang/test/vm/13_collections/002_hashmap_insert_str/hashmap_insert_str.case.ad | hashmap_insert_str.expected.result | d0ad2791547695ef | 5e4c74f8810d45fd |
| N0129 | vm / 13_collections/003_hashmap_insert_int | crates/auto-lang/test/vm/13_collections/003_hashmap_insert_int/hashmap_insert_int.case.ad | hashmap_insert_int.expected.result | 17ee71284fd3e5cb | c261c5392be09297 |
| N0130 | vm / 13_collections/004_hashmap_contains | crates/auto-lang/test/vm/13_collections/004_hashmap_contains/hashmap_contains.case.ad | hashmap_contains.expected.result | 4efbf7abbba25f17 | d51a30d2fc6cdb8f |
| N0131 | vm / 13_collections/005_hashmap_size | crates/auto-lang/test/vm/13_collections/005_hashmap_size/hashmap_size.case.ad | hashmap_size.expected.result | d81d26a9cab4b728 | 6ea5a43fd1249891 |
| N0132 | vm / 13_collections/006_hashmap_remove | crates/auto-lang/test/vm/13_collections/006_hashmap_remove/hashmap_remove.case.ad | hashmap_remove.expected.error | a6684173134e4ce5 | 3d2f3736e6781ab7 |
| N0133 | vm / 13_collections/007_hashmap_clear | crates/auto-lang/test/vm/13_collections/007_hashmap_clear/hashmap_clear.case.ad | hashmap_clear.expected.result | 439098d9a2364264 | 606c690ff34197db |
| N0134 | vm / 13_collections/008_hashset_new | crates/auto-lang/test/vm/13_collections/008_hashset_new/hashset_new.case.ad | hashset_new.expected.result | 5356ef9f8ed2e215 | 1e9a26d239b4d81c |
| N0135 | vm / 13_collections/009_hashset_insert | crates/auto-lang/test/vm/13_collections/009_hashset_insert/hashset_insert.case.ad | hashset_insert.expected.out | 9ced2e4e936214b5 | e8fac6c516221fad |
| N0136 | vm / 13_collections/010_hashset_duplicate | crates/auto-lang/test/vm/13_collections/010_hashset_duplicate/hashset_duplicate.case.ad | hashset_duplicate.expected.result | 255c9fa19e3ac2f9 | 62d58ed63331cc81 |
| N0137 | vm / 13_collections/011_hashset_remove | crates/auto-lang/test/vm/13_collections/011_hashset_remove/hashset_remove.case.ad | hashset_remove.expected.error | 87305a88e5169df7 | 44982976c3d67e9f |
| N0138 | vm / 13_collections/012_hashset_size | crates/auto-lang/test/vm/13_collections/012_hashset_size/hashset_size.case.ad | hashset_size.expected.result | 29546dc2d1f3b990 | bf580b8602a3d340 |
| N0139 | vm / 13_collections/013_hashset_clear | crates/auto-lang/test/vm/13_collections/013_hashset_clear/hashset_clear.case.ad | hashset_clear.expected.result | 158b644f303fc445 | 26f425ff3ee9aaf4 |
| N0140 | vm / 13_collections/014_sb_new | crates/auto-lang/test/vm/13_collections/014_sb_new/sb_new.case.ad | sb_new.expected.result | b7fcaa59ace6295f | a04acce54ca300bb |
| N0141 | vm / 13_collections/015_sb_append | crates/auto-lang/test/vm/13_collections/015_sb_append/sb_append.case.ad | sb_append.expected.result | 89a1fdbf2985444d | 11c7c1989d453bac |
| N0142 | vm / 13_collections/016_sb_append_char | crates/auto-lang/test/vm/13_collections/016_sb_append_char/sb_append_char.case.ad | sb_append_char.expected.result | 7908b05a25462f34 | 14dba7985a504868 |
| N0143 | vm / 13_collections/017_sb_append_int | crates/auto-lang/test/vm/13_collections/017_sb_append_int/sb_append_int.case.ad | sb_append_int.expected.result | acd0c7253c93373c | 3c8b9a07b775af0f |
| N0144 | vm / 13_collections/018_sb_len | crates/auto-lang/test/vm/13_collections/018_sb_len/sb_len.case.ad | sb_len.expected.result | 993dfe36ddb86aff | 4dcc713cdbde52ed |
| N0145 | vm / 13_collections/019_sb_clear | crates/auto-lang/test/vm/13_collections/019_sb_clear/sb_clear.case.ad | sb_clear.expected.result | 3e6d54901e0ad31f | 369229cb8b507f64 |
| N0146 | vm / 13_collections/020_list_new | crates/auto-lang/test/vm/13_collections/020_list_new/list_new.case.ad | list_new.expected.result | 77ead8e434af99b1 | e013322f7db6ecd4 |
| N0147 | vm / 13_collections/021_list_push_pop | crates/auto-lang/test/vm/13_collections/021_list_push_pop/list_push_pop.case.ad | list_push_pop.expected.result | 8031d9b0630cf28d | 1cc89e7a64945cba |
| N0148 | vm / 13_collections/022_list_push_pop_multi | crates/auto-lang/test/vm/13_collections/022_list_push_pop_multi/list_push_pop_multi.case.ad | list_push_pop_multi.expected.result | 89a2f2f2ffec4f6e | 08b331f584257d40 |
| N0149 | vm / 13_collections/023_list_len | crates/auto-lang/test/vm/13_collections/023_list_len/list_len.case.ad | list_len.expected.result | 2a2e36c5ce7f1c79 | e5f81108b5573a60 |
| N0150 | vm / 13_collections/024_list_is_empty | crates/auto-lang/test/vm/13_collections/024_list_is_empty/list_is_empty.case.ad | list_is_empty.expected.result | db7c2d395682e41a | ab9faff961324282 |
| N0151 | vm / 13_collections/025_list_clear | crates/auto-lang/test/vm/13_collections/025_list_clear/list_clear.case.ad | list_clear.expected.result | d321263f19400a4a | af54c501689c0a89 |
| N0152 | vm / 13_collections/026_list_get_set | crates/auto-lang/test/vm/13_collections/026_list_get_set/list_get_set.case.ad | list_get_set.expected.result | 6ed027bc7583747c | ed4319ef77ee98b2 |
| N0153 | vm / 13_collections/027_list_insert_remove | crates/auto-lang/test/vm/13_collections/027_list_insert_remove/list_insert_remove.case.ad | list_insert_remove.expected.result | 8a97cca10ba19efd | 9ce55a6fbb0228fe |
| N0154 | vm / 13_collections/028_list_reserve | crates/auto-lang/test/vm/13_collections/028_list_reserve/list_reserve.case.ad | list_reserve.expected.result | c155180bc9d10bca | 9587bce5e70a40bc |
| N0155 | vm / 13_collections/029_list_comprehensive | crates/auto-lang/test/vm/13_collections/029_list_comprehensive/list_comprehensive.case.ad | list_comprehensive.expected.result | 48e29402e76af05e | 5e238ddfc4dd5a0f |
| N0156 | vm / 13_collections/030_list_multi_ops | crates/auto-lang/test/vm/13_collections/030_list_multi_ops/list_multi_ops.case.ad | list_multi_ops.expected.result | 1896334e29bb775d | 0d4eda23dccff4eb |
| N0157 | vm / 13_collections/031_list_index | crates/auto-lang/test/vm/13_collections/031_list_index/list_index.case.ad | list_index.expected.result | 44c5abcff3f7b611 | b3c5e2bf18e65520 |
| N0158 | vm / 13_collections/032_list_varargs | crates/auto-lang/test/vm/13_collections/032_list_varargs/list_varargs.case.ad | list_varargs.expected.result | 19bb68922ac0258e | 999e61ed0adec676 |
| N0159 | vm / 13_collections/033_list_varargs_empty | crates/auto-lang/test/vm/13_collections/033_list_varargs_empty/list_varargs_empty.case.ad | list_varargs_empty.expected.result | 9376addbded5b258 | ee8c2cd298f35230 |
| N0160 | vm / 13_collections/034_list_for_iteration | crates/auto-lang/test/vm/13_collections/034_list_for_iteration/list_for_iteration.case.ad | list_for_iteration.expected.result | fde889dd21778071 | 464f7762e5647565 |
| N0161 | vm / 13_collections/035_list_for_empty | crates/auto-lang/test/vm/13_collections/035_list_for_empty/list_for_empty.case.ad | list_for_empty.expected.result | a978b2da89c9406a | 5452b55e4eca8d6c |
| N0162 | vm / 14_borrow/001_view_basic | crates/auto-lang/test/vm/14_borrow/001_view_basic/view_basic.case.ad | view_basic.expected.result | c89a4727b95bcc85 | 2b812086b5948d28 |
| N0163 | vm / 14_borrow/002_view_multiple | crates/auto-lang/test/vm/14_borrow/002_view_multiple/view_multiple.case.ad | view_multiple.expected.result | 2bd5beafe3baf3ff | 0bf632a7977ae86c |
| N0164 | vm / 14_borrow/003_mut_basic | crates/auto-lang/test/vm/14_borrow/003_mut_basic/mut_basic.case.ad | mut_basic.expected.result | 716fb9d474ef4ae8 | 14e7e315d229e799 |
| N0165 | vm / 14_borrow/004_move_basic | crates/auto-lang/test/vm/14_borrow/004_move_basic/move_basic.case.ad | move_basic.expected.result | 7f90c95a824501ac | f3de7d765126aad2 |
| N0166 | vm / 14_borrow/005_view_preserves | crates/auto-lang/test/vm/14_borrow/005_view_preserves/view_preserves.case.ad | view_preserves.expected.result | 2348a49288be8f92 | 61a984a48d219f7f |
| N0167 | vm / 14_borrow/006_nested_view | crates/auto-lang/test/vm/14_borrow/006_nested_view/nested_view.case.ad | nested_view.expected.result | 119c440dc0fb759a | 0f3ca15b61050ebf |
| N0168 | vm / 14_borrow/007_borrow_arithmetic | crates/auto-lang/test/vm/14_borrow/007_borrow_arithmetic/borrow_arithmetic.case.ad | borrow_arithmetic.expected.result | a7361c921fda8053 | 75587cbd3d375018 |
| N0169 | vm / 14_borrow/008_view_in_array | crates/auto-lang/test/vm/14_borrow/008_view_in_array/view_in_array.case.ad | view_in_array.expected.result | f1fbf69e3ac4da0e | aa283e3a76141411 |
| N0170 | vm / 14_borrow/009_view_in_expr | crates/auto-lang/test/vm/14_borrow/009_view_in_expr/view_in_expr.case.ad | view_in_expr.expected.result | 2b836f43d79f6f21 | d0e55c8a7cf3f4c0 |
| N0171 | vm / 14_borrow/010_borrow_diff_types | crates/auto-lang/test/vm/14_borrow/010_borrow_diff_types/borrow_diff_types.case.ad | borrow_diff_types.expected.result | d74a3202780ffa21 | 3a51802b81578cd9 |
| N0172 | vm / 14_borrow/011_move_chaining | crates/auto-lang/test/vm/14_borrow/011_move_chaining/move_chaining.case.ad | move_chaining.expected.result | 3180504bb08be48c | a421b05cf2b2b615 |
| N0173 | vm / 14_borrow/012_str_sliceview | crates/auto-lang/test/vm/14_borrow/012_str_sliceview/str_sliceview.case.ad | str_sliceview.expected.result | 19fc55c041bfaae6 | 7112bb730499aa10 |
| N0174 | vm / 14_borrow/013_str_slice_multi | crates/auto-lang/test/vm/14_borrow/013_str_slice_multi/str_slice_multi.case.ad | str_slice_multi.expected.result | 0693b44628a36a2e | 40aa35e1775f3668 |
| N0175 | vm / 14_borrow/014_str_slice_nested | crates/auto-lang/test/vm/14_borrow/014_str_slice_nested/str_slice_nested.case.ad | str_slice_nested.expected.result | c89a4727b95bcc85 | 173053d74174061a |
| N0176 | vm / 14_borrow/015_str_slice_in_array | crates/auto-lang/test/vm/14_borrow/015_str_slice_in_array/str_slice_in_array.case.ad | str_slice_in_array.expected.result | aadd8a5a91f18586 | b35e1ac11febe1d0 |
| N0177 | vm / 14_borrow/016_str_slice_take | crates/auto-lang/test/vm/14_borrow/016_str_slice_take/str_slice_take.case.ad | str_slice_take.expected.result | b92c60ae61587373 | 7cd7275d8cbdb381 |
| N0178 | vm / 14_borrow/017_str_slice_mixed | crates/auto-lang/test/vm/14_borrow/017_str_slice_mixed/str_slice_mixed.case.ad | str_slice_mixed.expected.result | b610eb5c0dbe5951 | 055bfe0d527ee872 |
| N0179 | vm / 14_borrow/018_str_slice_preserves | crates/auto-lang/test/vm/14_borrow/018_str_slice_preserves/str_slice_preserves.case.ad | str_slice_preserves.expected.result | 28c32acfa62342b5 | 638cbe693e3845f0 |
| N0180 | vm / 15_nested_mutation/001_object_field | crates/auto-lang/test/vm/15_nested_mutation/001_object_field/object_field.case.ad | object_field.expected.result | 52f4537d1bbcbccc | cdb404353c0df029 |
| N0181 | vm / 15_nested_mutation/002_array_element | crates/auto-lang/test/vm/15_nested_mutation/002_array_element/array_element.case.ad | array_element.expected.result | a01666b59f3a0bdd | faaeee453f73f5e9 |
| N0182 | vm / 15_nested_mutation/003_multiple_fields | crates/auto-lang/test/vm/15_nested_mutation/003_multiple_fields/multiple_fields.case.ad | multiple_fields.expected.result | 35581fd5d0df3230 | 7c56319e6da57f0d |
| N0183 | vm / 15_nested_mutation/004_multiple_array | crates/auto-lang/test/vm/15_nested_mutation/004_multiple_array/multiple_array.case.ad | multiple_array.expected.result | e7899e5b702f941d | cdd18ea82a90daf3 |
| N0184 | vm / 15_nested_mutation/005_type_field | crates/auto-lang/test/vm/15_nested_mutation/005_type_field/type_field.case.ad | type_field.expected.result | 042ace9df4347a48 | 65672a857e1f3ed2 |
| N0185 | vm / 15_nested_mutation/006_nested_object | crates/auto-lang/test/vm/15_nested_mutation/006_nested_object/nested_object.case.ad | nested_object.expected.result | 40fb4e8088a53bc4 | 468a4055ae3a398c |
| N0186 | vm / 15_nested_mutation/007_array_of_obj_field | crates/auto-lang/test/vm/15_nested_mutation/007_array_of_obj_field/array_of_obj_field.case.ad | array_of_obj_field.expected.result | 0ee74e426c6b1f48 | c965b1583d7c0af4 |
| N0187 | vm / 15_nested_mutation/008_obj_array_element | crates/auto-lang/test/vm/15_nested_mutation/008_obj_array_element/obj_array_element.case.ad | obj_array_element.expected.result | df450aff75484692 | 83713f0117fe027f |
| N0188 | vm / 15_nested_mutation/009_nested_array | crates/auto-lang/test/vm/15_nested_mutation/009_nested_array/nested_array.case.ad | nested_array.expected.result | 6f5766ed99a23091 | 4d4ae8610a4938b3 |
| N0189 | vm / 15_nested_mutation/010_type_nested_field | crates/auto-lang/test/vm/15_nested_mutation/010_type_nested_field/type_nested_field.case.ad | type_nested_field.expected.result | 09d7d00e902dc1af | 2f5e8c15a1dd0afd |
| N0190 | vm / 15_nested_mutation/011_three_level | crates/auto-lang/test/vm/15_nested_mutation/011_three_level/three_level.case.ad | three_level.expected.result | 75ef4522c04aaf23 | 8773c872db3885ed |
| N0191 | vm / 15_nested_mutation/012_deep_array_obj | crates/auto-lang/test/vm/15_nested_mutation/012_deep_array_obj/deep_array_obj.case.ad | deep_array_obj.expected.result | 2ca845bf60093c70 | 81d1b4019ecf4991 |
| N0192 | vm / 15_nested_mutation/013_structure_preserve | crates/auto-lang/test/vm/15_nested_mutation/013_structure_preserve/structure_preserve.case.ad | structure_preserve.expected.result | 9f00720ad4f4736f | 158c00e75e61a8a0 |
| N0193 | vm / 15_nested_mutation/014_out_of_bounds_error | crates/auto-lang/test/vm/15_nested_mutation/014_out_of_bounds_error/out_of_bounds_error.case.ad | out_of_bounds_error.expected.error | a5be4f00e56872ea | 63a608b88763b55b |
| N0194 | vm / 15_nested_mutation/015_self_list_push | crates/auto-lang/test/vm/15_nested_mutation/015_self_list_push/self_list_push.case.ad | self_list_push.expected.out | 8a63fa51485c94d5 | 184ebeaca0a66a03 |
| N0195 | vm / 16_option_result/001_option_type | crates/auto-lang/test/vm/16_option_result/001_option_type/option_type.case.ad | option_type.expected.result | 64477e8f6cc8982b | 2150495a2f68c98b |
| N0196 | vm / 16_option_result/002_result_type | crates/auto-lang/test/vm/16_option_result/002_result_type/result_type.case.ad | result_type.expected.result | f6960e8386a84954 | 4973d0f185f9eed6 |
| N0197 | vm / 16_option_result/003_none_literal | crates/auto-lang/test/vm/16_option_result/003_none_literal/none_literal.case.ad | none_literal.expected.result | 563a1c697c4481f2 | fae8cdc15ea14445 |
| N0198 | vm / 16_option_result/004_some_ctor | crates/auto-lang/test/vm/16_option_result/004_some_ctor/some_ctor.case.ad | some_ctor.expected.result | 785c3779e31c3559 | 6ac88e8fa0e0d6f8 |
| N0199 | vm / 16_option_result/005_ok_ctor | crates/auto-lang/test/vm/16_option_result/005_ok_ctor/ok_ctor.case.ad | ok_ctor.expected.result | 23a89d5539b408a2 | f9098b5a2b75cc4e |
| N0200 | vm / 16_option_result/006_err_ctor | crates/auto-lang/test/vm/16_option_result/006_err_ctor/err_ctor.case.ad | err_ctor.expected.result | 9d6b03d2854d00bd | 7c5e05b3519f7244 |
| N0201 | vm / 16_option_result/007_propagate_some | crates/auto-lang/test/vm/16_option_result/007_propagate_some/propagate_some.case.ad | propagate_some.expected.result | 6cd6c7ffdb58ecc5 | 794567daa6c2b728 |
| N0202 | vm / 16_option_result/008_propagate_none | crates/auto-lang/test/vm/16_option_result/008_propagate_none/propagate_none.case.ad | propagate_none.expected.result | e16a92d4de11ca21 | 88e6af75fc703a05 |
| N0203 | vm / 16_option_result/009_propagate_ok | crates/auto-lang/test/vm/16_option_result/009_propagate_ok/propagate_ok.case.ad | propagate_ok.expected.result | 9042676035247c8f | f6dae942d85e37ce |
| N0204 | vm / 16_option_result/010_propagate_err | crates/auto-lang/test/vm/16_option_result/010_propagate_err/propagate_err.case.ad | propagate_err.expected.result | d4ff4b6bc192bf38 | b553e6e81c862eba |
| N0205 | vm / 16_option_result/011_coalesce_some | crates/auto-lang/test/vm/16_option_result/011_coalesce_some/coalesce_some.case.ad | coalesce_some.expected.result | e939335e8c71efad | 4c2c7929f57fa19f |
| N0206 | vm / 16_option_result/012_coalesce_none | crates/auto-lang/test/vm/16_option_result/012_coalesce_none/coalesce_none.case.ad | coalesce_none.expected.result | c5caf39059681fb3 | 8655c4812347934f |
| N0207 | vm / 16_option_result/013_is_some_binding | crates/auto-lang/test/vm/16_option_result/013_is_some_binding/is_some_binding.case.ad | is_some_binding.expected.result | 64d5eb53443791ed | 96eda4a6ef8ad82b |
| N0208 | vm / 16_option_result/014_is_none_match | crates/auto-lang/test/vm/16_option_result/014_is_none_match/is_none_match.case.ad | is_none_match.expected.result | 1bbbc2021e3bfa08 | 0fa61f6bfe4d9b20 |
| N0209 | vm / 16_option_result/015_is_ok_binding | crates/auto-lang/test/vm/16_option_result/015_is_ok_binding/is_ok_binding.case.ad | is_ok_binding.expected.result | 787f6db0a4e98b51 | bbd6f4fdbfd61654 |
| N0210 | vm / 16_option_result/016_is_err_match | crates/auto-lang/test/vm/16_option_result/016_is_err_match/is_err_match.case.ad | is_err_match.expected.result | 38420435e3356276 | b84de2379361ada9 |
| N0211 | vm / 16_option_result/017_unwrap_or | crates/auto-lang/test/vm/16_option_result/017_unwrap_or/unwrap_or.case.ad | unwrap_or.expected.out | a36b9a86b67c7b04 | bd72532b7a7f100a |
| N0212 | vm / 16_option_result/018_option_or | crates/auto-lang/test/vm/16_option_result/018_option_or/option_or.case.ad | option_or.expected.out | e40489dc06d9f54a | 0f520954373f0b07 |
| N0213 | vm / 16_option_result/019_result_heap | crates/auto-lang/test/vm/16_option_result/019_result_heap/result_heap.case.ad | result_heap.expected.out | 1ce6c6ee4c60051b | db7acd2987014994 |
| N0214 | vm / 16_option_result/020_result_propagate | crates/auto-lang/test/vm/16_option_result/020_result_propagate/result_propagate.case.ad | result_propagate.expected.out | c1bc36815fa456d0 | dc3a5777b972f05a |
| N0215 | vm / 16_option_result/021_result_enum_error | crates/auto-lang/test/vm/16_option_result/021_result_enum_error/result_enum_error.case.ad | result_enum_error.expected.out | 3fa95284ca8bcd17 | 0674e62644d377c3 |
| N0216 | vm / 16_option_result/022_result_multi_error | crates/auto-lang/test/vm/16_option_result/022_result_multi_error/result_multi_error.case.ad | result_multi_error.expected.out | 83e8621f9a054f3b | acf3eb42783074d1 |
| N0217 | vm / 17_modules/001_use_fn | crates/auto-lang/test/vm/17_modules/001_use_fn/use_fn.case.ad | use_fn.expected.out | 54d3a35f27206b05 | 0c36dd8a372f446e |
| N0218 | vm / 18_ffi/001_file_exists | crates/auto-lang/test/vm/18_ffi/001_file_exists/file_exists.case.ad | file_exists.expected.result | 051bdece4a4d4a5a | 2ae3de356d70f6de |
| N0219 | vm / 18_ffi/003_file_is_dir | crates/auto-lang/test/vm/18_ffi/003_file_is_dir/file_is_dir.case.ad | file_is_dir.expected.result | 041e483cec89551a | 269eec6bf5e1818a |
| N0220 | vm / 18_ffi/006_string_len | crates/auto-lang/test/vm/18_ffi/006_string_len/string_len.case.ad | string_len.expected.result | 2e68db184bc07ad1 | 58d4a51940eb46a5 |
| N0221 | vm / 18_ffi/007_string_is_empty | crates/auto-lang/test/vm/18_ffi/007_string_is_empty/string_is_empty.case.ad | string_is_empty.expected.result | 5b846a5cfddc106d | 8229f9bc53f8cab0 |
| N0222 | vm / 18_ffi/008_string_contains | crates/auto-lang/test/vm/18_ffi/008_string_contains/string_contains.case.ad | string_contains.expected.result | c7b13e7bf87b0ba3 | bdab4371b3ad3917 |
| N0223 | vm / 18_ffi/009_string_starts_with | crates/auto-lang/test/vm/18_ffi/009_string_starts_with/string_starts_with.case.ad | string_starts_with.expected.result | caafa48abc7a2052 | 276d43955ff85730 |
| N0224 | vm / 18_ffi/010_string_ends_with | crates/auto-lang/test/vm/18_ffi/010_string_ends_with/string_ends_with.case.ad | string_ends_with.expected.result | 03802500129efafd | 85d94b49d2414ee8 |
| N0225 | vm / 18_ffi/012_char_is_alpha | crates/auto-lang/test/vm/18_ffi/012_char_is_alpha/char_is_alpha.case.ad | char_is_alpha.expected.result | a1aea6d103d554b8 | 494a063442c9b989 |
| N0226 | vm / 18_ffi/013_char_is_digit | crates/auto-lang/test/vm/18_ffi/013_char_is_digit/char_is_digit.case.ad | char_is_digit.expected.result | 13c99706482b7e85 | 5f3106f938c9604a |
| N0227 | vm / 18_ffi/014_json_is_valid | crates/auto-lang/test/vm/18_ffi/014_json_is_valid/json_is_valid.case.ad | json_is_valid.expected.result | 53cbbfe0d58add29 | 4c166076b69d844f |
| N0228 | vm / 18_ffi/015_json_len | crates/auto-lang/test/vm/18_ffi/015_json_len/json_len.case.ad | json_len.expected.result | e6fc2f21da05a29f | 82b2322d05fca9a6 |
| N0229 | vm / 18_ffi/016_json_is_null | crates/auto-lang/test/vm/18_ffi/016_json_is_null/json_is_null.case.ad | json_is_null.expected.result | 76938dd2edc9fde4 | ca19f2671ece89b2 |
| N0230 | vm / 18_ffi/017_json_as_bool | crates/auto-lang/test/vm/18_ffi/017_json_as_bool/json_as_bool.case.ad | json_as_bool.expected.result | a8a8b143e1b9c2cc | 852ee4cda62bcb4b |
| N0231 | vm / 18_ffi/018_json_has_key | crates/auto-lang/test/vm/18_ffi/018_json_has_key/json_has_key.case.ad | json_has_key.expected.result | 2e0f53c7617f01f6 | 5a82efee83f727d0 |
| N0232 | vm / 18_ffi/019_math_abs_f | crates/auto-lang/test/vm/18_ffi/019_math_abs_f/math_abs_f.case.ad | math_abs_f.expected.result | 9b7041c9dde690e0 | caf47c451ff1ec8a |
| N0233 | vm / 18_ffi/020_str_char_at | crates/auto-lang/test/vm/18_ffi/020_str_char_at/str_char_at.case.ad | str_char_at.expected.result | 6b3b72a0c50062be | 70da5ff449c10582 |
| N0234 | vm / 18_ffi/021_str_substr | crates/auto-lang/test/vm/18_ffi/021_str_substr/str_substr.case.ad | str_substr.expected.result | ad61ea5a434a7611 | 67ee40554af9b929 |
| N0235 | vm / 18_ffi/022_str_trim | crates/auto-lang/test/vm/18_ffi/022_str_trim/str_trim.case.ad | str_trim.expected.result | 53d428ff6e89708b | 3eb3bd3411fa9402 |
| N0236 | vm / 18_ffi/023_str_repeat | crates/auto-lang/test/vm/18_ffi/023_str_repeat/str_repeat.case.ad | str_repeat.expected.result | ab24fe313ffc3f0e | 24ab7d029c280920 |
| N0237 | vm / 18_ffi/024_str_replace | crates/auto-lang/test/vm/18_ffi/024_str_replace/str_replace.case.ad | str_replace.expected.result | 893a198d819ef267 | 577fb5b3166eaf5a |
| N0238 | vm / 18_ffi/025_str_case | crates/auto-lang/test/vm/18_ffi/025_str_case/str_case.case.ad | str_case.expected.result | b3f54936e2dfdc44 | 76d9b9317c1bf6a4 |
| N0239 | vm / 18_ffi/026_str_reverse_find | crates/auto-lang/test/vm/18_ffi/026_str_reverse_find/str_reverse_find.case.ad | str_reverse_find.expected.result | d78962b7eb66c266 | 30ac1020fea04ef4 |
| N0240 | vm / 18_ffi/027_str_find | crates/auto-lang/test/vm/18_ffi/027_str_find/str_find.case.ad | str_find.expected.result | e0913c7591a45752 | fa8f77864103fd31 |
| N0241 | vm / 18_ffi/028_str_replace_first | crates/auto-lang/test/vm/18_ffi/028_str_replace_first/str_replace_first.case.ad | str_replace_first.expected.result | c81d5247d559085e | 44ea5258fa196a05 |
| N0242 | vm / 18_ffi/029_str_match_count | crates/auto-lang/test/vm/18_ffi/029_str_match_count/str_match_count.case.ad | str_match_count.expected.result | 8af6e37df1f0f4f7 | ce7031dfc676801f |
| N0243 | vm / 18_ffi/030_str_contains | crates/auto-lang/test/vm/18_ffi/030_str_contains/str_contains.case.ad | str_contains.expected.result | c7b13e7bf87b0ba3 | 0352326325c866e0 |
| N0244 | vm / 18_ffi/031_str_starts_ends | crates/auto-lang/test/vm/18_ffi/031_str_starts_ends/str_starts_ends.case.ad | str_starts_ends.expected.result | caafa48abc7a2052 | 72be2617e48510c6 |
| N0245 | vm / 18_ffi/032_char_is_alpha | crates/auto-lang/test/vm/18_ffi/032_char_is_alpha/char_is_alpha.case.ad | char_is_alpha.expected.result | a1aea6d103d554b8 | 6b8b19df6744177e |
| N0246 | vm / 18_ffi/033_char_is_digit | crates/auto-lang/test/vm/18_ffi/033_char_is_digit/char_is_digit.case.ad | char_is_digit.expected.result | 13c99706482b7e85 | c3ef5faa3770396d |
| N0247 | vm / 18_ffi/034_char_is_alphanum | crates/auto-lang/test/vm/18_ffi/034_char_is_alphanum/char_is_alphanum.case.ad | char_is_alphanum.expected.result | 75f886ceaee0763e | 9f5ed6d2dbe89ef8 |
| N0248 | vm / 18_ffi/035_char_is_whitespace | crates/auto-lang/test/vm/18_ffi/035_char_is_whitespace/char_is_whitespace.case.ad | char_is_whitespace.expected.result | 0e7b442fa8c1f9e7 | 747f8dd8604bccad |
| N0249 | vm / 18_ffi/036_char_to_lower | crates/auto-lang/test/vm/18_ffi/036_char_to_lower/char_to_lower.case.ad | char_to_lower.expected.result | c99e300dd68528af | 352f853638eb1b19 |
| N0250 | vm / 18_ffi/037_char_to_upper | crates/auto-lang/test/vm/18_ffi/037_char_to_upper/char_to_upper.case.ad | char_to_upper.expected.result | 305c4aaffe0da233 | d48ddbb03aa485dd |
| N0251 | vm / 18_ffi/038_json_encode_parse | crates/auto-lang/test/vm/18_ffi/038_json_encode_parse/json_encode_parse.case.ad | json_encode_parse.expected.out | af3a7d76093a8360 | e76e28c08f40efd0 |
| N0252 | vm / 18_ffi/039_json_get | crates/auto-lang/test/vm/18_ffi/039_json_get/json_get.case.ad | json_get.expected.out | b8d0b1ee71dc15aa | dc1ff2985bd098f3 |
| N0253 | vm / 18_ffi/040_json_array | crates/auto-lang/test/vm/18_ffi/040_json_array/json_array.case.ad | json_array.expected.out | 4330d37bd4a3b0d2 | ced2131c57f45b7b |
| N0254 | vm / 18_ffi/041_json_keys | crates/auto-lang/test/vm/18_ffi/041_json_keys/json_keys.case.ad | json_keys.expected.out | f9f627622d7d8283 | 2f779eb812f11d26 |
| N0255 | vm / 18_ffi/042_json_type_as | crates/auto-lang/test/vm/18_ffi/042_json_type_as/json_type_as.case.ad | json_type_as.expected.out | 26ac8e68582965af | 2af67fa8400aecda |
| N0256 | vm / 18_ffi/043_path_parent | crates/auto-lang/test/vm/18_ffi/043_path_parent/path_parent.case.ad | path_parent.expected.out | 5e55e6f03c46a3c4 | 4d2f9d80c48cb6c4 |
| N0257 | vm / 18_ffi/044_path_ext_filename | crates/auto-lang/test/vm/18_ffi/044_path_ext_filename/path_ext_filename.case.ad | path_ext_filename.expected.out | d3beda36be95c2cd | c2578fc405847192 |
| N0258 | vm / 18_ffi/045_path_join | crates/auto-lang/test/vm/18_ffi/045_path_join/path_join.case.ad | path_join.expected.out | 257e3122af5c5952 | eb3e713044209aaf |
| N0259 | vm / 18_ffi/046_env_get_set | crates/auto-lang/test/vm/18_ffi/046_env_get_set/env_get_set.case.ad | env_get_set.expected.out | bf2853d7796351b4 | 2b4e676c0ece56b2 |
| N0260 | vm / 18_ffi/048_url_encode_decode | crates/auto-lang/test/vm/18_ffi/048_url_encode_decode/url_encode_decode.case.ad | url_encode_decode.expected.out | 478701bc7698f50d | 73ea1336b3f9ac50 |
| N0261 | vm / 18_ffi/049_url_parts | crates/auto-lang/test/vm/18_ffi/049_url_parts/url_parts.case.ad | url_parts.expected.out | 87f0f66e5b55444b | 0866947975922d33 |
| N0262 | vm / 18_ffi/050_regex_is_match | crates/auto-lang/test/vm/18_ffi/050_regex_is_match/regex_is_match.case.ad | regex_is_match.expected.out | 28efe4103b7c1fa0 | fd7c975723888023 |
| N0263 | vm / 18_ffi/051_file_write_read | crates/auto-lang/test/vm/18_ffi/051_file_write_read/file_write_read.case.ad | file_write_read.expected.out | aeac27610b617594 | 0553aa6bf6b8b1d9 |
| N0264 | vm / 18_ffi/052_file_copy_append | crates/auto-lang/test/vm/18_ffi/052_file_copy_append/file_copy_append.case.ad | file_copy_append.expected.out | 7fb8e1de161b6f41 | 70234b5af20d0467 |
| N0265 | vm / 18_ffi/053_file_dir_ops | crates/auto-lang/test/vm/18_ffi/053_file_dir_ops/file_dir_ops.case.ad | file_dir_ops.expected.out | 3dd14f77f6dab394 | 59769f09c6b880b3 |
| N0266 | vm / 18_ffi/054_process_current_dir | crates/auto-lang/test/vm/18_ffi/054_process_current_dir/process_current_dir.case.ad | process_current_dir.expected.out | 86c2b772722fe9e3 | a9095dc21d7791a8 |
| N0267 | vm / 18_ffi/056_math_pow | crates/auto-lang/test/vm/18_ffi/056_math_pow/math_pow.case.ad | math_pow.expected.result | 562451d783b496d6 | b7b3f44c1c156f45 |
| N0268 | vm / 18_ffi/057_str_is_digit | crates/auto-lang/test/vm/18_ffi/057_str_is_digit/str_is_digit.case.ad | str_is_digit.expected.result | 789367d274b73df3 | 86d3877ff842a1f5 |
| N0269 | vm / 18_ffi/058_read_text_range | crates/auto-lang/test/vm/18_ffi/058_read_text_range/read_text_range.case.ad | read_text_range.expected.out | 084319f8ce910f4c | 4a7ea87d278d2d4f |
| N0270 | vm / 19_rust_std/001_time | crates/auto-lang/test/vm/19_rust_std/001_time/time.case.ad | time.expected.out | 2303a4a33a8318eb | f1a4a897e358df28 |
| N0271 | vm / 19_rust_std/002_duration | crates/auto-lang/test/vm/19_rust_std/002_duration/duration.case.ad | duration.expected.out | 9eec95150ee51136 | 64714773d5ca85c2 |
| N0272 | vm / 19_rust_std/003_pathbuf | crates/auto-lang/test/vm/19_rust_std/003_pathbuf/pathbuf.case.ad | pathbuf.expected.out | c0c0dc2de08c56d4 | 7cad4257bf76a1b4 |
| N0273 | vm / 19_rust_std/004_duration_print | crates/auto-lang/test/vm/19_rust_std/004_duration_print/duration_print.case.ad | duration_print.expected.out | 9ede26bc4c534b34 | a43ce71fa5d845d4 |
| N0274 | vm / 19_rust_std/005_instant_duration | crates/auto-lang/test/vm/19_rust_std/005_instant_duration/instant_duration.case.ad | instant_duration.expected.out | 4934d9ecb55ee0bf | 600721eb5f261f74 |
| N0275 | vm / 19_rust_std/006_sync | crates/auto-lang/test/vm/19_rust_std/006_sync/sync.case.ad | sync.expected.out | 3a8d0b1921066771 | 0f7a3049890acbb4 |
| N0276 | vm / 19_rust_std/007_pathbuf | crates/auto-lang/test/vm/19_rust_std/007_pathbuf/pathbuf.case.ad | pathbuf.expected.out | e48a6bfdbf6a6603 | d6edefba4ac13c26 |
| N0277 | vm / 19_rust_std/008_box_cell | crates/auto-lang/test/vm/19_rust_std/008_box_cell/box_cell.case.ad | box_cell.expected.out | 72657401159ea2ed | d879ff96a4a5da41 |
| N0278 | vm / 19_rust_std/009_duration_f64 | crates/auto-lang/test/vm/19_rust_std/009_duration_f64/duration_f64.case.ad | duration_f64.expected.out | 5cd410a9d4c3ccfd | f870bd07e7d6326e |
| N0279 | vm / 19_rust_std/010_process_command | crates/auto-lang/test/vm/19_rust_std/010_process_command/process_command.case.ad | process_command.expected.out | 16b4389c0501503c | f847dbef98ed7d3b |
| N0280 | vm / 20_permission/001_scalar_mode | crates/auto-lang/test/vm/20_permission/001_scalar_mode/scalar_mode.case.ad | scalar_mode.expected.out | 076c195c177aeb4f | adc92630e5c4088a |
| N0281 | vm / 20_permission/002_ext_policy | crates/auto-lang/test/vm/20_permission/002_ext_policy/ext_policy.case.ad | ext_policy.expected.out | 374a6f4a7ec012ca | 4aec5aca9f8db027 |
| N0282 | vm / 20_rust_ffi/001_serde_json | crates/auto-lang/test/vm/20_rust_ffi/001_serde_json/serde_json.case.ad | serde_json.expected.error | 60a7300bfe3443ee | a2d6f1c25624a8ad |
| N0283 | vm / 21_conv/002_neg_i32_to_str | crates/auto-lang/test/vm/21_conv/002_neg_i32_to_str/neg_i32_to_str.case.ad | neg_i32_to_str.expected.out | 432608b94d2ffbd9 | b0c9788245cccc62 |
| N0284 | vm / 21_conv/003_hashmap_mono_insert | crates/auto-lang/test/vm/21_conv/003_hashmap_mono_insert/hashmap_mono_insert.case.ad | hashmap_mono_insert.expected.result | 0ef7d3f6ba566cf2 | 483da03151b68145 |
| N0285 | vm / 21_conv/004_hashset_mono_insert | crates/auto-lang/test/vm/21_conv/004_hashset_mono_insert/hashset_mono_insert.case.ad | hashset_mono_insert.expected.result | 2dd5d71d542c31a0 | 63f134039bcd3090 |
| N0286 | vm / 22_generator/001_sum | crates/auto-lang/test/vm/22_generator/001_sum/sum.case.ad | sum.expected.out | 2815e1baa84fb291 | 3f1e385c356316ef |
| N0287 | vm / 22_generator/002_no_dup | crates/auto-lang/test/vm/22_generator/002_no_dup/no_dup.case.ad | no_dup.expected.out | b0588f713fe99695 | 0177bbc78898da07 |
| N0288 | vm / 22_generator/003_string_yields | crates/auto-lang/test/vm/22_generator/003_string_yields/string_yields.case.ad | string_yields.expected.out | 610f4a480e693aa6 | 25f1524ad099cb99 |
| N0289 | vm / 22_generator/004_indirect_return | crates/auto-lang/test/vm/22_generator/004_indirect_return/indirect_return.case.ad | indirect_return.expected.out | ae40e0d7d21f4f28 | 316f27bd996d4c96 |
| N0290 | vm / 23_actor/001_start_hook | crates/auto-lang/test/vm/23_actor/001_start_hook/start_hook.case.ad | start_hook.expected.out | e5370eca551c4a37 | b4966d8bcb1e7b28 |
| N0291 | vm / 23_actor/002_message_handler | crates/auto-lang/test/vm/23_actor/002_message_handler/message_handler.case.ad | message_handler.expected.out | 5ad3f3a430ec5c97 | 6da25029a7312c62 |
| N0292 | vm / 23_actor/003_multi_message | crates/auto-lang/test/vm/23_actor/003_multi_message/multi_message.case.ad | multi_message.expected.out | 9e7bbf5f238cc902 | d53d4f715726523e |
| N0293 | vm / 23_actor/004_else_handler | crates/auto-lang/test/vm/23_actor/004_else_handler/else_handler.case.ad | else_handler.expected.out | 9bfbaef0b828e580 | 5d987e7d95d58481 |
| N0294 | vm / 23_actor/005_state_write | crates/auto-lang/test/vm/23_actor/005_state_write/state_write.case.ad | state_write.expected.out | 09766175eed8b2f0 | 025d76f84896b222 |
| N0295 | vm / 23_actor/006_state_increment | crates/auto-lang/test/vm/23_actor/006_state_increment/state_increment.case.ad | state_increment.expected.out | 9649bf64f3935abc | a4f65da79f9466f6 |
| N0296 | vm / 23_actor/007_named_variants | crates/auto-lang/test/vm/23_actor/007_named_variants/named_variants.case.ad | named_variants.expected.out | eeea69de974078dd | 310e128dc7233e40 |
| N0297 | vm / 23_actor/008_string_pattern | crates/auto-lang/test/vm/23_actor/008_string_pattern/string_pattern.case.ad | string_pattern.expected.out | 0d710e8cd6a632dc | cc7b8a0355a64167 |
| N0298 | vm / 23_actor/009_stop_hook | crates/auto-lang/test/vm/23_actor/009_stop_hook/stop_hook.case.ad | stop_hook.expected.out | c3934ef83ef17d88 | 10ddfe5048bbb883 |
| N0299 | vm / 23_actor/010_handle_cross_fn | crates/auto-lang/test/vm/23_actor/010_handle_cross_fn/handle_cross_fn.case.ad | handle_cross_fn.expected.out | 478447d2fe93fddf | 597d032cfa11cd95 |
| N0300 | vm / 23_actor/011_handle_param | crates/auto-lang/test/vm/23_actor/011_handle_param/handle_param.case.ad | handle_param.expected.out | 9a395b0eaacfe1aa | 73caa4585b520600 |
| N0301 | vm / 23_actor/012_external_enum_msg | crates/auto-lang/test/vm/23_actor/012_external_enum_msg/external_enum_msg.case.ad | external_enum_msg.expected.out | ebc2f465efed6b76 | 6c82387bef443a0a |
| N0302 | vm / 23_actor/021_handle_struct_field | crates/auto-lang/test/vm/23_actor/021_handle_struct_field/handle_struct_field.case.ad | handle_struct_field.expected.out | bb01267f7d37491c | f22e0db6481c5afb |
| N0303 | vm / 24_generics/001_list_struct | crates/auto-lang/test/vm/24_generics/001_list_struct/list_struct.case.ad | list_struct.expected.out | 6adacccc13da0620 | bc7693bea3f099f2 |
| N0304 | vm / 24_generics/002_to_array | crates/auto-lang/test/vm/24_generics/002_to_array/to_array.case.ad | to_array.expected.out | e93454327dee43f7 | 34fb70e491bc86c0 |
| N0305 | vm / 24_generics/003_filter | crates/auto-lang/test/vm/24_generics/003_filter/filter.case.ad | filter.expected.out | ae058008d5937f5d | 46c0ec16a899c49b |
| N0306 | vm / 25_method_u64/001_to_uint_basic | crates/auto-lang/test/vm/25_method_u64/001_to_uint_basic/to_uint_basic.case.ad | to_uint_basic.expected.out | 7d774c4a554cdf7d | e97a28a73415a88b |
| N0307 | vm / 25_method_u64/002_to_uint_double_promote | crates/auto-lang/test/vm/25_method_u64/002_to_uint_double_promote/to_uint_double_promote.case.ad | to_uint_double_promote.expected.out | 3d1b05dd096694a9 | 63425217e993710c |
| N0308 | vm / 25_method_u64/003_let_u64_from_method | crates/auto-lang/test/vm/25_method_u64/003_let_u64_from_method/let_u64_from_method.case.ad | let_u64_from_method.expected.out | 8b96a5f493d848b2 | f54beca232a2dbcc |
| N0309 | vm / 25_method_u64/004_assign_u64 | crates/auto-lang/test/vm/25_method_u64/004_assign_u64/assign_u64.case.ad | assign_u64.expected.out | e1c1d06f4c485a84 | 75b976e4ab72d01e |
| N0310 | vm / 25_method_u64/005_two_methods_arith | crates/auto-lang/test/vm/25_method_u64/005_two_methods_arith/two_methods_arith.case.ad | two_methods_arith.expected.out | b45a3fc4c3bc38b4 | a371e28f9b382a30 |
| N0311 | vm / 25_method_u64/006_fstring_u64 | crates/auto-lang/test/vm/25_method_u64/006_fstring_u64/fstring_u64.case.ad | fstring_u64.expected.out | 5b82862f8bcdab33 | 3ab3d09da4e34e0c |
| N0312 | vm / 25_method_u64/007_other_i64_methods | crates/auto-lang/test/vm/25_method_u64/007_other_i64_methods/other_i64_methods.case.ad | other_i64_methods.expected.out | e96ccbb1e0680651 | b3f8361c24a923b9 |
| N0313 | vm / 25_method_u64/008_i32_unaffected | crates/auto-lang/test/vm/25_method_u64/008_i32_unaffected/i32_unaffected.case.ad | i32_unaffected.expected.out | 0a2b10b88fe9f1ff | 6f3a9b094e887499 |
| N0314 | vm / 25_method_u64/009_large_value | crates/auto-lang/test/vm/25_method_u64/009_large_value/large_value.case.ad | large_value.expected.out | fe9c7414c036116b | 457cb713a5191d83 |
| N0315 | vm / 25_method_u64/010_global_u64_large | crates/auto-lang/test/vm/25_method_u64/010_global_u64_large/global_u64_large.case.ad | global_u64_large.expected.out | c99bff67df5f2400 | 435c15e263965dc1 |
| N0316 | vm / 25_method_u64/011_bigint_overflow | crates/auto-lang/test/vm/25_method_u64/011_bigint_overflow/bigint_overflow.case.ad | bigint_overflow.expected.out | 7b3021fc159e5646 | e1ca834c8ac497fa |
| N0317 | vm / 25_method_u64/012_list_hof_value | crates/auto-lang/test/vm/25_method_u64/012_list_hof_value/list_hof_value.case.ad | list_hof_value.expected.out | 8c0788cec32b5a65 | 9368edbd4140ac65 |
| N0318 | vm / 26_str_method_on_heap/001_lower_on_split | crates/auto-lang/test/vm/26_str_method_on_heap/001_lower_on_split/lower_on_split.case.ad | lower_on_split.expected.out | fb733570115bad32 | 2b4af2a67683f92c |
| N0319 | vm / 26_str_method_on_heap/002_lower_literal | crates/auto-lang/test/vm/26_str_method_on_heap/002_lower_literal/lower_literal.case.ad | lower_literal.expected.out | 7687f3f02d6aab90 | ce161a481d875399 |
| N0320 | vm / 27_function_reference/001_basic_ref | crates/auto-lang/test/vm/27_function_reference/001_basic_ref/basic_ref.case.ad | basic_ref.expected.out | 42c1535cefc9e615 | f3f6b0b4ae6be783 |
| N0321 | vm / 27_function_reference/002_pass_as_arg | crates/auto-lang/test/vm/27_function_reference/002_pass_as_arg/pass_as_arg.case.ad | pass_as_arg.expected.out | 4c167aaaaedf0bdf | 6341c92bbec349a4 |
| N0322 | vm / 27_function_reference/004_call_unchanged | crates/auto-lang/test/vm/27_function_reference/004_call_unchanged/call_unchanged.case.ad | call_unchanged.expected.out | ffe0c3fb83003a05 | 3961b546a90d0541 |
| N0323 | vm / 28_enum_methods/001_basic | crates/auto-lang/test/vm/28_enum_methods/001_basic/basic.case.ad | basic.expected.out | c0b55aaf0b5839b5 | d1745272a382e9ae |
| N0324 | vm / 28_enum_methods/002_is_match | crates/auto-lang/test/vm/28_enum_methods/002_is_match/is_match.case.ad | is_match.expected.out | 327cfbd1f21fb0de | c934e92b53ded3e0 |
| N0325 | vm / 28_enum_methods/003_payload_method | crates/auto-lang/test/vm/28_enum_methods/003_payload_method/payload_method.case.ad | payload_method.expected.out | 0304fd85841d88a1 | aac49f6b74c7353c |
| N0326 | vm / 28_enum_methods/004_wildcard | crates/auto-lang/test/vm/28_enum_methods/004_wildcard/wildcard.case.ad | wildcard.expected.out | 76ea3971165b8c76 | 96bce33c64651355 |
| N0327 | vm / 28_enum_methods/005_generic_enum_method | crates/auto-lang/test/vm/28_enum_methods/005_generic_enum_method/generic_enum_method.case.ad | generic_enum_method.expected.out | 39161e08f33a1c8d | 0ab31a035c4243e2 |
| N0328 | vm / 28_enum_methods/006_stdlib_result | crates/auto-lang/test/vm/28_enum_methods/006_stdlib_result/stdlib_result.case.ad | stdlib_result.expected.out | c24c1ed4b17a6eee | 60e92adeddd2ac75 |
| N0329 | vm / 29_list_shims/001_int_list | crates/auto-lang/test/vm/29_list_shims/001_int_list/int_list.case.ad | int_list.expected.out | 77c9ce9527688603 | f12f07ca6af6270b |
| N0330 | vm / 29_list_shims/002_struct_list | crates/auto-lang/test/vm/29_list_shims/002_struct_list/struct_list.case.ad | struct_list.expected.out | 2508f117dd064249 | 8a5a15e3f5a3fc1d |
| N0331 | vm / 99_bootstrap/001_token_enum | crates/auto-lang/test/vm/99_bootstrap/001_token_enum/token_enum.case.ad | token_enum.expected.out | d8e6bccbac1d7553 | fbf067ba5e658a28 |
| N0332 | vm / 99_bootstrap/002_keyword_map | crates/auto-lang/test/vm/99_bootstrap/002_keyword_map/keyword_map.case.ad | keyword_map.expected.out | 265700b37bfaf081 | eb8689a95a600b21 |
| N0333 | vm / 99_bootstrap/003_lexer_basic | crates/auto-lang/test/vm/99_bootstrap/003_lexer_basic/lexer_basic.case.ad | lexer_basic.expected.out | 8d980dbd2d443008 | 437f79de65651191 |
| N0334 | vm / 99_bootstrap/004_str_slice_let | crates/auto-lang/test/vm/99_bootstrap/004_str_slice_let/str_slice_let.case.ad | str_slice_let.expected.out | 4854c6621a19dc9a | 638b8c5050ca4b54 |
| N0335 | vm / 99_bootstrap/005_str_slice_concat | crates/auto-lang/test/vm/99_bootstrap/005_str_slice_concat/str_slice_concat.case.ad | str_slice_concat.expected.out | 7fbb225a585e080c | 97a4bf8338b4a247 |
| N0336 | vm / 99_bootstrap/006_bool_compare | crates/auto-lang/test/vm/99_bootstrap/006_bool_compare/bool_compare.case.ad | bool_compare.expected.out | 292a4c97246d41f7 | 1ddcba489657ffc3 |
| N0337 | vm / 99_bootstrap/007_nested_control_flow | crates/auto-lang/test/vm/99_bootstrap/007_nested_control_flow/nested_control_flow.case.ad | nested_control_flow.expected.out | 57954614a86916f2 | bece5c8c7ae6b466 |
| N0338 | vm / 99_bootstrap/008_parser_hello | crates/auto-lang/test/vm/99_bootstrap/008_parser_hello/parser_hello.case.ad | parser_hello.expected.out | 71f9896689417499 | 5486e8e899af9a17 |
| N0339 | vm / 99_bootstrap/009_parser_arithmetic | crates/auto-lang/test/vm/99_bootstrap/009_parser_arithmetic/parser_arithmetic.case.ad | parser_arithmetic.expected.out | 34725c4268d2eb6a | 968fabb5b654efa9 |
| N0340 | vm / 99_bootstrap/010_parser_precedence | crates/auto-lang/test/vm/99_bootstrap/010_parser_precedence/parser_precedence.case.ad | parser_precedence.expected.out | 3e907b6a433aff2f | edde4dfc95f87e09 |
| N0341 | vm / 99_bootstrap/011_parser_unary | crates/auto-lang/test/vm/99_bootstrap/011_parser_unary/parser_unary.case.ad | parser_unary.expected.out | 1beacdaddbcf9a8d | 3f5b784b7b765a73 |
| N0342 | vm / 99_bootstrap/012_parser_not | crates/auto-lang/test/vm/99_bootstrap/012_parser_not/parser_not.case.ad | parser_not.expected.out | 9c6d08706012aae9 | b6b46896431abeff |
| N0343 | vm / 99_bootstrap/013_parser_comparison | crates/auto-lang/test/vm/99_bootstrap/013_parser_comparison/parser_comparison.case.ad | parser_comparison.expected.out | fce2ebfdfaf7d123 | 2a43c7e37be3c401 |
| N0344 | vm / 99_bootstrap/014_parser_equality | crates/auto-lang/test/vm/99_bootstrap/014_parser_equality/parser_equality.case.ad | parser_equality.expected.out | 17bad830d44f42fa | 6c22fd1b1b574dba |
| N0345 | vm / 99_bootstrap/015_parser_logical | crates/auto-lang/test/vm/99_bootstrap/015_parser_logical/parser_logical.case.ad | parser_logical.expected.out | df1ad7d2ef3ec00a | eb2fe2185ecee172 |
| N0346 | vm / 99_bootstrap/016_parser_let | crates/auto-lang/test/vm/99_bootstrap/016_parser_let/parser_let.case.ad | parser_let.expected.out | 7a7835f29ed4d38a | 4b60a276eeec8941 |
| N0347 | vm / 99_bootstrap/017_parser_var | crates/auto-lang/test/vm/99_bootstrap/017_parser_var/parser_var.case.ad | parser_var.expected.out | eabd0e4e044a1636 | f444f8749eed7a8c |
| N0348 | vm / 99_bootstrap/018_parser_fn_decl | crates/auto-lang/test/vm/99_bootstrap/018_parser_fn_decl/parser_fn_decl.case.ad | parser_fn_decl.expected.out | 3fffc5e708aa22e2 | aa5d53dfc41172c5 |
| N0349 | vm / 99_bootstrap/019_parser_fn_call | crates/auto-lang/test/vm/99_bootstrap/019_parser_fn_call/parser_fn_call.case.ad | parser_fn_call.expected.out | e545d1f7db7db298 | 7ef31f1a755bad84 |
| N0350 | vm / 99_bootstrap/020_parser_if_else | crates/auto-lang/test/vm/99_bootstrap/020_parser_if_else/parser_if_else.case.ad | parser_if_else.expected.out | 9c7b11a4f27bcbf4 | f62797686f28238e |
| N0351 | vm / 99_bootstrap/021_parser_for_in | crates/auto-lang/test/vm/99_bootstrap/021_parser_for_in/parser_for_in.case.ad | parser_for_in.expected.out | 56f607ef548054e4 | bd0c1d55406ece64 |
| N0352 | vm / 99_bootstrap/022_parser_return | crates/auto-lang/test/vm/99_bootstrap/022_parser_return/parser_return.case.ad | parser_return.expected.out | 288330cbcb5b95f9 | 6ffd469a2f9d411c |
| N0353 | vm / 99_bootstrap/023_parser_dot | crates/auto-lang/test/vm/99_bootstrap/023_parser_dot/parser_dot.case.ad | parser_dot.expected.out | 8843e442ec3541e3 | dd941802cf4facf9 |
| N0354 | vm / 99_bootstrap/024_parser_assign | crates/auto-lang/test/vm/99_bootstrap/024_parser_assign/parser_assign.case.ad | parser_assign.expected.out | d90010442581920b | 982571de17436dc0 |
| N0355 | vm / 99_bootstrap/025_parser_range | crates/auto-lang/test/vm/99_bootstrap/025_parser_range/parser_range.case.ad | parser_range.expected.out | 13ddd7b20feb6f91 | 431f0f0f15a41812 |
| N0356 | vm / 99_bootstrap/026_parser_string | crates/auto-lang/test/vm/99_bootstrap/026_parser_string/parser_string.case.ad | parser_string.expected.out | 78e5bd5bc7daf9d3 | 71f634901723888a |
| N0357 | vm / 99_bootstrap/027_parser_multi | crates/auto-lang/test/vm/99_bootstrap/027_parser_multi/parser_multi.case.ad | parser_multi.expected.out | ce3add32fc5a936d | f5f3dcfae02b4ad8 |
| N0358 | vm / 99_bootstrap/028_parser_alias | crates/auto-lang/test/vm/99_bootstrap/028_parser_alias/parser_alias.case.ad | parser_alias.expected.out | 6698be26eda7dd20 | 0e4e0236f3f06581 |
| N0359 | vm / 99_bootstrap/029_parser_enum | crates/auto-lang/test/vm/99_bootstrap/029_parser_enum/parser_enum.case.ad | parser_enum.expected.out | 86800982b94d4344 | a96a0f38f2ca2963 |
| N0360 | vm / 99_bootstrap/030_parser_use | crates/auto-lang/test/vm/99_bootstrap/030_parser_use/parser_use.case.ad | parser_use.expected.out | 6366eee4789c6b3b | cd9c3d5864e31385 |
| N0361 | vm / 99_bootstrap/031_parser_spec | crates/auto-lang/test/vm/99_bootstrap/031_parser_spec/parser_spec.case.ad | parser_spec.expected.out | 4308e0116fffcd8b | 2844c6b8ed7e156a |
| N0362 | vm / 99_bootstrap/032_parser_ext | crates/auto-lang/test/vm/99_bootstrap/032_parser_ext/parser_ext.case.ad | parser_ext.expected.out | 6a2b2b6ff4b82de1 | 1cfca4029dddbe44 |
| N0363 | vm / 99_bootstrap/033_parser_closure | crates/auto-lang/test/vm/99_bootstrap/033_parser_closure/parser_closure.case.ad | parser_closure.expected.out | 46f09225d1683871 | 591d702c763b0aa3 |
| N0364 | vm / 99_bootstrap/034_parser_closure_multi | crates/auto-lang/test/vm/99_bootstrap/034_parser_closure_multi/parser_closure_multi.case.ad | parser_closure_multi.expected.out | 4dd9dbc1e4dd07ac | a1073fbd68507da2 |
| N0365 | vm / 99_bootstrap/035_parser_fstr | crates/auto-lang/test/vm/99_bootstrap/035_parser_fstr/parser_fstr.case.ad | parser_fstr.expected.out | 018052b8b3a72d36 | e35bf5ccd6abdf12 |
| N0366 | vm / 99_bootstrap/036_parser_is | crates/auto-lang/test/vm/99_bootstrap/036_parser_is/parser_is.case.ad | parser_is.expected.out | 2937fb9e7333a6ec | 8632cada61c7ecde |
| N0367 | vm / 99_bootstrap/037_parser_object | crates/auto-lang/test/vm/99_bootstrap/037_parser_object/parser_object.case.ad | parser_object.expected.out | 8c6e7b7a5ee104ad | e875785985205fc0 |
| N0368 | vm / 99_bootstrap/038_eval_arithmetic | crates/auto-lang/test/vm/99_bootstrap/038_eval_arithmetic/eval_arithmetic.case.ad | eval_arithmetic.expected.out | de6bac4f0d1a9fee | 99b5563b474a9cb9 |
| N0369 | vm / 99_bootstrap/039_eval_variable | crates/auto-lang/test/vm/99_bootstrap/039_eval_variable/eval_variable.case.ad | eval_variable.expected.out | 296e7f3ab5a0ffe0 | 74937043df01eb9c |
| N0370 | vm / 99_bootstrap/040_eval_fn_call | crates/auto-lang/test/vm/99_bootstrap/040_eval_fn_call/eval_fn_call.case.ad | eval_fn_call.expected.out | 934a6cdb9f310a2c | ee10844a89deff16 |
| N0371 | vm / 99_bootstrap/041_eval_if_else | crates/auto-lang/test/vm/99_bootstrap/041_eval_if_else/eval_if_else.case.ad | eval_if_else.expected.out | d5d54e2867866d78 | f64753a65a5ea44d |
| N0372 | vm / 99_bootstrap/042_eval_for_loop | crates/auto-lang/test/vm/99_bootstrap/042_eval_for_loop/eval_for_loop.case.ad | eval_for_loop.expected.out | d137a0af7177dbc0 | becfc411d6de0f88 |
| N0373 | vm / 99_bootstrap/043_eval_recursion | crates/auto-lang/test/vm/99_bootstrap/043_eval_recursion/eval_recursion.case.ad | eval_recursion.expected.out | ed60d200a3d6f9da | 3c9a03ef0a1e375b |
| N0374 | vm / 99_bootstrap/044_eval_string | crates/auto-lang/test/vm/99_bootstrap/044_eval_string/eval_string.case.ad | eval_string.expected.out | 459ee5de8843e50d | 58e77c2e44c63773 |
| N0375 | vm / 99_bootstrap/045_eval_list | crates/auto-lang/test/vm/99_bootstrap/045_eval_list/eval_list.case.ad | eval_list.expected.out | bb0119624cebe651 | cf16c7171316b461 |
| N0376 | vm / 99_bootstrap/046_eval_closure | crates/auto-lang/test/vm/99_bootstrap/046_eval_closure/eval_closure.case.ad | eval_closure.expected.out | 82f85dd885294d81 | dccf59a9f63cd636 |
| N0377 | vm / 99_bootstrap/047_eval_multi_fn | crates/auto-lang/test/vm/99_bootstrap/047_eval_multi_fn/eval_multi_fn.case.ad | eval_multi_fn.expected.out | 58f93f6b15e72108 | ad7a6efe0b15d704 |
| N0378 | vm / 99_bootstrap/048_eval_str_print | crates/auto-lang/test/vm/99_bootstrap/048_eval_str_print/eval_str_print.case.ad | eval_str_print.expected.out | 522f9fe89368f0ce | 08ee9e301d319bcd |
| N0379 | vm / 99_bootstrap/049_eval_str_var | crates/auto-lang/test/vm/99_bootstrap/049_eval_str_var/eval_str_var.case.ad | eval_str_var.expected.out | 198f6cfae11606d5 | 96a4ce9b051d77a3 |
| N0380 | vm / 99_bootstrap/050_eval_str_concat | crates/auto-lang/test/vm/99_bootstrap/050_eval_str_concat/eval_str_concat.case.ad | eval_str_concat.expected.out | 835af1b339b1f6b2 | 23563c33d4c7083c |
| N0381 | vm / 99_bootstrap/051_eval_str_fn_arg | crates/auto-lang/test/vm/99_bootstrap/051_eval_str_fn_arg/eval_str_fn_arg.case.ad | eval_str_fn_arg.expected.out | 675cff714bfe5c3c | 67bfd8a1d6a9359a |
| N0382 | vm / 99_bootstrap/051a_eval_str_fn_arg_simple | crates/auto-lang/test/vm/99_bootstrap/051a_eval_str_fn_arg_simple/eval_str_fn_arg_simple.case.ad | eval_str_fn_arg_simple.expected.out | 569574098756500e | 4c14425164b970ba |
| N0383 | vm / 99_bootstrap/052_eval_str_return | crates/auto-lang/test/vm/99_bootstrap/052_eval_str_return/eval_str_return.case.ad | eval_str_return.expected.out | bc3cdc69c0e798b2 | 4b4c911e1517decd |
| N0384 | vm / 99_bootstrap/053_eval_str_literal | crates/auto-lang/test/vm/99_bootstrap/053_eval_str_literal/eval_str_literal.case.ad | eval_str_literal.expected.out | 7495aeff3e2f58d7 | 6d7962fefc41cf0e |
| N0385 | vm / 99_bootstrap/054_type_infer_basic | crates/auto-lang/test/vm/99_bootstrap/054_type_infer_basic/type_infer_basic.case.ad | type_infer_basic.expected.out | aa98dc1d19e9dae1 | a5752981f48401be |
| N0386 | vm / 99_bootstrap/055_type_infer_fn | crates/auto-lang/test/vm/99_bootstrap/055_type_infer_fn/type_infer_fn.case.ad | type_infer_fn.expected.out | 22cdc357c26813ed | c03ba9efb7a14262 |
| N0387 | vm / 99_bootstrap/060_bytecode_int | crates/auto-lang/test/vm/99_bootstrap/060_bytecode_int/bytecode_int.case.ad | bytecode_int.expected.out | c74307620d0ceca7 | f35674f098bfc99f |
| N0388 | vm / 99_bootstrap/061_bytecode_var | crates/auto-lang/test/vm/99_bootstrap/061_bytecode_var/bytecode_var.case.ad | bytecode_var.expected.out | 9f9a3a7d95209d65 | 4cfe4b5a54ee2a91 |
| N0389 | vm / 99_bootstrap/062_bytecode_fn | crates/auto-lang/test/vm/99_bootstrap/062_bytecode_fn/bytecode_fn.case.ad | bytecode_fn.expected.out | 99f23c3e0b22bb57 | dc3919ed0f6502fb |
| N0390 | vm / 99_bootstrap/063_bytecode_if | crates/auto-lang/test/vm/99_bootstrap/063_bytecode_if/bytecode_if.case.ad | bytecode_if.expected.out | 862be84871496607 | 21b479b4afe3c10a |
| N0391 | vm / 99_bootstrap/064_bytecode_for | crates/auto-lang/test/vm/99_bootstrap/064_bytecode_for/bytecode_for.case.ad | bytecode_for.expected.out | e94c341fbb428d09 | 99b0117f319036fe |
| N0392 | vm / 99_bootstrap/065_bytecode_str | crates/auto-lang/test/vm/99_bootstrap/065_bytecode_str/bytecode_str.case.ad | bytecode_str.expected.out | 435c330b2a59cc1c | fe29d592c31fe09f |
| N0393 | vm / 99_bootstrap/069_bvm_str_len | crates/auto-lang/test/vm/99_bootstrap/069_bvm_str_len/bvm_str_len.case.ad | bvm_str_len.expected.out | ed6f9e286caa4dec | 96a1ad226d8bac36 |
| N0394 | vm / 99_bootstrap/070_bvm_str_methods | crates/auto-lang/test/vm/99_bootstrap/070_bvm_str_methods/bvm_str_methods.case.ad | bvm_str_methods.expected.out | ba9c1bb4a0a90480 | 766e2a232013c034 |
| N0395 | vm / 99_bootstrap/071_bvm_map_str | crates/auto-lang/test/vm/99_bootstrap/071_bvm_map_str/bvm_map_str.case.ad | bvm_map_str.expected.out | ae21be1de896f900 | 48e6166446add4c8 |
| N0396 | vm / 99_bootstrap/072_bvm_list_set | crates/auto-lang/test/vm/99_bootstrap/072_bvm_list_set/bvm_list_set.case.ad | bvm_list_set.expected.out | 6513102932753129 | 81ef6e702e164f63 |
| N0397 | vm / 99_bootstrap/073_bvm_list_pop | crates/auto-lang/test/vm/99_bootstrap/073_bvm_list_pop/bvm_list_pop.case.ad | bvm_list_pop.expected.out | fb10c061b08748b3 | 2eec6ccd4423cf18 |
| N0398 | vm / 99_bootstrap/074_bvm_str_ops | crates/auto-lang/test/vm/99_bootstrap/074_bvm_str_ops/bvm_str_ops.case.ad | bvm_str_ops.expected.out | 3d150b446136a020 | 7c99dd42b88441e2 |
| N0399 | vm / 99_bootstrap/075_list_str_push_get | crates/auto-lang/test/vm/99_bootstrap/075_list_str_push_get/list_str_push_get.case.ad | list_str_push_get.expected.out | bf080117ba116508 | d1aa7b03fca32646 |
| N0400 | vm / 99_bootstrap/076_list_str_set | crates/auto-lang/test/vm/99_bootstrap/076_list_str_set/list_str_set.case.ad | list_str_set.expected.out | b5ba05499c6d4099 | 966766938af5162d |
| N0401 | vm / 99_bootstrap/077_list_str_pop | crates/auto-lang/test/vm/99_bootstrap/077_list_str_pop/list_str_pop.case.ad | list_str_pop.expected.out | 08adcb234047def5 | d66b466a96b7974b |
| N0402 | vm / 99_bootstrap/078_list_mixed_types | crates/auto-lang/test/vm/99_bootstrap/078_list_mixed_types/list_mixed_types.case.ad | list_mixed_types.expected.out | 2308fa41ef2f7b1d | f1e664696ff47664 |
| N0403 | vm / 99_bootstrap/079_list_str_fn_arg | crates/auto-lang/test/vm/99_bootstrap/079_list_str_fn_arg/list_str_fn_arg.case.ad | list_str_fn_arg.expected.out | ebfb57e05fc5855b | 798b41a910adeca7 |
| N0404 | vm / 99_bootstrap/080_list_str_loop | crates/auto-lang/test/vm/99_bootstrap/080_list_str_loop/list_str_loop.case.ad | list_str_loop.expected.out | da35ab55de2d9362 | 22d0b8906a27f96b |
| N0405 | vm / 99_bootstrap/081_a2r_hello | crates/auto-lang/test/vm/99_bootstrap/081_a2r_hello/a2r_hello.case.ad | a2r_hello.expected.out | 396497059656dd38 | 51df51b684262696 |
| N0406 | vm / 99_bootstrap/082_a2r_fn | crates/auto-lang/test/vm/99_bootstrap/082_a2r_fn/a2r_fn.case.ad | a2r_fn.expected.out | 623a87b8aad3ab16 | 4690338f5841aa03 |
| N0407 | vm / 99_bootstrap/083_a2r_var | crates/auto-lang/test/vm/99_bootstrap/083_a2r_var/a2r_var.case.ad | a2r_var.expected.out | daa8a059de311e62 | e783095d3cc5cc7d |
| N0408 | vm / 99_bootstrap/084_a2r_if | crates/auto-lang/test/vm/99_bootstrap/084_a2r_if/a2r_if.case.ad | a2r_if.expected.out | a87ff33b98a92f01 | 0567a958ef2023db |
| N0409 | vm / 99_bootstrap/085_a2r_for | crates/auto-lang/test/vm/99_bootstrap/085_a2r_for/a2r_for.case.ad | a2r_for.expected.out | 1e6f4577dd5f1a9c | 98f4c1d7316cffad |
| N0410 | vm / 99_bootstrap/086_a2r_str | crates/auto-lang/test/vm/99_bootstrap/086_a2r_str/a2r_str.case.ad | a2r_str.expected.out | 779b9e8688f1e415 | ba11ac4aac29b2b5 |
| N0411 | vm / 99_bootstrap/087_a2r_use | crates/auto-lang/test/vm/99_bootstrap/087_a2r_use/a2r_use.case.ad | a2r_use.expected.out | d97f6b5aea8b1e11 | 0f72a5c9e40363ba |
| N0412 | vm / 99_bootstrap/088_a2r_type | crates/auto-lang/test/vm/99_bootstrap/088_a2r_type/a2r_type.case.ad | a2r_type.expected.out | 3c6f8c671b5be1a9 | 80f7dbd35ba83453 |
| N0413 | vm / 99_bootstrap/089_a2r_enum | crates/auto-lang/test/vm/99_bootstrap/089_a2r_enum/a2r_enum.case.ad | a2r_enum.expected.out | 2159ac62dc060718 | 65a5dceb8f7d0afc |
| N0414 | vm / 99_bootstrap/090_a2r_is | crates/auto-lang/test/vm/99_bootstrap/090_a2r_is/a2r_is.case.ad | a2r_is.expected.out | 4321d46025e8727e | d44b14608400558c |
| N0415 | vm / 99_bootstrap/091_a2r_ext | crates/auto-lang/test/vm/99_bootstrap/091_a2r_ext/a2r_ext.case.ad | a2r_ext.expected.out | 91df2a507d865364 | 5298484da689ac11 |
| N0416 | vm / 99_bootstrap/092_a2r_fstr | crates/auto-lang/test/vm/99_bootstrap/092_a2r_fstr/a2r_fstr.case.ad | a2r_fstr.expected.out | 108aca9bf009bef0 | f2895b40b8ed609f |
| N0417 | vm / 99_bootstrap/093_a2r_spec | crates/auto-lang/test/vm/99_bootstrap/093_a2r_spec/a2r_spec.case.ad | a2r_spec.expected.out | 558ead16a6868e61 | bd04bc63471e7550 |
| N0418 | vm / 99_bootstrap/094_a2r_closure | crates/auto-lang/test/vm/99_bootstrap/094_a2r_closure/a2r_closure.case.ad | a2r_closure.expected.out | 6b879ececef56ad6 | 8957cb4d317e9224 |
| N0419 | vm / 99_bootstrap/095_a2r_alias | crates/auto-lang/test/vm/99_bootstrap/095_a2r_alias/a2r_alias.case.ad | a2r_alias.expected.out | 129fab86dbf722b9 | 0570ef8dc8175722 |
| N0420 | vm / 99_bootstrap/096_a2r_object | crates/auto-lang/test/vm/99_bootstrap/096_a2r_object/a2r_object.case.ad | a2r_object.expected.out | 506ca3d368812447 | 2b32a3785c04dc6a |
| N0421 | vm / 99_bootstrap/097_a2r_array | crates/auto-lang/test/vm/99_bootstrap/097_a2r_array/a2r_array.case.ad | a2r_array.expected.out | b3935676ca70e787 | 3752708e1381eac5 |
| N0422 | vm / 99_bootstrap/098_a2r_error_prop | crates/auto-lang/test/vm/99_bootstrap/098_a2r_error_prop/a2r_error_prop.case.ad | a2r_error_prop.expected.out | 8edd7222d9503a3a | ed3ddc0a968d3737 |
| N0423 | vm / 99_bootstrap/099_a2r_self_field | crates/auto-lang/test/vm/99_bootstrap/099_a2r_self_field/a2r_self_field.case.ad | a2r_self_field.expected.out | 3db41782c03c76d6 | 13a5c23978b474c4 |
| N0424 | vm / 99_bootstrap/100_a2r_use_ffi | crates/auto-lang/test/vm/99_bootstrap/100_a2r_use_ffi/a2r_use_ffi.case.ad | a2r_use_ffi.expected.out | f746d327eaa5e163 | ea010b20d061a644 |
| N0425 | vm / 99_bootstrap/101_a2r_is_multi | crates/auto-lang/test/vm/99_bootstrap/101_a2r_is_multi/a2r_is_multi.case.ad | a2r_is_multi.expected.out | 676d0ae42f0786f6 | db4cf28d3b175951 |
| N0426 | vm / 99_bootstrap/102_a2r_struct_ctor | crates/auto-lang/test/vm/99_bootstrap/102_a2r_struct_ctor/a2r_struct_ctor.case.ad | a2r_struct_ctor.expected.out | 170e32f4d28de998 | 2d87a0ea84547b00 |
| N0427 | vm / 99_bootstrap/103_a2r_generic_type | crates/auto-lang/test/vm/99_bootstrap/103_a2r_generic_type/a2r_generic_type.case.ad | a2r_generic_type.expected.out | 79f85fa6eb6c0572 | e27adfd4fd59fc20 |
| N0428 | vm / 99_bootstrap/104_a2r_generic_vec | crates/auto-lang/test/vm/99_bootstrap/104_a2r_generic_vec/a2r_generic_vec.case.ad | a2r_generic_vec.expected.out | 8ae308faa782f808 | cb6e1d8bce2bcb7a |
| N0429 | vm / 99_bootstrap/105_a2r_generic_map | crates/auto-lang/test/vm/99_bootstrap/105_a2r_generic_map/a2r_generic_map.case.ad | a2r_generic_map.expected.out | 1f97009e4970002a | e429df8905a70460 |
| N0430 | vm / 99_bootstrap/106_a2r_generic_enum | crates/auto-lang/test/vm/99_bootstrap/106_a2r_generic_enum/a2r_generic_enum.case.ad | a2r_generic_enum.expected.out | d888b2fccd153099 | 77830d9c654c828e |
| N0431 | vm / 99_bootstrap/107_a2r_option_match | crates/auto-lang/test/vm/99_bootstrap/107_a2r_option_match/a2r_option_match.case.ad | a2r_option_match.expected.out | eeeb9bb860c7b48c | b06ba9894ffc7a0d |
| N0432 | vm / 99_bootstrap/108_a2r_ownership | crates/auto-lang/test/vm/99_bootstrap/108_a2r_ownership/a2r_ownership.case.ad | a2r_ownership.expected.out | 261e3896185c506a | 85994567e687cb5c |
| N0433 | vm / 99_bootstrap/110_a2r_phase3 | crates/auto-lang/test/vm/99_bootstrap/110_a2r_phase3/a2r_phase3.case.ad | a2r_phase3.expected.out | 19a9a5bfbc1795be | 519b42b664d3bcb4 |
| N0434 | vm / 99_bootstrap/111_a2r_real_source | crates/auto-lang/test/vm/99_bootstrap/111_a2r_real_source/a2r_real_source.case.ad | a2r_real_source.expected.out | 1c0f1c5393779f7c | f71be27298d87bd6 |
| N0435 | vm / 99_bootstrap/112_a2r_generic_registry | crates/auto-lang/test/vm/99_bootstrap/112_a2r_generic_registry/a2r_generic_registry.case.ad | a2r_generic_registry.expected.out | caedd1a32191c039 | 2c7ccc2a14ea8ee0 |
| N0436 | vm / 99_idiom2/d01_nested_fn_void | crates/auto-lang/test/vm/99_idiom2/d01_nested_fn_void/nested_fn_void.case.ad | nested_fn_void.expected.out | c80cb518f23e0d2a | 24d7495ae2723410 |
| N0437 | vm / 99_idiom2/d01b_nested_fn_capture | crates/auto-lang/test/vm/99_idiom2/d01b_nested_fn_capture/nested_fn_capture.case.ad | nested_fn_capture.expected.error | e4c6dcca18b5a2e5 | 4f0a9f0abc20f45f |
| N0438 | vm / 99_idiom2/d02_struct_misuse | crates/auto-lang/test/vm/99_idiom2/d02_struct_misuse/struct_misuse.case.ad | struct_misuse.expected.error | 54ae7ec3387bdea2 | fae85693d251490c |
| N0439 | vm / 99_idiom2/m01_field_rw | crates/auto-lang/test/vm/99_idiom2/m01_field_rw/field_rw.case.ad | field_rw.expected.out | 4eac11c23b61115b | c2934b41c523c178 |
| N0440 | vm / 99_idiom2/m02_method_calls_method | crates/auto-lang/test/vm/99_idiom2/m02_method_calls_method/method_calls_method.case.ad | method_calls_method.expected.out | 52a03302fd983c70 | a5f72fa663c2fea5 |
| N0441 | vm / 99_idiom2/m03_static_new_chain | crates/auto-lang/test/vm/99_idiom2/m03_static_new_chain/static_new_chain.case.ad | static_new_chain.expected.out | e769159e2758dd15 | bdcc384a34129dea |
| N0442 | vm / 99_idiom2/m04_positional_ctor | crates/auto-lang/test/vm/99_idiom2/m04_positional_ctor/positional_ctor.case.ad | positional_ctor.expected.out | f2d817ff8ffaca07 | 76023ef309ede408 |
| N0443 | vm / 99_idiom2/m05_list_elem_method | crates/auto-lang/test/vm/99_idiom2/m05_list_elem_method/list_elem_method.case.ad | list_elem_method.expected.out | 7f603135c04f29df | c704ea03c0a6854a |
| N0444 | vm / 99_idiom2/m06_method_value | crates/auto-lang/test/vm/99_idiom2/m06_method_value/method_value.case.ad | method_value.expected.out | 8035d827a38c2ed8 | cf4fe6cd90ade857 |
| N0445 | vm / 99_idiom2/m07_ext_method | crates/auto-lang/test/vm/99_idiom2/m07_ext_method/ext_method.case.ad | ext_method.expected.out | 5398477de3dceb34 | 85d9591c0c907cf0 |
| N0446 | vm / 99_idiom2/m08_receiver_forms | crates/auto-lang/test/vm/99_idiom2/m08_receiver_forms/receiver_forms.case.ad | receiver_forms.expected.out | fabb33c7087e3e4c | 69fdfc884991ca19 |
| N0447 | vm / 99_idiom2/m09_method_name_set | crates/auto-lang/test/vm/99_idiom2/m09_method_name_set/method_name_set.case.ad | method_name_set.expected.out | 6aa492a47bfb719c | b08d29b2fd37395d |
| N0448 | vm / 99_idiom2/m10_pipe | crates/auto-lang/test/vm/99_idiom2/m10_pipe/pipe.case.ad | pipe.expected.out | f04cc6fe7ad474f6 | 2f1403c70cb73f74 |
| N0449 | vm / 99_idiom_probe/p01_is_string | crates/auto-lang/test/vm/99_idiom_probe/p01_is_string/is_string.case.ad | is_string.expected.out | 853d572f95112028 | c79425bc78908ba4 |
| N0450 | vm / 99_idiom_probe/p02b_enum_or_arm | crates/auto-lang/test/vm/99_idiom_probe/p02b_enum_or_arm/enum_or_arm.case.ad | enum_or_arm.expected.out | 1907f91d784da580 | cab49fbb08b4df49 |
| N0451 | vm / 99_idiom_probe/p03_enum_payload | crates/auto-lang/test/vm/99_idiom_probe/p03_enum_payload/enum_payload.case.ad | enum_payload.expected.out | 703c20b501f0d722 | 915730283ec54c8b |
| N0452 | vm / 99_idiom_probe/p04_runtime_concat_payload | crates/auto-lang/test/vm/99_idiom_probe/p04_runtime_concat_payload/runtime_concat_payload.case.ad | runtime_concat_payload.expected.out | 6b7b055d034fd2b7 | 77523c3f78f77e6e |
| N0453 | vm / 99_idiom_probe/p05_double_match | crates/auto-lang/test/vm/99_idiom_probe/p05_double_match/double_match.case.ad | double_match.expected.out | 1293aee01bd004b6 | 1262f0073b637904 |
| N0454 | vm / 99_idiom_probe/p06_while_continue | crates/auto-lang/test/vm/99_idiom_probe/p06_while_continue/while_continue.case.ad | while_continue.expected.out | 4e0d98a9cf75dcf9 | 8e86147bb462b93d |
| N0455 | vm / 99_idiom_probe/p07_cond_for_continue | crates/auto-lang/test/vm/99_idiom_probe/p07_cond_for_continue/cond_for_continue.case.ad | cond_for_continue.expected.out | 6e11d6158469f4e9 | fc69635c0c998b8c |
| N0456 | vm / 99_idiom_probe/p08_guard_arm | crates/auto-lang/test/vm/99_idiom_probe/p08_guard_arm/guard_arm.case.ad | guard_arm.expected.out | d12466e30ac5b495 | 755fbe12974d7dbb |
| N0457 | vm / 99_idiom_probe/p09g_local_var_workaround | crates/auto-lang/test/vm/99_idiom_probe/p09g_local_var_workaround/local_var_workaround.case.ad | local_var_workaround.expected.out | 5fc90b8c705d7ffc | 6608b76a13522357 |
| N0458 | vm / 99_idiom_probe/p09h_inline_enum_payload | crates/auto-lang/test/vm/99_idiom_probe/p09h_inline_enum_payload/inline_enum_payload.case.ad | inline_enum_payload.expected.out | f35d9d28a1ee2cbf | 6935a5dbaf188d74 |
| N0459 | vm / 99_idiom_probe/p10_enum_reassign_eq | crates/auto-lang/test/vm/99_idiom_probe/p10_enum_reassign_eq/enum_reassign_eq.case.ad | enum_reassign_eq.expected.out | e16be2ad2c5bc444 | d950d9af1bb8946c |
| N0460 | vm / 99_idiom_probe/p11_is_char | crates/auto-lang/test/vm/99_idiom_probe/p11_is_char/is_char.case.ad | is_char.expected.out | 21b4143f0a164f09 | b7db0e2e653c11b9 |
| N0461 | vm / 99_idiom_probe/p12_is_binding_types | crates/auto-lang/test/vm/99_idiom_probe/p12_is_binding_types/is_binding_types.case.ad | is_binding_types.expected.out | e1189e9116cc7da2 | f7a46eca1c7f19aa |
| N0462 | vm / 99_idiom_probe/p13a_struct_field_enum | crates/auto-lang/test/vm/99_idiom_probe/p13a_struct_field_enum/struct_field_enum.case.ad | struct_field_enum.expected.out | 99d63536e77d722a | e21891d9d7871dd4 |
| N0463 | vm / 99_idiom_probe/p13b_struct_literal_field | crates/auto-lang/test/vm/99_idiom_probe/p13b_struct_literal_field/struct_literal_field.case.ad | struct_literal_field.expected.out | 35e99a09eb9124e1 | ede8a565b0d3d642 |
| N0464 | vm / 99_idiom_probe/p13c_struct_literal_literal_payload | crates/auto-lang/test/vm/99_idiom_probe/p13c_struct_literal_literal_payload/struct_literal_literal_payload.case.ad | struct_literal_literal_payload.expected.out | 0d586b33ecbf3014 | 105c07697de602f1 |
| N0465 | vm / 99_misc/002_closure_by_ref | crates/auto-lang/test/vm/99_misc/002_closure_by_ref/closure_by_ref.case.ad | closure_by_ref.expected.out | 5e70c7acafe22e7e | f6e7172a803b06fe |
| N0466 | vm / 99_p588debt/001_inline_arg_accum | crates/auto-lang/test/vm/99_p588debt/001_inline_arg_accum/inline_arg_accum.case.ad | inline_arg_accum.expected.out | 5c4e2a329d4843a1 | 282264b64bd3e030 |
| N0467 | vm / 99_p588debt/002_str_return_fn | crates/auto-lang/test/vm/99_p588debt/002_str_return_fn/str_return_fn.case.ad | str_return_fn.expected.out | 5cb9383af2694da4 | f10a7954fa9074ec |
| N0468 | vm / 99_p588debt/003_operand_shapes | crates/auto-lang/test/vm/99_p588debt/003_operand_shapes/operand_shapes.case.ad | operand_shapes.expected.out | 56dbf5c71ad5405d | f3d41732665541e0 |
| N0469 | vm / 99_plan230/001_struct_f64 | crates/auto-lang/test/vm/99_plan230/001_struct_f64/struct_f64.case.ad | struct_f64.expected.out | 168149c917050b30 | 13bdf2b1c2ddeabf |
| N0470 | vm / 99_plan231/001_nested_mutfn | crates/auto-lang/test/vm/99_plan231/001_nested_mutfn/nested_mutfn.case.ad | nested_mutfn.expected.out | 17a48b3824099443 | 93dadea39ac94109 |
| N0471 | vm / 99_plan231/002_direct_drain | crates/auto-lang/test/vm/99_plan231/002_direct_drain/direct_drain.case.ad | direct_drain.expected.out | 3d79e2a5fc56d6a4 | 43d3e55c1b1d9a57 |
| N0472 | vm / 99_plan231/003_simple_for_option | crates/auto-lang/test/vm/99_plan231/003_simple_for_option/simple_for_option.case.ad | simple_for_option.expected.out | d126d3772cef08cd | 5b3a457197910c59 |
| N0473 | vm / 99_plan231/004_for_option_no_is | crates/auto-lang/test/vm/99_plan231/004_for_option_no_is/for_option_no_is.case.ad | for_option_no_is.expected.out | 75ac18890de6f364 | 5866436ddf51f9e0 |
| N0474 | vm / 99_plan231/005_mutfn_no_loop | crates/auto-lang/test/vm/99_plan231/005_mutfn_no_loop/mutfn_no_loop.case.ad | mutfn_no_loop.expected.out | df2f9c023471df5f | 08f49a35f8990351 |
| N0475 | vm / 99_plan231/006_mutfn_int_return | crates/auto-lang/test/vm/99_plan231/006_mutfn_int_return/mutfn_int_return.case.ad | mutfn_int_return.expected.out | d9a53097efc51a75 | a7752455e2ca2092 |
| N0476 | vm / 99_plan231/007_mutfn_void_return | crates/auto-lang/test/vm/99_plan231/007_mutfn_void_return/mutfn_void_return.case.ad | mutfn_void_return.expected.out | 1c0ad6efce6e33eb | 7ae332a5d7c7864c |
| N0477 | vm / 99_plan231/008_like_vmtest09 | crates/auto-lang/test/vm/99_plan231/008_like_vmtest09/like_vmtest09.case.ad | like_vmtest09.expected.out | f76ae097c1e3c3a4 | b1dde54a3ba8ef5a |
| N0478 | vm / 99_plan231/009_mutfn_print_direct | crates/auto-lang/test/vm/99_plan231/009_mutfn_print_direct/mutfn_print_direct.case.ad | mutfn_print_direct.expected.out | 7d5237baa6db014e | 93f3b15767dfdd0e |
| N0479 | vm / 99_plan231/010_mutfn_int_no_fstr | crates/auto-lang/test/vm/99_plan231/010_mutfn_int_no_fstr/mutfn_int_no_fstr.case.ad | mutfn_int_no_fstr.expected.out | 88c5c481965d2955 | d7e64c062a80d1f3 |
| N0480 | vm / 99_plan664/001_value_lexemes | crates/auto-lang/test/vm/99_plan664/001_value_lexemes/value_lexemes.case.ad | value_lexemes.expected.out | 928634d47d14dd03 | 7999a6387fa5f14b |
| N0481 | vm / 99_py_dispatch/01_dual_len | crates/auto-lang/test/vm/99_py_dispatch/01_dual_len/dual_len.case.ad | dual_len.expected.out | a3e93876869c87c4 | 68a0c11f84b84b05 |
| N0482 | vm / 99_py_dispatch/02_auto_str_unchanged | crates/auto-lang/test/vm/99_py_dispatch/02_auto_str_unchanged/auto_str_unchanged.case.ad | auto_str_unchanged.expected.out | 808e78c542319b55 | 6ee3d8ef3828dfd2 |
| N0483 | vm / 99_script_err/01_propagate | crates/auto-lang/test/vm/99_script_err/01_propagate/propagate.case.ad | propagate.expected.out | 87ec23fa31e445aa | faf07f5cb454cce3 |
| N0484 | vm / 99_script_err/02_catch_value | crates/auto-lang/test/vm/99_script_err/02_catch_value/catch_value.case.ad | catch_value.expected.out | 8afb666fafc90e6b | 5d8717d29d12a942 |
| N0485 | vm / 99_script_err/03_uncaught | crates/auto-lang/test/vm/99_script_err/03_uncaught/uncaught.case.ad | uncaught.expected.error | 09c97647f65b18a6 | 66a99f9a6dd6a32a |
| N0486 | vm / 99_script_err/04_with_as_propagate | crates/auto-lang/test/vm/99_script_err/04_with_as_propagate/with_as_propagate.case.ad | with_as_propagate.expected.out | 7e8f5d9e61546654 | 8854b766bf3dfa85 |
| N0487 | vm / 99_script_err/05_obj_call_auto_catch | crates/auto-lang/test/vm/99_script_err/05_obj_call_auto_catch/obj_call_auto_catch.case.ad | obj_call_auto_catch.expected.out | ebdf968cc2e29fdb | fd5deed3897e023f |
| N0488 | vm / 99_short_circuit/001_and_or_short_circuit | crates/auto-lang/test/vm/99_short_circuit/001_and_or_short_circuit/and_or_short_circuit.case.ad | and_or_short_circuit.expected.out | b6fcfae10920151d | 153714fd4b17b12d |
| N0489 | vm / 99_short_circuit/002_calc_eval_expr | crates/auto-lang/test/vm/99_short_circuit/002_calc_eval_expr/calc_eval_expr.case.ad | calc_eval_expr.expected.out | 1543b2b2537b0261 | a2b0c365bdbe4d7a |
| N0490 | vm / 99_short_circuit/003_negative_index_guard | crates/auto-lang/test/vm/99_short_circuit/003_negative_index_guard/negative_index_guard.case.ad | negative_index_guard.expected.out | fe101d236d8788d5 | b08736b2a9e91725 |
| N0491 | vm / 99_slice/001_slice | crates/auto-lang/test/vm/99_slice/001_slice/slice.case.ad | slice.expected.out | d6d9d479db61c874 | 4cf4c9a23cf5f97c |
| N0492 | vm / 99_spec_dispatch/000_spec_basic | crates/auto-lang/test/vm/99_spec_dispatch/000_spec_basic/spec_basic.case.ad | spec_basic.expected.out | 33f65b770780ae1a | fcb3d477fa99ee2d |
| N0493 | vm / 99_spec_dispatch/20_tool_registry | crates/auto-lang/test/vm/99_spec_dispatch/20_tool_registry/tool_registry.case.ad | tool_registry.expected.out | 75f52931259acfc9 | bf98eefb421fb6bb |
| N0494 | vm / 99_spec_dispatch/31_tool_exec_with_perm | crates/auto-lang/test/vm/99_spec_dispatch/31_tool_exec_with_perm/tool_exec_with_perm.case.ad | tool_exec_with_perm.expected.out | 43f5e78ecda06433 | 2c369e63929ca583 |
| N0495 | vm / aavm2/001_smoke | crates/auto-lang/test/vm/aavm2/001_smoke/smoke.case.ad | smoke.expected.out | ce7731aed91c240e | 4a9bf96f8568ad2e |
| N0496 | vm / aavm2/002_hello_compile | crates/auto-lang/test/vm/aavm2/002_hello_compile/hello_compile.case.ad | hello_compile.expected.out | defffe73cc1d5a85 | b5b508996ecbc910 |
| N0497 | a2r / 01_basics/001_hello | crates/auto-lang/test/a2r/01_basics/001_hello/hello.case.ad | hello.expected.rs | c39a91c5b8028e69 | 1947e03022dd1c21 |
| N0498 | a2r / 01_basics/002_sqrt | crates/auto-lang/test/a2r/01_basics/002_sqrt/sqrt.case.ad | sqrt.expected.rs | 4add51d9b4186d3a | 9a24c08f465d9a2a |
| N0499 | a2r / 01_basics/003_func | crates/auto-lang/test/a2r/01_basics/003_func/func.case.ad | func.expected.rs | 31457f27203ee666 | 0fc60c3ad23e3052 |
| N0500 | a2r / 01_basics/004_doc_comments | crates/auto-lang/test/a2r/01_basics/004_doc_comments/doc_comments.case.ad | doc_comments.expected.rs | a1c643eb7fc612f2 | 898f363f6cc7f046 |
| N0501 | a2r / 02_types/001_struct | crates/auto-lang/test/a2r/02_types/001_struct/struct.case.ad | struct.expected.rs | f9cd4417d2b3d02b | b69d5fb7f7645842 |
| N0502 | a2r / 02_types/002_enum | crates/auto-lang/test/a2r/02_types/002_enum/enum.case.ad | enum.expected.rs | cc2fe5f4e01ad703 | 9556a3128aaf3ea9 |
| N0503 | a2r / 02_types/003_union | crates/auto-lang/test/a2r/02_types/003_union/union.case.ad | union.expected.rs | 84afb46f3ff75943 | 2203b7185ac8ec91 |
| N0504 | a2r / 02_types/004_pointer | crates/auto-lang/test/a2r/02_types/004_pointer/pointer.case.ad | pointer.expected.rs | 294918b3465d5ab2 | 0ab6d488f9e23e69 |
| N0505 | a2r / 02_types/005_inheritance | crates/auto-lang/test/a2r/02_types/005_inheritance/inheritance.case.ad | inheritance.expected.rs | b6db60f822b10317 | 342a3ac34d0a11bd |
| N0506 | a2r / 02_types/006_object | crates/auto-lang/test/a2r/02_types/006_object/object.case.ad | object.expected.rs | baf7a1635ad62700 | 5b893c426c512d0f |
| N0507 | a2r / 02_types/007_cstr | crates/auto-lang/test/a2r/02_types/007_cstr/cstr.case.ad | cstr.expected.rs | f40277b6e56f6cf5 | 21be9aa391c9763f |
| N0508 | a2r / 02_types/008_mut_self | crates/auto-lang/test/a2r/02_types/008_mut_self/mut_self.case.ad | mut_self.expected.rs | 01e2b0f9f4d4515e | b5c7429f309aa5a3 |
| N0509 | a2r / 02_types/009_self_field | crates/auto-lang/test/a2r/02_types/009_self_field/self_field.case.ad | self_field.expected.rs | 5a68a6a9f90a9a51 | e87523ebac8549ee |
| N0510 | a2r / 02_types/010_ext_keyword | crates/auto-lang/test/a2r/02_types/010_ext_keyword/ext_keyword.case.ad | ext_keyword.expected.rs | 72e66d0957c38796 | 3af346bfd8a8fd5f |
| N0511 | a2r / 02_types/011_param_auto_mut | crates/auto-lang/test/a2r/02_types/011_param_auto_mut/param_auto_mut.case.ad | param_auto_mut.expected.rs | 0fcd71ddc58d1921 | d3b2df1113fb7969 |
| N0512 | a2r / 03_control_flow/001_if_basic | crates/auto-lang/test/a2r/03_control_flow/001_if_basic/if_basic.case.ad | if_basic.expected.rs | 758be0d917092702 | cade4b3a0d372b8a |
| N0513 | a2r / 03_control_flow/002_if_nested | crates/auto-lang/test/a2r/03_control_flow/002_if_nested/if_nested.case.ad | if_nested.expected.rs | 1592ce319f0f4975 | 3af0b21181cc03fd |
| N0514 | a2r / 03_control_flow/003_if_multistmt | crates/auto-lang/test/a2r/03_control_flow/003_if_multistmt/if_multistmt.case.ad | if_multistmt.expected.rs | 0977bf265be6c67d | 3b2daeee7e46c7aa |
| N0515 | a2r / 03_control_flow/004_if_return | crates/auto-lang/test/a2r/03_control_flow/004_if_return/if_return.case.ad | if_return.expected.rs | 107b7968251d5350 | 468ad3d1afa59215 |
| N0516 | a2r / 03_control_flow/005_for_range | crates/auto-lang/test/a2r/03_control_flow/005_for_range/for_range.case.ad | for_range.expected.rs | 4e7f76e6a347be76 | bc831301f77e1f83 |
| N0517 | a2r / 03_control_flow/006_for_conditions | crates/auto-lang/test/a2r/03_control_flow/006_for_conditions/for_conditions.case.ad | for_conditions.expected.rs | 23f7e91365fcbf0e | 331f0305f0b449f2 |
| N0518 | a2r / 03_control_flow/007_while_loop | crates/auto-lang/test/a2r/03_control_flow/007_while_loop/while_loop.case.ad | while_loop.expected.rs | 3929d2152e313280 | 69ea0f44ea627511 |
| N0519 | a2r / 03_control_flow/008_is_match | crates/auto-lang/test/a2r/03_control_flow/008_is_match/is_match.case.ad | is_match.expected.rs | 6ff9d9e0732d1e7b | a646ec076b00b6f2 |
| N0520 | a2r / 03_control_flow/009_is_multi_stmt | crates/auto-lang/test/a2r/03_control_flow/009_is_multi_stmt/is_multi_stmt.case.ad | is_multi_stmt.expected.rs | 2ed471c107f19fe0 | 99e53d50deae047a |
| N0521 | a2r / 03_control_flow/010_is_non_exhaustive | crates/auto-lang/test/a2r/03_control_flow/010_is_non_exhaustive/is_non_exhaustive.case.ad | is_non_exhaustive.expected.rs | 86bc26fa1440a1c6 | 05afac3224a0208a |
| N0522 | a2r / 03_control_flow/013_if_tail_value | crates/auto-lang/test/a2r/03_control_flow/013_if_tail_value/if_tail_value.case.ad | if_tail_value.expected.rs | ccd069e512d69942 | 08b62620ad7dd73d |
| N0523 | a2r / 04_strings/001_fstring | crates/auto-lang/test/a2r/04_strings/001_fstring/fstring.case.ad | fstring.expected.rs | 5e2f064d4d5cf6c7 | 9c210b1d732befdc |
| N0524 | a2r / 04_strings/002_fstring_edge | crates/auto-lang/test/a2r/04_strings/002_fstring_edge/fstring_edge.case.ad | fstring_edge.expected.rs | df23eb91d7e8ea4a | 7a0346de11522367 |
| N0525 | a2r / 04_strings/003_multi_str | crates/auto-lang/test/a2r/04_strings/003_multi_str/multi_str.case.ad | multi_str.expected.rs | 983ba239308d7ce4 | ea93bb0065d18af6 |
| N0526 | a2r / 04_strings/004_backtick_string | crates/auto-lang/test/a2r/04_strings/004_backtick_string/backtick_string.case.ad | backtick_string.expected.rs | d9e33430123ce9bb | a8ad96d9f21afcfe |
| N0527 | a2r / 04_strings/005_escaped_quotes | crates/auto-lang/test/a2r/04_strings/005_escaped_quotes/escaped_quotes.case.ad | escaped_quotes.expected.rs | fbae2871b57acad3 | 8c3ee7fd39a6db46 |
| N0528 | a2r / 04_strings/006_multi_fstr | crates/auto-lang/test/a2r/04_strings/006_multi_fstr/multi_fstr.case.ad | multi_fstr.expected.rs | 7bc018282ed7fb22 | b331bfcc42f9aed4 |
| N0529 | a2r / 04_strings/007_char_at_infer | crates/auto-lang/test/a2r/04_strings/007_char_at_infer/char_at_infer.case.ad | char_at_infer.expected.rs | b9cbf6594b614c4c | 36e99e086db2091b |
| N0530 | a2r / 04_strings/008_str_param_borrow | crates/auto-lang/test/a2r/04_strings/008_str_param_borrow/str_param_borrow.case.ad | str_param_borrow.expected.rs | 1132dfc23d502793 | bcba810bcfb9562e |
| N0531 | a2r / 04_strings/009_asstr_arg_guards | crates/auto-lang/test/a2r/04_strings/009_asstr_arg_guards/asstr_arg_guards.case.ad | asstr_arg_guards.expected.rs | 0108f70dd9cc23f7 | 2824fee58a65e0d4 |
| N0532 | a2r / 04_strings/010_print_concat | crates/auto-lang/test/a2r/04_strings/010_print_concat/print_concat.case.ad | print_concat.expected.rs | 6f8de1ea3c3d0a85 | bb8a76c58f675368 |
| N0533 | a2r / 05_expressions/001_arithmetic | crates/auto-lang/test/a2r/05_expressions/001_arithmetic/arithmetic.case.ad | arithmetic.expected.rs | f9b5dc0e0556cf80 | 32d94463a892d2e0 |
| N0534 | a2r / 05_expressions/002_unary | crates/auto-lang/test/a2r/05_expressions/002_unary/unary.case.ad | unary.expected.rs | 087cd335f2efe3f5 | f0ded29d08db25e0 |
| N0535 | a2r / 05_expressions/003_indexing | crates/auto-lang/test/a2r/05_expressions/003_indexing/indexing.case.ad | indexing.expected.rs | 5a647b20a244506a | 39d6c30805b2a2f3 |
| N0536 | a2r / 05_expressions/004_blocks | crates/auto-lang/test/a2r/05_expressions/004_blocks/blocks.case.ad | blocks.expected.rs | 3ecf9258b5745026 | 68ce7bde002a6ca0 |
| N0537 | a2r / 05_expressions/005_ref_expr | crates/auto-lang/test/a2r/05_expressions/005_ref_expr/ref_expr.case.ad | ref_expr.expected.rs | 6565630f2147c819 | 8337b71dd0afa8c0 |
| N0538 | a2r / 05_expressions/006_range_expr | crates/auto-lang/test/a2r/05_expressions/006_range_expr/range_expr.case.ad | range_expr.expected.rs | 47bdcab3d57feb83 | c74f12ab8da3a086 |
| N0539 | a2r / 05_expressions/007_composition | crates/auto-lang/test/a2r/05_expressions/007_composition/composition.case.ad | composition.expected.rs | b0f8a7c167791e3d | 2daf6f987760f682 |
| N0540 | a2r / 05_expressions/008_field_composition | crates/auto-lang/test/a2r/05_expressions/008_field_composition/field_composition.case.ad | field_composition.expected.rs | b14b0a94ba0cfea6 | f8226e47321e53ad |
| N0541 | a2r / 05_expressions/009_comprehensive | crates/auto-lang/test/a2r/05_expressions/009_comprehensive/comprehensive.case.ad | comprehensive.expected.rs | ceb755d731e6877d | 3fe13e8d8fa792e4 |
| N0542 | a2r / 05_expressions/010_or_keyword | crates/auto-lang/test/a2r/05_expressions/010_or_keyword/or_keyword.case.ad | or_keyword.expected.rs | 823bb92bffaa8e53 | cf463fa63f0c66c4 |
| N0543 | a2r / 05_expressions/011_no_left_shift | crates/auto-lang/test/a2r/05_expressions/011_no_left_shift/no_left_shift.case.ad | no_left_shift.expected.rs | 8b21319816a23dde | 19b26df7965241e7 |
| N0544 | a2r / 05_expressions/012_turbofish | crates/auto-lang/test/a2r/05_expressions/012_turbofish/turbofish.case.ad | turbofish.expected.rs | db57b722c57db2bf | eecdaa38f5300004 |
| N0545 | a2r / 06_pattern_matching/001_enum_pattern | crates/auto-lang/test/a2r/06_pattern_matching/001_enum_pattern/enum_pattern.case.ad | enum_pattern.expected.rs | 557252acffe38b7b | 95984d82d3f2da8a |
| N0546 | a2r / 06_pattern_matching/002_struct_destructure | crates/auto-lang/test/a2r/06_pattern_matching/002_struct_destructure/struct_destructure.case.ad | struct_destructure.expected.rs | f5f66820acc6cb40 | 126758f5b55822b3 |
| N0547 | a2r / 06_pattern_matching/003_empty_variant_match | crates/auto-lang/test/a2r/06_pattern_matching/003_empty_variant_match/empty_variant_match.case.ad | empty_variant_match.expected.rs | 19c73d67de70b49b | 0d5eee7e172b74af |
| N0548 | a2r / 06_pattern_matching/004_hetero_enum | crates/auto-lang/test/a2r/06_pattern_matching/004_hetero_enum/hetero_enum.case.ad | hetero_enum.expected.rs | cd8d3f1b53f52ffc | ab66c669a7bf75c7 |
| N0549 | a2r / 06_pattern_matching/005_generic_hetero_enum | crates/auto-lang/test/a2r/06_pattern_matching/005_generic_hetero_enum/generic_hetero_enum.case.ad | generic_hetero_enum.expected.rs | 54db207a80c1b4c9 | 825751645120f5df |
| N0550 | a2r / 06_pattern_matching/006_enum_fn_param | crates/auto-lang/test/a2r/06_pattern_matching/006_enum_fn_param/enum_fn_param.case.ad | enum_fn_param.expected.rs | 3bc86175f509d47a | e20380fed09b6706 |
| N0551 | a2r / 06_pattern_matching/007_is_in_ext | crates/auto-lang/test/a2r/06_pattern_matching/007_is_in_ext/is_in_ext.case.ad | is_in_ext.expected.rs | 390735f27a266232 | 6dafbbe8902dcf2d |
| N0552 | a2r / 06_pattern_matching/008_hetero_enum_multistmt | crates/auto-lang/test/a2r/06_pattern_matching/008_hetero_enum_multistmt/hetero_enum_multistmt.case.ad | hetero_enum_multistmt.expected.rs | c7a9b4254b982b16 | e1cf95f94c6178df |
| N0553 | a2r / 06_pattern_matching/009_qualified_unit_variant | crates/auto-lang/test/a2r/06_pattern_matching/009_qualified_unit_variant/qualified_unit_variant.case.ad | qualified_unit_variant.expected.rs | f36a75362fef41d2 | 3eac2f072ff422aa |
| N0554 | a2r / 07_ownership/001_borrow_view | crates/auto-lang/test/a2r/07_ownership/001_borrow_view/borrow_view.case.ad | borrow_view.expected.rs | 21bac3cdda1069dc | c2aa0664eff3119e |
| N0555 | a2r / 07_ownership/002_borrow_mut | crates/auto-lang/test/a2r/07_ownership/002_borrow_mut/borrow_mut.case.ad | borrow_mut.expected.rs | 8e2bdb8ad9d7eac6 | 0601b6815af8c80b |
| N0556 | a2r / 07_ownership/003_borrow_move | crates/auto-lang/test/a2r/07_ownership/003_borrow_move/borrow_move.case.ad | borrow_move.expected.rs | b8b2e0172804353e | 55d775ed86923180 |
| N0557 | a2r / 07_ownership/004_borrow_conflicts | crates/auto-lang/test/a2r/07_ownership/004_borrow_conflicts/borrow_conflicts.case.ad | borrow_conflicts.expected.rs | 389865bc81f6e71f | 9fe562937d65613d |
| N0558 | a2r / 07_ownership/026_get_ref_binding | crates/auto-lang/test/a2r/07_ownership/026_get_ref_binding/get_ref_binding.case.ad | get_ref_binding.expected.rs | 2b7f05e37f101371 | 3a0047c1a921fee0 |
| N0559 | a2r / 08_generics/001_type_alias | crates/auto-lang/test/a2r/08_generics/001_type_alias/type_alias.case.ad | type_alias.expected.rs | a0b8d95fd8279fdb | 4a78b12d89bf0822 |
| N0560 | a2r / 08_generics/002_const_generics | crates/auto-lang/test/a2r/08_generics/002_const_generics/const_generics.case.ad | const_generics.expected.rs | 9b6677412c5b2e08 | 922a7c1198ff45ea |
| N0561 | a2r / 08_generics/003_generic_field | crates/auto-lang/test/a2r/08_generics/003_generic_field/generic_field.case.ad | generic_field.expected.rs | 0f2005f71a54e124 | 20e0c202cfec943c |
| N0562 | a2r / 08_generics/004_generic_ptr_field | crates/auto-lang/test/a2r/08_generics/004_generic_ptr_field/generic_ptr_field.case.ad | generic_ptr_field.expected.rs | 51c01b4a1916ec68 | d857abf5071dc9bb |
| N0563 | a2r / 08_generics/005_with_constraint | crates/auto-lang/test/a2r/08_generics/005_with_constraint/with_constraint.case.ad | with_constraint.expected.rs | 275f46eef080bc41 | 77c58517a059064c |
| N0564 | a2r / 08_generics/006_map_type | crates/auto-lang/test/a2r/08_generics/006_map_type/map_type.case.ad | map_type.expected.rs | af8f8bc8b601538c | de4f08b07b15d239 |
| N0565 | a2r / 08_generics/007_no_tuple_generic | crates/auto-lang/test/a2r/08_generics/007_no_tuple_generic/no_tuple_generic.case.ad | no_tuple_generic.expected.rs | 7694b5d782b76042 | 74c77fcccbac581d |
| N0566 | a2r / 08_generics/008_bounded_generic_fn | crates/auto-lang/test/a2r/08_generics/008_bounded_generic_fn/bounded_generic_fn.case.ad | bounded_generic_fn.expected.rs | 2a0c8f5fa3e06ee1 | 41b87b1200205498 |
| N0567 | a2r / 09_option_result/001_option | crates/auto-lang/test/a2r/09_option_result/001_option/option.case.ad | option.expected.rs | d918389cdb080e7d | 43cc30921906fa5d |
| N0568 | a2r / 09_option_result/002_option_construct | crates/auto-lang/test/a2r/09_option_result/002_option_construct/option_construct.case.ad | option_construct.expected.rs | 49d7123787dbf92c | 39350487dbf6dbc3 |
| N0569 | a2r / 09_option_result/003_null_coalesce | crates/auto-lang/test/a2r/09_option_result/003_null_coalesce/null_coalesce.case.ad | null_coalesce.expected.rs | ec18a9c14fd5ae37 | 9fcc7f3b254e4fb8 |
| N0570 | a2r / 09_option_result/004_error_propagate | crates/auto-lang/test/a2r/09_option_result/004_error_propagate/error_propagate.case.ad | error_propagate.expected.rs | 9d9826aac162eb18 | 60f9406c40ea1366 |
| N0571 | a2r / 09_option_result/005_question_uint | crates/auto-lang/test/a2r/09_option_result/005_question_uint/question_uint.case.ad | question_uint.expected.rs | f5adcadd2df787bc | 22a5b15a675c5508 |
| N0572 | a2r / 09_option_result/006_question_float | crates/auto-lang/test/a2r/09_option_result/006_question_float/question_float.case.ad | question_float.expected.rs | bae3e76839379e2e | de8f06508ba3e5aa |
| N0573 | a2r / 09_option_result/007_question_double | crates/auto-lang/test/a2r/09_option_result/007_question_double/question_double.case.ad | question_double.expected.rs | e1af3015933876b4 | 7e7720512f4e4902 |
| N0574 | a2r / 09_option_result/008_question_return_int | crates/auto-lang/test/a2r/09_option_result/008_question_return_int/question_return_int.case.ad | question_return_int.expected.rs | ce969fe1a86ee7e6 | 54fe671e5444f8f1 |
| N0575 | a2r / 09_option_result/009_question_return_str | crates/auto-lang/test/a2r/09_option_result/009_question_return_str/question_return_str.case.ad | question_return_str.expected.rs | 1d2dd40ae12f9e65 | bb5b93eac2d5dbd8 |
| N0576 | a2r / 09_option_result/010_question_return_bool | crates/auto-lang/test/a2r/09_option_result/010_question_return_bool/question_return_bool.case.ad | question_return_bool.expected.rs | 7cb06237b9ee97df | 62e8d60126fa21ff |
| N0577 | a2r / 09_option_result/011_question_propagate | crates/auto-lang/test/a2r/09_option_result/011_question_propagate/question_propagate.case.ad | question_propagate.expected.rs | 652f63109d322252 | 63fa9371877cceb8 |
| N0578 | a2r / 09_option_result/012_question_return_float | crates/auto-lang/test/a2r/09_option_result/012_question_return_float/question_return_float.case.ad | question_return_float.expected.rs | bae3e76839379e2e | 38cc385cb44289fa |
| N0579 | a2r / 09_option_result/013_question_return_double | crates/auto-lang/test/a2r/09_option_result/013_question_return_double/question_return_double.case.ad | question_return_double.expected.rs | 006c0abb6d561476 | 019ee5437bcd2f0f |
| N0580 | a2r / 09_option_result/014_question_return_char | crates/auto-lang/test/a2r/09_option_result/014_question_return_char/question_return_char.case.ad | question_return_char.expected.rs | c6c83b9955a45862 | 0fe794f4610e149a |
| N0581 | a2r / 09_option_result/015_question_return_uint | crates/auto-lang/test/a2r/09_option_result/015_question_return_uint/question_return_uint.case.ad | question_return_uint.expected.rs | 3a7ece8a21c8c4b5 | ea73066bb9207fb7 |
| N0582 | a2r / 09_option_result/016_question_return_float_v2 | crates/auto-lang/test/a2r/09_option_result/016_question_return_float_v2/question_return_float_v2.case.ad | question_return_float_v2.expected.rs | 83d383184b41dab2 | f9134c6929e83af6 |
| N0583 | a2r / 09_option_result/017_question_return_double_v2 | crates/auto-lang/test/a2r/09_option_result/017_question_return_double_v2/question_return_double_v2.case.ad | question_return_double_v2.expected.rs | 8784b77e14ca21ee | ff2e76b61cd5a873 |
| N0584 | a2r / 09_option_result/018_question_return_char_v2 | crates/auto-lang/test/a2r/09_option_result/018_question_return_char_v2/question_return_char_v2.case.ad | question_return_char_v2.expected.rs | df16d3acc9da7c15 | 2a6fdb6e1dc2c623 |
| N0585 | a2r / 09_option_result/019_question_nested_call | crates/auto-lang/test/a2r/09_option_result/019_question_nested_call/question_nested_call.case.ad | question_nested_call.expected.rs | 7ff531e7c287799f | 7cd1103b95166824 |
| N0586 | a2r / 09_option_result/020_question_arithmetic | crates/auto-lang/test/a2r/09_option_result/020_question_arithmetic/question_arithmetic.case.ad | question_arithmetic.expected.rs | e520c081a0e03bb3 | eb70b9b20fdfd70d |
| N0587 | a2r / 09_option_result/021_question_comparison | crates/auto-lang/test/a2r/09_option_result/021_question_comparison/question_comparison.case.ad | question_comparison.expected.rs | 9197474657f7137d | 40b1beb96f753e91 |
| N0588 | a2r / 09_option_result/022_question_literal | crates/auto-lang/test/a2r/09_option_result/022_question_literal/question_literal.case.ad | question_literal.expected.rs | 6124b93675cf2008 | 00287eb00af498d6 |
| N0589 | a2r / 09_option_result/023_question_negation | crates/auto-lang/test/a2r/09_option_result/023_question_negation/question_negation.case.ad | question_negation.expected.rs | b641985c3ac9a319 | f438c594c604d6b8 |
| N0590 | a2r / 09_option_result/024_question_zero | crates/auto-lang/test/a2r/09_option_result/024_question_zero/question_zero.case.ad | question_zero.expected.rs | ce3c1a868f5411bb | 8d11782595eec9ed |
| N0591 | a2r / 09_option_result/025_question_negative | crates/auto-lang/test/a2r/09_option_result/025_question_negative/question_negative.case.ad | question_negative.expected.rs | f81007f62fc3c621 | 6cbc64aa2580a3b2 |
| N0592 | a2r / 09_option_result/026_list_basic | crates/auto-lang/test/a2r/09_option_result/026_list_basic/list_basic.case.ad | list_basic.expected.rs | 1e712469aaa52db3 | 39de2f6cacaa0d83 |
| N0593 | a2r / 09_option_result/027_list_methods | crates/auto-lang/test/a2r/09_option_result/027_list_methods/list_methods.case.ad | list_methods.expected.rs | de80161fd4c96e81 | 6c208425b42e335e |
| N0594 | a2r / 09_option_result/028_list_may | crates/auto-lang/test/a2r/09_option_result/028_list_may/list_may.case.ad | list_may.expected.rs | a9a33b3421ba1238 | b69791bd4fdd618f |
| N0595 | a2r / 09_option_result/029_list_propagate | crates/auto-lang/test/a2r/09_option_result/029_list_propagate/list_propagate.case.ad | list_propagate.expected.rs | 50b5766aade18c62 | 6dc2e7200be77fbe |
| N0596 | a2r / 09_option_result/030_list_coalesce | crates/auto-lang/test/a2r/09_option_result/030_list_coalesce/list_coalesce.case.ad | list_coalesce.expected.rs | 8b4c3ae0ffd87523 | 04eb84678f481179 |
| N0597 | a2r / 09_option_result/031_option_bool_field | crates/auto-lang/test/a2r/09_option_result/031_option_bool_field/option_bool_field.case.ad | option_bool_field.expected.rs | d14f0e0632576442 | 2b94b9109adc1d11 |
| N0598 | a2r / 09_option_result/032_fn_result_enum | crates/auto-lang/test/a2r/09_option_result/032_fn_result_enum/fn_result_enum.case.ad | fn_result_enum.expected.rs | ef3ae55b7f0ab29a | 18b10d9980e03af6 |
| N0599 | a2r / 09_option_result/033_result_is_match | crates/auto-lang/test/a2r/09_option_result/033_result_is_match/result_is_match.case.ad | result_is_match.expected.rs | a81371f68cd0e27c | f3c64eb3f513723b |
| N0600 | a2r / 09_option_result/034_result_bang_type | crates/auto-lang/test/a2r/09_option_result/034_result_bang_type/result_bang_type.case.ad | result_bang_type.expected.rs | 6d2e8599142ac53d | b5263d7c51998632 |
| N0601 | a2r / 10_collections/001_array | crates/auto-lang/test/a2r/10_collections/001_array/array.case.ad | array.expected.rs | ce3358240a282670 | aa8be9acbc0c943c |
| N0602 | a2r / 10_collections/002_list_storage | crates/auto-lang/test/a2r/10_collections/002_list_storage/list_storage.case.ad | list_storage.expected.rs | 7ec2d57aeb1249e1 | 2bfd40d382fca9e6 |
| N0603 | a2r / 10_collections/003_map_func | crates/auto-lang/test/a2r/10_collections/003_map_func/map_func.case.ad | map_func.expected.rs | c12e96bb970c442d | 739c6d7fbc6c5601 |
| N0604 | a2r / 10_collections/004_list_as_cast | crates/auto-lang/test/a2r/10_collections/004_list_as_cast/list_as_cast.case.ad | list_as_cast.expected.rs | a0ce82f7c5ce5370 | c6c7d8d146b9a189 |
| N0605 | a2r / 10_collections/005_method_chain | crates/auto-lang/test/a2r/10_collections/005_method_chain/method_chain.case.ad | method_chain.expected.rs | b30b48e2c13dd60b | 699af73f4557cd6a |
| N0606 | a2r / 10_collections/006_map_literal | crates/auto-lang/test/a2r/10_collections/006_map_literal/map_literal.case.ad | map_literal.expected.rs | 7fd015fd1f0a486e | ceba90fbe409c097 |
| N0607 | a2r / 10_collections/007_list_new | crates/auto-lang/test/a2r/10_collections/007_list_new/list_new.case.ad | list_new.expected.rs | 4e3f7d1b9d567638 | 453f1c9e89f49f31 |
| N0608 | a2r / 10_collections/008_list_set_index | crates/auto-lang/test/a2r/10_collections/008_list_set_index/list_set_index.case.ad | list_set_index.expected.rs | 0349eee16fe0904f | 47917a76a141d241 |
| N0609 | a2r / 10_collections/009_global_list_set | crates/auto-lang/test/a2r/10_collections/009_global_list_set/global_list_set.case.ad | global_list_set.expected.rs | a017d172c1632bf9 | 9d5f96d7da284f0a |
| N0610 | a2r / 11_methods/001_method | crates/auto-lang/test/a2r/11_methods/001_method/method.case.ad | method.expected.rs | ebf4ebce25f02384 | 9f2d5b1d0358aa00 |
| N0611 | a2r / 11_methods/002_struct_methods | crates/auto-lang/test/a2r/11_methods/002_struct_methods/struct_methods.case.ad | struct_methods.expected.rs | 42bf2bacc0266982 | b4e1489d4026a3ff |
| N0612 | a2r / 11_methods/003_closure | crates/auto-lang/test/a2r/11_methods/003_closure/closure.case.ad | closure.expected.rs | 9e41232ab6389fde | b9008d3d15d5be69 |
| N0613 | a2r / 11_methods/004_closure_infer | crates/auto-lang/test/a2r/11_methods/004_closure_infer/closure_infer.case.ad | closure_infer.expected.rs | 6650a7aaf50fbb75 | 5fff1937e6fbc042 |
| N0614 | a2r / 11_methods/004_static_fn | crates/auto-lang/test/a2r/11_methods/004_static_fn/static_fn.case.ad | static_fn.expected.rs | 88a04ce0058162be | 54d94597093b57a5 |
| N0615 | a2r / 11_methods/005_func_literal_return | crates/auto-lang/test/a2r/11_methods/005_func_literal_return/func_literal_return.case.ad | func_literal_return.expected.rs | e31cc3096902ec0b | 3d26b3a9d1d7f330 |
| N0616 | a2r / 11_methods/006_ext_for | crates/auto-lang/test/a2r/11_methods/006_ext_for/ext_for.case.ad | ext_for.expected.rs | 2bdbe36e6c54749f | 8ce62c8e4802ed7f |
| N0617 | a2r / 11_methods/007_ext_from | crates/auto-lang/test/a2r/11_methods/007_ext_from/ext_from.case.ad | ext_from.expected.rs | 02442b07b02ff484 | 9bb9308129f78b59 |
| N0618 | a2r / 11_methods/008_empty_body | crates/auto-lang/test/a2r/11_methods/008_empty_body/empty_body.case.ad | empty_body.expected.rs | dacd99d8b0c0c665 | 68aa041c07f871a6 |
| N0619 | a2r / 11_methods/009_ext_const | crates/auto-lang/test/a2r/11_methods/009_ext_const/ext_const.case.ad | ext_const.expected.rs | 969331d428346ad8 | 082f77db68629ce5 |
| N0620 | a2r / 12_specs/001_basic_spec | crates/auto-lang/test/a2r/12_specs/001_basic_spec/basic_spec.case.ad | basic_spec.expected.rs | 079f395f7293557f | 53f049b4c6e69e3f |
| N0621 | a2r / 12_specs/002_spec | crates/auto-lang/test/a2r/12_specs/002_spec/spec.case.ad | spec.expected.rs | 68526d5032ec50fd | 88cfa6dab17943f2 |
| N0622 | a2r / 12_specs/003_spec_delegation | crates/auto-lang/test/a2r/12_specs/003_spec_delegation/spec_delegation.case.ad | spec_delegation.expected.rs | 68526d5032ec50fd | 5a71b60798036cbc |
| N0623 | a2r / 12_specs/004_default_body | crates/auto-lang/test/a2r/12_specs/004_default_body/default_body.case.ad | default_body.expected.rs | 0d83da04c5a73c28 | 748fd3264f59c732 |
| N0624 | a2r / 12_specs/005_generic_impl | crates/auto-lang/test/a2r/12_specs/005_generic_impl/generic_impl.case.ad | generic_impl.expected.rs | 53ed174a871d9454 | 4ae117bfe1740182 |
| N0625 | a2r / 12_specs/006_spec_param_callsite | crates/auto-lang/test/a2r/12_specs/006_spec_param_callsite/spec_param_callsite.case.ad | spec_param_callsite.expected.rs | 374f9834606d7b62 | 9a503f9850fee82f |
| N0626 | a2r / 12_specs/007_box_fn | crates/auto-lang/test/a2r/12_specs/007_box_fn/box_fn.case.ad | box_fn.expected.rs | 74070a8eda55457b | 6a447737ea63f2cf |
| N0627 | a2r / 12_specs/008_arc_dyn_spec | crates/auto-lang/test/a2r/12_specs/008_arc_dyn_spec/arc_dyn_spec.case.ad | arc_dyn_spec.expected.rs | a0a596bd8e71cd6e | e8aaad80b39ac256 |
| N0628 | a2r / 12_specs/009_arc_fn_spec_param | crates/auto-lang/test/a2r/12_specs/009_arc_fn_spec_param/arc_fn_spec_param.case.ad | arc_fn_spec_param.expected.rs | 8691554d97d0b8f1 | dd56fc8fbbdd859e |
| N0629 | a2r / 12_specs/010_spec_supertrait | crates/auto-lang/test/a2r/12_specs/010_spec_supertrait/spec_supertrait.case.ad | spec_supertrait.expected.rs | 4817f785afbbf2e9 | a1205f871726dc17 |
| N0630 | a2r / 12_specs/011_associated_types | crates/auto-lang/test/a2r/12_specs/011_associated_types/associated_types.case.ad | associated_types.expected.rs | 40ee91cd1f2ca2f0 | 2e709f1003ebe132 |
| N0631 | a2r / 12_specs/012_generic_spec_assoc | crates/auto-lang/test/a2r/12_specs/012_generic_spec_assoc/generic_spec_assoc.case.ad | generic_spec_assoc.expected.rs | ac2c3bbfba5ca2e8 | afeb1c3e52c8678c |
| N0632 | a2r / 12_specs/013_generic_spec_typed_impl | crates/auto-lang/test/a2r/12_specs/013_generic_spec_typed_impl/generic_spec_typed_impl.case.ad | generic_spec_typed_impl.expected.rs | 76065d4132a84eb9 | 2b62b9722cdb3bcc |
| N0633 | a2r / 13_delegation/001_single | crates/auto-lang/test/a2r/13_delegation/001_single/single.case.ad | single.expected.rs | caeb1132d29277cb | cab8f6aac301df96 |
| N0634 | a2r / 13_delegation/002_multi_spec | crates/auto-lang/test/a2r/13_delegation/002_multi_spec/multi_spec.case.ad | multi_spec.expected.rs | e42cd4181e61399b | 03e14d3d6f20e4f4 |
| N0635 | a2r / 13_delegation/003_multi_delegation | crates/auto-lang/test/a2r/13_delegation/003_multi_delegation/multi_delegation.case.ad | multi_delegation.expected.rs | cbf36b9df6042602 | 342f948362c81d30 |
| N0636 | a2r / 14_modules/001_rust_use | crates/auto-lang/test/a2r/14_modules/001_rust_use/rust_use.case.ad | rust_use.expected.rs | 080fd5f7f7e9546f | 37249bfa91e479d7 |
| N0637 | a2r / 14_modules/002_pub_use | crates/auto-lang/test/a2r/14_modules/002_pub_use/pub_use.case.ad | pub_use.expected.rs | 6d4a93cc073f9ecf | 5b141eb199674510 |
| N0638 | a2r / 14_modules/003_pub_visibility | crates/auto-lang/test/a2r/14_modules/003_pub_visibility/pub_visibility.case.ad | pub_visibility.expected.rs | 5e432087d3485e5a | 1c09fdee58cf56c1 |
| N0639 | a2r / 14_modules/004_wildcard_import | crates/auto-lang/test/a2r/14_modules/004_wildcard_import/wildcard_import.case.ad | wildcard_import.expected.rs | 2d9c868b5d323a2e | 0d8da65f59915b41 |
| N0640 | a2r / 14_modules/006_const_decl | crates/auto-lang/test/a2r/14_modules/006_const_decl/const_decl.case.ad | const_decl.expected.rs | 0bf560cd496de36b | ed46255a56e2c66c |
| N0641 | a2r / 14_modules/007_shared_var | crates/auto-lang/test/a2r/14_modules/007_shared_var/shared_var.case.ad | shared_var.expected.rs | 74f23f690bc83c7a | 10460cf22b395ae3 |
| N0642 | a2r / 14_modules/008_derive_attr | crates/auto-lang/test/a2r/14_modules/008_derive_attr/derive_attr.case.ad | derive_attr.expected.rs | ad74f2d52cdb56c3 | 4408a59b0af30098 |
| N0643 | a2r / 14_modules/009_const_before_ext | crates/auto-lang/test/a2r/14_modules/009_const_before_ext/const_before_ext.case.ad | const_before_ext.expected.rs | 2cac6feb8318ac3a | c283bd922fc715f6 |
| N0644 | a2r / 14_modules/010_use_rs | crates/auto-lang/test/a2r/14_modules/010_use_rs/use_rs.case.ad | use_rs.expected.rs | 7c49efd69ffb3ca4 | 873a68dd50141a1d |
| N0645 | a2r / 15_type_conversion/001_type_cast | crates/auto-lang/test/a2r/15_type_conversion/001_type_cast/type_cast.case.ad | type_cast.expected.rs | 3e5eed01ff5936db | 79c576c652a2c9de |
| N0646 | a2r / 15_type_conversion/002_to_convert | crates/auto-lang/test/a2r/15_type_conversion/002_to_convert/to_convert.case.ad | to_convert.expected.rs | 20f66ce1218acd3e | 253db6cb2fc9745e |
| N0647 | a2r / 15_type_conversion/003_ptr_methods | crates/auto-lang/test/a2r/15_type_conversion/003_ptr_methods/ptr_methods.case.ad | ptr_methods.expected.rs | 578ce7ee03916c28 | fe88381f35824eb8 |
| N0648 | a2r / 15_type_conversion/004_box_arc | crates/auto-lang/test/a2r/15_type_conversion/004_box_arc/box_arc.case.ad | box_arc.expected.rs | 50c2f0302ac0acfe | 6c0eb5e5ca90c160 |
| N0649 | a2r / 16_interop/001_async_fn | crates/auto-lang/test/a2r/16_interop/001_async_fn/async_fn.case.ad | async_fn.expected.rs | 4c7f940b703497c8 | 2503a87668620b79 |
| N0650 | a2r / 16_interop/002_tokio_main | crates/auto-lang/test/a2r/16_interop/002_tokio_main/tokio_main.case.ad | tokio_main.expected.rs | 02be8960e6980401 | c3366c40c3f7f642 |
| N0651 | a2r / 16_interop/003_field_attrs | crates/auto-lang/test/a2r/16_interop/003_field_attrs/field_attrs.case.ad | field_attrs.expected.rs | 0427d443a8401622 | a45447fd7ae3601f |
| N0652 | a2r / 16_interop/004_variant_from_attr | crates/auto-lang/test/a2r/16_interop/004_variant_from_attr/variant_from_attr.case.ad | variant_from_attr.expected.rs | b6ef682514a3f008 | 1f6b30899f7d6461 |
| N0653 | a2r / 16_interop/015_impl_trait_return | crates/auto-lang/test/a2r/16_interop/015_impl_trait_return/impl_trait_return.case.ad | impl_trait_return.expected.rs | 0b8091bca43ab74c | 40b687fec4e1741d |
| N0654 | a2r / 16_interop/016_extractor_destructure | crates/auto-lang/test/a2r/16_interop/016_extractor_destructure/extractor_destructure.case.ad | extractor_destructure.expected.rs | 1298915e5b3c647d | 3cd1a7a5568c40d1 |
| N0655 | a2r / 16_interop/017_comptime_read_text | crates/auto-lang/test/a2r/16_interop/017_comptime_read_text/comptime_read_text.case.ad | comptime_read_text.expected.rs | aa094e0629cbd6a8 | 82912599289adb5b |
| N0656 | a2r / 16_interop/018_dotted_attrs | crates/auto-lang/test/a2r/16_interop/018_dotted_attrs/dotted_attrs.case.ad | dotted_attrs.expected.rs | 0e87cf12ee851330 | 8e6d72cbd430ab7f |
| N0657 | a2r / 16_interop/019_multi_bound | crates/auto-lang/test/a2r/16_interop/019_multi_bound/multi_bound.case.ad | multi_bound.expected.rs | 8438223c15e596c1 | d024dd2e91809f77 |
| N0658 | a2r / 16_interop/020_async_block_stmts | crates/auto-lang/test/a2r/16_interop/020_async_block_stmts/async_block_stmts.case.ad | async_block_stmts.expected.rs | 9c9c38d3e101383b | 34ee85067e147e24 |
| N0659 | a2r / 16_interop/022_imported_concrete_return | crates/auto-lang/test/a2r/16_interop/022_imported_concrete_return/imported_concrete_return.case.ad | imported_concrete_return.expected.rs | 7d8792e407877dff | 27aee81c5135e57f |
| N0660 | a2r / 16_interop/023_unit_none_return | crates/auto-lang/test/a2r/16_interop/023_unit_none_return/unit_none_return.case.ad | unit_none_return.expected.rs | fde80a7b40a1d56a | 65aaaf2aa2c07056 |
| N0661 | a2r / 16_interop/024_nested_async_await | crates/auto-lang/test/a2r/16_interop/024_nested_async_await/nested_async_await.case.ad | nested_async_await.expected.rs | 16aa27d4daf2e514 | 486c9b99310336f2 |
| N0662 | a2r / 17_rust_std/001_collections | crates/auto-lang/test/a2r/17_rust_std/001_collections/collections.case.ad | collections.expected.rs | d9b9de86fefd3dd3 | 4ebd897ce48cd7b1 |
| N0663 | a2r / 17_rust_std/002_fs | crates/auto-lang/test/a2r/17_rust_std/002_fs/fs.case.ad | fs.expected.rs | 5dbb7c4773cf7786 | d5319eae2ec7f3e9 |
| N0664 | a2r / 17_rust_std/003_sync | crates/auto-lang/test/a2r/17_rust_std/003_sync/sync.case.ad | sync.expected.rs | 18b4a5bdc887a30b | 392c6b0bdec5e0e4 |
| N0665 | a2r / 17_rust_std/004_time | crates/auto-lang/test/a2r/17_rust_std/004_time/time.case.ad | time.expected.rs | 7d22a3d9e76819ce | fe89eea666c48a5b |
| N0666 | a2r / 17_rust_std/005_path | crates/auto-lang/test/a2r/17_rust_std/005_path/path.case.ad | path.expected.rs | 3cee0b9ae83eb3eb | 4ff0f02c5715eb1b |
| N0667 | a2r / 17_rust_std/006_box_cell | crates/auto-lang/test/a2r/17_rust_std/006_box_cell/box_cell.case.ad | box_cell.expected.rs | c595ca8ad9358c84 | a98aa0bdc48569fb |
| N0668 | a2r / 17_rust_std/007_env_process | crates/auto-lang/test/a2r/17_rust_std/007_env_process/env_process.case.ad | env_process.expected.rs | 1bc6014485f271a3 | a0537602eff37d2b |
| N0669 | a2r / 17_rust_std/008_thread | crates/auto-lang/test/a2r/17_rust_std/008_thread/thread.case.ad | thread.expected.rs | 6084d8bf26a6cb9e | b4b9b3a196260671 |
| N0670 | a2r / 17_rust_std/009_serde_json | crates/auto-lang/test/a2r/17_rust_std/009_serde_json/serde_json.case.ad | serde_json.expected.rs | c7cc8b1457972037 | 7be5b1410efe5ab4 |
| N0671 | a2r / 17_rust_std/010_regex | crates/auto-lang/test/a2r/17_rust_std/010_regex/regex.case.ad | regex.expected.rs | 1922e2641933f03e | f4804b2e6af8c885 |
| N0672 | a2r / 17_rust_std/011_http_download_upload | crates/auto-lang/test/a2r/17_rust_std/011_http_download_upload/http_download_upload.case.ad | http_download_upload.expected.rs | d8550ee3a66c2e71 | 6a6c73dd376cdd9b |
| N0673 | a2r / 17_rust_std/011_math | crates/auto-lang/test/a2r/17_rust_std/011_math/math.case.ad | math.expected.rs | 1cc1c9c04cd5f864 | 3fcbaeb26a38b20d |
| N0674 | a2r / 17_rust_std/012_vec | crates/auto-lang/test/a2r/17_rust_std/012_vec/vec.case.ad | vec.expected.rs | b148079d60d81bc4 | b13e6cc727109734 |
| N0675 | a2r / 17_rust_std/013_option_result | crates/auto-lang/test/a2r/17_rust_std/013_option_result/option_result.case.ad | option_result.expected.rs | e1cd04c776b2a84f | fda94f4df2a996f4 |
| N0676 | a2r / 17_rust_std/014_iter | crates/auto-lang/test/a2r/17_rust_std/014_iter/iter.case.ad | iter.expected.rs | 48c12d5298309495 | 747e831115a60ba4 |
| N0677 | a2r / 17_rust_std/015_string_methods | crates/auto-lang/test/a2r/17_rust_std/015_string_methods/string_methods.case.ad | string_methods.expected.rs | 948c46dccc02a230 | a75024bd9c74063c |
| N0678 | a2r / 17_rust_std/016_cmp_ordering | crates/auto-lang/test/a2r/17_rust_std/016_cmp_ordering/cmp_ordering.case.ad | cmp_ordering.expected.rs | b311ed1a1dddf876 | 2ef6640ea5515640 |
| N0679 | a2r / 17_rust_std/017_hash_map_ops | crates/auto-lang/test/a2r/17_rust_std/017_hash_map_ops/hash_map_ops.case.ad | hash_map_ops.expected.rs | 12f47ea4bccea053 | 17d9e90f0300299b |
| N0680 | a2r / 17_rust_std/018_rand | crates/auto-lang/test/a2r/17_rust_std/018_rand/rand.case.ad | rand.expected.rs | 9da95203c4540ecd | d459f9c314868b7f |
| N0681 | a2r / 18_pure_rust/001_pure | crates/auto-lang/test/a2r/18_pure_rust/001_pure/pure.case.ad | pure.expected.rs | 0e42cee484b4f1b4 | e8593cf09a92232f |
| N0682 | a2r / 19_ownership/001_local_borrow | crates/auto-lang/test/a2r/19_ownership/001_local_borrow/local_borrow.case.ad | local_borrow.expected.rs | 4f70383646df605a | 1d265eec36b45674 |
| N0683 | a2r / 19_ownership/002_closure_capture | crates/auto-lang/test/a2r/19_ownership/002_closure_capture/closure_capture.case.ad | closure_capture.expected.rs | bb47150bd5156e9e | 0e3fe0f630cb589c |
| N0684 | a2r / 19_ownership/003_loopvar_owned_field | crates/auto-lang/test/a2r/19_ownership/003_loopvar_owned_field/loopvar_owned_field.case.ad | loopvar_owned_field.expected.rs | 94073039c05c3ee7 | 98b34888d3d2d594 |
| N0685 | a2r / 19_ownership/003_return_escape | crates/auto-lang/test/a2r/19_ownership/003_return_escape/return_escape.case.ad | return_escape.expected.rs | 747880add987455f | b2bb1f410f6f44d9 |
| N0686 | a2r / 19_ownership/004_move_hint | crates/auto-lang/test/a2r/19_ownership/004_move_hint/move_hint.case.ad | move_hint.expected.rs | 9856fd066ca78e6d | 25594099bf37d2a0 |
| N0687 | a2r / 19_ownership/005_async_move | crates/auto-lang/test/a2r/19_ownership/005_async_move/async_move.case.ad | async_move.expected.rs | 880dd4f66a06aed9 | bb1a6cbeed2b6113 |
| N0688 | a2r / 19_ownership/006_go_capture | crates/auto-lang/test/a2r/19_ownership/006_go_capture/go_capture.case.ad | go_capture.expected.rs | f6b618c745f648c8 | 9fbf7c05a5fbe211 |
| N0689 | a2r / 19_ownership/008_indirect_self | crates/auto-lang/test/a2r/19_ownership/008_indirect_self/indirect_self.case.ad | indirect_self.expected.rs | c86c596f13030afe | 328dd018bde2be35 |
| N0690 | a2r / 19_ownership/009_move_closure | crates/auto-lang/test/a2r/19_ownership/009_move_closure/move_closure.case.ad | move_closure.expected.rs | 511009eb3fe2e29e | abe7f9c2cae3fd04 |
| N0691 | a2r / 19_ownership/010_fn_closure_params | crates/auto-lang/test/a2r/19_ownership/010_fn_closure_params/fn_closure_params.case.ad | fn_closure_params.expected.rs | 3a076b7455616c5a | b21b404607306885 |
| N0692 | a2r / 21_generators/001_simple_yield | crates/auto-lang/test/a2r/21_generators/001_simple_yield/simple_yield.case.ad | simple_yield.expected.rs | f4de9b6de3cc5681 | 316eaa9b281f5677 |
| N0693 | a2r / 21_generators/002_stream_yield | crates/auto-lang/test/a2r/21_generators/002_stream_yield/stream_yield.case.ad | stream_yield.expected.rs | d5f21bde111e9aaf | 0e7a33c9437ec0ed |
| N0694 | a2r / 22_actors/001_start_hook | crates/auto-lang/test/a2r/22_actors/001_start_hook/start_hook.case.ad | start_hook.expected.rs | e5370eca551c4a37 | ca9c20ceecddc72c |
| N0695 | a2r / 22_actors/002_message_handler | crates/auto-lang/test/a2r/22_actors/002_message_handler/message_handler.case.ad | message_handler.expected.rs | 5ad3f3a430ec5c97 | f2f84c7739aea84b |
| N0696 | a2r / 22_actors/003_multi_message | crates/auto-lang/test/a2r/22_actors/003_multi_message/multi_message.case.ad | multi_message.expected.rs | 9e7bbf5f238cc902 | 692aab94ef99e323 |
| N0697 | a2r / 22_actors/004_else_handler | crates/auto-lang/test/a2r/22_actors/004_else_handler/else_handler.case.ad | else_handler.expected.rs | 9bfbaef0b828e580 | 7bf451849baa1869 |
| N0698 | a2r / 22_actors/005_state_write | crates/auto-lang/test/a2r/22_actors/005_state_write/state_write.case.ad | state_write.expected.rs | 09766175eed8b2f0 | 25769047acb7385a |
| N0699 | a2r / 22_actors/006_state_increment | crates/auto-lang/test/a2r/22_actors/006_state_increment/state_increment.case.ad | state_increment.expected.rs | 9649bf64f3935abc | 15ab5e5da2674484 |
| N0700 | a2r / 22_actors/007_named_variants | crates/auto-lang/test/a2r/22_actors/007_named_variants/named_variants.case.ad | named_variants.expected.rs | eeea69de974078dd | 214382a3db47fcc2 |
| N0701 | a2r / 22_actors/008_string_pattern | crates/auto-lang/test/a2r/22_actors/008_string_pattern/string_pattern.case.ad | string_pattern.expected.rs | 0d710e8cd6a632dc | 405bb6753b1c9867 |
| N0702 | a2r / 22_actors/009_stop_hook | crates/auto-lang/test/a2r/22_actors/009_stop_hook/stop_hook.case.ad | stop_hook.expected.rs | c3934ef83ef17d88 | 7605ece568b467de |
| N0703 | a2r / 22_actors/010_handle_cross_fn | crates/auto-lang/test/a2r/22_actors/010_handle_cross_fn/handle_cross_fn.case.ad | handle_cross_fn.expected.rs | 478447d2fe93fddf | 756d53157e3de46d |
| N0704 | a2r / 22_actors/011_handle_param | crates/auto-lang/test/a2r/22_actors/011_handle_param/handle_param.case.ad | handle_param.expected.rs | 9a395b0eaacfe1aa | e97487941cdacd78 |
| N0705 | a2r / 22_actors/012_external_enum_msg | crates/auto-lang/test/a2r/22_actors/012_external_enum_msg/external_enum_msg.case.ad | external_enum_msg.expected.rs | ebc2f465efed6b76 | 8961f6ae7f4a4855 |
| N0706 | a2r / 22_actors/013_forward_event | crates/auto-lang/test/a2r/22_actors/013_forward_event/forward_event.case.ad | forward_event.expected.rs | 8656607ca78987ea | 412abf59ddcd345b |
| N0707 | a2r / 22_actors/014_fstring_state | crates/auto-lang/test/a2r/22_actors/014_fstring_state/fstring_state.case.ad | fstring_state.expected.rs | da9501bd7b2684ad | 05cdd296b90e4f8b |
| N0708 | a2r / 22_actors/015_fnp_state_field | crates/auto-lang/test/a2r/22_actors/015_fnp_state_field/fnp_state_field.case.ad | fnp_state_field.expected.rs | c22c5525a91d207d | a97b5fdb4e5ea905 |
| N0709 | a2r / 22_actors/016_actor_methods | crates/auto-lang/test/a2r/22_actors/016_actor_methods/actor_methods.case.ad | actor_methods.expected.rs | 3351927b46958452 | c27721d7d5717500 |
| N0710 | a2r / 22_actors/017_cross_task_variants | crates/auto-lang/test/a2r/22_actors/017_cross_task_variants/cross_task_variants.case.ad | cross_task_variants.expected.rs | 965826fde29d455f | 40ef8363ccd0e49d |
| N0711 | a2r / 22_actors/018_typed_bindings | crates/auto-lang/test/a2r/22_actors/018_typed_bindings/typed_bindings.case.ad | typed_bindings.expected.rs | bdfc547da885d2dd | c61a520643742528 |
| N0712 | a2r / 22_actors/019_guards | crates/auto-lang/test/a2r/22_actors/019_guards/guards.case.ad | guards.expected.rs | af8950032bb99d53 | 22205e7aeed81d26 |
| N0713 | a2r / 22_actors/020_spawn_with_state | crates/auto-lang/test/a2r/22_actors/020_spawn_with_state/spawn_with_state.case.ad | spawn_with_state.expected.rs | bb6c0112854b2b0f | 02e1faed92ffe326 |
| N0714 | a2r / 22_actors/021_handle_struct_field | crates/auto-lang/test/a2r/22_actors/021_handle_struct_field/handle_struct_field.case.ad | handle_struct_field.expected.rs | bb01267f7d37491c | b7b717ebe7ac9ff5 |
| N0715 | a2r / 22_actors/022_closure_cb | crates/auto-lang/test/a2r/22_actors/022_closure_cb/closure_cb.case.ad | closure_cb.expected.rs | 7876fbc4109c82f2 | f60f48a4201ba7c8 |
| N0716 | a2r / 25_foreign_types/001_foreign_generic_field | crates/auto-lang/test/a2r/25_foreign_types/001_foreign_generic_field/foreign_generic_field.case.ad | foreign_generic_field.expected.rs | fa8efb9b5a41c425 | eaecc0642a88df4e |
| N0717 | a2r / 25_foreign_types/002_foreign_trait_impl | crates/auto-lang/test/a2r/25_foreign_types/002_foreign_trait_impl/foreign_trait_impl.case.ad | foreign_trait_impl.expected.rs | 90597c9e5dc59007 | b00ca1d5d6d0a764 |
| N0718 | a2r / 25_foreign_types/003_trait_object | crates/auto-lang/test/a2r/25_foreign_types/003_trait_object/trait_object.case.ad | trait_object.expected.rs | a613ffe373dfd50f | d2fd6609ed2e5b4e |
| N0719 | a2r / 25_foreign_types/004_bare_thread | crates/auto-lang/test/a2r/25_foreign_types/004_bare_thread/bare_thread.case.ad | bare_thread.expected.rs | 4f8589972e8bbfd3 | 943d00e180f3f171 |
| N0720 | a2r / 25_foreign_types/005_term_session_subset | crates/auto-lang/test/a2r/25_foreign_types/005_term_session_subset/term_session_subset.case.ad | term_session_subset.expected.rs | 6e6bafc0b96e4867 | 74084fadb4bf6cfb |
| N0721 | a2r / 25_lifecycle/001_ambiguous_closure_upgrades_rc | crates/auto-lang/test/a2r/25_lifecycle/001_ambiguous_closure_upgrades_rc/ambiguous_closure_upgrades_rc.case.ad | ambiguous_closure_upgrades_rc.expected.rs | b14ef52836041939 | 558f9a6f01f557d1 |
| N0722 | a2r / 25_lifecycle/002_non_escaping_stays_plain | crates/auto-lang/test/a2r/25_lifecycle/002_non_escaping_stays_plain/non_escaping_stays_plain.case.ad | non_escaping_stays_plain.expected.rs | f1508090c323a32b | aab3ff0e5ccbf3eb |
| N0723 | a2r / 25_lifecycle/003_copy_type_write_capture | crates/auto-lang/test/a2r/25_lifecycle/003_copy_type_write_capture/copy_type_write_capture.case.ad | copy_type_write_capture.expected.rs | 5eef0829bb69c8ca | 40d38865f6d26107 |
| N0724 | a2r / 25_lifecycle/004_read_capture_stays_plain | crates/auto-lang/test/a2r/25_lifecycle/004_read_capture_stays_plain/read_capture_stays_plain.case.ad | read_capture_stays_plain.expected.rs | d28b635889ee18e2 | 3f8a8edf49247a46 |
| N0725 | a2r / 25_lifecycle/005_write_capture_explicit_move | crates/auto-lang/test/a2r/25_lifecycle/005_write_capture_explicit_move/write_capture_explicit_move.case.ad | write_capture_explicit_move.expected.rs | 8e5ba14780e969bb | 044c4704b0425e99 |
| N0726 | a2r / 25_lifecycle/006_write_capture_no_move | crates/auto-lang/test/a2r/25_lifecycle/006_write_capture_no_move/write_capture_no_move.case.ad | write_capture_no_move.expected.rs | 36938896b1f3bbaa | 59fb93eeb929c238 |
| N0727 | a2r / 25_lifecycle/007_multi_write_capture_move | crates/auto-lang/test/a2r/25_lifecycle/007_multi_write_capture_move/multi_write_capture_move.case.ad | multi_write_capture_move.expected.rs | f90f1ed6c540826f | 56175784867db886 |
| N0728 | a2r / 26_plan577/001_block_model | crates/auto-lang/test/a2r/26_plan577/001_block_model/block_model.case.ad | block_model.expected.rs | be8623bfc9defa4e | cc36f1eac66a3474 |
| N0729 | a2r / 26_plan577/002_r1_dot_chain | crates/auto-lang/test/a2r/26_plan577/002_r1_dot_chain/r1_dot_chain.case.ad | r1_dot_chain.expected.rs | ac1fc85f287fbbcd | 89d5c5b3e156a8a7 |
| N0730 | a2r / 26_plan577/003_r4_loop_move | crates/auto-lang/test/a2r/26_plan577/003_r4_loop_move/r4_loop_move.case.ad | r4_loop_move.expected.rs | 6d67a1e1157b16cc | 2c9f7f69b0c1cb16 |
| N0731 | a2r / 26_plan577/004_phase0_small_fixes | crates/auto-lang/test/a2r/26_plan577/004_phase0_small_fixes/phase0_small_fixes.case.ad | phase0_small_fixes.expected.rs | 2e27e9f695cd3d70 | cb730b7b30539e6b |
| N0732 | a2r / 27_c_abi/001_export_basic | crates/auto-lang/test/a2r/27_c_abi/001_export_basic/export_basic.case.ad | export_basic.expected.rs | caef43788968e8ee | 24d7341809b1a2be |
| N0733 | a2r / 27_c_abi/002_export_cstr | crates/auto-lang/test/a2r/27_c_abi/002_export_cstr/export_cstr.case.ad | export_cstr.expected.rs | 10eab784fd81d2ce | f9725144a7836690 |
| N0734 | a2r / 27_c_abi/003_use_c_static | crates/auto-lang/test/a2r/27_c_abi/003_use_c_static/use_c_static.case.ad | use_c_static.expected.rs | 438119528d87dfc7 | ce2c1b3375898457 |
| N0735 | a2r / 27_c_abi/004_use_c_dynamic | crates/auto-lang/test/a2r/27_c_abi/004_use_c_dynamic/use_c_dynamic.case.ad | use_c_dynamic.expected.rs | dee2cf8b1009d580 | 1429074375a9064a |
| N0736 | a2r / 27_c_abi/005_engine_face_auto | crates/auto-lang/test/a2r/27_c_abi/005_engine_face_auto/engine_face_auto.case.ad | engine_face_auto.expected.rs | 3acfb1a0e85d01d8 | 37fb15b87759ce0e |
| N0737 | a2r / 28_sqlite/001_basic | crates/auto-lang/test/a2r/28_sqlite/001_basic/basic.case.ad | basic.expected.rs | 1a9baf0082455ab7 | 8f906055b377f7aa |
| N0738 | a2r / 28_sqlite/002_advanced | crates/auto-lang/test/a2r/28_sqlite/002_advanced/advanced.case.ad | advanced.expected.rs | 81ed77d927e9ce34 | f5de3634cee41f36 |
| N0739 | a2r / 29_redis/001_basic | crates/auto-lang/test/a2r/29_redis/001_basic/basic.case.ad | basic.expected.rs | 1e96bfc623511b87 | 0e60d45550db4ba8 |
| N0740 | a2r / 29_redis/002_advanced | crates/auto-lang/test/a2r/29_redis/002_advanced/advanced.case.ad | advanced.expected.rs | 7dbc16eee8db7029 | 10ee49551b66d324 |
| N0741 | a2r / 30_plan724/001_http_client_sync | crates/auto-lang/test/a2r/30_plan724/001_http_client_sync/http_client_sync.case.ad | http_client_sync.expected.rs | 3ab20dc82b46f669 | 96894f54c1d18174 |
| N0742 | a2r / 30_plan724/002_http_client_async | crates/auto-lang/test/a2r/30_plan724/002_http_client_async/http_client_async.case.ad | http_client_async.expected.rs | a0b73ed0126acd5f | 0735f8476f7993e5 |
| N0743 | a2r / 31_plan727/001_http_transfer_sync | crates/auto-lang/test/a2r/31_plan727/001_http_transfer_sync/http_transfer_sync.case.ad | http_transfer_sync.expected.rs | 38e4a4275d44368c | b664c135a39f7569 |
| N0744 | a2r / 31_plan727/002_http_transfer_async | crates/auto-lang/test/a2r/31_plan727/002_http_transfer_async/http_transfer_async.case.ad | http_transfer_async.expected.rs | 87282937c32b8ca5 | a86c5a4acc3da3a4 |
| N0745 | a2r / 32_plan729/001_http_file_response | crates/auto-lang/test/a2r/32_plan729/001_http_file_response/http_file_response.case.ad | http_file_response.expected.rs | 7eba2dba4dde94a2 | bbd74ab0a8683ed4 |
| N0746 | a2r / 33_plan730/001_http_upload | crates/auto-lang/test/a2r/33_plan730/001_http_upload/http_upload.case.ad | http_upload.expected.rs | 52912f9cb13c1dae | a63f63aba4036512 |
| N0747 | a2c / 01_basics/001_hello | crates/auto-lang/test/a2c/01_basics/001_hello/hello.case.ad | hello.expected.c,hello.expected.h | 8557427d5cc0d25f | 7f6de42664ec2823 |
| N0748 | a2c / 01_basics/002_sqrt | crates/auto-lang/test/a2c/01_basics/002_sqrt/sqrt.case.ad | sqrt.expected.c,sqrt.expected.h | 0a0ed24da8410168 | 8421e2ad63b71529 |
| N0749 | a2c / 01_basics/003_func | crates/auto-lang/test/a2c/01_basics/003_func/func.case.ad | func.expected.c,func.expected.h | 31457f27203ee666 | 30bfe90f1b568809 |
| N0750 | a2c / 02_types/001_struct | crates/auto-lang/test/a2c/02_types/001_struct/struct.case.ad | struct.expected.c,struct.expected.h | c53190d404ecfc37 | 8eb3bccb03413864 |
| N0751 | a2c / 02_types/002_enum | crates/auto-lang/test/a2c/02_types/002_enum/enum.case.ad | enum.expected.c,enum.expected.h | 2c77c68eadbe32c6 | 23cf4c623e62e419 |
| N0752 | a2c / 02_types/003_union | crates/auto-lang/test/a2c/02_types/003_union/union.case.ad | union.expected.c,union.expected.h | 84afb46f3ff75943 | 7c2a05305a295f64 |
| N0753 | a2c / 02_types/004_pointer | crates/auto-lang/test/a2c/02_types/004_pointer/pointer.case.ad | pointer.expected.c,pointer.expected.h | 294918b3465d5ab2 | ff4f25d977f0cc25 |
| N0754 | a2c / 02_types/005_inheritance | crates/auto-lang/test/a2c/02_types/005_inheritance/inheritance.case.ad | inheritance.expected.c,inheritance.expected.h | 473b0b3984343585 | 1de807f10db57ab5 |
| N0755 | a2c / 02_types/006_pointer_types | crates/auto-lang/test/a2c/02_types/006_pointer_types/pointer_types.case.ad | pointer_types.expected.c,pointer_types.expected.h | 771f4d795e3efe7d | 5555a91e4c1b94d5 |
| N0756 | a2c / 02_types/007_bool | crates/auto-lang/test/a2c/02_types/007_bool/bool.case.ad | bool.expected.c,bool.expected.h | e1031b3f06962eeb | f6c34090f1ee8d50 |
| N0757 | a2c / 03_control_flow/001_if_basic | crates/auto-lang/test/a2c/03_control_flow/001_if_basic/if_basic.case.ad | if_basic.expected.c,if_basic.expected.h | 3088961af2dbca9e | 833a893cb5241d6e |
| N0758 | a2c / 03_control_flow/002_for_range | crates/auto-lang/test/a2c/03_control_flow/002_for_range/for_range.case.ad | for_range.expected.c,for_range.expected.h | 4e7f76e6a347be76 | 7cdeb936f7c218e3 |
| N0759 | a2c / 03_control_flow/003_is_match | crates/auto-lang/test/a2c/03_control_flow/003_is_match/is_match.case.ad | is_match.expected.c,is_match.expected.h | 6ff9d9e0732d1e7b | 35be3c723c514497 |
| N0760 | a2c / 03_control_flow/005_mut_counter | crates/auto-lang/test/a2c/03_control_flow/005_mut_counter/mut_counter.case.ad | mut_counter.expected.c,mut_counter.expected.h | 53cff3c5512ec1c7 | 7f6d68ca287f8d6e |
| N0761 | a2c / 03_control_flow/006_mut_accumulator | crates/auto-lang/test/a2c/03_control_flow/006_mut_accumulator/mut_accumulator.case.ad | mut_accumulator.expected.c,mut_accumulator.expected.h | 41fd27252a9e5cd2 | 6ed6a667a1453916 |
| N0762 | a2c / 03_control_flow/007_mut_array_sum | crates/auto-lang/test/a2c/03_control_flow/007_mut_array_sum/mut_array_sum.case.ad | mut_array_sum.expected.c,mut_array_sum.expected.h | fa82e474fc76e8f5 | 6c26e04b6fc66b15 |
| N0763 | a2c / 03_control_flow/008_mut_multiple | crates/auto-lang/test/a2c/03_control_flow/008_mut_multiple/mut_multiple.case.ad | mut_multiple.expected.c,mut_multiple.expected.h | 65b9b4f5ab8f284f | f26db8fdd7986a9f |
| N0764 | a2c / 04_strings/001_str | crates/auto-lang/test/a2c/04_strings/001_str/str.case.ad | str.expected.c,str.expected.h | 4c3f3bf4f6df6cc3 | 373ffc6e1583ee48 |
| N0765 | a2c / 04_strings/002_str_split | crates/auto-lang/test/a2c/04_strings/002_str_split/str_split.case.ad | str_split.expected.c,str_split.expected.h | f4bca27735ca7975 | 6dd19d6435c13050 |
| N0766 | a2c / 05_expressions/001_complex_expr | crates/auto-lang/test/a2c/05_expressions/001_complex_expr/complex_expr.case.ad | complex_expr.expected.c,complex_expr.expected.h | 024ca4ba846b0119 | 5c893d8e75053ebf |
| N0767 | a2c / 05_expressions/002_field_access | crates/auto-lang/test/a2c/05_expressions/002_field_access/field_access.case.ad | field_access.expected.c,field_access.expected.h | 7f72b5fb71a406ca | 21dca6ee4adbff8b |
| N0768 | a2c / 05_expressions/003_bang_operator | crates/auto-lang/test/a2c/05_expressions/003_bang_operator/bang_operator.case.ad | bang_operator.expected.c,bang_operator.expected.h | 18c4a3eb77c6a0ee | ab79315dc828bede |
| N0769 | a2c / 05_expressions/004_binary | crates/auto-lang/test/a2c/05_expressions/004_binary/binary.case.ad | binary.expected.c,binary.expected.h | 278533828b2b4c84 | a7b1529c78ba052d |
| N0770 | a2c / 06_pattern_matching/001_hetero_enum | crates/auto-lang/test/a2c/06_pattern_matching/001_hetero_enum/hetero_enum.case.ad | hetero_enum.expected.c,hetero_enum.expected.h | cd8d3f1b53f52ffc | c281406251a8d5dd |
| N0771 | a2c / 06_pattern_matching/002_hetero_enum_verify | crates/auto-lang/test/a2c/06_pattern_matching/002_hetero_enum_verify/hetero_enum_verify.case.ad | hetero_enum_verify.expected.c,hetero_enum_verify.expected.h | cd8d3f1b53f52ffc | e0dd67ed41b04ed1 |
| N0772 | a2c / 06_pattern_matching/003_hetero_enum_types | crates/auto-lang/test/a2c/06_pattern_matching/003_hetero_enum_types/hetero_enum_types.case.ad | hetero_enum_types.expected.c,hetero_enum_types.expected.h | e3308c7f4217cc44 | 2818e54fec0b5946 |
| N0773 | a2c / 06_pattern_matching/004_enum_smoke_2var | crates/auto-lang/test/a2c/06_pattern_matching/004_enum_smoke_2var/enum_smoke_2var.case.ad | enum_smoke_2var.expected.c,enum_smoke_2var.expected.h | 83812f0637ef427d | a34a6fb8a61eced3 |
| N0774 | a2c / 06_pattern_matching/005_enum_smoke_3var | crates/auto-lang/test/a2c/06_pattern_matching/005_enum_smoke_3var/enum_smoke_3var.case.ad | enum_smoke_3var.expected.c,enum_smoke_3var.expected.h | 52c926ea78620ff7 | c04d441248768003 |
| N0775 | a2c / 06_pattern_matching/006_enum_with_functions | crates/auto-lang/test/a2c/06_pattern_matching/006_enum_with_functions/enum_with_functions.case.ad | enum_with_functions.expected.c,enum_with_functions.expected.h | 807d2f36875d41df | 7944abf854a27dc9 |
| N0776 | a2c / 06_pattern_matching/007_tag_types | crates/auto-lang/test/a2c/06_pattern_matching/007_tag_types/tag_types.case.ad | tag_types.expected.c,tag_types.expected.h | 58fd84c50620940b | 86c8e48f15310e22 |
| N0777 | a2c / 07_ownership/001_borrow_view | crates/auto-lang/test/a2c/07_ownership/001_borrow_view/borrow_view.case.ad | borrow_view.expected.c,borrow_view.expected.h | 541457309f66d4ca | 17338d6a19c799ad |
| N0778 | a2c / 07_ownership/002_borrow_mut | crates/auto-lang/test/a2c/07_ownership/002_borrow_mut/borrow_mut.case.ad | borrow_mut.expected.c,borrow_mut.expected.h | 04c31e332348e369 | 3ba0f04fc13be3f0 |
| N0779 | a2c / 07_ownership/003_borrow_move | crates/auto-lang/test/a2c/07_ownership/003_borrow_move/borrow_move.case.ad | borrow_move.expected.c,borrow_move.expected.h | b8b2e0172804353e | f29f5236f9d00d3b |
| N0780 | a2c / 07_ownership/004_borrow_conflicts | crates/auto-lang/test/a2c/07_ownership/004_borrow_conflicts/borrow_conflicts.case.ad | borrow_conflicts.expected.c,borrow_conflicts.expected.h | 389865bc81f6e71f | 0ce1eb5865a628e5 |
| N0781 | a2c / 08_generics/001_const_generics | crates/auto-lang/test/a2c/08_generics/001_const_generics/const_generics.case.ad | const_generics.expected.c,const_generics.expected.h | 9b6677412c5b2e08 | d2debdde4dd0c684 |
| N0782 | a2c / 08_generics/002_generic_field | crates/auto-lang/test/a2c/08_generics/002_generic_field/generic_field.case.ad | generic_field.expected.c,generic_field.expected.h | 0f2005f71a54e124 | 25ae85de8f12b646 |
| N0783 | a2c / 08_generics/003_generic_ptr_field | crates/auto-lang/test/a2c/08_generics/003_generic_ptr_field/generic_ptr_field.case.ad | generic_ptr_field.expected.c,generic_ptr_field.expected.h | 51c01b4a1916ec68 | 06744668baf1f07e |
| N0784 | a2c / 08_generics/004_with_constraint | crates/auto-lang/test/a2c/08_generics/004_with_constraint/with_constraint.case.ad | with_constraint.expected.c,with_constraint.expected.h | 73281a53cb354b65 | 32773705dc0313b6 |
| N0785 | a2c / 08_generics/005_generic_specs | crates/auto-lang/test/a2c/08_generics/005_generic_specs/generic_specs.case.ad | generic_specs.expected.c,generic_specs.expected.h | 4167fe0855760c82 | 792dede9a7f27844 |
| N0786 | a2c / 08_generics/006_generic_spec_ext | crates/auto-lang/test/a2c/08_generics/006_generic_spec_ext/generic_spec_ext.case.ad | generic_spec_ext.expected.c,generic_spec_ext.expected.h | 155caf3b17d4c7a9 | b9a140007eea6464 |
| N0787 | a2c / 09_option_result/001_null_coalesce | crates/auto-lang/test/a2c/09_option_result/001_null_coalesce/null_coalesce.case.ad | null_coalesce.expected.c,null_coalesce.expected.h | ec18a9c14fd5ae37 | 235f4c9149c9b907 |
| N0788 | a2c / 09_option_result/002_error_propagate | crates/auto-lang/test/a2c/09_option_result/002_error_propagate/error_propagate.case.ad | error_propagate.expected.c,error_propagate.expected.h | 9d9826aac162eb18 | ac92111fdd2dfb5f |
| N0789 | a2c / 09_option_result/003_closure | crates/auto-lang/test/a2c/09_option_result/003_closure/closure.case.ad | closure.expected.c,closure.expected.h | 2f82b65d2f8a038a | 70076e225073658f |
| N0790 | a2c / 10_collections/001_array | crates/auto-lang/test/a2c/10_collections/001_array/array.case.ad | array.expected.c,array.expected.h | ce3358240a282670 | 3c12c501270d7a8b |
| N0791 | a2c / 10_collections/002_array_return | crates/auto-lang/test/a2c/10_collections/002_array_return/array_return.case.ad | array_return.expected.c,array_return.expected.h | f4b9fad6c52ae4ae | 726d3d1b2478d643 |
| N0792 | a2c / 10_collections/003_array_declaration | crates/auto-lang/test/a2c/10_collections/003_array_declaration/array_declaration.case.ad | array_declaration.expected.c | b6f1358f4ff5acd9 | 4e2d55cebdd7d3a4 |
| N0793 | a2c / 10_collections/004_array_mutation | crates/auto-lang/test/a2c/10_collections/004_array_mutation/array_mutation.case.ad | array_mutation.expected.c | fb3c40d1579bbca1 | a140f7dd5282416c |
| N0794 | a2c / 10_collections/005_array_index_read | crates/auto-lang/test/a2c/10_collections/005_array_index_read/array_index_read.case.ad | array_index_read.expected.c | b5cbd1ae0aa682ef | 10df1acb1cade112 |
| N0795 | a2c / 10_collections/006_array_copy | crates/auto-lang/test/a2c/10_collections/006_array_copy/array_copy.case.ad | array_copy.expected.c | 429cee67826cc325 | b4681f8f832e0cc3 |
| N0796 | a2c / 10_collections/007_array_slice | crates/auto-lang/test/a2c/10_collections/007_array_slice/array_slice.case.ad | array_slice.expected.c | 8b18b48861ce0325 | a0be96f0c5f9e8f7 |
| N0797 | a2c / 10_collections/008_array_nested | crates/auto-lang/test/a2c/10_collections/008_array_nested/array_nested.case.ad | array_nested.expected.c | 545b874cdf6bc320 | 78606825ef3e96b4 |
| N0798 | a2c / 10_collections/009_array_zero_size | crates/auto-lang/test/a2c/10_collections/009_array_zero_size/array_zero_size.case.ad | array_zero_size.expected.c | df3788a961254e6d | 41677df19996231b |
| N0799 | a2c / 10_collections/010_array_loop | crates/auto-lang/test/a2c/10_collections/010_array_loop/array_loop.case.ad | array_loop.expected.c | cf1f74f3600b9f65 | d39deee9cd3f5c0f |
| N0800 | a2c / 10_collections/011_list_storage | crates/auto-lang/test/a2c/10_collections/011_list_storage/list_storage.case.ad | list_storage.expected.c,list_storage.expected.h | 7ec2d57aeb1249e1 | b0514748913d66d7 |
| N0801 | a2c / 10_collections/012_list_iter | crates/auto-lang/test/a2c/10_collections/012_list_iter/list_iter.case.ad | list_iter.expected.c | f6029f4ecf3d0fdb | 79acc71176d6a7a2 |
| N0802 | a2c / 10_collections/013_list_capacity | crates/auto-lang/test/a2c/10_collections/013_list_capacity/list_capacity.case.ad | list_capacity.expected.c,list_capacity.expected.h | 0ccdbe0e0ca9713e | 8f072bc4f20e8c6c |
| N0803 | a2c / 11_methods/001_method | crates/auto-lang/test/a2c/11_methods/001_method/method.case.ad | method.expected.c,method.expected.h | ebf4ebce25f02384 | 85fdc0e30e4e7173 |
| N0804 | a2c / 12_specs/001_basic_spec | crates/auto-lang/test/a2c/12_specs/001_basic_spec/basic_spec.case.ad | basic_spec.expected.c,basic_spec.expected.h | 079f395f7293557f | 62bc75c1838c4785 |
| N0805 | a2c / 12_specs/002_spec | crates/auto-lang/test/a2c/12_specs/002_spec/spec.case.ad | spec.expected.c,spec.expected.h | 68526d5032ec50fd | c47895458757b263 |
| N0806 | a2c / 13_delegation/001_single | crates/auto-lang/test/a2c/13_delegation/001_single/single.case.ad | single.expected.c,single.expected.h | ae9c79d51126184c | 73c271b9844bf57f |
| N0807 | a2c / 13_delegation/002_multi_delegation | crates/auto-lang/test/a2c/13_delegation/002_multi_delegation/multi_delegation.case.ad | multi_delegation.expected.c,multi_delegation.expected.h | e42cd4181e61399b | 68af7d0632f9c29c |
| N0808 | a2c / 13_delegation/003_delegation_params | crates/auto-lang/test/a2c/13_delegation/003_delegation_params/delegation_params.case.ad | delegation_params.expected.c,delegation_params.expected.h | 06baae268c06daf6 | 09aa56201cee7b6b |
| N0809 | a2c / 14_ext/001_ext_simple | crates/auto-lang/test/a2c/14_ext/001_ext_simple/ext_simple.case.ad | ext_simple.expected.c,ext_simple.expected.h | acd7a16f6d2e0333 | 874bdf5407a4c96f |
| N0810 | a2c / 14_ext/002_ext_instance_method | crates/auto-lang/test/a2c/14_ext/002_ext_instance_method/ext_instance_method.case.ad | ext_instance_method.expected.c,ext_instance_method.expected.h | bbcf175279f171e9 | add4b27c421a034d |
| N0811 | a2c / 14_ext/003_ext_builtin_type | crates/auto-lang/test/a2c/14_ext/003_ext_builtin_type/ext_builtin_type.case.ad | ext_builtin_type.expected.c,ext_builtin_type.expected.h | 7b93ff54c4beb31d | bab68799948ff599 |
| N0812 | a2c / 14_ext/004_ext_static_method | crates/auto-lang/test/a2c/14_ext/004_ext_static_method/ext_static_method.case.ad | ext_static_method.expected.c,ext_static_method.expected.h | b6d551375cf1cfd4 | 0af31ddec29e4be9 |
| N0813 | a2c / 14_ext/005_ext_multiple | crates/auto-lang/test/a2c/14_ext/005_ext_multiple/ext_multiple.case.ad | ext_multiple.expected.c,ext_multiple.expected.h | aeab2602caa52de3 | e90db4c4451d0c90 |
| N0814 | a2c / 14_ext/006_ext_prop_shorthand | crates/auto-lang/test/a2c/14_ext/006_ext_prop_shorthand/ext_prop_shorthand.case.ad | ext_prop_shorthand.expected.c,ext_prop_shorthand.expected.h | f08ca880a61cb2a6 | b2fa20f252ee7e5c |
| N0815 | a2c / 15_type_conversion/001_type_cast | crates/auto-lang/test/a2c/15_type_conversion/001_type_cast/type_cast.case.ad | type_cast.expected.c,type_cast.expected.h | 20d4b0f4bb30b3b2 | cd2e18f5dd4a9dd8 |
| N0816 | a2c / 18_c_interop/001_cstr | crates/auto-lang/test/a2c/18_c_interop/001_cstr/cstr.case.ad | cstr.expected.c,cstr.expected.h | f40277b6e56f6cf5 | 573c0607b136a86f |
| N0817 | a2c / 18_c_interop/002_alias | crates/auto-lang/test/a2c/18_c_interop/002_alias/alias.case.ad | alias.expected.c,alias.expected.h | 7cbfdc1036809ac7 | 67888d6916d9ef44 |
| N0818 | a2c / 18_c_interop/002_windows_console | crates/auto-lang/test/a2c/18_c_interop/002_windows_console/windows_console.case.ad | windows_console.expected.c,windows_console.expected.h | 9d87f8f2b534cd24 | 2361d51292abe2ff |
| N0819 | a2c / 18_c_interop/003_autoterm_ctrlc | crates/auto-lang/test/a2c/18_c_interop/003_autoterm_ctrlc/autoterm_ctrlc.case.ad | autoterm_ctrlc.expected.c,autoterm_ctrlc.expected.h | 6267ad788eed5e0b | e2d7d515fd93f19e |
| N0820 | a2c / 18_c_interop/003_unified_section | crates/auto-lang/test/a2c/18_c_interop/003_unified_section/unified_section.case.ad | unified_section.expected.c,unified_section.expected.h | efc439e7653ec252 | b1300ecfb76a1743 |
| N0821 | a2c / 18_c_interop/004_engine_face | crates/auto-lang/test/a2c/18_c_interop/004_engine_face/engine_face.case.ad | engine_face.expected.c,engine_face.expected.h | 3ef6657763aa3905 | 524d9fbc5d0f37c6 |
| N0822 | a2c / 19_option_type/001_question_syntax | crates/auto-lang/test/a2c/19_option_type/001_question_syntax/question_syntax.case.ad | question_syntax.expected.c,question_syntax.expected.h | f65be130856f4c2c | c1f455ff72c8f9cf |
| N0823 | a2c / 19_option_type/002_question_uint | crates/auto-lang/test/a2c/19_option_type/002_question_uint/question_uint.case.ad | question_uint.expected.c,question_uint.expected.h | 14664f3f457d5cad | fbeacc17351e91dc |
| N0824 | a2c / 19_option_type/003_question_float | crates/auto-lang/test/a2c/19_option_type/003_question_float/question_float.case.ad | question_float.expected.c,question_float.expected.h | 1eac697cdfc9b36c | 8e5e6defc2e45743 |
| N0825 | a2c / 19_option_type/004_question_double | crates/auto-lang/test/a2c/19_option_type/004_question_double/question_double.case.ad | question_double.expected.c,question_double.expected.h | 1b0b59161e7c7f48 | c8cd2303370f7b1d |
| N0826 | a2c / 19_option_type/005_question_char | crates/auto-lang/test/a2c/19_option_type/005_question_char/question_char.case.ad | question_char.expected.c,question_char.expected.h | 574efa5e07518325 | 5cd7c86d5267ca4c |
| N0827 | a2c / 19_option_type/007_question_return_int | crates/auto-lang/test/a2c/19_option_type/007_question_return_int/question_return_int.case.ad | question_return_int.expected.c,question_return_int.expected.h | 0a9a73008107702e | bae64e4cf4e931bb |
| N0828 | a2c / 19_option_type/008_question_return_str | crates/auto-lang/test/a2c/19_option_type/008_question_return_str/question_return_str.case.ad | question_return_str.expected.c,question_return_str.expected.h | 079cc4d0d73a69b6 | f57c33fc9a4ae19f |
| N0829 | a2c / 19_option_type/009_question_return_bool | crates/auto-lang/test/a2c/19_option_type/009_question_return_bool/question_return_bool.case.ad | question_return_bool.expected.c,question_return_bool.expected.h | bcd40629a651e74b | a09846fa72cffcc6 |
| N0830 | a2c / 19_option_type/010_question_return_uint | crates/auto-lang/test/a2c/19_option_type/010_question_return_uint/question_return_uint.case.ad | question_return_uint.expected.c,question_return_uint.expected.h | d0d1215dbe809d0b | c8402e0dacee2273 |
| N0831 | a2c / 19_option_type/011_question_return_float | crates/auto-lang/test/a2c/19_option_type/011_question_return_float/question_return_float.case.ad | question_return_float.expected.c,question_return_float.expected.h | ca7dd1628214bf83 | 5585cf318301878d |
| N0832 | a2c / 19_option_type/012_question_return_double | crates/auto-lang/test/a2c/19_option_type/012_question_return_double/question_return_double.case.ad | question_return_double.expected.c,question_return_double.expected.h | a7929f29d0d3a583 | 0b927d2360b64d53 |
| N0833 | a2c / 19_option_type/013_question_return_char | crates/auto-lang/test/a2c/19_option_type/013_question_return_char/question_return_char.case.ad | question_return_char.expected.c,question_return_char.expected.h | ea7ead8314821998 | c5f01f7a6df2483b |
| N0834 | a2c / 19_option_type/014_question_nested_call | crates/auto-lang/test/a2c/19_option_type/014_question_nested_call/question_nested_call.case.ad | question_nested_call.expected.c,question_nested_call.expected.h | 77baa8cdb4fa3804 | 4f8dc8663ee79886 |
| N0835 | a2c / 19_option_type/015_question_arithmetic | crates/auto-lang/test/a2c/19_option_type/015_question_arithmetic/question_arithmetic.case.ad | question_arithmetic.expected.c,question_arithmetic.expected.h | 5854e2a3edc35143 | 97e64b1cab1e23dc |
| N0836 | a2c / 19_option_type/016_question_comparison | crates/auto-lang/test/a2c/19_option_type/016_question_comparison/question_comparison.case.ad | question_comparison.expected.c,question_comparison.expected.h | d4fa564701eecf2a | 21a076edd44296ac |
| N0837 | a2c / 19_option_type/018_question_negation | crates/auto-lang/test/a2c/19_option_type/018_question_negation/question_negation.case.ad | question_negation.expected.c,question_negation.expected.h | 789c4ee0a1909f93 | b50b5a46d3a06542 |
| N0838 | a2c / 21_storage/001_storage_module | crates/auto-lang/test/a2c/21_storage/001_storage_module/storage_module.case.ad | storage_module.expected.c,storage_module.expected.h | c02acb2882a382d6 | aabdd7a364b52dbe |
| N0839 | a2c / 21_storage/002_storage_usage | crates/auto-lang/test/a2c/21_storage/002_storage_usage/storage_usage.case.ad | storage_usage.expected.c,storage_usage.expected.h | 9cfc135e3153e60e | a458b060c5f6e167 |
| N0840 | a2c / 21_storage/003_plan055_auto_storage | crates/auto-lang/test/a2c/21_storage/003_plan055_auto_storage/plan055_auto_storage.case.ad | plan055_auto_storage.expected.c,plan055_auto_storage.expected.h | 94ad57eef0deb9a5 | 30beb4145fd69438 |
| N0841 | a2c / 22_iterators/001_iter_specs | crates/auto-lang/test/a2c/22_iterators/001_iter_specs/iter_specs.case.ad | iter_specs.expected.c,iter_specs.expected.h | 1b245ee2baaaca61 | 8283e220b5a27e1d |
| N0842 | a2c / 22_iterators/002_map_adapter | crates/auto-lang/test/a2c/22_iterators/002_map_adapter/map_adapter.case.ad | map_adapter.expected.c,map_adapter.expected.h | 37849d6cf9e2ecc1 | 78117384dbdcef4e |
| N0843 | a2c / 22_iterators/003_terminal_operators | crates/auto-lang/test/a2c/22_iterators/003_terminal_operators/terminal_operators.case.ad | terminal_operators.expected.c,terminal_operators.expected.h | d98b191489299004 | 6d041d7cef4234dd |
| N0844 | a2c / 22_iterators/004_terminal_operators_2 | crates/auto-lang/test/a2c/22_iterators/004_terminal_operators_2/terminal_operators_2.case.ad | terminal_operators_2.expected.c,terminal_operators_2.expected.h | cee78ea4a94c9c42 | 855c9842f68a42fc |
| N0845 | a2c / 22_iterators/005_extended_adapters | crates/auto-lang/test/a2c/22_iterators/005_extended_adapters/extended_adapters.case.ad | extended_adapters.expected.c,extended_adapters.expected.h | 6fef8feb3bcba64e | 46dd9557cb77f2c2 |
| N0846 | a2c / 22_iterators/006_predicates | crates/auto-lang/test/a2c/22_iterators/006_predicates/predicates.case.ad | predicates.expected.c,predicates.expected.h | cd0c545d6fc61ae0 | ff652f973f8e2121 |
| N0847 | a2c / 22_iterators/007_collect | crates/auto-lang/test/a2c/22_iterators/007_collect/collect.case.ad | collect.expected.c,collect.expected.h | 15909ab0e56ecc38 | bf97281e81149dc7 |
| N0848 | a2c / 23_stdlib/001_std_hello | crates/auto-lang/test/a2c/23_stdlib/001_std_hello/std_hello.case.ad | std_hello.expected.c,std_hello.expected.h | 841f6fda93762229 | 85ef14eaedd8fe88 |
| N0849 | a2c / 23_stdlib/002_std_getpid | crates/auto-lang/test/a2c/23_stdlib/002_std_getpid/std_getpid.case.ad | std_getpid.expected.c,std_getpid.expected.h | cee714f7536a04ed | d86c8f6527349e21 |
| N0850 | a2c / 23_stdlib/003_std_readline | crates/auto-lang/test/a2c/23_stdlib/003_std_readline/std_readline.case.ad | std_readline.expected.c,std_readline.expected.h | 638a89954abbd63b | 1ef4887f505baca1 |
| N0851 | a2c / 23_stdlib/004_std_file | crates/auto-lang/test/a2c/23_stdlib/004_std_file/std_file.case.ad | std_file.expected.c,std_file.expected.h | 00e0b11f7b702002 | 7e53f1eb874593c3 |
| N0852 | a2c / 23_stdlib/005_std_repl | crates/auto-lang/test/a2c/23_stdlib/005_std_repl/std_repl.case.ad | std_repl.expected.c,std_repl.expected.h | 24b15266d103b401 | 56f502f0ab36f602 |
| N0853 | a2c / 23_stdlib/006_std_str | crates/auto-lang/test/a2c/23_stdlib/006_std_str/std_str.case.ad | std_str.expected.c,std_str.expected.h | 1338ae4b50429e43 | e38ac5ede6d8549e |
| N0854 | a2c / 23_stdlib/007_file_operations | crates/auto-lang/test/a2c/23_stdlib/007_file_operations/file_operations.case.ad | file_operations.expected.c,file_operations.expected.h | 402eb87a841e7daf | fa8fcba84225ac40 |
| N0855 | a2c / 23_stdlib/008_char_io | crates/auto-lang/test/a2c/23_stdlib/008_char_io/char_io.case.ad | char_io.expected.c,char_io.expected.h | 18141632a7c06828 | 2319047a8a8b45d2 |
| N0856 | a2c / 23_stdlib/009_advanced_io | crates/auto-lang/test/a2c/23_stdlib/009_advanced_io/advanced_io.case.ad | advanced_io.expected.c,advanced_io.expected.h | 261ace18ef52a6bc | a779a0a336085359 |
| N0857 | a2c / 23_stdlib/010_io_specs | crates/auto-lang/test/a2c/23_stdlib/010_io_specs/io_specs.case.ad | io_specs.expected.c,io_specs.expected.h | 62bbcdaf447823fb | 14e573cf2815d8de |
| N0858 | a2c / 23_stdlib/011_std_test | crates/auto-lang/test/a2c/23_stdlib/011_std_test/std_test.case.ad | std_test.expected.c,std_test.expected.h | b9dd20855f4ae82c | 5f2b7a50e54271fc |
| N0859 | a2c / 23_stdlib/012_std_readline_2 | crates/auto-lang/test/a2c/23_stdlib/012_std_readline_2/std_readline_2.case.ad | std_readline_2.expected.c,std_readline_2.expected.h | af971c9d4f802010 | 6dc46e55cbf62b47 |
| N0860 | a2c / 23_stdlib/013_std_file_flush | crates/auto-lang/test/a2c/23_stdlib/013_std_file_flush/std_file_flush.case.ad | std_file_flush.expected.c,std_file_flush.expected.h | f876b6b99465579b | 021b2e78a2d77565 |
| N0861 | a2c / 23_stdlib/014_std_file_read | crates/auto-lang/test/a2c/23_stdlib/014_std_file_read/std_file_read.case.ad | std_file_read.expected.c,std_file_read.expected.h | 404320e187fa24e4 | cc7fc1788af43b74 |
| N0862 | a2c / 23_stdlib/017_std_file_write | crates/auto-lang/test/a2c/23_stdlib/017_std_file_write/std_file_write.case.ad | std_file_write.expected.c,std_file_write.expected.h | 0246c90b48a78dba | 2a739fef422eb394 |
| N0863 | a2c / 24_runtime_size/001_runtime_size_var | crates/auto-lang/test/a2c/24_runtime_size/001_runtime_size_var/runtime_size_var.case.ad | runtime_size_var.expected.c,runtime_size_var.expected.h | dc34b4ea35628d0d | 4e481fe9271bf69c |
| N0864 | a2c / 24_runtime_size/002_runtime_size_expr | crates/auto-lang/test/a2c/24_runtime_size/002_runtime_size_expr/runtime_size_expr.case.ad | runtime_size_expr.expected.c,runtime_size_expr.expected.h | c410916dbf76ea8f | 0b7755b854a5a015 |
| N0865 | a2ts / 01_basics/001_hello | crates/auto-lang/test/a2ts/01_basics/001_hello/hello.case.ad | hello.expected.ts | eb421fcf3f6004cf | 527136eed98b1221 |
| N0866 | a2ts / 01_basics/002_func | crates/auto-lang/test/a2ts/01_basics/002_func/func.case.ad | func.expected.ts | 041a3d5f3a70142b | 9bfefa824fbc6719 |
| N0867 | a2ts / 01_basics/003_comments | crates/auto-lang/test/a2ts/01_basics/003_comments/comments.case.ad | comments.expected.ts | e0609523a090c198 | aeb12c82d932c138 |
| N0868 | a2ts / 01_basics/004_unary | crates/auto-lang/test/a2ts/01_basics/004_unary/unary.case.ad | unary.expected.ts | 15b10a657a2ef3b2 | d06de607dacf136f |
| N0869 | a2ts / 01_basics/005_multi_expr | crates/auto-lang/test/a2ts/01_basics/005_multi_expr/multi_expr.case.ad | multi_expr.expected.ts | 95665fa3d4bf37eb | 97e61ea64d79c801 |
| N0870 | a2ts / 01_basics/006_const | crates/auto-lang/test/a2ts/01_basics/006_const/const.case.ad | const.expected.ts | 289e5e7ef887236b | c831341d94cd429c |
| N0871 | a2ts / 01_basics/007_nested_call | crates/auto-lang/test/a2ts/01_basics/007_nested_call/nested_call.case.ad | nested_call.expected.ts | cd7903ae909c1bee | fc1704e063e2d793 |
| N0872 | a2ts / 01_basics/008_bool_ops | crates/auto-lang/test/a2ts/01_basics/008_bool_ops/bool_ops.case.ad | bool_ops.expected.ts | a53d45fdbb219175 | a1ce059c215753b2 |
| N0873 | a2ts / 02_types/001_struct | crates/auto-lang/test/a2ts/02_types/001_struct/struct.case.ad | struct.expected.ts | f9cd4417d2b3d02b | 0a03be522ca04729 |
| N0874 | a2ts / 02_types/002_enum | crates/auto-lang/test/a2ts/02_types/002_enum/enum.case.ad | enum.expected.ts | 565ddeb9117e47cb | a1e70c79d77ca144 |
| N0875 | a2ts / 02_types/003_alias | crates/auto-lang/test/a2ts/02_types/003_alias/alias.case.ad | alias.expected.ts | c49e1add3f1c13ec | 8eda4bdd69d36e4a |
| N0876 | a2ts / 02_types/004_type_alias | crates/auto-lang/test/a2ts/02_types/004_type_alias/type_alias.case.ad | type_alias.expected.ts | c60ad5c0357fe831 | 8c972e600eaf9895 |
| N0877 | a2ts / 02_types/005_nested_struct | crates/auto-lang/test/a2ts/02_types/005_nested_struct/nested_struct.case.ad | nested_struct.expected.ts | 143cddc8ec350914 | 6ad5a85c2282c81b |
| N0878 | a2ts / 02_types/006_enum_simple | crates/auto-lang/test/a2ts/02_types/006_enum_simple/enum_simple.case.ad | enum_simple.expected.ts | baff99b883ad4888 | d5e94aa23d4fa80f |
| N0879 | a2ts / 02_types/007_many_fields | crates/auto-lang/test/a2ts/02_types/007_many_fields/many_fields.case.ad | many_fields.expected.ts | 2646c00a465b6b0e | 3bb468668642f5b6 |
| N0880 | a2ts / 03_control_flow/001_if | crates/auto-lang/test/a2ts/03_control_flow/001_if/if.case.ad | if.expected.ts | 758be0d917092702 | 05275b2a2532a10d |
| N0881 | a2ts / 03_control_flow/002_for | crates/auto-lang/test/a2ts/03_control_flow/002_for/for.case.ad | for.expected.ts | 4e7f76e6a347be76 | 53fccac953e1c2df |
| N0882 | a2ts / 03_control_flow/003_while | crates/auto-lang/test/a2ts/03_control_flow/003_while/while.case.ad | while.expected.ts | 3929d2152e313280 | 47aeceaab507bc5c |
| N0883 | a2ts / 03_control_flow/004_nested_if | crates/auto-lang/test/a2ts/03_control_flow/004_nested_if/nested_if.case.ad | nested_if.expected.ts | 1592ce319f0f4975 | a9058fa2116c3124 |
| N0884 | a2ts / 03_control_flow/005_loop | crates/auto-lang/test/a2ts/03_control_flow/005_loop/loop.case.ad | loop.expected.ts | 1541f0df67150eeb | e58d530bd2419912 |
| N0885 | a2ts / 03_control_flow/006_blocks | crates/auto-lang/test/a2ts/03_control_flow/006_blocks/blocks.case.ad | blocks.expected.ts | 3ecf9258b5745026 | 4348cc626781f7e2 |
| N0886 | a2ts / 03_control_flow/007_async_fn | crates/auto-lang/test/a2ts/03_control_flow/007_async_fn/async_fn.case.ad | async_fn.expected.ts | 687a48c3c5ffd461 | c4ae273a8518297b |
| N0887 | a2ts / 03_control_flow/008_await_expr | crates/auto-lang/test/a2ts/03_control_flow/008_await_expr/await_expr.case.ad | await_expr.expected.ts | 641e53d1cd578464 | 492fa5b8a1962b02 |
| N0888 | a2ts / 03_control_flow/009_promise_return | crates/auto-lang/test/a2ts/03_control_flow/009_promise_return/promise_return.case.ad | promise_return.expected.ts | c0d3bf94ec43cd7a | 15bd8c77747e0bbb |
| N0889 | a2ts / 03_control_flow/010_while_loop | crates/auto-lang/test/a2ts/03_control_flow/010_while_loop/while_loop.case.ad | while_loop.expected.ts | bd989c81a2b7dde2 | 04c3dffdf378b5db |
| N0890 | a2ts / 03_control_flow/011_nested_loops | crates/auto-lang/test/a2ts/03_control_flow/011_nested_loops/nested_loops.case.ad | nested_loops.expected.ts | 7ab5419b9da278f3 | c43d4647c5440459 |
| N0891 | a2ts / 04_strings/002_fstring_nested | crates/auto-lang/test/a2ts/04_strings/002_fstring_nested/fstring_nested.case.ad | fstring_nested.expected.ts | 6364690cad92b035 | 20876341b6b6ad73 |
| N0892 | a2ts / 04_strings/003_fstring_multi | crates/auto-lang/test/a2ts/04_strings/003_fstring_multi/fstring_multi.case.ad | fstring_multi.expected.ts | 9aea778b479d0ff8 | ab2f1594bda5fca7 |
| N0893 | a2ts / 04_strings/004_fstring_edge | crates/auto-lang/test/a2ts/04_strings/004_fstring_edge/fstring_edge.case.ad | fstring_edge.expected.ts | 5905ca6241220e8d | 6f38295a391c7ae1 |
| N0894 | a2ts / 04_strings/005_str_split | crates/auto-lang/test/a2ts/04_strings/005_str_split/str_split.case.ad | str_split.expected.ts | 035b22c68886038e | 257bb6a193133b38 |
| N0895 | a2ts / 04_strings/006_str_replace | crates/auto-lang/test/a2ts/04_strings/006_str_replace/str_replace.case.ad | str_replace.expected.ts | 9bb872727aab78c5 | a9b09b4c12f19963 |
| N0896 | a2ts / 04_strings/007_str_case | crates/auto-lang/test/a2ts/04_strings/007_str_case/str_case.case.ad | str_case.expected.ts | 4ace65d09b341493 | 95d5e854d8839a8d |
| N0897 | a2ts / 04_strings/008_str_find | crates/auto-lang/test/a2ts/04_strings/008_str_find/str_find.case.ad | str_find.expected.ts | fbe4b3413b690acd | 870874b5c282e695 |
| N0898 | a2ts / 05_expressions/001_object | crates/auto-lang/test/a2ts/05_expressions/001_object/object.case.ad | object.expected.ts | baf7a1635ad62700 | cb4ce604714dd905 |
| N0899 | a2ts / 05_expressions/002_composition | crates/auto-lang/test/a2ts/05_expressions/002_composition/composition.case.ad | composition.expected.ts | b0f8a7c167791e3d | b96829cbb2acd0d0 |
| N0900 | a2ts / 05_expressions/003_range_expr | crates/auto-lang/test/a2ts/05_expressions/003_range_expr/range_expr.case.ad | range_expr.expected.ts | ffd6bfc3e5590a89 | 9f9fd6f4bad47ce0 |
| N0901 | a2ts / 05_expressions/004_ternary | crates/auto-lang/test/a2ts/05_expressions/004_ternary/ternary.case.ad | ternary.expected.ts | 897ac700a3c78e57 | bd2a2f84666bf5d3 |
| N0902 | a2ts / 05_expressions/005_composition | crates/auto-lang/test/a2ts/05_expressions/005_composition/composition.case.ad | composition.expected.ts | a564969274ed8245 | 64720adf57f6eaf5 |
| N0903 | a2ts / 05_expressions/006_index | crates/auto-lang/test/a2ts/05_expressions/006_index/index.case.ad | index.expected.ts | d71fafb6d4edaa90 | ddeab4d42aad65f2 |
| N0904 | a2ts / 05_expressions/007_cast | crates/auto-lang/test/a2ts/05_expressions/007_cast/cast.case.ad | cast.expected.ts | 47e9e09b2abf5778 | fbcd813f40de67f4 |
| N0905 | a2ts / 06_pattern_matching/001_hetero_enum | crates/auto-lang/test/a2ts/06_pattern_matching/001_hetero_enum/hetero_enum.case.ad | hetero_enum.expected.ts | cd8d3f1b53f52ffc | 4f6923b4a48c5e85 |
| N0906 | a2ts / 06_pattern_matching/002_switch_guards | crates/auto-lang/test/a2ts/06_pattern_matching/002_switch_guards/switch_guards.case.ad | switch_guards.expected.ts | 57e8b2cb3b33c269 | 72012414fcbb4cc7 |
| N0907 | a2ts / 06_pattern_matching/003_wildcard | crates/auto-lang/test/a2ts/06_pattern_matching/003_wildcard/wildcard.case.ad | wildcard.expected.ts | 9e0128282c0af73f | b6ccf50b8174183a |
| N0908 | a2ts / 06_pattern_matching/004_nested_destructure | crates/auto-lang/test/a2ts/06_pattern_matching/004_nested_destructure/nested_destructure.case.ad | nested_destructure.expected.ts | 67a65a8fe79eebe1 | 532b73829d273705 |
| N0909 | a2ts / 06_pattern_matching/005_multi_pattern | crates/auto-lang/test/a2ts/06_pattern_matching/005_multi_pattern/multi_pattern.case.ad | multi_pattern.expected.ts | ebb733f1f6e24a3b | 97c35ea0ef68a39a |
| N0910 | a2ts / 06_pattern_matching/006_enum_match | crates/auto-lang/test/a2ts/06_pattern_matching/006_enum_match/enum_match.case.ad | enum_match.expected.ts | 5b91824273e417b7 | 9bb450edff3deee6 |
| N0911 | a2ts / 06_pattern_matching/007_option_match | crates/auto-lang/test/a2ts/06_pattern_matching/007_option_match/option_match.case.ad | option_match.expected.ts | d22ea03d8133d4b5 | dc5c5519849c1bef |
| N0912 | a2ts / 06_pattern_matching/008_literal_match | crates/auto-lang/test/a2ts/06_pattern_matching/008_literal_match/literal_match.case.ad | literal_match.expected.ts | 5040036145406e13 | 99c1e58183ca58b6 |
| N0913 | a2ts / 07_ownership/001_union | crates/auto-lang/test/a2ts/07_ownership/001_union/union.case.ad | union.expected.ts | 84afb46f3ff75943 | bd8a5645f0f76396 |
| N0914 | a2ts / 08_generics/001_generic_fn | crates/auto-lang/test/a2ts/08_generics/001_generic_fn/generic_fn.case.ad | generic_fn.expected.ts | da9e13e821b7eca2 | f04079e01d386205 |
| N0915 | a2ts / 08_generics/002_generic_extends | crates/auto-lang/test/a2ts/08_generics/002_generic_extends/generic_extends.case.ad | generic_extends.expected.ts | e9b90b6a83e1f8a3 | adc8a35f156ab397 |
| N0916 | a2ts / 08_generics/003_generic_struct | crates/auto-lang/test/a2ts/08_generics/003_generic_struct/generic_struct.case.ad | generic_struct.expected.ts | 9da4bb5e7e0b4a4b | 097d6a62e69a1db3 |
| N0917 | a2ts / 08_generics/004_multi_param | crates/auto-lang/test/a2ts/08_generics/004_multi_param/multi_param.case.ad | multi_param.expected.ts | dab32e8d4c5b2040 | f72c6a64934b98ee |
| N0918 | a2ts / 09_option_result/001_closure | crates/auto-lang/test/a2ts/09_option_result/001_closure/closure.case.ad | closure.expected.ts | 9e41232ab6389fde | 097db41a3bb68e8f |
| N0919 | a2ts / 09_option_result/002_option_basic | crates/auto-lang/test/a2ts/09_option_result/002_option_basic/option_basic.case.ad | option_basic.expected.ts | 64a8dcb07261dafb | ca688b06a69fc31d |
| N0920 | a2ts / 09_option_result/003_null_coalesce | crates/auto-lang/test/a2ts/09_option_result/003_null_coalesce/null_coalesce.case.ad | null_coalesce.expected.ts | 9be6ddfbaf842df8 | b9f562fe139a3962 |
| N0921 | a2ts / 09_option_result/004_result_ok_err | crates/auto-lang/test/a2ts/09_option_result/004_result_ok_err/result_ok_err.case.ad | result_ok_err.expected.ts | de4f37bdc807842f | a1f9204ae59ff330 |
| N0922 | a2ts / 09_option_result/005_option_pattern | crates/auto-lang/test/a2ts/09_option_result/005_option_pattern/option_pattern.case.ad | option_pattern.expected.ts | 621bec0313e0dd4b | 32092d752b269a01 |
| N0923 | a2ts / 09_option_result/006_is_some | crates/auto-lang/test/a2ts/09_option_result/006_is_some/is_some.case.ad | is_some.expected.ts | 4348b8e38c29653d | 9e3fef8fe08b25ac |
| N0924 | a2ts / 09_option_result/007_is_none | crates/auto-lang/test/a2ts/09_option_result/007_is_none/is_none.case.ad | is_none.expected.ts | cf960ef55ba206ba | 69f17b804ff7c111 |
| N0925 | a2ts / 09_option_result/008_unwrap | crates/auto-lang/test/a2ts/09_option_result/008_unwrap/unwrap.case.ad | unwrap.expected.ts | ca98e13308a095fd | c576effa022291da |
| N0926 | a2ts / 09_option_result/009_or_else | crates/auto-lang/test/a2ts/09_option_result/009_or_else/or_else.case.ad | or_else.expected.ts | 475a8936e4584a62 | e45fbcf6153bc9d4 |
| N0927 | a2ts / 09_option_result/010_option_map | crates/auto-lang/test/a2ts/09_option_result/010_option_map/option_map.case.ad | option_map.expected.ts | 23b300a31afa6ef3 | 21ecd7399547ee75 |
| N0928 | a2ts / 09_option_result/011_filter | crates/auto-lang/test/a2ts/09_option_result/011_filter/filter.case.ad | filter.expected.ts | 3896245e29417578 | 90c6ef72261c74e2 |
| N0929 | a2ts / 10_collections/001_array_basic | crates/auto-lang/test/a2ts/10_collections/001_array_basic/array_basic.case.ad | array_basic.expected.ts | 9392b661e8b00e15 | 25a431f09df1de58 |
| N0930 | a2ts / 10_collections/002_array_methods | crates/auto-lang/test/a2ts/10_collections/002_array_methods/array_methods.case.ad | array_methods.expected.ts | 639547ad156cc6b4 | b9d93f9859430d02 |
| N0931 | a2ts / 10_collections/003_map_type | crates/auto-lang/test/a2ts/10_collections/003_map_type/map_type.case.ad | map_type.expected.ts | 8a4ab1aa00452f98 | d71f2a5e630fc735 |
| N0932 | a2ts / 10_collections/004_set_type | crates/auto-lang/test/a2ts/10_collections/004_set_type/set_type.case.ad | set_type.expected.ts | b840269b70637930 | 4c1ae95d545e6526 |
| N0933 | a2ts / 10_collections/005_object_literal | crates/auto-lang/test/a2ts/10_collections/005_object_literal/object_literal.case.ad | object_literal.expected.ts | 9161e84fa69fd709 | 011c6dfba1c84ed0 |
| N0934 | a2ts / 11_methods/001_method | crates/auto-lang/test/a2ts/11_methods/001_method/method.case.ad | method.expected.ts | ebf4ebce25f02384 | adebc2b1d60d4afd |
| N0935 | a2ts / 11_methods/002_struct_methods | crates/auto-lang/test/a2ts/11_methods/002_struct_methods/struct_methods.case.ad | struct_methods.expected.ts | 42bf2bacc0266982 | 0d8db424e1efba72 |
| N0936 | a2ts / 11_methods/003_ext | crates/auto-lang/test/a2ts/11_methods/003_ext/ext.case.ad | ext.expected.ts | 8437481c092fc5ba | a222fa446b216b6e |
| N0937 | a2ts / 11_methods/004_static_method | crates/auto-lang/test/a2ts/11_methods/004_static_method/static_method.case.ad | static_method.expected.ts | 99bef47a7889e897 | 29d68838c5fe38ff |
| N0938 | a2ts / 11_methods/005_method_params | crates/auto-lang/test/a2ts/11_methods/005_method_params/method_params.case.ad | method_params.expected.ts | 649cad75bcb6f678 | 56c844590f0f4d0b |
| N0939 | a2ts / 12_specs/001_basic_spec | crates/auto-lang/test/a2ts/12_specs/001_basic_spec/basic_spec.case.ad | basic_spec.expected.ts | 079f395f7293557f | 4e5905164621f95b |
| N0940 | a2ts / 12_specs/002_spec | crates/auto-lang/test/a2ts/12_specs/002_spec/spec.case.ad | spec.expected.ts | 68526d5032ec50fd | 9e90525be7c137f6 |
| N0941 | a2ts / 12_specs/003_spec_generic | crates/auto-lang/test/a2ts/12_specs/003_spec_generic/spec_generic.case.ad | spec_generic.expected.ts | 7d4763221cd9fb17 | baf7ff3eeca562ef |
| N0942 | a2ts / 13_delegation/001_delegation | crates/auto-lang/test/a2ts/13_delegation/001_delegation/delegation.case.ad | delegation.expected.ts | ae9c79d51126184c | 02dcfd2a86d11def |
| N0943 | a2ts / 13_delegation/002_delegation_fn | crates/auto-lang/test/a2ts/13_delegation/002_delegation_fn/delegation_fn.case.ad | delegation_fn.expected.ts | b77e73f627e90039 | a9b6b912698a7be0 |
| N0944 | a2ts / 14_modules/001_basic_import | crates/auto-lang/test/a2ts/14_modules/001_basic_import/basic_import.case.ad | basic_import.expected.ts | 1566d6fad47060b6 | b2d92e627a8c0786 |
| N0945 | a2ts / 14_modules/002_named_import | crates/auto-lang/test/a2ts/14_modules/002_named_import/named_import.case.ad | named_import.expected.ts | 9aa94726ff5a811b | 43ac8251fb52ec09 |
| N0946 | a2ts / 14_modules/003_export_fn | crates/auto-lang/test/a2ts/14_modules/003_export_fn/export_fn.case.ad | export_fn.expected.ts | 3c16a4421fa4506d | 55ad4fcaa064f28a |
| N0947 | a2ts / 15_type_conversion/001_cast | crates/auto-lang/test/a2ts/15_type_conversion/001_cast/cast.case.ad | cast.expected.ts | 89ce79651f475cb9 | 1a9fb401357b9f00 |
| N0948 | a2ts / 15_type_conversion/002_to | crates/auto-lang/test/a2ts/15_type_conversion/002_to/to.case.ad | to.expected.ts | 93bd4a2e50dfae79 | db15e14bd94ade1f |
| N0949 | a2ts / 18_ts_interop/001_for_each | crates/auto-lang/test/a2ts/18_ts_interop/001_for_each/for_each.case.ad | for_each.expected.ts | ac56254e192caa5e | ce8cc1c7e2415eff |
| N0950 | cookbook / algorithms/001_sort_int | crates/auto-lang/test/cookbook/algorithms/001_sort_int/sort_int.case.ad | sort_int.expected.rs | 0a5b3d64cae6a478 | 935cb97b433b9934 |
| N0951 | cookbook / algorithms/002_sort_float | crates/auto-lang/test/cookbook/algorithms/002_sort_float/sort_float.case.ad | sort_float.expected.rs | cdd6acddda327714 | acd8a7113515fcac |
| N0952 | cookbook / algorithms/003_sort_struct | crates/auto-lang/test/cookbook/algorithms/003_sort_struct/sort_struct.case.ad | sort_struct.expected.rs | c9373a022d475e15 | 2dc2f60889611ec8 |
| N0953 | cookbook / algorithms/004_rand | crates/auto-lang/test/cookbook/algorithms/004_rand/rand.case.ad | rand.expected.rs | 02ac8bc31f28b6b2 | b8440d90696ac7f7 |
| N0954 | cookbook / algorithms/005_rand_choose | crates/auto-lang/test/cookbook/algorithms/005_rand_choose/rand_choose.case.ad | rand_choose.expected.rs | 6dccead0359f1843 | 0a0ca31493d5b584 |
| N0955 | cookbook / algorithms/006_rand_custom | crates/auto-lang/test/cookbook/algorithms/006_rand_custom/rand_custom.case.ad | rand_custom.expected.rs | aec2c8b36be1c438 | 59e591a1a825064c |
| N0956 | cookbook / algorithms/007_rand_dist | crates/auto-lang/test/cookbook/algorithms/007_rand_dist/rand_dist.case.ad | rand_dist.expected.rs | 538e303444586445 | 847af091b3720119 |
| N0957 | cookbook / algorithms/008_rand_passwd | crates/auto-lang/test/cookbook/algorithms/008_rand_passwd/rand_passwd.case.ad | rand_passwd.expected.rs | c4a12e94ed8fccf6 | 6faddfb0bc95297c |
| N0958 | cookbook / algorithms/009_rand_range | crates/auto-lang/test/cookbook/algorithms/009_rand_range/rand_range.case.ad | rand_range.expected.rs | a17bfc94d010d2dc | 648315e7e5476559 |
| N0959 | cookbook / algorithms/010_rand_custom | crates/auto-lang/test/cookbook/algorithms/010_rand_custom/rand_custom.case.ad | rand_custom.expected.rs | adafc7e26b474c1b | 4b73195280dd5e02 |
| N0960 | cookbook / algorithms/011_rand_dist | crates/auto-lang/test/cookbook/algorithms/011_rand_dist/rand_dist.case.ad | rand_dist.expected.rs | 1f9a544c01b32333 | da1b8202e8a2fbe6 |
| N0961 | cookbook / cli/001_clap_basic | crates/auto-lang/test/cookbook/cli/001_clap_basic/clap_basic.case.ad | clap_basic.expected.rs | 1331a8b1d174373f | 9673460e38c16753 |
| N0962 | cookbook / cli/002_ansi_term | crates/auto-lang/test/cookbook/cli/002_ansi_term/ansi_term.case.ad | ansi_term.expected.rs | 92e5018c1834eeeb | 4072b56f8ccfaa70 |
| N0963 | cookbook / compression/001_tar_compress | crates/auto-lang/test/cookbook/compression/001_tar_compress/tar_compress.case.ad | tar_compress.expected.rs | 0fd8315dd401c5f6 | 82c074c81fd9c54d |
| N0964 | cookbook / compression/002_tar_decompress | crates/auto-lang/test/cookbook/compression/002_tar_decompress/tar_decompress.case.ad | tar_decompress.expected.rs | 86a7a31108dfe8b6 | f4ace2a0e8343a58 |
| N0965 | cookbook / compression/003_tar_strip_prefix | crates/auto-lang/test/cookbook/compression/003_tar_strip_prefix/tar_strip_prefix.case.ad | tar_strip_prefix.expected.rs | a69593eb796e08db | 53c57c368b85ce37 |
| N0966 | cookbook / concurrency/001_rayon_any_all | crates/auto-lang/test/cookbook/concurrency/001_rayon_any_all/rayon_any_all.case.ad | rayon_any_all.expected.rs | 1e4ee583d57915b8 | 4546a0e1c78cccf5 |
| N0967 | cookbook / concurrency/002_rayon_map_reduce | crates/auto-lang/test/cookbook/concurrency/002_rayon_map_reduce/rayon_map_reduce.case.ad | rayon_map_reduce.expected.rs | d9bc75a6c03df2b8 | 23efefa1802d74ef |
| N0968 | cookbook / concurrency/003_rayon_parallel_sort | crates/auto-lang/test/cookbook/concurrency/003_rayon_parallel_sort/rayon_parallel_sort.case.ad | rayon_parallel_sort.expected.rs | 836a556ce27a4a89 | 75a6691ef223d223 |
| N0969 | cookbook / concurrency/004_crossbeam_spsc | crates/auto-lang/test/cookbook/concurrency/004_crossbeam_spsc/crossbeam_spsc.case.ad | crossbeam_spsc.expected.rs | a9d3f4b3e58f27fa | 737b8ddbf320a209 |
| N0970 | cookbook / concurrency/005_rayon_iter_mut | crates/auto-lang/test/cookbook/concurrency/005_rayon_iter_mut/rayon_iter_mut.case.ad | rayon_iter_mut.expected.rs | b7607e1140784a82 | d19094caefaf8256 |
| N0971 | cookbook / concurrency/006_rayon_parallel_search | crates/auto-lang/test/cookbook/concurrency/006_rayon_parallel_search/rayon_parallel_search.case.ad | rayon_parallel_search.expected.rs | 209dc5458c2e6d2e | 4de16e8f328995c5 |
| N0972 | cookbook / concurrency/007_crossbeam_complex | crates/auto-lang/test/cookbook/concurrency/007_crossbeam_complex/crossbeam_complex.case.ad | crossbeam_complex.expected.rs | 918307410e2dfc31 | ac12c566f628b6ee |
| N0973 | cookbook / concurrency/008_crossbeam_spawn | crates/auto-lang/test/cookbook/concurrency/008_crossbeam_spawn/crossbeam_spawn.case.ad | crossbeam_spawn.expected.rs | 5700fe3e97c3e3b2 | dd1eb6747e903c36 |
| N0974 | cookbook / concurrency/009_global_mut_state | crates/auto-lang/test/cookbook/concurrency/009_global_mut_state/global_mut_state.case.ad | global_mut_state.expected.rs | ce31db9fd68e7231 | 79c56797ce63b5ae |
| N0975 | cookbook / concurrency/010_threadpool_walk | crates/auto-lang/test/cookbook/concurrency/010_threadpool_walk/threadpool_walk.case.ad | threadpool_walk.expected.rs | 08d20bd5207c5b59 | 1d5dac4cb2890bdf |
| N0976 | cookbook / cryptography/001_sha_digest | crates/auto-lang/test/cookbook/cryptography/001_sha_digest/sha_digest.case.ad | sha_digest.expected.rs | 50f5d4a7ee0a4e77 | 9c08160a864d307a |
| N0977 | cookbook / cryptography/002_pbkdf2 | crates/auto-lang/test/cookbook/cryptography/002_pbkdf2/pbkdf2.case.ad | pbkdf2.expected.rs | 50d4ad1f73227881 | 202528ad3f20965c |
| N0978 | cookbook / cryptography/003_hmac | crates/auto-lang/test/cookbook/cryptography/003_hmac/hmac.case.ad | hmac.expected.rs | 2e979d913b64d9d6 | 500e162f31feebc6 |
| N0979 | cookbook / datetime/001_elapsed_time | crates/auto-lang/test/cookbook/datetime/001_elapsed_time/elapsed_time.case.ad | elapsed_time.expected.rs | c5de1b427c174b9b | 06e3894eae5c80f4 |
| N0980 | cookbook / datetime/002_checked | crates/auto-lang/test/cookbook/datetime/002_checked/checked.case.ad | checked.expected.rs | 50945b75609896c4 | dd191589ac368991 |
| N0981 | cookbook / datetime/003_timezone | crates/auto-lang/test/cookbook/datetime/003_timezone/timezone.case.ad | timezone.expected.rs | 7624d72d86f7a4bd | 95bec40828600405 |
| N0982 | cookbook / datetime/004_current | crates/auto-lang/test/cookbook/datetime/004_current/current.case.ad | current.expected.rs | 6cd998e521daa5b3 | b18f354b399415ed |
| N0983 | cookbook / datetime/005_format | crates/auto-lang/test/cookbook/datetime/005_format/format.case.ad | format.expected.rs | 3d416bb53c6dc851 | 5707ff101c32b103 |
| N0984 | cookbook / datetime/006_parse_string | crates/auto-lang/test/cookbook/datetime/006_parse_string/parse_string.case.ad | parse_string.expected.rs | 81f94bbebe72721d | 1d04405cdf9c5ff9 |
| N0985 | cookbook / datetime/007_timestamp | crates/auto-lang/test/cookbook/datetime/007_timestamp/timestamp.case.ad | timestamp.expected.rs | 441dc07ca2ccaace | 7b3173db97098012 |
| N0986 | cookbook / devtools/001_log_debug | crates/auto-lang/test/cookbook/devtools/001_log_debug/log_debug.case.ad | log_debug.expected.rs | 3b0a7283c3eedcac | 8889bdd233ac26d0 |
| N0987 | cookbook / devtools/002_log_error | crates/auto-lang/test/cookbook/devtools/002_log_error/log_error.case.ad | log_error.expected.rs | 43519ce33e0b5c9d | 3223bec134db5907 |
| N0988 | cookbook / devtools/003_log_stdout | crates/auto-lang/test/cookbook/devtools/003_log_stdout/log_stdout.case.ad | log_stdout.expected.rs | 3108b3f070130f5c | 4ce8948cff0b8e61 |
| N0989 | cookbook / devtools/004_log_custom | crates/auto-lang/test/cookbook/devtools/004_log_custom/log_custom.case.ad | log_custom.expected.rs | ef1f688a51c0df33 | 786b212b36070e8d |
| N0990 | cookbook / devtools/005_log_syslog | crates/auto-lang/test/cookbook/devtools/005_log_syslog/log_syslog.case.ad | log_syslog.expected.rs | 4f4863e2799affff | 4839e9f3f2deec66 |
| N0991 | cookbook / devtools/006_log_env | crates/auto-lang/test/cookbook/devtools/006_log_env/log_env.case.ad | log_env.expected.rs | 949edaba15b428db | e6c7a7bf9c52952f |
| N0992 | cookbook / devtools/007_log_mod | crates/auto-lang/test/cookbook/devtools/007_log_mod/log_mod.case.ad | log_mod.expected.rs | cf800d5563511bb0 | 1712508452649a17 |
| N0993 | cookbook / devtools/008_log_timestamp | crates/auto-lang/test/cookbook/devtools/008_log_timestamp/log_timestamp.case.ad | log_timestamp.expected.rs | ea72cd38e257a09b | 3b071efe37e29135 |
| N0994 | cookbook / devtools/009_log_custom_location | crates/auto-lang/test/cookbook/devtools/009_log_custom_location/log_custom_location.case.ad | log_custom_location.expected.rs | cc3a26709594f7b4 | e3ecf6c2d0e55514 |
| N0995 | cookbook / devtools/010_tracing_console | crates/auto-lang/test/cookbook/devtools/010_tracing_console/tracing_console.case.ad | tracing_console.expected.rs | d8d11f26265bd1b1 | 260ac7560a440f84 |
| N0996 | cookbook / encoding/001_json | crates/auto-lang/test/cookbook/encoding/001_json/json.case.ad | json.expected.rs | 29b7c34d0e5f648d | d1c9c0775a16ba96 |
| N0997 | cookbook / encoding/002_toml | crates/auto-lang/test/cookbook/encoding/002_toml/toml.case.ad | toml.expected.rs | f0cdd614f055564b | 3578a3d9cc84ba7f |
| N0998 | cookbook / encoding/003_csv_read | crates/auto-lang/test/cookbook/encoding/003_csv_read/csv_read.case.ad | csv_read.expected.rs | 47e831bbc080d873 | 9d99a7e2f2a92952 |
| N0999 | cookbook / encoding/004_base64 | crates/auto-lang/test/cookbook/encoding/004_base64/base64.case.ad | base64.expected.rs | 53a38f264ae4e6c6 | 6d35564875e7f0f0 |
| N1000 | cookbook / encoding/005_hex | crates/auto-lang/test/cookbook/encoding/005_hex/hex.case.ad | hex.expected.rs | 2ee908303a9d9190 | 2aa5be5fdcbba9a2 |
| N1001 | cookbook / encoding/006_endian_byte | crates/auto-lang/test/cookbook/encoding/006_endian_byte/endian_byte.case.ad | endian_byte.expected.rs | a15f93961c6129f0 | 02f6448c09cc3c11 |
| N1002 | cookbook / encoding/007_csv_delimiter | crates/auto-lang/test/cookbook/encoding/007_csv_delimiter/csv_delimiter.case.ad | csv_delimiter.expected.rs | 1324404e40735f2f | 15eef4349ae034d9 |
| N1003 | cookbook / encoding/008_csv_filter | crates/auto-lang/test/cookbook/encoding/008_csv_filter/csv_filter.case.ad | csv_filter.expected.rs | 73eb29a188e01781 | f8756531304c5109 |
| N1004 | cookbook / encoding/009_csv_invalid | crates/auto-lang/test/cookbook/encoding/009_csv_invalid/csv_invalid.case.ad | csv_invalid.expected.rs | 35c6e268fe1232bb | 1645e8a02355ce49 |
| N1005 | cookbook / encoding/010_csv_serde_serialize | crates/auto-lang/test/cookbook/encoding/010_csv_serde_serialize/csv_serde_serialize.case.ad | csv_serde_serialize.expected.rs | ea587951deb1d6f2 | 772677f198bc6244 |
| N1006 | cookbook / encoding/011_csv_serialize | crates/auto-lang/test/cookbook/encoding/011_csv_serialize/csv_serialize.case.ad | csv_serialize.expected.rs | fde27add68b4892f | 4c55dad39eed0ac4 |
| N1007 | cookbook / encoding/012_csv_transform | crates/auto-lang/test/cookbook/encoding/012_csv_transform/csv_transform.case.ad | csv_transform.expected.rs | 8594cc669fb104ed | a8d4104bd1698719 |
| N1008 | cookbook / encoding/013_percent_encode | crates/auto-lang/test/cookbook/encoding/013_percent_encode/percent_encode.case.ad | percent_encode.expected.rs | 686d48c6b129d305 | bdd264c0bb760c0b |
| N1009 | cookbook / encoding/014_url_encode | crates/auto-lang/test/cookbook/encoding/014_url_encode/url_encode.case.ad | url_encode.expected.rs | 1ffa8ef181d5861b | 9327009c8f22d675 |
| N1010 | cookbook / errors/001_boxed_error | crates/auto-lang/test/cookbook/errors/001_boxed_error/boxed_error.case.ad | boxed_error.expected.rs | ec3c342dba6f8b86 | 9a683bb5fccd1fd6 |
| N1011 | cookbook / errors/002_anyhow | crates/auto-lang/test/cookbook/errors/002_anyhow/anyhow.case.ad | anyhow.expected.rs | 9657a94faf57318b | 5bf0b7ec80510f2f |
| N1012 | cookbook / errors/003_backtrace | crates/auto-lang/test/cookbook/errors/003_backtrace/backtrace.case.ad | backtrace.expected.rs | 690b77a13eabfcdb | ce64775eb749b3de |
| N1013 | cookbook / errors/004_retain | crates/auto-lang/test/cookbook/errors/004_retain/retain.case.ad | retain.expected.rs | e11d22ddd2cc673a | dda2df9adb722e18 |
| N1014 | cookbook / file/001_read_lines | crates/auto-lang/test/cookbook/file/001_read_lines/read_lines.case.ad | read_lines.expected.rs | 5b58f41a2e917112 | 4afba68f8c8172e6 |
| N1015 | cookbook / file/002_find_files | crates/auto-lang/test/cookbook/file/002_find_files/find_files.case.ad | find_files.expected.rs | a71bea4660d35aef | e9750652255cd130 |
| N1016 | cookbook / file/003_recursive_size | crates/auto-lang/test/cookbook/file/003_recursive_size/recursive_size.case.ad | recursive_size.expected.rs | 404311bd46bd995a | 7e30f285b463386b |
| N1017 | cookbook / file/004_modified | crates/auto-lang/test/cookbook/file/004_modified/modified.case.ad | modified.expected.rs | f08fad05d7c41e42 | e59c25ade8f74d38 |
| N1018 | cookbook / file/005_duplicate_name | crates/auto-lang/test/cookbook/file/005_duplicate_name/duplicate_name.case.ad | duplicate_name.expected.rs | 8d79efc99a74480a | f4faeaf3b8b91d7a |
| N1019 | cookbook / file/006_find_file | crates/auto-lang/test/cookbook/file/006_find_file/find_file.case.ad | find_file.expected.rs | 6d91d25891820ba0 | 7ff9f24836784369 |
| N1020 | cookbook / file/007_ignore_case | crates/auto-lang/test/cookbook/file/007_ignore_case/ignore_case.case.ad | ignore_case.expected.rs | 7fa489a0549bd132 | 1df6ca81966ae1ed |
| N1021 | cookbook / file/008_loops | crates/auto-lang/test/cookbook/file/008_loops/loops.case.ad | loops.expected.rs | e3c7f43a568eaf1b | 7ff081c3de0477bc |
| N1022 | cookbook / file/009_png | crates/auto-lang/test/cookbook/file/009_png/png.case.ad | png.expected.rs | e53eb7c25d6f068b | 4497f938b0661d43 |
| N1023 | cookbook / file/010_recursive | crates/auto-lang/test/cookbook/file/010_recursive/recursive.case.ad | recursive.expected.rs | c661cc45636b4fa5 | b313022e43799498 |
| N1024 | cookbook / file/011_sizes | crates/auto-lang/test/cookbook/file/011_sizes/sizes.case.ad | sizes.expected.rs | 2dd74e7b29d234bc | 05fa70229786abff |
| N1025 | cookbook / file/012_skip_dot | crates/auto-lang/test/cookbook/file/012_skip_dot/skip_dot.case.ad | skip_dot.expected.rs | 1ec3c75e7d1dbc0a | 20ba7522b5426258 |
| N1026 | cookbook / file/013_same_file | crates/auto-lang/test/cookbook/file/013_same_file/same_file.case.ad | same_file.expected.rs | 5bacf44bf0c21318 | 8e787c5ba023c424 |
| N1027 | cookbook / file/014_read_lines_temp | crates/auto-lang/test/cookbook/file/014_read_lines_temp/read_lines_temp.case.ad | read_lines_temp.expected.rs | e5f4b37c1417e7a0 | 009b18ea20c488e3 |
| N1028 | cookbook / hardware/001_cpu_count | crates/auto-lang/test/cookbook/hardware/001_cpu_count/cpu_count.case.ad | cpu_count.expected.rs | b96745556f10813b | 9530292982ecda71 |
| N1029 | cookbook / mem/001_lazy_cell | crates/auto-lang/test/cookbook/mem/001_lazy_cell/lazy_cell.case.ad | lazy_cell.expected.rs | e144bc179f451df2 | 13efbe31d14b778b |
| N1030 | cookbook / os/001_env_variable | crates/auto-lang/test/cookbook/os/001_env_variable/env_variable.case.ad | env_variable.expected.rs | a791b00f081e8cf2 | 8eb488e23c201cd3 |
| N1031 | cookbook / os/002_process_continuous | crates/auto-lang/test/cookbook/os/002_process_continuous/process_continuous.case.ad | process_continuous.expected.rs | 5e80873c6f7255e8 | 1708d1ed9a591aff |
| N1032 | cookbook / os/003_error_file | crates/auto-lang/test/cookbook/os/003_error_file/error_file.case.ad | error_file.expected.rs | f5492090376af242 | d8d00d03ba9fa76f |
| N1033 | cookbook / os/004_piped | crates/auto-lang/test/cookbook/os/004_piped/piped.case.ad | piped.expected.rs | b73f5d410555c17c | 5fb229ca4411637d |
| N1034 | cookbook / os/005_process_output | crates/auto-lang/test/cookbook/os/005_process_output/process_output.case.ad | process_output.expected.rs | db568b9d967946bf | 4b7fa62c38f45740 |
| N1035 | cookbook / os/006_send_input | crates/auto-lang/test/cookbook/os/006_send_input/send_input.case.ad | send_input.expected.rs | 915a034a318de5f8 | e5cc812be91f2540 |
| N1036 | cookbook / safety/001_heapless | crates/auto-lang/test/cookbook/safety/001_heapless/heapless.case.ad | heapless.expected.rs | 2f5d35fa2498ccba | bebe2eef1ebc9a6d |
| N1037 | cookbook / safety/001_memmap | crates/auto-lang/test/cookbook/safety/001_memmap/memmap.case.ad | memmap.expected.rs | 59e6e01d781e1585 | b46dd688764e8dd9 |
| N1038 | cookbook / text/001_regex_replace | crates/auto-lang/test/cookbook/text/001_regex_replace/regex_replace.case.ad | regex_replace.expected.rs | a64b94ac5d4c3a3e | 763cca1718b336de |
| N1039 | cookbook / text/002_regex_email | crates/auto-lang/test/cookbook/text/002_regex_email/regex_email.case.ad | regex_email.expected.rs | 63e77d6390419e0b | 563311873ba395a6 |
| N1040 | cookbook / text/003_regex_hashtags | crates/auto-lang/test/cookbook/text/003_regex_hashtags/regex_hashtags.case.ad | regex_hashtags.expected.rs | 75a762d818bb95aa | 5ecd7dbbe64f2669 |
| N1041 | cookbook / text/004_graphemes | crates/auto-lang/test/cookbook/text/004_graphemes/graphemes.case.ad | graphemes.expected.rs | 1f869dcc01e0d345 | c7672861c057bd47 |
| N1042 | cookbook / text/005_filter_log | crates/auto-lang/test/cookbook/text/005_filter_log/filter_log.case.ad | filter_log.expected.rs | 670926ae47498bd5 | d4cafed10ee235ca |
| N1043 | cookbook / text/006_phone | crates/auto-lang/test/cookbook/text/006_phone/phone.case.ad | phone.expected.rs | 821164cb6d3273d5 | 9857c07d6bdb27d1 |
| N1044 | cookbook / text/007_from_str | crates/auto-lang/test/cookbook/text/007_from_str/from_str.case.ad | from_str.expected.rs | ae0f39e347ee56d5 | d2abc896b600b286 |
| N1045 | cookbook / versioning/001_semver_parse | crates/auto-lang/test/cookbook/versioning/001_semver_parse/semver_parse.case.ad | semver_parse.expected.rs | a349c8f07b9ebf5d | 2c35b51d11017b76 |
| N1046 | cookbook / versioning/002_semver_increment | crates/auto-lang/test/cookbook/versioning/002_semver_increment/semver_increment.case.ad | semver_increment.expected.rs | abb794a8ad2c9271 | 3e5974b12c229364 |
| N1047 | cookbook / versioning/003_semver_latest | crates/auto-lang/test/cookbook/versioning/003_semver_latest/semver_latest.case.ad | semver_latest.expected.rs | a8a3842d289b0621 | 5c4a6c0f9bdda7c5 |
| N1048 | cookbook / versioning/004_semver_command | crates/auto-lang/test/cookbook/versioning/004_semver_command/semver_command.case.ad | semver_command.expected.rs | 4e7d4de51876af7f | 61b56d25d4f7d323 |
| N1049 | cookbook / versioning/005_semver_complex | crates/auto-lang/test/cookbook/versioning/005_semver_complex/semver_complex.case.ad | semver_complex.expected.rs | 043b54e2c9ac16eb | 6f2c5e85647d0dcd |
| N1050 | cookbook / versioning/006_semver_prerelease | crates/auto-lang/test/cookbook/versioning/006_semver_prerelease/semver_prerelease.case.ad | semver_prerelease.expected.rs | 5362c8b68b1406ce | 36a7935170ed1c4e |

### 附录C：33独立验证与30个proposal输入逐项映射

源/旧expected位于各doc同目录的input.at、expected_output.txt。普通验证原Rust namespace为tests::conformance_tests::<表内函数>。proposal按原30闭集，输出只在scope，绝不回写这些作者路径。

| 行 | 原case目录ID | 原验证test（保留） | 生成/验证角色 | 权威doc（repo相对） | doc SHA-256 |
|---|---|---|---|---|---|
| C01 | 001_int_add | conformance_001_int_add | proposal+验证 | crates/auto-lang/test/a2r/conformance/001_int_add/input.case.ad | 926125302979cfef2ce960713b615c4249431e59c44d753da4cdaf2910defaa8 |
| C02 | 002_str_concat | conformance_002_str_concat | proposal+验证 | crates/auto-lang/test/a2r/conformance/002_str_concat/input.case.ad | fe8aefab1138cb75033f191193411a838f967de825a50c62ee5036d67f5113c2 |
| C03 | 003_if_else | conformance_003_if_else | proposal+验证 | crates/auto-lang/test/a2r/conformance/003_if_else/input.case.ad | f7383fc451bcb2be7c30b155baf47d78657f7d7dc696e40194a17c2d93554693 |
| C04 | 004_for_range | conformance_004_for_range | proposal+验证 | crates/auto-lang/test/a2r/conformance/004_for_range/input.case.ad | 9eccb79523eb6e6398bcf0e9a0c6373d0fd718e7dd159b8d4cde347adca37edc |
| C05 | 005_fstring | conformance_005_fstring | proposal+验证 | crates/auto-lang/test/a2r/conformance/005_fstring/input.case.ad | d094264bb2cce39f50612705f5d7af629a87c6a725228b390ba4d00161b1823e |
| C06 | 006_struct_access | conformance_006_struct_access | proposal+验证 | crates/auto-lang/test/a2r/conformance/006_struct_access/input.case.ad | 18861136510245501beda599ca00de1c49f6c6346c8d7aa45249b93a19a381b5 |
| C07 | 007_func_call | conformance_007_func_call | proposal+验证 | crates/auto-lang/test/a2r/conformance/007_func_call/input.case.ad | 8289ed9cd3d0717fc3b31975aa96e2280aea2118bffdcef4b0ae79570af10124 |
| C08 | 008_arithmetic | conformance_008_arithmetic | proposal+验证 | crates/auto-lang/test/a2r/conformance/008_arithmetic/input.case.ad | c9079bee64b0cb06c0ea3f1b70efc4b545e67d2c70ca97749309ed8573e9c370 |
| C09 | 009_comparison | conformance_009_comparison | proposal+验证 | crates/auto-lang/test/a2r/conformance/009_comparison/input.case.ad | f06f42d2ecb56a49cbc232689bc1ab5554d8382c7a77584cc025fc33d6c886f4 |
| C10 | 010_array_index | conformance_010_array_index | proposal+验证 | crates/auto-lang/test/a2r/conformance/010_array_index/input.case.ad | cdb83b8eada3274a451891f88494be3b47aeea17911bbb276a30446bcdbd2864 |
| C11 | 011_int_div | conformance_011_int_div | proposal+验证 | crates/auto-lang/test/a2r/conformance/011_int_div/input.case.ad | d20f49cd46b1065a177fd9e983cb1b77ff077f92db062f6374afd70d2fecb2df |
| C12 | 012_int_mod | conformance_012_int_mod | proposal+验证 | crates/auto-lang/test/a2r/conformance/012_int_mod/input.case.ad | b1b460696c7379e214f5fce3d2157ac9892b267e29df0299fbf74ef75d5349c6 |
| C13 | 013_negation | conformance_013_negation | proposal+验证 | crates/auto-lang/test/a2r/conformance/013_negation/input.case.ad | 887c3ac418e03ef4092fac6a772cdb4d42e16992a910df1a8ce37df82aa2838e |
| C14 | 014_float_arith | conformance_014_float_arith | proposal+验证 | crates/auto-lang/test/a2r/conformance/014_float_arith/input.case.ad | 36aefe43f9b3bcb8fd3f81ff0b9d8005caa560129e6a1827330a21b1d9f7351c |
| C15 | 015_nested_if | conformance_015_nested_if | proposal+验证 | crates/auto-lang/test/a2r/conformance/015_nested_if/input.case.ad | d84a90e611e182a07280e6a1ec55ef8edb1e39648e893dcded11b9719ac30034 |
| C16 | 016_loop_break | conformance_016_loop_break | proposal+验证 | crates/auto-lang/test/a2r/conformance/016_loop_break/input.case.ad | 06b2f49ac38db4d36bf9d27f9694e59728577305bb7ae31cc0336d5e71358902 |
| C17 | 017_loop_continue | conformance_017_loop_continue | proposal+验证 | crates/auto-lang/test/a2r/conformance/017_loop_continue/input.case.ad | 44cf57e31b86adba4be9df35a41faeafab4834ea185919f27beeaa52f6238c42 |
| C18 | 018_for_iterator | conformance_018_for_iterator | proposal+验证 | crates/auto-lang/test/a2r/conformance/018_for_iterator/input.case.ad | ecce0d5291ccb4b9fdbb3e4370f4beea29959c52b06236f80d6620a12e8c987f |
| C19 | 019_enum_scalar | conformance_019_enum_scalar | proposal+验证 | crates/auto-lang/test/a2r/conformance/019_enum_scalar/input.case.ad | 9bbbdc02f13e7b1b859b45e3b068d88f77afba735717752863135dc85242a0a6 |
| C20 | 020_enum_match | conformance_020_enum_match | proposal+验证 | crates/auto-lang/test/a2r/conformance/020_enum_match/input.case.ad | ca3195c0703ff6f97709f9f5a136508f16328ea65eb0f669f766a06db01df959 |
| C21 | 021_string_methods | conformance_021_string_methods | proposal+验证 | crates/auto-lang/test/a2r/conformance/021_string_methods/input.case.ad | 157e028234950504b20a55cc3e54941d7900395bccc82a8d3019f8ce9544b990 |
| C22 | 022_list_push_pop | conformance_022_list_push_pop | proposal+验证 | crates/auto-lang/test/a2r/conformance/022_list_push_pop/input.case.ad | bfc1663ba25ac68999e7d23679af763545c974133aa035aca7cb8930e01a0585 |
| C23 | 023_list_map_filter | conformance_023_list_map_filter | proposal+验证 | crates/auto-lang/test/a2r/conformance/023_list_map_filter/input.case.ad | 1a9e709b4d6d864e41d277d6301b1c27f755da391debdb5a4e1e2f4e00fa2436 |
| C24 | 024_map_basic | conformance_024_map_basic | proposal+验证 | crates/auto-lang/test/a2r/conformance/024_map_basic/input.case.ad | cfb661b555a9e995feb6c61c7412dc9cd3608c53a1f279e0eda680dde82d3ec4 |
| C25 | 025_option_basic | conformance_025_option_basic | proposal+验证 | crates/auto-lang/test/a2r/conformance/025_option_basic/input.case.ad | 4befd908b884da58d13c70fba42b4edaa8ce82d2044a18661b8f3eb32b75236d |
| C26 | 026_result_basic | conformance_026_result_basic | proposal+验证 | crates/auto-lang/test/a2r/conformance/026_result_basic/input.case.ad | 1784cdb11b6955564d20f5f9b4ec5f25b15e85aca6b0df743053cd9a80f26f7d |
| C27 | 027_nested_func | conformance_027_nested_func | proposal+验证 | crates/auto-lang/test/a2r/conformance/027_nested_func/input.case.ad | 7cf92082efdf3bb53018ae579fc9c07200aa3f6944592d20320168c07f7096a4 |
| C28 | 028_recursive_func | conformance_028_recursive_func | proposal+验证 | crates/auto-lang/test/a2r/conformance/028_recursive_func/input.case.ad | b4d28f4630fc2bd0235d8fd7a93850d96b1790fb710c999641f72026fedc9ece |
| C29 | 029_multi_param | conformance_029_multi_param | proposal+验证 | crates/auto-lang/test/a2r/conformance/029_multi_param/input.case.ad | 59ed4e5a1c0d383afa9032511cd2a25a9abad6f5e465e12d8d2889d9e273b919 |
| C30 | 030_str_to_int | conformance_030_str_to_int | proposal+验证 | crates/auto-lang/test/a2r/conformance/030_str_to_int/input.case.ad | fbe3d2f16997217fa563139e426fe9159b0f4a46b521ea191cc13dd917f36513 |
| C31 | 040_while_basic | conformance_040_while_basic | 仅验证；不加入30例proposal | crates/auto-lang/test/a2r/conformance/040_while_basic/input.case.ad | ba1f594712e740e8d96ef0c969ba8e535e39c286a014a5c809b09ec5b6576ccb |
| C32 | 042_try_no_error | conformance_042_try_no_error | 仅验证；不加入30例proposal | crates/auto-lang/test/a2r/conformance/042_try_no_error/input.case.ad | f9b1f199ae860ab04a6dd66621eaa9e96e36f6e2eb3e646692181a62fd65845c |
| C33 | 043_try_catch_param | conformance_043_try_catch_param | 仅验证；不加入30例proposal | crates/auto-lang/test/a2r/conformance/043_try_catch_param/input.case.ad | 49366cb6a16f84ef7e331650ff6131ddc89d3ce51f59d495c8a45a2d41d34e80 |

### 附录E（r2）：78份已有文档的其他消费者边界

这些文档不属原N1050目录选择集合，但各自原包装/ignore/闭集继续有效；不随意启用或批量ignore。本表精确记录当前文档路径/hash，用于资格反控；新P两份工程doc另属P消费者，不误入N。C的33份与附录C重复引用，不是新增33个独立case。T-01核各实际owner，最终新增项继续纳入对账。

| 行 | family | 文档路径（repo相对） | SHA-256 |
|---|---|---|---|
| E01 | vm | crates/auto-lang/test/vm/08_strings/017_sub_semantics/sub_semantics.case.ad | be9632c087914a67debca2c983b94a3ecec4065af93c3e55e891476b7eb36b7d |
| E02 | vm | crates/auto-lang/test/vm/10_types/033_array_len_user_type/array_len_user_type.case.ad | 65dac91f9a629f1ff9f2935b4a05b0a4bf60c722b35b45f9049d043fb3400198 |
| E03 | vm | crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.case.ad | 8fed14f1168cb2b815939f001408d6d637ebc9d9297bc6766b68667d19fa10c6 |
| E04 | vm | crates/auto-lang/test/vm/17_cffi/002_math/math.case.ad | 216d2ca4468dc916d58a0fcfd6762a632620934ff6f110dd84fdf3fe33b9a46a |
| E05 | vm | crates/auto-lang/test/vm/17_cffi/003_string/string.case.ad | 6e0f93d3bc7a4c8b01b05e5c733e2cdd56e7cfb48f44a39fbbb85e4b007c6f16 |
| E06 | vm | crates/auto-lang/test/vm/18_ffi/047_time_now/time_now.case.ad | 6542df75885fa8db5752576455dbb81adbff11251c6f05df967708bab6e82243 |
| E07 | vm | crates/auto-lang/test/vm/18_ffi/055_process_args/process_args.case.ad | 12a743a75480b2e3e1fa6ad81a33714dbbef71857bd6d350e20aa7528ccfec6c |
| E08 | vm | crates/auto-lang/test/vm/99_bootstrap/066_bytecode_list/bytecode_list.case.ad | 50966acb8c09c30a2f0e9b5b3573bf56e09f1e2039c979bb496b06ae24733e82 |
| E09 | vm | crates/auto-lang/test/vm/99_bootstrap/067_bytecode_map/bytecode_map.case.ad | 586545f3f4c51aca71ed8e0748378e3aacc0470a130c0ec5570507f0c5903ae5 |
| E10 | vm | crates/auto-lang/test/vm/99_bootstrap/068_bytecode_list_fn/bytecode_list_fn.case.ad | 545da1974338b6e98b32f98462fd816455451f94aab78843db01f6966c4dde3e |
| E11 | a2r | crates/auto-lang/test/a2r/19_ownership/007_self_ref/self_ref.case.ad | 960015d54b7b7ac21395f09d2cb885c617f4495f33b5a1d3d8f412246056c31a |
| E12 | a2r | crates/auto-lang/test/a2r/conformance/001_int_add/input.case.ad | 926125302979cfef2ce960713b615c4249431e59c44d753da4cdaf2910defaa8 |
| E13 | a2r | crates/auto-lang/test/a2r/conformance/002_str_concat/input.case.ad | fe8aefab1138cb75033f191193411a838f967de825a50c62ee5036d67f5113c2 |
| E14 | a2r | crates/auto-lang/test/a2r/conformance/003_if_else/input.case.ad | f7383fc451bcb2be7c30b155baf47d78657f7d7dc696e40194a17c2d93554693 |
| E15 | a2r | crates/auto-lang/test/a2r/conformance/004_for_range/input.case.ad | 9eccb79523eb6e6398bcf0e9a0c6373d0fd718e7dd159b8d4cde347adca37edc |
| E16 | a2r | crates/auto-lang/test/a2r/conformance/005_fstring/input.case.ad | d094264bb2cce39f50612705f5d7af629a87c6a725228b390ba4d00161b1823e |
| E17 | a2r | crates/auto-lang/test/a2r/conformance/006_struct_access/input.case.ad | 18861136510245501beda599ca00de1c49f6c6346c8d7aa45249b93a19a381b5 |
| E18 | a2r | crates/auto-lang/test/a2r/conformance/007_func_call/input.case.ad | 8289ed9cd3d0717fc3b31975aa96e2280aea2118bffdcef4b0ae79570af10124 |
| E19 | a2r | crates/auto-lang/test/a2r/conformance/008_arithmetic/input.case.ad | c9079bee64b0cb06c0ea3f1b70efc4b545e67d2c70ca97749309ed8573e9c370 |
| E20 | a2r | crates/auto-lang/test/a2r/conformance/009_comparison/input.case.ad | f06f42d2ecb56a49cbc232689bc1ab5554d8382c7a77584cc025fc33d6c886f4 |
| E21 | a2r | crates/auto-lang/test/a2r/conformance/010_array_index/input.case.ad | cdb83b8eada3274a451891f88494be3b47aeea17911bbb276a30446bcdbd2864 |
| E22 | a2r | crates/auto-lang/test/a2r/conformance/011_int_div/input.case.ad | d20f49cd46b1065a177fd9e983cb1b77ff077f92db062f6374afd70d2fecb2df |
| E23 | a2r | crates/auto-lang/test/a2r/conformance/012_int_mod/input.case.ad | b1b460696c7379e214f5fce3d2157ac9892b267e29df0299fbf74ef75d5349c6 |
| E24 | a2r | crates/auto-lang/test/a2r/conformance/013_negation/input.case.ad | 887c3ac418e03ef4092fac6a772cdb4d42e16992a910df1a8ce37df82aa2838e |
| E25 | a2r | crates/auto-lang/test/a2r/conformance/014_float_arith/input.case.ad | 36aefe43f9b3bcb8fd3f81ff0b9d8005caa560129e6a1827330a21b1d9f7351c |
| E26 | a2r | crates/auto-lang/test/a2r/conformance/015_nested_if/input.case.ad | d84a90e611e182a07280e6a1ec55ef8edb1e39648e893dcded11b9719ac30034 |
| E27 | a2r | crates/auto-lang/test/a2r/conformance/016_loop_break/input.case.ad | 06b2f49ac38db4d36bf9d27f9694e59728577305bb7ae31cc0336d5e71358902 |
| E28 | a2r | crates/auto-lang/test/a2r/conformance/017_loop_continue/input.case.ad | 44cf57e31b86adba4be9df35a41faeafab4834ea185919f27beeaa52f6238c42 |
| E29 | a2r | crates/auto-lang/test/a2r/conformance/018_for_iterator/input.case.ad | ecce0d5291ccb4b9fdbb3e4370f4beea29959c52b06236f80d6620a12e8c987f |
| E30 | a2r | crates/auto-lang/test/a2r/conformance/019_enum_scalar/input.case.ad | 9bbbdc02f13e7b1b859b45e3b068d88f77afba735717752863135dc85242a0a6 |
| E31 | a2r | crates/auto-lang/test/a2r/conformance/020_enum_match/input.case.ad | ca3195c0703ff6f97709f9f5a136508f16328ea65eb0f669f766a06db01df959 |
| E32 | a2r | crates/auto-lang/test/a2r/conformance/021_string_methods/input.case.ad | 157e028234950504b20a55cc3e54941d7900395bccc82a8d3019f8ce9544b990 |
| E33 | a2r | crates/auto-lang/test/a2r/conformance/022_list_push_pop/input.case.ad | bfc1663ba25ac68999e7d23679af763545c974133aa035aca7cb8930e01a0585 |
| E34 | a2r | crates/auto-lang/test/a2r/conformance/023_list_map_filter/input.case.ad | 1a9e709b4d6d864e41d277d6301b1c27f755da391debdb5a4e1e2f4e00fa2436 |
| E35 | a2r | crates/auto-lang/test/a2r/conformance/024_map_basic/input.case.ad | cfb661b555a9e995feb6c61c7412dc9cd3608c53a1f279e0eda680dde82d3ec4 |
| E36 | a2r | crates/auto-lang/test/a2r/conformance/025_option_basic/input.case.ad | 4befd908b884da58d13c70fba42b4edaa8ce82d2044a18661b8f3eb32b75236d |
| E37 | a2r | crates/auto-lang/test/a2r/conformance/026_result_basic/input.case.ad | 1784cdb11b6955564d20f5f9b4ec5f25b15e85aca6b0df743053cd9a80f26f7d |
| E38 | a2r | crates/auto-lang/test/a2r/conformance/027_nested_func/input.case.ad | 7cf92082efdf3bb53018ae579fc9c07200aa3f6944592d20320168c07f7096a4 |
| E39 | a2r | crates/auto-lang/test/a2r/conformance/028_recursive_func/input.case.ad | b4d28f4630fc2bd0235d8fd7a93850d96b1790fb710c999641f72026fedc9ece |
| E40 | a2r | crates/auto-lang/test/a2r/conformance/029_multi_param/input.case.ad | 59ed4e5a1c0d383afa9032511cd2a25a9abad6f5e465e12d8d2889d9e273b919 |
| E41 | a2r | crates/auto-lang/test/a2r/conformance/030_str_to_int/input.case.ad | fbe3d2f16997217fa563139e426fe9159b0f4a46b521ea191cc13dd917f36513 |
| E42 | a2r | crates/auto-lang/test/a2r/conformance/040_while_basic/input.case.ad | ba1f594712e740e8d96ef0c969ba8e535e39c286a014a5c809b09ec5b6576ccb |
| E43 | a2r | crates/auto-lang/test/a2r/conformance/042_try_no_error/input.case.ad | f9b1f199ae860ab04a6dd66621eaa9e96e36f6e2eb3e646692181a62fd65845c |
| E44 | a2r | crates/auto-lang/test/a2r/conformance/043_try_catch_param/input.case.ad | 49366cb6a16f84ef7e331650ff6131ddc89d3ce51f59d495c8a45a2d41d34e80 |
| E45 | a2c | crates/auto-lang/test/a2c/19_option_type/017_question_logical/question_logical.case.ad | a4871ee43ce8e34dba9e7f4097de2e5ae100c00d189929935043314a3450914a |
| E46 | a2c | crates/auto-lang/test/a2c/200_auto_cffi_math/auto_cffi_math.case.ad | c9224e7bee19b1513d02e60a95ccba998e2cbf90b7055e6d8fa43f207c32993f |
| E47 | cookbook | crates/auto-lang/test/cookbook/asynchronous/channel/001_bounded.case.ad | 48a8efecb7b10d2b187b7c878229479fc869d16cde1d56e4db1c9dbc933c8ad6 |
| E48 | cookbook | crates/auto-lang/test/cookbook/asynchronous/fs/001_create.case.ad | 9949bff48ed42101454b0fd12bd35a7f158167518dee45eedb252679f03edd08 |
| E49 | cookbook | crates/auto-lang/test/cookbook/asynchronous/ftc/001_ctrl_c.case.ad | 774aa2f8746576d8c964cb33be7a8012472f6717bc4882ee437d845930f7a312 |
| E50 | cookbook | crates/auto-lang/test/cookbook/asynchronous/rt/001_tokio_macro.case.ad | 0c493af14af7455dc1218a7cdba621b7d48675ffd1cd94b213e3e3d9f96cf04c |
| E51 | cookbook | crates/auto-lang/test/cookbook/database/postgres/001_create_tables.case.ad | 5a16db49a1853bf44a1ce757351bc8c19e409e107d54fddcd2783fb3a9410f54 |
| E52 | cookbook | crates/auto-lang/test/cookbook/database/sqlite/001_init.case.ad | 094f012f83b13a2e7f7675162274c6aac4480e1834b1e2e86c54b42d83ad6983 |
| E53 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/complex_numbers/001_add_complex/add_complex.case.ad | c688c4e43b5edbb4fc40d361bbeff1b767414de787a1daac4e4af73757e00be0 |
| E54 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/complex_numbers/002_create_complex/create_complex.case.ad | 608d4445cec8d74dafd51ac1bbdb3e4d2c9182f442b19cbc4d61cb0e7b15e687 |
| E55 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/complex_numbers/003_math_functions/math_functions.case.ad | b260eacc65932478a38f3c5f1056bae732aa5ce815cd8144e07d7a5939931c3b |
| E56 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/linear_algebra/001_add_matrices/add_matrices.case.ad | e4c955771537479e08037f6a6f949f231dfae1ff3a27af8965c660ecf79d7369 |
| E57 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/linear_algebra/002_multiply_matrices/multiply_matrices.case.ad | 2178d771198ed879bfc5df9ba73753831ee31de079279811478e44bc2e98af3a |
| E58 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/linear_algebra/003_multiply_svm/multiply_svm.case.ad | d89510c2da4bfb846eb660a697e9b73615e58ad6c199928170e48500129022d9 |
| E59 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/linear_algebra/004_vector_comparison/vector_comparison.case.ad | afda241ed0e655ee35daba2b0fc14c1cab4bf927facc8c40fc69a0dfc50634f6 |
| E60 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/linear_algebra/005_vector_norm/vector_norm.case.ad | f3b77e0a4663d135f5ad10da0cd020eca8866101ff793f82c00d0535e4959f95 |
| E61 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/linear_algebra/006_invert_matrix/invert_matrix.case.ad | 0c8486beebea00c03ddc715b30221ba5aab7ae93bc02c3fd617e7918224f70ee |
| E62 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/linear_algebra/007_deserialize_matrix/deserialize_matrix.case.ad | 1b14919fcb9bd682d83e0561356e8ce0d0747bf6277667fa20e123339a97dc12 |
| E63 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/miscellaneous/001_big_integers/big_integers.case.ad | a8cde7c03e5092cf8452478199e8739dc8e4ecf7150bd4355b29a605df7071dd |
| E64 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/miscellaneous/002_math_functions/math_functions.case.ad | 29e1a24a4b38202ba494e60bae031ba9d8d9e4334af2f6c61dab47cc30cb2fb6 |
| E65 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/statistics/001_central_tendency/central_tendency.case.ad | cccf082d6f772424b80e98e1feaa735da4f0f065d5300e3f18f79b9ac7e1f5b5 |
| E66 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/statistics/002_standard_deviation/standard_deviation.case.ad | bed4389988dc63046f0edbe6c7532c4247d25c036740c24c33424988c9888a07 |
| E67 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/trigonometry/001_tan_sin_cos/tan_sin_cos.case.ad | 866673c6820b9025d3cb7736f30c69f10ca54e73d037dde647f8774a5fbd8ad1 |
| E68 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/trigonometry/002_side_length/side_length.case.ad | d1ffb458e4f2e391d3e315e9f4ae34abad3dbd1eb0fa60c7916be99911e3c7fe |
| E69 | cookbook | crates/auto-lang/test/cookbook/science/mathematics/trigonometry/003_latitude_longitude/latitude_longitude.case.ad | d24369e2dfdf5b0f22ee68ccf51c76482c690432a7e6643d3e3d63a762b69c9c |
| E70 | cookbook | crates/auto-lang/test/cookbook/web/mime/001_filename/filename.case.ad | c73669638ce0813ab5b515cfccc4ca716edb0eba3186a85b6410a4dd15655980 |
| E71 | cookbook | crates/auto-lang/test/cookbook/web/mime/001_request.case.ad | 89965deb7226377e7f0473fc1fad50c48346f0e7ca05355657b42cd3e78f2fc1 |
| E72 | cookbook | crates/auto-lang/test/cookbook/web/mime/002_string/string.case.ad | 92d0d5c34bf318a9f8418ad100706ebe76a71e43d5e674624478cf586b347b61 |
| E73 | cookbook | crates/auto-lang/test/cookbook/web/scraping/001_extract_links.case.ad | 0e0901759b9f86add10ea5bf6093edbfd7b0e76a56491304bdf4392b60eb57d2 |
| E74 | cookbook | crates/auto-lang/test/cookbook/web/url/001_base/base.case.ad | abd7c48b4b31531cae3580f3fb56e11d91ac04952f91b8dfb6ff933fc7434d17 |
| E75 | cookbook | crates/auto-lang/test/cookbook/web/url/002_parse/parse.case.ad | d8aa927a68ce7b592e6695f7f149b4117fbbeb37d4bb7bcfd19f47c25892b0c0 |
| E76 | cookbook | crates/auto-lang/test/cookbook/web/url/003_fragment/fragment.case.ad | dfc1f833247a4ff98a2dd3cdcb496173ab1498427f71af7069136abf77ac34b1 |
| E77 | cookbook | crates/auto-lang/test/cookbook/web/url/004_new/new.case.ad | 5df9b86ac3c431bad59851c8a2abdecefd387910a4ddf84b46262d080a2af69f |
| E78 | cookbook | crates/auto-lang/test/cookbook/web/url/005_origin/origin.case.ad | 38e1ca4a918e758052c6525db625e6619d154641b36649d48de8b74ddea80ef6 |

### 附录B（r2）：cookbook VM全部108个包装绑定与候选文档

helper原目录前缀为crates/auto-lang/test/cookbook/。单源目录的目标doc确定；6个多源目录列全部旧候选，最后一栏要求T-01以原同源Rust实际选中项/receipt核定。当前列表是静态声明/文件盘点，Windows条件77 active/30 ignore/1 cfg_attr ignore须与真正注册列表联接；无doc候选不擅升级为新活跃例，若证明旧实际选中即在本Plan补齐。禁止靠删除旧source或换成排序选项改变语义。

| 行 | 原Rust test | helper目录参数 | 原声明条件 | 原源对应的doc候选 | 目标/关闭条件 |
|---|---|---|---|---|---|
| B001 | tests::cookbook_vm_tests::cb_algorithms_sort_int | algorithms/001_sort_int | active声明 | sort_int.case.ad | sort_int.case.ad |
| B002 | tests::cookbook_vm_tests::cb_algorithms_sort_float | algorithms/002_sort_float | active声明 | sort_float.case.ad | sort_float.case.ad |
| B003 | tests::cookbook_vm_tests::cb_algorithms_sort_struct | algorithms/003_sort_struct | active声明 | sort_struct.case.ad | sort_struct.case.ad |
| B004 | tests::cookbook_vm_tests::cb_algorithms_rand | algorithms/004_rand | ignored | rand.case.ad | rand.case.ad |
| B005 | tests::cookbook_vm_tests::cb_algorithms_rand_choose | algorithms/005_rand_choose | ignored | rand_choose.case.ad | rand_choose.case.ad |
| B006 | tests::cookbook_vm_tests::cb_algorithms_rand_custom | algorithms/006_rand_custom | ignored | rand_custom.case.ad | rand_custom.case.ad |
| B007 | tests::cookbook_vm_tests::cb_algorithms_rand_dist | algorithms/007_rand_dist | ignored | rand_dist.case.ad | rand_dist.case.ad |
| B008 | tests::cookbook_vm_tests::cb_algorithms_rand_passwd | algorithms/008_rand_passwd | ignored | rand_passwd.case.ad | rand_passwd.case.ad |
| B009 | tests::cookbook_vm_tests::cb_algorithms_rand_range | algorithms/009_rand_range | ignored | rand_range.case.ad | rand_range.case.ad |
| B010 | tests::cookbook_vm_tests::cb_algorithms_rand_custom_2 | algorithms/010_rand_custom | ignored | rand_custom.case.ad | rand_custom.case.ad |
| B011 | tests::cookbook_vm_tests::cb_algorithms_rand_dist_2 | algorithms/011_rand_dist | ignored | rand_dist.case.ad | rand_dist.case.ad |
| B012 | tests::cookbook_vm_tests::cb_asynchronous_channel | asynchronous/channel | active声明 | 001_bounded.case.ad, 002_unbounded.case.ad（无doc，旧实选待核，不得擅启用） | T-01核旧实选，禁止按排序换源 |
| B013 | tests::cookbook_vm_tests::cb_asynchronous_fs | asynchronous/fs | active声明 | 001_create.case.ad, 002_read.case.ad（无doc，旧实选待核，不得擅启用）, 003_remove.case.ad（无doc，旧实选待核，不得擅启用）, 004_rw_traits.case.ad（无doc，旧实选待核，不得擅启用）, 005_write.case.ad（无doc，旧实选待核，不得擅启用） | T-01核旧实选，禁止按排序换源 |
| B014 | tests::cookbook_vm_tests::cb_asynchronous_ftc | asynchronous/ftc | active声明 | 001_ctrl_c.case.ad | 001_ctrl_c.case.ad |
| B015 | tests::cookbook_vm_tests::cb_asynchronous_rt | asynchronous/rt | active声明 | 001_tokio_macro.case.ad, 002_tokio_builder.case.ad（无doc，旧实选待核，不得擅启用） | T-01核旧实选，禁止按排序换源 |
| B016 | tests::cookbook_vm_tests::cb_cli_clap_basic | cli/001_clap_basic | active声明 | clap_basic.case.ad | clap_basic.case.ad |
| B017 | tests::cookbook_vm_tests::cb_cli_ansi_term | cli/002_ansi_term | active声明 | ansi_term.case.ad | ansi_term.case.ad |
| B018 | tests::cookbook_vm_tests::cb_compression_tar_compress | compression/001_tar_compress | active声明 | tar_compress.case.ad | tar_compress.case.ad |
| B019 | tests::cookbook_vm_tests::cb_compression_tar_decompress | compression/002_tar_decompress | active声明 | tar_decompress.case.ad | tar_decompress.case.ad |
| B020 | tests::cookbook_vm_tests::cb_compression_tar_strip_prefix | compression/003_tar_strip_prefix | ignored | tar_strip_prefix.case.ad | tar_strip_prefix.case.ad |
| B021 | tests::cookbook_vm_tests::cb_concurrency_rayon_any_all | concurrency/001_rayon_any_all | active声明 | rayon_any_all.case.ad | rayon_any_all.case.ad |
| B022 | tests::cookbook_vm_tests::cb_concurrency_rayon_map_reduce | concurrency/002_rayon_map_reduce | active声明 | rayon_map_reduce.case.ad | rayon_map_reduce.case.ad |
| B023 | tests::cookbook_vm_tests::cb_concurrency_rayon_parallel_sort | concurrency/003_rayon_parallel_sort | active声明 | rayon_parallel_sort.case.ad | rayon_parallel_sort.case.ad |
| B024 | tests::cookbook_vm_tests::cb_concurrency_crossbeam_spsc | concurrency/004_crossbeam_spsc | active声明 | crossbeam_spsc.case.ad | crossbeam_spsc.case.ad |
| B025 | tests::cookbook_vm_tests::cb_concurrency_rayon_iter_mut | concurrency/005_rayon_iter_mut | active声明 | rayon_iter_mut.case.ad | rayon_iter_mut.case.ad |
| B026 | tests::cookbook_vm_tests::cb_concurrency_rayon_parallel_search | concurrency/006_rayon_parallel_search | active声明 | rayon_parallel_search.case.ad | rayon_parallel_search.case.ad |
| B027 | tests::cookbook_vm_tests::cb_concurrency_crossbeam_complex | concurrency/007_crossbeam_complex | active声明 | crossbeam_complex.case.ad | crossbeam_complex.case.ad |
| B028 | tests::cookbook_vm_tests::cb_concurrency_crossbeam_spawn | concurrency/008_crossbeam_spawn | active声明 | crossbeam_spawn.case.ad | crossbeam_spawn.case.ad |
| B029 | tests::cookbook_vm_tests::cb_concurrency_global_mut_state | concurrency/009_global_mut_state | active声明 | global_mut_state.case.ad | global_mut_state.case.ad |
| B030 | tests::cookbook_vm_tests::cb_concurrency_threadpool_walk | concurrency/010_threadpool_walk | active声明 | threadpool_walk.case.ad | threadpool_walk.case.ad |
| B031 | tests::cookbook_vm_tests::cb_cryptography_sha_digest | cryptography/001_sha_digest | active声明 | sha_digest.case.ad | sha_digest.case.ad |
| B032 | tests::cookbook_vm_tests::cb_cryptography_pbkdf2 | cryptography/002_pbkdf2 | active声明 | pbkdf2.case.ad | pbkdf2.case.ad |
| B033 | tests::cookbook_vm_tests::cb_cryptography_hmac | cryptography/003_hmac | active声明 | hmac.case.ad | hmac.case.ad |
| B034 | tests::cookbook_vm_tests::cb_database_postgres | database/postgres | ignored | 001_create_tables.case.ad, 002_insert_query.case.ad（无doc，旧实选待核，不得擅启用）, 003_aggregate.case.ad（无doc，旧实选待核，不得擅启用） | T-01核旧实选，禁止按排序换源 |
| B035 | tests::cookbook_vm_tests::cb_database_sqlite | database/sqlite | ignored | 001_init.case.ad, 002_insert_select.case.ad（无doc，旧实选待核，不得擅启用）, 003_transactions.case.ad（无doc，旧实选待核，不得擅启用） | T-01核旧实选，禁止按排序换源 |
| B036 | tests::cookbook_vm_tests::cb_datetime_elapsed_time | datetime/001_elapsed_time | ignored | elapsed_time.case.ad | elapsed_time.case.ad |
| B037 | tests::cookbook_vm_tests::cb_datetime_checked | datetime/002_checked | active声明 | checked.case.ad | checked.case.ad |
| B038 | tests::cookbook_vm_tests::cb_datetime_timezone | datetime/003_timezone | ignored | timezone.case.ad | timezone.case.ad |
| B039 | tests::cookbook_vm_tests::cb_datetime_current | datetime/004_current | ignored | current.case.ad | current.case.ad |
| B040 | tests::cookbook_vm_tests::cb_datetime_format | datetime/005_format | ignored | format.case.ad | format.case.ad |
| B041 | tests::cookbook_vm_tests::cb_datetime_parse_string | datetime/006_parse_string | active声明 | parse_string.case.ad | parse_string.case.ad |
| B042 | tests::cookbook_vm_tests::cb_datetime_timestamp | datetime/007_timestamp | ignored | timestamp.case.ad | timestamp.case.ad |
| B043 | tests::cookbook_vm_tests::cb_devtools_log_debug | devtools/001_log_debug | active声明 | log_debug.case.ad | log_debug.case.ad |
| B044 | tests::cookbook_vm_tests::cb_devtools_log_error | devtools/002_log_error | active声明 | log_error.case.ad | log_error.case.ad |
| B045 | tests::cookbook_vm_tests::cb_devtools_log_stdout | devtools/003_log_stdout | active声明 | log_stdout.case.ad | log_stdout.case.ad |
| B046 | tests::cookbook_vm_tests::cb_devtools_log_custom | devtools/004_log_custom | active声明 | log_custom.case.ad | log_custom.case.ad |
| B047 | tests::cookbook_vm_tests::cb_devtools_log_syslog | devtools/005_log_syslog | active声明 | log_syslog.case.ad | log_syslog.case.ad |
| B048 | tests::cookbook_vm_tests::cb_devtools_log_env | devtools/006_log_env | active声明 | log_env.case.ad | log_env.case.ad |
| B049 | tests::cookbook_vm_tests::cb_devtools_log_mod | devtools/007_log_mod | active声明 | log_mod.case.ad | log_mod.case.ad |
| B050 | tests::cookbook_vm_tests::cb_devtools_log_timestamp | devtools/008_log_timestamp | active声明 | log_timestamp.case.ad | log_timestamp.case.ad |
| B051 | tests::cookbook_vm_tests::cb_devtools_log_custom_location | devtools/009_log_custom_location | active声明 | log_custom_location.case.ad | log_custom_location.case.ad |
| B052 | tests::cookbook_vm_tests::cb_devtools_tracing_console | devtools/010_tracing_console | active声明 | tracing_console.case.ad | tracing_console.case.ad |
| B053 | tests::cookbook_vm_tests::cb_encoding_json | encoding/001_json | active声明 | json.case.ad | json.case.ad |
| B054 | tests::cookbook_vm_tests::cb_encoding_toml | encoding/002_toml | active声明 | toml.case.ad | toml.case.ad |
| B055 | tests::cookbook_vm_tests::cb_encoding_csv_read | encoding/003_csv_read | active声明 | csv_read.case.ad | csv_read.case.ad |
| B056 | tests::cookbook_vm_tests::cb_encoding_base64 | encoding/004_base64 | active声明 | base64.case.ad | base64.case.ad |
| B057 | tests::cookbook_vm_tests::cb_encoding_hex | encoding/005_hex | active声明 | hex.case.ad | hex.case.ad |
| B058 | tests::cookbook_vm_tests::cb_encoding_endian_byte | encoding/006_endian_byte | active声明 | endian_byte.case.ad | endian_byte.case.ad |
| B059 | tests::cookbook_vm_tests::cb_encoding_csv_delimiter | encoding/007_csv_delimiter | active声明 | csv_delimiter.case.ad | csv_delimiter.case.ad |
| B060 | tests::cookbook_vm_tests::cb_encoding_csv_filter | encoding/008_csv_filter | active声明 | csv_filter.case.ad | csv_filter.case.ad |
| B061 | tests::cookbook_vm_tests::cb_encoding_csv_invalid | encoding/009_csv_invalid | active声明 | csv_invalid.case.ad | csv_invalid.case.ad |
| B062 | tests::cookbook_vm_tests::cb_encoding_csv_serde_serialize | encoding/010_csv_serde_serialize | active声明 | csv_serde_serialize.case.ad | csv_serde_serialize.case.ad |
| B063 | tests::cookbook_vm_tests::cb_encoding_csv_serialize | encoding/011_csv_serialize | active声明 | csv_serialize.case.ad | csv_serialize.case.ad |
| B064 | tests::cookbook_vm_tests::cb_encoding_csv_transform | encoding/012_csv_transform | active声明 | csv_transform.case.ad | csv_transform.case.ad |
| B065 | tests::cookbook_vm_tests::cb_encoding_percent_encode | encoding/013_percent_encode | active声明 | percent_encode.case.ad | percent_encode.case.ad |
| B066 | tests::cookbook_vm_tests::cb_encoding_url_encode | encoding/014_url_encode | active声明 | url_encode.case.ad | url_encode.case.ad |
| B067 | tests::cookbook_vm_tests::cb_errors_boxed_error | errors/001_boxed_error | active声明 | boxed_error.case.ad | boxed_error.case.ad |
| B068 | tests::cookbook_vm_tests::cb_errors_anyhow | errors/002_anyhow | active声明 | anyhow.case.ad | anyhow.case.ad |
| B069 | tests::cookbook_vm_tests::cb_errors_backtrace | errors/003_backtrace | active声明 | backtrace.case.ad | backtrace.case.ad |
| B070 | tests::cookbook_vm_tests::cb_errors_retain | errors/004_retain | active声明 | retain.case.ad | retain.case.ad |
| B071 | tests::cookbook_vm_tests::cb_file_read_lines | file/001_read_lines | active声明 | read_lines.case.ad | read_lines.case.ad |
| B072 | tests::cookbook_vm_tests::cb_file_find_files | file/002_find_files | ignored | find_files.case.ad | find_files.case.ad |
| B073 | tests::cookbook_vm_tests::cb_file_recursive_size | file/003_recursive_size | ignored | recursive_size.case.ad | recursive_size.case.ad |
| B074 | tests::cookbook_vm_tests::cb_file_modified | file/004_modified | active声明 | modified.case.ad | modified.case.ad |
| B075 | tests::cookbook_vm_tests::cb_file_duplicate_name | file/005_duplicate_name | ignored | duplicate_name.case.ad | duplicate_name.case.ad |
| B076 | tests::cookbook_vm_tests::cb_file_find_file | file/006_find_file | ignored | find_file.case.ad | find_file.case.ad |
| B077 | tests::cookbook_vm_tests::cb_file_ignore_case | file/007_ignore_case | ignored | ignore_case.case.ad | ignore_case.case.ad |
| B078 | tests::cookbook_vm_tests::cb_file_loops | file/008_loops | ignored | loops.case.ad | loops.case.ad |
| B079 | tests::cookbook_vm_tests::cb_file_png | file/009_png | active声明 | png.case.ad | png.case.ad |
| B080 | tests::cookbook_vm_tests::cb_file_recursive | file/010_recursive | ignored | recursive.case.ad | recursive.case.ad |
| B081 | tests::cookbook_vm_tests::cb_file_sizes | file/011_sizes | ignored | sizes.case.ad | sizes.case.ad |
| B082 | tests::cookbook_vm_tests::cb_file_skip_dot | file/012_skip_dot | ignored | skip_dot.case.ad | skip_dot.case.ad |
| B083 | tests::cookbook_vm_tests::cb_file_same_file | file/013_same_file | ignored | same_file.case.ad | same_file.case.ad |
| B084 | tests::cookbook_vm_tests::cb_file_read_lines_temp | file/014_read_lines_temp | active声明 | read_lines_temp.case.ad | read_lines_temp.case.ad |
| B085 | tests::cookbook_vm_tests::cb_hardware_cpu_count | hardware/001_cpu_count | active声明 | cpu_count.case.ad | cpu_count.case.ad |
| B086 | tests::cookbook_vm_tests::cb_mem_lazy_cell | mem/001_lazy_cell | active声明 | lazy_cell.case.ad | lazy_cell.case.ad |
| B087 | tests::cookbook_vm_tests::cb_os_env_variable | os/001_env_variable | active声明 | env_variable.case.ad | env_variable.case.ad |
| B088 | tests::cookbook_vm_tests::cb_os_process_continuous | os/002_process_continuous | active声明 | process_continuous.case.ad | process_continuous.case.ad |
| B089 | tests::cookbook_vm_tests::cb_os_error_file | os/003_error_file | Windows ignored（cfg_attr） | error_file.case.ad | error_file.case.ad |
| B090 | tests::cookbook_vm_tests::cb_os_piped | os/004_piped | ignored | piped.case.ad | piped.case.ad |
| B091 | tests::cookbook_vm_tests::cb_os_process_output | os/005_process_output | ignored | process_output.case.ad | process_output.case.ad |
| B092 | tests::cookbook_vm_tests::cb_os_send_input | os/006_send_input | ignored | send_input.case.ad | send_input.case.ad |
| B093 | tests::cookbook_vm_tests::cb_safety_heapless | safety/001_heapless | active声明 | heapless.case.ad | heapless.case.ad |
| B094 | tests::cookbook_vm_tests::cb_text_regex_replace | text/001_regex_replace | active声明 | regex_replace.case.ad | regex_replace.case.ad |
| B095 | tests::cookbook_vm_tests::cb_text_regex_email | text/002_regex_email | active声明 | regex_email.case.ad | regex_email.case.ad |
| B096 | tests::cookbook_vm_tests::cb_text_regex_hashtags | text/003_regex_hashtags | active声明 | regex_hashtags.case.ad | regex_hashtags.case.ad |
| B097 | tests::cookbook_vm_tests::cb_text_graphemes | text/004_graphemes | active声明 | graphemes.case.ad | graphemes.case.ad |
| B098 | tests::cookbook_vm_tests::cb_text_filter_log | text/005_filter_log | active声明 | filter_log.case.ad | filter_log.case.ad |
| B099 | tests::cookbook_vm_tests::cb_text_phone | text/006_phone | active声明 | phone.case.ad | phone.case.ad |
| B100 | tests::cookbook_vm_tests::cb_text_from_str | text/007_from_str | active声明 | from_str.case.ad | from_str.case.ad |
| B101 | tests::cookbook_vm_tests::cb_versioning_semver_parse | versioning/001_semver_parse | active声明 | semver_parse.case.ad | semver_parse.case.ad |
| B102 | tests::cookbook_vm_tests::cb_versioning_semver_increment | versioning/002_semver_increment | active声明 | semver_increment.case.ad | semver_increment.case.ad |
| B103 | tests::cookbook_vm_tests::cb_versioning_semver_latest | versioning/003_semver_latest | active声明 | semver_latest.case.ad | semver_latest.case.ad |
| B104 | tests::cookbook_vm_tests::cb_versioning_semver_command | versioning/004_semver_command | active声明 | semver_command.case.ad | semver_command.case.ad |
| B105 | tests::cookbook_vm_tests::cb_versioning_semver_complex | versioning/005_semver_complex | active声明 | semver_complex.case.ad | semver_complex.case.ad |
| B106 | tests::cookbook_vm_tests::cb_versioning_semver_prerelease | versioning/006_semver_prerelease | active声明 | semver_prerelease.case.ad | semver_prerelease.case.ad |
| B107 | tests::cookbook_vm_tests::cb_web_mime | web/mime | active声明 | 001_request.case.ad | 001_request.case.ad |
| B108 | tests::cookbook_vm_tests::cb_web_scraping | web/scraping | ignored | 001_extract_links.case.ad, 002_unique.case.ad（无doc，旧实选待核，不得擅启用）, 003_broken.case.ad（无doc，旧实选待核，不得擅启用） | T-01核旧实选，禁止按排序换源 |

## 6. 测试设计

全部是未来work/review/merge合同，本轮无产品运行。没有单独子agent/独立chat授权，不自动派生；review以独立上下文/证据重建为准，未独立完成不冒PASS。

### 6.1 正例与全路径oracle

| V / owner_stage | 真实输入、路径/状态 | 独立可观察关闭条件 | 任务/AC |
|---|---|---|---|
| V-01/work | S11+N1050/P2工程/编译250/C30&33、E78、VM cache506与cookbook VM108；当前组重发现 | 新旧完整集合、排序/条件/源/oracle/空H/消费身份对账，1161现存+9空槽=1170角色、两遗漏result补齐，实际原始结果而非父exit | T-01/09；AC-01 |
| V-02/work | 每N family先hello、双stdout/result、.as、error、99_bootstrap、C/H；五root全项 | discover→doc→原entry→原backend→原matcher→report；doc-only同集合/实际值，失败原子名/通道保 | T-02/03/08；AC-02 |
| V-03/work | test/runners四文件五函数及四公开Test.run_*_dir | 真实VM shim调用；选合法小镜像、错oracle/缺doc/空集，各子项计数/命名/失败定位准确；专项原1050逐项新旧实际核验 | T-03；AC-02 |
| V-04/work | S01～11原suite vs候选/正式suite | v2原source/options/expect无损；成功8输出/空stderr/exit，负3原stage/code；正常入口确读doc | T-04；AC-03 |
| V-05/work | P两工程doc-only及原16+3断言 | 原transpile_rust_project实际产生全部文件，19条独立判据逐条执行；改一个导入源实际输出变化 | T-05；AC-04 |
| V-06/work | 250编译原义务、ledger各结果类、M01～03的6份doc产物；准备/验证分开 | 原真实rustc逐项exit/stderr/产物/5类归账及M表原typecheck/4 witness/独立参考stdout判据；缺工具/旧receipt不绿，正常组无后代rustc/cargo | T-06；AC-05 |
| V-07/work | 30生成候选、33固定验证、普通注册/写入保护及wrong/GENERATE_EXPECTED旁路 | 正常/故意失败都无作者新文件或覆写；proposal只在scope且子失败非零；改独立expected仍红；33原trim原态 | T-07；AC-06 |
| V-08/work | 当前全组/所有file-reader、trace/inventory/旧floor | 零活跃旧载荷/资格回读，doc删改真实影响，新增组被扫描；history仅显式，source-dir明确闭包 | T-08/09；AC-07 |
| V-09/work | 每新增控制/迁移case、完整职责组和并集 | 每新增case≤10秒；普通ACE模块60目标/120上限；准备/执行/清理/等待分列、debug、无日常lazy build/release/WSL | T-10/11；AC-08 |
| V-10/review | 新鲜diff/AC/全部集合/actual/控制及SD | 覆盖路径重建，不信checked/生成器；冻结提交/工具/配置相符，无隐蔽旧入口 | T-12；AC-09 |
| V-11/work | VM cache506/8脚本及cookbook VM108绑定/6多源、附录E78边界 | 源/资格/模式/原Windows77/31条件全部对账，doc-only脚本仍lower、cookbook不扫旧.at、未实选候选不误升；失败/生成旁路零作者写入 | T-03/07/08/09；AC-01/02/06/07 |

N逐项新旧运行保原调用适用范围；原生目录调用本来与Rust ignored注册不同，不因迁移擅按另一个消费者的ignore删原义务，也不因此把1050全添进日常。环境/原产品红按固定旧基面并列；缺真实结果属于未验证，不充PASS。全运行以已有消费者/独立专项批次报逐项actual，新增控制各自小输入，不把1050打包成一个≤10秒新增case。

### 6.2 原27个具名负控（完整范围；新增A四控见后，owner_stage=work）

每项需真实命中+合法邻例+≤10秒实际时长；缺计时或10.001秒在review判needs_fix。原1050/250正式义务不可缩样；控制使用同机制小输入，不能用无语义脚本检查代替实际consumer。变异只在scope镜像，恢复后再读/再运行证明可用。

| NC | 变异/入口 | 必须看到的失败与合法邻例 | AC |
|---|---|---|---|
| NC-01 | 删N末项/复用不同family相同case ID合并/漏新第76组 | 集合差具名拒绝，不接受全绿子集；恢复精确全部集合 | AC-01/07 |
| NC-02 | 每族独立小镜像移除旧.at/.as/expected，再经discover和原entry；脚本/bootstrap另具名小probe | 各probe真实结果/资格不变；全1050 doc-only集合/实际证明归V-01/02专项批次，不包装成单个新增控制；额外10个VM smoke不升级；恢复一致 | AC-02 |
| NC-03 | 保旧文本但删/损doc | 明确doc路径/行/原义务错误，不能fallback、跳过或只回更少数量 | AC-02 |
| NC-04 | 污染旧hello.at/expected而不改doc | 四native族及常规包装结果不变；反改doc源/oracle只影响正确消费者 | AC-02/07 |
| NC-05 | VM双通道互换、空串当缺席、error内容变/错误存在性改 | stdout/result逐字节各自红；任意错误存在性保原规则，不增typed要求 | AC-02 |
| NC-06 | .as当.at/忽略99_bootstrap或使用旧正文/错误module名 | 与旧原机制不同必须被源/path/实际拒绝轴识别；正确脚本与bootstrap邻例过 | AC-02 |
| NC-07 | C/H或TS/Rust调错backend、cookbook选VM面、mixeddep+另一错误 | 真实选择/产物diff红；C-H同次转译、合法独立目标不多跑后端 | AC-02/05 |
| NC-08 | S01 stdout去尾LF/非空stderr/错误exit；S09 code错 | 具体流/exit/stage/code mismatch命中，原11项邻例保持 | AC-03 |
| NC-09 | scheduler新doc有但suite仍include旧文件；新增一个旧格式组 | 最终入口扫描红并定位group/suite/case；切正确include及合法doc组过 | AC-03/07 |
| NC-10 | P005只物化main、仍读旧db/api；修改doc db正文 | 缺闭包失败；旧模块污染不影响，doc模块改变实际输出且正确断言红 | AC-04 |
| NC-11 | P中漏1个contains/exists、产物路径错/大小写错 | 19断言集合缺项/具体产物匹配红；仅文本无关差异不升级bytes断言 | AC-04 |
| NC-12 | 绝对/..文件、重复模块路径、scope外物化/reparse | 写前具名拒绝、作者树hash不变；合法嵌套api/mod正常 | AC-04/06 |
| NC-13 | 编译门只保249项/假PASS/stale源或rustc/配置/ledger | 精确子项/身份不匹配，验证不启动编译；新鲜250真实收据通过 | AC-05 |
| NC-14 | external-dep同时夹E0308，或ledger项由红变绿 | mixed错误不当skip，obsolete-ledger必须红；纯E0432等保原skip | AC-05 |
| NC-15 | 普通cargo t a2r/corpus触发rustc/cargo/release，或缺准备auto-build | 进程/准备守卫红、review needs_fix；显式debug准备/只读验证合法 | AC-05/08 |
| NC-16 | proposal无out/out指向repo或旧expected、路径别名 | VM执行前拒绝；合法scope出候选且全部作者hash不变 | AC-06 |
| NC-17 | 固定30注册表删改/把040加入闭集，及单例VM失败/写候选失败小probe | 闭集差/子失败和总非零；全部30候选实跑归V-07专项，不把30运行包装为新增单例控制，不虚报验证PASS | AC-06 |
| NC-18 | 普通bootstrap仍注册可写、独立conformance doc expected改错 | 普通注册/写入守卫红；33验证中的对应项实际失败，生成结果不能自证金样 | AC-06 |
| NC-19 | inventory/trace重读旧sidecar自证、隐藏旧source fallback | doc-only/污染控制及receipt身份复验红；保历史显式回放邻例 | AC-07 |
| NC-20 | 新控制耗时10.001秒、缺计时、父exit0漏子失败或NOT_RUN冒绿 | 汇报/审查拒绝并具名，模拟治理控制自身≤10秒；合法原子/排除分列 | AC-08/09 |
| NC-21 | N0028/N0030只错doc result为sucC/11，stdout仍原值；另删result节 | 实际原生入口及VM缓存包装在result轴红/缺通道守恒红；原succ/10无尾LF且stdout同次采集时通过，禁止同源跑两遍冒单次 | AC-01/02 |
| NC-22 | 使用同机制小C夹具，C正文golden正确、实际header非空但doc声明原缺H/空H语义；另改空为缺席跳过 | 必在header轴红，不能因没有H文件而跳过；有正确H的hello和N0792原空H邻例通过 | AC-02 |
| NC-23 | 将附录E中任一其他消费者doc或未来P项目doc误加N；反删一个N doc使数量回1050 | 独立集合/资格键差而非只数1050拒绝；全部78边界元数据扫描可做一小控制，真实运行用有界代表，合法N与其他原消费者各归原域 | AC-01/07 |
| NC-24 | N0229 doc-only删除.as并放诱饵.at，或cookbook六个多源目录打乱旧源/候选doc | VM缓存模式仍由doc走原lower，实际路由/前后源hash与旧机制一致，不能仅因结果同值判过；native路径不重复lower。cookbook保T-01实选、不按旧文件/新排序换源；分小probe计时，108/506全集对账归V-11 | AC-02/07 |
| NC-25 | C01错oracle触发.wrong.out、B绑定缺oracle并设GENERATE_EXPECTED=1；预先不存在wrong/expected文件 | 正常及失败入口都不得在作者根创建文件；报告/候选仅scope；完整树清单/写入路径核查，不能仅比原doc hash | AC-06/07 |
| NC-26 | v2正常库存/trace仍打开旧source_path别名，或v2缺字段/失败后回退v1；伪造compiled分类而原exit失败 | doc-only/源文件读取监测、schema和原子raw evidence复判拒绝；显式历史v1查询与真实v2合法邻例分开，历史不充当前PASS | AC-05/07 |
| NC-27 | M01～03的6旧expected删除/污染；doc Rust产物改一字节；M03产品冒充oracle或stub hash漂移 | 旧文件变化无影响，doc改动使旧准备收据失效；真实编译/原witness/独立stdout拒绝错误，非读回旧金样；同fixture/正确doc邻例过，控制分有界小probe≤10秒 | AC-05/07 |

以上NC-21～27是r2补强，每个实现成可单独运行的小probe，真实源码读取/模式/原judge必须命中；全目录/大编译的完整义务属于对应V批次，不用一个新增测试吞掉全量。已有NC保留原ID。

#### 具体控制种子（回指原文件/hash及调用链）

| 种子 | 已有实际输入/消费者 | 对应控制与区分轴 |
|---|---|---|
| Seed-01 | N0028/N0030两VM用例、vm-stdout/vm-result | NC-05/21；stdout正确但返回值错误必须红，缺2个旧result不是legacy smoke |
| Seed-02 | N0229 vm/18_ffi/016_json_is_null，virtual json_is_null.as、syntax auto-script | NC-06/24；doc-only/诱饵.at下仍执行原脚本lower，原result="1"无尾LF |
| Seed-03 | N0331 vm/99_bootstrap/001_token_enum | NC-06；原库前置与token_enum正文不同来源，旧stdout="green\n"；小probe运行预算实测，不复制巨大样本 |
| Seed-04 | N0022 vm/03_variables/006_let_asn_error，vm-error | NC-05；任意错误存在性保，旧error文本不升级substring/typed oracle |
| Seed-05 | N0792及其余8个缺H角色 | NC-07/22；header原空字节仍为判据，109+9个H角色均保 |
| Seed-06 | N0747 a2c/01_basics/001_hello、N0497 a2r/01_basics/001_hello、N0865 a2ts/01_basics/001_hello | NC-04/07；不同source/产物与module stem不能串用；编译门N0497另保完整a2r/...参数 |
| Seed-07 | N0950 cookbook/algorithms/001_sort_int、B001 cb_algorithms_sort_int | NC-02/07；同doc cookbook-stdout与rust两consumer不互换 |
| Seed-08 | 附录B的cb_asynchronous_channel/fs/rt、cb_database_postgres/sqlite、cb_web_scraping | NC-24/25；6个多源选择和生成旁路；ignore/网络按原条件，不借真实网络做新控制 |
| Seed-09 | S01 message-handler、S09 neg-unknown-task、S10 malformed、S11 send-after-stop | NC-08/09；分别核stdout与resolve/parse/run三诊断，不混通道 |
| Seed-10 | P两工程及P01～P16/D01～D03 | NC-10～12；5源闭包、19原期望逐项可删反控；不能以未检查的旧golden加严替代 |
| Seed-11 | C01/001_int_add和C31/040_while_basic | NC-16～18/25；前者在30生成闭集，后者仅独立验证，二者均保Unicode trim |

### 6.3 分组、命令和运行成本

未来在专属树先查同机其他agent/工作树实际编译/测试进程和调度锁，占用则等待并告知用户当前模块/阶段；起跑前复查，不并发构建。显式source-bound debug/test准备runner/worker/必要runtime，不在日常case中触发构建或release，不跑WSL。

现有完整职责门：cargo t auto-test-case infra-fixed；N/共享entry加 cargo t vm a2r a2c a2ts corpus（T-01核真实组存在/选择，unknown触面具名扩大）；S加 cargo t case-ace-m2-scheduler-968 ace-adapters ace-av。inventory/host桥接变动按真实触面补相关完整组，避免只测author；复用未变身份的有效证据，修一个NC不重跑全部ACE。最终并集由V表/实际diff冻结，在merge前一次联合调度，不每阶段附全仓。周期cargo t all仅due时主检出单实例。

N的四公开入口验收用test/runners真实方法与受管小镜像，先跑具名命令/最小反例，再在显式专项核完整1050逐项；不机械新增巨大日常wrapper。P新a2r-compile组/显式prepare和C proposal命令均待T-02/06/07实现，禁止把本文命令记成已运行证据。保编译250全量专项及known-broken判决，不把新小控制移perf豁免10秒。

| 关闭入口（未来在专属树，工具先显式准备） | 具体输入/命令或已核实API | 必须取得的证据 |
|---|---|---|
| S声明/运行 | auto-case.exe lint / migration-inventory --schema-version 2 / run，--suite crates/auto-lang/test/cases/ace-m2/scheduler/scheduler.suite.ad；run显式--out指向scope/S/live | 11项逐键原expect与actual、正常suite的origin.format；故意错oracle拒绝 |
| N公开路径 | entry::test_file逐个调用test/runners/{vm_tests,a2r_tests,a2c_tests,a2ts_tests}.at（已核实API，非假设某CLI参数）；小镜像中的五个Test.run_*_dir调用用scope路径 | 原五个测试函数/四shim真实执行；每个child实际结果与失败数，不能仅信父assert或打印 |
| P工程定向 | cargo t a2r --test-name tests::a2r_tests::test_14_modules_005_multi_file；另单跑test_16_interop_021_path_dep同namespace | 实际5产物/16期望与Cargo.toml 3期望逐项记录；doc-only闭包/污染负控 |
| C独立验证 | cargo t corpus --test-name tests::conformance_tests::conformance_001_int_add；其余33原test按附录C逐名选择/完整corpus门 | 注册33保，bootstrap普通注册消失；改文档oracle必红；作者树运行前后hash不变 |
| P专项/C候选 | T-02冻结的显式准备→cargo t a2r-compile --ignored --test-name tests::a2r_tests::a2r_rustc_real_compile_gate；单一conformance-proposal入口（§5.4） | 前者250实际分类+收据验证不编译、旧test必须真实选中；后者30逐候选/错误、输出仅scope、生成不充PASS |

N现有FileTestReport/父打印不足以独立重读所有actual时，由T-03/08补受管实际载荷观测：VM值/双流/错误，a2r/TS真实产物，C/H同次输出及原判断字段。存路径/bytes/hash/原子结果，离线再次读取。不能仅给源码/oracle身份加一个自报PASS；观测接线不改变原judge，基线和候选使用等价、可核的采集方式。原失败也须保实际错误轴。

报告逐模块：选择数/实际执行/原条件排除/原始失败/恢复绿，以及准备/冷编译、发现、执行、清理、资源等待、整命令时间。新增S11/工程2与新控制各记内容、原身份映射和时长；编译250既存义务专项时长单列。无计时不判PASS，普通模块60秒目标/120秒上限，性能原预算不减。源/工具/配置变动使受影响phase证据失效，其余不重复。

### A片段负控（拟新增，全部未执行）

| NC / 原work owner | 输入与区分性关闭 |
|---|---|
| NC-A1 / T-01/02/10 | 从被冻结的原义务映射删N末项、合并跨family同ID、把九个空H变缺席或漏一原断言，A分母检查拒；恢复原集合/条件后绿，不能只数1050。 |
| NC-A2 / T-03/10 | N0028/N0030保持stdout正确、只将原result改sucC/11或删check，A真实文档/ACE消费者在result轴红，独立旧succ/10无尾LF控制绿；A未切doc的旧consumer只可保原输入证据，不能冒doc变异已命中。 |
| NC-A3 / T-07及03/08/10 | A实际普通入口触发错误/生成环境开关，作者根应零新增/覆盖；若仍写wrong/expected则红，候选/日志只scope；错误oracle仍红，关闭写入不能顺便关闭原33验证。 |
| NC-A4 / T-08/09/10 | 当前A实际选用source/oracle/consumer与receipt不一致、仅旧文件结果冒doc-only、父exit0漏子失败/缺工具、漏stdout-result一通道须拒签A；明确版本/真实actual及独立judge合法控制可过。 |

原27个NC的完整含义保留，B最终逐项运行；A四个新增控制只证明上文A子面，不冒NC-01..27已通过。新case/target由相应原任务在原TestCase/固定组接线，不建另一个runner。原input规模/手动ignored/平台/计时预算保留；A不做release/WSL或正式压力，不因完整迁移后移把正确性测试挪perf。

## 7. 验收标准

| 状态 | AC | owner_stage | 可观察关闭条件 | 证据 |
|---|---|---|---|---|
| [ ] | AC-01 | work | S11、**N1060**（1050 冻结项 + B 期登记的 10 个 doc-only VM 项）/**1171 角色**（1161 现存文件 oracle + 9 空 H + 新增 1 个 `017_sub_semantics` stdout 角色）、P两工程/19断言+编译250、C30/33、**E78=70**（68 存量边界 + 2 份 project 文档）、cache506/B108及当前新增消费者逐键完整；不混分母 | V-01/11，附录，NC-01/21/23 |
| [ ] | AC-02 | work | 四方法五根及相邻cache/cookbook选择模式均读doc，两原result补齐、C空H保；源/资格/原身份/module参数/路径/比较器/条件守恒，缺doc硬错，doc-only/污染通过 | V-02/03/11，NC-02～07/21/22/24 |
| [ ] | AC-03 | work | S11按原实际值与完整v2无损，正常suite全doc，其他已迁身份保持 | V-04，NC-08/09 |
| [ ] | AC-04 | work | 两工程5源/19期望均在doc并真实消费；原项目转译器、受管完整闭包、合法路径/目录语义守恒 | V-05，NC-10～12 |
| [ ] | AC-05 | work | 全编译250与ledger分类、M三个手动guard/6份doc产物和独立参考判决真实保留；日常无隐式编译，显式准备/精确ignored验证缺证不绿 | V-06，NC-07/13～15/26/27 |
| [ ] | AC-06 | work | 30生成仅显式proposal安全scope，33独立验证保；普通bootstrap及旧生成旁路退出，成功/失败/GENERATE_EXPECTED下均零作者新增/覆写，不以生成充PASS | V-07/11，NC-12/16～18/25 |
| [ ] | AC-07 | work | 五根（vm/a2r/a2c/a2ts/cookbook）全入口重发现零**未具名**的活跃旧载荷/资格回读（常驻分类审计：文件级 allowlist + ±2 行读窗口；相邻 a2gd/a2j/tscn 与 ffi_dep/AAVM/外仓 book 逐族具名，不宣称集合永不增长），E78不误启用；正常inventory/trace全部v2读doc、显式历史v1不作fallback；历史/外仓/support具名，1062更正与现状一致 | V-08/11，NC-01/04/09/19/23/24/26 |
| [ ] | AC-08 | work | 每新增case≤10秒、完整普通模块预算/模块及新例内容时长实报、debug前置/资源等待有效、无release/WSL | V-09，NC-15/20 |
| [ ] | AC-09 | review | 独立逐AC/原子actual/所有NC与合法邻例及SD，冻结代码/工具/输入；无遗漏/隐藏旧入口/未授权延期 | V-10 |
| [ ] | AC-10 | merge | 复核冻结影响门/必要due、SD落地，保历史更正与新交付边界；归档/全输出路径对账与guard清树/scope | T-13 |

| 状态 | AC | owner_stage | 可观察关闭条件 | 证据 |
|---|---|---|---|---|
| [ ] | AC-A-CORRECTNESS | work | A-DEN/ORACLE/NOWRITE/EVIDENCE与NC-A1..4真关闭并有独立phase复核/正式来源，所有B余量逐任务保原oracle/条件；不能把两个缺result或写作者树放B | 原T-01/02/03/07/08/09/10/11 A部分；T-12[A]/T-13[A]交IF-1066-A-CORRECTNESS |

## 8. 执行步骤

一个计划13任务，全部未执行。下一步只能在用户后续实施指令后开始T-01；草案不授权本轮实施。T-02的共享接口/资格、编译准备及proposal入口有界决策完成后，下游才实施，不另拆四个Plan。

| 状态 | T / owner_stage | 前置 | 实际交付与关闭证据 |
|---|---|---|---|
| [ ] | T-01/work | A：后续实施指令/原集合；B：源漂移复核 | 建唯一树/scope；同源重冻S/N/P/C/E78、cache506/8脚本、B108/6多源实选，旧Git/实际发现/注册/构造消费者和1170原角色/ledger/依赖，1050逐项完整hash；记录增量，不调用隐式prepare作只读发现；V-01/AC-01  本轮阶段：A冻结基线＋B漂移复核；片段完成不勾整项。 |
| [ ] | T-02/work | A：T-01[A]；B：T-01完整冻结 | 冻结共享doc视图、五根资格/E78/缓存与cookbook绑定、public structs兼容/原身份module参数、source-dir/path与actual；落实§5.2 v2及§5.3精确ignored编译接线/唯一proposal共享接口；S/P能力审计和各族小例，不闭门不做依赖项；AC-02/04/05/06  本轮阶段：A正确性子面；完整迁移接口B；片段完成不勾整项。 |
| [x] | T-03/work | A：T-02[A]；B：T-02[B]所用接口 | test_runner+entry四族+sys_env/runners切doc；补N0028/N0030原result、九项空H判据；VM缓存is_script/载荷、cookbook108选择与诊断输出切doc/scope，原名字/module/失败数/排序/条件保；运行局部NC-02～07/21～24。此任务关运行接线，治理v2和最终V-11由T-08/09/10完成，不提前认领全局门；AC-02  **[✅ 2026-11-17]** 四族发现/执行均以 `<stem>.case.ad` 为唯一作者源（72d7c2bd3）；VM cache `is_script` 由文档路径判定、cookbook108 选唯一文档且 oracle=`cookbook-stdout`、`.wrong.*` 全部转 `diag_scope`；载荷等价证明 1060 源 + 663 artifact + 556 check oracle 与冻结 sidecar 逐字节相同（4 个 `a2r/22_actors/*.expected.out` 属 a2r-actor-tests 外消费者，非 N oracle）；逐例 actual：a2r 351 项 1 红、a2c 118 项 8 红、a2ts 85 项 85 红（§D-1066-IGNORED-CORPUS-DRIFT / A2TS-GOLDEN，均被 `#[ignore]` 掩盖的既有漂移，同输入同 oracle 旧路径同样红）；NC-02..07/21/22/23/24 常驻控 9+2 测 1.16s（镜像 doc-only、删文档硬错、sidecar 污染无关、双通道 result 轴、bootstrap 规则/正文、空H 角色 vs header 轴、族 artifact 词汇不串、E78 键集边界、`.as` 诱饵/cookbook 001_* 选择）；`cargo t vm corpus` PASS 1137/0、child_validation valid；runner 全量重放遇 CWD 敏感挂起→§D-1066-SHIM-CWD-HANG（以镜像/路由等价 + wrapper 全量绿取证，不冒 shim 全量 PASS）|
| [x] | T-04/work | B：T-02[B]（不作A前置） | scheduler recipe author准入/双流诊断规则、11源/expect内联；v2/actual/控制后切suite唯一include，保持ID和75组旧身份；NC-08/09；AC-03  **[✅ 2026-11-17]** 载体 `scheduler.document.case.ad`（11例，源/期望逐字节由冻结件生成）+ suite 唯一include切换（aba42ca3a）；真门 `cargo t case-ace-m2-scheduler-968` 11/11 PASS、旧fence镜像↔新doc逐行等价（id/check/status/detail+source/oracle内容哈希11/11同，仅suite/case_file/contract结构轴按已记录口径漂移）；NC-08 常驻（`plan1066-s-doc-controls` 组，case-parent+av 显式工具，6控5.1s，单轴具名红+clean镜像11/11绿）、NC-09 常驻（`scripts/test-governance/document_case_active_scan.py` + `infra-fixed` 2测，真树76组/183文件全document、0 fence/0 unknown，陈旧fence/新增旧组/投毒各具名定位）；另记 master 既有红：`infra-fixed::test_ace_set_is_explicit_and_separate_from_daily`（PLAN-969 278b91204 加 `ace-facade` 未更新断言，基面 7c131286d 即红，非本任务）|
| [x] | T-05/work | B：T-02[B]（不作A前置） | 两份main.project.case.ad内联5源，19期望有限exists/contains+源闭包/受管物化；两旧Rust test转doc消费者，不依赖旧工程文件；NC-10～12；AC-04  **[✅ 2026-11-17]** 作者格式新增产物断言 `exists: true` / `contains: [...]`（与全文围栏互斥；未知键/空集/重复项/`exists: false` 具名拒；Exists 与 Contains 分型，避免凭空多出 exists 断言）（477bc1fed）；两份 `main.project.case.ad`（5 源按 §5.3 冻结哈希校验后逐字节内联，entry 走直接围栏、模块文件走 `#### <path>`；19 条 P01..P16/D01..D03 逐条声明）；两旧 Rust test 改为 doc 消费者（6c543e44f）：受管物化到 `<temp>/<工程basename>` 后调用原 `transpile_rust_project`，逐条断言单独判决并具名失败；绝对/`..`/重复路径写前拒绝；冻结 `*.expected.rs` 只作历史、从不读取。NC-10a/b/c（doc-only 镜像可建、缺闭包硬失败、改文档正文即命中 `db.rs: struct Connection`）、NC-11（错 contains 具名、声明但未产物具名）、NC-12（`../`/绝对/重复源路径解码期拒绝）常驻 1 测 2.11s。A 的 P 分母改读项目文档：19 断言集合逐键不变、E78 68→70（两份 project 文档入边界，不进 N）、n_rows 1060 不变。门：`cargo t a2r` PASS 432/0（139s）、`cargo t vm corpus` PASS 1137/0（79.9s）、`cargo t auto-test-case` PASS 151/0、plan1066 15 passed/3 ignored |
| [x] | T-06/work | B：T-02/03/08所用正式子面；不作T-09[A]前置 | 原250编译子项doc源+M三个手动guard/6份doc Rust产物；显式准备真实rustc/stub+source-bound收据/产物、原始exit/stderr重判及M真实guest/参考判据；全部scope独占输出，a2r-compile/M精确ignored重放、旧身份/ledger/ignore保；NC-13～15/26/27；AC-05  **[✅ 2026-11-17]** 主体 5dbfb6a15；NC-26 治理半边已由 round-7 的 eecef0878/ccd4a3d48 关闭（trace/inventory/association 全部改签文档身份，伪造 compiled 由 T-06 收据重判拒绝） 250 门=`prepare`（文档源 → 原转译器 → 真 `rustc --emit=metadata`）+`verify`（只读收据，按原始 exit/stderr 重算分类、ledger 逐条归账、toolchain/ledger/源 hash 陈旧即拒）两段式，收据 schema `plan1066-a2r-compile-receipt/1` 落 scope；实测 **159 compiled + 55 external-dep-skip + 36 known-broken + 0 obsolete + 0 unexpected = 250**，prepare 22.7s / verify 0.42s；门已 `#[ignore]` 并改由显式组 `a2r-compile`（all=false、tools=[]、domain=legacy、expression 精确该 test）承接，`cargo t a2r` 从 139s→56.3s 且该 test 出现在 excluded(ignored) 列表（NC-15 实证无日常 rustc）。M01/M02/M03 改为读文档 `.rs` 产物（`materialize_doc_product`）并在每次调用独占 scope（`a2r-m01/02/03-<pid>-<nonce>`，退役 `auto_a2r_599_*` 固定目录），实测 0.07s/2.22s/1.12s 全绿，M03 产品与独立 oracle 仍逐字节对比。控：NC-13（伪造 compiled/缺 exit/陈旧 toolchain/ledger/schema 各具名拒）、NC-14（外依赖+他错不 skip、ledger 项变绿=obsolete）、NC-15（`ExplicitCompileGate` 2 测：门必须 ignored 且理由具名、组不进 daily/all）、NC-27（4 份具名 fixture 冻结 SHA-256、6 份 doc 产物与 sidecar 逐字节同、M03 产品≠oracle）常驻 0.16s + 0.013s。**余量**：NC-26 的 v2 库存/trace 半边（a2r/a2c/a2ts 的 `execution_trace::legacy_files` 仍以 sidecar 路径做 `file_identity`）与 T-08 库存切换同轮关闭，故本项暂不勾。A 冻结增量：`p_compile.gate_ignored` false→true（显式组迁移的合法登记）。 |
| [x] | T-07/work | A：T-01/02的写入边界；B：T-02[B]完整proposal接口 | 普通bootstrap退出注册/旧读写；显式conformance-proposal读30 doc、scope候选与错误；33验证及trim保；conformance wrong/cookbook GENERATE_EXPECTED旁路不得写作者根；NC-16～18/25；AC-06  **[✅ 2026-11-17]** 显式 proposal 入口（773d01438）：读冻结 30 例闭集（=A 分母 `c_bootstrap`，逐例一致由常驻控断言）→ 同 AutoVM 机制产出候选 → 只写显式 scope；admission 拒空请求/仓内路径/符号链接别名（祖先 canonical）/非空目录（保前次候选）；逐例报告 + 部分失败即整体失败；实测 **30 written / 0 failed**，作者树 `git status` 零变化，且 30 份候选与已提交文档 oracle 逐字节相同。旁路全清（773d01438+72b449829）：a2r/a2c(两轴)/a2ts/aavm runner/ffi_dual/vm-ast/plan724/plan727 的 `.wrong.*` 全部改 `diag_scope`（默认丢弃），连同 A 已转的 conformance/cookbook/vm 与 entry 四族，普通（含失败）路径不再在作者根创建文件；常驻静态守卫 `plan1066_b_no_author_tree_writes_in_corpus_wrappers` 扫 10 个 wrapper 模块的写调用 + **注入旁路负控**（必须检出），零容忍回潮。NC-16（scope admission 四类拒绝）、NC-17（闭集=冻结分母逐例、040 不得入集、部分=失败）、NC-18（bootstrap 仍 `#[ignore]` 且理由点名 proposal、`generate_expected` 只经 write_scoped）、NC-25（静态守卫+注入负控，动态面由 NC-A3/NC-10 覆盖）全常驻 0.02s 级。门：`cargo t corpus` 240/0（21s）、`cargo t a2r` 432/0（44.6s）、`cargo t vm corpus` 1138/0、plan1066 19 passed/3 ignored。外仓 book 的 `generate_book_expected` 非 ignored 生成器写 `book/` 仓，属外仓 owner（本计划边界明确不转 book），登记 debt。 |
| [x] | T-08/work | A：T-03[A]与实际A证据；B：T-03[B] | §5.2当前v2/历史v1分派、VM helper/CLI、input_inventory/wrapper_bindings/source_bindings/trace身份和报告切doc，旧floor/association标签不作旧IO；重读actual及通用原子记录验真，常规B/cache/cookbook回归。先于T-06供治理，编译分类真实门归T-06；NC-19/26的schema部分；AC-02/07  本轮阶段：A实际证据身份；全v2治理B；片段完成不勾整项。  **[✅ 2026-11-17]** 身份切片 eecef0878+ccd4a3d48（trace/关联 569 行/inventory 行/VM cache 库存与 verifier 全部改签文档；`document_sha256` 与运行时 `source_sha256` 两轴分明；文档为唯一 input edge）；`verify_inputs` 由「无调用者的 sidecar 重算」改为**活跃门**（`source_bindings` 内每轮校验，670f5aa7f），旧 `.expected.*` 回读路径清零；`input-contract.json` 的 `legacy_ids`/`vm_cache_ids`/`baseline` 仍为身份宇宙与标签（不作 IO，未改签）。NC-19（镜像 doc-only/污染无关/删文档可见）与 NC-26（陈旧/伪造收据拒签）常驻。门：`cargo t vm` 950/0、`cargo t vm corpus` 1138/0、`cargo t a2r` 432/0、`cargo t corpus` 240/0、`cargo t auto-test-case` 151/0，均含活跃 verifier |
| [x] | T-09/work | A：T-01/02/03/07/08的A子面；B：T-03～08原完整面 | 当前全组/五根/四shim/cache506/B108/E78/工程/编译/所有生成旁路重发现和读写审计；doc-only/污染镜像、增量纳本Plan，源码/期望/资格/新增wrong均核，不泛豁免source-dir；AC-01/07  本轮阶段：A实际检查与原分母账；全路径doc-only重扫B；片段完成不勾整项。  **[✅ 2026-11-17]** 全路径重扫与读写审计：①活跃入口扫描（`document_case_active_scan.py`，76 组/183 include 文件全 document、0 fence）；②五根集合与 E78 边界=键集非计数（NC-23：N 1060 + E78 70 恰好覆盖五根全部文档，挪一个边界文档必须具名重复）；③四 shim 路由等价（NC-24）与 runner 入口真实调用（shim 全量重放 CWD 挂起记 D-1066-SHIM-CWD-HANG）；④cache506/8 脚本、B108/6 多源、S11/P/C 在 A 冻结与 T-03/T-05 增量后逐键一致；⑤工程/编译/生成旁路：T-05/T-06/T-07 已闭，本轮再关 plan724/plan727 四个 N 条目的 sidecar 读取（改读文档，8/8 + 11/11 绿）；⑥常驻 `plan1066_b_remaining_sidecar_readers_are_named`：扫描全 crates 的 sidecar 读点，**逐个具名理由**（migration 静态库存、NC-27 对拍控、AAVM 实验、外仓 book、五根外 ffi_dep、sidecar 合同合成 fixture、a2r-actor-tests 外消费者），出现未列出者按文件/行拒绝，并带注入负控；⑦增量纳本 Plan：T-03 的 10 个 doc-only VM 项与 T-05 的两份 project 文档（E78 68→70）均已登记。门：corpus 240/0、a2r 432/0、vm corpus 1138/0、plan1066 20 passed/3 ignored |
| [x] | T-10/work | A：NC-A1..4可运行；B：全部原功能与27NC可运行 | V-01～08/V-11与V-09单例计时部分、NC-01～27逐项真实失败+合法邻例/准确子项，补原子wrapper缺证反控/恢复复验，非零exit不代命中；最终完整组/并集计时归后置T-11，不反向等待它；AC-01～08  本轮阶段：A四负控；B全部原27NC与完整验证；片段完成不勾整项。  **[✅ 2026-11-17]** 27 个具名负控齐全且逐项接线（`PLAN1066_CONTROL_OWNERS` 常量即合同，测试断言 27 项唯一无缺）：NC-01=A-DEN/NC-23、02=doc-only 四族镜像、03=删文档硬错、04=污染 sidecar 无关、05=A-NC-A2+native 双通道、06=bootstrap/脚本、07=族 artifact 词汇、08=scheduler 组五轴、09=活跃入口扫描、10/11/12=工程闭包/断言/路径、13/14/26/27=编译收据（伪造 compiled/混错/陈旧/ledger/fixture/M03 oracle）、15=显式组治理、16/17/18=proposal（admission/闭集=bootstrap 分母/退役生成器）、19=库存文档身份、20=控制报告纪律（本轮新增）、21=native entry result 轴、22=空H vs header 轴、24=`.as` 诱饵+多源、25=静态零写入+注入负控。**NC-20 新增**：`validate_control_report` 要求报告恰好 27 行、每行自带 verdict 与实测 ms 且 <10s，拒 NOT_RUN/重复/缺行/超预算，并拒「父 PASS 掩盖失败子」；合法控制 + 六种缺陷各具名拒绝（0.11s）。**实测台账**（nextest libtest-json，21 个在进程内控制）：最大 1909.3ms（A-DEN）、NC-20 自身 114.1ms、其余 27.6–1497ms，全部 ≪10s；外进程控制另有实测：`case plan1066-s-doc-controls` 6 测 5.14s（单测 ≤1.5s）、`infra-fixed` 的 NC-09 2 测 0.068s / NC-15 2 测 0.013s。非零父 exit 不代命中、缺计时/NOT_RUN 不冒绿已由 NC-20 守护；最终完整并集计时归 T-11。门：corpus 240/0、scheduler 控 6/0、plan1066 21 passed/3 ignored |
| [x] | T-11/work | A：T-09[A]/T-10[A]；B：T-09/10完整面 | 冻结代码/工具/配置后资源检查与完整影响组/必要consumer并集，复用有效phase证据；模块/新例/排除/红/等待/准备计时汇总，零新warning/格式/调试残留；AC-08  本轮阶段：A片段影响/健康；B最终完整并集；片段完成不勾整项。  **[✅ 2026-11-17]** 冻结提交 `6682a6d0a`（source_state `da5c9599dcf61a08`、config `0bc07b11a19a2ad2`）上跑**一次并集**：`cargo t corpus a2r vm auto-test-case infra-fixed plan1066-s-doc-controls case-ace-m2-scheduler-968` → gate PASS，5 个 invocation 全部 PASS，executed **1810**（corpus+a2r+vm 1570 / auto-test-case 151 / infra-fixed 72 / 控制组 6 / scheduler 11），wall **155.2s**（准备/发现/执行/清理均含在 drive 计时内）。覆盖真实触面：core(`test_runner`/`entry`/`legacy_doc_for_trans`/`test_cases`)+auto-lang 十个 corpus wrapper+auto-test-case 解码器+治理脚本 4 个+固定组登记 1 个。健康面：`rustfmt --check` 与 base `7c131286d` 逐文件对照，**11 个原本 clean 的文件被本轮改动弄脏 → 已格式化，现 0 个 newly-unformatted**；`a2r_tests/a2c_tests/a2ts_tests/tests.rs/legacy_doc/author.rs/inventory.rs` 在 base 即非 rustfmt-clean（属存量），本轮未扩大其未格式化面。调试残留：新增行无 `dbg!/todo!/unimplemented!/FIXME`；仅显式工具用 `eprintln!` 汇报（payload/actuals/compile prepare/proposal），普通路径无输出副作用。**跨计划更正（具名，供复审核）**：`scripts/tests/test_fixed_groups.py::test_ace_set_is_explicit_and_separate_from_daily` 自 PLAN-969 `278b91204` 起期望 3 组而 toml 已含 `ace-facade`（基面即红）——本轮把断言同步为登记的 4 成员集合，infra-fixed 因此转绿；这是事实同步、未放宽该测试意图。集成/消费者覆盖与计时汇总见 §6.3 与各轮收据，最终并集不含 Linux/WSL 腿（按 2026-11-14 裁定归专用环境）。 |
| [ ] | T-12/review | A：T-01/02/03/07/08/09/10/11的A面；B：T-01～11全部 | 同树独立重建AC-01～09、逐义务/actual/全部NC、Spec delta/遗漏/延期/workaround；新增例超10/缺计时needs_fix；记录身份/限制与冻结门交接；AC-09  本轮阶段：A独立phase复核；完整final在B；片段完成不勾整项。 |
| [ ] | T-13/merge | A：T-12[A] phase pass；B：T-12完整final pass | 核冻结联合门/必要due，SD-01～05落地、保1062更正并据真实结果关闭债项；归档本Plan，摘要入Plan后对账全部输出并清scope/guard后清树/分支和登记；AC-10  本轮阶段：A片段正式fold保树；B完整交付/归档清树；片段完成不勾整项。 |

## 9. 复审记录

2026-11-16 stage=new | PLAN-1066 r1 | outcome=pass（规划交接，非实现review） | status=drafting 0/13 | next=work（收到用户实施指令后T-01）。本轮只取号、静态盘点及已授权的1062收口事实更正，未实施/编译/执行产品/启动WSL/切入口/建实施树。目录盘点不是产品测试PASS。

规划静态预检：N1050项的源/doc哈希与当前文件逐项相符、1161个文件oracle引用，S11、工程19断言、C33验证/30生成闭集、13任务/10 AC/10 V/20 NC及0～10章节、围栏/表格列数通过，第一轮核验0.631秒；新增计划与更正链接16处存在，1062 frontmatter及20项已勾任务逐字保留，复核0.959秒。git diff --check通过；新ID唯一，.next-id由官方脚本推进到1067（后续分配不得回退）。修订文件仅计划/规范事实更正/路线与债项簿记，其他会话WIP保留。无测试模块运行或新增测试case实施，所有V/NC/切换证据待work。

### 2026-11-16 规划复审：r1 needs_fix → r2 planning_pass（未实施）

用户本轮授权仅复审并完善计划，不授权实施。review_scope=plan，起始HEAD=aa257fa1425c31fab48bd544cbd116b2fdce7dcd；核验末主线推进到107b23d0f69c6ad11d036dc37bed0b688493db30，所核test_runner/entry/sys_env/inventory/包装/S语料/fixed-tests/配置/治理文件在这段主线历史中无差异。实际检查位置为主检出D:/autostack/auto-lang。本复审与起草同会话，基于当前源码/原件重新盘点，**不冒独立实施final review或产品PASS**；实施树尚未创建。

| finding | r1规划缺口/代码事实 | r2补正文档与未来关闭owner |
|---|---|---|
| PR-01 | N0028/N0030已有doc漏原result，不能仅写“已存在1050文档” | §5.2列succ/10精确字节/hash，T-03补载荷与两consumer，NC-21；代码未修 |
| PR-02 | 1161只数现存oracle，漏9个缺H仍判空的角色 | 明列九个case与1170角色分母，T-03/AC-01/02/NC-22；不以缺席跳过header |
| PR-03 | 只提VM额外10项，未列其余68 doc；cache靠旧扩展名判模式，cookbook108靠旧.at选择/6目录多源 | 新E78/B108附录，V-11、T-01/02/03/08/09、NC-23/24；Windows77/31和原实际选源保，未实选候选不误升 |
| PR-04 | P只列2工程/250门，漏3个ignored guard仍复制6份普通金样 | M01～03逐输入/原witness/独立参考对照，4个真正fixture具名hash，T-06/NC-27；原ignore/义务不减 |
| PR-05 | 仅删bootstrap不足以消除.wrong.out/GENERATE_EXPECTED作者写入旁路 | T-03/07/08明确成功/失败/生成flag的完整文件集合守卫与scope输出，NC-25；不扩大C30闭集 |
| PR-06 | 正常库存仍可能按旧source标签核文件；native名/转译参数未分消费者写清 | 明确当前v2/显式历史v1、CLI/VM helper/association标签与实际doc origin；原prefix/module参数保，T-02/08与NC-26 |
| PR-07 | plain a2r-compile无法自动选ignored，legacy group不能挂ACE tools；T-06/T-08和计时前后有隐性等待风险 | 精确ignored+test-name入口、all=false/tools=[]，T-08先于T-06，T-10单例计时先于T-11最终并集；具名种子和actual路由判据补齐 |

以上“planning_pass”表示原四类及本轮同类可达路径已经有明确输入/入口/判据/反控/任务；不表示这些代码缺陷已关闭。T-02仍负责有界落定共享视图/public structs字段兼容与实现接口，先证明全路径再开展依赖项；其未执行不得冒已可运行。保status=drafting/current_step=0/total_steps=13；plan_revision因合同补强递增2。旧AC/T/V/NC编号保留，新增V-11、NC-21～27及E/B/M映射，不删除原义务或另拆计划。

静态复核证据：逐项独立重盘N集合/源/doc hash1050/1050、现存oracle1161+空H9、S11、P19断言、M3/6输入、C30生成/33验证、E78全路径/hash、B108声明/6多源、13任务/10 AC/11 V/27 NC、链接/0～10章节/表格/围栏与显式依赖无环通过；结构/集合核验0.534秒。另辅助载荷字节检查1050源与1161现存oracle，只有PR-01两处缺result（0.632秒）；这是发现缺口的静态证据，不是生产decoder或真实执行绿。未新增/迁移任何case、未编译、未运行测试模块/产品/WSL、未切入口；现有材料仅系统临时目录和本Plan，其他会话WIP保持。

2026-11-16 stage:new/revision | PLAN-1066 r3 | outcome:pass（规划分类/静态合同，非产品或A出口PASS） | 13任务/0完成保持；原AC/NC与完整最终义务保留，新增AC-A-CORRECTNESS及NC-A1..4；A正确性小切片，格式/编译全集B | next:原owner未来获实施指令后按现合同执行；当前仅规划，不建树/写代码/运行测试/合入/清理/恢复目标。

本轮规划静态预检（非产品PASS）：18份规划/路线/债务文档；13份Plan共92个原任务，任务ID/勾选/状态/完成数/整包依赖保持。985本轮162父行分类索引唯一、F01..35唯一且全覆盖（20条A正确性、6条A约束/证据、2条已交复用、7条B）；新增本地链接/锚点、章节、表格、阶段/任务/原Case映射与行尾核对通过，阶段图34节点36边无环，1066的A终验不再依赖B-only格式/项目/250编译任务。仅验收合同冻结；真实矩阵/adapter/新控制及全部新产品结果仍待原work任务，未运行/签A。未改源码/测试资产/canonical Specs/取号计数器，未实施/建树/构建/窗口/合入/归档/清理或恢复目标。

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 1 | from: new | to: work | status: drafting→executing | outcome: pass（授权登记，无代码写入） | worktree: 待建 D:/autostack/.wt/lang-1066/auto-lang | commit: master 634b7d5c9（计划文件未入库，本轮随簿记提交） | next: work（T-01[A]）

### 2026-11-17 work：A 切片实施记录（T-01/02/03/07/08/09/10/11 的 A 子面）

stage: work | plan_id: PLAN-1066 | plan_revision: r3 | outcome: pass（A 子面完，13 任务整项不勾） | worktree: D:/autostack/.wt/lang-1066/auto-lang | branch: plan-1066-dev | base: fa8e3ccdb | code commits: dec529355 + 289e71b37 | dependency_revisions: 无跨仓 | 材料scope: D:/autostack/plan-artifacts/plan-1066/（probes/：冻结fixture副本、A receipt、vm+corpus门结果） | next_responsible_role: review（T-12[A] 独立 phase 复核）

**T-01[A]（A-DEN 冻结基线）**：新模块 `crates/auto-lang/src/tests/plan1066_a_slice.rs` + 冻结契约 `crates/auto-lang/test/governance/plan1066_a_obligations.json`（657KB，schema plan1066-a-obligations/1，*.json 母规则白名单豁免）。同源派生 `derive_actual()`：N 五根走 `test_runner::discover_*`（产品发现器）、vm_cache 走 `load_vm_test_cache`（506 键）、B108 走 syn 解析 cookbook_vm_tests.rs、S11 走 scheduler.case.ad test-case 块、P 19 断言走两旧 Rust 测试体有序扫描（mini-lexer：注释剥离+字面量槽）、C30/33 走 conformance_tests.rs 解析、E78=五根全 case.ad 扫描减 N 集、P250域=a2r 行、ledger 36 条、M3 guard #[ignore] 保持。**对账计划表（probes 脚本）**：N 1050/1050 行 0 缺 0 多、源 hash16 0 错；doc hash16 仅 N0028/N0030 两处预期差（A 补通道）；S11 11/11 全 SHA 吻合；C33+E78 111 份 doc 全 SHA 吻合；两旧 result oracle 盘上未动（SHA=05cc1c…/4a44dc…）；角色总数 1170（vm out295+result189+error14 / a2r rs250 / a2c c118+h109+h-empty9 / a2ts ts85 / cookbook rs101）精确等于计划分母。B108=108 绑定（77 active/30 ignore/1 cfg_attr(windows)）；6 多源目录=附录 B 全同（channel/fs/rt/postgres/sqlite/scraping，同源 read_dir 实选均 001_* 且有 doc，无 doc 候选记录未升级）。逐键 A/B/平台/ignore/actual 来源落在 fixture `segments`（A+B、windows、old-file fence 冻结选择、doc-only 发现=B）。

**T-02[A]（正确性子面决策）**：①A 冻结基线=当前 old-file fence 选择集+doc 载荷身份；doc-only 资格=T-02/03[B]。②E78 边界按 doc 集合冻结，NC-A1 跨 family 键防吞并。③身份保持：FileTestCase.name=category/case、A2rTestCase.name=suite/category/case（discover 原样）；compile gate 全名参数=B。④空H=h-empty 显式零字节角色（compare_frozen 逐角色拒绝缺席化）。⑤vm 双通道=单次运行双轴判定（run_vm_doc_once/diff_vm_channels）。⑥scope 输出=AUTO_TEST_DIAG_SCOPE 显式根，默认丢弃。⑦public structs 零字段变更（新 API 均 pub(crate)）。

**T-03[A]（A-ORACLE）**：N0028/N0030 doc 各补 `check-id: "vm-result"`+`final-newline: false`（succ/10，无尾LF；多 check 节为 author 格式既有能力，此为首例双通道 VM doc）。VM doc 消费者：cache loader 复用 `vm_doc_oracle`；`run_vm_doc_once` 单次执行双通道采集；`diff_vm_channels` 逐轴判定+轴归属。旧入口身份绑定：`build_dual_channel_receipt` 内 `run_vm_file_test`（旧 discover+旧 oracle 文件）对两例均绿——恢复的 doc 通道等值原 oracle 字节。receipt（scope/probes/plan1066_a_receipt.json）：2 doc-face check（stdout+result 双观察）+2 legacy-entry child（exit 0），engine 身份=legacy-autovm（金通道原引擎；ACE producer 不在本 receipt 声明，985 adapter dispatch 归 985）。

**T-07[A]（A-NOWRITE）**：core 新 `diag_scope::write_scoped`（默认丢弃/显式 scope 镜像 `<scope>/plan-1066-diag/<rel>`；scope 非存在目录=硬错）；tests/diag_scope.rs 复用+行为测试（corpus 组日常跑）。切 scope 的 A 执行路径写入点：vm_file_tests wrong.out/wrong.result、conformance wrong.out+generate_expected 候选、cookbook wrong.out+GENERATE_EXPECTED 候选、entry.rs VM 两轴 wrong 写入。`conformance_bootstrap` 退出普通注册（#[ignore] 精确理由；33 验证原样，corpus 组选择-1 生成器）。B 保留（未迁，具名）：entry.rs a2c/a2ts/a2r 四写入点、a2r/a2c/a2ts 包装 wrong 写入、aavm_runner/ffi_dual/plan724/727/book_listing/tscn 写入点、完整 proposal 与 v2 治理。

**T-08[A]/T-09[A]（证据身份/实际运行）**：ARReceipt/verify_receipt（schema plan1066-a-receipt/1）拒签轴=过期源 hash/旧 sidecar 冒 doc-face/父 PASS 掩盖失败子/缺工具/NOT_RUN/缺观察通道；`plan1066_a_tools`（ignored 显式工具）可再冻结/出 receipt。实际消费者执行：N0028/N0030 cache wrapper（--ignored 具名）0.28/0.35s 绿；邻例 hello(arithmetic)/let_asn_error 0.27-0.29s 绿；旧入口身份 2 child exit0；分母门 integrity 1218ms 全键对账。

**T-10[A]（NC-A1..4）**：A1=五类 fixture 变异（删N末项/跨族并键/9空H缺席/删P断言/删S11+C验证）各具名红+诚实图绿；A2=镜像只错 result（sucC/11）单轴红+原值单次双通道绿+删节通道守恒红（vs 冻结 doc_checks）；A3=conformance 错 oracle 镜像红+cookbook GENERATE_EXPECTED 镜像候选只落 scope+作者树（conformance/cookbook/vm 三根 tree_hash）前后零变化；A4=真 receipt 过+六类篡改各具名拒。计时：integrity 1218ms/nc-a1 1225ms/nc-a2 150ms/nc-a3 ~500ms/nc-a4 ~200ms，全部≤10s。

**T-11[A]（影响组/健康）**：冻结提交 289e71b37 上 `cargo t vm corpus` PASS（选择 1137=auto-lang 587+core 550，执行 1137，排除 467，wall 31.1s；驱动 input_bindings/child_validation 全过）。首跑红一次：NC-A3 cookbook 镜像 trace 污染治理流（"legacy wrapper input escapes repository"）→ run_cookbook_case 增 emit_trace=false 修复（289e71b37），合成输入不入 trace。零新编译告警（预存 vm_file_tests.rs:1676 双 #[ignore] 告警与 mem-profile mod 缺失为 master 存量，非本切片）。rustfmt 已过（本切片文件）；全仓 cargo fmt 因 master 存量 cfg 门 mem_profile 缺文件不可用，未动他人文件。auto-test-case/infra-fixed 组未触及（crate 零改动）；a2r/a2c/a2ts 组行为面未变（仅 entry.rs VM 写入点改，其 impl 代码路径未动），按真实触面未选，review 可核。

**B 余量（原义务不减）**：T-04/05/06 整项、T-03[B] N/caches/cookbook 全量 doc-only 切换、T-07[B] proposal CLI+全部旁路退出、T-08[B] v2 治理、T-09[B] 全路径重扫、T-10[B] 27NC、T-11[B] 完整并集、T-13[B] 完整归档清树。worktree/分支保留给 B。

blockers: 无。goal round 2 receipt 见 §9 末。

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 2 | from: work | to: review | status: executing | outcome: pass（A 子面 work 完：dec529355+289e71b37，cargo t vm corpus PASS 1137/1137，NC-A1..4 全绿） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: 289e71b37 | next: review（T-12[A]）

### 2026-11-17 T-12[A] 独立 phase 复核：pass（F-1066A-R1/R2 已修闭）stage: review | review_scope: phase（A 切片） | reviewer: 独立上下文代理（与实施会话分离；同树只读重建） | plan_id: PLAN-1066 | plan_revision: r3 | outcome: pass | worktree: D:/autostack/.wt/lang-1066/auto-lang | branch: plan-1066-dev | reviewed_commit: 289e71b37（初判）→ 2e606d28f（修复确认，终判冻结点） | base_commit: fa8e3ccdb | dependency_revisions: 无跨仓 | spec_inputs: 本 Plan §5 A-EXIT/§6.3/§7/§8；canonical Specs 未动（SD-01..05/SD-A-PHASE 沉淀归 T-13[B]） | acceptance_results: A-DEN pass / A-ORACLE pass / A-NOWRITE pass / A-EVIDENCE pass（R1 修复后） / NC-A1..4 pass / B 义务保持 pass（13 任务框未勾、树/分支保留、旧 oracle 未动） | findings: F-1066A-R1（低，verify_receipt 空选择漏拒）→ 2e606d28f 修复+NC-A4(g) 实证；F-1066A-R2（低，segments 未对账）→ 2e606d28f 修复+NC-A1(f) 实证；F-1066A-R3（信息，tests.rs rustfmt 重排惰性）不处置 | evidence: 复核代理独立重析 fixture（python 逐段再哈希 24 项全对）、全 diff 逐文件核对（14 文件无越界）、P mini-lexer 逐行对照两旧测试体（19 断言 1:1 零误报）、NC-A2 变异精度核查、receipt 重哈希、基线告警溯源（2 条告警均在 fa8e3ccdb 存在=零新增）；计时 integrity 1088ms/nc-a1 1097ms/nc-a2 188ms/nc-a3 365ms/nc-a4 186ms（≤10s 全过） | 修复后门：cargo t vm corpus @2e606d28f PASS（run-0o5gxy7_，executed 1137/excluded 467，wall 182.1s；excluded 名单可见 conformance_bootstrap 以 ignore 理由退场=生成器-1、33 验证不减的账面证据；结果归档 plan-artifacts/plan-1066/probes/vm_corpus_gate_2e606d28f.json） | next: T-13[A] phase fold（保树予 B）；Plan 状态保持 executing（phase pass 不授 reviewed/归档）

### 2026-11-17 T-13[A] A 片段正式 fold：IF-1066-A-CORRECTNESS 交付（保树予 B）

stage: merge | review_scope: phase | plan_id: PLAN-1066 | plan_revision: r3 | outcome: pass（A 片段交付；B 余量在途） | worktree: D:/autostack/.wt/lang-1066/auto-lang（保留） | branch: plan-1066-dev（保留）

| checkpoint | 证据 |
|---|---|
| prepared | 复核冻结点 2e606d28f（phase pass，R1/R2 闭）；wt-guard clean（无 reparse point）；无 canonical Spec delta（本 Plan 规范增量节明示 SD-01..05/SD-A-PHASE 实施后由 B 的 review/merge 沉淀，A 片段零 spec 改动=合同内 no-impact） |
| landed | rebase plan-1066-dev → master（docs-only 基线 34cbfbd6b + 旁会话 b73a3fc70，无冲突）；`git range-diff master..2e606d28f master..HEAD` 三提交全 `=`（dec529355=a18ab5a88、289e71b37=ed33c0a3e、2e606d28f=7c131286d=delivery commit）；`git diff 2e606d28f HEAD -- crates/ scripts/` 空（代码/配置/输入零漂移→门证据复用合法）；`git merge --ff-only` 落地，master tip=7c131286d，线性历史无 merge commit |
| canonical_verified | A 片段无 canonical spec 改动（上述批准的 no-impact）；B 的 SD-01..05/SD-A-PHASE 沉淀义务保留在 §5 规范增量表，T-13[B] 完整 merge 时落地 |
| archived | 不适用（Plan 保 executing；A 片段不假归档，13 任务整项待 B 完整范围） |
| cleaned | 不适用（worktree/分支/plan-artifacts scope 全部保留予 B；REGISTRY 更新 in-flight(B)） |

**IF-1066-A-CORRECTNESS 交付内容（985/A 可消费的正确性切片，真实来源=master 7c131286d）**：
1. 冻结分母契约 `crates/auto-lang/test/governance/plan1066_a_obligations.json`（schema plan1066-a-obligations/1）+ 常驻逐键门 `tests::plan1066_a_slice::plan1066_a_denominator_integrity`（corpus 组日常；篡改/缺项/并键/空H缺席/segments 改动必红）。
2. N0028/N0030 双通道 oracle 恢复（vm-result succ/10 无尾LF）+ 单次运行双轴判定消费者（run_vm_doc_once/diff_vm_channels）+ 旧入口身份绑定绿（receipt children）。
3. A 执行路径零作者树写入（diag_scope：默认丢弃/AUTO_TEST_DIAG_SCOPE 镜像）；conformance_bootstrap 退普通注册（33 验证不减，gate excluded 名单可证）。
4. 证据 receipt 合同（schema plan1066-a-receipt/1）+ verify_receipt 拒签轴（过期 hash/旧文件冒 doc-face/父PASS掩盖/缺工具/NOT_RUN/缺通道/空选择）+ NC-A1..4 常驻负控。
5. 边界：engine 身份=legacy-autovm（金通道原引擎，具名对照；不冒 ACE 通过——985 adapter dispatch 归 985）；doc-only 发现/全量载体迁移/proposal/v2 治理/250 编译=保 B，原义务与 27NC 不减。

**落地检查**：master 主检出 `cargo test -p auto-lang --lib plan1066` 5 passed/1.28s（主检出代码面 clean）；goal round 4 收据见下。批次回归不触发（未达 cleaned；1066 非 5 倍数，due 检查归 B 的完整 T-13 或独立 regress 档）。

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 4 | from: merge | to: done(A) | status: executing（B 在途） | outcome: pass（IF-1066-A-CORRECTNESS 已交付 master 7c131286d；worktree/branch/scope 保留予 B） | next: user（B 实施另行授权启动）

### 2026-11-17 B 期 work（round 1）：T-04 S 载体迁移交付（NC-08/09 控已实证，常驻接线待落）

stage: work | plan_id: PLAN-1066 | plan_revision: r3 | outcome: partial（T-04 主体交付；整项不勾，常驻负控接线未落） | worktree: D:/autostack/.wt/lang-1066/auto-lang | branch: plan-1066-dev | base: 7c131286d（A fold 交付点，主线此后仅他计划 docs/ACE 提交） | code_commit: aba42ca3a | dependency_revisions: 无跨仓 | task_ids: T-04（B-FORMAT，其中 NC-08/09 仅 scope 探针） | next_responsible_role: work（同树继续 B）

**T-04 交付内容**：`crates/auto-lang/test/cases/ace-m2/scheduler/scheduler.document.case.ad`（11 例 AutoDown 文档式：`## Case:` + 内联 `### Auto` + `### Probe av-scheduler`/`#### stdout`/`#### stderr`，负例 stage+code，S11 保 message-contains）；suite 唯一 include 切到该文档，`scheduler.case.ad` 留盘作冻结历史且不再进入活跃入口。生成器逐字节取冻结 `src/*.at` 与冻结 atom 块期望（脚本 `plan-artifacts/plan-1066/probes/gen_s_doc.py`，非手抄）。

**实跑门（真 consumer）**：`cargo t case-ace-m2-scheduler-968` → PASS，executed 11/11（wall 171.77s：prepare 冷建 av-worker release + auto-case；run 1.18s）；逐例 duration_ms 351～672，最大 0.672s ≪10s。standalone `auto-case run --suite scheduler.suite.ad` 1.49s 11/11（未预热工具时受同机他计划构建影响的一次侧跑另计，属资源等待非用例成本）。

**新旧等价（S11 原身份）**：冻结 fence suite 镜像（只含 `scheduler.case.ad` + `src/`）与新文档 suite 各跑一次真实 consumer，`report.json` 逐行比对（脚本 `probes/compare_s_runs.py`）：11/11 case id/check id/status/detail 全同；`identity.source` 与 `identity.oracle` 内容哈希 11/11 逐例相同；native counters/toolchain 相同。仅 `identity.suite`/`identity.case_file`/`identity.contract` 三轴随载体迁移漂移 33 行——与 `docs/specs/auto-test-case/document-cases.md`「内联期望与身份」已记录的 contract/check 表示轴漂移同口径（source/oracle 内容轴不变），不冒源/期望漂移。

**NC-08（scope 探针，真 auto-case；`probes/nc08_s_controls.py`）**：单轴变异各具名红——S01 stdout 去尾 LF（`final-newline: false`）→ `stdout[bytes] mismatch at byte 7`；S02 stderr 由空改非空 → `stderr[bytes] mismatch at byte 0`；exit 0→1 → `exit[exit] mismatch`；S09 code 错 → `code[code] mismatch`；S11 message-contains 不命中 → 红。未变异镜像 11/11 绿且与真文档逐字节同。单控 ≈1.5s。

**NC-09（scope 探针，活跃入口作者格式扫描；`probes/active_entry_scan.py`+`nc09_scan_controls.py`）**：真实树 76 个 `suite` 组、183 个被 include 的 case 文件 → document 183 / fence 0 / unknown 0（T-04 切换后旧围栏已零活跃入口，与 r2「75组」表述的差额正是本任务补的 scheduler）。控：suite 仍 include 旧 fence 文件、新增旧格式组、被投毒文件三种各自具名定位到 group/suite/case；文档式镜像绿。扫描 0.6s。

**未闭/下一手**：①T-04 整项待两件事——NC-08/09 由 scope 探针提为常驻门（决定：按 `test_case_pilot_ace.py` 先例落一个显式 ACE python 组 `tools=[case-parent,av]` 承载 CLI 级控，NC-09 扫描落 `scripts/test-governance/` 并入 `infra-fixed`），以及 AC-03 逐项证据表；②T-03[B]/T-05/T-06/T-07[B]/T-08[B]/T-09[B]/T-10[B]/T-11[B] 未开工（A 已冻结的 1050 分母与四家族旧文件读取路径仍在：`test_runner::discover_*` 仍按 `.at`/`.expected.*` 存在性选例、`entry::run_*_file_test_impl` 仍直接读旧源/旧 expected、a2r/a2c/a2ts 仍写 `.wrong.*` 到作者树、VM cache `is_script` 仍由旧 `.as` 文件判、cookbook108/506 缓存与 v2 治理未切）。③本 Plan 13 任务仍 0 勾（T-04 只闭其中载体+控的实证）。

**同机资源**：本轮起跑前观测到 977/1067/727 等旁会话 cargo/nextest 构建并发；S 门与侧跑均在真实竞争下完成，已按实测记录（准备/执行分列）。

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 1 | from: work | to: work | status: executing | outcome: partial（T-04 载体迁移+等价+NC-08/09 scope 实证已交 aba42ca3a；常驻控接线与 T-03[B] 起未开） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: aba42ca3a | next: work（B 期继续：常驻 NC-08/09 接线 → T-03[B] doc-only 发现/执行切换）

### 2026-11-17 B 期 work（round 2）：T-04 关闭——NC-08/09 常驻化落地

stage: work | plan_id: PLAN-1066 | plan_revision: r3 | outcome: pass（T-04 整项关闭，13任务 1/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | branch: plan-1066-dev | base: 7c131286d | code_commits: aba42ca3a（载体）+ 1aea0ef5e（常驻门） | dependency_revisions: 无跨仓 | task_ids: T-04 | next_responsible_role: work（T-03[B]）

**NC-09 常驻（治理扫描，纯只读）**：新 `scripts/test-governance/document_case_active_scan.py`（`scan(root)`：读 test-groups.toml 每组真 suite → include → 逐个分类 document/fence/unknown，报 group/suite/case；`report()` 人读；CLI 非零退出）。`scripts/tests/test_fixed_groups.py` 新增 `ActiveDocumentCaseEntries` 2 测（0.068s）：①真树 `fence=0/unknown=0/offenders=[]/missing=[]` 且 groups≥76、files≥183（防空跑绿）；②临时树镜像控——文档式绿、陈旧 fence include 报 `OFFENDER[fence] group=case-stale suite=... case=...`、投毒文件报 `[unknown]`、缺 include 报 `missing`。归 `infra-fixed`（日常 python 组）。

**NC-08 常驻（真 consumer CLI 级）**：新 `scripts/tests/test_plan1066_case_doc_controls.py`（6 测，实测 5.14s，单测 ≤1.5s）+ 显式 ACE 组 `plan1066-s-doc-controls`（runner=python、domain=ace、tools=[case-parent,av]、all=false，登记于 `scripts/test-groups.toml`）：clean 镜像与真文档逐字节同且 11/11 绿；五个单轴变异各具名红——`final-newline: false`→`stdout[bytes] mismatch`、stderr 非空→`stderr[bytes] mismatch`、exit 0→1→`exit[exit] mismatch`、S09 code 错→`code[code] mismatch`、S11 message 不命中→`message[...] mismatch`；每控断言变异不泄漏到邻例（其余 10 例全 PASS）。`cargo t plan1066-s-doc-controls` → PASS（executed 6、skipped 0、failures 0；wall 26.37s 含工具身份复检，用例本体 5.14s）。组为显式（all=false），不进日常。

**同轮门与既有红归属**：`cargo t infra-fixed` 70 测 4.04s，唯一失败 `test_ace_set_is_explicit_and_separate_from_daily`（期望 3 组，实得含 `ace-facade`）——静态归属 PLAN-969 提交 278b91204 把 `ace-facade` 加进 `ace` 集合却未更新断言；该断言在基面 7c131286d 的测试文本与 toml 集合已互相矛盾（`git show 7c131286d:scripts/tests/test_fixed_groups.py:281` vs 同提交 toml `ace = [... "ace-facade"]`），故为**基面既有红，非本任务引入**；本轮新增 2 测在该次运行内通过。未在本树修（属 969 面，需各自 fix worktree），供 T-11 联合门前处置/通知用户。

**AC-03 证据（S）**：①S11 原实际值——11/11 真实 consumer PASS，逐例 duration_ms 351–672；②完整 v2 无损——新 doc 11 例投影与冻结 fence 件逐行等价，`identity.source`/`identity.oracle` 11/11 内容哈希相同，native counters/toolchain 相同；③正常 suite 全 doc——真树 76 组/183 include 文件 100% document（扫描常驻）；④其他已迁身份保持——本任务 diff 仅 scheduler 目录 + 治理/组登记，其余 75 组文件零改动。

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 2 | from: work | to: work | status: executing | outcome: pass（T-04 关闭：载体 aba42ca3a + 常驻 NC-08/09 1aea0ef5e；13任务 1/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: 1aea0ef5e | next: work（T-03[B] N 四族 doc-only 发现/执行切换）

### 2026-11-17 B 期 work（round 3）：T-03[B] N 四族文档权威切换交付（T-08 首片同轮）

stage: work | plan_id: PLAN-1066 | plan_revision: r3 | outcome: partial（T-03[B] 主体+NC-02..06/21 交付；T-03 整项不勾，余 NC-07/22/23/24 与 shim 1050 重放） | worktree: D:/autostack/.wt/lang-1066/auto-lang | branch: plan-1066-dev | base: 7c131286d | code_commits: 72d7c2bd3（N 族切换）+ 250b93e49（T-08 首片/分母增量） | dependency_revisions: 无跨仓 | task_ids: T-03（B 面）、T-08（首片） | next_responsible_role: work

**T-03[B] 交付**：迁移文档成为 N 四族唯一作者源——`test_runner::discover_{vm,a2r,a2c,a2ts}` 按 `<stem>.case.ad` 选例（doc-only 树照常发现；文档缺失但冻结 sidecar 仍在时**保留该项**并让执行报 `migration document: …`，绝不静默缩集/回退）；虚拟源路径与 oracle 通道（vm-stdout/vm-result/vm-error、`.rs`/`.c`/`.h`/`.ts` artifact、a2c 空 H 零字节角色）全部取自文档；`entry::run_*_file_test_impl` 改读文档源/内联 oracle，a2r/a2c/a2ts 的 `.wrong.*` 改写为 `diag_scope` 作用域输出（作者树零写入）；四族 trace 身份改记文档（`document`），VM cache `is_script` 由文档声明路径 `.as` 判定（不再看哪个 sidecar 存在）、缺文档=具名 panic；cookbook 选择改为「目录内唯一 `*.case.ad`」（多文档=歧义拒绝、无文档=硬错），oracle 用 `cookbook-stdout`。

**T-08 首片**：`execution_trace::cookbook_doc_identity` 单一派生函数同时服务运行期 trace 与源码静态 inventory（`inventory_wrappers.rs`），以 `document:{path}` 形态让两侧治理 normalize 都落到仓库相对路径；`legacy-wrapper-contract.json` 108 条 cookbook 关联行整体改绑文档（sidecar 身份退役，非 cookbook 行零改动）。**分母增量登记**：文档权威暴露出 10 个旧 sidecar 选例静默跳过的 VM 项（`08_strings/017_sub_semantics` 的 oracle 用整目录名命名；`10_types/033_array_len_user_type`、`17_cffi/{001,002,003}`、`18_ffi/{047,055}`、`99_bootstrap/{066,067,068}` 只有文档内 oracle）——A 冻结分母 1050→1060、E78 78→68，其余集合（cache506/8 脚本、cookbook108/6 多源、S11、P、C）逐字节不变；A 族计数与 provenance 文本同步更新，A-DEN/NC-A1..A4 全绿。

**R2 更正（2026-11-17 复审）**：上文称这 10 项只有文档内 oracle 并不准确——`08_strings/017_sub_semantics` 另有一份 13 字节冻结 `017_sub_semantics.expected.result`（按整目录名命名，原选例规则未拾取），其内容正是五条 `print` 的 stdout；已把这 13 字节作为该文档的 `vm-stdout` oracle（`el\nell\nel\n\no\n`），该例从 smoke 升为真实输出判定，角色总数因此 1170→1171，冻结分母仍 1060。其余 9 项确为文档内 oracle。

**门与负控**：`cargo t vm corpus` **PASS**（1137 executed、0 errors、child_validation valid、77 条 legacy child 关联、wall 56.1s；本轮经历三次治理红并逐一定位修复：①A 分母漂移→增量登记；②cookbook `legacy_children` 漂移→单派生函数；③`legacy-wrapper-contract` 关联丢失→108 行改绑。其间发现治理工具陈旧二进制问题，见债务）。新增 `tests/plan1066_b_slice.rs` 4 控（0.83s）：四族 doc-only 镜像仍发现并执行；**删文档保留 sidecar**→集合不缩、运行报 `migration document … .case.ad` 硬错；污染旧 `.at`/`.expected.out` 判定不变、改文档源即红（stdout 轴）；双通道例只错 `vm-result` 只在该轴红。`cargo t plan1066-s-doc-controls` 仍 PASS。

**本族既有红/债务（不代修、不充 PASS）**：①a2ts 85 例金样早于当前 emit（`console.log`→`globalThis.console.log`），整族显式重放统一 `transpiler output mismatch` 且全 `#[ignore]`——记 D-1066-A2TS-GOLDEN（改写期望需用户许可）；②治理 input inventory 以 `source_state:"unknown"` 收据复用陈旧 auto-case（实测造成假红，删收据重建后 PASS）——记 D-1066-STALE-TOOL-RECEIPT；③`infra-fixed` 的 ace-facade 基面红记 D-1066-INFRA-FIXED-ACE-FACADE。另见 `KNOWN-DEBT` 既有 D-1037-01（ac-semantics 文档语料缺 `status: executed` 谓词，owner 记 1062/1066）——下一轮按 1066 owner 处置。

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 3 | from: work | to: work | status: executing | outcome: partial（T-03[B] 主体 72d7c2bd3 + T-08 首片 250b93e49；vm corpus 门 PASS 1137/0；13任务仍 1/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: 250b93e49 | next: work（NC-07/22/23/24 + 四 shim 1050 重放 → 再 T-05/T-06）

### 2026-11-17 B 期 work（round 4）：T-03 关闭——余下负控 + 载荷/逐例证据

stage: work | plan_id: PLAN-1066 | plan_revision: r3 | outcome: pass（T-03 整项关闭，13任务 2/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | branch: plan-1066-dev | code_commit: 9bd255fbc | dependency_revisions: 无跨仓 | task_ids: T-03 | next_responsible_role: work（T-05/T-06）

**余下负控（常驻，`crates/auto-lang/src/tests/plan1066_b_slice.rs`）**：NC-06 bootstrap 规则=原 `99_bootstrap` 类目、文档正文被 prepend、改正文即红；NC-07 族 artifact 词汇不串（a2c 仅 .c/.h、a2ts 仅 .ts、a2r 仅 .rs、cookbook 有 VM oracle 时必须 `cookbook-stdout` 而非 Rust 面）；NC-22 空 H 零字节角色仍过（9 例）且 header 轴独立判红（`C header mismatch`，与 `.c` 同次转译分轴）；NC-23 E78 为键集非计数（N∩E78=∅、五根全部文档恰好分类一次、挪一个边界文档进 N 必须具名重复）；NC-24 `.as` 诱饵（删 frozen `.as`、放诱饵 `.at`）后虚拟源仍 `.as`、模式由文档定、镜像判定与仓库判定逐轴相同且诱饵正文从未执行；cookbook 六个多源目录选 `001_*` 文档。B 片共 10 测（含 2 个显式工具），活动 8 测 0.87s + bootstrap 0.29s；全套 plan1066 14 passed/3 ignored，1.66s。

**载荷等价（关键证据，`plan1066_b_payload_proof` 工具 + scope 比对）**：用产品解码器逐项取文档源/内联 artifact/check oracle 的 sha256，与冻结 sidecar 比对——**1060/1060 源、663/663 artifact、556/556 check oracle 全部逐字节相同**；4 个 `a2r/22_actors/*.expected.out` 属 `crates/a2r-actor-tests` 外消费者（`actor_parity.rs` 具名读取），不列入 N oracle（NC-07 反证）。故文档权威改变的是「谁读」，不是「读到的值」。

**逐例 actual（`plan1066_b_family_actuals` 工具，V-02）**：a2r 351 项（250+101 cookbook）1 红 `a2r/16_interop/024_nested_async_await`；a2c 118 项 8 红（field_access 类型错、const_generics/array_nested/list_iter C mismatch、array_zero_size 编译拒等）；a2ts 85 项 85 红（金样早于 `globalThis.console` emit）。三者逐例 wrapper 日常均 `#[ignore]`；同输入同 oracle 下旧 sidecar 路径判红相同（载荷等价已证），故为既有漂移、非迁移回归——登记 D-1066-IGNORED-CORPUS-DRIFT（a2r/a2c）与 D-1066-A2TS-GOLDEN，不充 PASS。

**V-03 shim 取证与边界**：`test/runners/{vm,a2r,a2c,a2ts}_tests.at` 四 shim 调用的就是被切换的 entry（`sys_env` 不改代码、只改其下游）；以镜像/路由等价控（NC-02..06/24）证明 shim 下游判定与仓库判定逐轴一致。全量 runner 重放（`test_file("test/runners/vm_tests.at")`，CWD=工作区根）推进到 2218 行后在 `99_plan230/001_struct_f64` 之后停住 >20min，另有 `99_p588debt/{002,003}` stdout mismatch——CWD/path 通道敏感，1066 前即存在，记 D-1066-SHIM-CWD-HANG，未以 wrappr 绿冒充 shim 全量绿。

**门**：`cargo t vm corpus` **PASS**（executed 1137、0 errors、child_validation valid、wall 50.2s）。

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 4 | from: work | to: work | status: executing | outcome: pass（T-03 关闭：载荷 1060/663/556 全等 + 9 负控 + 逐例 actual；13任务 2/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: 9bd255fbc | next: work（T-05 P 两工程 doc 化 → T-06 250 编译 doc 源）

### 2026-11-17 B 期 work（round 5）：T-05 关闭——两份工程文档 + 有限产物断言

stage: work | plan_id: PLAN-1066 | plan_revision: r3 | outcome: pass（T-05 整项关闭，13任务 3/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | branch: plan-1066-dev | code_commits: 477bc1fed（作者格式）+ 6c543e44f（工程文档/消费者/控） | dependency_revisions: 无跨仓 | task_ids: T-05 | next_responsible_role: work（T-06）

**作者格式扩展（477bc1fed）**：产物小节新增有限断言 `exists: true` 与 `contains: [...]`——与全文围栏**互斥**（同产物只能一种判据）、`final-newline/compare` 在有限模式下拒绝、未知键/空数组/空串/重复项/`exists: false` 全部具名定位拒绝；`ArtifactMatch::{Bytes, Exists{contains}, Contains{contains}}` 分型（Exists=显式声明、Contains=仅 contains，避免为 021 凭空生成第 4 条 exists 断言）；旧 decode 接受/拒绝与 v2 投影未变（`cargo t auto-test-case` 151/0 绿）。

**两份工程文档（6c543e44f）**：`a2r/14_modules/005_multi_file/main.project.case.ad`（id `legacy.a2r.project.14_modules.005_multi_file`，entry `main.at` 直接围栏 + `db.at`/`api/mod.at`/`api/handlers.at` 三个 `####` 附加源，16 条 P01..P16）与 `a2r/16_interop/021_path_dep/main.project.case.ad`（D01..D03，Cargo.toml 仅 contains）。5 份源在写文档前逐份校验 §5.3 冻结 SHA-256，再按冻结字节内联（final-newline 随源）。旧 `*.expected.rs` 保持历史、**从未被读取**。

**消费者**：`a2r_tests::run_project_case`（pub(crate)）——加载文档 → 逐路径 `validate_virtual_path`（绝对/`..`/重复/反斜杠写前拒绝）→ 物化到 `<temp>/<工程 basename>`（保留 entry 名、相对模块路径与工程目录名，Cargo 包名因此不变）→ 调原 `transpile_rust_project` → 对每个产物单独判 exists/contains 并具名失败；两个原 test 名保留、改为调用它（0.87s / 0.78s）。

**负控（常驻 1 测 2.11s）**：NC-10a doc-only 镜像可建；NC-10b 删 `db.at` 闭包即硬失败；NC-10c 改文档 `db.at` 正文 → `db.rs: struct Connection` 命中；NC-11 错 contains 具名（`fn connect_nowhere()`）、声明但未产物具名（`not-produced.txt: missing product`）；NC-12 `../`/绝对/重复源路径解码期拒绝且作者树零写入。

**分母与门**：A 的 P 派生改读两份文档（19 断言按 test/product/kind/literal 逐键不变；`sources`/`old_tests` 不变），E78 68→70（两份 project 文档入边界、不进 N），n_rows 1060 不变；A-DEN/NC-A1 全绿 1.6s。`cargo t a2r` **PASS** 432 executed/0 errors（wall 139s，含原 250 编译门）；`cargo t vm corpus` **PASS** 1137/0（79.9s，child_validation valid）；`cargo t auto-test-case` PASS 151/0（19.4s）；`cargo test -p auto-lang --lib plan1066` 15 passed/3 ignored（1.64s）。无新债务（工程文档按设计留在 E78 边界；a2ts/a2c 漂移债延续）。

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 5 | from: work | to: work | status: executing | outcome: pass（T-05 关闭：477bc1fed+6c543e44f；a2r 432/0、vm corpus 1137/0、auto-test-case 151/0；13任务 3/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: 6c543e44f | next: work（T-06 250 编译门 doc 源 + M3 手动 guard + a2r-compile 显式组）

### 2026-11-17 B 期 work（round 6）：T-06 主体交付（编译门两段式 + M 三 guard + 四控），NC-26 治理半边留 T-08

stage: work | plan_id: PLAN-1066 | plan_revision: r3 | outcome: partial（T-06 主体 5dbfb6a15；余 NC-26 的 v2 库存/trace 半边） | worktree: D:/autostack/.wt/lang-1066/auto-lang | branch: plan-1066-dev | code_commit: 5dbfb6a15 | dependency_revisions: 无跨仓 | task_ids: T-06 | next_responsible_role: work（T-08 库存切换时并闭 NC-26）

**编译门（AC-05 主体）**：`prepare_compile_receipt` 逐文档源（1060 中 a2r 250 项）用原 `transpile_rust_with_source_dir` 生成产品→独立 scope 写 `.rs`→真 `rustc --edition=2021 --crate-type=lib --emit=metadata`（debug，不 release、不 cargo）；`verify_compile_receipt` 只读收据：schema/toolchain(`rustc -vV` hash)/ledger hash/源 hash 任一陈旧即拒，逐行用**原始** `rustc_exit`+`stderr_head` 重算分类（`recompute_row_class`），伪造 compiled、缺 exit、ledger 项未归账均具名拒。实测 **159/55/36/0/0 = 250**（compiled/external-dep-skip/known-broken/obsolete/unexpected），prepare 22.74s、verify 0.42s；收据 `plan-artifacts/plan-1066/a2r-compile/compile-receipt.json`。第二入口 `a2r_compile_receipt_verify` 只验证不编译，缺收据即报「prepare missing」。

**显式组**：`a2r-compile`（all=false、tools=[]、domain=legacy、expression=`test(/^tests::a2r_tests::a2r_rustc_real_compile_gate$/)`）登记于 test-groups.toml；实测 `cargo t a2r-compile --ignored --test-name tests::a2r_tests::a2r_rustc_real_compile_gate` PASS（executed 1）。日常 `cargo t a2r` 由 139s→**56.3s**，该 test 明确出现在 `excluded(ignored)` 名单（无日常 rustc）；`--select-only` 证实 a2r-compile 不在 daily/a2r/all。

**M 三 guard（6 份 doc Rust 产物）**：新增 `materialize_doc_product`（只读文档 artifact，绝不读 `.expected.rs`）与 `manual_guard_scope`（每次调用独占 `<plan-scope>/<prefix>-<pid>-<nonce>`，退役固定 `auto_a2r_599_gate/capstone`）；M01 0.07s、M02 2.22s（stub 编译 + 4 程序真实运行，witness 逐字保留）、M03 1.12s（产品与独立 oracle 共 rlib、stdout 逐字节相等）。原 test 名/`#[ignore]`/内部子输入全保。

**控（常驻）**：NC-13/14 内嵌 `plan1066_compile_gate_classification_and_receipt_controls`（0.16s，纯函数级：外依赖+他错=unexpected、ledger 变绿=obsolete、伪造 compiled/缺 exit 各拒、陈旧 toolchain/ledger/schema 各拒）；NC-15 `ExplicitCompileGate` 2 测（门必须 `#[ignore]` 且理由点名显式组；组不进 daily/all）；NC-27 同测内（4 份具名 fixture 冻结 SHA-256、6 份 doc 产物与 sidecar 逐字节同、M03 产品≠oracle）。

**A 冻结增量**：`p_compile.gate_ignored` false→true（显式组迁移的合法登记，其余 p_compile 字段不变）。

**门**：`cargo t a2r` PASS 432/0（56.3s）；`cargo t vm corpus` PASS 1137/0（46.1s、child_validation valid）；`cargo t infra-fixed` 72 测仅既有 ace-facade 红（D-1066-INFRA-FIXED-ACE-FACADE），本包新增 2 测通过；`cargo test -p auto-lang --lib plan1066` 16 passed/3 ignored。

**余量（具名）**：NC-26 的 v2 库存/trace 半边——`execution_trace::legacy_files` 仍以 `<stem>.at` 等 sidecar 路径做 `file_identity`（a2r/a2c/a2ts 三个 wrapper 与静态 inventory 共用），需与 T-08 的库存/trace 文档化切换同轮改绑并重签 `legacy-wrapper-contract`。故 T-06 暂不勾。

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 6 | from: work | to: work | status: executing | outcome: partial（T-06 主体 5dbfb6a15：250 门 159/55/36/0/0、M 三 guard 全绿、NC-13/14/15/27；余 NC-26 半边） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: 5dbfb6a15 | next: work（T-08 v2 库存/trace 文档化 → 并闭 NC-26 → 回勾 T-06）

### 2026-11-17 B 期 work（round 7）：T-08 身份切片交付 + NC-26/NC-19 关闭 → T-06 回勾

stage: work | plan_id: PLAN-1066 | plan_revision: r3 | outcome: partial（T-08 身份切片完成；T-06 关闭 4/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | branch: plan-1066-dev | code_commits: eecef0878（trace/association）+ ccd4a3d48（inventory/NC-19） | dependency_revisions: 无跨仓 | task_ids: T-08（身份切片）、T-06（回勾） | next_responsible_role: work（T-07 proposal/旁路 + T-08 余量）

**NC-26（v2 身份/库存，已闭）**：`execution_trace::legacy_files` → `legacy_document_identity`（单一派生：`source` = 文档 hash、`document:{path}` = 文档路径；不再按 `.at`/`.expected.*` 求 file_identity）；a2r/a2c/a2ts 三个 wrapper 与 `inventory_wrappers` 静态守卫同调该函数，故运行期 trace 与静态绑定结构上不可漂移；`legacy-wrapper-contract.json` 569 条 legacy-trace 关联行改签文档（其余 389 条 vm-cache 与全部非 legacy 行零改动）。**库存侧**：`legacy_inventory` 四个族的行改报文档身份（`source`/`oracles` 均指向文档、`source_path` 只报文档声明的虚拟路径、新增 `document_checks`），1060 行实测"source 不是文档" = 0；被删文档显示 `present:false` 而非缩集/回退。

**NC-19 常驻控（0.30s）**：镜像（含 crate 布局以便静态守卫解析）证明——doc-only 仍被清点为文档身份；污染 `.at`/`.expected.rs` 后身份逐字段不变（证明零 sidecar 读取）；删文档后行数不变且 `present:false`。

**门**：`cargo t a2r` PASS 432/0（69.3s，child_validation valid）；`cargo t vm corpus` PASS 1137/0（31.7s，valid）；`cargo test -p auto-lang --lib plan1066` 17 passed/3 ignored（1.73s）。

**T-06 回勾依据**：其唯一余量（NC-26 治理半边）由本轮关闭；NC-13/14/15/27 已常驻；AC-05 主体（250 门 prepare/verify 159/55/36/0/0 + M 三 guard 文档产物 + 显式组）在 5dbfb6a15 实证。

**T-08 余量（具名）**：①`input_inventory.verify_inputs` 仍从 `.expected.*` sidecar 重算 VM oracle——该函数**当前无调用者**（grep 仅定义），属未接线的冻结交叉核；接回或退役由其 owner 在 T-11 前决定（记 debt 候选）。②`input-contract.json` 的 `legacy_ids`/`vm_cache_ids` 宇宙与 `baseline` 标签仍按旧身份命名（标签不是 IO，未改；如需改签另立）。③§5.2 的 v2/历史 v1 分派本身未变（仍文档式 v2 为默认、历史 v1 仅显式查询）。

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 7 | from: work | to: work | status: executing | outcome: partial（T-08 身份切片 eecef0878+ccd4a3d48；NC-26/NC-19 闭；T-06 回勾 → 13任务 4/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: ccd4a3d48 | next: work（T-07 C 侧 proposal/旁路退出 → T-08 余量 → T-09 全路径重扫）

### 2026-11-17 B 期 work（round 8）：T-07 关闭——显式 proposal + 全旁路退出（13任务 5/13）

stage: work | plan_id: PLAN-1066 | plan_revision: r3 | outcome: pass（T-07 整项关闭） | worktree: D:/autostack/.wt/lang-1066/auto-lang | branch: plan-1066-dev | code_commits: 773d01438 + 72b449829 | dependency_revisions: 无跨仓 | task_ids: T-07 | next_responsible_role: work（T-08 余量 / T-09 全路径重扫）

**显式 conformance-proposal（唯一作者入口）**：`conformance_proposal`（`#[ignore]` + 需 `AUTO_TEST_1066_PROPOSAL_OUT`）→ `admit_proposal_scope` 只接受显式、非仓内（含祖先 canonical 别名检测）、不存在或空目录的 scope；逐例 `CANDIDATE <case> -> <path>`，任一失败即整体失败（不许部分成功冒充完整）。实测 **30 written / 0 failed**；`git status crates/auto-lang/test/a2r/conformance` 零变化；30 份候选与已提交文档内联 oracle **逐字节相同**（说明生成与冻结期望一致，不是自证）。

**闭集对齐（发现并纠正）**：初版把 30 例清单按过时注释抄成 `012_float_ops..027_fibonacci`（16 例不存在），实跑暴露 14/30 后按 A 分母 `c_bootstrap` 重排；常驻控断言 `PROPOSAL_CLOSED_SET == fixture.c_bootstrap` 逐例相等，故两侧任一改动都会被具名拒绝（这正是 NC-17 要求的"不能只数 30"）。

**旁路全清（NC-25）**：a2r、a2c（`.wrong.c`/`.wrong.h` 两轴）、a2ts、aavm runner（两轴）、ffi_dual、vm-ast、plan724、plan727 的 `.wrong.*` 全部改 `diag_scope::write_scoped`（默认丢弃），与 A 已转的 conformance/cookbook/vm/entry 四族合起来，普通成功与失败路径都不再在作者根创建文件；新增静态守卫 `plan1066_b_no_author_tree_writes_in_corpus_wrappers` 扫 10 个 wrapper 模块的写调用，并带**注入旁路负控**（构造一条 `test/…` 写必须被检出），避免守卫空转。

**门**：`cargo t corpus` PASS 240/0（21.0s，33 conformance 验证与 trim 比较不变）；`cargo t a2r` PASS 432/0（44.6s）；`cargo t vm corpus` PASS 1138/0（38.2s，bootstrap 与 proposal 均在 excluded(ignored) 名单）；`cargo test -p auto-lang --lib plan1066` 19 passed/3 ignored（1.69s）。

**外仓边界（记 debt）**：`book_listing_tests::generate_book_expected` 仍是未 ignore 的生成器并写 `book/` 仓（非本仓作者根，本计划边界明确不转 book）；登记 D-1066-BOOK-GENERATOR，owner=book listing 线。

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 8 | from: work | to: work | status: executing | outcome: pass（T-07 关闭：773d01438+72b449829；proposal 30/30=文档 oracle、旁路全清+静态负控；13任务 5/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: 72b449829 | next: work（T-08 余量 → T-09 全路径重扫 → T-10 27NC → T-11 并集）

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 9 | from: work | to: work | status: executing | outcome: pass（T-08 关闭 670f5aa7f；T-09 首片 d0c7fb1a1；13任务 6/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: d0c7fb1a1 | next: work（T-09 全路径重扫余量 → T-10 27NC → T-11 并集）

### 2026-11-17 B 期 work（round 9）：T-08 关闭（VM 库存/verifier 文档化 + 活跃门）+ T-09 首片

stage: work | plan_id: PLAN-1066 | plan_revision: r3 | outcome: partial（T-08 关闭；T-09 首片；13任务 6/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | branch: plan-1066-dev | code_commits: 670f5aa7f（T-08）+ d0c7fb1a1（T-09 首片） | dependency_revisions: 无跨仓 | task_ids: T-08、T-09（首片） | next_responsible_role: work

**T-08 收口（670f5aa7f）**：VM cache 库存行改报**文档身份**——`source_path`=文档、`document_sha256`=文档字节 hash，同时保留 `source_sha256`=被执行源文本 hash（运行时 trace 的绑定轴），两轴分明避免拿文档 hash 冒充运行时身份；库存测试改为断言「缓存源 == 文档源」并**不再打开 `.at/.as` sidecar**；`input_inventory.verify_inputs` 由「无调用者的 sidecar 重算」改为**按文档重算**（`_document_channel` 解码 vm-stdout/vm-result/final-newline、error 存在性）并**接入 `source_bindings` 成为活跃门**（每轮 wrapper 组都校验），`input_paths` 收敛为「文档 = 唯一 input edge」。实测首跑暴露 `source_sha256` 混轴（51 例 binding differs）→ 拆轴后 `cargo t vm` 950/0、`vm corpus` 1138/0、`a2r` 432/0、`corpus` 240/0、`auto-test-case` 151/0 全绿；债务 D-1066-VERIFY-INPUTS-SIDECAR 关闭。

**T-09 首片（d0c7fb1a1）**：全路径读审计（scope 探针）扫出仍写作者树的转译器内嵌测试——`trans/{gdscript,javascript,python}.rs` 的 `.wrong.gd/.js/.py` 都被改写为 `diag_scope::write_scoped`，且顺带消除 `../auto-lang/...` 这种**跨 crate 相对路径**（原写法依赖 CWD，属隐患）；`conformance_tests::generate_expected`（已退役作者辅助）改读迁移文档而非 `input.at`，普通路径不再有 sidecar 读回。

**本轮教训（记录）**：尝试"删除退役生成器"时用行号/花括号启发式批量删除，误删了 10 个 conformance 验证用例（C 33→23）并使文件语法损坏；已用 `git checkout` 完整回滚，改用**最小安全切片**（保留函数、只改数据源）。后续删除类改动一律按 git 快照+精确文本锚点，不用基于行号范围的批量删除。

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 9 | from: work | to: work | status: executing | outcome: pass（T-08 关闭：VM 库存/verifier 文档化并接入活跃门；T-09 首片：a2gd/a2j/a2p 旁路 scope + 退役生成器读文档；13任务 6/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: d0c7fb1a1 | next: work（T-09 余量 → T-10 27 NC → T-11）


stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 10 | from: work | to: work | status: executing | outcome: pass（T-09 关闭：a05d30079；13任务 7/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: a05d30079 | next: work（T-10 27 NC 逐项 → T-11 冻结并集）

### 2026-11-17 B 期 work（round 10）：T-09 关闭——全路径重扫与分类读写审计（13任务 7/13）

stage: work | plan_id: PLAN-1066 | plan_revision: r3 | outcome: pass（T-09 整项关闭） | worktree: D:/autostack/.wt/lang-1066/auto-lang | branch: plan-1066-dev | code_commit: a05d30079 | dependency_revisions: 无跨仓 | task_ids: T-09 | next_responsible_role: work（T-10）

**新增收口**：plan724/plan727 四个 N 条目（`30_plan724/*`、`31_plan727/*`）是 a2r 发现集成员却仍直接读 `.at`/`.expected.rs`——本轮改为读文档（`legacy_doc::load_case` + `artifact(".rs")`），plan724 8/8、plan727 11/11 绿；其文档载荷与 sidecar 的字节等价由 T-03 的 1060/663/556 载荷证明覆盖（同族同条目）。首次 plan727 跑在 HTTP e2e 上出现 1 红、隔离重放 11/11 绿，属并发/网络环境归因（非本改动；载荷等价已独立证明）。

**分类审计（常驻 0.37s）**：`plan1066_b_remaining_sidecar_readers_are_named` 遍历全 `crates/` 的 sidecar 读点（`.expected.*`/`expected_output.txt`/`input.at` + 读调用窗口），要求每处落在**具名 allowlist**：migration 比较工具的静态旧侧库存、NC-27 的 doc↔sidecar 对拍控、AAVM 实验语料（按用户裁定仅手动重放）、外仓 book、五根外 ffi_dep 家族、sidecar 合同合成 fixture、a2r-actor-tests 外消费者；未列出者按文件:行拒绝，且带"注入读点必须被模式命中/未列模块不得放行"的两个负控，避免审计空转。这落实了"不泛豁免 source-dir"：豁免是逐条具名而非按目录通配。

**门**：`cargo t corpus` PASS 240/0（18.7s）、`cargo t a2r` PASS 432/0（41.0s）、`cargo t vm corpus` PASS 1138/0（45.0s）、`cargo test -p auto-lang --lib plan1066` 20 passed/3 ignored（1.70s）。

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 10 | from: work | to: work | status: executing | outcome: pass（T-09 关闭：HTTP 包装改读文档 + 具名分类审计 + 两个负控；13任务 7/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: a05d30079 | next: work（T-10 27 个具名负控逐项 → T-11 并集）

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 11 | from: work | to: work | status: executing | outcome: pass（T-10 关闭：e434b74a4；13任务 8/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: e434b74a4 | next: work（T-11 冻结并集/完整组计时）

### 2026-11-17 B 期 work（round 11）：T-10 关闭——27 个具名负控齐备 + NC-20 报告纪律（13任务 8/13）

stage: work | plan_id: PLAN-1066 | plan_revision: r3 | outcome: pass（T-10 整项关闭） | worktree: D:/autostack/.wt/lang-1066/auto-lang | branch: plan-1066-dev | code_commit: e434b74a4 | dependency_revisions: 无跨仓 | task_ids: T-10 | next_responsible_role: work（T-11）

**本轮新增（NC-20）**：`Plan1066Control { nc, owner, verdict, ms }` + `validate_control_report`——报告必须恰好覆盖 27 个具名控制（缺行/多行/重复逐条拒绝）、每行有实测 `ms` 且 `<10000`、verdict 不得为 `NOT_RUN`、owner 必填；并拒绝「父 PASS 掩盖失败/NOT_RUN/未计时子项」。合法 27 行报告通过，六类缺陷各具名拒绝（10.000s 超预算、缺计时、NOT_RUN、重复、缺行、父掩盖），常驻 114.1ms。`PLAN1066_CONTROL_OWNERS` 常量把 NC-01..27 逐条映射到真实控制（测试断言唯一且 27 项齐全），供复审逐条核。

**实测台账（同轮 nextest libtest-json，21 个进程内控制全 PASS）**：A-DEN 1909.3ms、NC-A1 1801.7ms、工程控 1497.0ms、doc-only 四族镜像 1013.0ms、族词汇 899.0ms、NC-A4 790.9ms、编译收据控 621.1ms、库存身份 598.3ms、sidecar 审计 578.2ms、NC-A3 498.1ms、E78 467.2ms、bootstrap 378.8ms、脚本/多源 374.6ms、NC-A2 225.0ms、双通道 native 195.6ms、污染 sidecar 194.9ms、header 轴 138.2ms、控制纪律 114.1ms、零写入守卫 117.2ms、proposal 32.0ms、删文档硬错 27.6ms——最大 1.91s，全部远低于 10s。外进程控制：scheduler 五轴组 6 测 5.14s（单测 ≤1.5s）、扫描 2 测 0.068s、显式组治理 2 测 0.013s。

**门**：`cargo t corpus` 240/0、`cargo t plan1066-s-doc-controls` 6/0、`cargo test -p auto-lang --lib plan1066` 21 passed/3 ignored（1.68s）。

**提交卫生记录**：本轮 commit 657ddeb39 意外把**另一会话**在共享主检出中已 stage 的 `docs/plans/KNOWN-DEBT-AND-RISKS.md`（1038 归档后的 F04/F07/F09/F11/F13/F15 研究决定更新）一并提交。内容为合法文档工作、未改动其语义，未回滚（避免破坏他会话在途工作）；此处具名记录归属，供 T-12 复审/T-13 merge 识别。后续提交前一律先看 `git status --porcelain` 的暂存区，避免再次夹带。

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 11 | from: work | to: work | status: executing | outcome: pass（T-10 关闭：27 控齐备 + NC-20 纪律 + 实测台账；13任务 8/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: e434b74a4 | next: work（T-11 冻结代码后完整影响组并集与计时汇总）

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 12 | from: work | to: work | status: executing | outcome: pass（T-11 关闭：6682a6d0a；13任务 9/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: 6682a6d0a | next: review（T-12 独立 final review）

### 2026-11-17 B 期 work（round 12）：T-11 关闭——冻结并集门 + 健康面收口（13任务 9/13）

stage: work | plan_id: PLAN-1066 | plan_revision: r3 | outcome: pass（T-11 整项关闭） | worktree: D:/autostack/.wt/lang-1066/auto-lang | branch: plan-1066-dev | code_commit: 6682a6d0a | dependency_revisions: 无跨仓 | task_ids: T-11 | next_responsible_role: review（T-12）

**冻结并集（一次调度）**：冻结提交 `6682a6d0a`，source_state `da5c9599dcf61a08`、config `0bc07b11a19a2ad2`；`cargo t corpus a2r vm auto-test-case infra-fixed plan1066-s-doc-controls case-ace-m2-scheduler-968` → **gate PASS**，5 invocation 全 PASS，executed **1810**（1570/151/72/6/11），wall **155.2s**。选组依据=真实 diff 触面（core 四模块+test_cases、auto-lang 十个 corpus wrapper、auto-test-case 解码/合同、治理脚本与固定组登记、S 组文档与工程文档），非工作日全仓翻底。

**健康面**：①格式——以 base `7c131286d` 为基线逐文件 `rustfmt --check`：11 个 base-clean 文件被本轮改脏，已全部格式化，**newly-unformatted = 0**；7 个 base 即非 clean 的文件（a2r/a2c/a2ts/tests.rs/legacy_doc/author.rs/inventory.rs）保持原状，未借机大改他人代码。②告警——`cargo check -p auto-lang --lib --tests` 在 plan1066/entry/test_runner/legacy_doc 上零新增 warning。③调试残留——新增行无 `dbg!/todo!/unimplemented!/FIXME`；`eprintln!` 仅出现在 4 个显式工具（payload proof/family actuals/compile prepare/proposal）作进度汇报。

**R3 更正（2026-11-17 复审）**：上文称零新增 warning 在冻结提交时并不成立——复审实测 4 条本轮引入的告警（`conformance_tests.rs` 未用 import、`plan1066_b_slice.rs` 未用 `root`、`legacy_doc.rs` 死 `has_check`、`plan1066_a_slice.rs` 死 `parse_project_assertions`）。四条已在同轮修复（`cargo check -p auto-lang --lib --tests` 无匹配告警），此更正保留原陈述以便复核对照。

**跨计划更正（具名）**：`test_ace_set_is_explicit_and_separate_from_daily` 自 PLAN-969 `278b91204` 起与 toml（4 成员，含 `ace-facade`）不一致、基面即红；本轮同步断言为登记集合，`infra-fixed` 72/72 转绿。属事实同步，不放宽该测试"显式且不进 daily"的意图；记录供 T-12 复核与 969 线知悉。

stage: goal | plan_id: PLAN-1066 | plan_revision: r3 | round: 12 | from: work | to: work | status: executing | outcome: pass（T-11 关闭：冻结并集 1810 executed/0 红、155.2s；健康面收口；13任务 9/13） | worktree: D:/autostack/.wt/lang-1066/auto-lang | commit: 6682a6d0a | next: review（T-12 独立 final review：AC-01..09 + 27 NC + SD）

### 2026-11-17 T-12[B] 独立 final review：needs_fix（AC-01/07 记录面与健康面须收口，功能面复核通过）

stage: review | review_scope: final | reviewer: 独立上下文代理（review-1066；与实施会话分离，工作树内只读重建，未看实施者中间产物） | plan_id: PLAN-1066 | plan_revision: r3 | outcome: **needs_fix** | worktree: D:/autostack/.wt/lang-1066/auto-lang（分支 plan-1066-dev，工作树 clean） | reviewed_commit: 6682a6d0a | base_commit: 7c131286d | dependency_revisions: 无跨仓 | spec_inputs: 本 Plan §5/§6/§7/§8；canonical docs/specs/auto-test-case/{document-migration-closure,architecture,document-cases,governance-and-inventory,module-selection}.md 当前 HEAD 文本（未改）

**acceptance_results（逐 AC；命令为我本轮实跑，均 @6682a6d0a）**
| AC | 判定 | 证据（命令 + 观测数字） |
|---|---|---|
| AC-01 | **partial** | 键集/分母**已交付但与本 Plan 文本不一致**：fixture `plan1066_a_obligations.json` n_rows **1060**（vm506/a2r250/a2c118/a2ts85/cookbook101）、e78 **70**；文件系统核 `*.case.ad` 五根共 **1130** = 1060+70，交集/未分类/幻影均 0（自写 denominator_check.py）。但 §7 AC-01/§5.2/附录N 仍写 1050/VM496/1170 角色/E78 78，附录N 只有 1050 行；实际角色总数 **1180**（1170 + 10 个 vm-smoke）。A 已交付切片（master 7c131286d = IF-1066-A-CORRECTNESS）的冻结分母在本轮 B 被改（n_rows 1050→1060、e78 78→70、p_compile.gate_ignored false→true），无 plan_revision 递增/无 AC 文本修订 → R1 |
| AC-02 | pass | 我自建外部探针（plan-artifacts/plan-1066/review-1066/mirror-probe，OS temp 镜像）：四族 doc-only（删全部 `.at/.as/.expected.*`）判定与仓库逐字相同（vm/a2r/a2c PASS，a2ts 同一条 `transpiler output mismatch`）；污染 sidecar 不变；改 doc oracle → 具名轴红（vm `stdout mismatch`／a2r `transpiler output mismatch`／a2c `C header mismatch`）；删 doc 保 sidecar → `migration document: …hello.case.ad missing/unreadable (sidecar fallback is forbidden)`；N0028/N0030 doc 实含 `vm-result` 且 `final-newline:false`，字节与冻结 `succ`(73 75 63 63)/`10`(31 30) 相同；载荷独立比对 1060/1060 源、663/663 artifact、556/556 check oracle 与冻结 sidecar 逐字节相同（0 差异） |
| AC-03 | pass | 我用真 auto-case 双跑镜像（自写 s11_equivalence.py）：旧 fence `scheduler.case.ad` 与 `scheduler.document.case.ad` 各 11/11 PASS，case/check/status/detail 全同，identity.source/oracle 22/22 相同，时长 381–771ms；`scheduler.suite.ad` 只 include 文档；NC-08 五轴/NC-09 扫描常驻（见下） |
| AC-04 | pass | 两份 project 文档 19 条有限断言（16+3；`exists:true`/`contains` 展开后逐条）；`test_14_modules_005_multi_file` 1.896s、`test_16_interop_021_path_dep` 1.978s PASS；NC-10a/b/c、NC-11、NC-12 常驻实测（doc-only 镜像可建、缺闭包硬失败、改 db 正文命中 `db.rs: struct Connection`、错/多断言具名、`../`/绝对/重复解码期拒绝） |
| AC-05 | pass | 我实跑显式门 `AUTO_TEST_1066_COMPILE_OUT=…/review-1066/a2r-compile cargo test -p auto-lang --lib a2r_rustc_real_compile_gate -- --ignored` → **159 compiled / 55 external-dep-skip / 36 known-broken / 0 obsolete / 0 unexpected = 250 行**，15.70s，收据落我的 scope，工作树 `git status` 仍空；M01/M02/M03 ignored 重放 0.05/0.89/0.46s 均 PASS；日常 `cargo t a2r` 的 excluded 名单实名含 `a2r_rustc_real_compile_gate`（无日常 rustc） |
| AC-06 | pass | 显式 proposal 我实跑 30 written / 0 failed（我自己的 scope），仓内 `git status` 零变化；union 门 excluded 实名含 `conformance_proposal` 与 `conformance_bootstrap`；33 项 conformance 验证仍在 corpus 选择内；NC-16/17/18/25 常驻 + NC-A3 动态（改 conformance doc fence → 真 consumer panic；GENERATE_EXPECTED 候选只落 scope；三根作者树 hash 前后相同） |
| AC-07 | **partial** | doc 权威在发现/执行/库存三处独立复核通过（探针四族 + doc-only `legacy_inventory` rows=1 source=…hello.case.ad）；`document_case_active_scan.py` 实跑 76 组/183 文件 document 183/fence 0/unknown 0；但“零活跃旧载荷”的审计声称强于实现：审计 needle 只含五族后缀，`trans/gdscript.rs:1848-1850`、`trans/javascript.rs:755-757`、`trans/tscn.rs:542-550` 仍是活跃 `a2gd`/`a2js` 组的 `../auto-lang/...expected.gd|js|tscn` 回读，未被审计命中且本 Plan 未具名为边界 → R4 |
| AC-08 | pass | 新增/改动例全部 ≪10s：最慢 `plan1066_a_denominator_integrity` 4.164s、NC-A1 4.130s、NC-A4 2.448s、NC-A3 2.162s、NC-A2 2.037s；两 project 测试 1.896/1.978s；python 控 6 测整组 7.5s（均摊 ≤1.5s）；scheduler 11 例 266–771ms；1 次并集 debug 调度，无 release/WSL |
| AC-09 | fail（owner=review） | 本复核即其内容；在 R1–R4 关闭前不给 pass；R5 需 T-13 按真实结果落地 |

**findings（须在 work 收口；均带证据与关闭条件）**
| id | 严重度 | 事实 | 关闭条件 |
|---|---|---|---|
| F-1066B-R1 | medium | 分母/文本漂移：A 冻结契约在 B 被改（fixture n_rows 1050→1060、e78 78→70、`p_compile.gate_ignored` False→True、p_projects.assertions 重派生），无 revision 递增；§7 AC-01 仍 1050/1170、§5.2 仍 VM496/E78 78、附录N 仍 1050 行（实测 1060/70、角色 1180） | 递增 plan_revision 并在 AC-01/§5.2/附录N/E 写入增量（10 个 doc-only VM + 2 份 project 文档）与 1180 角色；或恢复 1050 目录选择。T-13 只能用 1060/70/1180 数值落 Spec |
| F-1066B-R2 | medium | §5.2「cache 另 506 项中 10 个无 oracle/旧发现排除项**不能因有 doc 被擅自升级为目录验证**」被反做：这 10 项已入目录分母（1050→1060）并作为 `vm-smoke` 执行；其中 `test/vm/08_strings/017_sub_semantics` 仍存 13 字节 `017_sub_semantics.expected.result`（hex `65 6c 0a 65 6c 6c 0a 65 6c 0a 0a 6f 0a`）而 doc 未承载、无任何判据，T-03 收据却把这 10 项统一描述为「只有文档内 oracle」 | 要么按原 oracle 补 `vm-result` 并判红/判绿，要么具名登记为排除项（写进 §5.2 与债），并改正收据描述 |
| F-1066B-R3 | medium-low | T-11 健康面收据称「plan1066/entry/test_runner/legacy_doc 零新增 warning」，实测 `cargo check -p auto-lang --lib --tests` 有 **4 条本轮引入**的告警：`conformance_tests.rs:18` 未用导入、`plan1066_b_slice.rs:482` 未用变量 `root`、`legacy_doc.rs:96` 死方法 `has_check`、`plan1066_a_slice.rs:778` 死函数 `parse_project_assertions`（T-05 改为读文档后失调用）。AGENTS §3 健康门为零告警 | 清除或具名 allow 并说明理由，重跑 check，改 §9/T-11 记录 |
| F-1066B-R4 | low-medium | `plan1066_b_remaining_sidecar_readers_are_named` 的 needle 只覆盖五族后缀（`.expected.out/result/error/rs/c/h/ts`、`input.at`、`expected_output.txt`），±2 行窗口 + **文件级** allowlist；实际活跃 `a2gd`/`a2js` 组经 `../auto-lang/test/a2gd|a2j|tscn` 读旧 expected 未被命中且 Plan 未具名；AC-07「当前全入口零活跃旧载荷」因此过宽 | 在 Plan/债项具名这些族为边界（或扩审计 needle/路径构造并改行级 allowlist），并把 AC-07 措辞限定到五根+本包触面 |
| F-1066B-R5 | low-medium | SD-01..05 未落地：主检出仅存 4 份**未提交**的 1066 规划期 Spec 改动，且现已自相矛盾（`docs/specs/auto-test-case/document-migration-closure.md:19`「76组中75组…scheduler 仍旧围栏」、`module-selection.md:20`「1066…仍仅规划」）。若原样提交即落假知识 | T-13 必须改写并提交这 5 份：document-migration-closure（76 组/183 文件全 document、1060/70/1180、scheduler 已切）、architecture（共享 doc 视图/发现/四 shim/受管物化/显式 prepare）、document-cases（scheduler recipe、project exists/contains 有限断言、源闭包/consumer 资格）、governance-and-inventory（doc 身份 inventory/trace、v2/历史 v1、verify_inputs 活跃门）、module-selection（显式 `a2r-compile` 与 `plan1066-s-doc-controls` 两组登记、ace 4 成员）；并核对 supersedes 5 文件/空 new_spec_components |
| F-1066B-R6 | info | `PLAN1066_CONTROL_OWNERS` 测试部分只验证表自身（27 唯一/无缺）；`plan1066_b_family_artifact_vocabulary_is_not_crossed`（NC-07 owner）只对现网文档做不变量断言、无跨族注入；NC-15/17/27 属配置/常量断言（与各自合同相符）。逐条核 NC-01..27 **未发现空转/纯常量冒充**：其余 24 项均有真实变异或真实数据不变量 + 合法邻例 | 可选：给 NC-07 增加一条跨族注入负例；非阻塞 |

**NC-01..NC-27 逐条核对（映射常量 + 代码 + 实跑）**：27 项均有真实控制；常驻 21 项在 `cargo test -p auto-lang --lib plan1066` 全绿（21 passed/3 ignored/2.23s）；外进程 4 项：`case plan1066-s-doc-controls` 6/0、`infra-fixed` NC-09 2 测 + NC-15 2 测（本轮并集 72/72 绿）、scheduler 11/11。NC-08 五轴各自具名红且不泄漏邻例；NC-12/23/24 用镜像/键集变异；NC-13/14/26/27 用收据/原始 exit-stderr 重判与 fixture 冻结 SHA。

**边界诚实性核对**：a2ts 85/85 红（我实跑 85 items/85 failed，全 `transpiler output mismatch`；trans/typescript emit 本轮未改 + doc artifact 与冻结 `.expected.ts` 逐字节同 → 既有漂移，非本 Plan 引入）；a2c 8 红（118 项，逐条具名消息与收据一致）；a2r 351 项 1 红 `a2r/16_interop/024_nested_async_await`；shim CWD 挂起：我首次误把整族在进程内跑同样 >10min 挂住（旁证该边界），单例/镜像路径正常；外仓 book `generate_book_expected`（union 中 6.028s 实跑）仍在日常组写 book 仓=已登记 D-1066-BOOK-GENERATOR；PLAN-969 ace-facade 更正为事实同步（基面 toml 已 4 成员、断言 3 成员，`git show 7c131286d` 证基面即红）。

**worktree/build discipline**：`pwsh -NoProfile -File D:/autostack/wt-guard.ps1 D:/autostack/.wt/lang-1066/auto-lang` → clean（无 reparse point）；分支 plan-1066-dev HEAD=6682a6d0a、工作树 clean；主检出 `crates/ scripts/ Cargo.toml .cargo` 无本 Plan WIP（仅 4 份 docs/specs 规划期改动见 R5）。rustfmt 逐文件：25 个改动 .rs 中仍非 clean 的 7 个在 base 即非 clean（entry.rs/tests.rs/a2c/a2r/a2ts/legacy_doc/author.rs），newly-unformatted=0（与收据一致；收据把 inventory.rs 列为 base-dirty，实际已格式化）。

**计时**：本会话并集 `cargo t corpus a2r vm auto-test-case infra-fixed plan1066-s-doc-controls case-ace-m2-scheduler-968` @6682a6d0a = **6/6 PASS，executed 1810（1570/151/72/6/11），excluded 495，wall 170.5s**（比收据 155.2s 慢，同机他计划 cargo/ace-semantics 并发）；准备/编译均该次调用内（case-parent 0.99s、av reuse）。新增/改动例无 >10s。

**next**：T-12 不给 pass → 回 work 关闭 R1–R4（R5 归 T-13 merge 落地，R6 可选）；Plan 保持 executing/9-13，T-12/T-13 框不勾；工作树/分支/scope 保留。不得以 checked/收据替代上述实测。

### 2026-11-17 合同修订 r3 → r4（复审 F-1066B-R1/R2 关闭）

`plan_revision` 由 3 升为 **4**：B 期把冻结分母从 1050 改为 **1060**（T-03 登记的 10 个 doc-only VM 项）、
E78 由 78 改为 **70**（T-05 两份 project 文档进入边界）、并在 R2 修复中把 `017_sub_semantics` 的
13 字节冻结 stdout 角色补进文档（角色总数 **1171**）。这是合同数字的实质变更（非簿记），故按规范升版并
逐处更正 AC-01/§5.2/附录N；原 1050 行快照与全部原 oracle/条件不减。

## 10. 待澄清事项

四类补漏范围/原oracle/副作用边界已明确，无需用户先决定日常与显式编译的常规实现细节。T-02有界确定共享资格表达、public structs兼容与唯一proposal入口（不得保旧fallback）；真实不可行性在本Plan具名修约，不拆后继/不缩水。1061环境与外仓owner不阻Windows迁移，NOT_RUN不冒通过。原语义/编译债务保持独立，不用两侧同FAIL充功能绿。
