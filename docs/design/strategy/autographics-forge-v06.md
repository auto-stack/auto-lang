# AutoGraphics、AutoForge 与承载游戏：v0.6 阶段设计

> **2026-10-30 总体要求纠偏**：AG用AutoUI式的声明、组件、绑定和事件表达图形应用；图形节点扩展共同作者模型，普通use复用代码/组件，复杂声明或逻辑可拆分。Godot/Bevy作为能力与样本参照，不决定作者API。当前JSON/单文件行为桥仅是独立原型接口，“可审计性”不构成禁use、单模块/单VM或单场景的产品要求。树状声明不固定运行存储，未来可选择数据导向/ECS执行；现阶段不实施该优化。详[目标要求与六项讨论草案](../../../../auto-ui/docs/design/autographics/declarative-authoring-model.md)。本条修正总体方向，不重写历史交付、在途AC或既有ACE成熟度进入门。

> 更新：2026-10-26；M1已交付并建立ag-v0.6-m1，当前进入M2规划：优化为起始阶段，随后补网格视口与公共接口2D/3D游戏。具体顺序见[M2战略](autographics-m2-v06.md)。
> 2026-10-28续接：874/877已有在途实施/准备，875/878仍待交付；以[跟踪快照](../autographics/tracking-handoff-2026-10-28.md)重核Plan/owner。Gallery、跨端与纯归仓候选已分开设计，未实施跨端/迁仓；M1标签来源映射差异亦在快照记录，不自行重打标签。
> AutoUI融合、图形归仓/生产主分支合入与ACE集成等ACE开发完成后再讨论；当前保持autographics-dev。本安排更新此前v0.6生产融合的排期，长期目标保留，详[Design45 §12](../45-autographics-wgpu-engine-and-game-prototypes.md#autographics-stage-851)。
> 本日用户澄清：首版渲染底层确定为 **wgpu**，与 AutoUI/Iced 的 wgpu 路径融合；调研聚焦其上的图形/游戏能力，并以 AutoGraphics 复刻 Godot/Bevy 游戏示例验证。首批代表性示例与自研承载游戏分开排期。
> 来源：AutoOS《3D引擎方案比较》后续讨论；继承[第三代互联网战略](thirdgen-internet-strategy.md)。本稿不替代实施 Plan 的支持面、资源预算和验收。
> 829/830为首批游戏/组合实验来源；当前已交付能力查[模块Spec](../../specs/auto-graphics/overview.md)与归档收据，当前阶段功能合同见[851](../../plans/archive/851-autographics-bounded-move-and-slide.md)。本文排期更新不改变在途Plan验收、进度或历史交付事实。

## 1. 定位与已有材料

AutoGraphics 提供 UI、2D、3D 的场景与组合能力；AutoForge 是以运行中的产品为主要操作对象的创造应用。AutoMusk 继续承担通用对话式 Agent，AutoEdit 保持轻量编辑器定位，不因新创造工具合并产品。

首版采用 wgpu 作为确定的 GPU 渲染底层，与 AutoUI/Iced 的 wgpu 路径共用图形基础。AutoGraphics 在其上建立自己的场景、组合、2D/3D 渲染与游戏运行能力；Godot/Bevy 是功能、架构与示例参照，不再作为首版整体引擎选型候选。早期 Godot/GDScript 路线及已有 demo 保留为参考资产，不算 AutoGraphics 自身运行证据。
版本对齐、实际 Device/Queue 共用、离屏目标/合成顺序、颜色/alpha/深度约定及资源生命周期仍需实测；两边都依赖 wgpu 不等于已经完成共享。未来统一替换底层时同步适配 UI 与图形渲染路径，公共场景/资源/输入语义不直接暴露 wgpu 对象；本轮不预建第二套后端。
现有 `forge/` 文档和 auto-musk 的历史 Forge 拆分设计描述旧 Agent，不证明新的产品创造应用已经存在；旧仓保全与新产品建仓另定 Plan。

## 2. 设计边界

- 公共概念候选为 World/Node、Space、View、Surface、Portal、资源与输入路由；各自身份、所有权、更新时间、坐标系和销毁规则先于引擎绑定。
- Auto 拥有场景、组合与游戏公共语义；首版渲染使用 wgpu，物理、动画、字体、模型加载和音频等按能力复用适当库。wgpu 提供 GPU 基础，不代替游戏循环、碰撞/物理、资源系统和玩法状态。公共资源 ID 不直接采用某个库的对象指针，不以重写所有基础库为目标。
- 独立原型可以手工构造场景模型；这只验证图形核心，不证明 Auto 源码前端或 AutoUI 已接通。声明语法/lowering 后续消费语言 SDK，不把场景关键字塞回 core AST。
- 组合语义支持 UI、2D canvas、3D canvas 的任意类型嵌套与递归；共享 GPU、UI 纹理、Portal 输入是必须验证的接缝。明确资源/深度边界、循环拒绝、帧同步和释放；支持递归组合不等于无限层数或无成本。
- AutoForge 的选择对象需要稳定身份与修订号，修改须经预览/应用/失败恢复。操作日志记录任务、变更和结果；尚未完成的任务可观察和取消，不能把动画效果当作真实执行进度。

## 3. v0.6 阶段目标与验收

| 子线 | 本版交付目标 | 可验收成果与失败判据 |
|---|---|---|
| AutoGraphics 核心 | 独立可操作的2D/3D研究原型，851为阶段功能终点 | 已批准Plan逐项验证真实Auto、资源/场景编辑、查询/运动及原组合/输入/生命周期；独立原型交付不代表生产AutoUI或ACE集成 |
| 游戏运行与示例复刻 | 以首批代表性 2D/3D 游戏示例驱动必要引擎能力 | 从 Godot/Bevy 官方示例选择并冻结来源版本、玩法/画面/资源与交互分母；Auto 源码经 AutoGraphics 自身完成启动、游玩、计分/胜负或等价结束、重开及退出。所需碰撞、动画、音效、UI 等逐项验收，不以调用原引擎或仅画相似截图冒领复刻；具体首批样本在 Plan 确认，不承诺整个示例库本月全覆盖 |
| 组合接缝 | 当前保留独立Iced/Surface/Portal实验，生产融合后议 | 保留UI→3D→UI等原实验分母；真实AutoUI、共同HIR方言和生产宿主接线等ACE完成后重新讨论，不能把现有实验标为融合完成 |
| AutoForge | 独立场景作者工具的研究雏形 | 查看/选择→修改→预览→保存→撤销→重开及失败恢复按已批准实验Plan验收；生产组件、统一声明、产品化和Agent闭环等后续讨论，不追加当前里程碑 |
| 阶段总结与优化 | M1总结已建立，M2/S0收敛后实施新增能力 | S0/S1及860/861已交付；874→875原范围保持，另规划Gallery/能力矩阵并行内容，详M2战略与[Gallery专题](../autographics/m2-gallery-and-capability-matrix.md) |
| 承载游戏 | 远期待选，本版不承诺 | 国风种田/资源与战斗、装备/符箓的规则与内容方向保留；下一轮选择最小状态/资源闭环后再立项，不把规则原型或完整游戏加入本月发行门槛 |

首个实施 Plan 需固定目标平台、库版本、场景规模、帧时/内存预算和输入序列；本轮不编造性能成绩。先做能淘汰错误方案的组合实验，之后再决定展示内容与生产引擎投入。

## 4. 依赖与拆分边界

独立图形/游戏原型不等待 AC/AAC 或三仓搬迁；以 wgpu 和固定窗口/资源依赖建立独立可运行宿主。可设独立图形集成开发分支，各实施 Plan 仍使用专用 worktree，定期同步主线；不改迁移中的 UI/语言核心、公共 manifest/lock 来便利原型。物理/动画 adapter 可独立验证，运行时 AutoABI 接口需消费对应版本合同。
当前继续独立图形/真实Auto游戏及轻接口实验，保留autographics-dev；M1后按M2先优化，再补视口与公共游戏能力。AutoUI迁仓本身不放行图形合入或ACE接线，等ACE开发完成后再讨论范围/接口和实施Plan。早期Rust driver不替代真实Auto证据，现有真实Auto实验仍不代表HIR/ACE生产融合。本文不创建分支或实施树。

未来真实AutoUI融合除Design37 renderer/宿主所有权及稳定SDK/输入/生命周期接口外，还须按Design45 §12等ACE完成后再讨论；涉及iced patch、UI native、VNode/VTree或RenderQueue仍由对应owner负责。独立作者工具继续用稳定身份和局部接口验证编辑；生产组件、源码与构建重载接线不提前加入当前阶段。
不修改正在迁移的 UI 核心来便利原型，不复制 VTree、类型系统或包管理器。独立原型验收与集成验收分别出证据。
RQHost 现有 CPU 原语记录/重放与缺省软件绘制属于宿主接线现状；研究怎样容纳 wgpu 图形绘制、进程间场景更新、资源身份和输入，不据此重开首版渲染底层选择。同进程共用 Device/Queue 与跨进程协议分别验证，不能通过直接跨进程传 Rust/GPU 对象引用冒充接线完成。

## 5. 本版不承诺与待决事项

完整分层编辑器、无限层数/无资源边界递归、XR/网络世界、自研成熟游戏、完整物理/动画工具链进入远期待选。当前仅认领独立研究Plan支持的类型组合、嵌套与首批示例能力，生产AutoUI/HIR/ACE融合按2026-10-26再议门后置；既有示例必需的碰撞/动画与原验收不删减。Godot/Bevy、a2gd、旧demo/仓库/历史资产保留来源与对照。
首个实施 Plan 冻结第一宿主/target、场景交换模型、共享设备方式、预算与首个真实编辑任务；本次没有选定具体库版本或声称性能达标。

## 6. wgpu 之上的图形与游戏能力调研

下表是长期能力分解与首轮探针的范围，不将完整 Godot/Bevy 功能集自动加入一个月的交付。优先按第 7 节示例所需能力建立真实实现；高级能力按后续月度容量选择。

| 层 | 要研究的能力 | 首轮探针/决策 |
|---|---|---|
| UI/图形共同 GPU 基础 | wgpu 版本与 Device/Queue、纹理/缓冲区/采样器、pass 顺序、深度/颜色/alpha、离屏合成、资源预算 | 同一宿主完成真实 Iced UI 与 2D/3D 合成，确认资源所有权与释放；版本相同但设备不同的情况不能声称已共享 |
| 场景与运行调度 | 节点/组件或 ECS 的边界、层级变换、多世界/视图、对象稳定身份、事件、帧更新/固定步长、暂停/重开 | 明确保留状态及变更集；调度、输入、碰撞与渲染时序可复验，不要求复制 Godot API 或 Bevy ECS 内部实现 |
| 2D 渲染 | sprite/图集、几何/路径、文本、层级/裁剪、相机、tilemap、sprite 动画、批处理 | 先支撑选中 2D 游戏；tilemap 等由实际样本选择，复用现有字体/文本能力，不重写全套 UI |
| 3D 渲染 | 网格/索引、相机/投影、材质/贴图、灯光/PBR、阴影、透明、实例化、剔除、后处理 | 先完成首个 3D 游戏需要的相机、材质/光照和场景；阴影/高级效果按样本与阶段分开验收 |
| 组合与输入 | Surface/Portal、UI/2D/3D 嵌套、坐标/射线/UV 映射、命中/遮挡、焦点、拖拽、键盘/手柄动作映射 | 可交互混合场景与真实 UI 回路；录制固定输入，核对目标对象与状态，不只检查画面 |
| 资源与动画 | 模型/纹理/字体/场景加载，glTF 等格式、缓存/复用、异步与错误；时间轴/tween、骨骼/蒙皮/动画状态 | 按示例资源盘点需要的格式与动画；局部热改、缺资源拒绝、重复加载/卸载与关闭后资源有界 |
| 碰撞与物理 | 2D/3D 形状查询、触发器、刚体、角色控制、重力/跳跃、碰撞事件与调度同步 | 简单碰撞和完整刚体系统分别决定；复用库的集成成本与版本匹配取证，游戏玩法不以手工摆画面代替 |
| 游戏服务与 Auto 接口 | 输入动作、计时/随机、音效/音乐、关卡/生成销毁、计分/胜负/重开、存档及 UI；Auto 描述/更新/事件接口 | 从公共 AutoGraphics API 实现首个游戏；声音/状态与暂停退出一致；Rust driver、旧 a2gd 和真实 Auto 入口分别报告 |
| Forge 与诊断 | 场景/对象检查、选择与 gizmo、属性编辑、保存/撤销、源码映射、帧时/内存/资源计量 | 编辑同一个实际运行场景，验证修改/恢复；不先依赖完整系统 DevTools 或完整游戏编辑器 |

## 7. 用 Godot/Bevy 示例驱动复刻

长期目标是让 AutoGraphics 能够复刻 Godot/Bevy 的代表性图形与游戏示例，由示例覆盖逐步扩展到较完整的引擎能力。[PLAN-829](../../plans/archive/829-autographics-independent-research-and-game-prototypes.md) 首批选择 Bevy Breakout 与 Godot Squash the Creeps，冻结来源提交及原型 profile，并安排真实 Iced 共享设备探针。共同基础先行，2D 先形成完整反馈，3D 模型/渲染风险提前验证；合同稳定后两轨可重叠推进，不等所有 2D 完成才开始 3D。
本 Plan 的 Rust driver 原型与后继真实 Auto 入口、生产 UI/RQHost 和 Forge 集成分开验收；后者仍 pending，不能用原型完成抵扣。本表其余候选不会自动变成本 Plan 新增样本。

| 候选 | 可驱动的能力 | 来源 |
|---|---|---|
| 2D Breakout | 游戏循环、sprite/几何、输入、碰撞/反弹、计分、音效与重开 | [Bevy 官方示例](https://bevy.org/examples/games/breakout/) |
| 2D Dodge the Creeps | 场景/实体生成、移动/碰撞、sprite 动画、计时、HUD 与游戏状态 | [Godot 首个 2D 游戏](https://docs.godotengine.org/en/stable/getting_started/first_2d_game/index.html)；与 Breakout 可择一先做，不能自动推称两者都已纳入 |
| 首个 3D 游戏教程 | 3D 场景/相机、模型/材质、移动/跳跃、碰撞、生成/结束及 UI；逐项以选定来源为准 | [Godot 首个 3D 游戏](https://docs.godotengine.org/en/stable/getting_started/first_3d_game/index.html) |
| 可交互的 3D 中 UI，加嵌套 canvas | 渲染到纹理、射线/UV 输入、焦点、多层组合和实时编辑 | [Bevy UI to Texture](https://bevy.org/examples-webgpu/ui-user-interface/render-ui-to-texture/)，再补 Auto 自有混合/Forge 流程 |

每个选中样本登记来源 tag/commit、资产/许可来源、所需能力、Auto 实现入口、固定输入/随机种子、关键画面与状态 oracle、性能/资源预算、错误/退出反例。截图验证外观，输入回放与状态验证玩法，两者分别过关。
改写到 AutoGraphics 可以使用自有 API，不要求源码/API 逐字兼容 Godot/Bevy；选中样本如需删减功能，应明确取得范围决定并列出差异，不能通过运行原引擎、转译到 GDScript 或隐藏删项宣称复刻成功。

底层依据：[wgpu API](https://docs.rs/wgpu/27.0.1/wgpu/)、[Iced wgpu Primitive](https://docs.rs/iced/0.14.0/iced/widget/shader/trait.Primitive.html)。链接版本用于对照当前工程依赖，实际实施以锁文件和 Iced 适配版本为准；本稿没有执行这些游戏或设备验证。

## 8. 从基础图形走向跨生态语言架构（2026-10-22）

后续方向为 **ACE → 基础 AutoGraphics → CAP → 多生态扩面**；设计合同见[Design46](../46-context-profile-capability-cross-ecosystem.md)，阶段与命名见[跨生态战略](cross-ecosystem-programming-strategy.md)。此前独立 Draft 0.2 未检出，本次依讨论重建这两份补编，不替换本文件已有 wgpu、示例、组合和迁移交接合同。

Context 表达场景，Profile 组织语言/运行时/库与资源约束，Capability 表达具体代码需求。函数与语言特性的需求经 inference/调用图传播、约束求解及合法实现选择，进入 ACE 受检 HIR/MIR 和适用 backend；core/math 作为通用基座，host/device/MCU 库与包按细粒度摘要、条件依赖和可达闭包裁剪。实际非法调用报错，不能通过删除代码或静默 fallback 掩盖。

AutoShader 是同一 Auto 语言的 GPU stage 接口，GPU compute/AI kernel 属后续 profile，不是新语言。基础 AutoShader 后，可组合 Shader Components 为首个主打候选；Web/desktop/mobile 目标按支持矩阵逐项验收。首版建议 WGSL/wgpu 纵向切片，其他 GPU/accelerator 后端另立范围。

完整 CAP 在**新版 ACE 架构完成且基础 AutoGraphics 已建立**后进入实现。两项完成均须实际冻结交付和可消费接口，ACE 有限核心骨架、Rust 游戏 driver 或独立 Iced 探针不能自动替代。独立图形研究仍可并行；本次只登记长期设计，不把求解器、全部 shader/compute 后端或 realtime/deterministic 保证加入当前v0.6、829/830验收。

### 近期细化：830与AutoShader设计/实现分开（2026-10-22）

近期继续[830 r2](../../plans/archive/830-autographics-surface-portal-composition-and-input.md)的Surface/Portal、真实Iced嵌套输入与生命周期工作；本轮只修订计划，未开实现。T-02/04/08补当前Shader资源/布局、renderer局部分层和交接，核心S/V/AC分母与G829/迁移门保持，不加入Shader语言、CAP求解、自定义Shader加载/重载或新平台。

现在先完成[AutoShader小范围设计](autoshader-minimal-language-and-renderer-interface.md)。基础AutoGraphics可用手写WGSL建立，不依赖AutoShader完成；待可消费ACE语义/类型/来源接口与基础图形具备后，以Auto→WGSL→现有wgpu的最小纵向切片逐步接入CAP首版，不要求完整CAP全部profile/包系统先完成。之后再做Shader Components与compute等扩面；正式支持面和交付门由后续Plan冻结。完整CAP仍守新版ACE完成与基础AutoGraphics建立的进入门。

## 9. 后续规划：AutoUI 完整组件与默认设计体系（2026-10-24）

用户确认将这两项列入后续规划，详细研究范围统一在 [Design47 §16](../47-autoui-state-design-and-feature-architecture.md#16-autographics-后续完整组件接入与默认设计体系2026-10-24)，[远期 TODO](../../roadmap-future.md)登记候选。当前 Iced 按钮/input 是实验样本，组件树接缝不等于生产组件全量兼容，默认 Iced 外观也不是新的 AutoUI 设计包。

后续以 AutoUI 独立仓迁移完整交付的版本为基面，完整盘点公共组件、组合组件及后端专属项，形成含行为、外观、嵌套输入、弹层、资源/生命周期与宿主服务的兼容矩阵，再分期接入。AutoUI 提供组件/样式，AutoGraphics 提供渲染/组合；避免为图形线复制另一套组件体系。

设计研究包含旧主题/配方兼容、新默认设计包、逻辑 owner 的主题继承、共享 Portal 与 overlay 的 scope、样式优先级及缓存失效；Vue/VM 同源设计语义与真实画面对拍，适用 Rust/RQHost 及其他平台单列证据。默认外观改变须有明确版本/选择和迁移方案，不能静默替换旧应用。

本次只补规划，**840保持原范围，暂不实施上述内容，也不分配后继Plan ID或月份**。2026-10-26当前阶段改按独立研究交付，I2/I3继续未交付并等ACE完成后再讨论，不保留先前作为当前版最低交付的排期；完整组件/设计目标及兼容分母保留。

## 10. 后续规划：Schema 统一声明与ACE/热重载（2026-10-24）

目标是以统一Auto声明方式构建传统UI、2D、3D及混合工程。分层与分期以 [Design45 §11](../45-autographics-wgpu-engine-and-game-prototypes.md#11-目标态schema-统一-ui2d-与-3d-声明语义2026-10-24)为准，状态/设计及迁移约束见 [Design47 §17](../47-autoui-state-design-and-feature-architecture.md#17-schema-统一声明841与ace热重载的衔接2026-10-24)。Schema描述节点契约和受限语法映射，领域分析/lowering和真实运行时补齐布局、变换、输入/资源语义；场景持久化schema与作者文法版本分别管理。

841继续既定领域模型解耦和消费者迁移，后继消费其正式端口，不追加验收。排期更新为等ACE开发完成后再讨论生产Schema/共同HIR方言和ACE适配；未来首例仍核真实增量/依赖/cache与未改函数复用，初期可Stop/Run，保状态HMR另验。普通Auto、AutoUI与AutoShell按Atom＋Schema定义的新版HIR方言体系演进，当前不把旧VM原型视为已迁入HIR。

完整组件/默认设计体系保留原兼容清单和ThemeScope研究；统一Schema不静默砍组件/改外观。840/845及在途851范围保持，当前只更新设计/候选排期；不改canonical或预排编号/月次，I2/I3和生产合入前置按Design45 §12新增ACE完成后的再议门执行。

## 11. 当前执行顺序（2026-10-26 用户确定）

**M1→M2/S0优化→视口/公共游戏能力→独立2D与3D游戏→M2整体验收，Gallery内容按已交付能力并行积累。** S0/S1及860/861已交付，874→875范围保持；Gallery首12项、能力矩阵及后继包见[M2战略](autographics-m2-v06.md)和[Gallery专题](../autographics/m2-gallery-and-capability-matrix.md)。本轮不实施Gallery/建仓。优化保持面和ACE再议门仍见[Design45 §12](../45-autographics-wgpu-engine-and-game-prototypes.md#autographics-stage-851)，不扩大851或提前融合生产AutoUI/ACE。
