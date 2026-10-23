---
plan_id: PLAN-837
status: archived
completion_kind: delivered
feature_name: AutoGraphics场景层级与组合对象创作原型
plan_revision: 1
author: [Codex]
created_at: 2026-10-23
updated_at: 2026-10-23
current_step: 9
total_steps: 10
depends_on: [PLAN-836]
supersedes_spec_components:
  - docs/specs/auto-graphics/scene-document.md
  - docs/specs/auto-graphics/authoring-session.md
  - docs/specs/auto-graphics/viewport-interaction.md
  - docs/specs/auto-graphics/auto-source-driver.md
  - docs/specs/auto-graphics/prototype-profile.md
  - docs/specs/auto-graphics/project.md
  - docs/specs/auto-graphics/integration-contract.md
new_spec_components:
  - docs/specs/auto-graphics/scene-hierarchy.md
touched_goals: [GOAL-007, GOAL-008]
affects: [auto-graphics, authoring-experimental]
---

# [PLAN-837] AutoGraphics 场景层级与组合对象创作原型

## 0. 变更摘要

从836的单对象直接操作，推进到**用场景树组织对象：创建组 → 挂接已有对象 → 整组移动／显隐 → 重新挂接 → 撤销／保存／重开一致**。首版仅2D/3D各自空间内、group只继承平移和可见性；叶子保留现有有限形状/模型及自身旋转缩放，不将通用TRS、Portal或生产UI层级纳入本Plan。

R2历史快照：**reviewed 9/10，final R2 PASS，F-837-R1-01/P2 CLOSED**。受审提交`2ffc92d34143bc97a98f2684406160b183fd2ff3`，同一树`D:/autostack/.wt/lang-837/auto-lang` / `plan-837-dev`，基面`52ec4c7c13b337992278b343ef89d8aa17102ff6`、SDK9576b4d不变。独立重放原R1反例及真实透明/残影GPU反控，均由同一move像素断言拒绝，native/report/aggregate一致；正常7状态、旧wrong-parent/drop-release/partial与保存后新进程重开通过，定向3项/check/fmt/source通过。191项库/测试/样本/AGP输入与R1一致，未受影响38项原独立结果按指纹复用，不复用原错误像素主张。T-01..09完成；**T-10完整冻结合入门及实际合入/沉淀/归档/清理均pending**，reviewed不等于交付。当前依据：[独立R2](../attachments/837/review-r2/review.md)、[R2基线/复用指纹](../attachments/837/review-r2/baseline.json.txt)、[冻结规范提案](../attachments/837/review-r2/spec-delta-frozen.md)。[独立R1](../attachments/837/review-r1/review.md)和工作侧记录保留历史，原树保留供merge。

范围为L1独立authoring模块特性；不重构AGP Scene/CompositionGraph、核心运行时协议或生产架构。源码仍仅合autographics-dev。若实现需要这些架构写面，回new并按L2先补正式架构设计，不能藏在本Plan局部任务中。

当前合入状态：archived9/10，完整冻结门、autographics-dev源码快进、canonical与ledger已验证；T-10仅归档提交及guard清理待完成。详[合入收据](../attachments/837/merge-r1/receipt.md)。下文各阶段pending为历史交接。

## 1. 目标

1. 同一创作窗口显示稳定ID的2D/3D场景树，支持空组创建、命名、父子关系、组/叶子选择与局部属性；节点移动后实际后代画面与拾取一致。
2. 真实Auto计算层级world translation和effective visibility；父组平移/隐藏自动作用于所有后代，未相关兄弟不变。Rust负责输入、图准入、结果核验与渲染，不代算结果冒充Auto。
3. 支持保世界位置重新挂接到同维group或根、删除子树及完整Undo/Redo；每次结构命令或成功拖动只产生一条before/after历史。
4. 新版本场景文件保存层级/局部属性，兼容导入835/836旧文件；失败不发布半棵树、不污染accepted内容/历史/文件，重开恢复同一场景。
5. 把独立原型推进到“有限场景组织与直接编辑闭环”，并据实际证据说明其与生产AutoGraphics/AutoForge、完整引擎的差距。

profile：Windows x64、单窗口stock Iced/wgpu、既有两个固定预览及固定相机；2D circle/quad，3D cylinder/player/mob；总节点数≤128（组+叶子共享预算/ID域）、历史≤64步/16MiB、文件≤1MiB、名称≤128 Unicode。无环层级最大路径自然受节点预算限制，不另加隐蔽深度截断。旧128叶子文件仍合法；再创建组必须按总额明确拒绝，不能丢旧对象腾位置。

非目标：group旋转/缩放/剪切、跨2D/3D父子继承、任意UI/VTree/Surface/Portal拓扑编辑、多父实例/Prefab、树拖拽排序、多选/框选、旋转缩放gizmo、相机漫游、新资源/材质/贴图/动画/物理扩面、游戏玩法编辑、AI/源码生成、AutoShader/CAP/ACE、生产AutoUI/RQHost/正式Forge/跨平台发行。本切片不据此认领I2或I3。

## 2. 架构方案

```text
v1旧文件 / v2层级文件 → 有界严格decode / 版本映射
  → 有限Document（叶子+group，稳定ID，局部属性，parent）
  → 图准入：同维/单父/完整引用/无环/预算/数值
  → EditCommand或836 Gesture → 暂存候选Document
  → 显式Auto authoring profile4（父先子后，world/effective值）
  → 输出身份/数量/语义核验 → 固定资源准备
  → 一次发布Document+树+真实预览+history / 临时gesture预览
  → v2保存，Undo/Redo，新进程Open/inspect
```

仅扩`experimental/autographics-auto-driver/`，沿用`autographics-authoring`，AGP/SDK/根workspace只读。建议新增`src/authoring/hierarchy.rs`、`samples/authoring_hierarchy/main.at`和定向测试；具体模块等价划分由work决定。Document和derived快照不含wgpu/VM/Iced对象。层级是authoring对象模型，不复用或改义830的Surface/Portal组合DAG。

