# PLAN-797 独立 final re-review R8

2026-10-20（Asia/Shanghai），**needs_fix / executing 2/9**。受审修复提交 `bf2455758e074cced103d7da5245b0991a3e3e8f`。上轮10个开放finding中，**5 CLOSED，5 PARTIAL/OPEN**；未新增finding编号，不把新选择的同不变式反例改称新需求。保留T-01/T-03，重开T-02/T-04/T-05/T-06/T-07，T-08/T-09 pending。未修改实现或合入。

## 基线与独立性

- stage: review；review_scope: final；plan_id: PLAN-797；plan_revision: 1。
- reviewer_identity/limitations：当前Codex复审上下文，没有实施本分支，没有创建代理；具体模型/session ID未暴露。以代码、独立反例和当前测试建立结论，不以work的"10/10"声明代替独立验收。
- worktree `D:/autostack/.wt/lang-797/auto-lang`，branch `plan-797-dev`，base `13367d127584461449ee2f151c0c5987f0740044`；主检出 `08ba8ca2bd5f21b0063c9ed2c0ef84f0a1f7db94`。入审/收尾HEAD相同、实现树clean、受审输入无漂移，见[baseline](baseline.json.txt)、[end baseline](end-baseline.json.txt)。共享Plan虽frontmatter仍executing，但最新work交接声称全部work-owned完成，本轮已验证其根因/覆盖调整后进入完整复审。
- 依赖tree：ac-core `8e01299a76e4e9ad34ac61c8ce5302a84adea400`；ac-artifact-probe `d4c37faca3843b4a241f5f0770156a09efc23f65`；ac-semantics/contract `13b4ac2821e1745b1d644e49932538ee6d642880`；auto-down sibling `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。D-796/D-788已满足。
- rustc1.98.0；MSVC14.44.35207目录（banner19.44.35222）；SDK10.0.26100.0；当前sysroot rust-lld。工具SHA及版本见[验证摘要](verification-summary.json.txt)。本轮重新请求构建当前release worker和debug auto-case，Cargo判断fresh；二进制身份见[binary identities](binary-identities.json.txt)。未因二进制时间相同而跳过反例重放。
- 7份canonical Spec在主检出与实施树逐项一致；受审SD SHA256 **`06c55078c395ee54c8b766be2b11befc005e6844941c4359a596adbda24ec594`**，冻结副本[reviewed-spec-delta.md](reviewed-spec-delta.md)。支持矩阵冻结于[reviewed-support-matrix.md](reviewed-support-matrix.md)。未改canonical Spec/live ledger。
- 现有review-r7目录有未收口的探针工件，没有Plan中的R7最终结论。本轮另用review-r8保存当前实跑，不把那些输出冒称本轮测试，也不虚构R7 verdict。此前完整final失败为R4/R5，本轮为第三次完整final验收。

## 本轮实跑与覆盖

Cargo均在原树，`--locked --offline`；构建进程内清空RUSTC_WRAPPER。没有整档`t/tv/tt/tb/tu/taa/tf`或完整verify。完整ABI closeout及日常/触面档仍merge-owned。

| 命令/入口 | 实测 | 证据与限制 |
|---|---|---|
| `cargo test --manifest-path experimental/auto-abi-probe/Cargo.toml --test t02_contract_layout_admission --test t03_calls --test t04_resource --test t05_cache_turnaround abi_ -- --nocapture` | 19+7+5+8=39 pass | [scoped-native](scoped-native.log.txt)。包含真实C三腿、资源边界、include旧例、512-churn helper与六场景。绿不能覆盖下述独立反例。 |
| 同manifest，`--test t06_binding abi_binding_ -- --nocapture` | 3 pass，2性能项筛除 | [binding](binding.log.txt)；真实semantic绑定/退役/typed deck。正式父硬门另跑，未重复worker-only 30样本。 |
| root `--features test-cases --lib abi_worker -- --test-threads=1` | 5 pass | [lifecycle](lifecycle-tests.log.txt)；响应后hang、迟到stderr、溢出且hang、infra分类、双信封。 |
| root `--features test-cases --lib abi_recipe -- --test-threads=1` | 3 pass | [formal-runner](formal-runner.log.txt)；语料/互斥/当前硬门，报body-edit544.0ms、reject410.2ms。计时口径仍失败，见F09。 |
| root `--features test-cases --lib abi_deck_finite` | 1 pass | [deck-finite](deck-finite.log.txt)；finite但不可转换的1e30受控typed拒绝。 |
| `cargo test -p auto-test-case` | 87 pass | [contract compatibility](contract-compatibility.log.txt)，含30单测+57集成用例；0例doc-test不计。 |
| root `--features test-cases --lib test_cases_tests -- --test-threads=1` | 26 pass，1既有ignored | [runner compatibility](runner-compatibility.log.txt)。 |
| 独立旧输入重放 | 详闭环表 | [replay.py](replay.py)、[library](library-probes.log.txt)、[worker raw](worker-raw/worker-results.json.txt)、[parent](parent-results.json.txt)、[stale](stale-results.json.txt)、[C资源](resource-results.json.txt)、[缓存坏块](cache-failure-results.json.txt)。 |
| 独立C预处理兄弟反例 | 3种漏键，均实际PE17→29 | [脚本](include-siblings.py)、[源码](include-siblings.rs)、[结果](include-siblings-results.json.txt)、[闭包/descriptor输出](include-siblings.log.txt)。 |
| scoped fmt/root rustfmt/diff空白 | pass | [probe scoped fmt](probe-fmt-scoped.log.txt)、[root fmt](root-fmt.log.txt)、[diff-check](diff-check.log.txt)。初次加`--all`跟随path依赖检查无关root workspace，产生既有格式差异，日志保留，未归罪797；随后准确限定probe复核通过。 |

合计**164个不同定向测试通过，另1 ignored**；反例程序/脚本不混入此数。独立反例的script exit0只表示探针执行完成，不表示被测合同满足。原生与父性能跑在各自采样时段，后续正式硬门仍须以正确计时/原始资源口径重证。

## finding闭环

| ID | 状态 | 本轮证据/剩余问题 |
|---|---|---|
| F-797-R4-01/P1 | CLOSED（本轮既定RecMix/deck绑定面） | 原标签+篡改载荷、真实新标签+篡改载荷均拒；i32/u32与char/u32原攻击不再admit；i32范围、NaN、1e30 typed拒。承认所支持有限recipe的修复，不外推任意未来名义图。 |
| F-797-R4-02/P1 | PARTIAL/OPEN | RecSum生产描述已为`pure(rec:RecMix:const)->i64`，枚举/handle/callback表面增加；但C provider真实物理原型/ownership仍未如实描述，mut/out支持仍无生产原生证据。 |
| F-797-R4-03/P1 | CLOSED（所列故障/组合） | 溢出且继续运行独立mock约298ms即ERROR/exit2；5永久生命周期例通过；worker infra/resource父侧短路已接；1e30走diags，原"期待success"控制正确FAIL，不能误称infra。 |
| F-797-R4-07/P1 | PARTIAL/OPEN | 旧空格/尾注释/local-angle均换键，macro明确Err；但合法预处理注释与续行仍被静默忽略，PE依赖真实变化却同键。 |
| F-797-R4-09/P1 | PARTIAL/OPEN | C腿/共享store/31独立cold体/compute32修正已认可；body-edit门仍只计worker内部阶段，原始样本/资源/失败保留仍不完整。 |
| F-797-R4-10/P1 | PARTIAL/OPEN | 四次真实查询current root现只有1；失败native仍保留旧root；C compile/PE capped已接；但原长字段名布局反例仍返回19,398,656B>8MiB，预算/资源trace未完整。 |
| F-797-R4-12/P2 | PARTIAL/OPEN | probe/root格式及descriptor/2已修；本计划自身3条compiler warning仍在，freeze/verify/SD事实未对齐。 |
| F-797-R5-01/P1 | CLOSED | 原独立C槽满+64次失败16KiB分配+全释+再次分配，exit0；slot-first修复实证。 |
| F-797-R5-02/P1 | CLOSED | 原独立C65,536代控制exit0；16位一致+耗尽退休后新槽可用，旧句柄拒绝。 |
| F-797-R5-03/P1 | CLOSED（layout坏块原路径） | 原真实ABI-L0块翻bit后worker exit0、outcome ok，stderr受控降级且meter计拒绝；无原expect panic。 |

R4已关闭04/05/06/08/11继续保持CLOSED；相关资源、生产计数、混合stale正反序也已重放。R1锁交付F03继续CLOSED；R1完整绑定/终稿的剩余部分对应本轮开放项，不能把文件存在当最终验收。

## 五项剩余问题与修复要求

### F-797-R4-02/P1：生产descriptor仍不能如实供独立consumer调用

AC-01/03/06；T-02/T-04/T-06/T-07；分类为原合同缺口，非扩展平台要求。

`src/descriptor.rs:389–396`把`p797_ledger`描述为`fallible()->status,out i64`；独立header `cside/interop_resource.h:38`实际是`int p797_ledger(long long *live,long long *allocs,long long *frees)`，**三个输出地址**。新增注释"3-out aggregate: spelled in doc"没有改变descriptor；独立consumer无法从此描述重建真实参数列表。此处不执行已知错误签名制造UB。

`descriptor.rs:437–442`把`p797_two_step`输出标为普通I64，但真实协议是owned handle B、原provider destroy。创建函数已经有`Handle{provider}`，而复合创建返回仍丢失所有权。支持矩阵还称record mut/out均已支持，实际生产描述/原生操作仅有const校验或读，没有对应mut/out提交/失败回滚控制。只添加ViewRole枚举不能等同生产边界实现。

永久`abi_contract_c_provider_descriptor_consumable`最后用**同一descriptor自身的signature**填consumer expectation，再admit；这是自洽性测试，不是独立header/物理形态oracle。需要把完整物理参数、status-only/多out、handle归属/销毁及callback ctx如实描述，独立审定C原型逐项对照；兑现既定mut/out面或走new正式修订合同。不要隐藏在"C桥例外"中削减已承诺profile。

### F-797-R4-07/P1：修复的文本扫描仍漏实际C预处理依赖

AC-04；T-05。`src/cache.rs:129–140`只识别原始行trim后以`#`及`include`开头；没有处理预处理前的注释替换/续行拼接，也没有对这类未知形态fail closed。

