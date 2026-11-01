---
plan_id: PLAN-904
status: archived
feature_name: AX-C HIR到Rust集合/变体与含类型声明的函数级增量
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01
plan_revision: 2
supersedes_spec_components: [docs/specs/auto-a2x/project.md, docs/specs/auto-a2x/architecture.md, docs/specs/auto-a2x/incremental-and-cache.md, docs/specs/auto-a2x/checked-resource-facts.md, docs/specs/auto-a2r/project.md, docs/specs/auto-a2r/record-text-hir-backend.md, docs/specs/auto-ac/function-incremental-source.md, docs/specs/auto-ac/source-collections-variants.md, docs/specs/auto-test-case/a2r-hir-certification.md]
new_spec_components: [docs/specs/auto-a2r/collections-hir-backend.md]
touched_goals: [GOAL-002]
affects: [auto-a2x, auto-a2r, auto-ac, auto-test-case]
depends_on: [PLAN-902, PLAN-864, PLAN-889]
current_step: 14
total_steps: 14
---

# [PLAN-904] AX-C：HIR→Rust集合/变体与含类型声明的函数级增量

## 0. 变更摘要

承902正式交付，在现有ACE四A2X crate内新增显式collections域的Checked HIR→共享prepare→Rust typed provider/runtime→真实认证，覆盖原14 L/V源、List<Str>/拥有record/Variant、is/for-in、资源/失败/恢复及真函数增量。源语义沿864/889，不经MIR/AV/native结果恢复或执行，不新增workspace/runner。

**拆分决定：一个Plan，type声明片段缺口纳本包前置S任务，不另拆号。** 同一core source adapter的保留声明/函数fragment坐标与凭证接口已经服务collections；单独先做一个无目标消费者的修复包会重复S/验收/交接。T-02先含type/enum最小真链，T-03闭合有界共享修复，再扩主体；若必须扩成通用多文件坐标/增量平台则needs_replan，不悄悄扩大或删增量门。

依据[Design51](../design/51-shared-a2x-and-hir-to-rust.md)、[A2X战略](../design/strategy/shared-a2x-and-hir-to-rust-roadmap.md)。本轮复核修订为drafting r2、0/14；不建树/编译/测试/探针，不改893/903合同、现有源/expected/binding或current Spec。

## 1. 目标

- G1：原源码完整进入新显式Rust通道，与独立手写reference/闭式oracle相符，14 L/V及6补源逐字节保留。
- G2：集合/变体的名义类型、源码序/单求值、转移/view/活动臂、失败原子性、逆序清理与预算正确；源拒绝不靠rustc错误代替。
- G3：含type/enum声明的纯函数body编辑真片段解析/检查/发射，未改函数实际生产0及居民身份保持，来源正确，实际依赖失效与候选/发布/恢复闭环。
- G4：新recipe/fileset/cert严格隔离，日常emit/golden真实目标编译/执行spawn0，显式actual+reference两腿认证，文件化/固定组/资源/四状态正式时间门首例即闭合。

非目标：List/Variant源码clone、List参数/返回/record字段/嵌List/Variant载荷内List、Variant参数/record字段、source str/char/wide、用户泛型/递归类型、用户方法扩面/闭包/HOF/console/host/FFI、Unit入口、第二目标或跨平台承诺；不实现全部822 typed Place/指令API，不做native ABI/COFF或旧A2X切默认/退役，不修全M1/13慢测/通用block增量平台。902方法体仍按其正式声明上下文策略处理，不借此新增方法暖编辑接口；本包纯body快路径对象为当前collections合法自由函数及902含type源中的自由函数。

## 2. 架构方案

`原.at+collections profile → ac-core source/resolve/typecheck/Collections verifier及受检资源事实 → SemanticView/FunctionPlan → Rust typed IR/显式资源helper/printer → fileset/manifest → 父侧golden 或显式actual/reference认证`。

共享层只加中立List/Variant/名义图/guard与投影模式、源序求值、已验证拥有/消费/帧出口事实和依赖；Rust Vec/enum/间接表示、借用安排、容量增长/helper/打印归Rust provider。collection受检规则只有core权威，不在A2X重新做ownership/类型推导。若现有凭证不够则补S最小受检事实出口，不消费MIR作为语义输入，也不在Rust runtime解释HIR/ABC。

## 3. 技术栈

现有experimental/ace及ace-a2x-core/rust/store/worker、core source/verify、ManagedStore与受控auto-tool-execution/Job、Rust edition2021与沿902 Windows x64工具链、AutoDown/TestCase/固定cargo t。目标runtime是typed值/容器/清理helper，可用Rust内部泛型但不开放Auto用户泛型；无新依赖、workspace成员、静态native runtime或第二store/裁判。

## 4. 需求分析与背景调查

### 4.1 授权及交付基面

用户先授权AX-C立项，本轮进一步明确复核904的type/enum片段、资源/增量/认证与893903交接，遗漏可直接修订，仍只规划。r2保持目标/20源/14任务及原数值预算，不取得实施/构建/探针/清理/跨聊天授权。main=`b1a4871aa6a8d960130fc2e1aed8bf6dc4183c5a`，保留多会话WIP。

| 基线 | 已核实交付 / 不能冒认 |
|---|---|
| 902 AX-B | archived14/14，delivery `4303c4457050`为HEAD祖先(exit0)，SD-01..06已沉淀/读回，树/分支/材料scope已清。record/text/text-record、参数模式/名义record、A2xStr显式清理、资源凭证、AXS2/AXR2、rt fileset/cert和四正式原例已交付 |
| 902限制 | 含type源的record族warm片段不可用，现走整路径但后端未改片段复用；不能当全前端0工作。warm_edit只接自由函数，方法编辑走声明上下文冷路径。不能把原type-free性能证据套新含声明负载 |
| 864/889 | source collections合同与原14+3源、拥有型/enum命名图/函数片段凭证已交付；889的native机器runtime不等Rust容器实现，不拿其执行结果当oracle |
| 899 | archived7/7，delivery1796774d4为HEAD祖先；接受新profile/工具命名。D-899-A三项测试读去跟踪附件仍是已登记债，新树红不冒全局绿，不在本包顺手修 |
| 893/903 | 用户确认实施中，lang-893/lang-903树存在，读取HEAD同b1a4871aa；头部draft/0为未同步簿记，不反推写权释放。本包只读其合同/实际占用，不代改进度或发消息 |

902归档同时登记批量回归due待处置；既有主检出与fresh树结果有D-899-A差异，本包不称全ACE无债，不从主检出磁盘旧附件抄到新树凑绿；批回归仍现行regress队列/单实例，不由立项自动运行。

### 4.2 代码/Spec调查与拆分依据

| 当前路径 / SHA256 | 静态事实与本包工作 |
|---|---|
| core/src/source/adapter.rs `70a088d897b2fff7714f6ff09f68f70200c44b6c799dce88c223027a73dc8090` | emit_function_body/emit_scoped_document、prepare_candidate_document/FnCandidate；声明仍来自保留env，body src是fragment。record/variant span仅scoped Collections设内部inert0，其他record族仍可拿原声明span对短fragment检查；reanchor_spans要求来源内UTF8边界，不能简单删除check |
| core/src/verify.rs `be1e3ed1516a9e81c8aaea1ccd5ca2b8430a254b4154b05ce0685dff852d84e1` | verify_collections与verify_function_ref已有同源selected-body规则/名义context；Collections凭证resource_plan=None，因原执行合同为frame-owned LIFO，并非无资源责任。须补可消费的受检集合拥有/出口事实或等价不可伪造凭证 |
| a2x-core/src/semantic_view.rs `a30086c4d0c949ff41d8b03c69b7b52266ccbb6ed0aac83a72f2ac5869639fa7` | Ty只有i32/bool/char/str/Str/Record，profile_of与合法节点不接Collections；from_parts绑定context_revision和单体凭证。List/Variant不能只加printer分支 |
| a2x-store/src/lib.rs `e990518cb99da8ec9aeea7c0a9d963adafe6123034730c7fd67c3902baee5269` | Family仅Scalar/RecordText，cold_assemble_profile拒Collections；warm_edit以fragment src准备，之后构造居民文件集。AX-C需把候选commit、source/current坐标、resident/manifest/有效receipt作为同一发布事务核闭合，不假定沿旧函数就自动成立 |
| auto-a2r/record-text-hir-backend.md `787811a44f2c5e1cb3ec8792dcd4efcf33503348929b5ddb8816a8680182fb19` | 正式记录902支持/限制；仅review+merge后按实证移除含type自由函数片段限制，不重开902/改其历史收据 |

以上代码路径相对experimental/ace/crates；Spec相对docs/specs。已读source-collections-variants、collection-place-profile、auto-av/collections-variants、native-collections-variants、function-incremental-source及A2X/cache/认证现行合同。类型跨度修复局限同一单模块source候选链、原声明上下文和来源投影，可与本包直接证明；本轮没有执行反例，不把静态原因当已修复证据。

### 4.2a r2补查的具体缝隙（静态观察，未执行反例）

