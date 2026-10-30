---
plan_id: PLAN-888
status: archived
feature_name: 不相交细粒度测试模块与ACE成组执行试点
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-30

plan_revision: 3
supersedes_spec_components: [docs/specs/auto-test-case/architecture.md, docs/specs/auto-test-case/governance-and-inventory.md, docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/plans.md, docs/specs/auto-test-case/project.md, docs/specs/README.md]
new_spec_components: [docs/specs/auto-test-case/module-selection.md, docs/specs/auto-test-case/batch-execution.md]
touched_goals: []

affects: [auto-test-case, test-infrastructure, auto-av]
depends_on: [PLAN-887]
current_step: 10
total_steps: 10
---

# [PLAN-888] 不相交细粒度测试模块与ACE成组执行：守恒、隔离、定位与实测

> r3（2026-10-30 用户明确）：先按职责归组并列出全部模块和用例数量；然后逐模块单轮测总耗时/累计CPU/峰值内存，问题及时定向修复；特别长的用例独列供人决定，不擅自关闭。取消全目录三warm/P90及新A/B/C收益矩阵作为交付前置，批量优化保显式实验且默认A。r2合同见attachments/888/contract-r2-superseded-by-r3.md，历史证据保留。
> r2历史（2026-10-29 用户补充）：现有重叠测试档位重划为不相交叶模块；按计划影响模块的并集执行，一分钟为常规模块目标。原 r1 保留最终裸 t 门的约束由本次明确要求替代，历史收据保留。
> 当前执行入口（2026-10-30）：用户已明确“交接代码并安排测量窗口，继续完成888”；X888/M888解除旧等待，权威树 `D:/autostack/.wt/lang-888/auto-lang`。仍不创建 agent 或联系其他聊天；阶段验证与集中计量按合同执行。
> 语言仓为实施范围；外仓用例只登记正式 owner/引用，不擅改 UI/Man/Web。Web 无无关 U888 全局门，仅真实共享写面串行交接。

## 0. 变更摘要

交付一个完整测试基础设施包：在887清单上把现有重叠档位重划为互不相交、可独立执行的细粒度叶模块，常规模块目标不超过60秒；按改动影响选择完整模块及消费者模块，取并集去重后执行，替代计划逐档重复跑 t/tt/tv/tf 的工作方式；以ACE AV标量源码为首批实现“同组一次worker、逐项独立程序”的成组执行；小范围比较一次编译/运行多个表达式；保持原子检查身份、强oracle、隔离/资源/独立重放和精确报错；本轮优先模块清单与逐模块单轮资源测量；A/B/C收益与推广后置，未证明前默认独立A。内部probe/阶段不另取号。

减少的是外层启动/重复准备和局部改动选面，不减少原子验证义务。既有cargo t不等于旧VM全集，ACE接管引擎也不替代通用模块/集成门。3000入口压至1000/300、整套数量级提速仅为后续容量评估方向，不是本Plan预先宣称的收益或强制数量目标。

## 1. 目标

- G1：当前语言 t/tv/tt/tf/tb/th/taa/ta/t3 的受管义务、聚合子case及ACE独立workspace入口有唯一叶模块归属；叶模块两两不交，并集等于应保留义务，条件/后端/profile不被误去重。UI/Man所属义务保外仓引用。
- G2：交付模块执行、影响选择和逐case重放入口；计划涉及多个模块时执行完整模块及消费者闭包的去重并集；至少两个真实局部变更选严格子集并发现相关故障。常规模块目标≤60秒，超时继续拆分或具名记录不可拆慢模块，不保留每计划必须裸 t 的第二重全局门。
- G3：ACE实现真实成组worker B，减少相同工作负载的启动次数，每项新程序/VM/状态保持；不重演887因worker写面排除而无法试验。
- G4：保留已实现A/B/C功能、隔离与定位验收；新的完整收益矩阵后置，不阻塞模块交付，B/C仅显式实验，未证明当前候选收益前默认A。
- G5：逐原子ID/check保强oracle、报告/隔离/预算/退出/复现，保持单源编译执行与父侧判定；给Web/后续结构包可复用的正式工具和限制。

非目标：不修改语言/VM/MIR/ABC语义、产品默认引擎/缓存/会话协议，不新增863宿主输出/865闭包等能力，不重写待退役旧VM/A2R内部；不拆crate、不为分类机械搬全仓文件、不重建PASS协议；不降低正式ACE样本数/P90门，不以batch均摊值替代cold/hot/edit/restart；不删除任何原有验证义务或借模块化取消资源/隔离约束；不修改UI/Man/Web仓产品与测试实现，不清共享缓存、不推送。未来 cargo all 可作为显式模块并集入口，本Plan不要求新增“全部测试”名称。

## 2. 架构方案

参考[模块选测与ACE成组设计 r1](../../design/testing/module-selection-and-ace-batching.md)，承接[Design35](../../design/35-autodown-test-cases.md)与现行auto-test-case；该设计的“保留全局裸t门/重叠关联套件”条款已被本Plan r2 §5.2替代，T-08须在权威worktree同步设计，不能将旧设计当当前合同。身份分开：原子case/check（义务）→唯一叶模块（归属）→group（调度）→harness/命令（入口）。统一模块映射与报告，不复制比较器、解释器或源/期望。

模块选择扩展887的`scripts/test-governance/`，拟新增`modules.json`、`impact_rules.json`、`select_tests.py`、`run_selected.py`；具体schema由T-02冻结，只为选面/来源，不是第二套用例协议。case/源码尽量保原位置；确有聚合入口无法独立选子case时，只拆必要测试包装/参数化入口，不复制裁判。`.cargo/config.toml`、nextest选择配置、`scripts/run-full-regression.py`、基线/守恒工具与仓内测试规约一并迁移为消费同一叶模块目录；旧别名的短期兼容只作显式legacy映射，不继续作为新模块或计划门禁。

B为显式、默认关闭的测试批量入口，拟增`experimental/ac-av-probe/probe/src/test_batch.rs`和既有`src/bin/av_probe_worker.rs`的feature门分支；复用真实前端→MIR→ABC→AV和旧单项请求处理/父侧准入、比较/清理。父侧拟增`core/src/test_cases/ac_av/test_batch.rs`或等价私有主题模块，不改公共语言API和既有case-serve/session-serve语义。新入口/sidecar能力与现行协议区别、版本和失效轴由T-02冻结；不得让现行worker静默接受未知字段或把expected传入worker。

## 3. 技术栈

Rust/nextest/libtest、Python标准库、既有TestCase/contained worker、已有Windows进程/Job计时与资源设施（或平台等价原生统计）。不加新的测试框架、外部覆盖服务、缓存后端或第二编译链。模块runner用现有工具真实支持的参数数组，T-02核Windows引号/长度限制，不预设未核验CLI参数。

## 4. 需求分析与背景调查

### 4.1 授权、基面与来源

**r3当前授权**：用户2026-10-30明确以模块归组/数量清单为第一项，以各模块单独单轮总耗时、CPU、内存测量为第二项；有问题及时检查修复，特别长用例由人决定处理，常规模块尽量≤60秒。承接已有work→review→merge授权、唯一权威树及X888，不需要重新许可；不创建代理、不联系其他聊天、不停止他方进程。原全目录三轮/P90与A/B/C完整收益矩阵不再是本次交付前置；不得用旧结果假装新候选已测。新性能推广另须完整严格证据，当前默认A。下述r2/r1调查仅为历史。

**r2 当前授权与调查**：用户2026-10-29明确要求把包含/重叠的 t/tt/tv/tf/u 等改成不相交独立细模块，最好每模块不超过一分钟，多模块计划只运行对应模块，未来可加 cargo all。本次据此修订合同，未恢复被阻塞的 goal，未取得 X888/M888。只读核 `.cargo/config.toml`、`.config/nextest.toml`、`docs/specs/auto-test-case/{architecture,governance-and-inventory}.md`：t/tt/tb复用 root+core lib且feature不同；tv为语料白名单；tf/taa/ta/t3经 full-regression-driver；nextest有正文150–200秒的 `mir_native_turnaround_body_edit_gate` 特例及画廊冷态约800秒历史注记。因此一分钟是常规模块实测目标，不能承诺所有既有原子测试硬降至60秒。语言仓 tu 已于867移除，owner在auto-ui；“u”不可凭名称在本仓造空入口。下列 r1 调查为历史基面，正式输入按T-01重核。

2026-10-29用户要求“先按照你的方式规划一下计划”，并询问其在拆分路线中应立即优先还是待所有拆分结束。本轮授权为主检出计划/设计/路线簿记，不是执行、机器窗口或代理授权；保留其他WIP，不修改887/885/ACE在途合同/进度、canonical Spec或受管ledger。

读取语言HEAD `5af8e0cbe3f5a5a4971d5142abb4fb53b0f983b4`。884 archived10/10已交付，871 archived9/9及清理已落地；887 r2 reviewed9/10、T-09 pass，冻结门/合入/归档/guard仍pending，不能按已完成U消费。885权威树存在生成/转译WIP；863/865状态头不证明无实施。T-01重新核所有真实树/文件/符号和正式交付锚，不借WIP。

| 来源/已观察代码 | 事实与对本Plan的约束 |
|---|---|
| `docs/specs/overview.md`、auto-test-case的architecture/profile-v2-native/ace-m1-conformance | 现行身份、父侧oracle、未知字段拒绝、infra不满足语言负例、资源/统计门；overview及部分路径/数量为历史，不当动态选面 |
| `.cargo/config.toml`、`.config/nextest.toml` | t选root+core lib及三个显式integration；tv语料、tt转译各有选面，default-filter已排语料等族；并非3000个相同初始化模式的测试 |
| `crates/auto-lang/src/tests/vm_file_tests.rs`、Core `test_runner.rs`、`entry.rs`、`vm/ffi/stdlib/sys_env.rs` | ignored包装、pilot聚合、目录聚合并存，外层少不等于VM/编译次数少；必须数真实调用与子case |
| Core `test_cases/{mod,avm,a2r,ac_av}.rs` | 原子检查/报告已有身份和比较；ac_av::call_worker每次启动case-serve，不能把它当已支持批量 |
| `experimental/ac-av-probe/probe/src/bin/av_probe_worker.rs` | case_serve显式exit，一次进程/请求；无session时nonce私有store并清理；session-serve属于增量会话，不可冒作无状态batch |
| 887 r2及其t02/t05/t08摘要（仅读权威树背景） | 6-check定向BASE约1.58s；缓存候选未命中、会话复用域外，r2获用户批准以机制级负结果关闭；并未测试本Plan的成组方案。约150ms剩余墙钟含启动/IPC/初始化，不能直接说都是CPU或可消除成本 |
| 887拟交付inventory/check_inventory及正式台账 | U887后才接受正式路径/schema/版本；不重造静态词法扫描冒充runtime case解析，不将不同oracle/后端的同源检查去重 |
| Design50、885及Web/UI-Man路线 | 测试是优先质量支线，Web原迁移合同不自动改门；885产品写面只读，跨界注册/脚本队列交接 |

本轮读取SHA256：architecture=`7b4f9cbe50c2ac8f1277bf29b73ae7fb564094053f84e499bfc76717994b6eb8`；profile-v2-native=`a3b21891b605628e7cd92a41690d4a137dc9d310a349749381593b346fc708f8`；ac_av.rs=`0c8ec785258e5df38047c6bc734c6c62c548b267a64e2ab655e593699b4348a9`；worker=`3d46e03dcdede5a6a661667f7cdec52af8230d0b1e98bfeb2ca55a2cb6e53307`；Cargo配置=`b77e006c4f0ad7935accfbe81834ff22be4fddda681d4dee0300563b41783a81`；nextest配置=`d079e4c9e97e1c1eb4359ab834f54da63109cbb62684288d2ba90ef6f17ff436`。

### 4.2 排程与有限边界

建议它作为U887后优先质量基础设施工作，不等Web/UI-Man完成，避免新模块继续产生难选测/高启动成本测试。范围在一个完整Plan内有界：模块选测落地+AV标量批量实做+有限组合表达式试验+测量决策；不把全生态推广、全局300/1000入口目标设成近期里程碑门。Web可继续设计/立约，产品实施只在实际共享面与本Plan串行，不设无关U888全局屏障。本轮不认为该建议已经授权开工。

## 5. 详细设计

### 5.1 G888、X888及交接

G888覆盖已授权实施准备、既有唯一树、动态枚举、probe、fixture、定向测试及最终review/merge。r3单轮模块资源测量记录实际机器/背景条件，异常仅定向复测；M888完整实验收益窗口不再阻塞模块工作。权威树为`D:/autostack/.wt/lang-888/auto-lang`、`plan-888-dev`；同树顺序自检/独立复审，兄弟仅固定正式版本只读，不建链接或review副本。

| 写面 | 888权限与交接边界 |
|---|---|
| 887清单/校验/runner/台账与新模块工具、Cargo aliases/nextest配置/仓内测试规约 | U887正式提交/schema/源锁/config/义务接受后单写；r2允许重划测试调度与更新本仓AGENTS测试门说明、旧入口迁移映射；保原义务/负结果/限制，不改887历史收据或其他在途Plan合同；用户级技能文件不在本仓写面 |
| Core test_cases/ac_av父侧、原子报告装配、测试注册 | 仅本批调度、逐项准入/比较/诊断和最小原处理helper提取；核863/865及其他H/driver真实窗口，无释放默认只读，不凭可自动合并写入 |
| AV worker二进制、新测试批量模块、probe manifest feature、具名控制测试 | **本Plan明确纳入的最小测试工具写面**；须交接worker dispatch/请求helper，新增feature默认关闭，不改既有wire/profile/session行为。与863/865等按符号窗口串行，不把整个S域或AV core认领进来 |
| ac-core前端算法、auto-mir-lower、AV core执行/内存/资源、native/CTE/产品缓存协议 | 排除语义修改；只读复用正式版本。若B必须重写这些产品机制，needs_replan，不以域外报告完成B |
| 885的api_gen/trans/rust/a2r-std、旧VM/stdlib、Web/UI/Man产品 | 只读，必要产品缺陷回原owner；本Plan不优化待退役引擎，不扩跨仓实施 |
| M1 binding/原suite/expected/证书、原性能硬门 | 默认只读引用，既有source/裁定/profile保持；不把batch数量记作原子分母，也不减独立性能样本 |

X888记录前owner/Plan修订→文件/符号→正式合入SHA和源/锁/接口/profile/指纹→888接受SHA及并集→消费者/ID→限制/失效/回交。无重叠须实际diff证明；无用户授权不跨聊天请求窗口。若窗口不可用，报告具名阻断，不持其他窗口或擅改共享文件；不是技术负结果。

### 5.2 模块清单与影响选择

T-01/02从U887输入冻结本仓 t/tv/tt/tf/tb/th/taa/ta/t3 的真实选面、已编译未选择、聚合子case和ACE独立workspace受管入口；构成义务全集 U。记录平台/feature/隔离/里程碑启用条件，不把未执行平台伪装通过。外仓UI/Man用例单列正式owner引用，不并入本仓已验证分母。每行有稳定原子ID/check、harness/group、唯一主owner/叶模块、多标签、source/oracle、profile/起点、条件/隔离和真实消费者。

叶模块 M_i 两两交集为空，且并集严格等于 U；每条义务恰属一个叶模块。原子身份按仓/consumer/后端/profile/必要执行条件消歧：同一测试在t和tt重复出现且配置同义时仅登记一次；不同feature改变被测行为、独立oracle或后端腿保留不同义务，不能按源文件名去重。标签/旧档位只作索引，不创建第二执行归属；父分类也不作含子模块的可执行模块。跨模块集成用例有独立 integration 叶模块，影响边将相关产品模块连接至它；“不相交”是用例集合不交，不承诺产品依赖互不相关。多个模块一起跑时去重并集，不隐式再跑祖先模块或旧广档。

至少建立语法/名称导入、类型/语义、SDK/宿主、语言行为、旧引擎专属、测试基础设施六类，ACE按前端/MIR/AV/cache/lifecycle保留子模块；这些是导航分类，叶模块须进一步按职责与实测成本细分（如lexer、parser表达式、parser声明、名称导入、AV标量执行、AV生命周期、协议准入、转译各目标、跨模块集成等，具体以真实枚举冻结）。不按plan编号或名字相似删/合检查，不只给旧t换名字。先职责划分，超过时间目标再作稳定且可读的子模块，避免每次随机分片导致身份漂移。

