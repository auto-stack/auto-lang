---
plan_id: PLAN-834
completion_kind: delivered
status: archived
feature_name: 真实Auto驱动Squash与双游戏SDK闭环
plan_revision: 1
author: [Codex]
created_at: 2026-10-22
updated_at: 2026-10-23
current_step: 9
total_steps: 10
depends_on: [PLAN-833]
supersedes_spec_components:
  - docs/specs/auto-graphics/auto-source-driver.md
  - docs/specs/auto-graphics/integration-contract.md
  - docs/specs/auto-graphics/prototype-profile.md
  - docs/specs/auto-graphics/project.md
new_spec_components:
  - docs/specs/auto-graphics/auto-3d-game-profile.md
touched_goals: [GOAL-007, GOAL-008]
affects: [auto-graphics, language-sdk-consumer]
---

# [PLAN-834] 真实 Auto 驱动 Squash 与双游戏 SDK 闭环

## 0. 变更摘要

**2026-10-23 交付已落autographics-dev，规范/账本已沉淀；归档9/10，T-10清理待完成。** 独立R1 PASS与完整合入门禁通过，交付提交`19adeebf1b1031554c71acabaa219731b3a28b96`。master仅知识文档，I1实验双游戏分母交付，I2/I3/Shader/CAP/生产迁移仍pending。[最终合入收据](../attachments/834/merge-r1/receipt.md)。

源、依赖、样本、容差和生产边界不变；完整门与真实失败/重放、投影/归档/清理实际checkpoint见§9最新merge收据。历史work/review的pending描述保留为阶段记录，不表示当前门禁仍未执行。

## 1. 目标

1. 通过833交付的公开SDK/AutoVM桥，持续执行真实 `.at` Squash玩法：移动/跳跃/重力、敌人生成与运动、碰撞/踩踏、计分/死亡/停止生成、越屏释放与retry/reset。
2. 消费原player/mob GLB、原场景/相机/光照/阴影、场景级float动画、HUD/Retry及原循环音乐，真实绘制与Auto状态同源。
3. 将833只含circle/quad/cylinder和Breakout事件的有限协议扩为明确分型的3D数据合同，保持旧profile及Breakout行为，错误在使用前拒绝、失败保last-good。
4. 120Hz固定物理与渲染/挂载解耦；真实窗口输入、焦点、单次jump边沿、双视图共享/独立实例、retry与关闭的寿命/事件/音频可复跑。
5. 复用原独立golden、图像/音频资产与故障敏感性验证，证明不是调用Rust SquashDriver代跑，也不是Auto只控制一个几何体。

支持profile：Windows x64、单原生窗口、键鼠、833最终wgpu/Iced/SDK版本线；复用829/830冻结Squash的受限实现语义。原single-pass move_and_slide近似及屏幕退出判据沿用已有profile，不承诺完整Godot物理兼容。手柄/右键jump仍按原已记录差异不纳入，不新增平台或Portal几何面。

非目标：修改SDK/VM/语言语法、修通用数学转换、全局graphics native目录、通用物理/动画编辑器/骨骼、任意模型/用户shader、第二渲染后端、生产移仓/默认入口/发行、热重载、安全沙箱、联网/XR。复用有限场景动画不等于新增完整动画系统。

## 2. 架构方案

```text
实际窗口输入 / 固定120Hz时钟 / 有界确定性数值服务
                         ↓
         833公开SDK桥 → Auto Squash状态与玩法
                         ↓
       分型输出 → 整包准入 / session与资源身份
                         ↓
    Scene / 原GLB与camera / 830 SurfaceRuntime → wgpu
                         ↓
       HUD/Retry / 原背景音乐 / 实际呈现与事件证据
```

不复制SDK、parser或语言桥。继续使用 `experimental/autographics-auto-driver/` 作为语言消费方，AGP `experimental/autographics-prototype/` 提供现有图形/资源。VM类型不进入AGP core；GPU对象/资产加载留host。

### 2.1 前置门、授权与唯一树

| 门 | 成立条件 | 当前状态/责任 |
|---|---|---|
| G833 | 最终review pass、冻结merge-owned门通过、实际合入autographics-dev、对应SD/交接可消费 | 已满足；833 archived/delivered 10/10，实际交付540c1e70f，旧树已清 |
| GSOURCE | G833后的driver/AGP/SDK完整SHA、两份依赖锁/源锁/build输入及资产身份重核并冻结；若833未变SDK则继承其已发布精确版本 | 已固定540c1e70f/SDK9576b4dad及原锁/资产；见research/plan834-source.json，独立消费在T-07收口 |
| GSTART | 用户启动834 | 2026-10-23用户明确授权实施；已创建唯一树，无agent/新聊天 |

观察：833 R1受审候选 `b136129fed9a018f07ea49445868066449eaf5e9`，当次driver锁 `f01c231218b2f1534f971df128b9d188fa19648e5a4399dbe81ce37e5386063d`；autographics-dev仍为830交付 `764e87559cff016d9e0d6b2141e5f1f9903398c5`。候选仅用于只读定位，不是834实施基线。源码对象/hash见[观察清单](../attachments/834/planning/source-observation.json.txt)。