`SemanticView::from_parts`当前先核domain/context_revision，再按def_id匹配一个函数并挂上credential.resource_plan；这不足以作为“同owner/同声明上下文、不同body”凭证不可混用的证据。`CheckedFunction::body()/def()/owner()`和候选的env_id/base_generation可供绑定；具体等价/规范内容比对由T-04落实，不假定`context_revision`或函数名自动证明body和资源事实相同。原文件SHA与r1相同；补读a2x-rust/src/cert.rs SHA256 `409825d89b33d7633ac0aebd6bbad1e09ed0ff76657f4abc1968d662396cd9ba`，旧CertDraft字段并不自动提供新通道的执行配置/模式隔离。

本轮查看893/903目标树状态：893观察时clean；903有`docs/specs/auto-test-case/ace-m1-conformance.md`修改和未跟踪`ace-m1-gap-audit.md`，均是其WIP，只读状态不消费其候选规则/结论。新增ui-020兄弟只读检出不是本包scope。不以计划头draft/0推断会话闲置，不改其他Plan或取新号。

### 4.3 required源码与独立oracle

源根`crates/auto-lang/test/cases/ace-m1/src/`。新资产逐字复制/hash钉定，原源/expected/binding不改，辅助例另ID；不能删分支/换算法或手写HIR代源。

| 输入 | required oracle |
|---|---|
| L01/L02/L04/L05/L07 | 42 |
| L03 | 0 |
| L06 | 唯一list-index原因，非普通return70、rust panic或任意非零 |
| V01/V02/V04/V05/V06/V07 | 42 |
| V03 | 3 |
| LSTR864 / FIN864A / FIN864B | 51 / 42 / 53 |
| N889-list-variant / N889-nested-variant / N889-optional-variant | 各42（按这些实际源，不套其他同名Token历史值） |

规划读取的20源SHA256如下；T-01核当时主线并复核，漂移先说明来源，不改原源凑hash。所有值来自源/独立手算及既有冻结oracle，不拿AV/native输出生成expected。

| 源ID | SHA256 |
|---|---|
| L01 | `8247b7e7688e1931a36bc09eb4ec2cc789a3b099237d20d5dc8095a71e76a4f5` |
| L02 | `8276953bd3fe700fe44593092117d2b93b691aff0747bfb0f1099cc2008432e2` |
| L03 | `299034a89f81f97a0ab9ffa71f278d05262db5747a77ec5a460dbdbfce070339` |
| L04 | `a79b03e53e6e82bda9000a1b6cd1bcfae695d4cc427928261e9d13bda19b0b99` |
| L05 | `3c55fc65f45724c072aaf7a772cfdc7d52da0d4f117ec6f23841795551629149` |
| L06 | `9b4aaa8834835283629077af431d7864d9c88a732060cebbd50fe24921c21be8` |
| L07 | `8aa9b61d53b3f0ce715a98d3c6ff1b0a7fd5f75ede2ea1dc132c650e97577aec` |
| V01 | `900acc88f43f9505b91fa6c4096642589fa7e3ca770833106bd552c481857dee` |
| V02 | `281396b1a3d5ea129a5c4e34672287975bac87ed7b543af7fbc828ce7869259f` |
| V03 | `99bc231454ba7426c61828525e58dfd2514c6be54249ccbcab43d9c1e0ebf532` |
| V04 | `c9303cad597133e0a517b70d5f0693a8a4961199a3148b82abffe039e22839cc` |
| V05 | `734c211d1ea0e62d52f14617369a77c975c0a23c915e3ed9dff91fa1f4c7c0c3` |
| V06 | `7145928b2c570c848f6ca7746545d1ca3d31790045855080d05db4c3d4112383` |
| V07 | `04844f208d458a043dd6ef2d2202a8ffb09789010d750d075cc164ee59135478` |
| LSTR864 | `f36e92cfa92d4bc4240995c75e312fa84076e7bb7d983b75141dadb7e70524ff` |
| FIN864A | `39524c0208966b1aca826afbc1b6c3898526f9373275fb62d7e8dd13662ceb81` |
| FIN864B | `d0ce9a71534a37a289eea7ff22307d05414042420548263b3d1a4b4c431286b1` |
| N889-list-variant | `b90d64918e20ea8bb7a702871067cf177064adab604cad4af1205a3f6fb33a7e` |
| N889-nested-variant | `4b6433b03528d986ef67edf9d7420409f8e7d64ebccafceadf9fc4d23da6e61f` |
| N889-optional-variant | `e800cab21196d7e848e3ad19bc7af5903f8534e55d52d064695ab38c5c781b84` |

## 5. 详细设计

### 5.1 支持与拒绝闭包

| 面 | required支持 | 保留拒绝/边界 |
|---|---|---|
| 源域/入口 | 显式Collections，main零参i32结果，原域i32/bool控制流/直接调用/递归 | 不自动扩scalar/record-text域，bool值内部仍支持，bool入口不在首期；无宽/Unit/多模块 |
| 类型 | 局部List<int/bool/Str/无环record/Variant>；Option/Result/有限名义enum、单载荷、无环嵌套Variant；record字段只原值/Str/record | List不嵌List、不作参数/返回/record字段/Variant载荷；Variant不作参数/record字段，允许原域合法拥有Variant返回；无用户泛型/递归类型/str/char |
| List操作 | new(cap4逻辑合同)/push/insert/get/remove/set/len/clear，LHS/索引/RHS各一次、源码序；Copy捕获在自身求值点 | owned push/insert/set要显式move局部或fresh rvalue，不能隐式clone/复制析构责任；索引负值和边界同list-index |
| owned view/for-in | get的受检只读投影/Str.len，Copy取值；for-in len循环前一次，Copy绑定/owned投影，break/continue | view不物化/不逃逸/不隐藏深clone；迭代内push/insert/remove/clear按原source.iterate-mutate拒，原允许set不能误拒 |
| Variant/is | 命名enum/Option/Result构造，穷尽is及载荷绑定、无环前向引用、旧臂替换，Copy拷出/owned投影一次move | 无guard/通配/重复/漏臂，错臂/二次move静态拒且运行防御；源码List.clone/Variant.clone仍拒 |
| record/text | collections原受限Str literal/len/clone及record字段/构造/读、跨函数合法拥有返回/模式 | 不据902较宽text面自动扩Collections，用户方法/ext/trait、mut实例和任意字段写形态按原域拒绝 |

完整支持/拒绝矩阵T-01按真实节点/上下文/参数模式冻结；source与typed helper探针分账。源码参数/返回/字段排除不能用Rust类型“能表示”放行。若core原受检规则未覆盖某拥有/guard安全条件，补权威S验证及反例，不让目标后端另造第二分析器。

### 5.2 含type/enum的片段增量S前置（同包）

T-02先在当前源链证明最小限制，T-03修复adapter/来源封装及A2X消费。保留声明的名义图和来源属于retained snapshot，候选函数body属于新fragment；内部scaffold可用明确inert span，但它不是对外源位置。声明诊断/source map需通过retained/current来源引用定位，body要映射到当前完整源；禁止把全部跨度夹零、放宽越界检查或把旧fragment位置冒当前source。

必验：type在编辑函数前/后、声明位移超过fragment长度、中文/多字节与变长/换行编辑、无环前向enum/type混排、同形异名义类型；重锚后与冷路径语义一致，对外所有实际body/声明诊断能定位正确原源/UTF8边界。仅目录/声明元数据装配和必要坐标重基可全局分账，未改body AST/检查/HIR/目标片段不复制重产。

候选须有受基面约束的编辑授权：当前完整SourceSnapshot/hash、UTF8替换range、目标稳定函数身份及候选fragment；可用等价既有接口，不强制新增公共类型。range越界/切半个UTF8/覆盖第二声明、错base source hash、签名/owner错配不能进入body快路径。fragment是解析工作单元，**manifest/cert/source身份绑定替换后完整源码**（包含保留type/enum及全部其他函数），不是fragment-only hash。保留声明的原始位置、当前源投影、候选片段偏移/mint位置分列，序列化恢复也保持可核对的来源。

连续编辑两个不同函数A→B→C（且前一编辑改变长度/声明相对位置），每一步把实际编译运行值、可见body/声明诊断与独立冷编译对照；再restart→编辑第三处/回退A验证。不能仅用单函数A→B→A片段值证明env/source generation已发布。接口/声明编辑明确走cold/dependency门，合法body编辑不能误归声明变更绕过零生产门。

V-14的补例`INC-904-DECL`在T-02/03变成正式文件化输入（本轮仅设计）：以两个函数`left_value()=20`、`right_value()=22`为A；main在`List<Choice>`中放入`Choice.Data(Item(left_value()+right_value()))`，remove后穷尽is，Data臂返回item.code，End臂返回0。`type Item { code int }`与名义变体Choice（Data(Item)/End两臂）按已有语法分别置于函数前/后，源正文由T-02按实际语法冻结/hash，不能因当前实现拒绝而改算法。独立手算序列：A=42；仅改left为`20+1`（变长）得B=43；仅改right为`22+1`得C=44；restart仍44；再仅恢复left得D=43；仅恢复right回A=42。每步各有完整源身份和手写reference，不只拿冷编译作oracle；错误range/base和取消候选保持前一步实际值及generation。

902含type的Record/TextRecord自由函数body也必须得到共享修复的兼容见证，恢复真实warm片段，不用整源cold_assemble后片段hit冒前端0工作；902原文件集/golden/recipe不变，内部inert metadata不改变语义键。Collections用户方法不在本域，原902方法声明上下文策略不暗改；类型/字段/臂/签名/配置编辑走具名interface/cold道并诚实计数，不伪称body快路径。

