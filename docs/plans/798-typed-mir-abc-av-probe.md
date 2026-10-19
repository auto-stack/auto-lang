---
plan_id: PLAN-798
status: executing
feature_name: Typed MIR 与 ABC/AV 执行合同及首个全路径探针
plan_revision: 2
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-20
current_step: 8
total_steps: 10
supersedes_spec_components:
  - docs/specs/auto-ac/artifact-contract.md
  - docs/specs/auto-hir/stage-contract.md
  - docs/specs/auto-test-case/architecture.md
new_spec_components:
  - docs/specs/auto-mir/project.md
  - docs/specs/auto-mir/core-i32.md
  - docs/specs/auto-av/project.md
  - docs/specs/auto-av/abc-core-i32.md
  - docs/specs/auto-av/cache-and-conformance.md
touched_goals: []
affects: [auto-mir, auto-av, auto-ac, auto-semantics, auto-test-case]
---

# [PLAN-798] Typed MIR 与 ABC/AV 执行合同及首个全路径探针

## 0. 变更摘要

承接[Design43](../design/43-ac-av-typed-mir-and-abc.md)及[后端战略](../design/strategy/auto-native-backend-evolution.md)，在独立实验workspace建立可验证的 `Checked HIR → Typed MIR → ABC → Rust AV`，以已交付core/scalar native路径和独立预期对拍。第一例即有受管cache、AutoDown用例及阶段/端到端计量；完成后为AV驱动CTE和未来AJ共用编译服务提供真实基础。

本计划交付有限执行profile及其信任/持久化合同，不把格式说明、CT模型或旧AVM调用冒称新引擎。与796/797并行的写面及依赖见[并行合同](attachments/798/parallel-contract.md)。只起草，未授权本轮实施。

## 1. 目标

1. 定义最小Typed MIR（默认有类型）的块/局部槽/调用/来源、verifier及阶段凭证；不要求SSA或性能pass。
2. 定义独立版本化ABC模块、紧凑二进制codec和验证装载，Rust AV真实执行有限i32/bool、控制流/调用及精确trap。
3. 保持短路、源码实参求值顺序、初始化、调用/返回和诊断来源；旧native作为独立后端对拍，不迁移其生产路径。
4. 从首例按函数/实例逻辑单元记录身份/依赖，真实持久MIR/ABC、验证恢复、跨进程/工程共享及有界GC；不缓存PASS或AV运行结果代替执行。
5. 接入796实际合入的语义ref和统一probe/run，正式AutoDown执行；正常局部编辑“请求→当前正确AV结果/诊断”P95<1000ms。
6. 明确AC/AV/AJ共同服务、跨层调用/代际、target/ABI/profile、可重定位原生产物和增量发布的接缝；只做合同/有限准入控制，不实现完整AJ或发布器。

范围：新 `experimental/ac-av-probe/` 独立workspace及lock、新 `test/cases/ac-av/`、本计划脚本/报告/附件；D-796/D-788后按已合入合同新增recipe/worker绑定。规范沉淀归merge，不在起草阶段写canonical Spec。

非目标：完整语言/生产默认接线、text/record/堆对象/泛型/反射执行、真实CTE、旧AVM兼容/替换、AAC/AAV、自主native后端、MIR→native新后端、native AutoABI/FFI调用、DLL loader/实际热重载、热点采样/快速或优化JIT、OSR/deopt、通用发布器/自研链接器、整个Batom引擎。未支持形态明确拒绝，不能降级到旧VM/源码解释。

## 2. 架构方案

```text
已有scalar/core源码或HIR
  → 已有bind/verify → CheckedModule
  → 本计划HIR adapter → Unchecked MIR → verify_mir → CheckedMir
  → ABC emit/encode → decode/verify_abc → LoadedModule → AV execute
                                      ↓
                           typed outcome/value/trap + trace
同一输入 → 旧Checked HIR→Cranelift→COFF→PE + 独立expected
MIR/ABC/依赖manifest → 783 ManagedStore → 恢复重验（不恢复trusted）
```

拟新workspace内两个package：`auto-av-core`（`core/`：MIR/ABC/AV/纯合同，无auto-lang/UI/man/Cranelift依赖）；`auto-av-probe`（`probe/`：消费ac-core、783 store、D-796后语义contract及测试worker）。若796合入core库本身纯净，可由core依赖其公开描述；最终cargo依赖闭包须实测。根workspace成员/旧产品manifest不改。

MIR选用显式基本块+有类型的可变局部槽/临时槽+terminator，非SSA；ABC采用有类型槽的指令与函数/块索引，装载时形成紧凑表。编码宽度/表字段/版本和opcode编号由T-01冻结有限v1并保留未知版本拒绝，不预先承诺公开永久字节码ABI。Atom+Schema是逻辑/调试视图，wire为自有紧凑格式+manifest；不宣称未实现Batom可零拷贝执行。

D-796以前adapter只把已有HIR本地类型/声明映射为本计划owner作用域ref，不另造权威推导器；796类型codec更广不扩大本AV的i32/bool执行面。最终绑定真实语义描述，依旧由已有HIR verifier授源执行信任，796不自动替代该前门。

## 3. 技术栈

Rust独立workspace、锁定依赖；复用ac-core公开API、783 `ManagedStore`/identity/root/lease/GC，native参考复用 `native::lower_object`、`link::link_object_staged`/contained执行器。Windows x86_64-pc-windows-msvc是native对拍/周转参考平台；AV标量语义不依赖宿主位宽，不声称已验证全部OS。禁止新造全局数据库或未受控子进程启动器。

拟新文件：`core/src/{contract,mir,mir_verify,abc,abc_verify,av}.rs`、`probe/src/{hir_adapter,cache,driver,native_reference,observations}.rs`及受限worker。实际名称可等价调整但保留验证边界。未来入口 `scripts/verify-ac-av-798.ps1 -Scope admission|semantics|cache|resources|turnaround|binding|autodown|acceptance|closeout` 由T-01/T-08创建；下文新命令/测试族均未存在、未执行。

拟议探针策略（非用户给定值、非语言永久语法上限）：单source/HIR/ABC输入≤4MiB，单次codec页≤256KiB；条目预算为**每持久/准入工件的分配上界**（r2 修订，替代 r1 的"全部表合计"字面读法）：模块级表（类型+常量+导入+函数计数）≤65536 且**每个函数单元**（槽+块+指令）≤65536，均按实际分配前检查；整模块/闭包**组合上界**由请求级预算约束——单输入≤4MiB、MIR/ABC验证每请求≤200万工作步且wall≤5s。修订动因：r1 冻结负载（256KiB/100 函数，§5.5 同为冻结项不得缩小）整模块展开 72k 条目，与"合计≤65536"不可同时成立；按工件族划分后单表分配守卫不变，组合规模仍有显式上界（步数/墙钟/输入），不构成无界放宽。旧source/HIR文本前端探针递归预算为128层（含括号/块及连续表达式构造路径，按现有token流作有界前置准入，不能只数花括号），超出在旧递归parser前报resource；不改变旧独立CLI支持面。MIR/ABC验证每请求≤200万工作步、wall≤5s；AV每执行≤100万指令步、≤1024帧、活槽数据≤8MiB、wall≤2s，取消每≤1024指令检查。大于表预算是resource拒绝，不是非法语法/unknown opcode。条目计数和指令fuel定义在T-01钉死，不能通过不计验证/分配规避。

总受管热点/暂存/VM工作集≤32MiB（代码/数据热点8MiB、解码/索引/编译暂存16MiB、执行帧/槽8MiB），worker private-bytes相对空载增量目标≤128MiB；外部native构建/工具进程单报，不冒称受管预算覆盖其内存。请求/响应元数据各≤4MiB、stdout/stderr各≤1MiB、测试trace≤64KiB；溢出报resource不截断成成功。

native参考每请求build≤60s/run≤5s/total≤70s，沿788合入的有界bytes/Job清理；D-788前只能用已有contained能力并在隔离wrapper补齐本计划所需cap，失败就阻断该用例，不能裸跑。反例/资源极限不得成为OS栈溢出或宿主panic，进程timeout不冒充语言trap。

## 4. 需求分析与背景调查

授权：用户明确要求起草本计划、与796/797并行、首例cache/AutoDown/亚秒指标、暂不实现完整AJ或发布器。继承本仓设计/Plan/worktree规约；仅在auto-lang主检出写设计取材及计划簿记，不创建实施树/代理、不安装工具、不联系其他会话或开始编码。未指定实现人/模型/时间预算。

事实源：[overview](../specs/overview.md)、[HIR现状](../specs/auto-hir/project.md)、[阶段合同](../specs/auto-hir/stage-contract.md)、[产物合同](../specs/auto-ac/artifact-contract.md)、[cache](../specs/auto-cache/project.md)、[TestCase](../specs/auto-test-case/project.md)。起始调查HEAD `3e5aa91f65eca89dab97d652f3782c72c5435526`，并行合入后来源采集HEAD为`f59de89d0d5480c52faaac0f74ba8a9e30b3c5cb`；源码与工作区设计SHA-256见[planning-inputs.tsv](attachments/798/planning-inputs.tsv)。Design40–43是目标态，尚无auto-mir/auto-av当前Spec；它们列为SD新增。788的profile-v2 Spec尚未合入/不存在，不能以计划文本当实现。

| 实际取材入口 | 已观察事实与采用边界 |
|---|---|
| ac-core/src/hir.rs、verify.rs | Bundle/Body表式引用、Expr/Stmt/Loop、core/scalar分域、私有CheckedModule；可只读消费，不扩旧profile |
| ac-core/src/source/{mod,adapter}.rs | compile_scalar_source/compile_parts_with等已有前端；不重写lexer/parser/infer |
| ac-core/src/native.rs、link.rs | 原生COFF→PE参照；当前旧API无788全部bytes/cap能力，不继承虚假安全保证 |
| ac-core/tests/scalar_native.rs、fixtures/scalar-source/valid/{p-binop-args,p-gcd-euclid}.at | 现有命名实参、嵌套调用/循环标量语料和真实PE；可引用对拍，不搬成另套expected |
| ac-artifact-probe/src/{schema,identity,codec}.rs、store/{mod,gc}.rs | A08仍Future，已有通用存储/页容器；put/get及ChunkKind不自动验证新payload语义 |
| auto-test-case/contract.rs、auto-case CLI | 当前仅v1 avm/run和a2r/emit；新AV不能冒充旧avm adapter |
| 796 r1、797 r1、788 r1 | 796拥有语义ref/公共probe接缝；797原生ABI；788 v2及执行观测在途，本计划不抢写 |

