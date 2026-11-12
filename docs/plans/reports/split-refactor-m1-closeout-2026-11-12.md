# 拆分与重构M1收口确认与联合标签交接

日期：2026-11-12。用户本轮确认在013静态生成AddTodo未交付的边界下进入本路线M1收尾；上轮已明确将该项安排到M2。本文汇总已正式交付的来源/验收、M2移交与联合出口状态，不新增功能、复活原材料或重开归档计划。

**当前结论：本路线M1交付与定向验收按已接受边界闭合，本轮完成收口文档整理；联合v0.6-M1标签仍待当前领域签章、到期批门与最终跨仓清单。** 本轮没有重新运行产品验收，没有签署新的独立终审、创建标签或发布。

## 1. 已交付范围与正式依据

| 面 | 已完成交付/验收 | 可取回来源 |
|---|---|---|
| 仓界/独立消费 | 既定拆分、语言L1/L2、UI/Man/Web真实消费与必要闭包 | 路线既定归档链；[989 r8](../archive/989-split-refactor-joint-delivery-and-tag-readiness.md)最终验收 |
| 编译产物治理 | 保留910共享target；942集中workspace研究conditional，不采用未证明额外收益的集中方案；1001治理研究、1003 C生产默认/full恢复/身份与有限净益 | [942](../archive/942-generated-workspace-incremental-sharing.md)、[1001](../archive/1001-incremental-cache-lifecycle-and-generated-debug-policy-validation.md)、[1003](../archive/1003-generated-debug-policy-c-production.md)及[C生产Spec](../../specs/repository-boundaries/generated-debug-policy-production.md) |
| Windows RQ | UI通用绘制/资源/运行、双Host装配与OS068明确修约的实际支持域；按真实断言与适用格复用 | [OS068](../../../../auto-os/docs/plans/archive/068-windows-rq-remaining-fixes-and-full-acceptance.md)、[Windows交付Spec](../../../../auto-os/docs/specs/shell/windows-rq-complete-delivery.md)及989 |
| Web成员独立导出 | service-export、SDK/源/资源/锁闭包、移根locked构建、真实HTTP/持久数据与首版A/B恢复；P2关闭 | [1006 r1](../archive/1006-web-generated-member-independent-delivery-and-recovery.md)，11/11；library6/6、driver30/30 |
| 联合核查与有限补验 | 989归档9/9；1015归档9/9，Counter A/B/恢复A真实业务、013有限业务，服务/数据恢复及旁观隔离；D-989-CLOSE-01/02按已接受边界关闭 | [1015 r1](../archive/1015-split-refactor-closeout-supplement-and-contract-alignment.md)、[里程碑Spec](../../specs/repository-boundaries/split-refactor-milestone-acceptance.md)、[恢复清单](split-refactor-recovery-objects.md) |

942/1001/1003等个别归档头部current_step未同步最终收据；沿989/1015已核的最终review/merge/规范与清理记录确认交付，不改写归档计数，不由此重开。研究完成不等集中workspace go，有限工作集节省不外推全生态。

## 2. 收口运行基准与来源

本记录沿已验N及1015补验增量，**不是把此刻所有主检出HEAD重新签成已验运行组合**。最终联合标签必须另给可消费的全仓锚点及必要差额验证。

| 身份 | 完整Git对象/内容身份 | 说明 |
|---|---|---|
| 989被测语言N | `075f65ccb683d9c290e011200e68f3f404d015eb` | 冻结运行源；不从当前HEAD替换输入 |
| 语言1003交付 | `11e1f1fd5e79766266ce8d54356155f018b8d661` | C生产与P1关闭 |
| 语言1006交付 | `7296df7d848c628a00e65feec8e8799a6311471c` | Web中立闭包/提交支撑 |
| 989正式落地 | `28803252ec1d0890f88624ff35b3a0c71e2b22df` | 原核查/规范交付 |
| 1015正式落地 | `8abc862f45771389cb4c0b2a3067474b752f9e41` | 补验代码/规范交付，归档提交`d1ee64099ba1877f85a005da96d282c02e00617a` |
| N UI | `691047651590e716b8b77651ea68bbdc9773c119` | 当前UI主线`ad9c05da65c9f48ec8a5ab0ef8815cff1ded0fb4`相较该源只变081计划文档；运行证据仍钉N UI |
| N Man | `54fe4070bdbd06015ea4521074479ae2e2f8b14f` | CLI/SDK修复组合 |
| N Web | `8b1742a0f84c24173768265cb999a648039d1e2b` | 成员导出/SDK修复组合 |
| N OS | `868c80d3e697ad6eaa9cce27596430a9abe08d5a` | OS068交付及收据；O-RQ为`3ea7ead67d2da739a04c771209b1cc18a0a18210` |
| Down | `58ea33f699efa5cb1194473a2d45dcec51006842` | 原组合来源 |
| O-RQ真实Todo产品pin | `0510e0501b3783bd68b5ff6f00b7a5f81fc1d0f0` | 从OS正式树gitlink核得，非教学013内存backend |
| 当前milestone Spec SHA256 | `d9a257e6f274f4369adac55ded5fbc1c405b599a442e7d7504e0ef13de5843c2` | 1015落地规范字节与本轮读回一致，未在本轮重写历史ready |

上述关键交付均可取回；1003/1006/989/1015语言交付及UI/OS关键源已核为对应当前主线祖先。被引用的工件hash、运行命令/退出码、设备/策略/输入/断言和摘要留在归档计划；原scope已按政策删除，不要求复活原始exe/截图。后续真实生产源/工具/锁/资源/配置变动使受影响证据失效时，按差额定向补证。

