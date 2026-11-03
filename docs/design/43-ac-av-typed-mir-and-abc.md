---
status: accepted-direction
describes: target-state
created_at: 2026-10-16
updated_at: 2026-10-30
implemented_by_plans: []
---

> **PLAN-858 口径更新（2026-10-27）**：本文历史 ACE P95<1s 目标在开发初期由用户调整为 P90≤2000ms；原正式样本和正确性、结构、资源约束保持，架构基本成形后以实测另定长期目标。其他 P95 数值作为历史或诊断保留，不再作为 ACE 开发期交付硬门。当前规则见 [TestCase v2 周转口径](../specs/auto-test-case/profile-v2-native.md)。

# Design 43：ACE统一架构、Typed MIR、ABC、CTE与增量发布

> **2026-10-28续接基线**：798/828、817/818和820–827已有有限合同基础；868已归档交付，共享HIR→MIR唯一实现为experimental/auto-mir-lower。870的native data/owned扩面与871五域切换/退役尚待实施；862正在补入口/Unit全链，前端部分通过不等于全链交付。Typed MIR不再整体延期v0.7；A2X仍以结构化HIR为主入口，完整AJ/发布器不塞入本批。 统一进度快照与未完工作见[ACE续接基线](strategy/ace-implementation-handoff.md)，后续按current Spec/Plan及merge收据重核；本注不扩大原Plan验收。

规划导航：[ACE总路线 §0](strategy/auto-native-backend-evolution.md)汇总共同设计、AV能力矩阵、真实程序替换路线及Plan入口，并规定后续规划的现状/依赖核验步骤。继续规划可只指定该总入口；本文负责统一技术合同。

本文整合2026-10-16用户确认的路线，取代此前“MIR整体延期v0.7、CTE可直接沿HIR→ABC复用旧AVM”的目标态安排。本次同时确认AC/AV/AJ共同服务、分层执行与增量发布方向，实施边界见§8–10。方向已确认；指令集、Schema、实现路径及首批profile仍由后续Plan冻结，本文不声明已实现；§12采纳ABC评审后修订798正确性验收，仍不改现状Spec。

关联：[HIR战略](strategy/auto-hir-design.md)、[后端战略](strategy/auto-native-backend-evolution.md)、[Design40产物/cache](40-compiler-artifacts-and-cache-lifecycle.md)、[Design41 ABI](41-interface-metadata-and-foreign-abi.md)、[Design42语义/CTE](42-semantic-metadata-inference-and-comptime.md)、[Design44 AV数据/Place/所有权](44-av-data-layout-place-and-ownership.md)、[v0.6路线](../roadmap-v0.6.md)。

## 1. 名称、实现语言与分期

2026-10-20用户命名方向：统一架构称为 **ACE（Auto Compile & Execution）架构**，涵盖增量编译、解释执行、JIT、源码转译、CTE、全局分层cache、热重载与构建发布。AC/AV/AJ是ACE中的职责组件，A2X是源码转译路径；它们共享语义/身份/依赖与产物生命周期，不要求所有后端经过相同物理IR。ACE为架构名，本次不创建新CLI/crate或重命名现有工件/协议；AAC/AAV继续表示Auto实现主体的阶段。历史文献中的“AC/AV/AJ统一架构/平台”指向ACE，文件路径保持以兼容已有引用。

| 名称 | 职责与阶段 | 不等同于 |
|---|---|---|
| ACE | Auto Compile & Execution；统一编译与执行架构，包括转译及共同增量/cache/热重载服务 | 三套独立工具的统称、或已全部交付的单一可执行程序 |
| AACE | Auto语言实现的ACE主体；2026-10-24确认延后至v0.7，在Rust版ACE全示例成熟度门后实施 | v0.6的小子集自举、或仅包装Rust引擎的CLI |
| AC / `ac` | 编译入口/驱动，负责前端、语义/CTE调度、提前编译及发布；参照实现先用Rust，当前原生后端为Cranelift | 必须随每个应用携带的完整编译器 |
| AV / `av` | 新架构Auto VM；先用Rust实现ABC解释执行，衔接模块/宿主与执行状态服务 | 旧AVM的改名或旧字节码兼容承诺 |
| AJ | Auto JIT，按需快速编译与优化编译，复用AC的编译核心和产物服务 | 独立维护另一套语义/优化器，或已发布的`aj`命令 |
| AAC | Auto实现的编译器主体；旧文档ACC指同一路线 | 包装Rust前端的CLI |
| AAV | Auto实现的新版AV，按所需能力成熟后迁移/对拍 | 现有AAVM/AA2R展示或其自举证据 |
| CTE | compile-time execution，由语义查询调度并调用AV | 一个独立于AV的临时表达式解释器 |

AV是长期产品名，不使用navm，也不在Auto迁移后改回avm。AAC/AAV表示实现阶段/主体；产品职责为AC/AV/AJ。正式CLI安装与包名兼容需要实施Plan处理，本文不改现有命令。

先建立基础AC和共享语义合同、最小Typed MIR/新ABC/Rust AV，再接语义查询→AV→冻结结果，随后按依赖扩语言/FFI与产品消费。**2026-10-24用户明确：v0.6提高Rust版ACE成熟度，以复刻现有AutoVM/A2X全部示例和M1–M4验收为目标；v0.7再实施Auto版AACE（含约定AAC/AAV主体）**。不再以编译器源码的小子集可用作为提前迁移依据；当前保留Auto可消费的公共格式与接口，不启动主体重写。完整优化JIT/OSR等不因兼容目标提高而自动成为自举前置，具体主体/宿主桥/代际范围在v0.7冻结，见[版本合同](../plans/reports/ace-milestones-version-boundary-2026-10-24.md)。

## 2. 共同管线与各层职责

```text
source/CST/AST → resolve / infer / instantiate ↔ CTE request
                         ↓                         ↓
                 final Checked HIR      checked executable HIR fragment
                      │    │                       ↓
                      │    └───────────────→ Typed MIR
                      ↓                         │    │
              A2X structured lowering           │    └→ ABC → AV
                      ↓                         ↓             ↓
               Rust/C/JS/...          Cranelift adapter    frozen CT result
                                              ↓             ↳ semantic queries
                                            native
```

- Typed/Checked HIR：保留Auto语义类型、结构化控制流、解析后的引用、泛型/ownership/effect和来源；A2X的主要入口。
- Typed MIR（简称MIR）：将执行语义显式化，供AV和native共享；默认有类型，不建立一套长期“无类型MIR”平行产品。
- ABC：AV的可执行模块及指令格式；有独立版本、验证和执行profile，不是MIR的任意内存转储。
- backend/低层IR：目标布局、物理调用约定与机器表示在适用阶段降低；未来自主后端可以再有Machine IR、指令选择和寄存器分配。

MIR是编译器内部合同，但拥有可版本化Schema/缓存表示，不代表向第三方发布永远稳定的公共ABI。它不含CLIF私有对象、宿主指针或ABC物理指令编号。A2X按目标需要复用部分语义lowering，不强迫所有源码转译先失去结构再恢复循环。

**A2X共享分层补充（2026-10-30，仅设计）**：具体合同见[Design51](51-shared-a2x-and-hir-to-rust.md)与[实施战略](strategy/shared-a2x-and-hir-to-rust-roadmap.md)。各目标共享受检语义查询、结构化准备/转换工具、依赖/cache/来源/产物服务；目标类型表示、能力适配、运行库映射、Target AST及打印分开。共享层不重建推导/ownership权威、不依赖旧AST/AV执行/native ABI；pass按目标能力组合，不强制统一低层IR或同一转换序列。首期拟三包：共享骨架＋Rust标量闭环→record/text资源→集合/变体，每包首例含AutoDown、独立oracle、函数增量和恢复/预算；无Plan号或实施授权。

已有Checked HIR→Cranelift实验管线按原profile继续工作。未来共享MIR接线应另立迁移Plan，以native/AV对拍验证后逐步替换；设计更新不使既有成果失效，也不把旧直达路径冒称已采用MIR。**2026-10-28 迁移合同已建立**：收敛路径、模块边界、布局/ABI、身份/cache 与逐 profile 切换/退役判据见 [Design49](49-native-single-mir-lowering.md)；实施链为 [868](../plans/archive/868-hir-mir-lowering-shared-crate.md)→[870](../plans/870-ac-mn-record-text-native-expansion.md)→[871](../plans/871-ac-native-switch-and-old-lowering-retirement.md)，在 871 完成前本条“旧直达路径”仍是事实。

## 3. 最小MIR：正确性先于优化

首批profile应明确块/局部值或place、语义类型/常量/声明ref、操作与terminator、调用及来源映射。必须表达条件/循环/短路的实际路径、求值顺序、精确数值/trap、资源移动与清理及失败退出。异常等未进入首批语言面的能力明确unsupported，不能用空操作伪实现。

MIR verifier检查引用及类型、块终结和控制流、定义/使用或初始化规则、调用签名、profile、资源/effect义务。具体采用可变局部值还是SSA/block arguments在有界探针决策；最小MIR不以SSA、通用pass管理器或性能优化集为前置。

v0.6实施必要的语义lowering与验证；v0.7按实测补高收益优化、SSA/低层IR与自主后端。共享MIR的必要性来自AC/AV执行语义复用，不再以Cranelift是否足够快决定是否存在MIR。

## 4. ABC模块与AV执行模型

ABC设计须与MIR一起完成首批全路径可行性探针，不能先写引擎再补模块合同。至少作出以下决策：

| 决策面 | 首批须明确的内容 |
|---|---|
| 模块与身份 | 格式/语义profile、模块接口/依赖、types/meta/constants/functions/imports/exports、来源映射；各版本域分开 |
| 指令和控制流 | 类型化操作、值/局部槽/寄存器或栈策略、分支/调用/返回、数值和trap语义；编号及编码经Plan冻结 |
| 值与内存 | 标量精确宽度、聚合和引用、动态Any显式化、分配/释放/ownership；不默认全Value装箱或全局锁堆 |
| 装载与验证 | 结构和引用界限、签名/控制流/类型、能力和资源准入；坏输入受控拒绝，不成为成功模块 |
| 执行限制 | 指令/时间/内存/调用深度预算、取消及宿主调用的中止责任；资源状态与语言错误分开 |
| 宿主与互操作 | 类型化导入、授权能力、受控句柄/资源和释放者、错误/回调及代际；native FFI另作适配 |

编译器语义类型/meta是共同事实来源；ABC携带执行所需投影/引用，而非复制整个语义库。常量采用可验证、可重建的portable图；字节码文件、装载后类型/常量表和活对象堆是不同表示。磁盘可用Atom/Batom+Schema描述或紧凑指令payload+manifest，AV内存可用紧凑结构；不要求解释循环遍历通用动态Node，也不承诺未定义Batom可直接零拷贝执行。

与旧AVM的数据/指令兼容性默认未建立，必须实测类型、精确整数、聚合、调用、资源、错误和Meta访问。允许按语义证明复用算法/组件；不因同名ABC直接执行旧字节码，不通过旧AST/字符串roundtrip承接新CTE。必要时编写显式转换器并注明支持/拒绝面；旧bug不成为新语言语义。

## 5. CTE是AV的受控执行profile

CTE依赖真实AV执行前，先有该profile的MIR→ABC→验证→执行闭环。语义服务检查请求片段及其可执行依赖，不等待全程序最终Checked；AV只执行准入后的片段。求值读取Meta通过有依赖记录的查询接口，访问类型不等于获得宿主地址或任意runtime调用权限。

CTE与runtime共享AV执行语义，但权限、限额和可用导入不同。默认禁止未追踪的文件/环境/时间/随机/网络/FFI，所需外部输入通过broker授权并计入依赖。AV同进程原生调用不自动提供内存隔离；完整不可信插件的安全保证需额外执行隔离设计。尚未选定以WASM替代自有ABC；该选型不能由会话建议直接当成产品决定。

结果须冻结为portable值/类型/生成项及依赖manifest，经消费校验返回推导/实例化。查询环带调用链诊断，取消/超限不永久缓存成程序非法。将目标布局作为显式target查询，不误用运行AV的宿主布局。

## 6. 统一产物、cache和亚秒周转

### 6.1 早期架构约束与分期实现（2026-10-20澄清）

