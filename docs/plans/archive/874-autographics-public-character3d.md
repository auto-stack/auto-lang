---
plan_id: PLAN-874
status: archived
feature_name: AutoGraphics公共3D角色与水平动作
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-28

plan_revision: 1
supersedes_spec_components:
  - docs/specs/auto-graphics/auto-source-driver.md
  - docs/specs/auto-graphics/scene-play-session.md
new_spec_components:
  - docs/specs/auto-graphics/game-character3d.md
touched_goals: [GOAL-008]
affects: [auto-graphics]
current_step: 5
total_steps: 9
---

# [PLAN-874] AutoGraphics公共3D角色与水平动作

## 0. 变更摘要

消费860真实Auto三维探针和861公共2D角色交付，建立用户Auto源码＋真实OS输入驱动的单3D角色。先完成静态三维运动与最终支撑，再由[875](875-autographics-runtime-cameras.md)完成公共运行相机和相机相对运动闭环。

## 1. 目标

- 新显式profile支持单3D角色、固定静态solid、三维有限位移/最终支撑，旧profile保持。
- WASD/方向键、Space、Enter经单一有Run/tick身份的输入路径进入Auto；用户Auto负责水平归一、积分、跳跃和运动朝向数据。
- 公开准入、真实Session/Host/GPU、来源/预算、fresh consumer与最后owner闭环。

非目标：完整G3游戏、交互运行相机、动态实例/触发、危险/胜负/重开规则、通用旋转writer/模型动画、纹理/精灵/声音/HUD、坡面/移动平台/自由刚体、SDK/ACE/AutoUI迁移。Enter只交付动作；朝向先是用户Auto的最后非零水平向量，真实模型朝向由后继表现计划完成；本计划不关闭完整G3-01。

## 2. 架构方案

保留一个Host、一个Play/Session和原完整Frame准入/发布栈。受信Auto负责有限接触/几何支撑；用户Auto负责速度/重力/跳跃/归一及玩法；Rust负责系统输入、配置/输出准入和GPU。独立Rust几何oracle只用于测试，不进入正常玩法。

建议新命名域`autographics.character3d/1`，内部编号在T-02核冲突后冻结。不得扩大旧`autographics.character/1`四动作/二维形状或改旧behavior/1..5装配字节；未知behavior/6继续拒绝。assembly/SourceSnapshot/来源区域/Play/CLI与consumer一致识别新身份。

## 3. 技术栈

沿用experimental两独立workspace、旧SDK、现有wgpu/Iced/glam，无新物理依赖。driver实际入口：`src/authoring/{behavior.rs,behavior/assembly.rs,game_input.rs,play.rs,spatial.rs}`、`src/authoring/host/{play_adapter.rs,viewport.rs}`、`tools/{package_consumer.py,verify_source.py,check_character2d.py}`。

拟新增driver `api/character3d_v1.at`、`tests/{character3d_api,character3d_host,character3d_samples}.rs`、`samples/character3d/`、`tools/check_character3d.py`及current identity/verifier。新增路径由T-03/04建立，不冒称已有接口。优先不改AGP；当前取景只服务可见性，不宣称运行相机已完成。

## 4. 需求分析与背景调查

授权：用户2026-10-28告知860/861合并并要求继续规划；本轮仅Plan/设计，不实施、建树、新开agent、创建远端仓库或发布。

权威输入：[全局overview](../specs/overview.md)、[AG现状](../specs/auto-graphics/overview.md)、[探针](../specs/auto-graphics/game-runtime-feasibility.md)、[2D角色](../specs/auto-graphics/game-actions-and-character2d.md)、[Play](../specs/auto-graphics/scene-play-session.md)、[研究分母](../specs/auto-graphics/m2-game-sample-research.md)及[冻结关卡](../design/autographics/m2-game-sample-matrix.md)。new只提出Spec delta，不修改权威Specs。

核对结论：857已归档；860交付go；861 archived 9/9，最终交付/受影响复审提交`8b12632d7b5b1b0df72fbb4eef84297a054de31d`，当前autographics-dev同此提交。861两端完整93族通过，consumer曾需补构建栈配置/非identity夹具，不把导出成功当闭包证明。SDK逻辑pin仍`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`；历史对象重写后沿用861已验证的681文件保留副本，入场须验证字节，不虚构Git祖先。文件指纹见[规划基线](attachments/874/planning/baseline.md)。

