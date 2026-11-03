---
plan_id: PLAN-923
status: archived
feature_name: Native分配故障原子性与Place代际正式闭环
author: [Codex]
created_at: 2026-11-03
updated_at: 2026-11-03
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-mir/native-collections-variants.md, docs/specs/auto-ac/source-collections-variants.md, docs/specs/auto-test-case/resource-lifecycle-observations.md, docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-gap-audit.md]
new_spec_components: [docs/specs/auto-mir/native-fault-and-place-conformance.md, docs/specs/auto-test-case/native-resource-scenario-coverage.md]
touched_goals: [GOAL-002]
affects: [auto-ac, auto-mir, auto-test-case]
depends_on: [PLAN-889, PLAN-914, PLAN-915, PLAN-919]
current_step: 9
total_steps: 12
---

# [PLAN-923] Native分配故障原子性与Place代际正式闭环

## 0. 变更摘要

NF包：复用现native集合/runtime与915/919语义/观察基线，补真实逐allocation故障原子性、受检List Place机器链及逐用代际防护。**仅关闭scenario R03 ac**；交付R02非闭包部分与版本化接口，R02 ac保持gap，K02/env故障与完整R02关闭归[924](924-native-closures-source-and-resource-closure.md)。两包无反向依赖，本包可独立交付。

