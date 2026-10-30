---
plan_id: PLAN-889
status: reviewed
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
current_step: 7
total_steps: 10
---

# [PLAN-889] native集合与变体源码闭环

## 0. 变更摘要

将864已有的集合/变体合法源码，经同一Checked HIR→共享collections MIR→mn CLIF/COFF→真实PE执行；关闭M1原L01–L07/V01–V07共14个native源码格。维持零参i32入口/结果，继承822集合与资源语义；同时补集合源码的真实单函数增量，而非整模块重算后报告carry。

[拆包决定](reports/ace-native-phase2-split-decision-2026-10-29.md)：native集合/变体与带位宽typed entry拆为两个计划，先本包；后继typed entry现已另立891（仅规划，不构成本包D门，也不授权本包启动它）。List和Variant共享布局/资源闭包，合法List<Variant>必须本包支持，故不再拆成两半。r2据T-01反例调整T-02顺序：先修规范已承诺的命名enum类型查找，再做N全链资源probe；区分源码clone拒绝与typed容器clone验收，不缩减G/AC。T-01..06已验证并提交；正式五路径各30/512的完整实测与受影响cold verifier修复已按精确机制分界合并证据，最终代码/验收/规范复审和merge-owned门仍待执行。

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

权威实施树D:/autostack/.wt/lang-889/auto-lang，分支plan-889-dev，base=e0b29ff7a，当前实现检查点cb5ce40db；一Plan一权威树，实施/独立review/修复均同树顺序执行。根workspace不新增实验栈依赖，auto-av-core零依赖边界不变。

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
| T-02 | work | T-01；先接受已释放S最小子集并提交，再N；H不写 | 按§6执行入口先修命名enum规范支持域误拒及兄弟上下文/无环/拒绝控制，保源码容器clone拒绝；再冻结类型表示/内部ABI/runtime/context清理、预算/样本、typed排除和trap观测，跑最小source→唯一shared lower→真实PE全链资源probe及失败反控。S子门通过不能单独勾本步；固定codegen/cache/runtime升版与891交接边界，失败回本步不缩合法域 | AC-01/02/04/06 | [x] |
| T-03 | work | T-02资源全链probe通过；N/必要S窗 | 实现N内容器/变体runtime与clone/drop/故障/预算/跨帧清理helper，文本复用及总账；V-03..05；新runtime位置/导出/库合同实测 | AC-02/04/06 | [x] |
| T-04 | work | T-03；N | emit/lower/layout/driver/CLI/worker typed准入、Variant调用返回、CLIF/COFF与runtime依赖；新profile选择/缓存/恢复/原子发布；原L/V与List<Variant>、V-01/02/07..09定向 | AC-01/02/04/06 | [x] |
| T-05 | work | T-04；S及必要N同时释放接受，默认865后 | 补集合单体前端/环境/候选/坐标与native增量消费者，扩现有828路径；先一次真单体probe再扩V-06/07；不以CollectionSourceSession下层carry代前端0工作 | AC-03/04/06 | [x] |
| T-06 | work | T-05；H与必要N/S释放接受 | 新recipe/SDK/suite/原binding实测，V-01..11与source/native/资源/恢复/性能/负控；受影响消费者兼容，提交实现及SD草案/证据摘要，给后继typed包N接口清单 | AC-01..06 | [x] |
| T-07 | review | T-06 | 独立final review绑定Plan/代码/依赖，merge-owned门pending | AC-07 | [x] |
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
2026-10-30 T-02/S追加最小查缺接受：V-03a合法Str.clone源例经已有resolver/emitter生成expr.text_clone，collections.atom却遗漏该case，报source.internal/bind.unknown-case；现行Spec“owned局部.clone”和r2 V-03a均已承诺此路径。889接受`experimental/ac-core/schema/collections.atom`的ExprKind/text_clone分支及既有前置tests定向补例，888/890该路径提交diff无重叠；不接受H写入、不启用List/Variant源码clone。不新增HIR表达式种类/MIR opcode/AV执行语义；复用已有descriptor绑定、owned共享lowering的MirInstr::Clone。属于既有约定域缺口的机制修复，原17源/阈值/AC不改。
2026-10-30 T-02/S Str.clone实际链补记：descriptor补case后shared lower仍在已有TextClone→Clone分支用!collections过滤同一操作，报lower.expr；接受`experimental/auto-mir-lower/src/hir_adapter.rs`该TextClone分支最小guard修复，888/890无该符号提交diff。沿同一已有Str→Clone映射，不另造lowerer、不加opcode/AV语义，不改有效旧源行为或旧五域lowering身份；本次是恢复Spec/r2已约定的Str.clone面，实际canonical修正和source支持身份由SD-05/T-04/06记录。没有扩大源码容器clone。
2026-10-30 T-02/S record参数查缺接受：合法普通Pair参数的已Checked源码在shared lower后被verify_mir以mir.param-type拒绝（param槽RecordView、collections_fn_sig_for_def签名Record）。接受同一hir_adapter.rs该签名builder的View模式映射最小修复及对应AV/native相邻用例；调用方/函数体继续共享现行lower_function与受检前端，不改MIR或AV，owned/data旧分支不改。T-05 retained单函数同签名消费也须复验此分支，不能只修whole形态。
2026-10-30 T-02/S命名变体Str载荷查缺接受：最初新增caller/callee清理fixture中的Token.Data("caller")在源层报expected Str/found str；先用既有Item载荷形状验证清理后，确认该失败也属已约定的owned Str变体面，不能以改fixture隐藏。接受resolve.rs的RecvClass::VariantArm构造分支最小上下文literal retag（复用Result/record已有retag_literal）及定向AV/native正反例；旧L/V源不改，不放行错误int载荷，不增加容器源码clone。修后保留直接Str载荷caller-trap与Item载荷两种控制。
### 2026-10-30 work 第4轮：共享MIR→CLIF/COFF→真实PE首例接通

- 上轮分类progress：299cafa18/8096d8fdf已改变实现与资源观测，非状态重述。本轮继续auto-plan-work；889权威树/root/plan-889-dev重核，入树HEAD8096d8fdf、实现clean，main代码路径无WIP；他方文档与888/890树保留，未建agent、联系其他聊天或写外仓。仍1/10、executing；T-02的源码机器首例已过，预算/身份/样本冻结缺口未清，不提前勾T-02或T-03。
- 实现提交 `0a73c1c82`：新`src/collections_lower.rs`消费同一shared collections MIR，admit先verify_mir并显式列类型/指令/签名排除；算术/溢出/除零/一元运算、分支、跨函数调用直接发CLIF，容器/拥有值直接调用原语，发每函数COFF再真实链接PE。runtime无MIR/ABC/opcode dispatch/AV旁路。新私有内部ABI `mn.collections.cell/1`（参数Cell指针+尾随out Cell，Win64）适配既有零参i32进程结果；生成函数通过受管frame_open申请Cell槽区，32B自动frame头登记整链、避免大自动槽数组先爆本机栈。Copyable record以不可变node共享，源View参数无drop权；resource Move/Clone/Drop及临时字段共享沿受检类型发射。`make_isa`只提升crate内可见性，旧emit机器行为未改；旧driver/cache/profile/worker/recipe**尚未选此入口**。
- 当前冻结新增触面验证 `cargo test --locked --manifest-path experimental/ac-mir-native/Cargo.toml --test collections_source_tier --test collections_typed_tier -- --test-threads=1` exit0：source7/7（10.54s）、typed2/2（0.63s）。source是**33个具名输入**（17原源逐字+3 List/嵌套/Option补源+normal70+两个资源caller陷阱+10个相邻值/陷阱输入），不是反复测试次数；均真shared MIR→CLIF/COFF→PE，并保source hash/发射单位数/库hash/真实stderr。原L04/L05=42、LSTR864=51、V03=3、FIN864A/B=42/53、L06精确code21+exit70，正常return70无trap行，所有正常/受控trap容器和文本live0、invalid0。
- typed builder不冒源：两个Clone组合各先verify_mir→unit→同一emitter/PE，List<Variant<Item>>及额外Variant根clone经独立移出/投影/显式drop后原根仍可读，结果84；实际文本alloc/free分别2/2与3/3，CLONE id次数1/2。错List元素类型在合法unit对照先admit后真实变异，精确`mn.admit.mir`含`mir.list-type`拒绝，ObjectModule创建在admit后。未把runtime-only探针当typed机器链。
- `2c1e5f104`修复同一既有S链的两个规范面缺口：Str.clone的collections descriptor漏text_clone case及共享lower已有Clone分支误排Collections；普通record View参数签名原为Record而body槽为RecordView，统一collections_fn_sig_for_def模式映射。未复制前端/lowerer；原Text/Data/Owned有效映射和旧五域身份不改；Str.clone guard只补Collections、不扩大Closures的text_clone。source新前置8/8、AV新前置6/6通过，含whole与真实lower_function_collections_unit→verify/ABC/FunctionCode/snapshot链接执行对照42。
- `9035173d5`补命名变体Str字面量上下文：起初caller-trap的Token.Data("caller")在源层expected Str/found str；Item载荷形状验证清理后未将该缺口掩掉，按既有retag_literal修命名VariantArm分支，再保留直接Str载荷与Item载荷两个caller陷阱。新AV具名控制1/1、实际错误int载荷反控拒source.type-mismatch，native直接Str载荷caller-trap通过。源参数/容器clone排除不变；原17源与oracle不改。两项首次fixture失误（owned返回直接push及保留字copy）均只修新fixture合法拼写，原红日志保留；source产品规则未放宽。
- S本轮追加接受与精确符号已在前序记录，本轮提交后停止S写入；未来T-05 retained前端仍须复验模式/接口与坐标。shared lowering身份保持现行3：此次恢复原约定操作/参数模式的既有映射、无新的MIR/AV语义；旧可成功验证单元继续有效，修前相关源码或MIR链即拒绝。Collections新native ABI/runtime/cache身份在T-04冻结绑定，不能拿旧标量标签冒用新布局。
- runtime CONTRACT.md已更新实际路径和未完项。T-02仍需完整元数据/真实物理预算（现requested计数未含text shim retained ledger、heap overhead/描述符及进程physical peak）、数据比例轮询/取消、完整边界/类型布局身份、性能负载/样本/计时口径冻结。T-04正式SDK/CLI/cache/恢复/发布、T-05真源码单体前端、T-06H与性能及所有复审/merge门均pending；没有关闭正式M1 ac gap或宣称AV/profile4全typed支持。
- 证据仅本地：`D:/autostack/.wt/lang-889/auto-lang/docs/plans/attachments/889/baseline/native-affected-current.log` SHA256 `87ba53136235adf85bc45c600a511cae531916fcd60b339fdabfcdaf79769c6c`（大小写不影响hex）；av-prerequisites-current.log `734706db874d8a509881f8192f06166482c5b901c967add505ca9ce84c1fbb1c`；av-named-variant-text-context.log `f1eeab80340948b6ba2a5033c5ee2a0f72a03996319b80ab54e2d46569e5aada`；source-prerequisites-current.log `e99d3097e16f60aedbf723c1fb90c3b6009ee8fa86e0e3ab0377191c42bad1da`。每个source-<tag>-<pid>-<nonce>目录保实际COFF、PE、stderr和receipt.json（source/typed起点分列，expected未进child）。未暂存raw，fmt/diff-check通过，新N无新增warning（probe旧shell_lo/hi已知）；未跑整档t/tv/tf/docs_gen、未正式性能采样、未写H/canonical/ledger。

