---
plan_id: PLAN-875
status: archived
feature_name: AutoGraphics公共2D跟随与3D环绕运行相机
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-28

plan_revision: 1
supersedes_spec_components:
  - docs/specs/auto-graphics/scene-play-session.md
  - docs/specs/auto-graphics/auto-source-driver.md
new_spec_components:
  - docs/specs/auto-graphics/runtime-game-cameras.md
touched_goals: [GOAL-008]
affects: [auto-graphics]
current_step: 9
total_steps: 9
---

# [PLAN-875] AutoGraphics公共2D跟随与3D环绕运行相机

## 0. 变更摘要

在[874](874-autographics-public-character3d.md)实际交付后，补公共2D跟随/边界与3D跟随/环绕/缩放相机，把同dispatch相机水平基交给真实Auto角色。复用角色/Play/GPU，不复制编辑视口或另建游戏Host。

本稿是有依赖的语义草案，874尚未实施。T-01/02消费其交付后冻结实际profile/签名/计费；改变语义须先递增revision，不以本稿作为抢跑许可。

## 1. 目标

- 用户Auto公开配置角色跟随目标、2D矩形/视区或3D参数，无场景名专用Rust逻辑。
- 2D跟随已接受角色并夹界；3D真实鼠标左拖环绕/滚轮缩放，以角色为target。
- 同逻辑步的相机水平基→Auto角色→接受位置→最终camera形成数值/像素/输入闭环。
- 保持原暂停/故障/Stop/新Run/准备失败/owner；编辑视图、Grid、历史不被运行导航污染。

非目标：动态实例/触发/胜负/新局服务、纹理/动画/声音/HUD、通用camera节点持久格式、多camera/切镜头/stacking、遮挡避让、平滑弹簧/震动、生产AutoUI/ACE。每Run只支持一个选定维度的camera和一个角色。完整G3-01中的可辨识模型朝向仍由后继表现计划关闭。

## 2. 架构方案

编辑View2d/View3d和运行camera分别保存；运行camera是Run owned有限服务，复用原渲染面/矩阵入口，不进入Document/scene4、dirty或Undo/Redo。用户Auto声明配置并计算积分/相对移动，Host处理指针/投影等系统图形数学和受检发布，不计算角色碰撞/玩法。

系统输入→带Run/tick的快照→候选orbit/水平基→用户Auto更新→完整Frame准入→以该Frame角色构造camera→共同prepare/accept/publish。不能先装camera再发现角色错误，或拿上一Run/上一tick heading代替本步值。

## 3. 技术栈

实际入口：driver `src/authoring/host/navigation.rs::active_view2d`在Play中返回默认视图；`host/viewport.rs::{prepare_frame,publish,camera}`负责投影/呈现；`host/play_adapter.rs::pump_play`先prepare后accept/publish；`view2d.rs/view3d.rs`只管编辑语义。AGP `src/render3d/camera.rs::Camera3d`已有右手Y-up、wgpu深度0..1投影，非公共运行服务。

拟新增driver `src/authoring/runtime_camera.rs`、`tests/{runtime_camera_api,runtime_camera_host,runtime_camera_input}.rs`、`samples/runtime-camera/{two,three}/`、`tools/check_runtime_camera.py`；T-02确认局部分解。必要时有限修改AGP相机安装/组合面以支持事务/letterbox，优先driver复用；无新增SDK/依赖/物理库。

## 4. 需求分析与背景调查

授权：用户2026-10-28请求继续规划；本轮不实施/建树/新开agent/远端发布。依据[运行阶段设计](../design/autographics/m2-runtime-and-character-control.md)、[研究分母](../specs/auto-graphics/m2-game-sample-research.md)、[冻结关卡/输入](../design/autographics/m2-game-sample-matrix.md)、[编辑视口](../specs/auto-graphics/editor-viewports.md)、[Play](../specs/auto-graphics/scene-play-session.md)及[861角色](../specs/auto-graphics/game-actions-and-character2d.md)。new不改权威Specs。

