# PLAN-797 独立 final re-review R9

2026-10-20（Asia/Shanghai），**needs_fix / executing 3/9**。受审 `34db55429c134fe0fd8cfaf502f606b956ef65c7`。R8开放五项中，F-R4-02本轮关闭，F-R4-07/09/10/12仍PARTIAL/OPEN；另新增F-797-R9-01/P1（缓存重写引入的身份准入回归）。**共5项OPEN**，不得挂债收口。保留T-01/T-03，完成T-04；T-02/T-05/T-06/T-07仍未完成，T-08/T-09 pending。

## 基线、范围与独立性

- stage: review；review_scope: final；plan_id: PLAN-797；plan_revision: 1。
- reviewer_identity/limitations：当前Codex复审聊天，延续前轮审查上下文，未实施本分支、没有创建代理；具体模型/session ID未暴露。没有把work的“5/5”或总数当验收结论。
- 原树 `D:/autostack/.wt/lang-797/auto-lang`，branch `plan-797-dev`；reviewed_commit `34db55429c134fe0fd8cfaf502f606b956ef65c7`，上轮 `bf2455758e074cced103d7da5245b0991a3e3e8f`，diff base `13367d127584461449ee2f151c0c5987f0740044`；入审/收尾HEAD不变、实现树clean、受审输入无漂移。详[baseline](baseline.json.txt)、[end baseline](end-baseline.json.txt)。工作交接声称work-owned全部完成，已先核验根因与oracle调整后进入完整复审。
- dependency trees：ac-core `8e01299a76e4e9ad34ac61c8ce5302a84adea400`，ac-artifact-probe `d4c37faca3843b4a241f5f0770156a09efc23f65`，ac-sema-contract `13b4ac2821e1745b1d644e49932538ee6d642880`；auto-down sibling `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。锁/配置未变，D-796/D-788已满足。
- rustc1.98.0，MSVC14.44.35207目录（banner19.44.35222），SDK10.0.26100.0，当前sysroot rust-lld；工具与实际worker/parent二进制SHA见[tools-and-binaries](tools-and-binaries.json.txt)。构建使用进程级空RUSTC_WRAPPER，未改系统/Cargo配置。
- 主检出HEAD `0a8273b2926e6b968348c71d6e58895bf9a734de`。6份模块Spec与工作树相同；`goals.md`主检出SHA `e2367955474da86d3c203f8354b0225d5633f86dafa12f0309467c5b8e5ed260` 与树内 `ec5170b6…` 不同，差异为已合入811媒体域的GOAL-003/007历史条目，无新ABI产品目标；据当前主Spec重新核对，`touched_goals: []`理由仍成立。这是已协调的输入漂移，不另造实现finding，最终沉淀应使用当前主检出输入。
- 受审SD SHA256 **`023768e8a2514cfd9f71c817e9947525cdb2c10c3437cfdccfc65c41e7b59463`**，冻结于[reviewed-spec-delta.md](reviewed-spec-delta.md)；支持矩阵副本[reviewed-support-matrix.md](reviewed-support-matrix.md)。关键代码冻结摘录[reviewed-source-excerpts.txt](reviewed-source-excerpts.txt)。本轮不改canonical Spec/live ledger。
- R8辅助工件已被其他簿记事件清除，Plan中留有事件和重建报告。本轮不伪称复用了那些已丢失日志：原长字段名输入及新实现的兄弟控制均重新构造/实跑，代码与命令保存于本目录。本轮是继R4/R5/R8后的第四次完整final验收。

## 实跑清单

全部Cargo在原工作树，`--locked --offline`。仅定向验收，不运行整档`t/tv/tt/tb/tu/taa/tf`或完整verify。

| 入口 | 实测 | 证据/覆盖 |
|---|---|---|
| standalone `--test t02_contract_layout_admission --test t03_calls --test t04_resource -- --nocapture` | 20+7+6=33 pass | [scoped-native](scoped-native.log.txt)：header/descriptor独立表、旧payload准入、原长名字预算永久例、真实三腿、资源生命周期与新mut/out正反控制。 |
| standalone `--test t05_cache_turnaround abi_cache_` | 8 pass | [cache-tests](cache-tests.log.txt)：8形态真实include17→29、系统头入闭包、512-churn/helper等。未重复六场景210PE，不能用其旧数字认证新的C缓存实现。 |
| standalone `--test t06_binding abi_binding_` | 3 pass | [binding](binding-tests.log.txt)：真实semantic绑定、退役、typed deck。 |
| standalone `--test t06_binding abi_worker_concurrent_gc_same_store` | 1 pass | [concurrent-gc](concurrent-gc.log.txt)：同store两worker并发/8个query通过；没有`ran/deleted`断言，不能认证回收执行，详F10。 |
| root `--features test-cases --lib abi_ -- --test-threads=1` | 12 pass | [parent-tests](parent-tests.log.txt)：10个本计划ABI测试+2个已有FFI ABI测试；其中dep-parity的a2r腿按原开关未启用，不把该腿算已执行。本计划正式硬门/预算/生命周期/语料均实跑。 |
| `cargo test -p auto-test-case` | 87 pass | [contract compatibility](contract-compatibility.log.txt)，30单测+57集成；0例doc-test不计。 |
| root `--features test-cases --lib test_cases_tests -- --test-threads=1` | 26 pass+1既有ignored | [runner compatibility](runner-compatibility.log.txt)。 |
| scoped probe fmt、三份修改root文件rustfmt、diff空白 | pass | [probe fmt](probe-fmt.log.txt)、[root fmt](root-fmt.log.txt)、[diff-check](diff-check.log.txt)。 |

合计**170个不同定向测试pass、1 ignored**；其中2个为已有FFI测试，不能冒充本计划新覆盖。根自身warning仍有2条，见F12。没有将反例脚本exit0计入pass总数。完整结果与warning摘录见[verification-summary](verification-summary.json.txt)。

独立补充程序：[cache-probes.rs](cache-probes.rs)、[layout-memory.rs](layout-memory.rs)，以[run-rust-probe.py](run-rust-probe.py)从本轮release库独立编译，分别运行`python .../run-rust-probe.py cache-probes unicode-run`与`python .../run-rust-probe.py layout-memory`，原生命令/输入/输出均在相应`*-command.json.txt`、`*.log.txt`。它们只写本计划target中的私有测试store/源/产物，不改产品实现或安装工具。

## 闭环表与AC

| Finding | 本轮结果 | 旧例闭合 / 完整不变式 / 修复回归 |
|---|---|---|
| F-797-R4-02/P1 | **CLOSED** | descriptor/3有三out、two_step handle归属、callback ctx/i64视图，header独立表及t02的20例通过；新增record mut/out真实C正反控制通过。修复只是如实描述既有C桥/补齐承诺面，非扩大Auto源码支持。 |
| F-797-R4-07/P1 | **PARTIAL/OPEN** | 旧8种预处理拼写及系统头内容观察已修、Unicode路径兄弟控制也17→29正确miss；但新增近端header改变搜索结果时旧manifest仍hit，见下文。 |
| F-797-R4-09/P1 | **PARTIAL/OPEN** | 已计父IPC/等待/解码，消费者复用与独立cold输入正确；计时仍不含最终typed判断/发布，body样本仍等31请求全返回后才落盘，失败/资源行不齐。 |
| F-797-R4-10/P1 | **PARTIAL/OPEN** | 原长字段名输入现在受控拒绝；短名合法控制成功。memo/index本身未入预算，合法小输出链新增堆峰值>8MiB；并发GC的实际回收oracle仍缺。 |
| F-797-R4-12/P2 | **PARTIAL/OPEN** | 原response_opt/exit_ok/hot_bytes三警告消除，格式/版本/旧数字同步改善；新增2个自身warning仍未处理。 |
| **F-797-R9-01/P1** | **NEW/OPEN** | 新generation发现路径忽略source目录与当前target/toolchain/flags；实际跨工程复用了错误C对象。分类：repair regression，见下文。 |

R4-01/03/04/05/06/08/11、R5-01/02/03以及R1锁F03历史CLOSED保留；本轮重跑了其相关永久测试，未用旧已丢工件扩大覆盖结论。

| AC → tasks | 结果 | 当前依据 |
|---|---|---|
| AC-01 → T-02 | partial/fail | descriptor/header/布局/语义准入pass；F10暂存及索引预算未过。 |
| AC-02 → T-03 | pass（代表三腿） | 修改后7个真实三腿/异质反控通过；不外推全部平台/源码前端。 |
| AC-03 → T-04 | pass（当前有限资源协议） | 6个资源用例含handle边界、原方destroy、回滚、UTF-8/值校验和新增record mut/out正反控制。 |
| AC-04 → T-05 | fail | F07真实依赖失效、R9-01身份准入、F10索引/GC证据仍有缺口。 |
| AC-05 → T-05/T-06 | fail | 数值断言绿，whole-path计时/逐样本/完整资源oracle仍不满足F09/F10。 |
| AC-06 → T-06/T-07 | partial/fail | 正式绑定/语料/共享合同兼容pass；准确支持声明/SD及自身warning未过。 |
| AC-07 → T-08 | fail | 5项OPEN，不能reviewed。 |
| AC-08 → T-09 | merge-owned pending | 本轮未合入/规范沉淀/归档/guard清理。 |

## F-797-R9-01/P1：缓存generation发现跳过真正的输入身份

AC-04，T-05/T-06；**修复回归**。`src/cache.rs:112–119`注释称源码身份包含文件位置以区分本地include，但实际只用`file_name()+content_hash`。`183–216`按`%source=...,target=%`枚举generation，只检查manifest.source和旧dep内容，**没有检查当前target/toolchain/flags与key一致**。

独立真实C控制（[cache-probes.log](cache-probes.log.txt)）：

1. 工程A和B都有同名同内容`same.c`：`#include "dep.h"; value(){return VALUE;}`。A头为17，B头为29，头各留在本项目、都可读。A首次编译PE17；B请求返回`hit=true`，累计编译次数仍1，PE也**17**，正确应29。两项目source_identity逐字相同。A的旧header仍存在/内容未改，manifest复验没有证明B实际将绑定哪个头。
2. 正控对象载荷以公共store API合法存入一个**不同target/toolchain/flags**的generation（载荷未损坏，不执行不兼容机器代码）；当前Windows-x64请求仍`hit=true, compiles_during_request=0`。这证明绕过的是key适用性，非CAS字节校验。`c_object_key`里有字段，但发现路径没有消费它们，不能靠key单测变化证明失效。