源码观察：`character2d_v1.at`仅dim2/x-y/四动作，assembly显式匹配CharacterV1，GameInput四位，Host无W/S；860三维probe绕过用户准入，不能直接宣称公开交付。旧motion三维仍最多两接触，本计划不承诺通用碰撞solver。

## 5. 详细设计

### 5.1 实施入口和先决门

唯一树`D:/autostack/.wt/lang-874/auto-lang`、分支`plan-874-dev`，从上述AG提交创建；源码只合autographics-dev。默认建组脚本基面master，不得直接套用；主检出只簿记。同树顺序work/独立review/merge，review时暂停写入，无junction/symlink。

T-01复核实际源/SDK/两锁/current分母。T-02在`attachments/874/work/character3d-contract.md`先冻结公开签名、动作值、token/错误/支持与资源，再做有界整路径检查：128节点下三维move＋四方向读取＋跳边沿＋grounded以及未来camera方向接缝。通过才T-03；失败保留证据回new，不加预算/拆callback重置限额/改Rust算角色。

保持场景总节点≤128（含角色/组/装饰）、32credits/callback、1000steps/slice、累计100000steps/callback、120resumes/250ms、用户256KiB/managed64KiB/combined320KiB、初始化30s/整tick2s/每轮8ms目标。move精确成本须由实际charge路径计数冻结，不只抄861表中的25；观察差异解释留痕，不追改旧版本。

### 5.2 配置、动作与角色语义

每Run一个dim3叶角色，实际资源AABB半宽/高/深(.4,.5,.4)，仅平移祖先；显式静态solid集合、去重、同维叶、非角色，允许空集，所有节点合计≤128。先solid后bind的交叉成员也在freeze拒绝。未知ID/错维/组/不支持形状/非有限受控拒绝，不隐式换代理。freeze记录角色/solid世界xyz及中立初始支撑；tick边界核三轴与祖先影响，不能漏z。solid外部移动、角色绕过controller、freeze后改配置按明确Fault/last-good规则拒绝；具体检测时机T-02冻结，不夸称从未存在过渡态。隐藏solid仍碰撞。

新profile六动作Left/Right/Forward/Back/Jump/Restart，与旧四动作隔离。W/↑前、S/↓后、A/←左、D/→右；反向相抵0、水平对角归一后速度仍4。Space只边沿，Enter只用户读取；OSrepeat无新edge，快down/up保一个待消费edge，dispatch只消费一次。文本/面板无游戏输入，同键不进legacy。Starting/Pause/Stop/Fault/blur/leave/resize/DPI/模态/新Run清held/pending；中立Step全中立，恢复需新按下。固定大小快照，不建无界队列。

样板dt=1/60、speed4、gravity20、jump8、clearance.002，参数在用户Auto；一步一次角色更新。初始支撑实测，步前grounded才允许跳；地后墙、顶碰撞、边缘失支撑、墙顶落地、初始重叠status3、PrecisionStop/ContactLimit分别测试，不能把Fault当grounded或只读最后法线。

### 5.3 接缝、可见性和消费者

默认水平基F=(0,0,-1)、R=(1,0,0)；用户Auto参数改90°正交基后世界运动随之旋转，保存最后非零水平朝向。此为数学接缝，875才接同dispatch真实运行相机，874不改旧Frame静态yaw承诺。

两个平台布局＋用户speed4→5无Rust重编对照，使用既有合法小占位资源。实际OS三维跑动/跳跃在最终合成图像可辨识，合法Host取景须显式配置；不能复用860“G3像素不变”的限制冒称可见运动。数值AND三维画面绑定同Run/tick。故障保last-good，Stop恢复原编辑内容/相机；20轮成功/故障/Stop/关闭验证VM/spatial/GPU最后weak owner释放，非计数清零。