只读锚点autographics-dev `8b12632d7b5b1b0df72fbb4eef84297a054de31d`，SDK逻辑pin/字节核对同[规划基线](attachments/874/planning/baseline.md)。**真实执行base须为874最终交付提交**，T-01记录SHA/差异/锁/SDK后冻结，不预填未来SHA。860默认3D相机下运动像素不变的限制须在本计划以真实取景解决，不能用其数值门冒称camera完成。

## 5. 详细设计

### 5.1 入口、身份和可行性

唯一树`D:/autostack/.wt/lang-875/auto-lang`、分支`plan-875-dev`，874实际交付后从其AG提交建树，源码只合autographics-dev；同树顺序work/独立review/merge，无junction/symlink，默认master建组脚本不能直接套用。

显式新组合profile或等效新身份隔离旧behavior/1..5、character/1及874公开版本，不能偷偷改旧装配。T-02在`attachments/875/work/runtime-camera-contract.md`先冻结选择/签名/参数/错误/计费、输入输出和全部消费者，名称/内部编号是设计输出，不假造已冻结值。保持scene4格式，用户Auto app_start配置并绑定本Run角色；冻结后只允许规定的系统orbit，改配置需Stop/新Run。

2D可配置矩形/视区，3D可配置yaw/pitch/distance及有限投影；默认只为858样板。第二套公开参数对照必须证明Host不写死G2/G3或场景名。未知/非角色/错维/重复绑定/NaN/非正距离/退化投影/视区大于矩形受控拒绝，不回退旧camera伪成功。

T-02先做有界整路径检查：六动作/水平基/角色move/支撑/camera输出/128节点。保持32credits、1000steps/slice、100000steps/callback、120resumes/250ms、256/64/320KiB源上限、30s初始化/2s整tick/8ms每轮目标；不足回new，不抬预算或拆callback。不依赖874未验证的“未来资源余量”。

### 5.2 相机/输入确定语义

**2D样板**矩形x[0,30],y[-4,12]、视区12×8；center=`(clamp(actor.x,6,24),clamp(actor.y,0,8))`，不插值。视区在800×380 child内等比居中letterbox为570×380、左右各115 logical px，边条不接游戏指针，不拉伸坐标。自定义矩形/视区按同一规则求投影/内容区，不写死上述常量。旧无camera profile默认像素保持。

**3D样板**右手Y-up，target=本步接受角色中心；distance初始8夹[4,12]，pitch初始45°夹[15°,75°]，yaw初始0并规范化。eye=`target+d*(sin(yaw)*cos(pitch),sin(pitch),cos(yaw)*cos(pitch))`，up=(0,1,0)，竖FOV60°、near .05/far256、aspect=实际3D内容区。yaw0时eye在+z上方，水平forward=(0,0,-1)、right=(1,0,0)。只覆盖858有限地图，无camera遮挡避让。

Running且3D游戏面拥有输入焦点时LMB drag：yaw+=.25°×dx，pitch+=.25°×dy（逻辑pixel，向下正）；wheel每notch distance乘`1.1^(-notches)`再夹界。物理→逻辑只转换一次，沿用856每40逻辑pixel/notch、每事件±8notches。拖动始于内容区、无首帧跳变；Up/blur/leave/Pause/Stop/Fault/resize/DPI/模态清capture/未消费delta，迟到move/up无效。2D不orbit/zoom；面板/文本/编辑导航不能同时消费。

固定大小输入累加器，T-02冻结有限累计/饱和与诊断，禁止无界队列。Starting/Paused/Faulted不接受orbit；中立Step无orbit/zoom，但camera随该中立逻辑步的角色位置更新。Resume需新按下拖动；新Run恢复初值。不能角色暂停而camera继续漂移。

### 5.3 同tick相对运动和共同发布

