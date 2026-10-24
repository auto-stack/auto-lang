---
plan_id: PLAN-827
status: execution_done
feature_name: AV真实源码程序验收基线与首个文件执行闭环
plan_revision: 3
author: [Codex]
created_at: 2026-10-20
updated_at: 2026-10-25
current_step: 7
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-av/resident-function-install.md
  - docs/specs/auto-av/project.md
  - docs/specs/auto-av/cache-and-conformance.md
  - docs/specs/auto-test-case/architecture.md
new_spec_components:
  - docs/specs/auto-av/language-support.md
  - docs/specs/auto-av/source-program-conformance.md
touched_goals: [GOAL-002]
affects: [auto-av, auto-mir, auto-test-case]
---

# [PLAN-827] AV真实源码程序验收基线与首个文件执行闭环

## 0. 变更摘要

建立[能力矩阵](../design/strategy/av-language-capability-matrix.md)和[替换路线](../design/strategy/av-source-programs-and-avm-replacement-roadmap.md)的G0/G1事实基线：全量盘点能力/程序/入口/stdlib消费者，建立以**现有Auto源码文件**为输入的实际AV执行/验收入口，运行AP-01/AP-22及准确trap的AP-23。不是再建一个MIR模型或解释器，不扩798 scalar语义，不切产品默认。

本轮仅起草，未编写实现/测试或创建实施树。后续广泛语言/库/async/UI/FFI按R2-*..R8工作包另立适中Plan，不能顺带纳入本计划。

## 1. 目标

1. 冻结能力/程序/入口/SKU分母，起码覆盖矩阵C-01..43、程序目录AP-01..23及EP-01..10类别的真实call-site/包/feature；未知项保留gap和owner，不以种子目录代替全量库存。
2. 用户给出的真实source文件字节经既有AC source profile→Checked HIR→共享MIR→ABC→真实verifier/AV→typed结果。AP-01/AP-22 entry i32=0，AP-23实际整数除零trap；保存source依赖/工具/产物/运行身份及独立oracle。
3. 同store fresh/hit/新worker restart、受控源修改/损坏/错身份/撤销授权、正式AutoDown父侧判定与资源/延迟反控实际可达，不缓存PASS/运行结果。
4. 形成source-vs-fixture-vs-product的真实支持Spec和后续分期输入；G1通过仅表示实验scalar文件入口，不冒领通用脚本/CLI替换。

非目标：Unit值/print/宽算术、List/enum/闭包源码、多模块/完整stdlib/宿主IO、生产默认run/REPL/UI/FFI、CTE/native新数据、JIT和旧AVM删除。821等未来profile不因目录中的程序提前支持。oldAVM/native仅用于独立参考，不参与新路径实际执行。

## 2. 架构方案

source-file/固定内容引用 → 可信宿主读取/冻结bytes与根/工具 → existing source编译/verify → D798共享MIR/ABC/AV → 当前typed actual → 既有父侧judge。

优先复用D798已交付source API、driver、worker和recipe，不重写其已有source→AV能力。T-01先以真实文件/父侧oracle证明现有入口覆盖情况：若已覆盖，T-03只登记新程序清单/薄适配及反控，不新增重复worker/workspace/recipe；缺口有具体证据才增最薄消费层。这个有界复用决策在后续实施前冻结，不减少AP/Scope/安全验收。

确需source-program worker时它只做薄消费层，可拟新增独立workspace `experimental/av-source-probe/`（名字/包布局T-01核实际owner后冻结），只消费D798及其已交付probe公共driver/API，不复制parser、类型推导、MIR、codec或VM。D798实际public API不足时，本Plan在单写窗口做最小同语义导出/adapter，并重放原支持面；需要扩语义/协议或改owner时先回new，不要求798在途分支为本Plan额外交付。

实验入口名称由T-01冻结，拟recipe `av/source-run-v1`、worker命令run-source/case-serve；它们**当前不存在、未验证**。不新增adapter enum或第二份协议decoder/runner，沿实际已交付probe/run注册方式与受控worker。只允许声明式源输入/限额/结果；不能在case内放shell或任意exe。unknown recipe/字段/能力受控拒绝。

## 3. 技术栈

Rust、D798共享独立workspace和现有source profile、783 ManagedStore、796身份、正式AutoDown probe/run/contained进程。观察源码锚：`experimental/ac-core/src/source/`、D798 `experimental/ac-av-probe/{core,probe}`；具体公开symbols/包/manifest和注册路径由T-01按真正交付版本确认，不能编译未合入分支。

按§2复用调查确需时才新增 `experimental/av-source-probe/`；其余拟新增 `test/cases/av-source/827/`、`scripts/verify-av-source-827.ps1`、`docs/plans/attachments/827/contract-decision.md`，测试前缀 `av_source_827`。已有AP源码只读；oracle/变体/inputs副本在本计划fixtures或独立scratch，不在原目录写入。正式recipe的薄注册/父侧判定在当前真实owner位置追加；根workspace成员/产品manifest、旧VM、UI/Man/SDK及其他在途代码不改。