未来唯一树 `D:/autostack/.wt/lang-834/auto-lang`、分支 `plan-834-dev`，从通过G833的autographics-dev完整SHA明确创建；不要用默认master基面的捷径。work/internal check/final review同树顺序执行，主检出维护Plan。开工必须读主检出当前修订，历史基面未必含当前合同。审毕仍合autographics-dev，master只沉淀知识，原生产迁移门不变。

## 3. 技术栈

复用833已交付的 `Session/Scalar/Script/Publication/SceneConsumer` 或其最终等价公开符号、SDK源码载体工具与锁；无依赖升级。继承AGP GLB加载、MeshStore/MeshPipeline、相机/阴影、TextRenderer、Kira及Iced输入。必要新接口在两个experimental工作面内实现，不写crates/、根manifest/锁、Man/CLI或其他仓。

新增Auto游戏源码与有界3Dprofile，不新增一套语言/场景引擎。拟新增的API/CLI参数/测试target由实施任务建立，本文未运行它们。

## 4. 需求分析与背景调查

### 4.1 授权与来源

2026-10-22用户说明833实施完毕、处于review/修复，要求提前规划下一计划。授权范围为调查和Plan草案，不含启动实现/另开agent/修改833/合入。未指定实施、内部检查、最终review人或模型；不编造角色身份。

从[overview](../../specs/overview.md)、[SDK](../../specs/language-sdk/project.md)、[图形project](../../specs/auto-graphics/project.md)/[profile](../../specs/auto-graphics/prototype-profile.md)/[交接](../../specs/auto-graphics/integration-contract.md)、Design45（规划时背景笔记，主检出既有未跟踪文档；正式知识见上述canonical图形规范）取材。[833合同](833-autographics-auto-source-minimal-loop.md)与[R1独立报告](../attachments/833/review-r1/review.md)用于判断依赖；不以work自检代替交付。

当前canonical尚无833的 `auto-source-driver.md`；这是833拟交付来源，834 frontmatter中的modify为provisional，必须等G833后重核，不能从候选SD提前发布知识。设计中旧状态标题保留历史意义，以实际收据为准。

### 4.2 已观察事实与缺口

| 事实（候选仅规划证据） | 后继所需工作 |
|---|---|
| 833使用保留VM/Task、公开CPU slices、标量callbacks/queries和changed(slot)，已实现Auto Breakout候选 | 复用机制，不再造语言桥；其最终版本/预算等G833交付后重核 |
| 候选Protocol 1只有平移/缩放/RGBA、shape 0/1/2，事件校验绑定Breakout碰撞/消砖 | 明确新增3D profile/schema；不能把mob当brick或偷偷改变旧事件范围 |
| 当前consumer转换不带旋转，3D只显示已有cylinder | 补固定GLB资源身份、完整变换/层级动画与实际mesh消费，不用立方体/cylinder替代原样本 |
| Squash为120Hz，输入包含两个方向、jump边沿、retry/dead状态；833为60Hz且tick带单方向 | 让profile决定步长/动作，保持旧Breakout60Hz；不把一次渲染当物理tick |
| 833已暴露SDK sqrt/float转换缺口，用有限Auto f32算法满足原golden | 3D atan2/sin/cos/sqrt、旋转与随机流必须先有界实证，不推断2D有限算法能覆盖整个3D域 |
| 原Squash sim/contract有完整冻结轨迹，live输入及原模型/声音已存在 | 复用资源/独立oracle，Rust sim仅对照，不驱动Auto模式 |
| 833 R1 TextOverlay归属/上传计量未闭合 | 留833修复，G833后消费；834新增3D/音乐资源仍须同类完整owner/计量验证 |

### 4.3 冻结样本分母（文件盘点，不是本轮执行结果）

来源沿用Godot `b761b4cd5718a7118f2c95eb55f181d0a5f004f1` 的 `3d/squash_the_creeps/`，清单为AGP `research/sample-manifest.md` §2及其evidence-notes，源码副本在 `testdata/squash_the_creeps/`，资产在 `assets/squash_the_creeps/`。

现有 `testdata/squash/golden_squash_trace.json`：seed27、f32(1/120)、3600 actions/trace ticks、45 spawns、91 events（spawn45/despawn42/squash3/death1）、原逐tick位置容差0.01，最终score3/dead。沿用 `tests/squash_contract.rs` 对spawn位置/yaw的1e-4和velocity的1e-3等现有阈值；事件类型/顺序/身份/计分精确，浮点字段按原合同比较，不把“精确事件”擅自扩成所有浮点bitwise。该轨迹最多同时7个mob是**此样本观察**，不是引擎容量上限；T-02还要覆盖实时更长运行的资源预算。

必须继承：spawn y=0、敌人朝出生时玩家实际位置加jitter、直线运动/越屏释放；body踩踏与MobDetector死亡双通道、每次最多一个踩踏得分；角色朝向/pitch及场景级1.2s bob动画。GLB本身没有骨骼动画；不能用“GLB无动画”省略场景动画。唯一音频是原循环 `House In a Forest Loop.ogg`；不添加不存在的踩踏/死亡音效作为复刻证明。

## 5. 详细设计

### 5.1 执行入口与T-02 whole-path决策