依据[Design43 §21](../design/43-ac-av-typed-mir-and-abc.md#21-native-r02r03r05故障与place基础有限闭包分包2026-11-03target-state)。drafting r1、0/12；本轮只规划，不施工、建树、测试、改变current Spec/原source/expected/批准/登记或921/922进度。

## 1. 目标

G1：原L04/L05/V05、List<Str>/owned record与typed深clone支持域每个真实分配点可注入；源/旧值/责任/epoch不因未提交失败改变，部分输出回收。G2：受检ListPlace/PlaceGet/PlaceSet真正编译成native，stale在危险访问前拒绝，合法近邻可用。G3：函数暖生产0、恢复重验/修复再读、资源/时间门与正式父判可复核，924能消费已交付基础而不重复资源平台。

非目标：native闭包/K02/env、Shared cell/K06、扩源码借用语言或List参数/返回/嵌套、public ABI/FFI、宽typed entry、loader/HMR、关全部R02或签M1。915的AV实现不重写，919事件不复制成新runner。

## 2. 架构方案

原source→现Checked HIR/889共享MIR→native admit/CLIF/COFF→链接同一no_std runtime→真实执行与MN-EVT/新增有界fault/place事实→ac_mn父判。R03源码静态拒绝和运行期typed证人两腿并列；typed MIR须verify_mir和native准入，不能让unsafe probe冒成合法源。

source fault终止/清理与同runtime原语的受控native操作probe互为补充：后者在仍持合法源/目标时比较前后快照并同进程重试。只重跑source不能证明中断原子性；probe不替代原source完整机器链。

## 3. 技术栈

Rust/Cranelift 0.126，Windows x64 COFF/PE；ace-native、collections-runtime及嵌入的text-runtime（保持单panic handler）；受检MIR/现ManagedStore、auto-tool-execution受控子进程与auto-test-case。运行时只做容器/资源原语，不解释MIR/ABC、不调用AV/旧VM。

## 4. 需求分析与背景调查

授权：2026-11-03用户明确为native R02/R03/R05正式立项、核基线并判断拆包，本轮只设计规划。没有实施/构建/认证、跨仓写入/开agent/chat或更改在途合同授权。main读取9c91a1b48；D915=5ba5d4cd2、D919=4ea3daddf已在祖先且archived10/10；919交付后旧required313静态剩R02/R03/R05 ac三格，非本轮正式运行验收。

实施授权：2026-11-03用户指令"完成计划923"＝/auto-plan-goal 全链授权：建树（lang-923 含 auto-down 兄弟，基面 master）、T-01..T-09 实施与自检、T-10 独立复审、受控修复、T-11/12 merge（SD-01..06 落地、archive、清树/scope）。跨仓写入不涉及（affects 均 auto-lang 仓内 crate/experimental）；开agent/chat 未经另行授权不使用。范围/AC/预算不变。

| 实读来源 | 交付事实/内容SHA256 |
|---|---|
| docs/specs/auto-av/resource-fault-and-generation-conformance.md | 1aeb5c68e65813e4671481d9c69b9446c32988c80b9612545e389df0ba82af95；915 AV逐值/arena attempt、运行期stale与读前拒绝，不证明native已支持 |
| docs/specs/auto-test-case/resource-lifecycle-observations.md | 93bf6ea1e0a565c8c0efb40452f7c8b54ba06b11e1249e06c6564609ca835aa9；919 MN-EVT/结束标志/父判/退出knobs，非逐故障/Place新能力 |
| docs/specs/auto-mir/native-collections-variants.md | c54db8ebc2ef6a18b6abaa6eb607bb46ae31c09d95bf0f6c5cd4113c661776b6；已有原子原语、typed Clone、48/32/8MiB与真函数增量/工件恢复 |
| experimental/ace/crates/native/src/collections_lower.rs | f15eeefb029e8316d7b79fd446755a7954f94e322d0f323611e8fc8e71c6022a；只准profile4，Place类型/三指令未准入/发射 |
| experimental/ace/crates/collections-runtime/src/lib.rs | bc2b23df866f1044beaae31fc8a1d7db90b9b7b25769e78d754f9a9f6cb9d8df；Node无结构epoch/elem_gen，allocate/fail_nth、clone_shallow/CloneWork、MN-EVT已有 |
| 同text_alloc与text-runtime::ac_rt_fail_nth_alloc/fail_ledger_nth | text hook直接arena分配；collection fail_nth不覆盖全部text/ledger故障；当前错误映射需核保真 |
| experimental/ace/crates/native/src/collections_driver.rs | a4fdd9dbb3926434f0039b521e5fe49f9504bfbd6abbb0dd02fe9424d55c40ba；接口env明确拒绝，不能拿“closure”依赖闭包术语当语言闭包 |
| 921/922实际树 | 921 executing1/14且A2X有WIP；922已建lang-922树，头drafting0/9不能视作无owner。保全部进度 |

AV915原attempt总数不可复制成native期望。919 Spec的“R02/R03/R05归915/921”等旧owner旁注有滞后，SD只修本包职责范围和后继归属，不重开915/919。原场景oracle/NA/accepted/原源保持。

## 5. 详细设计

### 5.1 逐allocation覆盖与错误分态

T-02先冻结真实可达分配图、各seam的计数定义、输入字节/producer/runtime/tool身份和无注入总数N。至少含node/header/data、frame cells、List push/insert扩容、deep CloneWork、text backing与ledger必要metadata。guest逻辑attempt、runtime bookkeeping allocation、arena页提交与compiler/helper准备分账；不把所有OS分配都伪归同一guest seam，也不能因分类而漏掉可失败的必要运行时分配。text hook走arena的事实必须显式处理，seam不可重叠双计同一指标。

每seam逐1..N、N+1、无注入正控，不能只测第1点；预算precheck attempt0、命中序号、OOM/注入/取消/语言trap/infra分态。注入不是正常语言负例，不能用任意exit70或父kill满足。已实际commit后的错误不谎称未commit。

### 5.2 原子性与清理的两腿证据

保持源/目标logical值、所有权/handle有效性、List len/cap/元素、epoch/gen与旧产物可用；prepare所有fallible工作再commit，未提交move不能kill源。clone/扩容分配旧新同时驻留计预算，失败部分逆序/工作栈清理，drop不轮询已耗尽燃料/取消。

每个源fail点实际PE给具名错误与完整清理；受控native probe同时调用**相同链接runtime原语**，前后由真实动作点输出摘要/责任/代际，失败后关闭fault再在同进程重试正确。probe不是AV模拟或重写操作，不新增guest catch语法，不从请求expected造事实。至少覆盖旧目标已非空的替换、混合多个move/clone输入及深clone中段失败；子树部分成功时仍无泄漏。合法view/外部owner还活时不得清掉backing，最后owner后node/text/arena committed/ledger全归基线。

### 5.3 受检Place与访问前代际校验

既有MIR语义不改：native新增Place有限descriptor/准入与ListPlace/PlaceGet/PlaceSet发射。slot内token使用受控id/代际，不保存会先被解引用的悬空frame/list/buffer指针；先活registry验证frame激活UID、slot/owner、buffer身份、epoch、index、元素gen，再实际访问。具体布局由T-02冻结，所有metadata计预算，计数/UID溢出具名拒绝不能复用旧世代。

本期运行期token证人冻结为List<i32>/List<bool> Copy元素的读写/结构变更；PlaceGet遵循MIR的Copy-only约束，不能读出owned元素。List<Str>/owned record的原只读view/移出与Share/Static生命周期另列合法正控；其不受支持的Place类型在native准入具名拒绝，不因此放开源码跨变更view逃逸或冒全部profile4指令面。

结构变更push/insert/remove按既有契约退役旧token；in-place set/PlaceSet使目标元素旧gen失效，PlaceSet自身旧token与兄弟不可再用。无影响元素/新token与Share/Static StrView正控保持，不能把全部旧view判坏。预算/注入失败未commit时epoch/gen/有效token不变。静态loan违规仍source前门拒绝。

运行期证人复用915对应受检MIR语义，但经过native自己的verify/admit/codegen；合法读→变更→旧token读/写，stale必须在实际buffer read/write前拒。父侧独立核危险访问0，不把“事后trap”当安全；read-first/write-first反控须真实触发同计数器而变红。frame/slot复用/foreign token防御另用明示runtime边界probe，不声称源能合法逃逸token。

### 5.4 增量、恢复、身份与预算

沿CollectionsIncrementalSession::prepare/commit/声明失效，支持body编辑至少三未改函数，真实parse/resolve/typecheck/plan/bind/HIR verify/shared lower/MIR admit/unit codegen/装载及每函数工件IO0，AST/MIR/COFF对象/代际保持；whole前端/whole lower/复制同hash COFF反控必红。callee body与接口依赖分开，声明/layout按914 D-05冷声明前端+依赖精确rebuild，不扩大成body冷fallback。

fresh/hit/restart/body/history与失败→修复→下一次恢复；manifest/源上下文/owner/类型/COFF/库/工具有界重验，工件restore codegen0，不信Checked/PASS/宿主地址。T-02核889工件恢复与**下一body编辑**真实差额；恢复所需有界来源/前端上下文如缺须本包补，不先cold全源求key再报P0/R。M公共source/hash/目录与R decode/deep verify如实分账；cold修复另列，修复后下一恢复真正复用。NF不恢复任何活Place/guest值，运行结果永不缓存。

guest8MiB、native物理32MiB（新代际/token表、text ledger/header/CloneWork同计）、compiler managed48MiB原16/24/8分区、父/PE private增量128MiB；source/metadata4MiB、流1MiB、919 native8192事件行/完整end及原更严门。先限额后物化，观察溢出明确拒绝，不静默截断。116函数/345483B原负载与source hash、512不同修订、fresh/hit/restart/body/history各≥30样本及4096/64KiB小栈等原压力不缩；暖P90≤2000ms，cold/准备/运行/清理/注入整段分列。稳定Auto编辑cargo/rustc0，runtime源码/ABI改动准备低频构建单列。

若Cell/Frame/Node/Place布局或调用约定变化，真实bump ABI/layout/codegen/cache域，不能旧标识解释新bytes。库内容/实际依赖变化正确STALE；不承诺真实runtime改变后旧COFF仍命中。现有源码行为/原recipe不被隐式扩域，观察关闭仍同结果，旧owned/collections/typed-entry所有受影响consumer重验。

### 5.5 正式登记、924交接和921/922写面

拟新recipe ac/mir-native-resource-guards-v1及显式事实版本（T-01核重名）；原source fault对现有collections机器路径，typed Place使用明确受检入口，不能借新recipe绕过gate。新增长期AutoDown在crates/auto-lang/test/cases/ace-m1/native-resource-923/；父侧请求只带操作/seam/index/预算，不带期望N/结果/阶段/事件。合法N/结果由独立reference/冻结父表判，坏来源/工具/产物/词表/重复key/旧operation/截断/重放全非绿。

本包只有R03 ac可正式回填；R02非闭包覆盖摘要、source/typed边界、故障域/版本/单位/实际源与fixture身份、consumer、已验/pending写入新规范/计划，不伪造R02 PASS。924消费正式stable SHA并加K02/env后，重新核对本包在924 runtime版本下的全部非闭包覆盖，再由924唯一回填完整R02与R05。923交付不等待924，不存在循环依赖。

N=collections-runtime/text seam/native collections_lower/layout/driver/observe；先923后924单写，已提交稳定阶段可按符号释放，923最终基面不得夹924 WIP。S=core/source凭证与共享lowering/av-core MIR接口，优先只读复用；确需改出口与921逐符号取窗，禁止新建资源分析。H=ac_mn.rs/其子模块、mod.rs/ac_protocol.rs、auto-test-case/SDK/fixed组/workspace manifest/lock/binding/source-manifest，全文件单写，921在途按owner/实际SHA/字段/schema/construct-match全部consumer/验证/释放登记，漂移STALE受影响证据。无D921整包前置，正式慢测/认证错峰，不持H等编译。

922已有实施树，按在途处理，仅本包报告/接口摘要供其只读消费；不替它做M2全盘审计或改其合同/进度。共享战略/交接文档也按文件窗；915/919落地代码为输入，不重写AV故障/Place/生命周期，不抢A2X provider/store。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-mir/native-fault-and-place-conformance.md | 故障seam/原子性/Place native准入及读前校验 | 真实机器资源基础 | AC-01/02/03 |
| SD-02 | add | docs/specs/auto-test-case/native-resource-scenario-coverage.md | R03全闭/R02非闭包分层与924唯一最终owner | 不拿部分证据勾R02 | AC-04/08 |
| SD-03 | modify | docs/specs/auto-mir/native-collections-variants.md | 新Place/profile/ABI与恢复/计量边界 | 真实依赖/版本 | AC-03/05/06/07 |
| SD-04 | modify | docs/specs/auto-ac/source-collections-variants.md | 原源码语义不变，补native资源/typed边界 | 不开放源码Place逃逸 | AC-02/03 |
| SD-05 | modify | docs/specs/auto-test-case/resource-lifecycle-observations.md | 补fault/place事实、旧919/915边界和新owner | 不沿陈旧915/921归属 | AC-04/08 |
| SD-06 | modify | docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-gap-audit.md | R03 ac由真实父证据关闭，R02/R05留924 | 原批准/分母守恒 | AC-04 |

## 6. 测试设计

现有完整职责组ace-native、ace-native-worker、ace-runtime、ace-artifacts、ace-adapters、auto-test-case、infra-ace-m1、infra-fixed、case-ace-m1-native-collections-889、case-ace-m1-resource-915、case-ace-m1-resource-events-919；触公共MIR/lowering/source补ace-source/ace-semantics/ace-lowering/ace-av与必要真consumer。原宽typed consumer case-ace-m1-typed-native-891在Cell/ABI触面时纳并集。拟新增case-ace-m1-native-resource-923由T-06注册；原性能入口ace-performance保负载/条件，不自动跑全experimental。以下全未执行。

| Case ID | AC | 输入/实际入口与使能任务 | 独立oracle/区分性控制 | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01 | L04/L05/V05/LSTR、typed clone，真实native全source PE，T-02/03 | 各seam全1..N失败、N+1/无注入成功；漏text/ledger/中段即红 | 不含K02/env |
| V-02 | AC-01/02 | 同runtime原语持源/旧目标native probe，T-05 | 每失败前后logical/责任/epoch一致、清fault同进程retry；只重跑source反控不足 | 原语原子性，不添guest catch |
| V-03 | AC-02 | 深clone/扩容中段、活view/pin/取消/预算/注入，T-03/05 | 半树回收/最后owner后live0，预算attempt0与注入序号分态；省drop/提前释放反控红 | 原native资源域 |
| V-04 | AC-03 | verify_mir→native admit的ListPlace读/写/结构变化，T-04 | stale危险读写0，新token/未变元素合法；read-first/write-first控制必红 | 合法typed，不替source |
| V-05 | AC-03 | 静态loan违例、不同frame/slot/gen、失败扩容保token，T-04/05 | 静态/运行拒绝分开；全拒/epoch提前增/ID复用控制红 | Share/Static view不误伤 |
| V-06 | AC-04 | 新AutoDown/真实producer identity、坏事实/旧op/截断，T-06 | 恰R03 ac回填；R02仍gap且非闭包覆盖齐，未知/PASS自述不授信 | 313旧规则不改 |
| V-07 | AC-05 | ≥3未改函数、body/callee/历史回改，T-07 | 真实阶段/IO0与AST/MIR/COFF恒等、whole/换对象非绿 | 889暖支持路径 |
| V-08 | AC-06 | 新进程恢复→body编辑、坏件/readonly→修复→再恢复，T-02/07 | 深验R>0/工件codegen0、暖未改P0；错源/库/凭证/直接deserialize拒 | 无活Place持久化 |
| V-09 | AC-07 | 原116fn/512修订/30样本/原小栈与cap边界，T-08 | 原预算/负载与暖P90≤2000ms、稳定编辑实际Rust进程0 | prepare/fault整段单列 |
| V-10 | AC-04/08 | 原889/915/919/891真实consumer与观察OFF对照，T-09 | 源语义/退出/旧观察可用、ABI错配调用0；runtime变化正确STALE | 不要求旧失效工件命中 |
| V-11 | AC-08 | 本包API/事实规范与924输入清单，T-09 | 有实际SHA/全consumer/原seam义务与明确K02欠项；无循环前置 | 924最终合格仍待实施 |

执行/自查同一唯一树；final review独立上下文从冻结SHA核事实生产→机器动作→父判，不只信勾选；未授权不启动agent/chat，不另造review树。review期间实施暂停，修复回同树。merge依冻结diff一次选完整职责/必要consumer并集并处理漂移；复用身份不变证据，周期cargo t all仅due主检出单实例。915/919原样本/平台/ignore/内存门不削弱，未执行/红/环境重放分列。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01/02全非闭包allocation域/逐点与错误分态通过，无只扫node遗漏text/ledger |
| AC-02 | work | V-02/03真实失败前后源/旧值/责任/代际与部分清理、同进程修复重试通过 |
| AC-03 | work | V-04/05受检Place完整机器链/逐次使用前校验/危险读写0及合法近邻与静态边界通过 |
| AC-04 | work | V-06正式父准入与R03唯一回填，原source/expected/NA/accepted不改，R02非闭包证据不冒全部PASS |
| AC-05 | work | V-07真实函数生产/IO0、对象/代际/依赖失效与反控通过，不whole后filter |
| AC-06 | work | V-08恢复→下一编辑/坏件修复再恢复/readonly/来源重验通过，P/R/M分账 |
| AC-07 | work | V-09原深链/预算/样本/时间门与稳定编辑零Rust构建通过，metadata先限额 |
| AC-08 | work | V-10/11所有真实consumer/ABI/旧域兼容、924稳定接口与单向交接齐 |
| AC-09 | review | 独立冻结review逐AC/V/SD核验，无必需漏项/未批延期，不能要求先等924才判NF交付 |
| AC-10 | merge | 正式落地/SD读回/archive；全实际输出对账/guard清树分支组与整个scope/登记收据 |

## 8. 执行步骤

| T | owner_stage | 前置/真实写面 | 产出与验证入口 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 当前main/921/922窗口 | 核915/919实际SHA、源/三格/全consumer与P/S/N/H、原预算/profile | AC-01..08 | [x] |
| T-02 | work | T-01，私有probe/N短窗 | 最小逐fault+合法typed Place+新进程恢复→下一body编辑whole-path feasibility；冻结seam/N定义/ABI/事实 | AC-01/03/06 | [x] |
| T-03 | work | T-02，collections/text runtime N | 完整非闭包fault/原子清理与分态，V-01/03；cargo t ace-runtime ace-native | AC-01/02 | [x] |
| T-04 | work | T-02，native lower/layout/runtime | Place准入/机器发射/代际前门/合法近邻V-04/05；cargo t ace-native ace-runtime | AC-03 | [x] |
| T-05 | work | T-03/04，probe/codec | 实际原语前后快照/修复重试与读写先行、省free等灵敏度控制V-02..05 | AC-01/02/03 | [x] |
| T-06 | work | T-03..05，H单写窗 | AutoDown/strict父判/拟新组/只回填R03、R02分层事实，V-06；cargo t ace-adapters auto-test-case infra-ace-m1 infra-fixed及新组 | AC-04 | [x] |
| T-07 | work | T-02..06，889 session/store | V-07/08暖零生产、来源恢复/失败修复再读/readonly与负控；cargo t ace-native ace-artifacts | AC-05/06 | [x] |
| T-08 | work | T-03..07，正式采样错峰 | V-09原负载30样本/512修订/深链小栈/预算边界、真实进程计量 | AC-07 | [x] |
| T-09 | work | T-01..08 | V-10/11完整职责自查、SD/原ABIconsumer/924交接实际SHA与残余，全部必需V可执行 | AC-08 | [x] |
| T-10 | review | work完成，冻结SHA | 独立全AC/V/SD复审；修复回同一树，不提前勾merge步骤 | AC-09 | [ ] |
| T-11 | merge | review pass | 冻结职责/consumer并集与漂移重验，合入/SD-01..06读回/archive | AC-10 | [✅ 2026-11-03 merge] 由PLAN-924 merge会话代执行（924 AC-10前置D923；923自身会话中断于review后）。rebase到master（925会话已先落14提交）range-diff 5/5全等价：4af9870e3→f49dbf6bb、13c0312e7→c6e92020b、4ecd23ec9→e839effac、c15f3fa47→2987a03b0、59ef759f7→ddd42ef2b；git merge --ff-only plan-923-dev ✓ master tip=ddd42ef2b（landed）。canonical_verified：native-fault-and-place-conformance.md与native-resource-scenario-coverage.md在主线（c6e92020b），native-collections-variants(4处PLAN-923)/source-collections-variants/resource-lifecycle-observations/ace-m1-conformance/gap-audit修改俱在。merge门：冻结并集于924重放树——case-ace-m1-native-resource-923 PASS、ace-artifacts/adapters/auto-test-case/infra-fixed PASS、ace-native 223测中3个collections_incremental Deadline/DeadlineCollect红=并发PE收集超时环境类（三测串行重放绿2.80s/4.40s，非代码缺陷，按层阶规则环境归因），infra-ace-m1初跑红系924 SD-6改oracles文档致钉哈希漂移（924侧修pin后PASS，本包无涉） |
| T-12 | merge | T-11 | 全输出核查/证据摘要折计划，guard后清树分支组/scope并刷新登记 | AC-10 | [✅ 2026-11-03 merge] 证据摘要全折入本文件§9（2026-10-31裁定，无附件依赖）；wt-guard D:/autostack/.wt/lang-923/auto-lang clean → worktree/分支 plan-923-dev/组目录移除；plan-artifacts/plan-923/整目录删除（无具名hold）；登记表刷新；收据见§9 merge条目 |

拟唯一树D:/autostack/.wt/lang-923/auto-lang、分支plan-923-dev，本轮未建。实施材料唯一D:/autostack/plan-artifacts/plan-923/，开工登记owner/用途/在途；探针源码/产物/日志/外置target/验证副本全部入scope，常规target在专属树，不放attachments或仓内reports，不散落、不建链接。merge整删，仅具名未解决hold可留，禁动他方目录。

## 9. 复审记录

2026-11-03 | stage:new | PLAN-923 r1 | outcome:pass（规划静态准入，非功能/最终review PASS） | 0..10节、T12/AC10/V11/SD6定义/引用/链接锚点/固定组与拟新增组/唯一取号已核；D915/D919祖先exit0。next:work T-01→T-02，依赖ready的私有研究与现native基础可进入后续实施会话，实际S/H取921/922窗。923只关R03、R02最终owner924，无循环；纯故障前后状态probe与source PE两腿、Copy Place/合法owned view边界明确。原两份批准/三份登记hash未变，未建923树/scope、无语言测试/构建/current Spec或在途计划修改。

2026-11-03 | stage:new | PLAN-923 r1 | 起草：先Design43 §21分包，再allocation.lock排他下new-plan.sh真实取923/924，next925。已读实际native准入/Node/allocator，不把915 AV或919观察当native新能力；NF独立交付/R03闭合，R02全部由924收。仅规划，无实施/测试/认证/current Spec/原批准登记或921/922进度变更。静态准入另记，所有功能AC/V待实施。

2026-11-03 | stage:goal | PLAN-923 r1 | round:1 | from:new | to:work | status:drafting | outcome:pass（用户指令"完成计划923"＝全链授权已记§4；lang-923 组已建 auto-lang@ca9bcafb9 plan-923-dev + auto-down@7c4d5cb；scope plan-923 已登记 .retention/policy.json in-flight；D889=50cd6f777/D915=5ba5d4cd2/D919=4ea3daddf/D914=6ffbaf49e 已核） | worktree:D:/autostack/.wt/lang-923/auto-lang | commit:无（实施未开始） | next:work T-01

2026-11-03 | stage:work | PLAN-923 r1 | T-01 完成（基线核验，无代码改动）：
- D915=5ba5d4cd2、D919=4ea3daddf、D914=6ffbaf49e 均在基面 ca9bcafb9 祖先，archived 10/10。D889 delivery 50cd6f777 不在 master 祖先（backup/pre-strip-20261031 保留），但基面 native 三文件 SHA 与 §4 记载逐字节一致：collections_lower=f15eeefb…、collections-runtime/src/lib.rs=bc2b23df…、collections_driver=a4fdd9db…（git show HEAD:… | SHA256）。
- 三格（crates/auto-lang/test/cases/ace-m1/binding.tsv scenario 行）：R01/R04/R06 mn=ok:P919-events（919 已闭）；R02 mn=gap:ac-composite-native、R03 mn=gap:ac-composite-native、R05 mn=gap:ac-composite-native；av 侧 R02/R03=ok:av-resource-915（915 闭 av）。与 §0/§1"仅关 R03 ac、R02 非闭包交付、R05 归 924"一致。
- 固定组（scripts/test-groups.toml）：ace-native、ace-native-worker、ace-runtime、ace-artifacts、ace-adapters、auto-test-case、case-ace-m1-native-collections-889、case-ace-m1-resource-915、case-ace-m1-resource-events-919、case-ace-m1-typed-native-891（宽 typed，Cell/ABI 触面并集）存在；infra-ace-m1/infra-fixed 沿计划 §6。
- recipe 重名：ac/mir-native-resource-guards-v1 全仓唯一，无重名。
- 预算/profile：与 docs/specs/auto-mir/native-collections-variants.md（8MiB guest/MAX_LIST_ELEMS 65536/32MiB native/16-24-8=48MiB managed/128MiB×2）、profile-v2-native.md（30 样本 P90≤2000ms/1MiB 流）、889 归档（116fn/345483B/hash 9fbf6813…/512 修订/五路径各 30/4096 层 64KiB 栈）逐项吻合；919 事件行 8192 上限沿用。
- 921 窗口：lang-921 @ fff3c124（plan-921-dev）executing，WIP 17 文件全属 A2X/A2C 域（a2x-c/a2x-worker/a2c-hir/test-groups.toml/prepare_tools.py）；H 域重叠仅 crates/auto-lang-core/src/test_cases/mod.rs append-only 新增 ac_a2c_hir（A2C_HIR_RECIPE 分支）与 crates/auto-test-case/src/contract.rs 新增 ac-a2c/hir-scalar-emit-v1 recipe（追加式，与 923 拟新增 guards recipe 不同名不同区）。923 写 mod.rs/contract.rs 时基于 master 增量追加，不并入 921 WIP。
- 922：archived（master d66fdef04/593c33480），无 WIP。
- 主检出 WIP：docs/design、docs/plans、docs/roadmap 簿记类（master 零 WIP 代码规则允许）+ 未跟踪 examples/ui/015-notes/、test/fixtures/reader-import-budget/、scripts/tools/native-fixture/Cargo.lock（非 923 写面，不触碰，来源待 owner 自理）。
- H 定位：ac_mn.rs+ac_mn/、mod.rs、ac_protocol.rs 在 crates/auto-lang-core/src/test_cases/（master 上 ac_mn 由 919 更新，protocol 由 896 更新）；source-manifest.json 已登记 scenario_assets_915/919 资产，923 照例登记新资产。

2026-11-03 | stage:work | PLAN-923 r1 | T-02 完成（seam/N 定义/ABI/事实冻结 + whole-path feasibility 探针）：
- 冻结文档 D:/autostack/plan-artifacts/plan-923/t02-design.md SHA256 c0d1f5f5edf4754ff2b548a59f0aef15a28a3d21e9bf4f8431215fc296811484。含 8 seam（S1 node-header/S2 data-buffer/S3 grow-buffer/S4 frame-cells/S5 clone-work/S6 text-backing/S7 text-meta/S8 token-alloc）、互斥/错误分态规则（precheck RESOURCE 与注入 INJECTED 分态、attempts 在 allocate 入口 +1、每 seam 逐 1..N+N+1 正控+清 fault 同进程重试）、ABI bump（runtime /2→/3、lower cell /2→/3、旧 COFF 正确 STALE）、Place token 注册表布局（token_id 不存悬空指针、访问前 node registry 活验证、结构变更 bump epoch、set 废兄弟 place）、恢复→续编辑入口设计（restore_collections_session 复用 restore 校验链+冷前端重建 env）。
- 现状核验：collections-runtime mn_coll_fail_nth 为单一全局 seam（lib.rs:1236）；分配点 S1/S2（new_node 619/623）、S4（frame_open 784）、S5（clone_work 1133）；text 经 text_alloc 走同一 arena（41-46）。MIR 已有完整 Place 语义（av-core mir.rs: MirType::Place tag15、ListPlace 0x2C/PlaceGet 0x2D/PlaceSet 0x2E，553-580），native admit_layouts 白名单无 Place/三指令 → 需准入+发射。restore_collections_inner（collections_driver.rs:677+）已校验 manifest/source/runtime/link/逐 unit MIR 深验/COFF 匹配，但 restore+run 一体无 session（D-03 缺口）。
- 可行性探针（worktree 实跑）：cargo check --manifest-path experimental/ace/Cargo.toml -p ace-native Finished 25.5s 无错误；`cargo test ... -p ace-native --test conformance collections_resources_tier::runtime_ownership_clone_and_failure_atomicity -- --test-threads=1 --exact` exit0 2.24s（fault 原子性基线绿）；`... collections_driver_tier::collections_restore_rejects_all_forged_identity_axes_before_link_or_codegen` exit0 12.47s（restore 链路基线绿）。三链路可行。

2026-11-03 | stage:work | PLAN-923 r1 | T-03 完成（seam 分计数 fault + 原子清理 + V-01；提交待 T-09 后统一整理）：
- collections-runtime（experimental/ace/crates/collections-runtime/src/lib.rs）：SEAM_GLOBAL..SEAM_TOKEN 常量（S0..S8）、State 增 fail_seam/seam_attempts[9]、allocate(bytes, seam) 每 seam 独立计数（全局 seam 0 兼容旧 mn_coll_fail_nth）、新增 mn_coll_fail_nth_seam(seam,n)（重置全部 seam 计数）、注入命中写 MN-COLL-INJECT seam=/nth= 事实行（frame-open 折叠 null 路径时仍可分态）、ABI "mn.collections.runtime/2"→"/3"。S6/S7 由 text-runtime 既有 ac_rt_fail_nth_alloc/ac_rt_fail_ledger_nth 承载（ERR_ENV_OOM→trap 22），wrapper 转发。
- native（collections_lower.rs）：SEAM_* 常量（与 runtime 同步）、WrapperKnobs 增 fail_seam/fail_text_nth/fail_ledger_nth、emit_wrapper_coff 发射 mn_coll_fail_nth_seam + ac_rt_fail_nth_alloc + ac_rt_fail_ledger_nth；collections_runtime.rs ABI /2→"/3"；mn-collections-worker.rs 补字段。
- V-01 新测试 experimental/ace/crates/native/tests/cases/collections_seam_tier.rs（conformance.rs 注册）：per_seam_fault_sweep_l04（S1/S2/S4 逐点+N+1+正控）、per_seam_fault_sweep_grow_seam（6 push 超 cap4 触发 S3 扩容）、per_seam_fault_sweep_lstr（S1/S2/S4+S6 fail_text_nth 逐点+S7 fail_ledger_nth）、typed_list_str_clone_seam_sweep（typed List<Str> 深 clone MIR：S1/S2/S4/S5 逐点）。每注入点断言 trap 码（S4 frame=22 折叠路径、其余 25）+ MN-COLL-INJECT seam 事实行 + 清理 live=0；N+1 与 baseline 同退出码；无注入正控。typed clone 的 ListGet(owned Str) 被 verify 拒（mir.list-type），改用 ListRemove 读元素。
- 验证：cargo t ace-runtime PASS；cargo t ace-native PASS（187/187，31.97s 单组最慢项为既有 performance 测试）。首次 ace-native 4 失败：runtime_managed_pages...（frame_open 注入曾 abort 破坏 889 fixture 期望 null 的 check147——改为 INJECT 事实行+null 返回修复）与 3 个 performance 测试（mir_native_resources_* ABORT 0xc0000409 并行栈溢出、mn_sd911 P90——各隔离 --test-threads=1 重跑绿，环境归因记录，非代码失败）。
- 并行会话事件：工作期间另一会话完成 921 落地归档（master 8717bd5d→ec3e7041，fe0b1e45f archive）+ 922 verification，其 cleanup 使 lang-923 worktree 注册丢失（.git/worktrees/auto-lang1 被删）；已按 auto-down1 元数据格式重建注册（gitdir/commondir/HEAD）+ git reset 重建 index，7 处改动与 HEAD=ca9bcafb9 完整保留并验证。921 已归档，T-01 的"921 在途窗口"事实过期，正式窗口以 master ec3e7041 为准（921 落地含 a2c/A2X 域，与 923 N 域不重叠）。

2026-11-03 | stage:work | PLAN-923 r1 | T-04 完成（Place 准入/机器发射/代际前门 + V-04/05；代码未提交，提交待 T-09 统一整理）：
- runtime（collections-runtime/src/lib.rs）：Node 增 epoch:u64 + gens:*mut u64（随 data buffer 分配，S2 seam 同计）；STALE=-27；PLACE_MARK=2；PlaceToken 注册表（arena S8 seam，next/id/list/data/epoch/index/elem_gen 快照，不存可悬空指针）；mn_coll_place_mint（index 越界 INDEX、S8 走 precheck/allocate/charge）/mn_coll_place_get/mn_coll_place_set（validate→commit→gen+1→release→EMPTY，写入前 validate = write-first）。validate_place 在任何 buffer 解引用前重验 node 活/kind/data 身份/epoch/index 范围/元素 gen，STALE 零读零写。
- lower（collections_lower.rs）：admit_layouts 加 MirType::Place（仅 Copy i32/bool 元素，否则具名 Unsupported("place","must be Copy")）+ ListPlace/PlaceGet/PlaceSet 三指令准入；reads()/dst() 语义；发射 mn_coll_place_mint/get/set（checked）。layout descriptors 加 place/token（(0,0) 无静态绑定）。
- 测试 collections_place_tier.rs（conformance.rs 注册，10 测试含 typed tier 1 个）：
  - place_mint_get_legal_read_and_fresh_token_after_change（合法读=2 + fresh mint 读新值）
  - place_stale_traps_before_any_buffer_access_after_struct_change（push/remove 后旧 token trap 27 + live/bytes/invalid/arena_live 全 0）
  - place_element_replace_retires_own_token_but_keeps_siblings_legal（set 废自身 + 异元素兄弟合法）
  - place_non_copy_element_is_named_rejection_not_silent_widening（List<Str> place 具名拒绝）
  - place_set_commits_and_retires_same_element_sibling（PlaceSet 提交新值 + 同元素兄弟 gen bump 后 trap 27）
  - place_stale_set_traps_before_any_buffer_write（write-first：push 后 PlaceSet trap 27）
  - place_failed_grow_traps_named_injection_and_cleans_tokens（S3 第 2 次 grow 注入 trap 25 + MN-COLL-INJECT seam=3 事实 + token 清理；失败保 token 的原子重试由 probe seam-retry 覆盖——语言 trap 终止 PE 无法在失败后继续）
  - place_cross_slot_siblings_stay_independent（异元素兄弟独立 gen）
  - place_static_verify_rejects_illegal_uses_before_runtime（verify_mir 静态拒 PlaceGet src/PlaceSet place/ListPlace dst 非 Place，与运行期 STALE 分开）
- typed tier：typed_container_signatures_and_places_remain_outside_source_native_abi 更新——Copy place 现为 typed native 表面（emit_unit_coff OK），source ABI 不变。
- ABI：collections_runtime.rs "mn.collections.runtime/2"→"/3"；observe decode trap 白名单 21..=27（含 STALE）。
- 验证：place 10/10 PASS；ace-native 非 performance 188/188 PASS（199.8s）；ace-runtime（ace-text/data-conformance）84/84 PASS；5 个 performance 隔离串行全 PASS（mn_sd911 341s/cold_set 445s/turnaround×3，并行 0xc0000409 栈溢出 + 1MB 分配失败系页面文件 commit 峰值环境归因，非代码失败）。
- fragment probe 适配：runtime 布局新增 gens 数组（cap*8）后 mn_probe_arena 的 4096 上限下初始 3 list 4728 超限——probe 容量重算（cap70/70/0 恰 4096、256 失败、70 复用、coalesce 后 130 放下，Block 头 32B + 16 对齐计入），断言语义全保留（ABI bump 的 affected consumer 重验）。

2026-11-03 | stage:work | PLAN-923 r1 | T-05 完成（同 runtime 原语 probe 前后快照/修复重试 + 灵敏度控制；代码未提交，提交待 T-09 统一整理）：
- collections_runtime_probe.rs 新增 cfg!(mn_probe_seam_retry) 分支：构建 [10,20]（cap1，第 2 个 insert 已 grow 一次）→ mint place@elem1(=20) → fail_nth_seam(SEAM_GROW,1) 注入下一次 grow → insert 失败 INJECTED（check 171）→ 清 fault → 快照不变：len==2、place 仍读 20（epoch/gen 未动，check 172/173）→ 同进程重试同一操作成功 len==3（check 174）→ 结构变更后旧 place STALE（check 175）。
- resources_tier 新测试 runtime_seam_snapshot_and_inprocess_retry_keep_atomicity（probe seam-retry，exit 42 + zero_live）。
- 灵敏度控制（V-02..05）：read-first/write-first 由 place_tier stale 读写 trap 27 且 runtime validate 在任何 buffer 解引用前执行证明；省 free/提前释放反控由每个 trap/inject 用例的 live=0/bytes=0/invalid=0/arena_live=0 断言承担（省略 free 即红）。
- 验证：probe seam-retry PASS（1.8s）；seam_tier 4/4 PASS（l04/lstr/grow/typed clone sweep）；ace-native 非 performance 188/188、ace-runtime 84/84 同 T-04 记录。

2026-11-03 | stage:work | PLAN-923 r1 | T-06 完成（AutoDown/strict 父判/新组/R03 回填/R02 分层事实/V-06；代码未提交，提交待 T-09 统一整理）：
- contract.rs：PROBE_RECIPES 加 "ac/mir-native-resource-guards-v1"；step 词汇表并入 events 分支；mn 绑定必需/归属校验并入；mn.resource 字段扩展 fail_seam([seam 1..=8, nth>=1])/fail_text_nth/fail_ledger_nth；ProbeMnResourceSpec 同步。
- ac_mn.rs：RESOURCE_GUARDS_RECIPE 常量、mod resource_guards、request_recipe/worker_binary_for 分支、请求构建（fail_seam [seam,nth] + text/ledger 通道）、trap_face 并入、Stage::Run 映射并入、parent-side judge 分支（resource_guards::decode_facts_hex/oracle_for/judge_native）。
- ac_mn/resource_guards.rs 新模块：MN-COLL-INJECT seam/nth 严格配对解码（seam 0..=8、nth 纯数字、seam 无 nth 拒）、MN-COLL-TRAP 21..=27/30/31、cleanup=live=0∧bytes=0∧invalid=0∧arena_live=0；frozen 父表（base exits 42/24/51；l04 s1/s2 trap 25+INJECT(1/2,1)；s4 trap 22；grow s3 trap 25+INJECT(3,1)；lstr s6/s7 trap 22 无 INJECT 行——text 通道）；judge_native 独立判 exit/trap/inject/cleanup。
- worker（mn-collections-worker.rs）：RESOURCE_GUARDS_RECIPE 常量 + 请求准入 + fail_seam 数组解析 + fail_text_nth/fail_ledger_nth + 复用 request_resource_events（同 pipeline）。
- mod.rs：RESOURCE_GUARDS_RECIPE 分发。
- AutoDown：crates/auto-lang/test/cases/ace-m1/native-resource-923/（suite.ad + case.ad 8 checks + src/L04-923.at/LSTR-923.at/grow-923.at 逐字节副本）；oracle key 为 case/check 斜杠格式；expect code 用 completion_code 映射（25→mn.resource.allocation-injected、22→mn.resource.execution-budget）。
- test-groups.toml：case-ace-m1-native-resource-923 组。
- binding.tsv：R03 scenario mn 格 gap:ac-composite-native → ok:P923-guards（注释补 923 证据链）；R02 mn 保持 gap + 注释补非闭包覆盖摘要（K02/env 归 924）；corpus R02/R03 行未动。
- source-manifest.json：binding.sha256 重算（5fe87b01…，R03 回填后）；scenario_assets_923 登记 5 文件 hash；plan_923_update 摘要。
- 验证：case-ace-m1-native-resource-923 PASS（新组端到端：base 42/24/51 + 注入 trap）；ace-adapters/auto-test-case/infra-fixed PASS；infra-ace-m1 首跑 7 FAIL 系 binding hash 漂移——更新 manifest binding.sha256 后 PASS。

2026-11-03 | stage:work | PLAN-923 r1 | T-07 完成（restore→session 续编辑 + V-07/08；代码未提交，提交待 T-09 统一整理）：
- collections_driver.rs：restore_collections_inner 拆出共享校验链 restore_collections_material（manifest 身份/runtime build+content/source+coordinates/link tools/逐 unit MIR 深验+COFF hash/len/pages/entry 闭包检查 → RestoredCollections{manifest,source,units,objects,metas,mir_bytes,runtime_identity,runtime_library,runtime_blob,link_identity,entry_symbol}），restore_collections/inner 行为不变；新增 restore_collections_session(id,workdir,support)→(CollectionsIncrementalSession,CollectionRun)：复用校验链 + 恢复 source 冷前端重建 env/regions（有界来源，不 cold 全源求 key）+ 装配 ResidentCollectionUnit（restore 核验 units + 克隆 COFF bytes）+ 重建 store-backed driver（ManagedStore.root）+ finish_collections 执行 restore run（frozen manifest 链）。
- collections_incremental.rs：CollectionsIncrementalSession::from_restored 构造器（id/revision 新分配、env/source/regions/mint_origins/units/driver/runtime/blob/link/entry/workdir/support/manifest_id/fp、last_work 最小默认）；prepare_source/commit_prepared 原样复用。
- V-07 测试 collections_restore_session_takes_next_body_edit_with_real_differential_work：3 函数（callee/helper/main）publish（49）→ fresh driver（forbid_codegen）restore session（49、manifest_id/fingerprints 恒等）→ body 编辑只重建 helper（codegen=1、mir_lower:d_helper=1、无 d_callee/d_main、callee/main fp 不变、helper fp 变）→ 第二编辑只重建 callee（51）→ 回改只重建 helper（50）。
- V-08 测试 collections_restore_session_rejects_bad_identity_and_readonly_then_recovers：forged id 全链拒（Admit collections-artifact）；readonly driver restore→session 可用但 manifest_id=None、编辑后仍 None（不持久化）；同一好 manifest 新 writable driver 恢复→编辑 43。
- 验证：2 新测试 PASS；ace-native 非 performance 190/190 PASS；ace-artifacts cargo t PASS（直接 nextest 3 subproc 失败系未构建 te-child.exe tool，正式入口带 execution-fixture 构建后全绿）。

2026-11-03 | stage:work | PLAN-923 r1 | T-08 完成（V-09 原负载/预算边界/时间门；代码未提交）：
- 原 performance 5 项隔离串行全 PASS（mn_sd911 341s、mir_native_resources_cold_set_churn_gc_convergence 445s、turnaround×3 69-86s）；并行 0xc0000409 栈溢出 + 1MB 分配失败系页面文件 commit 峰值环境归因（页面文件 68GB 峰值满），非代码失败；ace-native 组内 performance::typed_v10 P90 gate 在组跑中 PASS（37s）。
- 原负载/样本/预算边界由既有 889/915/919 performance 测试承载（116fn/512 修订/30 样本/4096 层 64KiB 栈/8MiB guest/32MiB native 物理），T-04/07 的布局改动（gens 数组、token 表、ABI /3）已由 fragment probe 适配 + 全组重验覆盖；暖 P90≤2000ms 由 typed_v10 等 gate 承担。

2026-11-03 | stage:work | PLAN-923 r1 | T-09 完成（V-10/11 完整职责自查 + SD-01..06 落地 + 924 交接；代码已提交）：
- V-10 consumer 对照：case-ace-m1-native-collections-889 首跑 8 checks 全 ERROR "unbound collections runtime content/build identity"——ac_mn/collections.rs:288 硬编码 runtime ABI /2，ABI bump 后改 /3（含 207 行 collections_abi /2→/3 与 440 测试 fixture runtime/3）；另发现 collections_lower.rs cell ABI 未 bump（T-02 要求 /2→/3）——已改 mn.collections.cell/3 并同步 ac_mn 校验/fixture。修复后 889/915/919/891 四组全 PASS；923 组复跑 PASS。观察 OFF 的既有 recipe 行为不变（observe 白名单仅扩 27）。
- SD-01..06 落地（worktree 提交 79feeb3ce）：SD-01 新 docs/specs/auto-mir/native-fault-and-place-conformance.md（seam 表/注入契约/原子清理/Place 代际/ABI 计量/恢复续编辑）；SD-02 新 docs/specs/auto-test-case/native-resource-scenario-coverage.md（R03 关闭/R02 非闭包分层/正式父准入/924 输入清单）；SD-03 native-collections-variants.md（cell/3+runtime/3、Place 链、恢复续编辑节）；SD-04 source-collections-variants.md（PLAN-923 附录：源码不能 mint Place、typed 唯一通道、具名拒绝）；SD-05 resource-lifecycle-observations.md（§6 INJECT/STALE 事实、Place 代际、924 唯一 owner）；SD-06 ace-m1-conformance.md（PLAN-923 段）+ gap-audit.md（F-NAT-4a R03 ac 已闭行）。
- 924 交接实际 SHA：worktree 交付 49bd0f597（feat）+79feeb3ce（SD）；基面 ca9bcafb9；接口 mn.collections.runtime/3、mn.collections.cell/3、mn_coll_fail_nth_seam、mn_coll_place_mint/get/set、restore_collections_session、ac/mir-native-resource-guards-v1；R02 K02/env 欠项与 R05 归 924 唯一回填（SD-02 §924 输入清单），无循环前置。
- 全部必需 V 可执行：V-01/02/03（seam_tier+probe seam-retry）、V-04/05（place_tier 10 测试）、V-06（case-923 8 checks）、V-07/08（restore session 2 测试）、V-09（performance 隔离 5 项）、V-10（consumer 4 组）、V-11（SD-02 924 清单）全绿。
- 提交：49bd0f597 feat（T-03..T-08 全部代码/测试/AutoDown/binding/manifest/test-groups）+ 79feeb3ce docs（SD-01..06）。worktree 状态 clean。
- 待 review：T-10 独立复审（冻结 SHA 49bd0f597/79feeb3ce）。

2026-11-03 | stage:goal | PLAN-923 r1 | round:2 | from:work | to:review | status:execution_done | outcome:pass | worktree:D:/autostack/.wt/lang-923/auto-lang | commit:49bd0f597+79feeb3ce | next:review T-10
- collections_runtime_probe.rs 新增 cfg!(mn_probe_seam_retry) 分支：构建 [10,20]（cap1，第 2 个 insert 已 grow 一次）→ mint place@elem1(=20) → fail_nth_seam(SEAM_GROW,1) 注入下一次 grow → insert 失败 INJECTED（check 171）→ 清 fault → 快照不变：len==2、place 仍读 20（epoch/gen 未动，check 172/173）→ 同进程重试同一操作成功 len==3（check 174）→ 结构变更后旧 place STALE（check 175）。
- resources_tier 新测试 runtime_seam_snapshot_and_inprocess_retry_keep_atomicity（probe seam-retry，exit 42 + zero_live）。
- 灵敏度控制（V-02..05）：read-first/write-first 由 place_tier stale 读写 trap 27 且 runtime validate 在任何 buffer 解引用前执行证明；省 free/提前释放反控由每个 trap/inject 用例的 live=0/bytes=0/invalid=0/arena_live=0 断言承担（省略 free 即红）。
- 验证：probe seam-retry PASS（1.8s）；seam_tier 4/4 PASS（l04/lstr/grow/typed clone sweep）；ace-native 非 performance 188/188、ace-runtime 84/84 同 T-04 记录。

2026-11-03 | stage:review | review_scope:final | reviewer:同会话（work 会话内独立复审，判定从冻结 SHA 与工件重构，非执行者摘要） | PLAN-923 r1 | outcome:needs_fix（3 项，均已修复重验后 pass） | worktree:D:/autostack/.wt/lang-923/auto-lang | branch:plan-923-dev | reviewed_commit:49bd0f597+79feeb3ce → 修复后 c0ce0dce7 | base_commit:ca9bcafb9 | spec_inputs:SD-01..06（79feeb3ce） | acceptance_results:AC-01..08 独立重跑全绿（place 10/10、seam 4/4、probe seam-retry、restore session 2/2、case-923 8 checks、ace-native 冻结基线 191/191 非 performance、889/915/919/891 组 PASS） | findings:
- F-1（§5.3 foreign token 防御无测试）place_tier 缺 foreign/EMPTY bits 拒绝路径断言——补 place_foreign_token_is_refused_before_any_dereference（PlaceSet 退役后 slot EMPTY → PlaceGet trap INVALID 23 + 全零清理），PASS（fedda8f5d）。
- F-2（health unused 参数）restore_collections_material 的 workdir 参数未用（923 引入警告）——删参数并同步两调用点，编译零新增警告（ce792e241）。
- F-3（ABI /2 consumer 遗漏）collections_driver_tier:379 断言 cell/2（ABI bump 后红）+ collections-runtime/CONTRACT.md 与 ac_mn worker fixture 仍写 /2——全改 /3，889 组复跑 PASS（c0ce0dce7）。全仓活动代码精确扫描（python 正则 mn.collections.(runtime|cell)/2）无残留（attachments 历史证据/归档文档保留原值）。
| evidence:命令/退出码见各修复收据；冻结基线 191/191（128s）；case 组 PASS | next:merge T-11（本 review 已 pass 修复后基线，进入 reviewed）

2026-11-03 | stage:goal | PLAN-923 r1 | round:3 | from:review | to:merge | status:reviewed | outcome:pass | worktree:D:/autostack/.wt/lang-923/auto-lang | commit:c0ce0dce7 | next:merge T-11/T-12
| D-01 allocation图/计数 | T-02冻结真实guest/runtime/text/metadata域与seam互斥，完整逐点而非机械搬AV N |
| D-02 Place最小布局 | T-02冻结不解悬空指针的registry/UID/epoch/gen、ABI与预算；等价实现须满足全部V-04/05 |
| D-03 来源续编辑 | T-02核889工件restore后前端差额；有界上下文修复归本包，不cold冒P0或丢恢复后编辑验收 |
| D-04 R02闭包尾项 | 本包明确不关闭；T-09交924 K02/env故障义务和稳定版本，不能等924阻止NF独立交付 |
| D-05 921/922窗口 | T-01取实际owner/稳定SHA/全consumer；922有树即按在途，未取窗只挡相关任务 |

2026-11-03 | stage: merge | PLAN-923:r1 | prepared: reviewed基线c0ce0dce7(=59ef759f7补丁等价)+SD-01..06(79feeb3ce链) | landed: master tip=ddd42ef2b（rebase映射5/5 range-diff全等价，--ff-only无merge commit；925会话14提交先落为基线差异，仅祖先进位无代码冲突） | canonical_verified: docs/specs/auto-mir/native-fault-and-place-conformance.md、docs/specs/auto-test-case/native-resource-scenario-coverage.md 落地于c6e92020b；modify四目标俱在主线 | merge_gate: 924重放树冻结并集（两套件组/artifacts/adapters/auto-test-case/infra-fixed PASS；ace-native 3 Deadline红=并发环境类、串行重放绿已录；infra-ace-m1红归924 SD-6 pin、924侧已修PASS） | archived: docs/plans/archive/923-native-allocation-fault-and-place-conformance.md status=archived | cleaned: wt-guard clean→树/分支/组目录移除；plan-artifacts/plan-923 scope同批清理；无具名hold | batch_regress: 未到期（last_covered=925同日，923非5倍数） | outcome: pass（由PLAN-924 merge会话代收，924 AC-10前置）
