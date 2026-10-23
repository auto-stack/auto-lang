---
plan_id: PLAN-836
status: archived
completion_kind: delivered
feature_name: AutoGraphics视口选择与平移交互原型
plan_revision: 1
author: [Codex]
created_at: 2026-10-23
updated_at: 2026-10-23
current_step: 9
total_steps: 10
depends_on: [PLAN-835]
supersedes_spec_components:
  - docs/specs/auto-graphics/authoring-session.md
  - docs/specs/auto-graphics/scene-document.md
  - docs/specs/auto-graphics/prototype-profile.md
  - docs/specs/auto-graphics/integration-contract.md
  - docs/specs/auto-graphics/project.md
new_spec_components:
  - docs/specs/auto-graphics/viewport-interaction.md
touched_goals: [GOAL-007, GOAL-008]
affects: [auto-graphics, authoring-experimental]
---

# [PLAN-836] AutoGraphics 视口选择与平移交互原型

## 0. 变更摘要

835已完成final R2、冻结合入门、归档、清理及批量回归收尾。本Plan在其独立创作窗口上补齐下一段用户路径：**点击实际画面选对象 → 2D直接拖动／3D世界轴平移手柄 → 真实Auto预览 → 一次撤销／重做 → 保存重开**。

这是AutoForge前置的有限交互原型。final R2及完整冻结门通过，源码已快进合入`autographics-dev`的52ec4c7c1；master仅接收canonical/ledger/归档证据。F-836-R1-01已关闭；当前archived9/10，T-10仅guard清理待完成；835前置已满足。源观察和实施冻结分别记录，见§2及[来源清单](../attachments/836/planning/source-observation.json.txt)。

## 1. 目标

1. 视口、对象列表和属性面板选择同一持久ID；点击画布空白清除选择，选择不进入文档历史、不改变dirty。
2. 2D circle/quad可直接拖动；3D cylinder/player/mob可点击选择并通过X/Y/Z世界轴手柄平移。拖动候选必须经过真实Auto profile3，不直接修改Rust矩阵冒充Auto预览。
3. 一次成功拖动只提交一个编辑事务；取消、失焦、过期事件、非法坐标和预览故障不污染历史或保存内容，随后合法操作可恢复。
4. 复用835的场景schema、有限对象、保存与历史合同；给出原生事件、画面、状态及资源证据，推进独立创作可用性。

支持面保持Windows x64、单窗口键鼠、stock Iced/wgpu、两个固定预览、128对象、64步／16MiB历史、1MiB文件。2D不支持旋转；3D只平移，既有yaw/pitch/scale仍可通过属性面板编辑。

非目标：生产AutoUI/RQHost/Forge仓迁入、任意嵌套编辑、层级/reparent、多选/框选、旋转缩放gizmo、相机漫游、网格吸附、三角形精确拾取、资源/材质导入、游戏玩法编辑、AI/源码生成、AutoShader/CAP/ACE或新平台。本Plan不扩SDK和生产语言接口。

## 2. 架构方案

```text
winit真实按下/移动/释放/取消
  → CompositionInput实际Surface/local/捕获路径
  → 已呈现场景pick snapshot（身份、几何、相机、viewport）
  → 选择 / Gesture（起点、epoch/revision/target/gesture token）
  → 有界候选文档 → 真实Auto profile3 → 输出准入/资源准备
  → 临时预览（accepted文档/dirty/历史不变）
  → 释放：最终端点成功才一次提交；取消/失败：恢复accepted预览
  → 既有Undo/Redo/Save/Open
```

写面为`experimental/autographics-auto-driver/`，延续bin `autographics-authoring`；AGP只读消费，不另建编辑器框架或workspace。新增driver-local `src/authoring/interaction.rs`及定向测试；必要投影辅助可放同目录新小模块。持久Document不含gizmo、相机、指针或运行时ID。

| 门 | 当前事实与执行条件 | owner_stage |
|---|---|---|
| G835 | 已满足：835 archived 10/10；final R2审查`0c29fa2a7cfb0dbc08a906542c511bd20f34ebed`，正式落点`2a7b10250245332f46b7867dc140da1a4f9d0969`；见[合入收据](../attachments/835/merge-r1/receipt.md)及[批量收据](../attachments/835/batch-r1/receipt.md) | work核实 |
| GSOURCE | 本轮观察上述autographics-dev完整SHA；SDK仍为`9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`。T-01核当前合同，固定源码/锁/资源/prepare输入并写实施冻结收据；分支变化先判明相关差异 | work |
| GSTART | 用户本轮授权规划，未要求启动实施；本次停在drafting交接 | work启动前 |

唯一未来实施树`D:/autostack/.wt/lang-836/auto-lang`，分支`plan-836-dev`，从通过GSOURCE的**明确autographics-dev SHA**创建，不能使用默认master基面捷径。实施、internal check与final review顺序使用同树，主检出维护Plan/附件；合入目标仍为autographics-dev，master只沉淀知识。树内不建junction/symlink，移除前跑既有wt-guard。

## 3. 技术栈