新current identity/verifier覆盖新增源/build配置/小夹具，旧manifest/装配不改，易变报告不纳入自身hash。fresh own-SDK consumer离线实际运行公开3D源码/checker；包必须含构建栈配置及必要夹具，若沿用证据侧补齐需逐文件来源/hash，不能借生产者target/绝对SDK路径。采样常规/128wide/128deep各5新进程、每类至少30有效暖步，按853分报Auto/查询/准入/prepare/credits/源码/内存/上传，不承诺60Hz。原始报告/截图/探针本地保留；attachments只提交Markdown摘要，含命令/退出码/关键数字/本地路径/hash。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/game-character3d.md | 三维probe→显式公共有限角色/动作/资源/错误与排除 | 对外合同 | AC-02/03/04/05 |
| SD-02 | modify | docs/specs/auto-graphics/auto-source-driver.md | 仅2D角色→新3D身份/准入/来源/consumer，旧保持 | 实际入口 | AC-01/02/05 |
| SD-03 | modify | docs/specs/auto-graphics/scene-play-session.md | 旧character seam→隔离3D路由/配置与生命周期 | 单一owner/调度 | AC-03/04 |

## 6. 测试设计

实施自查→同树独立最终review→冻结merge门；未指定模型/新agent。新增入口由T-03/04建立，以下均未执行。

| Case ID | AC/不变量 | 输入/实际入口 | 区别错误实现的oracle | work任务/覆盖 |
|---|---|---|---|---|
| V-01 | AC-02 准入/配置 | character3d_api；SourceSnapshot/assembly/Play；错版本/维度/shape、先solid后bind | 合法新源可运行，旧六种装配不变；错输入具名阶段拒绝，无未受检消费 | T-02/03；单角色静态集合 |
| V-02 | AC-03 输入 | game_input/新OS probe；四向/相抵/对角/repeat/快按/文本/暂停/Step/新Run | Auto实际edge/held/位移，单router、速度4、edge第二读0、清理后零漂移 | T-03/04；桌面键盘 |
| V-03 | AC-04 三维接触 | character3d_api/host；地墙顶/边缘/z薄墙/墙顶/重叠/128末序最近体 | 独立几何/运动学与支撑ID；stale-grounded/最后法线/漏z校验真实变异红并恢复绿 | T-03/04；有限两接触solver |
| V-04 | AC-04 事务/owner | Host错身份/非有限/超预算/prepare失败及20轮成功/故障/Stop/关闭 | 整tick/像素保旧、关闭VM、Stop恢复、最后weak owner释放，不以计数代替 | T-04；列明失败阶段 |
| V-05 | AC-01/05 用户/消费 | 两布局、4→5、90°基、OS/GPU、own-SDK包/current checker | 三维数值AND最终内容同tick；冻结输入/漏z输出/错runner/partial/0selected应红，还原绿 | T-04/05；非完整G3/相机 |
| V-06 | AC-05 资源 | regular/wide128/deep64 三类×五进程；128 宽与密集超限反例 | 实际charge及原hard limits、runner/source绑定；超限无半帧（guard 拒绝），不用oracle替代玩法 | T-02/05；无任意负载实时保证 |

