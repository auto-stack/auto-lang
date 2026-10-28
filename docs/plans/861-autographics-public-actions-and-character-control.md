---
plan_id: PLAN-861
status: reviewed
feature_name: AutoGraphics公共动作输入与2D角色控制
author: [Codex]
created_at: 2026-10-27
updated_at: 2026-10-28
plan_revision: 1
supersedes_spec_components:
  - docs/specs/auto-graphics/auto-source-driver.md
  - docs/specs/auto-graphics/scene-play-session.md
new_spec_components:
  - docs/specs/auto-graphics/game-actions-and-character2d.md
touched_goals: [GOAL-008]
affects: [auto-graphics]
current_step: 6
total_steps: 9
---

# [PLAN-861] AutoGraphics公共动作输入与2D角色控制

## 0. 变更摘要

M2/S2首个公共运行能力切片：以860明确go和冻结机制为前置，让用户Auto脚本通过公开动作输入和有限2D角色服务完成跑跳、地面/墙顶判定与Stop/再Run。3D角色在后继单独扩展，不把编辑相机或现有两接触位移当作完整控制器。

本Plan为有条件语义草案；860尚未执行，具体接触算法/签名/状态物理上限不得假装已有结论。T-01/02须先从其实际交付冻结接口合同；如需改变本文支持面/预算/验收，先修订revision，不直接进入实现。

## 1. 目标

- 支持Editing选用一个显式新角色实验profile，旧behavior/1..5、scene4和默认旧路径保持。
- 实际OS输入进入有Run/步身份的动作快照，Auto读取held/pressed，正常文本/编辑/导航不能驱动角色。
- 单个2D角色按固定dt、速度/重力/跳速、静态轴对齐solid集合完成有界控制；正确处理空中跳、顶/墙/边缘和最终支撑。
- 用户修改Auto参数/控制逻辑后Stop/Run得到真实变化，无专用游戏名/Rust玩法分支；公开源/诊断/准入/consumer/owner闭环。

非目标：完整G2游戏、3D角色、动态对象/触发服务、运行相机、精灵/动画/声音/HUD、自由刚体/坡面/移动平台、土狼时间/跳跃缓冲、通用旋转writer、资产下载/仓迁移/HIR/ACE。保持面见[共同合同](attachments/860/planning/runtime-contract.md)。

## 2. 架构方案

原Host保留一个Play/Session/输入owner。新增GameInput快照和2D character profile接线；角色积分/接触/支撑/速度与状态在真实Auto执行，Rust只采样输入、传静态配置、受检收集/发布和管理系统/GPU资源。公开入口必须走SourceSnapshot准入与区域/版本身份，不能把860受信Session探针直接当用户接口发布。

只支持一个绑定的2D角色/Run、原128节点上限内的静态solid集合和固定AABB角色形状。角色和solid身份/维度/几何在Run初始化时固定；源/配置/资源变化必须Stop后新Run。solid与trigger/装饰显式区分，不用visible=false冒充销毁或碰撞层；更通用生命周期由后继合同负责。

## 3. 技术栈

沿用旧SDK/driver/AGP和现有依赖，保持SDK/锁/旧API字节。新增拟定`src/authoring/game_input.rs`、`character2d.rs`、`api/character2d_v1.at`、`tests/{game_input,character2d_api,character2d_host}.rs`、小型公共Auto示例及`tools/check_character2d.py`；实际局部命名由T-02核明。没有根crate或新物理库。

## 4. 需求分析与背景调查

用户2026-10-27要求规划下一阶段；当前仅new，不开始实施/新agent/远端发布，无额外时间/token预算。[858研究](../specs/auto-graphics/m2-game-sample-research.md)已固定G2-01..03与G3后继目标，2D先形成真实控制器符合[M2战略](../design/strategy/autographics-m2-v06.md)。

已读[AG概览](../specs/auto-graphics/overview.md)、[原行为](../specs/auto-graphics/behavior-api.md)、[原运动](../specs/auto-graphics/bounded-motion-profile.md)、[Play](../specs/auto-graphics/scene-play-session.md)与[研究矩阵](../design/autographics/m2-game-sample-matrix.md)。[源观察](attachments/860/planning/source-observation.json.txt)绑定856图形`4150a8bea`/SDK7b987ca8b；857未交付，860尚未执行。当前Input的x/y/toggle不表达可靠jump上升沿；旧v5只给位移和最后法线，不能证明最终支撑。新profile不能改变旧未来未知版本负例的语义来凑测试。

