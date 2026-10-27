---
plan_id: PLAN-856
status: reviewed
feature_name: AutoGraphics 2D编辑视口网格与导航
author: [Codex]
created_at: 2026-10-27
updated_at: 2026-10-27
plan_revision: 1
supersedes_spec_components:
  - docs/specs/auto-graphics/viewport-interaction.md
  - docs/specs/auto-graphics/authoring-session.md
  - docs/specs/auto-graphics/integration-contract.md
new_spec_components:
  - docs/specs/auto-graphics/editor-viewports.md
touched_goals: [GOAL-008]
affects: [auto-graphics]
current_step: 8
total_steps: 10
---

# [PLAN-856] AutoGraphics 2D编辑视口网格与导航

## 0. 变更摘要

M2/S1第一片：让现有2D创作窗口具备网格/轴、平移/锚点缩放、聚焦/重置和可选世界平移吸附；同步修正绘制、拾取、组轮廓和真实Auto拖动的坐标来源。work已在855实际交付基面提交；独立final R2已确认R1逻辑pixel缩放与失败态导航取消两项修复闭合，冻结4150a8bea，reviewed7/10。完整两端/native-owner-content及受影响AGP门仍由merge负责，尚未合入或交付。

## 1. 目标

1. 2D视口导航不改变场景内容，原点/轴/主次网格可读且资源有界。
2. 任意支持缩放/平移下，circle/quad、组选择与拖动消费同一已呈现变换；吸附规则明确。
3. 对象手势、导航捕获、面板/文本与Play输入分离，取消/失败/旧事件不能污染内容或历史。
4. 保持原游戏/脚本/保存/预算、默认视图和全部原验收分母，独立消费者实际可用。

非目标：3D导航（857）、旋转/缩放gizmo、多选/框选、TileMap、精灵资源、运行相机、UI组件/主题改版、游戏服务、仓迁移/生产UI/HIR/ACE、新平台。[共同视口合同](attachments/856/planning/viewport-contract.md)是本Plan保持面和门禁的一部分，S0优化附件不自动禁止本Plan新增视图状态。

## 2. 架构方案

在driver新增独立`View2d`与呈现revision，将2D world↔child逻辑坐标计算收敛到一个纯接口；Host仍保留一份Editor/Play/输入/资源。View2d仅影响Editing的投影与辅助层，真实对象候选仍经原Auto预览/整帧准入。网格不写Document、节点树或Auto源码。

默认`center=(0,0), zoom=1`严格复现原`(400+x,190-y)`；child支持面仍800×380。优先复用现有DrawCmd，CPU侧变换2D中心/尺寸和独立辅助批次；不必新增渲染后端。T-02验证共同接法和draw容量，再改完整消费路径。

## 3. 技术栈

现有driver、glam、stock Iced/winit、wgpu/QuadPipeline与固定旧SDK，无新依赖/锁改动。独立workspace，Windows x64单窗。AGP为优先只读依赖；仅在T-02证明公开接点不足时可做有界辅助批次接点，保持原默认行为并登记受影响门。

## 4. 需求分析与背景调查

