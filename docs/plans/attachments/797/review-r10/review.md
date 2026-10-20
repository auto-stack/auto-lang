# PLAN-797 独立 final re-review R10

2026-10-20（Asia/Shanghai），**needs_fix / executing 3/9**。受审修复提交 `3bcc4195431bf17f493ff0d40487b93c6bf8efe5`。上轮5项中 **2 CLOSED（R9-01、R4-12）**、3项PARTIAL/OPEN（R4-07/09/10）；新增 **F-797-R10-01/P1**（合法缓存失效/跨目录重编产生不可恢复Conflict，修复回归）。共4项OPEN。未修实现、未合入。

## 受审基线与独立性

- stage: review；review_scope: final；plan_id: PLAN-797；plan_revision: 1。
- reviewer_identity/limitations：当前Codex复审上下文延续前轮，未实施本分支、没有创建代理；具体模型/session ID未暴露。本轮从代码、真实C/PE和独立分配计量重建结论，不将“5/5”或ALL PASS作为验收依据。
- worktree `D:/autostack/.wt/lang-797/auto-lang`，branch `plan-797-dev`，reviewed_commit `3bcc4195431bf17f493ff0d40487b93c6bf8efe5`，上一基线 `34db55429c134fe0fd8cfaf502f606b956ef65c7`；diff base `13367d127584461449ee2f151c0c5987f0740044`。入审/收尾HEAD不变、实现树clean，受审输入无漂移，见[baseline](baseline.json.txt)、[end baseline](end-baseline.json.txt)。已读当前主Plan的根因/closure后进入本轮完整复审。
- dependency trees：ac-core `8e01299a76e4e9ad34ac61c8ce5302a84adea400`，ac-artifact-probe `d4c37faca3843b4a241f5f0770156a09efc23f65`，ac-sema-contract `13b4ac2821e1745b1d644e49932538ee6d642880`；auto-down sibling `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。D-796/D-788已满足，依赖/锁未改。
- rustc1.98.0，MSVC14.44.35207目录（banner19.44.35222），SDK10.0.26100.0，当前sysroot rust-lld。当前release worker/debug父入口的实际SHA与工具指纹见[tools-and-binaries](tools-and-binaries.json.txt)。构建仅进程内清空RUSTC_WRAPPER。
- 主检出HEAD `91a1b622d4aa588982e0006c98ae74148b9ebfa7`；6份模块Spec与树内一致。goals主SHA `e2367955474da86d3c203f8354b0225d5633f86dafa12f0309467c5b8e5ed260`、树内 `ec5170b6…`；仍是R9已经核对的811媒体域历史条目差异，不新增ABI产品goal，`touched_goals: []`仍有依据。
- 受审SD SHA256 **`8f89d804dae9ea644ec2c487ddf0456433ffc1c59d98e3d6b6cd1e043e525ab5`**，冻结副本[reviewed-spec-delta.md](reviewed-spec-delta.md)，支持矩阵副本[reviewed-support-matrix.md](reviewed-support-matrix.md)。相关代码行冻结于[reviewed-source-excerpts.txt](reviewed-source-excerpts.txt)。未改canonical Specs/live ledger。
- 这是R4/R5/R8/R9后的第五次完整final验收；没有重命名审查轮次掩盖失败次数。

## 本轮实跑

Cargo均在原树、`--locked --offline`。仅执行修复相关验收/兼容，不运行整档`t/tv/tt/tb/tu/taa/tf`或完整verify。

| 入口 | 实测 | 证据/边界 |
|---|---|---|
| standalone `--test t02_contract_layout_admission --test t03_calls --test t04_resource -- --nocapture` | 21+7+6=34 pass | [scoped-native](scoped-native.log.txt)：准入、原预算例、独立header、三腿、资源及mut/out控制。 |
| standalone `--test t05_cache_turnaround abi_cache_` | 11 pass | [cache-tests](cache-tests.log.txt)：旧include族、跨工程/异工具链/顶层遮蔽永久反例及缓存helper。六场景210PE未重复；本轮不据旧六场景数字认证新修复。 |
| standalone `--test t06_binding abi_binding_` | 3 pass | [binding](binding-tests.log.txt)：真实绑定、快照退出、typed deck。 |
| standalone `--test t06_binding abi_worker_` | 2 pass | [GC tests](gc-tests.log.txt)：同store并发小worker，以及新实际删除>0/当前rooted交付物存活的断言。认可真实GC子项修复，不把单worker删除与并发小负载合称已测任意并发删除组合。 |
| root `--features test-cases --lib tests::probe_adapter_tests::abi_ -- --test-threads=1` | 10 pass | [parent-tests](parent-tests.log.txt)：精确限定本计划ABI族，含语料、即时判定新门、预算及生命周期。 |
| `cargo test -p auto-test-case` | 87 pass | [contract compatibility](contract-compatibility.log.txt)：30单测+57集成，0例doc-test不计。 |
| root `--features test-cases --lib test_cases_tests -- --test-threads=1` | 26 pass、1既有ignored | [runner compatibility](runner-compatibility.log.txt)。 |
| scoped fmt/root三文件rustfmt/diff空白 | pass | [probe fmt](probe-fmt.log.txt)、[root fmt](root-fmt.log.txt)、[diff-check](diff-check.log.txt)；当前构建无本计划文件自身warning。 |

共 **173个不同定向测试pass、1 ignored**。完整摘要及自身warning检索见[verification-summary](verification-summary.json.txt)。独立探针不混入测试pass总数。

独立证据：[cache-old-replay.rs](cache-old-replay.rs)、[cache-siblings.rs](cache-siblings.rs)、[layout-memory.rs](layout-memory.rs)，以[run-rust-probe.py](run-rust-probe.py)对本轮release库独立编译/运行，所有C对象实际cl编译、rust-lld链接、真实PE执行。父负控[raw-row-controls.py](raw-row-controls.py)走当前auto-case/真实worker，完整命令和输入、报告、原始行可复跑。只写本计划target的私有源/store/产物，无实现修改。

## 闭环与AC

| Finding | 本轮结果 | 依据 |
|---|---|---|
| F-797-R9-01/P1 | **CLOSED（原身份错用）** | A/B同名同内容不同头：A=17、B=29且B真实miss；foreign-target/toolchain合法代不服务、触发一次真实编译；旧top-level shadow与Unicode路径控制正确。独立负控选用src_dir严格匹配当前A的合法载荷，避免由目录不匹配掩盖key轴。新合法域Conflict另记R10-01。 |
| F-797-R4-12/P2 | **CLOSED** | 原peak_job_memory_used/unused_mut两条消除，root/probe修改文件无自身warning，格式/空白通过。SD整体正确性仍随其他finding判断，不重复挂在本条。 |
| F-797-R4-07/P1 | **PARTIAL/OPEN** | 旧顶层`.h`增删遮蔽已修；嵌套路径与无扩展名include仍静默漏负依赖，真实cache17而fresh29。 |
| F-797-R4-09/P1 | **PARTIAL/OPEN** | 已即时typed判定、按请求落盘并记录完整资源；真实验收mismatch仍写`ok:true,error:null`，不能认证失败样本。 |
| F-797-R4-10/P1 | **PARTIAL/OPEN** | 原1950字节名链受控拒绝且此次峰值低于8MiB，实际GC删除/保活断言通过；预算仍在建memo后才检查，2000字节名的近邻合法链先超配再拒；极小图又被虚构最大深度预算误拒。 |
| **F-797-R10-01/P1** | **NEW/OPEN** | 新v2信封适用域与listing参与准入但不参与key，合法重编后Conflict无法找到新适用代，产生Internal错误。分类：repair regression。 |

历史R4-01/02/03/04/05/06/08/11、R5-01/02/03及R1锁F03继续CLOSED。相关永久测试本轮重跑；不以历史闭合外推新缓存/内存路径。

| AC → task | 结果 | 本轮依据 |
|---|---|---|
| AC-01 → T-02 | partial/fail | descriptor/名义准入/独立C布局pass；F10资源控制仍未过。 |
| AC-02 → T-03 | pass（代表三腿） | 7个三腿/异质反控pass，不外推全平台或Auto源码前端。 |
| AC-03 → T-04 | pass（当前有限协议） | 6个真实资源/原方destroy/回滚/值校验/mut-out正反控制pass。 |
| AC-04 → T-05 | fail | F07错误缓存、R10-01合法重编失败、F10预算未闭合。 |
| AC-05 → T-05/T-06 | partial/fail | 正式数字断言与判定时序改善；F09失败原始行、F10内存仍不满足合同。 |
| AC-06 → T-06/T-07 | partial/fail | 绑定/语料/共享合同兼容pass；准确SD/终稿尚不能冻结。 |
| AC-07 → T-08 | fail | 4项OPEN，不能reviewed。 |
| AC-08 → T-09 | merge-owned pending | 未合入/沉淀/归档/guard清理。 |

## F-797-R4-07/P1：负依赖快照只看顶层、过滤后缀，仍漏真实解析变化

AC-04，T-05。`src/cache.rs:154–178`的search_listing只列两目录的顶层普通文件，且只收`.c/.h/.hpp/.hh/.inc/.inl`。这不是完整C include搜索环境；C允许带子路径或无扩展名文件，本profile没有这些拼写的拒绝规则，当前实现也没有fail closed。

独立[cache-siblings.rs](cache-siblings.rs)及[结果](cache-siblings.log.txt)：

- `#include "sub/dep.h"`：两个搜索目录的`sub/`先存在，源侧无文件，最初从include目录得到VALUE17。新增源侧`sub/dep.h`=29，source及所有旧依赖不变，cache仍`hit=true`、PE**17**；同一cl命令fresh编译PE**29**。
- `#include "settings"`：无扩展名头同样从include目录17变为源侧29；文件名被后缀白名单排除，cache仍**17**而fresh**29**。

