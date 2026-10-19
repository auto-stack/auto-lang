# v0.6 剩余迁移总路线图：按完整模块交付

更新：2026-10-20；路线版本：6。809/I-PROJECT、810完整文法、811媒体均已交付归档；814–816修订r2按A/B阶段发布与并行审查修复组织。当前来源与状态见[本轮输入](../attachments/sdk-tools-r2-20261020/inputs.json)，历史快照不覆盖当前调度。

本文件是剩余迁移工作的统一调度入口，承接[Design 37](../../design/37-v06-modularization-and-repo-boundaries.md)。用户已明确：后续提高模块粒度，先整理交付目标、依赖、并行边界及搬仓验收，**暂不为全部工作包分配Plan号**；已按滚动请求建立807–811，本轮仅修订既有814–816三份合同，不分配新编号。812/813是其他路线，不算迁移计划。R00–R12是路线工作包标识，不是实施计划号，也不改变在途Plan合同或授权执行/切仓。

当前规则以[Specs总览](../../specs/overview.md)及各模块Spec为准；本图描述未来目标，不发布为已实现规范。过去按批次追加的时间线从本文件当前正文退出，保留在Git历史与文末批次报告中，避免多个“当前状态”并存。

## 1. 当前切换点

| 范围 | 本轮可核实的交付/状态 | 对后续的意义 |
|---|---|---|
| 789/793/799/801 | 词法、诊断、共享兼容AST、轻方言接口已合入；[auto-ast](../../specs/auto-ast/project.md)、[auto-parser-api](../../specs/auto-parser-api/project.md)已有Spec | 无须再次规划AST/端口抽取；AST仍含UI兼容节点，完整UI文法/默认组合未解耦 |
| 779/785/790/794/802 | frame、文本/媒体剪贴板、scroll及editor22项已交付；[VM宿主合同](../../specs/auto-lang/vm/overview.md) | 不重复迁这些族；diff四项另由807交付，term/media继续808/811 |
| 780/786/791/795/800/803/805 | 已有project-model有限消费、gen-core、包锁、resolver、Man资源与工具链规则；见[Man Spec](../../specs/auto-man/project.md) | 纯规则地基成立；Target/Pac/管理编排/构建/CLI仍未整体独立 |
| 803簿记 | delivery `89bb13944`已是当前HEAD祖先，资源与Spec已合入；归档收据存在，活动副本删除等工作区簿记仍由原会话处理 | 按已落地实现取材；本轮不代提交其删除/清理或重新打开计划 |
| 792 | [选定Rust/Iced交付合同](../../specs/repository-boundaries/generated-ui-distribution.md)已交付 | 是可复用发行地基；受管包仍含语言大库，不等于全部UI/Man独立发行已成立 |
| 784 | [旧仓保全合同](../../specs/repository-boundaries/project.md)已交付，准备树/恢复载荷有交接 | 搬仓时重新核对源仓状态及恢复能力，不另建auto-ui/auto-man仓，不先清空旧默认分支 |
| [804](../archive/804-dialog-dnd-host-provider.md) | archived 8/8，合入及清理终收据`601954cca` | 对话框/拖出已交付；后续装配保留其真实接口及语义 |
| [806](../archive/806-ui-declarations-dialect-extraction.md) | 实际交付`c4158b89e`，归档/清理收据成立；header仍7/8、尾部收据8/8为原簿记差额，本轮不代改 | R01前置已满足；msg/model不是完整UI文法 |
| [807](../archive/807-text-snapshot-diff-core.md) | 已交付文本/分页/快照/diff，delivery `63fe29fd9`，归档清理终收据`4880ba708`，10/10 | R02已完成，不再次规划其数据算法；R04消费已合入文本core |
| [808](../archive/808-terminal-host-domain-extraction.md) | 已交付5833e155e，终态归档cabe023e2、10/10 | 终端已满足R04前置，不再重复等待 |
| [809](../archive/809-man-project-management-build-core.md) | archived 11/11，实际交付b819a602e | R05/I-PROJECT已可消费，815协议阶段不等814 |
| [810](../archive/810-ui-grammar-compiler-entry-extraction.md)／[811](../archive/811-media-host-runtime-extraction.md) | 均archived 11/11，实际交付f0ec8a91f／d06cdc6f6 | 814文法/媒体前置已满足；本轮不代重新验收 |
| [814](../814-language-sdk-ui-package-separation.md)／[815](../815-ui-project-tools-and-assets-extraction.md)／[816](../816-tool-execution-protocol-and-thin-cli.md) | drafting r2，原进度0保留；814已有登记树 | 按A/B阶段与任务门组织，独立scope实施/审查/修复并行，完整UI集成等实际发布 |