沿用835锁定的Rust、stock Iced 0.14、winit 0.30.13、wgpu 27、glam 0.34、serde与原始GLB。复用公开`CompositionInput::{pointer_pressed,pointer_moved,pointer_released,cancel_all}`、`Scene3dContent::view_proj`、`ModelData/MeshData::positions`、`MeshStore`及profile3 `Preview/Building`；不改profile1/2、SDK或根workspace/lock。

拾取在CPU上完成，规模为固定资源和最多128对象，无需GPU ID pass、BVH或物理库。3D采用各原模型**全部mesh顶点合并的模型空间AABB，经当前TRS成为OBB**；允许包围盒空隙命中，UI帮助/Spec须说明，不能声称像素/三角形准确。

## 4. 需求分析与背景调查

### 4.1 授权、现行知识与来源

2026-10-23用户说明“计划835实施完成，请规划下一个计划”。授权为调查和创建下一Plan；未指定模型/审核人、预算、新聊天或实施。选择本切片依据既有AutoForge可选择/编辑/实时预览/保存/撤销目标，不预排多个未来月份。

已读取[全局overview](../../specs/overview.md)、[图形overview](../../specs/auto-graphics/overview.md)、[project](../../specs/auto-graphics/project.md)、[authoring-session](../../specs/auto-graphics/authoring-session.md)、[scene-document](../../specs/auto-graphics/scene-document.md)、[输入](../../specs/auto-graphics/nested-input.md)、[profile](../../specs/auto-graphics/prototype-profile.md)、[集成合同](../../specs/auto-graphics/integration-contract.md)，以及[835归档](835-autographics-scene-authoring-save-and-undo.md)/[R2](../attachments/835/review-r2/review.md)/正式收据。源码事实绑定Git对象，主检出Spec和[创作策略](../../design/strategy/autographics-forge-v06.md)当前字节另记SHA256；其他会话未提交内容不算本Plan交付。

### 4.2 已观察的接口和真实缺口

| 来源（autographics-dev观察SHA） | 已观察事实 | 本Plan处理 |
|---|---|---|
| driver `src/authoring/host.rs::Host::{click,prepare,publish,frame}` | click只给Iced模拟按下/释放，画布仅blur；prepare用真实Auto输出构造DrawCmd和原3D全部mesh | 接真实事件生命周期和对象拾取，保留控件回路 |
| driver `src/bin/autographics-authoring.rs::App::window_event` | CursorMoved仅存位置，左键仅处理Pressed，释放未接Host | 接down/move/up/cancel，不用即时click代替拖动 |
| driver `session.rs::Editor::{begin,pump,commit,reject}` / `preview.rs::Building` | accepted事务、历史、每候选新VM、8ms pump已存在 | 增加有限gesture候选生命周期，不逐采样记history |
| driver `host.rs::prepare` | 2D中心[400+x,190-y]；3D为T*Ry*Rx*S；3D选框是中心附近固定屏幕框 | 从实际呈现数据建快照；选框改为代理边界投影，不靠旧框证明拾取 |
| AGP `render3d/camera.rs` / `Scene3dContent` | 创作相机为固定正交sample_camera(2.0)，NDC深度0..1 | 使用呈现view_proj及child viewport，不另造常量相机 |
| AGP `host/composition_input.rs::ray_to_plane` | helper面向Portal平面，以-1/1反投影，不是模型拾取接口 | 不照搬其深度约定；driver-local射线按wgpu0..1验证，不顺手修改旧AGP |
| AGP `assets/glb.rs` / `render3d/pipeline.rs::MeshStore` | 节点变换已烘焙，全mesh CPU positions可读，固定资源有GPU身份缓存 | 全mesh生成bounds，拖动不重上传同一模型 |
| 835 Spec/收据 | schema/失败/保存已交付，拾取/gizmo未覆盖 | 不把835的pass外推为新交互证据 |

816仍executing 6/14，831/832为带启动锁的drafting r2。本Plan不进入它们的准备/写面；I2/I3、Linux、Shader/CAP仍受各自前置约束。全局overview含历史快照，旧图形段落的pending以835正式交付段和收据为准；合入时只更新本次实际增量。

## 5. 详细设计

### 5.1 下一执行入口与有界可行性探针

依赖就绪入口为T-01，然后T-02；T-03..08不可跳过探针。835开放任务为零；836当前T-01..09完成，final R2已通过；T-10的完整冻结门/源码合入/知识沉淀已完成，仅guard清理待完成。执行者先读主检出合同/附件，确认唯一树、基面/SDK/锁/资产；旧基面不带本Plan不构成另建合同副本的理由。

T-02在该树建立`research/viewport-feasibility.md`（新）：一个2D拖动和一个原3D模型轴移，走真实OS事件→实际Surface/local→真实Auto候选→可见画面→release一条历史→undo。另做取消和最终端点失败反例，证明临时呈现可恢复且不会误提交。

最多比较两种driver-local候选/准备接法，固定正交相机，不探索完整gizmo库。记录坐标系、全mesh bounds、候选owner、提交点、编译/pump/呈现时延及资源复用方式。若需新SDK/AGP能力、生产UI或无法保住文档/预览/历史一致性，给确切缺口回new修订，后续相关任务不开始；不能降为release后才更新、仅Rust预览或不支持取消。