用户再次明确：增量编译、层级化全局cache与热重载从较早阶段共同决定ACE的工程结构。三者是统一编译与执行平台的基础合同，贯穿前端、语义/CTE、MIR、ABC、AV及未来native/AJ；不能将“本Plan尚不交付完整功能”解释为“本Plan可以不考虑这些结构，等语言完整或AJ上线后再重构”。统一架构名采用ACE，组件职责见§1。

| 基础责任 | 早期即须设计并由相关实现消费 | 实现与证据边界 |
|---|---|---|
| 增量查询与失效 | 稳定逻辑声明/实例身份、不可变源码snapshot、内容修订、实际接口/body/layout/配置/负依赖、来源sidecar；共同查询服务对接现有store | 按已实现producer证明命中/重算与fresh等价；源文本、推导、MIR和代码生成的增量粒度分别报告，不能以单层命中宣称全管线精细增量 |
| 层级化全局cache | Design40的阶段目录、query与content身份分离、版本化Schema/recipe、required-data与失效边分离、跨工程上下文隔离、分页及有界热点/暂存 | 新producer接统一生命周期；未实现层显式future，不用空payload补全目录；相同内容共享不合并不同声明身份，也不共享工程权限 |
| 解释器在线热重载 | 逻辑调用入口与具体代码修订分离，完整候选依赖快照、安装准入/提交点、调用链的版本可见规则、旧代frame/env/object/callback的lease与回收 | 早期最小AV profile就可单独验证，不依赖JIT或完整语言；缓存恢复/下一次重新运行/只验证generation标签不能当在线替换通过 |
| 数据与跨层寿命 | 类型/布局版本绑定、Place/loan来源、旧代活对象/闭包与清理代码保活；adapter和不兼容变化的拒绝/显式迁移边界 | 旧值不按新布局重解释；状态迁移、活帧迁移、OSR及不可信插件分别冻结，不因最小重载默认承诺 |
| 并发与发布一致性 | 编译请求绑定snapshot；过期/取消结果不可安装覆盖新版本；候选闭包验证成功后按已冻结可见规则发布，失败保留可用旧版 | 发布事务不等于回滚已经发生的guest IO；同一调用链是否留在旧快照必须在实现前明确，不默认新旧callee任意混用 |
| 性能与资源 | 端到端trace、cold/warm/edit/reload分场景、各层真实命中计数，root/lease/GC及旧代数量/总峰值预算 | 亚秒目标按代表profile冻结；重编快、装载快、整次热更新可用三个时间分别计量，不能只测emit或靠无限保留旧版达标 |

首次扩展相关模块时就必须给出上述身份、依赖、接口与生命周期的适用设计。计划应逐项注明“已实现且验证 / 本计划实现 / 有明确owner的设计合同 / 当前明确拒绝”，不得把架构义务全部写成无人负责的future；每一阶段也不必同时实现全部生产能力。

早期验证应按最小程序切片推进：同进程增量编辑与错误诊断、跨进程/工程的分层复用与失效、持续存活AV中的候选安装/旧调用保活/新调用选择/失败与迟到结果拒绝/旧代释放。精确支持矩阵、原子提交和调用可见策略、预算与oracle须由独立有界Plan先冻结，不能用概念演示代替执行证据。源码/数据扩面每新增一种长期存活值，都重核其代际责任。

798当前“实际热重载非目标”保持，只表示其有限交付范围；820–827现有任务/验收不因本节自动扩张。后续规划须较早安排基础合同审计及最小AV热重载验证，不能统一拖到G6 UI/服务迁移或AJ专项。未交付前持续标记缺口，不宣称本批已有全部热重载/全阶段cache能力。

### 6.2 产物与执行计量