拟唯一实施树 `D:/autostack/.wt/lang-827/auto-lang`、分支 `plan-827-dev`，当前未创建。work/独立review/merge顺序使用同树；主检出仅计划簿记。若需要ac-core语言能力改变则退出本范围，改由后续source owner计划处理。

## 4. 需求分析与背景调查

用户2026-10-21确认：828之后优先推进817/818、827与820功能线，825/826择机插入；全链路增量、D828门和原硬依赖保留。授权本轮更新计划/战略及单写交接规则，不启动实施、不改变已有进度/预算/支持面；当前合同以§5–8及共同排期为准，旧排期/待裁定记录为历史。

**2026-10-24 用户指令"计划827: 实施"= 本Plan实施授权**；入口门核验：D798/D828 均已归档（archive/798、archive/828，828 merge 收据 11/11）；主检出无代码路径 WIP（仅 docs/plans 等簿记）；唯一实施树 `D:/autostack/.wt/lang-827/auto-lang` @369e1c349 分支 `plan-827-dev` 已创建；status drafting→executing。注册面在途 owner 为 PLAN-820（av.data/1，未合入），公共注册追加按交接合同串行，薄消费层/清单/自身验收可先行。

授权：2026-10-20用户要求建立新版AV语言能力矩阵、缺口、真实程序及最终替换路线/门槛/优先级，明确先不写代码。仅授权设计/计划/输入簿记；无实施、建树、安装工具、代理/消息其他会话或切默认授权。未给时间/性能比例/内存新预算。

2026-10-20初稿调查快照（历史，当前状态按Spec/Plan核验）：当时798在途r4、相关新Spec未交付，820–824 r3、825/826 r2为drafting。当前798已交付，828在实施，817/818仍是有限scalar消费者；已有core/scalar/record/text-record是AC源域，不是新AV完整语言。纯MIR fixture、native可运行或旧AVM有同名语言功能都不替代AV源程序实际运行。

依据：[Spec overview](../specs/overview.md)、[HIR阶段](../specs/auto-hir/stage-contract.md)、[AC source](../specs/auto-ac/project.md)、[旧VM](../specs/auto-lang/vm/overview.md)、[stdlib装配](../specs/stdlib/design/assembly-manifest.md)、[test-case](../specs/auto-test-case/architecture.md)、Design43/44及[23程序源目录](reports/av-source-program-catalog-2026-10-20.md)。路径/SHA、取号及文档预检见[inputs](attachments/av-language-roadmap-20261020/inputs.json.txt)、[preflight](attachments/av-language-roadmap-20261020/preflight.md)。

**D798**=最终独立review、冻结门禁通过、真实合入、Spec及merge收据齐全。T-01只读准备可提前，T-02起遵守D828排期门，T-03实际worker还须D798与本Plan实施授权；不等待817/818/820–826全完成，不消费WIP。共享API/注册窗口跟随实际814–816 owner；没有全批强依赖，持窗口不得等待上游。

## 5. 详细设计

### 5.1 G0库存、语义与oracle前门

T-01对现有VM corpus发现规则、CLI/SDK/REPL/服务/UI/模板/构建/FFI实际调用点及stdlib inventory/actual manifest建立可重放枚举清单。保留source/依赖/expected/入口/tool/profile/platform/features身份、适用状态和owner；parser扫描失败/无expected/未测为gap，不从分母删除。现存23种子必须原样登记，且源域类型/应用/外库样本区分。

T-02手审AP-01/AP-22各检查结果0、AP-23第一条短路不执行而第二条除零必须trap，冻结强expected与实际stage/code。native测试exit70与AV guest trap分通道，普通OS exit不冒充guest结果。库/环境复杂AP只登记缺口及后续owner，不在本Plan给全通过结论。

对SEM-01..08创建证据/权威来源/保留或改变选项/owner/阻塞门：wrapping与trap、负索引、str字段/拥有、共享捕获、入口/错误等冲突不能用旧expected覆盖798已批准语义。本Plan不实现legacy兼容profile，也不擅自批准语言破坏变更；有冲突的能力停在其所属后续G门。G1明确已有scalar共同域，AP-23按其已批准trap合同，不等待所有将来语义都裁定。

### 5.2 原始source首例与信任

T-03首例必须读取AP-01现存文件的冻结bytes（source原样，不改entry/循环/return），经实际source编译→MIR→ABC→AV返回typed i32 0；由既有native路径+手审各组gcd作为独立oracle。加入同store put/hit、真正新进程恢复重验后再执行和正式父侧比较。任一门不可行，记录具体输入/限制/owner冲突回new，不能改成直接执行MIR或旧VM。

run/source路径同时支持原始输入身份与root上下文，统一从冻结快照读取一次。请求准入后磁盘变更不能偷偷换成另一版；不同内容同路径必须有不同身份。源解析/编码/错误span引用当前source；bad UTF-8/越根/缺文件/IO按实际合同分态，不能当guest trap。宿主工具绑定来自可信配置，case不能自选任意exe/跨根文件。