## 5. 详细设计

### 5.1 前置、入口与写面

前置860实际交付，decision明确go，包含可直接消费的2D机制/语义/资源/ABI测量；requires_replan或证据缺失阻止T-01后依赖实施。唯一树`D:/autostack/.wt/lang-861/auto-lang`/`plan-861-dev`从860交付图形提交创建，只合autographics-dev。当前无dependency-ready实施任务。

写面限driver内新游戏输入/2D角色接线、behavior来源/新profile选择、Host/Play必要适配、tests/tools/research/README与当前清单；旧API/算法/资产/两锁/SDK不改，AGP只读。角色通过原几何/位置实际呈现；当前不要求正式美术，但需声明仍是功能样板。

### 5.2 T-02先冻结新公开合同

产出`attachments/861/work/character-contract.md`：从860实际go补全显式profile版本（建议独立`autographics.character/1`命名域，不自动占用behavior/6）、callbacks/准入/来源区域、公开函数签名/错误、actor/solid配置、result/token生命周期、资源界/初始化/清理和当前入口，再做T-03。标识可等价调整，旧v1..v5接受/拒绝/生成物保持；不得让旧scene4严格解码静默接受新字段。

配置通过显式新profile初始化接口/声明传入，在app_start完成时冻结：仅现有2D叶节点可绑定角色，static solids来自明确持久ID到Run身份映射，不含角色自身/组/另一维度/装饰。至少一份不同solid布局对照，证明未按ID/名称硬编码；无配置或坏/重复/越Run身份受控拒绝。solid空间变换/形状在本profile只读，受管角色的位置只由控制器发布，其他setter绕过时明确拒绝；可见性与solid成员资格分开，不能隐藏后自动失去碰撞。不借新配置实现任意运行拓扑变更；若确需新增持久格式必须回new明确版本/迁移后再实现。

角色形状以858半宽0.4/半高0.5为固定初始支持；零/非有限/维度错/不合适scale或初始strict overlap明确拒绝，不修改旧profile对应行为。支持平移祖先、静态平移/AABB solid；不支持坡面、移动solid、旋转角色或动态scale。single-character限制属于新profile，不削减旧双actor实验。

原hard CPU/源码/节点限制保持。新profile状态/接触/输出逻辑与实际内存上限必须等860测量后写明，旧32credits和固定query存储不被无声复用/提高：若需要不同公开计费/状态合同，须在T-02完成语义revision和规范增量再实现。没有完整资源合同不得进入T-03；不能以“稍后测”绕过。

### 5.3 动作与固定步规则

初期公开动作MoveLeft/Right、Jump和Restart的held/pressed（Restart只给后继Auto游戏规则读取，不在本Plan实现胜负/重开游戏）。A/D或左右键，空格Jump，Enter Restart；相反移动同时按为0，OS repeat不产生pressed，快速down/up至少留下一个待消费edge，同步pressed每个实际dispatch tick最多消费一次。单独采样/多次渲染不重复消费，未dispatch不得丢边沿；输入队列/计数有860合同确定的有限界，溢出明确失败/清空政策而非无限堆积。

Starting/Pause/Stop/Fault/失焦/离窗/resize/DPI/模态/文本焦点/新Run清held/edge和旧Run身份；恢复需新按下，不把repeat复活。neutral Step的所有动作中立；导航MMB/scroll/文本与按钮不送游戏动作。单一输入router按profile只激活legacy或GameInput一种采样路径，同一KeyEvent不能被两个采样器消费。采样快照包含Run、逻辑tick与唯一消费身份；旧帧/跨Run数据在入口和使用处拒绝。

dt固定1/60，左右速度4、重力20、跳速8；角色状态/积分顺序消费860冻结机制。起跳只在动作edge且更新前已确认grounded时允许，空中无二段跳、不因按住在落地反复跳。无缓冲：空中该步pressed不能积存到未来落地；初始grounded由实际支撑/中立初始化确认，不能默认true。

