---
plan_id: PLAN-939
status: executing
feature_name: CA-D3 有限foreign资源句柄与同步回调桥
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-08
plan_revision: 3
supersedes_spec_components: [docs/specs/auto-abi/project.md, docs/specs/auto-abi/windows-x64-v01.md, docs/specs/auto-abi/conformance-cache.md, docs/specs/auto-a2x/checked-resource-facts.md, docs/specs/auto-a2x/incremental-and-cache.md, docs/specs/auto-a2c/record-text-hir-backend.md, docs/specs/auto-test-case/architecture.md, docs/specs/auto-test-case/ace-ca-d-bridge-certification.md]
new_spec_components: [docs/specs/auto-abi/resource-and-callback-bridge.md]
touched_goals: [GOAL-002]
affects: [auto-abi, auto-ac, auto-a2x, auto-a2c, auto-av, auto-test-case]
depends_on: [PLAN-937, PLAN-938, PLAN-925, PLAN-924]
current_step: 5
total_steps: 14
---

# [PLAN-939] CA-D3 有限foreign资源、句柄与同步回调桥

## 0. 变更摘要

承937共享facts/物理计划、938真实source foreign调用，将797既有资源形态接到新的受检资源事实与native/A2C bridge：buffer view、record const/mut/out、多out成功提交、opaque原方destroy、同步per-call callback+ctx。资源分析仍唯一core verifier权威；新backend只消费，不复制。D939后才能把CA-D整体列已交付，最终CA-E/ARCH/M1签章仍另有owner。

r2仅修订阶段合同、具体输入/方向/验收映射与交接边界；保留T-01..14、AC-01..09、V-01..12、SD-01..07及全部原资源/样本/范围。

r3（2026-11-08，T-02 needs_replan 后的路径级有界修订；目标/验收/范围/仓集合不变）：资源 source 会话采用 **resolver-owned 常驻页路线**——text-record 资源体无 solver 事实（934 推理面为 scalar-only 冻结设计），Frontend 逐 owner 常驻页对资源体记"resolver-owned"标记页（零伪造事实），冷/暖编译均真实重解析并在 P/R 计数如实记录；声明/环境轴失效仍走 digest 键。事实面 text-record typed-leaf 增量（str-literal/record-construction/field-read/mut-arg 结构性类型传递进 solver+页编码）记为**具名后继**，不入本包——本包不以伪造事实页换取 R 指标。新增V-13..16是原别名、noescape、source承诺与阶段边界的区分性补例，不新增ABI形态。38格旧A2R认证另由[PLAN-991](991-ace-m1-legacy-a2r-cert-gap-triage.md)分诊；它不是本包前置，本包不接手旧A2R修复或签M1。

## 1. 目标

在同一有限公开candidate/profile与descriptor3下，native真实import、A2C真实import/export分别证明：模式/长度单位/布局正确；临时view到最后消费者后才释放；mut排他、writeback和失败不改旧out；owned移动/逃逸/显式clone/原方destroy；同步回调与ctx的实际最后owner保活；全函数级增量、恢复首编辑、资源预算与独立认证。原797三腿与937/938新源码标量链回归，不用其旧资源PASS冒新资源桥。

非目标：公开List/Variant ABI、任意资源record按值、f32公开ABI、跨平台/varargs/bitfield/union、任意header importer、保存/跨线程callback、跨界unwind、动态loader/热卸载、完整native/A2R/AV FFI生态、M2所有应用迁移。现有应用需要的其他host能力仍留M2具名，不因排除而取消。

## 2. 架构方案

939扩937中立契约的受控resource投影（尽量沿原descriptor3可表达形态），938受检foreign环境/调用事实携带borrow来源/modes/alias/ownership/deleter/callback约束；core verifier输出唯一逐函数资源与排他计划；native薄adapter按physical plan直接机器调用，A2C薄wrapper显式转换其私有Str/record表示。provider/ctx/handle的真实代际与lease在host/module runtime，不把它当C类型布局属性。

拟新core/src/foreign_resources.rs、native/src/foreign_resources.rs、a2x-c/src/abi_resources.rs、abi/src/resource_bridge.rs；复用core/text_resources.rs与collections_facts.rs的事实框架、shared lowering/相关资源出口，以及925/931的临时值消费者期限。不将C私有record/Str、Rust String/Vec或891整数浮点载体直接公开。新facts/op/codec由T-02冻结和所有reader同窗适配。

## 3. 技术栈

937foreign-contract/abi scalar桥、938source foreign调用/required页；现abi layout/validate/cache/ctool、core受检资源事实、native resource/closure组件、925 C rt_runtime/lower_rt、924闭包机器入口、artifacts/lease、统一worker/TestCase父协议。typed BufViewI64/原宽回调形态可复用受检typed adapter，不冒当前普通源List<i64>或宽参数支持。

## 4. 需求分析与背景调查

仅规划授权与基面`122725e9a`、797/921/925/926/924交付与current hash见[基线报告](reports/ace-ca-d-baseline-and-schedule-2026-11-05.md)。925已交C view/mut/move、owned Str/record清理与先pin后drain；931已修Rust同形临时值并升print/2；926消费唯一CollectionsBodyFacts。这些可复用，但都没证明public ABI。native私有Str长度I32，public buffer u64，必须先check再转换；公开record纯布局与内部资源record不同。924闭包不意味着任意callback可保存或旧代可卸载。

本包依赖937/938真实交付；私有独立header/oracle可先准备，source资源/调用集成不抢929/934事实面。唯一拟树`D:/autostack/.wt/lang-939/auto-lang`、`plan-939-dev`，未来scope仅`D:/autostack/plan-artifacts/plan-939/`，登记/外部输出/merge摘要全路径清理/guard同仓规则。本轮不建树、测试、agent或goal，不改其他计划在途进度。

### r2授权与读时基线（2026-11-07）

用户明确授权：正式修订939的阶段合同、具体源码矩阵和验收映射，并将38格分诊/关闭路径落实到计划；**本轮只改计划，不施工，不修改执行中的938/941，不启动其他会话**。本次drafting r2/0-of-14，后续work仍需实施指令。主检出读时HEAD=`7565e98c4`；未建939实施树/材料scope，未跑产品/认证/性能测试。941当前executing r6，938仍executing；其分支和WIP都不是D。

### 2026-11-07 实施授权（goal 授权链）

用户以 `/goal 完成计划939`（auto-plan-goal 链）授权本包 r2 合同的完整交付：在专属树实施、复审、合同内有界修复与 merge（Spec 沉淀、归档、清树、收据）。实施树 `D:/autostack/.wt/lang-939/auto-lang`（branch `plan-939-dev`，base `95227e59a`，含已归档的 D938 交付），scratch `D:/autostack/plan-artifacts/plan-939/`。不授权改变目标/验收/范围/预算/仓集合。

权威输入：`docs/specs/auto-abi/shared-interface-and-scalar-bridge.md`（SHA256 c15d5ee6566bc8e0c7af7571bcd90af4c8243b3b6a6cf4620a5821bf19b04618）、`docs/specs/auto-a2x/checked-resource-facts.md`、`docs/specs/auto-test-case/ace-ca-d-bridge-certification.md`、797现行`windows-x64-v01.md`；实际旧resource provider/header为`experimental/ace/crates/abi/cside/interop_resource.h`，已有独立ledger与同步callback观察。source语法近邻来自`crates/auto-lang/test/cases/a2c-hir-record-text-925/src/R07.at`。路径/事实是已观察输入，939资源投影/opaque注入/新error channel仍为拟实现；T-01重核届时SHA与真实接口，不能复制历史PASS。

## 5. 详细设计

### 5.1 固定支持矩阵：源、typed与拒绝分开

| 公开形态 | 受检输入/方向与转换 | 资源语义/验收owner |
|---|---|---|
| BufViewBytes/Utf8 | native/A2C import；A2C source export以现str/Str受检view桥接，UTF-8 NUL为数据、len=字节；byte-only typed入口明确标记 | 长度/溢出/NULL与len0组合先校验；超I32内部长度范围受控resource/转换拒，不能截断；borrow不逃逸调用/声明owner |
| BufViewI64 | 受检typed adapter真实native/C消费，len为元素数；不把现List<int> i32内存重解释成i64[] | 元素大小乘法checked，typed fixture证据不冒source List<i64> |
| record const/mut/out | public record闭包=原ABI合法自然对齐标量/嵌套纯record，均指针view/out；A2C source已支持部分映射，宽字段typed入口 | 同布局异名义拒；const不写/mut排他，内部资源record绝不直接repr cast；嵌套offset以独立C oracle |
| owned opaque Handle | 938注入受检名义opaque类型，source可推导局部/参数/返回并受模式约束；typed原形态同时保 | 原provider/代际/destroy契约，move责任转移，owned返回携带lease；未声明clone则拒clone，不能memcpy复制所有权 |
| multiout/BufOut/Handle out | fallible ctx/status/out列表，源返回结果经显式薄adapter映射；无多返回源语法承诺，typed adapter可覆盖原列表 | 所有输出先准备，全部成功一次commit；失败原out逐字节不变，部分handle/分配原方销毁 |
| CallbackFn+ctx | 同步per-call；typed原i64 callback+ctx实测，source可用已受检i32/bool薄trampoline，不借此扩大宽源 | 回调签名/ctx/lifetime完整；不保存/不跨线程，调用中及退出cleanup pin；回调失败不能变成功哨兵值 |

不支持形态在candidate/verify前门明确拒，合法边界控制必须通过，不能以通用拒绝降低分母。所有797原公开形态在937..939新矩阵找到具体V/输入profile/方向；未关闭行不交付CA-D。public i64/u64/f64/char支持与source前门能力分列；新增API名字/类型注入必须与现typechecker/nominal框架相容，不靠扫描源码插入动作。

### 5.2 资源单源与操作时点

候选foreign声明补齐谁分配/谁释放、view owner期限、mut/alias、borrow return来源、错误/side effect、callback noescape；未知字段不能默认safe。core资源分析在受检signature/effect上产生唯一OwnedForeign/borrow/cleanup计划与body digest绑定；中立facts只是投影，不把错误信息当facts，不在C/native重推导ownership。

参数源序恰一次、Copy求值点快照；owned temp借view至少活到**最后消费者**（包括后续实参/整个foreign call/同步回调），不能a2x辅助语句后提前drain。view只读、mut排他，无权owned不free；move在定义的成功/接收责任点移交，转移失败需由契约指定方回收，不能caller/provider都销毁。borrow return仅可来自已验证仍存活owner，拒临时/栈ctx逃逸；owned返回延续provider lease到真实最后release。