| 门 | 条件与当前状态 | owner_stage |
|---|---|---|
| G836 | 836 final review、冻结完整门、实际合autographics-dev、Spec沉淀/归档/guard清理及到期检查均完成。已核archived10/10，全部满足；见836 merge-r1正式收据 | work进入T-01前核实 |
| GSOURCE | G836后固定实际交付完整SHA、SDK/锁/源锁/资源和新增836 canonical内容；候选5f3e218仅观察，不是837冻结 | work T-01 |
| GSTART | 用户本轮明确授权837 work，G836已满足；GSTART成立 | work启动前 |

本次可在主检出调查、起草和记录观察，**G836之前不建837树、不跑准备/build/probe、不改836或canonical**。唯一未来树`D:/autostack/.wt/lang-837/auto-lang`，分支`plan-837-dev`，显式从通过G836的autographics-dev完整SHA创建，不能用默认master基面捷径。work/internal check/final review同树顺序执行；Plan进度在主检出。源码合autographics-dev，master只沉淀知识；禁止junction/symlink，移树前wt-guard。

## 3. 技术栈

沿用836的Rust、stock Iced 0.14、wgpu 27、winit、glam、serde、固定7 mesh与原资源/锁；SDK观察值`9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`，实施时核实，不自行升级。用现有`Session/Scalar/Progress`的公开标量/有界CPU-slice调用；复用`Building`、`Editor` before/after事务、`Storage`协议、836拾取/gesture/owner/counter工具。

新profile4显式标识层级创作，profile1/2游戏和profile3原平面样本的协议及来源保留。新建/导入后的层级编辑走profile4，不凭数据形状猜profile。新文件schema `autographics.scene/2`属于独立实验交换格式，不是Auto语法或新的核心引擎协议；不引入ECS/通用场景图框架或物理库。

## 4. 需求分析与背景调查

### 4.1 授权与取材

规划阶段（历史）：2026-10-23用户要求“计划836已经开发完毕；请规划下一个计划，并说明完成后体系进度”。授权只读调查/新Plan/进度分析；没有实施、合836、创建聊天/agent、指定模型/角色或预算授权。遵循月度小步滚动，只细化下一份，不提前把后续候选变成承诺。

现行依据：[全局overview](../../specs/overview.md)、[图形overview](../../specs/auto-graphics/overview.md)、[project](../../specs/auto-graphics/project.md)、[scene-document](../../specs/auto-graphics/scene-document.md)、[authoring-session](../../specs/auto-graphics/authoring-session.md)、[surface-composition](../../specs/auto-graphics/surface-composition.md)、[integration-contract](../../specs/auto-graphics/integration-contract.md)、[v0.6图形目标](../../roadmap-v0.6.md)、[Design45](../../design/45-autographics-wgpu-engine-and-game-prototypes.md)、[创作策略](../../design/strategy/autographics-forge-v06.md)。历史路线有旧pending/旧Plan路径，本轮以已归档834/835及836现行Plan/R2为状态依据，不重写他人WIP。

836依据：[Plan](836-autographics-viewport-picking-and-translation.md)、[final R2](../attachments/836/review-r2/review.md)、[冻结Spec提案](../attachments/836/review-r2/spec-delta-frozen.md)，只读候选代码/测试及research/viewport-handoff.md。候选提案尚非canonical；其新viewport-interaction组件须由836 merge先沉淀，837再modify。来源hash见[观察清单](../attachments/837/planning/source-observation.json.txt)。

### 4.2 观察到的链路及差距

| 实际触点（836候选SHA） | 已观察事实 | 下一步 |
|---|---|---|
| document.rs::Document/Object/Kind | schema1、平面objects表、严格有限codec；没有group/parent | 引入独立v2格式及v1显式迁移，保持久ID/旧内容 |
| samples/authoring/main.at | profile3保存/返回平面配置；无父子计算 | 新profile4真正计算world/effective，不能只给Rust画面加组框 |
| preview.rs::Building::{new,pump,finish} | 全配置→真实Session→精确输出准入；现expected为平面Document | 输入局部图，核验完整group/leaf派生结果；语义错误不部分发布 |
| session.rs::Editor/Edit | Replace/Create/Delete/Undo/Redo，历史为before/after整文档 | 增组/挂接/子树操作，沿用同一事务/dirty/预算 |
| interaction.rs::Gesture::{document,sample} | target为单Object，delta目前直接改其translation | 区分选中group/leaf、world/local位移，沿用token/最后端点/取消 |
| host.rs::Panel/Host::prepare/publish | 列表/属性、固定资源；presented对象为真实Auto输出 | 列表扩为树，derived世界叶子同时供渲染/拾取/覆盖层；不直接拿局部Document拾取 |
| bin::App与authoring_probe_exit | 836原生流程和完整性判断已复审；早退不能exit0冒认成功 | 新层级原生门沿用统一终态/报告规则及partial/负控 |

选择层级是因为当前能编辑单个对象，却不能稳定组织/移动一个组合。只做更多gizmo没有补场景结构；先加资源/高级材质会同时扩大资产、渲染和创作面。组只继承平移/可见性，可避开非等比缩放叠加旋转的剪切/分解问题；不把有限支持宣传成通用TRS场景图。

生产边界：831/832仍带全计划交付启动锁，816未完整交付。本Plan不准备或实施这些迁移；I2生产AutoUI/RQHost、I3正式Forge/发行、Linux及Shader/CAP保持各自前置条件。

## 5. 详细设计

### 5.1 入口与有界whole-path探针

当前T-01..09完成，final R2在2ffc92d34独立确认F-837-R1-01已关闭；下一环节为T-10冻结完整门及实际合入收口，原树保留。工作侧历史结果及实际触点见§9 work-r1/work-repair-r1，当前审查见§9 review-r2。G836实际交付SHA/SDK/锁/原资源均已重新核验，未在旧候选基面分叉。T-02从scratch的Iced/Auto/GPU探针延伸为同树真实Document/Building/Editor/Host/v2新进程whole-path，在前置可行性闭合后继续hardening；只需一种driver-local接法，无SDK/AGP/生产写面。下面§5.1探针条款保留为原合同，实际结果见research/hierarchy-feasibility.md。

