# ACE 能力差距与 UI 2D 3D 示例补强审计

同日规划调整已把现 AC 的共享修复分给 [AAC 与既有生产任务](../59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)。UI 原生宿主/W2 由 [1097](../../plans/1097-ace-ac-autoui-w2-and-age-host.md) 交付，AGE AC 由 1102/1104/1105 交付，1068 接收正式证据和最终三端验收。以下源码状态是审计时点记录。

2026-11-19；auto-lang观察HEAD `2b829c5d18b492d30c990dc09a0f33eeec03ce9e`。这次回答七件事：AV与AutoVM、AC与AV、标准库、A2R、A2PY、全部AV-UI示例，以及新增的2D/3D逐例递进。依据当前工作区源码、canonical Specs、已交收据和活动计划；**本轮只读静态审计，没有运行compiler/demo/GPU/GUI或测试，没有改实现、原Plan或current Spec，也没有新取计划号。** 下列“已验”均是引用既有证据，不是本轮重新验证。

**结论：ACE已有可用基础，但不能声明AV与AutoVM全语言/全产品齐平，也不能声明AV/AC/A2R/A2PY已经在同一普通工程里全能力齐平。当前最需要的是统一能力/示例分母和可复现逐项证据，并修复现有交付边界留下的承接缺口。** 不给未经逐项映射的百分比或人天估计。

## 1 当前交付与真正的支持边界

| 事项 | 当前证据 | 不代表什么 |
|---|---|---|
| M2-A | [M2-A规范](../../specs/ace-migration/m2-certification.md)的M2A-08已成立，已交普通语言/库/有限UI/AGE/OS及A范围证据 | 2026-11-16以后A范围已裁成MIN-A等最小出口；不是所有原始Auto语言、完整UI或全部平台已交 |
| AV普通能力 | 964–970等原合同已归档，具名profile和指令/资源规则有实现 | 计划叫“完整”或有一种profile成功，不等于所有源码构造/组合都支持 |
| AC异步 | [1067](../../plans/archive/1067-ace-ac-native-async-and-task-parity.md)已归档，真实Windows机器码scheduler、sleep、send/ask/reply/await/go/stop等已有 | 不再说“AC没有异步”；但动态面、平台/调度组合、资源边界及原生UI仍有差额 |
| UI基础/AGE基础 | [980](../../plans/archive/980-ace-m2-autoui.md)与[981](../../plans/archive/981-ace-m2-age.md)已按基础范围收口并移交1071–1077 | 不代表完整示例/游戏/三个后端已全部通过 |
| AV-UI W1 | [1071](../../plans/archive/1071-autoui-w1-av.md)归档；真实输入mini夹具、Hello/Counter与生产入口已有收据 | §9及Spec §14明确U02–U06完整逐例尚未关闭；不能称全部UI/Widgets完成 |
| Python生态调用 | [1026](../../plans/archive/1026-python-interop-standard-auto-syntax.md)已归档，AV→CPython/PyTorch CPU训练及资源/异常有证据 | 这不是新A2PY生成后端的完整语言/生态支持；GPU/异步Python/跨后端另有边界 |
| 三端对齐 | [1068](../../plans/1068-ace-m2b-three-backend-parity-and-aac-proof.md)仍drafting 0/10；普通能力/系统库/原生图形与AAC实证的汇总owner | 不得用其计划文本补当前后端缺口；本版AV/AC/A2R对齐是目标而不是现状 |

支持状态分四类：**已交适用面 / 明确具名拒绝 / 尚未验证 / 规范待裁定差异**。未验证不等于不支持；拒绝比静默假执行诚实，但也不等于与旧语言能力齐平。旧VM缺陷、未授权IO可捕获与无限资源不应机械复制，差异要有语言规范决定和正反例。

## 2 AV 与 AutoVM 是否齐平

**不能判齐平。** AV已具备数值/聚合/泛型、闭包、部分动态对象、控制效果/生成器、任务与模块等基础；但下面已有明确限制或具名债务。有限负例通过表示拒绝合同成立，不是功能已经实现。

| 能力族 | 当前AV/共享前门事实 | 补强要求与独立判据 |
|---|---|---|
| 管道/调用兼容 | 旧parser有`PipeGt`脱糖；[泛型Spec](../../specs/ace-migration/m2-generics.md)D-963-C1记`\|>`未实现 | 补普通调用与方法管道、单次求值、源序/错误定位；旧合法样例→ACE实际值和事件轨迹 |
| 函数值 | resolver仍对普通命名函数作值发`source.fn-as-value`（外部callback等特例不等于通用函数值） | 顶层/跨模块函数作为参数、返回、容器元素、绑定方法，配闭包/资源释放；不能只把lambda成功当全函数值成功 |
| 动态对象/字典 | [动态Spec](../../specs/ace-migration/m2-dynamic.md)冻结对象字面字段同型、循环插入拒绝、缺键trap；不是任意动态对象 | 对异构值、字段/索引、反射、身份/别名、缺键与循环策略逐项定规范；typed record不是动态对象的替代证明 |
| Option/Result/错误/生成器 | `??/.?/try/finally/yield`已有；965明确跨调用`fail`不被调用者catch捕获，借用生成器参数拒绝 | `~T/?T/!T`跨模块/返回/闭包/循环/取消/错误出口的组合；跨调用捕获语义先对规范，禁止把host权限/resource/cancel统吞成普通catch |
| 标注/泛型/方法 | 963有spec/关联类型/ext/模板等，但`#[with(T)]`等属性形、默认参数作用域、局部泛型注解与varargs兼容有具名边界 | 保留已交能力，补旧合法拼写/诊断与缺省实参绑定；不能将旧限定拼写拒绝算覆盖完成 |
| 模块与顶层文档 | AV有GlobalGet/Set等；owned记录模块全局`.move`仍在963债务；配置文档是独立profile | 全局初始化/共享/移动、包身份/依赖、顶层脚本/配置/领域声明组合；不能一个text-record配置例子证明完整Auto程序 |
| 完整CT/宏/反射 | 966的`#if/#is/#for/#in`已交的是有界装配谓词；i32/bool/已注册sys事实，1024展开上限；元数据有FieldCount/HasField | 扩任意已批准CT函数/类型/结构、字段遍历、生成语义与hygiene/递归/缓存依赖；必须区分装配折叠、真实AV CTE、静态元查询三条已存在路径，不说“无CT”也不说“完整任意CT” |
| profile可组合性 | Core/Scalar/Text/Record/Collections/Closures/Numeric/Aggregates/Scheduler/Age*/ConfigDocument分别有门 | 用同一普通工程组合类型/泛型/错误/CT/异步/FFI/标准库，证明走统一受检链；成功的分离profile数量不是语言覆盖率 |