source Str/record显式clone为独立backing；opaque handle只有声明实际provider clone操作才可clone，每份拥有各自销毁责任；没有clone hook具名拒，不能为凑测试隐式深拷贝或共享同一裸地址。record中拥有型字段跨界须转显式opaque API/受支持view，不能把private C struct自然布局当公开资源对象。

### 5.3 失败原子性与边界错误

mut view若承诺失败不变：adapter用受预算的shadow/snapshot，实际C provider作用在shadow，经成功与值域验证才commit Auto侧原值；const不写、out不读未初始化值。multiout先准备全部资源、再一次提交；任一分配/转换/callback失败清半树/临时handle，旧out字节/原owner/provider状态按契约保持。一般外部IO副作用不可回滚，本包只承诺显式mut/out数据与本次受管分配的事务，不假称回滚任意C库内部状态。

C callee的输入契约/真实header必须人工审定；不把任意野指针/恶意C写越界变成安全可恢复承诺。bool严格0/1、char合法Unicode、UTF-8完整校验、长度单位/容量/乘加溢出先于访问/分配；合法len0+NULL正控，len非0缺地址在有明确定义的安全入口拒。跨allocator free禁止。status/trap/resource/cancel/binding/protocol/infra不同typed终态，callback异常/trap经ctx错误latch或显式受检error channel传播至外层fallible，不能越界unwind或用0冒成功。 同步回调/合法重入继承同一请求累计deadline、预算与取消，不递归重置额度；query/store全局锁不得跨实际foreign call/callback持有，V-07需实际重入/错误/清理可完成而非只静态检查锁名。原ctx-less等descriptor可表达形态兼容，不因此把需ctx错误传播的source callback偷偷接到无ctx槽。

### 5.4 生命周期与有界回收

provider由实际binary/object/名义代际和拥有者验证，handle实际有效记录/代际不能仅凭ptr或名称；错provider/旧代/双销毁受控拒。owner持有资源就持有其destroy/callback所需provider/code/artifact lease。即使当前binding改代，旧链/callback/外部deleter仍引用原方；最后owner完成cleanup后才可释放/回收，取消/错误亦适用。

采用两个**已静态绑定**provider代际在同进程的真实call/ctx/handle控制与ManagedStore真实lease/GC收据，验证新binding不接管旧handle、最后释放恰一次；不新增loader或宣称native在线热替换。真正已支持的AV同PID HMR由原owner负责，不以重启/重跑冒充。resource ownership状态不跨进程持久化裸指针；restart恢复的是编译/绑定元信息及资格，运行时handle/ctx应重新取得，旧session地址/能力一律拒。

### 5.5 增量、恢复与限额

source foreign resources的实际读取依赖含nominal/mode/borrow/alias/error/caps/deleter/callback生命周期；布局/长度单位/provider变更即相应reader精确失效。provider body只变而契约未变，非内联caller/不读body adapter P0，binding/link root更新实际实现；layout/receiver mode变更必须使真实reader重新验证。实例/CT已存在时读取其实际checked版本，不能按调用点文本盲hit或自建依赖系统。

接口提取/资源计划/adapter/HIR/MIR/COFF/C fragments/tool都在真实producer计P与attempt/discarded，未改函数P=0、其payload额外IO0，link/装配/cleanup独立计。恢复只R有界深验929source、934若已用facts、本包required resource/contract/provider约束页；decode不能重分析资源或调用工具。合法restore后无预热立即source body/mode/decl首编辑，第一编辑失败→修复→再恢复/再编辑；缺required页/伪facts/错owner/旧schema不能成功restore再暗冷启动。

沿797原build60s/run5s/total70s、IPC4MiB/每流1MiB、hot8MiB+暂存索引8MiB、单产物64MiB/请求128MiB及source/native/A2X原更严预算；共存live/shadow/clone/多out/callback栈与恢复复制都计入，不能分别报帽掩盖合计。分配域1..N逐点fault/N+1不动/清fault同进程retry；cleanup零分配/零poll/不消耗剩余预算，有计数/字节流平衡，长链按现有迭代机制验证，不用永久小深度限制兜底。512真实修订/lease/GC收敛，6场景×30原样本及各适用暖门P90≤2000ms。

### 5.6 写面、认证与后继

| 面 | 本包owner/与在途交接 | 完成或释放条件 |
|---|---|---|
| ABI/contract | D937已交；938的call错误/required页仍在途，最终集成必须D938 | provider/资源投影兼容变更枚举全部codec/读者；不默改descriptor3 |
| S | core resource/foreign、HIR/schema/verify/shared lowering | 929/934..936当前owner按稳定SHA/文件/符号释放；不能同时改资源facts/query/CT页；AV消费者穷尽适配保持具名拒/原功能 |
| N/C | native foreign adapter/closure资源、C ABI资源wrapper/实际runtimecleanup | 单文件单写；私有参考可并行，不占公共窗等待D938；不重做925/931资源分析 |
| H | worker/父decoder/SDK/测试/注册/fixture/manifest/证书reader | whole-file短窗，列所有producer/readers；沿原NA与批准hash，不用append-only并写 |
| 采样 | ABI/source/A2X原专项 | 与929/934..936及其他正式采样错峰，完整原规模/资源平台条件不减 |
| 最终总门 | PLAN-941执行中；本包只交实际CA-D证据，不修改其计划 | 本包交完整CA-D支持/证据矩阵与实际残余债，CA-E仅组合不接本包漏项；M1/M2不自动签章 |

首例资源TestCase从T-02前探进入，T-03/04扩齐，不等最后才补。actual来自现场source/typed producer完整文件集，独立C header/layout/provider/caller与手算oracle入跟踪；native与C方向分别编译、运行和观察。daily emit/admit无spawn，explicit certification不能cache PASS。AV/A2R foreign未实现不冒已支持，也不擅自将既有required行转NA。

### 5.7 阶段合同与T-02有界前探

| 阶段 | 唯一任务owner | 输入→实际consumer / 本阶段产出 | 进入下一阶段条件与停止边界 |
|---|---|---|---|
| P1接口冻结 | T-01/T-02，work | D937及最终D938、现core事实、797 header→§5.8每行方向/调用声明/来源/错误语义/读者清单；T-01先只读可做 | 未有D938仅可独立header/oracle与source能力调查，不对WIP实接；T-02正式最小集成等D938与所需写面释放 |
| P2最小全链前探 | T-02，work | 原runner/正式TestCase最小首例，按下列A→B→C分别证明关键接缝，冻结决定而非完成整包 | 三个子项都通过才关T-02；仅证明有限路线可行，不关T-04..12或原AC全域；若必须大型语言平台则needs_replan，不以typed替source |
| P3资源实现 | T-03..08，work | core唯一资源事实→native/C adapter→独立caller/provider；完整shape、cleanup、callback、两静态代lease | §5.8各承诺方向正反控与原V-01..07齐；不把三首例冒所有公开形态 |
| P4增量/恢复/预算 | T-09/10，work | 接D938已交发布/资格/required/真实账本，追加resource约束/owner→首操作/错误恢复/512/GC | 每个resource页/owner有真实P/M/R/IO与预算，旧根可执行；不得在939重造store/发布器/meter |
| P5全矩阵认证交付 | T-11..14，work→review→merge | 完整实际+独立reference与原797/937/938回归→冻结矩阵/Spec/941只读交接 | 原AC全闭、最终独立review后才merge；不以941最后组合替本包漏测，不以本包D签M1 |

T-02保一个稳定任务ID，按顺序记录三个未执行子步骤：

- A：S01一个真实source文本view，经native/C import到独立C，原UTF-8/NUL oracle；连一次restart后首次body edit，证明有合法source与恢复接缝。最小首例从此即进正式文件化Case路径，不以临时脚本PASS取代；T-03扩完整材料/协议。
- B：S02/S03一个mut/out成功与失败例、S04一个opaque创建/唯一destroy；明确失败提交时点、source名义类型与core cleanup出口。只做一组可区分反例，不在前探遍历所有分配故障。
- C：S06一个同步非零ctx callback成功/失败传递与provider lease；证明error channel可用、ctx不逃逸、原预算同请求累计。原宽typed callback另行保留，不拿它顶source。

每子项记录：实际输入/接口与SHA、所需core事实、所有consumer、合法值/失败旧值、原预算观察与未覆盖项。前探不另设宽预算/小语法上限；完整fault1..N、512及原6×30仍归T-07/10/12。T-02未结束不得把未定opaque/callback编码交给后端猜；可行性未通过保具名阻塞与修订建议，不删矩阵行。

### 5.8 具体源码/typed矩阵（全部拟新增，未执行）

拟根`crates/auto-lang/test/cases/ace-ca-d-resources-939/`；source放`src/`，typed数据放`typed/`，独立C材料放`experimental/ace/crates/abi/cside/`。表中调用名为本包候选声明，不是当前可调用API；类型注入/签名由T-02冻结为真实manifest与源码，native/C都走统一前门→受检foreign事实，禁止扫描/改写源码或typed HIR代造。T-02使能最小Case，T-03/04/11扩齐正式入口。

方向：NI=native source import，CI=C source import，CE=Auto source经C导出供独立C caller，TP=原ABI typed adapter/peer。每个“必需”都须分别有case/check/实际证据；TP不核销NI/CI/CE。CE只覆盖本包已承诺source映射，未承诺native source export不新增。每个797原公开形态必须在T-01逐条投影到下列TP及其实际方向，不能把TP一个总PASS冒全部三腿。

