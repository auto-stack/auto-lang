---
plan_id: PLAN-891
status: executing
feature_name: native宽标量typed入口与逐位结果通道
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-10-31
plan_revision: 2
supersedes_spec_components:
  - docs/specs/auto-mir/data-place-profile.md
  - docs/specs/auto-mir/native-data-owned-lowering.md
  - docs/specs/auto-ac/native-backend-and-retirement.md
  - docs/specs/auto-test-case/ace-m1-conformance.md
  - docs/specs/auto-test-case/architecture.md
new_spec_components:
  - docs/specs/auto-mir/native-typed-entry.md
touched_goals: [GOAL-002]
affects: [auto-ac, auto-mir, auto-test-case]
depends_on: [PLAN-820, PLAN-828, PLAN-871]
current_step: 3
total_steps: 10
---

# [PLAN-891] native宽标量typed入口与逐位结果通道

## 0. 变更摘要

交付av.data/1的六标量typed entry：i32/bool/i64/u64/f32/f64参数与单标量结果，经受检MIR→单函数COFF→真实PE，在独立、版本化的mn私有ABI及结果通道上逐位验证。关闭M1 W01–W05的native缺口；不以32位进程退出码、浮点十进制文本、is_nan或AV结果代替原生返回值。

承接[第二阶段拆包决定](reports/ace-native-phase2-split-decision-2026-10-29.md)，889继续独立实施集合/变体；本包现在立项，N/H实际写入按889/888交接，不复制WIP、不改其合同。未授权本轮施工、建树、构建、探针或跨聊天通信。

## 1. 目标

- G1：六标量在const/copy、局部、分支、跨函数参数/返回及入口输入/结果链上保持类型与原始位型；纯值record的内部wide字段随Aggregate/FieldGet/FieldSet及调用正确传递。record不得出typed entry。
- G2：新增明确选择的typed入口模式；旧零参i32退出码模式保持行为，不静默改为typed，也不把进程状态当值。
- G3：实际机器结果进入有界、带身份的精确通道，父侧独立oracle判定；恢复重验、失败分类、负控与资源回收成立。
- G4：按受检MIR起点证明函数级精确增量、依赖失效、fresh/hit/restart；输入参数改变只重新执行，不重产用户COFF或wrapper。

非目标：宽数值算术/比较/转换、宽字面量源码推导或新源码profile、全语言函数参数模式、Char/Str/record/Unit作为新typed-entry值、wide与拥有record/List/Variant的组合、native闭包/Dyn/HMR、public AutoABI/FFI、多平台ABI、console profile原生化、完整M1验收。既有旧模式的Char/Str/record/collections合法能力按实际交付合同保留，不能因本包排除typed组合而倒退。

## 2. 架构方案

文件化typed fixture/受检MIR输入（相同内容供AV独立对照）→verify_mir→单函数unit→mn typed-data准入/布局→CLIF/COFF→专用typed wrapper+最小transport helper→真实PE；父侧worker发送实际参数，接收PE返回值帧→SDK形状/身份验证→父侧手算oracle。worker不接收expected，也不得从fixture常量表、AV执行或request参数直接合成“实际结果”。

**ABI决策**：本包冻结独立版本的mn-private typed-data ABI，属于内部调用合同，不是797公开AutoABI。数据运算尚未准入，F32/F64在本ABI以不透明I32/I64位载体传输；I64/U64为I64，I32/Bool为I32且bool仅0/1。这样无需任何浮点算术/格式化/宽窄转换即可保留sNaN payload。逻辑类型在MIR/verifier、签名及cache身份中始终区分，不因载体相同混用。

与旧ABI连接必须显式adapter或具名拒绝，不能把两种约定的COFF链接成同名函数。独立typed wrapper依据已验证签名装载参数并取回原生返回位型；参数来自运行时帧，不烘焙到用户对象或wrapper中。T-02以跨对象、多参数真PE可行性probe冻结具体符号、tag、helper及帧布局，等价局部机制可选，但不能改上述语义。

## 3. 技术栈

| 写面 | 已核真实路径/符号与拟新增项 |
|---|---|
| N准入/布局/codegen | experimental/ac-mir-native/src/{emit,lower,layout,config,symbols,fingerprint}.rs；admit_unit/cranelift_sig_of/emit_wrapper_coff/Shape/SlotRep/render_unit/const_repr，新增typed ABI选择、精确指纹与独立wrapper，不覆盖旧模式 |
| N缓存/发布/执行 | src/{driver,cache,incremental,link_driver,worker,resources}.rs及src/bin/mn-worker.rs；CoffMeta/LinkManifest/MnDriver，新增受检MIR typed入口，不走源码推导 |
| N拟新增transport | experimental/ac-mir-native/src/typed_entry.rs、runtime/typed_entry/最小静态库、tests/{typed_entry_tier,typed_entry_incremental_tier}.rs；T-02冻结名称/manifest，不能冒称当前已存在；W可只读复用probe/src/typed_fixture.rs的fixture_module，新的输入/builder仍须verify |
| 可复用的受控进程设施 | experimental/ac-core/src/link.rs的run_deadlined_captured；experimental/ac-av-probe/probe/src/native_exec.rs的run_stage只支持无stdin捕获，typed输入须最小有界扩展或N本地适配，不直接假设现有函数已可写入 |
| H父侧与文件化合同 | crates/auto-lang-core/src/test_cases/{ac_mn,mod}.rs（mod.rs实际dispatcher）；crates/auto-test-case/src/contract.rs；必要公共测试注册、report/stale及ace-m1 binding；均逐文件取窗。ac_av.rs仅只读对照，抽取共用判决须另交其具体符号；不修改av_probe_worker的批量分派 |
| 拟新增正式验证 | scripts/verify-native-typed-891.ps1；crates/auto-lang/test/cases/ace-m1/typed-native-891/下.suite.ad/.case.ad及typed输入文件；确切编码在T-02/05建立，不修改原W oracle |

拟权威实施树D:/autostack/.wt/lang-891/auto-lang，分支plan-891-dev；当前不建树。work、独立review、修复同树顺序进行。仅语言仓；不新增根workspace对实验栈的产品依赖，不触公开AutoABI或外仓实现。

## 4. 需求分析与背景调查

授权：用户2026-10-30指出889已开始实施，询问并要求先规划带位宽typed-entry后继包。范围继承前轮拆分：单独验证i64/u64/f32/f64、负零/NaN payload；本轮只有计划/路线簿记。未授权启动另一实施会话、修改889/888进度、探针/测试或代码。