独占取号：mutex及`docs/plans/.allocation.lock`跨扫描/调用`new-plan.sh`/唯一性检查，active/archive最大797→领取798、`.next-id=799`。Git Bash受限启动失败后，获准仅运行取号脚本；无其他实现。来源包含此前已获授权的未提交设计更新；work必须消费包含相同设计版本的提交，重新记录依赖实际HEAD/hash。

依赖门：D-796=796独立review/冻结门禁/实际合入/Spec与merge收据齐全；D-788同理。T-01–05使用已合入ac-core/783独立开展；T-06绑定D-796，T-07使用其受限probe/run（796本身依赖D-788）。797不是执行或合入前置；不以假native ABI调用补“完整性”。没有796↔798反向依赖。

## 5. 详细设计

### 5.1 最小支持面与T-01全路径可行性门

支持已合入core/scalar中如下有限执行面：i32/bool常量、局部let/var及赋值/遮蔽、精确加减乘除余/比较/同类型相等、一元负/非、lazy and/or、if/else、loop/break/continue及scalar条件for的既有HIR展开、显式签名的直接函数调用（位置/命名绑定、参数/返回）、有fuel保护的递归。语义含i32溢出trap、除0、MIN/-1及MIN%-1 trap，与现有scalar合同一致。bool非任意整数，调用实参按源码eval_args先求值再映射参数，不按bindings重排。

仅允许已批准测试能力 `hir.test.trace.v1` 中mark_a/mark_b内建（签名按已合入HIR声明核验）；映射为AV测试trace事件用于观察顺序，不授普通IO/FFI。其他intrinsic/宿主import、text/record/drop/move/借用、unit/f64/opaque执行均明确unsupported；无trace能力时含mark的模块在执行前拒绝，回调/资源/Meta类别在合同矩阵标future。未来扩面加新profile，不扩大旧profile。

T-01在上述范围先做一个两函数算术/分支的最小vertical：真实Checked HIR→MIR→ABC→AV，与native/固定expected一致；首份MIR/ABC即通过真实store写入并在新进程恢复重验执行，加入AutoDown源/oracle（正式runner pending）和实际span。输出 `contract-decision.md` 冻结逻辑Schema、wire编号/布局、内部调用/返回、entry、预算/计步、资源分类、合法opaque页容器与参考负载。其余T-02–05前须通过这道whole-path门，不能先造完整引擎再证明存储/API不可行。不能成立则记录具体反例回规划，不静默改为HIR解释或弱化cache/预算。

### 5.2 MIR、ABC与消费信任

MIR有模块/函数owner、类型和常量ref、typed slots、顺序指令、唯一terminator的基本块、分支/跳转/call/return及来源映射。读写槽和初始化/合流规则显式；循环按保守可终止worklist校验，成功不得含未初始化read、错类型call/return或跨owner local引用。基本块/槽编号是局部身份，不是稳定跨修订ID。私有CheckedMir只能由verify_mir产生，改写后旧凭证失效。

ABC保存schema/revision/profile/语义版本、模块/函数/常量/类型表、指令/块目标及逻辑imports、entry/能力和来源sidecar。无native地址、Arc、旧Value内存或MIR Rust enum的裸dump。明确字节序、长度/offset的checked算术、布尔合法值和未知版本/opcode拒绝；先检查shape/范围/配额，再检查控制流、初始化和签名。结构/类型检查覆盖全部表及不可达块；初始化按入口可达CFG的保守合流规则验证，不把不可达代码当作任意未知指令的藏身处。只有verify_abc产生可执行LoadedModule（或等价私有句柄），AV不接受未验证bytes；独立ABC输入不因“由本编译器生成”跳过验证。

cache恢复后重新解码及两层适用验证：MIR→CheckedMir、ABC→LoadedModule。不恢复trusted标记；恶意但自洽frame/正确hash的非法内层不能执行。恢复路径检查owner/profile/实际语义版本及当前能力，不能仅验证外壳。AV使用显式帧/槽栈和分派循环，不用Rust递归承载guest递归。

来源映射保留HIR span/函数/当前snapshot；位置变更可以共享不含位置的代码内容，但更新source sidecar/诊断绑定。成功值/语言trap、unsupported、invalid MIR/ABC、取消/resource/infra分态；普通未执行分支的trap保持潜在runtime行为，不被lowering/验证提前求值成编译错误。

### 5.3 AC/AV/AJ合同与有限native参照

最小共同编译服务输入是Checked MIR/profile与实际依赖，输出产物/诊断及计量，AV消费ABC；AJ entry仅合同future，不实现成功空adapter。逻辑签名、执行表示/内部调用、原生AutoABI和平台C ABI独立；为未来adapter、能力/错误/资源及代际lease定义描述/unsupported准入，不把内部槽编号定义成797 ABI。

逻辑UnitRef=(逻辑模块/声明/实例owner, schema/profile)，Revision为body/interface内容指纹；临时HIR索引不能当全局ID。内容相同不合并不同声明，绝对路径不进入跨工程内容身份。MIR/ABC按函数单元及模块链接manifest保存；不优化内联、不做跨模块链接，直接call依赖callee签名和执行manifest映射，callee body变重编其单元/重装执行闭包，不无条件使caller代码失效。未来内联/推测读body/layout则额外记录依赖，禁止统一仅用接口hash的未来承诺。

generation/ExecutionProfile/semantic版本与target/layout/ABI描述区分。只有完整一致模块闭包安装才可执行；用有限准入模型测试错generation/profile/未知能力拒绝，不宣称已实现在线热重载/跨层栈切换。T-01定义native产物ref必须具备可重定位代码/数据、符号、target/ABI/runtime、recipe等，探针复用现有COFF及其manifest作正例，拒绝把模拟进程地址当portable产物。只验证引用/准入，不增加native emitter，不做发布闭包/链接器重写。

native对拍沿旧HIR→Cranelift，不由新MIR/ABC生成expected；观察真实PE退出/trace及自检断言。同一输入同时核独立expected和两后端，不只比两者同错。原生main正常0/失败分支非0、trap=现有测试退出70，AV用结构化结果另比较，host超时/panic不能映射为trap70。全i32值可由独立常量/断言验证，不能只以exit低8位掩盖结果。

### 5.4 首例cache、引用与恢复

使用783真实ManagedStore的独立producer namespace，以本计划自有LayerSpec/内层envelope描述A08 MIR与A09 ABC/manifest，不修改783/787底层registry/ChunkKind或把Future登记冒称已实现。若复用通用LayoutData等容器，标明它只是carrier，写入/读取都校验producer、schema、owner及阶段；不得伪标为HIR/Object。T-01证实该方式合法；无法保证时回规划，不私建SQLite/绕过公开API。

key至少区分owner/profile、当前body和签名、实际callee签名/类型/常量、算法/Schema、MIR/ABC/AV语义版本、能力/资源影响语义的配置；source映射分轨。纯portable MIR/ABC不因无关native CPU改变而全失效；native参考对象key另含target/CPU/ABI/backend/flags。producer版本变化失效适用层。能力变化至少重走装载准入，不从持久“已授权”字段继承权限。

测试fresh、同进程hot、重启磁盘热、跨工程相同逻辑输入共享、单body/签名/依赖/profile/语义版本变更。正常cache hit必须仍运行AV并重新比较oracle，计数语义（r2 收紧）：`mir_lower_modules`=整模块 lowering 的**真实调用次数**（任一 miss 触发即为 1，不按单元数冒领）；`mir_units_persisted`/`mir_units_restored`=MIR 单元真实持久化/恢复重验（restore 须重入 verify_mir，不得以 ABC 恢复或共用 Rust 结构冒充）；`abc_units_emitted`=单元 emit 次数；`closure_encodes`=闭包装配后的整闭包 codec 次数；`av_execute`=执行次数。命中声明须配**禁调反控**（producer 未被调用才是 0）；分别计mir_build/abc_emit/verify/load/av_execute 的 r1 表述以上述计数为准。CTE结果缓存留后续计划，不能将av_execute=0当本执行测试命中目标。

数据强引用是执行所需类型/常量/函数单元和模块manifest；失效/来源ref不永久pin旧HIR/AST。实际执行中lease保活数据，不将活帧存盘。良好frame替换错地址/错owner/key、非法内层、缺块必须在真实消费入口拒绝或受控miss重建；没有再生输入报unavailable，不读Nil/0或旧PASS。冲突/取消/写失败保留旧已提交结果并释放暂存/lease。

### 5.5 资源、预算及可观测周转

从T-01 trace请求、已有前端/verify、依赖验证、cache读取/解码、MIR lowering/verify、ABC emit/verify/load、AV执行/结果比较和发布观察；包含IPC/排队/IO/取消，嵌套span不能重复相加。native编译/链接/运行单独trace用于对拍，不混入AV执行微基准后称完整native周转。

T-05使用至少32MiB真实MIR/ABC产物（多个各≤4MiB模块，流式生成）、随机只读一个函数/模块和512次局部修订+GC（r2 收紧：每轮**真实编辑输入**——替换目标必须在当轮源码中命中且输入 hash 变化，逐轮断言新独立 expected 与实际重编单元数；不得出现零变更循环）；GC/水位断言作用于**承载冷产物的同一store**；32MiB**受管分区**预算（热点8/暂存16/执行8）与 worker private-bytes 相对空载增量≤128MiB 须有实际计量（不得以持久文件大小冒充运行工作集），783物理水位收敛（无live超额时继承64/48MiB策略）在压力 store 上取回收前后证据；强引用及活lease保活，用户哨兵不变。新建小scratch不扫用户cache。

新MIR/ABC的4096块直链及混合CFG在预算内遍历/编码/验证/释放；旧source/HIR路径的连续/交错控制/表达式嵌套在128预算边界两侧分别完成或前置resource拒绝，另用4096层输入证明拒绝发生在旧递归parser之前。T-01复用`source::lexer::tokenize`及`atom_text::tokenize`核其真实递归风险和前门覆盖，不另造lexer/parser；若无法安全覆盖所有表达式递归路径则该可行性门不通过，回规划而非靠扩大线程栈或崩溃转译。验证坏ref/表达式引用环反控，CFG合法循环不能一概当非法cycle。成功或明确resource拒绝均依先冻结预算决定，不能把宿主stack overflow算资源状态。1024帧guest递归预算与HIR深度不同，执行链在边界两侧分别证明完成或resource；死循环靠真实fuel/取消终止，trace/IPC过量同样有边界控制。

