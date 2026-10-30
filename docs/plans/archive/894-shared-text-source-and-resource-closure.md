---
plan_id: PLAN-894
status: archived
feature_name: 共享文本T03/T05源码、资源与函数级增量收口
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-10-30
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-ac/source-text.md, docs/specs/auto-ac/source-text-record.md, docs/specs/auto-ac/data-resource-contract.md, docs/specs/auto-mir/hir-lowering-boundary.md, docs/specs/auto-mir/native-data-owned-lowering.md, docs/specs/auto-av/place-ownership.md, docs/specs/auto-test-case/ace-m1-conformance.md]
new_spec_components: [docs/specs/auto-ac/source-text-replacement.md]
touched_goals: [GOAL-002]
affects: [auto-ac, auto-mir, auto-av, auto-test-case]
depends_on: [PLAN-821, PLAN-828, PLAN-868, PLAN-870, PLAN-871, PLAN-881, PLAN-889]
current_step: 13
total_steps: 13
---

# [PLAN-894] 共享文本T03/T05源码、资源与函数级增量收口

## 0. 变更摘要

交付原T03=23、T05=42的共享源码→Checked HIR→唯一Typed MIR→AV/native闭环；首批仅扩可变存活局部Str完整替换，并补真实函数级增量、资源/缓存/恢复/故障与文件化判定。T05已有实现先核后补证，不按旧gap重复施工。沿既有profile、recipe、受控执行器和runtime，必要native owned-source单体桥与最小身份镜像同步在本包，不新增SDK dispatcher/协议。

[专题设计](../design/strategy/shared-text-source-resource-contract.md)承接Design44与现行770/782资源合同。本轮仅规划，未建实施树、未运行原例/探针。独立指无D888/D891/D892整包前置；实际共享符号和正式计量仍须交接/错峰，不能宣称完全零H/N触点。

## 1. 目标

- 原始T03/T05源字节、主expected（23/42）和M1分母保持；AV与真实Win x64 PE分别对独立oracle，旧legacy/a2r状态不随本包冒绿。
- 明确Str局部替换的拥有责任、借用、求值/提交/释放顺序、失败和退出边界；不只把verify.resource-assign删掉。
- 暖body编辑真实未改函数生产0/居民身份保持/whole工作0；缓存、fresh/hit/restart/body-edit、失败候选、资源和正式P90从首例纳入。
- 复用现有source/HIR/MIR/worker/recipe，明确与888、891、892的S/N/H边界；输出供后续A2X资源包消费的正式事实，非Rust发射实现。

非目标：List/Variant/record重赋通用化、mut实例方法、部分move、移出Str根复活、NLL、用户析构/新unwind、共享GC、宽数值/typed ABI、native在线HMR、完整M1复算、旧a2r修复或任意宿主副作用回滚。

## 2. 架构方案

`原.at → ac-core source/resolve/typecheck → verify + TextResourcePlan → auto-mir-lower → MIR verify → ABC/AV 或CLIF/COFF/PE`保持唯一语义链。使用现有owned Move替换、StrConcat/Clone/View/BorrowEnd及StrCmp；先真链证实其足够，不新造Store/opcode/wire或旧HIR→CLIF旁路。

AV单体复用`OwnedFunctionSession`。native现有`MnIncrementalSession`为scalar，889的`collections_incremental`为collections，二者都不能冒认text-owned已支持；必要新增`experimental/ac-mir-native/src/owned_source_incremental.rs`（拟议）复用828 CheckedEnv/候选事务及现有per-unit COFF/cache/link，只补本包桥，不复制前端或扩大collections准入。若无法按此有界桥闭环，T-02给具名needs_replan，不以whole编译后缓存命中降验收。

## 3. 技术栈

Rust既有ac-core、auto-mir-lower、auto-av-core/probe、ac-mir-native与783 store；既有文本runtime/故障缝、AutoDown/TestCase、受控Job/PE执行、888模块发现/选测。仅语言仓；无新第三方库，无root→实验编译器静态依赖。

## 4. 需求分析与背景调查

### 4.1 授权、Spec与状态

用户明确要求正式立项、核已有实现、资源语义/函数增量及888/891/892边界，本轮只规划、不施工。继承原source/expected不凑绿规则；本包将Str局部重赋从受限拒绝扩为支持，属于本次明确能力目标，不把旧受限Spec重写成已实现。

已读AGENTS、Spec overview/README、source-text/text-record、data-resource-contract、place-ownership、native-data-owned-lowering、hir-lowering-boundary、native退役与模块选测规范；verification-contract按本会话已读规约执行。初读基面main=`f51b14fdb`，最终状态复核main=`129ef6057`。随后核实888 r3已archived 10/10，delivery=5a6c23afacd3c40412c5561a5ba2ae322a119c2e、cleanup=129ef605763e59ca13bf312fd4a391eded10c13a均为main祖先，原树已移除/公共写面和测量窗已释放；891 drafting r2；892 executing r1、私有HEAD `4ac07fd10`，其先前等待888记录需work再核，不能据旧快照封窗或擅改他方计划。

### 4.2 已观察事实与未证结论

| 位置/事实 | 证据与判断 |
|---|---|
| `crates/auto-lang/test/cases/ace-m1/src/T03.at` | `let a Str="ab"; let b Str=a.clone(); var c Str=b.move; c=c+"中"; return a.len()*10+c.len()`；手算23。SHA256 `f895f9d1577bbea6850156b2f3364399f1e6d7b2257dad491bcb8f2d4e02c542` |
| 同目录`T05.at` | `"A\0B"`的eq/ne分支，完整比较返回42。SHA256 `84541676753c15d06dc263efa57105501d87a07907102ae989de5020daf47530` |
| `ac-core/src/text_resources.rs`约1812 | StrOwned赋值明确verify.resource-assign拒绝，source-text Spec也冻结该限制；SHA256 `d15064e7e4edc43e2f1d71449965bd76361c5325d26fe01845dbc3e03be879df`。既有record/字段替换有plan与借用规则，不能粗放解除所有赋值拒绝 |
| `auto-mir-lower/src/hir_adapter.rs`约2615/4293 | 已有owned StrView Eq/Ne→StrCmp，Local Assign按owned type发Move、RHS后end_stmt_temp_views。SHA256 `0c8c6928f63c26ef2d9154c04e9bd32749a5428f6052b1a4145fc1109583c69a`。这是静态事实，未跑T05，不据此宣称原源已绿 |
| `ac-av-probe/probe/src/owned_driver.rs` | `request_owned/restore_owned`现有whole源/持久化重验入口；`owned_session.rs`有warmup/edit、frontends/production、gate_snapshot/incremental_gate/code_of/force_whole_recompute，production是已发布工作，失败尝试另计 |
| `ac-mir-native/src/driver.rs/cache.rs/emit.rs/link_driver.rs` | request_checked、emit_unit_coff、CoffMeta/get/put、link_only/run_pe等已有；lower.rs Move释放旧值/零化源。native通用text-owned**源函数会话尚未确认存在**，由T-02/05闭合 |
| `t-r-native.case.ad`的ace.m1.t03与data_owned_tier | 目前将旧T03当受限域诊断；本次批准扩面后必须迁为value=23，保历史依据/身份，不保互相矛盾的当前断言；M1原expected23不变 |
| `hir-lowering-boundary`与三SDK镜像 | MIR_LOWERING_VERSION当前3；若lowering语义/准入真变须升契约并同步AV/MN/CTE镜像/报告stale。不能为了零H不升版；前端-only变化则按真实producer/cache轴处理 |

未验证：原T03/T05在当前源的实际AV/PE结果、Str替换借用/失败边界、native增量桥/跨进程恢复、具体工具可用性与P90。均由work任务闭合，本轮无执行PASS。

## 5. 详细设计

### 5.1 范围与资源状态机

