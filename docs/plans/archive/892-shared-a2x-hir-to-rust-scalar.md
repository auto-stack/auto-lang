---
plan_id: PLAN-892
status: archived
feature_name: 共享A2X骨架与Checked HIR到Rust标量闭环（AX-A）
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-10-30
plan_revision: 1
execution_note: 用户2026-10-30授权开工（892具备启动条件，T-01全量+T-02私有部分先行；首例正式注册/后续H接线与888分窗，正式采样错峰）
supersedes_spec_components:
  - docs/specs/auto-test-case/architecture.md
  - docs/specs/auto-test-case/ace-m1-conformance.md
new_spec_components:
  - docs/specs/auto-a2x/project.md
  - docs/specs/auto-a2x/architecture.md
  - docs/specs/auto-a2x/incremental-and-cache.md
  - docs/specs/auto-a2r/project.md
  - docs/specs/auto-a2r/scalar-hir-backend.md
  - docs/specs/auto-test-case/a2r-hir-certification.md
touched_goals: [GOAL-002]
affects: [auto-a2x, auto-a2r, auto-hir, auto-test-case]
depends_on: [PLAN-745, PLAN-772, PLAN-783, PLAN-828, PLAN-852]
current_step: 6
total_steps: 11
---

# [PLAN-892] 共享A2X骨架＋HIR到Rust标量闭环（AX-A）

## 0. 变更摘要

正式认领[Design51](../design/51-shared-a2x-and-hir-to-rust.md)与[A2X战略](../design/strategy/shared-a2x-and-hir-to-rust-roadmap.md)的AX-A：真实core/scalar Auto源码→既有受检前端/Checked HIR→共享最小语义view/结构化prepare→Rust目标适配与打印→现场actual文件集→独立rustc认证/父侧oracle。公共准备、依赖/cache、来源/产物服务从首例进入真实Rust消费者，不先造空平台。

本轮只立项：所有任务、probe、编译运行及验收未执行；不建树、不施工、不修改889/891/888合同。AX-B record/text资源、AX-C集合/变体仍未编号，不预占号。

## 1. 目标

- G1：覆盖现有core-i32与scalar-i32两域的计算源码闭包：i32/bool、函数/递归/调用、局部与赋值、条件/loop/break/continue、各自合法循环、精确算术/短路与静态检查；保持域准入差异和原源oracle。
- G2：建立目标中立的受检事实view、结构化准备/转换工具、目标能力及产物最小合同；Rust只是首个实际provider，预留第二目标而不宣称多目标共享已实证。
- G3：首例文件化AutoDown/TestCase、独立oracle、fresh/hit/read-only/restart/body-edit与恢复重验；暖未改函数parse/recheck/HIR/prepare/target-lower/print/store-write生产=0、实际payload/代际保留。
- G4：日常真实emit+新金样/证书核查目标工具spawn0；显式认证对现场actual和独立reference真实编译运行，各自对父侧值/trap oracle；失败不发布、不把Rust panic/编译失败当语言负例。
- G5：共享S、私有P、公共H按真实owner分窗，预算/限额/完整依赖与原子发布从早期受检，不复制旧前端、AV执行器、MIR lowering或store。

非目标：record/text/char/方法/所有权资源扩面、集合/变体、闭包/HOF、Unit/脚本入口、console/host-import、wide typed或宽源码、泛型/meta新语法/新CTE能力、多模块/全stdlib、public AutoABI/FFI、native loader/HMR、第二目标实现、旧A2X默认切换/退役、全M1/M2/M4验收。未来语义查询接口空间不代表本包已支持meta/实例化；不支持输入须显式拒绝。

## 2. 架构方案

源码+显式source profile→ac-core source::compile_*→CheckedModule（或已保留环境中的CheckedFunction候选）→A2xSemanticView→每函数共同prepare/需求/依赖摘要→Rust provider的目标结构/Doc→函数片段+相对source map→受检manifest/文件集装配→snapshot判决；独立certify再用相同actual文件集+冻结wrapper/helper→真实rustc→目标程序，父侧判决。

共享core只消费已经验证的HIR与事实，不依赖auto-mir-lower、AV execute、Cranelift或旧AST/Database。现有单体环境/候选接口若不够，取S最小出口；不通过FunctionIncrementalSession运行AV来取得转译结果。目标无关prepare与Rust合法化/类型映射/打印分开，不增加万能目标AST或强制数十个emit_* hook。

Rust初期内部接口可用Result<i32/bool, AutoTrap>或语义等价形式：显式checked算术与失败传播，wrapper只编码真实函数返回值/失败及已授权trace；不是mn私有ABI，不使用旧a2r的i64 wrapper。精确函数/类型形式、helper/证书schema及新recipe字面量由T-02真链probe冻结，不改变本合同的结果与失败语义。

## 3. 技术栈

| 范围 | 已核入口与拟写集（新增路径标明） |
|---|---|
| 拟新增P-A2X独立workspace | experimental/auto-a2x/，拟core/共享、rust/目标provider、probe/driver三个角色；src中semantic_view/prepare/target/artifacts/cache/session/observe与Rust结构/打印/helper，确切crate划分T-02冻结。当前不存在，不向根产品workspace添加实验栈依赖 |
| S现有输入/单体前端 | experimental/ac-core/src/{hir,verify,lib}.rs，source/{mod,adapter,resolve,parser,typecheck}.rs；CheckedModule::{bundle,domain,required_capabilities}，CheckedFunction及context_revision，CheckedEnv、prepare_candidate_document/FnCandidate::{parts,receipt,commit}；body_work在lib.rs内联 |
| 中立store/进程服务 | experimental/ac-artifact-probe/src/{identity,codec,schema}.rs及store/{mod,gc}.rs；ManagedStore/PutRequest/SpaceReport；ac-core/src/link.rs的run_deadlined_captured及Job/有界捕获，原则只读复用 |
| H公共SDK与注册 | crates/auto-test-case/src/contract.rs；crates/auto-lang-core/src/test_cases/{mod,report,ace_certify}.rs和拟新增ac_a2r_hir.rs；crates/auto-lang/src/tests/ace_m1_tests.rs及必要bin/ace-certify显式路由，逐符号取窗 |
| 拟新增正式资产 | ace-m1旁独立a2r-hir-892目录下.case.ad/.suite.ad、逐源引用/新golden/wrapper/reference/cert；scripts/verify-a2r-hir-892.ps1；新workspace定向scalar_semantics/incremental_cache/contract_certification测试入口，T-02建立可跑形式 |
| 只读参照 | auto-lang-core/src/trans.rs/Sink与旧trans/rust*、a2r-std；原M1 src/oracle/binding及a2r证书，现有verify-ac-source-745.ps1/verify-ac-scalar-767.ps1冻结义务；probe/src/load256.rs提供原115函数负载 |

未来实施树D:/autostack/.wt/lang-892/auto-lang，分支plan-892-dev，一Plan一权威树；实施/内部自检/独立final review/修复同树顺序进行。仅语言仓，不涉及外仓Web/UI/Man代码。拟加文档/正式资产均在该树实施，计划簿记留主检出。

## 4. 需求分析与背景调查

授权：用户2026-10-30明确为AX-A正式立项，写清范围/验收及889/891/888写面；本轮只有规划，没有实施、建树、构建/测试、另开会话、代理或跨聊通信授权。

读取master=130caf7ff；取号时主线因并发文档交付推进至74469529d，后续读取0b8a84301。相关实现输入按下面hash绑定，非实施分支WIP。初读889 executing r2/5-of-10，T-06/07曾因真实共享runtime bitflip污染并发控制而重开；其当前review/repair/merge状态以正式收据刷新，未核到D889。888 executing r2/5-of-10，H治理与正式成本收口仍在途；891 drafting r2/0-of-10。不能把较新分支commit或文档pass当窗口释放。

已读AGENTS/Spec overview、auto-hir stage-contract/project、auto-ac source-core-i32/source-scalar-i32、auto-test-case架构/原a2r金样认证、Design40/43/51与A2X战略及三相邻计划的相关合同；实际核hir/verify/source candidate/前端profile、旧Trans、ace_certify与783 store预算/GC接口。正式开工重新核输入及源锁。

