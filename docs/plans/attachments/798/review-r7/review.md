# PLAN-798 修复复审 R7：needs_fix

2026-10-20；plan_revision=2；stage=review；review_scope=final（修复闭合及相邻路径核查，非完整最终验收通过）。当前 Codex 主审上下文延续本聊天的独立复审，未参与实施、未使用代理，未伪造模型/session 身份。

## 冻结基线和证据范围

- 原实施树 `D:/autostack/.wt/lang-798/auto-lang`，branch=`plan-798-dev`，reviewed_commit=`36108624a432d50df65bda0725e66a205f56950b`；repair_base=`072cb4e14d9793b1990062acd043ff152295af39`；完整实现base=`13367d127584461449ee2f151c0c5987f0740044`。入口与结论前提交/工作树、依赖、锁/配置/Spec指纹见 baseline.json.txt、final-baseline.json.txt。实施clean；未修改实施。
- ac-core、ac-artifact-probe、ac-semantics/contract为原树只读路径依赖，最后变更提交见baseline；auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean。两份oracle已进入受检提交，本次实际兼容复验通过，不继续沿用“813在途缺件”的旧阻断。
- 共享Plan取自主检出，输入hash与主检出HEAD见main-inputs.json.txt。主检出在其他工作推进中移动，不等于实施漂移；五canonical Spec在主检出与实施树hash相同。SD-01..07冻结见sd-frozen.md，SHA256=`6c2720de8afd5147b2ec1142d1328a4f44064c7982e7f41bf55b740c3b1657d8`。contract-decision当前hash=`47683b1191bf960199a773a374764adf3b82fc3f4c131f88c6b1de4ec6c056aa`。
- 旧review-r6目录现只剩锁和work重放日志，源/报告/原始日志已不在目录；本次不将其work重放当作独立通过证据。根据本聊天此前保存的输入语义重建反例，存于probes/src/main.rs，并新增相邻组合。这里不声称重建文件与丢失文件逐字节相同。新源码、锁、原始输出全部存于本目录，后续不得用整理目录破坏复现材料。
- 独立probe直接依赖同一实施树，构建产物位于该树target，临时store均为PID独占；slow-linker.rs只延迟指定毫秒后exit9，不链接/执行任意外部内容。测试实际worker/根测试exe身份见binary-identities.json.txt。

## 旧8项修复的闭合边界

| 旧finding | 本轮核实 | 完整不变量状态 |
|---|---|---|
| F798-R6-01 cache身份 | A09使用真实原body_fp的bound信封放到另一body key仍拒绝；改名manifest拒绝；A08读写绑定路径源审及cache族通过 | 已检查范围闭合，不再报错值7/9旧例 |
| F798-R6-02执行凭证/错型参数 | 正确I32(5)成功，Bool→i32在执行门拒绝，trace import单元executable=false且dispatch前拒绝 | 旧例闭合，正常真实PE/trace族通过 |
| F798-R6-03恢复来源 | 缺sidecar拒绝，正确snapshot trap含func/23..28 | 部分闭合；空而可解码sidecar仍使无来源trap成功，见R7-03 |
| F798-R6-04前门 | `(1)+(1)+…` 128通过，129/400/600在门前resource拒绝 | 部分闭合；bool关键字与命名实参仍重置计数，见R7-02 |
| F798-R6-05 decode放大 | 原3,241,603字节输入在decode内拒绝，private bytes仅增28,672，不再先分配23MB对象；新增core预算测试通过 | 原例及本次检查的分配路径闭合；不把它外推成所有请求32MiB峰值的普遍证明 |
| F798-R6-06 native结构 | 原20字节伪头拒绝，真实COFF/PE正负控通过 | 部分闭合；节数据区间仍未验，见R7-04 |
| F798-R6-07 native截止 | 新61000ms伪钟控制只证明已过期时不启动lower/link | 仍开放；剩余截止反例见R7-01，不认可削弱合同的“澄清” |
| F798-R6-08 warning/兼容 | probe目标零新增warning；根798新增文件未再报新增warning；oracle实际兼容16/16 | 行为/健康子项闭合；acceptance.md仍写813未合入，当前资料需同步 |