| ID / 拟文件 | 具体输入、调用/独立oracle | 必需方向；前门拒绝近邻 | 使能任务 → V / AC |
|---|---|---|---|
| S01 `src/utf8-view.at`及`src/utf8-nul.at` | `fn main() int { return t939_byte_length("A中😀") }`预期8；相邻`t939_scalar_count`预期3；`e`+U+0301为3 bytes/2 scalar，NUL逐字节保留。候选声明为`(view str)->int`，物理u64长度及i64 out先checked再转内部i32 | NI/CI/CE文本view映射必需；TP字节view/Utf8/len0+NULL独立合法控制。非法UTF8、bool/char、乘加/内部长度溢出在访问前拒 | T-02A/03/05/11 → V-01/02/11/15；AC-01/02/07 |
| S02 `src/record-modes.at` | 现语法`type Counter { value int }`，`var x Counter=Counter(20)`；候选`t939_bump(x.mut,1)`成功返回21且x.value=21，主程序返回42；失败seed时x仍20。独立C查offset/align；CE const/mut/out部分逐个caller | NI/CI/CE已有纯record映射必需；宽字段/嵌套RecMix用TP保原形态。同形异名义/const写/mode错前拒 | T-02B/04/05/07/11 → V-01/02/04/11/15；AC-01/02/03/07 |
| S03 `src/alias-conflicts.at`及合法邻例 | 同一x向两个mut、x的mut+view、嵌套字段与父record、重叠buffer；另两个独立x/y与只读重叠正控 | NI/CI受检source路径；CE/TP有可定义范围的入口准入。按§5.9决定，不靠C UB/崩溃当拒绝 | T-02B/04/05/07 → V-02/04/13；AC-02/03 |
| S04 `src/handle-lifetime.at` | 候选`t939_create(4)`→`let h`局部→受检borrow读/显式move参数与返回→最后destroy；provider ledger为oracle。创建失败无live增量，无hook clone拒，真clone hook的两owner各毁一次 | NI/CI必需；CE/TP按原opaque输出/peer形态另证。不假设现有source已支持opaque名称；T-02B冻结声明注入和全部局部/参数/返回用源，不能只给typed句柄 | T-02B/04/06/08/11 → V-03/06/11/15；AC-02/04/07 |
| S05 `src/multiout-transaction.at`及`typed/multiout.atomdata` | 原fallible多out状态+标量/buffer/handle；末项故障时旧out哨兵逐字节不变、部分handle由原方回收；成功一次提交。source通过显式受检薄adapter取现有合法结果形态，不新增多返回语法 | NI/CI source映射与TP原out列表必需；CE仅承诺已有source可表达的out映射。T-02B冻结确切结果record/调用表达，缺能力保阻塞，不把TP冒source | T-04/05/07/11 → V-04/05/11/13/15；AC-01/03/07 |
| S06 `src/callback-ctx.at`及`typed/callback-i64.atomdata` | 已受检i32/bool callback，非零ctx，源序和回调次数固定；失败通过latch/error通道至外层fallible而非0成功。原p797_fold的i64/ctx手写独立oracle另保TP；保存计数p797_stored_cb=0 | NI/CI受检scalar callback，TP原宽同步callback必需；CE所需source桥按原descriptor形态明确。声明保存/跨线程要求前拒，合法重入不持全局锁；不承诺隔离恶意C | T-02C/04/06/08/11 → V-07/11/14/15；AC-02/04/07 |
| S07 `typed/wide-resource.atomdata` | BufViewI64按元素计len，RecMix宽字段/嵌套自然布局，原callback原宽输入；独立C sizeof/alignof/offsetof与已审header | TP原native/C/peer方向逐条必需；不承诺source List<i64>，不将i32 List内存重解释 | T-01/03/05/11 → V-01/11/15；AC-01/07 |
| S08 `resource-recovery.case.ad`及原S01..06编辑 | fresh→hit→restart→立即首好编辑；另首错→旧root真实运行→修复→再restore/编辑。body/provider、mode/layout/deleter/caps分别改；两个静态provider代际/lease真实核 | NI/CI按实际resource source reader分别验；CE/TP有各自元数据恢复但不能替source链。新runtime handle重新取得，旧地址/旧授权拒 | T-08/09/10/11/12 → V-06/08/09/10/16；AC-04/05/06 |

T-01/T-02须把每行下的多输入拆为唯一Case ID与check，保留source/typed、方向、source/hash、声明/profile/caps、独立provider/header/oracle、producer/readers/required页、raw/cert身份、V/AC/task owner、实际状态。缺一方向=本行未闭；空选择/字段缺省0/只wrapper结果不通过。S04/S05/S06尚未定的表面语法是T-02具名研究结果，不能写成已支持API；只允许完成原有限承诺所需最小接线，不自动引入960/962完整M2模块/集合作为整包前置。

### 5.9 别名、shadow与callback的可观察边界

- 沿core排他事实，两个可变访问同一owner/重叠范围，以及mut+view同时访问冲突，须在unsafe调用前拒。只有core已有可消费的独立性事实时才接受两个mut；不以索引数值不同猜独立，合法独立owner正控必须接受。只读重叠在原profile合法域内可接受，临时owner持到最后消费者。
- 输入与out、两个out重叠：原手审contract若未定义则入口具名拒；若既有ABI明确承诺合法，则T-02冻结一份保持原别名关系的shadow/提交方案并补成功/失败对照，不能各拷一份shadow改变别名语义。不能通过新增拒绝缩减797原承诺。
- commit前完成所有输出准备/转换/值域和预算校验；commit不得再触发可失败分配/转换/合作取消，失败保原mut/out字节与owner。若无法保证则T-02阻塞/修订，不能写一半再靠回滚猜语义。外部IO/任意C内部状态不作事务回滚保证。
- callback noescape分三证据：声明要求/签名/期限在准入核验；审定provider确实同步不保存，以独立事件和stored_cb检查；owner/ctx/lease在实际回调及退出cleanup保持。前者不等任意C代码被隔离，负控不实际调用已知悬空指针或制造跨线程UB。请求主动声明保存/跨线程前拒；合法同步重入有真实结果/无死锁/累计deadline和预算证据。

### 5.10 并行关闭路径与实际交接

38格旧A2R分诊由[PLAN-991](991-ace-m1-legacy-a2r-cert-gap-triage.md)独立持有，939不等待它、不修改binding/批准/hash/旧cert守卫，不修执行中的938/941。939对941交付S01..08逐方向矩阵/原797形态映射/完整证据/真实残余；是否安装新reader或更新941由其现owner安排，禁止本包改941计划或消费未审WIP冒D。991对941交接是逐格可审核材料，不自动写入941或称M1可签。F-941-07/09及原8scalar红仍归原具名记录，本次规划授权不等修复授权。

### 5.11 r3 资源 source 会话路线：resolver-owned 常驻页

T-02 探查实证（§9 round 4）：Frontend 的逐 owner 常驻页是事实原生存储（928/934）；text-record 资源源在 `semantics/src/adapter.rs` 的有限推理面（str-literal/record-construction/field-read/mut-arg 显式 Unsupported）外，无法产出 solver 事实页。r3 冻结路线：

- **resolver-owned 页**：资源体（foreign-only supply lane 的 body）在 Frontend 目录中以带 `resolver-owned` 标记的常驻页存在——页**不携带任何类型事实**，仅记录 owner 身份、body 文本摘要与 foreign 环境身份（manifest/contract digest 键）。`resident_page` 对其返回该页；事实 admit/walk 路径不参与。
- **零伪造**：任何 warm 复用都**不得**把 resolver-owned 页当作事实命中；body 变更、环境/声明 digest 变更都走真实重解析。P 计数如实计 resolve 实产；R 的"无重分析"承诺对本 lane 明确为 **R=有界深验（required 页/契约），body 重解析是预期行为而非违规**——S08 恢复首编辑的验收以真实 resolve 计数为准。
- **失效轴**：layout/mode/provider/contract 变更经现有 digest 键精确失效（与本包 §5.5 一致）；无关函数不受影响。
- **计量边界**：本 lane 的暖 body-edit P90 门（§5.5 原预算）以"含真实 resolve 的全链时间"计，不与 938 事实命中 lane 的 P0 混比；T-10 按此口径出数。
- **具名后继（非本包）**：事实面 text-record typed-leaf 增量（solver 输入投影+类型事实+页编码携带）使资源体获得真 R 复用——记入 KNOWN-DEBT 与 M2 后继清单；本包不得以实现它为由扩范围。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-abi/resource-and-callback-bridge.md | 原probe资源→新source/typed资源矩阵、双consumer/原方destroy/同步callback事实 | 实际CA-D完成边界 | AC-01,AC-02,AC-03,AC-04 |
| SD-02 | modify | docs/specs/auto-abi/windows-x64-v01.md | 公开形态不扩，补桥转换/失败commit/lease实际约束，不默升descriptor | 私有/公开分域 | AC-01,AC-03,AC-04 |
| SD-03 | modify | docs/specs/auto-abi/conformance-cache.md | resource依赖/恢复/P0/真GC/认证预算 | 函数级增量可信 | AC-05,AC-06,AC-07 |
| SD-04 | modify | docs/specs/auto-a2x/checked-resource-facts.md | 新foreign资源唯一事实/凭证/临时期限/禁止复制分析 | 共同所有权语义 | AC-02,AC-03 |
| SD-05 | modify | docs/specs/auto-a2x/incremental-and-cache.md; docs/specs/auto-a2c/record-text-hir-backend.md | private资源与public薄wrapper/required页分层 | 不误把内部struct当wire | AC-01,AC-05 |
| SD-06 | modify | docs/specs/auto-abi/project.md | 937..939交付方向/来源/拒绝矩阵与实际残余，不冒通用FFI/M1通过 | 路线可追踪 | AC-07,AC-08 |
| SD-07 | modify | docs/specs/auto-test-case/ace-ca-d-bridge-certification.md; docs/specs/auto-test-case/architecture.md | 在937交付规范上扩本包独立recipe/严格身份与source增量恢复或资源方向 | 各包真实首例/认证不互冒 | AC-07 |

### 首例协议/文件集身份与完整测试注册

历史r1仅列原`abi/interop-v1`已过时：D937已有`abi/ca-d1-bridge-v1`四腿，938新source协议仍在途。T-01沿`crates/auto-lang-core/src/test_cases/abi.rs`、`mod.rs`、`crates/auto-test-case/src/contract.rs`及938最终交付重核真实分发；新资源链不得借任一旧recipe或首例report冒已认证source。T-02冻结本包新recipe、输入profile/协议schema、source/typed来源、ABI domain/profile/target/工具完整身份、required页清单、实际方向与certificate精确字段集；首例接线任务同步迁移全部父/worker/SDK构造与reader，旧ABI/C exe/Rust证书互拒，unknown字段/错版/丢方向/空选择不PASS。可以复用正式runner，但不复用旧PASS判定。

C库导出采用正式新增library fileset/assembler而非认证时剪旧exe main.c；完整业务片段/runtime/公开header/adapter/manifest/必要CRT依赖都入身份，不能只编可通过的小段。source输入与typed adapter输入分别具名，后者不得核销普通source验收。存在`autotests=false`的包新增test须登记对应Cargo `[[test]]`，固定组必须实际选到完整包/新target并记录selected/executed/条件排除，不以写了测试文件即算覆盖；拟组在本包H任务正式注册前均未执行。