代码锚：[旧管道parser](../../../crates/auto-lang-core/src/parser/expressions.rs)、[旧AST](../../../crates/auto-ast/src/ast.rs)、[ACE resolver](../../../experimental/ace/crates/core/src/source/resolve.rs)、[SourceProfile](../../../experimental/ace/crates/core/src/source/mod.rs)。这是已确认的最小差额清单，**不是所有旧case已经逐例审完**；下一步须把旧case/check/源码/规范→新入口/profile→独立expected逐行对齐，已知旧bug另行裁定。

## 3 AC 与 AV 的差距及补法

共同前端/Checked HIR/MIR不等于后端自动支持全部指令。当前[native/lower.rs](../../../experimental/ace/crates/native/src/lower.rs)仍明确拒绝GlobalGet/GlobalSet、DynPack/DynIs/DynTake及ObjectNew/Get/Set；AV同名指令有实现。这是具体后端差额，不能用emit成功、CRT stub、返回固定值或AV fallback补证。

| 优先批次 | 要补的当前差额 | 交付/验收要求 |
|---|---|---|
| AC-0 正确性与拒绝台账 | 1067规范仍记owned-text循环临时退出72、进程外取消源未交、owned-text消息/回复表面拒绝 | 先核当前源码/收据并复现具名边界；≥2次迭代释放平衡、外部cancel/late结果零复活、消息/回复真值。这里不把本轮未重放的规范债务称新测缺陷 |
| AC-1 值与全局 | dynamic标签/对象/别名、module globals及owned跨边界 | Native runtime真身＋source/HIR/MIR/ABI/资源准入；与AV分别对独立值/事件/资源oracle，不要求相同内存表示 |
| AC-2 库与调度组合 | SCHED专用车道已交，但`platform.*`/net等形与sched组合、OutBytes/系统库packages合流仍具名差额 | 同一个普通程序实际文件/进程/网络/任务执行，生命周期和错误/取消可观察；不能永远用“不在本profile”豁免最终公共能力 |
| AC-3 错误与工程 | LangFail在native是trap code/class，AV有message/span；完整包/FFI/重建/恢复 | 定清跨端共同错误信息/阶段/来源，测试失败保旧与仓外可搬迁真实程序 |
| AC-4 UI/2D/3D | Node桌面、真实图形输入/资源/恢复不由AV Hello自动证明 | 1097 交 UI 宿主与 W2，1102/1104/1105 以各自已交 AV 与 W2 交准确`UI-W2-AC`/`IF-AGE-GFX-AC/<cap>`，再供1074；不能在纯Rust renderer截图上冒AC业务执行 |

[原生异步Spec](../../specs/auto-ac/native-async-scheduler.md)证明“已补哪些/还剩哪些”；[1068](../../plans/1068-ace-m2b-three-backend-parity-and-aac-proof.md)接收正式差额关闭证据并最终验收，核心补齐由 Design59 实施归属表中的任务交付。

## 4 标准库离完善还有多远

**已经过了只有设计/Hello的阶段，尚未到可宣布生产完备与旧库全兼容的阶段。** 972有八模块装配、core/alloc分层和多端证据，973/1064/1065有真实Windows文件/环境/进程/存储/ZIP等，974有TCP/HTTP/SSE/WS基础，975有Auto JSON/Redis及显式SQLite provider。

但旧Spec中的“62公共符号”是历史快照；本轮对`stdlib/ace/asm/*.at`按`pub fn`静态扫描为 **66个声明**。这只是源码声明数，不能用来计算已验证API比例；须重对当前manifest/导出/别名/后端矩阵。bytes解码已经存在，不能照旧Spec再报“hex/base64/byte→Str完全未实现”。

| 工程包 | 还需完善的范围 | 现有可复用基础/关闭要求 |
|---|---|---|
| LIB-1 公共语义与基础算法 | 完整泛型map/字符串key、比较器sort、Unicode分类/大小写、数值边界/format/parse、正则/摘要等实际公共操作 | 972已有map/int、list.map/filter/sort和纯Auto math；1064已有bytes decode/byte→Str，975已有char→Str。精确摘要消费1056等真实后继，避免在产品或AAC各抄一套库 |
| LIB-2 生产网络 | HTTPS/TLS与proxy可选provider、完整Response/header/query、URL decode、背压、WS ping/pong/close/分片、可配置超时与流式文件体 | 974已有TCP与协议基础；其“char→Str未裁”落后于975原语，不重建底座。API和native机器运行腿仍须补，不以一个回环例子推全协议 |
| LIB-3 数据与持久化 | 动态JsonValue、泛型encode/反射遍历、完整number/嵌套数据和用户实际API、CSV；数据库事务/绑定/资源/迁移场景 | 975已交JSON语法walker/typed decode切片、Redis/SQLite有限真运行；不能宣称完整serde或所有数据库已原生重写 |
| LIB-4 后端与场景 | 完整AC系统/网络/OutBytes、各实际provider dispatch、无堆/有界Storage场景、仓外SDK与真实产品消费 | 1064/1065/1067是基础，1045/1051 与各产品接缝继续闭合。core-only低资源证明不外推为MCU已验；Linux在1061/1063专用环境验证，Windows不运行Linux/WSL腿 |

