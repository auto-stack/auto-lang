# AutoGraphics（AG）v0.6 M2战略步骤

> **本仓编号更正（2026-10-29）**：auto-ui计划从001起独立编号，当前001=旧auto-lang/882（运行对象）、002=旧883的引擎部分（触发/新局），下一个003；ag-examples/001独立承接G11，硬等auto-ui/002实际交付，下一号002。旧880为legacy历史。下文早期882/883只是旧号映射；当前唯一合同见auto-ui/docs/plans。各仓不能以外仓迁入编号作为本地高水位。

> **计划管理归属更正（2026-10-29）**：AG引擎计划在auto-ui/docs/plans，Gallery独立内容计划在ag-examples/docs/plans；跨仓计划由主责任仓持一份合同。882/883已迁至auto-ui（drafting r2），880最新收据已交付归档并迁至ag-examples。auto-lang原位置只留定位指引，历史Specs/设计本次未批量迁移；下文早期“中央Plan仍在语言仓”表述已由本条取代。

更新：2026-10-29。状态：战略基线v11（按用户批准拆分）；AG源码权威为auto-ui/experimental，879源4adf2d03；旧auto-lang/autographics-dev仅历史指引。旧880已交付8项14入口。当前[auto-ui/001对象](../../../../auto-ui/docs/plans/001-autographics-runtime-object-lifecycle.md)已有权威工作树，进度由其owner记录；[auto-ui/002引擎触发/新局](../../../../auto-ui/docs/plans/002-autographics-trigger-events-and-round-restart.md)为drafting r4，只做引擎；新[ag-examples/001 G11](../../../../ag-examples/docs/plans/001-autographics-gallery-runtime-objects.md)为drafting r1，硬等002实际交付后实施。本轮仅修计划/引用，不接管001实现或启动后两份。设计见[M2](../autographics/m2-editor-and-public-game-prototypes.md)、[运行](../autographics/m2-runtime-and-character-control.md)、[Gallery](../autographics/m2-gallery-and-capability-matrix.md)、[平台](../autographics/platform-architecture-and-research.md)。

## 1. 阶段定位