先决定动作/起跳，再按冻结积分/碰撞更新，最后计算最终grounded/墙/顶与速度，整份状态原子发布。顶碰撞停止向上速度、落地停止向下速度；跨平台边缘最终无支撑不继续允许跳。正常接触/运动限制与API Fault、PrecisionStop/预算分别呈现，不能吞错。公开result快照需要区分同一步多读与过期token，其具体有界结构由T-02绑定860证据。

### 5.4 真实用户入口与生命周期

复用单文件app_start/app_update和原保留名称/常量/来源映射规则，扩展只认领明确新profile的公开函数。示例有同一角色、两种solid布局和用户速度4→5的真实原始源码改动对照；只重读/重编用户源码，不重建专用Rust业务。Run内配置/源码不热重载；Stop/新Run新状态/输入/句柄，Fault保最后完整帧与原编辑内容恢复。

至少真实OS窗口完成移动松开、跳一次/按住/空中再按、墙顶、边缘、Pause/neutral Step/失焦/Stop；数值、源身份/状态和最终合成像素共同验收。新工具约定`python tools/check_character2d.py <fresh-output> --executable <retained-exe> --controls`由T-04建立；真实重复edge/stale-grounded/漏画等变异须命中同门，恢复正例通过。初始化/回调/输出/预算错误均有可定位阶段，残余owner/迟到input不能留到新Run。

T-04/05新增plan861 current identity/verifier，更新source/consumer选择；旧manifest/金样不改。fresh own-SDK消费者实际运行新公开脚本；旧生成物/原分母与860同条件预算/资源对照保留。不把样板当G2金币/敌人/胜负、G3运动或正式游戏交付。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/game-actions-and-character2d.md | 无公开动作/角色合同→明确新profile、单角色/静态solid、输入边沿、2D角色/状态/资源/错误/生命周期 | 首个公共运行切片 | AC-02/03/04 |
| SD-02 | modify | docs/specs/auto-graphics/auto-source-driver.md | 当前来源/consumer新增角色profile，旧五版本身份/算法保持 | 正确公开消费 | AC-01/05 |
| SD-03 | modify | docs/specs/auto-graphics/scene-play-session.md | 旧Play保持，新增profile动作采样/中立Step/整帧角色发布/Stop新Run规则 | 共用一运行栈 | AC-03/04/05 |

## 6. 测试设计

实施自查→独立最终review→merge，唯一树/未分配新agent；完整门在最终review后执行。新测试/工具由对应任务建立，未假称现有或已运行。

| V | AC/不变量 | 实际/拟建入口与输入 | 可区别错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-02 配置/准入 | 新character2d_api，显式profile、合法/重复/组/错维/跨Run/初始重叠/未知版本 | 原五版本字节/拒绝不变；新配置失败在副作用前、合法不同布局均可消费；用户/受信区域诊断正确 | T-02/03；单2D角色静态图 |
| V-02 | AC-03 动作 | 新game_input，down/up/repeat/反向、未dispatch、重复render、跨Run、Pause/Step/失焦/text | 单步消费一次且不提前丢edge；中立/清空规则、旧身份拒绝；重复edge变异红 | T-03/04；单窗键盘，不含运行相机 |
| V-03 | AC-04 真角色 | 新character2d_host，858几何/参数、地墙顶/两接触/边缘/空中再跳、128wide/deep | 实际Auto位置/速度/最终grounded与860独立oracle一致，原hard limits；错误最后法线/stale支撑红 | T-03/04；无坡/移动体，非完整G2 |
| V-04 | AC-04/05 发布/退出 | 输入/配置不合法、输出部分缺失、预算、Starting cancel/Stop/再Run/20轮owner | Fault保last-good/旧编辑hash，真实owner最后释放；新Run不继承输入/角色状态 | T-03/04；原允许外部pin保留 |
| V-05 | AC-05 OS/用户/consumer | 新native窗口、两布局、源码4→5、partial/错runner/漏画像/恢复、fresh consumer | 真OS→winit→公开动作→Auto→GPU，源变化实际速度变化；数值AND内容红绿、全新消费者可运行 | T-04/05；图形样板不是正式美术/游戏 |

