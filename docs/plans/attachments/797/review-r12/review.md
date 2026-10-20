# PLAN-797 独立 final re-review R12

2026-10-20（Asia/Shanghai），**needs_fix / executing 3/9**。受审提交 `634d4d0abed97846f2e1c212104eba3581b47ca8`。上轮3项中 **F-797-R4-09 CLOSED**；**F-797-R4-07/10 PARTIAL/OPEN**。原具体输入已修复，但组合输入仍违反同一缓存/预算不变式，没有新增 finding ID 或提高规模承诺。

## 基线与独立性

- stage: review；review_scope: final；plan_id: PLAN-797；plan_revision: 1。当前Codex独立复审上下文，未实施本分支、未创建代理，具体模型/session ID未暴露。已核读R11后的根因/覆盖交接，再从代码、实际C/PE和实际分配重建结论，不把“3/3”或ALL PASS当验收。R4/R5/R8/R9/R10/R11之后，这是第七次完整final验收；就绪核查不混计。
- worktree `D:/autostack/.wt/lang-797/auto-lang`，branch `plan-797-dev`；reviewed_commit `634d4d0abed97846f2e1c212104eba3581b47ca8`，前轮 `26e307386661498d821af7fffba3a243cc218d79`，diff base `13367d127584461449ee2f151c0c5987f0740044`。入审/收尾HEAD、实现树clean、锁/配置/相关输入均一致，见[baseline](baseline.json.txt)、[end baseline](end-baseline.json.txt)。主检出HEAD亦保持 `4d7c5f5817cf5d40a34c1b837e663512ec7c8637`。
- dependency trees：ac-core `8e01299a76e4e9ad34ac61c8ce5302a84adea400`，ac-artifact-probe `d4c37faca3843b4a241f5f0770156a09efc23f65`，ac-sema-contract `13b4ac2821e1745b1d644e49932538ee6d642880`；auto-down sibling `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。D-796/D-788仍满足，双锁/依赖无变。
- rustc1.98.0、MSVC14.44.35207目录（banner19.44.35222）、SDK10.0.26100.0、sysroot rust-lld。本轮构建当前release worker/debug父入口；二进制/工具SHA见[tools-and-binaries](tools-and-binaries.json.txt)，仅进程内清空RUSTC_WRAPPER。
- 6份canonical模块Spec与树内相同且未漂移；goals主SHA `e2367955474da86d3c203f8354b0225d5633f86dafa12f0309467c5b8e5ed260`，树内 `ec5170b6…`，仍是已协调811历史条目差异，`touched_goals: []`原理由有效。
- SD SHA256 **`439fc7f0f34ef5d6ddac0f3889ea2f7588a39a14ad4c305574ad0a6276c96fa6`**；冻结[spec-delta](reviewed-spec-delta.md)、[support-matrix](reviewed-support-matrix.md)、[源码行](reviewed-source-excerpts.txt)。未修改canonical Spec/live ledger或实现。

## 定向验证与旧例闭合

Cargo均在原树，`--locked --offline`；没有整档`t/tv/tt/tb/tu/taa/tf`或完整verify。此轮重跑受影响准入/缓存/正式父路径及兼容；未重复 standalone 六场景210PE，也不据其旧数字认证新实现。

| 范围 | 实测 | 证据 |
|---|---|---|
| standalone `--test t02_contract_layout_admission --test t03_calls --test t04_resource -- --nocapture` | 23+7+6=36 pass | [准入、三腿、资源](scoped-native.log.txt) |
| standalone `--test t05_cache_turnaround abi_cache_` | 16 pass | [缓存](cache-tests.log.txt) |
| standalone `--test t06_binding abi_binding_` | 3 pass | [绑定](binding-tests.log.txt) |
| standalone `--test t06_binding abi_worker_` | 2 pass | [worker](worker-tests.log.txt)，并发及真实GC删除/rooted保活 |
| root `--features test-cases --lib tests::probe_adapter_tests::abi_ -- --nocapture --test-threads=1` | 11 pass | [父ABI](parent-tests.log.txt)，精确本计划家族 |
| `cargo test -p auto-test-case` | 87 pass | [契约兼容](contract-tests.log.txt) |
| root `--features test-cases --lib tests::test_cases_tests:: -- --nocapture` | 26 pass、1既有ignored | [runner兼容](runner-tests.log.txt) |
| scoped cargo fmt、root三文件rustfmt、git diff --check | pass | [probe fmt](probe-fmt.log.txt)、[root fmt](root-fmt.log.txt)、[空白](diff-check.log.txt) |

共 **181个不同定向测试pass、1 ignored**，见[verification-summary](verification-summary.json.txt)。本计划改动文件无自身warning，既有依赖/root告警不冒称全仓清零；未发现额外调试输出。独立探针另计，不混入pass总数。

所有独立Rust探针经[run-rust-probe.py](run-rust-probe.py)针对当前release库编译，真实cl编译、rust-lld链接和PE运行。命令在各`*-command.json.txt`，源和结果保全在本目录。仅写本计划target私有目录/store，不改实现/共享源、没有创建链接。重放使用新的scratch标签（runner第二参数），避免已有store/新文件污染初始状态。

| Finding | R12状态 | 旧例与当前边界 |
|---|---|---|
| F-797-R4-09（上轮残项P2） | **CLOSED** | span已移到写行完成之后；正式门消费该span；行内旧时点另存row_request_ms。真实失败false/true与写失败ERROR均保持正确。 |
| F-797-R4-07/P1 | **PARTIAL/OPEN** | R11 `common/outer.h → dep.h`遮蔽现真实miss、PE29；但与已支持的子路径include组合后，新增`common/sub/dep.h`仍cache17而fresh29。 |
| F-797-R4-10/P1 | **PARTIAL/OPEN** | 原1700字节名直链已拒，新增峰7,250,788B<8MiB；小图/短字段控制通过。但共享子图的size memo命中截短测得链峰，flatten完整路径仍超8MiB。 |

历史R1锁F03、R4-01/02/03/04/05/06/08/11/12、R5-01/02/03、R9-01、R10-01继续CLOSED。[旧缓存重放](cache-old-replay.log.txt)、[嵌套/无扩展名/合法重编](cache-siblings.log.txt)、[上轮传递头重放](cache-transitive.log.txt)、[上轮内存重放](layout-memory.log.txt)保存实际结果，承认旧例修复，不外推完整不变式已通过。

## F-797-R4-07/P1：传递头父目录仅顶层快照，仍漏子路径负依赖

AC-04，T-05。`cache.rs:160–168`称依赖父目录只列顶层即可覆盖子路径，但实际只有**已解析到的文件**父目录进入集合；新候选尚不存在时，其可能获胜的子目录没有被记录。`:313–315`对这类目录使用`recursive=false`，`:198–201`不进入也不记录子目录内容。

独立[cache-transitive-nested.rs](cache-transitive-nested.rs)和[输出](cache-transitive-nested.log.txt)：

```text
project/src/value.c         #include "../common/outer.h"; return VALUE
project/common/outer.h     #include "sub/dep.h"
project/common/sub/        初始已存在的空目录
project/include/sub/dep.h  #define VALUE 17
```

第一次真实编译PE17。仅新增`project/common/sub/dep.h`（VALUE29），source、outer.h、所有旧依赖内容以及源/-I两棵树不变。`common`的顶层文件清单仍只有outer.h，旧依赖父目录是`include/sub`，不会覆盖`common/sub`。

实际第二请求 **hit=true、PE17**，同一compile入口fresh得到 **PE29**；缓存producer总调用仍1。这是R10子路径与R11传递头组合，文件规模未增加、普通私有目录、无链接/工具链变更，违反原§5.4精确失效/未知依赖不可当纯函数缓存；不是新增C importer要求。

应覆盖真实候选搜索路径的负依赖，或对尚不能证明适用的输入保守不缓存。工具链遍历性能可优化，但不能把顶层快照称为搜索域全集。保留原例、组合反例、相同输入hit和合法重编控制，不再仅凭增加一个已观测目录的正例宣告闭合。

## F-797-R4-10/P1：size遍历的链峰不等于共享子图flatten路径峰

AC-01/04/05，T-02/T-05/T-06。原直链1700名已在终门拒绝，且实测新增堆峰低于8MiB；800名、小图长名、65536短字段控制继续正常。实际GC回收子项也保持通过。

当前`layout.rs:317–319`以size阶段`chain_acc.1`估flatten所需seen空间，`:355`加入终门；但`:402–403`命中memo后立即返回，不会把该子图的完整路径加入chain峰值。共享子图经短路径先算完、之后从更长路径引用时，size走查被缓存截短，flatten仍沿完整路径展开。图是有向无环图，复用不等于没有后续递归工作态。

独立[layout-memo-reuse.rs](layout-memo-reuse.rs)、[结果](layout-memo-reuse.log.txt)沿用GlobalAlloc/System计量，输入建好后设baseline，不计输入、64MiB执行线程栈或allocator额外开销：

- 共1024个记录，名字1700字节，records数组JSON **3,548,355B**。根按顺序引用8段；第一段127节点到u8，后续7段各128节点接回已计算的上一段，形成共享后缀。无循环、最长完整路径与原例相同（字段路径1023个点），输出8字段。
- size实际展开每段后就命中memo，测得链峰只有约一段；flatten最后一条路径仍穿越全部段。
- 实际 **accepted=true，新增堆峰8,993,472B > 8,388,608B**。800字节名合法对照接受，峰值4,385,472B。没有提高1024节点/1700名/深度规模，仅改变合法引用结构；也不声称该records数组经正式deck worker解析。

需预算实际flatten路径及其同时存活的全量memo/结果，而非size遍历被memo裁剪后的调用栈峰；可使用有界的子图路径成本或减少复制等方式，但修复由work决定。保留直链/共享子图/合法小图及真实峰值控制，最后返回Err或一个直链绿例不足以证明全路径预算。

## F-797-R4-09 CLOSED：发布完成计时与失败传播

`abi.rs:1081–1091`写行成功返回后，`:1092–1095`才关闭parent-request span。正式硬门按请求顺序消费这一完成后span，并保留行内判定时点为`row_request_ms`；不再以写行前值冒充完整口径。没有要求物理介质fsync，核验边界为当前合同的行写入完成。

本轮正式硬门 body-edit(edit+request) P50/P95/P99=**585/745/751ms**，reject P50/P95=**81/104ms**，均满足P95<1000ms。[61行正式样本](formal-gates.jsonl.txt)、[summary](formal-summary.txt)保全。完成后span相对行值差0–312ms（包含写行及期间调度等），均未倒置。

独立真实CLI[raw-row-controls.py](raw-row-controls.py)：合法期望1/0报告PASS、行true/true；错误期望2/0报告FAIL、行false/true且含失配文本。[结果](raw-row-control-results.json.txt)。从同一报告提取的完成后span与逐行时点一一对应，见[raw-span-boundary-results](raw-span-boundary-results.json.txt)。[写失败注入](raw-publish-failure.py)仍报告ERROR/exit2并给出request-evidence拒绝访问路径，见[结果](raw-publish-failure-results.json.txt)。计时、分类、传播子项均关闭，不继续挂旧问题。

## AC、规范与下一步

| AC → tasks | 结论 | 依据 |
|---|---|---|
| AC-01 → T-02 | partial/fail | 独立C布局/准入通过；F10预算仍失败。 |
| AC-02 → T-03 | pass（代表三腿） | 7项真实三腿/异质反控，保留有限target/声明IR边界。 |
| AC-03 → T-04 | pass（当前有限协议） | 6项真实资源/原方destroy/回滚/值校验/record mut-out控制。 |
| AC-04 → T-05 | fail | F07错误对象、F10资源上限未闭合；原GC/生命周期子项通过。 |
| AC-05 → T-05/T-06 | partial/fail | 正式完成后硬门/失败原始行通过；F10暂存/索引预算仍失败。 |
| AC-06 → T-06/T-07 | partial/fail | 真实绑定/语料/兼容通过；缓存/资源SD事实尚不能冻结。 |
| AC-07 → T-08 | fail | 两项OPEN，不能reviewed。 |
| AC-08 → T-09 | merge-owned pending | 未合入/沉淀/归档/guard清理。 |

SD-01/04/05的candidate、单target、非Auto源码前端及774分域方向正确；SD-02/03 **partial/fail**：搜索域全集和两阶段全路径峰值保证尚未满足。性能发布计时规则此次与实现一致。supersedes保留`docs/specs/auto-ac/project.md`、`docs/specs/auto-ac/data-resource-contract.md`；new保留`docs/specs/auto-abi/{project,windows-x64-v01,conformance-cache}.md`；`touched_goals: []`书面理由有效。没有改plan_revision/语义合同或发布canonical Spec/live ledger。

遗漏/延期/workaround扫描：两项均为原合同缺口的组合路径，登记不批准延期/缩面；没有新独立finding。旧例已修复、计时已闭合应如实保留，但不能把size memo峰或父目录顶层清单当完整oracle。现有先精确反例、后全冻结门禁的测试时序保持。

**next: work**，同树修F-R4-07/10、提交后再审。status executing，current_step3/9，T-01/T-03/T-04完成，T-02/T-05/T-06/T-07仍OPEN，T-08/T-09 pending。

**pending_closeout_gates**：最终代码/验收/Spec final pass后，merge在新冻结提交统一执行完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t和实际diff触面档；tf仅主检出到期批量单实例。门禁前不合入/宣称交付；规范/ledger沉淀、归档、wt-guard清理均pending。本轮没有实施修复或合入。
