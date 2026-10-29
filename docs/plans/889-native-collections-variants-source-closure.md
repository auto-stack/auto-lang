---
plan_id: PLAN-889
status: executing
feature_name: native集合与变体源码闭环（共享MIR、拥有值与真实函数增量）
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-30
plan_revision: 2
supersedes_spec_components:
  - docs/specs/auto-mir/native-data-owned-lowering.md
  - docs/specs/auto-ac/native-backend-and-retirement.md
  - docs/specs/auto-test-case/ace-m1-conformance.md
  - docs/specs/auto-av/resident-function-install.md
  - docs/specs/auto-ac/source-collections-variants.md
new_spec_components:
  - docs/specs/auto-mir/native-collections-variants.md
touched_goals: [GOAL-002]
affects: [auto-ac, auto-mir, auto-av, auto-test-case]
depends_on: [PLAN-864, PLAN-871, PLAN-881, PLAN-822, PLAN-828]
current_step: 1
total_steps: 10
---

# [PLAN-889] native集合与变体源码闭环

## 0. 变更摘要

将864已有的集合/变体合法源码，经同一Checked HIR→共享collections MIR→mn CLIF/COFF→真实PE执行；关闭M1原L01–L07/V01–V07共14个native源码格。维持零参i32入口/结果，继承822集合与资源语义；同时补集合源码的真实单函数增量，而非整模块重算后报告carry。

[拆包决定](reports/ace-native-phase2-split-decision-2026-10-29.md)：native集合/变体与带位宽typed entry拆为两个计划，先本包；后继typed entry现已另立891（仅规划，不构成本包D门，也不授权本包启动它）。List和Variant共享布局/资源闭包，合法List<Variant>必须本包支持，故不再拆成两半。r2据T-01反例调整T-02顺序：先修规范已承诺的命名enum类型查找，再做N全链资源probe；区分源码clone拒绝与typed容器clone验收，不缩减G/AC。当前只有T-01基线证据，native执行/资源验收未完成。

## 1. 目标

- G1：864原合法源码面在AV/native分别对独立oracle通过；原L04/L05必须42、L06仍list-index负例，不以typed fixture/改源/unsupported替代正例。
- G2：拥有List元素/Variant载荷的转移、借用读取、显式clone、switch、正常及失败清理正确，无隐式深拷贝、失效view、双free或错误提交；物理内存/逻辑预算均受检。
- G3：沿871唯一路径，native运行时只实现机器布局/容器原语，不执行MIR/ABC、调用AV或恢复旧HIR→CLIF。正式入口与cache/COFF恢复确实执行native产物。
- G4：source body编辑只重做改变函数的前端/共享lower/MIR/代码生产；未改函数保producer/code身份；依赖/接口改变正确失效，候选失败保旧，恢复重验。

非目标：W01–W05宽位型/typed entry、宽数值算术、native闭包/Dyn/loader/HMR、完整typed Place API、List参数/返回或容器record字段、嵌套List、用户泛型/trait/HOF库、Unit入口、产品默认引擎/旧AutoVM/A2X退役、全M1验收。不修改AV profile4的opcode/wire/执行语义，不改变864约定的源码支持/拒绝集合；T-01发现的List<命名enum>误拒须修到既有约定，不以当前缺陷取代规范。容器本体List.clone/Variant.clone源码方法仍拒绝，V-03容器clone走受检typed MIR/runtime；不能由Spec含混的“owned局部.clone”表述推导新源码方法授权。

## 2. 架构方案

source→既有compile_collections_source/CheckedModule→auto-mir-lower::{lower_checked_collections,lower_function_collections_unit}→verify_mir→单函数unit→mn容器表示/CLIF→COFF→多对象链接/collection runtime→PE；AV独立走现行ABC验证执行链作语义对照，各自先对手算oracle。

**计划支持集是864完整源码域与其实际产生的MIR指令闭包，不是整个822 typed profile的任意机器接口。**参数/返回中合法Variant、Str与record、List<Variant>、非递归record及无环嵌套Variant须成立。源域已拒List参数/返回/字段；直接typed Place/List参数、宽数值等本包未承诺输入触达时在codegen前具名拒绝，不能NOP。允许profile4不代表无条件放行所有指令/签名。

表示选择为T-02决策：倾向类型导向原生布局/描述符及有界runtime helper（含必要frame cleanup登记），CLIF直接调用小型原语；可等价采用生成的类型专用helper，不固定未验证实现。容器/文本owner责任、临时共享、无环图清理、跨函数结果不得退化为通用解释器。runtime库有独立合同/内容hash和依赖身份，属于mn私有ABI，非797公开AutoABI。

## 3. 技术栈与写面

| 范围 | 已核真实路径/符号与拟新增项 |
|---|---|
| N准入/机器布局 | experimental/ac-mir-native/src/{emit,lower,layout,driver,symbols,config}.rs；admit_unit/cranelift_sig_of/SlotRep/Domain/Arm/arm_of_domain |
| N单元/cache/发布 | experimental/ac-mir-native/src/{incremental,cache,link_driver,resources,observe}.rs；COFF/meta/LinkManifest/runtime哈希、reuse/fresh/read-only与原子发布 |
| N独立runtime（拟新增） | experimental/ac-mir-native/runtime/collections/独立静态库及src/collections_runtime.rs定位/版本门；可用等价类型专用helper，T-02冻结目录/导出面。现有experimental/ac-core/runtime/text/shim只证明文本buffer，不能冒称已有容器runtime |
| S支持域前置与真实函数增量 | experimental/ac-core/src/source/{parser,resolve,typecheck,adapter}.rs及其中body工作观察接口（非独立body_work.rs）；resolve::{vt_of_ann_coll,classify_generic_recv,CollTables}的命名enum查找/上下文限制；experimental/auto-mir-lower/src/hir_adapter.rs；experimental/ac-av-probe/probe/src/{collections_864_session,collections_session,fn_hir_codec,cache}.rs（按实际最小写集） |
| H正式消费 | crates/auto-lang-core/src/test_cases/ac_mn.rs及必要ac_av身份；crates/auto-test-case/src/contract.rs现有recipe/解码缝；crates/auto-lang/src/tests/ace_m1_tests.rs；ace-m1的binding/新case与公共worker注册 |
| 拟新增验证 | ac-mir-native/tests/{collections_source_tier,collections_resources_tier,collections_incremental_tier}.rs；scripts/verify-native-collections-889.ps1；ace-m1/native-collections-889.{suite,case}.ad（当前不存在） |

