# 共享A2X第二目标：Checked HIR→C与M1实施路线

> **2026-11-05 CA-E／ARCH正式立项（仅规划，旧快照保留为历史）**：[941最终组合验收](../../plans/941-ace-m1-expanded-and-architecture-acceptance.md)drafting r1、0/15，新增独立target/机制registry与acceptance/3父侧总门，旧313 required/批准hash/NA/accepted/future及source/expected不改；Q4各对独立oracle、完整Rust/C认证、真实函数级P0/IO0、无预热恢复首编辑/首坏修复、原资源/样本/P90与同PID HMR均有独立反控。只读基面5f6737467：929 D29a7d7007与937 Dfa35943ac已归档交付；934已reviewed待D，935/936 executing1步，938/939仍drafting。941私有登记/reader/oracle可先，最终联合取D934/935/936/938/939；公共S/N/X保持原owner，H全文件与正式采样分窗，不把reviewed/WIP当D。另静态核出原313能力token无非绿，但38旧A2R cert-gap仍被924现行守卫阻签，按CE-BASE-01显式保留、不得以新HIR证书替旧域或默撤守卫；验收系统交付不等M1签章。详[基线/38项/写面排期](../../plans/reports/ace-ca-e-arch-baseline-and-schedule-2026-11-05.md)。M2全部旧场景接管/disable仍是v0.6硬门，本轮不施工。

> **2026-11-05 CA-D正式立项（本条为当前快照，下列早期状态保留为历史）**：有限ABI/FFI桥拆[937 CA-D1共享事实/标量底座](../../plans/937-shared-foreign-interface-and-scalar-abi-bridge.md)→[938 CA-D2受检源码foreign标量](../../plans/938-checked-source-foreign-scalar-calls.md)→[939 CA-D3资源/句柄/同步回调](../../plans/939-foreign-resource-handle-and-callback-bridge.md)，均drafting r1、0/12、0/13、0/14，仅规划。937带797 native与新A2C真实consumer/source导出、独立C认证，保持不改core/source/shared lowering/a2x-session恢复，可与929/934主体并行；H整文件与正式采样仍需短窗。938取D937及929稳定source接口，934若已接管则消费其facts权威，939取D938及已交925/924资源底座；不无条件等935/936整波，也不绕开实际读到的实例/CT依赖。完整CA-D须三包齐，最终CA-E/ARCH组合签章仍未立项，不预占号。当前main122725e9a：929executing r2 10/14由934获授权收尾；934 header drafting1/13但已有执行授权，935/936 drafting0步；928/931/932/933已交付。未跑实现/测试，不改在途合同/旧expected/分母/NA；797仍candidate/descriptor3/原预算，v0.6 M1含C与M2替换旧引擎disable硬门保持。详[基线/拆包/排期](../../plans/reports/ace-ca-d-baseline-and-schedule-2026-11-05.md)。

> **2026-11-04 r2当前复核优先**：927已交付，928 executing4/12，929 executing1/14按用户裁定等D928；931–933 drafting r2，仅规划。932继承914有效typed策略拒绝为verdict，坏typed/协议资源才ERROR；旧911/host判面漂移具名修订。933先交独立材料，931/932私有修复可并行，最终恢复首编辑接929稳定接口。整波后仍有SQ/INST/CT、CA-D ABI、最终ARCH组合准入三条机制线未立项，933不是M1最后包；详[最新复核/写窗/距离](../../plans/reports/ace-931-933-plan-review-and-m1-distance-2026-11-04.md)。下列早期状态为历史。

> **2026-11-04后续优先级**：921/925/926已交付，C scalar/record-text/collections主体可复用，CA-D有限ABI桥前置已具备但尚未立项。新立931优先补Rust同形临时值过早释放，932补native暖准入/父分类，933补sema跟踪来源及904缺manifest；详[完整排期](../../plans/reports/ace-next-wave-correctness-and-admission-2026-11-04.md)。语法927合入→928→929按原owner继续；CA-D建议先核checked foreign/中立metadata及native+A2C真实消费者，再拆标量三腿/资源生命周期，不把ExternalFn兄弟签名或C内部struct当FFI。原797 candidate/descriptor3/资源门与M1必需身份不变，不在本轮施工。

> **2026-11-03 M1新增架构机制门**：统一语法正式拆927→928→929，见[机制调查/写面交接](../../plans/reports/ace-m1-architecture-gap-audit-2026-11-03.md)。925/926的C provider/runtime原职责与顺序不变，公共source/AST/type-enum/method片段/A2xSession/来源恢复按实际SHA单写；929消费已交付能力，复验restart后下一编辑与真实未改生产0。CA-E最终必须合并原门+C门+SYN/ARCH门，不能仅C/native已绿就签M1；SQ/实例/CT接线、CA-D及最终组合收口仍待立项。本轮只规划，不扩在途合同或施工。