T-02建立新`research/hierarchy-feasibility.md`：2D一个组两叶子、3D一个嵌套组及原GLB，经真实Auto求world/effective；真实Iced树控件改父/组平移→画面与拾取→一次undo→v2保存/新进程重开；再测加环、错误Auto parent累加、准备失败恢复。用最小scratch数据闭合协议/状态owner，不要求后续正式codec/UI已实现。最多比较两种driver-local接法，记录输出布局/拓扑排序/ID映射/数值规则/时延和故障恢复。需要SDK、AGP、生产UI扩面或无法保全836手势合同则回new，后续相关任务不开始。

### 5.2 有限文档及图准入

拟议v2固定结构为document_id、固定resources、objects、groups及schema；object保留原字段并增加显式可空parent，group有id/name/dimension(2d|3d)/parent/translation[3]/visible。组无resource/rgba/rotation/scale；其行为固定为平移继承。具体serde标签由T-02固定，必须严格拒绝未知/重复字段，不能通过忽略字段支持未实现的变换。

ID为共享非零u64，objects/groups之间唯一；既有u64::MAX合法数据须可打开/编辑/保存，耗尽只拒绝新建。父只可为存在的同维group或null根，叶子不能作父、单父、不自环/成环，无跨空间继承。图入界一次完整校验后按确定性父先子后拓扑生成映射；遍历/销毁在128节点边界有界，不用静默截断、跳坏节点或递归死循环。

2D group z固定0；局部数值和派生world分量均有限且abs≤100000，原叶子scale/rgba/yaw/pitch规则保留。即使隐藏后代也校验数值。节点/文件/历史超限属于明确资源拒绝，坏引用/环/类型属于图语义拒绝，不混为“文件损坏”或溢出后clamp。组/叶子无环最长链在128节点内，接近上限的合法深链必须接受。

objects数组顺序继续定义2D叶子绘制先后；group树显示、挂接或折叠不暗改数组顺序。树的父先子后行序在同父下按原对象/组稳定顺序固定；本版不支持排序编辑，不能把UI重排顺便当z-order修改。fold/selection等仅UI态，不入文件/dirty。

### 5.3 层级的真实Auto计算与统一消费

profile4输入为完整局部节点配置、明确node kind/dimension、parent ordinal及原叶子字段；parent采用事务内有界ordinal，不把u64持久ID压成SDK int。Rust持唯一双向映射；给Auto的parent关系不能被host先展平成world再送入。根父采用明确哨兵，输出profile/count/id/kind/dimension/parent/world translation/effective visibility及叶子属性，group也可查询校验。

Auto按父先子后计算：root world=local；child world=parent world+local（f32、每层一次加法）；effective_visible=local.visible AND parent.effective_visible。叶子世界矩阵为T(world)*Ry(leaf.yaw)*Rx(leaf.pitch)*S(leaf.scale)；group不继承旋转/缩放。全配置、调用/输出查询仍走Session/Building有界pump和累计预算。

Rust检查profile、完整ID/parent映射、数量/唯一性/维度、数值和派生语义。可用独立CPU计算作准入oracle，但**渲染/拾取/选框必须消费实际Auto返回的world/effective输出**，不渲染Rust参考值。错parent、少节点、错误累加/visibility或旧token结果拒绝整包并保last-good；不接受部分子树，不以其他profile兜底。单次发布包含Document、选择/树、真实Auto预览、history和revision；gesture仍保持836临时与accepted隔离。

### 5.4 结构命令与数值语义

命令绑定epoch/revision、target稳定ID、期望旧parent及新parent；同名不同ID不可误操作。支持创建空2D/3D组、命名/局部平移/visible、将组或叶子重新挂到同维组/根、删除组整棵子树。新建叶子允许选择同维父组；不合维/容量/数值则拒绝。

Reparent仅有明确的**保持世界位置**模式：new local=old world−new parent world（根为0），只改target的parent/local，后代局部字段保持；自身visible不改，实际可见性依新祖先重新计算，不承诺保持视觉显隐。禁止挂到自身/后代、叶子/跨维/缺失父；新局部或派生值越界拒绝整次命令。f32舍入允许每坐标误差≤max(1e-3,8*f32::EPSILON*max(1,abs(before),abs(after)))；须回算target及所有后代world，因大数相减等无法满足此容差时明确拒绝整次重挂并保旧状态，不能放宽精度或移动后代补偿。测试小尺度固定1e-3，不放宽到pixel近似。不产生逐帧累计漂移。

删除组须在控件中明确显示后代数并确认删除子树；取消不改状态。一次delete/undo恢复全部原ID/parent/局部属性/数组顺序，created ID high-water不回退。选择若被删除则清空，Undo不复活旧回调/VM所有权。创建/重挂/删除/属性操作全部复用before/after预算与hash-based dirty；一次命令或完成拖动只加一个revision和history，拒绝/noop不加；undo后编辑清redo。

### 5.5 树控件、组手柄与局部/世界一致

真实Iced场景树按2D/3D根区展示group与leaf，支持树选择、父目标选择及“保持世界位置挂接”按钮、根目标、创建/删除组和局部属性；首版不要求树行拖拽。控件和旧draft必须带stamp/target，选择变化清旧draft；父选择项以稳定ID绑定，不能只按名称找父。显示local及只读world/effective，用户能看出父隐藏的影响。

叶子点击仍按836几何/OBB选择实际叶子，不自动换成父；组由树选择。选中组时，对可见后代显示一个聚合边界轮廓和group origin平移手柄：2D拖手柄平移该组，3D用原世界X/Y/Z手柄；空组/无可见后代或origin裁剪/退化时只提供树属性编辑，并显示原因。聚合取所有可见后代实际几何边界，覆盖层全部绘制仍在既有4096条预算内，不能按每后代无限叠加线段。轮廓是辅助显示，不把联合屏幕框伪称精确对象拾取。