### 5.2 拾取与坐标合同

按`CompositionInput`实际命中Surface/local，再查询该空间对象。初次按下必须在有效viewport/clip内；面板、视口间隙、窗外、非有限坐标不能拾取对象。零尺寸/最小化不除零、不开始手势。使用当前**已呈现**快照，包含epoch/revision、呈现序号、稳定ID、Auto受检属性、模型身份、viewport、实际相机；文档替换或身份不匹配不命中。

2D仅visible且alpha>0参与；quad用真实矩形，circle按DrawCmd尺寸对应椭圆内部判定，不能用外接矩形角落选中。按实际对象绘制顺序后画者优先，选框/手柄不算普通对象。平移local dx→world+x、local dy→world-y，从按下起点算绝对差，避免累计漂移。

3D对cylinder/player/mob全部mesh positions求联合AABB，使用与renderer相同的T*Ry*Rx*S。child像素反投影NDC z=0/1得到近远有限线段，变换至模型空间与代理相交，以统一世界线段参数取最近命中，排除near/far外和隐藏对象；同深度平局按稳定ID升序。不能只用中心、首mesh或OBB投影矩形。相机/逆矩阵退化或数值不可用时受控拒绝，不选择任意对象。

3D选中显示代理投影轮廓及标色X/Y/Z手柄；覆盖层不写Document。只对已选且处于可编辑投影范围的对象显示/命中，手柄优先于普通对象；交叠规则由T-02固定为可复现距离/优先序。屏幕投影轴过短/退化或origin不在有效裁剪范围时禁用该手柄并提示，仍可用属性面板；不能把无效计算伪装为零位移成功。

### 5.3 手势事务、临时预览和一次提交

状态Idle→Armed→Dragging→Finishing→Idle，取消/故障从活动态回Idle。绑定epoch、起始revision、稳定target ID、起始对象/指针、Surface/捕获路径、viewport/camera版本及唯一gesture token。2D对象按下后移动≥3 child逻辑像素启动；3D只有手柄按下可平移，点击模型只选择。未越阈值、最终回到起始内容的noop不提交历史。

2D自由XY；3D世界X/Y/Z单轴，起始相机投影轴与pointer位移反解标量，其他轴、yaw/pitch/scale不变。固定正交profile可用投影单位轴点积反解；具体手柄尺寸/命中带/数值阈值由T-02记实，T-03锁定测试，不引入相机运动或透视支持承诺。

accepted Document、saved hash、history、revision在拖动中不变。每个呈现候选由完整候选Document经过真实Auto `Building`和输出/资源准入，成功才更新临时画面；面板明确预览状态，不显示为已提交/已保存。释放必须消费**最后端点**，准备成功后一次发布accepted文档、真实预览、history和revision；旧ready端点不能在最终端点失败后偷偷提交。重复release去重，结果检查gesture及sample token。

仅一个inflight候选和一个最新待处理坐标，旧move合并；最多一份accepted预览、一份已显示临时预览、一份inflight候选及其有界文档。不得逐鼠标事件排VM/线程/文档队列；inflight完成可发布该临时帧再取最新坐标，但呈现sample序号只能前进，迟到旧结果不得覆盖新帧。release后等待最终端点的有界终态，不能无限重试。沿用8ms pump和原Session累计预算；同步编译耗时单独报告，不声称可抢占或每帧满足8ms。

Auto/输出/资源失败：取消整个手势、恢复accepted预览，doc/history/revision/saved hash不变，显示具体错误；下一合法手势须成功。accepted预览owner保留到结束，取消不依赖重新编译/上传模型才恢复。禁止Rust实例先移动、Auto回填后才核对的旁路。

### 5.4 捕获、焦点、取消及命令互斥

接真实down/move/up，不复用835画布按下即释放快捷路径。捕获后跨同窗口面板/另一视口仍只动原目标，UI不吃同一拖动。离窗、失焦、Esc、最小化/零尺寸、resize/DPI/相机版本变化、目标/文档失效均取消并释放捕获；恢复后旧move/up/Auto结果不能碰新对象。

Idle下列表/画布选择同步，换选丢弃旧属性draft并释放旧文本焦点；纯选择不改内容。Armed/Dragging/Finishing中UI明确busy，其他选择、Apply、Create/Delete、Undo/Redo、Save/Open/New拒绝且不排迟到命令；关闭先取消，再按accepted内容走835 Save/Discard/Cancel。文本焦点下未按中画布/手柄时键入不触发平移；合法画布按下可转焦并清旧draft。不向游戏透传键。pending事务未结束不开始第二手势；无先行down的up/move、非主键、旧epoch/revision/selection/token不提交。多mount/报告消费不得重复操作。

### 5.5 资源、持久化与边界

`.agscene` schema `autographics.scene/1`和835所有范围/拒绝规则不变，无新增持久字段。代理/手柄/选择/捕获/样本/临时预览不保存；Save仅Idle且既有事务完成可用，只存accepted内容。成功手势进入一条before/after历史；Undo/Redo精确恢复，dirty按canonical hash；重开历史空/选择空，内容/画面为保存版本。