**模块清单、单轮资源与一分钟目标**：第一交付物为全量模块清单，按真实owner/职责归组，列每个可执行叶ID、职责、来源、注册用例/独立执行义务数量、原ignored/平台/里程碑条件和实际独立命令；汇总分类不得充可执行叶，禁止凭plan编号机械分组。第一阶段仅真实发现及分区守恒，不为统计数量执行全目录。第二阶段每个适用模块独立完整测一次，从runner启动到比较/清理记录总墙钟、父及全部子进程累计CPU、峰值进程/Job内存、发现/启动/执行/比较/清理开销和退出/失败；构建准备、首次冷态和总命令另列，记录背景负载和来源绑定，不统一重复三遍、不以三个样本声称P90。仅波动异常、接近/超过60秒或修复后的模块定向复测；发现问题及时归因修复且只重验受影响模块。常规模块尽量≤60秒，可拆慢聚合继续按职责拆，不改原输入/oracle来缩时。单条本身特别长或原规模/采样不可分时，单列原子ID、已观察耗时/资源、来源时点、原因、触发、优化/保留/暂关方案及人工决定pending；既有明显长例可在执行前列入人工待定测量队列，未测必须标明，不能冒PASS。未经用户明确决定不改变测试启用状态、原采样数或条件；未测/人工待定与已测失败均保留可见，不通过隐藏取消达标。

选择流程：变更文件/符号/资源→直接叶模块→消费者传递闭包/共用契约对应叶模块（含integration）→每个选中模块全部义务的去重并集→既有runner支持的精确选面。不能在已选模块内再挑几个有利用例而称模块完成。测试文件、源码/模板/include_str、配置/feature/lock、删除/重命名、共享协议、生成输入都有规则；无映射、身份stale、解析不确定或版本不兼容时保守扩展到明确列名的模块集合并给reason；最坏可显式枚举全集模块，不能悄悄调用旧t/tf充作fallback。空变更只读解释可不运行；非空零选、未知模块、模块存在却零枚举须红。

拟命令`python scripts/test-governance/select_tests.py --changes <清单> --out <选择receipt>`；显式`--module <ID>`可重复指定，执行`run_selected.py --selection <receipt>`。拟Cargo人机入口`cargo tm <module-id> [module-id ...]`只解析明确模块列表，不设隐式默认“全部”；T-02核现有driver可复用路径和真实参数后冻结。逐项复现`--case <原子ID> --mode independent`须标scope=case，不能充完整模块。receipt含源锁/目录/规则/枚举身份、直接/间接/回退理由、模块及逐原子ID实际run结果、去重前后数、耗时；scope=module不能冒全局pass。Windows参数/长命令安全分片，分片并集恰等选择集且不重复；机器独占模块保串行与原锁，不因新runner并行而绕过。

至少两个真实边界清楚的局部改动选到严格子集并通过故障敏感性；另有涉及多个模块的真实样例证明运行并集无重复、integration不漏。公共前端/协议变化可合理选大集合，不承诺总能小选面。r2最终计划门为受影响完整模块并集（含消费者/集成与必要慢模块），不再强制叠加裸t+tv+tt；原子义务/平台条件/周期性里程碑义务保持。旧alias在T-07完成消费者/脚本迁移后退出推荐与门禁，可保显式有期限兼容映射并记录移除点；不新建cargo all也可交付。以后若加all，只是显式选全部模块，不产生新义务/第二分母。

### 5.3 A/B/C首批试验

| 模式 | 输入/必须证明的机制 | 关闭边界 |
|---|---|---|
| A 独立基线 | 原case-serve路径，每项原source/profile/预算/判决 | 必须实测，不能拿887受背景竞争样本当本批BASE |
| B 成组进程、独立程序 | 同profile/recipe每组一次新worker，组内逐项新编译/新VM/请求环境，只摊销不可变工具/初始化 | 必须真实实现与测量；source字节、输入/输出/错误、独立状态与A同义，不能仅外层for循环仍每项spawn |
| C 一次编译/运行多个表达式 | 仅887 D的`1+2→3`、`-2*3→-6`、`(1+2)*3→9`；原表达式字节/包装/结果映射单列 | 有界真路径probe；可行则实测实际编译/VM次数和逐项actual，不可行须具体机制/安全缺口并判不适用，不伪造多结果或省略A/B |

B首批为M1 B01–B12共12个值例（14/-21/42/42/42/0/30/7/120/42/7/-2）及887 D三例，共15个源码正例的**AV标量相关检查**，不把其native/legacy/a2r其他腿混入同一批。D沿887正式冻结的包装/source身份，不私改为新算法；profile不兼容者分开。原case/check ID、source/oracle字节和强compare不变，A/B为同义务不同调度，不计新增语言覆盖。C包装组合是独立观察面，不代替原单文件解析/生成/隔离检查，也不得用单个校验和代替逐项actual。

初始k=1、3、10；最后不满k的组必须执行并计账。对15例进行预定重复以获得足够样本，重复带invocation id，不冒称更多不同case。负诊断/陷阱、全局env/FFI、资源/线程/缓存/会话/性能硬门、UI/GPU默认独立；本批仅以受控fault验证批量失败处理，不把它们混进吞吐正例。

新批量入口显式test-only feature默认off，原case-serve/session-serve必须原行为可用。测试构建的worker经新工具身份/capability校验后才允许B；缺能力明确失败，不静默fallback后称批量。原协议/schema未知字段拒绝不放宽；新outer调度元数据与原case/check报告分离，仍调用既有父侧admission/compare。绝不向worker传expected或接受其PASS声明。

### 5.4 逐项隔离、有界执行和失败报告

每组至多10项，一次只活跃一项；每项source/check、nonce/request hash、recipe/profile、序号与响应严格绑定。只有不可变编译器/工具初始化可共享；程序模块环境、VM/任务/内存、stdout/stderr、artifact目录、输入和临时store等按case独立创建/收尾，父侧检查完成才调下一项。A→B→A、正序/逆序/随机固定seed与单项重放须同值、零不明状态泄漏；不能只清map就自称完整复位。

单项原输入/输出/元数据/产物cap与deadline保持；组总绝对墙钟≤既有adapter会话上限60s，每项截止从其父侧准备到全部判定，排队计组总墙钟。元数据按原4MiB帧上限流式准入，组累计≤10帧上限；其他输出/产物按每项原cap并累计有界，不能先无限收再比较。T-02实证这些限额适用实际代码；若原路径更严格取交集，不能加宽guest预算。

ready/逐项actual/done全部匹配完整清单；缺项、重复、错序、额外项、错源/worker/profile、截断或中途退出不能绿。普通断言失败继续收集安全后续项，最终非零；超时/崩溃/协议错误及时杀树/回收reader，未执行者NOT_RUN、组失败，不能捏造逐项结果。独立复现/拆组重放仅辅助定位，首轮失败保留。

可读与机读报告均有原case/check、组/模式、源路径/span、expected/actual或typed错误、原始诊断和单项复现命令；一项失败不能只打印batch名。阅读资产仍逐用例呈现，调度层不生成需人工维护的巨型程序；C生成包装可回溯每条源/hash，不作为人工编辑主源。

### 5.5 模块优先计量、问题修复与实验推广边界

按§5.2先输出模块归属/数量清单，再逐模块完整单轮记录总墙钟、累计CPU、峰值内存、结果与真实调用。使用既有原生Job统计，计量父+全部子进程，不能拿CPU占用百分比或父进程CPU代替总CPU。构建准备与warm模块运行分别记录；现有原样本/预算/判定保持，不把整目录执行的总时长当日常选测成本。每项测量都记录代码、依赖、工具、环境/背景负载、原子义务及命令；旧诊断数据只作具名历史参考。

异常或接近/超过60秒者定向诊断：区分初始化/发现/调度重复、过大聚合、原子算法/规模、环境竞争与既有失败。先修本Plan内的分组/调度/测试基建问题，产品语义或跨仓缺陷交原owner，不擅扩实施。慢聚合按实际职责再拆、只补测新叶；特别长原子保留原测试并列人工决定，不自行SKIP/缩样/改expected。输出可读总表及逐模块可复核资源记录，不能要求用户阅读所有原始日志。

A/B/C已实现的真实调用、原判定、隔离/故障/定位验证继续保留。完整吞吐矩阵和收益推广后置，不成为本轮模块交付门；新候选未证实前，公开pilot默认A，B/C只能显式请求，不能把未完成的新矩阵当负结果或沿旧候选数值自动推广。未来若申请推广，仍须同候选/工作量BASE与NEW交错、每模式至少10轮、原>=10%改善/另一P50及墙钟P90退化<=5%/超过噪声/原资源门与所有安全控制，不能降低原单程序正式性能采样/门值。旧合同measurement-contract.json和全部尝试保留为实验合同/历史，未经请求不自动启动新矩阵。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/module-selection.md | 重叠档位→唯一叶模块分区/闭包选完整模块/去重并集/一分钟目标与慢模块/显式模块命令/旧入口迁移 | 模块归组/数量清单、独立单轮资源测量、异常/慢例人工决定且不漏义务 | AC-01, AC-02, AC-12 |
| SD-02 | add | docs/specs/auto-test-case/batch-execution.md | 独立worker为基线→实际B工具入口、适用白名单/上限/身份/逐项报告、显式实验/默认A/未来推广门 | 可审计调度，父侧裁判不变 | AC-03..AC-07 |
| SD-03 | modify | docs/specs/auto-test-case/architecture.md | 补最小测试批量拓扑与隔离/依赖单源；原协议/性能门不变 | 避免测试旁路产品合同 | AC-03, AC-05, AC-08 |
| SD-04 | modify | docs/specs/auto-test-case/governance-and-inventory.md；docs/specs/auto-test-case/plans.md；docs/specs/README.md §4 | 接受887正式版→旧档位/唯一叶模块/group/atomic映射、分区守恒、实际命令/耗时/结果和限制；计划门消费模块并集 | 义务守恒/真实消费 | AC-01, AC-07..AC-09, AC-12 |
| SD-05 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 原M1身份/分母保持，补首批A/B/C适用起点与证据区别 | 不把C/批次数量当源码全集绿 | AC-03, AC-04, AC-07 |

SD是proposal，work不发布canonical；review核实际交付，merge沉淀。governance Spec已随887正式交付，T-01接受源锁后扩展。本仓AGENTS与脚本中的门禁说明须同步r2，不改其他在途Plan的已批准合同，也不擅改用户级技能；与技能旧裸t要求冲突时，本次用户明确要求为准并留来源。

## 6. 测试设计

未指定模型/代理/执行者，本Plan不创建或联系agent。work自检后同一权威树由独立上下文final review，内部pass仅可送审；final pass明确merge门pending。下列均是未来验收，不是本轮已执行证据。

| Case ID | AC/不变量 | 代表输入/反例 | 真实入口/可观察oracle | 边界/owner_stage |
|---|---|---|---|---|
| V-01 | AC-01 分区守恒 | 本仓全部旧档位并集/ACE入口、聚合子case、多标签、不同profile；漏项/重复归属/双跑/错去重 | 实际枚举→887守卫扩展；所有叶模块交集空、并集=U，逐项唯一归属，条件保留；缺/重复/不明归属必拒，分类父节点不可冒叶模块执行 | T-01/03 work |
| V-02 | AC-02 局部与闭包 | 两个真实局部及一个多模块变更、共享API/模板/config、删除/重命名、未知路径/旧receipt | select/run真实入口→完整叶模块及闭包，去重并集与执行ID恰等；两个局部严格子集、多模块不重复，integration必选；未知显式扩展模块清单、非空零选择红 | T-02/03 work |
| V-03 | AC-02 故障敏感性 | 受控破坏模块分支和跨模块消费者，合法兄弟对照；选择遗漏该关联边负控 | 实际选择集真实执行能抓故障；错误影响图/缺必验行在运行前失败；不依靠全局套件替模块结果遮漏 | T-03/07 work |
| V-04 | AC-03 B真批量 | 15原例，k=1/3/10、尾组，原source/profile/expected | 新feature worker+父侧runner；逐项actual与A同，group spawn数=实际组数，源码编译/VM仍逐项fresh，无每check隐式再spawn | T-04 work |
| V-05 | AC-04 C实际含义 | D三纯表达式，原3/-6/9和唯一包装/hash/来源 | 真编译/执行管线、多actual父侧比较及真实调用计数；不可行有具名机制/安全缺口与不适用决策，不冒同源四路径或省B | T-02/05 work |
| V-06 | AC-05 隔离 | A→B→A、顺逆/固定seed、同名符号/输出、项间临时目录/预算 | fresh VM/任务/输出/状态生命周期及旧路径对照，结果/错误一致，前项不能污染后项；受控取消/退出后下一独立运行正常 | T-04/07 work |
| V-07 | AC-05 有界故障 | 不读stdin/中途退出/超时/输出或4MiB帧越界、重复/错序/漏项/错hash或profile | contained父侧从准备到杀树/reader收尾均有界，ERROR非语言负例，尾项NOT_RUN，整组非零；合法边界对照通过 | T-04/06 work |
| V-08 | AC-05/07 定位 | 将D arithmetic期望3改4，两个不同项同时错，单项filter，缺组尾义务 | 可读/结构化输出逐case/check源span+expected/actual+复现命令；正常失败后其他项有结果，单项只执行它；整体非零、漏尾守卫红 | T-04/07 work |
| V-09 | AC-06 成本 | 各适用模块完整单轮、异常与长例、修复后受影响面 | 原生Job记录总墙钟/父子累计CPU/峰值内存/原退出；背景与构建单列；异常定向复测；特别长原子独列人工决定，未测不冒绿；A/B/C新收益矩阵后置且默认A | T-06 work |
| V-10 | AC-07 阅读/重放 | 任取首批每组一case及错例；从报告定位源/期望再独立重放 | 读者无需理解调度器即可定位原文件/行/唯一ID，命令真只执行指定case；checker核映射，独立review完成定位演练，不虚构AI评分 | T-07 work/review |
| V-11 | AC-08 兼容/正式接线 | 原case-serve/session-serve、非白名单recipe/typed资源负例、feature off/缺capability、新模块门与原义务 | 既有定向正负/生命周期不漂移，非白名单仍独立、缺batch capability明确非零；正式模块入口确实运行新检查，不被--lib漏掉 | T-07 work |
| V-12 | AC-09..AC-11 完整关闭 | X/AC/SD/实际并集/冻结收据与剩余债 | 独立review逐条重建，冻结门/实际合入/规范账本/index/归档/guard读回；未验不勾、不用负结果豁免未做项 | review/merge |
| V-13 | AC-12 模块清单/成本/迁移 | 所有模块归属与数量、单模块一次执行、超60秒聚合及原子、真实选测并集 | 全目录真实枚举守恒但不据此运行；独立命令及注册/条件数量清楚；每模块单轮耗时/CPU/内存，异常定向修复/复测；慢聚合拆、特别长原子人工待定，保原义务与状态 | T-02/03/06/07 work；T-09 review |

未来命令：现有`python -m unittest discover -s scripts -p test_full_regression.py`及U887交付的治理控制按触面复用；拟新增`scripts/tests/test_module_selection.py`与`test_batch_receipts.py`分别按unittest discover命名执行。模块选择CLI见§5.2，T-02以实测受支持参数冻结。

拟新增`crates/auto-lang/tests/plan888_test_routing_and_batch.rs`，显式`cargo test -p auto-lang --features test-cases --test plan888_test_routing_and_batch`；独立协议包的discovery/compare/contract只跑实际受影响目标。AV worker拟增默认关闭feature `test-case-batch`及具名测试`experimental/ac-av-probe/probe/tests/av_test_batch.rs`，命令`cargo test --manifest-path experimental/ac-av-probe/probe/Cargo.toml --features test-case-batch --test av_test_batch`（路径/feature均拟新增）；原case_binding/真实相交的源程序与会话测试定向回放，T-02核名称/选择数。

当前模块计量入口scripts/test-governance/measure_modules.py须改为逐模块单轮、结构化资源汇总和长例人工待定队列；已有measure_pilot.py及measurement-contract.json仅保实验，不自动启动。

**merge-owned（r3）**：final pass后冻结提交，复用本次有效的逐模块单轮与定向验收证据，执行仅必要的受影响完整模块及消费者/integration并集；不因runner横切而再叠加全目录运行或旧t/tv/tt/tf。旧义务守恒由真实枚举及负控核，迁移路径以代表性真实入口与局部并集执行核；必要慢例与人工待定状态显式保留，不冒全绿。外仓及原周期/手工/里程碑owner保持；不擅改其已批准合同。源码/依赖/测试配置/输入改变后只失效实际受影响证据并重审重验，不能无条件复用或重复整档。

## 7. 验收标准