修复不是通用多文件SourceMap重构、CST平台或任意方法增量项目；若发现必须改变这些共同协议，T-02 bounded feasibility转needs_replan，保原AC，不拆号预占/无据延期。选择同包并不豁免S窗口和完整消费者验证。

### 5.3 受检事实与Rust容器/变体表示

共享SemanticView/prepare导出名义type/list-element/variant-arm图、声明序discriminant、owner/模式、eval_args+bindings、临时/guard/只读投影与受检拥有/消费/帧出口责任。整模块和单体都来自core原verify_collections/verify_function_ref及同一规则；Collections没有TextResourcePlan，不允许默认为“无需清理”或强套902text计划。最小S可补不可伪造的Collections body facts/清理责任摘要，绑定body/owner/domain/context及实际模式/类型依赖；错误/旧/跨owner凭证在目标emit前拒。

凭证准入同时核body/声明签名/资源事实的规范内容、module/名义owner、域/profile/caps与候选来源；不能只比decl context、def_id、source名或单个源码hash。必须有“同owner+同context，A的CheckedFunction/资源计划配B的SourceParts”拒绝反例及B自身合法凭证接受控制；资源分类/参数mode/载荷型/臂序变化、未使用类型表中的越界/自环/间接环都按原权威门拒/失效，不只遍历被当前函数引用的表项。坐标等价重基不强迫语义重验全部函数，语义内容与来源分别核。

Rust采用typed容器/名义Variant表示，具体Vec/enum/显式间接节点方案由T-02冻结。new的逻辑cap4、增长/预算、元素转移/只读view和活动臂保持源合同；不同owner同布局不能混装。禁止靠Rust隐式Clone/自动Drop、unchecked index、panic/OOM abort满足语义；合法Auto因Rust借用/类型报错是后端未完成/infra，不改源凑编译。

无环Variant深度沿原域预算，不新增语法深度上限；类型图遍历、构造失败回滚、清理须显式有界工作结构，不由宿主递归栈决定支持域。T-02先核Rust表示/编译限制，资源侧用3072层/256KiB小栈同类见证和独立浅层控制证明不乘宿主栈；它是见证非语言深度常数。不得以提高rustc递归阈值或静默拒深链替代正确表示；若此机制超包范围先replan。

### 5.4 资源语义与失败原子性

List/record/Variant/Str拥有责任沿原collections帧合同：源码move只转一次、旧根dead；get/view不分配或克隆；remove把责任转出；变体owned投影后原payload标moved、guard在switch/相关写后失效；返回拥有值转出帧，不提前free。正常return/break/continue/短路/循环/trap/cancel均清规定owned临时/帧根一次；List元素逆序、buffer最后，活动臂仅释放实际活载荷，不按Rust默认词法Drop猜等价。

push/insert/set/switch：索引/类型/预算/必要容量及新载荷完整准备后才进入无用户代码/不可失败commit；失败保旧目标和未提交输入责任，部分新输出回收。RHS既有任意副作用或已完成异根move不承诺回滚。增长失败、set释放旧元素、clear尾删、嵌套Variant/资源record及callee多帧trap均需真实事件/oracle，不能只检查最终值/exit。合作取消在工作轮询点转受控拒并收尾；外部kill是infra，不冒guest清理。

T-02须冻结操作计费/合作取消表：循环回边、调用/递归、List容量/节点工作、Str字节复制、构造/clone工作逐类给量纲、预算/轮询点和超限原因；应用原profile/caller更严额度，不能用Rust机器指令数冒ABC fuel或只靠外部run≤30s杀树。保原CLONE按节点/新Str字节预充与零allowance入口检查的语义观察，目标内部表示开销单列；变更计费规则/扩大预算须先replan。至少有真实长循环/递归及大clone/构造在规定工作点收到取消/耗尽后清理的控制，立即返回的小例不证明取消。未冻结表与对应正负控不得关闭T-02或开展依赖实现。

清理不得因已取消、fuel耗尽或再次分配失败而被中途放弃：原根/活臂/部分输出的必要清理使用已预算保证的空间或无分配结构，清理阶段不再因轮询抛取消，不尝试不保证成功的扩容。先以浅层正常/失败控制，再以3072层小栈覆盖构造/遍历/部分失败回滚/末主清理，不能只测drop一个已完成值。直接typed helper资源探针是required辅助见证（含深clone及第n子对象失败），独立于20源支持；不因源码容器clone拒绝就跳过已约定资源反例。

RHS/索引/实参在源语义规定的求值点各一次，源checker与typed helper两条路径都核同根别名/guard/消费责任。push/insert/set/switch失败前后分别记录旧容器完整内容/len/cap/活臂、原owner或已求值临时持有的责任、实际live/事件；前置检查失败不能提前标旧owner dead，成功提交后不能保留双owner。failed mutation的可观察不变性在helper故障缝证明，不向源码新增catch/回滚任意副作用；for-in允许set的合法控制与结构变异拒绝同时保留，不能靠一律拒变异逃避view语义。

复用并按新runtime版本扩902默认关闭的fault/events钩子，增加容量/节点/活字节/所有权转移观察，fault在实际分配/提交前生效。逻辑语言事件与真实backing request/capacity/metadata分列，不能打印fake alloc/free假装内存回收；close/取消/失败后实际live/lease/节点回落。typed helper直接探针可验证深clone/部分clone回滚/类型错配/guard防御，须消费同一生成runtime内容及合法typed入口；不开放源码容器clone或把探针成功算源码支持，不造typed MIR到Rust旁路。

### 5.5 函数级增量、依赖、发布与恢复

增加独立Collections family/语义/目标片段层命名空间（拟AXS3/AXR3，T-02固定注册ID/版本），复用ManagedStore/AXM1类required闭包；旧AXS1/2与AXR1/2键/字节/语义不变。共享语义键不含Rust字段；覆盖稳定owner/module、签名/参数mode、List元素、Variant名+有序臂/载荷传递图、record字段/资源分类、callee实际读取接口/事实、权限/profile/规则/pass。Rust键再加edition/合法化/printer/runtime实际内容/flags；声明产物与完整文件集绑定全部必需类型和片段。表序/span变化不伪失效，名义/臂序/mode/实际依赖变化不漏失效；未启用的meta/CT不冒支持，已查询事实不能只看调用点源码。

纯body候选只解析授予fragment，目标resolve/typecheck/emit/verify/prepare/lower/print/persist按真实dispatch计数；未改caller/disjoint全0并保持AST/HIR与派发片段对象身份，不用新分配同hash冒carry。A→B→A真实历史字节命中，变长编辑外部map指当前源；类型/布局/签名变化有cold/dependency原因及真实工作，不套body门。source/checked凭证及资源计划必须属于本次candidate，不能从env旧body发射。

零生产证据取真实parse/resolve/typecheck/emit/verify/lower/print/persist分派与whole入口，而非只在A2xSession入口加计数。居民身份核实际被派发的payload/字节所有者及AST/HIR对象；复制同hash或只保wrapper地址的反控必须被区分。最终文件拼装必要复制与坐标侧表重基单列，不借装配遍历重新克隆未改生产payload。类型/元素/载荷闭包含局部和表达式隐式类型操作数；body-local intern池新增/重排、无关同形类型/未读事实不误触发全量生产，实际读取依赖变化不能漏失效。

工具/flags或执行配置变化按所属层失效：语义层保持target中立，目标runtime/printer变则目标片段/manifest失效；仅rustc身份或认证配置变化可保持相同语义/源码片段，但旧证书必STALE。不能以“都重编”遮掩失效边界，也不能用语义hit授信旧认证。

候选持有完整body-local类型池与资源事实，prepare/verify/分配/print/required持久化/完整fileset/父侧发布先在旁路完成；唯一commit把source/env generation、居民片段、坐标、manifest/有效receipt同代安装。失败/取消/foreign/stale/关闭/写盘满/部分文件/迟到不改变旧发布；失败后旧产物实际可复验，再继续合法编辑证明env已是正确版本。只clear map或仅计数不算事务/回收。T-06核现有warm_edit候选实际commit与resident装配路径，不沿旧实现假定成立。

restart新进程从不可信字节有界decode/reverify类型/资源/身份/required闭包，坏/缺项拒或重算，修复后再次restart真hit；read-only写0。GC不能回收当前/历史必需依赖及合法pin，最后owner释放才清；source close拒后续编辑，session/drop/pin分别验证资源释放，原close_env仍可装配的合同不被误改为“所有对象已释放”。

### 5.6 文件化、日常与独立认证

拟新recipe `ac-a2r/hir-collections-emit-v1`，仅Collections profile；fileset `auto.a2x.rust.collections.fileset/1`、证书 `ace.a2r-hir.collections.cert/1`与892/902/旧a2r严格互拒。复用同a2x-worker/受控协议/父侧adapter，不建新worker进程体系。文件相对路径/required集合、typed ListIndex/VariantBranch/Resource观察词表T-02固定，字段未知/跨recipe/profile/缺必需输入拒；旧文件集/golden不变。

新资产拟`crates/auto-lang/test/cases/a2r-hir-collections-904/`，20源逐字复制/hash、独立手写Rust reference及闭式value/trap/trace/资源oracle；T-02首例就进正式TestCase/父侧判定，T-07扩完整suite/固定组。golden由真实发射、跨nonce确定性重复、独立审查冻结，不自动把当前输出覆盖期望。原M1 binding/expected/分母及旧a2r身份不改；向903交并列producer/限制/证据，不代其裁定迁移。