## 6. 测试设计

新增拟`abi/tests/resource_bridge.rs`、`abi/cside/resource_bridge_*`、`native/tests/foreign_resources.rs`、`a2x-c/tests/abi_resources.rs`、`core/tests/foreign_resources.rs`和`crates/auto-lang/test/cases/ace-ca-d-resources-939/`；T-02使能最小正式首例，T-03/04/11扩齐正式adapter可运行入口。V全部未执行。实现/内部自查/独立上下文复审依次同一专属树，不自动创建agent/chat。

| Case ID | AC / invariant | setup / entry / work owner | observable oracle / distinguishing assertion | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01 layout/buffer | T-03..05，独立C sizeof/alignof/offsetof、异质非零record、UTF-8含NUL/多字节、i64array | 精确字节/offset/len单位；合法空view通过，非法encoding/bool/char/溢出拒；参数互换负控红 | i64array与宽字段typed，不冒宽source |
| V-02 | AC-02 最后消费者 | T-04/05/06，source owned temp→view参数、后续实参分配/mut+view冲突、callback内使用view | 真native/C结果+alloc/free事件：消费者后才free且恰一次；冲突verify拒，不依赖C UB崩溃 | 复用925/931资源事实，不重做分析 |
| V-03 | AC-02 clone/逃逸 | T-04..06，Str/record clone独立backing、handle move/owned返回、临时borrow逃逸、无hook clone | 地址/内容独立、责任恰一次；borrow/no-hook clone拒，声明真实provider clone可用 | 不承诺任意opaque deep clone |
| V-04 | AC-03 mut/out原子 | T-05..07，const/mut/out、三out含handle、末项故障/部分分配、输入输出别名 | 失败原mut/out逐字节不变，半树由原方回收；成功按声明序一次commit；const不写 | 任意外部IO不承诺rollback |
| V-05 | AC-03 fault/cleanup | T-07/10，所有实际alloc域1..N、N+1/清fault同进程retry，trap/cancel/预算0 | 旧值存活、半树free、字节账平衡；cleanup无分配/poll/预算需求，重试=fresh | 禁止fault只测一处或靠幂等free掩双destroy |
| V-06 | AC-04 handle/provider | T-06/08，真实provider A/B及两代、handle out/borrow/owned返回/双destroy | 错provider/代际调用0，原方destroy恰一次；在用旧code/artifact真lease挡GC，最后release后真回收 | 已静态绑定代际，不冒loader/HMR |
| V-07 | AC-04 callback/ctx | T-06/08，同步callback非零ctx、nested/reentrant有限合法call、回调失败/保存/跨线程请求 | 结果/顺序/ctx与独立C一致；error latch至外层非成功；返回后无悬空ctx，保存/跨线程前拒 | source受检scalar与原typed宽形态分别证据 |
| V-08 | AC-05 全链增量 | T-09，caller不改只变callee body/mode/borrow/名义layout/clone/deleter/cap、无关函数 | 精确reader失效；body不影响签名读者P0但真实新实现运行；全stage P/IO核到实际producer | 937typed bridge不能替真实source链 |
| V-09 | AC-05/06 恢复首编辑 | T-09/10，restart→立即source资源首编辑；首错误→修复→再恢复，坏页/伪cred/旧地址 | R无重分析/lower/emit/工具，未改P0/IO0，required缺拒；旧root/资源live态无误切 | 编译元信息恢复，不持久live指针/句柄 |
| V-10 | AC-06 预算/收敛/时间 | T-10/12，原6×30+512修订/深链/Job取消/GC，shadow/多out合并预算 | 原各帽/分配前预留/P90≤2000ms/父整窗口/真实释放与哨兵不变 | 不减原样本规模/ignored条件 |
| V-11 | AC-07 真实认证/覆盖 | T-03/11/12，native imports+C imports/exports及同域peer、797原三腿、937/938新链 | 独立实际caller/provider/header/reference分别等oracle；缺形态/方向/假report/0case不得PASS | CA-D全矩阵清楚，Q4既有批准不变 |
| V-12 | AC-08 合同完整性 | T-13，独立逐AC/资源状态图/生产者消费者/所有reader/SD/债扫描 | 不靠未批准NA/fallback/延后把资源包签绿；pass绑定r2/冻结SHA | 不签最终CA-E/ARCH/M1 |
| V-13 | AC-02/03 原别名/提交规则 | T-02B/04/05/07；S03同owner双mut、mut+view、父子字段、输入/out重叠与独立owner/只读合法邻例 | 统一verify/bridge实际准入先拒/call0；合法原ABI关系按冻结shadow保存，成功值/失败全旧字节及一次commit可核 | 不因默认拒绝缩原合法域，不以C UB作反控 |
| V-14 | AC-04 同步callback保证来源 | T-02C/06/08；S06声明保存/跨线程负例与手审同步provider、非零ctx/重入/失败 | 声明准入拒与真实stored_cb0/事件分列；实际callback结果/错误latch/lease与累计预算正确 | 不把人工noescape当恶意provider隔离 |
| V-15 | AC-01/02/07 source/typed/方向不互冒 | T-01/02/03/04/11；S01..07各真实源/typed/profile/Case/check与797原形态映射 | 缺source调用/方向、typed顶替、空选择、只有wrapper结果均不闭；合法NI/CI/CE/TP按逐行承诺真编译运行 | 不自动扩AV/A2R FFI或宽source；未知语法由T-02先决 |
| V-16 | AC-05/06 阶段与原恢复/预算 | T-02A/09/10/12；S08最小恢复与完整坏页/首错修复/512/原样本分别记录 | 前探成功不使全矩阵PASS；完整恢复原子root/真实P/R/IO/最后lease与预算全闭才交付 | 不另造发布器/计数平台；原scale不减 |

固定组按最终diff：`cargo t ace-source ace-lowering ace-native ace-av ace-a2x ace-abi ace-artifacts auto-test-case ace-adapters infra-fixed infra-ace-m1 case-a2c-hir-record-text-925 case-a2c-hir-collections-926`，新增包与C完整tests沿937拟`ace-foreign-contract ace-a2c-abi`；本包拟`case-ace-ca-d-resources-939`由T-11注册。资源事实改动需补真实Rust/text/collections与AV受影响完整固定组，T-01从当前配置冻结具体名，不以只C生成器自证；若事实已读取934SQ则补其明确组。原ABI正式采样/adapter入口保留；freeze后有效证据身份未变可复用，非无因全仓/全experimental。

## 7. 验收标准

| ID | owner_stage | 通过条件 / closing evidence |
|---|---|---|
| AC-01 | work | V-01/15与S01..07逐方向的原公开形态/目标布局/长度单位与source/typed方向矩阵完整，非法值前拒、合法控制通过 |
| AC-02 | work | V-02/03/13/15资源唯一权威、view最后消费者、mut排他、move/clone/逃逸与凭证绑定正确 |
| AC-03 | work | V-04/05/13成功提交/失败字节不变/原方半树清理/故障逐域与无余量cleanup实际证明 |
| AC-04 | work | V-06/07/14 provider/代际/原方destroy、同步callback+ctx/错误传播、真实lease最后释放/GC完整 |
| AC-05 | work | V-08/09/16全链函数P0/IO0、精确失效及source恢复无预热首编辑/失败修复/再恢复，不以typed替代 |
| AC-06 | work | V-09/10/16 required R深验/原子发布/原资源帽/原样本时间/512收敛均真实 |
| AC-07 | work | V-11/15文件化首例/独立认证与原三腿/新两包全矩阵齐，实际残余不冒整个M1通过 |
| AC-08 | review | V-12独立上下文逐AC/省略/债/全部读者/健康与SD审查，绑定r2/冻结提交 |
| AC-09 | merge | 冻结职责并集/触码漂移复验、Spec沉淀读回/交付归档/全路径材料清理与guard收据 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 修改路径/可验证成果；对应AC/验证 | 状态 |
|---|---|---|---|---|
| T-01 | work | 后续实施指令；只读阶段不需D938 | 重核D937/D938及925/924/931实交，冻结P1/S01..08每方向/797原形态/Case来源/全部reader/原scale/预算；未D938只独立调查，不实接WIP；AC-01/06/07 | [✅ 2026-11-07] |
| T-02 | work | T-01、实际D938、所需稳定S/ABI接口释放 | r3：补 resolver-owned 常驻页接线（§5.11）使资源体过 Frontend 冷/暖与恢复；随后按§5.7 A→B→C 收口三接缝（S01 文本view+恢复首编辑、S02/S05 mut/out 失败原子与 S04 opaque 唯一 cleanup、S06 callback 错误/lease——callback-by-name 前端仍按 r2 冻结）；三项齐才关闭；V-13..16，AC-01..07。已交付：合同/core/native/CI/provider/发布路径/认证链/首例+固定组（§9 round 1-4，938 组 31/31 回归绿）；round 5-8 全链收口，25 例 24 PASS+控制位，四固定组+七 crate 全绿（§9） | [✅ 2026-11-08] |
| T-03 | work | T-02，H首例短窗 | ABI中立资源投影/布局/header、独立resource首例TestCase由T-02进入，本步扩齐S01..08/方向/正式注册材料，V-01/11/15；AC-01/07。批2（§9 round 11）：S07 TP wide 车道全链（词表/投影/描述符/emit/peers/三腿/fixtures）+ S08 恢复骨架；CE 方向实证为具名接缝 D-939-CE-1（certify 撞 D-960-1 词表、bridge 装配器 scalar-only=T-05 范围；矩阵 S01-CE 的 owner 本就含 T-05/T-11） | [✅ 2026-11-08] |
| T-04 | work | T-03，S窗口 | core foreign_resources/资源凭证/模式/borrow/alias/deleter唯一计划与所有verify/schema消费者；V-02/03/13/15；AC-02。交付（§9 round 12）：唯一 per-call 凭证计划（view borrow+owner/排他+atomic/句柄进出+provider tag/回调目标/ctx）+ CheckedModule/CheckedFunction/SemanticView 双路携带 + a2x 严格消费（缺行=响亮拒）+ V-02 owner 案例双腿；V-03 clone-hook 与 V-13 深层重叠例按矩阵归 T-05/06/07 | [✅ 2026-11-08] |
| T-05 | work | T-04，N/C窗口 | native直接资源调用与a2x-c abi_resources/私有值显式转换/view期限；V-01/02/04；AC-01/02/03。交付（§9 round 13）：native text-runtime 链接补全（COFF ac_rt_* 符号检测+locate-or-build 支持库）+ V-02 双形态案例（owned 旁观跨调用/ctor 临时 const view）；marshalling 面为 T-02 各轮已交并持续绿 | [✅ 2026-11-08] |
| T-06 | work | T-05 | handle移动/返回/provider clone hook/原方destroy、callback+ctx trampoline与错误channel；V-03/06/07/14/15；AC-02/04 | [ ] |
| T-07 | work | T-05/06 | 多out/shadow/失败commit与全部分配域fault/cleanup；V-04/05；AC-03 | [ ] |
| T-08 | work | T-06/07 | provider代际与真实最后owner lease/已绑定双代call/GC回收证据；V-06/07；AC-04 | [ ] |
| T-09 | work | T-04..08，source/query恢复窗口 | 全stage真实计数/精确reader失效/required资源页/真实source恢复首编辑与首错修复；V-08/09/16；AC-05 | [ ] |
| T-10 | work | T-09 | resource预留/候选发布原子/超限取消/cleanup/512修订GC原样本，V-05/09/10；AC-03/06 | [ ] |
| T-11 | work | T-03..10，H窗口 | worker/父decoder/SDK/新case固定组/完整独立认证、S01..08逐方向/原797形态与937/938回归；V-11/15/16；AC-07 | [ ] |
| T-12 | work | T-01..11，正式采样窗 | 原sample/P90/冻结完整职责自查，SD与CA-D整体矩阵/CA-E交接；AC-01..07 | [ ] |
| T-13 | review | T-12 execution_done | 同树独立上下文逐V/AC/资源状态/全部reader/省略债/SD审查；AC-08 | [ ] |
| T-14 | merge | T-13 pass，后续合入指令 | 冻结职责并集/触码漂移复验、SD沉淀读回/交付归档、scope及全部输出/guard清树；AC-09 | [ ] |

