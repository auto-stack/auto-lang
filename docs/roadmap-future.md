# Auto 远期 TODO：滚动规划候选池

> 更新：2026-10-26。承接超出当前交付范围的目标，以及后续技术与产品方向；图形排期见当日补充。
> 本文不承诺交付月份，不把全部候选自动装入 v0.7；当前版见 [v0.6 Roadmap](roadmap-v0.6.md)，节奏见[总入口](roadmap.md)。
> 2026-10-24 补充 AutoUI 完整组件接入与默认设计体系候选；研究范围见 Design47 §16，PLAN-840 保持原范围，暂不实施。
> 2026-10-24 进一步补充 Schema 统一 UI/2D/3D 声明、ACE消费增量与分级热重载候选；范围见 Design45 §11 / Design47 §17，840/845不扩面，841合同保持，暂不实施。
> 2026-10-24版本裁定：AACE已明确列为v0.7目标，不再是无版次候选；Rust版ACE全示例成熟度及既有AutoVM/A2X迁移属v0.6主线。详细任务仍按滚动规划冻结，其他候选不自动进入v0.7。
> 2026-10-26图形排期：851已交付为ag-v0.6-m1；下一轮目标已确定为[M2](design/strategy/autographics-m2-v06.md)，先优化，再补网格视口与公共接口2D/3D独立游戏。截至2026-10-27，853–856与858交付、857复审修复，860/861运行草案已建立，其余滚动细化；AutoUI融合/图形归仓及ACE集成等ACE开发完成后再讨论，详[Design45 §12](design/45-autographics-wgpu-engine-and-game-prototypes.md#autographics-stage-851)。下表其余长期能力不自动续建计划。

> 2026-11-12：用户将面向实际工作的[Auto ROS2 SDK](design/57-auto-ros2-sdk.md)纳入v0.6专项，见[六阶段路线](design/strategy/ros2-sdk-delivery-roadmap.md)。SDK核心不再属于本文“待选择”；ROS扩面、系统adapter与机器人新算法仍按各自范围后续选择。本轮只规划，未实施。

## 1. 如何使用

当前版收尾前 1–2 周，结合实际交付、用户反馈、成熟前置、最小成果、风险和容量，从这里选择下一版目标。届时建立 `roadmap-v0.7.md`；本文行序不是执行顺序，也不预排 v0.8/v0.9。

候选进入版本前，必须说明用户流程/工程收益、最小可验收成果、依赖与支持面、成本及不做范围。只选一部分，不因候选数量或“已写设计”全部开工。
未选项目继续保留；已交付项目回链 Plan/Spec/发行证据后更新状态，不能仅凭历史文字推断仍未实现。

本次统一标为“待下一轮选择”，不取消任何已批准在途 Plan，不反向修改已交付合同。v0.6 四条主线所需的局部 ABI、库、安装、诊断等切片仍可随主线交付，其余部分留在此处。

上述“待下一轮选择”不覆盖已由用户确定的AACE版次。v0.7 AACE须先满足Rust版ACE成熟度门；不因最小自举子集可编译而提前实施，范围与出口见[版本合同](plans/reports/ace-milestones-version-boundary-2026-10-24.md)。

## 2. 工程、语言与生态候选

| 候选 | 后续选择的成果边界 | 依赖、来源与约束 |
|---|---|---|
| CAP 跨生态编程架构 | Context/Profile/Capability推导与求解、合法实现选择、弹性stdlib/包；再按需扩展MCU、GPU compute、realtime/deterministic及accelerator | [Design46](design/46-context-profile-capability-cross-ecosystem.md)、[跨生态战略](design/strategy/cross-ecosystem-programming-strategy.md)；新版ACE架构完成且基础AutoGraphics建立后进入实现；AutoShader/Shader Components为纵向候选，未承诺全部平台/后端或固定版次，不扩张在途Plan |
| ACE 性能与后端扩面 | 按实测收益选择 MIR 优化、SSA/低层 IR、自主代码生成、多 native target 或完整 AJ 分层执行 | [ACE 总入口](design/strategy/auto-native-backend-evolution.md)、[Design43](design/43-ac-av-typed-mir-and-abc.md)；Cranelift 可保留，不要求一次完成全部方向 |
| AACE：Auto实现的ACE主体（已定v0.7） | Rust版ACE成熟并复刻现有AutoVM/A2X全部示例后，实施约定Auto主体及代际构建/行为对拍；AAC/AAV组件范围与宿主桥单列 | [主体合同](design/strategy/auto-acc-bootstrap-contract.md)、[版本合同](plans/reports/ace-milestones-version-boundary-2026-10-24.md)；不在v0.6提前迁移，不以包装Rust引擎冒充自举，不保证所有远期优化同版完成 |
| ACE超出现有基线的新语言/库/后端扩面 | 现有AutoVM/A2X全示例及对应源码/宿主/产品迁移已进入v0.6；此处只保留超出该成熟度分母的新增能力 | [能力矩阵](design/strategy/av-language-capability-matrix.md)、[G0–G8路线](design/strategy/av-source-programs-and-avm-replacement-roadmap.md)；不得把当前版必需缺口重新移入远期，新增能力另按需求选择 |
| 旧A2X/AVM迁移后的额外优化/清理 | M4内的兼容迁移/切换按v0.6收口；无关额外重构和扩展继续按证据选择 | [工程拆分M2](plans/reports/v06-extraction-execution-roadmap.md)、[HIR](design/strategy/auto-hir-design.md)；工程拆分M2与ACE-M2不相同，原拆分Plan验收不改 |
| AutoABI、FFI、插件与重载扩展 | 超出当前主线所需部分的动态插件、隔离、所有权与重载能力 | [ABI RFC](design/strategy/auto-native-abi-rfc.md)；按真实互调、代际、退出与失败恢复验收 |
| Atom/Batom v2、Schema 与方言扩展 | 选择实际消费场景，验证表示/往返、版本、资源和语言/工具接线 | [Atom/Batom](design/strategy/atom-batom-v2-design.md)、[Schema DSL](design/strategy/atom-schema-dsl-design.md)；不因新编码另复制 Value/类型系统 |
| 正式标准库与多目标一致性 | 盘点集合、迭代、错误、字符串/编码、JSON、时间、环境、FS/IO、异步与 net；统一 API、稳定性及目标矩阵 | [标准库 Spec](specs/stdlib/project.md)、[装配合同](specs/stdlib/design/assembly-manifest.md)、[Design33](design/33-stdlib-runtime-and-http.md)；承接旧 `roadmap.md` 的完整标准库路线，当前 ACE 所需子集单独纳入 |
| HTTP/net 深化 | 绑定/ABI/流迭代收敛、真实多目标行为对拍；按需求再选 TLS、WebSocket、HTTP/2/3、静态资源和部署扩展 | [旧路线详情](roadmap-history.md#httpnet-重点建设)、[部署合同](specs/stdlib/design/http-service-deployment.md)、[已知债](plans/KNOWN-DEBT-AND-RISKS.md)；不重新打开已归档 738/755，既有发布必修问题不能当成可选远期功能 |
| Python / AI / 科学计算高级支持 | 用代表性模型/科学任务验证数据交换、设备、回调、环境与错误诊断 | [Python 路线](design/strategy/python-parity-roadmap.md)；原“v0.6 主打”改为待选，库和设备矩阵实施时冻结 |
| Rust 生态与 C 接口摄入/部署 | 按真实库选择互操作、header/Foreign IR、布局/ABI或部署切片 | [Rust 库路线](design/strategy/rust-library-replication-roadmap.md)、[C 战略](design/strategy/c-ecosystem-strategy.md)；MCU/板级部署按既有重启条件，不自动启动 |
| Auto语言工具基础设施：parser generator与语言包 | P1复用现有底座服务Auto/auto-edit；P2以差异语言验证通用Grammar/Schema/query与scanner；P3分别验证原生库/绑定、ABC/AV插件及A2X输出；P4按需扩展语义服务/LSP和轻量Rust服务 | [Design39长期定位](design/39-ac-incremental-syntax-and-language-platform.md#language-tool-infrastructure)、[阶段路线](design/39-ac-incremental-syntax-and-language-platform.md#language-tool-roadmap)、[ACE](design/strategy/auto-native-backend-evolution.md)、[HIR](design/strategy/auto-hir-design.md)；787有限语法实验已交付；完整产品未交付，进入版本前逐切片冻结实际API/语言支持/资源及正确性基准，不预设版次；auto-edit轻量消费、语义按需外置，不改其现行后端/计划，本次只设计 |
| 调试、测试与开发体验 | 选择 LSP、错误诊断、trace、反例与基准等可独立验收的改进 | [旧完整基线](roadmap-v0.6-baseline-v2.md)、[更早路线](roadmap-history.md)、[Spec 入口](specs/overview.md)；先核对现状，不能把历史清单当作今日缺口 |

## 3. 系统基础设施与平台候选

| 候选 | 后续选择的成果边界 | 依赖、来源与约束 |
|---|---|---|
| 系统级 Package / Install | 应用、工程、知识包和用户数据的统一身份/生命周期；CLI 与 Launcher 共用安装升级卸载及恢复 | [统一安装设计](design/strategy/unified-package-install-strategy.md)；原“本版必做”不再生效，v0.6 Linux/工程所需最小交付仍要验收，不能另造弱安装器 |
| 应用通信与 ROS2 扩面 | AutoOS F0–F2及系统adapter、多版本/RMW/设备、空间工具等继续按需选择；SDK的R06-01..12已纳入v0.6，不留在本池 | [系统基础设计](design/strategy/system-foundations-v06.md)、[SDK设计](design/57-auto-ros2-sdk.md)；直接节点不等待整套系统通信，新机器人算法仍另议，bridge不代SDK完成 |
| AutoAI 与 Agent profile | 普通 client、Agent/MCP bridge 共用真实能力与任务合同 | [系统基础设计](design/strategy/system-foundations-v06.md)；复用现有 daemon/client，明确取消、结果、资源与可观察性 |
| 系统级 DevTools / AutoLens | 系统 Inspector、真实树/布局/像素比较、两 backend 共同能力、第三方样板与采集来源 | [F3–F5 设计](design/strategy/system-foundations-v06.md)；v0.6 Forge 局部对象检查可先交，不能推称整个系统能力已完成 |
| Linux 扩面与产品化 | 扩大发行版、硬件、会话、系统集成与应用矩阵，逐步完成原四大 app/28-demo 等目标 | [平台专项](design/strategy/platforms-v06.md)；先复用 v0.6 首发证据，按支持矩阵扩展，不重复冒领首次发行 |
| 鸿蒙 | 虚拟桌面与 28-demo 的启动、布局、交互、状态、导航和必要服务；复杂独立 app 后续分期 | [鸿蒙战略](design/strategy/harmonyos-ecosystem-strategy.md)、[平台专项](design/strategy/platforms-v06.md)；原 v0.6 最低目标改为远期待选，HelloWorld 不替代整套矩阵 |
| Android、iOS/macOS、MCU/LVGL 等 | 按实际用户需求选择一个平台和有限支持面 | [Mobile 战略](design/strategy/mobile-jetpack-strategy.md)、[平台专项](design/strategy/platforms-v06.md)、[C 战略](design/strategy/c-ecosystem-strategy.md)；保留已有后端，不默认重选或删除 |
| AutoScape / AutoWeb | 应用访问/provider、URI/Intent、session/导航与网络激活按阶段选择 | [Design31](design/31-autoscape-autoweb.md)；沿用已有施工图，网络激活及其安全关闸同船；复用安装/通信，不另造一套 |

## 4. 图形、应用与个人服务候选

| 候选 | 后续选择的成果边界 | 依赖、来源与约束 |
|---|---|---|
| AutoShader最小纵向切片与Shader Components | 现在只完成小范围设计；未来Auto→受检语义→WGSL→现有wgpu，验证纹理着色/参数背景/共享helper，再做可组合效果 | [最小设计](design/strategy/autoshader-minimal-language-and-renderer-interface.md)；需实际ACE/图形接口，有限CAP可逐步接通；基础图形不依赖AutoShader完成，830只补资源/布局/renderer交接，不加入编译器/自定义Shader加载/重载；完整CAP仍守原进入门 |
| AutoGraphics M2质量收敛 | M1总结已形成；853基线/证据→854行为源→855宿主职责，保留原行为/预算/验收分母并作前后对照 | [M2战略](design/strategy/autographics-m2-v06.md)；853–855已交付，S0完成；856/858交付、857复审修复，860/861为下一批，未认领M2完成，不扩大851 |
| AutoGraphics M2及后续引擎能力 | M2补网格视口、公共服务与独立2D/3D平台游戏；更广渲染/资源/物理/动画/平台继续候选 | [M2设计](design/autographics/m2-editor-and-public-game-prototypes.md)；后续功能按阶段细化，不承诺额外3～5计划或下月全量扩面。独立功能留开发分支，生产融合仍等ACE完成后再议 |
| AutoUI 完整组件接入 AutoGraphics | 保留迁移后全部公共/组合组件分母，props/事件/状态、外观、嵌套/弹层/输入/资源及各适用后端兼容证据与差异 | [Design47 §16](design/47-autoui-state-design-and-feature-architecture.md#16-autographics-后续完整组件接入与默认设计体系2026-10-24)、[Design45 §12](design/45-autographics-wgpu-engine-and-game-prototypes.md#autographics-stage-851)；等ACE开发完成后再讨论融合，UI迁仓不是单独放行条件；长期分母保留，不作当前v0.6最低生产交付，840不扩面 |
| AutoUI 默认设计体系与跨端样式一致性 | 研究 token/语义角色、多部件 recipe、默认设计包和 legacy 映射；明确 app/window/subtree 主题继承、共享 Portal/弹层归属、更新/缓存及 CSS/native 的一致语义与实际视觉容差 | [Design47 §7/§16](design/47-autoui-state-design-and-feature-architecture.md#163-默认设计体系的研究范围)；复用现有主题/样式，旧应用与新默认分别验收，Vue/VM 为共同基线、其他路径逐项列支持表；不静默换肤，不把 Figma 双向同步或整个 Design47 实现变为前置，暂不实施或绑定版次 |
| 基于新版HIR的UI/2D/3D统一方言 | Atom＋Schema定义HIR及节点/组件合同；普通Auto、AutoUI/图形、AutoShell以方言语法/语义校验和lowering消费共同基础，保留来源/版本与持久项目区别 | [Design45 §11/§12](design/45-autographics-wgpu-engine-and-game-prototypes.md#autographics-stage-851)；等ACE开发完成后再讨论生产融合，消费841正式端口而不扩其解耦验收；组件分母/布局/坐标语义保持，旧VM原型或scene/4不冒认HIR迁移；840/845不扩面 |
| 图形/AutoUI消费ACE增量与保状态热重载 | 未来首例带真实函数级复用、依赖失效、fresh等价与生命周期；Stop/Run和状态schema/安全点/代际/资源迁移分别验收 | [Design45 §12](design/45-autographics-wgpu-engine-and-game-prototypes.md#autographics-stage-851)、[Design47 §17](design/47-autoui-state-design-and-feature-architecture.md#17-schema-统一声明841与ace热重载的衔接2026-10-24)；排期等ACE自身开发完成后再讨论，不按局部profile提前启动；首例增量合同/失败保旧/未保存状态/owner要求保留，编译增量不抵扣HMR |
| AutoForge 产品化与承载游戏 | 更完整编辑/创作、Agent实际修改与恢复；游戏按规则/内容闭环选择 | [图形/Forge专项](design/strategy/autographics-forge-v06.md)；独立作者雏形按实际Spec认领，生产融合/产品化等ACE完成后再讨论，旧Forge Agent文档不作新应用交付证据 |
| 知识底座与个人知识管理 OS | 分期连接采集、理解、存储/来源、检索、学习实践、创作与反馈；Jade/Musk 共享底座但 scope 分明 | [知识/学习设计](design/strategy/knowledge-learning-v06.md)；原“v0.6 原型/v0.7 完整主线”改为候选池，完整流程不自动承诺一个月完成 |
| 论文阅读、伴读与作品驱动学习 | 选择导入/来源→注释/查询→重开或目标→练习→反馈的有限流程 | [知识/学习设计](design/strategy/knowledge-learning-v06.md)；抽取准确性、可追溯性和真实学习流程分别验收 |
| AutoDown / Typst 排版 | 固定文档与导出、增量、资源及编辑预览接线 | [排版设计](design/strategy/autodown-typesetting-v06.md)；复用已有文档引擎，不复制实现，先核对 auto-down 的在途交付 |
| 输入法 | 选择独立引擎/宿主与输入组合、候选、提交取消、焦点和配置恢复 | [输入法设计](design/strategy/input-method-v06.md)；v0.6 RQHost 所需的现有输入法接入不能因此延期，新完整输入法另选范围 |
| 四大应用持续演进 | Shell、Musk、Edit、Jade 各选高价值且可恢复的真实流程 | [应用战略](design/strategy/demo-app-independence-strategy.md)；锁定工具链与资源，依赖 SDK 的迁移随工程合同验收，独立产品新功能待选 |
| 28-demo 教学/产品分工与应用独立化 | 保留教学源、来源历史和已建产品仓，逐应用完善独立构建、运行与发行 | [应用战略](design/strategy/demo-app-independence-strategy.md)、[建仓清单](reports/demo-app-repositories-20261003.json)；28 个 demo 与产品仓数量不是一一对应；已完成建仓不重复立项 |
| Launcher、Notes、Reader、Blog | 分别选择 Windows 启动/搜索插件、可靠捕获、阅读摘录、持久草稿与本地发布等产品闭环 | [旧应用规划入口](roadmap-v0.6-baseline-v2.md#26-首批真实应用规划2026-10-04)、[应用战略](design/strategy/demo-app-independence-strategy.md)；所属仓已有规划继续保留，跨仓排期需在那里落实 |
| 个人服务、新交互、Career 与娱乐 | 从代表性个人活动确定流程、上下文与 AI 参与方式 | [历史服务定位](roadmap-v0.6-baseline-v2.md#12-个人操作系统与个人服务2026-10-03-用户确认)；需求与价值需验证，不提前绑定版次 |
| 社区/媒体共享、OpenBusiness 与应用市场 | 内容/作品共享、成本分摊、定价/分配/治理、开放发现与用户应用列表 | [历史商业方向](roadmap-v0.6-baseline-v2.md#13-openbusiness共建内容与成本分摊)、[开放生态](roadmap-v0.6-baseline-v2.md#14-开放应用生态与应用市场2026-10-03-用户确认)；机制假设与可运营产品分别验证 |

机器人算法仍保持“通信框架成立后另议”的约束，BEV/SLAM、训练与板级算法 demo 不因移动到本池而自动启动。

## 5. 下一版选择时的记录

每个被选候选记录：实际已交付基线、用户/工程成果、版本内支持面与验收、必要依赖、所属仓与 owner、容量/收口余量，以及继续留在远期的部分。跨多个版本的大方向可以保留长期设计，但每次只承诺本轮切片。

当前候选未排序，也未分配新 Plan ID；旧日历和进度保留在[完整原基线](roadmap-v0.6-baseline-v2.md)与[更早路线](roadmap-history.md)。本次没有执行实现、功能验证或跨仓改写。