以上是来源明确的快照，不是本会话重做验收；具体完成范围以Spec/merge收据为准。当前其他AC/AutoDown等在途工作按真实共享写面协调，不作为全局停工条件。当前来源见本轮r2输入快照；[v2原输入](../attachments/extraction-roadmap-20261018/inputs.json)保留历史。

## 2. 两个明确里程碑

**M1：首次三仓独立交付。** auto-lang基础SDK、auto-ui、auto-man各有可消费边界，三仓能独立解析依赖/构建/测试/打包，生成工程与CLI组合实际可用；语言基础正常闭包不反依赖UI/Man实现。UI兼容AST及有期限的上层兼容入口可暂留，必须列明归属和退出路径。

**M2：干净解耦与减法收口。** UI专属AST/关键词职责归UI领域，旧兼容层按消费者迁移后退役，残留反向边与重复实现清理；语言内部模型/前端/VM/codegen/trans职责进一步清楚，同口径全生态源码/依赖指标可复核。M1完成后不能把M2默默取消。

数据与通用runtime模块先保持独立可消费的crate边界，其仓名/单独建仓与发布渠道可后续决定，不设为三仓首次交付的额外前置。全语言IR重写、后端退役、业务新功能不自动纳入本轮迁移。

### 计划粒度与数量口径

- R00的804/806已有实际交付，不为收口再开新计划，也不重新拼装在途工作。
- M1余下归为R01–R09 **9个完整工作包**，默认每包一份较大的Plan；一个Plan包含生产接线、旧实现清理、资源/测试与独立消费，内部3–5个内聚任务阶段不再各自取号。
- 新增实施计划预算仍按**约8–12份**：R01/R03/R04若存在真实的独立语法/生命周期/发布边界，可各拆两份；R07发行验收若完全纳入R08/R09合同，可以不单独建Plan。依据可交付/可复审/可回退边界调整，不按文件数、函数数或实现耗时拆。
- M2归为R10–R12三包，约3–5份Plan。估算是排程预算，不是压低验收标准的数量上限，不预占后续编号。
- 上一轮“7项待合入”及804/806待最终交付均属旧时点；当前808–811已交付，814–816已按r2阶段立约。上述九包与8–12份是整段M1编排基准，含已建立/已交付项，不是今天尚需新增数量；已有工作量、待建合同、仓库切换分别计数。

## 3. 目标模块及允许依赖

| 模块/所有者 | 完整交付职责 | 正常/构建闭包红线 |
|---|---|---|
| 语言基础与SDK／auto-lang仓 | 来源/诊断/兼容模型、Core解析与语义、编译执行/转译入口、LSP等基础消费 | 基础闭包不依赖UI grammar/renderer/Man管理实现；类型/推断存储有实际归属，不靠公开私有Parser或复制模型 |
| UI编译与运行／auto-ui仓 | 领域文法、UI分析/IR/生成、运行桥、renderer、UI工具及其资源 | 依赖语言SDK与轻工具协议；不拉Man管理大库或语言私有实现，不要求语言基础反向链接UI |
| 工程管理与工具／auto-man仓 | 项目/依赖/锁/构建规划、工具协议、语言/系统构建adapter、薄CLI | 管理核心无完整VM/renderer依赖；语言执行与UI工具由明确adapter/进程协议接入 |
| 共享数据与运行能力／各自明确所有者 | 已有值模型、通用规则、runtime/capability合同与非UI实现 | 无Parser/UI/Man反向边，不建立装所有跨界类型的common巨包；一份类型/算法来源 |
| 过渡兼容层／上层组合所有者 | 旧公开入口、默认宿主选择、弃用提示与迁移桥 | 必须在基础闭包之外，有消费者/期限/退出条件；不能一面依赖完整UI一面宣称同一基础包“纯Core” |