M1标签名`ag-v0.6-m1`，历史收据/注释记录源`11911a4a6d809bfd515f745b77d00bc8f04f6ba9`；交接时实际标签peel为`679681b3db9f1b82dbe9ad6a6733245bc3c82d36`，旧对象不可达，映射/内容等价尚未核实，见[快照§2](../autographics/tracking-handoff-2026-10-28.md#2-已交付基线及身份注意事项)。不重打/移动标签或改历史SHA。851及前序范围见[审计](../../reports/autographics-milestone-851-2026-10-26.md)；M2需新增编辑/公共服务和真实游戏工程，优化完成不等于M2。

目标是“能够创作公共接口驱动的小游戏”，不是以更多专用demo累计数量。2D横版平台跳跃、3D第三人称平台探索收集各一个；Windows x64单窗研究路线，879后在auto-ui独立experimental workspace继续旧AutoVM SDK。等ACE自身完成后再讨论AutoUI/HIR/ACE及生产归仓，M2不提前承担这些集成。

用户追加Demo Gallery：以three.js为图形覆盖参考，建设C01..17能力矩阵和首12个逻辑示例，支持源码阅读、独立原生运行与按需下载。目录/内容与引擎能力分别规划；完整游戏仍验证系统协作。图库不把全部three.js效果纳入M2，不提前承诺Web运行。

跨端架构目标已列入：桌面原生、Vue/DOM＋WebGPU Canvas、移动原生及移动Web分别适配宿主/输入/资源/生命周期，共享场景/运行语义与图形数据。XP候选调研可并行，不启动正式适配，不增加桌面M2-01..09前置；原11–15份估计不含XP或各端产品化；874/875/877落地后粗余8–12份，非重新测量的工期承诺。完整AutoUI/ACE融合仍等相应路线成熟后另议。

## 2. 滚动规划原则

- 只给近期已取证切片分配正式Plan；874/875/877/878已交付；879纯迁仓/880 GY-B已交付；下一批由auto-ui/001→002引擎与ag-examples/001内容分仓管理，其余保留工作包名，不占号/排月份。已交付合同不被新计划扩大，本轮不启动执行。
- 本设计确定阶段目标；每个功能Plan仍须从现行Specs和源码冻结输入、支持/排除、可行性、AC、具体例子与资源预算。
- 阶段可包含多个Plan，完成Plan或完成优化不会自动触发M2标签。
- 引擎共享运行合同保持单一序列874→875；允许独立仓的Gallery内容/资源准备形成并行工作包，固定消费已交付版本。跨线公共协议/Registry/Play/assembler改动先冻结接口与owner，串行接线；不得抢跑依赖或共改一棵实现树。
- 跨端先验证边界与最小路径，渲染、Auto执行、DOM/UI和完整交互分别认领。XP只读审计/独立研究宿主可与引擎/Gallery并行，公共接缝提取另立Plan串行；不追改874/875、不把研究成功当生产接线批准。

## 3. 路线与退出条件

| 阶段 | 工作与结果 | 退出条件 | 下一阶段依赖 |
|---|---|---|---|
| S0：M1基线与质量收敛 | 性能/资源与产物基线、行为源版本构成、作者宿主职责，当前能力表 | 853→854→855完成独立复审、冻结门与实际合入/知识沉淀；M1兼容和原分母保留 | M1已交付；首批正式计划见§4 |
| S1：可用的2D/3D编辑视口 | 网格/轴、平移/缩放、3D环绕、聚焦、平移吸附、正交/透视与一致拾取 | 设计M2-02真实OS/画面/状态证据；导航不污染游戏或文档历史 | S0收敛；实现前冻结相机/吸附/输入规则 |
| S2：样本冻结与公共运行能力 | 固定两款小关卡、来源/许可/资源盘点和差异表；角色、生命周期、动作/事件、相机、基础表现/声音/HUD | M2-03；有界探针通过，2D核心语义可复用到3D，旧profile不变 | S0；接口设计可与S1局部交叠，发布与拾取合同先稳定 |
| S3：2D平台游戏独立闭环 | 轻量独立工程，经公共接口完成跑跳/敌人/收集/失败/胜利/重开 | M2-04/06；编辑关卡并保存重开，修改Auto玩法无需专用宿主代码 | S1、所需S2服务；工程/资源合同见§5 |
| S4：3D平台探索独立闭环 | 可操作第三人称相机、立体跳跃/落地、收集/危险/终点/重开 | M2-05/06；与2D消费相同公共引擎基线，具有真正3D交互 | S2共同服务及2D角色语义；3D资源/相机调查可提前，不另复制运行栈 |
| S5：M2整体验收与发布 | 两仓外游戏、Gallery/首12项、固定版本/资源矩阵、单项/完整包、离线搬迁与质量总结 | M2-01..09全部有当前冻结证据，独立复审和门禁完成，再决定标签 | S0–S4、Gallery内容及资源分发交付 |

S2与S1不是“两个大架构同时重写”。共同公开边界与资源/状态所有权先明确，再由具体Plan划分互不冲突的工作。2D先稳定角色和服务；3D的模型/相机风险可独立调查，不要求等待2D全部美术完成。

## 4. 首批正式优化计划

| 顺序 | Plan | 主要成果 | 当前状态/进入门 |
|---|---|---|---|
| 1 | [853：M2基线与证据收敛](../../plans/archive/853-autographics-m2-baselines-and-evidence.md) | 可比性能/资源数据；不可变runner与证据身份；原门逐项映射及实际消费者 | archived/delivered9/9；图形提交c1961665d，实际证据见归档收据 |
| 2 | [854：行为源码生成与版本兼容](../../plans/archive/854-autographics-behavior-source-assembly.md) | 显式版本构成与具名拒绝；v1..v5生成字节和运行行为等价 | archived/delivered9/9；图形提交523d27b86 |
| 3 | [855：作者宿主职责拆分](../../plans/archive/855-autographics-authoring-responsibility-split.md) | 编辑/运行/视图/CLI职责梳理，保留单一owner与状态机；当前入口表 | archived/delivered9/9；图形提交05fdc81c5；S0完成 |

这三份覆盖阶段审计OPT-01..04；性能测量在源码/模块整理之前建立，不把行数、yield数或逻辑字节当性能收益。它们不增加Grid、跳跃、动态节点、贴图或正式公共SDK。

### 4.1 第二批：编辑视口与样本调查

| Plan | 范围 | 前置/接棒 |
|---|---|---|
| [856：2D视口网格与导航](../../plans/archive/856-autographics-2d-viewport-grid-and-navigation.md) | 统一world/child变换、Grid/轴、pan/锚点zoom/focus、世界吸附、真实Auto编辑与输入隔离 | archived10/10；图形4150a8bea，已交付 |
| [857：3D视口导航与网格](../../plans/archive/857-autographics-3d-viewport-navigation-and-grid.md) | orbit/pan/zoom/focus、两投影、深度grid、实际相机拾取/透视轴拖动/吸附 | archived/delivered10/10；S1已交付 |
| [858：双游戏样本与接口缺口](../../plans/archive/858-autographics-m2-game-samples-and-api-gaps.md) | 固定来源/两款小关卡分母、许可/真实资源大小/闭包、公共/专用/缺失能力矩阵和后继建议 | Phase1研究交付；此处不替其后增补的其他路线工作下结论；正式资源/游戏未交付 |

856/857共同保持面、资源与真实OS/像素/深度正反验收见[视口合同](../../plans/attachments/856/planning/viewport-contract.md)。它们只完成M2/S1；858只给S2提供研究输入，三者均不认领完整M2。

### 4.2 第三批：运行可行性与首个公共角色切片

| Plan | 主要成果 | 前置/接棒 |
|---|---|---|
| [860：真实Auto角色/运行预算可行性](../../plans/archive/860-autographics-game-runtime-feasibility.md) | 固定G2/G3/128负载、真实Auto/资源/CPU和Host，decision=go | archived/delivered；probe不是公共API |
| [861：公共动作与2D角色控制](../../plans/archive/861-autographics-public-actions-and-character-control.md) | character/1、真实OS动作、单2D角色/静态solid、Auto跑跳及错误/Stop/consumer | archived/delivered9/9；当前图形8b12632d7 |

第三批共同保持面见[运行合同](../../plans/attachments/860/planning/runtime-contract.md)。未改变SDK/旧profile/原预算；第三批没有正式游戏/资产制作/运行相机/音频交付，不把角色样板算M2完成。新计划如需改变资源/语义，先据证据修订，不静默抬限。

### 4.3 第四批：公共3D角色与运行相机

| Plan | 主要结果 | 前置/进入门 |
|---|---|---|
| [874：公共3D角色与水平动作](../../plans/archive/874-autographics-public-character3d.md) | 独立新profile、六动作、三维地墙顶/最终支撑、水平归一、真实Auto/OS/三维内容与预算 | archived/delivered；交付c02a9c222，已由875承接；宽/稀疏128、密集深度64包络，旧树已清 |
| [875：公共2D/3D运行相机](../../plans/archive/875-autographics-runtime-cameras.md) | 2D跟随夹界/等比视区、3D跟随/环绕/缩放、同tick相机相对运动与共同发布 | archived/delivered；交付1cc68aba0，原SDK/两锁保持，旧树已清 |

引擎能力按874→875顺序实施，二者不新增Gallery任务/AC。874固定heading是接缝，875才交互运行相机；模型朝向/动画在后继表现计划，不能先认领完整G3-01。两者遵守旧SDK/32credits等原硬界，全路径不足即回new，不抬限；基线见[规划核对](../../plans/attachments/874/planning/baseline.md)。独立Gallery可按§5.1并行消费已交付产物，不与本序列争抢写面。

**原估计11–15份已含874/875/877；三份已交付后粗余约8–12份，含880 GY-B。879纯迁仓、878/XP跨端另计；未逐项重新估工，不作进度百分比。** 见[工作包估算](../autographics/m2-runtime-and-character-control.md#4-当前m2剩余工作估计)。Gallery示例分批消费原能力，不按12个示例各建一个引擎Plan；并行不会消除目录/验证/分发成本。不同拆分/可行性会改变数量，不预占号/承诺月份，不能删分母兑现估计。

### 4.4 第五批：并行内容与边界审计

| Plan | 范围与固定输入 | 进入门/写面 |
|---|---|---|
| [877：GY-A目录与基础六项](../../plans/archive/877-autographics-gallery-catalog-and-basic-six.md) | G01..06六项十子入口，目录/校验/公共CLI/六单项包；固定861 AG 8b12632d7＋原SDK | archived/delivered10/10；ag-examples master6a223d9，六项十入口；固定861，内容树已清 |
| [878：XP-0边界与依赖审计](../../plans/archive/878-autographics-platform-boundary-and-dependency-audit.md) | 固定AG/17包SDK、只读auto-ui manifest/交付事实；静态依赖/跨端接缝/XP-1建议及纯归仓评估 | archived/delivered；audit_complete=complete、readiness=limited，固定861/旧SDK；未实现跨端/迁仓，审计树已清 |

两者不改共享AG/SDK/auto-ui/874/875，目录/target/报告独立，GPU/窗口验收单实例。877落实原GY-A估计，不新增M2计数；878为原估计之外XP研究。初始规划来源见[基线](../../plans/attachments/877/planning/baseline.md)，当前状态按最新Plan/owner重核；交接发现的在途变化见[快照](../autographics/tracking-handoff-2026-10-28.md)。本跟踪会话不替其他会话启动/暂停/接管执行。

引擎Plan使用唯一`D:/autostack/.wt/lang-<id>/auto-lang`、指定图形前序提交与plan分支，源码只合autographics-dev；默认master建组脚本不能直接套用。877用自己的ag-examples树，878用语言master基面报告树、AG固定对象只读。多仓每仓一树，同树顺序实施/复审；主检出仅簿记，无链接。本轮不建树/agent或实施。

## 5. 工程与资源分发步骤

正式样本独立于auto-lang和未来auto-graphics，各自一个仓库；建议名`ag-platformer-2d`/`ag-platformer-3d`，实际远端与所有者由后继建仓合同确认。引擎仓保留小型测试/最小示例和索引，不把M1资产搬迁混入优化。

1. **样本盘点先行**：冻结参考提交、功能子集、资源SHA/许可，实测运行资源/制作源文件大小；据此定默认下载预算与大包划分。
2. **建立外部消费者**：以固定公共引擎版本运行最小工程，证明无需生产者路径、私有Rust Host或全仓示例；正式游戏代码在各自仓演进。
3. **资源按需交付**：小型必要资源随Git，大资源包固定版本发布；manifest记录身份、长度、SHA、目标路径、来源/许可。引擎下载不捆绑两个游戏。
4. **完整包闭环**：每款发布源码＋运行资源ZIP，下载后可离线、搬迁、编辑、保存和启动；损坏/缺失先拒绝，补齐后验证实际消费。
5. **绑定验收矩阵**：记录引擎/SDK/两游戏/资源/平台版本及实际运行报告。任何资源变更使相关画面、碰撞或性能证据失效，重新验证受影响项。

当前仅规划，尚未创建远端仓库/发布资产；不提前实现下载CLI、资产商店、LFS或跨平台安装器。发布需按用户已授权范围与当时具体合同执行。

### 5.1 Gallery执行顺序和并行安排

首批分析见[Gallery专题§4](../autographics/m2-gallery-and-capability-matrix.md#4-首批12项顺序与复杂度)，并行写面见[§6](../autographics/m2-gallery-and-capability-matrix.md#6-并行路线与后继工作包)。GY-A已落实877，其余仍为候选；本轮不实施：

| 顺序/工作包 | 内容与进入条件 | 与既有路线并行方式 |
|---|---|---|
| 先[GY-A/877](../../plans/archive/877-autographics-gallery-catalog-and-basic-six.md) | G01..06六项十子入口；固定861引擎、目录/单项包，公开运行和编辑验收 | ag-examples独立树，不改引擎；可与874/875/878并行 |
| 同期GY-R：资源准备 | 自制测试图、模型/纹理/clip/音效许可与闭包/大小分析 | 可以在正式精灵/动画/声音能力前准备；准备不算运行交付 |
| 再GY-B：W1角色/相机 | 874/875均已交付；880独立内容Plan固定消费875，不增原计划范围 | G07可与875开发并行；G08随后接入，不等两款游戏 |
| W2与能力Plan同行 | G09精灵；G10a纹理→b节点动画/朝向；G11身份/触发用平色资源先做 | 2D/3D表现与事件内容可按写面拆开；共同资源/Play协议接线串行 |
| W3与声音/HUD同行 | G12a动作一次音效先做，b消费G11事件的HUD后做 | 资源准备早做，a不等触发；b等事件合同，避免专用Rust路径 |
| 最后GY-C收敛 | 十二项全部子入口、公共/编辑/研究分类、单项ZIP/离线搬迁及统一最终引擎矩阵 | 可与游戏内容收尾并行整理；最终原生GPU/共享重型测试串行，优先并入原分发验收Plan |

独立`ag-examples`保存小示例/元数据/许可/小资源；两款游戏各自独立仓，Gallery只链接。大资源/录屏/制作源固定包按需，单项运行包实体化本地完整闭包；网页初版提供预览/下载，原生运行与Web能力分开标识。详情见[专题§5](../autographics/m2-gallery-and-capability-matrix.md#5-仓库目录和资源组织)。

GY-A约1份、GY-B约1份，GY-C并入原分发或另拆1份，净增2–3份；G09..12仓外示例在尚未冻结的未来支持Plan明确认领。若适配超出轻内容范围再显式拆分。M2不额外纳入PBR/后处理/compute/骨骼等效果；能力矩阵保留这些后续候选。

### 5.2 跨端调研旁路（仅规划）

步骤/反例与支持层次见[跨端专题§3/6](../autographics/platform-architecture-and-research.md)。XP-0/878已交付complete/limited，XP-1..5仍无正式编号；本轮不启动探针或适配：

| 顺序 | 调研与退出结果 | 并行条件/硬前置 |
|---|---|---|
| [XP-0/878](../../plans/archive/878-autographics-platform-boundary-and-dependency-audit.md) | 固定AG/SDK、用途/target/feature依赖分类和平台接缝；另评估auto-ui纯迁仓触面 | 可与874/875/877只读并行；不编译WASM/改源码/迁仓 |
| XP-1 同场景双端绘制 | G01/G02最小场景分别native与浏览器WebGPU；真实内容/resize/DPI/资源/包体；明确render-only局限 | XP-0后独立研究宿主，优先WASM/wgpu；JS对照最多一个，不复制游戏系统 |
| XP-2 Vue/Canvas桥接 | 一个文本输入/按钮和图形面，状态/事件/布局、焦点/IME、关闭与迟到输入可判别 | 可与XP-1后段并行，须有可用Canvas/最小bridge；不接生产AutoUI |
| XP-3 UI进入3D | 比较DOM投影overlay与可采样UI adapter，明确遮挡/clip/透明/命中/IME/可访问性与成本 | 方案分析先做，运行probe等XP-1/2及必要adapter；不是把DOM直接当纹理 |
| XP-4 移动两路线 | 原生与移动Web各自验证touch/软键盘/旋转/safe area、后台/恢复和GPU/surface代次 | 设备/依赖盘点可并行，实际probe复用成熟小场景；不同宿主分别记录 |
| XP-5 收敛决策 | 跨端支持矩阵、共享/特有触面、执行环境缺口与后继适配Plan建议 | 只认领实测子集，limited/requires_replan可交研究结论；正式适配另约 |

共享语义树不等于同一物理渲染树：DOM承载Canvas、DOM覆盖Canvas、UI作为纹理进入3D分别协商。常规任意Vue/DOM子树不能直接作为通用WebGPU纹理；overlay不天然参与3D深度。跨DOM/GPU一致性需独立commit/layout/输入合同，GPU失效也不能承诺旧帧缓冲可见。后继Plan据[平台边界](../autographics/platform-architecture-and-research.md)冻结异步资源/ID精度/动作/时钟/能力与生命周期，不给874/875新增跨端AC。

## 6. 延后项与风险

| 风险/候选 | 当前处理 |
|---|---|
| 旧VM的CPU/managed源码空间紧张 | S0先测量；S2探针先证明所需服务可行。不得无声抬限或以Rust替代玩法；失败则据证据重新规划 |
| “完整参考示例”范围失控 | M2交付自有小关卡，参考特性逐项取舍；明确区分参考实现与完整复刻 |
| 静态GLB不足以支持选中美术 | 先盘点，再立公共纹理/动画等必要合同；不静默丢资源，也不默认建设全套PBR/蒙皮 |
| 128节点等旧上限与动态关卡不匹配 | 旧profile保持；新profile规模/生命周期/预算须由S2独立证实并版本化 |
| 网格看起来完成，创作仍不可用 | S1同时验导航、拾取、吸附、取消/历史/Play输入边界与真实画面 |
| 资源外置破坏可搬迁 | manifest＋完整本地闭包＋离线ZIP，缺失/损坏与补齐重启并验 |
| 过早依赖ACE或生产AutoUI | 等ACE完成后再讨论；M2保持独立单窗研究，不用完成UI迁仓替代集成批准 |
| DOM覆盖被误认为纹理/深度正确UI，或native结果推广各端 | XP分别验证呈现层次、UI/输入与实际执行/能力，Gallery按平台分列，不降低M2原分母或提前宣称跨端可用 |

斜坡/移动平台、完整TileMap、45度等距、复杂刚体、骨骼编辑、联网/存档/背包、生产Forge和完整组件主题仍为后续候选；若某项成为冻结样本不可缺条件，应在对应Plan前解决，而不是实施中静默延后。

## 7. 当前接棒（2026-10-29，表现阶段）

状态以本节新核证据为准；前文批次是历史规划背景，不再用旧drafting/在途描述覆盖实际delivery。

| 计划 | 最新事实/本轮合同 | 前置/边界 |
|---|---|---|
| [auto-ui/001 运行对象](../../../../auto-ui/docs/plans/archived/001-autographics-runtime-object-lifecycle.md) | archived，代码交付537122b09c025191355e260bc8e9880a4358cda2 | 不重开/接管 |
| [auto-ui/002 触发与新局](../../../../auto-ui/docs/plans/archived/002-autographics-trigger-events-and-round-restart.md) | archived r5，实际交付0ee5ad0747ec3bb67e1288401fcc31daf3901c6e；簿记已入7e56dc8 | 引擎已交付，不再要求G11反向作前置 |
| [ag-examples/001 G11](../../../../ag-examples/docs/plans/001-autographics-gallery-runtime-objects.md) | 用户确认正在实施；已见唯一.wt/ag-001/ag-examples、codex/plan-001-dev，主Plan簿记尚未同步 | 继续固定002交付，Gallery自己的目录/锁/README/包与独立门；本轮只读不修改 |
| [auto-ui/004 图片与2D精灵帧](../../../../auto-ui/docs/plans/004-autographics-public-images-and-sprite-frames.md) | 新drafting r1，0/10；002实际前置已满足，未启动 | 公共PNG/有界closure、真实Auto取帧/翻向、与角色/相机/触发共同发布 |
| [auto-ui/005 3D纹理与UV](../../../../auto-ui/docs/plans/005-autographics-textured-models-and-uv.md) | 新drafting r1，0/10 | 硬等004实际交付；固定素材色图/UV/变换与完整离线依赖 |
| [auto-ui/006 节点动画与视觉朝向](../../../../auto-ui/docs/plans/006-autographics-node-animation-and-visual-facing.md) | 新drafting r1，0/10 | 硬等005实际交付；无skin clip/Auto phase/yaw，碰撞独立、same-tick |

本仓003已由Blog原生VM路线占用；AG没有另起独立数字序列，按auto-ui本地中央取号器004..006分配，下一号当前007。后续取号仍实时核验。Gallery本地下一号当前002，本轮未预占。

近期引擎顺序：002已交付→004实际交付→005实际交付→006实际交付。它们修改共同Registry/scene/Frame/assembly，串行实施与冻结；不得消费开发tip。ag-001固定002、只写ag-examples，可与引擎新阶段并行；独立SDK准备/target/报告，GPU/共享重型门单实例。新AG路线不改Blog003或其他UI在途实现。

本阶段架构/独立review/各Plan资源合同见[auto-ui表现设计](../../../../auto-ui/docs/design/autographics/m2-presentation-and-resources.md)，实际观察/来源风险见[规划基线](../../../../auto-ui/docs/plans/attachments/004/planning/baseline.md)。004先补2D精灵；005只静态纹理、不伪称character动画可用；006再闭合原character节点clip/朝向。每份T-02必须先验证有限支持与完整真实Auto预算，不静默抬限、删AC或改成专用Rust业务。

### 后继内容与剩余范围

- G09在004实际交付、ag-001共享catalog/工具基线落地后另立ag-examples本地内容Plan；G10a等005、G10b等006，按实际写面拆分。G09/G10的目录、按项锁、README/许可、公共启动、单项ZIP离线A→B不能塞回引擎Plan。现在只列候选依赖，不预取内容号。
- G12声音/HUD、两款独立仓完整关卡、资源制作/发行与Gallery首12项最终闭环仍在后继，不因004..006立项便宣称M2完成；旧G2/G3分母保持。计划数量估计随证据滚动，不把旧11–15份变成完成百分比。
- 旧858素材清单当前可见路径缺失，同号master附件属于ACE路线。005 T-01负责从固定源/可证保留副本恢复身份/许可/依赖，不能混用同号材料或换成当前上游；本轮没有原件新实测/包大小结论。

实现/计划归属随仓：引擎在auto-ui，Gallery在ag-examples；旧canonical Specs仍跨仓引用，本轮未批量复制迁史或修改现状Spec。原始拆分分母见[G11契约](../../../../ag-examples/docs/plans/attachments/001/planning/split-contract.md)，未删除原内容验收。

必要测试仍为受影响用例/可复现抽样，新AC直接正反例必闭合；OS抢焦点自动化暂停，全量AG里程碑pending。XP-1等旁路不在本轮取号/实施，跨端非桌面M2前置，生产UI/HIR/ACE融合继续各自成熟度门。

## 8. 会话续接与动态状态

先读各owner仓当前Plan/§9、delivery/worktree/HEAD/WIP，再核上述依赖。004可作为下一份引擎work的计划入口，本次用户仅要求规划，尚未启动。ag-001在途不接管、不重复建树或发消息。旧880/879/874/875/877/878交付与长期来源风险仍见[2026-10-28交接快照](../autographics/tracking-handoff-2026-10-28.md)，快照及本页旧批次不能覆盖最新收据。远期不预取一串编号、不创建agent/新chat/远端发布。