权威实施树D:/autostack/.wt/lang-889/auto-lang，分支plan-889-dev，base=e0b29ff7a，当前实现提交69d7b6b26；一Plan一权威树，实施/独立review/修复均同树顺序执行。根workspace不新增实验栈依赖，auto-av-core零依赖边界不变。

## 4. 需求分析与背景调查

2026-10-30 执行授权：用户 /goal 完成计划889 授权本合同 work→独立review→有界修复→merge/规范/归档/guard清理全链。沿用当前active goal，不重复创建。保留他方文档/号源WIP；本次只提交889合同，.next-id当前891包含888/889/890累计取号，不夹带提交或回退。


历史规划授权：用户2026-10-29要求“规划下一个计划，先判断一个还是两个”，当日仅规划；2026-10-30执行授权已如上记录并继承到本次有界r2修订。按AGENTS在master中央取号889；保留其他会话未提交文档与号源WIP，不夹带提交。不自动创建agent或联系其他聊天。

已读AGENTS.md、Spec总览及auto-mir的native-data-owned-lowering/collection-place-profile、auto-av/collections-variants、auto-ac/source-collections-variants/native-backend-and-retirement，Design43/44/49既有架构（Design49问题表的旧双lowering快照已经被871终态取代）。读取master=5af8e0cbe，D871=87dd2b6ad及5af8e0cbe清理收据已入master；D864=6273cd4a4、D881=db0773130均具交付。输入hash与拆分依据见拆包报告。

| 静态事实 | 对本包的影响 |
|---|---|
| emit仅准入core/data/owned；List/Variant/Place槽与集合指令全部拒绝 | 不可只改profile枚举；布局、调用、runtime和清理是实际新实施 |
| driver.compile_escalating与arm_of_domain不选Collections | N正式编译/CLI/worker必须显式新增Collections通道，保持旧五profile选择/诊断，不因宽松fallback吞真实错误 |
| wrapper/driver入口仅零参i32-family | 本包原样保留；typed位型另包，两个包无需相互语义等待 |
| collections_864_session::edit总是compile_units；其内整源编译+whole lower+split | 原Spec的下层carry不等全链增量；必须补前端保留环境/单体生产及原子候选，不可直接套scalar MN会话后自称完成 |
| split_units已有容器类型闭包修复；MN_INCREMENTAL仍绑scalar FunctionIncrementalSession | 复用受检单元/缓存设施，扩实际消费者，不能新造第二份语义；接口/模式/类型依赖与恢复覆盖必须实测 |
| 871已删除旧native | 本包只做AV/native两腿+独立oracle；历史旧腿不重建，不用同一mn入口当“三路独立对拍” |
| T-01实跑List<Token>（拥有Item载荷）在vt_of_ann_coll误走record-only查找，尚未进入shared MIR | 在T-02接受已释放的S最小写面，修命名enum局部/载荷/返回及List元素查找，保上下文排除；先AV对独立oracle，再N同源。既有17行与源码哈希不动 |
| T-01源码List.clone显式unsupported、Variant.clone无方法；typed profile已有Clone | V-03原有source/typed分账落到具体入口：合法Str.clone走source；List/Variant本体clone走未信任typed builder→verify_mir→mn与runtime，source拒绝作兼容负控。原T-01两个“源码clone成功”假设作废，不删除容器clone资源义务 |

r2输入身份：canonical `docs/specs/auto-ac/source-collections-variants.md` SHA256 `a82dafa63e2a2426c4d4c2716a1bca64c4adee9385207b7fc9c6fdeb080cfdcd`；权威树 `experimental/ac-core/src/source/resolve.rs` SHA256 `4c7f8ca56f116a7076dbebdb00a1f9a2e690123f81fa7b04b5fb93ad863b25ae`。T-01的可重放源码、现行路径、基面反例与raw hash见[执行入口r1](attachments/889/execution-entry-r1.md)；这些是修订依据，不是native通过证据。

## 5. 详细设计

### 5.1 分阶段交接与并行

- D864/D871/D881/D822/D828是语义/产物前置；**D865/D863不是本包语义依赖**。T-01已实核前置交付祖先。T-02先S支持域前置再N私有资源probe；T-03/04在T-02实际probe通过后实施，不能跳过其所有权/资源证据。
- 865交付5fa3133fa、863交付2d5f3d842并已移除其语言树，旧“865→889→863”规划插窗已过时。T-02可在重新核S owner后接受`resolve.rs::vt_of_ann_coll/CollTables/相关调用方`与新定向测试最小子集，修规范支持域的前置误拒；先真实提交/记录释放，再进入N全链。T-05另核parser/adapter/共享lower/session所需S子集，不凭T-02接受记录占有整个S。不消费他方WIP，不复制旧前端/运行时；若触及text shim须另登记该文件/符号。
- T-06公共H仍须与888逐文件/符号交接；T-01核888已提交diff涉及ac_av父侧、worker/helper/driver/lib及发现/执行器，不涉及本包N私有或ac-core/source。865/863/887已交付不等于888已释放；新891仅规划且N默认889→891，无权覆盖889。不得依据append-only或不同worktree推断无冲突。本Plan不改runner/filter/选择守卫，不要求U888作整包前置。
- 交接记录至少含前owner/Plan修订、具体释放文件/符号、delivery、Spec/profile/schema/API/lowering/ABI/runtime/cache身份、已交付消费者兼容/限制、后owner接受。跨S/N/H任务同时拿所需窗；阶段完成必须真实提交/释放，不能一个窗口内等另一个。若后继typed entry取N，刷新本包阶段/正式交付版本后单写，不能并改emit/config/cache。

### 5.2 支持与准入合同