unit/module kind、entry/imports、版本/producer及当前权限沿D798修复后的真实合同，不再猜函数数，不恢复trusted。AP-23有未走与必走trap，验证不提前求值普通运行期trap；真实AV执行次数/guest步数/结果/trace可观测，但字段标签本身不是新引擎执行证明。

### 5.3 首例扩展、cache与旧入口边界

T-04跑AP-22/23及必须的顺序/loop/坏输入反控，调用D798公用lowering/verify，不新增语言checker。修改源程序的私有overlay例如AP-01最终return0→9形成新变体/hash及独立expected9；不得修改仓库原AP。源更新必须影响实际结果/依赖，不把两个永远返回0的文件当变更反控。

T-05通过783 managed store/root/lease/GC复用代码，hit/restart仍执行AV并由父侧重新比较；源码/依赖/profile/工具/producer/entry/授权改变正确miss或拒绝，坏内层即使hash/frame自洽也不执行。source路径/位置变化重绑span，不序列化LoadedProgram或活帧。跨工程/根目录的复用不得串源身份。

旧avm/run/ac/probe及实际已交付817/818 recipe按其原支持面回归，不改其期望或扩面；老codec/profile按D798/已批准826版本矩阵。源码入口的成功不得由内部fallback到AVM/a2r/native实现；参考执行独立标识、隔离副作用，旧dispatch计数在新路径为0。注册若已搬迁跟随owner，不造永久双份worker/runner。

### 5.4 有界资源与可测性

沿D798实际冻结cap交集：单输入4MiB、模块类型+常量+imports+functions合计65536、每函数slots+blocks+instructions合计65536，组合verify≤200万步/5s；执行≤100万步/1024帧/2s，沿已冻结cancel检查间隔；受管hot8/staging16/exec8合计32MiB、活槽8MiB、worker增量128MiB、控制元数据/流/trace及父侧绝对截止沿真实profile，不凭本段提高数值。T-01必须逐项核实际单位和源构造128层适用门，不能把表数值域当合法ref范围。

读取/冻结source、编译/解码/索引、allocator/参考工具及临时对象分别测真峰值；不能扣掉前端达成进程门。已知原前端峰值未达若尚未收口则D798不成立；记债不豁免。外部native构建分列真实成本/期限，不以进程强杀作guest trap/Drop证据。

普通准入非法dispatch=0；已入AV的trap/cancel/resource保留合法前缀、危险目标访问/提交/子调用为0并实际释放帧/lease/owned资源。0预算前拒申请attempt=0，分配故障attempt存在但successful_alloc=0，先前合法事件不抹掉。

沿798固定256KiB/100有意义函数的已批准标量编辑负载及正式session口径复测两条原P95<1000ms门，≥30样本及raw全保留；本Plan薄source入口的IPC/IO/准入不从端到端扣除。新小文件/冷态数据另报，不拿gcd毫秒数代替原负载，不扩大承诺到完整应用。资源或周转不可达先带数据回new，不删样本、抬cap或跳frontend。

### 5.5 实际验收清单与Script Scope

T-01冻结program/capability/entry/SKU→case/子case→Scope→真实入口→phase/code→独立oracle→输入/工具hash清单。future脚本 `pwsh -File scripts/verify-av-source-827.ps1 -Scope first-case` 由T-03建立，当前未运行；Scope为inventory/first-case/semantics/admission/resources/cache/compatibility/autodown/acceptance。inventory只证明清单，first-case只证明AP-01首例，不代表全部源程序通过。

semantics=V-01..03及顺序子例；admission=V-04..06；cache=V-07/08；resources=V-09；compatibility=V-11中实际旧consumer子例；autodown=V-10；acceptance核V-01..12全部必需子case和V-12差异状态。不适用的未来AP状态是gap/unsupported，不当当前正例PASS或删出未来替换分母。0例/漏例/skip/重复冒计/任意Err不得绿，脚本和父runner失败退出码透传。

### 全链路增量兼容与828优先排期（r2）

798实际交付后优先828，本计划在原依赖和D828均满足后取得共享写窗口。D828=当前revision最终review/冻结门禁/实际合入/Spec与merge收据齐全，不消费WIP；只读调研/合同准备可先，已有准备保留。共同合同见[ACE顺序与兼容](reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)与[828](828-av-function-incremental-session.md)。

专项：真实源码热编辑复用828会话，不每次new Driver/整source编译；文件revision/来源/父侧actual绑定保留，cold/run另列。

T-09 是work必做项：接入D828公共编译/装载/snapshot接口，保留稳定函数身份、私有凭证与实际依赖。纯body编辑、签名/实际依赖/profile/配置/授权不变时，未改HIR/MIR/代码对象和代际不变，未改生产阶段及读盘/decode/verify/load为0。元数据更新单列，不能用全source hash/总revision使所有函数失效。新工件仍走适用准入，恢复不可信。