用户2026-10-27确认规划2D视口、3D视口、双游戏调查三份计划，规划轮仅new；2026-10-27本轮用户已明确授权work实施，没有新agent/外仓写入/推送授权。已确认M2/S1范围见[设计§4](../design/autographics/m2-editor-and-public-game-prototypes.md#4-编辑视口)、[战略](../design/strategy/autographics-m2-v06.md)。

取材[全局概览](../specs/overview.md)、[模块概览](../specs/auto-graphics/overview.md)、[当前视口](../specs/auto-graphics/viewport-interaction.md)、[作者会话](../specs/auto-graphics/authoring-session.md)、[场景](../specs/auto-graphics/scene-document.md)、[测量基线](../specs/auto-graphics/research-quality-baseline.md)。现行Spec明确没有camera motion/grid snapping，本文提出有版本溯源的编辑扩展，不把目标写成既有能力。

[source-observation](attachments/856/planning/source-observation.json.txt)绑定已交付854=`523d27b863dbe306c89af0ea546334225a29f8e9`与855观察提交`70f1addf0a12a6af7f3b9bf447428f558a779d26`及具体文件SHA；该引用来自work交接，不能替代855最终review/merge收据，当前未实际交付。其实际Host分为`host/{panel,commands,play_adapter,viewport}.rs`；`interaction::pick2`、`Gesture::document`、prepare/group_geometry等重复固定800×380映射。QuadPipeline容量4096，旧overlay_line每边最多241个quad，网格不能耗尽该池后截掉内容。

## 5. 详细设计

### 5.1 基线与可执行入口

硬前置：855完成独立review、冻结合入门并实际交付。然后从该autographics-dev提交建唯一`D:/autostack/.wt/lang-856/auto-lang`/`plan-856-dev`，记录实际HEAD/路径/允许diff/SDK/工具/旧分母。规划时没有dependency-ready实施任务；本轮855已实际交付05fdc81c546aff735eefdb6c39fbc5fb35dc546a，T-01已核实并从该提交接棒，未使用旧855观察提交提前冻结实施基线。

写面：driver `src/authoring/host/{viewport,panel,commands}.rs`、`interaction.rs`、Host必要状态、bin事件/CLI/probe接线、定向tests/tools/research/README。新增拟定`src/authoring/view2d.rs`、`tests/editor_viewport2d.rs`、`tools/check_editor_viewport2d.py`；T-02可选等价局部命名并登记。没有第二Editor/VM/项目格式。

### 5.2 坐标、支持界与操作

View2d保存世界中心c、正有限zoom z、Grid/Snap设置、revision。`screen=(W/2,H/2)+z*(world.x-c.x, -(world.y-c.y))`，逆变换用于拾取/拖动；只在composition入口把window/DPI转换成child逻辑坐标一次。对象尺寸随zoom，UI手柄/选框线宽和3px启动阈值保持逻辑像素。默认矩阵/结果与旧路径一致。

支持z∈[1/8,8]、中心每轴[-65536,65536]。有限用户导航超过范围饱和到边界；NaN/Inf、零尺寸或不可逆输入受控拒绝且状态不变。该界只限制新视图，不能据此拒绝原合法Document。变换计算溢出不得把非有限数据交GPU或当裁剪成功：可在可靠范围判定后裁剪不可见内容，否则拒绝该view更新、保最后完整画面并给原因，accepted文档不变。超出可聚焦范围/极大对象显示具体原因，不宣称已经完整frame；空/隐藏无可见几何、选择另一维度同样无内容修改。

操作：Editing的2D viewport中，MMB拖动平移；wheel围绕当前指针世界锚点缩放，每逻辑notch倍率1.1；pixel delta以40逻辑px换算1notch、每事件幅度限8notches。F聚焦、Home重置仅在该viewport拥有键盘焦点且无文本焦点时有效；面板提供Grid/Snap/步长/聚焦/重置明确控件。没有viewport焦点则快捷键不全局生效。

聚焦使用选中叶或组所有实际可见后代的世界边界，居中并以80%可用宽高适配，zoom按支持界限制。New/Open成功重置，失败保旧；Save/Undo/Run/Stop规则按共同合同。默认Grid off、Snap off，显式开启后显示新能力；不改旧golden/default玩法。

### 5.3 网格与吸附

网格以世界原点对齐，主次线以1/2/5×10^k系列按屏幕密度选择，优先使次线间距16–64逻辑px；若总线数超限继续整体变粗。含可见坐标轴在内最多128条线，辅助命令最多128个矩形、CPU辅助数组不超过16KiB；几何裁剪在viewport内，轴独立颜色。网格/轴先于场景绘制，选择轮廓/手柄保留其后层次且不拦截交互。不得按对象数量生成网格、不无限循环、不挤占场景/轮廓/手柄预算；总提交仍符合4096容量。实际显示spacing/counters可诊断，Grid off同时关闭本Plan新增轴等辅助绘制，默认原画面保持。

Snap步长可选1/5/10/50/100世界单位，默认10，独立于随zoom变化的显示网格。对拖动目标的最终**世界原点**各轴执行`s*round_away_from_zero(x/s)`；先从手势起点与屏幕总位移求绝对world候选，再吸附，再按父世界平移反解local。2D z仍0；叶/组移动只改自身local，不连带改子local、角度或scale。释放no-op不加历史；成功仍一次history，Undo/Redo恢复原内容。

### 5.4 全路径、取消和有界探针

T-02最多比较CPU统一变换与Quad相机统一变换两种局部接法，验证“导航→绘制/拾取→真实Auto预览→释放提交→Undo/保存/重开”后冻结`attachments/856/work/view-decision.md`。材料包含变换数据流、实际呈现revision、辅助容量/上传、输入捕获及拒绝边界；不得选只修一个入口的接法。

pick2、group_geometry、轮廓、2D组原点、拖动候选和probe全部消费同一View2d/呈现快照；保留反序绘制拾取/ellipse精确形状/alpha与clip规则。对象drag期间导航/设置拒绝；导航期间不做对象修改；Esc/失焦/离窗/resize/DPI/关闭结束捕获，过期view/候选不能发布。wheel与原面板scroll不能同时消费。Play期间回原运行视图并禁导航，Stop恢复编辑view与accepted内容。

新增探针入口拟为`--viewport2d-check`，新工具约定`python tools/check_editor_viewport2d.py <fresh-output> --executable <retained-exe> --controls`（T-05创建，不是现有命令）。工具通过真实OS鼠标/键盘走winit，记录实际呈现变换、目标ID/world值/文档hash/dirty/history、状态与最终合成像素。复用原bin的interaction-probe路径和现有输入/报告设施，不复制正常运行框架。

当前来源清单新增plan856/verifier并更新verify_source/package consumer，文件集合/构建入口/SDK都须校验，旧manifest不改。源码/owner/原帧时基线与新辅助成本分别报告。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/editor-viewports.md | 固定视图→有限2D ViewState/变换/导航/网格/吸附与生命周期，未包含3D导航 | 公共编辑合同 | AC-02/03/04/05 |
| SD-02 | modify | docs/specs/auto-graphics/viewport-interaction.md | 历史固定坐标保留默认，新增ViewState-aware pick/drag与过期拒绝 | 统一实际消费 | AC-02/04/05 |
| SD-03 | modify | docs/specs/auto-graphics/authoring-session.md | view不进入scene4/history；导航与编辑/Play边界 | 防止状态混用 | AC-05 |
| SD-04 | modify | docs/specs/auto-graphics/integration-contract.md | 856来源/consumer及新OS/GPU正反面；原分母保留，I2/I3 pending | 可复查交付 | AC-01/06/08 |

## 6. 测试设计

实施自查→独立最终review→merge，未指定模型/额外agent；同一实现树。work仅命名模块/反例/必要check，新工具/fixtures由列出的任务建立，尚未执行。

| Case ID | AC/不变量 | 实际/拟建入口与输入 | 区别错误实现的断言 | work任务/覆盖 |
|---|---|---|---|---|
| V-01 | AC-02 可逆映射 | 新editor_viewport2d；默认、中心正负、z=1/8,1,8；入口DPI1/1.5/2、clip外/零/非有限 | 默认旧点位一致；world↔screen往返，错误双DPI/旧400常量可被非默认case区别；无效保旧 | T-02/03；正常数值误差≤1e-3world，像素≤1逻辑px，不承诺极端场景可聚焦 |
| V-02 | AC-03 网格资源 | Grid开关、缩放/平移极限、dense要求、20次导航/开关/关闭 | 原点世界对齐，≤128线/16KiB，变粗而非删对象/轮廓；真实像素/计数AND内容与最后owner | T-03/05；有界2D辅助 |
| V-03 | AC-04 真实编辑 | ellipse角外、重叠后画者、带parent叶/组；导航后拖动，snap±半步/关闭、回原点 | 选中同一ID；世界候选按固定公式，子local/其他字段不变；一次history/noop零、Undo/Redo/保存重开 | T-04/05；现有同维平移层级 |
| V-04 | AC-05 输入/状态 | 新OS probe与旧interaction；跨面板/text、busy、Esc/blur/resize、旧view晚到、Play/Stop | 真capture消费单次，旧候选拒绝，accepted/磁盘不污染；Stop恢复edit view，游戏不被导航输入驱动 | T-04/05；单窗native实际DPI＋unit多比例 |
| V-05 | AC-06 oracle敏感 | 实际错用旧逆变换、漏画grid两种局部变异，正常恢复；partial/缺readback | 同一个数值/内容门具名红，不能只改JSON或任意崩溃；恢复绿/完整报告后退出 | T-05；新oracle，不替代原门 |
| V-06 | AC-01/06 来源兼容 | 当前verifier/consumer；漏新模块、坏SDK、原default/旧profiles与855对照 | 漏项拒绝，fresh consumer真实新模式；旧生成身份/数值/内容/owner/预算不变，新辅助成本单列 | T-05/06；共同合同 |

定向入口driver cwd：`cargo check --locked`；新增`cargo test --locked --test editor_viewport2d -- --test-threads=1`（T-03/04）；原`cargo test --locked --test authoring_interaction --test authoring_host --test authoring_session -- --test-threads=1`；新check工具与当前verifier。完整两端/native/AGP触面按共同合同只在最终review后merge执行；本次new无Cargo/GPU运行。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 855实际交付/唯一树/SDK/锁/允许写面核实，当前闭包完整；V-06 |
| AC-02 | work | 统一View2d/呈现revision，默认旧坐标、有限导航/焦点/聚焦/拒绝正确；V-01及源码数据流 |
| AC-03 | work | Grid世界对齐/开关/主次密度与辅助上限成立，内容未被截断、资源复用释放；V-02 |
| AC-04 | work | 所有pick/轮廓/group/真实Auto拖动/吸附/历史/保存一致；V-03 |
| AC-05 | work | 导航-对象-面板-Play边界、取消/旧事件拒绝和Stop恢复成立；V-04 |
| AC-06 | work | 新OS数值AND合成像素、真实变异恢复、consumer/default/测量对照齐备，未改旧分母；V-05/06 |
| AC-07 | review | 独立验证work AC/V/SD、warnings/fmt与遗漏/绕行；完整merge门明确pending |
| AC-08 | merge | 冻结两端完整/受影响AGP/原新native-owner-content门通过，复用逐项证明 |
| AC-09 | merge | 源码只落autographics-dev；规范/ledger/归档及guard清理实际完成，交接857冻结提交 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 855交付 | 唯一树/实际基面、依赖/旧分母与工具接棒；记录允许diff和入口 | AC-01 | [x] |
| T-02 | work | T-01 | view-decision：两种接法有界全路径探针、变换/输入/容量和oracle冻结 | AC-02/04 | [x] |
| T-03 | work | T-02 | 新view2d及Host投影/Grid/导航UI/事件；V-01/02和新定向fixture | AC-02/03 | [x] |
| T-04 | work | T-03 | interaction/group/drag/snap、view revision及busy/取消/Play接线；V-03/04 | AC-04/05 | [x] |
| T-05 | work | T-03/04 | 新OS/GPU probe/check、实际变异恢复；当前来源/verifier/consumer、V-05/06 | AC-01/06 | [x] |
| T-06 | work | T-05 | 定向收口、资源/性能原始对照、提交代码/SD/证据，不跑整档 | AC-01..06 | [x] |
| T-07 | review | T-06 | 独立final R2 pass，4150a8bea；两finding闭合，完整merge门pending，见review-r2/review.md | AC-07 | [x] |
| T-08 | merge | T-07 pass | 冻结完整两端/原新native-owner-content/AGP触面与复用审计 | AC-08 | [x] |
| T-09 | merge | T-08 | 源码合autographics-dev，main沉淀SD/overview/plans/ADR/ledger/索引并归档 | AC-09 | [ ] |
| T-10 | merge | T-09 | guard clean后清树/临时分支、产物恢复路径/SHA与实际10/10；给857真实提交 | AC-09 | [ ] |

## 9. 复审记录

2026-10-27 new r1：stage=new；outcome=blocked（855未实际交付）；next=855独立验收/合入后核冻结提交，进入T-01。范围/AC/V/T/SD已具体化，全未执行；855观察代码不是可实施冻结基线。未改855合同或实现，没有新Cargo/native/性能结果。计划保持drafting。



2026-10-27 work entry：用户明确授权实施。855已archived/delivered9/9，源码autographics-dev实际HEAD为05fdc81c546aff735eefdb6c39fbc5fb35dc546a，原new阶段等待结论已过期；从该提交创建并断言唯一树D:/autostack/.wt/lang-856/auto-lang / plan-856-dev。主检出原有未跟踪crates/Cargo.lock及test/fixtures已披露、保持不动。本树prepare_sdk及verify_source通过：681 SDK文件/455来源资产/479packages/17local/29edges；SDK固定7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467，两锁不变。T-01继续核工具/旧分母，T-02采用CPU统一接法进行全路径有界验证，尚未冻结decision/完成标记；无整档执行、无合入。



2026-10-27 work最终交接：`stage: work | plan_id: PLAN-856 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-856/auto-lang | branch: plan-856-dev | base_commit: 05fdc81c546aff735eefdb6c39fbc5fb35dc546a | code_commit: 3c39a96d21346e06b5c0ebf93b6e80ec4e5bb7a2 | dependency_revisions: SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467；AGP/两锁/API/assets/samples不变 | task_ids: T-01..06 | evidence: attachments/856/work/readiness.md + view-decision.md + performance.md + handoff.json.txt + spec-delta.md | blockers: [] | next_responsible_role: independent review T-07`。实际855已交付，T-01前置闭合；CPU统一View2d/呈现revision、Grid/导航/焦点、world吸附/真实Auto/组/历史/保存及Play恢复完成。producer新8例、旧Host/interaction/session18例、Play两个具名反例、source工具两例通过；真实OS20阶段/partial两端通过，原interaction27阶段通过，旧逆变换/漏网格真实代码变异具名红后恢复；零尺寸窗口实反例红→永久绿。当前462来源/681SDK/依赖闭包与fresh own-SDK/空target消费者通过，原分母不变；默认855/856三对80帧内容/像素/资源一致，导航off/on40次测量与复用限定见performance.md，不虚报GPU/60Hz或加速。实施与守界两提交0d2b7a3a4/3c39a96d2已冻结且clean，open work findings/gaps/decisions为空。execution_done6/10；T-07独立最终review及T-08冻结完整两端/native/owner/content/AGP受影响/复用审计、T-09源码仅autographics-dev与规范ledger归档、T-10guard清理仍pending；未合入/发布canonical或ledger/归档清树，不称交付完成。



2026-10-27 independent final R1：`stage: review | review_scope: final | reviewer_identity: independent child context /root/review_856（未参与实施，继承对话仅作上下文，以源码/提交blob/原始证据和独立反例重建结论） | plan_id: PLAN-856 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-856/auto-lang | branch: plan-856-dev | reviewed_commit: 3c39a96d21346e06b5c0ebf93b6e80ec4e5bb7a2 | base_commit: 05fdc81c546aff735eefdb6c39fbc5fb35dc546a | dependency_revisions: SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467；AGP/两锁/API/assets/samples无diff | spec_inputs: attachments/856/review-r1/audit.json.txt | acceptance_results: AC01/03/04在work边界pass，AC02/05 fail，AC06 partial，AC07未通过，AC08/09 merge pending | findings: [F-856-R1-01, F-856-R1-02] | evidence: attachments/856/review-r1/review.md + counterexamples.rs + replay.ps1 + counterexamples.json.txt + audit.json.txt | next: same-tree work T-03..06`。F-01：winit PhysicalPosition的PixelDelta未经逻辑换算，DPI2/80物理px应zoom1.1而实际1.21；DPI1/40px合法控制1.1，当前OS LineDelta正例不覆盖pixel支路。F-02：已呈现MMB平移后resource拒绝期间取消导航，期待回起点0/原图，实际残留-40/移动后图；直接cancel、blur、resize及after-upload拒绝同因复现，无故障控制正常，文档/history断言保持。独立当前正常OS20阶段及frame1partial负控、462提交blob/681SDK/依赖闭包、125原证据hash、fmt/diff与SD冻结完成核准，未因已有缺陷跑全档。T-01/02保留；T-03/04及依赖T-05/06重开，executing2/10，T-07未通过。拟SD目标/GOAL008有效，SD01/03现承诺的pixel/无条件取消须由实现修齐，不降合同；冻结副本/hash见audit。HEAD/clean复核不变，无实现修复/新树/合入/规范ledger发布/归档清理。完整两端/原新native-owner-content/AGP合格复用及沉淀清理仍merge-owned pending。



2026-10-27 repair entry：接回独立R1写权，权威树/branch/HEAD=lang-856/plan-856-dev/3c39a96d21346e06b5c0ebf93b6e80ec4e5bb7a2，clean，SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467；原主检出未跟踪crates/Cargo.lock及test fixtures保持不动。已按review-r1/replay.ps1原输入在冻结基线重放：DPI1/40px=1.1控制通过，DPI2/80px=1.21反例；无故障cancel原像素恢复，resource/after-upload故障的cancel、blur、resize残留-40及移动图。F-01执行合同：实际winit PixelDelta物理坐标→Host输入边界→同CompositionInput物理/child映射一次→每40逻辑px一档，LineDelta和面板事件不变；测试DPI1/1.5/2、正负/大值/锚点、零/非有限及bin实际转接。F-02执行合同：MMB开始保存已准入2D绘制与view，失败/取消从快照恢复，不重建VM/mesh；捕获终结后迟到move/up不可发布，原故障和错误可见，accepted/history不变。用原两故障seam、直接cancel/blur/resize/关闭及正常对照验证真实图像、状态和资源，保留未变场景/配置证据；不运行整档，merge统一门保持pending。完整闭合前T-03..06仍未完成。



2026-10-27 needs_fix修复交接：`stage: work | plan_id: PLAN-856 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-856/auto-lang | branch: plan-856-dev | base_commit: 3c39a96d21346e06b5c0ebf93b6e80ec4e5bb7a2 | code_commit: 4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7 | dependency_revisions: SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467；AGP/两锁/API/assets/samples未改 | task_ids: T-03..06 | evidence: attachments/856/repair-r1/closure.md + handoff.json.txt + spec-delta-r1.md | blockers: [] | next_responsible_role: independent affected final review T-07`。F-856-R1-01/02按原r1合同修复：winit PixelDelta单次物理→逻辑换算、极值/非有限输入受控；导航取消直接恢复已准入起始2D命令和view，不触发可能失败的资源重备。原review7输入全部重放通过，producer与独立consumer各自定向测试绿；原窗口20阶段、bin winit转发、旧取消/拒绝和零窗口控制、失败wheel/close及cargo check通过，SDK146原warnings、driver无新增；资源seam状态与错误、真实起始图像、历史/accepted、迟到move/up和mesh/target计数已核。未重跑无关全档/性能，统一producer/consumer/native/owner/content门仍merge-owned pending。审查产物注意：首次重放脚本覆盖review-r1探针exe和PDB，原audit这两个二进制哈希不再匹配；原输入/原结果及其他审查原始文件仍匹配，旧audit不篡改，差异和修复后独立重放见closure.md。代码提交clean；work findings/gaps/decisions为空，execution_done6/10。T-07受影响面独立最终复审未通过前不得合入或宣称交付；未发布canonical/ledger或清理worktree。

2026-10-27 independent affected final R2：`stage: review | review_scope: final | reviewer_identity: current /root context（本聊天未参与修复实施，从源码/提交/原始证据及新独立反例重建；未创建额外agent） | plan_id: PLAN-856 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-856/auto-lang | branch: plan-856-dev | reviewed_commit: 4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7 | repair_base_commit: 3c39a96d21346e06b5c0ebf93b6e80ec4e5bb7a2 | base_commit: 05fdc81c546aff735eefdb6c39fbc5fb35dc546a | dependency_revisions: SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467；AGP/两锁/API/assets/samples不变 | spec_inputs: attachments/856/review-r2/audit.json.txt | acceptance_results: AC01..07 pass（AC03/04/06保留work边界），AC08/09 merge pending | findings: F-856-R1-01/02 CLOSED，open=[] | evidence: attachments/856/review-r2/review.md + audit.json.txt + original-counterexamples.json.txt + sibling-controls.json.txt | next: merge T-08..10`。四个具名永久定向测试各1/1，原R1七输入7/7，新独立九控9/9，覆盖非等比窗口pixel锚点、失败up/离窗/未呈现取消/零resize×两故障、真实2D/3D图像/捕获/revision/资源/后续合法导航；当前OS20阶段exit0及partial exit1通过。check/fmt/diff clean，SDK146旧warnings、driver无新增。462提交blob/consumer来源、681SDK闭包、125work及18repair证据hash核实；R1只有已披露exe/PDB失配，其余原始证据保留匹配。SD冻结副本与hash在audit，修正旧测试数8→11及bin接线例的证据记述，无语义改约；并行858仅增加overview/GOAL008研究历史，与当前SD无冲突，准确diff已保存。最终HEAD/branch/clean复核一致；reviewed7/10，不跑完整整档/不改实现/不合入/不发布canonical或ledger/不归档清树。完整producer/consumer及原新native-owner-content/AGP23逐槽复用审计和沉淀清理仍pending，review pass不能替代交付。



2026-10-27 merge entry：用户授权merge；独立final R2 pass绑定plan_revision1、代码4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7、SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467及冻结SD（SHA256 9daf99d4cfbc5f482ea41e1c223e9a5a8625cca91a058a01693d9245f3f260f3）。唯一lang-856树/plan-856-dev/HEAD/clean与autographics-dev前置05fdc81c5核准；T-08启动冻结producer/fresh own-SDK空target consumer的完整实验面及原/新native-owner-content门、AGP逐槽复用核准。主检出其他会话WIP已披露并保持不动；仅实验源码合autographics-dev，规范与ledger/计划另落master，保留PLAN-858已沉淀的并发知识。当前reviewed7/10，门未完成、未落源码/规范、未归档清理。



2026-10-27 merge T-08 attempt1：生产端冻结4150a8bea完整80入口/300实际用例全部通过0failed/ignored；source/SDK负控、all-target check/fmt和栈构建通过。fresh own-SDK空target消费端完整测试退出4294967295，日志在editor_viewport2d最后一个zero_window用例开始处截断，前10个该入口用例成功，没有panic/FAILED或cargo错误诊断；不得将不完整全档计作通过，精确根因尚未确定。原日志与gate记录保留D:/autostack/plan-artifacts/plan-856/merge-r1/consumer-test.log.txt及gates.json.txt。系统当时可用内存约16GiB，不能单凭这一观察排除瞬时资源问题；正在同一消费包、相同冻结源码/依赖/配置下隔离重放末例和其完整入口。T-08仍pending，未合入/未发布规范。


2026-10-27 T-08消费端中断诊断：冻结源码/SDK/锁及同一fresh consumer target不变；最后zero_window单例实际1/1通过，原11例入口在新进程完整11/11通过，零skip；producer完整也已通过相同入口。日志 consumer-zero-diagnostic.log / consumer-viewport-diagnostic.log 与各.exit 保留。第一次cargo退出4294967295、没有panic/错误尾迹；不能断言根因或把隔离成功当作完整门通过。此新证据支持一次同基线完整重跑；原失败和失败日志不覆盖，完成前T-08仍pending。

2026-10-27 merge T-08 attempt2与诊断：同一fresh consumer完整测试exit 101，`segment_budget::redundant_source_lines_reproduce_the_diagonal_budget_failure`在第199行只接受`cumulative CPU budget exhausted`，实际收到100000步的`instruction budget exhausted ... per-request cumulative budget ... AUTO_HTTP_STEP_BUDGET`；原日志与记录为D:/autostack/plan-artifacts/plan-856/merge-r1/consumer-test-attempt2.log.txt及gates.json.txt。该断言与SDK在855交付基面已存在、856未改；SDK CpuSlice每片1000步/2ms，累计上限100000，片尾达到上限与片内超限分别返回不同文案，片内路径文案还错误指向HTTP配置。首次中断根因仍未知。该同一consumer二进制/源码/SDK/配置定向隔离连续12次通过（consumer-budget-diagnostics.json.txt及各log），表明已观测到断言不稳定，不能将隔离通过当成完整门。保留两次失败，基于时间切片证据及当前定向12/12通过，再进行一次同基线完整门；若再失败须按实际输出归因，不自动无限重试。详细诊断见attachments/856/merge-r1/budget-diagnosis.md。T-08仍pending，reviewed 7/10，未合入、发布或归档。

2026-10-27 merge T-08 attempt3：同一冻结4150a8bea/SDK/consumer条件下完整消费端门exit0，80入口/300例、0 failed/ignored；原预算反例在完整运行通过。独立清单审计证明两端测试ID完全一致，包含855全部288项及856新增12项；consumer build/fmt/两binary栈构建通过。原attempt1退出4294967295根因未知，attempt2既有预算断言对时间切片不稳定仍属风险，不掩盖或称修复；两次日志与12次隔离通过均保留。见attachments/856/merge-r1/selected-ids.json.txt、budget-diagnosis.md及外部gates/原日志。原/新增native-owner-content仍在运行，AGP23槽已逐项核准复用；T-08未完成、reviewed7/10，未合入或发布。

2026-10-27 merge native进展：producer新视口正常20阶段/partial负控、交互/重开、hierarchy/pixel故障和assets已通过。play第一轮wrong-gpu-asset在到达预期像素故障前由真实窗口报告`native fixture pointer occluded by another process`而exit1；原`play/`现场及`play.log.txt`保留。确认无残留测试窗口后，在全新`play-retry1/`运行同一7项原生控制，exit0且全部按预期，见`play-attempt2.log.txt`和results；未覆盖失败现场。其余producer/consumer原生、owner/content仍pending。

2026-10-27 merge blocked handoff：`stage: merge | plan_id: PLAN-856 | plan_revision: 1 | outcome: blocked | worktree_path: D:/autostack/.wt/lang-856/auto-lang | branch: plan-856-dev | code_commit: 4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7 | dependency_revisions: SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 | task_ids: T-08..10 | evidence: attachments/856/merge-r1/blocker.md + selected-ids.json.txt + budget-diagnosis.md + producer/consumer-new-api.json.txt | blocker: 原生项目场景被另一进程遮挡，单例复测phase0超时且原因未知 | next_responsible_role: merge after desktop focus/occlusion resolved`。producer/consumer完整各80入口300/300且ID完全一致、21生成文件和各7项控制通过；producer原生新视口、交互、层级像素、assets/play/user/step通过；projects bind-patrol原始遮挡、隔离复测phase0超时，均保留原始日志和报告；不无限重试或把隔离结果冒称完整门。其余native/owner/content、实际落源码/规范ledger/归档/清理均pending，autographics-dev仍05fdc81c5。reviewed7/10，工作树clean；消费端旧预算文案不稳定与首次无诊断中断如实保留，未称修复。

2026-10-27 merge恢复诊断：用户要求856交付后实施857；856唯一树HEAD仍4150a8bea、autographics-dev仍05fdc81c5。当前只读进程检查未发现其他AutoGraphics验证，同一冻结二进制的bind-patrol在独立`projects-diagnostic-bind-patrol-resume/`再次phase0超时；另一次后台窗口观察`projects-window-observation/`也phase0超时，运行期间窗口清单未列出测试窗口，因后台启动方式可能影响显示，不能作为正常可见窗口门通过或确定根因。两份独立原始日志保留，不盲目再试。待桌面焦点/窗口条件确认后继续856，857的T-01明确依赖856实际交付，当前仅进行了只读计划/代码/规范核对，没有创建857树或实施。

2026-10-27 merge T-08 pass：用户确认桌面可交互后，原bind-patrol同冻结代码/SDK在新目录成功，producer完整项目15项重跑通过并保留旧遮挡现场；producer/consumer所有native、sweep、motion、owner与content负控通过。实际双端各300/300，继承288 ID及新增12 ID；21生成文件一致、双端各7控制通过，AGP23槽逐项不变复用审计通过。合并前证据索引与原始报表审计见attachments/856/merge-r1/validated.json.txt、evidence-index.json.txt及failure-closures.json.txt；此前无诊断中断根因未知、旧预算文案断言时序不稳定继续记录，不称修复。T-08完成，reviewed8/10；源码落autographics-dev、规范ledger/归档与guard清理仍pending。

## 10. 待澄清事项

855实际交付前置已闭合；T-02 CPU统一接法已取证冻结，未改AGP/SDK/锁/公共算法，未突破4096容量。既定r1合同不变；R1的F-856-R1-01/02已在4150a8bea修复，独立受影响final R2确认闭合，详见review-r2/review.md。work/review合同结论未改变；merge T-08完整门通过，沉淀/归档/清理仍pending。原遮挡/phase0超时现场及历史预算断言风险保留，见attachments/856/merge-r1/blocker.md与budget-diagnosis.md。独立pass只覆盖冻结代码/验收/SD，不代表已合入或交付。
