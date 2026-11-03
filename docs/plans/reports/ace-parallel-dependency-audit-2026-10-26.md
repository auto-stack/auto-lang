# 823–826实际依赖审计与模块并行合同

> 后续验证phase修订：[852底座与823～826统一验证交接](ace-852-baseline-and-capability-validation-2026-10-26.md)。
> 原T-10独立模块/公共核心单写规则保持；当前计划新增T-11/AC-11，任务总数与revision以各Plan最新合同为准。
> 852先交付底座、不等M1全绿；本报告下文的10任务及旧进度属于本次并行审计的历史基线。

2026-10-26用户授权：审计823–826，尽量并行独立模块，明确共享核心单写/交接；仅修订计划和战略，不启动实现、建实施树、联系其他会话或合入代码。此合同细化[ACE执行顺序](ace-execution-order-and-incremental-compatibility-2026-10-20.md)，取代旧823→824整计划串行和“所有实现都等核心窗口”的安排；821→822及v0.6 Rust ACE/v0.7 AACE边界不变。

## 1. 已核实事实与依赖裁定

输入提交、文件指纹见[inputs](../attachments/ace-parallel-20261026/inputs.json)。调查初始821已archived r6/9 of 9，交付提交92b8daf60；822为executing r5/6 of 9。取材期间其他审查者写入822 R1 needs_fix并重开至4 of 9（列表预算提交顺序及真实增量consumer门），两次观察都不满足D822。本轮保留其原始输入hash及后续观察，不修改822合同、代码或Spec，也不代其复审；实施时再核当前交付状态。

| Plan | 实际消费与代码证据 | 裁定 |
|---|---|---|
| 823 | owned捕获的分配/移动/清理用821；首例含List<record>，完整环境类型/集合资源用822；新环境/间接调用不是现有Fn直接调用 | T-01和T-10独立模块可在D821/D828后做；T-02全路径及集成仍等D822，不从822 WIP摘实现 |
| 824 | 白名单为标量/char/Str/owned record/List/Option/Result/enum；计划§5.1明确Fn/闭包/借用/资源拒绝，不消费823环境构造/调用/清理指令 | **移除D823，T-02集成等D822**；可与823并行。T-10先验证821既有值子集；新集合仍等D822。禁止为去依赖放松递归白名单或闭包拒绝 |
| 825 | 私有PC索引/装载表示读取已交付MirFunction/CheckedMir；不消费822集合或823/824新语义；真实派发仍在av.rs私有Frame/ModuleView | T-10可独立开发派生表，T-02/T-03实际loader/dispatch等核心窗口；不等数据链，不另写一套解释器 |
| 826 | 编码需要语义MIR/指令合同与版本，并不需要825私有帧布局；最终装载执行、共同benchmark与loaded体积依赖825 | T-10编码/解码模块可基于D798/D828独立开发；**T-02全路径及最终集成仍等D825**，不能把独立codec通过当完整执行/825已交付 |

当前`MirType`只有已交付标量、Fn、record与owned文本/view类别；主检出无List/Variant/Closure/Dyn新增类型，不把拟议路径当实现。`mir_verify::verify_mir`是CheckedMir唯一构造入口；`abc::DecodedAbc`公开但`abc_verify::LoadedModule`字段私有、verify_abc复核profile/version/owner。`FunctionCode::admit_unit`与ProgramSnapshot负责受检单元和链接。`av.rs::ModuleView`为pub(crate)、Frame私有，外部模块不能假称已有可插拔executor API。

821 r6只交付静态类型/借用隔离，MutRefV没有解引用入口，不承诺运行期旧帧/跨store/generation回绕校验。823的新borrow capture/frame-env访问、824的新borrow_as/typed host访问必须由各自T-01冻结可执行保护与T-02/03实际实现/反例证明，不能假借821未交付的运行期保证。

## 2. 拆成两个实施门