G833/GSOURCE/GSTART满足后，T-01读主检出本合同，确认唯一树/分支/固定source/锁/资产与上游零开放合同finding。只重核冻结来源，不追随最新tip；833修复不能转成834债务。T-01准备已有SDK载体，T-02的scratch fixture/最小manifest均在834树内，不依赖T-03尚未实现的正式3D协议。

T-02产 `research/squash-driver-feasibility.md`（新），用真实Auto程序→已有Session→参数/结果→现有GLB最小绘制验证：

- 所用3D数学/归一化/角度、旋转和场景动画的f32路径、四象限/零方向/边界输入；与冻结参考/实际矩阵比较，有限算法注明适用域，不认领通用SDK修复。
- 确定性xorshift64*随机流及调用顺序：最多比较“纯Auto实现”和“显式有界host原始数值服务”两接法。host只可提供原始seeded随机数/通用数学值，Auto负责何时生成、选位置/速度/目标、碰撞/胜负；不能用Rust SquashDriver给出预计算游戏结果。选择host服务时，协议必须记录消费序号/数量、session与重置语义，候选数据/失败不能暗中多推进随机流。
- 一个mob生成→Auto变换/动画→资源准入→原模型渲染、真实tick预算/队列开销，及错误输入/失败保旧状态对照。

T-02冻结当前样本与实时profile的容量/命令/队列/CPU预算及拒绝分类，保留UI有界pump。原Profile1预算不变；新profile预算选择须有样本峰值与病理控制实测，不承诺无限资源，不用任意语法深度限制代替支持。

两接法均无法经已发布SDK满足原数值/事件、真实3D消费和有界调度时，记录具体SDK/renderer缺口与owner，T-03之后不开展，保持未完成回new；不改SDK/扩大写面、放宽golden或切Rust代跑。这里的探针尚未执行。

### 5.2 有限3D输出合同与旧profile兼容

复用833解析/执行/准入机制，新增分型3D数据。以下为概念字段，正式根函数/字段编号由T-02/03写进可审schema与README，不假称现有符号。

| 规则/数据 | producer→consumer与必须成立的结果 |
|---|---|
| profile/schema | Auto导出→bridge选择明确profile/版本；Profile1原字段/事件不变，新增Squash版本独立校验；未知/错配在消费前拒绝，不根据内容形状猜测fallback |
| 3D对象 | Auto状态→受检snapshot→Scene/mesh；slot+generation/session、资源key、translation/rotation/scale、显式可见/删除；所有分量有限、资源/旋转表示合法，原模型多mesh/子变换不丢 |
| 资源 | host固定资源清单→有代际/owner的句柄；仅原player/mob/ground/场景已有资源与HUD/音乐，不接受任意路径/指针；加载失败/错种类/旧代/跨设备在GPU工作前具名拒绝 |
| 旋转与动画 | Auto负责朝向、pitch/bob时间与游戏相关状态→通用矩阵/mesh绘制；物理碰撞态与动画局部偏移分清，旋转/scale不被consumer丢弃；可复用纯数学插值/矩阵运算，不让renderer根据角色标签重新执行玩法 |
| 状态/事件 | Auto产生spawn/squash/despawn/death、score/alive/retry；准入检查来源身份/顺序/唯一性/先后状态，render和HUD用同一接受snapshot；不能复用Breakout contact→sound配对假设 |
| 发布/失败 | 先完整解码/校验/准备所需资源再提交，last-good包/scene/HUD/音频保持一致；坏包不能留下半spawn/半score或外放一次音乐；事件必须每次callback新生，非tick输入不能重放旧事件 |

保持对象身份不以数组位置猜player/mob：有显式受检资源/角色映射，销毁/slot复用后不变成另一模型。既有Rust projection可作等价对照，Auto模式不得进入 `SquashDriver::tick`、`spawn_mob` 或Rust玩法状态机。

### 5.3 Auto玩法、呈现、音乐与输入寿命

- Auto拥有场景初始状态、生成时钟/停止条件、玩家输入/速度/重力/jump、敌人运动/寿命、两通道碰撞、计分/死亡、retry状态；现有受限物理近似按原oracle继承，不新承诺通用物理。
- 复用原GLB、材质/节点烘焙、相机/光照/阴影和背景；由Auto变换决定角色位置/朝向/pitch/bob。score与Retry由同一次接受输出驱动真实文本/overlay；不预写固定score、死亡帧或CPU截图替代活体画面。
- 原音乐作为宿主级autoload资源：Squash宿主启动一次循环，retry不重启/叠播，共享多个Portal不多播，关闭最后宿主owner后停止并释放。独立脚本实例各有玩法状态，音乐按单窗口宿主共享规则，不把脚本reset当整宿主退出。无新squash/death SFX。
- 真实窗口→830路由surface/local/唯一焦点→动作→Auto。左右/前后/WASD，Space alive仅just-pressed jump，dead时Space/Enter retry且不把旧jump带到新会话；文本控件/菜单持焦点不驱动游戏，blur/卸载清held键和未消费边沿。
- 每个Squash实例按120Hz固定tick一次，渲染/共享视图数不影响模拟或生成/音乐；120Hz与833 Breakout60Hz由profile选择，不能更改旧默认。暂停/隐藏策略继承上游既定profile并记时钟恢复规则，不以追赶巨量tick卡住事件循环。
- reset/retry生成新脚本session/句柄，旧输入/事件/随机服务响应不得进入新实例；close取消未完成slice/输出并释放Task/VM/桥/mesh引用/音频owner。外部合法pin在最后释放前可保活，须如实区分，不靠清计数假称资源已drop。