日常仅预构建worker emit、manifest父侧全文件重hash/golden和strict证书当前性，目标rustc/reference编译器/生成程序spawn=0；不隐式certify或每case维护构建worker。缺worker/工具当前性未知是infra/STALE，准备在正式窗外单列；真实进程观测/偷偷编译红控不能只信worker自报。

显式认证对同次现场actual完整文件集与逐例手写reference分别真实rustc编译运行，双方各等独立oracle；reference不得复用生成runtime/helper或抄actual算法。L06必须具名ListIndex，普通value70/任意非零/panic不通过；debug/release覆盖原14及资源/边界补例。错oracle/reference、源/文件/runtime/producer/schema/工具/nonce错配、额外/缺文件、先输出成功后异常exit/hang/flood均拒，不能从任意exit分类。

证书绑定source/全actual/manifest/reference/oracle/runtime/flags/profile/producer与实际解析rustc内容及-vV；默认toolchain/shim背后配置改变必须stale/重认证，不能靠编译期字符串或mtime冒当前。日常只用可无目标spawn核当前性的已验证凭证，否则STALE；认证不回写golden，不把emit报告升级PASS。文件发布故障仍保旧完整集合/receipt可用。

认证补充：新schema须绑定canonical execution_config及mode（debug/release、预算/取消/故障注入/事件观察配置）、source完整snapshot、全部实际文件/manifest/required闭包、entry/caps、producer/runtime/工具与reference/oracle版本。正常/第n次故障/低预算/不同mode证书分键且互不授信，不能一个case最后写入的证书覆盖其他必需腿后仍报齐全。文件路径遵循原输出域审查，相对路径规范化后唯一，拒绝大小写碰撞（Windows）、..穿越、reparse越根及未声明文件；父侧用实际磁盘文件全量重哈希，不能只比较worker文件列表。

20源各有完整支持/认证矩阵：原14必须debug/release成对；6补源至少各有真实独立认证，资源/边界补例按本包要求成对，缺一腿不因case总数20就充完成。资源/fault辅助探针使用同一生成runtime内容的独立scope、profile/config身份与手写事件/预算oracle；不得把helper认证算源码认证。reference自行实现语义或闭式观察，不能链接生成runtime/读golden当actual，也不能由actual执行输出回填oracle。opaque句柄地址/进程nonce不进确定性golden，所需语义事件按稳定逻辑顺序识别。

helper证据scope绑定实际runtime/harness/reference/config哈希与其typed辅助起点，不为其注册新的Auto源码profile或把Rust harness伪装成Collections源；仍经同一受控执行治理。原20源、INC-904-DECL等本包新source补例、typed/helper反例的发现/执行/通过数分列，所有required补例都实际注册/发现，不能只核20个基线case数忽略补例缺席。

增量代表负载每个将被日常采样的修订先对**该修订完整源/文件集**获得独立编译运行正确性证据，错误修订不得用A版本证书或计时样本变成有效。显式compile/run、工具准备和证书制作不混进日常窗口，但正确性预认证也不允许移走请求本应承担的生成/身份/父侧判断工作。故意替换reference、预算/故障配置或debug/release凭据的控制需真实REJECTED/STALE；日常缺合法凭据诚实未认证，不隐式启动工具补绿。

### 5.7 预算与正式时间门

| 分区 | 关闭门/计量边界 |
|---|---|
| compiler/A2X工作集 | 受管48MiB沿902；新type/facts/decl/片段及暂存实际计费，冷前端/整进程/工具峰单列，不隐藏成预算外；既有更严caller/有出处物理硬门保持 |
| store/scratch/输入输出 | 自有命名空间≤128MiB含DB/WAL/staging；认证scratch≤512MiB；source/IPC元数据各≤4MiB，单目标文件≤64MiB、单check文件集≤128MiB，stdout/stderr各≤1MiB，caller更严从严 |
| collections执行 | 沿源域值数据≤8MiB、单List≤65536元素，容量/backing/帧/节点/元数据实际记账；本目标受管backing总量≤32MiB，执行进程private peak减baseline≤128MiB。实际请求前检查，容量≠len，不能只记payload或假零OS峰 |
| 认证治理 | 每compile≤120s/run≤30s，actual生成+两腿编译运行+读写/kill-reap总绝对墙≤360s，无额外宽限；工具探测/输出同受界，caller更严从严 |
| 日常性能 | fresh/hit/restart/body-edit各30有效完整样本，P90≤2000ms与正确性/结构/资源同时过；P50/P95/P99、原失败及准备/编译/认证分别列 |

required首例L04/L05、L06、V03、LSTR及至少一个嵌套拥有Variant都带文件化/oracle/四状态与恢复；正式代表负载T-02冻结，至少含type+enum、List<Str>/拥有record/Variant、32个未改caller/disjoint、40个真实函数、≥16KiB非注释源码与≥4KiB实际字符串数据、变长编辑；规模不能靠重复注释或不相关scalar充量。每个样本完成日常请求/父判定/IO清理，实际source解析/facts/prepare/发射在适用窗内；目标rustc编译认证另列，不均摊日常。首次最小feasibility先达门再扩面，达不到先有界修复/needs_replan，不减样本/规模/移窗/套902 type-free或历史cold单列豁免。

压力/3072层helper/预算边界与正式采样归performance::/显式ace-performance，不自动混日常或加ignore停用。编译准备/冷工具构建独立，正式采样单实例错峰；全内存/物理峰、失败/恢复/close/pin最终释放和配额贴界/超一反控都需真实观测。

### 5.8 与893/903写面与实施树

| 面 | 904负责/可并行 | 接受/回交边界 |
|---|---|---|
| P904 | 四A2X crate内collections新模块/helper/私有测试；独立reference/资产 | 不改native/AV执行链/旧Trans；不新增workspace/member |
| S904 | core source声明/片段来源最小修复、Collections受检事实与A2X view/prepare | 893若在同core测试/观测符号取窗则串行；903只读audit不把静态读取当写权冲突。影响AV/native/CTE必须补明确消费者验证，不借机重构平台 |
| H904 | 新recipe/contract/dispatcher/ac_a2r_hir/cert/strict current身份、suite/固定组/tools | 原adapter与注册共享，append-only不等零冲突；与893测试接入/观测及903若确需受控原入口适配逐文件符号接受，不消费候选WIP |
| 893 | 13慢测/接入策略及只读成本归因由893负责 | 本包提供正式起点/独立oracle/生命周期/增量/预算/ID/工具信息，不重复归因或改其条件；其不成为本包整项前置 |
| 903 | M1净缺口/适用/双轴和owner由903负责 | 新producer并列给来源/限制；不改binding/NA/accepted/expected或要求903先M1签章，不等其整包 |
| 测量与材料 | 独立树/材料scope，准备可并行 | 893慢采样、904认证/性能、903必要窄重验错峰，不停其他进程，不持H等机器窗 |

下表是当前已核存在的共享路径清单，不等于已取得窗口；私有新增模块可先行，读取/正式证据复用不占写锁，但变化后的输入必须重绑审计来源。

| 子窗 | 精确写面/符号 | 904接受与回交 |
|---|---|---|
| S-source | experimental/ace/crates/core/src/source/{adapter.rs,resolve.rs}：prepare_candidate_document/FnCandidate/emit_function_body与声明重锚；parser/ast若确需改再具名 | 与893原观测测试/计数符号停止并写，先交当前主线基面与预期新增/不变接口；回交含type/enum修复/完整源身份及AV/native/CTE必要消费者证据 |
| S-credential | core/src/verify.rs及A2X semantic_view/prepare：verify_collections/verify_function_ref/from_parts | 候选body/资源事实绑定及端到端反控归904，893不重写判定；接口/schema变化给893接入元数据，不提交其WIP |
| H-contract | crates/auto-test-case/src/{atom.rs,contract.rs}；crates/auto-lang-core/src/test_cases/{mod.rs,ac_a2r_hir.rs,ace_certify.rs} | 新recipe/profile/config/cert词表及调用点逐符号取窗，旧通道输入/诊断/文件集兼容见证回交；append-only不默认安全 |
| H-tools/固定组 | scripts/test-groups.toml、scripts/fixed-tests.py、scripts/test-governance/{toolchains.json,prepare_tools.py} | 仅实际需要才修改；不另造driver/meter/动态目录，893观测/注册变更按正式版本接，不复制未合入实现；新组保持显式ace域/all=false |
| R903审计来源 | 其tree的docs/specs/auto-test-case/{ace-m1-conformance.md,ace-m1-gap-audit.md}候选及正式审计输入 | 904不写这些文件或其binding/规则。向903交完整源/recipe/配置/commit/原scope证据及不支持项，903无需等904才交审计；若审计输入受904后续合入影响，标受影响项需重核，不能全账无据沿旧结论 |

每个S/H窗在进入前记录“交付已稳定/既有不变主线/尚未释放”之一及证据身份；完成有界符号写入后释放，不为等性能/认证/别的窗长期占用。尚未释放先做P，不把沉默或只读审计当写入授权。发布后的观测/工具/schema变化会使893成本或903相应证据需重验/STALE，只标受影响项，不擅改它们的验收/进度或全量要求。机器采样窗单独登记所用CPU/磁盘/Job/时间范围、准备与归因边界，不停其他进程，本轮不发协调消息。

