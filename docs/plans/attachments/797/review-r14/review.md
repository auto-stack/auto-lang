# PLAN-797 独立 final re-review R14

2026-10-20（Asia/Shanghai），**needs_fix / executing 4/9**。受审 `e1088b0adf56906bee7c6ce854347830d429c66e`。上轮父路径反例在新建store上已经修复，1024候选上限分支也正确保守不缓存；但 **F-797-R4-07 PARTIAL/OPEN**（实际文本头被后缀过滤、旧v3域信封仍被接受），另 **F-797-R9-01 REOPEN/P1**（相对路径跨工作目录仍错用对象）。无新增finding ID；后者是原身份不变式的未覆盖输入，并非声称本提交新引入。

## 基线与独立性

- stage review；review_scope final；PLAN-797 r1。当前独立Codex复审上下文，未实施本分支、未创建代理；具体模型/session ID未暴露。已读R13后的根因/覆盖调整，再从代码、实际C/PE、持久化入口重建结论。这是R4/R5/R8/R9/R10/R11/R12/R13后的第九次完整final验收；就绪核查不混计。
- 唯一worktree `D:/autostack/.wt/lang-797/auto-lang`，branch `plan-797-dev`；reviewed_commit `e1088b0adf56906bee7c6ce854347830d429c66e`；前轮 `ae51fea21171ef14ae0630498fab3203318edbce`；diff base `13367d127584461449ee2f151c0c5987f0740044`。入审/收尾HEAD一致、实现树clean，见[baseline](baseline.json.txt)、[end baseline](end-baseline.json.txt)。
- dependency trees：ac-core `8e01299a76e4e9ad34ac61c8ce5302a84adea400`，ac-artifact-probe `d4c37faca3843b4a241f5f0770156a09efc23f65`，ac-sema-contract `13b4ac2821e1745b1d644e49932538ee6d642880`；auto-down sibling `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。D-796/D-788满足，双锁/依赖/验证配置稳定。
- rustc1.98.0、MSVC14.44.35207目录（banner19.44.35222）、SDK10.0.26100.0、sysroot rust-lld；本轮构建release worker/debug父入口，工具/二进制SHA见[tools-and-binaries](tools-and-binaries.json.txt)，仅进程内清空RUSTC_WRAPPER。
- 主检出从 `0aefa84ab733f6df7f09a66cef1f63349124402c` 前进到 `deb05632a9fdb4b4a7ff27dc18999577dc6077f8`，受审实现、依赖及相关主Spec哈希均未漂移。6份模块Spec主/树一致；goals主SHA `e2367955474da86d3c203f8354b0225d5633f86dafa12f0309467c5b8e5ed260`、树内 `ec5170b6…`，仍为已协调811历史条目差异，`touched_goals: []`理由有效。
- SD SHA256 **`d197ea52b9d4a2b625884a83226a357ec03088e5671b3e2bc71c1edaf9fd41cc`**；冻结[spec-delta](reviewed-spec-delta.md)、[support-matrix](reviewed-support-matrix.md)、[源码行](reviewed-source-excerpts.txt)。未修改实现/canonical Spec/live ledger。

## 本轮实跑

Cargo均在原树、`--locked --offline`；只执行受影响验收/兼容，没有整档`t/tv/tt/tb/tu/taa/tf`或完整verify。

| 范围 | 实测 | 证据 |
|---|---|---|
| standalone `--test t02_contract_layout_admission --test t03_calls --test t04_resource -- --nocapture` | 24+7+6=37 pass | [准入/布局、三腿、资源](scoped-native.log.txt) |
| standalone `--test t05_cache_turnaround abi_cache_` | 18 pass | [缓存](cache-tests.log.txt) |
| standalone `--test t06_binding abi_binding_` | 3 pass | [绑定](binding-tests.log.txt) |
| standalone `--test t06_binding abi_worker_` | 2 pass | [worker](worker-tests.log.txt) |
| root `--features test-cases --lib tests::probe_adapter_tests::abi_ -- --nocapture --test-threads=1` | 11 pass | [正式父ABI](parent-tests.log.txt) |
| `cargo test -p auto-test-case` | 87 pass | [契约兼容](contract-tests.log.txt) |
| root `--features test-cases --lib tests::test_cases_tests:: -- --nocapture` | 26 pass、1既有ignored | [runner](runner-tests.log.txt) |
| standalone `--test t05_cache_turnaround abi_turnaround_ -- --nocapture` | 1 pass、210样本全ok | [六场景](turnaround-tests.log.txt)、[summary](turnaround-summary.md)、[原始文件清单](turnaround-manifest.json.txt) |
| scoped fmt、root三文件rustfmt、git diff --check | pass | [probe fmt](probe-fmt.log.txt)、[root fmt](root-fmt.log.txt)、[空白](diff-check.log.txt) |

共 **185不同定向测试pass、1既有ignored**，见[verification-summary](verification-summary.json.txt)。本计划改动文件无自身warning，既有依赖/root告警不冒称全仓清零；未发现额外调试输出。独立探针输出包含明确产品反例，不混入pass总数。

独立Rust探针用[runner](run-rust-probe.py)针对本轮release库编译，真实cl编译、rust-lld链接、PE执行。命令在对应`*-command.json.txt`；重放须使用新scratch标签（runner第二参数）。仅写本计划target私有源/store/产物，不改实现/共享源，没有创建链接。

## 已认可的修复及保守分支

- [R13父路径反例](cache-transitive-parent.log.txt)本轮17→29且真实miss；[旧传递子路径](cache-transitive-nested.log.txt)、[顶层/无扩展名/合法重编](cache-siblings.log.txt)、[绝对路径跨工程/异工具链/Unicode](cache-old-replay.log.txt)均正确。
- 新[上限反控](cache-overcap.rs)用真实source include 1025个不同目录的头，cl观察全部依赖。两次请求都成功PE17、均`hit=false`、真实编译次数2、store artifacts=0，见[结果](cache-overcap.log.txt)。认可派生集合过大时不缓存，没有将资源状态伪装成hit。
- F-R4-10维持CLOSED：本轮[共享DAG实际堆计量](layout-memo-reuse.log.txt)900名控制接受（峰5,880,880B），1300/1700名在8MiB前拒绝；24个准入/布局测试通过。其余R13直链/长字段/小图原始计量可复用：layout实现及其依赖、配置、输入均未变化；不冒称本轮重新运行了未执行的独立探针。
- F-R4-09维持CLOSED：真实CLI合法PASS/true-true、错误期望FAIL/false-true，写行失败ERROR/exit2，见[原始行结果](raw-row-control-results.json.txt)、[写失败结果](raw-publish-failure-results.json.txt)。[时点对照](raw-span-boundary-results.json.txt)与完成后span一致。

## F-797-R4-07/P1：候选完备性仍被后缀过滤和旧信封适用性破坏

AC-04，T-05，影响T-06/T-07整体完成。认可新三元组派生修复了上轮父路径，不能据此认定原缓存不变式全闭。

### 实际被包含的文本头被当作编译产物过滤

`cache.rs`的`is_artifact`仍按后缀无条件排除`.tmp/.obj/...`及含`.tmp-`名字，而不是只排除本次编译器/链接器实际创建的产物。合法C include并无该文件名限制，当前入口也没有在不能证明适用性时拒绝缓存。

独立[cache-tmp-header.rs](cache-tmp-header.rs)将旧合法同目录遮蔽形态的文件名改为 **`config.tmp`**：source含`#include "config.tmp"`，初始从-I读到VALUE17。新增source旁的`config.tmp`文本头（VALUE29），所有旧依赖内容不变，候选目录也已经在快照中。实际 **hit=true、PE17**，fresh **PE29**，见[结果](cache-tmp-header.log.txt)。该文件由测试作为C头显式创建和引用，不是编译/链接输出；单按后缀排除它不满足“仅排除编译产物”的原规则。这是已有任意拼写依赖/精确失效规则，不是扩展C importer或平台。