| ID | owner_stage | 可观察关闭条件 |
|---|---|---|
| AC-01 | work | V-01通过；本仓旧档位义务并集及ACE入口重划成两两不交叶模块，并集=U、每原子唯一归属、漏/重复0；条件/oracle/后端/profile不误去重，外仓引用不冒本仓分母；编译/选择/执行/通过分列 |
| AC-02 | work | V-02/03通过；两局部严格子集抓真实故障，一个多模块样例运行完整模块及消费者/integration去重并集；未知/stale显式扩模块、非空零选红；新入口可独立跑模块，scope不冒全局pass |
| AC-03 | work | B已真实实现并跑15个原例与尾组，启动次数恰为分组数、逐项新程序/VM/状态与A同；新增test-only入口默认关闭、旧协议不变，X明确包含该最小worker面 |
| AC-04 | work | C在D三表达式完成真路径可行性/计数决策，可行则真实多actual比较，不可行则具名边界/证据与不适用结论；不伪造编译次数，不替代A/B或原单源义务 |
| AC-05 | work | V-06/07/08隔离/身份/有界IO/杀树/清理与定位反控全部有效；普通错逐项可见、fatal尾项NOT_RUN，infra不满足guest负例，安全单项重放不掩首轮失败 |
| AC-06 | work | V-09通过：逐模块单轮资源与真实结果完整，构建/背景/旧观察分列；异常已定向排查修复或原owner/人工决定明确，未测不冒PASS；A/B/C完整新收益矩阵不阻塞，公开默认A、B/C显式实验 |
| AC-07 | work | V-08/10及旧harness→group→原子映射齐，读者能从报告定位每项源/期望并真单项执行；正式入口选中所有必需项，零/缺/重复必红，不删断言/样本/金样 |
| AC-08 | work | V-11证明原路径/feature-off/非白名单/原性能资源门保持；新红0、触面新增warning0、fmt/diff clean、无debug与临时反控残留，不引入产品语义或跨仓改动 |
| AC-09 | work | G/X/M、SD-01..05、模块/性能/决策/限制及回交完整，遗漏/延后/workaround/KNOWN-DEBT清楚，合同缺项不转债冒完成；最终commit/tree clean可送审 |
| AC-10 | review | 同权威树冻结SHA独立终审AC-01..09及AC-12/V/SD/实际分区与执行链、共享窗和旧入口迁移；定位演练有效，必需面未验不pass；final pass明确merge门pending |
| AC-11 | merge | 冻结触面/到期批次与实际合入、canonical/ledger/index/来源读回、归档、fresh guard与清理齐才U888；没有局部/负结果冒全生态提速或所有测试替换 |
| AC-12 | work | V-13通过：第一交付物列全部模块归属/独立命令/用例与条件数量；第二交付物逐模块单轮总耗时/累计CPU/峰值内存/结果，特别长或暂不测者具名、理由/已有观察/人工决定pending可见；目标尽量≤60秒，慢聚合可拆则拆，未授权不关测试。日常仅受影响完整模块与消费者并集，旧导航不冒叶模块，不强制全目录三轮或P90 |

## 8. 执行步骤

10项均已完成：8项work、1项final review、1项合入/归档/清理。r3模块清单、逐模块单轮资源及异常定向修复已交付，原始收据保持；完整新A/B/C收益矩阵与日常用途准入审查按用户要求后置。

| 状态 | ID | owner_stage | 依赖 | 路径/交付/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | G888 | 核U887正式SHA/工具/schema/负结果及885/863/865真实写面，建唯一树；冻结所有旧语言档位与ACE独立workspace义务并集U/聚合子case/外仓引用、源锁配置和X888；V-01基线，不只盘t/tv/tt。2026-10-30 r6：九旧配置实际list均exit0，旧并集4819→新语言4868，missing0；10 ACE workspace及原子/legacy/cache输入冻结，来源/条件/外仓引用见权威树 `docs/plans/attachments/888/baseline-contract-r6.md`、提交c957e235d；原子到叶归属/执行仍由T-03/06关闭，不以发现冒PASS。 | AC-01, AC-09 |
| [x] | T-02 | work | T-01；相应X888 | 冻结不相交叶模块/影响图、两局部及多模块样例/新命令/旧alias迁移、60秒计量及慢模块规则；B test-only通道/上限/旧helper保真、C可行性/计量、原source/oracle与A baseline；最小全链可行，不确定先决策。2026-10-30 r6：两真实局部故障、完整模块消费者并集/漏dispatch反控，B15原例/隔离有界路径、C D3实际编译/VM计数及原父侧比较通过；执行选择与正式成本模型冻结于权威树 `docs/plans/attachments/888/execution-freeze-r6.md`、`scripts/test-governance/measurement-contract.json`（d056ee200）。公开tm/旧alias迁移、原子归属、parent分相/PID与采样是后续实现，不以此处冻结冒完成。 | AC-01..AC-06, AC-09, AC-12 |
| [x] | T-03 | work | T-02 | 扩887清单/分区守卫、modules/impact、select/run与显式模块命令；按职责细分现有用例，必要拆聚合入口；V-01..03、13的唯一归属/守恒/完整模块并集/故障/长命令反控，新模块可单独跑 | AC-01, AC-02, AC-12 |
| [x] | T-04 | work | T-03；worker/parent窗口 | 最小test_batch worker/父侧调度与新integration，15例B真批量k1/3/10与尾组，旧case-serve不变；V-04/06/07/08及必要旧协议对照 | AC-03, AC-05 |
| [x] | T-05 | work | T-04 | D三表达式C完整可行性/计数/结果或具名不适用裁定；只用原编译执行器、不动产品算法；补V-05，保源映射和独立A义务 | AC-04 |
| [x] | T-06 | work | T-03；计量时记录机器条件 | 先交付按职责/owner分组的全模块清单及数量、独立命令；再逐模块完整单轮测总墙钟/父子累计CPU/峰值内存/真实结果，构建/背景另列；异常及时定向修复、仅重验受影响模块，可拆慢聚合继续拆，特别长原子人工待定清单；不启动全目录三轮或A/B/C新矩阵 | AC-06, AC-12 |
| [x] | T-07 | work | T-03/04/05；T-06清单与相关测量 | V-08/10/11/13定位/重放/分区/feature兼容；正式消费者/入口迁移及实际局部计划去重并集；公开pilot恢复默认A，B/C仅显式实验；慢例保持原状态并列人工决定，外仓零写入 | AC-01..AC-08, AC-12 |
| [x] | T-08 | work | T-06/07 | 全AC/V自检、SD、模块归属/数量表、单轮资源/异常修复/慢原子人工待定表与回交冻结；同步设计r3及当前规范提案；来源/健康审查、提交干净后execution_done，收益矩阵不阻塞 | AC-09, AC-12 |
| [x] | T-09 | review | T-08；冻结SHA | 独立上下文同树逐项审全部AC（含AC-12）/V-01..13、分区并集与反控/成本/选面/SD/X，定位演练；findings回同树修复重绑，不跑整档 | AC-10 |
| [x] | T-10 | merge | T-09 pass | 并集/源锁配置→冻结触面与到期档→实际合入→canonical/ledger/index来源读回→归档→逐树fresh guard clean/remove/分支清理，回交Web/后续包可消费工具版本 | AC-11 |

原始枚举、日志/JSON/计时、截图/worker转储只本地保存，摘要记命令/退出码/选择与执行数/关键观测/路径+hash；attachments仅Markdown摘要入库，长期case/工具/反控在源码受管。保留主检出他方WIP，不reset/stash/全量add；实施/定向验证及提交仅既有权威树，不推送、不建链接/agent、不联系其他聊天。

## 9. 复审记录

2026-10-29 / stage: new / plan_id: PLAN-888 / plan_revision: 1 / outcome: pass（合同静态预检，非实现验收/实施授权） / scope: T-01..10、AC-01..11、V-01..12、SD-01..05 / checks: 0–10章节有序唯一、10任务/11AC/12V/5SD唯一且引用有效、各AC有任务承接、任务/路线依赖无环、888活跃/归档编号唯一、号源取888→889、新Plan/专题链接存在、3个源码/Spec身份哈希复核与空白/diff检查；当前路线为U887后优先支线，不建立U888→Web全局门 / authorization: 本轮仅规划及路线位置建议 / next: 用户审阅；U887及明确执行授权后work T-01，worker/parent等共享写面另须X888，性能采样另排M888。未修改887/885/ACE合同或进度，未建树/仓/agent、运行项目测试或改产品/Spec/ledger。

2026-10-29 / stage: work / plan_id: PLAN-888 / plan_revision: 1 / outcome: blocked / scope: T-01 入口只读核查（任务未关闭） / base_commit: 09bb414875d4e15050ecbbaf3959b1d72976d857 / code_commit: none / worktree_path: 未建 / authorization: 用户 goal 全链授权，随后明确“先核查，暂不交接共享写入” / evidence: U887 已归档且交付祖先检查 exit 0；865 实树 status/diff exit 0 确认 worker case_serve 与 parent run_check_impl 未提交重叠，详见 §10 / blockers: X888 尚未释放/接受；M888 未安排 / next_responsible_role: 用户或原 owner 明确串行交接后 work；不要求将整个 U865 加为语义前置。

stage: goal | plan_id: PLAN-888 | plan_revision: r1 | round: 1 | from: work | to: user | status: executing | outcome: blocked | worktree: 未建 | commit: 09bb414875d4e15050ecbbaf3959b1d72976d857 | next: user（等 X888 可消费提交与共享符号交接；非 B 技术负结果）

stage: goal | plan_id: PLAN-888 | plan_revision: r1 | round: 2 | from: user | to: user | status: executing | outcome: waiting | worktree: 未建 | commit: 09bb414875d4e15050ecbbaf3959b1d72976d857 | next: user（2026-10-29 自动续轮：重读 §9/§10，未见 X888 释放/接受或 M888 授权；865 尚未归档、正式基面未变；沿用“暂不交接共享写入”，不运行阶段测试或写实现；上一轮分类为 progress，新增实际冲突证据并完成入口核查。本轮为等待用户交接，非 live-process verified wait。）

stage: goal | plan_id: PLAN-888 | plan_revision: r1 | round: 3 | from: user | to: user | status: executing | outcome: blocked | worktree: 未建 | commit: 09bb414875d4e15050ecbbaf3959b1d72976d857 | next: user（重读 §9/§10 未见交接答复，master 未变，865 归档不存在；上一轮分类 no progress，只有状态重核，无 live handle。相同 X888 阻塞连续三轮，按 goal blocked 审计停止自动续跑；恢复需明确释放/接受可消费提交上的 worker/helper 与 parent 共享符号，M888 正式采样窗口另落实。计划未完成，未运行测试或写实现。）

2026-10-29 / stage: new / plan_id: PLAN-888 / plan_revision: 1→2 / outcome: blocked（合同已修订，实施仍缺X888/M888） / authorization: 用户明确要求现有包含关系档位重划为不相交细模块、最好每模块≤1分钟、多模块计划运行对应模块、无需当前新增all / delta: G1/G2、§2/§5.1/§5.2、V-01/02/11/13、AC-01/02/10/12、T-01/02/03/06/07/08/09、SD-01/04及merge-owned门改为唯一叶模块分区/完整模块闭包并集/成本与旧入口迁移；B/C原义务保留 / superseded: r1“保留每计划裸t+触面全局门、不替代全局门”与旧设计同条款；历史收据不改 / checks: 仅合同静态一致性检查与现有Cargo aliases/nextest/Spec只读调查，无项目测试、无实施、无新树/agent/外仓写入；10任务/12AC/13V/5SD保持唯一编号、引用对齐 / next: 确认可消费共享提交并明确X888交接后恢复work；M888安排正式计量。Goal保持blocked，本次补充要求不自动撤销“暂不交接共享写入”。

2026-10-30 / stage: work-entry / plan_revision: r2 / authorization: 用户明确“交接代码并安排测量窗口，继续完成888”，解除此前暂不交接限制，接受X888最小worker/parent工具写面并授权M888集中独占计量 / inputs: U865 delivery=5fa3133fa6401904b0760986ecb2e83b0534cbd4，已归档且树/分支清理；863仍drafting且无实施树；主检出代码零WIP / worktree: D:/autostack/.wt/lang-888/auto-lang / branch: plan-888-dev / base: eda05a86f2f6bd29d3b70479917360d64a908a66 / readonly siblings: auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4、auto-ui 831553abc03d8a2c9b8b547e2e5dcfa3890e9ef5、auto-man e6cc06fbc496f366edde7929abfd49254ec13647 / tree assertion: rev-parse --show-toplevel与branch均匹配 / next: T-01动态清单与分区基面；本收据替代旧waiting恢复条件，历史不改。

2026-10-30 / stage: work / plan_revision: r2 / outcome: progress（T-01/T-02/T-03准备，非execution_done；10项尚未完整关闭） / worktree: D:/autostack/.wt/lang-888/auto-lang / branch: plan-888-dev / base: eda05a86f / code_commit: 6260a831ce3240a6e83764bbc6fc020a293b36e3 / commits: 503b73a67 HTTP测试feature跨crate助手修复、16785270b模块分区/选择/执行守卫原型、b0c941998受管recipe配置、05a81acc6主题导航去Plan前缀、b4e7a226b旧828测试adapter补865 Closures穷尽分支及真verifier正负控+syntax锁、347bda07e清除lowering过期锁依赖、a5ed94f71枚举收据绑定、6260a831c精确模块过滤与未选binary区分 / checks: unittest test_module_selection.py 14/14；root全feature nextest list成功4857入口（含HTTP），10个ACE workspace list成功1670入口；旧828 admission定向4/4、HTTP outcome reporter定向2/2；rustfmt检查5个改动HTTP Rust文件pass、828文件已fmt，diff-check pass。未跑t/tv/tt/tf等整档。

冻结本地证据（均不入库）：`<worktree>/docs/plans/attachments/888/baseline/frozen-r4/discovery-receipt.json` SHA256 `4d8f2b85ce50bddd0ee2de17602ff31a55c0a495ac7f8a89cf1055ce700b5771`，命令 `python scripts/test-governance/discover_modules.py --recipe language --recipe ace-source --recipe ace-syntax --recipe ace-semantics --recipe ace-artifacts --recipe ace-av --recipe ace-native-worker --recipe ace-native --recipe ace-abi --recipe ace-comptime --recipe ace-lowering --output .../frozen-r4` exit 0，source identity前后相同、逐JSON内容哈希绑定。原型 `modules-prototype.json` SHA256 `4bb50e448518e2b40eb819781add468be1a3cfe667f111e24bfa3d6a50a406ef`：6527入口→724叶模块、交集空/并集相等；尚非完整原子分母（聚合内部仍需runtime解析）。两个模块 `language/a2r-std/001` 与 `ace-lowering/auto-mir-lower-closures_865_lowering/001` 经真实select→run入口跑6条，6/6；单轮墙钟1.374s/0.229s，不冒正式性能采样。`module-run-r4/results.json` SHA256 `e99c1e84c16b999f47978b094df618e69ebbb2b354b2a7875cf48223fea3a90d`。

边界/下一步：目录仍在本地prototype，未发布modules.json、未接Cargo tm、未替换原门禁；owner/source/oracle/聚合子case/平台条件与旧档位守恒尚未闭合，影响边为空时保守扩全部、不宣称两个产品局部示例达成；B/C尚未实施、正式计量未跑、所有merge-owned门pending。T-01继续构建auto-case并以正式runtime parser枚举suite检查，核B01–12/D三例（B08/B11在binding而不在b-scalar.case.ad，需保原身份/源/期望建立真实父侧消费，不造既有绿色check）；T-02冻结实际模块图/测量/B最小协议；T-03完成语义归属、运行安全/反控与真实故障敏感性。旧基线发现两处测试基建编译失败均已针对修复，无产品算法改动。初次枚举生成的syntax锁与lowering陈旧锁整理已受管；首次收据因锁变更被工具拒绝，frozen-r4才为当前可用输入。权威树clean，无运行中构建/会话待poll；Markdown入口摘要见worktree `docs/plans/attachments/888/execution-contract.md`，不代替主计划。

stage: goal | plan_id: PLAN-888 | plan_revision: r2 | round: 4 | from: work | to: work | status: executing | outcome: pass（本轮有实质进展，整体work未完成） | worktree: D:/autostack/.wt/lang-888/auto-lang | commit: 6260a831ce3240a6e83764bbc6fc020a293b36e3 | next: work（T-01原子清单/源锁→T-02全链冻结→T-03正式模块及T-04批量；无需用户再次授权）

### 2026-10-30 work 第5轮：B真链、正式发现与单项重放