Work的两连败诊断表已存在，值得保留；R7-01/02/03/04分别证明其覆盖调整仍只覆盖一个代表样本，没有覆盖同一不变量的真实入口/组合状态。新R7-05属于原支持面遗漏，不是本次修复引入的新回归。

## 开放findings

### F798-R7-01 [P2] 剩余build/total截止仍不覆盖正在执行的link，且“澄清”未获合同变更授权

AC03/06/08，T01/05/08，V03/09；延续F798-R6-07。

native_reference.rs:106..145新增伪钟，实际调用仍是`link::link_object_staged`，没有剩余时间参数；ac-core/src/link.rs:802继续给linker新的60秒。阶段开始/结束看钟不能中断其间的耗时操作；find_rust_lld/find_sdk也有各自子进程预算。run使用剩余时间不能倒推build已满足总预算。

本次使用**实现自身**的`AV798_NATIVE_FAKE_ELAPSED_MS=59000`模拟前阶段耗时59秒，用ac-core已有`AC741_RUST_LLD`覆盖到本目录slow-linker.rs构建的helper，令其延迟12000ms后exit9。实际新增墙钟=**12150ms**，helper完整执行，返回`Err(Link("rust-lld exit Some(9): ...12000ms"))`。这不是实测等待71秒：在本次实现自己的注入时钟下，总耗时为约71.15秒；应有的build剩余约1秒未被传递/执行，连total剩余约11秒也未覆盖。证据native-budget.log.txt。

contract-decision.md §6 r4(c)却把build≤60s改为依赖owner后续工作，并声称total≤70s硬性成立；共享Plan §3仍冻结60/5/70且plan_revision仍2。用户授权是修复全部问题，没有批准降格为事后检查。不能以ac-core只读为理由宣布完成：可以在798自己的受限worker边界覆盖整次native调用，或提出有证据的显式计划修订并取得范围决策；当前不得改写既定合同为通过。

修复应覆盖“开始时尚未过期、执行中越过剩余截止”、错误退出和清理，不只测61000ms已经过期的入口。现有反控没有证明它注释声称的link前阶段保护：61000会在lower前就退出。

### F798-R7-02 [P2] 关键字/命名参数的词法重置仍绕过同一128层构造前门

AC02/04/06，T02/05，V09；延续F798-R6-04。

driver.rs:1085..1093遇任意`:`或`Tok::Kw`即清零chain_ops。`true/false`是表达式操作数，不是语句边界；命名实参的`:`也在表达式内部。当前累计不是表达式构造深度。

独立正例`idfn(x:1)`调用链1/2/16项返回1/2/16。129/400项仍front_door_ok=true并返回129/400；600项通过前门后才报`mir.lower.depth` Unsupported。布尔`true && true ...`使用有正常后续return的形态（避免下述双return问题），1/2/16/129/400项均通过并返回1，600项同样在lower而非前门拒绝。既定128预算未改变，没有声称发生宿主崩溃。

应根据表达式结构及所在层级累计真实构造深度，区分bool字面量、命名参数、真正语句边界，并保持合法低深度/宽参数输入可用；不能为每类标点再追加一个会漏组合的全局run补丁。

### F798-R7-03 [P2] 可解码的空sidecar仍被当作有效当前来源，发布无来源trap

AC02/05，T02/04，V05/06/07；延续F798-R6-03。

worker_restore.rs:201..203只要求sidecar存在并能按字节长度解码，随后merge_spans；没有验证该映射能覆盖实际trap指令或对应当前源码。core verifier仅检查已有span排序/范围，不要求覆盖，所以空表也通过。