四必需子case：1)D828真实source edit基线273575B/115函数，至少30不同修订，结构与稳定会话P95<1s双门；2)本计划新增能力真实支持入口两函数局部body更新，未改单元保留且语义/清理正确；3)实际接口/layout/helper body/授权事实变化的精确失效/拒绝正反控；4)强制whole前端/lower/重装载或只保hash却换对象/代际，正式gate必非零。仅手工HIR/MIR支持的新类型不能冒领该类型完整源码能力，但scalar全链仍须重放。由T-09 建立真实入口计数/身份/IO/反控，当前未有新增证据，review不得豁免或推给merge。

原支持面/资源/原生等性能门不放宽；cold/restart/版本迁移或真实广泛失效另报，不掩盖正常body退化。旧帧/值/drop引用期间代码与布局保活，失败/取消/迟到不安装，退出后回收；不兼容布局按实际支持策略拒绝/扩大失效，不擅自迁移活值。

### 执行排期与单写交接（2026-10-21）

优先批次的真实源码验收线，须D798和D828，不等817/818或820–826全部完成。薄消费层/程序清单与独立验收可并行；最小公共导出/recipe注册串行交接。只认领现有scalar源码，数据链通过不自动扩大本计划源码范围。

所有后继在D828前仅作只读准备，现有实现与证据保留。共用[执行顺序与交接合同](reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)：T-01准备写面/owner/消费者映射，T-02在D门满足后刷新基面、版本、预算与接口并记录窗口接受；既有兼容/coverage工作记录定向结果，独立review核记录，merge补实际合入与窗口释放。公共核心默认单写，公共注册追加也串行；不同worktree不豁免。不能持窗等未交付依赖、复制WIP或改变他方在途合同；无公共写面者可并行消费稳定API。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/language-support.md | 无源码/内核/产品分层实证→登记本Plan实际source支持、尚缺能力及固定分母/证据等级 | 不以typed fixture冒领源码 | AC-01/02/06 |
| SD-02 | add | docs/specs/auto-av/source-program-conformance.md | 无通用source程序消费→限定scalar文件入口/快照/强oracle/资源/父侧判定及明确拒绝 | 真程序闭环 | AC-02/03/04/05/06 |
| SD-03 | modify | docs/specs/auto-av/cache-and-conformance.md | D798已交付→追加source-file身份/依赖/位置/恢复及反控 | 不缓存PASS/trusted | AC-04/05 |
| SD-04 | modify | docs/specs/auto-av/project.md | 实验typed执行→增加实际交付source入口，保留stdlib/UI/完整语言/默认切换未交付 | 如实支持面 | AC-02/06/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有probe/run保持→登记实际复用或必要新增的source recipe/worker（候选av/source-run-v1）、typed父侧oracle/输入来源与边界 | 不复制协议或旧解释器 | AC-02/04/06 |
| SD-06 | modify | docs/specs/auto-av/resident-function-install.md | 继承D828函数IR/代码复用、受检安装/依赖/寿命；新增能力不退回整模块热重编/装载 | 用户明确的顺序与增量机制保留 | AC-09 |

Spec路径为proposed，当前尚未全部存在，D798后核modify/add；review最终确认，merge才沉淀。GOAL-002仅记有限源码周转贡献，不据此核销热重载/调试或整VM替换目标。

## 6. 测试设计

排期交接沿本节既有兼容性入口与独立review核查：对照记录中的实际基面/API/版本/消费者验证结果；公共写面无交接、依赖未交付或旧证据失配时不得判就绪。不新增整档测试；原增量、AutoDown、P95/内存及反控门保持。

本轮未运行。所有fixture/工具反控由T-03..06建立，不能用未来接口文字作运行证据。每V含合法对照、实际坏输入/故障及独立oracle，Scope按§5.5；必须保存selected数和输入/工具/产物身份。