| 实际事实 | 本包处理 |
|---|---|
| core合法while、scalar拒while但支持条件for；操作/一元/逻辑面不同 | source profile显式选择，准入不因Rust能表达而扩大；前端阶段/code/span保持，核心/标量双向拒绝测试 |
| CheckedModule凭证私有构造，CheckedFunction只核一个body但context包含声明类型/签名 | 冷态真verify，暖态只复核改变body，完整context漂移拒绝或保守重验；不伪造checked标志或只比函数名 |
| CallBinding单独映射arg→param，源码eval顺序与参数序不同 | 共同prepare按源码序保存一次值再按binding落位，目标不重新猜顺序；intrinsic必须看已绑定DefId/授权，不按名字替换 |
| 旧RustTrans和旧certify以旧AST/i64 wrapper及严格ace.m1.cert/1为合同 | 新独立入口/新证书身份与i32/trap协议，旧decoder/recipe/golden保持；提取共用服务须H释放，不误复用旧证书 |
| 828负载源由probe/src/load256.rs::generate_revision产生，基线273575B/115函数 | 以原源字节冻结/文件化，不把probe executor作为转译依赖；独立闭式oracle只在父侧，不发给worker |
| ManagedStore已有required-data、SpaceReport、run_gc；不等已对A2X自动限额 | 新producer复用机制并给出本包quota/租约/写前检查，不另建数据库、不漏WAL/staging等实际总量 |

输入SHA256：Design51=072e33c696631778b1e90767ecc908506fa74eeabbefb4d6224b31ff84beff35；hir.rs=a2a7606dc8e4b1e0f0b5bf7f25ce3a4c314eabf2d0b4e1d50b5d368d3da70c5e；verify.rs=8a7768d1ab37c085a831aac02812703758788f9b83c4cf4c6675371d603cd84f；source/adapter.rs=905a345264aa41d3ca356879543dc01f057179ccb24597b75fa7a1b64ae7b6a6；ace_certify.rs=51fc68f4084a24003d6d3e1df373ff0b4c4c5edf53f7406749b9fad3c9fb99b9。

本次立项回链后Design51 SHA256=1653293f3bcd02843cee2356cbf1480d63a154f506613d3cf7e285805454b15d（仅AX-A编号/导航更新，设计语义未变）。终检主线0b8a84301时889已reviewed r2/7-of-10，merge-owned门/交付仍待；888 executing r2/5-of-10、891 drafting r2/0-of-10。该刷新不授予任何S/H窗口。

取号：独占.git/plan-allocation.lock下复核active/archive最大891，scripts/new-plan.sh分配892、号源892→893；.next-id原HEAD仍887、包含他方未提交累进，不夹带提交或回滚这些分配。未来开树前协调提交本合同/号源与Design51依据。

## 5. 详细设计

### 5.1 支持矩阵与共享/私有边界

**本包完整消费现有core/scalar两个profile的合法计算源码，而非只做B01/B09演示。** T-01按745/767冻结源/反例与现行Spec逐项列库存；手工HIR仅作坏图/顺序辅助，不代替原源。入口显式源码函数名、零参i32结果；函数内部i32/bool参数/返回，let/var/作用域、全路径return、前向/相互递归、named/positional调用、受支持循环/最近break/continue全部覆盖。新入口不接Unit、bool/wide typed entry。

| profile | 源运算/循环合同 | HIR域与边界 |
|---|---|---|
| core | add/mul/lt，signed literal；if/while/loop；通用取负/减除余/全逻辑等拒绝 | auto.hir.core.draft/1/core-i32-draft，真verify；未调用函数和未执行分支也检查 |
| scalar | sub/div/rem、比较/同类eq/ne、neg/not/短路、条件for/loop；while具名拒绝 | auto.hir.scalar.draft/1/scalar-i32-draft，真verify_scalar；不因扩enum放宽core |

Rust i32不提升到i64；add/sub/mul/neg越界、div/rem除零及MIN/-1与MIN%-1均具名trap，商向零截断/余数随被除数。checked guard或typed helper显式实现，release/debug不靠Rust默认溢出选项。Const中的边界值必须正确打印（含MIN）；普通常量算式依原profile在执行路径产生trap，不变成转译诊断或提升未执行分支。

共同core持有只读语义view、decl/local稳定ID、操作需求/求值顺序/短路计划、查询依赖和source-origin工具；Rust provider持有Result/primitive表示、Rust合法化/目标结构或Doc、关键字卫生/字面量/优先级与打印。标量helper只实现typed原语/授权trace，不接收AST/MIR/ABC执行指令。每目标各有上下文，禁止全局mutable target开关；无需先实现Python来完成本包，但只报告Rust消费者已验证。

trace只支持既有显式hir.test.trace.v1的mark_a/mark_b绑定、用户函数/局部名优先及已有返回语义；不是新增print/console。T-02核已交付intrinsic合同/实际轨迹格式再冻结目标helper；named实参标记按源序只执行一次，按bindings传参，不能发成目标参数序再执行原表达式。

### 5.2 S/P/H交接与并行

D745/D772/D783/D828/D852为已交付语义/设施前置；不新增D889/D891/U888整包硬依赖。授权未来执行后可先只读T-01及已释放P私有准备；T-02的首例正式注册需要P+H最小窗，必要时加S；没有H可推进私有probe但不能勾T-02完成。每阶段同时取得所需子窗，不持一窗等待另一窗。

| 写面 | 892允许动作与具体交接 |
|---|---|
| P新auto-a2x/core/rust/probe、独立测试/helper | 确认路径独占即可独立于889/891 N准备；只依赖正式主线HIR/受控服务，不指向其实施树或复制其WIP |
| S ac-core/source/{adapter,resolve,mod}、hir/verify/lib::body_work、schema | 889 T-05/06正在用/修单体及冷路径，892不得据旧交接或分支绿宣布释放。T-01列CheckedFunction/context/type与FnCandidate parts/receipt/commit实际所缺出口；先用现有接口，确需改动则接受最小符号及新受影响测试。不得为scalar顺带修record/List/ownership算法 |
| S/H link.rs受控进程helper或中立artifact store新公开接口 | 默认复用，不改公共行为。确需有界取消/原子发布接口扩展另交实际owner；不从测试worker提取另一套无界spawn，不改变783全局GC参数影响他方 |
| H contract.rs、test_cases/{mod,report,ace_certify}及必要显式certify CLI、拟新增ac_a2r_hir | 888模块/发现/报告，889正式collections身份，891拟typed通道可能交集。按文件/函数取最小recipe/身份/判决缝；ac_av/ac_mn/AV worker/native worker无具体必要保持只读。新增文件/append-only不等H无冲突 |
| H治理/正式采样 | 新case进当时正式清单；888已交付则叶模块/消费者affected并集，未交付沿现行正式入口并留后继映射。不给888冻结报告打补丁、不抢源码/配置/样本窗口，不默认加入B/C白名单；与889/888 formal错峰 |
| N native ABI/runtime、旧R转译/stdlib/金样与外仓 | 只读，不依赖891位型通道；892不写emit/layout/MN_ABI，旧Trans/AST/Database/a2r-std/旧golden不重构，不切产品默认，不改Web/UI/Man |

交接必须包含前owner/Plan revision、精确文件/符号、正式基面/delivery SHA、schema/profile/参数模式/pass/runtime/helper/协议/cache轴、消费者兼容/限制、已跑和pending门、接受/回交。若消费新阶段事实，须适用review/gate完成并已合入主线；分支commit/scoped绿不等正式交付。释放既有主线未变符号只需正式基面及停止并写，不虚构整项依赖。未经用户授权不跨聊请求窗口。

### 5.3 源码、生产凭证与目标产物

冷入口按显式source profile走相应既有compile/verify。后端门检查owner/HIR三重身份、decl/type/调用绑定和required capabilities一致；未知profile/坏引用/图环/初始化/非法类型必须受控拒绝，不从未验证Bundle绕过凭证。磁盘checked只是待验数据，恢复由原producer rebind/reverify，不序列化Rust指针或凭证布尔值。

source位置经单体prepare/目标结构保留：相对函数span与装配映射分离；多次变长编辑原子平移声明坐标，不为位置变化重新打印所有函数。目标名称从稳定decl/local ID按函数局部生成，Rust保留字/用户main/runtime helper同名不得劫持；不同声明或scope重名不冲突，Rust诊断与产物span可回原.at。无固定语言嵌套数排除，新增walk/print/drop/类型/调用图路径受预算并避免宿主递归栈溢出/指数展开。

拟发射完整文件集含生成函数、最小typed runtime和显式entry wrapper及manifest，确切单/多文件形式T-02冻结。新TargetProfile至少绑定语言Rust/edition2021/host Windows x64、source/HIR身份、语义/pass/后端版本及capabilities；目标不是跨平台已认证承诺。函数片段源序/稳定装配，import/helper从真实需求取传递闭包；不携带整个旧a2r-std。

golden对应确定性的语义文件集及规范manifest（相对路径、稳定名称/内容hash、明确文件清单）；请求nonce、时间、scratch绝对路径、物理store ID及运行日志放独立receipt，不混入生成源码或golden。source map用逻辑source身份/相对origin并随当前坐标装配，换worktree/restart不让相同源输出随机漂移。产物路径拒绝绝对路径/父目录逃逸与重复文件，unexpected/missing文件都不能PASS。

