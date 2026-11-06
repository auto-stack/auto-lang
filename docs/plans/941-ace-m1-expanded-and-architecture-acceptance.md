---
plan_id: PLAN-941
status: executing
feature_name: CA-E／ARCH 扩大后M1登记与最终组合验收
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-06
plan_revision: 5
supersedes_spec_components: [docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-applicability.md, docs/specs/auto-test-case/unified-source-syntax-admission.md, docs/specs/auto-test-case/ace-ca-d-bridge-certification.md, docs/specs/auto-test-case/architecture.md, docs/specs/auto-syntax/source-frontdoor.md, docs/specs/auto-a2r/record-text-hir-backend.md, docs/specs/auto-a2x/incremental-and-cache.md, docs/specs/auto-av/data-layout.md, docs/specs/auto-cte/source-query-integration.md]
new_spec_components: [docs/specs/auto-test-case/ace-m1-expanded-acceptance.md]
touched_goals: [GOAL-002]
affects: [auto-test-case, auto-ac, auto-av, auto-a2r, auto-a2x, auto-a2c, auto-abi, auto-semantics, auto-cte, auto-syntax]
depends_on: [PLAN-908, PLAN-929, PLAN-934, PLAN-935, PLAN-936, PLAN-937, PLAN-938, PLAN-939]
current_step: 7
total_steps: 23
---

# [PLAN-941] CA-E／ARCH 扩大后M1登记与最终组合验收

## 0. 变更摘要

正式认领CA-E与ARCH最终组合门，一个验收集成包，不拆成空schema与以后才找consumer。新增独立target/机制登记、严格父侧证据适配、四端同源独立oracle与版本化组合报告，组合原313 required门、C A/B/C/D、SYN与ARCH-01..08；复核真实函数级增量、恢复首编辑、资源、在线更新及独立认证。编译/语言/资源功能仍由原owner交付，本包不变相接走延期功能，不再造编译器、store、runner或meter。

架构：[Design51 §16](../design/51-shared-a2x-and-hir-to-rust.md#16-ca-earch最终组合验收2026-11-05target-state)；现状/基线/遗留认证差异见[立项报告](reports/ace-ca-e-arch-baseline-and-schedule-2026-11-05.md)。规划轮仅授权规划；2026-11-06执行授权及当前进度见§4/§9。 r2已加入并完成F-941-04条件解析修复phase（§5.8/T16/17）；r3的F-941-05 Rust字段写修复已完成（§5.9/T18/19）；r4按用户第48轮明确授权新增F-941-06 AV data结构化诊断透传修复phase（§5.10/T20/21）。r5按第60轮明确授权追加F-941-08 source-CT meta能力修复phase（§5.11/T22/23），F07 test-only提案仍独立待批。原完整目标/阈值及未闭任务保持。

## 1. 目标

- 新M1必须来自完整已冻结义务集合与实际证据，四端身份固定为AC-native、AV、ACE-A2R-HIR、ACE-A2C-HIR；legacy-AVM/A2R/A2C各有独立身份。共同源域从批准范围与各正式合同逐项认领，不动态求当前能力交集。
- 原131行/524格/313 required（285+28）、134 NA、32 accepted-diff、45 future及批准来源原样核对；旧binding不插第五列，source/expected/NA/分母不为结果修改。旧脚本exit0或登记ok不代实际执行。
- 在稳定新事实/实例/CT/ABI消费链上实测P/M/R/IO、无预热恢复首编辑与首错误修复、原预算/样本/P90，缺页/坏事实/迟到/资源与失败不能变PASS。
- 原工具链执行/严格证书与日常金样分轴；实际完整Rust/C产物及独立reference各对oracle，不因四端相等就通过。只有完整、有效证据才允许`can_claim_expanded_m1=true`。

交付边界：本包验收系统可以在正确输出真实red及具名blocker后交付；**M1里程碑签章必须另满足全部真实required门**，不能将验收系统实现pass冒M1通过。特别是现行924守卫下38格旧A2R cert-gap仍阻签，见§5.2；本包不默撤守卫或替旧A2R实现复合lowering。M2所有现有应用/全旧A2X目标接管与旧引擎disable依旧属v0.6另一路硬门，不由M1签章代替。

## 2. 架构方案

跟踪registry/批准规则→完整必需集合与证据需求→现固定组/正式worker生产者的原始证据和证书→域专用严格reader→独立source/oracle/target/producer/config/deps/tool/caps对账→纯组合器→版本化完整报告/外部内容hash收据。拟新增：

- `crates/auto-lang/test/cases/ace-m1-expanded/registry.json`与首例`.case.ad/.suite.ad`及独立oracle映射（新增）；
- `scripts/test-governance/ace_m1_arch_acceptance.py`（新增只读组合器，无工具启动/第二meter）；
- `scripts/tests/test_ace_m1_arch_acceptance.py`（新增治理反控）；必要语言侧新`test_cases/ace_m1_arch.rs`用于现有观察/报告边界适配，具体最小边由T-02冻结。

拟registry=`ace.m1.expanded-registry/1`、组合报告=`ace.m1.acceptance/3`，旧`ace.m1.acceptance/2`原样并列，不重用schema冒扩展通过。现有case-parent/strict cert reader/Job计量为事实源，新组合器只核验/关联/归约，复用已有固定组结果与正式case-parent事实边；不从最终返回值倒推生产计数，不让backend读expected或当前父判定。若需要补观察字段，原producer owner在其窗口接线，本包不复制分析器。

## 3. 技术栈

原`scripts/verify-ace-m1-852.ps1`、`scripts/tests/test_ace_m1_acceptance.py`；`auto-test-case/{contract,report}`及语言`test_cases/{mod,ac_protocol,ac_a2r_hir,ac_a2c_hir,abi}.rs`；929 `crates/auto-lang/src/tests/syn_report_929.rs`；893 `ace_cost_report.py`与固定组真实`results.json/*command.json`/原始terminal事件；现A2X worker `contract_certification/cert_rt_902/cert_coll_904/a2c_*cert`与937 bridge首例。路径以T-01实际现状复核；拟文件不是已存在实现。

## 4. 需求分析与背景调查

2026-11-06 执行授权：用户 `/goal 完成计划941` 授权 r1 全链 work→独立 review→范围内有界修复→merge/Specs/归档/清理，取代下文历史规划轮的“仅规划”动作限制；不改变验收、分母、预算、仓库范围或 CE-BASE-01。按计划 §6 不自动派发 agent/创建 chat。

2026-11-06 r2额外授权：用户明确“直接在941计划添加新的phase修复”。F-941-04因此在当前唯一941树内修复，取代本缺陷原先拟独立计划的流程；无需再问同范围授权、不分配新ID、不另开树。允许本缺陷必要的source条件表达式/CST投影及手写兼容oracle/相关语法身份、永久回归和实际AV/native消费接线修改，保原oracle/源码/预算/原scope其余义务；此例外不扩大其他S/N/X业务写面。当前进入 T-01；未将依赖未齐或真实 red 当作 work 完成。

授权：用户明确“为CA-E／ARCH最终组合验收正式立项……本轮只做规划，不施工”。本轮允许只读调查/设计/计划/路线文档；不建实施树/goal/agent、运行测试/计量、改代码或current Specs，不接管别会话。初始main `5f6737467`（比上轮只读状态推进：934已reviewed，未见实际D），见报告记录的输入hash和最后状态。原WIP与940等他方计划保留。

929归档14/14，D=`29a7d7007`为当前祖先、树已清；canonical source-frontdoor/SYN准入确已交付，但只签SYN，不能冒SQ/CT/ABI总绿。937归档12/12，D=`fa35943ac`为当前祖先，canonical shared-interface/bridge-cert已交；它的native是797声明IR消费者，source是C exporter，typed三腿与source import边界不能混同。原recipe`abi/ca-d1-bridge-v1`与旧`abi/interop-v1`共worker信封，但deck/leg互拒。旧空steps可空过是记录在案的族属性，本包必须在组合侧以executed>0/完整义务强制拒，不据其exit0签新门。

934当前reviewed r2 11/13（有复审pass收据，步骤数字不代替复审记录）；935 executing1/12、936 executing1/14，主体等D934/实际实例接口；938 drafting0/13、939 drafting0/14。最新交付/树与主线漂移在T-01再核，不把reviewed或在途branch当D。940是另一会话新立的legacy Core/UI parser推进性计划，不是ACE统一前门替代，不作为整包前置，仅真实H/consumer交集协商。

只读重算（非运行验收）：原binding131行/12列，批准hash匹配；required313、非绿能力token0、required旧A2R cert-gap38，required-set hash `35ad43aae27ffb2d2c9a162c3dd5b99f971e4cd7b2ed36f4fe747a5634bd770e`。现acceptance scope只核算且exit0不等里程碑pass；其ParentReport检查非空/PASS不能独自证明source/producer/义务完整，/3须独立核闭包。/2已有`certGapCells.Count -eq 0`守卫，不能被本包暗绕。

唯一实施树`D:/autostack/.wt/lang-941/auto-lang`、`plan-941-dev`（已建，当前基线以§9检查点为准），实现/内部自查/独立上下文复审顺序使用同一树。未来材料仅`D:/autostack/plan-artifacts/plan-941/`，开工登记owner/用途/在途；consumer/外置target/日志/cert scratch显式归scope，正常构建在树内。merge摘要折计划、全输出路径对账、删除scope与guard清树；不操作他方产物/SDK/工作目录，不写legacy specs.json。

2026-11-06 r3追加授权：用户明确“授权941新增修复phase（推荐）”，认领§9 F-941-05最小修复提案；已按第43轮完成T18/19。该授权不外推其他缺陷。

2026-11-06 r4追加授权：用户在第48轮对F-941-06归属问题明确“授权941新增修复phase（推荐）”（call_ad921d1cbddb484085b350b7ee0bc3f5）。本次认领原AV data链结构化诊断丢失的最小提案，新增§5.10/T20/21/AC-16/V25..28/SD-10；继续原941唯一树与完整goal，不新立计划、不增加仓库或提高预算。允许work、原授权独立review及merge；不重复确认。

2026-11-06 F-941-08追加授权已收到：用户对call_f0d62e537ab843b6a2d98593aaf23bcd明确“授权941新增修复phase（推荐）”。本项允许把第59轮具名source-CT meta能力冷/暖/撤销检查提案纳入941修复、复审与合入原链；继续同一树/仓库/原预算，不重复确认。第61轮已将该授权落实为r5 §5.11/T22/23/AC-17/V29..32/SD-11；第62轮T22实现提交be77185d5，当前7/23；下一work T23冻结职责/真实consumer/身份闭包复验后才能核销F08。F-941-07独立test-only提案仍待答复，不能从此授权推定。

## 5. 详细设计

### 5.1 登记与不可变义务集合

新registry引用旧批准件/hash与313集合hash，追加M1 A2C增补/架构机制裁定及各profile合同。义务键至少包含domain/case/ns/profile/target/obligation/direction/edit-kind；每项有来源路径+内容hash、独立oracle来源、owner/正式D、需用recipe/输入profile、正/负终态、原样本/预算域和证据入口。必需集合hash只由批准范围/义务构成，不受本次执行选择、报告状态或支持探测影响；证据集合另算hash。registry不含本次报告hash，报告含registry/child hash，避免自指循环。

两类登记单列：Q4同源语义行（四个真实新目标）与ARCH机制/方向行（按真实能力定义target适用性）。C既定域来自892/902/904与921/925/926完整冻结语料及补例，不只取当前3+5+20个C首例；这些28例是首例不是完整分母。scalar原61例、资源/method/collections负控与typed helper边界逐项映射。934/935/936正式承诺的有限新source必须进入相应Q4行；937/938/939有限FFI按source/typed及真实native/C方向登记，不冒四端FFI全覆盖。C未承诺闭包/Dyn/全Unit/console/宽源码等按原批准明确列future/out-of-profile，不将缺支持动态排出required或自动套旧NA；原已required不得转换。

每个required原格位、新target行、ARCH原子义务必须映射可执行case/check/producer或明确未交付owner，group名仅定位不能代覆盖证明。相同basename/共享编号不等同相同源；源码字节、profile/caps与oracle语义版本绑定。已有无源码typed/scenario不伪造Auto源，宽typed frame证明不冒宽源码覆盖。修改登记范围/NA/accepted-diff须独立明确裁定，不由实现者为了绿自行决定。

### 5.2 判定谓词与38格旧认证差异

总谓词固定为：`legacy-current-gate AND legacy-required-execution-evidence AND C-core/resource AND Q4-oracle/certification AND SYN/ARCH-01..08 AND precise-incremental/restore AND budgets/time AND expanded-registry/evidence-valid`。其中`legacy-current-gate`沿当前/2实际更严守卫，不能因旧脚本exit0、core token全绿或新HIR A2R证书通过而视为true。

差异`CE-BASE-01`：G-DEC-3/旧适用性与scenario规范将旧a2r金样和认证分轴；924之后脚本及gap Spec明确新增cert-gap=0条件。**本r3继续继承r1/r2的更严现行守卫且将两份规则/来源同时呈现，不改旧批准文件/hash，不把38 cert-gap改PASS或自动由新A2R证书顶替。** 38项逐格进入独立阻塞表（见报告完整清单），功能责任owner=旧A2R复合lowering/认证线，证据接线owner=本计划T-01/T-04。存在本域真认证仅漏接线时可由证据owner按原规则回填映射；缺编译/语义/trap能力需原线后继正式包，不预占号，不塞本包修后端。若要依原批准恢复独立记账签章，须用户另作明确裁定及语义修订后执行，不能借报告/3更名暗撤守卫。

完整报告分出`accounting_valid`、`evidence_complete`、各域verdict、`acceptance_system_delivered`与`can_claim_expanded_m1`。前四类不得推导最后者；真实缺项/red、not-executed、STALE、infra、resource、cancelled都不能claim。preview允许局部查看但恒为ineligible，不能生成正式签章身份。报告中旧32 accepted-diff永不变绿、45 future照实展示、38旧cert账保其域身份。组合器逻辑正控用隔离测试收据，不得进真实里程碑证据集合。

### 5.3 证据来源、版本与独立oracle

每项证据至少绑定：registry/义务key，source/typed输入hash，producer源码/构建位与版本，target/backend/grammar/schema/ABI，profile/caps/权限，tool/helper/runtime/config，实际依赖与provider代际，oracle/reference/完整actual文件清单，run/window/request/nonce与真实终态、selected/executed/排除原因，原始run/command/terminal/计量文件内容hash。不同schema/recipe/legacy/HIR、native/C/Rust、source/typed方向互不授权。

父侧当前期望从跟踪registry、实际源/文件/工具及已交付合同独立重建，不从child echo/default-zero/can_claim字段获得。每个域/profile冻结必需计量字段及合法字段集，不能仅因p./r./budget.前缀合法就任意放行或缺项补0。严格解码拒未知/缺字段、重复key/义务、错类型/枚举、计数溢出、尾随、路径逃逸/重复目标/缺件、多件、旧schema；重hash真实产物与原始收据，检查actual invocation和terminal，不因“全部PASS”文本通过。reader只读registry与显式授权scope根下的常规文件，按真实规范路径/重解析点检查拒绝路径逃逸，不跟随child自报任意路径去读用户文件。raw run必须来自正式driver/worker，声明selected>0不够；缺原始可追溯收据就是证据缺口。哈希用于内容/适用性对账，不声称能证明任意伪造本机执行历史；不是新安全签名平台。

Q4每条positive/trace/trap/diagnostic分别对独立冻结oracle；不能仅四端一致。Rust/C actual由现场当前producer完整生成，手写reference为独立平移单元，两者分别实际编译运行对同oracle；四端公共同错、reference从actual复制、只改expected、只编片段/删分支/main/依赖都要可区分红。库形式使用正式library assembler身份（937），不能认证时临时剪exe文件。诊断保raw stage/code/span/message/序及各引擎合同映射，不强求字面同名；普通非零、工具错误不充语言负例，typed policy拒与坏事实ERROR沿914/932现行规则。诊断负例须真实前端按冻结族拒且codegen/run=0，不为不存在的可执行产物伪造编译认证；有效程序的value/trap行仍必须有其正式执行/认证证据。

日常emit/金样/证书核查不得隐式spawn cargo/rustc/cl/link/guest；预构建worker准备单列。显式certify只沿现受管执行器/Job，缺工具infra、无自动下载/安装或fallback。身份完全相同的已完成认证可以严格重验复用，记录理由/实际证书/完整依赖；这不允许复用TestCase PASS或旧guest值冒本次运行。native/AV及本次要求实际执行的编辑场景仍需真实调用；source/provider/helper/target/工具/头/库/caps变化按实际身份STALE，必须显式重新认证，日常不自愈证书/golden。

### 5.4 ARCH与真实观测责任矩阵

| 机制 | 证据owner/输入→consumer | 本包独立复核与拒绝控制 |
|---|---|---|
| ARCH-01语法 | 927/928/929 source/CST/decl→全部真实consumer；SYN-01..06 | 新/旧parser入口计量和恢复；SYN child schema准入不足以自证全部机制；公共首编辑/闭包子体复用不能丢 |
| ARCH-02 SQ/实例 | 934query facts/935实际实例→core HIR/四端 | 冷关solver红、暖hit关solver绿、坏facts消费前红、异owner拒；真实类型事实驱动/实例集合及正负依赖，不看query调用日志 |
| ARCH-03 IR/Q4 | core Checked HIR→唯一MIR→AV/native，HIR→新Rust/C | 同源四端各对oracle，源序/短路/副作用/checked trap，typed与source分列；不回退旧引擎 |
| ARCH-04真实AV CT | 936 source Hash→checked callee/935实例→真实AV→冻结→推导/HIR→四端 | 首miss真实execute>0、hit execute=0；调用点不改而callee/meta/实例/子CT变化必失效，host折叠/旧deck不冒新source |
| ARCH-05 cache/P/R | 929及934..936/938939各required页→当前selected target | 真实producer与IO交叉核对；bad/missing required拒P0 restore，无预热首编辑/首错修复后再restore |
| ARCH-06资源/ABI | 881/894/904/915/919/923..926/931与937..939 | 单源facts/清理、独立C布局和原三腿+新source方向；private ABI/内部struct不冒public，错配call0 |
| ARCH-07在线更新 | 866/929同PID safepoint/旧链/lease | 入口一次、在途旧链与新调用分别看代际，最后owner才回收；重启/暂停恢复不可充在线HMR；不要求新native loader/完整AJ |
| ARCH-08父准入/来源 | 908/893/933真实固定组、独立oracle与本包registry/report | 完整义务集合、真实运行终态与成本归因，不用摘要/目录存在/忽略附件/零选择/伪PASS铸绿 |

T-01/T-02把表拆到每个required原子义务/具体entry与closing oracle；不能只有八个总布尔字段。存在新功能缺口仍归原owner，已归档能力发现新缺陷默认走具名独立修复流程；**F-941-04/05/06及F-941-08分别获用户明确941内phase授权，按§5.8/r2、§5.9/r3、§5.10/r4及§5.11/r5执行**。F-941-07未授权；不得把这些具名例外推广到其他缺陷、悄悄重开旧Plan或在组合器模拟能力。

### 5.5 函数级增量与恢复首操作

固定编辑集含单body/方法、变长前缀/闭包子体、签名/layout/模式、meta正负依赖、935模板/实例、936callee/子CT/frozen结果、938/939契约/库/provider/caps。哪些函数/实例/绑定真正受影响由事前依赖图/独立预期确定，不能看计数后重命名“受影响集合”。callee只改未被caller读的body：非内联caller生产0，但target引用/绑定/link必须使用新实现；签名/布局/实例有效实现/CT依赖改变按实际读集失效，调用点字节不变不能盲hit。

真实parse/project/resolve/solver/instantiate/CT execute/resource verify/lower/encode/COFF/C-Rust fragment/tool入口装attempt/produced/reused/discarded计数，未改函数/实例/绑定P=0、其payload额外IO=0且resident身份保留。冷声明重建/恢复R、合法整体装配/link另列，不把整源生产藏M/R、先whole后filter或用缓存guest值填本次结果。缺观测=UNOBSERVED/证据缺口，不自动0；跨目标语义事实可共享，目标artifact/cert/ABI互拒，强制copy/whole/fake-zero控制应红。

恢复链至少：fresh→hit→restart/P0深验→**无任何空编辑/ensure_env/预热直接首次source编辑**；另restart→首次非法编辑→旧成功root仍能执行→修复→fresh等价→再次restart→立即编辑。覆盖来源漂移/历史回访、多函数/方法/闭包、实例与CT/foreign实际读取页；各页owner仍按929来源、934facts、935实例、936Exec/Use、938foreign、939资源契约分责。错owner/credential/旧schema/缺required/cap撤销在使用前拒；缺页不能restore成功后首次编辑隐式cold。失败/取消/迟到/基线冲突不得安装半套source/semantic/target root；故障修复必须经下一实际restore/read重验，不只测当次fallback成功。

### 5.6 资源、时限与分态

各域原更严预算AND，严禁一张全局大帽覆盖原域。新组合器单registry/子报告/输出各≤4MiB（UTF-8字节，不以字符数代替）、自有活数据合计≤48MiB、父private峰值增量≤128MiB，哈希大产物有界流；解码/索引/候选/重复副本分配前预留。若与A2X共进程计入其原48MiB总工作集；只核已记录峰值不再启动第二meter。输出过限typed resource，不截断必需条目伪完整，不遍历用户目录或任意路径。

| 域 | 继承的原正式预算/负载与证明 |
|---|---|
| A2X/C | 总会话48MiB、同owner store128MiB、cert scratch512MiB、单source/meta4MiB/每流1MiB、值8MiB/managed32MiB、父privateΔ128MiB；C编译120s/run30s/cert wall360s，原更严者优先 |
| ABI | 797/937/938/939 build60s/run5s/total70s、IPC4MiB、每流1MiB、hot8MiB+temp/index8MiB、单产物64MiB/请求128MiB、真实lease/GC/原方destroy；不被上行宽门覆盖 |
| source/SQ/INST/CT/native/AV | 逐项继承原profile实值/原scale/ignore/平台/Job条件；934原1MiB/30编辑、16MiB页、512 churn、4096深链及各真实后继合同不降；无原观测需具名补证据而非编0 |
| 资源语义 | 正常/return/branch/loop/trap/resource/cancel/close、逐分配fault1..N/N+1与同进程retry；失败原值/mut/out不变，cleanup不需剩余预算，last consumer/last owner/provider正确；已支持pin挡GC/最后释放后收敛 |

正式时间沿原样本（A2X fresh/hit/restart/body-edit各30、原load256=273575字节/115函数、各资源/方法/collections原负载及更严样本，ABI六场景×30、512修订等），适用暖门P90≤2000ms，P90序位N-floor(N/10)。缺样/非法样/错误值不绿，原scale/深链/平台条件不减；编译准备、发现、实际工作、父判定落盘、cleanup分列，子span不代父完整窗口。完整固定组尽量≤60s不是强杀门，单轮归因不冒正式P90。不把Rust当前无累计runtime配额谎报为已实现8MiB cap；沿真实events/峰值与其现合同，对共享内存/进程树按893定义避免双加。

### 5.7 执行入口与写面交接

拟只读入口（T-02/04使能，当前不存在）：`python scripts/test-governance/ace_m1_arch_acceptance.py --repo-root <tree> --registry <tracked-registry> --evidence-index <scope-index> --mode milestone --out <scope-report>`。existing legacy核算可由显式命令`powershell -NoProfile -File scripts/verify-ace-m1-852.ps1 -Scope acceptance -RepoRoot <tree> -OutDir <scope-legacy>`产生，仍不当成执行或新增绿。旧批准/报告文件本轮不改。

| 写面/交付 | 本包允许先做 | 必须等待的具体窗口/证据 |
|---|---|---|
| 独立P | 新registry草案/纯reader/独立治理反控/可追溯oracle/coverage对照 | 不需934..939整波交付；不得消费未审branch冒D，未知字段/入口保pending |
| 934 facts | 适配需求/producer-reader清单/只读准入设计 | D934（现reviewed不是D）及实际TypeSupply/query/required/计量协议；source-query仍其owner |
| 935实例 | 实例义务/逻辑ID/签名-vs有效实现/反控草案 | 需真实实例source/四端/恢复页时取稳定接口，最终验收需D935，不用796模型 |
| 936 CT | Exec/Use/真实execute/失效/恢复义务设计 | stable真实AV端口+各required页，最终D936；本包不把CT执行搬host、不扩大CT FFI权限 |
| 938/939 | ABI方向/认证/lifetime/typed-vs-source mapping可先 | 最终D938/D939及其codecs/父reader/实际source imports/resources；D937不替它们 |
| S/N/X | 除§5.8 F04、§5.9 F05、§5.10 F06与§5.11 F08四个具名已授权phase外，本包不修改core/HIR/MIR/lowering/语法/资源分析/native/C业务实现 | 若需补观测由原owner稳定SHA单写交字段/计量；不借总验收施工修功能 |
| H | 新case/contract/SDK/worker/strict cert reader、mod.rs、Cargo/lock、test-groups与父报告适配 | 934→935/936、938/939与本包逐文件SHA/符号/全部consumer/兼容门释放接收，whole-file单写；不按append-only并写，不占窗等无关D |
| 940等外线 | 只读Core/UI parser与测试注册实际交集 | 非新增语义依赖，不触其WIP；只有真实H/legacyconsumer交集协商 |
| 正式采样/认证 | 计划/样本/oracle准备并行 | 与当前owner错峰，原串行/resource组与单实例保持；最终联合门在实际所需D全就绪后 |

与在途contract/schema的等价适配不自动扩大其语义，必要reader变更须列producer→required字段→所有reader→发布/恢复路径→冻结验证。缺项保owner与关闭oracle，不用“本包后面统一补”让原包假pass。公共窗口释放后可接，不要求全部计划先结束才做独立段。

### 5.8 r2新增phase：F-941-04 条件表达式/记录构造消歧

用户授权在941内直接修复，非另立计划；原T01..15/AC01..13/V01..18保持，新增T16/17与AC14/V19..21。此phase完成后回到原完整验收，不以修好此例终结941，也不在本phase单独归档941。唯一写树仍D:/autostack/.wt/lang-941/auto-lang，基线4cd5782d1；计划簿记仍主检出。

实际输入→consumer：原冻结925文字源`text-char-925--v03_eq_full_bytes.at`（c88f8407c7235a1042d062575aa52c68a258eca8752e49844620ab3dfd1358c7，oracle1）→`compile_text_record_source`/`compile_parts_with`→统一CST前门/投影→Checked HIR→owned AV与native升级链。原记录两次18/20、同一源两consumer均source.syntax拒；单text Rust/C认证均接受。隔离括号兄弟源owned AV实际1，仅诊断用途，**原冻结源不改成括号版**。

写面与闭合：先沿adapter.rs/parse_program_with实际dispatch确认CST grammar、`source/cst/project.rs`条件/错误诊断projection和`source/parser.rs`手写差分oracle中的共同歧义；修复位置由T16证据确定，不预定只改手写parser就算生产修复。条件尾部变量或调用不得把随后body左花括号当记录构造；表达式上下文中已有合法具名/嵌套/带括号record ctor仍能解析，错误记录字段/缺body仍按原分层诊断拒。覆盖record/text-record及共享recordal消费者、全fn与fragment；旧empty-ctor等无关兼容怪癖不借机放宽。若语法canonical bytes变化，沿既有GrammarId/codec/缓存键合同更新并重验，不复用旧身份或绕Typed resource。

原parser/AST/grammar资源预算、源码字节帽、工作栈/深链与原回归负载保持；不增语法支持上限、不允许截断/递归捷径，不引入第二parser或在consumer预处理加括号。共享source修改的完整职责组明确为`ace-source`，消费者按实际修改选`ace-av`与`ace-native`（已有组，含原资源/平台条件），以及`case-ace-m1-text-q4-941`真实20项与已有Rust/C同源日常/认证严格读回。实现迭代先命名反例/代表兄弟，最终冻结并集只跑一次；若实际改动触语法库则补`ace-syntax`，如未触独立语法库须记录免跑理由。原全仓批门仍按到期规则，不变为本phase门。

此phase交付闭合需：原冻结源四端各对独立值1，十文字源原全部四端仍逐各oracle/具名trap；if/while尾变量与比较/括号、合法记录构造和非法字段/缺body代表组合的AST/诊断、whole/fragment保持；原失败在未修baseline可重现而修复后原例及兄弟全部通过。观察到原shared producer变化的首链/CT/text cert与源码合同按真实identity刷新，身份未变的证据继续复用。独立复审仍由T14在同树新上下文执行，不能以本phase自测顶最终review。

### 5.9 r3新增phase：F-941-05 Rust字段写值根可变绑定

**授权/基线**：用户2026-11-06明确“授权941新增修复phase（推荐）”；沿§9已审阅最小提案执行，不另占计划或工作树。唯一写树仍D:/autostack/.wt/lang-941/auto-lang、plan-941-dev，入口冻结a236a44d4ad6ee7595d601bcef6f01af56d9570b。r3修订入口时为executing2/19；T18/19现已完成（第43轮）。r4当前executing4/21，原T01..15与所有未齐义务不撤，不因本phase独立通过就交付941。实施/内部核查/最终独立review仍同树顺序，不自动派发agent/chat，最终T14绑定当前修订（现r4）和冻结提交。

**已证不变量与缺陷**：规范auto-a2r/record-text-hir-backend.md承诺字段路径读写、Copy快照及view/mut/move边界。原M01/R01/R02各AV/native/C通过，Rust完整actual实际E0594失败；对应独立Rust参考分别0/19/9。Checked HIR的普通record值局部与引用参数不能混同：字段赋值虽不重绑整个record，Rust值局部仍需可变绑定；mut record参数是&mut引用，字段写不要求参数绑定自身mut；view参数非法写仍由既有验证器拒。text-record move参数按值拥有，现verify::local_writable已允许该域MoveOwn根写，清理责任仍由原BodyResourcePlan决定。

| 规则/写面 | 实际路径与责任 | 正反例/关闭条件 |
|---|---|---|
| 值根与引用根 | source→Checked HIR FnFacts.body/LocalDef.role+param_mode/PlaceKind→a2x-rust lower.rs::collect_assignees/FnLowering::new/rust_params→RStmt::Let/AssignField→print.rs | 普通值根浅/嵌套字段写能编译；mut引用参数不误加绑定mut；不全局给var/param加mut，不修改源语义/前端/HIR/C/资源计划；原whole-local赋值、BorrowMut与free根规则保持 |
| 原样本与兄弟态 | 原ace-m1/src/M01.at/R01.at/R02.at及original-data-reference/对应rs，原共同profile/caps/值不变；新拟record_field_writes_941.rs并注册现conformance.rs | 完整actual+手写reference实际rustc默认与-O对0/19/9；额外覆盖分支/循环字段写、readonly Copy根、mut/view/move与receiver边界；不能剪main/改actual源码或加宽allow吞unused_mut |
| 旧target片段 | provider.rs::RustTargetProvider::fragment_key_inputs已将RUST_PRINT_VERSION入Rust AXR1..3键，当前a2x-print/2 | 本次改发射语义沿既有打印轴升新版本（预期/3），旧/2 Rust片段miss/明确STALE后重新lower，不能命中旧let无mut片段；语义事实仍按原target-neutral规则复用，C键不改，不手删store或把旧产物重标新key |
| 证书与实际consumer | a2x-worker的daily-emit/certify（Rust/C共用worker身份）、既有contained executor与941 strict reader | 重建实际worker后按SHA/依赖变化显式刷新现289成功默认cert及3失败项，18 data源36项均应完整认证；旧证书不得换SHA回填，C等未变发射文件逐件等旧字节。conformance exe变更时原14 Rust优化配置按原精确flags完整重放；原AV/native工具若未变只读核验复用，不把身份更新当语言执行 |

本phase原始生产输入pin：lower.rs SHA=031e994e4c45be5b804b88015a3f659507a0973c3ae240799d95fd8030af5d0f；print.rs SHA=be49d9f13627a39f75fb367e4db69d152c60aa298e6bcf4b32d023ed41768cee；provider.rs SHA=bf77b1dcb2e6ccc180c0042cefcbd95565ee21444e61c1895a4c6dbaf68ea60f；record/text当前Spec SHA=5867db025399f723c62585f246494b723d0d4bc2b40cb310f2cf53afedb66d44。原3失败/33成功raw与独立隔离见§9第37轮，永久参考合同SHA=92cbb23d49bc7ed9cb108055dc696c88b6b8f2a7b53e2536888d9972fdde17aa；新phase测试尚未实现/执行，不能用现治理绿顶替。

测试与门禁：T18先用原失败及窄兄弟回归区分旧错误实现；T19在冻结修复提交运行现完整`cargo t ace-a2x`与涉及的`infra-ace-m1-arch`，含其原ignore/平台/预算/命名性能条件，不另跑全experimental/all。原360s cert wall/120s compile/30s run/每流1MiB与48MiB等既定上限保持；新增回归复用scalar_release_941现有受管capture/现有工具执行器，不新造runner/meter。默认与-O回归可另加-D unused_mut作额外guard，既有正式证书/原14配置flags不变。若实际触及共享源/原语法/AV/native生产模块，停下列出越界原因和必要完整消费者验证，不能顺手扩大修复；当前定位不需改这些模块，原F-941-04 1779并集证据按未变身份继续复用。

关闭F-941-05须原3Rust完整默认认证真实PASS、兄弟/缓存/参数反控与完整职责组通过、当前全链只读组合重新绑定且该3格不再缺证书；F05关闭时的原8语言FAIL、4AV缺码、旧38cert-gap与所有ARCH/legacy/资源/恢复未闭照实保留；T06独立span随后已由第44轮核明。已有非snake_case/dead_code健康基线单列，不能拿本修复通过冒整个T13无warning。本phase不修GAP-941-AV-DATA-DIAG-01、不猜T06 span、不修改C/Checked HIR/源oracle或提高预算。

### 5.10 r4新增phase：F-941-06 AV data结构化诊断透传

**授权与执行入口**：承接§4用户明确授权和§9第43轮最小提案、第48轮收口。基线`330c2534497e90ea6b06c5156382ffda4597aa76`，同树`D:/autostack/.wt/lang-941/auto-lang`、`plan-941-dev`，6/21为T16..21完成；F06在第52轮完成实际验证闭环，继续原完整T01..13，不以本phase替代最终T14。材料仍唯一`D:/autostack/plan-artifacts/plan-941/`。实施/内部自查/最终独立review顺序同树，不派发新agent/chat；最终独立审查仍T14，不以phase回归冒整包review。

**规范与已观察事实**：auto-av/data-layout.md规定独立data profile与既有MIR/AVAD验证/资源；auto-test-case/profile-v2-native.md与architecture.md要求父侧以真实stage/code判断，不让I/O/infra满足语言负例。实际`compile_record_source`返回`Vec<ace_core::atom_text::Diagnostic>`，包含stage/code/message/span；`data_driver::request_data`当前将其首项转为String，`av-worker`的data-run Err分支按前缀推断阶段并固定`diag.code=""`。原M09/M10/M11/R09 Case均FAIL，严格reader拒绝从message补code。core已有结构化StageDiag边界；owned仅作原协议兼容参照，其worker仍有字符串前缀/首词提取路径，不作为typed透传的证明，也不是本phase改造面。

| 规则/实际数据流与所有权 | 写面与兼容边界 | 可观察关闭条件/owner |
|---|---|---|
| 源→共享前端typed Diagnostic→data driver持有→worker只投影实际主诊断→原父adapter/strict reader | 在现`data_driver.rs`增加或等价接出具名typed失败API/载体（拟新符号，由work择等价实现），保留现`request_data(...) -> Result<DataRequestOutcome,String>`公共签名作兼容包装；worker使用typed边，不从兼容String反解。前端原primary stage/code/message/span保持到投影边，wire stage仍按实际pipeline边映射为既有frontend/verify，code/message复制实际primary；不将bind/verify源码阶段冒改为新协议stage | 实际四原源码经过真实worker返回frontend与source.not-callable / verify.immutable-write / verify.mut-conflict / source.type-mismatch，父原parse completion可判PASS；消息改变不影响stage/code。work T20；AC-16 |
| front-door、lower、MIR verify/encode/ABC admission与store/open/persist/缺诊断边界 | 在原错误仍有类型的边按种类分类；有稳定code则沿原pipeline阶段投影，不按Display/字符串前缀分类。I/O、基础设施或没有可用诊断的未知失败不制造源码code；保原request-error/exit纪律。保留原成功/精确guest trap/trace/value/typed、持久恢复及资源合同。String兼容包装只供旧调用/展示，不成分类来源 | 合法record仍实际对独立值；typed错误的code与消息解耦；坏store明确infra且不输出冒充4源码负例的语言诊断；合法typed非前端拒绝不混到frontend。work T20；V26/27 |
| 正式Case→原始wire/父结果→当前strict组合 | 不改avworker/1外形、data recipe/profile、AVAD/语义/缓存键、源码/oracle/原completion或预算；不因修复添加不需要的新字段/改旧missing字段规则。若必要协议形状变化须先举证并修订合同，而不能迁就reader。旧空code材料继续保历史/无法证明，不回填code或hash；无需放宽现reader以接纳正确非空code | 原16诊断逐项严格PASS且4原FAIL闭环；原8不相关scalar FAIL、所有U与旧38守卫保持。work T21；AC-02/03/16 |

实施最小面为`experimental/ace/crates/av/src/{data_driver.rs,bin/av-worker.rs}`，新的永久`tests/cases/data_diagnostics_941.rs`及现`tests/conformance.rs`注册；必要同模块typed边界单测与文档候选可改。不得为此修改共享前端语义、HIR/MIR/AV执行器、owned/core生产逻辑、native generic trap或A2X typed缺口。若实际触面超出此定位，先记录必要性/影响与新验证责任，不顺手接其他缺陷。

源码基线：data_driver.rs SHA `cb5fc1388053894782db80479212169adfa205996a7c3d4cb4bbfb749fb619c3`；av-worker.rs SHA `42389efe3a2043764c6115834f9972be58fe93f2f3c8bf06bf8f21e62cf2acb5`；compile_record_source adapter SHA `955eee5a3e6d147cc664f15022f93727108fe30726e36ed30a5f64282a6b6161`。完整相关9文件与当前AV worker基线见`<scope>/f94106-contract-baseline/sources.json`（SHA `4eaa047eeaf49d377f5e96586aa67e5ef150d31f930937083c10feccf6a386fa`）；当前AV worker=`e4c2006f31c4a8aff96e5f0422f0d8ce5cf7667dedfb6aebe1347f7b2521a544`。这些是只读定位证据，不是新测试通过。

**冻结与消费者闭环**：T20先使V25..27成为永久可运行测试，在未修baseline用原4源/消息脱钩控制见红，再实施。T21冻结后运行完整现`cargo t ace-av ace-native infra-ace-m1-arch`职责/直接消费者并集，以及`case-ace-m1-original-data-q4-941`和`case-ace-m1-original-data-diagnostics-941`实际36+16项，保原ignore/平台/性能/资源条件；这些组分别覆盖AV所属包、直接调用request_data的native record_method_tier/data_owned_tier等消费者及父准入，非全experimental/all。若新增补例使用现受管capture/SDK；不复制runner/meter，不裸spawn无界采集。整命令可分轮续真实句柄，但证据冻结身份须一致，不能冒已完。

正式工具准备先于取证；逐当前hash核AV、native、case-parent、A2X/CT/conformance及域依赖。AV identity改变的所有原source/旧required/SYN/bridge关联材料依各reader绑定真正受影响闭包重采；A2X或其他工具若因链接AV库而变更，其证书/优化证据亦实际刷新，不能假设未改源码就可复用二进制证据。identity/input/config全未变的证据复用并记理由；不改旧before/原nonce/command。最后完整只读组合与第48轮675P12F319U逐格核对，原4 AV red应转PASS，其他verdict差异须具名解释。原4MiB文档/48MiB同一LiveBudget/privateΔ128MiB、各固定组/IPC/deadline/编译认证帽和原样本全保。

F-941-06/GAP-941-AV-DATA-DIAG-01仅在V25..28、AC-16及T20/21全部真实闭环后关闭；本阶段不减少原313/ARCH/其他义务，不修其余8 scalar red，不借机解除38旧A2R守卫。T13已有warnings/完整目标继续未闭。

### 5.11 r5新增phase：F-941-08 source-CT meta请求准入早于执行与缓存命中

**授权/基面**：用户第60轮对call_f0d62e537ab843b6a2d98593aaf23bcd明确“授权941新增修复phase（推荐）”，仅F08，F07仍待答。继续唯一树`D:/autostack/.wt/lang-941/auto-lang`/`plan-941-dev`，实施基面**3b364267bcdc4bd0492f67718f55a51d640b1d54** clean。原T01..21/AC01..16/V01..28和SD01..10保留，追加T22/23/AC17/V29..32/SD11，6/23完成，不把source-CT能力修复替代原验收系统。只在auto-lang实施；材料仍统一scope，不建新Plan/分树，不提高预算或扩大语言域。

**依据与反证**：canonical `docs/specs/auto-cte/source-query-integration.md` §1要求`cte.meta.fields/v1`默认CLOSED，并保未授时普通同名函数；当前product.rs先`serve_program`/`substitute_all`，后`bind_with_facts(capabilities)`，使原meta表达式在权限检查前已变literal。第59轮原meta2缺cap仍42/AV1/publish1、暖撤cap且forbid仍hit；HasField同样复现。规范SHA eafa1082c933454dac0adeec25d70937c13c5a1a71a65e710304935c8526c208；product.rs SHA a92d96c429931fafc5414987ace0707d02f727e4da365a607dd18b4015eff0a1；其余8件基面`<scope>/f94108-r5-contract-baseline.json` SHA **2858ff0866bc8e5f3490a52cff88e3420e3c007956b2624493d5af6dc3c1ee4b**。原型raw/源hash/冷暖与普通名正控详§9第59轮；不靠静态推测或最终error认定已早拒。

**边界与方法**：在既有source-CT产品入口，对当前请求的meta能力和保护名语义作权威准入，必须早于第一次CT求值、缓存probe/hit及publish。覆盖当前有限scalar-CT及instances lane内的实际需求、嵌套/被调callee需求；不能只扫main字符串或只拦冷miss。复用core已有cap-aware绑定/诊断规则与当前AST，保source-query facts→最终HIR的原权威路径，不复制parser/solver/解释器、不用host fold、不在各backend补禁令。可用的既有接口包括core `source::resolve_with`（HashExpr可绑定、未服务拒绝属后续typecheck）和query `bind_with_facts`；**不得为方便早检查引入第二类型权威、重算全模块再冒P0或对未服务CT调用原typecheck使合法源全拒**。具体接缝由T22先做有界可行性对照：合法meta/普通同名/实例子CT均不被误拒，缺cap与保护名冲突在所有CT副作用前拒；若某现接口不满足，记录实际失败再选等价局部接缝，不能删语义或放宽预算。

主写面product.rs（必要私有helper仍在source-cte）及新**`experimental/ace/crates/source-cte/tests/meta_capability_941.rs`**。仅当经实核必须暴露/复用已有诊断接缝时，允许同树core/source-query局部接口适配，并具名追加其完整职责验证；不借机重写绑定/语法或改非meta语义。原`serve_program`/CtService/ExecKey/Memory/ManagedSink、最终fact bind/HIR与AV/native/Rust/C发射仍为原producer。每次冷/暖请求重新核权；不是把meta cap塞进cache key就算修好，也不是撤权时清空合法旧页。普通`ct_field_count`/`ct_has_field`用户函数在未启用meta时保普通调用；启用meta后的保护名冲突保持原命名错误语义，不可先执行后报错。

| 入口/状态 | producer→consumer与所有权 | 必需观测、独立oracle与关闭条件（work T22/23） |
|---|---|---|
| 冷缺meta cap | 当前cap vector/已parse或实例化的原AST→产品准入→原CtService/Sink | 原meta2与HasField缺cap均拒；没有CheckedModule、AV/hit/publish计数全0，原sink无新页。具名绑定错误来自原core语义；不能把infra/resource伪成meta错误 |
| 暖态撤权/重新授予 | 同service已成功页归原owner；每次product请求持自己的当前caps | 有cap先42；去cap且forbid仍必须拒，所有原service计数/日志/publish不变，旧合法product仍可执行；再授cap应正常命中旧页/值42/无新AV，不借清缓存掩盖遗漏 |
| 普通名/保护名 | core声明/绑定规则→同一产品入口 | 缺meta cap的普通同名函数独立得到42/meta_queries0；授cap冲突必须在CT执行/命中/publish前诊断。不能按裸名字一刀切，不能改source绕保护 |
| 合法meta/CT/实例 | FieldCount2→3、HasField缺→有→原真实AV参数与key；原pick子CT→checked实现 | 42→43、42→40、sum42、choose42/pick40等原源不变；原实例声明轴与有效执行轴分别核对，不以同值/模型证明真实执行。服务/owner正常成功路径及原资源/失败分态守恒 |
| 实际消费者 | av/query_entry.rs→AV与native(driver.rs调用compile_scalar_source_via_facts)；a2x-worker.rs共用helper→Rust/C daily/certify | 在各真实前门传当前caps并复核正/负与原产物；不是只修SDK而CLI仍放行。保原wire字段/入口/元数据格式；A2X已有仅status/detail的负例不得据文案冒typed诊断PASS，既有U边界仍保 |

**诊断/资源约束**：保持现有typed错误和各consumer既有投影边界，T22把每类core错误/实际product投影的stage/code/span在永久例中预先锁定，不按message前缀分类；新增缺cap拒绝复用既有meta-capability语义，保护名沿既有name-conflict及原聚合展示边界。SDK可核真实原因与source/span；若原CLI不提供typed code，仅核实际操作拒绝与无目标产物/执行事实，不能声称其诊断权威已补齐。本phase不扩A2X错误协议。原源/Meta输入4MiB、front-owned64MiB、CT累计10s/单AV min(2s, remaining)、现store/Job/IPC/编译认证与组合器4MiB/48MiB帽保留；新增准入工作不藏进预热或伪P/R，遍历/临时分配受原限额，合法深/大源不新增任意语法排除，超限保resource/infra分态。

**执行入口/验证顺序**：T22先在原3b364基面使新永久反例真实红，再实现；不改原meta2/meta3/pick40语料、旧批准/分母、F07 test或300ms/10s门。迭代用`cargo nextest ... -p ace-source-cte --test meta_capability_941`及实际触面的命名控制。T23在冻结实现一次运行完整`cargo t ace-comptime infra-ace-m1-arch`（原GC/32MiB512修订/性能/ignore不减），加真实消费定向面：原`case-ace-source-cte-936`14 checks、既有`worker_ct_frontdoor_958`受影响测试（组ace-a2x内命名选择），以及新增host测试（拟`experimental/ace/tests/conformance/tests/host/ct_meta_capability_941.rs`及注册；`cargo t ace-host --test-name ct_meta_capability_941`）。host只新增/受影响Case，不以该测试变动要求全ace-host/全workspace；若生产core接口实际改动，显式补`ace-source`完整组，其他共享触面同理具名追加，不按包构建依赖机械跑全experimental。

原F07 fast控制可能依调度真实红：**不得为了F08过门改其断言/阈值/负载、忽略或重标父exit**。F08自身关闭要求新增/受影响cap与消费者全部通过、完整职责除已具名F07外无新增红，原F07继续T13与整体门；自然重放绿也不把未修F07删除。原T12全目标/健康/最终联合门保持，phase完成不等941或M1交付。

正式工具先准备，冻结前后逐实际worker/helper/source/config/tool hash核影响闭包：变更身份的原wire、source/CT实际与独立Rust/C cert、daily/原优化腿必须重验，不能只更新hash接受旧命令；未变证据按原reader严格复用并记理由。不为每步修复重复全目录；本次child接线的正式观察、两项旧CT机制与其余因身份变化的证据在T23同一冻结窗口归并。旧记录/旧before不可覆写或after回填before，probe与正式Cargo feature/config要相同。合并并集/最终独立T14仍继承原goal授权，无新模型/agent/chat。

F08仅在V29..32/AC17/T22/23当前源、实际四端接口、冷暖状态与永久反控全部闭合后核销；SD11候选交最终r5/冻结SHA独立review。F07及其他旧红/旧38cert-gap/所有原未闭任务不在本phase自动关闭。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/ace-m1-expanded-acceptance.md | 原分立证据→新registry/严格父组合/真实签章谓词、旧认证阻塞与独立来源 | 可审M1总门 | AC-01,AC-02,AC-10,AC-11 |
| SD-02 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 新Q4/ARCH与原313平行AND，不插原第五列/改分母 | 旧/新身份分离 | AC-01,AC-03,AC-09 |
| SD-03 | modify | docs/specs/auto-test-case/ace-m1-applicability.md | 明列924当前cert守卫与原双轴差异/blocked ledger，不撤旧批准 | 不暗绕旧门 | AC-10,AC-11 |
| SD-04 | modify | docs/specs/auto-test-case/unified-source-syntax-admission.md | SYN签章消费范围与新ARCH原子义务证据映射/反控 | 929不冒全ARCH | AC-04,AC-05,AC-06 |
| SD-05 | modify | docs/specs/auto-test-case/ace-ca-d-bridge-certification.md | D1/D2/D3证据方向、source/typed/生命周期与总门映射 | 937不冒完整FFI | AC-08,AC-09 |
| SD-06 | modify | docs/specs/auto-test-case/architecture.md | 新只读父组合器/正式TestCase/报告schema/证据闭包与完整固定组 | 不造runner/meter | AC-02,AC-07,AC-11 |
| SD-07 | modify | docs/specs/auto-syntax/source-frontdoor.md | 条件尾部ident/body与记录构造区分；实际whole/fragment/旧oracle及语法身份按修复一致 | F-941-04既有text-record联合面应接受合法text条件 | AC-03,AC-04,AC-14 |
| SD-08 | modify | docs/specs/auto-a2r/record-text-hir-backend.md | 字段写值根与借用参数正确区分，原浅/嵌套写与Copy/参数模式保持；不扩大语义 | F-941-05真实Rust产物可编译 | AC-03,AC-09,AC-15 |
| SD-09 | modify | docs/specs/auto-a2x/incremental-and-cache.md | 沿既有Rust打印轴版本失效旧错误片段；语义键/独立C键与恢复失败分态保持 | 不让旧缓存掩盖producer修复 | AC-02,AC-04,AC-09,AC-15 |
| SD-10 | modify | docs/specs/auto-av/data-layout.md | data请求在实际失败类型仍存活处保留主诊断，再按既有worker边界投影stage/code；String API兼容、infra分态和成功/缓存域不变 | F-941-06缺码链修复，不以文本猜诊断 | AC-02,AC-03,AC-16 |
| SD-11 | modify | docs/specs/auto-cte/source-query-integration.md | meta能力/保护名称由当前请求在任何执行或命中前按同一权威语义校验，保普通名/合法meta与再授cap复用旧页；实际四端入口/诊断边界与身份复核 | 修F08先serve/substitute后bind导致的缺cap执行及暖撤权命中，不扩旧域 | AC-02,AC-03,AC-07,AC-17 |

本表为计划规范增量；实现树中的候选仍须独立review后由merge沉淀并读回，不把候选当主线已交付，不写派生索引。


## 6. 测试设计

V-01..18为原完整合同，实际分项证据见§9，未闭项不以登记或控制替代执行；r2/r3补例按下表状态。r4的V25..28已按第52轮完成工作验证；r5的V29..31已按第62轮T22完成工作层验证，V32/T23尚待；不把第59轮诊断原型当修复通过。实际既有entry见§3；新增拟治理测试`scripts/tests/test_ace_m1_arch_acceptance.py`、套件`crates/auto-lang/test/cases/ace-m1-expanded/`、必要新语言`test_cases/ace_m1_arch.rs`首例由T-02沿既有正式父入口使能，全域由T-04/05/11使能。独立oracle包括现scenario规范、902/904/reference、C三域手写reference与布局/原797 caller/provider；新增跨机制正例的手算oracle与实际source先固定，不运行emitter生成expected。实现自查→同一树独立上下文review，用户未指定其他人员/模型，不自动spawn或创建chat。

| Case ID | AC / invariant | setup / actual entry / work owner | observable oracle / distinguishing assertion | boundary |
|---|---|---|---|---|
| V-01 | AC-01 冻结分母/范围 | T-01/03，原registry+新Q4/ARCH registry，改旧批准/删必需case/缺C目标/动态交集 | 313集合hash/批准/NA/accepted/future全保；每个新required有实际映射；漏一项即拒/不可签 | 不机械131×5，不把首例28当完整域 |
| V-02 | AC-02 真实身份/严格准入 | T-02/04，新reader，重复key/错type/旧schema/跨target/错source或oracle/截断/多件/nonce错 | 同shape异owner/合法非零ctx正控，坏包REJECTED或STALE，缺原始执行收据不PASS | hash不是本机恶意历史的签名证明 |
| V-03 | AC-03 四端公共同错 | T-05，真实同源scalar/record/text/collections，E源序/短路、副作用trace、诊断负例 | 四者各对独立期望；四端统一偏1/trace颠倒仍红，不能只pairwise equal | source负例不拿工具失败过门 |
| V-04 | AC-04 真P0/精确失效 | T-06，body/method/closure/签名/layout/negative-meta/实例/CT/FFI依赖编辑 | 未读body caller真P/IO0、实际新实现可见；读依赖变化精确miss；whole后filter/fake-zero/forced-copy反控红 | 事前固定affected set、link/R单列 |
| V-05 | AC-05 恢复首操作 | T-06/08/09，真实restart→立即首好编辑；首坏→旧root运行→修复→再restore/编辑 | required R深验、restore P0、未改P0、来源位置fresh等价，无dummy/prewarm/ensure_env | typed bridge恢复不代ordinary source |
| V-06 | AC-05 坏页/原子安装 | T-06/08/10，缺source/facts/instance/Exec/Use/foreign资源页、伪cred、cap撤销、取消/迟到/发布故障 | 使用前拒或显式cold另计；旧source/semantic/target coherent；repair后下一restore仍正确 | 不从当前fallback成功推持久已愈 |
| V-07 | AC-06 资源清理/限额 | T-07，所有真实alloc域fault1..N/N+1/清fault同进程retry、shadow/out/clone/last-consumer | 原值/字节out不变、逐owner释放/无余量cleanup、合法小深图不误拒；超限分态不伪源码错 | Rust无既有quota不假报0/已支持 |
| V-08 | AC-06 在线HMR/GC | T-07，866/929同PID safepoint/旧在途链+新调用/外pin/close/512修订 | 入口1次、新旧链代际正确、last owner前保活后收敛；restart/暂停恢复冒HMR反控红 | 不新增native loader/AJ |
| V-09 | AC-07 SQ驱动 | T-08，934实际cold solver关/暖hit关/坏facts/异owner/negative翻转 | 冷红暖绿、真实bind/HIR消费前拒、未改caller生产0，与producer计数交叉核对 | 不以query日志出现代事实权威 |
| V-10 | AC-07 真实实例与AV CT | T-08，935真实choose两实例、936 sum=42与callee变43、meta/子CT变而callsite不改 | 真instantiate/AV执行边首miss>0/hit0、ExecKey/UseBinding与有效checked版本绑定；断AV执行边反控红 | 模型实例/host fold/deck旧值不能替 |
| V-11 | AC-08 ABI全方向 | T-09，797原三腿+937typed/source出口+938source导入+939资源/同步callback | 独立C header/layout/异质栈参/ctx/长度单位/原方destroy；坏ABI调用0；source/typed/mn私有/public分域 | 不是四端FFI无差别全覆盖 |
| V-12 | AC-09 独立编译认证 | T-05/09/11，完整actual+独立手写reference实际rustc/cl，各对oracle；工具/helper/header/lib改变 | 缺件/剪main/错flags/旧证书/复制reference/只编成功/晚失败不得PASS；身份不变复用有严格理由 | 原A2R旧cert-gap不免新HIR/C认证 |
| V-13 | AC-09 daily/cert分轴 | T-04/05/11，daily现场emit与strict cert核查，显式certify独立入口 | daily compiler/link/guest spawn0、准备单列；STALE不隐式重编/改golden；工具缺infra | 预构建worker可spawn |
| V-14 | AC-10 旧门/空过 | T-04/10，原脚本exit0+claimfalse、38旧cert缺、child空steps/0selected/仅群组PASS | 系统可输出valid accounting，但expanded不可签；不能读旧布尔/exit0或新证书替旧域 | 旧38真实清单保留，不减分母 |
| V-15 | AC-10/11 缺门与失败终态 | T-10，删任意required child/C/D/SQ/CT/ARCH门、隐藏infra/resource/cancel/SKIP或partial preview | 全部明确blocker/终态，preview恒不可签；JSON伪全绿无执行闭包拒；合法完整测试正控过归约 | 合成正控不进真实里程碑证据 |
| V-16 | AC-06/11 时间/内存/成本来源 | T-07/12，原全样本/scale/P90/各域帽、组合器限额、未知phase | 原门AND；缺样/错result/缩规模红；未知UNOBSERVED；重叠峰值按893，不填0/二次meter | 成本单轮不冒正式周转 |
| V-17 | AC-11 新鲜源/实际联合报告 | T-11/12/13，tracked-only输入与明确当前全部D，固定组/正式adapter真实产物→只读组合器 | report逐项映射完整、只写scope新输出、原子报告不污染旧报告；真实red列owner，只有全满足才claim | 不靠ignored附件/本机残留；不签M2 |
| V-18 | AC-12 独立完整审查 | T-14，同树独立上下文重建义务/reader/正反oracle/SD/债 | 无未知缺口默认PASS/未批延后/篡改分母；review绑定当前r5与冻结SHA | 系统pass与M1claim分开审查 |

r2修复补充验证（T16/T17已完成，见§9第22轮；最终独立review仍由T14执行）：

| ID | 不变量/AC | 实际入口与区分oracle | 边界/owner |
|---|---|---|---|
| V-19 | AC-14 条件/body消歧 | CST生产前门与手写oracle whole/fragment；if/while尾变量、比较、括号/换行，AST保变量/比较而非Ctor，原冻结eq值1 | recordal共享profile；work T16 |
| V-20 | AC-14 合法/非法记录构造守恒 | 原合法具名/嵌套/条件内带括号Ctor继续通过；错字段/缺body按独立诊断与span拒，无consumer补括号 | 原支持域，不扩future语法；work T16 |
| V-21 | AC-03/09/14 真实两consumer与身份 | 原十源正式20Case及原Rust/C认证/daily；原eq的两FAIL转真实值1，其余值/三具名trap不退化；原source不改，producer/grammar/runtime变更按身份重验 | 最终冻结ace-source+实际消费者并集；work T17，独立review T14 |

r3修复补充验证（T18/T19已按§9第40..43轮执行通过，最终独立复审仍T14待）：

| ID | 不变量/AC | 实际入口与区分oracle | 边界/owner |
|---|---|---|---|
| V-22 | AC-15 正确字段写与完整编译 | 新拟conformance::record_field_writes_941（T18注册），直接原M01/R01/R02→完整4文件→真实rustc默认/-O与各独立rs分别运行0/19/9；字段写在分支/循环仍覆盖 | 旧实现实际E0594红已存；不以字符串含mut代运行；额外模式不虚增原14模式 |
| V-23 | AC-15 值/引用/move与合法边界 | 普通Copy值根修改且旧copy不变，readonly根不多加mut；mut record参数原写回、view/同根冲突负控、text-record move拥有根与清理；现record_text_modes真实链及新窄兄弟 | 真编译与独立值/具名诊断；新增unused_mut不允许，原命名/dead_code基线另记；不改验证器 |
| V-24 | AC-02/04/09/15 cache与认证新身份 | 现provider/store构造原/2片段→新打印键读取，旧片段不能hit且当前实际执行正确；新worker current SHA下292默认成功目标（289+原3）及原14优化配置重认证/重放与全组合严格读取 | 保原逐目标/flags/完整文件集；旧原始命令不改写；全部旧red/缺门保；T19使能 |

职责组：现存`cargo t infra-ace-m1 infra-ace-cost-report infra-fixed auto-test-case ace-adapters`作为新增父判面基础；已注册并实际运行`infra-ace-m1-arch`、`case-ace-m1-q4-941`及§9列出的各扩展子组；其余适用ARCH组由T-11按现固定机制使能，尚无证据的不冒可执行（不复活tm/叶目录发现系统）。完整真实证据并集由registry逐行引用现`case-ace-m1-*`各具名组、`case-a2r-hir-record-text-902`、`case-a2r-hir-collections-904`、`case-a2c-hir-scalar-921`、`case-a2c-hir-record-text-925`、`case-a2c-hir-collections-926`、`case-ace-unified-source-929`、`case-ace-ca-d-scalar-937`及新上游正式组，T-01列成显式名单而非shell wildcard执行。source/query/CT/ABI/缓存与HMR的原完整职责/专项由真实触面加`ace-source ace-semantics ace-comptime ace-abi ace-foreign-contract ace-a2c-abi ace-artifacts ace-av ace-native ace-a2x ace-lowering`，不得仅跑新增治理脚本自证实际语义。

固定组和专项原ignore/平台/资源/Job/输出帽保留、执行/排除/未执行/首次红/重放绿分别列。性能/压力与其他正式采样错峰，merge冻结提交一次职责并集，输入身份未变的有效证据可复用且逐项写理由；不为纯文档跑构建、不叠无因全experimental/all，周期`cargo t all`只按due在主检出单实例。新增Rust test在autotests=false包要显式Cargo target登记且确认被完整组选中。

r4 F06补充验证（T20定向12项见§9第50轮，T21完整职责1493/当前源408与原data52/严格联读及156反控见第52轮；phase工作完成，最终独立复审仍T14）：

| ID | 不变量/AC | 实际入口与区分oracle | 边界/owner |
|---|---|---|---|
| V-25 | AC-16 原4 typed诊断 | 已注册conformance::data_diagnostics_941经typed driver及真实av-worker case-serve→原M09/M10/M11/R09，code依次source.not-callable、verify.immutable-write、verify.mut-conflict、source.type-mismatch；与原共享前端primary核对，worker pipeline stage为frontend，消息改变不影响code | 不改原源码/expected；T20使能，未修空code必须实际红；work |
| V-26 | AC-16 分类失败边界 | 实际front-door拒绝/缺entry的lower拒绝、typed verify/encode/ABC错误代表态，以及实际不可写/文件占用store的I/O；按错误种类而非rendered前缀分类 | 真实code/stage与原主诊断一致；infra不冒4源码负例；若某内部失败不可从合法源到达，用同模块具名类型边界控制，不声称真实worker执行该态；work T20 |
| V-27 | AC-16 合法与旧调用兼容 | 原M01/R01/R02合法record各0/19/9、data精确trap/trace、owned/core代表原入口，typed API与String兼容入口分别经过同实际pipeline；现直接native消费者调用旧API | 成功/guest trap/typed值/trace与store恢复不变，旧签名仍可编译；不更改源profile/caps/缓存轴，不为负例绕旧String→prefix路径；work T20/21 |
| V-28 | AC-02/03/09/16 当前生产者严格联读 | 冻结完整ace-av/ace-native/infra组、原data36+诊断16、工具准备后pin闭包和受影响原始wire重采，再原reader全组合 | 原4真实FAIL转PASS；其余8 scalar red和缺义务保持，旧code空/错code/错source/旧worker/错nonce依旧拒或非PASS；不只跑合成packet测试；work T21 |


### r5 F08补充验证（第62轮T22完成；T23消费者/冻结验证待执行）

| ID | AC / owner_stage | 入口与独立观察 | 可区分反例/覆盖边界 |
|---|---|---|---|
| V-29 | AC-02/07/17；work T22 | 新meta_capability_941原meta2/HasField冷缺cap、嵌套/被调callee有限需求；原SDK typed原因/来源与service计数、sink实际publish观察 | 必须在AV/cache probe/hit/publish前拒；原旧实现会42/AV1/publish1而红。不能用最终error、无target文件或伪0代执行前拒 |
| V-30 | AC-05/07/17；work T22 | 同service有cap成功→去cap+forbid→旧product仍执行→再授cap+forbid重取；每步原日志/计数/页与当前请求caps | 去cap不能命中旧页，失败不破坏原合法页，再授cap正常hit/无新AV。仅冷态加guard、cache-key独加cap或清空缓存的错误修法会红；不是process restore证明 |
| V-31 | AC-03/07/17；work T22 | 两个普通同名用户函数无meta cap各实际42；授cap保护冲突早拒；合法FieldCount2→3/HasField缺→有、普通CT/choose/pick | 保42→43/42→40和真实AV/key/negative字段事实；名字一刀切、只拦main、先AV后报保护冲突均红。原源码/profile不为修复改写 |
| V-32 | AC-02/03/09/17；work T23 | 冻结完整CT职责/治理、原14CT source与真实AV/native/Rust/C worker四向消费、Rust/C四CLI操作、当前changed身份raw/cert/daily/原优化及child观测 | SDK与实际consumer都拒缺cap，合法原oracle守恒；命令/真实终态/旧F07红与8 scalar红/其余U守恒。无typed code的A2X拒绝不靠message充PASS；不以group绿或新worker hash顶严格读取 |

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 / closing evidence |
|---|---|---|
| AC-01 | work | V-01完整旧313与新固定Q4/ARCH集合/来源/coverage映射/批准守恒，无动态交集或自动NA |
| AC-02 | work | V-02真实producer→原始执行→严格父reader→义务闭包可核，所有身份/shape/路径/版本负控及合法正控成立 |
| AC-03 | work | V-03完整冻结共同源域四端各对独立oracle，负例/trace/trap保真，公共同错可抓 |
| AC-04 | work | V-04真实函数/实例/绑定P0/IO0与精确依赖失效、跨目标隔离，no hidden whole/cache value |
| AC-05 | work | V-05/06无预热source恢复首好/首坏修复/再次restore、各required页/来源/权限/原子安装正确 |
| AC-06 | work | V-07/08/16原预算/时间/cleanup/lease/同PID HMR/收敛与组合器本域帽，不虚构观测 |
| AC-07 | work | V-09/10 SQ事实真实驱动/实际实例/真实AV CT及完整传递依赖，由稳定D链实测而非模型 |
| AC-08 | work | V-11原797与937..939全有限方向/资源事实、独立C布局/真实call及错误调用0，不冒未支持域 |
| AC-09 | work | V-12/13完整独立实际编译运行认证与daily spawn0、strict reuse/STALE，旧新证据不互替 |
| AC-10 | work | V-14/15全部门AND与38旧认证差异显式处理、旧守卫不绕、空选择/缺门/失败不能claim |
| AC-11 | work | V-15..17跟踪输入自足、真实联合报告/完整收据/原父预算与实际verdict：不全绿则明确red/owner，绝不写M1交付pass |
| AC-12 | review | V-18独立逐AC/SD/遗漏/差异/债/全部reader/warnings/fmt审查，系统交付与里程碑claim分别绑定当前r5/冻结SHA |
| AC-13 | merge | 冻结职责并集/触码漂移重验、SD沉淀读回/交付归档、全部输出对账及scope/guard清理；真实M1未绿仍标未绿 |

AC-01..09的语言/机制实测须所需D真实就绪，不能仅靠合成reader正控关闭。AC-10/11允许真实报告诚实red来证明验收系统正确，不能据此豁免前述应执行证据或声称M1已过。若底层新缺陷导致某项实际验证无法关闭，留具名阻塞/owner/永久反例，不伪造work完成。

**AC-14（r2，owner_stage: work/review）**：§5.8条件解析修复由生产CST/whole/fragment及真实AV/native原例验证；V19..21全部具名通过且合法/非法记录构造和既有诊断边界守恒，原源/值/预算不变。缺原消费、只改旧oracle或consumer补括号不能通过。

**AC-15（r3，owner_stage: work/review）**：§5.9字段写修复使原3Rust完整actual与独立reference真实运行0/19/9，默认与-O兄弟验证区分值根/借用参数，不引新unused_mut或扩大view写权限；旧print轴片段不被新producer命中，当前worker/证书身份实际刷新。V22..24全闭才核销F-941-05；该phase不能代941其余任务或T14独立review。

**AC-16（r4，owner_stage: work/review）**：§5.10 typed data失败链真实保留并透传共享primary诊断，既有String调用兼容，stage/code不从message分类；原4 worker负例按原16诊断合同PASS，合法record/owned/core及非前端/infra代表态守恒，完整职责/直接消费者与当前受影响证据严格联读成立。V25..28全闭才核销F-941-06；保持原分母/预算/其他具名red，最终T14须按当前r5与冻结提交独立复审。

**AC-17（r5，owner_stage: work/review）**：§5.11中source-CT当前请求的meta能力与保护名在执行/命中/发布前按既有权威语义校验；冷缺cap与暖撤cap早拒、旧合法product/缓存不破坏且再授cap真实hit。无meta cap的普通同名函数仍正常、合法meta/实例/CT与原四端产物不变；V29..32/当前身份和影响闭包齐备才关闭F08。原source/expected/预算/分母/旧red不改，F07不由此授权修复，最终T14按当前r5与冻结SHA独立复审。

## 8. 执行步骤

| ID | owner_stage | 依赖/写面 | 具体路径/可验证结果；AC/验证 | 状态 |
|---|---|---|---|---|
| T-01 | work | §4原goal全链授权；只读当前Specs/收据 | 冻结main/929/937/934..939实际D与reader、旧313/38清单、完整语料/原预算/具体固定组/全部owner，scope登记；AC-01/10，V-01/14 | [ ] |
| T-02 | work | T-01，私有P | 首例同源文件/独立oracle/四目标AutoDown-TestCase从本步入跟踪并走既有正式父入口，不等收尾才补；有界全链：真实一个scalar四端+原旧核算/SYN/937收据→strict reader→registry→报告；四端同错/缺C/旧claimfalse控制红，冻结schema/required闭包/预算无自指/不spawn；不成立needs_replan；AC-02/03/10 | [ ] |
| T-03 | work | T-02，新registry私有面 | 新ace-m1-expanded registry逐source/target/机制/原格位与义务/组/owner/条件映射，完整覆盖而不改旧批准；AC-01，V-01 | [ ] |
| T-04 | work | T-03，必要H短窗 | 新只读ace_m1_arch_acceptance.py与域reader适配/严格身份/路径/版本/原始终态/旧认证差异；AC-02/09/10，V-02/13/14 | [ ] |
| T-05 | work | T-03/04，Q4已有域稳定API | 正/负同源四端与独立oracle/reference/完整文件集认证、跨目标身份/daily分轴；AC-03/09，V-03/12/13 | [ ] |
| T-06 | work | T-04/05，929/各实际required来源页稳定窗口 | 真实P/M/R/IO计数/依赖变化/多函数方法闭包、source恢复首操作/坏页/原子安装反控；AC-04/05，V-04..06 | [ ] |
| T-07 | work | T-06，原资源/HMR接口与正式窗口 | 原预算/cleanup/fault/lease/同PID safepoint/原样本P90/peak与新组合器资源帽；AC-06，V-07/08/16 | [ ] |
| T-08 | work | T-04，934/935/936已审稳定接口；最终实际D | SQ/真实实例/CT执行/事实依赖及required恢复/四端实证适配，不修改其语义实现；AC-05/07，V-05/06/09/10 | [ ] |
| T-09 | work | T-04，937/938/939已审稳定接口；最终实际D | ABI原三腿/全source-typed方向/资源/ctx/callback/独立C认证与source恢复；AC-05/08/09，V-05/06/11/12 | [ ] |
| T-10 | work | T-04..09 | 总门AND/完整required/preview/38旧cert差异/缺门/伪PASS/空steps/迟到及各失败分态治理反控；AC-10/11，V-14/15 | [ ] |
| T-11 | work | T-03..10，公共H单写窗 | 从T-02首例扩齐全部TestCase/独立oracle、必要父adapter/SDK/所有reader、scripts/test-groups.toml新完整组/测试target注册；AC-01/02/09/11，V-12/13/17 | [ ] |
| T-12 | work | T-11及T-18/19/20/21/22/23；最终D934/935/936/938/939与已交929/937；正式采样窗 | 冻结真实完整证据并集/原样本规模、fresh/hit/restart/first-edit/资源/真实独立认证→组合报告，缺项保持真实red；AC-01..11，V-01..17 | [ ] |
| T-13 | work | T-12 | SD/coverage守恒/全部owner接口/健康冻结/遗留cert与新缺陷交接；明确系统交付与M1是否可签，不能藏in-contract未执行；AC-11 | [ ] |
| T-14 | review | T-01..13及T-16..23完成，停止实现写入 | 同树独立上下文重建source→producer→consumer→oracle→gate全链/反控/所有SD与债，不采信摘要；AC-12，V-18 | [ ] |
| T-15 | merge | T-14 pass与§4已有全链合入授权 | 冻结职责并集/漂移复验、SD沉淀/主线读回/归档、所有输出scope与树guard清理；AC-13；M1未绿不改成绿 | [ ] |
| T-16 | work | 用户r2直接加phase授权；§5.8；当前941唯一树 | 确认生产CST/条件诊断与旧oracle共享歧义，修复既有条件/body-vs-Ctor边界并补whole/fragment与合法/非法Ctor永久回归；AC-14，V-19/20 | [x] |
| T-17 | work | T-16，实际provider/grammar/工具身份冻结 | 原十源20Case/同源Rust-C认证逐oracle重验与原失败闭环；完整ace-source及实际AV/native消费者职责并集、必要ace-syntax及身份变化刷新，SD-07；AC-03/04/09/14，V-21 | [x] |
| T-18 | work | 用户r3新增phase授权；§5.9；a236a44基面、原失败与Spec | Rust lower.rs值根/参数模式修复；沿provider既有print版本轴失效旧片段；新record_field_writes_941.rs并conformance注册，原3源+独立reference/default与-O、分支/循环/readonly/mut/view/move兄弟；AC-15，V-22/23，SD-08/09 | [x] |
| T-19 | work | T-18冻结提交；实际worker/conformance新身份 | 完整ace-a2x与相关治理组、原3证书闭环/全部受身份影响认证及14原优化配置刷新、旧print缓存miss见证与当前组合读回；F-941-05关闭、SD-08/09交接；AC-02/03/04/09/15，V-24 | [x] |
| T-20 | work | 用户r4 F06内修授权；§5.10；330c25344同树基线/原4失败 | data_driver具名typed错误边与旧String API兼容；真实worker按类型投影而非文本；新data_diagnostics_941永久注册，原4/合法/失败与分类边界；AC-16，V25..27，SD-10 | [x] |
| T-21 | work | T-20冻结提交；正式准备后的真实工具/依赖身份 | 完整ace-av/ace-native/infra及原data36+诊断16门；受AV/其他实际hash变化影响的source/wire/证书闭包真实重采，原strict联读与675P12F319U逐格对账；F06具名关闭、SD-10候选交接；AC-02/03/09/16，V-28 | [x] |

| T-22 | work | 用户第60轮F08内修授权；§5.11、3b364267b基面/第59轮反证与Spec | 先永久反例红与准入接缝有界可行性，再修source-CT产品每请求meta/保护名准入；新meta_capability_941冷/暖/撤销/再授cap、普通名、实例/嵌套兄弟及失败/资源分态；只用原producer/权威规则；AC-17/V29..31/SD11 | [✅ 已完成] 第62轮be77185d5；原实现9例1P8F，当前9例及原受影响产品共17P、core CT10P；完整职责/预算与真实消费者仍归T23 |
| T-23 | work | T-22冻结；正确Cargo feature/config与真实当前工具先prepare | 完整ace-comptime/治理与具名真实consumer联合；原14CT/四CLI与受实际身份变化影响的raw/cert/daily/优化严格重验，child/原CT机制同窗采；守恒其余red/U、保F07真实终态；SD11候选与最终T14交接；AC-02/03/07/09/17、V32 | [ ] |

T-18/19已完成（§9第43轮）；T20/T21已完成（第52轮），当前回到原完整T-01..13。 r5接续T22/23修F08；F07仍独立待批，不预占/勾选任务。T-01/T-02/T-03/04不等全部D，可独立推进；最终联合T-12必须全部真实相关D齐。共享接口稳定可提前接单个切片，不能把“方案已写/审过”当正式交付。用户已授权work/review/merge全链并恢复目标；继续原T-01/T-02等完整任务，尚未关闭的验收保持未完成。



## 9. 复审记录

2026-11-05 stage:new / PLAN-941 r1 / outcome:pass（仅规划静态就绪） / next:后续指令后work T-01。仅规划：保留旧批准/当前更严守卫，明确38旧A2R cert-gap阻塞，不把新HIR/C认证替旧域；真实同源Q4、实际P0/恢复/资源/HMR/SQ/实例/真实AV CT/ABI各有owner、closing oracle与反控；不得由checker正控推M1已通过。静态核查已通过：0..10节、T15/AC13/V18/SD6及交叉引用/owner、现存固定组与拟新组标注、新链接、38旧认证项、22输入hash、D929/D937祖先/唯一编号和文档空白/范围；详本包基线报告。所有V与work/review/merge均未执行。

### 2026-11-06 work 首例检查点与 F-941-01（尚未完成 T-01/T-02）

- 授权为本节 §4 的 `/goal 完成计划941`。主检出基面 `3b09e32b016ceafed97c4415d021626474fcadf7`；唯一实施树 `D:/autostack/.wt/lang-941/auto-lang`，branch `plan-941-dev`，检查点提交 **`91968fd403d0bf0c7bf2f5a4e002ae50d44440a6`**。组内只读 auto-down detached `58ea33f699efa5cb1194473a2d45dcec51006842`。系统 `bash` 误路由 WSL，建树完成后 Windows Git 无法识别 `/mnt/d/...` gitfile；已分别用原仓 `git worktree repair <绝对路径>` 修复双仓登记，Windows 下两树 `git status --short` clean，未创建任何链接。后续 bash 必须显式 `C:/Program Files/Git/bin/bash.exe`。他方文档 WIP 未暂存/提交。
- scope=`D:/autostack/plan-artifacts/plan-941/`；`.retention/policy.json` 已登记 owner=PLAN-941、purpose=入口调查/真实首例/显式认证/临时材料、status=in-flight。`entry-survey.json` SHA256 **`7474b407b8c1748e338663a41b345be06bf6f2daf57ad10609c7ac1bb2274b89`** 固定486输入指纹、49现有组、旧313 required/134 NA/32 accepted/45 future与38旧a2r认证缺口。934/935已归档（区别于规划基线），936 executing1/14、938 executing6/13、939 drafting0/14；最终D936/938/939尚未齐。T-01完整原子义务/所有原负载/预算到case/check映射仍未关闭，故 current_step 仍0，不以输入文件数量冒完整分母。
- 交付到实施分支：`ace-m1-expanded/` 首例Q4-001（相同源码 `leaf(41)=42`，原934已审golden字节原样复用），两份独立手写Rust/C reference；正式 `case-ace-m1-q4-941`；新增 `infra-ace-m1-arch` 的14项严格证据读取正反控（重复key/未知或缺键/错误类型/整数溢出/非有限数/截断/尾随/UTF8字节帽/48MiB分配前保守预留/累计预留/路径逃逸/设备及重解析点/实际产物重hash）。组合器现仅具备读取原语，直接执行明确exit2拒发验收结论，**尚无完整registry/reader/reducer，不是验收系统交付**。
- `powershell -NoProfile -File scripts/verify-ace-m1-852.ps1 -Scope acceptance -OutDir D:/autostack/plan-artifacts/plan-941/legacy-accounting` exit0：核算有效，required集合hash仍 **`35ad43aae27ffb2d2c9a162c3dd5b99f971e4cd7b2ed36f4fe747a5634bd770e`**，selected/executed=0，38 cert-gap，`can_claim_m1=false`。未将exit0算实际执行或M1通过。
- 首次 `cargo t case-ace-m1-q4-941 --out .../q4-first-run` **exit1保留**：4个动态check均PASS，但运行期间本会话补`.gitignore`使golden manifests变为可见输入，触发`code/config/inputs changed during execution`。准备/冷编译与运行分列：父工具准备161.77s，实际4check窗口2194.28ms，整命令373.16s。证据失效是本会话冻结纪律问题，不归环境、不充PASS。
- 冻结后 `cargo t infra-ace-m1-arch case-ace-m1-q4-941 --out .../first-slice-frozen` **exit0**：14治理case + 4日常check全部实际执行，无条件排除，整命令4.312s；`results.json` SHA256 **`aa07ffc7ed22d1c591fa13370606dc5ff45fbbee0c0a13a8ca5fc051cf7274ba`**。补路径非法字符控制后仅重跑受影响 `cargo t infra-ace-m1-arch --out .../admission-final` exit0、14/14，Q4的源码/合同/golden/工具未改，复用前一有效4check证据；`git diff --check` exit0。未执行全仓或完整ACE组（尚非最终冻结门）。正式case父入口使用 `%TEMP%/tmcase-we_lycf6`，原始runtime已由验证复制后删除，`artifact-copy.json.runtime_cleanup_complete=true`；副本仅位于scope内。默认正常构建路径仅本树`target/`、`scripts/full-regression-driver/target/`、`experimental/ace/target/`，无第二外置target/安装目录。
- 显式认证沿**现有** `a2x-worker` `certify`/`certify-c` 与 `process_capture.capture` 受管Job顺序运行，未在新组合器启动工具或另造runner/meter；完整现场actual与独立reference均实际编译运行，各得`value:42`，Rust进程exit0、0.637s，C进程exit0、2.568s。原始stdout/stderr/命令+meter分别为scope的`q4-001-cert-{rust,c}.{stdout,stderr,command.json}`，产物在同名目录；命令收据SHA256：Rust **`4dd3912220436e36b54b60cdbb92eb7cdad7591a586ed9d497b40c4d09dfac29`**、C **`59b58898de585bd1e3d240166bd3c3299c9ad647b0e816abf2262cd0bacfa79f`**。不把worker自报PASS当父侧准入。

**F-941-01（阻塞 T-02 / AC-02 / AC-09）**：真实 Rust `case.cert.json` 经标准 `json.loads`及本包`EvidenceScope.read_json`均拒绝，`Expecting ',' delimiter: line 36 column 3`。文件SHA256 **`63a49eab9a4d8f9a0cf9b1bc3ee7947dcf481a1beaf8f28ac91b51484f52c8ad`**；尾部实际为 `"capabilities": [ ]` 后缺逗号，然后 `"execution_config": null,` 后紧接对象闭合。根因已定位 `experimental/ace/crates/a2x-rust/src/cert.rs::CertDraft::to_json`：capabilities与execution_config间缺`,`，最后字段多`,`；共享scalar/record-text/collections序列化路径均受影响。`valid_draft_passes_and_serializes_stably`及worker现有cert测试只验typed draft和重复序列化字节相等，未将输出解析成JSON，故历史绿不证明其可被严格reader读取。**合法C正控**：真实C证书可严格解析，schema=`ace.a2c-hir.scalar.cert/1`，SHA256 **`a255c5c3c6a0ea4ba35e7119af26984aa3ecda39ed0d43667cdfe7f62e7d6bcb`**。

F-941-01 finding包=`.../certificate-json-finding.json`，SHA256 **`4c61f09c3b78edcdf454a04129000e0f3abebb31ea0e21c8d02492885a65c8ca`**。最小拟修复（未实施）=在原producer修正两处JSON标点，补真正解析序列化结果的永久反例/正控（`execution_config=None`及collections非空配置，含能力列表空/非空）；在独立修复包按所属`ace-a2x`完整组及必要消费者核验，再对同一Q4首例重新显式认证/严格读取，保C正控。不得在941组合器修补坏JSON、跳过证书或把新证据顶替旧38格。依据§5.4，已归档能力新缺陷须具名独立修复流程；是否授权该新增producer修复包见§10-4。

stage: work | plan_id: PLAN-941 | plan_revision: r1 | outcome: blocked | worktree_path: D:/autostack/.wt/lang-941/auto-lang | branch: plan-941-dev | base_commit: 3b09e32b016ceafed97c4415d021626474fcadf7 | code_commit: 91968fd403d0bf0c7bf2f5a4e002ae50d44440a6 | task_ids: T-01/T-02（均未关闭） | evidence: 上述真实worker证书与严格JSON反例、14+4冻结绿证据 | blockers: F-941-01独立producer修复授权；最终另需D936/D938/D939 | next_responsible_role: user（仅决定独立修复范围，941原授权继续有效）

stage: goal | plan_id: PLAN-941 | plan_revision: r1 | round: 1 | from: work | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-941/auto-lang | commit: 91968fd403d0bf0c7bf2f5a4e002ae50d44440a6 | next: user


2026-11-06 waiting audit：上一goal轮有实际进展（实施检查点91968fd40、14+4冻结验证、真实证书缺陷复现）；本轮复读§9/§10及当前工作树/主线记录，未见§10-4的用户决定或F-941-01独立修复交付。相同阻塞连续第2轮；未运行测试/构建、未推进依赖步骤，原待答问题继续有效。

stage: goal | plan_id: PLAN-941 | plan_revision: r1 | round: 2 | from: user | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-941/auto-lang | commit: 91968fd403d0bf0c7bf2f5a4e002ae50d44440a6 | next: user


2026-11-06 blocked audit（连续第3轮同一条件）：上一轮为no progress（仅待答状态复核/簿记，没有已确认活跃进程可轮询，不计verified wait）。本轮重读§10-4，仍无独立producer修复授权；主线CertDraft::to_json仍含已定位的两处JSON标点缺陷，实施树clean且HEAD仍91968fd40。按r1 §5.4/§8有界首链门与既有待决定记录，不能修补child文本、跳过证书，亦不能在未授权独立修复时扩写原producer；T-03及后续依赖T-02成功，不具备可继续执行的独立入口。未重跑测试/构建。连续3轮阈值满足，goal转blocked，Plan仍executing、不归档，树/branch/scope保持在途。解阻动作=用户授权§10-4最小独立修复包，或原owner交付修复及可严格读取证书后恢复目标。

stage: goal | plan_id: PLAN-941 | plan_revision: r1 | round: 3 | from: user | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-941/auto-lang | commit: 91968fd403d0bf0c7bf2f5a4e002ae50d44440a6 | next: user

2026-11-06 解阻授权：用户答复“授权独立修复并继续（推荐）”，已批准§10-4最小独立producer修复全链；新PLAN-951承接F-941-01。941继续等待D951与严格证书重放，不改变r1验收或38旧认证守卫。应用goal仍显示blocked，按用户明确“并继续”直接执行授权工作，不另建goal。


2026-11-06 依赖修复已交付：PLAN-951 archived6/6，delivery=fec8a9602413847c681c6944d490d7a39e18e272，真实Rust/C证书均可由原941严格reader解析；全ace-a2x183/183及canonical source读回已通过，scope/tree/branch已清。F-941-01的producer缺陷已闭；941尚须同步D951、重建本树worker、重放证书后继续T-01/T-02（未预先勾完成）。用户独立修复并继续授权持续有效；旧38格认证及D936/938/939真实交付条件不变。next:work。


2026-11-06 work恢复/依赖闭环：941分支rebase master，`git range-diff 3b09e32b0..91968fd40 master..HEAD`为1/1 `=`，旧91968fd40→当前 **225ba6561bd571be6e69e56f20bdd2037d5a1c88**，本包首例/reader/tests补丁不变；新基面包含D951修复。树clean。当前树原a2x-worker release重建exit0、43.52s，随后原同源Q4-001 fresh显式certify/certify-c（现有受管Job，完整actual+独立ref各value:42）exit0，Rust682.25ms/C2772.64ms；原941严格JSON reader直接接受、source/ref及actual逐文件hash校验通过。Rust新cert hash `9fb138828f59c9776f327c047264d258b8be038c4f1f99d0f2e9329d65812c1d`，命令hash `2649ea02d7593807175638c6ef9946603a58ea02b8d4f4b806c3947f5dda2f66`；C正控新cert hash `ee9d3104298d979d0628508747967533909f99b14563603e869f52d81e1d7ea9`，命令hash `a9d3774b5daab54fda016b851ea1163c558b1d2321cbf681e94aff5ac1c58912`。材料仅原941 scope的worker-after-951.log / q4-after-951-* / after-951-summary.json，登记状态仍in-flight。F-941-01实际关闭，T-01完整映射与T-02组合闭包其余部分仍未完成，current_step0/15，next:work，不进入review/merge。

应用自动目标仍显示blocked，工具不提供active/resume状态写入；用户“并继续”授权已据实执行951全链与上述941本树重放，未另建goal或声称应用goal已恢复。按auto-plan-goal的inactive恢复规则，后续自动续轮需用户`/goal resume`；这是目标运行控制，不是重新索要实施范围授权。


### 2026-11-06 恢复目标后的执行检查点（仍在work，非完成移交）

用户按恢复指令激活goal，应用objective显示`resume`，按紧邻原指令解析为继续完成PLAN-941；不是替换成“只恢复状态”的新目标。先前目标轮有实际进展：D951独立修复/归档/清理与941本树严格认证重放。当前应用goal active，原授权继续有效，无需再问范围。

检查点提交 **b002a738ae5a3a99a45c0440f1442287417dd993**（941树clean）：

- T-01实际清单盘点：冻结显式34个现存TestCase组，原`cargo t <明确名单> --discover-only --out .../fixture-inventory`成功；另对存在但无独立固定组的892 suite调用原父入口只读盘点，总35suite、**511 atomic checks、executed=0**，不是511执行通过。初始尝试猜测`case-a2r-hir-892`被原固定组入口拒绝（两次纯前置校验，无测试），已查实际配置确认其无该组并保留未注册定位，不伪造组成功。原scalar61及资源/方法/机制原子义务到所有实际source/check/owner的完整映射仍未闭，T-01不勾完成。
- T-02必要H观察边：新增原`auto-case identity-inventory`维护入口→**test-case-inventory/2**，共用原parser/resolver与执行时同一`fingerprint(spec/effective check)`，从当前源码独立输出contract/check身份，不从child PASS反推期望；原`inventory`与/1保持输出不变，固定driver仍用/1。35suite版本化元数据及原始命令均在`.../identity-inventories/`。Q4/SYN/937的/2删新增两键并投影schema回/1后与原/1逐值完全相同；在隔离scope复制首例，**只改expected 42→43**，正式/2的source hash保持、contract hash必变，AV/native oracle hash变而Rust/C emit oracle保持；原始/2 contract pin又与实际Q4执行report.identity.contract相等。控制件=`.../identity-oracle-control/`及`identity-oracle-control.json`。此为身份观察元数据，不是新parser/runner/语义分析器。
- 正式真实首例事实：`cargo t case-ace-m1-q4-941 case-ace-unified-source-929 case-ace-ca-d-scalar-937 --out .../t02-real-inputs` exit0，实际4+11+6=**21检查全部PASS**，整命令54.794s（准备/原命令窗口由原command/meter分别保留），`results.json` SHA256 **18626a7539acbb0641da5dd5ea5474caeecd2d7b1faa81d0ecfe427d21d0d5fa**。所用原runtime分别tmcase-w3qpj_0a/pjndfuxa/v7ny1t6q均已由原驱动验证复制并清理，副本在scope内。SYN这些parse/AV腿与937有限桥首例不冒完整ARCH或D938/939。
- 新只读域reader `scripts/test-governance/ace_m1_legacy.py`：对独立父冻结binding/applicability/source-manifest/批准hash重建全部524原格位、313 required集合及38旧A2R cert-gap，逐字段/类型/规则/分母/原报告严格对账；只消费/2零执行accounting模式，执行来源始终**UNOBSERVED**，不借旧parent_admission/selected/exit0/claim布尔授信，旧认证缺口如实false。原始真实/2报表SHA25640f006ca303e424abab473e5bc1cfd9ad9af49caa8a8c1a5a074116dbd8bbd18；这条实际组合读取的全根累计保守预留7,362,912字节。通用EvidenceScope增加**跨授权根共享48MiB预算**与有界raw读取，不得每child重置额度。
- `cargo t infra-ace-m1-arch --out .../legacy-reader-controls` exit0，**23/23**实际通过：既有14输入严格控制+1跨根活数据预算控制+8组真实PowerShell /2输出突变控制。实际/2由原script在隔离scope新生成；mutation拒claimfalse→true、缺/重原格位、required/cert-axis/token改写、bool/int/float混淆、分母/unknown/schema/future/accepted改写、parent/selection/执行计数冒证据、批准文档漂移。reader准入阶段以Popen禁用断言证明不spawn。raw results SHA256 **b8f3a806bcf561cf3e74a0dd5b3da20594643a9195f8feb83336e82550fc8aaa**。
- 健康：改动两Rust文件定向rustfmt --check与git diff --check exit0。原parent编译存在131条core+5条lang warnings（含summary行共138个warning header）；与开工前admission-ytb2d4sw原父构建日志逐header Counter对比完全相同，新增0、触及metadata文件无warning。不以本包顺手改无关存量warning；原日志与归因保留。编译元数据入口87s，最终仅格式/空行调整不改变执行/parser/fingerprint算法；有冻结后affected group门归最终阶段，不重复全仓/ACE全量。

下一入口仍T-01/T-02：纯组合器尚缺完整registry、Case/证书/原始command严格关联与版本化报告/正反归约；当前直接CLI仍exit2拒发结论，不能宣称系统交付或M1通过。真实首例还暴露**观察适配**待接线：ABI parent的toolchain axes仍写通用`abi/interop-v1`，而其实际执行已严格验证active_recipe=CA-D1；request-samples主要保计量，未持有供独立组合器重验的完整request/response身份。A2X daily也尚未保原始请求/终态收据。这是本包T-02/T-04已授权的必要父观察边：在原owner输出窗口保留真正请求/响应/来源/nonce/工具身份并保原分态/预算，不由新组合器改写旧标签或模拟后端。不得仅凭21个父PASS关闭strict证据闭包。T-01/T-02都仍未完成，counts仍0/15；下一步按真实wire与全部状态路径逐writer/reader接线，后续T-03仍依赖T-02成功。

### 2026-11-06 T-02 原父执行器观察边与Case身份闭包检查点（仍0/15）

实施提交 **`1b6e524b5a3218199fb788b89b1d9fca715b17d5`**，唯一941树/branch未变、提交后clean；仅完成首链中的可核验单元，**不是T-02 pass，不进T-03**。

- 在既有`execution_trace`增加显式`AUTO_CASE_WIRE_EVIDENCE=1`观察模式，沿原AV、native冷会话、A2R/C daily、ac/syntax与ABI父执行器保存原请求载荷/响应字节及实际退出码；PID+序号/create_new保不覆盖，START hash链接TERMINAL。未写PASS，提前失败保缺终态，默认模式不写文件/不额外计算worker哈希。原进程/Job/预算/错误路径继续拥有执行，不添runner/meter；native persistent rounds与其余机制路径尚未接线，不冒全覆盖。
- ABI工具身份现记录实际active recipe，stale从当前有效合同决定ABI family/recipe，不能靠旧报告echo换recipe。真实937报告6/6 FRESH（exit0），仅将第一row的recipe篡为interop后恰1 STALE（exit1）；收据`.../abi-recipe-stale-controls.json` SHA256 **`0ed88e0958c71e5b1e2eff3c91d0ccc4c12c22559e264379db9c48b50a4cbf44`**。
- 新`ace_m1_case_evidence.py`只做Case `/2`当前identity/全check集合/summary/复制清单/原始字节终态关联。原绝对路径只作名字映射，禁止回读已删除临时root；逐引用核对bytes/SHA，显式scope/reparse及4MiB/48MiB共用原门。reader输出`execution_judgement=UNOBSERVED`，不把schema/child PASS当执行归约；nested domain内容、counter语义、oracle与证书闭包仍归后续域reader。当前inventory/worker哈希必须来自caller冻结的正式输入，不能由child提供。

定向验证（未减原样本/预算；完整模块并集仍归最终冻结门）：

1. `AUTO_PLAN_ARTIFACTS=.../plan-941 cargo t auto-lang-core --test-name test_cases::execution_trace::tests --out .../wire-recorder-tests` exit0、**5/5**，无条件排除；保无效UTF8/NUL原字节、非零exit、缺终态/不覆盖与超cap不截断。结果SHA **`fca60682de63c3258680f27a20eed4dfc138d1a7b988a3d470b9519f1ce7b693`**。
2. `AUTO_CASE_WIRE_EVIDENCE=1 cargo t case-ace-m1-q4-941 case-ace-unified-source-929 case-ace-ca-d-scalar-937 --out .../wire-real-inputs` exit0、**21/21**（4+11+6），无条件排除，整命令51.410s；原始Case runtime均复制核验并clean。结果SHA **`4ef669e2df472844fb1c034fc239fda4b1958262c7e14d9ad9bc6a6708eae9e3`**。对同一正式identity-inventory `/2`及当前原worker实际文件SHA，独立调用新reader：21check、**26组**原请求/终态关联有效（4+11+11），保`UNOBSERVED`，每scope保守reservation分别1,312,452 / 2,480,645 / 2,339,685 bytes；摘要`case-wire-admission.json` SHA **`3cc79f795d1b5d7cd49c24df0b5817ff06df8225c2ad7fd6127e44d61fb2cdd7`**。
3. `cargo t ace-host --test-name abi_worker --out .../abi-wire-liveness` exit0、原6个反例**6/6**，169个不匹配过滤项不充执行/条件排除。涵盖回答后hang、晚stderr溢出、溢出后hang、infra分态、双envelope、原produced-budget resource；准备/冷编译主要在发现阶段（2m35s），实际nextest3.772s，整命令190.170s，结果SHA **`94a7a5554d9450d0e6b626c98f52b56e6a5b245a72f26d2fdf2bfdd6510245cf`**。编译仍为已确认baseline core131 warnings，无新增warning；未对既有警告泛化修复。
4. `cargo t infra-ace-m1-arch --out .../case-binding-controls` exit0、**31/31**，无条件排除，整命令1.953s；新增8个schema/linkage控制含删/重复/额外row、来源/oracle/contract/key漂移、bool冒计数、copy改字节/重复/未清理、越界旧路径、终态/worker/recipe不符。生成的ERROR输入仅验reader边界，不冒真实M1正控。结果SHA **`a234d2918876b01ff2dce72ca14814007f9b27d49546b7100e588b848a2bfa04`**；`git diff --check` exit0。

输出对账：ABI既有conformance helper仍硬编码std::env::temp_dir，首次运行遗留本次6个PID的12目录。已核对创建窗口、精确tags（abiboth40360、abibudget23800、abihang45672、abiinfra44444、abilate41436、abiovh9632）、无活动owner、无reparse后，用单PowerShell原生Move-Item迁入本scope的`abi-wire-liveness/legacy-runtime/`；**78文件/312,998 bytes**，逐文件迁移前后SHA/大小/数量相同，原路径消失。清单`abi-runtime-migration.json` SHA **`ccb0d17b191b6870493f13bab5e14c1e2279dde9aa24d8d6aca26a97ea98f900`**。其他日期/PID目录归属未知，未动。后续这类原helper验证应显式给child TEMP/TMP为本scope子目录，避免再散落。正常编译target仍仅专属树默认路径。scope保持in-flight，最终merge再全删，非本轮清理完成。

拟Spec沉淀补充：显式wire证据schema及缺终态含义、Case复制件身份层与执行判定分离、ABI recipe必须来自当前合同。尚不改canonical；最终SD需随完整reader/registry/report再核。

下一入口仍T-01/T-02：用原始请求重算nonce/request/source绑定、四端独立oracle与真实Rust/C完整执行证书归约、SYN/937原语义及计量适配、完整有界首链与same-wrong/删C/旧假签章反控，然后才判T-02。当前证书命令曾钉225ba时worker哈希，与后来固定组重建的worker不同，后续须在最终确定producer身份重做显式认证，不能静默用旧producer凑当前证书。完整registry、/3 reducer及其余12AC仍待做，旧38cert-gap照旧阻M1；当前CLI仍exit2。D936/D938/D939交付依赖保持，非本观察边自动解除。

stage: work | plan_id: PLAN-941 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-941/auto-lang | branch: plan-941-dev | code_commit: 1b6e524b5a3218199fb788b89b1d9fca715b17d5 | task_ids: T-01/T-02（未关闭） | evidence: 5+21+6+31定向验证与真实21check/26exchange身份关联 | blockers: 无本单元待用户决定；最终依赖及域语义闭包未完成 | next_responsible_role: work

stage: goal | plan_id: PLAN-941 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: progress | worktree: D:/autostack/.wt/lang-941/auto-lang | commit: 1b6e524b5a3218199fb788b89b1d9fca715b17d5 | next: work

### 2026-11-06 T-02 双端oracle与显式认证reader；F-941-02转952

上一goal轮为progress：1b6e524b5完成原始wire/Case身份单元。本轮941检查点`55c8b37ea`→`5b2f795fc`→`a834b9cfa`（树clean），继续0/15，未给T-02先行pass。

- 新`ace_m1_scalar_evidence.py`只适配真实首例AV core-run/native scalar：原始request独立重算有序JSON hash、base64/source SHA、FNV、nonce、recipe/protocol/当前工具身份；每端单独对当前正式inventory oracle42，不看Case PASS。真实Q4二端各42绿；同时改两端观察为43，**二者均FAIL**，错nonce/request hash/foreign recipe各拒，reader阶段Popen禁止断言通过。`q4-scalar-admission.json`保原observed/expected/响应hash；全scope reservation1,777,669 bytes。35个治理控制实际通过（原31+新4，不重复继承原测试），合成单元只验边界，真实Q4结果单列。
- 原a2x-worker的explicit certify/certify-c新增opt-in `execution_observations`，保存**同一原compile/run**的原始stdout/stderr hex及exit_code，actual/reference各独立2进程；不添runner/meter、不重复执行，原默认比较/输出不变。显式identity请求的`certification_target`可在认证准备阶段复用原工具identity helper，先独立冻结rustc-vV/C原cl+vcvars+flags；默认identity/daily仍spawn-zero。新原conformance `cert_execution_941` 3/3：保无效UTF8/NUL/空白原字节、真实Rust/C两端actual/reference输出均value:42\n、工具身份与证书一致；最终定向命令9.400s。C既有证书`version_sha256`键实际承载原locator BLAKE3 image digest，reader保持其原语义，不擅改算法/键名。
- 新`ace_m1_cert_evidence.py`严格核对原process_capture/native_meter命令+Job终态、冻结request/producer SHA/commit、当前source/reference、完整golden/actual文件、cert/manifest、compiler工具/flags/caps/config与原始compile/run观察。每个actual及reference都单独对独立oracle字节，child PASS/marks布尔不供应正向授权，child明确red不提升。单命令360s/每流原1MiB及4MiB JSON/共48MiB门保持。C scalar manifest /1原本不含profile/domain，依其正式profile与实际bundle域证书绑定；不编造字段，不用Rust schema硬套C。39个治理控制通过（原35+4命令/原始终态反控）；raw ref/计量exit bool/超时/不一致CPU、错producer/request/extra字段均拒。

**F-941-02（真实producer身份缺陷）**：冻结55c8源码的真实Rust Q4认证产物manifest.hir_domain=`scalar-i32-draft`，同一cert.profile=`auto.source.scalar-i32.draft`而cert.hir_domain=`core-i32-draft`。原`domain_of`对profile字符串剥壳后得到scalar-i32，仅匹配scalar而落default core；actual EmitOutcome已经有正确域，C直接消费它。新reader拒该Rust包，合法C正控PASS；两个完整实际+reference原输出均42也不能掩盖身份错误，累计reservation1,072,088 bytes。原旧坏包保留在`q4-cert-55c8-frozen/`，`scalar-cert-before-952-final.json`保拒绝原因与C证据hash。

注意保留首次红/失败：首次build误用package `a2x-worker`被Cargo前置拒，查正式package后改`ace-a2x-worker`；首次显式identity仍命中旧release二进制，缺certification_toolchain在capture前置终止（`capture-q4-cert-55c8.log`、`q4-cert-55c8/identity-rust.*`保留），不冒通过。原因=ace-a2x组tools为空、conformance新测试用CARGO_BIN_EXE debug产物，不自动重建release；显式正确release构建后只在新目录`q4-cert-55c8-frozen/`执行完整认证，未覆盖旧失败证据。后续仍须在D952后新producer身份重放，旧55c8记录仅历史反例。

用户已明确授权“授权独立修复并继续（推荐）”，最小独立**PLAN-952**承接F-941-02：从实际out.hir_domain绑定，6 profile实际认证回归。952独树base1f1e42cb9，修前6例5绿/1scalar红，修后6/6；实现6308a8ca5经冻结独立复读复审pass，正在merge所属完整ace-a2x职责门。无需再次授权，也不将952完成等同941验收完成。941还缺SYN/937域语义与计量归约、四端首链/full registry与/3 AND等，T-03尚未开放。最终D936/D938/D939与旧38cert-gap仍按原合同。


2026-11-06 952合入门再次阻塞：两次完整ace-a2x各188/189，失败分别为未改动record/text事件stderr空、collections3072 stdout空，两个单例各重放绿。已停止刷完整组；952通过原std库字节相同副本的确定性private probe实锤共享CaptureWorker“空peek→最后写入/owned关闭→用旧peek判EOF”竞态（F-952-01），生产代码未改。诊断与最小独立修复提案已完整入952 §9；这不是证书元数据修复，需新的明确授权，问题已发且待答。952撤回可合入pass，执行3/6；941保0/15，当前a834b9cfa clean。原问题F-941-02由952持有，未关闭；不得把新reader的C正控或旧Rust拒绝冒完整四端成功。

本轮实际进展收据：原始双端oracle+cert过程观察+strict命令/证书reader提交55c8b37ea/5b2f795fc/a834b9cfa；35/39治理控制与3认证观察控制、真实双端same-wrong反控、真实C严格认证PASS/坏Rust域拒；发现并获授权独立952，6域修前红/修后绿；两次完整门红及原采集器确定性竞态改变下一步，非no-progress。当前新待决定条件首轮出现，goal保active，不提前blocked。

stage: goal | plan_id: PLAN-941 | plan_revision: r1 | round: 5 | from: work | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-941/auto-lang | commit: a834b9cfa | next: user（仅F-952-01新增共享采集器独立修复授权；941/952原授权保持）

2026-11-06 waiting audit（F-952-01同一条件连续第2轮）：上一轮为progress，已提交a834b9cfa、952真实6域红绿与共享采集器确定性反例，非纯状态复述。本轮仅复读941/952 §9/§10、当前工作树身份与新增计划存在性；未收到共享采集器独立修复授权，未见原owner交付修复。941/952两树均clean，952 HEAD仍6308a8ca550586f5e9d221483611e991a6db57ba；上一完整组进程已明确exit1，不存在可当verified wait的活跃门禁。按auto-plan-goal待答规则不实施新修复、不运行测试/构建、不重复提问、不缩小原941目标。当前goal保active，未满足blocked三轮阈值；本轮属于no progress（待答簿记）。

stage: goal | plan_id: PLAN-941 | plan_revision: r1 | round: 6 | from: user | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-941/auto-lang | commit: a834b9cfa | next: user（F-952-01共享采集器独立修复授权）

2026-11-06 blocked audit（F-952-01同一条件连续第3轮，含发现/待决定原轮5）：上一轮为no progress，仅待答簿记。当前再次核对952 §10与941原问题记录，仍无新增共享采集器独立修复授权或原owner修复交付；941/952树clean，952仍6308a8ca550586f5e9d221483611e991a6db57ba；未出现已分配953/954承接交付，亦无本会话已确认活跃门禁可verified wait。原完整门两次终态红，不能靠未修复原始字节采集器继续当已可信证据或合入952；也不能把952原证书元数据授权解释为共享设施修复授权。按既有待答规则，本轮不实施/测试，停止自动续行并把goal标blocked；941保持executing 0/15，952保持executing 3/6，均不归档/清理。解阻动作=用户明确授权已发F-952-01最小独立修复（原采集器owned-close后重新排空、原错误/停止/进程树分态及永久反例/职责与消费者验证），或原owner交付等价有效修复；原941/952全链授权保留。

stage: goal | plan_id: PLAN-941 | plan_revision: r1 | round: 7 | from: user | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-941/auto-lang | commit: a834b9cfa | next: user（F-952-01独立共享采集器修复授权/原owner有效交付）

2026-11-06 F-952-01新增解阻授权：用户答复“授权独立修复并继续（推荐）”；独立PLAN-953承接原采集器竞态、错误分态/永久反例/职责与消费者回归、复审合入后返回952/941。当前仅已授权/立项，不冒修复已交付，不再重复询问该范围。host goal仍blocked，本次按明确直接用户指令执行953全链，不新建goal或伪称自动active。

2026-11-06 **D953已交付**：PLAN-953 r2 source修复65db88be2150991b9c3bbe79d527891fd6a7a27f已线性合main，完整auto-tool-execution43/43、无LEAK/残留，必要ACE两case在新950基面也各1/1，canonical SHA535cdafa3c393a06ceae0635dce506bb04577c304d7fc09565b699dde7b00fb4。953实际归档6/6、两树/branch/组/scope删除完成，无hold。F-952-01独立共享采集器修复授权/交付阻塞解除，不再问授权；952应重基并复审/完整ace-a2x门后合入，941原目标继续保持。周期cargo t all已由950交regress等待936 CT活动门，本会话不重复启动。

2026-11-06 恢复检查点：D953已交付；952在新基面完整ace-a2x **189/189**（results SHA25c198b56e3fc3187b507c0add67e573518b037e9f341f1a902f82947b4336c6），下一步为952 merge，未冒称952归档。941新syntax严格reader提交原5c5547e71，10个真实SYN观察用临时显式当前929约定逐项准入、45治理单元控制通过；**正式identity CLI新metadata导出尚未编译验证**，原10例不充正式新导出证明，T-02仍未完成。原native/AV scalar、cert、SYN等局部证据均不代表完整registry/reducer或/3 AND交付，仍executing 0/15。

941重基含D953的main6b6b191c3，原5c5547e71最终映射到**2033c1e6335e85842fdc568c7c019b93dea55eea**。唯一冲突为950移动bridge_flags注册；保原模块一次、cert_execution_941一次排序注册。rebase已终态成功，正式metadata自有WIP从scope补丁恢复，只有inventory.rs未提交，不误删。原始补丁在plan-941/pending-identity-metadata.patch供恢复；无第二checkout。后续须核重基diff并执行正式导出/相应定向门，未把旧证据自动全绑定到新源码。

host实际goal objective=resume/status=blocked；原F-952-01授权与修复均已解除，但host未自动转active。按auto-plan-goal inactive规则请求`/goal resume`，本次保存已授权工作与已结束门结果，不创建第二goal、不mark complete。恢复后先952合入收尾，再941 T-02真实四端/SYN/937链。

2026-11-06 **D952已交付**：r1/08df260df8fad98ad2b56a1ba79e68a64e161e55线性合main，canonical SHAa354c7807e538d9f34482f2710083752d7837a4ffb55c6bd459c6c1e0342192b，6域真实认证/全组189/189，归档6/6，两树/branch/组/材料目录已删除、无hold。F-941-02关闭；941须在自己新producer上重建/重放严格认证，旧55c8结果不得冒当前身份。用户`/goal resume`已明确恢复续行，不重复请求同一操作；host状态未更新仅记录，不建第二goal。下一阶段auto-plan-work继续原T-02，941仍0/15。

### 2026-11-06 D952后原始首链与跟踪合同预览检查点

用户已明确`/goal resume`，原全链授权恢复，不重复请求。941唯一树/branch仍原身份、auto-down只读58ea33f；纳入D952后7个原patch重基，唯一注册冲突同时保cert_domain_952与cert_execution_941且各一次；最终旧2033c1e6映射51360595e。后续分段提交56be0ce5e（原typed completion/AC options/artifact compare导出）、6108e7135（ABI原请求sample nonce及typed步骤/oracle导出）、1b9014c2b（bridge reader）、18b23f6e0（首链preview）、**615b53051**（跟踪typed合同与正式工具路径冻结）。树clean；T01完整义务映射/T02整体依赖尚未闭合，仍**executing 0/15**，未把首链局部交付偷算全计划。

- 正式原auto-case导出build exit0（27.81s；继承core131/lang5旧warning，无新增），929全11条/其中10语法；/1原字段与/2剥去新增身份/typed期望字段逐项相等，旧schema/正式driver不改。真实`cargo t case-ace-unified-source-929 infra-ace-m1-arch --out .../syntax-formal-metadata` exit0、11+45执行，整99.549s含新基面工具准备；results SHA379053295bf5380e5cf2874ff127f7d597cad8dc95c0d011379a9277b182c77a。strict reader用正式当前合同对10条原wire逐项准入（源码/nonce/hash/grammar/预算/恢复原字节/诊断），非临时期望；`syntax-formal-admission.json` SHA230ad62a47db43c92cb11097f00dcded19844a8ee5e0a5a36c97bf84f7cb83e6，reservation3,139,554。首次辅助脚本误拼ac-case-worker.exe而拒，纠正到原locator正式ace-case-worker.exe后才通过；不是生产失败/不冒首次绿。
- 原ABI append_request_row仅在AUTO_CASE_WIRE_EVIDENCE=1时加同一已存在nonce，五个终态调用点全覆盖；默认样本shape不改，不新meter/时钟/runner。原typed exporter直出abi leg/query/edit/restart及resolved probe cache/compute/bindings/stages期望，非regex读deck语义。build exit0（20.56s）后`cargo t case-ace-ca-d-scalar-937 --out .../abi-query-original` exit0、6/6、49.146s；results SHAfbfb0d95063966aea434130a32d513b8b0217e577756d26fd72736bc1bfddced。新bridge reader核11次原查询、当前deck字节与恰一次literal edit、正式recipe/方向/nonce/有序request hash、独立缓存/生产数/诊断oracles、每次原Job的P/M/IO与父sample nonce/seq/raw计数关联，6check全PASS；`abi-query-admission.json` SHAe5181563b97ef54536fcab10a3c99718d87c3716329bd180b2d11c3d29736d9f，reservation2,766,798。真实diags负例原schema合法省略stages，仅原字段absence，不伪造阶段计量或性能pass。`cargo t ace-host --test-name abi_worker_ --out .../abi-nonce-liveness` exit0、原6协议/预算/late overflow/response后挂起反例6/6、42.874s；完整ace-host留最终merge职责并集。
- 新D952基面显式build正式ace-a2x-worker release后原capture顺序认证Rust/C，actual/reference各自value42，strict完整生成文件/manifest/profile/domain/tool/current source/reference/Job/raw终态均接受。随后原ace-host工具准备实际重建a2x-worker、二进制SHA变化，首组合**确实拒STALE**，不把相同源码预测当相同producer；保q4-cert-after-952历史正控，工具准备结束后新q4-cert-combined目录显式重新认证。原源码/样本/预算/golden/独立reference均不改，当前producer冻结18b23f6e0；之后仅reader/registry更改，正式二进制hash不变。`cargo t case-ace-m1-q4-941 --out .../q4-after-952-original` exit0、4/4、4.090s；daily与显式cert保持独立。后续原始证据均经当前实际binary/content hash重验，不重跑未影响的编译腿。
- `registry.json`明确scope=first-slice-preview、full_registry_closed=false，冻结20个义务（Q4四目标+SYN10+937六），并把**原formal identity exporter**的21check typed合同/源与oracle身份正规化路径后入跟踪（53,443 bytes），实际source/case/suite/oracle重新hash。读入的父metadata改oracle/recipe/source路径/缩check数即使自身重hash也不能覆盖跟踪合同；正式worker路径固定到原production入口而非任意repo文件。未把20作扩大后完整分母；完整旧313/38守恒仍独立reader核。
- 只读组合入口`ace_m1_arch_acceptance.py --first-slice-preview --repo <941 tree> --evidence <plan-941 scope> --bindings first-slice-current-bindings.json`实际逐层读取原wire/终态/文件/当前期望/认证与旧核算，不启动工具。当前`tracked-first-slice-preview.json` SHA **afa8464d8663339d3a7978d7104310fef4c3293c55d17098cbebb4d28518621d**、registry SHA **69bbffe22276ce1115f4f5464f41c26af24b36ddd453544822dcfaa05f830836**；schema /3 mode preview、20义务全PASS，first_slice_evidence_complete=true；legacy-current=false/legacy-required-execution=false/full-registry=false，所以CLI **exit1**、evidence_complete=false/acceptance_system_delivered=false/can_claim_expanded_m1=false。没有输出正式签章。原process_capture/native_meter实测wall281.683ms、whole process peak15,020,032 bytes（比private delta更强保守上界），同一共享LiveBudget累计16,755,503 bytes<48MiB；JSON约15KiB<4MiB，未用第二meter/新runner。
- 隔离反事实拷贝只作控制、绝不进入实际执行集合：四端原始观察及Rust/C独立reference都改为43并重算副本hash/原始终态链接，实际四个reader各FAIL，first-slice AND false；原child PASS不能覆盖当前oracle42。删除C目标pin严格拒，任一20义务删除均破AND；旧claimfalse保留并阻签。伪当前导出的expected_exit=43即使重hash仍被跟踪typed合同拒。`first-slice-real-controls.json`/`first-slice-pinned-controls.json`保摘要。首次控制脚本错选Q4-001路径未实际改AV/native（两个仍PASS），断言立即失败；修选择器为真实ace.m1.expanded.q4-001后新control2副本四端均FAIL，原失败副本/脚本保留，不冒mutation曾发生。另首子scope传目录给regular-file guard被正确拒，改为验证manifest后取已核父目录；无绕路径门。
- 永久治理单元60（原45+bridge8+AND4+tracked oracle3），实际`cargo t infra-ace-m1-arch --out .../tracked-preview-frozen` exit0 **60/60**，结果SHA **f67078c51a3c40a441c69bcb166b56b99c200b9c93ac16a82dc0ddb44e4dbc00**。受影响格式/git diff --check通过；不用合成单位冒真实Q4/SYN/ABI执行。

当前未闭：T01完整scalar61/resource/method/collections/所有ARCH到原case/check/负载预算映射；T02此次只证明bounded首链，完整身份/闭包还须随T01审计冻结；T03完整registry、T04..T13机制/报告与最终review/merge均未交。D936现executing11/14、D938 executing6/13、D939 drafting0/14，最终真实required输入尚未交齐，不能替owner实现或靠preview空过。下一步继续T01实际原子来源/预算与全部责任映射，不先开放仅凭首例推导全覆盖。

### 2026-11-06 T01原始义务发现与完整交付前置检查点

最新941源码提交 **a5105c3e8**（唯一树/branch clean）；新增c5fe47756原typed scalar表显式只读导出、2140dd941 scalar义务表、08d7776fd Case目录与只读审计、61448f2ce来源记录、000ac7b9b实际source/oracle重hash、a5105c3e8永久失鲜反控。未修改主检出代码，主检出他方docs WIP保留。

- 原`scalar_semantics::cases()`直接导出61个typed Case：40 core/21 scalar、53 value/8 trap、4 trace配置、8个release_too；source_sha/独立oracle/marks/配置均来自原表，**discovery executed=0**。新named ignored test只写显式plan-941输出、父目录reparse拒绝/create_new、4MiB帽，不调用frontend/compile/guest，不把table记录算语义执行。`cargo t ... --ignored`被固定入口的仅aavm规则前置拒；native nextest首次libtest-json缺显式experimental env也前置拒；随后原`cargo nextest run --manifest-path experimental/ace/Cargo.toml --target-dir experimental/ace/target -p ace-a2x-worker --test conformance --run-ignored only -E test(=inventory_941::read_scalar_contract_inventory_941)`配原NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1 exit0、仅1 metadata test实际ok，未运行标量语义组。原libtest兼容事件对ignored过滤数出现u64max，因此不采用该过滤数计量覆盖；61来自实际输出和原typed表，不拿过滤数/ignore事件充PASS。
- `scalar-original-contract-inventory.json` SHA20a0054e641d4010e05d1f8ccddb7e963fe25aa69dfe25a5998274921af85a53；原表源SHA9007cdd1ecfbac5b9fcd73e7629983bb2ff35fac1fff8a63a740fec6079861b3。跟踪`scalar-obligations.tsv`完整61条，原Rust正式入口scalar_semantics::full_corpus_true_chain=61 debug+8 release=69次声明配置，C正式入口a2c_scalar_semantics::c_full_corpus_true_chain=61次声明配置，组ace-a2x。原caller deadline/资源与nextest-full slow-timeout按代码来源记录，不把AC默认预算拿来冒probe成本，不把声明130配置计为本轮执行。TSV SHA28e65dfd5b80d2d4334f5524ab973870eabfdcc6ca9c2cbd6fb60a9223637b1b。
- 在当前同一formal auto-case typed exporter重新发现35 suites/511 checks，2.739s、executed=0，scope/t01-current-identities-final/summary.json SHAb31fcc69c8dccdf4c4ca9444a1a500744a2e801f28f0135ed2c4f50cbeddd85d。首次辅助循环误将旧summary.json列表当inventory，TypeError中止；新final目录显式排除该索引后35输出全成功，保首次目录，不覆盖失败记录。跟踪`case-obligations.tsv`按完整suite/case/check、原group/suite文件、源码与oracle身份、合同/有效check指纹、state/skip、原声明AC预算、source/oracle文件内容摘要映射；不是保存旧PASS日志。原正式metadata全部显式input_paths整理为542个去重source/oracle文件，父准入流式重hash连同case/suite共668个当前文件，不能只hash.case.ad后放过源/金样漂移；额外resolved probe sidecar/helper依赖仍明确pending。TSV最终539,431 bytes，SHA717d4c8a5635369a3b37a6e99ed4071367be4e4b3a9e861dcc9999ba6ef301b4。
- 35中的`unregistered-a2r-hir-892`是原已具名的5check手动suite，不存在独立固定组。首次catalog审计拒绝了把它混当固定组的做法；当前精确列出原suite路径/owner=941显式原auto-case调用，其他任意缺失组仍拒，不自动映射别名或新加全局组。Case/Scalar两目录已由真实只读组合器消费，冻结TSV/原表hash、原组输入hash/完整列与分母、唯一性/状态/原始配置均核；报告执行保持UNOBSERVED、mapping_complete=false，域预算/额外sidecar/helpers/剩余ARCH/旧required闭包不补0、不充green。
- 永久治理64：原60+目录4组控制（511/61/69/61守恒、duplicate/missing、伪执行/未知组、current source/oracle漂移、削release/trap配置、重写跟踪目录）。`cargo t infra-ace-m1-arch --out .../catalog-final-frozen-controls`实际exit0 **64/64**，results SHA **d28edc13e07b4d1b3bb0edd76b6c33a634c549b3ecf46945d00d6aac14a4fda2**。正常构建仍唯一树target、ACE target，无外置target/其他盘安装或消费根。
- 最终当前首例预览`catalog-first-slice-preview.json` SHA **656f3cb3033dc95c64000c86768624e838e1741254b1edba1881a142fee2998d**，16,876 bytes，20首例义务仍全PASS，CLI exit1（旧门/旧required执行/完整registry未闭，preview恒不签）。同一原capture/meter实测wall548.376ms、whole process peak18,956,288 bytes<128MiB，共享保守reservation25,177,278 bytes<48MiB；记录parent bindings hash、9个实际reader源文件hash与catalog/原typed表hash，来源闭合到当前入参。原始new ABI/SYN/Q4/cert source/工具/契约未改，复用先前真实执行并在此逐字节重验，未为catalog/schema变动重复昂贵认证/性能门。

**当前前置阻塞（不假称本包work完成）**：936仍executing11/14，938仍executing6/13，939仍drafting0/14；required source CT/frozen pages、foreign标量调用/事实代际、资源/opaque/callback/error/ctx及其预算/恢复观测没有已交付最终合同与全部原始接口。依据本Plan §5.1（required行须具体producer或明确未交付owner）、§5.4（原owner交付功能/观察，不由组合器模拟）、§8 T01→T02→T03依赖，不将在途branch或939待证明source机制冻结成已发布能力，也不开放依赖该完整映射的T03及后续验收。当前原scalar/Case清单和首例链已形成可恢复检查点，但T01域预算/ARCH/旧313到执行闭包等自有审计仍未闭，不能因依赖缺失写execution_done或将本轮0/15改成完成。解阻=原936/938/939 owner交付稳定D及承诺字段/原始计量/required页；恢复先核实际source身份，补全T01，再按依赖扩展registry/reader/正式联合报告，不重复问原941/952/953授权。

stage: work | plan_id: PLAN-941 | plan_revision: r1 | outcome: blocked | worktree_path: D:/autostack/.wt/lang-941/auto-lang | branch: plan-941-dev | code_commit: a5105c3e8 | task_ids: T01/T02（检查点，未闭） | blockers: required D936/D938/D939稳定producer/原始接口未齐；完整映射/后续验收pending | next: 原owner交付后继续work，非final review

### 2026-11-06 goal恢复active后的独立旧格位执行与父终态修复

上一goal turn分类=**progress**（D952收尾、20项真实首链/64控制、511/61源目录）。本轮get_goal实际active；恢复检查承认此前“待D”不是所有独立工作的停点：D936/938/939仍未齐，但原旧格位接线可安全推进，继续work而不重复请求授权。最新main仅他方docs WIP、无代码WIP；941唯一树clean，auto-down只读58ea33f，原未归档工作保留。

- 逐旧required源格位审计发现B08/B11：binding原oracle均7、ac/av required，但35个原suite缺对应四条正式Case入口（原legacy-AVM差异与a2r emit-unverified依旧分开）。在941自有fixture增加独立legacy-coverage.suite/case与**逐字节复制**原B08/B11，直接用原formal native/AV父入口及原oracle7（B08外层x保持7；B11 a=10/b=3故7）；原批准binding/source-manifest/NA/denominator/旧assets零改。新增显式固定组case-ace-m1-legacy-941，all=false、不改daily/batch范围。提交abf8fea4e。
- `cargo t case-ace-m1-legacy-941 --out .../legacy-b08-b11-original` exit0、4/4、5.147s，results SHA **8e36158c251d0b03099db2f2d4e6ddc68d5dc912bbfc6636596af99c011c92d0**；原wire opt-in开启，所有runtime/TEMP/TMP在plan-941 scope。原35/511目录不动态扩分母；新增四条在独立legacy执行视图单列。
- 新`ace_m1_legacy_execution.py`（caf72c902）从原已hash批准表提取原value7，重hash原source并核新copy同源、受跟踪typed合同的当前Case/check/oracle/recipe/target/报告身份、当前正式worker与原始终态，再调用原scalar reader分别判断。不是按Case PASS回填。所需313集合仍从原冻结ledger导出，当前实际**4/313 PASS、309 UNOBSERVED**，完整执行verdict=UNOBSERVED；旧38 cert-gap不替换、不免除。registry仅追加该typed合同，preview仍20原首例义务，四旧格位另有execution_summary。缺旧required项不归绿。
- 隔离控制只修改副本四条raw观察7→8并重算原始链接/清单hash，Case自报PASS未动，strict四个旧格位均FAIL、其余309未观测；`legacy-execution-wrong-control.json`保留明确counterfactual标记，不进真实执行集合。永久控制验证部分4项不冒313、任一313删除不闭合、外来key/重复分母/布尔verdict拒、原copy源hash与批准件完全相同；合成全313正控只验证纯AND，不冒实际覆盖。
- **本包自有新准入缺陷修复**：pre-fix反事实Q4副本把父Case AV状态改ERROR/summary改3PASS+1ERROR，raw value42保真，原组合器竟仍给q4/AV PASS。`parent-error-before-fix.json`确立expected FAIL/observed PASS；无需另立已归档能力修复，此代码为本包未交付实现，在原AC-02/10/11范围修复。32c2d7aaa保每个原Case父状态，PASS不能提供正向授权，FAIL/ERROR/SKIP必须阻断提升；`parent-error-after-fix.json`实际ERROR→FAIL，first_slice false。2a460d6d4进一步保全部declared Case父终态（包括daily-only emit check），避免fresh explicit cert盖住daily ERROR；独立daily父ERROR副本原20 raw域观察全部PASS，组合首链仍明确false、source_case_reports列原ERROR。此为真实区分性副本反例，不声称发生过实际工具ERROR，也不更改生产runner/语义预算。
- 冻结提交 **2a460d6d467e654bd18a784de59492d3d3881c79**，`cargo t infra-ace-m1-arch --out .../legacy-and-parent-frozen-controls` exit0、**71/71**、1.819s，results SHA **7e2ecf2d74e53964bdc7dfe80b001fa11df50dd9c1e42966c31ab372718d5b58**。git diff --check通过；没有final review/merge pass，也没有启动全仓或跨owner修复。
- 新完整首例CLI用first-slice-with-legacy-bindings.json（默认无旧执行输入时依旧UNOBSERVED）；`legacy-four-final-preview.json` SHA **ea4aeb71b6f8070cf67c778d23cc1b1fd75a1094a6d5b96959873237e989015a**，24,884 bytes；真实20首例PASS/旧执行4/313，CLI exit1、全局claim=false。原capture/native_meter实测398.860ms、whole process peak18,563,072 bytes<128MiB，保守共享reservation28,810,324 bytes<48MiB。10个reader来源hash与父bindings/case reports/raw命令/文件身份可追溯，组合器Popen禁用控制通过。

936最新复审needs_fix（F-936-R1-01 source CT到a2x缺接线，main fb68b6a3e收据），不能当D936稳定交付；938/939仍待原owner。下一工作仍为T01/T02：从原规范与真实入口继续补旧required/ARCH/预算映射，未支持reader明确UNOBSERVED，不能将四条新执行或71控制替代313、扩大后全registry或最终验收。当前仍executing0/15，留active继续，不把有独立进展的轮次算成waiting。

stage: goal | plan_id: PLAN-941 | plan_revision: r1 | round: 8 | from: work | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-941/auto-lang | commit: 2a460d6d467e654bd18a784de59492d3d3881c79 | next: work（剩余实现/映射继续；不是新增final review失败轮）

### 2026-11-06 原父合同发现、AV/native typed严格准入检查点（goal第9轮）

本轮继续T-01/T-02，仍executing **0/15**，无work完成/复审pass/扩大后M1签章。实际实现提交`269590f00`、`291d88cbb`、`983fb8f8d`、`cf520423f`、`b974ee884`、`aa23ed8be8528dff01c3ed81a23d617909e00ca2`；唯一941树clean。原授权有效，继续范围内工作，不因最终D未齐中断独立映射/reader施工。

- 在原auto-case增加只读`parent-inventory`：直接读取原AV/native/ABI父限额与原常量表，不启动worker/solver/语言执行。11表43行（含8个typed值）为DISCOVERY、executed=0；AV/ABI原session60s、native120s与各request/response/stderr/build/grace帽分开保留，未把发现视作预算运行通过。当前7份原producer源码SHA与实际auto-case二进制SHA、显式命令/原native_meter/stdout/stderr绑定，表/来源/命令/计量伪造均拒。
- 原identity-inventory /2增加已解析probe_bindings、declared_probe_steps、resolved_probe_oracles、typed_oracle及真实File/Text/Block来源；不解析Debug字符串。旧/1在34个原固定套件506check上逐JSON对照不变；原35套件511目录另含manual892五项，目录仍DISCOVERY。ac-av原18check含4个inline Text源，严格绑定原Case文档及当前resolved合同，不伪造源文件。
- 真实`cargo t case-ac-av infra-ace-m1-arch --out <scope>/parent-contracts-and-ac-av`：18/18原check及76/76当时治理控制，37.1690141s；results SHA256=`281854e863717b80fd7ede7f26ddcb5447f9820db497e37c3a24717d482563f4`。随后冻结b974ee的`cargo t infra-ace-m1-arch --out <scope>/av-typed-final-controls`实际83/83，results SHA256=`c812e73708a1422ec7ed11da5579021033c2441fdbde541532fd039168ce353b`，不是新增83项真实语言用例。
- AV data typed原始request/terminal/response严格绑定原entry/fixture、carrier SHA、caps、store owner、recipe/nonce/request hash与当前worker；原typed分支source_hash实际是entry FNV，不能套carrier FNV。8原typed值逐(kind,bits)比较（含NaN符号/payload/负零），5项映射原W01..W05:av。它们是typed fixture，`.at`只作请求carrier，**不冒宽类型源码**。位值相同把i64标成f64、同时重hash的副本，仅W01:av FAIL，原4+其余4仍PASS，304未观测；`typed-av-wrong-kind-control.json`为隔离反控而非执行。
- native原891入口的opt-in原始exchange采集接入既有call_worker_observed；原二进制路由把多个recipe归一为scalar/collections，新增采集记录改取原request.recipe，业务路由不变。这是941新观察接线，不改891功能/私有ABI。严格reader核验44字节原机器帧magic/version/签名ID/状态/FNV校验/类型/完整位值与JSON摘要一致，再单独对原binding及当前Case独立oracle；12个原Case父拒绝全保留，严格执行认领仅5项W01..W05:ac。原父text nonce在机器帧仍为0，报告明列此事实，不声称帧freshness或public ABI。
- **冻结最终有效门**：`cargo t case-ace-m1-typed-native-891 infra-ace-m1-arch --out <scope>/native-typed-frozen-gate`，原native **12/12**、治理 **90/90**，exit0/gate_passed=true，36.8467436s；results SHA256=`1f9b3f9e07a1c0c191de87e8ac59d993b6643cd67e97673b0b97497224445467`。新7反控检查一致造错的机器帧+摘要仍FAIL、只改摘要拒、坏checksum/签名/nonce/状态/ABI/目录/fixture/args/计数/transport拒；没有缩样或替换原oracle。最终双bin构建37.37s，原core131/lang5基线warnings未增加，本单元无新增warning；diff检查通过，不对原大文件整体格式化。
- 新鲜身份重新发现到`native-typed-frozen-metadata/`，与6份跟踪typed合同逐项相等，当前配置`native-typed-frozen-bindings.json`。只读CLI `python scripts/test-governance/ace_m1_arch_acceptance.py --first-slice-preview --repo <941-tree> --evidence <scope> --bindings native-typed-frozen-bindings.json`真实exit1（preview恒ineligible），20首例独立义务完整；原执行仅**14/313 PASS、299 UNOBSERVED**，所有旧38 cert-gap仍阻签，claim=false。报告`native-typed-fourteen-final-preview.json`37508 bytes，SHA256=`89b02c48c7580847c60810e777fda6ad6367b918b7e69315fb0910830206b198`；原capture/native_meter 559.1872ms、whole process peak20,430,848 bytes<128MiB，共享保守reservation44,496,668 bytes<48MiB。13个reader来源hash入报告，Popen禁止控制证明组合阶段不执行工具。未来更大集合须在同预算下明确对象释放/流式归约，不能重置仍存活对象的预算或放宽帽。
- 真实证据副本反控`typed-wrong-kind-native-and-av-control.json`：native帧及摘要同改f64且位值仍8000000000000000并重算帧checksum/所有文件hash，只有W01:ac FAIL（13PASS/299未观测）；再把AV同格也改同一错误类型，两格都FAIL（12PASS/299未观测），不因双端相等变绿。控制副本不列真实执行。

保留首次失败/无效结果：原`native-typed-original`12项父绿但无wire；第一版接线在`native-typed-observed`未显式打开采集开关，仍被严格reader拒无exchange；`native-typed-wire`打开开关后原记录recipe错为scalar，被拒；`native-typed-exact-recipe`运行期间施工变动，固定门报`code/config/inputs changed during execution`，整轮无效不计PASS；本地90控制初跑因ac_mn源码pin未刷新89/90，重新冻结源码后90/90。均未覆盖、未当环境绿；最后有效冻结门如上。原auto-case仅构建一个bin与固定组双bin feature-union会有不同二进制SHA；先前parent收据因此STALE，重新用实际双bin当前producer发现，没有将同源码当同工具身份。新鲜源/命令/原始输出都在唯一scope。

当前936已更新到reviewed r2 13/14（仍非D），938 executing6/13、939 drafting0/14；此状态只影响真实消费其稳定接口及最终T12，不阻独立T01/T03/T04。下一步补**完整原313与Q4/ARCH义务到producer/case/check/owner/原预算/样本的显式映射**并推进完整registry；不得把14条已实现reader或90控制替代T01全闭包，也不再只复述外部D等待。SD草案保持，最终冻结职责并集/独立review/merge未执行。

stage: work | plan_id: PLAN-941 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-941/auto-lang | branch: plan-941-dev | code_commit: aa23ed8be8528dff01c3ed81a23d617909e00ca2 | task_ids: T-01/T-02（未关闭） | evidence: 原父11表43行发现、83→90治理控制、原18AV/12native check与14/313严格旧执行、same-wrong类型反控、20首链与预算实测 | blockers: 本单元无新增待用户决定；完整registry/机制reader与真实D仍欠 | next_responsible_role: work

stage: goal | plan_id: PLAN-941 | plan_revision: r1 | round: 9 | from: work | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-941/auto-lang | commit: aa23ed8be8528dff01c3ed81a23d617909e00ca2 | next: work（剩余映射/实现继续，不是final review失败）

### 2026-11-06 全旧313义务显式映射与对象生命周期预算（goal第10轮）

上一轮分类progress（aa23ed8be真实typed接线/冻结12+90/14旧格位）。本轮main零代码WIP，唯一941树复用；当前施工提交`0dbae763b`→`4ea428e12`→**`07874db1ab270c3a4a3bee51a91dde979b2bd624`**，冻结树clean。继续T-01/T-02，**0/15**；没有缩减原目标、提前final review或签章。

- 显式原parent只读发现到`full-map-formal-identities/`，随后canonical刷新到`full-map-canonical-identities/`：37套件533check（旧35/511 + 原ac-av18 + 941旧源4），执行=0。新增长期源合同`crates/auto-lang/test/cases/ace-m1-expanded/legacy-required-map.json`，完整**313** required逐格保原layer/token/namespace/target/binding oracle、source/typed/scenario身份、原source SHA、正式suite/case/check/recipe/当前Case与sidecar SHA、双owner及具名缺项；不是runtime收据。**251格**有显式原正式入口locator（共304个不同entry），**62格**尚未绑定正式入口，78个scenario仍欠完整机制/原样本序列，原38 cert-gap完整保留。所有discovery execution=UNOBSERVED/executed=0，14个已支持reader不冒313实际执行或全部机制覆盖。legacy-A2R/legacy-AVM不被新HIR/C身份替换；同源hash只核源，不自动判覆盖/执行。
- 新`ace_m1_legacy_obligations.py`在当前授权repo内严格核固定map hash、原四批准件+scenario规范、313集合hash/原oracle、正式组与suite/target身份、全部显式当前源/sidecars和alias边界。W01 i64-min与W03 f64负零虽有相同bits也不能交换locator；多文件project保main+helper，不误删合法二文件腿。原sample冻结：I01 30修订、I10 512不同修订；I11按既有批准规范P90≤2000ms、至少9/10，P95<1000ms仅历史诊断并列，未用binding的旧简写改验收口径。缺项/unknown/type错/样本削减/认证gap删项/父PASS冒执行均拒。
- 将完整旧义务discovery接入现只读preview，未把它称完整新registry。加入后原累积reservation触发48MiB拒，**没有放宽帽**：新增显式对象生命周期释放，三个已知旧执行reader与两个已知discovery reader返回后，原raw/decoded表/索引不再存活，只保有界摘要预留（执行摘要512KiB、discovery128KiB）；原CaseCopy、registry/identity等仍存活部分不释放。LiveBudget保留全生命周期`peak_reserved`，summary过大/无效释放/新分配越帽仍拒，不清峰值、不从传入counter推导。永久反控证明保留摘要预算、峰值不回退、后续超限拒、过大返回对象不释放原预算。
- **canonical交付缺陷本轮关闭**：全局*.json最初吞掉新map，4ea428e12以明确永久源合同例外纳入git（raw evidence仍只在scope）。随后实际`git show`对照发现Python默认写CRLF，而core.autocrlf=input保存LF：本地source pins与fresh blob不一致。归一15个本包文件的本地换行（大部分Git源码blob本来已LF，无业务改动），更新registry中ABI/native父源码hash与legacy Case/suite字节身份、完整map及其固定hash；最终map705009 bytes、SHA256=`ebe902c39c042e422b7eef604b119a4fab0f88122374914b9083a3e604d1d1fb`。原批准manifest/binding/NA/分母零改。最终`git cat-file --batch`逐件核验**191个**map/registry/parent/批准/原源/Case/sidecar：工作字节全部等于当前提交blob，`full-map-committed-input-audit.json`留明细。之前CRLF工作副本的收据不再充作新鲜检出证明。
- legacy B08/B11 Case/suite换行字节身份变化，按实际需要重取原4条：`cargo t case-ace-m1-legacy-941 --out <scope>/legacy-canonical-lf` exit0 **4/4**，15.4418981s，results SHA256=`ffaf16e3f1b72ba84d571c0cf68d10898ba6a924ed6f7e0e3c8075acbbea9567`。原AV/native worker SHA、其他原Case/suite/source/oracle/steps均未改变，20首链与typed原18/12有效原始证据复用；只读37份正式身份+原parent discovery重新从实际双bin当前auto-case取得并与6份跟踪合同对账，配置`full-map-canonical-bindings.json`，没有拿旧producer身份冒新二进制。
- **最终冻结有效治理门**：`cargo t infra-ace-m1-arch --out <scope>/full-map-canonical-frozen-controls` exit0/gate_passed=true，**100/100**（90旧控制+7 map+3生命周期），2.6169286s；results SHA256=`0196907038cc7332338bfba7548231f9ec2ede32a47ed922042743f1e2a40d54`。本轮只改父只读治理/源合同，未重复运行未改语言原完整组或全仓batch；新鲜LF Case4已单列实际重放。diff检查通过。
- 最终原capture/native_meter包实际CLI `python scripts/test-governance/ace_m1_arch_acceptance.py --first-slice-preview --repo <941-tree> --evidence <scope> --bindings full-map-canonical-bindings.json`：exit1、20首链完整、实际旧执行仍**14PASS/299UNOBSERVED**，map发现执行0，claim=false。`full-map-canonical-final-preview.json`39525 bytes，SHA256=`05546a8ed3c63cbb6a2cb84ccef7cf3bcf2b91761ec6a415c83fbf4ec9781f60`；1429.2239ms，whole process peak20,496,384 bytes<128MiB，当前保留reservation30,045,318、**峰值46,727,672 bytes<48MiB**（双值都入报告）。14个reader来源SHA可追溯；组合Popen禁止正控通过。该运行不是正式性能采样/P90 gate。
- 当前canonical配置重放既有真实证据副本反控（`full-map-canonical-controls.json`）：native帧及摘要同改错误f64/正确原bits，只有W01:ac FAIL（13PASS）；再将AV同改同错，两格均FAIL（12PASS）；299未观测与map UNOBSERVED原样保留。证明新locator与对象释放不覆盖反例。副本不是实际运行证据。

首次失败/中间状态保留：首遍map适配误限single source而拒合法916/917二文件project，补完整source列表逐件核验；生命周期初版只释放catalog仍越48MiB而拒，未调高帽，按已知reader真实返回生命周期释放后通过；新增预算单元首遍漏算EvidenceScope既有256KiB缓冲而1/10红，校正预存基数后10/10；首个100门时map被ignore/后续CRLF blob不等，均未作为最终交付门，最终跟踪LF提交与191件blob对照重新证明如上。全部原始材料仅plan-941 scope，未写新Spec/legacy specs.json或操作别会话。

T-01尚欠**新Q4/ARCH完整原子义务、全部域具体原预算/完整生产者及后继owner映射**（map明确legacy in-process budget仍待审，不套AV/native默认帽）；新全registry、全机制reader及T05..13未关闭。当前D936/938/939待真实交付只限制所需consumer/最终T12，不阻后续独立审计；下一轮以完整Q4/ARCH表和预算收口为主，不将251 locator/100控制等同系统交付或完成验收。

stage: work | plan_id: PLAN-941 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-941/auto-lang | branch: plan-941-dev | code_commit: 07874db1ab270c3a4a3bee51a91dde979b2bd624 | task_ids: T-01/T-02（未关闭） | evidence: 完整313显式locator/gap表、100冻结控制、canonical旧4实测与191件提交blob对账、20首链/14旧格位与46.7MB峰值 | blockers: 无新增用户决定；完整Q4/ARCH/预算映射与剩余实现 | next_responsible_role: work

stage: goal | plan_id: PLAN-941 | plan_revision: r1 | round: 10 | from: work | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-941/auto-lang | commit: 07874db1ab270c3a4a3bee51a91dde979b2bd624 | next: work（余下完整映射/实现，不是final review失败）

### 2026-11-06 完整补例语料实体化、分域预算与Q4源登记（goal第11轮）

上一轮为progress（完整旧313映射、100控制、191件blob对照）；本轮host实际active，main零代码WIP，941唯一树复用并保持冻结后clean。施工提交`aea20fb66`→`633bbe92a`→`f0d32842e`→`6db57d4a1`→**`656f48cb4d21d10a985af01ee9eabb58e20c6c87`**。仍executing **0/15**，没有把登记/原门绿冒新Q4严格执行或final review。

- 审计明确“C首例28不等完整域”：925还承诺模式/方法、Unicode/NUL、TextIndex/InvalidScalar等补例。将原`a2c_rt_cert`的8模式+3trap私有Vec、原`a2c_rt`的10 text/char源码/手算oracle搬成可读typed函数，并让**原测试继续使用同一表**；926原20 cert CASES仅开放兄弟模块只读。无源码/Rust解析器、无新语言执行器、不改原source/oracle/reference语义。`composite-literal-preservation-audit.json`证明原10源字面量顺序、原CertCase字段字面量序列完全相同；原有runtime #[test]入口保留。新ignored只读导出`inventory_941::read_composite_contract_inventory_941`原生nextest精确选择1/1，发现**41**合同（11+10+20）、execution=0；scope内create_new、词法祖先reparse先检后canonicalize，旧scalar导出payload不改。
- **长期源输入入库**：61份scalar源码逐字节取原typed cases()（表SHA保持9007cdd1…），41份composite源码及31份原手写独立C reference逐字节取原typed表/原926文件；均为后续消费者需要的产品fixture，不是runtime证据。`scalar-obligations.tsv`新增明确source_file；`composite-obligations.tsv`保41原family/profile/caps/value/trap与reference身份，**34 value+7 trap**，其中10个text/char原测试没有独立reference，显式列`reference_gaps`/owner941，未伪造认证。新reader在每次准入重hash三份原Rust合同、实际source与reference；未知域/改profile/caps/错trap/删除/重复/假PASS/路径逃逸拒。source文件不再依赖本机临时导出来供只读准入。
- 原冻结标量里**两个CRLF是有意语料**（core-p20-utf8-crlf-min、sc-crlf），不能像生成metadata一样归一LF。最初Git input规则会改这两件blob；以`.gitattributes`对新scalar-src/composite-src/composite-reference目录声明-text，重暂存后保持原CRLF字节和原SHA，不更改旧母语料/批准文件。其余metadata保持LF。最终源/reference/预算167件及Q4引用121件分别实际`git cat-file --batch`核对工作字节=提交blob（两清单有交集，不相加为唯一数），收据`composite-budget-committed-input-audit.json`、`q4-source-committed-input-audit.json`。
- **原实际职责组**：冻结aea20fb66 `cargo t ace-a2x infra-ace-m1-arch --out <scope>/composite-contract-frozen-gate` exit0、gate_passed=true，**192/192实际原A2X测试 +106/106治理控制**，2个按原条件ignored的只读导出不充语义执行；175.6240846s，results SHA256=`2287f6bf2fcb645bff5a80c8a6c10b93bbb51ada7ef429aa3226b344d277cd71`。包含原text-char10、cert模式8/trap3、全scalar61/完整原资源与证书反控，未减原样本/删除条件。后续只变Python/静态合同/fixture Git属性，原Rust源码、参考/工具与运行输入未变，复用该192结果；不为数据/提交重复完整编译组。它是原职责门，不是新组合器已严格认领全部Q4产物/计量的证明。
- **原预算16域**：新增`original-budget-contracts.json`及`ace_m1_budget_contracts.py`，26个当前原code/Spec hash绑定；区分Case整suite外层1800s/64MiB stdout/16MiB stderr、nextest900s×2、仅ac适配器60/5/70、AV/native/ABI原IPC窗口、public ABI真正60/5/70及hot8MiB+staging/index8MiB、CT 10s/逐AV2s/8+16+8MiB三分区/private384MiB/tree512MiB、A2X暖管理48MiB、scalar/record-text/collections各自原负载/30样本/P90/512修订、941显式捕获与组合自有帽。A2X daily/旧avm-a2r无额外inner deadline/stream cap的真实代码事实列None，不能借ac默认或把None填0；外层原门仍有独立作用。warm cold分开、115函数/273575B scalar不借给40函数/16helpers/32callers/16KiB+4KiB的collections，RT原24callers与256 free-body+256 method也单列。所有政策**runtime_verdict=UNOBSERVED、can_claim_budget_pass=false**；读到声明不等实际预算已过。旧313图中的legacy budget从pending-audit改连该已审声明，实际预算观测仍未充绿。
- **Q4逐源登记**：新增`q4-source-contracts.tsv`，现明确**126源合同、504个四目标必需格**：scalar61、cert模式/trap11、text/char10、collections20、record/text首例5、934 source6、935 instance6、936有限源7。119有当前真实文件/独立原oracle，936七项明确`pending-provider-source`，不读取未交分支冒D、不伪造source/profile/caps；其中bool原承诺未给可冻结数值，单列pending-provider-oracle而不猜42。四端固定AC-native/AV/ACE-A2R-HIR/ACE-A2C-HIR，不按能力交集删C/换legacy。marks按原Vec与trace能力位共同区分无trace/有trace但空事件，不改oracle。这是**已登记源行，不是已封闭完整新M1分母**：reader仍scope_closed=false，剩余原源变体/完整ARCH机制映射和真实D尚待收口。
- **最终治理冻结门**：`cargo t infra-ace-m1-arch --out <scope>/q4-source-budget-final-controls` exit0、**114/114**，2.818744s；results SHA256=`58f43d99b84dc116e6b29d0e75b1a935b2b9023df3cccf523f9d50e11dad581a`。新增composite6+budget4+Q4源4控制；包括不可削分母/换目标、错误source/reference hash、ABI与IPC/outer帽互换、预算zero/bool/假PASS/篡改、未来source/profile/未知bool oracle伪造拒、CRLF保原字节。diff与Rustfmt通过，无新增Rust warning。
- 最终原capture/native_meter实际只读CLI沿`full-map-canonical-bindings.json`：exit1，20首链完整、旧严格执行仍**14/313 PASS、299 UNOBSERVED**，新增源/预算发现都不算执行，全局claim=false。`q4-source-budget-final-preview.json`49594 bytes，SHA256=`9fe76f18c9c39c569988d0be78804318c3c3c6c22174e5b15d12c9b0d59ebace`；874.5552ms，whole process peak20,410,368 bytes<128MiB，当前reservation30,583,032、峰值46,727,808 bytes<48MiB。16域声明及7未来源在真实报告可见，没有因扩表放宽组合预算或隐式启动工具。该单轮不是正式P90采样。

保留首次失败/纠正：初次native导出漏NEXTEST_EXPERIMENTAL_LIBTEST_JSON被CLI前置拒（exit95，未执行），设原开关后精确1/1；第一次定向命令传多个--test-name，原入口只保最后值，实际仅trap 1/1，**未当成3个测试**，后续完整192统一覆盖所有原入口。新Q4反控一开始误把无trace的原marks Vec[]当None而1/4红，按原typed表改为Vec[]+能力位区分，最终114全绿；工作中6db57d4a1为短暂红中间提交，656f48cb4修复后才冻结。完整原输入/预期保守，不把源字节/期望改成观察输出。

**临时磁盘故障已解除**：写Q4 TSV时ENOSPC，留下0字节未跟踪文件，未算完成；核对941独享idle incremental缓存后，递归清理与更窄仅dep-graph.bin文件清理均被**自动审批拒绝：只返回“blocked by policy”**，均未执行，agent删除0文件。随后外部状态释放空间（0→约61MiB→15.26GB，最后检查12.59GB），不归功本agent/不操作他方目录；在足够小写入空间出现后完整重生39400字节TSV、hash校验/提交/114验证，再核所有blob。`disk-pressure-incident.json`保过程；不再留待授权问题/不设blocked，后续Cargo仍先查实际≥2GiB。原证据/当前producer二进制均保留。

T-01现已补实原预算声明与首批完整源族，但**完整ARCH-01..08原子映射、剩余原source变体/reader闭包、完整new registry仍未完成**，T01/T02不先行关闭。936当前reviewed r2 13/14仍无D，938/939未交齐；只限制其consumer/最终T12，不阻继续独立工作。下一轮重点转完整ARCH与registry结构收口，继续原941目标；不以114控制、192原门或504登记格代替未执行义务。

stage: work | plan_id: PLAN-941 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-941/auto-lang | branch: plan-941-dev | code_commit: 656f48cb4d21d10a985af01ee9eabb58e20c6c87 | task_ids: T-01/T-02（未关闭） | evidence: 原41补例及61源实体、31原手写reference、16域预算、126源/504格登记、192原职责+114治理冻结与canonical blob/预算实测 | blockers: 本轮环境故障已解除；完整ARCH/registry及剩余实现仍欠，无新用户决定 | next_responsible_role: work

stage: goal | plan_id: PLAN-941 | plan_revision: r1 | round: 11 | from: work | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-941/auto-lang | commit: 656f48cb4d21d10a985af01ee9eabb58e20c6c87 | next: work（完整目标继续，不是final review失败）

### 2026-11-06 ARCH原子义务和分域登记索引（goal第12轮）

本轮沿原授权继续work，唯一941树、main零代码WIP。冻结施工提交 **`717278a6194fd9dc74ad88ae16b1654e7d4114d7`**；仍executing **0/15**，不把schema/locator/控制数当实际机制交付，不缩减原941范围。

- 新长期源合同`architecture-obligations.json`（47280 bytes，SHA256=`6332fa16b812fd81405c45b42e6c7f4d7eff3faec98919b5b4d5f447eda630c2`）把ARCH-01..08展开为**49原子义务**，含29个入口（12 Case套件、14 native模块locator、3 provider-pending），逐项列真实owner/941证据owner、原预算域、V-01..18、独立closing oracle、错误实现反控和所需原始观察类别。契约定位是待核验执行入口，不声称每个原测试已经证明该完整机制；尤其公共恢复首操作、所有required页、真实AV CT、历史owner/迟到发布/最后lease等仍须实际观测。
- 新严格reader`ace_m1_arch_obligations.py`核固定合同hash、49不可削ID集合/8族、29入口闭包、实际固定组/套件、native package归属、当前文件hash、预算域/观察种类/完整V集合和provider owner。所有locator都executed=0/UNOBSERVED；不接受child PASS注入。纯AND只在内部已准入结果/隔离控制使用；逐一删除/置FAIL/置UNOBSERVED任一49义务均红，外来ID/bool/PASS对象拒。原始合同中的oracle被改即固定hash拒，声明不能晋升执行。
- 根`registry.json`显式绑定旧313map/Q4逐源表/ARCH合同/原预算4个当前文件hash；新`ace_m1_registry.py`接入真实只读组合链，索引**886已登记记录=旧313 + Q4已登记504 + ARCH49 + 首链20**，四个namespace分开，**这不是最终M1分母，也不把重叠验收层相加成覆盖率**。剩余原source变体与全域reader闭包未齐，`required_universe_closed=false`；即隔离全部886 PASS也不能关闭完整门。严格旧执行结果和首链实际reader输出才提供已支持部分verdict，Q4全表和ARCH不得据source存在/原组绿补绿。
- 冻结职责门：`cargo t infra-ace-m1-arch --out <scope>/architecture-registry-frozen-controls` exit0、**121/121治理控制**，3.3587846s，results SHA256=`22973bd90c392feff00632a2c10fd5a59ccb872b7bf386acbd3b6e6d78b4f291`。新增7个有实际断言的控制；49删除等为其subtests，不冒额外语言执行量。复用上一轮192原A2X测试（本轮Rust/原输入未改），不重复编译原职责组或跑全仓；最终T12并集和独立review仍未执行。32件新合同/入口/组/reader的工作字节逐件等于冻结提交blob，`architecture-registry-committed-input-audit.json`。
- 冻结真实CLI经原process_capture/native_meter：`python scripts/test-governance/ace_m1_arch_acceptance.py --first-slice-preview --repo <941-tree> --evidence <scope> --bindings full-map-canonical-bindings.json`实际exit1（preview拒认领），首链20完整、旧**14 PASS/299 UNOBSERVED**、49 ARCH全未观测；登记记录34 PASS/852 UNOBSERVED（34=首链20+旧14，不能混作语言覆盖率），旧38认证差异和完整范围门仍红。报告`architecture-registry-frozen-preview.json`124148 bytes，SHA256=`03ee44311fcab67a3d9125eb3a78bd08188e88765da6134ec2d62678ed9f35be`。632.9666ms，whole process peak20,619,264 bytes<128MiB，共享当前reservation34,279,897、生命周期peak46,742,224 bytes<48MiB；单轮非正式P90。17个reader hash入报告。
- 原same-wrong反控在新组合链重放，且Popen禁止验证无隐式工具：native帧+摘要一致改错误f64/同bits并重hash，W01:ac FAIL（登记33PASS/1FAIL/852未知）；再AV同错，W01:ac/av同时FAIL（32PASS/2FAIL/852未知）。49 ARCH仍UNOBSERVED。`architecture-registry-counterfactuals.json`明确为隔离副本，不是真实执行；未被索引/对象释放掩盖。

保留首次失败：首次真实CLI exit2 `discovery summary exceeds retained reservation`，49行完整oracle对象超过原512KiB摘要预留，未放宽48MiB总帽。reader返回后只保id/owner/entry/budget/V有界摘要（原oracle仍在固定hash合同）；886索引按实际对象成本显式预留3MiB，不清峰值/不释放存活对象，最终上述保守总峰值通过。首次121控制通过在最后压缩前，最终冻结门另跑121/121；未用前者冒最终revision证明。

**新的依赖事实**：本轮后段只读发现936已实际交付 **D936=`a41dbd1bd4dd214198565f54abc76a423735c6a0`**，main中归档/merge pass/5 canonical路径核验及guard清树、scope删除有收据；不能继续称D936未交。936 frontmatter仍13/14但T14已勾/交付收据存在，以实际交付/源码后续核对，不凭归档字段推全部941验收完成。其最终收据明确新lane正式30样本、worker级V14及暖会话CT复用留后继：**这些若为941原required，仍在941实际观测义务内，不能随provider的非blocker分类消失**。当前941冻结树尚未同步D936，故其本地合同中ct/pending七源暂时准确表达“本树尚未消费”，下一施工单元需同步稳定D936、更新真实source/profile/oracle及源hash/observer接口再认证。938仍executing6/13，939 drafting0/14；没有操作其树或请求其发送消息。

下一步先同步实际D936及必要主线依赖、按原计划核对7CT源和required页/计量，再补真实Q4和机制reader；剩余原源变体、全registry封闭、严格执行实验、SD/完整冻结门/独立review/merge继续，不以本轮886登记替代完成目标。无新增用户决定，不重授已授权修复。

stage: work | plan_id: PLAN-941 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-941/auto-lang | branch: plan-941-dev | code_commit: 717278a6194fd9dc74ad88ae16b1654e7d4114d7 | task_ids: T-01/T-02/T-03/T-04（未关闭） | evidence: ARCH49原子/29入口、886分域登记、121冻结控制、32blob一致、真实CLI与same-wrong反控、D936交付只读确认 | blockers: 无新用户决定；D936待同步消费，938/939在途及其余全域实现仍欠 | next_responsible_role: work

stage: goal | plan_id: PLAN-941 | plan_revision: r1 | round: 12 | from: work | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-941/auto-lang | commit: 717278a6194fd9dc74ad88ae16b1654e7d4114d7 | next: work（用户resume继续原完整目标；不是final review失败）

### 2026-11-06 D936同步与真实worker接缝反例（第13轮在途检查点）

已在唯一941树合入master到`db7b869b8`，D936的语法/源CT/实例/标准库等实际依赖无冲突进入树；main既有他方docs-only WIP未纳入。施工`79c24f1d4`将七CT源从待交付转真实source-bound（126/126有源、四端504 required不减；bool主程序原oracle为7），根登记加入原14检查typed快照；新CT严格reader读取原Case/wire/每端独立oracle/caps，不把cold值正确等同CT hit/restore机制。ARCH ct入口转实际Case+source-cte模块locator，29→30入口、49原子义务不减；仅938/939仍provider-pending。原预算两Spec pin及935恢复测试pin按实际D936变动重锚。

原`cargo t case-ace-source-cte-936`首轮准备av时`--locked`拒（未执行），记录`ct936-original-after-sync`。随后离线重生成锁并跑`ct936-original-valid-lock`实际**14/14**、146.6677171s；严格读取14项值全部对独立oracle，但该轮使用了扩大第三方版本的中间锁，**不是最终冻结/交付证据**。新Python定向控制实际**122/122**，2.548s（`ct936-governance-updated.log`），新增caps必须匹配独立binding及same-wrong仍红；原3个测试因119/126、29/30及profile已交付的过时预期首红已按实际交付纠正，未删义务。

锁文件事实矫正：experimental/ace/Cargo.lock是tracked文件；最初generate-lockfile造成145增/66删的无关第三方更新，没有纳入79c24提交。发现后停止本树runner PID27232（当时Q4/SYN已有partial PASS，**整轮中断不算冻结通过**），拟保存为`aborted-fixed-runner-owner.json`的迁移操作被自动审批拒，实际文件仍原路径。原HEAD锁恢复后以`cargo metadata --offline`最小补齐，仅**21行增加、无第三方包版本升级**（新增已合入stdlib-conformance member和既有依赖边），提交`a59fe5232`。无Cargo/worker进程残留已核，后续固定门需清理原dead owner锁目录。

**F-941-03：已归档源CT的真实A2X worker入口未接前门（待独立修复授权）**。本树在最小锁上显式构建当前a2x-worker成功（54.47s）。对同一个真实worker、同profile=scalar、entry=main、独立oracle42进行8次原操作：普通Q4源的Rust/C daily均exit0、Rust/C certify均exit0且实际/reference PASS；原936 sum42源、caps=[ac.source.cte.scalar/v1]的Rust/C daily及Rust/C certify **4/4 exit2**，均`source.syntax: expected expression, found #`（C包装为a2x.compile）。原始8份request/raw stdout/stderr/Job meter/worker及source hash在`ct936-original-worker-entry-probe/`，汇总`ct936-worker-entry-results.json` SHA256=`5f9737861d33b613bc6f1b7d822b2d559f5137eaeb4bf68378bb6a575c919ba7`。不把A2xSession的ct_lane_936测试或发射文本含42当真实worker已接通。

源码直接链：`experimental/ace/crates/a2x-worker/src/bin/a2x-worker.rs` helper `compile_scalar_product_934`（42行）仍无条件调用source-query的普通scalar入口；Rust emit（207行）和C emit（376行）都走它；936在A2xSession层接通的CT前门没有覆盖这两个真实worker分支。**具体最小提案**：独立新修复Plan在该共用helper按已授CT capability委托既有ace-source-cte::compile_scalar_ct（沿真实AV/结构回灌/事实车道），普通源仍用已有source-query入口；不造parser/solver/store，不放宽caps或预算。补原始worker两个daily和两个cert真实入口的7源回归（bool7/meta42→43/实例42/子CT40/体编辑43）、无CT能力拒绝及原普通源接受；独立复审、合入后继续941。941 §5.4原文“已归档能力发现新缺陷要具名新修复流程”，故不能在941组合器内模拟或直接修原owner功能。尚未创建修复Plan/改该producer，待用户明确授权范围后立项。

**自动审批实际阻拦**：中断门留下`D:/autostack/.wt/lang-941/auto-lang/target/test-groups/active/owner.json`，仅该1文件，owner PID27232已不存在。带路径/reparse/内容/PID核验的“移动owner.json到941 scope并删除空active目录”整条操作被自动审批拒绝，理由仅`blocked by policy`，没有执行、没有清理成功；不换API/壳重试等效删除。新固定门调用在前置以WinError183拒绝，未运行。已请用户手动删除这个dead-owner目录；在此期间继续独立合同/真实A2X认证探针，未以另一工作区或绕锁门顶替正式固定验收。

当前进度仍0/15、executing；122治理/中间锁14值证据/8原worker反例不等整个系统已交付。D936已交付，但真实worker四方向仍红，CT缓存/恢复观测和原30样本仍未核销；原38旧cert差异照旧保留。待原worker最小修复授权和dead-owner目录人工清理；不影响其余已授权reader/源映射施工。

stage: work | plan_id: PLAN-941 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-941/auto-lang | branch: plan-941-dev | code_commit: a59fe5232 | task_ids: T-01/T-02/T-03/T-04/T-05（未关闭） | evidence: D936实际同步、7CT源126/126、14原值检查及严格读取（中间锁非最终）、122定向治理、最小21行锁修正、8次真实worker正反复现F-941-03 | blockers: 独立worker修复待授权；dead-owner目录清理被自动审批拒需用户手动处理；不影响其他源/reader施工 | next_responsible_role: work（其余已授权义务）/user（两个具体动作）

stage: goal | plan_id: PLAN-941 | plan_revision: r1 | round: 13 | from: work | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-941/auto-lang | commit: a59fe5232 | next: work（保持原完整目标；F-941-03修复和固定门恢复待用户动作，其余reader/独立reference继续）

### 2026-11-06 文字补例独立双端认证与958修复交窗（goal第14轮）

941施工提交 **60736ac261bcfb31ce6d7438bed56f34e415d458**，树clean；原目标不缩，仍executing **0/15**。本轮实际补T05一段真实认证链，并沿用户新授权推进独立修复958；不是等待/仅复述状态。

- 原925十文字/字符补例原表没有独立reference（原31份C reference与其10个历史缺项仍如实保留），本轮另增**10份手写C+10份手写Rust参考程序**，不冒原作者已提供。C用有限已知UTF8数据及独立码点/边界算术、长度含NUL；Rust用std chars/bytes/char::from_u32，均不读目标emitter/runtime/源码解析器。独立TSV `text-reference-contracts.tsv`固定SHA=dcd2744a0dc827e555dff7474a5c335425d393b813d6862584b453886cf04752，十原source/profile/caps/value/trap及两ref SHA逐件绑定；root expanded_contracts加入该第五具名源合同。新reference -text属性保字节身份，30件本提交工作字节=Git blob实际对账（scope/text-certification-committed-input-audit.json）。
- 原a2x-worker实际运行**20个显式认证**（十源×Rust/C），actual/reference各自编译运行，总40编译+40运行阶段；另各原daily先现场发射，以预认证fileset hash冻结产物身份，oracle只取原手算表。`text-char-original-certification/`保60个原worker调用（20 identity/20 daily/20cert）及4子进程原始终态、完整manifest/证书/source/reference哈希和原Job计量。所有20认证strictly admitted；逐个对独立值506/33/1/233/3/20013/112或TextIndex/InvalidScalar，未以双方相等/child PASS冒oracle。daily原记录尚无完整新domain准入，AV/native此十源也未读取，故报告仍UNOBSERVED，不把20 cert冒十源Q4完整。
- `ace_m1_cert_evidence.py`保scalar入口并新增有限text认证：请求/recipe/证书/manifest家族、profile/text-i32-draft、原target/toolchain/flags、source/ref/完整fileset、原raw compile/run4终态及具名trap逐层核。C版本摘要原locator用BLAKE3的事实保留，不把字段名当可重算SHA；Rust-vV hash独立重算，Job缺峰值/坏峰值拒。`ace_m1_text_evidence.py`完整20集合去重/缺项拒，核独立reference合同后接只读组合器及原504 Q4对应slot；cert完成不自动认领ARCH/whole M1。原scalar两端另以当前worker实际/reference重认证2/2并由修改后的原reader接受（text-reader-scalar-compatibility），验证新text分支不回退旧strict scalar。
- **实际raw副本反控20项**（text-certification-raw-controls/results.json，显式counterfactual_not_execution）：2代表源（值/具名trap）×2target×5控制，4合法重定位副本PASS；actual/reference同时改相同错值507或错trap InvalidScalar、child flags仍PASS并重hash，4项均FAIL；换scalar schema、双manifest/cert同时错domain并重hash、compile exit改bool，12项REFUSED。Popen禁止证明reader不隐式执行。另新增永久5测试覆盖4终态/独立值和trap同错、NUL/Unicode/source与ref字节、compiler flags/version身份。
- 用户已手动清理原dead owner目录，实际Test-Path=false，固定入口恢复。冻结60736 `cargo t infra-ace-m1-arch --out <scope>/text-certification-frozen-controls` **127/127**、exit0、2.9485956s，results SHA256=`1635349db3d89573138b0ce834b3f4c1ac7a96268bd119e71edcdfc8edc9e340`；没有把此前直接127单测或中断门冒本次冻结门。真实只读text domain经原capture/native_meter、禁Popen：20 cert complete，全局claim=false，424.6714ms、peak process22,507,520 bytes<128MiB，当前保守reservation1,048,576/peak4,763,294 bytes<48MiB（仅该domain，不是whole组合预算复测）。报告10013 bytes，SHA256=`8a885e6096aa68d29130a62b2c3b14e662ad36b617d21a078ee31179a4771a6c`。原20编译结果已明确，不把门127控制合算语言覆盖。

保留首次错误：采集脚本首遍错误套C scalar header名a2x_c_runtime.h，首Rust认证后在C daily阶段FileNotFound，未当完整20成功；按原RT实际a2x_c_rt_runtime.h修正后只续未完成认证。新certreader来源变化使预算合同两个控制STALE，未放宽原帽或改oracle；刷新本包真实来源pin后冻结127绿。未跑全仓、未复用旧worker SHA为新工具背书。

**F-941-03已获授权，独立PLAN-958实际执行**：用户本轮明确“授权独立修复并继续”，无待同范围授权。主检出原子取号958、计划a63a20b4e、唯一lang-958树（auto-down只读58ea33f），原producer共用helper接既有CT服务并补直接依赖。code **688207b67**，实际check18.81s；七个永久真实CLI测试 **7/7**，29.553495s，40原CLI操作（七源28+普通/无cap8+错oracle/reference4），另32次运行现场新编译出的actual/reference exe逐raw字节对独立oracle。results SHA256=`8ff8425c432259270655f0b0845de2db11f37baa1414f4b2597a177e7205e3d7`；原189项因具名过滤未执行不充绿。原5测试首轮2项因误读主线没有941 opt-in观测字段而红，按原已有owned facility直接执行现场两腿修正，不搬本包未合入仪表代码。958工作4/6，execution_done，独立review与完整职责组/merge/归档/清理仍待，F03尚未关闭、不先消费未交958代码。

本轮只读查官方OpenAI Docs并答复用户policy问题：当前拒绝只返回blocked by policy，没有规则ID；本地config未见auto_review自定义策略/用户rules，不假定AGENTS或sandbox权限能覆盖host审批。官方支持完整[auto_review].policy替换且managed guardian优先，以及支持的CLI /approve精确重试；没有修改用户安全配置、没有换壳绕拒。用户清理后入口已恢复，无环境待授权项。

stage: work | plan_id: PLAN-941 | plan_revision:r1 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:60736ac26 | task_ids:T01..05（未关闭） | evidence:20新ref/20真实text cert/20raw反控/127冻结/2scalar兼容，958已实现并交review | blockers:无新增用户决定；958尚待独立review/merge，941其余全域义务继续 | next_responsible_role:review958→merge958→work941

stage: goal | plan_id: PLAN-941 | plan_revision:r1 | round:14 | from:work | to:work | status:executing | outcome:needs_fix | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:60736ac26 | next:review958（独立修复交付后继续原941，不是941 final review失败）

### 2026-11-06 D958复审、完整合入门与941实际消费闭环（goal第15轮）

上一轮为progress：941文字20双端认证及958冷worker实现；本轮继续原941完整目标，未重定义完成。独立修复958已**review/merge/归档/清理6/6**完成，F-941-03关闭。941仍executing **0/15**，余下完整registry/reader与实际机制验收继续。

- 958新复审轮按r1/实际diff/原始nextest事件重新重建AC，说明同一会话主agent的限制，未冒称另一个模型/会话。自选10项真实holdout：有CTcap但无#的ordinary程序两端daily+cert四项接受；仅instances cap/错误CT/v10能力各两端daily共四项syntax拒；CT实际除零两端daily返回source.cte.plan/cte.trap/av.trap.div-by-zero两项。daily实际Job process count=2，拒绝零产物，无target编译。7源原40操作/32现场exe重放证据与源码/SD通过，reviewed688207b67。metadata/phase/budget未扩大，source-cte/solver/AV原实现不动。
- 主线只增计划簿记，958 rebase `688207b67→06251b0ccb4916d2e70abd7f98197987b778ca9e`，range-diff单patch完全等号，所有ACE/code与两SD逐字节等价。最终完整固定组 **cargo t ace-a2x，196/196 PASS、0skipped**，发现716.9643ms、执行/清理131639.6381ms、整132.3891907s；results SHA256=`4495c24c08ef3cf04e4568418f299868e265da1cbcb11057df79bc4706fa88e6`。首轮182pass/14fail确认为新树缺原release worker（NotFound/prebuilt要求）前置失败，未进这些认证；显式release构建59.38s后才跑全196绿，原失败和120.1627227s/SHA3022b10d...已折958，未缩分母或改expected。所有原189+新7保原条件，没跑全仓/性能门。
- **D958已落master ff-only**，delivery06251b0cc；canonical两Spec逐字节=reviewed/delivery/current blob，source-query-integration SHA=eafa1082c933454dac0adeec25d70937c13c5a1a71a65e710304935c8526c208，a2r-hir-certification SHA=a32d3555b5657cb72c2ae909ce2ac000110662ef5aa46e218f96c7895561581a。归档`docs/plans/archive/958-a2x-worker-source-ct-frontdoor.md`。958/auto-lang与auto-down两树及分支/空组已由用户清理，Git列表/路径实际全部不存在；材料目录首次核验仍2034文件/44,395,461 bytes，未虚报清理。用户随后“材料已删”，实际scope不存在后才结6/6。
- 自动审批对958整条收尾删除再拒（仅blocked by policy），**agent该条操作未执行，也未换方式重试**。删除前两树guard clean、无reparse/活动进程，worktree文件快照lang29,765/4,149,099,644 bytes，down1,256/26,409,169 bytes，scope2034/44,395,461 bytes均已折958；无target/exe/PDB保留迁移、无具名持有。全局登记成功刷新；仓库本地登记同步时JSON报错，发现scope数组末尾孤立结束括号/逗号（已有deleted_scopes958记录），只修该标点并保全936/935/938/941与958删除记录；原坏文本在941scope/retention-local-before-syntax-repair.txt。先前脚本局部失败后shell仍提交了档案，此处补齐了本地登记并复核两份JSON有效，没有以该中间提交冒收尾已全部执行。
- **941实际消费**：唯一941树合master得到 **d88359466a855153ca2d464913cedf6a6e7db4e4**；旧接口观测接线与958helper/lock/conformance自动合并无冲突。原release worker重新构建2.84s、既有core dead-code warning1，无新warning。原8入口逐条重放**8/8 PASS**：普通源四方向保绿，原# CT四方向从exit2转exit0（两cert实际/reference四phase exit0、raw value:42换行；daily spawn0/Job2）。汇总`after-d958-original-entry-results.json` SHA256=`93286914b78cf0b5a5c5cea117bf0c70bf5ca2d15ba0f163d9993973bdc4b68c`。这是原F03实际反例复放，不是只git合并/静态函数存在；F03独立修复流程与消费者功能均关闭。
- main已装旧worker仍2026-11-05T05:29:34.608315Z，SHA e2ba687e...，实际CT smoke仍source.lex拒#；合入958已精确记录stale项而非声称已部署。941本树tool已新鲜，941最终交付继续按实际生产artifact做新鲜性检查，不用main旧tool证据。旧38 legacy-A2R认证差异保持，不因本HIR CT修复充绿。
- 复核941预算/来源合同所有现有pin在D958后**零漂移**。原准备脚本误预期certification Spec在该26源pin集合内，assert在写入前拒（source pin drift []）；没有改任何合同或造新hash，没有新的治理红；127有效治理证据的相关Python/static输入未变可复用，不因仅HEAD变化再跑。下一工作单元要新增有限CT cap认证reader及七源独立ref，届时按真实修改重核。当前worker二进制SHA已变化，旧首链两cert与文字20cert是先前工具上的有效历史证据，**不冒当前工具的新鲜准入**；须与新的CT14 cert一起刷新后再做全组合报告。

958批门due核对：956最新收据仍<48h、958非5倍数，之后未有新的5倍数交付，不due；旧批37组红及957编译器triage保持原结论，无all-green声明。main代码零WIP，仅原他方docs-only在途保留；全部本轮源/证据只在独享941/958的登记范围，958已全清。

下一步恢复941 T01..05：基于已交付CT前门完成七源14份独立Rust/C参考/实际认证、严格原4phase与caps/source/domain准入，并刷新受到新worker身份影响的既有cert；剩余source variants/ARCH机制、全部required reader和938/939真实接口继续保原义务，不用8原操作代整个M1签章。无用户待决定项。

stage: goal | plan_id:PLAN-941 | plan_revision:r1 | round:15 | from:work | to:work | status:executing | outcome:needs_fix | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:d88359466a855153ca2d464913cedf6a6e7db4e4 | next:work941（D958全部关闭；完整目标和原阈值保持）

### 2026-11-06 CT独立参考/双端认证接入与全组合当前工具刷新（goal第16轮）

941施工提交 **f0ef7b36fd88113091af53f3563cec4619b99688**，代码树clean。958清理已完成，不再待用户；本轮实际推进T01..05，941仍executing **0/15**，原AC/预算/实验规模与完整目标不缩。

- 新增CT七源**14份独立Rust/C参考**和12字段TSV，固定SHA=`4fa162f7fe6722f540fb9c90a45fd89ed13b7a938d2ebc852c303ed8733dda69`。原sum42/callee-body43/bool/meta-fields2/meta-fields3/choose-instance42/child-ct-chain40独立oracle为42/43/**7**/42/43/42/40；meta与instances能力分别绑定，不把Bool原程序出口误成1。参考只用本语言的简单算术/条件/静态数组，不依赖目标emitter、runtime或源解析器。登记execution仍UNOBSERVED；root第六具名合同cte_reference不冒执行。24个提交文件工作字节=Git blob核对，记录在scope/d958-current-certifications/committed-input-audit.json。
- 新有限CT认证reader先核固定reference合同，再核每源两target恰好14集合、case/source/ref/current worker/profile/caps/toolchain与原4进程终态，接到原504 Q4 target slot。Rust manifest严格绑定`auto.source.scalar-i32.draft`及scalar-i32-draft；原scalar无cap和text/具名trap分支保持。能力必须实际请求/证书/独立合同一致；child PASS或两腿相等不能替代手算oracle。CT值认证不认领CT缓存/hit/restore/ARCH；daily的完整准入也未由cert自动填绿。
- 首次在D958手工构建worker上完成CT14+text20+首例2认证，均真实actual/reference编译运行。随后全组合如实拒绝旧AV worker SHA，未仅更新旧证据hash。冻结 **cargo t infra-ace-m1-arch case-ace-m1-q4-941 case-ace-unified-source-929 case-ace-ca-d-scalar-937 case-ace-m1-legacy-941 case-ac-av case-ace-m1-typed-native-891 case-ace-source-cte-936 --out <scope>/d958-current-certifications/frozen-consumer-gate**：8组**200/200**（治理131+Case69），gate=true，115.0840698s；results SHA=`1d66f7f6f174d98d3e3e19113dd9dee1cc5363031f010b05167f4559317f2b94`。治理131执行3096.8865ms；7 Case依次4/11/6/4/18/12/14，原选择不缩；生产Rust源码本轮零改，不额外跑全ACE/全仓。
- 固定入口按既有参数重建worker，a2x最终SHA=`a83a46c946f80ceb681f6bbdfad934b71b59db813500aae58977922c0a7e78f2`，与先前手工构建不同。**构建全部结束后**再次实际捕获CT14+text20+首例2，共36认证、72编译+72运行原终态，并每target先现场daily冻结actual fileset、identity冻结编译器；108个原worker调用。最终材料唯一在scope/**d958-final-worker-certifications/**，旧工具证据保为历史，未混用。后续Case重放工具hash实际仍相同。7份原identity-inventory /2与parent-inventory亦由当前auto-case重新导出，严格读取无Debug解析/推测。
- 首次200门未开启AUTO_CASE_WIRE_EVIDENCE，Case报告绿但无原通信帧，全组合按`required original exchange missing`拒；未据父PASS填证据。仅为补缺开启既有开关重放7个受影响Case组，**69/69**、94.6862898s（包括准备），原始wire/terminal/artifact-copy完整；scope/d958-current-certifications/frozen-consumer-raw-gate/results.json SHA=`c8acc9a1598030e0410f6631ff33093abac967634e5523f48d895b445d8ea261`。未重跑已有效131控制。最终所有父Case非PASS行扫描为空，原required旧格仍14/313，299未观测/38旧认证差异不填绿。
- **当前完整只读preview实际成功**：新CT原AV/native14值+CT14 cert+text20 cert+首链20、原旧typed/native/ABI/query/syntax及登记/预算/ARCH一起读取；Popen禁用，无隐式工具启动。registered886中**82 PASS/0 FAIL/804 UNOBSERVED**，required universe仍unclosed、can_claim_expanded_m1=false。共享预留当前36,698,919/峰值**49,139,156 bytes <48MiB=50,331,648**，没有抬帽/重置峰值；实际单独Job计量1121.6977ms、peak process27,492,352 bytes、peak Job28,733,440 bytes（private<128MiB），报告147,993 bytes，SHA=`385237585a403ee9e1a54cd12e124fa1362d57200615b6d393f1bb8048cebd23`，路径scope/d958-final-worker-certifications/metered-preview.stdout；计量和stderr原件同目录。这是当前全组合读取证据，不是完整M1通过。
- **当前CT原raw反控56项**：14合法完整reader PASS，去CT cap/独立值改错28 REFUSED；actual/reference原run帧同时改相同错值，14对原独立oracle均FAIL。后14明确仅4phase oracle层反控，不冒整证书重hash反控；全部标counterfactual_not_execution。结果SHA=`1703c75270a94b591a1657d353d184c1831430a114b6d8365c64705631fb60b3`，路径scope/d958-final-worker-certifications/cte-reader-controls.json。新增永久4控制验证七源oracle/cap、源及两ref字节漂移、缺cap/错版本/重复/异类能力、重复/缺失target集合；总131通过。此前text20 raw对照控制仍是历史工具上的有效控制，不重标当前执行。

保留首次采集脚本错误：首例刷新脚本误读identity字段toolchain（实际certification_toolchain），随后误用emit操作（实际daily-emit，exit2）及C generated.h（实际generated.c）各在本地采集前置拒；修正按原pin真实文件名后仅续缺失C，Rust已完成记录保留，失败unknown-op三原件另存failed-unknown-op-*。新CT reader首次profile字面误写auto.source.scalar.draft，严格读取报stale；按原实际manifest/既有scalar合同改正后14通过。预算source pin只因本轮certreader真实变化刷新1项及预算合同hash，不改16域原帽。上述中间失败不充最终绿，也未作为新producer缺陷/另立功能计划。

剩余：完整Q4所有源/变体的四target daily/cert与旧313required全读；完整registry封闭；SYN/ARCH真实P/M/R/IO、恢复首编辑/坏首编辑/required页、原样本/预算/性能/HMR/lease；938/939的真实交付与ABI机制；全AND反控/最终冻结并集/独立review/Spec/merge/归档/清理。仍无新增用户决定，不将已认领机制交给未立项后继。

stage: work | plan_id:PLAN-941 | plan_revision:r1 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:f0ef7b36fd88113091af53f3563cec4619b99688 | task_ids:T01..05（未关闭） | evidence:14独立CT refs/当前36原认证/56反控/131治理+69Case冻结/实际完整preview预算49,139,156<48MiB | blockers:无新增用户决定；完整原验收继续 | next_responsible_role:work941

stage: goal | plan_id:PLAN-941 | plan_revision:r1 | round:16 | from:work | to:work | status:executing | outcome:needs_fix | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:f0ef7b36fd88113091af53f3563cec4619b99688 | next:work941（当前工具证据已统一；完整原目标保持）

### 2026-11-06 日常发射与认证的真实文件集联立准入（goal第17轮）

上一轮为实际progress：CT双端新reader/当前36cert/完整组合刷新。本轮继续work；施工提交 **233e0df44**，树clean，941仍executing **0/15**。T04/05原daily/cert分轴不再仅以认证代日常记录，未缩原scope/预算/全量source义务。

- 新`ace_m1_daily_evidence.py`为原source-CT14、text20、首例2的**36份既有实际日常命令**补只读准入：原argv提供nonce而非child响应提供期望；source/profile/entry/caps/输出根分别对已独立准入的认证输入；命令/原stdout/stderr摘要、producer与当前worker实际字节、nonce区别于cert、原Job成功并reap、原计量完整逐层核。此有限Windows原capture的Job process count必须为2（owned launcher+worker），独立于worker自报spawned_rustc/cl=0；额外子进程拒绝。它不推定daily程序已运行，执行正确性仍来自独立实际/参考认证。
- 日常manifest与3个代码/header文件**逐字节SHA等于认证前冻结的actual_files完整4件**，manifest/response两清单分别完整、唯一、目标家族/原source FNV/SHA/入口/Rust target/resource-runtime/HIR domain/C函数origin范围核对。只对专用daily目录受限扫描四个声明成员及首个多余项，未声明exe/目录/链接拒，不遍历其他scope或跟随链接。C原scalar /1不强塞不存在的profile域；text与scalar schema互斥。
- CT/text集合reader与首链现在都必须通过daily+cert；返回具名daily命令/响应摘要、nonce、compiler_spawn_zero及daily_judgement，不自动认领AV/native/CT缓存/ARCH。新reader代码哈希进入admission_sources。日常缺件时即使cert PASS也不能接受；原cert FAIL继续FAIL。原16域预算source pins零漂移，不制造刷新或放宽原帽。
- **原raw副本反控48项**：CT meta-fields2与text NUL各两target×12；4合法重定位副本PASS，其余44（spawn bool/1、额外Job子进程、请求cap变更、源/nonce错、重复/缺失文件、字节数bool、实际main变更、未声明exe）全REFUSED。除实际文件变更/额外文件之外，原命令/response摘要按变更重算仍不能通过独立约束。Popen禁用，明确counterfactual_not_execution；结果scope/daily-original-controls/results.json SHA=`586c41416a0ea6dda1f22e40457d20d040b24df8d08440d6b61f01fa1537c871`。控制是日常证据准入，不把48计为新语言运行。
- 新永久5测试包含四个source-profile/target合法分支、26个重hash坏请求/response分支、两件实际字节漂移、未声明exe/目录、缺daily而cert PASS。初轮缺件测试误期待OSError（实际EvidenceScope统一AdmissionError），及新增专用目录扫描暴露fixture复用scalar/text C目录留下旧header，两者按真实合同改测试后绿；没有放松reader允许旧产物。冻结`cargo t infra-ace-m1-arch --out <scope>/daily-admission-frozen-controls` **136/136，exit0，3.8030970s**，results SHA=`f4d42a1a848fef658f204fb140ac5ddac70a42e35af66f2fb10ed99b2a8acabb`。5提交文件工作字节=Git blob核对，记录scope/daily-admission-frozen-preview/committed-input-audit.json。
- **完整原组合重新实际读取**，包括所有新增36 daily，Popen禁用，当前二进制与输入不变因此复用第16轮实际69Case及36cert原始执行，不为Python读取器变化重复编译/运行或跑全仓。单独原Job计量1383.7342ms，peak process**27,246,592** bytes、peak Job28,479,488 bytes；共享预留37,032,920/峰值**49,464,064 <48MiB=50,331,648**。报告159,043 bytes SHA=`c4dca274163b0746c985e63e6748e8cbc6d47e848d09f7bc38f385081599d971`，scope/daily-admission-frozen-preview/preview.stdout，原meter/空stderr同目录。CT14/text20两域daily_admission均PASS；登记仍82PASS/0FAIL/804UNOBSERVED，universe未封闭/expanded claim=false。增加的是这些格的日常证据强度，不假报新增覆盖行数。

后续原义务保持：全部Q4源/变体四target与独立refs/trace/trap/负例；旧313required全读/38gap；SYN/ARCH原P/M/R/IO/restore首编辑、required页、样本/预算/HMR/lease；938/939实际合同；完整AND治理、最终冻结/独立review/Spec/merge/归档/清理。无新增用户决定/权限等待。

stage: work | plan_id:PLAN-941 | plan_revision:r1 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:233e0df44 | task_ids:T04/05（未关闭） | evidence:36原daily+认证联立/48原raw副本反控/136冻结/全组合49,464,064<48MiB | blockers:无新增用户决定；完整原验收继续 | next_responsible_role:work941

stage: goal | plan_id:PLAN-941 | plan_revision:r1 | round:17 | from:work | to:work | status:executing | outcome:needs_fix | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:233e0df44 | next:work941（完整四target/source/机制与原阈值保持）

### 2026-11-06 F-941-04 原text-source四端扩大验收发现条件末尾记录构造歧义（第18轮工作中）

原925冻结`text-char-925--v03_eq_full_bytes.at`（SHA=c88f8407c7235a1042d062575aa52c68a258eca8752e49844620ab3dfd1358c7，期望值1）在当前Rust/C text daily与认证四操作已通过，但正式`case-ace-m1-text-q4-941`十源×AV/native两次真实执行均**18PASS/2FAIL**，仅此源两腿source.syntax拒，native原诊断为`mn.unsupported.frontend: Text/source.syntax: expected expression, found return`，codegen/run未发生。原完整source不改、期望不改；两次分别是新增正式入口首次运行和补native三陷阱具名RT-FAIL原码/cleanup字节后运行，不将补观测当修复。

源`if a == b { return 1 }`在text-record域把条件尾部`b {`按记录构造开始；原`core/src/source/parser.rs`约2954处仅看record surface+标识符后同线左花括号，尚未以条件上下文区分。AV owned_driver与native升级链都调用compile_text_record_source；text单域A2X无此歧义。诊断用隔离兄弟输入仅加条件括号`if (a == b) {`，原owned-request实际返回i32=1、两分配均释放（scope/text-q4-entry-survey/eq-parenthesized.*），只为定位原因，**绝不以此替换冻结源或覆盖原FAIL**。

最小独立修复提案：既有record/text-record条件表达式消除尾标识符/记录构造歧义，保合法带括号/嵌套记录构造；补if/while条件、多行/括号及实际AV/native原冻结源回归，核共享前门消费路径；独立计划/review/职责门/merge后941重放原十源。按§5.4已归档能力新缺陷独立处理，本941不修改parser或改oracle，不以扩大profile的现有拒绝当正常通过。独立修复请求用户授权；在答复前继续只读证据reader等不依赖修复工作。

本次原结果摘要：`{"text-q4-original-entry/results.json": {"sha256": "2f9912077ca7bfdecd3a3ca901af779358dabdc9dcbed06824193b6617d75cf7", "gate_passed": false, "wall_seconds": 13.564703900003224}, "text-q4-source-bound/gate/results.json": {"sha256": "5b7a50e0a773ab3a21d48be1a5bfbffe05b90cd93ebbcfa26463f48cc39c4d62", "gate_passed": false, "wall_seconds": 13.042500700001256}}`。原Case/worker帧与source/hash/Job完整留在统一941scope；native三trap同时发现已有RT-FAIL code=-10/-8，可沿原770库合同正当读取具体TextIndex/InvalidScalar，不能仅从exit70推断。

### 2026-11-06 文字十源实际AV/native接线与失败保真（goal第18轮work）

上一轮36 daily/cert联立为progress；本轮新增正式十文字源AV/native全20项及严格原帧reader，施工提交 **4cd5782d1fde03a5060e35dee4717dc2233bfc83**，树clean，尚未修F04。原T01..05仍未关闭。

- 永久`text-q4.case.ad/.suite.ad`及显式固定组`case-ace-m1-text-q4-941`接原925十源，**源码字节/原oracle不改**；AV用原owned-run-v1，native用原mir-native-scalar-v1。首次20项18PASS/2FAIL；后续补native三个具名trap的原770 RT-FAIL+cleanup精确字节（status -10→TextIndex、-8→InvalidScalar），再运行仍18PASS/2FAIL，结果scope/text-q4-source-bound/gate/results.json SHA=`5b7a50e0a773ab3a21d48be1a5bfbffe05b90cd93ebbcfa26463f48cc39c4d62`，exit1、13.0425007s。后者才为当前Case合同，不以首次native只exit70当完整trap证明。两个FAIL恰为eq_full_bytes合法值1的AV/native前端拒，其他值及三trap实测均符合原oracle。
- 独立 typed /2永久输入`text-q4-contract.json` SHA=`7db3492644cf64d2e378b192519a40e25d8c8edb9ef2aed923dc7c9ef0c6d1e8`，20 check的源/Case/profile/recipe/caps/oracle逐项绑定。以单独合同文件+具名root指针存，避免把整20行复制进常驻root registry；原登记886/旧313集合不变，无动态交集/NA。当前root第七具名合同text_q4，旧511原case映射仍原分母，新增20单列。
- 新`ace_m1_text_q4_evidence.py`读真实Case/artifact-copy/START+TERMINAL/原request/stdout/stderr，重核当前worker、源SHA/FNV、请求hash/nonce、recipe/version/ABI、原owner的store/work路径、canonical owned i32及release ledger；native按原固定counter/span集合与真实link/run=1核，实际PE stdout/stderr逐字节。native RT-FAIL码+exit70+独立三trap精确事件链共同供语义trap权威；仅exit70从不通过。编译期语义diag作为**合法可读取的FAIL**保留，未知schema/协议/坏源/路径仍拒，不吞成工具绿。AV/native真实结果各自对独立文字oracle，不靠同错/父PASS。
- native实际text lib与11个生产依赖（两worker、ac_text_runtime.lib、原library locator/runtime/driver/cache/link/config、owned_driver、AV trap合同）在第二次执行前后逐件SHA不变，原Job实际运行中按已钉原worker的固定locator路径消费；source/runtime漂移读入即STALE。材料scope/text-q4-source-bound/dependencies-before/after.json，本reader不运行该locator或启动cargo。资源/全预算/所有生命周期证明仍UNOBSERVED，此单元不冒资源全域验收。
- **原帧反控22项**：value/trap×AV/native各5（4合法PASS、4同错语义FAIL、12换nonce/未知field/改source并重算requesthash均REFUSED）；再将两条实际前端diag的父级状态强制PASS，结果仍2FAIL。明确counterfactual_not_execution，结果SHA=`5690185423a948aa8d12d718065ac8841b5836f5d426855e8814061a61aad7aa`，scope/text-q4-source-bound/reader-controls.json。新增永久5治理测试覆盖两recipe值/陷阱、双方同错、exit70无具名trap、原事件/ledger坏cleanup与实际diag保FAIL；初轮6子例因测试构造误给native request加steps={}而拒，按真实协议改fixture后绿，未放宽生产reader。
- 冻结4cd578 `cargo t infra-ace-m1-arch --out <scope>/text-q4-frozen-controls` **141/141**、exit0、3.2894021s，results SHA=`a21adbb14ff23c09c0473b962deaa7d21fe2e727dadf97cf4fd2c69a0f031342`。原20Case实际源/producer/工具/依赖与冻结提交相同，复用其红证据，不为提交重复执行。10提交文件工作字节=Git blob核，scope/text-q4-frozen-preview/committed-input-audit.json。
- 新完整只读组合实际原Job：1690.3642ms、peak process26,583,040/Job27,820,032 bytes，共享reservation37,585,665/峰值**49,999,391<50,331,648（48MiB）**，无抬帽/重置峰值。报告169,760 bytes SHA=`86a22936a202e3d14956dc2dc7b82289e2d6a505892b2be32eb3c0f500bfc089`，scope/text-q4-frozen-preview/preview.stdout，原meter/空stderr同目录。登记为**100PASS/2FAIL/784UNOBSERVED**，source_case_reports=false、expanded claim=false；未隐藏新红，也未把141治理绿冒语义20绿。

F04诊断/原兄弟括号探针及后续用户裁定见本节相邻记录。剩余完整原941四target/源变体、旧313/38账、全机制/原规模/预算与938/939等义务保持，不以文字域已有36/40通过当整包完成。

stage: work | plan_id:PLAN-941 | plan_revision:r1→r2授权修订在后 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:4cd5782d1 | task_ids:T01..05（未关闭） | evidence:20真实Case18PASS2FAIL/具名native traps/22反控/141冻结/全组合100PASS2FAIL784UNOBSERVED | blockers:F04实际缺陷，用户已裁定在941新phase修复 | next_responsible_role:new r2→work T16

### 2026-11-06 用户裁定F04在941内新增phase（r2）

用户对F04独立修复问题明确回答：**“直接在941计划添加新的phase修复。”** 本条为新授权，§4/§5.4/§5.7已按该具体例外同步；改plan_revision 1→2、total_steps15→17，仍executing0/17。不另立计划/取号/新树；T16/17在既有941树执行，原全部AC/样本/预算/旧38账保持。新增AC14/V19..21/SD07与具体producer→consumer、whole/fragment/合法/非法Ctor合同，修复后继续原全部验收。

补充定位：生产`compile_text_record_source`经adapter/统一前门消费CST；`source/cst/project.rs`约2152处同样按record profile与同线`ident {`启动Ctor诊断路径，约4062处有Ctor投影；旧parser约2954是兼容oracle的相同历史歧义，**不能仅修旧手写parser宣称生产已修**。具体根因/合法record guard规则由T16沿实际语法结构与投影确认；未在规划轮改功能。没有新的用户待决定项。

stage: new | plan_id:PLAN-941 | plan_revision:r2 | outcome:pass | changed:T16/T17/AC14/V19..21/SD07及F04特定S/N/X授权 | baseline:4cd5782d1 | worktree:D:/autostack/.wt/lang-941/auto-lang | next:work T16→T17→原941

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:18 | from:work→new（用户明确新增phase） | to:work | status:executing0/17 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:4cd5782d1 | next:work T16→T17→原941（F04内修已授权，无待用户决定）

### 2026-11-06 r2 F04生产语法与投影修复实现（goal第19轮，T16完成/T17在跑）

用户r2内修授权已生效，无新计划/编号/树。唯一941树提交 **a8cc58bd368dfcdccecae000a2877086e64718d0**，code clean；**T16完成，current_step1/17**，T17完整消费者并集/当前身份数据刷新尚未完成，F04尚未关闭。

- 根因实际生产是共享auto-source grammar `ctor-expr`在`ident {`左花括号cut提交；`compile_text_record_source→compile_parts_with→parse_program_with→CST`不是旧手写parser。新record/text-record/collections/closures条件用独立condition Pratt/rule族：bare guard的body brace不再成为ctor；真正带fields的裸具名构造、括号内构造/嵌套实参仍保留，普通expr原ctor cut/空ctor历史怪癖不改。while保原Core域，其余profile仍按原规则拒while、条件循环用for。
- 条件规则节点按具名kind映射进入**同一**AST投影帧机，callee/方法调用/后缀分类及结构诊断路径一致，不新增parser/consumer源重写。原诊断叶walker与仅供差分的手写oracle增加同一条件上下文；非法构造残留Ctor frame的旧oracle unreachable改为与既有production诊断一致的source.syntax，新增反例实测不panic。sourcecap身份**ac-source-front/4**、共享grammar revision**3**，canonical换GrammarId；不复用旧cap/grammar身份，现旧worker/cert/preview仅历史，待T17原始数据刷新。
- 新三个永久命名`source_cst_927::conditional_941_*`：四recordal profile×bare变量/比较/一元/调用/括号×空/非空body（40组合）与合法裸ctor/括号ctor/实参ctor（12组合）；record/text-record条件for/free片段、whole方法/MethodFragment、原while排除与三类非法构造源诊断/span及AST/旧oracle差分。新增具体AST不为Ctor断言，不仅两parser相等。现T16已覆盖原phase的多消费者解析机制，真实执行结果仍由T17证明。
- 基线两新测真正执行**0PASS/2FAIL**（修复前），scope/conditional-941-baseline-compiled/results.json SHA=`62ef1157e9e10585dcefebe3a323e2b330787fb0607c1d70ab7cf72a16fe6fc3`、3.7856365s。更早首编译因测试误写AST枚举RecordCtor（实际CtorNamed）失败，不充语言反例；修正测试后才有上述真实red。
- 第一版修复语法把alt和普通steps混同rule，被grammar编译器明确拒（source.infra），按现有步骤profiles门控修正；随后两定向测绿。完整差分发现3/567原语料AST在条件方法call分类漏识别新别名，及caps旧常量断言红；通过统一expression kind规范化修正所有callee/投影路径，按新实际身份更新三处caps断言，原567语料/70edge不删。新增非法ctor测试另击中旧oracle的Ctor-frame panic，按生产既有typed syntax拒绝修复后绿。这些失败全保原scope，未调低oracle/样本。
- 最终`cargo t ace-source-syntax-927 --out <scope>/conditional-941-final-syntax` **13/13 PASS**、5.8016579s，results SHA=`c5bde3e89da0f01a9ef9e0521f6577e18fe84142bc8295ba71a92d018c63b3e9`；内含原完整差分/edge/片段/typed预算探针与新3测。旧first_lex_fact dead_code warning1保持已知baseline，无新增compiler warning。9文件rustfmt/diff-check完成；SD07的source-frontdoor.md修复知识仅准备在worktree，未提前落master Spec。
- 冻结a8cc后已启动一次明确完整phase并集：`cargo t ace-source ace-syntax ace-av ace-native case-ace-m1-text-q4-941 --out <scope>/conditional-941-frozen-union`，含原全部样本/平台/ignore；新grammar变更因此**实际补ace-syntax**，没有按“纯parser”豁免真实下游。原语义source/期望/十源Case未改，AUTO_CASE_WIRE_EVIDENCE=1。工具release准备完成（AV约74s、ac51.43s等），现在原nextest ace-source执行中；尚无最终PASS/FAIL，不能填绿。具体活句柄unified exec **session_id=50141**，Python PID35700，nextest PID46300，最后实际检查均活；恢复时先轮询同句柄/原results，不因长时间无输出重开。

T17后续：收原完整union真实终态/原十源20实际值与具名trap，再以当前worker/GrammarId/caps/runtime/source重新捕获及严格准入首链/CT/text受影响数据；现旧预览/36cert不冒当前有效。完整原941其余T01..15照旧，最终独立review仍待T14，不以phase自测顶review。

第19轮续核：完整`ace-source`已真实终态**383/383 PASS，1原条件排除，not_run=[]**，原nextest Job执行175507.6743ms；未截断深4096/8192压力（实际在跑的deep_unclosed案例约1.15GB private，沿原nextest资源/截止退出，未人为强杀）。当前`ace-syntax`原64MiB streaming压力仍在执行：session50141/Python35700继续活，nextest现PID42064，其子PID33416为`syntax_budget::performance::syntax_budget_64mib_streaming_small_queries`，不是已停或可随意重跑。最终整union尚未终态。

重核当前五个工具av/native/syntax/bridge/a2x的真实SHA均不同于第18轮绑定，原证据不能冒当前；对账scope/conditional-941-identity-drift.json SHA=`1624c1e36f9d5a945c2cfce959426f679a6534f528c93f97457a3d77e88c650e`。旧100PASS/2FAIL/784UNOBSERVED和36cert是修前有效历史；T17要在新工具下重建/准入，当前无新版组合签章。sourcefrontdoor.md SD07仅worktree候选；未动main代码。

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:19 | from:work | to:work | status:executing1/17 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:a8cc58bd368dfcdccecae000a2877086e64718d0 | next:work T17（首先轮询原活session50141；随后刷新实际身份/证据；其余原941完整目标保持）

### 2026-11-06 T17同一冻结并集继续：source+syntax已实际515绿（goal第20轮）

上一轮T16修复/13语法回归/source383终态为progress。本轮接续**原session50141**，没有重开或覆盖结果。原64MiB流式bench活进程PID33416持续CPU/约10..13MiB private，最终按原负载/200查询/原预算正常退出；不是超时或被取消。

- `conditional-941-frozen-union/0/result.json`：ace-source **383PASS、1原条件排除、not_run=[]**，Job执行175507.6743ms，SHA=`229047f85f25b80b5fb1e85111041642d5d9367ce713f62aa217ddbb43223379`。
- `conditional-941-frozen-union/1/result.json`：ace-syntax **132PASS、0排除、not_run=[]**，Job执行**624447.4833ms**，SHA=`7233b4e03f2788b80f135b60d415bd9498ca66c471cf4989090f946c09063149`；原64MiB外存构建/恢复小查询压力保完整，不据运行长而忽略/减载。当前已真实515项执行通过，不能加排除项充PASS。
- 现在同一固定并集处于ace-av阶段，nextest PID36136，已观察原512修订与512编辑/GC/restart sentinel相继实际运行，末次活子PID9008（v07_full_storage_axis_512_edits_gc_restart_sentinel，CPU>121s、private约76MiB），Python35700/session50141仍活。AV尚无终态，不声称其测试绿；native与文字20随后，原union不能重启。无实现写入，code仍a8cc58bd368dfcdccecae000a2877086e64718d0、树clean。
- 为随后实际文字20捕获**执行前**11依赖快照：当时union还在syntax、尚无group4 run记录，scope/conditional-941-text-source-current/dependencies-before.json SHA=`eef3ee9198e7e2a2dfea5cab263598c918a2e9867c8fdd6babf01014a30e38fe`。执行后仍须逐件比较/当前重hash；若中途helper/tool漂移必须具名刷新，不能仅改SHA冒原执行。
- 只读重核预算source pins与ARCH native-module source pins零漂移；938仍executing6/13、939仍drafting0/14，不冒正式D。已在统一scope准备当前36cert与7正式Case组69项、/2 identity-exporter和parent-inventory的刷新脚本/显式命令；**未在现正式压力采样期间并发启动**，等原union真实终态再依次执行。新工具五SHA均漂移的事实保持，修前36cert/旧100PASS2FAIL预览不冒当前。

T16保持完成，T17仍在执行，941 r2 **1/17**，无新增用户决定或权限等待。新phase原冻结eq/值1与十源字节/独立oracle不变；整包其余required/原规模/legacy38/938939/机制证明与最终review/merge/清理原义务保持。

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:20 | from:work | to:work | status:executing1/17 | outcome:needs_fix（T17真实活运行，非新失败） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:a8cc58bd368dfcdccecae000a2877086e64718d0 | next:work T17（轮询原session50141/Python35700，当前AV；全部终态后当前身份刷新）

### 2026-11-06 T17原冻结并集AV完成及原日志落点审计（goal第21轮）

继续原session50141，未重启/减载/改忽略。`conditional-941-frozen-union/2/result.json` ace-av **1019PASS、0原排除、not_run=[]**，Job执行428485.4901ms，SHA=`756b99992787c1178e1448b6fb87e1f4f616e48e9c2b09d0b100db78e58206db`。加source383+syntax132，三组实际1534项绿；native仍在原完整运行，不能提前填绿。

native原924闭包负载、collections故障/producer与resources cold-set churn已实际结束；末次活子PID35924为`source_diagnostics_911::performance::mn_sd911_source_v1_four_state_p90_30`，保原30样本四状态/P90，无并发认证/源码修改。同一父Python35700和session50141仍活，最后文字20组尚待执行。原11依赖before快照保持，终态后再逐件核对。

发现原AV `av_source_incremental.rs::report_file`硬编码回写本树docs/reports/plan827，导致tracked `incremental-raw.json`被本次实际30修订覆盖并新增`incremental-raw-1793935380-pid49312.json`；负控日志与HEAD字节相同。已确认原writer49312不在、三文件单链接/无reparse，逐项bytes/SHA/HEAD归属清单在scope/av-827-report-migration/before-migration.json。实际immutable日志15419B SHA=`f7c9a7b6df7026d3cb34b11c6e8c13e4cb6ad78699ae8b3c14ae5879af513ed7`；pointer15447B SHA=`88a25228e6ff4aa4a16b0bb641065388246dcfe8e9ae0d07edf0726d5b2a00bc`，来源明确为本次门禁，不触其他会话WIP。

按统一材料政策，**等原冻结并集终态后**，精确迁移本次新日志/当前pointer副本到同一941scope、核验数量/字节/hash，再恢复tracked pointer原HEAD字节；原已归档827历史报告不改。随后仅将测试输出路由接既有AUTO_PLAN_ARTIFACTS→系统临时目录约定，保持原raw格式/样本/计时窗/P90/语言功能不变，使用既有具名写失败raw负控核实际路径；该项为941 T11/H材料治理，不扩语言修复边界。当前不在冻结门活跑时修改任何源码。raw摘要缺完整114未改身份/全计数等，不能凭此summary代T06完整ARCH证明。

T16完成/T17未关闭，r2 executing1/17；原36cert/组合仅历史身份，后续刷新已准备但未启动。原全部941目标、门槛与T01..15继续，无待用户决定。

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:21 | from:work | to:work | status:executing1/17 | outcome:needs_fix（T17原并集尚活，无新语义失败） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:a8cc58bd368dfcdccecae000a2877086e64718d0 | next:work T17（原session50141 native→文字20；终态后精确迁移原日志/输出路由及当前身份认证刷新）

### 2026-11-06 T17原冻结全并集/当前认证准入完成（goal第22轮）

前轮AV终态与日志归属审计为progress，本轮始终接原session50141，最终exit0且原父PID35700实际退出；无重启/降载/改ignore。冻结实现a8cc58bd368dfcdccecae000a2877086e64718d0的明确并集source383+syntax132+AV1019+native225+文字20＝**1779实际PASS，1原条件排除**，未执行不充绿。总命令2958.4283208s（含准备/发现/原完整负载/清理），各段原meter独立保留，不能与组合器1477ms混同。

- native `conditional-941-frozen-union/3/result.json` **225/225、not_run=[]、0排除**，原Job执行1510523.2084ms、7199进程，SHA=`d05dabf0be17678d95de640cbf73e196050368c234cfd2c1831c58400010cbf9`；保原闭包/collections/资源收敛/诊断四状态各30/GC与热更新原样本，全部原P90断言由正式原入口执行。
- 原十源文字20实际全绿，`4/result.json` SHA=`c32f5b8547911fb8e5bf61fd1560a740ac43992c24f0dd66976b0778dd4b7821`，原Job11130.8839ms，源码/Case/独立oracle未改。原eq_full_bytes源SHA仍c88f8407c7235a1042d062575aa52c68a258eca8752e49844620ab3dfd1358c7，两原FAIL现在分别实际AV/native返回值1；其余值与TextIndex/InvalidScalar三陷阱保持。整并集results SHA=`e60e2aeb14302b4f4701dd216e305de3e61325b3b57b3d07f1baaa3c53c3e29c`。
- 执行前后11实际依赖SHA逐件不变；当前text binding指原group4真实artifact-copy与原before/after快照，after SHA=`45bcc22a8cf31c211320782fda5027bbba5beaaa4afeb10c2f60b9e44bb41fbb`。当前/2正式identity-exporter原producerSHA与原Case合同对账通过，不只换旧binding数字。

原日志落点已按前轮归属清单精确修正：原writer/父进程退出后，复制当次pointer15447B、负控1181B，移动唯一新immutable15419B，总3件**32047B**至同一scope/av-827-report-migration；三件SHA前后一致，清单after-migration.json SHA=`f2522fa6a43da91c0cff5a8583328236c63f2055c6d4931654cae71e7f37d450`。tracked pointer用已核HEAD原字节恢复（SHA826c614810e9afc2e61d07250447489b6f2d1f465562b49384f41e0bf9a094aa），原已归档报告不改；无链接/共享hardlink、源/目标均严格核定归属范围，无盲删。测试report_file采用现有AUTO_PLAN_ARTIFACTS→temp约定、子目录ace-av-plan827；仅输出位置变化，原raw格式/样本/计时窗/P90守恒。既有具名负控`cargo t ace-av --test-name av_source_incremental_failing_verdict_still_writes_raw --out <scope>/av-827-report-migration/routing-check`通过，实际scope新raw1181B SHA48649c6debcb840fe39c95854f03a3043d6b90d1eee19e15d6df31c4a0127d5c，包含两样本与预期FAIL/2100ms；原仓baseline仍HEAD字节。gate SHA=`036a29de7a5ad29a155148886a6f030dbcc1ee266900ce541db95ba6c787af4e`、9.8130831s。rustfmt/diff-check通过，提交**27457e6b3**，树clean。此输出路由不改生产二进制/解析/原消费者，已逐件核11依赖不变，复用上述冻结全并集，不为logger目的重跑1019/原压力。

新身份后已顺序重建全部受影响数据：

- current-certifications包含**36实际认证+36 daily emit+36工具身份请求**，每认证actual与独立reference各编译/运行，共72编译/72运行；保原CT14、文字20、首scalar2，独立reference与源不改。daily的完整actual_files在cert前冻结，不以cert回填伪造daily；本轮真实原命令/nonce/Job/stdout/stderr均留统一scope/conditional-941-current-certifications。
- `cargo t case-ace-m1-q4-941 case-ace-unified-source-929 case-ace-ca-d-scalar-937 case-ace-m1-legacy-941 case-ac-av case-ace-m1-typed-native-891 case-ace-source-cte-936 --out <scope>/conditional-941-consumer-refresh`原7组**69/69 PASS**、42.7570281s，results SHA=`8e1609287f4a43c5dc58faefaba89172173178c7145645406fe9119c2a90d6b9`。新7个/2 inventory与parent-inventory实际捕获，严格reader重新核对原字段/当前工具/ABI/source/hash/完整产物/独立oracle/真实终态与原trap字节。
- 整组合在禁止subprocess.Popen的独立原受管Job重读通过：**102PASS/0FAIL/784UNOBSERVED**（仍登记886且full_registry_closed=false）；first_slice_evidence/source_case_reports=true，原legacy313/38认证缺口、完整分母与ARCH仍阻断，can_claim_expanded_m1=false、acceptance_system_delivered=false。当前preview169200B SHA=`2e7934d2643068a1657c217469311fce760b553a2146b12c30eaf5fe12ad447f`，1477.7109ms，peak process27406336/Job28672000B，共享reservation37584805/峰值**49998531<50331648B（48MiB）**，原单件4MiB/private128MiB帽未改，未重置峰值。scope/conditional-941-current-preview保原meter/空stderr。

**T17完成，current_step2/17**；F04实现与真实原例消费者闭环完成，最终独立review仍T14未执行，不能以本phase自测冒review/merge，也不单独归档941。SD07候选仍在同树source-frontdoor.md。原T01..15完整任务继续；新五worker身份的36cert/当前数据已刷新，旧pre-repair100/2只保历史。当前无新增用户决定，后续继续完整登记、剩余Q4/原313/机制/ARCH证据适配。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress（T16/T17完成，原完整任务未闭） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:27457e6b3 | task_ids:T17/T11材料落点 | evidence:原1779PASS/1排除、69消费者、36daily/cert严格准入、当前102PASS0FAIL784UNOBSERVED | blockers:无待用户决定，原任务继续 | next_responsible_role:work T01..13

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:22 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原完整任务未完成，非phase失败） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:27457e6b3 | next:work 原T01..13完整验收（F04 phase验证完成；最终独立review待T14）

### 2026-11-06 原61标量源实际AV/native扩大准入（goal第23轮）

上一轮T16/17原修复phase完成为progress。本轮回原T03/04/05/10/11，唯一941树提交 **ea4243fd332b252dbd0eef3a12cf35163570e7cd**，树clean，未修改任何生产编译器/语言/资源功能；原总任务仍executing2/17。

- 将已冻结scalar-obligations.tsv的**61源（40core/21scalar，53值/8具名trap）**全部接到正式AV/native两recipe，永久scalar-q4.case.ad/.suite.ad与固定组`case-ace-m1-scalar-q4-941`共**122检查**。源字节/原oracle/独立marks/原Rust debug61+release8/C61义务一律未改。两trace源保hir.test.trace.v1原cap，源序mark_b,mark_a→AV trace[b,a]/native stderr `ba`，短路源须无mark。
- 首次入口因6个历史case名含大写B（正式Case ID仅小写）在Case装载时拒，未进入语言执行，不充语言反例；仅把新Case ID转小写，原源文件名/字节不动。第二次真正执行**114父PASS/8父FAIL**、原Job47881.497ms、整命令52.5619123s，scope/scalar-q4-current-source/results.json SHA=`31663f553f595c34072e918ec78b6443ca7fb87a01f311538ac49f25d5d5ab14`。第一次装载红原scope/scalar-q4-original-entry保留，未覆盖。
- 8真实拒绝逐项保真：`core-f11-add-chain-100`、`core-g02-parens-200`的**AV**原admission前端预算报av.resource.frontend-nesting；`core-p04-while`、`core-p05-while-zero`、`core-p06-continue`的**AV/native两腿**均source.unsupported，native原标量→record→text-record升级链终止于Text/source.unsupported while。当前AV scalar默认入口明确走934 scalar fact前门，native依870/911原升级链；源原core域while不因consumer现默认Scalar域改成for。本轮仅具名记录**GAP-941-SCALAR-01（预算/profile覆盖差异）**，不擅自抬帽/扩profile/修语言，也不将其删行或NA。原§5.4 owner责任及T12真实red交付边界保持；这组发现尚不是经过归因认定的新已归档实现缺陷，不借此给941扩大功能修复授权。
- 另8 native trap父PASS只有通用exit70+空stdout/stderr，缺具名Overflow/DivByZero/DivOverflow/RemOverflow/RemByZero语义证据，strict reader明确**UNOBSERVED**，不把通用70当trap证明。`bd-value70`原正常返回70是独立正控：其value70必须PASS，其他值被替成70必须FAIL。AV按原owner trap taxonomy，RemByZero与DivByZero原本共享av.trap.div-by-zero，仅在固定source/oracle下显式映射。

新`ace_m1_scalar_q4_evidence.py`严格接原Case/artifact-copy/START+TERMINAL与请求/响应原字节：当前sourceSHA/FNV/nonce/重算request_hash、独立源/值/trap/trace、版本/ABI/真实link/run、store/work owner、原父终态全部核对。原fixed group在执行前准备的AV/native与trace-runtime.lib路径/SHA与当前独立文件核对一致；trace库为runtime-dev/ace_trace_runtime.lib SHA=e6a07ae71bb0d6e59aa4362b48ebf581c8dfde6f03bb538e945720ceb0a129b5。122正式/2 inventory实际捕获；永久独立portable contract277819B SHA=`b8fc0bf59aa1eb42996af15676edcb162ad8994510b52fe7b7f491da8feaae0b`，并再次逐行对原固定TSV/原表source hash/61分母/原caps/手定oracle核验，非用子报告自带oracle。最终这组严格判定为**106PASS/8FAIL/8UNOBSERVED**，不把父114绿冒语义114绿；8真实diag即使父改PASS仍FAIL。

组合器原48MiB帽仍不变。为接完整122且避免同时保留已消费的原始输入，将原首例inventories/copies/认证/accounting的原读取块提取到有界helper；helper返回后原始对象实际失去引用，再由既有_compact_admission释放它们的预留，保2MiB摘要预留，**同一共享LiveBudget与全程peak从未清零**。标量逐exchange同理保32KiB摘要、外层完整标量摘要保3MiB；单件4MiB/解码与索引先预留/64KiB流SHA/private128MiB不改。先用旧当前bindings重放，原完整registered rows/gates/domain observations逐项与上一轮完全相同；仅原始对象生命周期缩短，旧不带标量预览reservation22559012/peak34957144B。接新域后仍在原帽内，不另开域预算来伪装聚合。

- **20原始帧反控**：5原实际源×两端（值、Overflow、源序、正常value70、真实while拒绝）及换值/反转marks/换nonce/加未知field/同时换source并重算requesthash+FNV；10原观察按PASS/FAIL/UNOBSERVED正确判，另10反控分别FAIL/REFUSED。明确counterfactual_not_execution，scope/scalar-q4-current-source/raw-reader-controls.json SHA=`4928dd4b7d0fcb1a41bc7b010d1f3cad0b75640ad51c23607d0dfd378a291a4b`。这些反控不算新增语言执行覆盖。
- 新5永久回归保same-wrong两端、value70/未具名trap/错具名trap、mark顺序与短路、原diag强制父PASS仍FAIL、nonce/source/未知字段/真实link计数拒绝。冻结ea4243 `cargo t infra-ace-m1-arch --out <scope>/scalar-q4-frozen-controls` **146/146**、3.4104766s、exit0，results SHA=`38d520c38f96ba61a6ce71cefef62773c0227bfbc8a3d4fe7f469ab192e6c656`。未为Python reader重新跑1779原native/AV重负载；生产工具/源/Case字节身份不变，实际122正式结果由新strict reader逐项重核后复用。py_compile/diff-check通过。
- 冻结整组合在禁止subprocess.Popen的独立原受管Job重读：**208PASS/8FAIL/670UNOBSERVED**，登记886/全部required集合不变（expanded contracts增加scalar_q4指针，未改原分母）。父source_case_reports=false与first_slice_evidence=false保留新真实红，full_registry_closed=false、acceptance_system_delivered=false、can_claim_expanded_m1=false。当前preview236951B SHA=`dcf7ccf6de06ae3bb4892b8626235ca6ee4f7293492baedd919f4f87fa0b3ca5`，2264.1561ms，peak process28909568/Job30154752B，共享reservation25717242/峰值**47244667<50331648B**。scope/scalar-q4-frozen-preview保原meter/空stderr；新模块与现配置/全部当前依赖被真实读取，不只测小对象。

原T01..15仍未完，原938/939未冒D、原旧38cert缺口不变；这轮补齐标量AV/native实证，并未声称61源Rust/C独立reference/全部debug-release认证、剩余复合/source SQ/实例/机制/ARCH完成。下一步继续原完整四目标与独立reference/认证及机制适配；上述预算/profile red和native具名trap缺证据保持可见，不在941业务实现补丁内绕过。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:ea4243fd332b252dbd0eef3a12cf35163570e7cd | task_ids:T03/04/05/10/11（未关闭） | evidence:122真实父114PASS8FAIL→严格106PASS8FAIL8UNOBSERVED；20反控；146冻结；全组合208PASS8FAIL670UNOBSERVED | blockers:具名budget/profile覆盖差异与native trap观测缺口；无新增用户决定，继续独立工作 | next_responsible_role:work 原完整Q4/认证/机制/ARCH

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:23 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原任务未完/真实覆盖红保留） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:ea4243fd332b252dbd0eef3a12cf35163570e7cd | next:work 原完整目标，先补剩余四端/独立reference与机制证据

### 2026-11-06 原61标量源独立Rust/C reference与默认配置认证（goal第24轮）

上一轮106标量AV/native严格观察为progress。本轮继续T03/04/05/10/11，唯一941树提交 **aa125c8a1667fed8692934ee6ab8d8045c5fa547**，132文件（含122永久reference）已提交且树clean；未修改编译器/worker/生产语义，原总任务仍executing2/17。

- 新61源×Rust/C **122独立手平移参考程序**，不读取/复制actual发射代码，不以oracle值生成函数体，不另造Auto编译器。各源独立保运算、调用、循环、递归、22项比较表、所有条件/短路分支与命名实参源序；原core-f11虽名100实际为300个1，独立checked累加300；语法深括号的reference按相同值规范化，原被编译源深度/字节不改。Rust用标准checked_*，C用int64范围检查及独立div/rem零/最小值溢出规则；输出仍原value/trap协议，C stdout/stderr明确binary，避免CRT换行混淆。
- 永久scalar-reference-contracts.tsv 61行，SHA=`4d4c5a3904f0a5955855f6127aa388ba6ae1a14cc4df8858b00b05aa6d53a094`；逐行再对原scalar-obligations与原provider表hash验证源、profile、caps、值/5类trap、marks与Rust debug61/release8/C61配置，122参考文件各自SHA固定。新strict `ace_m1_scalar_cert_evidence.py` +原cert/daily reader有界扩展支持**原Core/Scalar**身份、checked trap与trace，不以C的Scalar身份代Core，原text/CT/首例读法守恒。
- 实际原a2x-worker顺序执行**122默认认证+122对应daily发射+122工具身份请求**，actual/reference各实际编译与运行，故有效认证共**244编译+244运行**。Rust原--edition2021默认配置，C保原-nologo/-std:c11/-TC/-W4/-WX/-O2/-MT；每case daily actual_files在cert前冻结，完整main/runtime/generated/manifest均真实重hash，原Job/meter/argv/stdout/stderr与失败尝试只留同一scope/scalar-q4-certifications。当前strict 122默认认证/daily全部通过，原**40Core/21Scalar**及三while源Rust/C原Oracle均通过；两AV深度预算/六AV/native while拒仍是上一轮真实覆盖红，不因本轮另两腿绿消失。

首次自写C reference中10条被原-W4/-WX拒：8条trap的常量传播导致C4702不可达警告，另两条短路常量表达式C4127。明确是**新reference编译失败/INFRA**，不冒Auto trap或原语言缺陷。保首次112认证成功+10失败原输出；只修这10个C参考，用volatile输入读取保持checked操作动态、显式动态bool保持短路，不改flags/oracle/source/实际target。新尝试独立目录，仅重跑10受影响C目标（原112已通过且字节/工具不变则复用），10全通过，未覆盖失败材料。最后122被选择的reference compile stderr审计**零未处理warning**。旧失败10个仍具名保scope/attempt-failures.json及各attempt；它们不充244有效编译/运行数字。

配置义务不偷换：原worker标量cert接口只会使用默认Rust flags，不能靠传未实现mode或改环境wrapper冒release。本reader把**8条原Rust release要求继续UNOBSERVED**：122默认cert皆实过，但新Rust/C目标登记仅**114PASS/0FAIL/8UNOBSERVED**，certification_complete=false。release8单列需下一步沿原真实producer/完整产物与独立reference取得-O编译执行证据；本轮未补或减样本，亦未声称原Rust69/C61配置全完成。

第23轮文字更正：该轮“两个trace源”只描述末尾bd两个示例，原61源trace=true实际为**4源**：sc-short-circuit-contexts、sc-short-circuit-trace、bd-named-source-order、bd-or-skips-rhs，对应独立marks数13/5/2/0。上一轮实施Case与strict reader本来就逐原TSV为四源带cap并验证全marks，没有漏执行；本轮Rust/C四源同样与手平移reference及独立marks逐字节通过。该表述更正不变旧源/分母/配置。

- **18原认证帧反控**：Core值、Overflow、命名实参副作用三个源×两target，逐一原合法PASS、双方同错值/错trap/反marks→FAIL、重hash未知response字段→REFUSED；修改后同步raw显示/命令/meter与摘要哈希，保原oracle/源码/完整产物，明确counterfactual_not_execution。scope/scalar-cert-raw-controls/results.json SHA=`b9558deca5091d896f916ccb4d48bc14c872aeebf68c283e694bb71fbcc03ee2`，6原正控+6同错FAIL+6REFUSED，不充新语言覆盖。
- 新2永久配置反控证明默认绿不核销release8、default FAIL不被pending改绿、C原配置不被误算Rust release，且拒未知target/计数/布尔verdict；既有daily正控新增Core双target。冻结aa125c8 `cargo t infra-ace-m1-arch --out <scope>/scalar-cert-frozen-controls` **148/148**、exit0、3.4805118s，results SHA=`a3c2395bc759f1d3cca3bc3b53107ff7073ba3986252cf34692b6ce9674e0763`。py_compile/diff-check通过；不因Python/fixture提交重复1779原压力，实际122当前源/工具/配置/完整文件及原69消费者/文字20旧有效证据已由新reader重新逐项准入。
- 新reader先按原严格边界拒了过期budget source pin（cert reader扩展导致SHA变），在确认16条政策/全部数字与原HEAD逐项相同后，仅刷新该**唯一**源pin；原budget合同内容hash更新为`8f3ab799eb1bbbb3e7bde62f8849af3cd438577e3e9fb5a9e1ac2539e3d939f0`，其余25pin及阈值未改。没有用忽略STALE或放大预算让报告过关。
- 冻结全组合原受管Job/Popen禁用重读：**322PASS/8FAIL/556UNOBSERVED**，原登记886及required分母不变，新增scalar_reference第9合同指针；system_delivered/expanded claim/full_registry_closed仍false。输出345303B SHA=`6912823b3d4c276d7239010d318173045bce2de305627fa9ce86ce1cd5c2e0d6`，scope/scalar-cert-frozen-preview，5144.6472ms、peak process28717056/Job29970432B，单共享LiveBudget reservation28876098/峰值**47245787<50331648B**；原4MiB单件/private128MiB/不重置峰值/先预留后解码守恒。

这轮158（原36+新122）默认认证与daily有当前完整严格证据，不能替原release8/剩余复合及SQ/实例/ARCH/旧313/38/938939。下一入口先补8原Rust release的真实producer/独立reference观察，再继续其余原范围。无活运行遗留、无新增用户决定；原T01..15不提前勾选。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:aa125c8a1667fed8692934ee6ab8d8045c5fa547 | task_ids:T03/04/05/10/11（未关闭） | evidence:122独立reference/122实际默认cert+daily严格通过，8Rust release仍UNOBSERVED；18反控；148冻结；全组合322PASS8FAIL556UNOBSERVED | blockers:原release8/其他完整任务未完；无新增用户决定 | next_responsible_role:work 原配置/完整Q4/机制/ARCH

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:24 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原完整任务未完，默认认证不冒release） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:aa125c8a1667fed8692934ee6ab8d8045c5fa547 | next:work 原8Rust release真实观察及其余四端/机制/ARCH

### 2026-11-06 原8 Rust release配置实证与严格联立完成（goal第25轮）

前轮122默认cert+独立reference完成为progress。本轮补原T05遗漏配置，唯一941树提交 **7de59cacec4f375db461f8e4756c81c3ce810b8f**，6文件，树clean；仍executing2/17，不以本切片冒整包。

- 新正式producer `ace-a2x-worker::conformance::scalar_release_941::original_eight_release_variants`直接接**原cases().release_too的完整8源**，不硬造小子集、改ignore或把debug当release。原精确flags **--edition 2021 -O -C debug-assertions=off**；actual与独立Rust reference各8次真实编译/运行，合**16 compile+16 run**。测试始终可从永久fixture自行重建，不依赖将来会清理的941材料；本轮AUTO_PLAN_ARTIFACTS显式指统一scope，normal cargo target仍专属worktree。
- 实际仍由既有a2x-worker现场daily生成完整main/runtime/generated/manifest，再使用既有core::link受管Job executor真实rustc和PE执行；未改worker协议/业务实现、未新增编译器/runner/meter。原默认认证worker和此处正式debug worker身份分别记录；**8份完整发射文件集全部逐字节等于当前默认cert/daily被冻结文件集**，不剪main/依赖/分支，不拿不同代码的-O结果顶配置。8源原值/具名Overflow与DivByZero分别对原冻结oracle；两侧共同错仍失败。
- producer按每phase先保存raw argv/cwd/deadline/wall/exit/stdout_hex/stderr_hex再断言，失败/空产物/部分执行不写完整8集。原compile120s/run30s/whole360s、1MiB双流原帽不变；每case整体截止向各phase传剩余时间。tool -vV原字节单独记录并与原默认cert当前toolchain匹配；源/profile/caps、独立reference、actual完整文件与manifest身份均单列。

开发失败与身份漂移保真：最初构建有u128→JSON Value编译错误，改为已受360s约束的u64；格式化命令连带改6邻文件，已逐件恢复HEAD原字节，未混进提交（原patch保scope/release-formatter-collateral.patch），后续仅skip_children定向格式化。首次8实际执行PASS，但固定组准备重建debug worker，手工build前快照278c009b...与正式准备后3aaadc09...不同，原scalar-release-current只保历史/STALE，不回填before。后续稳定运行补核source/worker；再明确加入原测试exe、executor源与nextest配置，**全8 pin执行前/后**快照的最终正式8重放才供本轮准入；中间开发样本不冒最终配置覆盖，不为此重跑192整包或1779原压力。

最终入口：`cargo t ace-a2x --test-name scalar_release_941::original_eight_release_variants --out <scope>/scalar-release-frozen/gate`，父实际**1/1正式测试**，内部完整8，exit0、整命令7.0859622s，results SHA=`7c559bc167ce26f7eb5648a83f0c07d93236c4878e459b4b9596fc4bdbde3b01`。两个旧只读inventory exporter的原ignore未动；原完整ace-a2x职责并集仍待T12/最终冻结门，不把此命名验证充整个组通过。

新`ace_m1_scalar_release_evidence.py`只读显式binding：原父nextest终态/实际选择/当前discovery可执行文件、producer/table/reference合同SHA、8个执行前后dependency（含实际测试exe与debug worker）、原source/ref路径与字节、所有4完整文件、原发射请求/nonce/返回每field与source FNV、精确编译argv/原flags/工作目录/预算、四个真实终态/独立value或具名trap全部严核。支持命名入口及后续完整同组的原producer选择，不以父PASS或echoed verdict作oracle。consumer不启动任何工具；默认cert失败或release失败均不能被另一腿绿掩盖，缺release仍维持UNOBSERVED。

- **48原帧反控**：每8源原合法PASS、两侧同错值/错trap→FAIL、两侧都删-O/都改debug-assertions=on/加未知field/换sourceSHA→REFUSED，**8PASS正控+8FAIL+32REFUSED**。明确counterfactual_not_execution且限per-case原帧；完整parent/index/deps另经真实binding准入，不把反控算语言执行。scope/scalar-release-frozen/raw-reader-controls.json SHA=`95503e2764a50ec46be7335264da647fa18fcafa19c42b320c39724f39acf7b7`。
- 新2永久回归与原148组合**150/150**：完整原release argv/flags/双流hex/截止/计数类型/未知field拒绝，default/release两侧失败保红。`cargo t infra-ace-m1-arch --out <scope>/scalar-release-final-controls` exit0、5.7645774s，results SHA=`b804f93c61e87d5389b12e34e56029c4402b73bc4ab39261ffa130581e2c20bd`。此门在最终相同代码字节执行，提交后6文件=Git blob逐件核对（scope/scalar-release-frozen-preview/committed-input-audit.json），不为单纯commit重复150。定向rustfmt --check(skip_children)、py_compile、diff-check通过。仅旧first_lex_fact dead_code warning基线1保持，无新增未处理warning。
- 联立后本域**61源×Rust/C默认122 + 原额外Rust release8**配置证据齐，scalar certification_complete=true；原158默认cert+daily仍有效，不把8附加模式错误计成8份新默认证书。全组合登记从322→**330PASS/8FAIL/548UNOBSERVED**，886分母/原required/legacy38不改；原AV深度预算/while六拒绝和native8通用70缺具名trap仍未关闭，source_case_reports/system_delivered/expanded claim/full_registry_closed仍false。
- 冻结7de59c整组合原受管Job/Popen禁用重读：350061B SHA=`74ac4e8320c40b16f1303a7864d2c570294a313fb785b971ddc7c4d54aa4660a`，scope/scalar-release-frozen-preview，7731.5649ms、peak process28594176/Job29847552B，统一LiveBudget reservation28906071/峰值**47258571<50331648B**。原单件4MiB、private128MiB、解码先预留与64KiB流hash仍保，不重置peak/不另开域预算；所有当前producer与原始材料hash被真实读回。

原T01..15保持，未修改F04之外语言语义；剩余复合RT/collections与SQ/实例四端/机制、旧313/38、938939与完整ARCH/最终独立review/merge继续。下一入口继续剩余复合域同源与独立oracle/真实配置，不能以已补标量配置代替它们；当前无活进程/权限/用户决定等待。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:7de59cacec4f375db461f8e4756c81c3ce810b8f | task_ids:T04/05/10/11（未关闭） | evidence:8原release实际双腿16compile16run/8deps双快照；48反控；150验证；全组合330PASS8FAIL548UNOBSERVED | blockers:原完整任务仍未完；无新增用户决定 | next_responsible_role:work 原剩余复合域Q4/配置/机制/ARCH

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:25 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原完整任务未完，标量release缺口已补） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:7de59cacec4f375db461f8e4756c81c3ce810b8f | next:work 剩余复合RT/collections/SQ/实例四端与机制/ARCH

### 2026-11-06 原record/text 16源四端与独立参考默认认证（goal第26轮）

唯一941树提交 **74dc77d8e989a35b7953c072ffe0d2492c9ac7c3**，代码树clean。继续原T03/04/05/10/11，未改生产编译器/worker/语言语义，原总任务仍executing2/17。

- 原record-text-cert-925十一源与record-text-base五源共16：7record/6text/3text-record，13值/3具名trap；源码、oracle、原caps不改。正式record-text-q4 Case/suite执行AV/native **32/32 PASS**，25.005807s，scope/record-text-q4-source/gate/results.json SHA=`f8d4a434ea2caac251c5cceb8bc56fbc2a0da89cee0b253a0e7b9fa3b5634d50`。record明确用data-run，text/text-record用owned-run；native三个trap要求原具名事件及清理字节，不凭exit70。11依赖执行前后SHA稳定。原R07等在expanded外，suite fixture_root指共同cases根，保持实际原路径，不复制或改源。
- 永久portable合同75150B SHA=`52bc1915dfed1a030cf6be30fa7bec430fd1bb6edf57d5e50bc1a75f544477e3`；新16行reference合同SHA=`7869cb3729409f4dce41bb5018e51bd0fab694d4a90d18ac74866dd923b78f8e`。16独立Rust参考+5新C参考保结构、复制/移动/借用/多次修改、命名空间、完整UTF8/NUL和边界操作，另11C严格复用原composite参考，不读actual发射源码。新T05参考在执行前把两字面NUL改为转义，不改Auto原源。新参考实际编译stderr审计零warning。
- 原worker真实 **32默认cert+32 daily+32身份请求**，actual/reference各编译运行，共**64 compile/64 run**，全部通过，scope/record-text-certifications。daily完整四文件在cert前冻结；原profile、HIR域、caps、工具链/原flags、完整文件hash与具名trap逐项校验。当前累计**190默认cert/daily**（原158+32）；另8标量release是额外模式，不充默认认证。原902要求R07/R08/T03/T05 Rust额外`-O`仍**4 UNOBSERVED**，本组Rust/C目标28PASS4UNOBSERVED，certification_complete=false，不以默认绿消除原模式。
- 新record_text reader核正式inventory/Case-copy/START+TERMINAL、nonce/FNV/requesthash/source、版本/域/能力、原工具和owner、实际link/run与独立oracle。data域独立校验canonical i32，未伪造owned ledger；老owned/text/CT/core/scalar读法保持。cert/daily有限增加record/text-record及原runtime text能力，拒未知profile。原budget合同唯一变化是cert reader源pin；确认16条政策和全部数值不变，其余25pin未动，新合同SHA=`1f0bfe4c1dee01c58c07cad5d6a1b0ea799e8b76ec543848d549dc956b5bd77b`。
- **60源原帧反控**：10合法PASS、10双方同错FAIL、40nonce/未知field/caps/源码连重hash REFUSED；scope/record-text-q4-source/raw-reader-controls.json SHA=`5e53750a09bac4abe5853b422d99203d7908170484f1a96b1b1f2fc55ffc96d7`。**30认证原帧反控**：10合法PASS、10同错FAIL、10重hash未知field REFUSED；scope/record-text-cert-raw-controls/results.json SHA=`637a36fbfd1df9e56049ac297c49a6269a9b22ca6fb43ee3198d56dfeb1b0aa7`。全部counterfactual_not_execution，不充新增语言执行。
- 新3永久回归联原150，`cargo t infra-ace-m1-arch --out <scope>/record-text-final-controls` **153/153 PASS**、exit0、3.7981875s，results SHA=`aaa039cd1411e3d94606835db59b408a0998422e5f73e7f53b4089b6d4a9a7d1`。py_compile/diff-check通过。提交后逐件验证变更文件等于Git blob，scope/record-text-frozen-preview/committed-input-audit.json；不因commit重复原153或1779压力，未改生产工具的有效原证据由新reader逐项重读。
- 冻结74dc77d整体在原受管Job且Popen禁用下重读：**390PASS/8FAIL/488UNOBSERVED**，886原分母不变；新增两个RT合同指针共11，full_registry_closed/system_delivered/expanded claim均false。scope/record-text-frozen-preview/preview.stdout 394752B SHA=`c9db3b59f3b20aec772ff7c921f6a94b9a8242a5392f8da29451f3b8eafb8b78`，5478.3814ms，peak process28835840/Job30072832B。单共享LiveBudget reservation31035016/全程peak**47267155<50331648B**，保原48MiB/单件4MiB/private128MiB/先预留解码/64KiB流hash；逐exchange32KiB、域摘要1MiB仅在原对象实际失去引用后释放，不重置peak。

原标量8FAIL/8native trap缺证据保留；RT额外Rust-O四项、collections/SQ/实例、旧313/38、机制/ARCH/938939及最终独立review/merge仍待，不提前勾T01..15。当前无活运行或新增用户决定。下一入口先补四项原RT优化编译，再继续原范围。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:74dc77d8e989a35b7953c072ffe0d2492c9ac7c3 | task_ids:T03/04/05/10/11（未关闭） | evidence:32原AV/native、32默认cert/daily、90反控、153回归、整体390PASS8FAIL488UNOBSERVED | blockers:原完整任务未完，无新增用户决定 | next_responsible_role:work 原RT四release/剩余Q4/机制/ARCH

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:26 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原完整任务未完，RT默认认证不冒release） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:74dc77d8e989a35b7953c072ffe0d2492c9ac7c3 | next:work 原RT四release及其余原范围

### 2026-11-06 原record/text四项Rust优化配置补齐（goal第27轮）

前轮32默认cert+四端实证为progress。本轮继续原T04/05/10/11，唯一941树提交 **351dae1049e6bf554006a971e73b8b3d30ba71ce**，7文件，树clean；仍executing2/17，生产worker/编译器/语言语义未变。

- 新正式`ace-a2x-worker::conformance::record_text_release_941::original_four_release_variants`逐原902 **R07/R08/T03/T05**，逐字节核对925现源与902原源；四条保record/record/text-record/text、值42/42/23/42及原caps。原902精确优化参数 **--edition 2021 -O**，不混标量`-C debug-assertions=off`。actual与原独立Rust参考各四次真实编译/运行，新增**8compile+8run**；完整发射main/runtime/generated/manifest均与既有默认证书文件集逐字节相同。不是新默认cert、不剪产物或改源。
- 直接复用标量observer现有contained capture（仅六helper改pub(super)，原标量逻辑不改），无新runner/meter。两observer均可从永久fixture自行重建；有AUTO_PLAN_ARTIFACTS则所有材料留scope，否则用系统临时目录并自行关闭清理。各phase先保raw argv/cwd/deadline/wall/exit/双流hex再断言；原compile120s/run30s/whole360s、1MiB双流帽保持。
- 共用helper及conformance测试exe身份已变，因此旧8标量release材料保历史，不回填快照。最终同一次`cargo t ace-a2x --test-name release_941::original_ --out <scope>/record-text-release-frozen/gate`实际选择**2/2正式测试，内部4RT+8scalar**，exit0、11.8845302s，results SHA=`2ea6939b916304b88e7af24199ac91918d3f258d3c316b544c212f0f94268f62`。RT9与scalar8依赖（含实际exe、worker、producer/原表/参考合同、executor和nextest配置）执行前后SHA稳定；scope/record-text-release-frozen与scalar-release-rt-refresh。此次共**24compile+24run**（其中16+16为必要原标量刷新），不把命名2测试冒整个ace-a2x通过。
- 只读release reader有限区分scalar与record-text两域，原schema/producer/table/source/flags/recipe/profile/caps、原父终态与discovery测试exe、前后全依赖、当前-vV、完整文件集/manifest、原nonce/FNV/emit与四phase实际终态、独立oracle均校验；原标量逻辑及flags守恒。RT默认与release联立，任何一腿FAIL均FAIL；缺release仍UNOBSERVED，C不误算Rust额外模式。当前RT **32/32目标配置证据通过**，certification_complete=true；RT资源/生命周期整体仍另待，不能冒本域所有义务完成。
- **32 RT原帧反控**：4原合法PASS、4双方同错FAIL、24缺-O/混标量flags/未知field/错source/profile/caps REFUSED，scope/record-text-release-frozen/raw-reader-controls.json SHA=`fc1ddc11d276721044c6996699a3e87e5677bdfd2edb95a4f7370dd297f9c243`。原48标量反控在刷新帧重放仍8PASS8FAIL32REFUSED，SHA保持`95503e2764a50ec46be7335264da647fa18fcafa19c42b320c39724f39acf7b7`。均counterfactual_not_execution，不充语言执行。
- 新永久参数守恒正/反控联原153，`cargo t infra-ace-m1-arch --out <scope>/record-text-release-final-controls` **154/154 PASS**、4.7336548s、exit0，results SHA=`376a4a81e8a2baaffec5db2113364da788303e3f14fa36bf92eaa82f3d61952a`。定向rustfmt(skip_children)、py_compile、diff-check通过。提交后7变更文件均等Git blob（scope/record-text-release-frozen-preview/committed-input-audit.json），复用同代码154证据，不为commit重测原1779或190默认认证。

健康记录保真：初次准备发现新observer unused Value import，已在执行与前后快照之前删除，正式准备/执行无此warning。尝试要求actual所有stderr为空的额外审计失败，随后逐项证明**24个compile stderr均与原默认证书同腿字节完全一致**：独立参考12条全空；既有scalar两产物4条unused_parens/unused_variables、R08两条non_snake_case原发射warning保留，scope/record-text-release-frozen/compile-warning-baseline-audit.json。未抑制lint/改flags/修生成源码；旧first_lex_fact基线dead_code亦未改，最终T13需如实保健康边界，不谎报全部编译零warning。

冻结351dae1整组合原受管Job/Popen禁用重读：**394PASS/8FAIL/484UNOBSERVED**，原登记886/required/legacy38不变，full_registry_closed/system_delivered/expanded claim仍false。scope/record-text-release-frozen-preview/preview.stdout 397060B SHA=`b5e59e6d9c9e426d266d4bb2f8fef632bdab2e1d61a70644d4e7f75f46ada004`，10453.5749ms、peak process28590080/Job29822976B；单共享LiveBudget reservation31051093/全程peak**47280323<50331648B**。48MiB聚合/4MiB单件/private128MiB/解码先预留/64KiB流hash原帽未改、不清峰值。累计190默认cert/daily+12原额外Rust模式有实证，不能替剩余collections/SQ/实例、旧313/38、机制/ARCH/938939及独立最终review/merge。无活进程或新增用户决定；下一入口继续原collections等剩余四端与机制，原8FAIL/8native trap缺观测保留。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:351dae1049e6bf554006a971e73b8b3d30ba71ce | task_ids:T04/05/10/11（未关闭） | evidence:4原RT优化+8scalar身份刷新/24compile24run；80反控；154回归；整体394PASS8FAIL484UNOBSERVED | blockers:原完整任务未完，无新增用户决定 | next_responsible_role:work 原collections/SQ/实例四端与机制/ARCH

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:27 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原完整任务未完，RT四优化配置缺口已补） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:351dae1049e6bf554006a971e73b8b3d30ba71ce | next:work 原剩余完整四端与机制/ARCH

### 2026-11-06 原collections二十源AV/native严格准入（goal第28轮）

前轮RT四release完成为progress。本轮继续原T03/04/05/10/11，唯一941树提交 **ff9ce906227b750ac865e6ed1b84da1e60d9f187**，10文件，树clean；仍executing2/17，未改任何生产编译器/worker/资源语义。

- 从已冻结Q4与composite两合同逐行核对**collections-926原20源**，原19值/1具名ListIndex、profile=collections、caps=[]及源码SHA不改。永久collections-q4.case.ad/.suite.ad保原AV **ac-av/collections-source-run-v1** 与native **ac/mir-native-collections-v1**，不用fixture或通用scalar入口替代。原L06分别预期av.trap.list-index/mn.trap.list-index；原19值含0、3、51、53等不统一改42。
- 首轮正式40/40功能PASS，但该工具会话未设置AUTO_CASE_WIRE_EVIDENCE，只有父receipt而无原始报文，**不供严格准入**；保scope/collections-q4-source/gate，未事后伪造START/TERMINAL或回填原帧。发现缺采集后显式开原留存开关，源码/oracle/预算不变，第二轮同正式完整40 **40/40 PASS**，48.4272582s，scope/collections-q4-wire-source/gate/results.json SHA=`bda26a90f4edf38ecf10d6e5a2329c846f875341d7de9b54fbeab19497597e7f`。两轮是80实际检查但仅40唯一目标观察，不重复计覆盖。实际worker各由原prepare_tools准备；AV/new native collections worker/实际5.6MB runtime lib与stamp/相应来源共**12依赖**执行前后SHA稳定，原库只64KiB流hash，不整件解码。
- 正式/2 inventory实际导出，永久portable合同92514B SHA=`6e6095e16b9df77c4918218221eddf47ddfbdbda433dc4599ea5ab048f785086`。第二轮Case-copy对原实际inventory契约逐项核对，inventory/原合同/40选项与各原source/caps/oracle一致；首轮inventory可复用因Case/suite/源/producer字节未变，不把无wire首轮的父PASS用作raw语义证据。
- 新ace_m1_collections_evidence只读校验独立原表、实际wire START/TERMINAL及当前**mn-collections-worker**独立身份（不套mn-worker SHA）、sourceSHA/FNV/base64、原nonce与重算requesthash、protocol/recipe/version/域、owner/store/work、真实父终态。AV source输出collections=null，明确不伪造fixture ledger；value或具名trap分别对原oracle。native完整manifest/artifact/runtime ABI+build stamp/link identity、实际link/run/原cold counters与spans/worker及allocation计量结构校验；L06必须原**MN-COLL-TRAP code=21 + exit70**，正常值不能凭70冒trap。runtime INDEX=-21原定义已核，不从期望倒造输出。
- native运行后collection live/bytes/guest/arena/registered/text_metadata为0、allocs=frees、invalid=0；嵌套text前缀按既有严格native_events逐owner配对并与SUMMARY实际allocs/frees及FREESTAT核对。初版reader只认纯MN摘要，按严格边界拒合法嵌入text事件；随后复用既有LIT/FREE等事件语法，仍拒未知/重复/缺释放，不删日志或放宽oracle。**40/40严格语义PASS**；原完整fault、恢复、HMR、资源上限及P90仍UNOBSERVED，不能以一次冷值和清理摘要顶T06..09。
- **60原始帧反控**：L01/L06/LSTR864/N889-nested-variant/V06五源×两端；10原合法PASS、10双方同错值/错具名trap FAIL、40nonce/未知field/caps/源码且连重hash+FNV REFUSED，scope/collections-q4-wire-source/raw-reader-controls.json SHA=`b2ae61bcb2d095d6b7779633baa5eb3eabecffb8915debedce34902e1d3c1d7c`。反控脚本首有未显式导入私有FNV helper的NameError，仅修临时脚本并重跑，无语言失败/新语言执行覆盖；全部counterfactual_not_execution。
- 新3永久回归涵盖独立值/具名trap、source与fixture混淆、nonce/caps/源码连重hash、完整native text/collections清理及缺释放/漏字段/坏trap；联原154，`cargo t infra-ace-m1-arch --out <scope>/collections-source-final-controls` **157/157 PASS**、5.4871164s、exit0，results SHA=`3f51efce4f7d142380997dff13a13332eb791d9c601968ff212a694b6d1e5730`。py_compile/diff-check通过。10变更文件提交后等Git blob，scope/collections-source-frozen-preview/committed-input-audit.json；原生产工具/190默认cert+12额外模式未变，严格重读后复用，不为新增Python/Case重复1779压力。

冻结ff9ce90整组合原受管Job/Popen禁用重读：**434PASS/8FAIL/444UNOBSERVED**，登记886/required集合不变，新增collections_q4为第12合同指针；full_registry_closed/system_delivered/expanded claim仍false，原8scalar失败和8native trap缺证据保留。scope/collections-source-frozen-preview/preview.stdout 416357B SHA=`b6968a4225a72f93f7231ede3fc042b6610afae8af661e120214f9ffba303eb1`，8879.3650ms、peak process30310400/Job31551488B；同一LiveBudget reservation32132924/全程peak**47287851<50331648B**。原48MiB聚合/单件4MiB/private128MiB/先预留解码/流hash原帽与不清峰值保持；collections外层保1MiB摘要、每exchange32KiB，原对象真正失去引用后释放。

本轮没有新增Rust/C认证：collections原20源×Rust/C的40目标及其原模式/配置、SQ/实例与完整机制、旧313/38、938939、最终独立review/merge仍待，不提前勾T01..15。无活进程或新增用户决定；下一入口补原collections独立Rust参考及双端真实认证，再继续其余原范围。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:ff9ce906227b750ac865e6ed1b84da1e60d9f187 | task_ids:T03/04/05/10/11（未关闭） | evidence:40原源两端严格PASS/12deps双快照；60反控；157回归；整体434PASS8FAIL444UNOBSERVED | blockers:原完整任务未完，无新增用户决定 | next_responsible_role:work 原collections Rust/C认证/其余四端与机制/ARCH

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:28 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原完整任务未完，collections AV/native证据已补） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:ff9ce906227b750ac865e6ed1b84da1e60d9f187 | next:work 原collections独立reference/认证及剩余完整范围

### 2026-11-06 原collections二十源独立Rust/C默认认证（goal第29轮）

前轮40源观察为progress。本轮继续T03/04/05/10/11，唯一941树提交 **ff18aaaab202c1b2bd8aa61d00d2d9f4dad3f586**，10文件，树clean；仍executing2/17，生产编译器/worker/语义未改。

- 实际检查发现904原已提供**20份手写std-only Rust reference**，逐份读回并与原源语义/独立oracle核对后直接复用，不复制/生成另一套Rust参考；C保原composite已冻结20参考，逐字节等于926原reference文件。两套均不读actual发射代码，L06用checked get输出具名ListIndex，Unicode长度用chars计数。参考中部分不走的分支按原选定输入规范化，不能据此声称额外分支覆盖。所有40参考均逐件等当前Git blob，未顺手修参考或压warning。
- 新20行collections-reference-contracts.tsv SHA=`f0b4adb0985304e6706215ff349f6f38a02af487d7318159ec8ca4772a7fdde1`，把原source/profile/caps/oracle、原Rust/C参考路径/SHA、debug配置及20×2默认义务固定。当前真实原worker顺序执行**40默认cert+40对应daily+40编译器身份请求**，actual/reference独立编译与运行，合**80compile+80run**，全部PASS、无失败尝试；daily完整四文件在cert前冻结。scope/collections-certifications/frozen-bindings.json SHA=`9931058191716dd8ff84499343d63876dbe14f9711f10e6139610c6deb5c360b`；120原命令sum wall92055.9576ms、max Job209178624B，command-summary SHA=`756049ddd206c68be385c3ec4b1febfcf4f196c262c5d225310d7bc06d819a29`（非正式P90或父整包墙时）。
- **全部40目标的每份完整main/runtime/generated/manifest字节仍等于原904/926金样**，不是剪支/改源重新造golden；scope/collections-certifications/original-golden-byte-audit.json SHA=`d88666914acdf74fc4c30e939d17fd553d450472c3f4ea2a788670444fe5a7f7`。原AV/native40有效源观察继续保留，无重复执行；本轮默认cert累计**230（190+40）**及对应daily，另外12原Rust额外模式仍另计。
- 原共享cert/daily reader有限增加collections独立域：原profile=collections/HIR=collections-draft、原collections schema/recipe、C专属a2x_c_coll_runtime.h/926 case域、Rust a2x-coll/1 runtime、完整sourceSHA/manifest/实际文件与原toolchain逐件校验。配置身份**Rust证书为canonical JSON字符串，C证书为typed object**，同16,777,216工作预算、cancel/events=false、fault_alloc=null、mode=debug；拒不同表示、错mode/预算/布尔类型/fault/未知field。daily请求同配置、独立nonce/完整专属目录、spawn0/Job2及先冻结文件集不改；四phase真实终态各对独立oracle，父PASS和双方同错不供正权威。
- 边界实证：当前Rust认证器compile_and_run固定`--edition 2021`，即使请求mode=release也只改变collections配置身份，并不传-O；本轮仅显式debug默认认证，**不把配置标签冒优化编译**。原904/926 mode-key对照、独立正式模式/配置及完整资源/fault/恢复/正式样本仍单列待核。本组default_certification_complete=true而configuration_key_controls/optimized_rust_execution/full_resources_and_lifecycle仍UNOBSERVED，未声称collections所有义务完成。
- **50原认证帧反控**：L01/L06/LSTR864/N889-nested-variant/V06×两target；10原合法PASS、10双方同错值/错trap FAIL、30重hash未知response/替配置mode并重算certSHA/冒record-text schema REFUSED。scope/collections-cert-raw-controls/results.json SHA=`a6dbdf409d709c1dc5fdf2a350687330b5603502cd4fec1317a6dc39d70da683`；均counterfactual_not_execution，完整parent/rawhash/实际文件另行严格准入，不充新语言执行。
- 新2永久回归保配置跨target表示/类型/值/模式与daily full source/config连重hash，既有daily正控扩至**六profile×两target**；`cargo t infra-ace-m1-arch --out <scope>/collections-cert-reader-controls` **159/159 PASS**、5.2774971s、exit0，results SHA=`aaa1484cc06fa4ed528226cebf16d2e340c3c85311b58cd7006e674d92de30df`。py_compile/diff-check通过；冻结后10文件=Git blob审计留scope/collections-cert-frozen-preview。未因Python提交重复原1779压力；旧190默认cert、12模式与所有source当前身份/文件/配置由新reader实际重读后复用。
- budget source pin按旧strict约束刷新：逐项确认16政策/全部数字与HEAD完全一致，26源中只更新cert_evidence这一SHA；新预算合同SHA=`1b020d6407084d5ec76b32572af681e1cddc4b73ca26923727cfa8924145b817`，其余25pin/原帽不改。临时更新脚本首次查错registry预算键而KeyError，已核正确expanded_contracts.budget并完成同源hash一致更新，不跳过STALE/预算检查。

健康记录：C actual/reference均零warning；Rust实际20流共138 warning、原reference两流共5 warning（N889-nested-variant/V06）。原参考均不改、实际完整产物逐件等原金样，因此是原发射/参考基线，不是本轮新代码warning；scope/collections-certifications/compiler-warning-baseline-audit.json保每流SHA/计数，具名**HEALTH-941-COLL-WARN**交T13健康/owner与最终独立review处理，不能谎报全部零warning，不在941例外授权之外改生成源码/加lint屏蔽。原完整资源与机制仍未被健康审计替代。

冻结ff18aaa整组合原受管Job/Popen禁用重读：**474PASS/8FAIL/404UNOBSERVED**，登记886/原required不变，第13合同指针collections_reference；full_registry_closed/system_delivered/expanded claim仍false。scope/collections-cert-frozen-preview/preview.stdout 447947B SHA=`aeb940a759aa3852d02c964607a2197b45deb87b298d227ae2b223a1ae9d7ea5`，9786.2158ms、peak process29782016/Job31023104B；同一LiveBudget reservation33195144/全程peak**47289027<50331648B**，保48MiB聚合/4MiB单件/private128MiB/先预留解码/64KiB流hash与不清峰值；外层1MiB/每cert128KiB有界摘要。原8FAIL/8native trap缺观测、旧313/38与938939/ARCH不被新40绿掩盖。

原T01..15未闭；下一入口继续SQ/实例同源四端及原collections配置/资源与完整机制，最终独立review/merge/清理仍待。无活进程或新增用户决定，授权持续。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:ff18aaaab202c1b2bd8aa61d00d2d9f4dad3f586 | task_ids:T03/04/05/10/11（未关闭） | evidence:40原Rust/C默认cert+daily/80compile80run；50反控；159回归；整体474PASS8FAIL404UNOBSERVED | blockers:原完整范围/机制与健康基线未闭，无新增用户决定 | next_responsible_role:work SQ/实例四端及原模式/机制/ARCH

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:29 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原完整任务未完，collections默认认证已补） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:ff18aaaab202c1b2bd8aa61d00d2d9f4dad3f586 | next:work SQ/实例四端与原配置/完整机制/ARCH

### 2026-11-06 原SQ/实例十二源AV/native与typed诊断准入（goal第30轮）

前轮collections40默认认证为progress。本轮继续T03/04/05/08/10/11，唯一941树提交 **8f5e748fa6fe804cc6e98c48d8c1c17fed85a363**，9文件，树clean；仍executing2/17，未改生产query/instance/编译器语义。

- 直接复用原934/935两套Case及固定入口，不另造改写源/期望的替代集。**SQ六源**leaf42/leaf43/named7/trace-ab7/两type-mismatch负例，**instances六源**choose42/choose0/expected-type/cross-wrap/recursion/neg-hetero；保原profile=scalar、instances capability与trace-ab7原trace cap、各原源码/SHA/独立值/诊断/marks。choose真假双实例、模板交换、期望类型、跨模板与递归只按本源值实证，完整实例物化/required页/恢复仍另待，不能从值倒推机制。
- 执行`cargo t case-ace-source-query-934 case-ace-source-instances-935 --out <scope>/query-instances-source/gate`，原wire开关显式1，**14+12=26/26 PASS**、15.7907889s，results SHA=`ce989b1011757ddd4200bfd2347775ffeb09744453b147c407f6158486e6b0b0`。0/result14 SHA=`99ddcd1287055e55b5ac57b21954ea163e943cce03f1da55e25b67e9a72c3d91`、1/result12 SHA=`185ab0c8add4e599a0a789c35f3aaabd039cd4f7753377cd0b6404ffa5ec36a1`。其中**24才是AV/native语义检查（18值+6预期诊断）**，额外2仅leaf42 Rust/C完整金样发射，不冒运行/认证，不充24分母。AV/mn/a2x现worker、真实trace-runtime及query/实例来源共13依赖前后SHA稳定。
- 两份原正式/2 inventory实际导出并固化portable合同：source-query-q4-contract.json SHA=`56ba33d6ad07c3f77378d681f5adeb2f15e66e0ff728cfd5d1fe328a438e159b`（原14检查）、source-instances-q4-contract.json SHA=`dc966586605122ded12ff2d2747026df4a3b2a7b1b029fcec541b9459507f447`（原12）；新reader核原Case/suite/source/options/oracle、当前producer以及Case-copy/真实START+TERMINAL原字节、nonce/重算requesthash/FNV/base64、recipe/版本、owner/store/work和独立marks。旧scalar reader仅在显式query_family有限增加原实例cap/诊断支持，旧core/scalar值/trap/8真实拒绝及8native通用70缺口逻辑保持。
- 负例不是泛化失败：AV必须原frontend阶段+具名source.type-mismatch/type-mismatch、frontend_reject=1/requests=1且无值/trap/trace；原Case归一parse不被当新parser业务。native必须**原911 source-v1 typed source_error**，校schema/producer/scalar尝试域/本次真实mn.at路径、独立sourceSHA/字节长度、UTF8 span、64 cause/4096 message/64KiB payload原帽、primary=0及cold操作candidate身份（无旧base/session/edit），同时校legacy projection与primary一致。raw verify→原typecheck、bind→原resolve；三负例primary span独立指向原true/!1操作数，范围内却指错字节仍FAIL。缺typed fact/错owner/操作/源码或普通工具失败不能当预期诊断PASS。payload以iterencode计量不另保全件串。
- 严格准入**24/24 PASS**；SQ-A事实权威/solver-off/caller P0/恢复与SQ-B实例物化/坏页/首编辑等仍UNOBSERVED，本轮不从cold结果假推。原2发射检查在golden_emission_only显式semantic_authority=false，Rust/C目标仍待真正独立认证/诊断入口实证，无新增默认cert，原累计230默认cert+12附加模式不变。
- **156原始帧反控**覆盖全部24原exchange：24原合法PASS、24双方同错值/诊断code FAIL、96nonce/caps/未知field/源码且连重hash+FNV/typed-source身份 REFUSED；另三个native负例各wrong-span/old-operation/primary-bool/missing-fact，3FAIL+9REFUSED。总24PASS/27FAIL/105REFUSED均符合预期，scope/query-instances-source/raw-reader-controls.json SHA=`f1985bc1592f20bd4e0281c319cb94710db175e9d780bff94d285b0b9d7b0e4e`；counterfactual_not_execution，完整parent/index/deps另行准入，不充新语言执行。
- 新3永久回归保原实例cap/trace、两消费者具名诊断独立oracle、native操作/primary类型/源码范围；联原159，`cargo t infra-ace-m1-arch --out <scope>/query-source-final-controls` **162/162 PASS**、3.7167689s、exit0，results SHA=`e4bcc4bc774201273e661cdbb42fa7061544a573574426c6c429b92dd1b07bdc`。py_compile/diff-check通过；9变更文件冻结后等Git blob，scope/query-source-frozen-preview/committed-input-audit.json。没有为纯reader/合同重复原1779压力、230cert或12模式；它们现生产工具/配置/源/完整文件不变，已由当前组合真实重读复用。

冻结8f5e748整组合原受管Job/Popen禁用重读：**498PASS/8FAIL/380UNOBSERVED**，886原登记/required集合不变，新query/instances两个合同指针共15；full_registry_closed/system_delivered/expanded claim仍false。scope/query-source-frozen-preview/preview.stdout 462703B SHA=`80b61b401c6397760ec6f80135ce8e873955420abe2cb3757190c7561fc44433`，12500.3819ms、peak process28979200/Job30257152B；同一LiveBudget reservation34281418/全程peak**47304155<50331648B**，原48MiB聚合/单件4MiB/private128MiB/先预留解码/64KiB流hash/不清峰值保持，新增域外1MiB/逐exchange32KiB。原scalar8FAIL/native8trap缺证据及legacy313/38、ARCH/938939/健康基线不被这些预期拒绝绿掩盖。

原T01..15未闭；下一入口补SQ/instances的原Rust/C同源独立认证与三个负例的真实发射前门诊断证据，再继续原模式/完整机制/ARCH/旧义务、最终独立review/merge。无活进程或新增用户决定，既有授权持续。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:8f5e748fa6fe804cc6e98c48d8c1c17fed85a363 | task_ids:T03/04/05/08/10/11（未关闭） | evidence:原26父检查→24语义严格PASS/13deps双快照；156反控；162回归；整体498PASS8FAIL380UNOBSERVED | blockers:原完整范围/机制与健康基线未闭，无新增用户决定 | next_responsible_role:work 原SQ/instances Rust/C认证与诊断/完整机制

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:30 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原完整任务未完，SQ/实例AV/native证据已补） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:8f5e748fa6fe804cc6e98c48d8c1c17fed85a363 | next:work 原SQ/实例Rust/C与完整范围

### 2026-11-06 SQ/实例Rust-C默认认证与负例前门实测（goal第31轮）

前轮24 AV/native观察为progress。本轮继续T04/05/08/10/11，唯一941树提交 **2841619ff261ec660d3cc5530915af0f2cfde708**，27文件（含18独立reference），树clean；仍executing2/17，未改生产编译器/worker/语义。

- 新**9正源×Rust/C 18手平移reference**，不读actual生成源码，保leaf调用/参数、命名实参求值次序、mark_a后mark_b、choose真假两个具体类型、模板交换、expected-type具体i32、跨模板wrap→choose与同实例递归。Rust用独立泛型/具体函数，C用手写typed int/bool函数和显式求值局部量；两流binary保证trace字节。新query-reference-contracts.tsv SHA=`469274867db24637eaa1b6b8a1eb086ec0514b59fdb06d0335ed9b5e9c90ba6b`，逐源/原caps/值/marks及参考SHA固定；原3非法源无伪造reference。
- 原worker真实**18默认cert+18对应daily+18编译器身份**完成，actual/reference各真实编译运行，共**36compile+36run**，18全部PASS；完整四文件在cert前冻结。scope/query-certifications/frozen-bindings.json SHA=`b78fef950f6357c94d82a28b112e6cfe1212d7c335f58947640f5767089791e0`。初次临时脚本继承了RT的C头名a2x_c_rt_runtime.h，9条C daily虽成功却在收集文件时因实际scalar头为a2x_c_runtime.h而停，均未认证/未编译目标，不冒语言缺陷。保9失败attempt/日志，修的仅是临时采集脚本头名；复用已通过且source/ref/worker不变的9 Rust，只在新attempt补9 C，全部通过，未覆盖旧失败材料。
- 新query_certification只在原scalar证书域明确允许原query/instances caps及独立marks，原scalar/text/CT/record/collections守卫不变。所有实际工具/完整文件/manifest与profile/HIR域/原nonce、toolchain flags、默认cert与独立daily/源与参考SHA、四phase原终态/独立oracle均严格核对；trace-ab7为stderr mark_a\nmark_b\n，双方同反序也FAIL。36条编译流actual/reference Rust/C均零warning，scope/query-certifications/compiler-warning-audit.json。原230→**248默认cert/daily**，12额外Rust模式仍另计。
- 原935 choose_instances_935.rs直接Rust消费者使用`-O`，choose42/choose0两源码的原额外优化义务不能被默认cert消除；合同明确这两条rust_release_runs=1/flags=[-O]并核原producer源SHA。因此18默认证书全过，登记目标仍**16PASS+2UNOBSERVED**，required_extra_rust_release_runs=2、certification_complete=false。其余7正源Rust及9 C按原默认配置闭合；没有把编译默认mode谎报优化或漏原模式。
- 3负源×Rust/C **6个真实发射前门**另执行原daily-emit/daily-emit-c。均exit2/空stderr、原Job2（launcher+worker）且输出目录未产生，无rustc/cl/target编译执行。source/profile/caps/entry/完整argv/nonce/owner、原worker当前SHA、独立sourceSHA、原meter终态与raw字节严格读回，scope/query-negative-frontdoors/bindings.json SHA=`f5a366327106ff44849b2193f091068a7ede43051d93231cd5726cdfed4bc93d`。实际Rust detail是frontend diagnostics:...、C是a2x.compile:...，能看到原source.type-mismatch/type-mismatch文字。

**GAP-941-A2X-DIAG-01**：原A2X失败API仅{status:error,detail:string}，不提供typed诊断stage/code/source/operation身份；本reader不从rendered detail解析语义权威，6负例仍**UNOBSERVED**，不因exit2或消息“看起来正确”升级PASS。原admit_command默认仍只准exit0；只读负例入口显式expected_exit=2且有限终态0/2、同样核Job/全raw/源与producer，不给cert成功路径开非零绕路。负例outdir必须不存在（通过已验证command正规文件的父目录与固定rust/c名检查，lexists也拒悬挂链接）；初版用file-only EvidenceScope.path查不存在目录而被拒，已按该API真实边界修正，未创建假目录/弱化路径检查。此项是具名观测缺口，未擅自修原worker或冒充已归档能力缺陷修复。

- **30原认证帧反控**：trace-ab7/choose42/choose0/expected-type/recursion×两端；10合法PASS、10双方同错值或反marks FAIL、10重hash未知response REFUSED，scope/query-cert-raw-controls/results.json SHA=`6359e3882fd475ae22b912df97194ec3c7b3583ecf6bfae5b023e69bb113c33e`。另**24负例响应反控**：6原untyped均UNOBSERVED、18加field/冒ok/空detail REFUSED，scope/query-negative-frontdoors/raw-reader-controls.json SHA=`f95e887f646ec49f9e406906ac4b86c4b6be2c4b4389d2d099d246d49d25cbd4`。全部counterfactual_not_execution，不充语言覆盖。
- 新2永久回归保exit2永不能冒成功command、受控refusal读取也不供cert verdict、未知终态/布尔拒绝及rendered错误绝不充typed PASS；联原162，`cargo t infra-ace-m1-arch --out <scope>/query-cert-final-controls` **164/164 PASS**、6.5553390s、exit0，results SHA=`f061c52eda40a3df18e4ef0e7b93f9db5e89635f572e3c677b7418abf4850696`。py_compile/diff-check通过。冻结后27变更文件等Git blob，scope/query-cert-frozen-preview/committed-input-audit.json；生产源/配置/工具未变的旧230cert/12模式/各source实际逐项重读复用，不为Python提交重复原1779压力。
- 原预算16政策/全部数字逐项等HEAD；26源只刷新cert_evidence一pin，其余25不动。新预算合同SHA=`5f51c01257dfa28e2aa82a0b28645229f52bfed029f2ea75f94c4bb90d69ec7d`。新增query_reference为第16合同指针，不改886登记/required集合或原NA。

冻结2841619全组合原受管Job/Popen禁用重读：**514PASS/8FAIL/364UNOBSERVED**，full_registry_closed/system_delivered/expanded claim仍false；原8FAIL/native8无具名trap/legacy313与38/ARCH与938939/健康边界继续保。scope/query-cert-frozen-preview/preview.stdout 483219B SHA=`30f9cbd993972c2acd57c9d1859c18382ff350045c4d4a7a65fa2b8f013f2587`，9089.0422ms、peak process29921280/Job31166464B；同一LiveBudget reservation35478245/全程peak**47773019<50331648B**，原48MiB/4MiB单件/private128MiB/先预留解码/64KiB流hash/不清峰值保持。新正证据域1MiB、负观察128KiB及逐项有界摘要，不靠另开预算拼总门。

原T01..15未闭；下一入口补choose原两Rust优化配置，并继续原SQ/实例/CT/collections资源、事实权威/恢复/原规模与ARCH/legacy机制，最终独立review/merge仍待。六负例typed观测缺口具名保留，不从错消息/跨target复制证书解决。无活进程或新增用户决定，授权持续。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:2841619ff261ec660d3cc5530915af0f2cfde708 | task_ids:T04/05/08/10/11（未关闭） | evidence:18默认cert+daily/36compile36run，原2优化待；6真实负入口typed缺口；54反控；164回归；整体514PASS8FAIL364UNOBSERVED | blockers:原完整范围与具名观测/健康基线未闭，无新增用户决定 | next_responsible_role:work 原choose两优化及完整机制/ARCH

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:31 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原完整任务未完，SQ/实例默认认证已补） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:2841619ff261ec660d3cc5530915af0f2cfde708 | next:work 原choose两优化及其余完整范围

### 2026-11-06 原choose两项Rust优化配置与共享测试身份刷新（goal第32轮）

前轮18默认认证/6负入口为progress。本轮继续T04/05/08/10/11，唯一941树提交 **93f692916f03516b98f797747f9986c6a9ca3e31**，5文件，树clean；仍executing2/17，生产worker/编译器/语义未改。

- 新正式producer `ace-a2x-worker::conformance::query_release_941::original_two_release_variants`逐原935 **choose42/choose0**，源码直接取原source-instances-935文件，运行字节与编译时include_bytes逐件等；profile=scalar、原instances cap、独立值42/0不改。原精确优化参数 **--edition 2021 -O**，不混scalar的-C debug-assertions=off。actual与上一轮独立reference各两次真实编译/运行，新增**4compile+4run**；完整main/runtime/generated/manifest全等默认cert/daily冻结文件，不能拿不同代码-O结果代配置。observer始终能从永久fixture自行重建，不依赖合并时会删的材料；复用原contained capture、原120s compile/30s run/360s whole/1MiB双流帽，未造runner/meter。
- 新模块注册改变conformance可执行文件身份；原scalar8/RT4的旧前后快照保历史，不把新SHA回填旧before。本轮统一命名入口 `cargo t ace-a2x --test-name release_941::original_ --out <scope>/query-release-frozen/gate` **3/3正式测试，内部8scalar+4RT+2query共14原配置**，exit0、7.1833073s，results SHA=`28eaf4e88999aa13b34071fa8d506edb923bb74c468f4d866ac9670b76a55e8a`。当前三组分别8/9/9个dependency pin（含实际测试exe、debug worker、producer/原表/参考合同、executor/nextest配置）执行前后稳定；scope/scalar-release-query-refresh、record-text-release-query-refresh、query-release-frozen。共**28compile+28run**，其中24+24为必要旧身份刷新，不冒整个ace-a2x职责组全过。
- 共享只读release reader有限加入query第三域，禁止混域；按原query producer/935表/独立合同、2源全集/原profile/cap/flags/完整文件集、原父终态与discovery实际exe、前后依赖/当前-vV、nonce/FNV/完整emit原字节和四phase逐项校验。默认与优化双腿联立，任一FAIL仍FAIL、缺优化仍UNOBSERVED；当前原query正源**18/18目标配置证据完成**，certification_complete=true。原6负例typed缺口及SQ/实例事实权威/恢复并未完成，不能由本字段冒所有义务完成。原248默认cert/daily不变，额外原Rust模式从12→**14**，不虚增默认认证数。
- **16 query原帧反控**：2原合法PASS、2双方同错值FAIL、12缺-O/混scalar flags/错profile/caps/source/未知field REFUSED，scope/query-release-frozen/raw-reader-controls.json SHA=`0edddbeeea93f3187c921ff0805b7224bf832181e6add79ee54a9dbaee75bc7c`。初次临时反控沿RT脚本把profile改scalar，而原query本来就是scalar，未真的改变条件，因此合法PASS触发反控断言；修的是临时变异令其实际改为core，未改checker/源码/oracle，重放才作为当前反控证据。原48scalar与32RT反控在新实际帧重放仍全符合预期，SHA分别保持`95503e2764a50ec46be7335264da647fa18fcafa19c42b320c39724f39acf7b7`与`fc1ddc11d276721044c6996699a3e87e5677bdfd2edb95a4f7370dd297f9c243`；总96counterfactual，不充新语言执行。
- 现有原通用flags/默认与优化不互相掩盖等治理回归在最终代码 `cargo t infra-ace-m1-arch --out <scope>/query-release-final-controls` **164/164 PASS**、4.3455418s、exit0，results SHA=`5bf9c4fd17ca28026bbb58c06a4e11d0b935185f48c010622a9a1c57244c035d`。新增实际双源producer本身是永久注册回归，不为同一通用规则再加镜像计数测试。定向rustfmt(skip_children)、py_compile、diff-check通过；冻结5文件等Git blob留scope/query-release-frozen-preview/committed-input-audit.json。原248cert/所有source生产身份不变，组合逐项重读后复用；原1779压力及最终完整ace-a2x职责并集不在此命名门冒PASS。

健康/准备记录保真：28个compile stderr逐件等原默认证书同腿，新增query四流全空；旧scalar4/RT2条发射warning和first_lex_fact dead_code基线保持，无新增未处理warning。scope/query-release-frozen/compile-warning-baseline-audit.json SHA=`20e44fd64b897e562f381c1062658e5a04ec8fc2b9eccde04aa97a41584290d2`。本轮准备曾复用旧prepare-record-text-release脚本，更新了其目录内三个临时准备缓存formal-discovery/formal-build.stderr/formal-build-command，已把本次副本置query-release-preparation并另存新脚本；**旧实际gate、原START/TERMINAL、index与before/after均未改**，当前准入只使用本轮新gate/快照，不把更新缓存冒旧执行证据。

冻结93f6929全组合原受管Job/Popen禁用重读：**516PASS/8FAIL/362UNOBSERVED**，886登记/required/16合同指针与原预算合同不变，full_registry_closed/system_delivered/expanded claim仍false。scope/query-release-frozen-preview/preview.stdout 484506B SHA=`15e6e12929aae3a15a6c778f37258f018b8ac5a73e4a5b05c361e17298de8c1b`，7942.7770ms、peak process29696000/Job30928896B；同一LiveBudget reservation35493698/全程peak**47785683<50331648B**，原48MiB聚合/4MiB单件/private128MiB/先预留解码/64KiB流hash/不清峰值保持。原8FAIL/native8trap缺具名观察、6 A2X负例typed缺口、legacy313/38、ARCH/938939及健康基线均未被优化绿掩盖。

原T01..15未闭；下一入口转原完整源/模式分母审计与事实权威、增量恢复首编辑、required页/资源/正式规模、ARCH及legacy证据，最终独立review/merge仍待。无活进程或新增用户决定，授权持续。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:93f692916f03516b98f797747f9986c6a9ca3e31 | task_ids:T04/05/08/10/11（未关闭） | evidence:2原query优化+12旧模式身份刷新/28compile28run；96反控；164回归；整体516PASS8FAIL362UNOBSERVED | blockers:原完整范围/机制与具名观测/健康边界未闭，无新增用户决定 | next_responsible_role:work 原完整分母及事实/恢复/资源/ARCH/legacy

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:32 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原完整任务未完，choose两优化配置已补） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:93f692916f03516b98f797747f9986c6a9ca3e31 | next:work 原完整源/模式审计与机制/ARCH/legacy

### 2026-11-06 原源全集审计与四个漏列标量变体（goal第33轮）

前轮原14 Rust优化配置完成为progress。本轮继续T01/03/04/08/10/11，941唯一树提交 **7ec6e56700df92ebb8577bab46d366f3bdc7a6d3**，11文件，树clean；状态仍executing2/17，生产编译器/worker/语义未改。

- 对原case-obligations.tsv列出的**35套件/511 checks**逐套重新调用当前正式auto-case identity-inventory，原受管capture保argv/退出码/计量/完整双流及producer前后稳定SHA；这是只读发现，不充语言执行。按group/suite/case/check逐件核contract、effective check、aggregate source、oracle、case文件指纹，**511/511一致、0漂移**。scope/source-universe-audit/index.json SHA=`73676713b1a01734f5444e436f1ab34e30412b3195aea8d8507f64c835ab44d6`、case-contract-current-audit.json SHA=`897419d2ac323105f710ea57f3a09641017a2f23925d634f52e1bd412c5487bb`。另按实际source_files字节而非aggregate hash交叉核对，共**183不同源hash**，209 checks匹配初始126源码目录（source-crosswalk.json SHA=`1da2fbc6f2f8e4b2c321a121d4a8f176a3e9f7d785bab2346f5f1d983d620f79`）。其余项须按原承诺/机制/目标独立分类，不能自动作NA；35套件亦不代程序化资源/恢复/规模全部范围。
- 发现原892/921已承诺但初始126漏列的四个真实字节变体：C01 helper倍增=42、C02 i32MIN/-1=DivOverflow、C03双层调用=61、NAMED-01命名参数源序=23且marks b→a。新增独立source-supplement-contracts.tsv（4原源/原Case/profile/caps/oracle/marks/owner与hash）SHA=`f0642c510f36ac682c71a1ad66228f4ca103162388f8dd3b7c124a2e5c66b6b6`，原126与其SHA不改。四端**新增16 required登记**，原886为历史部分目录，当前**902=313legacy+520源码+49ARCH+20preview**，不是最终M1分母；原required集合/NA/旧38cert-gap不变。永久治理控制保四源全部四端独立键，缺任何观察仍UNOBSERVED；即使合成全PASS仍不得关闭完整范围。
- 新正式固定组 `cargo t case-ace-m1-source-supplement-941 --out <scope>/source-supplement/gate`，原source直接引用，无改写，AV/native **8/8父PASS**、exit0、6.6994236s，results SHA=`ee68bc1c6a8487756f804de6408eaa5b3d334deb156fe8c5466e6516d1d7fde2`。原trace-runtime亦独立核身份。便携冻结/2合同18384B SHA=`f105ecee77cc2ed20885da5332b5d4263b8ed0f92e092b704f8d2d530c71d600`；专属reader验证原源与原Case/profile/caps/trace、完整8inventory、CaseCopy清理/全产物、当前worker、原START/TERMINAL、nonce/FNV/全response与独立值/具名trap，不从父PASS推导语义PASS。严格**7PASS/0FAIL/1UNOBSERVED**：C02native只exit70，无独立具名trap，沿原边界保UNKNOWN，不能冒DivOverflow已证。Rust/C新8目标尚未认证。
- **48原帧反控**：8原帧保7PASS/1UNOBSERVED；8同错值/错trap/错exit FAIL；32nonce/cap/source/未知field REFUSED。scope/source-supplement/raw-reader-controls.json SHA=`e5066802b782870f207c55421e3635bbe653f7dc3ec32b8c0bfd22b016fe9a37`，counterfactual_not_execution，不充源码覆盖。治理首轮164中163PASS/1ERROR：本轮机械替换误给read_tsv多传SHA参数，已修正调用、保独立hash校验；首轮原记录保source-supplement-final-controls，不抹红。最终 `cargo t infra-ace-m1-arch --out <scope>/source-supplement-final-controls-replay` **164/164 PASS**、exit0、7.1340136s，results SHA=`006ebac7eda4642d2a6e02afd39cefcc77e1ed869b91996072e7c88ef7ee8302`。py_compile/diff-check通过。冻结11变更文件等Git blob，scope/source-supplement-frozen-preview/committed-input-audit.json。

冻结7ec6e56全组合原受管Job/Popen禁用只读重验：**523PASS/8FAIL/371UNOBSERVED**，902登记/18合同指针，full_registry_closed/system_delivered/expanded claim仍false。scope/source-supplement-frozen-preview/preview.stdout 491148B SHA=`79c2bf67436c9f4952c92e13026e8ddc8a3af0a028df45b4827686574b36615f`，10341.8717ms、peak process29966336/Job31211520B；同一LiveBudget reservation36043052/全程peak**48325698<50331648B**，原48MiB/4MiB单件/private128MiB/先预留解码/64KiB流hash/不清峰值保持。新域512KiB有界摘要；原248默认cert/14优化与旧source身份不变，逐项重读复用，不为新增只读域重复原1779压力。原8FAIL、当前9 native trap具名缺口、6 A2X typed负例、299legacy/49ARCH等UNKNOWN与健康基线均保留。

原T01..15未闭；下一入口为四个漏列原源的Rust/C真实独立认证，并继续完整源/模式分类、事实权威/恢复首编辑/资源/正式规模与ARCH/legacy；最终独立review/merge仍待。938/939尚未交齐，不能据他树提交冒交付。无新增用户决定或活动验证进程，授权持续。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:7ec6e56700df92ebb8577bab46d366f3bdc7a6d3 | task_ids:T01/03/04/08/10/11（未关闭） | evidence:35套件511原合同零漂移；4漏源16目标登记；8真实AV/native；48反控；164回归；整体523PASS8FAIL371UNOBSERVED | blockers:原完整范围/机制与具名观测/健康边界未闭，无新增用户决定 | next_responsible_role:work 四漏源认证及完整机制/ARCH/legacy

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:33 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原完整任务未完，四漏源已补登记/双源端） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:7ec6e56700df92ebb8577bab46d366f3bdc7a6d3 | next:work 四漏源Rust/C独立认证与完整范围

### 2026-11-06 四个漏列原源的Rust/C独立认证（goal第34轮）

前轮35套件/511合同零漂移及四漏源登记/AV-native为progress。本轮继续T04/05/08/10/11，唯一941树提交 **fab1690e6c1870438c98ff1c10fadd108726c474**，12文件，树clean；仍executing2/17，生产源/worker/配置未改。

- C01/C02/C03/NAMED-01四个原始源各跑真实Rust/C完整actual+独立reference认证与独立daily，**8/8严格认证PASS、8/8 daily PASS**，新增**16compile+16run**；原source/profile/cap/oracle/marks保持，C02须具名DivOverflow，NAMED-01须23且mark_b→mark_a。重用原921 C01.c与892 NAMED-01.rs独立参考，另六份手写参考永久入树；C02 Rust使用标准checked_div，C参考以int64计算并独立判断i32范围，避免C signed overflow UB。新四源参考合同SHA=`e0aa3c236ea541624fbc31661fc739d0d99989ae4de5ca41a8a51dac0ea559ec`，登记仍UNOBSERVED不冒运行。原892/921对应默认认证配置，没有擅增或用旧14优化结果充这四源配置。
- 原显式worker受管capture逐命令保完整argv/exit/计量/raw，8认证单次父墙590.9980..3897.9931ms，不冒正式P90；原360s whole/120s compile/30s run/每流1MiB及各内门不变。scope/source-supplement-certifications/frozen-bindings.json SHA=`63c8786334101c950279a4285acc337a371cebd87cc5ac1605017ff1ae0b3fa8`；execution-summary.json SHA=`ad15df40473c19b57015056b1ee673f6596be85236a573f74c46d6dd488fafd7`。只读reader逐原源/参考合同、当前worker/显式工具链和flags、nonce/FNV、完整fileset/证书schema与四phase终态、独立值/trap/marks、actual/reference分别校验；相邻daily与cert完整四文件相等并由原Job进程数核daily未启动编译器。原native C02通用70观测缺口仍保，不拿Rust/C具名trap替它PASS。
- 原C三套+Rust命名参数一套共**16个原golden文件全等**，scope/source-supplement-certifications/original-golden-audit.json SHA=`d364f46942cdaf34bc31422d18605bf2bdec1125923cbfa4e7009c448ec43467`；全部**16 compile stderr为空**，compile-health-audit.json SHA=`d407ca6567d23de226edd4cde548a7ce2006ae76a7be1274d4c318dc116ed5c8`，无新增warning，原collections等健康基线仍保。
- **24原认证反控**：8原合法PASS、8 actual/reference双方同错值/错trap/反marks FAIL、8 rehash未知response field REFUSED，scope/source-supplement-cert-raw-controls/results.json SHA=`42bfaad8e986fa969d9a520e47d59d24ce3aaad488d69bffe76f669d99273231`；明确counterfactual_not_execution。新增永久原源/独立参考内容漂移拒绝回归，三类实际文件改字节均STALE，不靠当前双方相等覆盖原身份。`cargo t infra-ace-m1-arch --out <scope>/source-supplement-cert-final-controls` **165/165 PASS**、4.1058275s、exit0，results SHA=`2dbc34ab6b55d5dd5cd5386b51234a0e1c868c66ecac5bcbf70a0221ad6f6261`；py_compile/diff-check通过。冻结12文件等Git blob留scope/source-supplement-cert-frozen-preview/committed-input-audit.json。

冻结fab1690全组合原Job/Popen禁用只读重验：**531PASS/8FAIL/363UNOBSERVED**，902登记/19合同指针；原126主目录、旧313/NA/38cert-gap、预算合同保持，完整范围/验收系统交付/expanded claim仍false。scope/source-supplement-cert-frozen-preview/preview.stdout 497919B SHA=`595a4809893a4f219a91748a38aa9b21da66f16c0c8b555fcefa1d29a77f3fd1`，13586.6583ms、peak process29634560/Job30949376B；同一LiveBudget reservation36577854/全程peak**48854890<50331648B**，原48MiB/单件4MiB/privateΔ128MiB/先预留/64KiB流hash/不清峰值不变，新认证域512KiB摘要。当前默认认证/daily从248→**256**，额外原14 Rust优化不变；生产身份未变旧证据逐项重读复用，不重跑原1779压力冒职责全过。原8FAIL、9native trap具名缺口、6A2X typed负例、299legacy与49ARCH未闭，不能被新8绿掩盖。

原T01..15未闭；下一入口继续183源字节集合的批准范围逐项分类，补原方法/记录等漏列变体与机制/事实权威/无预热恢复/资源/原规模、ARCH/legacy，最终独立review/merge仍待。938/939尚未D，无新增用户决定或存活验证进程，授权持续。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:fab1690e6c1870438c98ff1c10fadd108726c474 | task_ids:T04/05/08/10/11（未关闭） | evidence:8默认cert+daily/16compile16run；16原golden等；24反控；165回归；整体531PASS8FAIL363UNOBSERVED | blockers:原完整范围/机制与具名观测/健康边界未闭，无新增用户决定 | next_responsible_role:work 完整源/模式分类及机制/ARCH/legacy

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:34 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原完整任务未完，四漏源Rust/C认证已补） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:fab1690e6c1870438c98ff1c10fadd108726c474 | next:work 完整源/模式分类及其余完整范围

### 2026-11-06 原method/record/text范围补登记与55原入口重放（goal第35轮）

前轮四漏源8独立cert/daily为progress。本轮继续T01/03/04/08/10/11，唯一941树提交 **6003a4817b6bfc10d58d2f7f09817cb6e050f6af**，6文件，树clean；仍executing2/17，生产语言/worker/测试配置未改。

- 沿§5.1/原881与text-record合同逐源审查，确认另**26个实际源码字节变体**未列入新Q4：M01..M12共12，R01..R06/R09/R10共8，T01/T02/T04/T06/T07/T08共6。逐原文件推导冻结**17值/1 TextIndex trap/8诊断族**，保原源字节、record/text/text-record/profile、独立共同caps、原Case身份/hash/原checks；M负例分别static误用/immutable写/receiver别名mut/owned保留方法，R负例类型错配/移动后使用，T负例借用逃逸/非法scalar。新original-data-source-contracts.tsv SHA=`a51d7b345013b063e5c04a146896bf509f2213bcab4fdcfe480e2c4bb4f4edf4`。不是从本次结果求能力交集或修expected；诊断行要求实际诊断证据，不能为非法源造exe/以任意非零充拒绝。
- 严格来源reader核26有限集合、各profile/caps/语义、原始source+Casehash、原511合同指针及与原126/四scalar补源无字节重复；不凭basename别名吞并。原131/313/NA与已登记源全保。四端每源独立必需键，**新增104 required**；当前**1006=313legacy+624源码+49ARCH+20preview**，20合同指针，仍是未闭完整范围的部分登记。永久控制明确12方法+14记录文本全部四端，缺证据全部UNOBSERVED；原自测全PASS合成门也不使完整范围关闭。
- 原正式 `cargo t case-ace-m1-method-881 case-ace-m1-tr-native` 真执行22+33=**55检查**（含7个已登记源/重复来源原checks），没有以新源表替代原样本。首轮scope/original-data-source/gate中55子项虽PASS，但我在运行期间新增catalog触发整体`code/config/inputs changed during execution`，exit1、gate=false、38.4686298s；原记录SHA=`6b4b6728182de83b77b239a17f6e382c13c6ca04168a1c086feec1ff75a8f6a9`保留、不作通过证据。完成全部改动后联合正式门 `cargo t infra-ace-m1-arch case-ace-m1-method-881 case-ace-m1-tr-native --out <scope>/original-data-source/final-gate` **165治理+55原Case=220/220 PASS**、exit0、30.2459567s，results SHA=`1f2bc2e3c4271814bad1e9c2d8dc568aed4166e51f57b46bcbc251b0ce6b3d34`。
- 上述第二轮新shell漏带wire记录环境，父执行有效但严格_raw准入探针因缺原exchange而拒；未删门/宽松读取补PASS。冻结6003a48后显式设置AUTO_CASE_WIRE_EVIDENCE=1及唯一scope，再跑两原Case组 **55/55 PASS**、exit0、25.9007745s，scope/original-data-source/final-wire-gate/results.json SHA=`4d340fd2e7f4727c89bc1c1d3bab101f46822e5fb788814d70c0728d08775702`。当前55逐项对原511冻结contract/check/source/oracle/Casehash、完整CaseCopy清理/产物及当前worker真实START/TERMINAL身份准入通过；scope/original-data-source/case-bindings-audit.json SHA=`942e5bee46f9f009ab35a14727ed77d2aa5fac11d5ecf73919ee524fbd521ba6`，单独LiveBudget peak11385448B。该摘要**只证明原父绑定/通信闭包，不含新Q4语义verdict**；原AV/native caps=[]与新共同源运行cap须逐target显式核，不把这55父PASS或summary冒新增104绿。

py_compile/diff-check通过；冻结6文件等Git blob，scope/original-data-registry-frozen-preview/committed-input-audit.json。冻结6003a48全组合原Job/Popen禁用只读重验：**531PASS/8FAIL/467UNOBSERVED**，当前新增104全UNOBSERVED，full_registry_closed/system_delivered/expanded claim仍false。preview.stdout 506117B SHA=`3531c98f2045097eef26d3f5ca26a58e69eddcbdd373989fbc72fad51e873270`，9984.1139ms、peak process29417472/Job30658560B；同一LiveBudget reservation36585044/全程peak**48858418<50331648B**，原48MiB/单件4MiB/privateΔ128MiB/先预留/64KiB流hash/不清峰值保持。原256默认cert/daily、14优化与source工具身份不变，逐项重读复用；原8FAIL、9native具名trap/6A2X typed负例/299legacy/49ARCH及健康基线不变，不因父Case绿而核销。

原T01..15未闭；下一入口为这26源四端逐独立oracle的严格语义/诊断读取与实际Rust/C正源认证，并继续其余原源/资源/事实/恢复/原规模、ARCH/legacy。当前55原始帧已可供原义务严格适配，4个方法负例的AV新义务不能以原native单腿替代。938/939尚未D，最终review/merge仍待，无新增用户决定或存活验证进程，授权持续。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:6003a4817b6bfc10d58d2f7f09817cb6e050f6af | task_ids:T01/03/04/08/10/11（未关闭） | evidence:26漏源104目标登记；165治理+55原Case正式绿；原55通信绑定核验；整体531PASS8FAIL467UNOBSERVED | blockers:原完整范围/机制与具名观测/健康边界未闭，无新增用户决定 | next_responsible_role:work 26源四端严格语义/认证与机制/ARCH/legacy

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:35 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原完整任务未完，26原data源已补登记/55原入口重放） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:6003a4817b6bfc10d58d2f7f09817cb6e050f6af | next:work 原data源严格四端语义/认证及其余完整范围

### 2026-11-06 原18个method/record/text合法与陷阱源的双端严格观察（goal第36轮）

前轮26漏源104目标登记/55原入口为progress。本轮继续T03/04/08/10/11，941唯一树提交 **b32de0ff6e48346237d15516ed21770c6ae08951**，10文件，树clean；仍executing2/17，生产worker/语言/降级源未改。

- 对26原源中的**全部17值+1 TextIndex陷阱源**建立永久显式固定组/Case，原源码直接引用，按冻结record/text/text-record与共同caps（record=[]、text/text-record=ac.runtime.text.v1）重新真实执行AV/native，**36/36父PASS**。这与前轮原cap=[]父证据分开，不把cap不同的旧帧当同身份复用。`cargo t case-ace-m1-original-data-q4-941 --out <scope>/original-data-q4-source/gate` exit0、24.8359746s，results SHA=`b456907da5be437ec6fe9294b7b1d1ce86094aa193e7217d14ddc01786ae42a5`；完整wire环境显式开启、scope单一。另8诊断源/32目标仍required，不伪造非法源码exe，不把只运行18源说成26源通过。
- 新永久便携/2合同81161B SHA=`e881a1f80c35cf9a36be29535abde1470085dbce0416656a712b658862259d82`；原26源表、原126/四补源及1006登记键不改，新增为第21合同指针。专属只读adapter验证全部36inventory/CaseCopy完整清理与产物、原正式group/执行数、三实际prepared工具和library、11依赖pin、每源profile/caps/独立值或具名trap、原START/TERMINAL/nonce/FNV/闭合response、AV canonical值与拥有ledger/native真实link/run与释放事件；原父PASS只保parent_status，不供语义权威。生产11依赖从**既存且真实早于本轮的**record-text-q4-source/dependencies-before.json（SHA=`022f065bb59be18d3b71b1636f1cc90f5fef46aeb836c99368d2e68d32df41ed`，13:06记录）到本轮15:04后快照全部稳定，after SHA=`7036da96920d4f937c27bfb187e8a89127adee9a206327744c24fadbed9f3a0b`；显式复用旧before，不回填假执行前时间/哈希。
- 严格首读**35PASS/1FAIL**：本包新写的T08 native辅助事件oracle错误地要求owned LIT/free。原源码是`let s str = "中"`，按source-text借用表示、hir_adapter.rs StrLit处“borrowing context immortal static view”的实际分派及text-runtime `ac_rt_own_from_lit`/`rt_fail`责任，**静态borrow不分配/不free，仅越界失败RT-FAIL code=-10**。据独立源码/合同纠正本包辅助事件预期为精确单行，不改原TextIndex oracle/源/值/生产实现，也不放宽接受任意事件。首读保scope/original-data-q4-source/first-oracle-observations.json SHA=`806e30ef5ae18258139079ea7adc1fe8d9a6a832431db284433b10c432a88134`；同一原始帧修正后严格**36PASS**，strict-observations.json SHA=`760734fca31811d9fbc25bfefc688d757f94d4c56372468f4ebc6c709d966b51`。T08的具名runtime失败与原9个只generic70的scalar trap分开，绝不互相顶替。
- **217原帧反控**：36原合法PASS、36同错值/错具名trap FAIL、144 nonce/未知field/caps/source变更且重hash仍REFUSED，另1T08加完整多余owned alloc/free即FAIL；scope/original-data-q4-source/raw-reader-controls.json SHA=`202d3546bd34a69a70be6d8ebaef40ee08687a2188ab0fbcef6dd16794e8074a`，counterfactual_not_execution。新增永久静态借用trap回归保精确具名事件正控，缺事件/错trap/额外alloc-free均FAIL；非模板镜像计数测试。`cargo t infra-ace-m1-arch --out <scope>/original-data-q4-source/final-controls` **166/166 PASS**、5.3907877s、exit0，results SHA=`3ae5a6747c7630163be23323b2afd14d7a3892ccaf304ef405dfef56b311ac64`。py_compile/diff-check通过，冻结10变更文件等Git blob留scope/original-data-source-frozen-preview/committed-input-audit.json。生产身份未变旧256默认cert/daily、14优化与各source严格重读复用，未重复原1779压力冒最终职责并集。

冻结b32de0f全组合原Job/Popen禁用只读重验：**567PASS/8FAIL/431UNOBSERVED**，1006登记/21合同指针，完整范围/系统交付/expanded claim仍false。scope/original-data-source-frozen-preview/preview.stdout 523273B SHA=`f1d85489276d948f09d2605d0ff3b73e959fadd6287ea3093bb4a5ddfd823350`，11065.3995ms、peak process29392896/Job30633984B；同一LiveBudget reservation37131637/全程peak**49397208<50331648B**，原48MiB/单件4MiB/privateΔ128MiB/先预留解码/64KiB流hash/不清峰值保持，新源域512KiB有界摘要。原8FAIL、9scalar native具名trap缺口、6A2X typed负例、299legacy/49ARCH与健康基线保留。新26源剩**36 Rust/C正源认证+32四端诊断**共68格均UNOBSERVED，不因本轮36绿核销。

原T01..15未闭；下一入口为18正源真实完整Rust/C独立参考认证及8诊断源的typed消费，随后仍有其余源/机制/事实/恢复/资源/正式规模、ARCH/legacy。938/939尚未D，最终独立review/merge仍待；无新增用户决定或存活验证进程，授权持续。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:b32de0ff6e48346237d15516ed21770c6ae08951 | task_ids:T03/04/08/10/11（未关闭） | evidence:18原正源36新caps实际AV/native严格绿；217反控；166回归；整体567PASS8FAIL431UNOBSERVED | blockers:原完整范围/机制与具名观测/健康边界未闭，无新增用户决定 | next_responsible_role:work 原18源Rust/C认证及8源诊断与机制/ARCH/legacy

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:36 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原完整任务未完，18原data正源36双端观察已补） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:b32de0ff6e48346237d15516ed21770c6ae08951 | next:work 原data源Rust/C独立认证及诊断/其余完整范围

### F-941-05（2026-11-06，已授权941新增修复phase）：Rust record值根字段写缺mut

本轮原18正源×Rust/C真实认证已采完：**33/36 PASS，M01/R01/R02的Rust三项实际命令exit2，未生成有效证书**；不能据C/AV/native通过填Rust PASS。原source/hash/profile/caps/oracle不改，三个失败完整产物与raw命令在scope/original-data-certifications/{original-method-881--M01,original-record-text--R01,original-record-text--R02}/attempt-1/rust*。三个C同源认证均PASS；前轮三个AV/native六项严格PASS。另三份独立Rust手写参考各实际编译/运行正确（0/19/9，独立isolation非source认证），scope/original-data-certifications/reference-isolation/results.json SHA=`242674a28954e14ec9c52daac787556003798e53f9af2f113f16a27364380613`。原actual的generated.rs分别把var q/p发射成let无mut，随后写`.f_left`/`.f_inner.f_value`，rustc E0594拒；完整daily/cert文件及producer均保留，不手改产物。

已定位 `experimental/ace/crates/a2x-rust/src/lower.rs` 的 `collect_assignees`（当前1720附近）：只收whole-local Assign，显式排除FieldPath，误认为字段写总是经引用；普通record值局部本身仍需mut。已有mut参数是&mut引用，不能简单给所有字段根/参数无差别加mut而引unused_mut或错误receiver/mode变化。

**最小修复提案**：由原Rust后端负责，将字段写根是否需要可变绑定按真实值根/引用参数模式区分；覆盖浅/嵌套字段写、分支/循环中的写、copy/source快照、mut/view/move参数与receiver，不改source/Checked HIR/C/资源计划，不全局加mut/改golden绕失败。永久补三原源完整actual+手写reference默认与原要求优化模式的真实编译运行回归及现有方法/参数负控，按实际触面跑ace-a2x正式职责组，独立review后合入/同步941，重建相关worker并按身份刷新认证。§5.4默认已归档能力新缺陷独立修复；先前F-941-04内phase例外不自动授权本项。用户可明确选择941新增phase或独立计划；当前不修改后端。941其余reader/33成功认证及剩余诊断/机制可继续，非全局停工。

### 2026-11-06 原18个data源独立认证与F-941-05真实失败（goal第37轮）

前轮36 AV/native严格正证据为progress。本轮继续T04/05/08/10/11，唯一941树提交 **000107a603f084897c38db2818c72288afc4ffa4**，42文件，树clean；仍r2 executing2/17，生产编译器/worker未改。F-941-05最小修复提案已单独写入上条§9并向用户请求941新phase或独立修复裁定，当前无授权答复，不擅用F-941-04例外；其他验收可继续，不是全局waiting。

- 为全部**18正源**永久提供36份独立手写Rust/C参考与冻结合同，原profile/caps/source/oracle保持。参考按原语义实现record copy/嵌套字段、方法与mut/view参数写回、命名参数、文本标量与字节长、clone/move值、递归和TextIndex检查；不读actual产物/结果，不打印预置正确值代运算。Rust用标准String/struct/char操作，C固定数组独立建模有限文本值，不冒资源生命周期证明（后者另门）。新original-data-reference-contracts.tsv SHA=`92cbb23d49bc7ed9cb108055dc696c88b6b8f2a7b53e2536888d9972fdde17aa`；原默认Rust edition2021/C11 /O2等参数与全文件集保，未擅增0优化替代原14优化配置。
- 原显式受管capture对36个目标逐identity/daily/cert实际调用：**33个完整actual+reference认证及其daily PASS**（Rust15+C18），**66compile+66run**获完整终态；另外M01/R01/R02三个Rust实际cert命令exit2、无有效证书，collector总断言33/36而exit1，保留首次红，不能说36认证全过。三失败完整actual四文件与预冻结daily逐hash相等，无人手改generated.rs；完整stdout错误/detail仅作故障定位，不解析为typed语义证据。scope/original-data-certifications/failure-audit.json SHA=`5a106fc7f03524c3cd4a05998ed4ec7efa97184757b76884c95cee6e527e4648`。三独立Rust参考另隔离3compile+3run通过0/19/9（非actual证书），证据与原因见F-941-05提案。
- 33成功项逐原完整actual/独立reference、两腿各对冻结oracle、原toolchain/version/flags、nonce/FNV/证书与manifest/完整依赖、四phase终态、daily spawn0及现场四文件相等严格重读PASS。原360s whole/120s compile/30s run/每流1MiB及Job保；成功cert父墙609.3993..3846.3254ms，不冒正式P90。frozen-bindings.json SHA=`93db215713a9dccb288e8df1fb461450a4f0b24da14230521737e0705b7ac935`，strict-observations.json SHA=`032f5a64151d681b263306bc8ab300145936f175e7cd73fe621ea4bc6168d701`。reader从18源合同独立构成全部**36必需证书槽**；33提交项准入，缺M01/R01/R02 Rust有效证书三格保UNOBSERVED/具名F-941-05，missing_certification_slots显式列出、certification_complete=false、整体daily=UNOBSERVED。没有拿未完成命令的untyped错误冒语言诊断或伪造证书，也没丢三格换较小分母。
- **99原认证帧反控**：33合法PASS、33双方同错值/错具名trap FAIL、33重hash未知field REFUSED；scope/original-data-cert-raw-controls/results.json SHA=`3a867c1fdcc4b79dcec72a7a8c61d0712004a66267aadc1e33659aa3d43be98b`，counterfactual_not_execution。新永久缺证书控制保当前三缺格、空提交仍全部36 UNOBSERVED、外来target/未知verdict拒，合成覆盖控制不入实际证据。`cargo t infra-ace-m1-arch --out <scope>/original-data-certifications/final-controls` **167/167 PASS**、5.5889070s、exit0，results SHA=`23fee9661d694939bbd751b3969bb7b53c20728f0bf66ffa9708c6795d00f245`；py_compile/diff-check通过。冻结42文件等Git blob留scope/original-data-cert-frozen-preview/committed-input-audit.json。

**HEALTH-941-DATA-WARN**：66成功compile流中，全部33reference与全部C流空；8个Rust actual流共**18条warning**（17条既有大小写hygiene命名non_snake_case，M08源含uncalled_shape产生1条dead_code）。当前生产打印器未改，原源保真，不加allow/剪函数静默消声；此健康项归Rust发射owner/T13，与旧collections/RT健康边界合并审查，尚未豁免/关闭。scope/original-data-certifications/compile-health-audit.json SHA=`a99d068cda4a8d420568c82ea0f2546af8f7abbd43e2d20c63ba35363692efe9`。三失败actual的编译错误与此warning汇总分开，失败不能充成功流。

冻结000107a全组合原Job/Popen禁用只读重验：**600PASS/8FAIL/398UNOBSERVED**，1006登记/22合同指针；原126/四补源/26data源、旧313/NA/38cert-gap与预算合同保持，完整范围/系统交付/expanded claim仍false。preview.stdout 549925B SHA=`65861415349741e486d0ad23dab7675758412e4750e94b40b70e6b69c8415a67`，12729.3871ms、peak process30453760/Job31682560B；同一LiveBudget reservation37666989/全程peak**49926272<50331648B**，原48MiB/单件4MiB/privateΔ128MiB/先预留/64KiB流hash/不清峰值不变，新认证域512KiB摘要。已完成默认cert/daily从256→**289**，额外14 Rust模式不变；生产身份未变旧证据逐项严格重读复用。原8FAIL、9generic native trap、6A2X typed负例、299legacy/49ARCH保；新增data域仍**3缺Rust证书+32四端诊断**共35格UNKNOWN，F-941-05真实失败具名，未被33绿掩盖。

原T01..15未闭；下一步可继续8诊断源与其余机制/事实/恢复/资源/原规模、ARCH/legacy的独立证据。F-941-05修复等待用户归属裁定，938/939尚未D；最终review/merge仍待，无存活采集进程，941授权持续。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:000107a603f084897c38db2818c72288afc4ffa4 | task_ids:T04/05/08/10/11（未关闭） | evidence:33完整cert+daily/66compile66run；3Rust失败F-941-05+独立参考隔离；99反控；167回归；整体600PASS8FAIL398UNOBSERVED | blockers:F-941-05修复归属待裁定，原完整范围/机制/健康未闭；独立工作仍可继续 | next_responsible_role:work 8源诊断与其余机制；F-941-05等待裁定后另行修复

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:37 | from:work | to:work | status:executing2/17 | outcome:needs_fix（原完整任务未完，33原data认证已补、3Rust真失败具名） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:000107a603f084897c38db2818c72288afc4ffa4 | next:work 原data诊断/独立机制证据；F-941-05归属待用户裁定

### 2026-11-06 原8个data负源诊断、活数据摘要与F-941-05授权（goal第38轮）

前轮33独立cert为progress。本轮继续T02/04/08/10/11；唯一941树先提交1b664056c、后补摘要预留边界，冻结 **a236a44d4ad6ee7595d601bcef6f01af56d9570b**，合计11文件，树clean；仍r2 executing2/17，生产语义/worker未改。期间用户明确授权F-941-05在941新增phase；已更新§4/§10，下一stage修订r3合同，不再等待裁定。

- 原26表中**全部8诊断负源×AV/native=16真实调用**，原字节/profile/caps/诊断族保，native显式source-v1。独立stage/code由旧方法/资源负控和实际resolver/typecheck/verifier责任核定，不从本次rendered文本反解。首轮formal16中11PASS/5FAIL（4个AV缺code+本包新Case把R09原source.type-mismatch误列resolve）；沿source/typecheck.rs mismatch明确Stage::Verify，纠正该Case映射typecheck，未改源码或错误族。新冻结门 **12父PASS/4父FAIL**，exit1，scope/original-data-diagnostics/frozen-gate/results.json SHA=`38091e3e8614737e62f86d764df7a609cd499a484a86a325f1bc9b2d9262d682`，9.0365666s；不冒全绿。首门SHA=`100ae41050322c19eef9fcb6c34b24134b707af8b9eabea2ec81c9fe2d222382`/6.1277654s保留。
- 新永久/2合同38039B SHA=`fd5ab92f07e0dee67faa89d2e48d35e811b8da2011cef2ea54c0ec52cc3e5941`，第23合同指针，1006登记/原源表不变。严格adapter逐16原inventory/check、CaseCopy/清理、当前prepared AV/native与真实wire workerSHA、既存早于调用的11生产依赖before及当前after稳定、nonce/FNV/源码SHA/共同caps/请求选项/完整response闭包验证。native保原911事实全部字段：实际profile/attempted升级链与surface拒绝、UTF8字节长/causes顺序/原primary0、cold operation来源、legacy投影一致、64KiB/64 causes/单message4KiB原帽及无执行结果互斥；不虚构codegen计量。原SQ/instances默认诊断分支全部保，只有限启用data_family新分支。
- 当前严格 **11PASS/4FAIL/1UNOBSERVED**，strict-observations.json SHA=`531487a5aa3052beee246f26ea5c103baf328c16ff6f09fcb48f8c42047a98f8`。**GAP-941-AV-DATA-DIAG-01**：M09/M10/M11/R09的AV record worker现行data错误分支（av-worker.rs约2497）显式code=""、省typed/trace，只给rendered描述。reader按该有限真实shape保原始文本但诊断权威UNOBSERVED；原Case FAIL继续传递为最终FAIL，**不填code/不从message解析通过/不把缺字段补零**。owner=AV data worker协议/原方法线，关闭需真实结构化code与完整父复核；本项未获F-941-05修复授权，不擅改worker。
- **GAP-941-DATA-BLOCK-SPAN-01**：native T06具名verify.borrow-escape与源码/操作事实有效，但独立精确block extent尚未核实，原raw span19..62保留。初版本包错误用大括号区间，后核adapter为live statement join，仍有closing-boundary差异；不能凭实际返回值调整expected使绿。首reader 11P5F记录保first-reader-observations.json SHA=`4a49bfd809d7d935086cd1aaf46c20f754f4f66c2d1f10c4f9822b86d3b17707`；现该格明确UNOBSERVED，owner=941 T04/原source span合同核查，待独立映射证据后关闭。其他7 native精确独立源码anchor（get/assignment/call/clone/错实参/移动后根/Unicode escape）及4 owned AV code真实PASS。
- **156原帧反控**：16原帧低层11PASS/5UNOBSERVED（四AV缺码在父层保FAIL）；双方错code、错stage与其他7 native错span FAIL，nonce/cap/source/profile/operation/primary布尔/缺事实/未知字段等REFUSED。T06 span未核与AV只有匹配message均不升PASS。scope/original-data-diagnostics/raw-reader-controls.json SHA=`be7a5382c6908ed9aba9b8c517a401dd81124a888b3935152ba0bcbbf596086c`，counterfactual_not_execution。新增2永久控制保AV message不授权code、原父FAIL不抹去、native profile/operation/primary/源码anchor与T06未核不冒PASS。

T02活数据修正：_compact_admission仍**在reader前预留原固定上限**，返回并丢弃临时输入后，用既有同一保守公式`7*UTF8 bytes+256*节点+4096`保留摘要；只释放未使用的padding及已死临时量，绝不清peak/另建预算域/提高48MiB/截证据。原先仅判实际摘要小于固定预留却一直保留全部padding，多域累积临近帽；现两个同时活摘要分别保留实界，32MiB临时+已有8MiB的控制峰值仍40.5MiB，40MiB超额及无界返回继续先拒/不释放。诊断完整摘要288389B，首冻结组合因256KiB预留不足而正确拒（原stderr/command在original-data-diagnostics-frozen-preview保留）；本域预留改512KiB覆盖真实上界后重验，原全局48MiB不变。`cargo t infra-ace-m1-arch --out <scope>/original-data-diagnostics/bounded-summary-controls` **169/169 PASS**、9.1415062s、exit0，results SHA=`73eb08a2d278b5f5d73616828d07c070b691409be8257c6277ab5b5b504b1f00`；py_compile/diff-check通过。两提交11文件等HEAD Git blob，combined-committed-input-audit.json SHA=`6f7e8953e20769de8bd5925350159e0de894f698cdd18163dfb36bf8b7166456`。

冻结a236a44全组合原Job/Popen禁用只读重验：**611PASS/12FAIL/383UNOBSERVED**，1006登记/23合同指针，完整范围/系统交付/expanded claim仍false。原990非本轮诊断格位verdict全部保持；新16格仅15变化（11P+4F，T06维持U），prior-verdict-preservation.json SHA=`d80d499e4ba296c8e11818e97e4c60b99e88b7e660a35a9a3f77619f5b0d90f7`。scope/original-data-diagnostics-bounded-preview/preview.stdout 563414B SHA=`af3eb670dfeca6ac9ca793934f602d7adca17e17ed321202222ab928beef39db`，14650.0534ms、peak process29990912/Job31223808B；同一LiveBudget reservation**26374947**/全程peak**46836339<50331648B**，实际峰值保留，原单件4MiB/privateΔ128MiB/先预留/64KiB流hash保持。原289默认cert/daily及14优化身份不变严格重读复用；原8FAIL/9native trap/6A2X typed负例/299legacy/49ARCH/健康及F-941-05三Rust缺证书均保。

原T01..15未闭。下一stage **auto-plan-new**按已授权F-941-05最小提案修订941 r3（同一树，新增修复与实际认证phase），再work修复；不再重问授权，不将新AV协议/未核span纳入Rust修复。其他全范围/机制与938/939未D、最终独立review/merge仍待，无存活采集进程。

stage: work | plan_id:PLAN-941 | plan_revision:r2 | outcome:needs_replan（用户已授权941新增F-941-05 phase，需落实r3执行合同） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:a236a44d4ad6ee7595d601bcef6f01af56d9570b | task_ids:T02/04/08/10/11（未关闭） | evidence:16真实负入口/严格11P4F1U；156反控；169回归；共享预算padding释放不清峰值；整体611P12F383U | blockers:原完整任务未闭；F-941-05已授权可推进 | next_responsible_role:new 已授权F-941-05 r3同树phase

stage: goal | plan_id:PLAN-941 | plan_revision:r2 | round:38 | from:work | to:new | status:executing2/17 | outcome:needs_replan（F-941-05新增phase获批，诊断证据已收束） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:a236a44d4ad6ee7595d601bcef6f01af56d9570b | next:new F-941-05 r3合同，随后work

### 2026-11-06 r3执行合同：已授权F-941-05同树修复（goal第39轮）

前轮真实诊断/预算修正是progress；本轮按auto-plan-new修订合同，**只改主检出Plan，不实施代码、不跑新语言测试**。入口唯一941树a236a44d4ad6ee7595d601bcef6f01af56d9570b clean，主检出无code WIP（其他文档WIP保留）。用户明确授权941新增F-941-05 phase，故r2→**r3**，status仍executing、current_step=2、total_steps=19；T16/17完成保留，原T01..15与全部门/分母/预算不撤。

按当前Spec及lower/print/verify实证补§5.9：普通record值根字段写需Rust mut，mut引用参数字段写不需重绑mut，view写仍拒；text-record MoveOwn值根沿已有许可/BodyResourcePlan。旧provider明确将a2x-print/2纳入AXR键，故修复必须用原打印版本轴隔离旧错误片段，而非删store/旧产物改key；此为该producer修复的既有身份/缓存义务，不新增缓存平台。静态pin已写入§5.9，现record/text Spec仍承诺字段写/Copy快照，生产collect_assignees却排除全部FieldPath，三实际失败及反证仍在§9，未用代码现状改写规范。

新增T18/19、AC15、V22..24、SD08/09；SD元数据加入auto-a2r record/text与auto-a2x incremental/cache两个真实canonical目标，未写Specs/索引。新测试路径标拟新增，现ace-a2x确实覆盖4个A2X包的lib/tests/conformance，infra-ace-m1-arch亦存在；原helper/contained executor与360/120/30s/1MiB/48MiB帽保持。292为需新身份闭合的默认成功目标数（当前289+原3失败），原14优化模式另列，不把新增-O回归当原额外模式完成。F-941-05授权不扩大至AV缺code、T06 span、C或shared HIR/前端修复。

更新当前合同的T14/T12依赖、已完成r2验证说明及当前review绑定r3，历史收据原样保。无最终review记录被冒更新，既往代码证据按真实identity复用/STALE规则办理。静态核对0..10节、T01..19唯一、完成仍2、AC01..15/V01..24/SD01..09覆盖、新目标文件/现工具组存在；实施源树仍clean，工作下一入口明确T18。

stage: new | plan_id:PLAN-941 | plan_revision:r3 | outcome:pass（合同与现授权就绪，非实现/验收通过） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:a236a44d4ad6ee7595d601bcef6f01af56d9570b | changed:T18/19,AC15,V22..24,SD08/09及依赖/metadata | blockers:无新增决策阻T18；原全范围与具名red仍开放 | next:work T18 Rust值根字段写/旧打印缓存身份修复

stage: goal | plan_id:PLAN-941 | plan_revision:r3 | round:39 | from:new | to:work | status:executing2/19 | outcome:pass（已授权phase合同落实） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:a236a44d4ad6ee7595d601bcef6f01af56d9570b | next:work T18，后T19再回原完整验收

### 2026-11-06 F-941-05值根修复与真实编译回归（goal第40轮）

按r3授权在唯一941树完成T18，冻结提交 **c4cb9dab71a45c72c3daab50672cec0ae1035298**，4文件156新增/6删除，树clean。collect_assignees对FieldPath区分普通值根/MoveOwn值参数与已有借用引用参数：前者加入Rust mut，后者不多加绑定mut。原whole-local/BorrowMut/资源释放判定保留；前端/HIR/verifier/C/AV/native生产模块未改。Rust原print版本轴从a2x-print/2升/3，旧缓存迁移真实见证属于T19，尚未宣称完成。

永久conformance::record_field_writes_941注册7项。先在旧生产实现实跑原M01/R01/R02：**0/3 PASS，3个真实E0594**，scope/f94105-red保留。修复首轮6/7 PASS，新增循环fixture误用profile不支持的while而失败，保留scope/f94105-field-write-green；只把该新fixture改为文档既有条件for，原3源与独立reference不改。随后f94105-field-write-frozen被可用磁盘不足2GiB提前拒，未执行测试，不能充FAIL/PASS。

磁盘不足期间已核941闲置incremental缓存5,976,290,503B、28969文件、无链接/活动进程；精确删除被自动审批整条拒绝（仅blocked by policy），没有执行/换工具重试，向用户发出手动清理请求。恢复读到D盘可用31,509,565,440B，原incremental目录仍存在；释放来源未核，不声称agent删除或用户已清理。门禁已解除，该缓存无需再删。

当前修正后的正式命令 `AUTO_PLAN_ARTIFACTS=<scope> cargo t ace-a2x --test-name record_field_writes_941:: --out <scope>/f94105-field-write-verified` **7/7 PASS、exit0**，总6.4772133s，准备3.6495422s、执行2.7911345s。6个正例各默认与-O、完整4文件actual与独立Rust reference分别编译/运行，合计**24 compile+24 run**，每个rustc额外-D unused_mut；期望原3例0/19/9、分支循环4、mut借用124、MoveOwn42，实际均匹配、guest stderr空。第7项真实共享verifier拒view字段写及同根alias，保immutable-write/mut-conflict。新mut冗余警告不允许；原first_lex_fact dead_code属于既存健康边界，未静默豁免。命名选择7项，其余组未作为本轮全量执行；scope原始完整receipts保留。results SHA256=`8a8962a015c5ffa72dfcb6211382480ba702f04f0e921d063cd78e1baaef7e1f`。rustfmt及git diff --check通过；本轮未跑完整ace-a2x/治理并集，留T19冻结验证。

**T18完成，current_step3/19**；T19、原T01..15继续开放，F-941-05真实worker证书闭环前仍未关闭。289默认成功目标与3缺Rust证书需当前worker新身份刷新至292，原14优化配置单列不由新增-O回归抵账；旧原始证据/已知red/完整分母/预算不改。不进入review/merge。

用户随后询问941材料占100G能否清理：当前只读全遍历实测scope **1,480,204,329B/62599文件**（约1.38GiB）、0链接，无941活动进程；compiler-temp248,966,245B。尚未执行材料删除，继续调查其所见占用来源；在途证据受保护。

stage: work | plan_id:PLAN-941 | plan_revision:r3 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:c4cb9dab71a45c72c3daab50672cec0ae1035298 | task_ids:T18已完成,T19待执行 | evidence:旧3例E0594红/当前7回归24compile24run通过；print/3身份 | blockers:无磁盘阻塞；原验收/具名red继续开放 | next_responsible_role:work T19旧缓存见证与实际worker新认证

stage: goal | plan_id:PLAN-941 | plan_revision:r3 | round:40 | from:work | to:work | status:executing3/19 | outcome:needs_fix（T18完成，T19及原完整任务仍待） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:c4cb9dab71a45c72c3daab50672cec0ae1035298 | next:work T19

### 2026-11-06 T19缓存迁移与当前工具门禁（goal第41轮，在途）

第40轮T18完成为progress。本轮继续T19，同一941树新增永久缓存回归提交 **62f57b4e2**；文档候选提交 **8c074c17c404b15a1d042a8bf3a77420b958997a**，当前树clean。实现代码仍c4cb9dab7的生产修复，测试新增三代真实store：受控旧provider沿原print/2键保存R01缺mut片段，旧完整产物真实rustc E0594；当前provider打开同一store实际lower=1、编译运行value:19；第三次打开lower=0、全部装配文件hash恒等、再次实际value:19。新回归受原120/30/360s、1MiB capture上限，scope/field-cache-941-*保完整receipt；单例正式门1/1 PASS，5.994877s，SHA=06c76a91bb7cb4c99d9bd89c39333e60fe9c36885a49245a053a1c5cbc15e235。

冻结62f57b4e2运行完整`cargo t ace-a2x infra-ace-m1-arch --out <scope>/f94105-frozen-union`，210+169 PASS，265.912002s，exit0；发现部分原conformance显式使用旧release预构建worker，该轮作为准备/历史证据保留，未冒当前worker完整门。随后实际`cargo build --release -p ace-a2x-worker --bin a2x-worker` exit0、55.91s，scope/f94105-release-build.log；新worker SHA **521d08210c6a03e0f1967285ce88b7ad0d67216447ba9241827b1f711344a0ea**。在新release身份下补正式完整并集 **210+169 PASS**、exit0、260.967499s，scope/f94105-current-worker-union/results.json SHA=ca3b4f7492bdc28bff830fefe093302897f11e589e37b77f9abbaff253d08f37。保原ignored/条件，不增过滤；baseline first_lex_fact dead_code未由本次引入，健康义务继续保。

独立提前冻结新debugworker/conformance/test source及原输入，`cargo t ace-a2x --test-name release_941::original_ --out <scope>/f94105-release-execution/gate` **3/3 test PASS**，原8 scalar+4 RT+2 query共14配置实际default-independent-reference/优化actual编译运行，flags原样（不拿新回归-O冲抵）；总8.800435s，SHA=e43602e97c117b90f281416d95e9a4b377c9809a4de29b121d5717e98bed32e3。三域before/after逐pin稳定，现新index与正式parent/discovery均已绑定在f94105-release-execution/{scalar,record-text,query}/bindings.json，最终严格reader与当前默认cert联读仍待。

SD08/09候选在同一树：明确值根/借用模式与旧print轴失效；同一record/text文档仍写F04已解除的条件尾部限制，按既有F04合同改为共享前门链接，未扩大源语法。文档改动不改本轮冻结生产/测试字节，尚未main沉淀/最终review。

按原当前绑定固定**292唯一默认目标=289已完成+原M01/R01/R02缺Rust3**，scope/f94105-default-refresh-targets.json。原旧R01 C证书在新工具下被原严格reader明确STALE（未解析message冒通过），scope/f94105-old-worker-certificate-rejection.json。首次额外探针脚本错误把STALE文案断言为hash/sha字样而自身失败，随后仅修探针断言为原AdmissionError的精确STALE，不改reader/旧证书。

刷新脚本首批使用了自行安排的编号目录，原text严格reader按既有原case路径拒绝；这是本包采集布局错误，非producer新缺陷。已精确停止本包Python PID25736，工具session67890 terminal exit1，旧scope/f94105-current-certifications所有原始命令保留，不迁移/改写原证书路径，不用改reader绕过。重采集脚本refresh-f94105-layout-correct-certifications.py复用原case/family/attempt相对层级，新scope/f94105-layout-correct-certifications；原新stdout/command/身份/daily/证书逐项保存。其已完成text **20/20**并原strict读取certification_complete=true，early-strict-observations.json，peak2239137B。当前292批次**仍在运行**，明确handle **exec session17398 / Python PID46428**；下轮先核实并poll同handle，禁止因观察过期重启或覆写该批。现producer固定62f57b4e2；随后8c074c17c只改2文档，生产/测试身份未变。T19未勾选，不能将尚在采集项算PASS。

独立身份对账scope/f94105-unaffected-source-identity-audit.json：text/RT/collections/original-data positive及diagnostic的before/after全部原pin当前匹配，四AV/native/syntax/bridge worker字节保持；**query_instances_execution的13 pins中仅a2x-worker两快照失配**。这是预期身份失效，下一步必须新快照+真实原source-query/source-instances固定组重放及6原typed负入口刷新，再读全组合，不修改旧before或将其复用为新执行。原6typed诊断缺code、其他原red/unknown/全分母/预算保持；T19全组合闭环/F-941-05关闭尚未完成。

用户明确回复已清理指定incremental缓存；当前该目录存在但仅251文件/37,682,894B（构建可重建），D盘可用128,779,685,888B；不再有磁盘阻塞。上轮材料全盘调查确认plan-941约1.38GiB、主要大户plan-942约156.6GiB，未删除其他计划材料。原清理自动拒绝不再是当前待用户事项。

stage: work | plan_id:PLAN-941 | plan_revision:r3 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:8c074c17c404b15a1d042a8bf3a77420b958997a | task_ids:T19进行中 | evidence:三代旧缓存/当前新worker210+169门/原14优化通过；292当前认证执行中 | blockers:无新增用户决定；旧完整任务与具名red未闭 | next_responsible_role:work 先poll session17398/PID46428，再当前查询证据/严格组合

stage: goal | plan_id:PLAN-941 | plan_revision:r3 | round:41 | from:work | to:work | status:executing3/19 | outcome:needs_fix（当前认证执行中，T19未闭） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:8c074c17c404b15a1d042a8bf3a77420b958997a | next:work T19续现存session17398，禁止重启

### 2026-11-06 当前认证全执行与正式准备身份切换（goal第42轮，在途）

前轮新增缓存/门禁/14优化是progress；本轮先核实session17398/PID46428确实存活，继续原批不重启。该批随后真实terminal **exit0**：scope/f94105-layout-correct-certifications完成**292默认cert+292 daily**（text20、CT14、scalar122、RT32、collections40、query18、supplement8、original-data36、first2）。原缺失M01/R01/R02 Rust三个完整actual+独立reference均PASS。早期原strict在worker身份A=`521d0821…`仍当前时验证text20、CT14、scalar122与原8优化、RT32与原4优化均complete，raw保留；**这些只是当时有效证据，当前已STALE，不能据此勾T19**。完整采集历史scope/f94105-round42-refresh-history.json SHA=ac50030401f443949ba6b9fe9c370247d92ef53be6efda0a34d1f2fc9ec03bff。

按A身份另实跑6原source-query/instances Rust/C负入口，均真实拒绝且原strict保持6 UNOBSERVED（仍缺typed诊断/source/operation，不能解析rendered文本冒PASS），scope/f94105-query-negative-frontdoors/strict-observations.json；其worker身份现同样过期。

随后实际原正式`cargo t case-ace-source-query-934 case-ace-source-instances-935 --out <scope>/f94105-query-instances-source/gate` **14+12 PASS**、exit0、15.603245s，results SHA=68982c13df5268cdf3e20ac9133da054b820805cbb61d8f2f81e1eb28f226587。但该正式入口prepare_tools又真实编译/链接a2x-rust/worker（工具记录原`cargo build --locked --offline -p ace-a2x-worker --target-dir <own ACEtarget> --release --bin a2x-worker`，3.9436556s），canonical worker变为 **B=`7b0be842feabdd55a7217ae783ae4af8d9a445b60e7807e5cf1bda67d5b4bb7e`**；AV/native等工具hash仍原值。未证明这次重链接的更深原因，不能推测为字节等价后冒用A身份。完整reader首项text明确`STALE artifact content: .../a2x-worker.exe`，因此A批292、A负入口及以A为before的query快照都保持历史，**不改旧producer SHA/原command/前快照、不绕reader**。这是本包准备顺序失当；已告知用户，未以292父PASS掩盖当前身份不合。

纠正顺序：先以正式工具准备后的B冻结13依赖及原26 inventory，scope/f94105-query-stable-source/dependencies-before.json；再在完全同一冻结生产/测试8c074c17c树运行**完整并集** `cargo t ace-a2x infra-ace-m1-arch case-ace-source-query-934 case-ace-source-instances-935 --out <scope>/f94105-stable-final-union`。目前明确live **exec session52277 / nextest PID18668**，未终止/未重启；下轮先poll本handle，终态后执行bind-f94105-query-stable-source.py核所有before/after（含B）一致并真实strict26格。若B变化需先查明稳定准备面，不能盲目再采集。原14优化debugworker/conformance绑定保留，按实际hash再核，生产/测试代码本轮零改。

已准备而**尚未执行**的后续：refresh-f94105-stable-certifications.py明确要求B身份，独立新scope/f94105-stable-certifications，保持292原目标/原布局/原flags/原360/120/30s与1MiB帽；正式所有工具准备与B稳定确认之后才启动，采集之后不再运行会改worker的准备入口。read-f94105-stable-certifications.py、capture-f94105-query-stable-negative-frontdoors.py、prepare-f94105-stable-composed-preview.py均已做语法核查，当前没有第二批cert后台进程。全严格联读/组合/T19/F05关闭继续pending，original whole611P12F383U仍仅旧基线，不冒当前完成。

独立T04证据推进：查明T06 block span end62而非63的来源，**旧legacy parser与现CST投影都把.to(str)表达式止于type-word末尾，排除右括号**；adapter按首终结语句之前的live statement join，text verifier borrow-escape取此HIR block span。独立源字节let起19、typeword末尾62，四责任文件SHA及行节选在scope/t06-independent-span-contract-investigation.json SHA=cc2a1dd7818408ea290635663262512cf066b78701aee0efe37c54755c87f667。此轮只调查，reader的T06 UNOBSERVED/具名gap未改；后续须永久独立anchor和错span反控完成后才能关闭，不能凭本次raw值调expected。原其他red/unknown、全部分母和预算不动。

当前8c074c17c404b15a1d042a8bf3a77420b958997a clean；主检出没有code WIP，本轮只写941计划簿记与统一scope临时材料。current_step仍3/19，无新用户决定/磁盘阻塞；继续现存正式并集，不进入review/merge。

stage: work | plan_id:PLAN-941 | plan_revision:r3 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:8c074c17c404b15a1d042a8bf3a77420b958997a | task_ids:T19进行中,T04 span合同调查 | evidence:292真实采集完成但A→B后STALE；26正式门通过；B冻结完整并集运行中 | blockers:无新增用户决定；需稳定B身份后重认证 | next_responsible_role:work poll session52277，不启动重复并集；确认B稳定再新292批

stage: goal | plan_id:PLAN-941 | plan_revision:r3 | round:42 | from:work | to:work | status:executing3/19 | outcome:needs_fix（认证身份刷新顺序已纠正，当前并集仍运行） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:8c074c17c404b15a1d042a8bf3a77420b958997a | next:work T19续session52277，后B严格认证

### 2026-11-06 第42轮续：B身份已稳定，重认证已启动

session52277真实terminal exit0：scope/f94105-stable-final-union完整 **210 A2X+169治理+14 query+12 instances全部PASS**，164.346036s，results SHA=53869e030815ea1a8205f16bbf7fb9e4b8c8dda497dee8886db305a593b1c8f5。正式prepare前后B worker SHA保持7b0be842feabdd55a7217ae783ae4af8d9a445b60e7807e5cf1bda67d5b4bb7e，13依赖全部稳定；原A→B重链接更深原因未断言，后续不再运行构建准备改变B。

首查询严格bind因本包漏设AUTO_CASE_WIRE_EVIDENCE而按原要求拒“required original exchange missing”；该次26父PASS不冒strict。仅补原既有采集env，正式原26完整重放至新scope/f94105-query-stable-wire-gate，**14+12 PASS**，22.201285s，results SHA=97aa86fe2ac6debbb05980af7aaf639dbe6a92599253cfdac2136ee7a57f6678。未改reader/源/预算/原raw；B仍稳定，before早于两次执行且全部当前pin一致。bind-f94105-query-stable-source-with-wire.py读新原交换，**24源义务严格入账**（原26 TestCase checks并不等于24源slot的分母），peak8560142B，bindings-wire.json/strict-observations-wire.json；旧无wire结果保留。6负入口亦以B重采至f94105-query-stable-negative-frontdoors，旧缺typed权威事实继续保未闭。

**当前唯一认证采集批**：refresh-f94105-stable-certifications.py，scope/f94105-stable-certifications，明确核B身份后已启动；live **exec session80190 / Python PID31556**。新20 text批已开始、未全完成，不启动重复进程。下轮首先poll80190及核PID；终态exit0后运行read-f94105-stable-certifications.py逐域严格联读（含原14优化），再prepare-f94105-stable-composed-preview.py与capture-f94105-stable-composed-preview.py，全组合核旧red/unknown保持、新3Rust slot关闭，才能勾T19/F05。必要negative reader采用新B bindings。旧A292只历史，不与B混用。8c074c17c树clean、生产/测试字节未再改，完整B并集证据已有效，不为文档收据/认证重复跑完整门。

stage: work | plan_id:PLAN-941 | plan_revision:r3 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:8c074c17c404b15a1d042a8bf3a77420b958997a | task_ids:T19未关闭 | evidence:B完整405项门/原query24 strict；B292当前live | next_responsible_role:work poll session80190/PID31556（取代上一条next的已结束52277），禁止重复采集/再准备worker

### 2026-11-06 F-941-06修复归属提案（原GAP-941-AV-DATA-DIAG-01，未实施）

原第38轮4真实AV record负源M09/M10/M11/R09的formal FAIL与空code原交换、156反控均已保留，AV worker当前字节未变。再次核实生产链为av-worker.rs data-run→data_driver::request_data→共享前端；worker Err分支按字符串前缀分类并固定输出diag.code=""，丢失原本可用于判定的结构化诊断。本包reader正确拒绝从rendered message反解，故仍4 FAIL；F04/F05授权不自动覆盖本项。

最小待授权范围：保留并透传data链既有共享前端诊断stage/code，沿既有driver/worker错误载体补有限结构化失败通道（保旧调用兼容/其他失败分态），禁止从message提取code或修改源/oracle；四原负例作永久真实worker反例，加合法record、旧owned/core及非前端失败边界；按真实改动执行完整所属固定组/必要消费者，AV生产身份变更后刷新其真正受影响的source证据，再严格核原16诊断与当前组合。不得将本项扩为native generic trap、A2X typed缺口或新源语义。按§5.4“已归档能力发现新缺陷要具名新修复流程”，请求用户选择941新增phase或独立修复；当前只提案，不改r3/T19/任务总数，不实施producer。B292采集及其他已授权工作继续。

### 2026-11-06 T19/F-941-05当前身份闭环（goal第43轮）

第42轮B稳定并集/真实查询是progress；本轮首先核session80190/PID31556存活且canonical B=7b0be842feabdd55a7217ae783ae4af8d9a445b60e7807e5cf1bda67d5b4bb7e保持，续原批至真实terminal **exit0**，292 cert+daily全部采集完成，原3缺Rust M01/R01/R02完整actual+独立reference均PASS。不重启/不再准备worker，8c074c17c404b15a1d042a8bf3a77420b958997a唯一树clean，生产/测试字节零改。

原strict逐域联读：text20、CT14、scalar122+原8优化、RT32+原4优化通过；首次collections严格拒daily缺mode——本包通用采集器漏写显式mode，不能因默认值相同绕精确请求合同。只重采该40项到**f94105-stable-collections-correct/collections_certifications**，每次原cert req的mode原样进入daily；session41291 exit0，40实际actual/ref重新认证+daily完成。保旧40原始命令/失败读取，不改旧证书，不重采其余252。新strict确认为default_certification_complete=true；初汇总脚本误取通用certification_complete键而KeyError，仅改本包scratch汇总到该域原有default键，未改生产reader/返回schema。

最终read-f94105-final-certifications.py原reader只读/Popen禁用：**292默认cert全部PASS、292 daily全部PASS且编译器spawn0，原14优化全部PASS**。明细252来自f94105-stable-certifications、40来自上述显式mode新scope，统一composed-bindings逐项指向真实来源；所有scope仍在唯一plan-941目录。逐292观察显式核judgement/daily_judgement=PASS、daily_compiler_spawn_zero is True，scope/f94105-stable-composed-preview/strict-default-and-daily-audit.json SHA=27a5d7d1f3e31919fe451a1ba38dde9e85ee1597c4e95101f2148384aa2560d7。原14优化的8/9/9各域source/debugworker/conformance/配置pin当前全匹配，f94105-stable-release-identity-audit.json；没有用新字段写回归的-O替代原14，也没有把集合优化/资源/生命周期UNOBSERVED充PASS。

最终B组合 read-f94105-stable-composed-preview.py经原受管capture且Popen禁用，exit0：**614PASS /12FAIL /380UNOBSERVED，1006登记/原完整分母保持**。同旧第38轮611/12/383逐格对账，**仅M01/R01/R02的ACE-A2R-HIR从UNOBSERVED→PASS，其余1003格verdict完全不变**，prior-verdict-preservation.json SHA=bc79bcf0cd1b5d594f82442be13443721e53944a85afbbb9cb15c6cc06ca7961。required_universe_closed、acceptance_system_delivered、can_claim_expanded_m1全false，旧8 scalar FAIL、4 AV data FAIL/已提F06、9 native generic trap、6 A2X typed缺口、T06未核span、299 legacy/49 ARCH及健康/完整机制仍保，不签扩大M1。

preview.stdout **565002B SHA=30a0918d94f207ae9489b8861984366f79a34bb162dbd85d01478176aef55b61**，9423.0275ms，peak process29769728/Job31010816B。共享原LiveBudget reserved**26407831**、全程peak**46869307<50331648B(48MiB)**；原单件4MiB/privateΔ128MiB/64KiB流hash/先预留及不清peak均未变。无临时放宽、截日志或新预算域。组合6个已提交文件审计SHA=3c0ddd60f5aa0abf9e63914600f5fdeecc5bc7a3a53b100f26fd457aaf096e04，代码逐字节等Git；首次raw全部比较被两文档正常CRLF/LF差异挡住，随后用Git过滤后blob identity+显式仅文档EOL规范化核查，代码仍要求原字节恒等，未把不同代码归为格式差。

**T19完成，current_step4/19；F-941-05 phase闭环关闭**。T18生产修复c4cb9dab7、永久三代缓存62f57b4e2、SD08/09候选8c074c17c以及原B完整405项门证据有效，最终独立review仍T14，不能以phase自验冒最终review。原T01..15未闭，继续原全范围/机制/独立证据；不因提交/收据重复跑完整组。

本轮另将已具名4 AV record缺code的最小修复提案F-941-06写入§9/§10并提交归属问题：此前F05授权不覆盖AV producer。当前等待用户选择新增941 phase/独立修复，**不阻已授权原证据工作**；尚无答复，不把预选项当批准。T06独立span责任链调查沿第42轮证据，尚未改变reader和该格UNOBSERVED。无存活本包采集进程、无新增磁盘阻塞；原goal授权持续。

stage: work | plan_id:PLAN-941 | plan_revision:r3 | outcome:in_progress（T19通过，原完整任务未完） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:8c074c17c404b15a1d042a8bf3a77420b958997a | task_ids:T19已完成 | evidence:292cert/daily+原14优化严格通过；三原Rust关闭；全组合614P12F380U仅3格变化/48MiB内 | blockers:原全范围/具名red未闭；F06归属待用户，独立工作可继续 | next_responsible_role:work 原T01..13；T04独立T06 span oracle/完整机制与legacy证据，F06获批后new修订phase合同

stage: goal | plan_id:PLAN-941 | plan_revision:r3 | round:43 | from:work | to:work | status:executing4/19 | outcome:needs_fix（F05关闭，原完整任务继续） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:8c074c17c404b15a1d042a8bf3a77420b958997a | next:work 原完整验收证据；F06若获批则new合同

### 2026-11-06 T04原T06诊断位置独立oracle闭环（goal第44轮）

前轮T19关闭为progress。本轮继续原T04（非producer修复），入口唯一941树8c074c17c clean；F06无答复、未实施AV改动。基于第42轮独立责任链证据与canonical native-source-diagnostics合同，落实原源T06借用逃逸span：legacy parser和CST投影均把.to(str)表达式止于type-word末尾（不含右括号）；source adapter按首终结语句以前的live statement span join，text_resources verifier返回该HIR block span。**oracle仅从固定原source的唯一let与return/typeword锚计算，不读取worker输出来设expected**；原字节start19/end62，非大括号13..65或含右括号19..63。

冻结提交 **5f3fff189c5781ee828deea9bd4b30e71532e217**，2个Python文件39新增/6删除，只有ace_m1_data_diagnostics.py与其永久tests；Rust/shared frontend/AV/native/worker源、proto、原源/原oracle错误族与所有预算不变。新2个永久case校验原19..62、Unicode注释前缀按UTF8字节平移、重复/缺锚具名拒；既有native facts控制升级原T06正确span PASS、start偏移/含右括号end+1 FAIL，profile/operation/primary bool等拒绝保持。先写这些回归在旧读取器实跑4 methods，4失败断言（3 methods含subtest）/1 method通过，t06-span-red.log；仅实现独立source锚后4/4 methods绿，t06-span-green.log。不把合成facts测试冒实际compiler执行。

使用原16真实交换做**158 counterfactual_not_execution反控**（156旧场景+T06错end/错大括号），全部符合预先期望：原native T06 PASS，错start/end/大括号以及错stage/code FAIL，坏nonce/cap/source/profile/operation/primary/字段闭包REFUSED，4 AV record原缺code仍UNOBSERVED/父FAIL保。scope/t06-span-admission/raw-reader-controls.json SHA=7d69ae6afd4f7c7e52f7931150e87fee3e1624886681fbbd0365e08922d027b8。正式`cargo t infra-ace-m1-arch --out <scope>/t06-span-admission/controls` **171/171 PASS**，exit0，4.092554s，results SHA=f6227e9837cd2f99f7183c4de40935d6264888ed2ed856c5aa3c82f9385f3082；新只读oracle变更只需相关治理组，生产A2X/full405原冻结证据按未变真实身份复用，未无因重跑。git diff-check通过，源码当前等冻结提交（Git规范化处理常规CRLF/LF）。

冻结5f3fff189受管全组合、Popen禁用，exit0：**615PASS/12FAIL/379UNOBSERVED，1006原登记**；相较第43轮仅`q4-source/original-record-text/T06/AC-native` U→PASS，其余**1005格verdict不变**。prior-verdict-preservation.json SHA=881789ac02d8e210db37514a6d69679ad5316f7abd30c58e4c44030e74ab807f。原16 data诊断最终12PASS/4FAIL（T06原raw span19..62与独立锚一致），closure-audit.json SHA=87beeaef6b818eed9ef04109d3722b774bcc530b6e52d8e751fb96f920ffacd7；**GAP-941-DATA-BLOCK-SPAN-01已关闭**。旧不完整/错误范围探索记录仍保；4 AV缺code/F06、6 A2X typed、9 generic native trap、原8scalar FAIL/299legacy/49ARCH和其余完整机制全部继续开放，required_universe_closed/acceptance_system_delivered/can_claim_expanded_m1仍false。

scope/t06-span-admission/composed-preview/preview.stdout **564879B SHA=317f048a8c01dd4fd3ba7d873a5093d086e673094a11b6827fb4793d7fc35dae**；8306.0515ms、peak process30179328/Job31420416B。同一LiveBudget reserved26406615/全程peak**46867464<50331648B**，原48MiB/单件4MiB/privateΔ128MiB/先预留/流hash/不清峰值不变。B worker仍7b0be842feabdd55a7217ae783ae4af8d9a445b60e7807e5cf1bda67d5b4bb7e、AV e4c2006f…/native5016566c…原值，原292默认/daily+14优化继续真实严格入账。

T04其余适配与原T01..13完整目标未完，**current_step仍4/19**，不把单用例T06误标为任务T-06恢复/编辑工作已完成。下一步继续完整原required/legacy与ARCH/事实/恢复/资源证据；F06归属待用户，已有phase授权不外推。没有存活本包采集进程，无新的用户决定阻本轮独立工作；未进入review/merge。

stage: work | plan_id:PLAN-941 | plan_revision:r3 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:5f3fff189c5781ee828deea9bd4b30e71532e217 | task_ids:T04子缺口已闭，原任务未闭 | evidence:原T06独立锚/158反控/171治理/组合615P12F379U仅1格变化；48MiB内 | blockers:原完整证据/具名red仍开放；F06待授权但不阻其他work | next_responsible_role:work 原required/legacy与ARCH全范围对账；若F06获批则new同树phase合同

stage: goal | plan_id:PLAN-941 | plan_revision:r3 | round:44 | from:work | to:work | status:executing4/19 | outcome:needs_fix（独立诊断位置缺口已闭，原完整任务继续） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:5f3fff189c5781ee828deea9bd4b30e71532e217 | next:work 原完整required/legacy/ARCH证据；F06获批后new

### 2026-11-06 原BASE正式重放与20 required值格位接入（goal第45轮）

第44轮T06位置闭环为progress。本轮继续原T01/03/04/10/11，按旧313 required map定位待适配入口：BASE有131引用（含多个scenario共用入口，不等于131独立执行）。冻结原97 typed inventory、四相关AV/native/case-parent二进制前快照（从更早原5-bin快照投影，原记录不改）后，`AUTO_CASE_WIRE_EVIDENCE=1 cargo t case-ace-m1-base --out <scope>/legacy-base-current/gate` **97/97 PASS**、exit0、22.903805s；原完整BASE未按最容易的20项过滤。原first_slice其他target/source能力未改，真实CaseCopy和每原始交换在统一scope。AV e4c2006f…/native5016566c…/auto-case a09ab303…/pilot de12cb96…前后完全稳定；B A2X亦保持，既有292 cert/daily+14优化未失效。

新原值严格适配与永久合同冻结 **387be853edbe54d7c870c519135559aad70fcd6e**，9文件；production Rust/AV/native/C/旧Case源码/冻结binding.tsv/批准required/预算不改。新增portable typed **legacy-base-contract.json 207665B SHA=d11266d9b5a67d774af7632ddff5799768cc885cd73e8b2c08b866f26b29c1dd**是交付读取器的正式声明合同（97当前Case/check/source/oracle身份、无执行结果/进程日志），按T02入跟踪，并明确.gitignore白名单；不是把临时plan证据入库。新增ace_m1_legacy_base.py：完整97当前typed合同/CaseCopy闭包、正式组97执行/四准备工具/前后依赖均核，再只对**原B01..07/B09/B10/B12×AV/native=20普通value格**由原binding与source-manifest独立定值/源字节、原entry/caps/单run/target recipe判定，真实worker值各对原oracle；父FAIL保FAIL。负值B12=-2保符号；native70不是本slice的value/trap授权。没有将97父PASS扩为全部313/所有BASE scenario通过。

旧map仅这20格reader_status pending→supported-slice、移除对应raw-adapter gap；**execution仍全UNOBSERVED**（discovery executed0），old required hash仍35ad43aae27ffb2d2c9a162c3dd5b99f971e4cd7b2ed36f4fe747a5634bd770e。其余字段/原30修订、512 churn、10样本P90门、历史P95、38旧A2R cert-gap、62未绑定格/251 linked/304 entries全保，mapping_complete仍false。map-change-audit.json SHA=6db3b7a015c46c39ba1c17fa99b77a0e09e9c1be40d4e74ab5ef9599bbada3d2。新map SHA=036fd5eed9a78ec32040ffde088e39599f3e01f2dff1307e4f965090d64f55f3；registry第24合同指针+旧map新指针更新，1006原登记key集合不变，无app派生索引/legacy specs.json写入。

新3个永久unit控制（含B01=14/B12=-2的两个target同错、父PASS非权威、错source/path/recipe/entry/caps/空steps/SKIP/emit/改Case expected、旧未适配trap/scenario/cert-gap保持）。相关10 methods通过；正式`cargo t infra-ace-m1-arch --out <scope>/legacy-base-current/final-controls` **174/174 PASS**、exit0、5.281217s，results SHA=ad96ac5732b69a2117bda34b001b1ecc4fc3e242d53bbbe57ae93a8e9c1bebd4。原20真实交换**200 counterfactual_not_execution反控**全按预期：原20 PASS、共同错值/父FAIL为FAIL；改oracle/source/cap/nonce/recipe/空steps/子echo等REFUSED，raw-reader-controls.json SHA=22135c565fcc03c2ca7a74d2165be1b9617716f1fd64bb5803ef165e2e348b8a。原actual BASE 97执行与反事实200分开记账，不充新执行样本。

单独旧BASE CaseCopy解析保守预留峰24323797B，组合按同一原LiveBudget**先读完整BASE并丢弃瞬态，再保留有界摘要**，不是另开预算域/清峰值或先全读后检查；合并旧4格时亦先预留。预载32MiB后真实相同CaseCopy读取被原ResourceError提前拒，peak49201569<50331648B；最初scratch探针误断言错误消息包含budget字样而自身失败，改为精确ResourceError类型后成立，生产预算/reader未放宽。原4+10 typed观察与新20互斥合并，**旧required observed14→34、passed34、unknown299→279**，仍不完整。

冻结387be853e只读/Popen禁用全组合 exit0：**635PASS/12FAIL/359UNOBSERVED**，1006登记/24合同；相较第44轮仅这20 legacy/corpus value格U→PASS，其他**986格verdict不变**。prior-verdict-preservation.json SHA=f040446f3efc9573154953eb4c4dda586cc60d765580f1ae3ef35f2d70839cd6；9提交文件过滤后Git blob等HEAD，committed-input-audit.json SHA=958b26a6425678545dff82d56c76c8b29f3733d85ed22618727d96284cd731fd。旧12FAIL/其余359UNOBS/38 cert-gap/49ARCH与完整恢复资源/所有原审批范围继续，required_universe_closed/acceptance_system_delivered/can_claim_expanded_m1仍false。

scope/legacy-base-composed-preview/preview.stdout **573067B SHA=adc8a1dd2701f6ea4a156a83aae24cd6eb2cfb183c591aaed500cf847db38b5f**，8223.3701ms，peak process30081024/Job31318016B；共享LiveBudget reserved27357912/全程peak**47326853<50331648B(48MiB)**，原单件4MiB/privateΔ128MiB/64KiB流hash/先预留/不清peak保持。验证顺序由新大CaseCopy的真实瞬态量决定，未偷减原cases/规模，也未重复无触面的生产完整405组。

原T01..13继续，current_step仍4/19；不把新20格当原全部legacy/场景/ARCH已闭。下一步可继续本次原BASE陷阱/typed诊断/legacy VM原始输出或其余原正式组的严格适配，原样本场景须按原全序列与规模独立实测。F06仍待用户归属答复，不阻这些已授权验收工作；新phase未擅自实施。当前唯一树387be853e clean，无活本包采集、无新磁盘阻塞，最终review/merge仍未到。

stage: work | plan_id:PLAN-941 | plan_revision:r3 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:387be853edbe54d7c870c519135559aad70fcd6e | task_ids:T01/03/04/10/11推进（未闭） | evidence:原BASE97全门/新20原required strict；200反控/174治理；635P12F359U、48MiB内 | blockers:原完整任务/具名red继续；F06待用户但独立work可继续 | next_responsible_role:work 原BASE其余kind/旧VM与其他required/ARCH/原规模机制；F06获批后new

stage: goal | plan_id:PLAN-941 | plan_revision:r3 | round:45 | from:work | to:work | status:executing4/19 | outcome:needs_fix（新20原required已入账，原完整目标继续） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:387be853edbe54d7c870c519135559aad70fcd6e | next:work 原required/legacy/ARCH完整实证，F06获批后new

### 2026-11-06 原BASE非值完成与旧VM值产物严格入账（goal第46轮）

第45轮原BASE97执行/20值入账为progress。本轮继续原T03/04/10/11，重核唯一树387be853e clean、AV/native/parent二进制和全部原source/Case未变，**复用同一已核真实BASE97 CaseCopy，不为读取器扩大而重复执行语言/全405组**。冻结 **bef5ddc180f251e7e623c79a7295777c5f53da93**，8文件302新增/169删除（含map35边界变更）；只改Python域读取器/测试与私有registry/map，生产Rust/SDK/Case/proto均未改。

现BASE严格读60个相关source checks：旧20 probe值；4 trap×AV/native；4 diagnostic×AV/native；24旧VM值。实际**55 PASS、5 UNOBSERVED**，单独保守读峰26984562B。新增可授权入账35格=**4 AV具名trap+4 AV诊断+3 native source-v1诊断+24 legacy-AVM原值**；5 native只保待观测事实（B13/B14/B15/B20通用70、B16默认拒绝缺typed source facts），不进入仅PASS/FAIL的semantic execution_summary。由legacy.base_pending_observations保留实际请求/响应hash、观测与原因，不能从旧父PASS/渲染消息构造具名语言结果。

旧trap taxonomy直接原binding overflow/div-by-zero/rem-overflow→既有AV trap代码，不扩原规则；旧B16..19 resolve/type/assign-immutable/missing-return分别钉shared source.undefined/source.type-mismatch/verify.immutable-write/verify.no-return-path。复用scalar_q4_observation/native_source_diagnostic，新增**有限且互斥bool legacy_base分支**，默认/既有SQ/data分支原样。Native原911 profile=scalar/attempted=[scalar]/冷operation/primary整数0/原64KiB与64 causes/4096B message/旧投影/sourceSHA/UTF8边界全核，B17 bool operand、B18 assignment place、B19 body.name_span由现typecheck/verify/adapter责任及独立源锚定23..27/38..43/3..7；永久测试亦手算这些位置并禁止expected_span=None蒙混通过。B16仅文本的同义/异义渲染均不授权，父FAIL仍保FAIL；通用70不冒具名trap，正常值70不被全局误判。

旧VM值证据来自原SDK avm.rs在oracle比较**之前**保存的真实result.txt/stdout.txt，CaseCopy逐产物闭包/hash/byte-count及exact case/check/source/adapter=avm/action=run/RUN与父状态均核。仅绑定原24个普通value corpus格，实际canonical整数返回文本各对原binding，不用Case PASS或Case expected当actual；原记录明确**kind=numeric-result-text、type_or_bits_authority=false、stdout_oracle=false**，stdout原hash保留但不凭未声明的stdout猜语义。没有拿这些文本证明W位模式、VM具名trap、生命周期/资源或场景。源码源谱与new AV/native/HIR域互斥，旧A2R/其他target不能冒名。

新5 methods覆盖AV错trap/code、native typed wrong span/operation/profile/primary、generic70/渲染消息不升PASS、旧VM规范数值文本/非规范与超界拒绝、域开关非bool/双开拒绝；连原相关类15 methods通过。首合成AV负例fixture漏requests=1而被原精确counter guard拒，只补fixture到真实原shape，生产reader未放宽。正式**179/179 PASS**，cargo t infra-ace-m1-arch --out <scope>/legacy-base-domains/frozen-controls，5.210840s、exit0，results SHA=38215b4b91ab978195f48a817a0998852633c93f61a99a25cf0a7b23592fb759。初179门通过后又补“独立span oracle丢失None须拒”控制/guard，故在最终输入重跑该相关组；不是无因重复全仓门。

原帧/产物**380 counterfactual_not_execution控制**全满足：AV trap/diagnostic错族、native typed错code/stage/span FAIL；nonce/source/cap/空steps/改Case oracle/echo/primary bool/foreign profile/operation REFUSED；native generic保持U；VM共同错数值/父FAIL为FAIL，错owner/hash/缺或增产物/错source/非规范文本拒。post-integrity VM错值明确是已验证产物之后的替代值控制，不充物理被改文件的真执行。scope/legacy-base-domains/raw-reader-controls.json SHA=64b9c496aa23dd9a7e0e1b68e96fc73667c17b0524f491dbb3430c0ae4222120。初counter harness重复创建EvidenceScope/在被拒后跨独立反事实累计预留而正确触48MiB guard；改harness为有界原帧预读、复用Scope及各反事实自身预算域，实际生产全组合始终是同一LiveBudget，未改/放宽资源规则，错误harness结果未计绿。

BASE输出保守摘要实际34033B、既有统一公式bound**940951B**，原512KiB摘要ceiling不足以承载新增55条/5 pending；本域与其合并摘要在reader**之前**各固定预留1MiB，真实读后仅留实际保守bound。原全局48MiB/单件4MiB/privateΔ128MiB不变，旧BASE仍先读完大CaseCopy并释放瞬态；不另开生产预算域、不截事实/清peak。9种旧原场景/全部原样本门未缩。

map仅新增35格reader_status/gap边界，execution仍discovery U；old313 required hash/原30修订/512churn/10样本P90/历史P95/38 A2R cert-gap/62 unbound/251 linked/304 entries不变。新map SHA=00d3af740318d4478e6f634ec68c2c3c28c5e4ee6380cd4b770a363a3bef3e7d，registry只刷新其指针，仍24合同/1006登记；map-change-audit SHA=830af22d62901c3f896c5468e1b3a1b8ac54d2c1e0101ee4ba053bb9ed0bf54f。实际required观察**34→69，全69 PASS，未观测279→244**。场景/VM错误/A2R旧缺证书不因BASE父97 PASS消失。

冻结bef5ddc18受管只读/Popen禁用全组合 exit0：**670PASS/12FAIL/324UNOBSERVED**；仅上述35 legacy格U→PASS，其他**971格verdict不变**，prior-verdict-preservation SHA=105b39380295b433d561f0fa004b0d2e453d1e0c5ab8c708dc74a3ef36a2b62d。8提交文件Git过滤后blob等HEAD，audit SHA=18293167dc749b57d1865e0d5dff183bdf1acb2114771382953c2d8a5fdad275。原required_universe_closed/acceptance_system_delivered/can_claim_expanded_m1仍false。scope/legacy-domains-composed-preview/preview.stdout **593452B SHA=a80bfa955bdb84baac739cdf911f4a1bd4f050943a8b760e82f6421550595ffe**，12662.6587ms、peak process30658560/Job31895552B；同一LiveBudget reserved28273728/全程peak**47819143<50331648B**。

新增具名**GAP-941-LEGACY-VM-COMPLETION-01**（evidence owner=941 T11）：BASE的旧VM B14/B16/B20/K04/T07负例已真实运行，但error.txt仅message且原失败分支没有结构化产物；row父PASS不是stage/code权威。已核SDK avm.rs::classify_staged返回实际Language Failure.stage/code并在parent oracle前可取，当前只是未持久化。下一步属于原T11“必要父adapter/SDK”有界证据适配：只把已有实际失败事实（非expected）作为结构化产物导出并保持原分类/行为，再测SDK/原Case消费者并刷新受parent身份影响证据；不改VM语言能力/错误类别、不解析message出code。未在本轮实施，旧VM五负格继续U。该SDK证据缺口与需另授权的AV data producer F06不同。

当前树bef5ddc18 clean，current_step仍4/19、T01..13完整任务未闭；无活本包采集/新增环境阻塞。继续原required完整适配/机制/ARCH，F06等待用户归属答复但不阻独立work，未擅修AV producer；最终独立review/merge仍待。

stage: work | plan_id:PLAN-941 | plan_revision:r3 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:bef5ddc180f251e7e623c79a7295777c5f53da93 | task_ids:T03/04/10/11推进（未闭） | evidence:复用BASE97、55严格pass+5显式U；新增35原required/380反控/179治理；670P12F324U、48MiB内 | blockers:原完整证据/具名red未闭；SDK旧VM失败缺产物归T11，F06待用户 | next_responsible_role:work T11旧VM结构化实际失败证据/必要SDK消费者与当前身份刷新；其余原required/ARCH，F06获批后new

stage: goal | plan_id:PLAN-941 | plan_revision:r3 | round:46 | from:work | to:work | status:executing4/19 | outcome:needs_fix（原required69/313入账，完整目标继续） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:bef5ddc180f251e7e623c79a7295777c5f53da93 | next:work 原T11证据适配/required/ARCH；F06获批后new

### 第47轮：T11旧VM真实失败产物与父工具身份刷新（2026-11-06）

- 在同一941树执行原T11必要SDK/父侧证据适配；不改语言分类、oracle、旧required批准或任何预算，不包含待授权F-941-06。实现提交`2a6018848`，当前冻结`d745efad8ddf6345df390a50849259c9dffbc005`。`avm/run`在oracle比较前写并注册`failure.json`（`autolang.avm-failure/1`）：实际类型化stage、可空code、组合source与entry字节hash；PASS/FAIL均保留，I/O/panic等基础设施错误不生成语言失败产物，写入失败为ERROR。成功result/stdout边不变。工作树Spec候选记录该边界。
- 新永久回归实际执行同一除零源码四次，对照正确/错code/错stage/错success预期：1PASS/3FAIL且四份产物字节相同，hash/size/source身份均核对。`cargo t auto-lang-core --test-name test_cases::avm:: --out <h>/legacy-vm-failure-sdk`退出0，5/5、0原条件排除，运行1.1484551s/整命令39.9960352s，收据SHA `284c37ec4b4e72865d8baf77b10ca6d468647e85a9558be371671eb4b26f2e8f`。`cargo t auto-lang --test-name tests::test_cases_tests:: --out <h>/legacy-vm-failure-facade-strong-oracle`退出0，完整模块27/27、原ignored bootstrap1项不充PASS，运行4.2003802s/整命令11.8581613s，SHA `511180a8dbe6d80dfb946f96fcf25f00587e518e718f1039cb702f116b892058`。采用原固定包入口下的完整受影响模块；无VM执行/分类语义变更，不扩跑整个SDK的无关消费者。
- 开发首次两次模块门各26P1F均为新增夹具合同错误（数字开头ID不准入；裸stage缺稳定识别），修正ID为t941.failure并给错stage保留DivisionByZero标识后全绿。原始红门分别保留`legacy-vm-failure-facade` SHA `bff4494eeea7fe5c44f52da136695ec6ef39e87ba4f4ddfff6e8e03f22f3af69`、`...-correct` SHA `3fd1d1e543e3855f0882c94d2faf29853788af384a4fca8ba91d8d8ea0a93b00`，不归环境或PASS。
- 正式工具准备先于取证：AV仍`e4c2006f31c4a8aff96e5f0422f0d8ce5cf7667dedfb6aebe1347f7b2521a544`、native仍`5016566c9511a8fd2181b588b99b9fde8595739d43dd331f97737f45deb322f6`；新auto-case=`5adb666b12e9a262c5958b392575d6302791a18d261ce99b8e881106998c4261`、pilot=`23376ea802177ed01f3210040d33ffe7ccb970eafa4558f8d390cfaac28e0abe`。四依赖before/after真实相等，旧快照未改写。`cargo t case-ace-m1-base case-pilot --out <h>/legacy-vm-failure-current/gate`退出0，97+24全PASS，整18.4268787s，SHA `fa30c00863e0c319edfa12b45d3f9971cefe75b425abf36c778c763b835c35f1`。该次未开启wire，严格联读正确拒绝required original exchange missing，不能作BASE严格证据。
- 开`AUTO_CASE_WIRE_EVIDENCE=1`重采原BASE97，`.../gate-wire`退出0、97/97，运行20.0335341s/整23.5267847s，SHA `a054646dc8bb57232cf6004539ca4271fc715167e8286019c4d1b20642e5ce26`。实际5产物独立核对CaseCopy hash/size、source/entryhash和原typed contract：B14/B20 run/DivisionByZero，B16 type/auto_link_E0401，T07 parse/auto_lexer_E0001，K04 parse/auto_syntax_E0001。`actual-failure-audit-wire.json` SHA `29e0313dd2e17d816ab89f96c8cd01e49cd85134239dabf3f0c12ac83635a4da`；此审计不替代严格reader，新增required记账仍0。
- 第二次联读准确拒绝旧parent-inventory的生产者hash；重新实采只读parent-inventory（executed0），原合同内容不变。第三次准确拒绝预算登记的旧avm.rs pin；`d745efad8`只刷新该源码hash及其两处合同hash指针，16域policy/limits/UNOBSERVED/执行数均逐字段不变。审计`budget-source-pin-audit.json` SHA `7968240a7106323b997d4bd327df4a4e20112a5a75d24a8bbe716eac25e7e850`。原预算合同新SHA `791ae6c4a69244c3995d2c4f84ecc26608a3ecdeeef1e161ff348c951f2d4e08`。
- 冻结`cargo t infra-ace-m1-arch --out <h>/legacy-vm-failure-frozen-controls`退出0、179/179，结果SHA `6ea67150d71d4cae94548ec59cc70eb25c177030fc8317b3bae8f06ca14086be`。代码及新增Spec `git diff --check`、rustfmt通过；既有Core131/宿主lib5及测试32 warnings仍属T13未闭，未加allow或消音。
- 当前完整绑定`<h>/legacy-vm-failure-current/composed-bindings-current.json`。最终纯组合器禁隐式Popen，退出0；`legacy-vm-failure-frozen-composed-preview/preview.stdout` 593452B、SHA `9bf435f010a48706300a7e77796581e8b455b1b4533eeb946fc630c31d5f06f7`，9.3227863s，进程峰29773824B/Job31006720B；同一LiveBudget保留28274104B/峰47819439B<48MiB。1006登记项仍670P12F324U，与第46轮逐行verdict完全一致，审计SHA `d40915ef910eb94be4874c952d75e8f4f85f711d9a62cf9c4ba6a25ec6d4f537`。原required仍69/313；required_universe_closed、acceptance_system_delivered、can_claim_expanded_m1均false。292证书及原14优化由完整当前身份联读复用，无A2X/AV/native语义变更。
- 下一步原T11补有限五格VM失败严格reader与结构/hash/来源/父FAIL/空code/基础设施反控，再考虑从U入账；`GAP-941-LEGACY-VM-COMPLETION-01`尚未关闭。原T01..15、ARCH、38旧A2R认证守卫和12真实red仍保持；F06等待原问题回答，不重复请求。无运行中进程。本次确认D盘空闲257742336000B，用户清理已生效；未进一步删除受保护的941材料。

stage: work | plan_id:PLAN-941 | plan_revision:r3 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:d745efad8ddf6345df390a50849259c9dffbc005 | task_ids:T11推进（未闭） | evidence:SDK5/模块27/BASE97/pilot24/治理179PASS；实际五VM失败产物，完整670P12F324U维持 | blockers:五格严格reader待补；原完整义务/具名red，F06待用户 | next_responsible_role:work T11五格实际VM失败严格适配与反控；其余required/ARCH，F06获批后new

stage: goal | plan_id:PLAN-941 | plan_revision:r3 | round:47 | from:work | to:work | status:executing4/19 | outcome:in_progress（SDK事实边已补，原required仍69/313） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:d745efad8ddf6345df390a50849259c9dffbc005 | next:work 原T11严格适配/required/ARCH；F06获批后new

### 第48轮：T11五格VM失败严格读取完成；接收F06内修授权（2026-11-06）

- 上轮属progress：实际SDK事实边/当前BASE与父身份已补。本轮从main r3 executing4/19及`d745efad8`同树clean核对后继续原T11；未在main修改代码，未新建树/agent。实现并冻结`330c2534497e90ea6b06c5156382ffda4597aa76`，仅6个交付文件（Python读者/测试、旧map/registry及Spec候选）。
- `ace_m1_legacy_base.vm_failure_contracts`从原已批准required/binding/source-manifest独立认领有限B14/B20、B16、T07、K04；actual数据不供oracle。Case完整97身份/生产工具before-after/源和冻结typed contract先校验，再读注册`failure.json`的精确owner/hash/size≤16KiB及source复合hash/entry字节hash、固定schema/adapter/action/outcome和实际stage/code。code空或非法形状拒绝；可空None在阶段正确时U，阶段已错误则FAIL；父FAIL/ERROR/SKIP保FAIL。error.txt或基础设施结果不供语言事实。K04原`ok-parse-reject`仅证明解析拒绝，显式`semantic_borrow_rule_authority=false`，未升格成借用语义证明。
- 原BASE来源仍上轮wire97与`legacy-vm-failure-current/composed-bindings-current.json`，没有改原源码/expected/生产者/编译能力，也未重跑无关语言组。BASE选读从60增65（60P+5显式U），与旧4+wide10合并得到原required74/313，239U；原5U native generic trap/无typed源诊断仍保留。`GAP-941-LEGACY-VM-COMPLETION-01`由实际产物+当前严格reader+控制关闭；不能据此关闭T11整任务或ARCH。
- map只改这5格reader_status/gaps，map.execution仍UNOBSERVED、发现执行数0，required313/原批准输入/所有原scenario 30/512/10样本及预算和38旧A2R cert-gap不变。新map SHA `0bf849f577868553f8c4535a3c8dea55a4bcd5eba77f5e6fcc659da4bd1b4964`，对应两处hash指针同步；变更审计`<h>/legacy-vm-failure-reader/map-change-audit.json` SHA `49cdd30d344ed5b6eafae02bccc6e299468c026ad2951fbb0ab4b2ac65521e84`。
- 新增永久6方法涵盖五原代码/父拒绝、actual错stage/错code/None、源与current check、基础设施/伪passed/消息文本、产品owner/hash/size/缺失/重复、非法/重复/超限JSON及分配前16KiB拒绝。开发185/185；冻结`cargo t infra-ace-m1-arch --out <h>/legacy-vm-failure-reader/frozen-controls`退出0、185/185，整5.0346166s，结果SHA `a16b1af75362550043e725a8fccf43f56b40821a344aad6249075c62645a2d14`。所有6文件Git过滤后blob与HEAD逐项一致（working EOL不冒陈旧），审计`legacy-vm-failure-reader-frozen-preview/committed-input-audit.json` SHA `225e3c16de5849b733f66f628bc6fb4c091c40ad4e4001195af75e8932d68f43`；git diff --check通过。
- 真实5产品经原完整Case绑定后另做43×5=215隔离counterfactual（全匹配），包括真实错码/错阶段、None+错阶段仍FAIL、来源/owner/hash/size、缺失/重复/超限、infra及父拒绝等。每个副本都标counterfactual，不冒运行收据；`legacy-vm-failure-reader/raw-controls-final.json` SHA `9f2785c908904abd34b91a297dfff1989d726cf0e48b1c3bfd19cf7ea01a25f0`，additional executions=0。此前210反控原记录保留SHA `efbbc9172decec82c3a2265637080a2cd633fe2259523de18035fa64eff42b31`。各反控独立预算只证明控制；全组合仍单一LiveBudget，不拆生产预算。
- 冻结纯组合器禁隐式工具启动，退出0，`<h>/legacy-vm-failure-reader-frozen-preview/preview.stdout`595584B、SHA `2fe94189f6552b4ce28a4bdb1a5956f9f2a950cdfb6012cbc2a01a325b49460a`，8.4208468s、进程峰29896704B/Job31125504B；保留28385727B/全时峰47872973B<48MiB。整体675P12F319U，逐行仅5个legacy/corpus:*:avm的U→P，其余1001项verdict完全不变；审计SHA `b2ba0a011cc2a9d83887f87808bd87581e20f1b3262e38db8c6ae0236af04bdd`。24合同指针/1006登记，required_universe_closed、acceptance_system_delivered、can_claim_expanded_m1仍false。292当前默认证书及14原优化证据在未变的A2X/AV/native身份下继续通过完整联读。
- **F-941-06授权已到**：用户本轮回答原问题`call_ad921d1cbddb484085b350b7ee0bc3f5`：“授权941新增修复phase（推荐）”。授权最小范围仍是data链保留并透传共享前端结构化诊断、原4 AV record负例与合法/失败边界及受影响消费者复验。§10改为已授权；无需再请求。依goal单轮单阶段，本轮先完成已在执行的T11读取，下一轮`auto-plan-new`修订r4加入具名phase/任务/验证合同，然后work；当前r3/4-of-19暂不伪写已交付新phase。未据本授权修改其他缺陷、预算或原目标。无活动测试句柄；T13 warnings与原全部未闭义务保持。

stage: work | plan_id:PLAN-941 | plan_revision:r3 | outcome:needs_replan（用户已批准F06内修，下一阶段修订合同） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:330c2534497e90ea6b06c5156382ffda4597aa76 | task_ids:T11五格子缺口关闭/原任务未闭 | evidence:185治理+215反控；74/313旧required；675P12F319U单预算 | blockers:原完整required/ARCH/具名red；F06授权已收到不再等待 | next_responsible_role:new r4 F06结构化诊断phase合同，继而work

stage: goal | plan_id:PLAN-941 | plan_revision:r3 | round:48 | from:work | to:new | status:executing4/19 | outcome:needs_replan（内修授权已到；完整目标不变） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:330c2534497e90ea6b06c5156382ffda4597aa76 | next:new F06新phase合同→work

### 第49轮：r4 F-941-06内修phase合同移交（2026-11-06）

上轮新增5格严格读取/185治理/215反控为progress。按第48轮用户明确授权执行new，未再请求；当前唯一树330c25344 clean，main代码路径零WIP，未新建Plan/树/agent，.next-id不动。本轮仅计划修订及scope只读源码定位，未实施新producer、未运行拟V25..28，也未提前勾新任务。

源与规范核实：current `data_driver::request_data`将共享Vec<Diagnostic>转换String，worker按前缀分frontend/verify并固定空code；core有StageDiag；owned实际仍从前缀提取首词code，仅作为原兼容消费者而非typed通道先例，未纳入本项修改。现strict reader已能承接正确非空code。公共String API仍有AV/native直接消费者，故新增typed失败边且旧签名保留，worker禁止回读String；内部共享原primary保留，wire按原pipeline映射而非把bind/verify直接写成未约定阶段。data/owned成功/精确trap/store/资源/缓存和协议外形不扩。只读基线`f94106-contract-baseline/sources.json` SHA=4eaa047eeaf49d377f5e96586aa67e5ef150d31f930937083c10feccf6a386fa。

r4增加§5.10、T20/T21、AC16、V25..28、SD10和auto-av/data-layout规范元数据；原current_step4不变，总任务21。更新T12/T14依赖与最终r4审查身份，保T16..19已完成、原T01..15未闭。V25为四原真实worker诊断，V26分类/未知与IO边界，V27合法及旧消费者，V28完整职责门+当前身份闭包联读。T20→T21无需其他新决定即可开工；实际API小实现形状由work在本合同选择，若触新协议或非最小语义面须举证路由。没有用规划文本替代运行证据。

继承当前有效证据为第48轮330c25344/675P12F319U与原required74/313，仍非系统交付或M1claim。r4使任何旧最终review许可无效（当前本来尚无T14最终pass）；未变输入/工具的细粒度执行证据保持有效，之后AV或其他二进制/源pin改变须真实刷新对应闭包，不修改旧raw。独立最终review仍同树新上下文T14。

静态预检退出0：0..10节完整，21唯一任务/4已完、16 AC、28 V、10 SD均唯一；新增case路径尚不存在且标拟新增；`cargo t --list`确认5个现固定组，新增T20/21与SD10仍在原任务/增量表内而非第二表。预检`<scope>/f94106-contract-baseline/preflight.json` SHA `165c4547e532d3b2ea1693eff8ac371efd56bcc18c7a1a58b7e45e9108ce7a19`，git diff --check通过。

stage: new | plan_id:PLAN-941 | plan_revision:r4 | outcome:pass（用户授权已覆盖；合同就绪） | changed_tasks:T20/21,T12/T14依赖 | changed_acceptance:AC16,V25..28,SD10 | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:330c2534497e90ea6b06c5156382ffda4597aa76 | next:work T20 typed data失败边/永久原4反例，再T21

stage: goal | plan_id:PLAN-941 | plan_revision:r4 | round:49 | from:new | to:work | status:executing4/21 | outcome:pass（仅合同阶段完成） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:330c2534497e90ea6b06c5156382ffda4597aa76 | next:work T20/21；原完整目标保持

### 第50轮：T20 typed data诊断修复与原4红→绿；T21完整并集在跑（2026-11-06）

- 第49轮r4合同实际写入并静态核查为progress。复核main r4 executing4/21、同树330c25344 clean/main代码零WIP后执行work。T20提交冻结 **be7593e00a03edf8fd1985de8bfe49a5a3475429**，5文件：data_driver.rs、av-worker.rs、conformance注册、新永久data_diagnostics_941.rs、SD10 data-layout候选；当前树clean。T20可勾选，current_step5/21；T21仍未闭，F06/GAP不提前关闭。
- **原实现反证**：只加永久worker回归，在原生产330c25344上用原M09/M10/M11/R09及真实debug av-worker `case-serve`逐项执行，原4个预期code均实际得到空串；nextest真实exit100、0P4F，904为未选（非执行/非PASS）。baseline capture `f94106-t20-baseline/command.json` SHA `6e99f5eced9404bc7698be3f392568133efdde44c8e6c806f80503900f2f6819`，build/test整8.2821417s，test0.231s；四份请求/真实stdout/stderr仅scope内具名data-diag-941-*，不改源/oracle。外层采集脚本成功不冒内部nextest成功。
- **实际修复**：新增`request_data_staged`与`DataRequestError`保留FrontDoor、完整有序Frontend(Vec<Diagnostic>)、Lower、MirVerify、Encode、AbcAdmit与Infrastructure。source primary stage/code/message/span到worker投影前仍完整；`diagnostic()`只按typed原因与既有pipeline映射投影frontend/verify，code来自原因。worker data-run不再按Display前缀分类、不固定空code。旧`request_data -> Result<_,String>`只调用同一staged管线再展示，原prefix/公共签名兼容；mem入口、restore、cache/AVAD/语义轴、源/HIR/MIR/执行器及owned/core生产逻辑无语义变动。rustfmt另规范一个既有resource event iterator换行，非逻辑修改。
- **失败边界决定与证据**：source.infra、空frontend诊断/空code、没有稳定code的AbcAdmit Identity不制造语言面，worker按既有request error退出2；Infrastructure的任意“frontend:”文本也无诊断权威。String包装保原文本。真实front-door输入4MiB+1保原ResourceKind/InputTooLarge与原admission原因，wire沿既有data frontend投影，资源code不被伪装成四源码code；缺entry真实Lower/lower.entry。MIR/codec/ABC的非前端typed边界用具名类型控制断言verify，未声称合法源码可实际触达这些内部坏态。
- **永久定向集12/12**：4原worker负例，完整frontend顺序/primary内容与String兼容、修改message不改stage/code及伪prefix infra、实际输入帽/缺entry、typed非前端准入；合法M01/R01/R02两API与worker分别0/19/9，owned/core各42，真实trace b,a/值3，B14精确av.trap.div-by-zero，真实store路径被文件占用时SDK Infrastructure/worker非零空stdout/原文件字节不变。worker回归复用auto-tool-execution::run的30s owned ProcessTree与单个有限原请求，原始请求/输出保scope，响应逐份≤64KiB（这是有限实际响应校验，不声称设施新增预分配输出限额）；外层原process_capture输出2MiB/错误1MiB保原具名退出。
- 迭代首修12P/0F，整10.4700508s/test0.388s，`f94106-t20-local/command.json` SHA `5c22fad34044c517335a644822d1485139f8c22e8d14ddd0f00831c9e66e5a20`。补source.infra frontend原因为None的控制并加SD10候选后，最终定向命令`cargo nextest run -p ace-av --features test-case-batch --test conformance --config-file ../../.config/nextest-full.toml -E test(/^data_diagnostics_941::/) --no-fail-fast`退出0，12P/0F，904未选不充完整组；整2.9781147s/test0.409s，`f94106-t20-final-local/command.json` SHA `def2f8fcec5d8927e26a01bd76897c6b7665e8b3e338977cfafd46947203724e`。rustfmt/diff --check通过；旧core1/AV3/test4 warnings原样单列，无新本修复warning/allow，不将其算T13健康关闭。
- **T21先正式准备再取证**：按原prepare_tools准备12个具名工具，合build48.4534535s；`f94106-frozen-preparation/preparation.json` SHA `929d4d5fc2857b2afc2c9a6693ca1ccfccdb5d132e1883137b3ba9bc01b5d91b`。实际新AV=`65017bf95a39da60f911c1dec5b1b3a3d0949305c054b1a0ffac893bc498c364`，native=`d6f6a1ba20aac88a6115eb8bbe837c7eca63a5c429a34b1592473ae612d04b1e`，SYN=`ebd67069b280e809d3c134bc061db2faa87f6852dc6cb1bdb931927a3f8fdf4c`，ABI=`765ce8cf0257b9ae5dad9eabccd22fdc68aa5f77a0d2687230b1ffb26d201e69`；mn-collections/CT/sema也实际变，不能仅按源未改而复用旧wire。A2X仍B=`7b0be842feabdd55a7217ae783ae4af8d9a445b60e7807e5cf1bda67d5b4bb7e`，case-parent/运行库当前hash未变；292 cert/原14优化仍须各域全部pin核对后才复用，不先声称全组合当前。
- 真before记录`tool-pins-before.json` SHA `5c6ad1c2b2a24bd6fa17f0b0b483560889e51ce3be5a4c402d49905abbfc970f`，原data/text共同11依赖before `source-dependencies-before.json` SHA `06eda51fde9700a9ed2fcceefab014de3210425197d29dc3c359bb91af0bf93d`。旧raw/旧before/旧证书不改写。§9第48轮675P12F319U和required74/313现在仅上一稳定身份历史，当前总组合需刷新所有实际失效闭包，尚未重新通过。
- 以冻结be7593e00、AUTO_CASE_WIRE_EVIDENCE=1、AUTO_PLAN_ARTIFACTS=<scope>、TEMP/TMP=<scope>/t启动原完整并集：`cargo t ace-av ace-native infra-ace-m1-arch case-ace-m1-original-data-q4-941 case-ace-m1-original-data-diagnostics-941 --out <scope>/f94106-frozen-union`。当前仍在首ace-av组，无终态，**live exec session27138 / fixed Python PID39216 / driver PID13584**已实际核存活并poll同session仍running；当前不要启动另一轮门或改交付代码。后续先核同handle，接原并集至终态，再比before/after、原16诊断strict/源证据刷新及全组合闭环。原所有样本/ignore/平台/性能条件不减；没有把进程存在或group启动当PASS。F06与T21仍pending，原8 scalar red/38 cert-gap/完整任务及所有ARCH保持。

stage: work | plan_id:PLAN-941 | plan_revision:r4 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:be7593e00a03edf8fd1985de8bfe49a5a3475429 | task_ids:T20完成/T21原完整并集运行中 | evidence:4实际baseline红→12定向绿；正式准备身份已冻结 | blockers:无新增用户决定；完整并集及当前失效证据刷新待 | next_responsible_role:work 先poll session27138/PID39216，禁止重启；终态后T21身份审计/strict刷新

stage: goal | plan_id:PLAN-941 | plan_revision:r4 | round:50 | from:work | to:work | status:executing5/21 | outcome:in_progress（原完整并集live） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:be7593e00a03edf8fd1985de8bfe49a5a3475429 | next:work 接原session27138至终态，继T21

### 第51轮：完整AV1031通过；当前身份闭包与后续408原Case精确划界（2026-11-06）

- 上轮T20实际提交/原4红→12绿/冻结正式准备为progress；本轮先重读main r4 executing5/21、树be7593e00 clean/main代码零WIP并核现存session27138、Python39216/driver13584。原AV命令cargo PID50108确实存活，接同handle，未重启。随后原组真实终态 **PASS ace-av，1031/1031、0原条件排除**，run437145.9391ms；`<scope>/f94106-frozen-union/0/result.json` SHA **ae74eb1c6ce9a53689bbae7b3e0c3f9e60cb1dbcf920fb2a6a712d5f0b38ac11**。比此前AV1019新增本次12永久回归；父完整selected/terminal/child核验通过，不仅总数或脚本exit。
- 同一完整并集已顺序进入**ace-native**，当前live仍 **session27138 / fixed Python39216 / driver13584 / native cargo PID42476**，实际poll仍running。Native discovery已有真实文件但运行未终态；不把其编译完成/进程存在算PASS，infra185与原data36+诊断16亦仍待此同并集后续。没有第二测试/认证进程，没有改冻结交付代码；当前T21/F06未核销。
- 只读核对上一稳定全组合的每份具体绑定，而非按包名推断：4角色中AV/native/SYN的raw worker身份失配，**bridge/ABI实际原本就是765ce8cf…，当前相同**。第50轮对ABI也失效的泛化描述/retention状态更正为本项精确对账；旧记录保历史，未把旧binding改写成新hash。10个带显式before/after域中text/RT/collections/query_instances/original_data/diagnostics/legacy_base仅各自AV与native或mn-collections二进制失配；3个原Rust release域before/after全部pin当前相符。无预算源pin涉及此次data_driver/av-worker源文件，预算声明不用改hash迁就修复。
- 8类+首例合计292条证书producer pin均仍为当前A2X B=7b0be842…；这只是producer/原14配置依赖身份复用候选审计，不替代完整cert/source/toolchain/产物/配置严格读取，未并行重跑认证。`<scope>/f94106-bound-identity-audit/audit.json` SHA **b7cb7cb2cb6320cdbd478caf4405f7b9ea231df63a90eb2e96ad742a7534b457**，明确标during-live-union而非伪after快照，无隐式执行。
- 准确定位仍需刷新原始交换的**14个既有Case组/408 checks**：q4 4、SYN原11（组合义务10不代原11check）、旧B08/B11 4、typedAV18、typed-native12、sourceCTE14、text20、scalar122、RT32、collections40、source-supplement8、原BASE97、source-query14、instances12；原data36+诊断16在当前冻结并集另52项，不能提前称已执行。原bridge6是hash相符的复用候选，仍需最终完整reader复核；独立declaration/typed inventory旧输入如果逐hash当前相符可复用，不以它充执行。`source-refresh-plan.json` SHA **cf05ac6888a630f8d9e8ca526f518bc5817ea991a46a9acc5d430affb589f3e7**；该表execution0、仅下一执行定位，不加分母或删原义务。
- 已准备而**尚未执行**scope/refresh-f94106-source-cases.py：先强制原完整5组实际terminal PASS、当前be7593e00/clean及全部正式prepared工具hash未变，才创建新scope并沿**原cargo t**批调14组；使用原capture/runner/meter和原wire留存，只建本次新的before/after，不回填旧raw。原scalar8真实FAIL仍使原formal gate失败并原样留红，脚本只核差异是否恰为已知原8，绝不把overall gate改PASS或隐式豁免。原data52结合当前并集的真before11pin与新after，其他域按真实依赖集新快照，最终只产待strict的新bindings。AST语法预检通过（未运行）；执行入口收据`next-execution-entry.json` SHA **387664a1cb6f5e46aff7144772166966b8b4679c5b30ec07d1cf9aa32cfd490f**，列明原handle terminal/工具身份前置；不与当前native组并行启动。若工具末态变更，先诊断稳定身份，不能让脚本覆盖或重标旧证据。
- 下一步先续现session27138到所有组终态，核工具after；若完整绿且身份稳定，再运行已准备的408批次及最终原reader，校原4code/其余verdict守恒，F06关闭仍未到。上一版675P12F319U仅旧身份历史，不冒当前已验证报告；原required74/313历史覆盖、8 scalar red、38旧cert-gap、全部ARCH/其余原任务和T13健康门继续保持。

stage: work | plan_id:PLAN-941 | plan_revision:r4 | outcome:in_progress | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:be7593e00a03edf8fd1985de8bfe49a5a3475429 | task_ids:T21完整AV1031通过/native仍运行 | evidence:AV1031真实terminal；现绑定身份审计和408刷新定位 | blockers:无新增用户决定；T21完整并集/身份闭包未齐 | next_responsible_role:work 先poll同session27138/PID39216，终态后工具审计与408原Case刷新/strict

stage: goal | plan_id:PLAN-941 | plan_revision:r4 | round:51 | from:work | to:work | status:executing5/21 | outcome:in_progress（AV完整绿/native live） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:be7593e00a03edf8fd1985de8bfe49a5a3475429 | next:work 同handle27138，禁止重启/并行新门

### 第52轮：T21完整1493门/408重采与当前严格679P8F319U；F06工作闭环（2026-11-06）

- 第51轮AV1031实际通过与身份闭包定位为progress；本轮main r4 executing5/21、be7593e00同树clean核实后续同session27138/Python39216/driver13584，未重启。实际native子用例从32MiB冷对象集/512次编辑与物理GC压力转入原30样本四状态source-diagnostic性能门，原样本与条件保持。其后原完整并集真实**exit0/全部5组PASS**：AV1031（437145.9391ms）、native225（1523882.1005ms）、治理185（4842.1934ms）、原data36（16507.9791ms）、原诊断16（3624.1615ms），合**1493执行**；AV/native均0原条件排除。整2019.480853s，`f94106-frozen-union/results.json` SHA **476fe6fd94d0b3b0d03f421d4d420db2cb007f73294b89eb7dfb340943fef626**。session27138真实terminal，不再有该live句柄。
- 正式准备的13件工具/库（12工具名有多二进制）在并集终态全部与真before相等，`f94106-frozen-preparation/tools-after-union.json` SHA **81952c96dc6246a93781ff19784fb83a8d32b6cbad302fe36e12c78d9158758d**。不因source code未改而假定工具未变；本次实核稳定才进入新source批。
- **未变A2X证据实际严格复用**：在禁止Popen的原域readers逐项核完整source/oracle/工具/flags/actual与独立reference/产物/daily（非仅producer哈希），292默认cert+292 daily全部PASS，daily compiler_spawn_zero全true；3域原优化8+4+2=14全部PASS。只读12.16825s、0新工具链/guest执行，原始材料未改；各域新派生报告与总收据只写`f94106-reuse-strict-audit/`，总audit SHA **d736b50ce907fb3e2849b611bde6a40717fcb6185453c59bdc2b63790ddf4d29**。该分域预审不是全组合预算证明；最终同一LiveBudget联读另见下。
- source脚本执行前增补冻结HEAD/clean和runner无source-state error的末态guard（避免原scalar red掩盖其他失败），新入口收据`next-execution-entry-final.json` SHA **53f336c79ec1a3230c307e5b83fd31f16f38b8ec0bb4c4b91e37c479ea43f00c**。确认前置全5组PASS/所有工具稳定后才启动scope/refresh-f94106-source-cases.py，原**14既有固定Case组/408 checks全执行**，13组PASS；scalar122组仍114P8F，失败case/check/status集合逐项等原8，原formal gate真实exit1、gate_passed=false，未重标PASS/改oracle。整172.6850814s（外capture173482.4554ms）；`f94106-current-source-cases/gate/results.json` SHA **d2a9c789f700fd3579a7fa8d7b443d344abb150a459f7f10a786a4258fdc9f26**，command SHA **237ac6e3d9ba21957597e3b34fd37d40ae029f57718c052e0c49e40c01ff91a6**，capture-audit SHA **e7545c47ebc0094a02f9ddec02dcf34e168c1a1975575455ddf5743d4920f7a6**。原data52复用本次完整并集的新wire，非旧缺码交换；Bridge6因真实全部身份未变继续原严格复用。两次采集均AUTO_CASE_WIRE_EVIDENCE=1，材料单scope，无第二runner/meter。session59208真实terminal。
- 首次全组合**明确拒绝**本包metadata域封套错误：原data/diagnostic需要rt-q4-dependencies/1，但共用11-pin真before被包装为text-q4-dependencies/1。原`f94106-current-composed-preview`拒绝日志/command保留，不归producer或PASS，不改reader。已证明两域要求的11个实际文件集合完全相同；对真实采集前before及真实after各生成只改schema名称的record/text视图，snapshot阶段和每个pin字节恒等，原文件不覆写、无after回填before。新`composed-bindings-rt-views.json`显式指向这些视图；独立来源/hash对照`f94106-current-source-cases/snapshot-domain-view-audit.json` SHA **bdb1d9f7208365b577329a75eca7a86e9da90fe755199dcc70978a05ea06eca2**。没有为封套错误重跑有效实际Case，也没有放宽准入。
- **最终当前完整严格联读**：冻结仍be7593e00；正式绑定`<scope>/f94106-current-source-cases/composed-bindings-rt-views.json`。原纯组合器Popen禁用，退出0；`f94106-rt-view-composed-preview/preview.stdout` **594473B / SHA f3c4f01858eea18c518b01dada450d9d8f234455175d9dd6574ef7d7d1ed6ff0**，9579.4788ms，进程峰30621696B/Job31854592B；保留28370407B/全时峰47805468B<50331648B，48MiB/4MiB/privateΔ128MiB/64KiB流hash等原帽不变，无拆预算或隐式重编。整体**679PASS/8FAIL/319UNOBSERVED**，1006登记/24合同指针原样；原16诊断当前全部PASS。
- 逐1006项与第48轮相较：**仅M09/M10/M11/R09的AV从FAIL→PASS，其余1002项verdict完全不变**，verdict-preservation SHA **46692278f56031a7fc90521976ef50f0a7669d0de351e5177511fd2c33e06bb0**。原required仍74/313（74P/239U），38旧cert-gap守卫及所有原尺度/批准保持；required_universe_closed、acceptance_system_delivered、can_claim_expanded_m1均false，不签系统或M1已交付。
- 基于这次真实16 Case/wire再做**156原始帧反控全部符合预期**，0新执行：全部original已PASS（包括当前T06 span）；错code/阶段/独立span必须FAIL、错nonce/源/caps/owner/profile/primary/operation与多字段必须拒；即使message填全四正确code，空code仍UNOBSERVED。原控制脚本旧source与旧未知状态不改，当前副本`f94106-current-diagnostic-controls.py`只认新wire；新`f94106-current-source-cases/raw-reader-controls.json` SHA **2657bbbfe235b92bbd5f8e8dc118cf8511d63d735a0d27df37c8a2ac701added**。每个counterfactual独立预算不作全局证明；永久185已有M09正确typed码/错码/缺码/伪PASS反控，未为修复重写这些断言。
- 工具在prepare before→完整并集after→408source after→最终strict之后全部13件恒等，current-tool-stability SHA **2803a7753f6783505b7af209a79d9e02714b0291680acc6f092d9be4c96c5ce3**；5交付文件Git EOL过滤后blob逐项等冻结HEAD、无脏文件，committed-input-audit SHA **6ff8f2ed3698bf973e1564382667e58a0bfa09b2195ca0243ccb0f432339783c**。
- **T21完成，F-941-06/GAP-941-AV-DATA-DIAG-01在工作阶段关闭**：V25..28/AC16的实际诊断、兼容/失败边界、完整职责消费者与当前身份严格闭环已具备，SD10候选留同树；T20/21均勾选，当前6/21（T16..21完成）。最终独立review仍T14按r4与冻结提交审，不以此内部工作核查顶review；未合入/归档/删除scope，原T01..15及全部未闭义务继续。无存活本包测试/采集句柄，下一步继续原required/ARCH/源域完整性与其余具名缺口，不重复请求F06授权。

stage: work | plan_id:PLAN-941 | plan_revision:r4 | outcome:in_progress（T21/F06工作闭环，原完整任务未齐） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:be7593e00a03edf8fd1985de8bfe49a5a3475429 | task_ids:T21完成，原T01..13继续 | evidence:1493完整门/408实际Case保8红/16诊断严格PASS/156反控/当前679P8F319U | blockers:原required/ARCH及未闭源域机制；无新增用户决定 | next_responsible_role:work 原完整T01/03/04/11所需证据适配/required和ARCH义务；T14最终review仍待

stage: goal | plan_id:PLAN-941 | plan_revision:r4 | round:52 | from:work | to:work | status:executing6/21 | outcome:in_progress（F06关闭，完整目标保持） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:be7593e00a03edf8fd1985de8bfe49a5a3475429 | next:work 原完整required/ARCH/证据；无待poll句柄

### 第53轮：原记录/文本28格严格同源接线，required102/313（2026-11-06）

- 入口复核main r4 executing6/21、be7593e00同树clean/分支归属正确、main无代码WIP，无待poll或待授权；本轮继续T01/04/11的原义务适配，不新增producer修复。冻结提交 **880c1a94d5251e7c721c564927297459bb5fa588**，7文件；新增只读ace_m1_legacy_data.py、组合接线、旧map/registry一致更新、治理反控和architecture候选。原T01..15仍未完整闭合，完成计数保持6/21。
- **独立对应关系**：旧冻结binding必须以namespace/ID/target取行，不能把同名scenario:R01当corpus:R01；manifest源字节、现行独立source合同与原required目标同时一致。有限14例：R01..06、T01/T02/T04数值原19/9/23/51/42/21/38/27/128512；T08原slice-oob映射TextIndex；R09 nominal-type→source.type-mismatch、R10 use-after-move→verify.use-after-move、T06 borrow-escape→verify.borrow-escape、T07 invalid-scalar→text.invalid-scalar。仅AC/AV共28格，M01..12不在旧131行中，不拿同返回值方法变体替R07/R08/T03/T05，不碰legacy-AVM/A2R。
- **事实重新判定**：接线只消费原严格data/diagnostic reader已核验当前工具、原Case/source/request/response/owner/profile/caps的事实，重新按旧独立oracle比较实际值/具名trap/结构化code与目标stage，保留来源slot及请求/响应SHA。原readerFAIL或父Case非PASS不能转绿；诊断无权威仍U、pending不塞execution_summary。T08 native仅实际RT-FAIL code=-10且exit70支持具名越界；通用70不命名trap，实际成功返回70也不替代T08。现data profile是AV显式目标内同源执行，不冒称旧owned recipe内部已验证，原资源/生命周期/场景义务另算。
- **发现表不冒执行**：新增28个精确formal locators，旧304 entries逐项原样，现332；原313cells/251已绑定/62未绑定、旧38 cert-gap、批准source/expected/NA/accepted/future/样本/预算与全部其他字段原样，只有这28格reader_status和adapter gap转supported。表仍execution UNOBSERVED/executed0/mapping_complete=false。map SHA **2e37ca8851e440e69344f750909acf28bf5615df7c8ee799e185db2be78fe159**；registry只同步该合同指针。
- **定向反例与正式门**：新增7永久测试（6 join +1 locator），覆盖同名命名空间/方法变体排除、错数值/bool数值/同值异源/错schema或hash/声明oracle更改、父FAIL/源FAIL/U、诊断无权威/错code或目标stage、伪message、generic70/错runtime trap、缺原slot、exact新locator不可由旧同源locator冒替。冻结后正式`cargo t infra-ace-m1-arch --out <scope>/legacy-data-frozen-governance/gate`，**192/192 PASS，0skip/0error**；内部整7.181279s，原capture整7750.4141ms，exit0。results SHA **d8867fe020189654ba1b6471928408f636db77ad946bad3752c600d0187ac4bb**。Python观察层改动不影响生产工具/语义，不重复上轮1493编译/性能门或408实际源执行。
- **当前冻结全组合**：沿第52轮真实before/after与composed-bindings-rt-views.json，原reader/Popen禁用、同一LiveBudget48MiB/原4MiB输出等帽不变，exit0。`legacy-data-frozen-preview/preview.stdout` **610285B / SHA c8a0b7553e6a0fdf5ba249c68c9532b8bf09aa65d101568d5c82a22ed13e49e0**；14774.5970ms，进程29822976B/Job31076352B，全时峰**48478012B<50331648B**/保留30351848B。开发预览也通过，但不与冻结结果重复计为义务执行。
- **守恒审计**：现**707P8F291U /1006登记/24合同**，仅原28格U→P，其他978项判定原样，原74条已观测对象逐项一致；原required **102P/211U/313**。data/diagnostic/certification和ARCH源报告逐对象不变；13工具/库当前SHA全部与第52轮真身份相同；7提交文件Git过滤后blob逐一等冻结HEAD、树clean。`legacy-data-frozen-preview/closure-audit.json` SHA **53e8adf1e02162c4ecb8ff794a714baf9791dc5b24870f3741655c83d550ce4a**。0新producer/guest执行；资源/场景/旧认证不由数值或定位器推断。required_universe_closed、acceptance_system_delivered、can_claim_expanded_m1仍全部false。
- 继续原T01..13：旧211U、ARCH49U、源域其余31U与8实际scalar红；D938/939尚未交齐、旧38cert-gap及T13健康门继续保持；最终独立T14与合入T15未开始，无新用户决定/未删除在途scope。下一步盘点旧场景/ARCH真实producer观察边与剩余同源义务，按原预算接线，不用注册名或当前PASS摘要核销。

stage: work | plan_id:PLAN-941 | plan_revision:r4 | outcome:in_progress（原record/text28格证据闭合，完整任务未齐） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | base_commit:be7593e00a03edf8fd1985de8bfe49a5a3475429 | code_commit:880c1a94d5251e7c721c564927297459bb5fa588 | task_ids:T01/04/11原required部分适配 | evidence:192正式治理PASS/707P8F291U/102原格位/28守恒变化/13工具稳定 | blockers:其余required/ARCH/具名源域和健康门未齐；无新增用户决定 | next_responsible_role:work 剩余同源义务、场景与ARCH真实观察边

stage: goal | plan_id:PLAN-941 | plan_revision:r4 | round:53 | from:work | to:work | status:executing6/21 | outcome:in_progress（原required74→102；原尺度保持） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:880c1a94d5251e7c721c564927297459bb5fa588 | next:work 原完整required/ARCH/证据，无待poll句柄

### 第54轮：CT真实观察接口接线，完整CT职责门仍运行（2026-11-06）

- 上轮原28格实际闭合及707P8F291U为progress；入口复核main r4 executing6/21、同树880c1a94d clean/归属plan-941-dev、main无代码WIP，继续T08/T06真实机制观察。源CT四端值与认证已齐，但原reader明确未观察execute/hit/ExecUse/restore，不能把42→43数值差直接冒ARCH通过。本轮未修改生产CT语义、core/HIR/worker或协议。
- 冻结新增永久观察测试提交 **5b3501b2aeae0b8ba7845f7afd8fca3726220166**，仅`experimental/ace/crates/source-cte/tests/architecture_941.rs`，自动进入原ace-comptime固定职责组。直接调用既有compile_scalar_ct/CtService/MemoryEnvelopeSink及原run_entry_on_av：原sum42/sum43 include_str字节、fn main后缀完全一致；空sink且forbid AV先实拒、ct_execute0/publish0；新空sink冷态真AV=1/publish1/正steps/checked owner42；forbid AV下相同源真hit而execute不增长/owner42；callee edit在forbid下拒且不发布；恢复AV后新key/execute2/publish2/owner43。原执行日志的key/attempt/from_cache/steps/wall与原计数逐态导出；没有第二执行器、host fold或从最终值倒填计数。
- 可选导出只在显式AUTO_PLAN_941_CT_OBSERVATIONS提供时生效，原断言无env仍执行；scope规范化核对、create_new拒覆盖、16KiB固定小报表及sync_all，字符串按hex输出、无新增serde依赖/锁文件变动。已运行原nextest新目标：1/1 PASS、0skip、真实exit0，测试0.023s/准备4.59s（外命令5.5128064s）；原core first_lex_fact warning保留，T13未冒健康完成。rustfmt与diff --check通过，冻结树clean。
- **完整职责门正在运行，未计PASS**：scope/capture-ct-source-mechanisms.py先正式prepare/list该唯一目标，冻结debug test binary **a77fa2fdddb8f546efac9334e08403fc5466c966cefd133e5a277cd1d879c654**；随后原`cargo t ace-comptime --out <scope>/ct-source-mechanisms-frozen/gate`，原ignore/平台/资源/性能/样本保留。句柄 **session84298**、capture Python **PID1912**、fixed driver Python **35420**。20:30实核子进程 **11612**在跑原`cte_resources_tests::performance::resources_real_watermark_gc_physical_delete_and_converge`（CPU196.98s、工作集29245440B），非仅锁文件推断；不重启、不改冻结source、不并行其他build。正式raw观察文件尚未生成，不能提前读/声称strict接纳或新ARCH verdict。
- 准备的220件候选source pins真before SHA **f76fd1e8b999c839f411c747f729dbc4e44f5eb522e240e1977d42c90d4cfef2**，其范围不是完整依赖闭包。另递归现Cargo path依赖取得**16本地包/557跟踪输入**，逐实际文件SHA与Git EOL过滤后blob核冻结提交一致；`ct-source-mechanisms-frozen/source-closure-committed.json` **140140B / SHA 1f87a4ba62836bd52ba395bd43612a2c1de276f0989455fffe2da408035b0907**，明确phase为从冻结已提交输入导出，绝不改名/回填成before。其只读派生句柄9156已terminal exit0。正式driver仍负责实际窗口全代码/config/input source_state守恒；完整门terminal后还须after及二进制/13工具稳定审计。
- 本轮观察适用于ARCH-04 cold-real-av-and-hit与callee-body-change候选，不证明managed-store恢复、Use页深验、meta/真实实例/资源/四端冻结HIR或其余ARCH；strict父reader/独立错计数/错log/错source/伪PASS/原始terminal等准入仍待接线。**当前有效报告仍第53轮707P8F291U，原102P/211U/313、ARCH49U及所有claim false，任务6/21不变**。8源域red/38旧cert-gap/D938/939与T13继续具名未闭，无新增用户决定。
- 下一轮必须先续**同session84298**至真实终态，再读gate/results、原观察raw、current input/binary/tool audit；不得因观察超时另起门。若原门完成并稳定，按原48MiB预算使能CT专用strict reader/原始反控与组合关联；未观察的其他机制继续U。

stage: work | plan_id:PLAN-941 | plan_revision:r4 | outcome:in_progress（CT真实观察永久测试1PASS；完整原职责门live） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | base_commit:880c1a94d5251e7c721c564927297459bb5fa588 | code_commit:5b3501b2aeae0b8ba7845f7afd8fca3726220166 | task_ids:T08/T06机制观察部分 | evidence:原源冷AV/hit/forbid/callee新key回归；16包557冻结输入；session84298真实live | blockers:完整CT职责终态/strict reader尚未齐；无新增用户决定 | next_responsible_role:work 先poll同session84298，不重启或并行新门

stage: goal | plan_id:PLAN-941 | plan_revision:r4 | round:54 | from:work | to:work | status:executing6/21 | outcome:in_progress（真实CT观察接线，原门live） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:5b3501b2aeae0b8ba7845f7afd8fca3726220166 | next:work 同session84298/PID1912与35420，终态后strict准入

### 第55轮：续同CT完整门；真实压力转场与读取规则反控准备（2026-11-06）

- 第54轮新增永久真实观察测试与557冻结输入核验为progress。本轮入口重读main r4 executing6/21/末receipt，同树5b3501b2a clean、main代码无WIP、worktree归属正确；继续同session84298，未重启或启动其他编译/测试。原capture Python1912/fixed driver35420均仍live，不以旧锁或历史意图推运行状态。
- **已验证运行转场**：原PID11612在原物理GC水位/删除/收敛压力用例中CPU持续增长（20:34末436.109375s、工作集30400512B）；其后该PID缺失，20:35:46实际新PID**35764**进入原`cte_resources_tests::performance::resources_thirty_two_mib_pages_and_512_distinct_revisions`，session84298仍live。只证明前项进程已终结/后项运行，不推前项PASS或整个门terminal；原parent stdout/results尚未最终落盘。完整discovery实际135用例/0ignored，含新`ace-source-cte::architecture_941`唯一测试且filter matches；列出不算执行。
- 原nextest配置对`::performance::`占满当前测试池，故后继包括新观察测试须等原压力原子项，不减少64MiB水位、32MiB页或512修订，不改线程/ignore/样本/截止。原观察export尚未生成，不能提前填strict证据。正式门含编译准备/原完整测量；同树source/config持续冻结。
- **只在统一scope准备读取草稿，未写实施树**：`draft-ct-mechanism-reader.py` SHA **3c1f024ae291bfbe0b956e76266d5355c9a44ce816fbd06eb8981ca89ce8be8b**；逐独立原source字节/不变main后缀、五态真实计数/原key关联/正steps/发布与forbid负控/checked-owner42、42、43判定。实际服务hit log的attempt=0/from_cache=true，steps/wall保原executed页面值；不能错当本次新AV工作或伪造0步。该事实来自现service.rs实际实现，终态后仍须对原始payload验证。
- 草稿**28合成反例控制全部符合预期**，均明确0真实执行/不入组合：合法保留hit旧steps；错冷计数、伪hit标记/attempt/key/outcome/steps/wall、编辑冒hit/旧key/零steps、fake counters无log、陈旧43/两者同错值、forbid码不符；bool计数/owner、额外PASS字段、旧源码/旧deck sink/缺态必须拒。`ct-draft-countercontrols.json` SHA **9e935ff312df566c4dbd1c0e74db97227086a0e24906dfc64e0bb61b79899150**。只有纯规则及py_compile/合成控制，不冒已使能正式reader或ARCH执行通过；producer source闭包与原native terminal关联仍须完整接线。
- 顺带只读核依赖现态：938推进到executing6/13、分支af26e0d47，尚未D；939仍drafting0/14，不能消费开发分支冒已交付或由941接走其功能。无新用户决定。
- 当前有效全组合继续**707P8F291U /原102P211U313/ARCH49U**，完成6/21、全部claim false、原8red/38cert-gap/T13健康与其余未闭任务保持。下一步先续同session84298（capture1912/driver35420，当前压力子35764）至真实terminal；再核after/原始观察/13生产工具/完整native结果。没有重复运行门，没有写入冻结生产或交付文件。

stage: work | plan_id:PLAN-941 | plan_revision:r4 | outcome:in_progress（verified_wait同CT原压力门；28纯草稿反控准备） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:5b3501b2aeae0b8ba7845f7afd8fca3726220166 | task_ids:T08/T06原机制观察 | evidence:session84298/1912/35420/35764实核live，135发现/0ignored不冒执行；28合成控制 | blockers:同门仍运行、原raw/strict准入未齐；无新用户决定 | next_responsible_role:work 同session84298，不并行build/不改冻结源

stage: goal | plan_id:PLAN-941 | plan_revision:r4 | round:55 | from:work | to:work | status:executing6/21 | outcome:in_progress（verified_wait及读取规则反控准备） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:5b3501b2aeae0b8ba7845f7afd8fca3726220166 | next:work 先poll同session84298/capture1912/driver35420，当前原32MiB512修订压力子35764

### 第56轮：同CT门继续转场；完整原始native准入草稿备妥（2026-11-06）

- 第55轮为verified_wait并有28条纯规则反控准备。本轮复核main r4 executing6/21/第55轮末记录、main代码无WIP、同树5b3501b2a clean与worktree归属正确，继续**同session84298**，capture1912/fixed driver35420实际仍live。没有重启/新编译/并行测试，没有改动冻结代码/config/输入。
- 原32MiB/512修订压力子35764已缺失，随后20:40:22新PID3580进入原`cte_turnaround_tests::performance::cte_turnaround_injected_spike_control_is_red`，其实际worker子20980也被观察到；之后这两个PID均结束，20:45末实际新子**24544**进入`cte_turnaround_tests::performance::cte_turnaround_resident_degrade_gate_and_control`。每次均用实际进程/commandline加同handle轮询核实，不把转场解释为前项PASS，也不把暂无输出解释为结束。原完整135用例/0ignored仍在执行；新CT observer的raw导出尚未生成，结果未计绿。
- 在唯一scope内继续准备**draft-ct-native-admission.py**（**8063B / SHA 420fd705d400e529427793a282d4eaf64c385e53f4a81d4b8b38dbe0c9c1f76d**），仅草稿/py_compile通过，未写入实施树、未对真实本次结果宣称准入。规则列明：独立557 producer输入当前SHA；真实before/after子集相等而不冒完整before；冻结clean source_state窗口；当前测试binary与真实prepare/list及after关联；实际唯一测试选择/非ignore；复用现父侧纯selection/terminal parser前先严格UTF-8/重复键JSON核验；完整原group命令/argv/cwd、原始run.stdout逐项终态/执行数与command/meter，而不是group PASS或executed摘要；16KiB CT payload在解码前核帽。旧组其他失败与本测试事实分开保留，不删除失败或变造全组绿。
- 同scope生成**ct-source-mechanisms-contract-draft.json**（**77739B / SHA e360662ae12ed50d7955bfb8ec5e06418f53cfc79890005842f7663e9a056506**），含两个现有ARCH原子义务ID、实际producer测试/557输入pins/原sum42与sum43字节身份、固定独立42/42/43及冷/hit/编辑计数规则、禁止AV码与明确scope。合同自身仍execution UNOBSERVED，不登记新义务/不认managed恢复、meta/实例、resource或四端HIR。待原门terminal、binary-after与actual payload/原始反控齐备后，才考虑正式落树并联读；本轮不对仍运行测试改代码。
- 当前唯一有效组合保持**707P8F291U /原102P211U313/ARCH49U**，全部claim false，任务6/21与原8red/38cert-gap/D938/939及T13健康缺口保持。下一轮先续session84298至真实terminal（capture1912/driver35420，当前驻留退化控制子24544），再核本次source/binary/13工具after与原始终态；不能因长运行重采同门或提前填写PASS。

stage: work | plan_id:PLAN-941 | plan_revision:r4 | outcome:in_progress（verified_wait同CT正式门；native准入与合同草稿准备） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:5b3501b2aeae0b8ba7845f7afd8fca3726220166 | task_ids:T08/T06原机制观察 | evidence:session84298/1912/35420/24544实核live；完整native准入和77739B合同草稿 | blockers:原门未terminal、raw/after/正式reader未齐；无新增用户决定 | next_responsible_role:work 同session84298，不并行build/不改冻结源

stage: goal | plan_id:PLAN-941 | plan_revision:r4 | round:56 | from:work | to:work | status:executing6/21 | outcome:in_progress（verified_wait及原始native准入准备） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:5b3501b2aeae0b8ba7845f7afd8fca3726220166 | next:work 同session84298/capture1912/driver35420，当前驻留退化控制子24544

第56轮交接补充：只读draft静态预检发现原220-input before比真实557-input依赖闭包多13个syntax-core兼容层文件（该层不在实际本测试依赖闭包）。当前native草稿的before⊆closure条件将拒绝这份合法保守快照，因此草稿尚未ready；后续须显式认领snapshot-only pins或声明570-pin保守并集，不能删/改旧before或冒13件为实际语义依赖，也不为metadata差额重跑有效执行。具名预检`ct-native-draft-preflight-gap.json` SHA **71986c93913c3990c4cee2fef89592b13cdc4b60704291a7a5691da31e5cf659**；无新增用户决定，原gate仍同session84298。本补充不改上述历史草稿hash与未准入状态。

### 第57轮：CT门真实134P1F收尾；两项ARCH机制严格闭合，709P8F289U（2026-11-06）

- 第56轮为verified_wait并产出native准入草稿/发现13额外快照pins，构成准备进展。本轮重读main r4 executing6/21、同树5b3501b2a clean/归属及main无代码WIP，先续**同session84298**；实际经过wire-mutation性能控制，随后原完整门terminal。capture1912/fixed driver35420均已结束，原session不再live，不重启该门。
- **完整原CT职责终态**：实际**135执行/134PASS/1FAIL/0原条件排除**，全部原64MiB真实GC、32MiB页/512修订及原性能/注入/驻留/线帧反控原样跑完。父exit1、gate_passed=false保留。外capture整**1372673.4950ms**（含准备，Job2306195456B）；内部discovery准备17584.7429ms/Job2280448000B，实际run1354368.9243ms/Job737943552B，准备与运行不混成单例runtime峰。results SHA **ff1a30de2a6d5b26b516934dacfeeaa39955666213c730ec555a944e329de9f1**；run.command SHA **6a38b31c8fffb98c646a1b5f9167ae3394a9d72ef0b30cdb8d112995667ccd89**。新CT observer自身真实PASS/五态raw已生成，不以其通过顶完整门。
- **新增具名健康缺口CT-DEADLINE-01，尚未修复/未授权修生产**：原`cte_resources_tests::cte_resources_request_deadline_fast_control`在ct_execute>=1断言失败（0.5779634s）；前面的resource.deadline/status/非language断言已过，因此实际记录ct_execute=0。冻结源不改再走正式`cargo t ace-comptime --test-name cte_resources_request_deadline_fast_control`隔离重放，仍**0P1F、0.3778119s**，134未选不充PASS；整2795.9133ms，真实父exit1。replay results SHA **2bd6657d97dbde4bc8e17b738685fad5508a2891f1f042d4d53ca08dae9aa177**。不能只归因并发。driver已有构造期Core/Record预热与300ms累计预算，原source-AV到达后才计执行；尚须实核是AV前预算用尽、计数丢失或其他原因，再给最小修复范围，不能先放宽300ms/降低负载/把0改成伪执行1。此红归T13/CT原职责闭环继续保留，不隐去或冒已关闭。
- **草稿快照差额已显式修正**：真实semantic闭包仍16包/557输入；原before额外13个syntax-core兼容文件只作snapshot-only，另加8份正式组路径依赖manifest绑定Cargo特性合并配置，共**578固定pins**，不冒这些额外件为实际CT语义依赖。新跟踪合同`ct-source-mechanisms-contract.json` **82545B / SHA a89aa249648f3960a03efe9f5e035fd46b86242e178d145d1094fd2c421cfc72**，原两before/after均不改。合同自身execution U、现有49 ARCH分母不变，只登记两个已有义务的独立数值/forbid/key/log规则。
- **真实准入先正确拒绝旧准备配置**：最初单包manual prepare的debug产物`architecture_941-8bfd72fc5b113f18.exe` SHA a77fa2fdddb8f546efac9334e08403fc5466c966cefd133e5a277cd1d879c654，与原完整组Cargo特性合并实际使用的`architecture_941-362b9c89c90c4c8c.exe` SHA **3ad9ab34de074cba6414c5353f58b063ba03c9f6e9791406979a186f6b2426e1**不同；不是同名路径格式问题。旧134P1F/原raw/旧before保留，不用after回填before、不放宽身份检查。按正式完整Cargo包/feature/config union正确prepare，再只重采具名新observer，避免重跑不受影响原压力/性能。
- **实施冻结**：提交 **ff2a4e6e7af1f0efd4162add65e9c21f336e6e7e**，7文件（CT strict reader/跟踪合同/registry+纯组合+注册索引/7永久测试/SD06候选）；另**1548132986430e7298bcaaccf7f0d1554d79cb31**只补已有legacy_data reader的报告来源hash，未改生产/判定。readers不启动工具；原fixed parent's纯selection/terminal parser复用，先做严格UTF-8/重复键检查；源/578pins/当前binary/真before-after/冻结source_state/实际listing/精确named配置/原始terminal/command/meter均关联。cache hit的继承steps/wall不冒新AV；值/计数/log/forbid/新旧key独立交叉判定。发现表仍49U/executed0，真实执行另列architecture_execution，不把locator改成证明。
- **当前正式重采与治理**：`ct-mechanisms-correct/`在ff2a冻结clean下按正式组配置先prepare正确3ad9二进制、578真before→具名原observer→真after，全部相同；新observer**1/1 PASS，0.0182436s**，外整2055.6717ms；listing135中134未选，不计通过（本版nextest不为未选项输出ignored terminal，native_unselected数组为0，收据按discovery与selected集合差计算134，未改源报告）。gate SHA **6ca4353439e44255c04b3880f0e35dda4f3a536375350fea2548e4658470a390**；raw SHA **2ff9b262f81547d87949e443379b50d8c7f70a1d44ef7b86d463e1e3db814c87**，与早次raw同字节是当前实际再次运行的结果，非复制。原13生产工具/库逐当前SHA稳定。
- 正式`cargo t infra-ace-m1-arch` **199/199PASS、0skip**，整5.2091796s/外5731.0211ms；results SHA **97510c9ec8dffd40de55f3331b4d4536b8a5861f91e96318d046472b1f9f8e35**。新7条永久反控：真实same-source但错值/计数/缺log、hit伪新增执行/旧key编辑/零步、false typed/旧deck/source/额外PASS、父PASS却无raw终态、错实际binary路径/字节、实际失败terminal保FAIL、错误source_state/选择/重复键/16KiB解码前拒。之前28合成反控不计真实执行。最后来源hash一行只改报告元数据，复用身份不变的199证据，不为提交再跑135或199。
- **冻结当前全组合**：绑定`ct-mechanisms-correct/composed-bindings.json`，原Popen禁用/所有原reader/同一LiveBudget48MiB与4MiB输出等帽，exit0。`ct-mechanisms-frozen-preview/preview.stdout` **615074B / SHA c77a1d16ed86d5c551c191f48c0a53b9f39a91fc1ac336f6d978407f9e6b4706**，11435.6825ms、进程30777344B/Job32014336B，保留30504083B/全时峰**48602931B<50331648B**。现**709P8F289U、1006登记/25合同指针**；原required仍102P211U313，source其余31U、ARCH **2P47U**。仅cold-real-av-and-hit与callee-body-change两格U→P，其他**1004 verdict原样**，旧legacy对象逐项相同。不能用这两格替CT managed恢复/required页、meta/实例、resource/四端HIR等其余机制。
- 守恒/当前7提交路径blob/13工具/原完整134P1F及隔离红均折在`ct-mechanisms-frozen-preview/closure-audit.json` SHA **0c2f86c77981135cdeaffb155b8375e3fc416467cc44d9c4e5811803967b5b11**。closure辅助收据首次误把native_unselected数组当全部134未选项而拒；按原discovery135−selected1修正收据计算后复用同一已有效冻结组合raw，不重复运行组合或工具，不变任何正式源结果。required_universe_closed、acceptance_system_delivered、can_claim_expanded_m1仍全false。当前无live测试/采集句柄；任务保持6/21（原T01..15仍未完整齐）。下一步诊断CT-DEADLINE-01实际阶段/最小修复范围，同时继续其余原required/ARCH/源域证据；8 scalar red/38旧cert-gap/D938939/健康门保持。

stage: work | plan_id:PLAN-941 | plan_revision:r4 | outcome:in_progress（两个ARCH实测机制闭合；原CT完整门另有真实红） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | base_commit:5b3501b2aeae0b8ba7845f7afd8fca3726220166 | code_commit:1548132986430e7298bcaaccf7f0d1554d79cb31 | task_ids:T08/T06两个机制/T11reader部分 | evidence:完整135=134P1F/隔离1F/正确配置observer1P/治理199P/当前709P8F289U/1004守恒 | blockers:CT-DEADLINE-01原因待实核与其余完整义务；无新增待用户决定 | next_responsible_role:work 诊断原300ms deadline计数失败、继续其他真实证据

stage: goal | plan_id:PLAN-941 | plan_revision:r4 | round:57 | from:work | to:work | status:executing6/21 | outcome:in_progress（CT两个ARCH闭合，完整原目标保持） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:1548132986430e7298bcaaccf7f0d1554d79cb31 | next:work CT-DEADLINE-01阶段诊断与其余required/ARCH；无待poll句柄

### 第58轮：CT-DEADLINE-01定位为计数测试的时序前提错误；F-941-07最小提案（2026-11-06）

- 第57轮两项ARCH实测闭合与当前709P8F289U为progress；本轮入口重读r4 executing6/21、同树154813298 clean与worktree归属、main代码无WIP。没有live测试待续；只诊断既有快速截止控制的两次真实失败，不改生产driver/计数/预算/源码oracle，不据重放偶绿关闭问题。
- **同库真实阶段探针**：只在scope调用当前既有public CteDriver与原`baseline_source_bench(115,0)`，原wall_deck生成函数逐字复用（4distinct calls、267218B、300ms、native off），显式新空store，无复制编译器或AV；使用当前唯一预建libace_comptime-dd3d00d39c760d33.rlib，run前后哈希恒等。普通调度实测project173.2839ms→AV span1205.8118ms，deadline资源态/非language、ct_execute1、不发布；原默认限额单call合法正控1080697、真实AV1、发布成功。第一次手工rustc链接缺windows.0.52.0.lib，原失败capture保留；仅补现有x64 native链接路径与项目rust-lld/32MiB栈后诊断build通过，非语言/CT缺陷或认证通过。
- **排除“分配跟踪器单独导致”假设**：原conformance确实有全局TrackingAlloc；把完全相同allocator复制入诊断调用方（只是诊断代码），其他库/输入/300ms全不变，project222.2337ms、AV span1390.2312ms，仍ct_execute1/资源态/不发布。它增加开销，但仅凭此不能归因为唯一原因。
- **原测试精确现场**：在唯一树临时只插入outcome输出（先eprintln、后独立具名trace文件）且原115helper/4call/300ms/全部assert不变；两次实际具名测试均通过，trace显示project**279.1709ms**、AV span1332.2566ms、wall1623.2449ms、ct_execute1。计时/执行判断发生在打印前，不把这次绿涂成原2红已修；其接近300ms表明该测试依赖现场调度。临时文件每次finally逐字节恢复原SHA、树clean；完整源与断言从未提交改动。恢复后又正式`--discover-only`重建原conformance二进制（整2389.6582ms，执行0，不充PASS），避免留下诊断产物冒当前原测试。
- **决定性阶段反证**：只对诊断进程自身设置单核affinity并加一条自有竞争线程，其他进程不改；其余仍原同库/115helper/4call/267218B/300ms，竞争在query返回立即停止并join。实际project**383.5717ms**、wall**386.5982ms**、cache None、**没有av_execute span/ct_execute=0**，resource.deadline/非language、**published=None**；AV前已用尽累计预算，0是真实而非丢计数。恢复无竞争后同原默认单call合法正控仍1080697、AV1/发布成功。原快deck在普通与竞争两目录字节完全相同，SHA **e5fcb26e665afc3bb590dd4ee1d006ec735382c739773a55ae86f03806c0325c**；原driver loop进入首AV call前`if dl.exceeded(){return dl.diag(...)}`与现场精确一致。
- 根因是原fast控制把“300ms累计资源超时”误推成“必已进入AV且ct_execute>=1”；前端耗时/调度稍变即可在进入AV前正确拒绝，原2红不宜归为生产计数缺陷。也不能把AV前过期伪记成1，或放宽时间、缩源码/查询来保这个前提。诊断audit`ct-deadline-restored-discovery/diagnosis-audit.json` SHA **43b4d5108191baaf81e292bfb84c2b3ab76e8cc7f483f008a4804da6cadcce22**；决定性raw`ct-deadline-contention-diagnosis/run.stdout` SHA **3f497322fdf6058721acba8cb6c85bcad4e6ecf569a9d4591153ecd2379f8960**。所有578当前CT输入与13生产工具/库恢复后SHA仍恒等，无存活探针/负载线程。
- **具名F-941-07最小修复提案（待用户授权，不在本轮实施）**：在941新增test-only phase，主要触`experimental/ace/crates/comptime/tests/cases/cte_resources_tests.rs`，必要的私有确定性计数回归仍用原driver/AV边界；不改生产driver/公共协议/计数规则。让原300ms/115helper/4call资源控制按真实阶段核计数（AV前耗尽=0，实际进入执行的合法路径必须有真实执行事实），补明确AV前拒绝与实际AV超时/失败计数的区分、漏记/伪造计数反控、无发布与合法恢复。原`cte_resources_request_wall_real_ten_seconds`的32查询10s累计负例及同规模4调用正控原样保留，用其实际AV与发布断言保原执行义务，不能靠修改fast断言删除AV侧保障。验证走具名新/受影响deadline控制与原10s用例，未触的GC/32MiB512修订/原性能证据保持来源和历史状态，不为test-only更改立刻重跑整135压力；最终T12完整冻结并集仍待，当前134P1F原记录不重标绿。
- §5.4明确“已归档能力发现新缺陷默认走具名独立修复流程”，不能把F04/F05/F06具名phase扩成任意修复授权；将询问用户是否再加F07 phase（或独立修复）。这里只提交可审提案，没有改变旧验收或任务数量。若授权，按new技能修订r5再执行T22/23；未答复时继续其他原required/ARCH证据，仅F07修复不动。
- 当前有效组合仍**709P8F289U、原102P211U313/ARCH2P47U、6/21、claim全false**。本轮无交付代码commit；临时诊断源已还原、原binary恢复prepare、所有原真实红/guard/D938939/健康门仍保持。下一步F07授权若到则r5→work；否则继续其余合法证据适配，不因为部分待裁定停掉整目标。

stage: work | plan_id:PLAN-941 | plan_revision:r4 | outcome:in_progress（CTdeadline时序前提原因已实证；F07最小修复待裁定） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:1548132986430e7298bcaaccf7f0d1554d79cb31 | task_ids:T13诊断/T08真实分态 | evidence:原源/同库有AV1与前端383.57ms导致AV0对照、原test临时trace并恢复、578/13身份恒等 | blockers:F07新phase授权只阻该修复；原其余证据仍可推进 | next_responsible_role:work 其余证据；授权到达则new修r5

stage: goal | plan_id:PLAN-941 | plan_revision:r4 | round:58 | from:work | to:work | status:executing6/21 | outcome:in_progress（原因实证与最小提案，完整目标保持） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:1548132986430e7298bcaaccf7f0d1554d79cb31 | next:work 其他原required/ARCH；F07授权到达则new→work，无待poll

### 第59轮：CT meta/实例真实观察；新发现F-941-08能力判定在执行/命中后缺失（2026-11-06）

- 第58轮实证CTdeadline时序前提并提交F07提案为progress。本轮复核main r4 executing6/21、同树154813298 clean/worktree归属、main无代码WIP；F07无答复，仅继续独立T08机制证据，不修改其测试。当前938分支推进37e1e584d但尚未D，939仍未交齐。没有其他live测试。
- **真实SDK路径而非值回推**：scope内薄Rust调用方直接使用当前预建`libace_source_cte-a6f9ae4b88c33d2c.rlib`的compile_scalar_ct/CtService。RecordingSink只委托原MemoryEnvelopeSink的probe/publish，在成功publish后记录原ExecKey；不复制store/解释器/solver，不造计数。原meta2/meta3/pick40从跟踪源读取相同字节，新探针与输出仅在统一scope。原库/三源run前后hash恒等、578当前合同输入/13工具最终再核全部相同。此为诊断/接线原型，尚非正式native contract准入，不改全组合verdict。
- **正例与失效事实**：同service/storage下meta2 field-set x,y：当前meta_queries1、真实AV1/publish1、值42；forbid AV重取原源hit，AV仍1/hits1；同站点改三字段在forbid下精确av-forbidden，未新执行/发布；解除forbid实际43、AV2/publish2，callee逻辑/签名/体版本不变而实参canonical从i32:2,i32:40变i32:3,i32:40，执行key变化。HasField独立变体原无z→加入z使实际AV参数false→true、结果42→40，原站点不改；每代各一次真实AV，非meta-only冒执行。
- **实例/子CT组合实证**：原pick40只改flag return true→false，模板及main站点字节不变；同service累计真实AV2→4、freeze2→4、结果40→42，随后forbid下两级均hit而AV保持4/hits2。实际InstanceRecord的logical/key/模板signature/body轴仍相同，这是未改模板声明的正确事实；有效外层pick_i32 ExecKey的callee_body_version从a5d59665…变9129ba48…，inner flag体版本也改变。不能错误地要求raw template InstanceRecord键变化，或只凭返回42认定checked-child有效实现已换；正式reader应分别核实例声明与回灌后执行实现轴。
- **F-941-08反证A（冷态缺meta cap仍执行/发布）**：原meta2仅授`ac.source.cte.scalar/v1`、未授`cte.meta.fields/v1`，compile_scalar_ct仍返回OK/42、meta_queries1、ct_execute1/publish1；新空service独立重放相同。HasField变体缺meta cap也OK/42并真实AV1/publish1。未授CT cap的兄弟负控则在前门Diagnostics/source.unsupported拒绝、AV0/publish0，证明不是request配置始终被忽略或探针把错误当成功。
- **反证B（暖态撤销能力仍命中）**：在有cap成功生成原页后，同service设置forbid AV、去掉meta cap再请求原meta2，仍OK/42，meta_queries1，AV维持2但hits从1→2；权限未在命中前重新判定。不能用给cache key加一轴代替缺cap冷态/暖态的请求准入。
- **普通名称兼容与受保护冲突边界**：仅CT cap、用户声明普通`fn ct_field_count(x int) int { return x+7 }`，调用35实际42/meta_queries0/AV1，这是应保的普通函数语义；不允许按裸名字全禁。加入meta cap时相同用户声明最终Diagnostics/source.cte.bind、内含原meta-name-conflict，但已先真实AV1/publish1；说明现有产品顺序确实先调度执行/替换，再发生能力/保护名绑定，不宜把最后的错误当执行前拒绝证明。
- **权威约束与原因定位**：当前规范`docs/specs/auto-cte/source-query-integration.md` §1明确meta capability default CLOSED，未授时名字普通；源码SHA **eafa1082c933454dac0adeec25d70937c13c5a1a71a65e710304935c8526c208**。产品product.rs SHA **a92d96c429931fafc5414987ace0707d02f727e4da365a607dd18b4015eff0a1**，先serve_program→substitute_all，再bind_with_facts(capabilities)；未授权meta需求已被换成literal，原core meta_enabled检查看不到它。不是组合reader可修的缺字段或批准域变化，不能把未授cap变成自动支持。
- 三次真实原型raw分别`ct-meta-instance-diagnosis/run.stdout` **12887B/SHA48fff635513acd7f145a3e7462e2b1f1682d284495fd0275eece3025f9c1cb3b**、v2 **15333B/SHAb2b55af3b858cb3e3f71b34952bfd125ba65716f0e7eb1d3826c2437f97f6462**、v3 **16363B/SHA c3ee9c20b12f2e708ae9edd8b413c8effed468f3a13989c783962b3ad58d4dd0**；最后inputs-audit SHA **b945340109f507e49e4f0deb22270083519e8e5f516819f2d1f3a6dfee15ab0a**。v1 build545.6101ms/run159.4188ms，v2 build531.7574ms/run149.6324ms；退出0是探针完成，不冒全部语义PASS/未授cap负例通过。所有raw/源码副本/受管链接产物留单scope，无树代码写入/无新编译器runner或meter。
- **F-941-08最小提案（待授权，不在本轮实施）**：在941新增具名source-CT meta能力修复phase，主要product.rs及既有调度/绑定需求边，复用既有前门/语义规则在执行及缓存命中前核验meta能力/保护名称。保无meta cap下普通同名用户函数、合法FieldCount/HasField、普通CT/实例/子CT、原code/stage与预算/缓存分态，不在组合器屏蔽或改expected。补冷态缺cap、暖态撤销cap/forbid、普通同名函数/启用cap冲突、合法meta与真实AV实参/失效的永久回归；原真实四端consumer与source/cert/Daily等按实际身份改变范围重验，未变输入身份严格复用。该项是936已归档产品能力的新缺陷，按§5.4不能自动套F07 test-only提案或F04/05/06授权。准备询问用户新增941 phase或独立修复；未授权时仅不动F08生产代码，继续其他证据。
- 本轮无交付代码commit，当前有效组合仍**709P8F289U、102原格位、ARCH2P47U、6/21、全claim false**。meta/child新事实尚未正式合同/原始准入，不擅自将另外2格计PASS；F08证据单列具名缺口，原两CT格位/全部原red不改。F07/F08只分别阻其具名修复，非整941待答即停；下一步获授权时按new修订，未到答复继续其他原required/ARCH合法接线，无待poll句柄。

stage: work | plan_id:PLAN-941 | plan_revision:r4 | outcome:in_progress（meta/child真实观察及F08能力缺口定位） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:1548132986430e7298bcaaccf7f0d1554d79cb31 | task_ids:T08 meta/instance真实机制/T13缺口 | evidence:同store真实key/参数/有效版本；冷缺cap/暖撤cap/HasField与普通名正控；578/13恒等 | blockers:F07、F08分别待具名修复授权；其他证据仍可推进 | next_responsible_role:work 其他原required/ARCH；授权到达按new→work

stage: goal | plan_id:PLAN-941 | plan_revision:r4 | round:59 | from:work | to:work | status:executing6/21 | outcome:in_progress（真实机制与独立缺口，完整目标保持） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:1548132986430e7298bcaaccf7f0d1554d79cb31 | next:work 其余证据；F07/F08授权到达则new，无待poll

### 第60轮：子CT有效版本正式接线提交；F08已获授权，下一阶段修r5（2026-11-06）

- 第59轮真实meta/child机制观察与新F08定位为progress。本轮入口核main r4 executing6/21、同树154813298 clean、main代码无WIP及归属，无待poll；在F07/F08未答复时先推进独立T08 child接线，未修F07测试或F08生产能力。
- 新`source-cte/tests/architecture_child_941.rs`及跟踪pick42.at沿原pick40只改flag true→false；复用既有compile_scalar_ct、CtService、MemoryEnvelopeSink及真实owner AV。RecordingSink只在原publish成功后观察原ExecKey，未复制producer/store。冷代真实AV2/value40；编辑后AV4/value42、无误hit；forbid后两级真hit、AV保持4/hits2/owner42。原InstanceRecord完整相等、inner/outer逻辑和签名稳定，而两级实际callee_body_version和执行key均改变。新目标实际nextest **1/1PASS、0skip，0.033s**（build1.16s、外1.9547234s）；原core first_lex_fact warning仍保留，未冒健康完成。
- 新只读`ace_m1_ct_child.py`以原/变体精确源字节、3次独立owner值40/42/42、声明实例身份不变、4个成功发布key、每代真实attempt/正steps、日志前缀守恒和hit继承事实交叉判定，不从返回值推版本。复用已有native raw admission，只允许原observer与新child两个有限binary/test对，完整参数/二进制/源/终态准入仍在。每项raw精确字段、16KiB原payload帽/同预算保留。新合同581 pins/26th registry pointer（原578+child测试/true源/false源），SHA **02470c7bdaabce07b1f64fcc6c78ebe89a840f150d848a4553cbfcba15355329**，原49 ARCH分母不改、声明execution U。多adapter用带来源map的独立architecture-execution/1归并，拒重复认领；发现表不冒执行。
- 新5条永久父侧反控：模板/实例声明误换key、有效checked key不变/逻辑或签名错误/实参或callee错误；相同错误结果/假计数无日志/发布key不对；hit伪执行/attempt/step/时长/归属错；错源/bool计数/额外PASS；原parent native目标不能冒child。控制初版给hit改key时误写成它本来就持有的c*64，测试如实红；改为不同f*64，未削弱断言/实现。**完整治理204/204PASS，4.468s**；与7组旧CT native反控共存，未改变旧199判定。最后删除一个未用的Python局部length绑定/候选Spec文字不改逻辑；复用204证据，不为提交重复全组。
- 冻结提交 **3b364267bcdc4bd0492f67718f55a51d640b1d54**，10文件，树clean；581合同物理输入再次核全相等。新reader/native/export的最终原始Case/严格全组合取证尚未做，**不能把child第三ARCH格位提前计PASS**。当前最近实际组合仍154813298上的709P8F289U/ARCH2P47U/原102P211U313；本轮只新增已测实施接线与治理，不冒完整T08/T11闭合。基于随后收到F08修复授权，生产source-CT身份将变；把新child正式取证与F08最终冻结并集一起做，避免先采一轮随即因生产修复失效。原134P1F及F07红仍原样保留。
- **用户新授权**：本轮中收到F08答复“授权941新增修复phase（推荐）”，question call_f0d62e537ab843b6a2d98593aaf23bcd。已记§4并关闭F08待决定状态；授权仅F08，F07尚未答复。下一阶段按auto-plan:new修订r5：把第59轮冷/暖/撤销cap与普通同名函数兼容的最小修复纳入当前941合同，保原目标/预算/已完成T16..21，补明确AC/V/SD/执行入口与真实受影响consumer/身份刷新；不再询问相同授权，不在当前work阶段先改生产。未批准F07保持独立，不能把test-only和F08混授权。
- 本轮scope资料仍在途，未清理/未改主线生产。next阶段**new**而非继续先修；按r5合同就绪后work处理F08，最终独立review/T14及merge/T15仍待。所有原缺口/8 scalar red/38 cert-gap/D938939/健康与旧义务保持，current_step仍6/21直到新合同落地。

stage: work | plan_id:PLAN-941 | plan_revision:r4 | outcome:in_progress（child接线/204治理提交；F08授权到达需new） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:3b364267bcdc4bd0492f67718f55a51d640b1d54 | task_ids:T08/T11 child adapter部分 | evidence:新native1PASS/204治理PASS/581输入稳定；F08用户明确授权 | blockers:child正式取证并入F08冻结；F07仍独立待裁定 | next_responsible_role:new F08 r5合同，继承原全链授权再work

stage: goal | plan_id:PLAN-941 | plan_revision:r4 | round:60 | from:work | to:new | status:executing6/21 | outcome:in_progress（child实施提交，F08授权驱动合同修订） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:3b364267bcdc4bd0492f67718f55a51d640b1d54 | next:new r5 F08 phase；F07无授权不动，无待poll

### 第61轮：r5 F08授权合同冻结，转work（2026-11-06）

- 第60轮子CT接线/204治理提交并收到F08授权是progress；本轮入口核main r4 executing6/21、同树3b364267b clean/main代码无WIP、已有F08授权与F07未决，按new阶段只修合同，无生产代码/构建/测试执行。参考new及verification-contract，读取canonical source-CT Spec/原core绑定与typecheck分界、产品先serve后bind路径、真实AV/native/A2X caller、实际固定组；默认CLOSED及普通名字承诺来自Spec，不以当前缺cap42结果改规范。
- 语义修订**r4→r5，executing6/23**，新增§5.11/T22/23/AC17/V29..32/SD11及source-query-integration规范影响metadata；原T01..21/AC01..16/V01..28/SD01..10、完成6行和原全部预算/分母/源/expected/guard保留。r4完成证据继续历史，不因新phase重置原完成项；受F08生产身份影响者须按实际变化刷新后才能称当前。
- 冻结冷/暖/撤权/再授cap/普通同名函数/保护冲突的所有权与可观察状态；在每次请求、任何执行/probe/hit/publish前核权，失败不清合法旧页，既有source-query facts仍最终类型权威。T22先以既有cap-aware绑定接缝做有界可行性，合法Hash/实例不能因错误调用typecheck而全拒，不新增词法禁令/第二solver。具体局部等价实现留work选，不能改批准语义或预算。
- 测试入口明确新source-cte集成目标、真实host四向与原A2X worker_ct_frontdoor_958；完整CT/治理与实际触面消费者一次冻结联合，若core/source-query接口触码才补其完整职责。原F07可真实红并继续T13，不因F08强行改其断言/缩负载/转PASS；其他8 scalar red/38cert-gap/D938939/旧U与T12/T14最终要求保持。A2X原缺typed错误权威不在F08扩协议，不以status/detail文本充typed PASS。
- 源基面3b364267b、8件baseline pins已存`f94108-r5-contract-baseline.json` SHA **2858ff0866bc8e5f3490a52cff88e3420e3c007956b2624493d5af6dc3c1ee4b**；新测试和预算/来源/关闭边界均具名，当前均未执行。r5只确认可开工合同，不冒实现/完整系统/里程碑已交付。无新增用户决定阻F08，既有第60轮授权直接继承work/review/merge，不再询问；F07独立等待不阻本phase。

stage: new | plan_id:PLAN-941 | plan_revision:r5 | outcome:pass（已授权F08合同可执行） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:3b364267bcdc4bd0492f67718f55a51d640b1d54 | changed_ids:§5.11/T22/23/AC17/V29..32/SD11 | next:work T22，先核基面并使永久反例红/接缝可行性，再改生产

stage: goal | plan_id:PLAN-941 | plan_revision:r5 | round:61 | from:new | to:work | status:executing6/23 | outcome:pass（合同阶段，完整目标未齐） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:3b364267bcdc4bd0492f67718f55a51d640b1d54 | next:work F08 T22；F07无授权不动，无待poll

### 第62轮：r5 work T22 F08每请求meta准入修复（7/23，T23待执行）

- 用户F08授权沿第61轮r5执行，未重复询问。唯一树`D:/autostack/.wt/lang-941/auto-lang`、分支`plan-941-dev`，基面3b364267bcdc4bd0492f67718f55a51d640b1d54；当前已提交 **be77185d5bb08214be4683c114eb86eddf1e7396**，4文件/363增9删，树clean。主检出只有其他会话的文档/计划WIP，未触代码、未合入这些改动。
- **准入接缝有界可行性**：直接复用core cap-aware `resolve_with`，仅采用typed `source.cte.meta-capability`/`source.cte.meta-name-conflict`作为policy拒绝，完整typed Resolve集合按原product Bind/`source.cte.bind`/0..0聚合；输出HIR全部丢弃，其他推导诊断不作为第二类型权威。最终仍为serve→结构替换→source-query事实→HIR/verify，不调用未服务Hash的typecheck。准入放在原parsed AST、实例化之前，避免原模板名被具体化改名移除后绕过保护名；新增准入与实例化耗时纳入同一原10s累计RootBudget，单AV min(2s, remaining)不变。
- **实际必要的局部适配**：既有resolver误把无cap的同名已声明函数在Hash内判为meta；两条调用分支均按既有签名表保普通声明。Hash旧叶处理对子算术/一元节点会panic，改为调用原迭代表达式绑定器；合法算术、一元、128层嵌套准入均永久验证。调度器原来还会把已声明普通函数的形似record参数误fold成2/true；改为按同一core `meta_intrinsic`和实际声明排除，非法普通调用保持原`cte.frontend/bind`且AV未到达，不造meta结果。无新parser/VM/缓存生产者、词法禁名或预算扩张。
- **永久oracle与红控**：新`experimental/ace/crates/source-cte/tests/meta_capability_941.rs`共9例。委托原MemoryEnvelopeSink，实际逐次记录probe与成功publish，核原service counters/log；覆盖FieldCount/HasField缺cap冷拒、嵌套/被调callee需求、有cap成功→撤权+forbid→旧product仍运行→再授权真实hit、两种普通名的零/有参调用及保护冲突、模板声明保护、形似selector的普通非法调用、shape实际AV参数变化、原pick实例/child。core诊断stage/code/精确span与product聚合投影均锁定，不按message前缀分类。
- 最终同字节9例在**原3b364三生产文件**实际重放：**1PASS/8FAIL、0skip、exit100、run0.047s**，fixture/生产SHA及原始命令在`f94108-final-baseline-red-v5/receipt.json`（SHA **212f03bcf3d9d9e88e8afe91e9dede274bfca9bb25a2bea0a9525fb52a10d3d5**），raw SHA **35ca7ff4a23355c044832583420e4daf949e35c2239cf57b3a931604ea2c193b**。try/finally后修复文件逐字节恢复；早期拆例时的空壳已纠正，v1..v4及探索raw保留但不充本次最终证据。
- 当前正确完整ace-comptime Cargo包/feature/config union下，仅选新9例+原T07/T08产品6例+两项实际CT机制observer：**17/17 PASS、128导航未选、exit0**，编译准备7.75s、运行0.135s；命令为`cargo nextest run -p ace-comptime-contract -p ace-comptime-core -p ace-comptime -p ace-source-cte --lib --tests --config-file ../../.config/nextest-full.toml -E 'test(t07_) | test(t08_) | binary(meta_capability_941) | binary(architecture_941) | binary(architecture_child_941)' --no-fail-fast`（cwd experimental/ace）。raw `f94108-green/t22-final.txt` SHA **48eac609ca4e972e2a7d09cca0b8f86feedf07aa27218cf23b782815ad6d0694**；不把128未选项当原条件排除或PASS，不宣称正式observer raw已准入。
- 另`cargo nextest run -p ace-core --lib --tests --config-file ../../.config/nextest-full.toml -E 'test(source_ct_936)' --no-fail-fast` **10/10 PASS、374导航未选、exit0**（准备12.72s、运行0.059s），包括无CT cap、原精确meta诊断、未服务Hash绑定后typecheck拒绝；raw `f94108-green/core-ct-seam.txt` SHA **07ceec13983c46c21d39218a6e00b385ad0610369d83c34d94373e9039ddbc72**。rustfmt --check与git diff --check通过；现有first_lex_fact/AV宿主dead-code warnings未由本phase引入，原健康T13仍待，不称零warning。
- 组合实例域不扩张：探索模板中直接meta selector/字符串、以及经flag携带meta的样例仍受原有限实例推导`source.instances-unsupported`/`unknown-name`拒绝；未把这些未交付组合改为支持或充PASS。当前原choose/pick及child有效版本链正例保持。完整原资源/GC/512修订/性能条件未在T22导航中执行，留T23原样执行。
- SD11候选为scope内`f94108-sd11-candidate.md` SHA **c228d10944bd40417d349cf028056e4fe17e9fddcb057261f35d96dd03215699**，待最终r5独立复审沉淀，不提前改主Spec。4个当前文件=提交blob、baseline/current测试与材料索引审计在`f94108-t22-closure/audit.json` SHA **a0893caf87d4f44c6bb5e0bbe54a09a63ca389e07e601270e290b998bfdc919a**。
- **F08尚未核销**：T22完成，V29..31工作层闭合；T23/V32/AC17的完整职责组`ace-comptime infra-ace-m1-arch`与因core生产适配追加`ace-source`、原14 CT/host/四CLI真consumer，以及工具prepare后变更身份闭包的原wire/cert/daily/优化重验仍待。578/581旧contract pins及旧before/raw保持历史，当前生产变更使其不再可直接认领；下一窗口更新当前contract/worker、重新采CT/child和受影响证据，不能回填旧before。原709P8F289U预览现仅历史，不声称当前严格组合报告仍绿。
- F07仍无授权，原300ms/10s测试/实际FAIL未动；原8 scalar红、38旧cert-gap、D938/939及其余原required/ARCH/U仍未关闭。无live构建/采集任务；T23就绪后才可核销F08，最终T14仍按r5冻结SHA独立review，尚未合并/归档/清理。

stage: work | plan_id:PLAN-941 | plan_revision:r5 | outcome:progress（T22完成，完整work未齐） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | base_commit:3b364267bcdc4bd0492f67718f55a51d640b1d54 | code_commit:be77185d5bb08214be4683c114eb86eddf1e7396 | task_ids:T22/V29..31/SD11候选 | evidence:f94108-t22-closure/audit.json | blockers:F07独立待批；T23/原任务未闭 | next_responsible_role:work T23

stage: goal | plan_id:PLAN-941 | plan_revision:r5 | round:62 | from:work | to:work | status:executing7/23 | outcome:progress（F08实施单元完成，完整目标未齐） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:be77185d5bb08214be4683c114eb86eddf1e7396 | next:work T23，先补真实consumer永久例/当前contract并prepare，再冻结并集与身份闭包重验；无待poll

### 第63轮：r5 work T23消费者回归/当前合同已提交，原完整CT压力门运行中

- 上一轮属**progress**：T22生产修复与永久反例已提交，非等待/无进展。本轮沿唯一树、同一r5/T23继续；生产仍be77185d5，新增测试/当前声明提交 **4ece6bd6a08a4cdb7658de9c96a8d1574dfefc26**（7文件/237增11删），当前树clean，主检出代码无WIP。总进度仍**7/23**，T23未勾，F08仍未关闭。
- 实际AV/native host新增`experimental/ace/tests/conformance/tests/host/ct_meta_capability_941.rs`及注册：原meta2/HasField缺cap、嵌套缺cap、两个普通名的保护冲突、模板保护、合法field/HasField缺→有、普通同名42，共**22个真实recipe检查**；另**2个错误code反控**必须各为FAIL（unit本身PASS）。AV按既有parse投影、native按既有source-v1/resolve投影精确核`source.cte.bind`，不改producer协议或期望词表。`cargo t ace-host --test-name ct_meta_capability_941 --out <h>/f94108-consumer-iterations-v3/host`实际 **2/2 PASS、175导航未选**；运行5.799s，整命令15.2675s，收据SHA **98d45fb5c3859777ed9935c66c1ed36fb9aaae0bf32b5880b0f4a74d1dcb466c**。
- 原`worker_ct_frontdoor_958.rs`增加3个941永久例，真实daily-emit/daily-emit-c/certify/certify-c检查**缺cap8操作、保护名称12操作、普通名称8操作**；拒绝必须exit2/status error/无目标产物目录，正例daily文件/manifest hash、实际与独立Rust/C reference可执行产物均核42。A2X错误面仍status/detail，未据其文案声称typed诊断权威。`cargo t ace-a2x --test-name worker_ct_941 --out <h>/f94108-consumer-iterations-v3/a2x`实际**3/3 PASS、212导航未选**，运行9.336s/整命令24.9207s，收据SHA **2d93b1390e50d36288a15f13e56526f8d79e0a00a931ee4905c81cd3c6c1e7f7**。仅C认证沿原Windows条件；当前Windows四操作全执行。随后只把原有非Windows C认证条件显式共用于新例，冻结窗口将执行全部10个958/941具名原测试，未以本次迭代顶最终窗口。
- 新例统一通过`AUTO_PLAN_941_CONSUMER_ROOT`把临时输入/产物放941 scope，唯一新建所有权目录由原测试生命周期清理，host输入与输出分离。最初两次host **0P2F**只分别触发新测试生成器的case id格式、输入/输出重叠门（真实consumer尚未执行）；修正为合法ace前缀与分离目录后v3实际绿。旧raw保留，不归生产故障，不充PASS。
- 显式原prepare入口12工具完成，**92.0694s**，`f94108-consumer-tools/prepared.json` SHA **fa4a75da946006a4931082d61684f800a2a8405d7ccb4d629d79a158707cbf00**；13个实际产物中**8个worker变更**（AV/SYN/native/collections/CT/ABI/sema/A2X），2父入口+3运行时库字节不变。当前A2X release **f8fb404be8aa3b0c9ecc6f472fedd7dce3515f3cd9dc748115f1d33b94c649fb**，AV **e25a343c7f3984ee3ca5706473ee2722ddf19191ee55802e843be711d845c42f**，native **3ef84fb8784707613615ff5264aa6eabd7bd967815e44b40c5e2daf7d6126e92**。所有正式before均待真正窗口抓取，不将本次after回填旧before。
- 两个CT声明维持UNOBSERVED及原独立oracle，仅r5/三个实际生产输入更新（resolve/product/schedule）：原**578** pins合同当前SHA **b9ccc38470fa1ffaea997bf6cb83a27c44d8884b399d70f2718c02c5062d9f89**；child原**581** pins合同SHA **de1ba9c91407fee1c1e985ee6d93b17f878a471aa49b895ac8491485cac00c40**。旧合同原字节折入`f94108-contract-refresh/before-*`，原raw/旧before不覆盖；审计SHA **1b1f176826d2513493abff211323a143b0e9f4f21fe3376ecad4b81fab0caff0**。204/204 Python治理控制PASS（6.846s），raw SHA **52c5c3bc17c033cbc6cbf643030b1073c15037d77b28cc3b518466eb7edb05a6**；格式/diff门通过。观察者正式当前raw仍待本窗口后续命名采集，不以合同更新/控制充正式观察。
- **当前原完整冻结门正在真实运行**：`cargo t ace-source ace-comptime infra-ace-m1-arch --out <h>/f94108-frozen-window/union`，原入口/样本/性能/GC/ignore保留。source职责已真实终态 **383 PASS、1原ignored**（`link::tests::run_exe_60s_deadline_bounds_descendant_pipe_hold`）、not_run/errors均空，运行105421.0402ms，`union/0/result.json` SHA **761098d55612548f70bd5a1e1f3ef34a589de41acc89531493d79df01f7d39fa**。CT与infra未终态，不能宣称完整门PASS；F07旧fast deadline测试不动，若当前再红，必须保父exit和原FAIL。
- **续轮只跟随现活句柄**：exec session **57112**，外层Python PID **45112**，固定父Python **38264**，nextest **25044**。OS进程树实核当前测试PID **21592** 为`cte_resources_tests::performance::resources_real_watermark_gc_physical_delete_and_converge`；最后核其存活、CPU155.50s/工作集32MiB。外层脚本`<h>/run-f94108-frozen-window.py`将在完整门结束并核真实终态后，顺序执行冻结host2、原958+新941合计10例、原source-CT14，以及两个当前CT/child observer的完整配置准备/原before→实际执行→after采集；不并发其他编译/认证，不因暂未输出重启该作业。最终`window-terminal.json`当前未产生，待原句柄终态后实读。
- 当前身份影响审计 **78aa67364b9d5ac813412908b22643ac8d65f67924de027963eab6e84c58cbe8**（`f94108-identity-impact/audit.json`）确认**292默认证书/daily全部stale**，原14优化腿的实际debug worker/测试二进制也变更；旧ABI/bridge worker本轮也变更，不能沿上次F06“bridge未变”复用。Case真实影响为原408+原数据52+bridge6=**466**，其中本冻结窗口将执行原CT14，其余452按真实影响重采；原query负前门6操作也须重采但仍保持untyped/UNOBSERVED。父入口/语料/独立oracle未变可逐hash复用声明，原8 scalar红/38旧cert-gap及全部旧U守恒。这里仅审计，尚未刷新这些source/cert/优化证据，709P8F289U仍只历史。
- 7个交付文件=冻结提交blob、新消费者终态/工具/合同/当前source383与继续点审计在`f94108-t23-checkpoint/audit.json` SHA **8d66419eddd0efef627e3f1f015889b7a867fc70ef63198fef75a15b24ddbfc8**。F07仍独立待批，F08仍待本次完整窗口和实际身份闭包严格读取；原任务/D938939/健康门/最终独立T14均未核销。下一轮先poll 57112，完整窗口终态后才开始source/cert/daily/优化重验，无新增授权问题。

stage: work | plan_id:PLAN-941 | plan_revision:r5 | outcome:progress（T23消费者/合同交付，完整冻结门在途） | worktree_path:D:/autostack/.wt/lang-941/auto-lang | branch:plan-941-dev | code_commit:4ece6bd6a08a4cdb7658de9c96a8d1574dfefc26 | task_ids:T23/V32 | evidence:f94108-t23-checkpoint/audit.json | live_handle:57112/root45112 | next_responsible_role:work，跟随原完整窗口终态再刷新当前身份闭包

stage: goal | plan_id:PLAN-941 | plan_revision:r5 | round:63 | from:work | to:work | status:executing7/23 | outcome:progress（新消费者/合同提交及source383终态，完整目标未齐） | worktree:D:/autostack/.wt/lang-941/auto-lang | commit:4ece6bd6a08a4cdb7658de9c96a8d1574dfefc26 | next:poll exec57112，原窗口终态后当前466 Case/292 cert+daily/14优化/6 untyped负前门闭包；T23/F08未关

## 10. 待澄清事项

1. 没有新增ABI形态、放宽原预算/原样本或自动NA授权。新组合报告格式/逐义务适配/本域预算由T-02有界证明，不能靠先全读大日志/隐式工具启动解决。
2. CE-BASE-01不是本轮解除项：默认沿当前924旧cert守卫保红，旧38格功能缺失归旧A2R线，证据接线由T-01/04审计。若希望恢复G-DEC原双轴的独立记账签章，须用户明确裁定并修订r1后执行；不把该选择预填为已批准。本包规划与只读实现准备不依赖该口径变更。
3. 在途934..936、938/939若无法提供承诺字段/原始计量/required页，原owner交稳定接口或具名新缺陷计划；不能由本包改expected/删行/绕checker模拟通过，不给未立项后继预占号。
4. **F-941-01 已关闭（2026-11-06，D951=fec8a9602）**：用户已授权独立修复并继续；951实际复审/合入/归档/清理6/6完成，941本树重建原worker并重放Rust/C完整actual+独立reference，两证书均由原严格reader接受。原授权和T-01/T-02其余工作继续有效，无待决定producer修复事项；不得用本项关闭替代其余验收或旧38格认证。

5. **F-952-01 已由D953关闭**：独立修复交付/归档/清理完成，952已合入/归档/清理6/6；原941/952授权保持。用户已`/goal resume`且继续推进原941，不重复请求；host状态未更新仅记录，工作树与材料在途保护。

6. **当前前置与验收缺口**：D936已交付并同步；F-941-03已由D958=06251b0cc独立修复交付/归档/清理6/6关闭，941消费8原入口通过。D938（source foreign/代际与恢复接口）、D939（opaque资源/callback/error/ctx与实际source合同）仍未交齐；936在941所需CT/frozen/required页与原计量仍须真实观测。首例/治理控制不核销这些义务，无需重授941。

7. **F-941-03 已关闭（D958=06251b0cc）**：用户授权的独立修复完成review/全196门/ff合入/Spec/归档/清理6/6；941已合入并新worker重放原8操作全通过。dead-owner目录及958所有清理均实际完成，无待清理/重授项；941其余验收继续。

8. **F-941-04已获r2内修授权**：按用户“直接在941计划添加新的phase修复”执行§5.8/T16/17，不再等待独立修复授权；T16/T17实现与原例/完整并集/当前严格认证验证已完成（第22轮）；最终独立review仍待T14，不以phase自测顶review。

9. **F-941-05已关闭（r3 T18/19，第43轮）**：同树Rust字段写值根修复、参数边界/三代缓存、稳定B工具完整405门、292当前默认证书/daily及原14优化严格联读已闭，原3缺Rust格位转PASS。最终独立复审仍T14；不扩为F06/其他后端修复，原完整任务与全部具名red/unknown保持。

10. **F-941-06工作阶段已关闭（r4 T20/21，第52轮）**：用户授权的typed data诊断修复、原4反例/合法与失败边界、1493完整职责/消费者门、408当前Case重采/原16诊断严格PASS及当前全组合679P8F319U已闭。SD10候选已在冻结树；最终独立review仍T14，不再等待授权，不扩为其他后端修复。

11. **F-941-07待裁定**：原CT 300ms fast控制在AV前已过期时正确ct_execute0，但旧断言固定>=1而误红；同库/原267218B输入的单核竞争反证已定位（第58轮）。提请在941新增test-only修复phase：保原115helper/4call/300ms与32query/10s原门及合法4call正控，按真实AV到达分态修正测试并补确定性回归/反控，不改生产计数/预算/driver。依据§5.4需具名授权；未授权仅不动该修复，继续原其他证据。

12. **F-941-08已授权并纳入r5，T22完成、下一work T23**：第60轮用户授权已落实§5.11/T22/23/AC17/V29..32/SD11（第61轮），无需新确认；source-CT meta能力/普通名兼容与真实四端/身份回归按当前合同实施。当前生产提交be77185d5、9永久例与17产品/10 core CT定向通过（第62轮）；child正式采集及真实四端/职责/身份闭包归T23冻结。F08尚未关闭；F07仍独立待裁定，不能互相套用。
