---
plan_id: PLAN-849
status: reviewed
feature_name: AutoGraphics有限AABB扫掠与遇障停止
author: [Codex]
created_at: 2026-10-25
updated_at: 2026-10-26
plan_revision: 1
current_step: 9
total_steps: 10
depends_on: [PLAN-848]
supersedes_spec_components:
  - docs/specs/auto-graphics/segment-query-profile.md
  - docs/specs/auto-graphics/spatial-query-profile.md
  - docs/specs/auto-graphics/behavior-api.md
  - docs/specs/auto-graphics/behavior-project.md
  - docs/specs/auto-graphics/scene-play-session.md
  - docs/specs/auto-graphics/auto-source-driver.md
  - docs/specs/auto-graphics/prototype-profile.md
  - docs/specs/auto-graphics/project.md
  - docs/specs/auto-graphics/integration-contract.md
new_spec_components:
  - docs/specs/auto-graphics/swept-aabb-profile.md
touched_goals: [GOAL-007, GOAL-008]
affects: [auto-graphics, authoring-experimental, swept-aabb]
---

# [PLAN-849] AutoGraphics 有限 AABB 扫掠与遇障停止

## 0. 变更摘要

846解决当前位置的包含/重叠，848解决点沿有限线段的最近命中；849让**有体积的物体沿给定位移查询最早碰撞**，由真实Auto脚本决定移动到何处，实现高速越过薄障碍时仍能遇障停止。交付两个可编辑、绑定、保存、搬迁重开的2D/3D项目。

新增显式`autographics.behavior/4`及profile10，沿用受检AABB代理、Auto求交和原整帧准入。原v1/v2/v3源身份/保留名及既有语义不变，scene4只增加可接受的Binding.api。扫掠只查询，不移动节点；样例停止/留隙是用户Auto逻辑。滑动、重力、刚体和通用角色控制器留待后继。

本次只制定草案。核对期间848由executing1/10更新为r2 execution_done8/10，work报告完成，独立最终review和merge仍pending。849以848最终正式交付为前置，不以在途candidate充当实施基面。未实施、建树、提交、运行构建或测试。

## 1. 目标

1. 对最多128节点的当前场景，查询同维叶子的固定朝向/尺度AABB代理作纯平移时，与其他同维叶子的最早闭区间接触；返回节点、位移比例、移动节点world原点、进入面法线和初始重叠标志。
2. 查询/排序/结果生成由Auto运行；父组/叶子先写后查见最新world，查询无隐含节点写入，后续setter不改变结果快照。
3. 明确最初相交、起点接触、零位移、终点接触、最近tie、溢出/过期；与848混用不会误读旧结果。
4. 两个真实Auto项目区分离散终点检查/点线段与体积扫掠，证明高速薄墙、偏心GLB和实际Auto停止/反馈；无需修改Rust即可改变用户运动规则。
5. 保持CPU/时间/资源/源码闭包与版本兼容，通过正式UI、原生GPU、独立参考、owner及仓外新进程验证。

非目标：旋转扫掠、球/胶囊/凸包/triangle精确碰撞、动态目标相对运动CCD、重叠消解/逃离接触策略、自动move-and-slide/台阶/坡地/重力/刚体、层mask/事件总线、节点增删/缩放旋转写入、相机/屏幕射线、改原Breakout/Squash玩法、完整AutoUI/设计体系/统一Schema、ACE/增量/HMR、RQHost/生产迁移/Linux/发行。代理碰撞不升级为完整物理承诺。

## 2. 架构方案

```text
正式848 Document/资源/受检Descriptor + 显式behavior/4
 → 完整图/几何初始化 → Auto derive当前world
 → 移动叶子静态box offsets + 目标当前world boxes
 → Auto构形空间AABB + 有限slab/最早接触
 → 有界类型化结果快照 → 用户Auto停止/留隙/反馈
 → 原whole-frame验证 → 原Surface/Iced/GPU → Pause/Step/Stop
```

L1实验driver局部增量，只写`experimental/autographics-auto-driver/`，消费848固定SDK/AGP/Descriptor。根crates、SDK/auto-val、AGP、根manifest/三锁、生产UI/迁移树均只读；不新建native Rust碰撞后端、物理依赖、第二parser/通用ECS。需要改这些写面或支持合同先提出具体修订，重大扩面按L2处理。

| 门 | 实施前成立条件 / 当前观察 | owner_stage |
|---|---|---|
| G848 | 848全部work、有效独立最终review、冻结完整merge门、正式autographics-dev落地、Spec/ledger/归档/guard收据完成；目前pending | work T-01前 |
| GSOURCE | T-01冻结正式graphics全SHA、SDK/三锁/闭包及既有分母；candidate仅调查 | work T-01 |
| GSTART | 用户后续明确实施849；本次只授权规划 | work启动前 |

唯一未来树`D:/autostack/.wt/lang-849/auto-lang`、分支`plan-849-dev`，显式从848最终正式autographics-dev SHA建树，不用默认master基面。实施/internal/final同树顺序，主检出只记计划/知识；不创建或联系其他agent/chat。禁止junction/symlink，移除先wt-guard clean。图形源码只落autographics-dev，I2/I3迁移门保持。

