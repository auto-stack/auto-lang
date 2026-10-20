---
plan_id: PLAN-818
status: executing
feature_name: AC接入共享Typed MIR与原生一致性闭环
plan_revision: 1
author: [Codex]
created_at: 2026-10-19
updated_at: 2026-10-20
current_step: 1
total_steps: 10
supersedes_spec_components:
  - docs/specs/auto-ac/project.md
  - docs/specs/auto-ac/artifact-contract.md
  - docs/specs/auto-hir/stage-contract.md
  - docs/specs/auto-test-case/architecture.md
new_spec_components:
  - docs/specs/auto-ac/mir-native-scalar.md
  - docs/specs/auto-ac/mir-native-cache.md
touched_goals: []
affects: [auto-ac, auto-mir, auto-av, auto-test-case]
---

# [PLAN-818] AC接入共享Typed MIR与原生一致性闭环

## 0. 变更摘要

承接[Design43](../design/43-ac-av-typed-mir-and-abc.md)与[后端战略](../design/strategy/auto-native-backend-evolution.md)，在独立workspace建立 `Checked HIR → 798共享Typed MIR → 新Cranelift adapter → COFF → PE`，同一MIR可走AV。保留旧HIR→native作为独立参照，并以独立expected抓住两后端同错。

首个可执行用例即真实单元cache/增量、正式AutoDown、端到端周转和内存计量。不替换既有生产默认入口，不实现AJ或发布器。[817 CTE](817-av-cte-query-frozen-cache.md)与本计划无实施依赖，公共写面按[批次并行合同](attachments/ac-execution-batch-20261019/parallel-contract.md)串行。

## 1. 目标

1. 从实际合入的CheckedMir直接生成目标代码，准确降低typed slots/CFG、精确数值、调用、trace、trap和bool，不回译HIR或绕旧native emitter。
2. 已有scalar有限面在AV、新MIR-native、旧HIR-native和独立expected间一致，保留输入/source/profile身份。
3. 按函数单元缓存可重定位COFF及manifest/ABI/runtime需求，body编辑复用无关单元，真实链接/执行每次重新比较。
4. 将可信恢复、坏MIR/错target/依赖/对象身份及事务失败纳入实际消费门；首例正式AutoDown与阶段trace。
5. 固定局部编辑请求到正确PE结果的P95<1000ms和有界编译内存/磁盘治理，为未来AJ共用codegen建立事实，不增加完整JIT/发布验收。

范围：新 `experimental/ac-mir-native/` workspace及Cargo.lock、`test/cases/ac-mir-native/`、818脚本/报告/附件，最小recipe注册/独立父侧adapter。ac-core原生/前端/旧CLI及所有既有profile保持不动，不删除旧参照。

非目标：自研codegen/SSA优化器、替换Cranelift、公开AutoABI/FFI/C导入、text/record/堆对象/泛型/CTE执行、热重载/在线装载/OSR/deopt、完整AJ、通用发布器/跨平台包、完全增量链接器、生产SDK/默认命令迁移。机器码产物可复用不等于已交付通用增量发布。

## 2. 架构方案

```text
已有core/scalar输入 → 既有verify → Checked HIR → 798 HIR adapter/verify → CheckedMir
                                               ├→ ABC→AV（参照）
                                               └→ 新MIR→CLIF→函数COFF → 平台链接 → PE
同一原输入 → 旧HIR→native（另一路） + 手审expected
函数COFF/模块链接manifest → 783 managed store → 恢复/新鲜性 → 真实link/run
```

仅driver消费ac-core前端/native参照、798公开adapter和783；新codegen库输入是798 CheckedMir及显式配置/符号计划，不能读取HIR AST重做类型/控制流语义。无auto-lang/UI/auto-man/旧VM依赖。根members/旧产品manifest不改；Cranelift IR/平台ABI留后端内，不返回公共MIR。