修复要求：generation发现与当前完整recipe输入同域，至少绑定/验证实际源解析位置、include搜索配置、target、工具和flags；不能把旧依赖仍存在当当前source/recipe适用性。保留跨工程正确复用控制，也保留同内容不同依赖、旧工具/错target拒绝反控。

## F-797-R4-07/P1：已观测的旧头文件内容相同，不代表include解析结果相同

AC-04，T-05。本轮编译器观察代替文本扫描，原注释/续行/macro控制通过，认可旧例修复。仍缺**搜索环境变化**。

独立C源在`shadow/`，`#include "dep.h"`最初从显式`-I shared/`找到VALUE17；编译入store。随后在源码同目录新建合法`shadow/dep.h`（VALUE29），不改source或旧shared header。当前cache仍命中旧对象，PE为**17**；同一实际cl命令直接fresh编译的PE为**29**。源和先前列出的依赖哈希都没变，依赖解析却已变。见同一[cache probe](cache-probes.rs)的shadow-before/shadow-after/shadow-fresh分支与日志。

这是§5.4“实际输入/未知依赖不能当纯函数缓存”的原不变式，非要求新C importer。须把头搜索/遮蔽等负依赖或等效可验证环境纳入身份/准入；不可只重验上次打开过的文件。新增/删除近端头、改变include目录、相同输入合法hit均需区分性控制。