基面master=e0b29ff7a；889 executing r1/0-of-10、888 executing r2/2-of-10。863已交付2d5f3d842、归档8/8；865已交付5fa3133fa、9/9。当前Spec为现状，计划仅提出增量；开工重核实际owner/正式提交。

2026-10-30 r2复核基面master=b3f43a7c4：889已executing r2/1-of-10，权威树提交69d7b6b26；其T-02现先修S命名enum支持域误拒、再做N资源全链probe，**尚无N ABI/runtime稳定释放**。888仍executing r2/2-of-10、权威树注册提交b81f06206；此仅为状态读取，不消费两树WIP。用户本轮明确授权直接修订遗漏，仍只做规划；r1原语义/阈值保留，r2消除实现与验收空白，详见[规划复核](reports/ace-891-plan-review-2026-10-30.md)。

已读AGENTS、Spec overview/README、auto-mir/data-place-profile/native-data-owned-lowering、auto-av/data-layout、auto-ac/native-backend-and-retirement、auto-test-case/ace-m1-conformance及Design41/43相关内部ABI/精确增量规则；读取889只为确认并行/后继边界。

| 已观察事实 | 规划结论 |
|---|---|
| AV data profile已有六标量Const/Copy/Call/Return/entry，wide Bin/Un拒绝mir.type-op | 复用语义与verifier，不新增运算或wire opcode；本包是native数据面，不是完整宽数值源码 |
| emit::admit_unit拒wide触达，layout的shape/field_shape拒wide；driver和ac_start限零参i32族 | 准入、槽、字段、调用、wrapper、正式结果消费均需实施；不能只撤掉type拒绝或扩退出码 |
| native config MN_ABI=msvc-x64/priv-i32-bool-v1，target=x86_64-pc-windows-msvc；实际owned ABI已更丰富 | 不凭旧tag名称推导ABI，T-01接受889最新实际合同；typed ABI独立新身份，所有消费者镜像同步 |
| ac_av父侧DATA_TYPED_FIXTURES已有独立kind/bits表；W01–05 native仍gap:mn-no-typed-entry | 复用既有oracle内容/类型规则，新增mn正式recipe及判决；不拿AV返回值当oracle、不改旧expected/分母凑绿 |
| native_exec::run_stage只有输出捕获，不能输入交互 | 有界双向I/O/关闭/杀树须先真probe；共享设施扩展取H/S实际符号窗，不复制无截止时间的spawn |
| fingerprint::shape把I64/U64/F32/F64都渲染为unsupported，const_repr只保宽常量discriminant | 新准入必须同时修类型/位型指纹，区别相同低32位、不同NaN payload及负零；不能沿用占位渲染后的键 |
| symbols::function_symbol实际只含owner+unit，未含签名/ABI；worker恢复按全局MN_ABI准入 | typed符号域、闭包级ABI/签名校验与按请求传递的ABI选择必须显式实现，不能仅换一个全局tag |
| 当前MnIncrementalSession走scalar FunctionIncrementalSession，暖路径及restore每次emit_wrapper_coff | typed MIR需居民单元更新入口与wrapper独立缓存/保活；不能把整模块verify/split后carry当新增能力 |
| .next-id HEAD=887、主检出已有891号前的其他会话取号WIP | 独占分配锁下复核active/archive无同类计划，scripts/new-plan.sh分配891→892；保留他方号源累进，不夹带提交其WIP |

源输入SHA-256：emit.rs=4b2252b06f1ea0c6961b09ada01537525a7f78bc06cbb591db16c07207fcd58b；layout.rs=89fea890852707619f8d29fd52253e6c1f82a3b60972886ec5dae4566a4b36ad；config.rs=59411657583ed120e3591e05d2dd93a9eaec2edd44d2ff49f2b4b1a85c444459；data-place-profile.md=753bcd3ed09e198c1999ee50acdcd759d293987e78e3957e60db1abe2ebda194。889未提交实现不属于这些已读输入，不消费或替其定ABI。

r2新增静态输入：fingerprint.rs SHA256 dc0080b8ac6da9dddfe8bc412cdf758557567dc967c5c5ffcd665becf324d5c5；symbols.rs SHA256 88ceb303491ab2763d40cff77f2e9048be1518168140490b9748efc5c9d3c3fa；共享probe/src/cache.rs SHA256 b681706571be575cc8a030f7c62718f47f510684265be080d9c054af6497f681。旧hash是各自读取快照，不是未来889交付的ABI承诺。

## 5. 详细设计

### 5.1 前置、窗口及第一步

语义前置D820/D828/D871已交付；D889不是新增语义依赖，**N默认889→891单写**。889可交付整个计划后释放，或以已提交且验证的稳定阶段交接具体emit/layout/config/cache/ABI符号；review/repair WIP不作稳定阶段。891 T-01可先只读，T-02及后续N probe/实现等真实释放，不能以新文件名规避共同ABI写面。

H须与888及889对齐recipe/worker请求、ac_mn/ac_av父侧、contract/dispatcher、测试注册、binding/report镜像。888只读产品native/前端语义，不构成891整包硬前置；无法同时拿所需N/H时做已释放独立任务，不持一窗等待另一窗。transport若改共享link执行helper须明确符号窗口，不能泛称只改N。正式性能采样与888错峰。

T-01入口记录实际树/branch/HEAD、正式D门、889版本/支持与拒绝面、888 H owner/源锁、精确文件/符号接受清单。交接带commit、Spec/profile/ABI/runtime/layout/COFF/link schema/codec版本、兼容与限制；最终891反向释放给后继，不改其他Plan合同或主动跨聊请求。

| 具体共享面 | 891取得写权的条件与边界 |
|---|---|
| N emit/lower/layout/fingerprint/symbols/config/driver/cache/incremental/worker、mn-worker及runtime依赖 | 889 T-02尚未冻结/释放；891只读。稳定阶段须889实际提交并验证所交符号、889不再继续并写、891记录接受与回交，合入/基面推进顺序明确；不能以master目前未改该文件当空窗 |
| S共享split_units、MIR类型/verify/codec、ac-core::link或probe::native_exec | 默认只读复用；若typed单元准备或双向I/O确需改共享实现，T-01列具体符号与消费者、接受889/实际owner释放；相关T-02/04同时需N+该S/H子窗，未交接可用已授权等价N适配，不复制语义或无界进程设施 |
| H contract/dispatcher/ac_mn/可选ac_av共享判决、测试注册/worker请求/报告stale | 与888和889各核真实已提交diff；mn-worker也是N/H交界，不能仅按N准入后先注册公共请求。ac_av及av_probe_worker现为888触面，未释放保持只读；新typed recipe沿独立单次执行，不默认纳入888 B/C白名单 |
| 测试发现/模块清单、源锁/原子映射与正式计量 | T-05新增typed用例必须纳入当时实际交付的模块/消费者义务，非手改888冻结报告。变更影响已冻结清单/源锁时，记录增量与owner接受，按其合同在下一冻结点刷新；不强迫888采样中rebase。正式采样需明确机器窗口，撞窗先让窗，不改其已有样本 |