## 3. 必须随M1交接的能力边界

| 面 | 本M1可声明 | 必须保留的限制/后继 |
|---|---|---|
| RQ性能 | OS068已批≤60Hz/DWM呈现、机制/协议与分层配对的有限证据 | 不宣称高刷/光子终点、字节级传输计数、replay-p95、原生输入≤100ms或独立GPU时长已测 |
| 1015 UI | Counter A/B/恢复A实际0→+→1→Reset；013渲染/勾选/过滤/计数等真实业务，UI20/20+controls | **013静态生成AddTodo仍不可用**，M2修复；不能从已测业务推导完整添加链路 |
| 1015 RQ恢复 | 服务Host/真实Todo数据/备份恢复/旁观隔离18/18；窗口面按068/989身份规则复用 | 未新跑standalone/desktop-RQ窗口演练，不写成1015全窗口重验 |
| Video | 已批demo机制与具名后端owner移交 | 产品扫描/流服务后端未因本路线收口自动实现，按原产品owner关闭条件 |
| 治理收益 | 1003原同源有限工作集11.64～11.70GiB→约5.08GiB及成本/恢复口径 | 不外推全产品节省比例，不把清缓存量当持续结构净益 |
| 支持平台/包版本 | 当前Windows支持域、首版受控包对及有限同schema恢复 | 不扩大成Linux全能力、任意历史跨schema升级或离线构建保证；源码交付不等用户安装/运行进程已更新 |

## 4. M2移交：D-1015-UI-ADD

- owner：AutoUI源/生成器域；正式实施Plan尚未建立，本轮不取号。
- 阶段决定：2026-11-12用户同意不影响M1时移至M2，并据此确认进入本路线M1收尾。因此本项不阻本路线已接受范围收口，也不单独阻联合M1；M1清单明确未交能力。
- 缺口：013静态生成将App.input和TodoStore.input分离，真实输入/提交不能增加任务；scope桥接探针只是修复方向，不是生产交付。不能按同名字段全局共享而破坏组件/Store隔离。
- M2关闭条件：核源显式绑定/生成语义责任，正式修复并交付可追源/工具，在实际013 Rust交付A/B/恢复A上添加唯一marker→列表+1→勾选/可见状态读回；共享触面才补VM/Vue。原教学013内存backend与RQ产品SQLite分别验，不能互抵。
- 跟踪入口：[D-1015-UI-ADD](../KNOWN-DEBT-AND-RISKS.md)、[路线战略](../../design/strategy/modularization-and-refactoring-strategy.md)。M2收尾前真实关闭或由用户另行明确裁定，不靠持续转交owner无限延期。

OS062仍drafting r2/0-13，是Linux发行应用准入支线，不影响本Windows M1收口。ACE/AGE M2、Linux发行版/App强化及版本新增专项仍按各自合同，不由本收口记录减免。

## 5. 周期批回归与联合标签交接

| 项目 | 本轮确认 | 交接条件 |
|---|---|---|
| 本路线交付/定向验收 | 已完成，989/1015及上表正式来源；AddTodo按用户决定移交M2 | 保有限声明，不自动重跑整个989/1015 |
| 周期批回归 | 最新收据covered_plan=1014、时间2026-11-12 07:29:40+08:00、covered_commit=`82fc90f86`；36次组调用/4524执行、known-reds-only/new_reds=[]，余plan988预算边界具名。**该提交不含1015落地** | 1015为5的倍数且已落地，周期门已到期；需按现行固定组在主检出单实例完成后更新收据，不以旧known-red或定向验收假称已覆盖 |
| 批门运行条件 | 本轮读到ACE侧cargo/nextest测试在运行，包括lang-1013组，及其他构建活动 | 按独占重门规则当前不另起全量；不停止/抢占/代操作他人测试，不把未跑算PASS。待独占条件满足后由批回归入口跑一轮、逐新红归因，非重新建立路线计划 |
| ACE M1当前签章 | 1013尚未正式交付，头部drafting/6-10；1016为execution_done/6-7 | 以原ACE会话最新实际采集/终审/正式交付为准。本轮已观察到其测试活动，不再用旧“仍暂停等待”记录断言没有推进；不在本会话接管或代签 |
| AGE M1 | AG026已archived11/11；语言本地存在v0.6-AGE-M1 | 联合时仍核AGE正式源/SDK与同一跨仓组合，领域标签不代联合签章 |
| 联合v0.6-M1 | 989旧joint=true是历史冻结候选结论；本轮未创建联合标签，未查询远端 | 当前领域签章/周期门有效→冻结最终跨仓源/工件/锁/资源清单、核新增差额→收口文档正式纳入提交→用户明确打标/推送指令。无须重开归档989 |

正式v0.6仍另守ACE M2、AGE M2、Linux发行/App及当前版本专项门，M1联合标签不是正式v0.6。

## 6. 清理与本轮操作记录

- 1006、989、1015正式实施组/旧组与对应plan-artifacts scope本轮逐路径核为不存在；与归档的guard/清理记录一致。没有在本轮删除或扫描处置其他未知目录、用户缓存/安装/数据或持有。
- 本轮仅核来源/祖先、Plan/Spec与最新批门/活动条件、整理本收口记录、同步路线及M2交接。不建新Plan/实施树、不重跑GUI/安装/恢复、不改生产/归档/现有canonical、不暂停其他会话、不提交或创建/移动/推送tag。
- 本记录是本轮工作区收口整理，尚不是新独立终审或已发布tag收据。最终联合标签冻结时必须把此记录/范围/M2移交与可消费清单纳入正式提交；未提交文档不得充当可取回标签锚点。