### 5.4 双游戏SDK闭环与规范增量

最终证据包含两款由真实Auto驱动的冻结游戏；原Rust双游戏、830组合/输入和833全部适用能力保持。根据实际runtime/平台/profile、公开SDK符号/来源与双游戏证据逐项核I1；只在最终review/merge证据齐备后更新其完成状态。I2生产AutoUI/RQHost迁移与I3 Forge/发行仍pending，不认领CAP进入门或完整新版ACE。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-graphics/auto-source-driver.md | 833单游戏有限协议→旧profile保持、新Squash动作/时钟/3D对象/事件与准入/寿命 | 复用单一SDK桥 | AC-02,03,05,06 |
| SD-02 | add | docs/specs/auto-graphics/auto-3d-game-profile.md | 新增Auto Squash实际玩法/数值/RNG/模型动画/音频/失败/容量及实测边界 | 首个完整Auto 3D游戏 | AC-02,03,04,05,06 |
| SD-03 | modify | docs/specs/auto-graphics/integration-contract.md | I1首片→双冻结游戏按原分母出证/审定；I2/I3及Shader/CAP保持pending | 防实验外推 | AC-07 |
| SD-04 | modify | docs/specs/auto-graphics/project.md, docs/specs/auto-graphics/prototype-profile.md | 增加真实Auto3D入口/资源/120Hz与实测，保原829/830/833能力和限制 | 当前事实沉淀 | AC-01,04,06,07 |

modify以833未来实际交付版本为来源。该文件尚无canonical时明确缺席，不从候选复制为current Spec；merge时模块plans.md/ledger/index为常规关联簿记。

## 6. 测试设计

下表保留规划验证合同；本轮work已完成V-01..09，实际命令、断言、正反控、版本绑定见[readiness](../attachments/834/work-r1/readiness.md)。V-01..09 owner_stage=work，各fixture由task建立，具体新target/CLI由README公布；不得把零选择测试/skip当通过。独立最终review在同一冻结树逐AC/SD核验，自检不代替final review；未建agent。

| ID | AC / invariant | 入口/输入与使其可运行的task | 可观察oracle/反控与边界 |
|---|---|---|---|
| V-01 | AC-01来源/旧兼容 | T-01/07，G833源/锁/SD→真实prepare/verify/package；旧Profile1与Breakout全部适用验收 | 来源/闭包/资产hash明确，fresh+locked A→B无producer/target；旧协议行为/原golden不变；坏source/hash/锁拒绝 |
| V-02 | AC-02数学/RNG/3D whole-path | T-02真实Auto→Session→状态/值→最小GLB渲染；四象限、零方向、有限数值/随机流边界 | 与冻结参考数值/矩阵/随机序列比，合法修改Auto角度导致变换/像素变化；错序/错误数值/失败不会暗推随机流；不证明通用数学库已修 |
| V-03 | AC-03分型准入/资源 | T-03真实producer packet→publication→scene/资源；旧/新/未知profile、错kind/代际/device、坏旋转/混合好坏包 | 合法对照接受；使用前具名拒绝，scene/HUD/音频/last-good hash不半变；新旧profile隔离；完整内容每个GPU资源归属检查 |
| V-04 | AC-04完整玩法 | T-04真实.at在固定seed/输入下跑原3600tick/golden；另补spawn对当前玩家、独立死亡通道/旋转、jump/retry合法对照 | 45spawn/91事件的种类/顺序/身份/计分精确；逐tick位置0.01、spawn位置/yaw1e-4与velocity1e-3等按原合同；score/alive/敌数一致。改Auto重力/碰撞/停止生成使同一门红，不可Rust代跑/硬编码trace；保继承物理近似 |
| V-05 | AC-04视觉/音乐 | T-05已接受Auto状态→原GLB/材质/相机/阴影/动画/HUD→capture及Kira | 踩踏/死亡/Retry关键帧与原冻结图像oracle比较；旋转/bob-only变化实际入矩阵和画面；原music hash/播放/loop真实，retry/多挂载不新增play，关宿主停止；禁音/漏模型/丢旋转反控使正式门红 |
| V-06 | AC-05输入/时钟 | T-06真实窗口/830实际路由→Auto；held/release/repeat、text focus、blur、dead Space/Enter retry、两种渲染节奏/共享两挂载 | 单press只跳一次、无输入不自跳，retry首tick无旧jump；真实目的地/状态和计数可核，120Hz不随view/render改变；错焦点/重复投递/重复tick变异必红 |
| V-07 | AC-06错误/寿命 | T-03/06有效后坏包/资源失败/容量超限、长Auto计算真实Runnable、reset/retry20次、旧响应/close/外部pin | UI有界pump/超预算分类，保last-good；无旧事件串入，最后owner/Task/VM/对象/音乐释放；同设备接受/跨设备拒绝；实际drop替代计数清零 |
| V-08 | AC-06计量/稳定性 | T-07原profile场景与Squash cold/warm、单/双挂载/实例、20次retry，真实buffer/texture/文字/mesh/音频资源出口 | 分项与总量一致、漏计文字/3D上传使正式门失败；CPU P50/P95、积压/tick欠账、工作集/owned资源真实，GPU时间/驱动驻留不可用写原因；warm不重复载模型/开音乐 |
| V-09 | AC-07双游戏与SD | T-07/08实际源→SDK/bridge→渲染链/独立消费；Breakout和Squash各自真实运行 | 公开SDK/源/profile/测试/限制齐备，I1按原分母核、I2/I3不置done；两游戏无Rust sim代跑、无改golden/放宽容差；SD与实际一致 |