stage: work | plan_id: PLAN-889 | plan_revision: r2 | outcome: progress（T-02原源与组合机器首例/typed clone已实跑，完整资源冻结尚未完成） | worktree_path: D:/autostack/.wt/lang-889/auto-lang | branch: plan-889-dev | base_commit: e0b29ff7a | code_commit: 0a73c1c82 | dependency_revisions: auto-down=c3464d0a | task_ids: T-02 | evidence: baseline/native-affected-current.log; runtime/collections/CONTRACT.md | blockers: none requiring user decision | next_responsible_role: work（T-02资源元数据/物理计量/轮询与口径冻结）

stage: goal | plan_id: PLAN-889 | plan_revision: r2 | round: 4 | from: work | to: work | status: executing | outcome: pass（原源真实native已接通，有实质实现/反控进展，work未完成） | worktree: D:/autostack/.wt/lang-889/auto-lang | commit: 0a73c1c82 | next: work
2026-10-30 T-02/S运行时分配/轮询接受：为关闭实际metadata/物理预算而非只估算logical bytes，889接受已交付text/shim/src/lib.rs的分配/释放缝（5处HeapAlloc/HeapFree）、新增可选allocator/poll安装与metadata查询出口、own_from_lit/own_clone数据拷贝轮询。888最新4bf398580/890在该路径无提交diff，主检出代码零WIP。新N进程在首次资源分配前显式安装有界VirtualAlloc arena；默认旧shim不安装、继续原HeapAlloc/HeapFree/诊断与ABI。安装须拒绝已有live或ledger，禁止跨allocator释放。metadata/节点/文本buffer/clone工作栈/槽区共用32MiB实际页提交上限，保guest8MiB/list65536；测试分开报告requested/页提交/进程private观测。没有新AV opcode/wire/执行语义或公开语言FFI，旧五域不自动启用该策略。仅在本权威树写入并做旧shim具名兼容；S提交/释放须记录，H未接受。
### 2026-10-30 work 第5轮：实际页额度与取消回滚

- 继续同一权威树/分支，入树HEAD0a73c1c82且实现clean；main仅文档WIP，未写他方文件、未建agent或联系其他聊天。T-02仍未整体关闭，current_step保持1/10。
- S提交`8eb007eb5`落实前序接受的最小分配缝：text shim新增可选allocator/free/poll安装、metadata查询与idle释放；旧消费者未安装时继续原process heap和保留策略。安装拒绝live文本或retained ledger，避免跨allocator释放；own_from_lit/own_clone每4096字节轮询，取消回滚新buffer并报-12，N映射为-26。无AV opcode/wire/执行语义改动。S该子集提交后释放，后续S写入仍需按符号重新核窗。
- N提交`1e240cfa2`：新增32MiB保留地址域/按4096B提交的VirtualAlloc arena，节点/数组/Cell堆槽/clone工作栈、文本buffer及ledger、allocator头/对齐/碎片均占同一页额度；尾部回收decommit。请求字节与arena live/commit/peak/invalid、text_metadata分列。probe额度只能单调收紧且不低于已commit，生产上限不能扩展。自动32B Frame头、原生栈和PE映像不算入arena，仍归进程128MiB delta独立门；没有把arena上限当整个进程上限。
- clone按节点/child和文本chunk轮询，半成品沿无分配/不轮询drop路径回收，原src/dst保持；list插入/扩容/移动工作在修改前按数据量准入，remove亦先准入。last-frame清理释放idle文本ledger，正常/受控trap要求容器与文本余额以及arena live/commit/invalid/metadata均0。外部kill仍为INFRA，不宣称kill后执行清理。
- `cargo test --locked --manifest-path experimental/ac-mir-native/Cargo.toml --test collections_resources_tier --test data_owned_tier -- --test-threads=1` exit0：资源11/11（7.63s）、旧data/owned机器面6/6（5.23s）。新增实际PE控制：4KiB页额度拒绝且peak4096、非尾碎片复用/前后合并、初始和第17条文本ledger增长失败回滚、精确8MiB guest及+1、精确65536列表及+1、文本chunk与partial列表clone取消、原Heap live/retained metadata时拒绝接管。depth4096/64KiB reserved stack、跨帧trap和两个遗漏释放负控仍绿。
- 同轮受影响source7/7（33具名输入，9.26s）、typed2/2（0.55s）通过，父侧新增arena余额断言；这些运行绑定相同allocator/清理实现，随后仅收紧probe额度不可重新调大并补相应反控，定向`runtime_physical_quota_fragment_reuse_and_coalescing --exact`再次exit0、1/1（1.53s）。旧shim默认路径未启用新hook，兼容证据为实际机器六项；未新造AV旁路、未将runtime探针冒充source。
- 本地证据根仍`D:/autostack/.wt/lang-889/auto-lang/docs/plans/attachments/889/baseline/`：arena-controls-final.log SHA256 `0468ccb01dc3ec7def3f9a9805da95beae157ef96172aba8cf88a2854a48c54f`；arena-final.log `164e0466175954187ebc18294732ef116839cd6ee8d50f974d68a4dc768df624`；arena-monotone-limit.log `a768d884d1a57b695376ab7e7e1d532ceef84219b484a345df746e1674599d96`。raw仅本地，无add/commit；fmt/diff-check通过，新N无新增warning，既有shim mem* ABI lint四条与probe shell_lo/hi warning未被allow隐藏。
- T-02余项仍为：完整source指令/签名与类型布局身份准入、正式runtime/cache/codegen版本边界和891交接、性能负载/样本/计时冻结、进程private峰值/delta及compiler分区实际观测、剩余helper/deadline/父侧trap受检。T-04正式driver/cache/worker与T-05源码单体前端/T-06 H/SDK尚未接入，不关闭M1 gap，不把此子门当完整资源验收；无用户决策阻塞。未运行根整档t/tv/tf/docs_gen或正式性能采样。

stage: work | plan_id: PLAN-889 | plan_revision: r2 | outcome: progress（T-02实际页额度/metadata/取消反控已过，完整入口及计量冻结待办） | worktree_path: D:/autostack/.wt/lang-889/auto-lang | branch: plan-889-dev | base_commit: e0b29ff7a | code_commit: 1e240cfa2 | dependency_revisions: auto-down=c3464d0a | task_ids: T-02 | evidence: baseline/arena-controls-final.log; baseline/arena-final.log; baseline/arena-monotone-limit.log; runtime/collections/CONTRACT.md | blockers: none requiring user decision | next_responsible_role: work（关闭T-02剩余冻结/计量入口，再T-03）

stage: goal | plan_id: PLAN-889 | plan_revision: r2 | round: 5 | from: work | to: work | status: executing | outcome: pass（实际allocator/取消/边界反控新进展，work未完成） | worktree: D:/autostack/.wt/lang-889/auto-lang | commit: 1e240cfa2 | next: work

### 2026-10-30 work 第6轮：布局/父侧观测与冷态峰值反例

- 上轮为progress（已提交allocator/取消实现与反控）；本轮同树/branch重核，入树1e240cfa2、实现clean，main代码零WIP。他方树与文档保留，未建agent、未联系其他聊天；本轮仅N写入，S/H未重新取窗。实现提交`5ec76de04`，权威树clean；T-02仍有真实资源红，不勾完成，保持1/10。
- 新`collections_layout`按名义owner/name、声明序字段/臂名、传递元素/载荷形状生成BLAKE3完整256bit描述符；显式拓扑工作表，不用宿主递归。type-table重排不改身份，rename/owner/field/arm序/payload变更传播；borrow/own签名模式单独绑定，borrowed view与原值共享物理node描述符。机器构造后绑定，所有资源使用/每段字段路径/参数/返回核registry/kind/布局；clone复制binding。descriptor是共享验证后生成的常量words，helper不解引用任意输入descriptor pointer。私有runtime/call ABI升为`mn.collections.runtime/2`、`mn.collections.cell/2`，新layout/codegen/profile/recipe候选为`mn.collections.layout/1` / `mn.collections.codegen/1` / `mn.collections.source/1` / `ac/mir-native-collections-v1`；旧域身份不动，A09/A10实际绑定仍T-04待办。
- 布局3/3：含4096无环Variant在64KiB宿主栈计算、环/坏引用反控；以及原合法source对照PE42后，真实改helper单元名义record而caller不改，两unit各自verify合法但实际跨单位返回在消费前拒23+exit70，双方资源/arena/text归零。初次加强检查误把重复BorrowEnd当资源解引用，L04/FIN864B反例红；共享MIR允许已初始化token重复end、其只清token不解引用，修为遵循原语义后原源全绿，未放宽源/verify规则。
- `collections_observe::decode`真实接收PE exit+raw stderr，受STREAM_CAP约束，精确单trap已知reason与exit70、完整资源/文本/OS观测和预算受检，正常return70无reason仍Return70。原始runtime日志而非新PASS协议；正控制实际IndexTrap21/Return70，15项伪造prefix/重复/缺失/截断/错exit/未知reason/超cap/超budget负控拒绝；移除reason不会冒认index trap。曾有超budget控制替换错现值未实际变异，测试明确报红；改为从真实行取值并断言mutation已发生后绿。List/Variant参数、List结果、Place共享verify合法但本native source ABI具名拒绝，codegen在admit之后。
- OS层新增baseline/end PrivateUsage及系统历史PeakPagefileUsage（明确称process commit peak，不冒称poll已捕获所有private瞬峰）；PE delta128MiB受检，0/失败样本不得绿。8MiB guest边界实际日志：base/end557056B，peak8966144B、delta8409088B，arena_peak8392704B且退出commit/live0。PE本机栈/映像含于该OS观测，但compiler worker为独立边界；组合frame注册/native栈与执行32MiB交集仍待闭合，不用arena32MiB单独冒领完整执行受管门。
- 定向`cargo test --locked --manifest-path experimental/ac-mir-native/Cargo.toml --test collections_layout_tier --test collections_observation_tier --test collections_resources_tier --test collections_source_tier --test collections_typed_tier -- --test-threads=1` exit0：3/1/11/7/2（source33输入，7.17s，typed0.53s）。随后补borrow/own身份与typed签名/Place反控，最终`--test collections_layout_tier --test collections_observation_tier --test collections_typed_tier` exit0，3/1/3（0.55/0.63/0.63s）。变化仅布局指纹模式/具名反控，不改变原源runtime行为。fmt/diff-check通过，新N无新warning，未跑根整档t/tv/tf/docs_gen。
- 首例负载冻结到永久`collections_workload`：保留原115函数、≥256KiB算术/循环工作量，加拥有List<Variant<Item>>/Str.clone/Variant返回helper，共116函数、345483B；collections的for仅迭代/while被既有域拒绝，条件循环改用同条件与次数的loop/break，不改源产品规则或减工作。base独立mirror1786，revision n只改owned helper、预期1786+n，main与原115函数编辑间逐字保持；source hash `9fbf68139857bf473e291240df8ac48f6fc2bb493ffe4e9c2df8cc6830ba70de`。各fresh/hit/restart/body-edit/历史回退最少30样本、512真churn、P90≤2000ms及P95/P99/失败原始计量、完整父侧窗口/库准备不得移出、16/24/8MiB编译分区和128MiB worker/PE分别记，写入runtime CONTRACT，正式采样未开始。
- **资源红/诊断检查点（不缩合同）**：完整负载共享frontend→MIR→116 COFF→PE=1786已真跑。debug首例17584ms、worker delta211955712B；release首例1879ms但worker delta200810496B。再次在冻结实现上分段定位：release wall1843ms，OS历史commit峰214061056B已在frontend形成；frontend结束private23490560B，MIR29409280B，drop CheckedModule后10608640B，verify13750272B、split15474688B、emit19320832B。5ms sampler峰200986624B、delta198967296B、ticks159、child_ticks30。这是**冷态整模块前端瞬峰超128MiB**，endpoint低值/单次时延/PE余额绿不能豁免，故T-02不pass；不存在用户决策阻塞。下一步在原CheckedEnv/emit_function_body链证明有界冷态按函数生产，按S符号重新核owner/接受后实现，不复制前端、不缩负载、不借warm baseline隐藏cold红。该机制也正是T-05既有真函数前端要求，正式driver/cache/SDK仍后续任务，不以原全量前端作交付。
- 本地证据根`D:/autostack/.wt/lang-889/auto-lang/docs/plans/attachments/889/baseline/`：t02-current.log SHA256 `23f7b0391973f0a9499eb397d16b475464146916f07127e023b53f33ab886875`；t02-final-targeted.log `cc8bf7fb888f4a09552ff7bb3dc93d9e00dec5e4e0fe943d063abf58cdf3bdf5`；workload-freeze.log `fc0a6d355bf7219e58b0de57265693b5bd554738db6f9b38cb638649b95f4e10`；workload-release-memory-stages.log `51a8bca78bf08c154519e71889e2e06f51859a77f8633b7f1572145b97cce32f`。layout-first/refusal日志保留实现反例；各source/runtime目录保COFF/PE/真实stderr/receipt。raw未add/commit，未改canonical或ledger，未关闭M1 gap。