每dispatch消费一次orbit后得候选yaw/pitch/distance和水平正交基`F=(-sin(yaw),0,-cos(yaw))`、`R=(cos(yaw),0,-sin(yaw))`。Auto计算`F*forward_axis+R*right_axis`并归一，再乘用户speed/dt；pitch/zoom不改水平速度，最后非零方向由Auto保留供后继表现用。本计划不扩通用yaw writer。

输入身份与角色/camera共用Run/tick；方向读取和实际呈现同候选yaw。全部Frame及camera有效才共同prepare/accept/publish；OS重复重绘只复用last-good，不重复消费输入或推进逻辑。错Run/tick、连续orbit、丢uniform、prepare/install失败需要区分性case。失败不能留下新camera配旧actor，保上一完整两者并按原Fault/闭VM规则处理；初始化失败也不能破坏编辑恢复资源。

Stop复用原编辑Prepared与View2d/View3d恢复，不重编/import/重传静态mesh；runtime Grid/handles关闭，编辑Grid/snap/view原值保留。不累积轨迹，每Run仅候选/last-good camera和有限input。20轮运行/环绕/resize/Pause/Fault/Stop/关闭验证camera/input/VM/GPU最后weak owner及静态资源无重复上传；首次分配与动态uniform上传分开计量。

### 5.4 公共工程、真实内容和来源

两维最小公共工程各两套配置，证明目标/视区/投影/源码参数敏感性。真实OS环绕90°后同W世界移动应变90°、对角速度不变；角色可见、取景随位置、地标投影随orbit/zoom改变。数值AND最终合成内容，不能只看矩阵/日志。冻结camera/仍世界轴/上一tick基/漏uniform真实变异同门红、还原绿，绑定source/config/Run/tick/retained runner。

current identity/verifier/consumer包括全部build配置/夹具/新文件，易变证据不进入自身闭包；own-SDK导出包离线实际运行两维源码，不用生产者target/SDK绝对路径，必要补齐逐文件来源/hash。每维常规/128各5新进程、每类≥30有效暖步，按853分报Auto/camera/准入/prepare/上传/工作集，原始报告/截图/探针本地保留；attachments仅Markdown摘要/契约可提交。编辑视口数据不能替代运行camera测量，不承诺60Hz。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/runtime-game-cameras.md | 编辑/固定camera→公开有限运行配置/相对基/输入/事务/资源 | 公共运行服务 | AC-02/03/04/05 |
| SD-02 | modify | docs/specs/auto-graphics/scene-play-session.md | 角色整帧→camera与角色同tick/Pause/Step/Stop/失败恢复 | 防跨帧错配 | AC-04 |
| SD-03 | modify | docs/specs/auto-graphics/auto-source-driver.md | 无公共camera→新身份/准入/选择/consumer | 工程可复现 | AC-01/02/05 |

## 6. 测试设计

实施自查→同树独立最终review→冻结merge门，未指定模型/新agent。新入口T-03/04建立，以下均未执行。

| Case ID | AC/不变量 | 输入/实际入口 | 区别错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-02 准入 | runtime_camera_api/SourceSnapshot；两维合法与错维/目标/NaN/退化/重复配置 | 合法公开运行；错输入具名阶段拒绝，不fallback；旧装配保持 | T-02/03；单camera/Run |
| V-02 | AC-03 2D几何 | runtime_camera_host；x2/15/28、y-3/4/11、自定义矩形/视区 | 独立clamp结果x6/15/24、y0/4/8，实际570×380 letterbox；世界点投影/像素一致 | T-03/04；固定视区 |
| V-03 | AC-03 3D输入/几何 | runtime_camera_input/OS窗口；90°orbit、pitch/distance双极值、DPI/文本/leave/迟到事件 | eye/投影/实际内容与Auto方向；W随yaw变、对角speed不变、暂停无漂移/迟到无效 | T-03/04；无遮挡避让 |
| V-04 | AC-04 事务 | runtime_camera_host；旧Run/错tick基、当步orbit+移动、坏Frame/prepare/install失败 | actor/camera同tick；失败保旧；冻结camera/上一tick基/漏uniform实际变异红还原绿 | T-03/04；列明可注入失败阶段 |
| V-05 | AC-04/05 寿命 | 20轮含故障/resize/Stop；非默认编辑view/Grid/snap | Stop原像素/相机/历史、最后weak owner释放，无静态mesh重上传，旧角色/视口门仍过 | T-04/05；当前单窗 |
| V-06 | AC-01/05 来源/预算 | 两维own-SDK包、checker、五进程/128、超限 | 真离线消费；partial/0selected/错runner/旧报告拒绝；原限额无半帧，数值AND合成内容 | T-04/05；非完整游戏 |