| Case | AC / owner | 输入/实际入口 | 可区分错误实现的oracle | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-02；T-03/04 | AP-01原source file→实际source compile/verify/MIR/ABC/AV/native | guest i32=0、数学checks手审；stage spans/字节身份可追溯，非手填MIR | 第一个真实程序 |
| V-02 | AC-02/03；T-04 | AP-22原file；另D798原trace授权源例 | guest0、命名/位置/嵌套/多行语法，trace顺序精确；修改实参源值改变独立结果 | 仅当前scalar支持面 |
| V-03 | AC-03；T-04 | AP-23原file及相同false-only衍生正控 | safe RHS未执行、第二条除零guest trap；false-only guest正常，native exit70独立匹配 | 不以任意Err/进程timeout充负例 |
| V-04 | AC-04；T-03/04 | 错source hash/同名不同bytes/冻结后改磁盘、非法UTF-8/越根/缺文件 | 固定快照不漂移，准入/IO明确阶段码；拒绝时AV dispatch0，哨兵原样 | 未创建symlink、不写原源码 |
| V-05 | AC-03/04；T-04 | AP-05或print/List输入送scalar源入口；未知profile/capability/错entry | 真实前门拒绝、不fallback旧VM、无host副作用；unsupported不计程序成功 | 不扩语言与宿主 |
| V-06 | AC-04；T-04 | 好hash坏MIR/ABC、手工Decoded/伪kind/错producer、坏actual/frame | 消费门真实拒绝dispatch0，合法单函数完整module正常；不按函数数猜kind | 沿D798完整kind修复 |
| V-07 | AC-04/05；T-03/05 | fresh/hit/new-worker restart及同路径overlay0→9 | actual从0变9，输入hash真变、依赖重键；hit/restart仍执行且每次父侧比较 | 不缓存PASS/actual |
| V-08 | AC-04/05；T-05 | 改profile/工具/依赖/grants、跨根/工程、坏内层与位置变化 | miss/拒绝恰当，授权撤销不执行，source span新绑定，root/lease不串 | source与runtime身份分别核验 |
| V-09 | AC-05；T-04/06 | cap内/超1、无限循环/fuel/cancel、故障第n次、瞬态spike；原256KiB编辑两路径 | 预算前拒/实际清理/峰值真值，30样本原P95门及阈值0反控非绿 | 不扣frontend/IPC，native分列 |
| V-10 | AC-04/06；T-03/06 | 正式recipe正例、错actual+好expected、nonce/hash/worker错、漏子case/skip | 正例真实dispatch；每项注入让父侧总体非零/ERROR，错误实际发生有raw | worker不读取expected造结果 |
| V-11 | AC-01/06；T-01/02/05/06 | 全量inventory扫描；缺目录/解析失败/缺oracle；实际已交付旧recipe/817/818 | 分母不静默变小，未知有owner/gap；旧支持面不变，新recipe不扩老协议 | 23种子非全语言分母 |
| V-12 | AC-01/03/06；T-01/02/06 | SEM-01..08差异材料、wrapping-vs-trap/负下标/str字段/旧误报见证 | 正确归类profile/未决门，AP-18不机械当新规范；超范围语义不擅自实现 | 本Plan不声称已兼容全旧语言 |

work负责实现与反控；内部检查只表示可送审；最终review须独立上下文、同唯一树冻结输入核验，不修代码；merge负责冻结门禁/Spec/归档/guard清理。未指定人/模型，当前未创建代理。work/review仅定向测试、fmt/check；例如独立manifest的 `cargo test ... av_source_827` 路径由T-01真实冻结，selected=0不可PASS，不能照未来包名假跑。

最终review后merge按实际diff统一跑根裸cargo t及编译器触面tv、新独立workspace完整有限合同及实际相关consumer门；无相应Rust触面不机械加整档。tf只主检出到期单实例；不跑旧进程内双重解释塔/aavm全档，无UI/Rust生成器/语法参考变化不跑tu/docs_gen。

### 本轮新增的增量兼容验收（work-owned）

| Case ID | AC / owner | 输入 | 实际入口 | 区分性断言 | 边界 |
|---|---|---|---|---|---|
| V-13 | AC-09 / T-09 | D828 source30修订+新增能力两函数+依赖变化+强制退化四子case | 本计划真实consumer/正式recipe与D828 edit接口，由T-09 接线 | 未改对象/代际相同、未改函数生产阶段/产物IO=0，结果/清理正确，P95<1s；强制退化非零 | 当前profile真实起点，不冒领未交付源码/迁移能力，四子case无skip |

## 7. 验收标准

| ID | owner_stage | 可观察完成标准 |
|---|---|---|
| AC-01 | work | G0库存/owner/差异清单可由真实发现规则复现，C/AP/EP/库符号分母清楚，缺项保留gap；V-11/12 |
| AC-02 | work | 原source AP-01/22得到typed0/0，AP-23准确trap，实际共享管线/byte身份、首例持久化/新worker/正式recipe；V-01..03/07/10 |
| AC-03 | work | scalar语义/顺序、trap/unsupported与SEM边界保真，合法对照不误拒，无旧VM fallback；V-02/03/05/12 |
| AC-04 | work | 源/产物/worker/current grants/kind及实际父侧准入成立，非法准入dispatch0，运行期目标故障与合法前缀分开；V-04..08/10 |
| AC-05 | work | current profile资源/32分区/128进程峰值与原编辑P95、异常清理/root/lease/真实restart全部满足；V-07..09，不能凭归因豁免 |
| AC-06 | work | 原/新recipe兼容、Scope必需清单及父侧反控、强oracle/覆盖报告可复算，0例/skip/缺必需工具非绿；V-10..12 |
| AC-07 | review | 独立逐AC/V/SD、遗漏/延期/workaround及scope审查，绑定HEAD/依赖/配置/输入hash，无未处理in-contract finding；merge-owned门禁pending明确 |
| AC-08 | merge | 冻结门禁通过、SD沉淀/实际合入归档与索引/收据齐全，wt-guard clean后清树；未切默认或冒称AVM替换 |
| AC-09 | work | D828与新增能力保持全链路增量，V-13 四子case及结构/身份/资源/时间/反控raw齐全；未改函数不重编/重装载，时间绿不替代粒度绿 |