stage: work | plan_id: PLAN-889 | plan_revision: r2 | outcome: progress（布局/观测通过，完整负载揭示cold frontend128MiB红，下一动作已由证据改变） | worktree_path: D:/autostack/.wt/lang-889/auto-lang | branch: plan-889-dev | base_commit: e0b29ff7a | code_commit: 5ec76de04 | dependency_revisions: auto-down=c3464d0a | task_ids: T-02 | evidence: baseline/t02-final-targeted.log; baseline/workload-release-memory-stages.log; runtime/collections/CONTRACT.md | blockers: none requiring user decision | next_responsible_role: work（重核S窗，保留环境/单体冷生产峰值反例修复）

stage: goal | plan_id: PLAN-889 | plan_revision: r2 | round: 6 | from: work | to: work | status: executing | outcome: pass（新实现/反控与冷峰归因有进展，T-02资源门未过） | worktree: D:/autostack/.wt/lang-889/auto-lang | commit: 5ec76de04 | next: work

2026-10-30 T-02/S有界前端接受：前轮冷态frontend实际峰214061056B反例后，重核888最新26b05c7ed的提交diff，source/resolve.rs、source/adapter.rs、verify.rs、auto-mir-lower/src/hir_adapter.rs无其修改（仅lower crate Cargo.lock别项），865/863已交付、main代码零WIP。接受S最小子集：CheckedEnv新增调用同一typecheck_with的冷typecheck入口；verify_function扩Collections且复用同一collections规则的指定body检查、完整名义声明context身份（旧core/scalar指纹不改）；必要既有emit_function_body/recheck_swap的Collections缺口与其定向测试。N新冷producer只复用parse_program_with→resolve_env→同一typecheck→emit_function_body→共享verify_function→既有lower_function_collections_unit→verify_mir，逐体释放Atom/bundle，不创建第二前端/lowerer，不伪造whole credential；scaffold仍body-less、不能获whole-module credential。提交后记录精确释放；H仍未取窗。功能/资源合同不变，完整345483B/116函数负载与128MiB门不缩。

### 2026-10-30 work 第7轮检查点A：T-02完整首例/资源机制通过

- 继续`/auto-plan:work`，上轮为progress，权威树入树5ec76de04、clean；main代码零WIP；888更新至26b05c7ed，所需S源码无其提交改动，本轮S接受记录见前条。无agent/其他聊天通信。S提交`b5302262e`、N提交`30a77778c`后树clean，T-02勾选，2/10；本轮继续T-03，不表示work阶段或交付完成。
- S：CheckedEnv.typecheck_all调用同一typecheck，不再先做整模块Atom；verify_function增加Collections，选定body仍跑同一结构/类型/控制流/collections view规则，scaffold不获whole credential。新context coll1按完整名义类型/字段/臂/引用图与声明绑定，不递归、不按深度截断；旧core/scalar指纹原样。原合法单体body与whole语义相等、只plan/bind一体；view-escape、坏引用、未读record字段/variant臂接口变化、关闭/错误源码反控4/4。先前不够精准的“删len”变异只触发eval-position，改成真实ListGet→clone→len后精确view-escape，未通过放宽oracle掩盖。
- S共享lower：新递归源证明per-function Collections自调用曾把u32::MAX dummy当导入索引而panic，保红日志；改为真实self import。只收集目标body实际callee，未用接口不进入unit，改变未引用函数签名后main的实际COFF逐字相同。AV新前置7/7含真实递归/clone/View单位链接执行42，旧scalar828函数合同4/4；MIR/AV opcode/wire无新语义，旧有效单元仍合法，新单元只省去无关imports并修原拒/崩溃路径。S当前子集提交后释放，后续T-05修改按符号再核窗，不占整个S。
- N冷态producer复用parse_program_with→resolve_env→typecheck_all→既有emit_function_body→同一verify_function→lower_function_collections_unit→verify_mir/admit。一次仅一真实body的Atom/bundle/credential，保留env及MIR单位；不复制前端/lower，不把scaffold伪装whole CheckedModule。超过原4MiB input cap在任何body work前拒绝。bounded路径逐原17源（含L06精确trap21）、List<Variant>/嵌套/Option补源与跨帧资源trap、Str调用返回、RecordView调用、非首递归实际PE全部符合独立oracle；不是formal SDK/recipe已接入。
- 冷峰反例关闭：未缩`collections_workload`345483B/116函数/hash/独立1786。release全新测试进程exact首例，真实source→116 COFF→PE1786与资源父侧校验，private baseline1372160B、peak36524032B、delta35151872B，tree37277696B、ticks174/child32，≤128MiB。2017ms仅一可行性样本，不能宣称P90通过；T-06各30正式样本与512真churn、完整计时/失败计量保持。旧整模块frontend峰214061056B保留为反控，T-04必须选bounded路径，不允许silent fallback。
- 执行32MiB交集补齐：真实arena commit页+注册Frame头/stack Cell存储共用生产上限，heap-cell帧分配前保留32B头，失败退还；summary新增registered_bytes/managed_commit_peak。真实PE反控requested bytes仍容纳但32MiB页+80B帧将超限，拒于commit前；32MiB-4096页+80B成功、peak33550416B且返回余额0；还覆盖失败Nth/空帧。代码spill/原生栈slack/PE映像归单独OS process128MiB观测，容器元数据/descriptor words/文本ledger/clone work均在实际arena页内。编译侧16/24/8MiB正式分区ledger与driver/发布数据由T-04/06验，当前OS峰绿不冒充各分区证明。
- 定向affected命令`cargo test --locked --manifest-path experimental/ac-mir-native/Cargo.toml --test collections_resources_tier --test collections_source_tier --test collections_typed_tier --test collections_frontend_tier --test collections_observation_tier -- --test-threads=1` exit0，12/7/3/3/1（6.01/8.12/0.67/23.50/0.50s）；随后新增调用/自调用/未用签名/input cap控制按具名重放。source core旧864正3/负13、新S4+前置8全过；AV7与scalar828合同4全过。fmt/diff-check通过，无新增N warning（旧shell_lo/hi已知）；未跑根t/tv/tf/docs_gen，未写H/canonical/ledger或关闭M1 gap。
- 冻结合同=`runtime/collections/CONTRACT.md`：Cell16/Frame32/Node144（Win64 reprC）、runtime/call ABI2、layout/codegen/profile/recipe身份、descriptor/kind/成员校验、帧链清理与状态码、各预算分区/首例反控、样本窗口及891后继交接条件。T-02最小source+typed+runtime三起点全链可行且失败反控通过，T-03继续完整clone/set/switch/drop/故障覆盖；T-04正式cache/SDK/CLI/worker、T-05真实body编辑/候选/坐标、T-06正式资源/时延/父suite与复审/merge门均未完成。
- 本地证据根仍`D:/autostack/.wt/lang-889/auto-lang/docs/plans/attachments/889/baseline/`：bounded-cold-release-current.log SHA256 `3163cbeb9fba22aff3c178943a544d3d65dae508f0e6e463296eb20b164b0650`；t02-bounded-affected.log `78f6def7381e77b12b3feb29b4be7ada35b7c473a2195eb86b85bcf5415316b2`；bounded-source-compat.log `407ad8580bcadb13cb66fa09f27be474709a055be2044e3ea278090a04615b83`；collection-unit-final-compat.log `0142efe391e9c91109a4dca83dff5ffd75dbed8c543c8ffa3eb0cb400cd31a1b`；bounded-scalar-function-compat.log `1d0e5fd1376b4bafccdf2a9bb89fa0c857677c2ded2adeba9f4e3598e1834e39`；bounded-unused-signature-control.log `84622eb3eb8531814dd7c786a4a7d75ea917f8a90799629e3e3e4f621926563e`。raw不入库，code checkpoints b5302262e/30a77778c；dependency auto-down=c3464d0a，next owner work/T-03。

### 2026-10-30 work 第7轮检查点B：T-03克隆变异与已有owner回滚

- 在30a77778c的T-02通过基面上顺序完成T-03，未重新取S/H窗；N提交`0bbbbd691`，树clean，3/10。runtime实现已在T-02可行性迭代落地，本步补足V-03..05验收，不把前置实现重复计为新机制；仍是work阶段，继续T-04正式driver/cache/CLI/worker接入。
- 新受检typed MIR实际发CLIF/COFF/PE：List<Variant<Item>>深clone后对副本ListSet为End并remove，clone该End再VariantSwitch为全新拥有Item；guard/project/drop副本之后原List的拥有Item仍可读，两个42合为84。真实文本alloc/free3/3、CLONE id仅1次（End clone无文本buffer），可区别别名/隐式clone；全部node/requested/guest/arena/registered/text余额0。重复move后payload投影、move后的旧root clone在shared verify具名payload-moved/use-after-move拒绝、object生成0；原typed clone与错List元素/typed排除仍通过，typed档5/5（0.93s）。
- runtime-only真实PE另验：越界ListSet保持原元素/输入owner/len/bytes；正常set旧buffer恰释放一次；切换**已含owned载荷**的Variant在Nth分配失败时旧target/新输入/原bytes均保持，旧payload还能读取；重试成功后旧载荷恰释放一次，错臂/重复已move投影拒24且dst70不变，最终四literal buffer alloc/free4/4、无CLONE事件、invalid0。此探针不冒充source/typed代码生产链。
- 原L04/L05真实源补资源时点oracle：投影/临时remove-view-insert、remove转移，各恰1个文本buffer、alloc/free1/1且无CLONE；5个具名原owned首例合组仍exit0。已有T-02资源12项（含深4096/64KiB reserved栈、partial clone、metadata/页/guest/list边界、协作取消、跨caller trap及遗漏释放负控）生产代码未改变，证据复用；T-03只加这两类定向反例，未重跑整档。
- runtime/collections/CONTRACT.md补typed与runtime/source三起点分账及owner替换/活动臂语义；代码/导出/文本rlib静态库和单panic/no-CRT链接已实测，Cell/Frame/Node及combined32MiB预算不变。正式worker恢复/发布、配置、runtime hash/build依赖绑定及T-06整套正式测量仍待办；未关闭M1 gap、未写canonical/ledger，fmt/diff-check通过，无新warning。
- 本地证据根`D:/autostack/.wt/lang-889/auto-lang/docs/plans/attachments/889/baseline/`：t03-clone-mutations.log SHA256 `a2b1a795c2d835214abf876f8b8a6cf871b395de411955d09712a055a2f180ba`；t03-switch-atomic.log `28efa8c9a2308d8a5f3f424ff1662e5d1c0e6df7547168d4e785c9bba1e34a29`；t03-owned-projection-events.log `cc43095c3df7ffa1c1bba58b985e3f557812adcdb1baec3cc6ca3d1d46cab601`。命令分别`cargo test --locked --manifest-path experimental/ac-mir-native/Cargo.toml --test collections_typed_tier -- --test-threads=1`，以及resources/source档对应两个具名case的`--exact --test-threads=1`；exit均0。raw仅本地，不入库；dependency auto-down=c3464d0a。

