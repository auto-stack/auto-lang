# PLAN-842 独立最终复审 R7：pass，合入门禁待执行

独立复审子代理 /root/review842；继承会话历史，但对本次修复从实际提交、规范、源码、真实安装及独立反例重新判断。计划 revision 2；同组专用工作树 clean、分支 codex/plan-842-dev：lang a8b4a5bcf53d9be484cc38f028453a4cebbc947f、UI bd0e0f38253c5fd0676c8ef729c8c030d372ae82、Man 9ac33ad871c4e0278b322e9712930ed059029e9c；down detached c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8。lang 基面为当前主线 a547f52c5fbd5a24a203e94051af1e040c808f42，UI 基面 4903bef64532750d1597627b1197f3d2cac1effc、Man 基面 9a75d3bab6eaeda961d5dcbf7932258f89e650af。三仓 HEAD、clean 和自基面 diff-check 审查前后复核通过。规范、别名与测试配置字节 [哈希](input-hashes.json)，[受审 SD 提案快照](spec-delta-snapshot.md)。

## R6 修复复审

独立复制当前真实 Full 安装 A、不同版本的真实 Full 安装 B、真实项目与有效旧交付；为旧 app 加哨兵并保存旧 app/manifest 字节。分别选择默认安装、显式同安装、显式另一受管安装 UI 源，在 UI compiler staging 目录出现且被测文件尚未复制时短暂修改该源文件，待 staging 目标出现后立即恢复源字节。三种实际 CLI 均证明变化被实际复制，随后 exit 1 具名拒绝，旧 app 和旧 manifest 逐字节保留；[独立摘要](independent-summary.txt)及 [默认](independent-default-generation.log)、[同安装](independent-same-generation.log)、[另一安装](independent-other-generation.log)。这直接覆盖 R6 的修改→复制→恢复组合；未以预检成功或返回码单独作为 oracle。

代码审查确认每个被选中受管 UI 源独立冻结 InstalledSdkReceipt；实际 staged UI crate 在投影前按该源的 owner/path/hash 文件集合校验，swap 前复核两份实际受管收据。工作期另以受控变化收据证实提交前拒绝并保留旧产物；普通开发 UI 以自身 Git 源身份正控通过。实际两份不同版本的完整安装 A、B 均 verify，读取产物见证 A 的 UI 来源 bd0e0f38253c5fd0676c8ef729c8c030d372ae82、B 的 UI 来源 ab81f0406e3401f94f960c60eb87ff460763ded3；真实交付 ui_source_root 指向 B 且 ui_git_head 等于 B 的收据，而非 A 或空值。工作期 [真实身份输出](../repair-r6/r10-real-other-ui-owner.log)、[收据变化控制](../repair-r6/ui-receipt-changed-during-copy.log)、[安装记录](../repair-r6/report.md)。F-842-R6-01/02 在上述范围 CLOSED。

## R1–R5 与验收映射