拖叶子时以实际父world为基准将目标world位置转回local；拖组时只改该组local，后代local不改但其Auto world一起变化。所有拾取、手柄、临时预览用同一已呈现derived快照/相机/epoch/revision；不得在world渲染后还用local位置拾取。父/结构修改与活动手势互斥，不排迟到命令，保留836捕获/取消/失焦/DPI/最后端点/重复release规则。结构命令失败恢复树/selection/preview，随后合法命令恢复。

### 5.6 文件迁移、历史、资源与边界

严格识别v1/v2，未知版本拒绝。v1合法文件映射为同ID/数组顺序/原局部值、parent=null且groups=[]的v2内存文档；旧格式坏输入仍拒绝，不能借迁移绕开835 duplicate-key/finite/budget规则。打开不自动写盘，源文件原始字节/hash留给Storage原有冲突检查；新内存canonical语义hash作saved_hash，刚导入clean、history空。用户显式Save/Save As才写v2；v1无组时Auto世界输出/像素与原内容一致。旧版程序不能读v2，须明确版本不向后读兼容，不能假称835 reader也支持层级。

保存只消费Idle accepted内容，原owned lock/temp/flush/rename/readback及unverified恢复保持；parent关系/组局部字段持久化，derived world、树折叠/选择、history、gesture/VM/GPU不存。新进程Open/inspect经v2准入和profile4重建，结构、local/world/effective、画面/拾取一致，history/selection空，dirty=false。文件或Auto/资源失败，保旧accepted与旧文件，旧epoch保存回执不能清新文档dirty。

总额128、1MiB、64步/16MiB保留，不按后代数复制独立history。新增组无GPU mesh，同固定7 mesh资源复用；warm组/叶子平移、显隐/挂接不重建pipeline/target或重传固定vertex/index。836最多accepted/显示临时/inflight各一个VM和一个最新坐标的边界保留；计量UI/组合更新开销不冒认零帧buffer写。20轮嵌套组移动/取消/失败/删除重建验证实际弱owner/mesh pin最后释放，公开Iced内部/驱动/GPU计时排除。

### 规范增量

本节全为拟议，当前不写canonical。836新viewport组件必须先正式落地，review再冻结837增量，merge沉淀并刷新overview/plans/goals/ledger。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/scene-hierarchy.md | 无对象层级→有限单父同维group、平移/显隐、图准入和world语义 | 场景组织可消费 | AC-02, AC-03, AC-04, AC-08 |
| SD-02 | modify | docs/specs/auto-graphics/scene-document.md | schema1平面→v1严格导入/v2层级保存、shared预算、dirty/文件原始冲突身份 | 持久化不丢旧内容 | AC-02, AC-06, AC-08 |
| SD-03 | modify | docs/specs/auto-graphics/authoring-session.md | 单对象事务→组/重挂/子树原子命令、统一history/选择/失败；实际profile4输出 | 不发布半树 | AC-03, AC-04, AC-06, AC-08 |
| SD-04 | modify | docs/specs/auto-graphics/viewport-interaction.md | 836单叶子world=local→树选组/组手柄、world/local转换、derived统一拾取；旧取消/末端合同保持 | UI与画面一致 | AC-04, AC-05, AC-07, AC-08 |
| SD-05 | modify | docs/specs/auto-graphics/auto-source-driver.md; docs/specs/auto-graphics/prototype-profile.md; docs/specs/auto-graphics/project.md; docs/specs/auto-graphics/integration-contract.md | 独立创作→显式profile4/有限组织能力/实测边界，I2/I3仍pending | 不冒认生产/通用引擎完成 | AC-01, AC-07, AC-08, AC-10 |

### 5.7 完成后的体系位置

以下以836/837均完成review、完整门、实际合入为前提，不是当前完成声明。独立实验可称“**2D/3D基础运行与组合已验证，有限场景组织和创作闭环基本成型**”。具体分母和剩余工作见[进度分析](../attachments/837/planning/progress-assessment.md)，不用功能计数百分比或Godot/Bevy完成度百分比。

| 能力层 | 已交付／本Plan新增 | 837后仍缺 |
|---|---|---|
| wgpu图形/游戏基础 | 829/833/834固定2D/3D小游戏、真实Auto、原GLB/音频/输入/生命周期 | 更广资产/材质/渲染/物理/动画/平台支持不由两示例证明 |
| UI/2D/3D组合 | 830独立Iced同设备、多层Surface/Portal及输入已验证 | 生产AutoUI/RQHost adapter和迁移未完成 |
| 创作与文档 | 835保存/undo，836候选拾取/平移；837增加树/组/重挂/继承 | 通用TRS/Prefab/资源浏览导入、游戏编辑/脚本绑定及完整Forge产品 |
| 语言/公共API | 固定SDK/profile实验已跑真实Auto；837层级计算也入Auto | 未完成稳定通用AutoGraphics库/引擎API、ACE/native全面接线 |
| 发行/生态 | 原型独立source consumer可复跑 | 生产迁移/多平台打包、正式Forge和Linux交付、Shader/CAP等 |

优先转向生产集成还是补资产/公共API，由届时迁移交付与真实消费者缺口决定；此处不自动新增下一Plan，不在独立编辑器上无限堆功能来代替I2/I3。

## 6. 测试设计

以下保留原验收合同；新增入口/fixture已建立，实际执行、数量、反控与冻结身份见§9 work-r1及其收据。未指定实施/internal/final reviewer模型或新聊天；final review同一837树、独立上下文，暂停实施写入，修复回同树。实现会话自查披露局限，不冒充另指定审核人。merge完整门后置。

命令均在当前837树`experimental/autographics-auto-driver`执行。T-01先用现有`python tools/prepare_sdk.py --repository D:/autostack/auto-lang`及`python tools/verify_source.py`核固定来源。新定向入口由T-03..07建立：`cargo test --locked --test authoring_hierarchy -- --nocapture --test-threads=1`，并按触面跑旧authoring_document/session/storage/host/interaction/probe_exit。check/fmt沿用已有`cargo check --locked --all-targets`和`cargo fmt -- --check`。