底层依赖必须按操作列真链：文件系统规范记录Win32直接调用；网络[platform.net实现](../../../experimental/ace/crates/av-core/src/platform_io.rs)当前能看到`std::net::TcpStream/TcpListener`。Auto编写HTTP算法不等于其TCP原语已无Rust std委托；需明确它是哪个provider/宿主层，并按默认自有底座目标补适配或记清边界。不是要求整个AV宿主/外部renderer无Rust，而是不能混淆公共标准库操作的真实实现来源。A2R/Python目标runtime依赖也应与默认AV/native provider分账。

来源：[核心库Spec](../../specs/ace-migration/m2-stdlib-core.md)、[系统库Spec](../../specs/ace-migration/m2-stdlib-io.md)、[网络Spec](../../specs/ace-migration/m2-stdlib-network.md)、[数据Spec](../../specs/ace-migration/m2-stdlib-data.md)。不能仅凭972–975 archived给出“还差10%”或固定工期，需先把这些具名公共面映射成有owner和真实oracle的剩余行。

## 5 A2R 与 A2PY 建设

### 5.1 A2R要成为与AV/AC同等的执行目标

当前M2最终三端对齐要求已在1068，不需要再发明一条A2R语言前端。复用共享Checked HIR/事实，补Rust emitter/runtime、受检FFI/宿主、工程发布和资源/错误合同。

[A2X resolve_ty](../../../experimental/ace/crates/a2x-core/src/semantic_view.rs)当前值类型有数值/text/record/list/variant/closure等，但Object/Nil/Dynamic未进入同一普通值类型集合，不能因为Rust本身可表示这些值就说A2R已支持。还要审模块全局/owned转移、跨帧错误信息、泛型与async/回调/资源组合，而不是只看生成Rust能否编译。

分三批：**A2R-1普通语言/库全矩阵 → A2R-2任务/FFI/恢复/仓外工程 → A2R-3 UI/AGE公共宿主与实际应用**。第三批沿[1074](../../plans/1074-autoui-w3-a2r.md)，两游戏发行沿[1076](../../plans/1076-age-a2r-game-2d.md)/[1077](../../plans/1077-age-a2r-game-3d.md)；Rust业务必须由新ACE生成并真实执行，纯Rust宿主窗口或旧AST emitter不能代签。

### 5.2 A2PY要单列语言与工程能力面

1042与977/PY已有真实Python 3.14.2执行、数值/文本/record/list/variant等语料及库端口。977/PY现Spec是 **9个旧语料家族代表＋8条case-line check**，旧a2p的99个字节金样未逐例迁成新语义用例；这是覆盖深度差额，不是零后端。

优先顺序：**PY-1旧公开语言义务逐例映射与统一普通工程 → PY-2 dynamic/函数值/生成器/错误/任务及模块组合 → PY-3标准库/FFI/打包/诊断/数据生态实际消费**。共享A2X类型门、Python返回类型门和运行时资源协议都要一起核；不能仅增加几个print分支。需保精确整数、C向零除余、Auto长度bytes/Unicode与显式drop/close，不能依赖Python GC掩盖Auto资源问题。

AV调用CPython/PyTorch已由1026交付，但它的Python adapter不是“将Auto生成Python”的A2PY后端。[AV Python互操作Spec](../../specs/auto-av/python-interop.md)当前还有CPU/Windows/串行、有限文本显示、动态浮点keyword拒绝、嵌套调用实参未验证等边界；这些应分别建设，不能互相挪用证据。A2PY强化先覆盖非图形公共语言/库和真实工程，不自动追加Python图形后端为本轮已批准目标。

## 6 AV UI 必须按原源逐例接管

**重要审计发现**：1071原目标/AC要求接完整U01–U06/Widgets，而交付收据和Spec §14明确只交U01输入子面＋若干生产入口，U02–U06逐例延后仍指向980 B-AV-I/V/R；但980/1071都已归档。因此剩余完整AV UI义务缺一个清晰可执行的活动承接者。不能重开归档件或把1072（Vue）当全AV应用owner；应在活动计划明确承接或新建AV逐例补全后继，迁移原行集/AC/oracle，原收据保有限边界。

更具体地，[ace_node_desktop.rs](../../../../auto-ui/crates/auto-ui-runtime/tests/ace_node_desktop.rs)的`input_fixture()`明写converter-mini（str+int、无主题/导入/f64），并注明003/011完整原源仍压④。**mini输入成功不是003温度转换器或011计算器完整通过。** 001/002的已有功能证据也只能按各自实测原源/身份范围接收。

分母：当前canonical `auto-ui/examples/ui`有 **35个编号应用**；1071 U01–U06表实际也列35个，文字却写36，需对账，不能把编号相同的`031-image-viewer`/`031-paint`误去重。Widgets canonical为 **70页＋18组件**。语言仓旧`examples/ui`与auto-os ui-gallery副本不双计；下方仅盘canonical源，generated/node_modules/产物不计源码。

每例的验收链应固定为：**当前原源/依赖/资源身份 → 共享前门/类型与效果 → AV真实状态与事件 → Iced真实窗口/输入 → 可观察业务oracle → 坏输入/坏编辑保旧 → close/late事件/资源归零**。层次分别记录，parse、结构census、mini夹具、截图或Rust renderer成功都不能互相代替。

建议顺序：001/002完整原源 → 静态/主题/基本输入与003/011完整数值 → 列表/计时/编辑/持久化 → 网络/系统/DB → 媒体/图表/绘图 → 综合应用与剩余Widgets。每个应用独立签对应cap，不必等全部35例才交真实已闭子面；不得把新画廊外壳当剩余功能本身。

## 7 2D 和 3D 从最小能力逐个增强

