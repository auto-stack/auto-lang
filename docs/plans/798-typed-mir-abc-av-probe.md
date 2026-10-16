---
plan_id: PLAN-798
status: executing
feature_name: Typed MIR 与 ABC/AV 执行合同及首个全路径探针
plan_revision: 1
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
current_step: 5
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

拟议探针策略（非用户给定值、非语言永久语法上限）：单source/HIR/ABC输入≤4MiB，单次codec页≤256KiB；所有类型/常量/函数/块/局部槽/指令表条目合计≤65536，按实际分配前检查；旧source/HIR文本前端探针递归预算为128层（含括号/块及连续表达式构造路径，按现有token流作有界前置准入，不能只数花括号），超出在旧递归parser前报resource；不改变旧独立CLI支持面。MIR/ABC验证每请求≤200万工作步、wall≤5s；AV每执行≤100万指令步、≤1024帧、活槽数据≤8MiB、wall≤2s，取消每≤1024指令检查。大于表预算是resource拒绝，不是非法语法/unknown opcode。条目计数和指令fuel定义在T-01钉死，不能通过不计验证/分配规避。

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

测试fresh、同进程hot、重启磁盘热、跨工程相同逻辑输入共享、单body/签名/依赖/profile/语义版本变更。正常cache hit必须仍运行AV并重新比较oracle，分别计mir_build/abc_emit/verify/load/av_execute；禁调build/emit反控证明跳过生产，不缓存PASS。CTE结果缓存留后续计划，不能将av_execute=0当本执行测试命中目标。

数据强引用是执行所需类型/常量/函数单元和模块manifest；失效/来源ref不永久pin旧HIR/AST。实际执行中lease保活数据，不将活帧存盘。良好frame替换错地址/错owner/key、非法内层、缺块必须在真实消费入口拒绝或受控miss重建；没有再生输入报unavailable，不读Nil/0或旧PASS。冲突/取消/写失败保留旧已提交结果并释放暂存/lease。

### 5.5 资源、预算及可观测周转

从T-01 trace请求、已有前端/verify、依赖验证、cache读取/解码、MIR lowering/verify、ABC emit/verify/load、AV执行/结果比较和发布观察；包含IPC/排队/IO/取消，嵌套span不能重复相加。native编译/链接/运行单独trace用于对拍，不混入AV执行微基准后称完整native周转。

T-05使用至少32MiB真实MIR/ABC产物（多个各≤4MiB模块，流式生成）、随机只读一个函数/模块和512次局部修订+GC，验证32MiB受管/OS增量观察及783物理水位收敛（无live超额时继承64/48MiB策略）；强引用及活lease保活，用户哨兵不变。新建小scratch不扫用户cache。

新MIR/ABC的4096块直链及混合CFG在预算内遍历/编码/验证/释放；旧source/HIR路径的连续/交错控制/表达式嵌套在128预算边界两侧分别完成或前置resource拒绝，另用4096层输入证明拒绝发生在旧递归parser之前。T-01复用`source::lexer::tokenize`及`atom_text::tokenize`核其真实递归风险和前门覆盖，不另造lexer/parser；若无法安全覆盖所有表达式递归路径则该可行性门不通过，回规划而非靠扩大线程栈或崩溃转译。验证坏ref/表达式引用环反控，CFG合法循环不能一概当非法cycle。成功或明确resource拒绝均依先冻结预算决定，不能把宿主stack overflow算资源状态。1024帧guest递归预算与HIR深度不同，执行链在边界两侧分别证明完成或resource；死循环靠真实fuel/取消终止，trace/IPC过量同样有边界控制。

T-01固定参考Windows机器/工具、256KiB core/scalar输入（由100个有意义函数body组成，非纯空白填充）、局部编辑位置及独立expected；key规模/负载不能随结果缩小。六场景各≥30样本：fresh、同进程hot、restart/cross-project磁盘热、单body编辑、callee签名变更（合法更新或正确诊断）、GC并发。报告P50/P95/P99、真实重算/命中、I/O/内存、全部失败样本。

**硬门**：warm session的单函数body编辑，或造成调用签名冲突的编辑，从实际请求入队至当前正确AV结果/诊断发布P95<1000ms；含当次必要前端/verify/依赖/cache/MIR/ABC/AV及比较。前端可整模块重查，但如实计量，不谎称前端细粒度增量已完成。native参考冷构建、全冷/广泛失效、超过预算的压力另报；不可扣实际IPC/等待，不能只加阶段P95。T-07正式AutoDown接入后按同负载重测；不能以Rust定向结果代替runner端到端。

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