MIR可变局部槽映射为Cranelift Variables或等价栈槽；FunctionBuilder处理其SSA桥，不修改MIR成强制SSA。bool保持逻辑值0/1，当前私有标量调用通道采用i32物理值，内部函数按Windows x64调用约定；entry wrapper按现有实验合同退出。此私有测试ABI不是797公共AutoABI，不拿同版本字样证明兼容。

## 3. 技术栈

Rust独立workspace，跟踪lock并复用ac-core锁定Cranelift 0.126栈/ObjectModule。首target `x86_64-pc-windows-msvc`，锁CPU baseline/flags/opt_level（首版none），SDK/import libs、linker工具身份真实入key；其他target拒绝，不静默用host机器码或AV兜底。

拟新增 `src/{config,admission,symbols,lower,emit,cache,link_driver,driver,worker,observe}.rs`，以及 `scripts/verify-ac-mir-native-818.ps1`。后续new命令如 `cargo test --manifest-path experimental/ac-mir-native/Cargo.toml mir_native_` 和脚本Scope `first-case|admission|semantics|cache|resources|autodown|turnaround|acceptance|closeout` 在相应T创建，当前不是执行证据。

进程使用已交付 `ac-core::link::{run_deadlined_captured_with,link_object_staged,...}` 受控设施。多函数COFF链接若旧API只收一个object，新workspace只组装标准linker输入并复用同一contained执行器/工具定位，不复制Job/管道生命周期或修改旧link.rs。T-02先证实多obj在真实SDK/链接器上可行，不以拼接字节伪造obj。

拟议实验硬cap（非用户指定/永久语言上限）：沿D-798 source/HIR/MIR字节/表族/深度/验证预算，不默默重解释r2合同；单COFF≤16MiB、一个请求对象合计≤64MiB、link输入≤128MiB、输出/IPC元数据各沿788冻结上限（元数据4MiB，stdout/stderr各1MiB）；build≤60s/run≤5s/请求≤70s。native不承诺AV fuel语义，循环/递归对拍用有界程序，原生超时/崩溃是infra而非语言trap。

新后端受管数据≤48MiB（MIR/符号热16、lowering/索引/codec暂存24、I/O8），工作区/依赖库分配分类完整报告；worker private bytes峰值硬门≤512MiB，包含受控link/PE子进程的请求树峰值≤768MiB。Cranelift内部allocator纳入进程峰值，不能只计自有Vec；编译可能不可中途取消时用父侧contained预算终止并归resource/infra，不继续输出成功产物。工具首次构建内存单列，不从请求内实际worker峰值扣掉前端。

## 4. 需求分析与背景调查

用户授权本轮草拟817/818、明确797/798及拆分并行、首例cache/AutoDown/亚秒/内存；未授权实施/开树/代理/工具安装或生产切换。只在auto-lang主检出写计划，不修改旧VM/前端/产品SDK。

依据[overview](../specs/overview.md)、[AC现状](../specs/auto-ac/project.md)、[HIR阶段](../specs/auto-hir/stage-contract.md)、[产物/cache](../specs/auto-ac/artifact-contract.md)、[测试架构](../specs/auto-test-case/architecture.md)、[v2](../specs/auto-test-case/profile-v2-native.md)；主检出调查 `13367d127584461449ee2f151c0c5987f0740044`，来源hash/在途分支blob见[inputs.json](attachments/ac-execution-batch-20261019/inputs.json)。

已核：ac-core `native::lower_object`目前从CheckedModule读HIR做CLIF；本计划不能调用它生成新路径，只在native_reference中调用。`link.rs`已有bounded bytes/Job/发布生命周期；783有store/ref/root/lease/GC，796已有semantic ref/查询与probe/run。auto-mir/auto-av新Spec/API仍待798实际交付。只读branch预测包含 `MirInstr::{Const,Copy,Bin,Un,Call,Trace}`、`Terminator::{Jump,Branch,Return}`、typed mutable slots；不是冻结依赖。