值结果由真正生成函数返回，再由wrapper编码i32十进制；value70仍是value。trap由真正guard/helper返回可区分overflow/div-by-zero原因，不拿Rust panic、exit70或任意非零推导语言成功。进程/协议/工具/超时错误分列。trace及结果通道互不污染，输出/状态/完整文件集manifest由父侧核验；worker永不接收expected、oracle或PASS。

### 5.4 cache、函数增量与候选事务

复用783 ManagedStore/CAS，A2X注册独立producer/schema/逻辑层，不另建数据库；精确层名在T-02核Design40目录并冻结，不预占通用层号。按函数准备、目标片段、装配manifest分层，target-independent结果不读任何target配置；读到TargetProfile的pass结果必须进target-dependent key。

key至少包含owner/decl、逻辑profile/schema、完整类型/签名/模式、函数语义body、实际读事实/授权及pass版本，目标层再加Rust edition/target/表示/后端/helper合同与内容、格式配置。源码绝对span/snapshot、表重排/重复primitive等价项不改变语义fp；内容命中不等只比调用点源码。未支持CTE/meta/实例需求具名拒绝而非缓存未知事实；以后接查询需把完整实际传递依赖纳入。证书/目标编译另绑定全actual文件集、工具链/deps，不与函数片段键混用。

冷初始化允许整源，暖纯body编辑只parse/recheck/emit改变声明，保留已验证环境和其他body；独立source→CheckedFunction→prepare→Rust片段，不能先整源compile或全模块clone/print再carry。真实producer按function计parse/recheck/HIR plan/bind/verify、shared prepare、target lower/print/fp/persist的次数/字节与whole fallback；未改函数生产/工件写入均0，resident token/Arc+mint generation与实际消费fragment ID/hash保持。源schema必要的便宜接口核验、文件装配暂存/写出、冷恢复re-admit单列，不把普通输出copy当resident替换或声称全链零拷贝。

同签名helper body编辑只重产helper片段，caller复用，装配真正包含新helper；不以缓存PASS代执行新语义认证。A→B→A内容回访复用受检历史payload，不能再mint同hash假复用。接口/类型/context变更至少有正确、具名的拒绝或显式整上下文再验，精确记录保守边界；不持有过期CheckedFunction继续发射，不宣称首包已实现所有接口最小失效。

prepare不修改current；前端/准备/目标合法化/完整文件集校验/必要原子发布成功后才一次提交source revision、span、居民/历史索引。迟到/foreign/已close候选、取消、写盘/manifest/helper缺失或资格不符均保旧文件集/居民/当前源与可用性；恢复再核required-data、目标/内容/schema/依赖/来源，损坏拒绝或受控重算，重建后下一restart必须真hit。fresh不读旧artifact，read-only miss真实重算但不写store，reuse命中仍实际emit装配或消费受验片段、重读oracle，不缓存判决。

### 5.5 AutoDown、金样和独立认证

新增明确选择的recipe/profile（拟ac-a2r/hir-scalar-emit-v1）、worker协议和证书schema（拟ace.a2r-hir.cert/1）；准确名称/decoder字段在T-02冻结，旧a2r/emit、ace.m1.cert/1和i64 wrapper接受/拒绝集不改。源码/源profile/entry/capabilities/预算/hash/nonce是请求输入；expected只在父侧oracle。首例在T-02有正式可运行.case.ad/.suite.ad，不等T-06才补文件化。

**日常emit/snapshot**：生产者现场生成或从受验片段装配actual全文件集→独立已审阅新golden逐字节比较→严格证书/身份核查→父侧判决。rustc/cargo/目标Rust程序spawn=0，包括miss/stale/修复负例路径；缺证书/金样不隐式认证、不当PASS。维护构建另记，不能在日常报告中隐身；首次consumer前预构建工具。golden不是actual来源，更新须独立审阅并重认证，不提供自动overwrite expected凑绿。

新日常通道要求显式可用的prebuilt worker；缺失或身份不符时受控infra，不在case计时路径自动cargo build。工具维护构建是独立可观察操作，不借旧adapter的惰性构建惯例绕过spawn0门。

**显式certify**：消费同一生产入口现场actual，先核金样/manifest相符，再以byte-identical文件集+冻结i32/trap wrapper实际rustc编译运行；独立手写reference同样编译运行，两者各自等于手算/闭式oracle。值、具名trap、授权trace分别观察，覆盖debug/release关键算术对照；正常值70与trap区别永久验证。静态非法源在前端诊断轴验证，code emit=0，不用“Rust编译拒绝”证明Auto检查。

证书严格精确schema/键集与case/profile/entry/verdict/oracle、source/actual/golden全文件集、wrapper/helper/reference、shared/backend/协议/能力版本及内容、实际rustc -vV/target/flags与std依赖闭包相互绑定。正例全轴符合才复用；自洽错误的case/owner/oracle、缺字段/未知字段/错误观测是REJECTED，不以force覆盖坏证书。真实文件/配置/工具链轴变化为STALE，显式认证后再用；日常工具当前性从锁定配置/内容凭证及现有build身份核查，不能调用rustc探测，不确定即不授信。T-02明确默认/显式rustc来源及其实际可核依赖边界。

独立新golden/reference/cert存新目录，不修改旧M1 binding行/expected/NA/accepted-diff或旧golden。新ACE-A2R证据并列记录；以后是否替换M1 a2r消费者列由专门迁移/审计认领，892不宣称M1或旧A2X替换完成。

### 5.6 资源与周转冻结

| 预算/观测 | 本包边界 |
|---|---|
| A2X管理工作集 | ≤48MiB；T-02按实际同存source/checked事实/prepare/target片段/编码/IO分区，所有新容器/拼接分配前核限。对未来fixture不能只报峰值而无拒绝点 |
| 输入/协议/产物 | 源≤4MiB、IPC元数据≤4MiB；单目标文件/产物≤64MiB、单check实际文件集≤128MiB；stdout/stderr各≤1MiB，达到caller更严cap即按更严；参数算术/长度/路径/UTF8先校验 |
| Store配额 | 本包自有测试/项目命名空间总占用≤128MiB（块+index/DB/WAL/SHM+staging/trash/log）；current/历史lease最小集合受保护，quota内GC不能回收活依赖；不足回收则写前具名resource保旧，不修改他方根/GC策略。用户交付文件/证书不归GC |
| 显式认证 | 沿852每例compile≤120s、run≤30s；父侧完整流程（含actual生成、两个编译/运行、读写/kill/reap）绝对总墙≤360s，无额外宽限，caller更严从其更严；工具身份探测/输出同样受界 |
| 受控进程与磁盘 | CREATE_SUSPENDED/Job、并发有界stdout/stderr drain、写入/等待/回收统一deadline；不得以父调用已返回当杀树完成。自有认证scratch单会话≤512MiB，清理全出口；工具链物理peak/库文件单列，现有物理硬门有出处则继承，未知只报实测限制，不混成48MiB emitter已全树受控 |
| 日常正式周转 | 原load256基线273575B/115函数不缩：fresh/hit/restart各30正式样本、body-edit至少30不同真实修订，warmup/工具构建单列；每状态从正式请求/编辑入口到actual装配/父侧snapshot判决/清理，nearest-rank P50/P90/P95/P99，P90≤2000ms。认证时间独立列，不冒称完整rustc编译运行2s |

大文件装配/哈希/写盘须流式或沿既有分页机制（首期页≤256KiB），不能因单产物cap64MiB就在48MiB管理预算外申请整文件buffer；必要中间对象计入同存峰值，不能只算最终字符串。配额不足的清理/拒绝均保留活lease/当前交付物，不删除其他会话target或共享库。

正式样本的每个不同修订应先通过显式独立认证并冻结对应golden/证书，计时日常路径不能临时编译新修订补证。准备成本与认证总量如实另报，不用缩样本/更换小源/整源重算更快/删慢样本通过；历史回访至少有结构/正确性控制并单独记录。资源边界合法贴界与超限/故障、深层合法/未闭合/坏HIR、caller更严cap都必须区分语言诊断、resource和infra。

### 规范增量

| delta_id | add/modify/retire | docs/specs目标 | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-a2x/project.md；architecture.md | 无A2X新共享实现→受检语义view/结构准备/目标能力/依赖方向、首个Rust消费者与排除域 | 不误认旧Trans就是新骨架或已有多目标实证 | AC-01/07 |
| SD-02 | add | docs/specs/auto-a2r/project.md；scalar-hir-backend.md | 无Checked HIR Rust provider→core/scalar支持矩阵、i32/short-circuit/调用/trace/trap与目标合法化 | Rust默认/旧i64语义不能覆盖Auto profile | AC-02/03 |
| SD-03 | add | docs/specs/auto-a2x/incremental-and-cache.md | 无函数级A2X producer→完整key/依赖、居民/历史/恢复、原子文件集、quota/close及生产计量 | 目标不同不误命中、末端carry不冒零生产 | AC-04/05/06 |
| SD-04 | add | docs/specs/auto-test-case/a2r-hir-certification.md | 旧a2r独立认证仅旧profile→新增ACE-HIR日常spawn0/现场actual独立i32及trap认证、严格新证书身份与预算 | 双轴保持，坏工具/协议不冒语言结果 | AC-03/05/06 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有adapter/报告→追加显式新recipe、父侧判决/current身份/stale/发现模块消费者；旧合同不变 | H接线和治理可复核 | AC-03/07 |
| SD-06 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 原M1 a2r证据→并列记录892新producer/独立原源证据与未迁移边界，不改旧分母/expected/规则 | 新后端能力不冒认旧路径交付或M1全绿 | AC-07/10 |