定向driver：`cargo check --locked`；新增`cargo test --locked --test game_input --test character2d_api --test character2d_host -- --test-threads=1`（T-03建立）；受改原authoring_behavior/Play命名case；新OS工具与860预算工具。完整两端/原新native-owner-content由merge执行，不运行根整档。T-02资源/公开签名未冻结前只做其规定的有界设计/核查，不进入T-03。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 860实际go/唯一树/SDK/锁/写面/输入证据核实，source/consumer完整 |
| AC-02 | work | 新公开合同/资源界先冻结；角色/solid准入与版本/诊断正确，旧五版本保持；V-01 |
| AC-03 | work | 有界动作held/pressed、dispatch一次消费、焦点/Run/Step清理与旧身份拒绝成立；V-02 |
| AC-04 | work | 真Auto有限2D控制器/最终地墙顶/跳跃、错误/整帧发布/预算/owner成立；V-03/04 |
| AC-05 | work | 原始用户源码/真实OS/两布局/consumer/预算与内容正反证据齐备，未认领完整游戏；V-05 |
| AC-06 | review | 独立核work AC/V/SD、860go参数、公开准入与实际数据流/warnings/fmt；merge门pending |
| AC-07 | merge | 冻结两端完整/原新native-owner-content及复用输入证明通过，失败不豁免 |
| AC-08 | merge | 源码只落autographics-dev，规范/ledger/归档/guard清理实际完成；3D/游戏后继仍pending |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 860交付且go | 唯一树/实际提交、决策/资源/样本/SDK/分母及工具核对，补真实实施入口 | AC-01 | [✅] |
| T-02 | work | T-01 | character-contract：公开profile/签名/角色solid配置/状态与token/资源上限/错误；必要时修订后再实施 | AC-02 | [✅] |
| T-03 | work | T-02 | 新动作/2D受信Auto/API/公开来源准入、原Play适配与定向tests，V-01..04 | AC-02/03/04 | [✅] |
| T-04 | work | T-03 | 新OS/GPU check/真实变异与恢复、Stop新Run/owner/current identity/consumer，V-04/05 | AC-01/05 | [✅] |
| T-05 | work | T-04 | 用户源码/两布局/128预算/资源对照与定向收口，提交代码/SD/证据，不跑整档 | AC-01..05 | [✅] |
| T-06 | review | T-05 | 独立最终review，核公开规则与实际Auto/OS/发布/状态；merge门pending | AC-06 | [✅] |
| T-07 | merge | T-06 pass | 冻结完整两端/原新native-owner-content和身份/复用审计 | AC-07 | [ ] |
| T-08 | merge | T-07 | 源码合autographics-dev，main沉淀SD/overview/plans/ADR/ledger/索引并归档 | AC-08 | [ ] |
| T-09 | merge | T-08 | guard clean清树/临时分支、原始产物恢复/实际9/9，交后继3D/生命周期设计 | AC-08 | [ ] |

## 9. 复审记录

2026-10-27 new r1：stage=new；outcome=blocked（860未执行/未交付go）；next=860交付后T-01核证据，T-02冻结实际公开/资源合同再进入T-03。所有AC/V/T/SD未执行，profile名为建议，具体签名不冒称已冻结。未改变857合同/源码、未创建实现树或执行Cargo/native/性能。本草案只覆盖2D，不能据此认领3D角色或M2完成。