## 3. 技术栈

复用Rust、stock Iced/wgpu、Auto Parser/Session、有限f32、SourceSnapshot/Api、spatial::Descriptor、Play配置/整帧检查和scene4 CodePayload/Storage。无需sqrt、方向归一化或新数学库。

调查固定对象为848 work完成候选`cef25b5c41dbbfbec6b7bfd4b13507d8d5f1e34d`（初读d3032163e，后续SDK存储计量增量已核）；当前正式graphics仍`bf22db3465203f3babd649494f4b37aa2a012027`，观察SDK`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`。它们是调查来源，实施冻结由T-01重核。848尚未沉淀的segment Spec列为待交付modify目标，而非假定已存在。

## 4. 需求分析与背景调查

### 4.1 授权与来源

用户2026-10-25要求“848已经快实施完了，请规划本路线的下一个计划”，授权调查与落一份后继合同；没有本轮实施/合并、新建chat或SDK扩面授权。按滚动规划，只取849一个编号。

已读[全局overview](../specs/overview.md)、[图形overview](../specs/auto-graphics/overview.md)、[空间查询](../specs/auto-graphics/spatial-query-profile.md)、[行为API](../specs/auto-graphics/behavior-api.md)、[行为项目](../specs/auto-graphics/behavior-project.md)、[Play](../specs/auto-graphics/scene-play-session.md)、[资产](../specs/auto-graphics/scene-assets.md)、[集成边界](../specs/auto-graphics/integration-contract.md)、[848合同](848-autographics-segment-casts-and-nearest-hit-interactions.md)及其候选feasibility/handoff。固定Git对象及文档hash见[来源清单](attachments/849/planning/source-observation.json.txt)。

设计沿[Design45](../design/45-autographics-wgpu-engine-and-game-prototypes.md)既有有限游戏/Auto能力路线；完整AutoUI/默认设计、Schema/841/ACE/HMR按后继分期，849不依赖未交付生产端口。847是另一条可选测试feature修复，不是本路线功能前置；批量回归状态由收尾按实际收据判断。

### 4.2 真实路径与缺口

| 观察触点（固定848 candidate） | 已知事实 | 849缺口/消费点 |
|---|---|---|
| src/authoring/spatial.rs::Descriptor | 全mesh、静态Ry*Rx*S八角offset、资源/base身份，Run内复用 | 同一移动叶子offset与目标box，不取scale/2伪造偏心GLB |
| api/behavior_segment_v3.at | Auto维度展开slab、ordinal最近、几何先验、单槽token、cast8/getter1 | 新体积扫掠；结果原点与点命中含义分清 |
| behavior.rs::segment_transport/from_bytes_for | v3局部derive特化/clean末尾、AST准入、旧v1/v2字节隔离 | v4等价复用；新名只对v4保留，v3也冻结 |
| Play::new_user_spatial/new_user_with_spatial | Descriptor/完整源身份验证后初始化，header/每节点完整消费 | 显式profile10全路径，不只改返回数字 |
| behavior_project.rs、Host/Panel/bin | scene4版本化绑定、raw共享/凭证隔离、UI/CLI、来源/历史/事务 | /4绑定与临时选择、坏选项、失败保旧 |
| segment_feasibility/geometry/host/owners | r1两方案触100000步，r2宽/深128控制通过；候选另修t下溢/末尾world测试 | 849先验证预算与邻接数值/dirty路径，不假定追加代码自然可承受 |

v3 API原始文本约10.9KiB，v1/v2约2.6/3.3KiB；不等于最终managed大小或余量。T-02测真实v4组装/命名空间/编译/Host，原64KiB不变。候选绿测试不替代848正式验收，也不是849测试结果。

## 5. 详细设计

### 5.1 第一实施入口：有界整路径可行性

T-01关闭G848后，T-02先在实际v4 SourceSnapshot→Parser/Session→完整spatial配置→Auto callback→Play完整frame建立探针。UI/项目/GPU由T-06/07完成，不形成前后依赖循环。正式核与探针共用，不能native代算过探针再换Auto。

T-02负责探针所需最小v4枚举/managed组装/typed构造与profile消费接线以及候选求交核；这些是同树的正式实现起点。T-03在该基础上补齐全部准入、身份、旧版本兼容及项目事务，T-04收口完整几何/数值语义。T-02不得要求尚未完成的T-03入口，也不把修改现有v3源码后的旁路Session称为正式v4验证。

最多比较两种局部组织：A复用848展开slab并传移动offset；B在v4局部展开/提升不变量、沿848先验几何校验和合法最近剪枝降成本。T-02记录选型、managed长度、yield/CPU/owner与反控；不复制图/Descriptor、不跨callback缓存world。等价局部机制可选，不预定字符串替换或cache实现。