新原生自检模式已实现为`autographics-authoring --hierarchy-check --report <path>`，T-07建立；原interaction-check/window-probe/inspect继续可用且终态判断不弱化。具体测试名/CLI拼写可等价调整，T-08给准确命令及数量，0selected/skip不算完成。

| case_id | AC / work owner | valid/invalid输入及状态 | 实际入口与observable oracle | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01/07；T-01/08 | 836正式交付SHA、SDK/锁/资源，源漂移/缺资产反控 | 原verify_source/独立consumer闭包指纹；原profile1/2/3与原资产/golden不漂移，scope审计 | G836后实施，不消费候选作冻结 |
| V-02 | AC-02；T-03 | 128节点有效深链/空图/同名/u64MAX；重复跨表ID、缺父、自环/2节点环/长环、叶子父、跨维、129节点、坏数值/文件 | 正式codec/图准入；合法接受且可遍历/close，非法在Auto/GPU前具名拒绝、accepted不变；隐藏越界也拒绝 | 有限同维森林，不是Portal DAG |
| V-03 | AC-03；T-04 | 2D/3D多级组，parent+child各非零位移，可见组合，旋转/非等比叶子 | profile4真实Session→query→consumer→GPU；world/effective/原mesh矩阵与独立f32参考相符；parent累加/AND改OR/输出wrong-parent受控变异红，不能仅比改过的Auto自回放 | 仅平移继承；真实输出为消费数据 |
| V-04 | AC-04/05；T-05/06 | 原点偏移父下拖叶子、拖多后代组，非相关兄弟/同名ID，终点/重复release/取消 | Host真实gesture/Auto/画面+pick；只改target local，后代local不变/world同步；组move一次history/revision，cancel/noop无新增；world位置可实际点中，旧local位置不可误中 | 2D组handle和3D三轴，无旋转缩放组 |
| V-05 | AC-04；T-05 | 重挂到同维父/根，隐藏父，挂后代/跨维/缺父/越界、f32相消无法保位→合法重试 | Editor正式命令+Auto输出；parent变、target及后代world在§5.4容差内、local/effective为约定值；失败doc/hash/history/selection/preview不变；合法重试成功 | keep-world仅位置，不承诺keep-visibility |
| V-06 | AC-04；T-05 | 删除嵌套子树/确认取消、undo/redo/分支、容量/历史上限、耗尽ID | Editor+真实控件；完整成员/顺序/ID恢复，兄弟不变，1步历史；无旧消息复活，空组可编辑，超限不部分删除 | 单次原子命令，不多选/组实例 |
| V-07 | AC-02/06；T-03/07 | v1旧文件/坏v1、v2嵌套；打开后不保存、显式save、原字节外部冲突、损坏v2/未知schema | Storage+新进程inspect；v1导入clean/原字节不改，v2写后parent/local/顺序/effective/像素/pick重建一致；history/selection空；冲突不覆盖，失败保旧 | 不承诺旧reader读v2，沿用本地保存范围 |
| V-08 | AC-03/06；T-04/05/06 | 错Auto/资源准备/末端失败、pending结构命令、旧epoch/parent/draft/result/save receipt | Host/Building真实故障→下一合法结构操作；树/accepted/history/文件全保，临时preview正确恢复，无部分子树/迟到清dirty；前序836取消族定向复跑 | 保留trusted同步compile限制 |
| V-09 | AC-07；T-04/08 | 128节点世界数值边界，20轮移动/挂接/取消/故障/删除重建/关闭及合法外部pin | 实际VM弱owner/7 mesh Arc/target/pipeline/上传；活owner保持836上限，固定mesh无warm重传，last-owner真实释放，刻意保留owner/重传反控拒绝 | 报告CPU/内存范围；不虚构GPU时间 |
| V-10 | AC-05/06；T-07 | 实际OS树选择/创建组/挂接/整组动/undo/redo/显隐/save/重开；wrong-parent/drop-release/frames1 | 新hierarchy-check经OS→Iced/router→命令→Auto→最终合成；state/具体像素与world/pick同步；正门完整exit0/passed=true，错父/漏释放/不完整均非零及明确报告 | 真实桌面必须可用，环境gap不算pass |
| V-11 | AC-08；T-08 | SD01..05/实际实现/边界和体系进度 | 冻结Spec提案及README/research交接，G836/源码/命令可定位；I2/I3/通用引擎剩余项明确 | 静态检查不替代原生/几何/Auto门 |

每case记录准确command、退出码、实际断言/数量、code/SDK/锁/fixture指纹及原始证据。V-03的变异须进入实际Auto输出/consumer；V-10不是直接dispatch模拟交互，图像变化和node计数不能单独证明层级正确。复用前序证据必须确认实现/依赖/配置/输入未变；新增profile4/文件格式/树UI不能只复用profile3的绿。

开发与修复仅定向/check/fmt，不跑root t/tv/tt/tb/tu/taa/docs_gen或完整driver/AGP档。最终review冻结提交后，merge统一跑完整locked driver/AGP test+build、原16 AGP case、双游戏/835/836原生正反门、新层级正反/partial/reopen、两道counter/content合成门和source/asset/独立consumer门；T-08列可执行准确命令/选面。若实际触及生产Rust，须回new界定范围和适用root门，不能自动扩面。tf仅主检出批量到期档。

## 7. 验收标准