本轮独立选择三种合法兄弟：`/*comment*/ #include "dep.h"`、`#/**/include "dep.h"`、`#inc\`接下一行`lude "dep.h"`。每组源SHA不变，header VALUE17→29：closure均返回**相同Ok(hash)**，真正cl编译、rust-lld链接、运行PE均**17→29**。普通include合法对照确实换键。完整命令、源码SHA、运行结果保存于include-siblings附件。

该问题仍是§5.4"实际include闭包/未知依赖不当纯函数缓存"的原不变式。应消费真实编译器依赖或完整归一化/保守拒绝，覆盖这些输入。当前"system headers covered by toolchain"的注释也不成立：`toolchain_fingerprint`只含cl路径/版本/exe哈希/flags，没有MSVC include文件闭包；不能把编译器exe哈希当header内容哈希。

### F-797-R4-09/P1：硬门继续把worker阶段和当作父入口完整耗时

AC-05；T-05/T-06/T-07。认可本轮统一C腿、消费者一次编译/31cold callee、compute32断言。测试实际通过，报body-edit P95=544.0ms/reject410.2ms；**不认可其whole-path性能认证**。

`test_cases/abi.rs:689–691`把worker响应的`wall_ns`转毫秒填stage_spans；`probe_adapter_tests.rs:1100–1111`把这些阶段求和，在link-run处分样本。没有父入口请求计时/IPC/等待/响应解码/父比较发布，worker的root替换/GC也发生在link-run阶段计时后。把这种内部和写作`request_ms`或"parent-path完整链"仍违反§5.5"不扣真实等待/IPC"。31个查询在一个worker session中批量返回，当前协议没有每个编辑请求完整发布的计时边界。

`1123–1129`在整个case返回、pass和compute断言之后才批量写body样本；reject30次也在`1155`末尾写，发生断言失败会丢该批证据。已复制的[formal-gates.jsonl](formal-gates.jsonl.txt)只有scenario/sample/ok/总数，没有每样本完整阶段、hit/producer/I/O/peak内存；"逐样本先写再继续"与代码不符。应在正式父入口保留可归因的完整请求时间（保持消费者复用），并在断言前持久化所有原始成功/失败及资源样本。不能靠worker-only另一组数据拼接认可。

### F-797-R4-10/P1：原暂存预算反例没有被修复

AC-04/05；T-02/T-05/T-06。`layout.rs:303–304`改成leaf-count×(FieldLayout+24)，仍没有计算实际字段路径字符串。`flatten:442`每层拼接`parent.child`，路径长度随层级和字段名增长。

**原R5输入逐字重放**：5个record节点、4层16分叉、字段名64字节、65,536B物理payload；仍成功返回65,536字段，仅结果已拥有的向量容量+String容量下界 **19,398,656B > 8,388,608B**。见[probes.rs](probes.rs)、[library-probes.log](library-probes.log.txt)。work永久测试换成短名字而得到约4.9MiB，未覆盖原输入；不变式未闭环。须在物化前计入实际路径长度、容量/中间态及索引，或采用有界表示；至少原输入应受控resource拒绝，合法短名字控制仍可用。

认可root累积、layout坏块及C capped修复，不把它们重复记成未修。其他资源边界仍须补实际覆盖：`worker.rs`在producer前仅查已有produced_bytes是否`>128MiB`，当前生产的增量发生在检查后，之后还可C编译；不是"剩余预算预留/累计全产物上限"。`meter.scratch_peak_bytes`统计的是磁盘scratch目录当前最大大小，**不是**内存峰值；正式样本没有完整必要I/O/内存。受管store helper的512次GC绿不能替代真实worker预算/取消/并发GC全链证据。修复不得通过提高原阈值或将失败误判ABI不兼容。

### F-797-R4-12/P2：自身warning与终稿事实仍未闭合

AC-06/07；T-07。probe/root格式、空白和SD `/2`认可。当前构建仍明确报告本计划`test_cases/abi.rs`三条warning：385行response_opt初值未读、429行exit_ok初值未读、236行hot_bytes未读；代码本身和[health摘要](verification-summary.json.txt)对应。root其他408/569条历史warning不要求本计划顺手清偿，但不能遮蔽这三条。

freeze proposal仍混有旧820.7/479.1数据及"逐样本落盘/完整链"的不实表述；verify脚本301行仍固定输出R4旧数字；SD末尾还留"T-06后必须闭合"三项，在前文却称已完成。应修正当前事实/健康后再冻结提案；SD-02/03也须随F02/F07/F09/F10真实闭环同步，不能据ALL-PASS模板发布。

## AC→任务→结论 / Spec审查

| AC | 任务 | 结果与当前依据 |
|---|---|---|
| AC-01 | T-02 | partial/fail：RecMix语义载荷绑定、C布局、准入旧反例pass；F02/F10仍失败。 |
| AC-02 | T-03 | pass（代表三腿及异质反控）：7个calls实跑pass，不外推全部profile/源码支持。 |
| AC-03 | T-04 | partial/fail：资源生命周期/代际/失败回滚的旧反例关闭；F02的可消费资源/record mut/out面未过。 |
| AC-04 | T-05 | fail：root及坏块恢复已修；F07 include闭包/F10资源仍失败。 |
| AC-05 | T-05/T-06 | fail：计时值绿但whole-path/original资源数据不满足F09/F10。 |
| AC-06 | T-06/T-07 | partial/fail：真实796绑定、AutoDown语料、父错误分类及v1/v2合同定向兼容pass；准确终稿/SD仍不合格。 |
| AC-07 | T-08 | fail：5个合同内开放finding，不能发reviewed。 |
| AC-08 | T-09 | merge-owned pending：本轮未合入/沉淀/归档/guard清理。 |

SD-01/04/05的候选/单target/非Auto源码前端/774域分离方向正确；SD-02/03 **fail**，当前"经实证完整资源/descriptor/whole-path"的规则未由代码和证据支持。保留supersedes两条与new三条真实docs/specs路径。`touched_goals: []`有原书面理由：canonical goals无独立公共ABI probe产品目标；未接产品不冒称目标交付。本轮没有语义合同修订或改plan_revision。

**next: work**，原树修F-R4-02/07/09/10/12，保留所有原反例（尤其长字段名布局，不能改短名字替代）和合法控制，提交准确closure/readiness再审。根因分析里"载荷对标签"和"资源代际"调整已实证；"表示字节"和"同一正式路径"分析仍没有落实到实际内存/端到端计时，应围绕不变式修正设计与oracle，不再只追加绿色总数。

**pending_closeout_gates**：最终代码/验收/Spec review pass后，merge在新冻结提交跑ABI完整套件/真实C三腿、796/788受影响兼容、根裸cargo t和按实际diff定界的触面档；tf只主检出到期批量单实例。未过门禁不得合入或宣称交付。Spec/ledger沉淀、归档、wt-guard和清理均pending。
