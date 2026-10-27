---
plan_id: PLAN-857
status: reviewed
feature_name: AutoGraphics 3D编辑视口导航与深度网格
author: [Codex]
created_at: 2026-10-27
updated_at: 2026-10-27
plan_revision: 1
supersedes_spec_components:
  - docs/specs/auto-graphics/editor-viewports.md
  - docs/specs/auto-graphics/viewport-interaction.md
  - docs/specs/auto-graphics/surface-composition.md
  - docs/specs/auto-graphics/integration-contract.md
new_spec_components: []
touched_goals: [GOAL-008]
affects: [auto-graphics]
current_step: 7
total_steps: 10
---

# [PLAN-857] AutoGraphics 3D编辑视口导航与深度网格

## 0. 变更摘要

M2/S1第二片：在856的编辑ViewState/输入边界交付后，为3D创作加入地面网格/轴、环绕/平移/缩放、聚焦/重置、透视/正交切换和世界轴平移吸附。重点补齐相机改变后的all-mesh代理拾取、投影/手柄和透视拖动；不沿用固定相机近似冒领。

## 1. 目标

1. 在有限3D编辑相机范围观察和定位已有场景，网格具有正确世界位置与实际深度遮挡。
2. 渲染、拾取、输入相机、轮廓/组边界/手柄、拖动使用同一已呈现快照。
3. 透视/正交及导航后仍能真实Auto平移、吸附、Undo/保存/重开；临近视线退化明确禁用。
4. 沿用856导航/内容/Play分离、原兼容与源/consumer/计量/内容门。

非目标：运行时游戏相机、三角形精确拾取、旋转/缩放gizmo、多选、曲面Portal、PBR/蒙皮/阴影重写、动态对象/角色、主题改版、外仓迁移/ACE/生产UI或新平台。[共同合同](attachments/856/planning/viewport-contract.md)适用；本Plan显式允许有限AGP深度辅助绘制接点，不能照搬S0的AGP完全只读豁免。

## 2. 架构方案

driver新增独立`View3d`，复用856的导航状态/revision/busy/cancel框架，维度数学分开。现有`Camera3d`可生成两种投影；Host将同一当前相机提交Scene3dContent及相关CompositionInput相机表，在实际呈现后更新可拾取快照。保留唯一Editor/Play/GPU/device/资源owner。

地面grid是编辑辅助、非场景节点；在同device/同camera/depth下绘制，不写碰撞、阴影caster或Document。最多比较复用有限mesh strip与独立有界depth辅助pass两种接法，T-02证实端到端后决定。置顶screen_overlay只适合手柄/选择，不可伪装成地面深度网格。

## 3. 技术栈

现有driver/AGP、glam、wgpu/Iced/winit及固定SDK；不加依赖或改锁。允许AGP `render3d/`和`host/surface_runtime.rs`的最小辅助接点/计量/测试，默认关闭保原路径。Windows x64单窗，3D child800×400。

## 4. 需求分析与背景调查