必过控制：2D/3D各128总节点，宽树多个parent切换、深树合法深度，至少两个group setter（query前/后分别覆盖），一次sweep16credits+八次sweep getter+八次原AABB尝试=32；原v3宽/深负载不退化。小场景另过cast8+两次cast getter+sweep16+六次sweep getter=32混用。正常100000累计steps/120resumes/250ms callback、30s初始化/2s tick/8ms pump保持；yield不冒称精确steps。末尾world直接观察，不能用触发lazy derive的事后getter修好未发布world。

若两候选仍失败或managed超64KiB，留反例并executing/needs_replan，先诊断同一计划；不减128/提CPU/借SDK加native。通过后固化永久敏感性断言，T-03..08补全正式入口/数值/生命周期/GPU。

### 5.2 公共入口、候选与不写入保证

拟`ag_sweep_aabb(node int, visible_only int, dx float, dy float, dz float) int`。node须合法可查询叶子（0合法，−1/group/越界错误），维度从node读，2D dz精确0；visible_only精确0/1，位移全部finite。D是world位移，不是local或速度。隐藏移动节点仍可查询，visible_only只过滤目标。

目标为同维其他叶子，自动排除自身/group/其他维度；visible_only=1用effective可见性，alpha独立；无额外ignore/subtree/mask。目标固定在查询瞬间，之前setter已生效，不预测目标下一运动。相同最近t取最小当次Run ordinal。

移动节点与全部匹配目标的当前world box须可用/finite，坏匹配几何即GeometryUnavailable，近命中/零位移不得遮蔽；过滤者不参加目标几何校验。无目标合法miss。查询不改local/visibility；允许按既有dirty合同派生更新world数组，但不制造额外移动或错误清dirty。后续移动由用户setter决定。错误返回placeholder0不是正常token。

### 5.3 几何与数值合同

移动节点当前world原点O，静态offset `[mlo,mhi]`，目标当前world box `[tlo,thi]`。逐轴构形空间区间`L=tlo-mhi`、`H=thi-mlo`，在`O+t*D, t∈[0,1]`上做闭slab。必须取实际offset两端，不能假定box中心在节点原点或由scale推半径。静态旋转后仍为原world轴AABB代理；仅平移，非OBB精确连续碰撞。

f32运算次序：先按846求目标world两端，再按上述表达式减offset；每轴用`(L-O)/D`、`(H-O)/D`，D=±0只比较不除零。初始重叠按O是否在构形空间闭区间内计算；正常有限实数下等价于两AABB相交，但极端舍入不额外承诺与846另一运算次序的overlaps逐位相同，原846语义不改。T-02/04独立f32参考按公开次序核验，单列±1ULP，不用测试反向定义合同。

- 起点包含/边界接触均为hit、t0、start_overlap1、normal全0；即使位移离开也报告初始接触。不解穿插/判逃离方向。零位移按初始相交规则，其他miss。
- 非初始相交取最早f32进入t，t1终点接触计hit，无epsilon扩张。进入面normal为轴±1，同t多轴按X/Y/Z。正进入值下溢为0时，outside仍返回进入normal与start_overlap0，继承848边界经验。
- 最近按实际f32 t再ordinal；法线非mesh接触法线。返回origin=`O+t*D`，是移动节点world原点，非表面接触点/box中心/local。miss为t1/origin=O+D/normal0/node−1/start_overlap0。
- 输入finite不保证中间finite：移动起点box/目标world box非finite报GeometryUnavailable；构形区间、O+D、实际执行的差/商/结果非finite报ArithmeticNonFinite。按848政策先校验每个匹配目标world box，再合法miss/nearest剪枝；未执行的后续商不造错，坏world几何不能被遮蔽。不能让Inf进入排序成为正常miss。
- hit/节点/状态、精确t排序、normal轴符号用精确oracle；浮点观测沿848 fraction `max(1e-6,8ε)`、world `max(1e-3,8ε*max(1,|expected|,|actual|))`，只作观测容差，不作用于boolean/tie。独立手算与f32参考并列。

### 5.4 结果、版本与预算

拟getter：`ag_sweep_hit(token) int`、`ag_sweep_node(token) int`、`ag_sweep_t(token) float`、`ag_sweep_origin(token,axis) float`、`ag_sweep_normal(token,axis) float`、`ag_sweep_start_overlap(token) int`。axis0/1/2，2D z输出0。hit/miss均正token、查询瞬时快照；后续setter不改它。

v4的cast/sweep共享一个最新结果身份/单槽政策：每次尝试先失效前槽，包括失败；成功用同一Run内单调正i32序列，不能各自从1计数造成ABA。结果带kind，cast getter只读cast，sweep getter只读sweep；错kind/旧/无token、跨callback均具名InvalidQueryResult及sticky Fault。callback begin/end和Stop失效，耗尽先Fault不wrap。原AABB不覆盖结果。token不是持久ID/安全能力；v3版本原cast合同保持。

实现可复用固定字段，但v4总常驻query结果/计数/kind/序列≤32个32-bit标量逻辑内容（128B，Auto容器/VM固定开销另记）；禁止随query增长的list/整图副本/私有cache。callback/Run间无有效结果。

沿848实测方式分开报告NanoValue、global键String结构/容量与共享map边界；128B只限逻辑标量，不伪称实际SDK总占用。重复查询须同时核heap对象不随次数增长与真实最后owner释放，固定对象数不能单独证明字节/owner合同。