## 8. 执行步骤

| Task | owner_stage | 前置 | 文件/操作与实际结果 | AC / 验证 |
|---|---|---|---|---|
| [x] T-01 | work | 本Plan实施授权；只读准备可早于D798 | attachments/827/contract-decision.md **r2**（r1 冻结复用裁定/零公共写面/薄适配面/预算/API 逐项 file:line + review-r1 修复升级：逐项库存 schema、V-13 身份检测器合同、汇总器 skip 必红规则，见其 §8）；SEM-01..08 决策材料 sem-decisions.md | AC-01..06；V-11/12 合同与材料就绪（R1-F03 修复后） ✅ |
| [x] T-02 | work | T-01只读材料、D828排期门、当前单写窗口交接 | contract-decision r1 直接写在 D798+D828 合入后基面（无 pre-D828 旧准备需刷新，记录于其头部）；oracle 手审冻结（AP-01/22=success 0、AP-23=av.trap.div-by-zero、overlay return 0→9 恰一次）；SEM-01..08 决策材料 sem-decisions.md（AP-18 保留见证、18 旧 AP .expected=待核输入 gap）；写面登记=无公共写面、注册面在途 owner=820 不相遇（contract-decision §1） | AC-01/03；V-01..03/11/12 材料齐，未决项归后续 G 门 ✅ |
| [x] T-03 | work | T-01/02、D798、当前注册/API owner交接 | 薄适配落地 @f91215eac：corpus `av-source-827`（AP-01 字节恒等副本+SHA 回执，recipe=既有 ac-av/core-run-v1）；`av_source_827_first_case` 4/4 绿（种子身份双向/真实文件 ephemeral 0+计数/同 store put→hit 仍执行+真新进程 restore-run 0/双单元复验/native 通道 0）；根门 `av_source_827_corpus_green` 正式 recipe 父侧判定 1/1；脚本 first-case 档 selected 非零强制。零公共写面维持 | AC-02/04；first-case、V-01/04首轴/07首轴/10首例 ✅ |
| [x] T-04 | work | T-03 | @ff1c9a06e：AP-22/23 入 corpus（success 0/精确 div-by-zero trap）；`av_source_827_semantics` 7/7（实参变体改值改果/AP-23 trap+false-only 正控+native exit70 双通道/trace 授权 ABA 精确序+无授权=source.undefined/同路径异 bytes+冻结快照/前门拒 dispatch=0/超尺寸 admission 拒/无限循环=InstrSteps 资源态+预置 cancel=Cancelled）；根门 4/4（UTF-8=infra、缺文件=resolution Error、`../`/绝对/盘符=decode escape 拒）；错 entry 面=probe 侧精确断言（mir-lower:lower.entry），corpus 词表无 verify 面不扩公共词表（记录于案例注释） | AC-02..05；semantics/admission 定向 V-02..06 ✅ |
| [x] T-05 | work | T-04 | @d7a071763：`av_source_827_cache` 5/5（overlay 恰改一单元/caps 变更全重键/同键异内容 Conflict+好件存活/跨工程=内容寻址复用仍执行/位置平移新身份）；corpus session-overlay 正式面（每 check 新 worker 共享 warm store+父侧判 9）；脚本 cache/compatibility 档接线 | AC-04/05/06；cache/compatibility V-07/08/11 ✅ |
| [x] T-06 | work | T-05 | 修复轮 @R1（在 5f4c32e84/bd396d732 基础上）：根门 8/8（corpus 必需清单+**重复 ID 拒绝**+负 oracle+故障注入+**逐项 inventory 714 项 JSON 工件**（23AP+14fixtures+47 类目行+517case 行+37provider 行+43C+10EP+10G+13 锚点行；失败保留 scan-error 行/缺行补全 gap-missing-row 必红/无 oracle 行 owner=future-G）+**inventory 五类负控 2/2**）；脚本 acceptance **串联含 resources 全 scope**、`Test-Selected` skipped/ignored>0 必红（0 选择亦红）、**`selftest` 档永久重放**（valid/skipped/ignored/missing/真实失败子 scope 经同一 Invoke-Scope 路径传播全红）、raw 时间戳留档；bench 30 样本门 970.7ms<1000（首轮 1007.2 瞬态归因留档）；fmt/check 净 | AC-01..06；acceptance 聚合无洞（R1-F01/F03 修复后） ✅ |
| [x] T-09 | work | D828及本计划其他work实现完成 | 修复轮 @R1：`av_source_827_incremental` **5/5**——(1.1)D828 冻结基线 30 修订双门（结构+P95<1s）**且未改函数 (ptr,generation) 双轴跨全部修订对原始 warmup 保持、改函数必动；每编辑 phase_io 增量 rescan<len/100 且零 store IO** (1.2)两函数局部更新恰一安装/他函数零阶段 (1.3)签名/未授权=记录性拒绝且修订保留 (1.4)**强制退化负控**：重 warmup 每编辑=真实全体换对象（≥100 findings）→身份稳定门（镜像 828 judge edit_identity_gate；warmup 冷路径零计数/零IO 已实测记录）红且值仍正确；**同 hash 换对象判别力**=双 warmup 同 content version 不同 ptr；快径编辑两门皆绿；NoConvergence 形状边界注记保留 | AC-09；V-13 四子case 含真实退化注入与身份/代际/IO 断言（R1-F02 修复后） ✅ |
| [ ] T-07 | review | work-owned完成、同树冻结提交、T-09 完成 | 独立复核实际AC/V/SD、源字节/执行真实性/阶段/gap及健康；记录revision/依赖/证据 | AC-07，merge-owned整档/收尾pending |
| [ ] T-08 | merge | T-07 pass | 冻结提交门禁→SD/ledger→真实合入/归档→到期批回归→wt-guard清唯一树/分支 | AC-08，实际收据，未达不得称交付完成 |