每次阶段移交同时给出文件/符号、基面及候选SHA、未提交WIP范围（保持只读）、ABI/布局/fingerprint/wrapper/runtime/协议版本、消费者兼容、旧键失效、已跑与pending门及接受/回交记录。上述均为891执行入口要求，不宣称本次已取得窗口；T-01只读部分可先做，不能单独计作全部窗口已就绪。

若阶段提交将成为891的依赖输入，须有该阶段适用review/gate及已合入主线的正式交付收据；仅实施分支已commit/scoped绿不等可消费交付，不从889/888分支cherry-pick未交付候选。若释放的是既有主线未变符号，记录对应正式基面与停止并写即可，不人为新增整项D889/U888。

### 5.2 数据与调用支持闭包

- 新typed模式仅av.data/1；六标量entry参数与单标量result。内部支持Const/Copy/Call/Return、现有i32/bool控制流、纯值无环record的Aggregate/FieldGet/FieldSet及参数/隐藏结果。宽字段I64/U64/F64为8B/align8，F32为4B/align4；声明序、checked offset/size与既有record身份规则一致。纯值record不含Str/borrow/owning资源。
- 不引入任意param数量语法上限；沿shared verifier与既有输入/工作预算。新调用arg区所有count×width/align checked算术且分配前核预算；超限是具名resource，不冒类型错误。
- Bool入口非0/1、kind/width错、参数数目/顺序/签名不符在执行前拒绝。i64与u64、f64虽同载体仍不兼容；record名义不同不能互换；宽Bin/Un/转换、wide-owned/collection组合继续codegen=0具名拒绝，不隐式升级889支持面。
- F32绝不经f64，浮点从常量到slot/字段/参数/结果/序列化全链无算术、十进制roundtrip或NaN规范化；sNaN也是数据位型。机器加载/存储及copy必须明确位载体，不依赖宿主浮点值比较。

**物理ABI补全**：目标固定x86_64-pc-windows-msvc、Cranelift WindowsFastcall。逻辑参数声明序映射至整数位载体；Win64前四个位置与后续栈参、shadow space、栈对齐及callee-saved均由同一受检CLIF调用约定生成，wrapper/runtime声明必须一致，不手写另一套浮点C ABI。I32/Bool/F32只消费低32位且存储/帧宽4B，I64/U64/F64宽8B；不从I32机器寄存器未定义高位补出语义，不做i64↔u64数值转换。单标量返回用对应整数载体；helper外部C接口只传固定宽整数/字节指针，不把F32/F64声明成C float/double。

内部pure record参数为地址，但callee先复制到自己的值槽；重复实参/别名不能让callee FieldSet改caller。record返回用尾随隐藏out地址、无寄存器record结果，caller分配对齐且与live实参不重叠的结果暂存后再赋值，保持RHS先物化；不把此ABI错当平台C聚合ABI。新增宽字段的copy/嵌套布局、参数与结果暂存及销毁均走实际大小/对齐，padding不得进入语义指纹或结果位型；T-02明确caller/callee/helper各自临时区所有者与生命周期。

所有输入先走共享verify_mir，单位准备后再核mn支持闭包和imports；未经验证的fixture builder/磁盘unit不能靠“profile正确”绕过类型、def-use/初始化、名义record或操作合法性。typed ABI选择从请求→单元meta→符号→link→恢复→wrapper完整传递，不能临时改进程全局MN_ABI影响并发旧模式；新typed符号域需与旧域隔离，并核每个import的真实签名及callee ABI。body编辑保持逻辑符号，签名/ABI漂移必须拒错链或产生正确新链接，不因同owner/函数名可解析就放行。

### 5.3 typed通道、信任与失败

新增独立recipe（拟ac/mir-native-typed-v1）与协议/ABI/transport身份；旧ac/mir-native-scalar-v1保持行为。父侧期待值仅留case/oracle，worker请求只带输入/MIR/入口/参数/预算/nonce；最终typed结果来自真实PE机器返回。

拟使用子PE专用stdin/stdout二进制帧，stderr只承载有界诊断；不占用worker对父侧的JSON通道，也不把guest console输出当结果。此模式不准入console profile，保留未来专用handle adapter空间。T-02固定帧头/version、signature/arg schema、nonce/request-hash、length、little-endian payload与EOF规则，最大输入沿MAX_INPUT_BYTES=4MiB、输出与stderr各≤现行1MiB；仅1次调用/1结果，无任意尾随字节。rawbits不经JSON number。若等价专用pipe/文件方案更合适，须提供同样反控及身份/关闭证据，记录决策而非下调验收。

T-02必须冻结完整请求/响应状态机及准确schema：请求的recipe、owner/entry、逻辑签名、ABI/transport版本、参数kind+原始字节、nonce及实际输入内容hash与受准入链接产物身份绑定；响应只携实际观测及同次调用身份，不含expected/PASS。长度/count/宽度先checked校验并分配前核上限，禁止负长/溢出、重复字段或冲突判别、未知kind/version/flags及尾随帧。父侧 independently 从实际请求与已准入产物核上述绑定；不能信worker回显的自洽hash或仅nonce相等。wrapper从寄存器/真实结果槽序列化，runtime不得根据fixture ID、请求参数或签名重建应有值。

结果关联还包括实际launch的PE内容身份、required-data闭包、wrapper/helper内容及二进制/源输入profile指纹，路径名和进程exit0不证明执行了期望产物。runtime固定代码不烘焙每次nonce/hash，运行时读取并校验请求；同签名换参和重启换nonce均无需新wrapper。文件化输入编码/fixture builder在T-02冻结：输入/编辑内容hash真实进入请求与report，same typed MIR分别进入AV/native各自verify；内部单函数patch格式不含oracle，不用改变expected充当body编辑。

成功=进程exit0且恰一完整合法结果，kind等于实际签名，机器值按canonical hex给父侧（i64/u64/f64固定16小写位，f32固定8位；i32/bool沿既有typed判决规则）。真实i32结果70是typed value、exit0，不得误判trap；trap为独立非成功状态/具名原因，不输出成功typed值。缺帧/截断/重复/旧nonce/错hash/超限/异常exit+结果均不能PASS；语义trap、资源失败、协议/宿主I/O infra分列。