### 2026-10-30 work 第7轮检查点C：T-04正式native驱动/恢复/CLI

- N提交`70fc03d2b`，权威树clean，4/10；T-04通过定向门，work阶段继续T-05，不是交付或final review。仍无H写入、canonical/ledger修改、根整档测试或远端push。auto-down=c3464d0a不变。
- 同一MnDriver抽取并复用A09 lookup/admit/emit；新`build_collections/request_collections`显式选择有界cold前端，完整callee签名/owner和零参int/bool入口在对象生产之前检查；无整模块frontend回退。新`build-source --source-profile collections`与原profile常量可选，legacy仍明确拒绝；五旧域的表/指纹/manifest字节身份与路径保持，新增独立集合profile/ABI/recipe身份。原始HIR build仍限原五域；typed容器clone走既有受检MIR→native专用入口，不冒领带位宽typed entry。
- 新单元指纹以BLAKE3绑定完整实际内容、名义类型/字段/臂/参数模式与import签名，忽略span/snapshot/table编号；实际codegen源码/依赖锁另入producer轴。runtime ABI2、库实际内容hash、runtime/text源码/锁、rustc身份/选项/环境与Cargo配置绑定build key；配置wrapper的可执行文件内容亦绑定。全部准备在调用内计时，匹配stamp避开cargo但重查当前内容；未认证/陈旧派生库先删确切输出再实际build，不用mtime将未知库盖章。override同尺寸改一byte仍具名拒绝。新profile每次重hash linker/kernel32，不借旧path/len memo冒称当前身份。
- 新A08/A10集合清单分页持久化runtime+MIR+COFF完整required闭包；不序列化verified credential。restore先比schema/profile/ABI/target/codegen/runtime/tool，再经共享`verify_abc_bytes`重入MIR深验、重新计算全部COFF元数据与布局、重验COFF内容和实际continuation页、完整callee/entry，随后真实link/run。初次直接decode_abc的传输owner schema与MIR schema不同，实测拒绝后改用既有共享wire准入，不放宽MIR schema检查。新by-id页读取先查询chunk地址再调用现有bounded/CAS-gated读，防超页先分配；集合COFF给头预留4096B，单payload≤256KiB，旧五域分页格式不改。
- 新进程`mn-worker collections-restore-run`结构上不做unit codegen，独立协议名`mncollections-restore/1`，实际新进程结果42、2单位重验、unit emit0。reuse/fresh/read-only同一缓存机制；只读恢复与source缺失重算都实际执行，block数/roots不变。命中时补齐后续COFF页required引用，三页真实可链接COFF控制从cache命中→manifest→restart再执行42，漏声明页/超页受拒。23项伪造schema/profile/ABI/target/producer/owner/entry/runtime/选项/MIR/COFF/页/tool控制全部在link/codegen前拒绝，旧PE与root保持。
- 新候选manifest先非root落盘，独立目录link并按runtime观测执行，成功后才root/publish；index trap21仍是合法语言结果，普通return70不算trap。预算22/无效值23/注入25/取消26分别具名分类、失败不发布。旧PE/receipt/root在语法、类型、入口、实际MAX_LIST+1资源失败及收据目的目录占用时保持；新增实故障缝在新PE已放置后令第二placement失败，原两个文件逐字恢复，无旧文件时两个新目的路径均消失。重启后从本profile清单重建当前发布集合，成功后释放旧根，保持旧五域根独立。
- 新driver档7/7，23.75s，覆盖原L01..L07/V01..V07、LSTR864/FIN864A/B、List<Variant>/嵌套/Option、Variant helper调用返回、CLI/native产物、跨进程恢复、只读缺失、发布资源失败与负控。旧cache tier/CLI871/first_case分别全过（旧五域真build/run、legacy拒绝）；新跨单元错签名/entry歧义1/1，发布中途回滚1/1，旧backend控制6/6；未引用函数签名改变后main实际COFF、单元指纹及完整meta相等。曾两项新测试fixture误用match语法/附加非法COFF尾部，分别改成既有is语法/合法扩充COFF字符串表；原红保留，不改生产语义绕过。fmt/diff-check通过，新N零warning（AV shell_lo/hi baseline不变）。
- 本地证据根`D:/autostack/.wt/lang-889/auto-lang/docs/plans/attachments/889/baseline/`：t04-driver-final.log SHA256 `19e8984d85bda00a8eaa20d683c089e87860d2db3fb6f225d03e3d3e9a8a35d2`；t04-driver-compat.log `0e7cee84450ca4dadc4628e2bf4c6232c51c84dab7a57c975b7a5486dae6b2f2`；t04-publication-rollback.log `a4d35e9d31bb7bbc7175468acfc78fbcfecb489a91f6ac08cf00cc4d0a56bffa`；t04-closure-admission.log `665074b86ecc9d333cf8e6c43892ebb6204117e625278b2724faf6734a7e3556`；t04-unit-binding-control.log `31ce542d643129ee489bf21d6c03d86de93a8cd20a3bedeba1e4a89a46990902`。命令为manifest experimental/ac-mir-native/Cargo.toml下对应`--test collections_driver_tier`或`--test cli_871 --test cache_tier --test first_case`，lib控制按模块过滤；exit均0且非零选择。raw仅本地，正式H/SDK与编译分区/30×各场景P90/512 churn仍归T-06，真实单体源候选/坐标归T-05。

2026-10-30 T-05/S窗重新接受：main代码零WIP，888已更新至a436985c5，新增提交与WIP仍不触及ac-core/source、verify_function、共享hir_adapter及集合session；890 afea065b2亦无这些S改动。前S子集已在b5302262e提交释放，本次仅接受`ac-core/src/source/{resolve,adapter,parser,mod,ast}.rs`中CheckedEnv/recheck_swap/FnCandidate/prepare_candidate_document及必要单函数fragment解析、坐标与容量观察接口，`auto-mir-lower/src/hir_adapter.rs::lower_function_collections_unit`必要缺口和新定向测试；N消费者本Plan保有。优先扩同一828生产接口，不复制frontend/lower，不写现有AV executor/worker或H、不消费他方WIP。新intern List/Option等body局部类型池需与body一起候选提交/恢复，不能漏池引用后以whole回退隐藏；纯body无变化函数必须实际各生产阶段0工作并保对象身份。接口/声明编辑走显式受检失效路径；提交后记录精确S释放。无用户决策阻塞，不请求重复执行许可。

### 2026-10-30 work 第7轮检查点D：T-05源候选类型池与单体解析前置

- S提交`3d8a3d0b3`后树clean；T-05仍未勾，4/10，继续N真实单体消费者。新反例：完整frontend接受helper body从List<int>编辑为List<Str>并引入?int，保留环境prepare却在原Str push处报expected i32/found Str；原红说明新body引用了旧类型池，不是源规则限制。修复`recheck_swap`只在Collections域把body+新intern类型池作为移动状态临时graft/check，prepare还原完整current、候选持有完整新状态；commit同owner/base/slot验证后一起安装，cancel/drop/检查失败均不写当前。旧core/scalar/record/closure的池发布行为不扩面，close显式释放本体类型池。
- 同一Parser新增`parse_function_fragment`：仅将授权UTF-8字节range交给原lexer/parser，token span直接加绝对offset，EOF锚定fragment边界；不解析/词法重扫前缀或未改函数、不建第二语法实现。producer侧独立SyntaxWork观察实际tokenize range和body parser入口，828原四字段BodyWork接口保持。CheckedEnv公开冷函数区间与实际body statement allocation身份供消费者核保留；FnCandidate可只validate后再在native事务成功点commit，原commit仍重验。
- 新源候选3/3：新List/Option类型与body提交/取消一致，失败/foreign/stale/close均保旧；native未接入前不冒领全链。fragment在故意非法前缀和UTF-8注释之后仍只解析helper，实际range/name/错误span绝对、边界合法；main原producer allocation在候选提交后不变。此前Collections单体/whole等价及view/context控制4/4，旧828 scalar候选事务8/8（prepare/cancel/多候选stale/public recheck/failed recheck/foreign/post-verify/close）通过；scoped check/fmt/diff检查通过。未跑整档/未写H。
- 精确S本次完成子集释放：source/{resolve,adapter,parser,mod}.rs上述新接口/事务与新collections_889_candidates测试，代码3d8a3d0b3；后续必要S若改动再核窗，不占所有source/AV session。证据本地baseline/t05-new-types-before.log SHA256 `ab3cf24b8619fdc02ee4ef8f262e837263abfaf273b8b265f36edcd07131317c`；t05-source-candidate-controls.log `bf143b6f0f7f4e620634b5e0ae1705e49c3390bde0577088b8d8f6803f88a13a`；t05-scalar-transaction-compat.log `0ab8bf475d780728195021b321b08806a2dce9cc1152b8413134f51e00ae2600`。对应manifest ac-core的两个具名test文件与ac-av-probe/probe的function_incremental_828_contract/contract_828_transaction_过滤，exit0非零选择。raw不入库，依赖版本/合同门不变。

2026-10-30 T-05/S坐标最小窗重新接受：N真实单体probe已通过42→43→42及main源/MIR/COFF真实对象保留，但连续UTF-8变长编辑在scoped emitter的历史Option声明name_span报source.internal边界错误（baseline/t05-native-incremental-controls.log保红）。重核888=5ffb4eb3a且其与890现有WIP均无source/adapter.rs修改，本Plan顺序接受`Emitter::render`中**Collections scoped** nominal record/variant声明元数据锚点与对应新定向回归；不动其他域/原解析语义，不更新未改函数体。此修复是既有坐标/候选合同内必要动作，T-05仍pending。

2026-10-30 T-05/S坐标子修复交还：提交`2e18c8d62`仅adapter scoped Collections nominal声明span与具名回归；其他域不变，N会话仍有未提交实现继续本步。新源4/4、原单体4/4；再加真实record+enum声明的具名控制1/1；N当前真实首例与4项增量控制通过（7.79s），包括UTF-8变长/回退、source/MIR/COFF main身份、候选取消/坏链接/stale/foreign、声明臂序变化只重产make/main而保untouched MIR/code、实际whole frontend/whole lower/同内容新code对象三种退化反控报红。T-05尚缺完整持久化坐标/恢复、资源/截止与完整采样消费者，未勾。证据baseline/t05-source-coordinates.log SHA256 `c07987e5e09e77d6352a056596b78ca58f8cc29e4c6e605fddd1fd50a26ae7bc`，t05-source-nominal-anchor-control.log `0355643f9e8602cdd1e5e46d5e3a60eed41b58875ef560ff1762dfadfd61a05d`，当前t05-native-incremental-controls.log `487f29c15a587ffd6811d329e82df8e57ff6558edf5f23539abe426723a9b0d9`。首次UTF-8红见本轮原始工具返回（source.internal: Span 71..75 not UTF-8 boundaries）；同名native控制日志已被修后重放覆盖，不能把该文件称为原红文件。S最小窗再次释放，继续N；raw不入库。