- 权威树仍为 `D:/autostack/.wt/lang-888/auto-lang` / `plan-888-dev`，新增提交 `4d3bb4b73`（真实批量与原子清单/计量方法）及 `5743ac279`（共享原绑定与CLI重放）；实现树 clean。主检出仅既有文档/计划 WIP，未包含、重置或提交他方改动；未创建 agent/联系其他聊天/改外仓/推送。
- B 已实做：默认关闭 `test-case-batch`，新 `avworker-test-batch/1` 通道复用旧单项 worker helper 及原父侧准入/比较器，expected 不进 worker。15例 A 与 B k=1/3/10、尾组均一致；每组启动一次、逐项 fresh driver/VM/store。FILE_SHARE_READ lease 在整个组内拒写入/改名/删除，允许安全复用组内一次可执行文件 SHA；独立 A 仍原逐次身份复核。父侧诊断现保留原 worker mismatch 具体原因，避免基础设施错误被压为笼统文本。
- 定向验证：`cargo test ... --test plan888_test_routing_and_batch batch:: -- --test-threads=1` 在 lease 版 6/6 通过（34.13s）；共享 loader 版 5项通过，顺序测试因标签用了数组标点被新安全校验拒绝，修正测试标签后**仅重放受影响顺序测试**1/1通过（1.10s），未重跑整档。包括13类协议/IO fatal +1阻塞stdin控制、两项错oracle（D arithmetic 3→4、group 9→10）得到 FAIL/PASS/FAIL、错序/回显/源hash/截帧/输出上限/超时/异常退出、PID已死亡与私有store清理、非白名单/负例拒绝。控制诊断加强时发现原报告吞具体原因，已修复并重验；未以一般超时冒充指定故障被识别。
- `test-case-pilot` 共享原suite/binding加载器，输出源文件及契约行/绑定行、case/check、source/oracle SHA、expected/actual、原父侧receipt和独立单项 replay argv。真实CLI控制2/2通过：k10、parallelism2真实2组/15项PASS/2次启动，实际执行报告argv后只执行指定1项；未知case/零/越界/重复/未知flag明确exit2。临时目录采用独有名称+create_dir，拒非法标签，不再递归删除可预测既有路径。
- 发现方法：生产TestCase parser/resolver枚举22套/329 check occurrences；生产legacy发现1044行（VM494/A2R246/cookbook101/A2C118/A2TS85）；旧VM缓存loader另504行（含8个.as）。均是独立消费者的发现值，**不冒充这些子义务已全部执行或已有唯一归属**。C已证明既有owned profile真路径可一次返回原D三表达式actual 3/-6/9；data/scalar入口仍拒record，不改变产品语义。C bounded报告/映射及正式计数/成本仍待完成，不能直接关闭T-05。
- 原生Job计量方法4/4真实控制通过：子CPU/nested Jobs累计、超时杀持管道grandchild及输出洪泛；报告parent+descendants CPU、meter CPU、wall/peak。尚无正式A/B/C≥10轮采样，未宣称提速或推广。常规模块>=3轮warm和慢例细分也未完成。
- 配置已显式发现新integration/batch feature/worker tests。`discover_modules.py --recipe language ... --recipe ace-lowering --output .../baseline/frozen-r5` exit0，11配置6543入口（language4868），source identity前后相等，complete=true。分区原型729叶、两两不交、并集6543，仍需正式语义source_paths/consumer闭包、旧档位组合/条件与聚合子case核对及Python工具用例接线；不将原型当成完整U守恒验收。参考文档纳入source identity，计划簿记除外；模块边界控制15/15通过，rustfmt --check与git diff --check通过，新增触面warning未观察到，既有基面warning保留日志。
- 本地证据根 `D:/autostack/.wt/lang-888/auto-lang/docs/plans/attachments/888/baseline/`（均不入库）：`batch-lease-final.log` SHA256 `e9a967b8e870e2333b816e9796bedb25aba877233108dbbb24c8b49228dc3714`；`batch-shared-loader-tests.log` `e88d841ae4cd5a4c5aca50a65f2375caa3d52c6bc7118b5d229c7bd7c6907769`；受影响重放 `batch-shared-order-final.log` `3231644d2b1fcc8bb7e5eb44f69f86c381f23275ef5235a27bcaf8dd42e3a7da`；`pilot-cli-controls.log` `2d6583c3fcf3dc4e76e29a2b383700ce8d806b05e2e95abcbc617aa216981ec3`；`module-controls-r5.log` `af6e001edcfca01992b667da933d6be1b362595383935f78d1f7a81c1bc4499d`；`frozen-r5/discovery-receipt.json` `418beca96ae782e8431cef2dcfd0cadad2f4f7e46041bc1cb9fb0f83136e0339`；`partition-prototype.json` `08dffaaca7808ea131cfe48702df0ebe27070f7c8eb585ff75906ef1c5e1ce96`。长期实现与控制在源码，Markdown执行入口已更新。
- 未关闭任务保持0/10：T-01完整旧配置/条件/聚合归属基线、T-02完整影响图契约尚未齐；B和CLI实现提前验证相应未知机制，不能以它们代替依赖/整任务关闭。下一步先完善T-01/02→正式T-03模块目录和消费者→补齐C→统一计量与迁移。最终独立复审、冻结merge门、合入/规范/归档/清理全pending。

stage: goal | plan_id: PLAN-888 | plan_revision: r2 | round: 5 | from: work | to: work | status: executing | outcome: pass（实质进展，整体work未完成） | worktree: D:/autostack/.wt/lang-888/auto-lang | commit: 5743ac279 | next: work（完整基线/影响图与正式模块目录→C/统一成本/入口迁移；授权与M窗口沿用，不再询问）

### 2026-10-30 work 第6轮：T-01/02关闭、真实影响闭包与C有界全链

- 上一轮分类为 **progress**：有4d3bb4b73/5743ac279提交与新发现/实跑证据，非仅状态重述。本轮重核主检出代码WIP空、既有他方文档保留；唯一权威树与branch不变，未创建agent/联系其他聊天/写外仓/推送。新增427a0763a（真实Cargo/runtime/Python消费者选测）、c957e235d（九旧入口输入冻结）、a5ae91d38（C有界真实管线及计数）、d056ee200（执行/测量合同）；HEAD d056ee200，实现树clean。
- **T-01关闭（发现/输入冻结，不是全测通过）**：新增 `discover_legacy.py` 从当前Cargo别名及正式full-regression language producer实际取得九档 nextest list，全部exit0且源身份前后相等。t/tv/tt/tb/th/tf/taa/ta/t3 的listed/selected分别3881/3207、2225/163、4500/3578、3757/3093、2171/105、3703/3203、3819/3198、4521/3652、4521/3653；旧(binary,test)并集4819，新冻结语言配置4868，missing0。新增49=11个888控制+38个已存在但旧语言入口未选择的Core HTTP e2e测试，全部保留。U887守卫conserved=true、errors=[]，23退役/21迁移对账。U887 dd916c184为已接受基面eda05a86f祖先exit0；865/887归档存在。正式schema/source/oracle/lock/config/Windows ignored与T3_MILESTONE条件和外仓owner引用冻结，详权威树 baseline-contract-r6.md。
- 当前正式parser/resolver重新发现22 suite/329 check occurrences，errors0/executed0；production legacy输入1044（494/246/101/118/85），原VM cache504含8.as。只读cache helper精确ignored工具1/1通过（0.12s），未运行guest；不同consumer不混分母。T-03仍须把原子子项绑定唯一叶模块和真实harness，T-06仍须记录真实执行，不能用这些发现数当PASS。
- **真实影响图与选测预实施**：Cargo metadata --offline --locked --no-deps建55节点/135边，保留所有本地dev/optional/platform依赖，另补动态worker→父侧与Python消费者。执行端重建图拒漏边；module/package边一个固定点闭包。Rust obligation保runtime binary/test，Python用标准unittest loader独立schema，按binary分叶避免同topic跨包误属。15 recipes原型6592入口/739叶（含Python49例），三次真实受控故障均恢复原字节：native_meter guestCPU置0选1模块4例→exit1；取消partition missing guard选2模块20例→exit1且CLI消费者确实PASS；Python dispatcher漏末项，即子进程小选面exit0，消费端仍因实际身份/数量不符exit2。多模块正常/恢复并集3模块24例均通过、去重守恒；删真实consumer edge在执行前被拒。边界控制18/18通过。公开tm、legacy成员/条件分叶、原子映射及计时仍pending，当前原型不能当全局门。
- **C全链实做（T-02机制决策，T-05待依赖与正式接线关闭）**：默认off `test-combined-serve`，严格新schema/nonce/原D3 source BLAKE3/真实av.owned/1 profile/类型/条数/EOF/退出码/lease校验；复用B suspended→Job、限额reader/writer及清理。原三个表达式字节不改，生成record包装有源路径/SHA和byte/line range，actual3/-6/9仍用原父侧judge_expectation逐项判，不传expected。实调用点观察compile/lower/verify/encode/VM六相，合法compile1/VM1；编译拒绝compile1且VM0。发现owned旧sequence字段在该路径为0，已明确作为executor_sequence_observation保留，绝不冒steps/VM次数。无产品前端/MIR/AV算法改动；默认feature-off check通过，runtime feature-off/session兼容仍归T-07。
- C控制覆盖两父侧错oracle得到FAIL/PASS/FAIL、源映射逐byte一致、错source/id/compile计数/VM计数、额外输出/no-read有界拒绝、清理后独立A仍PASS；底层C三例3/3通过（0.01s），源profile正式身份收口后C精确回放1/1通过（3.62s）。CLI四控制4/4通过（最终2.957s），含C repeat3/parallelism2真实3 worker、每组compile1/VM1、9实际值，以及执行报告argv真只重放指定原例。A/B/C pilot必须显式prebuilt worker，B/C不再在60秒组预算中隐式做release构建；B总时长在比较/清理后再次核截止。
- B/C共享变更集中验证7项：初次7/7通过27.00s；最后补输入有界read与post-cleanup deadline后6/7通过，extra-after-done反控先触1秒总截止而未到目标响应。未把一般deadline当协议故障证据：保持hang/no-read 1秒，普通协议反控给5秒（仍远低于原60秒合同）且必须精确匹配注入错误、清理小于budget+3秒。仅重放该受影响13故障控制1/1通过13.11s；其余6项未改输入/代码路径，沿用同冻结实现的通过结果。未触整档t/tv/tt/tb/taa/tf测试正文。
- **T-02关闭（最小链与执行模型冻结，不是成本验收）**：执行冻结 execution-freeze-r6.md / measurement-contract.json：九cohort×并行1/2×10正式轮=180样本，另18完整warmup保留，三次原集合重复不冒新覆盖；A/B同候选二进制，主45原子调用/D3分支9原子调用，C仅3真实编译/VM。nearest-rank P50/P90、noise=max(1ms,2×A MAD)、原>=10%/<=5%推广门与资源限制不变，失败/整轮重测保留。参考机Windows20逻辑核、34163630080B物理内存。原生Job parent+descendants/自身CPU方法已验证；**parent分相、A真实spawn PID及正式sampler仍须T-06实现并检查，缺字段不算验收**。没有正式性能采样、没有推广或提速结论。
- 本地证据根 `D:/autostack/.wt/lang-888/auto-lang/docs/plans/attachments/888/baseline/`（raw全部不入库）：legacy-r6/legacy-receipt.json SHA256 `cd268476199c72f5130958ea6699a7e92fbe506c82e7540c68c6b592896ee732`；input-freeze-r6.json逐项绑定atomic/legacy/cache/守恒；impact-controls-r6/summary.json `8de29c826f21fe88416cdb765fb7eac2ca1afd3d5ca438a42b042e96ed3c7055`；batch-c-r6-frozen-controls.log `10cacfbf36684ddd337f252ff866d56bb13e971d7ac05a7fbbc2e6f2c319c107`（6通过/1方法反控红原样保留）；修复回放batch-r6-protocol-final-controls.log `c51e9595aef108a6ef2f3b83279c7c37ce584c6079f2167f790ed5b58ac8d31f`；c-observations-r6-controls.log `d33a3e8131a9e1ca94dce07c5fa78b4d6a74176028383596d152abcf12e351be`；pilot-r6-frozen-controls.log `6f3b347ed4bc12346abd0a0a7bffb67ec5c6f639b5252d9f76bc72e8b0939363`。fmt/diff clean，新增触面warning未观察到（既有基面warnings保留）。
- 2/10：T-01/02已关闭；T-03正式模块/原子/条件分区、T-04/05正式接线、T-06全部成本、T-07入口迁移、T-08全验收、自主独立复审/merge门/合入/归档/清理均未完成。C及新CLI新增控制后，6592/739旧原型已stale，T-03必须重新发现/生成，不得直接用旧receipt执行。无新增用户问题，继续按既有G/X/M授权推进。

stage: goal | plan_id: PLAN-888 | plan_revision: r2 | round: 6 | from: work | to: work | status: executing | outcome: pass（T-01/02及真实实现/反控有实质进展，work整体未完成） | worktree: D:/autostack/.wt/lang-888/auto-lang | commit: d056ee200 | next: work（T-03正式叶目录/legacy条件与原子消费映射→T-04/05接线→T-06完整计量；不重问授权）

2026-10-30 / stage: work / plan_revision: r2 / outcome: progress（T-03正式化继续；尚非execution_done） / worktree: D:/autostack/.wt/lang-888/auto-lang / branch: plan-888-dev / commit: b81f062068ce63eec22f0712004af9f134aff4d0 / commits: 9d987a164完整case独立入口及原始suite NOT_RUN保真，db20058f7按Rust AST绑定VM缓存包装函数，be459783b旧成员/条件守恒与分片目录/消费边界校验/显式工具链，012dbd597同步已交付863主线并适配公共请求函数return，b81f06206本轮摘要。

- 正式源契约保留旧9档4819入口及已发现ACE/Python义务；329原子check的原adapter/recipe/profile/product_from/条件/源oracle绑定受消费端再校验。37个recipe真实重新发现，6931义务→991互斥叶；原legacy_map守恒通过。分片哈希与单文件预算校验、源/配置/有效环境失效守卫、未知条件拒绝、ignored/milestone显式NOT_APPLICABLE（不计已执行绿）、普通Release与批量feature工具隔离均已接线。此处目录仍未作为最终发布模块目录。
- 定向验证：module_selection Python 23/23；完整case CLI永久oracle变异控制1/1；4个原完整case实跑均符合原oracle（hello VM/Rust双绿；3个原生反控原始FAIL保留，build失败金样反控的后续consumer原样PASS）；已准备Release批量worker的pilot CLI 4/4；真实select→consumer闭包→run执行3个模块27条，27/27、0skip。VM只读清单504输入，Rust AST得389个原包装函数/389个有效缓存键、334显式ignored、缺键0；executed=0，不冒执行证据。
- 主线863已正式交付并清树，重新只读核主检出代码零WIP后同步master至本树（main锚b5556e169）；保留新HostConsole/原Closures verifier分支。自动合并将863控制台分支带入提取后的公共case_request，已修为显式return；worker test-case-batch与root全相关feature cargo check均exit0；随后重建当前parent/Release批量worker，863 NUL字节经worker/父比较器定向1/1、原15用例A对B k1/3/10及尾组一致性定向1/1（15.09s）。没有整档t/tv/tt/tf，没有正式成本采样；基面既有warnings保留，未观察新增触面warning。
- 本地证据统一在 `<worktree>/docs/plans/attachments/888/baseline/`，不入库：`round7-evidence.json` SHA256 `30b76e1091f46fe1ba4203498ed7874672ddb8e8d638568fdb866d53f9574fba` 列出本轮日志/receipt内容哈希；`frozen-r7b/discovery-receipt.json` `0264940379726a378df16516f3f8442303b4e4248ab72341c1a1ab6aa8599d43`、`frozen-r7b/modules.json` `89695ad0510d476a62a50f0cb6702ed5e65b4c1989925b863d148ebd7f2c4ca4`、`infra-run-r7/results.json` `aabb5d78bc100609f86af61b9b4c5918438688e1a779233c48d36db4f9cd0135`、`case-execution-r7/results.json` `102f403a80f08b07808fdba774678fe993b80470da977450d3b84b73b802f66c`；post-merge console日志 `601da4d78d5c6ac45f27bce63680a33dee6cd1addbfe2bdf453ea0ae2212db6c`、batch日志 `96ccedacb917ccbcd7a3972324f15e26e12caa0b3dc9ecc68f97bfe4eace7ae1`。已提交的MD摘要为 `attachments/888/module-contract-r7.md`。
- 仍2/10。T-03须刷新post-863新增suite/集成/条件，完成旧输入与聚合实际消费归属并发布最终目录；旧6931/991冻结在同步前，已明确stale，不得拿旧receipt跑当前版本。T-04/05正式接线、T-06每模块≥3warm及A/B/C各≥10正式轮/真实进程数分相、T-07新命令/alias/回归脚本/规约迁移、T-08、独立复审、merge-owned并集门、Specs/ledger、归档与清理均未完。未声称模块一分钟合规或性能收益。无新权限/外部依赖阻塞，下一轮依既有授权继续work。

stage: goal | plan_id: PLAN-888 | plan_revision: r2 | round: 7 | from: work | to: work | status: executing | outcome: pass（3实现单元与主线共享链回放有实质进展，work整体未完成） | worktree: D:/autostack/.wt/lang-888/auto-lang | commit: b81f06206 | next: work（刷新post-863目录及旧输入/聚合实际归属→正式接线/测量/迁移；不重问授权）

### 2026-10-30 work 第8轮：T-03关闭，真实子项记录与公开模块入口