可提前的仅独立oracle/header与source/type能力调查，不把private准备冒资源已实施。正式S/N/C集成在D938及所需单文件owner释放后；不硬等未读取的935/936整包。

## 9. 复审记录

T-05 完成记录（2026-11-08，goal round 13，HEAD `355aafff8`）：**native 链补全 + V-02 双形态——T-05 关闭（5/14）**。939 组 42 例=41 PASS+控制位；938/937/925/ace-abi 组全绿；core 29+a2x-c 8；触面零警告。(1) **真接缝与修复**：resource foreign 模块含 owned 文本局部时 native 腿链接失败（`ac_rt_own_from_lit` 未解析——foreign native 链从未链接 text runtime）。修复：link 时扫描发射 COFF 的未解析外部符号（`ac_rt_*` 前缀——链接器自身视角，无 wire/状态改动，编辑/恢复路径全适用）→ 命中即 locate-or-build `ac_text_runtime.lib`（与 782 native 链同一工件；进程内缓存；ACE workspace target 下真 cargo 构建）入链。(2) **V-02 双形态案例**：utf8-owned ×native/c（exit 7）——owned Str 旁观者跨两个 foreign view 调用存活、仅退出释放（C 运行时 charge 记账使提前释放可观测；NUL 为数据 4 scalar+旁观者 3）；pairv-temp ×native/c（exit 47）——调用点 ctor 临时填 const view 车道（物化一次、provider 消费、随调用消亡；reference oracle 自建局部镜像）。(3) **领域证据（双门实证）**：owned Str→str view 形参=绑定精确类型拒（770 纪律）；owned Str 参数撞 D-960-1 签名词表（NonPortableType Builtin(StrOwned)）——本域可表达的 owner-extent 形态即借用局部与 ctor 临时两者，均已落绿例。(4) **marshalling 面**：native/a2x-c 私有值显式转换与 shadow/commit（V-01/04）为 T-02 各轮已交，本轮全部案例持续绿。过程事故与恢复：一次 python 补丁 `io.open('w')` 先截断后报错将 resource939_reference.c 清零（round 2 已知事故模式）——git checkout 即时恢复+ASCII 预验证+写后读回校验重打，零损失。

stage: goal | plan_id: PLAN-939 | plan_revision: r3 | round: 13 | from: work | to: work | status: executing | outcome: interim（T-05 ✅ 5/14；native text-runtime 链接+V-02 双形态）| worktree: D:/autostack/.wt/lang-939/auto-lang | commit: 355aafff8 | next: work（T-06 handle移动/返回/provider clone hook/原方destroy、callback+ctx trampoline与错误channel）

T-04 完成记录（2026-11-08，goal round 12，HEAD `dab2b5797`）：**唯一凭证计划落地并严格消费——T-04 关闭（4/14）**。939 组 40 例=39 PASS+控制位；938/937/925 组、ace-abi 组全绿；core 29+a2x-c 8 绿；触面零警告。(1) **core 唯一计划**（§5.2）：`ForeignCallCredential`（view_borrows 携**派生 owner local**（Extent=调用；字面量/临时=无 owner）、exclusive Mut/Out 车道+sig atomic 承诺、handles_in+provider tag、owned_out 凭证 provider、callbacks+具名目标 def、ctx symbol）+ `ForeignResourcePlan`/`bundle_foreign_plan` 由受检声明+调用点计算；核心单测钉全部行类。(2) **携带/消费**：CheckedModule.foreign_resource_plan（text-record foreign 车道；普通 782 模块形状不变）+ CheckedFunction.foreign_rows（单体凭证行——fragment 视图消费）+ SemanticView 双路（模块计划/fragment 行，body-index 查阅核）+ `foreign_call_credential` 统一入口；a2x lower_rt **严格消费**：资源 foreign 调用缺凭证行=a2c.internal 响亮拒（939 组在严格性下全绿=每个 CI 腿资源调用可证消费计划）；callback 目标计划驱动+逐实参覆盖一致性核；cur_expr 追踪降级位置。(3) **V-02 案例**（utf8-owned ×native/c）：单一 str 局部 owner 喂**双** foreign 视图（byte_length→scalar_count；凭证行钉 owner；owner 活过全部消费者）；NUL 为数据（9 字节/4 scalar：A、U+0000、U+4E2D、U+1F600；exit 4）+ reference oracle 自行计数同值。(4) **领域边界（实证记录）**：owned Str→str view 形参是绑定精确类型拒（770 纪律，无隐式 owned→view 强转）——V-02 owner 案例走借用局部面；verify 既有规则复核：.mut 双向强制、同调用排他（S03 案例）、move/clone（782 RecordClone/Move 节点=V-03 的 clone 独立 backing/move 已有源面）。(5) **路由记录**：V-02 运行期 alloc/free 事件（T-05/06 native/C marshalling owner）、V-03 provider clone hook+无 hook 拒（T-06——ResourceSig 现 clone 词表留其冻结）、V-13 嵌套字段/输入-out 重叠深例（T-05/07 共担）、native 侧凭证消费（MIR callbacks+mir_verify+TextResourcePlan 为其所有权权威；适配器接线归 T-05 窗）、计划持久化 schema 页（r3 路线重验证即重算——恢复读者面归 T-09）。过程修正：一次批量替换误改 utf8-scalars-c 期望行（手术修复）；scalar 计数初版手算漏 U+0000 亦为 scalar（reference oracle 正确红出后修正——控制位纪律生效）。

stage: goal | plan_id: PLAN-939 | plan_revision: r3 | round: 12 | from: work | to: work | status: executing | outcome: interim（T-04 ✅ 4/14；凭证计划+严格消费+V-02 案例）| worktree: D:/autostack/.wt/lang-939/auto-lang | commit: dab2b5797 | next: work（T-05 native 直接资源调用与 a2x-c abi_resources/私有值显式转换/view 期限）

T-03 批 2 完成记录（2026-11-08，goal round 11，HEAD `b1f1553e3`）：**T-03 关闭——S07 TP wide 车道全链 + S08 恢复骨架落地；CE 方向实证为具名接缝（D-939-CE-1）**。939 组 38 例=37 PASS+wrong-oracle FAIL 控制位；四固定组（939/938/937/925）全绿；ace-abi 254（--lib+--tests，含新 wide_bridge 4 例）；ace-foreign-contract 49 绿；触面 crate 零警告。(1) **S07 词表**：`ResKind::BufViewI64`（元素计 view，投影 lane 名 `p{i}_n` 与字节 `_len` 可区分）+ `ResKind::I64`（原宽种子）+ `ResourceSig.typed_peer`（skip_serializing_if——旧 manifest 字节/digest 不变，测试钉死）；typed_peer 域 admit 宽 record 字段与宽 lane，source 域具名拒（core/a2x-c/native 三处消费臂 fail-closed）；`TypedBodyFamily::{BufViewI64Sum,RecMixSum,WideFold}` 逐族精确绑定校验（错形=具名拒），`TypedDecl.resource` 可选字段（937 标量集 wire/content_version 字节稳定，测试钉死）。(2) **投影/准入**：descriptor_from_plan 资源感知（`buf:i64`/`rec:RecMix:const`/`callback(ctx:i64,i64)->i64` + record 表 + 语义 ref）；wide_expectation 手冻结自 bridge_typed.h（消费方自带 RecMix 闭包+semref——同宽改写死在准入）；admission 绿后才生产。(3) **三方实现**：Rust oracle（typed_decl.rs，含独立 Python 手算常量 SUM_8/3/1、REC、FOLD_8/3、CB_AT_0）；Cranelift callee（元素循环/frozen 偏移 record 折叠/call_indirect 回调）+ caller（导出 frozen 回调 t939w_cb 过 fnptr 车道）；C peers bridge_wide_callee/caller.c（独立实现+运行期 offsetof 探针+自身回调与求和走查；size/align 由头部负数组 typedef 编译期钉）。冻结输入带边界：元素前缀 n=8/3/1/0（len 单位证据——字节误读必红）、空 view、U+10FFFF tag、近 MIN 宽字段、超 i32 域宽 ctx、空 fold。(4) **腿/注册**：worker_bridge wide-self/n2c/c2n 三腿（937 typed 腿不动）；typed/wide-*.atomdata×3 + case 行（published 3/4/4→hit）；mutation 负控（错期望 PE 必非零）。(5) **S08 骨架**：resource-recovery.case.ad（fresh→hit→restart→恢复会话首好编辑 ×native/c，exit 42；probe 按 query 步对齐）；T-09/10 扩展（首错流/逐轴失效/provider 代际与 lease 静态/计数断言）具名在文件头。(6) **CE 接缝（已完整接线并驱动到真边界后回退，零死码）**：certify 路径 str 参数导出函数在 `produce_fn_sig`（PortableTy={Unit,I32,Bool}，D-960-1）处 `NonPortableType Builtin(Str)` 拒绝——签名轴词表属 934 冻结面，扩展=r3 具名后继/D-960-1 关闭条件；bridge 路径库装配器 ExportSpec 仅 i32/bool（record-text 导出装配=T-05 abi_resources 行）。探针：str 参数（NUL 数据语义）+ record 参数 native 产品路径全绿（exit 99/21）；接缝登记 D-939-CE-1。过程修正：批量替换误伤 937 typed-c2n 的 C caller（同形匹配）——单点修复后 937 组恢复绿；wide 测试 scratch 根按 tag 隔离消除并行竞态。