typed模式的结果是互斥value/trap/resource/error判别：value70与i32除零/溢出等实际trap分别可观察，T-04在真正trap分支接出有界原因，不从exit70猜具体语言诊断。没有合法失败证据的异常退出归infra；合法trap/resource帧须与其进程状态一致、无value载荷。成功帧先到但之后hang/异常exit/超限仍不得PASS；父侧等待EOF、进程完成及清理后才提交判决。Bool结果也必须0/1，不能把任意非零归true；i32最小值/负一以低32位raw编码并按现有canonical hex判定，未知/重复/缺失轴、错误有符号解释不能默默纠正为成功。

复用受控spawn/Job/进程树/有界read与drain。写入亦有deadline，先读双方输出避免死锁，取消/超时关闭pipe并清理owned子孙及reader/writer；回收必须观察，不以调用函数已返回替代。现行build60s/run5s/request70s及父侧更严预算保持，不延长来掩盖阻塞。

### 5.4 cache、恢复、增量及预算

- 单函数身份绑定逻辑类型/位宽、ABI、backend/target/flags、MIR profile/schema/semantics、body、import签名、record layout、runtime/helper合同与内容hash。链接还绑定entry/signature、wrapper/transport版本及required-data；被调用body编辑改其对象和最终link，调用者是否重产按实际依赖，不能只看调用点源码。
- 修复fingerprint::shape/const_repr的wide占位渲染：逻辑类型各自唯一，整数按完整宽度、浮点按原始bits入语义渲染；import Fn签名也完整展开，不能继续是泛fn/unsupported。类型/常量表重排重映射、重复primitive等价项、span/source_snapshot改变不改语义键；未触达死表项不引起无关单元失效。新typed指纹版本/ABI隔离旧占位键；原域渲染变化如不可避免须同步版本与消费者/失效证据，不能悄悄套旧收据。
- invocation参数/nonce不进入用户COFF语义键，也不烘焙进wrapper；同签名换参/nonce仍实际运行，COFF与wrapper生产为0。调用结果不缓存为PASS。签名/布局/ABI变化则正确失效；旧标量/889对象不得误作typed ABI。类型表死项按现行触达规则，不能靠全部表变化重产所有单元。
- 新typed wrapper是明确拥有/有界的居民产物，并可按ABI+entry逻辑符号/签名+target/flags+transport/helper身份持久恢复，纳入required-data及hash准入；其缓存不是简单跳过函数生产计数。暖body编辑不改签名时保wrapper payload身份，参数only修改同样生产0；冷恢复必须从已验证bytes取wrapper或如实计wrapper rebuild，不宣称未实现的冷wrapper零生产。旧模式每次生成wrapper的历史行为不因本包新模式承诺被重写。
- hit/restart/read-only/fresh均复核meta、COFF结构/hash、运行库及link.required-data、ABI/签名和输入shape；损坏miss→重建后下一次真hit，不能只当次fallback成功。只读miss不发布；候选构建/链接失败保旧产物及当前会话，错类型参数在执行前拒绝，旧合法调用仍可运行。
- 增量起点诚实为typed MIR；两函数body编辑在实际verify/split/emit/persist点计数/identity，未改函数的上述生产=0。签名/layout改需失效真实依赖，恢复重新验证允许且单列；不宣称未实现的宽源码parse/typecheck=0。继承828与889已有scalar/source增量，受影响命名回归不得退化。
- **暖态单体更新与冷恢复分账**：T-04建立保留已验证unit/import签名/类型布局事实的typed居民会话；初次整模块verify/split可以计账，暖body patch只准备/verify受影响单元，不先全module verify/split/clone后筛hash。每个生产点计unit_prepare/remap、verify_mir、语义render/fp、native_admit、emit、persist次数/字节，unchanged各项=0，whole fallback=0；同时核实际resident payload地址/所有权token及mint generation、被链接COFF ID/hash未换，拒“拷贝payload但保generation”。依赖图/链接闭包轻量一致性检查和冷COFF结构/hash重新准入独立计，不假报零；计数不得只由changed列表反推。
- 上述identity约束指居民unit/code payload不得重建替换；为真实link/run进行的必要有界I/O或链接暂存复制允许，但须独立记账并追溯同一居民对象/COFF，不据此声称“全链零拷贝”。反控应替换resident而仍伪保旧generation，不能用普通临时输出buffer复制当作结构门必红。
- warm body edit保持callee签名/layout时只其对象变化、依赖caller复用，link实际换用新对象，独立结果随之变化；signature/layout/ABI变则重验/重产真实依赖而不波及disjoint函数。A→B→A内容回访须实际复用历史受检对象而非再mint同hash，缓存有界、GC/会话close释放居民wrapper/unit且不遗留临时产物；候选失败不得提前提交当前unit/manifest或历史索引。
- managed预算沿48MiB分区16/24/8MiB，不把观测ledger冒成防分配护栏；新帧/arg区实际记账并在分配前拒。记录worker/子PE分别与树级物理peak、staging/I/O/cache常驻容量及清理后live。compiler预算不与guest逻辑预算混为一个最小值；不无据放宽889等现行硬上限。
- T-02按真实数据大小固定单次arg/record暂存与native栈帧的资源防线，不能等宿主栈溢出才冒称resource；沿既有共享type/layout/work预算，边界单位、分配前拒绝点和诊断来源明确。typed类型/布局/指纹闭包采用受预算的迭代或等价有界遍历，避免无环深层/共享DAG的宿主递归栈溢出与指数展开；不擅加语言类型深度排除。已有物理peak硬门有正式出处则逐入口继承；没有的只报告测量与局限，不把测到一个peak称为物理预算已受控，也不编造与managed/guest混用的新上限。
- 周转统一预构建工具、20正式样本及warmup分列，fresh/hit/restart/body-edit分别P50/P90/P95/P99；开发正式门P90≤2000ms，长期亚秒目标另列。所有正式样本失败/重测保留，888计量窗口互斥。
- 采样口径T-02锁定：四状态各20正式样本，nearest-rank分位，计时从各自正式请求/编辑入口到真实PE完成、父侧验证与清理；warmup/工具构建单列。不能只测emit、扣掉link/run或跨状态混P90；参数only执行另作结构观测，不拿它替body-edit时间样本。

### 规范增量

