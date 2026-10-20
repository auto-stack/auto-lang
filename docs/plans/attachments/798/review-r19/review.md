# PLAN-798:r7 最终审查 R19 — pass

review_scope: final。reviewed_commit: `83086e76f62f6a79b902c9c52784d28cc359258a`；base: `cb2194c2461a203712bd601c7d314ab35544fec3`。唯一工作树`D:/autostack/.wt/lang-798/auto-lang`、branch `plan-798-dev`，审查期间代码冻结且clean。主检出合同为r7/execution_done9/11。

reviewer_identity/limitations：当前Codex实施会话，未获得独立上下文/模型或代理；不冒称另一位独立审查人。以重新阅读真实消费路径、独立选取的R18组合反例、原R15反例重放及原始数据重算建立结论。冻结来源/锁/配置/Spec哈希见`../review-r18/source-hashes.json.txt`；auto-down组内/主检出均c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 clean，依赖无漂移。`unchanged-components.txt`为空，证明相对a00c259f2 core/独立锁/ac-core/artifact/semantics/Atom诊断组件未变。

## 用户裁定与审查边界

用户明确warm未达归因整模块重查、完整函数增量归优先828，798负责真实计量和PASS证据保留。已显式修订为r7，同步§5.5/5.10、AC06、V11及拟Spec，原失败样本与历史review均保留。R17-01以用户合同修订关闭，不能写成旧测试“代码已修好”；不以异时样本推导优化/环境各自因果。正式30样本P95<1s、正确性/内存/K14/生命周期从未豁免。当前新代码warm亦实际PASS，但798仍非全链路函数增量。

## Finding关闭矩阵

| finding | 原失败与当前验证 | 结论 |
|---|---|---|
| R15-01 | A08/A09/闭包/无源恢复按当前grants重授；原注入trace在无grant时拒、有grant时MarkA真执行 | CLOSED，原独立probe与永久cache控制 |
| R15-02 | 可信unit信封不接受完整module载荷；raw合法kind声明正控保留，unit不可直接执行 | CLOSED，A08/A09与恢复路径均核实 |
| R15-03 | sidecar可解码不等于适用；空/局部映射重推导或拒绝，Bin/NegI32 trap保真 | CLOSED，cache20与原probe |
| R15-04 | manifest required-data边与真实消费lease；GC后闭包恢复 | CLOSED，含R16/R18更深消费组合 |
| R15-05 | 准备器双流cap、单deadline、suspend/Job/resume、杀树/退出释放 | CLOSED，正式7项内真实失败形态与源码共用双流控制 |
| R15-06 | core冗余import移除 | CLOSED，实验包无新增警告；依赖旧warning单列 |
| R16-01 | 首unit lease不能沿manifest反向边保护siblings；补A08/A09每消费单元以及恢复manifest lease | CLOSED，两函数/失败返回/全部锁释放永久控制 |
| R17-01 | 两次warm 1027.0/1465.5ms失败保留 | CLOSED_BY_AUTHORIZED_CONTRACT_REVISION，r7用户裁定 |
| R18-01 | 位置修订同manifest只保旧sidecar；仅manifest lease GC后新来源丢失 | CLOSED，format2/diagnostics_snapshot及key generation绑定、代码unit身份不变 |

R18旧例实测same_manifest=true，GC后no source sidecar；修复后same_manifest=false，required_edges=2，GC实际删3旧块后仍恢复`div-by-zero@func#0:25..30`。新永久控制还验证同unit命中、错快照/旧缺字段/未知格式拒绝、空/局部sidecar仍触及覆盖检查、lease释放后可回收。诊断与为什么旧同快照GC/有软根位置测试漏掉组合见`../review-r18/diagnosis.md`；不是只通过旧单例。

## AC→任务→证据

| AC | 实现/验收与证据范围 | 结果 |
|---|---|---|
| 01 | T01/03：有限Typed MIR/ABC/AV、私有Checked凭证与版本/类别。R15/K14原例、当前缓存消费/正式路径。 | pass |
| 02 | T02：HIR→MIR、CFG/类型/初始化/来源；当前resources17重放R7–R13构造路径/前缀/兄弟/尾操作数边界；恢复新span。 | pass |
| 03 | T03：codec/数值trap/短路/顺序/调用及真实原生参考；纯core/native未变部分复用已识别旧验收，不外推修改driver。 | pass；完整独立workspace/native重跑归merge |
| 04 | T03/07：当前授权、kind、坏wire/资源/取消、worker/preparer协议与失败退出。 | pass，cache20/正式7/原R15反控 |
| 05 | T04/05：单元身份/依赖/恢复/跨工程/Conflict/来源/required-data/lease/GC，位置修订保持代码复用。 | pass，cache20、资源17、独立R15/R18 |
| 06 | T05/07/11：同轮六场景180样本及正式30样本，双线/分区、所有旧失败保留，r7口径。 | pass，见计量节 |
| 07 | T07：真实corpus、负oracle、解码/故障/准备器/混合stale；ABI与AV注册/Options/dispatch/身份独立，ABI拒AV-only Run。 | pass，正式7；根接缝27/contract30未改组件证据可复用 |
| 08 | T06/08：真实796身份域、临时快照退出、AJ/native有限边界、r7全部SD与后继828。 | pass |
| 09 | T09：本次完整代码/验收/规范复核，所有in-contract finding闭合，限制与merge-owned门明示。 | pass |
| 10 | T10：冻结整档、沉淀、归档/实际ff-only、guard清理。 | merge-owned pending，未声明交付 |

