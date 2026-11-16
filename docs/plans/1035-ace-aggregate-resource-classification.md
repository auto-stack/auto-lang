---
plan_id: PLAN-1035
status: executing
feature_name: ACE聚合资源分类、字段投影与真实故障回归
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-16
plan_revision: 5
current_step: 1
total_steps: 6
depends_on: [PLAN-1024]
supersedes_spec_components: [docs/specs/auto-syntax/composite-source-projection.md, docs/specs/auto-av/collections-variants.md, docs/specs/auto-av/resource-fault-and-generation-conformance.md, docs/specs/auto-a2x/checked-resource-facts.md, docs/specs/ace-migration/m2-aggregates.md, docs/specs/auto-mir/owned-access-profile.md, docs/specs/auto-a2x/python-stdlib-readiness.md]
new_spec_components: []
touched_goals: []
affects: [ACE]
---

# [PLAN-1035] ACE聚合资源分类、字段投影与真实故障回归

## 0. 变更摘要

2026-11-13 r4当前：r3复审发现MutBorrow跨层表述可能抹平拥有槽/MutRet写回，以及故障统一Resource和epoch不变的笼统断言。已按现MIR ABI、915两seam的Infra及List代际/slot gen分域修订责任表与oracle，不扩大聚合/closure/clone支持。 本轮只改计划；drafting 0/6及原Task/AC/Case编号保留，无实现/测试/配置/canonical Spec改动或ACE执行。

承接[1024](1024-ace-m2-code-quality-and-refactoring-audit.md)的F30/F33/F34，优先级**P1（F30证据缺口P2）**。本轮仅立规划合同，drafting、0/6；未建树、未写实现/探针/fixture、未构建或运行ACE。用户当前仅授权复审与规划，不授权本包代码实施。真实动态影响由T-01反例界定，不把静态风险冒实际失败。

2026-11-16 r5阶段裁定：F30/F33/F34全部A-required，先关闭原RES-01/02/03及r4完整矩阵与对应独立oracle再供985/A；资源分类、Named字段融合、真实聚合故障与清理均影响A普通语言/FFI/owned合同；三个发现全部A，不只补一个漂亮fault例。原6任务/0完成/既定设计与历史保留，仅规划。

## 1. 目标

已准入聚合的语义owner、物理drop与借用责任各有明确且一致的消费规则；用户Named投影永远受控；真正中途失败保旧且回收半构造资源。

非目标：扩大语言/ABI/平台支持、默认切换、删除oracle、降低样本/规模/原条件、重写全ACE或接管其他Plan。F34仍是跨层静态差额，双free/错机器结果未观察；F30不否定旧Str fault覆盖，只补新聚合形态。

## 2. 架构方案

各域S/D/R定义与producer/consumer已由§5 DEC-RES冻结；只补既有aggregate资源闭包与Named融合参数门，不扩旧profile/clone、公共IR/ABI，不粗暴全record=owned。

沿1024真实producer→consumer分层，恢复不授信，source/typed、guest错误/Resource/Cancelled/Infra分账。不因共用文件扩大范围。§5已冻结跨层责任与最小纠正，work只选等价内部细节，不能改变profile/借用/ABI或临时加语言深度限制。

## 3. 技术栈

Rust、现固定组/原TestCase及Markdown。仅语言仓。后续唯一树D:/autostack/.wt/lang-1035/auto-lang，建议分支codex/plan-1035-dev，尚未创建；work/review/merge顺序使用同树，禁止链接。材料唯一D:/autostack/plan-artifacts/plan-1035/，创建即登记owner/用途/状态，当前无scope。

## 4. 需求分析与背景调查

授权：2026-11-13用户要求先复审1024，再逐发现分配到已有/新有界计划，明确优先级与1002关系，暂不改实现。无指定预算或自动施工授权。r1固定源Git=81d8fc03f10f4a7d179b0bc3a917602406ed7b91（历史）；r2历史取材Git=c2760324da25795594ab0cc01b581e3a306d2375，具体文件身份以本节SHA为准；1024 r2审查包与源码分开绑定，开工重核配置/输入/来源漂移。旧反例是静态证据，不是运行日志。

已读docs/specs/overview.md作导航；当前规则以module Specs为准，原始证据及测试入口如下：

| 已核源路径 | 字节SHA256 | 状态 |
|---|---|---|
| [experimental/ace/crates/core/src/source/resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs) | 80eb12467732dd657008918c90bf36158d52d6bbea63bc87e7c511a698263c10 | 静态取材，非运行证据 |
| [experimental/ace/crates/core/src/collections_facts.rs](../../experimental/ace/crates/core/src/collections_facts.rs) | 4ce74d59e6a5305ad1f2540cc5298ed077607a33208af58fe68f041e2421827f | 静态取材，非运行证据 |
| [experimental/ace/crates/lowering/src/hir_adapter.rs](../../experimental/ace/crates/lowering/src/hir_adapter.rs) | fa54e8c60ac15ff33c6663638f8c000ae951af0d19ddde9f598b5b4758f51761 | 静态取材，非运行证据 |
| [experimental/ace/crates/av-core/src/ownership.rs](../../experimental/ace/crates/av-core/src/ownership.rs) | 6be41a08ecf80af6a5c1acffe4df139ba77741850a568a44b4d4ba01807673c9 | 静态取材，非运行证据 |
| [experimental/ace/crates/av/tests/cases/aggregates_t01_probe.rs](../../experimental/ace/crates/av/tests/cases/aggregates_t01_probe.rs) | 9103e290ef8891f41d8acc7300c50e23daf145c47a1ad489d7d23b398bbe4552 | 静态取材，非运行证据 |
| [experimental/ace/crates/av/tests/cases/aggregates_v_rows.rs](../../experimental/ace/crates/av/tests/cases/aggregates_v_rows.rs) | bda3489be2e11390a1dd4deba752dcb9a3e49e9b797c5603b85ca6061a12adfe | 静态取材，非运行证据 |

| Spec来源 | SHA256 | 适用边界 |
|---|---|---|
| [docs/specs/auto-syntax/composite-source-projection.md](../../docs/specs/auto-syntax/composite-source-projection.md) | 529bc7eccd6f9028d4ad0224d5db86f66d36b54689e4e11ad1a50a44fa8bb891 | 当前规则保持；开工重读核漂移 |
| [docs/specs/auto-av/collections-variants.md](../../docs/specs/auto-av/collections-variants.md) | 5801fc0d16d7ee4099fdbf6e33fd995e65e74793f6f7d33a1cdb6c552df1cd1b | 当前规则保持；开工重读核漂移 |
| [docs/specs/auto-av/resource-fault-and-generation-conformance.md](../../docs/specs/auto-av/resource-fault-and-generation-conformance.md) | 1aeb5c68e65813e4671481d9c69b9446c32988c80b9612545e389df0ba82af95 | 当前规则保持；开工重读核漂移 |
| [docs/specs/auto-a2x/checked-resource-facts.md](../../docs/specs/auto-a2x/checked-resource-facts.md) | 38da0f0d66e92992b408741320616820856aed8111ec73dd3ec3d5f8249e31da | 当前规则保持；开工重读核漂移 |

源码/Spec是实际规则依据。1024 M/S/D/L/A/N/U为保留账，不代Rust reader/nextest发现；同文件在途修改串行单写，正式合入来源重核，不用他Plan WIP当已交付依赖。


### r2来源差额与执行入口预检

当前取材c2760324da25795594ab0cc01b581e3a306d2375；r1关键文件无差额的证据可保留，实际有差额者已回读并按本轮合同修正。预检发现：r1列了类型名但没有完整源、分类所指语义层与物理释放层、Named普通方法近邻或真实fault期间旧值断言；新MutBorrow事实Spec也需保留。

| 现有测试/补充实现入口 | SHA256 | 本轮用途 |
|---|---|---|
| experimental/ace/crates/av/tests/cases/aggregates_t01_probe.rs | 9103e290ef8891f41d8acc7300c50e23daf145c47a1ad489d7d23b398bbe4552 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/av/tests/cases/aggregates_v_rows.rs | bda3489be2e11390a1dd4deba752dcb9a3e49e9b797c5603b85ca6061a12adfe | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/av/src/resource_915.rs | b9c66c8f4813264fed38398e201832ce36b3e15837926305a16212b08ff5c087 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/core/tests/conformance.rs | 9d364af39988518abeb3a127fb9411f33135fc037a3a2463265eb0c404476b8c | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |

1028已由用户启动实施，本轮未写其计划或工作树；不要仅因其主检出frontmatter尚未更新就判断未开工。与它共享的计量结果在后续验证时绑定实际已合入SHA，本包不新增1028整包硬依赖。963/947等新源与本批修复共用位置时按正式合入基线串行重核，主检出其他文档WIP保留。


### r3补充依据与来源绑定

r3静态取材Git=6584e94d465dc1f8b2a1cd58344913de4d5ed18a；§4原r2 SHA表是历史快照。本轮重核原表及下列补充来源，当前字节身份见§9核验记录。以下是规则/代码证据，不是运行结果。

| 补充来源 | SHA256 | 作用 |
|---|---|---|
| docs/specs/ace-migration/m2-aggregates.md | 60c9fc5d70b3c8671b37a40f665362efebae82fc4702f9c321f4fadd8e8ff577 | 962新cap与旧面隔离、tuple/Box/四端能力边界 |
| experimental/ace/crates/core/src/verify.rs | cba93ead10ab8ab01ddbdd91e615453870d2fae7b851666c110fc5348baf3632 | HIR owned_elem/list_elem_owned与text-only分类职责分流 |
| experimental/ace/crates/av-core/src/contract.rs | e62668cf09ee300974f6357c33cc30dcc578278f32d085ea876d0cb8fa113abb | Rc record/List/Variant与ClosureRef/env实际表示 |
| experimental/ace/crates/av-core/src/layout.rs | e925bee804e1340315150eba218eee26e782e96d2aa3853fc2aa2000ebfe889b | List/Variant固定句柄与is_copy当前Str-only |
| experimental/ace/crates/native/src/collections_layout.rs | 370e0560c0e32bbc2a991fd6a8a19c8d732bb26021e7895ad0e1cc55c8f6f22f | 描述符身份/kind不是源ownership |
| experimental/ace/crates/collections-runtime/src/lib.rs | 8bbbb4aa477f111064a42fa43a9b8073084317defddf3947c90f0ff54c5b5364 | Cell refs/物理载体迭代清理 |
| experimental/ace/crates/av/src/resource_915.rs | b9c66c8f4813264fed38398e201832ce36b3e15837926305a16212b08ff5c087 | 现fault helper无Aggregates入口 |