| delta_id | add/modify/retire | docs/specs目标 | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-mir/native-typed-entry.md | 无native typed ABI/transport→本包数据支持矩阵、bit-carrier、entry及结果/失败/预算合同 | 防宽位型和退出码混淆 | AC-01/02/03/05 |
| SD-02 | modify | docs/specs/auto-mir/data-place-profile.md | native wide一概拒→av.data/1受检typed数据面可消费，运算/组合域拒绝保留 | 不扩大AV或源码语义 | AC-01/04 |
| SD-03 | modify | docs/specs/auto-mir/native-data-owned-lowering.md | 单旧ABI与wide拒绝→并列typed模式及旧域兼容、实际889交付后基面 | 禁止覆盖他方未交付合同 | AC-01/04/06 |
| SD-04 | modify | docs/specs/auto-ac/native-backend-and-retirement.md | 旧五域driver/cache事实→追加typed选择/ABI与transport身份/恢复 | 维持唯一共享MIR native源 | AC-02/04/06 |
| SD-05 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | W native gap→逐条真PE正式绑定、kind/bits父侧oracle及范围；NA/旧轴保留 | typed能力不冒领source/全M1 | AC-03/06/09 |
| SD-06 | modify | docs/specs/auto-test-case/architecture.md | 现有输出/exit/typed观察→追加native typed recipe的形状/身份/互斥失败轴及stale规则、模块发现消费者边界 | 正式父侧判决与H接线可追溯，不改变旧通道 | AC-02/03/06/09 |

## 6. 测试设计

所有未来case当前未执行。T-02建最小全链probe，T-05把V-01..14做成可运行正式测试/AutoDown，辅助typed布局/失败测试单列。AV及native各自对手算oracle，彼此相等只作补充。W可同用已交付typed_fixture::fixture_module产生的未信任MIR，各自verify；新增参数/record/编辑fixture不得把其常量表当native结果。实施与独立final review顺序用同一权威树，未指定模型/新代理；实施自检不替独立review。

| Case ID | AC / invariant | 输入/状态 | 真实入口与observable oracle | owner / boundary |
|---|---|---|---|---|
| V-01 | AC-01/03 W原位型 | W01 i64-min、W02 u64-max、W03 f64负零、W04 f64负NaN、W05 f32 payload | 新正式recipe→真实PE；kind分别i64/u64/f64/f64/f32，bits=8000000000000000/ffffffffffffffff/8000000000000000/fff80000deadbeef/7f80ad00；保持原五行oracle | work T-04/05；typed起点，非source |
| V-02 | AC-01 ABI实传 | 每种wide分别identity(input)与constant-return；六标量混合参数跨前四位置/栈边界、重排并分别选择每个参数返回，导入callee→return；i64输入0000000100000001/0000000200000001 | separate COFF链接PE，六次选择各对应参数原bits；两高位不同值可区分，非所选输入改变不改变结果；wrapper/helper与caller/callee调用约定相同，无浮点C ABI转换 | work T-02/03/05；区分截断/错位置、input直回伪实现 |
| V-03 | AC-01 关键浮点 | +/-0、+/-Inf、f32最小subnormal=00000001、f64最小subnormal=0000000000000001、正负qNaN及sNaN；W05保留原sNaN payload | const/copy/分支/call/return/entry全路径逐位；f32负零80000000、f64正零0000000000000000；NaN符号/payload逐bit比较，绝非is_nan | work T-03/05；不承诺浮点运算 |
| V-04 | AC-01/04 纯record布局 | mixed i32/f32/u64/f64及嵌套record Aggregate/FieldSet/FieldGet、跨函数隐藏record返回；同一record重复传参、callee更新局部副本、结果dst与输入同逻辑槽；错owner同宽负例 | 手算声明序offset/size/align；caller原值不变、副本独立、隐藏结果不覆盖live实参、RHS先快照；分别返回每个宽字段原bits；非法名义类型verify拒绝、codegen0 | work T-02/03/05；仅pure data，非wide owned/容器 |
| V-05 | AC-02/03 trusted channel | typed i32 70/-1/MIN、bool0/1；i32除零/溢出trap；错kind/bits/nonce/hash/version/宽度/参数数目/bool2/unknown-kind；截断/重复/溢出length/尾随/无帧、异常exit带帧 | value exit0且hex分别46/ffffffff/80000000、bool0/1；实际trap有具名失败且无value；形状/状态冲突在准入或父侧拒绝，错误bit/类型oracle只红对应格，missing-axis/SKIP永不PASS | work T-04/05；worker从不接expected |
| V-06 | AC-04 cache恢复 | fresh/hit/新worker restart/read-only miss；ABI/目标/签名/layout/transport/runtime/hash或required-data损坏，跨owner/import类型不匹配 | 合法hit用户codegen0但PE实际运行；每轴污染在使用前拒/重算，修复后再次restart真hit；read-only store写0 | work T-04/05；目标Win x64，不重建旧HIR后端 |
| V-07 | AC-04 精确增量 | 暖居民caller/helper+disjoint单元，helper body改变、A→B→A；换参数/nonce；cold restart；signature/layout/ABI变化与候选失败 | body仅helper prepare/verify/fp/admit/emit/persist>0且caller/disjoint各0、whole0，resident token/generation/COFF一致；PE实际新值、wrapper同身份；历史A真reuse；参数only用户COFF/wrapper生产0但实际再执行；冷恢复重验单列，依赖漂移只更新真实依赖、失败旧闭包仍可调用 | work T-04/05；typed MIR起点，无宽source前端 claim |
| V-08 | AC-05 I/O生命周期 | 子PE不读输入、超量写stdout/stderr、部分写/读失败、deadline/取消、链接/发布注入失败 | 实际受控执行器，具名resource/infra且无成功typed；budget内停止并观测进程树和reader/writer回收；旧产物hash/旧合法执行不变 | work T-02/04/05；自身拥有资源，不能误杀其他会话 |
| V-09 | AC-06 兼容及拒绝 | 旧scalar exit含负值与70、871五域、889实际交付List/Variant命名case、863 console独立路径；wide Bin/Un/转换/owned/collections/Unit等 | 旧模式身份/结果保持；受检拒绝有stage/code与codegen0；新模式不能偷改旧source默认选择 | work T-01/03/05；按affected命名case，不整档旧脚本 |
| V-10 | AC-05/06 成本/敏感性 | fresh/hit/restart/body-edit各20正式样本、循环换参/失败恢复；强制whole verify/split、复制替换未改resident仍保generation、把f32转f64/清NaN payload的受控反控 | 每状态从正式入口到结果判定/清理的P90≤2000ms，完整分位/原样本/资源峰值及close后live；各mutation确实命中真实消费路径并使对应结构/bit门红，不只改日志计数；未改义务不能SKIP | work T-05；无安全mutation时给同等直接区分与限制，不以更快whole重算当增量 |
| V-11 | AC-01/04 指纹与ABI隔离 | 同kind常量改高位/NaN payload/零符号；i64/u64/f64同bits；同owner/unit替换callee ABI或签名；合法表重排/重复primitive/死表项/span/snapshot控制 | 改语义或ABI必须不同key/受检失效；等价重映射key相同；跨ABI/import错配在link前拒，不执行错函数；真PE结果区分新旧常量，不依赖重新编译掩盖错误hit | work T-03/04/05；新typed渲染，旧域版本变更单列 |
| V-12 | AC-02/03 结果来源和晚失败 | 保留同签名但改body常量的真实PE、input固定而callee变；旧nonce帧/错误PE或wrapper/hash替换；成功帧后hang/异常exit/超量stderr | 新body的机器结果与独立oracle一致；换错产物或重放旧响应即拒，不能信worker自洽回显；晚失败整次非PASS；完整成功对照可过且value/trap/resource/error互斥 | work T-02/04/05；观察actual launch/bytes，而非“worker有spawn” |
| V-13 | AC-06 正式身份与测试发现 | 新recipe/input/MIR/args/ABI/transport/helper/worker/源锁及单unit消费者变化；合法旧/新行；从新case定位单项重放 | report真实实际身份，相关改变使旧报告STALE或重验，未改旧域不污染；decoder/dispatcher/worker/current identity同步；新typed义务被当前有效治理清单/入口发现，若888已交付则进其叶模块/affected消费者并集，否则保现行正式入口并登记后继映射，不消费888 WIP；独立A重放可用，不擅加B/C | work T-01/05；需H释放/接受与源锁刷新证据，不修改888合同 |
| V-14 | AC-05 边界与释放 | 输入/length/count/arg及record暂存/栈帧checked算术与现行限额边界；合法贴界/超限对照，深层无环与共享DAG类型、非法循环输入；多轮build失败/超时/回滚/close与GC | 分配/执行前拒对应resource，非法循环由verify拒且无codegen，非panic/宿主栈溢出；记录真实charge与48MiB分区门、worker/子PE/树物理peak及有出处的硬门；成功/失败收尾live稳定，close释放wrapper/unit，无 owned scratch/reader/writer累积 | work T-02/04/05；T-02固定边界/物理口径，不擅加语法深度上限 |