T-01固定参考Windows机器/工具、256KiB core/scalar输入（由100个有意义函数body组成，非纯空白填充）、局部编辑位置及独立expected；key规模/负载不能随结果缩小。六场景各≥30样本（r2 语义收紧，防"标签与条件不符"）：**fresh**=每样本独立冷store（真实冷态，不得共用已热根）；**同进程hot**=同进程同store重复请求；**restart磁盘热**=每样本**新进程**（子进程入口）冷内存热磁盘，跨工程样本交替项目标签；**单body编辑**=每样本产生**当轮新修订**（逐样本不同的编辑内容与随之变化的独立expected），不得重复同一源码；**callee签名变更**=校验**目标诊断 stage+code**（frontend 绑定诊断），不接受任意 diag/resource/infra 错误；**GC并发**=GC与请求**真实并发**（独立GC进程/线程与请求重叠），不得串行完成后计量。每样本绑定输入 hash 与独立 expected，值/诊断全量校验。报告P50/P95/P99、真实重算/命中、I/O/内存、全部失败样本（不得删失败样本）。

**硬门**（r2 强制接线语义）：warm session的单函数body编辑，或造成调用签名冲突的编辑，从实际请求入队至当前正确AV结果/诊断发布P95<1000ms；**正式入口必须执行该门**——worker 以 gate 阈值断言两路径 P95 与全部样本成功并以非零退出码传递失败，`--gate-ms 0` 故障注入必须失败（反控）；脚本透传退出码并生成真实汇总结构（非 raw 复写）；含当次必要前端/verify/依赖/cache/MIR/ABC/AV及比较。前端可整模块重查，但如实计量，不谎称前端细粒度增量已完成。native参考冷构建、全冷/广泛失效、超过预算的压力另报；不可扣实际IPC/等待，不能只加阶段P95。T-07正式AutoDown接入后按同负载重测；不能以Rust定向结果代替runner端到端。

### 5.6 796/797绑定与AutoDown

D-796后只读消费其真实ref/codec公开库，将本profile的i32/bool/函数及依赖映射到其owner/版本，测试同名异owner、重启intern排序变化、schema/类型版本错配；删除早期临时语义ref快照adapter（HIR适配器仍需保留）。不复制796推导/CT库，不调用其模型producer当AV。若真实接口不够，用具体字段/用例回规划或由796 owner显式修订，不在其文件暗加字段。

AutoDown源/oracle从T-01进入 `test/cases/ac-av/*.case.ad` 和suite；D-796/D-788前清楚记录execution pending，同时由Rust定向测试实际跑vertical。T-07通过796合入的registered `probe/run`接缝增加独立recipe/worker（拟 `ac-av-core-run`、`ac-av-admission`），只消费声明式Atom参数或已有source引用，不新建adapter名伪装已存在功能，不重写共享decoder/mod/CLI。worker不读取expected决定结果，父runner校验actual来源/工具身份/产物bytes/状态并比较oracle。

正常、trap、诊断、cache/restart/编辑/资源负例由正式runner实际选择/执行；未知recipe/字段、错worker/profile和坏actual+好expected必须gate红。required缺例/skip/缺native工具/timeout不得绿。原v1 avm/run、a2r/emit及788 native/syntax、796语义recipe和已合入797 recipe兼容回归；没有实际合入的797不能伪造其兼容证据。晚绑定如需扩公共IPC，先形成有owner的显式revision，不与其他计划并发改协议。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-mir/project.md、core-i32.md | 无实现→有限Typed MIR、slot/CFG/初始化和Checked门 | 正确性共同层 | AC-01/02 |
| SD-02 | add | docs/specs/auto-av/project.md、abc-core-i32.md | 无新版AV→有限ABC wire/验证/装载/执行及unsupported | 不混旧AVM和新引擎 | AC-01/03/04 |
| SD-03 | add | docs/specs/auto-av/cache-and-conformance.md | 无→MIR/ABC producer身份、恢复/依赖/root/lease/资源/计量 | 跨工程复用可信 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-ac/artifact-contract.md | 783 A08 Future不改历史→追加798 producer的实际实现范围与carrier/引用/准入 | 不给旧producer冒领覆盖 | AC-05/08 |
| SD-05 | modify | docs/specs/auto-hir/stage-contract.md | 旧S0–S6保留→追加HIR到新MIR的消费前门/资源支持表 | 不扩大旧Checked保证 | AC-02/08 |
| SD-06 | modify | docs/specs/auto-test-case/architecture.md | 在D-796后probe/run支持表追加本recipe、worker身份/失败/观测 | 使用正式runner | AC-07 |
| SD-07 | add | docs/specs/auto-av/cache-and-conformance.md | AC/AV/AJ/profile/native产物引用为有限合同；完整AJ/发布仍future | 不把模型当运行能力 | AC-08 |

Spec导航/ledger只在merge按实际增量登记；796语义及797ABI的权威Spec由各自owner维护，本计划只引用，不回写其未合入文件。

## 6. 测试设计

现有实际命令/fixture为取材；拟新测试族由对应T创建：`mir_admission_`、`mir_lower_`、`abc_codec_`、`abc_admission_`、`av_semantics_`、`av_cache_`、`av_resources_`、`av_turnaround_`、`av_binding_`、`av_boundary_`。运行示例 `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-core mir_admission_` 及脚本对应Scope；包名/入口在T-01落盘前不算证据。

| Case ID | AC/invariant | 输入或变更 | owner/entry | 可观察oracle/区分性断言 | 边界 |
|---|---|---|---|---|---|
| V-01 | AC-01 私有阶段信任 | 真HIR/合法MIR/ABC、未知schema/profile/请求trusted | T-01/02/03、真实verify_mir/verify_abc/AV | 合法执行；伪trusted不能构造句柄，AV dispatch计数0 | 每个消费入口含恢复 |
| V-02 | AC-02 类型/初始化/CFG | 同类型越owner ref、悬空目标、无terminator、合流一支未初始化、循环 | T-02、mir_admission_ | 指定阶段/代码拒绝，合法循环/合流不误杀；变异对象哈希证明实际改动 | 不用另一类型错误替代owner证明 |
| V-03 | AC-02/03 数值/短路 | MAX+1、MIN负、除0、MIN/-1及MIN%-1；未走RHS含trap | T-02/03、av_semantics_及native参考 | 结构化trap/PE70和独立预期；未走RHS正常、trace顺序一致 | 不把常量trap提前为编译错误 |
| V-04 | AC-03 参数/控制/递归 | p-binop-args、p-gcd-euclid、命名参数含mark/嵌套调用、合法递归 | T-02/03、av_semantics_ | 独立值/trace+真实PE；参数调换/顺序错/漏循环必红 | native不经新MIR自证 |
| V-05 | AC-04 wire/恢复 | 截断、长度溢出、非法bool/opcode、错槽/签名、坏frame好内层及好frame坏内层 | T-03/04、abc_admission_/真实load | 有界拒绝，不panic、不执行；正常新进程恢复仍执行正确 | 不仅测frame自检 |
| V-06 | AC-05 真缓存 | fresh→hot→重启/跨工程；producer禁调、两次运行输出观察 | T-01/04、av_cache_ | 命中mir_build/abc_emit为0且av_execute>0、每次独立比较 | 不缓存PASS或运行结果 |
| V-07 | AC-05 依赖/配置 | 只改body、callee签名/执行body、布局无关CPU、profile/AV语义/producer变更 | T-04、av_cache_ | 受影响单元重算，未读body的caller代码可复用；执行manifest更新、结果=无cache | portable/native键区分 |
| V-08 | AC-05 错身份/并发/回收 | 自洽异址frame、错owner/key、同键异内容、GC与lease、写失败 | T-04/05、av_cache_ | 真消费拒绝或miss；冲突不覆盖旧提交、lease期间保活、哨兵不变 | 含失败释放/再生缺失unavailable |
| V-09 | AC-06 规模/内存 | ≥32MiB冷产物、512churn、4096块CFG、文本嵌套128边界/4096前拒及坏ref | T-05、av_resources_ | 真字节/OS峰值、32MiB受管和水位收敛；全路径无宿主栈溢出 | 形状/步数和资源边界先冻结 |
| V-10 | AC-04/06 有界执行 | 无限循环、递归1024帧两侧、燃料/取消/trace边界 | T-03/05、AV实际执行入口 | 完成或预定resource/cancel；trap/timeout/infra分开，释放帧/lease | 不用宿主递归/进程杀死冒充fuel |
| V-11 | AC-06 亚秒 | 六场景≥30样本、局部body和签名错编辑 | T-05/07、av_turnaround_/正式runner | raw trace/结果绑定当前输入，硬门两路径P95<1000ms，不删失败样本 | 工具准备/全冷/native分列 |
| V-12 | AC-07 AutoDown真实执行 | 本recipe正/负、未知recipe/字段、错误actual且expected正确 | T-07、auto-case run及实际worker | 真实dispatch、actual身份、gate0/1/2符合合同；篡改必红，旧支持面不变 | D-796/D-788后才执行 |
| V-13 | AC-08 绑定及并行 | 796同名异owner/版本、core依赖闭包、797未合入、不支持native地址产物/代际 | T-06/08、av_binding_/av_boundary_ | 真codec校验；临时模型退出，无旧VM/产品依赖和伪AJ安装；未知代际拒绝 | 仅合同模型不称热重载 |

以上V-01–13均为必验补充，不以一个固定语料族的通过总数替代。实现时逐例绑定实际命令/输入/commit/依赖hash/raw结果；若预期与现有Spec冲突，先查明并修订，不悄改oracle。native工具缺失是infra阻断，不能用AV两次执行代替对拍。

实施/自检由后续work负责；最终review用同一树、独立上下文、冻结commit和依赖，暂停实施写入；未指定人/模型，不创建代理。预定唯一树 `D:/autostack/.wt/lang-798/auto-lang`、分支 `plan-798-dev`，本轮未创建；书面进度在主检出。