### r4复审补充依据

基线Git=09119bd3cface39a8c794e54f9dacdd3f7dd4452，主检出D:/autostack/auto-lang，branch=master。本次按现Spec/代码重新构造反证；r3结论不作为证据。关键补充来源如下（SHA256为文件字节）：

| 来源 | SHA256 | 核查用途 |
|---|---|---|
| docs/specs/auto-mir/owned-access-profile.md | 479dfbb75bf344479332c08b5b9c0aac7b8a7964cfa0a23f712c38e5c337cddd | MutBorrow record在MIR为拥有槽/MutRet写回 |
| docs/specs/auto-av/place-ownership.md | 013a0baae3324884da8f403609adca5b4196e37804e96a20556bcba94f7d0607 | 内部slot gen不是运行期代际保证 |
| experimental/ace/crates/av-core/src/av.rs | 3e874b69f1b2cb7c05c1683315c54dd576e1db477b29c247a316d204bfd96c2e | slot gen先于fault、操作各自提交/kill边界 |
| experimental/ace/crates/av/src/resource_915.rs | b9c66c8f4813264fed38398e201832ce36b3e15837926305a16212b08ff5c087 | 两故障seam判fault-injected Infra |
| docs/specs/auto-av/closure-environments.md | 38bd861fb0e6c2a02987d988bacc6bb786b35924d1b5434afe62704dc4d08afe | closure资源域外字段/List元素仍拒 |

本轮授权与来源（2026-11-16）：用户只要求重冻985/A验收合同、核1024及审计后继和1066的A/B边界，明确任务/关闭条件并更新A顺序；不启动实施，不新增Plan。读取主线107b23d0f69c6ad11d036dc37bed0b688493db30、当前Spec导航/相关模块合同、1024 F01..35/S01..13、1027/1028归档收据、1029..1038和1066当前合同及实际源码消费者。静态风险不是已经复现的bug；A行须真实修复后验证或可复核充分反证，未执行保持pending，不能以本轮分类当PASS。原资源/平台/性能/ignore与UI三波边界保持。

## 5. 详细设计

唯一范围=F30/F33/F34。关闭行必须连接“发现→现代码不变量→反例→真实consumer→永久断言→修复提交”，承接登记不算关闭。T-01使§6全部case在原入口可运行。

数据/错误/责任：已准入聚合的语义owner、物理drop与借用责任各有明确且一致的消费规则；用户Named投影永远受控；真正中途失败保旧且回收半构造资源。 T-01先冻结各域定义与生产者/consumer；限定补既有aggregate资源闭包与融合参数门，不扩大旧owned profile或新增公共IR。若需改变跨核心合同，先独立设计并修约再实施；不能粗暴将所有record改owned。 confirmed/candidate/rejected/unknown沿1024分开；充分反证需保源码/合同依据，不能删分母。当前测试与新增反例分别保完整身份、源hash、expected、条件；共享机制不共享被测oracle。


### r4执行合同（继承r2输入，以下决定优先）

**输入到消费路径。** 走compile_and_execute_aggregates(src,"d_main")的真实source→Checked HIR→shared MIR→AV，native按其当前admit支持域独立跑；纯typed补充只核MIR ownership，不能代source。F34采用下表S/D/R三列，不能要求所有层owned布尔相等或全record=owned。

**状态、故障与责任。** RES-03复用915的ExecLimits::fault_after_allocs及StrArena::set_fault_at机制，现source_fault_sweep没有Aggregates分支，须新增测试私有aggregate前门接线（见下文）；记录每个实际attempt和两种seam分别的N，不能沿用旧Str的N冒新聚合。v03原正常42例保留并准确命名/注明其不注入；新增真fault断言不能仅对fresh frame两次成功。现MutBorrow参数/return shadow、exclusion与C资源事实新合同保持，修改分类不旁路凭证或简化物理drop。

**决定状态。** DEC-RES已在本节按r4修订闭合，T-01只建立真实设置/观察与源漂移复核，不再决定矩阵或扩域。
### r4跨层资源责任决定 DEC-RES（复审修订后设计闭合）

状态：2026-11-13 closed/design-static-r4；r3缺陷及改正依据见§9。先区分三个问题：S=源/受检类型的move与重复使用纪律；D=该表示实际必须回收的缓冲/载体/环境；R=哪种借用根及凭证允许读写。它们不是同一个owned布尔，后端不能按物理表示另推源语义。下表是在已准入profile中的规则；越界类型仍由原门拒绝，分类为资源不等于该profile已支持。

| 类型 / 表示 | S：source/HIR/MIR语义纪律 | D：物理释放与权威 | R：借用责任 |
|---|---|---|---|
| int/bool/char及numeric标量，Unit | Copy，无owner转移；保各位宽/位精确行为 | Collections facts=None；AV立即值、native相应标量无资源drop | 不因无drop自动变借用根 |
| 源借用str / HIR Str / MIR StrView | 借用，无缓冲drop权；不是owned Str | facts=None，借用者不free属主缓冲 | 保原root-set/外部参数provenance、范围与生命周期；不铸新owner |
| 源拥有Str / HIR StrOwned / MIR Str | move或显式已准入clone；owner转移 | facts=Str；AV StrHandle末owner报告free、native对应buffer释放、目标消费权威exit/replace/err事实 | Str可root；活view/mut阻冲突写/move，MutBorrow凭证保持 |
| 纯标量record（可嵌套纯值record） | S=false，隐式值Copy仍合法，不能全record=owned | facts=None不等于无物理对象：AV Record=Rc<RecordValue>、native collections record node均仍需末引用回收；C直接struct纯值无deep-free | MIR所有Record仍可root，即使S=false；RecordView为借用载体，不能取得drop权 |
| record含StrOwned/List/Variant/运行时Closure资源叶，或经record/tuple间接到达 | S=true；MIR type_is_owned与逻辑LayoutDesc.is_copy互补，不再只认Str。List/Variant/Closure即资源叶，不穿opaque边求无限展开 | HIR Collections facts按实际buffer/payload需要Str/List/Deep；record仅含Copy载荷Variant时facts可None，不能为了与S相等伪造free。AV/native沿实际子载体级联；closure环境属独立K owner | Record仍按根+字段路径借用；已有禁止view字段/逃逸与排他不变 |
| List<T>（即使T全Copy） | S=true，buffer责任转移；修复不开放List.clone | facts=List，按原LIFO元素清理再释放backing；AV handle共享只是宿主保活，不是用户语言隐式Copy许可；native node refs单独守恒 | List不加入type_is_rootable通用Str/Record集合；索引借用仍走Place/ElemView，绑定buffer/epoch/index/elem_gen |
| Variant（unit或全Copy载荷也包括） | 保现source/MIR S=true；投影是否转移按活动payload的语义分类，Copy payload不被误标已move | HIR facts仅当某payload含资源才Deep；C直值unit/Copy Variant无deep-free是正确差异。AV payload Rc、native tagged node各回收实际载体，unit无payload时不假造payload free | 保is/guard与payload-moved、switch提交后旧臂释放及guard失效，不把variant当通用root |
| 运行时Closure / 仅命名Fn引用 | 运行时closure按现S=true与捕获模式；命名函数引用/签名不是拥有环境值，不能一律owned | HIR Collections classifier对Function/Closure为None沿现边界：closure资源由K-domain/env store处理，AV ClosureRef是metadata，引用值drop本身不free环境；native环境由closure runtime/调用pin负责。C目标不支持的closure仍具名拒 | 捕获view/mut/move、外部根及环境pin规则保持；不将K-domain并入Collections重复free |
| Tuple / Box / 递归间接 | Tuple的S=任一成员S，降成匿名record；Box=单元素List胞故S=true；Node经Box/List间接合法，直接by-value record环仍拒 | 同已有匿名record/List表示；分类碰到List资源叶立即成立，不能跨opaque边递归展开或把合法Box环判非法 | tuple字段路径/Box元素沿原视图准入；whole-element view仍不得let/实参/push/return物化 |
| RecordView/MutRef/Place等借用或token（MIR补充行，不等于源MutBorrow record的降低表示） | 无payload owner转移；token本身的排他/退休不是拥有所指缓冲 | 按borrow_end/token/frame出口退役，不把借用值加入属主deep-free表 | 保外部根、活token、generation与跨帧约束；不放宽字段存放/逃逸 |

HIR 17种TypeKind均已纳入：8个numeric整数/浮点种类加Bool、Char为无资源标量，Str/StrOwned分开，Record/List/Variant按表，Function/Closure归最后的环境与签名区别。Unit与borrow token是其他层的补充表示，不冒HIR新增枚举项。

#### 支持域与操作冻结