**D-798**=独立最终review/冻结门禁/实际合入/Spec/merge收据齐全。T-01可只读设计/语料准备，T-02起实际编译共享MIR必须等D-798；798若改profile/指令，重读合同与支持表，不照旧branch开工。D-797不是前置，本计划只私有测试ABI；817不是前置，不依赖尚未交付的CTE常量生产者。814–816可能迁移SDK/测试CLI，正式recipe写入以届时owner和真实路径为准，不抢改其大规模产品面。

## 5. 详细设计

### 5.1 支持面、信任与首例可行性

支持D-798实际有限i32/bool MIR：typed const/copy、精确加减乘除余/比较/相等、一元、jump/branch/return、直接函数调用、已批准trace能力。短路/命名参数求值已在共同MIR显式化，新后端只按指令顺序/CFG执行，不按参数声明重排、不提升未走分支trap。bool不能与i32语义混淆，fn/参数/返回保持类型区分。

只接受本进程verify产生的CheckedMir。持久MIR在进入codegen前用798真实解码/verify重新授信；坏owner/ref、slot未初始化、profile或能力错，在codegen计数0处拒绝。运行imports只允许测试trace，不开放FFI/任意native函数/AutoABI。新的MIR版本/指令未知即unsupported，不把它当NOP或转回HIR。

T-01在已观察支持面内冻结symbol/内部ABI、source/debug sidecar、全指令表映射、预算/负载/错误及对象key；只读，不在未合入API上造mock后端。T-02首个两函数例必须 `CheckedMir → 新COFF → 真link/run`、AV/旧native/独立expected一致，并有真实store put→新进程命中跳codegen、每次仍link/run、正式AutoDown首例/trace/内存。任一路不可行即报告阻断回规划，不先铺满框架再换成旧后端。

### 5.2 codegen与数值/调用语义

按MIR块创建CLIF块、槽到Variable/栈槽映射及block sealing，尊重initialize-before-use。保留被验证的语义，Cranelift verifier失败是backend错误，不能重做语义推导/删分支以通过。不要额外引入与MIR不一致的默认零值。

加减乘/负使用显式overflow检查；除0、MIN/-1和MIN%-1按既有scalar合同在执行时trap。不能先发硬件div再检查或被常量折叠提升为无条件编译错误。trap在本实验runtime包装层走明确失败出口（参考PE code70），正常结果通过自检entry或完整u32退出观察，不能只比低8位。Trace输出按原始bytes/source顺序比较，测试支撑库具名，不接受任意路径。

函数先声明再定义，支持前向/互递归签名；参数序列已由MIR决定，测试>4个参数、bool/i32混合、非对称值、嵌套调用及局部重赋值。有限native递归测试只在栈/墙钟可完成范围，不宣称与AV frame限额相同。未支持聚合、所有权/异常展开列unsupported，而不是给Drop空语义冒充资源支持。

### 5.3 函数单元、对象缓存与增量

每个函数COFF可重定位并保留external symbol；逻辑symbol由owner/声明/签名/私有ABI配置生成，不含绝对地址，不以body hash改掉所有callee符号。模块entry wrapper/必要runtime和link输入manifest独立。callee body-only改动重编该函数，caller对象在未内联/只依赖签名时可复用；callee签名/ABI变化使相关caller失效；函数次序改变不能串错引用。

直接从共享MIR类型/指令构造对象，不能 `MIR→HIR→旧lower_object` 或缓存旧native输出冒称新codegen。新模块不修改ac-core native.rs；通过计数、codegen故障注入及独立函数对象变化证明路径真实。

key包括当前函数MIR语义内容/owner、读取类型/常量/callee签名与外部符号计划、MIR语义/codec版本、Cranelift/backend版本、target/CPU/flags/opt_level、私有ABI/runtime及debug策略。codegen实际读到module级信息时必须完整入key，先实现明确保守粒度再优化，不能漏依赖伪称细粒度。产物路径/时间等非语义项从可共享payload剥离或显式换轴，不靠clear-store规避同键异内容Conflict。

