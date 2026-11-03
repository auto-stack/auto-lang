---
plan_id: PLAN-924
status: archived
feature_name: 有限Native闭包源码与R05及完整R02资源收口
author: [Codex]
created_at: 2026-11-03
updated_at: 2026-11-03
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-ac/source-closures.md, docs/specs/auto-mir/native-collections-variants.md, docs/specs/auto-test-case/resource-lifecycle-observations.md, docs/specs/auto-test-case/native-resource-scenario-coverage.md, docs/specs/auto-test-case/ace-m1-scenario-oracles.md, docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-gap-audit.md]
new_spec_components: [docs/specs/auto-ac/native-closures-source.md, docs/specs/auto-mir/native-closures-resource-abi.md]
touched_goals: [GOAL-002]
affects: [auto-ac, auto-mir, auto-test-case]
depends_on: [PLAN-865, PLAN-912, PLAN-913, PLAN-915, PLAN-919, PLAN-923]
current_step: 11
total_steps: 14
---

# [PLAN-924] 有限Native闭包源码与R05及完整R02资源收口

## 0. 变更摘要

NC包：把原K02源码中factory创建/返回拥有Str闭包、creator退出后间接调用得42与最后env/buffer释放，接入共享受检HIR→Typed MIR→真实native COFF/PE；覆盖显式move/deep clone、env捕获准备失败与资源退出。最终独占回填scenario R05 ac及**完整R02 ac**，后者必须包含[923](923-native-allocation-fault-and-place-conformance.md)非闭包与本包K02/env全覆盖，不能只补K02就宣布R02过门。