work仅定向模块/上述验收和必要cargo check，修复完再送审；不触发整档t/tv/tt/tb/tu/taa。最终review pass须记merge-owned pending。merge在冻结提交跑新独立workspace完整套件/真实native对拍、受影响796/788/已合入797的定向兼容；若只新增recipe文件且零根Rust变化，不机械跑根cargo t。若实际改根测试编排，按最终diff追加根裸cargo t及VM/转译触面tv/tt，禁止假称“独立crate”免掉真实触面。无旧aavm/UI/生成器/语法参考改动不跑taa/tu/docs_gen；tf仅到期在主检出单实例批量执行。

## 7. 验收标准

| ID | owner_stage | 结果与验证 |
|---|---|---|
| AC-01 | work | 有限profile/Schema/版本/支持表及T-01真实最小vertical含首例store/新进程恢复/AutoDown源/trace；V-01/06 |
| AC-02 | work | HIR→MIR真实lowering与独立verifier，控制/求值/类型/初始化正确；V-02–04 |
| AC-03 | work | ABC codec/验证与Rust AV实际执行，全标量语义/顺序与native及独立expected一致；V-03–05 |
| AC-04 | work | 无trusted绕过，坏输入/未知能力拒绝，fuel/帧/取消有效且分态；V-01/05/10 |
| AC-05 | work | 真单元cache/依赖/恢复/跨工程及坏地址/冲突/GC正确，命中仍执行；V-06–08 |
| AC-06 | work | 资源/规模和六场景计量、两路径含正式adapter的P95<1s，原始样本可复验；V-09–11 |
| AC-07 | work | D-796/D-788后正式AutoDown真实dispatch/比较/反控及旧协议兼容，无required gap；V-12 |
| AC-08 | work | 真796合同绑定、临时快照退出、AJ/native/797边界、SD/来源/coverage表完整；V-13 |
| AC-09 | review | 同树独立审查逐AC/V/SD并绑定revision，所有in-contract finding闭合；merge门禁pending明示 |
| AC-10 | merge | 冻结门禁、Spec/ledger沉淀、归档合入/实际计数/guard清理齐全；最终10/10 |

## 8. 执行步骤
**当前独立复审（2026-10-20，`36108624a`）**：needs_fix，开放 F798-R7-01..05；旧8项中的错键、错型参数、原解码放大、警告/缺oracle已有实证修复，其余完整不变量仍有缺口，并发现合法双分支return被lower拒绝。[报告/闭合矩阵/原始反例](attachments/798/review-r7/review.md)。保留T01最小vertical、T03核心codec/执行门、T06语义绑定；重开T02/04/05/07/08及相关完整验收，T09/10未完成，current_step=3/10。下表旧work完成说明属于历史，不代表当前最终验收通过。

| Task | owner_stage | 前置 | 文件/产物与可验成果 | 验证及AC |
|---|---|---|---|---|
| [x] T-01 | work | 用户实施指令 | 新workspace/lock、contract-decision.md、最小两函数vertical/MIR/ABC/Rust AV/store恢复、AutoDown源/oracle和计量首例；冻结§3/5.1策略 | [✅ r2 已闭合 31628c5d7：contract-decision §6 升格 r2 正式合同（三反控 mir_admission_ 单函数/模块表/组合步数全绿）；Cargo.lock 已 -f 入库；两 package 零新增警告+fmt 过检。F02/06/07 闭合。历史收据] ce860fb24/fee55cd2d：`experimental/ac-av-probe/`（core+probe 独立 workspace）+ `contract-decision.md`（冻结 Schema/wire/预算/carrier/负载，含 r1.1 每表族预算修订）；vertical 三测绿（AV=独立期望、命中跳产仍执行、真实 PE 对拍）；store 首写+新进程 worker 恢复重验执行（`av_cache_worker_restore_new_process`）；AutoDown 源/oracle `test/cases/ac-av/`（5 用例）+ Rust 定向绑定 5 测绿；AC-01 |
| [ ] T-02 | work | T-01 | core/mir与mir_verify、HIR adapter、scalar控制/调用/来源，补V-02–04 | [R3重开→r4 已闭合：F798-R3-01（owner 核对/类型表 ==len/import allowlist+单元 fn: 链接/能力 allowlist）+F798-R3-04（前门平坦链 128 构造深度）+F798-R3-03（span 重绑定）。永久测试：mir_admission_foreign_function_owner_rejected/_type_ref_equal_len_rejected/_unknown_capability_rejected/_unknown_import_rejected/_unit_fn_linkage_import_allowed、av_resources_flat_chain_front_door_boundary] [✅ 已完成] `core/src/{mir,mir_verify}.rs`+`probe/src/hir_adapter.rs`；mir_admission_/mir_lower_ 10 测绿：变异矩阵（悬空目标/越界槽/非法分支类型+哈希证明实际改动）、半初始化合流拒绝、合法循环/合流不误杀、landed p-binop-args/p-gcd-euclid 降级验证、迭代语句栈（200 连续 if）、text 域 unsupported 拒绝；AC-02 |
| [x] T-03 | work | T-02 | ABC codec/verifier/LoadedModule、AV显式帧/槽及全有限profile、预算/取消/测试能力 | [R3重开→r4 已闭合：F798-R3-01（verify_abc 复核手工 DecodedAbc 版本/profile）+F798-R3-04（根帧/调用帧预算分配前生效+peak_slot_bytes 全退出真值）。永久测试：abc 手工版本反例（core lib）、av_zero_frame_budget_rejects_root_frame/_zero_slot_budget/_success_reports_real_peak_slot_bytes] [✅ 已完成] `core/src/{abc,abc_verify,av}.rs`；abc_codec_/abc_admission_/av_semantics_ 13+1 测绿：字节稳定往返/截断/尾随/校验和/未知版本-opcode（手工字节流）/bool 合法性；仅 verify_abc 产生可执行句柄、篡改内层必拒；trap 全矩阵（MAX+1/MIN负/除0/MIN÷-1/MIN%-1）结构化分态；未走 RHS trap 潜伏；trace 顺序+能力门（无能力执行前拒绝）；命名实参源序求值；fib(15) 燃料递归；燃料/取消/1024帧两侧；wire 携带 capabilities；AC-03/04 |
| [ ] T-04 | work | T-03 | probe/cache、函数单元/依赖manifest/恢复准入及root/lease，错身份/版本/并发反控 | [R3重开→r4 已闭合：F798-R3-02（UnitLink payload_fp+SemaRefSer 信封、manifest fingerprint 全字段、恢复侧 profile/av_sem/sig/logical/payload/信封/当前 caps 七轴准入、driver manifest 轮实调 admit_unit_ref）+F798-R3-03（span 重绑定级联+sidecar 写失败传播）。永久测试：av_cache_restore_identity_negative_controls（七轴）、av_cache_restore_requires_current_trace_grant、av_cache_position_drift_keeps_trap_span、binding_tier UnknownPayload 负控] [✅ r2 已闭合 31628c5d7：A08 MIR 单元真持久化+恢复重入 verify_mir（driver 两层查找 ABC→MIR→lower）；计数分离落地（mir_lower_modules=整模块真实调用/mir_units_persisted/restored/abc_units_emitted/closure_encodes/peak_staging_bytes），cache_tier/vertical 断言改钉新计数。F04/06 的缓存腿闭合。历史收据] `probe/src/{cache,worker_restore}.rs`+bin；av_cache_ 7 测绿：fresh→hot→restart/跨工程共享（命中 mir_build=0/abc_emit=0 且 av_execute>0 每次重比较）；body-only 单单元重键+caller 复用、签名变更全 caller 重键+正确 frontend 诊断；错 producer/schema/stage 信封受控拒绝（非 miss 非覆盖）；坏帧坏内层拒绝；同键异内容 Conflict 且旧提交原样存活；lease 保活过 GC、stale sweep；跨进程 worker 恢复（manifest→artifact id→重验→执行，值一致）；AC-05 |
| [ ] T-05 | work | T-04 | 规模/预算/GC/深链、六场景trace与周转脚本/raw报告 | [R3重开→r4 已闭合：F798-R3-08（heap_estimate 真实台账替代 wire 冒充+init 位图分配前 16MiB 守卫 av.resource.verify-memory+decode 放大守卫，dense 24581 条目反例分配前拒绝；组合上界反控内存精简形态重钉）+F798-R3-04（平坦链前门）。永久测试：mir_verify_dense_analysis_memory_pre_rejected/_analysis_bytes_recorded_on_credential、mir_admission_combination_bound_is_verify_steps（r4 形态）。bench 30 样本 gates []（edit P95 418ms）] [R2复审重开 F798-R2-01/02 → r2.1 已闭合 d72269f54：脚本 benchOut 空管道截断移除+成功/失败双路汇总可解析校验（实测）；峰值采样器（5ms 轮询）+request/gc 子进程自采样折入+前端隔离归因（worker 215.7MB 中前端 211.8MB≈98%，target_met=false 如实上报+债务路由 KNOWN-DEBT）；受管分区硬门 staging≤16/hot≤8/exec≤8MiB 逐样本入 validate_and_gate，staging 计数后移至闭包 codec 完成后，AV peak_slot_bytes 高水位；瞬态峰值区分性反控测试。历史收据] [✅ r2 已闭合 31628c5d7：六场景 r2 语义重做（fresh 每样本冷 store/restart 每样本新进程 request 子命令/edit 每样本当轮新修订 30 不同 hash/签名冲突仅认 frontend:source.*/gc-loop 进程真并发）；churn 512 轮真实编辑（逐轮 hash 变化+独立期望+重键数断言）+同 store 水位回收+哨兵存活+受管暂存峰值断言；worker private-bytes 增量 8.6MiB 计入 gate；硬门真接线（gate-ms 0 反控 exit 5；正式跑 PASS：edit P95 506.5ms/冲突 21.3ms/零失败，docs/reports/plan798 r2 版）。F03/04/05 闭合。历史收据] av_resources_/av_turnaround_ 6+6 测绿：4096 直链+菱形+合法循环预算内验证（修 block_base O(n²)：31k 块 verify 419ms→11ms release）；超表预算=resource 分类；128 嵌套两侧+4096 前拒（door 在旧递归 parser 前、resource 态）；atom `[` 深度前拒；33.5MiB 冷 ABC 产物（64 模块分页合规）随机只读重验+512 修订+GC 水位有界；六场景×30 release 样本+**硬门 PASS**（body-edit P95 829.4ms/签名冲突 41.4ms/零失败，raw 180 行可复验 `docs/reports/plan798/turnaround-{raw.jsonl,summary.json,md}`+`scripts/verify-ac-av-798.ps1`）；AC-06（正式 runner 重测留 T-07） |
| [x] T-06 | work | T-05、D-796 | 绑定合入语义ref/codec，退临时语义快照，冻结消费hash | [R3重开→r4 已闭合：F798-R3-02（admit_unit_ref 增 payload 绑定参数并在 driver manifest 轮实调；restore 侧 admit_restored_unit 字节级比对）+R3 反例重放闭合。binding_tier 4 测绿+UnknownPayload 负控] [✅ 已完成] b9682967f（rebase master 13367d127 后）：D-796 门满足（796 归档合入+Spec/收据齐全）；只读消费 ac-sema-contract（`probe/src/semrefs.rs`：logical_id/DeclRef/ContentVersion/SemaRef::admit）；驱动内容轴从 FNV 快照切换到 796 ContentVersion 域（canonical dump 同源）；manifest 单元携带 logical ids；av_binding_ 4 测绿（同名异 owner 三作用域分离+缓存不串、重启序稳定+跨进程命中、schema/owner/tamper/内容版本/种类轴拒绝、消费面钉扎）；未触碰 796 推导/CT 库；AC-08 |
| [ ] T-07 | work | T-06、D-788 | 基于796已合入probe/run仅追加本recipe/worker/suite，正式AutoDown语义/cache/资源/周转与负控/兼容 | [R3重开→r4 已闭合：F798-R3-05（exit 码/恰一协议行/步骤回显/父侧 fnv source_hash 逐 run 绑定）+F798-R3-06（call_worker 受限执行器重写：writer 线程/帽定读者/单一绝对截止/杀树/READER_GRACE/BUILD_WALL 有界 cargo）+F798-R3-07（两调用显式 session 模型，窗口=受测 [edit,run] 调用 request_wall_ms 连续含 IPC，30 样本编辑窗 P95≈600-640ms<1000ms，前端 290ms 如实计入；decode 合同 r4：前导 edit 放行+须 run 结尾+av.session 绑定）+F798-R3-10（locate_worker 恒构建绑定二进制）。永久测试：ac_av_probe_worker_contract_faults_are_infra（exit9/extra-frame/wrong-hash/delay 四故障全 Infra）、formal gates（hits/misses 形状+当前修订值断言）；5 测全绿] [✅ 已完成] b9682967f：D-788 门满足（788 归档合入）；recipe `ac-av/core-run-v1` 注册（配方专属步骤词表 run/edit-source+av 绑定+probe exit 通道/run 完成态）；共享侧 `ac_av.rs`（Job 收容+resume/envelope+nonce+hash+recipe+双版本镜像准入/stale 按配方族路由/父侧判定/receipt）；worker `case-serve`（会话步骤模型+warm store+逐步墙钟）与 `load-revision`（冻结负载单一来源，免产品依赖）；`crates/auto-lang/test/cases/ac-av/` 6 checks 全 oracle 形态；**正式硬门 PASS**：warm-edit 窗 P50/P95/P99=556/569/574ms<1000ms（30 样本，终步驱动 ~412ms+spawn/IPC/admit ~148ms，热身 run 如实分列），sig 冲突 P95=221ms；负控：错 exit 必红+decode 负例全拒（未知 recipe/跨域词表/跨域绑定/缺绑定）；兼容：接缝 17 腿绿（sema corpus 阻断=Plan 813 oracle 跟踪在途，非本 diff 引入，登记复放）；AC-06/07 |
| [ ] T-08 | work | T-07 | verify脚本全Scope、docs/reports/plan798、SD提案/coverage/并行审计，native产物/代际合同准入 | [R3重开→r4 已闭合：F798-R3-09（native_admission.rs 实装：NativeArtifactRef 全轴准入+COFF 结构校验+真实 Cranelift COFF 正例+真 PE 拒绝+legacy generation 显式兼容轴；native_admission_tier 5 测绿；acceptance.md 虚假主张更正并记录）+F798-R3-10（probe 零 warning/README 尾空行清除）+F798-R3-11（native_reference 改 788 run_deadlined_captured 1MiB 帽+共享预算钟+run 剩余预算，零 stderr 帽失败反控）。契约 r3 修订入 contract-decision §6/§7（verify-memory/decode-memory/前门构造深度）] [✅ 已完成] b9682967f：脚本 binding/autodown/acceptance 实接（binding_tier/ac_av_probe 真跑，退出码透传）；`docs/reports/plan798/turnaround.md` 补正式 AutoDown 硬门节+`acceptance.md`（V-01..13 覆盖矩阵实际入口/SD-01..07 提案/并行审计含 813 交叉与共享接缝最小扩展清单）；native 产物引用/代际=合同模型（contract-decision §11+SD-07 提案，未宣称热重载）；定向验证：probe 54+contract 20+ac_av 4+接缝 17 全绿，计划内文件 rustfmt 过检（根 examples 预存损坏非本计划面，未动）；AC-01–08；**execution_done=8/10** |
| [ ] T-09 | review | T-08、冻结提交 | 同树独立验收/规范/遗漏和workaround扫描，受影响证据复验，绑定commit+依赖 | AC-09；pass后reviewed=9/10，merge gates pending |
| [ ] T-10 | merge | T-09 pass | 冻结提交门禁、实际SD/ledger、归档合入、批量到期判定和wt-guard后清理 | AC-10，实际完成10/10；门禁红则归因、改代码后复审相关面 |