v4仍每callback32query credits：sweep尝试16、cast尝试8、所有结果getter/原AABB尝试1，内部helper不重复收费，首个超额在几何/结果访问前QueryBudget。§5.1为128支持控制，不承诺任意32组合/用户代码均在CPU上限内。v1/v2/v3原收费、managed/ENTRY/合法私有helper集合不变。

任何错误保持sticky第一错/code-node-argument，拒绝整callback、关闭VM、保last-good。完整配置才app_start；Starting取消、Pause/neutral Step/Resume、Stop恢复接受编辑内容、新Run重建保持。actual native math NaN/±Inf经参数/local/return/container的合法路径均受检，源码解析失败不算runtime控制。

### 5.5 正式入口、项目与资源

显式Api SweepV4/profile10贯通SourceSnapshot::read_for/from_bytes_for、managed region/保留名、Play typed构造/全身份/header/consumer、Host/Panel及`--behavior-api v4`。完整source/API/bridge/combined/base/Descriptor身份不减，不猜版本/回退。旧v1..v3原managed字节/API/bridge/ENTRY hash冻结；旧脚本合法自定义ag_sweep_*，只在v4重定义拒绝。

默认临时脚本仍原v1政策，scene4默认明确Binding.api；字段结构不变，精确新增/4，旧reader未知/4明确unsupported。缺值/孤立/未知CLI在普通运行与--inspect均exit2；合法缺省/明确v1..v4保留。

same-raw跨版本可共用Arc/blob，准入/编译凭证含版本/managed身份。切版本为内容事务；dirty/Undo/Redo/失败保旧、Save/SaveAs/Open/Verify/新root/旧epoch、memory或确认hash来源沿845/848合同。只存接受源码/绑定，不存runtime结果。源256KiB/managed64KiB/combined320KiB、8raw/2MiB及诊断/report上限保持，无新Schema/模块/ACE/HMR。

Descriptor继续借Host实际builtin/Registry ModelData，全mesh bounds/静态offset一次；查询读取actual Auto world，无逐tick顶点重扫/上传/读源/readback/targets/pipelines。20轮混合版本绑定/Undo/Open/Run/Fault/Stop核source/VM/Descriptor最后owner；合法Editor/历史/外部pin单列，warm counter AND content保持。报告query/compile/full tick CPU与工作集，GPU timestamp/驱动驻留/Iced内部上传仍排除，不把60Hz逻辑dt当性能保证。

### 5.6 两个用户项目与后继

新增拟`samples/sweep_projects/dash2d/`：Quad/Circle代理角色高速穿薄墙，结束位置与墙不相交但路径相交；Auto先sweep再按结果/私有clearance规则移动、显示阻挡。独立例：actor x-offset[-1,1]、墙x[4,4.2]、O.x0/D.x10，则sweep t=.3、原点cast t=.4、终点离散pair miss。Clearance是样例规则，不改query闭区间、不替引擎偷偷加skin。

新增拟`samples/sweep_projects/dash3d/`：实际多part偏心GLB作移动叶子，非零parent和静态yaw/pitch/scale；全mesh offset决定提前停止，至少两障碍显示最早命中。包含同callback父写→query→子写/旧快照不变→下callback新query，核直接发布world。独立参考由冻结实际资产计算，不从被测输出生成expected，不能仅2D换维度常量。

两样例在有限受验坐标/位移上采用用户clearance，小于所测间距；每次命中保留可核间隙，正常反向输入可离开。起始已重叠/接触给可见状态、t0并不动，不承诺解穿插；零位移/空候选独立验收。只改用户速度/停止条件后Update绑定、Save/reopen改变行为，Rust/API/bridge未重编。Pause不重复动作、Step一次、Resume不重置app_start；Stop恢复编辑内容。

正式UI选v4/绑定/SaveAs后复制仓外不同cwd，原root/源/导入path不可用，同一runner字节SHA在新进程--open再明确Run。断言最早node/t/normal/origin/停止位置、反馈和全mesh最终像素；partial/透明/漏part/错资产对应失败。原双游戏/Move-Patrol/Zone2D-3D/Beam2D-Sight3D分母保持。

849交付后形成“位置感知→方向感知→有体积路径感知+脚本停止”的有限碰撞链。下一未编号候选才是有界多接触/滑动策略，需根据预算/初始接触研究后另立合同；不宣称完整角色控制器或引擎。渲染/动画/动态场景、完整UI/Schema、ACE和生产集成仍是独立待办。

### 规范增量

