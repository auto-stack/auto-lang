# PLAN-828 r2 独立复审 R1

2026-10-20，`review_scope: final`，**outcome: needs_fix**。本轮是送审就绪核查及有界独立反例检查，不是完整验收通过。实现已明确承认正式 worker/IPC 与会话内存测量未完成；核查确认这些是 work-owned 必需项，不能移到 batch/后继。其余源码检查和反例也发现原合同内缺陷。

## 基线与边界

- 主计划读取自主检出，`plan_revision: 2`；读取时 SHA256 见 [baseline.json.txt](baseline.json.txt)，未更改语义合同或伪造 r3 授权。
- 实施树 `D:/autostack/.wt/lang-828/auto-lang`，`plan-828-dev`，冻结提交 `940445bf57674780bc6ff6c68928c0c7148a0dbf`；diff base `f79b1b83591310b45b3828dd300e6d368987d6c0`。23 文件，4687 additions/222 deletions。前后 `git status --porcelain` 均空，未写实施文件。
- reviewer：本聊天 Codex `/root`，未参与该实现、从计划/代码/原始断言重建结论；无模型或会话 ID 的额外身份声明。未创建其他代理或聊天。
- 同组 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，本轮未调用。依赖锁、清单、测试可执行文件、skill 及 Spec/Design 输入哈希见 baseline。Design40 在主检出原有未提交修改，按文件哈希记录，没有将其当已提交新授权。
- 所有问题都是既有合同缺口、支持面回归或交接缺陷；登记债务不批准延期。主检出其他既有 WIP 保留。

## 本轮验证

全部运行 cwd 为上述实施树；独立反例源码与日志持久存于主检出的本目录。

