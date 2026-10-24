# PLAN-840 最终交付与收尾核验

结论：**PASS**。原定验收及后续授权 Phase2/3/4 已闭合，原支持面未缩减，计划保持 archived/delivered、26/26；没有重新打开计划或重复合入。

本轮 inspected main：`e30b9051cf68639f4278dc2d41f7c168ffc11ae9`。原行为实现冻结 `1f19395254d7e676bd188e43e3d8f734d57aaa12`，正式图形分支 `8b62db5cba54b9bfe7f759f4920b22ccb9b75e88`；修复复审冻结 `ba6479da53b7999e490dd0e8ac8d701d5ffe86b2`，主检出落地 `53298cd273e0c04fabc508745ce342655394f1a4`，最终归档/清理提交 `c0705306371d4c2c399f9040577be17aa52d3fcf` / `01c421a1a4ec9622a287be33f5251fff0cdd04ff`。全部祖先关系、分支实际指向与代码树对应成立。其后两次提交仅为其他计划文档，未改变已测试生产树。

审查限制：同一对话/实现上下文，没有另一个会话、模型或 agent 的隔离独立性。本轮从 Git 对象、提交链、当前状态、原始断言及哈希重新构建结论，未以实施总结或 checkbox 作为证据。是历史交付核验，未重新跑原生/GPU/Cargo完整档；原完整证据的代码、依赖、配置和输入身份可复用。生产者旧 exe 随工作树按合同清理，身份由交叉绑定的原始资产/进程报告核对；保留的独立消费者 exe 另行实际重算 SHA，匹配最终原生记录。

## 逐项验收

| AC | 结论 | 实际核验及证据 |
|---|---|---|
| AC-01 | pass | G839正式基面/唯一树/冻结来源；SDK932固定引用、两graphics锁及后续六root/worker锁、源299/SDK681和独立消费检查保留；scanner三个提交stable patch-id相同 |
| AC-02 | pass | SourceSnapshot有界读取、256KiB/64KiB/320KiB、原字节/CRLF身份/来源映射/每次Run重读；原行为测试及用户场景源SHA逐trace核对，未改字节绕过scanner |
| AC-03 | pass | 真实Parser/AST准入；共享Rules遍历表达式并在嵌套函数边界重置Unit规则；源序const/let与常量FStr合法，动态/可变/私有/循环反控保留；原三finding及review-r2关闭表匹配实际1f源码 |
| AC-04 | pass | 从Git API源确认19个公共函数；handle0/缺actor、parent/world/effective读后写、仅local translation/visible写面；原API-error Fault、全mesh像素及源轨迹断言未移除 |
| AC-05 | pass | Builtin6与User7并存，初始化/回调/预算/暂停/中性Step/Stop/Editor隔离、Fault/last-good保持；实际Play各正例27phase、6阶段像素、owner反控核验 |
| AC-06 | pass | 正式UI/CLI入口、Move/Patrol/速度变体/不同cwd/搬迁根/Save边界；两套13场景同binary SHA、原用户source SHA与trace相符；搬迁与缺源反例实际生效 |
| AC-07 | pass | 原profiles/素材/128节点/600tick/20轮A-B/owner与计量边界；旧完整门、真实弱owner和GPU资源反控保留；未把容器计数升级为硬内存额度 |
| AC-08 | pass | 63有效门全部原始断言重放；16官方AGP、旧游戏/属性/交互/partial/reopen/层级/资产/Play/owner、新用户及独立消费者矩阵齐；DPI真实输入及counter+content双反控成立 |
| AC-09 | pass | R1三finding→修复1f→R2受影响审查和相邻合法控制；清楚保留当时merge pending及同会话限制，不以旧审查冒称新代码被测 |
| AC-10 | pass | 原完整门后合入autographics-dev8b62；与1f的可执行图形源码一致；master图形目录未被原840源码补丁污染；SDK精确四文件补丁、canonical/ledger/归档/guard/到期批量收据闭合 |
| AC11 | pass | Case/AV/CTE/probe注册与可选test-cases边界一致，off/on原定定向证据保留，未扩大默认依赖 |
| AC12 | pass | 全档选择面核销；full/t3实际组成员0→全部23条ffi_dual，原feature/filter/ABI oracle不变；最终各完整档均保留原skipped基线并增加3新回归 |
| AC13 | pass | 四个共享Instant边界闭合测量空隙，原30样本P95<1000ms及tiling/墙钟输入不变；计时两项独占当前nextest池而非单组伪隔离；最终完整档通过 |
| AC14 | pass | 定向类型/格式/源锁/SD与提交冻结可追溯；原失败与修复控制未删除 |
| AC15 | pass | r7/r8受影响审查覆盖真实分组及新增生产路径，完整门明确后置，最终原始7条整档记录与冻结绑定 |
| AC16 | pass | master修复落地53298；主检出单实例完整tf/tt/tb全绿后收据835→840，规范/台账/归档与清理提交齐 |
| AC17 | pass | Init完成态实际多片正例及原早退/截止反控、48实际病例，生产quiescence helper未被偷改；原几何/交互assert保持 |
| AC18 | pass | 有界完成驱动与真实RQHost服务端pending发布后再断言；原退出/双窗/boot断言及4真实管道控制保留 |
| AC19 | pass | Phase3冻结/规范/相邻控制闭合；审查、完整门与合入身份链成立 |
| AC20 | pass | r5/r6历史门记录保留，最终新冻结完整门和主检出批量补齐，T-10/T-16在实际清理后才标完成 |
| AC21 | pass | Windows实际延迟删除错误5红例、原计数100、stale/硬错误/25ms截止降级及释放后Executed控制；2s缺省/10s stale和best-effort合同保持，未误称降级后仍互斥 |
| AC22 | pass | SSE首次pull固定任务分片冲突红例、旧随机首次pull120s挂起；clone Arc释放registry guard；内层单点撤回mutation仍红，嵌套值/EOF/任务回收与原5s/3s、600ms内spawn<30保持 |
| AC23 | pass | 34定向、4病例50轮=200次、4冻结复审回放及check/fmt/六锁；规范SDL实际源码一致，最终统一门完成，暂无未闭合finding |

