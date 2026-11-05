---
plan_id: PLAN-941
status: executing
feature_name: CA-E／ARCH 扩大后M1登记与最终组合验收
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-06
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-applicability.md, docs/specs/auto-test-case/unified-source-syntax-admission.md, docs/specs/auto-test-case/ace-ca-d-bridge-certification.md, docs/specs/auto-test-case/architecture.md]
new_spec_components: [docs/specs/auto-test-case/ace-m1-expanded-acceptance.md]
touched_goals: [GOAL-002]
affects: [auto-test-case, auto-ac, auto-av, auto-a2x, auto-a2c, auto-abi, auto-semantics, auto-cte]
depends_on: [PLAN-908, PLAN-929, PLAN-934, PLAN-935, PLAN-936, PLAN-937, PLAN-938, PLAN-939]
current_step: 0
total_steps: 15
---

# [PLAN-941] CA-E／ARCH 扩大后M1登记与最终组合验收

## 0. 变更摘要

正式认领CA-E与ARCH最终组合门，一个验收集成包，不拆成空schema与以后才找consumer。新增独立target/机制登记、严格父侧证据适配、四端同源独立oracle与版本化组合报告，组合原313 required门、C A/B/C/D、SYN与ARCH-01..08；复核真实函数级增量、恢复首编辑、资源、在线更新及独立认证。编译/语言/资源功能仍由原owner交付，本包不变相接走延期功能，不再造编译器、store、runner或meter。