driver定向：`cargo check --locked`、`cargo fmt --check`、`cargo test --locked --test character3d_api --test character3d_host --test character3d_samples --test game_input -- --test-threads=1`，受影响原2D/assembly/Play命名case，以及新checker报告/runner命令＋`--controls`。T-04记录其实际完整命令，help成功不算验收。final review后merge冻结producer/own-SDK consumer两个独立workspace完整测试、AGP触面和原新native-owner-content；根crate零改动不跑根cargo t/tf/tv/tu/docs_gen。批量到期仅路由既有流程。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 861交付、唯一树、SDK字节/锁/current source与fresh包闭包核实 |
| AC-02 | work | 新公开身份/资源合同先冻结，V-01成立，旧版本保持 |
| AC-03 | work | 六动作/归一/方向、真实OS/单router/edge/清理/neutral Step成立，V-02 |
| AC-04 | work | 实际Auto三维有限接触/最终支撑、事务/恢复/owner成立，V-03/04 |
| AC-05 | work | 两布局/源码变化/数值AND三维内容/consumer/资源/真实负控闭合，V-05/06 |
| AC-06 | review | 独立核AC/V/SD/来源/资源/遗漏/fmt/warnings，merge门pending |
| AC-07 | merge | 冻结完整两端及受影响原新门通过；复用证明代码/依赖/配置/输入未变 |
| AC-08 | merge | 仅AG落源码、知识/ledger/归档/guard清理实绩，交875真实SHA |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与产出 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 861交付，入场再核 | 唯一树/8b12632d7/SDK副本/锁/current分母核对 | AC-01 | [x] |
| T-02 | work | T-01 | 冻结character3d-contract与有界整路径预算/签名/状态检查，失败回new | AC-02/05 | [x] |
| T-03 | work | T-02通过 | 新Auto/API/assembly/准入/Play/样板与tests，V-01..03 | AC-02/03/04 | [x] |
| T-04 | work | T-03 | OS/GPU/真实负控/失败/owner/current identity/consumer，V-04/05 | AC-01/04/05 | [x] |
| T-05 | work | T-04 | 五进程/128采样、定向收口/fmt、代码提交/SD草案/摘要 | AC-01..05 | [x] |
| T-06 | review | T-05 | 同树独立最终review；完整门pending | AC-06 | [ ] |
| T-07 | merge | T-06 pass | 冻结完整两端/AGP触面/原新门；失败回work/受影响复审 | AC-07 | [ ] |
| T-08 | merge | T-07 | 源码仅合AG，主检出知识/ledger/索引沉淀和归档 | AC-08 | [ ] |
| T-09 | merge | T-08 | guard clean清树/分支/组目录，保留必要本地产物，实际9/9收据 | AC-08 | [ ] |

## 9. 复审记录

2026-10-28 new r1：`stage: new | plan_id: PLAN-874 | plan_revision: 1 | outcome: pass | next: work T-01→T-02`。861交付已核，T-02签名/资源为work内首门，不是已测通过。全部T/AC/V/SD未执行，仅只读源码/Specs与文档规划，无建树/Cargo/native/GPU/采样/新agent/远端操作。