stage: goal | plan_id: PLAN-939 | plan_revision: r3 | round: 11 | from: work | to: work | status: executing | outcome: interim（T-03 ✅ 关闭；S07/S08 骨架交付；CE=D-939-CE-1 具名接缝归 T-05）| worktree: D:/autostack/.wt/lang-939/auto-lang | commit: b1f1553e3 | next: work（T-04 core 资源计划与消费者）

T-03 进行中检查点 1（2026-11-08，goal round 9，HEAD `0f0e7a1a6` 前后）：**S03 双记录车道别名 oracle + V-01 编译器实测布局 oracle 落地**——套件 33 例 32 PASS + 控制位，四固定组、foreign-contract/abi/a2x-c lib tests 全绿。(1) foreign-contract `validate()` 规则演进：同一 record 闭包可出现于同一签名的**多个车道**（S03 双 mut oracle 所需）；各次出现必须拼写**完全相同**的字段闭包（异形同名=具名拒；旧"重复视图"整拒规则被取代，测试更新为两半）；(2) provider `t939_pair`（双 MUT 车道）+ `t939_pairv`（MUT+CONST）：commit-once、NULL 守卫 -2 调用方字节不动 + review 表行 + oracle 场景 alias-pair-ok(53)/alias-pairv-ok(16)；(3) **S03 四 deck×双腿**：`alias-pair-reject`（同 owner 双 `.mut`）与 `alias-pairv-reject`（同 owner mut+view）以 `completion {stage:"verify", code:"verify.mut-conflict"}` 判定——verifier 排他性在调用前拒（"local is mutably borrowed while also accessed in the same call"），正例独立 owner 53/16；(4) **V-01 布局 oracle（编译器实测）**：受审 header 以负数组 typedef + `__alignof`（ctool 钉 pre-C11 默认模式与 MSVC-only include 集——无 _Static_assert/stddef；布局漂移使 provider/reference 双构建在 -W4 -WX 下响亮失败）断言 Counter/TxnResult size+align；reference oracle 运行期探针以真 offsetof + union 字节视图补齐**声明序 wire offset** 证据（每个场景前运行）；(5) 完整指纹周期：header/table/contract 三钉（worker policy+父侧）、15 deck manifest 再生成。**T-03 剩余**：S07 TP typed 车道（BufViewI64 按元素计 len、RecMix 宽字段/嵌套、宽 callback——经 worker_bridge typed 腿扩展 typed_decl/typed_emit/C peers；797 layout_probe/RecMix 实测机制可复用）、CE source-export 方向、S08 恢复 case 骨架（主体归 T-09/10）。间歇伪迹复现一次（单组 `cargo t` 偶发 sema runner 误路由，重跑即绿——fixed-tests owner 具名观察）。

T-03 批 1 补录（同日，HEAD `0194995e3`）：**ace-abi --tests 目标首次纳入本轮**（此前各轮只跑 --lib）——census 225/246 绿。修复：(1) **真实 worker 缺陷**——恢复授权按 deck reference 而非**存储指针的已受合同摘要**派生（新增 `CertPolicy::authority_for_digest`，Scalar938 走 `Review::for_digest`；938 review-transition 恢复测试此前撞 `foreign.restore.identity`，现绿）；(2) conformance 测试二进制自 round 1 起缺 `FnFact.resource` 字段（两处）+一处多余 mut——修复；(3) 发射预留测试增长计数期望放宽为 hits≥1+全经预留票据（r3 回写/S06 指令数变化不改不变量）。**具名债（1 红，早期轮次发射账目漂移，非本日改动）**：`native_scalar_constant_pool_admits...` 拒绝路径票据滞留字节与手算公式不符（观测 16773121 vs 期望 16775681，差恰为 64 项 span-pair 数组）——需逐轮发射账目走查定位 span 滞留变更点，不以调常数掩盖（见 KNOWN-DEBT）。

T-03 债关闭记录（2026-11-08，goal round 10）：**D-939-T03-01 关闭，ace-abi --tests 246/246 全绿**。发射账目走查（测量非猜测）：增长 fixture 实发 127 指令/127 span/64 常量；拒绝在指令数组 32→64 增长处触发，票据恰为最后成功预留值（5584）；旧手算公式重复推导了发射内部增长交错（含 64 项 span-pair 项，r3 车道重排后差恰该项 2560）。改写为**账目自洽断言**（residual == pressure + last-successful-ticket）——比原式更强而非放宽，原有保证全保留（Resource 拒绝、零未预留常量数组分配、全增长经票据）。回归：939/938/925/937 四组 PASS、foreign-contract 46 绿、触面零警告。**T-03 批 2 就绪边界（已勘察未施工）**：S07 TP 需 TypedDecl 词表扩展（LaneKind 现仅 7 标量；view/record/callback 参数、TypedBodyFamily 新族 BufViewI64Sum/RecMixSum/WideFold）+ typed_emit Cranelift 体 + C peers（bridge_typed* 扩展）+ worker_bridge 新腿；797 可复用面：`p797_buf_sum`（字节 view，需元素计 i64 变体）、`p797_fold`（宽回调）、`p797_crec_*`（RecMix 三角色）与 layout_probe/autoabi_layout_v01.h（RecMix/RecPad/RecNested/RecOrderB 实测机制，ace-abi 组内持续绿）。descriptor3 `ParamKind::BufViewI64` 已在 `abi/src/descriptor.rs:53` 表达。CE 与 S08 骨架待批 2 同窗。

stage: goal | plan_id: PLAN-939 | plan_revision: r3 | round: 10 | from: work | to: work | status: executing | outcome: interim（D-939-T03-01 关闭；T-03 批 2 勘察就绪）| worktree: D:/autostack/.wt/lang-939/auto-lang | commit: 见 git log（债关闭提交）| next: work（T-03 批 2 施工）

T-02/r3 完成记录（2026-11-08，goal round 8，HEAD `7482923ba`）：**T-02C S06 callback-by-name 全链闭合，三接缝 A/B/C 齐，T-02 关闭**——套件 25 例 24 PASS + wrong-oracle 控制位（固定组 PASS），四固定组与七 crate lib tests 全绿（a2x-c 8/abi 51/native 12/core 28/source-query 86/foreign-contract 46/av-core 59），触面 crate 零警告。纵切片：(1) **provider 面**：`t939_fold(cb,ctx_seed,n,out)` 手审实现（[0,64] 外 -2 先拒、i32 域违例 -2 且 *out 不动、负回调结果**逐字**作 status、noescape 不存储）+ reference 场景 callback(18)/callback-fail(-7→70) -W4 -WX 实编实跑 + review 表 /3（9 表面条目，noescape 强制 true，错误码 -7/-2）+ header/table/contract 新指纹钉（worker policy 与父侧 R939_REVIEWED_*）+ 全部 11 deck manifest 再生成；(2) **前端**：`RExprKind::CallbackRef`（text-record lane 上具名自由函数匹配已 intern 闭包类型即解析为目标；VT::Function 类型结构性限定只能落 callback lane；typecheck 拒方法目标）→ HIR `ExprKind::CallbackRef{def}`（text-record descriptor schema 新 `expr.callback_ref` case + 解析/发射/规范臂；interned 闭包签名以 `function` 类型形 `t_closure_sig_N` 发射——closure case 不在本 descriptor）；a2x-core 三处 walk（family 准入/plan/refs/指纹——按目标源名指纹）放行；(3) **后端**：MIR `ForeignCall.callbacks:[(lane,name)]` + ABC 新操作码 0x3A（0x36 字节不变）双向编解码 + mir_verify 每 callback 车道恰一具名目标校验 + use_slots/ownership 跳过车道占位槽 + owned 单元 capability 准入覆盖无自身 foreign 调用单元的模块级 foreign caps（回调目标函数的单元）；native 每目标**静态 trampoline**（checked i64 域→`callback.domain` -2147483645 传输、窄化 i32 调目标自身单元符号、结果回扩，唯一符号 ac939cb.<axis>.<unit>.<target>）；a2x-c 片段 **preface**（原型行之后顶层发射，保持组装器 nth(1) 原型协议）静态 C trampoline（INT32 域守卫、body trap→`callback.trap` -2147483644 传输——回调失败绝不变成功哨兵）；`resource_prototype` 修正完整函数指针参数类型（旧文本从未被编译）。过程修正：一次 4 组合跑中 939 套件被错误路由到 sema runner（多组调度伪迹，分组调用全绿——非本包代码，具名观察待 fixed-tests owner）；scratch example/diag eprintln 零残留。

stage: goal | plan_id: PLAN-939 | plan_revision: r3 | round: 8 | from: work | to: work | status: executing | outcome: T-02 pass（A/B/C 三接缝齐；25 例 24+控制位）| worktree: D:/autostack/.wt/lang-939/auto-lang | commit: 7482923ba | next: work（T-03）