| Task | owner_stage | 前置 | 文件/产物与可验成果 | 验证及AC |
|---|---|---|---|---|
| [x] T-01 | work | 用户实施指令 | 新workspace/lock、contract-decision.md、最小两函数vertical/MIR/ABC/Rust AV/store恢复、AutoDown源/oracle和计量首例；冻结§3/5.1策略 | [✅ 已完成] ce860fb24/fee55cd2d：`experimental/ac-av-probe/`（core+probe 独立 workspace）+ `contract-decision.md`（冻结 Schema/wire/预算/carrier/负载，含 r1.1 每表族预算修订）；vertical 三测绿（AV=独立期望、命中跳产仍执行、真实 PE 对拍）；store 首写+新进程 worker 恢复重验执行（`av_cache_worker_restore_new_process`）；AutoDown 源/oracle `test/cases/ac-av/`（5 用例）+ Rust 定向绑定 5 测绿；AC-01 |
| [x] T-02 | work | T-01 | core/mir与mir_verify、HIR adapter、scalar控制/调用/来源，补V-02–04 | [✅ 已完成] `core/src/{mir,mir_verify}.rs`+`probe/src/hir_adapter.rs`；mir_admission_/mir_lower_ 10 测绿：变异矩阵（悬空目标/越界槽/非法分支类型+哈希证明实际改动）、半初始化合流拒绝、合法循环/合流不误杀、landed p-binop-args/p-gcd-euclid 降级验证、迭代语句栈（200 连续 if）、text 域 unsupported 拒绝；AC-02 |
| [x] T-03 | work | T-02 | ABC codec/verifier/LoadedModule、AV显式帧/槽及全有限profile、预算/取消/测试能力 | [✅ 已完成] `core/src/{abc,abc_verify,av}.rs`；abc_codec_/abc_admission_/av_semantics_ 13+1 测绿：字节稳定往返/截断/尾随/校验和/未知版本-opcode（手工字节流）/bool 合法性；仅 verify_abc 产生可执行句柄、篡改内层必拒；trap 全矩阵（MAX+1/MIN负/除0/MIN÷-1/MIN%-1）结构化分态；未走 RHS trap 潜伏；trace 顺序+能力门（无能力执行前拒绝）；命名实参源序求值；fib(15) 燃料递归；燃料/取消/1024帧两侧；wire 携带 capabilities；AC-03/04 |
| [x] T-04 | work | T-03 | probe/cache、函数单元/依赖manifest/恢复准入及root/lease，错身份/版本/并发反控 | [✅ 已完成] `probe/src/{cache,worker_restore}.rs`+bin；av_cache_ 7 测绿：fresh→hot→restart/跨工程共享（命中 mir_build=0/abc_emit=0 且 av_execute>0 每次重比较）；body-only 单单元重键+caller 复用、签名变更全 caller 重键+正确 frontend 诊断；错 producer/schema/stage 信封受控拒绝（非 miss 非覆盖）；坏帧坏内层拒绝；同键异内容 Conflict 且旧提交原样存活；lease 保活过 GC、stale sweep；跨进程 worker 恢复（manifest→artifact id→重验→执行，值一致）；AC-05 |
| [x] T-05 | work | T-04 | 规模/预算/GC/深链、六场景trace与周转脚本/raw报告 | [✅ 已完成] av_resources_/av_turnaround_ 6+6 测绿：4096 直链+菱形+合法循环预算内验证（修 block_base O(n²)：31k 块 verify 419ms→11ms release）；超表预算=resource 分类；128 嵌套两侧+4096 前拒（door 在旧递归 parser 前、resource 态）；atom `[` 深度前拒；33.5MiB 冷 ABC 产物（64 模块分页合规）随机只读重验+512 修订+GC 水位有界；六场景×30 release 样本+**硬门 PASS**（body-edit P95 829.4ms/签名冲突 41.4ms/零失败，raw 180 行可复验 `docs/reports/plan798/turnaround-{raw.jsonl,summary.json,md}`+`scripts/verify-ac-av-798.ps1`）；AC-06（正式 runner 重测留 T-07） |
| [ ] T-06 | work | T-05、D-796 | 绑定合入语义ref/codec，退临时语义快照，冻结消费hash | av_binding_及完整受影响重验，V-13；AC-08 |
| [ ] T-07 | work | T-06、D-788 | 基于796已合入probe/run仅追加本recipe/worker/suite，正式AutoDown语义/cache/资源/周转与负控/兼容 | auto-case run、脚本autodown/binding，V-11/12；AC-06/07 |
| [ ] T-08 | work | T-07 | verify脚本全Scope、docs/reports/plan798、SD提案/coverage/并行审计，native产物/代际合同准入 | acceptance V-01–13/依赖树/格式警告；AC-01–08；execution_done=8/10 |
| [ ] T-09 | review | T-08、冻结提交 | 同树独立验收/规范/遗漏和workaround扫描，受影响证据复验，绑定commit+依赖 | AC-09；pass后reviewed=9/10，merge gates pending |
| [ ] T-10 | merge | T-09 pass | 冻结提交门禁、实际SD/ledger、归档合入、批量到期判定和wt-guard后清理 | AC-10，实际完成10/10；门禁红则归因、改代码后复审相关面 |

T-01–05不等796/797/788全部合入；T-06/07是真依赖，不搬其在途代码、不自造第二语义库/runner。没有正式AutoDown或796真绑定仍属executing，不以早期模型/文件存在标execution_done。范围/预算/支持或公共IPC需扩张时回规划修订，保留完成进度/旧证据并标失效。