R04必须解决crate DAG与旧根API的组合位置。若旧`auto_lang::ui`等入口必须暂保，需以实际包图选择上层兼容承载与纯SDK入口，连同workspace解析/发行闭包一起验证；不能把UI re-export重新塞回基础SDK制造环。涉及公开路径退役或支持范围变化时，由具体Plan明示兼容方案及所需决定。本路线不预授权破坏已有接口。

## 4. 完整工作包及交付目标

“硬前置”约束依赖接口的实施/最终交付；无关调查、测试资产和独立子任务可以先做。“并行范围”不授权多方同时改同一生产接线。

| 工作包 | 完整交付目标与退出证据 | 硬前置 | 可并行范围／推荐Plan数 |
|---|---|---|---|
| **R00 现有合同收口** | 804真实模态/拖出和806完整msg/model均有实际交付/归档/清理收据 | 已交付；806 header簿记差额由原owner处理，不重开合同 | **不新增Plan** |
| **R01 完整UI文法与编译入口** | widget/component/view/fragment、props/setup/refs、store/computed/on/actions/style等既有支持面全部归UI文法模块；Core expr/type/body由通用端口委托，默认UI入口/递归/导入保持装配，原Parser算法删除。完整正反语料、源位置/恢复与Core view参数/task on/同名变量对照通过 | 806实际合入；799/801作为已交付基础；端口差额有界设计 | 可与R02/R03/R05并行；widget结构与事件/样式相互调用先形成闭合图，不强拆成多个同改Parser的计划；**1–2** |
| **R02 文本／快照／diff基础模块** | rope/snapshot共享数据与diff算法有最小独立边界；行/文件/快照/目录/窗口四native及真实UI消费者使用单份规则；UI registry/FS观测有明确adapter。保持剪枝、serial/parallel等价、窗口为全量投影、换行/Unicode及错误envelope，纯算法无UI依赖 | 802已交付；盘清RopeSnapshot实际依赖。无需等待R01或804 | 可现在独立推进，不能把纯diff算法盲包成OS provider；与editor运行桥只协调公开面；**1** |
| **R03 终端／媒体等剩余宿主域** | 对实际目录/调用图穷举剩余能力，迁移term多会话/配置/几何/背压，以及其他实际仍跨界的媒体/窗口等能力；定义/编解码/实现/装配分层，真实VM/a2r消费、线程亲和、队列、水位/句柄、取消/退出/资源清理完整 | 已交付provider模式；具体子域共享类型/生命期先冻结。804仅在共同装配/整体闭包验收中合流，不是所有新族实施前置 | 可与R01/R02/R05并行；同一有状态族只一个owner，独立终端/媒体生命周期才允许分开；**1–2** |
| **R04 UI分层、语言SDK与默认组合出口** | UI模型/分析生成/运行桥/renderer成为职责清楚的包组；Man内的UI生成器及其gallery/wm/shadcn等专属资源归UI工具，项目配置通过轻合同输入。语言基础SDK与UI默认组合分离，Core消费不拉UI实现；既有支持消费者从公开入口工作，私有API/全局注册/缓存归属清楚，旧入口兼容有承载 | 最终交付需R01/R02/R03及804、806相关交付；从Man抽UI生成器还需R05的**I-PROJECT轻数据合同**实际合入。可先设计包图/公开出口和无关子模块 | 与Man线并行；不得与R01/R03同时搬其核心文件，也不与R05改同一Pac/生成请求接线；**1–2** |
| **R05 Man项目／管理／构建核心** | Target/Pac的声明数据、包管理/依赖/锁/构建规划及纯规则形成可消费核心；先冻结I-PROJECT（轻项目/生成请求数据：来源/后端/资源与输出根等），供UI生成器脱离Man大库。复用既有轻包，动态pac/am.at/模板、转译、Git/FS/外部工具留具名adapter；真实生产命令切核心，旧重复扫描/规则收口 | 现有780/786/791/795/800/803/805基础；无799/801/R01接口依赖 | 可现在与UI/宿主线并行；轻合同先交接，静态/动态支持面与旧失败语义冻结，不把静态子集当全部pac替代；**1** |
| **R06 工具协议、构建adapter与薄CLI** | 统一请求/能力/版本/producer/资源输出根；UI工具与语言/服务构建adapter明确，auto命令flags/stdio/exit/取消/watch/daemon及前后端路由保持；管理核心不导入UI大库，真实CLI→工具→产物可运行 | 完整交付需R05核心及R04公开UI工具/SDK出口；协议/非UI路由可先实现，不靠fake替代最终UI生产链 | 协议定义、进程控制与未重叠命令可先行；与R04/R05各有一次接口交接；**1** |
| **R07 发行闭包、资源与消费矩阵** | 将792现有地基扩至新的SDK/UI/Man/CLI实际支持面：每个profile有来源/lock/patch/producer/cache/模板/schema/stdlib/资产/构建输入与安装路径；干净环境独立fresh解析、locked构建、重定位、升级/失败保护可复跑 | profile发布以R04/R06及其真实依赖为准；清单/driver设计和每次局部闭包验证从现在持续做 | 可按UI/Man profile分别推进，不等全生态停工；迁包者持续维护清单，不能把漏包推迟到此包；**1，可并入R08/R09** |
| **R08 auto-ui旧仓承接与独立交付** | 将已验收UI包组/工具/资产/测试/示例/脚本/CI/许可证导入既有auto-ui迁移分支；来源历史与旧legacy可恢复；独立仓和真正外部工程使用新版本，语言基础无需UI checkout仍可构建 | R04、R06的UI工具协议、R07 UI发行profile；784保全须按当前源状态复核 | 与R09可并行做owner清单/保全/导入准备；涉及语言消费切换的窗口串行；**1** |
| **R09 auto-man旧仓承接与三仓组合交付** | Man核心/adapter/协议/CLI及资源进入既有auto-man；与固定UI、语言版本完成包管理/构建/运行/生成/服务/watch组合验收，独立manifest/CI/锁与恢复/升级流程可用；语言仓迁出实现删除 | R05/R06、R07 Man profile；最终三仓验收需R08可交付UI版本，准备/非UI命令验收不必等R08全部结束 | 与R08准备并行，最终消费版本接线和默认分支切换有序；**1** |