同一固定资源的拖动只更新对象/覆盖层数据，warm后模型vertex/index不重上传，不新建pipeline/target；可在Host建立时一次准备固定资源，保持身份和同device准入。候选失败不清空accepted资源。临时VM按实际Drop/弱owner、mesh pin验证释放，不能以history长度/自增计数代替。20轮成功/取消/故障手势后同时活owner满足§5.3上限，关闭无非授权残留；合法外部pin在最后owner释放后才消失。记录真实上传/保留内存及排除Iced内部/驱动/GPU timestamp的边界。

不改835保存协议，不顺手修旧AGP mapper/生产UI，不更新golden/原资源来过门。I1双游戏保持，I2生产AutoUI/RQHost、I3正式Forge/发行仍pending，不允许graphics源码入master。

### 规范增量

以下均拟议，当前不写canonical；review冻结实际措辞，merge再沉淀并更新overview/plans/ledger。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/viewport-interaction.md | 无对象拾取/手势→2D几何、3D OBB、轴移、坐标/取消及精度支持面 | 可消费交互边界 | AC-02, AC-03, AC-05, AC-07 |
| SD-02 | modify | docs/specs/auto-graphics/authoring-session.md | 离散编辑→有界真实Auto临时预览、release一次提交和失败恢复；列表保留 | 不破坏历史/dirty/焦点 | AC-04, AC-05, AC-06, AC-07 |
| SD-03 | modify | docs/specs/auto-graphics/scene-document.md | schema1→schema/持久字段不变，明确gesture不保存和accepted保存 | 旧文件兼容 | AC-06, AC-07 |
| SD-04 | modify | docs/specs/auto-graphics/prototype-profile.md; docs/specs/auto-graphics/integration-contract.md; docs/specs/auto-graphics/project.md | 835独立创作→836实测支持/限制/入口，保留I2/I3及合分支门 | 交接准确 | AC-01, AC-07, AC-09 |

## 6. 测试设计

新入口/fixture由对应work任务建立，以下不是已跑证据。用户未指定实施/内部检查/最终审核人，不创建聊天或指定模型；默认final独立review在同树、独立上下文。实施会话自查须披露上下文局限，不冒充另指定审核人。review时暂停实施写入，修复返回同树。

以下命令在未来836树的`experimental/autographics-auto-driver`运行。T-01用已有`python tools/prepare_sdk.py --repository D:/autostack/auto-lang`、`python tools/verify_source.py`核冻结来源，不换更新SDK。开发定向：`cargo check --locked --all-targets`、`cargo test --locked --test authoring_interaction -- --nocapture --test-threads=1`（新，T-03..07建立）、835原authoring_host/session/storage/document受影响测试。新增原生模式拟为`autographics-authoring --interaction-probe --report <path>`，T-07建立；原`--window-probe`、`--inspect`兼容。复用既有OS事件设施，脚本在本workspace tools/research。

| case_id | AC / invariant；work owner | 代表输入/状态 | 入口及区分错误实现的observable oracle | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01；T-01/08 | 固定SHA/SDK/锁/资源；源漂移/缺资产反控 | 原verify_source、打包独立consumer实际closure/hash正确，负控拒绝；双游戏golden及834/835入口保持 | 独立Windows实验，非生产迁移 |
| V-02 | AC-02；T-03 | 重叠2D、circle外框角、隐藏/alpha0、同名不同ID、空白/clip外 | 新authoring_interaction穿Host路由；后画者ID正确，circle角miss可选下层，空白清空，doc/hash/history不变；错坐标/错ID反控红 | 矩形/椭圆，非任意path |
| V-03 | AC-03；T-03/05 | 全mesh GLB、旋转/非等比缩放、近远重叠、代理空隙、near/far外、退化矩阵 | 实际view_proj/viewport→OBB；具体ID/统一t/all-mesh bounds与独立几何oracle一致；z=-1替换/首mesh/中心近似至少一条变异被同门拒绝；空隙按proxy命中 | 当前正交，非三角形精度 |
| V-04 | AC-04；T-04/05 | 2D固定dx/dy、3D分别X/Y/Z、多move/release、回起点、重复release | Host→Auto→GPU/属性；固定尺度1e-3世界单位容差，非目标字段/同名B精确不变；成功revision/history各+1；noop/重复up不加；一次undo/redo回对应hash | 支持手势；不得为过门放宽容差 |
| V-05 | AC-04/05；T-04/06 | 慢Auto高频move、旧sample、release最后端点失败、资源错→合法重试 | 实际Building/资源故障；Auto输出变异或Render旁路反控红；最终失败恢复accepted pixels/hash/history/revision；1inflight+1latest，不提交旧端点；合法重试成功 | trusted源码预算，compile另计 |
| V-06 | AC-05；T-06 | 捕获跨面板/另视口，Esc/leave/失焦/resize/DPI/minimize/close，旧up/新epoch、文字draft | Host事件；跨视口只动原target；取消恢复且无提交，旧事件无效，文本/busy隔离，close走原dirty决策；OS补V-09 | 单窗主指针，非窗外OS捕获 |
| V-07 | AC-06；T-06/07 | 拖动/取消/undo/redo/Save，835文件重开，活动手势Save | Storage+新进程--inspect；文件仅schema1原字段，hash/对象/GPU画面一致，历史空/选择空；取消/活动Save不改旧文件，dirty正确 | 本地同卷，不重验无关网络/掉电 |
| V-08 | AC-06；T-04/08 | warm20轮成功/取消/故障/恢复，合法pin和关闭 | 实际VM弱owner、mesh/target/pipeline/上传计量；活owner有界，无每move模型重传/target/pipeline新建，last-owner释放；保留候选/故意重传负控拒绝 | 明示Iced/驱动排除，无GPU timestamp |
| V-09 | AC-02..06；T-07/08 | 原生down/move/up，2D/3D每轴、选择/undo/redo/cancel/save、窗口缩放/机器DPI | 新interaction-probe经OS→winit→router→Auto→composite capture；实际target/端点/帧/hash，不能直接dispatch手势；wrong-target和drop-release负控均红；保留两道原合成门 | 必须真实桌面，环境不可用保留gap |
| V-10 | AC-07；T-08 | SD-01..04与实际CLI/资源/限制 | 拟议Spec diff/readme/research可复跑、版本对应，不冒认生产I2/I3或完整引擎 | 静态检查不代V-02..09 |