2026-10-25 R1 修复轮完成：work 7/7 恢复全勾（T-01/T-06/T-09 按 review-r1 三 findings 修复后再勾，证据见各行"修复轮"），status=execution_done，next=T-07 独立复审（R1 报告的修复重放要求逐项对照：F-01=selftest 五类负控+acceptance 含 resources+重复 ID 门；F-02=身份/代际/IO 断言+真实退化注入+同 hash 判别力；F-03=逐项库存 714 项+失败保留行+五类负控）。merge-owned 门禁保持 pending（冻结提交裸 cargo t+触面档、tf 到期判定、SD 沉淀）。源码/测试/build 只在本 Plan 唯一树；主检出仅簿记，不创建 junction/symlink。后续 Plan 应复用这条 source 验收路径/登记支持 profile，不建各自假 runner。

## 9. 复审记录

2026-10-25 stage:work（R1 修复轮） | PLAN-827 r3 | outcome:pass（三 findings 全闭包，送审就绪） | worktree:`D:/autostack/.wt/lang-827/auto-lang` | branch:`plan-827-dev` | base:369e1c349 | code:修复提交于 bd396d732 之上（见 git log "R1 修复"） | 闭包行（finding→invariant→修复→重放）：
- **F-827-R1-01**（汇总门漏 resources/skip 可绿）→ invariant=acceptance 覆盖全部必需 scope 且 skip/重复/缺失不可绿 → 修复=`Test-Selected` 拆分（skipped/ignored>0 必红、0 选择必红）+ acceptance 经 `Invoke-Scope` 串联**含 resources** + corpus 门重复 ID 去重断言 + **`selftest` 档**永久重放五类负控（valid 绿 / skipped 红 / ignored 红 / missing 红 / 真实失败子 scope 经同一 Invoke-Scope 路径红）→ 重放=selftest PASS + 根门 8/8；
- **F-827-R1-02**（V-13 缺真实退化注入及身份/代际/IO 断言）→ invariant=未改函数对象 (ptr,generation) 双轴稳定+生产/IO 事实+强制退化必被检测器抓住 → 修复=基线 30 修订逐修订全函数身份绑定（对原始 warmup）+phase_io（rescan<len/100、零 store IO）+`identity_findings` 身份稳定门（镜像 828 judge edit_identity_gate；实测 warmup 冷路径零计数/零 IO，指针稳定即该类退化的判别轴）+**真实强制退化注入**（重 warmup 每编辑=全体换对象≥100 findings→门红且值仍正确）+**同 hash 换对象判别力**（双 warmup 同 content version 不同 ptr）+`whole_module_findings` 计数器门（会话内 fallback 形态）→ 重放=`av_source_827_incremental` 5/5；
- **F-827-R1-03**（G0 只有数量摘要）→ invariant=分母成员逐项持久化、失败保留行、owner/gap 不删 → 修复=逐项扫描器（AP 23 行含 SHA/nature/oracle/owner、三 profile fixtures 14 行、类目行 47+case 行 517、provider 37 行、C-43/EP-10/G-10 逐行解析含 owner/gate、13 锚点行带符号发现）→ 工件 `docs/reports/plan827/inventory.json`（714 项）+scan-error/gap-missing-row/future-G 保留语义+**五类负控测试**（缺目录/坏 JSON/无 providers 数组/无 oracle case/缺表行）→ 重放=inventory 2/2；
其余未受影响证据复用（T-03..T-05 各档未改，同轮重放绿：probe 21 测+根门 8/8）；统一回归仍 merge-owned pending | blockers:无 | next:review（R1 报告"修复重放要求"逐项可对照）