| 检查 | 命令/方法 | 实际结果 |
|---|---|---|
| 既有828定向测试 | `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828 -- --test-threads=1` | 17 passed，0 ignored，12.40s；含30修订进程内计时测试，但不能据此宣称正式链路/资源通过 |
| scalar差分 | `cargo test --manifest-path experimental/ac-syntax/Cargo.toml -p ac-syntax-core-adapter --test scalar_projection_ -- --test-threads=1` | **6** passed（清单写5组的描述非当前测试总数），0 ignored，0.02s |
| 独立反例 | `powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-r1/run-replay.ps1` | **7 failed / 7**，exit101，0.22s；[源码](replay.rs)、[原始输出](replay-output.txt)。断言按应有行为书写，失败表示缺陷复现 |
| 公开frontend脚本 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-function-incremental-828.ps1 -Scope frontend` | exit1，Cargo报告包名含空格，未运行该scope的测试 |
| 格式 | `rustfmt --check --edition 2021 --config skip_children=true` 后接 function_code/program_snapshot/session/formal_828/function_queries 五个新模块 | exit1；[完整格式差异](fmt-output.txt)，未自动修复 |
| diff健康 | `git diff --check f79b1b835 HEAD` | exit0 |

定向测试构建有 `shim-metadata/src/classify.rs:213` unused doc comment；该文件不在828 diff，作为既有告警记录，不将其归因给本计划。未运行整档 t/tv/tt/tb/tu/taa/tf 或 docs_gen。最终修复和审查通过后，merge 在新冻结提交上执行 AC-10 门禁；本轮无门禁提前通过声明。

独立 replay 使用当前定向测试产生的 rlib，不另建实施 checkout；脚本从 Cargo build 输出读取原 native link-search 路径。首次编译的跨度类型和 native 搜索路径已在 review harness 中修正；以上最终7例均成功编译并执行，失败不来自这些环境准备问题。

## Findings（全部 OPEN）

### F-828-R1-01 / P1：热前端仍是整 Program resolve/typecheck/Atom render-bind

AC-01/02/05，T-02/03/09。`probe/src/session.rs:863–889` 为全部函数克隆 stub AST，调用 `compile_program_parts`。`ac-core/src/source/adapter.rs:255` 起实际仍调用整 Program resolver、typechecker、Emitter::plan_module/render、descriptor::bind；`resolve.rs:674` 对每个函数执行 resolve_fn，`typecheck.rs:46` 对每个函数 check_block，adapter 多处对全部函数创建/渲染新节点。

这减少了未改 body 的遍历，却仍重建未改函数的语义桩/HIR并经过整 Program Atom 中转，违反§1目标2和§5.2明确禁令。`session.rs` 在调用结束后只给 changed_name 加计数，`whole_module_*` 没有在这些真实入口绑定，因此“未改全0”是错误归因，不是入口未调用的证明。需真实保留受检接口环境，建立 changed-body resolve/typecheck/HIR producer；计数绑定实际生产入口，并用真实强制全量反控验证。

### F-828-R1-02 / P1：安装重建全部未改执行代码 payload

AC-01/03/04，T-04/05/06。`core/src/program_snapshot.rs:59–138` 遍历每个 unit，`blocks.clone()`、重建所有 slots、克隆 spans、重映射所有 Const/Call，构造完整 `merged: MirModule`。`av::execute_program` 实际派发这个 merged，而非 Arc code 列表。

未改 FunctionCode 包装 Arc 相同不能证明实际执行 IR/代码相同；每次安装都复制和重建全体执行 payload，违背§2“未改代码payload/IR不复制重建”。允许更新根及绑定元数据不授权复制所有函数指令。需快照持有独立不可变代码单元，执行器通过受检绑定访问；身份/内容/代际断言须覆盖实际派发对象和 MIR，不能只看未被派发的包装对象。

### F-828-R1-03 / P1：合法递归体编辑被拒

AC-03/04/05/08，V-01/04/05，T-04/05/09。[replay `recursion_in_nonfirst_function`](replay.rs)：主函数在前、递归函数在中、末尾留 tail 满足词法收敛；冷源码返回3，冷编修改版返回6。仅把递归体 `+ 1` 改为 `+ 2` 时，`edit` 返回 `mir.unit-callee: a unit must address callees through imports`，没有新结果。

`hir_adapter.rs:228` 将自身引用编成 `CalleeRef::Func(0)`，而既有 unit 准入要求调用均经 imports。需与单元/链接合同一致地表示自身递归并验真实热编辑，覆盖前中后位置及递归/互递归代表组合。

### F-828-R1-04 / P1：前部编辑后未改函数仍返回旧来源位置

AC-02/05/06，V-02/05，T-02/07。[replay `unchanged_trap_location_after_prefix_edit`](replay.rs)：在 helper 中插入13字节语句和换行，未改 main 的除零 trap 仍为 `70..75`，相同新源码冷编为 `83..88`。现有 t07 span 测试只检查被修改函数内部，漏掉后续未改代码。

当前 span 直接保留在 immutable unit/merged，`line_col` 只将传入绝对旧offset放到新snapshot查表，没有修订映射。需按函数相对来源+所属snapshot映射诊断；重放旧snapshot、当前snapshot及内容命中A→B→A后的位移，保持代码payload不变。

### F-828-R1-05 / P1：PreparedEdit 可跨会话提交，源与执行结果不一致

AC-04/05/06，V-03/06，T-06。[replay `prepared_candidate_cannot_cross_sessions`](replay.rs)：A/B 均 revision0；A的 main=`helper()+10`，B的 main=`helper()+100`。将A准备的 helper→3候选交给B，`commit` 返回ok，B源码显示 `+10`，实际返回**103**（按该源码应13）。

`PreparedEdit` 只有 base_revision 和位置索引，`commit:985` 只比数字修订，未绑定会话、输入根、owner/配置/授权环境。需私有、不可伪造的会话/基面身份和完整适用性核对。另，prepare/commit没有取消/关闭状态，现有V-06仅测乱序和fuel，不覆盖合同要求的取消候选不能安装及关闭释放；须补真实事务控制和原例/相邻状态测试。

### F-828-R1-06 / P1：credential_current 不验证实际 FnEnv

AC-01/05/08，V-03/04，T-03/05/09。[replay `credential_must_bind_actual_environment`](replay.rs)：A/B helper参数数量不同，A凭证在B环境调用 `credential_current(&cf_a, cf_a.context_revision())` 仍为true。

`function_queries.rs:131–139` 仅查def_id存在并比较凭证与**调用方传入的字符串**，没有比对self的真实环境；CheckedFunction也未携带模块owner/capability绑定，FnEnv.owner可公开修改。已有测试只传入B的正确字符串，未测试使用A字符串对B消费。需由环境自身导出受检上下文身份并核owner/域/配置/能力/版本和实际依赖，不能让外部参数代替环境事实；后继公开接缝也须遵守。

### F-828-R1-07 / P1：恢复后的第一次编辑替换未改函数代码对象

AC-01/04/06/08，V-07，T-07/09。[replay `restored_unchanged_code_identity_survives_edit`](replay.rs)：真实落盘、关闭、restore两unit成功，值11；仅改helper后值12，但 main 在恢复前后快照的 `Arc::ptr_eq` 为false。

`restore` 先warmup产生一套 fns/history，再将磁盘恢复代码仅赋给 `session.current`（480），未同步rec.code/history。commit从rec收集codes，因此把未改函数恢复代码换回冷编对象。需使所有resident/快照/历史记录引用统一的受检恢复对象，并验证连续编辑与持有旧snapshot后的真实身份及回收。

### F-828-R1-08 / P1：旧标量 trace intrinsic 支持丢失

AC-05/08，V-03/09，T-02/03/09。[replay `trace_intrinsic_support_is_preserved`](replay.rs)：提供 `hir.test.trace.v1` 后旧 `compile_scalar_source` 接受 `mark_a()`，会话warmup却报 `def i_mark_a is not a plain function`。`session.rs:311` 把所有defs都要求为普通函数，查询和stub逻辑也未处理固有函数引入/移除。

需保留旧支持面和授权门，区分普通函数与受检intrinsic声明；加入冷/热/恢复正控、无授权负控和源序/短路trace对拍，不能只用无intrinsic的函数负载证明兼容。

### F-828-R1-09 / P1：绕过继承的输入准入和资源门

AC-05/06/07/08，V-06/09，T-01资源决策及T-06/07/08。[replay `session_preserves_frozen_input_cap`](replay.rs)：会话接受**4,194,333B**源码，超已有 `MAX_INPUT_BYTES=4,194,304`。旧Driver在parse前有真实size/nesting准入；新warmup直接compile_scalar_source，prepare也未将新源码/编辑量接入这些边界。

会话没有32MiB驻留/候选/旧代的统一计量和准入，默认syntax Budget和执行slot预算不能代替这些责任。需在分配/parse及候选构建前接真实cap，保留typed resource分类、当前可用根，验证边界两侧及资源失败释放；本轮未运行可能导致宿主栈溢出的深度探针，不能宣称其安全。

### F-828-R1-10 / P1：函数HIR A07与实际依赖/寿命消费未完成

AC-01/02/05/06/08，V-03/07，T-03/07/09。T-03仍未勾；contract-decision将A07延期至T-07，但T-07实际仅有A09单元/manifest，没有函数HIR codec/producer/恢复重验/统一引用。FnEnv只有接口字符串表，没有承诺的实际读取事实查询/变更集及body事实查询核心负控；不得以当前无CTE产品语法为由删除计划明确要求的依赖探针。

512测试仅history条目上限且全程保留rev0，没有真实store GC/lease、旧引用释放后Weak死亡或稳定水位证据、用户cache哨兵及关闭/异常释放矩阵。需完成这些原任务和区分性oracle；单纯Arc或有界map不能自证整个系统的回收/预算合同。

### F-828-R1-11 / P1：持久会话每次编辑重编码、重写全部函数

AC-01/03/06/07，V-01/07/08，T-04/07/08。`commit:1035` 每次调用 `persist_all_units`，该函数650起遍历全部fns、clone每个完整module、encode_abc并put_enveloped；未改函数产物IO和编解码非0，却没有记入每函数计数。普通formal用无store的warmup，完全绕开这条实际会话路径。

需只发布changed unit与必要目录/引用元数据，保持未改payload/IO为0；真实持久会话结构计量和反控必须覆盖此入口。还需给manifest真实required引用和恢复消费者；现manifest写入但restore不读取它，不能称已完成目录恢复和GC保活。

### F-828-R1-12 / P1：正式端到端、内存及反伪造门未交付

AC-06/07/08，V-08/09，T-08/09。`formal_828.rs` 仅进程内函数测试，无正式recipe/持续worker/父侧judge接线；coverage-manifest开放项3将其和A≤224MiB/B≤128MiB会话内存归batch/未来，违反§5.5。报告只保留内存对象，失败直接Err不留完整raw，缺输入/二进制/阶段wall/IO归因包；timer在结果比较之前停止，未计入声明的完整发布窗口。

反控仅手造 `FormalReport` 数值，未实际强制全量/全重装载/坏actual/资源越界/required skip，也无法发现F01/F02/F11。需完成真实正式入口、同轮≥30样本及失败留存、版本绑定/父侧比较、双线内存与继承预算；反控必须改变实际生产行为并让同一门非零。进程内P95通过不可替代这些原合同工作。

### F-828-R1-13 / P2：交付验证脚本不可运行且未防零测试

AC-08，V-09，T-08/09。`scripts/verify-function-incremental-828.ps1:31` 把`-p ... --test ...`存成一个字符串，63处作为单参数传给Cargo，实测包名非法退出1。即使拆分参数，额外frontend应从ac-syntax自己的workspace运行。`Invoke-ScopeTests`仅查exit code，没有兑现文件头“filter至少匹配一例”的断言；Cargo零匹配通常exit0。

另用正确拆分参数、原ac-av-probe manifest运行 `-p ac-syntax-core-adapter --test scalar_projection_ --no-run`，exit1：`cannot be tested because it requires dev-dependencies and is not a member of the workspace`。需用真实参数数组/正确manifest，验证每个filter的实际选择、数量与退出传播，保留合法scope控制及零选择/失败用例反控；不能改成吞错继续给PASSED。

### F-828-R1-14 / P2（格式子项P3）：交接和规范增量与现状不符

AC-08/09，T-09/10。主计划execution_done/current_step9与T-03空框冲突；work receipt还称T01..09全完成/无blocker。contract-decision §3仍写合成源码/旧入口，§9把已送审内容写成待未来收敛；successor合同声称restore计数证明零前端，实际restore先完整warmup。覆盖清单将必需面转future。SD-01..06仅规则摘要，未形成与当前已验证实现一致的具体规范文本；无可授信prepared Spec diff。

需修复实现后重整当前合同/证据/公共接口描述及具体规范增量，保持语义改变走正式修订；新Rust文件fmt检查修齐。不得通过改文案降低全链路增量、资源或正式验收要求。

## AC → tasks → evidence

| AC | 本轮结论 | 主要任务/证据 |
|---|---|---|
| AC-01 | fail | T02/03/04/07；F01/02/06/07/10/11；V01双函数部分成立，不能推广到全路径 |
| AC-02 | fail | T02/03；真实CST单声明投影和6组差分成立；F01/F04，缺A07 |
| AC-03 | partial/fail | T04/05；changed lower/ABC确实存在；F02/03/11 |
| AC-04 | fail | T05/06/07；原有旧/新snapshot暂停测试通过；F02/03/05/07 |
| AC-05 | fail | T02/03/05/07；F03/04/06/08/09/10 |
| AC-06 | partial/fail | T06/07；有history上限、旧snapshot保活；F04/05/09/10/12，释放/GC/内存未完成 |
| AC-07 | fail | T08；进程内30次双门测试通过，不覆盖F01/02/11；F12正式和资源缺口 |
| AC-08 | fail | T09；F03/06/07/08/10/12/13/14，必需oracle和消费者完整性缺口 |
| AC-09 | fail（本次needs_fix已记录） | T10；冻结输入和独立反例可复用，不授reviewed |
| AC-10 | pending / merge-owned | T11；独立pass后才执行裸t+tv、必要workspace/consumer门及沉淀/归档/guard |

V01部分正控通过；V02位置反例失败；V03上下文反例失败、body依赖/授权/版本未全；V04递归合法控制失败、其余完整恶意矩阵待修后复查；V05既有屏障通过但递归及错误来源缺口；V06乱序/fuel通过但跨会话、cancel/close缺口；V07恢复身份失败，GC/释放未证；V08进程内通过、正式/资源未交付；V09手造数据能红不等于生产退化/坏actual/资源能红。没有将未检查分支标pass。未做整个798兼容集的重复最终运行；本轮已有必需项未完成，不能用更多绿总数弥补。

## SD审查

[冻结的原始SD表](spec-delta-input.md)，SHA256 `d2cf61b3dd120e7e08edd9997877618df6e31fec0199a2effc9762cbfc42f383`；目标路径及操作类型可定位，`GOAL-002`和现frontmatter保留，不发布canonical或ledger。

SD01被F01/06/10阻断；SD02被F01/04/08/09阻断；SD03被F02/03/11阻断；SD04被F02/05/07/10阻断；SD05被F09/10/11/12阻断；SD06被F12/13阻断。现canonical仍准确把这些能力归828待交付，不能把“新增函数封装/计数器”写成完整已实现。修复后的具体文本须按当前Spec哈希核对并再次冻结。

## 路由

退回 **executing 0/11**，T-01首例本身也受真实粒度/资源归因问题影响，故重开T-01/02/04..09，T-03继续未完成；T-10/T-11保持未完成。已存在的实现和通过测试仍作为历史证据保留，不表示全部代码需重写；完整任务断言未通过，不能保留完成标记。原树work一次处理所有finding并重放7原例、相关合法控制和相邻组合，再提交新的完整候选；不得只修测试断言或将必需面记债延期。本轮未修实施、未合入、未归档、未清树。