每个case记录command/退出码、基面/冻结提交、SDK/锁/输入指纹、实际断言值、原始报告及限制。总数绿、capture hash变化或选择变量变化不能单独证明目标/画面；V-09还核最终合成及具体对象位置/列表属性；counter和content门并列保留。

开发/修复先完成全部findings，仅定向测试/check/fmt；不跑root整档t/tv/tt/tb/tu/taa/docs_gen，也不每次review跑完整driver/AGP。final review后，merge在同树冻结提交统一跑完整driver/AGP test+build、既有16 case、双游戏/835原生门、新V-09正反门、两道合成门、来源/资产/独立consumer门。T-08据现有README登记准确命令和选面，0selected/skip不通过；复用证据须证明代码/依赖/配置/输入未变。若需生产Rust改动先回new界定范围及root门，不自动扩写面。

## 7. 验收标准

| ID | owner_stage | 可观察标准与方法 |
|---|---|---|
| AC-01 | work | G835/GSOURCE、唯一树/来源成立，写面仅driver，SDK/profile1/2/AGP/root输入不漂移；V-01 |
| AC-02 | work | 2D画布几何/顺序/选择同步、空白/clip/DPI/焦点正确，选择不改history/dirty；V-02/09 |
| AC-03 | work | 全mesh OBB近远/变换/投影一致，三轴手柄可操作、精度公开；V-03/04/09 |
| AC-04 | work | 真实Auto临时画面、最后端点单次提交、undo/redo、合并有界、失败恢复成立；V-04/05 |
| AC-05 | work | 捕获/互斥/取消/旧身份及文本焦点隔离，失败后恢复，真实OS正负门成立；V-05/06/09 |
| AC-06 | work | schema1和accepted保存重开/dirty正确，warm复用及候选last-owner有界释放；V-07/08/09 |
| AC-07 | work | SD-01..04、研究结论/支持面/入口/计量/可复跑交接完整；V-10和work证据 |
| AC-08 | review | 独立核AC-01..07及Spec，扫描遗漏/延后/workaround/fmt/warnings/debug；版本绑定pass、merge门显式pending |
| AC-09 | merge | 最终冻结完整门通过后实际合autographics-dev，canonical/ledger/归档/guard清理完成，按规约检查批量到期；源码不入master |

## 8. 执行步骤

当前archived9/10；final R2冻结5f3e218ba，delivery52ec4c7c1及完整门/实际合入/知识/ledger/归档均已验证；T-10仅清理收尾pending。work T-01..08完成且无in-contract finding才execution_done（8/10）；T-09通过为reviewed（9/10，merge门pending）；T-10实际完成才归档交付10/10。