driver定向：`cargo check --locked`、`cargo fmt --check`、`cargo test --locked --test runtime_camera_api --test runtime_camera_host --test runtime_camera_input -- --test-threads=1`，受影响Play/角色/视口命名case和新checker实际报告/runner命令＋`--controls`，T-04记录可复现命令，help不算验收。若改AGP做相机/组合定向测试；final review后merge冻结完整两端/AGP触面/原新native-owner-content，根crate零改动不触根档；复用证明代码/依赖/配置/输入未变。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 874真实交付SHA/SDK/锁/唯一树/current source/consumer核实 |
| AC-02 | work | 新组合身份/配置/输出/计费先冻结，全路径可行性与V-01成立，旧保持 |
| AC-03 | work | 2D跟随/夹界/letterbox及3D有限orbit/zoom/相对移动公共可操作，V-02/03 |
| AC-04 | work | 同Run/tick事务/失败last-good、Pause/Step/Stop/owner成立，V-04/05 |
| AC-05 | work | 两配置/OS/数值AND内容/负控/fresh包/资源闭合，V-05/06 |
| AC-06 | review | 独立沿真实路径核AC/V/SD/完整准入/资源/遗漏/fmt/warnings；merge门pending |
| AC-07 | merge | 冻结完整两端/AGP触面/原新门通过，不掩盖skip，复用证据有输入证明 |
| AC-08 | merge | 仅AG交付、知识/ledger/归档/guard清理实绩，后继对象/表现/游戏仍pending |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 874实际交付 | 唯一树、实际SHA/SDK/锁/公开角色/current资源/分母核对 | AC-01 | [x] |
| T-02 | work | T-01 | 冻结runtime-camera-contract身份/配置/输入/输出/事务/资源与全路径检查，必要先修订 | AC-02 | [x] |
| T-03 | work | T-02通过 | 新公开配置/assembly/Run camera/输入/相对基/共同发布及tests V-01..04 | AC-02/03/04 | [x] |
| T-04 | work | T-03 | 两维公共样板/OS/GPU/真实负控/故障/owner/Stop/identity/consumer，V-02..06 | AC-01/03/04/05 | [x] |
| T-05 | work | T-04 | 五进程常规/128采样、旧面定向/fmt、代码提交/SD/证据摘要 | AC-01..05 | [x] |
| T-06 | review | T-05 | 同树独立最终review，原始数值/图像/输入/故障/资源；完整门pending | AC-06 | [x] |
| T-07 | merge | T-06 pass | 冻结两端/AGP触面和原新门，失败回work/受影响复审 | AC-07 | [x] |
| T-08 | merge | T-07 | 仅AG源码合入，权威Spec/ledger/索引沉淀和归档 | AC-08 | [x] |
| T-09 | merge | T-08 | guard clean清树/分支/组目录，保留必要本地产物，实际9/9收据 | AC-08 | [x] |

## 9. 复审记录

2026-10-28 new r1：`stage: new | plan_id: PLAN-875 | plan_revision: 1 | outcome: blocked | next: 874实际交付后T-01/02冻结实施基线`。blocked为前序依赖未满足，草案可评审。全部AC/V/T/SD未执行，无实现/建树/Cargo/native/GPU/采样/新agent/远端操作，不能假装已冻结874接口和成本。

