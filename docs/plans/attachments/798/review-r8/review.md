# PLAN-798 R8 修复复审：needs_fix

2026-10-20；stage=review；review_scope=final（旧例闭合及受影响验收，未授完整最终pass）；plan_revision=2。延续本聊天的独立Codex主审上下文，未参与实施、未使用代理，未伪造模型/session身份。

## 基线与复现资料

- 实施原树`D:/autostack/.wt/lang-798/auto-lang`，branch=`plan-798-dev`，实际受审提交=`f9d9821577eefd0ccfb74530b7d1077923ea9c33`；repair_base=`36108624a432d50df65bda0725e66a205f56950b`；完整实现base=`13367d127584461449ee2f151c0c5987f0740044`。Work摘要仍写`6da9f510d`，本轮不据此复用证据，所有调用均绑定实际f9d982157。入口与结论前实施clean、HEAD/依赖/锁/配置/Spec稳定性见baseline.json.txt、final-baseline.json.txt。
- ac-core/ac-artifact-probe/ac-semantics-contract同树只读，auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean。主检出Plan输入身份见main-inputs.json.txt；SD冻结SHA256=`6c2720de8afd5147b2ec1142d1328a4f44064c7982e7f41bf55b740c3b1657d8`（sd-frozen.md）；当前contract-decision仍是`47683b1191bf960199a773a374764adf3b82fc3f4c131f88c6b1de4ec6c056aa`。
- 从仍完整保存的R7 probe复制到本目录，原样保留支持面/旧反例输入，仅适配native API新增的(src,caps)元数；新增`edge-probes`分支。独立manifest/源码/锁均在`probes/`，依赖仍直接指向同一实施树。沿用独立package名称plan798-review-r7不代表执行旧文件：命令的manifest始终为review-r8/probes/Cargo.toml，输入hash另存probe-identities.json.txt。
- 本轮先显式`cargo build --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --bin av_probe_worker --release --offline --locked`，再执行native/正式测试；不拿存在的旧worker当当前提交产物。实际worker和根测试exe hash见binary-identities.json.txt，worker-build.log.txt记录构建。这个审查前显式构建不会修复下述默认首次准备路径。

## 上轮5项的实际闭合边界

| R7 finding | 本轮核查 | 结论 |
|---|---|---|
| R7-01 native剩余截止 | fake前耗59000ms+真实12000ms迟滞linker，现约1013ms即Budget，helper已回收；正常native/输出帽/已过期控制通过 | 在途build超时旧例闭合；新worker准备路径无界、旧降格文字未撤销，R8-03开放 |
| R7-02前门 | bool/命名实参/括号链1/2/16成功，129/400/600均前门resource拒绝 | 深链旧例闭合；合法浅层宽参数被误拒，R8-01开放 |
| R7-03 sidecar | 缺/空/不覆盖Bin的映射拒绝；诚实除零映射保留func/span | 二元运算旧例闭合；一元负号溢出漏覆盖，R8-02开放 |
| R7-04 COFF节范围 | 原82字节raw pointer=u32::MAX输入拒绝；真实COFF/PE及新增窗口测试通过 | 本轮所验区间准入闭合，不外推成完整linker语义验证 |
| R7-05双return | 原程序AV=1、真实native exit1，落空对照亦双端=1；新增8组合测试通过 | 已检查终结/合流路径闭合 |

此前错键、错型参数、import执行门、解码放大、manifest改名、oracle兼容/健康等旧例也在本次新提交重放或指定测试族核实；没有重报已修的具体例子。Work对5项修复的说明存在，但前门仍用全局run累计、sidecar只枚举Bin、native只监督worker准备之后的阶段，覆盖边界尚未达到原合同。

## 当前开放findings

### F798-R8-01 [P2] 前门把兄弟实参中的操作数总量误判成构造深度，拒绝合法浅层调用

AC02/04/06，T02/05，V09；R7-02的完整不变量未闭合（修复后的相邻回归/误拒路径）。

driver.rs:1082..1102不再因命名参数冒号重置全局chain_ops，但也没有给每个兄弟实参建立独立表达式层级。任意多个浅层`aN: 1 + 0`因此被累加成同一条深链。前门预算是128层递归/构造深度，不是每次调用最多127个二元操作；原有限profile没有这个新宽度限制，现有签名/codec允许≤255参数。