2026-10-28 work T-01/T-02：`stage: work | plan_id: PLAN-874 | plan_revision: 1 | outcome: pass(T-01/T-02) | worktree_path: D:/autostack/.wt/lang-874/auto-lang | branch: plan-874-dev | base_commit: 8b12632d7b5b1b0df72fbb4eef84297a054de31d | dependency_revisions: SDK 681 保留副本（plan-860 consumer-package，prepare_sdk --verify-only PASS）、两 Cargo.lock 字节未变 | task_ids: T-01,T-02 | evidence: (T-01)树/分支/toplevel 核过；baseline 表 15 个 Git blob 逐项复核一致（behavior.rs 70283b2…、assembly a94182c…、game_input 2493277…、play 73b66ee…、spatial 969e0e5…、play_adapter 3162d07…、viewport 7695d3b…、navigation bd68ef2…、character2d_v1.at 3d8b28c…、sample main.at fc6e70c…、package_consumer 23818d8…、verify_source c900863…、driver Cargo.lock 47176cc…、prototype Cargo.lock bc7287b…、camera.rs 2bc2d28…）；cargo check --locked exit 0（仅 SDK 既有 146 warnings）。**来源面观察（非阻塞）**：`research/plan861-identity.json` 未入库（根 .gitignore `*.json` 母规则；工作区实测 verify_source.py 在 8b12632d7 即红），故 861 代际闭包当前不可执行；[T-02 合同](attachments/874/work/character3d-contract.md) 冻结身份（`autographics.character3d/1`、version 102、profile 13、区域 `<character3d_v1.at>`）、六动作/键位/边沿、freeze/三轴只读、fault 20..26、credits=**26**（24 move＋1 预约＋1 融合支撑，实测 oracle：move+6 次 action 读取恰好 32 绿、第 7 次 fault 10）、形状盒 0.8/1.0/0.8、`.agscene` 零改动。(T-02 有界整路径实测) **发现并闭合一处设计不可行**：以第二次 `_ag_motion_three` 全量三维扫描做最终支撑，在 debug VM 指令计数下使 128 节点单 callback 越过冻结的 cumulative 100000 steps（三维 move 在 112 节点即红，2D 同几何 128 仍绿）；改为新单元内等价**专用竖直支撑扫描**（顶面 ∈[脚底-0.01,脚底]＋严格足印＋取最高面，先做 3 读顶面判别；v5 单元字节与 charge 不变，且不覆盖 `_ag_cast_*` 槽位）。实测结果：方案形状 128 节点 wide（126 体沿 x、最近体末序）60 tick 全绿、裁剪 `x=1.598`=2.0−0.4−0.002；over-limit 反例（127 体全足印重叠）由未改动的 guard 拒绝并留为永久回归。blocking: 无work内阻塞；**待裁定**见 §10 第 1 条 | next_responsible_role: T-03。

2026-10-28 work T-03/T-04/T-05：`stage: work | plan_id: PLAN-874 | plan_revision: 1 | outcome: pass(work面) | worktree_path: D:/autostack/.wt/lang-874/auto-lang | branch: plan-874-dev | base_commit: 8b12632d7b | code_commit: 701ac107a+4db26e4be+1714f05c6 | dependency_revisions: SDK 681 不变、两锁不变、AGP 零改动 | task_ids: T-03,T-04,T-05 | evidence: (T-03)交付=`api/character3d_v1.at`（公开 `ag_char3_bind/solid/move/grounded/support`＋`ag_action_held/pressed` 0..5，`_ag_char3_map/_ag_char3_input`，freeze 三轴快照＋中立实测支撑＋交叉成员拒绝＋tick_begin 三轴只读）、`Api::Character3V1`（102/13，装配=v5全链＋3D 单元、不装 2D 单元、behavior/6 仍拒）、`Play` 的 `CharProfile{TwoD,ThreeD}` 路由（`_ag_char3_map/_ag_char3_input`、按 profile 动作位上限守卫）、viewport 单 router 6 键位（W/S/A/D＋四方向＋Space/Enter）、panel/commands/CLI 选择器；样例 `samples/character3d/{main.at,main-fast.at,layout_a,layout_b}`（3D cylinder 占位、A 平台/B 缺口）；测试 character3d_api **14/14**（V-01/02/03：身份、准入 23/24/25/22/26、三轴 20/21、六动作 held=63/边沿一次消费、credits 26 oracle、x/z 轴裁剪 2.598、天花板、边缘失支撑、墙顶落地 y=1.002、初始重叠 status 3、128 最近体末序、90°基）。回归：lib 18/18、character2d_api 10/10、character2d_host 8/8、character2d_samples 1/1、game_input 6/6、character3d_host 9/9、character3d_samples 1/1；`cargo fmt -p autographics-auto-driver -- --check` exit 0。(T-04)OS 窗口探针 `--character3d-check`（`authoring_probe/character3d.rs`，真实 Win32 键事件）passed=true、phase 10、202 frames：spawn[0,0.5,0]、W 使 z=−1.3333336、Space 跳 apex y=1.9999998、blur 自动暂停且恢复零漂移、Stop 恢复编辑视图；checker 四族报告（actions/host/layouts/sampling）+window 全过、5 条负控（partial/zero-selected/oracle-edit/trace-divergence/bound-runner-mismatch）全拒；identity 685 文件 `verify_plan874 --controls` PASS（删件/未列件/错哈希全拒后恢复）+`verify_source.py` 三段 PASS（identity/SDK 681/479 包 17 local 29 边）；fresh own-SDK consumer 包 1367 文件，包内 identity PASS + `cargo test --offline --locked --test character3d_api --test character3d_samples` = 14/14 + 1/1。(T-05)采样矩阵 3 形状×5 新进程×200 测步（弃 30 暖步）：regular(3 节点) P50 中位 1.167ms/P95 2.079ms、wide128(128) 18.505/49.762ms、deep128(64) 12.629/16.219ms，全部 <250ms 回调限、credits ≤26/32。**SD 草案**（worktree，merge 发布）：SD-01 `docs/specs/auto-graphics/game-character3d.md`（新）、SD-02/03 追加至 `auto-source-driver.md`/`scene-play-session.md`。blockers: 无work内阻塞；pending_closeout_gates（merge-owned，不因本记录视为完成）: 冻结两端独立 workspace 完整 `cargo test --locked`、AGP 触面与原新 native-owner-content 复用审计、`/auto-plan:regress` 批量档到期判定、spec 沉淀与 ledger/归档/guard 清理 | next_responsible_role: 独立最终 review（T-06）。