| ID | owner_stage | 可观察结果和验证 |
|---|---|---|
| AC-01 | work | G836/GSOURCE、唯一树、固定SDK/写面成立；V-01 |
| AC-02 | work | v1严格导入/v2有限森林、共享ID/预算、边界准入无环/无丢节点；V-02/07 |
| AC-03 | work | world/effective真实Auto计算并统一实际消费，错parent/数值/旧结果不部分发布；V-03/08 |
| AC-04 | work | 组/重挂/删除子树及单步history/dirty/恢复正确，keep-world容差及兄弟隔离；V-04/05/06 |
| AC-05 | work | 真实树/属性/叶子拾取/组手柄world-local一致，原生正反/partial门成立；V-04/10 |
| AC-06 | work | accepted保存、v1原字节保护、v2新进程结构/画面重建、失败/迟到/dirty保护；V-07/08/10 |
| AC-07 | work | 原profile/game/资源/依赖不漂移，128节点/20轮owner/mesh复用有界；V-01/09 |
| AC-08 | work | SD-01..05、准确可复跑入口/研究/体系进度及未完生产边界完整；V-11 |
| AC-09 | review | 独立核AC-01..08、Spec、遗漏/无授权延后/workaround/fmt/warnings/debug，绑定revision/commit pass；merge门pending |
| AC-10 | merge | 完整冻结门通过才实际合autographics-dev，canonical/ledger/归档/guard清理和批量到期收据完成，源码不入master |

## 8. 执行步骤

当前reviewed9/10：final R2独立确认2ffc92d34修复有效，F-837-R1-01 CLOSED且无新增阻断；T-01..09完成。最终review历史为R1 needs_fix、R2 pass，work自查不计作独立review。T-10完整冻结门和实际合入/知识/归档/清理仍pending，实际完成T-10才交付10/10。

| task_id | owner_stage | 依赖 | 实际触点/新路径和动作 | 可执行关闭条件；AC |
|---|---|---|---|---|
| T-01 | work | GSTART/G836 | 主检出合同/836正式收据；唯一837树；tools/prepare_sdk.py、verify_source.py及锁/资源 | [x] 固定实际交付SHA/SDK/closure/hash，prepare/verify及反控；写实施冻结，不用候选；AC-01；当前结果见§9 work-r1/收据AC矩阵 |
| T-02 | work | T-01 | document/preview/session/interaction/host/bin；新research/hierarchy-feasibility.md及scratch profile | [x] §5.1真实whole-path+取消/错误parent/准备失败恢复可跑；协议/排序/数值/owner/资源决策闭合，否则回new；AC-02, AC-03, AC-04, AC-06；当前结果见§9 work-r1/收据AC矩阵 |
| T-03 | work | T-02 | document.rs；新hierarchy.rs/new tests/authoring_hierarchy.rs；Storage有界decode入口 | [x] V-02及V-07的codec/迁移部分定向通过，完整拒绝分类/ID/深链/legacy映射，无写盘副作用；新进程部分由T-07关闭；AC-02, AC-06；当前结果见§9 work-r1/收据AC矩阵 |
| T-04 | work | T-03 | 新samples/authoring_hierarchy/main.at；preview.rs及SDK bridge调用/受检consumer | [x] V-03及V-08/09的preview/owner部分通过，真实world/effective/全输出准入/变异反控/预算；Host故障及整轮资源分别由T-06/T-08关闭；AC-03, AC-07；当前结果见§9 work-r1/收据AC矩阵 |
| T-05 | work | T-04 | session.rs::Edit/Editor、hierarchy.rs、Host命令准入 | [x] V-05/06/08，完整结构事务、keep-world/visible/删除/历史/失败恢复/旧parent拒绝；AC-04, AC-06；当前结果见§9 work-r1/收据AC矩阵 |
| T-06 | work | T-05 | Panel/Host::prepare/publish、interaction::Gesture及presented快照 | [x] V-04/08，真实树/属性/组选中/手柄、世界局部转换/末端/取消与受影响旧interaction测试；AC-04, AC-05, AC-06；当前结果见§9 work-r1/收据AC矩阵 |
| T-07 | work | T-06 | bin新hierarchy-check及原inspect/完成性；新fixtures/tools/research | [x] F-837-R1-01工作侧修复：7状态实际top像素断言、真实透明/原位置残影故障及原审查反例正确拒绝，正常/parent/release/partial/reopen保留；AC-02/05/06；见§9 work-repair-r1 |
| T-08 | work | T-07 | README/new research/hierarchy-handoff.md、身份/打包与定向测试，树内Spec proposal | [x] 247项来源/SDK680/closure479与来源反控、独立consumer核验及定向check/fmt通过；README/handoff/SD-05/旧覆盖主张已纠正并冻结；AC-01/07/08；见§9 work-repair-r1 |
| T-09 | review | T-08 | 同树冻结code/Plan r1/Spec proposal，主检出独立review记录 | [x] final R2 PASS@2ffc92d34，原R1反例/透明与残影GPU故障正确拒绝、正常7状态/旧负控/reopen/定向3项/check/fmt/source通过；AC01..09 pass，F-837-R1-01 CLOSED，无新增finding；[独立R2](../attachments/837/review-r2/review.md)，完整merge门pending；AC-09 |
| T-10 | merge | T-09 pass | 同树冻结完整门→autographics-dev；主检出知识/ledger/archive/guard | AC-10全流程实际收据；改代码先复审再重冻重跑受影响门，不提前交付 |

## 9. 复审记录

2026-10-23：stage: merge；PLAN-837:r1；prepared/landed/ledger_refreshed/archived，cleanup pending。final R2 2ffc92d34；纯规范/台账后继delivery cf2e99f50ee2a70b70ca2cfc23a5af56b405638b，实施/依赖/配置/输入diff为空。完整driver80/AGP162零失败忽略，官方16case、双游戏与835/836原生正反、hierarchy7状态/新进程重开/真实透明残影反控、counter与content双门及全新consumer检查均通过。主检出仅知识；台账离线保护73→74。归档提交后才能guard清理。详[收据](../attachments/837/merge-r1/receipt.md)、[门禁审计](../attachments/837/merge-r1/validated.json.txt)。

