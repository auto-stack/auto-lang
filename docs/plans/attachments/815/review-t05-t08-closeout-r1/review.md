# PLAN-815 T-05～T-08 整任务联合收口复审 R1

2026-10-22，**pass（四项任务联合验收，非 T-09 整计划终审）**。T-05/T-06/T-07/T-08 可联合勾选，完成计数 **10/17→14/17**；Plan 保持 **executing**。无新增 finding，所有当期修复 finding 维持 CLOSED。T-09 单列保留待审，未授权提前合入或宣布M815-B发布。

## 冻结与审查边界

- 唯一树 `D:/autostack/.wt/lang-815/auto-lang`，branch `codex/plan-815-dev`，HEAD **a67f114fda8abf52630126d4aa52eeacd6d3248a**，base **afd1a66bff2584879687d2016eefe2f8153eb726**；本提交仅修改 handoff.md。Plan r3，M814-B交付祖先93a25c924d1beafab130588bad4d1041dc86e48c保持。
- reviewer=Codex `/root` 延续独立审查上下文；有早期合同/账本参与限制、未参与当期实现。按/auto-plan:review重建联合覆盖，不仅累加绿色数量；未创建agent/第二实现树，未修代码/发布canonical/live ledger/合入/归档/清树。
- 当前源码、依赖、锁、测试配置、资源及driver相对afd无变化。起止同HEAD clean、5943项输入和282项Spec零漂移，见[baseline](baseline.json.txt)、[终核](final-baseline.json.txt)。生产语义冻结e22fc7592、验收树afd、docs叠加a67分别记录，不把生产部分不变表述为整棵源树相同。

## 四任务逐项验收

| 任务 | 合同/验收 | 联合裁定与可复核证据 |
|---|---|---|
| T-05 | AC03 / V03，真实工具执行/控制/日志/交互/取消/退出 | **PASS**。按R6→R7→R8→末项终核关闭链核对：真实VM呈现和Input q/取消/EOF、owned子树和reader收口、会话环境隔离、后端失败传播、非自然捕获具名failed、PM请求单源/合法bun与legacy语义均已闭合。Windows及独立Linux记录保留，未以平台名称外推未执行GUI面。 |
| T-06 | AC01/02/04 / V01/02/04，完整profile派发与旧入口兼容 | **PASS**。生成/构建/运行服务集、无第二主声明、front节点优先与全链来源、真实输出根/计数、多目标完整生成、build/run诚实失败和托管取消统一设施已审；Rust源目录/Ark启动ID邻接组合闭合，旧Man门面/legacy返回对照保留。原生设备私有工具记录器只证明argv/失败传播，不冒充真实设备成功。 |
| T-07 | AC01..04，新增consumer/V01..04、发行/资源/失败证据 | **PASS**。末项dist JS加载oracle；子批1四回执/精确帧深度及绕过writer的raw reader边界、Linux real_tree；子批2两实修复及真实build；子批3独立33/33（新tool包A/B locked构建/外部消费）、377文件对账、源唯一性/资源许可事实与字段消费矩阵均已闭合。分支777 strict0 violations/3 grandfathered在案，master并集按合同归T10，不冒称已经执行。 |
| T-08 | AC04，接口/能力/源与产物身份/来源/SD01..07交接 | **PASS**。本次定稿消除了历史debug指纹充当当前身份的问题；源树/生产语义/参考artifact分层、实际release身份核实、能力×profile与协议生命周期、PM/owned树/资源及七项SD正文可供T09独立终审，所有handoff链接可解析。 |

具体 `AC→任务→证据→观察→边界` 见 [task-acceptance-map](task-acceptance-map.json.txt)。引用的R6/R7/R8、最后dist闭环、三个验收子批报告与原始日志均仍在主检出附件中。早期needs_fix保留为历史，最新关闭状态没有依靠删项、弱化oracle或把合同内缺口转为非阻断债务。

## T-08 参考 artifact 的独立核实

实际 `target/release/auto-ui-tool.exe`：**85,116,928字节**，SHA256 **1626cec1087e8ba8dec4aede5f24e75ba12245e5b40c40705326ccbfab661696**，`--version`=**auto-ui-tool 0.1.0**，与身份表相同。本轮没有重跑6m52s release编译；构建命令/toolchain为work记录的provenance，审查额外确认当前树生产部分与已审冻结位一致，并核实实际文件及运行行为。

[release-probe](release-probe.json.txt)、[脚本](release-probe.py)、[日志](release-probe.log.txt)使用该精确release文件进行四组实际控制，**4/4**：

1. describe精确五操作及cancellable/interactive位与handoff一致。
2. 合法I-PROJECT请求validate ok；磁盘name/render与请求不同，后续不采用盘值。
3. 协议99的generate具名拒绝、gen目录不存在；同流协议1随后可实际生成。
4. Vue真实产物使用请求name；release嵌入的shadcn button物化文件逐字节等于登记源；EOF干净exit0，control始终可解析。

新probe产物仅在唯一树target私有目录，未改生产文件。它补的是本次新release参考文件的有限消费证据，不替代先前debug/包内构建的全部生命周期或跨平台运行实证。

身份语义可接受：不同debug构建哈希不被宣称相同，release是精确文件的参考指纹，不承诺可重建相同字节；源码身份由完整提交及受管manifest来源绑定。792实现对排序后的逐文件rel/len/blake3计算crate目录digest，manifest保存目录digest、根输入blake3、producer.git_head/dirty/ui_gen_fingerprint/root_lock_sha256等；**并非manifest逐项展开存储每一源文件hash**。文中“逐文件blake3”按该实际算法解释，不将不同源树或不同构建产物当同一artifact。T10发布收据仍须绑定届时实际冻结身份与祖先。

## 证据复用、归档与卫生

- [reuse-and-archive](reuse-and-archive.json.txt)记录源码/依赖/配置未变及归档字节核验。work归档的driver收据、日志、embed对账三份与独立审查开始时保存的原始快照**完全一致**；tool stderr在batch3留盘。主检出review目录另保留独立33/33日志/收据、377文件哈希与holdout请求/结果，scratch删除未删除这些独立证据。
- 复用范围明确：前期真实生命周期/兼容/单源结果、子批1协议/Linux、子批2实际build与两个实修复、子批3发行/资源/实际效果、两枚新断言均按原冻结来源和已确认未变输入复用；不称本轮重新执行了它们。
- 相对最后修复对象仅docs改动，本轮未运行cargo测试、docs_gen、整档或重新生成发行包；handoff链接、git diff --check通过，无本次新增编译诊断。既有warning和shell_pack/plan734等披露并未被写成全库全绿；T09仍须结合原合同裁定整体，不以“预存”免除范围内义务。

## 规范和路由

[SD01..07受管核验](spec-delta-reviewed.md)与[frozen-handoff](frozen-handoff.md)冻结本轮拟沉淀文本。T08完成指形成准确、可审的交接及规范正文，**不是canonical已发布或T09已审定**；新component在A阶段已存在的文件，B阶段应在现存文件受管增补，保留A发布事实。

本次勾选T05～08并记录14/17后，**next=T-09全计划独立终审**。T09继续未勾，Plan未标reviewed。最终pass之后才由T10执行冻结裸cargo t+实际触面档、777 master并集与必要发行接线核验、串行master合入；T11随后Spec/ledger/发布收据/归档和guard清理。merge-owned门禁全部明确pending，本轮未提前执行或宣称正式交付。