- 逐项沿864：List<int/bool/Str/record/variant>，record字段int/bool/Str/record且无环、无容器字段；Option/Result/有限enum带copyable或owned载荷（V01/02已有i32载荷，不因822概述漏列而拒绝）；Variant可无环嵌套，无擅增深度上限。List函数局部、不嵌List，source str/char/Unit值与方法/闭包等沿原拒绝域。
- 承诺操作=864产生的ListNew/Push/Insert/Get/Set/Remove/Len、ListFieldGet及其FieldGet/View/BorrowEnd组合，clear尾删循环、VariantPack/Guard/Project/Switch、聚合/Move/Clone/Drop、普通调用/控制流和既有Str.len；T-01/02从正式源码/共享lowering枚举实际指令闭包。ListCap/Disc/FieldSet等实际需要者可同门实现，不宣传未验typed面。
- 命名enum在局部/结果/容器载荷/List元素位置与同域Option/Result一致保有名义身份，不能fallback为I32；未知名字、错误元素型、重复/递归声明须具名拒绝。修复不得顺带放行Variant参数/record字段等当前源码上下文排除；本合同“参数/返回中合法Variant”以该源码域合法位置为限，typed签名的机器承载和拒绝另行准入。T-02以嵌套无环Variant合法控制与自环/间接环反控检查共享查找分支，保持无新增深度上限；声明顺序相关机制在T-02按同一支持域修复，不靠全表默认I32填洞。
- 显式clone按输入起点分账：合法源码Str.clone（及确实通过原前端的owned局部）走同源AV/native；List/Variant本体clone走V-03受检typed资源probe，覆盖独立副本/原根存活/深度/部分失败，不能只调用runtime绕过verify/codegen。源码List.clone/Variant.clone继续具名拒绝并留兼容控制，不声称本包新增这些方法。
- 先共享verify_mir和单位/布局准入再codegen；owner、元素/臂型、guard/payload-moved、嵌套/循环/坏引用不能因本包绕过。允许的源码拒绝仍在原阶段具名，后端未支持的typed输入codegen=0。
- CLI/SDK优先新增显式collections profile/recipe选择，不把真实type/verify错误当升级信号。自动选择如果保留，只对原Text阶段source.unsupported继续；旧core/scalar/text/record/text-record身份/支持集及retired:legacy拒绝不变。

### 5.3 运行时、转移和资源

T-02冻结：List控制块/元素stride/len-cap、Variant判别/活动载荷、名义type描述符、内部call/return ABI、context/allocator/frame清理、状态码与计量。所有地址/容量运算checked，验证通过不免除runtime的边界/生命周期防御；helper只接受经生成代码绑定的受控描述符，不把任意输入pointer当安全数据。

List get对Copy值拷出，owned读取沿864受检投影，无隐式clone；remove转移owner，push/insert/set接受新构或显式move，失败不抽干输入/原容器。owned字段.len展开中的remove/view/insert须保元素与len、零额外资源事件；不能“移出后drop再重建”。Variant投影只读活动臂，owned payload转移且不双用，switch先备完整新载荷再提交、旧活载荷恰释放一次，guard失效。显式clone独立资源，深构造/clone/drop/部分回滚均显式工作栈，不以宿主递归处理任意无环深度。

正常跨函数返回/变体载荷存活到caller最后owner；本集合profile的当前帧及有资源caller帧在trap/资源失败出口也需按冻结责任清理，不能沿用已知mn“只清当前帧”作为本包免责。T-02先以双帧活List/Variant探针决定统一cleanup context/生成清理路径，失败必须调整方案；不要求本包泛化修复所有其他profile历史语义。

**预算分别计量，不把不同分区上限混为一数**：guest值数据≤8MiB、执行受管≤32MiB、MAX_LIST_ELEMS=65536、worker增量≤128MiB沿822；N编译侧原48MiB（MirSymbol16/Staging24/IO8）、LINK/OBJECT/stream/deadline沿config/resources。适用同一对象的合同取交集，不擅放宽；container元数据、描述符/工作栈/台账也纳入物理内存，不只报逻辑估算。T-02记录各预算观测边界及边界控制。

runtime逻辑事件与物理alloc bytes分列：native布局不要求与AV Vec的物理字节逐项相等；相同文本buffer字节、owner转移/活动载荷/once-release和必要时点对照。不可用这种布局差异豁免峰值/live预算或隐式clone。故障注入覆盖容器buffer扩容、Str clone和元数据登记；precheck→commit，分配/预算拒前原状态不变，部分输出回滚，正常清理后所有容器/文本live=0。

索引负数/越界/空容器在访问前拒绝，native采用既有exit70并提供父侧可区分的有界trap原因证据；正常return70必须不被当list-index陷阱。具体沿现有v2 exit/probe/原始stderr观测，不另建PASS协议；T-02核解码兼容与伪造/缺失/截断负控。clone按数据量的预算/轮询沿822义务映射到native helper；取消是AV/host/原生进程截止的哪一层必须分列，不能用外部kill冒认helper协作取消或清理成功。

### 5.4 真函数增量、缓存与发布

冷态可整源；纯body编辑用保留类型/签名环境及函数分段，真实单体parse/resolve/typecheck/HIR/lower，unit verify/encode/admit/COFF只变更目标函数；未改函数producer对象、代码/单元fp身份保留，实际工作=0。helper body编辑只换helper原生单元和必要链接，main源不变不得重lower；type/variant臂/record layout/签名或配置改变走明确依赖失效道，不要求该道前端零工作。

复用828候选与发布：失败/取消/坏链接/过期候选不改变当前source/env/坐标/root/manifest/可运行旧产物。连续变长编辑后坐标正确；失败尝试实际工作仍计量。native仅静态新产物，不要求活native帧HMR。CODEGEN/runtime/ABI/profile/类型内容/链接runtime库内容进入A09'/A10'身份，新增字段先做精确key/渲染/恢复兼容；旧profile不因无关库而改变身份。

fresh/hit/restart/body-edit及body回退历史内容分别实测；restart走decode+MIR/COFF结构准入+manifest/required runtime校验，持久化不恢复trusted标志。runtime版本/hash/类型布局改变不得命中不兼容COFF/manifest；没有变化时不得重新构建helper库却在计时/生产计数外隐藏。原三种cache模式与GC/required-data发布闭包保持，read-only miss真重算但零store writes。

### 5.5 正式消费与支持分账