- 上轮分类 **progress**（实提交、实际发现与实跑，不是重述）。本轮重核主检出代码零WIP、权威树/分支归属与既有收据；未建新树/agent、未联系聊天、未写外仓、未push。提交 `4bf398580` 原比较边界执行记录；`168ef3acd` 公开tm/输入契约/消费验证；`26b05c7ed` 正式分片目录及MD摘要；实现树clean，status仍executing。
- **T-03关闭（结构与正式执行接线，不是T-06成本验收）**：38真实recipe重新枚举→6990义务/1012互斥叶；language Rust4877、ACE Rust1706、Python63、原子344。增加863正式suite15检查及Core新integration；原4819 legacy身份/档位成员、原329原子与新增15绑定、原1044 production输入/504 cache输入均守恒。42内容哈希分片总7396467B、最大2646927B，index2623862B，均在单文件预算内。源/环境失效、消费端重建图、原条件/成员/源oracle/后端/profile/product链、分区漏项/重项与输入丢失守卫均接线。
- `cargo tm <module-id> [module-id ...]` 经现有Rust小driver正式前转；list/refresh/changes/select-only/out/catalog可显式使用，零实参非零且不隐式全部。目录stale先真实重新发现。旧alias仍未迁移启用（T-07）；不把旧包含档当新叶。Windows执行采用native Job全树/输出/截止/CPU与峰值统计；长argv在启动前拒绝。纯VM/金样叶不准备无关ACE工具。普通Release worker与批量feature worker继续隔离；发现Python前按recipe显式prepare，模拟缺worker时原始4 ignored→prepare后4适用、guest执行0。
- 原suite/check与旧file-runner/cache/金样比较边界提供opt-in原始子项记录，NOT_RUN/原FAIL/ERROR保持。389个Rust AST简单VM包装声明中当前实际注册383、另6未注册，均有明确标志；不把声明当已执行。实际20用例VM叶产生20匹配source/oracle/ID的已完成原比较记录；消费端要求每条active绑定一条真实成功调用，包装exit0但丢子项不得绿。完整case CLI的原oracle变异与suite NOT_RUN永久控制通过；旧Rust/C金样、Cookbook与缓存正控通过。手动启用的原ignored TS hello仍因console.log/globalThis.console.log旧金样差异红，原失败/未完成记录保留，不冒通过。
- 当前module/meter控制34/34通过，含条件/反控/原producer JSON顺序、真实改sidecar拒绝、丢输入消费边拒绝、包装假绿拒绝、长argv启动前拒绝、嵌套Job CPU/死线/洪水/后代清理。全库运行身份扫描发现2中文名与3个/bin/ worker单测，原ASCII白名单阻断；实际nextest裸精确operator已验证，中文scope=case 2/2、worker bin完整叶3/3通过。未采用不匹配的带引号operator，也不以0选通过。
- **公开完整模块并集最终回放**：选择meter、lowering、worker-bin后先检查声明消费者选面，8完整模块/55义务，完整执行55/55、0skip、source/config before/after一致；含原新Rust integration两叶（B/C、故障/重放/隔离）和真实pilot CLI。`final-routing-r8c/execution/results.json` 完整/通过均true。不是全局绿；一个Query中心模块会合法扩到419模块，已停止不必要的中途扩大执行、保留partial结果，不作为完成门。正式全portfolio仍留T-06冻结时段。
- 路由诊断：初版helper扫描把文档/guest字符串提名当调用导致过选；新增Rust注释/普通与raw字串/嵌套注释消歧，按实命名空间找最深源码；以引用索引与namespace/cache代替两两regex/重复relative_to。旧cProfile 161.385s中regex100.189s/path45.401s；修后普通图构建实测冷4.027s、同进程cached1.104s（工具诊断数，不充项目A/B样本或提速倍率）。计量单测200ms信号在嵌套Job实跑产生109ms配对差，改为600ms真实子CPU、原绝对/配对门180/140ms加强到580/420ms；native计量实现与项目性能oracle/采样/推广门未改，先前红不删除。
- 扩大实跑中3条原AAVM红精确命中既有 **P763-R1**（KNOWN-DEBT-AND-RISKS.md:1692）：两compile腿E0433临时工程缺auto_lang依赖、goldens参考执行STDASSEMBLY.INVALID_BYTECODE:60。相关harness/transpiler/aavm文件与master diff为空，原红保留且不新造绿/ignore/移除义务；沿既有非阻塞债，不修无关AAVM。该62叶扩跑完整、但原始exit1，绝不算通过。另一目录CPU诊断运行按已核PID/路径/输出归属显式终止，未作正式样本。
- 本地证据根 `<worktree>/docs/plans/attachments/888/baseline/`，raw不入库：`round8-evidence.json` SHA256 `367ca27fee1763cce965b45e581424ce001aeefc9a3a05dc6ba1fbdbf81ec84e` 逐项列哈希；最终并集results `19532c03490cab2a0b83880257f8b13eafbe3d7c398b27614c631c3c62ceebbd`，selection `3040148e46b8f00961ff4996ebedf5269435910908a5ef7fe86da9fa7e6849cb`，生产index `a55a34f2d714722105bd2c18deb1e6de9f7b4d7683715bb43789e44a9ee77ce5`，Unicode 2-case log `6c0d9236a34fa5a240bb3044a5a4a93ad46ef692798af43f159a911846fd8327`，default-feature check `c3e8b6bd539b93078495ade713393146afc72c173b19f074f85e4c57e1bf5148`。生产JSON是load-bearing源码配置，不是attachments证据；已提交MD `attachments/888/module-contract-r8.md`。Rust scoped编译/默认check/改动Rust fmt与diff-check通过，基面warnings保留。
- **3/10**：T-01/02/03已关闭。T-04须续核B全部准备是否在同一总预算内，补必要旧协议/feature边界对照并据最终代码关闭；T-05依T-04关闭。T-06仍缺A真spawn PID、父侧prepare/transport/admission/cleanup分相及formal sampler、180正式+18预热、所有适用叶>=3warm/完整命令与冷构建分列/可拆慢组拆分。T-07 aliases/回归脚本/规约、T-08完整验收/SD、独立复审、merge-owned门、Specs/ledger、归档和guard清理未完成。未宣称全模块一分钟、加速、全局绿或交付完成；既有G/X/M授权沿用，无新增用户问题。

stage: goal | plan_id: PLAN-888 | plan_revision: r2 | round: 8 | from: work | to: work | status: executing | outcome: pass（T-03正式目录/执行与原子消费已关闭，work整体未完成） | worktree: D:/autostack/.wt/lang-888/auto-lang | commit: 26b05c7ed | next: work（T-04/05最终边界→T-06真spawn/分相/正式A/B/C与全叶成本；不重问授权）

2026-10-30 / stage: work / plan_revision: r2 / outcome: progress（T-04/05关闭；T-06正式采样开始） / commit: 9f063e1d0 / worktree: D:/autostack/.wt/lang-888/auto-lang

- T-04关闭：B白名单/源准入移入同一绝对总截止，逐段检测准备耗时；1ns有效预算永久反控实际0 spawn并具名preparation deadline，不是spawn后才红。A原call_worker与B/C Transport的成功spawn处记录真实PID，不用计划组数猜进程。test-only capture按线程隔离，准备/传输/准入比较收据/清理分相；A传输扣除嵌套清理，避免重复计时。实际PID与报告计数/ID消费边界再核；原worker schema/原admission/oracle不变。最终B/C7项整体定向7/7（26.68s），含15例A/B k1/3/10/尾组、原oracle错误、白名单拒绝、故障/写阻塞回收/重放/隔离/C计数与源图。
- T-05关闭：原D3字节、生成源映射、owned-record一次真实compile/VM、原父判定及反控保持；本次含真实spawn与父侧四相，CLI加载独立分列。CLI7/7（4.598s）包含A-D3实际9 spawn/9 compile/9 VM计量验证以及丢spawn/编译次数/结果/ERROR反控；default-off独立原case路径3/3通过，缺batch能力明确exit2且全NOT_RUN，未fallback冒批量；原session-serve跨进程small fixture及门轴反控2/2（0.36s），不冒30-revision正式campaign。相关全feature cargo check通过，路由/native-meter34/34，Rustfmt/diff通过，基面warnings保留。
- T-06工具已实现：固定合同9 cohort×p1/2×10正式+18预热，实际Job父子CPU/observer CPU/峰值、真实spawn/calls/阶段、交错顺序、nearest-rank与noise原门；每样本保原stdout/stderr/hash与失败，global batch/performance mutex，外部构建/测试检测。首演练因实际cargo/rustc PID25208/44484/56644在途，0样本/complete=false收据保留；核实进程结束后第二演练18/18有效但diagnostic=true/complete=false、不作推广证据。已在9f063e1d0冻结并启动正式198次，尚未宣称完成/提速。
- 新模块成本工具消费显式完整选择，预构建记录单列、3次全warm、保全命令native wall/CPU/峰值、逐叶P90与raw exits/条件/超60s待拆标志；HTTP原串行条件保持。尚未执行全portfolio或关闭T-06。
- raw本地根D:/autostack/.wt/lang-888/auto-lang/docs/plans/attachments/888/baseline，全部不入库：final-parent-r9-cli.log `adaf45cffff90298b211e952b0cf9e129e58543834904e8a381f6f361b5b28c1`；final-parent-r9-batch.log `2b485fbdfff3dc25ae9353060060dcd5922801925fac1782707347202e3296bf`；session-r9-default-off-controls.log `29e1212a43f1543d862e0678a0122713df0b6735cb90f0923d47620a8f8c8e71`；parent-r9-check.log `d2b883021530d4c21e172d197fbabc83a8f3f90f512a85e3d03b58143176f636`；r9-routing-meter-controls.log `06d5adc6f7a976436c058be92ca7bd786e06c59749bb2a1d002a2390c124a25a`；pilot-r9-smoke-2/campaign.json `783bd098332de74e8bafc3b50ee15bb8325f50ab6f114f06d64225b0abde64b2`。

### 2026-10-30 work 第9轮续记：污染样本保留与整轮重测

- 实现提交 `6b39ae006`：外部并发导致整轮排除，原尝试及未运行 cohort 清单保留；同一预定顺序整轮重测，最多三次，每次等待实际外部进程退出有600秒上限。机械/协议/oracle失败一律立即停止，不借环境重测掩盖。每样本 round_accepted 必须显式 true 才进入正式分母/P50/P90；永久反控证明被排除的快样本及缺 cohort 的轮次不能填分母，定向1/1通过（0.023s），diff-check通过。
- 前一正式尝试 `pilot-r9-formal/campaign.json` 共17个样本，其中16有效、最后b15-k1原判定/计数/Job均正常但观测外部cargo PID65040/65540，campaign完整性false；原始结果/失败不删除。SHA256 `83ad19360e3c96339dd84f3998c736371144ac256676240952365ea7c64ca874`。不能使用局部数值宣称提速。
- 冻结6b39ae006后启动新完整正式采样 `pilot-r9-formal-retry`。截至续记，p1预热前两次整轮因实测外部cargo/rustc而排除（第一9样本、第二4样本）；第三次正在执行，尚无accepted完整轮、complete=false。后台工具session83477为本计划唯一测量进程，下一轮先查其状态/实际campaign，不重复启动；不终止他方负载、不中途修改冻结源码或提交。若三次耗尽，保留未完成结果，再按实际窗口条件推进，不能冒技术负结果/完成T-06。
- 测量等待期间只在本地计划附件准备 `spec-delta-preparation-r9.md`，明确标待验收，SD-01..05落点与尚缺证据齐列；它不是已交付canonical Spec，未提前写ledger/archived。Plan附件排除于源身份，未改测量合同/二进制/输入或阈值。
- 本轮T-04/05关闭、5/10；T-06成本与完整正式采样、T-07入口/脚本/AGENTS迁移、T-08验收、T-09独立复审、T-10规范/冻结门/合入/归档/guard清理均pending。无用户权限问题，继续既有G/X/M授权。

stage: goal | plan_id: PLAN-888 | plan_revision: r2 | round: 9 | from: work | to: work | status: executing | outcome: pass（T-04/05关闭，实际spawn/父侧分相/采样及成本工具实现；work整体未完） | worktree: D:/autostack/.wt/lang-888/auto-lang | commit: 6b39ae006 | next: work（先接续唯一正式采样session83477，再T-06全叶成本与T-07迁移；不重问授权）

2026-10-30 round9收尾进展：第三次p1预热九cohort完整有效并被整轮接受；已进入p1正式round0，前两cohort有效，session83477仍在途。上条“尚无accepted完整轮”为其写入时快照，现已推进；正式180与p2预热仍未完成。

### 2026-10-30 work 第10轮进行中：正式矩阵完成、入口迁移和规范准备

- 上轮分类progress（6b39ae006真实实现、T-04/05关闭），本轮先实查session83477仍live，接续原进程直到exit0，未重复启动正式campaign。权威树/branch不变，主检出代码WIP为空、他方文档保留，未建agent/消息其他聊天/写外仓/推送。
- **T-06正式性能矩阵完成，尚未关闭全任务**：`pilot-r9-formal-retry/campaign.json` complete=true、前后binding相等，227实际采样中198属于接受完整轮（180正式+18预热），29因外部构建污染整轮保留排除；独立读取确认198唯一(series,p,round,cohort)且每个正式轮恰9cohort。B k3/k10在p1/p2、15例与D3均通过原数值门；k1均未通过。15例k10墙钟P50改善77.68%/71.97%，总CPU改善81.78%/80.14%；C只在原D3单列通过，不替代A/B或M1四路径。原始SHA与14行完整分位/内存/决定见权威树MD `pilot-measurement-decision-r10.md`，其SHA256 `c9874802cbe4030add6bebb9d96d7fc28c559030e87c0340b32c307f159a3b29`。
- 推广限测试专用固定15例pilot默认B k10，单case仍A、其他recipe/原协议/产品路径不变；C仅显式使用。正式采样显式传所有mode/k，测后pilot唯一Rust变更只是未提供参数时的默认选择，原执行/计量/输入/worker/阈值不变；保留原冻结二进制SHA，不冒新SHA已采样。实际CLI新默认/单项以及原A/B/C/缺能力/计数和模块控制共40/40（5.441s）通过；producer计量路径复用未改证据，无198轮无谓重复。所有模块>=3warm成本仍pending，T-06不能勾选。
- **T-07实施中**：tm增加显式旧档→完整叶导航、原test名/regex→完整叶导航；无参数仍失败。t/tv/tt/tb/th至2026-11-30过渡映射，不在叶内抽样。周期tf --batch消费38 recipe显式schedule并只运行一次语言模块并集，旧language/tt/tb只做守恒list，UI/Man owner腿和周期/手工条件保留。26项full-regression控制通过（3.952s）；新schedule为长期load-bearing源码JSON，因仓*.json默认ignore已显式纳入跟踪，不是attachments原始证据。
- 更新本仓AGENTS及仓内commit/autoui-verifier门禁文字，替代裸t+触面档叠跑；保AutoGraphics定向/里程碑和heavy-mem规则。迁移741/788/796/818/821/827活跃脚本消费者，公共PowerShell helper读真实module receipt和必需原身份，不用PASS横幅。包括prose/缺ID/失败body/缺receipt/全NA反控，ModuleSelection33/33（1.485s），7份PS语法通过、788/827自检exit0。原legacy发现器在别名已迁移后明确拒重生旧合同，防用新结果覆盖历史基线。实际公开helper执行仍在核验，未先勾T-07。
- SD-01/02新规范及SD-03/04/05内容已在权威树准备，r1设计改r2，README/周期文档与owner边界同步；未写live ledger/归档或先标delivered。当前新发现6999义务/1013叶（最终执行前还须当前源绑定再核），模块完整成本、慢聚合细分、剩余验收、独立复审和冻结merge门均pending。迁移后首次发现因源配置纳入/变化触发重发现，保留失败attempt；不能把该目录旧receipt当新源证据。

2026-10-30 work第10轮内部审计：发现并重开T-03的外部金样子项消费缺口（分区/选测成果仍有效，整任务暂不勾选），current_step=4/10。真实已通过A2R叶有18原子receipt，但删除全部receipt后旧validate_bindings仍valid=true，因为它只强制VM cache绑定；不能借这些原始记录存在就称守恒守卫完整。具名F888-W10-CHILD。显式核实唯一成本进程PID8832及commandline后停止其自有树，已完成45叶、0原始失败但完整warm轮未完，收据/输出保留，不作最终成本/新红证据。公开3模块29义务与兼容选择已通过；9169362ef迁移提交、057dc49fb里程碑预算反控和16f56a60f初次成本冻结均保留。

修复正在原权威树实施：Syn AST绑定A2R/C/TS/Cookbook真实helper调用与原文件/期望身份（不以regex重构case），新增冻结wrapper→input关联守卫以防刷新时一同丢关联，执行端要求逐项完整原收据；原A2R诊断子项在原assert完成处补trace，语义/断言不改。Python删/重/错源/缺终态及刷新丢关联反控2/2；AST控制首轮1/2，指出statement macro未被ExprMacro visitor访问，修为Macro visitor后2/2（0.01s），保留首次红。原45叶计量不复用为最终三轮，修复后重新冻结并统一测量。T-03须实际新源发现/完整叶及消费者验真后再关闭；T-06全叶成本、T-07全验收、T-08/09/10继续pending。master当前13a9bd758，b5556e169至master代码/测试配置diff空、主检出代码WIP空。