依据[Design43 §21](../design/43-ac-av-typed-mir-and-abc.md#21-native-r02r03r05故障与place基础有限闭包分包2026-11-03target-state)。drafting r1、0/14，本轮只规划；923可独立先交付R03/故障基础，不等待924。本包不是全native闭包/K06共享cell或M2总替换。

## 1. 目标

G1：原K02 source不改，闭包环境真正跨creator frame返回并由native代码间接调用，结果42，creator退出不提前释放。G2：受检捕获/mode/env布局与真实callee匹配，move/clone/失败原子/最后owner及在途调用pin有完整清理。G3：K02/env-clone逐fault、923基面在本runtime版本下复验，独立父判关闭R05和完整R02。G4：单函数/lambda暖生产0、源绑定恢复与下一编辑、原资源/时间门和所有consumer完整。

非目标：任意闭包语言、Shared cell/K06写回、捕获List/Variant/Closure/Dyn/宽位型、借用逃逸、外部callback/public ABI/797拓展、native loader/HMR/OSR、Unit/脚本入口或A2R/A2C闭包。不能把新增有限native能力自动改旧M1的K01..K06 NA/其他分母；其重入按独立范围规则处理，M2后继由922盘点。

## 2. 架构方案

原auto.source.closures.draft→现ClosuresFunctionFrontend/CheckedFunction+ClosureBodyFacts→lower_function_closures_unit_checked→共享MIR验证→有限native closure admit/签名/env布局→原生函数COFF与窄runtime原语→PE执行→MN-EVT/闭包事实→ac_mn父判。

native新profile消费既有profile6语义，不修改AV wire/指令、不回到旧HIR→CLIF、不调用AV executor/旧VM、不在runtime存MIR/ABC解释。复用923的allocator/fault/Place与919生命周期设施、889对象/manifest/事务服务；窄closure backend模块允许新增，但不能复制全driver/cache/进程runner。

## 3. 技术栈

Rust/Cranelift、Windows x64 COFF/PE、显式内部Cell/env调用约定；现有no_std资源库及单panic handler、受控Job/工具执行、ManagedStore/auto-test-case。拟新native模块experimental/ace/crates/native/src/closures_lower.rs、closures_driver.rs、closures_incremental.rs（均新，T-01核冲突）；runtime可在现collections-runtime内分模块，具体布局由T-02证真链后冻结，不预写实现。

## 4. 需求分析与背景调查

授权：2026-11-03用户要求native剩余三格正式设计/规划，不授权本轮实现/构建/认证/跨仓写/新agent/chat。本包仅正式立约；main读取9c91a1b48，D915=5ba5d4cd2、D919=4ea3daddf已合入。921 executing且公共A2X/core接口有WIP；922已建实际lang-922树，头滞后不代改。

| 来源/实际符号 | 当前事实/内容SHA256 |
|---|---|
| experimental/ace/crates/native/src/collections_lower.rs::admit_layouts | f15eeefb029e8316d7b79fd446755a7954f94e322d0f323611e8fc8e71c6022a；仅profile4且缺Closure/三闭包指令 |
| experimental/ace/crates/native/src/collections_driver.rs::collection_interface_parts | a4fdd9dbb3926434f0039b521e5fe49f9504bfbd6abbb0dd02fe9424d55c40ba；import.env被拒；“collections closure”是依赖闭包，不是lambda支持 |
| docs/specs/auto-ac/source-closures.md、closures-function-production.md | 后者c99a4c8c70192048875acdb148cc5aac2ada22734e86604d4fca07d7537d27dd；865/912真源码/捕获事实/单体凭证/兄弟lambda复用已有，native未消费 |
| experimental/ace/crates/core/src/source/closures_fn.rs::ClosuresFunctionFrontend | 原source/lambda编辑、候选registry与body work真实生产点，不需要新捕获分析 |
| experimental/ace/crates/lowering/src/hir_adapter.rs::lower_function_closures_unit_checked | 已消费ClosureBodyFacts/env事实生成CreateClosure/CallClosure/EnvGet，不能native后端猜捕获 |
| experimental/ace/crates/av-core/src/mir.rs、auto-mir/closure-capture-profile.md | MirType::Closure与env签名/mode/site唯一、move/clone/borrow规则已有；native需独立真实ABI/执行 |
| docs/specs/auto-av/closure-environments.md及resource-fault-and-generation-conformance.md | AV env prepare/commit、clone失败及K02证据是语义参考，不是native结果 |
| crates/auto-lang/test/cases/ace-m1/src/K02.at | make(move s Str)返回move lambda fn(int) int；main真实f(37)结果42，原source/expected不可改 |
| docs/specs/auto-test-case/resource-lifecycle-observations.md | 93bf6ea1e0a565c8c0efb40452f7c8b54ba06b11e1249e06c6564609ca835aa9；native env事件/逃逸尚未覆盖 |
| PLAN-923 r1 | 规划中的runtime故障/Place基础，不当作已交付；其R03独立关闭，R02非闭包交本包完整收口 |

原R02 oracle明确包含L04/L05/V05/K02；若包只关集合故障即勾R02，会漏K02。故拆包后本包承担完整R02聚合/跨版本重验责任。共享方法/资源既有915/919归档不重开；新native有限profile不自动开放AV支持的全部闭包形态。

## 5. 详细设计

### 5.1 有限源码/typed支持矩阵

支持i32/bool按值Copy捕获、显式move的Str与非递归owned record（其字段沿现资源域）；lambda i32参数/i32结果；普通factory接move资源参数、返回Closure；Closure局部move/显式深clone、真实间接调用。零参i32普通函数为程序entry，闭包body不能entry。用于R02的typed CaptureKind::Clone/混合Move+Clone在现MIR验证规则内支持；源码无合法形式的clone capture要明示typed，不伪称新增源语法。

Copy捕获不隐式clone；owned move杀源仅在commit后；Closure.clone深复制env及owned backing，不能用同env多加ref冒深clone。EnvGet沿现语义可临时handle-share，env保有其责任，不能读取时偷move最后owner。非move Str借用、Shared/MoveFromEnv、捕获容器/闭包或未列模式在**native admission**具名unsupported；共享前端和AV仍按原合同支持，不以native有限域退化全语言。Borrow逃逸等非法输入仍原source/verify前门拒绝。

新增显式recipe拟ac/mir-native-closures-resource-v1、native profile/ABI/cache族独立于collections，T-01核冲突并冻结；不向现五profile静默猜测/回退。旧Cell/runtime若扩布局，按真实字段/调用约定bump身份并核全部consumer，不能只换字符串绕准入。

### 5.2 内部ABI、原生间接调用与调用前准入

闭包值携被调身份/签名及受控env handle；机器code pointer从本次已验证/链接image绑定，持久工件只存symbol/descriptor/deps，不存宿主地址。closure body调用约定显式env+out/result+原参数，参数/mode/return与env capture声明在单元/link准入时逐位匹配；具体物理形状T-02选等价最小方案，必须和代码生成/所有调用者同步，不冒797公共AutoABI。

CreateClosure通过真实callee/当前env descriptor准备；CallClosure由运行时值决定目标，Cranelift真间接机器调用（含CLIF/对象证据），不能把K02工厂常量折成返回42或把runtime做成指令解释器。至少两个同签名不同body/env的实例给不同真实结果，间接目标准入后正确分派。signature/nominal owner/env形状/foreign或stale handle在实际调用前拒，被调计数0；合法非零id/gen必须接受，不能全拒过门。直接Call lambda body、错entry、错env声明仍按现MIR/link合同拒。

本包image固定为当前PE，未实现native loader/HMR；“未改对象恒等”指编译会话resident MIR/COFF，不是跨PE热换机器码。creator退出与调用pin是在同一guest PE内的真实frame生命周期，不能用compiler cache lease替代env活性。

### 5.3 Env构造、clone、逃逸、清理与fault

env事务先为所有fallible clone/metadata/容量准备并验证，再一次commit moves和目标Closure；任一失败，源逻辑值/责任、旧Closure/代际仍有效，已准备部分逆序无分配回收。测试同时有Move/Clone字段且失败落在后一个字段，避免第一字段早move被错误掩盖。typed/source身份分开，禁止对不受检IR直接运行。

factory返回只移交返回Closure，frame sweep不能杀该env/capture。closure move源失效；clone独立env/backing，原与clone均可调用且互不共享析构权；同环境多个合法调用临时handle-share必须计活owner。调用进入后真实env pin保活，关闭slot/返回/异常/取消时按现责任释放，最后pin/owner后env/backing/node/text/arena/ledger全归基线；在进程退出前观察，不拿ExitProcess或清空registry等于析构。取消/fuel耗尽不能阻断cleanup，观察保持纯数据不延长寿命。

K02原source每seam1..N、N+1/无注入正控；typed env capture-clone/Closure.clone覆盖env metadata、backing/CloneWork及中段失败，与923相同完整seam定义。source PE终止/清理与同runtime原语持源/旧Closure的状态probe均需要；fail后同进程关闭fault重试，源可读/旧Closure仍可调用并得独立结果。新增closure allocation计入最终R02分母，不能只测923旧节点。

### 5.4 R02/R05的唯一最终关闭

本包正式测试含：
1. 原K02完整source→共享MIR→native→42，creator frame已退出而env尚活，最后Closure/call pin后精确释放（R05）。
2. K02/env-clone全fault与同进程原子修复（R02闭包部分）。
3. **在本包最终runtime/ABI/工具身份下**复跑923 L04/L05/V05/LSTR/typed深clone非闭包全seam/原子性/清理，与本包义务合并成完整R02。
4. 923 R03和919 R01/R04/R06回归，env新类型不能打破旧Place/事件/清理。

native-resource-scenario-coverage.md由923创建，本包在取得其正式稳定SHA后modify，记录R02全域/全consumer/每seam/源typed界和实际证据；不缓存PASS、不将旧版本N直接搬入。只在两门真实父证据全部过后回填R02/R05 ac，其余格/原source/expected/NA/accepted/原313守恒；旧三格全闭仍只说明旧基面，不签含C的新M1/M2/v0.6。

### 5.5 函数生产、依赖、恢复与资源验收

复用912 frontend/单体凭证与shared lower，接nativeCOFF/session；不调用AV执行/用ABC解释得到native结果。至少64顶层/70单元、每owner≥3兄弟lambda；lambda单body/owner body在原受支持拓扑内，未改单元真实parse/resolve/typecheck/plan/bind/HIR verify/shared lower/MIR admit/codegen/load及每函数工件IO0，AST/MIR/COFF实际对象/代际保持，whole尝试0。Code body与env layout/signature/site事实分离：普通callee只改body不让不读body的creator/caller重编；捕获mode/type/order/名义布局变更使真正创建者/消费者精确失效或按现策略拒绝且旧成功版可用。拓扑/签名/跨单元非快沿912/914规则具名分账，不全编后过滤samehash。

native持久闭包工件用有界MIR/COFF/声明与捕获上下文/源anchor/deps身份，restore按caller实际来源→有界读→深验→准入→link/run，P0/R>0，不反序列化Checked/PASS/活env/cell/code pointer。仅工件恢复codegen0不证明下一body可暖续接；T-02从新进程恢复→下一lambda编辑闭环，复用913的中立restore_context/凭证验证思想和既有core公开出口，不能导入AV执行会话/AV专有root平台。所需有界native上下文出口修复归T-08，不藏parse/lower于R、不推给921/922。坏源/owner/env/deps/工具/截断/超量好hash坏载荷拒；cold修复另列，修复后下一恢复真正P0，readonly写0。失败/取消/迟到候选保旧source/registry/manifest/PE/COFF；current/history及合法外pin最后释放后物理回收，不删他方哨兵。

guest8MiB、物理32MiB（包括env/token/generation/ledger）、compiler managed48MiB及原16/24/8分区、前端池16MiB和code current/history8MiB/32的更严既有约束合并计量，不能各报总cap后叠加。父/PE private增量128MiB，单源/metadata4MiB、每流1MiB、919完整MN-EVT/native8192行/AV16384门不放宽。先限额后物化，fallback/codec/descriptor表也计预算，资源拒绝先于变异。原3072层/256KiB AV参照、native4096/64KiB实际支持域与512修订条件不削减；不为新环境任设语言语法深度上限。

fresh/hit/restart/body-edit四态各≥30样本、原64/70负载/512不同修订、暖P90≤2000ms；实际准备/cold/runtime Rust构建/link/run/清理/计量分列，稳定Auto编辑rustc/cargo0。真实runtime源码/ABI变更允许低频准备重建并按实际deps失效；不每次编辑cargo、不依mtime假命中。不把K02微例或现AV性能收据冒native正式门。

### 5.6 923/921/922写面与交付窗口

N公共native/runtime/ABI/driver先923后924，924可先做私有closure模块/源fixture/reference/接口研究；公共集成只消费923已提交稳定SHA/字段，最终合入必须包含D923，923最终交付不反向等本包。若可提前按符号交接，记录source/ABI/schema/构造匹配/所有consumer/验证/释放，不提交他方WIP。

S=core/source/closures_fn/单体凭证/context、shared hir_adapter、av-core MIR契约；优先复用现出口。921主A2X/provider/store，本包不改其TargetDescriptor/证书/AXS/AXC层，公共core/ManagedStore缺出口与921逐符号短窗，不因“仅加字段”省construct/match/全部reader核查。AV915/919实现不归本包重写。

H=ac_mn/公共mod/ac_protocol、auto-test-case/SDK/fixed组/workspace manifest/lock/binding/source-manifest，全文件单写；schema/recipe新身份互拒，原消费者保持。共享战略/交接文件与922也按文件窗，922有树按在途，提供本包有限closure/三格证据/仍不支持K06等清单供其只读使用，不替其完成M2审计或改进度。无D921/D922整包前置，正式采样/认证错峰，不持H等编译。后续是否扩K06/容器捕获/通用FFI由M2真实应用需求另立，不在本包偷偷NA/延期本来承诺项。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/native-closures-source.md | 无native closure→有限K02源码/准入/增量/恢复/明确拒绝 | 不冒全闭包语言 | AC-01/02/05/06 |
| SD-02 | add | docs/specs/auto-mir/native-closures-resource-abi.md | MIR closure事实→内部native env/call/资源/版本/原子性 | 非797公开ABI | AC-01/02/03/04 |
| SD-03 | modify | docs/specs/auto-ac/source-closures.md | 不变公共语义，补新native消费/支持边界 | 不复制捕获分析 | AC-01/05 |
| SD-04 | modify | docs/specs/auto-mir/native-collections-variants.md, docs/specs/auto-test-case/resource-lifecycle-observations.md | 共用runtime/版本与env事件/原Place/退出兼容 | 原consumer完整 | AC-02/03/08 |
| SD-05 | modify | docs/specs/auto-test-case/native-resource-scenario-coverage.md | 923 R02非闭包→native完整R02/闭包域/每seam与最终owner | 新runtime下重新核齐 | AC-04/07 |
| SD-06 | modify | docs/specs/auto-test-case/ace-m1-scenario-oracles.md, docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-gap-audit.md | 原oracle不变，R02/R05 ac真实证据关闭/余账 | 不签扩大后M1 | AC-07 |

SD-05当前尚未存在，由923正式交付后消费；不是本轮写current Spec或依赖未提交草案。

## 6. 测试设计

现有完整职责组ace-native、ace-native-worker、ace-runtime、ace-source、ace-semantics、ace-lowering、ace-artifacts、ace-adapters、auto-test-case、infra-ace-m1、infra-fixed；必要既有consumer为case-ace-m1-native-collections-889、case-ace-m1-resource-915、case-ace-m1-resource-events-919、case-ace-m1-closures-865-source、case-ace-m1-closures-fn-912、case-ace-m1-closures-recovery-913、ace-av；跨Cell/ABI加case-ace-m1-typed-native-891。923拟组case-ace-m1-native-resource-923须交付后确认为现有组，本包拟新增case-ace-m1-native-closures-924由T-10注册。ace-a2x仅真实共享接口变更所需或复用身份未变证据，不因同路线无因叠全experimental。

长期新AutoDown拟crates/auto-lang/test/cases/ace-m1/native-closures-924/，原K02 source只引用原字节，手写独立reference/事件oracle不得从expected生成执行输入。新recipe/证书若有均与旧AV/旧M1/新C/Rust身份独立，本包父判以真实source/typed产物/工具/runtime绑定；不建第二runner。以下均未执行。

| Case ID | AC | 输入/真实入口与使能任务 | 独立oracle与区分性控制 | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/02/07 | 原K02，factory返回/creator已退出→native调用→drop，T-03..06/10 | 值42+creator leave前后env活性/最后释放；直接返回42/只活到frame/OS回收反控红 | 真实源，不只typed |
| V-02 | AC-02 | Copy/move Str/owned record、Closure.clone、混合Move/Clone，T-04/05 | 源责任与clone独立env/backing、各自可用；share冒clone/早move控制红 | 已列有限域 |
| V-03 | AC-03 | 同签名双目标/错env布局/名义owner/foreign/stale/sig，T-03/05 | 真call_indirect与不同结果；错配callee执行0，合法非零id可用 | 内部ABI，不做FFI |
| V-04 | AC-02/04 | K02/env clone每seam 1..N/N+1、失败保旧Closure/源，T-04/09 | 部分env回收、后字段失败不kill前字段、清fault同进程retry正确 | source终止+同runtime状态probe两腿 |
| V-05 | AC-04/08 | 923全部非闭包域在924最终库下重跑，T-09 | L04/L05/V05/LSTR/typed clone全seam和cleanup仍过，不能拼旧版本PASS | 完整R02组成部分 |
| V-06 | AC-05 | 原64/70、≥3兄弟lambda，单lambda/owner body编辑，T-07 | 未改真实P/IO0、AST/MIR/COFF恒等、whole/换对象反控红 | 原暖支持拓扑 |
| V-07 | AC-03/05 | body vs env mode/type/order/signature变化，T-03/07 | 真依赖精确失效/既有拒绝保旧；只hash调用点/漏env依赖反控红 | 不承诺任意状态迁移 |
| V-08 | AC-06 | 新进程源锚恢复→下一lambda edit，坏件/readonly/修复→再恢复，T-02/08 | P0/R>0、未改P0，坏来源/凭证/工具/超量拒，不序列化活env/地址 | native工件/context，不用AV执行 |
| V-09 | AC-07 | 首例/strict父判/完整R02+R05登记/坏身份/事件重放，T-10 | 恰两格可核、原313/NA/accepted不改、缺一domain不通过 | 不签含C的新M1 |
| V-10 | AC-08 | 原负载四态30样本/512修订/预算/小栈/取消，T-11 | 原cap与暖P90≤2000ms、稳定编辑实际Rust spawn0；先限额再物化 | cold/准备单列 |
| V-11 | AC-01/03 | 不支持Shared/Borrow逃逸/容器capture/直接call body/错entry，T-03/10 | native有限域明确拒、非法前门拒、AV已支持正例不退化 | 不借有限native禁全语言 |
| V-12 | AC-02/08 | 最后调用pin/错误退出与923 Place/919/旧profile消费者，T-04/06/11 | 退出前一次清理、最后pin回收、旧域/观察OFF不变、ABI不混 | 无native loader/HMR承诺 |

实施/自查/最终独立review在同一唯一树顺序进行，review冻结SHA暂停写、修复回work。无指定外部review模型/agent，不自行创建聊天或第二checkout。final review从真实code/产物/反控重建判定，不信工作勾选；merge按冻结diff跑明确完整职责/必要consumer并集，身份未变有效证据可复用。周期cargo t all仅due主检出单实例，正式采样与921/923错峰。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01/11原K02真实native源码闭环与有限域准入，不用AV/解释器/硬编码结果 |
| AC-02 | work | V-01/02/04/12捕获move/deep clone/creator逃逸/最后env及pin/失败清理全过 |
| AC-03 | work | V-03/07/11内部ABI/真实间接调用/错配调用0/合法近邻与源语义一致 |
| AC-04 | work | V-04/05闭包+923全部非闭包在最终runtime下逐seam/原子性/恢复合格，完整R02无漏 |
| AC-05 | work | V-06/07真实函数生产/IO0、对象代际/依赖失效，禁止冷fallback冒暖门 |
| AC-06 | work | V-08纯来源恢复/坏件修复再读/下一编辑/只读与P/R/M分账通过 |
| AC-07 | work | V-09严格父证据/原source/oracle/规则守恒，唯一回填R02/R05 ac并保新M1/M2余账 |
| AC-08 | work | V-05/10/12原资源/性能/样本/压力、stable Rust0及全部旧consumer/923接口兼容 |
| AC-09 | review | 独立冻结review逐AC/V/SD通过，必需恢复/clone/fault/增量不可记后继延期 |
| AC-10 | merge | 含D923正式主线交付、SD读回/archive、全输出对账/guard清树分支组scope与收据 |

## 8. 执行步骤

| T | owner_stage | 前置/真实写面 | 产出与验证入口 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 当前main/923/921/922 owner | 核915/919交付、923稳定接口、原K02/有限域/全部consumer与profile/预算/窗口 | AC-01..08 | [✅ 2026-11-03 work] 证据：D915=5ba5d4cd2/D919=4ea3daddf均`git branch --contains`在master✓；921/922已archive落地（fe0b1e45f/ec3e70414，§4起草事实滞后已核正，无在途窗口冲突）；923稳定接口在其reviewed头59ef759f7已提交：`mn.collections.runtime/3`+`cell/3`、S0..S8 seam/`mn_coll_fail_nth_seam`、`mn_coll_place_mint/get/set`(validate_place先于解引用)、`restore_collections_session`、recipe `ac/mir-native-resource-guards-v1`、MN-COLL-INJECT/MN-COLL-TRAP(21..27)/清理事实行、suite case-ace-m1-native-resource-923+组已注册；原K02=src/K02.at（make(move s Str)→move lambda，main f(37)→42）未改；闭包拒绝点四处实锚：emit.rs::admit_unit(profile门)、lower.rs:169-189/1819-1825、fingerprint.rs:332-336、collections_lower.rs:104-150(catch-all)/151-202；av.closures/1为AV-only现况（mir_verify.rs:470-475门）；预算常量实锚：GUEST_CAP 8MiB(CRT lib.rs:28)/MANAGED_CAP 32MiB(:29)/arena LIMIT 32MiB/PARTITION_CAPS 16/24/8(NAT resources.rs:25)/MANAGED_TOTAL 48MiB(:26)/MAX_INPUT 4MiB(config.rs:62)/STREAM 1MiB(:66)/EVT_CAP 8192(CRT:304)/AV 16384/PE private 128MiB(observe:251-259)/前端池16MiB(closures_fn.rs:475)/HISTORY 8MiB/32(closures_session.rs:104-105)/栈4096深64KiB证人(collections_resources_tier.rs:290)；五native recipe面与worker模型（mn-collections-worker承载新族recipe先例=resource-guards）确认。924树已建D:/autostack/.wt/lang-924/auto-lang@plan-924-dev基59ef759f7（923 reviewed冻结头，符合§5.6"公共集成只消费923已提交稳定SHA"；merge时D923须在master）；scope D:/autostack/plan-artifacts/plan-924/已登记 |
| T-02 | work | T-01，私有Coff/runtime probe | 最小原K02 creator退出真调用+深clone/fault+新进程恢复→下一lambda edit whole-path feasibility；冻结内部ABI/新身份 | AC-01/02/03/06 | [✅ 2026-11-03 work] 原K02真实native闭环已通（真实source→共享closures MIR单元→native admit/CLIF/COFF→PE link→真实执行exit 42；creator frame退出后env存活、真call_indirect分派、EnvGet handle-share、live=0/bytes=0/invalid=0清理；tier首测`closures_native_tier::k02_creator_exit_native_indirect_call_returns_42`绿）。probe产物：plan-artifacts/plan-924/t02-k02-mir-dump.txt（K02三单元MIR实dump：make CreateClosure Move Str→Import body；body EnvGet→View→StrLen→Bin→BorrowEnd；main Const/Call/CallClosure）与t02-dump_k02-probe.rs。**内部ABI冻结（D-01）**：closure值=Cell{bits:env节点地址,owned:1}，env=Node kind=CLOSURE 单块[S9]（header+[Cell;len]，type_fp[0]=code_ptr(本image重定位绑定)/[1]=sig_id(blake3 desc)/[2]=owner_id，disc=pins，epoch=accepted dispatches，moved=parked）；body机器签名`(env:i64, params:*Cell.., out:*Cell)`prologue pin/epilogue unpin/trap abort扫parked；CreateClosure两段式：`mn_coll_env_prepare(len,code,sig,owner)`(S9 staging,refs=0)→逐capture`stage_move`(retain共担,commit后generated code对Move源mn_coll_drop杀责)/`stage_clone`(clone_value深拷)→失败`mn_coll_env_discard`逆序无分配回滚→成功`env_commit`发布；CallClosure=`mn_coll_closure_target(bits,sig_id)`前置registry+签名准入(拒→MN-COLL-CLOSURE refusal=N,计数0,INVALID)后CLIF`call_indirect`真间接调用；EnvGet=`mn_coll_env_get(dst,bits,field)`retain share；Closure深clone=mn_coll_clone CLOSURE臂(clone_env两段式新env+逐字段独立owner)。**新身份冻结**：`mn.closures.cell/1`/`mn.closures.layout/1`/`mn.closures.codegen/1`/`mn.closures.source/1`/recipe `ac/mir-native-closures-resource-v1`；runtime ABI `mn.collections.runtime/3`→`/4`（env面+SEAM_ENVNODE=9,SEAM_COUNT=10；collections cell/3与923 seam S1..S8不变）；单元身份=spans-cleared ABC wire bytes（同AV wire纪律，fingerprint.rs scalar面不放宽，closures_compat测试不变红）。恢复→下一编辑可行性：923 restore_collections_session模式+913 AcceptedSourceState链已核可用（T-08实施）。深clone/fault臂待T-04/09 tier覆盖（env发射器已内建discard回滚） |
| T-03 | work | T-02，native admission/layout | 有限Closures profile/函数签名/env/entry/link准入与依赖边，V-03/07/11；cargo t ace-native ace-lowering | AC-01/03 | [✅ 2026-11-03 work] 证据：运行时级探针`runtime_closure_state_probe_rollback_admission_clone_and_park`（collections_resources_tier，真实PE同进程原语）：合法非零id连续两次dispatch返回绑定code（206/207）、错sig拒refusal=2、foreign bits拒refusal=1、finalize后stale bits拒、env_get越界INVALID、未pin先unpin INVALID；typed层具名拒绝4腿（closures_typed_tier：容器捕获type面含"List"、直接call闭包body import拒"closure value"）；源码前门K04借用逃逸=mn.unsupported.frontend、closure-body当entry拒"ordinary zero-argument int function"；link准入cross-unit签名/env声明比对在closures_driver::check_closures_interfaces（V-07依赖边见T-07负载测试：body/owner/topology三级精确失效+nonfast诚实整面）；`cargo t ace-native` PASS（217执行，gate_passed=true，results run-yb_sowtl） |
| T-04 | work | T-02/03，923释放runtime N窗 | env registry/prepare→commit/move/clone/最后pin/错误退出与有界事件，V-02/04/12；cargo t ace-runtime | AC-02/04 | [✅ 2026-11-03 work] 证据：两段式事务完整腿（探针）：后字段Clone注入失败(S1)→env_discard逆序回滚bytes==base、move/clone源均完好（204/205，失败不落第一字段）、清障同进程重试成功；深clone独立env（bytes增长+原closure死后clone仍dispatch、字段独立读取214-217）；pin保活（最后owner drop后live_bytes不变218）+最终unpin finalize（此后stale 220）；MN-EVT env生命周期（k02_env_lifecycle_events：own kind=closure在creator frame-leave之前创建、free≥2、live=0）；S9 seam扫描K02(1块)/K05(2块)含N+1正控+trap后清洁；**V-12真实缺陷修复**：mn_coll_abort陷阱路径漏调ac_rt_release_idle_metadata（body已跑StrLen再trap泄漏288B arena）→修复后trap_inside_closure_call_parks_env_and_aborts_clean全零基线（含arena_live=0）；同轮修复923探针结构性空转（seam-retry分支在默认体ExitProcess后为死代码→移至可达位+frame count 1→8修token清扫泄漏，测试由空转变真跑仍绿）；runtime ABI mn.collections.runtime/4（S9+SEAM_COUNT=10）全部调用者经ace-native组覆盖 |
| T-05 | work | T-03/04，native emitter | CreateClosure/EnvGet/真CallClosure机器发射及ABI所有调用者、双目标控制，V-01/03 | AC-01/03 | [✅ 2026-11-03 work] 证据：发射器三指令全腿（c99946f9b竖切+本轮）：CreateClosure两段式prepare/stage/discard_guard/commit+Move源post-commit drop；EnvGet仅env body发射（typed无env直接拒）；CallClosure经mn_coll_closure_target前置准入后Cranelift真call_indirect（env+params+out机器签名）；双目标控制dual_target 5+37与3*4=54两真间接分派；所有K01/K02/K03/K05、typed混合/record、负载72单元均经同一发射器；探针/typed/native三tier+ace-native组全绿 |
| T-06 | work | T-03..05，native driver/worker | 新recipe/源→sharedMIR→COFF/PE/受控资源执行、完整原K02，V-01/12；cargo t ace-native ace-native-worker | AC-01/02 | [✅ 2026-11-03 work] 证据：driver `request_closures/request_closures_resource/build_closures`全tier所经（MnRequest陷阱解码22/23/25/26具名）；worker `mn-collections-worker`承CLOSURES_RESOURCE_RECIPE（fail_seam 1..=9闭包域、observe常开、parent判stderr事实）与manifest持久化/发布回滚链；recipe ac/mir-native-closures-resource-v1独立于collections/events/guards；完整原K02真实42；ace-native组PASS（worker二进制随组构建，其cfg(test)分配控制腿在内）；ace-native-worker组归T-11自查并集复跑 |
| T-07 | work | T-03..06，912 frontend/native session | 单lambda/owner body计量/registry事务/实际依赖/不改单元恒等，V-06/07；cargo t ace-source ace-semantics ace-lowering ace-native | AC-05 | [✅ 2026-11-03 work] 证据：warm_session_formal_scale（65顶层/72单元，sum与main各≥3兄弟lambda，全部native域内）：cold=137真实exit+72单元codegen；hit零生产route="hit"；首/中/末lambda编辑各恰1单元重编（d___closure_main_0/1/2，值137→138→143→142手算模型）；owner body编辑恰d_main重编+三兄弟对象SHA256恒等（fast-owner）；owner增lambda=nonfast:owner-topology-changed、跨单元新增函数=nonfast诚实整面且结果正确（V-07三级精确失效+反控）；未改单元（d_h0/h33/h61/sum/factory）对象哈希恒等（V-06 COFF恒等；MIR经wire-diff不重lower）；K02微会话（3单元42→43→hit→44，hit修复：重跑发布后collections.exe非candidate临时件）；912前端组（ace-source/ace-semantics/ace-lowering）代码身份未变复用其923基线证据；正式30样本/512修订P90归T-11 |
| T-08 | work | T-02/07，native artifacts/context S窗 | 来源恢复/数据重验/下一lambda编辑/坏件修复再读/只读V-08；cargo t ace-artifacts ace-native | AC-06 | [✅ 2026-11-03 work] 证据：`restore_closures_session/restore_closures`（closures_driver）镜像collections验证链（manifest schema/profile=closures/ABI=cell/1/target/codegen_build、runtime内容身份、source/coordinates闭包、逐单元MIR wire verify_abc重准入+closures admit+CoffMeta全等、COFF hash/len/pages、entry符号）；`ClosuresNativeSession::from_restored`以913链重建retained前端、wire身份取自恢复字节、首跑route="restore"零codegen（P0）+cache_hit_coff=3/restored_coff_bytes_read>0（R>0）；tier两腿绿：恢复→下一lambda编辑恰d___closure_make_0重编42→43+hit零产；forged id拒、全文件字节翻转坏store在open/链条具名拒（mn.infra/admit/artifact面，绝无错结果）、readonly会话manifest_id=None且除零字节读门锁外store字节零写、恢复后good manifest再恢复+下一编辑43；工件仅MIR/COFF/声明，不序列化Checked/PASS/活env/code pointer；组门禁见§7 |
| T-09 | work | T-04..08，最终runtime | K02/env各seam＋923全非闭包域在最终版本重验，状态probe/修复retry与全R02覆盖V-04/05 | AC-04 | [✅ 2026-11-03 work] 证据：`k02_full_seam_sweep_every_injection_traps_clean_or_completes`（seam 1..=9×nth 1..=4 全笛卡尔：每个注入要么INJECTED(25)陷阱+全零清理（S4按923冻结分类学走frame-open空路径trap22+INJECT事实行），要么序号越界跑真42，无第三态）；K02/K05 S9 env节点1..N/N+1（tier+AutoDown双面）；923非闭包域在最终runtime/4（含abort idle-metadata修复）复验：`cargo t case-ace-m1-native-resource-923` PASS + ace-native组全绿（collections seam/place/resources/events/typed L04/L05/V05/LSTR tiers在内）；同进程状态probe（rollback/retry/admission/clone独立/pin-park-finalize，T-04）+ trap-in-pinned-call零基线（V-12）→ 完整R02闭包+非闭包双域在此最终库下闭合 |
| T-10 | work | T-06..09，H全文件窗 | AutoDown/strict父判/拟新组/完整R02与R05唯一回填，V-09/11；cargo t ace-adapters auto-test-case infra-ace-m1 infra-fixed与两新组 | AC-07 | [✅ 2026-11-03 work] 证据：套件crates/auto-lang/test/cases/ace-m1/native-closures-924（frozen K02/K05字节副本+S9/S1/S4故障腿+N+1正控，recipe ac/mir-native-closures-resource-v1经mn-collections-worker）；strict父判表ac_mn/native_closures（S1..S9分类学+R05 env生命周期事件形状：creator帧内own、creator退出后存活、终扫free、live=0，全部父侧从严解码，不信worker自述）；contract.rs注册recipe+mn.resource门+S9 seam域+mod.rs派发+trap-face manifest豁免；固定组case-ace-m1-native-closures-924注册并PASS 9/9；binding.tsv唯一回填R02/R05 scenario ac格gap→ok:P924-closures（corpus行/source/expected/313未动），source-manifest scenario_assets_924+plan_924_update+binding sha刷新；两父门（本套件+923套件在最终库复验PASS）先绿后回填✓；H组并集结果见§7 |
| T-11 | work | T-01..10，错峰 | V-10/12正式原负载/四态/512/小栈预算、全部职责自查与SD提案/余账/接口释放 | AC-08 | [✅ 2026-11-03 work] 证据：closures_native_924_performance（ace-performance档::performance::）：四态×30样本（fresh 72单元137/hit零产/restart发布=setup恢复链计时P0+下一暖编辑138/body-edit恰1单元）P90≤2000ms全过；512不同修订churn（每修订唯一常量、137+k手算、恰1单元、churn后hit零产）；稳定编辑rustc/cargo=0（暖apply无prepare计数器）；预算0→trap22清洁、K05深clone cancel→trap26清洁（closures recipe wrapper）；职责组：ace-native(217+)与ace-artifacts双组无竞争PASS、case-ace-m1-native-closures-924 PASS 9/9、case-ace-m1-native-resource-923复验PASS、ace-native-worker/ace-runtime/ace-adapters/auto-test-case/infra-ace-m1/infra-fixed并集见§7收据；ace-source/ace-semantics/ace-lowering代码身份未变复用923基线；SD提案=frontmatter supersedes/new_spec_components（SD-01..06归T-13 merge读回）；预算/样本不放宽，K02微例不冒正式门 |
| T-12 | review | work完成，冻结SHA | 独立逐AC/V/SD全链复审，必需门缺失不pass，修复回同树 | AC-09 | [✅ 2026-11-03 review] final pass@1b789cc28（plan-924-dev，基59ef759f7，7提交+6291/-629，树清洁）。AC-01..08全过：本轮冻结树重验closures族23测（154s含全seam扫描/负载/恢复）、双探针、AutoDown组9/9、infra-ace-m1 23/23（突变控制绿）；守恒核验131行/91+40/12列、仅R02/R05两ac格变动、corpus src字节一致（K02 be2e36e0…/K05 1e72b1ad…）；SD目标7路径全在、SD-01/02已在树预备且规则与实现一致（D-01冻结ABI/断言面读码核对）；V矩阵逐案走查无漏项、无workaround（claim守卫收紧为显式强化，突变测试证明无弱化）。复审局限：与实施同会话同上下文——判定从工件重建（重跑+diff+守恒+哈希），非采信执行者自述。merge前置：D923（plan-923-dev@59ef759f7已reviewed未合master）须先/同窗落主线；冻结职责组并集与统一回归归merge |
| T-13 | merge | review pass且D923在基面 | 漂移/冻结职责并集与实际合入、SD-01..06读回/archive，不签扩大后M1 | AC-10 | [✅ 2026-11-04 merge] D923先落（rebase映射5/5 range-diff全等价→ddd42ef2b，--ff-only）；本包rebase两段（923落地后+923归档docs提交后，923补丁自动跳过、代码blob恒等）：c99946f9b→a33d6dfca→(二段重放)…交付链至 **48c2d8066=delivery commit**（master tip，--ff-only无merge commit）。master漂移=925会话14提交+归档docs——ace crate无冲突，代码身份不变复用复审证据；重放树冻结并集：**ace-native PASS 223**（两次瞬时红均归环境并留证：3×Deadline/DeadlineCollect并发PE收集超时串行重放绿2.80/4.40s、typed_v10 P90 2210ms与closures性能档并行争抢单例——2测配对串行实证双绿151.1s+14.6s）+ artifacts40/adapters67/auto-test-case111/infra-ace-m1 23(pin修复后)/infra-fixed37/两套件组9+9全PASS。SD-01..06读回：7目标主线在册（2add+5modify+oracles pin刷新28678656c）。M1不签：claim守卫含cert轴，can_claim_m1=false冻结 |
| T-14 | merge | T-13 | 全输出核查/证据摘要折计划、guard后清树分支组与整个scope/刷新登记 | AC-10 | [✅ 2026-11-04 merge] 输出对账：本会话命令涉及目录=两worktree(target随树删)、plan-artifacts/plan-923+924(全删)、temp探针(即时清)；无第二证据目录/外置target/他盘散落。wt-guard clean→worktree/branch plan-924-dev/组目录移除；plan-artifacts/plan-924/整目录删除（证据已全折本文件，无具名hold）；登记表刷新。生产工件观察：本仓为语言/实验工具链，无运行中部署进程消费本次变更；桌面release二进制/gen bundle如由他处构建将滞后本次合入——已作显式观察项记入§9 merge收据 |

拟唯一树D:/autostack/.wt/lang-924/auto-lang、分支plan-924-dev，本轮未建。实施材料唯一D:/autostack/plan-artifacts/plan-924/，开工登记owner/用途/在途；native探针/源码/产物/日志/SDK副本/外置target入该scope，常规构建在专属树，不放attachments/项目reports或散落第二目录。merge全删，仅具名未解决hold可留；不建链接、不删他方产物。

## 9. 复审记录

2026-11-03 | stage:new | PLAN-924 r1 | outcome:pass（规划静态准入，非native closure实现/认证/最终review） | 0..10节、T14/AC10/V12/SD6定义/引用/路径锚点/固定组与新组前置已核；SD-05明确由923将来先add、本包modify。next:work T-01/私有T-02研究，N集成取923稳定接口且最终基面含D923；完整R02必须在本包最终runtime下重核923全域，不能拼历史PASS。原K02/有限域/源与typed、真call_indirect、env原子性/clone/最后pin、暖P0/来源恢复后下一编辑门齐。原批准/登记未改，未建924树/scope、无构建测试/实施/跨仓或921/922合同修改。

2026-11-03 | stage:new | PLAN-924 r1 | 起草：先核native明确拒env/profile6/closure三指令，决定与923分包；本包独占R05及全R02最后闭合，923无反向等待。有限新native ABI/机器执行不是AV复用结果；源码/typed、P/R/M和闭包支持/排除域明确。本轮只规划/正式取号，未实施/构建/认证/current Spec/原登记或921/922进度变更；静态准入另记，功能全待执行。

2026-11-03 | stage: work | PLAN-924 r1 | outcome: pass (work-owned T-01..T-11 complete; AC-09 review / AC-10 merge pending) | worktree: D:/autostack/.wt/lang-924/auto-lang | branch: plan-924-dev | base: 59ef759f7 | code: 1b789cc28 (chain c99946f9b→ec2094c48→f62a2da12→e0775add8→9c9f56bb8→895e38754→1b789cc28) | deps: 923 reviewed head pinned at base; D923 merge to master is merge-time (AC-10) | tasks: T-01..T-11 all ✅ with per-row evidence | gates: ace-native(217+,PASS×2 clean), ace-artifacts PASS, case-ace-m1-native-closures-924 PASS 9/9, case-ace-m1-native-resource-923 re-verified PASS, ace-native-worker 39/ace-runtime 84/ace-adapters 67/auto-test-case 111/infra-ace-m1 23/infra-fixed 37 all PASS; ace-performance leg green (四态30样本P90≤2000ms/512修订/预算取消拒配, 203s); ace-source/ace-semantics/ace-lowering unchanged-code evidence reuse | blockers: none (D-01 frozen ABI; D-02 923 stable heads; D-03 domain listed; D-04 restore via core public chain; D-05 no 921/922 contract touched) | notable findings: mn_coll_abort idle-metadata leak fixed; 923 seam-retry probe vacuousness fixed (debts recorded); M1 claim guard strengthened to cite the cert axis; hit-path published-exe fix | next: review (T-12, frozen SHA on request; unified regression owned by merge per PLAN-896)

stage: goal | plan_id: PLAN-924 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-924/auto-lang | commit: 1b789cc28 (worktree) / df2de24cf (main-checkout bookkeeping) | next: review

2026-11-03 | stage: review | review_scope: final | reviewer: 同实施会话/上下文（局限已声明；判定由冻结树工件重建：重跑+diff+守恒核验，非采信自述） | plan_id: PLAN-924 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-924/auto-lang | branch: plan-924-dev | reviewed_commit: 1b789cc28e82e82651f97ca78a32b0f800ff9d18 | base_commit: 59ef759f7 | dependency_revisions: 923 reviewed头钉基（D923主线落地为merge前置） | spec_inputs: SD-01..06目标7路径存在；SD-01/02树内预备稿与实现一致（native-closures-source.md/native-closures-resource-abi.md） | acceptance_results: AC-01..08 pass（closures族23测+双探针+AutoDown 9/9+infra-ace-m1 23/23本轮重验；perf/全组证据身份未变复用）；AC-09本review；AC-10归merge | pending_closeout_gates: D923先落主线；冻结职责组并集+统一回归（merge所有）；plan-artifacts scope清理（merge所有） | findings: 无阻断项；3条非阻断观察已记KNOWN-DEBT（abort元数据泄漏修复、923探针空转修复、claim守卫收紧；另基面既有警告归属原计划） | evidence: 见T-12行与本收据（全部为计划文件内摘要，无附件依赖） | next: merge

stage: goal | plan_id: PLAN-924 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-924/auto-lang | commit: 1b789cc28 (reviewed) / fa03d7e6f (bookkeeping) | next: merge

2026-11-04 | stage: merge | PLAN-924:r1 | prepared: reviewed 1b789cc28 + SD-01..06 delta（docs-only后裔42c4b0cbd链） | landed: master tip=48c2d8066（两段rebase、923补丁跳过、--ff-only；映射：c99946f9b→a33d6dfca,ec2094c48→f0c62edd2,f62a2da12→83d1b3371,e0775add8→d3701c3f2,9c9f56bb8→9bbcd53ce,895e38754→cc295d65c,1b789cc28→6384df84b,+a6435498e/28678656c/48c2d8066） | canonical_verified: auto-ac/native-closures-source.md(afdfae34…)、auto-mir/native-closures-resource-abi.md(cd1bd703…)主线在册+5 modify全落地+oracles pin 3ed6ca2a→6179f473 | merge_gate: 重放树冻结并集全PASS（ace-native 223/artifacts40/adapters67/auto-test-case111/infra-ace-m1 23/infra-fixed37/套件9+9；两起环境瞬时红留证：Deadline×3串行绿、perf争抽数据配对绿） | archived: docs/plans/archive/924-native-closures-source-and-resource-closure.md status=archived completion_kind=delivered | cleaned: wt-guard clean→树/分支/组目录移除；plan-artifacts/plan-924全删；登记刷新；无具名hold | stale_artifacts_observation: 无本仓运行中部署消费；外部release/bundle构建将滞后（非本计划门） | batch_regress: 未到期（last_covered=925、924非5倍数、同窗口48h内） | outcome: pass

stage: goal | plan_id: PLAN-924 | plan_revision: r1 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed, guard clean) | commit: delivery 48c2d8066 on master; archive 246d82787 | next: done

## 10. 待澄清事项

| D | owner / 有界关闭 |
|---|---|
| D-01 最小closure ABI | T-02已冻结（见T-02行：env=单块CLOSURE Node/S9、(env,params,out)机器签名、两段式prepare/stage/commit、closure_target前置准入+call_indirect、pin/parked语义）；T-04/08 tier证真后如需微调按真实字段bump身份 |
| D-02 923交付接口 | T-01核stable版本/实际SHA；私有研究可先，N集成取窗，最终D923先合且全R02按最终库重验 |
| D-03 额外捕获域 | 本包仅列明支持；未列明确native unsupported，K06/Shared/容器等归M2需求，不自动改旧NA |
| D-04 source恢复上下文 | T-02/08以core现公开凭证/restore数据出口接native；缺口有界修复，不借AV执行/先全编冒P0 |
| D-05 921/922交接 | T-01实际owner/consumer窗口；922有树视在途，本包不改其状态/审计报告合同，正式采样错峰 |