## V-01..10 支持面核销

| 验证项 | 核验 |
|---|---|
| V-01 来源/旧profile | 固定SDK/AGP、source身份与漂移/缺资源/锁反控、旧16官方case及原分母保留 |
| V-02 源快照 | 原边界/UTF8/CRLF/缺源/BOM/版本/来源映射、运行中不热载及新Run读新字节保持 |
| V-03 AST/恢复 | 原10规范反例与18嵌套return/4合法运行/常量与动态相邻控制、Host坏源→修源→实际pixels→Stop完整 |
| V-04 API/帧 | 19函数全部在冻结源；首错粘滞、handle/group/child/world、非法API/实际全part像素反控保持 |
| V-05 callback/预算 | profile6/7、完整初始化、tick序、Pause/Step/Stop及坏输出/预算故障原判据保持 |
| V-06 正式入口 | Editing选择与CLI、活动态拒变更、path/SaveAs/root/no fallback断言保留 |
| V-07 双示例/搬迁 | producer/consumer各Move、Patrol、fast、relocated四正例与9失败场景，binary/source SHA及轨迹/像素核对通过 |
| V-08 生命周期/容量 | 128nodes、20轮、实际弱owner、资源不重传及leak/upload反控、旧owner继承闭合 |
| V-09 OS/GPU正反 | 两套原生13场景及旧native家族、真实OS点击/失焦/partial/reopen，counter正与实际content失败可区分 |
| V-10 文档/移交 | SD01..05/八canonical目标、API/示例/入口/诊断/预算/版本边界与生产I2/I3非目标保留；后续SD-P2/P3/P4增量一致 |

原始门禁复核使用既有审计器的全部业务断言，适配仅限读取已提交.json.txt、已清理工作树的历史身份校验和审计结果输出位置，详audit-adaptation.json.txt。没有重写历史validated记录。全部248原.json都有受版本控制的原件或对应持久.txt。两套用户native各13用例均覆盖4正例（各27phase、6阶段pixels）和9有区分力的预期失败；旧交互wrong/down/drop/partial/truncated及资产/Play/VM/GPU owner错误均按具体失败阶段/原因核对。

## 完整门禁与落地

| 阶段 | 完整结果 |
|---|---|
| 原Phase1 | 63有效调用，driver140/AGP163/独立consumer66+4；原root5332/5332及tv163/163 |
| 最终冻结ba6479da5 | bare t5345/5345、tv163/163、tt5717/5717、tb5316/5316 |
| 主检出53298批量 | tf6291/6291、tt5717/5717、tb5316/5316，0失败/超时；skip518/1800/1548与先前一致 |

复算raw日志Summary、每条调用exit/expected、冻结code_commit、源树/六锁/auto-down身份和批量receipt，未只采信汇总JSON。归档之后生产树未变，故历史门可复用；没有无理由再跑完整档。

## 规范、归档和清理

- 规范与ledger：三次投影78→79（11目标/1171无关raw项）、79→80（6/1180）、80→81（6/1183），前后SHA、JSON内容、所有无关item原始字节及顶层字段均重新验证。20个受影响当前索引条目的source-sha256均匹配实际canonical或审查文件。Regime B offline通道如实标识。
- SDK扫描三份实际Git补丁stable patch-id均为842d36b4e00b54af7d6320b2b9482f04771a774a；不是仅凭来源声明。图形原交付1f→8b代码一致，后续修复仅master既定触面。
- 归档26/26有Git提交，不是磁盘未追踪状态；未为840另建修复计划；此前846草稿撤回记录保留。终态保持archived，不重新开启。
- auto-lang和auto-down两注册/目录均已不存在，plan-840-dev不存在；各有fresh guard clean和移除日志，SDK固定引用与autographics-dev实际指向正确。剩余组目录为有意保留的非Git消费/复现资产，不属于遗漏worktree。
- 两轮既有失败与诊断记录保留；21个Phase1及47个修复文本的原始字节可重构且SHA复算通过，未用整理日志消除红例。
- 主检出既有P796报告与批量前字节一致；历史38条外部WIP保护收据存在，本轮只写840审查记录，不改无关WIP、源码、canonical或实时ledger。

## 审查发现与边界

F-840-C01（记录可读性，已关闭）：正文§8与§10用“当前”描述r2旧11/12/pending，易与r8最终26/26混淆；本轮仅将其明确标为r2历史检查点，并指向r8最终收据，不修改任何验收合同、旧失败/门禁数据或计划revision。无代码/支持面/完整门/规范/清理阻断finding。

D-762-1生产UI quiescence helper仍为原有范围外债务；840完成的是测试载体实际完成态等待，没有冒认修复该生产helper。生产I2/I3、通用稳定SDK/安全沙箱、任意容器硬配额等原非目标保持，不把非目标变成“遗漏”。既有编译warning保留处理边界，没有借本次历史核验宣称全仓零warning。

next：无需work/merge/再次清理；归档状态保持，提交本次核验附件与历史检查点说明即可。