R01包含完整结构/事件文法闭包，不能把“搬几个parse函数”当退出证据。当前映射：**R01=810、R02=807已交付、R03终端=808已交付/媒体=811、R04=814/815、R05=809、R06=816**。811负责图像及本地媒体服务，i18n/a2r frame等运行桥残边由814具名收口；未清偿前不得宣称所有宿主边清零。R04/R05只完成三仓解耦所需拆缝，不顺手重写整个编译器/格式。

**I-PROJECT属于已交付809，I-TOOL属于815，均不另占Plan号。** 815-A只依809，交轻协议/同源codec与真实窄端点，不引AST/VM/renderer/Man；816-A实际消费已发布端点。814-A发布真实SDK/编译出口后815主体和816管理adapter可展开；完整UI生成/运行与CLI集成仍分别等814-B/815-B。阶段发布须独立审查、冻结门、实际master/Spec来源收据，不引用WIP、不复制请求或假backend。详见[阶段合同](staged-delivery-review-814-816.md)。

### M2工作包（继续保留，暂不分配Plan号）

| 工作包 | 交付目标／退出证据 | 前置与并行 |
|---|---|---|
| **R10 UI领域AST／lowering终态** | UI专属节点和关键词分类归UI，基础AST不再拥有UI声明；跨模块/来源/序列化/Clone/诊断/生命周期与lowering→类型/VM/各生成器全链保持，未lower的扩展有明确拒绝；无全局Any/side table规避所有权 | R01/R04的公开类型和完整消费者稳定；可在搬仓发行准备期并行设计/验证，不强制等R09才开始。进入终态必须全链验收，不能以AC小原型替代 |
| **R11 兼容入口／旧发行路径退役** | 消费者已切新API/包源/资源机制；按逐入口清单移除上层兼容壳、旧直接UI臂和遗留隐式路径，原命令兼容或有经确认的弃用迁移合同；旧默认分支的legacy恢复引用继续保留 | R08/R09和相关消费者迁移已交付；依赖R10的入口等待其完成，无关入口可先退役，不一次删除所有兼容层 |
| **R12 语言内部组织／去重／净瘦身** | 模型/前端与推断存储/字节码元数据/VM/codegen/trans/SDK责任清楚；代码拆分及重复实现/无消费者路径按证据退役；共享数据/runtime保持可独立消费，三仓统一口径的净删/依赖/构建指标可复核 | R04/R05建立的接口及稳定消费者；纯去重可并行，改全局模型/ABI/缓存/loader须停在实际交接窗口。后端退役和新IR替换单独决定 |