2026-10-28 work 收口（冻结提交定向面）：`stage: work | plan_id: PLAN-874 | plan_revision: 1 | outcome: pass | code_commit: 1714f05c6（701ac107a..1714f05c6 共3提交，工作树零WIP） | evidence: 冻结提交上定向面一次跑齐 = lib 18 + authoring_behavior 12 + authoring_behavior_host 4 + character3d_api 14 + character3d_host 9 + character3d_samples 1 + character2d_api 10 + character2d_host 8 + character2d_samples 1 + game_input 6 = **83/83 绿**；`cargo fmt -p autographics-auto-driver -- --check` exit 0；`verify_plan874 --controls` PASS（685 哈希）；身份/采样/窗口报告均为本地证据（`research/character3d-reports/` 与 `plan-artifacts/plan-874/`，未入库）。**未执行**整档 t/tv/tt/tb/tu/taa（merge-owned），未改 SDK/AGP/两锁，未改旧 profile 装配字节。 | blockers: §10 第 1 条待裁定（不阻塞 T-06 开始，但 T-07 前须有结论） | coverage_boundary_for_review: V-04"Host 错身份"仅由 shared `Play::admit` 既有 profile 用例（7→6，`play.rs` deadline_tests）＋本次身份/装配用例覆盖，**未**新增"producer 报告 profile 12/101 而装配为 13/102"的运行时错配用例（需在同模块内构造被篡改的 managed 源；评审可要求补）。 | next_responsible_role: 独立最终 review（T-06）。

2026-10-28 independent final R1：`stage: review | review_scope: final | reviewer_identity/limitations: 实施同会话/同模型（已声明，不声称独立会话），结论从冻结提交工件重建而非执行者摘要 | plan_id: PLAN-874 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-874/auto-lang | branch: plan-874-dev | reviewed_commit: d4ec7481b | base_commit: 8b12632d7 | dependency_revisions: SDK 681 未变、两锁未变、AGP 零改动 | spec_inputs: docs/specs/auto-graphics/game-character3d.md@d4ec7481b（sha256 6DC38BE4BFBEAEF096EA1A7E15ECE37743C9140D2AAD14BDBA19C2C0B5C26CC4）+auto-source-driver/scene-play-session 增量 | acceptance_results: AC-01 PASS（树/分支/HEAD 复核，工作区零WIP；identity 685 --controls PASS 复审重跑；consumer 1367 文件 identity PASS + 离线 14/1 由 work 记录并可由包内重放）；AC-02 PASS（装配身份 102/13 与 region、behavior/6 拒、旧 6 版本可解析；准入 fault 23/24/25/22/26 断言齐全）；AC-03 PASS（六动作 held=63/边沿一次消费/越界 fault 7、neutral Step、router 守卫、blur 清空经窗口探针）；AC-04 **PARTIAL**（接触/支撑/预算/原子发布/owner/Stop 有断言且绿；但合同 §4 两条规则**无任何断言覆盖**，见 F-874-R1-01）；AC-05 PASS（两布局 pit 对比、speed 4→5、窗口探针 passed/phase10/202 帧、checker 4 族+5 负控、采样 3×5 进程全部 <250ms）；AC-06 本记录（needs_fix）。 | findings: **F-874-R1-01（in-contract 覆盖缺口）**：合同 §4/§5.2 与 SD-01 明文规则"隐藏 solid 不改变碰撞""祖先(group)平移在 tick 边界被拒（三轴）"在冻结提交的 `character3d_api.rs`/`character3d_host.rs` **零断言**；复审自建探针因 scene 稳定 id 参数错配（把 `ag_set_local`/bind 指向错误 run 索引）未能提供证据——按规则记为**探针输入错误**并单独分类，不作为产品缺陷，但其结果不构成覆盖证据；代码面只有"扫描不含 visible 过滤"与"tick_begin 逐成员比对 worlds（derive 传递祖先）"的静态推理。关闭 oracle＝补两条断言（隐藏 solid 支撑仍为 1；group 平移后该 tick 边界 fault 20/21 且 last-good 不被半帧替换）并在冻结提交重跑 `character3d_api`。**F-874-R1-02（低severity/覆盖边界，可延后）**：Host 侧"producer 报告 12/101 而装配 13/102"的运行时错配无用例（work 已自报）；不阻塞 T-07，若 merge 门不做则记入 KNOWN-DEBT。 | evidence: 复审基线复核（toplevel/分支/HEAD/dirty=0/plan rev1）；identity --controls 与 `verify_source.py` 三段 PASS 于 d4ec7481b 重跑；探针源码与输出为本地临时件（已删除，未入库）；工作区在复审结束时重新核为零 WIP。 | pending_closeout_gates（merge-owned）: 冻结两端完整 `cargo test --locked`、AGP 触面与原新 native-owner-content 复用审计、批量回归到期判定、spec 沉淀/ledger/归档/guard 清理 | next: work 修复 F-874-R1-01（T-03 重开）→ 受影响面复审（仅该文件与受影响断言）。