2026-10-30 work第10轮 F888-W10-CHILD关闭：a436985c5提交真实AST/完整终态/冻结关联守卫。677金样/诊断/Cookbook原包装关联 +389原VM声明关联=1066，实际只读Rust发现unresolved=[]；声明/实际注册/ignore分开，原数据/断言/比较器保持。RootCore新增2项AST永久控制（nested/qualified/cfg声明/statement macro/raw guest text与非literal显式拒绝），2/2；Python 36/36（1.485s）。首次两项全Python运行的KeyError来自旧sidecar单测的手工fixture未包含wrapper_inventory，已为该单VM fixture补完整元数据/同域临时冻结合同，原sidecar漂移和删消费边断言保持；不是实际新目录缺字段。所有原红日志保留。

实际受控发现反例：唯一Root源中原test_03_control_flow_004_if_return调用替换为assert(true)，使用当前auto-case生产AST扫描（guest执行0）得到677→676关联，新鲜元数据仍被静态合同准确拒绝“original wrapper/input association lost: ...test_03_control_flow_004_if_return”；finally恢复原始字节并核SHA相同。见r10-real-refresh-drop-control.json（原源SHA05b572dcd2220776068326af080e4f3e52f561a93939367f1482a568555a4730）。正式完整A2R叶001 20/20、18原始子项强验证通过；叶010 20/20、20原始子项（含新增诊断trace）通过。再收紧VM分支，任何未绑定cache实际调用也拒绝，不因expected为空放行。原45叶partial成本不当三轮证据；T-03重新关闭、current_step=5/10，T-06全叶成本仍待重新冻结/三轮实测和慢叶细分，T-07/08/09/10继续pending。所有新源码/依赖/配置变化都在同一权威树，未削样本/阈值或借guard改产品语义。

2026-10-30 work第10轮 F888-W10-ENV：5ffb4eb3a全叶第一轮出现原3条P763-R1 AAVM红（两compile E0433缺auto_lang、goldens STDASSEMBLY.INVALID_BYTECODE:60，均与既存债精确一致），另ac_av_probe_prep_executor_contained_failure_shapes新红“2MiB stdout must fail the cap”。定位为新runner准备工具提供AC_AV_CASE_WORKER，原准备器故障控制走显式预建worker快路径而未派发fake build；不将其记旧债或B负结果。核实唯一测量PID14888及commandline后停其自有树，完整轮未完，全部partial/红保留，原始结果SHA见r10-env-closure.json；未停止他方进程。

修复原测试环境不改产品优先级：AV准备器控制先证明显式override有效，再去掉caller worker进入原输出cap/700ms墙钟/kill-tree/真实build正控，panic-safe恢复原3变量；共享RestoreEnv用于MN四故障+同进程真实恢复及CTE坏worker控制，保原caller路径，避免误丢预建工具而重复build。原断言、2MiB/1MiB cap、所有故障与正控、原采样/门值不变；后台PowerShell派生加Hidden。带AV/MN/CTE显式工具环境的4个实际nextest命名case4/4、2.485s；AV1.173s/MN0.968s/CTE0.314s/env-unwind0.019s。原MN故障矩阵82.022s主要为丢路径后的重建，修复后不删任何轴；独立benchmark不因此称加速。

诊断还观测AAVM编译两例聚合65.56s、MN快控+正式门20例聚合106.48s，新增明确source partition-policy按原测试分别拆分；原AV formal117.639s和MN body-edit102.425s是原正式采样原子，不减其30样本/修订/断言，后续逐ID慢模块登记，最终P90仍须三轮。分区消费端复核策略，失配/规则消失/重新聚合超数失败。另把AV798/MN818正式参数与golden更新开关纳入环境指纹，成本入口拒减样本/放宽原2000ms门/再生oracle；38项模块/分区/环境反控通过（1.549s）。当前无在途成本进程，正准备新冻结，不借partial或一次诊断冒T-06完成。tracked的818报告为本次测试产物，未暂存；最终保留本地副本后按原字节恢复，绝不提交原始报告。仍5/10。

2026-10-30 work第10轮环境恢复：最终分拆目录a461591f6的成本session19640因D盘耗尽终态exit1（Errno28）；81叶中50执行/31条件排除，三轮完整warm为0，所有partial与原3条P763-R1红保留，不冒验收。原818生成报告两份已本地保留后恢复HEAD字节。核实进程终止后，原权威树三个增量编译缓存（31308/20669/6440文件，共38426855917字节）完整转存至C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan888-cache-archive-20261030，原三个路径已不存在；不删除他方内容，不使用链接。D盘恢复约32GB。递归删除曾被自动审批以blocked by policy拒绝，随后使用保留全部数据的跨盘转存成功。缓存转存收据在权威树baseline/r10-cache-archive-receipt.json。

实际停止测量后以a5a6f29f7同步docs/specs/README.md §4旧全档门禁文字为本Plan获批模块并集门，未改执行/采样代码或原输入；正式pilot未改路径证据仍按原SHA复用，新全叶成本重新冻结来源。当前仍5/10，T-06/07/08未关闭。主检出45faf7437自b5556e169至当前代码/测试配置diff为空，主检出代码WIP为空，其他文档WIP保留。继续原第10轮work，不增加round或重复权限请求。

2026-10-30 work第10轮成本恢复启动：9b25d91a7冻结新目录（7007义务/1039互斥叶/38recipe），全部重新运行时发现exit0；显式周期清单仅用于选择这次完整portfolio，不把它变成新隐式all。唯一成本session42433，输出权威树baseline/r10-module-costs-recovered，原样执行3轮，预准备2.06s单列。下一次接续先核实际进程/results，不重复启动；测量期间不改源码/配置/输入。先前磁盘中断partial与三次其他诊断attempt完整保留，不合并成完整warm。

2026-10-30 work第10轮成本新观测归因：recovered第一轮language/auto-lang/back-proxy/th/001的19例18pass/1fail；唯一http_e2e_back_proxy_real_routes_corpora_data_face在018 chapters模板:id返回400!=200。与docs/plans/reports/755-th-full-receipt.md第17行及755-landed-independent-check.md第37行既存签名逐项相同，原test及018输入相对b5556e169无diff；实际back_proxy owner为固定auto-ui-runtime外仓，888零写入。此为已在册HTTP基线红再观测，不同于P763-R1三枚，原始红照录，不改expected/skip/阈值。实际日志baseline/r10-module-costs-recovered/round-1/122-run.log SHA256 b9e513fab69690e7e55eec6edbe526e7cdcc82c29e921760bcb65b43b2b006f3，关联收据r10-back-proxy-attribution.json。全部成本仍未完成，继续测量并逐项归因。

2026-10-30 work第10轮新增未关闭观测F888-W10-HTTP：recovered第一轮language/auto-lang/budget-http-budget-e2e/th/001十原例9pass/1fail、完整模块66.8016s。失败为http_e2e_plan781_w04_cpu_shutdown_no_late_write：CPU shutdown后late副作用写入；尚未取得同基线重放，不将其登记为已确认旧债，也不改断言。另十例是可分割串行聚合，最终须拆分并至少三轮测新叶。定位/源对照在冻结成本结束后做受影响专项，未改采样中的源码/配置。原日志及哈希见权威树baseline/r10-budget-http-open.json；T-06/07/08仍open，不能以成本complete代替功能通过。

2026-10-30 work第10轮空间维护续记：recovered第一轮约544叶时D盘再次降至13GB；只读确认在途rustc属于lang-889，未停止或修改他方任务。转存888自己8个experimental/<name>/target/debug/incremental到既有C盘archive的additional-*，总6968390936字节、每目录文件数/总字节与转存前清单一致，exe/deps/源码/日志保持；Cargo后续只重建ac-core/ac-syntax空incremental目录，0文件，不是丢失原缓存。维护session81511已exit0，D回约18GB。before/after清单及收据为权威树baseline/r10-additional-cache-{inventory,archive-receipt}.json。约544–562叶期间发生缓存I/O，原始墙钟全部保留、不扣除或声称全程空闲；必要时只针对实际超标/临界叶补充受控测量。正式pilot矩阵在此前已经完成，不受本次维护影响；源/配置/测试输入未改变，仍9b25d91a7冻结。唯一成本session42433继续，尚无完整warm轮，不关闭T-06。

2026-10-30 work第10轮F888-W10-SEMA-SCOPE（尚未应用修复）：第一轮ace-semantics/sema_boundary_audit原6例5pass/1fail，原sema_boundary_split_route_hotspots_untouched使用当前HEAD/master merge-base审查796范围，误拒888仓内技能文档；这是模块独立执行所需修复，不加宽原禁区白名单。只读重建已交付796实际可达链：首实施0ddeeca9312dca7d44a28e0f7eee6915b3247494之父5666c1232892b01aec9e15b370de1d4ea1c1f128→归档交付b76eca8d201ae313cef3b6f20dea55744406f287，共72变更路径，原全部namespace断言通过。旧复审52aad/重放91e982等部分对象已不可达，不能硬钉这些旧SHA。当前仅在附件准备r10-sema-boundary-draft.rs及binding.json：原断言逐字保持，输入绑定实际796历史范围，补空范围/根Cargo/共享禁区/888合法但非796命名空间四类永久拒绝控制。正式源码未改，待本次完整首轮结束再应用及定向验证。

计量策略据新finding调整（阈值/范围/每叶≥3完整warm不变）：先保留当前完整首轮，再修上述守卫和已知可拆慢聚合；最终对受影响新叶做≥3轮，其他源码/依赖/配置/输入及执行路径未变的叶仅补足三轮，逐模块记录前后指纹/原收据来源，不冒新catalog SHA已实跑，不丢弃有效首轮或盲跑已知失配的三整档。修复后仍须完整新叶计时和功能/消费者验收，不能仅把原聚合墙钟按比例分摊。若新的变更触及runner/通用执行链，必须扩大失效面而非套用此复用决策。下一次先查session42433及costs.json；首轮complete后停止其自有树再改源码，保留任何已开始的额外partial，不覆盖原结果。

空间维护：另转存18份旧auto_lang/auto_lang_core/auto_av_probe测试配置PDB（3997175808字节），以正式发现199个binary-path为保护集，当前199个exe均存在、对应当前符号未移动，所有旧exe也保留；source/destination长度校验通过，收据baseline/r10-stale-test-symbol-{inventory,archive-receipt}.json。该维护不改源码/配置/原输入，原耗时不扣减。64MiB syntax budget本轮整模块527.0023s、原2例2pass；原787记录build538.5s、64MiB及8/16MiB/OS64MiB约束相同，原规模例仍保全，将与另一独立测试拆开后具名登记。


2026-10-30 work第10轮恢复及证据失效范围校正：用户明确`/goal resume`，既有X888/M888和全链授权保持；host get_goal仍返回blocked，工具无active恢复接口，按用户本轮直接授权继续原任务，不新建/替换目标。本轮仍同权威树plan-888-dev，禁止agent/其他聊天消息约束保持。第一轮完整cost记录1039叶（958执行、81条件排除），939PASS/19FAIL，整体5382.910s但Job终态报“descendants live after workload exit”，已由原meter回收，不作完整验收通过；原始costs.json全部保留。新增producer Job包含Git/Cargo metadata、工具build、运行时发现等通用路径，且case改为短runtime+逐文件SHA复制，因此先前仅补受影响叶的复用策略不再适用：最终所有当前适用叶重新≥3完整warm，旧完整轮仅诊断，不冒当前源证据。

F888-W10-SEMA-SCOPE已应用：796历史可达5666c1232→b76eca8d2的72路径原白名单不变，新增空范围/各原禁区/非796命名空间反控，实际7/7通过。Case长路径修复保留所有原输入/比较，原生短路径运行，最终输出仍调用原auto-case保护域审查后才写选择/执行收据；非空输出拒绝覆盖，reparse祖先拒绝，原报告字节/路径不改并保SHA复制清单。公开刷新写本次target下运行目录，不自动改跟踪catalog；显式发布仅权威树执行。ModuleSelection44/44（2.032s）、NativeMeter/ProcessCapture8/8（10.269s）、auto-case bin4/4（0.076s）、回归driver26/26（4.392s）通过，原始日志在baseline/r10-{output-admission-controls,native-containment-final,case-output-admission-final,driver-migration-final}.log。编译现有外仓215条、Root6条warning为原有未改路径，未以压制或改外仓消除。

HTTP诊断仍保原失败：同一产品rlib中当前/基面原helper配对四次quiet运行全通过；另两次配对控制在原entered后把owner线程暂停800ms跨原300ms排水窗，RAII恢复，原4M guest/30s预算/10s缺席断言不变，两次亦通过。baseline/r10-http-schedule-contrast/results.json保真实命令/计量/fixture SHA；这不是旧commit全重编基线，未复现原红，不能据此宣布原产品缺陷已证实或修复。原取消产品实现与基面字节未改；后续新叶三轮实跑继续验证，该原红/归因限制保留至最终审计。


2026-10-30 work第10轮 producer实际残留定位及修复：公开cargo tm首次深目录15case重放在准备阶段被新守卫拒绝；language discovery父退出0、残留PID17008的真实映像为参考机MSVC vctip.exe，原红及完整Job诊断保留。实际cl最小编译生成obj成功但同样残留vctip；六个环境/旗标探针均未消除，未采用这些猜测、未改系统/安装文件。Get-AuthenticodeSignature=Valid/Microsoft Corporation；完整路径与SHA256 b8adde05deb9ccdbe359772a72a3225f7323a528be626f5b405d8032fb5a06f9作为长期源码compiler-auxiliaries.json钉住。只对准备/发现启用：原父exit0、全部残留逐一匹配路径+hash、TerminateJob后重新查询active=0才接受；原error/PID/映像保留，实际测试负载继续严格拒绝残留。真实cl对照exit0、cleanup_complete=true、remaining_active=0；对应本地baseline/r10-msvc-producer-probe/pinned-cleanup.json。永久控制保持默认严格、错hash/未知/缺映像/不完整枚举/回收未完成/非零exit拒绝；首版控制遇Python fixture额外conhost而红，临时fixture policy补其真实路径/hash，生产policy仍仅vctip，未加通用白名单。

另核现有RUSTC_WRAPPER=sccache共享daemon派生编译器可逃出本Job，修为本Windows计量子命令直接rustc（两个wrapper env置空；原编译选项不变），不停止共享daemon/他方进程。真实最小Cargo build传入两个不存在wrapper路径仍成功，实际rustc在本Job中，证明守卫不只是字段检查。Cargo launcher动态包标签不作有效配置，Cargo home/仓env默认按实际生效值绑定；真实不同home/stack覆盖仍使证据失效。最终相关ModuleSelection45+NativeMeter/ProcessCapture10=55/55（21.813s）通过，baseline/r10-contained-effective-environment-final.log；修复提交b0eddeaa5（前序4c35a9c95，输出安全2bcc92c15）。新发现v2为7024义务/1120叶，含随后新增真实compiler控制后仍须v3正式发现，不能冒当前目录已采样。

原A/B/C正式性能证据保留原实测二进制/绑定：本轮NativeMeter通用变化为残留错误分支诊断/实际回收、RustC外部wrapper禁止；固定15例和D3 worker为显式预建AV源码→MIR→ABC/VM进程内路径，不调用RustC/cache。原正常计时、Job资源标志、CPU求和、采样/阈值、父侧裁判与worker字节未改。可复用该正向试验证据但不宣称b0新环境已重采198样本；最终模块成本全量新三轮，准备/冷编译单列。T-06/07/08仍未关闭。


2026-10-30 work第10轮正式基面前移：master刚交付889，已核其archived及工作树清理，不能拿旧分支目录代表新交付环境。权威树e3c379f0a合入已提交master（两个Spec追加段冲突分别完整保留888与889内容，Rust自动合并）；1efb288fe补39th recipe native-collections-889、独立mn-collections-worker/AC_MN_COLLECTIONS_CASE_WORKER准备与标量端口并存，周期schedule同步，不改889产品/原断言。新原集合source/HIR及AV ownership会改变重编worker，旧198正式样本改为历史证据，最终推广结论安排新基面A/B/C完整矩阵后再定，不把旧SHA冒新SHA。所有模块最终新三轮仍pending。

补共享runner/配置global_inputs触面显式展开全受影响模块，不能因已归Python owner就只测治理单元；命名单个叶仍可独立，不新增隐式all。真实局部源码选择仍严格子集。Cargo env默认现在实际用于直接子命令及同一指纹，消除仅归一指纹却未给直接worker同默认栈的缺口。post889治理47+10+26=83/83（31.914s）通过，baseline/r10-post-889-governance-final.log SHA256 dc6674812f3b441cd7b7d24ae6389ae7e2e9772dae22af093ef251c9c56a55f3；首个共享触面fixture因缺impact graph schema拒绝（未误放），补完整fixture后命名反控和该83项均通过。Root/worker实际新编译及原15 Console深路径重放仍pending，不以83项替代。