| task_id | owner_stage | 依赖 | 已验证触点／新增路径与动作 | 可执行关闭条件；AC |
|---|---|---|---|---|
| T-01 | work | GSTART/G835 | 主检出合同/附件、836树，已有tools/prepare_sdk.py和verify_source.py、锁/源锁 | [x] 完整基面/SDK/资源hash，prepare/verify及来源反控，research实施冻结清单；AC-01；证据见§9 work-final |
| T-02 | work | T-01 | host/preview/session/bin；新research/viewport-feasibility.md | [x] §5.1真实whole-path/取消/最后失败反例可跑，坐标/owner/手柄/时延/资源决策闭合，否则回new；AC-01/03/04/05；证据见§9 work-final |
| T-03 | work | T-02 | 新src/authoring/interaction.rs、新tests/authoring_interaction.rs，mod.rs/Host快照 | [x] V-02/03定向通过，独立几何expected、深度/首mesh等反控和实际dataflow可查；AC-02/03；证据见§9 work-final |
| T-04 | work | T-03 | session.rs/preview.rs/host.rs及新增gesture类型 | [x] V-04/05、原authoring_session定向通过，token/合并/Auto临时/最后端点/一次history成立；AC-04/06；证据见§9 work-final |
| T-05 | work | T-04 | interaction/host.rs覆盖层、2D拖动、三轴、公开MeshStore | [x] V-04每轴/非目标字段、代理轮廓/手柄命中通过，warm mesh复用；AC-03/04/06；证据见§9 work-final |
| T-06 | work | T-05 | bin::window_event、Host::key/blur/resize/dispatch/request_close、Panel；Storage协议不改 | [x] V-06及原host/storage受影响定向通过，busy/cancel/焦点/文件正确；AC-02/05/06；证据见§9 work-final |
| T-07 | work | T-06 | 新interaction-probe及tools/research回放，原window-probe/inspect | [x] F-836-R1-01修复：所有self-check事件循环未完成出口统一非零，报告提前写最终错误；永久截断/观察/启动失败与完整原生正负门、部分帧、新进程重开通过；AC-02..06；见§9 work-repair-r1 |
| T-08 | work | T-07 | 相关测试、README、新research/viewport-handoff.md、树内拟议Spec | [x] 修复冻结5f3e218ba；来源227/SDK680/closure479、反控及独立打包check通过，fmt/check及SD04/交接已刷新；AC-01/06/07；见§9 work-repair-r1 |
| T-09 | review | T-08 | 同树独立审查冻结code/Plan修订/Spec，主检出review附件 | [x] final R2 pass，F-836-R1-01 CLOSED、无新增finding；AC-01..07及SD复核通过，版本绑定5f3e218ba，完整merge-owned门显式pending；AC-08；见[review-r2](../attachments/836/review-r2/review.md) |
| T-10 | merge | T-09 pass | 冻结完整门→autographics-dev；主检出知识/ledger/archive→guard清理 | [ ] 完整门/落点/知识/ledger/归档通过；guard清理和批量到期检查待完成，见§9 merge收据；AC-09 |

## 9. 复审记录

2026-10-23：stage: merge；PLAN-836:r1；checkpoint prepared/landed/ledger_refreshed/archived，cleanup pending。review5f3e218ba，文档/台账后继delivery52ec4c7c13b337992278b343ef89d8aa17102ff6，implementation/dependencies精确不变。完整driver69/AGP162全测零失败忽略，构建、16AGP场景、双游戏/835原生正负、新interaction正负/截断/观察/保存重开、counter与content正反控、来源227/SDK680/closure479及外部consumer全链通过。autographics-dev从2a7b10250本地no-force FF到交付提交；master无graphics源码。canonical SD01..04及索引已读回；离线guarded ledger72→73，9相关条目、1146无关raw条目保留。所有原始失败和等输入重试边界保留。详[合入收据](../attachments/836/merge-r1/receipt.md)、[冻结核验](../attachments/836/merge-r1/prepared.json.txt)。归档提交后再guard清理；当前不宣称T-10完成。

2026-10-23：stage: review；review_scope: final（R2受影响复审）；reviewer_identity: Codex，沿用R1独立上下文，未参与实施修复、未委派agent/model；PLAN-836 revision1；outcome: **pass**。唯一树D:/autostack/.wt/lang-836/auto-lang / plan-836-dev，reviewed_commit `5f3e218ba5d605cfb3e007d1c9bade5a5fe399a1`，总体base及autographics-dev `2a7b10250245332f46b7867dc140da1a4f9d0969`，repair_diff_base87fb0b051，SDK9576b4d；身份/Spec/依赖/二进制见[baseline](../attachments/836/review-r2/baseline.json)。**F-836-R1-01/P2 CLOSED，无新增finding**。fresh永久2单测+1CLI/7子进程通过；原3frame1、legacy、观察与Host前失败均断言退出/report一致；独立完整native exit0/passed=true/phase27/456frames/6records，wrong-target原phase1非零、drop-release原phase4非零；部分frames52在phase3/1record非零且incomplete，组合模式截断同样非零。新进程重开保存hash完全一致、history空/selection空/dirty=false。227来源/SDK680/closure479/local17/edges29、fmt/check/diff通过；SDK146既有warnings无新增作者warning。Host/几何/历史/存储/样本/AGP/依赖/配置及canonical Spec字节未变，R1的22库定向+4独立holdout仅在对应范围复用；新binary/fixture采用本轮完整重放。夹具只在起始Idle期准备焦点/指针，活动取消/原oracle未削弱。SD01..04重新冻结：raw SHA256 `f61a6de1e419ab4357f985f03ac6e0eef168029e797d9c2bc1da4905fc7647fd`、LF SHA256 `259137cc772613eca30db99db5b458eda5616215f765992bba5f8b0d2acb8ecf`，SD04为原no-skip澄清、revision不变，canonical/ledger未写。T-07/T-08确认完成、T-09通过，status reviewed/current_step9/10；完整driver/AGP/原16case/双游戏与835原生/新interaction正负与reopen/两道合成/source-consumer等merge-owned门pending，T-10未执行、未合入/归档/清树。详[完整R2报告/证据/复用边界/后续门](../attachments/836/review-r2/review.md)。next: merge在同树冻结提交先跑完整门再落autographics-dev，源码不入master。