复用783独立namespace/合法载体及producer envelope；大对象按≤256KiB页和manifest落盘/恢复，使每次publish服从783暂存上限而非一次写16MiB，A08消费798产物，A09保存新COFF，A10保存链接输入manifest/回执；required边保活当前对象/runtime/必要描述，MIR/source仅为来源时不永久强pin。命中对象可在其上游MIR缓存已回收但recipe/身份仍可计算的测试中真实link/run，不能将CheckedMir直接从磁盘trusted标志恢复。bad-address/owner/target/profile/callee-signature和自洽异址/内层manifest错配在实际消费门拒绝或miss，不能链接不匹配对象。

cache hit证明codegen_emit_units=0（禁调反控），link/run和父侧比较>0；不缓存PASS/进程退出结果代替运行。fresh/read-only/restart/cross-project分开，逐轮新body输入和输出oracle、真实并发GC/root/lease、同键冲突及失败保旧提交必须实测。

### 5.4 对拍、链接发布与失败

三条实际路径：同一输入经旧HIR-native；同一已验证MIR经AV；同一MIR经新native。每条都与独立expected核对，typed结果/PE自检码、trace及trap之前的副作用顺序一致。AV resource与native超时不要求同一种语言trap，也不能忽略双方状态。

多obj链接调用平台linker；复用路径审计/工具定位/Job/deadline/输出限额。只有全部对象/manifest/ABI/runtime需求预检通过才进入link。存在旧输出时，坏对象、link失败、超时、发布IO失败按既有staging/commit合同保留旧产物并回收owned暂存（发生提交后不可回滚的状态须有准确收据）；不以删除输出再重建冒充原子发布。任务只做实验构建输出，不通用化安装/部署发布器。

### 5.5 AutoDown与性能/内存

拟recipe `ac/mir-native-scalar-v1` 用probe/run、atom-data recipe输入、query/edit-source/restart，独立worker/父侧adapter；正常产物/manifest身份由父侧准入，每次真实PE结果重新比较，不复用worker PASS。T-02即最小正式case，T-06再覆盖所有分支/恢复/负控/旧协议兼容。原v1、788 native/syntax、796/实际合入797/798 recipes不改语义。

公共注册/入口只在现行owner交接窗口追加独立recipe，若817先合入须同步再改注册表。不得复制sema.rs执行器、再写.ad parser、让测试核心静态依赖Cranelift。SDK/工具拆分改了路径就按合入后公开面注册，不为本计划新建第二兼容层。

T-01固定参考Windows机器/CPU/SDK/工具、256KiB/100有意义函数的输入与独立expected。六场景各≥30：fresh真冷store、same-process hot、每次真新进程restart/cross-project、每次不同body编辑、调用签名/私有ABI编辑、真实并发GC。每次trace包含请求准备/序列化、前端/verify/MIR或恢复、依赖验证、codegen/对象命中、link、run、父准入/比较及实际等待/I/O。

**时间硬门**：warm session单函数body编辑及可合法重建的调用签名编辑的“请求准备→当前正确PE执行结果经父侧准入/比较”P95<1000ms；包含真实link/run，不扣编译/IPC/排队。对拍的旧native/AV另有完整正确性suite，性能主路径不强制每样本重编旧参考，但必须比较独立预期，不能拿旧runner结果当新执行。初始工具构建、全冷/大失效另报，非完整热重载或任意仓库亚秒承诺。