建议目标入口（T-05实现前不存在）：cargo test --manifest-path experimental/ac-mir-native/Cargo.toml --test typed_entry_tier / --test typed_entry_incremental_tier（分别运行）；新runtime manifest定向test；powershell -File scripts/verify-native-typed-891.ps1 -Scope <abi|transport|incremental|compat|acceptance>。真实包名/新增case名在T-01/02回填；禁止空选择PASS。当前只规划，不运行这些命令。

## 7. 验收标准

| ID | owner_stage | 可观测关闭条件 |
|---|---|---|
| AC-01 | work | typed六标量entry/内部wide数据闭包及pure record布局成立，物理调用约定/参数和隐藏结果所有权明确、原始bits全路径保持；V-01..04/11，未准入运算/组合codegen0 |
| AC-02 | work | 独立版本ABI/typed入口/真实PE返回值通道及绑定成立；value与失败互斥、晚失败不PASS，旧退出码模式保留；V-02/05/08/12及全链probe，不能宿主合成结果 |
| AC-03 | work | W01..05全部正式文件化、native/AV分别独立oracle，kind/bits双轴、wrong-oracle/missing/SKIP及结果来源/状态反控；V-01/05/12 |
| AC-04 | work | exact typed指纹无占位碰撞、ABI隔离、恢复重新准入/持久修复/原子保旧及暖typed MIR函数增量成立，wrapper复用、冷验分账；V-04/06/07/11，起点/source限制明确 |
| AC-05 | work | 有界I/O/分配前预算/退出清理、失败分类与四状态各20样本周转过门；V-08/10/14，不提高上限或抢888采样窗 |
| AC-06 | work | N/H及必要S逐符号交接、889真实稳定基面兼容、消费者镜像/报告stale/模块义务齐全；V-06/09/10/13，唯一lowering、无AV计算旁路、SD候选及完整证据摘要 |
| AC-07 | review | 独立逐项追溯AC/V/SD，检查截断/浮点roundtrip/复制expected/整模块carry伪增量/遗漏及warnings/fmt；最终冻结revision绑定pass，merge门显式pending |
| AC-08 | merge | final review后的冻结提交通过当时有效required gates；若888已交付按其正式affected模块并集，否则按现行裸t+tv及相关触面规则；不擅自提前套尚未交付模块门，不重复整档开发回归 |
| AC-09 | merge | SD-01..06/模块overview/plans/索引/ledger沉淀，W原行证据来源回填、残留source/算术/组合gap保留；归档、guard清理及真实交接完成，不宣称M1全绿 |

## 8. 执行步骤

| ID | owner_stage | 依赖/窗口 | 工作与验证产出 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 用户后续明确执行；只读可先，写入需对应窗 | 核树/HEAD/D门/889 r2真实N阶段及888 H状态；按§5.1列N/S/H具体符号、consumer/源锁与接受/回交；冻结实际输入/拒绝闭包、预算和V-01..14映射；只读基线不冒充已释放 | AC-01/04/06 | [✅ 已完成] |
| T-02 | work | T-01；N及所需S/H helper符号释放 | 冻结物理ABI/record复制与隐藏结果、typed文件输入/patch编码、帧状态机/来源绑定、预算/采样口径；真PE跨对象identity+constant return、六混参各位返回、最小record往返、W05、value70/trap对照、晚失败/挂起/超量I/O；不通过回本步，不缩W支持 | AC-01/02/05 | [✅ 已完成] |
| T-03 | work | T-02；N独占，改共享闭包时需对应S | emit/lower/layout/symbols/fingerprint/config的typed独立准入、调用/record及精确类型/const指纹，按请求传ABI与符号隔离；V-02/03/04/09/11命名验证，旧域/889兼容及拒绝保留 | AC-01/02/04/06 | [✅ 已完成] |
| T-04 | work | T-03；N及必要S/H helper同时释放 | typed wrapper/runtime/runner、trap失败帧、COFF与wrapper恢复/required-data、typed居民unit更新与真实生产点/身份计量、候选原子回滚/close；V-05..08/11/12/14定向，不能套scalar session全量carry | AC-02/04/05 | [ ] |
| T-05 | work | T-04；H及必要N/S同时释放、正式采样择窗 | 文件化recipe/SDK/父侧独立oracle/report stale与decoder/worker镜像；W五行及V-01..14全验收、当时实际模块发现/原子义务与源锁接纳、四状态各20样本/资源及敏感性反控；SD-01..06候选与证据摘要、提交实现，不擅加888批量白名单 | AC-01..06 | [ ] |
| T-06 | review | T-01..05完成、冻结commit | 独立final review，修复回同树work并复核受影响证据；不把work自检当final、不跑merge整档 | AC-07 | [ ] |
| T-07 | merge | T-06 pass | 冻结提交required gates与batch到期判定；代码改变则回受影响review/新freeze门禁 | AC-08 | [ ] |
| T-08 | merge | T-07 | 实际合入、SD沉淀/索引/ledger与逐行M1来源/限制读回；不改旧expected/分母 | AC-09 | [ ] |
| T-09 | merge | T-08 | archive为真实delivered并保留证据，N/H交接版本/支持/拒绝/缓存限制；如清理pending如实记录9/10而非全部完成 | AC-09 | [ ] |
| T-10 | merge | T-09 | 指定树fresh wt-guard clean后清理树/分支，核真实状态10/10与最终收据；保留其他会话/兄弟WIP | AC-09 | [ ] |