2026-10-25 stage:review | review_scope:final | reviewer:独立于实施的当前聊天上下文 | PLAN-827 r3 | outcome:needs_fix | reviewed_commit:bd396d732cc82883b1e30e6be7e1f80a5df853d1 | base:369e1c349ea9587907f073aaecd3082e2bb486bd | worktree:D:/autostack/.wt/lang-827/auto-lang | branch:plan-827-dev | findings:F-827-R1-01..03（汇总门漏resources/skip可绿；V13缺真实退化注入及对象/IO身份断言；G0缺逐项库存） | T01/T06/T09重开，current_step=4/9；旧7/7仅为历史work声明 | evidence:[R1报告](attachments/827/review-r1/review.md)、冻结SD及hash见报告 | 定向退化控制1/1通过，摘要skip反控复现错误绿；其他未验项pending，不宣称全AC通过 | merge-owned统一门禁pending | next:work同唯一树修复后独立复审，无合入/归档/清树。

2026-10-24 stage:work | PLAN-827 | plan_revision:3（合同未改，T-01 冻结复用裁定于 attachments/827/contract-decision.md r1） | outcome:pass（work-owned 全清，送审就绪） | worktree:`D:/autostack/.wt/lang-827/auto-lang` | branch:`plan-827-dev` | base:369e1c349（master） | code:5f4c32e84（含 f91215eac/ff1c9a06e/d7a071763/5f4c32e84/bd396d732 + master 侧簿记 52ddab977/f0c0f9dc7） | deps:D798/D828 归档交付，只读消费；无公共写面（注册面在途 owner=PLAN-820 未相遇） | tasks:T-01..T-06,T-09 全勾（7/7） | evidence:probe 4+7+5+4=20 测全绿；根门 7/7（corpus 8 checks 必需清单+3 准入负控+负oracle+4 故障注入+inventory JSON 回执）；resources 档 bench 30 样本 P95=970.7ms<1000（首轮 1007.2 瞬态：同轮 fresh p50 4904→4784，未删样本、未抬门，raw 入库 docs/reports/plan827）+828 正式战役 V-08 1/1+gate-0 必红反控；fmt 净/cargo check -p auto-lang 绿（预存警告未增） | blockers:无 | 记录性发现：①probe 声明式 stage 词表无 verify 面→错 entry 面留 probe 侧精确断言（mir-lower:lower.entry）；②AP-01 main 尾部 return0 会话快径编辑=NoConvergence 记录性拒绝（整源/其他锚点正常，测试内注记，后续 syntax owner 可查）；③2026-10-20 路线图权威文档原为未跟踪态，已原样入库（f0c0f9dc7）并合入计划树 | merge-owned 门禁 pending：冻结提交上裸 cargo t+触面档（无 VM/编译器语义改动，预计 t+tu 不需要）、tf 到期判定、SD-01..06 沉淀 | next:review

2026-10-21 stage:new | PLAN-827 r2→r3 | outcome:pass（排期/交接合同静态修订，非实现验收或依赖解锁） | authority:用户确认828后优先817/818、827和820功能线，825/826择机插入 | changed:§4授权、§5排期/共享单写、§6交接核查、T-02及§10当前入口；T/AC/V/SD ID、数量、状态和原进度保留 | next:只读准备可先，D828及本计划原D门/单写交接满足后work；不自动启动 | evidence:[本轮预检](attachments/ace-order-20261021/preflight.md)。

2026-10-20 stage:new | PLAN-827 r1→r2 | outcome:pass（规划修订，非实现通过） | authority: 用户指定798→828优先/后继保留全链路增量 | delta: AC-09 / V-13四子case / SD-06 / T-09；review依赖新work项，D828排期/集成门，原D门不减 | 原进度保留，total 8→9，新证据pending | next:只读准备可先，D828后串行实施；无代码/测试/Spec发布/合入。

2026-10-20 stage:new；plan_revision:1；outcome:pass（文档合同起草预检，非实现/独立验收pass）；next:在实施授权和D798满足后work T-03，其前仅明确授权范围的静态材料准备。中央取号器在互斥/独占allocation.lock下分配827，保留空锁文件；当前未开树/编译/测试/改canonical Spec/默认/归档。证据见[preflight](attachments/av-language-roadmap-20261020/preflight.md)。

## 10. 待澄清事项

无阻断本轮文档交付的问题。T-01需在真实D798交付后确认导出接口/源码profile/注册owner、精确错误码/资源单位及工具；API不足先做有界whole-path验证，不能造mock或抄在途树。SEM迁移的具体语义选项和产品engine-selector名称由后续具名Plan在使用前冻结，超出已批准语义的改变另取得用户决定；本Plan不代替批准。完整AVM替换范围须有G0全量分母，不能由23种子或43类别穷尽承诺。


**当前排期与验收（2026-10-21）**：D828后按§5本计划优先级/原D门及单写交接推进，不等待无硬依赖的整批；T-09 / AC-09 / V-13 未完成，旧revision/测试总数不覆盖新增义务。用户已确认该顺序/目标，不重复请求同范围授权。