| 入口 / 形状 | 本Plan决定 |
|---|---|
| 显式compile_aggregates_source / capability av.aggregates/1 | 保962已交付的List字段、嵌套List、tuple/Box及opaque合法递归。Bag只含List是必要正例，Node同时含Str+List只是已覆盖控制，不能用后者掩盖List-only分类缺口。 |
| 旧collections/closures及owned等冻结面 | 保各自原admit gate；不能因分类认资源就扩旧profile的容器字段/嵌套List，也不能因旧collections的禁令误拒Aggregates。源码准入、MIR准入、目标admit、发射、实际执行分别记证据。 |
| native及A2X Rust/C | 沿962真实支持矩阵：native arena-free形可执行，arena形现admit/COFF发射不能冒机器运行；递归Box native已有D-962-C2及C tuple调用实参D-962-C3等边界不在本包补能力，也不拿它们抹本包分类责任。A2X语义/清理事实变化必须跑两目标受影响消费者，原域外继续拒。 |
| clone | 当前source List.clone明确source.unsupported；record builtin clone仅SourceProfile::TextRecord，不将其扩到Aggregates。source正例用原已准入move、read/clear、参数/返回和Str clone；聚合深clone的资源/中途失败只在当前验证器本已准入的typed MIR/现内部clone执行入口验证，不能据typed绿声称源码新能力。 |
| Named get融合 | 普通list.get的现resolve_args接受Named并由单参value取index，故本包决定融合也接受Named（保现名字不额外校验的普通builtin行为），按Pos同一index类型/arity/求值顺序与一次性执行；去掉用户可触发unreachable。若将来要收紧标签名字，须另有语言合同变更，不在本修复夹带。 |

#### 最小修正与跨层owner（已决定）

source/resolve的live vt_is_coll_owned及资源record闭包判据、core/verify的collections规则owned_elem/list_elem_owned及其资源判据、lowering的hir_record_reaches_str/hir_elem_owned分类调用、av-core::ownership的record分支及layout的is_copy统一到各自表示的“record传递可达资源叶”规则。原仅Str查询若有确需字面Str的消费者仍保原义，资源分类调用不得再复用它偷换定义；不要简单把所有名字带owned的谓词改成同一返回值。遍历使用显式栈/visited及现工作预算，List/Variant/Closure资源叶即终止，不展开不透明递归；不得新增语言深度上限。派生结果来自验证过的类型图，不新增可信序列化owned字段、公共HIR/MIR schema、ABI或跨crate反向依赖。core/verify两处同义元素判据必须共享本core语义规则；text-record专用record_resource_map/TextResourcePlan仍保原冻结文本域，不全局重定义为新的Collections释放计划。HIR Str/StrSlice借用与StrOwned分开，当前view字段拒绝不放宽。VT::is_owned_coll目前rg仅找到定义，无仓内生产调用；不能冒作已使用前门或以其单测代live helper。若保留该公开helper，Tuple/record资源闭包须与live语义一致，但不新增consumer以证明修复。

CollectionsPlan/CollectionsBodyFacts的唯一生产者仍为core verifier：Str/List/Deep按D规则产生owned、pending、exit_frees、replace_frees、err_frees、loop_survivors；其现Record→List fixpoint已经可得Deep，不是待修漏分类。lowering/AV依受检类型实施S纪律和R约束，A2X Rust/C只消费现权威事实和凭证，不重新做ownership推导。native Cell.owned/refs及closure store是物理保活位，不得当源owned标志或将其当非法输入授信。

责任转移按两次边界分别记：RHS求值/显式Move可能先把原局部交给pending临时；操作自身再prepare/precheck→commit到dst/容器。MIR Move/Aggregate/ListSet的直接输入槽按各指令原合同在操作提交后kill；若它已由更早的Move从别处取得，后续失败不回滚整个表达式的早先转移。目标后端已接管的未提交输入由err_frees/helper按原责任释放一次，不能再给调用者补一份owner。替换旧dst的数据/属主buffer只能在新值成功提交后释放。frame abort按原清理所有剩余owner一次；不是异常后可恢复继续读整帧。

复审中1042已合入Python目标：当前Python D由A2xOwnedStr/A2xResourceHandle及生成finally显式清理，不依赖GC/__del__；lower_rt_fn_py等消费SemanticView/FnFacts，仍不能按Python赋值自行重推S或借资源分类扩旧准入。保原Python支持/拒绝/单次释放，ace-a2x固定组现已包含ace-a2x-python，属于分类消费者回归；此处不把Py run实际程序流混为emit元数据。

#### r4参数、写回及失败责任补表（已决定）

| 边界 | 责任合同 / 不可混用的表示 |
|---|---|
| source/HIR View、MutBorrow | ParamMode和exclusion/credential仍由core唯一裁决；HIR清理事实不把借用参数当另一个拥有者排free，replace责任和写回必须沿原mode计划。不得仅用“callee不拥有”推导所有MIR槽都是借用型。 |
| 现data/owned ABI的MutBorrow record | 依据owned-access-profile：callee MIR参数是move-in拥有record槽；callee返回t_MutRet_{fn}包source result及按mut参数位排序的wb字段。返回前这些槽的拥有责任转入wrapper，callee不能再free已转出值；caller按每字段实际分类FieldGet/FieldMove，wb回写原目标，wrapper必须完整消费。这是降低表示的临时拥有责任，非源参数模式变MoveOwn。 |
| collections/aggregates及A2X目标 | 现collections_fn_sig_for_def仅View record映射RecordView，其他mode走原类型walker，结果不是上述自动MutRet builder；不能把data/owned包装机制强套此域或声称新增List-only mut参数写回已支持。本Plan正例保962已承诺的View/MoveOwn参数与返回，MutBorrow沿原profile/type/target实际准入和现事实；新增聚合mut ABI能力不在本包。A2X只消费原facts/mode，Rust/C不重推ownership。 |
| List内容与Place六元组 | 对真实List precheck拒绝，旧数据、buffer id、epoch/elem_gen及直接输入槽按既有失败原子性保持；成功结构变更才按原规则更新代际。必须在对应List helper处观察，不把frame slot gen充当buffer epoch。 |
| 通用AV slot写入 | commit_slot_write当前在allocation fault前执行fr.gens[slot]++；保旧slot数据/计量不等于承诺slot gen回滚。place-ownership已明确内部gen bump不构成该profile运行期失效保证。本Plan记录slot gen实际变化，不新增全局代际回滚合同、不把它混作List内容epoch回退；若观测的是List helper，仍须满足上一行真实原子性。 |
| 915值/arena故障seam | fault_after_allocs与StrArena::set_fault_at的现注入均是ExecOutcome::Infra并带fault-injected辨识；真正SlotBytes等预算耗尽是Resource，取消保原Cancelled。负控不得要求人为fault变成Resource，也不能接受任意Infra作命中。两seam分别核attempt N/命中序号/ledger及无半成品。 |

MutBorrow ABI回归采用原已准入data/owned资源record/纯值record、结果含owned值与混合只读参数及前向callee控制，检查f0/wb分类、字段完整消费和各owner单次释放。聚合的List-only记录源/typed、目标支持/拒绝仍分列，不扩data/owned冻结面。运行时closure可为语义资源叶，但现closure字段/List元素的域外拒绝保持；不以矩阵行推出新的聚合closure支持。

F34结论：List-only record在source、lowering/MIR及layout存在资源闭包漏项；C/目标facts、pure record载体、Copy Variant与Closure None则存在有意层差异，不作为“全部布尔必须一致”的修复。静态差额已确认，双free/错机器结果尚未动态观察。F33按上面Named兼容修，F30仅补聚合真fault证据，不冒旧Str覆盖失效。


**范围与执行门。** 当前仍只规划；T-01/02/03分别按本节和§6执行，T-04逐Case核证，不把正确性留给merge。G1002只约束改写新作者格式/相关接线，不约束原入口反例。本轮§5 r4修订后的设计决定已闭合，新增策略是本Plan的拟交付合同；本轮不含实现授权，后续work按既定合同实施，不把关键边界再留到实现中任选。

### A-EXIT-20261116：本包在重冻985/A中的阶段（当前优先）

本包所列原发现/Case全部为A出口必须关闭的正确性/可信证据输入，**不改变已冻结语义、预算或测试规模**。A取原T-01→T-02/03实现/反证与永久例→T-04独立复审→T-05正式合入/Spec来源；T-06按原规约收尾归档，不能把仅分支绿当正式D。可用已审落主切片准备消费，但985/A最终必须绑定真实代码/输入/oracle/目标/工具/原条件及结果。当前草案/0完成保持；本轮没有证明任何发现已修复。

实现若发现主线已有修复，须保原反例/合法邻例、真实入口/事实和充分反证，经原review确认后关闭，不能为了“做修复”重复改代码，也不能用未复现/组总绿当反证。文档/G1002只在实际改写作者格式或相应adapter接线时消费；原入口/Rust回归不等1066全量迁移。UI三波只约束图形新增支持，已有非图形AV/AC/A2R及原C/Py支持腿/错误/资源回归不取消。
**本包绑定**：F30/F33/F34 → 原RES-01/02/03及r4完整矩阵 → T-01/02/03 → T-04/05正式来源 → 985 T-A03/A04。原因：资源分类、Named字段融合、真实聚合故障与清理均影响A普通语言/FFI/owned合同；三个发现全部A，不只补一个漂亮fault例。关闭谓词：遵r4 DEC-RES分S语义move、D物理释放、R借用凭证；Named/Pos合法投影无panic；List-only record资源闭包、MutBorrow原准入/写回和当前目标facts一致；真实聚合失败/旧dst/单次释放/N+1恢复闭环，详专条。

#### 1035特别验收：三个发现与失败类型不得互代

| finding / Case / 原任务 | A必须取得的区分性证据 |
|---|---|
| F34 / RES-01 / T-01/02/03 | List-only及经record/tuple可达资源叶的S/LayoutDesc/共享MIR纪律正确，合法返回/View/MoveOwn/move/clone拒绝/字段消费按原profile；纯值record、Copy Variant、AV/native载体与C值表示的D差异保留，不能强求所有owned布尔相等。MutBorrow只验r4原已准入ABI，不顺手扩新List写回能力。 |
| F33 / RES-02 / T-01/02/03 | 真source Named/Pos字段投影、合法近邻及越界原span；fuser参数前提完整，无合法程序panic，不能只测没人调用的VT helper。 |
| F30 / RES-03 / T-01/02/03 | 保原正常释放例，再用r4测试私有聚合seam核真实中途故障、旧dst/输入责任/ledger/次数/后续N+1；真正预算耗尽=Resource，915 fault-injected seam按原合同是带辨识的Infra，取消为Cancelled，不能把任意Infra当命中或把注入强改Resource。frame abort按原slot gen规则，不增加整体表达式回滚。 |

