---
plan_id: PLAN-922
status: archived
feature_name: ACE M2全场景替换盘点与发布关键路径
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-03
plan_revision: 1
supersedes_spec_components: []
new_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
touched_goals: [GOAL-002]
affects: [ace-migration, v06-release-roadmap]
depends_on: [PLAN-903, PLAN-908, PLAN-906, PLAN-910]
current_step: 9
total_steps: 9
---

# [PLAN-922] ACE M2全场景替换盘点与发布关键路径

## 0. 变更摘要

把已批准M2/v0.6全替换合同转成有源路径、应用/目标/入口、owner、独立关闭oracle与依赖/写面/容量依据的迁移分母和后继拆包。区分能力缺失、已有能力未接线、证据缺失、现状未知；给关键路径和可信排期条件，不从Plan数量猜“一周足够”。

drafting r1、0/9，本轮只建本计划，不执行审计/测试/跨仓迁移；执行本包本身也只写语言仓盘点/设计/路线和Spec提案，不改任何产品代码、默认、发行或退役入口。不是第二次903旧M1审计，不是918收据/模块边界修复。

依据：[M2 REP-01..11](reports/ace-replacement-first-m2-contract-2026-11-02.md)、[v0.6发布门V06-ACE-01..05](reports/ace-v06-m2-release-gate-2026-11-02.md)、[Design43 §20](../design/43-ac-av-typed-mir-and-abc.md#20-915917在途时的下一波2026-11-02target-state)。

## 1. 目标

G1：全现有应用/模式/所有在用A2X目标/公共入口/实际安装发行构成可追溯、不漏consumer的迁移分母。G2：每项绑定新ACE能力/接线/证据状态、真实依赖/工具/语言缺口、原Rust重编触发和未来开发快路径。G3：REP-01..11/V06-ACE-01..05每门有owner、关闭oracle、拟后继包与写面/跨仓交接；给关键路径和容量约束。G4：旧执行/AST emitter、API兼容、共享中立库、测试/证书/注册/构建/包内二进制分类到可执行退役顺序。

非目标：在本包补语言/stdlib/FFI/多目标、跑全部产品与性能认证、改测试选择/预期/NA、关闭M1/M2、开始disable、替各仓owner实施或预占所有未来Plan号。M3/M4新增扩展当前不排，AACE仍v0.7。

## 2. 架构方案

只读源码/公开API/现行Spec/已交付收据/产品入口/包清单→带实际SHA与file:symbol证据的矩阵→能力/接线/证据分类→REP/V06-ACE覆盖映射→按真实接口依赖和单写窗口的可执行后继包/关键路径。

报告不是第二测试registry/自动发现器/规则引擎；现M1 binding/applicability不重写。运行证据复用须来源身份适用，未执行/不可访问/历史非祖先保未知；审计完整不等于产品通过。源目录名/grep命中只是候选，必须跟踪到可达入口/consumer或有证说明非生产用途。

## 3. 技术栈

Markdown及只读Git/rg/Cargo manifest/现行Spec分析；可用系统临时目录生成本包表格核算，不引入生产脚本/第二模块索引。跨仓只读，不联网搜索替代本地代码证据，不隐式启动服务/窗口/构建/安装。

## 4. 需求分析与背景调查

授权：用户将M2定义为补全ACE并替换旧AutoVM/A2X、覆盖AutoUI/AutoMan全部场景，旧实现disable，且加入v0.6发布硬门，接受适度顺延（如一周）但未确认工期。本轮要求规划下一波，只规划正式取号，不执行本审计。未来本包实施仅在语言仓写盘点文档/Spec提案，其他仓只读；跨仓修改必须另有正式合同/owner，不通过本包扩大。

读取main=5a0ef7609；913/914/916已交付；915/917在途；919/920分别认领资源六格/B18一格，921仅CA-A，其余C门和native三格仍open。903/908旧M1分母仅作一项输入，不代表完整旧AutoVM/A2X产品支持范围。918治理包的D-910-1/D-906-3不在本包修复范围。

| 实读来源 | 当前能建立的事实/限制 |
|---|---|
| docs/specs/overview.md | 模块地图/旧API归属入口，快照时间旧，必须按代码与各模块Spec重核 |
| crates/auto-lang-core/src/trans.rs | C/Rust/JavaScript/TypeScript/Python/GDScript/TSCN/r2a模块真实存在；存在不等于所有都是相同生产层次 |
| crates/auto-lang-core/src/lib.rs | execute_autovm_with_deadline_and_input/extract_autovm_result等生产旧VM API可达线索 |
| D:/autostack/auto-man/crates/auto/src/cmd_a2c_stdlib.rs | 生产旧CTrans消费线索，不能只迁A2R/A2C新标量就签全目标 |
| D:/autostack/auto-ui/crates/auto-ui-tools/src/tauri_backend.rs | 旧transpile_rust实际消费者线索；renderer/工具库不能按目录名全部删除 |
| D:/autostack/auto-ui/crates/auto-ui-compiler/src/aura/types.rs | AutoVM dynamic模式数据线索；UI模式/互动/回调/退出均须完整列明 |
| D:/autostack/auto-web/crates/auto-web/src/{host,http_server_handle,http_transport,lib}.rs | AutoVM/owner-thread/run_with_capture真实宿主依赖，不因Web独立仓已交付而视作ACE迁移完成 |
| docs/specs/repository-boundaries/{standalone-language-build,web-four-repository-delivery,generated-cargo-build-lifecycle}.md | 906/910已交付边界/发行身份义务；918仍独立治理，不抢写其实现/规则 |
| M2合同/v0.6发布门/C路线 | 全替换/零日常Rust重编/全目标/真实发行与旧实现disable；M1含C，不能自动缩回旧313 |

以上是初步取材，不是已完成产品盘点。T-01记录lang/UI/Man/Web/OS等实际repo/branch/SHA/WIP与只读范围；版本发布窗口尚未有可信当前承诺，不从旧12月底加七天推出日期。

## 5. 详细设计

### 5.1 输出与分母

拟长期交付docs/plans/reports/ace-m2-replacement-inventory-2026-11-02.md及docs/specs/ace-migration/replacement-scope-and-evidence.md，前者为审计快照/拆包，后者仅规范盘点口径和已核知识。每行至少有稳定row id、repo/实际SHA、生产file:symbol/入口、source/project/模式/platform、旧producer/工具/依赖、实际consumer/安装位置、原职责测试/oracle、ACE目标/现状、缺口分类、owner、REP/V06门、拟后继/真实前置/写面、证据/待核项。实现时可用一张主表+正交小表，禁止同义重复行冒覆盖。

分母至少覆盖：
- CLI/SDK/嵌入、REPL/持久服务、AutoGen/模板/Web消费者、所有现有AutoUI模式/应用交互和AutoMan配置求值/任务/build/run/dev/generate/发行。
- 全部现有支持的正向A2X目标与工程产物；Rust/C/Python/JS/TS/GDScript/Godot/TSCN按真实用途区分，r2a逆向工具单独归属；无证不删除，也不把中立data emitter强称独立语言引擎。
- 语言数值/模块/stdlib/类型/泛型/meta/反射/CTE/host/FFI按真实消费者需要映射；不仅看M1共子集。RF-I/RF-C等旧路线标签保定义来源，不先承诺一切未来生态。
- 开发Auto编辑→语义查询/真实依赖→AV/支持AC入口→同PID更新、恢复/GC；Rust宿主/库构建触发与显式A2X发布认证单列。
- default/all-features、注册/feature/隐藏开关/fallback/测试直接链接、构建/发行/安装包及helper可达路径，不能只看主CLI默认。

跨仓不可读/owner未知/未发布安装物保具名unknown和下一步；无来源不填“支持/无差额”。所有候选行须判生产/测试/历史/中立库/未来，各有事实，不以grep零命中证明全覆盖。至少对CLI、UI、Man、Web、Gen/SDK及每目标作入口→producer→consumer双向追踪，漏任一类别则AC未满足。

### 5.2 能力、证据与最小迁移切片

状态列至少分implemented且已验证、implemented但未接线、证据未重验、capability missing、unknown；附实际适用profile/identity，不用一个“完成百分比”混合。每个缺口给独立oracle与最小真实工程切片，不能只交demo/首帧/import；例如UI状态编辑/事件/外库回调/退出，Man真实配置/任务/产物，Web实际请求/生命周期。

legacy修复的M1证据保历史，新M2 successor ledger/recipe/cert迁移原义务；原source/expected/批准NA/accepted不改，不为新版日常保留旧执行器对拍。退役表逐项分类旧executor/compiler/emitter、薄compat API（只转ACE）、中立value/诊断/parser工具/runtime/Rust库/渲染器，给保留/移动/断依赖/删除顺序及所有consumer。不能以“换名字包装旧引擎”或内置旧binary作为新ACE完成。

native剩余R02/R03/R05三格须单列：R02/R03与915真实fault/Place事实及native现设施映射；R05先核源码/typed/MIR/执行ABI/闭包支持边界，输出一个有界包还是拆包的依据。无能力不能自动NA，也不把919资源事件包默认为已实现native closure。C后继B/C/D/E沿921正式接缝与各自资源/ABI门，工作标签不提前分配编号。

### 5.3 工程验收口径与排期

REP-01..11/V06-ACE-01..05逐门给有owner的未来验收入口/实际负载/独立oracle/失败控制。至少覆盖：稳定宿主Auto body编辑实际cargo/rustc进程0、全局真实依赖、未改函数各生产/工件IO0及对象/代际、type/meta/实例/CT与负依赖、同PID新旧调用链/失败原子、来源恢复/坏数据修复再读/有界工作集/最后pin回收、原工程P90≤2000ms及原更严资源门、实际独立安装/重定位/升级/旧版独立回滚。

914 D-05声明编辑允许冷声明前端+精确rebuild的已批准事实须明确标注；M2全局目标若仍需要更细前端，应列真实缺口/owner/后继，不把body编辑也放宽，不把历史M1例外冒成M2工程全门完成。本审计不执行这些验收，不为没跑的格填写PASS。发布/认证调用Rust/C工具合法；不要求引擎/外库自身改动也零构建。

后继包每个给目标/非目标、输入/出口/consumer、依赖ready任务、独立关闭oracle、P/S/H/N/跨仓写窗、owner与估量依据。关键路径按实际串行依赖/资源争用/可并行任务和原规模测试/修复/发行容量计算；区分实测、工程估量、未知，不编造精确日数。给乐观/保守区间的假设或明确尚不可估，说明延一周是否有数据支持，不能为了满足日期隐含减范围。

### 5.4 写面、审计边界与交接

本包P=自身inventory报告/Spec提案，R=各仓源码/Spec/收据只读；不改915/917/919/920/921计划进度/current实现/registry，不更新918的web_delivery.py或modularity配置，不启动其他聊天联系owner。共同ACE战略/交接文档只有本包交付时按全文件短窗更新，921/其他包仍可独立推进；报告每条接口变动标SHA与需重核consumer，不将快照复制成新的永恒状态源。

执行可先做T-01/02；不等915/917/921全部交付，当前gap如实列，后续正式SHA到来只刷新受影响行。关键原始证据只放本包scope，长期摘要进本包报告/计划；不把跨仓WIP当已批准交付，也不自行提交他方文件。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ace-migration/replacement-scope-and-evidence.md | 策略原则→来源绑定全场景分母/分类/证据边界/owner交接 | 给后继可执行输入，不冒实现 | AC-01/02/03/04 |
| SD-02 | add | docs/specs/ace-migration/replacement-scope-and-evidence.md | 新旧义务/中立库/实际disable与安装证据归属 | 不留双引擎兜底 | AC-03/04 |

本包不修改当前语言能力Spec，仅merge将经review的盘点规则/已核来源知识沉淀；每个尚未实现项明确target-state/open，不以Spec新增变成implemented。

## 6. 测试设计

纯文档/审计，不运行cargo测试/产品构建/窗口/服务/认证，不建新测试runner。只做真实来源交叉检查、表格关系/本地链接/重复遗漏/状态与SHA核对；执行时可用临时脚本只读检查报告，脚本不入产品，结果摘要折本计划。下面是审计验证用例，不是产品运行PASS。

| Case ID | AC | 输入/入口与使能任务 | 可检查结果/区分反控 | 边界 |
|---|---|---|---|---|
| V-01 | AC-01 | 生产入口/包/目标清单双向追踪，T-02/03 | 每类别/目标有来源consumer；在临时副本删一种模式/目标须被人工/结构对账识别 | 非全产品执行 |
| V-02 | AC-02 | ACE Spec/真实producer/原职责测试与收据，T-03/04 | implemented/未接线/缺能力/证据/未知分开；非祖先旧证据不冒当前支持 | 读取时SHA |
| V-03 | AC-03 | REP11门/V06五门及所有矩阵row，T-04/05 | 每门有owner/独立oracle/后继/前置；漏consumer或空oracle不通过 | 未运行的判据保pending |
| V-04 | AC-04 | default/all-features/dispatch/工具/测试/包内旧引擎与中立库，T-03/05 | 全部有归属与退役/转发/保留闭环；compat包旧执行循环不得标“中立” | 不实际disable |
| V-05 | AC-02/03 | native三格、C后继、console未来45/独立认证账，T-04 | 原M1范围/旧证据/新C/M2分开，无自动NA/expected调整 | 审计不签M1/M2 |
| V-06 | AC-03/05 | 写窗/依赖图/采样/发行容量，T-05/06 | 串并行/前置/估量依据一致；去掉强制发布验证应暴露缺口 | 未实测不造精确日期 |
| V-07 | AC-05 | 报告引用SHA/路径/链接/状态、跨仓WIP，T-06 | 来源可定位、未知有owner、旧快照标日期；无编译/迁移/发布动作 | 纯文档门 |

实现者完成盘点与自查后，同树独立上下文final review复核来源/遗漏/拆包与Spec delta；不能只相信报告自评。未授权不创建review agent/chat，review冻结SHA后实施暂停写，修复回同树。merge只做本包文档验证/漂移检查，不因ACE标题跑无改动代码组；周期批量回归仍独立按due处理。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01全应用/模式/目标/入口/安装分母可追溯，候选归属/缺漏/不可读有明确owner |
| AC-02 | work | V-02/05能力/接线/证据/未知与旧M1、新C、M2分账，native三格给真实边界/后继依据 |
| AC-03 | work | V-03/06 REP/V06全门映射与依赖ready后继包/窗口/独立oracle齐，关键路径/容量不伪估 |
| AC-04 | work | V-04全旧实现/测试/注册/构建/发行与中立库/compat分类到owner/消费者/退役顺序，不实际切换 |
| AC-05 | work | V-06/07报告与代码/Spec/收据一致、来源身份/未知/变更影响清楚，链接/结构验证无遗漏 |
| AC-06 | review | 独立来源/遗漏/反控/拆包与SD review pass，不以规划完整宣称引擎交付 |
| AC-07 | merge | 主线文档/SD读回/archive、全输出对账/guard/树分支scope清理收据，未修改跨仓产品 |

## 8. 执行步骤

| T | owner_stage | 前置/写面 | 结果/验证入口 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 当前各仓只读，lang计划/报告 | 核repo/path/branch/SHA/WIP/权限与已交付/在途，定义分母/状态/行身份 | AC-01/02 | [x] 已完成（8 仓状态表落报告 §0：lang 4276f493b（54 docs WIP+3 未跟踪）、down 7c4d5cb clean、**ui cfe325d4 有 6 项代码 WIP（他方，只具名不消费）**、man 47d8f7b clean、web 3de5524 clean、os 65fc70d（16 项含子模块/docs WIP）、gen 593b4ee、musk de92b90；919 merge 在途、921 executing 1/14、915/916/917/920 archived 核实；行身份 A-G 七类 60+ 行定义） |
| T-02 | work | T-01，入口/consumer只读 | CLI/SDK/UI/Man/Web/Gen/服务等双向追踪，V-01完整源模式清单 | AC-01 | [x] 已完成（5 个只读 Explore agent 并行+主会话直读，全部 file:line：A 表 12 行（entry.rs 执行漏斗 :8-163→lib.rs:210、eval_config_with_vm entry.rs:735、autovm_repl/persistent/daemon/client/mcp、auto-vm/auto-case/playground/autogen 二进制、execution_engine.rs 单变体）；C 表 UI 七模式（tool_ops.rs:334 分派、run_vm 全管线、tauri_backend.rs:130-141 旧 transpile_rust、ts/ark_adapter 委托旧 TypeScriptTrans、vm_bridge.rs:43-47 起 127 引用+host natives 30+）；D 表 Man 全命令（cmd_trans.rs 13 点、a2c-stdlib CTrans :63-87、pac.at 双过 automan.rs:123-169）；E 表 Web owner-thread 链（host.rs:8-130、server_dispatch.rs:699/856-859）；F 表 os-host/term/backend_abi/jade-garden/musk-vm/auto-gen(断)/vscode/launcher/coder/ai/book/ag-examples/安装器 12 行） |
| T-03 | work | T-01/02，只读目标/构建/包 | 全A2X目标/逆向工具及旧实现/compat/库/测试发行归属表，V-01/02/04 | AC-01/02/04 | [x] 已完成（B 表 11 行：rust/c/python/js/ts/gdscript/tscn-godot/r2a/s2s/escape(未接线 Plan-310)/相邻发射器，全 AST+Sink 经 Trans trait trans.rs:158 无 typed-IR 前端，逐行生产 consumer（TS=UI 生成器生产依赖最高优先；r2a 独立逆向工具）；G 表 7 类退役分类，含 vendored frozen SDK 副本×2、launch-vm.cmd:27 硬编码、a2c-stdlib CWD 约定、corpus a2vue/a2ark/a2j 本仓无 Rust 消费者） |
| T-04 | work | T-02/03，现ACE与M1/C合同 | 能力/接线/证据映射、REP/V06表/native三格边界与C后继，V-02/03/05 | AC-02/03 | [x] 已完成（报告 §2：binding.tsv 131 数据行分账——ac/native 33 gap（composite 12+console 15+scalar-only 6）、avm 16+ diff-legacy 差异行已登记、a2r emit-ok-run-unverified/snapshot cert 缺口；能力×场景映射：AV 语义执行 IV 但**产品入口零接线**（唯一 ACE 接线=auto-case ac 适配器测试工具级）、宿主 API/REPL/daemon/#[test]/config 求值/非 Rust-C provider 全 capability-missing；native 三格：R02/R03 有界可行（915 fault/Place 设施映射 mn 配方）、**R05 实证不支持——collections_driver.rs:46 "cannot carry closure environments"+check_closure:109 拒绝**，N-CLOSURE 设计包先行不自动 NA；console 45 格=O01..O15×3 未闭轴口径；C 后继 CA-B/C/D/E 沿 921 接缝；REP-01..11+V06-ACE-01..05 全门 owner/oracle/后继/前置映射无空 oracle） |
| T-05 | work | T-02..04，本包报告 | 后继有界包/owner/依赖ready任务/单写与跨仓交接/退役顺序/原验收，V-03/04/06 | AC-03/04 | [x] 已完成（报告 §4 后继 20 工作标签（M2-DEN/EXEC/SESSION/DEBUG/TEST/HOST/A2X-R/A2X-T1/A2X-T2/UI/MAN/WEB/GEN/OS/REL/TEST-MIG/RETIRE-1..n/N-CLOSURE/N-RESOURCE/CA-B..E），各带目标/非目标/依赖 ready/独立 oracle/写面 owner，未取号；§5 退役 8 步序+compat 保名转发/中立库判据；§6 关键路径=DEN∥EXEC→HOST→{UI,WEB,MAN}→OS→RETIRE/REL 汇尾、A2X-R/T1 并行、与 921 H 面（test_cases/mod.rs 等）漂移重验排窗；容量锚点=近两周单格包 1-3 天/域级包参照 863/866；估量=20+ 包 4 层串行，乐观 6-10 周/保守 3-5 月，**"晚一周"无数据支持（除非显著扩 owner 并行），排期决策交用户**） |
| T-06 | work | T-01..05，战略文档短窗 | 关键路径/估量依据与未知/测试发行容量、报告+Spec提案+来源/链接/遗漏自查V-06/07 | AC-05 | [x] 已完成（交付三件@0abcf3218：①报告 docs/plans/reports/ace-m2-replacement-inventory-2026-11-02.md（§0 基线/§1 分母 A-G/§2 分账/§3 门映射/§4 后继/§5 退役/§6 排期/§7 UK-01..10/§8 自查/§9 文档关系）②Spec docs/specs/ace-migration/replacement-scope-and-evidence.md（SD-01 分母/五类分类/证据边界、SD-02 义务迁移/中立库/disable 安装证据/owner 交接）③ace-implementation-handoff.md 全文件短窗（更新时间/M2 r2 定义取代旧 M4 表述/§4 状态表刷新 862-920→921/922/131 行计数）；V-01..07 对账落报告 §8；链接检查：交付物内部+specs 链接 OK，两份 M2 合同报告在主检出未跟踪（具名 UK，merge 簿记处置）；健康=纯文档零 cargo） |
| T-07 | review | work完成，冻结SHA | 独立来源抽查及类别完整性/门映射/后继/SD复核，pass才reviewed | AC-06 | [x] 已完成（2026-11-03 final review pass @0abcf3218：同会话独立工件重建——10/10 跨五仓 file:line 抽查独立复放（entry.rs:735/lib.rs:210/trans.rs:158/c.rs:4570/tauri_backend.rs:131/cmd_a2c_stdlib.rs:73/server_dispatch.rs:856-859/term.rs:490/collections_driver.rs:46+109/auto-gen lib.rs:90-93）；binding.tsv 131 数据行与 ac 列 33 gap=12+15+6 独立重算一致；REP-01..11 全 11 行在表、V06-01..05 全映射；分母 A-F 61+G 7=68 行、类别→门喂养结构完整（去任一类即失对应 REP oracle）；SD-01/02 逐条对 §5 规范增量表、无 implemented 越权声明、supersedes 空有据（全新目录）；边界核查=零跨仓写入/零产品代码/零取号。限制如实声明：与实施同会话/同上下文，未另建独立 agent/chat（计划未授权）。发现仅 F-922-O1（P4 簿记）：M2 合同报告×2/921/922 计划/wave-handoff 在主检出未跟踪，仓库内链接暂仅在主检出工作树解析——已具名 UK，merge 簿记处置，非阻断） |
| T-08 | merge | review pass | 文档身份/漂移验证、主线交付/SD-01..02读回/archive；不跑无关构建 | AC-07 | [x] 已完成（漂移重验发现两类：①master 两次前移（919 落地 5550944b6、AG 线 55feb6ab6），rebase range-diff 全等 0abcf3218→1bba0ef73、修复提交 d66fdef04；②**并发事件**——另一会话 2026-11-03 新立 923 NF/924 NC 认领 native 三格 + 主检出交接文档有大量未提交会话块：修复=报告 §2.1/§2.3/§4/§9 刷新 923/924 引用（D-05 规程首次执行）+交接文档以未提交版为基线调和（保全证明=diff 仅 3 行有意替换的过期事实行，他方块零丢失，stash==备份后丢弃）；受影响复审 pass 后 ff-only 落地 master tip=d66fdef04 线性无 merge 提交；canonical_verified=SD/报告 blob e022a9e1b9e0/6a602f187051 + sha256 f4690665…/8932fd05…；F-922-O1 处置=两份被引 M2 合同报告随簿记入库；纯文档零 cargo 按 PLAN-896 规则只做内容/链接检查；archive 本提交） |
| T-09 | merge | T-08 | 全实际输出核查、摘要折计划，guard后清树分支组与整个scope/刷新登记 | AC-07 | [x] 已完成（输出对账：本计划全部输出=worktree 两提交（1bba0ef73+d66fdef04 随树消亡，无外置 target/构建产物）+plan-artifacts/plan-922/{SCOPE,t01-facts,agent-reports×3,handoff-backup}——已整删（rm 后 ls 验证零残留），无第二证据目录/外盘散落/链接；落地检查=明示观察：纯 docs 变更无运行进程/发布件/web bundle 需重建重启，主检出 target/debug 构建产物早于本合并；批量回归 due 核对=last_covered 920@2026-11-03T01:01Z<48h、922%5≠0、其后无整 5 落地→**不 due**；.retention/policy.json 刷新 closeout（holds=[]，无具名未解决持有）） |

拟唯一实施树D:/autostack/.wt/lang-922/auto-lang、分支plan-922-dev（本轮未建），其他仓只读不新建写树。实际开工材料唯一D:/autostack/plan-artifacts/plan-922/，登记owner/用途/在途；merge整删，仅具名未解决hold可留。当前规划的临时检查用系统TEMP，不在项目目录散落证据/脚本；不动他方文件或建立链接。

## 9. 复审记录

2026-11-02 | stage:new | PLAN-922 r1 | outcome:pass（规划静态准入，非审计完成/产品验收） | 0..10节、T9/AC7/V7/SD2定义引用与链接/后继边界已核；next:work T-01，只读跨仓/自身文档，当前915/917/921尚未交付不构成盘点开工前置。未建922树/scope、未执行产品/语言测试，无跨仓写入或后继取号；必须产全分母/关键路径，不能以本计划已drafting冒M2收口。

2026-11-02 | stage:new | PLAN-922 r1 | 正式规划M2全场景替换盘点/关键路径，独立于903旧M1、918边界治理和921实现。仅初步实读生产入口线索，不声称已做完整盘点/运行；无跨仓写入/构建/测试/取后继号/切默认/disable。静态预检另记；本包完成也不等于M2或v0.6可发布。

2026-11-03 | stage: work | plan_id: PLAN-922 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-922/auto-lang | branch: plan-922-dev | base_commit: 4276f493b | code_commit: 0abcf3218 | dependency_revisions: 跨仓全只读（ui cfe325d4/man 47d8f7b/web 3de5524/down 7c4d5cb/os 65fc70d/gen 593b4ee/musk de92b90 @读取时；ui 有他方 6 项代码 WIP 只具名未消费） | task_ids: T-01..T-06 | evidence: 报告+SD-01/02+交接文档短窗三件提交于 0abcf3218；分母 A-G 60+ 行全 file:line（5 只读 agent 并行+主会话直读，原始件 plan-artifacts/plan-922/agent-reports/ 三份+t01 事实件）；binding.tsv 131 行分账（ac 33 gap/avm diff 已登记/a2r cert 缺口）；R05 native 不支持实证 collections_driver.rs:46/109；REP-01..11+V06-01..05 全映射；后继 20 标签未取号；退役 8 步序；关键路径/估量（晚一周无数据支持）显式交用户；V-01..07 对账报告 §8；链接检查=交付物内部 OK、两份 M2 合同报告主检出未跟踪（UK，merge 簿记处置）；健康=纯文档零 cargo/零新 warning | blockers: 无合同内未决（UK-10 auto-gen 复活/归档为用户决策项，已具名不改分母完整性；D-01..05 按 §10 闭或转后继） | next_responsible_role: review（T-07 独立冻结复审）

stage: goal | plan_id: PLAN-922 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-922/auto-lang | commit: 0abcf3218 | next: review

2026-11-03 | stage: review | review_scope: final | reviewer: 同会话独立复审（工件重建：10/10 跨五仓 file:line 抽查独立复放、binding 131 行/33 gap 重算、REP/V06 全门对账、68 分母行与类别→门结构核查、SD 逐条对照、边界核查；限制如实声明：与实施同会话/同上下文，计划未授权另建独立 reviewer） | plan_id: PLAN-922 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-922/auto-lang | branch: plan-922-dev | reviewed_commit: 0abcf3218 | base_commit: 4276f493b | dependency_revisions: 跨仓全只读@读取时（ui/man/web/down/os/gen/musk SHA 见报告 §0） | spec_inputs: SD-01/SD-02 全文 vs 计划 §5 规范增量表（add×2 同目标文件、规则/理由/AC 绑定一致）；supersedes 空=全新目录有据；无 implemented 越权声明；frontmatter new_spec_components 与实际一致 | acceptance_results: AC-01 pass（分母 A-G 68 行全 file:line，抽查 10/10 复放，UK-01..10 具名 owner）；AC-02 pass（五类分类有实例、binding 分账重算 33=12+15+6 一致、R05 实证不支持、console 45 格口径=15×3）；AC-03 pass（REP-01..11 全表+V06-01..05 映射无空 oracle、后继 20 标签带依赖/oracle/写面、估量区间与"晚一周无数据支持"显式）；AC-04 pass（G-1..G-7 分类+8 步退役序+compat/中立判据，未实际切换）；AC-05 pass（V-01..07 对账+链接检查：交付物内部 OK；两份合同报告主检出未跟踪具名 UK）；AC-06 本 review 关闭 | pending_closeout_gates: merge-owned——文档身份/漂移重验（919 合并后受影响行刷新）、SD-01..02 读回、archive、F-922-O1 未跟踪输入簿记处置、guard 清树/分支/组/scope、.retention 刷新 | findings: 无阻断。F-922-O1（P4 簿记）M2 合同报告×2/wave-handoff 及 921/922 计划文件主检出未跟踪→仓内链接暂仅主检出解析，merge 簿记处置 | evidence: 本记录内嵌命令/结果即持久证据；原始抽查命令与 agent 追踪件在 plan-artifacts/plan-922/（merge 折叠后整删） | next: merge

stage: goal | plan_id: PLAN-922 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-922/auto-lang | commit: 0abcf3218 | next: merge

2026-11-03 | stage: merge | PLAN-922:r1 | outcome: pass | prepared: reviewed基线 0abcf3218 + 漂移修复（报告 923/924 刷新 + 交接文档调和保全他方未提交块，受影响复审 pass）→ 修复提交经两次 rebase（master 两次前移：919 落地、AG 线落地），range-diff 全等映射 0abcf3218→365843437→**1bba0ef73**，修复=**d66fdef04** | landed: master `--ff-only`，tip=d66fdef04 线性无 merge 提交（主检出交接文档脏块先 stash、落地后证 stash==备份且备份⊂落地版仅 3 行有意替换，随后 drop） | canonical_verified: docs/specs/ace-migration/replacement-scope-and-evidence.md blob e022a9e1b9e0/sha256 f469066533628711…、docs/plans/reports/ace-m2-replacement-inventory-2026-11-02.md blob 6a602f187051/sha256 8932fd05aefd33dd…（landed-identity OK）；交接文档短窗更新含全部会话块 | archived: docs/plans/archive/922-ace-m2-replacement-inventory-and-critical-path.md（status: archived，随簿记提交；F-922-O1 处置=被引 M2 合同×2 报告同提交入库） | batch-regression: 不 due（last_covered=920@2026-11-03T01:01Z 距今<48h、922%5≠0、其后无整 5 落地） | cleaned: 见 T-09（guard 后树/分支/组/scope 全清、.retention 刷新 holds=[]） | evidence: 本收据内嵌命令/结果即持久证据；原始件随 plan-artifacts/plan-922 整删 | next: done

stage: goal | plan_id: PLAN-922 | plan_revision: r1 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: （已移除） | commit: d66fdef04 | next: done

2026-11-03 | stage: goal（归档态交付核实，无新写入） | PLAN-922 r1 | outcome: pass | verified-on-disk: master tip=593c33480（=archive 提交），交付链 1bba0ef73→d66fdef04→593c33480 线性 ff 无 merge 提交；canonical Spec docs/specs/ace-migration/replacement-scope-and-evidence.md committed @1bba0ef73、archive 文件 committed @593c33480（frontmatter status: archived）；无 plan-922 分支（git branch -a 零命中）、git worktree list 无 lang-922 树、组目录 D:/autostack/.wt/lang-922 已删；landing checks 已记录（T-09：纯 docs 无重建重启需求、target/debug 早于本合并）。主检出现存的 design/roadmap/918/921/923/924 等修改与未跟踪文件为他方在途会话工作，非本计划输出（922 自身三件交付均已提交），不阻断归档态核实 | next: done

stage: goal | plan_id: PLAN-922 | plan_revision: r1 | round: 4 | from: done | to: done | status: archived | outcome: pass | worktree: （已移除） | commit: 593c33480 | next: done

2026-11-03 | stage: goal（归档态交付复核，独立重验，无实现写入） | PLAN-922 r1 | outcome: pass | verified-on-disk: master tip=fe0b1e45f（593c33480 之后 921 等他方计划正常落地，不触及 922 交付）；merge-base --is-ancestor 三证 OK（1bba0ef73/d66fdef04/593c33480 均为 master 祖先），git log --merges 1bba0ef73~1..593c33480=0（线性 ff 完好）；canonical Spec docs/specs/ace-migration/replacement-scope-and-evidence.md git ls-files 已跟踪、@1bba0ef73 提交、磁盘存在；archive 文件提交版 frontmatter 第3行 status: archived；git branch -a 无 922 分支、worktree list 无 lang-922、D:/autostack/.wt/ 无 lang-922 组目录、plan-artifacts 无 922 scope（scope 整删核实）；工作树对 archive 文件的全部差异=round-4 两条核实收据（+4 行），随本收据一并簿记提交；主检出其余 design/roadmap/923/924 等脏块为他方在途会话工作，非本计划输出 | next: done

stage: goal | plan_id: PLAN-922 | plan_revision: r1 | round: 5 | from: done | to: done | status: archived | outcome: pass | worktree: （已移除） | commit: fe0b1e45f+簿记 | next: done

## 10. 待澄清事项

| D | owner / 有界关闭 |
|---|---|
| D-01 全consumer可达性 | **已闭（T-01..03）**：8 仓+12 附属仓逐一核对，缺漏以 UK-01..08 具名（rust_ui 深扫/嵌入面仓外消费者/旧 ui 面可编译性/apps 兄弟 clone/corpus 归属/frozen 差异/musk 生成物身份/worker 定位），各带 owner 与下一步，未凭猜测剔除 |
| D-02 R05 native闭包 | **已闭（T-04）**：实证 native 拒绝闭包环境（collections_driver.rs:46/109）；拆包依据=N-CLOSURE 设计调查包（源码/typed/MIR/执行 ABI 边界），不自动 NA，交下一次正式立项 |
| D-03 发布时间 | **已闭（T-05/06）**：关键路径/容量锚点/估量区间形成（20+ 包、4 层串行、乐观 6-10 周/保守 3-5 月）；"晚一周"无数据支持（除非扩 owner 并行）显式交用户，未从晚一周反推日期、未编造精确日数 |
| D-04 跨仓实施权限 | **边界保持**：本包只读已执行（零跨仓写入）；UI/Man/Web/OS 修改由后继正式合同与对应 owner 执行；UK-10 auto-gen 复活/归档为用户决策项 |
| D-05 变动快照 | **已立规程（T-06）**：报告 §0 记录读取 SHA；919 merge 在途/921 未完成如实 pending（UK-09）；后续正式 SHA 到来按 M2-DEN 规程只刷新受影响行，不代改状态 |
