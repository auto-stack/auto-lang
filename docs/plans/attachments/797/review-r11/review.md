# PLAN-797 独立 final re-review R11

2026-10-20（Asia/Shanghai），**needs_fix / executing 3/9**。受审修复提交 `26e307386661498d821af7fffba3a243cc218d79`。上轮4项中 **F-797-R10-01 CLOSED**；**F-797-R4-07/09/10 PARTIAL/OPEN**。09的失败分类和写失败传播已闭合，仅保留既有计时发布边界残项；07与10的旧输入也已修好，但完整不变式仍有下述反例。没有新增 finding ID、提高规模要求或修改实现。

## 基线与独立性

- stage: review；review_scope: final；plan_id: PLAN-797；plan_revision: 1。当前独立Codex复审上下文，未实施本分支、没有创建代理；具体模型/session ID未暴露。先核读R10后的根因/覆盖调整和当前共享Plan，再从代码及实际运行重建结论，不用“4/4”或ALL PASS代替验收。这是第六次完整final验收（R4/R5/R8/R9/R10/R11），就绪核查轮不混计。
- 唯一worktree `D:/autostack/.wt/lang-797/auto-lang`，branch `plan-797-dev`；reviewed_commit `26e307386661498d821af7fffba3a243cc218d79`，前轮 `3bcc4195431bf17f493ff0d40487b93c6bf8efe5`，diff base `13367d127584461449ee2f151c0c5987f0740044`。入审/收尾HEAD相同、实现树clean、锁/验证配置/受审SD及模块输入不变，见[baseline](baseline.json.txt)、[end baseline](end-baseline.json.txt)。
- dependency trees：ac-core `8e01299a76e4e9ad34ac61c8ce5302a84adea400`，ac-artifact-probe `d4c37faca3843b4a241f5f0770156a09efc23f65`，ac-sema-contract `13b4ac2821e1745b1d644e49932538ee6d642880`；auto-down sibling `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。D-796/D-788满足，依赖及双锁没有修改。
- rustc1.98.0、MSVC14.44.35207目录（banner19.44.35222）、SDK10.0.26100.0、sysroot rust-lld。本轮构建release worker和debug父入口，实际二进制/工具SHA见[tools-and-binaries](tools-and-binaries.json.txt)；仅进程内清空RUSTC_WRAPPER。
- 主检出从 `668547845355806807e28c15cce9983efeef6d3b` 前进到 `9e80ae0fb6f907ab5b0bc5320c28c0fa6f75acbb`；本计划受审实现/依赖/相关规范输入未随之漂移。6份模块Spec主检出与树内一致；goals主SHA `e2367955474da86d3c203f8354b0225d5633f86dafa12f0309467c5b8e5ed260`、树内 `ec5170b6…`，仍为前轮已协调的811历史条目差异，未新增ABI产品goal，`touched_goals: []`有书面依据。
- 受审SD SHA256 **`aa8c61397b8ca708dd72a0d71750fb6850480106560366382a2a5bae9b8b18f7`**，冻结[spec-delta](reviewed-spec-delta.md)、[support-matrix](reviewed-support-matrix.md)；[相关源码行](reviewed-source-excerpts.txt)。未发布canonical Specs或改live ledger。

## 本轮实跑

全部Cargo在原树、`--locked --offline`，只运行受影响的定向验收/兼容；没有运行整档`t/tv/tt/tb/tu/taa/tf`或完整verify。

| 命令范围 | 结果 | 证据 |
|---|---|---|
| standalone `--test t02_contract_layout_admission --test t03_calls --test t04_resource -- --nocapture` | 22+7+6=35 pass | [准入/三腿/资源](scoped-native.log.txt) |
| standalone `--test t05_cache_turnaround abi_cache_` | 15 pass | [缓存](cache-tests.log.txt)，六场景210PE未重复，不复用其旧数字认证新实现 |
| standalone `--test t06_binding abi_binding_` | 3 pass | [绑定](binding-tests.log.txt) |
| standalone `--test t06_binding abi_worker_` | 2 pass | [worker](worker-tests.log.txt)，同store并发及实际GC删除/rooted保活 |
| root `--features test-cases --lib tests::probe_adapter_tests::abi_ -- --nocapture --test-threads=1` | 11 pass | [父侧](parent-tests.log.txt)，精确本计划家族，未把2个已有FFI测试混计 |
| `cargo test -p auto-test-case` | 87 pass | [契约兼容](contract-tests.log.txt)，30单测+57集成 |
| root `--features test-cases --lib tests::test_cases_tests:: -- --nocapture` | 26 pass、1既有ignored | [runner](runner-tests.log.txt) |
| scoped `cargo fmt`、root三文件rustfmt、`git diff --check` | pass | [probe fmt](probe-fmt.log.txt)、[root fmt](root-fmt.log.txt)、[空白](diff-check.log.txt) |

共 **179个不同定向测试pass、1 ignored**，见[汇总](verification-summary.json.txt)。构建有既有依赖/root告警；没有本计划改动文件自身告警，不冒称全仓零告警。独立反例不计入pass总数。

独立Rust探针由[run-rust-probe.py](run-rust-probe.py)针对本轮release库编译，调用真实cl、rust-lld、PE执行；输入/脚本/结果均保全在本目录。仅写本计划target私有产物/store，没有修改实现、符号链接或共享源。

## Finding closure 与 AC 映射

| Finding | R11结果 | 证据与边界 |
|---|---|---|
| F-797-R10-01/P1 | **CLOSED** | 新增未引用unused.h仍可合法重编PE17；无include同名同内容跨目录两个工程均成功PE17；永久同store并发控制通过。完整适用域已入generation键，无原Conflict死端。 |
| F-797-R4-07/P1 | **PARTIAL/OPEN** | 原顶层/嵌套/无扩展名/Unicode/跨工程/异工具链例修复；传递头所在目录超出两棵快照树时仍静默错用旧对象，PE17而fresh29。 |
| F-797-R4-09（原P1，本轮残项P2） | **PARTIAL/OPEN** | 原始行false/true与报告一致、写失败ERROR已闭合；request_ms仍在append前读取，硬门及“judged+persisted”收据没有包含本次写行耗时。 |
| F-797-R4-10/P1 | **PARTIAL/OPEN** | 原1950/2000字节名链在8MiB前拒绝、小图不误拒、真实GC回收通过；1700字节名合法链被接受，新增堆峰9,018,591B>8MiB。 |

历史R1锁F03、R4-01/02/03/04/05/06/08/11/12、R5-01/02/03、R9-01继续CLOSED，不因残项否认已完成修复。[旧缓存重放](cache-old-replay.log.txt)、[R10反例重放](cache-siblings.log.txt)保留合法控制。

| AC → tasks | 结论 | 本轮依据 |
|---|---|---|
| AC-01 → T-02 | partial/fail | 准入/独立C布局pass；F10资源上限未满足。 |
| AC-02 → T-03 | pass（代表三腿） | 7个真实三腿与异质反控pass；不外推全平台/Auto源码前端。 |
| AC-03 → T-04 | pass（有限资源协议） | 6个资源/回滚/值校验/record mut-out独立C控制pass。 |
| AC-04 → T-05 | fail | F07缓存错误、F10暂存/索引预算未闭合；R10-01和实际GC子项已过。 |
| AC-05 → T-05/T-06 | partial/fail | 正式门数字body-edit P95=619ms、reject85ms；失败原始行正确；仍有F09口径残项和F10内存反例。 |
| AC-06 → T-06/T-07 | partial/fail | 正式绑定/语料/契约兼容pass；SD关于完整缓存和资源/性能事实不能冻结。 |
| AC-07 → T-08 | fail | 3项OPEN，不得reviewed。 |
| AC-08 → T-09 | merge-owned pending | 合入/沉淀/归档/guard清理未做。 |

## F-797-R4-07：传递头的负依赖搜索目录仍缺失

AC-04，T-05。本轮`search_listing`的递归和无扩展名支持确实修复原例。但`cache.rs:291–295`仍只比较源目录与显式include目录两棵树，`:306–308`只复核已记录头文件的内容。真实预处理器的引用搜索还涉及**已包含头文件所在目录**，它不必位于这两棵树内。

独立[cache-transitive.rs](cache-transitive.rs)和[实际输出](cache-transitive.log.txt)：

```text
project/src/value.c        #include "../common/outer.h"; return VALUE
project/common/outer.h    #include "dep.h"
project/include/dep.h     #define VALUE 17
```

首次缓存编译得到PE17。仅新增`project/common/dep.h`（VALUE29），源/outer.h/旧include/dep.h均不变。这个新文件处于真实优先搜索位置，却不在src/include递归树里。第二请求实际 **hit=true、PE17**；同一`compile_c`入口重新编译实际 **PE29**，缓存producer总调用仍1。所有文件为同一私有工程的普通目录，未改系统/工具链、不用链接。

这是原§5.4真实依赖/精确失效不变式的传递include组合，未要求增加C importer或平台。应基于实际搜索过程保全足够负依赖/适用域，或对未证明覆盖的路径保守不缓存；不要仅把当前两个根的递归目录等同于完整搜索环境。保留现有嵌套/无扩展名、域隔离、合法重编、同输入hit控制。

## F-797-R4-10：flatten工作态与完整memo同时存活，增量预算没有覆盖该峰值

AC-01/04/05，T-02/T-05/T-06。认可R10旧例闭合：原1950/2000字节名链此次均在8MiB前受控拒绝；单记录2000字节名的小图已接受，峰值11,697B；65536短字段叶控制接受、峰值3,671,954B；旧长字段图在物化前拒绝。真实worker GC删除及当前rooted存活断言也通过。

但`layout.rs:516–518`在size阶段pop时回扣chain费用，阶段结束budget_used只剩四张memo。`:341–346`把该数加入物化门，未再加flatten的seen峰值；`:547`又逐层`name.to_string()`压入seen，此时四张完整memo仍全部存活。注释“chain峰值在同一预算内”忽略两个阶段的memo存活量不同，不能用先前已经释放/回扣的临时额度证明后阶段总峰值。

独立[layout-memory.rs](layout-memory.rs)与[结果](layout-memory.log.txt)，使用GlobalAlloc/System记录实际申请/释放，输入建好后设baseline（不计输入、64MiB执行线程栈、allocator额外开销）。沿用1024节点单链、一个u8最终字段，仅将旧名字**缩短至1700字节**：records数组JSON **3,536,223B**，字段向量及路径串2079B（不含输出record名），调用实际 **accepted=true，新增堆峰9,018,591B > 8,388,608B**。这已是超限下界，不是对输入加总或理论猜测。未提高原图规模、也未宣称该数组可经正式deck worker解析。

需使预算覆盖每阶段所有同时存活的工作态/输出，并在对应分配前拒绝或减少复制；正反控制应测实际峰值，包含“size阶段可过、flatten峰值越界”的合法近邻，而非只断言最终Err。

## F-797-R4-09：分类/传播已修，计时发布边界残项仍在

AC-05/06，T-06/T-07；本轮残项优先级P2，保留原ID。独立[raw-row-controls.py](raw-row-controls.py)：合法首次compute1/再次0，PASS、两行true；只把首次期望改为合法2，实际1，报告FAIL/exit1，首行false且含失配文本、第二行true。[完整结果](raw-row-control-results.json.txt)。[raw-publish-failure.py](raw-publish-failure.py)在私有产物目录把行文件位置设为目录，实际写入拒绝访问；报告 **ERROR/exit2**并给出request-evidence路径，见[结果](raw-publish-failure-results.json.txt)、[报告](write-failure-report.json.txt)。这两项明确CLOSED，不能继续说原始行将失败记成功或吞掉写错误。

保留的既有边界问题：`abi.rs:1076–1079`在append前关闭parent-request span；`:1093`在进入append前读取request_ms，而`:705–746`才构建/序列化行、建目录、打开并写文件。硬门消费的request_ms因此排除了本请求原始证据发布，尽管[本轮收据](formal-summary.txt)仍称span在“judged+persisted”关闭；Plan§5.5和SD-03§7均要求含发布/落盘完成。这是R10已指出的残项，非新增计时要求。

保存了[61行正式原始样本](formal-gates.jsonl.txt)，当前数字619/85ms不代表真实性能超1s；问题是度量与所声明边界不一致。应让门禁使用覆盖写行完成的实测值/证据，并明确原始行和完成后耗时如何关联，或走显式合同修订；不能仅改收据措辞来豁免既有发布要求。本轮仅据执行顺序与消费者代码判定该残项，不把源检查冒称已测慢盘超时。

## 规范、遗漏扫描与路由

SD-01/04/05的candidate/单target/声明IR/774分域方向正确；SD-02/03 **partial/fail**：精确工作态预算、完整C搜索适用性及发布计时仍未由实现满足。supersedes保留`docs/specs/auto-ac/project.md`、`docs/specs/auto-ac/data-resource-contract.md`；new保留`docs/specs/auto-abi/{project,windows-x64-v01,conformance-cache}.md`；`touched_goals: []`理由成立。没有降低已批准合同或增加plan_revision。

遗漏/workaround扫描：上述均在原合同内，不批准登记债务代替修复。R10两个死端、原始行失败分类、错误传播、旧超限/小图控制、真实GC已改善，应保留。部分失败计时口径不是新运行性能劣化 finding。未发现需要另增scope的独立阻断项。

**next: work**，同树修F-R4-07/09/10残项、提交后再审；T-01/T-03/T-04保持完成，T-02/T-05/T-06/T-07及T-08/T-09未完成，current_step3/9，status executing。

**pending_closeout_gates**：最终代码/验收/Spec final pass后，由merge在新冻结提交统一执行完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t与实际diff触面档；tf仅主检出到期批量单实例。未达这些门禁不得合入/宣称交付。规范/ledger沉淀、归档、wt-guard清理均pending。本轮未改实现、未合入。