V01私有授信、V02类型/CFG、V03数值/短路、V04参数/循环/递归、V05坏wire/producer、V06真缓存仍执行、V07依赖/配置、V08错误身份与回收、V09 4096块/33.5MiB/512真实修订、V10预算/取消/失败退出、V12真实runner负控、V13 796/有限native边界均通过。未改纯core/codec/native的已有102条组件证据按相同源码/依赖/输入复用；修改后的消费路径由当前cache20/resources17/formal7及独立反例覆盖，不能拿旧102作为当前总数。

K14-01完整单函数self-call module=42；02 unit可验不可直接执行/raw合法module声明不误拒；03 complete module入口/参数/返回/imports规则；04未知flags/version及可信unit producer类别冲突；05 A08/A09/closure/restart当前grant重授；06旧v1隔离与位置/来源适用性/恢复闭包。每条都有原正反控与当前真实消费映射，不以单个类别往返绿冒领整个矩阵；R18给K14-06补了跨snapshot/GC组合。

## 当前计量、原始证据与健康

- 新代码正式族7/7通过，其中正式30样本edit P50/P95/P99=816.4/872.5/1197.0ms、sig P95=387.7ms，exit0。首个较慢窗口未删，P95门通过；运行中CPU68%及并发rustc亦如实保留。零门反控exit101且明确`>= gate 0ms`，不是启动/解析故障冒充反控。
- 用户所述旧01dc保存日志687.4/234.9ms PASS与新代码日志各自审计30个不同编辑窗口，重算打印精度下edit P95一致；sig没有逐窗口日志，本结论引用真实执行断言/摘要，不伪造sig raw独立重算。
- 当前release同轮180条六场景各30、全部正确、30不同编辑source hash；2不同结果值逐样本与独立expected一致。重算warm edit P95=946.69ms、sig=38.2213ms，native exit0/hard_gate_failures=[]。
- A前端峰=211697664B（201.890625MiB），B真实残差=5746688B（5.480469MiB），两线/全部受管分区均通过；未把端点增量当残差。raw/summary SHA及所有字段在`bench-current-audit.json.txt`，与同轮原件绑定。
- 两包fmt/diff-check通过；root既有warning及shim-metadata旧warning单独归因，无本改动新增warning。`fmt --all`涉及只读auto-down既有生成代码格式差异，不改依赖来取绿。无新debug打印、无以债务登记替代当前要求。

## 规范增量与遗留

SD01有限MIR、SD02 ABC/执行类别、SD03 producer/恢复/身份/当前授权/来源/lease、SD04 AC产物carrier、SD05 HIR前门、SD06 TestCase正式配方/有界准备、SD07有限AJ/native边界、SD08逐阶段交付边界与828，逐项与当前源核对。准备了11新合同/导航、3修改合同及派生INDEX/分组登记；源码/锁/配置/规范完整哈希与冻结delta已保存。恢复metadata JSON格式2与ABC wire2是不同轴，旧无快照恢复manifest受控拒绝；函数代码身份/codec/能力合同未借此变更。

模块plans导航在merge后改为归档来源，纯元数据closeout不授新功能。touched_goals=[]理由为experimental与测试接缝，不改产品默认入口。原前端内存长期优化与全链路函数增量归相应owner/828，当前r5双线仍严格通过，不属未批延期。顺序798交付→优先828→后续ACE强制增量兼容保持。

## Verdict与下一步

outcome: **pass**。Plan可进入reviewed10/11。pending_closeout_gates：merge在冻结提交执行根无过滤cargo t（含docs_gen）、cargo tv；独立workspace全套/真实native及auto-test-case共享合同/实际796/788/797兼容。代码/依赖/配置变化则重验受影响项。tf按主检出单实例到期规则，当前2026-10-20T09:20Z新鲜且798%5!=0，不到期。Spec/ledger/index发布、ff-only落地、归档、wt-guard与树/分支清理均pending；未跑整档、未合入，不用review pass宣称已交付。