## 9. 复审记录

2026-10-30 stage: new | plan_id: PLAN-891 | plan_revision: 1 | outcome: pass（规划静态预检，非实现验收） | next: 待用户指定实施后T-01；T-02等889真实N释放，T-05等H/采样窗 | authorization: 本轮只规划 | changed: AC-01..09 / V-01..10 / T-01..10 / SD-01..05。

仅查Spec/实际代码/相关计划/合入收据与分配号源；未建树、未实现、未构建测试、未运行probe、未改889/888合同或canonical/ledger；全部任务未执行。源hash为读入快照，889后必须更新实际基面。主检出号源含他方累进，不夹带提交；实施开树前按仓规协调提交自己的合同/取号簿记。

### 2026-10-30 规划复核：r1→r2

用户授权本轮直接修订遗漏，保持drafting/0-of-10，不触889/888合同及实施树。静态核到wide类型/常量指纹占位会碰撞、逻辑符号不含ABI/签名、scalar增量会话及wrapper每次重建的实际接缝；原r1“独立ABI/精确增量”概述未把这些具体工作列全。已补物理ABI、record复制/结果暂存、结果来源/互斥失败/晚失败、typed指纹/居民wrapper、暖生产与冷准入分账，以及889 r2/888真实符号和模块源锁交接；T-01..05与AC-01..06/09同步，新增V-11..14/SD-06。r1静态预检仅留历史，不作r2验收证据，详见[复核清单](reports/ace-891-plan-review-2026-10-30.md)。没有执行任何计划测试或实现probe。

stage: new | plan_id: PLAN-891 | plan_revision: 2 | outcome: pass（规划修订静态预检，不是实施review pass） | next: 用户后续指定实施后T-01；T-02等889稳定N及必要S/H释放，T-05等公共H及采样窗 | changed: AC-01..06/09, T-01..05, V-01..14, SD-06 | pending: work全部、final review、merge门均未执行。


stage: work | plan_id: PLAN-891 | plan_revision: 2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-891/auto-lang | branch: plan-891-dev | base_commit: b55889dff | code_commit: 7dd44182a | dependency_revisions: 889/888/894已归档(窗口释放),ace-workspace@worktree,auto-web sibling@master c48924f33 | task_ids: T-01..T-05 | evidence: 见下 | blockers: 无 | next_responsible_role: review (final)

### 2026-10-31 work 交付记录（T-01..T-05）

- **T-01 基线**：master=b55889dff 开树（组 D:/autostack/.wt/lang-891，含
  auto-down/auto-web 兄弟）。窗口释放核验：889/888/894 均已 archived 10/10
  → N/S/H 写面无在途 owner。路径迁移适配：`experimental/ac-mir-native` →
  `experimental/ace/crates/native`（包 ace-native），probe=`crates/av`，
  contract=`crates/auto-test-case`，父侧=`crates/auto-lang-core`。S 共享面
  窗口（§5.1 登记）：`ace-core::link::run_deadlined_typed`（新增、纯增量、
  消费者仅 typed runner；既有执行器符号零修改）；`ace_av::cache::split_units`
  只读复用；888 的 H 合同四点（contract/dispatcher/stale/groups）均已释放。
- **T-02 ABI/transport 冻结**：`msvc-x64/priv-typed-data-v1`；帧协议
  `mn891.typed.frames/v1`（请求 32B 头+对齐 payload+fnv1a64，响应恰 44B，
  nonce/签名 id/校验绑定，value/trap/resource/error 互斥，trap 原因 1..6，
  无尾随字节）；typed-entry-runtime 为 no_std/kernel32-only 指针纪律静态库
  （一次 slice-index 会拖入 panicking core CGU——已实测并消除）。真 PE
  probe：七标量 const-return 逐位、identity 高位区分、六混参逐一选择
  （跨寄存器/栈边界）、跨对象调用、纯 record 宽字段往返、value70 vs
  除零 trap 帧互斥，全绿（typed_entry_tier）。
- **T-03 准入/指纹**：typed 准入闭包（六标量+纯值宽 record；Str/borrow/
  owned/collections/closures/Dyn 具名拒绝，宽算术拒绝）；`ac9.` 符号域与
  旧域隔离+import 闭包 link 前逐签名校验；fingerprint 宽类型按逻辑标签、
  宽常量按精确位型（同 bits 不同 kind/NaN payload/零符号不碰撞，死表项
  不入键）；layout 宽字段 8/8、4/4；lower 增 LowerMode::Typed（legacy
  行为不变——128 项 conformance 全绿证实）。
- **T-04 缓存/恢复/居民会话**：typed CoffMeta（abi 轴隔离+恢复重准入含
  typed 符号重算）；wrapper 居民缓存（签名身份键，参数/nonce 不入键；
  信封+长度+blake3+结构 COFF 重校验）；TypedSession 暖编辑（只重备受影响
  单元、whole_fallback 恒 0、A→B→A 历史对象复用、原子多单元签名变更、
  冷重启 emit 0 仍真跑）；fresh/hit/param-only 生产计数（typed_entry_tier
  + typed_entry_incremental_tier 共 15 测试绿）。