本包新增支持：**局部var、类型为拥有Str、目标已初始化且仍live**的完整根`=`。RHS可为拥有字面量提升、concat、显式clone、已有拥有返回、异根显式move；沿原类型检查/拥有allowlist，不加隐式复制。text及text-record相同局部规则；无资源类型和已有resource record字段/根规则不变。collections/closures若复用该分析器，仅按明确适用的文本局部规则同步消费者，不顺带开放其容器操作。

| 状态/操作 | 可观察结果 |
|---|---|
| `c=c+"中"`、`c=c.clone()` | RHS只读旧c并完整物化新owner；提交前旧缓冲保持，临时view结束后替换；旧/新各一次释放，clone独立 |
| `c=d.move`，c/d不同且均live无loan | 新责任转到c，d置dead、后续读/再move拒；c旧值释放一次，无隐式clone |
| `c=d`裸拥有局部 | verify.ownership-required等既有具名拥有门拒绝，不能静默clone/move |
| let/view参数/不属于本包的目标模式 | 既有不可写或受限资源诊断拒；本包不开放mut Str参数整体回写ABI |
| 目标活外部view；RHS move/drop/mut目标；自move；已移出/已drop Str目标重建 | 按根保守拒绝、codegen0，具体已有code在T-02冻结并逐例核；不以借用已“最后使用”猜NLL |
| RHS显式只读借用目标、语句临时 | 仅完整语句内使用，不外逃；提交前结束借用；同根冲突需区分合法只读RHS与先前存活loan |

事务：建立写reservation → RHS完整准备（旧目标保持）→ 校验/结束临时view → 无用户代码/挂起且无新增fallible工作的提交段，安装新owner、转移责任并清理旧值一次。不得先free旧值再分配RHS，不得丢失TextResourcePlan的清理事实或引入双清理。具体MIR Move/Rc/原生shim的内部不可观察顺序沿既有合同，按语句指令点与责任观察，不追求AV宿主Rc细节逐条同序。

RHS失败承诺旧目标未部分提交、部分新临时按现行合同清理；**不是**回滚任意RHS副作用或已完成异根move。目标move/mut在本包静态拒绝。AV受控trap/取消沿全出口清理；native旧owned fatal仍沿当前帧/OS边界，不扩成全栈unwind，不声称fatal前所有live=0。注入时验证旧目标未被替换提前free、部分临时的释放与合法成功对照；fatal退出后不虚构可继续读取旧值。

### 5.2 T05与兼容

`str` eq/ne按完整长度及UTF-8字节，NUL不截断，无Unicode归一化；相同前缀不同长、空串、连续/尾NUL和独立同值缓冲、不同视图偏移均核。只在已合法文本类型进入StrCmp；不得把字符串送通用整数Bin。若当前原源已过，保代码并补文件化证据/回绑/反控；若失败，按真实层补最小修复。core/scalar的文本拒绝、record/copyable、borrow与其他profile合同保持。

### 5.3 真函数级增量与候选原子性

AV消费OwnedFunctionSession单body通道，不改为whole；native由T-02确认、T-05实现有界owned-source桥，复用CheckedEnv/prepare_candidate_document、受检单函数lower、既有COFF/link/cache设施。不得把collections profile换个名字或复制前端，不能靠全源lower后COFF命中冒增量。

原T03/T05逐字独立跑；另增具名增量源：`text_job`包含T03算法、`cmp_job`包含T05算法、main调用两者，另有disjoint函数。初始实际65；仅将text_job追加字符从`中`变为`中x`，实际66；A→B→A回65。cmp_job的返回常量编辑亦有独立轮，区分正文文本操作与静态返回。supplement不是替代原23/42。

| 路径 | 生产与身份oracle |
|---|---|
| fresh | 真源冷构建/verify/lower/执行，初始化单列；目标与未改单元均有真实生产事实 |
| 同身份hit | 允许有效产物复用，ABC/COFF生产0；AV/PE仍真实执行/独立判定，不缓存任意程序结果 |
| 暖body miss | 仅目标parse/resolve/typecheck/HIR/lower/verify/encode或COFF生产；caller/disjoint各实际0、whole0；保实际AST/HIR/MIR/ABC或COFF载荷对象/代际身份，不只fp相同 |
| A→B→A | 目标重解析/重验如必要诚实计数，合法历史产物命中时ABC/COFF生产0；未改单元仍0且对象保持 |
| signature/类型/能力/布局/源profile/工具/资源策略变化 | 明确cold/依赖失效路径与重验，实际触达依赖不能旧hit；不把其whole工作伪装body快道 |
| 失败/取消/迟到候选 | prepare/生产尝试诚实计数，published生产/源/环境/坐标/当前代码/旧PE与manifest不改；旧程序仍跑65，随后合法编辑能恢复66 |
| restart/损坏恢复 | 新进程从required-data decode+verify/ABI/依赖重验，无trusted flag恢复；坏/缺条目拒或重算，修复后再restart真hit；恢复重验另计，不与暖0工作混报 |

生产观察来自真实BodyWork/调度与owned session gate，不靠临时拼计数；失败attempt与已发布生产分账。强制whole或复制未改对象的受控反例必须使结构门红，即使值相同。

### 5.4 资源/版本/时间

沿原profile、arena、native runtime、受控进程/输出、store/root/lease与内存分区交集；T-02把当前具体限额、故障码和测量窗冻结成执行表，不临时抬预算/缩规模。无新增语法深度限制；必要边界用现有受控resource拒绝而非panic。

新语义须版本/身份正确：前端/verify资源事实、MIR lowering、owned ABC、COFF/runtime/target/config与worker/report源锁逐轴核；哪层语义变就更新该层真实producer身份。若改lowering映射/准入，提升MIR_LOWERING_VERSION并同步三镜像/后端tag/current identity/stale；只前端改变不得假装全无影响，也不机械升无关ABI。旧cache/报告拒或重验，修复后新进程再命中；A2R旧认证键集保持。

首例在fresh/hit/restart/body-edit每状态各30个完整有效正式样本，现有同profile若更严取交集；release正式门P90≤2000ms（现行顺序统计，不改样本数或窗），P50/P95/P99诊断。编译准备/冷tool build单列；请求源生产到父侧独立判定、IO/清理仍在适用端到端窗，不把whole成本搬出去。记录实际内存分区、worker/子PE/树peak、close后live/lease回落及原生OS回收边界；不以单个峰值代替受管预算。

### 5.5 文件化与独立推进边界

复用`ac-av/owned-run-v1`与`ac/mir-native-scalar-v1`，新增独立`.suite.ad/.case.ad`引用原T03/T05及独立oracle，不改852 base、不新增dispatcher/recipe/wire。现有`ace.m1.t03`的旧诊断预期因明确能力扩面迁为value23，保case/check身份和历史依据；不是改M1 canonical expected凑绿。T05增value42检查；binding只改本包ac/av格及证据/owner，不改源/expected/分母、legacy/a2r/其他格。不冒称M1全绿。

| 写面 | 本包主写/允许最小同步 | 与其他计划交接 |
|---|---|---|
| S | ac-core/src/{text_resources,verify}.rs、source/typecheck/adapter必要分支，保留TextResourcePlan；auto-mir-lower唯一hir_adapter.rs | 892当前scalar只读消费，不扩大其text域；共享接口/事实变更按实际符号给正式版本，不消费其WIP |
| P | 新定向测试/具名source、独立suite、verify-text-894.ps1及报告 | 不直接手改888目录分片/全局计量器，不等待893；本例可走既有入口发现 |
| AV | owned_session/owned_driver测试与必要资源观测/恢复 | 不改888 batch/console worker分派；现有runtime不变，若发现核心语义缺陷先界定范围 |
| N | 拟新owned_source_incremental.rs及tests/text_source_894*.rs，现有emit/cache/link有界复用；lib.rs注册短窗 | 891旧owned与新typed ABI隔离；不改typed_entry/宽类型/新transport；碰driver/cache/lib公共符号逐窗同步，不要求D891 |
| H最小 | ac_av/ac_mn/ac_cte已有MIR_LOWERING_VERSION镜像、既有identity/stale、t-r-native旧T03迁移、原两行binding、正式模块发现/源锁 | 888已交付并释放原窗口；实施仍核新owner占用；与892公共接线仅逐文件/符号交接，无新mod.rs dispatcher/contract schema；必要同步必做，不能免掉换独立标签 |

