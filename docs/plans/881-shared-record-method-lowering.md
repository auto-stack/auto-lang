---
plan_id: PLAN-881
status: drafting
feature_name: 共享 record 方法 lowering 与 AV/native 对等覆盖
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 2
current_step: 0
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-mir/hir-lowering-boundary.md
  - docs/specs/auto-mir/data-place-profile.md
  - docs/specs/auto-mir/owned-access-profile.md
  - docs/specs/auto-mir/native-data-owned-lowering.md
  - docs/specs/auto-test-case/ace-m1-conformance.md
  - docs/specs/auto-av/resident-function-install.md
new_spec_components:
  - docs/specs/auto-mir/record-method-lowering.md
touched_goals: [GOAL-002]
affects: [auto-ac, auto-mir, auto-av, auto-test-case]
depends_on: [PLAN-868, PLAN-870, PLAN-862, PLAN-828, PLAN-820, PLAN-821, PLAN-852]
---

# [PLAN-881] 共享 record 方法 lowering 与 AV/native 对等覆盖

## 0. 变更摘要

将旧record/text-record已批准的static/只读实例方法，经唯一auto-mir-lower映射为普通MIR函数/调用，供AV与mn同源消费。关闭ACE-M1原R08（42）及871对应方法覆盖行；整模块、单函数生产、单元链接、cache恢复均须验收。

本轮仅规划。下述探针、测试入口及oracle为待执行合同，不是新运行证据。D881不等于871全局G-COV/G-DELETE完成；本包不切默认、不删除旧native。

## 1. 目标

- 旧合法方法声明与调用全部进入共享链：static无receiver；view receiver恰一次、先于显式实参，含字段路径/临时receiver、命名实参和方法间调用。未调用的合法方法也不得再使整模块lower.method。
- 保留771/782名义类型、只读/互斥借用、显式参数模式、move/clone、拥有返回与清理、源码诊断。AV/mn各自对独立oracle，旧native仅作第三条差分证据。
- 方法成为函数级稳定单元，包含owner/方法身份、类型/模式及完整依赖；fresh/hit/restart/body-edit可观测，未改函数生产工作=0。

非目标：mut实例方法、trait/ext/动态派发/用户泛型、新重载规则、Unit方法、collections/闭包方法、生产ABI/FFI、新opcode、CTE复合扩面、优化及871迁移/删除。864的List内建方法仍由864拥有；record方法不自动扩至av.collections/1。

## 2. 架构方案

source(record/text-record)→既有resolver/typecheck/adapter→Checked HIR(Method+Call)→auto-mir-lower→verify_mir→ABC/AV或mn/CLIF/COFF/PE。

HIR已有DefKind::Method及ReceiverKind，View receiver已经是签名params[0]；eval_args/bindings已有求值与绑定顺序。优先将方法去糖为唯一普通MIR函数，static不补receiver，view不重复补receiver；不建第二套方法runtime。此表示是实施假设，T-02先以全链探针确认，不能只删拒绝分支。

维持av.data/1、av.owned/1及MIR/ABC/wire指令语义；统一方法Def映射、签名/模式、单元import/link，不在AV/mn复制lowering。语义扩面时相对实际合入基面升MIR_LOWERING_VERSION，同步三消费者身份及旧语义cache失效，不预钉版本数字。

## 3. 技术栈

