# AGE M2三路线统一strategy

2026-11-19，W1 按 1071 原子面和 AutoUI 104–108 剩余 AV 分母验齐，1097 交完整 W2，1074 消费 W2。AGE 逐域交付见下表。1068 接收正式证据并执行最终三端验收。

2026-11-17，target-state v1；**仅规划，未实施/测试/建树/素材/真实模型调用/发布**。本页是三路线的当前统一策略，替代旧M3/M4排期，历史文件名/标签/已交范围保持。架构与任务数据流见[共同设计](../autographics/age-m2-three-route-architecture.md)，引擎详细实现边界见[引擎合同](../../../../auto-ui/docs/design/autographics/age-m2-engine-extension-contract.md)，产品见[AutoForge架构](../../../../auto-forge/docs/design/architecture.md)。980/981 收基础能力，原产品义务按下列计划继续，已交来源沿原身份复用。

## 后继交付与消费

[980 基础](../../plans/archive/980-ace-m2-autoui.md)和[981 基础](../../plans/archive/981-ace-m2-age.md)收口自身的基础接口和当前联合修复。完整 UI 基线按 [1071 AV 子面](../../plans/archive/1071-autoui-w1-av.md)及 AutoUI 104–108 完整 AV 证据 → [1097 AC](../../plans/1097-ace-ac-autoui-w2-and-age-host.md) → [1074 A2R](../../plans/1074-autoui-w3-a2r.md) 签三个独立交付。1068 的最终三端汇总和 985/B 仍接收各后端正式证据。

生产 Vue/Tauri 分别由 [1072](../../plans/1072-autoui-vue-web.md)、[1073](../../plans/1073-autoui-tauri.md) 交付。OS064 消费实际 UI 三波，实际使用的 AGE 能力按下表接对应版本。AG031、1068 和 985/B 各按原合同核收。

| 范围 | AV | AC | A2R 与发行 |
|---|---|---|---|
| AutoUI / Widgets | 1071 U01 ＋ AutoUI 104–108 的 AV/Iced 阶段 | [1097](../../plans/1097-ace-ac-autoui-w2-and-age-host.md) 原生 AC 宿主及完整 W2 | [1074](../../plans/1074-autoui-w3-a2r.md) 交完整 W3 和 Iced 工程接口。982 接该接口联合验真实 CLI |
| Gallery 小 demo 与公共 AGE 场景 | [1075](../../plans/1075-age-av-content-and-fusion.md) G01–G12/原 22 入口和公共会话 | [1102](../../plans/1102-age-ac-gallery-fusion-and-capabilities.md) | [1103](../../plans/1103-age-a2r-gallery-fusion-and-capabilities.md) |
| UI/2D/3D 混合 Showcase | [1099](../../plans/1099-age-av-mixed-showcase.md) | [1102](../../plans/1102-age-ac-gallery-fusion-and-capabilities.md) 对应场景 | [1103](../../plans/1103-age-a2r-gallery-fusion-and-capabilities.md) 对应场景 |
| MAP/ASSET/PBR/CLIP | AutoUI 099–102 各自正式 AV | [1102](../../plans/1102-age-ac-gallery-fusion-and-capabilities.md) 对应能力 | [1103](../../plans/1103-age-a2r-gallery-fusion-and-capabilities.md) 对应能力 |
| 完整 Platformer 2D | [1100](../../plans/1100-age-av-game-2d.md) | [1104](../../plans/1104-age-ac-game-2d.md) | [1076](../../plans/1076-age-a2r-game-2d.md) 交 G2 A2R 结果和独立发行包 |
| 完整 Platformer 3D | [1101](../../plans/1101-age-av-game-3d.md) | [1105](../../plans/1105-age-ac-game-3d.md) | [1077](../../plans/1077-age-a2r-game-3d.md) 交 G3 A2R 结果和独立发行包 |