**内存/磁盘硬门**：按§3实时峰值和分区，包含编码/输出收集/Cranelift及子进程；以瞬态分配spike反控采样器。至少32MiB真实不同COFF/manifest冷集（不是重复复制同一个块）、随机只读一个函数对象，512次真实编辑/重键/结果+同store水位GC；按继承783策略无live超额时物理收敛、用户哨兵不变。测试须用真实不同产物推动同store超过高水位，确保至少一次物理删除并在活集低于低水位时实测收敛；不以未触发GC的低占用算通过。大型MIR块/循环/分支混合在798预算两侧通过或resource，不扩host栈掩盖不安全路径。raw中零/缺失/NaN/失败样本不能按0ms或0bytes默认通过；阈值=0的时间/内存反控必须使聚合gate非0。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/mir-native-scalar.md | 新CheckedMir→CLIF/COFF/PE有限实现、数值/调用/能力/unsupported | 与旧HIR路径分清 | AC-01/02/03 |
| SD-02 | add | docs/specs/auto-ac/mir-native-cache.md | 单元/symbol/key/依赖/恢复/资源/周转及真实链接 | 增量可复验 | AC-04/05/06 |
| SD-03 | modify | docs/specs/auto-ac/project.md、artifact-contract.md | 追加818 consumer/producer面，旧profile/producer不扩 | 不回溯宣布生产默认迁移 | AC-01/04/08 |
| SD-04 | modify | docs/specs/auto-hir/stage-contract.md | 追加共享MIR的native消费门，旧S0–S6事实保留 | 不泄露CLIF到HIR/MIR | AC-02/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 追加正式recipe/worker/actual/stale/观测支持面 | 单一runner | AC-07 |

## 6. 测试设计

拟test族 `mir_native_first_`、`mir_native_admit_`、`mir_native_codegen_`、`mir_native_parity_`、`mir_native_cache_`、`mir_native_transaction_`、`mir_native_resources_`、`mir_native_turnaround_`、`mir_native_adapter_`。由相应T创建并绑定真实命令/commit/raw；当前计划文本不构成已执行证据。

| Case | AC/invariant | 输入/反控 | owner/真实入口 | 可观察oracle/覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/02 真路径/信任 | 首例、坏MIR owner/slot/profile/能力、禁调旧lowerer | T-02/03 新codegen/verify | 合法新PE真执行；坏MIR codegen=0，不能靠旧native生成actual |
| V-02 | AC-02/03 数值/分支 | 所有scalar运算边界、负除余/MIN%-1、未走trap、loop/break/continue | T-03 三路径/独立expected | 每条独立符合值/trap/trace；不只比两后端同错 |
| V-03 | AC-03 调用/槽 | >4参数、bool/i32、非对称命名实参、嵌套/前向/递归、循环合流 | T-03 codegen/三路径 | 参数/求值/slot更新正确，初始化不靠隐式0；有限递归不等同AV帧预算 |
| V-04 | AC-04 真增量 | 禁调codegen的hit、新进程/工程、body编辑30不同输入 | T-02/04 cache→link/run | emit单元数准确、命中仍真link/run；依赖不变caller复用、结果=无cache |
| V-05 | AC-04 版本/依赖 | callee签名、函数重排、target/CPU/ABI/opt/debug/producer变化 | T-04入口/manifest | 适用层失效、不串符号；异target不跑host，坏配置受控拒绝 |
| V-06 | AC-04/05 坏块/GC | 自洽异址、坏obj身份/链接manifest、无MIR的对象命中、冲突/root/lease/512churn | T-04/05真实消费 | 拒绝/miss保旧提交；无上游活缓存仍可合法link；不删哨兵 |
| V-07 | AC-05 事务 | 三个旧输出、link错/timeout/IO失败与合法发布 | T-04 staged/link/publish | 比对旧bytes+旧程序结果；终止回收子孙与owned暂存，准确恢复收据 |
| V-08 | AC-05 内存/规模 | 32MiB真冷对象、深CFG、瞬态spike及cap两侧 | T-05采样/编译器 | 48MiB受管/512MiB worker/768MiB树硬门，超限不当trap或成功 |
| V-09 | AC-06 周转 | 六场景×30、body/签名编辑、时间/内存阈值0/坏raw | T-07正式runner | 两路径单墙钟P95<1s、真实PE；反控gate非0、阶段窗口可对账 |
| V-10 | AC-07 runner | 未知recipe/字段/工具、坏actual好expected、restart/stale/原协议 | T-02/06 auto-case | 真父判/工具身份/产物准入；infra不满足语言负例，旧通道兼容 |
| V-11 | AC-08 边界/来源 | D-798、SDK路径、依赖树、零797/817强依赖、SD | T-08审计 | 不复制MIR/AV、无产品默认/ABI冻结冒称、共享注册串行 |