授权为用户2026-10-27建立三份计划的请求；本轮仅new，没有实施/建仓/新agent/推送授权。设计入口为[M2§4.3](../design/autographics/m2-editor-and-public-game-prototypes.md#43-3d)。

取材[模块概览](../specs/auto-graphics/overview.md)、[视口](../specs/auto-graphics/viewport-interaction.md)、[组合](../specs/auto-graphics/surface-composition.md)、[嵌套输入](../specs/auto-graphics/nested-input.md)、[资产](../specs/auto-graphics/scene-assets.md)、[基线](../specs/auto-graphics/research-quality-baseline.md)。当前公开Spec只证明固定正交创作视口；AGP存在透视矩阵函数不表示创作导航或透视拖动已交付。

[观察SHA](attachments/856/planning/source-observation.json.txt)绑定855 work提交，仅观察。实际`interaction::{project,pick3_with,axes,handle}`依赖固定800×400；`Gesture::document`用投影单位轴点积反解，不能直接推广到透视。`Camera3d/Projection`为右手Y-up、wgpu NDC z∈[0,1]；`CompositionInput`登记父Surface相机。当前screen_overlay无真实场景深度语义。856将提供的代码/接口当前尚未存在，T-01须核实际交付，不假定文件命名。

## 5. 详细设计

### 5.1 前置与写面

硬前置856实际交付（含855已交付）。从该autographics-dev提交建唯一`D:/autostack/.wt/lang-857/auto-lang`/`plan-857-dev`，最终只合autographics-dev。T-01核实际源码、856 ViewState/工具/输入合同、SDK/锁/保持面与原分母；当前无可执行实施任务。

driver写面为host viewport/panel/commands、interaction、导航view模块及bin/probe/tests/tools/research/README；新增拟定`src/authoring/view3d.rs`、`tests/editor_viewport3d.rs`、`tools/check_editor_viewport3d.py`。AGP新增拟定`src/render3d/editor_grid.rs`及surface-runtime可选挂载/计量、定向测试。等价局部命名由T-02决定，不能更改公共游戏/设备owner或原mesh/material/灯光语义。

### 5.2 View3d与用户操作

View3d保存target、yaw/pitch、distance、投影/ortho高度、辅助设置和revision。右手Y-up；target分量[-65536,65536]，distance[0.5,4096]，pitch[-85°,85°]、yaw有限归一化，ortho高度[1/8,8192]。有限导航饱和，非有限/零尺寸/退化view受控拒绝且保旧；这些界不修改Document合法性。显式导航透视FOV60°、near0.05/far16384；默认/reset直接复用原sample_camera全部数值（包括原near/far/矩阵），不重生旧golden。

Editing三维viewport：RMB拖动环绕target、MMB相机平面平移、wheel推拉/正交缩放。wheel换算与856一致；F聚焦、Home reset、P投影切换仅该viewport焦点且无文本焦点时有效；面板有明确控件。LMB继续选择/轴拖动。角度/平移速度按child逻辑尺寸与当前距离/投影尺度计算，T-02写具体公式并在新case前冻结，不依赖帧率累加。

透视/正交切换保持target与方向，以target平面的可见高度`s=2*d*tan(FOV/2)`匹配ortho size；所需d/size越界时有限饱和并显示限制，不虚称完全等比例。聚焦实际选中模型所有mesh联合世界边界或组可见后代聚合，考虑aspect/投影给20%屏幕边距；空/隐藏/跨维/超界或无法完整frame有可观察原因，不改内容。reset恢复原视图；New/Open/Save/Undo/Run/Stop按共同合同与856约定。

### 5.3 拾取、手柄与透视拖动

所有空间计算使用已呈现view_proj、Surface child尺寸、文档stamp/view revision。pick仍是全部mesh联合局部AABB经TRyRxS形成的OBB，使用NDC0/1有限射线段与共同t最近命中/稳定ID平局；维持alpha/visibility/clip与空隙proxy规则，不认领triangle精度。组边界/手柄同样使用实际当前相机，近远外/投影w不合法不生成可点伪影。T-02须区分有效大视野矩阵与真正退化矩阵，不能沿用原绝对det阈值误拒整个合法缩放区间；用有限可逆/数值稳定对照冻结判据。导航上限不承诺任意距离下可分辨任意微小物体，退化手柄仍明确禁用。

轴拖动在按下时冻结世界原点、轴、已呈现相机/逆矩阵与拖动平面。建议平面包含该轴、法线取camera forward去掉轴分量后归一化；通过屏幕近远射线与该平面交点投影求轴位移，再从起点总位移求世界候选。不能用原“屏幕delta÷投影单位轴”近似透视。视线几乎平行轴、平面不稳定/交点无效时明确禁用手柄或拒绝该sample，保最后合法候选，不任意跳跃。

世界轴候选按856步长与半步away-from-zero规则吸附，仅当前轴变化，再反解parent平移得到local；真实Auto预览和整帧准入仍唯一发布。原54px/7px手柄/3px启动阈值保持逻辑像素；退化判定与数值容差在T-02用有效/无效对照冻结。默认旧路径须通过原数值容差，不能以透视升级重写原expected。

导航/设置与对象手势互斥，旧view/camera候选无效；所有变更同时更新渲染和需要的输入相机登记。不得渲染用新相机、plane UV/拾取仍借旧相机；没有相机的父面保持miss规则。面板/2D Surface不被三维导航消费。

### 5.4 深度网格与资源

XZ地面y=0，世界原点/轴不随相机伪移动；在target附近有限区域生成，单轴半径不超过4096世界单位，spacing从1/2/5×10^k选择并整体变粗，最多128条辅助线（含轴）、1024个提交顶点。CPU辅助几何≤64KiB，GPU可归属辅助buffer+uniform≤64KiB，分别按实际分配计量；不声称是总驱动/进程内存。无新纹理，无场景节点/碰撞/阴影caster。接近地平线/无法与地面相交时保有限区域或明确隐藏，不扩成无穷网格。

深度绘制与模型共用实际view/depth规则；通过前景遮挡块和网格空白区域同时证明遮挡与可见。共面处理/depth bias由T-02在正反例中确定，不允许全程置顶或通过改变原模型深度让测试变绿。Grid关闭不绘制/上传辅助，默认原像素不变。相机变化只更新合法camera/grid动态量，不重复上传原静态模型/重建pipeline/target；resize/切换/关闭/多轮导航的新增owner真实释放。

### 5.5 探针与当前身份

T-02最多两种depth网格接法＋一条透视drag全路径探针，产出`attachments/857/work/view3d-decision.md`：公式/退化界、view revision与输入登记、buffer/计量和默认兼容，之后T-03/04实现。没有SDK或Auto算法扩展。

新probe拟为`--viewport3d-check`；工具`python tools/check_editor_viewport3d.py <fresh-output> --executable <retained-exe> --controls`由T-05建立，真实OS导航/切换/选择/轴拖动/吸附/Undo/保存重开，含相机/目标/数值、内容像素、深度遮挡与生命周期oracle。至少“旧相机拾取”与“grid强制置顶/漏画”实际变异在原新同一门红，恢复后绿；partial/无readback不能成功。

新增plan857来源/verifier与current consumer入口，覆盖AGP新文件/构建入口；AGP触面完整回归或逐槽不变复用需重新判定，不能沿用856“AGP未变”结论。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-graphics/editor-viewports.md | 在856 2D上增加有限View3d/投影/导航/深度grid，不加游戏相机 | 明确编辑支持集 | AC-02/03/05 |
| SD-02 | modify | docs/specs/auto-graphics/viewport-interaction.md | 固定正交默认兼容＋透视/正交实际相机拾取/轴拖动/吸附/退化 | 防止旧近似漂移 | AC-04/05 |
| SD-03 | modify | docs/specs/auto-graphics/surface-composition.md | 有限编辑辅助深度绘制/同device所有权与计量，默认旧内容不变 | 可追踪新增资源 | AC-03/06 |
| SD-04 | modify | docs/specs/auto-graphics/integration-contract.md | 857当前source/consumer/AGP触面和OS-GPU数值/内容/深度负控 | 保留原分母 | AC-01/06/08 |

## 6. 测试设计

独立上下文最终review使用同一实现树，完整门归merge；用户未分配模型/额外agent。新case和probe在指定任务创建，当前没有执行证据。

| Case ID | AC/不变量 | 实际/拟建入口和输入 | 可区别错误实现的断言 | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-02 相机 | 新editor_viewport3d；reset、两投影、orbit/pan/zoom、focus、饱和/NaN/零尺寸 | reset原矩阵/像素，target尺度切换公式；有限边界/无效保旧，焦点/面板不串 | T-02/03；有限view，不承诺所有合法巨大场景可frame |
| V-02 | AC-03 深度/资源 | grid开关、不同视角、遮挡块、128线密度、20轮导航/resize/Stop/关闭 | 实际像素既有可见线又有被挡线；世界对齐、配额/上传/最后owner；原mesh不重传 | T-03/05；现有固定光照，不是无限grid |
| V-03 | AC-04 选择/轴 | 多mesh旋转非均匀scale/近远重叠/group、透视和正交、X/Y/Z、视线近平行 | 同ID/最近t、近远clip正确；独立ray-plane oracle得到world位移，退化禁用非跳跃 | T-04/05；OBB代理，正常case原1e-3world/1逻辑px容差 |
| V-04 | AC-04/05 发布/历史 | parent叶/组、snap正负半步、旧view候选、cancel/busy/重复release、Undo/保存重开 | 单轴候选/子local/非目标字段正确，一次history/noop零；迟到拒绝、真实Auto发布 | T-04/05；原同维平移层级 |
| V-05 | AC-05 输入/Play | 原生三维RMB/MMB/wheel/P/F，2D/面板/text、blur/resize、Run/Stop | 单一capture，导航不动内容/游戏；Stop恢复编辑view，输入相机与渲染一致 | T-04/05；native实际DPI＋unit多比例 |
| V-06 | AC-06 oracle/消费 | 新OS/GPU probe、旧相机/置顶或漏grid实际变异、恢复、partial、fresh consumer | 同门数值/深度/内容具名红；恢复绿；新AGP模块漏导出拒绝，consumer真实新模式 | T-05/06；不只改JSON或看截图 |

定向driver：`cargo check --locked`、新增`cargo test --locked --test editor_viewport3d -- --test-threads=1`（T-03/04）、原authoring_interaction/authoring_host/authoring_play_host；AGP改动用`cargo test --locked --manifest-path ../autographics-prototype/Cargo.toml render3d -- --test-threads=1`及新depth命名测试。完整AGP/两端/原新native门按共同合同只在最终review后执行；没有根整档门。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 856实际交付、唯一树/SDK/锁/写面/当前identity与工具核对；consumer闭包完整 |
| AC-02 | work | View3d/两投影/导航/焦点/聚焦/默认reset/支持界成立，真实矩阵一致；V-01 |
| AC-03 | work | 深度grid与轴世界对齐、可见/遮挡均正确，配额/复用/释放可核；V-02 |
| AC-04 | work | 全部相关pick/group/轮廓/手柄/真实Auto透视drag/吸附/历史一致，退化受控；V-03/04 |
| AC-05 | work | 输入相机登记、呈现revision、互斥/取消/Play恢复成立；V-04/05 |
| AC-06 | work | 实际OS/GPU/深度正反/consumer与原默认/855-856基线对照齐全，原分母不弱化；V-06 |
| AC-07 | review | 独立核work AC/V/SD/默认与新路径/资源计量、warnings/fmt；merge门pending |
| AC-08 | merge | 冻结两端与受影响AGP完整/原新native-owner-content门通过，复用有逐项输入证明 |
| AC-09 | merge | 源码合autographics-dev，SD/ledger/归档/guard清理有实际收据；只认领M2/S1而非整个M2 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 856交付 | 核实际基面/ViewState/工具/分母/允许diff，创建唯一树并记录入口 | AC-01 | [x] |
| T-02 | work | T-01 | view3d-decision：两种有限depth接法/实际透视drag探针，冻结公式/退化/计量/原默认兼容 | AC-02/03/04 | [x] |
| T-03 | work | T-02 | View3d、相机导航与grid depth接点、同device资源/计量；V-01/02 | AC-02/03 | [x] |
| T-04 | work | T-03 | interaction/Host投影拾取/轴drag/snap/view revision/输入登记/Play；V-03..05 | AC-04/05 | [x] |
| T-05 | work | T-03/04 | 新native probe/check、真实旧相机/深度变异恢复、current identity/consumer；V-06 | AC-01/06 | [x] |
| T-06 | work | T-05 | 原默认与新辅助性能/资源对照、定向收口、提交代码/SD/证据，不跑整档 | AC-01..06 | [x] |
| T-07 | review | T-06 | 独立最终review，核深度/实际相机/透视手势/计量；merge门pending | AC-07 | [x] |
| T-08 | merge | T-07 pass | 冻结两端/受影响AGP完整/原新native-owner-content门与复用审计 | AC-08 | [ ] |
| T-09 | merge | T-08 | 源码合autographics-dev，main沉淀SD/overview/plans/ADR/ledger/索引并归档 | AC-09 | [ ] |
| T-10 | merge | T-09 | guard clean清树/临时分支、实际10/10/恢复产物，记录S1剩余边界供S2 | AC-09 | [ ] |

## 9. 复审记录

2026-10-27 new r1：stage=new；outcome=blocked（856/855未实际交付）；next=856验收合入后核冻结接口/提交进入T-01。当前为语义草案，所有AC/V/T/SD未执行；855观察和856拟定接口均不是已交付证明。status保持drafting，不提前开工或宣称M2完成。

2026-10-27 work T-01：用户授权856完成后实施857。856已实际交付并归档10/10，源码autographics-dev冻结 `4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`；唯一工作树 `D:/autostack/.wt/lang-857/auto-lang`、分支 `plan-857-dev` 从该提交建立，路径/分支/clean已核。固定SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467` 验证681文件、17 local、479 packages、29 edges；856来源462项/两锁/consumer原闭包与双端300 ID见原交付收据，本树实际verify_source与prepare_sdk verify-only通过，日志 `D:/autostack/plan-artifacts/plan-857/work-t01/`。原2D ViewState/导航取消和GPU网格/输入坐标合同是已落地事实，3D仍固定Camera3d/pick/正交投影近似；AGP已有实际场景depth、screen_overlay无depth。允许写面/预算/API不变，T-02必须在实际producer→consumer路径中验证depth方案与透视拖动后冻结decision，不能只靠静态观察宣称通过。保留主检出其他会话WIP；executing1/10，尚未实现857、未跑整档或提交。

2026-10-27 work T-02：场景mesh strip与独立有界深度pass实际GPU对照，screen overlay反例遮挡错误；后者选同color/depth、无depth write/阴影、独立资源计量，实际共面可见/前景遮挡、关闭零上传、静态mesh零重传；临时关闭真实draw同oracle具名红后恢复绿。zero-bias变异在当前共面case仍通过，未证明bias必要；T-05须扩大实际共面/深度敏感性，不冒充已闭合。相机速度、投影匹配、有限逆残差及退化平面阈值冻结；大视野旧阈误拒与零矩阵对照、独立透视ray-plane/真实Host→Auto X轴+Undo通过。decision与逐文件SHA见attachments/857/work/view3d-decision.md + decision-inputs.json.txt，日志D:/autostack/plan-artifacts/plan-857/work-t02/。代码仍在专用树未提交，T-03..06未完成；不把Host测试当OS/native/consumer或完整门。executing2/10。

2026-10-27 work T-03..06：stage=work | plan_id=PLAN-857 | plan_revision=1 | outcome=pass | worktree_path=`D:/autostack/.wt/lang-857/auto-lang` | branch=`plan-857-dev` | base_commit=`4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7` | code_commit=`ecf9939c5b2e369f9ce7d6ea25b0b66439bb7da6` | dependency_revisions=SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`，两锁未变 | task_ids=T-03..06 | evidence=[工作完成记录](attachments/857/work/completion.md)，含当前来源身份、20项最终consumer定向测试、双端实际OS/GPU、三项实际变异及恢复、单场景性能/资源、原兼容定向档与证据复用边界 | blockers=无已知work内阻塞 | next_responsible_role=独立review。状态execution_done、6/10；尚未复审/合入。AGP受影响全量与producer/consumer完整及原新native-owner-content门均为merge-owned pending，不能以此工作记录代替。旧SDK 146项既有warnings保留；共面zero-bias单例仍通过，不声称bias必要或跨GPU证明。

2026-10-27 independent final R1：`stage: review | review_scope: final | reviewer_identity: 独立子agent /root/review_857（未参与实施） | plan_id: PLAN-857 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-857/auto-lang | branch: plan-857-dev | reviewed_commit: ecf9939c5b2e369f9ce7d6ea25b0b66439bb7da6 | base_commit: 4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7 | dependency_revisions: SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、两锁不变 | spec_inputs: attachments/857/review-r1/baseline.json.txt；delta快照spec-delta.md | acceptance_results: AC-01通过；AC-02/06部分；AC-03现有定向通过；AC-04/05失败；AC-07未通过 | pending_closeout_gates: 完整双端/受影响AGP/原新native-owner-content门仍归merge | findings: F-857-R1-01..05 OPEN | evidence: attachments/857/review-r1/review.md、持久反例及原始日志SHA | next: auto-plan-work在同树修复并定向验证，提交后复审受影响内容`。真实Host+GPU正控证明有效distance4096的21像素半高对象可见却无法选择；完全被近/远平面裁剪的组仍可触发手柄；中/右键交叉release均提前结束另一个捕获。另缺原默认3D像素与856冻结基线对照及855同条件性能/工作集对照。20+2+1个计划内定向测试通过不能覆盖这些反例；原合入整档未运行。T-03/04/05/06重开，执行进度2/10；所有发现为合同内阻塞，不能记债延期后判通过。

2026-10-27 work repair R1：`stage: work | plan_id: PLAN-857 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-857/auto-lang | branch: plan-857-dev | base_commit: ecf9939c5b2e369f9ce7d6ea25b0b66439bb7da6 | code_commit: e20b95e175f9184e13c00c78efa7b63349165276 | dependency_revisions: SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、两锁不变 | task_ids: T-03..06 | evidence: attachments/857/work/repair-r1/completion.md、evidence.json.txt、repair-r1-performance.json.txt | blockers: 无已知work内未完成项 | next_responsible_role: 独立复审受影响内容`。F-857-R1-01..03原始反例、邻接路径与正控已定向修复验证；F-04补856冻结默认/重置真实像素对照及变异灵敏度；F-05补855与857同条件CPU/工作集/上传/owner测量并标明导航附加负载。生产/全新consumer实际窗口及partial通过，源码树已提交；复审前不称reviewed，完整合入门仍由merge执行。此前SDK 146项warnings保留；主检出其他会话WIP及风险登记保留。

2026-10-27 independent final R2：`stage: review | review_scope: final | reviewer_identity: 独立子agent /root/review_857（未参与实施） | plan_id: PLAN-857 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-857/auto-lang | branch: plan-857-dev | reviewed_commit: e20b95e175f9184e13c00c78efa7b63349165276 | base_commit: ecf9939c5b2e369f9ce7d6ea25b0b66439bb7da6 | dependency_revisions: SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、两锁不变 | spec_inputs: attachments/857/review-r2/baseline.json.txt、spec-delta.md | acceptance_results: R1-01/02/03及05所查路径关闭；R1-04实际默认/Home正例通过但比较器不变量partial；AC-06未闭合 | pending_closeout_gates: 完整双端/受影响AGP/原新native-owner-content门仍归merge | findings: F-857-R2-01 OPEN | evidence: attachments/857/review-r2/review.md | next: auto-plan-work按diagnosis-r2.md修复比较器并定向验证后复审`。三项原反例和正控重放通过；独立9项修复测试、144拾取与126轴点投影检查通过。实际旧/新runner对照中，缺失重置报告或真实partial报告时比较器仍exit 0；重置报告runner SHA篡改全零时仍exit 0且reset_equal=true。两次最终复审未通过的原因和调整见 [诊断](attachments/857/review-r2/diagnosis-r2.md)。T-05/06重开，executing4/10；源码未修改，合入门未运行。

2026-10-27 work repair R2：`stage: work | plan_id: PLAN-857 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-857/auto-lang | branch: plan-857-dev | base_commit: e20b95e175f9184e13c00c78efa7b63349165276 | code_commit: 4d9fa8e8c4ad00f5c88d9d1c5be619f0c592b837 | dependency_revisions: SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、两锁不变 | task_ids: T-05/06 | evidence: attachments/857/work/repair-r2/completion.md、controls.json.txt、evidence.json.txt | blockers: 无已知work内未完成项 | next_responsible_role: 独立复审受影响内容`。两次未通过[诊断](attachments/857/review-r2/diagnosis-r2.md)所要求的缺报告、真实partial、错误runner、缺截图、不同像素及来源/截图收据等负控均在同一比较器非零，真实856与当前默认/Home正控通过；已停用不安全的历史脚本。producer冻结窗口和独立consumer窗口收据通过，consumer程序按逐文件不变证明复用。状态execution_done、6/10，未复审/合入；完整双端、AGP及原新native-owner-content门归merge。

2026-10-27 independent final R3：`stage: review | review_scope: final | reviewer_identity: 独立子agent /root/review_857（未参与实施） | plan_id: PLAN-857 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-857/auto-lang | branch: plan-857-dev | reviewed_commit: 4d9fa8e8c4ad00f5c88d9d1c5be619f0c592b837 | base_commit: e20b95e175f9184e13c00c78efa7b63349165276 | dependency_revisions: SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、两锁实际未变 | spec_inputs: attachments/857/review-r3/baseline.json.txt、spec-delta.md | acceptance_results: R1行为与性能证据按无关差异证明复用；R2原8负控和真实正控通过；AC-01/06身份链partial | pending_closeout_gates: 完整双端/受影响AGP/原新native-owner-content门仍归merge | findings: F-857-R3-01/02 OPEN | evidence: attachments/857/review-r3/review.md | next: auto-plan-work按diagnosis-r3.md收口身份不变量并定向验证后复审`。两份收据同时将locks改成README/Cargo.toml的真实hash，比较器仍exit 0且摘要误标锁；实际Cargo.lock文件未改。真实旧提交runner可由record_runner附会当前checkout身份，复制其真实旧Home报告并重组负控receipt后比较器仍将旧runner归为当前提交；旧/新Rust源码未变不能代替显式跨提交复用证明。原R2缺失/partial/错runner、截图与像素8负控已关闭，额外单侧错锁/SDK/commit/phase等8例正确拒绝。[三次未通过诊断](attachments/857/review-r3/diagnosis-r3.md)已记录；T-05/06重开，executing4/10；源码未修改，合入整档未运行。

2026-10-27 work repair R3：`stage: work | plan_id: PLAN-857 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-857/auto-lang | branch: plan-857-dev | base_commit: 4d9fa8e8c4ad00f5c88d9d1c5be619f0c592b837 | code_commit: 81b0974ac2512126790b0ef2af79235edf45b485 | dependency_revisions: SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、两锁不变 | task_ids: T-05/06 | evidence: attachments/857/work/repair-r3/completion.md、evidence.json.txt、current-controls.json.txt、reuse-controls.json.txt | blockers: 无已知work内未完成项 | next_responsible_role: 独立复审受影响内容`。F-857-R3-01固定精确锁路径并核收据/实际锁/两份来源；F-857-R3-02绑定受控构建来源，缺证明旧runner不标为当前，显式跨提交复用保留真实旧构建提交。原双侧伪锁及真实旧Home反例拒绝；当前构建及明确复用各12项比较负控与实际856/默认/Home正控通过，未授权复用与配置变化拒绝，2D相邻窗口通过。execution_done、6/10；尚未独立复审或合入，完整双端、AGP和原新native-owner-content等门仍归merge。

2026-10-27 independent final R4：`stage: review | review_scope: final | reviewer_identity: 独立子agent（未参与实施） | plan_id: PLAN-857 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-857/auto-lang | branch: plan-857-dev | reviewed_commit: 81b0974ac2512126790b0ef2af79235edf45b485 | base_commit: 4d9fa8e8c4ad00f5c88d9d1c5be619f0c592b837 | dependency_revisions: SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、两锁实测未变（2b131d3dd3…/c3df168e72…，跨e20b95e1→4d9fa8e8→81b0974a逐字节同） | spec_inputs: attachments/857/review-r4/baseline.json.txt、spec-delta.md（与R3冻结副本逐字同） | acceptance_results: AC-01通过；AC-06通过（R3两项身份缺口关闭）；AC-02..05实现与输入未变按无关差异复用且本轮真实856/857双80帧窗口正控重证默认/Home逐像素等；AC-07通过 | pending_closeout_gates: 完整双端/受影响AGP/原新native-owner-content门仍归merge（T-08） | findings: F-857-R3-01 CLOSED、F-857-R3-02 CLOSED、无新发现 | evidence: attachments/857/review-r4/review.md | next: auto-plan:merge（T-08..T-10），merge前执行冻结提交上的完整门，门禁通过前不得合入`。核实方式：静态审查9文件diff（全在tools/与来源清单，全部为新增检查或收紧，无删检/放宽）；被审方29个证据文件SHA-256逐项重算全符；独立重放锁/身份/复用类全部负控与正控（provenance套件12例、current与reuse两矩阵各12负控+真实窗口正控、无授权复用与配置变化拒绝、注册表回退record/verify_runner），且5个关键结果文件与被审方收据SHA逐字节一致；另自构4个收据外对抗用例全拒（AD-01单侧锁键伪造、AD-02重签名换芯构建证明、AD-03复用收据code_commit改标、AD-04重复JSON键），原始日志与SHA见review-r4。健康：py_compile 8工具过、无杂散调试输出、退出码语义明确；SDK 146项历史warnings未新增。merge-owned门禁pending显式登记，本pass不含这些门。

## 10. 待澄清事项

硬前置856交付已核实；T-02负责局部depth接法和冻结数值退化界，不得静默换成置顶grid、只测正交或取消某轴。若现有设备/接口无法提供受检深度或正确透视拖动，记录具体证据回new修订；不通过改SDK/原预算/原expected规避。