实际输入由probe生成：`fn first(a0 int, ..., a127 int) int { return a0 }`，`main`调用`first(a0:1+0, ..., a127:1+0)`。128个参数的表达式各自只有一个加法、括号nesting=2。已有scalar前端/Checked HIR成功，绕过错误前门后真实lower→encode→verify_abc→AV返回I32(1)，387步；正常Driver却报`av.resource.frontend-nesting`，声称构造深度129。129/200参数同样误拒；16参数对照正常。全部远低于4MiB输入/表/步/帧等预算，证据edge-probes.log.txt。

修复应区分调用外侧串行运算链与实参列表中的兄弟表达式，以实际层级/最大构造路径计预算；保留R7的深链拒绝，增加浅层宽参数正例。不能把总操作数改名为“深度”或通过增阈值隐藏误拒。本项正是R7报告已要求的“保持合法低深度/宽参数输入可用”，不是新增规模承诺。

### F798-R8-02 [P2] sidecar只检查Bin，NegI32溢出仍可发布无来源trap

AC02/03/05，T02/04，V03/05/06/07；R7-03的完整来源不变量未闭合。

worker_restore.rs:225只把`MirInstr::Bin`当成trap-capable。冻结profile也包含`MirInstr::Un { op: NegI32 }`的MIN取负溢出；av.rs的un_op对此返回IntOverflow，必须同样具有当前来源。

独立最小输入：`fn main() int { var x = -2147483648\n return -x }`。正常Driver和诚实snapshot恢复均为`av.trap.int-overflow@func#0:45..46`。用真实store在另一请求snapshot下写入正确sidecar信封包裹的可解码空表，恢复仍`executed_ok=true`，trap=`av.trap.int-overflow@`。原来的二元除零反例现已正确拒绝；本输入不含Bin，所以新增覆盖循环完全漏过。证据edge-probes.log.txt。

应覆盖有限profile的完整可trap指令集合（包括NegI32），最好由共享分类定义驱动恢复检查/断言；按溢出、除零、除法/余数MIN/-1及一元MIN取负矩阵保留正常来源和缺映射反控。不能把“每个Bin有span”作为“每个可能语言trap有来源”的等价条件。

### F798-R8-03 [P2] native监督器之前的首次worker准备仍无界，且旧预算降格说明并未撤销

AC03/06/08，T01/05/08，V03/09；R7-01的准备/冷启动路径缺口（新监督器引入），另含知识收口缺陷。

native_reference.rs:235开始请求计时，:237先调用`native_worker_binary()`，到:249/:262才计算剩余预算并进入supervise。helper在默认release worker不存在时，:154..162用`Command::new(CARGO).args(build...).output()`构建：没有deadline、收容/杀树或输出帽。构建被锁/编译工具卡住或产生大量输出时，父调用可在监督器建立前无限等待/收集内存；完成后再检查60秒不能补救。worker文件已存在时又只检查is_file，不能自动证明适用于当前源码/新增子命令。

这是明确调用链的源审结论，本轮没有删除现有worker来伪造冷启动超时，也不声称实测等待超过60秒。旧“进行中的link剩余截止”反例约1.01秒被杀，认可已修；不能由热worker路径通过外推到首次准备。即使工具准备另列为冷态，也必须有限、单独计量和明确不在受测请求内，原计划禁止新增未受控启动器。

可复用正式adapter已有的有界准备机制，或要求调用前显式完成有界准备并绑定源码/依赖/二进制身份，让受测API只消费已准备的worker；不要在受测请求内做无界首次构建。补准备失败/被锁或迟滞/输出超帽及就绪成功控制，检查子树回收与失败传播。

此外work收据说“撤销r4(c)未授权澄清”，但本次13文件diff没有contract-decision，实际hash与上一提交完全相同；§6 r4(c)仍声称build≤60s是ac-core后续工作、不在本计划触面。该段没有标为已撤销，与新监督执行和原60/5/70合同冲突。SD/contract-decision必须同步真实有限行为，不把旧降格说明继续作为现行合同。此处是规范增量收口的一部分，未要求改变原验收预算。

## 实际测试与证据