三个M2包预计3–5份较大Plan；不能用“代码已搬到旁仓”计作全生态净删除，也不把数据/runtime必须另建Git仓作为新增硬门。

## 5. 依赖关系与关键路径

下图实线表示最终交付硬依赖；“既有基础”均需以实际merge/Spec为证。包内无依赖部分可以先做，最终不能越过对应出口门。

```mermaid
flowchart LR
    P806[806已交付] --> R01[ R01 完整UI文法与入口已交付 ]
    P802[802已交付] --> R02[ R02 文本与diff基础 ]
    Base[既有provider基础] --> R03[ R03 剩余宿主域 ]
    P804[804合入] --> R04[ R04 UI分层与语言SDK ]
    R01 --> R04
    R02 --> R04
    R03 --> R04
    ManBase[Man轻基础已交付] --> R05[ R05 Man管理构建核心已交付 ]
    ManBase --> IP[ R05内 I-PROJECT已合入里程碑 ]
    IP --> R04
    IP --> R05
    R04 --> R06[ R06 工具协议与薄CLI ]
    R05 --> R06
    R04 --> R07[ R07 发行profile与外部消费 ]
    R06 --> R07
    R07 --> R08[ R08 UI仓承接 ]
    R07 --> R09[ R09 Man仓与三仓交付 ]
    R08 --> R09
    R04 --> R10[ R10 UiAst与lowering ]
    R10 --> R11[ R11 兼容退役 ]
    R08 --> R11
    R09 --> R11
    R04 --> R12[ R12 语言组织与净瘦身 ]
    R05 --> R12
```

完整交付关键路径仍包含R04→R06→R07 UI profile→R08→R09三仓验收，但内部不再整项等待：**809已交付→815-A→816-A**与**810/811等已交付→814-A→814-B**并行；814-A/815-A放行815生成主体，814-A/816-A放行816管理adapter；815最终runtime正控等814-B，816最终UI切换等815-B。范围发布而非接口草图放行，完整目标未缩减。上图为工作包最终硬依赖，近期细粒度阶段DAG以[共用合同](staged-delivery-review-814-816.md)为准。R07 Man/UI profile分别放行，无关工作不等全生态归档。