开发仅在未来834树按新测试target做定向验收/反控与 `cargo check --manifest-path experimental/autographics-auto-driver/Cargo.toml --locked --all-targets` / 两workspace fmt。全量driver test/build、AGP适用完整兼容/六surface/真实窗口资源门由最终review pass后的merge冻结提交统一跑；定向验收不可后置。

仅experimental Rust，不跑根cargo t/tv/tt/tb/tu/taa/docs_gen；若需crates/变更先重规划。tf按主检出到期单实例，不在本树私跑重型档。

## 7. 验收标准

| ID | owner_stage | 可观察完成条件 |
|---|---|---|
| AC-01 | work | G833/GSOURCE固定，纯SDK/driver/AGP/资产闭包可追，V-01旧profile/Breakout兼容及A→B成功；零生产写面 |
| AC-02 | work | T-02真实Auto数学/RNG/状态→GLB whole-path正反控V-02通过，数值/服务边界/容量/预算明确；未修SDK或偷降标准 |
| AC-03 | work | 新3D profile保持旧语义，模型/旋转/事件/资源整包准入、错版本/错归属/失败事务V-03通过 |
| AC-04 | work | Auto拥有完整Squash玩法，V-04原golden与补充反例、V-05原音画/动画/HUD/Retry闭合；Rust只作服务/呈现/对照 |
| AC-05 | work | V-06真实窗口输入/焦点/just-pressed/retry与120Hz、共享单tick/实例隔离成立，正式门消费实际路由输出并对变异红 |
| AC-06 | work | V-07/08失败保旧、预算/取消/20retry/最后owner、精确计量与稳态测量成立，继承833修复不退化 |
| AC-07 | work | V-09两款真实Auto游戏通过各自分母，SD-01..04全文可审、实际符号/来源/测量/限制齐全，生产门不变 |
| AC-08 | review | 独立逐AC/SD与全路径/反控/遗漏/未授权延期/workaround审查，冻结SHA上无开放合同finding，merge门pending明确 |
| AC-09 | merge | review冻结提交的完整driver/AGP适用门通过，合autographics-dev、Spec/索引/ledger沉淀、归档与guard clean清理，真实10/10 |

## 8. 执行步骤

用户已授权work并请求检查，T-01..09完成；以下路径为合同中的建议布局，实际等价布局为src/squash.rs、src/squash_host.rs及tests/squash_profile.rs等，未改变验收语义；逐任务证据见§9、readiness与final R1。T-10由merge角色在冻结提交上执行。