## 9. 复审记录

2026-10-16 stage:work；PLAN-798 r1；outcome:pass（规划契约预检，非独立实现验收）；next:work（待用户实施指令）。新增T-01–10、AC-01–10、V-01–13、SD-01–07；drafting 0/10。前五项可独立推进，T-06需D-796，T-07需D-788；797无实施前置。未创建树/实施/编译/运行新测试/发布ABI或新ABC。

2026-10-16 stage:work | PLAN-798 | r1 | outcome:blocked（T-01..T-05 完成，T-06/07 等 D-796/D-788） | worktree `D:/autostack/.wt/lang-798/auto-lang` | branch `plan-798-dev` | base `30d3e1d2a` | code `ce860fb24`（+`fee55cd2d`） | deps: ac-core/ac-artifact-probe@master 30d3e1d2a（只读公共 API；783 store 走 LayoutData carrier 自有 LayerSpec；796/788 未合入未消费） | task_ids T-01..T-05 | evidence: 47 测试全绿（core 7+probe 40：mir_tier 5/abc_semantics 13/cache_tier 7/resources 6/vertical 3/case_binding 5/native_trace_parity 1；含真实 PE 对拍、stderr trace 序列一致、跨进程恢复）；release 硬门 PASS（六场景×30 样本零失败，body-edit P95 829.4ms、签名冲突 41.4ms<1000ms，raw 180 行存 docs/reports/plan798）；33.5MiB 冷产物+512 churn+GC 有界；contract-decision.md 冻结（含 r1.1 每表族预算修订——冻结 256KiB/100 函数负载整模块 72k 条目超出"全表合计"字面读法，改为模块表+每函数表各自≤65536，单表守卫不弱化，总量由 2M 步/5s 墙/4MiB 输入约束） | blockers: T-06 阻塞 D-796（796 独立 review+冻结门禁+实际合入+Spec/merge 收据；当前 796 executing 未合入）；T-07 阻塞 D-788（788 未合入，且 T-07 另需 D-796）；T-08 传递依赖 | next:work（依赖合入后继续；本轮 work-owned 任务无遗留，无待答问题）。

执行摘要（本次 work 会话）：新独立 workspace `experimental/ac-av-probe`（auto-av-core 纯 std：MIR 模型/验证器、ABC wire codec/admission、显式帧栈 AV；auto-av-probe：HIR 适配器、函数单元缓存（span 分轨 sidecar）、driver+observations、native 参考、受限 worker、冻结负载与六场景 bench）；AutoDown 源/oracle `test/cases/ac-av/`（正式执行 pending T-07，Rust 定向同 oracle 已覆盖）；`scripts/verify-ac-av-798.ps1` 全 Scope（binding/autodown 显式 fail-closed 记录阻塞）。根 workspace 成员/旧产品 manifest 零改动；未触碰 783/787/796/797 写面。定向验证门：仅本 workspace `cargo test`（49 项），未触发根仓整档 t/tv/tt/tb/tu/taa（merge-owned pending）。

起草核查与来源见[planning-preflight.md](attachments/798/planning-preflight.md)、[planning-inputs.tsv](attachments/798/planning-inputs.tsv)。源含未提交设计时按内容指纹明确，不以HEAD冒充其来源。取号及文档提交仅限本会话授权内容，不并入其他计划WIP。

## 10. 待澄清事项

无阻断草拟的用户决策。T-01仅在既定范围内冻结准确opcode/表字段/计步、现有store合法carrier、参考机/256KiB负载、初始公共快照与AutoDown recipe输入。预算为拟议实验政策，若全路径不可行须带数据回规划，不能临时降低负载/提高阈值/改语义。D-796/D-788未满足影响后段接线，不阻断前五项；最终公开CLI/全部Auto语言、CTE/AJ/发布器仍后续Plan。

**进行中阻塞（2026-10-16 work 会话末）**：
- T-06 等待 **D-796**：796 独立 review + 冻结门禁 + 实际合入 + Spec/merge 收据齐全。当前 796 仍 `executing`（T-01..05 收据在案，未合入 master）。解锁动作：796 走完 review/merge 后，本计划在新 base 上重取依赖 hash，实施真实语义 ref/codec 绑定并退临时快照。
- T-07 等待 **D-788 + D-796**：788（TestCase v2/native worker/共享协议）未合入。解锁动作：788 合入后按 796 已合入 probe/run 接缝仅追加 `ac-av-core-run` recipe/worker/suite，重测正式 AutoDown 端到端 P95。
- T-08 传递依赖上述两项。verify 脚本 `-Scope binding/autodown` 当前显式 fail-closed（exit 3）以保持门诚实。
- r1.1 合同修订（每表族预算）已记录于 `experimental/ac-av-probe/contract-decision.md` §6；review 时请复核该修订是否需要升 plan revision（当前判定：冻结负载与字面读法冲突的解释性修订，单表安全性质未弱化，不构成 needs_replan）。