全部拟议，merge前不改canonical。segment-query-profile先等848正式沉淀，再修改v4混用边界，v3语义不改。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/swept-aabb-profile.md | 无volume query→固定代理平移/构形f32/初始接触/origin/16credits/脚本停止边界 | 有体积高速运动 | AC-02, AC-04, AC-05, AC-08 |
| SD-02 | modify | docs/specs/auto-graphics/segment-query-profile.md; docs/specs/auto-graphics/spatial-query-profile.md; docs/specs/auto-graphics/behavior-api.md; docs/specs/auto-graphics/behavior-project.md | 旧版本保持→v4/10、类型化共享token及scene4版本闭包 | 兼容/不串结果 | AC-03, AC-04, AC-06 |
| SD-03 | modify | docs/specs/auto-graphics/scene-play-session.md | 原生命周期→v4配置/错误/失效/dirty发布；原CPU/Stop不变 | 运行时安全 | AC-01, AC-04, AC-05, AC-07 |
| SD-04 | modify | docs/specs/auto-graphics/auto-source-driver.md; docs/specs/auto-graphics/prototype-profile.md; docs/specs/auto-graphics/project.md; docs/specs/auto-graphics/integration-contract.md | 查询样例→脚本停止项目及搬迁/原生/owner证据，I2/I3仍pending | 交付边界准确 | AC-06, AC-07, AC-08, AC-10 |

## 6. 测试设计

本轮只静态调查，不运行Cargo/docs_gen/prepare/build/原生/GPU；下列拟议fixture由T-02..08建立，未运行不记pass。用户未指定其他实施/reviewer模型/chat；work/internal/final同树顺序，最终review独立上下文并暂停实现写入，自查不能充当独立最终pass。

未来849树driver目录沿`python tools/prepare_sdk.py --repository D:/autostack/auto-lang --verify-only`、`python tools/verify_source.py`及最终848来源门核闭包；新849 manifest/verifier在T-08建立，不能重生基准后冒称验证。开发用`cargo check --locked --all-targets`、`cargo fmt --all -- --check`，拟`cargo test --locked --test sweep_<对应target> -- --nocapture --test-threads=1`及受影响旧target。0selected/skip/工具或GPU缺失不是通过。

| case_id | AC / work owner | 真实入口/输入（new由该任务建立） | 可区分oracle / 反例 | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01；T-01/08 | 正式848/SDK/三锁/source、原producer/consumer身份门 | 正式交付/缺源/错pin/资源漂移拒绝，原分母不缩 | candidate非交付收据 |
| V-02 | AC-02；T-04 | new sweep_geometry/actual Auto：厚度、偏心多mesh/非零parent/静态旋转 | sweep .3对cast .4、终点pair miss；正确origin非接触点，hand/f32参考 | 固定world轴代理平移 |
| V-03 | AC-02；T-04 | ±0、零位移、inside/boundary且离开、t1、退化box、XYZ tie、ULP/下溢t0 | hit/normal/start_overlap按§5.3；epsilon/错inside/tie/忽略厚度变异失败 | 无解穿插/OBB精确 |
| V-04 | AC-02, AC-05；T-04/05 | nearest/ordinal、actor0/坏group/异维、visible/alpha0、后置坏box/过滤，native非finite流转 | 忽略自身/先验几何/具名第一错，近命中不能遮坏几何，合法排除成功 | 目标瞬时静止 |
| V-05 | AC-03；T-03 | SourceSnapshot/Parser/Play v1..v4、旧合法ag_sweep helper、伪source/base/profile | 旧managed/hash/reserved保持，新名只v4保留；身份错误不dispatch | 无auto-upgrade/fallback |
| V-06 | AC-04；T-04/05 | sweep→setter旧读/第二次；cast↔sweep、错kind/token、callback/Run、i32末值 | 快照稳、共享序列不ABA，坏读Fault，AABB不覆盖，混用成功 | 一个最新槽/非持久 |
| V-07 | AC-05；T-02/05 | 正式v4 Host/Session128宽/深多parent双写、16+8+8=32；小场景混用/越额/原v3控制 | CPU内控制过、超额先拒、managed≤64KiB、直接检查world非lazy repair | 非任意脚本性能保证 |
| V-08 | AC-04, AC-05；T-05 | 错序/漏配置、Starting取消/Stop、错误后setter、Pause/Step/Resume/Fault/修源 | 配置后才callback，first-error/whole-frame/last-good/VMclose/Stop重跑 | 无热更新/VMrollback |
| V-09 | AC-06；T-03/06 | UI/CLI v4、same-raw事务/Undo、scene4 Save/Verify/Open/SaveAs/新root/失败/旧epoch | 凭证隔离/raw共享，切版本非noop、失败保旧；缺值/孤立/未知exit2，合法默认成功 | 不存runtime |
| V-10 | AC-06, AC-08；T-06/07 | 两Auto项目、OS输入/绑定/SaveAs/搬迁cwd/new process/全mesh像素 | 停止位置/node/t/normal/origin独立参考；改源改行为无Rust重编；透明/漏part/错asset/partial非零 | counter不能代内容 |
| V-11 | AC-07；T-08 | 20轮版本/Undo/Open/Run/Fault/Stop、合法pin、累积/重扫/上传变异 | source/VM/Descriptor实际最后owner和GPU守恒，固定槽/warm无重扫重传 | 任意heap/驱动/Iced内部排除 |
| V-12 | AC-08；T-08 | API/预算/数值/初始接触/clearance/闭包/SD01..04/merge矩阵 | 源码/CLI/文档与新旧分母一致，反控命中目标oracle，raw可重跑 | 静态检查非运行验收 |