### 加强域算法仍接受旧v3的较窄域

持久化信封仍`v==3`；发现路径只比较**载荷已有的**`dir_listings`，不检查它是否包含当前算法要求的候选目录。新候选计算只发生在miss后。旧版合法v3因此可以绕过本次修复。

独立[cache-legacy-envelope.rs](cache-legacy-envelope.rs)在R14私有store先真实编译17，再用完全相同object/编译器观测依赖/源/target/toolchain，按R13源算法恢复旧域与旧generation哈希（源/-I/全部观测依赖父目录，共4根；新域12根），经公共store API保存合法旧v3。随后新增R13那个父路径遮蔽文件29。结果 **hit=true、PE17，fresh29，producer总调用1**，见[结果](cache-legacy-envelope.log.txt)、[命令](cache-legacy-envelope-command.json.txt)。新建store的R13反例通过，旧信封输入却仍失败。

这是语义加强后的持久化兼容缺口，不是任意伪造信封：key hash、载荷域及内容依照旧实现生成，没有篡改obj/dep/hash。初次模拟曾漏一个因路径分隔符不同的父目录，已弃作证据；最终源码直接从`deps`取全体父目录，使用新scratch `run2`重跑，最终4旧根/12新根。另有不控制历史来源的现存私有store探针返回正确29，只作为正向控制，不用于证明旧格式完全覆盖。

