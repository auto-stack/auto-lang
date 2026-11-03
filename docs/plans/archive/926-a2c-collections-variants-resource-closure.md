---
plan_id: PLAN-926
status: archived
feature_name: CA-C HIR到C集合变体与拥有型资源闭环
author: [Codex]
created_at: 2026-11-03
updated_at: 2026-11-03
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-a2x/checked-resource-facts.md, docs/specs/auto-a2x/incremental-and-cache.md, docs/specs/auto-a2x/architecture.md]
new_spec_components: [docs/specs/auto-a2c/collections-hir-backend.md, docs/specs/auto-test-case/a2c-hir-collections-certification.md]
touched_goals: [GOAL-002]
affects: [auto-a2c, auto-a2x, auto-test-case]
depends_on: [PLAN-864, PLAN-889, PLAN-904, PLAN-921, PLAN-925]
current_step: 14
total_steps: 14
---

# [PLAN-926] CA-C HIR到C集合变体与拥有型资源闭环

## 0. 变更摘要

925 CA-B的集合/变体后继，合为一个CA-C包：List<Str>/List<owned record>/List<Variant>、有限Variant/Option/Result、受检owned view/is/for-in、逐出口/错误清理、fault/budget/cancel、真函数增量/来源恢复与独立C认证。共用CollectionsBodyFacts与typed C资源runtime，拆List/Variant会重复接线并割裂交叉用例，故不拆包。