2026-10-28 work T-01/T-02：`stage: work | plan_id: PLAN-875 | plan_revision: 1 | outcome: pass(T-01/T-02) | worktree_path: D:/autostack/.wt/lang-875/auto-lang | branch: plan-875-dev | base_commit: c02a9c2227f54542082abb35608d828fc4dbf674（874最终交付tip，8b12632d7规划锚+874的6提交） | dependency_revisions: SDK 681保留副本（plan-874 consumer-package复制后 prepare_sdk --verify-only PASS） | task_ids: T-01,T-02 | code_commit: a3434978a | evidence: (T-01)树/分支/toplevel核过、工作区零WIP；两 Cargo.lock blob 与874基线逐字节一致（47176cc0/bc7287b5）、AGP camera.rs 2bc2d28c 未变；`python tools/prepare_sdk.py --verify-only` PASS（681文件/17本地包）；`python tools/verify_plan874.py --controls` PASS（685哈希+篡改负控）；`cargo check --locked` exit 0（仅SDK既有146 warnings）；冻结限额在树核对：32credits（`_ag_spatial_calls>32` fault10）、256/64/320KiB、30s/2s/8ms/250ms。(T-02) [合同冻结](attachments/875/work/runtime-camera-contract.md)：新身份 `autographics.runtime_camera2d/1`(103/14) 与 `runtime_camera3d/1`(104/15)，装配=v5全链+对应character单元+camera单元，公开签名 ag_camera_follow_2d / ag_camera_orbit_3d+ag_cam3_*，错误码27/29/30，相机读0计费，事务=Frame携带camera经prepare/accept/publish原子发布，letterbox边条draws覆盖+指针惰性（零AGP修改）。**有界整路径实测**（874 sampler 方法学，128节点稀疏宽场景、同控制器对照）：`_ag_tick` p50 base 52.705ms→camera 53.012ms（+0.6%，≤10%门）、credits 双方恰 32/32（相机读免费）、init 42.5ms（≪30s）、Host整事务 p95=49.5ms（2s内）；90° orbit后世界位移=-F(90°)=+x（dz漂移≈3e-9）。**发现并闭合7处设计点**（合同§9）：Session参数上限4→拆basis/orbit两笔；powf(-0.0)非恒等→零缩放跳过；cam_sent事务级（backlog暂停不得解除echo配对，实测红后修正）；VM trig 返回 double→基由Host计算+Auto常数归一；紧凑单元禁撇号；Auto词法`a-b`单标识符；相机查询须在camera_dim接线后追加。**分母结论**：128节点=空间场景分母（874裁定延续），碰撞成员=角色+2结构体；127 solids 绑定实测越过100000步init预算（永久反例保留）。探针源码本地留存 attachments/875/work/tmp_bounded_probe.rs.bak（不入库） | blockers: 无work内阻塞 | next_responsible_role: T-03（tests V-01..04+样板+checker）。

## 10. 待澄清事项

无额外近期用户产品决策。874交付后先证明组合profile在原预算可行；不足保留证据回new，不先做半份后删相对运动或改分母。874＋875不构成完整游戏或M2，后继继续滚动规划对象/触发、表现/音频/HUD与独立游戏。

2026-10-28 work T-03/T-04/T-05：`stage: work | plan_id: PLAN-875 | plan_revision: 1 | outcome: pass(work面) | worktree_path: D:/autostack/.wt/lang-875/auto-lang | branch: plan-875-dev | base_commit: c02a9c222 | code_commit: a3434978a+e34a3f98d+29a0a1791+eae6c226d（4提交，工作树零WIP） | dependency_revisions: SDK 681副本不变、两Cargo.lock字节不变（blob 47176cc0/bc7287b5）、AGP 零改动 | task_ids: T-03,T-04,T-05 | evidence: (T-03)V-01 `runtime_camera_api` 12/12（两维身份103/14、104/15；错维函数编译拒；未知/非角色目标fault 30；八种退化域fault 27；重复/冻结后配置29；缺配置27；直接5参调用被Session传输profile拒；基/轨道镜像逐位；2D块回显；相机读0计费=32上限对照证明）；V-02/V-04 `runtime_camera_host` 10/10（2D夹界x2/15/28→6/15/24+letterbox 570×380 bars=115px独立oracle+第二配置506.67×380；3D view_proj==glam逐位oracle（含ortho16）；co-publish=本步actor而非上步；故障保last-good对；NaN配置具名拒绝；无camera profile保持860默认采样相机；暂停orbit丢弃/中立Step跟随/饱和诊断；暂停zoom丢弃/±8notch夹界；orbit90°世界位移旋转）。修复screen_2d缺content半宽偏移的真实投影bug（此前被自洽断言掩盖）。V-03 `runtime_camera_input` 1/1=真实OS窗口探针（`--runtime-camera-check`，16 phase全绿：0.25°/px拖拽至yaw 90、旋转基世界位移dx=+1.333/dz≈0、滚轮8→8.8、blur清捕获+迟到move惰性、resume无press不orbit、新press-drag orbit至100、Stop恢复编辑相机、饱和0）。两维样板 `samples/runtime-camera/{two,three}`（各main/main-alt两套配置）。(T-04)identity `verify_plan875 --controls` PASS（702哈希+删件/未列件/错哈希负控）；`verify_source.py` 三段PASS（identity/SDK681/479包17local29边）；`check_runtime_camera.py --controls` PASS（partial窗口/19环host/缺进程sampling全拒）；20环host lifecycle测试（单Host×2维各10环：VM weak owner环环关闭、Stop恢复呈现标记、上传deltas非故障环逐字节平坦）；fresh own-SDK consumer包1367+文件 `cargo test --offline --locked` api 12/12+host 10/12→12/12（修libtest默认2MiB栈冷构建溢出：全部测试体改64MiB scoped fn-pointer线程，裸cargo t兼容）。(T-05)采样矩阵 4形×5新进程×50暖步：camera2d-regular p50 3.94ms/p95 5.08ms、camera2d-128 42.3/57.7（max 73.6）、camera3d-regular 4.18/5.68、camera3d-128 41.9/45.2（max 64.2），全部≪250ms回调限、camera_block逐tick在帧；旧面定向 lib+character3d 22+12、character2d 10+8、game_input 6、authoring_play 16、authoring_behavior 9 全绿；`cargo fmt -p autographics-auto-driver -- --check` exit 0。**SD草案**（worktree，merge发布）：SD-01 `docs/specs/auto-graphics/runtime-game-cameras.md`（新）+ SD-02/03 增量至 `scene-play-session.md`/`auto-source-driver.md`。原始报告/截图/探针本地保留（plan-artifacts/plan-875/ 与 research/，未入库） | blockers: 无work内阻塞 | pending_closeout_gates（merge-owned，不因本记录视为完成）: 冻结两端独立workspace完整 `cargo test --locked`、AGP触面与原新native-owner-content复用审计、批量回归到期判定、spec沉淀与ledger/归档/guard清理 | next_responsible_role: 独立最终 review（T-06）。