R10/R12可以与发行/搬仓准备交错；M1允许登记的兼容AST，M2才要求其退出。R11按实际消费者及相关R10能力逐入口退出，不把所有旧入口一次性砍掉。

## 6. 可并行范围与合入纪律

| 阶段 | 可并行的完整工作线 | 串行交接／不可越过的关口 |
|---|---|---|
| 现在 | 809–811已交付；814-A纯SDK/编译、815-A协议/真实窄端点、816独立执行设施可并行实施与审查修复 | 816真实客户端等815-A发布；SDK/协议不得消费WIP，资产未真实接线不搬走原生产源 |
| A发布后 | 814-B runtime与兼容收口、815生成主体、816执行/管理adapter及各scope审查修复可重叠 | 815生成主体等814-A+815-A；816管理adapter等814-A+816-A，先前已发布窄能力保持 |
| 最终完整集成 | 上游B合流同时，无关scope审查/修复继续 | 815完整runtime正控等814-B；816完整UI路由/默认profile切换等814-B+815-B；最终原AC全部通过 |
| 分层与工具期 | R04与Man核心/协议定义/非UI adapter可交错，R07清单/消费driver持续更新 | R04取得文法/宿主及I-PROJECT交接后搬相应实现/Man中UI生成器；R06全部生产验收须实际R04/R05接口，不能只对协议fake判完成 |
| 搬仓期 | R08与R09清单、legacy刷新、隔离准备树、非UI验收可并行；R10/R12不重叠部分继续 | 同一公共类型/source/manifest/资源迁移及消费方切换一窗口一owner；R09最终组合验收需已交付UI版本 |

共有文件（根/成员Cargo.toml、锁、lib导出、宿主安装、777规则/基线、Spec索引）按精确片段接线，后合入者同步master取并集并重放受影响验证。不能整份覆盖，不宽化forbidden规则，不因不同worktree而宣称没有语义冲突。

**无全工程停工要求。** AC/ABI/AutoDown及业务新功能按实际写面协调；新UI语法/native/pac协议需先交给相应owner。迁移窗口内受影响WIP先形成可恢复交接，不能stash/reset丢代码。跨仓一Plan一仓一树、组内解析、不建junction/symlink，清理必须wt-guard clean。

## 搬仓前的共同硬门

每条门由对应工作包提供受版本约束的实际证据；“文件存在”“代码能编译一次”“计划已勾选”均不足以单独放行。R08/R09的实施Plan须将下表变为稳定AC和可复跑入口。