## 6. 测试设计

下表全部是待执行合同。T-02使第一例正式可跑，T-03..06扩完全部mandatory cases。实施/自检与独立final review用同一权威树顺序进行；未指定模型或代理，不创建其他聊天，自检不替独立review。

| Case ID | AC / invariant | 代表输入/状态 | 真实入口与独立oracle/反控 | work owner / coverage |
|---|---|---|---|---|
| V-01 | AC-01/02 原源正例 | 原B01=14、B04=42、B09=120；745/767合法语法与操作库存、named/positional、前向/相互递归 | source→Checked HIR→新Rust provider→现场actual→真实rustc认证，各自等父oracle；原源不改字节，T-01全支持闭包库存不得只跑三源 | T-01/02/03/06；core/scalar，非全语言 |
| V-02 | AC-02 精确数值 | B13 overflow、B14 div-by-zero、MIN neg/MIN/-1/MIN%-1、-7/3=-2与-7%3=-1、边界值70/-1/MIN；常量算式与不执行分支 | 实际Rust debug/release关键对照，i32值与具名trap正确；value70是值而非trap；panic/编译拒绝必须不能满足语言负例 | T-02/03/05/06；无wide算术 |
| V-03 | AC-01/02 准入 | core含scalar-only op、scalar while/core条件for、未知profile/错HIR身份、非零参/非i32入口；坏图引用/环/未初始化/未执行RHS未知名或错类型 | 实际source/verify/target门阶段/code/span与独立合法兄弟对照；非法在emit前拒、无Rustcodegen/认证进程；不能靠Rust报错或扩大源域通过 | T-01/03/06 |
| V-04 | AC-02 顺序/短路 | false && trap()/true || trap()；named参数源序与参数序相反、mark_a/b授权与用户同名遮蔽；嵌循环break/continue/条件for重判 | 独立trace/结果证明左→右各一次、RHS未执行、最近loop目标；无授权/签名错拒绝；辅助HIR与真实source分账，不假借新console | T-02/03/06 |
| V-05 | AC-04 暖增量 | load256 fn_037连续变长/30不同body修订、同签名helper改body、A→B→A | 实际parse/recheck/HIR/prepare/target-lower/print/fp/persist逐unit工作；未改各0/whole0，实际resident token/generation/片段消费ID保持；helper新值对新独立golden，历史payload真reuse | T-04/06；新A2X单体消费，不运行AV取代码 |
| V-06 | AC-04 结构敏感性 | 真强制whole frontend/prepare/print、替换resident但伪保generation、跳改变helper/只改计数不改消费片段 | 对实际被消费通道的受控变异必须使对应结构/值门红，且合法有界装配copy不误红；记录mutation实际发生，单纯hash一致/更快不通过 | T-04/06 |
| V-07 | AC-04/05 cache与身份 | fresh/hit/新worker restart/read-only；schema/profile/owner/签名/模式/target/edition/pass/helper/授权漂移；错required-data/hash；表重排/span合法控制 | 恢复再准入、错轴在消费前拒或重算；修复后下一restart真hit，read-only写0；等价表/span不重产无关片段，context改不消费旧CheckedFunction；不支持meta输入受控拒绝 | T-03/04/06；类型/CT全能力不冒领 |
| V-08 | AC-03 日常双轴 | 已认证正例/不同修订、missing/stale cert、raw/golden漂移、wrong-oracle/trace、额外成功工具spawn故障 | 真emit/装配actual与新golden逐字节、父判；日常rustc/cargo/目标程序创建0，miss/stale不隐式认证；expected改1格只红该格，SKIP/零选择不PASS | T-02/05/06 |
| V-09 | AC-03 严格认证 | 同现场actual/reference独立编译运行、错case/profile/oracle/观测/未知字段/缺hash、改wrapper/helper/flags/deps/实际rustc来源、旧i64证书冒充 | 新证书真实绑定全文件集/工具身份/预算；坏结构REJECTED不可force覆写，身份漂移STALE且仅显式重认证；旧recipe/decoder/证书与新通道互不误授信 | T-02/05/06 |
| V-10 | AC-05 候选与发布 | 当前有效文件集上prepare后source变更/候选foreign/close/迟到、cancel、helper缺失、写满/重命名/manifest/GC lease冲突注入 | 受控prepare/真实文件集publish；失败旧source/revision/span/resident/文件hash不变、旧合法snapshot可用，成功新manifest引用完整实际片段，无部分成功/孤儿累积 | T-04/06 |
| V-11 | AC-06 资源/可信输入 | 4MiB等输入/IPC/产物/管理工作集/cache/scratch边界、深合法/未闭合/共享DAG与坏图、进程no-read/hang/大stderr、取消 | 分配前受控拒绝、非panic/宿主栈溢出；正确resource/infra且不是语言负例；deadline内杀owned树并回收reader/writer、全出口close/GC总账/目录干净；合法边界兄弟通过 | T-02/04/05/06；caller更严cap保留 |
| V-12 | AC-06 正式周转 | 原273575B/115函数，fresh/hit/restart各30样本、body至少30不同修订；预认证新golden/cert，warmup单列 | 实际入口→snapshot父判/清理各状态P90≤2000ms；完整raw/P50/P95/P99和结构/资源同时绿，失败/重测保留；认证单列，不用小源/缓存PASS/删慢样本替代 | T-06；与888/889正式计量错峰 |
| V-13 | AC-01/02/07 共享边界/来源 | Rust保留字或helper/user同名、作用域重名、多目标配置独立、错误span；依赖方向树及旧消费者对照 | 稳定ID/名称不碰撞，源错误与目标诊断可定位原.at/当前变长坐标；新core不含旧AST/Database/AV execute/MIR/native依赖，旧a2r/native语义/报告不变；第二目标未实现明确保留 | T-01/03/06 |
| V-14 | AC-07 接线/治理 | 新recipe、contract/dispatcher/worker/current身份/报告stale；新case/单项重放/发现源锁/受影响消费者；889/891/888释放接受 | 硬身份轴同步、变相关代码/input/helper使报告STALE或重验，不污染旧族；新义务被当前正式清单发现，若888已交付进其模块并集；无窗口不得接线、不默纳B/C | T-01/02/05/06 |

建议未来定向入口：cargo test --manifest-path experimental/auto-a2x/Cargo.toml -p <T-02真实冻结包名> --test <scalar_semantics|incremental_cache|contract_certification>；powershell -File scripts/verify-a2r-hir-892.ps1 -Scope <source|snapshot|certify|incremental|resources|formal|acceptance>（各选一个值，不按当前已存在命令宣传）。旧745/767仅抽实际受影响命名cases，开发/review不跑其内嵌整档或裸t/tt/tv/tf；脚本未知/空选择必须非零。

## 7. 验收标准

| ID | owner_stage | 可观测关闭条件 |
|---|---|---|
| AC-01 | work | 独立新A2X共享骨架/目标能力和Rust provider真接Checked输入；两源域支持/拒绝完整库存与可信门，依赖方向/来源明确；V-01/03/13 |
| AC-02 | work | core/scalar合法计算源闭包真实生成Rust、精确i32/调用/短路/trace/循环/静态语义独立认证；debug/release及关键反控齐全；V-01..04/13 |
| AC-03 | work | 首例正式AutoDown、现场actual与新金样、严格新证书/父oracle；日常工具spawn0，独立真实编译运行值/trap/trace，旧cert互拒；V-08/09 |
| AC-04 | work | 真实单函数前端/共享prepare/Rust生产、居民/历史复用、完整key/依赖/恢复再验及正确context处理；V-05..07，不以末端carry冒零工作 |
| AC-05 | work | 原子候选/完整文件集、失效/缺依赖/迟到失败保旧、只读恢复/持久修复、quota/lease/close成立；V-07/10/11 |
| AC-06 | work | 48MiB管理工作集、完整store/scratch配额/流/墙预算与杀树/回收，原规模/样本日常P90≤2000ms、认证独立；V-11/12，工具peak/限制如实 |
| AC-07 | work | S/P/H及必要中立服务逐符号释放接受、889/891/888兼容/治理源锁与报告stale/消费者门齐全；V-13/14，SD候选、不改旧golden/分母或产品默认 |
| AC-08 | review | 独立对冻结revision/commit逐项AC/V/SD和实际数据流审查、遗漏/延期/workaround/冗余/warnings/fmt；已知合同内缺口0，merge门显式pending |
| AC-09 | merge | 最终review后冻结commit执行当时有效required gates及batch到期判定：888未交付沿现行裸t+compiler tv/trans tt等真实触面，已交付则其正式affected完整模块/消费者并集；不提前套未交付WIP门或重复开发整档 |
| AC-10 | merge | SD-01..06、模块导航/overview/plans/ADR及索引/ledger沉淀、真实归档交付/guard清理、向AX-B释放稳定合同；不宣布全M1/全A2X或第二目标完成 |

