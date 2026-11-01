---
plan_id: PLAN-909
status: archived
feature_name: 共享可变捕获C-19与K06源码闭环
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-02
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-ac/source-closures.md, docs/specs/auto-ac/function-incremental-source.md, docs/specs/auto-mir/closure-capture-profile.md, docs/specs/auto-av/closure-environments.md, docs/specs/auto-av/cache-and-conformance.md, docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-gap-audit.md]
new_spec_components: [docs/specs/auto-ac/shared-mutable-captures.md]
touched_goals: [GOAL-002]
affects: [auto-ac, auto-mir, auto-av, auto-test-case]
depends_on: [PLAN-865, PLAN-823, PLAN-868]
current_step: 12
total_steps: 12
---

# [PLAN-909] 共享可变捕获C-19与K06源码闭环

## 0. 变更摘要

承865源码闭包、823 AV环境与唯一共享lowering，为显式closures源域交付两条真实链：①原K06 move闭包的私有环境可写且跨调用保留；②非move标量var的真实共享写捕获，外部局部/兄弟闭包同cell、合法逃逸保活、clone独立及全出口回收。补真实函数增量，不用“全量生产后只提交变化wire”冒未改生产0。

**一个Plan，drafting r1、0/12。** 共享捕获分析、受检事实、lowering、AV和源码消费者必须一起验，拆无消费者cell平台会重复接口/资源/增量验收。T-02先最小全链定受检表示/身份，不预定新opcode或另占计划号。架构见[Design43 §14](../design/43-ac-av-typed-mir-and-abc.md#14-c-19k06共享可变捕获源码切片2026-11-01target-state)。本轮仅规划，不建实施树/材料、不编译/测试/探针、不改生产/current Spec/原源/expected/binding/908合同。

本包支持新增共享写cell的值域为函数局部`var int/bool/char`；原owned move/clone/Str借用域保持。共享拥有型值、循环图、跨挂起借用等仍属C-19后继，不能随K06通过把C-19完整大类全部关绿。新语义范围/拒绝面是本合同拟实施内容，不冒当前已支持。

## 1. 目标

- G1：未修改的`corpus:K06`原源从编译拒绝变为真实AV结果42；另有证明move不写回外部的正控，不能靠把捕获读改成常量或每调用重建环境通过。
- G2：用稳定词法BindingId生成共享事实，支持同绑定的非move共享读写、创建顺序/遮蔽与外部写回；mode由前端/验证出口判定，lowering不重做分析。
- G3：明确拥有cell逃逸、原Str借用拒逃逸、move转移及clone深复制/内部共享拓扑；不借闭包clone暗改原共享效果。
- G4：构造/clone/执行失败原子、清理与资源预算成立，代码/环境/活cell生命周期分离，坏工件/旧身份不能进入执行。
- G5：fresh/hit/restart/body-edit/恢复和捕获布局/效果变化依赖齐；未改函数真实生产/工件IO=0、对象/代际恒等，不只看submitted计数。
- G6：首例与区分性反例文件化AutoDown/TestCase+独立oracle，沿原固定组/runner；与908按S/P/H交接，不改其适用规则/分母或把本包提前记交付。

非目标：共享写cell内Str/record/List/Variant/closure/Dyn、mut/view参数提升、循环/递归环境、跨挂起借用、完整NLL/GC、async/线程、持久活闭包、native closure ABI、A2R闭包发射、map/filter/reduce库/方法HOF、默认/变参/函数项值、旧引擎替换。保留865已支持的owned捕获回归，不以本包有限共享域反向删旧支持。AV通用资源故障/全部句柄代际专项不在此包，只为本包cell/env路径补足必要机制和验证。

## 2. 架构方案

`原Auto源 → BindingId/捕获读写与共享分组/逃逸分析 → Checked HIR与descriptor/受检捕获事实 → ace-lowering唯一实现 → typed MIR验证 → ABC/准入 → 既有ClosuresUpdateSession/AV → 原TestCase父侧oracle`。

cell是语义责任，不先假定其物理表示。T-02核既有共享List句柄/owned env是否可满足私有可写与共享模式、clone拓扑、生命周期及所有准入门；如需有界专用cell描述/指令，先写清schema/profile/语义身份与全部消费者，再实施。普通List复制/无校验env别名不能充cell合同；新增类型/字段缺少MIR/ABC/恢复验证则不得先上线。代码cache仅存可重验代码/布局/效果，活cell不入cache、不恢复运行结果。

## 3. 技术栈

现有Rust ACE workspace（ace-core、ace-lowering、ace-av-core、ace-av）、受检HIR/Atom描述、函数凭证/查询与managed store、AutoDown/TestCase、固定cargo t组。复用既有资源/取消/故障及进程控制设施，无新workspace/runner/动态目录/全仓新鲜度图/三方库。架构设计增加于已登记Design43，不另建无消费者的通用平台设计号。

## 4. 需求分析与背景调查

### 4.1 授权与交付身份

用户要求正式立项C-19／K06源码闭环，核865/现有AV基线，明确捕获共享、写回、逃逸、clone、清理/函数增量及908边界；**本轮只规划不施工**。允许主检出计划/设计簿记；不授权构建/探针/源或TestCase更改、联系其他会话、清理他方树或自动开工。既有M1 G-DEC-1..5/NA/R/双轴批准直接继承，不重复索取，也不扩大范围。

读取主线`ed4286792e181de580a1241a878b84d1d2f86d1a`。865 archived r2、9/9；其历史delivery `5fa3133fa6401904b0760986ecb2e83b0534cbd4`对象存在，但本次`merge-base --is-ancestor`为1，不能声称是当前HEAD祖先。现行源码Spec沉淀提交`fe509bfff`为HEAD祖先(exit0)，当前实现及迁移路径实存；依据现行Spec/代码冻结本包，不重开865或伪造旧SHA新证据。823 archived r10、11/11，历史落地2f90821dc；现行wire/资源以closure-environments及av-core实现复核，不因旧归档的ledger/测试别名重新写旧JSON或恢复材料。

868交付唯一lowering的职责保留；896整合后当前实际路径为`experimental/ace/crates/lowering/src/{hir_adapter,lib}.rs`（最近路径提交1795038c9），旧`experimental/auto-mir-lower`与probe路径不能作为当前写面。904已交付的新A2X资源/片段合同可参照，不能冒闭包源码已具备同等暖路径。

908用户确认实施中且`lang-908`有真实WIP，计划头仍drafting0/12，不能据滞后头判未开工或代改其进度。读取WIP：binding.tsv、scope决策件、852脚本、test-groups，新增NB17..19源/诊断suite、40场景Spec候选及治理测试；本包只核占用，不消费其未发布规则/候选Spec。906已reviewed、在合入收尾，当前树71f922720；905已归档7/7。T-01重核主线与实际当前owner，不将任一整包设无关语义前置。

### 4.2 来源与观察

| 来源/当前路径 | SHA256或实际事实 |
|---|---|
| docs/specs/auto-ac/source-closures.md | `0350192e7fdcfd2ab047cedc04ce738d13b3a0b7e344f0b4f5be840cacd85bc5`；Copy快照/owned move/Str非逃逸借用，写捕获统一拒绝 |
| docs/specs/auto-av/closure-environments.md | `6e15e363d178909ca8d90e88a697c41361b83a33b80b9893d0f09c7860c4b5b2`；id6/AVADv5/tag17、CREATE/CALL/ENV_GET、env拥有/clone/清理与恢复 |
| docs/specs/auto-mir/closure-capture-profile.md | `6c88ae708a96c0c7a8087310affced172a68569de6710b40aac760deb2a2e51e`；MIR Move/Clone/Borrow，env声明/链接精确相等及旧代句柄防护 |
| crates/auto-lang/test/cases/ace-m1/src/K06.at | `aa426014e0d01ea43505e0a2cc23cc672105481d59cba301030b7dc5282382c3`；move base=1、f(20)+f(0)=42，原行为expected不变 |
| 同目录reference/K06.rs | 手写Rust move closure参考，assert 42；不是a2r golden/cert，仍可作为独立参考来源 |
| core/src/source/closure_capture.rs、resolve.rs::resolve_closure | 首次自由变量顺序、written位；written当前直接报source.closure-shared-mutable；需从按name线索升级词法绑定/写效果事实 |
| core/src/hir.rs、descriptor.rs、verify.rs；core/src/source/{typecheck,adapter}.rs | HIR CaptureMode仅Move/Borrow；需捕获mode/共享身份/效果与受检数据出口，未知/矛盾事实不得授信 |
| lowering/src/hir_adapter.rs | lower_function_closures_unit及CreateClosure消费mode，Copy先复制临时再Move入env；普通ReadLocal/Assign不能自动成为共享cell访问 |
| av-core/src/{mir,mir_verify,ownership,instr_spec,abc,abc_verify,av}.rs | ENV_GET读字段是Value浅共享；既有List/env机制不等源码写回合同；clone_owned_value为显式工作栈，新增cell须核别名/回滚 |
| av/src/closures_865_support.rs | `0a11feb8a5e1e0d34cc35e16f6cf43644f854a996941777bfbb84bbd3f6f4e82`；apply_source整源compile→逐函数lower/encode→比wire→提交；未改前段工作尚不为0 |
| av/src/{closures_session,function_queries,session}.rs | 已有单元安装、FnEnv/body事实、对象/代际与资源门可复用；不能仅贴相同计数器证明新source路径复用 |
| av/tests/cases/av_closures_source.rs；lowering/tests/cases/closures_lowering.rs | 当前K06拒绝、K01..05/间接调用/捕获变化与submitted单元等回归；旧计数不证明全前端/lower零工作 |
| closures-source-865.case.ad / suite.ad、scripts/verify-av-865.ps1 | 正式source recipe ac-av/closures-source-run-v1；K06旧check期望resolve拒绝，须经能力扩面实证迁移；脚本注释20样本不等当前路径已做正式计量 |

当前R5-O/865-N1是未关闭共享能力的owner，不是已运行PASS。本包接其中标量共享写/私有环境可写的源码切片；循环图/跨挂起借用仍具名后继，typed mutable env证据不得代替K06源码42。

## 5. 详细设计

### 5.1 SEM-04扩面与类型边界

| 模式/绑定 | 拟实施规则 | 区分性结果与拒绝面 |
|---|---|---|
| move捕获Copy int/bool/char | 创建时复制当前值，闭包私有环境可写、跨调用保存；外部源仍活、不写回 | K06=42；加外部base后为43，若误共享原base则63 |
| 非move、var Copy，存在非move写捕获 | 按同一词法BindingId形成cell；外部读写与该绑定所有非move读/写捕获同cell；各次函数激活独立cell | 兄弟writer f(20)+g(0)=42，误复制为22；只读reader先创建也读当前共享值 |
| 非move、无共享写捕获的Copy | 沿865按值快照；不因var或后来外部赋值自动改变 | 原只读Copy正例保持；新增writer会改变本拥有者的捕获事实，须使真实读者精确失效 |
| 混合move与非move | move在创建点取快照/转移，与原cell分离；非move仍共享原binding | 先创建move快照，再共享写原绑定，move仍读创建值；不得全按引用或全按值 |
| let / lambda参数 / 同名遮蔽 | 捕获let写入仍为静态不可变赋值拒绝；参数/内层同名局部不误当外层写；用BindingId判 | 合法var邻例可写；遮蔽/不同激活不串cell，首次自由出现字段顺序保留 |
| 原owned move/clone、同步Str Borrow | 865/823原支持域保持；owned整体move杀源、clone显式、Str借用钉源且不逃逸 | K01..05与资源捕获旧正反控；K04仍source.closure-borrow-escape |
| 新共享写cell的非Copy或参数view/mut | 首期不接：StrOwned/record/List/Variant/closure/Dyn及mut/view参数提升具名unsupported | 不能以隐式clone、浅共享owned、栈指针或全局变量模拟支持；原owned只读/move不误拒 |

共享分组要在词法拥有者的受检捕获摘要可用后确定，不能按遇到lambda的次序猜。摘要至少包含稳定binding身份、value类型、move/private/shared/borrow模式、读写/逃逸效果、首次使用顺序、创建点/环境与清理责任。捕获写判定覆盖本切片的整体Assign和嵌套词法引用；新增共享模式不支持的字段/方法可变效果须拒，不能漏识为只读，既有owned move域允许的方法仍保持。嵌套lambda的稳定身份/自由变量传递由T-02核可支持既有语法；若增加间接可变效果语法，先修订范围。

共享分组是静态语义：若某var在此拥有者中属于共享组，在该var声明初始化时创建一次cell（先按源码顺序求初始化值一次，再记账/分配/提交cell），即使写lambda在条件分支内尚未执行。这样先创建的reader也不被错误快照化；未实际创建的闭包不持owner。外部赋值和读取从此访问同cell；move创建点从cell读取当前标量快照。cell创建失败是资源/infra分态，不能吞掉初始化副作用或重复求值。捕获let不能写；闭包值自身的let/var不改变被捕获绑定的mutability，不借本包新造Fn/FnMut语言体系。

### 5.2 写回、逃逸与clone

1. 拥有者局部和每个共享闭包环境持有可追踪cell owner；外部赋值与闭包调用都更新同一受检位置，求值/写回顺序沿原源码语义。允许顺序重入调用，共享值不留宿主RefCell借用跨guest调用，别名不能触发Rust panic；异步/并行不在域内。
2. shared标量cell为提升后的拥有对象，可由返回/原已支持闭包参数传递保活，创建者返回不悬空；各次factory激活独立。合法move捕获逃逸保持；Str Borrow、mut/view等真实借用仍在编译/verify门拒逃逸，不能把提升机制当通用borrow豁免。闭包字段/List元素/持久化仍拒。
3. closure move转移环境owner，源死；显式clone深复制当前可达环境与cell，原/新互不写回。一个clone根内同一cell只复制一次，保其内部共享关系；不会回连原图。受检重复绑定不合法须拒；合法内部别名的typed反控必须分清，不能无论来源一律复制或一律共享。clone失败不改变原值/别名图；不引入无限循环图。
4. 私有move可写环境是独立可变状态，不得每次CALL重新初始化。源码f.clone()沿原显式clone语义，不能改为共享原环境；K05独立性反例仍11而非12。

独立例：factory返回非move计数闭包，创建者已退出后两次调用仍21+21=42。clone例从x=1创建writer f，令g=f.clone()，依次a=f(20)、b=g(1)、c=f(0)，结果a+b+c=44；若clone共享原cell会得到65。两个同时活factory闭包分别变化，不能借全局单例状态过门。新源码语法/固定ID及手写参考在T-02核合法后冻结，不改原K06。

### 5.3 生命周期、失败原子与资源

捕获构造/clone先prepare资源/深复制/计量再commit；失败前原owned源与已有cell owners不被消费，不发布半env，临时cell/env/owned全部清理。commit后按转移责任清理，不能复活已move值。赋值在预算/合法性通过后提交；失败保原scalar值和owner集合。取消只承诺既有poll点生效，不能写立刻取消的假合同。

正常return、trap、budget/wall/cancel、注入分配失败、候选拒绝、重复close、最后闭包drop均测真实cell/env/owned live字节/分配释放。session.close释放当前/历史代码roots；已授权外部活帧/closure pin可保活旧代码/cell，不能强制清空或报告0，最后owner释放后才归零。store/cache只承载代码/描述，不持有live env；重启从初始源状态真实执行，不恢复上次cell值。

预算沿现行更严门，不擅增：源/元数据输入≤4MiB、活值含cell/env/owned合计≤8MiB、验证分析≤16MiB、decoded module≤16MiB；code current+history共享8MiB、每unit历史≤32条。新增cell owner索引/clone memo/元数据和计量辅助也记账，不能藏在预算外map。父进程private/working-set峰值及原ExecLimits/Job/流/deadline由T-01记录真实配置、T-02冻结新增分区归属，不提高旧cap；超界分态为Resource/Infra而非语言诊断或PASS。无任意语言嵌套深度新常数；沿既有显式栈/有界资源遍历，3072层/256KiB栈原机制证人不缩水，不把其误当本包共享图允许循环。

### 5.4 受检表示与身份

T-02最小链必须实际穿越source→HIR/descriptor验证→共享lowering→MIR/ABC双门→AV及存储恢复，包含K06、真共享、逃逸、clone独立、一次失败回滚和一次暖编辑。选择物理表示的记录含对象owner/状态/操作/字段、每条生产/验证/消费边、cap归属、旧载荷命中/拒绝策略；不可只做typed fixture或解释器旁路。

共享事实由core生成并验证，descriptor roundtrip不丢模式/别名/写效果；lowering仅消费事实。MIR/ABC校验env字段类型/顺序/创建点/mode/owner、读写权限/生命周期；运行时在危险访问前复核句柄/代际/状态。缺/伪模式、错绑定/类型、未初始化/已move、旧环境/闭包错签名必须拒且危险操作0，合法非零身份正控保留。

优先复用既有av.closures/1资源/环境；新增源语义/捕获身份进入语义key，不能让旧“写捕获拒绝”的缓存结果或旧无共享事实的载荷命中。若需要schema/profile/指令扩面，T-02先冻结与现有id6/v5/tag17和dynamic id5/tag16的隔离及合法旧823对象兼容；不自行抢号或让旧解码器猜新含义。不能同时承诺“wire不变”和引入未编码的语义字段。A2X/native等未支持consumer须受检拒新mode，不默默当Move；本包不据共享HIR改动宣称Rust/native已支持闭包。

### 5.5 真函数增量与恢复

现行apply_source的whole compile→全函数lower/encode→比较wire是已观察缺口，本包必须在closures真实入口补齐，不能只增加漂亮计数。复用core片段/CheckedEnv、AV FnEnv/body facts和单元session；冷启动/声明不收敛可显式全路径，**支持的body-only暖编辑**不得静默整源fallback。

- synthetic lambda ID绑定其词法拥有者/稳定创建身份，不用全模块递增序号让无关函数随插入/重排全失效。单lambda纯body编辑且捕获mode/布局不变，只生产该单元；创建点父函数、其他lambda与free函数的HIR/MIR/FunctionCode对象及代际保留。
- 捕获集合/类型/顺序、read→write、private→shared、逃逸/效果变化是body事实：即使签名/调用点字节不变，创建者与实际读者也须精确失效。依赖是完整读取事实/类型/实例/CT结果和配置身份，不只哈希调用点；对未读的事实变化不能无谓失效。
- 暖路径实际观测parse/project/resolve/typecheck/HIR-build+verify/lower/MIR-verify/ABC encode/decode/admit与artifact读写的调度次数及真实对象身份；未改生产=0。计数窗口包含比较/摘要准备，不能把全量前段藏到窗口前；有效驻留对象不得为保hash换新Arc。
- fresh/hit/restart/body-edit/改回历史、含type/enum文档的闭包函数、同context异body凭证、捕获mode/类型变化、store损坏/陈旧身份、只读store及候选失败后的下一次真实restore/read都验。hit不生产；restart可以read/decode/重验，但不重新parse/build/lower/encode已有有效产物；readonly工件writes0。
- 错误source/HIR/MIR/链接候选不覆盖当前snapshot、事实或历史；下一次有效修改/恢复可运行。代码旧pin保活既有代；不把重启/重跑当在线HMR，也不本包扩造866在线调度。

### 5.6 文件化验收与K06迁移

复用`ac-av/closures-source-run-v1`与现有worker/adapter为默认方案；若语义身份需新recipe，先在T-02明确注册/三缝镜像/版本、旧入口兼容及消费者，不另造runner。拟新增长期`crates/auto-lang/test/cases/ace-m1/shared-mutable-909.case.ad`、`.suite.ad`、`shared-mutable-909/src/`及手写reference/oracle，T-06使可运行；原K06通过引用原文件进入首例。它是corpus，不能混到908的40个scenario中。

首期原K01..06及反例稳定ID，source hash/行为oracle独立；actual与手写参考/手算各自对oracle，不让两个共用bug的后端互相比较充独立证明。资源/失败/身份内部计数由真实机制支持并由父侧判决，wrapper exit0、0选择/SKIP/只unsupported都不能绿。AV路径不要求或生成a2r证书，日常不调用rustc认证/改golden；手写参考可显式独立验证但与旧认证轴分账。

K06原source、binding id/ns/outcome/expected=42、reference/golden均保持；旧865源码check中的“resolve/source.closure-shared-mutable”是已支持边界断言，能力扩面实证后应迁移为success/42，并新增真正不支持模式的负控，不能留新实现通过却旧套件永红。此为正式新能力导致的受控预期迁移，不是修改行为expected凑绿；只改AV检查及证据注记，不动legacy/a2r差异、NA规则或认证。

### 5.7 正式计量与验证选择

语义/资源/结构门必须与时间同时成立。拟对fresh/hit/restart/body-edit各30个原规模样本列P50/P90/P95/P99，正式开发期P90≤2000ms，长期亚秒目标保留；准备/冷编译/发现/执行/清理与整命令分列，不混为一次编辑时间。body-edit规模覆盖多free函数/多lambda及含type文档、512修订churn+GC；记录管理代码/值/历史、父峰值、内存预算及最后owner回收。保留823/865原样本与原机制反控，不以新30样本代替必要旧门，也不把旧脚本注释当实际计量证据。

日常用例与正式资源/性能分入口，沿`cargo t ace-performance`/现有ignore与原子资源组，不把新压力默塞普通legacy组或跑13慢测整包。V-15由T-02/07启用的命名case接入既有正式入口；未知规模/父峰值以原更严cap为基线冻结，不缩样本/放宽门/强杀60秒。

### 5.8 与908、当前公共写面的交接

| 面/owner | 909可写 | 不可越界/窗口与回交 |
|---|---|---|
| S909 | core/source capture/resolve/typecheck/adapter、HIR/descriptor/verify；唯一lowering及必要MIR/ABC/ownership合同 | 908本包不修生产语言能力；909不改其诊断归一器/规则，新增事实字段列所有真实consumer并跑必要完整组 |
| P909 | av-core闭包执行/clone/资源、AV closures_source_session/cache、现有测试目标里的私有语义/增量反例 | 继承823安全门与866既有生命周期，不另做native/A2R闭包、全局GC或整波资源平台 |
| P908 | 适用sidecar/source-manifest、852入口、test_ace_m1_acceptance、40场景oracle、scope决策件 | 默认只读，不改G-DEC/NA/required/总数/独立认证；未跟踪候选不是已发布接口，不提交其WIP |
| H：K06状态/旧865源码case | 909须更新AV源码检查、与能力证据对应的K06 AV token/注记；908管理总表与准入 | 具体`corpus:K06 × av`及旧check id短窗单写，先冻结owner/revision/源hash/旧token/规则和当前治理版本；不并写整表，不提前把主线能力改绿 |
| H：worker/adapter/测试注册 | ac_av闭包source分支、test_cases/mod.rs/contract/recipe镜像及test-groups.toml仅必要符号 | 908正改固定组，906尚在收尾；先核主线和当前owner，登记文件/段/函数/正式SHA/recipe与schema，再串行短窗，不能以append-only冒无冲突 |
| H：Spec/路线 | 909 merge沉淀共享捕获/源闭包/MIR/AV与K06实证；908沉淀规则/场景与总验收 | source-manifest或证据身份因新能力漂移时只标注受影响/重核，不回写903冻结数，不恢复旧raw；capture事实属于909，不让908另造资源分析 |
| 正式运行 | 909自身相关固定组、源case与正式性能 | 与908真实窄诊断/验证及906四仓构建错峰，不停他方进程、不持H等机器窗；不把整个908/906完成设前置 |

接口交付序：若908先合入，909在正式治理基面上迁移K06 AV证据并读回其规则/报告身份；若909先合入，908接受其delivery/schema/语义版本/source哈希/真实报告，重核K06受影响输入，余格不借机改绿。K06旧拒绝→成功的case迁移必须在909验收前取到合法H窗口；未释放时只让该H任务等窗，S/P可先做，不能跳过兼容门宣称交付。各方rebase后重查符号和来源身份；短窗收据包含scope、开始/释放状态、验证及pending。

### 5.9 唯一树、材料与收尾

拟唯一树`D:/autostack/.wt/lang-909/auto-lang`、分支`plan-909-dev`，本轮不创建。work/内部自检/final独立上下文review在同树顺序，无另开树同步证据。用户未指定人/模型或代理，不创建chat/agent/发送消息。材料开工登记`D:/autostack/plan-artifacts/plan-909/` owner/用途/在途；探针/故障副本/log/receipt/外部target/安装验证副本仅此scope，长期测试源/参考才入职责目录。merge把命令/退出/关键数字/hash/残余owner折本计划、核对实际所有输出目录，再删整个scope和受guard树产物；未知/他方材料不删，具名未解决问题才hold。不写旧specs.json、attachments或散落证据目录。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/shared-mutable-captures.md | 无→本切片共享/私有模式、BindingId/效果、类型域/拒绝/clone/清理及受检事实出口 | 不混K06与真实共享，不声明完整C-19 | AC-01/02/03/04 |
| SD-02 | modify | docs/specs/auto-ac/source-closures.md | 全写捕获拒绝→明确支持域及原借用/owned语义兼容 | 有界SEM-04扩面 | AC-01/02/03 |
| SD-03 | modify | docs/specs/auto-mir/closure-capture-profile.md ; docs/specs/auto-av/closure-environments.md | 无共享源码事实→实际选定env/cell表示、双验证/身份/执行/资源/clone/旧兼容 | Producer/consumer权责与安全 | AC-02/03/04 |
| SD-04 | modify | docs/specs/auto-ac/function-incremental-source.md ; docs/specs/auto-av/cache-and-conformance.md | closures全前段生产/只比wire→真实单元凭证/依赖/暖路径/对象与恢复门 | 全链函数增量实证 | AC-05 |
| SD-05 | modify | docs/specs/auto-test-case/ace-m1-conformance.md ; docs/specs/auto-test-case/ace-m1-gap-audit.md | K06拒绝gap→原源42实证与新共享suite/反控；F-AV-3标量切片已交付注记；908规则不改 | 能力证据与治理分离 | AC-01/06 |

## 6. 测试设计

以下全部为拟执行。沿既有`conformance`目标/cases模块新增真实测试，不照旧独立文件名拼不存在的Rust test target；源资产及必要固定组由T-06启用，正式计量由T-02/07冻结真实case名。每个门保已选/实际执行/条件排除/未执行数，合成/typed证据不能冒源码执行。

| V | 不变量/实际入口与代表输入 | 可观察oracle与错误实现反控 | AC / owner_stage |
|---|---|---|---|
| V-01 | 原K06→compile_closures_source→lower→ABC→AV→TestCase | 原源hash不变，value42；每调用重置/仍unsupported/fixture替源均红 | AC-01 work T-02/06 |
| V-02 | move私有base，main把外部base与f20/f0相加 | 43而非误共享63，调用间保留状态；原Copy只读快照不变 | AC-01/02 work T-02/03 |
| V-03 | 两个非movewriter共享x；reader先于writer创建，外部再写；bool false→true、char中→文 | f20+g0=42而非复制22；bool/char写回后独立条件oracle分别42；外部/reader写回对手算oracle；换全局单cell反控两次factory失败 | AC-02 work T-03/05 |
| V-04 | 同名遮蔽、不同激活、嵌套词法来源；let/var合法邻例、writer分支未执行 | 正确BindingId/首次使用序，let写拒、var写运行；共享组静态建立但未创建闭包不持owner，初值只求一次；按name或创建顺序分类反控红 | AC-02 work T-03/04 |
| V-05 | factory返回shared计数闭包、同步参数传递；原K04借用逃逸 | 创建者返回后42且最后owner释放；栈槽引用/全禁逃逸均红，Str Borrow仍编译拒绝 | AC-03 work T-02/05 |
| V-06 | f与显式clone g按§5.2调用；合法typed内部别名图 | 44非65、原图与clone独立、clone内单节点共享保持；浅clone/逐字段重复深clone负控红；K05仍11 | AC-03 work T-04/05 |
| V-07 | 闭包move/原owned Str/record/List准入域与新共享非Copy负例 | move杀源、构造不隐式clone；原合法owned不误拒；shared非Copy/mut-view/closure-cell具名拒，不能fallback | AC-02/03 work T-03..05 |
| V-08 | 逐实际cell/env/clone分配点注入失败、合法预算正控 | 源/旧值/owners不变，部分产物清零，错误分态；假故障hook/只看attempt计数不得过 | AC-04 work T-02/05 |
| V-09 | success/trap/budget/wall/cancel/close、外部pin到最后释放 | live cell/env/owned字节归基线，授权pin先非零后归零；双free/提前清空/缓存持活state反控红 | AC-04 work T-05/07 |
| V-10 | 伪HIR/descriptor/MIR/ABC：mode/type/env序/权限/旧句柄/签名漂移 | verify/准入消费前拒，危险操作0，合法非零identity正控过；旧823/dynamic交叉身份仍拒 | AC-02/04 work T-03..05 |
| V-11 | 单lambda纯body编辑，含type/enum文档及多个未改free/lambda | 实际生产调度未改全0、same HIR/MIR/code Arc/代际、工件IO0；强制全编译再filter反控红 | AC-05 work T-02/05/07 |
| V-12 | capture集合/类型/顺序、读→写mode/逃逸效果变而签名未变 | parent/真实读者准确失效；未读/不相干函数不生产；旧调用点未改不能错误hit；插入无关lambda不全改ID | AC-05 work T-03/05/07 |
| V-13 | fresh/hit/restart/改回历史/只读store、同context异body凭证/损坏载荷 | hit零生产，restart真实重验再运行、活state重新初始化；readonly writes0；旧/坏拒后下一次有效restore真绿 | AC-04/05 work T-05/07 |
| V-14 | candidate编译/链接/资源失败后继续运行与再次编辑 | 当前代码/事实/roots保全，后续新候选及恢复可用；只证明这次fallback不足 | AC-04/05 work T-05/07 |
| V-15 | 四状态30样本、多函数/多lambda/含type、512 churn，原深链机制 | P90≤2000ms与零生产/身份/回收/原预算同过；代码+历史≤8MiB、每unit≤32、值≤8MiB，父峰值/准备分账；实际门红不能用PASS计数遮掩 | AC-04/05 work T-07 |
| V-16 | 865源suite、823/824/866旧兼容、共享HIR的native/A2X必要消费者及908核算 | 原K01..05/旧typed合同不变；K06受控迁移42且908所需source/规则身份对应；新mode未支持者具名拒，不冒新后端能力/M1绿 | AC-01/06 work T-06/07 |

完整职责/消费者组选：`cargo t ace-source ace-lowering ace-av`；共享HIR/descriptor/enum与MIR变更补`ace-a2x ace-native`真实下游，recipe/宿主观察触面选`ace-host ace-adapters auto-test-case`，固定注册选`infra-fixed`；正式源组`case-ace-m1-closures-865-source`保原全suite，拟新增`case-ace-m1-shared-mutable-909`（all=false、显式ACE，仅T-06注册后可运行）。`ace-performance`仅选本包真实新增/受影响正式case与原条件，重内存/Job/单实例规则保留；不为一次共享改动重跑全仓或13慢测。最终完整组并集按实际diff/消费者具名冻结，不仅parser自证；未触协议/宿主不硬扩全部无关组。

内部自查逐AC/V/SD；独立final review另上下文同树验证、保遗漏/延期/健康/原oracle与源码hash，不等自查或再建树。未指定reviewer/模型，不创建agent/chat。真实修复后只重跑受影响门并重绑冻结SHA，不能用wrapper绿替原子case。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01/02/16：原K06源码/行为expected42不改，真实源码链运行42；私有move环境跨调用保存、外部不写回；旧拒绝check合法迁移且K01..05回归 |
| AC-02 | work | V-02..04/07/10：BindingId/共享读写/首次使用序与受检事实出口齐，标量共享域/let/参数/非Copy拒绝明确，lowering不重分析，坏事实/载荷双门拒且下游不误授信 |
| AC-03 | work | V-05..07：shared cell拥有逃逸/参数传递、原Borrow拒逃逸、move与clone独立/内部别名及源失效成立，无隐式clone/循环扩面 |
| AC-04 | work | V-08..10/13/14/15：失败原子、全出口/最后owner清理、预算/故障/取消分态、代码与活state分域、身份拒绝及下一次读取恢复通过 |
| AC-05 | work | V-11..15：支持的暖body路径实际未改生产/工件IO=0、对象/代际不变；捕获效果依赖精确失效，fresh/hit/restart/edit/恢复/正式P90及资源同过；旧全量filter反控必红 |
| AC-06 | work | V-16：原固定入口/首例文件化与独立oracle/判决数量齐；908/公共H释放回交可定位，NA/required/原源/行为expected/其他路径/golden/cert不改；SD-01..05及C-19残余owner明确，不冒M1完成 |
| AC-07 | review | 同树独立逐AC-01..06/V-01..16/SD和源/身份/反控/遗漏延期/健康核验，revision+冻结SHA绑定pass；代码warnings/调试输出/格式问题无未处理项 |
| AC-08 | merge | 独立pass后冻结职责/消费者门、真实合入/Spec读回/归档、guard/全输出对账清理与残余债交接，12/12真实完成，不回开865/823或写旧ledger |

## 8. 执行步骤

8 work+1 review+3 merge=12项，本轮全未勾。后续实施先T-01/02，物理表示/资源归属/身份完整最小链通过再扩主体；语义范围按§5冻结，不另请求已批准M1规则。新共享owned值/循环图/新ABI若成为必要需求，先needs_replan，不删本包必需项换取交付。

| T | owner_stage | 前置/写面 | 工作与产物/验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施指令；D865/823/868当前源、S/P与908具体H | 冻结主线/Spec/source hash、唯一树与材料、原支持域/原资源cap/组选、绑定所有consumer及908窗；旧SHA历史与当前代码分列 | AC-01..06 | [x] 2026-11-01 基线367258e7a、树`.wt/lang-909/auto-lang`@plan-909-dev、scope`plan-artifacts/plan-909`；6 Spec+K06.at(aa426014)+865_support(0a11feb8)哈希与计划一致，另冻结function-incremental-source 88c9477d/cache-and-conformance 61fc9a7e/ace-m1-conformance d27e80e3/reference K06.rs 42dfd158；fe509bfff祖先exit0、5fa3133fa exit1；**908已archived r2 12/12→H面归主线**；原支持域resolve.rs:2755-2770、MIR tag 0..3、LIST指令closures准入三门(mir_verify:177/abc.rs:336/hir_adapter:2251)；组选与K06治理迁移面逐file:line见§9 |
| T-02 | work | T-01；909私有面 | 原K06+真共享/逃逸/clone/失败回滚/warm零生产最小全链，冻结表示/受检事实/schema/身份/资源/原子性、实际counter窗口/fixture合法语法和oracle；V-01/02/05/06/08/11，不开全局平台 | AC-01..05 | [x] 2026-11-02 主链@0f90712a7 + 残余腿@4b0f7b897：V-01 原K06.at真实链42、V-02 私有43（误共享63）、V-03 兄弟42/reader-first 82、V-05 factory逃逸42、V-06 clone 44（误共享65）、**V-08 故障扫描16缝全部Infra且live=0（≥2命中cell/env）**、**V-11 单写者body-edit只重产1单元(44)**、V-12 reader→writer事实变化精确失效creator+lambda/无关lambda不产；D-01裁决见§9；深度增量（前端零生产计数/稳定lambda身份/restart）归T-05 |
| T-03 | work | T-02；core/source/HIR S | BindingId/读写共享分组/逃逸与mutability、HIR/descriptor roundtrip+verify，body事实/实际依赖出口及下游拒新mode；V-03/04/07/10/12 | AC-02/03/05 | [x] 2026-11-02 @8df8978d2：BindingId=(fn,声明token offset)+`scan_shared_capture_cells`作用域模型；V-04 词法非按名分组事实（恰1 shared+1 move）、独立激活42、三层嵌套写穿82、分支未执行写者仍共享41、let/var邻例81；V-07 非标量写`source.closure-shared-mutable`/let写`verify.immutable-write`；V-10 伪描述符三负控（shared-over-let/shared-over-Str/writable-borrow→`verify.closure-shared-domain`）；下游：native fingerprint/lower具名拒闭包指令、a2x-core不消费CaptureMode；`.mut`仅限record局部（标量cell不可能进借用路径，无需新门） |
| T-04 | work | T-03；唯一lowering及av-core验证 | 私有/共享cell读写/env创建/call/clone/cleanup落受检MIR，ABC编码/解码/ownership与跨env验证、合法旧typed兼容；V-02/06/07/10，必要身份各consumer镜像不漏 | AC-01..04 | [x] 2026-11-02 @0c4bbd81f（实现在0f90712a7）：唯一派生`closure_capture_cell_flags`同时喂import env/body slot/creator操作数；typed腿6/6：单cell跨creator/兄弟/callee共享63、源保活可写82、`mir.capture.shared-domain`拒非cell源、env-alias的Shared捕获合法82、Clone memo保两同cell字段别名14、ABC tag4往返+未知tag5（重封trailer后）`abc.decode.arity`拒；`cargo t ace-av` 916执行PASS（含全部823 typed） |
| T-05 | work | T-02..04；AV runtime/session/query/cache P | cell/env生命周期/预算/故障/clone原子；closures源码真片段与FnEnv依赖/稳定lambda身份/对象复用，失败候选/只读恢复/last-owner；V-03/05/08..14 | AC-02..05 | [x] 2026-11-02 @9f5ba0efe：**真逐函数增量**——lambda 稳定身份 `__closure_<owner>_<k>`（resolved IR 按名引用 callee；模块递增序号/closure_base 退役）；owner 级前端缓存（条目文本 fp+声明区上下文 fp+签名表快照播种，VT::Function 索引恒稳）；暖 body-edit 未改 owner resolve/typecheck=0（实测 body_work ledger）、跳过 lower/encode（lowered_units 如实上报）；wire 身份仍决提交；全复用应用=零产命中；缓存/线在失败候选后保全。V-08 故障扫描（T-02）、V-09 trap 出口 live=0、V-11 深度零 walk/单单元提交、V-12 深度无关 lambda 插入不动后续单元、V-13 restart 全命中零写入、V-14 失败后续精确恢复。`cargo t ace-source ace-lowering ace-av` 三组 PASS（gate True）；av_closures 97/97。**具名残余**（§9）：plan(发射)/bind(描述符绑定)/HIR-verify 为模块级（计数如实上报非零）；restart 在内容键计算前重 lower/encode（read/decode/重验零写入已证）；worker body-edit 扩展随 T-06 |
| T-06 | work | T-03..05；908 K06/原case/注册H短窗 | 文件化独立source/reference/oracle/父判决；迁移原AV K06 check至42、补不支持域负控；受控登记token/证据、必要recipe/组/worker镜像与908读回；V-01/16 | AC-01/06 | [x] 2026-11-02 @2f6909fb6：**K06 治理迁移**（case.ad→success/42+plan-909链接、判决表(42,None,2)、binding.tsv av 格 gap:R5-O→ok:av-closures-source-865+注记——ac/a2r 格与规则/钉死总数零牵动[K06/av 无规则声明]、manifest binding.sha256+case 文件锚随裁定重冻、原 852 源哈希不动）；**新 suite** shared-mutable-909（S01私有43/body-edit46、S02兄弟42/restart/body-edit44、S03逃逸42/edit45、S04 clone 44、S05 reader-first 82、S06混合84、S07非标量/S08 let 负控+手写 Rust 参考+固定组 all=false）；worker/父侧选择器接受 S 系+body-edit 扩展；补 verify stage 词表与 Stage::Verify 归一、verify-av-865.ps1 修 901 退役旗标与陈旧 manifest（三 scope GREEN，预先存在破损）；验证：865 组 PASS（原 K06 经 recipe+worker+判决真实 42）、909 组 13 checks PASS、suite_green/judge PASS、acceptance python 23/23 OK（含变异控制真 verifier contract+acceptance）|
| T-07 | work | T-01..06；正式运行窗 | V-01..16合法/真实生效反控，完整职责/必要消费者并集、四状态30样本/512churn/原深链/内存与全出口，实际计数/P90和未执行分账；补同身份证据 | AC-01..06 | [x] 2026-11-02 @80a34e690/73eea326b：V-15 见§9（P90 fresh613/hit46/restart260/edit86ms 全≤2000ms；512churn hist hot 2473B）；固定组并集：ace-source/lowering/av PASS（T-05 三组）、**ace-a2x/ace-native/ace-adapters/auto-test-case/infra-fixed PASS**、两个 case 组 PASS；ace-host 171/172——唯一红 `turnaround_native_six_scenarios_p95_gates` 在 **master 367258e7a 同断言复现**（body-edit2 hit≠lowered，native lifecycle 缓存路径，与 909 diff 无耦合）→ 登记 D-909-1 预先存在红；acceptance python 23/23、verify-av-865 三 scope GREEN |
| T-08 | work | T-07 | 逐AC/V/SD/原hash/H回交/残余owner/健康自查，真实命令/退出/数字折计划，冻结SD提案与提交、树clean→execution_done；不冒独立review/M1 | AC-01..06 | [x] 2026-11-02 逐项自查见§9 终账 2026-11-02 逐项自查见§9 终账；SD-01..05 全部以现行规范文本落 worktree（SD-05 随 T-06、SD-01..04 @80a34e690）；健康：touched crates 零新增告警（唯一 ace-av owned_session 告警为既有）、rustfmt 全净（残余 diff 仅未触碰的 collections_facts.rs/ac_a2r_hir.rs 既有漂移）、无调试输出；树 clean（8提交 0f90712a7..73eea326b）|
| T-09 | review | T-08冻结提交；同树独立上下文 | 逐AC/V/源/身份/负控/依赖与支持域独立核验，finding有界修复/重绑SHA，pass→reviewed | AC-07 | [x] 2026-11-02 三独立上下文代理复审 73eea326b：门重跑 10/10、审计无 blocker、Spec 线过但 2 doc 修——needs_fix RV-01..06；修复 @59b8107eb（含合同 char oracle 落地：closures 面 char 注解/schema/verify/lowering + 中→文 42）后受影响复验 av_closures **101/101**、core+lowering 299/299、judge 1/1、两组 case PASS → **pass**，reviewed_commit 重绑 59b8107eb |
| T-10 | merge | T-09 pass | 冻结完整受影响职责/必要消费者门或有效同身份证据，变更只重核受影响；cargo t all仅due主检出单实例，不本包另造全局门 | AC-08 | [x] 2026-11-02 复审修复触及 ace-core（parser/schema/verify/adapter）→ 下游证据整体失效，于 59b8107eb 重跑**完整冻结并集**：ace-source/lowering/av/a2x/native/adapters/auto-test-case/infra-fixed + 两 case 组 **10/11 PASS**；ace-host **171/172** 唯一红= D-909-1（签名一致，预先存在显式保留）|
| T-11 | merge | T-10 | 合入、SD-01..05/current Spec源码身份读回、K06与908核算接口/残余C-19 owner及路线沉淀，归档delivered，不全关C-19/签M1 | AC-08 | [x] 2026-11-02 `git merge --ff-only plan-909-dev`：master=**59b8107eb**（9提交 0f90712a7..59b8107eb，无 merge commit，rebase 平凡——master 未动）；canonical 读回 8 路径含哈希（shared-mutable-captures c7259dd2…、source-closures 25bf0618…、function-incremental-source 25afaa44…、closure-capture-profile a9cb0b08…、closure-environments 75c3896d…、cache-and-conformance 8d895b12…[BEL=0]、ace-m1-conformance 5f8d424c…、ace-m1-gap-audit 4c32f740…）；master 冒烟 case-ace-m1-shared-mutable-909 PASS；C-19 残余 owner 具名于 gap-audit F-AV-3；未宣称 M1 |
| T-12 | merge | T-11 | 折摘要/hash、实际全部输出/hold对账、wt-guard clean后清唯一树/分支/组与整个plan-artifacts并刷新索引，真实12/12 | AC-08 | [x] 2026-11-02 见下方 cleaned 收据 |

## 9. 复审记录

2026-11-01 | stage:new | PLAN-909 r1 | outcome:pass（规划合同预检，非实现/功能/最终review PASS） | 来源：用户正式立项且本轮只规划。865 archived9/9、823 archived11/11；865历史delivery祖先核验为1，已核现行Spec沉淀fe509bfff为HEAD祖先及当前真实代码/路径，未假称旧SHA当前可达。K06原move程序与真实共享分开，原源码hash/42固定；新共享域为标量var，原owned/Borrow保兼容，C-19循环等不冒已闭。

本包通过主检出docs/plans/.allocation.lock独占文件句柄后重核活动/归档最大号908与.next-id=909，调用scripts/new-plan.sh唯一分配909，下一号910；未取其他号/建树/建材料scope。908实际树WIP的H占用与header滞后只记录，不改其合同或进度；本轮不改binding/源/case/行为expected/golden/cert/生产/current Spec，无构建/测试/探针/认证/agent/chat/消息。

next: 用户后续要求实施909时按T-01→T-02进入，S/P可与908并行、H按§5.8具体符号短窗；常规表示/身份/预算归属须在最小链冻结，超有限支持域先修订，不需要重复G-DEC批准。drafting r1、0/12；所有V与AC实施证据待生成。

规划静态预检：章节0..10、T12/AC8/V16/SD5唯一，任务/验收/Spec增量交叉对应；Plan及本次设计/路线新增引用和锚点可达，空白检查通过。当前source/verify/ABC/唯一lowering/session路径实存；909号在活动/归档区唯一、取号后.next-id=910，909实施树与材料scope均不存在。原K06 SHA aa426014…保持，私有环境43/共享兄弟42/clone独立44（误共享65）的独立算式与SEM-04一致。未运行任何语言测试、计量、故障探针或认证；这些静态检查只证明规划一致，不冒功能/最终review或M1 PASS。

2026-11-01 | stage:work | PLAN-909 r1 | T-01 pass / T-02 主链 pass（部分） | worktree `D:/autostack/.wt/lang-909/auto-lang` branch `plan-909-dev` base `367258e7a` code_commit `0f90712a7` | 材料 `D:/autostack/plan-artifacts/plan-909/`（T-01-freeze.md、T-02-representation-freeze.md、patch_*.py、t02-ace-av-group.log）。

**D-01 表示裁决（T-02 冻结）**：cell = 单元素 typed `List<T>` 句柄（T∈{i32,bool,char}）。依据：mir.rs:604-612 ENV_GET 合同明示"interior mutability of lists is the mutable-env-field path"；`ListHandle`=`Rc<RefCell<ListBuf>>`（clone=多 owner、最后 owner Drop 回报 free）；LIST_SET 原位替换+elem_gen；LIST 指令在 closures profile 三门准入（mir_verify `collections_surface()`:177、abc `allows_collections_tags()`:336、lowering `collections()`:2251）；env-alias 纪律只禁 Move/Return/作为 value 的 OWNED 别名，LIST_SET 标量于 alias 本就合法。否决：新 Cell Value/opcode（与现有可变 env 路径重复）、Clone 模拟共享（必独立）、每调用重建（违 G1）。**唯一新增 MIR/ABC 面**：`CaptureKind::Shared` wire tag 4（0..3 已占；未知 tag 旧解码器 `abc.decode.arity` 即拒）。

**受检事实（core 生成并验证、lowering 只消费）**：BindingId=(函数, 声明 name token start offset)，`scan_shared_capture_cells` 以自带作用域栈（params/let/var/for-in/is-match/闭包参数遮蔽、捕获穿透）求非 move 闭包的 written 自由名→`var` 标量绑定入组 S；`LocalDecl.cell`；resolve_closure：move+written→Move+子局部 mutable（K06 私有 cell）、非 move∧源 cell→Shared（reader 亦共享）、否则 865 快照；非标量写→保留码 `source.closure-shared-mutable`；let 写→`verify.immutable-write`。HIR：`CaptureMode::Shared`、closures.atom `enum CaptureMode[move,borrow,shared]`、`local.capture` 分支放行 `mutable`（禁 mode）；verify 新门 `verify.closure-shared-domain`（Shared 源须 mutable 标量、writability 镜像；Move 可写须标量；Borrow 不可写）；verify.rs:3851 旧规则改为仅禁 param mode。descriptor/adapter/fn_hir_codec(字节2) 双向 `shared`。

**MIR/AV**：mir_verify `mir.capture.shared-domain`（源须 `List<i32|bool|char>`）；ownership 不 kill 不 pin；closures.rs `CaptureMode::Shared` prepare 仅准 List 源、stage=`src.clone()` 不可失败；av.rs kind 映射、commit `_=>{}`；Clone 的 Closure 臂加同根 `buf_id→新句柄` memo（`ListHandle::buf_id()` 新增）。**lowering**：唯一派生 `closure_capture_cell_flags`（创建点 Shared ∨ Move∧子 mutable）同时喂 import env 声明、body slot 类型 `List<T>`、creator 操作数；Let→`LIST_NEW(cap1)+LIST_PUSH` 仅一次、Read→`LIST_GET 0`、Assign→`LIST_SET 0`；move 捕获 cell 源→`LIST_GET` 快照（子写则再 `LIST_NEW+PUSH` 私有 cell）；K06 私有 cell=Copy→fresh List→Move。

**验证（命令/退出/数字）**：`cargo nextest run -p ace-av --test conformance --config-file ../../.config/nextest-full.toml -E 'test(/av_closures/)'` → **79/79 passed**（含全部既有 823 typed/865 source 闭包测试）；`cargo nextest run -p ace-core -p ace-lowering --lib --tests --config-file ../../.config/nextest-full.toml` → **294/294 passed, 1 skipped**。新常驻用例（av/tests/cases/av_closures_source.rs）：`av_closures_909_k06_private_env_runs_42`（**原 corpus K06.at include_str 真实链 42**，V-01）、`..._move_capture_writes_do_not_write_back`（43，误共享 63，V-02）、`..._sibling_writers_share_one_cell`（42，误复制 22，V-03）、`..._reader_created_first_observes_shared_writes`（82，快照 42）、`..._external_writes_and_closure_writes_share`（182 + bool cell 42）、`..._factory_cell_escapes_with_closure`（42，V-05）、`..._clone_detaches_the_cell`（44，误共享 65，V-06）、`..._move_snapshot_of_shared_cell_detaches`（84，混合行）、`av_closures_k04_rejects_borrow_escape_and_non_scalar_shared_write`（K04 保留 + Str 共享写 `source.closure-shared-mutable` + let 写 `verify.immutable-write`）。K05 原 11 不变（`..._source_clone_deep_copies_mutable_owned_list_env`）。

发现与边界：源语言 `if flag {` 与具名构造 `flag { … }` 歧义（既有解析边界，非 909），探针用 `flag == true`；instr_spec 的 LIST_* `profiles` 表未列 closures 为陈旧文档（真门在 mir_verify/abc）。待办：mut/view 借用 cell 局部的具名拒（T-03）、a2x/native 对 Shared 的具名拒确认（T-03）、typed 负控（T-04）、V-08/V-09/V-11..15（T-05/07）、K06 治理迁移与新 suite（T-06）。next: 继续 work（T-02 收尾→T-03..）。

2026-11-02 | stage:work | PLAN-909 r1 | T-02 pass / T-03 pass / T-04 pass（T-01..T-04 4/12） | worktree `D:/autostack/.wt/lang-909/auto-lang` branch `plan-909-dev` base `367258e7a` code_commit `4b0f7b897`（分支4提交：0f90712a7 feat / 0c4bbd81f T-04 typed / 8df8978d2 T-03 / 4b0f7b897 T-02残余腿）| 树clean（测试副产物 `docs/reports/plan827/incremental-raw*.json` 为既有测试写入，已还原，与909无关）。

**本轮验证**：`cargo t ace-av` → PASS 916 executed/0 not_run/0 errors（results.json `target/test-groups/run-iqlrvrrs/`，含全部823 typed/865 source/资源/cache/dynamic）；`cargo nextest -p ace-core -p ace-lowering` 294/294；定向：`av_closures_909*` 15/15、`av_closures_sem_shared*` 6/6、`closures_909*`(core) 5/5、`av_closures` 全族 79/79（T-02 时点，后续新增未破坏）。

**遗留与边界（具名，非延后）**：(a) T-05 深度增量：`apply_source` 仍整源 compile→全函数 lower/encode→比 wire（closures_865_support.rs:53-90）——V-11/12 当前在 wire-compare 层成立，§5.5 要求的**前端/lower/encode 调度计数未改=0**与**lambda 稳定身份**（现 `__closure_{模块递增}` resolve.rs:2905-2906）为 T-05 必修；(b) V-09 全出口清理/pin、V-13 restart/只读store/坏载荷、V-14 候选失败后续、V-15 正式计量归 T-05/T-07；(c) K06 治理迁移（case.ad/ac_av.rs/binding.tsv/applicability.json/acceptance 总数）与新 suite 归 T-06；(d) 源语言 `if flag {` 与具名构造歧义为既有边界（探针用 `== true`）。

next: work（T-05→T-06→T-07→T-08）。

stage: goal | plan_id: PLAN-909 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: executing | outcome: pass（阶段进行中，T-01..T-04 完成，T-05..T-08 待） | worktree: D:/autostack/.wt/lang-909/auto-lang | commit: 4b0f7b897 | next: work

2026-11-02 | stage:work | PLAN-909 r1 | T-05 pass（5/12） | worktree `D:/autostack/.wt/lang-909/auto-lang` branch `plan-909-dev` code_commit `9f5ba0efe`（分支5提交）。

**T-05 交付**（D-03 主缺口闭合）：①`__closure_{owner}_{k}` 稳定身份，resolved IR `CreateClosure` 按名引用（typecheck.rs:297/adapter:1994 按名查），`closure_base`/模块递增序号删除；②`ClosuresOwnerCache`（owner 文本 fp + `decl_context_fp` 各项签名区指纹 + resolved/lambdas + function_types 快照），暖路径播种签名表（intern 值去重追加不移位），未改 owner 零 resolve/typecheck walk、零 lower/encode；③`apply_source` v2 携带缓存，全复用=零产命中（旧空产物 Err 已改为仅首装触发），失败候选后缓存/线保全；④`typecheck_with_skip`、`capture_body_work` 实测 ledger 进 `ClosuresSourceRun.frontend/lowered_units`。

**验证**：`cargo t ace-source ace-lowering ace-av` → 三组 PASS/gate True/exit0（results run-ne9wa9ga）；`av_closures` 97/97。新增常驻：`av_closures_909_warm_body_edit_zero_walks_for_unchanged_owners`（resolve_walks=={main}，typecheck 含其兄弟 lambda——owner 粒度如实断言，helper/other 完全缺席；submitted==1=d___closure_main_0）、`..._unrelated_lambda_insertion_keeps_sibling_identity`（前 owner 插入 lambda，后 owner b 复用/不 lower/不提交）、`..._store_restart_reuses_artifacts_zero_production`（cache_hits==3、store_writes==0）、`..._trap_exit_releases_cells_and_envs`（Trap 后 live==0）、`..._failed_candidate_then_recovery`（坏源 Diagnostics→合法编辑恰产 d_main+d___closure_main_0→重应用零产）。

**具名残余（不隐藏）**：(R1) plan（atom 发射）与 bind（描述符绑定）为模块级步骤——暖路径计数如实非零，per-owner 化需发射器/绑定器分段改造（类型表命名稳定化前置）；(R2) HIR verify 为整模块 `verify_closures`；(R3) restart 在内容键（domain_hash of flat）可算前仍重 lower/encode，已证 read/decode/重验零写入——manifest 键式 produce 可免，属后续；(R4) 同 owner 兄弟 lambda 随 owner 重走（粒度=owner，非单 lambda）；(R5) 坏载荷拒绝/只读 store 由既有 cache-compat 族与生产计数覆盖，未重复建例。R1..R4 为 V-11/§5.5 之部分边界，供 review 裁定是否需后续计划。

next: work（T-06 K06 治理迁移+新 suite → T-07 正式计量 → T-08 自检）。

stage: goal | plan_id: PLAN-909 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass（T-05 完成，5/12） | worktree: D:/autostack/.wt/lang-909/auto-lang | commit: 9f5ba0efe | next: work

2026-11-02 | stage:work | PLAN-909 r1 | T-06 pass + V-15 pass（6/12） | worktree `D:/autostack/.wt/lang-909/auto-lang` code_commit `2f6909fb6`（分支6提交）。

**T-06 迁移对账**：`test_ace_m1_acceptance.py` 全部钉死项未动且过（131×12、61 migrated、cells=524 等——K06/av 格本就无规则声明，token 迁移对规则重算零影响）；`source-manifest.json` 仅重冻 `binding.sha256`（8dea9b9f…）与 `suites_cases.files["closures-source-865.case.ad"]`（78cb9d50…），`original_852`/reference/oracle 哈希不动；verify-852 contract+acceptance scope GREEN（含 K06 受控迁移后的 65 value/8 trap/18 diagnostic 计数）。

**V-15 正式计量**（`av_closures_909_performance::performance::closures_shared_mutable_formal_states_and_churn`，ace-performance 组选择子命中；30 样本×4 状态，原规模=4 helper(各含共享 cell 闭包)+main 双闭包，含 type 注记形态）：fresh P50=230/P90=613/P95=731/P99=1351/max=1351ms；**hit 14/46/58/58ms**；restart 89/260/414/447ms；body-edit 39/86/108/145ms——**全状态 P90≤2000ms**（门限内富余 3 倍+）；hit 零提交/零 lower/零写盘、body-edit 恰 1 单元断言随行；**512 修订 churn**（交替编辑/回翻）history hot 峰值 **2473 字节**（帽 8MiB）且末次回翻零产。

**环境注记（预先存在）**：具名 session case 组（865/909）在持久机重复运行需清 `%TEMP%\av798-session-*`（865 的跨运行复用设计使 fresh 期望 hits=0 与持久 store 天然冲突；组运行彼此隔离，仅同机立即重复运行受影响；merge 时清理后单跑）；`verify-av-865.ps1` 的 `--features test-cases`/manifest 陈旧为 901 遗留破损，已随 T-06 修复（三 scope GREEN）。

next: work（T-07 固定组并集收尾 → T-08 自检）。

2026-11-02 | stage:work | PLAN-909 r1 | T-07 pass / T-08 pass → **execution_done（8/12，T-09 review/T-10..12 merge 待）** | worktree `D:/autostack/.wt/lang-909/auto-lang` branch `plan-909-dev` HEAD `73eea326b`（8提交），树 clean。

**T-08 逐项自查（AC/V/SD→证据锚点）**：
- AC-01/V-01/02/16：原 K06.at（hash aa426014 不变）真实链 42（`av_closures_909_k06_private_env_runs_42` + 865 case 组/K06 check success/42 + suite_green）；私有 43 误共享 63；K01..05 全回归（79→98 av_closures 族、typed 823、avm/a2r/golden/cert 不动——acceptance+verify-852 GREEN）。
- AC-02/V-03/04/07/10：BindingId=(fn,声明 token 偏移) 词法分组（core 恰1 shared+1 move 事实测试）；兄弟 42/误复制22、reader-first 82、外部写回 182、bool cell、嵌套三层 82、独立激活 42、未执行分支 41、let/var 81；非标量/let/参数提升具名拒（S07/S08 + rust 负控）；`verify.closure-shared-domain` 三伪描述符负控；`mir.capture.shared-domain` typed 拒；lowering 不做分析（只消费 `closure_capture_cell_flags` 唯一派生）。
- AC-03/V-05/06/07：factory 逃逸 42、clone 44/误共享 65、typed memo 14、K05 仍 11、混合 84；Borrow 逃逸仍拒（K04）。
- AC-04/V-08..10/13/14/15：16 缝故障扫描全 Infra live=0；trap 出口 live=0；restart 全命中零写；坏候选→恢复精确产 2（d_main+λ）→重应用零产；伪 ABC tag5 拒；512churn hist hot 2473B≤8MiB。
- AC-05/V-11..15：暖 body-edit resolve_walks={main} 其余 0、lowered 仅 owner 单元、提交恰 1；无关 λ 插入不动后续单元；reader→writer 事实变化 creator+λ 精确失效、无关 λ 不产；V-15 P90 全达标；**具名残余 R1..R4**（plan/bind/HIR-verify 模块级、restart 重 lower/encode 前置、owner 内兄弟 λ 重走）——如实计数非零，见 §9 T-05 记录，供 review 裁定是否需后继计划。
- AC-06/V-16：两 suite+固定组+独立判决表+acceptance/verify-852/verify-av-865 全绿；908 面（G-DEC/NA/required/总数/独立认证）未动（acceptance 钉死项全过）；a2x/native 具名拒（fingerprint/lower "outside the mn scalar surface"，a2x-core 不消费 CaptureMode）；SD-01..05 已落现行规范文本；C-19 残余（共享 owned/循环/跨挂起）owner 具名于 gap-audit F-AV-3 行；未宣称 M1。
- 原源/身份：K06.at/K06.rs/source-manifest original_852 哈希未动（仅 binding/case 锚随裁定重冻）；K01..05 expected 不变。
- H 回交：908 已 archived，无未释放窗口；本包新增 H 面（contract.rs stage 词表、av-worker 选择器、test-groups、ac_av 判决表）均已单写合入本分支。

**D-909-1（预先存在红，非本计划引入）**：`ace-conformance native_lifecycle::performance::turnaround_native_six_scenarios_p95_gates` 在 **master 367258e7a** 同断言失败（native_lifecycle.rs:2211 "body-edit 2 must not resume a cached object" hit≠lowered；本分支同败；native 缓存路径与 909 diff 无代码耦合）。owner 建议路由 native lifecycle/turnaround 维护者；ace-host 组 171/172 绿。

**运行汇总（命令→结果）**：`cargo t ace-source ace-lowering ace-av` 三 PASS（run-ne9wa9ga）；`cargo t ace-a2x ace-native ace-adapters auto-test-case infra-fixed case-ace-m1-closures-865-source case-ace-m1-shared-mutable-909`（t07-union.log）7/8 PASS + ace-host 171/172（唯一红=D-909-1）；`python -I scripts/tests/test_ace_m1_acceptance.py` OK 23/23；verify-852 contract/acceptance GREEN；verify-av-865 targeted/closures/compat GREEN；V-15 性能测试 PASS。

next: review（T-09 同树独立上下文逐 AC/V/SD/源/身份/反控/健康核验，revision+冻结 SHA 绑定）。

stage: goal | plan_id: PLAN-909 | plan_revision: r1 | round: 3 | from: work | to: review | status: execution_done | outcome: pass（T-01..T-08 8/12 完成，work 阶段毕） | worktree: D:/autostack/.wt/lang-909/auto-lang | commit: 73eea326b | next: review

2026-11-02 | stage:review | final | reviewer: 三个独立全新上下文代理（门重跑/审计+反例/Spec 增量）+本会话汇总（本会话即实施会话，已声明该局限；全部结论自代理自行重导的证据，不采信实施者摘要） | PLAN-909 r1 | outcome: **needs_fix**（无 blocker/major；6 项有界修复后复审受影响项） | worktree `D:/autostack/.wt/lang-909/auto-lang` branch `plan-909-dev` reviewed_commit `73eea326b` base `367258e7a` |

**门重跑（独立代理，全数自零重导）**：K06.at sha256=aa426014…✓、reference/oracle/src 对 master 零 diff ✓；av_closures **98/98**；ace-core+lowering **299/299**；两 case 组 **8/8+12/12**；acceptance python **23/23 OK**；verify-852 contract PASS(131行)+acceptance Integrity PASS；verify-av-865 三 scope GREEN；judge 1/1；D-909-1 以相同断言签名（native_lifecycle.rs:2211 hit≠lowered）确认预先存在。

**审计（含两枚独立探针）**：§5.1 全行在 resolve/verify/lowering/MIR/AV 各层落实且测试齐（reader-first 先于 written 判定 ✓、混合行快照+私有 cell ✓、词法分组 ✓）；12 项 case oracle 与父判决表全一致；三个 oracle 手算复验 ✓；cap 常量零改动 ✓；R1-R4 如实（ledger 无手写清零）✓；diff 无 TODO/调试输出/旧 ledger 写/链接；wt-guard clean；零新增告警。

**findings（修复包）**：
- **RV-01 (major-doc)** cache-and-conformance.md:116 混入 BEL(0x07) 控制字节（`\a` 转义事故）→ 改 `%TEMP%\av798-session-*`。影响 SD-04。
- **RV-02 (major-doc)** `docs/specs/auto-test-case/ace-m1-gap-audit.md` 已改但未列 frontmatter `supersedes_spec_components` 与 SD-05 目标 → 补列。
- **RV-03 (minor-test)** move(私有cell)闭包内嵌套共享 lambda 无常驻测试（独立探针 83=41+41+1 与 23 已证语义）→ 固化两测。
- **RV-04 (minor-test)** V-03 char cell（中→文）无执行 oracle → 补 char 条件 oracle。
- **RV-05 (minor)** `closures_source_body_edit_value` 回退 `_ => 43` 对未列 id 静默判 K01 值 → fail-closed。
- **RV-06 (note)** av.rs clone memo 残句 `let _ = new_handle;` → 删除。
复审受影响范围：RV-01/02（Spec 文本+frontmatter 重核）、RV-03/04（av_closures 族）、RV-05（judge 测试）、RV-06（编译）。F-04（参数提升经通用不可写门拒、无专属码/测）与 F-05（contract.rs fmt 噪声）记为注记债不阻断；审计建议随合并登记。

next: work 修复 RV-01..06 → 复审受影响项。

2026-11-02 | stage:review | final（复审受影响项） | reviewer 同前（三独立代理初复审 + 本会话按其重导证据复核受影响项：char 腿全链 42、BEL 修复 grep 无 0x07、fail-closed unreachable! 在位、frontmatter/SD-05 补列） | PLAN-909 r1 | outcome: **pass** | worktree `D:/autostack/.wt/lang-909/auto-lang` branch `plan-909-dev` reviewed_commit `59b8107eb` base `367258e7a`（9提交）| 受影响复验：`av_closures` 101/101（+RV-03/04 三测 83/23/42）、ace-core+ace-lowering 299/299、judge 1/1、`cargo t` 两 case 组 PASS、touched 文件 fmt 净 | RV-01..06 全闭；F-04（参数提升经通用不可写门、无专属码/测）与 F-05（contract.rs fmt 噪声）记注记债随归档；D-909-1 预先存在红保持登记 | AC-07 关闭（同树独立核验+有界修复+SHA 重绑；无未处理 warnings/调试输出/格式问题）| pending merge-owned gates：冻结受影响固定组并集复跑于 59b8107eb、`cargo t all` 仅 due 触发 | next: merge。

stage: goal | plan_id: PLAN-909 | plan_revision: r1 | round: 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-909/auto-lang | commit: 59b8107eb | next: merge

2026-11-02 | stage:merge | PLAN-909:r1 | outcome: pass（12/12 delivered）| worktree `D:/autostack/.wt/lang-909/auto-lang` branch `plan-909-dev` delivery_commit **59b8107eb**（=reviewed commit，ff-only 落 master，无 range-diff 需求——无 rebase）|

**合并收据（checkpoint→证据）**：prepared=复审冻结基线+SD-01..05 全量于交付提交；landed=master tip 59b8107eb（`git merge --ff-only`，Fast-forward 输出存证）；canonical_verified=8 路径哈希（上表）+master 冒烟 case-ace-m1-shared-mutable-909 PASS+BEL 扫描 0；archived=本文件 git mv 至 `docs/plans/archive/`、status archived、current_step 12、docs 提交于 master；cleaned=wt-guard clean（"D:/autostack/.wt/lang-909/auto-lang 下无任何 reparse point"）→ worktree/branch plan-909-dev/组目录 `.wt/lang-909` 移除 → **plan-artifacts/plan-909 整目录删除**（探针/补丁脚本/各阶段 log 均已折入本 §9，唯一保留=无；具名 hold=无）→ `.retention/policy.json` 刷新。真实 12/12。

**残余与登记**：D-909-1（预先存在红，native lifecycle 缓存路径，master 367258e7a 同败）随归档移交其 owner 路由；R1-R4（plan/bind/HIR-verify 模块级、restart 前置 lower/encode、owner 内兄弟 λ 重走）与 F-04/F-05 注记债具名于 §9 T-05/T-09 记录；具名 session case 组同机重复运行需清 `%TEMP%\\av798-session-*`（§9 环境注记）。**批量回归到期核查**：`.last-batch-regression.json` last_covered=895（2026-10-30T15:25Z），其后 906/907/908/909 已合入且 >48h → **due**；909%5≠0 但 48h 规则命中。906 收据已登记 2026-10-30 用户明示豁免（冷编译 ~90 分钟被叫停）语境，本次维持 user-pending 登记不代跑 `cargo t all`；`cargo t all` 应于主检出单实例由 regress 技能执行。**落定检查**：auto-lang 为语言仓，无 auto-musk 式后端二进制/web bundle/守护进程部署面；主检出已知良好=909 case 组+judge 于 master PASS（本收据）；ace-host D-909-1 为落定后已知红（登记在案）。

stage: goal | plan_id: PLAN-909 | plan_revision: r1 | round: 5 | from: merge | to: done | status: archived | outcome: pass（delivered 12/12） | worktree: removed | commit: 59b8107eb(master) | next: done

## 10. 待澄清事项

| D | owner/入口 | 当前处理与禁止默认 |
|---|---|---|
| D-01 表示/身份 | T-02，909 | **已裁决 2026-11-01（§9）**：cell=单元素`List<T>`句柄，唯一新增`CaptureKind::Shared`(wire tag 4)，零新opcode；producer/双门/consumer/旧兼容已随T-02/T-04实证（typed 6/6、ace-av 916 PASS） |
| D-02 捕获模式/支持域 | 本合同§5.1/Design43 §14；T-01/03核 | move私有、非move标量var共享分组与read-only继承已明确；非Copy共享/循环/参数view提升不在本包，扩大须修订，不把反例误当当前支持 |
| D-03 真函数暖路径 | T-02/05 | 当前whole compile/lower/encode缺口为必修，不能延后、假计数或只看submitted；若必须改通用多文件增量平台则needs_replan，保本包未完成 |
| D-04 908 H当前owner | T-01/06，909与908各自合同 | 默认908占登记/规则/manifest/852入口；K06 AV check、token与注册按正式窗口/身份单写。未释放时仅相关H等窗，不冒发布、不整包互等 |
| D-909-1 预先存在红 | native lifecycle/turnaround 维护者（909 只登记） | `turnaround_native_six_scenarios_p95_gates` 在 master 367258e7a 即败（body-edit2 hit≠lowered）；909 分支同败、无代码耦合；修复须另行路由，不随本包 |
| D-05 C-19后继 | ACE共享/生命周期路线；本包merge登记owner | 共享owned/cyclic/跨挂起/线程等未立项且不预占号；本包支持“标量共享写＋私有可变环境”，不能把C-19整行完整支持或旧AVM已替换 |