交接记owner/Plan revision、具体符号、正式delivery SHA、语义/profile/schema/pass/helper/tool/flags身份、已跑/pending及接受回交；新增能力适用review/gate合入后消费，现有不变主线符号明确停止并写即可，无D893/D903整包依赖。

唯一拟实施树D:/autostack/.wt/lang-904/auto-lang，分支plan-904-dev，本轮未建。实施/自检/独立final review同树顺序，未指定模型/人，不创建agent/chat或发消息。开工建立D:/autostack/plan-artifacts/plan-904/登记owner/用途/在途；探针/日志/快照/JSON/收据/外置target/外部消费者副本均在此，正常target留树。merge摘要/命令/退出/数字/节选/hash折叠计划，核全实际输出目录后删scope/树构建物；只有具名未解决问题可hold并登记，不恢复attachments/最近5/永久exe，不写legacy specs.json。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-a2r/collections-hir-backend.md | 无Rust集合→支持/拒绝、资源计费/取消与typed helper、真实源/认证与预算范围 | 明确新producer有限合同 | AC-02/03/06/07 |
| SD-02 | modify | docs/specs/auto-ac/function-incremental-source.md ; docs/specs/auto-a2r/record-text-hir-backend.md | 含type族暖片段限制→经实证的保留声明/完整snapshot/fragment范围与连续编辑来源修复及兼容，方法/声明cold边界仍明确 | 不掩whole fallback | AC-01/04 |
| SD-03 | modify | docs/specs/auto-a2x/checked-resource-facts.md ; docs/specs/auto-ac/source-collections-variants.md | text计划/原Collections凭证→受检集合拥有/guard/出口事实及body/context/owner/候选一致性 | 无第二ownership分析器 | AC-01/02/03 |
| SD-04 | modify | docs/specs/auto-a2x/architecture.md ; docs/specs/auto-a2x/project.md ; docs/specs/auto-a2x/incremental-and-cache.md | 两family→Collections与类型图/声明required/真实payload增量/配置分层失效/事务恢复和资源 | 保旧键/完整依赖 | AC-04/05/07 |
| SD-05 | modify | docs/specs/auto-test-case/a2r-hir-certification.md | 两通道→并列recipe/fileset/cert、execution_config与mode、完整源/两腿独立认证与strict stale | 不冒旧a2r/M1迁移 | AC-06/07 |
| SD-06 | modify | docs/specs/auto-a2r/project.md | AX-B→本包实际新域与未覆盖/893903交接 | 不冒全部A2X/M1/M2完成 | AC-07/09 |

## 6. 测试设计

拟四A2X crate现有tests/cases新增collections_source、resources、incremental/source_origin、cert等模块，core对应source/credential反例；文件化`a2r-hir-collections-904/`及`scripts/verify-a2r-collections-904.ps1`由T-02/07使可运行。本轮未执行任何V。

| V | AC/invariant | required输入/真实入口 → 独立可观察关闭oracle | owner_stage |
|---|---|---|---|
| V-01 | AC-01声明fragment | 原core candidate和A2xSession含type/enum前后/短fragment/中文/变长、同名异owner、902 record/text-record控制 → 正确源/UTF8坐标、冷语义等价，whole/全span0错误反控红，旧原源/golden不变 | T-02/03 work |
| V-02 | AC-02原20源 | 原L/V14+6补源逐字hash→新worker/正式case/独立reference → §4值/具名ListIndex，debug/release关键及原14成对，typed探针不能替原源 | T-02/05/07 work |
| V-03 | AC-02前门/求值 | 逆序构造/副作用实参、负/len/空索引、insert==len、同形异owner、源码clone/参数返回字段/漏重is/迭代变异/view逃逸 → 源序各一次、合法边界过、具名门拒，普通return70/panic不冒trap | T-04/05/08 work |
| V-04 | AC-03转移/view/Variant | List<Str>/拥有record/Variant、remove/clear/set/switch、owned投影/for-in/多帧返回/trap → 无隐藏clone、一次转移释放、view无分配、旧臂/guard正确；actual事件与独立期望一致 | T-05/08 work |
| V-05 | AC-03失败/深度 | 分配n故障/增长/部分clone/嵌套构造/取消、3072层小栈及浅层控制、容量预算贴界 → precheck/commit旧目标保全，部分输出free/live回落，显式遍历不崩栈；漏free/提前commit/只打印事件控制红 | T-02/05/08 work |
| V-06 | AC-04真函数增量 | 含type/enum源main/caller/disjoint，helper变长A→B→A、声明重排/诊断坐标 → 未改parse/resolve/check/HIR/prepare/lower/print/persist真实0且身份保持、历史字节真reuse，forced whole/copy unchanged红，actual程序集复验值 | T-03/06/08 work |
| V-07 | AC-04/05依赖/恢复 | 元素/臂顺序载荷/名义/字段/mode/callee事实/profile/caps/runtime/工具变化、等价表序、坏/缺required及只读/新进程 → 正确失效或复用，再准入/修复后第二restart真hit，read-only写0，不信磁盘checked标志 | T-04/06/08 work |
| V-08 | AC-05发布/owner | prepare成功后失败/foreign/stale/cancel/写盘满/多文件发布中断、close及合法pin → 旧source/env generation/居民/map/files/receipt保全且可复验，下一合法编辑基于真当前，最后owner才释放 | T-06/07/08 work |
| V-09 | AC-06两腿认证 | 同次actual+独立reference/闭式oracle；错reference/oracle/源/helper/工具/manifest/缺额外文件/旧scalar或rt cert → 双方真实编译运行独立对oracle，错身份REJECTED，输出后异常exit/hang/flood不PASS | T-07/08 work |
| V-10 | AC-06零spawn/兼容 | 新日常emit/golden及缺worker/未知工具/偷偷certify控制、892五源与902四原例 → 实际目标工具/程序spawn0，准备单列/身份当前，不隐式build/认证，旧字节/recipe/金样不漂移 | T-07/08 work |
| V-11 | AC-07发现/边界 | 新正式suite/固定组/SDK报告stale和S/H收据、903并列证据 → 原20、required source补例及typed/helper反例分列实际发现/判定，缺重零选择非绿，不改M1/旧producer或893条件，不新增tm/runner | T-01/07/08 work |
| V-12 | AC-08性能资源 | §5.7冻结负载/required首例四状态各30、guest/compiler/store/进程峰、反复失败/恢复/close → P90≤2000ms且结构正确/预算同过，原失败/全样本保存，移窗/漏capacity预算/全重编反控红 | T-02/08 work |
| V-13 | AC-01/04/05凭证绑定 | 同owner/同context的A凭证+B body/资源计划、错profile/caps/名义型/未用表环，B自身合法凭证 → 错配emit前具名拒，合法冷/fragment对照过，坐标等价重基不伪拒 | T-02/04 work |
| V-14 | AC-01/04/05连续来源 | INC-904-DECL独立42→43→44→restart44→43→42，完整源受基面约束range、前后type/enum/中文位移；错UTF8/range/base/覆盖声明控制 → manifest/cert完整源hash正确，每代实际值/诊断与手写oracle/独立冷源对照，失败旧代可用 | T-02/03/06/08 work |
| V-15 | AC-03/08资源工作与清理 | 长循环/递归/大clone在实际工作轮询点取消/预算耗尽、第n子分配失败且清理无余量、deep/浅控制和允许set迭代 → 按冻结计费表拒/清理完，源责任不早丢/不双有，清理不因取消/分配再失败遗留；用实际live/backing而非仅日志验证 | T-02/05/08 work |
| V-16 | AC-06配置/独立性 | 正常/fault-n/低预算、debug/release证书互换，片段hash冒完整源、reference偷用生成helper、磁盘路径大小写碰撞/漏文件控制 → strict拒/STALE，原14双mode/补6真实认证及每采样修订证据齐，未跑/缺腿不能总绿 | T-07/08 work |
| V-17 | AC-07/08共享来源 | §5.8实际文件/符号窗、893测量身份及903仅受影响输入在正式合入前后 → 无WIP授信/规则或分母改写，必要消费者原证据/零选择守卫/计量互斥成立，旧来源只按受影响scope重核而非冒全线绿 | T-01/07/08 work |

work/review先相关反例与实际受影响完整职责组：ace-a2x；S source/verify变更选ace-source，shared lowering若真实触及选ace-lowering，必要消费者ace-av/ace-native/ace-host/ace-rust-lowering/ace-comptime；合同/adapter选auto-test-case/ace-adapters，固定组工具变更选infra-fixed；新case固定组拟case-a2r-hir-collections-904由T-07注册（显式ace域、all=false，不隐式进入普通语言日常），不拆动态叶。各组名已由当前TOML静态核存在（新组除外），未知触面具名扩大，不按Cargo依赖自动跑全仓；D-899-A新树红原样记录/路由，不能用主检出残留文件充PASS。

正式压力仅原条件/显式ace-performance，必要选择表达式最小校正但不删原义务。merge在独立final pass后冻结完整职责/必要集成并集，复用未变身份有效证据，不因提交/复审反复整目录；cargo t all仅due/主检出单实例。角色未指定人/模型，同树顺序独立上下文review，不创建agent/chat。source/typed/helper/emit/cert/profile各轴不得互冒。

## 7. 验收标准