每Plan保留原T-01..09，新增work **T-10独立模块**（total 9→10，原进度0不变），顺序为T-01→T-10，之后满足集成门进入T-02原全路径及T-03..06/T-09，再独立review/merge。T-10完成是模块检查结果，不是Plan验收、reviewed或D-N；原首个全路径T-02仍同时纳入真实store/new-worker、AutoDown、端到端trace、P95/内存与全部后续反控。

| Plan | T-10真实独立工作 | T-02/最终集成门 |
|---|---|---|
| 823 | 单一生产源`core/src/closures.rs`：821 Str/record捕获准备/提交、owned环境容器/销毁、分配失败不消费源；不虚构间接调用或List能力 | D822+D828；环境/签名/opcode/verifier/dispatch/租约绑定及首例真实AV |
| 824 | 单一生产源`core/src/{dynamic,portable}.rs`：已有scalar/Str/record的类型擦除、错型take保源、portable树验证/冻结/恢复与故障清理；不声称已形成新MIR执行profile | D822+D828；List/Variant白名单、Dyn指令/typed host/真实AV；不等D823 |
| 825 | 单一生产源`core/src/loaded_program.rs`：从真实CheckedMir/已授信函数单元构造私有PC/参数映射表，边界/预算与身份检查；不另起dispatch循环 | D798+D828、当前公共核心窗口；真实三执行入口与单元派发兼容，旧data/owned路径保留 |
| 826 | 单一生产源`core/src/compact_abc.rs`：新版字节编解码、目录/长度/数值域/密度/有界拒绝；只产未授信MirModule，经真实verify_mir核合法性 | D825+D828、当前公共核心窗口；新wire正式路由→真实准入→D825表示→AV及store/new-worker/benchmark |

**同源、可编译、不复制**：各Plan在唯一实施树内拟建独立测试workspace `experimental/ac-av-probe/parallel/<NNN>/`（新路径，尚不存在），读取同树已合入auto-av-core公开API、跟踪自己的lock。可用路径模块编译上述同一生产源，并在harness crate root重导出已交付core模块以解析`crate::mir/contract/...`，不改core/lib.rs或父workspace members；或采用等价且无环的单源布局。T-01冻结路径/依赖方向和公开可达符号，T-10第一件事是最小可编译接缝；私有字段不足时登记具体缺口，只推进已满足部分，申请小的正式兼容导出窗口，不能深clone/复制core、fake类型、unsafe绕过或依赖WIP。

测试workspace是宿主/构建壳，不复制语义IR、值体系、store或VM；最终生产adapter直接调用同一源文件函数。阶段临时adapter不得成为第二实现，文件/函数身份和差异清单由T-03/T-05及独立review核对。T-10不创建公共profile/opcode/注册，不能通过小模型模拟“全路径已过”。新源目录/依赖若会造成core↔扩展循环，先改为同源harness方案，不提交循环。

## 3. 写面、单写与复审期间的并行

| 写入类别 | 默认owner与边界 | 能否并行 |
|---|---|---|
| 私有新增源与测试壳 | 823 closures；824 dynamic/portable；825 loaded_program；826 compact_abc；各自parallel/<id>、定向fixture/报告 | 可以，各一树，限定未注册的私有源；不碰别人文件，不在master落代码 |
| 公共类型/指令/信任/生命周期/执行路径 | `mir.rs,contract.rs,instr_spec.rs,ownership.rs,place.rs,layout.rs,mir_verify.rs,abc.rs,abc_verify.rs,av.rs,function_code.rs,program_snapshot.rs,frame_allocation.rs`及受影响调用者 | **默认一个核心窗口**；不同文件若改变同一类型/所有权/版本合同也不算互不影响 |
| 公共装配与测试注册 | core/probe lib.rs与manifest、HIR adapter、共用driver/session/cache/worker、实际recipe/dispatch/Schema与版本分配 | 同样单写；只追加一个条目也先同步前项合入再交接，不能与核心写者平行改耦合合同 |
| 资源测量/统一门禁 | 原有机器单实例测量与merge-owned整档 | 串行，不与其他重负载争抢；独立模块定向轻量测试可并行 |