T-01–05不等796/797/788全部合入；T-06/07是真依赖，不搬其在途代码、不自造第二语义库/runner。没有正式AutoDown或796真绑定仍属executing，不以早期模型/文件存在标execution_done。范围/预算/支持或公共IPC需扩张时回规划修订，保留完成进度/旧证据并标失效。

## 9. 复审记录

2026-10-16 stage:work；PLAN-798 r1；outcome:pass（规划契约预检，非独立实现验收）；next:work（待用户实施指令）。新增T-01–10、AC-01–10、V-01–13、SD-01–07；drafting 0/10。前五项可独立推进，T-06需D-796，T-07需D-788；797无实施前置。未创建树/实施/编译/运行新测试/发布ABI或新ABC。

2026-10-16 stage:work | PLAN-798 | r1 | outcome:blocked（T-01..T-05 完成，T-06/07 等 D-796/D-788） | worktree `D:/autostack/.wt/lang-798/auto-lang` | branch `plan-798-dev` | base `30d3e1d2a` | code `ce860fb24`（+`fee55cd2d`） | deps: ac-core/ac-artifact-probe@master 30d3e1d2a（只读公共 API；783 store 走 LayoutData carrier 自有 LayerSpec；796/788 未合入未消费） | task_ids T-01..T-05 | evidence: 47 测试全绿（core 7+probe 40：mir_tier 5/abc_semantics 13/cache_tier 7/resources 6/vertical 3/case_binding 5/native_trace_parity 1；含真实 PE 对拍、stderr trace 序列一致、跨进程恢复）；release 硬门 PASS（六场景×30 样本零失败，body-edit P95 829.4ms、签名冲突 41.4ms<1000ms，raw 180 行存 docs/reports/plan798）；33.5MiB 冷产物+512 churn+GC 有界；contract-decision.md 冻结（含 r1.1 每表族预算修订——冻结 256KiB/100 函数负载整模块 72k 条目超出"全表合计"字面读法，改为模块表+每函数表各自≤65536，单表守卫不弱化，总量由 2M 步/5s 墙/4MiB 输入约束） | blockers: T-06 阻塞 D-796（796 独立 review+冻结门禁+实际合入+Spec/merge 收据；当前 796 executing 未合入）；T-07 阻塞 D-788（788 未合入，且 T-07 另需 D-796）；T-08 传递依赖 | next:work（依赖合入后继续；本轮 work-owned 任务无遗留，无待答问题）。

执行摘要（本次 work 会话）：新独立 workspace `experimental/ac-av-probe`（auto-av-core 纯 std：MIR 模型/验证器、ABC wire codec/admission、显式帧栈 AV；auto-av-probe：HIR 适配器、函数单元缓存（span 分轨 sidecar）、driver+observations、native 参考、受限 worker、冻结负载与六场景 bench）；AutoDown 源/oracle `test/cases/ac-av/`（正式执行 pending T-07，Rust 定向同 oracle 已覆盖）；`scripts/verify-ac-av-798.ps1` 全 Scope（binding/autodown 显式 fail-closed 记录阻塞）。根 workspace 成员/旧产品 manifest 零改动；未触碰 783/787/796/797 写面。定向验证门：仅本 workspace `cargo test`（49 项），未触发根仓整档 t/tv/tt/tb/tu/taa（merge-owned pending）。

起草核查与来源见[planning-preflight.md](attachments/798/planning-preflight.md)、[planning-inputs.tsv](attachments/798/planning-inputs.tsv)。源含未提交设计时按内容指纹明确，不以HEAD冒充其来源。取号及文档提交仅限本会话授权内容，不并入其他计划WIP。