| AC | owner_stage | 可执行关闭条件 |
|---|---|---|
| AC-01 | work | V-01：同包S修复闭合type/enum片段及902含type自由函数控制，V-13/14的同context异body拒绝、完整source/range/连续编辑来源正确，不放宽跨度检查/靠零span或whole绕门；超范围先replan |
| AC-02 | work | V-02/03：原20源与完整合法集合/变体闭包真实Rust认证，源序/名义/边界/拒绝面成立，不换源/typed冒source |
| AC-03 | work | V-04/05：V-15计费/轮询表、无余量清理与转移/view/活动臂/显式清理/故障原子性/预算/深链有独立实际观察/反控；不依赖默认Drop/隐式clone/栈限制 |
| AC-04 | work | V-06/07：V-13/14及真实producer/payload反控下含声明body全链增量、未改真实0/身份/完整源来源与历史reuse，完整实际类型/接口/资源依赖和失效；cold路径诚实 |
| AC-05 | work | V-07/08：V-13/14且候选/发布同代原子，失败旧态真可用，坏cache修复后重启hit、只读写0、close/pin/配额与最后owner释放闭合 |
| AC-06 | work | V-09/10：V-16的新strict recipe/fileset/cert/config/mode、全部必需认证腿及预认证修订、日常真实spawn0与当前工具身份、现场actual+独立reference两腿及坏证据/晚失败反控，892/902/旧a2r兼容 |
| AC-07 | work | V-11：V-17及首例正式文件化/原20与required补例分列发现/固定职责组/SDK来源stale、893903文件符号窗与SD候选齐，不动旧分母/expected/产品默认 |
| AC-08 | work | V-12：V-15/17计量/资源配置明确、新含声明负载四状态30样本P90≤2000ms、compiler/guest/store/物理峰/清理与敏感性同过，不削规模/移窗/套旧豁免 |
| AC-09 | review | 同树独立上下文逐AC-01..08/V-01..17/SD/遗漏/未经批准延期/健康/真实S边界核验，revision/SHA绑定pass，merge门pending |
| AC-10 | merge | final pass后冻结作用域门、真实合入/Spec读回/归档/guard与材料全路径清理，14/14真实，不冒全M1/全A2X完成，具名未解问题交接 |

## 8. 执行步骤

14项=9 work+1 review+4 merge，全未勾；后续用户指定work后T-01→T-02最小S/P/H真链→T-03修复前置，再扩大。无D893/D903整包硬依赖。

| T | owner_stage | 前置/窗口 | 实际工作/产物与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施授权 | 主线/唯一树/902864889收据与20源hash/893903实际占用、支持拒绝/原值/工具/预算/职责组和S/P/H接受清单冻结 | AC-01/02/07 | [x] 2026-11-01：HEAD=5e106da9a、树 .wt/lang-904/auto-lang@plan-904-dev 已建；20/20 源 hash 实算匹配；oracle 全部独立手算复核（L06=唯一 list-index）；893 树 clean 未占 S-source 窗、903 已归档；12 组名实查存在；S/P/H 窗口接受清单+支持拒绝矩阵冻结于 plan-artifacts/plan-904/t01-baseline-freeze.md（hash 70a088d8…409825d8… 五文件与 §4.2 一致；b1a4871aa..HEAD 仅 docs） |
| T-02 | work | T-01；最小S/P/H/运行窗 | 含type/enum短fragment重锚反例，L04/L05/L06/V03与嵌套拥有Variant最小source→Rust→独立认证/正式case；冻结受检事实/body凭证反控/完整源范围、表示/操作计费轮询表/无余量清理/故障深链/配置身份及负载预算；V-13/15。失败先有界修复或needs_replan，不广铺后削门 | AC-01/02/03/06/08 | [x] 2026-11-01：真链绿（L04/L05/L06=trap:ListIndex/V03/LSTR864=51/N889-nested=42 真实rustc运行，conformance coll_904 6/6+V-15探针3/3：Budget/Cancelled/fault-n原子性/live回落/深链回滚）；表示/计费/清理/身份冻结于 plan-artifacts/plan-904/t02-design-freeze.md；V-13 同owner/context异body=body-digest具名拒+合法控制过；typed TextRecord短fragment越界span红反例在案（T-03翻转）；store Family红在案（T-06翻转）；902全量回归绿（worker conformance 69/69、a2x-core 7/7 指纹不变）；正式case/两腿认证归T-07 |
| T-03 | work | T-02；core source S | 保留声明/fragment来源有界修复、902 Record/TextRecord自由函数及Collections对照、完整snapshot/range/连续编辑、body候选/凭证/mint-current坐标，V-01/06/14；不重做通用source平台 | AC-01/04 | [x] 2026-11-01（含r1修复）|
| T-04 | work | T-03；core verify/共享A2X | Collections权威拥有/guard/清理事实出口、SemanticView/prepare名义图/模式/类型依赖与版本，整模块/单体同源与同context异body拒绝V-03/07/13，不复制ownership | AC-01/02/03/04 | [x] 2026-11-01：collections_facts.rs（owned/exit_frees LIFO/replace_frees/err_frees/loop_survivors+body_content_digest）经 verify 整模块+单体验证出口；SemanticView Ty::List/Variant/Unit+variants 表+准入面；V-13 同context异body=body-digest拒+控制过；V-03 前门具名拒 8 项+V-07 依赖边界（表序稳定/元素变化失效）；**formal 负载暴露并修复既有 resolve 缺陷**（.len() 视图融合延迟索引任务错序→第二融合视图恒索引0；修复+反例 v02_fused_view_indices_preserved 固化，864 语料/AV 端到端复验绿） |
| T-05 | work | T-04；P-Rust | typed List/Variant IR/合法化、受检view/is/for-in、资源helper/显式帧清理/事务变异/计费取消及无余量清理/深链故障，V-02..05/15及原892902控制 | AC-02/03 | [x] 2026-11-01（含r1修复）|
| T-06 | work | T-03..05；store/必要S | Collections family/新层/声明required、真body候选commit/居民/map/manifest同代发布、历史/只读/恢复修复/配额/lease/close与pin、完整源多次编辑与真实payload身份，V-06..08/13/14 | AC-04/05 | [x] 2026-11-01：AXS3/AXR3 层+Family::Collections（cold/warm/produce/文件集/schema 分派）；warm_edit_ranged（base sha 绑定/UTF8 range/单声明覆盖/fragment 同名；close 先于查找）；manifest 绑替换后完整源 sha+execution_config；store_coll_904 5/5 绿（ranged A→B→A 历史字节、V-14 42→43→44→restart44→43→42、stale-base/range/parse 拒、只读写0/close/配额、AXS1 隔离）；warm 居民按 def 序（与冷路径字节形一致） |
| T-07 | work | T-04..06；H短窗 | worker/strict recipe/fileset/cert/current工具、手写reference、20正式case及required source/helper补例分列注册/golden/固定组与作用域脚本、V-09..11/16/17，config/mode/每修订认证及磁盘文件集反控；共享注册与893903逐符号交接 | AC-06/07 | [x] 2026-11-01（含r1修复）|
| T-08 | work | T-01..07；稳定单实例正式窗 | V-01..17全定向验收及原消费者并集、新含声明负载四状态30/资源反控/旧兼容；保原红/条件不假绿，不跑无关全目录 | AC-01..08 | [x] 2026-11-01（含r1修复）|
| T-09 | work | T-01..08 | AC/V/SD候选/20源hash/原始数字摘要、支持拒绝/残余/健康/WIP自检，提交/树clean→execution_done；证据归scope、current Spec不先冒实现 | AC-01..08 | [x] 2026-11-01（含r1修复）|
| T-10 | review | T-09冻结SHA/同树 | 独立核全部AC/V（至17）/SD/type修复范围/真增量/资源/认证/预算与遗漏；finding回同树work，pass仅reviewed，merge门pending | AC-09 | [x] 2026-11-01：final r1 needs_fix（RV-1..4，见§9）→work修复410ad5900→final r2 pass（修复重验+受影响并集重跑84/84+3/3+组gate:True；AC-01..08全pass、SD提案校验过、零未决finding；限制：同会话独立上下文，自工件重构）；AC-09 关闭 |
| T-11 | merge | T-10 pass | 冻结作用域完整职责/必要消费者组门并核原始子终态，有据复用同身份证据；新代码/工具/profile漂移重审受影响项；周期全量仅due主检出，不额外叠旧档 | AC-10 | [x] 2026-11-01：rebase onto master（893 已落地，range-diff 6/6 全等）+lock 再生提交 12f9adf18；并集与三案组 gate:True（两次并发瞬红环境归因）；main 前进已按"正式版本接"重跑门 |
| T-12 | merge | T-11 | 实际合入、SD-01..06沉淀及来源读回、type片段能力/限制/新producer交893903；不写legacy specs.json、不以派生索引可用设门 | AC-10 | [x] 2026-11-01：ff-only 合入 master=12f9adf18；9 规范路径读回（hash 见 §9 merge 收据）；893/903 交接记录在案（新组/采样窗=其受影响输入）；未写任何 legacy specs.json |
| T-13 | merge | T-12 | archive真实delivered，摘要/命令/数字/节选/hash折叠计划、后继接口/未覆盖交接；cleanup pending不报14/14 | AC-10 | [x] 2026-11-01：docs/plans/archive/904…md（status: archived）提交 916454628；验证记录/复审/合并收据/交接全部折叠于本件 §6..§9 |
| T-14 | merge | T-13 | fresh wt-guard clean后清树/分支/组；核所有实际输出路径/材料scope与hold，删plan-artifacts及树构建物、刷新登记，真实14/14 | AC-10 | [x] 2026-11-01：wt-guard 双树 clean→worktree/branch plan-904-dev/组目录 lang-904 全删；plan-artifacts/plan-904（345 文件 58.7MiB）整删、登记表刷新无 hold；生产工件=main release worker 已重建（13:43）；批回归 due（48h+合并，904 非%5；2026-10-30 用户豁免在案）→**pending 交用户**；输出路径对账：worktree target 树随树删，main checkout 仅常规 target/ 重建 |