## F-797-R4-09/P1：计时改善，但仍未到验收规定的发布边界

AC-05，T-05/T-06/T-07。正式父测试实际通过，报body-edit P95 **657ms**、reject P95 **86ms**；31/30原始行已复制于[formal-gates.jsonl](formal-gates.jsonl.txt)、[summary](formal-summary.txt)。认可父IPC/等待/响应解码及Job资源计数新增，**不认定实际性能超1s**；问题是数值尚不能证明规定的whole-path硬门。

`test_cases/abi.rs:710`在收集响应、进入最终typed outcome/PE/cache/compute判断之前结束`parent-request:query`；全部请求之后才在`1000–1009`判断并生成`parent-publish`。硬门测试`probe_adapter_tests.rs:1152`明确**跳过parent-publish**，最终仍取request span算P95。源码edit也在`630–633`计时前应用。单请求“decode完成”与合同“必要校验/比较发布完成”不是同一边界；SD自己要求edit+query请求对，但本实现排除了edit及最终发布。

持久化亦未全部兑现：body调用`run_abi_deck_case_expect`先把31个请求跑完（1109），直到返回report后才`1162`循环落盘；并不是“每样本落盘后再运行下一样本”。响应过程出错时没有给失败请求添加request span，原始记录可能缺该失败行。body各行复用整case的`check_ok`，不能准确归因哪一请求失败。reject行目前只有stage/peak_mem，没有其余cache/compute/I/O/meter，body阶段没有父发布/判定归因。