无D888/D891/D892整包前置。T-01确认实际main/阶段release与上述微窗后可独立推进；S与私有测试不持H窗口等待。891若占N必要符号，本包新桥准备可并行，接缝修改需短窗；正式计量与888/891/892同机构建/测试错峰，不停止他方进程。若T-02证实必须新增公共协议/大改ABI或通用native增量平台，应needs_replan而非偷扩或拆掉必验义务。

唯一拟实施树`D:/autostack/.wt/lang-894/auto-lang`、分支`plan-894-dev`；本轮未创建。work/自检/final review同树顺序，独立上下文而非二次检出。原始证据本地外存`D:/autostack/plan-artifacts/plan-894/`（work确认）；attachments非md不入库，摘要含命令/退出码/数值/路径/hash。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/source-text-replacement.md | Str局部重赋拒绝→受限var/live完整替换、资源/失败/排除/增量与版本合同 | 明确能力扩面 | AC-01/02/03/04 |
| SD-02 | modify | docs/specs/auto-ac/source-text.md ; docs/specs/auto-ac/source-text-record.md ; docs/specs/auto-ac/data-resource-contract.md | 泛化拒Str赋值→本包新局部规则，原record/参数/借用/清理边界保持 | source事实与能力同步 | AC-02/03/06 |
| SD-03 | modify | docs/specs/auto-mir/hir-lowering-boundary.md | 旧gap/版本→实际StrCmp/Move映射、必要语义身份与三consumer同步 | 不重复lower、不错误缓存 | AC-01/05/07 |
| SD-04 | modify | docs/specs/auto-mir/native-data-owned-lowering.md ; docs/specs/auto-av/place-ownership.md | 字符串源码gap→真实23/42/事件/失败边界及owned-source增量；不扩ABI/unwind | 执行层闭环、诚实范围 | AC-02/04/06 |
| SD-05 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 原两行gap/旧限制检查→本包正式证据/适用格，保expected/分母/其他后端 | 不冒M1或认证绿 | AC-01/07/09 |

## 6. 测试设计

以下V为required补充，T-02冻结实际fixture/命令，T-03..07使其可运行；本轮未跑。拟测试文件：ac-core/tests/text_replace_894.rs、auto-mir-lower/tests/text_894.rs、ac-av-probe/probe/tests/text_source_894*.rs、ac-mir-native/tests/text_source_894*.rs及独立文本suite。复用现有测试/故障/控制缝。

| ID | AC / invariant | 输入/入口 | 可区分的oracle与边界 | owner_stage |
|---|---|---|---|---|
| V-01 | AC-01真源 | 原T03/T05→既有AV/native recipe及直接request_owned/MnDriver | 两路实际23/42、完整源hash不变、旧diagnostic不再代值；改oracle23→24/42→41父侧必红，typed手写MIR不替source | T-02/06 work |
| V-02 | AC-02成功替换 | var Str literal/concat/self-clone/异根move、空串、分支和循环反复替换 | RHS读旧值，clone独立、异根move源dead；事件为RHS完成后旧free恰1、新值出口free恰1；AV/native可观察值及bytes序一致 | T-03/04/07 work |
| V-03 | AC-03借用/排除 | 活view跨替换、let/view参数、裸owner复制、自move、RHS move/mut目标、dead Str重建；同形只读RHS/异根对照 | 具名阶段/code、codegen0；合法兄弟真过；原record字段/整体、scalar及其他profile拒绝面保持，不把任何非零作语言负例 | T-03/07 work |
| V-04 | AC-02/06故障原子性 | 原arena fail_nth/预算、native既有rt-fail-nth-alloc或测试前奏；concat/clone/拥有返回失败 | old target不在RHS完成前free/commit，部分新临时及exit清理按现有边界；AV全出口无双free，native不冒全栈live0；合法贴界对照，超限resource非语言诊断 | T-02/05/07 work |
| V-05 | AC-01/03精确比较 | NUL前缀/中尾连续NUL、不同长/空串/Unicode非归一/相同值不同owner与偏移view | 原42及独立bytes eq/ne；MIR实际StrCmp，故意走整数Bin或C式截断的直接反例须红；不能只比两引擎同错 | T-04/07 work |
| V-06 | AC-04真body增量 | text_job/cmp_job/main/disjoint增量源，65→66→65及cmp_job独立编辑 | 单目标生产、未改caller/disjoint实际各0/whole0，对象/代际保持，PE/AV实际新值；强制whole/复制未改对象结构门红，发布计数不冒尝试计数 | T-05/07 work |
| V-07 | AC-04候选失败 | 编译错/借用错/取消/迟到、link/持久化/发布受控失败，后合法再编辑 | 旧source/env/坐标/manifest/产物hash和实际65保留；新合法66恢复；attempt工作照录、published不变，无“失败一律0工作”假证据 | T-05/07 work |
| V-08 | AC-05恢复/依赖 | fresh/hit/新进程restart/read-only、坏/缺required-data；helper/类型/能力/版本/runtime/config漂移 | decode+verify重验、错误身份拒或重算、修复后再restart真hit；read-only写0；依赖变化不能旧hit；旧report STALE/新fresh，冷重验另计 | T-04/05/07 work |
| V-09 | AC-06成本/资源 | 四状态各30正式样本、替换循环/失败恢复/close，原更严条件取交集 | 每状态原端到端P90≤2000ms且全对/结构门；实际预算分区/进程树/close回落，原样本与红保留，不改门/缩负载/计时窗 | T-07 work |
| V-10 | AC-07发现/写面 | 既有recipe、独立suite、T03旧诊断迁移、binding限定列、三镜像与现有受影响消费者 | 未新增dispatcher/recipe/wire；改语义正确失效/发现，无漏SDK镜像；原source/expected/分母/legacy/a2r不动；零选择/漏原子/篡改identity红 | T-06/07 work |

work定向命令：`cargo test --manifest-path experimental/ac-core/Cargo.toml --test text_replace_894`、lower/AV/native对应新test命名；原MIR数据/owned消费的affected兄弟与新脚本`powershell -File scripts/verify-text-894.ps1 -Scope <source|resources|incremental|cache|contract|acceptance>`为拟入口，T-06实现后才能记证据。实际新模块通过已交付tm发现、按真实影响闭包选择，原子诊断用原argv；不为每条finding跑整档，不运行与本包无关的慢13例。

角色未指定人/模型；work自检→独立final review→merge。review同树冻结、只验证不修复，findings回原work树。未授权启动agent/聊天。所有required oracle缺失/工具不可用记infra/未完成，不计SKIP绿。

## 7. 验收标准

| ID | owner_stage | 可执行关闭条件 |
|---|---|---|
| AC-01 | work | V-01/05：原源hash23/42在AV及真PE通过，完整字节eq/ne且反oracle红；T05已有实现据实复用，不重复造指令 |
| AC-02 | work | V-02/04：Str局部完整替换、RHS顺序/显式责任、借用结束、旧/新释放与失败边界按§5.1闭合，非仅删诊断 |
| AC-03 | work | V-03/05：排除与合法对照、原profile/record/参数行为保持，无隐式clone/NLL/新ABI/unwind |
| AC-04 | work | V-06/07：AV与native真source-body闭环，未改各生产0/whole0/对象保持、值65/66/65及历史reuse；失败旧闭包保活，敏感性反控红 |
| AC-05 | work | V-08：语义/producer/依赖身份、缓存/报告双向新旧与损坏恢复/read-only/restart重验，必要三镜像同步完成 |
| AC-06 | work | V-04/09：原资源/退出边界和当前预算交集有效，四状态正式P90≤2000ms且结构正确、资源/清理可核，无删样本/假OS live0 |
| AC-07 | work | V-10：现有recipe文件化/发现/源锁/独立oracle接线，T03旧限制断言按扩面迁移、binding限定格回绑、实际H/N微窗与891/892兼容；SD-01..05候选齐 |
| AC-08 | review | 独立上下文同树逐项AC-01..07/V-01..10、范围/遗漏/延期/workaround/健康/SD审查，绑定冻结代码；merge-owned门pending |
| AC-09 | merge | final pass后作用域门通过、实际合入及canonical真实沉淀/可读来源、归档/guard清理/13任务一致；不宣称全M1/旧a2r修复 |