V-01–11为必验区分性补充，固定语料族总绿不替代反控；required缺例/skip/缺SDK/编译器都是缺口。expected由手审算例/既有oracle固定，新backend不生成expected；native错误码、语言trap和执行器infra分开。

预定唯一树 `D:/autostack/.wt/lang-818/auto-lang`、分支 `plan-818-dev`，本轮未创建。实现/自检后同树独立review、冻结commit/依赖，暂停写入；未指定模型/人员，不创建代理。work定向测试/check，最终review pass后merge在冻结提交跑独立workspace完整套件/三路径/native及相关runner定向兼容；若改root Rust则按实际触面补裸t及tv/tt，未触及aavm/UI/生成器不跑taa/tu/docs_gen，tf仅主检出到期单实例。

## 7. 验收标准

| ID | owner_stage | 可验证结果 |
|---|---|---|
| AC-01 | work | T-01合同与T-02首例新MIR-native/真cache/正式AutoDown/计量齐；V-01/04/10 |
| AC-02 | work | 真CheckedMir前门/全部有限指令正确lower，旧路径未替代actual；V-01/02 |
| AC-03 | work | AV/新native/旧native/独立expected的值、trap和顺序一致；V-02/03 |
| AC-04 | work | 函数COFF/符号依赖/cache恢复/配置失效/真link-run；V-04–06 |
| AC-05 | work | 原子输出/失败清理/GC/全路径内存预算及规模硬门；V-06–08 |
| AC-06 | work | 两编辑正式用户路径P95<1s/六场景raw、时间内存反控有效；V-09 |
| AC-07 | work | 正式recipe/实际产物/父侧oracle/stale/旧通道兼容；V-10 |
| AC-08 | work | 依赖/并行边界/coverage/SD齐，非公开ABI/JIT/发布器；V-11 |
| AC-09 | review | 同树独立逐AC/V/SD审查，绑定冻结提交，in-contract finding全闭 |
| AC-10 | merge | 冻结门禁、Spec/ledger、归档合入/guard清理实际完成，10/10 |

## 8. 执行步骤

| Task | owner_stage | 前置 | 文件/交付 | 验证/AC |
|---|---|---|---|---|
| [x] T-01 | work | 用户实施指令 | 只读MIR映射/符号/ABI/工具/内存/负载/测试owner设计，contract-decision.md及独立golden | 指令覆盖/资源可审；AC-01，不编译未合入接口。[✅ 已完成 2026-10-20：[contract-decision.md](attachments/818/contract-decision.md)（全指令表映射/符号/私有ABI `msvc-x64/priv-i32-bool-v1`/trap70/预算与mn.*错误分类/对象key/层与恢复/工具链与参考机身份/48-512-768MiB分区/测试owner映射）+ [golden.md](attachments/818/golden.md)（首例源冻结+手算：exit 0/stderr "aab"/无trap审计；256KiB/100caller+14leaf+main负载设计、LCG seed 0x818_5eed、K=19、fn_041编辑位/fn_leaf_2签名冲突位、无trap界分析、闭式镜像+手审义务）。全程只读：主检出零代码WIP预检过、未编译在途接口、未造mock、未开树；观察基面=798在途分支072cb4e14（forecast非冻结，D-798后重读） ] |
| [ ] T-02 | work | T-01、D-798、公共注册窗口 | 新workspace/lock、两函数多obj新codegen→PE、三路径/expected、cache新进程恢复/正式AutoDown/trace/峰值首例 | first-case，V-01/04/10；AC-01 |
| [ ] T-03 | work | T-02 | 完整有限MIR指令/CFG/槽/调用/数值/trace与负例，精确错误/来源 | mir_native_codegen_/parity_、V-01–03；AC-02/03 |
| [ ] T-04 | work | T-03 | 函数对象/符号/manifest/版本key，恢复/再生/无MIR命中、事务/失败/反控 | mir_native_cache_/transaction_、V-04–07；AC-04/05 |
| [ ] T-05 | work | T-04 | 32MiB冷对象/512真实编辑/GC、内存分区/树峰值/深CFG/超限清理 | resources、V-06–08；AC-05 |
| [ ] T-06 | work | T-05、公共单写窗口 | 正式AutoDown完整suite及兼容/坏actual/身份/stale | adapter、V-10；AC-07 |
| [ ] T-07 | work | T-06 | 真实runner六场景、两编辑P95门、raw报告及反控 | turnaround、V-09；AC-06 |
| [ ] T-08 | work | T-07 | verify全Scope、docs/reports/plan818、边界/coverage/SD提案 | acceptance V-01–11；AC-01–08；execution_done=8/10 |
| [ ] T-09 | review | T-08冻结提交 | 同树独立验收/规范/遗漏审查及反控重放 | AC-09；reviewed=9/10，merge门禁pending |
| [ ] T-10 | merge | T-09 pass | 冻结门禁、Spec/ledger、归档合入、批量判定/wt-guard后清理 | AC-10；实际10/10 |