H选择器拟为ac/mir-native-collections-v1（新recipe，T-02按真实合同冻结最终名字/字段）。旧ac/mir-native-scalar-v1及ace.m1.cert/1精确键集不变；新native suite引用原L/V源码及父侧expected，AV同源对照，不向worker传expected，不改原source/金样/分母/NA。只有真运行通过才改对应ac格为ok；W、闭包/Dyn/native HMR与其他M1 gap保持。codegen/构建/运行/恢复/资源与source/typed分列，profile4准入不冒领全typed指令面。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-mir/native-collections-variants.md | 无→864源码完整native支持集/布局/内部ABI/helper/资源/恢复、typed排除矩阵 | 诚实扩面 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-mir/native-data-owned-lowering.md | 一律拒collections→按本包子集准入；旧域及wide拒绝保持 | 不误称全profile | AC-01/02/06 |
| SD-03 | modify | docs/specs/auto-ac/native-backend-and-retirement.md | 五旧域终态→追加collections原生消费者/CLI/产物身份，legacy仍退役 | 不恢复旧lowering | AC-04/06 |
| SD-04 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | L/V ac gap→原源实测绑定，新增suite来源/资源/恢复负控；W仍gap | 分母守恒 | AC-01/05 |
| SD-05 | modify | docs/specs/auto-av/resident-function-install.md；docs/specs/auto-ac/source-collections-variants.md | 原collections后段carry→补真实单体前端/源坐标/候选/单位消费的实际证据与限制；修命名enum支持域误拒并明确源码与typed容器clone边界 | 明确M-前端差额，不能把低层carry冒领全链；纠正源码支持表含混，不扩源码clone方法 | AC-01/02/03/04/06 |

merge按实际delta更新overview/plans/索引/ledger；本轮不改canonical。

## 6. 测试设计

T-01仅完成17原源AV与native准入差额实核（证据在§9），本节其余为待执行work，不冒充native验收。真实路径：source→共享前端/lower→verify_mir→split_units→emit/link→PE；AV走collections_864_support::compile_and_execute/原正式recipe。source原例查crates/auto-lang/test/cases/ace-m1/src，资源/增量可复用822/864既有probe案例作为独立语义期望，不能运行AV后复制其输出为新golden。

| V | AC/不变量 | 输入/入口 | 区分错误实现的oracle/证据 | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 14原源码 | L01..L07/V01..V07原字节，经新SDK及直接mn PE；同源AV | L01/02/04/05/07=42，L03=0，L06=list-index trap；V03=3，其他V=42；正常return70控制不能满足L06。原L04/L05不是改写标量/typed fixture | T-02首例/T-04/06；父侧oracle |
| V-02 | AC-01/02 组合域 | 864 Str变体hello/中移出len；原for-in；新增List<Variant>含owned record与嵌套Variant，跨函数返回Variant/Result | Str变体51，原copy/owned for-in=42/53；V-02a复用T-01`list-variant-owned`精确源，Token.Data(Item("中",41))经List remove/is→name.len()+code=42，须先AV后native各自对oracle；V-02b无环嵌套命名Variant及同结构Option/Result控制；载荷正确且无隐式clone，inactive臂不能读 | T-02先S反例修复/冻结补源，再N；T-04/06。不得改原14源；不支持嵌List/容器字段 |
| V-03 | AC-02 显式clone/转移 | V-03a合法源码Str.clone对照；V-03b受检typed MIR Clone拥有List/Variant后独立set/remove/switch/drop，原根继续读取；move后读取/重复投影负例 | 原/副本内容独立，clone确有独立文本buffer；move无新文本alloc且旧根不可再用；每buffer/元素/活动臂释放恰一次。V-03c源码List.clone/Variant.clone维持具名拒绝，不把其前端红当native缺口 | T-02冻结入口；T-03/04/06。source/typed/runtime三起点分列，typed必须verify_mir→mn emit/link→PE，不能runtime直调替代机器链 |
| V-04 | AC-02 失败原子性 | push/insert/set扩容边界及clone中段Nth失败/元数据失败，预算0/上界/越界；旧目标已有内容；runtime直接受控探针与PE | 失败发生在commit前，原len/cap/元素与未提交输入责任保持；部分新缓冲回滚；双free/无主引用为0；分配拒绝、畸形、trap、timeout类别分明。PE fatal后不能用OS回收证明成功清理 | T-02先probe/T-03/06；故障缝不进入源语言 |
| V-05 | AC-02 生命周期/深度 | caller保List/Variant，callee trap；临时owned字段读取remove/view/insert；无环嵌套Variant clone/drop | 正常与本集合profile错误出口按冻结责任无泄漏；投影零隐式clone/额外资源事件；以822既有小栈/深链证人形状设计runtime级探针，T-02冻结可执行规模，不能以新增语言深度上限规避 | T-02/03/06；全路径显式栈，非只测最后drop |
| V-06 | AC-03 全链增量 | helper→main集合源；body 42→43→42、连续变长/UTF-8、接口/臂布局编辑；失败候选 | 纯body道每个实际生产阶段仅变更函数增量，main/其他函数工作0与producer/code身份保持；接口失效集正确；失败保旧source/坐标/manifest并可运行；强制whole compile/lower、同内容新对象反控红 | T-05/06；计数不能只来自下层carry |
| V-07 | AC-03/04 hit/restart | warm/hit/new worker、read-only miss、损坏COFF/manifest、正确校验和错owner/profile/ABI/runtime/type-fp、required库缺失/更换 | hit零COFF生产；恢复真实准入与原生运行；错误/过期永不trusted命中，read-only零写入；改变runtime内容下一次读取确实miss/stale并重建或拒绝，不能只贴标签 | T-04/05/06 |
| V-08 | AC-02/06 可信门/排除 | 合法原源控制；错List元素/Variant owner/guard/moved payload/环图/引用；typed Place/List参数/宽数值/闭包/Dyn/Unit入口 | 非法按共享verify或本包准入具名拒绝、codegen0、不panic；合法控制必须通过；旧拒绝测试更新只针对本包新支持，W gap不归绿 | T-02/04/06 |
| V-09 | AC-04 发布与身份 | 已有输出/manifest，运行时库更新、链接/发布失败；旧五域与新collection混合缓存 | 已有exe/receipt失败后字节/运行值保持；staging仅清理自有文件，残留如实报；旧五profile身份/支持集/871 legacy拒绝保持；新库必入依赖/key | T-04/06 |
| V-10 | AC-05 文件化/灵敏度 | 原14行+补充suite，新SDK真实worker，故意错oracle/错元素类型/漏一次free/强制重算 | 父侧FAIL、非零总门，且确认实际变异发生；缺原行/零选择/SKIP不PASS；report标source/native真实起点，原AV suite保持兼容 | T-06 |
| V-11 | AC-04/06 首例性能/内存 | T-02先冻结代表owned集合/变体多函数负载、样本/冷热/机器条件，fresh/hit/restart/edit及churn | P90≤2000ms（沿858 N-floor(N/10)排序位、原正式样本不减），P95/P99另报；guest8MiB/managed32MiB、MAX_LIST65536/超一边界，编译侧48MiB及worker128MiB真实峰值/回落成立；缺样本/失败不补成绿 | T-02口径/T-06实测，helper准备/构建不移出计时窗 |

