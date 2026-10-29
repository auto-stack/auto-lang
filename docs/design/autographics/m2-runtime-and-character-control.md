# AG M2运行能力：从真实Auto探针到公共角色接口

> **计划管理归属更正（2026-10-29）**：AG引擎计划在auto-ui/docs/plans，Gallery独立内容计划在ag-examples/docs/plans；跨仓计划由主责任仓持一份合同。882/883已迁至auto-ui（drafting r2），880最新收据已交付归档并迁至ag-examples。auto-lang原位置只留定位指引，历史Specs/设计本次未批量迁移；下文早期“中央Plan仍在语言仓”表述已由本条取代。

日期：2026-10-29。状态：设计v7；角色/相机874/875与纯迁仓879已交付，AG唯一实现权威为auto-ui/experimental（4adf2d03）；880在途。下一批[882有限对象](../../../../auto-ui/docs/plans/001-autographics-runtime-object-lifecycle.md)→[883触发/新局](../../../../auto-ui/docs/plans/002-autographics-trigger-events-and-round-restart.md)已规划，尚未实施。长期目标与分母不变；早期状态段落是历史背景，当前以Plan/代码/收据为准。归属[Design45§13](../45-autographics-wgpu-engine-and-game-prototypes.md#13-m2运行对象触发与新局事务2026-10-29拟议合同)。

## 1. 当前进度与目标

853–855质量收敛、856/857两维编辑视口均已交付，S0/S1完成。858 Phase1研究固定了G2-01..09/G3-01..08、来源和候选原始资源；此处只认领AG研究部分，不替其他路线扩展阶段下结论。860真实Auto探针已交付go，861公共动作/2D角色已交付，当前AG源码为`8b12632d7b5b1b0df72fbb4eef84297a054de31d`。

下一阶段继续补游戏运行服务。858源素材与研究参数并非可运行工程：公共3D角色、对象生命周期、运行相机、精灵/纹理/动画、声音/HUD与两款独立游戏仍待实现。所选character模型有纹理和节点动画、无skin；M2需要相应支持或受检派生，不自动承诺通用骨骼工具链。

## 2. 先证明机制，再建立公开入口

现有v5只支持两接触位移并输出最后接触法线。角色需要最终支撑/墙顶/速度与跳跃边沿：地面后再碰墙、切向离开边缘、顶碰撞及舍入停止都不能靠上一帧grounded或最后法线猜测。

[860](../../plans/archive/860-autographics-game-runtime-feasibility.md)在固定SDK和原硬预算下完成真实Auto G2/G3静态角色与128节点探针，两候选均通过，结论go；公共接口以861为准，probe仍不是公开交付。

[861](../../plans/archive/861-autographics-public-actions-and-character-control.md)已交付`autographics.character/1`：真实OS输入→Run/步动作→用户Auto回调与受信有限控制器→完整候选校验→原Host/GPU呈现，有限单2D角色/静态solid，旧行为版本/scene4保持。公开细节见[canonical Spec](../../specs/auto-graphics/game-actions-and-character2d.md)。

[874](../../plans/archive/874-autographics-public-character3d.md)接公共3D角色：独立新profile、六动作、水平归一、静态三维地墙顶/最终支撑、实际OS/三维内容与128负载。先冻结整路径资源，仍保留32credits等原硬界；方向基先由用户源码配置，未交付运行相机或模型旋转。

[875](../../plans/archive/875-autographics-runtime-cameras.md)等待874交付后冻结组合profile，补2D跟随夹界/等比内容区、3D跟随/左拖环绕/滚轮缩放及真实相机相对运动。候选orbit先确定同dispatch水平基，Auto积分后以接受角色位置构造camera，再共同准入/prepare/publish；失败保留原camera与actor，不能分步泄漏。编辑view/Grid/历史分离，Stop恢复。编辑camera或固定heading数学测试不能替代此验收。角色可辨识模型朝向/动画仍由后继表现计划完成。

## 3. 拥有者与支持面

| 层 | 负责 | 边界 |
|---|---|---|
| 用户Auto | 固定步积分、速度/跳跃、方向归一、游戏状态、参数与动作消费 | 不编写transport，不依赖专用游戏宿主分支 |
| 受信Auto角色单元 | 有限碰撞/最终支撑、位移/接触结果与配置边界 | 有限静态形状/预算；不绕过用户准入或改旧API |
| Host/Play | 输入采样/Run与步身份、配置、完整收集/准入/准备/发布、系统/GPU资源 | 不把Rust oracle放进正常玩法；保持一份状态/调度栈 |
| 测试/证据 | 独立数值/几何oracle、实际内容/owner、来源与预算记录 | 回放/探针与真实OS、研究与公共交付分别认领 |

初期861支持单个2D角色、128节点内固定solid集合、静态AABB与平移祖先。没有坡面/动态平台/自由刚体/动态拓扑，也不把金币显隐当实例销毁；后继对象/触发服务提供正式身份/代次/事件语义。动作队列/结果/状态与资源有明确上限，失败保last-good、Stop释放、新Run隔离旧数据。

## 4. 当前M2剩余工作估计

以下为860/861交付后的滚动估计，**包含本批874/875**；不含已完成计划，不是时间或交付承诺。

| 工作包 | 预计Plan数 | 主要结果 |
|---|---:|---|
| 3D角色扩展 | 1（874） | 静态三维地墙顶/运动方向及相机水平基接缝 |
| 动态实例/身份与触发事件 | 1–2 | 创建/销毁/新局代次、静态/trigger区分、一次触发及错误恢复 |
| 运行相机 | 1（875） | G2跟随边界、G3跟随/环绕/缩放和相对运动，独立于编辑view |
| 2D精灵/帧动画及派生资产 | 1 | 可辨识玩家/敌人/状态、真实纹理与资源闭包 |
| 3D纹理材质/节点动画 | 1–2 | 所选GLB/外部色图/纹理变换与idle/walk/jump；或受检且等价的派生 |
| 公共声音与HUD | 1–2 | 事件一次播放、计数/结束同步、暂停/重开/退出和失败处理 |
| 独立2D、3D游戏 | 2 | 各自仓外公共接口工程，完整失败/获胜/重开、编辑保存/搬迁 |
| M2资源发行与整体验收 | 1 | 固定引擎/SDK/两游戏/资源矩阵、完整离线ZIP、损坏/修复、独立最终复审 |
| Gallery目录/基础与增量内容 | 2–3（净新增） | GY-A基础6项、GY-B角色/相机；其他示例在未来对应能力Plan认领，GY-C可并入原发行或另拆1份 |

原引擎/游戏路线约9–12份，加Gallery净增2–3份，合计**约11–15份Plan**，含874/875；两份交付后约9–13份。Gallery目录/基础6项可独立并行，G07/G08分别等874/875交付，精灵/纹理/节点动画/触发/声音示例消费原定能力，详[Gallery专题§6](m2-gallery-and-capability-matrix.md#6-并行路线与后继工作包)。下一批引擎优先对象/触发及表现。若接口/素材/预算/内容适配需拆分则重估，不能删分母或抬限兑现数量。

## 5. 结束条件与依赖

角色样板、研究报告或两个视口完成均不独立构成M2。最终须M2-01..09（Gallery为本轮增补目标）、G2/G3全部必需条目、公共入口/资源闭包与真实图像/声音/输入/身份/owner结果。冻结源素材仅约0.2MiB，最终派生资产、引擎和ZIP大小须实测，不因当前候选小就省略离线/搬迁验收。874/875不因此增加Gallery任务或验收项。

引擎861→874→875；874现场实现WIP不等于交付，875继续等实际进入门。并行[877](../../plans/archive/877-autographics-gallery-catalog-and-basic-six.md)固定消费861、已executing/仓树建好，[878](../../plans/archive/878-autographics-platform-boundary-and-dependency-audit.md)静态审计仍drafting；不追加874/875任务。最新状态按[交接](tracking-handoff-2026-10-28.md)入口重核，不重复建树/接管。两维角色＋相机仍非完整M2，UI/HIR/ACE融合继续原门，纯归仓仍只是875收口后的独立候选。

跨端公共运行边界见[平台专题](platform-architecture-and-research.md)：语义动作/逻辑tick与原生/浏览器调度隔离，JS/WASM身份精度、异步资源代次及GPU/surface恢复另立合同；DOM/GPU跨呈现域一致性不直接继承原生同设备整帧保证。874/875的键鼠/Windows/单窗/预算与验收不变，XP研究不纳入本节11–15份桌面M2估算，也不成为交付前置。


## 6. 下一阶段执行合同（879交付后，拟议）

882补有限运行实例：从已接受叶模板注册新身份(run,slot,generation)，最多8槽非solid根叶，base+reserved≤128、mesh潜在≤512；创建/销毁不是显隐，代次不回绕，旧句柄不可操作同槽新生。静态角色/solid和camera维持原合同。完整新对象集合/角色/相机同事务，源/SDK/时间/步数/32credits预算不升，T-02先做真实Auto全路径可行性门。

auto-ui/002等auto-ui/001实际交付后冻结签名/价格，补最多8对触发、单tick确定enter/stay/exit及一次读取、圆/球与实际AABB几何（不能用包围盒替半径区域）、事件/实例共同发布与新Run隔离。用户Auto拥有收集/巡逻/胜负/Enter请求；Host只按受检请求创建新VM/Run，新初始化失败保完整旧呈现、旧消息不可入新局。G11两维/目录/锁/验收/包现由[ag-examples/001](../../../../ag-examples/docs/plans/001-autographics-gallery-runtime-objects.md)独立承接，硬等auto-ui/002实际交付；002只做引擎与必要夹具，不写内容仓。

各Plan在所属仓独立簿记/工作树/复审/合入，auto-ui/001→002引擎串行，G11另在内容仓固定消费002真实交付；已有001实施不由本轮接管，旧知识保持跨仓引用。真实OS自动化仍暂停，用公开逻辑输入/无窗口内容与适用的未变证据；不把新语义未测写Ready。本阶段不交付完整游戏/精灵动画/声音/HUD，也不提前跨端或融合ACE。
