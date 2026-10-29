# AutoGraphics路线跟踪交接（2026-10-28）

> **2026-10-29最新拆分**：用户批准[auto-ui/002](../../../../auto-ui/docs/plans/002-autographics-trigger-events-and-round-restart.md)仅引擎，G11完整移交[ag-examples/001](../../../../ag-examples/docs/plans/001-autographics-gallery-runtime-objects.md)，硬等002实际交付。两份仅修合同、暂不实施；旧883混合分工为历史。

> **计划管理归属更正（2026-10-29）**：AG引擎计划在auto-ui/docs/plans，Gallery独立内容计划在ag-examples/docs/plans；跨仓计划由主责任仓持一份合同。882/883已迁至auto-ui（drafting r2），880最新收据已交付归档并迁至ag-examples。auto-lang原位置只留定位指引，历史Specs/设计本次未批量迁移；下文早期“中央Plan仍在语言仓”表述已由本条取代。

> **2026-10-29最新续接**：879已切源交付，AG源码唯一权威现为auto-ui/experimental（4adf2d03）；auto-lang只留计划/规范知识及旧分支历史指引。880用户已启动开发，固定消费旧875，不接管。下一批[882对象生命周期](../../../../auto-ui/docs/plans/001-autographics-runtime-object-lifecycle.md)→[883触发/新局＋G11](../../../../auto-ui/docs/plans/002-autographics-trigger-events-and-round-restart.md)已规划、尚未实施；883引擎等882、G11接线还等880。下文旧快照及前次续接是历史。