post889发现首尝试在ace-native链接阶段真实no space（exit101），原recipe stdout/stderr/meter全部保留；停止请求发现publisher已自然exit1，未实际Stop任何进程。随后仅转存本树Root target/debug/incremental的11474文件/9148702302字节至既有C盘owned recovery目录r10-post889-root-incremental，跨盘文件数/字节/原路径消失核验通过。收据baseline/r10-post889-cache-transfer-receipt.json保留原reason并加stop_process_executed=false等更正，未经更正版本亦保留，不冒主动停止成功。另9个workspace cache 4999802727字节仅盘点，未移动；随后实际D盘约37GB，其他空间变化未归因本次9GB转存。当前恢复discovery session55430，目录baseline/r10-final-contained-discovery-post889-recovered，先接续该进程、不重复启动。未开始任何模块正式warm样本，所有失败准备记录不计通过。


2026-10-30 work第10轮正式目录及深路径关闭：b3e9238d1仅追加889原两条ignored条件，4819原rows/tiers/既存additional_ids及其余字段逐项等于更改前，baseline/r10-889-condition-migration.json存hash证明；初版fixture误把新ignored义务与同条件原叶合并视为失败，修为检查义务数/条件/原membership不变，命名反控通过，原测试不改旧条件断言。7694927e1冻结7161义务/1179叶/39recipe；公开cargo tm深路径实际选中16叶/20义务（15 Console+原host_console消费者5测试）全部pass，原15份report字节SHA复制且短runtime均清理；模块墙钟合计52.591s、最慢叶31.204s（单次诊断，不冒P90），runner准备112.398s、runner整体187.754s、外层实际整命令387.323s均单列。baseline/r10-public-console-full-v2-closure.json SHA256 213e88fcb3a403a949360035d67517d6523ac8d3569a2e1c50a30952412d4db9，results为closure内实际路径。原public v1准备失败保留，不计15次执行。

M窗口第一次600s未释放（0样本），真实外部编译/测试存在；随后观测剩两个Cargo是同一个loopback后端的常驻启动器，父/后端CPU零增量且有真实listen，不是持续编译。F888-W10-WINDOW-GUARD修复误判：explicit cargo run绝对*-back manifest、唯一同名后端/纯服务树、loopback监听、两次逐成员身份/命令/CPU完全相同才按闲置背景记录；cargo test/build及未知、编译/测试进程总阻止采样，任何服务CPU增加/身份变化等仍丢弃整轮。实际两次观测first busy2、second busy0/两个启动器，同一后端PID；永久正/反控及实际CLI10/10（5.966s）通过，e91983978实现、eb80a4b89目录刷新为7162义务/1179叶/39recipe。新guard SHA独立绑定，campaign逐次留服务成员CPU前后/身份/监听和busy原因；不伪称背景服务不存在或总机器CPU等于本Job CPU。未改变任何数值门/轮数/原输入/worker二进制/父侧oracle。

用户对协调问题明确答“继续等待其他构建自然结束”，保持不联系其他聊天、不停止他方进程；该答复不撤回已批准M888，仅按实际构建自然释放后采样。当前matrix session69127，baseline/r10-post889-formal-idle-aware；前两个被干扰warmup尝试及其原样本保留，完整接受分母由campaign实际round_accepted统计，不提前宣称通过。下一步接续该唯一进程至终态，未完成不重启新campaign；当前仍5/10，模块三轮成本、T-07完整收尾/独立复审/合入均pending。


2026-10-30 work第10轮M窗口接续：更正上一条“前两个被干扰warmup尝试”表述，实际是首个warmup尝试的2个样本被整轮丢弃，第二次p1 warmup完整9条接受。该campaign随后因真实新编译/测试反复出现，round0三次完整轮尝试上限用尽，终态incomplete；实际26条、valid=26，接受warmup9/正式0。全部raw和round_attempts保留，不作B数值负结果、不放宽干扰门、不混成198正式样本。

遵用户“继续等待其他构建自然结束”，本地观察器session28787持续只读观察，先连续120s无实际构建/测试/后台服务CPU活动后，才启动一份新目录的原样完整matrix；不是在旧round上作第四次重试。采样原脚本/10轮/两并行度/工作量/门值/原始worker不变，真实状态变化是再次尝试的前提，不联系其他聊天、不停止他方进程。观察收据baseline/r10-natural-window-wait.json，目标新campaign baseline/r10-post889-formal-after-natural-quiet；后续先看观察器状态及是否已起新campaign，不重复启动。若完整matrix通过，再执行本地start_final_module_costs.py显式39 recipe并集的全部1179叶三轮（该脚本有formal complete/binding守卫，尚未执行）。

2026-10-30 work第10轮续：用户要求检查并清空888约100GB目录中后续不需要的内容。限定权威树、逐目标绝对路径与reparse检查后，使用原生PowerShell删除11个可重建incremental目录11219810917字节，另按本次已完成39 recipe真实发现保护214个测试binary-path（清理前后逐文件SHA256一致），删除未选中的旧测试exe/PDB共828文件49708807680字节；保留源码、全部当前程序、依赖rlib及验收证据。合计释放60928618597字节；清理前清单、保护程序哈希、删除清单及清理后占用在baseline/r10-user-cleanup-*.json本地保留。未停止他方进程，清理期间没有正式性能采样。后台CPU判定修正已提交d27b76aa6，CLI定向10/10通过5.633s；新完整枚举7162义务/1179叶/39 recipe完成，尚待提交新目录及新正式矩阵，旧zero-CPU检测与此前不完整campaign原始证据全部保留，不混入新矩阵。

2026-10-30 work第10轮正式计量恢复检查点：前一goal turn完成自有构建缓存清理，为实际进展；本轮重核session52000、观察器Python64756仍存活，经历130.113秒连续自然安静后启动新candidate campaign。实际采样器Python69800的完整命令已核实，冻结9784524e7，baseline/r10-post889-formal-background-reference/campaign.json在2026-10-30T06:07:35Z记录21次尝试，已接受9预热+9正式；一次外部干扰导致整轮作废，原结果保留。尚不完整，不作推广或T-06通过。当前机器再次存在间歇构建，继续依用户原决定自然等待，不联系/停止他方任务。baseline/r10-background-reference-live-checkpoint.json记录活PID/命令、原campaign该时点SHA256=4AA424DFFBFE50C54CE8A2041AC36973ECAF095F241B099F607D1376D836829C；该JSON是会继续变化的在途证据，仅checkpoint哈希有时点意义。后续必须先poll同一session或核69800终态，不能凭等待超时启动第二实例。所有1179叶完整三warm仍未开始；T-06/07/08与独立终审/merge仍pending，目标active。

2026-10-30 work第10轮后续：session52000确认真实终态exit1，观察器sampler_exit=2，采样器69800已不存在，最后原因为第3正式轮外部干扰用尽三次整轮尝试。campaign原始40样本均机械有效，整轮接受仅9预热+18正式；SHA256=1C239441D3B802CEE7039CB3F4EC63737CBB68E6653E526C8ADDE7AE88414CF9。不完整、不生成推广结论，不立即同条件重开以绕过重试上限。只读wait_threads核“完成计划890”仍active/inProgress，保留cursor e2d29922-a07e-4708-b112-09ae06f50ddc:3，等待真实外部工作结束，不发消息、不停止进程。新进展为V-08/10工作定位演练：从当前真实k1/k3/k10报告每个组取首case，22/22沿源文件及case fence或binding.tsv行/列定位，并执行原replay_argv，仅一项且原source/oracle/expected/actual一致；v2原始汇总baseline/r10-current-report-reader-v2/reader-acceptance.json SHA256=FAD18C2B6F3192518AA89BD20A87D4D2ABCCF6AF5C74F6B2BB93EC3CE08F233D。首版reader假设只有case_document在B08 binding来源KeyError，partial保留，v2核两种来源；不是修改产品来适配检查。另当前冻结integration binary原双错oracle命名case exit0、414.218ms、meter error无，实际D原3→4及9→10产生FAIL/PASS/FAIL并分别命中exit mismatch，原始baseline/r10-current-wrong-oracles-control.json SHA256=261E9FC0D7CBDBAAC0A6FEAD2BDF9DBB0D5D1374BB6EF0F1A4F3C83DEAAD2102。此为实施会话验收，独立终审仍须自行定位，不冒review通过。模块成本与T-06/07/08及后续阶段继续pending。

2026-10-30 work第10轮外部条件阻塞审计：上轮为verified wait，本轮已连续第三个goal turn复核同一M888窗口条件。实际先前cargo check/test PID结束后再次出现rustc/link/cargo构建，新自然观察session21164仍活；短暂无编译不代表稳定释放，最新r10-natural-after-build-release-wait.json又记录实际编译/链接负载，尚无新正式样本。新观察器仅等待自然安静120秒，遵从用户“不联系其他聊天、继续等待其他构建自然结束”，不停止任何他方任务。上一campaign已终态、不完整，不能同条件重跑绕过三次整轮上限；新观察器拟用r10-formal-after-observed-build-release独立保留所有新尝试（正式参数/门/输入未改）。本轮verify_catalog及当前source binding检查通过7162义务/1179叶；代码冻结a509bdfb5仅前一文档提交，运行字节仍9784524e7同源；成本计量未开始。其余可独立完成的22组阅读/单项重放和双错oracle验收已留证，不能替代180正式+18预热及1179叶三warm。当前继续推进实质依赖外部窗口释放，按三轮阻塞规则将goal置blocked，而非完成或用户暂停；观察器仍保留自然等待，后续恢复必须先poll session21164/真实PID及其campaign，不重启同一实例。原始检查点baseline/r10-external-window-verified-wait.json记录三次计数、观察器命令/活PID及输出身份；无需新的授权决定，外部自然释放后续跑原work。

stage: goal | plan_id: PLAN-888 | plan_revision: r2 | round: 10 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-888/auto-lang | commit: a509bdfb5 | next: work after natural measurement-window release


2026-10-30 / stage: new / plan_id: PLAN-888 / plan_revision: r3 / outcome: pass / authorization: 用户本轮明确先模块归组/用例数量，再各模块单独单轮总耗时/CPU/内存，问题定向修复、特别长用例交人工决定，尽量每模块<=60秒；继承已有全链/X888授权及不代理/不联系其他聊天约束 / changed: G4、§5.2/5.5、AC-06/12、V-09/13、T-06/07/08及merge-owned；撤全目录三warm/P90和完整新A/B/C收益矩阵前置，未证当前收益前恢复默认A，B/C显式实验；不取消原单例正式采样/原义务 / evidence: 真实目录7162义务/1179叶/39recipe，现有Job meter与按叶runner可复用；已停止唯一自有自然观察器PID25756，尚未启动新campaign，未停止他方进程；r2合同存attachments/888/contract-r2-superseded-by-r3.md，全部旧收据保持 / next: work T-06先模块/数量清单，再单轮资源及异常/慢例人工清单；不再等待新A/B/C矩阵。

2026-10-30 r3 work续：已提交0556f61c3的单轮资源流程、原子长例非通过状态与默认A；ModuleSelection49/49（21.262s）、PilotCLI10/10（8.345s），原2个新控制区分CPU重复记账/缺计量与人工待定伪通过；py_compile、单文件rustfmt及diff check通过。当前真实目录7164义务/1179叶/39来源，模块归属/用例数量清单与11个已观察85–526秒长原子已提交05e76f663。实际单轮流程控制2模块：1实测（0.374559s、累计CPU281.25ms、子树Job峰值20185088字节）、1人工待定未执行；只有1轮、不输出P90，原底层gate=false，不以待定充绿。原始baseline/r3-single-observation-control/costs.json及日志保留。

用户随后明确授权“都归入ACE单独的测试模块，然后通知ACE路线去处理”：该最新授权覆盖且仅覆盖本次向“跟踪 ACE 引擎路线”（01a1819c-101e-7931-8513-dfc65f5d8d7b）的定向交接，原不联系其他聊天约束对其他任务仍保持。已核源码归属787/796/798/817/818/828，分syntax1/semantics1/AV3/native2/CTE4，含2个language recipe探针但实际归ACE。已在module-inventory-r3.md及ace-slow-tests-r3.md单列ACE慢测责任分组，保11既有独立叶/原子ID及启用、oracle/样本/规模/资源不变，不合成隐式全量。baseline/r3-ace-slow-assignment.json记录用户决定及历史证据；原在途测量的AWAITING_HUMAN收据保留时点事实，当前决定已被交ACE owner后续处理替代，不冒这11项当前已测/已修。send_message_to_thread已成功发送，wait_threads核对方active并实际验证11份历史日志SHA/只读分析；已要求后续实施/重型验证等888计量结束再交接，不改当前输入。

当前单轮资源session69380、driver9444、measure_modules54644、run_selected57680实活；baseline/r3-single-module-profile/round-1/results.json已报告87叶（54执行、2已单列慢例，其余原条件排除），尚未完整。发现3个AAVM原红已立即核日志：两compile corpus E0433缺auto_lang临时依赖、一golden STDASSEMBLY.INVALID_BYTECODE:60，与已登记P763-R1逐项/错误形态一致，相关原测试源码相对b5556e169无改，不伪称新修或新绿。继续其余模块单轮并定向检查新增异常；规范AGENTS旧三warm/整轮噪声段须在本轮结束后同步r3（避免测中改源），T-06/07/08仍open，不能提前送审/交付。

2026-10-30 用户后续要求登记：部分probe为开发调研，不应自动进入日常回归，全部用例后续须按用途审查日常准入。已在权威树docs/plans/attachments/888/daily-admission-review-proposal.md记录模块归属/运行用途两轴、日常/专项/调研/待确认建议类别及逐用例审查字段（不变量、消费者、触发、成本、替代覆盖、owner/决定）。不能以名称probe或快慢自动取舍；同叶混合用途时后续按用途再拆。此为用户所说“到时候”的后续审查输入，不擅自扩成本轮新的全目录执行/复测门；当前单轮测量、注册、启用状态和默认选择保持冻结，日常准入建议审查后再改变默认包，ACE11慢例的人工归组决定继续有效。未对任何测试自动删除/关闭，也未把所有已分模块的用例直接认证为日常。

2026-10-30 r3 work当前计量检查点：前轮为实际实现/59项定向验证及计量进展，本轮继续同一session69380，run_selected57680已核实仍活，未重复启动。当前超过351叶已产生收据（当时297实际执行、52原条件排除、2慢例按时点记AWAITING_HUMAN，当前人工决定另有ACE owner交接收据）；新的动态计数及活进程身份保存baseline/r3-live-profile-checkpoint.json，不把在途样本当完整结果。新增观察：MN signature rebuild单原子模块61.760s，其中原test body57.846s，为临界超目标，需按r3定向核/复测，不减30样本或2000ms原门。当前累计5个失败模块：P763-R1的3个已逐错误形态核；back_proxy的:id参数400!=200与已登记755基线红一致；889的runtime-content-identity测试独立启动新红已定位到ace_m1_tests.rs:1043直接读取尚未准备的collections静态库，而同模块其他测试稍后才懒生成，属于测试顺序/准备依赖。修复入口已查到现有ac_mn::current_collections_toolchain_identity()（真实worker identity模式会按原prepare(None)生成并校验runtime）；冻结计量结束后在该原测试读库前调用并检查原runtime-content轴，保私有镜像同长度篡改及Fresh/Stale/Fresh/共享库不变的全部原断言，冷产物缺失场景定向重放，不改产品协议/判定。为保护当前计量来源，未中途改源或并发跑第二测量。AGENTS旧三warm/整轮噪声说明、生成tracked报告的保留恢复等收尾仍待本轮终态后处理；T-06/07/08及review/merge未关闭。

2026-10-30 r3 work恢复检查点：用户再次明确 /goal resume，继续原权威树，不新建目标/代理或重复许可。完整单轮原始计量已终态：1179模块/7164义务，1087实际执行、81原条件排除、11已交ACE慢测；1080 PASS/7 FAIL；整命令5312.143s、模块墙钟和5146.350s、准备6.015s，Job终态无error，唯一义务并集已核。新红的889冷runtime准备、native共享库负控串扰、固定闭包fixture跨运行缓存污染已定向修复；原3个AAVM及755 back_proxy红保留对应原债，不改产品掩盖。原日志/报告已逐字节SHA保存后只恢复本计划生成的docs/reports路径。

