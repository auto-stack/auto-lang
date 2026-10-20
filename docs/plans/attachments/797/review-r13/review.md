# PLAN-797 独立 final re-review R13

2026-10-20（Asia/Shanghai），**needs_fix / executing 4/9**。受审提交 `ae51fea21171ef14ae0630498fab3203318edbce`。上轮两项中 **F-797-R4-10 CLOSED**、**F-797-R4-07 PARTIAL/OPEN**；没有新增finding ID或提高规模要求。T-02布局验收本轮完成，T-01–04完成；缓存及相关终稿尚不能关闭。

## 基线、独立性与规范输入

- stage: review；review_scope: final；plan_id: PLAN-797；plan_revision: 1。当前独立Codex复审上下文，未实施本分支、未创建代理，具体模型/session ID未暴露。核读R12后的根因/覆盖交接后，从代码、实际C/PE、实际堆分配重建结论。R4/R5/R8/R9/R10/R11/R12之后，这是第八次完整final验收；就绪核查不混计。
- worktree `D:/autostack/.wt/lang-797/auto-lang`，branch `plan-797-dev`；reviewed_commit `ae51fea21171ef14ae0630498fab3203318edbce`，前轮 `634d4d0abed97846f2e1c212104eba3581b47ca8`，diff base `13367d127584461449ee2f151c0c5987f0740044`。入审/收尾HEAD相同、实现树clean，见[baseline](baseline.json.txt)、[end baseline](end-baseline.json.txt)。主检出HEAD保持 `40f2627cd93b9b6a98a47aae79776cc5210c208e`。
- dependency trees：ac-core `8e01299a76e4e9ad34ac61c8ce5302a84adea400`，ac-artifact-probe `d4c37faca3843b4a241f5f0770156a09efc23f65`，ac-sema-contract `13b4ac2821e1745b1d644e49932538ee6d642880`；auto-down sibling `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。D-796/D-788满足；双锁、依赖、验证配置和相关输入无漂移。
- rustc1.98.0、MSVC14.44.35207目录（banner19.44.35222）、SDK10.0.26100.0、sysroot rust-lld。本轮构建release worker和debug父入口，实际工具/二进制SHA见[tools-and-binaries](tools-and-binaries.json.txt)。构建仅进程内清空RUSTC_WRAPPER。
- 6份模块canonical Spec与树内相同且稳定；goals主SHA `e2367955474da86d3c203f8354b0225d5633f86dafa12f0309467c5b8e5ed260`，树内 `ec5170b6…`，仍为已协调811历史条目差异，`touched_goals: []`原理由有效。
- SD SHA256 **`e67557ba253d51ba5c19c1233c7ec4e4cb89bae88a6cf0b1cec519fc3a64366e`**，冻结[spec-delta](reviewed-spec-delta.md)、[support-matrix](reviewed-support-matrix.md)、[源码片段](reviewed-source-excerpts.txt)。未修改实现、canonical Spec或live ledger。

## 本轮验收

Cargo均在原树、`--locked --offline`，只执行定向验收/兼容；没有整档`t/tv/tt/tb/tu/taa/tf`或完整verify。

| 范围 | 实测 | 证据 |
|---|---|---|
| standalone `--test t02_contract_layout_admission --test t03_calls --test t04_resource -- --nocapture` | 24+7+6=37 pass | [准入/布局、三腿、资源](scoped-native.log.txt) |
| standalone `--test t05_cache_turnaround abi_cache_` | 17 pass | [缓存](cache-tests.log.txt) |
| standalone `--test t06_binding abi_binding_` | 3 pass | [绑定](binding-tests.log.txt) |
| standalone `--test t06_binding abi_worker_` | 2 pass | [worker](worker-tests.log.txt)，同store并发、实际GC删除及rooted保活 |
| root `--features test-cases --lib tests::probe_adapter_tests::abi_ -- --nocapture --test-threads=1` | 11 pass | [正式父ABI](parent-tests.log.txt)，精确本计划家族 |
| `cargo test -p auto-test-case` | 87 pass | [契约兼容](contract-tests.log.txt) |
| root `--features test-cases --lib tests::test_cases_tests:: -- --nocapture` | 26 pass、1既有ignored | [runner兼容](runner-tests.log.txt) |
| standalone `--test t05_cache_turnaround abi_turnaround_ -- --nocapture` | 1 pass，210样本全保留 | [六场景+拒绝](turnaround-tests.log.txt)、[summary](turnaround-summary.md)、[逐场景清单](turnaround-manifest.json.txt) |
| scoped cargo fmt、root三文件rustfmt、git diff --check | pass | [probe fmt](probe-fmt.log.txt)、[root fmt](root-fmt.log.txt)、[空白](diff-check.log.txt) |

合计 **184个不同定向测试pass、1 ignored**，见[verification-summary](verification-summary.json.txt)。本计划改动文件无自身warning；既有依赖/root告警不冒称全仓清零。未发现额外调试输出。独立探针不混入pass总数。

独立Rust探针经[run-rust-probe.py](run-rust-probe.py)针对当前release库编译，真实cl编译、rust-lld链接、实际PE执行。每次命令保存在对应`*-command.json.txt`，重放应给新的scratch标签（runner第二参数）。仅写本计划target私有源/store/产物，无实现修改或链接。

## 闭环与唯一开放项

| Finding | R13状态 | 依据 |
|---|---|---|
| F-797-R4-10/P1 | **CLOSED** | 第五张memo按子图记忆化完整flatten路径成本，终门不再使用被memo裁剪的size链峰。原1024节点1700名直链及共享DAG均在8MiB前拒；1300名共享DAG隔离终门通过拒绝控制；900名控制接受且实际峰低于8MiB，小图不误拒。 |
| F-797-R4-07/P1 | **PARTIAL/OPEN** | 原顶层/嵌套/无扩展名/传递头子路径均已修；但`../shared/dep.h`指向记录目录之外的候选，新文件仍不改变这些目录的递归清单，cache17而fresh29。 |

F-797-R4-09维持CLOSED（完成后计时、逐请求判断、写失败传播重验正确）；R1锁F03、R4-01/02/03/04/05/06/08/11/12、R5-01/02/03、R9-01、R10-01继续CLOSED。旧输入重放见[cache-old](cache-old-replay.log.txt)、[嵌套/无扩展名/合法重编](cache-siblings.log.txt)、[传递头](cache-transitive.log.txt)、[传递头子路径](cache-transitive-nested.log.txt)。旧输入关闭不等于缓存完整适用域已经证明。

### F-797-R4-07/P1：父路径候选不在已记录目录的后代树里

AC-04，T-05，影响T-06/T-07整体完成。`cache.rs:358–372`仍只从source目录、显式-I目录和**已成功读取依赖的父目录**生成快照；`:305–309`比较这些树的文件名，`:317–320`只检查旧依赖内容。递归现在覆盖了这些根的后代，但不是相对include的所有候选解析位置。

独立[cache-transitive-parent.rs](cache-transitive-parent.rs)，源SHA256 `8131e2326d47ffc0e6b1c2a9690b47e0c3e85ee22827c95407f79d8ebdf95029`；[实际结果](cache-transitive-parent.log.txt)：

```text
project/src/value.c                    #include "../external/headers/outer.h"; return VALUE
project/external/headers/outer.h      #include "../shared/dep.h"
project/shared/dep.h                  #define VALUE 17
project/external/shared/              初始已存在的空目录
显式 -I                              project/include/
```

初始头从`project/include/../shared/dep.h`成功解析，真实PE17。仅新增`project/external/shared/dep.h`（VALUE29），在包含者目录解析`../shared/dep.h`时优先获胜。source、outer.h、旧dep.h全部不变，新文件不在src、include、external/headers或旧dep父目录shared的任何后代树里。

实际第二请求 **hit=true、PE17**，相同compile入口fresh编译实际 **PE29**；缓存producer总调用仍1。全部文件位于同一私有project、普通目录，无链接/系统/工具链修改。这是有限候选probe里已经接纳的C相对include路径，未增加编译器/importer/平台或规模要求；原§5.4要求未知依赖不能当纯函数缓存。

本轮不新增ID，也不否认原子路径反例已经修好。剩余设计缺口是：**已解析文件的目录集合，不等于预处理器可能尝试的候选位置集合**；把已知目录递归更深仍不能覆盖父路径。修复需明确真实候选依赖/适用性如何获得与重验，或对不能证明覆盖的输入保守miss/不缓存。不能再仅增加一个目录/拼写例子便声明“搜索域全集”。建议固定同输入hit、内容变化、同目录/子路径/父路径遮蔽及未知适用域失败关闭矩阵，按同一不变式一次验收，而非缩减已批准合同。

### F-797-R4-10 CLOSED：共享子图完整路径预算

源检查确认`fpath`按每记录自身名字成本+最大子路径成本递推，memo命中时仍能读取完整子图值；新增第五张表的名字复制在插入前收费，终门计全量表、seen路径及结果表示。[layout-memo-reuse.rs](layout-memo-reuse.rs)沿用GlobalAlloc/System实际请求计量，输入建好后设baseline，排除输入、64MiB线程栈及allocator额外开销；实测：

| 图 | 结果 | 新增堆峰值 |
|---|---|---|
| 原1024节点1700名直链 | 受控StagingTooLarge | 7,872,356B |
| 原1024节点1700名、8段共享后缀 | 受控StagingTooLarge | 8,005,624B |
| 1300名共享后缀（size可过、终门拒） | 受控StagingTooLarge | 6,972,484B |
| 900名共享后缀 | 接受、8字段、路径深度1023 | 5,880,880B |
| 单记录2000名小图 | 接受、1字段 | 14,065B |
| 65536短字段叶控制 | 接受 | 3,672,332B |

均符合本轮覆盖的8,388,608B暂存/索引预算；原1950/2000名超限例与长字段名图也在超限前拒绝，见[完整内存重放](layout-memory.log.txt)、[共享DAG重放](layout-memo-reuse.log.txt)。永久24项准入/布局测试包含坏图、正反控制并通过。认可不变式设计调整，不外推无限规模或任意调用者输入内存也包含在8MiB内。

## 性能、原始证据与AC

六场景各30样本+descriptor-reject30，共210条，全部ok，见[summary](turnaround-summary.md)。standalone body-edit P95 **313.0ms**、reject **0.3ms**；它是临时等价native入口，C consumer沿原测试的固定输入helper缓存，不冒称它覆盖新的负依赖适用域或正式父IPC。

正式父路径另实测 body-edit(edit+request) P50/P95/P99=**594/683/719ms**，reject P50/P95=**79/84ms**，满足两个P95<1000ms硬门；[61行正式样本](formal-gates.jsonl.txt)、[summary](formal-summary.txt)保全。完成后span相对判定时点差1–3ms；[独立CLI时点对照](raw-span-boundary-results.json.txt)一致。真实CLI正反控制依然PASS/true-true、FAIL/false-true，写行失败ERROR/exit2，见[原始行结果](raw-row-control-results.json.txt)、[写失败结果](raw-publish-failure-results.json.txt)。不把性能通过抵销另一个输入的错误PE。

| AC → tasks | R13结论 | 依据 |
|---|---|---|
| AC-01 → T-02 | pass（有限profile） | 24准入/布局测试、独立C oracle及实际预算/合法控制；F10关闭。 |
| AC-02 → T-03 | pass（代表三腿） | 7真实三腿/异质反控。 |
| AC-03 → T-04 | pass（当前有限协议） | 6真实资源/回滚/值校验/record mut-out控制。 |
| AC-04 → T-05 | fail | F07错误缓存对象；其他生命周期/GC/预算子项通过。 |
| AC-05 → T-05/T-06 | pass（本轮性能/trace验收） | 新基线210场景原始样本及正式61行，完成后时点、资源、硬门/错误传播齐全；不表示AC04通过。 |
| AC-06 → T-06/T-07 | partial/fail | 绑定/退役/语料/兼容通过；缓存规范事实和终稿不能冻结。 |
| AC-07 → T-08 | fail | 一项P1 OPEN。 |
| AC-08 → T-09 | merge-owned pending | 未合入/沉淀/归档/guard清理。 |

SD-01/02/04/05的candidate、单target、声明IR/774分域及本轮预算规则与实现证据一致；SD-03 **partial/fail**：“搜索域全集”和完整适用性仍不能成立，性能规则通过。supersedes两条`docs/specs/auto-ac/{project,data-resource-contract}.md`及new三条`docs/specs/auto-abi/{project,windows-x64-v01,conformance-cache}.md`准确，`touched_goals: []`原书面依据有效。未改plan_revision/语义合同或发布规范。

遗漏/延期/workaround扫描：唯一开放项在原合同内，登记不批准延期/缩面；此前顶层妥协已取消、旧例及内存修复承认完成。不新增独立问题，也不以测试总数替代缓存适用性证明。

**next: work**，同树修F-R4-07并提交再审。status executing、current_step4/9；T-01/T-02/T-03/T-04完成，T-05/T-06/T-07仍OPEN，T-08/T-09 pending。T-06自身绑定/计时/兼容子项通过，但T-05缓存未闭，T-07规范整体也未过。

**pending_closeout_gates**：最终代码/验收/Spec final pass后，merge在新冻结提交统一执行完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t及实际diff触面档；tf仅主检出到期批量单实例。门禁前不得合入/宣称交付；规范/ledger沉淀、归档、wt-guard清理均pending。本轮只复审与簿记，未实施修复或合入。