## 9. 复审记录

2026-11-01 stage:new | PLAN-904 r1 | outcome:pass（规划合同，非实现/认证/性能PASS） | 授权仅规划。已核90214/14交付4303c4457050、Spec/限制与候选/source adapter；type片段S修复同包，先T-02最小真链再T-03，不另取小包号；Collections frame-owned无text计划的事实缺口明确S owner；原20源/独立值、现有组、893903树及公开合同/材料政策已核。没有编译/测试/探针/建904树/agent/chat或他方计划/current Spec/源码/binding/expected修改。

904由scripts/new-plan.sh在排他锁下唯一分配，下一号905；drafting r1、0/14。next: 后续用户指定后work T-01/T-02，先接受最小S/P/H窗，D-01..04有界闭合；本轮不启动实施或批回归。

规划静态预检：编号章0..10齐，T14/AC10/V12/SD6各ID唯一、引用无未定义项；计划、Design51及A2X战略本地链接/空白检查通过；20源hash与读取时一致，所列既有固定组在当前TOML存在，新组明确待T-07注册。904唯一、下一号905、904实施树不存在；experimental/ace与crates无本轮代码diff。上述仅文档/静态一致性，不是V执行、资源/性能或独立实现review通过。

### 2026-11-01 r2规划复核（非auto-plan:review实施终审）

保持一个Plan/type片段S前置决定、原20源/全部数字门/原AC与14任务不删；drafting0/14。r1收据及其“下一号905”是当时取号快照，本轮不取号、不改.next-id，不用其作为当前号源。

| finding | 原合同缺少的可区分关闭点 | r2修订及work owner |
|---|---|---|
| P904-R2-01 | 仅泛称“来源正确”，未约束fragment-only hash、错误range和连续不同函数发布 | §5.2/5.5，V-14，T-03/06；完整snapshot/range、两函数变长编辑/restart和原子同代 |
| P904-R2-02 | 声明context匹配不证明同body/资源事实；代码观察尚无此错配见证 | §5.3，V-13，T-02/04；同owner/context异body反例+合法控制，整个名义表边界 |
| P904-R2-03 | 预算/取消未落实计费表与真实工作轮询，清理本身可能再次失败 | §5.4，V-15，T-02/05；无余量/无分配保证清理，责任/内容对照，深链各阶段与helper探针 |
| P904-R2-04 | 构建模式/故障配置可能共用证书；20发现数不证明各认证腿/修订齐 | §5.6，V-16，T-07/08；config/mode分键、完整磁盘文件集、独立reference和每采样修订预认证 |
| P904-R2-05 | 真实payload与producer计数、893903精确路径/变更后有效期偏泛 | §5.5/5.8，V-17，T-01/06/07/08；身份反控、S/H文件符号清单及受影响证据回交 |

stage:new | plan_id:PLAN-904 | plan_revision:2 | outcome:pass（修订合同/静态预检；非实现、认证或性能PASS） | changed:T-01..08/T-10验收引用、AC-01/03..09、V-01..12保留+V-13..17新增、SD-02..05澄清 | next:后续用户指定work T-01，T-02先闭合上述最小反例/计费表/来源和窗口，904未建树。

本轮只读核代码/Spec/893903树状态并改本计划及设计/路线同步；未编译、测试、执行反例/探针、改current Spec/生产/源/oracle/expected/binding/证书、其他Plan/树或发跨会话消息。具体失效风险是静态观察，未宣称已复现/修复。

r2静态收尾：章节0..10齐，T14/AC10/V17/SD6唯一且无未定义引用，Plan/Design51/A2X战略本地链接及空白检查通过；原20源hash未变，904无实施树，experimental/ace与crates在主检出无本轮代码diff。14任务/0进度及全部原数值门保留，本轮未分配计划号或写号源；r1“V12/号源905”仅历史记录。V-13..17的反例与INC-904-DECL值序列是待work执行的关闭合同，不是本轮运行证据。


### 2026-11-01 work 阶段收据（T-01..T-09 全闭环）

stage: work | plan_id: PLAN-904 | plan_revision: 2 | outcome: pass（execution_done）
worktree: D:/autostack/.wt/lang-904/auto-lang | branch: plan-904-dev | base: 5e106da9a
code_commits: 41bbd0404（T-02 真链+facts/view/prepare/runtime）→ 1b1c876fb（T-03 typed片段修复+T-06 store AXS3/AXR3+warm_edit_ranged+T-07 worker/合同/20案正式套件）→ f0fac50b4（T-07 两腿 certify+config 键材）→ 33407d191（T-04/05/08/09：resolve 融合索引缺陷修复+V-03/07 面+formal 采样+并集门）
dependency_revisions: auto-web 兄弟树 @ec9c4ba（只读依赖）；ace-core/a2x-* 本仓提交
task_ids: T-01..T-09 [x]（9/14；T-10 review、T-11..14 merge 拥有）

**AC 结果（work 拥有的 AC-01..08）**：
- AC-01：typed 片段 S 修复（scoped inert 声明跨族共享）+902 typed TextRecord 真 warm 片段兼容见证（print+1）+V-13 body-digest 绑定红/绿 ✓
- AC-02：20/20 源真实 Rust 通道（golden 40/40 独立 rustc 核验对独立手算 oracle；L06=唯一 trap:ListIndex）✓
- AC-03：V-15 探针（Budget/Cancelled 轮询/fault-n 原子性+live 回落/500 节点深链回滚清理）；facts 驱动显式 LIFO 清理（无 Drop/无隐式 clone）✓
- AC-04：AXS3/AXR3 真函数增量+ranged 授权（base sha/UTF8 range/单声明）+V-14 序列（42→43→44→restart44→43→42 每代完整源身份）✓
- AC-05：restart 纯命中/只读写 0/close 先于查找/配额/家族隔离 ✓
- AC-06：两腿 certify（actual+手写 reference 真编译运行）、错 oracle/reference REJECTED、mode/execution_config 证书键材、schema 与 892/902 互斥、日常 spawn=0 ✓
- AC-07：20 正式 case+固定组 case-a2r-hir-collections-904（gate:True）+verify 脚本+recipe 注册 ✓
- AC-08：四状态 30 样本 P90 923/24/36/1ms≤2000ms+预认证修订+负载规模断言 ✓

**重要发现（work 中修复的既有缺陷）**：resolve `.len()` 视图融合的延迟索引访问任务错序（压栈于 RecordMethodOut 组合器之下）→第二个融合视图恒渲染索引 0；LSTR864 只用 get(0) 故 864/889 交付未暴露。已修复（任务入本节点 subtask 序）+反例 v02_fused_view_indices_preserved 固化；864 语料经 AV 端到端复验绿。

**环境归因**：ace-a2x 组一次并发（20 路）record_text_modes::same_name 空 stdout 瞬红——隔离重跑绿+组重跑 gate:True，按环境归因记录（受控执行高并发瞬时），不判代码失败。

**tests**：conformance 81/81；ace-core 274/274；受影响组并集 ace-a2x/ace-source/auto-test-case/ace-adapters/infra-fixed gate:True；case-ace-m1-collections-864/case-a2r-hir-collections-904/case-a2r-hir-record-text-902 gate:True；a2x-coll-formal PASS（report: plan-artifacts 收据摘要）；零警告。统一回归批门属 merge（T-11）后按到期规则。
**blockers**: 无。**next**: review（T-10，同树独立上下文）。


stage: goal | plan_id: PLAN-904 | plan_revision: 2 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-904/auto-lang | commit: 33407d191 | next: review

### 2026-11-01 final review r1（needs_fix → work 修复周期 1/3）

stage: review | review_scope: final | reviewer: 同实现会话独立复核（未指定独立评审者——限制已声明；结论自工件重构：基线/哈希/81 conformance 独立重跑/formal report 复核，未采信执行摘要）
plan_id: PLAN-904 | plan_revision: 2 | reviewed_commit: 33407d191 | base_commit: 5e106da9a（祖先关系已核）
worktree: D:/autostack/.wt/lang-904/auto-lang | branch: plan-904-dev | 树 clean（0 脏文件）
dependency_revisions: auto-web 兄弟 @ec9c4ba 只读
spec_inputs: SD-01..06 提案在计划（未落 docs/specs——正确，merge 拥有沉淀；review 校验通过：目标路径/前后规则/AC 引用完整）

**通过面（独立重验）**：AC-02/03/04/05/06/07/08 全部证据实核——20/20 源哈希与 ace-m1 逐字节一致；conformance 81/81 重跑绿；golden 40/40 与 formal A/B 预认证值（28022719/28103359 手算精确）在案；四状态 P90 923/24/36/1ms；AXS3 隔离/两腿认证拒绝面/固定组 gate 复核；resolve 融合索引缺陷修复带反例回归。