AGE 内容按小 demo、混合场景、完整游戏安排。两游戏各自等本游戏同源 AV、对应 AC 和实际公共接口。跨计划依赖以正式版本的能力或工程交付为准。1102/1103 分能力发布，未被消费的能力各按自身输入继续。切断一项实际依赖时保留具名未交状态。两个发行计划共用打包器时，公共改动先落主，随后按各自角色独立出包。

迁移对账用 981 归档 §0 和 §5–8 的原行 ID。1075 原 T-02/T-03 与 B-CONTENT/SESSION/LANGUAGE/PROOF/FUSION 的 AV 行按上表分到四个内容交付，公共 AV 源/会话只实现一次。1097 原 T-05..T-08 及 AC-AGE 对应行迁 1102/1104/1105，每包独立复审、合入。1074 原 981 B-RUST 公共 AGE 及失败行迁 1103，980 B-RUST 的 UI 与通用工程行留 1074，原 A2R 游戏行由 1076/1077 完成。982 的 CLI 联验、1068 的证据收取、985/B 的原认证和 AG031 的联合关卡继续沿原要求接收。

原 AG029 国风、AG030 光影等按自身合同验产品，所用的公共能力由 1102/1103 按原能力键交。G2/G3 分别有独立的 AV、AC、A2R 收据和最终 ZIP。

## 1. 三路线、owner和完成谓词

| 路线 | 交付/owner | 当前执行合同与终点 |
|---|---|---|
| R1 生产UI替代 | auto-lang公共语言/后端；auto-ui组件/宿主；auto-os桌面/App | 消费980/981基础接口与1071及后继AV/1097/1074的实际交付：同Schema Node覆盖原ui-examples/Widgets，完整AV→AC→A2R；再OS064全量迁移/984/985 B候选/986安装默认/987旧链退役。不为Hello/widget另开线。 |
| R2 2D/3D引擎增强 | auto-ui公共renderer/typed服务；age-examples内容/独立oracle/联合签章 | 原UI058→AG023→AG029二维首切片；UI080已交→AG030正式光影；新增UI099/100/101/102四扩展AV包，AC/A2R 分别由 1102/1103 接公共引擎格。AG031 r2保原E/G2/G3/F/真实OS/JOINT并收四扩展三端来源后签AGE M2-ready。 |
| R3 新版AutoForge | auto-forge产品工程/编辑/Agent；消费R1/R2公共能力 | Forge001→002→003先交可实际创作App/Game的AV产品，004再AC→A2R输出。独立产品收据，不把旧Forge Agent/纯编辑器冒新产品，也不把Forge反加为985/A或AG031前置。 |

三路线共享一个源语言/Schema/资源/渲染体系，**不是三套引擎**。原E/F是R1执行/融合分轴，G2/G3是R2；R3是独立消费产品，不吞并通用Agent/轻量编辑器或现有AutoUI。

## 2. 已交复用与不重复立项

- M1签章/原游戏/Gallery/编辑器完整原分母保留；UI055/059/AG022的研究、固定r187/621项目录不重做。目录是研究覆盖账，不是621个已支持项或本轮621个实施计划。
- UI080已归档10/10，正式0790cdd7；单方向光/单硬阴影/opaque与有限作者链复用，旧路线的3/10等待记录失效。新增PBR包只补多灯/明确BRDF，不重开080。
- UI006/022/034/043的精灵、有限动画、GLB/模型实例/材质和MeshStore/SpriteStore均是基础，不把它们再建一套。UI040/835/837场景编辑/保存/层级是有限原型，提供来源与oracle；不能直接充新版有业务Node/AI创作产品。
- 085/087/088/091/092/093等通用组件由各原owner交付，Forge先用已交最小原生组件子集，专业停靠/属性/资产库有实际产品需求后消费，不为Forge复制控件。
- 新仓auto-forge当前仅README；旧auto-forge-legacy只读保全，不迁移/删除旧产品，不复活旧ledger/specs.json。新产品模块/协议均在新仓后续实现。

## 3. A/B与三波的唯一调度