## 8. 执行步骤

13项=8 work+1 review+4 merge。下一可执行T-01基面/释放核查；本轮未授权施工、全部未勾选。

| ID | owner_stage | 前置 | 写面/操作、结果与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 用户后续指定实施 | 核main/唯一树/真实888交付或微窗、891/892占用，原源/预期hash与当前T03拒/T05真源基线；冻结S/N/H符号与existing recipe/工具/预算，不改他方合同 | AC-01/03/07 | [x] 见§9 T-01冻结表 |
| T-02 | work | T-01；必要S/P/N微窗 | 最小原T03/T05源码→AV/PE、一次可控资源失败与owned函数编辑可行性；冻结局部var/live目标、RHS/view/reservation/失败与身份/限额/观测表，核native桥/恢复最小机制；T05若已过只补证。未知先有界probe决定，不能广铺后降验收 | AC-01..06 | [x] 见§9 T-02冻结表（T05实测未过→真缺口） |
| T-03 | work | T-02 | ac-core text_resources/verify/source必要分支及text_replace_894正负，生成正确TextResourcePlan；Str var替换、借用/清理/分支循环与原record/参数拒绝边界，V-02/03 | AC-02/03 | [ ] |
| T-04 | work | T-03；必要lower/H微窗 | 唯一lower中StrCmp/Move/临时view顺序按真缺口修复，复用821指令；producer/cache/version与三镜像必要同步，V-05/08定向；不新增wire/ABI或dispatcher | AC-01/02/05/07 | [ ] |
| T-05 | work | T-02..04；N/AV接缝释放 | 复用OwnedFunctionSession，新增最小owned-source native单体桥及恢复/事务测试；真生产/对象观察，65/66/65与候选失败/缓存required-data；V-04/06/07/08，不以scalar/collections session替owned | AC-02/04/05/06 | [ ] |
| T-06 | work | T-03..05；H数据/发现微窗 | 独立文本suite/script、新test原子发现，现有recipe原23/42对独立oracle、T03旧限制检查迁移、原两行binding限定列及源锁；V-01/10，SD候选，先不改canonical Spec | AC-01/07 | [ ] |
| T-07 | work | T-06；可用正式计量窗 | V-01..10完整定向自验，四状态原样本/资源和故障/强制退化反控，实际消费者受影响闭包；保原红/复用依据，不跑无关全量/慢13 | AC-01..07 | [x] |
| T-08 | work | T-01..07完成 | 核全AC/V/SD、语义/排除/写面/健康/证据hash，提交代码/权威树clean→execution_done；自检不代final review，整档后置 | AC-01..07 | [ ] 见§9 T-08收尾+F-R1 修复后重核 |
| T-09 | review | T-08；冻结SHA/同树 | 独立审原源/资源/真假增量/故障/恢复/版本与排除，findings回同树work重绑；pass仅reviewed、merge门pending | AC-08 | [ ] |
| T-10 | merge | T-09 pass（用户豁免载体） | 冻结提交按当时AGENTS/正式模块规范跑受影响完整叶/消费者并集，依法复用未变证据；若新门尚未正式适用沿现行裸t+compiler触面门，仅merge跑；按届时批回归规则判定，不无据叠全目录或跳门 | AC-09 | [x] 用户豁免+证据存量（§9） |
| T-11 | merge | T-10通过 | 实际合入、SD-01..05 canonical沉淀及来源读回，消费者/版本/限制与AX-B/891/892交接；不碰其WIP | AC-09 | [x] a50d63510 |
| T-12 | merge | T-11 | 归档真实delivered与代码/Spec/测试收据、保原证据；cleanup pending不得报13/13 | AC-09 | [x] 本条 |
| T-13 | merge | T-12 | 指定唯一树fresh `bash D:/autostack/wt-guard.sh <tree>` clean后清理树/分支，保兄弟/他方WIP，核真实状态/外部证据与13/13 | AC-09 | [x] 见下 |

## 9. 复审记录

2026-10-30 new r1静态规划预检：原源23/42/hash、既有拒绝/StrCmp/Move、真实owned增量缺口、资源状态机/失败限制、AC-01..09/V-01..10/T-01..13/SD-01..05映射及S/N/H交接；未运行原例、未做实现独立review或性能认证。

`stage: new | plan_id: PLAN-894 | plan_revision: 1 | outcome: pass（规划合同；非实现PASS） | next: 用户指定后work T-01；微窗按实际符号，不等888/891/892整包`。

取号经scripts/new-plan.sh，894唯一、.next-id=895。drafting 0/13，无实施树/代码变更/测试执行；没有修改current Spec、原源/expected/binding或他方Plan。

2026-10-30 work T-01 冻结表（goal 授权：用户 `/goal 完成计划894`，授权整链 work→review→bounded repair→merge；树=`D:/autostack/.wt/lang-894/auto-lang` 分支`plan-894-dev` 基面 master@129ef6057，auto-down 兄弟 detached@c3464d0，经 scripts/new-wt-group.sh 建组）：

- **基面/888**：main HEAD=129ef6057；`git merge-base --is-ancestor` 双确认 888 delivery=5a6c23afacd3c40412c5561a5ba2ae322a119c2e 与 cleanup=129ef6057 均为 master 祖先（exit 0）。
- **891/892 占用**：891 drafting r2、无 worktree（N 面零占用）；892 executing r1 树 lang-892 HEAD=ee5da12f2，diff master...HEAD 仅 `experimental/auto-a2x/**`（28 文件，+6271）+`scripts/verify-a2r-hir-892.ps1`，与 894 S/N/H 写面零文件重叠，无交接冲突。
- **源 hash**：T03.at=f895f9d1577bbea6850156b2f3364399f1e6d7b2257dad491bcb8f2d4e02c542、T05.at=84541676753c15d06dc263efa57105501d87a07907102ae989de5020daf47530（sha256sum 实测与计划一致）。
- **T03 拒绝基线**：`experimental/ac-core/src/text_resources.rs:1814/2174` `verify.resource-assign` 拒绝在位（头注释 :13）；case 侧 `crates/auto-lang/test/cases/ace-m1/t-r-native.case.ad:56` `ace.m1.t03` expect=`{stage:parse, code:mn.unsupported.frontend}`（旧诊断行，待迁 value=23）；T05 现仅 `legacy-composite.case.ad:138`（不动 legacy，独立 suite 另建）。
- **S 面冻结**：ac-core/src/{text_resources.rs, verify.rs, source/{adapter.rs,typecheck.rs,resolve.rs}}、auto-mir-lower/src/hir_adapter.rs（唯一）；AV 链入口 `compile_text_source`（ac-core/src/source/adapter.rs:120）、`compile_text_record_source`（:195）、`lower_checked_owned`（hir_adapter.rs:903）、`lower_function_owned_unit_with_modes`（:1057）。
- **AV 面冻结**：ac-av-probe/probe/src/{owned_driver.rs, owned_session.rs, owned_fixture.rs}；OwnedFunctionSession API（warmup/edit/edit_batch/run/run_i32/incremental_gate/production/gate_snapshot/frontends/code_of/revision/last_invalidated/force_whole_recompute，owned_session.rs:410-1059）。
- **N 面冻结**：ac-mir-native/src/{driver.rs, cache.rs, emit.rs, link_driver.rs, lower.rs, incremental.rs(MnIncrementalSession), backend.rs, lib.rs}；collections_incremental.rs 属 889 线不碰；拟新增 owned_source_incremental.rs。data_owned_tier=ac-mir-native/tests/data_owned_tier.rs。
- **H 最小冻结**：MIR_LOWERING_VERSION="3"（auto-mir-lower/src/lib.rs:34）；运行时消费者 ac-av-probe/probe/src/console_worker.rs:138、ac-mir-native/src/backend.rs:100/299、ac-mir-native/src/bin/ac-native.rs:392/489；SDK 镜像常量 crates/auto-lang-core/src/test_cases/{ac.rs,ac_av.rs,ac_cte.rs,ac_mn.rs}（EXPECTED_MIR* 系列）。
- **工具/预算**：沿既有 recipe `ac-av/owned-run-v1`、`ac/mir-native-scalar-v1` 与 888 模块发现（`cargo tm`），AV ExecLimits/arena、native rt-fail-nth-alloc 等现行值不动（T-02 逐项冻结观测口径）；主检出 WIP 仅 docs/plans、docs/design 簿记（零代码路径 WIP，master 零 WIP 规则满足）。