| Gate | 验收条件／必须观察的结果 | 主责与放行范围 |
|---|---|---|
| **G1 功能与支持面** | 冻结命令/语法/backend/平台矩阵；正常输入、Core对照、错误/缺能力/取消和真实消费与旧臂或独立oracle一致。既有失败具名复现，不新增静默降级，不靠重写golden掩盖漂移 | R01/R03/R04/R05/R06；每个交付profile明确支持面，缺环境标pending，不将未测写成pass或默默取消原支持 |
| **G2 依赖方向与类型身份** | 真实normal/build及必要dev/feature闭包无环；纯语言消费者无UI/Man checkout仍可解析构建/运行，Man核心不拉完整VM/renderer；UI只经公开SDK/轻工具协议；AST/Value/协议/FFI只来自单一source身份 | R04/R05/R06；组合/legacy facade与基础SDK分别测，不能在大workspace可建就宣称基础包独立 |
| **G3 资源及构建闭包** | 源码、build.rs/include_str、模板/schema/stdlib、patch、字体/图像/脚本、工具二进制和producer/cache身份全有归属/版本；manifest/lock可重放。fresh解析、真实build/test、locked重放和A→B重定位成功，不借原target/rlib/临时树兜底 | R07持续＋R08/R09重放；缺资源/坏hash/漏patch/同名双source/不兼容版本确实非零失败，恢复后复绿 |
| **G4 宿主生命周期与退出** | 实例选择/默认优先/clone/merge、字符串/句柄/队列/线程亲和/模态/重入保持；term/media/拖出与watch/daemon在成功、失败、取消、超时后实际子进程/窗口/reader/临时资源退出或给准确恢复结果 | R03/R04/R06；真实OS/renderer/进程证据不能用fake计数或提前返回代替 |
| **G5 旧仓与来源可恢复** | 按当下HEAD/完整refs/WIP/ignored载荷刷新状态寻址保全，独立恢复验证；源commit/path→目标commit/path及资产/许可证映射闭合；新提交进入旧仓迁移分支，原tag/legacy不移动 | R08/R09，复用784工具/准备树/恢复载荷。没有先清空默认分支、orphan或强推覆盖历史；状态已变则新快照，不覆盖旧证据 |
| **G6 真正独立仓及CI** | 从目标仓/固定依赖的新检出和独立外部工程完成支持profile的构建/测试/生成/运行；每仓所需锁/CI/工具安装说明完整，source不存在D:/autostack或工作树隐式依赖。准备独立CI不等于实际已运行，执行结果须注明环境/提交 | R08 UI交付、R09 Man与三仓组合；本地受管bundle、真实仓检出、第三方consumer是三个不同证明对象 |
| **G7 切换事务与失败保护** | 明确受管产物/目录和写入owner；依赖/资源校验、生成、发布等失败/中断时旧可用产物与用户文件保持，或有预先规定且实际演练的恢复步骤；版本/缓存不兼容在执行前拒绝，不能误报成功 | R07/R08/R09。792已有admission/staging机制优先复用；不为搬仓另写一套弱事务。业务copy的旧先截断失败债另行治理，不豁免发行/搬仓保护 |
| **G8 审查／合入／交接** | 相关实现独立final review通过，代码/依赖/配置/输入绑定冻结SHA；merge-owned门通过，Spec/ledger/来源与剩余债沉淀；迁出实现真实删除，消费者已切源，树/分支清理有guard收据 | 各Plan和R08/R09。reviewed不等于已交付；旧默认分支只在本门与前七门齐备后以新提交推进，远端push/默认分支更名仍需具体授权 |

每个profile至少包括其实际适用的Core SDK消费、Vue生成/构建、VM/Iced交互、Rust/Iced生成外部构建以及Man/CLI主要路径；Jet/Ark、其他平台和工具链等按**现行支持合同**登记验收或明确环境缺口。792选定Iced正控不能外推为所有renderer/平台已通过。若支持范围需要变化，在具体Plan中取得明确决定，不由路线图擅自缩减。

所有跨界增量都随所属模块验收闭合，不把必需验证延期给未知“最后一次总测”。新crate/路径/资源影响792闭包时，产生变更的Plan立即沿其现有接口更新/定向验证；R07/R08/R09再验新的完整支持面。

跨Plan代码审查、定向验证与修复可并行；同Plan同树单写者、冻结提交交接。共享SDK/协议修复由owner发布，影响方重放受影响scope。master/公共manifest/777/792/Spec ledger及裸t/触面整档走串行队列，GUI/OS全局资源与重型测试预约独占。反例回放集、证据失效和阶段非归档流程以[共用合同§2–5](staged-delivery-review-814-816.md)为准。

## 8. 测试、证据与完成口径

work先完成全部实现/修复及相关定向反例、模块测试、必要check；同树独立final review验AC/SD，不运行整档来代替缺失证据。最终review pass后，merge在新冻结提交上统一跑裸cargo t及实际触面的tv/tu/tt等；变更影响证据须复审/重放，复用须证明源码/依赖/配置/输入未变。tf由主检出到期批量单实例执行，不因多模块并行而各树重复开重档。

每个工作包收据包含：来源/目标commit与路径、真实依赖/feature、支持profile、正反oracle、原始结果、旧实现删减、剩余边/兼容及失败回退。新文法不能复制Parser算法，新模型不能复制类型，新runner优先复用已有设施。已登记的模板/工具链加载预存缺陷按合同保留或另立明确行为修复，不借搬迁随手改语义。