| 验收 | 证据及本次判断 |
|---|---|
| AC-01 | [冻结 R01–R16](../retirement-matrix.md)、[外部旧入口与 canonical 正控](../t04-verification/report.md) 和 [矩阵/ID 对账](../t08-audit/report.md)：四语言过渡模块、两 UI 别名、旧 builder 和 use-bigvm 已实际退役，正式薄 API 与必要宿主登记按合同保留。原相关生产、测试、依赖输入在本轮未变，证据限定复用。pass。 |
| AC-02 | 同源、独立受管 UI 与开发 UI 三种来源、生产和生成器调用、受管源码 owner/字节/身份对齐；R2/R3/R5 坏语言根、Windows SDK 大小写、包图与本次 UI staged 反例均有实际拒绝和恢复，R6 原项关闭。pass。 |
| AC-03 | [T-06](../t06-progress/report.md) 中纯 SDK/无 UI、已链接 UI 默认、显式 provider、双实例、线程/退出与登记消融控制；生成应用宿主实际 probe。最近修复不改宿主实现及依赖输入。pass。 |
| AC-04 | 旧 Core/Full 升级/恢复、资源/清单未知版本、备份失败恢复与 R1–R6 坏源保旧实际控制；[T-05](../t05-progress/report.md)、[最终安装报告](../repair-r6/report.md)。UI source receipt 独立版本已从真实不同版本两安装确认。pass。 |
| AC-05 | 测试 ID/alias、迁移文档、[最终选择摘要](../repair-r6/final-selection-summary.json.txt)及实际 nextest 列表：本次独立校验四份原始列表 SHA-256，21/21 映射目标实际 listed+selected；UI 9 项 integration target 确在命令，旧基线以明确映射保留。language 3212、UI 3117、tt 3563、tb 3102；这是 list-only，绝非全档执行。9+10+2 实际定向测试和 18/18 driver 正反控在代码及配置未变时复用。pass。 |
| AC-06 | 本次独立 final、提交/依赖/配置/规范绑定、R6/R5 及相关反例闭合；[语言来源测试](r7-independent-lang-tests.log) 6/6、[UI 控制](r7-independent-ui-tests.log) 7/7 本轮独立重跑。pass，合入门禁仍 pending。 |
| AC-07 | merge-owned，未执行；下述冻结门、并集实际合入、规范及 ledger 回读、归档和 guard 清理均待后续。 |
| AC-08 | [T-00](../t00-scoped-verification/report.md) 合法源码/中间安装投影/已投影态及结构坏态，最终真实 Full 安装、fresh/locked 交付和失败恢复、Core 隔离与外部 consumer，结合此次三种受管 UI staged 反例。[最终来源报告](../repair-r6/report.md)。pass。 |

R5 受管 SDK Windows 大小写身份、无效版本的两入口实测重演、独立实际 Cargo 合法省略版本/继承及坏继承控制见 [R6](../review-r6/report.md)；R4 的普通/构建环、dev 环、别名、版本/身份及 workspace 继承的实际 Cargo 对照、R3 的非 UI 源/根输入/锁/patch/复制期以及 R2 的坏根拒绝按具体未变输入与实际最终安装日志复用。两种 Full UI 来源实际 staging 漂移在本轮独立重放。

## 规范与健康

SD-01/02/03 的 0.2 退役与正式保留、非 UI 领域合同、宿主默认和 runtime 迁移与实际代码/矩阵一致；SD-04 的显式来源优先、各实际受管来源冻结收据与 staged bytes、相应 revision 和开发源边界已核本轮实际修复；SD-05 按 R01–R16 增加持久退役规格；SD-06 的安装投影、schema/profile 与失败保护由既定实际正反控支持。十个既有目标路径存在，新目标尚待 merge 创建；GOAL-003/007 可在现有 ledger 中定位。SD-01..06 审查通过，canonical 文档与 ledger 本轮未修改；如合流改变规范或源码，必须审受影响内容及重新绑定证据。

三仓实际基面至 HEAD diff-check 通过；新增改动范围未发现 debug 打印或新的相应代码 warning。现有编译日志仍含大量基线 warning（例如 Core 145、UI runtime 216、UI tools test 19，配置不同计数可能变化）；既有 warning 不宣称清零，后续统一门须如实保留并归因。MPV DLL/真实视频播放未实测；原 1 项 ignored 不冒充通过。jet/ark 实构建依 canonical 832 用户裁定转 v0.7 环境门控，当前不计实测，不豁免其他门。

## 待合入阶段

本次 pass 仅表示实现、工作期验收与 SD 复审通过，不表示交付已完成。merge 须在冻结三仓、当前合流与依赖输入上执行裸 cargo t 及适用 VM/编译器、trans、UI 生成触面档（按实际迁后 owner 等价入口），对已记录的 plan707 cancellation 红做实际归因；按到期规则在语言主检出单实例执行 tf 批量回归。上述全档均由 merge/regress 所有，本次未运行。完成门后方可合入、对来源/规范/ledger/索引读回、归档并按逐树 fresh wt-guard 清理；任何代码/依赖/配置/输入或并集改变须复审受影响内容、重跑受影响门。