每case记录命令/退出码/断言数、code/SDK/三锁/用户源/managed/fixture/几何算法/runner SHA与raw/像素。独立参考不调用被测核/不以被测t生成expected。反控须真实改目标机制并有恢复正控；setup早退/缺工具/错误阶段不算敏感性证据。

T-08给准确merge矩阵和Spec proposal；T-09独立final pass后，T-10在冻结提交统一跑完整locked driver/AGP test/build、原双游戏/合成counter AND content/835..848原生/owner/source/package及fresh own-SDK/own-target consumer，加新扫掠正反/搬迁/预算。候选848最新readiness预计driver208、AGP163/16 native、原87 graphics调用，仅定位线索；T-01/08按正式清单/命令/结果重核，不能以旧数量当通过。复用先核代码/依赖/配置/输入未变；改代码返回受影响review后重冻。

849 driver-only不触root t/tv/tt/tb/tu/taa/docs_gen，不能替上游还门禁。tf主检出单实例到期批量，merge记录实际判定/847关联，不冒称全局绿。新warnings清零，固定SDK预存warnings按正式848清单单列，不扩写SDK消警。

## 7. 验收标准

| ID | owner_stage | 可观察标准 |
|---|---|---|
| AC-01 | work | G848/GSOURCE、正式基面/唯一树/只读边界和原分母冻结，V-01 |
| AC-02 | work | actual Auto扫掠/构形f32/初始接触/最早tie/过滤/数值符合独立参考，V-02/03/04 |
| AC-03 | work | v4/profile10全身份/准入和旧v1..v3字节/语义/合法helper兼容，V-05 |
| AC-04 | work | 查询不写、最新world/瞬时origin快照、类型化单槽token和原生命周期成立，V-06/08 |
| AC-05 | work | 128宽/深/32credits、managed/原CPU门、非finite/配置/sticky/整帧Fault闭合，V-04/07/08 |
| AC-06 | work | 正式UI/CLI、版本绑定/事务/来源/闭包与两Auto停止项目，V-09/10 |
| AC-07 | work | 固定槽、20轮owner/pin、warm无重扫/上传/读源，原GPU计量/内容保持，V-11 |
| AC-08 | work | 同binary仓外new-process/OS/GPU正反无必需gap，API/SD/数值/精确merge矩阵齐，V-10/12 |
| AC-09 | review | 独立核AC-01..08/SD/遗漏/延后/workaround/健康，revision+code绑定pass，完整merge门明确pending |
| AC-10 | merge | 冻结完整门过后只落autographics-dev，Spec/ledger/归档/guard/到期收据完备，master仅知识 |

## 8. 执行步骤

当前reviewed9/10，G848/GSTART/GSOURCE关闭，T-01..08完成并提交6bb85f354；T-09独立最终复审pass，完整merge门pending。T-10真实收尾才10/10；唯一849树保持冻结，尚未合入。

| task_id | owner_stage | 依赖 | 实际触点/拟新增路径与动作 | 关闭条件；AC |
|---|---|---|---|---|
| T-01 [x] | work | G848/GSTART | 正式848收据/Specs、唯一849树、SDK/三锁/source闭包/分母 | 实际path/branch/HEAD/正式源及未闭债务边界冻结；AC-01 |
| T-02 [x] | work | T-01 | behavior.rs/Play/Descriptor；new research/swept-aabb-feasibility.md、tests/sweep_feasibility.rs | §5.1最多两候选，真实构造/Auto/Host128控制及budget/源大小/dirty/反控；AC-02, AC-04, AC-05 |
| T-03 [x] | work | T-02 | behavior.rs/behavior_project.rs；new tests/sweep_api.rs、sweep_project.rs | v4/10 typed source/版本/准入/scene4隔离，旧身份/helper，V-05/09；AC-03, AC-06 |
| T-04 [x] | work | T-03 | new api/behavior_sweep_v4.at、tests/sweep_geometry.rs；同Descriptor只读 | actual Auto几何/过滤/初始接触/normal/origin/token、独立参考及反控，V-02/03/04/06；AC-02, AC-04 |
| T-05 [x] | work | T-04 | v4组合/Play/callback配置/consumer；new tests/sweep_host.rs | V-04/06/07/08、预算/末尾world/whole-frame/Starting/Stop/Fault闭合；AC-04, AC-05 |
| T-06 [x] | work | T-05 | Host/Panel/bin、new tests/sweep_cli.rs、samples/sweep_projects/dash2d和dash3d | UI/CLI/绑定事务、两Auto业务/有限clearance/初始接触反馈，V-09/10；AC-06 |
| T-07 [x] | work | T-06 | new tools/check_swept_aabb.py、OS/GPU/仓外reopen | V-10同binary两项目/全mesh/改源/停止位置/正反/partial无gap；AC-08 |
| T-08 [x] | work | T-07 | new tests/sweep_owners.rs、tools/check_sweep_owners.py、849 source/verifier、research/swept-aabb-handoff.md/Spec proposal | V-01/11/12、20轮owner/旧受影响定向/check/fmt/精确merge矩阵；AC-01, AC-07, AC-08 |
| T-09 [x] | review | T-08 | 同树冻结code/Plan/Spec/raw；主检出独立review附件 | 逐项AC-01..08/SD/遗漏/延后/workaround/健康和revision+code结论；AC-09 |
| T-10 | merge | T-09 pass | 冻结完整门→autographics-dev；主检出Spec/ledger/archive/wt-guard/到期 | AC-10真实全部收尾，改代码先受影响复审，不提前标交付 |