规模指标跨三仓统一比较：生产/测试/生成/第三方来源分开披露；迁出行数、净删除量、重复实现清偿量分列；同一份实现搬到别仓不算全生态净删。依赖禁向边/包闭包、最大实现模块、cold/warm/增量构建在相同环境/输入下测量；没有实测不承诺速度/内存提升百分比。

**M1完成判据**：R00和R01–R09所需交付已过G1–G8，三仓支持profile独立与组合消费成立，允许的兼容项逐条列明。**M2完成判据**：R10–R12退出条件及受影响消费方已验收，兼容/净瘦身清单无未授权遗留；不能只以仓库个数或计划归档数宣布完成。

## 9. 下一步与滚动规划

1. 809–811已实际交付；本轮不重做其review或修改历史交付。
2. 按[814](../814-language-sdk-ui-package-separation.md)、[815](../815-ui-project-tools-and-assets-extraction.md)、[816](../816-tool-execution-protocol-and-thin-cli.md) r2阶段门实施；现可并行814-A、815-A及816执行设施，815-A发布后816实际客户端，不等815完整工具完成。主体与审查修复的并行/最终集成门见[本批调度](sdk-tools-parallel-order-814-816.md)。
3. 每次只为下一批前置/owner/oracle明确的2–3个模块取Plan号。已有在途计划继续收口，新规则不追溯拆改它们；未经用户指定不自动创建agent或新chat。
4. 在R04/R06形成实际SDK/协议后重新核数量和风险，再建立R08/R09跨仓交付合同；旧仓refs/默认分支/远端操作在具体合同中明确，本路线不执行这些操作。

## 10. 来源、历史与本轮验证

当前合同来源：[前端Spec](../../specs/auto-lang/frontend/overview.md)、[AST](../../specs/auto-ast/project.md)、[解析端口](../../specs/auto-parser-api/project.md)、[VM宿主](../../specs/auto-lang/vm/overview.md)、[UI](../../specs/auto-lang/ui/overview.md)、[Man](../../specs/auto-man/project.md)、[工具链核心](../../specs/auto-toolchain-core/project.md)、[旧仓治理](../../specs/repository-boundaries/project.md)、[发行地基](../../specs/repository-boundaries/generated-ui-distribution.md)。这些Spec陈述已交付范围，本路线只提出未来目标。

历史批次：[789–792](../attachments/extraction-batch-20261016/inputs.json)、[799–800与792/794合流](model-decoupling-parallel-order-20261016.md)、[801–803](extraction-next-parallel-order-20261017.md)、[804–806](extraction-followup-parallel-order-20261017.md)、[807–809](module-migration-batch-807-809.md)、[810–811](grammar-media-parallel-order-810-811.md)；最新为[814–816](sdk-tools-parallel-order-814-816.md)。历史状态不覆盖当前快照，不反向修订已归档合同；旧A–J映射为A/B→R01/R04/R10，C→R02/R03，D→R04，E→R05/R06，F→持续发行＋R07，G/H→R08/R09，I/J→R10–R12。

总路线v2仅整理路线，见[原预检](../attachments/extraction-roadmap-20261018/preflight.md)；v3新增807–809，见[其预检](../attachments/module-batch-20261018/preflight.md)；v4新增810/811，见[其预检](../attachments/grammar-media-batch-20261019/preflight.md)。v5建立814–816的r1，见[旧预检](../attachments/sdk-tools-batch-20261019/preflight.md)；本轮v6按用户授权修订r2，开放真实A/B阶段发布与并行审查修复，不增加Plan号、不写编号计数器。只改规划文档，不改Rust/测试/canonical规范/ledger，不创建agent/树、不实施搬仓。Category A无Cargo/docs_gen；[r2预检](../attachments/sdk-tools-r2-20261020/preflight.md)仅验证合同，未来实现验收未执行。