| 门/阶段 | 允许/交付 | 不等待/不可冒领 |
|---|---|---|
| ACE M2-A | 沿985现行冻结合同的原基础、MIN-A和正式收据；本轮新增图形扩展/Forge不进入A分母 | 不用图形新特性或Forge完成反卡M2A-08；不把本页当已签A |
| A后 W1-AV | 1071 加 AutoUI 104–108 验完整 UI AV；1075 Gallery 后推进 1099 混合，1100/1101 完整游戏最后；扩展按所用正式 host/Schema 字段推进；Forge001需最小文本/按钮/输入/列表/场景端口与事件payload/参数B接口正式齐 | 不等完整UI-W1才交所用早期子集；UI-W1不等全部新引擎扩展/Forge；原暂停任务不自动恢复 |
| W2-AC | 原 UI-W1 后由 1097 交 UI-W2-AC；1102/1104/1105 各接自身 AV 并交对应域 AC；各新图形包在该收据及自己AV/所用域AC接口齐后补同源AC，Forge004 AC同理 | UI-W2不等全部新扩展AC才签；全AAC自举/1068最终T-06不是前置；AV≠AC |
| W3-A2R | UI-W2后1074真实生成/编译交UI-W3-A2R；新扩展和Forge输出在该基础及自身AC/域A2R接口齐后验证 | AV批次不反复编整套A2R图形；现UI080/旧游戏A2R只复用原证据，不据此认全新功能 |
| AGE联合与产品 | AG031原门＋四扩展AV/AC/A2R独立oracle/真实混合/资源/包全齐→AGE M2-ready；Forge AV产品与三端输出分别独立review/fold | 不等ACE整个M2标签或非AGE生态全部迁完；AGE和Forge不互等最终签章 |
| OS/发行退役 | 原三UI波后OS064全桌面/App→984/985 B→986→987；新增AGE扩展只在原真实consumer实际需要时纳相应产品格 | 不把AG031最终签章倒置为W1/W2/W3或OS最小接口的前置 |

原UI058/AG023/AG029/AG030/AG031仍持完整验收责任；AV可按新增阶段条款先独立R/fold，同一唯一树留后续波，未闭整任务不提前勾。四个**新**UI扩展Plan特意只交中立实现＋AV，完成即归档清树；后续端由 1102 和 1103 分能力接。最终三端责任在AG031精确收据键，不能因新Plan归档丢失AC/A2R。

## 4. 新增引擎包与后续候选

| 包 | 首个正式支持范围 | 明确未包含 |
|---|---|---|
| UI099 MAP | 64×64×2层、16²分块（4空间块×2层）/可见集/同Tick查询与精灵、版本化预算/拒绝/恢复 | 无限世界/逐格VM/完整地图编辑/新物理 |
| UI100 ASSET | 增量静态GLB/.gltf scene/TRS/primitive/material/依赖闭包/共享上传和身份 | 全glTF/压缩/skin/morph/任意格式 |
| UI101 PBR | opaque metallic-roughness选定GGX数学模型、1..4方向灯、唯一shadow caster | IBL/透明/多阴影/完整three.js兼容 |
| UI102 CLIP | 复用已有通用TRS LINEAR/STEP采样；新增统一时钟/播放状态、seek/pause/loop与两clip过渡 | 骨骼/IK/morph/高级物理或动画编辑器 |

骨骼蒙皮、完整刚体/导航、粒子/后处理、更多灯/阴影和跨平台是下一滚动候选，触发条件=真实承载场景＋复用/容量证据＋独立合法/失败oracle；本轮不占号、不伪装已规划全功能。本轮明确将四新增包列AGE M2增强增量required，AG031原全部义务保留并增行接收；不改变M1，不新增985/A前置。所有容量是版本化场景/任务支持域，不把旧8状态/128行/32credits或新样本规模变成永久语言限制；更大规模须先实测/明确合同，不静默抬额。

## 5. AutoForge产品推进