2026-10-30 T-05/S借用验证最小窗接受：完整116函数resident probe现冷态2.8s单样本（仍非P90通过），计时揭示函数验证调用为保留同一bundle而先clone整个声明目录。N已用不可变进程内MIR准入消除重复深验并逐字COFF/fp/meta等价；持久化仍从wire重验。重新核888/890无verify.rs WIP，顺序接受`ac-core/src/verify.rs::verify_function`新增只读borrow入口及同一实现委托、相同错误/credential控制；只避免调用方复制bundle，不缓存/跳过验证、不改旧四字段BodyWork/旧域身份或验证规则。N冷态/单体消费者改调用借用入口；本最小S窗完成即提交释放，预算/样本/门不变。

2026-10-30 T-05/S借用入口交还：`6de639fa9`只新增verify_function_ref并让旧owning入口委托同一原验证体；返回仍是owned CheckedFunction，无规则/身份/BodyWork变更。新集合5/5（两入口credential/body/context等价、当前上下文变化与坏owner相同拒绝）、原scalar828事务8/8通过；N消费者的调用切换仍在本Plan N WIP内。证据baseline/t05-borrowed-verifier-controls.log SHA256 `fece1e7b6b736d726becf713ecc153c26b49696852cd84d342fc1bc46e463939`，t05-borrowed-verifier-scalar-compat.log `dc4267d2ebaa4ac9be17ec5d4212b8cdbc184afae46e694ecc855f855184e141`。S最小窗释放；T-05/T-06及正式P90/物理分区门仍pending。

## 10. 待澄清事项

- 支持面已选864完整合法源码与其MIR闭包，不留“是否包含List<Str>/拥有record/Variant”的开放裁定。T-02选择等价native表示/allocator/helper机制；若预算内无法实现，给最小反例与方案needs_replan，不自行缩范围或加语言深度限制。
- 865/863/887已交付；T-02/S开写接受与T-05后续S、T-06/888 H逐符号交接、SDK路径、runtime支持库兼容与trap有界观测仍归相应任务核验。提前准备不取得公共H写权，不自动联系其他会话；当前不阻塞独立T-02/S/N。
- typed entry另立891仅规划，其范围/结果通道由该计划冻结；本包保持W01–05等拒绝/原gap，不因它未实施阻塞本包交付，不自动启动或修改其合同。


2026-10-30 T-05/N实际交付 `43ae3d854`（S前置 `3d8a3d0b3/2e18c8d62/6de639fa9`）：
- `CollectionsIncrementalSession` 使用同一保留CheckedEnv、fragment parser、828候选池事务、borrowed function verifier与共享单函数lower；body 42→43→42、连续UTF8变长、声明臂顺序失效均经真实PE验证。未改115函数实际source-body/MIR/COFF Rc地址保持，实际parse/resolve/typecheck/plan/bind/lower工作0；历史仅COFF命中，目标MIR/坐标重新生产。显式声明/config失效道如实全前端，仅不兼容native身份重产。
- 源/环境/坐标/manifest/root/旧PE/receipt发布事务覆盖取消、foreign/stale、错源、真实坏链接、执行Resource22与5s外部截止；失败原始PE字节保留，外部kill不冒充清理成功。实际whole frontend、whole lower与同字节新COFF对象三反控均红。恢复manifest /3绑定source页和每单元mint/current坐标，重新ABC/native准入，不序列化credential。
- 当前不可变MIR的私有live admission凭证消除同请求重复深验；等价测试验证COFF/fp/meta逐字节不变，裸不可信restore仍重验。cold MIR共享有界256KiB carrier页，独立单元ABC精确hash、边界/跨页/缺页/错offset/换页反控9类，required闭包去重；纯body新单元独立落盘，原单元producer不重建。旧五profile collector分支、key/ABI未变；8项old cache兼容通过。
- 定向命令均 `cargo test --release --manifest-path experimental/ac-mir-native/Cargo.toml`：`--test collections_incremental_tier -- --test-threads=1 --nocapture` 6/6；`--test collections_driver_tier` 7/7；`--lib packed_mir_tests` 1/1（9反控）；`--lib checked_unit_tests` 2/2；`--test cache_tier` 8/8；必要check与所改文件rustfmt/diff-check exit0。仅AV既有shell_lo/shell_hi dead_code诊断，N无新增warning。未跑根整档。
- 冻结116函数/345483B原负载probe：cold1991ms、body558/430ms；worker private baseline2342912、peak29978624、delta27635712B；named ledger峰mir4073520/staging2121404/io5581512，总11498891B。**这是单次机制probe，不是正式P90、完整编译分区物理归因或30×5/512churn验收**，这些仍归T-06。首次release因OS112磁盘满失败，raw保留；空间恢复后重编通过，未删除缓存（删除操作被自动审批拒绝 blocked by policy，工具未给具体理由；已改安全继续路径）。
- raw根为权威树 `docs/plans/attachments/889/baseline/`，本地不入库：t05-packed-incremental.log SHA256 a949d5ee3eb5ee9eaa5949eb260730637ad24ae2415e356063f8c72a28fccff5；t05-packed-controls.log c489ca1fc42f4ed89deddf1bb6fdc3a0049b65627618cbbe2adec42ce640d411；t05-final-driver-replay.log af3b828e51ec09ae2b363a444ca2b804c221f524a241073ddc72a21e7123dde7；t05-final-live-admission.log 6ee70cb629d3a0726aadd6aebcc153aa061df5ca86847a89657f3cbabf54f319。T-05窗口实施已提交，S无继续写权；N在T-06继续自有消费者接线，未最终review/merge。

2026-10-30 T-06/H最小无重叠接受：重核888权威树HEAD `9b25d91a7`、tracked/untracked代码clean，实际diff从889基面只涉及ac_av父侧/批量工具、inventory/execution_trace及mod.rs的run_suite_selected包装；不涉及ac_mn.rs、contract.rs、ace_m1_tests.rs，mod.rs的MN recipe dispatch符号亦无改动。按§5.1实际逐文件/符号差额接受 `ac_mn.rs` 新recipe参数化/新worker入口/父侧新观察准入、`contract.rs`新recipe及mn binding、`mod.rs`仅MN dispatch分支、`ace_m1_tests.rs`新增889具名SDK验收与原binding新suite；不取888的run_suite_selected、ac_av、inventory/trace/helper批量符号。旧五域/旧MN recipe及协议默认保持。根不引入实验依赖、无新runner/filter守卫。前owner已交付865/871为基面祖先；888所改符号不在本次接受集合，双方未提交WIP不互相消费。H尚未实现/释放，本记录不构成验收。


2026-10-30 T-06/H消费链追加实际查缺接受：`test_cases/stale.rs`当前按已知probe族选择现行toolchain map，尚无MN集合族，若仅注册新recipe则集合报告会误走sema身份。重核888实际diff该文件无改动，主检出无代码WIP；接受仅新collections recipe的current身份分支及独立惰性槽，原scalar/AV/sema分支不改。`ac_mn.rs`新current_collections_toolchain_identity使用同一受控worker的显式身份查询取得实际runtime内容/build身份；报告回填同一次真实运行的已验证runtime轴，不在正常case计时窗外预备库。此发现属于T-06既定正式消费/cache身份闭环，不新增根实验栈依赖、外仓代码或新任务。


2026-10-30 T-06/H+N功能检查点 `cb5ce40db`（T-06整体未关闭，仍5/10，goal round7 work继续）：
- 新 `ac/mir-native-collections-v1` recipe沿既有containment/probe transport及父侧oracle，使用独立 `mn-collections-worker`；共享MIR为唯一native语义源，旧MN recipe/default五域及Root实验依赖不变。正式 `native-collections-889.suite.ad` 包含原17源native、原17源AV及3补例×2共40检查，源字节/独立oracle保持；binding14格尚未提前翻绿。
- Source SDK40检查与runtime/report identity当前性反控通过。新native报告9轴绑定recipe/shared lowering/worker实际SHA及runtime ABI/build/content、lld/SDK/kernel32；stale对集合族独立重算。保持同长度的runtime库bitflip使报告STALE，字节恢复后Fresh。父侧独立解析实际stderr/退出/资源，18种坏raw与5种坏单体记录反控红；身份查询签名字段位于JSON中间的真实请求通过，篡改hash拒绝。
- 实际机器漏free控制：仅ephemeral driver省略frame_leave，真实PE仍42，registered_bytes非0；父侧报owned resource leak，不能用正常exit假绿。持久driver拒绝该故意坏代码control，codegen0/roots0，后续正常请求成功。实际whole frontend/whole lower/同字节新COFF对象控制经SDK报Error，含真实parsed/body/counter证据。首次whole-lower控制断言误写d_main（实际观察键main），只修测试断言并定向复跑3反控通过，原红日志保留。
- 增量准备与提交间再检查runtime/configuration。准备后同长度篡改静态库，提交在link前拒绝，旧source/env/实际producer身份/root/manifest/PE/receipt保持；旧PE在库仍坏时运行42；恢复库后目标43命中此前合法候选COFF并成功。外部runtime变更不绕过候选原子性。
- 专用worker采用全Rust allocator backing-request计量（含header/对齐，释放按原partition），MIR/IO显式分区，其余前端/CLIF/store暂存归Staging；OS private commit另计堆余量/C侧存储引擎。真实边界/重分配/4096B对齐/跨scope释放/panic scope恢复控制通过。初次冻结负载实测Staging25981082B >24MiB在发布前拒绝；消除签名校验后多余unsigned/base64副本保活后，同一345483B/116函数输入真实峰Mir10898823/Staging25058113/IO6111533B、总25058113B，worker物理delta30314496B，PE1786；单样本parent1991ms。**不是正式P90**。3次本地协议探针因签名对象Map::remove的保序交换拒绝，修为保序过滤重建、永久真实query正/反控覆盖，原raw保留不冒作语言/性能通过样本。
- 验证实跑（全部定向，不跑根整档）：Root `cargo test -p auto-lang --features test-cases --lib ace_m1_native_collections_889_ -- --test-threads=1 --nocapture`分项source40/实际漏free/编辑+restart/stale/wrong oracle+type均通过；其中原聚合有上述反控断言红，修后仅 `...real_whole_work_and_code_copy...`通过3实际反控。添加prepared runtime guard后仅重放 `...incremental_rounds_and_restore_sdk`通过。Core `cargo test -p auto-lang-core --features test-cases --lib native_collections_889_` 2/2（18+5 codec controls及真实身份query）。Native `--bin mn-collections-worker allocation_controls` 1/1，`--test collections_incremental_tier collections_prepared_runtime_drift` 1/1，`--test collections_driver_tier collections_missing_free_sensitivity_cannot_poison_persistent_cache`1/1。旧 `mir_native_adapter_old_protocol_and_mixed_compat` 1/1。测试源assets新增N889Load严格等于T-02冻结generator，SHA256 df314cc77208bcc115f02b446b8c1012046e96b046784a1a0c6e2e2094a1715c；单文件345483B，不是本地证据入库。
- 根check初次缺组内UI Cargo依赖；补只读detached `D:/autostack/.wt/lang-889/auto-ui` pin `3d6e1f15c1142aa797734edc26dca561511b72c3`，代码零修改。Root source SDK首次以--no-default-features编译被既有未cfg UI测试26错误阻断，改正常默认features通过；保留first/default两个日志。Core normal/check-feature诊断147/149，Core libtest169、语言libtest43均落既有非本包源码；新H/N无新增warning，未自动改他方模块。fmt/diff与新PS脚本AST parse exit0。
- `scripts/verify-native-collections-889.ps1`提供source/resources/incremental/compat/formal/acceptance定向入口；acceptance包含显式formal门。SD-01..05待复审草案 `attachments/889/spec-delta-r2.md` 已提交，不写canonical/ledger，不宣称M1全绿/891实施。N/H功能源码现冻结，正式campaign仍pending，final review和merge-owned门未执行。
- formal observer沿同一parent transport/真实worker与独立oracle，30 samples EACH fresh/hit/restart/body/historical，512不同真实owned-helper修订，不减源/分母/阈值；raw逐attempt写JSONL。fresh各store最初不存在、helper路径hint在同组父目录共享，首次准备计时且实际工具/库内容每请求重hash。固定P90 one-based N-floor(N/10)≤2000ms、P95/P99报告；600s长会话有界，不改每请求deadline/2s门。当前888正在连续运行reference资源/计时模块，先不竞争正式采样；已异步询问用户是否允许向“完成计划888”协调约8分钟窗口及完成通知，未获答前不跨聊发消息。不因此标目标完成或阻塞，也未取消其进程。
- raw本地根 `<authoritative>/docs/plans/attachments/889/baseline/`，未暂存：t06-sdk-dependency-and-controls.log d3aafd777e222c69e1c84d5a9fd3d893d5b96a8cb2b47583f0d20df2ea0924aa（上述分项/断言红分账）；t06-sdk-real-degeneration-fixed.log 22cdf8478e394d8c6a82ab7e82f3c23459f33eb171bb7ef962943c63c2af4d16；t06-sdk-prepared-guard-and-restore.log 7bbe41d0f6b4157c14634dedb5a461e970109edc59c0f2b90edfcd8fbfc4675a；t06-parent-final-controls.log a1c0dfef1d39e84cde7d7a9f6a15b7b76ab3258249af6a14667b84384734afc2；t06-allocation-boundaries.log aa90e7240faf4c382072688c6d040a9963a0e8bd0ecebcadcf00eb942d0d8903；t06-prepared-runtime-drift.log 5d37ffbc0f4dc8a0c74611c5cb61404aaa7fcf61519c06a8d6f17b46d9051452；t06-sensitivity-persistence-control.log b3d1314c923aa437d339b18d75bf02e90ec2a8cfef4c7da29b71284054f60b90；t06-old-recipe-protocol-compat.log 80e0548652ed7017e61095ce0d1e00845151f2d21d38467bc28d414936f65b4f。