## 8. 执行步骤

| ID | owner_stage | 依赖/窗口 | 具体产出与验证关闭 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 用户后续指定执行；只读可先 | 核权威树/HEAD/D门、745/767语法/操作/拒绝全库存与source/hash/oracle/profile；当前S/P/H owner、FnCandidate/context出口/预算/旧cert复用点，逐符号接受/回交及V映射；不把初读状态当释放 | AC-01/04/07 | [x] 已完成（work r1，2026-10-30）：基面=master `5243568ae`（889交付`50cd6f777`已入主线，树/分支已清）；D门745/772/783/828/852全归档；888占用实测=`test_cases/mod.rs`(+24)+新`inventory*.rs`/`ac_av/*`/`execution_trace.rs`+`docs/specs/auto-test-case/*`（即SD-05/06目标）+`scripts/test-governance/*`（共161文件）→H首例注册/SD沉淀/正式采样分窗确认；890=`vm/ffi/http_*`与892写面零重叠；891无树。S出口全在：compile_source/compile_scalar_source、CheckedModule{bundle,domain,required_capabilities}、CheckedFunction+context_revision、resolve_env/CheckedEnv、emit_function_body、prepare_function_candidate、FnCandidate{parts,receipt,validate,commit,cancel}；预算/受控进程=link.rs run_deadlined_captured、ManagedStore/PutRequest/SpaceReport/put_artifact/get_artifact_bounded；hash重锁：hir.rs`a2a7606d`不变、ace_certify.rs`51fc68f4`不变、verify.rs→`5e445b6b`、source/adapter.rs→`4388e03a`（889交付演进）、Design51→`ded55ac9`（CRLF→LF规范化）；HIR事实冻结：自由函数def名`sf_`前缀、bool字面量=`0<1`/`0<0`比较、mark_a/b零参返回i32（值1/2、stderr a/b、`hir.test.trace.v1`授权、用户名优先不劫持）；load256在`experimental/ac-av-probe/probe/src/`（§3记载修正）；745/767冻结库存=`experimental/ac-core/fixtures/source/{valid,invalid}`+ace-m1 `src/B*.at`+binding.tsv |
| T-02 | work | T-01；P+首例H最小窗，必要S | 最小独立workspace/semantic view/provider、正式第一case与recipe路由；真source→HIR→Rust→实际认证跑B01、短路/参数顺序和B13/B14；单body凭证小probe，冻结模块/输出/结果/trap/trace/新证书/受控工具/分区与quota真实实现机制。私有probe先行可但不替首例H/全链关闭 | AC-01/02/03/06/07 | [x] 已完成（r1-r9，末次commit `40113b4e0`）：H窗随888归档交付释放（其树/分支已清、archive在案），master合入plan-892-dev零冲突（14测试目标保持绿）。首例正式注册=recipe`ac-a2r/hir-scalar-emit-v1`进REGISTERED_PROBE_RECIPES+ProbeA2xHirSpec绑定（recipe专属词表/期望面，其他probe recipe装载即拒artifacts）+新adapter`ac_a2r_hir.rs`（只spawn预构建a2x-worker、身份回显核验、父侧manifest重哈希、golden逐字节+多余产物拒）+首例资产`test/cases/a2r-hir-892/`（B01字节复制+真实worker冻结四文件golden跨nonce确定性复验+case.ad/suite.ad）+`a2r_hir_892_tests`3/3（首例绿/篡改golden红/缺worker=infra带维护命令）。worker=`a2xworker/1`协议（identity/daily-emit零rustc spawn/certify显式腿=实际+独立reference+闭式oracle）。回归=ace-m1 14/14绿（888面无扰动）、auto-test-case绿。前置各轮：真链58用例/单body凭证/证书schema冻结草案/store会话/quota/预算实测 || T-03 | work | T-02；P及需要的S同时释放 | 实现共享prepare/walk/依赖摘要、TargetProfile准入、Rust完整core/scalar结构/typed helper/卫生与source map、真检查/拒绝；V-01..04/13/相关资源边界，旧域兼容；无第二前端/解释器旁路 | AC-01/02/07 | [x] 已完成（r2-r4+r9-r11，末次`60378d608`）：prepare/FunctionPlan+语义指纹（r2/r3）；完整core/scalar结构过全量冻结语料58用例真链rustc（r2）；source map v1（origin span→注释+manifest，r2）；卫生实证（c06/d07/p19，r2）；深度预算具名拒绝（r2）；V-13依赖方向boundaries 3/3（r4）；正式suite化=a2r-hir-892五例注册（r9-r10）；边界反控=text域/entry形状/too-deep/篡改golden红（r2/r9） || T-04 | work | T-03；P、单体S/必要store接口窗 | 居民Checked环境/单unit更新、规范key与分层store、完整文件集/事务/lease/GC/48MiB+quota、fresh/hit/read-only/restart/history/close；V-05..07/10/11与真实反控，失败回滚不先提交环境 | AC-04/05/06 | [x] 已完成（r3-r4+r11，末次`60378d608`）：a2x-store分层cache（AXS1/AXR1/AXM1）+A2xSession（源名居民复用/ProducerStats/quota门/lease+GC/close）；48MiB实测（暖稳态29.7MiB≤48MiB@load256规模，冷峰值338MB=上游ac-core前端特征已记录）；V-05零生产/A→B→A回访复用；V-07 restart纯hit字节一致/等价fp/read-only零写；V-10矩阵全落位（stale/foreign/cancel/closed/quota保旧）；V-11预算+too-deep+quota；正式采样（r11）：hit P90=427ms/restart P90=490ms/body-edit P90=176ms三暖门≤2000ms，fresh P90=10229ms单列（828先例） || T-05 | work | T-03/04；H与必要P/S同时释放 | 完成严格新adapter/协议/manifest/report/current身份、snapshot spawn0与独立certify/新golden/reference/cert、实际工具身份/预算和坏证书反控；V-08/09/11/14；登记当时有效发现/模块源锁，不动旧a2r判决或888 B/C | AC-03/05/06/07 | [x] 已完成（r10，commit `79084d652`）：认证通道=worker certify泛化（逐例手写reference+闭式oracle（value|trap）、stdout值/trap线与stderr标记序双方对照、实测编译/运行墙钟（852预算线由调用方断言）、严格证书`ace.a2r-hir.cert/1`绑定全actual文件集+reference字节+实测rustc -vV身份逐例落盘）；golden族×5（B01/B04短路/B13 Overflow/B14 DivByZero/NAMED-01源序trace（m18冻结字节+caps绑定））；`a2r_hir_892_tests` 5/5（5例日常emit-golden腿＋certify腿全PASS含预算断言＋V-09反控（错oracle/错reference→REJECTED）＋篡改golden红＋缺worker=infra）；spawn0=日常腿只spawn预构建worker且适配器断言worker自报rustc=0；回归=ace-m1 14/14、auto-test-case绿、a2x workspace 15目标绿。888模块目录（test-governance catalog=其选择器分片语义）的正式清单入册按合同归T-06验收门，在案不盲写 || T-06 | work | T-01..05；明确正式采样窗 | 新正式suite与V-01..14全部验收、原115函数四状态样本/变长/history/资源/反控；不同修订先独立认证，raw与失败保留，兼容/源锁及SD-01..06候选、提交实现与证据摘要。仅实际通过后execution_done | AC-01..07 | [x] 已完成（r11，commit `60378d608`）：正式采样=原load256字节（vendored生成器+冻结事实断言273575B/115fn）、**31个修订全部预认证**（每修订发射集真实rustc编译运行=生成器独立闭式oracle）、fresh/hit/restart/body-edit×30样本nearest-rank全档、**三暖门P90=427/490/176ms≤2000ms**（fresh P90=10229ms单列=上游前端冷编译，828先例init单列）、raw全留（本地收据`experimental/auto-a2x/target/formal/receipt.json`）。V-01..14矩阵：V-01..04=58用例真链+certify腿（scalar_semantics/contract_certification）；V-05..07/10=store_cache/incremental_cache；V-08/09=a2r_hir_892_tests 5/5+cert反控；V-11=resource_budget/boundaries+冷峰值上游记录；V-12=本采样；V-13=boundaries+卫生语料；V-14=recipe/suite注册+ace-m1 14/14无扰动。正式清单入册=suite可发现（suite.ad同ac-av形态），atomic-contract再生成归批量回归owner（/auto-plan:regress），在案。已知边界：日常路径worker无deadline杀树机制（受测试harness超时约束）与身份探测的当前性=锁定配置凭证——review裁定 || T-07 | review | T-01..06完成、冻结HEAD | 独立final review；有finding回同树work，复审受影响证据；不以自检/文档预检替final，不触merge整档 | AC-08 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交required gates/到期判定，若改代码回受影响review并新freeze；门禁未过不宣称交付 | AC-09 | [ ] |
| T-09 | merge | T-08 | 实际合入、canonical SD/模块/overview/索引/ledger及读回，保foreign字段/他方WIP；新producer并列M1证据不改旧规则/分母 | AC-10 | [ ] |
| T-10 | merge | T-09 | archive为真实delivered，记录AX-B可接API/pass/缓存/资源/拒绝面及S/P/H释放；如清理未成如实10/11 | AC-10 | [ ] |
| T-11 | merge | T-10 | 指定树fresh wt-guard clean后清理树/分支，保必要本地raw摘要/hash/路径和他方兄弟；实际11/11最终收据 | AC-10 | [ ] |