| 写面 | 已核实际路径/拟新增入口 |
|---|---|
| S前端/共享lowering | experimental/auto-mir-lower/src/{hir_adapter,lib}.rs；experimental/ac-core/src/source/{ast,parser,resolve,typecheck,adapter}.rs的方法分段、保留环境及候选发射缝是必要调查/改造面；hir.rs、verify.rs仅改必要消费点，既有整模块方法语法不重写 |
| S会话/单元 | experimental/ac-av-probe/probe/src/{data_session,owned_session,function_queries,cache,fn_hir_codec}.rs；T-01沿真实调用图确定必要写集；core/src/**不新增指令/执行语义 |
| N | experimental/ac-mir-native/src/{incremental,lower,emit}.rs及tests；仅方法消费必要适配，不独占整个N |
| H | crates/auto-lang-core/src/test_cases/{ac_av,ac_mn,ac_cte}.rs身份/适配点；crates/auto-lang/src/tests/ace_m1_tests.rs；crates/auto-lang/test/cases/ace-m1/binding.tsv、SDK/worker/公共注册（以迁移后实际路径为准） |
| 拟新增 | experimental/ac-mir-native/tests/record_method_tier.rs、probe/tests方法与增量定向文件、文件化TestCase suite、scripts/verify-method-lowering-881.ps1；当前均未实现 |

复用既有case worker、contained executor、text runtime，不另造框架。唯一拟实施树D:/autostack/.wt/lang-881/auto-lang，分支plan-881-dev；当前未建树。实施/复审/合入同树顺序执行，主检出仅计划簿记。

## 4. 需求分析与背景调查

授权：用户2026-10-29要求立共享方法lowering计划、核清871旧方法缺口、明确864交接；仅规划，未授权本轮施工。允许Plan/规划报告/中央取号簿记；不写canonical Spec/ledger、不建实施树、不启动或联系其他会话、不构建/测试。

同日后续授权：复核871方法覆盖、参数模式/资源/函数增量及864/866交接，有遗漏直接修订，仍只规划。r2据此补强验收，不改变旧语言合法域、预算和本轮动作范围。

已读AGENTS.md、docs/specs/overview.md。依据现行auto-ac/source-record.md、source-text-record.md，以及auto-mir的overview、hir-lowering-boundary、data-place-profile、owned-access-profile、native-data-owned-lowering；目标态遵循Design43/49与ACE波次合同。调查基面master 62fb3e3f1（调查期间其他会话推进master）；D868=7754c4e4c、D870=bc90ef718已合入，D862已归档。输入hash与分账详见[缺口报告](reports/ace-method-lowering-gap-2026-10-29.md)。work必须刷新实际revision，不消费上游WIP。

| 库存ID | 已观察旧能力/当前缺口 | 本包关闭条件 |
|---|---|---|
| M-01 | 771 static new返回record；R08；lower_checked_data/owned遍历任一Method即lower.method | 真发射声明/调用，含未调用方法控制；R08原源AV/mn=42，771 p01=0 |
| M-02 | HIR view receiver=params[0]，verify有owner/mode门；771 receiver_once_then_args_in_order | 只读借用不复制owner、先求值一次；原trace=aab且exit0；含嵌套字段/temporary |
| M-03 | 782 Tok.new(move Str)、text_len及Token微程序；owned整体拒Method | 拥有构造/返回/借用读全链，原Token=57、资源守恒且无隐式clone |
| M-04 | lower_function_data_unit/owned_unit_with_modes也拒Method；owned_fn_sig_for_def及callee_mut_record_params取模式只匹配Function | whole/unit模式一致，方法显式move/mut参数责任不退化为默认View，receiver不重复 |
| M-05 | cache::split_units用fn:<name>；function_queries的scalar入口仅收Function | 核data/owned真实入口；owner+method稳定身份、同名方法/自由函数不混链、完整依赖与恢复；不能直接放开scalar冒领支持 |
| M-06 | owned_session::decl_facts_of遇类型内methods即fast_path_ok=false，identity_of/body_fp_of/保留mode表仅收Function；data_session::outline_of只把顶层Fn记为函数，Type整体记非函数；两者单体parse均以自由Fn为入口 | 方法体编辑必须真正单体parse/recheck/emit/lower，不能重编整type后报cache hit；方法owner/self环境、类型壳/兄弟方法声明坐标与候选提交/回滚均需接通；V-11/12 |

870的文本view eq/ne源级gap、split_units内嵌类型索引、mut资源字段释放与多帧trap边界不等于方法拒绝同一问题。方法所需签名/类型重映射修复归本包；其他旧域gap由871继续具名列账。若方法原合法例触发尚未裁定的资源语义矛盾，T-02给最小反例/差异/方案，未决不得pass，不缩合法面或套NA。

## 5. 详细设计

### 5.1 864及其他owner交接

语义前置为已交付820/821/828/852/862/868/870；D864是本轮共享基面/兼容交接门，不代表用户方法依赖集合语义。现在可只读核查；公共实施必须等864最终review、冻结门禁、合入、Spec及写面释放齐备。

默认新增包插入位置为：864→866→881→（871删除窗，若就绪）→865→863；保持866→865→863相对顺序。这是规划排序，尚无owner释放。若866尚未接线且相关owner登记释放/接受，可在864后提前插881。881未就绪就让窗，不能持窗等条件；871盘点可并行，其公共修改与881互斥。排序不是新增D866语义依赖。

**分段执行门**：T-01只读调查可与864并行；T-02起包括首例探针在内的任何源码/测试/注册写入，都须D864已交付并接受相关窗口。N当前空闲也不能单独启动881主体绕过S门。本包无可独立交付的私有后端实现段；若要另拆私有段，先修订计划并列清无公共合同写入的真实边界。

| 范围 | 864/当前owner释放→881接受 | 881释放→871/下一owner接受 |
|---|---|---|
| S源码/HIR/lowering | Plan修订、delivery、collections profile/descriptor/API/公开枚举、lowering版本、限制；刷新已合入基面 | 方法身份/签名/版本及拒绝面；864原L04/L05=42、List<Str>新变体51及profile4兼容 |
| S会话/cache | 函数级接口/codec/恢复、失效键、生产计数/预算 | 方法body/interface依赖及恢复门、未改函数零生产和payload身份、旧cache失效 |
| N | 当前mn owner释放具体文件/符号；与871改动互斥 | 方法调用/布局/ABI实测、N写集、871逐行关闭/残留gap |
| H | SDK迁移owner真实路径；driver/注册/binding窗口 | R08实测、mir-lowering镜像、父侧oracle/反控及分母审计 |

866方向分两种：默认先D866后881时，881接线前接受866的snapshot/AV/session/driver/worker公开签名、版本/恢复规则、安全点、candidate/pin/lease及H注册清单，并以已交付866最小标量在线用例（新调用见新版、旧链保旧、失败不覆盖、有界回收）做定向兼容回归。若881提前插在866前，必须登记866未持有相关窗口；881交付同样的公共接口/版本/限制清单，866实际接线前刷新D881并验其既有标量合同。本包V-12只验方法在828事务快照的寿命，不借此宣称复合方法在线HMR/迁移已完成。任何866 review/repair在途接口均不能被消费。

跨S/N/H同批取所需窗口，不持一个等另一个；review/repair期间仍归原owner，不交WIP。T-02在同一实施树接受交接后做首例探针。本轮不改864正文/状态/勾选，冻结件不等于其实现已验收。

### 5.2 语义、身份及准入

- 按名义owner、方法名及已验证Def身份定位单元，不仅用裸方法名/表位置。两个type同名方法、同名自由函数、无关声明插入/重排不能错链。
- static无receiver；view params[0]只读，receiver先于显式eval_args且恰一次；按bindings映射签名不重排求值。self/隐式字段同义；temporary receiver活到调用及合法视图消费结束。
- 沿771/782支持的参数模式/返回型，特别是owned static的move参数和record返回、view参数合法借用返回；禁止Method模式默认化。mut实例/Unit方法仍按旧域拒绝，不因862/864新类型而放开。
- **receiver模式与显式参数模式分离**：只读实例方法可以有作用于其他对象的显式mut参数，不能因“只读”抹掉其回写；receiver为view不能占用MutRet.wb。mixed view/mut/标量参数按原签名位置映射wb，按eval_args源码位置求值；多个move实参按binding逐个转移，不能只认第一个或将receiver偏移错算。
- whole与single-function产物均真实verify/admit；错误owner/receiver/签名/绑定受控拒绝。无第二份HIR→MIR，不fallback旧native。
- body-only只重做改变的方法单元，调用者/无关函数生产=0且payload身份保持；结果由新链接改变。签名、模式、owner布局、候选/配置变化按实际依赖失效，不能以调用点不变命中。
- restart decode+verify；旧语义cache真实下一次读取必须miss/stale再重建，不恢复trusted标记。同步mir-lowering三消费者，ace.m1.cert/1精确键集不变。
- 类型外壳与方法体分开路由：方法body-only（包括变长、UTF-8与花括号内注释/字符串）不能触发整type/模块前端；type字段/方法声明集/签名/模式变更显式走接口失效道，不要求其未改函数前端也为0。源码坐标平移只随成功发布提交，拒绝候选不平移。不得只靠名称替换或扫描大括号猜测方法边界。
- 使用原828保留环境/候选事务：prepare不改变当前source/env/root/revision；resolve/typecheck、MIR/ABC/admit/link、运行探针失败或取消/foreign/stale/close均不覆盖可用版。成功提交在同一发布点更新source/owner方法表/环境槽/坐标/快照；body-less scaffold必须保留owner/receiver/模式事实。旧pin保持旧代码，close/最后pin释放按原寿命合同回收。

### 5.3 支持与资源/首例门

保持旧record/text-record合法域，不附加方法数/深度语言上限。T-02从820/821/828/858与current Spec提取值数据/managed/worker/stack/cache预算交集，列来源、单位、阈值、测量入口及边界控制；不自行提高预算。显式clone才复制，借用方法不能偷偷复制receiver资源。

首例即文件化AutoDown/TestCase；fresh/hit/restart/body-edit分别计量端到端周转、峰值/live/cache bytes。P90≤2000ms为现开发期正式门，长期亚秒与P95/P99另报；T-02沿上游冻结代表负载/轮数/冷暖边界。实际parse/resolve/typecheck/lower/verify/encode/store/codegen生产分发与payload身份分别观察；强制全模块生产反控必红。normal/budget/cancel/trap退出分别按冻结责任验证，不能将旧native fatal依赖OS回收解释成live0。

资源oracle分层记录：view receiver/显式view参数由caller持有，callee不得free/clone；move-in由callee承担清理，move-return在交回caller前不得free。临时receiver的buffer活到最后合法使用，存入超出其寿命的借用必须拒绝。正常退出精确alloc/free/live；构造/clone失败检查部分输出回滚和旧目标未提交；AV取消/预算/trap按其帧释放规则，旧native fatal按782部分回滚+退出70规则分别核实，不强求fatal live0或跨后端不存在的同一释放序。多帧trap等尚未裁定差异仍列明阻塞，不以统一“exit70”掩盖资源问题。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-mir/record-method-lowering.md | 无→static/view、签名/owner/资源及whole/unit方法合同 | 唯一归属/边界 | AC-01..04 |
| SD-02 | modify | docs/specs/auto-mir/data-place-profile.md、owned-access-profile.md | 首批拒Method→旧record/text-record方法共享lowering可用；其他排除保留 | 准入扩面 | AC-01/02 |
| SD-03 | modify | docs/specs/auto-mir/hir-lowering-boundary.md | lowering语义版本、身份镜像与旧cache失效 | 可追溯 | AC-03 |
| SD-04 | modify | docs/specs/auto-mir/native-data-owned-lowering.md | 方法消费/单元适配；非方法gap独立保留 | 不冒领D871 | AC-04/05 |
| SD-05 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 原R08 ac/av实测42与补充文件化方法case；a2r双轴不混 | 真实分母/oracle | AC-05 |
| SD-06 | modify | docs/specs/auto-av/resident-function-install.md | 原data/owned自由函数单体编辑→方法分段/owner环境、body-less模式事实、候选事务/坐标平移、方法快照寿命 | 记录M-06真实扩面，区分866在线机制 | AC-03/04 |

source-record/text-record语言语义不改；merge更新来源索引/overview/plans/ledger。本轮不发布canonical。

## 6. 测试设计

以下均由work建立/运行；expected按旧Spec/原源码手算。shared实际入口为lower_checked_data/owned与lower_function_data_unit/owned_unit_with_modes→verify→ABC/AV；mn为incremental/split_units→emit/link→PE；旧native用auto-ac-prototype build-source/build及contained executor。文件化父侧沿现有ac-av/ac-mn SDK/worker，T-05建立suite。负例沿既有具名source/verify码族，不能用unsupported拒合法例。新变体另编号，保留原源/expected。

| V | AC/不变量 | 输入/状态 | 区分错误实现的observable oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 声明与调用 | 原R08、771 p01、未调用合法方法变体 | 42/0；data/owned域分别whole及unit真执行，不再lower.method | T-02探针/T-03/05；非仅typed fixture |
| V-02 | AC-01 求值/身份 | 771 receiver_once_then_args_in_order；逆序命名参数、同名方法/自由函数、前向/嵌套调用补例 | 原trace=aab、exit0；补例T-02冻结手算值/trace；重复receiver/按声明序eval/裸名错链反控红 | T-03/05 |
| V-03 | AC-02 owned方法 | 782 p01、v12 Token原源；Str移入static构造与view读字段、显式clone | p01=30（5+5+6+9+5），Token=57；Token一LIT7B/一CLONE7B，各一次free、normal live0，无隐式clone；台账沿870三级口径 | T-02/03/05；不扩mut实例 |
| V-04 | AC-02 准入/隔离 | Pair.get()静态误调、view写、错owner/签名/绑定、move后读、用户clone方法；core/scalar/text非法Method | source.not-callable、immutable-write、ownership等既有码族；HIR变异无Checked，backend生产0；合法同型控制通过 | T-03/05；原拒绝集合不放宽 |
| V-05 | AC-03 增量/恢复 | 方法body42→43→42；模式/签名/owner布局及配置改变；旧/坏cache；fresh/hit/restart | body编辑只生产改变单元，未改生产=0、payload身份保持；接口依赖精确失效；restart真实重验；强制全模块生产红 | T-04/05；AV/mn分别量测 |
| V-06 | AC-02/04 资源/链接 | 拥有temporary receiver、合法借用返回；normal/budget/cancel/trap；多record+异构参数返回扰动unit类型表 | 不逃逸/双free/隐式clone，签名布局正确；预算及退出责任准确；未裁定资源差异保留阻塞 | T-02/04/05；通用非方法gap另列owner |
| V-07 | AC-04 兼容/性能 | 864原L04/L05与Str新变体；旧五profile及CTE标量消费者 | 42/42/51；原profile4、拒绝域不漂移；版本一致，P90≤2000ms及冻结内存/cache门成立 | T-05；不声称native collections |
| V-08 | AC-05 交871/负控 | 方法库存逐行case、原R08；删一覆盖/改父oracle/错owner链接实际变异 | 父侧FAIL且总门非零，确认变异发生；缺方法行证据不能交付；非方法gap不被归绿 | T-05；不执行871切换/删除 |
| V-09 | AC-01/02 参数位置/模式 | 下方P-01..06；static/view、混合双mut、两个move、拥有返回、逆序命名实参；whole/unit与body-less scaffold对照 | P-01/02=75且q=5/p=7；P-03/04=42且4B/1B buffer各恰一次free、零clone，交换a/b=33；P-05=135、P-06=52；错wb/漏模式/receiver偏移反控红 | T-02冻结源，T-03/04/05执行；同源冲突只换对象，不引入类型错误 |
| V-10 | AC-02 寿命/失败原子性 | 具名存活owner字段借用返回；临时receiver立即读与借用逃逸；静态方法构造/clone多Str字段时注入第2次分配失败，含旧目标自clone替换 | 字段“A中”len=2且borrow本身零alloc/clone、callee零free；合法立即读完成后才释放；逃逸verify.borrow-escape；失败只回滚已产新字段、不提前释放/提交旧目标，AV/native分别按已冻结失败码和责任核 | T-02/03/05；失败fatal不强求live0，不以没崩溃作oracle |
| V-11 | AC-03 方法前端/候选 | 同一type两方法+同名异type+自由caller；一个方法body连续变长42→123→7、含UTF-8注释/字符串及花括号；拒绝插入/接口mode编辑/owner字段变更 | body道实际只parse/recheck/emit一个方法，整type/模块前端0；兄弟HIR/producer/code身份保留，后续坐标正确；接口编辑明确走失效道且不复用错类型。compile/admit/link/run失败、cancel、foreign/stale保原source/env/root/revision/坐标及旧值可运行；close后候选拒绝发布，关闭环境不承诺新执行、外pin寿命按V-12；失败实际生产仍计量 | T-02先探通真单体方法前端，T-04/05；生产计数与提交前后状态直接观察，不能用后端hit替代 |
| V-12 | AC-03/04 快照与866兼容 | AV方法body42→43，旧snapshot/pin保留；失败发布、close、最后pin释放；mn比较旧/新产物及失败发布保旧；默认已D866则跑其实际最小标量在线回归 | AV新snapshot43/旧pin42，close后外pin仍按原合同执行，最后owner释放以Weak/回收证明；mn旧产物仍42、新产物43，失败不覆盖旧产物；866新调用/旧链/失败/有界回收不退化 | T-04/05；不要求native loader/pin/HMR，828方法快照不冒领复合在线；提前插窗时866回归由后owner刷新后完成 |
| V-13 | AC-01/03 调用闭包/覆盖库存 | static/view各有带终止条件的递归方法（基例42，步进计数减1）、方法→自由函数→方法；新旧声明次序与未调用合法方法 | whole/unit真执行42，递归self import和跨方法闭包均被真实link绑定；覆盖清单有旧Spec/永久方法正反case→原源/补例→具体V/结果/owner，删一行/漏callee门必红 | T-01冻结库存，T-03/04/05；有限燃料/截止，不新增无界执行承诺 |

V-09的强制输入形状（拟新增补例，T-02先确认旧profile的源码/HIR合法性，合法域不因旧实现缝而缩水）：

| ID | 独立setup与操作 | 手算oracle/区分点 |
|---|---|---|
| P-01 | record P.v；static mix(a P, mut b P, c int)修改b.v+=c，返回a.v*10+b.v；p.v=7/q.v=2，命名逆序调用c=3,b=q.mut,a=p | 75；caller p=7/q=5；仅b有wb，a/c不能占wb字段；去掉.mut拒绝 |
| P-02 | P的view实例mix(mut q P,d int)修改q.v+=d并返回self.v*10+q.v；receiver p=7，q=2/d=3 | 75，p=7/q=5；self只是view，显式q为mut；将q换成p.mut必须verify.mut-conflict，换回异源q必须通过 |
| P-03 | static eat(tag int,move a Str,move b Str)返回tag+a.len()*10+b.len()；a=“A中”、b=“Z”、tag=21，命名参数按b/tag/a源码序传入 | 42，alloc bytes=4/1（创建时顺序），无额外clone，两个根都移入并恰清理一次；call后读任一根拒use-after-move；同根移两次拒绝；交换a/b为33 |
| P-04 | view实例eat(move a Str,k int,move b Str)返回self.v+a.len()*10+k+b.len()；self.v=17、k=4，其余同P-03，显式实参换命名序 | 42；self偏移+1后a/b模式、caller moved标记及callee清理位置仍正确；有body/body-less分别验；交换a/b为33 |
| P-05 | static mixed(ro P,mut a P,k int,mut b P)，以及以self代ro的view实例版本；ro/self.v=1、a.v=2、b.v=3、k=1；a.v+=k、b.v+=2*k，返回ro/self.v*100+a.v*10+b.v；caller再次读取原对象重算 | callee135且caller重算135，ro/self=1、a=3/b=5；wb仅a/b且不混入ro/k/self，交换wb则caller重算153，即使callee仍135也失败；同根双mut拒绝、异根合法通过 |
| P-06 | text-record域static方法update(mut q P) Str将q.v置5并返回新Str“A中”；P为copyable；caller用返回值.len()和q.v计算q.v*10+len | 52，q回写为5，result f0与wb不互吞；一4B buffer返回交caller、callee不提前free，caller恰一次free；无clone |

V-10中的旧native runtime注入复用782的--rt-fail-nth-alloc/--rt-set-budget/--rt-fail-ledger-nth测试面，AV用已有ExecLimits/CancelFlag及受控分配缝；T-02在冻结件列出实际注入点与观测位置。无对应注入能力须记录实现任务，不得以SKIP或只有exit70关闭资源AC。trace/test-support导入仅作测试观察，不要求本包新增console/host-import语言能力。

871方法覆盖交接必须逐record/text-record分域列出static/view、显式参数模式、拥有/借用返回、字段/临时receiver、前向/递归/互调及非法原例；每行有实际入口/原源hash/独立expected/实测结果，原case优先，补例不能替代原源。core/scalar/text保留非法Method拒绝及原内建面。871接受D881只关闭这些方法行，非方法gap另列producer且不能无人认领后默认绿。

实施角色未指定；最终review在同一树以独立上下文核全部AC/V/SD，自检不替代final review；不自动建agent/chat。review pass显式记录merge-owned门pending。r2新增V-09..13与P-01..06均为强制work oracle；原V-01..08继续有效，汇总绿数不能替代任一必要行。

拟定向命令（新入口T-02/05建立后才可执行）：cargo test --manifest-path experimental/ac-mir-native/Cargo.toml --test record_method_tier；probe新增方法test target按实际workspace调用；powershell -File scripts/verify-method-lowering-881.ps1 -Scope targeted|incremental|compat|acceptance（选择单一值）。旧record_source/record_native、text_record_source/text_record_native用精确target/filter。不运行内嵌整档的旧脚本作为开发门；cargo check按拆分后实际包名auto-lang-core执行，实验workspace独立check。最终review前只跑定向测试，完整档归merge。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | M-01/02/05旧方法逐域支持齐全，原R08=42、771原例/aab与P-01..06、递归互调成立；whole/unit均验；V-01/02/09/13 |
| AC-02 | work | M-03/04混合参数模式、所有权/借用返回及失败责任无漂移；782原例30/57、V-09具体位置oracle、V-10寿命/部分回滚与拒绝门成立；V-03/04/06/09/10 |
| AC-03 | work | M-06方法真单体前端与whole/unit同语义；fresh/hit/restart/body/interface依赖、连续变长坐标/候选事务、未改函数零生产、快照寿命/旧cache重验可证；V-05/11/12/13 |
| AC-04 | work | D864真实基面及S/N/H、866方向性释放接受齐全；唯一lowering/无新opcode反向依赖；42/42/51、预算/性能及已交付866最小标量机制不退化；V-06/07/12 |
| AC-05 | work | 文件化父oracle/负控成立；原R08实测关闭ac/av gap；交871逐record/text-record方法正反库存与残留gap producer，分母/a2r轴不改；V-08/13 |
| AC-06 | review | 独立重建AC-01..05、V-01..13/P-01..06与SD-01..06；审遗漏/隐式复制/变通、warnings/fmt/debug；必要oracle缺失不得pass |
| AC-07 | merge | 最终review后冻结提交裸cargo t+触面tv及experimental必要定向档；失败归因，修复代码后重审受影响面与新冻结门 |
| AC-08 | merge | SD-01..06/索引/ledger沉淀、归档/guard清理及真实revision交871/下游；不宣称871已完成 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续执行授权；只读盘点可提前 | 刷新M-01..06与871逐域方法正反库存；原Spec/永久case/whole/unit/AV/mn/oracle/owner映射；核source分段/保留环境/候选、S/N/H写集及864/866进度，不碰WIP | AC-01/02/04/05 | [ ] |
| T-02 | work | T-01、D864、S/N/H释放接受；默认先接受D866公开交接 | 冻结身份/签名/模式/cache版本、预算/样本、V-09..13及P-01..06原源；先R08+owned static/view探通whole+unit→AV/mn，并含一次实际方法体单体前端编辑/原子发布。没有真前端零重做证据不能进入扩面；失败回本步调整机制，语义未决needs_replan不缩合法面 | AC-01..04 | [ ] |
| T-03 | work | T-02探针通过 | shared whole/single Method、显式参数模式/调用接通，source/resolve/adapter必要候选接口；V-01..04/09/10/13及唯一lowering守卫 | AC-01/02/04 | [ ] |
| T-04 | work | T-03、必要窗口接受 | data/owned方法分段/owner环境、模式/指纹、坐标/候选提交和cache/link；mn必要类型索引/身份；接口编辑/恢复、快照寿命和866已交付机制兼容；V-05/06/11/12/13 | AC-02/03/04 | [ ] |
| T-05 | work | T-04、H释放接受 | 文件化suite/SDK/原R08实测，V-01..13/P-01..06、资源/性能/兼容/实际变异负控；逐域G-COV交接、非方法gap producer、SD-01..06草案；提交实现与本地摘要 | AC-01..05 | [ ] |
| T-06 | review | T-05 | 独立final review绑定修订/代码/依赖，非本轮静态预检 | AC-06 | [ ] |
| T-07 | merge | T-06 pass | 冻结提交裸cargo t+tv及触面定向档；tf主检出到期单实例 | AC-07 | [ ] |
| T-08 | merge | T-07 | SD/索引/ledger、方法证据交871、公共写面交下owner、归档与revision收据 | AC-08 | [ ] |
| T-09 | merge | T-08 | wt-guard确认clean后移除树/分支/无兄弟组，回写9/9与终态 | AC-08 | [ ] |

原始源码/输出/JSON/截图只本地保留attachments，禁止add/commit/push；Markdown摘要记录命令/退出码/数字/路径和hash。不建junction/symlink，不纳入其他会话改动。

## 9. 复审记录

2026-10-29 `stage: new | plan_id: PLAN-881 | plan_revision: 2 | outcome: pass（计划复核/修订静态预检，非实现review） | changed: M-06、§3/§5.1..3、V-09..13/P-01..06、AC-01..06/08、T-01..05、SD-06 | next: T-01只读可并行864；T-02起仍等D864及实际S/N/H释放接受，默认D866后接线；未取得窗口 | authorization: 用户明确要求查漏直接修订，仍只规划`。

r2修正：补真实方法前端快道（owned会话原有methods即禁快道、data仅顶层Fn）；增加显式参数位置/多个move的不对称oracle、borrow返回/部分失败清理；补连续变长坐标、候选事务/快照pin回收；逐域旧方法正反覆盖与递归互调；明确866双向公共接口/标量在线兼容责任。current_step仍0、9项全部未执行。r1记录作为历史保留如下，其静态pass不代表已覆盖r2新增项。

2026-10-29 stage: new | plan_id: PLAN-881 | plan_revision: 1 | outcome: pass（规划静态预检，非实现/性能/review验收） | next: 用户后续指定work后T-01；T-02等D864及S/N/H释放接受，默认866释放后插窗 | authorization: 本轮仅规划 | changed: AC-01..08 / T-01..09 / SD-01..05。

已读Spec/源码/永久用例，核R08及整模块/函数级拒绝；中央取号881。未施工/构建/测试/建树或新会话。871方法producer改指881但仍未交付；执行步骤全部未勾。

## 10. 待澄清事项

- static/view支持面沿771/782，不需重裁；T-02决策为表示/单元身份/签名/资源与恢复机制，等价机制继承原范围授权。
- 864/866释放时点、SDK落点、预算与性能基面由T-01/02核定；日期/规划排序不代表owner已释放，不自动联系其他会话。
- 若方法原合法源与AV冻结资源语义冲突，记录最小源、两侧实测/Spec依据/方案影响，未决不扩面不pass；通用旧域gap继续由871登记另立能力包。