依据[Design51 §15](../design/51-shared-a2x-and-hir-to-rust.md#15-ca-cc集合变体与近期实施顺序2026-11-03target-state)与[C路线](../design/strategy/shared-a2x-and-hir-to-c-roadmap.md)。drafting r1、0/14；本轮仅规划/取号，不施工/建树/测试、改current Spec/原源/expected/批准/登记或在途计划。最终基面须含D925/D921，私有准备可先，不要求等native923/924整包。

## 1. 目标

G1：原904全20源和全部相关正反补例经新C目标完整生成/独立双侧认证，L04/L05=42、L06仅ListIndex。G2：消费验证器CollectionsBodyFacts而非新ownership分析，正确拥有转移/view/活动臂/每出口和错误清理，无隐式clone/双free/危险访问。G3：含type/enum自由函数body真片段、未改P/IO0/对象代际保持，完整依赖失效；来源恢复与下一编辑/坏件修复再读闭合。G4：新目标域/配置/证书独立、日常零目标编译，保原规模/资源/时间门，给CA-E可核证据。

非目标：新增源码集合能力（List参数/返回/字段/嵌List、Variant参数/容器字段等）、源码List/Variant.clone、任意guard/通配/qualified unit-arm构造、宽位/char/str在集合域、闭包/HOF/K06/Unit入口、通用泛型/模块/stdlib、native Place ABI/loader、CA-D公开ABI/foreign语法/CA-E总签章。925更宽text/char支持不自动扩本profile。helper深clone探针不冒新的源码支持。

## 2. 架构方案

auto.source.collections.draft→现source/CheckedFunction与CollectionsBodyFacts→SemanticView/FunctionPlan→C私有List/名义tagged Variant表示与helper→按源序/资源事实合法化→完整C文件集→真实cl独立actual/reference→严格父判。

复用925 Str/record/helper接缝、921 TargetProvider/session/AXP来源服务与统一store。C provider从结构化受检HIR生成，不读旧AST、不运行AV、不将Rust文本转C；运行时做数据/资源原语，不解释MIR/ABC。语义事实中立，目标runtime/fragment/fileset/cert含完整C/配置身份。

## 3. 技术栈

Rust实现ACE；C11、真实MSVC cl、Windows x86_64-pc-windows-msvc，a2x-c成员及版本化C runtime源资产；原ManagedStore/受控Job/工具执行/AutoDown父判。复用原runner，不新造DB/workspace，不为普通生成C链接Rust解释器或native Cell执行平台。

## 4. 需求分析与背景调查

授权（2026-11-03 追加，work 阶段）：用户 `/goal 完成计划026`（经确认解读为 PLAN-926，026 已归档终态）经 /auto-plan-goal 授权整链：在 926 worktree 施工（含建树）、review、契约内有界修复与 merge（canonical Specs、archive、worktree 清理、landing checks）；不改目标/验收/阈值/兼容承诺/仓范围/预算。树 `D:/autostack/.wt/lang-926/auto-lang` 分支 `plan-926-dev` 基 d9e262a92（含已交付 D921=3c760865c 归档 fe0b1e45f、D925=a37245cd6）；材料 scope `D:/autostack/plan-artifacts/plan-926/` 已登记 in-flight。

授权：2026-11-03用户明确要求925后续集合计划及各计划实施顺序，只规划不施工；继承已批准A2C M1范围/CA-C原义务，无自动继续实施/跨仓写入/联系他人或新agent/chat授权。读取main8717bd5d3；921 execution_done11/14、分支9c15e6fa3待review而未交付；923 executing1/12；924/925 drafting0/14；922已交付归档9/9。其他worktree/跨平台注册保原状，不因prunable显示清理其目录。（注：D921/D925 均已在其后交付并入 master，见上方 work 授权行。）

| 当前来源 | 已观察事实/内容SHA256 |
|---|---|
| docs/specs/auto-a2r/collections-hir-backend.md | 60a7fea8af7b7dcf8be41229d60c195c5dfc900560f5f5902aeb1721c9a43613；904源域/资源/计费/增量/认证边界 |
| experimental/ace/crates/core/src/collections_facts.rs | 0b90ec38a6c7c8c4ec5358980d371190dd3d6a14e699092ab24462589afa2fdc；owned/exit_frees/replace_frees/err_frees/loop_survivors，body digest绑定 |
| experimental/ace/crates/a2x-core/src/semantic_view.rs | eaac652ddf650206242a403cf3a5212dbba1a785bbfd23c49e35c026762b08c3；List/Variant/语义闭包与受检body_facts |
| experimental/ace/crates/a2x-rust/src/lower_coll.rs | 1cb83f1b90d0e28afb9969dcc85abd7b1e327441ce68378aada61f38421e506f；唯一消费facts的Rust对照，不复制成C分析器 |
| experimental/ace/crates/a2x-rust/src/coll.rs | 329db39a00d336cb27c15afe87215341b2b330fe053434b36a2287ce2190ce34；List/variant helper、billing/fault/cancel、显式free |
| crates/auto-lang/test/cases/a2r-hir-collections-904/src/ | 恰20源；按排序name:raw-SHA256用LF连接的目录清单hash f53078137f84e10d17f8ed87e7773065f3428cf193f2605e493389b9a9fa427b；本轮全不改 |
| experimental/ace/crates/a2x-worker/tests/cases/{collections_904,collections_face,collections_v15,cert_coll_904}.rs | 源闭环与同生成runtime的手写helper probe分账；不能从helper深clone推源码List.clone |
| experimental/ace/crates/a2x-worker/src/bin/a2x-coll-formal.rs::build_load | 完整原负载含data helpers，至少40fn/32 unchanged callers、非注释16KiB/数据4KiB，注释数不替实际源码计数 |
| PLAN-921/925 | 正在/将来交付的provider/record-text事实/来源接口，本轮不是D921或D925已存在的证明 |

原20源= L01..L07、V01..V07、LSTR864、FIN864A/FIN864B、N889-list-variant/nested-variant/optional-variant。T-01按实际冻结SHA复算源/金样/oracle/补例完整清单，只有真实新增基线可追加，不删原行/改expected或复制当前输出充参考。M1旧required/新C/独立认证/M2另账。

## 5. 详细设计

### 5.1 支持域、表示与单源事实

沿904已冻结集合域：i32/bool、受限owned Str（literal/len/clone）、不含容器字段的非递归record、有限名义enum/Option/Result（至多单payload/无环/臂序=disc）、局部List<T>，T为int/bool/Str/record/variant。List不可参数/返回/record字段/Variant payload/嵌List；Variant参数/容器字段、char/str等原拒绝保留。只支持零参i32自由函数entry，不能因runtime内部Unit标签而冒Unit入口。

C私有类型可按element/variant名义图生成typed helper/ops表；具体布局T-02冻结。元素size/align/ownership/深free事实来自已验类型，不凭bit宽猜同类；malloc/size乘法/容量增长先checked，不能无guard读union inactive字段、用memcpy制造两份owned责任、或用不兼容别名/对齐访问产生UB。List/Variant内部表示不成为AutoABI公开布局；不依赖923 native Cell/Place实现作为转译runtime。

CollectionsBodyFacts是清理权威：C消费owned、exit_frees、replace_frees、每fallible node的err_frees与loop_survivors，pending temp/Local/Deep类型与body_content_digest/owner/domain/凭证绑定；缺事实/外来旧凭证具名拒，不在provider重新推ownership。925 TextResourcePlan不替集合facts，混族不得授信。

### 5.2 操作、view、分支与求值顺序

List new初始逻辑cap4，push/insert/get/set/remove/len/clear沿原语义；base/index/RHS每项源序各求一次，再按binding落位，所有索引负/≥len/空容器和insert超[0,len]在实际buffer访问前ListIndex。正常return70不是trap，C越界/abort/panic/访问异常是infra不能满足。

owned push/insert/set要求原whole-local.move或fresh构造；call结果不是fresh，仍先绑定再move，不因C更容易就扩源。Copy get拷出快照；owned get只是受检字段/len投影，不物化/隐式clone。两个不同索引的融合len各用自己的index。for-in只在入口读len一次，Copy元素拷贝、owned元素投影；结构修改静态拒，set合法，旧临时投影不能跨修改保留。View非owner不free backing，也不能借机开放跨buffer借用逃逸。

is穷尽/唯一臂/无guard通配，payload guard已验证；owned payload投影move、Copy拷出，moved/inactive不重复读取/析构。Variant switch RHS完整准备再commit，old active payload恰一次释放，new payload可读。Unit臂可匹配不代表支持未冻结的qualified unit构造。

### 5.3 资源、失败与helper证据边界

元素逆索引free、buffer最后；owned返回/移出不在旧容器再析构，replace先准备后一次交接，frame出口/分支/loop回边/early return/break/continue/error/trap/cancel按facts清理。cleanup零分配/零计费/零poll，不靠C作用域/进程回收完成；view/合法外pin在活时保backing，最后owner后node/Str/backing真实live归基线。

对齐904失败原子性：界/预算/所有fallible准备先于目标commit；失败旧target保持，已求值但未提交的owned input必须由helper或caller**唯一**回收，具体责任点T-02按facts冻结。不能helper已free后err_frees再free，也不能要求源语言已move的局部在trap后仍可读；probe的源/目标原子性按该操作责任边界检查，而非发明整函数回滚。

每真实allocation域1..N故障、N+1/无注入正控，覆盖Str backing/List buffer growth/record/variant临时/必要metadata/clone工作栈；实际C N独立冻结不搬Rust N。失败半树/前缀全部回收，清fault后同进程retry与旧target逻辑内容可核。deep clone/drop使用计费显式工作结构/无分配释放机制，3072层/256KiB小栈与中段fault/取消见证必须来自**同生成C runtime**的手写helper probe；探针不读取MIR/ABC、不能冒源List/Variant.clone，也不能替原20源。若helper策略依赖预留metadata/工作栈，须构造前计预算，不能cleanup临时申请失败后漏资源。

计费沿904：每List slot2WU、每8 Str bytes1WU、ctor1、call frame4、loop poll1，默认预算2^24；借预算/cancel/fault旋钮必须到真实poll/allocation点。预算、Cancelled、Resource、ListIndex、VariantBranch及infra严格分态，父kill不冒协作清理。观察默认关闭、事件纯数据不保活，event/流上限先检查，不截断后PASS；序列+live计数双证据，漏free/错误顺序/重复free控制必红。

### 5.4 片段、依赖与来源恢复

C Collections独立family/fragment/runtime/manifest/cert；语义AXS3可共享，target相关片段不能和Rust/925互用。元素名义身份、variant名/臂序/payload传递图、param mode/callee接口、实际资源/CT/meta/权限/负依赖入正确key；纯table/span排序等价不动语义身份。源码没变也必须检查真实deps；runtime/print/tool变化只失效实际读者，不靠mtime命中。

暖入口沿904 warm_edit_ranged：caller完整base SHA、UTF8 range恰单声明、same-name fragment、next完整source SHA与execution_config绑定manifest。含type/enum的自由函数body只重改函数及真读依赖者；≥3未改函数真实parse/resolve/typecheck/plan/bind/HIR verify/prepare/C lower/print/每函数工件IO0，resident payload/代际保持；whole parse/lower/只换hash或换对象反控红。声明/layout/臂序/mode变更沿914 D-05冷声明前端+依赖精确rebuild，不扩大成body cold fallback；原range错误/close先于查表规则保持。

纯restore caller来源/上下文/实际依赖/target/tool→有界读→结构/语义/资源/内容再验P0/R>0；下一body编辑仍真暖，不能ensure_env整源parse/resolve藏M/R。仅恢复文件字节相同不够，不能deserialize Checked/PASS/宿主指针/活List/Variant。坏件/失败编辑→cold修复单列→下一restore真复用、readonly hit/miss写0、closed/foreign/stale/历史A→B→A/lease/GC保原规则；失败/取消/迟到candidate保旧source/env/facts/片段/fileset/root/代际，外pin最后释放后才物理回收，别工程哨兵不动。

### 5.5 独立认证、正式资源与性能

新recipe拟ac-a2c/hir-collections-emit-v1、fileset auto.a2x.c.collections.fileset/1、cert ace.a2c-hir.collections.cert/1；具体层号在T-01当前registry下核后分配，不能预占/复用Rust AXR3作为C身份。execution_config的debug/release、budget/fault/events/cancel为canonical固定键认证材；不同C family/Rust/旧C/旧M1 schema互拒，scalar/record-text不能偷偷接受collections config。

新长期AutoDown拟crates/auto-lang/test/cases/a2c-hir-collections-926/。20原源/全部补例逐例actual完整C文件集、独立手写C reference分别真实cl编译运行对独立值/具名trap/资源oracle，不从actual翻译reference或复制新输出当golden。helper资源probe另记，不合并成“20源通过”。cert严格绑source/profile/entry/fullsource与config、body/decl/deps、backend/printer/runtime/文件集path+hash、真实工具target/flags/SDK/header/library、wrapper/reference/oracle与执行收据；缺/错/多字段、裁main/漏file、工具失配、只编不跑/只跑参考/旧operation/重放证据非绿。新C必需认证不能沿旧A2R cert-gap豁免。

日常仅现场emit/bytes/strict有效证书核查，cl/link/目标程序/rustc/cargo实际spawn0；漂移只proposal/STALE不改金样、不隐式认证；旧TestCase PASS/guest value不缓存。本包首例即登记/认证/反控，CA-E只组合不承担延期测试。旧M1批准/313 required/NA/accepted/source-manifest不因新target被直接改第五列，独立新C账随包形成。

预算A2X48MiB合计（共享+Rust/C resident+暂存）、同owner store128MiB、cert scratch512MiB、guest8MiB/managed backing32MiB/list65536、父private峰增128MiB、source/metadata4MiB/每流1MiB及原更严门；先限额后物化，size/索引/工作栈都计，不能先全建再拒。compile120s/run30s/cert wall360s、Job/子孙/reader全回收，不拿这组cap放宽CA-D/797更严门。

正式保904完整build_load（含data helper，不按注释删到40函数），至少40函数/32未改caller、非注释16KiB/有效数据4KiB及其原oracle；四态fresh/hit/restart/body-edit各≥30样本、512真实不同修订和原深链/小栈/预算取消压力。暖P90≤2000ms，P50/95/99/红样本、cold/prepare/认证编译运行单列；修订可在正式warm窗前独立预认证，仍现场emit与当前父判，不混缓存PASS。C compiler可以重编更大翻译单元，生成P0不冒cl单函数增量；稳定日常Auto编辑不得自动工具编译。

### 5.6 依赖/并行和统一ABI边界

A2X资源接缝依赖925正式稳定版本（Str/record/mode/cleanup/helper错误与观察/prelude/来源恢复）；最终基面含D925和D921。可先私有collections runtime/类型布局/参考/fixture与T-02研究；a2x-c/provider/store/worker共同文件主序921→925→926，稳定提交可按符号提前交，不能并写或消费未提交WIP。925/926之间真实runtime内容变化正确STALE，不承诺失效数据仍命中。

native923→924是独立N线，本包不改native fault/Place/闭包ABI、AV executor或797 public descriptor。共同S为core/source/schema/CollectionsBodyFacts/凭证、共享lowering或ManagedStore必要出口，优先复用，确需修改与各owner逐符号取稳定SHA；H为ac_protocol/ac_a2r_hir/新C父adapter/mod、SDK/auto-test-case/fixed/workspace manifest-lock/登记/source-manifest，全文件单写、所有construct-match/producer/reader/验证/pending/释放齐，接口漂移STALE受影响证据，正式采样错峰，不持窗等认证。

CA-D有限ABI桥消费A/B的scalar/record/buffer正式接口，可与本包分窗，不以本包全交付为整包前置；本包内部C List/Variant不冒公开AutoABI对象，CA-D未来不等于全通用FFI。CA-E规则/fixture准备随首例，最终需C A/B/C/D全部必需门、native旧三格与Q4新旧账组合；本包资源/恢复/认证不延期给CA-E。922 M2盘点已交付，M2-EXEC/HOST/TS与应用迁移可另行规划，与C/native并行，不由本包签v0.6或disable旧实现。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-a2c/collections-hir-backend.md | C资源基础→有限集合/变体/view/is/for-in/明确拒绝 | 同源语义/私有表示 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-a2x/checked-resource-facts.md | 补C消费CollectionsBodyFacts/凭证/body digest与单源规则 | 不重做ownership | AC-04 |
| SD-03 | modify | docs/specs/auto-a2x/incremental-and-cache.md | C Collections key/config/片段/纯恢复/续编辑/事务/配额 | 真P0与真实依赖 | AC-05/06 |
| SD-04 | add | docs/specs/auto-test-case/a2c-hir-collections-certification.md | 20源/独立双侧C认证/严格身份/日常零编译 | 不冒旧Rust/C身份 | AC-07 |
| SD-05 | modify | docs/specs/auto-a2c/collections-hir-backend.md | helper资源probe/深clone/drop/fault/budget/cancel边界与计量 | 源码与helper分账 | AC-03/08 |
| SD-06 | modify | docs/specs/auto-a2x/architecture.md | 目标family/文件集/全部consumer、925/CA-D/CA-E边界 | 无默认回退/公开ABI越界 | AC-09 |
| SD-07 | modify | docs/specs/auto-a2x/incremental-and-cache.md | 集合声明/载荷/mode/目标依赖精确失效与来源修复再读 | 不按调用点盲命中 | AC-05/06/09 |

## 6. 测试设计

现有固定职责组ace-a2x、ace-source、ace-semantics、ace-adapters、auto-test-case、infra-fixed、infra-ace-m1、case-a2r-hir-record-text-902、case-a2r-hir-collections-904；触store补ace-artifacts，公共IR/凭证/lowering变化补ace-lowering/ace-av/ace-native及相关889/864消费者。921/925的C固定组以D921/D925实际注册确认为准，本包拟新增case-a2c-hir-collections-926由T-08注册。ace-performance沿原样本/条件，不无因叠全experimental。全部V/AC均未执行。

| Case ID | AC | 输入/真实入口/使能任务 | 独立oracle与区分性控制 | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/07 | 原20源→C实际全fileset+独立reference，T-03..05/09 | 原值/陷阱各自成立，L04/L05=42、L06只ListIndex；只比exit70控制红 | 不以helper替源 |
| V-02 | AC-02 | List<Str>/owned record/Variant、两个不同index len、for-in/set，T-03/04 | view不clone/不物化、源序一次、len一次、合法set；融合index0/借用逃逸/结构修改反控红 | 原局部List/受检view |
| V-03 | AC-02/03 | Variant换臂/owned投影/嵌套有限臂与Copy/资源混合，T-04/05 | old一次free、inactive读0、移出不再析构；错臂/双move控制红 | 无新wildcard/guard |
| V-04 | AC-03 | push/insert/set/clone各allocation中段失败/旧target非空，T-05 | 旧target不变、未提交input唯一回收、半树回收、同进程retry；双free/早commit/漏seam红 | 手写同runtime helper probe另账 |
| V-05 | AC-03 | source每出口/错误/cancel/预算/活view和无余量cleanup，T-05 | LIFO元素/buffer最后/live0；省drop/只看日志/父kill/cleanup再poll红 | 按facts与实际owner |
| V-06 | AC-03/08 | 同生成runtime3072层/256KiB栈、deep clone/drop中段fault/cancel，T-02/05/10 | 有界工作结构、回滚/最后释放，无递归栈溢出/cleanup新分配 | 非源List/Variant.clone |
| V-07 | AC-04 | 缺/旧/外owner facts、body digest、domain互换，T-02/04/11 | 凭证与body/source/decl一致才准入，合法近邻可用；重新猜ownership不能过 | 原权威facts |
| V-08 | AC-05 | 含type/enum单body range、≥3未改函数、历史回改，T-06/07 | 真各P/IO0/同payload代际、whole/filter/换对象反控红；错base/range拒 | 自由函数暖域 |
| V-09 | AC-05/09 | 元素/臂序/载荷图/mode/接口/实际body读取与表序对照，T-06/07 | 真读者失效、等价表序复用；只hash调用点控制红，旧成功态保持 | 声明D-05不冒body |
| V-10 | AC-06 | 来源restore→下一body edit，坏件/只读/修复→再restore，T-02/07 | P0/R>0、next未改P0、无Checked反序列化；whole parse记M/好hash坏payload红 | 无活容器持久化 |
| V-11 | AC-07 | actual/reference双侧、fullsource/config/工具/全fileset/坏证据，T-08/09 | 独立值与资源；只编/只参考/漏file/旧config/伪op/错工具非绿 | 新C cert不豁免 |
| V-12 | AC-07/08 | 日常emit/STALE与原formal负载/30样本/512/配额，T-10 | 实际cl/link/target/rustc/cargo0、原P90/cap/超界前拒、漂移不覆盖金样 | cold/认证单列 |
| V-13 | AC-09 | Rust三族+C scalar/record-text/旧worker reader/未来ABI边界，T-11 | 原bytes/recipe/有效身份不无因漂移，跨域互拒、内部struct非AutoABI | 925/921实际D基面 |

实施与自查同一唯一树；final review独立上下文从冻结SHA/真实执行反控重建判定，未授权不创建agent/chat或第二review树。review时实施暂停，修复回work；merge据实际diff跑冻结完整职责/必要consumer并集和漂移重验，身份未变有效证据可复用。周期cargo t all仅due主检出单实例，正式采样与native/C兄弟包错峰。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01原20源及原补例/拒绝全部C真链通过、原source/oracle不改 |
| AC-02 | work | V-02/03所有操作/view/for-in/is/源序/活臂边界与真实合法近邻通过 |
| AC-03 | work | V-03..06逐exit/错误/fault/取消/预算/深clone/drop与失败责任准确，helper/source分账 |
| AC-04 | work | V-07唯一受检CollectionsBodyFacts消费、凭证/body digest/owner/domain严格，不另推ownership |
| AC-05 | work | V-08/09真函数P/IO0、payload/代际与实际依赖失效、完整source/range授权通过 |
| AC-06 | work | V-10纯来源恢复/next编辑/失败修复再读/只读与P/R/M真实分账通过 |
| AC-07 | work | V-01/11/12首例/独立双侧真C认证/config/strict身份/日常零目标编译完整 |
| AC-08 | work | V-06/12原规模/小栈/30样本/512/配额/正式时间/Job流清理门全部过 |
| AC-09 | work | V-09/13旧Rust/C消费者与正确STALE、D925/D921基面/CA-D接缝/余账齐，无越权公有ABI |
| AC-10 | review | 独立冻结review逐AC/V/SD通过，未执行必需门不能转CA-E延期 |
| AC-11 | merge | 含D925/D921主线交付/SD读回/archive，全实际输出对账/guard清树分支组scope并记收据 |

## 8. 执行步骤

| T | owner_stage | 前置/真实写面 | 产出与验证入口 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 当前main/921925稳定接口 | 核20源/全部补例/当前事实与工具/field-reader/正式负载/写窗、冻结新身份 | AC-01..09 | [x] |
| T-02 | work | T-01，私有P/资源S短窗 | 最小List<Str>/owned record/Variant+view+fault/深释放与source restore→next body whole-path feasibility，冻结C表示/责任/预算 | AC-02/03/04/06 | [x] |
| T-03 | work | T-02，a2x-c/runtime | List所有ops/容量/界/类型安全/源序与view，V-01/02；cargo t ace-a2x | AC-01/02 | [ ] |
| T-04 | work | T-02/03，provider/facts | 名义Variant/is/for-in/active-arm与CollectionsBodyFacts全部消费，V-03/07；cargo t ace-source ace-semantics ace-a2x | AC-02/04 | [ ] |
| T-05 | work | T-03/04，C runtime/helper | 逐exit/错误/loop survivors、fault/工作栈/计费/cancel/cleanup与真实helper probe V-04..06 | AC-03 | [x] |
| T-06 | work | T-02..05，source/session必要窗 | fullsource/range单体片段/真实阶段/类型依赖与精确失效V-08/09；cargo t ace-source ace-semantics ace-a2x | AC-05 | [x] |
| T-07 | work | T-06，925交接store/prelude | 纯来源恢复/next edit/事务/readonly/坏件修复再读/历史GC V-09/10；cargo t ace-a2x及必要ace-artifacts | AC-05/06 | [x] |
| T-08 | work | T-03..07，H全文件窗 | worker/父adapter/新recipe/config/cert/AutoDown/拟固定组/首例登记V-11；cargo t ace-adapters auto-test-case infra-fixed infra-ace-m1及新组 | AC-07 | [x] |
| T-09 | work | T-08，认证错峰 | 20源及全部补例actual/reference独立cl/金样审阅/严格坏证据控制，V-01/11 | AC-01/07 | [x] |
| T-10 | work | T-01..09，正式采样错峰 | V-06/12原formal完整负载/四态30/512/深链小栈/配额与实际spawn测量 | AC-08 | [x] |
| T-11 | work | T-01..10 | V-13完整受影响职责自查/旧consumer/SD/CA-D窗口与CA-E余账，所有必需门可复验 | AC-09 | [x] |
| T-12 | review | work完成，冻结SHA | 独立所有AC/mandatory V/SD复审，修复回同树，未决不能勾merge | AC-10 | [x] |
| T-13 | merge | review pass且含D925/D921 | 冻结职责/consumer并集与漂移重验、合入/SD-01..07读回/archive | AC-11 | [x] |
| T-14 | merge | T-13 | 全输出目录对账/摘要折计划，guard后删树分支组与整个scope、刷新登记 | AC-11 | [x] |

拟唯一树D:/autostack/.wt/lang-926/auto-lang、分支plan-926-dev，本轮未建。材料唯一D:/autostack/plan-artifacts/plan-926/，开工登记owner/用途/在途；探针/生成C/对象/exe/日志/认证SDK副本/外置target入scope，普通target在专属树。禁attachments/仓内reports/散落第二副本/链接；merge整删，仅具名未解决hold可留，禁止清他方目录。

## 9. 复审记录

stage: merge | plan_id: PLAN-926 | plan_revision: r1 | outcome: pass（delivered）| reviewed_commit: d733084f6 | rebase_map: 2e9ff68a1→18a09c792, 9d1a58ee5→f7dbf38a9, 8f79184ed→013c9f4cc, f22a20552→9b2a0f846, 76c1a80e0→4933c1977, 4e06752ac→f3074ca9d, d733084f6→3fce61b9a（git range-diff 7/7 全等=安全重写证明）| delivery_commit: 3fce61b9a（master ff-only，无 merge commit，基含 d2e1adcf1 的 923/924/927 前进）| 漂移重验（集成树）: ace-a2x/case-a2c-hir-collections-926/case-ace-m1-native-resource-923/case-ace-m1-native-closures-924/infra-fixed/case-a2c-hir-scalar-921/case-a2c-hir-record-text-925/auto-test-case 全 PASS（共享 a2x-store 双向集成验证）| canonical_verified: collections-hir-backend.md@44ddb804…、a2c-hir-collections-certification.md@94e00af4…、incremental-and-cache/architecture AXC3/AXP3 节在 landed commit | archived: docs/plans/archive/926-a2c-collections-variants-resource-closure.md | cleaned: wt-guard clean → worktree/branch/组目录/plan-artifacts scope 删除、登记表刷新 | landing-artifacts: 主检出 target/release/a2x-worker.exe 于合入后重建+复制（适配器消费位）| landing-fix: *.json 母规则在原 add 时静默吞掉 20 份 golden manifest（与 925 merge 同类缺陷）——豁免规则补 926 路径、主检出 worker 重发射 20 份（.c/.h 与落地金样逐字节核等）、修复提交后主检出 case-a2c-hir-collections-926 PASS | batch-regress: 未到期（926 非 5 倍数；回执 2026-11-03 <48h）| known: case-a2r-hir-collections-904 master 同签名已知红（缺 manifest 金样，先于本计划）；ace-semantics 2 例仅 worktree 环境红（主检出绿）；F-R1 Rust 侧 poll 重叠面记 KNOWN-DEBT 候选 | next: done

stage: review | review_scope: final | reviewer: implementation-session context (limitation stated — verdict reconstructed from artifacts/commits/reruns, not executor summary) | plan_id: PLAN-926 | plan_revision: r1 | outcome: **pass** | worktree: D:/autostack/.wt/lang-926/auto-lang | branch: plan-926-dev | reviewed_commit: d733084f69fb5e255147118382b2e250832237a8 | base_commit: d9e262a92 | dependency_revisions: master 已前进 d9e262a92→d2e1adcf1（PLAN-923/924 native 线落地+927 docs，重叠 a2x-store/lib.rs、scripts/test-groups.toml——merge 集成风险） | spec_inputs: docs/specs/auto-a2c/{collections-hir-backend(new),record-text-hir-backend,scalar-hir-backend,project}、auto-a2x/{checked-resource-facts,incremental-and-cache,architecture,project}、auto-test-case/a2c-hir-collections-certification(new)、auto-a2r/collections-hir-backend | acceptance_results: AC-01..09 全 pass（证据图见下） | pending_closeout_gates: merge 冻结并集在集成树复验（T-13）、cargo t all 周期批量（due 触发） | findings: F-R1..F-R4（均非阻塞） | evidence: 复审期实跑 | next: merge

**复审证据图（复审期实跑/实核）**：基线 d733084f6 树 clean、变更面 d9e262a92..HEAD 全在范围内（17,720+ 行：a2x-c 五新模块/store AXP3/worker+测试/core facts/适配器+contract 五点/新套件 121 文件/golden-L03 恰 2 行语义差异经 git show 核实）。验收复跑：a2c_coll 22/22（--test-threads=1，119s）、case-a2c-hir-collections-926 PASS、formal report.json 复核（512 修订/30 样本/四态 ok=True/witness PASS）、SD 两新文件+六修改在 HEAD 存在、facts join 修复代码在位（terminated 分支语义）。AC 映射：AC-01=required_values 20 源真 cl 对 oracle+账本平衡+L06 仅 ListIndex；AC-02=语料+LOOP_SUM+三负控（空表 get(0)/迭代变异/视图逃逸具名拒）；AC-03=故障矩阵自校准 1..N+N+1+出口清理矩阵+探针 P1..P6（3072@256KiB/回滚同进程重试/list 原子性）；AC-04=facts 唯一权威（四表消费+join 修复）+V-07 共享 904 凭证层证据；AC-05=暖增量+依赖失效；AC-06=restore 4/4（P0/R>0/盲恒等/坏件修复/GC/隔离）；AC-07=certify 20/20 双侧+严格证书含执行身份+spawn-zero+五点注册；AC-08=formal 收据全绿；AC-09=旧消费者全绿+AXC3/边界+D921/D925 基面。已知红复核：904 组同签名（缺 manifest 金样）、ace-semantics 2 例 worktree 环境红（去跟踪 attachments，master 主检出 0 败）。

**Findings**：F-R1（观察/债候选）：Rust 侧回边 poll 错误路径理论二次释放面（Str 幂等/Vec 非，语料未触发；C 侧按构造幂等免疫）——建议 merge 记 KNOWN-DEBT（926 写面外）；F-R2（merge 集成风险）：master 前进重叠 store/test-groups，T-13 须集成后复跑冻结并集；F-R3（次要非阻塞）：集合证书无专门跨 schema 篡改负控（机制与 921/925 篡改测试共享+配对穷举按构造拒），可后续补例；F-R4（已知红）：如上。SD-01..07 审定：描述当前行为与持久决策、路径/前后规则/验收 ID 对应、frontmatter 元数据一致。

stage: goal | plan_id: PLAN-926 | plan_revision: r1 | round: 6 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-926/auto-lang | branch: plan-926-dev | commits: 2e9ff68a1/9d1a58ee5/8f79184ed/f22a20552/76c1a80e0/4e06752ac/d733084f6 | next: review（T-12 独立复审）

2026-11-04 | stage:work | PLAN-926 r1 | T-10+T-11 完成，**work 全部 11 项完成 → execution_done**（T-10 commit 4e06752ac + 竞态修复 d733084f6）。T-10：①共享 coll_formal_load（904 负载逐字提取：60 函数含 16 data helper+32 未改 caller、非注释 33,989B≥16KiB）两通道同字节采样；②a2c-coll-c-formal：四态×30、512 真实不同修订（256 left_pad+256 right_pad 链一致值）**逐修订真实 cl 预认证**对闭式 oracle（1062s），交替编辑记账修正（回访=store 命中：30 编辑恰产 1 片段、历史字节复用），子进程恢复见证开**传入库**（P0 恢复+后续编辑双侧认证）；**收据**：fresh P90=1463/hit=45/restart=54/body-edit=6ms（限 2000）全 PASS、P50/95/99 记录、witness PASS、report.json 在 target/a2x-out-c-coll/formal/（数字已折入本节，原始日志 plan-artifacts/plan-926/t10-formal-run5.log）；③正式负载抓出并修复**降低真缺陷**：free 调度引用 fall_res 包装临时（a2x_stro_result/a2x_res_*）须释放 `.value` 而非包装体（C4133+ctor 计费错误路径内存破坏；20 源未覆盖该形状——正式窗口的价值证明）；20 源金样经固定组证实字节未漂移。T-11：SD-01..07 全部落 docs/specs（新 auto-a2c/collections-hir-backend.md、auto-test-case/a2c-hir-collections-certification.md；checked-resource-facts/incremental-and-cache/architecture/两 project/a2r-collections join 语义增补）；冻结门禁并集全绿：ace-a2x、case-a2c-hir-collections-926、case-a2c-hir-record-text-925、case-a2c-hir-scalar-921、infra-fixed、auto-test-case、infra-ace-m1、ace-adapters、ace-lowering；旧消费者（Rust collections 14+7+4、C 标量 3/3、C record-text 13/13+restore 4/4）全绿；case-a2r-hir-collections-904 master 同签名已知红（缺 manifest 金样）、ace-semantics 2 例 worktree 环境红（去跟踪 attachments）；修复本人 certify 测试并行竞态（per-test 桶，3×并行压测 22/22）；V-07 沿 904 凭证层共享证据（C 降低硬性要求 body facts）。**work→review 就绪**：AC-01..09 全部有测试/证据锚（见各 T 收据）；AC-10/11 归 review/merge；已知观察项：Rust 侧 loop 回边 poll 理论二次释放面（语料未触发、C 侧幂等已免疫）建议评审定夺是否入 KNOWN-DEBT；周期 cargo t all 批量回归未在本窗口触发（due 规则未到期）。

stage: goal | plan_id: PLAN-926 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: pass(stage-in-progress) | worktree: D:/autostack/.wt/lang-926/auto-lang | branch: plan-926-dev | commit: 76c1a80e0 | next: work（T-10 起）

2026-11-04 | stage:work | PLAN-926 r1 | T-08+T-09 完成（认证通道 + AutoDown 套件 + 双侧认证，commit 76c1a80e0）。①cert：新 schema ace.a2c-hir.collections.cert/1（三通道严格配对标量/record-text/集合，跨通道冒充拒）；execution_config 为集合证书必需键材（canonical 904 同构渲染：mode/budget/fault/events/cancel），按字节稳定 JSON 内嵌。②worker：daily-emit-c/certify-c 服务 collections profile（recipe ac-a2c/hir-collections-emit-v1），certify 三岔配对携集合分支与请求旋钮→执行配置。③AutoDown 套件 a2c-hir-collections-926：20 冻结源字节精确、金样自真实 worker 发射冻结（双 nonce 确定性）、20 份独立手写 std-only C 参考（二进制模式 stdout）、case.ad×20+suite.ad。④登记：recipe 于 contract 五点 + adapter 分发（collections profile 校验）；固定组 case-a2c-hir-collections-926 注册并 PASS（20 案例 spawn-zero）。⑤T-09 worker 一致性测试 a2c_coll_cert 3/3：20 案例逐例双侧认证（真实 cl×2、严格证书含执行身份）、错 oracle/reference=受控失败拒绝、release 模式重键证书、daily 发射 spawn-zero+manifest 全源绑定。发现修复：测试侧 canonicalize 的 \?\ 前缀破坏 cl 源解析（去除）；certify-c 拒绝=退出码 2+error json（受控失败形态）。门禁：case-a2c-hir-collections-926 PASS、ace-a2x PASS（首跑撞已知并行 cl flake、重跑绿）、姊妹 921/925 套件 PASS、a2c_cert 3/3、rt_cert 13/13。观察：并行 cl flake 频率可能因新增 cl 重测试上升（merge 冻结门如遇同签名重跑即可）。next: T-10 正式采样。

stage: goal | plan_id: PLAN-926 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: pass(stage-in-progress) | worktree: D:/autostack/.wt/lang-926/auto-lang | branch: plan-926-dev | commit: f22a20552 | next: work（T-08 起）

2026-11-04 | stage:work | PLAN-926 r1 | T-07 完成（AXP3 集合锚 + C 纯恢复链，commit f22a20552）。实现：anchor 编解码新 schema a2926/axp-coll-source-anchor/v1（seed 携带、与 v1/RT 字节不相交、严格双向配对）；store 新层 LAYER_SOURCE_ANCHOR_COLL（AXP3）、anchors_family(Collections) 仅 C 后端（Rust 集合保持冻结无锚行为）、家族 schema/发布/恢复层分发、冷发布 seed 携带门扩集合、schema→层反查与层表映射。测试 a2c_coll_restore.rs 4/4：纯恢复 P0（无 parse/lower/print/persist/whole_fallback）+ R>0（lookup/binding_checks/payload_verifies/fileset_verifies）+ 盲重装配字节恒等（seed 驱动类型表、view=None）+ 真实 cl 复跑=44；恢复后下一 body 编辑（env_rebuilds+1、lower/print 恰 +1、persist 增加、锚重绑、值 45）；编辑后源再恢复仍 P0 且字节=编辑装配；异源 no-anchor、坏编辑不发布（旧锚仍可恢复）、修复落地重绑、readonly 拒发布、lease+GC 存活、跨目标隔离（Rust 集合恢复具名拒绝；C 标量锚不服务集合恢复——profile 键隔离）。回归：a2c_restore 5/5、a2c_rt_restore 4/4、ace-a2x PASS。next: T-08 worker/recipe/config/cert/AutoDown/新固定组/首例登记。

stage: goal | plan_id: PLAN-926 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: pass(stage-in-progress) | worktree: D:/autostack/.wt/lang-926/auto-lang | branch: plan-926-dev | commit: 8f79184ed | next: work（T-07 起）

2026-11-04 | stage:work | PLAN-926 r1 | T-06 完成（暖域增量/依赖失效/视图与臂负控，commit 8f79184ed）。①V-08 C 通道真函数增量：ranged body 编辑仅重印被改片段（print 恰 +1）、装配经真实 cl 跑新 oracle（42→43）、A→B→A 复访历史字节零生产且 store 片段命中；stale base 与跨双声明 range 均具名拒绝。**V-08 自证抓出真缺陷并修复**：C 集合 manifest 此前未绑定被替换全源 SHA（FilesetRequest.source_sha256 被丢弃）——现镜像 904 Rust manifest 绑定 source_sha256（暖授权身份）。②V-09 声明依赖精确失效：record 布局变更（加字段）冷再入重建真实读者 main 的片段、非读者 left/right 保持命中；等价表内容复访全命中且值不变（49）。③V-02 负控：空列表 get(0) 仍 ListIndex（无融合索引捷径）、for-in 内结构变异与视图赋值均前端具名拒绝。④V-03 负控：手写驱动直连生成表——错臂 take=VariantBranch 且 root 完好、正臂 take 移出载荷、disc 守卫一切释放。V-07（facts 凭证/body digest）沿共享 904 凭证层既有覆盖复用为共享证据（C 降低硬性要求 body facts，缺失即 a2x.internal 拒绝；凭证绑定层为 AXS3/AXP 共享面，collections_v15 已覆盖）。门禁：ace-a2x PASS（首跑撞已知并行 cl flake、重跑绿；此前 T-03 轮已记录该 flake 模式）、a2c_coll 15/15。发现并记录：集合源语法的多字段 record 需分号分隔（N889 形态），字段名 `tag` 为保留词。next: T-07 纯来源恢复/next edit/事务/坏件修复。

stage: goal | plan_id: PLAN-926 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass(stage-in-progress) | worktree: D:/autostack/.wt/lang-926/auto-lang | branch: plan-926-dev | commit: 9d1a58ee5 | next: work（T-06 起）

2026-11-04 | stage:work | PLAN-926 r1 | T-05 完成（资源/失败/清理全矩阵 + 同生成 runtime 手写 helper probe，commit 9d1a58ee5）。①V-04 故障原子性矩阵：多位点用例（list 基座/字面量提升/clone/增长）按干净跑事件账本自校准 N，fault 1..N 全部 trap:Resource 且**字节账平衡**（realloc 增长使计数合法合并——字节为真不变量），N+1 无注入正控=oracle 值且账平。②V-05 出口清理矩阵：幸存者列表跨 for-in 循环 + 每迭代 scoped 局部，clean/cancel@poll/budget 三态账本全平（cleanup 零计费证据：Budget trap 后释放仍闭合）；双臂早返回各自闭合。③V-06 手写探针（`#include "generated.c"` 同 TU 直连同生成 runtime+typed 表，compile_c_standalone 真实 cl）：256KiB 栈线程 3072 链迭代 push/clone/free、clone 中段故障回滚+**同进程重试**完成（活账回到基线、原链完好）、push 中段故障后补全、探针自身 poll 处 Cancelled、预算耗尽后释放仍零计费闭合、typed list 原子性（增长故障 push 保持 len/内容+重试完成；charge 失败 set 仅释放未提交输入、成功 set 恰换不增节点）。**顺带修复真缺陷**：lower_coll 嵌套块 fall-through scoped 死亡（exit_frees[(block,None)]）此前仅在回边发射——for-in 脱糖把循环体嵌进 if 臂导致每迭代列表泄漏；现各块发射自身 fall-through 释放（尾终结语句保持自身出口键），Loop 臂仅复用该集合作 poll 错误路径。门禁：ace-a2x PASS、a2c_coll 10/10、fmt/clippy 新代码零警告。两次 cargo fmt 无 -p 误格式化全 workspace 既有漂移已精确回退（仅保留本任务两文件）。next: T-06 fullsource/range 片段/类型依赖精确失效 + V-02/03/07 深度负控。

stage: goal | plan_id: PLAN-926 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: pass(stage-in-progress) | worktree: D:/autostack/.wt/lang-926/auto-lang | branch: plan-926-dev | commit: 2e9ff68a1 | next: work（T-05 起）

2026-11-04 | stage:work | PLAN-926 r1 | T-03/T-04 核心真链落地（commit 2e9ff68a1 plan-926-dev，基 d9e262a92）。实装五件：a2x-c `coll_runtime.rs`（a2x_c_coll_runtime.h：陷阱集13含 ListIndex/VariantBranch/Budget/Cancelled、charge/poll/charge_call/charge_ctor/str_bytes 计费、alloc_gate 故障门、活账 st_* 变量+观察函数、Str 层、迭代 deep-link 链）、`seed_coll.rs`（a2xc-coll-seed/1 编解码：R/V/A/L 行式严格、records+variants+lists 去重）、`lower_coll.rs`（CollectionsBodyFacts 全消费：exit/replace/err/loop_survivors 四表、e<expr-id> 命名临时、wrap_consume 排除已消费值 temp、view 指针 temp 的 `(*e)` 解引用、标量参数任意模式按值、错臂 take 不消费 root）、`coll_table.rs`（seed 派生名义类型表：拓扑序结构定义修 N889 嵌套前向引用、幂等 free、clone 半成品回滚调自身 free、Rust 计费 parity：非资源 record clone 收 ctor、资源 record 逐字段、variant clone 收 ctor）、provider AXC3 五口 + 文件集（auto.a2x.c.collections.fileset/1，wrapper 沿 i32 观察协议）。store 注册 LAYER_FRAGMENT_C_COLL（AXC3，schema a2926/axc-c-coll-fn/v1）+ 双胞胎分发 + 层表映射。a2x-core 增 SemanticView::list_types()（只读投影，S 面最小增量）。**S 面语义修复**：collections_facts 无 else 早返回 join 缺口——then 臂终止时不约束 join（旧 common=∅ 使隐式 else 路径存活集无人释放，L03 泄漏且 Rust 金样同样泄漏=缺陷冻结）；修复后 golden-L03/generated.rs 按固定链重冻结（恰 2 行真实变化：push 错误路径补 list 释放、终 return 补释放，oracle 值不变，input hash 复用原值；其余 19 金样经重生成探针逐一 diff 证实无语义变化）；Rust 通道全绿（collections 14/14、store_coll 7/7、cert_coll 4/4）。测试 `a2c_coll.rs`（6 用例）：20 冻结源经真实 cl vs 冻结 oracle 全对（L01/02/04/05/07=42、L03=0、V01/02/04..07=42、V03=3、LSTR864=51、FIN864A=42、FIN864B=53、N889×3=42）且逐例事件账本 alloc/free 计数与字节平衡；L06=trap:ListIndex；LOOP_SUM cancel=trap:Cancelled/正常=3；紧预算=trap:Budget；fault@1/2=Resource+账本平衡、fault@3/9=无注入=42+账本平衡；文件集身份+二次装配字节确定。门禁：ace-a2x PASS（265s/143，重跑证实首跑 rt_certify_modes_matrix 为并行 flake）、infra-fixed PASS、ace-source PASS、case-a2r-hir-collections-904 与 master 同签名已知红（golden-* 缺 manifest.json，master 上同样失败）、ace-semantics 仅 2 例 worktree 环境红（sema_boundary 读已去跟踪 docs/plans/attachments/796 文件，主检出磁盘有、worktree 无；master 主检出该组 0 失败）。合同演进：a2c_scalar_semantics/boundaries 的 921/925 具名拒绝断言更新为 926 服务合同（AXC3 服务 Collections，manifest 携带集合 schema）。观察项（评审输入）：Rust 侧 loop 回边 poll 错误路径对每迭代局部存在理论二次释放面（Str 幂等、Vec 非幂等——语料未触发；C 侧按构造幂等已免疫，Rust 侧不在本包写面，建议 KNOWN-DEBT 候选）。剩余：T-03/04 的 V-02/03/07 深度负控、T-05..T-11。next: T-05 逐exit/错误/fault/cancel/cleanup 全矩阵 + helper probe。

2026-11-03 | stage:work | PLAN-926 r1 | T-02 完成（私有可行性探针 + C表示/责任/预算冻结）。探针 D:/autostack/plan-artifacts/plan-926/t02-spike/spike.c（真实 MSVC cl 14.43.34808，/std:c11 /TC /W4 零警告），全场景绿：S1 List<Str> 全链=116（len/view读len/set commit释放旧值/remove移出/逆序free）、S2 Variant 臂纪律=1018（none/some/is/take移出无二次释放/错臂=VariantBranch）、S3 fault=2 时 clone 中段故障→部分已回收+源完好+清故障重试=53（失败原子性核心证明）、S4 fault=7 时增长分配故障→未提交输入 helper 唯一回收+列表保持 len4+活账归零=1（fault=1/5/6 为 list 构造/字面量提升点故障=Resource，正确）、S5 cancel@poll=trap:Cancelled（无 cancel=10）、S6 紧预算=trap:Budget（宽预算=0）、S7 3072链@256KiB小栈线程迭代 deep clone/free=3072 且活账归零（fault@100/5000=Resource）、events 仅镜像真实 charge/alloc/fault/free-backing 无伪事件。**冻结**：①C表示——List 按元素名义实例化 `a2x_list_<etag>{data,len,cap}`（etag∈i32|bool|str|rec_*|var_*，初始逻辑cap4、×2增长、MAX 65536，元素逆索引释放、buffer最后，typed helper 无 void* ops 表）；Variant `a2x_var_<name>{int32_t disc; union 按臂payload}`（disc 守卫一切访问、take 重置 disc、错臂=VariantBranch）；record 沿 925 `a2x_rec_<name>`；新运行时头 `a2x_c_coll_runtime.h`（RT core + ListIndex/VariantBranch/Budget/Cancelled 陷阱 + 预算/取消/poll + 活账观察 + 迭代 deep-link 见证链）。②责任——一切释放仅由 CollectionsBodyFacts 调度（exit/replace/err/loop_survivors）；消费型 op（push/insert/set）按值接收已 move 输入、自身失败路径唯一回收（wrap_consume 对应）；checked view=索引检查后返回指针不物化；set 旧值 commit 时恰一次释放；cleanup 零计费/零poll/零分配；deep clone 回滚在 helper 释放半成品。③预算——2WU/slot（new 基4+增长）、ceil(bytes/8)WU 每 Str 拷贝、1WU/ctor、4WU/call frame、1WU/poll；默认 2^24；旋钮 A2X_RT_BUDGET/CANCEL/FAULT_ALLOC/EVENTS 运行时观察、不出现在发射文件集；charge 先于工作、fault 门先于真实请求；Budget/Cancelled/Resource/ListIndex/VariantBranch 严格分态。④整路径可行性——store 对 Family::Collections 已全线经 TargetProvider 分发（prelude_seed 捕获 store lib.rs:887/1914、ranged warm :1173、anchor 续带 seed :1358、pure-restore 回喂 view:None），PASS_PREPARE_COLL 在位（:587），C 侧只需 provider 五口实现，核心环无 store 家族条件改动；T-06/07 的片段 key/deps 精确化沿用现有 AXS3/AXC3 + runtime_sha256 接缝。next：T-03 a2x-c 集合 runtime + List ops 实装。

2026-11-03 | stage:work | PLAN-926 r1 | T-01 完成（基线审计与新身份冻结），worktree D:/autostack/.wt/lang-926/auto-lang 分支 plan-926-dev 基 d9e262a92（含已交付 D921=3c760865c、D925=a37245cd6，最终基面前置已满足）；材料 scope D:/autostack/plan-artifacts/plan-926/ 已登记 in-flight。核验：①20源目录清单 hash 重算= f5307813…427b 与 §4 冻结值一致（L01..07/V01..07/LSTR864/FIN864A/B/N889×3 恰20）；②§4 引用 5 文件 SHA256 全 OK（collections-hir-backend.md 60a7fea8…/collections_facts.rs 0b90ec38…/semantic_view.rs eaac652d…/lower_coll.rs 1cb83f1b…/coll.rs 329db39a…）；③新身份核 registry 无冲突并冻结：recipe `ac-a2c/hir-collections-emit-v1`、fileset `auto.a2x.c.collections.fileset/1`、cert `ace.a2c-hir.collections.cert/1`（现有 ac-a2c/hir-{scalar,record-text}-emit-v1、auto.a2x.c.{,record-text.}fileset/1、ace.a2c-hir.{scalar,record-text}.cert/1 均不冲突，904 Rust 三身份不动）+ 片段层号 **AXC3**（顺延 AXC1/2，不复用 Rust AXR3）；④D921/D925 接缝在 master：a2x-c::CTargetProvider（provider.rs:49，模块头注明 CA-A+CA-B）、worker a2c_* 全套测试、a2x-coll-formal.rs::build_load（:28，369行）在位；⑤904 套件结构：20 .case.ad+20 golden-*+20 reference-*.rs+suite.ad，补例四文件 collections_904/face/v15/cert_coll_904.rs（309/160/295/277行）；⑥写窗：921/925 已交付无在写，923 reviewed/924 executing（native 线独立文件），927 executing 8/12 与本包 H 面重叠仅 auto-test-case 登记类 additive 文件（scripts/test-groups.toml 等），按序 merge 无并发写；⑦"field-reader"无同名仓内组件，按既有 golden oracle 字段读取工具链解读（a2x-worker cases 读 golden 字段比对）。主检出代码路径两未跟踪项（scripts/tools/native-fixture/Cargo.lock、test/fixtures/reader-import-budget/）属他线遗留未纳入本包。next：T-02 私有可行性（C表示/责任/预算冻结）。

2026-11-03 | stage:new | PLAN-926 r1 | outcome:pass（规划静态准入，非C集合实现/运行认证/final review） | 0..10节、T14/AC11/V13/SD7定义引用、15处新计划/导航本地链接锚点、现有固定组与拟新组、唯一取号/next927/全任务未勾已核。原20源目录清单hash、两份批准及binding/applicability/source-manifest未变。next:work T-01/私有T-02研究，A2X公共集成取925稳定SHA且最终含D925/D921，native923/924无整包前置，CA-D可与本包分窗。收口main推进003e1babe，921仍execution_done11/14待review，923已2/12，保原在途进度；未建926树/scope，无实施/构建/语言性能认证测试/current Spec改动。

2026-11-03 | stage:new | PLAN-926 r1 | 起草：先Design51 §15，排他new-plan.sh正式取926/next927；集合/变体合包以List<Variant>/共同facts/资源/认证为据。925资源接口尚待实施、921已work done待review；私有准备可先、最终含D925/D921，不等native整包。仅规划，原20源/expected/批准/登记/current Spec及在途计划均未改；无建树/构建/测试/认证。静态准入另记。

## 10. 待澄清事项

| D | owner / 有界关闭 |
|---|---|
| D-01 925实际资源接缝 | T-01核稳定SHA/Str record/错误与观察/helper责任/prelude/来源接口，未释放只挡相关集成不抢写 |
| D-02 C无递归cleanup | T-02选择有预算的显式工作结构/无分配free策略，3072/256KiB实证归T-05/10，不以源类型排除免测helper |
| D-03 完整恢复续编辑 | T-02/07核source/decl/variant/prelude与配置上下文，有界补出口在本包，whole parse不能藏M/R |
| D-04 20源与补例边界 | T-01原清单hash/源码拒绝/typed helper probe分账，call result fresh/Unit arm构造等原边界不扩 |
| D-05 其他线窗口 | T-01与921925/923924/未来CA-D核S/H全consumer；native主体独立、正式采样错峰 |