首例probe至少含原L04/L05、List<Str>51、List<Variant>42、跨函数V02/V03与有资源caller的失败清理。T-02先核资源/表示全链可行后才扩面；T-05扩增量前再证明一次真实单体源码body生产/原子发布。没有机制就回T-02/needs_replan，不能缩掉拥有元素/变体组合。

T-02 r2执行入口（不增加AC、不拆走原必需项）：

| 子门 / owner | 基面/实际路径与状态责任 | 具体区分证据 / 关闭条件 |
|---|---|---|
| T-02/S前置，work | 权威树69d7b6b26；865/863已交付，开写前重核`ac-core/src/source/resolve.rs`及测试文件无他方占有；只改同一SourceProfile::Collections生产链，不另造前端 | 原T-01`list-variant-owned`source.undefined反例→修后完整frontend/CheckedHIR/shared lower/verify/ABC/AV=42；新`experimental/ac-core/tests/collections_889_prerequisites.rs`与`experimental/ac-av-probe/probe/tests/av_collections_889_prerequisites.rs`（拟新增）检查命名enum注解/载荷/返回、嵌套无环及未知/错型/递归/上下文排除；原17源hash不变。兼容保留源码容器clone拒绝。先提交S修复及实际释放清单；此子门绿不等T-02完成 |
| T-02/N首例，work | 接收已提交S前置，source→唯一shared lower→verify_mir→unit→CLIF/COFF→runtime→PE；runtime仅布局/原语，不解释指令。所需S/N同时就绪，H不取写权 | 原L04/L05=42、LSTR864=51、List<Variant>=42、跨函数V02/V03=42/3、有资源caller+callee trap清理及故障反控。显式记录source/typed/runtime起点；容器clone的typed输入先verify再真实emit/run。ABI/布局/清理、预算物理边界、样本/计时与trap70辨别以实际probe结果冻结；全部最小链和失败反控通过后才勾T-02、进入T-03 |

T-02/S只重放受影响新定向测试、原864相邻具名用例及必要check；不得以计划修订为由运行t/tv/tf等整档。T-01基线是可复用未改源历史证据，修复后重新执行其受影响源路径并保留红/绿分账。本文“拟新增”的测试/机制尚不存在，不能作为已通过依据。

角色未指定；正常独立final review从提交与工件重建全部AC/V/SD，实施自检不替代独立验收，不自动创建agent/chat。review期间实施停写同树，修复回同树；pass记录merge-owned pending。原始探针/日志/JSON/截图仅本地attachments，Markdown摘要记录命令/退出码/数字/路径/hash，禁止add/commit/push非md证据。

拟验证入口（T-06建成前不可宣称已运行）：cargo test --manifest-path experimental/ac-mir-native/Cargo.toml --test collections_source_tier / collections_resources_tier / collections_incremental_tier（分别选目标）；新runtime manifest的定向tests及实际PE；powershell -File scripts/verify-native-collections-889.ps1 -Scope source|resources|incremental|compat|acceptance（选一个值）。复用旧822/864/881/871受影响命名case，开发/repair不跑内嵌整档旧脚本或裸t/tv；必要cargo check按真实workspace包名执行。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 864完整合法源域进入native，14原L/V+Str/for-in/List<Variant>补例正确，不改源/期望/分母；V-01/02 |
| AC-02 | work | 容器/载荷/跨函数所有权、临时view/显式clone、深度安全、失败原子性与本包清理/预算合同成立；非法与不支持输入分开拒绝；V-03..05/08 |
| AC-03 | work | source→native全链真单体增量、身份保持/精确失效、失败候选保旧、真实restart准入；强制退化红；V-06/07 |
| AC-04 | work | runtime/ABI/key/依赖/发布受检、原旧域兼容与P90≤2000ms/内存门实证；V-07/09/11 |
| AC-05 | work | 文件化父侧原L/V真实消费和变异负控、gap仅凭运行关闭，W等其他gap不冒领，source/typed/emit/build/run分列；V-10 |
| AC-06 | work | S/N/H逐阶段释放/接受与已有消费者兼容齐全，唯一lowering/无AV解释旁路/不复活旧native、无新opcode与根实验依赖；V-08/09/11及依赖守卫 |
| AC-07 | review | 独立核AC-01..06、V-01..11/SD-01..05，显式扫描漏原源、借用偷clone/只测后段cache、未处理警告/fmt/debug；必要oracle未执行不pass |
| AC-08 | merge | 最终review后冻结提交裸cargo t+必要tv及实际experimental触面定向档通过；修复代码则重审受影响面后新冻结门；tf归主检出到期单实例 |
| AC-09 | merge | SD/overview/plans/索引/ledger沉淀、归档、guard清理、正式交接真实revision与残留gap；不宣称M1全绿/typed entry完成 |

## 8. 执行步骤