## 9. 复审记录

2026-10-25：stage:new；PLAN-849 revision1；outcome:blocked（G848正式交付pending，草案可评审）；next:848完成独立验收/冻结合入和知识收尾，再按后续work指令由849 T-01冻结正式基面、T-02验证有界可行性。changed:T-01..10、AC-01..10、V-01..12、SD-01..04；drafting0/10。

规划静态检查见[preflight](attachments/849/planning/preflight.md)。本轮无代码/依赖/Spec/ledger修改、worktree创建、运行测试/构建或agent/chat分派；未改848合同/进度。候选与簿记不同步如实记录，不替848补勾选。新接口/fixture/期望为未来合同，静态pass不表示可越过G848实施。

### 2026-10-26 work T-01关闭 / T-02执行入口

T-01正式基面dd7076a7f40b35ffda6b876c36f826af3443fb97，树D:/autostack/.wt/lang-849/auto-lang，branch plan-849-dev；848最终archive/guard/交付收据及canonical f126a98d7、ledger dbeb8d463/v87已核。SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、三锁/source-lock SHA冻结在实施树research/sweep-baseline.json.txt。prepare_sdk681文件/17包、verify_source374源/资产及479包/29路径边/forbidden0通过。旧87graphics语义分母、216driver与AGP163/16native保留；全档仍merge-owned。

T-02风险入口见实施树research/swept-aabb-feasibility.md：SweepV4/profile10正式SourceSnapshot→Parser/Session→完整配置→Auto求交→Play整帧，不用旁路v3冒充。候选A复用展开slab并传offset；必要时B提升不变量，最多两候选，预算/128/写面不扩张。厚度独立例及128宽深2D/3D/32credit/direct-world/Host与实际变异为关闭oracle；尚未执行通过不记完成。代码/测试只在唯一实施树；主检出既有WIP保留。当前1/10，next work T-02。

### 2026-10-26 work final — pass / execution_done8/10

stage:work | plan_id:PLAN-849 | plan_revision:1 | outcome:pass | worktree_path:D:/autostack/.wt/lang-849/auto-lang | branch:plan-849-dev | base_commit:dd7076a7f40b35ffda6b876c36f826af3443fb97 | code_commit:6bb85f354e718f6541256f4b3d75ca97af75b8c4 | dependency_revisions:SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、原三锁/AGP未变 | task_ids:T-01..08 closed,T-09/T-10 pending | evidence:attachments/849/work-final/readiness.md、identity.json.txt、raw/；实施树research/swept-aabb-handoff.md / spec-delta / merge-matrix | blockers:none in work | next_responsible_role:independent review。

完成显式v4/profile10、actual Auto固定AABB扫掠、类型化共享最新结果及预算、全身份/scene4/UI/CLI、Dash2D薄墙与Dash3D偏心多part GLB用户停止样例。候选A通过，不启用候选B；128节点宽/深和多parent切换、原100000steps/120resumes/250ms等上限保持，最终观察23..84yield非精确步数。厚度独立例sweep .3 / cast .4 / endpoint pair miss、f32/±0/ULP/下溢/初始接触/nearest/filter/native非finite/快照/dirty/整帧/Starting/Stop均有永久定向断言和变异/前红证据。

31新增Rust测试均有实际定向passing证据（30integration+1内联存储单测，跨日志滤串合计，不计0selected）。受影响旧segment API8/derive3/project1、CLI及Beam2D/Sight3D原生正控保留。最终原生14主例+4次更新/SaveAs后新进程重开，在仓外不同cwd同一runner2591a08ef9bb16f811dc13f78c7021f24acff8714f54b70182b46ce895ecaf61；直接核持有输入后的node/t/normal/origin、实际停止位置、用户源改变后的间距、全mesh最终像素及partial/透明/漏part/错资产/源码闭包反控。20轮混合版本owner+warm counter AND两画布内容，源码/VM/Descriptor/重扫/重传/内容六反控在各自oracle失败、恢复正控成功；真实最后owner释放和合法pin边界明确。18标量72B逻辑，SDK值/键/共享map观测分开，不伪称总SDK128B。

work发现F849-local-nonfinite：local NaN初报Arithmetic12，永久反例要求Geometry9；现先校验未强制类型转换的world坐标，移动者和后置匹配目标均9，最终geometry7/feasibility5/Host4及最终完整新原生矩阵重验通过。反控首次保留全部源码触8source预算、content检查首次遮蔽VM oracle均已调整为精确作用域，历史错阶段失败留档不计绿。无SDK/AGP/根crate/锁修改，无未闭work finding/必需coverage gap/合同决策。