## 10. 待澄清事项

当前无须新增用户产品决策。T-02决定局部签名/内部编号与计费；若原hard limits不支持必要组合，先保留证据回new，增加预算/SDK/依赖、改变旧版本或M2分母须由用户决定。875须等本计划实际交付再冻结执行基线。

**work 实测后新增一条待裁定（已裁定，2026-10-28）**：

1. **128 节点不变量与冻结 step 预算的组合边界 —— 已由用户裁定选择 (a)：接受实测包络**。实测（debug VM 指令计数，T-02/05，证据见 [合同§6](attachments/874/work/character3d-contract.md) 与 §9 记录）：旧 2D `character/1` 在"127 体全部与角色足印重叠"的密集几何下仍能容纳 128 节点；三维 `character3d/1` 由于 v5 三轴 sweep 的逐节点成本，等几何下整 tick 越过冻结的 cumulative 100000 steps——全量控制器（含跳跃/归一）在密集深度布局下支持到 **64 节点**，宽布局（128 节点、候选稀疏）与 128 节点静态场景仍绿，简单源（仅 move+grounded）到 112 节点。**裁定结果（2026-10-28）**：本 profile 节点不变量登记为"**宽/稀疏布局 128 节点；全足印重叠的密集深度布局 64 节点**"，已写入 SD-01（`docs/specs/auto-graphics/game-character3d.md` §Agreed contract），采样形状固定为 regular(3)/wide128(128)/deep64(64)。不修改任何限额、不改旧版本、不把玩法移入 Rust；over-limit 反例为永久回归（guard 拒绝、无半帧）。