## 9. 复审记录

2026-10-19 stage:new；PLAN-818 r1；outcome:pass（规划契约预检，非实现验收）；next:work（需用户实施指令；T-02起等D-798）。新增T-01–10/AC-01–10/V-01–11/SD-01–05；drafting 0/10，未实施/编译/跑性能/创建树。详细预检见[planning-preflight.md](attachments/818/planning-preflight.md)。实际指令/资源/接口不符时以证据显式revision，不能改成旧native代理来维持pass。

2026-10-20 stage:work | PLAN-818 | plan_revision 1 | outcome:blocked（T-01完成；T-02起阻断于D-798）| worktree:无（T-01只读，主检出完成，产物落docs/plans/attachments/818/；实施树按§6于T-02创建）| branch:—| base_commit:13367d127584461449ee2f151c0c5987f0740044（主检出，未动代码）| code_commit:—（仅计划簿记）| dependency_revisions:798在途分支plan-798-dev@072cb4e14（只读观察，forecast非冻结）；ac-core/ac-artifact-probe主检出交付面（inputs.json hash）| task_ids:T-01（1/10）| evidence:attachments/818/{contract-decision.md,golden.md}；预检=主检出crates/test/experimental/scripts零WIP + 798 status=executing(1/10) | blockers:D-798未满足——798仍executing（最终review/冻结门禁/合入/Spec/merge收据未齐），T-02起禁止实际编译共享MIR/开实施树编译面；另公共注册窗口按并行合同串行 | next_responsible_role:等待D-798后由work继续T-02（新workspace/两函数多obj首例）

T-01要点：入口=用户实施指令（drafting→executing授权）；范围严守只读——冻结合同与golden，不编译未合入接口、不copy在途树代码。观察基面与798预测一致（MirInstr六指令/Terminator三终结器/11 BinOp/2 UnOp/typed slots/is_unit+fn:imports），未发现迫使revision的反例；工具链身份实测入冻（rustc 1.98.0/LLM 22.1.8、rust-lld LLD 22.1.8、SDK 10.0.26100.0、Cranelift 0.126.2、i5-13600KF参考机）。

## 10. 待澄清事项

无阻断起草的用户决定。T-01固定准确MIR版本/符号/私有ABI/产物与负载；D-798后T-02验证新codegen、多obj链接、缓存及正式runner全路径。若API不足、必须修改公共MIR或hard性能/内存不达，回规划，不抬预算、不缩负载、不改在途798。公开ABI/FFI及真正生产默认切换后续单列；不需要等待817或797全部完成才做本有限profile。

**2026-10-20 在案阻断（work记录）**：T-02..T-08全部等待D-798（798最终review/冻结门禁/实际合入/Spec/merge收据齐全；当前798=executing 1/10）。解除动作：D-798达成后，work以实际合入的798 API/Spec重读contract-decision.md §1/§2/§9（接口/指令/验证规则若变则以证据revision），再开 `D:/autostack/.wt/lang-818/auto-lang`（分支plan-818-dev）执行T-02首例。公共注册窗口在817/814–816先合入时按并行合同同步master后串行追加。