> **2026-10-29续接更正**：下文为10-28历史快照，现场状态已经过时。874/875/877/878均已交付归档，当前AG交付为875的`1cc68aba012f76951af2556763a5fb21be6a6c8b`；基础Gallery为`ag-examples/master 6a223d9`。下一轮已规划[879纯迁仓](../../plans/archive/879-autographics-source-move-to-auto-ui.md)与[880 GY-B](../../../../ag-examples/docs/plans/legacy/auto-lang/archive/880-autographics-gallery-character-and-cameras.md)，均drafting、尚未实施，详见[最新战略§7](../strategy/autographics-m2-v06.md#7-当前接棒2026-10-29)。旧SDK来源/M1标签映射风险仍保留。

本页保存长会话的决策与交接时观察，**是带日期的快照，不是新的实施合同或状态权威**。继续跟踪时先读实际Plan、其最新记录、Git/worktree与当前owner交付收据。设计/战略描述目标，canonical Specs描述已交付，实际代码/证据确认实现；勾选/status不能单独证明通过。

## 1. 入口与工作方式

- 主检出`D:/autostack/auto-lang`，Windows/PowerShell；AG实现仍在`autographics-dev`的`experimental/autographics-prototype`与`experimental/autographics-auto-driver`两独立workspace，默认master主要保存Plan/设计/知识。
- 优先读[M2战略](../strategy/autographics-m2-v06.md)、[AG Specs索引](../../specs/auto-graphics/overview.md)，然后仅读相关Plan/代码，不重新加载829以来全部历史。
- 遵守AGENTS及auto-plan new/work/review/merge。只跟踪/规划的会话不自动实施/合入/发布/迁仓或新建agent。读取其他会话授权时以真实最新记录为准，本快照不暂停或撤销其他会话已获授权的工作。
- 一Plan一权威实现树，每仓一树；独立review是独立上下文，同树顺序进行。不得重复建树/接管在途修改，不放junction/symlink；清理先wt-guard。AG计划建树从明确AG提交，默认master建组脚本不可直接套用。
- 本机已有大量其他路线WIP及未跟踪设计/Plan；禁止宽泛add/commit、覆盖或清理。Plan ID全仓统一，使用中央脚本/独占分配，不能硬编码“下个879”；交接观察.next-id为879，后续必须重核。

## 2. 已交付基线及身份注意事项

M1是有限独立交互原型，标签`ag-v0.6-m1`；包含共享wgpu/Iced组合、真实Auto专用游戏、scene创作/便携源码资产、有限公共behavior/query/motion，非完整通用游戏引擎。审计见[851阶段报告](../../reports/autographics-milestone-851-2026-10-26.md)。

S0：853性能/资源/证据基线、854行为源装配、855Host职责收敛已交付。S1：856/857两维编辑网格/导航/拾取已交付。858 **Phase1**是两款游戏样本/资源/能力缺口研究交付，不把后来其他路线附加阶段混为AG功能。859为被858后续阶段替代的归档草案，未交付新AG能力，编号不复用。

860真实Auto G2/G3角色/路线/128负载可行性已交付，decision=go；861已交付公共`autographics.character/1`与真实OS四动作/单2D角色/静态solid。它们都不意味着公共3D角色、运行相机、对象生命周期、纹理/动画/声音/HUD或完整M2已经完成。

当前已交付AG观察基线为`8b12632d7b5b1b0df72fbb4eef84297a054de31d`，也是877/878的固定消费/审计输入；即使874继续开发也不追tip。SDK逻辑pin为`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`；历史对象不可达时须验证681文件/17包SDK内容锁，可定位保留副本`D:/autostack/plan-artifacts/plan-860/consumer-package/experimental/autographics-auto-driver/.sdk-source`，不可用当前SDK替换。

### M1标签身份差异（待核实，不自行修标签）

交接读回发现：历史文档/标签注释的Frozen source是`11911a4a6d809bfd515f745b77d00bc8f04f6ba9`，本机该commit对象当前不可达；实际`git rev-parse 'ag-v0.6-m1^{}'`为`679681b3db9f1b82dbe9ad6a6733245bc3c82d36`，annotated tag对象为`de42b9f6e9c066e272dbe3642f1bfa0859d8d362`。标签注释仍写原11911a4a。

此前861已有历史重写/等内容收据背景，但本交接**未证明上述两个M1来源内容等价，也未确认变化原因**。保留两种身份，后续按851冻结源/identity/交付收据核对映射；不能宣称标签未变、强制重打/移动标签、改写历史SHA或由提交标题推断等价。此为来源核对项，不据此重跑整套里程碑重型门或暂停独立固定pin开发。

## 3. 最近四份未交付计划与现场状态

以下为交接时只读观察，不是完成证明；现场正在变化，新会话先重核。

| Plan | 合同/范围 | 观察到的状态/权威树 | 接棒边界 |
|---|---|---|---|
| [874](../../plans/archive/874-autographics-public-character3d.md) r1 | 公共3D角色、六动作、归一、最终支撑、真实Auto/OS/GPU/预算；不含运行相机/模型动画 | Plan仍drafting 0/9，但`D:/autostack/.wt/lang-874/auto-lang`、`plan-874-dev`已存在，HEAD为8b12632d7，driver/API/样例/tests有未提交WIP | **在途实施存在、簿记尚未同步**。不重复建树、不自行改status/勾任务或接管；先核实施owner与最新记录 |
| [875](../../plans/archive/875-autographics-runtime-cameras.md) r1 | 2D跟随夹界/letterbox、3D orbit/zoom及同tick相机相对移动/发布 | drafting 0/9；本轮未观察到交付 | 硬等874实际交付再冻结实施base；不把874工作完成或固定heading数学测试当相机完成 |
| [877 GY-A](../../plans/archive/877-autographics-gallery-catalog-and-basic-six.md) r1 | Gallery目录/G01..06六项十子入口、公共CLI、资源/单项离线包 | executing 0/10；`D:/autostack/ag-examples`已建本地锚点master `4df37bd2b973b98ee6e21ed03885f9067894e6d5`，唯一树`D:/autostack/.wt/lang-877/ag-examples`、`codex/plan-877-dev`存在；检查时内容树clean，仅.git | 已进入执行/准备，非“仓还没创建”；任务尚未勾选、暂无交付收据。normal入口消费固定861，不改引擎，禁止另建第二仓/树 |
| [878 XP-0](../../plans/archive/878-autographics-platform-boundary-and-dependency-audit.md) r1 | 固定AG/旧SDK静态依赖与平台边界、XP-1建议、纯归仓评估 | drafting 0/9；观察时`D:/autostack/.wt/lang-878/auto-lang`不存在 | 可独立规划/后续按授权启动，不等874/875/877；仅写自己的报告，不编译WASM/运行探针/适配/迁仓 |

本跟踪会话未修改四份Plan文件或它们的实现树。874/877与上一轮“暂不实施”的快照已有现场变化；不得用旧授权摘要否定其他会话的后续指令，也不得凭树存在假定所有任务已完成。

依赖：`861已交付→874→875`；877与878均固定读861，可以与引擎线及彼此并行。877写ag-examples内容，878写自己的语言仓报告；AG/SDK/auto-ui只读，target/报告/SDK准备目录独立。真实GPU/窗口焦点/重型门按机器单实例，不能因分支不同争抢设备。

## 4. M2目标、示例与滚动规划

M2为Windows x64桌面研究交付：可用网格编辑视口＋公共服务驱动的自有小型2D横版平台关卡与3D第三人称平台探索/收集关卡＋独立工程/资源发行＋Gallery。游戏需角色/碰撞、收集/敌人或危险、胜负/重开、相机、必要动画/声音/HUD，全部真实Auto；测试Rust oracle不进入正常玩法，不用专用游戏Host凑公共能力。

[M2设计](m2-editor-and-public-game-prototypes.md)M2-01..09与[冻结G2/G3研究](m2-game-sample-matrix.md)共同约束。3D所选模型有节点动画/纹理且无skin，当前静态loader拒绝相关资源；必须有后续支持或受检等价派生，不能默认已可用。完整骨骼/PBR/后处理/大规模compute不强加给M2。

仍待公共3D角色/相机、动态对象身份/生命周期/触发、2D精灵、3D纹理/节点动画/视觉朝向、声音/HUD、两游戏和分发总验收。原引擎/游戏估计9–12份＋Gallery净增2–3份=约11–15份，含874/875/877，878及完整跨端/纯迁仓另计；是滚动工作量估计，不是实际进度百分比/固定月份。只给近期切片取号，远期保留候选。

月度发行与版本规则已见[总Roadmap](../../roadmap.md)：0.x功能版可v0.9.0→v0.10.0→v0.11.0，修复用patch；AG M1/M2研究标签不等于产品版本或绕过质量门。

## 5. Gallery、跨端和迁仓的已讨论边界

**Gallery**：[专题](m2-gallery-and-capability-matrix.md)17项能力轴；参考three.js官方r186样例/依赖，不承诺全功能复刻。首12项：W0形状/静态GLB/层级/编辑教程/两维查询/2D角色（877六项十入口）；W1三维角色/两维相机（后继GY-B消费874/875）；W2精灵、纹理→节点动画/朝向、对象触发；W3音效→事件HUD。内容与引擎补齐难度分别记录，资源准备可早做，共同协议/Registry/Play接线先冻结再串行。

ag-examples独立小示例仓；两款完整游戏各自独立仓，Gallery仅索引。小必需资源随Git，大资源/制作源/长录屏按固定hash/许可包选下载；单项包实体化完整本地闭包、可离线搬迁，不依赖绝对路径/cache/链接。网页初版只做预览/下载，不标浏览器引擎可运行。

**跨端**：[专题](platform-architecture-and-research.md)共享语义/运行/资源与renderer/宿主分层；优先研究Rust→WASM/wgpu→WebGPU＋Vue/DOM，移动原生和移动Web分别profile。DOM内Canvas、DOM overlay、可采样UI进入3D不同；任意DOM不能默认当纹理，overlay不天然参与深度，跨DOM/GPU不承诺物理原子呈现。异步资源/ID精度/输入/后台时钟/device代次/恢复与能力分级另验，renderer能跑不等于Auto/完整UI能跑。

XP-0=878静态审计；XP-1小场景双端绘制、XP-2 Vue/Canvas桥接、XP-3 UI进3D、XP-4移动两路线、XP-5决策收敛尚无正式编号。跨端不是当前桌面M2新增前置，不把Windows通过推为所有端Ready。

**迁仓**：[分离门](repository-move-and-integration-gates.md)把纯源码归属、生产UI/声明融合、ACE/增量/HMR、切默认/退役分开。auto-ui已独立可消费，纯源码迁入技术上不必等ACE；建议875实际交付冻结后另立迁仓Plan，先放auto-ui/experimental两个独立workspace/锁/旧SDK，默认生产UI不反向拉入AG。此为候选排期调整，**未批准/启动迁移**；878只评估触面，生产融合/ACE/HIR统一仍等实际成熟后再讨论。

长期UI组件完整兼容、默认设计体系/主题/跨端样式范围在Design47；统一Atom＋Schema HIR下Auto、AutoUI（UI/2D/3D）、AutoShell为不同方言，Grammar由Atom配置并走各自语义/lowering。没有“旧VM已具备HIR/ACE/HMR”的交付结论；不把当前两个控件探针当完整AutoUI。

## 6. 新会话建议的第一轮动作

先核四Plan最新frontmatter/§9、真实worktree/owner/HEAD/未提交改动、当前AG已交付tip与877锚点；不写代码或改Plan进度。报告可继续的主线、875前置和在途授权/记录需确认之处。后续按用户明确的跟踪/新计划/复审/实施指令执行，不自动给其他会话发消息或新建agent。

纯文档跟踪不跑Cargo/SDK prepare/native/性能；实现仅定向验证，最终独立review后merge负责冻结完整门；根重型档与批量回归按AGENTS作用域/单实例，不为转会话重跑全量。证据非md本地保留，入库只摘要/契约；当前大量设计/Plan尚未提交，不能当作缺失或擅自批量提交。

## 7. 可复制的新会话开场Prompt

请继续跟踪和滚动规划AutoGraphics（简称AG）路线，工作仓库为D:/autostack/auto-lang。这个开场消息只建立跟踪上下文：先做只读核验与状态报告，不自动接管在途实现、启动新计划、合入/发布/迁仓或创建agent。

先读：
1. docs/design/autographics/tracking-handoff-2026-10-28.md（完整交接快照与来源注意事项）。
2. docs/design/strategy/autographics-m2-v06.md（当前主线与并行路线）。
3. docs/specs/auto-graphics/overview.md及与当前任务相关的Specs；按需读M2、Gallery、平台架构和归仓专题，不必重读所有历史计划。
4. 当前Plan874、875、877、878的完整合同、最新§9、实际工作树/HEAD/WIP与交付收据。快照可能过时，以新证据更新，不从status或勾选单独推断完成。

最近四份未交付计划：
- 874：公共3D角色/六动作/三维接触与最终支撑。交接时Plan仍drafting 0/9，但D:/autostack/.wt/lang-874/auto-lang（plan-874-dev）已有未提交实现；不能重复建树或擅自接管，先核owner/最新记录。
- 875：公共2D跟随夹界与3D环绕/缩放相机、同tick相机相对运动/共同发布。drafting 0/9，等874实际交付后冻结基线；保持原范围，不追加Gallery/跨端。
- 877（GY-A）：Gallery目录和G01..06六项十个必需子入口、公开启动/资源验证/单项离线包。交接时executing 0/10；D:/autostack/ag-examples已建本地锚点，唯一树D:/autostack/.wt/lang-877/ag-examples、分支codex/plan-877-dev已存在。不要重新初始化仓库，仍须核任务/验收是否真正完成。
- 878（XP-0）：固定AG/旧SDK的静态边界与依赖审计、XP-1建议、纯归仓准备评估。drafting 0/9，交接时尚无实现树；不编译WASM、不运行跨端探针、不适配或迁仓。

依赖为861已交付→874→875；877和878可与引擎线/彼此并行，固定读取861交付的8b12632d7b5b1b0df72fbb4eef84297a054de31d，不追开发tip。877只写内容仓，878只写自己的审计报告，共享引擎/SDK/auto-ui只读；GPU/窗口和重型验收按机器单实例。旧SDK逻辑pin与681文件内容锁、历史对象不可达的保留副本处理见交接文档，不能随意切当前SDK。

保留的决策：
- M1有限原型已交付；853–855优化、856/857编辑视口、858 Phase1研究、860 go、861公共2D角色均已交付，M2仍未完成。M1标签历史源11911a4a与当前peel679681b3存在未核映射差异，按原收据核内容，不自行重打标签/改历史SHA。
- M2目标是公共接口驱动的2D平台跳跃＋3D第三人称平台探索收集两款独立小关卡，以及编辑/资源发行/Gallery完整闭环；真实Auto玩法，不用专用Rust业务或截图替代。不静默提高原预算、删验收分母。
- Gallery参考three.js组织能力矩阵，首12项分波：基础6项→角色/相机→精灵/纹理/节点动画/触发→声音/HUD。小示例在ag-examples，两款游戏各自独立仓；小资源随Git，大资源按固定版本可选包，单项完整包可离线搬迁。
- 桌面为当前M2交付目标；浏览器WebGPU（Vue/DOM＋Canvas）、移动原生/移动Web分开研究。DOM overlay不等于可采样/深度正确UI，渲染可运行不等于Auto执行/完整UI可用；跨端不是当前M2新增前置。
- 纯AG源码归入auto-ui与生产UI/共同声明树/ACE融合分开。候选时机为875实际交付后另立纯迁仓Plan，先保留独立experimental workspace/锁/旧SDK；尚未批准/启动迁移。完整UI组件/主题、Atom＋Schema HIR方言、ACE增量/HMR按成熟路线后议。
- 桌面M2剩余暂估11–15份，已含874/875/877；878/跨端产品化和纯迁仓另计，未来按证据滚动细化，不预取一串编号。

遵守AGENTS与auto-plan技能，一Plan每仓一权威树、独立review与merge冻结门，禁止worktree链接，清理先guard。仓库有其他路线WIP/未跟踪文档，不覆盖或批量提交；计划号重新核中央取号器。请第一轮报告四计划最新状态/owner证据、875前置和建议接棒优先级，然后按我后续具体指令继续跟踪或规划。