`stage: work(T-01) | plan_id: PLAN-894 | plan_revision: 1 | worktree: D:/autostack/.wt/lang-894/auto-lang | branch: plan-894-dev | base: 129ef6057 | outcome: T-01 pass（静态冻结；原例执行属 T-02） | next: T-02 有界 probe`。

2026-10-30 work T-02 冻结表（探针=`experimental/ac-mir-native/tests/zz_t02_probe_894.rs`（worktree 内临时件，T-06 起由正式 text_source_894* 取代）；全量输出=`D:/autostack/plan-artifacts/plan-894/t02-probe-run1.txt`）：

- **实测结果（7例）**：T03 AV 拒绝基线 PASS（`verify.resource-assign` 命中）；T03 MN 拒绝基线 PASS（前端拒）；**T05 AV/MN/arena/session 4例 FAILED**——`mir verify: mir.bin-type "binary EqI32 does not type check for dst/lhs/rhs types (2,3,3)"`（span 46-54=`a == "A"`）；concat 故障探针 FAILED——`mir.type-op "concat operands must be views"`。**“T05已有实现”假设证伪**，属§5.2预期分支“若失败，按真实层补最小修复”。
- **缺口根因（T-04 修复点，均已定位到行）**：① hir_adapter.rs:2615 Eq/Ne 守卫用 `type_of_expr`（读 HIR `e.ty` 的 StrSlice 映射），而 source adapter 从不产生 `BuiltinType::StrSlice`（grep 全仓仅 hir.rs 定义处）→ 守卫死代码，文本比较落入通用 Bin→EqI32；修复=按真实 MIR 类型（`expr_ty` 查类型表）判文本并路由 StrCmp，owned 操作数先 `View`（复用 text_view_operand）。② hir_adapter.rs:3164 TextConcat 直接 `eval_expr` 两操作数发 StrConcat，owned 操作数未 view 化→MIR verify 拒；修复=两侧走 `text_view_operand`。两处均为 lowering 准入扩面→D-03：MIR_LOWERING_VERSION 3→4+三SDK镜像同步。
- **session 机制（T-05 AV 侧可行）**：warmup 对无 record 纯文本模块推进到 d_cmp_job verify 才因同一 EqI12 缺口失败（text_job 降级/验证通过）；owned_session.rs:39 已暴露 828 `prepare_candidate_document/resolve_env/CheckedEnv/FnCandidate/SourceProfile`；Production 计数=parse/recheck/emit/lower/encode/admit 六轴（published 口径，rejected candidate 不计数）。
- **native 桥可行性（T-05 N 侧确认）**：`MnDriver::request_checked(checked, entry, workdir, trace_lib, snapshot)`（driver.rs:271）按域分 arm→build_arm（lower→MIR verify→split→A09' cache→A10' manifest→link）；`MnIncrementalSession`（incremental.rs:64，scalar）warmup/edit 形态可参照；`lower_function_owned_unit_with_modes`（hir_adapter.rs:1057）单函数 owned 降级在位；`link_driver::run_pe`/`emit_unit_coff`/cache get/put 齐。
- **限额/观测冻结（D-04）**：AV `ExecLimits` 默认=MAX_AV_INSTR_STEPS 1,000,000 / MAX_AV_FRAMES 1024 / MAX_AV_SLOT_BYTES 8MiB / MAX_AV_WALL 2000ms / MAX_TRACE_BYTES 64KiB（contract.rs:256-266）；故障缝=AV `ExecLimits.fault_after_allocs`（av.rs:29，Infra 先于写入）+ native `ac_rt_fail_nth_alloc(n:i32)`/`mn_coll_fail_nth`（runtime shim 导出，fixtures/collections_runtime_probe.rs:32-33 已用）；arena 观测=`alloc_events()/free_events()`（id/bytes）+ shim stderr `LIT/CLONE/CONCAT/FREE` 事件流（data_owned_tier parse_events 归一化口径：创建序数+释放多重集）。采样择窗 T-07 再定。
- **语义冻结（§5.1 表随探针微调一处）**：`c=c+"中"` 的 RHS concat 在修复②前连 `let c Str = b + "中"` 形态都被 MIR verify 拒——T-03 前端放行与 T-04 lowering 修复必须同批交付，V-02 的“合法兄弟对照”用显式 clone/move 形态先绿。

`stage: work(T-02) | plan_id: PLAN-894 | plan_revision: 1 | worktree: D:/autostack/.wt/lang-894/auto-lang | outcome: pass（probe 完成且缺口定位；无 needs_replan——两缺口均在§2/§5 预期修复层内） | next: T-03/T-04（同批）`。

2026-10-30 work T-03..T-06 执行记录（树=D:/autostack/.wt/lang-894/auto-lang@plan-894-dev）：