402源/资产、681SDK/17local/29edges/479metadata闭包及四漂移反控通过，849新manifest与848旧收据分开。已导出仓外独立SDK/无构建target的source-consumer，其同样source/SDK/metadata/四反控通过；完整own-target消费者编译/测试仍merge-owned。最终cargo check --locked --all-targets、cargo fmt -p autographics-auto-driver -- --check和diff check通过；fmt --all遇固定SDK嵌套workspace边界，按848既有package范围等价覆盖可写Rust，未改SDK。固定146SDK预存warnings，无新增driver warning。

实现已提交且唯一树clean，写权限交给独立review，当前会话不自称最终独立pass，不新建/联系agent或chat。T-09独立AC/SD复审、T-10冻结完整门/合入/规范沉淀/归档/guard均pending。精确merge矩阵77原有当前命令组+4新双端命令组，原87graphics语义槽及23未变AGP收据槽（163tests/16native）不缩；全locked driver/AGP必要面/原生/owner/fresh consumer归merge。未跑root t/tv/tt/tb/tu/taa/docs_gen/tf；主检出批量due与B845-FEATURE-01/847边界保留，无合入master代码/发布规范ledger/清理或交付宣称。

### 2026-10-26 independent final review R1 — pass / reviewed9/10

stage:review | review_scope:final | reviewer_identity:/root/review_849_final_r1（独立子上下文，未参与实施；模型身份未暴露） | plan_id:PLAN-849 | plan_revision:1 | outcome:pass | worktree_path:D:/autostack/.wt/lang-849/auto-lang | branch:plan-849-dev | reviewed_commit:6bb85f354e718f6541256f4b3d75ca97af75b8c4 | base_commit:dd7076a7f40b35ffda6b876c36f826af3443fb97 | implementation_clean:true（入口/结论均核） | dependency_revisions:SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、三锁/source-lock匹配冻结值、AGP未变 | spec_inputs:canonical逐文件SHA见[baseline](attachments/849/review-r1-final/baseline.json.txt) | spec_delta_sha256:5c9844c4674579115e39807544cec6de5ba83d0b3eca9a89dc230911d2d03099（[冻结副本](attachments/849/review-r1-final/spec-delta-frozen.md)） | acceptance_results:AC-01..08 pass、AC-09 pass；AC-10 pending merge | findings:none | next:merge-owned冻结完整门，暂不合并。

完整逐项AC→T→路径/断言/证据及SD-01..04、遗漏/延后/workaround/健康审查见[独立review](attachments/849/review-r1-final/review.md)，机器收据见[receipt](attachments/849/review-r1-final/receipt.json.txt)。新执行28项定向测试/36组方向偏心混轴holdout/681SDK+402source+479metadata闭包通过；原生3D改源保存新进程重开通过，漏part反控phase5像素oracle拒绝；20轮实际owner/warm counter AND双画布内容及最后pin释放通过。复核work原14主例+4reopen、6owner反控/恢复和样例/旧兼容raw，复用边界与未变身份明确，不把work raw重标独立运行。原77唯一命令组无遗漏，新增4组全pending，原87graphics及AGP163/16native分母保持。独立runner因定向cargo重新构建，hash656e8c35…，与work2591a08e…分别记录；均不冒称完整merge门。

无实现/依赖/规范/ledger修改，无新增合同内finding或债务候选；主检出其他WIP保留。T-09关闭，status reviewed/current_step9/total10；T-10保持开放。完整locked driver/AGP必要面/原生/owner/fresh own-SDK-own-target consumer等81组、autographics-dev落地、Spec/ledger/归档/guard/批量到期归因均merge-owned pending；这些门通过前不得宣称交付。未运行root t/tv/tt/tb/tu/taa/tf/docs_gen、完整driver或AGP套件，未合并/发布/清理。

## 10. 待澄清事项

无必须追加询问的产品问题。固定AABB扫掠/初始闭接触、类型化单槽、v4/10、16credit、原128/CPU/源码上限和脚本停止边界明确。必要前置是848正式交付；T-02承担实现可行性，不预支成功。

离开初始接触特殊判定、自动滑动/解穿插、相对运动、其他shape精度、改预算/支持上限、SDK/AGP/生产写面或Schema/ACE/HMR均须具体修订/后继，不默默塞入。848验收若改正式接口/语义，T-01核受影响条目，实质变化回new修订后再依赖实施。

### 2026-10-26 work 启动记录

用户明确授权实施，GSTART关闭。848 archive最终pass及cleaned收据已核，G848关闭。唯一工作树 D:/autostack/.wt/lang-849/auto-lang，branch plan-849-dev，base/current dd7076a7f40b35ffda6b876c36f826af3443fb97（正式autographics-dev）。主检出既有文档及未跟踪锁文件保留。首次verify-only因新树尚无.sdk-source失败；随后按原prepare_sdk从固定Git对象恢复，681文件/17本地包验证通过。verify_source通过374源/资产hash、479包/29路径边、forbidden=0。未修改实现、未运行Cargo；T-01尚需完整冻结记录，T-02..08未实施，保持0/10，不构成ready-for-review。下一责任角色work，同树继续。