| ID | owner_stage | 前置/窗口 | 位置、操作、关闭证据 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续明确执行授权；只读准备可提前 | 核D864/D871/D881与现行S/N/H owner、源码/MIR指令/类型/模式闭包、native缺口及资源runtime；逐14原源/补例/oracle/父侧入口映射，记录标量MN会话与集合whole-recompile差额 | AC-01/02/03/06 | [x] |
| T-02 | work | T-01；先接受已释放S最小子集并提交，再N；H不写 | 按§6执行入口先修命名enum规范支持域误拒及兄弟上下文/无环/拒绝控制，保源码容器clone拒绝；再冻结类型表示/内部ABI/runtime/context清理、预算/样本、typed排除和trap观测，跑最小source→唯一shared lower→真实PE全链资源probe及失败反控。S子门通过不能单独勾本步；固定codegen/cache/runtime升版与891交接边界，失败回本步不缩合法域 | AC-01/02/04/06 | [ ] |
| T-03 | work | T-02资源全链probe通过；N/必要S窗 | 实现N内容器/变体runtime与clone/drop/故障/预算/跨帧清理helper，文本复用及总账；V-03..05；新runtime位置/导出/库合同实测 | AC-02/04/06 | [ ] |
| T-04 | work | T-03；N | emit/lower/layout/driver/CLI/worker typed准入、Variant调用返回、CLIF/COFF与runtime依赖；新profile选择/缓存/恢复/原子发布；原L/V与List<Variant>、V-01/02/07..09定向 | AC-01/02/04/06 | [ ] |
| T-05 | work | T-04；S及必要N同时释放接受，默认865后 | 补集合单体前端/环境/候选/坐标与native增量消费者，扩现有828路径；先一次真单体probe再扩V-06/07；不以CollectionSourceSession下层carry代前端0工作 | AC-03/04/06 | [ ] |
| T-06 | work | T-05；H与必要N/S释放接受 | 新recipe/SDK/suite/原binding实测，V-01..11与source/native/资源/恢复/性能/负控；受影响消费者兼容，提交实现及SD草案/证据摘要，给后继typed包N接口清单 | AC-01..06 | [ ] |
| T-07 | review | T-06 | 独立final review绑定Plan/代码/依赖，merge-owned门pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交裸t+必要tv/experimental定向门；归因、修复重审/重冻；批量tf到期按主检出单实例 | AC-08 | [ ] |
| T-09 | merge | T-08 | SD-01..05、索引/ledger、逐源M1绑定来源/限制、交接N与S/H；归档及真实delivery记录 | AC-09 | [ ] |
| T-10 | merge | T-09 | wt-guard clean才移除树/分支/无兄弟组；记录10/10和cleaned收据，typed后继仍未因本包自动实施 | AC-09 | [ ] |

## 9. 复审记录

2026-10-29 stage: new | plan_id: PLAN-889 | plan_revision: 1 | outcome: pass（规划静态预检，不是实施验收） | next: 用户明确执行后T-01，T-02 N私有probe；T-05等S实际释放，T-06等H | authorization: 本轮仅规划 | split: 集合/变体与typed entry两包，后继未分号 | changed: AC-01..09 / V-01..11 / T-01..10 / SD-01..05。

已核旧native退役、三类准入/槽/指令/driver拒绝及集合会话whole compile事实。未编译/执行探针/测试、未建worktree/agent、未写源码/canonical/ledger或接触他方WIP。.next-id已有他方取号改动，中央889→890累进保留主检出，未来开树前提交本合同与协调后的号源，不夹带888未提交计划。

### 2026-10-30 work 第1轮：T-01实际源码闭包与上游前置差额

- 授权继承用户 `/goal 完成计划889`；创建唯一权威树 `D:/autostack/.wt/lang-889/auto-lang` / `plan-889-dev`，base `e0b29ff7a0afd2479904b304780d6818306b54e1`，auto-down只读pin `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`。主检出既有文档/号源WIP保留，未夹带；889合同单独提交e0b29ff7a。未建agent、联系其他聊天或推送。
- T-01完成，1/10。实现提交 `69d7b6b26`：永久 `experimental/ac-mir-native/examples/collections_inventory.rs` 与受控Markdown执行入口 `docs/plans/attachments/889/execution-entry-r1.md`。具体源→shared lower→verify_mir→split→native admission数据流、独立oracle与22条原例MIR指令闭包、S/N/H所有权/真实增量与资源差额见该入口。当前树clean；D864/D871/D881/D865/D863 delivery祖先逐项实核。
- `cargo run --locked --manifest-path experimental/ac-mir-native/Cargo.toml --example collections_inventory -- --original-only` exit0：17原例AV期望全部匹配，L04/L05=42、L06精确list-index trap、V03=3、补例51/42/53；成功资源live0且alloc/free平衡。所有native单位仍mn.unsupported.profile，native执行0，绝不关闭任何native gap。正常return70与List<bool>两个独立控制也AV通过。
- 三个补例揭示原设计假设不成立：List<Token>（Item("中",41)载荷）在`vt_of_ann_coll`仅查record名字、漏enum名字，source.undefined；List.clone显式source.unsupported、Variant.clone缺方法。后二者须利用V-03已允许的typed资源探针，不擅增源码方法。前者与既定Spec/AC-01支持List<Variant>冲突，须修规范已承诺域的上游缺陷再做N全链probe；不删除验收或降低范围。
- N-only首例先行的假设需有界调整，下一阶段auto-plan-new在既有授权下修订r2：T-02先记录已交付865/863释放的S最小子集接入，再N；澄清source/typed clone分账，其他AC/阈值/仓库/预算/源码拒绝政策不变。H仍888所有权未交接，本轮只读核diff，无自动写权。此为needs_replan、非用户决策阻塞，目标保持active。
- 本地raw根 `<worktree>/docs/plans/attachments/889/baseline/`，均未暂存：inventory.json SHA256 `931320b5f185124383f3501c8e1ff1b827979448092c5aa207f47c5c2d930323`；list-variant-owned.stderr `49df1f68f20e09661fbf69efbd333eeab41b48c4ee3b3e14db76a3daee3dc80f`；三补例汇总supplement-results.json `e97ce058187bd86fcb947c2a857f2700fa3a9ce4e3ecb2dd248b5b7ddfa9ea88`。fmt/diff-check通过；owned_session的shell_lo/shell_hi既有warning如实保留，无新example warning。未跑整档t/tv/tf/docs_gen。

stage: work | plan_id: PLAN-889 | plan_revision: r1 | outcome: needs_replan | worktree_path: D:/autostack/.wt/lang-889/auto-lang | branch: plan-889-dev | base_commit: e0b29ff7a | code_commit: 69d7b6b26 | dependency_revisions: auto-down=c3464d0a | task_ids: T-01 | evidence: attachments/889/execution-entry-r1.md | blockers: none requiring user decision; bounded design order/source-vs-typed correction | next_responsible_role: new