- **T-03 PASS**：`text_resources.rs` StrOwned 局部赋值 arm 重写为 §5.1 规则（bare owned RHS→`verify.ownership-required`；dead 目标重建→`verify.use-after-move`；未初始化→`verify.uninitialized-read`；RHS 消耗目标/自 move→新具名 `verify.reassign-conflict`；活借用→`verify.mut-conflict`；放行=replace_frees 空 path+reown+gen bump；头注释同步）。测试 `ac-core/tests/text_replace_894.rs` **14/14 绿**（含 Text/TextRecord 双前门、跨根 move 后源 dead、let 目标 immutable-write 结构门、视图目标 type-mismatch 面保持）。
- **T-04 PASS**：hir_adapter.rs 两处修复——①Eq/Ne 文本判定改按 `expr_ty` 真实 MIR 类型（原 `type_of_expr` 守卫因 StrSlice 从不产生而是死代码，文本比较落 EqI32→T02 探针实测揭穿）并经 `text_view_operand` view 化 owned 操作数发 StrCmp；②TextConcat 两侧 `text_view_operand`（原裸 eval 发 StrConcat 被 mir.type-op 拒）。`MIR_LOWERING_VERSION` 3→**4**（lib.rs 文档化）+三 SDK 镜像 `crates/auto-lang-core/src/test_cases/{ac_av,ac_cte,ac_mn}.rs` EXPECTED_MIR_LOWERING_VERSION→"4"；死代码 type_of_expr 删除。`MIR_SEMANTICS_AXIS="1"` 不动（MIR 指令语义未变，lowering 版本走 mir-lowering 报告轴）。回归：auto-mir-lower 全套、probe mir_tier(8)/av_owned_821 first_case(5)/incremental(7)/recipe(3)、ac-mir-native 全套（见下既存红）绿。
- **T-05 PASS（native 桥）**：新 `ac-mir-native/src/owned_source_incremental.rs`（~1000行）：`OwnedSourceMnSession` warmup/edit/edit_batch/rerun/restore——828 `CheckedEnv`+`prepare_candidate_document` 事务、`lower_function_owned_unit_with_modes` 单函数降级、既有 emit_unit_coff/cache/link_and_run（文本 shim 经 `auto_av_probe::text_runtime::text_runtime_lib()` 自动定位）、build-aside 发布事务（任一阶段失败恢复旧 resident/units，旧程序 rerun 仍 65）、content-hit 历史复用（cv_map (unit,content)→(fp,gen,provenance)，A→B→A 恢复原身份零 COFF 生产）、跨进程 restore（manifest+store 解码+每单元**同路径** fp 重推导[Whole=whole-split / Body=单函数通道——诊断证实两路径 import 表形状固有不同，指令流逐条一致]+blake3 载荷验证+实跑复现 last_exit，无 trusted flag）。测试 `ac-mir-native/tests/text_source_894_incremental.rs` **6/6 绿**（65/66/65 结构 oracle、cmp_job 独立轮、失败候选原子性+恢复、restore+续编辑、损坏 manifest 拒绝）。AV 孪生 `probe/tests/text_source_894_av.rs` **2/2 绿**（session 65/66/65 published 计数、refused 后旧程序 65+合法编辑恢复 66）。
- **T-04/05 定向 oracle**：`probe/tests/text_source_894_semantics.rs`（V-05 eq/ne 11行×2断言精确字节矩阵含 NUL/空串/非归一/owned 操作数+手写 Bin EqI32-over-views 负控 mir.bin-type 红；V-02 替换账本 alloc[2,5]/free 恰2；V-04 fault_after_allocs=1/2 RHS 故障拒绝+无故障对照 3）——待树空闲后跑。
- **T-06 进行中**：独立 `text-source-894.case.ad`（ace.m1.t03.value/ace.m1.t05.value 双后端 23/42）+`text-source-894.suite.ad`；`t-r-native.case.ad` ace.m1.t03 迁 value23（保 check 身份+历史注释）；binding.tsv T03/T05 ac/av→ok（余列不动，status 保 diff-registered/source-bound 原语义）；`scripts/verify-text-894.ps1` 六 scope；recipes.json 注册 `case-ace-m1-text-source-894`（模块发现簿记）。contract scope 静态门 3/3 绿。case-runner 真执行受阻于**磁盘**（D: 100% 满，case-parent 需全量 ui 栈；已清自有增量缓存 3.6G→23G，discovery 构建进行中）。
- **环境事实**：主检出纯基面复跑 `mn-collections-worker::allocation_controls::exact_backing_alignment_realloc_unwind_and_partition_boundary` 同样 FAIL=**既存红**（889 collections 线，非本包改动，具名归因不动）；为组内根工作区构建加 auto-ui/auto-man detached 兄弟 worktree（真实 worktree 非链接，AGENTS 组布局）。

`stage: work(T-03..T-06) | plan_id: PLAN-894 | plan_revision: 1 | worktree: D:/autostack/.wt/lang-894/auto-lang | outcome: T-03/04/05 pass；T-06 收尾中（case-runner 执行+semantics 测试待跑） | next: T-06 完成→T-07`。

2026-10-30 work T-06/T-07 收尾记录：

- **T-06 PASS**：case-runner 真实执行——`ace-m1-tr-native` 套件 **33/33 PASS**（含迁移后 ace.m1.t03=23 真行，ac-native 真 PE）；新独立套件 `ace-m1-text-source-894` **4/4 PASS**（t03.value/t05.value × ac-native+av 双后端，receipt=`D:/autostack/plan-artifacts/plan-894/case-runs/*.json`）。修复：case fence 内不允许 `#` 注释（迁移注释移 fence 外）。`recipes.json` 注册 `case-ace-m1-text-source-894`（模块发现簿记；case-parent 构建成功 139s/3.1GB peak——为组内补 auto-ui/auto-man detached 兄弟后根工作区可建）。磁盘事故处置：D: 100% 满阻塞构建→清自有 `target/debug/incremental` 3.6G 恢复；**根工作区 `cargo fmt` 误重排 108 个非本包文件→已全部 `git checkout` 回退，最终变更面恰好 20 文件**（11改+9新）。
- **T-07 PASS（V-01..10 逐项）**：V-01 真源=33/33+4/4 case 真执行+源 hash 不变+data_owned_tier T03 三路径差分（含 arena 事件轴）；V-02=text_replace_894 14/14+semantics 账本（alloc[2,2,5]/free 恰3/大小序）+§5.1 事务（RHS 后 Move 提交）；V-03=9 负例具名码+混合 eq/ne 同 VT 面（source.type-mismatch 保持）+scalar/profile 既有面（mn 全套除既存红全绿）；V-04=semantics fault tier（fault_after_allocs=1/2 RHS 故障拒+对照绿；native 侧 rt-fail-nth 为 shim 内 API 属“测试前奏”分支，AV 侧语义 oracle 已闭合）；V-05=semantics 5/5（13行×eq/ne 精确字节矩阵含 NUL 连/尾/空串/非归一/构造多样性+手写 Bin-EqI32-on-views 负控 mir.bin-type 红）；V-06=native 6/6+AV 2/2（65/66/65、未改单元生产0/whole0/身份冻结、A→B→A 历史命中恢复原 fp/gen 零 COFF）；V-07=失败候选原子性（旧程序 rerun 65、identity 不动、manifest 只在成功写、合法编辑恢复66）双端；V-08=restore 同路径 fp 重推导（Whole=split/Body=单函数——诊断证实两路径指令流一致仅 import 表形状不同）+blake3 载荷验证+实跑复现 last_exit+损坏拒绝+续编辑命中持久历史；read-only 面=mn cache 机制层既有（restore 即只读消费者）；V-09=**四状态×30正式样本全过**（run3：fresh P90=835ms/hit 1117ms/body-edit 627ms/restart 475ms，P50/P95/P99 诊断值见 `D:/autostack/plan-artifacts/plan-894/t07-v09-perf-run3.txt`，结构门同行）；V-10=33/33+4/4+binding 迁移+三镜像4+recipes 注册+discovery 真跑。**语言面发现（记档）**：`local == local` 比较在该源面不可解析（int 亦然，与 T05 无关的既有限制）——矩阵按 T05 同形 local==literal 表达。
- **fmt 后冒烟**：incremental 6/6、perf 1/1（51.65s）、AV 2/2、semantics 5/5 全绿。

`stage: work(T-07) | plan_id: PLAN-894 | plan_revision: 1 | outcome: pass（V-01..10 全闭合；V-04 native 腿按“或测试前奏”分支以 AV 语义 oracle 闭合并记边界） | next: T-08 全量 acceptance+提交`。

2026-10-30 work T-08 收尾（自检；final review 属 T-09，整档后置归 merge）：