架构：[Design51 §16](../design/51-shared-a2x-and-hir-to-rust.md#16-ca-earch最终组合验收2026-11-05target-state)；现状/基线/遗留认证差异见[立项报告](reports/ace-ca-e-arch-baseline-and-schedule-2026-11-05.md)。规划轮仅授权规划；2026-11-06执行授权及当前进度见§4/§9。

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

2026-11-06 执行授权：用户 `/goal 完成计划941` 授权 r1 全链 work→独立 review→范围内有界修复→merge/Specs/归档/清理，取代下文历史规划轮的“仅规划”动作限制；不改变验收、分母、预算、仓库范围或 CE-BASE-01。按计划 §6 不自动派发 agent/创建 chat。当前进入 T-01；未将依赖未齐或真实 red 当作 work 完成。

授权：用户明确“为CA-E／ARCH最终组合验收正式立项……本轮只做规划，不施工”。本轮允许只读调查/设计/计划/路线文档；不建实施树/goal/agent、运行测试/计量、改代码或current Specs，不接管别会话。初始main `5f6737467`（比上轮只读状态推进：934已reviewed，未见实际D），见报告记录的输入hash和最后状态。原WIP与940等他方计划保留。

929归档14/14，D=`29a7d7007`为当前祖先、树已清；canonical source-frontdoor/SYN准入确已交付，但只签SYN，不能冒SQ/CT/ABI总绿。937归档12/12，D=`fa35943ac`为当前祖先，canonical shared-interface/bridge-cert已交；它的native是797声明IR消费者，source是C exporter，typed三腿与source import边界不能混同。原recipe`abi/ca-d1-bridge-v1`与旧`abi/interop-v1`共worker信封，但deck/leg互拒。旧空steps可空过是记录在案的族属性，本包必须在组合侧以executed>0/完整义务强制拒，不据其exit0签新门。

934当前reviewed r2 11/13（有复审pass收据，步骤数字不代替复审记录）；935 executing1/12、936 executing1/14，主体等D934/实际实例接口；938 drafting0/13、939 drafting0/14。最新交付/树与主线漂移在T-01再核，不把reviewed或在途branch当D。940是另一会话新立的legacy Core/UI parser推进性计划，不是ACE统一前门替代，不作为整包前置，仅真实H/consumer交集协商。

只读重算（非运行验收）：原binding131行/12列，批准hash匹配；required313、非绿能力token0、required旧A2R cert-gap38，required-set hash `35ad43aae27ffb2d2c9a162c3dd5b99f971e4cd7b2ed36f4fe747a5634bd770e`。现acceptance scope只核算且exit0不等里程碑pass；其ParentReport检查非空/PASS不能独自证明source/producer/义务完整，/3须独立核闭包。/2已有`certGapCells.Count -eq 0`守卫，不能被本包暗绕。

唯一实施树`D:/autostack/.wt/lang-941/auto-lang`、`plan-941-dev`（已建，当前基线以§9检查点为准），实现/内部自查/独立上下文复审顺序使用同一树。未来材料仅`D:/autostack/plan-artifacts/plan-941/`，开工登记owner/用途/在途；consumer/外置target/日志/cert scratch显式归scope，正常构建在树内。merge摘要折计划、全输出路径对账、删除scope与guard清树；不操作他方产物/SDK/工作目录，不写legacy specs.json。

## 5. 详细设计

### 5.1 登记与不可变义务集合

新registry引用旧批准件/hash与313集合hash，追加M1 A2C增补/架构机制裁定及各profile合同。义务键至少包含domain/case/ns/profile/target/obligation/direction/edit-kind；每项有来源路径+内容hash、独立oracle来源、owner/正式D、需用recipe/输入profile、正/负终态、原样本/预算域和证据入口。必需集合hash只由批准范围/义务构成，不受本次执行选择、报告状态或支持探测影响；证据集合另算hash。registry不含本次报告hash，报告含registry/child hash，避免自指循环。

两类登记单列：Q4同源语义行（四个真实新目标）与ARCH机制/方向行（按真实能力定义target适用性）。C既定域来自892/902/904与921/925/926完整冻结语料及补例，不只取当前3+5+20个C首例；这些28例是首例不是完整分母。scalar原61例、资源/method/collections负控与typed helper边界逐项映射。934/935/936正式承诺的有限新source必须进入相应Q4行；937/938/939有限FFI按source/typed及真实native/C方向登记，不冒四端FFI全覆盖。C未承诺闭包/Dyn/全Unit/console/宽源码等按原批准明确列future/out-of-profile，不将缺支持动态排出required或自动套旧NA；原已required不得转换。

每个required原格位、新target行、ARCH原子义务必须映射可执行case/check/producer或明确未交付owner，group名仅定位不能代覆盖证明。相同basename/共享编号不等同相同源；源码字节、profile/caps与oracle语义版本绑定。已有无源码typed/scenario不伪造Auto源，宽typed frame证明不冒宽源码覆盖。修改登记范围/NA/accepted-diff须独立明确裁定，不由实现者为了绿自行决定。

### 5.2 判定谓词与38格旧认证差异

总谓词固定为：`legacy-current-gate AND legacy-required-execution-evidence AND C-core/resource AND Q4-oracle/certification AND SYN/ARCH-01..08 AND precise-incremental/restore AND budgets/time AND expanded-registry/evidence-valid`。其中`legacy-current-gate`沿当前/2实际更严守卫，不能因旧脚本exit0、core token全绿或新HIR A2R证书通过而视为true。

差异`CE-BASE-01`：G-DEC-3/旧适用性与scenario规范将旧a2r金样和认证分轴；924之后脚本及gap Spec明确新增cert-gap=0条件。**本r1保更严现行守卫且将两份规则/来源同时呈现，不改旧批准文件/hash，不把38 cert-gap改PASS或自动由新A2R证书顶替。** 38项逐格进入独立阻塞表（见报告完整清单），功能责任owner=旧A2R复合lowering/认证线，证据接线owner=本计划T-01/T-04。存在本域真认证仅漏接线时可由证据owner按原规则回填映射；缺编译/语义/trap能力需原线后继正式包，不预占号，不塞本包修后端。若要依原批准恢复独立记账签章，须用户另作明确裁定及语义修订后执行，不能借报告/3更名暗撤守卫。

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

T-01/T-02把表拆到每个required原子义务/具体entry与closing oracle；不能只有八个总布尔字段。存在新功能缺口仍归原owner，已归档能力发现新缺陷要具名新修复流程，不能悄悄重开旧Plan或在组合器模拟能力。

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
| S/N/X | 本包不修改core/HIR/MIR/lowering/语法/资源分析/native/C业务实现 | 若需补观测由原owner稳定SHA单写交字段/计量；不借总验收施工修功能 |
| H | 新case/contract/SDK/worker/strict cert reader、mod.rs、Cargo/lock、test-groups与父报告适配 | 934→935/936、938/939与本包逐文件SHA/符号/全部consumer/兼容门释放接收，whole-file单写；不按append-only并写，不占窗等无关D |
| 940等外线 | 只读Core/UI parser与测试注册实际交集 | 非新增语义依赖，不触其WIP；只有真实H/legacyconsumer交集协商 |
| 正式采样/认证 | 计划/样本/oracle准备并行 | 与当前owner错峰，原串行/resource组与单实例保持；最终联合门在实际所需D全就绪后 |

与在途contract/schema的等价适配不自动扩大其语义，必要reader变更须列producer→required字段→所有reader→发布/恢复路径→冻结验证。缺项保owner与关闭oracle，不用“本包后面统一补”让原包假pass。公共窗口释放后可接，不要求全部计划先结束才做独立段。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/ace-m1-expanded-acceptance.md | 原分立证据→新registry/严格父组合/真实签章谓词、旧认证阻塞与独立来源 | 可审M1总门 | AC-01,AC-02,AC-10,AC-11 |
| SD-02 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 新Q4/ARCH与原313平行AND，不插原第五列/改分母 | 旧/新身份分离 | AC-01,AC-03,AC-09 |
| SD-03 | modify | docs/specs/auto-test-case/ace-m1-applicability.md | 明列924当前cert守卫与原双轴差异/blocked ledger，不撤旧批准 | 不暗绕旧门 | AC-10,AC-11 |
| SD-04 | modify | docs/specs/auto-test-case/unified-source-syntax-admission.md | SYN签章消费范围与新ARCH原子义务证据映射/反控 | 929不冒全ARCH | AC-04,AC-05,AC-06 |
| SD-05 | modify | docs/specs/auto-test-case/ace-ca-d-bridge-certification.md | D1/D2/D3证据方向、source/typed/生命周期与总门映射 | 937不冒完整FFI | AC-08,AC-09 |
| SD-06 | modify | docs/specs/auto-test-case/architecture.md | 新只读父组合器/正式TestCase/报告schema/证据闭包与完整固定组 | 不造runner/meter | AC-02,AC-07,AC-11 |

本轮仅提议，review后由merge沉淀已实现事实，不写current Spec或派生索引。

## 6. 测试设计

V全部为拟新增/未执行。实际既有entry见§3；新增拟治理测试`scripts/tests/test_ace_m1_arch_acceptance.py`、套件`crates/auto-lang/test/cases/ace-m1-expanded/`、必要新语言`test_cases/ace_m1_arch.rs`首例由T-02沿既有正式父入口使能，全域由T-04/05/11使能。独立oracle包括现scenario规范、902/904/reference、C三域手写reference与布局/原797 caller/provider；新增跨机制正例的手算oracle与实际source先固定，不运行emitter生成expected。实现自查→同一树独立上下文review，用户未指定其他人员/模型，不自动spawn或创建chat。

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
| V-18 | AC-12 独立完整审查 | T-14，同树独立上下文重建义务/reader/正反oracle/SD/债 | 无未知缺口默认PASS/未批延后/篡改分母；review绑定r1与冻结SHA | 系统pass与M1claim分开审查 |

职责组：现存`cargo t infra-ace-m1 infra-ace-cost-report infra-fixed auto-test-case ace-adapters`作为新增父判面基础；拟新增`infra-ace-m1-arch`、`case-ace-m1-q4-941`及适用ARCH首例组（T-11按现固定组机制登记，新组目前不可执行，不复活tm/叶目录发现系统）。完整真实证据并集由registry逐行引用现`case-ace-m1-*`各具名组、`case-a2r-hir-record-text-902`、`case-a2r-hir-collections-904`、`case-a2c-hir-scalar-921`、`case-a2c-hir-record-text-925`、`case-a2c-hir-collections-926`、`case-ace-unified-source-929`、`case-ace-ca-d-scalar-937`及新上游正式组，T-01列成显式名单而非shell wildcard执行。source/query/CT/ABI/缓存与HMR的原完整职责/专项由真实触面加`ace-source ace-semantics ace-comptime ace-abi ace-foreign-contract ace-a2c-abi ace-artifacts ace-av ace-native ace-a2x ace-lowering`，不得仅跑新增治理脚本自证实际语义。

固定组和专项原ignore/平台/资源/Job/输出帽保留、执行/排除/未执行/首次红/重放绿分别列。性能/压力与其他正式采样错峰，merge冻结提交一次职责并集，输入身份未变的有效证据可复用且逐项写理由；不为纯文档跑构建、不叠无因全experimental/all，周期`cargo t all`只按due在主检出单实例。新增Rust test在autotests=false包要显式Cargo target登记且确认被完整组选中。

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
| AC-12 | review | V-18独立逐AC/SD/遗漏/差异/债/全部reader/warnings/fmt审查，系统交付与里程碑claim分别绑定r1/冻结SHA |
| AC-13 | merge | 冻结职责并集/触码漂移重验、SD沉淀读回/交付归档、全部输出对账及scope/guard清理；真实M1未绿仍标未绿 |

AC-01..09的语言/机制实测须所需D真实就绪，不能仅靠合成reader正控关闭。AC-10/11允许真实报告诚实red来证明验收系统正确，不能据此豁免前述应执行证据或声称M1已过。若底层新缺陷导致某项实际验证无法关闭，留具名阻塞/owner/永久反例，不伪造work完成。

## 8. 执行步骤

| ID | owner_stage | 依赖/写面 | 具体路径/可验证结果；AC/验证 | 状态 |
|---|---|---|---|---|
| T-01 | work | 用户后续执行指令；只读当前Specs/收据 | 冻结main/929/937/934..939实际D与reader、旧313/38清单、完整语料/原预算/具体固定组/全部owner，scope登记；AC-01/10，V-01/14 | [ ] |
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
| T-12 | work | T-11；最终D934/935/936/938/939与已交929/937；正式采样窗 | 冻结真实完整证据并集/原样本规模、fresh/hit/restart/first-edit/资源/真实独立认证→组合报告，缺项保持真实red；AC-01..11，V-01..17 | [ ] |
| T-13 | work | T-12 | SD/coverage守恒/全部owner接口/健康冻结/遗留cert与新缺陷交接；明确系统交付与M1是否可签，不能藏in-contract未执行；AC-11 | [ ] |
| T-14 | review | T-01..13完成，停止实现写入 | 同树独立上下文重建source→producer→consumer→oracle→gate全链/反控/所有SD与债，不采信摘要；AC-12，V-18 | [ ] |
| T-15 | merge | T-14 pass与后续合入指令 | 冻结职责并集/漂移复验、SD沉淀/主线读回/归档、所有输出scope与树guard清理；AC-13；M1未绿不改成绿 | [ ] |

下一入口T-01/T-02/T-03/04不等全部D，可独立推进；最终联合T-12必须全部真实相关D齐。共享接口稳定可提前接单个切片，不能把“方案已写/审过”当正式交付。用户已授权work/review/merge全链并恢复目标；按T-01/T-02继续，尚未关闭的验收保持未完成。

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

## 10. 待澄清事项

1. 没有新增ABI形态、放宽原预算/原样本或自动NA授权。新组合报告格式/逐义务适配/本域预算由T-02有界证明，不能靠先全读大日志/隐式工具启动解决。
2. CE-BASE-01不是本轮解除项：默认沿当前924旧cert守卫保红，旧38格功能缺失归旧A2R线，证据接线由T-01/04审计。若希望恢复G-DEC原双轴的独立记账签章，须用户明确裁定并修订r1后执行；不把该选择预填为已批准。本包规划与只读实现准备不依赖该口径变更。
3. 在途934..936、938/939若无法提供承诺字段/原始计量/required页，原owner交稳定接口或具名新缺陷计划；不能由本包改expected/删行/绕checker模拟通过，不给未立项后继预占号。
4. **F-941-01 已关闭（2026-11-06，D951=fec8a9602）**：用户已授权独立修复并继续；951实际复审/合入/归档/清理6/6完成，941本树重建原worker并重放Rust/C完整actual+独立reference，两证书均由原严格reader接受。原授权和T-01/T-02其余工作继续有效，无待决定producer修复事项；不得用本项关闭替代其余验收或旧38格认证。

5. **F-952-01 独立共享采集器修复授权待答**：具体最小范围与确定性反例见952 §9；当前goal blocked，原941/952授权不撤销，工作树与材料在途保护。