已关闭的顶层`.h`反例和合法旧输入控制同时保存于[cache-old-replay.log](cache-old-replay.log.txt)。这仍是§5.4真实闭包/未知依赖不能当纯函数缓存的原不变式，非要求扩大C语言面。需记录实际搜索候选/负依赖、或对无法覆盖的路径保守拒绝缓存；不能继续按几个正例增加文本/后缀白名单。

## F-797-R10-01/P1：失效后合法重编的适用域没有进入键，永久Conflict

AC-04/06，T-05/T-06；**修复回归**。新信封v2含src_dir/include_dir/target/toolchain/listing，当前准入确实使用它们，认可原错用修复；但`cache.rs:313–329`仍只用source_identity+observed-closure+target/toolchain生成key。新增适用域或listing变化而真实依赖闭包不变时，新旧信封不同、key相同。store的内容绑定正确拒绝覆盖；Conflict分支又只发现旧的、不适用代，于是永久Internal错误。

真实C/公共store输入（同一[cache probe](cache-siblings.log.txt)）：

1. 合法编译`value.c`依赖`dep.h`=17。只新增一个完全不被引用的`unused.h`，使listing变化；应仍可正确编译/运行17。当前实际真实重编后返回 **Internal("closure key occupied by a foreign entry...")**。
2. 两个工程各有同名同内容、**不含include**的`same.c`，都只返回17，共享store。A首次成功PE17；B的合法域不同、闭包仍空，同样在真实编译后返回上述Internal。与上轮“服务错误17代替29”不同，本轮是有效输入无法完成；未执行不兼容机器代码。