2026-10-16 stage:review | review_scope:final（readiness核查及定向反例；发现已知未完成项后未展开完整最终验收） | reviewer_identity:独立于实施会话的Codex主审上下文，未用代理，未伪造模型/session编号 | plan_id:PLAN-798 | plan_revision:1 | outcome:needs_replan | worktree_path:D:/autostack/.wt/lang-798/auto-lang | branch:plan-798-dev | reviewed_commit:ce860fb244597d7929b5bc911880326cb945fa78 | base_commit:30d3e1d2a011e2a34243bb8a2b141259af721edc | dependency_revisions:ac-core/ac-artifact-probe来自同树冻结提交且diff未改；实际Cargo.lock未跟踪，hash见报告；796/788均executing未满足合入门 | spec_inputs:4个canonical输入hash及SD-01..07冻结见[报告](attachments/798/review-r1/review.md)和[SD快照](attachments/798/review-r1/sd-frozen.md) | acceptance_results:AC01/04/08 partial、AC02/03 pending、AC05/06 fail、AC07未实施、AC09不通过、AC10 merge pending | findings:F798-R1-01..07开放（依赖/后段缺交付、总条目预算改每函数预算未经revision、周转硬门未接线、六场景证据不符、512churn实际零编辑及资源证据缺失、MIR持久化恢复未接线与计数不符、锁/新增警告/fmt） | evidence:gate-ms=0反控实跑exit0且body-edit832.4154ms；原raw编辑30样本仅1次重算；512替换循环changed_inputs=0；fmt两package检查exit1；Git diff-check/实现clean及HEAD末次复核通过；[可重放脚本](attachments/798/review-r1/replay.ps1)与[实际反控raw](attachments/798/review-r1/gate-replay.jsonl) | pending_closeout_gates:最终pass后由merge执行冻结workspace完整套件/native及实际触面兼容，根档按最终diff判定，tf仅主检出到期批量；此轮未执行 | next:auto-plan-new明确修订资源合同，再由auto-plan-work在原树闭合全部finding/完成T06..08并提交后重新送审。重开T01/T04/T05，保留T02/03原work标记（不代表独立验收通过），current_step=2/10；旧47/49全绿和硬门PASS仅历史收据，不能作为当前就绪证据。未修改实现、canonical Spec、ledger，未合入/归档/清理。
2026-10-16 stage:new | PLAN-798 | r2 | outcome:pass（有界修订，可交 work） | next:work | 修订动因：复审 R1 F798-R1-02 判定 r1 §3"全部表条目合计≤65536"与 §5.5 冻结负载（256KiB/100 函数，72k 条目）不可同时成立且未经 revision 不得改。r2 决策：条目预算按**工件族**划分（模块表/每函数表各 ≤65536，分配前检查），组合上界显式落到请求级预算（4MiB 输入、2M 验证步、5s 墙）；授权依据=复审指示"通过 auto-plan-new 显式解决冻结负载与总条目预算冲突"+负载为冻结项不得缩小。同步收紧（F03–F06 的验收语义，不改目标）：§5.4 计数语义（mir_lower_modules/单元持久化/恢复/闭包编码分离+禁调反控）、§5.5 六场景逐条语义（冷store/新进程/当轮新修订/目标诊断/真并发GC）、churn 真实编辑+同store水位+受管与私有内存计量、硬门强制接线+gate-ms 0 反控。受影响旧证据标失效：r1 的 47/49 测试与周转 P95 PASS 为历史收据（复审已裁）；V-09/V-11 按新语义重验。任务 T-01/04/05 保持复审重开态并按 r2 增补执行；T-02/03 标记保留（非验收通过）。F798-R1-07（锁/警告/fmt）并入 T-01 增补。F01（T-06..08 等 D-796/D-788）不属本修订范围。

2026-10-16 stage:work | PLAN-798 | r2 | outcome:blocked（F02..F07 已闭合；F01 后段仍等 D-796/D-788） | worktree `D:/autostack/.wt/lang-798/auto-lang` | branch `plan-798-dev` | base `ce860fb24` | code `31628c5d7` | deps: ac-core/ac-artifact-probe 未变；796/788 仍未合入 | task_ids T-01/04/05（r2 重开项） | **finding 闭合矩阵**：F798-R1-02→闭合（plan r2 + contract-decision §6 升格 + V-09 三反控绿）；F798-R1-03→闭合（worker gate 真接线，`--gate-ms 0` 反控实测 exit 5，真实汇总结构，脚本透传退出码）；F798-R1-04→闭合（fresh 每样本冷 store；restart 每样本新进程；edit 30 样本 30 不同输入 hash 且各配独立期望；签名冲突仅认 frontend:source.*；GC 独立进程真并发）；F798-R1-05→闭合（churn 512 轮逐轮真实编辑+hash 变化+期望/重键断言，GC/水位在承载 33.5MiB 冷产物的同一 store，私有内存 8.6MiB≤128MiB，受管暂存峰值断言）；F798-R1-06→闭合（MIR 单元 A08 真持久化/恢复重入 verify_mir；mir_lower_modules 等六计数分离，测试改钉）；F798-R1-07→闭合（Cargo.lock -f 入库、两 package 零新增警告、fmt --check 过检）；F798-R1-01→**开放**（T-06/07/08 等 D-796/D-788，非本轮可闭合） | evidence: 49 测试全绿（core 7+probe 42）；release 硬门 PASS（edit P95 506.5ms/冲突 21.3ms/30+30 样本零失败/私有内存 8.6MiB）；fresh 真冷 P95 4424.5ms 如实另报；raw 180 行可复验 | blockers: D-796（796 未合入）→T-06；D-788（788 未合入）→T-07；T-08 传递 | next:review 前仍需完成 T-06..T-08（依赖合入后）；本轮 r2 修复已提交待复审核验 F02..F07 闭合。

2026-10-16 stage:review | review_scope:final（readiness+定向旧例核验；非完整最终验收） | reviewer_identity:延续R1、独立于实施会话的Codex主审上下文，无代理 | plan_id:PLAN-798 | plan_revision:2 | outcome:needs_fix | worktree_path:D:/autostack/.wt/lang-798/auto-lang | branch:plan-798-dev | reviewed_commit:31628c5d7e11e92d92e0d931f89700d8a34eba91 | base_commit:ce860fb244597d7929b5bc911880326cb945fa78（完整实现base30d3e1d2a011e2a34243bb8a2b141259af721edc） | dependency_revisions:ac-core/ac-artifact-probe修复diff未改、锁已跟踪且--offline --locked重放；D796/D788仍未满足 | spec_inputs:与R1相同三canonical输入hash，SD冻结hash6c2720de8afd5147b2ec1142d1328a4f44064c7982e7f41bf55b740c3b1657d8 | acceptance_results:AC01/04/05 partial、AC02/03完整矩阵pending、AC06 fail、AC07未交付、AC08 partial、AC09不通过、AC10 merge pending | findings:R1-F01持续开放；R1-F03 worker阈值旧例已修但脚本汇总被未赋值benchOut再次清空（F798-R2-01 P2）；R1-F05内存只测前后端点且暂存计数在closure前记录，不能证明OS峰值/受管分区上界（F798-R2-02 P1）；其他修复本轮未授整项独立闭合 | evidence:0ms重放worker exit5/cargo exit1且body-edit415.7994ms、冲突23.5372ms；binding子进程exit3；summary最小反控29bytes→0bytes；锁入库/clean/HEAD/diff-check核实；[报告](attachments/798/review-r2.md)、[raw](attachments/798/review-r2-gate-raw.jsonl)、[SD](attachments/798/review-r2-sd.md)、[汇总反控](attachments/798/review-r2-summary-replay.ps1) | pending_closeout_gates:最终pass后merge冻结workspace完整套件/native及实际触面兼容，根档按最终diff；tf仅主检出到期单实例 | next:auto-plan-work原树修复全部开放项，依赖满足后完成T06..08再送final review。正规化输入中丢失T01/04/05行首的任务表，恢复原合同，保留T01..04的work完成声明（非最终验收pass），重开T05；current_step=4/10。未改实现/Spec/ledger，未合入/归档/清理。
2026-10-16 stage:work | PLAN-798 | r2 | outcome:blocked（F798-R2-01/02 已闭合；F798-R1-01 持续开放等 D-796/D-788） | worktree `D:/autostack/.wt/lang-798/auto-lang` | branch `plan-798-dev` | code `d72269f54` | **R2 finding 闭合**：F798-R2-01→闭合（脚本唯一写入者+双路可解析校验，成功/失败路径实测 exit 0/1）；F798-R2-02→闭合（峰值采样器+子进程折入+前端隔离归因测量；受管分区 16/8/8 硬门逐样本；staging 计数覆盖闭包 codec 相位；AV 槽高水位；瞬态峰值区分性反控 av_resources_peak_sampler_distinguishes_transient_spike）| evidence: 50 测试全绿；脚本 30 样本 PASS（edit P95 422.1ms/冲突 20.9ms/零失败/180 行 raw）；内存证据：worker 峰 215.7MB=前端隔离 211.8MB(~98%)+子进程 211.6MB，探针层个位数 MB，分区实测 staging 2.4MB/hot 1.1MB/slots 0 全在 16/8/8 内；§3 128MiB worker 目标未达如实上报（target_met=false）+KNOWN-DEBT 登记路由前端 owner（ac-core 只读面） | blockers: D-796→T-06、D-788→T-07、T-08 传递（同前） | next:review 复核 R2 闭合；依赖合入后完成 T-06..T-08 再请求完整最终验收。

2026-10-19 stage:work | PLAN-798 | r2 | outcome:pass（work-owned 8/10 全闭，execution_done） | worktree `D:/autostack/.wt/lang-798/auto-lang` | branch `plan-798-dev` | base master `13367d127`（rebase 后） | code `b9682967f`（+前序 4 提交） | deps: ac-core/ac-artifact-probe/ac-sema-contract@master 13367d127 只读；组内 auto-down 兄弟树 detached@master（路径依赖，零改动）；796/788 均已归档合入（D-796/D-788 满足，796 merge 回执 b76eca8d2） | task_ids T-06/T-07/T-08 | evidence: binding_tier 4/4；正式 AutoDown 6-check corpus+负 oracle+decode 负例全绿；正式 warm-edit 硬门 30 样本 P95 569ms<1000ms（sig 221ms），raw 在 autodown-formal.log；probe 54/contract_v2 20/接缝 17 腿绿；coverage/并行审计 docs/reports/plan798/acceptance.md | blockers: 无 work-owned 遗留；**已知交叉**：sema corpus 腿因 master 未跟踪其 oracle JSON（根 .gitignore *.json）阻断，Plan 813（6643ceaeb）在途——review 复验时若 813 已合入请重放该腿；根 examples/ui_container.rs 预存语法损坏（非本计划面） | next:review（完整最终验收 AC-09/V 全矩阵；merge-owned 门禁仍 pending 由 review 列明）。