窗口登记必须有：Plan/revision、已合入base commit、输入/Schema/profile/wire/语义版本、具体文件/符号、只读依赖、消费者、开始/释放与下一owner。取得窗口前满足本阶段D门且前owner明确释放，不能拿窗口等上游。core枚举tag/opcode/版本/recipe编号由当前核心owner统一登记，其他Plan在T-01只提出请求，不自行同时占号。

**复审期间不让整个队列空转**：被审树保持冻结，只读review和repair仍按原工作流；其他Plan继续T-01/T-10的无共享写面工作。若前项有公共核心修改，窗口通常保持至相关修复/合入/收据完成；不能仅因“代码已冻结”就将未交付公共定义交下游。前项没有公共写面时登记“无公共面”，不占核心窗。只读review、不同私有模块开发可以同时进行；不启动跨分支复制或在review树直接修码。

每次交接前owner给出冻结/合入提交、API/版本差异、实际profile/消费者兼容结果、限制与释放；后owner核祖先关系和Spec/API，接受后串行集成。公共基面漂移时刷新模块编译和受影响反例，不盲用旧绿；最后一个集成者负责已交付消费者并集的实际定向兼容。不把所有future组合强加上游，也不删除已交付能力。需要更小公共基础先行交付时必须有独立范围、最终review、冻结门禁/Spec和合入收据，不能把needs_fix整体先合入。

## 4. 823/824交叉类型拒绝与验收责任

解除D823不等于允许闭包进入Dyn/freeze。824 V-12必须递归验证已交付类型的完整图：合法List<Str>/record正控，List<Fn>或record中Fn/borrow、未知/保留类型tag、环/重复owner拒绝。823未交付时真实Closure类型不存在，用已存在Fn类别及保留/未知tag验证封闭白名单，不伪造一个“已支持Closure”或把未来闭包样本列required skip。

若823先合入，824必须运行真实嵌套Closure pack/freeze拒绝；若824先合入，823 T-04/T-05必须在引入真实Closure类别时补同一拒绝控制及824合法值正控。案例是否适用绑定当次已合入profile/类型/版本矩阵，不能用unknown-tag测试替代已经存在的Closure类别；最后集成者独立review核实际组合，责任不留给无人认领的future。823/824原borrow、资源、失败保源、旧代保活和精确增量义务不因并行减少。

826的独立codec也不获得新LoadedModule构造能力：decode结果仍未授信，不可借公开DecodedAbc填旧版本元信息把新wire伪装成旧wire。正式路由/适用版本与预算在D825后核心窗口实现；T-10编码/密度结果不计作V-01全路径或V-08 loaded体积/性能已过。

## 5. 当前可交work的边界

- 821已经交付；822继续原独占核心工作，本轮不重开它或改变其验收。823/824等待D822的只是T-02真实profile集成，T-01/T-10可用D821的已交付值面开展实际模块工作。
- 825 T-01/T-10无需等待822–824；826 T-01/T-10无需等待825私有装载表示。四者T-02正式集成仍按各自D门/窗口，不占着窗口等待。
- 这是规划修订授权，不是本轮代为启动实现/代理。下一次work核实际D门、写面接受与唯一树；每Plan仍完整最终review、merge-owned门禁和归档后才称D-N。825/826保持择机优化优先级，不因独立模块可做就挤掉功能线的核心窗口。

本轮修订823/824 r5→r6、825/826 r4→r5，新增T-10、AC-10、823/824 V-14及825/826 V-13；原状态/进度、目标/非目标、预算、原AC/V/SD承诺保留，824 V-12仅明确类型可达性与跨集成责任。原T-02首例依旧从开始纳入cache/AutoDown/P95/内存，不以T-10冒充；新检查纳入现有SD-01及正式脚本汇总。静态预检见[preflight](../attachments/ace-parallel-20261026/preflight.md)。