修复应把决定信封适用性的完整输入纳入generation身份，或使用能容纳不同合法域的索引/发布模型；不能靠Conflict后重新发现一个明确不适用的旧代兜底。保留同输入零编译hit、域变化/不相关目录变化、闭包相同的多工程、并发同输入发布控制，验证成功结果及失败传播。

## F-797-R4-09/P1：逐请求判定已提前，失败样本却继续标成功

AC-05/06，T-06/T-07。认可新流程在每请求响应后即时执行`judge_query_step`，并在下一请求前由适配器写原始行；body-edit的判定/IPC时序原缺口已明显改善。本轮正式硬门报body-edit P95 **629ms**、reject P95 **56ms**，61行原始样本保留于[formal-gates.jsonl](formal-gates.jsonl.txt)、[summary](formal-summary.txt)。不声称实际性能超1s。

但`test_cases/abi.rs:1017`把验收不匹配追加到mismatches，`1044`的append_request_row却固定传`error:None`；该函数`706`以`error.is_none()`计算`ok`。所以真实typed/cache/compute/stage验收FAIL也被写为成功。

独立[raw-row-controls.py](raw-row-controls.py)使用真实A腿、两个query。合法控制期待首次compute1、再次0，报告PASS且两行ok=true。反控仅把首次期望改成**合法值2**：父报告明确 **FAIL/exit1**，`expected 2, actual 1`；原始两行仍 **`ok:true,error:null`**，没有指出第一请求失败。完整证据[raw-row-control-results](raw-row-control-results.json.txt)、[wrong-compute报告](wrong-compute-report.json.txt)、[原始行](wrong-compute-raw.jsonl.txt)。首次误用999被合同正确拒绝，已保存[初次日志](initial-invalid-control-cli.log.txt)，改用2后才形成本文产品证据；该探针错误未记finding。