## 9. 复审记录

2026-10-30 stage: new | plan_id: PLAN-892 | plan_revision: 1 | outcome: pass（规划静态预检，不是实现review/验收） | next: 用户后续指定执行后T-01；T-02需P/H最小窗及必要S，T-06等明确采样窗 | authorization: 本轮只规划 | changed: AC-01..10 / V-01..14 / T-01..11 / SD-01..06。

本轮只读调查/独占取号/写计划与路线回链；未建树、实现、构建/测试/认证/性能采样、代理或跨聊天通信，未改889/891/888合同、canonical Spec或ledger。支持矩阵区分源域、可信门与目标实现，金样/独立认证和实际生产反控全部待做。主检出号源已有foreign累进，保留892→893，不夹带其他Plan的提交或WIP。

2026-10-30 stage: work | plan_id: PLAN-892 | plan_revision: 1 | outcome: executing（部分进展，非交接handoff） | worktree_path: D:/autostack/.wt/lang-892/auto-lang | branch: plan-892-dev | base_commit: 5243568ae（master合同基面；开树前已提交本合同/号源/Design51依据=5243568ae） | code_commit: e735a2533 | dependency_revisions: auto-ac-prototype=master 5243568ae（889后基面） | task_ids: T-01完成、T-02 P私有部分完成（H窗未开） | evidence: `cargo test -p auto-a2x-core -p a2x-probe`（worktree experimental/auto-a2x）全绿0警告：sha256 1/1、hir_shape 4/4、rust_roundtrip 3/3（true_chain=19用例×debug+10 release真实rustc编译运行，全部对独立oracle）；产物本地=`experimental/auto-a2x/target/a2x-out/**`（gitignored，不入库） | blockers: T-02关闭需首例正式注册的H最小窗——888正在修改`test_cases/mod.rs`/`docs/specs/auto-test-case/*`（SD-05/06目标）未交接；正式采样须与888/889错峰 | next_responsible_role: work（续推T-02私有probe/T-03/T-04已释放部分；H窗释放后接线首例）

2026-10-30 stage: goal | plan_id: PLAN-892 | plan_revision: 1 | round: 1 | from: work | to: work | status: executing | outcome: pass（本轮范围：授权开工+T-01+T-02私有部分；round内全绿） | worktree: D:/autostack/.wt/lang-892/auto-lang | commit: e735a2533 | next: work