> **2026-11-03最新CA-C/实施顺序**：CA-C正式为[926集合/变体资源闭环](../../plans/926-a2c-collections-variants-resource-closure.md)，drafting r1、0/14；承[925 CA-B](../../plans/925-a2c-record-text-resource-closure.md)资源底座，复用CollectionsBodyFacts，不拆List/Variant重复架构。原20源+全部补例/typed-runtime helper边界、P0/来源恢复/双侧真cl认证/预算时间均随包关闭。921已work done11/14待review，主序921复审合入→925→926；私有准备可先，公共A2X按stable SHA单写。native923→924与C主体并行，公共S/H与正式采样分窗。CA-D有限ABI桥得A/B稳定target/资源接口后可与926并行，不等集合全交付；CA-E最终需C A/B/C/D与旧M1/native三格/Q4组合门齐，D/E未取号。设计见[Design51 §15](../51-shared-a2x-and-hir-to-rust.md#15-ca-cc集合变体与近期实施顺序2026-11-03target-state)，下文旧“CA-C未取号”为历史。

> **2026-11-03当前接续**：921 CA-A实际在途，分支已有共享TargetProvider/C scalar/来源恢复，未交付主线；新立[925 CA-B](../../plans/925-a2c-record-text-resource-closure.md) record/text/模式/资源/static-view方法，drafting r1、0/14，仅规划。方法暖P0、来源恢复后下一method编辑、逐点fault/清理与独立C认证不延期。CA-C/D/E未取号，CA-D有限桥沿[Design41 §11](../41-interface-metadata-and-foreign-abi.md#11-a2c统一abi与ffi的近期结构2026-11-03target-state)共享逻辑metadata、分别验证AutoABI/C/内部ABI；不因A2C内部struct直接宣称FFI通过。922盘点已交付，native923实施/924待前置与A2X主体独立，公共S/H真实SHA/全部consumer单写，正式采样错峰；925最终集成含D921，私有准备可先。下文旧CA-B未取号/921 drafting/922待开工为历史快照。

> **2026-11-02最新正式立项**：CA-A对应[PLAN-921](../../plans/921-shared-a2x-c-scalar-target.md)，drafting r1、0/14；本轮只规划，C后端尚未实现。原共享目标去耦/C标量/真认证/函数增量/来源恢复与资源预算不缩，restart调用cold的现状纳T-02/06真链修复。CA-B..E仍未取号。913/914/916已交付；915/917实际实施中，原快照的“913在途/CA-A未立项”不作当前状态。主体A2X可并行，公共S/H与915/917/919/920逐文件单写，正式采样错峰，不等整波完成才开工。

> **2026-11-02 M2全替换补充**：见[M2合同r2](../../plans/reports/ace-replacement-first-m2-contract-2026-11-02.md)。C仍是M1必需目标；M2还须接管旧A2X全部现有目标与AutoUI/AutoMan等场景，并完成旧实现disable，不停在A2R/A2C或仅就绪。生成C用于发布/认证，不强制日常Auto编辑经C/Rust工具链；有限CA-D桥及Q4不冒全部产品接管。未来M3/M4新增扩展暂不规划，旧M4迁移/退役职责已纳M2。

> target-state r1，2026-11-02。用户已明确将A2C加入M1；批准与签章增补见[M1范围增补](../../plans/reports/ace-m1-a2c-scope-addendum-2026-11-02.md)。本轮只设计/规划，未实施、未运行探针/认证，不取正式Plan号。主体设计见[Design51 §13](../51-shared-a2x-and-hir-to-rust.md#13-a2c第二目标与m1范围增补2026-11-02target-state)，ABI边界见[Design41 §10](../41-interface-metadata-and-foreign-abi.md#10-a2c第二目标的有限abi桥2026-11-02target-state)。

## 1. 当前基线与实测边界

规划读取main=44a774998。Rust版ACE的AC/AV/A2X不变；C同样消费结构化受检HIR，执行共享Typed MIR路线仍负责ABC/AV和Cranelift/native，不从MIR反建C循环，不复制旧AST Trans前端。

| 基线 | 交付与当前事实 | 本轮核查限制 |
|---|---|---|
| AX-A / 892 | archived；历史delivery348df0964，最终收据11/11，header仍6/11属既有簿记残留；core/scalar语义、Rust provider、session/认证已有 | 旧delivery不是当前HEAD祖先(exit1)；不据该SHA直接复用证据，按当前code/Spec/新冻结SHA重核，不回开归档 |
| AX-B / 902 | archived14/14，D4303c4457050在当前祖先(exit0)；record/text/text-record、模式/资源/static-view方法、真实Rust认证 | 方法编辑仍有冷路径限制；不得据typed或旧方法缓存命中称全过程P0 |
| AX-C / 904 | archived14/14，D12f9adf18为祖先(exit0)；集合/变体20源、type自由函数片段、资源与认证 | List嵌套/字段/参数返回、闭包/HOF、Unit入口等未覆盖不能推导支持 |
| 797 AutoABI | archived9/9；历史merge1799f86ca，Windows x64 v0.1 candidate、descriptor/3、独立C三腿、layout/准入/所有权/原方释放已交付 | 旧merge非当前祖先(exit1)；当前abi/descriptor及canonical Spec最后触点1795038c9（896迁移），以当前内容为集成基面；本轮未重跑三腿。声明IR不是Auto源FFI |
| 913–917 | 913主恢复T-01..03已记完成；914 executing1/10且T-03观察代码在分支；916已有lang-916实际树（header滞后）；915/917头drafting0/10 | 不把头滞后或clean树当写面空闲；未合入接口不消费WIP，开工重核 |

已读实现：a2x-core::{SemanticView,FnFacts,ParamFacts,FunctionPlan,semantic_fingerprint}；a2x-store::{A2xSession,Family,ResidentFragment}；a2x-rust::TargetProfile/文件集与runtime；a2x-worker/src/bin/a2x-worker.rs；abi::{decl,descriptor,layout,validate,emit,ctool,cache,worker}；auto-lang-core/test_cases::{ac_a2r_hir,abi,ac_protocol}.rs。旧a2c的Trans/Sink与旧产品保留，不作为新C后端语义权威。

| 当前源路径 | SHA256 |
|---|---|
| experimental/ace/crates/a2x-core/src/semantic_view.rs | eaac652ddf650206242a403cf3a5212dbba1a785bbfd23c49e35c026762b08c3 |
| experimental/ace/crates/a2x-core/src/prepare.rs | 345ba347e933516d9f55559a1d3a3fdb6f21b6d0cdd4ba76990c1939dbc44e4b |
| experimental/ace/crates/a2x-store/src/lib.rs | 220cade6557e2e08d2d8190cb502999561142e205d77f4665212cf59e3ffaa0f |
| experimental/ace/crates/a2x-rust/src/target.rs | 448f1daa57dbc9188da3af35a3c547cb402f209a23db0ea599804cdd8452a3d0 |
| docs/specs/auto-abi/windows-x64-v01.md | 97fa28d356759e1aeb6eaed631ff3896dc4dc5affd4ea829d3dde21dd8588c5f |
| docs/specs/auto-abi/conformance-cache.md | b6ac43fadce609d0bd194258c77492cd45ad2ae44e809d1378fce10c8a1b3ca5 |

## 2. 共享层与C provider的实施边界

当前a2x-core视图/prepare已有目标中立职责，但a2x-store Cargo依赖ace-a2x-rust，直接import其lower_functions/TargetProfile，Family表示语义族而非目标，ResidentFragment/文件集/worker仍默认Rust。因此增加C不能只改lang字符串；也不值得先造没有C消费者的通用框架。

目标流：

`Checked HIR/单函数凭证 → SemanticView/权威资源事实 → target-neutral FunctionPlan及实际依赖 → 目标能力准入 → C类型/控制流/清理合法化 → C Doc/AST → 有界确定性文件集 → 同一store服务 → 日常snapshot或显式C认证`。

| 共享职责 | C目标职责 | 禁止绕过 |
|---|---|---|
| 类型/名义/参数模式、Call.eval_args源序与bindings落位、资源/借用事实 | int32_t/uint8_t/uint32_t及C私有结构、参数/返回/临时值表示 | 后端猜类型、按C函数实参顺序赌源码序、重做ownership分析 |
| 每函数prepare、短路/单求值临时计划、稳定身份与source origin | 显式临时、C控制流/错误跳转、正确优先级/字节转义、关键字避免 | 共享层生成Rust字符串再转C；就地改共享HIR影响Rust目标 |
| 实际依赖、quota/事务/lease/GC/文件集检查、诊断事实 | helper/头文件/目标标准/工具flags/具体库映射与内容身份 | 为C复制一套DB/runner/cache；用语义cache装载目标相关结果 |
| target-neutral能力描述与明确拒绝 | C11/target/toolchain profile与可用runtime | 旧A2C或旧A2R隐式fallback，未支持项默认为PASS |

拟新增一个ACE workspace成员a2x-c（具体包名/路径由CA-A正式Plan冻结）；C runtime先作为目标内版本化源资产，不另开workspace/产品依赖。a2x-store保语义族Family，另外引入明确target身份/组合的provider接缝；角色名可为TargetDescriptor/BackendEmitter，但不要求巨型逐节点trait。共享session服务不再拥有Rust私有TargetProfile，目标编译和证书可有各自driver；worker保持原a2r recipe/schema兼容，增加独立C recipe而非复用Rust字段伪装。

真正读取target/layout/capability的pass结果必须进目标相关键；原AXS中立内容与旧Rust命中可保则保，确需语义/schema升级须具名新版本与兼容回执，不能以“共享”静默重写全部Rust金样。C/Rust runtime/helper/printed fragment、manifest、证书不得串域授信。层号按当前store登记在实施时分配，不预占全局A00–A23或复制Rust层号。

## 3. C目标的M1支持域

### 3.1 首平台和源域

首平台Windows x86_64-pc-windows-msvc；C11可移植子集（不依赖VLA、编译器私有类型/表达式扩展）。正式首编译器MSVC cl，实际版本/SDK/flags由认证探测并绑定；Clang交叉编译作为支持条件明确的独立工具腿，未安装记infra/pending、不冒PASS，不成为全平台承诺。每个正式Plan先真实feasibility再固定有效工具组合，不以换编译器掩合法C错误。

源语义跟随已交付profile，至少认领：AX-A的58个逻辑计算样例/边界及正式load256；AX-B全部冻结record/text/text-record/参数模式/资源/static-view方法正反例；AX-C全部20源与原补充资源/活动臂/错误反例。原source不改；逻辑测试数、文件数和检查数分别登记，不把一例debug/release重复计覆盖。源边界沿902/904，不把内建List实例化当用户泛型，不借新目标增加永久语法上限。

核心/record/text：i32/bool/char、str/Str、非递归具名record及资源字段、static/view方法、直接调用/命名绑定、view/mut/move/显式clone/拥有返回、已支持控制流；entry沿现有零参i32（内部支持bool等不等任意入口ABI）。集合：函数局部List<int/bool/Str/record/variant>、new/push/insert/get/set/remove/len/clear、已冻结for-in/is、有限无环Variant/Option/Result；不扩嵌套List/容器字段/List参数返回/变体List载荷/复杂guard通配。拒绝也必须走明确语义/unsupported/资源分类。

### 3.2 数值、顺序、文本和trap

- int=i32不能映射成平台long；Auto char是合法Unicode scalar，不是C char；bool边界规范为0/1，外界非法输入须按ABI规则先拒。
- 溢出/除零/MIN除-1/余数负号按所选Auto语义profile实现checked helper，不能依赖C有符号溢出、移位/别名/对齐的未定义行为。检查必须在危险运算前；普通sanitizer终止不能充Auto trap。
- 所有有副作用实参/字段先按源序物化临时，再按已验bindings落位；短路RHS保分支内。E01=7且ba、E02=103且ba必须同时可观测，不能只检查C文本中函数名先后。
- 文本显式UTF-8字节+长度，NUL不截断；len/slice/char_at按冻结scalar单位，len_bytes按字节；非法scalar/index先拒，再访问。literal按字节转义不依赖源文件locale。
- C内部采用明确的值/trap/resource结果和清理传播，不让abort、SIG、任意非零或宿主panic充Auto语言trap。普通语言失败不穿foreign边界unwind/longjmp；观测wrapper只序列化事实，不参与推导/造结果。

### 3.3 资源、模式与失败原子

消费BodyResourcePlan/CollectionsBodyFacts等同body/context受检事实；缺事实由producer有界补出口，不在C重复分析。所有权转移、临时生命期、view来源、mut排他、record值复制/资源深clone、出口LIFO/分支/循环/错误清理明确；不能用C结构浅拷贝或Rust自动Clone代替Auto clone。

prepare/precheck→完整新值→commit→释放被替换旧值；失败源/旧值与generation不变，部分输出恰一次回收。清理采用显式工作表/可证明有界循环，保原3072层/256KiB小栈、512修订等负载；不能因C无Drop而漏free，也不能新分配工作表失败后泄漏。合法外部pin/持有者保活到最后owner，close/history为空不等资源已释放。

new/push/insert/set、variant switch、字段替换、构造/clone的逐实际分配点失败注入，资源/预算/cancel/墙钟退出，必须经真实helper与原子语义；注入默认关闭、合法正控同语义。sanitizer/heap工具可补内存错误证据，不替独立live/事件/父oracle。

## 4. 目标产物、增量与恢复

C文件集建议包括generated.c、generated.h、必要a2x_runtime.{c,h}、独立main/观测wrapper与manifest（具体最小闭包由CA-A冻结）；绝对路径/nonce/时间不入可重复内容。每函数/声明片段和实际helper需求装配，稳定相对来源映射、完整required边、未声明extra拒绝。输出和已授信输入根不相交；路径穿越/大小写重复/链接与超额均按原产物前门拒。

语义键含body/名义type/signature/mode及实际meta/实例/CT/授权依赖；C片段键另含lang/标准/target/布局与语义flags/backend/printer/helper内容，证书再绑定实际C编译器/链接器/SDK/宏/头与库解析闭包。相同调用点源码不代表依赖未变；头文件由真实工具观测并记录实际解析位置/负依赖候选，沿797既有规则，不只扫描include文本。C/Rust语义可共享，目标片段/工具产物不互用。

支持的暖body/method编辑须真实parse/resolve/typecheck/HIR/prepare/C-lower/print/每函数工件IO仅改函数及实际依赖者；未改函数P=0、同进程resident/payload/代际保持。共享key/文件集目录与source map必要工作M另记，不把全编译后比较wire/文本当零生产。已有Rust方法冷路径是待核残余，CA-B需给本包承诺的method编辑真实门；若要新共享前端大平台应修订/拆包，不能承诺暖编辑却接受cold fallback。

restart恢复以caller来源绑定/实际依赖和工具上下文准入，持久元数据与payload仍未授信；不deserialize Checked标志、host地址或活值。新进程纯恢复支持路径P=0、真实decode/结构/身份/资源重验R>0；不能先编译/lower求内容键再报hit，也不能从未知index自报hash授信。若现有A2X恢复未满足，CA-A最小真链先暴露并纳其必要有界修复，不能把913的AV专用恢复原封移成第二平台。C源码片段的内容hash/编译成功本身不证明源语义，只认已绑定生产来源，失去来源须具名拒绝或显式cold修复并计量。

失败/取消/迟到候选不覆盖现行source/resident/文件集；发布完整依赖后原子切root，旧reader/lease保活，修复后下一次独立restart真hit。readonly数据/root写0，lease控制动作分账；store只存数据，不缓存PASS或guest value。C编译器可能重编大于一个函数的C单元，目标生成P0不冒称cl也只编一个函数。

## 5. 三端一致性扩为四端，旧身份独立

新的Q4套件目标固定为AC-native/AV/ACE-A2R-HIR/ACE-A2C-HIR，按共享源语义逐例映射；原legacy AVM/A2R及旧A2C是独立参考/义务，不作为新目标的fallback。不能按执行当日“谁支持谁参加”动态缩交集；先冻结共同域行和扩展行，每个missing/NA/unsupported须有本次范围依据/owner，冻结后移除必需格须修合同。

每例一份不变Auto source与独立手算oracle/手写reference。四端各自对值/精确位型（仅承诺类型）、stdout/stderr字节、求值trace、语义诊断族与资源/失败观察；不可四端一致就判对。diagnostic按冻结族映射保raw stage/code/span，不要求不同实现错误码字面同名。负例不能拿工具错误/泛非零满足；没有source的scenario保持typed/fixture角色，不伪称源码覆盖。

文件化TestCase首例从CA-A进入，CA-B/C/D各自补全本域case/oracle/反控，CA-E只组合与独立复核。新C的emit/manifest/cert family与既有Rust/旧M1标量证书互拒；actual必来自现场producer、完整原字节编译，reference独立手写，二者分别等于oracle。C代码预先由Rust重写、直接拼reference、剪main/丢依赖“编过”都必须强制红。

日常A2R/A2C均为emit+逐字节已审golden+严格证书/工具身份检查，Rust/C编译器/链接器/目标程序spawn=0（可spawn预构建worker，不能自动构建worker）。M1签章必须持有本C冻结基线的真实显式compile/run证据；cert-gap不能沿旧A2R批准规则自动免除新C必需认证。source/helper/flags/target/ABI/依赖/工具/独立oracle/reference变化按真实身份STALE；坏证书REJECTED，不自动覆盖golden/伪造PASS，日常不隐式重认证。

完整身份未变时可严格复用已完成的认证收据，仍须现场emit、重核产物/证书轴并由父侧比较；这不同于缓存TestCase PASS或把旧guest value当本次执行结果，不要求每次日常重编译来证明已有证书。

## 6. 有限AutoABI桥与通用FFI边界

CA-D只在autoabi.win-x64.v0.1-candidate/descriptor3既有形态中接生成C：pure scalar与fallible context/status/out、buffer/record view/owned opaque handle、原方destroy、同步per-call callback+ctx。C内部结构可私有；跨界record布局和调用按独立header/sizeof/alignof/offsetof及完整物理签名验证，不把生成C struct天然视作AutoABI对象。不扩797不支持的按值聚合/varargs/bitfield/union、f32公开ABI、异步/跨线程callback、unwind、loader/热卸载、跨平台。

必须区分：①Auto source函数→A2C生成实现→显式wrapper→独立C caller；②已受检有限foreign声明→目标C import/wrapper→独立C callee；③同域AutoABI peer↔C wrapper。②若仅有typed declaration fixture，报告typed外部调用支持，不冒Auto use.c源语法/任意header导入。不得只在外层wrapper调用C、实际Auto业务从未经过受检foreign调用却称Auto→C已接通。

共享Interface Metadata/Foreign IR方向沿Design41，只取本profile所需的逻辑签名/来源/ownership/effect/target投影；不新造通用FFI数据库，不把目标布局写回语义类型。调用前拒错域/target/packing/接口/runtime/权限，native被调0；合法非零handle/ctx/混合异质参数必须通过，参数交换/错offset/漏调用须能区分失败，已知不兼容不制造UB盲调。

资源由原provider释放，callback/foreign deleter实际持有者按原支持生命周期保活；不跨allocator free、不把Rust String/Vec私有布局传给C。C ABI桥只验证这组边界，不核销M3通用header/importer、Rust shim/Python GIL、跨线程、动态库状态迁移等未交付能力。797原三腿必须在当前整合SHA真实复验，历史非祖先收据只作线索。

## 7. 首期五包：职责、入口与关闭标准

下列CA-*是工作标签，不是Plan号；CA-C不等于已交付的AX-C（Rust集合包）。正式立项时原子取号并形成AC/V/T/SD合同，不能只靠本路线开工。

| 包 / 拟改面 | 可执行最小入口与正式前置 | 必须关闭的结果 / 交付者 |
|---|---|---|
| CA-A 共享目标合同+C标量 | 现有SemanticView/FunctionPlan→C provider→cl完整文件集→独立reference；a2x-store解Rust耦合、C/Rust同会话隔离，先feasibility | core/scalar全部冻结样例/原负控与load256、源序/短路/checked trap、首例/认证/暖增量/来源恢复/预算；Rust既有bytes/命中/证书兼容，故意跨目标串key/强制whole反控红。共享接口必须带实际C消费者，不单交抽象 |
| CA-B record/text资源 | D-CA-A正式API/实际受检BodyResourcePlan；record/text/方法最小source→C→双腿认证 | AX-B全输入/模式/static-view方法/Unicode-NUL/借用返回、深clone/move/逐出口清理/失败原子；本支持函数/method编辑P0/恢复；不以cold命中顶暖门 |
| [CA-C / PLAN-926 List/Variant](../../plans/926-a2c-collections-variants-resource-closure.md) | D-CA-B资源基础；CollectionsBodyFacts等正式事实与原20源；私有准备可先，最终含D925/D921 | 全20原源+原补例、List<Str>/owned record/Variant、活动臂/view/for-in/is、index/分配/budget/cancel错误及逆序清理/深链、真实函数增量/恢复/认证；保原源与结果 |
| CA-D 有限ABI桥 | CA-A稳定provider/文件集与797当前ABI；资源形态消费CA-B正式接口（可分阶段，不等CA-C整包） | 明确source出口/typed import边界、实际三腿/独立layout与header、完整准入/错配调用0/原方destroy/per-call callback；失败旧out不变、头/target/SDK变化精确失效、原797更严cap不放宽 |
| CA-E M1登记/四端总验收 | CA-A时先冻结新ledger/schema草案并随每包首例接入；最终集成需CA-A..D正式证据及旧M1全部required | 新增来源和目标ledger/严格组合报告、Q4分别对独立oracle、完整C认证身份、原313门+新增C门AND；未知/缺门/空选择/旧报告/绕过C门强制红；不承接A..D延期的功能/测试 |

默认CA-A→CA-B→CA-C，CA-D在其正式接缝就绪后与后两包分窗推进，CA-E规则/fixture准备并行、最终签章在全部门后。每包独立review/冻结受影响职责门/主线合入/Spec读回/归档清理；不要求五包WIP堆在一树，不新增整波全完成才释放写面的规则。

### 必需验收case（正式Plan映射为稳定AC/V/T）

| Case ID | owner / 覆盖范围 | 独立oracle和可区分的错误实现 |
|---|---|---|
| CAV-01 | CA-A 标量/源序 | 原58逻辑样例；E01=7+ba/E02由B接103+ba、短路不执行RHS；故意C实参重排/重复求值/只比value红 |
| CAV-02 | CA-A 目标隔离/真共享 | 同一checked快照生成C+Rust，语义准备复用/目标产物隔离；Chelper变化不重产无关Rust片段；Rust版本冒C证书/把target读入中立键红 |
| CAV-03 | CA-B 资源/模式 | 原text/record及方法，R07/R08=42/T03=23/T05=42；合法view/mut/move/拥有返回与非法近邻；浅clone、漏free、错误caller写回/错borrow来源红 |
| CAV-04 | CA-C 集合变体 | 原L04/L05=42/V03=3/L06 index陷阱及拥有元素补例；旧臂once-free/inactive读0、各实际分配点故障回收；只typed/改源/事后trap红 |
| CAV-05 | CA-A/B/C 增量恢复 | fresh/hit/restart/body-edit、连续多函数/方法变长编辑/改回历史；未改P/IO0/对象代际，纯恢复P0且R真实；源hash同但meta/CT/实例/权限变正确失效，whole后filter/重编求key/旧PASS红 |
| CAV-06 | CA-A..D 失败生命周期 | 准备/生成/发布/取消/readonly/GC/close与外pin；旧版安全、故障修复后下次真恢复hit；半manifest/迟到覆盖/只清history伪回收红 |
| CAV-07 | CA-D ABI | 独立编译三腿/异质非零参数/ctx/record view与out/原方handle；错域/packing/签名/runtime/头依赖变化按原门，native调用0；仅repr(C)或同源layout互证红 |
| CAV-08 | CA-A..E 认证父准入 | 现场完整actual+独立reference分别对oracle，正确typed值/trace/trap；坏类型/重复key/错producer/target/配置/漏required/多余文件/旧证书拒，日常编译器spawn0 |
| CAV-09 | CA-A..D 预算/正式时间 | 原规模/深链/512/四状态30/峰值；C编译与日常周转分列；缩样本/截断事件/累计峰值遗漏/只杀进程当清理红 |
| CAV-10 | CA-E 总门与分母 | 原M1绿而任一新增C/Q4/ABI门缺失，expanded签章仍拒；旧NA不可自动套C，未执行/infra/零选择/丢目标/伪报告不能绿 |

## 8. 资源、正式测试与材料

正式周转沿P90≤2000ms、原样本与正确性/结构/资源同时满足、长期亚秒方向保持。沿A2X日常emit+snapshot/receipt与显式编译认证分轴；fresh/hit/restart/body-edit各30样本，原load256（273575字节/115函数）及既有更严原规模不减。真实源→C→编译运行总时长另列，不把仅emit时间冒编译运行时间，也不缩已有native端到端门。

同一A2X会话工作集沿48MiB（共享状态+当前目标/并行目标resident/暂存全部计，不各报48掩总量），同owner store≤128MiB、certify scratch≤512MiB；source/metadata单项≤4MiB、每流≤1MiB、值≤8MiB/managed backing≤32MiB、父private峰值增量≤128MiB与原更严门保留。编译≤120s/运行≤30s/认证wall≤360s；797 ABI请求仍build60s/run5s/total70s、热点8MiB/暂存索引8MiB/单产物64MiB/请求128MiB原门优先，不能被泛化预算放宽。预算不合最小真链时正式修订，禁止先分配再拒、隐蔽旁路map或用语言永久深度限制兜底。

固定职责组根据最终diff冻结：ace-a2x、case-a2r-hir-record-text-902、case-a2r-hir-collections-904、auto-test-case、ace-adapters、infra-fixed、infra-ace-m1；共享HIR/source/资源变更加ace-source/ace-semantics/ace-lowering及必要ace-av/ace-native真实consumer；CA-D加ace-abi，正式周转/深链用原专项条件。新C域与Q4固定组由各正式Plan拟定并注册，不现在改test-groups.toml、不把旧a2c组当新目标覆盖。完整组/必要consumer并集按冻结SHA一次选择，证据身份未变可复用；不叠加无因全experimental/all，周期cargo t all仅due主检出单实例。

正式Plan才创建唯一组worktree/分支与D:/autostack/plan-artifacts/plan-<NNN>/，新例长期入职责case目录；所有探针/编译副本/外部消费根/认证scratch/外置target显式置统一材料scope。merge把命令/退出/关键数字/内容hash折计划后删整个scope及树中间物，具名未解才hold；不写attachments/仓内reports，不清他方TEMP或store，不建链接。本轮未创建这些目录。

## 9. 与913–917的写面交接

| 写面 / 当前owner | 新A2C可做 / 必须等待的具体接口 | 验证与释放合同 |
|---|---|---|
| P-C 私有C provider/typed runtime/reference | 设计和独立参考可并行，不改913恢复或旧Trans；正式实现待本包立项/执行授权 | 不能复制Rust session或MIR前端假共用；新workspace成员/lock登记属H |
| S-FE core HIR/schema/单函数凭证/资源事实、914布局/场景事实 | CA-A..C只消费正式事实，缺出口具名拿窗；不把914分支布局策略当主线已支持 | source/owner/body/context/实际deps与全部AV/native/A2R消费者绑定；修改后受影响证据STALE |
| S-STORE artifacts/ManagedStore及913来源恢复 | 通用有界读取/发布/lease接口取主线或913明确冻结提交；不写其closures_source_recovery/root/schema/accepted-context | 913当前get_artifact_by_id_bounded仍分支事实，不默认已释放。C自有namespace/来源合同在CA-A验证，不共用AV活cell/结果 |
| A2X core/store/rust/worker | CA-A主目标接缝与去Rust耦合，B/C顺次扩面；保Rust既有recipe/文件集/证书 | 同文件单写、列construct/match与所有consumer，旧Rust golden/hit不自动更新 |
| R-legacy 916/917、914旧a2r/avm修复 | C不改trans/rust.rs/trans/c.rs或旧A2Rchecker；915资源观察尽量只读消费 | 若触公共infer/ownership/diagnostic，按实际owner短窗，下游完整组不可豁免；无D916/917整包前置 |
| H test_cases/mod.rs、ac_protocol.rs、ac_a2r_hir.rs、新C adapter、auto-test-case、SDK、fixed组、workspace manifest/lock、M1登记/证书reader | CA-A首例需要H；CA-E新ledger不回写旧批准文件/hash或修改914–917已有分母 | 全文件单写，不因append-only认为可并写；旧协议保兼容，新C/组合报告显式版本，所有父读者/工具/SDK构造匹配核齐 |
| 正式认证/采样 | 与913恢复规模/914布局周转及915资源/916–917认证错峰 | 不持代码写窗等机器，不停别方进程；记录准备/发现/执行/清理、完整终态与峰值，环境红/重放绿都留 |

每次交接记录owner/revision/基面与实际SHA、符号/字段/schema/profile/工具身份、全部producer/consumer与构造/匹配、已跑/pending/释放状态；稳定阶段可以先交具体符号，不强设D913..917整波完成前置。未取窗只阻依赖任务；接口变动STALE后只重验受影响身份，不能提交他方WIP或要求他方整包等待。

## 10. 后续正式立项入口与当前未完

下一步可先为CA-A正式立项：重核main/913–917实际owner与797现行三腿/892–904实际内容，冻结C11工具组合、provider合同/原Rust兼容、C基线输入与新ledger草案，最小真实C标量+单函数暖编辑+纯恢复/独立认证后才扩面。CA-B/C/D/E的正式号按准备就绪顺次申请，本轮不预占。

需要在各包T-01/02关闭的机制选择：Rust耦合如何最小解开而不改原bytes；来源绑定恢复如何真实P0；source/method片段与资源凭证是否够用；有限foreign声明如何接真实调用而不冒源FFI；新ledger/父报告版本如何保旧consumer并阻止漏C签章。这些是技术feasibility，不是重新请求已批准的“A2C加入M1”；若需扩大语言/ABI/预算，才按正式Plan修订规则处理。

本轮只形成设计/路线/范围增补；尚无新C生产实现、C证书、Q4执行PASS或扩大后M1通过证明。旧M1统计与914–917交付继续有效但只覆盖其原合同；它们全部完成也不能单独满足新增M1门。

### 后续规范沉淀责任（当前不写Spec）

| 正式包 | 拟Spec delta（新路径为未来目标） | 必须留下的现状/限制 |
|---|---|---|
| CA-A | 修改auto-a2x/architecture.md、incremental-and-cache.md；新增auto-a2c/project.md、scalar-hir-backend.md（均在docs/specs） | target-neutral接缝、C支持矩阵/明确拒绝/来源恢复/原Rust兼容，不能仅写抽象已通用 |
| CA-B | 新增docs/specs/auto-a2c/record-text-hir-backend.md；必要时更新auto-a2x/checked-resource-facts.md | 参数模式/借用来源/方法编辑/资源计划及真实清理、失败与增量证据 |
| CA-C | 新增docs/specs/auto-a2c/collections-hir-backend.md；必要时更新共享Collections事实 | 原20源和支持/排除域、owned view/活动臂/清理/预算，不冒完整容器语言 |
| CA-D | 更新docs/specs/auto-abi/{project,windows-x64-v01,conformance-cache}.md并补C桥边界文档 | candidate不擅自升版；source出口/typed import分层、独立三腿与实际头/库依赖/生命周期 |
| CA-E | 新增docs/specs/auto-test-case/a2c-hir-certification.md与扩展M1目标登记规范；修改M1 conformance/applicability/project | 原批准链不变、Q4身份/新C强认证/总签章AND、旧报告不能充新M1，通过才沉淀current |

各正式Plan以实际diff和全consumer复核最终SD；不能在本轮把这些未来文件创建为空Spec，或凭设计条目勾实现PASS。

### 本轮规划静态核查（2026-11-02）

新路线10个主节、CA-A..E五个互异职责和CAV-01..10十个稳定覆盖case已核；新增决策/路线的全部本地链接与锚点、各设计/战略/交接入口新增导航、现有固定组名称及空白检查通过。只证明文档结构/来源一致性，不证明C语义、资源、周转、ABI或认证PASS。

旧批准件、binding/applicability/source-manifest及915–917与本轮核查快照hash保持；913/914在核查期间由各自会话继续提交进度（主线到1261b3b15），保留其新收据，未改写/回退它们。本轮没有调用new-plan.sh或写.next-id；其他会话已分配918跨仓治理（与本C目标不同），核查时.next-id为919，CA-*仍未取号。无C实现/current Spec/运行证据、工作树或材料目录创建，无构建/语言测试/认证/代理/会话启动；主检出原有无关WIP保持。