`项目/SourceKey/Revision → 人工或Agent Changeset → 共同compile/verify → 隔离真实预览 → 用户接受 → journal发布/保存/undo → 同源运行/导出`。Agent只提案，产品服务持授权写入；模型自报完成、mock响应或静态截图均不能签产品。

第一批Forge001/002/003分别可AV整包交付/清树，需有真实模型一次成功/失败修复、真实App与Game旅程。004先AC phase、再A2R，原三波/域端口为实际前置；不重建编译器/SDK/打包系统。生产组件子集可先行，不要求大地图/PBR全部做完才能有Forge。

## 6. 执行顺序、写面与验收

A先由原owner收口；本次只规划。之后优先R1 AV最小编辑组件与事件接口，同时UI058/AG023 AV、AG029原产品及AG030内容；四扩展依赖为MAP←058，ASSET←043，PBR←080＋ASSET，CLIP←006/034＋ASSET。Forge001随所需UI/源/预览接口齐进入；002随001；003随002＋058/AG023 AV；004随W2/W3与域端口。共享grammar/Schema/FFI/renderer按短写窗，不让产品反等其自身终验；GPU/音频/原生输入重门单实例。

每行交`source/asset hash＋Schema/profile＋SDK/host ABI＋backend＋platform/toolchain＋真实command/actual＋独立oracle＋错误/资源/恢复＋正式repo/SHA`。结构检查和“peer-equal”不能代行为。AV/AC/A2R业务语义共享、各端结果独立；AC无AV解释回退、A2R真rustc；零选择/工具缺失/未运行保持明确状态。每计划仅受影响用例，里程碑全量由AG031原安排单独执行，不每次合并40分钟GUI。

## 7. 可执行阶段顺序与并行窗口

以下为阶段DAG，不把frontmatter最终owner并集当开工AND门；每条边指**实际所用、已独立复核并正式合入**的子面，不指在途分支自报。985/A仍按自身冻结分母完成，本页不裁定它已可签。

| 波次/窗口 | 建议顺序 | 可并行/阶段结束 |
|---|---|---|
| A收口 | 原A-required→985 T-A03→独立T-A04→T-A05正式M2A-08 | 当前规划不启动任何work；中立设计/既有准备权限按原合同，不自动恢复AG029 |
| B-AV优先 | 981 EVENT-B/源/资源、980 最小组件→1071 及 AutoUI 104–108；AGE 依次 1075→1099→1100/1101 | UI058 AV→AG023 AV→AG029 AV；080已交→AG030 AV；UI100 AV可与058/030并行；UI099 AV等058 AV；UI101与102等100 AV后可按renderer写面短窗并行。它们不等W1全交，但均需各自实际typed接口 |
| Forge首批AV | 所用早期AV编辑组件/事件/源描述齐→Forge001→002→003；003另等058/AG023 AV | 001/002不等完整W1或新四cap；003不等AG029全游戏/地图/PBR。前三包各自完整review/落主/归档清树 |
| B-AC | 1071 U01＋AutoUI 104–108 AV/Iced→1097→UI-W2-AC | 1102、1104、1105 在对应 AV＋W2＋最小域接口后分别交 AC；Forge004 AC只等003、W2及实际G2 AC，不等其未使用的新cap |
| B-A2R | 原UI-W2-AC→1074第三波→UI-W3-A2R | 1103、1076、1077 在对应 AC＋W3＋实际所用 A2R 端口后真实 rustc/运行；Forge004 A2R等自身AC阶段及实际G2 A2R。原已有A2R证据复用，不反复重编 |
| 产品/联合收口 | W3后OS064全量迁移→984/985 B候选→986安装→987旧链退役；AG031原门＋四cap三端＋真实OS齐后独立签AGE | AGE不等Forge总产品；Forge不等AG031最终签；985 B不反等986/987完成后才给候选。实际安装/默认/退役门不被候选或AGE-ready冒领 |

新增计划均为草案，进入各 T-01 前核本计划实际接口与实施安排。所有新增API/fixture/包型均为拟定，实际最低负载不成立走needs_replan，保原样本与预期，不悄悄改功能/抬预算。