2026-10-28 work T-01/T-02：用户授权实施861（860已归档交付go）。唯一树`D:/autostack/.wt/lang-861/auto-lang`、分支`plan-861-dev`自autographics-dev tip `7b4e89e6b`建立，`rev-parse --show-toplevel`/`--show-current`核过；主检出WIP仅docs/plan簿记面。**来源核验发现（已闭合，非阻塞）**：860计划记录的交付哈希（base `81b0974ac`/tip `7144526c3`）与本仓autographics-dev分支哈希不同（该线曾以等内容提交重写过一次，旧对象已不可达：`git cat-file 7b987ca8b`/`81b0974ac`均失败）；内容身份改以冻结收据闭环——`verify_plan860.py --controls` PASS（407文件与860冻结身份逐字节一致，含Cargo.lock=`2b131d3dd3`，绑定`base_commit 81b0974ac`+`sdk_commit 7b987ca8b`断言），零rebase声明与交付等价性由字节收据承担。SDK路线偏差（已闭合）：860的worktree清理使pinned SDK git对象不可达，`prepare_sdk.py`生成路径失效；改用860保留证据`D:/autostack/plan-artifacts/plan-860/consumer-package/.../.sdk-source`副本置入worktree后`prepare_sdk --verify-only` PASS（681文件/17 local，字节锁校验），SDK字节与860交付完全一致，记录于合同§10。decision=`research/game-runtime/decision.json` go（A=27cr推断组合、B=26cr融合支撑、tick CPU P50 ≤17.5ms vs 250ms界、工作集≤67MB、credits≤27/32、G2/G3路线+128负载全过）；`cargo check --locked` exit 0（RUSTC_WRAPPER= 空 + RUST_MIN_STACK=16777216，worktree根.cargo/config继承/STACK:33554432；直接设RUSTFLAGS会覆盖config并破坏sccache探测——实测留痕）。[character-contract r1](attachments/861/work/character-contract.md)已冻结：`autographics.character/1`独立命名域（behavior/6字符串与号均不占用，`_ag_version()=101`、profile 12）、装配=v5全链+`api/character2d_v1.at`追加、公开面`ag_char_bind/solid/move/grounded/support`+`ag_action_held/pressed`（动作0..3、OS键位A/D/←/→/Space/Enter、repeat无edge、焦点/暂停/故障清空、单一router）、公开采纳候选B（char_move=25cr=24+1，池32不变，A留作测试对照）、tick_begin只读强制（solid快照3×128 f32、外部移动fault 20/21）、character域fault 20..26、形状钉858半宽0.4/半高0.5、初始grounded由冻结中立下扫实测、.agscene格式零改动。executing 2/9；T-03未跑任何结果。