| 完成 | Task | owner_stage | 依赖 | 已核路径/拟新增产出与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | G833/GSOURCE/GSTART | 核833交付、最终driver/SDK锁与Spec，明确树/源；准备已交付SDK，复用tools/prepare_sdk.py/verify_source.py，建立V-01来源图 | AC-01 |
| [x] | T-02 | work | T-01 | 新scratch fixture/最小manifest与research/squash-driver-feasibility.md；833 Session、AGP rng/collision/anim/mesh仅参考/服务；有界两接法whole-path V-02，冻结预算，失败停止依赖任务 | AC-02 |
| [x] | T-03 | work | T-02 pass | driver src/{script,output,consumer}.rs扩明确3D profile/资源/旋转/事件/准入，不改旧Profile1；必要AGP纯数据出口；新3D admission tests，V-03/07 | AC-03,06 |
| [x] | T-04 | work | T-03 | 新samples/squash/*.at及tests/squash_replay.rs（拟新增）；参考AGP games/squash/sim.rs/frozen源码与trace，玩法在Auto；V-04与真实producer准入反例，不改原sim/golden | AC-04 |
| [x] | T-05 | work | T-04 | driver host/consumer与AGP assets/render3d/TextOverlay/SurfaceRuntime公开接缝；原GLB/场景动画/HUD/Retry/music；V-05实际像素/声音/身份反控 | AC-04 |
| [x] | T-06 | work | T-05 | driver input/clock/lifecycle，参考AGP games/squash/live.rs/830 CompositionInput；V-06/07真路由、120Hz单tick、取消/retry20次/最后owner | AC-05,06 |
| [x] | T-07 | work | T-06 | V-01/08/09双游戏/旧profile兼容、独立包A→B/资源与精确计量/有限负控；冻结测量、永久反例与root collateral | AC-01,06,07 |
| [x] | T-08 | work | T-07 | README/源码样例/研究交接及SD-01..04拟议全文、真实命令/符号/限制/I1分母与债务归属，自检提交冻结→review | AC-07 |
| [x] | T-09 | review | T-08 | 同树独立final R1 PASS，逐AC/SD/冻结版本与正反控、遗漏/延期扫描通过；merge门仍pending，见attachments/834/review-r1/review.md | AC-08 |
| [ ] | T-10 | merge | T-09 pass | 冻结提交统一driver全测/build+AGP适用兼容/六surface/真实音画输入/资源反控门；合autographics-dev、知识沉淀/归档、wt-guard clean清树 | AC-09 |

## 9. 复审记录

### 2026-10-23 merge T-10：门禁全部通过，已合入/沉淀，归档清理阶段

- stage: merge | key: PLAN-834:r1 | outcome: in_progress（cleanup-owned） | reviewed_commit: ebeab12d2888b9c9fb7e00e3c24479fb3c028011 | delivery_commit: 19adeebf1b1031554c71acabaa219731b3a28b96 | integration_ref: autographics-dev（无force快进，未重写SHAs） | reviewed implementation: unchanged。
- driver42/AGP162/C独立消费42完整测试及构建pass，零failed/ignored；16/16 AGP探针/六surface/实际OS IME、Iced counter+content两门及漏画/透明反控、双游戏真实窗口输入正负控、原音画/资源/寿命/计量门全部通过。Breakout160frames/161ticks、6请求=6实际音效；Squash独立13事件正控phase9/music1/errors空、focus负控同oracle具名失败。
- 历史锁屏/预算拒绝及一次15事件额外Space输入现场保留；解锁/无其他探针+键未按住条件下原正式门通过，无实现/SDK/配置/容差修改或豁免，额外事件来源不杜撰。见[完整最终收据](../attachments/834/merge-r1/receipt.md)、[门禁摘要](../attachments/834/merge-r1/gates-summary.json.txt)。
- canonical五组件+module overview/plans、GOAL-007/008、global overview/INDEX已投影master，仅知识；I1原实验双游戏分母交付，I2/I3/Shader/CAP/生产迁移仍pending。账本guarded offline projection v70→71，8项upsert/1138无关raw items保持，[投影收据](../attachments/834/merge-r1/ledger-projection.json.txt)。
- landed/ledger_refreshed已核；计划归档后先核实际链接/计数/版本控制，再强制Git Bash wt-guard clean清理唯一树、merged branch及生成包B/C，最后补T-10与10/10。当前9/10仅cleanup pending。


### 2026-10-23 merge T-10：环境阻塞交接，保留reviewed9/10

- stage: merge | key: PLAN-834:r1 | outcome: blocked | reviewed_commit: ebeab12d2888b9c9fb7e00e3c24479fb3c028011 | implementation/dependencies/configuration: unchanged | worktree: D:/autostack/.wt/lang-834/auto-lang | branch: plan-834-dev | integration_ref: autographics-dev仍540c1e70f0a4682af245191ffb36fa0d8a636665。
- [完整T-10收据/待办与续跑](../attachments/834/merge-r1/receipt.md)、[门禁摘要](../attachments/834/merge-r1/gates-summary.json.txt)。driver42/AGP162/C消费包42完整测试及构建通过，零fail/ignored；12/16 AGP探针通过，含独立源码导出/locked release/A-B29字段零差异。
- blocked：两次Iced真实点击有界超时；实测前台Windows默认锁屏界面，SetForegroundWindow=False。需解锁并保持桌面解锁，补真实Iced/六surface剩余window/IME与两游戏窗口正负控。另原生无输入Breakout160帧触发callback budget exhausted，须在解锁条件重跑归因，持续则回work/受影响复审，不能豁免。
- 未执行合入、canonical发布、ledger投影、归档、工作树/生成包删除；T-10/AC-09仍pending，status reviewed/current_step9不变。review通过不抵扣未通过合入门；保留冻结树、分支与全部正负/失败证据。next_responsible_role: merge，用户解锁后继续实际门禁，不重复已通过全测。


### 2026-10-23 merge T-10：锁屏输入前置阻塞，其他门继续

- 完整driver42/AGP162测试与构建通过；8个基础AGP运行门及surface-compose/scenarios/lifecycle通过。iced-shared-device初次与诊断重跑均只有初始截图，click未观察到；首轮和诊断报告全部保留，没有把超时当通过。
- [桌面诊断](../attachments/834/merge-r1/desktop-diagnosis.txt)实测前台为Windows默认锁屏界面；一次实测SetForegroundWindow=False，真实OS投递未进入probe。初始图像按钮位置、实际192 DPI与队列投递返回也保留，未修改冻结源码/容差/判定。已请求用户解锁并保持桌面解锁，待前置恢复后重跑同一正式门及真实窗口输入/IME。
- 独立AGP源码搬迁构建继续；未合入/沉淀/归档/清树，T-10仍pending。

### 2026-10-23 merge T-10：用户授权，冻结门禁进行中

- stage: merge | key: PLAN-834:r1 | outcome: in_progress | reviewed_commit: ebeab12d2888b9c9fb7e00e3c24479fb3c028011 | final_review: R1 pass，当前revision1/依赖/Spec输入一致 | worktree: D:/autostack/.wt/lang-834/auto-lang | branch: plan-834-dev | integration_ref: autographics-dev。
- 用户授权完整T-10；源码只合autographics-dev，master仅沉淀知识与计划。两独立实验workspace完整driver42/AGP162测试与构建通过（零failed/ignored）；AGP16探针/六surface/真实窗口合成与输入音画反控进行中，尚未合入。根cargo整档不适用原约定experimental-only写面；保留全部其他会话预存docs WIP。
- [准备收据](../attachments/834/merge-r1/prepared.json.txt)、[门禁执行记录](../attachments/834/merge-r1/gate-exits.txt)。独立review pass不能抵扣完整运行门；T-10保持pending直到合入、知识投影、归档与guard清理实际完成。


### 2026-10-23 final R1：独立复审PASS，9/10

- stage: review | review_scope: final | reviewer_identity: Codex当前独立复审聊天（从work交接重建证据，未参与实施；未创建额外agent，不编造model/session ID） | plan_id: PLAN-834 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-834/auto-lang | branch: plan-834-dev | reviewed_commit: ebeab12d2888b9c9fb7e00e3c24479fb3c028011 | base_commit: 540c1e70f0a4682af245191ffb36fa0d8a636665 | dependency_revisions: SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9、两workspace冻结锁/source-lock（hash见baseline）。
- [独立报告/逐AC任务与实际oracle](../attachments/834/review-r1/review.md)、[基线/Spec输入/依赖hash](../attachments/834/review-r1/baseline.json)、[冻结拟议Spec全文差异](../attachments/834/review-r1/spec-delta-reviewed.diff)。AC-01..08 pass，AC-09 merge-owned pending；T-09勾选，status reviewed / current_step 9 of 10。frontmatter四modify/一add与GOAL-007/008已核；未发布canonical或ledger。
- 独立重放27项定向测试（18 staged/numeric/input+2 GPU/audio+7 math/旧兼容，零fail/ignored）、两个组合holdout组；原Squash3600tick/45spawn/91event与Breakout10800tick/210contact不变，五帧原图像及模型/阴影/Auto动画/物理负控通过。实际OGG25.195s wrap、20retry/真实末owner、137124B/16writes与漏计反控通过。
- 当前冻结入口重建后真实窗口正控phase9/exit0/passed=true，错误焦点负控同oracle phase9/exit1/passed=false，面板均为`z `且errors空。A与保留独立C分别verify 680 SDK/180 authored hash/17 local packages/29 edges/479 metadata closure；36项work证据hash复核。原隔离冷构建/producer不可用操作使用同源码/依赖/配置身份下的冻结原始证据，未冒称本轮重做。
- 两workspace all-target check/fmt及diff whitespace通过，无本次新增源码warning；未变SDK146条core+1条shim-metadata警告如实披露。遗漏/延期/workaround扫描无开放合同finding或新债务候选；有限Auto数学属于原授权路线，通用SDK债务仍归既有owner。入场/出场实施树clean且HEAD不漂移。
- pending_closeout_gates: T-10统一完整driver test/build、AGP适用兼容/六surface/真实窗口合成counter+content/音画输入及资源反控；pass后合autographics-dev、规范/索引/ledger沉淀、归档及wt-guard clean收树。next: merge；本轮未跑完整合入档、未合入/归档/清树，不能宣称交付完成。

### 2026-10-23 work r1：冻结执行交接，8/10

- stage: work | plan_id: PLAN-834 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-834/auto-lang | branch: plan-834-dev | base_commit: 540c1e70f0a4682af245191ffb36fa0d8a636665 | code_commit: ebeab12d2888b9c9fb7e00e3c24479fb3c028011 | implementation_commit: 0ea8bdb1582d05e0f39c6e5bfc6d349f9280473b | dependency_revisions: SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9、原两实验workspace锁 | task_ids: T-01..08 | blockers: none | next_responsible_role: independent review。
- [逐AC/V/SD readiness](../attachments/834/work-r1/readiness.md)、[身份](../attachments/834/work-r1/handoff.json)、[源码消费/破坏负控/真实窗口收据](../attachments/834/work-r1/834-source-consumer-summary-final.json)、[180项源码资产清单](../attachments/834/work-r1/plan834-identity.json)。T-03分型整包事务/资源归属、T-04完整Auto玩法及原3600tick/45spawn/91events、T-05原三mesh模型/图像/实际OGG循环、T-06真实路由/120Hz/取消重置/last owner、T-07精确计量/双游戏独立冷消费、T-08拟议Spec与说明全部核验。必测无skip，开放合同finding/coverage gap/未决问题均为空。
- 内部补强：原始yaw的±PI不能仅以矩阵等价验收；先复现红，再以Auto零轴helper及movement/spawn永久回归修复，重跑原轨迹/图像/当前C窗口与源码消费。RNG流事务、出生顺序、consumer使用前检查与init计数均有永久反例。源/SDK/原golden/容差与写面不变；不调用Rust SquashDriver代跑。
- 两workspace all-target check与本workspace fmt通过；SDK既有警告146条，未引入本次源码警告。180hash verify通过，唯一树交接时tracked/untracked delivery clean。全部实现只在experimental两workspace，5份Spec为worktree拟议文本；未改root crates/锁、SDK或live ledger。
- AC-01..07/V-01..09 work pass；AC-08/T-09未执行，AC-09/T-10及完整driver/AGP适用门禁pending。未合入、未归档、保留唯一树，停止实施写入供独立review。既有SDK-MATH-CONVERSION为原非目标，不延期本Plan任务；I1候选仍待最终审定，I2/I3/生产门不外推。


### 2026-10-23 work：来源/可行性关闭，实现候选验证收口

- T-01：固定交付SHA/原SDK保存锁/driver与AGP锁；680文件、17本地包、29路径边、479包实际metadata闭包，禁入包0；当前canonical四文件与冻结来源文本一致。资产hash及来源图在实施树`research/plan834-source.json`；build profile继承Windows PE32MB与test worker16MB。
- T-02：有限Auto f32数学+显式有序raw xorshift64*服务通过真实Session→原mob三mesh→GPU whole-path。原native atan2返回f64标签、同名abs lowering碰撞与64对象初版预算红均保留归因；未修改SDK。最终数学/预算/容量/错误分类见`research/squash-driver-feasibility.md`。两份.at被同一Session编译，非新语言桥。
- 实现布局等价调整：新增`src/squash.rs`承载显式profile2 staged decoder/producer与consumer publications，`src/squash_host.rs`承载资源/宿主，不扩污染profile1字段。完整三轴旋转/scale，birth-order槽复用、随机流整包提交及consumer逐资源前置校验已自检修正。
- 已执行候选证据：原3600tick/45spawn/91事件/135随机值、位置max0；旧Breakout10800tick/210contacts；五个GPU关键帧在原3/12容差内（四帧零差，踩踏帧mean4.63e-7/p99=0），无模型/无阴影/Auto动画及物理变异被正式门拒识。原25.195s OGG在实际Kira完成一轮wrap；20retry owner/cache与分项计量通过。原生完整流程已到phase9，含D移动/Space jump/文本焦点/Space+repeat与Enter retry，音乐只play1次。
- 以上为work候选验证，不替代最终冻结身份。T-03..08当前正在统一重核/提交与A→B源码消费，暂不提前勾选；当前2/10，T-09/T-10全pending。原生探针历史失败分别归因旧helper全局counter/重复动作、启动时角色自然死亡、以及合法松键迟到的基线屏障；已改为一次队列注入、fresh-session setup和实际焦点转移后采样，不放宽游戏/路由条件。



### 2026-10-23 work：用户启动，冻结交付来源

- 用户明确授权实施834；GSTART满足，历史“只规划”描述属于2026-10-22记录，不再限制本次work。G833满足：833 archived/delivered 10/10，独立R2及merge门通过，旧树已清。
- stage: work | plan_id: PLAN-834 | plan_revision: 1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-834/auto-lang | branch: plan-834-dev | base_commit: 540c1e70f0a4682af245191ffb36fa0d8a636665 | code_commit: 同base。
- GSOURCE：固定833交付autographics-dev完整SHA如上；SDK固定9576b4dad5ceb8b201adedd46f9689f25c2dd5f9，不追随master。主检出代码WIP为空，预存docs改动保持。唯一树top-level/branch已实核；680文件/17本地包SDK载体已准备。
- T-01/02进行中，尚未宣称任何V/AC通过；T-03依赖T-02 whole-path通过。T-09独立review、T-10 merge全部pending；本次授权work，完成后交接review，保留树。



### 2026-10-22 new r1：下一份计划草案

- stage: new；plan_id: PLAN-834；plan_revision: 1；status: drafting；current_step: 0/10。
- 依据canonical模块规范与833合同/独立R1，仅只读查看固定候选Git对象定位缺口；已知原golden3600tick/45spawn/91事件来自文件盘点，不是本轮执行成功。来源指纹见[观察清单](../attachments/834/planning/source-observation.json.txt)。
- outcome: **blocked（仅实施交接）**：G833未交付、最终来源未冻结、用户只要求提前规划；草案编制完成。next: 833 owner继续原review/修复与验收合入；之后重核/冻结834来源，用户启动时由work进入。
- 合同T-01..10、AC-01..09、V-01..09、SD-01..04全部待实施；[规划预检](../attachments/834/planning/preflight.md)。没有改变833、current Spec或代码，没有建立树/分支/agent，没有运行Cargo/游戏/窗口/音频测试。

## 10. 待澄清事项

1. 当前无开放合同finding、未决设计或用户决定；历史锁屏/额外输入/预算拒绝的实际重放闭合见最终merge收据，未修改受审实现或放宽验收。
2. 已合入autographics-dev并投影canonical/ledger；当前仅归档实际状态检查与guard清理待完成，T-10完成前不宣称10/10。
3. I1实验分母交付；既有SDK通用数学债务、I2/I3/Shader/CAP/Forge生产边界保持。无生产部署/默认入口迁移。

## spec-sync 回写记录

SD-01 auto-source-driver、SD-02新增auto-3d-game-profile、SD-03 integration-contract、SD-04 project/prototype-profile按受审增量与实际完整门证据沉淀；module overview/plans、GOAL-007/008/global overview回写，spec-index.py再生INDEX。源码只在autographics-dev；master知识投影。派生ledger guarded offline v70→71，8项upsert，1138无关条目原字节保持；canonical仍为docs/specs，独立review/merge历史指向已提交证据及本归档。

归档实际状态门修正：Design45背景笔记是主检出既有未跟踪材料，保留其规划取材身份为文字说明；不纳入其他会话WIP，发布知识只链接已提交canonical图形规范。此为归档文档修正，不改受审实现/Spec验收。