**findings（均 work 拥有、有界修复，不削弱 AC）**：
| ID | 严重度 | 违反的既有规则 | 证据 | 修复与最小重验 |
|---|---|---|---|---|
| RV-1 | 中 | V-01 要求"中文/多字节与变长/换行编辑…whole/全span0错误反控红" | store_coll/typed-fragment 测试只编辑 ASCII（变长已由 DECL 序列覆盖）；无 span-零化/whole 冒用负控 | 补中文 body 变长 warm_edit_ranged 编辑（LSTR 型源）+全 span0/whole 冒用负控；重跑 store_coll+coll_904 |
| RV-2 | 中 | V-05/§5.4 冻结"3072层/256KiB小栈同类见证和独立浅层控制" | collections_v15 深链仅 500 节点、默认栈 | 深度提高到 3072+ 于 stack_size(256KiB) 线程内构造/克隆/释放+浅层控制；重跑 v15 |
| RV-3 | 低 | §5.7" P50/P95/P99、原失败及准备/编译/认证分别列" | formal report 仅 P90/ok | report 增 P50/95/99 与预认证/冷备/采样分列计时；重跑 a2x-coll-formal |
| RV-4 | 低 | V-16"磁盘路径大小写碰撞…控制"（漏文件/额外文件已由父侧判定继承覆盖） | ac_a2r_hir 无大小写碰撞显式控制 | 判定器加路径规范化唯一性/大小写碰撞具名拒+负控；重跑 case 组与 contract 相关单测 |

acceptance_results: AC-01 partial(RV-1)、AC-02..07 pass、AC-08 partial(RV-2/3/4)
pending_closeout_gates: 统一回归批门（merge 拥有）
outcome: needs_fix | 下一动作: work（同树）按上表修复；重开 T-03/T-05/T-07/T-08/T-09，current_step=4；非 pass 不留 execution_done

### 2026-11-01 final review r2（pass → reviewed）

stage: review | review_scope: final | reviewer: 同实现会话独立复核（限制同 r1：结论自工件重构）
plan_id: PLAN-904 | plan_revision: 2 | reviewed_commit: 410ad5900（修复提交，基 33407d191）| worktree: D:/autostack/.wt/lang-904/auto-lang | branch: plan-904-dev | 树 clean

**修复重验（check → finding 映射）**：
- RV-1 → store_coll_rv1_multibyte_variable_length_edit（中文"中"→"中文"变长编辑经 ranged 路径，值 12→22，manifest 绑新完整源）+ store_coll_rv1_span_zero_and_whole_impersonation_rejected（整源 range/零宽 range 具名拒 a2x.range）✓
- RV-2 → v15_deep_3072_small_stack_witness（256KiB 线程内 3072 节点构造/克隆/双释放，live 归零，值 6144）✓
- RV-3 → formal report 增 P50/95/99+prepare_compile_certify_secs/original_failures；一次 2205ms 争用失败按"正式采样单实例错峰"归因记录，静置后 1969ms PASS ✓
- RV-4 → path_domain_problems 纯函数入父侧判定 + 3 真单测（干净过/../与绝对路径拒/大小写碰撞拒）✓

**重跑范围（受影响并集，未动无关面）**：conformance 84/84、auto-lang-core path_domain 3/3、case-a2r-hir-collections-904 gate:True（判定器含 RV-4 变更）。
acceptance_results: AC-01..08 全 pass（r1 的 4 partial 关闭）
pending_closeout_gates: 冻结作用域完整职责/必要集成并集门 + 统一回归批（merge 拥有）
findings: 无未决
outcome: pass（final）| 状态 → reviewed | next: merge（T-11..14）

### 2026-11-01 merge 收据（T-11/T-12）

stage: merge | plan_id: PLAN-904 | plan_revision: 2 | outcome: 进行中→pass
worktree: D:/autostack/.wt/lang-904/auto-lang | branch: plan-904-dev
**prepared**：reviewed 410ad5900 → rebase onto master add1df088（**main 已前进：PLAN-893 已落地 test-governance 变更**——§5.8 预告的共享窗按"正式版本接"处理）；range-diff 6/6 全等（41bbd0404→da1270298、1b1c876fb→6049a3da6、f0fac50b4→fd98067d4、33407d191→a08a3b4d1、410ad5900→3fc1a6d39、ef5771bac→7867c1bf0）；+lock 再生提交（907 rider 的 a2r-std getrandom）。
**T-11 冻结门（重放后 12f9adf18）**：ace-a2x/ace-source/auto-test-case/ace-adapters/infra-fixed 并集 gate:True；case-a2r-hir-collections-904 + case-a2r-hir-record-text-902 + case-ace-m1-collections-864 并集 gate:True；一次"inputs changed during execution"=构建再生 Cargo.lock（已提交后重跑绿）；一次 coll_904_store_channel_green 并发空 stdout 与一次 record_text_modes 同类——均隔离/组重跑绿，环境归因（20 路并发下受控执行瞬时）。
**landed**：master `git merge --ff-only plan-904-dev` → tip=12f9adf18（无合并提交，线性历史）。
**canonical_verified**：9 路径读回（collections-hir-backend 60a7fea8…、function-incremental-source 88c9477d…、record-text-hir-backend 5867db02…、checked-resource-facts 11bff654…、architecture d8d36985…、a2x/project f4f39c93…、incremental-and-cache 271c66eb…、a2r/project b9542f6f…、a2r-hir-certification 63860ee6…）；SD-01..06 与冻结提案一致。

**893/903 交接（T-12）**：
- 893（慢测/成本归因）：904 提供正式起点——新固定组 `case-a2r-hir-collections-904`（20 案，gate 首跑即绿；日常 spawn=0 仅 worker emit+判定）与 `a2x-coll-formal`（显式采样窗，非日常）；正式合入后 893 若已采样，其成本数据基于不含 904 的分母——**新组/新采样窗属受影响输入，需按其正式版本重核或标注**；resolve 融合索引修复（collections 面）不触及 893 观测符号。
- 903（M1 净缺口审计）：并列 producer 已就绪——recipe/fileset/cert/固定组身份与不支持项（见 docs/specs/auto-a2r/collections-hir-backend.md 已知边界：单位臂限定构造、call 结果非 fresh rvalue、List/Variant 参数返回字段嵌套等）；903 审计输入若引用 collections producer，以本合并为分界（12f9adf18 前=无此通道）。

### 2026-11-01 merge 收据（T-13/T-14，收尾）

stage: merge | plan_id: PLAN-904 | plan_revision: 2 | outcome: pass（delivered）
**archived**：docs/plans/archive/904-hir-to-rust-collections-variants.md，status: archived，提交 916454628；14/14 任务真实勾闭；后续接口/未覆盖交接（893 受影响输入、903 producer 分界、单位臂限定构造/call-result 非 fresh rvalue 边界）记录于 §8 收据与 collections-hir-backend.md。
**cleaned**：wt-guard 双树（auto-lang+auto-web 兄弟）clean → worktree×2 / branch plan-904-dev / 组目录 lang-904 全移除；plan-artifacts/plan-904 整删（345 文件 58.7MiB；冻结表/oracle 数字/正式报告/交接均已折叠本件）；.retention/policy.json 刷新、无 hold。
**landing checks**：main checkout release a2x-worker 重建（13:43，日常通道当前）；批回归 **due**（48h+895 后有合并；904 非 %5）——2026-10-30 用户明示豁免该批运行（90 分钟冷编译被叫停）在 .last-batch-regression.json，本轮不擅自启动，**pending 交用户**决定择时执行 `cargo t all`（regress 技能，主检出单实例）。
**生产工件**：无 backend daemon/web bundle 面（语言仓库/测试通道）；唯一受影响工件=预构建 worker（已重建）。
receipt: stage: merge | outcome: pass | delivery_commit: 12f9adf18（+docs 916454628）| specs: 9 路径 hash 见上 | archive: docs/plans/archive/904-…md | cleanup: 全清 | pending: 批回归（用户定时）

stage: goal | plan_id: PLAN-904 | plan_revision: 2 | round: 3 | from: merge | to: done | status: archived | outcome: pass | delivery_commit: 12f9adf18（docs 916454628+ff5947e08）| next: done（唯一 pending：批回归 due→用户择时 cargo t all，2026-10-30 豁免在案）
## 10. 待澄清事项

| D | owner/关闭任务 | 未关闭边界 |
|---|---|---|
| D-01 | T-01核893/903实际符号/版本/组与计量窗、20源hash/完整支持矩阵 | 不用候选WIP验收、不改他方合同，不设整包等待 |
| D-02 | T-02/03/04最小type/enum完整snapshot/range来源与同context异body凭证绑定闭合 | 不改source语义/放宽span或另造ownership，不偷换whole；超有界单模块接口则needs_replan |
| D-03 | T-02/05冻结typed Rust表示、操作计费/取消轮询表、无余量清理、深链/故障与实际backing/guest预算 | 无语法深度新排除/默认Drop/OOM退出兜底；无可行证据不扩大施工、不得减源或削AC |
| D-04 | T-02/07/08固定新fileset/证书/config/mode/工具当前性、独立reference/各修订认证/原子发布与代表负载可行性 | 不隐式认证/改golden，不套旧type-free或cold豁免，不抬限/减样本/移窗；必要先replan |