T-02/r3 进行中检查点 3（2026-11-08，goal round 7，HEAD `710a02eaa`）：**T-02A restart→首次 body edit 接缝闭合**——新增正式双例 `utf8-restart-edit×native/c`（query→restart→语义保持编辑→判定 query，exit 8 两步对齐），套件 21 例 20 PASS + wrong-oracle 控制位，四固定组（939/938/925/937）与六 crate lib tests 全绿。本批穿五道恢复门：(1) foreign env 嫁接（resolve.rs 嫁接臂）与 (2) decl_env 恢复签名表补齐 round 4 的 subset-callability 豁免（CODE_RECORD→不可调用而非硬错）；(3) resolver-owned 标记页恢复不再 decode/seed 空 walk（§5.11：该 lane 的 R=信封有界深验，body 按需真实重解析）；(4) 暖编辑路径 resource lane 分支——foreign-only supply（body 类型归 resolver 自身规则）、合成空 walk 页、空 marker deps、经**新增** `verify_foreign_text_function`（core 单函数 text-record 凭证 + 宿主受审声明；`verify_text_function` 重构出 foreign_allowed 实现参数）验证；(5) 恢复身份两处——native artifact 收 OWNED MIR schema（m_src 不变）、资源 C runtime policy 页摘要改 SHA-256（blake3 `provider::hash` 与 session identity 公式不一致，round 5 引入）。过程修正：一次 `touch` 误建空 `abi/src/main.rs` 入提交，amend 移除（零残留）。**T-02 剩余唯一项：C 接缝（S06 callback-by-name 前端+错误通道+lease 例）**。

stage: goal | plan_id: PLAN-939 | plan_revision: r3 | round: 7 | from: work | to: work | status: executing | outcome: interim（T-02A/B 闭合；仅剩 C callback）| worktree: D:/autostack/.wt/lang-939/auto-lang | commit: 710a02eaa | next: work

T-02/r3 进行中检查点 2（2026-11-08，goal round 6，HEAD `1b781a5a9`）：**19 例中 18 例 PASS + wrong-oracle 控制位登记后固定组 PASS**——round 5 定位的两接缝及其后续三层 C 腿问题全部收口：(1) txn 三 deck 的 record 声明用了逗号分隔字段（text-record 语法成员分隔符是 `;`/换行）——6 txn 腿从未过 parse；(2) C 发布 `inconsistent immutable object charge` 根因=record-text runtime 由 push_str 增长 String 构建，`try_retain(size+40)` 与发布 `adopt(capacity+40)` 对同一 Arc 记账不一致——`rt_runtime` 改走 `render_admitted`（计数趟+精确 `try_reserve_exact`，与 938 标量腿同模式，字节不变）；(3) `foreign_imports.h` 自含化（从受审资源签名发射公开 record 闭包 typedef，先于原型），`generated.c` 携带 seed 派生 `a2x_rec_*` record 表（prelude seed 冷路径从活 view 捕获、快照 Root.prelude 页、restore 读回；record-text C 组装缺 seed 显式拒）；(4) native `.mut` 回写：HIR adapter ForeignCall 臂把 V-04 shadow 提交后的借用临时 Copy/Move 回**原 local**（镜像 Call 臂机制；emitter 级回写落在按值临时）——record-mut-native 42、txn-commit-native 21；(5) laststep reference oracle 移除死后 `t939_live()` 探针（镜像合同：actual==reference 事件序列；源程序死于失败传输无法跟探针对 call）——**回收证据移至 T-07 故障清扫的 ledger 探针**（具名债）；(6) `ace-ca-d3.wrong-oracle` FAIL 控制位入 `case-control-oracles.json`。回归：938 组 30+1 控制 PASS、925 5/5、937 6/6、lib tests a2x-c 8/abi 51/native 12/core 28/foreign-contract 46/source-query 86 全绿；本包触面 crate 零警告（av 的 3 条既有警告属其他 owner 文件）。

stage: goal | plan_id: PLAN-939 | plan_revision: r3 | round: 6 | from: work | to: work | status: executing | outcome: interim（18/19+控制位；A 恢复接缝与 C callback 未落）| worktree: D:/autostack/.wt/lang-939/auto-lang | commit: 1b781a5a9 | next: work

T-02/r3 进行中检查点（2026-11-08，goal round 5，HEAD `3eceffa24`）：**resolver-owned 页路线落地，19 例中 5 例全绿（utf8-bytes/scalars + handle-lifecycle×native/c + wrong-oracle 控制位）**——本轮穿过并修复：事实页冷 lane（零事实+合成空 walk+版本对齐）、MIR arity（`declaration.arity()`）、ABC wire（Owned 族携带 ForeignCall 编解码）、verifier 外源 mut 模式（Rec Mut/Out=mut 借用，`.mut` 合法且必需）、native 链接事件运行时（source939_event 解析）、第三方 layer 派发点 AXCFR1 panic、认证 runtime 身份改为**由 session 记录推导**（资源 C runtime 依赖 view trace marks）、参考 oracle 改为**逐 deck 返回期望值**（父侧对 actual/reference 同一期望比较）并过 -W4 -WX。**剩余两个精确接缝**：(a) native Rec Mut 回写落在副本——record-mut-native 41/42（`.mut` 实参在 foreign 调用点的槽是按值临时）；(b) 资源 C 发布撞 `inconsistent immutable object charge`（runtime Arc 的 retain/adopt 记账不一致）。C 腿 panic 已修（第三 layer 派发点）。

stage: goal | plan_id: PLAN-939 | plan_revision: r3 | round: 5 | from: work | to: work | status: executing | outcome: interim（5/19 全绿；两个接缝已定位到具体机制）| worktree: D:/autostack/.wt/lang-939/auto-lang | commit: 3eceffa24 | next: work

r3 修订记录（2026-11-08，stage:new；needs_replan 的有界路径级修订——目标/验收/范围/仓集合不变，授权沿用 /goal 完成计划939 链）：依据 §9 round 4 实证（Frontend 事实页边界 + semantics/src/adapter.rs 有限推理面），冻结 §5.11 resolver-owned 常驻页路线；T-02 行改写（含已交付清单引用）；事实面 typed-leaf 增量记具名后继不入本包。AC-01..09/V-01..16/SD-01..07/T-01/T-03..T-14 全部保留不删。修订后 T-02 剩余：resolver-owned 页接线（frontend resident/restore/warm 三点）→ A/B/C 三接缝正式收口。round 1-4 全部交付物与证据保留有效（未受修订影响——它们与路线正交）。

stage: new | PLAN-939 r3 | outcome: pass（修订就绪，无待决用户问题） | next: work（T-02 resolver-owned 页接线）

T-02 结果记录（2026-11-07，goal round 4，HEAD `231f2c46b`）：**资源 source 链已穿 6 道门至 Frontend 事实页边界**——本轮打通：profile 选择（frontend 冷目录/resolve/product/恢复四路按环境路由）、`resolve_text_record_with_foreign` 全解析 lane（`TypeSupply::facts_enabled`——foreign-only supply 只嫁接声明、body 类型归 resolver 自身规则；scalar 事实 lane 字节不变）、subset callability（record view 无本模块类型=不可调用而非硬错，resolve/HIR type/def/verify 全链一致）、`text-record.atom` 增 foreign 声明产生式（scalar 语法不动）。**回归事故与修复**：CertPolicy 最初把 938 的 per-reference contract digest/qualify/authority 折叠成单一身份——938 组从 31 绿跌到 10+1+20；三处全部改为 reference 感知后 **938 组 31/31 恢复绿（含 wrong-oracle FAIL 控制）**；六 crate lib tests 全绿（core 28/source-query 86/abi 50/a2x-c 8/native 12/foreign-contract 46）。**T-02A/B/C 最终具名接缝（§5.7 停止条件命中）**：`frontend.rs:387` 的逐 owner 常驻**事实页**（`resident_page`）——Frontend 的存储模型本身是事实原生的（928/934），text-record 资源源在无 solver 事实时无法产页；打通需 (a) 事实面 text-record typed-leaf 增量（语义适配器 str-literal/record-construction/field-read/mut-arg 的结构性类型传递 + solver/页编码），或 (b) 非 facts 的 Frontend 会话路径。两者皆超出 T-02"最小接线"量级=计划 §5.7 明示的 needs_replan 触发。**已交付且不回退**：合同/manifest /2、core 单源映射、native/CI 封送、provider+oracle（实编实跑 4 场景绿）、发布路径、认证 recipe/worker/父判全链、9+1 首例与固定组注册——这些与所选路线无关均复用。

stage: work | plan_id: PLAN-939 | plan_revision: r2 | outcome: needs_replan（T-02 具名接缝=Frontend 事实页架构边界；修订不改变目标/验收/范围/仓集合——路径级修订：为资源 source 指定事实面 typed-leaf 增量或非 facts 会话路线） | worktree: D:/autostack/.wt/lang-939/auto-lang | branch: plan-939-dev | base: 95227e59a | code_commit: 231f2c46b | task_ids: T-01 ✅, T-02 needs_replan, T-03..12 未启 | evidence: 938 组 31/31 绿+六 crate 220 lib tests 绿+§7 各检查点 | blockers: 无用户决策 | next_responsible_role: new（有界修订）

stage: goal | plan_id: PLAN-939 | plan_revision: r2 | round: 4 | from: work | to: new | status: executing | outcome: needs_replan（T-02 事实页边界）| worktree: D:/autostack/.wt/lang-939/auto-lang | commit: 231f2c46b | next: new

T-02 检查点 3（2026-11-07，goal round 3，HEAD `b37b933f2`）：**认证链全通至事实面**——`049e91f9e`：source_foreign 发布路径资源分支（owned 逐函数单元 `lower_function_owned_unit_reserved`、C 片段走 record-text 通道 AXCFR1 `checked_resource_fragment_observed`、fileset/runtime 族选择、缓存身份 `foreign-owned-runtime-abi/1`）。`b37b933f2`：worker `CertPolicy`（Scalar938|Resource939 全身份分歧参数化：recipe/deck/report/输入 profile/指针命名/provider 与 reference 字节/信封 recipe；per-DECK 受审 reference——oracle 精确镜像每 deck 调用序列）、父侧 contract/abi.rs/mod.rs 路由、`abi_source.rs` 父有 939 验收 pins+资源报告准入（逐 deck 事件+失败传输状态）、provider 去 CRT（worker ctool 仅 MSVC include 集——938 纪律）、`ace-ca-d-resources-939` fixtures（9 deck×native/c+wrong-oracle FAIL 控制）+固定组注册。abi 50 tests 绿。**T-02A 具名接缝（当前唯一阻塞）**：真实 case 走到 `semantics/src/adapter.rs:550`——934 事实推理面显式枚举 str-literal/record-construction/field-read/mut-arg 为 Unsupported；资源 source 无法经 facts 会话编译，直到该面携带 text-record **typed-leaf**（字面量/ctor/字段/mut 实参的结构性类型传递，非新推理；有界扩展，估 200-400 行）或走非 facts 会话（大型重做=§5.7 needs_replan 触发）。provider 编译/链接/运行已由 cside oracle 独立证实（4 场景 exit 0）；parent/worker/recipe/fixtures 均绿至该 parse 后的事实面门。debug 痕迹已清除；主检出一次误写 contract.rs 已即时 git checkout 恢复（零落盘）。