2026-10-30 T-06正式首轮（冻结bd81551ef，exit101，严格保留failed门）：实际空闲窗口出现后未跨聊发消息，沿已授权goal执行；异步协调问题尚未获答，不视沉默为授权。`powershell -NoProfile -File scripts/verify-native-collections-889.ps1 -Scope formal`完整运行464.12s，五路径各30样本+512不同真实owned-helper churn全部语义/资源/identity/close验证通过，invalid/failed-sample0；整体**未通过**，只有fresh P90=2185.8468ms>2000。其他P90：hit558.196、body561.539、history565.8193、restart1818.7989ms；P95/P99完整原样在campaign.json，未去掉慢样本/改门/减少源或次数。close实测MirSymbol current=0。当前T-06/AC-04仍未关闭、不送final review。
- raw目录 `<authoritative>/docs/plans/attachments/889/baseline/formal-campaign-71640-1793323321036960900`，campaign.json SHA256 1dd6280e42faa8646445aabad77fcf250259cff5d6b4b51701a853a0f45234c9；samples.jsonl d7e58099f2691556a553fdea4233e27e5510981b882aba653133bc8ea0102e48；首轮log 6cbd4f675298a2b74f254374bc17c30a4aa0599427842d24a35c0f4de7bd787b。全部本地保留，不提交raw。
- 原因定位：fresh median parent1928.24/worker1821ms，parent准备/IPC/admit约104.93ms；worker median frontend约690ms、codegen约317ms、COFF persist约180ms。尾部慢样本多个阶段同时变慢，但不凭“环境噪声”删样本或判绿；修可重复的cold前端开销。独立源/旧协议检查已在cb5ce40db冻结，14个原L/V native绑定及N889Load精确LF属性以bd81551ef提交在权威树，尚未合主检出。
- 接受S有界优化最小写面：`ac-core/src/source/adapter.rs`的同一Emitter选择/绑定逻辑、`source/mod.rs`导出与新collections batch等价测试。重核888该两文件实际diff为空，前S owner=889已释放6de639fa9，现在由889 T-06重新接受。仅cold消费一次绑定最多4个真实函数，源body字节工作集上限16KiB，超量单体走原道；所有函数仍真实resolve/typecheck/plan/bind/lower/verify/codegen，不改纯body单体API、Source支持/拒绝域、opcode/ABI/门槛。冷态允许全源，分批复用相同类型/签名scaffold是实施优化，不修改r2的G/AC/预算/116fn/345483B/30×5/512合同。
- 修复先验证batch与原single-path逐函数MIR/COFF/fp/坐标等价、真实每体工作1及大body自适应单体控制，必要check；然后只重测受影响fresh门，其他四路径与512基线按实际代码/依赖/输入不变性及等价证据决定复用，避免无由重复整套campaign。若等价证明不成立，扩大受影响路径重验；不得复用被改动破坏的证据。


2026-10-30 T-06 cold修复冻结 `1eeff4832`：同一Emitter/binder每批最多4函数、cold消费者按16KiB源body总量选择批，超量仍原single。冻结116函数+17原源+3补例+Str返回/递归组合，逐函数完整ABC（含坐标）/unit fp/真实COFF字节等价；逆序indices亦一致，真实plan/bind各1；大body单体与空/重复/越界/closed/非Collections拒绝控制通过。首个测试源码未标注Str，被原拒绝域str拒绝，仅把新测试fixture改为合法Str，生产支持域不变；初次测试直接比较无PartialEq的MirModule未编译，改完整wire字节比较，原日志保留。源候选/单体frontend9/9、真实incremental7/7通过，纯body生产/保对象和rollback不改。
- 本地baseline日志：t06-cold-batch-equivalence-3.log SHA256 ef6e82dc7f8484ddeaa8deb7b2ee4a8653281d2fb965a6fa1d3f318862478dcb（equivalence通过，含上述fixture红）；t06-cold-batch-boundary-4.log 1f7de15742cbfad9ebd5513d32348da3c35970fd53837c5c86df3dec59a0293f（fixture修正后边界通过）；t06-batch-source-controls.log e9140cb2b13539eb9e7149d47e9210ff1fed73846a02253513c8f02d17132653；t06-batch-incremental.log dd9ffb8afefa9730b1dbb55e5712894098ad008bae50ef8c3be35229fbca2ac3。fmt/diff-check通过，新代码无新增warning，S本次子集提交释放。
- 用户已明确答复允许协调888计时窗口，已发送一次协调请求，未取消他方命令。虽然产物逐字等价，但cold批改变临时分配与resident初始物理状态；因此扩大必要复验至完整五路径/512，检查新冻结worker的allocator峰/close与正式P90，不复用旧峰值。第一轮失败保留，阈值/样本/源不变。


2026-10-30 T-06第二轮冻结1eeff4832：完整547.09s，exit101；五路径各30/512churn/close语义与资源全部通过，无invalid样本，fresh P90=2535.9908ms、restart=2009.321ms未达2000，其余hit929.786/body687.2667/history650.2525ms通过。不能拿首轮restart绿覆盖当前红。raw本地formal-campaign-54960-1793325000976026800：campaign.json SHA256 ac72320bb823067d8dead129360b3520131fac12c1ba23443469d4683eae43af；samples.jsonl f8fd8ef41a59017de682081aee32fc639f94172b9650a38b68380c2f77510bee。修复没有被本次性能门证明充分；完整保留失败。888测量窗已实际释放并通知恢复，后续若独占计时再协调。
- 定向release阶段probe（非正式样本、不判P90）确认parse10ms/resolve10ms/typecheck1ms，emit208ms、function verify38ms、shared lower36ms、native admission245ms，其中同一shared verify_mir229ms/ownership233ms、layout描述符0.2ms。旧115函数中大多数271槽/153基本块；即使全int/bool，每块所有权状态仍分配n个借用根/字段移动空header，构造/复制是当前必要优化点。探针源和输出保留本地，不入库。
- 接受S最小修复窗：`ac-av-probe/core/src/ownership.rs`只优化**全部槽均I32/Bool**函数的恒空token_roots/moved_fields存储，仍执行相同ownership数据流、所有init/move/drop/CFG/错误门及原steps/analysis预算，其他类型不变；不跳过验证、不用profile标记推测已验证，不改opcode/wire/AV执行语义。重核888该文件从基面diff及WIP均为空。需旧dense表示与优化表示逐结果/错误/steps/analysis等价控制、共享资源/owned具名兄弟与Native增量/源控制；保留旧全header强制模式仅私有测试，不开放API或持久credential。若不能保持一致则撤回该实现。修复后只重测受影响fresh/restart门；已两次闭合的512语义不再无由全重放，初始/回落由实际受影响新worker定向代表复验，精确记录复用边界。


2026-10-30 T-06修复冻结185fbeec2：S ownership只在全部slot均I32/Bool时省恒空两张侧表header，转移/worklist及上游typed门不跳过；保原保守analysis bound/steps，资源/记录/借用槽继续原n表。私有dense对照768组（4/31/64/271槽×32个diamond/backedge/move/drop/未初始化形态×scalar/Str/StrView×正常/steps边界），结果+完整诊断+steps+analysis值逐项一致。真实AV owned25/collections59通过、Native incremental7/typed5通过；release诊断old ownership233ms→new23ms（非正式P90）。首个局部fmt误用root --all，触发既有missing mem_profile且顺带格式化100无关文件；已逐path恢复这100项、只保4个自有文件，最终精确rustfmt --check/diff-check通过。诊断example源已移动本地attachments，未暂存。S提交释放，无AV语义/opcode/wire变更。
- 新正式repair入口明确scope=cold-verifier-repair：fresh/restart各30，仍2s/N-floor(N/10)原门；另各3个hit/body/history以及真实close/current MirSymbol0。原full入口仍五条各30+512，不减它的门。该repair不自称full矩阵或512通过，正式完整证据两轮原样保留。新变更只影响纯scalar侧表物理存储，768等价与AV+native兄弟控制证明规则/传递结果不变，owned-helper的全部slot非全scalar、每次body/churn走原表实现；其他三个路径生产/transport/link/runtime/缓存/源输入没有改变，因此复用1eeff4832第二轮各30的hit/body/history与512实测，必须在最终receipt显式绑定两次提交/worker SHA，fresh/restart和新初始/close实际峰值以新repair重测覆盖，绝不复用第二轮failed fresh/restart。
- scoped命令：core --lib scalar_side_tables_889；probe --test av_owned_821_semantics --test av_collections_822_semantics；native --test collections_incremental_tier --test collections_typed_tier；root新ignored具名test --list确认非零选择及编译通过。日志SHA256如下（均本地baseline，exit0）：
  - t06-sparse-dense-controls.log: 0529bd02c11371ad8a64e5e1bc53d6721f3d48557a122b62517e59c2af874ad7
  - t06-sparse-shared-semantics.log: b8ffe851fd3cee8388bd51af1f760df66b844a5e6a93794268b62e920a8d6501
  - t06-sparse-native-controls.log: 51abb7afbaea634724c54abd1490e9c6ef632d4ed78f7fff8da5beddea4732d2
  - t06-phase-probe-sparse.log: eb8b7104b6191a0a8c1d07c4bdee2d073877caffd1e47c604741087fad6d4e03
  - t06-cold-repair-harness-build.log: c78bbdb48810a7d5a8d5bd71df937de684cbe6e778033fbe750da43593bee3a5