stage: goal | plan_id: PLAN-889 | plan_revision: r1 | round: 1 | from: work | to: new | status: executing | outcome: needs_replan | worktree: D:/autostack/.wt/lang-889/auto-lang | commit: 69d7b6b26 | next: new

### 2026-10-30 new 第2轮：有界r2修订，回到work

- 上轮分类progress：T-01实际oracle/准入/反例改变下一步、工具与入口提交69d7b6b26；不是状态重述。本轮按needs_replan加载auto-plan-new，重核主检出无代码WIP、889同一权威树clean且HEAD69d7b6b26；未建第二树/agent、未联系其他聊天、未改他方计划或WIP。
- r1→r2只调整T-02设计顺序/入口与source-typed分账：规范原已承诺List<variant>，`vt_of_ann_coll`仅查record表是上游前置缺陷，T-02先接受865/863已交付后的S最小子集、修源域并提交，再N真实资源probe。必要相邻命名/嵌套/上下文反控列入同一步；不是以失败原例缩支持面。S子门不单独计T-02完成，原全链资源可行性门不后移。
- 原V-03允许source/typed资源探针分列，现在显式绑定：源码合法Str.clone保持原路径；容器本体List/Variant Clone以受检typed MIR→mn→PE与runtime分层验证，源码List.clone/Variant.clone既有拒绝保留。两个T-01“源码clone成功”假设已作废，不删除容器clone、深度、失败原子性、独立buffer与once-release验收。SD-05补明确Spec表含混的修正来源。
- 目标G1..4、AC-01..09、阈值（P90/样本/预算）、仓库/授权、14原源及17基线oracle不变；无新增用户决策。当前1/10，仅T-01完成。历史规划“本轮仅立项/不建树/865→889→863/后继未取号”等已由现状替换，原规划和r1记录保留在§9与Git历史；891另立但未授权施工，N仍889先行，不自动启动后继。
- 静态依据：source Spec SHA256 a82dafa63e2a2426c4d4c2716a1bca64c4adee9385207b7fc9c6fdeb080cfdcd；权威树resolve.rs SHA256 4c7f8ca56f116a7076dbebdb00a1f9a2e690123f81fa7b04b5fb93ad863b25ae；旧Enum登记在payload解析后、Optional/Result已拒Field/Param，r2要求兄弟路径一致保限制。未编译/运行新探针或写实现/canonical/ledger；仅计划修订与静态一致性校验。

stage: new | plan_id: PLAN-889 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-889/auto-lang | branch: plan-889-dev | base_commit: e0b29ff7a | code_commit: 69d7b6b26 | changed: T-02 order/entry, V-02/V-03 supplemental binding, SD-05 wording; AC unchanged | pending_decisions: none | next: work（T-02/S前置→N首例，S开写前重核接受清单）

stage: goal | plan_id: PLAN-889 | plan_revision: r2 | round: 2 | from: new | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-889/auto-lang | commit: 69d7b6b26 | next: work

2026-10-30 T-02/S写入接受：前owner865 r2/863 r1，delivery分别5fa3133fa/2d5f3d842，均已落master并清语言树。889 r2在权威树69d7b6b26接受`experimental/ac-core/src/source/resolve.rs`的CollTables/resolve_detailed/vt_of_ann_coll与新增`tests/collections_889_prerequisites.rs`、probe新增`tests/av_collections_889_prerequisites.rs`；当前889树clean，888已提交diff在上述文件/符号无重叠，未消费其WIP。shared HIR/profile4/MIR schema/opcode/ABI均沿现行已交付身份；本次只修规范支持域查找、图拒绝及上下文限制，提交后记录精确释放。H未接受，text shim未接受，其他S路径不预占；新增N原型仍限本包权威树。主检出仅他方文档WIP，未改其他计划/联系聊天。
2026-10-30 T-02/S阶段提交/释放：`299cafa18`（6文件）完成上述命名enum查找/表序无关无环检查与定向永久控制，source7/7、shared MIR→ABC→AV3/3、原864前端16/16、Closures共享resolver相邻1/1通过；原17源与AV观测逐项hash/结果相同（inventory-after-source.json与原inventory.json SHA256均931320b5f185124383f3501c8e1ff1b827979448092c5aa207f47c5c2d930323）。新源码最初两fixture误把owned函数返回直接push，触既有move-required；改为局部+显式.move后通过，未改该产品拒绝政策。S子门提交可复核，当前停止S写入并转N私有runtime/机器首例；T-02整体仍未完成，不宣称native交付或H释放。
2026-10-30 T-02/N文本库组合追加接受：首个runtime PE链接精确报两静态库重复`__rustc::rust_begin_unwind`，为同一进程组合既有文本实现而非复制文本语义，889接受已释放的`experimental/ac-core/runtime/text/shim/Cargo.toml`之crate-type行：保留staticlib并增加rlib输出供新collections静态库依赖；text/shim/src/lib.rs不写，旧静态库出口/默认ABI/生产选择不变。N库复用该rlib提供的唯一panic_handler及原text函数，移除自身panic_handler，真实PE仅链接组合库，禁止/force:multiple掩盖符号错误。当前其他活跃888/890提交未触该文件；先验证组合/旧出口兼容再记录释放。
### 2026-10-30 work 第3轮：S前置修复提交，N运行时真实PE资源probe

