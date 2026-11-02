---
plan_id: PLAN-912
status: archived
feature_name: 闭包源码单函数前端生产收口
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-02
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-ac/function-incremental-source.md, docs/specs/auto-ac/source-closures.md, docs/specs/auto-ac/shared-mutable-captures.md, docs/specs/auto-mir/closure-capture-profile.md, docs/specs/auto-mir/hir-lowering-boundary.md, docs/specs/auto-av/resident-function-install.md, docs/specs/auto-av/cache-and-conformance.md, docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-ac/project.md]
new_spec_components: [docs/specs/auto-ac/closures-function-production.md]
touched_goals: [GOAL-002]
affects: [auto-ac, auto-mir, auto-av, auto-test-case]
depends_on: [PLAN-909, PLAN-828, PLAN-868]
current_step: 12
total_steps: 12
---

# [PLAN-912] 闭包源码单函数前端生产收口

## 0. 变更摘要

承909真实共享捕获/K06与owner级复用，关闭R1模块Atom发射/绑定、R2整模块HIR验证、R4同owner兄弟lambda重走，并消除支持的单body暖编辑中隐藏的整源body解析。补Closures域真正的单函数凭证/资源事实及单体lowering消费，未改函数在真实source→HIR→MIR→ABC→安装全链生产/工件IO=0、同对象/代际。