Gallery索引有 **12逻辑项/22子入口**（2D=12、3D=10），另有platformer-2d和platformer-3d两个完整游戏。旧Node/scene、旧AutoVM driver和新ACE session并存；旧GPU/M1可玩包不等于新AV/AC/A2R的作者行为已逐例接管。[1075](../../plans/1075-age-av-content-and-fusion.md)负责 Gallery AV，混合与游戏分别由 1099/1100/1101 交 AV。实际 AC、A2R 和发行见 [AGE 交付表](age-m2-three-route-strategy.md#后继交付与消费)。

| 梯子 | 从简单到复杂的建议 | 每步需要看到的真实结果 |
|---|---|---|
| 2D-0 | 最小窗口/画布、单shape（g01）、变位置/颜色/显示、最小输入驱动 | 原Auto源经AV；画面随一次输入变化、两实例隔离，错误源保旧 |
| 2D-1 | 父子/图层g03、相机g08、atlas/帧g09、角色g06 | 层级/坐标/相机/帧序与独立期望一致，资源销毁不泄漏 |
| 2D-2 | 查询/碰撞g05、spawn/despawn g11、音频/HUD g12、editor g04 | 真输入/查询/ID代次/暂停恢复、音频/保存/搬迁及坏资源恢复，不只静帧 |
| 2D-3 | 完整platformer-2d、混合UI、国风首切片等已有产品 | 关卡/状态/胜败重开/存档及原完整oracle，最后AV离线包→同cap AC→A2R包 |
| 3D-0 | **先有最小单方块/平面＋相机**（可从已交引擎fixture抽为公开最小例），再g03层级/g08相机 | 真Auto业务控制物体/相机；不能以GLB复杂游戏首次暴露所有前门问题 |
| 3D-1 | g02 GLB、g10 textures/animation | 真实资产加载/资源身份/变换/纹理/动画及非法资源拒绝；复用既有许可/锁 |
| 3D-2 | g05查询、g07角色、g11运行对象、g04编辑 | 独立射线/碰撞/控制/动态对象/资源事件；坏编辑保旧和同PID恢复 |
| 3D-3 | platformer-3d、混合UI/2D/3D、正式方向光/阴影扩面 | 完整游戏oracle及UI080/AG030实际cap；CPU场景/行为、GPU画面/深度、输入/音频分别验证 |

以上是按实际已交接口递进的**建议批次，不是新增Plan号或实施授权**。每个cap沿既有AV→AC→A2R接面验证，可先交已闭子面，不把全部新cap/完整游戏做完设成基础接线前置。AG029既有暂停和各产品原阶段门保持。Linux/WSL不在Windows执行，专用Linux认证保持独立，不借本次盘点删除用例。

## 8 先做什么与如何判定齐平

| 顺序 | 工作 | 现有owner与待补归属 | 阶段出口 |
|---|---|---|---|
| 1 | 冻当前语言/库/原源示例矩阵，核已交收据；修正35/36与62/66等口径 | 985能力/证据汇总＋各生产者；1071剩余AV义务须活动后继，不在归档件留悬空todo | 每项有规范/源/原oracle/目标/profile/平台/provider/owner/状态，未知不充PASS |
| 2 | 公共前端补缺＋AC高风险资源/动态/全局，A2R/PY共享值域并行 | 1046–1051 对应共享语义、MIR 与 native 执行任务；963/960等归档债务迁活跃后继，不能在某个demo临时补第二语义系统 | 同源正/负例在目标端真实执行；错误阶段/资源/失效清晰 |
| 3 | 系统/网络/数据库与SDK组合，补普通非图形工程 | 1045、1051、1052 及 std/runtime/FFI 具名生产者，A2PY需独立强化承接 | 文件/网络/任务/JSON/FFI同工程闭环，默认实现依赖与各target差异明确 |
| 4 | AV UI按35应用与70 Widgets逐例；2D/3D最小demo同时开梯子 | 待补AV UI后继；AGE1075；共享cap由UI/语言owner | 真原源、业务事件/输入/画面/资源结果，逐例/逐cap交收据 |
| 5 | 同cap AC与A2R、仓外包，最后全量产品/旧实现退役 | UI 基线 1097→1074；AGE 按各域交付后接 1076/1077 及原产品认证 | 不能以三端交集削小分母；各端各自对独立oracle，全必需行无未知/未跑才签 |

“齐平”的目标是经规范确认的共同公共能力在AV/AC/A2R（新增建设中的A2PY列同表）可用：同一原源或规范化后可审计源、同一公共语义、实际执行和生命周期等价；不要求同一内部布局/调度实现，也不复制旧VM已知bug。UI/2D/3D把渲染与业务运行证据分开。现有case框架/worker/固定组足够承接，需补分母和反例，不因这轮又建另一测试框架。

## 9 全量源码清单 静态未运行

下表逐入口读取源目录并记录输入闭包摘要，仅为当前静态盘点；**没有运行这些应用，也不以词法触面标签判compiler支持/拒绝**。属性、handler与依赖的真正准入以原源driver验证。哈希绑定本次读到的源内容，不替正式依赖/资源SDK/工具链全闭包收据。

### 9.1 UI 编号应用

| 当前入口 | 组 | src内Auto文件数 / 源集合sha16 | 当前已核证据边界与下一步 |
|---|---|---|---|
| [001-helloworld](../../../../auto-ui/examples/ui/001-helloworld/src/front/app.at) | U01 | 1 / `caee6aa1eb4b03e6` | 已有首窗/计数功能见证；仍须绑定此当前原源与完整应用oracle，不能只复用mini |
| [002-counter](../../../../auto-ui/examples/ui/002-counter/src/front/app.at) | U01 | 1 / `d7f5d947fd3806f0` | 已有首窗/计数功能见证；仍须绑定此当前原源与完整应用oracle，不能只复用mini |
| [003-converter](../../../../auto-ui/examples/ui/003-converter/src/front/app.at) | U01 | 1 / `565d32c6b1de535f` | converter-mini输入已验；测试明确排除本app完整导入/主题/f64，完整原源待验证 |
| [004-profile-card](../../../../auto-ui/examples/ui/004-profile-card/src/front/app.at) | U02 | 1 / `ebd316743bf0c73c` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [005-login](../../../../auto-ui/examples/ui/005-login/src/front/app.at) | U02 | 1 / `572fcc192821ab1c` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [006-hero-section](../../../../auto-ui/examples/ui/006-hero-section/src/front/app.at) | U02 | 1 / `3b8ba013230a34c5` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [007-stats-board](../../../../auto-ui/examples/ui/007-stats-board/src/front/app.at) | U02 | 1 / `8bb3143eb5ada6e9` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [008-pricing-table](../../../../auto-ui/examples/ui/008-pricing-table/src/front/app.at) | U02 | 1 / `2e93dde41f57b811` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [009-article-feed](../../../../auto-ui/examples/ui/009-article-feed/src/front/app.at) | U02 | 1 / `14753f4dd9e431fa` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [010-contact-form](../../../../auto-ui/examples/ui/010-contact-form/src/front/app.at) | U02 | 1 / `6c0a90fb3a3184df` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [011-calculator](../../../../auto-ui/examples/ui/011-calculator/src/front/app.at) | U01 | 2 / `dc179db19efdd6f2` | converter-mini输入已验；测试明确排除本app完整导入/主题/f64，完整原源待验证 |
| [012-clock](../../../../auto-ui/examples/ui/012-clock/src/front/app.at) | U03 | 1 / `98bec87b6fba5678` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [013-todo](../../../../auto-ui/examples/ui/013-todo/src/front/app.at) | U03 | 6 / `a2039cb60ff0156b` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [014-weather](../../../../auto-ui/examples/ui/014-weather/src/front/app.at) | U04 | 3 / `4d485453423ee2ac` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [015-notes](../../../../auto-ui/examples/ui/015-notes/src/front/app.at) | U03 | 7 / `48a14d69d0f04131` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [016-calendar](../../../../auto-ui/examples/ui/016-calendar/src/front/app.at) | U03 | 3 / `a8d288df537ce61a` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [017-chat](../../../../auto-ui/examples/ui/017-chat/src/front/app.at) | U04 | 8 / `65d6d26b35b245d3` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [018-book-reader](../../../../auto-ui/examples/ui/018-book-reader/src/front/app.at) | U03 | 13 / `a628176e48862e0f` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [019-video-app](../../../../auto-ui/examples/ui/019-video-app/src/front/app.at) | U05 | 7 / `a16c121e2e502bb5` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [020-music-player](../../../../auto-ui/examples/ui/020-music-player/src/front/app.at) | U05 | 9 / `86c735a5fafd569c` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [021-blog-viewer](../../../../auto-ui/examples/ui/021-blog-viewer/src/front/app.at) | U06 | 7 / `e1deca9fd57a2648` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [022-kanban](../../../../auto-ui/examples/ui/022-kanban/src/front/app.at) | U03 | 5 / `d1b6b4940b8ed834` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [023-realworld](../../../../auto-ui/examples/ui/023-realworld/src/front/app.at) | U04 | 12 / `a104a17580693cf2` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [024-charts](../../../../auto-ui/examples/ui/024-charts/src/front/app.at) | U05 | 7 / `fa962b7a11f87d13` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [025-sys-monitor](../../../../auto-ui/examples/ui/025-sys-monitor/src/front/app.at) | U04 | 9 / `bdfb1e8f37c2743f` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [026-database](../../../../auto-ui/examples/ui/026-database/src/front/app.at) | U04 | 5 / `f9c75ca554394169` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [027-file-manager](../../../../auto-ui/examples/ui/027-file-manager/src/front/app.at) | U04 | 8 / `c71a28a806b6cd67` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [028-launcher](../../../../auto-ui/examples/ui/028-launcher/src/front/app.at) | U06 | 1 / `49da319f9fae1806` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [029-photo-gallery](../../../../auto-ui/examples/ui/029-photo-gallery/src/front/app.at) | U05 | 2 / `15b593488deaf8b2` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [030-video-player](../../../../auto-ui/examples/ui/030-video-player/src/front/app.at) | U05 | 6 / `faaab8210585cc42` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [031-image-viewer](../../../../auto-ui/examples/ui/031-image-viewer/src/front/app.at) | U05 | 5 / `91b3ed65ffe55581` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [031-paint](../../../../auto-ui/examples/ui/031-paint/src/front/app.at) | U05 | 2 / `9752e08c60469820` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [032-handler-async-probe](../../../../auto-ui/examples/ui/032-handler-async-probe/src/front/app.at) | U06 | 1 / `1c316b6b5c0db9de` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [038-minesweeper](../../../../auto-ui/examples/ui/038-minesweeper/src/front/app.at) | U06 | 2 / `a44d0d559dfd5a85` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |
| [041-auto-edit](../../../../auto-ui/examples/ui/041-auto-edit/src/front/app.at) | U06 | 6 / `73c777b9f670f6b6` | 1071记U02–U06逐例未闭；有组件/会话见证不等本app全功能已验 |


### 9.2 Widgets 每页均保留

以下70页均为canonical输入；组件族有已有生产见证，但本轮未核到逐页完整AV原源收据，不标全绿。70页之后还需把18个components、公共主题/资源依赖逐页纳入各自输入闭包。

| Widget页面 | 内容sha16 | AV逐页状态 |
|---|---|---|
| [absolute](../../../../auto-os/widgets-gallery/src/front/pages/absolute.at) | `211a055b5115eadc` | 当前原源逐页待验证 |
| [accordion](../../../../auto-os/widgets-gallery/src/front/pages/accordion.at) | `498cb3fdd9a50693` | 当前原源逐页待验证 |
| [alert](../../../../auto-os/widgets-gallery/src/front/pages/alert.at) | `4df6bde58333d695` | 当前原源逐页待验证 |
| [alertdialog](../../../../auto-os/widgets-gallery/src/front/pages/alertdialog.at) | `0055ac1a40bd2c66` | 当前原源逐页待验证 |
| [alignment](../../../../auto-os/widgets-gallery/src/front/pages/alignment.at) | `5a580e2f73daf868` | 当前原源逐页待验证 |
| [area-chart](../../../../auto-os/widgets-gallery/src/front/pages/area-chart.at) | `cb1d1a8e43d60e2b` | 当前原源逐页待验证 |
| [aspectratio](../../../../auto-os/widgets-gallery/src/front/pages/aspectratio.at) | `0844d5314d097c6d` | 当前原源逐页待验证 |
| [avatar](../../../../auto-os/widgets-gallery/src/front/pages/avatar.at) | `c55b1e0f8ed02549` | 当前原源逐页待验证 |
| [badge](../../../../auto-os/widgets-gallery/src/front/pages/badge.at) | `f6e0dbc1c9f40c5e` | 当前原源逐页待验证 |
| [bar-chart](../../../../auto-os/widgets-gallery/src/front/pages/bar-chart.at) | `7ba1491730417c4e` | 当前原源逐页待验证 |
| [breadcrumb](../../../../auto-os/widgets-gallery/src/front/pages/breadcrumb.at) | `9a4f3c0f53f221ec` | 当前原源逐页待验证 |
| [button](../../../../auto-os/widgets-gallery/src/front/pages/button.at) | `85b0b0f8c97d0193` | 当前原源逐页待验证 |
| [calendar](../../../../auto-os/widgets-gallery/src/front/pages/calendar.at) | `9409a83d60265901` | 当前原源逐页待验证 |
| [card](../../../../auto-os/widgets-gallery/src/front/pages/card.at) | `055f26424c93fdec` | 当前原源逐页待验证 |
| [carousel](../../../../auto-os/widgets-gallery/src/front/pages/carousel.at) | `f43f17c29a83924a` | 当前原源逐页待验证 |
| [center](../../../../auto-os/widgets-gallery/src/front/pages/center.at) | `51ba877a6f605ed0` | 当前原源逐页待验证 |
| [checkbox](../../../../auto-os/widgets-gallery/src/front/pages/checkbox.at) | `2881c9f8a7f746f8` | 当前原源逐页待验证 |
| [code-editor](../../../../auto-os/widgets-gallery/src/front/pages/code-editor.at) | `955d84b26fda302e` | 当前原源逐页待验证 |
| [col](../../../../auto-os/widgets-gallery/src/front/pages/col.at) | `cf80a058a04988e3` | 当前原源逐页待验证 |
| [collapsible](../../../../auto-os/widgets-gallery/src/front/pages/collapsible.at) | `9caccfa423a99895` | 当前原源逐页待验证 |
| [combobox](../../../../auto-os/widgets-gallery/src/front/pages/combobox.at) | `8ffe192c222bf729` | 当前原源逐页待验证 |
| [command](../../../../auto-os/widgets-gallery/src/front/pages/command.at) | `c7489992a060f6b3` | 当前原源逐页待验证 |
| [contextmenu](../../../../auto-os/widgets-gallery/src/front/pages/contextmenu.at) | `a2afe5860fac26e6` | 当前原源逐页待验证 |
| [datatable](../../../../auto-os/widgets-gallery/src/front/pages/datatable.at) | `5fb5672135a860d3` | 当前原源逐页待验证 |
| [datepicker](../../../../auto-os/widgets-gallery/src/front/pages/datepicker.at) | `8a1d03657469fefc` | 当前原源逐页待验证 |
| [dialog](../../../../auto-os/widgets-gallery/src/front/pages/dialog.at) | `f78beae423ed37c5` | 当前原源逐页待验证 |
| [donut-chart](../../../../auto-os/widgets-gallery/src/front/pages/donut-chart.at) | `1c8fa2c973bd07c9` | 当前原源逐页待验证 |
| [drawer](../../../../auto-os/widgets-gallery/src/front/pages/drawer.at) | `4d2a05cb8eca0a1e` | 当前原源逐页待验证 |
| [dropdownmenu](../../../../auto-os/widgets-gallery/src/front/pages/dropdownmenu.at) | `1d74a4dee7d1e441` | 当前原源逐页待验证 |
| [filetree](../../../../auto-os/widgets-gallery/src/front/pages/filetree.at) | `02ad63b9a756c125` | 当前原源逐页待验证 |
| [flex](../../../../auto-os/widgets-gallery/src/front/pages/flex.at) | `18b5078dd60907a9` | 当前原源逐页待验证 |
| [flow-diagram](../../../../auto-os/widgets-gallery/src/front/pages/flow-diagram.at) | `b31dbab67e4993d1` | 当前原源逐页待验证 |
| [form](../../../../auto-os/widgets-gallery/src/front/pages/form.at) | `35167e6258ab1440` | 当前原源逐页待验证 |
| [grid-span](../../../../auto-os/widgets-gallery/src/front/pages/grid-span.at) | `039db2952bb78aca` | 当前原源逐页待验证 |
| [grid](../../../../auto-os/widgets-gallery/src/front/pages/grid.at) | `b9afa27d72d57765` | 当前原源逐页待验证 |
| [hovercard](../../../../auto-os/widgets-gallery/src/front/pages/hovercard.at) | `8800e9924f5a2e40` | 当前原源逐页待验证 |
| [index](../../../../auto-os/widgets-gallery/src/front/pages/index.at) | `b61d2f6ef99016c8` | 当前原源逐页待验证 |
| [input](../../../../auto-os/widgets-gallery/src/front/pages/input.at) | `4231757a0706e8e9` | 当前原源逐页待验证 |
| [kitchen-sink](../../../../auto-os/widgets-gallery/src/front/pages/kitchen-sink.at) | `83237696fcbc5d4c` | 当前原源逐页待验证 |
| [label](../../../../auto-os/widgets-gallery/src/front/pages/label.at) | `5dfa76dcd7beb4f4` | 当前原源逐页待验证 |
| [line-chart](../../../../auto-os/widgets-gallery/src/front/pages/line-chart.at) | `c474d0cc89c05247` | 当前原源逐页待验证 |
| [menubar](../../../../auto-os/widgets-gallery/src/front/pages/menubar.at) | `b518f10a5988bd5c` | 当前原源逐页待验证 |
| [navigationmenu](../../../../auto-os/widgets-gallery/src/front/pages/navigationmenu.at) | `7c9c26a169d76e3d` | 当前原源逐页待验证 |
| [pagination](../../../../auto-os/widgets-gallery/src/front/pages/pagination.at) | `a36e8567ecd6eecd` | 当前原源逐页待验证 |
| [popover](../../../../auto-os/widgets-gallery/src/front/pages/popover.at) | `a5625d9859364c07` | 当前原源逐页待验证 |
| [position](../../../../auto-os/widgets-gallery/src/front/pages/position.at) | `2dcba0206d937c48` | 当前原源逐页待验证 |
| [progress](../../../../auto-os/widgets-gallery/src/front/pages/progress.at) | `e059b1e85e6d7c1c` | 当前原源逐页待验证 |
| [radiogroup](../../../../auto-os/widgets-gallery/src/front/pages/radiogroup.at) | `5befbc2e2d92ab26` | 当前原源逐页待验证 |
| [responsive](../../../../auto-os/widgets-gallery/src/front/pages/responsive.at) | `fe5ef0f0cd3f1111` | 当前原源逐页待验证 |
| [row](../../../../auto-os/widgets-gallery/src/front/pages/row.at) | `b309d79e8fb5c27e` | 当前原源逐页待验证 |
| [scroll](../../../../auto-os/widgets-gallery/src/front/pages/scroll.at) | `f3763b50a7318d28` | 当前原源逐页待验证 |
| [scrollarea](../../../../auto-os/widgets-gallery/src/front/pages/scrollarea.at) | `4e2753a584517495` | 当前原源逐页待验证 |
| [select](../../../../auto-os/widgets-gallery/src/front/pages/select.at) | `79d89538bfabba93` | 当前原源逐页待验证 |
| [separator](../../../../auto-os/widgets-gallery/src/front/pages/separator.at) | `868605cbb80317ce` | 当前原源逐页待验证 |
| [sheet](../../../../auto-os/widgets-gallery/src/front/pages/sheet.at) | `68457c6e11a53a9c` | 当前原源逐页待验证 |
| [sidebar](../../../../auto-os/widgets-gallery/src/front/pages/sidebar.at) | `a5fe5ab7ee0a3abe` | 当前原源逐页待验证 |
| [sizing](../../../../auto-os/widgets-gallery/src/front/pages/sizing.at) | `aa31a13b50efc59f` | 当前原源逐页待验证 |
| [skeleton](../../../../auto-os/widgets-gallery/src/front/pages/skeleton.at) | `39088e4187ee2a66` | 当前原源逐页待验证 |
| [slider](../../../../auto-os/widgets-gallery/src/front/pages/slider.at) | `53e490735f08200c` | 当前原源逐页待验证 |
| [sonner](../../../../auto-os/widgets-gallery/src/front/pages/sonner.at) | `16417f5c838e8bec` | 当前原源逐页待验证 |
| [spacing](../../../../auto-os/widgets-gallery/src/front/pages/spacing.at) | `8011fb71788c5407` | 当前原源逐页待验证 |
| [switch](../../../../auto-os/widgets-gallery/src/front/pages/switch.at) | `f496039dbe4e75e8` | 当前原源逐页待验证 |
| [table](../../../../auto-os/widgets-gallery/src/front/pages/table.at) | `cdc36d88dcebe82c` | 当前原源逐页待验证 |
| [tabs](../../../../auto-os/widgets-gallery/src/front/pages/tabs.at) | `181ab589e7436a5e` | 当前原源逐页待验证 |
| [textarea](../../../../auto-os/widgets-gallery/src/front/pages/textarea.at) | `a1e673a05d536590` | 当前原源逐页待验证 |
| [toast](../../../../auto-os/widgets-gallery/src/front/pages/toast.at) | `9d89a1e1bc4bac4d` | 当前原源逐页待验证 |
| [toggle](../../../../auto-os/widgets-gallery/src/front/pages/toggle.at) | `e4911010b5d66019` | 当前原源逐页待验证 |
| [togglegroup](../../../../auto-os/widgets-gallery/src/front/pages/togglegroup.at) | `e35180de8d658570` | 当前原源逐页待验证 |
| [tooltip](../../../../auto-os/widgets-gallery/src/front/pages/tooltip.at) | `1f50eb8bb1a7e545` | 当前原源逐页待验证 |
| [treeview](../../../../auto-os/widgets-gallery/src/front/pages/treeview.at) | `4b8f2f09b2974bfd` | 当前原源逐页待验证 |


### 9.3 Gallery 每个2D与3D入口

目录链接给定位；正式实现前由1075/T-01按既有gallery loader/manifest解析canonical Node源/variant、资源、引擎锁和原oracle，排除generated/旧副本，不凭README旧引擎运行记录签新ACE通过。

| Gallery子入口 | 维度 / 类别 | 当前源码目录 | 新ACE后续证据 |
|---|---|---|---|
| g01/main | 2d / public-auto | [g01-shapes](../../../../age-examples/examples/g01-shapes) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g02/main | 3d / public-auto | [g02-glb](../../../../age-examples/examples/g02-glb) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g03/2d | 2d / public-auto | [g03-hierarchy](../../../../age-examples/examples/g03-hierarchy) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g03/3d | 3d / public-auto | [g03-hierarchy](../../../../age-examples/examples/g03-hierarchy) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g04/2d | 2d / editor-tutorial | [g04-editor](../../../../age-examples/examples/g04-editor) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g04/3d | 3d / editor-tutorial | [g04-editor](../../../../age-examples/examples/g04-editor) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g05/2d | 2d / public-auto | [g05-queries](../../../../age-examples/examples/g05-queries) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g05/3d | 3d / public-auto | [g05-queries](../../../../age-examples/examples/g05-queries) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g06/layout-a | 2d / public-auto | [g06-character2d](../../../../age-examples/examples/g06-character2d) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g06/layout-b | 2d / public-auto | [g06-character2d](../../../../age-examples/examples/g06-character2d) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g07/layout-a | 3d / public-auto | [g07-character3d](../../../../age-examples/examples/g07-character3d) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g07/layout-b | 3d / public-auto | [g07-character3d](../../../../age-examples/examples/g07-character3d) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g08/2d | 2d / public-auto | [g08-runtime-cameras](../../../../age-examples/examples/g08-runtime-cameras) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g08/3d | 3d / public-auto | [g08-runtime-cameras](../../../../age-examples/examples/g08-runtime-cameras) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g09/atlas | 2d / public-auto | [g09-sprites-and-frames](../../../../age-examples/examples/g09-sprites-and-frames) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g09/character | 2d / public-auto | [g09-sprites-and-frames](../../../../age-examples/examples/g09-sprites-and-frames) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g11/2d | 2d / public-auto | [g11-runtime-objects](../../../../age-examples/examples/g11-runtime-objects) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g11/3d | 3d / public-auto | [g11-runtime-objects](../../../../age-examples/examples/g11-runtime-objects) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g10/textures | 3d / public-auto | [g10-textured-models](../../../../age-examples/examples/g10-textured-models) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g10/animation | 3d / public-auto | [g10-textured-models](../../../../age-examples/examples/g10-textured-models) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g12/audio | 2d / public-auto | [g12-services](../../../../age-examples/examples/g12-services) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |
| g12/hud | 2d / public-auto | [g12-services](../../../../age-examples/examples/g12-services) | 1075 AV→1102 AC→1103 A2R；当前本轮未运行 |

两个完整游戏：[platformer-2d原源](../../../../age-examples/games/platformer-2d/src/main.at)、[platformer-3d原源](../../../../age-examples/games/platformer-3d/src/main.at)。完整游戏不是22个Gallery入口的替代物，也不能以简单Demo绿就签游戏完成。


### 9.4 核心库源码声明清单

当前asm八文件只按顶层pub fn文本扫描；不将声明数冒导出/执行覆盖数。用它与旧62符号Spec及实际装配manifest对账。

| 模块 | 当前pub fn声明数 | 名称 |
|---|---|---|
| [bytes.at](../../../stdlib/ace/asm/bytes.at) | 11 | byte_at, utf8_len, byte_of, byte_hash, to_hex, to_base64, decode_utf8, decode_hex, decode_base64, decode_hex_str, decode_base64_str |
| [char.at](../../../stdlib/ace/asm/char.at) | 7 | is_digit, is_alpha_ascii, is_alphanum_ascii, is_ident_ascii, is_whitespace, to_upper_ascii, to_lower_ascii |
| [conv.at](../../../stdlib/ace/asm/conv.at) | 4 | to_int, is_uint, from_uint, from_int |
| [hash.at](../../../stdlib/ace/asm/hash.at) | 1 | codepoint_hash |
| [list.at](../../../stdlib/ace/asm/list.at) | 11 | sum, find_first_gt, pop, total_len, reduce, find, any, all, filter, map, sort |
| [map.at](../../../stdlib/ace/asm/map.at) | 6 | map_new, map_put, map_get, map_has, map_len, map_remove |
| [math.at](../../../stdlib/ace/asm/math.at) | 10 | abs, min, max, square, cube, abs_f, sqrt, sin, cos, powf |
| [text.at](../../../stdlib/ace/asm/text.at) | 16 | byte_len, scalar_len, char_code_at, equal, starts_with, ends_with, contains, repeat, upper_ascii, lower_ascii, find, reverse, trim, replace, split, lines |


### 9.5 关键取证内容指纹

| 实际代码/索引 | SHA256 |
|---|---|
| [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |
| [lower.rs](../../../experimental/ace/crates/native/src/lower.rs) | `fd0abb7036ab8bd03c87ce4d970e64040af669ea3adc76f1ba27eb723a9c099a` |
| [semantic_view.rs](../../../experimental/ace/crates/a2x-core/src/semantic_view.rs) | `47c9d4b505f0a91f197b4f2b359d46613f688eb8e048370c453ec88831ea8eb8` |
| [lower.rs](../../../experimental/ace/crates/a2x-python/src/lower.rs) | `ba349a25f77815f8fc007aba9fc8f97bb12728de04841a1fd533afc41c787831` |
| [platform_io.rs](../../../experimental/ace/crates/av-core/src/platform_io.rs) | `b54f9d83664f1df4c2f58392f3d7b9311fad74f5c8e49faea874ce6c55e00d02` |
| [ace_node_desktop.rs](../../../../auto-ui/crates/auto-ui-runtime/tests/ace_node_desktop.rs) | `777271a9e8f45eb04f64ebe58e43582b87424aff5ad28f535ad29a19d73b99ce` |
| [index.json](../../../../age-examples/catalog/index.json) | `8e6375b7d7a8910743a18a1f40854ecfac1b0bc2f8df4ea3d79baa995e2480d1` |

盘点完成，状态不变：所有本轮未运行项保持未验证；未修改任何原计划/归档/Spec、未打tag/签章、未创建实施树。后续若正式编补强计划，必须接走本文点名的悬空义务，保原分母/独立oracle及Windows/Linux环境裁定。