2026-10-23：stage: review | review_scope: final | reviewer_identity/limitations: 本聊天Codex既有独立审查上下文，与repair实施交接分离；未创建agent/新聊天，不虚构模型/session身份 | plan_id: PLAN-837 | plan_revision: 1 | outcome: **pass** | worktree_path: D:/autostack/.wt/lang-837/auto-lang | branch: plan-837-dev | reviewed_commit: 2ffc92d34143bc97a98f2684406160b183fd2ff3 | base_commit: 52ec4c7c13b337992278b343ef89d8aa17102ff6 | dependency_revisions: SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9、driver/AGP锁不变，source247/SDK680/closure479/17locals/29edges新核；191项库/测试/样本/AGP输入相对R1逐内容hash未变 | spec_inputs: [R2基线](../attachments/837/review-r2/baseline.json.txt)、[冻结SD01..05](../attachments/837/review-r2/spec-delta-frozen.md)，canonical目标hash与R1不变；修订SD05实际7状态/读回拒绝/透明与残影反控匹配实现 | acceptance_results: AC01..09 pass，AC10 pending | findings: **F-837-R1-01/P2 CLOSED，无新增finding** | evidence: [独立R2报告](../attachments/837/review-r2/review.md)、[独立状态断言核验](../attachments/837/review-r2/assertion-audit.json.txt)、[原R1反例重放](../attachments/837/review-r2/original-replay/result.json.txt) | pending_closeout_gates: 完整locked driver/AGP测试构建/16case、原双游戏/835/836 native正反门、hierarchy正反/partial/reopen/GPU反控、双合成counter/content、source/asset/全新consumer终档；实际autographics-dev合入/知识/ledger/归档/批量到期/guard清理均pending | next: auto-plan-merge在同一冻结树执行T-10，全部通过前不得合入或宣称交付。Plan reviewed9/10，勾T-09、T-10保留未完成。新原生正常phase16/204frames及7像素状态、wrong-parent phase5/漏release phase9/frames1 incomplete、新进程保存重建全符合预期；原反例只重绑HEAD guard（原脚本hash60727913...不变），实际transparent与ghost均phase9像素拒绝/native1/reportfalse/aggregate1，未以编译或环境失败冒认反控。定向3项0失败/忽略，check/fmt/source通过；R1独立38项只在未变输入范围明确复用，原错误像素主张废止。实现源码未改、树clean且HEAD未漂移；无新增driver warning，SDK146预存warnings不变；未跑root或完整driver/AGP档，未合入/归档/清树/发布canonical。

2026-10-23：stage: work | plan_id: PLAN-837 | plan_revision: 1 | outcome: **pass**（F-837-R1-01工作侧关闭，独立复审待） | worktree_path: D:/autostack/.wt/lang-837/auto-lang | branch: plan-837-dev | base_commit: 52ec4c7c13b337992278b343ef89d8aa17102ff6 | reviewed_commit: abe682c5a1144f7e9b72f4ffeb95ece19dd02841 | code_commit: 2ffc92d34143bc97a98f2684406160b183fd2ff3 | dependency_revisions: SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9、锁/AGP/authoring库/游戏资源未变 | task_ids: T-07,T-08 | evidence: [修复闭环表](../attachments/837/work-repair-r1/readiness.md)、[当前结构收据/指纹](../attachments/837/work-repair-r1/receipt.json.txt)、树内research/hierarchy-repair-r1.md | blockers: none | next_responsible_role: 原树受影响独立final re-review。当前execution_done8/10，实施停止写入并保留树；未创建agent/新聊天、未自行复审或合入。

原R1透明GPU脚本逐字节复制并修前/修后重放：修前native/aggregate假通过phase16，修后在phase9实际move像素断言拒绝、report false/native1/aggregate1。永久透明终点与保留正确终点但原位置残影两个真实GPU故障，各保留合法state/commands后仍被同一门拒绝，分别证明两坐标断言有效；正常temporary/move/Undo/Redo/hidden/Show/saved7状态像素通过，phase16/204帧，旧wrong-parent/drop-release/partial及新进程v2重开通过。新增正常/反控均为真实最终top readback，不以hash/命令代理。3项定向完成性/CLI测试、check全targets/fmt/source247/SDK680/closure479/17local/29edges与来源反控及新独立consumer source/controls/all-targets通过。原R1未受影响38项图/Auto/事务/资源等证据只在当前字节指纹/依赖/配置/输入不变范围复用；最初可见像素覆盖主张已撤销并更新README/handoff/SD-05。无开放工作侧finding/覆盖gap/产品决定；SDK146预存warning不变，无作者新增warning。final失败仍累计1轮，work自查不充作独立pass；完整driver/AGP/原生/双合成/consumer冻结门及实际合入/沉淀/归档/guard清理均merge-owned pending，未跑整档。

2026-10-23：stage: work；F-837-R1-01修复入口，用户needs_fix通知授权继续原合同修复。已核唯一原树D:/autostack/.wt/lang-837/auto-lang / plan-837-dev / HEAD abe682c5a1144f7e9b72f4ffeb95ece19dd02841，树clean，主检出无代码WIP。保留R1受审/透明GPU反例输入与正控，复制原重放脚本到work-repair-r1/pre-fix（原始review附件不覆盖）先复现。当前不变量：可见终点必须是初始叶色、被移空位置必须是背景；隐藏两处均背景；缺失readback不能当None相等通过。实际路径是OS/Iced→Host.frame→真实GPU最终top→native hierarchy断言→App报告/exit→聚合脚本，而不是world/commands/hash代理。适用兄弟状态含临时呈现、完成移动、Undo/Redo、隐藏/Show、保存取消选择后及新进程；保持旧parent/release/partial门。T-07/T-08维持未完成，原像素覆盖/SD-05/evidence已失效；图/Auto/事务/资源/锁/SDK通过内容未受影响。final失败累计1轮，不触发两轮诊断；不跑整档或创建agent。