2026-10-28 work T-03：`stage: work | plan_id: PLAN-861 | plan_revision: 1 | outcome: pass(定向面) | worktree_path: D:/autostack/.wt/lang-861/auto-lang | branch: plan-861-dev | base_commit: 7b4e89e6b | code_commit: 4a520aa0c+01da3d7df | dependency_revisions: SDK 7b987ca8b（保留副本验证）不变、两锁不变 | task_ids: T-03 | evidence: 交付=(1)`api/character2d_v1.at`受信单元——公开`ag_char_bind/solid/move/grounded/support`+`ag_action_held/pressed`（held/pressed位打包2参注入、tick_begin只读强制、freeze中立下扫初始化、fault 20..26、`ag_char_move`=25cr）；(2)`Api::CharacterV1`+装配（v5全链字节保持，装配测试扩展7组件）；(3)Play接线（character构造器、`_ag_char_map`稳定ID映射注入、dispatch折叠`_ag_char_input`、`char_key`仅Running积累edge、`char_held_restore`无沿恢复、neutral Step）；(4)单一router（viewport键位映射，character Run下legacy采样器不见游戏键）+面板/命令选择器。测试=character2d_api 10/10（V-01）、game_input 6/6（V-02）、character2d_host 5/5（V-03/04）、game_runtime_feasibility 20/20（860语料保持）+authoring lib 11/11=52/52绿。实现期发现：.at词法`hr-hl`无空格并成单标识符；0基索引哨兵两处；VM参数上限8（输入位打包2参）；fault 13为终态（后续读0）；无头capture恒呈现编辑视图（像素级oracle归T-04窗口探针）。blocking: 无 | next_responsible_role: T-04。

2026-10-28 work T-04 进行中（部分交付，未完）：`stage: work | plan_id: PLAN-861 | code_commit: 8f008ecca+990d762cc+6c2bd03ec | task_ids: T-04 | evidence: (1)样例项目`samples/character2d/`（layout_a/b.agscene经scene/2生产导入路径验收——需显式parent+groups+固定资源清单；同源不同几何：B布局同输入坠坑y<-3、A贴地；main.at用户源持全部玩法参数、main-fast.at=speed5编辑态，tick60速度5.0 vs 4.0对照过）；(2)`tools/check_character2d.py`——正例3报告族（actions/host/layouts schema逐项+运动学+credits≤32+几何/速度oracle+checkpoint绑定）全过，--controls 6负控全拒（partial/zero-selected/oracle-edit/trace-divergence/wrong-exe/bound-runner-mismatch），报告经`runner_sha256`绑定retained runner；报告发射=测试内`CHARACTER2D_REPORT_DIR`门控（research/character2d-reports/）；(3)`tools/verify_plan861.py`+`research/plan861-identity.json` 282文件闭包（api/src/tests/samples/tools+包根、文本规范化、钉base 7b4e89e6b+SDK 7b987ca8b）--controls PASS（删除/未列/错哈希全拒后恢复）。**T-04余项**：OS窗口探针（V-05真实winit键盘→动作→Auto→GPU像素，经authoring App phase机+native_interaction::post模式）与fresh own-SDK consumer运行新公开脚本。T-05余项：预算采样矩阵（853同条件）与decision收口。merge-owned完整门pending不变。next_responsible_role: 继续 T-04 窗口探针。
2026-10-28 work T-04 第二批：`stage: work | code_commit: a073c058d（含identity重绑定） | task_ids: T-04 | evidence: (4)OS窗口探针交付——`authoring_probe/character2d.rs`相位机+bin接线（--character-check、--behavior-api character、finalize相位门10、报告字段），实测**passed=True**：真实OS窗口+真实posted winit键盘（D按住→x=1.333、Space点按→跳峰y=2.002≈0.5+8²/2g、blur自动暂停清held（resume后零漂移）、Stop恢复视图、resize/DPI合法初始化暂停经OS resume按钮恢复）；命令=`cargo run --bin autographics-authoring -- --character-check --open samples/character2d/layout_a.agscene --behavior <abs>/main.at --behavior-api character --report research/character2d/window-report.json`（报告本地保留，未入库=证据政策）；单一router真窗验证。identity重绑定283文件`verify_plan861 --controls` PASS；定向面复跑23/23绿。**T-04余项**：fresh own-SDK consumer运行新公开脚本；check工具window-report族扩展（收口项）。next_responsible_role: T-04 consumer → T-05 采样/decision收口。
2026-10-28 work T-04/T-05 收口：`stage: work | plan_id: PLAN-861 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-861/auto-lang | branch: plan-861-dev | base_commit: 7b4e89e6b | code_commit: 31f3986c6（4a520aa0c..31f3986c6 共10提交） | dependency_revisions: SDK 7b987ca8b 不变、两锁不变 | task_ids: T-04,T-05 | evidence: (T-04尾)`package_consumer.py`计划列表纳入861（身份436→439文件，含research/game-runtime与character2d、prototype子目录、baseline/sdk-Cargo.lock）；**fresh own-SDK consumer离线实跑**：包内`verify_plan861` PASS（439哈希）+`cargo test --offline` 新公开脚本实际编译运行全绿（character2d_api 10/10+samples 1/1，日志D:/autostack/.wt/lang-861/consumer-test.log本地保留）；`verify_source.py`加861现行代际分支（verify_plan861为current，860退位gated）；check_character2d扩至**五族报告**（actions/host/layouts/sampling/window）+每族runner绑定多exe，6篡改负控全拒；窗口报告入检（started/d-down/moved-right/space-tap/jumped/blur-paused/blur-cleared/stopped/completed九事件，位移/跳峰/清空断言）。(T-05)采样矩阵=5新鲜进程×370测步（各弃30暖步）：tick CPU **P50中位1.075ms/P95中位1.902ms** vs 250ms回调限（≥100×余量）、credits≤31/32（`research/character2d-reports/character-sampling-report.json`）；**decision交付**`research/character2d/decision.json`（delivered：公开面/计费/实测/边界/后继入口）；**SD-01草案**`docs/specs/auto-graphics/game-actions-and-character2d.md`（worktree准备，merge发布）；SD-02/03落点=worktree docs/specs/auto-graphics/{auto-source-driver,scene-play-session}.md由merge按实际增量更新。用户源码对照（speed 4→5）与两布局（A贴地/B坠坑）已由character2d_samples锁定；128预算对照见character2d_host。最终worktree面：identity 439 --controls PASS+定向面25/25绿+checker五族全过 | blockers: 无work内阻塞 | pending_closeout_gates（merge-owned，不因本记录视为完成）: 冻结两端独立workspace完整`cargo test --locked`、AGP受影响面与原新native-owner-content复用审计、`/auto-plan:regress`批量档到期判定、spec沉淀（SD-01发布+SD-02/03增量）与ledger/归档/guard清理 | next_responsible_role: 独立最终review（T-06）。R-1窗口探针期间曾误在主检出根Cargo.toml加feature（python cwd重置所致）——当场上报并`git checkout --`恢复，主检出零残留（git diff核过）。
2026-10-28 independent final R1：`stage: review | review_scope: final | reviewer_identity/limitations: 实施同会话/同模型（已声明），结论从冻结提交工件重建——复审员在31f3986c6独立重跑identity --controls（439 PASS）、860语料20/20、装配11/11、consumer重建+离线测试（包内verify 439 PASS+api 10/10+samples 1/1）、告警面（全部为SDK既有）| plan_id: PLAN-861 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-861/auto-lang | branch: plan-861-dev | reviewed_commit: 31f3986c6 | base_commit: 7b4e89e6b | dependency_revisions: SDK 7b987ca8b不变、两锁不变 | spec_inputs: SD-01草案docs/specs/auto-graphics/game-actions-and-character2d.md（与实现逐项核对一致）+SD-02/03目标文件存在 | acceptance_results: AC-01 PASS（verify_plan860前置+字节收据链）；AC-02 PASS（api 10/10+装配）；AC-03 PASS（game_input 6/6）；AC-04 PASS（host 8/8）；AC-05 **PARTIAL**（行为面全过：consumer离线10+1、两布局/速度/窗口探针/采样均绿——但见F-861-R1-01健康检查缺口） | findings: **F-861-R1-01（closeout缺陷，唯一发现）**：`cargo fmt --check`在861触碰文件共58处违例（cli.rs/assembly.rs/autographics-authoring.rs/character2d.rs/全部4个新测试文件——未触碰的860文件零违例=基线本净，全部为861引入）；违反AC-06健康检查（fmt clean）与860交付惯例（README测试清单含cargo fmt --check）；修复=cargo fmt+身份重绑+受影响面（编译/定向面/identity/consumer/窗口/采样报告）在新冻结提交重跑；关闭oracle=`cargo fmt --check`零diff+`verify_plan861 --controls` PASS+定向面绿 | evidence: 复审运行记录见会话；consumer-review-test.log本地保留 | next: work修复F-861-R1-01（T-05重开为载体）→修复复审（仅受影响面）→final pass
2026-10-28 independent final R2（修复复审+final pass）：`stage: review | review_scope: final | reviewer_identity/limitations: 实施同会话/同模型（已声明），结论从修复后冻结提交工件重建 | plan_id: PLAN-861 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-861/auto-lang | branch: plan-861-dev | reviewed_commit: 0f53f05da | base_commit: 7b4e89e6b | dependency_revisions: SDK 7b987ca8b不变、两锁不变 | repair_closure: F-861-R1-01 CLOSED——`cargo fmt --check`零diff（0b9e6e711）；修复范围=纯格式化+身份重绑+易变报告排除（0f53f05da，同关闭行：window-report.json按名排除于identity生成与验证器两侧），无语义变更 | acceptance_results: AC-01..05全PASS（修复轮受影响面重跑：定向面 api 10/game_input 8/host 8/samples 1/语料 20/lib 11全绿；identity 438 --controls PASS；五族checker+6负控全过；窗口探针重跑passed=True；采样/报告重生成；consumer在0f53f05da重建离线verify 438 PASS+api 10/10+samples 1/1=日志D:/autostack/.wt/lang-861/consumer-review2-test.log本地保留）；AC-06 PASS（本记录；warnings全为SDK既有、fmt净、无遗漏/降级/未批准deferral——两次复审唯一发现即F-861-R1-01已闭） | pending_closeout_gates（merge-owned T-07..09，不因本记录视为完成）: 冻结两端独立workspace完整`cargo test --locked`、AGP受影响面与原新native-owner-content复用审计、`/auto-plan:regress`批量档到期判定、spec沉淀（SD-01发布+SD-02/03增量）与ledger/归档/guard清理 | findings: 无新增 | evidence: 修复轮工件同会话重跑记录；冻结规范增量副本=worktree docs/specs/auto-graphics/game-actions-and-character2d.md@0f53f05da | next: merge（T-07..09）

## 10. 待澄清事项

硬前置860实际go，T-02依据其证据完成明确接口/资源设计。若需扩SDK/依赖、增加预算、改持久格式/旧profile、删除碰撞/输入/样本必需项或转移玩法到Rust，须先回new说明范围并由用户决定，不能靠Plan尚为草案绕过支持决策。