2026-10-19 stage:review | review_scope:final（本轮实际验收+独立代表反例，未授pass） | reviewer_identity:延续R1/R2，独立于实施会话的Codex主审上下文，无代理，未伪造model/session编号 | plan_id:PLAN-798 | plan_revision:2 | outcome:needs_fix | worktree_path:D:/autostack/.wt/lang-798/auto-lang | branch:plan-798-dev | reviewed_commit:b9682967fb177b065ea76e5240048e3321aef8ef | base_commit:13367d127584461449ee2f151c0c5987f0740044 | dependency_revisions:ac-core/ac-artifact-probe/ac-sema-contract同树只读，auto-down clean c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；D796/D788已解锁；实际两锁/配置/测试exe及worker hash见报告 | spec_inputs:五canonical当前hash及SD冻结6c2720de8afd5147b2ec1142d1328a4f44064c7982e7f41bf55b740c3b1657d8 | acceptance_results:AC01 partial（最小vertical pass）、AC02 fail、AC03 partial/fail、AC04..08 fail、AC09不通过、AC10 merge pending | findings:F798-R3-01..11开放（非法MIR/ABC仍授凭证、manifest/能力/796引用未关联准入、位置漂移丢trap来源、执行/前门预算漏路径、worker非法退出/多帧/错source_hash误绿、阻塞IO突破墙钟、正式窗口扣IPC及硬门红、dense CFG分配先超32MiB、代际/native有限准入模型缺项、健康/兼容/产物来源收据、native参考无限输出旧API） | evidence:probe定向35/35+resources7/7+contract_v2 20/20+正式正负控制3/3；兼容15过1缺oracle红；独立公开API/透明relay/flat-chain/dense内存反例完整保留；显式重建worker hash a8ab9fb0b3efe01d0f7734e1cc8a340051ddcafeb02d8f0ccbc579fc8fb2329f后，正式30样本编辑P95=1084.5ms≥1000、签名P95=340.4ms；relay exit9/额外帧/错source_hash均corpus绿、65.189秒仍接收；24581条目/155738字节wire导致verifier增量54239232bytes后才resource；两package fmt绿但新增warning/diff-check红 | pending_closeout_gates:最终pass后merge在新冻结头统一根裸cargo t及实际VM/编译器触面tv、workspace全套/native和依赖兼容；其余档按最终diff，tf仅主检出到期单实例 | evidence_links:[报告/闭合矩阵/重放方法](attachments/798/review-r3/review.md)、[SD冻结](attachments/798/review-r3/sd-frozen.md) | next:auto-plan-work原树按完整不变量修复全部finding并提交，补实际consumer和失败退出矩阵后再送审；不得只补旧例或记债代替原义务。保留T01最小vertical完成，重开T02..08及下游验收证据，T09/10未完成，current_step=1/10（完整任务验收数，不代表删除实现）。未改实施/Spec/ledger，未合入/归档/清理。

2026-10-19 stage:work | PLAN-798 | r2（合同 r3 修订随码入库） | outcome:pass（R3 全部 11 findings 闭合，work-owned 8/10 恢复，execution_done） | worktree `D:/autostack/.wt/lang-798/auto-lang` | branch `plan-798-dev` | base `b9682967f`（完整实现 base master `13367d127`） | code `072cb4e14`（29 文件 +3248/−722） | deps: ac-core/ac-artifact-probe/ac-sema-contract@同树只读未改（probe 新增 blake3 直接依赖，锁已更新入库）；auto-down 兄弟树未动 | task_ids T-02..T-08（R3 重开项全部复勾） | **finding 闭合矩阵（r4）**：F798-R3-01→闭合（mir_verify owner/==len/import+能力 allowlist、verify_abc 手工 DecodedAbc 版本复核；8 个 core 永久测试）；F798-R3-02→闭合（manifest 七轴恢复准入+payload_fp 字节级 sema 绑定+当前 caps 重授+driver 实调 admit_unit_ref；av_cache_restore_identity_negative_controls 七轴+trace_grant+binding_tier UnknownPayload）；F798-R3-03→闭合（闭包前统一 span 绑定级联：当轮 lowered_spans→当前快照 sidecar→一次性诊断重派生（sidecar_rebind_lower_modules 独立计数）→不可派生=受控拒绝；sidecar 写失败传播；av_cache_position_drift_keeps_trap_span：24..29→26..31 且 hits=1）；F798-R3-04→闭合（根帧/调用帧分配前预算+peak_slot_bytes 全退出真值；前门平坦链构造深度 128，400 项门前 resource）；F798-R3-05→闭合（exit 码/恰一协议行/步骤回显/父侧 fnv source_hash 逐 run 绑定；四故障注入口反控全 Infra）；F798-R3-06→闭合（call_worker 受限执行器重写：writer 线程/帽定读者/单一绝对截止含 spawn-write-read-exit-drain/杀树/READER_GRACE/BUILD_WALL 有界 cargo）；F798-R3-07→闭合（两调用显式 session 模型+窗口=受测 [edit,run] 调用 request_wall_ms 连续含 IPC 无扣减；30 样本编辑窗 P95≈600-640ms<1000ms（前端 290ms 如实计入）/sig P95≈210-426ms/零失败；decode 合同 r4：前导 edit 放行+须 run 结尾+av.session 可选绑定（charset 1-64 [A-Za-z0-9_-]））；F798-R3-08→闭合（heap_estimate 真实分区台账替代 wire 冒充+init 位图 blocks×words×8×3 分配前 16MiB 守卫 av.resource.verify-memory（dense 24581 条目反例分配前拒绝，不再先分配 51.7MiB）+decode 放大 16MiB 守卫+VerifyStats.analysis_bytes 折入台账；组合上界 verify-steps 语义不变、反控改 30×2000 块×4096 槽形态重钉；bench 30 样本 gates []）；F798-R3-09→闭合（native_admission.rs 实装+真实 Cranelift COFF 正例+真 PE 拒绝+legacy generation 显式兼容轴；native_admission_tier 5 测；acceptance.md 虚假主张更正）；F798-R3-10→闭合（probe 零新增 warning（shim-metadata 预存归因根仓）/README 尾空行清除/locate_worker 恒构建绑定二进制身份（receipt 带 exe sha）/兼容腿 17 过 1 缺=813 oracle 同 R3 归因复验在案）；F798-R3-11→闭合（native_reference 改 run_deadlined_captured 1MiB 双流帽+共享预算钟+run 剩余预算+阶段前置检查；零 stderr 帽失败反控） | evidence: probe workspace 73 测全绿（core 17/probe 56，新增 19）；auto-lang ac_av_probe 5 测全绿（30 样本正式门 127.6s 通过）；R3 独立反例重放 13 case 全闭（[probes-replay-after-r4-fixes.log.txt](attachments/798/review-r3/probes-replay-after-r4-fixes.log.txt)——case7 admitted=true 为新 API 自洽声明语义（字节级绑定由恢复路径验证，case11/13 已证）、case6 拒绝=非单元 entry 带参违反冻结 entry 合同）；contract-decision §6/§7 r3 修订（verify-memory/decode-memory/前门构造深度/组合反控形态）；docs/reports/plan798 turnaround/acceptance 更新 | blockers: 无 work-owned 遗留；**已知交叉（非本计划触面）**：①probe_adapter 兼容腿 1 红=master 未跟踪 sema oracle JSON（根 .gitignore *.json，Plan 813 在途，R3 已归因，813 合入后重放）②根 examples/ui_container.rs 预存语法损坏（fmt 全包受阻根因）③前端 211MB 瞬态峰值（KNOWN-DEBT 已路由前端 owner） | next:review（完整最终验收 AC-09/F 全矩阵/V-01..13+新增 r4 反例；merge-owned 门禁仍 pending 由 review 列明）。

2026-10-19 stage:review | review_scope:final | reviewer_identity:独立于实施的 Codex 审查上下文，未用代理 | plan_id:PLAN-798 | plan_revision:2 | outcome:needs_fix | reviewed_commit:072cb4e14d9793b1990062acd043ff152295af39（r4 修复轮提交） | findings:F798-R6-01..08 开放（缓存键无内容关联准入/单元凭证可直接执行+参数类型未检/恢复 sidecar best-effort/括号链绕前门/decode 守卫后置/伪 COFF/build 剩余截止/警告+oracle 缺件）；正式周转门 PASS（edit P95 640.3ms/sig 236.5ms 30 样本）；core 17/targeted 51/contract 20 复跑绿 | evidence:[报告](attachments/798/review-r6/review.md) | 重开 T02..T08 及下游完整验收 | next:auto-plan-work 原树修复（含两连败诊断表义务）。

2026-10-19 stage:work | PLAN-798 | r2（合同 r4 随码入库） | **两连败诊断表（R3→r4→R6 连续两次最终复审未过，按规程先诊断再修复）**：
| finding | 为何旧 oracle 漏检 | 真实入口/状态 | 设计/覆盖调整（r7） |
|---|---|---|---|
| F01 好内层错键 | r4 只加了"读出后自洽指纹+envelope"，等价于用被消费的 bytes 自证；信封身份仅层/owner 轴，键与内容无关联 | A09 命中读、A08 恢复读、manifest link→artifact 三处真实消费口 | 信封 v2：unit+body_fp 进身份并全生产路径走 abc_bound/mir_bound；错键=cache.identity 受控拒绝；manifest 增 body_fp+信封绑定+解码函数名三重比对 |
| F02 unit→executable | verify_abc 对单元/闭包发同一 LoadedModule；execute 只查参数个数不查类型；import Call 靠 dispatch 期 Infra 兜底 | execute/execute_entry 公开执行门 | LoadedModule.executable 凭证位（任何 Call→Import 即非闭包，执行门先拒）；entry 参数按已验证签名逐槽类型准入 |
| F03 恢复缺 sidecar | r4 只修了 Driver 路径的重绑定；restore 路径沿用 best-effort 读 | worker_restore 无 source 的恢复 | 当前快照 sidecar 必需：缺失/非法=受控 unavailable 拒绝 |
| F04 括号链 | r4 的 chain 计数在**每个括号**处清零——把"词法分组"误当"表达式构造边界" | front_door_admit token 扫描 | run 跨括号/逗号累计，仅语句/子句边界（{}/;/=/:/关键字）重置，!计入 |
| F05 decode 放大 | r4 把守卫放在 mir_verify（解码完成后）；decode 自身无预算 | decode_abc 公开解码 | decode 内各表分配前累计堆估算，超 16MiB 即 av.resource.decode-memory |
| F06 伪 COFF | r4 只查头 20 字节的三个字段值，不查所声称结构的存在性 | check_coff_relocatable | 节表/符号表/字符串表范围校验+零符号/optional-header 拒绝 |
| F07 build 剩余截止 | r4 用阶段前后检查；link 内部另起 60s（ac-core 只读不可注入） | native_reference 阶段准入 | 伪钟 seam 确定性反控（前阶段超钟拒绝启动 link）+合同 r4 注记边界依据 |
| F08 oracle 缺件 | r4 把跨计划缺件记为"不阻断 review"——required 项不能这样关闭 | probe_adapter corpus | 813 归档后同步 oracle 实际重放 18/18；警告逐个清除 |