- **T-05 正式面**：recipe `ac/mir-native-typed-v1` 四点同步（contract/
  dispatcher/worker/stale）；`.mnf`（mn-typed-fixture/1，typed-fixture
  语法）文件化 fixture；expect.typed 父侧独立 oracle（canonical hex 与
  AV 同源规则）；suite typed-native-891 **12/12 PASS**（W01–W05 位型行、
  value70、除零 trap、identity、六混参选择、record 字段、跨对象调用、
  f32 分支 subnormal）；binding.tsv W01–W05 ac 列 → `ok:P891-typed`；
  SDK 对照绿 12 行+错 bits/错 kind 红控各命中具名 mismatch 轴；V-10
  四状态×20 样本 P90 实测 fresh 328/hit 424/restart 433/edit 182ms
  （门 2000ms），结构门随行；V-11 指纹隔离（kind/位型/callee 签名/ac9
  域）、V-14 具名预算（64KiB 参数区分配前拒）与循环 record 具名拒绝；
  scripts/verify-native-typed-891.ps1（abi/transport/incremental/
  performance/compat/sdk/acceptance 七档）；固定组 case-ace-m1-typed-
  native-891；SD-01 新增 docs/specs/auto-mir/native-typed-entry.md，
  SD-02..06 增量已落工作树。
- **验证汇总**：typed_entry_tier 12 绿、typed_entry_incremental_tier 3 绿、
  typed suite 12/12 PASS、SDK 2 绿（含红控）、legacy conformance 128 绿
  （V-09 兼容）、889 suite 40/40 绿、auto-test-case 79 绿；ace-core/
  ace-native/auto-test-case/auto-lang-core 编译零新增警告，rustfmt 已过。
  统一回归门（merge 冻结档）留 merge owner（AC-08）。
- 分支提交：02b8f0da4（ABI/transport/probe tier）→ 9a3a96c9e（居民会话）
  → 98c7c0cf7（recipe/suite/binding/specs）→ 7dd44182a（fmt 收口）。


stage: goal | plan_id: PLAN-891 | plan_revision: 2 | round: 1 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-891/auto-lang | commit: 7dd44182a | next: review


stage: review | review_scope: final | reviewer_identity: 同一会话执行+终审(计划未指定独立final reviewer);判决自工件重建(diff审计/代码核验/测试重放),未采信执行者总结 | plan_id: PLAN-891 | plan_revision: 2 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-891/auto-lang | branch: plan-891-dev | reviewed_commit: 7dd44182a | base_commit: b55889dff | dependency_revisions: 889/888/894 archived; master 已前进(基面后含 reader fixtures 退役等,与本计划文件无交集,merge 时 delete/unchanged 自动解决) | spec_inputs: docs/specs/auto-mir/data-place-profile.md, native-data-owned-lowering.md, auto-ac/native-backend-and-retirement.md, auto-test-case/ace-m1-conformance.md, architecture.md | acceptance_results: AC-01 partial, AC-02 partial, AC-03 pass, AC-04 partial, AC-05 partial, AC-06 pass, AC-07 本记录, AC-08/09 merge-owned pending | findings: F-01..F-06 | evidence: typed suite 12/12 + tiers 15 绿 + SDK 2 绿 + legacy conformance 128 绿重放确认;diff 审计确认无调试残留、worktree clean | next: work(同树修复 F-01..F-06 后重审)

#### 2026-10-31 final review #1 findings(needs_fix)

- **F-03(产品缺陷,AC-02/V-05/V-12,T-04)**:违反 §5.3 冻结"成功=进程
  exit0 且恰一完整合法结果;异常exit+结果均不能PASS"。`typed_pipeline::
  run_objects` 解码响应帧后未校验 `exit_code==Some(0)`——写帧后异常退出的
  PE 会以 Value 判定通过。修复 oracle:通道级强制(exit≠0 ⇒ 具名
  `mn.admit.typed-frame` 拒绝,不产 Value outcome)+ 显式负例(手工构造
  exit≠0 场景或注入校验路径的单元反例)。影响 sibling:所有 TypedRun
  消费者(session.run/worker/SDK 判定)。
- **F-01(测试缺,AC-01/V-03,T-05)**:±Inf、+qNaN 显式位型样本未落
  (现覆盖 -0/+0/W04 负NaN/sNaN/subnormal)。补 const-return 行:
  f64 +Inf 7ff0000000000000、-Inf fff0000000000000、+qNaN 7ff8000000000000。
- **F-02(测试缺,AC-04/V-06,T-05)**:损坏轴无显式负例。补:store 中
  wrapper 信封字段篡改 → 恢复 admission 具名拒;unit COFF 字节篡改
  (sha 不符)→ admit 拒。
- **F-04(测试缺,AC-04/V-11,T-05)**:import 闭包 unresolved callee →
  link 前具名拒无显式断言。补 fixture:main 调用不存在单元 →
  `mn.admit.callee-sig` 拒(不产对象)。
- **F-05(测试缺,AC-05/V-10,T-05)**:敏感性反控缺。补 codec 级位清洗
  mutation:f32/f64 结果位型翻转一位 → canonical hex 必变 → 父侧比对
  必红(证明位门真实消费路径)。无安全 whole-fallback 注入点的限制按
  计划 V-10"同等直接区分与限制"如实记录(生产计数结构门已随采样)。
- **F-06(测试缺,AC-01/V-04,T-05)**:record"callee 改局部副本、caller
  原值不变"无显式正例。补 fixture:main 建 record 传 helper,helper
  FieldSet 后 main FieldGet 原字段,期望原位型。

## 10. 待澄清事项

| ID | 事项与关闭owner | 阻止的工作 |
|---|---|---|
| D-01 | 889 r2的N ABI/runtime阶段尚未冻结释放，888 H仍在实施；T-01按§5.1核正式commit与N/S/H具体符号接受、源锁/采样窗 | 无释放不得执行对应任务；不阻止只读规划，891不自动获得S/公共H |
| D-02 | T-02真PE验证bit-carrier物理调用、pure record复制/隐藏结果、互斥帧状态机/晚失败及异步I/O回收；固定tag/schema/预算/计量样本 | probe不闭合不得广铺codegen/正式注册，constant-return不替identity；无新增待用户语义裁定 |
| D-03 | typed文件输入与单unit patch/fixture builder、父侧独立判决和正式模块发现/源锁入口；T-02/05核真实consumer | 共享面未交接不复制ac_av判决、不改888合同/冻结报告/批量白名单；新增义务不得漏出模块门 |
| D-04 | 889 wide组合仍不支持、public AutoABI/FFI/宽算术/源码不在本包；若未来需要回后续独立合同 | 不作为本包未决语义，不得借实施隐式扩面或全域放行wide |