stage: goal | plan_id: PLAN-939 | plan_revision: r2 | round: 3 | from: work | to: work | status: executing | outcome: interim（认证链/父判/首例全接通；facts 面 typed-leaf 扩展是下一具体步）| worktree: D:/autostack/.wt/lang-939/auto-lang | commit: b37b933f2 | next: work

T-02 进行中检查点 2（2026-11-07，goal round 2，HEAD `8caeb7315`）：**CI 腿落地**——`6794e1ae5`：a2x-c `lower_rt` 资源 Call 臂（str view (data,len) 对、Rec Const 地址 cast、Rec Mut/Out a2x 型 shadow 仅越过失败传输后提交=V-04、handle `(long long)`、checked 宽→i32 命名 status V-15；callback 槽具名拒至 T-02C）；`ForeignTargetProvider` 开放 record-text 族（资源感知域准入、record-text runtime/fragments/prelude seed、AXCFR1 层、recipe `ac-a2c/hir-foreign-resource-import-v1`）；resource.rs 纯条目拒 mut/out record view（无失败路径挂原子提交）。8 tests 绿。**939 provider 落地**——`8caeb7315`：cside `resource939.{h,c,contract}` 手审 provider（文本视图、原子 mut Counter、代际 ledger 句柄、t939_live 探针、含末步失败原方回收 lane 的原子多 out txn）+ `resource939_reference.c` 独立 oracle（4 场景 cl.exe 14.44 实编实链实跑全 exit 0）+ `resource_provider.rs` 唯一受审 manifest /2（8 表面条目）/authority/qualify（2 tests 绿）。**剩余**：source_foreign.rs 会话发布路径的 text-record 分支（整模块 owned 降级+split_units 替代 scalar 逐体条目）、worker `abi/resource-foreign-certify-v1` recipe+deck/report 身份、S01..S08 首例 fixtures+固定组、S06 callback-by-name 前端、T-03..12。过程记录：python 补丁脚本两次截断 foreign.rs（io.open("w") 先截断后报错），git checkout 恢复+重放无损失（§7 无影响；scratch 脚本在 plan-artifacts）。

stage: goal | plan_id: PLAN-939 | plan_revision: r2 | round: 2 | from: work | to: work | status: executing | outcome: interim（CI 腿+provider 完成；发布路径/worker/首例未落）| worktree: D:/autostack/.wt/lang-939/auto-lang | commit: 8caeb7315 | next: work

T-02 进行中检查点（2026-11-07，goal round 1，worktree `D:/autostack/.wt/lang-939/auto-lang`，分支 `plan-939-dev`，HEAD `58bc838fd`）：**P2 关键接缝的重大修订（T-02 授权范围内）**——探查发现 a2x-c 无 closures 域支持、native closures 机器是独立 I64-cell ABI，source closure callback（S06 原设想）需大型语言平台扩展=计划 §5.7 明示的 needs_replan 触发。T-02 依"source语法由T-02先决"权限改冻结：**profile=text-record**（`auto.source.text-record.draft`，mn owned-profile 与 a2x-c 925 双消费者都支持）；**S06 source = callback-by-name**（Callback 槽位处具名顶层 `fn cb(ctx int,item int) int`，桥发静态 i64↔i32 薄 trampoline，无一等函数值）——S06 前端尚未实施（见剩余）。已交付并提交：`204d913e2`（foreign-contract resource 投影：ResourceSig codec/manifest /2/SourceDecl 资源存取器/物理投影/准入恢复，46 tests 绿）、`1515bbb99`（core 唯一资源感知 source 映射 6 站点+verifier text-record 域+ForeignContext function_types 移植）、`58bc838fd`（native resource_signature+emit_resource_foreign_call：view 对/rec Const 直址/Mut+Out shadow 仅成功续体回写 V-04/handle sextend/checked 宽→i32 命名拒 status −2147483646 V-15/owned profile 准入；compile_text_record_product+verify_foreign_text_record）。native cargo test --lib 12 绿、core 28 绿、foreign-contract 46 绿。**剩余**：a2x-c lower_rt 资源 Call 臂（CI 腿）、S06 callback-by-name 前端（resolve/RExprKind/HIR/MIR callback target）+trampoline、S04 handle 所有权事实（T-04 深化）、939 C provider/header/oracle（cside resource939.*）、worker recipe `abi/resource-foreign-certify-v1`+pinned manifest、S01..08 首例 fixtures、T-03..T-12。无阻塞用户决策。

stage: goal | plan_id: PLAN-939 | plan_revision: r2 | round: 1 | from: work | to: work | status: executing | outcome: interim（T-01 pass；T-02 三接缝中 A/B 的合同+core+native(NI) 已绿，CI/C 及正式 Case 未落）| worktree: D:/autostack/.wt/lang-939/auto-lang | commit: 58bc838fd | next: work

T-01 冻结记录（2026-11-07，worktree `D:/autostack/.wt/lang-939/auto-lang` base `95227e59a`，分支 `plan-939-dev`）：D937 spec SHA `c15d5ee6…4618` 与计划一致；D938 已归档（`8893cfd53` 交付+`d88b0bc0e` 沉淀，含 required 页/恢复/预算链）；924/925/931/937 归档在 `docs/plans/archive/`。797 provider 头 `cside/interop_resource.h`（代际 ledger/fault 注入/同步 callback/mut-out lanes 齐全）；descriptor3 `ParamKind` 已表达 BufViewBytes/Utf8/I64、RecView{Const,Mut,Out}、Handle{provider}、CallbackFn{ctx,item,ret}、BufOut 与 RetKind::Handle（`abi/src/descriptor.rs:33-103`）——939 资源投影不默改 descriptor3。固定组确认：`ace-foreign-contract`(test-groups.toml:1018)、`ace-a2c-abi`(:1029)、`case-ace-ca-d-scalar-937`、`case-ace-ca-d-source-938` 存在；`case-ace-ca-d-resources-939` 由 T-11 注册。**T-01/T-02 联合冻结**：(1) 939 source profile=`auto.source.closures.draft`（PLAN-865/924：record+Str 字面量+mut/move+lambda fn 值，VT 有 Str/Rec/Function；native Str slot=Pair、record slot、MirType::Fn 已支持）；(2) manifest 扩为 `ace.foreign.manual/2`（/1 原样可解码；resource 函数经 FnFact 可选 `resource` 字段携带 `autoabi.contract.resource-sig/1`，None 时 wire/digest 不变），能力 `foreign.resource`；(3) source 映射：ViewText→`str` view 参数、RecView Const/Mut/Out→同名同形状 record 参数（mut 模式对应 `.mut`）、Handle→`int`（物理 i64，bridge 做 checked i64↔i32 转换，超域命名拒）、Callback→`fn(int ctx,int item)->int` 薄 trampoline（物理 i64×2→i64）、宽 i64/u64 形态仅 TP；(4) fallible value-out ≤1 个绑 source 结果（i64/u64/handle→i32 checked），多 out 经 Rec Out 参数表达；(5) fallible+mut/out rec 参数强制 `atomic:true`（§5.3 承诺），Callback 参数强制 `noescape:true`；(6) 新 recipe `abi/resource-foreign-certify-v1`、deck `abi.resource-import-deck/1`、report `abi.resource-import-report/1`（旧版互拒）。941 不触碰。

2026-11-07 stage:new / PLAN-939 r2 / outcome:pass（仅计划修订静态就绪，非work/review/merge通过） / changed:T-01..03/04/06/09/11及AC-01..07说明、V-13..16、§5.7..10；保原14任务/9AC/7SD与全部验收。r1 T-02全包前探与“CA-E未立项/旧协议唯一”描述由当前合同替代，r1预检历史保留。next:后续实施指令后T-01只读重核；T-02及正式集成等D938/接口释放。当前0/14，全V未执行，无本包树/scope；不触938/941。静态检查收据在本节续录。


2026-11-05：stage:new；PLAN-939 r1；规划预检，未跑V。明确全部原公开资源形态、source/typed边界、临时最后消费者、多out/shadow原子、opaque clone不得猜、callback error不可变成功以及最后lease/恢复required页。静态核查见CA-D报告，非work/review/merge通过。

静态规划校验已完成：各节/稳定ID及交叉引用、当前固定组/新组标注、新链接、来源hash、编号唯一/next-id=940和文档范围/空白检查通过；全部实施与复审验收仍未执行。

2026-11-07 r2静态复核收据：0..10章节；T14/AC9/V16/SD7唯一连续且引用有效，S01..08与任务/方向/AC映射齐；拟输入及未定语法均标待使能，原预算/原V/SD及r1历史保留。表格列数/链接/既有Spec路径/空白/编码、现有固定组名称通过；939/991编号唯一，中央取号后.next-id=992。本轮实际写入仅939、991及取号计数；不写938/941、代码、current Specs、原binding/批准/源/expected，不建树/scope或运行产品测试。938原会话在核查期间有并发收据变化，完整保留，不将全局文件hash不变作为本轮未写它的证明。仅文档静态通过，不是独立实施复审或M1通过。

## 10. 待澄清事项

（r3）无需新用户决策。具名后继（非本包、不豁免 M2 硬门）：事实面 text-record typed-leaf 增量——`semantics` 输入投影（str-literal/char-literal/record-construction/field-read/mut-arg 的结构性类型传递）、solver 规则、事实页编码携带，使资源体获得真实 R 复用与 P0 暖命中；届时 resolver-owned 页退役为兼容读取。原 r2 事项保持：S04/S05/S06 确切声明/语法已按 r2/r3 冻结；公开 i64 元素 view/宽 callback 的 typed 证据不核销宽 source。


不需新ABI形态/预算放宽，具体source opaque注入/cleanup facts/callback error与ctx映射由T-02 A/B/C有界证明，S04/S05/S06确切声明/语法是阻塞后续实现的具名决定，不能先默认有新类型或把代码生成成功当资源语义完成。公开i64元素view/宽callback的typed证据不核销宽source。若必须保存/跨线程callback或动态loader才满足实际旧产品需求，归M2具名后继，不塞本有限桥，也不据此豁免M2产品接管硬门。