具体层级、缓存粒度与格式基线以[Design40 §3.2–3.6](40-compiler-artifacts-and-cache-lifecycle.md#32-首批层级与粒度决策基线2026-10-20)为准：查询/重算、逻辑持久产物、物理页与验证/装载分别定义，HIR/MIR/ABC以声明/函数复用和模块manifest装配为目标。现有HIR模块Bundle等粗粒度实现不能冒称已经完成该迁移。

沿用Design40目录，不新增独立MIR数据库或VM私有cache：A07是公共HIR，A08承载已实现的MIR/lowering/分析结果，A09承载ABC及后端产物，A05保存CTE recipe/ABC身份、冻结结果和依赖，A06保留共同语义事实，A10是模块/发布manifest，A12管理装载代际lease。类别不意味着必须为每步写独立磁盘文件或重复存一份payload；按ArtifactId引用。

查询key包含实际输入、类型/meta/常量及helper body依赖、负依赖、语义profile、Schema/lowering/ABC/AV版本、target/配置、授权及受控外部输入等适用因素；内容身份与查询身份分开。不存执行地址、活帧、解释器堆或本地intern ID。持久结果恢复需要重新建立当前消费门，不反序列化trusted凭证。

热点、索引和暂存有预算；按函数/模块及强引用分页读取，共享store受root/lease/自动GC治理。可移植ABC与target相关原生cache分开身份。冻结结果跨进程/工程复用，不靠常驻全部依赖AST/Meta/堆。

首个MIR/ABC/AV用例即有AutoDown用例、独立oracle、fresh/命中/重启/失效/损坏负例和真实阶段span。CTE命中应观察到av_execute=0，miss正控必须真实执行；沿用既有vm_execute字段时须显式标识新AV引擎，不能把796模型计数当执行计数。普通代表编辑负载端到端P95<1s为共同目标，报告各阶段、嵌套/等待、内存与I/O；不是每阶段各自小于1秒，也不是全仓冷构建承诺。预算/负载先由Plan冻结，未测不冒称达标。

### 6.3 周转目标与分阶段扩展策略（2026-10-20 用户澄清）

保留 **P95<1s**，不改为算术平均。开发体验目标是在工程完成初始化、已得到首次结果之后，连续修改不同位置/类型的代码，至当前修订的新结果或准确诊断可见；首次启动和初始工程装载另报。编辑实际引起的依赖更新、编译、验证、安装、执行和呈现均属于该次周转，初始化不能按每次修改重新划分。持续存活会话、跨进程恢复及全冷构建分别报告，某一路径的结果不冒领另一条路径能力。

按用户认可的两种办法推进：**当前已实现流程先用真实入口/冻结负载达标；未实现工作的成本预估辅助安排次序；新功能接入后重新实测**。预估列输入规模、重算/分配/IO工作、参考来源、估计区间、串并行关系和未知项，不编造毫秒数，不把独立阶段的P95直接相加当整体P95，也不以估计替代验收。当前跨进程开销不能直接抵扣未来热安装/保活成本。新能力超标时据trace定位和评估优化；确需修改目标则显式修订相关Plan并由用户决定，不自动放宽或删除失败样本。

框架弹性由§6.1的公共接缝承担：逻辑身份与内容/布局修订分离，编译/验证装载/候选安装/执行分层，真实依赖失效，snapshot与提交点绑定，generation/lease/owner保活与有界回收。新模块的公共合同不得把“一次worker启动等于一次开发会话”或进程地址/局部槽号固化为唯一集成方式；相关owner在扩展时核适用接缝及状态，发现阻碍就作有界调整，避免留到完整语言或UI之后大改框架。

上述要求是早期结构约束，不要求一次交付完整在线切换、对象状态迁移、活帧迁移或OSR。798现有两条正式P95门、负载/预算及实际热重载非目标保持；本节不追加常驻会话实施phase，不用平均值或未来估算解锁当前失败门。后续最小AV热重载及数据/CTE扩面按其Plan冻结新入口/支持矩阵、实测更新后结果与生命周期。当前程序回收/重新运行或有限代际模型继续按其真实范围登记。

澄清的来源、现有实现边界及责任表见[798周转决策记录](../plans/attachments/798/turnaround-policy-20261020/decision.md)；该记录不是代码或未来能力的验收pass。

### 6.4 真正函数级增量的全链路合同（2026-10-20）

用户进一步明确：签名和实际依赖事实不变的函数体编辑，必须只重新编译并替换该函数，其他函数的中间表示和已加载代码保持不动。该目标约束**整个源码→语义/HIR→MIR→ABC→验证装载/安装→AV结果链路**。只缓存函数字节码、整模块重算后拆分、每次重新装载全部命中函数，即使P95<1s，也不构成真正函数级增量。§6.3的分阶段推进不豁免这些结构性要求。

798当前只实现有限执行/信任合同和函数持久产物复用：前端整模块检查与Atom往返，缺一个单元仍整模块MIR lower/verify/split，命中单元重新decode/verify后组装完整LoadedModule。874.6ms是该流程的本轮warm P95，不是单函数成本；sig 51.1ms是前端错误早退。已读源码及证据见[核查报告](../plans/attachments/798/function-incremental-audit-20261020/report.md)。

最小闭环由[828](../plans/archive/828-av-function-incremental-session.md)完整承接，D798实际交付后实施，不反向阻塞798；798 r6须在coverage/拟Spec准确说明边界。828不得再把任一中间层的增量推给无人负责的后续计划。现有语法787、共享身份796和store783优先复用；当前CheckedModule/LoadedModule的整模块信任边界需要安全的函数单元/上下文凭证，不能改为信任cache或翻unit标记。825/826在828后按原依赖先后交付，按实际基面消费与串行写入，保持已交付的函数复用/安装接口，不形成反向依赖。

| 结构 | 最小必须交付的行为 |
|---|---|
| 常驻项目/查询 | 保留source/CST、接口/签名环境、函数Checked HIR/MIR和代码；变更集驱动局部工作，cold/restart恢复与warm复用分开 |
| 函数语义与lower | 局部重扫/声明解析；仅变更函数resolve/typecheck/HIR验证及MIR lower/verify/ABC；未改函数不再投影、重算指纹、lower或从store读回 |
| 稳定执行单元 | 函数局部表/常量/PC、稳定FunctionId/调用绑定，改变一个函数的长度/槽数不重编号或重新装载其他代码 |
| 受检增量安装 | 候选函数在有效上下文验证和绑定后提交程序根；未变函数凭证/对象/代际复用，接口/授权/真实依赖变化按正确范围失效 |
| 版本与寿命 | 新顶层调用选新snapshot，在途整条调用链留旧snapshot；失败/取消/迟到不覆盖当前版，旧引用退出后有界回收 |
| 结构与时间双门 | 未改生产阶段调用数、读盘/decode/verify/load为0；HIR/MIR/代码identity与代际不变，安装集合恰为变更函数；实际编辑到当前结果P95<1s |

“不动”指函数语义/IR/已加载代码payload和身份复用，允许必要的编辑定位、词法边界、源位置目录、依赖摘要/版本根元数据更新，须单列实际计量。若caller只读取callee签名，callee body变化不重编caller；若确有const/CTE/inline/layout/body事实依赖，须记录并按事实失效，不能用“签名没变”漏依赖。普通程序为新结果需要调用哪些函数仍按语言语义执行，不隐含缓存运行值、跳过副作用或响应式值级增量求值。

首版是现有scalar有限面和单文件源码的端到端切片，不承诺对象/活帧迁移、任意签名原位重载或全产品UI。签名/配置改变、跨函数编辑、词法不收敛的扩大失效/拒绝/回退明确报告；正常受支持body编辑静默全量回退则结构门失败。正式持续会话在首个正确结果后计时，必要IPC/排队/验证/安装/执行/结果发布仍在编辑窗口内，不能预编变更函数或只测单阶段。完整目标在828实际验收前保持未交付。

### 6.5 执行优先级与后继增量兼容（2026-10-20用户裁定）

2026-10-26依赖审计细化现有优先级：正式数据集成链为**820→821→822→{823，824}**，824不消费闭包，不再等D823。823/824的T-10可在D821/D828上做私有同源模块；825/826的T-10可基于D798/D828分别做PC表/codec。823/824 T-02仍等D822，826 T-02正式装载/路由/执行仍等D825。817/818/827独立consumer保持并行，功能/源码核心窗口仍优先，优化择机；各Plan原P95/内存/AutoDown/精确增量不变。依据与可编译接缝、写面、适用拒绝责任见[823–826实际审计](../plans/reports/ace-parallel-dependency-audit-2026-10-26.md)。

D828指当前revision独立最终review、冻结门禁、实际合入、Spec/merge收据齐全，T-01或局部review通过不解锁后继。D828前仅只读准备，已有817/818树与818首例提交保留；818在D828后刷新并重验，不再保留是否绕过该门的待裁定选择。798和828都不反向等待后继。该排期细化2026-10-20顺序，取代825/826优先于828或默认优先于820功能线的候选排序。

单写按实际公共source/HIR/语义、MIR/InstructionSpec、ABC/verifier、loader/dispatch/snapshot及测试注册界定，默认共享核心一个窗口；独立consumer不占核心窗。前owner给出冻结/合入提交、版本及消费者兼容结果、限制与释放记录，后owner先核已交付基面/Spec和API并接受交接。无界面隔离证据不并发改公共合同，不持窗等上游，不从WIP抄接口；拆分按实际owner交接而非等待整批。细则与各Plan工作/复审/merge责任见共同合同。

独立源文件closures、dynamic/portable、loaded_program、compact_abc可由各Plan独立harness编译同一生产源；不注册公共枚举/指令/模块，不复制IR/VM或构造私有信任凭证。review/repair期间其他Plan继续T-10，不以冻结代替上游交付。新模块最终仍经原T-02完整首例及正式准入，模块绿不等于整Plan绿；最后集成者负责已交付消费者并集，core/注册接线只在D门和窗口接受后进行。

所有后继须保留§6.4全链路机制，在各自Plan §§5–8配置work-owned AC/V/SD/任务及真实反控；最终review以其完成为前置。共同验收包括D828完整source编辑基线、各自新增能力的两函数局部更新、依赖事实变化的精确失效和强制whole重算/重装载必红。新类型仅有HIR/MIR入口时诚实标其覆盖起点，现有scalar完整链不能退化。函数identity/IR/代码代际和实际生产工作量是结构门，不靠hash相同/cache hit/更快的全模块处理代替。

具体各Plan责任、四必需子case、例外和依赖定义见[执行顺序与兼容合同](../plans/reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)。828先发布函数查询/变更集/受检代码/snapshot公共消费接缝，817/818及数据、装载、格式和source入口扩展消费它；不另建平行parser/查询图/loader。新语义不得绕过当前上下文授信、旧代保活和失败/迟到提交门，真实不兼容变化按受支持策略失效或拒绝，不擅自重解释旧对象。

## 7. 在途796/797与下一步

2026-10-16调查基面62feb29e5be4f531b6e29ea90bb53fc65cd2d2a3；主检出796为executing r1、797为drafting r1（若他方已启动，以最新实施收据为准）。两者当前合同均明确排除VM/MIR实现，本次不改变其revision、任务、验收或已记录进展。

- 796继续拥有共同语义身份、类型/meta描述、CTRequest/CTResult及依赖/缓存合同。它的CT producer是模型，不授信真实AV。后续MIR/ABC引用796输出；AV执行布局另作投影，不反向要求796固化宿主对象布局。若实际已冻结Schema无法容纳所需CT合同，由证据触发显式revision，不在本次暗改。
- 797继续有限Windows x64原生AutoABI及C双向探针。AV内部调用/ABC版本不是AutoABI；AV↔native adapter后续消费其语义和原生合同，不在797追加解释器、字节码loader、沙箱或JIT。共享逻辑签名不意味着相同物理布局。
- 787/788及拆分路线保持各自边界；新实验模块不重写其在途热点。正式AutoDown接入遵守788及796的实际合入依赖，不复制共享协议或把未执行case算绿。

下一步优先规划“Typed MIR与ABC/AV执行合同及首个全路径探针”：在有限i32/bool、调用、控制流和trap范围，交付HIR→MIR→ABC→Rust AV，并与已有native路径做真实行为对拍。至少证明短路未走分支不trap、边界数值/调用语义一致、坏MIR/ABC拒绝、cache正确且资源有界。合同需列全资源/Meta/模块类别与unsupported面，但首例不伪称支持所有类型/CTE。

可先基于已合入HIR做独立研究与探针；消费796/797真实输出、共享协议与最终集成须绑定其实际合入revision，不等完整AC/AAC，也不修改他方工作树。该工作包若过大，先交共同设计/可行性，再拆最小MIR和ABC/AV两个单向依赖Plan，避免互相等待。

该首个工作包还须建立逻辑编译单元/ref、profile化产物身份、可重定位/再生边界和代码代际合同；验证修改一个body、依赖和profile时的正确命中/失效。不要求实现热点采样、机器码在线替换、优化JIT、OSR/deopt或完整发布器；先以现有native/reference路径验证可复用产物，AJ执行仍为future，不写空成功adapter。

之后再规划“AV驱动的CTE查询与冻结缓存闭环”，以及native公共MIR接线/有限增量发布、语言扩面；完整分层AJ另立计划，不挡住解释执行和CTE。完成约定能力后规划AAC/AAV迁移。这里是规划顺序，没有分配新Plan编号、启动实施或把AAV全量实现自动列为v0.6硬门槛。原native-only的计划数/日历估算不覆盖本次新增MIR/AV工作，须按探针结果重新估算，不能承诺原工期不变。

## 8. AC/AV/AJ统一平台与分层执行

用户进一步确认的长期方向：开发期间解释启动、热重载修改、热点快速JIT/优化JIT；发布复用有效中间产物并补编，而非默认从全部源码冷编。统一的是共同语义、查询/产物服务、ABI/模块边界和调度，不抹去解释帧、可执行内存、平台机器码的差异。AC是工具链入口，应用runtime按发布profile裁剪，不要求把整个前端/编译器带到部署端。

```text
                  AC：语义/CTE、构建与发布驱动
                              ↓
                   Checked HIR → Typed MIR
                      ↓                  ↓
                 ABC / AV解释       共同优化/codegen服务
                      ↓             ↑                  ↑
                热点/预算调度 → AJ快速/优化编译      发布提前编译
                                      ↓                  ↓
                              可重定位原生产物 → 链接/包发布
                 所有产物/依赖 → 统一store和query graph
```

AJ从经过验证的MIR（或未来明确定义的ABC到后端适配）编译，不强制把ABC逆向还原为HIR/MIR。部署提前编译和在线JIT调用同一编译核心，区别在触发时机、延迟/优化预算、装载和允许能力；发布不需要执行程序才能生成冷代码。

| 执行层 | 策略 | 必需边界 |
|---|---|---|
| T0 AV解释 | 低启动成本、快速修改反馈；有效原生cache已命中时可直接进入合适版本，无需每次重走解释/热身 | ABC验证、资源计量、可取消、语义/权限profile |
| T1 AJ快速编译 | 热点或明确请求触发，优先编译延迟，异步/受预算调度 | 不阻塞交互、快照身份、安装前复核代际；迟到结果不覆盖新代码 |
| T2 AJ优化编译 | 预期收益覆盖编译成本时启用；是否使用运行画像/推测分别声明 | 依赖/假设、检查/回退和失效规则；不保证每个单元都升级 |

三层共享语言语义和逻辑签名，内部帧/对象/物理调用可不同，但跨层必须有明确adapter及资源/错误传播合同。首版允许只在函数入口选择版本；执行中旧帧保留旧代至完成。模块热重载仍遵守批准的事务/一致性和lease协议，不能逐槽替换破坏相关模块快照。OSR（执行中换层）、去优化及任意活帧迁移不是首版前提。

逻辑增量查询单元、机器码编译单元和运行时装载单元分别定义：声明/函数body/实例/递归组可作为逻辑粒度，代码生成可分组或跨单元内联，MIR基本块不自动成为最小持久编译单元。内联或读到callee body/layout/候选集合就记录依赖；被内联函数修改会使相关原生版本失效，而非只换一个callee入口。

各tier的代码、元信息/索引、画像、编译队列和旧代在途代码共同计入相应内存/磁盘预算；限制每单元版本保留及后台编译并发，资源不足时退回已有合法层或受控延期，不为热点升级无限保留机器码。活帧/回调所需旧代不得被普通cache淘汰，采用lease和背压。

推测优化须保留可验证假设与guards/依赖；依赖运行时状态的专门化版本只有在其准入条件仍成立时才能安装/复用。不保存绝对地址、活反馈对象或进程内dispatch槽到portable cache。首版可以全部采用静态类型可证明变换，不为支持T2而提前承诺复杂推测/去优化。

CTE是独立权限/预算profile，可先只用T0，未来允许AJ时也必须维持确定性、计步/取消及宿主授权，不能因升级原生而绕过限制。开发采集的画像是可选、有预算且版本化的优化输入，不成为正确构建所必需的隐藏条件。

### 8.1 函数级首版与block级扩展方向（2026-10-20）

用户认可首版以声明/函数级实现，并要求预留后续显式block级增量与热重载，再扩展编译器自动语义分块。大函数内局部修改应有机会复用未变区域，而不永久要求整函数重做全部分析/降级。精确缓存/身份/边界合同见[Design40 §3.7](40-compiler-artifacts-and-cache-lifecycle.md#37-函数内区域与block级扩展空间2026-10-20)。这是演进方向，不增加798、820–827现有实现/验收，也不将首版视为已经交付在线热重载。

三个行为必须分别声明：区域编译结果复用、新函数/区域版本在后续调用/合法入口生效、正在区域中执行的活帧切到新代码。后续显式block阶段优先前两项；在运行函数的后续block入口选择新版，仍须先冻结安全点及边界状态兼容合同。任意语句处跳转、任意活帧迁移与OSR/deopt继续独立，不能因称为block就自动获得安全保证。

首版函数体可以作为唯一根区域，代码消费通过owner/profile/产物与版本接口，不让整模块序号或宿主帧布局成为未来区域的公共ABI。预留的是可分解的身份/依赖/manifest与验证接口，不是要求现在加无语义的占位opcode、持久空工件或所有block间接跳转。区域引入后仍按完整候选函数/依赖快照验证兼容并一致安装，优化不得把版本可见规则变成任意新旧block拼接。

A2X复用结构化HIR及来源/依赖，可将区域复用结果装配成合法目标源码；目标C/Rust等的编译/链接及运行时重载能力另按该后端验收。ACE包含转译不表示每个转译后端自动支持区域热替换，也不为区域cache将全部A2X强制改经低层ABC。

## 9. 增量发布：完整配置、闭包与有效产物复用

默认策略是“按发布配置解析并验证完整部署闭包，复用匹配产物，仅补编缺失/失效部分”。release是一套显式优化/运行时/保留策略，是否命中cache是另一维度；release不等于clean重编，cache hit也不等于产物满足release。

1. 固定source/overlay、依赖锁定、CTE受控输入、target/CPU、工具链及发布profile，形成不可变发布快照。未保存源码如何处理须显式，不从编辑中的多代结果随手拼包。
2. 从入口、导出、初始化、反射保留、泛型实例、插件/外部服务等根计算所需代码/数据闭包，包括从未运行的分支和冷函数。不按开发期覆盖率删代码；坏代码/缺实例必须在对应发布合同下诊断。
3. 对每个所需产物校验完整query key和依赖。开发期已有发布兼容的优化版本可直接复用；T1、调试/插桩、热重载间接调用、不同target或优化上下文的机器码不匹配时重新生成该层，尽量复用其语义/MIR/CTE结果。
4. 推测版本仅在发布环境证明假设、保留完整guards/合法回退，或重新生成后才能采用。禁解释/JIT的纯native发布不能依赖回退到AV才正确的版本；缺少闭包的动态生成能力须保留相应引擎或明确拒绝该发布profile。
5. 处理符号、重定位、调用/布局ABI、初始化、异常/栈展开信息（适用profile）、Meta保留/剥离及runtime服务，完成链接/验证/打包后原子发布。优先复用平台linker，自主驱动/链接图不等于v0.6重写PE/ELF链接器。

开发进程的JIT内存不是可发布对象文件。可复用机器码必须伴随可重定位的代码/数据、符号、ABI/runtime需求、调试或展开信息及生成recipe；不满足时从有效MIR等上游重生成。首次release或改变发布profile可能使多数原生单元重编，不承诺每次发布只做常数时间“缝合”；跨模块优化/链接仍可能读取摘要并重做受影响单元，允许最终链接非增量。

| 产物 | 内容与执行策略 |
|---|---|
| target原生发布 | 特定target机器码+必要runtime；闭包完整且无动态生成需求时可裁剪解释器/JIT，仍可能需要分配/模块/宿主服务 |
| 可移植发布包 | ABC+版本化接口/依赖/Meta及所需资源，可选附带各target兼容原生切片；不把机器码称为跨平台ABC |

发布manifest及交付副本拥有独立持久生命周期，不能依赖开发cache暂存或被cache GC删掉；部署不必携带源码/完整MIR。cache丢失有再生输入则重编，无输入则报缺失，不以热身运行“补出”无法重建的代码。

验证要求：相同快照/profile下，增量发布与无cache构建的可观察语义、诊断和闭包一致；字节一致需另定可复现合同。用冷函数从未执行、只改callee/body/layout、切target/profile、JIT地址重定位、陈旧假设、cache损坏/回收、发布后删除cache等用例区分真复用与漏编。无cache构建保留为验证路径，不作为每次发布的强制流程。记录命中/重编单元、链接/闭包成本、产物/代码内存和发布耗时；代表编辑P95<1s与发布/冷构建指标分别报告。

## 10. 现在设计、首批实现与后续实现

| 能力 | 现在必须设计 | v0.6/紧邻工作包的实施边界 | 可以后续实现 |
|---|---|---|---|
| 共同编译服务 | AC/AV/AJ职责、MIR/ABC输入和语义/profile、错误/取消 | 最小MIR→ABC→Rust AV，native按单独迁移接共享服务 | 完整AJ调度与所有后端扩面 |
| 单元/缓存 | 逻辑ID与内容版本分离、依赖/假设、产物层和版本键、预算/GC | 首例真实store、fresh/hit/restart/失效及真实计数 | 智能编译收益预测/预热/画像PGO |
| 跨层调用/代际 | 逻辑签名、物理adapter、代码依赖/lease、快照安装准入 | 先验证合同和有限模块拒绝/代际模型；已有native热重载原目标继续独立实施 | 全面解释↔JIT切换、OSR/deopt、活帧迁移 |
| 机器码持久化 | 符号/重定位/target/ABI/runtime及再生recipe | 首探针用现有对象产物验证身份/复用；native支持面内逐步交付 | 任意在线JIT代码的跨进程恢复/安装 |
| 增量发布 | 快照/根/完整闭包、profile匹配、两类包、原子发布 | v0.6在已承诺native profile上落实复用/补编与真实链接；正式发布器另立有限计划，不塞入首个VM探针 | 全平台可移植包、复杂全局优化与原生切片分发 |
| T1/T2执行 | 升级、失效/回退、权限与资源边界 | 本轮不新增完整分层JIT发行门，不阻塞CTE或796/797 | 热点快速JIT、优化JIT及收益驱动调度按专项Plan实现 |

**2026-10-28**：本表中“native 按单独迁移接共享服务”的迁移合同已落在 [Design49](49-native-single-mir-lowering.md)（§3 模块边界 / §4 布局 ABI / §5 身份 cache / §6 逐 profile 切换与退役）；该迁移不改变本表其余能力边界。

方向确认不等于上述全部已冻结或实现；v0.6新增发布接线也须重估容量。初始架构不能持久化进程指针、混用dev/release键或只记录直接callee而丢内联依赖，以免后续AJ/发布返工。AJ并不要求立即切换自主codegen，可先复用Cranelift；自制后端仍按性能和维护证据分期。

参考已有技术验证方向：[ThinLTO增量优化与cache](https://clang.llvm.org/docs/ThinLTO.html)、[Wasmtime预编译与裁剪编译器](https://docs.wasmtime.dev/examples-pre-compiling-wasm.html)。这些是架构参考，不是Auto已具备增量链接、分层JIT或可移植发布的证据。

## 11. AV数据架构扩展（2026-10-20）

用户认可静态数据类型化、动态数据独立Variant、首版不使用NaN-boxing，正式合同见[Design44](44-av-data-layout-place-and-ownership.md)：语义类型/执行布局/存储分层、原位Place、初始化/访问权限、move/borrow/replace/drop、句柄代际、全出口清理及portable冻结。布局是语义事实的下游投影，AV内部表示不等于797 native ABI；安全参考实现可保留enum，不为8B槽提前引入unsafe union。

当前正式集成依赖：[820布局](../plans/archive/820-av-typed-data-layout.md)→[821 owned Place](../plans/archive/821-av-place-owned-access.md)→[822集合/Variant](../plans/archive/822-av-typed-collections-variants.md)，其后[823闭包](../plans/archive/823-av-owned-closure-captures.md)与[824 Dyn/冻结](../plans/archive/824-av-dynamic-values-freeze.md)从D822分别接入。821已交付的静态类型/借用边界不等于运行期旧帧/跨store/generation守卫；823新环境访问、824新typed host各自设计与实证。新私有模块T-10可先并行，但不碰在途822共享核心或复制WIP。复合CTE/native、完整算术/共享图/async/产品接线另按Plan认领，不由本批冒领。


## 12. ABC评审采纳与实施责任

正式采纳[ABC评审](../plans/reports/798-abc-design-review-2026-10-20.md)的A01..11；目标设计、实现复现和独立验收分别留证，本轮只修订文档。

### 12.1 表示与正确性

采用 **Typed MIR → portable ABC → 私有LoadedProgram**。MIR拥有共同语义，ABC是版本化可移植格式，LoadedProgram是受检后适合解释器的派生执行表示，可预解析操作、块起点、常量/调用表和FrameLayout；它不是第二套语义IR，不作为portable产物。完整准入链才能构造可执行句柄；改写、恢复或换代不得沿用trusted标记。

798允许现有CheckedMir背书的安全解释先交付。825实现标量私有执行层；820–824没有825硬前置，可先经同一私有接口安全适配已有执行路径。若825已合入，则明确接入其表示或保留具名受检参考路径；未知profile不得进入无验证fallback。

unit/完整module类别、entry签名、imports闭包和执行资格属于语义。单函数不等于unit；wire或版本化可信信封无歧义保存类别，消费端复核一致性。798 r4必须复现并收口，改变编码/producer语义则同步版本/cache失效；好hash或本地生成不替代准入，未链接unit不得执行。该项与R10前门误拒均为798正确性，不转入性能计划。

### 12.2 类型、指令与所有权

820统一所有指令的类型相等性：primitive按规范种类或先intern；Fn按参数/返回类型、view/mut/move模式与效果；名义record/enum按owner、声明身份和revision。局部表索引、布局/宽度/名字相同不代表类型相等。重复primitive统一选择归一化接受或明确拒绝，不允许Const按种类接受而Copy/Call因索引不同拒绝。

820建立单一InstructionSpec合同表，列operand域/角色、类型、read/def/kill、借用/owner、控制流、trap/cleanup、逻辑fuel/资源、来源和profile；codec/verifier/loader/executor逐项覆盖。首版表加覆盖检查即可，不要求DSL。821–824扩展，825/826只消费实际已交付的合同，不另定语义；若先于820实施，沿D798既有指令合同核对覆盖，不能依赖尚不存在的820表。820后来接手时合并为唯一InstructionSpec。

821首次引入owned值时，把只增加初始化def的分析扩成move-kill、借用权限及全出口清理，不能直接推广Rust Copy的Value。参数模式/返回owner属于签名。实际使用的根/投影MovePath及前缀intern成有界树，不展开所有字段的笛卡尔积。owner、活分支与Drop信息不是debug；首批根move/部分move拒绝边界保持。

### 12.3 性能、格式与实施归属

825先消除每步对前置块长度求和：预计算block_base或平坦PC，执行定位O(1)，诊断不在每步重扫MIR。CALL参数临时Vec、帧分配、全槽初始化分别测量，再选择安全参数映射/帧复用；不读取宿主未初始化值，不复用仍有owner/borrow的帧。预解码/融合/复用保持逻辑fuel、取消、检查和顺序；派生表及暂存计入预算。

826比较ULEB与窄操作数+Wide后选择有界方案，采用新wire版本；磁盘与loaded体积分开，不强迫每步重解码。模块有section、函数body长度/目录及总预算，明确必需语义/可选debug和未知项政策。798驱动已有spanless code+sidecar，改进针对完整格式分区，不误称从零补debug分离；不强制懒验证。短常量、常量池选择、slot复用、调用窗口/融合按测量选择，不成为798/820功能前门。

| 建议 | owner与交付 | 798边界 |
|---|---|---|
| A01 类别往返 | 798 r4：kind/entry/imports、版本/cache、V-14；保留R10前门修复 | 必须收口 |
| A02 类型相等 | 820冻结，821–824继承扩展 | 不新增798 gate |
| A03 执行表示 | 本节固定三层；825实现，820等保留适配缝 | 安全参考解释可交付 |
| A04/A05 PC/调用 | 825 O(1)PC、安全帧策略 | 后续性能 |
| A06/A07 密度/分区 | 826新wire、section、大小/装载成本 | 后续性能 |
| A08 Place/owner | 821主责，820基础，822–824扩展 | 后续profile正确性 |
| A09 预算/授权 | 各profile随操作交付，825/826继承 | 原798预算仍必需 |
| A10 指令合同 | 820建表、821–824扩展 | 不要求798先造框架 |
| A11 公平对标 | 825执行、826体积 | 原编辑P95合同不变 |

功能正式集成链为D798/D828→820→821→822→{823，824}，优化正式集成链为D798/D828→825→826。两条链的T-10私有模块开发可重叠，公共核心仍单写；825/826集成按实测择机，不抢功能/源码前门，不新增反向依赖或提前承诺owned优化。详见§6.5两阶段合同。

### 12.4 WASM/V8比较口径

以[Wasm值编码](https://webassembly.github.io/spec/core/binary/values.html)、[模块分区](https://webassembly.github.io/spec/core/binary/modules.html)、[验证算法](https://webassembly.github.io/spec/core/appendix/algorithm.html)对照密度、授信与预算。Wasm允许部分非最短LEB；AV是否要求最短编码自行冻结。栈机与typed-slot分别计数，不凭指令数量判断速度。

[V8 Ignition](https://v8.dev/docs/ignition)用于JS解释器/操作数密度对照；V8 Wasm编译执行另列baseline/优化tier、编译和热身，[Wasm优化/deopt](https://v8.dev/blog/wasm-speculative-optimizations)固定实际版本/flags。AV溢出trap、Wasm整数wrap、JS Number不同，使用不溢出公共子集或显式检查的等价程序，除余边界/副作用另验。

报告准备/编译、解码验证/装载、冷启动、热执行、调用/分配、编辑端到端与峰值；体积分code/常量/类型签名/控制表/debug/封装、raw/压缩/loaded。固定输入结果、guest逻辑工作、版本/tier/flags、机器负载、样本与离散度，不凭小程序承诺全面胜过WASM/V8。

### 12.5 文档复审修订：职责、版本与无环启动

2026-10-20本次[文档复审](../plans/reports/av-architecture-contract-review-2026-10-20.md)从当前合同重建责任，修订798 r4、820–824 r3、825/826 r2。仅为文档验收，不授任何实现最终pass。

| Plan | 硬前置（消费构建/实现） | 本轮必须由谁收口 |
|---|---|---|
| 798 | 现有783，后段D796/D788；797仅实际已交付时做兼容 | kind/entry/imports及cache、原R10修复候选独立验收、原资源/周转全部门 |
| 828 | D798；当前优先 | 源码到AV安装的全链路函数增量、受检上下文/驻留代码/事务寿命 |
| 820 | D798及D828排期门 | 类型等价、派生Copy/drop/layout、指令表与纯值Place |
| 821 | D820 | init/owner/loan合流、全出口清理、借用返回与事务 |
| 822 | D821 | Index/guard失效、集合/Variant提交与清理 |
| 823 | T-10：D821/D828；T-02：D822/D828 | 同源owned环境先行；真实capture/间接调用集成，824已交付时补Closure拒绝并集 |
| 824 | T-10：D821/D828；T-02：D822/D828；不等D823 | 递归白名单/树恢复/权限；真实Closure拒绝由最后集成者负责 |
| 825 | T-10/T-02均D798/D828，实际派发另需核心窗 | 私有PC表先行；真实装载/三入口派发与基准不由表构建代替 |
| 826 | T-10：D798/D828；T-02：D825/D828 | 独立wire codec先行；正式版本路由/准入/loaded执行仍等D825 |

817/818消费D798的标量语义及D828已交付函数查询/代码会话接缝；它们不是820–826或827的硬前置。814–816仅就实际迁移的接口/注册owner交接，不新增等待整批结束的门。代码窗口是互斥资源而非反向依赖：先满足硬前置、同步已交付基面，再取得窗口；持有窗口时不能等待另一owner/未合入上游，必须释放并重新排期。2026-10-21优先推进817/818与827独立消费层，同时向820功能链交接核心窗口；可按证据在D820后插入825→826再继续821，或稍后插入，不强制每批都先做优化。798/828不等待本批任何后继。

兼容基线是启动时已合入的(profile, wire/schema, semantic-version, producer)矩阵，D798旧语义不被放宽；826只按显式版本路由及旧cache策略改变wire支持。后到的820–824保留该矩阵，不再要求把826已批准迁移的字节当原版本读取。仍有已交付消费者需要的旧decoder/路由必须保留或完成最小显式迁移并重放其原支持面，不能以“新格式”删除消费者能力。

正确性验收按阶段计数：未通过准入的工件dispatch=0；已准入程序的越界、失败take、取消和失效token可先执行合法前缀，应断言危险访问/提交或目标子调用为0、先前效果及cleanup正确，不要求整个执行为0。只改kind而仍满足完整module规则的字节不自动是坏输入；需要可信expected-kind不一致、未闭imports或错entry等区分反例。

预算继续按现有合同核对，32MiB受管与128MiB进程增量不能互相替代、相减前端或只取端点。798历史约211MiB前端峰值和最新总峰值是否已满足必须由T-01/05给同口径证据；登记归因不等于上限豁免。未收口资源门不得声明D798成立。PC索引构建O(B+I)不代表整个CFG验证/装载线性，后者独立计步/测峰值。

每个Plan维护稳定case→子case→Scope→真实入口→阶段/码→独立oracle清单；acceptance取必需集合并校验无遗漏、无重复冒计、无skip。仅支持本次profile；未交付owned/async/token类别不成为标量性能Plan的假验收。

## 13. 源码能力与旧AVM替换路线（2026-10-20）

本设计的执行内核合同继续有效；其交付不等于完整语言支持或产品替换。逐能力/逐层缺口见[AV语言能力矩阵](strategy/av-language-capability-matrix.md)，从真实原始Auto程序到默认迁移/退役的G0–G8、优先级和门槛见[专项路线](strategy/av-source-programs-and-avm-replacement-roadmap.md)，种子程序见[AP目录](../plans/reports/av-source-program-catalog-2026-10-20.md)。

新增[827](../plans/archive/827-av-source-program-baseline.md)仅承接G0证据清单与G1原始scalar源文件闭环，实际构建/执行等D798；不等待820–826全完工，也不反向阻塞798。820–824安全数据链与825/826性能链保持原依赖/验收，后续按程序需求补源码/标准库/consumer计划，不把新增全语言范围塞入当前Plan。817/818仍限其已批准scalar面。

wrapping/trap、负索引、str字段、捕获/alias、Unit入口与错误通道的迁移差异列为SEM-01..08，由源码/语言owner给具体材料与profile决定；本次不批准语义改变。G7只表示特定SKU默认AV，G8才表示所有决定保留的入口不再需要旧AVM；旧ABC兼容、公开API和fallback须有明确策略。完整退役/AAC自举没有由此自动变成v0.6硬门或固定日期承诺。本轮仅文档，未执行程序或改变默认。

## 14. C-19／K06共享可变捕获源码切片（2026-11-01，target-state）

正式实施合同为[PLAN-909](../plans/909-shared-mutable-captures-source-closure.md)。本节是拟实施语义与边界，不表示current Spec或运行时已支持；865/823归档合同与908在途规则保持。一个包完成共享分析→Checked HIR/descriptor→唯一共享lowering→MIR/ABC验证→AV→源码TestCase及真函数增量；不另造仅有cell平台而无源码消费者的前置包。

### 14.1 先区分私有可变环境与真实共享

原`corpus:K06`为`move`闭包：base=1，依次f(20)、f(0)，结果42。Copy标量按创建时值复制，闭包自身写入跨调用保留，但不写回原base；原外部base仍为1。它不能单独证明非move捕获共享。原K06源码/hash、行为oracle42均不改；仅在新能力实证交付后迁移AV源码check的旧unsupported断言，不动legacy/a2r差异或金样。

本切片新增的共享写捕获限定函数局部`var int/bool/char`（Copy标量），支持整体读写、多个非move闭包共享同一词法绑定，以及拥有cell的闭包安全逃逸。按稳定BindingId而非变量名识别，遮蔽不别名。对一个绑定，只要同一词法拥有者内存在非move写捕获，拥有者和该绑定的非move读/写捕获均引用同一cell，即便只读闭包在写闭包之前创建。若无此类写捕获，沿865原Copy快照语义；`move`始终按值/转移，绝不因兄弟闭包而变成借用或共享原binding。已有owned move、显式clone、非逃逸Str借用仍按原域支持。

共享组静态确定，在var声明初始化时求初值一次，再记账/分配并提交cell；未执行的writer分支不改变reader的绑定模式，未创建的闭包不持owner。move从当前cell取标量快照。分配失败保正确资源/infra分类及已有初始化副作用，不重复求值；闭包绑定自身let/var不改变捕获源的mutability。

首期不支持把Str/record/List/Variant等拥有值放入**新增共享写cell**，不支持对捕获的let写入、mut/view参数提升、闭包值cell、循环环境、跨挂起借用、async/线程、持久活闭包、native ABI或A2R闭包发射；必须具名拒绝，不能默默clone或退回快照。C-19完整大类仍有循环图等后继，本包只交上述源码切片，不能全行改“完整支持”。

### 14.2 生命周期、clone与事实出口

cell由词法拥有者与闭包环境持有，返回后可由合法逃逸闭包保活，不引用已退出的栈槽；最后owner释放一次。闭包move转移owner，不复制cell；显式clone深复制其可达环境/cell到独立图，clone图内同一共享节点只复制一次并保内部别名，禁止clone仍共享原可变cell。捕获构造/clone采用prepare→commit：提交前失败不消耗原owner，部分产物回收；正常返回、trap、资源拒绝、取消、close及外部pin最后释放均核实活cell/env/owned字节回到基线。

共享cell模式、绑定身份、读写/逃逸效果、创建顺序、环境布局与清理责任由共享语义分析及验证出口提供；lowering与后端不得重做所有权分析。T-02以K06+真共享+逃逸+clone+失败回收和warm编辑最小链选择受检表示：优先论证既有集合/环境机制是否足够，不预定新opcode/tag；若需要新schema/语义身份，先冻结全部生产/验证/消费边与拒旧/兼容门。不得把普通可复制List伪装成未经验证的cell，或扩大成循环GC/完整平台。

### 14.3 函数增量与908交接

现行`closures_865_support::apply_source`会整源编译并lower/encode全部函数，再比wire；提交数为1不足以证明未改生产=0。909必须复用/有界扩展既有函数凭证、接口环境与片段路径，实际计数parse/project/resolve/typecheck/HIR-build+verify/lower/MIR-verify/ABC/decode/admit/工件IO，未改函数生产0、同一HIR/MIR/code对象及代际保留。捕获mode/布局/逃逸及读写效果是body事实依赖，签名未变也能使实际读者失效；活cell值不入代码cache或持久化，恢复只重验代码再重新执行。

909主写S/P：core source/HIR/descriptor/verify、`experimental/ace/crates/lowering`、av-core闭包验证/执行及AV会话与私有反例。908主写binding/适用规则/manifest/40场景oracle/852验收入口/治理测试。公共H（865源码case、K06的AV状态/注记、worker/adapter/固定组/Spec导航）单写短窗，交接记录版本/hash、具体符号与消费者；不要求D908整包前置，不消费其WIP，不改NA/required/原源/行为expected。908先合入则909接受其治理基面；909先合入则908按新能力身份重核K06相关证据，不能把治理批准当能力交付。详见909 §5.8。

## 15. Native源码诊断事实通道与M1 B17–B19（2026-11-01，target-state）

**2026-11-01规划复核r2**：911仍drafting0/10，V扩至20；操作级不可变原因快照、真实终止profile/完整候选上下文、原始JSON先准入、typed拒绝/首失败停、全出口真实attempt及W-S1/2/N/H/M1释放是必需合同，不是已实现声明。实际入口矩阵区分标量worker warm与独立OwnedSourceMnSession，保原强门而不新造多域worker。

[PLAN-911](../plans/911-native-source-diagnostics-and-m1-b17-b19.md)承908已交付治理，单独修native现有前端诊断的有损传递，不新增语言检查器。原B17/B18/B19行为oracle分别为type、assign-immutable、missing-return，现行前端已有`source.type-mismatch`、`verify.immutable-write`、`verify.no-return-path`；driver将其包成`mn.unsupported.frontend`，worker只传code/message，parent又按该前缀归parse，故不能用旧generic check绿关闭M1语义族。909正在实施共享捕获，其S/P主体不依赖本包。

拟增加显式、版本化的native source诊断观察模式：默认/未选择者保旧字段/分类行为；严格模式按真实compiler诊断生成有界结构化事实，包含raw stage/code/message/span、profile/producer、source身份及primary顺序，由native库→worker→parent同源传递。父侧逐项核版本、来源、完整性和上限，再把text/bind/verify映射到parse/resolve/typecheck，保原raw字段；M1按批准表映射三个语义族。不能用case ID、expected、消息关键词、Debug反解析或第二次重编译推断原因；unknown/缺因/错误原因不补绿，infra/resource/backend/link/run不伪装语言合法性拒绝。

暖编辑事实随本次失败Err拥有，不从last-errors迟读；绑定base/candidate、operation/session/context、accepted-prefix/step及raw span基准，不能贴上旧snapshot身份。source frontend/HIR语义拒绝不commit、无后段codegen/link/run；非快路径/关闭/陈旧候选及后段MIR/resource/link错误分别保原分类，后段prepare工作不假报零。严格多step首失败即停，不能把tail修复物化完再报成功；语义错不fallback，允许的早期非快路径可第一次whole编译并明确记账。source-v1路由用typed refusal kind而非contains字符串；后续独立API修复与one-shot新进程恢复分别验，不能冒在线HMR。必要AV只读typed失败出口与909短窗，不改其捕获、IR或提交/生命周期。

版本/模式在decoder、worker和parent同时冻结；旧worker或缺typed cause不能在严格模式降级为generic PASS。载荷总量≤64KiB、条数≤64、单message≤4KiB，UTF8 span合法，超额为资源/协议错误，不截断后授信。源码/worker流/Job/deadline沿原更严cap。诊断证据不产生COFF/PE、不进成功代码cache，不保存PASS结果；有效程序的产物/数值/ABI/缓存身份和原全部域保持。

严格模式互斥成功/source-error/非语言错误：source-error可带worker元数据容器，不能夹带执行成功结果。原始字节的重复键/字段类型/整数primary=0、mode/实际工具/候选关联先核，再与oracle比较，协议错是ERROR而非可被expected error满足。显示位置不得clamp/造0，raw事实/次序/合法重复诊断保真。计数来自producer真实dispatch，所有失败早退也快照；只在成功后bump、Default零值或whole后filter都不能证明未改生产0。必要阶段未观测就是未闭，不冒PASS。

父侧优先用原ArtifactRef发布已核诊断事实，完整绑定source/mode/tool/operation/context与execution_key、hash及正式源/check；原子发布后下一次真实读取仍须准入。旧generic报告、半文件/错hash、raw ok或手改token不授信三格。工具SHA/能力元数据因新构建改变时按原STALE重核，不假称新旧worker或全部report字节相同。W-S1只读失败快照、W-S2实际attempt钩子均须909具名稳定接口与全consumer收据，Rust加字段不等天然兼容；N/parent可先行，必需warm门不能延期给909。

只迁移B17–B19的native具体check到严格模式及正确实因，原source与binding expected不变；NB17/18/19仍真运行2/2/0，补wrong-cause/伪worker/旧版本/超额/失败后修复反控。D908的manifest/规则/报告准入按正式版本同步，三格回填必须绑定真实父报告，其他gap/NA/accepted/认证轴不动，不借此签M1。911 N/parent主体可与909并行，AV只读诊断出口、公共注册和M1各自格位短窗；910为独立Cargo治理规划，不互设整包前置。全部任务/Spec增量/验收及具体写面见911，本节不表示已实现。

## 16. 909后继：闭包源码生产粒度与恢复入口（2026-11-02，target-state）

909已交付共享标量捕获/K06及owner级缓存，不重开其归档。本轮核实际代码：`compile_closures_source_incremental`仍整源parse、模块plan/render/bind及verify_closures；同owner兄弟lambda随owner重走；新会话需lower/encode取得A23按flat字节计算的unit-content key。现有verify_function_ref明确不接受Closures域，因此不能靠打开一个match分支或复制CheckedModule句柄冒单函数受检凭证。

**拆为两个有独立验收的计划**（本轮由中央号源正式分配）：A“闭包源码单函数前端生产收口”，关闭R1/R2/R4及隐藏整源body重解析；B“闭包源码来源绑定恢复”，关闭R3与恢复后的下一次编辑。A验暖单body未改全阶段生产0、B验可信来源锚定后的重启无前置生产且深重验。B以A正式交付的不可变接口/身份为实施前置，codec/反控设计可先行；不拿A单独交付宣称全部恢复门已闭，不把B变成无消费者cache平台。

正式号源：[PLAN-912](../plans/912-closures-function-production-closure.md)承A，[PLAN-913](../plans/913-closures-source-bound-recovery.md)承B，均drafting r1、0/12。执行顺序912→913；911仅按真实S/H符号释放，不互设整包前置。两个计划的原始语义/来源/独立oracle、预算与关闭点各自验收，不再分配未落实消费者的第三包。

### 16.1 A：单体凭证、稳定身份与真实零生产

复用现有片段/CST、CheckedEnv/FnCandidate、单body emitter、受检类型/资源事实和共享lowering；为Closures补有界的单函数凭证与无其他body的接口上下文。上下文验证覆盖closure创建点/唯一性、callee/env布局、共享/私有capture mode、类型/绑定/效果与资源责任，body只由验证器授信；lowering消费凭证和验证器事实，不新造所有权分析。未读事实变化不能强制重验，已读类型/实例/CT及capture/effect变化不能只凭调用点未改命中。

支持原closures源码域内已存在函数/lambda的单body UTF8编辑，包括同一owner内兄弟lambda不变的情况。保持稳定逻辑function/lambda/binding身份；909的token偏移BindingId和owner内lambda序号不能在位置变动后被当成永久语义身份。源坐标以独立映射更新，不为保hash重建未改HIR/MIR/code对象。创建/删除/捕获布局/签名等变化按实际依赖失效；无法证明身份匹配的语法变形明确non-fast-path，不能把普通支持的body编辑也统统赶去fallback。

冷初始化可整体验证；暖路径只解析/resolve/typecheck/plan/bind/verify/lower/encode被编辑的实际函数与语义依赖者，未改实体全0且同对象/代际。允许有限声明/类型目录遍历、输入hash及坐标映射，必须分账，不能遍历其他body再报0。每个真实dispatch含失败出口取attempt；whole后filter、默认零值、同hash换新Arc均为强制红负控。A保原AV执行、共享cell/clone/清理和其他producer域，不扩共享owned/循环图/语言面。

### 16.2 B：来源锚定恢复，不靠重生产算key

A提交点导出不可变的函数/source/context/依赖/代码绑定数据，B在同一managed store中发布有界source recovery manifest及required-data闭包，不保存受信Checked对象、live env/cell/栈/地址/PASS。source-only lookup只是hint；纯恢复需调用方从上次真实成功生产保留并授权的manifest强摘要/根绑定，连同完整source SHA、logical owner、entry、profile、cap/config、compiler/语义版本一起核。自洽索引或自报source_sha不能证明source与code的关系；无可信锚点/被改索引不能授信恢复，走明确cold或拒绝，不把重新编译叫恢复命中。不引入通用签名/远端信任平台。

新进程先验预期root/manifest与闭包依赖，再读未授信的前端上下文、HIR/MIR/ABC和unit envelope，真实decode/深验/授权/安装并重新执行。允许恢复阶段的结构/类型/资源重验和必要link/admit，不允许调用源compiler、source parse/resolve/typecheck/Atom plan/bind、lower或encode后才算key。重验逐项单列，不能改名掩盖源重编译；还需恢复可供**下一次**合法单body编辑的安全上下文，下一改未改全0，不能只跑一次恢复执行后仍回全源重建。

坏/漏required项、跨owner/profile/config/producer、同上下文异body、内payload合法但binding错误、授权收回、GC/并发/发布取消等沿原准入/原子合同。host预期根摘要不改而篡改全部缓存自描述字段仍须拒。cold重建修复后下一次独立恢复必须真正零前置生产；只读命中不写artifact/root/数据，所需lease/control动作分账，严格只读情形使用已有授权root/租约。active pin保活到最后owner，不因历史列表空就记释放0。

### 16.3 分层计量、预算与交接

共同的真实计量区分P源生产、R持久payload重验、M接口目录/源hash/坐标工作；每层按函数与whole轴记实际attempt/对象/IO。暖编辑未改实体P=0，恢复命中全部P=0、允许R>0，不能宣称“恢复所有工作0”。同源固定oracle+文件化TestCase覆盖fresh/hit/restart/body-edit/历史/错误后恢复；四状态各30样本、正式P90≤2000ms与资源/结构同时过，原更严门及长期亚秒目标保留，准备/编译/运行/清理分列。

新增保留前端状态（current/candidate/history/实际pin及intern/索引）合计≤16MiB，source≤4MiB；既有code current+history8MiB/每unit32、值8MiB、verify/decode各16MiB不放宽。B单恢复manifest≤4MiB、自己管理的恢复namespace≤128MiB（共享required backing记账，不动他方store），active/lease/root受保护；父private/working-set峰值按原更严门并拟增量≤128MiB列测。新池有失败原子/GC/最后owner门，不藏旁路缓存；这些新预算是本轮拟合同，必须由最小全链验证可行，不能自行加大或缩原规模。

911仍实施：它主native/parent与通用失败快照/attempt事实，A主closures source/descriptor/单函数验证、共享lowering及closures会话，B主来源manifest/准入/恢复。core body_work、verify调度、av/session只读出口、公共contract/worker/组与Spec为具名S/H短窗；共享观察尽量同源，不并写、不改变911语义族/首失败/默认兼容。不把D911整包设前置。A→B接受正式SHA/类型/凭证/序列化版本及消费者收据，稳定接口再变则标受影响证据STALE；不消费WIP、不要求整波一起结束。910构建治理仅真实工具/注册窗口，D-909-1 native生命周期既有红单独分诊，不混成A/B闭包前端能力或暗改expected。

## 17. M1下一阶段：场景观察、AV资源与旧A2R（2026-11-02，target-state）

> 本节34格/24格/剩10格为新增C目标前的M1基面；2026-11-02用户批准A2C第二目标纳入M1，当前新目标与总签章见§18及独立范围增补。914–917原合同/写面继续，不因新增C目标回开或被动扩容。

本节以main `0e93e3cc9`为规划基面：D911=`ca4848f5d`、D912=`13a465336`均已交付归档；913已建树并完成T-01，头部状态仍drafting的簿记滞后不代表窗口空闲。§15/16早期执行状态保留为历史。本节为下一阶段设计，不是当前实现或签章。

### 17.1 四包职责与逐格归属

正式合同：[914场景观察](../plans/914-m1-evaluation-order-and-layout-observations.md)、[915 AV资源](../plans/915-av-resource-fault-and-stale-place-conformance.md)、[916旧A2R静态准入](../plans/916-legacy-a2r-semantic-admission.md)、[917旧A2R发射/文本证据](../plans/917-legacy-a2r-mut-record-and-text-evidence.md)，均drafting r1、0/10。只规划，未建实施树/材料，不改当前Spec或原source/expected/binding。默认旧A2R公共入口短窗916→917；914/915与913按实际S/H符号交接，无整包串行。

| 包 | 目标required格（ns必须参与身份） | 生产/观察合同 |
|---|---|---|
| 场景观察首包 | scenario E01/E02的ac/av/avm/a2r共8格；I04/I09的ac/av共4格 | 接真实命名求值trace及布局依赖/活状态证据。E01=7且ba，E02=103且ba；I04实际布局依赖失效/无关函数不生产，I09按当前策略受控拒绝或明确重启要求，不做native在线loader/OSR |
| AV资源故障与代际 | scenario R02/R03 av共2格 | 复用StrArena/ExecLimits既有fault seam和List Place运行期检查，补正式逐点注入、事务/清理/危险访问0证据；先核已有防护，不把全局句柄重构当默认方案 |
| 旧A2R静态合法性 | corpus B16/B17/B18/B19/T06/R09/R10 + scenario E03的a2r共8格 | 真实TransRust生产入口在发射前返回可辨识语言原因，未执行分支亦查名；负例不生成可执行金样，不把rustc错误/infra当Auto拒绝 |
| 旧A2R发射与文本证据 | corpus R07/T08 a2r共2格 | R07真实mut-record写回得42并产生正确类型/所有权Rust；T08保原源及slice-oob，独立审阅金样。日常emit+bytes/证书核查零Rust编译，行为认证显式分离 |

四包各自有真实消费者，共认领24格，互不重复；当前34阻塞仅为读取binding/applicability后的静态复算，不冒最新父报告或formal acceptance。全交付也只预期余10格：R01/R04/R06的ac/av六格、R02/R03/R05 ac三格、B18 avm一格；仍须独立验收，不能以计划数代替M1 gate。native R05涉及closure而现NA白名单未认领该格，后继先核规则/真实生产能力，不自动扩NA也不为了过M1暗建native闭包ABI。D-909-1、D-911-1..4及A2R 38个cert-gap保具名，不随本波悄然关闭。

### 17.2 观察来源与信任边界

源/typed scenario fixture→真实production producer→同操作事实→有界worker/adapter artifact→父侧严格准入与独立oracle→有效父报告→逐格回填。scenario原来无source不代表可拿模拟executor代替真实引擎；新fixture只把已冻结oracle变成可执行材料，不改原corpus/source/expected/分母。observed trace、依赖、拒绝原因、fault attempt、代际必须由真实读/写/执行点产生；父请求不带expected指导worker。截断/缺事件/坏类型/重复键/错operation/旧source身份/泛非零均不通过。

a2r仍是旧production TransRust身份；新HIR-A2R（892/902/904）并列保留。E01/E02日常a2r腿只能emit/金样核查；首次trace行为若声称已实证，须显式真实actual/reference编译执行并绑定值+trace，新增独立场景证书域或等价受检事实，不覆盖旧证书schema或让日常spawn Rust。未知前端支持/布局策略由最小全链feasibility先定；若达成既定AC需扩大语言/ABI，必须needs_replan，不能删格或把“缺能力”审计当实施PASS。

AV R03保现有借用语义：合法Share/Static StrView继续可用，不能把故意持有旧buffer的share误判成stale；R03运行期反例用实际可授信List Place，验证frame/slot/buffer/epoch/element-generation及访问前拒绝。静态借用拒绝与运行时stale分别观察。R02先覆盖既有支持域Str、record、List<Str>/owned record、Variant与closure capture，fault=资源/infra分类按真实接口保真；失败后回到未注入合法链，不缓存运行PASS或恢复旧cell值冒新运行。

### 17.3 增量、预算和913交接

四状态fresh/hit/restart/body-edit、失败后修复与下一次真实运行、未改函数真实P=0/对象代际、恢复R/M/P分账沿已交付支持域验收；旧A2R cold发射没有函数级生产合同，不把其重发射统计假标0；若改共享前端/lowering必须验下游实际受影响域。正式周转仍P90≤2000ms、原样本/规模/更严cap与长期亚秒目标保留；显式Rust认证原编译120s/运行30s独立，不塞进日常P90。新观察材料≤4MiB/操作，流≤1MiB/流且原更严者优先；AV值8MiB/受管32MiB、父增量128MiB及原深链/512 churn等不放宽。新fault/trace默认关闭，失败先于变异，trace超额受控拒绝不静默截断成PASS。

913独占来源manifest/root/lease/restore/accepted-context与闭包恢复schema。场景包不改它的恢复算法；AV包主av-core executor/arena/place与resource观测；旧A2R包主auto-lang-core parser/infer/ownership准入及trans/rust发射，不改ACE受检凭证。交叉av-core/closures.rs、public failure/body_work/只读观察、ac_av.rs/test_cases/mod.rs/auto-test-case合同、fixed组和M1 binding/manifest必须逐符号/全文件单写短窗：owner+基面SHA+版本+构造/匹配/全部consumer+验证/释放。接口再变则受影响证据STALE；未取窗只挡具体任务，不设D913整包前置。正式性能/认证与913错峰，不持写窗等机器、不停他方进程。

## 18. M1新增A2C第二目标（2026-11-02，target-state）

> 日常开发与产品替换优先级见§19。A2R/A2C为M1必需能力，但不要求日常Auto运行先生成Rust/C再编译宿主。

用户已明确批准，见[M1范围增补](../plans/reports/ace-m1-a2c-scope-addendum-2026-11-02.md)、[Design51 r5 §13](51-shared-a2x-and-hir-to-rust.md#13-a2c第二目标与m1范围增补2026-11-02target-state)、[C实施路线](strategy/shared-a2x-and-hir-to-c-roadmap.md)。A2C与A2R共享Checked HIR/单函数凭证/资源事实/结构化prepare及依赖/产物服务，分别目标合法化；AC/AV仍走唯一共享Typed MIR，不改为先生成C，也不从CFG反造源结构。AutoABI/foreign adapter/FFI依旧分层，C生成成功不授予公开ABI兼容。

现a2x-store直接依赖Rust provider，CA-A必须以C真消费者带动目标合同/session/store去耦；CA-B record/text资源→CA-C集合/变体，CA-D沿797有限candidate接真实C/AutoABI三腿，CA-E收新增target ledger/Q4/总门。五个工作标签未正式取号，C未实现；每包首例纳文件化/独立oracle/真认证/未改P0/恢复深验/原预算与P90≤2000ms，不把资源和增量推给最后一包。

Q4固定AC-native/AV/ACE-A2R-HIR/ACE-A2C-HIR分别对oracle；旧AVM/A2R/C保持其身份/义务，旧NA/accepted不自动套C。新M1=旧M1-base全部门AND新增C语义/资源/增量/恢复/有限ABI/Q4与登记；旧acceptance/2仅旧基面，缺新增门期间不可签扩大后M1。不得往现12列binding硬插C列或改旧批准件SHA令在途证据失效，新target ledger由独立批准件与后续CA-E版本化接入。

913恢复、914场景、915 AV资源、916/917旧A2R的写面按原合同保持；C私有provider/参考可并行，core凭证/资源/ManagedStore/A2X层和public adapter/SDK/固定组/manifest/证书reader按owner+实际SHA/版本/全consumer单写短窗，正式计量错峰，无整波新前置。本轮只设计与规划，不改current Spec/源/expected或在途计划，不建树/材料或运行测试。

## 19. M2补全ACE并替换旧AutoVM/A2X（2026-11-02，target-state r2）

**版本绑定（2026-11-02最新裁定）**：本节M2全部完成并disable旧实现是v0.6发布硬门，与M1含A2C及[发布门V06-ACE-01..05](../plans/reports/ace-v06-m2-release-gate-2026-11-02.md)共同验收。为此可调整发布时间（如晚一周），不是已测定工期；先核全场景关键路径，不以双引擎兜底或降低原门完成发布。下方或前文旧“退役不自动成为v0.6硬门”仅保历史，已由本裁定取代；AACE仍v0.7，本轮不实际切换/disable。

用户已明确M2唯一当前目标：“补全ACE能力，并替换旧引擎（AutoVM/A2X）”。执行合同见[M2 r2/REP-01..11](../plans/reports/ace-replacement-first-m2-contract-2026-11-02.md)。M2结束必须AutoUI/AutoMan等全部现有应用场景、旧A2X全部现有目标/公共入口及真实安装发行接管，并完成旧实现disable；不是只证明语言完整或替换就绪。旧M4迁移/部署/退役义务已并入M2，M3/M4未来新增扩展暂不规划。

开发快路径选择预构建ACE/宿主上的AV，或已明确支持入口的AC函数产物；Auto源码编辑只驱动真正依赖的语义查询/HIR→MIR/ABC或native增量与安装。不要每次编辑生成Rust项目并cargo重建宿主/SDK，也不把Q4全目标显式认证塞进编辑循环。A2R/A2C继续承担发布/导出/独立认证；源码转译器替换与默认执行入口迁移分别核消费者，不强求每项都经同一后端。

v0.6引擎仍Rust，不先做AACE即可达到Auto业务编辑Rust构建0。Rust宿主/库/少量shim低频编译并按接口/实际依赖/工具/配置复用；其源码或真实ABI/配置变化仍可能重建。Auto到Rust库通过受控typed host import/已验证ABI/FFI接缝，借用、错误、原方释放/回调保活保持，不为了调用现有一项库功能先完成任意Rust crate自动绑定平台。

REP-01..11是M2完成门：原源/全消费者/全目标语义、实际rustc/cargo进程0、跨模块/stdlib/host真实依赖与未改函数生产/IO0、同PID兼容热更新与旧链保活、失败原子/最后pin回收、restart重验/有界工作集、原工程P90≤2000ms/更严资源、全入口安装发行切换、旧实现disable与新测试/证据迁移。type/meta/实例/CT/权限及负依赖不按调用点源码盲命中；不把单源微例、重启命中或wrapper计数冒工程级完成。

不兼容layout或外库ABI变化可按实际产品合同明确拒绝/要求重启，不承诺未使用的新任意状态迁移；当前工程所需的兼容编辑和UI/Man/服务/文件/网络/外库生命周期必须真接通，不因旧M3分类免验。分入口opt-in/切换只是进度；M2终态必须全部旧支持矩阵已接管，不能只新AV/A2R/A2C绿或只首帧/空命令。

旧执行循环/编译器/AST emitter及其注册、构建、链接、安装、发行、调用路径在M2全部退出，default/all-features/工具/日常测试没有可启用legacy实现、隐藏flag或fallback。薄compat可保旧名但只转ACE；共享值/诊断/中立工具/Rust库先按依赖归属，不按vm/trans目录名盲删。历史用Git追溯，新发行不内置旧引擎，回滚用独立已发布旧版本。新M2 ledger/recipe/测试/证书迁移原义务，旧M1批准hash/报告保历史，不为日常对拍永久编译旧引擎；REP-09..11在真实代码/依赖/dispatch/SDK安装和全consumer证据上关闭后才签M2。本轮只设计，未实际disable。

## 20. 915/917在途时的下一波（2026-11-02，target-state）

用户授权提前规划下一波，不实施。读取main=5a0ef7609；913/914/916已有交付收据且为当前祖先；915和917实际有实施树，915头仍drafting但current_step=5，不能据头标空闲或代改进度。下一波分四条有界线：资源事件观察、旧AVM B18 typed诊断、CA-A第二目标、M2全场景替换盘点。后者交付可执行拆包与跨仓迁移分母，不把一次审计冒成M2实现。

### 20.1 逐格范围与为什么拆包

| 方向 | 认领 | 不认领/后继 |
|---|---|---|
| 资源事件观察 | ns=scenario R01/R04/R06 × ac/av，六格 | 915仍独占R02/R03 av；native R02/R03/R05三格另待能力调查，尤其R05不可从native collections推定已有闭包ABI |
| 旧AVM B18 | corpus B18 avm唯一格；真实immutable assignment生产错误与父侧严格诊断准入 | 不重写旧VM/全部诊断、不扩大accepted/NA；旧实现修复只为原M1义务，M2仍必须disable |
| CA-A | 共享目标合同/会话去Rust耦合＋HIR→C core/scalar闭环 | 不吞CA-B资源、CA-C容器、CA-D public ABI或CA-E总签章；首例登记/认证/增量/恢复不能延期 |
| M2替换盘点 | 全源/全消费者/全目标/入口/安装/依赖与REP-01..11映射、实际能力与证据分类、关键路径/后继包 | 不执行跨仓迁移/切默认/disable；不重复903旧M1审计或918收据/边界修复 |

旧required当前静态14格=915两格+917两格+本波六格+本波B18一格+native三格。只是登记复算，不是新acceptance执行结果；旧认证轴、新A2C所有门、M2工程与产品门另账，不能由14清零推出新M1/v0.6可发布。native三格由ACE native路线owner保持，M2盘点给R05源码/typed/ABI能力证据和可执行拆包，不自动NA，也不要求为三格立刻重做全native闭包平台。

### 20.2 事件来自实际owner动作，观察不制造语义

数据流为受检source/typed fixture→既有HIR/MIR/ABC或native→真实构造/转移/释放/取消/异常动作→有界事实→父侧按独立oracle判。已有AV StrArena alloc/free、release_frame_owned_lifo和native collections frame_leave/节点计数是起点；总alloc=free、返回42或杀掉进程不足以证明逐owner与LIFO。事件身份区分logical owner、共享backing、代际和转移，禁止跨allocator裸id合并；只最后owner释放backing，活动view/返回值/pin须保活。观测不得延长owner寿命、触发深clone或重排drop。

R01覆盖正常/early return、分支、break/continue、临时与转移；R04覆盖old臂释放一次、inactive payload零读和42；R06在活owned树/集合及AV支持闭包时真正到达budget/cancel/trap，再清理到基线，最后宿主pin后live0。native closure不在本包支持假设内。可造受检typed运行态的场景保source/typed身份，不能拿未授信IR或进程退出代替。观察空间先限额，写满给具名resource拒/原子保旧，不能静默截断后PASS；清理不受已耗尽guest燃料/取消阻断。要修实际已承诺清理漏洞即在包内修，不只补打印。

函数暖编辑承诺沿真实支持路径：body仅改函数及实际依赖者生产，至少三个未改函数的真实阶段/每函数工件IO0、对象/代际保持；原AV集合whole-source前端不冒parse0，逐层计量并列残余owner。声明编辑沿914 D-05允许冷声明前端、精确rebuild；不把此例外扩大到body。恢复用913已交付来源/有界读取接口、caller上下文重验，P/R/M分账，失败修复后下一次恢复复验。值8MiB/managed32MiB/父增量128MiB、观察4MiB/操作及流1MiB与原更严cap保持；原深链/小栈/512churn及正式暖30样本P90≤2000ms保留，不推广成普通VM小修的性能门。

### 20.3 可并行的写面与发布边界

资源线的AV执行/arena/结果观察与915取逐符号稳定SHA短窗，不消费其未提交WIP；native私有观察/fixture可先做。B18主vm/codegen和avm adapter；CA-A主a2x-core/store/provider/worker；M2盘点只写自己的报告与Spec提案、只读跨仓。H包括test_cases/mod.rs、ac_av.rs、ac_mn.rs、a2r.rs、ac_protocol.rs、auto-test-case、fixed组、workspace lock/manifest、binding/source-manifest，全文件单写；开窗列owner/实际SHA/schema/全部构造匹配consumer/已验/pending/释放，不能以append-only并写。与917共用H时继承其窗口，918的web_delivery/modularity规则不动。正式认证/性能采样错峰，不持H窗等待编译。

CA-A用真实C消费者验证共享接口，不建立第二DB/前端/runner；共享prepare/语义key无target字段，fragment/manifest/cert有完整target/tool/runtime配置身份。C整数helper不得依赖signed overflow、除零、MIN/-1或C实参顺序未定义行为；短路/源序临时显式化。C pure source restore不得先cold_assemble重生产取key，历史restart_assemble今天仍调cold_assemble的事实在首包有界修复，不能借913的AV恢复收据宣称已支持。

M2盘点以真实可达生产入口和安装consumer为分母，AutoUI/Man/Web/Gen、CLI/SDK/嵌入/服务及所有在用旧A2X目标都要来源/owner/用例/新接口/缺口/关闭oracle。跨仓本包只读，各仓修改另立对应owner合同；未运行/不可访问/接口不明保持具名未知，关键路径给依赖与容量依据、低/高区间及未知，不从计划数猜晚一周足够。M2继续是v0.6硬门，未来M3/M4扩展和AACE不加入当前前置。

## 21. Native R02/R03/R05：故障与Place基础、有限闭包分包（2026-11-03，target-state）

用户本轮只授权正式设计/立项，不实施。main读取9c91a1b48；D915=5ba5d4cd2、D919=4ea3daddf已交付且为主线祖先。915交的是AV逐attempt/受检Place，919交的是AC/AV事件观察，均不证明native R02/R03/R05已实现。旧required313静态仍三格；新A2C/M2门与旧认证轴另账。

### 21.1 实际能力与两包裁定

native `collections_lower::admit_layouts`只接profile4，白名单无ListPlace/PlaceGet/PlaceSet及CreateClosure/CallClosure/EnvGet；`collection_interface_parts`拒import.env。Node只有创建id，没有List结构epoch/元素代际。已有`allocate/fail_nth`、text shim的独立fault/ledger seam、clone显式工作栈、precheck→commit与919 MN-EVT可复用；它们不是完整故障覆盖或闭包ABI。源与共享MIR已支持865/912的有限闭包/捕获事实；native不得复制捕获分析、恢复旧HIR→CLIF或调用AV冒充机器执行。

据此拆两包，而非一个大包或三个互相重复的场景包：

| 包 | 实施/验收职责 | 逐格关闭与前置 |
|---|---|---|
| [NF / PLAN-923](../plans/923-native-allocation-fault-and-place-conformance.md)：native分配故障与Place基础 | 原L04/L05/V05、List<Str>/owned record与typed深clone逐真实allocation attempt；源/目标原子性；新受检Place机器链、逐用代际与危险访问0 | 独占关闭scenario R03 ac；交付R02非闭包证据/版本化fault接口，R02 ac保持gap，K02/capture-env域交NC |
| [NC / PLAN-924](../plans/924-native-closures-source-and-resource-closure.md)：有限native闭包源码与资源闭环 | 865/912受检事实→共享MIR→COFF，真实创建/返回/间接调用/clone/drop；K02 creator退出后42，env/backing最后释放；补K02/env-clone逐fault | 独占关闭R05 ac及**完整R02 ac**（聚合NF非闭包与本包闭包事实）；消费NF稳定接口，无NF等NC才能交付的反向依赖 |

NF可独立交付/归档，不能因K02尚缺而勾R02全部通过；NC最终必须重核NF证据在其runtime/ABI变更后仍适用，不以旧收据无条件拼全。NF后旧阻塞3→2，NC后旧三格才全闭；这仍不等于含C的新M1或M2/v0.6已通过。NC私有前端/接口研究可与NF并行，native公共N写面先NF后NC，稳定提交可按符号提前交；不占用921的A2X主体。

### 21.2 故障、原子性与Place语义

NF冻结真实运行时allocation域及总数N：node/header/data、frame cells、List扩容、CloneWork、text backing/ledger及其必要metadata各有定义；相同申请不得跨seam重复计同一指标，compiler/runtime准备/OS页提交与guest allocation分账。既有text allocator hook直接走arena，不能只扫`mn_coll_allocate`就声称text全覆盖；预算precheck拒attempt0、注入命中、真实OOM、取消、语言trap与infra分态。逐1..N失败＋N+1/无注入正控；每失败点真实旧值/源/所有权/代际/已commit边界可观测，准备的半树逆序回收，未提交move不杀源。

两条证据腿同需：原source→真实PE逐fault终止/清理；与其链接同一runtime原语的受控native probe持有源/目标，比较故障前后逻辑快照、保旧及清故障后同进程重试。不能只因重跑源码仍得42推导中断时源未变，也不为观察而给语言新增catch/继续执行语法。所有探针是待work使能的新长期case，不是本轮实测。

Place使用不持有已释放地址的受控身份（frame激活UID/slot及owner代际、buffer身份、结构epoch、index/element generation或等价可证机制）；每次读写先在活registry验证，再解引用buffer。push/insert/remove/替换各按现行MIR语义失效，PlaceSet commit使旧token/同元素旧兄弟失效；合法新token/不受影响元素及Share/Static StrView保持。失败预提交不增epoch、不退役合法token。来源面跨结构变更的loan违规仍静态拒；运行期证人走`verify_mir`及native admit，不拿未受检伪MIR或静态拒绝代替stale事实。禁止用始终reject或事后trap过门，必须有读/写前验证灵敏度反控。

本期token读写证人限List<i32>/List<bool> Copy元素，PlaceGet不能读出owned；原List<Str>/owned record只读view/移出/Share/Static正控另验。未支持的native Place域明确准入拒绝，不改变共享MIR或源码借用规则，不把有限新准入冒成整个profile4支持。

### 21.3 有限闭包支持与native调用边界

NC只承诺R05最小但真实源码域：i32/bool Copy捕获、显式move的Str/非递归owned record；lambda int参数/int结果；普通factory的move资源参数、返回闭包、局部move/显式深clone及间接调用。K02原source不改，另补typed CaptureKind::Clone/混合捕获的失败原子证人。任意List/Variant/Closure嵌套捕获、Shared cell/K06共享写回、Borrow逃逸、Dyn/宽类型/Unit/public ABI/native loader/HMR不随本包开放；AV现有更广域仍可用，新native准入明确拒其余域，不改共享前端语义或旧NA批准。

native按既有ClosureBodyFacts/捕获源序/mode和env声明编译，env layout/signature/owner/声明依赖在生产与link/admit时核对；代码是原生函数符号和真实间接机器调用，不是把MIR/ABC放进runtime解释。闭包内部调用ABI显式携env/结果/参数（等价窄形状可由feasibility冻结），不冒797公开AutoABI；code pointer仅运行时有、不能落持久cache。创建callee必须实际env匹配，直接call lambda body与把body当entry仍拒；间接call错误signature/旧env/foreign handle在调用前拒，被调计数0。

env构造prepare所有fallible clone/容量再commit moves和目标；失败源与旧闭包可调用、部分env无泄漏。Clone是独立env与独立owned backing，Move只转移责任；EnvGet可借用/handle-share但不偷走env最后owner。creator返回只移交返回闭包，不能frame sweep提前杀env；闭包与调用中pin保活至各自最后owner，正常/early/budget/cancel/trap均恰清理一次。代码生命周期按当前固定PE image，不借“保活”偷偷承诺native在线换码；所有env/捕获backing须在进程退出前可核回收，不靠ExitProcess当析构。

### 21.4 函数增量、恢复与写面交接

NF沿889 CollectionsIncrementalSession，NC复用912 `ClosuresFunctionFrontend`/单函数凭证/共享lowering事实，接native COFF/session而非AV executor。支持的owner/lambda纯body暖编辑至少三个未改函数/兄弟lambda，真实parse/resolve/typecheck/plan/bind/HIR verify/shared lower/MIR admit/codegen/装载及每函数工件IO0、AST/MIR/COFF真实对象/代际保持；变化只及目标/实际读依赖者。closure code body与env layout分键，普通callee body不让creator/caller重编，读body者例外；声明/layout/拓扑变更沿具名非快/914 D-05精确失效，不能冷全编后筛同hash冒暖门。

fresh/hit/restart/body-edit及坏恢复/失败编辑→修复→下一恢复再验；纯工件恢复禁止unit codegen，MIR/COFF/runtime/owner/依赖/工具来源有界深验，不反序列化Checked/PASS/活env/宿主指针。既有889工件restore不自动证明恢复后的前端可暖续接；两包T-02分别核新进程来源/上下文重建→下一body编辑，必要的有界数据出口修复在对应包内完成，不把生产藏进R或延期到922。冷修复单列，修复后下一hit/restore必须真正复用；NC仅以不含活值的函数/捕获上下文和native工件恢复，不移植AV专用root/lease平台。

预算沿889：guest8MiB、native物理32MiB（含新token/env/代际表/ledger/stack）、compiler managed48MiB及原分区16/24/8MiB、父/PE private增量128MiB；源/元数据4MiB、流1MiB、919事件native8192行/AV16384与原更严门保留。NF沿116函数/345483B、fresh/hit/restart/body/history各≥30样本、512修订；NC沿912 64顶层/70单元、每owner≥3兄弟lambda及四态≥30/512修订，不能拿K02微例代替负载。暖P90≤2000ms，长期亚秒不变；runtime准备/编译与认证分列，稳定Auto body编辑cargo/rustc0。原深链/小栈/取消/燃料压力不降，token/环境表计入同预算，先限额后物化。

P私有case/codec/COFF探针可并行；N为native collections/runtime/layout/driver，NF→NC单写。公共S为core/source/单函数凭证/共享lowering/av-core MIR verifier形状，优先消费既有事实，确需出口修改与921按符号取SHA短窗；AV运行时915故障/Place、919事件实现不归新包重写。H为test_cases/ac_mn/公共mod/ac_protocol/auto-test-case/SDK/fixed组/workspace manifest/lock/binding/source-manifest，全文件单写，921现占部分需交接全部construct/match/producer/consumer。922已有实际lang-922树（头仍drafting），按在途对待：仅消费/刷新各自报告，不改其合同/进度或current实现；共同路线文件也取单写窗。每窗owner/revision/SHA/schema/全部reader/已验/pending/释放，接口变动STALE影响证据，正式采样错峰。没有D921/D922整包前置，两个新包的最终三格证据不能依赖921/922替补。

## 22. M1架构机制完整性与统一源码语法入口（2026-11-03，target-state）

用户将M1出口明确为：有限语言范围内，整套必需架构机制调研清楚并实现贯通，后续差距可以是语法/特性覆盖，不能遗留基础机制未接真实消费者。完整裁定、源码取材与SYN-01..06/ARCH-01..08验收见[决策件](../plans/reports/ace-m1-architecture-completeness-decision-2026-11-03.md)。这是目标增补，非本轮实现或测试结论。

源码入口收敛为版本化规则驱动的共享词法/无损CST→严格AST投影→共享语义检查与Checked HIR；冷初始化、暖编辑、错误修复、声明编辑与持久化恢复均须接通，适用AC/AV/A2R/A2C不另解Auto语法。scalar AV已消费通用CST增量链；手写整模块及其他profile片段入口是现有迁移基线，不得误报全面统一，也不得因其支持函数级增量而假定语法规则已经单源。M1最终不保留独立重复语法的生产兜底；迁移差分与隔离测试参照可留。

统一语法之外，依赖查询/实例与CT事实、共享受检IR、真实AV CTE、cache与恢复准入、资源/有限ABI、在线换版与有界回收、独立oracle和父侧严格验收均须逐项审计真实producer/consumer。原M1门 AND C新增门 AND SYN/ARCH门全部成立才签章。M2继续承担语言/在用目标与应用覆盖、实际产品迁移和旧实现disable，不能承接未闭M1机制。

“统一语法前端”“M1架构机制审计与联验”具名待立项；先调查可行性、覆盖与调用点再确定拆包，不预占号。私有调查可并行；公共S/H与923–926按稳定SHA及完整消费者取窗，不修改其在途合同/进度。CA-E最终签章必须等本增补各门交付并重验；原有完整AJ/block级增量等专项边界按决策件显式判适用，不因本条宣称全量已支持或自动豁免必需基础机制。

## 23. M1统一源码语法三包与机制余账（2026-11-03，target-state）

依§22用户裁定，已作[源码/Spec/收据审计](../plans/reports/ace-m1-architecture-gap-audit-2026-11-03.md)，正式拆[927 SG-A](../plans/927-unified-source-syntax-foundation.md)基础七profile→[928 SG-B](../plans/928-unified-source-collections-closures-syntax.md)集合/闭包/嵌套单元→[929 SG-C](../plans/929-unified-source-entry-incremental-retirement.md)全消费者增量恢复与重复parser退役；均drafting，仅规划。按后端拆会重建重复语法，合一则把运行时扩展、子体身份与所有恢复生命周期绑成大包。

依赖方向默认将Auto专属CST投影迁入core的source层，保core AST/Span/Profile身份，core只依赖中立syntax-runtime/grammar，syntax-core保持向core的兼容适配；禁止core再反依赖syntax-core形成环。公共规则/profile能力清单是语法来源，投影只映射结构/字面量，不重做类型、捕获和ownership。旧auto-lex等中立能力可取材，保留已批准差异，不机械复制旧Parser的Database/推导耦合。

927以七profile真实冷入口、片段接口/诊断/资源成立独立交付；928补九profile及只改lambda时不走owner/兄弟语法body，owner编辑复用语法不免当前capture事实重验；929统一所有源码driver/会话/工具/来源恢复，恢复后下一编辑禁止整源环境重建改记M轴。手写重复语法最终只可test-only或无规则兼容门面；Atom配置/受检HIR reader、不同领域语法与旧产品parser不误删。

923 N/fault与925/926 C provider/runtime主体可并行；source AST/candidate、924闭包registry/消费、A2X方法/type-enum片段、会话恢复与H面manifest-lock/父decoder逐具体SHA/符号单写，详审计§4。927→928→929公共语法面串行释放；无923–926整波等待，最终适用真实native/C认证仍需其能力和稳定接口，原合同/进度不变。

SYN闭环之外仍有SQ/INST/CT普通源码查询/实例/冻结事实接线、CA-D有限ABI桥和最终架构组合签章待立项。796模型与817真实AV执行及797 ABI候选分别保其已有证据，不能由独立交付冒称生产前端已全面贯通。新M1仍为原门 AND C新增门 AND SYN-01..06 AND ARCH-01..08；只剩语法/特性覆盖的目标不免这些机制缺口，M2继续全场景替换disable。