- 已向888协调约2分钟新的受影响计时窗口，尚未开测；不把等待当用户阻塞。T-06仍未勾，5/10，继续work。


### 2026-10-30 work 第7轮完成：6/10，转final review

stage: work | plan_id: PLAN-889 | plan_revision: 2 | outcome: pass（work-owned完成，不是final review/交付） | worktree_path: D:/autostack/.wt/lang-889/auto-lang | branch: plan-889-dev | implementation_commit: e188033c4e7866d19bc4e152937e6e1052b8ac69 | code_freeze:185fbeec2（后85c422c08/e188033c4仅合同/SD文档） | base_commit: e0b29ff7a0afd2479904b304780d6818306b54e1 | dependencies:auto-down=c3464d0a0d36e4f8a7988b0c0c26448fc52361b4; auto-ui=3d6e1f15c1142aa797734edc26dca561511b72c3（均组内只读detached，未改代码） | next: review(final)。本goal授权沿用全链，不请求重复批准，未启动891或远端push。

- AC-01/05：原17源+3补源×AV/native正式SDK40检查、父侧独立oracle、原源SHA不变与14L/V binding真运行证据见cb5ce40db/bd81551ef；source/typed/runtime入口分账，L04/L05=42/L06=专属index21，错oracle/错元素/正常return70不冒绿。新增source assets是正式测试输入，不是入库raw证据。
- AC-02：T-02/03资源/深度/Clone机器链及runtime负控已闭合，T-06漏free真PE父侧红、allocator边界、prepared runtime漂移/回滚控制闭合。当前共享ownership空侧表修复只改变全scalar的确定空存储，768dense对照与AV owned25/collections59、native incremental7/typed5实际通过；同一helper和既有拥有值/临时view/部分失败清理实现未变，不重跑无关深度/runtime档。
- AC-03：真实fragment/新类型池/UTF8坐标/failed/cancel/stale/foreign候选、source/MIR/COFF producer对象身份、声明失效、restart准入和实际whole-front/lower/copy反控已验证。新实现7个Native增量测试再过，非全scalar目标helper仍原所有权路径。
- AC-04：新冻结代码正式repair `cargo test -p auto-lang --features test-cases --lib ace_m1_native_collections_889_formal_cold_verifier_repair -- --test-threads=1 --nocapture --ignored`，exit0，89.35s，fresh30 P90/P95/P99=1829.868/2054.9538/2137.8745ms，restart30=1097.8078/1118.0927/1150.8019ms；原2000ms P90门不改，P95/99慢样本完整保留。另hit/body/history各3代表、新worker实际close通过，current MirSymbol=0，worker current5066752B。新compiler峰Mir5758920/Staging25058513/IO6112050B、combined26200688B，worker private delta34222080B，全部原门内。
- 正式矩阵复用：1eeff4832 full实际hit/body/history各30的P90=929.786/687.2667/650.2525ms、P95=1198.163/771.3943/719.0724ms、P99=1254.9996/941.8662/888.5897ms；512个不同1000..1511 revision和关闭全过，旧worker SHA dd4bfecbde2f1169788b26f82fbbc6ba57987d6b9192ef89c589b26dfabba55e、worker delta37691392B。修复只省全scalar侧表，owned-helper及三路径生产/源/依赖/配置/输入不变，768语义/错误/steps/bound等价、新代表往返/close与Native7反控支持复用。**该full的fresh/restart仍保原failed，不复用其失败门或将full整体改绿**；新repair明记churn0，仅替换受影响两门，不声称当次重跑512。codegen/runtime/build/content/link轴三轮完全相同，新worker实际SHA 3ee1a0102ead365b0560a89d51bf15781c90441f075372eefcb401cad215b291。
- AC-06：S/N/H每次具体子窗已提交释放，修复新增S ownership窗以185fbeec2释放；旧scalar协议混合控制已通过、旧五域代码路径/布局/身份保持；根Cargo依赖与AV opcode/wire/executor零diff，唯一共享lower仍在，legacy未恢复。N891只读接口清单已入冻结SD草案，N正式释放待889 T-09。SD-01..05六目标/四模块草案含现行Spec SHA、实际source/typed限制/资源/候选/恢复/SDK/性能及归属，canonical与live ledger尚未写。
- 本地正式repair目录baseline/formal-cold-repair-50504-1793326445916044600：campaign.json SHA256 5e1fb68d2241727b6bed5d3efab17980d7f19504fc47a37d7ecdb3935e97be86；samples.jsonl c866b89fa259c1d5c93037d6c7b52a2ff02cc968278b42e7993990c3228fe582；log SHA256 17eb7e9c0425ad1ffe4dcdc8f0952dc9631404bda3297a41681b1745ace02508。第二轮full目录/哈希及首轮failed均前记保留。新SD草案SHA256 4c8f12234cd52093f401fb1e3692e6050d14d9247bb746d617f4f706a2449eaa。raw全本地、无非md证据暂存。
- 用户明确授权协调后，已通知888计时开始/两次结束恢复；最后自然空档测量89.35s完成并发送恢复通知，没有取消任何他方进程。authority实现clean、main仅Plan及他方docs WIP；无自有build/test残留，无新代码warning。全仓fmt误触事件已回滚100无关paths，仅自有四文件提交并精确fmt校验。未在work跑根整档t/tv/tf/docs_gen。
- readiness:AC-01..06 pass；T-01..06已勾；T-07/AC-07独立final验收尚未执行。pending_closeout_gates:merge-owned裸cargo t+必要tv及实际experimental定向、到期tf判定、SD/ledger/index、archive/guard清理。work pass不授权跳过它们。

goal_receipt: objective=完成计划889 | goal_round=7 | stage=work | outcome=execution_done | completed_tasks=6/10 | next=review(final) | blocker=none | goal_active=true。


### 2026-10-30 final review 第8轮：pass，merge-owned门仍pending

stage: review | review_scope: final | reviewer_identity: Codex /root，本聊天内单独复审阶段 | limitations: 未创建其他chat/agent；本复审与实施同一上下文，不声称独立模型/外部reviewer。实施已停写；结论从实际75文件diff、原输入/绑定、生产/事务/信任边界、测试断言和raw结果重建，不以执行者勾选/总数代证据 | plan_id: PLAN-889 | plan_revision:2 | outcome:pass | worktree_path:D:/autostack/.wt/lang-889/auto-lang | branch:plan-889-dev | reviewed_commit:e188033c4e7866d19bc4e152937e6e1052b8ac69 | base_commit:e0b29ff7a（完整SHA见work交接） | dependencies:auto-down=c3464d0a0d36e4f8a7988b0c0c26448fc52361b4; auto-ui=3d6e1f15c1142aa797734edc26dca561511b72c3 | next:merge。

| 验收 | 复审核对/执行与结论 |
|---|---|
| AC-01 / V-01/02 / T-01/02/04/06 | 独立git-show基面逐字比17原源；14L/V仅native单元及来源注记改变，其他111行和所有其他列不变，原oracle/源/分母不动。正式40检查映射17原native+17AV+3补例×2；原L04/L05投影与L06专属trap证据、typed Clone独立起点及source clone拒面未混写。pass。 |
| AC-02 / V-03..05/08 / T-02/03/06 | 核runtime Cell/Frame/registry、List/Variant prepare→commit、clone显式工作栈/无分配drop、文本hooks/combined arena+registered边界、fatal caller sweep/普通70区别；核实际PE/typed/raw负控断言与T-02/03/06日志。共享ownership只省全scalar恒空表，原规则/诊断/保守预算与资源类型路径保留；768dense对照、owned25/collections59/native typed5及新物理峰证据有效。pass。 |
| AC-03 / V-06/07 / T-05/06 | 核fragment实际lexer/parser范围、新intern池与body同候选、exclusive validate/commit、UTF8目录与mint/current坐标、实际Rc身份和producer hook、history COFF复用但MIR重mint、声明显式失效。commit前重检runtime、全部fallible link/run/pub先于源候选安装；私有状态/独占引用保证validate后env不变。whole frontend/lower/同字节新对象反控实际发生且红；Native7重放覆盖失败/cancel/stale/foreign和prepared runtime漂移。pass。 |
| AC-04 / V-07/09/11 / T-04/06 | 核live credential不可Serialize、持久恢复再深验、manifest required闭包/源码坐标/COFF与实际runtime/tool内容、分页读取先查chunk地址再bounded读。独立执行33项实际伪造manifest身份/坐标/源控制：每项link/codegen0，旧PE/root不变。另独立执行合法三页COFF命中→manifest→新driver restart真42/codegen forbidden，漏页/超页控制拒绝。性能两轮failed整体保持；新fresh/restart各30的pass、旧hit/body/history各30+512与新9代表/close按确切修复分界复用合理，无削门/删样本/预备helper移窗。pass。 |
| AC-05 / V-10 / T-06 | 核独立父侧原始stderr/exit/memory/stage/identity解析，正式SDK不收expected，错oracle/type/真实漏free红、正常70不冒trap。新recipe/专用worker与旧receipt/protocol默认分离；完整campaign入口仍五×30+512，repair明确scope/churn0，两份结果没有冒称一次full绿。pass。 |
| AC-06 / V-08/09 / T-01..06 | 核S/N/H逐子窗接受/提交释放与main/888差额；唯一shared lower，无AV执行fallback/旧HIR→CLIF；根Cargo依赖及AV executor/ABC/opcode/contract受保护文件零diff。shim默认allocator与旧域路径保留，旧cache/CLI/protocol兼容证据适用。检查旧certify消费者实际按binding第8列a2r的ok-emit选择，14个第5列native变更不扩大旧10行证书集，无该调用缺口。pass。 |
| AC-07 / SD-01..05 / T-07 | 五现有Spec SHA与main逐字相同，新目标不存在；草案正确区分旧通用emit准入与显式collections路径、源码Str.clone/typed容器Clone、native静态增量与旧AV whole-source/HMR限制。六目标/四模块、supersedes/new路径及真实GOAL-002完整；SD SHA冻结4c8f12234cd52093f401fb1e3692e6050d14d9247bb746d617f4f706a2449eaa。未发布canonical或ledger。pass。 |

