# PLAN-797 独立 final re-review R15

2026-10-20（Asia/Shanghai），**needs_fix / executing 4/9**。受审 `e00133e626a3d8a7857fcd0bff21c4b5b101658c`。R14三个具体输入已修复；F-R9-01身份错用关闭，F-R4-07仍PARTIAL/OPEN（产物名与产物路径混淆）；新增 **F-797-R15-01/P2**（单文件相对路径误拒）、**F-797-R15-02/P1**（旧v3同键升级发布Conflict），均为本轮修复回归。

## 基线与独立性

- stage review；review_scope final；PLAN-797 r1。当前Codex独立复审上下文，未实施本分支、未创建代理，具体模型/session ID未暴露。先核读R14后的根因/覆盖交接，再依据代码和实际运行重建结论。这是第十次完整final验收，前序R4/R5/R8/R9/R10/R11/R12/R13/R14；就绪核查不混计。
- worktree `D:/autostack/.wt/lang-797/auto-lang`，branch `plan-797-dev`；reviewed_commit `e00133e626a3d8a7857fcd0bff21c4b5b101658c`，前轮 `e1088b0adf56906bee7c6ce854347830d429c66e`，diff base `13367d127584461449ee2f151c0c5987f0740044`。入审/收尾HEAD一致、实现树clean，见[baseline](baseline.json.txt)、[end baseline](end-baseline.json.txt)。
- dependency trees：ac-core `8e01299a76e4e9ad34ac61c8ce5302a84adea400`，ac-artifact-probe `d4c37faca3843b4a241f5f0770156a09efc23f65`，ac-sema-contract `13b4ac2821e1745b1d644e49932538ee6d642880`；auto-down sibling `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。D-796/D-788满足，双锁/依赖/验证配置及相关输入未变。
- rustc1.98.0、MSVC14.44.35207目录（banner19.44.35222）、SDK10.0.26100.0、sysroot rust-lld；构建当前release库/worker和debug父入口，SHA见[tools-and-binaries](tools-and-binaries.json.txt)，仅进程内清空RUSTC_WRAPPER。
- 主HEAD从 `9aaeb42b56fa7af763d32baf9e44a97eeabb8652` 前进到 `deecfb1a6327d538f619e1e5a6e7338df517386e`，受审实现/依赖/相关主Spec哈希稳定。6份模块Spec主/树一致；goals主SHA `e2367955474da86d3c203f8354b0225d5633f86dafa12f0309467c5b8e5ed260`，树内 `ec5170b6…`，仍是已协调811历史条目差异，touched_goals[]原依据有效。
- SD SHA256 **`793a630b735f4aaff6677b9150c0335b4e2ff77a479d446bf4c179faa6ca1606`**；冻结[spec-delta](reviewed-spec-delta.md)、[support-matrix](reviewed-support-matrix.md)、[源码行](reviewed-source-excerpts.txt)。未改实现、canonical Spec或live ledger。

## 本轮实跑与闭环

Cargo均在原树、`--locked --offline`，没有整档`t/tv/tt/tb/tu/taa/tf`或完整verify。

| 范围 | 实测 | 证据 |
|---|---|---|
| standalone `--test t02_contract_layout_admission --test t03_calls --test t04_resource -- --nocapture` | 24+7+6=37 pass | [准入/三腿/资源](scoped-native.log.txt) |
| standalone `--test t05_cache_turnaround abi_cache_` | 21 pass | [缓存](cache-tests.log.txt) |
| standalone `--test t06_binding abi_binding_` | 3 pass | [绑定](binding-tests.log.txt) |
| standalone `--test t06_binding abi_worker_` | 2 pass | [worker](worker-tests.log.txt) |
| root `--features test-cases --lib tests::probe_adapter_tests::abi_ -- --nocapture --test-threads=1` | 11 pass | [正式父ABI](parent-tests.log.txt) |
| `cargo test -p auto-test-case` | 87 pass | [契约](contract-tests.log.txt) |
| root `--features test-cases --lib tests::test_cases_tests:: -- --nocapture` | 26 pass、1既有ignored | [runner](runner-tests.log.txt) |
| standalone `--test t05_cache_turnaround abi_turnaround_ -- --nocapture` | 1 pass、210样本全ok | [六场景](turnaround-tests.log.txt)、[summary](turnaround-summary.md)、[清单](turnaround-manifest.json.txt) |
| scoped fmt、root三文件rustfmt、git diff --check | pass | [probe fmt](probe-fmt.log.txt)、[root fmt](root-fmt.log.txt)、[空白](diff-check.log.txt) |

共 **188个不同定向测试pass、1既有ignored**，见[verification-summary](verification-summary.json.txt)。本计划改动文件无自身warning；既有依赖/root告警不冒称全仓清零，未发现额外调试输出。独立探针另列，不把输出证明失败的probe当通过测试计数。

独立Rust探针经[runner](run-rust-probe.py)针对当前release库编译、真实cl编译/链接/PE执行。命令保存在各`*-command.json.txt`。由于新增artifact_names参数，旧探针仅适配API并声明它实际创建的obj名；旧信封构造同步适配canonical路径，重新生成旧域列表，最终`run2`为4旧根/10新根，不以初次只筛到2根的构造作为覆盖证据。重放请使用新scratch标签（runner第二参数）；没有改实现或创建链接。

| Finding | R15状态 | 证据 |
|---|---|---|
| F-797-R4-07/P1 | **PARTIAL/OPEN** | config.tmp反例已正确29/miss，合法旧v3不再服务；但新白名单在所有目录按basename排除，仍误删真正头文件。 |
| F-797-R9-01/P1 | **CLOSED（身份错用）** | R14相对src/same.c+include跨cwd现A17/B29且B真实miss；绝对路径、异target/toolchain控制通过。单文件路径误拒另记R15-01，不混为错用旧对象。 |
| F-797-R15-01/P2 | **NEW/OPEN，repair regression** | Path("same.c").parent()==Some("")，canonicalize空串报Internal，producer零调用，fresh正常17。 |
| F-797-R15-02/P1 | **NEW/OPEN，repair regression** | v4拒绝v3但版本不入generation；适用域不变的合法旧v3占同键，实际重编后永久Internal Conflict。 |

F-R4-10内存与F-R4-09计时/原始行/错误传播维持CLOSED；其余R1锁F03、R4-01/02/03/04/05/06/08/11/12、R5-01/02/03及原R10-01旧触发控制也维持关闭。不能将原触发关闭外推为新发布/过滤路径无回归。

## F-797-R4-07/P1：声明了一个产物文件名，却排除了所有目录的同名文件

AC-04，T-05。`cache.rs:170–172`对每个目录项的basename与artifact_names做比较，递归时丢失“哪个文件才是本次输出”的路径身份；staged临时件前缀也未与产物位置绑定。当前规则不能证明其排除的都是实际产物。

独立[cache-artifact-path.rs](cache-artifact-path.rs)、[结果](cache-artifact-path.log.txt)：source为`src/value.c`，明确include **`sub/value.obj`**。初始真实文本头在`include/sub/value.obj`中定义VALUE17；本次编译器输出是另一个文件 **`src/value.obj`**，按新API如实声明`artifact_names=["value.obj"]`。随后新增`src/sub/value.obj`文本头（VALUE29）。

实际 **cache hit=true、PE17**，fresh **PE29**。新头与声明产物只是basename相同，路径不同；它由probe显式作为C头创建和引用，不是编译器产物。合法文件名已有接受面，未申请扩大C importer/平台/规模。

应以实际输出的完整路径/所属目录和临时事务身份过滤，而不能把“调用方声明了一个输出名字”变成“所有树中同名文件都不是依赖”的新假设。保留旧config.tmp、子路径/父路径遮蔽、合法同输入hit和1024上限保守控制。

## F-797-R15-01/P2：单文件相对路径被空父目录误拒

AC-04/06，T-05/T-06；新增修复回归。`cache.rs:335`仅在parent为None时回退到`.`。Rust对`Path("same.c")`返回的是Some空路径，导致`canonicalize("")`失败。

独立[cache-bare-source.rs](cache-bare-source.rs)、[结果](cache-bare-source.log.txt)：在私有工程cwd内有`same.c`和`include/dep.h`，调用`produce_c_object(..., Path("same.c"), Path("include"), ...)`。当前返回 **Internal("canonicalize src dir: ... os error 3")**，producer调用0；同一源/include直接fresh cl成功、PE17。`src/same.c`组件形式的R14输入已修，不能代表这个普通合法Path形式也通过。

源入口没有“必须带目录组件”约束，旧逻辑可到达producer，新canonicalize加入后才拒绝。应将空parent按当前目录解析并保留工程身份/同输入hit控制，而非要求调用者手工补`./`或缩减原输入支持面。

## F-797-R15-02/P1：旧v3同键时，正确拒旧却无法发布新v4

AC-04/06，T-05/T-06；新增修复回归。`:386`只接收v4；`:503–519`的generation材料虽加入artifact_names，但**没有信封/适用域算法版本**。当原v3使用的实际锚点和目录集合已等于新集合、且artifact_names为空时，合法新旧代占用同一key，payload至少因v字段不同而不同。store内容绑定正确拒绝覆盖；Conflict分支再发现时又正确拒绝v3，最终只能返回`:553` Internal。

独立[cache-v3-key-collision.rs](cache-v3-key-collision.rs)、[结果](cache-v3-key-collision.log.txt)：合法不含include的`value.c`返回17，obj在搜索树外scratch，artifact_names为空；缓存锚点使用std canonical路径，producer以cl支持的普通路径别名编译**同一文件**。先真实编译得到17对象及空观测依赖，再按旧v3算法/旧generation材料保存诚实的v3载荷到另一个私有store。该输入的两根目录已完整，旧规则没有遗漏依赖。

探针从两个store读取实际key_canonical并 **assert_eq!**，证明不是凭空给旧载荷捏造一个现行key。新查询实际重编（总调用2，包括初始对照），却得到 **Internal("generation key occupied by an unverifiable entry ...")**。初始真实PE17，源没有改变；预期应安全miss并成功发布17或受控不缓存继续交付，而不是永久失败。

初次直接把Win扩展路径交给cl时出现工具拒绝，未作产品finding；最终`run2`只对缓存使用canonical锚点，compiler读取相同文件的普通别名，实际编译成功后才触发本文Conflict。没有恶意对象/依赖篡改或任意伪造generation；这是版本升级必须隔离的旧合法状态。

需使版本/算法身份进入存储键，或使用其他能容纳旧新合法代的发布/迁移模型；不能只让发现拒绝旧格式而保留同一内容绑定key。新增“域改变的旧v3”绿例不能覆盖“域未变的旧v3”，两者应同时保留。

## 其余控制、性能与AC

- [config.tmp](cache-tmp-header.log.txt)、[旧v3窄域](cache-legacy-envelope.log.txt)、[相对cwd](cache-relative.log.txt)、[绝对域/异工具链](cache-old-replay.log.txt)、[父路径](cache-transitive-parent.log.txt)、[子路径](cache-transitive-nested.log.txt)、[合法重编](cache-siblings.log.txt)均正确。新上限反控实际1025目录include两次PE17/miss、artifacts0，见[cache-overcap](cache-overcap.log.txt)。
- 内存实现未变，本轮[共享DAG](layout-memo-reuse.log.txt)900名控制接受峰5,880,880B，1300/1700名拒绝峰均<8MiB；其余未变layout输入/依赖/配置上的R13计量可复用，24准入/布局测试本轮重新通过。
- 六场景各30+拒绝30共210样本全ok，standalone body-edit/reject P95=**189.1/0.2ms**，[summary](turnaround-summary.md)及7份JSONL已保全；不把旧固定consumer helper测试当新负依赖证明。
- 正式父路径 body-edit(edit+request) P50/P95/P99=**598/642/772ms**、reject P50/P95=**83/98ms**，满足两硬门；[61行正式样本](formal-gates.jsonl.txt)、[summary](formal-summary.txt)保全，完成后span相对判定时点差0–146ms（包括写行及调度）。真实CLI PASS/true-true、FAIL/false-true与写失败ERROR仍正确，见[判定结果](raw-row-control-results.json.txt)、[写失败](raw-publish-failure-results.json.txt)、[时点对照](raw-span-boundary-results.json.txt)。

| AC → tasks | R15结论 | 依据 |
|---|---|---|
| AC-01 → T-02 | pass（有限profile） | 独立布局/准入/坏图/预算控制保持通过。 |
| AC-02 → T-03 | pass（代表三腿） | 7项实际原生互调/反控。 |
| AC-03 → T-04 | pass（有限资源协议） | 6项资源/回滚/值校验/record mut-out。 |
| AC-04 → T-05 | fail | 错对象以及合法单文件/升级输入失败。 |
| AC-05 → T-05/T-06 | pass（当前性能/trace） | 210样本+正式61行、资源/判定/完成后计时/错误传播。 |
| AC-06 → T-06/T-07 | partial/fail | 真实绑定/退役/语料/兼容通过；缓存SD事实仍不能冻结。 |
| AC-07 → T-08 | fail | 3项OPEN。 |
| AC-08 → T-09 | merge pending | 未合入/沉淀/归档/guard清理。 |

SD-01/02/04/05有限candidate/target/声明IR/774分域与预算规则成立；SD-03 **partial/fail**：实际产物排除和可重证/可发布的版本化身份仍未满足。supersedes两条`docs/specs/auto-ac/{project,data-resource-contract}.md`、new三条`docs/specs/auto-abi/{project,windows-x64-v01,conformance-cache}.md`准确，touched_goals[]书面依据有效。未改plan_revision/语义合同或发布规范。

遗漏/延期扫描：以上是原合同下的实际路径/版本/合法输入问题，不批准登记债务代替修复；承认旧具体输入与身份错用已关闭，同时单列两个新修复回归，没有重置完整final次数。

**next work**，同树修F-R4-07、R15-01、R15-02并提交再审。status executing、current_step4/9；T01–04完成，T05/T06/T07仍OPEN，T08/T09 pending。

**merge-owned pending**：最终代码/验收/Spec pass后，在新冻结提交统一执行完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t和实际diff触面档；tf仅主检出到期批量单实例。门禁前不得合入/宣称交付，规范/ledger沉淀、归档/wt-guard清理pending。本轮未实施修复或合入。