2026-10-23：stage: work（R1 needs_fix修复）；PLAN-836 revision1；outcome: pass；status execution_done8/10。同树plan-836-dev，baseline87fb0b051，修复提交 `5f3e218ba5d605cfb3e007d1c9bade5a5fe399a1`，SDK9576b4d；写面限native bin/fixture/永久CLI回归和文档/身份，Host/session/storage/geometry/Auto样本、AGP/原资源/锁未改。**F-836-R1-01已闭合**：[finding→不变量→兄弟出口→实际断言/结果/边界](../attachments/836/work-repair-r1/viewport-repair-r1.md)，[身份及readiness收据](../attachments/836/work-repair-r1/receipt.json.txt)。原3反例先复现exit0/false/null/0records，永久CLI先红后绿；2完成性单测+1CLI测试含7子进程通过，涵盖三种frame1截断、legacy截断、观察/普通窗合法有界退出、Host前初始化失败，并审查全部正常提前返回的共同守卫。完成性在report前统一判定；旧错误保留。完整native r3 exit0/passed=true/phase27/6required records，wrong-target原phase1及drop-release原phase4均exit1；部分执行frames52在phase3/1record时exit1且报告incomplete；新进程重开hash/空history/空selection/clean通过。两次完整正门失败保留：r2日志确认down后真实CursorLeft取消；精确OS来源仅推断。夹具改为Idle期准备自身焦点/指针并等待一轮事件消化，不屏蔽任何生产取消，r3才计通过。source227/SDK680/closure479/17local/29edges及源码/fixture/原GLB反控通过；独立新打包副本同门及all-targets check通过，fmt/check通过，SDK146既有warnings无新增作者warning。R1的22定向+4holdout仅在不变库输入范围复用，不替代新binary证据。T-07/T-08重新完成；work finding/必需gap/待决语义为空；[SD04澄清后提案](../attachments/836/work-repair-r1/viewport-spec-delta.md)已刷新，需final重新冻结。不是最终审查pass；T-09/T-10及完整merge-owned门仍pending，未跑整档/合入/沉淀/归档。树已提交clean并暂停实施写入，next responsible role: review，同树受影响复审。

2026-10-23：stage: review；review_scope: final；reviewer_identity: Codex独立审查上下文（本聊天未参与实施；无子agent/另模型声明）；PLAN-836 revision1；outcome: **needs_fix**。同树D:/autostack/.wt/lang-836/auto-lang / plan-836-dev，reviewed_commit `87fb0b051d8ab9b17fba0fcfe2580c774a79d058`，base及autographics-dev `2a7b10250245332f46b7867dc140da1a4f9d0969`；SDK9576b4d/锁/来源/Spec输入指纹见[baseline](../attachments/836/review-r1/baseline.json)。**F-836-R1-01/P2**：新self-check模式提前结束没有完成性保护，普通/错目标/漏release三个`--interaction-check --frames 1`都exit0、passed=false、0records，旧window-probe同设置exit1。原合同AC-05/07、V-09/10未闭；T-07及依赖T-08重开，T-01..06保持，current_step6/10、status executing，T-09未通过。22定向+4独立holdout通过；来源225/SDK680/closure479、fmt/check通过（SDK146既有warning，无新增作者warning）。实际物理鼠标dx40/dy20及一次Undo通过；新进程保存重开hash/空history/空selection/clean通过。完整自动原生正门本轮受额外桌面输入干扰未闭，原始失败保留，不冒称产品取消回归或pass。Spec delta SHA256 `76b1201dde4b2088cd78431eeb3629316e3b8daccf76a3d2254364712025d84f`已冻结，canonical/ledger未改；完整merge-owned门pending，未合入/归档/清树。详[复审、反例及关闭oracle](../attachments/836/review-r1/review.md)。next: work在原树修F01并提交、刷新T07/T08，受影响复审；本轮不修实施。

2026-10-23：stage: work；PLAN-836 revision1；outcome: pass；status execution_done，current_step8/10。唯一树D:/autostack/.wt/lang-836/auto-lang / plan-836-dev；base2a7b10250，code_commit87fb0b051d8ab9b17fba0fcfe2580c774a79d058，SDK9576b4d，AGP/原资源/锁/root生产源码不变。T-01..08及work-owned AC-01..07完成；[结构化AC/身份/交接收据](../attachments/836/work-final/receipt.json.txt)、[完整交接](../attachments/836/work-final/viewport-handoff.md)、[拟议规范](../attachments/836/work-final/viewport-spec-delta.md)。22定向测试+2原游戏数值回放通过，fmt/check通过；独立打包副本同225来源/680SDK/479closure/17local/29edges及check通过。实际原生中间帧/accepted不变、最终端点、Undo/Redo/Esc、原GLB各轴、最终失败恢复、Save/新进程reopen、resize/2xDPI路径正门exit0；wrong-target和drop-release同门负控exit1。几何NDC/首mesh、候选保留/重复模型上传实际源码变异均达到断言并红，恢复正控绿；初始未达到测试的构建失败不计证据。20轮真实VM弱owner/外部GPU pin/上传及Arc身份验证释放与复用。未完成finding/所需work覆盖gap/待决支持语义均为空；这是实施会话自查，不冒充独立最终复审。T-09/AC-08及T-10/AC-09明确pending，未跑root全档或完整driver/AGP合入档，未合入/归档/发布canonical或ledger。实施树已提交clean、保留并暂停写入；next responsible role: review，同树冻结基面逐项复核。