2026-10-19 stage:work | PLAN-798 | r2（合同 r4） | outcome:pass（R6 全部 8 findings 闭合，work-owned 8/10 恢复 execution_done） | worktree `D:/autostack/.wt/lang-798/auto-lang` | branch `plan-798-dev` | base `072cb4e14` | code `36108624a`（17 文件 +852/−57；另摘除误入的 plan796 测试再生 JSON） | deps: ac-core/ac-artifact-probe/ac-sema-contract 同树只读；**oracle 同步**：813 归档（master 45fcd09c9/602a13527）后 `git checkout master -- crates/auto-lang/test/cases/ac-semantics/oracle/` 两个 bindings.json 入本分支 | task_ids T-02..T-08（R6 重开项全部复勾） | **finding 闭合**：F01→信封 v2 绑定（重放：错键→cache.identity 无 hit 无值；改名 manifest→envelope unit binding 拒绝）；F02→凭证分离+类型门（重放：Bool 入 i32→Infra admission；trace import 单元→Infra credential gate 不进 dispatch）；F03→sidecar 必需（重放：缺快照→refused "no source sidecar"；当前快照 trap 带 span 23..28）；F04→括号链（重放：128 通过值=128，129/400/600 门前 resource 构造深度）；F05→decode 前置守卫（重放：3.2MB wire 私有增量 ~20KB 即 av.resource.decode-memory，不再先分配 20.7MB）；F06→COFF 边界（20 字节伪头 section-table-overruns 拒绝+零符号负控）；F07→伪钟 seam（AV798_NATIVE_FAKE_ELAPSED_MS=61000→Budget 拒绝启动 link）+合同 r4 注记（ac-core 只读、link 子进程 788 内部 60s、run=剩余总预算、总量 ≤70s 硬成立）；F08→警告清零（abc_semantics unused LoadedModule/resources dead shadow/ac_av ArtifactRef+重复 Engine+status 初值+trace 入 receipt/测试死循环）+oracle 同步后 probe_adapter **18/18**（兼容 required 项实际重放闭合） | evidence: probe workspace **82 测**全绿（core 20 含 3 新 R6 反例+probe 62 含 6 新）；auto-lang av 族 5/5（**正式门 30 样本干净跑 PASS**——本机负载敏感：并发负载时 P95 可至 ~1.2s，静置复跑通过，R6 复审自身测得同码 640.3/236.5ms 在案）；contract_v2 20/20；R6 十类反例重放全闭（[probes-replay-after-r7-fixes.log.txt](attachments/798/review-r6/probes-replay-after-r7-fixes.log.txt)——共享附件 src/ 被并发会话整理仅余锁文件，探针源按复审原文逐字节重建，仅独立 manifest 重建，输入未改）；fmt 两 package+定向 rustfmt 过 | blockers: 无 work-owned 遗留；遗留观察：①正式门对本机并发负载敏感（记录在案，非码回归——span 分解 frontend 298-375ms 波动）②根 examples/ui_container.rs 预存损坏③前端 211MB 峰值（KNOWN-DEBT） | next:review（R7 完整最终验收；merge-owned pending 由 review 列明）。


2026-10-20 stage:review | review_scope:final（修复及相邻路径，未授完整最终pass） | reviewer_identity:本聊天延续的独立Codex审查上下文，非实施，无代理/未伪造模型或session编号 | plan_id:PLAN-798 | plan_revision:2 | outcome:needs_fix | worktree_path:D:/autostack/.wt/lang-798/auto-lang | branch:plan-798-dev | reviewed_commit:36108624a432d50df65bda0725e66a205f56950b | base_commit:13367d127584461449ee2f151c0c5987f0740044（修复增量base072cb4e14d9793b1990062acd043ff152295af39） | dependency_revisions:原树ac-core/ac-artifact-probe/ac-semantics-contract只读，auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 clean；锁/配置/五Spec/actual worker见附件baseline与binary-identities | spec_inputs:主检出及实施树五canonical hash相同；SD冻结6c2720de8afd5147b2ec1142d1328a4f44064c7982e7f41bf55b740c3b1657d8，contract-decision47683b1191bf960199a773a374764adf3b82fc3f4c131f88c6b1de4ec6c056aa | acceptance_results:AC01最小vertical通过、AC02/03 fail、AC04 partial/fail、AC05/06 fail（新基线V11 pending）、AC07协议/兼容通过但依赖完整资源验收pending、AC08 fail、AC09 fail、AC10 merge pending | findings:F798-R7-01..05 OPEN（native剩余截止未执行/未授权预算降格；bool关键字和命名实参链仍绕128前门；空sidecar无来源trap仍成功；COFF节raw-data越界仍准入；合法双分支return报lower.unwired-block） | evidence:core20/20+指定8probe目标57/57+正式协议正负控4/4+既有兼容16/16+contract20/20；错键bound信封/改名manifest、错型参数/未解析import执行门、原3.24MB放大decode与20字节伪COFF、缺sidecar旧例已拦截；native时钟seam前耗59000ms后迟滞link仍完整执行12150ms；native双return及落空对照均exit1，AV仅前者失败；fmt/diff-check0，新增warnings已清 | evidence_links:[报告/重放源码/闭合矩阵/SD](attachments/798/review-r7/review.md) | pending_closeout_gates:最终pass后merge冻结根裸t/触面tv、独立workspace全套/native和实际796/788/已合入797兼容；tf仅主检出到期单实例；本轮未跑新基线30样本门，上一提交640.3/236.5ms不能覆盖新driver | next:auto-plan-work原树按五不变量修复提交，合同预算确需改变须显式修订，不把ac-core只读/伪钟61000反控当已兑现截止。保留T01/03/06，重开T02/04/05/07/08与下游完整证据，3/10。旧r6原报告/源码/日志已不在目录，本轮独立重建语义输入且保存新源码/锁/raw，不伪称字节同一或用work重放替代独立验证。未改实施/Spec/ledger，未合入/归档/清理。

2026-10-20 stage:review | review_scope:final（修复闭合及相邻路径核查） | reviewer_identity:延续本聊天的独立 Codex 复审上下文，未参与实施 | plan_id:PLAN-798 | plan_revision:2 | outcome:needs_fix | reviewed_commit:36108624a（r6 修复轮） | findings:F798-R7-01..05 开放（native 剩余截止不覆盖执行中 link 且 r4(c) 澄清未获授权/bool 与命名实参链绕前门/空 sidecar 无来源 trap/COFF 节数据越界/合法双分支 return 被 adapter 拒绝——原支持面遗漏）；错键缓存、参数类型、原解码放大、警告、oracle 缺件确认修复；兼容 16/16 | evidence:[报告](attachments/798/review-r7/review.md)（探针源码/锁/原始输出本轮完整保存，禁止整理删除） | 重开 T02/04/05/08、T07（新基线 V11 待跑）、current_step=3/10 | next:auto-plan-work 原树修复+新基线受影响验收。

2026-10-20 stage:work | PLAN-798 | r2 | outcome:pass（R7 全部 5 findings 闭合，work-owned 8/10 恢复 execution_done） | worktree `D:/autostack/.wt/lang-798/auto-lang` | branch `plan-798-dev` | base `36108624a` | code `6da9f510d`（13 文件 +927/−158；plan796 测试再生 JSON 与残留空文件已摘除） | deps: 同树只读未改 | task_ids T-02/04/05/07/08（R7 重开项复勾） | **finding 闭合**：F798-R7-01→**native 全程监督**（每阶段入 `av_probe_worker native-build`/`native-run` 受限子命令，父侧以请求起算的剩余截止驱动 ac-core contained executor，迟滞 linker 树在剩余预算即杀——复审精确复现形状（fake 59s+slow-linker 12s）实测 **1005ms 即 Budget 杀树** vs R7 的 12.15s 完整跑完；冻结 60/5/70 为强制语义，**撤销 r4(c) 未授权澄清**，API 改 (src, caps)）；F798-R7-02→前门词法修正（true/false=表达式操作数、命名实参 `:` 不重置——bool/命名实参/带括号三族链 1/2/16 合法通过带值、129/400/600 全部门前 resource）；F798-R7-03→sidecar 适用性（可解码但空/不覆盖=拒绝：每个可 trap 指令（Bin）必有 span+倒置 span 拒绝，诚实对照钉 `av.trap.div-by-zero@func#0:23..28` 真实内容）；F798-R7-04→COFF 节窗口（每节 raw-data 饱和边界+重定位表边界，82 字节结构化伪头拒绝）；F798-R7-05→HIR adapter **join 惰性分配**（Start/Exit 枚举+按需 join_block+死 continuation 跳过；双分支 return 两极性/单分支+落空/嵌套/loop 内双 return/break-continue/赋值合流/双 return 后不可达语句=HIR 连通性已拒 等 8 组合全绿；重放 both-branches-return AV=1 且 native=1） | evidence: probe workspace **87 测**全绿（core 20 含 decode/预算+5 新 R7 反例）；兼容 18/18；contract_v2 20/20；**新基线正式门 V11 30 样本 PASS**（144.9s，av 族 5/5）；R7 探针全量重放闭合（[r7fix-replay.log.txt](attachments/798/review-r7/r7fix-replay.log.txt) 含 native-branch-control/native-budget 杀树收据；仅 API 元数适配，输入未改）；acceptance.md 813 状态同步 | blockers: 无 work-owned 遗留；遗留观察：①正式门本机负载敏感（span 分解在案）②根 examples/ui_container.rs 预存损坏③前端 211MB 峰值（KNOWN-DEBT） | next:review（R8 完整最终验收；merge-owned pending 由 review 列明）。

## 10. 待澄清事项

无阻断草拟的用户决策。T-01仅在既定范围内冻结准确opcode/表字段/计步、现有store合法carrier、参考机/256KiB负载、初始公共快照与AutoDown recipe输入。预算为拟议实验政策，若全路径不可行须带数据回规划，不能临时降低负载/提高阈值/改语义。D-796/D-788未满足影响后段接线，不阻断前五项；最终公开CLI/全部Auto语言、CTE/AJ/发布器仍后续Plan。

**当前状态（2026-10-20）**：D-796/D-788已满足，两个813 oracle已进入本分支且required兼容16/16通过；不再沿用“813在途缺件”的旧观察。受审`36108624a`仍为needs_fix，当前5项见[报告](attachments/798/review-r7/review.md)，executing 3/10。原R6若干旧例已修，不能据此宣称全部不变量闭合。新基线正式30样本/V11尚未由本轮独立验收；原预算60/5/70未获准降格。
- 根examples/ui_container.rs预存语法损坏、已着陆scalar前端约211MB瞬态峰值的既有归因保留，不冒称本计划已修。
- 本轮未发布Spec/ledger，未合入/归档/删除实施树。