2026-10-28 goal r1（work stage 完成）：`stage: goal | plan_id: PLAN-875 | plan_revision: 1 | round: 1 | from: work | to: review | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-875/auto-lang | commit: 2da1d3b79（T-01..T-05 共5提交 c02a9c222..2da1d3b79） | next: review`

2026-10-28 independent final R1：`stage: review | review_scope: final | reviewer_identity/limitations: 独立子代理审计上下文（general-purpose agent，读取冻结树+合同+Plan，60次工具调用，未运行构建/测试；构建/测试证据由主会话在同树冻结提交上重跑） | plan_id: PLAN-875 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-875/auto-lang | branch: plan-875-dev | reviewed_commit: 1cc68aba012f76951af2556763a5fb21be6a6c8b | base_commit: c02a9c222 | dependency_revisions: SDK 681副本不变、两Cargo.lock blob=874基线（47176cc0/bc7287b5）、AGP零改动（diff零触prototype） | spec_inputs: docs/specs/auto-graphics/runtime-game-cameras.md@1cc68aba0（SD-01新）+ scene-play-session.md/auto-source-driver.md PLAN-875增量（SD-02/03） | acceptance_results: AC-01 PASS（HEAD/干净树/锁与AGP blob逐一核对）；AC-02 PASS（两新身份103/14、104/15，旧7版本与装配臂逐字未变，入口钩子链与合同§1一致）；AC-03 PASS（冻结域逐条在单元实现：2D排序+视区入矩形、3D distance[4,12]/pitch[15,75]/proj{0,1}/ortho[1,64]/yaw=0起，fault 27/29/30，相机读零计费；runtime_camera.rs基/夹界/letterbox含content半宽/投影常量全部核对，unit测试断言）；AC-04 PASS（admit_camera全部具名拒绝：run/tick回显逐位、tick0初始yaw、配置漂移；fold零pending跳过powf；cam_sent事务级；publish与instances同事务；pause/stop/blur/resize清捕获；scroll仅3D相机Run路由；2D bar惰性；迟到事件惰性）；AC-05 PASS（api 12/12+host 12/12+lib 22+input探针1/1于冻结提交重跑；verify_plan875 --controls PASS 702哈希；checker三族controls全拒；consumer包离线12+12）；AC-06=本记录；AC-07/08=merge-owned pending。修复回路：R1审计发现F-875-R1-01（low：three样例face保留要求双分量非零，轴对齐移动不更新，违反§5.3保留语义的样例参考）+F-02（probe诊断未插值）+F-03（3D单元注释笔误）→ 同树修复提交1cc68aba0，受影响面（api/host/input/lib+identity+consumer离线）全绿后本记录绑定新提交；F-04（§9收据14/16 phase笔误）已改；F-05（assembly负控覆盖深度）记为边界不改。coverage_boundary: 2D表面滚轮在3D相机Run仍走view2d.wheel（基线行为，非本计划diff，留给后继Play/input计划）；模态清捕获经stop_play传递成立；"相机读免费"为结构+32/32对照证明非计数器检查；90°轨道探针±8°/测试±2°容差为合成输入合并抖动，0.25°/px率在settled记录精确 | pending_closeout_gates（merge-owned）: 冻结两端独立workspace完整cargo test --locked、AGP触面与原新native-owner-content复用审计、批量回归到期判定、spec沉淀/ledger/归档/guard清理 | findings: F-875-R1-01 CLOSED（样例修复+探针重放绿）、F-02 CLOSED、F-03 CLOSED、F-04 CLOSED、F-05 记录为边界 | evidence: 子代理审计报告（会话内可溯）；受影响面重跑命令与结果：cargo test --locked --test runtime_camera_api --test runtime_camera_host --test runtime_camera_input --lib -- --test-threads=1 → 22+12+12+1 全绿 exit 0；verify_plan875 --controls PASS（702）；consumer offline 12+12 PASS | next: merge（T-07..T-09）。

2026-10-28 merge T-07..T-08：`stage: merge | plan_id: PLAN-875 | plan_revision: 1 | outcome: pass(进行中: cleanup pending) | completion_kind: delivered | reviewed_commit: 1cc68aba012f76951af2556763a5fb21be6a6c8b | delivery_commit: 1cc68aba0 | landing: autographics-dev c02a9c222 -> 1cc68aba0（git merge --ff-only 经临时 ag-landing worktree，merge-base --is-ancestor 证明线性快进零rebase，worktree已移除；主检出他线WIP故未切分支） | gates(T-07): 冻结提交1cc68aba0上定向面一次跑齐 = lib(runtime_camera+全lib) 22 + runtime_camera_api 12 + runtime_camera_host 12 + runtime_camera_input(真实OS窗口探针) 1 + 旧面定向 character3d 22+12、character2d 10+8、game_input 6、authoring_play 16、authoring_behavior 9 全绿；AGP触面复用审计=两锁与camera.rs blob与874基线逐字节一致（47176cc0/bc7287b5/2bc2d28c）+ diff零触prototype → AGP门复用874证据；consumer包离线 api12+host12；checker三族--window/--host/--sampling PASS+controls全拒；sampling 4形×5进程 p50 3.9-42.3ms 全部≪250ms。**完整AG套件按2026-10-28用户裁定不再作为合入门**（该套件约40分钟computer use占用用户机器），记里程碑档pending；批回归：.last-batch-regression.json last_covered=860、2026-10-28T09:33Z，875%5=0且晚于last_covered → **到期**，landed+cleaned后由 /auto-plan:regress 主检出单实例执行 | canonical_spec: docs/specs/auto-graphics/runtime-game-cameras.md（新，SD-01）+ scene-play-session.md/auto-source-driver.md PLAN-875增量（SD-02/03），autographics-dev（随1cc68aba0）与master（本提交）双线落地 | ledger: .autoos/specs.json v119->v120（guarded offline projection：字节级backup specs.json.bak-p875-merge、脚本化surgical编辑、scope守卫证明、schema六节+file/tags完整、读回验证；HTTP store不可达，Regime B ladder）；875四条（P875-CAM designs/P875-TESTS tests/P875-REVIEW-R1 reviews/P875-MERGE reports）+ **874四条pending追补**（P874-CHAR3D/P874-TESTS/P874-REVIEW-R1/P874-MERGE，其merge时ledger脏登记的欠账） | archive: 本文件 docs/plans/archive/875-autographics-runtime-cameras.md，status archived，9/9（T-01..T-05 work、T-06 review、T-07..T-08 merge） | pending: T-09 guard清树/分支/组目录+plan-artifacts中间产物回收 → cleaned 收据 | next: cleanup。