- 遗漏/延后/workaround扫描：未发现本合同内未完成项或擅自延后。源码支持并未靠typed替代；cold按原源字节选择批，大body走原single不是语言新上限；资源侧表优化不跳过验证。W01..05/typed entry/native在线帧/AV旧whole-source为既有明确排除或891规划，未冒关闭。没有需新增的debt candidate；已读KNOWN-DEBT-AND-RISKS既有记录，未为凑记录重写债务簿。
- health：无todo!/unimplemented!/dbg!；唯一collections candidate retained stderr为真实清理失败报告，非调试残留。新增代码无warning，已有Core/语言/AV警告数量及来源前记录保留；精确rustfmt --check与git diff --check通过，tree clean。所有raw仍本地，复审未修改实施代码。
- 独立执行命令：native manifest下 `cargo test --test collections_driver_tier collections_restore_rejects_all_forged_identity_axes_before_link_or_codegen -- --test-threads=1 --nocapture`（1test×33变异，4.54s exit0）；同文件 `collections_paged_coff_hit_keeps_required_closure_and_bounds_page_reads`（1test，1.36s exit0）。其余按已读断言/原raw/实际revision与明确不变性复用，不无由重跑全套性能/资源档。原源码/绑定/Spec的独立审计脚本exit0。
- 本地baseline/review-final-identity-controls.log SHA256 `3a36bc12be0c3b71af42ec0a91875aaabde65393748860af66ba9ae00222c82e`。
- 本地baseline/review-final-paged-closure.log SHA256 `01781ed25216a169d9957b75add429d624cbaa867b42d0c4a1cb0ab620e1b7a3`。
- 本地baseline/review-final-static-audit.json SHA256 `406babc03160cb95283328ce62634d1af2a0122d7cfb62b5f0cbc11f9850b02c`。
- findings: none（无待闭合in-contract项）；pending_closeout_gates: **merge-owned裸cargo t + VM/编译器触面cargo tv +实际experimental定向冻结验收门**；不得据此pass直接合入。cargo tf仅到期判定后主检出单实例批量回归；Spec/ledger/index/归档/guard清理仍T-09/10。Plan置reviewed，7/10，AC-08/09尚pending。

goal_receipt: objective=完成计划889 | goal_round=8 | stage=review(final) | outcome=pass | reviewed_commit=e188033c4e7866d19bc4e152937e6e1052b8ac69 | completed_tasks=7/10 | next=merge | blocker=none | goal_active=true。


### 2026-10-30 merge 第9轮：受影响控制隔离缺口，回work

stage:merge | plan_revision:2 | outcome:needs_fix | reviewed_commit:e188033c4e7866d19bc4e152937e6e1052b8ac69 | delivery:none | next:work | pending_closeout_gates:bare cargo t/tv/experimental/Specs/ledger/archive/cleanup。

- 裸cargo t已实跑，日志baseline/merge-cargo-t.log保留；尚未完成，既有ac_case_tests的r1_consumer_streams_observed_not_rejected实际120s timeout，当前别的旧消费探针在本树冷构建auto-cte/auto-abi release依赖。不是整档pass，未静默豁免。捕获退出/失败并完成归因后才可重冻统一门；不修改888的runner/filter/选择守卫。
- F889-M1（P1，AC-05/06；T-06/07）：新 `ace_m1_native_collections_889_report_runtime_content_identity_is_current` 直接bitflip authority的共享runtime .lib。ACE_M1_CASE_LOCK为进程内Mutex，而nextest每test独立进程且本模块无串行组；它能使同树其他collection请求读取坏库、拒绝或受扰。实际操作/缺隔离来自测试和nextest配置，未宣称已随机复现某一业务错误。期望：保留真实同长度bitflip→stale→字节恢复→Fresh控制，但只改该测试自己的库/源配置镜像，原共享库逐字保持。最小修复H仅该889具名test及私有fixture helper，不动生产worker/protocol/代码/阈值/正式采样/runner/filter；定向复跑该stale控制与同组source/rounds并发代表，核共享库SHA不变。此前产品语义/性能AC-01..04证据及Spec delta内容保持；受影响的AC-05/06/07 final pass暂失效。
- 为避免继续对已知待修控制跑整档，已核实本次own cargo t PID19928及完整cmd/父子nextest后仅停止该进程树；不取消888/890命令。原log完整保留失败/中断，不把剩余未跑条目当pass。回executing，T-06/07重开，5/10，其他已闭合项不重做。

goal_receipt: objective=完成计划889 | goal_round=9 | stage=merge | outcome=needs_fix | completed_tasks=5/10 | next=work(repair F889-M1 and gate diagnosis) | blocker=none | goal_active=true。


2026-10-30 work第10轮诊断补证/更正：前条“停止PID19928”在核对期间已过时——实际进程已由nextest fail-fast退出，安全身份核验拒绝发送taskkill，因此**没有终止动作**。实取gate session退出100，158.526s，137/3217执行：122pass、2fail、13timeout、675skip，3080未跑。两fail均889：实际whole-copy控制报runtime-content而没发生预期copy；source SDK L03同报library changed。该真实错误与stale bitflip测试同时pass的时间对应，确认F889-M1不只是理论竞窗。13timeout均旧ac_case/ac_av/cte冷构建消费（原日志逐项保存），仍需隔离重放归因，不把它们豁免成green。Test输出改写了tracked p818-series.json，先复制本地baseline/merge-t-generated-p818-series.json再精确restore该既有报告，不提交更新报告。
- 修复仅889 stale test：物理复制同一runtime/shim源与有效Cargo配置、已认证库/receipt到测试私有镜像；该进程AUTO_LANG_ROOT只指私有源/库，生产worker仍按同一源码/build/content检查。真实同长度bitflip与恢复只作用私有库，前后断言共享产品库逐字不变；环境RAII还原，无symlink/junction。定向stale test已实际通过exit0，3.90s；正在同一nextest并行池复跑889非ignored具名族验证shared source/whole-copy等真实兄弟，formal/perf/512不重跑（生产代码/输入完全零diff）。


### 2026-10-30 work第10轮完成：F889-M1已修，6/10

stage:work | plan_revision:2 | outcome:pass | repair_commit:44ed8db40 | worktree:D:/autostack/.wt/lang-889/auto-lang | branch:plan-889-dev | next:review(affected F889-M1) | blocker:none。
- F889-M1最小修复只改根ace_m1_tests.rs该具名stale控制56行及移除1行共享路径：复制本次实际runtime/shim源/有效Cargo配置、实际库和认证receipt至私有目录，真实worker仍prepare/hash/验证它，bitflip仍同长度真实byte，stale轴断言和恢复Fresh不变。原共享库前后逐字保持，新RAII还原AUTO_LANG_ROOT。没有修改生产pipeline、ABI/cache/protocol、源/样本/门限或runner/filter。
- `cargo test -p auto-lang --features test-cases --lib ace_m1_native_collections_889_report_runtime_content_identity_is_current -- --test-threads=1 --nocapture` 1/1，3.90s；随后用实际nextest多进程并行池 `cargo nextest run -p auto-lang --lib --features test-cases -E 'test(ace_m1_native_collections_889_)'` 6/6，26.643s，包括原失败source SDK40与真实whole-front/lower/copy反控、真实漏free/错oracle+type、rounds/restart/stale全部通过。该test实际进入新私有镜像，且共享库字节不变断言通过，不是靠串行组掩盖竞态。
- 旧timeout归因代表隔离重放（未改相应代码/断言/超时/过滤）：`cargo test ... --lib tests::ac_case_tests::r1_consumer_streams_observed_not_rejected -- --exact --test-threads=1 --nocapture` 1/1，1.72s；同入口`tests::ac_cte_probe_tests::cte_probe_wrong_exit_not_green` 1/1，15.68s。表明冷构建并行窗相关；不据两代表自动把全部13timeout当pass，后续merge统一门必须新冻结再完整跑过。开发本轮没有重跑裸t/tv或正式性能/512。
- 健康检查精确rustfmt/diff-check过，只有上述自有test改动，tracked p818输出已先保留本地再restore；authority clean。生产代码、正式采样器、依赖与输入无改，全部既有语义/性能/512/SD证据保留，不重新开其预算或阈值；本轮只闭合受影响AC-05/06测试隔离，final pass需更新baseline。
- 本地baseline/repair-m1-private-library.log SHA256 `902d73250fe8de169dffb8db02afffe9f31acc0d254ad09f8b3edd9b1ad73770`。
- 本地baseline/repair-m1-nextest-parallel.log SHA256 `21870102af2c981e1b0a4287da2b5957a1160df830cfe2c96c874c75b526ec88`。
- 本地baseline/repair-gate-cold-native-replay.log SHA256 `256e43aa1e7bc6a70fb7397b83e4c53cb71263398a92a82d5b54f6dee146c2bd`。
- 本地baseline/repair-gate-cold-cte-replay.log SHA256 `242c13c06be17093f4ba5bbd2f15a4e28af6ebcb98417fb2d644e4c783c47bba`。
- 本地baseline/merge-cargo-t.log SHA256 `fd76d8ce501cf42f3f446a82b9bcb8d89fc29a2f4a10f59bf5b786e8b358a362`。
goal_receipt: objective=完成计划889 | goal_round=10 | stage=work(repair) | outcome=execution_done | completed_tasks=6/10 | next=review(affected) | blocker=none | goal_active=true。


### 2026-10-30 final re-review第11轮：F889-M1 closed，pass

stage:review | review_scope:final(affected repair) | reviewer_identity/limitations:Codex /root，同实施聊天的单独artifact复审阶段，未冒充外部context；实施已停写 | plan_id:PLAN-889 | plan_revision:2 | outcome:pass | reviewed_commit:44ed8db40ad69334abd477c879450bcbf423c4e5 | previous_reviewed_commit:e188033c4e7866d19bc4e152937e6e1052b8ac69 | worktree:D:/autostack/.wt/lang-889/auto-lang | branch:plan-889-dev | dependencies:原auto-down/UI只读pin未变 | next:merge。
- F889-M1 closing oracle复核：git diff只1个test文件，真实源/配置/已认证库物理复制，worker真实read/hash/身份query仍生产路径；同长度bitflip仍使collections-runtime-content轴STALE，恢复原字节仍Fresh，断言未软化；原共享产品库前后逐字相等。拷贝无链接、不绕过runtime source/build/content admission；AUTO_LANG_ROOT在该测试进程由RAII还原，同模块libtest由既有case锁约束，nextest进程环境独立。6个889非ignored真实多进程并行场景全通过，包含两条原实际受扰失败、source40/whole-copy实反控，不靠串行filter过门。F889-M1闭合，AC-05/06/07 pass。
- 未改任何生产compiler/runtime/cache/protocol、采样器、依赖、原源/oracle/绑定或配置/门限；e188033c4的AC-01..04/SD-01..05结论和精确性能/512证据按此差额继续有效。SD草案SHA仍4c8f12234cd52093f401fb1e3692e6050d14d9247bb746d617f4f706a2449eaa，现行Spec/input版本同前。无新增debt candidate或in-contract延期，健康检查/工作树clean通过；root43条既有warning无新增。
- 两条旧timeout代表隔离重放通过只作归因证据；第一轮裸t的13个timeout/2fail与3080未跑仍保留失败，不据此变pass。统一门必须在此新冻结上由merge重新实跑，并取得完整成功。没有在re-review重跑t/tv/正式性能/512或不相干档。
- pending_closeout_gates:merge-owned裸cargo t、必要cargo tv、实际experimental定向面、canonical/ledger/index/archive/guard清理。更新final pass不授权提前合入。当前7/10，T-08..10未勾。

goal_receipt: objective=完成计划889 | goal_round=11 | stage=review(final affected repair) | outcome=pass | completed_tasks=7/10 | next=merge | blocker=none | goal_active=true。