此外append函数将目录/打开/写入错误全部吞掉，request_ms在写行前计算；“judged+persisted”边界及原始证据发布失败传播仍需如实处理。主要阻断是成功/失败事实失真：须把本请求自己的mismatch/infra结果写入行，保留后续合法行独立分类；用真实错误oracle验证逐行false/true和总报告一致，再由硬门消费。不能只在全部正确的工作样本上断言ok。

## F-797-R4-10/P1：预算检查在memo分配之后，且用虚构最大深度误拒小图

AC-01/04/05，T-02/T-05/T-06。原R5长字段名图现在物化前拒绝（独立新增峰值1706B），短名65536叶控制峰值3,671,954B；原R9的1024节点/1950字节record名图也被拒，这次新增峰值 **8,275,038B <8MiB**。**认可这些旧例的闭合**。新实际GC删除>0、当前rooted交付物回收后compute0的永久断言也实跑通过，原“只有入口调用无实际回收oracle”子项已改善。

完整内存不变式仍未闭合：`layout.rs:306`先调用compute_size，建立全部4张String键memo及chain；`348–355`才估算index_bytes并拒绝。门不能回溯阻止已经发生的分配。

独立[layout-memory.rs](layout-memory.rs)继续用GlobalAlloc计量实际System请求，baseline设置在输入构建完成后；不计输入、64MiB线程栈或allocator额外开销。仅将旧图record名从1950加到**2000字节**，仍1024节点、单字段输出、records数组JSON **4,150,323B**：返回StagingTooLarge之前，新增堆峰值已 **8,479,888B >8,388,608B**。输入是layout库原有的合法有界图，不是声称正式deck worker可解析该数组，也没有提升4096深度承诺。输出见[layout-memory.log](layout-memory.log.txt)。这是“先超限再拒绝”，与8MiB暂存/索引运行上限不符。

相反，对**一个record、一个u8字段**、record名2000字节的小图，调用实际峰值只有 **11,568B**，却被估成16,593,171B而拒绝；20字节名控制通过。原因是`index_bytes`无论实际深度都按`MAX_NESTING_DEPTH+1`×max_name为chain/seen计费，引入了并未发生的最大深度占用。这是修复附带的有效小输入拒绝，不应靠缩短合法名字或提高预算解决。

应在分配/复制前用当前实际图、实际工作状态做有界预检或增量记账，或避免memo字符串副本；同时验证实际峰值与合理正例。只断言最后返回Err不能证明资源上限，“所有图都按最大深度预留”也不能充当真实资源超限的证据。

## 规范及路由

SD-01/04/05的candidate/单target/非Auto源码/774分域方向正确；SD-02/03仍 **fail**：声称预算分配前控制、完整适用域缓存/失效与准确失败原始行的规则尚不符合实现。保留supersedes两条与new三条真实docs/specs路径；依据当前主Spec协调后保留`touched_goals: []`及原书面理由。没有更改plan_revision/语义合同，也未发布canonical Spec/ledger。

**next: work**，同树修F-R4-07/09/10与新F-R10-01，保留原例与这些组合、合法控制。根因/设计和oracle应覆盖“准入域与存储身份的一致性”“实际分配时序”“每请求判定和证据一致性”，不能只把反例变成最后Err或增加一批正例总数。

**pending_closeout_gates**：最终代码/验收/Spec final pass后，merge在新冻结提交统一执行完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t与实际diff触面档；tf只主检出到期批量单实例。门禁前不得合入/宣称交付。规范/ledger沉淀、归档、wt-guard与清理均pending。本轮未改实现、未合入。