- 上轮分类progress：5f35006cd将计划从r1修订为r2并形成可执行入口，G/AC逐字不变；本轮按auto-plan-work继续。重核889唯一权威树/root/branch，main无代码WIP；保留其他会话文档/新890树注册，不修或删除他方worktree元数据。未建agent/联系其他聊天/推送。T-02分项有进展，但整体仍**1/10、executing**，T-02不勾。
- `299cafa18`关闭T-02/S前置：同一Collections resolver先预登记命名enum、统一合法类型注解查找、迭代检测含Option/Result边的payload环；保Field/Param/嵌List/源码容器clone等排除。新source7/7、shared lowering→verify→ABC→AV3/3（List<Token>、前向嵌套enum/跨函返回、Option对照均42，live0且alloc/free平衡）；原864前端3+拒绝13=16/16，相同resolver的Closures具名控制1/1。原17行源码hash和AV事件/结果逐项相同，修后inventory JSON与T-01同SHA256 931320b5f185124383f3501c8e1ff1b827979448092c5aa207f47c5c2d930323。先前两个新fixture把拥有型函数返回直接push触原move-required，改为局部+显式.move后通过，未改该拒绝政策。
- `8096d8fdf`是N runtime-only可行性实现（不是完成T-02/源码native交付）：新增`experimental/ac-mir-native/runtime/collections`、锁、draft CONTRACT、`tests/collections_resources_tier.rs`及永久原生调用fixture。no_std容器/record/活动变体/owner原语，16B Cell、受管节点/借用共享、显式clone/无分配迭代drop、生成帧登记和跨帧trap cleanup；无MIR/ABC reader/opcode dispatch/AV执行依赖。原emit/lower/driver/cache/profile/recipe均未扩，原源native执行仍0。
- 真实COFF/PE定向资源控制**5/5 pass（2.70s）**：①List<Variant<Item>>与文本投影/显式clone独立、buffer扩容和元数据Nth失败前原列表/输入/旧dst保留、Str clone失败/零预算回滚；②trap清callee与有资源caller，exit70+code21且容器/文本live0，正常return70无trap行；③64KiB实际保留栈（CreateThread STACK_SIZE_PARAM_IS_A_RESERVATION）上构造4096层Variant、部分clone失败回滚、完整clone/drop，容器allocs/frees=8196/8196、peak requested=983334B、文本2/2、live0/invalid0；④真实编译的漏frame leave反控同exit42但live/bytes非零；⑤额外丢text owner反控容器live0仍由父侧text摘要比较拒绝。不是将runtime-only结果当source→MIR→CLIF证据；未来typed clone仍须verify/emit/PE链。
- 同一组合库最初两独立staticlib重复rust_begin_unwind，导出名尝试未改变该编译器lang-item符号，已弃；改为文本shim增加rlib输出供N库嵌入既有实现和唯一panic handler，文本src完全未改，旧staticlib仍保留。不用force-multiple。随后暴露core格式化/边界panic链接CRT，N有界观测改为受界raw字节写，PE仅kernel32真实链接通过。最初Str Nth注入用1未命中（shim绝对序号已到3），永久fixture改4并要求精确拒绝与旧值/bytes不变；保留失败日志，不冒称旧红已通过。
- 文本组合S只改已登记的Cargo.toml crate-type行；现停止该子集写入。旧单独staticlib `mn_owned_strcmp_and_codepoint_machine_faces`具名机器兼容1/1通过（1.48s），没有改旧backend/ABI/key。N runtime冻结候选build --release --locked通过；独立构建显露文本shim**原源**四项suspicious_runtime_symbol_definitions（u8指针与c_void指针ABI等价，mem*出口沿原源码，src diff=0）以及原probe shell_lo/shell_hi warning，均记录为已知基面诊断，未用全局allow掩盖；新N crate无新增warning。fmt/diff-check通过。
- T-02未完项明确在runtime/collections/CONTRACT.md：**源码→shared MIR→CLIF/COFF真实原L/V/补源首例与typed clone仍未接线；名义布局/签名准入待实现；文本shim retained ledger容量/描述符/heap overhead/进程physical峰值未纳入当前requested bytes计数；数据比例轮询/取消、完整边界/运行时身份key、正式负载样本与性能口径待冻结。**当前zero nodes/requested balance不能当32MiB/128MiB物理门通过。下一轮同树继续T-02/N shared-MIR emitter及资源计量缺口，T-03..06不提前关闭，不交最终review。
- 所有raw本地保留未暂存，根`D:/autostack/.wt/lang-889/auto-lang/docs/plans/attachments/889/`。baseline/source-prerequisites-final.log SHA256 9c8dc9f91c70884eeb1e162e570ed7b88abe1e0e804c3390d1a52b79bb324274；av-prerequisites.log 16e58bee0690e92d501de15f3c1484ecb35b29e63b817da1d6bb275086dd774c；runtime-current-controls.log 7ee3d31c0ccd2e248f62320b6837785a221c96e3294b1efe96034528632c88a5；text-staticlib-compat.log 33549e2762e11e5b4cb3a1d8b78a433959fb69e901a198cac0bbfe1db9e8b2d6。runtime-deep64k/primitives/trap/normal70/leak-control/text-leak-control各独有目录保PE和stderr；早期红日志保留。未跑整档t/tv/tf/docs_gen或正式性能采样；H未接受、canonical/ledger未写。

stage: work | plan_id: PLAN-889 | plan_revision: r2 | outcome: progress（T-02/S已提交，N运行时5项真实PE控制通过，T-02整体未完成） | worktree_path: D:/autostack/.wt/lang-889/auto-lang | branch: plan-889-dev | base_commit: e0b29ff7a | code_commit: 8096d8fdf | dependency_revisions: auto-down=c3464d0a | task_ids: T-02 | evidence: runtime/collections/CONTRACT.md; baseline/runtime-current-controls.log | blockers: none requiring user decision | next_responsible_role: work（shared MIR native emission+完整资源计量）

stage: goal | plan_id: PLAN-889 | plan_revision: r2 | round: 3 | from: work | to: work | status: executing | outcome: pass（有实质实现与反控进展，work未完成） | worktree: D:/autostack/.wt/lang-889/auto-lang | commit: 8096d8fdf | next: work
## 10. 待澄清事项

- 支持面已选864完整合法源码与其MIR闭包，不留“是否包含List<Str>/拥有record/Variant”的开放裁定。T-02选择等价native表示/allocator/helper机制；若预算内无法实现，给最小反例与方案needs_replan，不自行缩范围或加语言深度限制。
- 865/863/887已交付；T-02/S开写接受与T-05后续S、T-06/888 H逐符号交接、SDK路径、runtime支持库兼容与trap有界观测仍归相应任务核验。提前准备不取得公共H写权，不自动联系其他会话；当前不阻塞独立T-02/S/N。
- typed entry另立891仅规划，其范围/结果通道由该计划冻结；本包保持W01–05等拒绝/原gap，不因它未实施阻塞本包交付，不自动启动或修改其合同。