**本轮拆两包**：本包912只交进程内生产收口；[913来源绑定恢复](913-closures-source-bound-recovery.md)另交R3新进程恢复前置生产及恢复后的编辑。原因：前者需源片段/类型表/凭证，后者需独立的持久来源锚点/发布/required闭包/深验；可各自真实验收，不把两个入口的命中混算。架构已先沉淀[Design43 §16](../design/43-ac-av-typed-mir-and-abc.md#16-909后继闭包源码生产粒度与恢复入口2026-11-02target-state)。

drafting r1、0/12，仅规划。不建实施树/材料，不编译/测试/探针/认证，不改生产/current Spec/原source/行为expected/binding/911合同。912交付不自动关闭R3、完整C-19或签M1，不回开909。

## 1. 目标

- G1：原closures域纯body修改只生产被改函数/lambda及实际依赖者；同owner未改兄弟与其他owner每个阶段真实0，不能whole后filter。
- G2：完整闭包单函数授信：稳定逻辑binding/lambda/type身份、受检接口上下文、实际读取依赖、资源/capture/env事实、body/context凭证与唯一共享lowering。
- G3：候选在语义/链接/资源失败或取消前后保正确事务边界；Span变化不重建未改对象，代码及外部pin生命周期不退化。
- G4：预算/真实attempt/正式P90与独立文件化oracle齐备；交付913所需accepted source state只读合同，既有恢复路径重验保持，R3仍明示。
- G5：与911具体S/H短窗、工具/SDK/所有实际consumer接线完整；不假称当前所有body路径或其他backend已同样支持。

非目标：新语法/共享owned/cyclic/跨挂起借用、native/A2R闭包、默认切换/旧引擎退役、原D-909-1 native生命周期红修复、通用多模块增量平台、持久恢复manifest/纯恢复前门（913）、全局cache/GC重构。原909支持域和资源语义保持，不为减少分母改源/expected。

## 2. 架构方案

`冷初始化checked声明/类型/捕获目录 → retained source/CST及单body candidate → 单体resolve/typecheck → body-only Atom/descriptor → 闭包单函数verify/资源与capture事实 → checked function context view → ace-lowering → 单MIR/ABC/安装`。

已有emit_function_body/prepare_candidate_document、verify_function_ref与per-unit门可复用，但**verify_function_ref目前明确排除Closures**。本包须为闭包全义务建立验证入口/受检上下文，不能只放宽域match、用旧CheckedModule换切片或在lowering猜所有权。cold whole-closing truth路径保兼容；支持的warm路径不再调全module body生产。

## 3. 技术栈

现有ACE core/source/descriptor/verify、增量CST/片段、FnEnv/body facts、唯一lowering、av闭包session、ManagedStore与AutoDown/TestCase/固定组。新增类型/只读state view放现有crate内，不新workspace/runner/数据库。精确内部接口与stage字段由T-02最小全链冻结，文中拟新路径由对应T使可运行。

## 4. 需求分析与背景调查

### 4.1 授权与拆分基线

2026-11-02用户授权为909后继正式立项，核交付/残余、判断单包/拆包，明确零生产/恢复/资源/时间及911边界；本轮只规划。采用两包，均为该明确后继，不预占第三包。保多会话WIP，不提交混合修改、不建工作树/联系其他会话/启动agent或清理产物。

main `5a4eb4e257c4f5f4305f670bb52db482dc7e0917`；909 archived12/12、delivery`59b8107eb`为HEAD祖先(exit0)，原K06源码42和cell/Shared tag4真实交付，树/分支/材料清理已记收据。其R1..R4明示于归档§9；R5坏载荷/只读store沿既有测试，不当成零前置生产已闭。D-909-1为native lifecycle旧红，10/11冻结union绿、ace-host171/172，不能凭909归档泛称所有consumer绿。

911用户确认实施且树lang-911@5a4eb4e25实存，计划r2头滞后drafting0/10不代改；当前读取无列出WIP不等未开工。它拥有通用失败快照/typed拒绝/真实attempt与native/parent；912只沿既有观测共享扩面。910 executing4/9属于生成构建治理，不设语义前置。

### 4.2 来源/实际缺口

| 来源 | SHA256 / 静态事实 |
|---|---|
| docs/specs/auto-ac/function-incremental-source.md | `25afaa44f2bfa65f76bb8cd818c6a387a08d44c220d076d1d0dab863a6233de0`；owner复用，plan/bind模块级、兄弟lambda重走、restart前置生产 |
| docs/specs/auto-ac/shared-mutable-captures.md | `c7259dd23fcef1a73bb970938bca6c1a441eba5c5c6fd30f89b7a6d5d748022d`；当前BindingId依token偏移，模式/写回/clone/清理合同保持 |
| docs/specs/auto-av/cache-and-conformance.md | `8d895b127203b125874302a64f5ec1eef6a528be233042ca48de757849f946e4`；909恢复有read/decode/重验，仍先lower/encode |
| core/src/source/adapter.rs | `cb6352cff4f039f849f2bb9e719a5131b48388344bb9cfc668a97f4d4003eaa2`；compile_closures_source_incremental整parse→Emitter.plan_module/render→bind→verify_closures |
| core/src/source/resolve.rs | `4ecf2a34e2d54813e68c892650c357624e6e1d387c2c47cd790e2f8a3a7e3aca`；ClosuresOwnerCache/函数类型索引快照，CachedOwner含resolved与全部lambdas |
| core/src/verify.rs | `d6ac3861219bcdc4c787f3eaeb8bd04b201501e461d9ce724d1619736e1d80db`；verify_function_ref仅Core/Scalar/Record/Collections，不能冒Closures单函数凭证 |
| av/src/closures_865_support.rs | `e6ab4e46f946717cd8c98c00dc2014a478907ac634bf1aa34616da1b6c7f2da1`；reused owner跳lower/encode，改变owner仍遍历其各函数 |
| av/src/closures_session.rs | `9797e09aa8ab3257951efeeae642dc38b88a34b2b5bcc2a4d35d5a71211ba4c0`；per-unit verify/encode/admit与对象/代际/8MiB历史门可复用 |

实际支持源/typed域见source-closures、shared-mutable-captures、closure-capture-profile；当前新接口/对外源warm形态均未实现，本轮未跑语言。新scope不是降低909语义oracle，而是补其已明示生产粒度。

## 5. 详细设计

### 5.1 支持的编辑与稳定身份

支持原closures域内现有顶层函数和lambda单body编辑，必须包含**同owner一个lambda body变、另一个lambda不变**；纯表达式/块体、Copy私有可写/共享cell、原owned move/clone/非逃逸Borrow均保语义。T-02冻结base全文SHA+UTF8 range+目标逻辑单元的显式编辑入口；原apply_source可用差分包装，但计量窗口包括其diff/解析/辅助工作，不能另藏整源parse。仅已有函数片段API不代表已能投影lambda，须最小真链证实。

源生产只处理变更body。有限声明/类型目录、输入hash、增量lex收敛和坐标投影列M轴，不读其他body做语义工作。签名/捕获集合/顺序/mode/effect或共享分组变，按真正读者失效；decl/schema变、语法不收敛/多编辑无法证明对应时明确non-fast-path。固定普通单body样本不可统统fallback；拓扑增删/重排至少保不相关owner身份，能唯一匹配的存活单元保身份，歧义不误命中。

stable function/lambda/binding ID属于逻辑身份，不直接用易移动token偏移或owner内序号当永久键。源码坐标变动、同名遮蔽/不同激活、lambda相同文本/用户声明名碰撞都不可串单元；body-only插入注释/UTF8变长不改兄弟身份。显示Span/mint/current映射和语义key分开；不为调坐标换未改HIR/MIR/code Arc。保兼容需要的旧id展示与新内部身份映射T-02冻结，不添加语言保留字躲碰撞。

909的u64文本指纹只能作变化提示，不是受检来源/凭证的唯一证明。新warm context/cache由受控构造持有，绑定完整source/目标body强摘要与owner/context，普通调用方不能填公开缓存字段铸造已检查body；同hash提示/同上下文但异body的控制必须拒。source profile/descriptor/capture/IR/lowering的实际语义身份进入accepted state，不能只写不随实现变化的family常量充完整producer。

### 5.2 闭包单函数受检事实与依赖

冷声明/类型/创建点registry受检后驻留，warm仅更新必要部分。registry承担跨body原义务：closure callee/env声明、创建site唯一/关联、禁止直接调用lambda body、名义类型/资源/捕获mode及已支持env域；不能删除全模块验证就丢这些门。单体凭证只由验证器mint，绑定owner/domain/语义body digest/实际context与所读事实，携带本body完整资源/capture/cleanup事实；serialized/public bool/重切旧CheckedModule不能授信。

每个lambda保创建点可见的词法环境/作用域、初始化与mutability、借用来源/生命周期和param规则，不把整个owner所有locals都暴露为可捕获变量。暖body中新捕获已被遮蔽、尚未声明、未初始化或不可提升的绑定必须保持原诊断；lambda参数/内部局部不误当外捕获。依赖或声明变化使这份context过期时先重验相应目录/受影响单元，不拿旧快照扩大可见性。该context/旧摘要的复用必须可证明不走未改lambda的AST/语义body。

复用scoped emitter/binder、补Closures原有数据面，其他body只作无实体scaffold或受检引用；类型表身份按名义声明/规范类型图稳定，不因intern append/无关type索引变化全失效。创建点与lambda所读捕获分组/模式/效果是body事实；callee签名未变但实际读的type/meta/实例/CT/effect变化须失效，不凭调用点源码未变命中。未读声明变化不得顺带重走其他body。合法hit复用已有凭证/对象，保实际读取集，不把读一遍当刷新旧凭证。

单函数lowering接受真实凭证+受检context view，完整Shared/private/env/资源事实来自验证器；不得新增后端资源分析或强行铸造CheckedModule。保TextResourcePlan既有verify职责、其他profile完整正反控；闭包未支持consumer具名拒新形状，不因共享枚举扩面默当Copy/Move。

### 5.3 候选提交与913交接数据

语义/verify/MIR/链接/预算/取消在commit前拒绝时，current source/目录/凭证/已读事实/函数安装root/历史保持；新暂存资源只在自己scope，下一次有效编辑真可用。code commit之后的guest trap/cancel按原执行合同清理env/cell，不反称从未有compile/commit；T-02冻结compile-commit与run边界，不能为捕获同一层所有Err都报rollback。代码/源发布同一接受代，stale/foreign候选不覆盖更新代。

为913提供只读accepted source state（拟新接口，T-02冻结）：完整source强SHA/逻辑owner/entry/profile/config与producer版本；稳定decl/type/binding/lambda目录；各函数body/context/实际依赖digest、捕获/资源事实身份及既有MIR/ABC/wire/store引用；mint/current坐标映射、接受代与提交点。只在真实成功生产/安装commit形成，不含guest value/PASS、活env/cell/host地址；资源受管，不能复制一套不计账的大Bundle。此处data view不是持久凭证，913落盘后仍不受信；内存无store模式可照常执行，但不能冒已有恢复binding。

禁止的是运行观测result/oracle/PASS缓存，不禁止源字面量及按完整依赖冻结的编译常量。导出只来自对应操作已经接受的不可变state；若本操作在commit前失败，不能迟读last-state当作新源的接受证明。code已commit但guest运行失败时则按真实阶段记录，不将合法代码绑定与运行成功混为一事。

912只约定/交付数据和所有权/版本/消费者契约，来源manifest/信任锚点/持久发布归913。接口以正式SHA及验证收据交接；913可先设计codec/反控，实际串接在D912之后，接口变更标受影响证据STALE。本包保原909重启深验/零写回行为并测其实际前置生产非零，R3不关闭。

### 5.4 实际零生产、验证与强制红控制

按每个逻辑函数及whole轴观测真实attempt：source parse/project、resolve、typecheck、Atom plan/render、descriptor bind、HIR verify、MIR lower/verify、ABC encode、decode/admit、code-index构建与工件IO；失败早退也取完整快照，未到达0/未观测gap。暖支持路径未改body所有P轴=0且相同HIR/MIR/code对象/代际，声明脚手架/M轴可非零但不得含body。只有改函及真正依赖者可生产；同owner兄弟明确检查，不能以owner复用统计冒逐lambda。

必须实际触发旧whole parse/plan/bind/verify后filter、owner内全部lambda重走、同hash换新对象、早退Default0/只成功后计数、遗漏capture依赖等错误机制使门红，再恢复合法控制；不只静态搜函数名或把合成counter写成0。源、过程、对象与独立result/资源oracle共同判；不能为证明计数再调用一次compiler。

恢复阶段R与元数据M单列：912已有restart可能P>0，如实记录交913；不把HIR源验证重跑藏到“恢复重验”字段。913未来纯恢复允许decode/深重验，但二者同一阶段定义/输入角色须在本包导出，禁止改名洗零。

### 5.5 资源与正式时间门

本次拟新增保留前端预算≤16MiB：current/candidate/history与实际外部pin、AST/HIR/已验事实/类型intern/索引合计；共享backing至少可追踪，不能漏在旁路map。原source≤4MiB、code current+history合计8MiB/每unit32、活值含cell/env8MiB、verify analysis/decoded module各16MiB原更严门保持。父private/working-set记录原更严cap与相对冷基线增量≤128MiB（拟新门）；不能以RSS单点代峰值。T-02先证预算可行，不能默加大/缩规模。

512修订覆盖不同lambda/捕获类型/历史命中，类型intern/候选/索引有界回收；旧源/code/context pin到最后owner释放才算回收，超预算保原已装线并Resource拒新候选，不提前清pin/只清history向量报0。真实oom/fault/budget/wall/cancel/trap/close各出口分态，live cell/env/owned按原门，关闭代码cache不等活closure已释放。原3072层/256KiB栈与823/909机制/样本不缩，不为本包引入任意语言深度上限。

文件化实际输入+独立手写参考/oracle覆盖fresh/hit/body-edit/restart(旧行为重验分账)/历史/故障后恢复；四状态各30样本，P90≤2000ms与正确性/结构/资源同时过，保原更严门和长期亚秒。准备/冷编译/发现/执行/清理/整命令、M/P/R与父峰值分列。正式native旧红D-909-1与本包closure门分诊；不能改其expected、缩本包样本或靠周期回归豁免盖新回归。

新正式多函数规模至少64个顶层函数、每个被测owner至少3个兄弟lambda，函数从entry真实可达，分别编辑同owner首/中/末lambda；单独保留原909语料/规模和资源深链。T-02先冻结具体长期source/hash与手写reference、实际函数/lambda数、改动/依赖集合及手算结果，T-07不得换成单函数小例取得P90。64/3是本包测试规模，不是语言支持上限；512 churn与额外深链资源门另列，不把累计用时当一次编辑。

### 5.6 911与公共写面

| 面 | 912主责 / 交接边界 |
|---|---|
| S-FE | core/source parser/resolve/adapter、稳定身份/类型目录、descriptor/scoped bind、Closures单函数verify。911不改语言/捕获规则；912不改其native source-v1/三族/首失败策略 |
| S-OBS | core/src/lib.rs::body_work、verify调度、通用候选/只读事实出口；911 W-S1/2同源attempt/不可变失败快照，先核其正式/稳定已提交接口与owner，按字段/函数单写；不各造一套统计或改其默认显示 |
| P | 唯一experimental/ace/crates/lowering、av/closures_865_support与必要closures_session；不碰native错误链/活捕获语义/AV指令ABI |
| H | ac_av闭包source分支、contract/worker/版本/测试注册/SDK与Spec导航；911改ProbeMnSpec/ac_mn及B17–19，912只改闭包源相关符号/用例。全文件单写短窗、不能append-only并写或提交他方WIP |
| M1数据 | 原K06=42与共享suite/原oracle保持；新增测试输入/manifest hash具名更新，不改B17–19/NA/required/token凑绿；911已接数据再变只重核受影响 |
| A→B | accepted source state类型/身份/资源/阶段定义由912交付，913在D912正式基面后消费；不把B验证推给A、也不让A宣称R3已闭 |

窗记录owner/revision、实际SHA、符号/类型/构造/匹配及全部consumer、source/worker/schema身份、已跑/pending与释放状态。新增Rust字段/枚举不等天然兼容；接口再变STALE受影响重核，最终review绑定真正整合SHA。911 N/parent可与本包私有P/设计并行，S/H以当前owner释放为准，不索D911整包；未取窗只阻具体任务，必需门不能延期交付。910只按真实工具/注册交集分窗，正式采样错峰且不持写窗等机器。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/closures-function-production.md | 无→实际body入口/支持矩阵/凭证/依赖/计量/预算 | 原owner命中不冒逐函数0 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-ac/function-incremental-source.md ; docs/specs/auto-ac/source-closures.md ; docs/specs/auto-ac/shared-mutable-captures.md | R1/R2/R4+token偏移/序号绑定→单body/稳定逻辑身份与真实依赖 | 保SEM-04及坐标/语义分域 | AC-01/02 |
| SD-03 | modify | docs/specs/auto-mir/closure-capture-profile.md ; docs/specs/auto-mir/hir-lowering-boundary.md | whole Closures凭证→单函数闭包完整授信/事实消费 | 不跳过site/env/资源门 | AC-02/03 |
| SD-04 | modify | docs/specs/auto-av/resident-function-install.md ; docs/specs/auto-av/cache-and-conformance.md | owner提交→真实单函数生产/原子安装/accepted state；R3仍913 | 安装/恢复不混验 | AC-03/04/05 |
| SD-05 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 原K06/共享suite→新增阶段/资源/强制红证据，原oracle不改 | 首例与真实闭环 | AC-04/05 |
| SD-06 | modify | docs/specs/auto-ac/project.md | 现状导航→新凭证/源入口/913残余及窗口 | current可定位 | AC-05/08 |

## 6. 测试设计

拟长期`ace-m1/closures-fn-912.case.ad`、`.suite.ad`及`closures-fn-912/{src,reference}`由T-06启用，引用原K06和909既有source/oracle，不造a2r金样/证书。Rust扩现有core/lowering/av的conformance cases；每例稳定ID，实际step/result/trap/事件与计数由parent判，不拿同bug两backend对拍当独立oracle。以下均待执行。

| V | 实际入口/输入 → 独立关闭oracle与强制红 | AC / owner_stage |
|---|---|---|
| V-01 | 原K06/909共享与clone/逃逸/char等源→新冷/暖链；42/43/44/82/84等原手算与typed结果保持，不隐式clone/换共享语义 | AC-01/05 work T-02/06 |
| V-02 | 同owner两lambda仅改一body，带多个free fn；改者结果按新增手算oracle变，兄弟及其他函数P全0、same对象/代际；旧owner全走反控红 | AC-01 work T-02..05 |
| V-03 | UTF8变长/位置移动、遮蔽/相同lambda/用户合成名碰撞、创建点后声明/未初始化/参数新捕获；逻辑ID/坐标与创建点可见性正确，不串绑定或误扩作用域；合法邻例过，歧义明确非快路径 | AC-01/02 work T-03 |
| V-04 | 单函数typed凭证、body-less目录；同context异body/外owner/profile/坏site/env/mode/cleanup/直接call body | verifier前拒且合法非零ID正控过，不铸旧CheckedModule；miss只验证受影响函数 | AC-02 work T-02/04 |
| V-05 | 捕获mode/顺序/布局、callee签名未改但所读effect/type/实例/CT事实变，未读事实变 | 真读者精确失效；其余全0，普通caller body未改不误hit；不能粗全模块context fp凑正确 | AC-02 work T-03..05 |
| V-06 | 真片段parse/project/resolve/type/plan/bind/HIRverify/lower/ABC/安装调度及whole轴 | 强制旧whole后filter/隐藏窗口前重算/Default0/同hash新对象各使gate红；有限目录/M轴另列 | AC-01/04 work T-02/07 |
| V-07 | frontend/verify/MIR/link/预算/cancel候选拒绝、stale/foreign候选；之后合法编辑 | commit前旧source/context/roots/pin与事实保全，错误后真正下一次read可用；commit后guest trap不假称从未提交 | AC-03 work T-04/05/07 |
| V-08 | zero source hit、历史改回、只读store/坏载荷/原909 restart | hit零生产/历史真复用；restart实际decode/深验/执行且如实记录R3 P>0；不能提前声称纯恢复 | AC-03/05 work T-05/07 |
| V-09 | current/candidate/历史/外部pin、512不同签名/捕获修订、最后owner/close | retained front≤16MiB、旧code/value cap不增，intern/索引有界；不以向量清空冒live=0，超额保旧版 | AC-03/04 work T-07 |
| V-10 | 逐本包新增分配点fault与budget/wall/cancel/trap | 错误分类/prepare回滚及临时释放恰一次；原live cell/env归基线，pin未释放如实非零 | AC-03/04 work T-02/07 |
| V-11 | accepted source state导出，含函数源/语义/依赖/context/wire引用且无guest PASS | 只来自已接受commit、资源计账/只读，不从失败candidate导出；913能独立核contract，元数据不冒持久凭证 | AC-05 work T-05/07 |
| V-12 | 911只读失败/attempt接口、旧scalar/record/text/collections与native/A2X必要consumer | default/语义码/资源事实不退化，枚举/字段的实际consumer齐；接口再变STALE，不借旧pass放过新差异 | AC-05 work T-01/07 |
| V-13 | 四状态各30样本、原规模/深链+512 churn、父峰值 | P90≤2000ms与P=0结构/独立结果/原预算同过，准备/运行/清理与M/R分账；R3未闭明示 | AC-04 work T-07 |
| V-14 | parent/case/注册/manifest漂移、漏选/重复/SKIP/伪counter/旧tool | 必需集合与身份严格，不用退出0冒PASS，原source/expected/NA及其他格保护，正式父证据重读 | AC-05 work T-06/07 |

组选按实际完整职责：`cargo t ace-source ace-semantics ace-lowering ace-av`；shared HIR/verify变更补`ace-native ace-a2x`实际下游，宿主/decoder触面补`ace-host ace-adapters auto-test-case`，注册/治理选`infra-fixed infra-ace-m1`；原源组`case-ace-m1-closures-865-source case-ace-m1-shared-mutable-909`，拟新增`case-ace-m1-closures-fn-912`（all=false、显式ACE，T-06注册）。正式性能沿`ace-performance`原条件/样本，仅本包新/受影响命名case；完整职责/必要consumer最终以diff具名冻结，不自动全仓/13慢例。D-909-1保首红与旧基线对照，若因本包成新回归不可沿旧债豁免。

内部自查不代替同树独立上下文final review；未指定reviewer/model，不建agent/chat。证据相同身份可复用，修复/输入漂移仅受影响重验；首红/环境/重放/未执行分别保留。独立review逐AC/V/SD/真实调度/原oracle与遗漏延期/健康核验，绑定revision/SHA。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01/02/03/06：原源语义不改，普通同owner lambda body编辑真实仅改者/依赖者全阶段生产，未改P/工件IO0、对象/代际保留，无隐藏整源/whole后filter |
| AC-02 | work | V-03..05：Closures单函数凭证+受检context覆盖原site/env/capture/资源门，稳定逻辑身份/类型及完整实际依赖，无同context异body/未授权命中、lowering不重做资源分析 |
| AC-03 | work | V-07..10：候选/提交/guest执行边界正确，坏/过期/取消/资源失败保旧、下一次有效操作可用，pin/清理/历史/只读与恢复重验不退化 |
| AC-04 | work | V-06/09/10/13：真实attempt含失败/whole轴与M/P/R分账、强制红门有效，16MiB新增池/旧预算/父峰值/四状态30样本P90及原更严门同时成立 |
| AC-05 | work | V-08/11/12/14：正式首例/独立oracle/父判决/完整组，911窗口和913accepted state版本/所有权/消费者收据齐，SD-01..06与R3/其余C-19/D909-1 owner明确，不冒M1 |
| AC-06 | review | 同树独立核AC-01..05/V-01..14/SD、源/身份/反控/遗漏延期/健康，revision+冻结SHA pass，不拿checked框或合成统计代实证 |
| AC-07 | merge | 冻结完整职责/必要consumer门或有效证据，真实合入/Spec读回/归档、材料与guard清理、12/12及A→B收据成立 |
| AC-08 | merge | 913可读取正式A的接口/阶段定义/预算/版本，未交R3继续具名；不将stage分拆变成放宽门或重开909、写旧ledger |

## 8. 执行步骤

8 work+1 review+3 merge=12项，全部未勾。T-02必须先单lambda最小全链含真正Closures凭证和反控，不把整个平台未知机制写死后施工。扩大成通用多文件平台或提高预算先needs_replan，不能删R1/R2/R4。

| T | owner_stage | 前置/窗 | 工作与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施指令；D909；911具体S/H | 冻结主线/源/Spec/实际cap/所有consumer及唯一树/材料、阶段ledger与支持矩阵、A→B数据契约和窗口 | AC-01..05 | [x] 2026-11-02 树`D:/autostack/.wt/lang-912/auto-lang`@`plan-912-dev`基`master@5923d4dee`（兄弟auto-down detached 7c4d5cb）；材料`D:/autostack/plan-artifacts/plan-912/`（t01/freeze.md + 四份只读代码映射）。§4.2九项SHA全部与HEAD一致，另冻结hir_adapter `8878216…`、lib.rs body_work `346307c…`、parser `5abc5d7…`、ac_av.rs `ad1bd98…`、av-worker `f4f43c7…`、test-groups.toml `eafd08f…`、MIR_LOWERING_VERSION="4"。实读核实R1(`adapter.rs:241-244`)/R2(`:248`，verify在body_work窗外无计数)/R4(`resolve.rs:1299-1317`整item文本fp，`:2915`内联解析全部lambda)/隐藏整源parse(`:227`)；`verify_function_ref`排除Closures(`verify.rs:244-257`)；scoped文档中creator的callee body检查静默跳过(`:4114`)；lowering扫全模块body推导env/cell(`hir_adapter.rs:806,861`)；`recheck_swap`丢lambda(`resolve.rs:2097`)；无lambda片段parser。911 diff实核：不触core/lowering/av closures/ac_av/av-worker——S-FE/S-OBS/P窗空闲；H共享仅test-groups.toml（双追加）+3份Spec（merge时协调）。阶段ledger P/M/R轴、支持矩阵、A→B `AcceptedSourceState`类型与D-01..05有界关闭已冻结于freeze.md §5-8 |
| T-02 | work | T-01；最小S/P | 两同owner lambda纯body改→源片段→单体plan/bind→真Closures凭证→MIR/ABC/AV一单元；兄弟0/同对象、失败和whole负控；冻结类型/上下文/数据/资源表示 | AC-01..04 | [x] 2026-11-02 commit `88af52b13`（树plan-912-dev）。**链**：`parse_closure_fragment`（仅目标lambda range词法/解析）→`resolve_lambda_in_context`（退回保留的`LambdaSiteFacts`创建点事实，raw capture_uses/params/move/result任一变即具名`LambdaNonFast`，不静默扩权）→graft+`check_body_in`单体→`emit_function_body`作用域文档（4脚手架+1真body）→`verify_closures_function(bundle, def, &ClosuresVerifyContext)`铸`CheckedFunction{domain:Closures, coll_facts, closure_facts}`→`lower_function_closures_unit_checked`消费凭证事实（`ClosureFacts` trait：`ModuleScanFacts`冷整模块推导原样/`CheckedClosureFacts`凭证）→spanless wire对比→`ClosuresUpdateSession`单单元→`commit_candidate`（源/SHA/目录坐标M轴平移，不走任何未改body）。**实证**（`av/tests/cases/closures_fn_912.rs` 3例、`core/tests/cases/closures_fn_912.rs` 6例、`lowering/tests/cases/closures_fn_912.rs` 2例，全绿）：手算oracle fresh 51→g body改48→纯空白同wire 48（submitted 0、对象同）→zero-hit 48→f停写base具名non-fast 28、失败后恢复50；暖改body_work仅`{__closure_main_1:(1,1,1,1)}`、whole轴0、P轴仅`d___closure_main_1`（parse_project/hir_verify/mir_lower/mir_verify各1、abc_encode 2=identity+canonical、decode_admit/artifact_write 1）；`d_main/d___closure_main_0/d_helper/d_other` Arc::ptr_eq且代际不变，改者新对象新代际；`WarmGateReport`零违规。**等价oracle**：凭证路径单元spanless wire == 整模块`lower_function_closures_unit`推导，MIR_LOWERING_VERSION保持"4"。**反控**：空registry→拒（整模块路径对脚手架callee原本静默跳过的检查现为`verify.closure-env-unknown/closure-site`）；篡改capture local mutable→`verify.closure-env-mismatch`；脚手架目标→`verify.body-ref`；异body凭证→`lower.closure-facts`；`verify_function`对Closures→`verify.domain`；stale base/跨单元/类型错误保状态；865旧路径同窗口whole_parse/plan/verify≥1（强制whole即红）。**修正**：函数级text-kind表门对Closures须与`check_module`一致跳过（否则`t_str`预置项误拒）。**有界**：嵌套lambda与owner body改为具名non-fast（T-03）；owner级cache复用于non-fast路由待T-03。全组：ace-core conformance 285/285、ace-lowering 5/5、ace-av 826/826（run2/3；run1 820/6瞬时名未捕获，同码重放绿，环境归因）。材料`plan-artifacts/plan-912/t02-*.log` |
| T-03 | work | T-02；source S | retained语法/函数与lambda候选、稳定binding/type/ID与坐标、捕获依赖/作用域/读集、有限目录及缓存回收；V-02/03/05 | AC-01/02/04 | [x] 2026-11-02 commit `63a1ea067`。**owner路由+retained-lambda复用**：`decide_capture`为唯一捕获模式决策表；`LambdaReuse`/`try_reuse_lambda`——lambda表达式文本fnv64同且其每个保留捕获按**当前**创建点locals重推(mode/body_vt/writable/cell)全等时，从保留条目推入该lambda及其嵌套lambda（嵌套span按外层位移平移），不走其AST/语义body；`resolve_owner_in_env`经oracle重解析单owner（签名漂移=`OwnerSignatureChanged`非快）；真实走过的lambda记`record_resolve`（R4兄弟重走对所有路径逐单元可见——909两例ledger断言相应更新并注明）。**registry overlay**：`overlay_from_batch`从batch真实body重推creator各创建点（复用脚手架沿用保留env且site modes须等），仅在session安装成功后随`commit_candidate(c, Some(overlay))`落位。**候选多单元化**：owner+真正重解析的lambda进一份batch作用域文档（`emit_closures_function_batch`，复用兄弟保持脚手架零plan/bind）；lambda-only改若creation-site事实变→转owner路由（而非cold）；拓扑变化（lambda数变）=`OwnerTopologyChanged{before,after}`具名非快→cold重建但`cold_carrying`按(owner,文本)唯一匹配保留lambda逻辑ID（歧义/新增铸新ID）。**V-02/03/05实证**（av `closures_fn_912.rs` 7例全绿、core 6例、lowering 2例；ace-core conformance 285/285、ace-core lib 17/17、ace-lowering 5/5、av closures子集105/105）：owner body改(x 10→20)值61、路由`FastOwner{changed:[],reused:[f,g]}`、body_work仅main、两lambda+helper Arc同、submitted仅d_main；f停写base→`FastOwner{changed:[f],reused:[g]}`值28、submitted {d_main,d___closure_main_0}、g/helper Arc同、whole轴0；CJK变长owner插入后坐标重映射→随后g的lambda-only改路由`FastUnit`值48；创建点可见性：g引用其后声明的`later`→诊断且状态保全、参数遮蔽owner local→捕获集变走owner路由值55；插入lambda→`owner-topology-changed:2->3`具名非快、whole轴>0如实、f/g逻辑ID保持、z新ID。**边界如实**：owner路由的parse_project按owner item计1（lambda文本属owner body，无单独解析但无任何lambda语义阶段）；owner级`scan_shared_capture_cells`为语法扫描；嵌套lambda的lambda-only改仍转owner路由 |
| T-04 | work | T-03；descriptor/verify/lowering S | 分段发射/绑定、Closures单函数完整验证与resource/capture事实、受检view lowering及全实际consumer，V-04/06/07 | AC-01..03 | [x] 2026-11-02 commit `9f1db93e0`（分段发射/绑定、单函数验证入口、凭证lowering主体已在T-02/T-03落地；本项补齐验证义务与强制红）。**V-04**：原MIR-only的两项模块级义务现为HIR级registry拒绝——`verify.closure-body-direct-call`（把`helper(1)`重定向到同签名lambda g，仅闭包规则可拒）与`verify.closure-site-duplicate`（g的创建点改指f→overlay先拒）；复用脚手架的site mode被改→overlay `verify.closure-env-mismatch`；creator正控过；此前T-02已覆盖空registry/篡改env/脚手架目标/异body凭证/`verify.domain`。**V-06强制红**：`warm_gate_report`公开接收合成观测——changed单元无计数（Default-0）、隐藏兄弟walk、whole轴attempt、兄弟mir_lower泄漏、缺hir_verify各自为违规，干净观测pass；session级`set_degrade_resubmit_unchanged`从驻留MIR重铸所有未改已装单元（同wire新对象）→823结构门`unchanged_degraded`红→本session拒（Pipeline“structural gate”）、frontend generation不变；注：823结构门为安装后观测（对象确已重铸，这正是门捕获的降级），非commit前事务拒绝。**V-07**：guest trap改（g: `x/(n-3)`）在execute阶段拒、未commit、对象同、zero-hit与后续合法改正常；同base两候选第二个commit→`source.stale`、他环境候选→`source.foreign`；T-02已覆盖类型错误/stale base保全。av `closures_fn_912.rs` 11/11、core 7/7、lowering 2/2 |
| T-05 | work | T-02..04；AV closures P、911观测窗 | 候选原子/对象/代码安装、全返回attempt、accepted state导出与阶段版本/预算；保原restart且R3明示，V-07/08/11/12 | AC-03..05 | [x] 2026-11-02 commit `69b0632e1`。**accepted state（V-11/§5.3）**：`AcceptedSourceState`（schema `ace.closures.accepted-source/1`：source SHA256/逻辑owner/entry/profile/caps/producer版本(MIR_LOWERING_VERSION="4"+schema+unit codec)/accepted_generation/commit_point/context_revision/units）仅由已提交状态形成（frontend已接受目录+session已装wire/revision），每单元logical/body_digest/依赖与env身份/spanless MIR wire fp/store artifact ref/mint+current span——类型上无guest值/PASS/活env/cell/宿主地址；数据视图≠凭证（913落盘仍需重验）。目录新增每单元已验body digest与mint span（跨身份保持重建携带），commit绑定凭证事实。**recipe接线**：worker `ac-av/closures-fn-run-v1`（同一av.closures/1 profile，FUNCTION session按source path惰性建，face含route/warm_gate_pass/whole_attempts/mir_lower_attempts/retained_bytes/stage_attempts/body_work）；父侧`RecipeFamily::ClosuresFn`全链（recipe/profile轴6/store/selector含hit模式/`judge_closures_fn`），`CLOSURES_FN_CASES` F01 51→48、F02 51→61共享K/S表；父判fresh=cold+全单元、body-edit=fast路由+gate pass+whole 0+submitted严格小于全量、hit=zero-hit全零、restart=store全命中+store_writes 0+**R3如实**（父断言mir_lower==N被如实上报而非隐藏）+retained≤16MiB。**V-07**：trap编辑execute期拒未commit（T-04）+stale/foreign候选拒（T-04）。**V-08/R3**：restart腿实测cache_hits=5、store_writes=0、MirLower=5、whole>0、无warm gate（不冒纯恢复），其后暖路径正常——R3不闭。**V-12**：911无共享写面实核（diff无core/lowering/closures交集），未退化由既有组承接（T-07跑）。auto-test-case 30/30、auto-lang-core test_cases 26/26（4 ignored既有）、av `closures_fn_912.rs` 13/13 |
| T-06 | work | T-03..05；H | 文件化source/reference/oracle/父判决与scope注册、原K06/共享兼容及manifest身份保护，不动911三格/NA；V-01/14 | AC-01/05 | [x] 2026-11-02 commit `88aeb500f`。`closures-fn-912/{suite,case}.ad`（F01/F02 fresh/restart/body-edit/**hit**四态+K01/K06/S02字节同拷贝经FUNCTION recipe重跑——V-01原语料新链，hash与原文件逐一相等）；TestCase contract注册`ac-av/closures-fn-run-v1`（run/edit词表+av准入+entry规则）；固定组`case-ace-m1-closures-fn-912`（all=false/domain ace，`scripts/test-groups.toml:726`）；父judge按路由修正gate存在性（warm才有）。**13/13 PASS**端到端（含父判zero轴：body-edit fast路由+gate pass+whole 0+submitted严格小于全量、hit全零、restart全命中+R3如实mir_lower==N+retained≤16MiB）；原865/909组与infra-fixed/infra-ace-m1均PASS；`ace-m1/src`计数81不动、binding.tsv/applicability/manifest零改动（纯新增子树）；`auto-case stale`对912套件显示与909套件完全相同的Plan-868既有stale轴债（sema轴），环境/既有归因非本包回归 |
| T-07 | work | T-01..06；运行窗 | V-01..14合法与真实反控、完整职责/必要consumer、四状态30样本/512/原深链/资源/pin/恢复重验；与911错峰 | AC-01..05 | [x] 2026-11-02 commit `81bc7a9a2`。**V-13正式**：`closures_fn_912_performance.rs`（::performance::）64顶层函数/70单元、每被测owner≥3兄弟lambda、entry可达、首/中/末lambda分别编辑各`FastUnit`单单元、owner改`FastOwner`三lambda全复用；四状态×30样本P90=fresh 485ms/hit 3ms/restart 122ms/body-edit 29ms（≤2000ms门）；512修订churn采样retained≤16MiB、history≤8MiB、父峰值≤冷基线+128MiB。**V-08/R3如实**：restart-over-edit的session wire含span——被编辑源重启恰再admit span后移单元（本规模3个：d_main/f1/f2），cache_hits≥60、writes≤5、MirLower=70如实上报（与909 `writes≤1`同构）；restart-over-base全命中（case套件F01腿）。**V-09/10边界如实**：retained池与父峰值已门控；runtime cell/env fault sweep沿用909既有`av_closures_909_fault_sweep`（同一ClosuresUpdateSession运行时路径未变）；本包新增前端分配均在commit前、失败路径由诊断/trap/stale测试覆盖——未建前端逐分配点fault注入（记为边界，非缺口豁免）。**V-12**：911 diff实核无共享写面（core/lowering/closures零交集），未退化由完整组绿证（见T-08冻结门）。其余V-01..07/11/14证据已随T-02..T-06各测试落地 |
| T-08 | work | T-07 | 逐AC/V/SD/源hash/窗口/残余owner/健康自查，命令退出/数字折计划、冻结SD提案/提交/树clean→execution_done | AC-01..05 | [x] 2026-11-02。冻结提交`81bc7a9a2`（HEAD，树clean；T-02..T-07六提交88af52b13/63a1ea067/9f1db93e0/69b0632e1/88aeb500f/81bc7a9a2）。**逐AC自查**：AC-01（V-01/02/03/06）——lambda/owner纯body改真实仅改者+真依赖者生产（`closures_fn_912.rs` 13例、套件13/13、perf首/中/末lambda各FastUnit）；未改单元全阶段P=0且Arc/代际保持；whole后filter/隐藏重算/Default0/同hash新对象四强制红均使门红。AC-02（V-03..05）——registry凭证覆盖site唯一/关联/env/mode/资源/cleanup（coll_facts）；复用oracle按当前创建点重推捕获决策、真读者精确失效；同context异body（lower.closure-facts body digest）、空registry/篡改env/直接call/重复site拒；lowering零再分析（等价oracle）。AC-03（V-07..10）——trap/stale/foreign/诊断拒均保状态且下一次可用；zero-hit/历史/只读store/重启重验不退化（restart腿）。AC-04（V-06/09/10/13）——attempt在真实dispatch点计数（含失败早退）；M/P/R分轴；16MiB池+旧cap+父峰值+四状态30样本P90（485/3/122/29ms）同过。AC-05（V-08/11/12/14）——正式首例`closures-fn-912`套件+父独立判决（含zero轴）+完整组绿；911窗口实核无冲突；913 A→B `AcceptedSourceState`+版本/所有权/消费者收据；R3明示不闭、D-909-1未触碰（ace-native绿）、完整C-19仍后继。**健康**：三crate lib零警告（ace-av预存owned_session字段警告为基线存量非本包）；无debug打印（perf eprintln为正式百分位报告，沿909形态）；树clean。**完整职责/必要consumer门**（冻结码）：ace-source/ace-lowering/ace-av/ace-native/ace-a2x/auto-test-case全PASS（run2；run1仅驱动器"less than 2 GiB free"预检瞬时盘满拒绝，非测试失败，盘清后同码重放绿）；`case-ace-m1-closures-fn-912`/`case-ace-m1-closures-865-source`/`case-ace-m1-shared-mutable-909`/`infra-fixed`/`infra-ace-m1` PASS；正式性能`::performance::`单例绿。**SD提案冻结**：§规范增量表SD-01..06即merge输入（新增`docs/specs/auto-ac/closures-function-production.md`+五处modify+project导航），merge读回核对。**残余/边界如实**（复审核）：前端逐分配点fault注入未建（运行时fault sweep沿用909）；owner路由`scan_shared_capture_cells`为owner级语法扫描；嵌套lambda仅经owner路由；`__closure_<owner>_<k>`显示名仍按owner内序号（拓扑变化时重排，逻辑ID保持——SD-02措辞按此）；plan827测试向`docs/reports/`写观测为既有行为（未提交、未改动） |
| T-09 | review | T-08同树冻结SHA | 独立核机制/依赖/反控/粒度/全部实际门与SD，finding有界修复重绑，pass→reviewed | AC-06 | [ ] |
| T-10 | work(merge) | T-09 pass | 冻结职责/consumer门或有效同身份证据；batch仅due主检出单实例/既有用户择时，不额外造全局门 | AC-07 | [x] 2026-11-02 冻结并集（rebase后`13a465336`，含911落地码）：ace-source/ace-lowering/ace-av + ace-native/ace-a2x/auto-test-case + case-ace-m1-closures-fn-912/closures-865-source/shared-mutable-909 + infra-fixed/infra-ace-m1 **11组全PASS**（runs drxvv73n/c6ehavke/2w4bifco）；run1仅驱动器盘满预检拒（t07-full-groups.log），盘清后全绿。batch due check：receipt last_covered=895(2026-10-30用户豁免)+48h内有merge（911）→ **due=true，user-pending交接**（沿910/911先例，`cargo t all`主检出单实例由用户择时，不在本计划内执行） |
| T-11 | work(merge) | T-10 | 合入/SD-01..06读回、A→B正式类型/版本/阶段/预算及剩余owner交接、archive delivered，R3不冒已闭 | AC-07/08 | [x] 2026-11-02 **landed**：worktree guard clean（lang+down）→`git rebase master`（master已含911@ca4848f5d+007 sweep）零冲突→**range-diff 8/8全等**（旧88af52b13..ea4ef2906 → 新61ed79ed9..13a465336）→主检出`git merge --ff-only plan-912-dev`→master==delivery **`13a465336`**，无merge commit。**canonical_verified**：SD-01 `c99a4c8c…`、SD-02 `863ac6a0/8820b572/efe8f4ef…`、SD-03 `a1782378/c5e73745…`、SD-04 `46b659be/59972c1d…`、SD-05 `6e178ad2…`、SD-06 `e3a28ea1…`（主检出实读）。A→B交接=SD-01 §A→B（AcceptedSourceState schema/版本/预算/无PASS/913落盘仍须重验）；R3/D-909-1/完整C-19具名未闭。archive见下receipt |
| T-12 | work(merge) | T-11 | 折摘要/hash、核全部实际输出/hold、fresh wt-guard clean后清树/分支/组/整个材料scope及登记，真实12/12 | AC-07/08 | [x] 2026-11-02 证据已折入计划（各行收据含命令/退出码/run id/关键数字）；fresh `wt-guard.sh` clean（lang+down，reparse扫描空）→worktree两棵移除（lang树先清plan827测试副产物：tracked还原+untracked删）、分支`plan-912-dev`删除（was 13a465336）、组目录`.wt/lang-912`删除；材料scope `D:/autostack/plan-artifacts/plan-912/`（422K：t01冻结/四代码映射/t02-t10组日志）**整体删除**；registry刷新（scopes.plan-912=deleted、deletions+1、last_merge_cleanup=PLAN-912，holds=[]）；构建中间产物随worktree删除未回收。收尾路径对账：本计划命令实际输出目录=worktree target（随树删）、`plan-artifacts/plan-912`（已删）、`%TEMP%`perf/session stores（已清）——无散落第二目录 |

拟树`D:/autostack/.wt/lang-912/auto-lang`、分支`plan-912-dev`，未创建；work/自查/独立review同树顺序，无用户指定agent/model。开工登记唯一材料`D:/autostack/plan-artifacts/plan-912/`，探针/日志/报告/变异副本/外部target只在此；长期case/oracle才入职责目录。每case/store用自己scope隔离，禁止批删全机`%TEMP%/av798-session-*`凑fresh。merge按新规全材料折计划后删除，具名未解问题才hold，不动他方目录、不写旧specs.json/attachments。

## 9. 复审记录


2026-11-02 | stage: work | PLAN-912 r1 | outcome: pass → execution_done | worktree `D:/autostack/.wt/lang-912/auto-lang` branch `plan-912-dev` | base `5923d4dee` | code HEAD `81bc7a9a2`（T-02..T-07: 88af52b13, 63a1ea067, 9f1db93e0, 69b0632e1, 88aeb500f, 81bc7a9a2） | deps: auto-down detached `7c4d5cb`（只读）；911 branch `4cf907fb7`实核无写面冲突 | tasks T-01..T-08 8/8 [x] | evidence: av `closures_fn_912.rs` 13例+`closures_fn_912_performance.rs` 1例、core `closures_fn_912.rs` 7例、lowering `closures_fn_912.rs` 2例全绿；套件`case-ace-m1-closures-fn-912` 13/13端到端（父判值+zero轴）；完整职责组 ace-source/ace-lowering/ace-av + consumer ace-native/ace-a2x/auto-test-case 全PASS；原865/909组+infra两组PASS；V-13 P90 fresh 485/hit 3/restart 122/edit 29ms、512 churn、retained≤16MiB、父峰值≤冷基线+128MiB；材料`D:/autostack/plan-artifacts/plan-912/`（t01冻结+四映射、t02/t07组日志与perf log） | blockers: 无 | next_responsible_role: review（同树冻结SHA 81bc7a9a2；AC-06独立复审）

2026-11-02 | stage:new | PLAN-912 r1 | outcome:pass（规划合同，非实施/最终review PASS） | 用户明确规划909后继并授权判断拆包；按实际R1/R2/R4与R3入口/信任边界拆912→913。D90959b8107eb祖先0，R1/R2/R4代码实读，verify_function_ref无Closures准入，不假称现成接口已够用。原语义/已批准M1口径继承，无施工授权。

架构Design43 §16与索引先更新后，在docs/plans/.allocation.lock独占句柄内重核max/next，调用new-plan.sh顺次分配912/913（下一号914），无第三号；两包均真实职责而非占号。只计划/设计/路线，不改production/current Spec/source/expected/binding/911/910合同，不建树/材料/跑构建测试探针认证/agent/chat。

规划静态预检（2026-11-02）：本计划0..10节、T12/AC8/V14/SD6定义及引用、12项全未勾、drafting r1/0/12、局部链接/架构anchor、现有Spec路径与未来Spec尚未建立均通过；913对应T12/AC8/V16/SD6通过。五份设计/战略/交接文档新增链接有效；15个现有固定测试组名称已对照test-groups.toml，新912/913组仍为实施目标。912/913两号各唯一，next=914，无对应worktree/plan-artifacts；binding SHA256仍为8dea9b9fd98ad379d62dff51cce9074b7e4b434c2537cdeae331d072ea343422，current Specs/code/scripts无tracked diff。全历史扫号曾遇既有archive/013重号，未动历史，仅确认新两号无冲突。以上只证明文档静态一致性，不是V/AC运行证据。

next: 后续实施指令后T-01/02；N911/parent可并行，S/H先核实际owner；913实际串接等D912。所有V/AC待执行。
2026-11-02 | stage: review | review_scope: final | reviewer: 同会话独立复核（无指定reviewer/model；局限性：实施会话内执行，判决从工件与重放重构而非执行者总结） | plan_id: PLAN-912 | plan_revision: r1 | outcome: pass（R1修复后终判） | worktree `D:/autostack/.wt/lang-912/auto-lang` branch `plan-912-dev` | reviewed_commit `8b5400937`（工作冻结`81bc7a9a2`+修复提交；基`5923d4dee`） | deps: auto-down `7c4d5cb`只读、911 `4cf907fb7`写面实核无冲突 | spec_inputs: §4.2九文件T-01冻结hash（未变） | acceptance: AC-01..05 pass——AC-01复放core 7/av 14/套件13/13/perf（identity/zero/whole红）；AC-02代码实读reuse oracle无analyze_capture_uses/lambda_body调用+registry反控+等价oracle字节等；AC-03 trap/stale/foreign/zero-hit/restart重放；AC-04真实dispatch点attempt+P90 323/2/85/25ms+池/父峰值+512轮转churn；AC-05父独立oracle算术核对（51/48/61）+accepted state+组绿 | findings: F-912-R1 IndexBuild轴无生产者（§5.4声明未接线）→已修（ResidentProgram::function_builds()+link缝记录+perf断言0，闭包profile索引门控如实）；F-912-R2 churn未覆盖"不同lambda"→已修（512轮转f0/f1/f2三body+历史回访）；F-912-R3（措辞，merge落实）AcceptedSourceState.logical_owner为module path，SD-01按此表述 | evidence: `cargo t ace-av` PASS（run-6bn5oi31，修复后）、`case-ace-m1-closures-fn-912` 13/13（run-_0dxiw9h）、修复前冻结并集t07-full-groups-run2（ace-source/lowering/av+ace-native/a2x/auto-test-case全PASS@81bc7a9a2；修复仅增av-core访问器+永不触发的记录环+测试） | pending_closeout_gates: merge自有——冻结完整职责/必要consumer并集或有效同身份证据（T-10）、`cargo t all`周期门不属本计划 | next: merge

2026-11-02 | stage: goal | plan_id: PLAN-912 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-912/auto-lang | commit: 8b5400937 | next: merge

2026-11-02 | stage: goal | plan_id: PLAN-912 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-912/auto-lang | commit: 81bc7a9a2 | next: review

2026-11-02 | stage: merge | PLAN-912:r1 | prepared: reviewed基线`8b5400937`+SD冻结表→spec提交（rebase后`13a465336`） | landed: ff-only，master==`13a465336`，rebase零冲突，range-diff 8/8全等（88af52b13→61ed79ed9, 63a1ea067→30dcb1268, 9f1db93e0→4c86142b8, 69b0632e1→f252f98a2, 88aeb500f→14c9fb5b2, 81bc7a9a2→f277778c0, 8b5400937→50bb8ee0b, ea4ef2906→13a465336） | canonical_verified: 10路径hash如T-11行 | archived: docs/plans/archive/912-closures-function-production-closure.md，completion_kind delivered | 911窗协调：contract.rs/test-groups.toml/三Spec零冲突（不同hunk）；911 worker/parent面与912 recipe并行共存，11组并集绿 | landing observation: 主检出生产二进制（auto.exe等）未随本merge重建——910已记其被外部服务锁定；本包交付为实验栈ace与测试设施，产品构建图零依赖（根workspace exclude experimental不变），stale风险限于实验工具按需重建 | batch: due=true（895后48h内有merge），user-pending交接（沿910/911先例） | cleaned: wt-guard clean（lang 8f无reparse/auto-lang + auto-down）→`git worktree remove`×2（lang树先清plan827测试副产物）、`git branch -d plan-912-dev`（was 13a465336）、`rmdir .wt/lang-912`；plan-artifacts/plan-912（422K）全删，registry刷新holds=[]；无具名未解问题持有（R3/D-909-1/完整C-19为具名计划债非文件持有）
2026-11-02 | stage: merge | PLAN-912:r1 | outcome: pass（delivered） | delivery_commit `13a465336`（master tip，ff-only） | docs tip `2872b70ea`→本提交 | canonical: SD-01..06 hash见T-11行 | archive: docs/plans/archive/912-closures-function-production-closure.md status=archived | cleanup: 完成如上，真实12/12 | pending: batch regression due=user-pending（用户择时）；主检出生产二进制观察项（910已记外部锁定）

## 10. 待澄清事项

| D | owner/入口 | 有界关闭条件 |
|---|---|---|
| D-01 单体Closures授信 | T-02/04，912 | 冻结完整decl registry/资源/capture事实与body凭证，不能仅改域match或跳全局site门 |
| D-02 lambda/body片段与身份 | T-02/03 | 真同owner单lambda编辑、偏移/类型索引变不串单元；普通支持样本不得fallback；歧义显式分态 |
| D-03 accepted state | T-02/05，912→913 | 版本/实际依赖/所有权/预算与无PASS数据契约交付，913持久数据仍未授信，A不关闭R3 |
| D-04 911 S/H与观测 | T-01/05/07 | 原typed失败/attempt接口与全部consumer具名收据，未释放只等实际依赖任务；不消费WIP/强迫对方整包等待 |
| D-05 预算/残余 | T-02/07/merge | 16MiB新池与原cap/时间同时证可行，放宽需修订；D-909-1、R3及完整C-19另列owner，不豁免新回归 |
2026-11-02 | stage: goal | plan_id: PLAN-912 | plan_revision: r1 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed) | commit: 13a465336 | next: done