2026-10-23：T-02原生授权已通过；实际2D拖动/Undo和原GLB worldY手柄成功，原生释放资源失败与取消边界注入保持accepted坐标/dirty/history，下一合法拖动恢复。[本轮收据与截图/事件](../attachments/836/work-t02-r2/receipt.md)明示原生与Host补充证据边界，T-06实际Esc/失焦矩阵及T-07完整正负门未外推为通过。T-02有界可行性完成，当前2/10；继续T-03拾取及路由反控，状态executing，无环境阻塞。下述窗口授权blocked为已关闭历史。

2026-10-23：stage: work；revision1；outcome: blocked（T-02原生窗口授权）。同一836树/plan-836-dev，code_commit 1c4fb0ac0f038a13f4ecdfbf39cbc268a5a8dced，SDK/AGP/锁/原资源未变。T-01已完成；T-02探针原生down/move/up与有界真实Auto临时预览已接，最后端点单提交/取消恢复及原GLB三轴Host路径探针通过。host7+session4+probe1=12/12定向通过；all-targets check/fmt通过；219来源哈希和原源码漂移/缺fixture反控通过。SDK146条既有warning保留，无新增author warning。固定资源一次预热7mesh，原计量断言6→7有明确资源解释，上传遗漏反控保留。Computer Use发现唯一正确窗口，但激活/取状态返回Computer Use app approval timed out；无真实OS拖动证据，idle报告不算pass。T-02不勾选，T-03..08未开始验收，当前1/10，保持executing。主检出[本轮日志/交接](../attachments/836/work-t02/handoff.json.txt)及[探针结论](../attachments/836/work-t02/viewport-feasibility.md)；研究结论和代码已提交，实施树clean并保留。next: work，在允许访问AutoGraphics Authoring窗口后继续T-02原生2D/3D拖动、撤销、取消和最终失败反例；不能跳过此门。review/merge整档门pending，未运行root全档。

2026-10-23 work续行：T-01完成，当前1/10。混合换行已精确重构：前11行LF、末行CRLF得到原expected哈希；pathlib对.gitignore的suffix为空，原归一化未生效。提交3d04721d3修复点文件识别和对应指纹。verify_source通过216源码/资源、680 SDK文件、479依赖闭包、17本地包/29边；源码漂移/缺fixture反例及恢复正控通过。实施树research/viewport-source-reconciliation.md记录对账。T-02在同树建立原生事件和真实Auto临时预览探针，尚待验证。下面的blocked记录为已关闭历史。

2026-10-23：stage: work；PLAN-836 revision1；outcome: blocked（T-01来源门）。用户已授权WORK实施，GSTART满足。唯一实施树 `D:/autostack/.wt/lang-836/auto-lang` / `plan-836-dev`，base/code `2a7b10250245332f46b7867dc140da1a4f9d0969`；SDK `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`。已核根路径/分支及主检出无代码WIP。prepare通过680文件，依赖闭包479 packages、17本地packages、29 edges通过；verify_source在835继承清单的driver `.gitignore` 哈希处失败。全清单枚举仅此项不同，排除LF/CRLF差异；实际文件来自835提交8a7fce4ff。未直接更新期望哈希。证据见[入口收据](../attachments/836/work-t01/preflight.json.txt)。T-01未完成，T-02依赖来源门而未开始，当前0/10；无实施代码修改，无Cargo/图形验收，review/merge门pending。next responsible role: work，追溯835冻结原文件与正式落点差异，建立836完整来源冻结及正负控制后继续T-02。实施树及已准备SDK保留。


2026-10-23：stage: new；PLAN-836 revision1；outcome: pass（规划交接）。依835正式源码、R2和合入收据起草；G835满足，GSOURCE由T-01在实施树冻结。next: work，从T-01/T-02开始；本轮未启动实施。

规划历史：当时0/10，所有AC/task/case/SD尚未执行；以§9最新work-final收据为准。文档preflight见[检查记录](../attachments/836/planning/preflight.md)。final R1 needs_fix后，final R2已通过；merge未发生，完整门pending；若两轮final同因不收敛，先记录根因/漏测/兄弟路径再修，保留轮次。

## 10. 待澄清事项

历史来源/窗口授权阻塞和F-836-R1-01均已关闭；final R2及完整冻结门通过，delivery52ec4c7c1已合入autographics-dev，canonical/ledger已沉淀、实际归档；当前9/10仅清理pending。


无阻塞草案的产品决定。用户启动work后可按T-01/T-02推进，例行等价实现选择无需再审批。OBB精度、固定相机/三轴、不变schema已明示。若需逐三角形拾取、相机漫游、层级/多选、生产仓写面、SDK/AGP接口扩展或改变失败/资源支持保证，回new提出具体修订，不边实施边静默扩面。