实际store正例trap=`av.trap.div-by-zero@func#0:23..28`；在另一请求snapshot下用正确sidecar信封写入`encode_sidecar(&[])`，真实restore_and_run仍`executed_ok=true, trap="av.trap.div-by-zero@"`。缺映射拒绝的旧例修好，但空映射等价地丢失诊断来源。

cache_tier.rs:672的新增正例只断言`trap.contains('@')`，无来源字符串也满足，不能区分承诺。应验证来源映射适用性/必要覆盖；无source不能派生时受控拒绝/unavailable。补空表、缺少trap指令、错范围及正常当前snapshot对照，断言真实func/span内容。

### F798-R7-04 [P2] COFF节数据/重定位范围仍未校验，包内表存在不代表可重定位对象有效

AC08，T08，V13；延续F798-R6-06。

native_admission.rs:254..294新增节表/符号表/字符串表边界，但从不遍历节头检查其SizeOfRawData/PointerToRawData及重定位表引用。

独立输入82字节：完整20字节COFF头+40字节节头+18字节符号记录+4字节字符串表；AMD64/1节/1符号，字符串表正常闭合；节头声明16字节raw data位于`u32::MAX`，实际完全不存在。按实际bytes mint内容hash，`admit_native_ref=Ok`。原20字节伪头已拒绝，本输入则沿同一结构准入路径穿透。

至少验证声明的代码/数据/relocation区间的checked边界与必需引用，保留真实Cranelift对象及真实PE对照。不是要求新增linker/loader，也不接受仅对最短伪头做长度特判。

### F798-R7-05 [P2] 有效的if/else双分支return被新增MIR adapter拒绝

AC02/03，T02，V02/04；原支持面的新发现，不是新需求或本轮修复回归（adapter在本轮diff未改）。

最小输入：`fn main() int { if true { return 1 } else { return 0 } }`。已有scalar前端/Checked HIR接受，独立expected=1；真实native参考成功exit1，Driver却报`mir-lower/lower.unwired-block: block 3 never received a terminator`。对照“两个分支给局部赋值，末尾return该局部”native及AV均返回1。证据native-branch-control.log.txt。

hir_adapter.rs:684..715对每个If无条件创建join并排入continuation；两分支已经终结时该不可达join无后续语句、无exit，最后在:770..784被当成lower错误。计划明确支持if/else及显式return，不得把合法控制流缩成只允许末尾统一return。修复应按可达性/终结状态生成join，验证双分支return、单分支return+落空、嵌套分支及loop内break/continue等相邻终结组合；保持所有实际产出的MIR块有效，不能把非法块放进不可达代码后跳过验证。

## 本次实际检查

| 执行 | 结果 | 证据 |
|---|---|---|
| core lib定向修复测试 | 20/20 | core.log.txt |
| 指定8个probe验收目标 | 57/57：abc13、binding4、cache13、mir7、native admission7、trace1、resources9、vertical3 | targeted.log.txt |
| 正式AutoDown corpus/decode/负oracle/worker故障 | 4/4；明确跳过formal_turnaround_gates | formal-controls.log.txt |
| required既有probe兼容 | 16/16，两个oracle实际消费通过 | compatibility.log.txt |
| contract_v2单测试目标 | 20/20 | contract.log.txt |
| 真实native新反例与对照 | 两者native exit1；双return的AV失败、赋值落空对照AV=1 | native-branch-control.log.txt |
| 剩余截止控制 | fake前耗59s+实际12.15s迟滞link仍完整运行 | native-budget.log.txt |
| 独立准入/恢复/深度/解码组合 | 按每行actual判断，admitted=true不是pass | probes.log.txt |
| 两package fmt/完整diff-check | exit0/0；仅既有shim-metadata warning，798新增warning未再出现 | fmt.log.txt、diff-check.log.txt及构建输出 |