2026-10-28 merge 收尾（T-09 cleaned）：`stage: merge | plan_id: PLAN-875 | outcome: pass | completion_kind: delivered`。**cleaned**：wt-guard clean（无 reparse point）→ worktree D:/autostack/.wt/lang-875/auto-lang 移除、plan-875-dev 删除（tip 1cc68aba0 经 merge-base --is-ancestor 证明已在 autographics-dev；-d 因主检出在master被拒后按同线落地事实 -D）、组目录 lang-875 移除（ag-landing 临时worktree已在landing后即删）。plan-artifacts/plan-875：consumer-package target 树（4.2G）与全部 .pdb 回收；**偏差与修复**：首次回收把嵌在 target 内的 .exe 一并删除，违反保留裁定 → 以离线 --locked 重建（1m02s）恢复 autographics-authoring.exe/autographics-auto-driver.exe 至 exe-anchors/（168M，重建哈希与原测量产物不逐字节一致，已如实记录）后二次回收；保留 runtime-camera-reports/（window/host/sampling JSON+PNG）与 consumer 源码包（31M）。最终 4.2G->198M。**landing**：autographics-dev=1cc68aba0（ff-only 线性）；**master**：d5a3f3613（archive+specs+ledger v120，含 874 四条欠账追补）；master docs 提交后 git status 干净（他线既有 WIP 不变）。**批回归**：875%5=0 且晚于 last_covered=860 → 到期，由 /auto-plan:regress 在主检出单实例执行（本收据后）。**landing-not-deployment 检查**：本计划仅触 experimental/autographics-* 与 docs（后端二进制/web bundle/daemon 零接触）→ 显式观察项：无生产面产物需要重建。最终 9/9（T-01..T-05 work、T-06 review(独立子代理+修复复审)、T-07..T-09 merge）。