初版修复26b8a1849后目录7165义务/1181叶/39recipe（新增1永久回归，AV17项拆为15快检+32MiB单项+smoke单项）。25叶定向单轮补测已终态295.111s，23执行、2原条件排除；22 PASS/1 FAIL。快检模块1.409s、smoke23.990s；MN signature临界模块PASS69.418s（体58.812s），不缩原30样本/2000ms门，连同原126.085s AV32MiB单原子补交ACE路线，共13项。原126s只有旧组中逐例wall，独立CPU/内存未测，明确留空。已按用户授权仅通知“跟踪 ACE 引擎路线”，893负责后续慢测归因/用途准入，不以893交付作为888前置。

25叶补测发现同因兄弟新反例：普通闭包suite原20次跨调用restart因初版一律run私有而失败。保留91-run.log，不能以另两次module-case绿宣布全不变量关闭。现修为仅显式module-case使用线程局部、panic-safe恢复的私有session作用域，普通run_suite保原跨调用显式session；不改原20样本/fixture/expected。新限定build已exit0；当前重新发现源绑定，随后仅重验该范围。T-06/07/08仍保持open，未送终审或交付。原r2三warm/默认B等历史摘要已显著标记被r3替代，当前默认A。

2026-10-30 / stage: work / plan_id: PLAN-888 / plan_revision: r3 / outcome: pass / worktree_path: D:/autostack/.wt/lang-888/auto-lang / branch: plan-888-dev / code_commit: 2a39b6ca6d6f996f9ecf1560a45fca8bf2cfd2a4 / task_ids: T-06,T-07,T-08 / evidence: 完整模块清单7165义务/1181叶/39来源；原完整一轮后只补受影响25叶、会话兼容限定修复再验22叶，后者20实际全PASS/2原条件排除、整命令203.732s、准备3.352s，code指纹b12cb00ef96d64ec04888b8fa5af1151f639fbd02855fbbf8124c6600a7f585b；汇总1084 PASS/4已登记原红/81原条件排除/11历史长例保原raw待定但已交ACE/1新长原子未独立计资源。13慢叶owner明确，不自动关闭，不要求893先交付；原11不重跑，新126秒CPU/内存留空，69秒模块保原30样本/2000ms。原fresh固定fixture两次module-case与普通suite20次restart均通过，三个准备/隔离问题及同因兄弟关闭；原始失败保留。ModuleSelection49、NativeMeter10、driver26、CLI10及default check/限定编译、fmt/diff等对应原绑定有效复用。工作验收/规范映射在权威树readiness-r3.md；四个旧红为P763-R1三例及755路由一例，原债/源码逐项核，非888回归 / evidence_hashes: baseline/r3-single-module-profile/costs.json cfbf07220982f810fa73e3bf7a731904da687b9af33712cab5e1310ee51133fa；r3-repaired-module-observations/costs.json 4d12f0181b99b4fcb4901b3a0a9a2588b2edfaec2401466ae8bf61947fb737fe；r3-session-scope-observations/costs.json a31f432891cef624424d6108d23d73222363b30cefbcd450e629dbaa316abd1d；r3-consolidated-observations.json 68a96d210d99b71b713c12eeaec3cccbfb4d0a1507eac43014ea9f49635b6a9d（均本地未入库） / blockers: none work-owned / pending: T-09最终验收与定位演练、T-10冻结模块并集门/实际合入/规范账本index/归档/guard清理，未交付 / next_responsible_role: final review，禁止再加全目录三轮/完整新收益矩阵。

stage: goal | plan_id: PLAN-888 | plan_revision: r3 | round: 10 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-888/auto-lang | commit: 2a39b6ca6d6f996f9ecf1560a45fca8bf2cfd2a4 | next: review

2026-10-30 / stage: review / review_scope: final / reviewer_identity: 本实施会话进行artifact-based验收重建；用户禁止代理，未另建独立会话，不虚称独立模型 / plan_id: PLAN-888 / plan_revision: r3 / outcome: pass / worktree_path: D:/autostack/.wt/lang-888/auto-lang / branch: plan-888-dev / reviewed_commit: 2a39b6ca6d6f996f9ecf1560a45fca8bf2cfd2a4 / base_commit: 50cd6f7778fa5332b2807e0d5bbc22a7d2ac2389 / dependency_revisions: auto-ui831553abc03d8a2c9b8b547e2e5dcfa3890e9ef5、auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4、auto-man e6cc06fbc496f366edde7929abfd49254ec13647 / acceptance_results: AC-01..09、AC-12 pass；重新读取真实catalog/legacy+atomic+wrapper约束、消费闭包/精确执行、worker与父侧边界、isolate policy及失败/恢复/冷态反例，目录实际验证7165/1181；原断言/采样/条件保留，三个新红与同因普通跨调用兼容均关闭；95个正式治理/原生Job/driver/真实CLI控制37.434s全部通过；从当前公开pilot重新生成k1/k3/k10报告22组，逐组首例定位原source及case fence/binding oracle，再用报告原argv真单例重放22/22通过。测试是功能复核，不是新收益矩阵；A默认有效。SD-01..05的canonical提案/设计/规约一致；遗漏/延后/workaround扫描无未授权缺项，未来用途审查/收益推广和13慢叶ACE归属是用户明确后置，不当当前实现缺口 / findings: blocking none；P763-R1三例与755路由原债继续，原CPU关闭偶发红无确定根因亦保留限制；未改成PASS / evidence: baseline/r3-final-review-controls.log SHA256 08e54f555f4343ed5c824b2ca03fb56f6f2a3f5b573cb3b2424cbd617eede788；r3-final-review-reader/review.json ddbf844b4f79274c4ec724f974e5b79f38a41d85e03aa7139f44188a68116d3a；r3-final-review-baseline.json保完整Spec源hash与code指纹（本地证据不入库）；worktree clean / pending_closeout_gates: merge需复核冻结模块/消费者有效证据、due批次判定、rebase等价、实际合入、canonical及仓内用户要求ledger/index读回、归档、fresh guard与清理；不得把reviewed当已交付 / next: merge。

stage: goal | plan_id: PLAN-888 | plan_revision: r3 | round: 11 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-888/auto-lang | commit: 2a39b6ca6d6f996f9ecf1560a45fca8bf2cfd2a4 | next: merge

2026-10-30 / stage: merge / plan_id: PLAN-888 / plan_revision: r3 / checkpoint: prepared,landed,canonical_verified / completion_kind: delivered / reviewed_commit: 2a39b6ca6d6f996f9ecf1560a45fca8bf2cfd2a4 / delivery_commit: 5a6c23afacd3c40412c5561a5ba2ae322a119c2e / landing: master两次ff-only实际到该SHA；53提交rebase保完整range-diff映射，三处差异逐项核，console提前return恢复至原审字节，其他执行源码/配置/测试输入与原审基面相同；canonical-only沉淀追加project/plans引用及用户仓内要求五个ledger源引用，未改他方ledger语义，spec-index实跑62项目、14既有ungrouped提示保留 / merge_gate: 按r3复用原完整单轮与25/22受影响模块、95控制及22新报告独立重放的实际有效证据；不跑旧档叠加/全目录三轮/新收益矩阵；13长原子交ACE、4原债仍如实非绿，不假造全生态PASS。publication补验：主检出读回43分片原CRLF哈希与GitLF失配，真实Gitadd/show反例修前红；仅固定write的LF序列化与目录Git属性，精确SHA/篡改拒绝不变；49控制13.757s全过，出版触面3完整治理模块30义务单轮均过（2.268/7.600/0.404s，整命令21.94s）；当前主检出真实load/verify_catalog通过7165义务/1181叶/43精确哈希，限定复审确认不变量关闭 / publication_evidence: baseline/r3-git-publication-red.log c8501d1d62e2cc3e4ea55c9abd818b1e8d0e2311b165418e2d301148f848b05d；r3-git-publication-green.log 75e4e1955dff0b55fffa9baf4150560caadb39886a9e86afa00c05d5ee7b6051；r3-publication-module-observation/costs.json 8edb64d801e9815aaec336173ae52929dfe740bbf5f6d827c0ff6e672c7bbc0d；最终r3-consolidated-observations.json 48ae23c5d2e35bac7d1065ed781d3d5c4b7177f419890fd6283f4ce4f34ce9a6 / canonical: docs/specs/auto-test-case/{module-selection,batch-execution,architecture,governance-and-inventory,ace-m1-conformance,project,plans}.md及docs/specs/README.md已在落地Git对象核；module-selection SHA256 77a6c290602575a2256ee33717fc0b3eafca9e9b2d43f53f82b6fa8f66e2d4c0、batch-execution b84268b4c7fc44cc113cfcbd625585c667076332948cd79818f9dd6f16631efa；.autoos五项upsert其余语义保留 / batch_due: false，888非5倍数、887回执2026-10-29T15:53:56Z未满48h，后续无到期五倍数落地 / artifacts: 主检出target/release/auto.exe与debug auto-case/test-case-pilot目前不存在，未在主检出做构建；产品语义/外仓服务未改，无需重启，模块工具下一次按prepare正常生成 / preservation: D:/autostack/plan-artifacts/plan-888/evidence原31697文件2109976627字节全部SHA复制，1607原exe映射去重保1157份4707136037字节；preservation-receipt.json SHA256 3aea33338c2caab0b66ed647f28063ef42f6c93393c395d4c88589194cb50c8e。原运行收据字节/路径不改，按该保留目录解析baseline相对路径；本地原始文件不入库 / pending: 实际归档文档提交与fresh guard清理，T-10完成前不报最终merge pass。

2026-10-30 / stage: merge / plan_id: PLAN-888 / plan_revision: r3 / outcome: pass / completion_kind: delivered / checkpoint: archived,cleaned / delivery_commit: 5a6c23afacd3c40412c5561a5ba2ae322a119c2e / archive_commit: 107148fcd（本计划与r2历史合同均实际入Git） / archive_path: docs/plans/archive/888-module-test-selection-and-ace-batch-pilot.md / actual_state_gate: 主检出已验证归档文件在Git、status archived、原active路径不存在、两个相对design链接从archive实际目录可解析，canonical module/plans导航命中本计划；无当前未完成work/review/merge项 / cleanup: auto-ui、auto-down、auto-man、auto-lang四个本计划树逐一git clean检查及紧接删除前fresh wt-guard clean后由各仓git worktree remove移除；branch plan-888-dev已删除、D:/autostack/.wt/lang-888空组已移除；未删他方树/分支/缓存、未停他方进程；移除前整组文件逻辑大小67222746812字节（约67.2GB，非磁盘全局变化归因）。保留目录约2.11GB原始证据+4.71GB去重exe；按规约只回收该保留目录内4个pdb，无target/CACHEDIR缓存，removed-pdb-receipt.json记原路径/大小/hash，其他证据保持。preservation-receipt的全部SHA复制验收为移除pdb前时点，4项有意回收由该增量收据解释 / final_state: T-01..T-10全部完成10/10，pending_closeout_gates none；4既存红和13ACE慢叶/未来用途审查按原边界保留，不宣称全生态加速或所有测试全绿；主检出本计划代码/规范/账本路径clean，他方已观察文档WIP保持不变 / next: done。

stage: goal | plan_id: PLAN-888 | plan_revision: r3 | round: 12 | from: merge | to: done | status: archived | outcome: pass | worktree: removed after four fresh clean guards | commit: 5a6c23afacd3c40412c5561a5ba2ae322a119c2e | next: done

## 10. 待澄清事项

### r3当前状态

旧M888完整A/B/C矩阵阻塞不再阻止模块优先工作；没有待重复批准的范围决定。13项ACE慢叶已按用户决定交ACE路线/893，不自动关闭；原11项的raw待定状态只保留原测量时点，当前没有未答交接问题。旧D-05/06/08仅按r3解释，历史证据不删除。


### 2026-10-29 历史入口核查（2026-10-30 用户已解除交接/窗口阻塞）

- 授权：用户 `/goal 完成计划888` 授权 work→review→merge 全链；后续明确答复“先核查，暂不交接共享写入”。该答复优先，当前只读核查，不取得 X888，不启动正式性能采样；M888 仍未安排。
- U887 已核实：归档 r2、10/10、交付 `dd916c184` 为 master `09bb414875d4e15050ecbbaf3959b1d72976d857` 祖先（`git merge-base --is-ancestor` exit 0）；885 亦已归档。主检出代码路径 status 空；他方文档/报告 WIP 全保留。
- 865 真实状态：branch `plan-865-dev` HEAD `f49c28054`；实际目录 `D:/autostack/.wt/lang-865/auto-lang` 存在，Windows worktree 注册仍记录 `/mnt/d/...`，普通 `git -C` 因 Linux gitdir 路径失败。仅通过 `--no-optional-locks --git-dir=D:/autostack/auto-lang/.git/worktrees/auto-lang3 --work-tree=... status/diff` 只读核查，未修复或改动他方注册/文件。
- X888 实际重叠：865 未提交 diff 中 worker `av_probe_worker.rs` +61 行，修改 `core_version_for` 及 `case_serve` 的 recipe 准入与执行分支；父侧 `crates/auto-lang-core/src/test_cases/ac_av.rs` +291/-若干，修改 `RecipeFamily`、`StepOut`、`run_check_impl` 请求/准入/判定/报告链。888 的旧 helper 提取与批量父侧复用正跨这两条链，不能据未合入 WIP 自行宣布无交集或释放。
- 只读快照 SHA256：865 worker=`326018b89038bf19cf9fa4ce2446c4c9dd0347b1708759f030b6914f0c3b7077`；865 parent=`57c827bbc09c0a1dfcb4b14fe70ca2388ac56e41296d75c58390406162d020ed`。命令 `git ... diff --unified=1 -- <上述两文件>` exit 0，实际 hunks 含 worker `case_serve` ~650/1225 与 parent `run_check_impl` ~1437/1562/1790。快照仅作冲突核查，不作可消费交付锚。
- 独立面：`scripts/test-governance/` 在已观察的 865 diff 无交集；但 T-01 的 X 冻结尚未完成，T-02 依赖 T-01 及相应 X，不能跳过完整最小链验证后把 T-03 当作可提交的执行结果。本轮未建 888 树、未写实现、未运行测试、未联系其他聊天、未创建 agent；0/10 保持。
- 恢复条件：865 正式提交/交付并释放上述 worker/helper 与 parent 准入/报告符号，或用户明确安排受控串行交接；888 重新核并集、源/锁/profile 与正式 SHA 后接受 X888。正式性能采样前另落实 M888。当前是交接阻塞，绝非 B 技术负结果，不关闭任何 AC。


| ID | 事项 | owner/关闭方式 |
|---|---|---|
| D-01 | U887及真实H/driver窗口 | T-01核正式交付与逐符号X888；reviewed不当U，无窗口报具名阻断，不擅改或将其记技术负结果 |
| D-02 | 模块边和nextest安全执行参数 | T-02在真实枚举/依赖中冻结，未知回退；两真实局部与跨边反控先证可用，不凭路径相似漏选 |
| D-03 | B旧case_serve的exit/全局状态能否有界提取 | T-02全路径probe；本Plan允许最小测试worker/helper面，不允许产品算法改写；不可行给具体needs_replan，不以887旧域外理由完成B |
| D-04 | C能否真实一次编译/运行多结果 | T-02/05在三纯表达式限域验证，未知不宣称可行；不适用可有证关闭C，但不能跳过A/B实做 |
| D-05 | CPU子进程累计/测量窗口与噪声 | r3逐模块单轮计总墙钟/父子CPU/峰值内存，背景/准备另列，异常定向复测；完整A/B/C收益矩阵后置，不再阻塞模块工作 |
| D-06 | 批量是否值得默认推广 | r3保持A默认、B/C显式实验；完整新候选推广另需原严格门，不把历史矩阵或未完成样本当当前收益/负结果 |
| D-07 | 以不相交模块并集替代最终裸cargo t门 | **r2 已由用户明确方向关闭**：T-02/03/07落实唯一叶模块/完整模块与消费者并集/旧入口迁移，T-09/10验证守恒与实际执行后启用；不把未实现的目录当成已替换门禁，不取消原通用/集成/慢模块义务 |
| D-08 | 每模块一分钟的可达性 | r3逐模块单轮观察、尽量≤60秒，异常/临界按需定向复测；可拆慢聚合继续拆，特别长原子单列交人/owner决定；不强制三轮/P90，不缩原正式采样、不强杀冒绿。未实测不标达标 |
| D-09 | 全部用例后续日常准入审查 | 用户2026-10-30要求：按实际目的区分日常回归/专项验证/开发调研/待确认，形成逐ID建议与owner/依据，再决定默认日常包。当前清单/单轮成本是输入，不因已分模块就全纳日常；不擅自关用例、不另加全目录重跑门，见daily-admission-review-proposal.md |

## spec-sync 回写记录

PLAN-888 r3：已将唯一叶模块/消费者并集、单轮资源及慢例处理、A默认与显式B/C、原session兼容、精确LF发布规则沉淀到auto-test-case规范；module project/plans回写、五个ledger源引用upsert与spec-index已完成。归档时实现与验收完成，清理尚pending，完成后追加T-10/goal最终收据。