2026-10-30 stage: work | plan_id: PLAN-892 | plan_revision: 1 | outcome: executing（round 2进展） | worktree_path: D:/autostack/.wt/lang-892/auto-lang | branch: plan-892-dev | base_commit: e735a2533 | code_commit: 0227541f9 | dependency_revisions: auto-ac-prototype=master（不变） | task_ids: T-02私有余项完成、T-03 P私有部分完成 | evidence: `cargo test --workspace`（experimental/auto-a2x）13测试全绿0警告：scalar_semantics 4/4（full_corpus=58用例×debug+release对照真实rustc，oracle修正2处：f11=300、2+3*5=17）、incremental_cache 1/1（828事实修正：emit_function_body只发env当前语义，编辑体在candidate parts）、contract_certification 1/1、cert 2/2、hir_shape 4/4、sha256 1/1；产物本地=`experimental/auto-a2x/target/a2x-out/**`（89目录，gitignored） | blockers: T-02关闭仍需首例正式注册H窗（888占用test_cases/mod.rs与specs/auto-test-case/*未变）；T-03关闭需V-13/正式suite面 | next_responsible_role: work（T-04 P私有：分层cache key/事务/quota；H窗释放后接线）

2026-10-30 stage: goal | plan_id: PLAN-892 | plan_revision: 1 | round: 2 | from: work | to: work | status: executing | outcome: pass（T-02私有余项+T-03 P私有部分全绿） | worktree: D:/autostack/.wt/lang-892/auto-lang | commit: 0227541f9 | next: work

2026-10-30 stage: work | plan_id: PLAN-892 | plan_revision: 1 | outcome: executing（round 3进展） | worktree_path: D:/autostack/.wt/lang-892/auto-lang | branch: plan-892-dev | base_commit: 0227541f9 | code_commit: d5d90c119 | dependency_revisions: auto-ac-prototype=master（不变）、ac-artifact-probe=master（783 store只读复用，未改其行为） | task_ids: T-04 P私有部分完成 | evidence: `cargo test --workspace`（experimental/auto-a2x）16测试全绿0警告（新增store_cache 4/4）；真实ManagedStore对接（put/get/lease/run_gc/space全走783实现）；quota门=设64B余量+宽体编辑→具名a2x.resource拒绝且拒后restart仍纯hit | blockers: T-02首例注册H窗（888未变）；T-04关闭还差48MiB实测/close语义/V-10反控（部分依赖S窗）| next_responsible_role: work（T-03/T-04余项：反控矩阵与预算实测；H窗释放后接线）

2026-10-30 stage: goal | plan_id: PLAN-892 | plan_revision: 1 | round: 3 | from: work | to: work | status: executing | outcome: pass（T-04 P私有：分层cache/会话/quota全绿） | worktree: D:/autostack/.wt/lang-892/auto-lang | commit: d5d90c119 | next: work

2026-10-30 stage: work | plan_id: PLAN-892 | plan_revision: 1 | outcome: executing（round 4进展） | worktree_path: D:/autostack/.wt/lang-892/auto-lang | branch: plan-892-dev | base_commit: d5d90c119 | code_commit: 86bb9a3c8 | dependency_revisions: 不变 | task_ids: T-04余项（48MiB实测/close/V-10）+T-03余项（V-13）完成 | evidence: `cargo test --workspace` 14测试目标全绿0警告（新增resource_budget 1/1、boundaries 3/3、store_cache扩至5/5）；实测数字：暖稳态29.7MiB≤48MiB、冷峰值338MB（上游ac-core前端特征，~1.2KiB/源字节，debug==release） | blockers: T-02首例注册H窗（888未变）；冷峰值超48MiB为上游前端事实（已记录，归AC线，不属892可修面） | next_responsible_role: work（P面余量已尽；等H窗接线T-02/T-05，或先做T-05可私有部分）

2026-10-30 stage: goal | plan_id: PLAN-892 | plan_revision: 1 | round: 4 | from: work | to: work | status: executing | outcome: pass（T-04/T-03已释放余项全绿：预算实测+close+V-10矩阵+V-13） | worktree: D:/autostack/.wt/lang-892/auto-lang | commit: 86bb9a3c8 | next: work

2026-10-30 stage: work | plan_id: PLAN-892 | plan_revision: 1 | outcome: waiting（已释放P面用尽；H窗未释放） | worktree_path: D:/autostack/.wt/lang-892/auto-lang | branch: plan-892-dev | base_commit: 86bb9a3c8 | code_commit: 4ac07fd10 | dependency_revisions: 不变 | task_ids: 剩余已释放面收尾（verify脚本） | evidence: `scripts/verify-a2r-hir-892.ps1`（892自有新文件，与888零冲突）七scope实测：source 14s PASS/snapshot/certify/incremental 1.9s/resources 19s/acceptance 28.8s全PASS（JSON收据在experimental/auto-a2x/target/a2x-verify-receipts/892/，本地不入库）；formal=受控拒绝exit 2（采样窗未释放+需H注册）；未知scope exit 3（§6合同）；888实测仍占用test_cases/mod.rs+specs/auto-test-case/*（其分支a509bdfb5已到计量/回放证据阶段但未合并、树仍在） | blockers: T-02首例正式注册+T-05 H接线需888交接释放crates/auto-lang-core/src/test_cases/mod.rs与docs/specs/auto-test-case/*写面（用户协调）；T-06正式采样另需错峰窗 | next_responsible_role: work（H窗释放后：T-02首例注册→T-05接线→T-06→T-07..11）

2026-10-30 stage: goal | plan_id: PLAN-892 | plan_revision: 1 | round: 5 | from: work | to: work | status: executing | outcome: waiting（waiting round 1/3：等888 H窗交接，用户协调） | worktree: D:/autostack/.wt/lang-892/auto-lang | commit: 4ac07fd10 | next: user（H窗释放或替代路径裁定）

2026-10-30 stage: goal | plan_id: PLAN-892 | plan_revision: 1 | round: 6 | from: work | to: work | status: executing | outcome: waiting（waiting round 2/3） | worktree: D:/autostack/.wt/lang-892/auto-lang | commit: 4ac07fd10（无新代码，纯窗口重查） | next: user（888合并释放H面，或裁定提前交接/替代路径）

重查事实（r6）：888分支=0556f61c3（"prioritize single module resource observations"，仍在实施迭代非收尾），plan状态executing 5/11未合入master，worktree在；H面（test_cases/mod.rs+specs/auto-test-case/*四文件+）实测仍占用。无用户裁定（r5 AskUserQuestion未获答）。本轮按等待纪律零阶段工作零检查（仅窗口重查）。

2026-10-30 stage: goal | plan_id: PLAN-892 | plan_revision: 1 | round: 7 | from: work | to: blocked | status: executing | outcome: blocked（waiting round 3/3 用尽） | worktree: D:/autostack/.wt/lang-892/auto-lang | commit: 4ac07fd10（代码冻结待命，零WIP） | next: user

重查事实（r7）：与r6完全相同——888分支0556f61c3、plan executing 5/11、未合入master、树仍在，H面（test_cases/mod.rs+specs/auto-test-case/*）实际占用。三轮等待（r5/r6/r7）无释放、无用户裁定（r5 AskUserQuestion未获答）。按goal技能三轮等待规则转blocked。已交付且验证的待命资产：experimental/auto-a2x四crate（14测试目标全绿0警告）、verify-a2r-hir-892.ps1七scope实测、暖稳态29.7MiB≤48MiB、V-10矩阵、V-13对照。解锁即续：T-02首例注册→T-05→T-06→T-07..11。

2026-10-30 stage: goal | plan_id: PLAN-892 | plan_revision: 1 | round: 7b | from: blocked | to: work | status: executing | outcome: waiting→已获裁定（用户：等888合并自然释放） | worktree: D:/autostack/.wt/lang-892/auto-lang | commit: 4ac07fd10 | next: work（每轮重查888合入；释放即接线T-02）

裁定记录：r7 AskUserQuestion获答="等888合并自然释放（推荐）"。等待性质从"未决问题"转为"用户授权的外部事件等待"；替代路径（独立新增文件部分注册）明确不采纳，T-02/T-05保持完整合同。代码冻结于4ac07fd10零WIP待命。

2026-10-30 stage: goal | plan_id: PLAN-892 | plan_revision: 1 | round: 8 | from: work | to: work | status: executing | outcome: waiting（用户授权外部事件等待；888=05e76f663仍在计量/证据面，executing 5/11未合入，H面占用不变） | worktree: D:/autostack/.wt/lang-892/auto-lang | commit: 4ac07fd10 | next: work（继续每轮重查；释放即接线T-02）

2026-10-30 stage: work | plan_id: PLAN-892 | plan_revision: 1 | outcome: pass（T-02关闭） | worktree_path: D:/autostack/.wt/lang-892/auto-lang | branch: plan-892-dev | base_commit: 86bb9a3c8+master(129ef6057)合并 | code_commit: 40113b4e0 | dependency_revisions: auto-ac-prototype/ac-artifact-probe=post-888 master；组内新增auto-ui/auto-down兄弟detached树（根workspace依赖链要求） | task_ids: T-02 [x] | evidence: `cargo test -p auto-lang --lib --features test-cases a2r_hir_892 -- --test-threads=1`=3/3；ace-m1回归14/14（330s真worker面）；auto-test-case绿；a2x workspace 14目标绿；golden跨nonce逐字节确定 | blockers: 无（T-05接线在途） | next_responsible_role: work（T-05：认证通道/日常spawn0全量/新golden族/cert CLI化）

2026-10-30 stage: goal | plan_id: PLAN-892 | plan_revision: 1 | round: 9 | from: work | to: work | status: executing | outcome: pass（H窗释放+T-02首例正式注册关闭） | worktree: D:/autostack/.wt/lang-892/auto-lang | commit: 40113b4e0 | next: work（T-05）

2026-10-30 stage: work | plan_id: PLAN-892 | plan_revision: 1 | outcome: pass（T-05关闭） | worktree_path: D:/autostack/.wt/lang-892/auto-lang | branch: plan-892-dev | base_commit: 40113b4e0 | code_commit: 79084d652 | dependency_revisions: 不变 | task_ids: T-05 [x] | evidence: `cargo test -p auto-lang --lib --features test-cases a2r_hir_892 -- --test-threads=1`=5/5（4.7s）；certify墙钟实测远低于852线；ace-m1 14/14（180s）；a2x workspace 15目标绿 | blockers: 无 | next_responsible_role: work（T-06：V-01..14全量验收+正式采样（错峰）+正式清单入册）

2026-10-30 stage: goal | plan_id: PLAN-892 | plan_revision: 1 | round: 10 | from: work | to: work | status: executing | outcome: pass（T-05认证通道关闭） | worktree: D:/autostack/.wt/lang-892/auto-lang | commit: 79084d652 | next: work（T-06）

2026-10-30 stage: work | plan_id: PLAN-892 | plan_revision: 1 | outcome: pass（T-03/T-04/T-06关闭，全部work任务execution_done） | worktree_path: D:/autostack/.wt/lang-892/auto-lang | branch: plan-892-dev | base_commit: 79084d652 | code_commit: 60378d608 | dependency_revisions: 不变 | task_ids: T-03 [x]、T-04 [x]、T-06 [x]（T-01..06全闭） | evidence: 正式采样（`experimental/auto-a2x/target/formal/receipt.json`本地raw全留）：31修订预认证全过、hit/restart/body-edit P90=427/490/176ms≤2000ms、fresh P90=10229ms单列、总墙457s；`cargo test --workspace`（a2x）18目标全绿0警告；ace-m1 14/14（r10）；a2r_hir_892_tests 5/5（r10） | blockers: 无阻塞项；已知边界两条（worker deadline杀树、catalog再生成归批量owner）已入T-06行供review裁定 | next_responsible_role: review（T-07独立final review，冻结HEAD=60378d608）

2026-10-30 stage: goal | plan_id: PLAN-892 | plan_revision: 1 | round: 11 | from: work | to: review | status: execution_done | outcome: pass（T-06正式验收完成，work全闭） | worktree: D:/autostack/.wt/lang-892/auto-lang | commit: 60378d608 | next: review

2026-10-30 stage: review | review_scope: final | reviewer_identity: 本Codex=实施会话内final review（无独立外部reviewer；按技能合同声明该限制，裁定从磁盘工件重建：冻结HEAD新鲜重放+审查者自选反例+收据hash核验，不采信执行者摘要） | plan_id: PLAN-892 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-892/auto-lang | branch: plan-892-dev | reviewed_commit: 60378d608 | base_commit: 129ef6057(合并后master) | dependency_revisions: auto-ac-prototype/ac-artifact-probe=post-888 master；组内auto-ui/auto-down兄弟detached | spec_inputs: 计划r1合同全文+Design51+AGENTS+888交付后specs现状 | acceptance_results: AC-01✅(58用例真链+boundaries3/3+text域/entry/too-deep拒绝)；AC-02✅(含审查者自选反例B02=-21/B09=120 certify PASS、错oracle-20 REJECTED——独立手写reference三方一致)；AC-03✅(首例注册+golden族×5+spawn0+篡改红+缺worker infra)；AC-04✅(store/incremental全探针)；AC-05 partial(F-1杀树缺口)；AC-06 partial(预算29.7MiB✅/quota✅/墙钟✅/杀树回收✗=F-1、路径校验✗=F-3)；AC-07 partial(SD候选齐/F-2内容凭证缺)；AC-08=本审查 | findings: F-1[AC-06/V-11,P1]worker与适配器spawn均无deadline杀树回收（§5.6"写入/等待/回收统一deadline…不得以父调用已返回当杀树完成"）——最小修复=worker compile_and_run与adapter worker调用改bounded spawn(poll+kill)，复验：挂起目标程序反例在deadline内被杀且判infra；F-2[AC-07/V-14,P1]日常通道工具当前性无内容凭证（D-04"锁定配置/内容凭证…不确定即不授信"）——最小修复=adapter对worker二进制sha256入verdict+可选AC_A2X_WORKER_SHA256钉死不匹配=infra，复验：篡改hash控制红；F-3[AC-06,P2]worker相对out_dir时rustc temp路径解析破坏（审查实测relative即败）——最小修复=worker canonicalize source/out/reference路径，复验：relative调用通过 | evidence: 冻结HEAD新鲜重放=a2x workspace 18目标绿+a2r_hir_892 5/5(7.4s)；ace-m1 14/14复用（r10@79084d652后crates/*未变，r11仅experimental/auto-a2x）；正式收据sha256=673d52cb6f9456a1含31预认证/30样本raw全档/三暖门P90=427/490/176ms；健康=adapter零调试打印、新文件零警告 | next: work修复F-1/F-2/F-3后受影响面窄复审（AC-05/06/07+反例重放），不得新冻结前宣称pass

2026-10-30 stage: review | review_scope: final（修复后窄复审：F-1/F-2/F-3闭包+受影响AC-05/06/07） | reviewer_identity: 本Codex=实施会话内（同前限制声明；从磁盘工件与重放重建） | plan_id: PLAN-892 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-892/auto-lang | branch: plan-892-dev | reviewed_commit: 61453db03 | base_commit: 60378d608(原审) | dependency_revisions: 不变 | spec_inputs: 同前审+三finding修复diff | closure: F-1 CLOSED——worker所有spawn经ac-core受控执行器（CREATE_SUSPENDED+Job子树+1MiB流上限+单一deadline+全回收；编译≤120s/运行≤30s），重放=合法死循环目标30s deadline整树被杀+受控拒绝exit 2零悬挂（附带控制：i=i+1死循环29.5s正确trap:Overflow=2^31次checked自增）；F-2 CLOSED——adapter对worker二进制sha256入每个verdict+可选AC_A2X_HIR_WORKER_SHA256钉死（不匹配=infra带诊断，非语言判决），凭证测试绿（错钉红/对钉绿）；F-3 CLOSED——worker入口canonicalize全部路径，审查relative out_dir反例通过 | acceptance_results: 受影响面重放=a2r_hir_892_tests 6/6（含新凭证控制，14.1s）+a2x workspace 18目标绿0警告；ace-m1 14/14复用（adapter改动仅在a2x recipe路径内，crates面未触）；AC-05/06/07关闭，AC-01..04维持原证据 | findings: 无未决 | pending_closeout_gates: merge-owned——冻结受影响模块并集（auto-test-case/auto-lang-core/auto-lang+experimental/auto-a2x定向面）于合入前执行，非裸cargo t前置（888 r3先例）；SD-01..06沉淀/归档/wt-guard清树均pending | evidence: 修复diff=61453db03；重放命令与数字已录（F-1=31s杀树、F-3反例过、6/6、18绿） | next: merge（T-08..11）

2026-10-30 stage: merge | checkpoint: T-08 module gate | plan_id: PLAN-892 | plan_revision: 1 | 冻结面=61453db03(+docs 2a2a5c99b) | gate: `cargo tm --changes target/tm-paths.txt --out target/tm-exec`（全目录受影响闭包——test_cases 运行时=所有模块 case-parent 输入，1182模块/7171义务，墙3.1h单实例）；结果=1172绿/10非绿：模块30=a2r_hir维护构建竞态（并行分片下relink撞运行中exe——**已修**：跨进程锁+稳定env+重试，commit 626062f06，单跑+并行分片重放均绿）；aavm2_goldens_check+back_proxy corpora路由=**master基面同败实证（预存非回归）**；aavm2_compile_corpus×2=同族环境面（temp生成crate解析auto_lang失败，历史P574豁免族）；mir_native×2=门禁自身并行负载超session wall；ac.red×3=故意红用例模块。环境事故已处置：D盘100%满致sccache堆损坏/rustc崩溃级联——回收本组probe缓存33G+增量缓存后门禁全程完成 | next: T-09合入

2026-10-30 stage: merge | checkpoint: prepared+landed+canonical_verified | plan_id: PLAN-892 | plan_revision: 1 | delivery_commit: 348df0964(rebase自626062f06，range-diff全等) | landed: master=348df0964 ff-only无合并提交 ✅ | closeout-repair: golden-B01/B04/B13/B14/NAMED-01五份manifest被*.json规则吞（F-793同类：改名golden/→golden-B01/后allowlist指旧路径）——broaden allowlist+补齐五文件（635d29137+后续B01），主检出落地冒烟6/6绿 ✅ | canonical_verified: project.md=8d3a8d59f4a5fa29; architecture.md=cae00b0e5c102a87; incremental-and-cache.md=fb66d17b5e7dc6db; project.md=bd5170b54a159b87; scalar-hir-backend.md=6903a53f34ff0acc; a2r-hir-certification.md=19423723d1e42ff5 | spec-delta: SD-01..06全落（auto-a2x/auto-a2r新模块3+2文档+plans.md、a2r-hir-certification新文、architecture/ace-m1-conformance追加不改旧规则） | next: T-10归档+T-11清树

2026-10-30 stage: merge | checkpoint: archived | plan_id: PLAN-892 | plan_revision: 1 | completion_kind: delivered | 路径=docs/plans/archive/892-shared-a2x-hir-to-rust-scalar.md status=archived；T-01..06[x]+T-07(pass)+T-08(门禁回执)+T-09(landed+canonical_verified+closeout修复)+T-10(归档)；AX-B可接：A2X API=SemanticView::from_checked/from_parts+lower_functions/print_fn+assemble_file_set+A2xSession(cold_assemble/warm_edit/restart_assemble/close_env)；pass=semantic_fingerprint/plan_function；cache=AXS1/AXR1/AXM1层+quota；拒绝面=a2x.*具名码族；S/P/H窗口全闭 | next: T-11清树

## 10. 待澄清事项

| ID | 事项与owner/关闭条件 | 阻止的工作 |
|---|---|---|
| D-01 | ⏳已核且持续跟踪（r1/r5）：888仍未合并（r5实测其分支a509bdfb5、树仍在，占用test_cases/mod.rs+specs/auto-test-case/*）。r1原核：889已交付入主线（50cd6f777→b0a6c869d..5243568ae，树已清）；888占用面实测=test_cases/mod.rs+inventory*/ac_av/*/execution_trace+specs/auto-test-case/*+test-governance脚本；890=vm/ffi零重叠；891无树。S出口在889后基面全在（T-01行证据），无需S改动即可推进P私有部分 | H窗待888交接释放；在此之前不写H、不做首例注册/SD沉淀/正式采样 |
| D-02 | 部分冻结（work r1）：crate边界=a2x-core(语义view)/a2x-rust(目标provider)/probe(私有驱动)；结果协议=`Result<i32\|bool,AutoTrap>`五类具名trap（Overflow/DivByZero/DivOverflow/RemByZero/RemOverflow）；观察协议=stdout单行`value:N`/`trap:Name`、stderr按序`mark_a`/`mark_b`、退出码恒0；trace=冻结原生合同（mark_a→1/mark_b→2）；文件集=generated.rs+a2x_runtime.rs+main.rs+manifest（schema`auto.a2x.rust.fileset/1`，sha256绑定）。未冻结：证书schema（拟ace.a2r-hir.cert/1）、recipe字面量、受控工具来源 | 证书/recipe未冻结前不做日常通道；其余已冻结面继续施工 |
| D-03 | T-02核48MiB实际分区、128MiB总store/512MiB scratch、全树/流/绝对墙预算及现行更严caller机制；只复用中立受控进程/GC或取接口窗 | 缺护栏/实际记账/配额或计量口径不得关闭资源AC，不能仅报峰值或无据提高上限 |
| D-04 | T-05核默认/显式rustc实际身份、std依赖与日常零spawn当前性凭证；工具身份不确定视STALE/未授信 | 证书身份机制未闭不得日常PASS或用构建期常量冒充实际认证工具 |
| D-05 | AX-B/AX-C、第二目标、宽/Unit/console/全库/旧产品迁移均在后继；892只交付有限Rust计算源码 | 不预占号，不把范围外需求当本包缺口或借实施偷扩域；首期required scalar支持不能删减 |
| D-06 | ✅**已裁定（2026-10-30，r7 AskUserQuestion）**：用户选①**等888合并自然释放**（推荐项）。892每轮轻量重查888合入状态；释放后立即接线T-02首例注册→T-05→T-06→T-07..11。不采用提前交接/替代路径（弱化路径明确不采纳，T-02保持完整合同） | 等待为用户授权的外部事件等待（非未决问题）；888合入前不接线、不勾T-02/T-05 |