全部Case/目标事实、健康与原完整职责在T-04独立核，真实内容与源码/工具/commit绑定，T-05合入后才交985；没有“F30证据属P2所以等B”例外。原r4设计已决定，不重新立研究、不要求重新推一套ownership。

### 与1002及其他计划的依赖

- 本包调查、原入口回归与核心修复/研究不以1002为硬前置。只依赖1024审计交接；不等待1002全语料迁移或985终验。
- G1002=1002 T-03四例七检查及负控通过，T-04独立复审且读取/比较接口已正式合入并绑定SHA。只有新作者格式迁移或相关document/IR/adapter接线重构须等待G1002；Rust/旧case回归可先交付。
- 1002不反向等待本包全完。若本包真实反例影响其七行基线，只对受影响行具名先修/同源重跑，不抹真实红。
- 1017同路径计量/执行基线先协调冻结输入/构建身份，不等待整包、不擅跑ace-perf。能力计划只消费已审已合入修复，其M2义务不自动通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify（拟议） | docs/specs/auto-syntax/composite-source-projection.md | 现规则/支持域保持；补充以下修复后的明确合同：已准入List-field/opaque聚合的owner/drop/rootable职责分开且消费一致；Pos/Named融合复用同普通参数准入，无用户panic。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |
| SD-02 | modify（拟议） | docs/specs/auto-av/collections-variants.md | 现规则/支持域保持；补充以下修复后的明确合同：MIR move/field/clone/clear及borrow边界保现profile，不把所有record当资源或扩大List.clone。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |
| SD-03 | modify（拟议） | docs/specs/auto-av/resource-fault-and-generation-conformance.md | 现规则/支持域保持；补充以下修复后的明确合同：新聚合真实中途fault及旧目标数据/buffer保全、List epoch与slot gen分域，不能以正常两帧42作故障证据。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |
| SD-04 | modify（拟议） | docs/specs/auto-a2x/checked-resource-facts.md | 现规则/支持域保持；补充以下修复后的明确合同：保当前Collections/MutBorrow/参数模式事实权威和目标差异，不让分类修复重推ownership或改ABI。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |
| SD-05 | modify（拟议） | docs/specs/ace-migration/m2-aggregates.md | 保962显式cap与旧面隔离；明确资源record闭包及S/D/R差异，source List.clone/aggregate record builtin clone不扩大，目标既有admit/emit/run边界保留。 | 防误用旧collections禁令或typed成功扩源码支持 | AC-02/03/04 |
| SD-06 | modify（拟议） | docs/specs/auto-mir/owned-access-profile.md | 原MutBorrow record move-in/MutRet/f0+wb完整消费保持；补资源闭包分类消费者的回归边界，不改ABI/扩聚合支持。 | 防HIR借用与MIR临时拥有责任混淆 | AC-02/03/04 |
| SD-07 | modify（拟议） | docs/specs/auto-a2x/python-stdlib-readiness.md | 保Python现资源句柄/finally显式清理与原准入，作为共享分类事实consumer纳入回归，不依赖GC或复制ownership分析。 | 1042合入后的真实下游责任 | AC-02/03/04 |

本轮不改canonical Specs。frontmatter是拟议影响，review按实际交付收敛；touched_goals空，工程质量不擅扩产品目标。

## 6. 测试设计

拟执行完整受影响组：cargo t ace-source ace-lowering ace-av ace-native ace-a2x ace-a2c-abi。T-01按实际调用/修改冻结必要消费者并集，不能以单反例代完整职责组。 本设计纠正分类并追其Collections/MutBorrow消费，ace-a2x与ace-a2c-abi已纳入并集，逐目标保实际支持/拒绝，不让域外typed例代source。目标测试名/新fixture在T-01核原目录后使其可运行，当前均未创建。原样本/规模/平台/ignore/正式性能轴保持；merge复用同身份有效证据，不追加全仓/ace-perf。

| Case ID | AC / work owner | 区分性输入/设置 | 实际入口 | 独立关闭oracle | 当前状态 |
|---|---|---|---|---|---|
| RES-01 | AC-02 / T-02 | List-only record、Str+List、标量、嵌套/tuple-of-list/合法Box | source→Checked HIR→共享MIR ownership→AV/native | 支持域内按S/D/R核move/read/get/clear/原已准入clone及参数/返回；纯标量不误owned；旧域外继续拒绝 | 已实施并绿 2026-11-16（audit1035_res_01_*，§9） |
| RES-02 | AC-02 / T-02 | xs.get(index:0).code与Pos邻例；错型/多参/缺字段/owned与Copy | resolve的Field融合前置路径和真实source前门 | Named按普通get现准入与Pos等价；错型/arity具名诊断；无panic、无owned whole物化逃逸 | 已实施并绿 2026-11-16（audit1035_res_02_*，§9；修前 unreachable! panic 复现） |
| RES-03 | AC-02 / T-02 | 保v03正常42，再对新聚合构造/克隆/替换逐实际分配attempt命中原Infra注入 | 测试私有Aggregates前门→现ExecLimits/arena seam及真实AV consumer | 真实注入Infra确命中；旧目标数据/buffer保持，List epoch独立核；半构造回收；清故障重跑成功；N+1成功 | 已实施并绿 2026-11-16（audit1035_res_03_*，§9；旧代码同绿=F30 证据缺口而非缺陷） |

代表性路径/失败状态不冒全笛卡尔覆盖。发现/执行/原条件排除、准备/冷编译/发现/运行/清理、首次红/环境归因/重放绿分别记；父exit0不代子终态。当前未运行，无产品PASS。

角色：后续获授权work实施；review独立核证（无指定模型/另起agent授权）；merge收尾。使用同一实施树，本轮规划自检不冒实施review。


### r4具体输入与状态断言（全部待实施）

```auto
type Bag { items List<int> }
fn main() int {
    var xs = List<int>.new()
    xs.push(42)
    var a = Bag(xs.move)
    var b = a.move
    return b.items.get(0)
}
```
RES-01独立值42；另有Str+List、纯标量Pair、nested/tuple-of-list和现已支持Box，配已准入的move/read/clear/参数/返回；source List.clone及Aggregates record builtin clone继续拒，typed deep-clone单列，不冒source成功。

RES-02完整近邻沿现Item{name Str;code int}与xs.push(Item("ab",42))，对照xs.get(0).code和xs.get(index:0).code，独立值42；普通不带field的get调用作为实际参数准入参照，保源求值一次及顺序。错type/arity/缺field具名拒绝，不准unreachable panic。

| 原Case ID / 子面 | 实际设置/适用状态 | 独立关闭断言 |
|---|---|---|
| RES-01 | List-only、Str+List、纯标量、嵌套/opaque合法环 | 支持域内move/read/get/clear及原clone/call-return按各profile和降低ABI核责任；旧profile域外不因修复放行 |
| RES-02 | Copy/owned元素、Named/Pos、字段存在/不存在、单次有副作用index | 按现普通builtin支持面复用准入；若想收紧Named另修约；无panic、无重复求值/owned whole逃逸 |
| RES-03 | 新聚合构造/克隆/替换，arena与value两seam各逐attempt失败 | 确实命中fault-injected Infra；旧目标数据/buffer保持，List epoch与slot gen分列；半构造释放；N+1与清故障重跑成功 |