| 检查 | 本轮结果 | 原始资料 |
|---|---|---|
| core lib | 20/20 | core.log.txt |
| 指定8个probe目标 | 62/62：abc13、binding4、cache14、mir8、native admission9、trace1、resources10、vertical3 | targeted.log.txt |
| 正式corpus/decode/负oracle/worker故障 | 4/4，明确skip formal_turnaround_gates | formal-controls.log.txt |
| 既有probe_adapter required兼容 | 16/16 | compatibility.log.txt |
| contract_v2指定测试目标 | 20/20 | contract.log.txt |
| 上轮probe及更早信任/缓存/预算控制 | 各行实际值见日志；旧例修复如上表 | old-probes.log.txt |
| 新增独立兄弟路径 | 浅层宽参数误拒；NegI32空sidecar丢来源 | edge-probes.log.txt |
| 在途native剩余截止 | fake前耗59000ms、真实12000ms迟滞helper，1013ms即Budget；helper无遗留 | native-budget.log.txt |
| 原分支native与AV对照 | 双return和落空对照均native exit1、AV=1 | native-branch-control.log.txt |
| 两package fmt / 全diff-check | 0/0；只见已有shim-metadata warning，未出现798新增warning | fmt.log.txt、diff-check.log.txt及构建输出 |

所有构建/测试从原树offline/locked执行。独立probe命令：`cargo run --manifest-path D:/autostack/auto-lang/docs/plans/attachments/798/review-r8/probes/Cargo.toml --offline --locked --target-dir experimental/ac-av-probe/target`，`-- edge-probes`/`-- native-budget`/`-- native-branch-control`为独立模式。native-budget沿用R7 slow-linker.rs，环境仅独立shell内设置`AV798_NATIVE_FAKE_ELAPSED_MS=59000`、`AC741_RUST_LLD=<本树target/debug/review-r8-slow-linker.exe>`、`P798_R7_LINK_DELAY_MS=12000`，调用后清除。

core/probe测试命令同manifest分别用`-p auto-av-core --lib`和`-p auto-av-probe --test mir_tier --test abc_semantics_tier --test cache_tier --test binding_tier --test native_admission_tier --test native_trace_parity --test vertical --test resources_tier`。根为`cargo test -p auto-lang --lib --features test-cases --offline --locked tests::ac_av_probe_tests:: -- --test-threads=1 --skip ac_av_probe_formal_turnaround_gates --nocapture`，兼容filter=`tests::probe_adapter_tests::`；合同=`cargo test -p auto-test-case --test contract_v2 --offline --locked`。

本轮已有合同内缺口，未再跑完整最终30样本V11/六场景release验收；work摘要“30样本绿”不替代本轮独立证据，旧提交性能也不自动覆盖新adapter/worker/前门。未跑根整档t/tv/tt/tb/tu/taa/tf/docs_gen。本轮87总数是work的完整workspace口径（包含未在指定8目标中执行的case_binding5），本轮实际执行为core20+probe62，不混算。

## AC / V / SD与路由

AC01最小vertical通过；AC02/04因R8-01 fail，AC03常规语义/真实native及双return通过但R8-02/03相关义务未闭，AC05因R8-02 fail，AC06因R8-01/03 fail且新V11 pending，AC07协议/兼容实际通过但完整资源/性能验收未获pass，AC08因R8-03的准备身份/合同材料未闭，AC09 fail，AC10 merge pending。

V01/02/04/05常规身份/codec/CFG/单元执行控制通过，来源子项=R8-02；V03一元负号trap自身正确但恢复来源缺失；V06..08常规cache/GC/重启通过、来源=R8-02；V09=R8-01/03；V10定向fuel/帧/取消通过；V11当前基线独立复验pending；V12实际协议/兼容通过；V13本轮COFF节/代际模型正负控通过。此前被修复的具体反例没有再作为开放finding保留。

SD01的惰性join实现认可；SD02的执行门/数值语义改进保留；SD03/04来源依然缺口；SD05深度与宽度混淆；SD06正常协议/兼容通过；SD07本轮COFF区间通过，但native准备与contract-decision文本未统一。supersedes/new路径有效，touched_goals=[]有experimental范围说明；SD冻结未改，canonical Spec/ledger未发布。

outcome=needs_fix。保留T01最小vertical、T03核心codec/执行门、T06语义绑定；T02/04/05/07/08及其完整验收保持重开，T09/10未完成，current_step=3/10。原树work针对三项不变量修复并提交，明确下次控制须同时覆盖“长串深链/宽参数浅层”“Bin/NegI32来源”“就绪worker/首次准备/失败准备”，不能再只补上轮原样输入。更新current合同文字并保留本目录复现材料。

pending_closeout_gates：最终pass后merge在新冻结提交统一根裸cargo t及编译器触面tv、独立workspace全套/native及796/788/实际已合入797兼容，其余档按最终diff；tf仅主检出到期单实例。工作阶段的定向验收及新基线V11不能后置冒称merge-owned。未改实施/Spec/ledger，未合入、归档或清理实施树。
