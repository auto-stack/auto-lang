---
plan_id: PLAN-881
status: drafting
feature_name: 共享 record 方法 lowering 与 AV/native 对等覆盖
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 1
current_step: 0
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-mir/hir-lowering-boundary.md
  - docs/specs/auto-mir/data-place-profile.md
  - docs/specs/auto-mir/owned-access-profile.md
  - docs/specs/auto-mir/native-data-owned-lowering.md
  - docs/specs/auto-test-case/ace-m1-conformance.md
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
| S共享lowering | experimental/auto-mir-lower/src/{hir_adapter,lib}.rs；ac-core/src/source/**、hir.rs、verify.rs只修必要缺口（前端已支持方法，不预设重写） |
| S会话/单元 | experimental/ac-av-probe/probe/src/{data_session,owned_session,function_queries,cache,fn_hir_codec}.rs；T-01沿真实调用图确定必要写集；core/src/**不新增指令/执行语义 |
| N | experimental/ac-mir-native/src/{incremental,lower,emit}.rs及tests；仅方法消费必要适配，不独占整个N |
| H | crates/auto-lang-core/src/test_cases/{ac_av,ac_mn,ac_cte}.rs身份/适配点；crates/auto-lang/src/tests/ace_m1_tests.rs；crates/auto-lang/test/cases/ace-m1/binding.tsv、SDK/worker/公共注册（以迁移后实际路径为准） |
| 拟新增 | experimental/ac-mir-native/tests/record_method_tier.rs、probe/tests方法与增量定向文件、文件化TestCase suite、scripts/verify-method-lowering-881.ps1；当前均未实现 |

复用既有case worker、contained executor、text runtime，不另造框架。唯一拟实施树D:/autostack/.wt/lang-881/auto-lang，分支plan-881-dev；当前未建树。实施/复审/合入同树顺序执行，主检出仅计划簿记。

## 4. 需求分析与背景调查

授权：用户2026-10-29要求立共享方法lowering计划、核清871旧方法缺口、明确864交接；仅规划，未授权本轮施工。允许Plan/规划报告/中央取号簿记；不写canonical Spec/ledger、不建实施树、不启动或联系其他会话、不构建/测试。

已读AGENTS.md、docs/specs/overview.md。依据现行auto-ac/source-record.md、source-text-record.md，以及auto-mir的overview、hir-lowering-boundary、data-place-profile、owned-access-profile、native-data-owned-lowering；目标态遵循Design43/49与ACE波次合同。调查基面master 62fb3e3f1（调查期间其他会话推进master）；D868=7754c4e4c、D870=bc90ef718已合入，D862已归档。输入hash与分账详见[缺口报告](reports/ace-method-lowering-gap-2026-10-29.md)。work必须刷新实际revision，不消费上游WIP。

| 库存ID | 已观察旧能力/当前缺口 | 本包关闭条件 |
|---|---|---|
| M-01 | 771 static new返回record；R08；lower_checked_data/owned遍历任一Method即lower.method | 真发射声明/调用，含未调用方法控制；R08原源AV/mn=42，771 p01=0 |
| M-02 | HIR view receiver=params[0]，verify有owner/mode门；771 receiver_once_then_args_in_order | 只读借用不复制owner、先求值一次；原trace=aab且exit0；含嵌套字段/temporary |
| M-03 | 782 Tok.new(move Str)、text_len及Token微程序；owned整体拒Method | 拥有构造/返回/借用读全链，原Token=57、资源守恒且无隐式clone |
| M-04 | lower_function_data_unit/owned_unit_with_modes也拒Method；owned_fn_sig_for_def及callee_mut_record_params取模式只匹配Function | whole/unit模式一致，方法显式move/mut参数责任不退化为默认View，receiver不重复 |
| M-05 | cache::split_units用fn:<name>；function_queries的scalar入口仅收Function | 核data/owned真实入口；owner+method稳定身份、同名方法/自由函数不混链、完整依赖与恢复；不能直接放开scalar冒领支持 |

870的文本view eq/ne源级gap、split_units内嵌类型索引、mut资源字段释放与多帧trap边界不等于方法拒绝同一问题。方法所需签名/类型重映射修复归本包；其他旧域gap由871继续具名列账。若方法原合法例触发尚未裁定的资源语义矛盾，T-02给最小反例/差异/方案，未决不得pass，不缩合法面或套NA。

## 5. 详细设计

### 5.1 864及其他owner交接

语义前置为已交付820/821/828/852/862/868/870；D864是本轮共享基面/兼容交接门，不代表用户方法依赖集合语义。现在可只读核查；公共实施必须等864最终review、冻结门禁、合入、Spec及写面释放齐备。

默认新增包插入位置为：864→866→881→（871删除窗，若就绪）→865→863；保持866→865→863相对顺序。这是规划排序，尚无owner释放。若866尚未接线且相关owner登记释放/接受，可在864后提前插881。881未就绪就让窗，不能持窗等条件；871盘点可并行，其公共修改与881互斥。排序不是新增D866语义依赖。

| 范围 | 864/当前owner释放→881接受 | 881释放→871/下一owner接受 |
|---|---|---|
| S源码/HIR/lowering | Plan修订、delivery、collections profile/descriptor/API/公开枚举、lowering版本、限制；刷新已合入基面 | 方法身份/签名/版本及拒绝面；864原L04/L05=42、List<Str>新变体51及profile4兼容 |
| S会话/cache | 函数级接口/codec/恢复、失效键、生产计数/预算 | 方法body/interface依赖及恢复门、未改函数零生产和payload身份、旧cache失效 |
| N | 当前mn owner释放具体文件/符号；与871改动互斥 | 方法调用/布局/ABI实测、N写集、871逐行关闭/残留gap |
| H | SDK迁移owner真实路径；driver/注册/binding窗口 | R08实测、mir-lowering镜像、父侧oracle/反控及分母审计 |

跨S/N/H同批取所需窗口，不持一个等另一个；review/repair期间仍归原owner，不交WIP。T-02在同一实施树接受交接后做首例探针。本轮不改864正文/状态/勾选，冻结件不等于其实现已验收。

### 5.2 语义、身份及准入

- 按名义owner、方法名及已验证Def身份定位单元，不仅用裸方法名/表位置。两个type同名方法、同名自由函数、无关声明插入/重排不能错链。
- static无receiver；view params[0]只读，receiver先于显式eval_args且恰一次；按bindings映射签名不重排求值。self/隐式字段同义；temporary receiver活到调用及合法视图消费结束。
- 沿771/782支持的参数模式/返回型，特别是owned static的move参数和record返回、view参数合法借用返回；禁止Method模式默认化。mut实例/Unit方法仍按旧域拒绝，不因862/864新类型而放开。
- whole与single-function产物均真实verify/admit；错误owner/receiver/签名/绑定受控拒绝。无第二份HIR→MIR，不fallback旧native。
- body-only只重做改变的方法单元，调用者/无关函数生产=0且payload身份保持；结果由新链接改变。签名、模式、owner布局、候选/配置变化按实际依赖失效，不能以调用点不变命中。
- restart decode+verify；旧语义cache真实下一次读取必须miss/stale再重建，不恢复trusted标记。同步mir-lowering三消费者，ace.m1.cert/1精确键集不变。

### 5.3 支持与资源/首例门

保持旧record/text-record合法域，不附加方法数/深度语言上限。T-02从820/821/828/858与current Spec提取值数据/managed/worker/stack/cache预算交集，列来源、单位、阈值、测量入口及边界控制；不自行提高预算。显式clone才复制，借用方法不能偷偷复制receiver资源。

首例即文件化AutoDown/TestCase；fresh/hit/restart/body-edit分别计量端到端周转、峰值/live/cache bytes。P90≤2000ms为现开发期正式门，长期亚秒与P95/P99另报；T-02沿上游冻结代表负载/轮数/冷暖边界。实际parse/resolve/typecheck/lower/verify/encode/store/codegen生产分发与payload身份分别观察；强制全模块生产反控必红。normal/budget/cancel/trap退出分别按冻结责任验证，不能将旧native fatal依赖OS回收解释成live0。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-mir/record-method-lowering.md | 无→static/view、签名/owner/资源及whole/unit方法合同 | 唯一归属/边界 | AC-01..04 |
| SD-02 | modify | docs/specs/auto-mir/data-place-profile.md、owned-access-profile.md | 首批拒Method→旧record/text-record方法共享lowering可用；其他排除保留 | 准入扩面 | AC-01/02 |
| SD-03 | modify | docs/specs/auto-mir/hir-lowering-boundary.md | lowering语义版本、身份镜像与旧cache失效 | 可追溯 | AC-03 |
| SD-04 | modify | docs/specs/auto-mir/native-data-owned-lowering.md | 方法消费/单元适配；非方法gap独立保留 | 不冒领D871 | AC-04/05 |
| SD-05 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 原R08 ac/av实测42与补充文件化方法case；a2r双轴不混 | 真实分母/oracle | AC-05 |

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

实施角色未指定；最终review在同一树以独立上下文核全部AC/V/SD，自检不替代final review；不自动建agent/chat。review pass显式记录merge-owned门pending。

拟定向命令（新入口T-02/05建立后才可执行）：cargo test --manifest-path experimental/ac-mir-native/Cargo.toml --test record_method_tier；probe新增方法test target按实际workspace调用；powershell -File scripts/verify-method-lowering-881.ps1 -Scope targeted|incremental|compat|acceptance（选择单一值）。旧record_source/record_native、text_record_source/text_record_native用精确target/filter。不运行内嵌整档的旧脚本作为开发门；cargo check按拆分后实际包名auto-lang-core执行，实验workspace独立check。最终review前只跑定向测试，完整档归merge。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | M-01/02/05支持齐全，原R08=42、771原例与aab trace成立；owner/参数/声明不能丢，whole/unit均验；V-01/02 |
| AC-02 | work | M-03/04参数模式/资源/验证无漂移；782原例30/57，once-release与反例拒绝；V-03/04/06 |
| AC-03 | work | whole/unit同语义，fresh/hit/restart/body/interface依赖、未改函数零生产、身份/旧cache重验可证；mir-lowering消费者同步；V-05 |
| AC-04 | work | D864真实基面及S/N/H释放接受齐全；唯一lowering、无新opcode/反向依赖；42/42/51兼容、P90≤2000ms与冻结资源门通过；V-06/07 |
| AC-05 | work | 文件化父oracle/负控成立；R08只凭实测关闭ac/av gap；交871逐行方法证据及残留gap owner，原分母/a2r认证轴不改；V-08 |
| AC-06 | review | 独立重建AC-01..05、V-01..08和SD；审遗漏/隐式复制/变通、warnings/fmt/debug；必要oracle缺失不得pass |
| AC-07 | merge | 最终review后冻结提交裸cargo t+触面tv及experimental必要定向档；失败归因，修复代码后重审受影响面与新冻结门 |
| AC-08 | merge | SD-01..05/索引/ledger沉淀、归档/guard清理及真实revision交871/下游；不宣称871已完成 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续执行授权；只读盘点可提前 | 刷新M-01..05、871方法库存：原Spec/永久case/源码/HIR/whole/unit/AV/mn/oracle/owner逐行表；核data/owned真实入口及S/N/H写集/864进度，不碰WIP | AC-01/02/04/05 | [ ] |
| T-02 | work | T-01、D864、S/N/H释放接受 | 同一实施树冻结身份/签名/模式/cache版本、预算/样本与补case；先R08+owned static/view全链探针whole+unit→AV/mn。失败回本步调整机制；语义未决回needs_replan，不缩支持面 | AC-01..04 | [ ] |
| T-03 | work | T-02探针通过 | shared whole/single Method、签名/模式/调用接通；ac-core必要最小适配，V-01..04及唯一lowering守卫 | AC-01/02/04 | [ ] |
| T-04 | work | T-03、必要窗口接受 | probe真实data/owned会话/cache/link与mn消费；修方法必要类型索引/身份问题，接口编辑/恢复/资源/负控；V-05/06 | AC-02/03/04 | [ ] |
| T-05 | work | T-04、H释放接受 | 文件化suite/SDK/原R08实测，V-01..08、资源/周转/兼容/负控；方法G-COV交接、非方法gap owner、SD草案；提交实现与本地证据摘要 | AC-01..05 | [ ] |
| T-06 | review | T-05 | 独立final review绑定修订/代码/依赖，非本轮静态预检 | AC-06 | [ ] |
| T-07 | merge | T-06 pass | 冻结提交裸cargo t+tv及触面定向档；tf主检出到期单实例 | AC-07 | [ ] |
| T-08 | merge | T-07 | SD/索引/ledger、方法证据交871、公共写面交下owner、归档与revision收据 | AC-08 | [ ] |
| T-09 | merge | T-08 | wt-guard确认clean后移除树/分支/无兄弟组，回写9/9与终态 | AC-08 | [ ] |

原始源码/输出/JSON/截图只本地保留attachments，禁止add/commit/push；Markdown摘要记录命令/退出码/数字/路径和hash。不建junction/symlink，不纳入其他会话改动。

## 9. 复审记录

2026-10-29 stage: new | plan_id: PLAN-881 | plan_revision: 1 | outcome: pass（规划静态预检，非实现/性能/review验收） | next: 用户后续指定work后T-01；T-02等D864及S/N/H释放接受，默认866释放后插窗 | authorization: 本轮仅规划 | changed: AC-01..08 / T-01..09 / SD-01..05。

已读Spec/源码/永久用例，核R08及整模块/函数级拒绝；中央取号881。未施工/构建/测试/建树或新会话。871方法producer改指881但仍未交付；执行步骤全部未勾。

## 10. 待澄清事项

- static/view支持面沿771/782，不需重裁；T-02决策为表示/单元身份/签名/资源与恢复机制，等价机制继承原范围授权。
- 864/866释放时点、SDK落点、预算与性能基面由T-01/02核定；日期/规划排序不代表owner已释放，不自动联系其他会话。
- 若方法原合法源与AV冻结资源语义冲突，记录最小源、两侧实测/Spec依据/方案影响，未决不扩面不pass；通用旧域gap继续由871登记另立能力包。