| 原Case ID | 拟新增永久测试名（当前不存在） | 既有源码/目标落点 | 使其可运行的owner |
|---|---|---|---|
| RES-01 | audit1035_res_01 | [experimental/ace/crates/av/tests/cases/aggregates_t01_probe.rs](../../experimental/ace/crates/av/tests/cases/aggregates_t01_probe.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |
| RES-02 | audit1035_res_02 | [experimental/ace/crates/av/tests/cases/aggregates_t01_probe.rs](../../experimental/ace/crates/av/tests/cases/aggregates_t01_probe.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |
| RES-03 | audit1035_res_03 | [experimental/ace/crates/av/tests/cases/aggregates_v_rows.rs](../../experimental/ace/crates/av/tests/cases/aggregates_v_rows.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |

一个Case可需多个同族函数，以上名字是最小锚，不代全部状态。优先扩已有test根/私有cfg(test)模块；若新建test文件，须在其真实conformance根或Cargo target接线，不能只创建文件然后把rg属性数当发现。T-03必须在实际固定组结果中核各锚点被发现/执行；平台/工具条件排除具名列出，不计PASS。


### r4矩阵与故障证据补充（未来实施）

| 原Case | 明确的owner / 断言 / 支持边界 |
|---|---|
| RES-01 | Bag(List-only)=S true、MIR owned true、layout.is_copy false、HIR facts Deep；Node(Str+List)控制仍true；Pair纯标量=false/true/None且Record仍rootable。另Record<Variant<Copy>>=S true但facts可None，不能只比bool全相等。分别核move后读拒绝、合法view读42及alloc/free，纯值隐式Copy保持；补程序化HIR ListGet读取List-only record且不经合法view consumer的负控，由core verifier受控拒绝，不能只修source和MIR却留HIR门漏项。 |
| RES-01 | Tuple<List>按匿名record与Box按List胞核；直接Node.next Node拒，Node.next Box<Node>在aggregates准入/AV保原正例；旧collections仍拒同形。native/A2X各行精确区分admit/emit/run，不把D-962具名能力债冒通过或顺手实施。 |
| RES-02 | xs.get(index:0).code应与xs.get(0).code同值42，Named标签按现普通builtin兼容不额外收紧；错类型/arity/缺字段具名诊断，index副作用只执行一次，owned元素保持借用投影，不能whole物化逃逸。 |
| RES-03 | resource_915::SourceFrontend现只有Collections/Closures，不能直接用其source_fault_sweep声称已覆盖Aggregates。新增测试私有compile_aggregates_source→lower/verify/ABC admit→execute_with_arena设置，复用现ExecLimits与arena故障seam；不新增公开recipe/worker frontend枚举或改旧915负载。 |
| RES-03 | 新Bag/Str+List/Box构造与替换在aggregate真实入口逐attempt注入；clone只走上表已准入typed执行入口，source不支持形继续拒。两seam分别discover N，再逐1..N命中fault-injected Infra及回收，独立预算负控才期望Resource、取消才期望Cancelled；N+1成功；保原v03两次42控制且说明不是fault。 |
| RES-03 | 原子替换的旧值/epoch保持须在真实提交seam失败后、帧尚活时观察；仅List precheck要求buffer epoch/elem_gen保持，通用slot gen实际记录不要求回滚；全帧abort腿观察零live/释放一次。必要的cfg(test)局部观察只读真实dst/ledger，不能另造模拟提交函数或用fresh frame两次成功替故障保持。 |

本设计将影响聚合分类及其事实消费者，门禁并集明确包含ace-source ace-lowering ace-av ace-native ace-a2x ace-a2c-abi；不再将两A2X消费者推给T-01任选。正式性能/旧平台和ignore条件保留，未跑/未支持逐项分账。

本轮A阶段交接：RES-01/02/03及r4完整矩阵的全部原合法/拒绝/失败后恢复与资源断言是A必需；原AC-01..06范围不减，独立复审/正式来源和当前A/B分类由985接收控制核，不另建Plan状态或测试调度器。当前均未执行。

## 7. 验收标准

- [x] AC-01 / work：T-01冻结真实源、支持/资源边界与前置，落实§5 r4 DEC-RES的S/D/R、MutBorrow分层、故障分类与提交/abort观察，§6反例及邻例可在实际入口运行，不私改expected。
- [ ] AC-02 / work：r4具体输入/每项状态断言及§6实际测试落点均落实；§6每行独立oracle已取得；缺口修复或有充分反证，修前/修后及同族边界证据齐，原义务不减。
- [x] AC-03 / work：消费者、原测试、恢复/重试/失败责任保持，完成适用门、Spec准备及1002依赖分流，未闭不冒PASS。
- [ ] AC-04 / review：逐项核r4独立预期、后续使用闭环与具名决定，不能仅信组绿；逐AC/Case/代码/Spec独立核验、遗漏/延后/workaround及健康/来源检查通过，结论绑定revision/冻结提交。
- [ ] AC-05 / merge：review后冻结并集门/正式合入、规范沉淀及归档真实完成。
- [ ] AC-06 / merge：所有输出路径对账，摘要折Plan，scope删除或具名持有，guard clean后清树/分支。

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [x] 1024交接、未来实施授权及当前源 | 按§5 r4 DEC-RES重核源漂移；建立Bag/纯值/Variant/tuple/Box等真实前门、Named近邻、测试私有aggregate故障接线及活帧提交观察；记录目标admit/emit/run边界。 AC-01  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 **2026-11-16 完成**：base=b043117b8 重核 §4 全部源/Spec 字节身份（漂移见 §9），四个真实前门（aggregates source→lower→verify_mir→ABC→AV）跑通反例红：RES-01 `audit1035_res_01_list_only_record_ownership` 红、RES-02 命名投影在 `resolve.rs` `unreachable!` panic、core HIR 负控双红；RES-03 两 fault 例在旧代码即绿（证明 F30 是证据缺口而非缺陷，未把静态风险冒失败）。 |
| T-02 / work | [ ] T-01反例设置与r4既定决定 | 按最小owner/drop边界修F34与融合前置参数门；原v03正常义务保留，新增真实聚合fault永久例，不重推后端ownership。 AC-02  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 **2026-11-16 完成**：commit=34c8fc04a（11 文件）。F34 资源叶闭包统一到 source resolve / core verify（两处同义元素判据合一）/ lowering HIR walk / av-core ownership+layout；F33 两处融合点接受 Named 且移除用户可触发 `unreachable!`；F30 增 `resource_915::aggregates_fault_sweep` + `aggregates_commit_fault_probe`（测试私有 aggregates 前门，不改 `SourceFrontend`/recipe/旧 915 负载）。11 个永久例在修复后全绿，回退分类修复后 6 红重现（见 §9 红/绿表）。 |
| T-03 / work | [ ] T-02 | 按类型/操作与失败出口追下游MIR/AV/native及受影响C事实consumer；记录真实值/ledger/旧dst/epoch与N+1，完成职责组和必要消费者。 AC-02/03  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 **2026-11-16 完成**：冻结并集门 `cargo t ace-source ace-lowering ace-av ace-native ace-a2x ace-a2c-abi` 全跑（结果见 §9）：ace-lowering/ace-native/ace-a2x PASS；ace-source 2 红、ace-av 1 红、ace-a2c-abi 1 红均为 base b043117b8 已存在（回退本包修改逐项复现），ace-av 另 1 红 `sys973_v03_process_spawn_capture_timeout_exit` 隔离重放绿=环境抖动归因；本包 10 个永久锚点在该组结果中被发现并执行。 |
| T-04 / review | [ ] T-01..03完成、冻结提交 | 独立核AC-01..03、§6/SD/漂移、遗漏延后绕行及债务；失败回同树work不缩scope；AC-04  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-05 / merge | [ ] T-04 pass | 核冻结并集且复用同身份证据，正式合入、规范/归档；正确性不留merge；AC-05  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-06 / merge | [ ] T-05 | 所有实际输出对账、摘要/删除/持有登记；wt-guard clean后清树/分支；AC-06  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |

同文件串行；跨计划调度见1024分配表。草案stage:new pass不授权开T-01构建/探针。

## 9. 复审记录

2026-11-13 stage:new / r1 / outcome:pass（可审阅规划，非产品/独立实施review） / next:获实施授权后work。new-plan.sh中央顺序取号，源/Spec路径与组存在，Task/AC/Case/SD明确；drafting 0/6。未建树/材料、改实现/测试/配置、构建/运行、提交/合入/tag/归档。


2026-11-13 stage:new / r2 / outcome:pass（执行合同预检，可从T-01开始；非代码/独立实施review） / next:后续实施授权和当时基线重核后work。r1列了类型名但没有完整源、分类所指语义层与物理释放层、Named普通方法近邻或真实fault期间旧值断言；新MutBorrow事实Spec也需保留。 已补§4真实差额、§5边界/故障设置、§6完整输入/独立oracle/目标落点、§8前三项定制操作、SD具体规则；原AC/Case/T-ID及6任务/drafting0进度保留。 DEC-RES：T-01给每种类型跨source/HIR/C事实/MIR/runtime的owner/drop/rootable矩阵及已准入域，选择最小纠正；只补既有aggregate闭包。若需改变公共IR/ABI或原借用规则，先正式架构设计并注册后修约，未决只阻依赖分类改动，不阻Named受控准入修复。 该具名决定在T-01关闭前不进入依赖T-02改动；不是把支持判断留给任意实现。

r2所读源Git=c2760324da25795594ab0cc01b581e3a306d2375；r1已存在的源码差额：

| 路径 | r1字节SHA256 | r2字节SHA256 |
|---|---|---|
| experimental/ace/crates/core/src/source/resolve.rs | 24441ead81f35f8f3928183edc3e371dd478162b7ba6ee531dffb782d7e3d0df | 80eb12467732dd657008918c90bf36158d52d6bbea63bc87e7c511a698263c10 |
| docs/specs/auto-a2x/checked-resource-facts.md | d79eb509e7cb8242297320638601ffbc7c4b5df0626a22911e0adb9acd733704 | 38da0f0d66e92992b408741320616820856aed8111ec73dd3ec3d5f8249e31da |

本次只写本Plan，无实现/测试/配置/Spec/执行改动；未建树/材料/运行/取号/提交/合入。1028不在修改集合，同机其他计划WIP保留。


2026-11-13 r2目标文档终检：本批11份计划原66个任务/AC编号和29个Case ID保留；本Plan原3个Case不删减。共核114条当前源/Spec/测试入口hash，本批26个拟新增测试锚点只作未来T-02落点，未创建/未执行。各Plan 0–10章节、围栏、本地链接、实际组名、r2具体输入/断言、原任务顺序/owner_stage及drafting0进度通过。1028计划字节hash与本轮入场一致，本会话未写其实施树；并发其他计划WIP保留。结束HEAD=9a7674009725913f06f44b8176ae20c8aad2c455，c2760324→结束HEAD的本轮源/Spec/组范围diff为空，无相关源码WIP。git diff --check与新文档空白检查通过；这些均为静态规划检查，不是ACE功能/资源/性能PASS。T-01真实源/输入/预期及下一步已明确；T-02仍须原T-01反例/决定完成。 DEC-RES为具名T-01决定，关闭前不修改其依赖部分；不是新增用户批准的限额/支持承诺。


### r3设计核清记录

2026-11-13 stage:new / revision:3 / decision:DEC-RES closed/design-static。回读962规范及composite_record_list_field_executes、box_indirection_legalizes_recursion_and_frees，确认旧collections禁止容器字段不能冒Aggregates禁令；补该权威Spec到frontmatter/SD-05。核core TypeClassifier已处理Record→List fixpoint、ClosureRef为metadata且环境另责、AV/native物理record载体需释放；这些不是统一owned布尔的依据。源码核record clone只在TextRecord、List.clone具名unsupported；915 SourceFrontend只有两分支，不能冒现helper覆盖Aggregates。补读core/verify两处元素owner判据同样依赖type_reaches_str，纳入F34最小闭包修正；text-record专用map保持原域。F34静态差额仍成立但机器错误未运行观察。当前HEAD=6584e94d465dc1f8b2a1cd58344913de4d5ed18a；原r2留T-01的矩阵决定为历史，以r3表为准。保3 Case/6 AC/6任务，drafting 0/6；只改本Plan，未写实现/测试/配置/canonical Spec或运行ACE、未建树/提交/合入。

本轮静态来源复核：本Plan原§4来源均重算SHA256，仅core/tests/conformance.rs自r2新增1028 observation_scope模块注册：旧9d364af39988518abeb3a127fb9411f33135fc037a3a2463265eb0c404476b8c → 当前b7eeae01b4360ec6537d1d45a3fdf1fa3ab2edf205ea55ed60c76c6002f0b964；git diff已核该两行，无资源分类规则改变。其余原来源字节与r2相同。本轮三Plan共绑定39个去重来源（源码/Spec/组配置）；已于r3静态终检重核，无来源漂移，历史r2 hash不冒当前执行证据。


r3静态终检通过（2026-11-13）：三份文档0–10章节、r3/drafting 0/6元数据、原18个Task与18个AC及11个Case编号、表列/围栏/空白、本地链接和职责组实际名称核对通过；39个去重来源SHA256与本轮冻结快照一致。git diff --check通过，experimental/ace、crates、scripts无源码diff；未执行ACE测试/构建或把静态检查记作功能/资源/性能PASS。当前只关闭设计决定，未来实施与独立代码复审义务保留。


### r4目标复审与同轮修订记录

stage: review；review_scope: targeted-draft（r3执行前设计合同）；reviewer_identity: 本会话Codex；independence_limit: 与起草同一会话，未另起独立上下文/agent，本轮从现Spec、实际源码和独立构造反证重建裁定，不以r3闭合声明作证。outcome: r3 needs_replan；用户已授权直接修订，现改为r4并复核设计项，非final实现pass、不将状态改reviewed。

reviewed_plan_r3_sha256=fdfb1805959c88e0b3a9ab55a10dda0d296bb06b1a36b898f2635b9a21257790；source/base_commit=09119bd3cface39a8c794e54f9dacdd3f7dd4452；worktree_path=D:/autostack/auto-lang（计划主检出），branch=master；未创建实施树。reviewed_contract_r4_sha256=fccf0e8fcf6a0ccdbf1ad2fb0e521a9e0371865003940d3a1fa9476e38ecd9dc（LF UTF-8，§0–8含frontmatter）；proposed_spec_delta_sha256=f064e492f3ef30a3d2f15c8afb3e6a7055a00e31bdaa8c246db27cc0f11de817（§5“规范增量”至§6前，LF UTF-8）。原§9 r1–r3收据原样保留但其相容/截止/资源结论已被本次具名修订替代。

| finding / severity | r3合同问题与可复核证据 | 同轮修订 / 关闭oracle与范围 |
|---|---|---|
| R-1035-01 / P1 | HIR MutBorrow不是通用MIR借用槽：owned-access-profile规定mut record的callee move-in槽及MutRet/f0/wb。collections_fn_sig_for_def又不走该wrapper builder；统一“callee不拥有/不free”可能误导分类和写回。 | §5分层责任补表、原data/owned mixed/View/MutBorrow/owned结果及前向callee回归；不把MutRet强套聚合、不扩closure字段/clone。RES-01/AC-01/02/03/04。 |
| R-1035-02 / P1 | 915两seam按fault-injected Infra判命中，不是一般Resource；List buf_id/epoch/elem_gen不同于fr.gens。commit_slot_write先bump gen后fault，且Spec明确该profile不以内部gen作运行期保证，r3全称旧epoch不变过强。 | 故障Infra/预算Resource/取消分账；目标旧数据/buffer与List precheck代际保持，slot gen记录实际不新增回滚；RHS早转移与操作commit、frame abort分开。RES-03/AC-01/02/03/04。 |

验证方法：rg符号/逐入口读取/Spec对照；Python -B -仅内存算式和文档结构/hash检查。Pratt两方向分别得到rule_valid=true/token_valid=false与rule_valid=false/token_valid=true；N=5000000时wire上界65001024<67108864、受控请求同驻留下界280000000>268435456；ASCII源1048577B的最小信封1048595B<8388608。以上是设计反证，不是运行decoder/compiler/worker，未分配巨包，未运行ACE/构建/正式性能。

acceptance_results：AC-01/02/03及其T-01/02/03实现/动态oracle=pending，AC-04独立实施代码review=pending；本次仅其设计边界/Case/责任/Spec-delta逐项核清。AC-05/06 merge/archive/清理=pending。原6任务/6AC及本Plan Case分母保持，drafting 0/6，不把本轮核清勾为T-01全部完成。所有scope内复审 finding已落实到r4合同，未新增实现债务豁免或以1002迁移代正确性。

依赖快照：1024 r2 reviewed 4/5（SHA256=4834e56b888635bcb68f82bab88d61da39edbdb2c8897dee2f084960641825e5）；1002 r7 drafting 0（SHA256=751c438a33864cd333d19c6aa198c3cfa86aa88cb4974472fb5ac3b3d0358dfa，主检出未提交文档身份）。G1002仍只约束新作者格式/相关接线，不要求本包旧入口修复等待其全迁移。当前源码/Spec/配置以本轮54个去重源字节hash绑定；历史receipt不冒当前产品验证。next：未来获实施授权后在原计划专属树执行T-01反例设置与基线复核，按r4合同进入T-02；不得照r3已撤销限制施工。


补充来源绑定（其余见三Plan §4/既有来源差额表）：

| 来源 | SHA256 |
|---|---|
| docs/specs/auto-a2r/collections-hir-backend.md | 7cffe09c0ae08b86ba200dc742f30020633f01ddb094a70384ad46ef0385597f |
| scripts/test-groups.toml | e8b55f0eeecc01bb4eb2c5d672a11c2113ee00a39aae8842d81b3f650f6c7f2f |


复审期间并发合入1042 Python通道：首次来源冻结时HEAD=6584e94d465dc1f8b2a1cd58344913de4d5ed18a，现按正式合入HEAD=09119bd3cface39a8c794e54f9dacdd3f7dd4452重核相关差额；不能将初始HEAD与后合入源混写成同一提交。contract只新增Py recipe注册/门，conformance新增Py模块及排序，ace-a2x组新增Python包，原数值cap与原R/C recipe保持。其余已绑定来源按hash逐项复核；本会话不修改/接管他Plan实现。

| 来源漂移 / 新消费者 | 当前SHA256 |
|---|---|
| crates/auto-test-case/src/contract.rs | de83e05a610ca64c14e138c76fd9a7beb18ff25243e5eeb0710d3c0dfbc0af0a |
| experimental/ace/crates/a2x-worker/tests/conformance.rs | 32fa9de0918496bf9d851c427936848602b8a7de0d2d180e307f3795604f35db |
| scripts/test-groups.toml | 1b432bb63abff7fef0222d18db33246b7ec25f15a4015c2393f0f4cb1a84b3ce |
| crates/auto-lang-core/src/test_cases/ac_a2py_hir.rs | c175327e5a4e92d98227b03140df2407e0effbabd326a0d614aa4b3085bfb3b6 |
| docs/specs/auto-a2x/python-stdlib-readiness.md | 930ab8b2fb6eb202f95afa2218aa4530836260d6bd83de411dbf97d2fcd0aa90 |
| experimental/ace/crates/a2x-worker/src/pyrun.rs | edad642fcc60ba21b337341ef29e7d3a9fa0f516d218dd463701bf4237276e18 |
| experimental/ace/crates/a2x-python/src/lower.rs | 37c47e24a989b9561abaf925f90fdfee2042cf252fc744a4f11f00abfec2f3c6 |


r4静态终检（2026-11-13）通过：54个相关源/Spec/配置及2个依赖文档与漂移重核后的绑定一致，结束HEAD=09119bd3cface39a8c794e54f9dacdd3f7dd4452。三Plan的0–10章节、r4/drafting 0/6、原18任务/18AC/11Case、表列/围栏/空白、本地链接、实际固定组，以及§0–8合同SHA和拟议Spec delta SHA逐项核对通过；git diff --check通过，当前实现路径无WIP。本会话只写这三份计划，并发1042等正式合入差额已重核，不将它们说成本会话实现。设计finding的合同纠正已落实，运行oracle、T-04独立代码验收与merge门仍pending；未执行ACE/构建/性能，不签产品或最终实现pass。

2026-11-16 stage:new/revision | PLAN-1035 r5 | outcome:pass（规划分类/静态合同，非产品或A出口PASS） | F30/F33/F34/原RES-01/02/03及r4完整矩阵明确A；6任务/勾选/0完成与既定设计保持；源/结果不得以旧审计自报替代 | next:原owner未来获实施指令后按现合同执行；当前仅规划，不建树/写代码/运行测试/合入/清理/恢复目标。

本轮规划静态预检（非产品PASS）：18份规划/路线/债务文档；13份Plan共92个原任务，任务ID/勾选/状态/完成数/整包依赖保持。985本轮162父行分类索引唯一、F01..35唯一且全覆盖（20条A正确性、6条A约束/证据、2条已交复用、7条B）；新增本地链接/锚点、章节、表格、阶段/任务/原Case映射与行尾核对通过，阶段图34节点36边无环，1066的A终验不再依赖B-only格式/项目/250编译任务。仅验收合同冻结；真实矩阵/adapter/新控制及全部新产品结果仍待原work任务，未运行/签A。未改源码/测试资产/canonical Specs/取号计数器，未实施/建树/构建/窗口/合入/归档/清理或恢复目标。

### T-01..T-03 实施记录（2026-11-16，work）

stage: work | plan_id: PLAN-1035 | plan_revision: 5 | worktree_path: D:/autostack/.wt/lang-1035/auto-lang | branch: plan-1035-dev | base_commit: b043117b8 | code_commit: f86894ce3（review 冻结提交，前序 34c8fc04a / 7d06f14e9） | dependency_revisions: 无（仅语言仓 ACE workspace；未改其他仓、未取跨仓依赖） | task_ids: T-01, T-02, T-03 | outcome: pass | next_responsible_role: review

授权：objective「完成计划1035」经 /auto-plan-goal 继承为 work→review→merge 全链授权；实现只在专属树写，计划簿记只在主检出。

**入驻重核（base b043117b8，字节 SHA256；与 §4/§9 历史记载的 r4 快照对比）**

| 源 | base SHA256 | 与 r4 记载 |
|---|---|---|
| experimental/ace/crates/core/src/source/resolve.rs | b282282074654ead07d948c85461792afb3762c2f9b7a7f0e6a00a195e7315f0 | 漂移（r4 快照 6584e94 时点） |
| experimental/ace/crates/core/src/verify.rs | 64de0d8290587300514a053b093a8bff2a582cc6ec2a85531ef41187ff37331f | 漂移 |
| experimental/ace/crates/core/src/collections_facts.rs | dac38ace978d739fe3e3a9f03c3783c8ab202ad912325e48b22ee4bc2c2993c2 | 漂移 |
| experimental/ace/crates/lowering/src/hir_adapter.rs | 097c791bb992735bba0a5fa0e7e8791c9b16f6d9a26f15574931eadd81129167 | 漂移 |
| experimental/ace/crates/av-core/src/ownership.rs | fbe208179c99a9a93d04aaa26e5fee36d3de18931bd6089d8203a2d4cfcd4dd7 | 漂移 |
| experimental/ace/crates/av-core/src/layout.rs | 5c4bba8a2b0b4af013eee82700f2af47f1c76b20df5f28f03616ee6fd0c1fc63 | 漂移 |
| experimental/ace/crates/av/src/resource_915.rs | b9c66c8f4813264fed38398e201832ce36b3e15837926305a16212b08ff5c087 | 与 r4 一致 |
| experimental/ace/crates/av/tests/cases/aggregates_v_rows.rs | bda3489be2e11390a1dd4deba752dcb9a3e49e9b797c5603b85ca6061a12adfe | 与 r4 一致 |
| experimental/ace/crates/av/tests/cases/aggregates_t01_probe.rs | 151e679e5de4fde4d4e167e41ca789b9b1d958d0778094fd1b222d2a552da049 | 漂移 |
| scripts/test-groups.toml | 14b2ef9db5ca04f49126e7559b584a0c4ab9ef97d6326eb20d5efd6b6508e3f0 | 漂移（组名不变：ace-source/lowering/av/native/a2x/a2c-abi 均在） |

漂移来自 11-13→11-16 之间其他计划的正式合入；按现码逐条复核，r4 DEC-RES 的责任边界（S 语义 move / D 物理释放 / R 借用凭证）在当前代码仍成立，未改约定。

**实现（commit 34c8fc04a + 7d06f14e9 + f86894ce3，11+3+2 文件）**

| 层 | 位置 | 改动 |
|---|---|---|
| source | [core/src/source/resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs) | `record_reaches_str` → `record_reaches_resource(records, tuples)`：字段闭包到 owned Str/List/Variant/Function/Object 或 tuple 元素即资源；`vt_is_coll_owned`/`VT::is_owned_coll` 改用之（旧 live helper 与公开 helper 同义）。F33：两个融合点（`xs.get(i).f`、`xs.get(i).len()`）接受 Named 索引，去掉用户可触发 `unreachable!`。 |
| HIR verify | [core/src/verify.rs](../../experimental/ace/crates/core/src/verify.rs) | 两处同义元素判据合一为 `list_elem_owned`；新增 `type_reaches_resource`/`record_reaches_resource`（record 字段闭包到资源叶）。**保持 962 面**：裸 List/Closure/Object 元素仍是链式 view，不判 owned。 |
| lowering | [lowering/src/hir_adapter.rs](../../experimental/ace/crates/lowering/src/hir_adapter.rs) | `hir_record_reaches_str` → `hir_record_reaches_resource`（同叶集），`hir_elem_owned` 用资源闭包判 record 元素。 |
| MIR | [av-core/src/ownership.rs](../../experimental/ace/crates/av-core/src/ownership.rs)、[layout.rs](../../experimental/ace/crates/av-core/src/layout.rs) | `record_reaches_str` → `record_reaches_resource`（List/Variant/Closure 叶即真）；`type_is_owned(Record)` 与 `LayoutDesc.is_copy` 同源派生。 |
| 故障证据 | [av/src/resource_915.rs](../../experimental/ace/crates/av/src/resource_915.rs) | 新增测试私有 `compile_aggregates_module`、`aggregates_fault_sweep`（同 sweep 体，不改 `SourceFrontend`/`worker recipe`/旧 915 负载）、`aggregates_commit_fault_probe`（PLAN-919 lifecycle 观测真实 dst：replaced/swept/free id）。 |

**红/绿判别（同一永久例；红=回退上述分类修改后重跑）**

| Case / 永久锚点 | 修前 | 修后 | 判别性观察 |
|---|---|---|---|
| RES-01 `audit1035_res_01_list_only_record_ownership` | FAIL | ok | List-only record 不再被当纯值：帧账闭合（live 0、allocs==frees） |
| RES-01 `audit1035_res_01_move_then_read_rejects` | FAIL | ok | move 后读走 MIR `mir.use-after-move`（旧分类下无 kill） |
| RES-01 `audit1035_res_01_str_plus_list_and_scalar_record` | ok | ok | 纯标量 record 仍隐式 Copy（不误 owned） |
| RES-01 `audit1035_res_01_tuple_of_list_is_owned` | ok（tuple 早已 owned） | ok | tuple 成员 owned 纪律保持 |
| RES-02 `audit1035_res_02_named_get_projection` | FAIL（panic） | ok | 修前 `core/src/source/resolve.rs:12820` `internal error: entered unreachable code: get takes one positional index`；修后 Named 与 Pos 同为 42 |
| RES-02 `audit1035_res_02_named_neighbours_reject_with_diagnostics` | FAIL（同一 panic） | ok | 错型/arity/缺字段各有具名诊断，无 panic |
| RES-02 `audit1035_res_02_named_whole_element_stays_a_view` | ok | ok | Named 不放松 whole-element 物化禁令 |
| core HIR `list_only_record_element_is_owned_at_the_hir_gate`（含纯标量对照） | FAIL | ok | 程序化 HIR 裸 ListGet：List-only record 拒绝（`verify.view-escape`），纯标量 record 放行 |
| core HIR `nested_list_only_record_chain_is_owned_at_the_hir_gate` | FAIL | ok | 嵌套 record 链到 List 仍 owned；纯标量链放行 |
| core HIR `record_with_variant_field_is_owned_at_the_hir_gate` | —（修前同门漏项） | ok | 仅含 Variant 载荷的 record 也 owned |
| RES-03 `audit1035_res_03_aggregate_fault_sweep` | ok | ok | **旧代码同绿**：F30 是证据缺口而非缺陷；两 seam 逐 attempt 命中 fault-injected Infra、live 0、free id 唯一且 frees==successful、N+1 与清故障重跑均 42 |
| RES-03 `audit1035_res_03_commit_seam_keeps_old_destination` | ok | ok | 实测 clean: replaced≥1；每个故障 attempt：`replaced==0`、旧目的 buffer id 存活至 abort 且只释放一次、账平、N+1 重新提交 |

**组门（冻结 code_commit f86894ce3）**：`cargo t ace-source ace-lowering ace-av ace-native ace-a2x ace-a2c-abi`（run id run-1zd2ia6g）

| 组 | 结果 | 红的分账 |
|---|---|---|
| ace-source | 804/806 pass | `plan981_grammar_scope::{p981_age_package_grammar_id_is_frozen, p981_grammar_ids_match_the_recorded_frozen_values}`：回退本包修改仍红（grammar hash pin 漂移，非本包） |
| ace-lowering | PASS | — |
| ace-av | 1312/1314 pass | `stdlib_list_946_grammar_face`：回退仍红（packages 前门语法 pin，非本包）；`sys973_v03_process_spawn_capture_timeout_exit`：隔离重放绿（环境/时序抖动归因，非回归） |
| ace-native | PASS | 首轮 `mn_aggregates_collections_lane_admits_and_emits` 红由 HIR 门过度扩大引起，收窄后绿 |
| ace-a2x | PASS | 含 ace-a2x-python（1042 合入）分类事实消费者 |
| ace-a2c-abi | 8/9 pass | `admitted_text::runtime_admission_preserves_bytes_and_the_certification_pin`：回退仍红（C 文本 certification pin 漂移，非本包） |

ace-av 组结果 JSON 中本包 10 个 `audit1035_*` 锚点均被发现并执行（`results.json` 可复核）；core 侧 3 个锚点在 `ace-source` 组同文件内执行。

**范围与观测边界（不冒 PASS）**

1. **List epoch / 通用 slot gen**：故障注入在提交前返回 Infra 并整帧 abort，运行期 buffer epoch 与 `fr.gens` 没有外部观测面；本包只断言可观察面（无 `replaced` disown、旧目的 buffer 身份存活至 abort 且释放一次、账平）。List precheck 先验证后触碰仍由 915 R03 stale-place 成功路径套件覆盖，未新增 epoch 观测 API。
2. **Box 合法例补 `.move`**：`Node{next Box<Node>}` 在 F34 后属资源 record，`Box(tail)` 的隐式转移被 `source.move-required` 拒绝；V-04 fixture 与 t01 probe 内联源改为 `Box(tail.move)`，预期值 42 与资源账不变（同例 `Node(1, cell.move)` 一直如此）。纯标量 record 的 `Box(p)` 保持原样。这是分类修正的应有后果，不是支持域收缩。
3. 未执行：native/A2X 对 Box 递归的发射/运行腿（D-962-C2/C3 既有能力债，不在本包补）、聚合 MutBorrow 新 ABI、clone 深拷贝源码能力、UI/AGE/正式性能轴。
4. 首轮组门暴露的 HIR 过度扩大已在本包内回正（7d06f14e9）：裸 List/Closure/Object 元素不得判 owned，否则 962 的 chained whole-element view 与 native 聚合 lane 误红；该边界已由 `nested_element_view_read_runs`/`p946`/native lane 永久例守住。

**测试范围与时长（test-discipline：daily 用 debug/增量，未跑 release/ace-perf）**

| 范围 | 用例数 | 实测时长 | 说明 |
|---|---|---|---|
| av udit1035 过滤（RES-01/02/03 永久例） | 10 | 0.07s | 单进程 --test-threads=1；单个新增用例均 ≪10s |
| core ggregates_resource_1035（HIR 负控） | 3 | 0.04s | 同上，含程序化 HIR 变异构造 |
| cargo t ace-source | 806 | 7.05s | 2 红=base 已存在 grammar pin |
| cargo t ace-lowering | 全组 | PASS | 新增零用例 |
| cargo t ace-av | 1314 | 63.89s | 1 红=base 已存在 packages pin；1 红=环境抖动（隔离绿） |
| cargo t ace-native | 298 | PASS | 首轮含本包回归，收窄后全绿 |
| cargo t ace-a2x | 全组 | PASS | 含 ace-a2x-python 分类消费者 |
| cargo t ace-a2c-abi | 9 | 0.02s（meter wall 0.75s） | 1 红=base 已存在 certification pin |

准备/发现/执行：组门由 cargo t 一次性完成 discovery+run（日志中 per-group Summary 可复核）；冷编译首轮 32.8s（debug），后续增量 2-10s。复用证据：T-03 冻结提交 f86894ce3 的并集结果即为 merge 前的同身份证据，未重复跑全目录。
stage: goal | plan_id: PLAN-1035 | plan_revision: 5 | round: 1 | from: drafting | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1035/auto-lang | commit: f86894ce3 | next: review

### 2026-11-16 stage: review（final scope；同会话非独立上下文，结论由冻结提交重建）

stage: review | review_scope: final | reviewer_identity: 本会话（与实现同一上下文；independence_limit: 非独立 reviewer 上下文，故不采信 work 摘要，全部结论由 reviewed_commit 的实际代码、diff、独立探针与组门原始结果重建） | plan_id: PLAN-1035 | plan_revision: 5 | outcome: **needs_fix** | worktree_path: D:/autostack/.wt/lang-1035/auto-lang | branch: plan-1035-dev | reviewed_commit: f86894ce3（工作树 clean，diff base b043117b8） | dependency_revisions: 无（仅语言仓 ACE workspace） | next: work（受影响 T-02/T-03）

**基线**：Plan r5；实现分支 3 个提交 34c8fc04a/7d06f14e9/f86894ce3；`git status` clean；实现期未写 `docs/specs`、`.autoos`、`docs/plans`（`git diff b043117b8..HEAD -- docs/specs .autoos docs/plans` 为空），未发布 canonical Spec、未动 ledger。SD target 7 份全部存在；当前字节身份与 r4 快照比对：`auto-a2x/checked-resource-facts.md`=23e28db325f25a6de07ac456b30fb192901959e74116b7bab3be5b5cc0d15757（r4 记载 38da0f0d…）、`auto-a2x/python-stdlib-readiness.md`=695a26c3ade487ba97c6244582094a5f2a5cb5f95d7e678ba6f67748e704e591（r4 930ab8b2…）为他人合入造成的身份漂移；逐条读现文，SD-04/SD-07 规则未变，无冲突。其余 5 份与 §4 r4 值一致。

**独立探针（review 临时用例，跑完即删，工作树恢复 clean；未提交）**

| 探针 | 输入/入口 | 结果 |
|---|---|---|
| 派生分类三事实 | aggregates 源码→`lower_checked_closures`，取 `t_rec_Bag` | `type_is_owned=true`、`compute_layout().is_copy=false`、`compute_body_facts().owned={0,1,2}`（Bag 局部在 owned 集中） |
| Variant-only 资源 record | `type W { t Tag; code int }` | 隐式 `xs.push(w)` → `source.move-required`；`push(w.move)` 运行 value=7、live=0 |
| List 字段经元素视图读 | `xs.get(0).items.get(0)`（xs: List<Bag>） | 被 MIR 拒绝 `mir.field-type`: "list-field-get of owned field type 3 (fields ride records; use LIST_REMOVE first)" — 962 既有边界，非本包回归 |
| Object / 闭包域 | `List<Object>`、`type W { f fn(int) int }` | 源不可表达：`no type named Object`；`function values are not admitted in record fields or container payloads` → 分类里的 Object/Closure 分支是防御性死路，非缺口 |

**验收结果映射**：AC-01 pass（源/支持域/前置与 DEC-RES 边界落实，反例红可复现）；AC-02 **partial**（见 R-1035-R1-01..04：§6 RES-01 三事实与 native/clone 腿、RES-02 单次求值 oracle 缺永久断言）；AC-03 pass（消费者与恢复责任保持：ace-lowering/ace-native/ace-a2x 组绿，ace-av 除 4 项 base 既存红外全绿）；AC-04 本记录（结论 needs_fix）；AC-05/06 pending merge。

| finding / severity | 违反的既有规则与证据 | 修复与关闭 oracle（最小范围） |
|---|---|---|
| R-1035-R1-01 / P2 / RES-01、T-02 | §6 r4 矩阵 RES-01 行明确要求 "Bag(List-only)=S true、MIR owned true、layout.is_copy false、HIR facts Deep"，但永久用例只断言 source/HIR 门与动态账；三事实当前正确（review 探针实测 true/false/{0,1,2}），无永久断言 → 任一层回退不会被现有套件抓到 | 新增 `audit1035_res_01_derived_facts`（建议落 lowering 或 av-core 测试）：断言 MIR `type_is_owned`=true、`LayoutDesc.is_copy`=false、`CollectionsPlan` 的 owned 含该局部（record 名按 `t_rec_Bag` 后缀匹配），并配纯标量 record 对照（false/true/不在 owned）。 |
| R-1035-R1-02 / P2 / RES-01、T-03 | 同一矩阵 "native/A2X 各行精确区分 admit/emit/run" 只在 AV 执行腿有证据；List-only record 无 native admit/emit 永久例（native 组现有聚合 lane 只覆盖嵌套 List/Box/grid） | 在 `ace-native` 聚合 codegen tier 增 admit+COFF emit 用例（不声称机器运行；D-962 arena 运行债保持具名），并记录 A2X 两目标对同形的既有 admit/拒绝结论。 |
| R-1035-R1-03 / P2 / RES-01、T-02 | 矩阵 "clone 只走上表已准入 typed 执行入口"：F34 后 List-only record 在 MIR 由 "copyable→CLONE 拒绝" 变为 owned（CLONE 可准入），`clone_owned_value` 会重建 List buffer；无永久 typed 深克隆例证明克隆独立且双方各释放一次 | 增 typed MIR CLONE 例（沿 `av_collections_semantics` 现有 builder 形）：克隆后改一份元素不影响另一份；两 owner 各 free 一次、live=0。源码 List.clone 继续拒不动。 |
| R-1035-R1-04 / P2 / RES-02、T-02 | §6 RES-02 要求 "index 副作用只执行一次、求值顺序"，现例只断言 Pos/Named 等值；Named 分支的 idx 表达式若被重复求值不会被发现 | 增结构性永久断言：同一程序的 `xs.get(index:0).code` 与 `xs.get(0).code` 产出的 HIR 形状一致（标签不进入 HIR，fused 节点只带一个 index 子表达式）+ 一条真索引函数调用的值/顺序例；在 Plan 记录该平台无可观察 index 副作用，结构性单次引用即其可行 oracle。 |
| R-1035-R1-05 / P3 / T-01 记录 | work §9 的源漂移表只列源码/组配置，未重记 7 份 SD target 的当前身份；其中 2 份已漂移（他人合入），诊断影响虽为零但记录不全 | 在 §9 补 7 份 Spec 当前 SHA256（含上述两份），并确认 SD-01..07 的 after 文本仍与现规范一致；若措辞需补（如 SD-05 明确 "资源 record 隐式转移仍拒、`Box(tail)` 需 `.move`"）则并入同一 delta 修订。 |

**非阻塞记录（非缺陷，供 merge/后续闭包）**：① RES-03 的 "List epoch 独立核" review 接受 work 的反证边界——故障注入即整帧 abort，运行期 buffer epoch 无外部观测面；可观察等价物（无 `replaced` disown、旧目的 buffer 身份存活至 abort 且只释放一次、账平、N+1 提交）已有永久断言，List precheck-before-touch 仍由 915 R03 成功路径套件覆盖。② `resolve.rs` 5 行无关 rustfmt 归一化属健康清理，非功能改动。③ 分组门 4 红（plan981×2、stdlib_list_946_grammar_face、a2c certification pin）已在 base b043117b8 复现，`sys973_v03_process_spawn_capture_timeout_exit` 隔离重放绿=环境抖动；均非本包回归，不充 PASS。④ Box 合法例补 `.move` 是 F34 的应有后果（资源 record 隐式转移本就被拒），支持域未收缩，需在 SD-05 文本中明示。

**测试与时长（review 复用 + 新增探针；不重复 work 的组门）**：复用 work 在 reviewed_commit f86894ce3 的 `cargo t` 结果（run-1zd2ia6g：ace-source 804/806、ace-av 1312/1314、ace-native/ace-a2x PASS、ace-a2c-abi 8/9）；review 新增探针 3 例（derived-facts/variant/chain）单进程合计 <0.1s，另有 2 次 `ace-core check-source` 源级探针；探针文件已删除、`conformance.rs` 已还原、工作树 clean。本会话未新增交付用例；无 >10s 用例。

**实现写权**：review 期间未改实现；上述 R1-01..05 全部回 work 于同一树修复，修后仅需重跑受影响用例（探针等价物转永久例 + 相关组）与受影响 AC 的重审，不要求整目录重跑。



## 10. 待澄清事项

DEC-RES经r4复审修订后无关键设计未决：S/D/R、MutBorrow在HIR与各MIR/目标表示中的临时拥有/写回、fault Infra/预算Resource及List代际/slot gen分域已定。F34动态严重度/真实N及失败提交观察仍待实施，不据静态差额声称双free；不扩962未交付的聚合mut/closure/clone或运行期失效句柄保证。原具名能力债仍归原owner，本包资源闭包与Named受控准入不能延期。**2026-11-16 work 更新**：T-01..T-03 已实施（详见 §9 实施记录），F34 的真实动态后果已观察（List-only record 的帧账/move 纪律/四层分类一致），未观察到双 free 或错机器结果；F30 仍无缺陷证据（旧代码同绿），只补永久真实故障例。drafting→executing→execution_done 3/6。

stage: goal | plan_id: PLAN-1035 | plan_revision: 5 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1035/auto-lang | commit: f86894ce3 | next: work