2026-10-23：stage: review | review_scope: final | reviewer_identity/limitations: 本聊天Codex独立审查上下文，未创建agent/新聊天，不虚构模型/session身份；重建证据而非采信实施总结 | plan_id: PLAN-837 | plan_revision: 1 | outcome: **needs_fix** | worktree_path: D:/autostack/.wt/lang-837/auto-lang | branch: plan-837-dev | reviewed_commit: abe682c5a1144f7e9b72f4ffeb95ece19dd02841 | base_commit: 52ec4c7c13b337992278b343ef89d8aa17102ff6 | dependency_revisions: SDK9576b4d/原锁/AGP不变，source245/SDK680/closure479/17local/29edges独立核验 | spec_inputs: [完整hash基线](../attachments/837/review-r1/baseline.json.txt)、[冻结规范提案](../attachments/837/review-r1/spec-delta-frozen.md) | acceptance_results: AC01..04/07 pass，AC05/06/08 partial，AC09 fail，AC10 pending | findings: **F-837-R1-01/P2**（原V-10实际合成像素oracle缺失） | evidence: [独立R1与可重放GPU反例](../attachments/837/review-r1/review.md) | pending_closeout_gates: 完整冻结driver/AGP16cases/native/双合成/source/独立consumer、实际落autographics-dev/知识/ledger/归档/到期/guard清理均pending | next: auto-plan-work在原树一次修复/提交/刷新交接后受影响复审。Plan回executing6/10，重开T-07及依赖T-08，T-01..06保留。独立38项定向全绿、check/fmt/source及新层级普通正反门全绿；审查专用副本将真实GPU移动后圆形变透明、并直接断言像素已为背景，冻结原生探针与原聚合脚本仍phase16/204frames/exit0/passed=true，新进程错误图像一致也通过。当前普通产品画面正常，本finding是已约定验收门的漏检，不声称普通路径已发生透明缺陷。实施源码未改，树clean/HEAD未漂移；未合入/修代码/发布canonical/跑整档。此前work pass与其像素声明保留历史，不抵扣本次未通过。

2026-10-23：stage: work | plan_id: PLAN-837 | plan_revision: 1 | outcome: **pass** | worktree_path: D:/autostack/.wt/lang-837/auto-lang | branch: plan-837-dev | base_commit: 52ec4c7c13b337992278b343ef89d8aa17102ff6 | code_commit: abe682c5a1144f7e9b72f4ffeb95ece19dd02841 | dependency_revisions: SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9、原锁/AGP/原游戏/资源/root输入不变 | task_ids: T-01..08 | evidence: [工作交接](../attachments/837/work-r1/readiness.md)、[结构收据](../attachments/837/work-r1/receipt.json.txt)、树内research/hierarchy-{source,feasibility,handoff,spec-delta}.md | blockers: none | next_responsible_role: 独立final review，在同一冻结树继续。当前execution_done8/10，T-09/T-10不勾选。实施写入已停止，worktree保留，不创建agent/新聊天；本轮仅work授权，未自行启动review/merge。

38项唯一工作侧定向测试全部通过、0失败/忽略；之后强化3D原GLB全mesh世界矩阵消费断言，定向1通过，其余实现/依赖/配置/输入未变而复用。check全targets/fmt/source245/SDK680/closure479/17local/29edges及三来源反控通过；独立consumer同源/反控/all-targets编译通过。层级原生正门phase16/204帧、wrong-parent phase5/漏release phase9/frames1 incomplete负控均符合退出和报告；新进程v2重开文档/world/未选中2D+3D像素一致、clean/历史与选择空。旧原生属性门phase9和836 interaction phase27通过。20轮层级事务实际VM弱owner/7 mesh Arc/target身份与warm上传有界，真实保留旧VM/重传mesh源变异各在同一永久断言红，逐字节恢复后绿。SDK146预存warning保持，新增作者warning为0。无开放合同finding、必需工作覆盖gap或产品决定。SD01..05完整提案已准备，canonical/ledger未发布。完整driver/AGP/双游戏/原生/两道合成/独立consumer终档均merge-owned pending，未跑root整档或提前宣称交付；T-10完成前不能合入。

2026-10-23：stage: work；PLAN-837:r1；用户明确授权实施，GSTART满足。G836已重新核实：836 archived10/10、merge-r1完整门/知识/清理收据、autographics-dev实际52ec4c7c13b337992278b343ef89d8aa17102ff6，viewport-interaction已canonical；旧规划pending叙述仅为历史。主检出代码路径无WIP，其他文档WIP保留。唯一树D:/autostack/.wt/lang-837/auto-lang，分支plan-837-dev，base52ec4c7c13b337992278b343ef89d8aa17102ff6，已核root/branch。固定SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9准备680文件，verify_source227源码/资源与479依赖闭包/17本地包/29边通过，source-drift/missing-fixture/missing-GLB三个反控拒绝并恢复正控。T-01实施冻结记录准备中，下一步T-02有界whole-path验证；review/merge整档保持pending。技能来源D:/autostack/auto-musk/.agents/skills/auto-plan-work/SKILL.md，未创建agent。

2026-10-23：stage: new；PLAN-837 revision1；outcome: blocked（实施前置G836未完成，草案静态交接就绪）；next: 836 merge owner完成T-10后，837 work从T-01/T-02核基线及可行性。用户只请求规划，本轮没有启动work/merge或创建树/agent。

规划历史：当时drafting0/10；当时T-01..10、AC-01..10、V-01..11、SD-01..05均为新合同，未有执行pass。规划preflight见[检查记录](../attachments/837/planning/preflight.md)，进度分析不作为生产交付证明。final review未开始，merge-owned完整门pending；两轮同因不收敛时保留轮次并先记录根因/漏测/兄弟路径，再继续修复。

## 10. 待澄清事项

当前无开放finding/工作侧必需覆盖gap/产品决定。F-837-R1-01/P2已由final R2在2ffc92d34独立确认CLOSED，无新增阻断；T-01..09完成，T-10完整门及实际合入收口仍pending。原R1 needs_fix和work修复交接保留历史，以R2为当前审查结论。首版同维平移组、keep-world位置、v1导入/v2保存、共享128节点预算及不含生产集成已明确。若要完整TRS/剪切、跨维/多父/Prefab、任意Portal/UI树、改SDK/AGP/生产仓或改变数值/失败/资源保证，回new提出具体范围变更；架构扩面按L2先补正式设计。