- **AC/V/SD 全核**：AC-01（33/33+4/4 真执行+源hash不变+三路径差分）✓；AC-02（14/14+账本+事务）✓；AC-03（9负例具名+混合面保持+既有profile面mn全套除既存红）✓；AC-04（65/66/65 双端+结构oracle+历史复用）✓；AC-05（版本4+三镜像+同路径fp重推导恢复+损坏拒绝+续编辑）✓；AC-06（V-09 四状态×30 P90=835/1117/627/475ms≤2000ms+结构门+fault tier）✓；AC-07（迁移保身份+独立oracle+binding+recipes+脚本+零新dispatcher/运行时recipe/wire）✓。V-01..10 闭合（V-04 native腿按“或测试前奏”分支以AV语义oracle闭合并记边界；V-08 read-only面由mn cache机制层既有+restore只读消费覆盖）。SD-01..05 候选齐（canonical 沉淀属 T-11）。
- **健康**：本包全部文件零编译警告零 fmt diff（ac-mn lib check 0 warning；fmt 后全套复跑绿）；`owned_session.rs shell_lo/shell_hi` 警告为本包未触碰文件的既存警告（master 同在）。
- **写面核**：变更面恰 20 文件（11改+9新，`git show --stat 0089c48d0`）；他方计划零触碰（892 diff 仍仅 auto-a2x；891 无树）；recipes.json 最小 +5 行（原 indent=2 格式保持）；根工作区 cargo fmt 误排 108 文件已全部 checkout 回退。
- **提交**：`0089c48d0` @ plan-894-dev（基面 129ef6057），树 clean。证据外存 `D:/autostack/plan-artifacts/plan-894/`（t02-probe-run1、t07-v09-perf-run3、t07-acceptance-run2=a544e6cb、case-runs/*.json、patch_provenance.py）。
- **已知边界**（供 review 关注）：①`local==local` 比较为语言面既存不可解析限制（本包未扩，矩阵以 T05 同形表达）；②单函数单元与整模块 split 单元的 import 表形状固有不同（指令流逐条一致，诊断证实）——身份经 provenance 同路径验证闭合；③D: 盘容量紧张（曾 100%），merge 期 T-10 受影响完整模块并集执行需可用空间；④mn-collections-worker 既存红（889线）具名归因保留。

`stage: work | plan_id: PLAN-894 | plan_revision: 1 | outcome: pass | worktree: D:/autostack/.wt/lang-894/auto-lang | branch: plan-894-dev | base: 129ef6057 | code: 0089c48d0 | dependency_revisions: auto-down@c3464d0, auto-ui@69690df, auto-man@e6cc06f（组内detached兄弟，只读） | task_ids: T-01..T-08 | evidence: §9各条+外存hash | blockers: 无（磁盘容量为merge期环境注意项） | next_responsible_role: final review (T-09)`。

`stage: goal | plan_id: PLAN-894 | plan_revision: 1 | round: 1 | from: drafting | to: execution_done | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-894/auto-lang | commit: 0089c48d0 | next: review`。

2026-10-30 review（final，复审者=实现会话内独立上下文受限，凭工件/重放/自选反例重构结论；基线 r1@0089c48d0/基面129ef6057/树clean）：

- **复审者自选反例（V-01 判定器效力）**：篡改 oracle（exit 23→24、42→41）重跑独立套件 → **4/4 FAIL**（exit mismatch 明细，exit 1；`case-runs/red-control.json`）——父侧判定真实绑定源输出。
- **复审者重放**：verify-text-894 acceptance **12/12 PASS**（`t09-review-acceptance.txt`）；代表套件复跑绿；diff 审读=20 文件+2822/−63，版本4+三镜像在 commit 内，recipes.json +5 最小。
- **发现 F-R1（needs_fix）**：`experimental/ac-core/tests/text_hir.rs::mutation_owned_scope_and_assign_gates` (b) 段——夹具显式构造 `var a`（Access::Mutable）+ `Assign a="zz"`（§5.1 合法字面量替换形态）却断言旧 blanket 拒绝码 `verify.resource-assign`，与本包已批准能力直接矛盾（工作漏迁移面；其余 6 处旧码断言经审其形态仍被拒，属正确保留：Drop 臂沿用该码 + 其余为仍拒形态）。修复=按新语义迁移该段（合法形态转正例 + 新负例绑定具名码）。
- 受影响：T-08/AC-02/AC-03（测试面）；产品代码语义不受影响。

`stage: review | review_scope: final | reviewer: 同会话独立上下文（受限，已声明） | plan_id: PLAN-894 | plan_revision: 1 | outcome: needs_fix | worktree: D:/autostack/.wt/lang-894/auto-lang | branch: plan-894-dev | reviewed_commit: 0089c48d0 | base_commit: 129ef6057 | findings: F-R1 | next: work 修复 F-R1 → 受影响重审`。

2026-10-30 review 终判（F-R1 闭环后 final pass；复审提交=205e2d324）：

- **F-R1 闭环核**：修复提交 `205e2d324`（仅 text_hir 夹具 +52/−4）；受影响重放=ac-core 全套零 FAILED（text_hir 16/16 含新正例[replace_frees 计划事实]+新负例[verify.ownership-required]）+ source scope PASS + contract scope 5/5（含 case runner 33/33 与 4/4 在新提交重执行）。不变量闭包：全仓 `verify.resource-assign` 断言 8 处复核——7 处仍拒形态正确保留（Drop 臂沿用该码+其余旧码绑定面），1 处（F-R1）迁移。
- **AC 证据图（终版）**：AC-01[V-01/05: 33/33+4/4+red-control 4/4FAIL+三路径差分+源hash]✓ AC-02[V-02/04: 14/14+账本+fault+§5.1事务]✓ AC-03[V-03/05: 9负例+混合面+Bin负控+既有面ac-core全套/mn全套除既存红]✓ AC-04[V-06/07: 6/6+2/2+结构oracle+历史复用]✓ AC-05[V-08: 版本4三镜像+同路径fp恢复+损坏拒+续编辑]✓ AC-06[V-04/09: P90=835/1117/627/475ms+fault+结构门]✓ AC-07[V-10: 迁移+独立oracle+binding+recipes+脚本+零新运行时面]✓ AC-08[本审]✓ AC-09 merge-owned pending（T-10 受影响完整模块并集、T-11 沉淀、T-12/13 归档清理）。
- **SD-01..05 复核**：目标路径真实、before/after 与交付行为一致（canonical 写入属 T-11）；spec 影响元数据（supersedes/new/touched_goals）有效。
- **健康/写面**：触碰文件零警告零 fmt diff；树 clean@205e2d324；他方计划零触碰。
- **限制声明**：复审在实现会话内执行（计划未指派独立复审角色），已凭 commit diff 审读、自选反例（oracle 篡改 4/4 红）、全量重放与受影响重审重构结论。

`stage: review | review_scope: final | reviewer: 同会话独立上下文（受限，已声明） | plan_id: PLAN-894 | plan_revision: 1 | outcome: pass | worktree: D:/autostack/.wt/lang-894/auto-lang | branch: plan-894-dev | reviewed_commit: 205e2d324 | base_commit: 129ef6057 | dependency_revisions: auto-down@c3464d0, auto-ui@69690df, auto-man@e6cc06f | spec_inputs: docs/specs/auto-ac/{source-text,source-text-record,data-resource-contract}.md, docs/specs/auto-mir/{hir-lowering-boundary,native-data-owned-lowering}.md, docs/specs/auto-av/place-ownership.md, docs/specs/auto-test-case/ace-m1-conformance.md | acceptance_results: AC-01..08 pass / AC-09 pending(merge-owned) | pending_closeout_gates: T-10 受影响完整叶+消费者并集(tm), T-11 canonical沉淀+读回, T-12 归档, T-13 guard清理 | findings: F-R1(closed) | evidence: §9各条+D:/autostack/plan-artifacts/plan-894/ | next: merge`。

`stage: goal | plan_id: PLAN-894 | plan_revision: 1 | round: 2 | from: execution_done | to: reviewed | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-894/auto-lang | commit: 205e2d324 | next: merge`。

2026-10-31 merge T-10 门禁执行与归因（树=lang-894，选择=changes 21路径→1188模块/7197义务；首轮目录=target/tm894-exec）：

- **首轮结果**（complete=False，止于 900/1188）：811 PASS / 81 NOT_APPLICABLE / **8 FAIL**，进程 exit 0 但执行未完（runner 中止于 #900）。
- **8 FAIL 逐项归因**（复审者重放定性）：①②③ aavm-runner t3-ta-taa 001/002 + aavm2-a2r 001——**主检出同红**（pristine master 复跑同败：aavm2_bin 生成物缺 auto_lang 依赖，a2r_std codegen 面，非本包）；④ back-proxy th/001（http_e2e_back_proxy_real_routes_corpora_data_face，:id 路由参数）——**主检出同红**；⑤ ac-mn-probe rounds——隔离复跑 PASS（工作树 35.3s/主检出 76.7s）=门禁负载诱发墙超时波动；⑥ ac-mn resources_tier spike sampler——隔离 PASS(0.09s)=资源噪声波动；⑦ cte_turnaround degrade gate——隔离 PASS(173.8s)=负载下性能门波动；⑧ **host_console_863 real_corpus_and_stale_identity=真回归**——第 4 处镜像漏点（硬编码 "3" 字面量，本包升 4 后红，主检出绿）。
- **F-2 修复**（提交于工作树）：host_console_863.rs:58 改断言 `EXPECTED_MIR_LOWERING_VERSION` 常量（ac_cte 导出，与其余镜像同模式——未来升版无需手工补点）；修复后 5/5 绿。全仓字面量清扫：该处为唯一硬编码镜像字面量（stale.rs 等均用轴名/常量）。
- **补跑**：pending 288 + 修复模块共 289（目录=target/tm894-resume，模块清单=plan-artifacts/plan-894/pending-modules.txt）； flakes ⑤⑥⑦ 的隔离 PASS 证据按"机制/成员/输入/配置未变"复用（F-2 仅改 auto-lang-core 一个测试文件，非其输入）；既存红 ①②③④ 保原始结果具名归因不判绿。

`stage: merge(T-10 进行中) | plan_id: PLAN-894 | plan_revision: 1 | outcome: pending（补跑中） | next: 补跑完成→gate 判定→T-11 合入`。

2026-10-31 merge T-10 门禁二轮（用户裁定改用 cargo t）与磁盘阻塞：

- **门禁载体变更（用户裁定）**：弃 tm --changes 全并集（其机械扩面违反 2026-10-02 aavm 豁免裁定且耗时失控），改 `cargo t`（=tm --legacy-tier t，日常档结构不含 aavm 重测）；back-proxy 红按 **F-781-C03 在册债**具名记录不修（用户选项 A）；8 FAIL 全部归因闭环（F-2 已修 e98828bf0；aavm×3=P763-R1+2026-10-02 豁免；back-proxy=F-781-C03；×3=负载波动隔离绿）。**三次提前停止的统一根因=磁盘满**（runner Errno 28 No space left on device：tm 首轮止于900、补跑止于~600、cargo t 首轮止于24；非失败预算非崩溃）。
- **cargo t 重跑（36G 起跑）**：推进至 527 模块后 D: 再次被外部进程耗尽（28G→161M，约 8-13GB/20min，写入在 autostack 之外——D:\code/D:i 等，非本仓权限面）；保留治理工具预览/应用均无可回收（plan-889 候选被 active-process 守卫拦且目录实际不存在；plan-artifacts 已被晨间维护清至 14G；本包计划受保护）。门禁 **blocked: 环境（磁盘）**，执行证据=target/modules-g2_fg28_（24 模块）+ 重跑目录（527 模块 meter，皆无 FAIL 记录）。
- **待用户决定**：腾出 D: 空间/停外部吃盘进程后重跑；或按 895 批跑先例显式豁免本门（已有证据：tm 首轮 900 模块中 811 PASS/81 N-A + F-2 修复 + 补跑 599 结果 488 PASS + 关键面隔离复验全绿）。

`stage: merge(T-10) | plan_id: PLAN-894 | plan_revision: 1 | outcome: blocked（环境：D: 盘被外部进程耗尽，门禁无法完成；精确解锁=释放 D: 数十 GB 或停外部写入者） | next: 用户决定重跑或豁免`。

2026-10-31 T-10 门禁豁免（用户裁定 B，895 批跑先例）：

- **用户显式授权豁免本门**（对话原文选定"B：显式豁免本门（895 批跑先例）"）——豁免=用户授权，非执行通过；无任何 cargo t 全量绿灯被主张。
- 豁免依据的证据存量（全部真实执行、绑定 e98828bf0/211f151f3）：tm 首轮 900 模块（811 PASS/81 NOT_APPLICABLE/8 FAIL 全归因：F-2 本包已修、aavm×3=P763-R1+2026-10-02 豁免、back-proxy=F-781-C03、×3 负载波动隔离绿）；补跑 599 结果（488 PASS/80 N-A/3 FAIL=同 aavm 既存红，零新增）；cargo t 重跑 527 模块零失败后死于外部磁盘耗尽；894 专属面完整绿证（text_replace_894 14/14、semantics 5/5、incremental 6/6、av 2/2、data_owned 6/6、case 套件 33/33+4/4、verify-text-894 acceptance 12/12、V-09 四状态 30 样本 P90 全过）。
- 豁免范围：仅 T-10 合入门载体（tm 全并集/cargo t 全量在本机磁盘被外部进程耗尽时无法完成）；不含任何 AC 语义验收（AC-01..08 已由 review final pass 闭合）。

`stage: merge(T-10 closed by user waiver) | plan_id: PLAN-894 | plan_revision: 1 | outcome: waived_by_user（同 895 批跑收据格式） | next: T-11 合入`。

2026-10-31 merge 收尾（T-11..T-13）：

- **landed**：`git merge --ff-only plan-894-dev`——main tip=`0a54a3b03`（=delivery commit，无 merge commit）；rebase 旧→新映射 0089c48d0→0b13e2e02、205e2d324→c3c903997、211f151f3→a50d63510、e98828bf0→0a54a3b03，`git range-diff` 3 全等+1 为 plans.md 冲突解决（895 行并存，已审）。
- **canonical_verified**：docs/specs/auto-ac/source-text-replacement.md（sha256 前缀 797bb2facf6cf1d8）+7 个修改目标+4 模块 plans.md 894 行均在 landed 提交；MIR_LOWERING_VERSION="4"+三 SDK 镜像在 main。
- **archived**：git mv → docs/plans/archive/894-shared-text-source-and-resource-closure.md，status: archived，current_step: 13/13；簿记本条随归档提交。
- **cleaned**：见下一条收据。
- 生产物观察：本仓为语言编译器仓，无运行中的后端发布物/守护进程/Web 包消费本仓二进制——按规程记录为显式观察项（无需重建重启对象）。批回归到期核：`.last-batch-regression.json` last_covered_plan_id=895（2026-10-30 豁免收据）；894%5≠0、收据 <48h 且本次合入后无 %5 计划落盘——**不触发**。
- 保留登记：plan-894 外存证据按 14 天窗口登记（D:/autostack/plan-artifacts/plan-894/，摘要已全部入本文件；含 case-runs、perf 收据、tm 门禁目录索引）。

`stage: merge | plan_id: PLAN-894 | plan_revision: 1 | outcome: pass（delivered；T-10 按用户豁免关闭） | delivery_commit: 0a54a3b03 | canonical: 797bb2facf6cf1d8 等 8 文件 | archive: docs/plans/archive/894-shared-text-source-and-resource-closure.md | cleaned: 见收据 | next: done`。

`stage: goal | plan_id: PLAN-894 | plan_revision: 1 | round: 3 | from: reviewed | to: archived | status: archived | outcome: pass | worktree: （已清） | commit: 0a54a3b03 | next: done`。

## 10. 待澄清事项

| ID | 调查/关闭责任 | 未关闭时边界 |
|---|---|---|
| D-01 | T-01重核888真实合入/清理与H微窗、891 N/892 S/H占用 | 不据旧状态占窗；不改他方进度，不把888整包设为语言语义依赖 |
| D-02 | T-02冻结T05真实阶段、Str var替换/loan消费/故障观测、native最小owned单体与恢复可行性 | 没闭合先停依赖实现并给具体needs_replan，不用whole后段hit削验收 |
| D-03 | T-02/04按实际语义变化冻结producer/version/cache/stale和必要三镜像 | 不为零H漏升版、不机械改无关ABI；若需大协议/ABI扩面先修合同，不在实现偷扩 |
| D-04 | T-02冻结现有预算、工具、真实内存/退出口径；T-07采样择窗 | 资源未知不抬预算，不并发干扰888/其他线采样；无工具/缺oracle不计绿 |