命令均在原实施树、offline/locked。probe用`cargo run --manifest-path D:/autostack/auto-lang/docs/plans/attachments/798/review-r7/probes/Cargo.toml --offline --locked --target-dir experimental/ac-av-probe/target`；`-- native-branch-control`跑真实PE对照。native-budget使用该程序`-- native-budget`、前述环境变量及slow-linker helper；这些变量仅在独立shell进程设置，结束清除，不修改机器配置。

core用`cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-core --lib --offline --locked`；probe使用同manifest的`-p auto-av-probe --test abc_semantics_tier --test binding_tier --test cache_tier --test mir_tier --test native_admission_tier --test native_trace_parity --test vertical --test resources_tier`。根指定族使用`cargo test -p auto-lang --lib --features test-cases --offline --locked tests::ac_av_probe_tests:: -- --test-threads=1 --skip ac_av_probe_formal_turnaround_gates --nocapture`，兼容同形filter=`tests::probe_adapter_tests::`；合同=`cargo test -p auto-test-case --test contract_v2 --offline --locked`。

最初调查脚本误用保留字take作为函数名，已改为合法idfn并增加1/2/16项对照；probes-initial.log.txt只保留调查历史，不作产品缺陷证据。布尔链的最终对照使用普通落空路径，前门finding不依赖双return错误。

鉴于交接已承认native严格60s是后续owner工作、并存在上述in-contract缺口，本轮按readiness规则只做修复与受影响验收，**不再耗用完整最终性能/全部验收周期**。本轮未跑30样本formal硬门/六场景release重测，没有沿用work“静置重跑绿”作为独立pass；上一提交640.3/236.5ms不覆盖此次driver/信封改变。没有跑根整档t/tv/tt/tb/tu/taa/tf/docs_gen。

## AC / V / SD与状态路由

AC01最小vertical通过，完整native预算部分见R7-01；AC02/03因R7-05 fail；AC04因R7-02 partial/fail；AC05因R7-03 fail；AC06因R7-01/02 fail且新基线formal性能pending；AC07正式协议及required兼容本轮通过，但依赖完整资源/性能验收未获最终pass；AC08因R7-01/04及知识材料陈旧 fail；AC09 fail，AC10 merge pending。

V01/05的旧错键/凭证/解码样本通过，恢复来源子项=R7-03；V02/04=R7-05；V03正常trap/native通过但预算=R7-01；V06..08常规cache/GC/跨进程通过、来源=R7-03；V09=R7-01/02，旧放大反例关闭；V10常规fuel/帧/取消通过；V11当前提交本轮未执行；V12协议及兼容通过；V13 native结构=R7-04。没有把测试总数或旧例闭合替代完整不变量。

SD01被合法控制流遗漏阻断；SD02的执行门改进认可，完整语义仍依赖SD01；SD03/04来源仍未满足；SD05前门仍不完整；SD06的当前协议/兼容核实通过；SD07对象准入及budget描述仍不可定稿。supersedes/new路径有效，touched_goals=[]有experimental范围说明。canonical Spec与ledger未发布。acceptance.md/Plan §10还写“813未合入、缺oracle”，须跟实际输入同步；contract-decision r4(c)不得以“澄清”覆盖原冻结预算，plan_revision不因review簿记而更改。

outcome=needs_fix。保留T01最小vertical、T03本轮执行门/codec修复和T06真实语义绑定成果；重开T02/04/05/08及依赖完整资源/正式性能证据的T07；T09/10未完成，current_step=3/10。这是完整任务验收计数，不是删除已通过的实现/子项。原树work按5项不变量修复并提交；若确需改变冻结预算，走显式计划修订而非自行宣布关闭。禁止用整理附件删除本轮可重放证据。

pending_closeout_gates：最终pass后merge在新冻结提交统一根裸cargo t与实际编译器触面tv、独立workspace完整套件/native及796/788/实际已合入797兼容；其余档按最终diff；tf仅主检出到期单实例。工作阶段仍需完成受影响验收（含新基线V11），不得把未完成实现判作merge-owned。未合入、归档、删除worktree，也未修实施。