应将真实被包含文件纳入适用性（或明确保守不缓存），并让旧域信封失效/重新证明当前候选集合。只对新miss生成更大的集合，不能修复已经持久化的旧错误假设。保持父路径、同输入hit、合法重编和超限不缓存控制；登记不批准缩面/延期。

## F-797-R9-01/P1 REOPEN：相对目录字面量未绑定实际解析工程

AC-04/06，T-05/T-06。分类：原合同路径遗漏；不声称由本轮113行改动新引入。旧绝对路径跨工程、异target/toolchain输入本轮仍正确，重开仅因为完整身份不变式对普通相对路径仍失败。

`src_dir`取`src.parent().display()`、`include_dir`取原始display，未解析为实际目录，也没有绑定当前工作目录。`rel_spec`对相对锚点不能推导，但该分支仅跳过推导，并不转为不缓存。公共入口接受这些Path，cl也正常编译；没有经批准的“仅绝对路径”前置条件或受控拒绝。

独立[cache-relative.rs](cache-relative.rs)在一个私有探针进程内，依次切换两个工程cwd（不改外部终端/测试进程cwd）。两工程都有相同内容的`src/same.c`（include dep.h），各有`include/dep.h`，A=17、B=29，共享绝对路径store。每次以相同的`Path("src/same.c")`和`Path("include")`调用同一公开缓存入口；所有输入均经fresh cl合法执行。

实际A **17/miss**；切到B后 **17/hit**，fresh B **29**，缓存producer总调用仍1，见[结果](cache-relative.log.txt)。源内容相同，两个域的字面量都成了src/include；旧A依赖绝对路径仍存在且哈希相同，载荷据此服务A对象给B。

修复应以实际解析目录/当前工程绑定适用域，并明确处理相对锚点推导失败。保留绝对/相对路径、同输入hit及两cwd共享store的语义控制；不能只缩小公开Path输入的隐含支持面或只增加某一种绝对拼写。

## 性能与AC/任务

六场景各30+拒绝30共210样本全ok，standalone body-edit/reject P95=**307.5/0.3ms**。[summary](turnaround-summary.md)及7份原始JSONL已保全。此临时native入口沿原固定consumer helper缓存，不用其证明负依赖适用域正确。

正式父路径 body-edit(edit+request) P50/P95/P99=**589/739/929ms**，reject P50/P95=**87/114ms**，满足两硬门；[61行正式样本](formal-gates.jsonl.txt)、[summary](formal-summary.txt)保全，完成后span比行内判定时点晚1–10ms。认可性能/trace与错误传播通过，不用这些正确输入的数字抵销缓存错误PE。

| AC → tasks | 结论 | 依据 |
|---|---|---|
| AC-01 → T-02 | pass（有限profile） | 准入/独立C布局/坏图和预算控制，F10保持关闭。 |
| AC-02 → T-03 | pass（代表三腿） | 7项真实三腿/异质反控。 |
| AC-03 → T-04 | pass（有限资源协议） | 6项真实资源/回滚/值校验/record mut-out。 |
| AC-04 → T-05 | fail | F07的文件/旧域重验、F9相对路径工程隔离失败。 |
| AC-05 → T-05/T-06 | pass（当前性能/trace） | 210样本及正式61行、资源/完成后计时/错误传播。 |
| AC-06 → T-06/T-07 | partial/fail | 绑定/退役/语料/兼容通过；适用域与缓存SD事实不能冻结。 |
| AC-07 → T-08 | fail | 两项P1 OPEN。 |
| AC-08 → T-09 | merge pending | 未合入/沉淀/归档/guard清理。 |

SD-01/02/04/05的有限candidate/target/声明IR/774分域及预算规则仍成立；SD-03 **partial/fail**：“候选全集”及“适用域完全一致”的保证尚未满足。supersedes两条`docs/specs/auto-ac/{project,data-resource-contract}.md`和new三条`docs/specs/auto-abi/{project,windows-x64-v01,conformance-cache}.md`准确，`touched_goals: []`书面依据有效。没有改plan_revision/合同或发布canonical Spec/live ledger。

遗漏/延期扫描：不因输入从未被旧测试选中或问题在上轮已存在而豁免原合同。已认可旧例修复与超限保守分支；没有增加新ID或额外产品/规模承诺，两个问题均应回work修复。

**next work**，同树修F-R4-07、重开F-R9-01，提交再审。status executing、current_step4/9，T-01–04完成；T-05/T-06/T-07仍OPEN，T-08/T-09 pending。

**merge-owned pending**：最终代码/验收/Spec final pass后，在新冻结提交统一跑完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t和实际diff触面档；tf仅主检出到期批量单实例。门禁前不得合入/宣称交付，规范/ledger沉淀、归档/wt-guard清理pending。本轮没有实施修复或合入。