2026-10-28 affected re-review R1-fix（F-874-R1-01 关闭）：`stage: review | review_scope: final(affected) | reviewer_identity/limitations: 实施同会话/同模型（已声明） | plan_id: PLAN-874 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-874/auto-lang | branch: plan-874-dev | reviewed_commit: c02a9c222（语义修复 634ab7fb5） | base_commit: 8b12632d7 | dependency_revisions: SDK 681 未变、两锁未变、AGP 零改动 | repair_closure: F-874-R1-01 CLOSED——`character3d_api.rs` 新增两条合同断言：(1) `hidden_solid_still_collides_and_supports`（隐藏地面在 freeze 实测支撑 grounded/support=1；隐藏墙仍在 x=3−0.4−0.002 处裁剪 200 tick）；(2) `ancestor_group_translation_is_refused_at_the_tick_boundary`（自建 3D group→角色/地面父子场景，app_update 内平移 group 后下一 tick 边界 fault 20、fault node=2=被携带 solid 的 run 索引，bypass tick 本身不报错）。受影响面 = 该文件：**16/16 绿**（含 2 条新断言）、`character3d_samples` 1/1、`cargo fmt -p autographics-auto-driver -- --check` exit 0、identity 重生成 685 哈希 `verify_plan874` PASS；其余 AC 证据在 c02a9c222 上代码/依赖/配置/输入未变（仅新增测试与身份重绑）故复用。F-874-R1-02 保持低 severity 覆盖边界（未修；merge 可记入 KNOWN-DEBT） | acceptance_results: AC-01..05 PASS（AC-04 由 PARTIAL 升 PASS） | pending_closeout_gates（merge-owned，不因本记录视为完成）: 冻结两端完整 `cargo test --locked`、AGP 触面与原新 native-owner-content 复用审计、批量回归到期判定、spec 沉淀（SD-01 发布 + SD-02/03 增量）/ledger/归档/guard 清理 | next: merge（T-07..09）。

2026-10-28 merge T-07..T-09：`stage: merge | plan_id: PLAN-874 | plan_revision: 1 | outcome: pass | completion_kind: delivered | reviewed_commit: c02a9c222 | delivery_commit: c02a9c222 | landing: autographics-dev 8b12632d7 -> c02a9c222（`git merge --ff-only`，经 autographics-dev 临时 worktree 落地，`merge-base --is-ancestor` 证明线性快进，零 rebase；主检出 dirty（他线 WIP）故未切分支） | gates: **merge 全量 AG 套件按 2026-10-28 用户裁定不再执行**（该套件约 40 分钟 computer use，期间用户无法用电脑）；本计划以冻结提交 `c02a9c222` 上的**定向面 83/83**（lib 18 + authoring_behavior 12 + authoring_behavior_host 4 + character3d_api 16 + character3d_host 9 + character3d_samples 1 + character2d_api 10 + character2d_host 8 + character2d_samples 1 + game_input 6）＋1 次全量尝试结果（除 `authoring_probe_exit::asset_preparation_precedes_clipped_widget_scroll` 一次窗口所有权瞬时红外全绿，该 case 隔离重放 2 连绿归因环境；`authoring_interaction` 全量轮次曾挂起、隔离 7/7 绿 6s 归因环境）＋identity 685 `--controls` PASS＋checker 4 族 5 负控 PASS＋窗口探针 passed＋fresh consumer 离线 14/1 作为交付证据；**里程碑档 pending**（全量 AG suite 留待里程碑单独执行）。 | canonical_spec: docs/specs/auto-graphics/game-character3d.md（新，SD-01）＋ auto-source-driver.md/scene-play-session.md PLAN-874 增量（主检出已提交，含 2026-10-28 用户对节点包络的裁定） | ledger: `.autoos/specs.json` 主检出存在他线未提交改动，本次**未**写入（避免覆盖他人 WIP，登记为 pending/blocked 项，待 ledger 干净后由 `/auto-plan:merge` 追补或里程碑补录） | archive: 本文件 `docs/plans/archive/874-autographics-public-character3d.md`，`status: archived` | cleanup: wt-guard clean 后移除 worktree/branch，`plan-artifacts/plan-874` 保留 `.exe`/日志并回收 target 与 `.pdb` | 批回归: 874%5=4≠0，沿用 `.last-batch-regression.json` 判定（见收执） | 最终 9/9（T-01..T-05 work、T-06 review(含 R1 修复复审)、T-07..09 merge）。


2026-10-28 merge 收尾（T-09 cleaned）：`stage: merge | plan_id: PLAN-874 | outcome: pass | completion_kind: delivered`。**cleaned**：wt-guard clean（无 reparse point）→ worktree `D:/autostack/.wt/lang-874/auto-lang` 移除、`plan-874-dev` 删除（tip c02a9c222 经 `merge-base --is-ancestor` 证明已在 autographics-dev）、组目录移除；`plan-artifacts/plan-874` 回收 consumer-package 的 cargo `target` 树与 `.pdb`（保留 `.exe`/日志/源码/证据）。**archived**：`e455fbad6` 把本计划移入 `docs/plans/archive/`（status archived）并与 SD-01/SD-02/03 增量、AGENTS.md 的 AG 测试分级裁定同提交落 master。**batch_regression**：`.last-batch-regression.json` last_covered=860、timestamp 2026-10-28T09:33Z（今日，<48h），874%5=4≠0 → **不到期**。**里程碑档**：全量 AG suite 按 2026-10-28 用户裁定留待里程碑单独执行（AGENTS.md §AutoGraphics Test Tiering 已登记）。**ledger 未写**（主检出 `.autoos/specs.json` 存在他线未提交改动，避免覆盖他人 WIP；登记 pending：待 ledger 干净后补录 designs/tests/reviews/reports 四条并跑 `scripts/spec-index.py`）。