修复要求：确定并实际计到每个编辑后请求的验收/发布完成；一请求一结果一原始行，结果与资源（包括失败）持久化后再继续。保持C consumer复用与cold producer断言，不能重回阶段和或用批量check总耗时平均替代P95。不要通过删掉原合同的比较/发布或失败样本要求收口。

## F-797-R4-10/P1：物化表示变小，不等于暂存与索引预算满足

AC-01/04/05，T-02/T-05/T-06。原R5长字段名图本轮确实在物化前拒绝：`StagingTooLarge(size=19,399,184)`；短名65536叶控制成功，新增峰值3,671,954B。**认可这一原例闭合**。

但`layout.rs:296–314`为每个record建4个BTreeMap（sizes/leaves/path_bytes/max_path），并复制其record名；chain/seen也复制名字。这些索引和临时状态不计入`329–334`仅围绕root物化表示的gate。

独立[layout-memory.rs](layout-memory.rs)以GlobalAlloc计数实际System分配，先构建输入、在调用前设置baseline，**排除输入本身、排除64MiB线程栈、排除分配器额外开销**。1024节点合法无环链、每个record名1950字节、记录数组JSON **4,047,973B <4MiB**，结果只有一个字段（已拥有输出2079B），却返回成功，新增堆分配峰值 **10,298,591B >8,388,608B**。见[layout-memory.log](layout-memory.log.txt)。这是现有≤4096深链/暂存及索引8MiB的兄弟组合，不是增加规模要求；完整图/源码均在可复跑程序中。需对memo/index、递归工作状态和物化共同计预算，超门受控resource拒绝；不能只测结果owned bytes。

GC覆盖还存在未证明项：新增`ABI_TEST_GC_EVERY_STEP`只迫使worker调用`run_gc`，传入仍是8MiB/6MiB水位；`store/gc.rs:149–153`低于high直接返回`ran=false`。并发测试只跑两lane×4小对象，不断言GC的`ran/scanned/deleted`或live/哨兵。对相同配置的独立收据控制明确为`ran=false, scanned_chunks=0, deleted_chunks=0`（[gc-receipt.log](gc-receipt.log.txt)）；这份控制不是冒称测试内部收据，而是揭示该“force”调用不能保证进入回收。测试里没有当前收据可证明其实际回收，不能宣称“逐查询真实回收全链已通过”。应补超过有效水位或可验证force的真实worker/GC收据与live/lease/取消哨兵，保证该oracle会在不回收/删错live时失败。

## F-797-R4-12/P2：旧三条warning修了，新增两条仍由本计划承担

AC-06/07，T-07。原response_opt/exit_ok/hot_bytes三条警告消除，scoped格式/空白通过，descriptor/3、summary动态读取、旧数字清理均认可。当前编译仍有本轮新增：

- `test_cases/sema.rs:433`：`JobStats.peak_job_memory_used` dead_code。
- `tests/probe_adapter_tests.rs:1069`：`let mut append_jsonl` unused_mut。

完整当前诊断见[verification-summary](verification-summary.json.txt)、parent-build/parent-tests日志。不要以root其他历史warning或只门控standalone crate掩盖。本轮没有要求清理无关历史告警。freeze/SD的whole-path/预算/完整回收声明还须在F07/F09/F10/R9-01真正闭合后与实际证据一致。

## 规范、路由与下一步

SD-01/04/05关于candidate/单target/非Auto源码/774分域方向正确；SD-02/03仍**fail**：预算、缓存适用性、whole-path和实际GC证据没有满足拟发布规则。supersedes两条与new三条docs/specs路径保留；`touched_goals: []`已依据当前主Spec重新核对。未扩大语义合同/改plan_revision，也没有发布规范或ledger。

**next: work**，原树修F-R4-07/09/10/12与新F-R9-01，保留原例、合法控制和上述新组合，先修不变式/设计与oracle再交接；明确旧例闭合与完整不变式闭合的区别。本轮170绿不能抵销实际错误PE/超预算，也不证明GC发生。

**pending_closeout_gates**：最终代码/验收/Spec final pass后，merge在新冻结提交统一执行完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t和按实际diff触面档；tf只主检出到期批量单实例。门禁前不得合入/宣称交付。规范/ledger沉淀、归档、wt-guard与清理均pending；本轮未改实现/合入。
