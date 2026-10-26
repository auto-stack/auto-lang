---
plan_id: PLAN-851
status: execution_done
feature_name: AutoGraphics有界移动与滑动响应
author: [Codex]
created_at: 2026-10-26
updated_at: 2026-10-26
plan_revision: 4
current_step: 8
total_steps: 10
depends_on: [PLAN-849]
supersedes_spec_components:
  - docs/specs/auto-graphics/swept-aabb-profile.md
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
  - docs/specs/auto-graphics/bounded-motion-profile.md
touched_goals: [GOAL-007, GOAL-008]
affects: [auto-graphics, authoring-experimental, bounded-motion]
---

# [PLAN-851] AutoGraphics 有界移动与滑动响应

## 0. 变更摘要

849提供固定AABB扫掠及脚本停止。851增加**真实Auto实现的有限运动响应：一次位移请求，最多处理两次阻挡接触，沿墙滑动、墙角截停**。复用原Descriptor/Auto几何，不以Rust预设角色轨迹或接入完整物理引擎替代。

拟显式`autographics.behavior/5`、profile11及`ag_move_and_slide`，保持v1..v4原managed身份、查询/初始闭接触政策和收费。新动作单次24credits，留8次结果读取；内部最多两次扫掠和一次最终候选准入，不靠公开sweep16反复调用。32总额度与原128节点/CPU/源大小上限不变，先做整路径可行性。

规划时849尚在候选阶段；现已核正式849交付并基于`6bb85f354e718f6541256f4b3d75ca97af75b8c4`实施。851 r4 work已完成并提交`75452a3e1daaba74c5557e78b83eeab2cc1e0763`，独立final review及merge完整门仍pending。[850](archive/850-restore-cross-workspace-full-regression.md)已用于迁仓后跨workspace全量回归编排，与图形路线无功能依赖；本路线为848→849→851。

## 1. 目标

1. 对同维固定AABB叶子的一次world位移，在最多两次接触内计算安全前进、切向剩余运动和最终local变更；Auto负责求交/投影/结果，Rust只保留原验证和渲染职责。
2. 明确初始严格穿插、边界接触时向内/沿面/离开、clearance、二接触上限、f32精度停止及错误，避免角色贴墙后永久卡死或无限循环。
3. 结果含实际位移、未应用量、最终world原点、接触数/状态及最后阻挡对象/法线；与cast/sweep共享有界类型化结果身份，不误读旧槽。
4. 两个真实Auto项目证明2D沿墙/墙角、3D多part偏心GLB与层级变换下的滑动响应，保留绑定/保存/搬迁/重开及实际GPU闭环。
5. 验证128节点、源/CPU/credits、整帧失败/Stop恢复、最后owner/资源复用及旧版本兼容。

非目标：Godot/Bevy角色控制API逐项兼容，重力/跳跃/地面判断/坡度/台阶、自动解穿插、刚体/冲量/摩擦/弹性、相对运动CCD、移动平台随动、旋转/球/胶囊/凸包/triangle精确碰撞、多于两次接触、动态增删/形状写入、相机/屏幕射线、重写冻结双游戏、AutoUI组件/主题/Schema/ACE/HMR、RQHost/生产合入/Linux/发行。

## 2. 架构方案

```text
849正式接受Document/资源/Descriptor + 显式behavior/5
 → 完整图/几何配置 → Auto derive当前world
 → 动作开始快照 + 最多两次私有方向性扫掠/clearance/轴向投影
 → 对实际f32 local+parent候选作最终准入
 → 一次发布目标local/dirty + motion结果token
 → 原完整frame检查 → 原Iced/Surface/GPU → Pause/Step/Stop
```

L1局部实验driver增量，只写`experimental/autographics-auto-driver/`；SDK/auto-val/AGP、根crates/manifest、三锁、生产auto-ui/迁移树只读。不新建第二场景/图算法/碰撞后端或通用物理框架。超写面或支持合同先具体修订，重大架构扩面按L2；本计划不改849的既定范围。

| 门 | 成立条件与当前状态 | owner_stage |
|---|---|---|
| G849 | 849全部work、有效独立final review、冻结完整门、正式autographics-dev落地、Spec/ledger/归档/guard完成；T-01已核关闭 | work T-01前 |
| GSOURCE | T-01核正式849 full SHA/SDK/三锁/source闭包/原分母；在途WIP不作为基面；T-01已关闭 | work T-01 |
| GSTART | 用户已明确授权修订及全部work；已关闭 | work启动前 |

唯一实际树`D:/autostack/.wt/lang-851/auto-lang`、branch`plan-851-dev`，显式从849最终正式autographics-dev提交建树，不用master默认基面。实施/internal/final同树顺序，主检出记计划/附件/知识；无新agent/chat分派授权。禁止junction/symlink，移树先wt-guard clean。图形源码只入autographics-dev，I2/I3门保持。

## 3. 技术栈

复用实际Auto Parser/Session、有限f32、SourceSnapshot/Api/Play、spatial::Descriptor、scene4 CodePayload/Storage、stock Iced/wgpu。Auto标量/控制流处理slab与轴向投影，不需要sqrt/归一化/新数学或物理依赖。

初次规划时正式图形为848交付`dd7076a7f40b35ffda6b876c36f826af3443fb97`，SDK观察`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`。初读849时分支HEAD仍该基面、新增SweepV4是live dirty内容；核对期间work已冻结候选`6bb85f354e718f6541256f4b3d75ca97af75b8c4`，最终调查按该不可变Git对象取材，不能认定已正式交付。执行已以正式849重冻，见T-01与§9；以上初读状态为历史观察。

## 4. 需求分析与背景调查

### 4.1 授权、知识与版本状态

用户2026-10-26要求849开发中提前规划下一份，当时授权只调查草案；后续用户已明确授权实施、基于反例修订及全部work/提交/交接复审；仍未授权SDK扩面/新chat，merge另由后续阶段负责。采用滚动规划只取851一号。

已读[全局overview](../specs/overview.md)、[图形overview](../specs/auto-graphics/overview.md)、[空间查询](../specs/auto-graphics/spatial-query-profile.md)、[线段查询](../specs/auto-graphics/segment-query-profile.md)、[行为API](../specs/auto-graphics/behavior-api.md)、[项目绑定](../specs/auto-graphics/behavior-project.md)、[Play](../specs/auto-graphics/scene-play-session.md)、[849当前合同](849-autographics-swept-aabb-and-stop-before-impact.md)与实施入口研究/候选handoff。固定来源对象/hash及初读WIP到候选提交的状态变化见[来源清单](attachments/851/planning/source-observation.json.txt)，不反写849树。

初次调查时848已归档、849尚为候选；849后续正式review/merge/归档/guard已由T-01核验，原候选观察留作历史。swept-aabb-profile规范尚不存在，是849未来沉淀目标，851只列待交付modify；不把拟议/候选当canonical。850的回归编排由[语言Spec](../specs/auto-lang/project.md)和[UI Spec](../specs/auto-ui/project.md)规定；不是850功能依赖。完整AutoUI/默认设计、Schema/841/ACE/HMR及生产接线仍按[Design45](../design/45-autographics-wgpu-engine-and-game-prototypes.md)后继，不凭独立碰撞原型抵扣。

### 4.2 已观察真实路径

| 触点 | 事实 | 851消费点/缺口 |
|---|---|---|
| spatial.rs::Descriptor（正式848） | 全mesh静态offset/资源与base身份；Auto current world查询 | 复用叶子代理，不重扫模型/造中心半径 |
| behavior.rs::segment_transport/base_api（正式848） | 四dirty轴/visibility、原数组parent-first公式、trusted compact数字区，v1/v2隔离 | 消费正式849同机制；motion改local后正确标dirty，旧版字节不变 |
| 849候选 behavior_sweep_v4.at | 查询收费16、cast共享kind/token，初始闭接触t0/normal0；叶子不写入 | 不能两次公开调用后再读；motion内部方向性接触策略独立，公开sweep政策保持 |
| SourceSnapshot/Play（正式848和849候选） | typed API/profile、完整几何配置、来源和whole-frame验证 | 新MotionV5/profile11全路径，不旁路Session或只改profile数字 |
| behavior_project/Host::Panel/bin | scene4版本化绑定/raw共享与凭证隔离、正式UI/CLI | /5内容事务、来源/历史/闭包不变 |
| 849候选 sweep_feasibility/derive/host及handoff | 已有128/厚度/Host断言；work报告完成，final尚未验收 | 不当作851证据；motion独立双接触/终值准入/预算探针 |

关键缺口：v4两sweep=32、结果getter另收费；闭接触无方向区分，直接重复调用会卡住；world位移转local时必须遵守实际f32 parent加法，不能把数学O+D当已发布world。851显式解决这些有界问题，不把完整角色控制器加入本版。

## 5. 详细设计

### 5.1 下一实施入口与有界可行性

T-01核G849后，T-02负责最小MotionV5/profile11 source/typed构造/consumer接线和实际Auto候选核，沿SourceSnapshot→Parser/Session→完整spatial配置→Auto动作→Play整帧验证；T-03再补完整准入/兼容/项目，T-04收口全部数值/接触，避免前后依赖循环。UI/原生/项目由后续任务关闭，预研不是最终pass。

r4执行方案：标量提升、第二扫描逐轴延迟计算与同图leaf同步：每阶段将actor静态offset、当前world/local/parent、方向和数组借用提升为局部标量；第一扫描同时完成全部匹配目标几何准入及严格初始穿插检测，然后进行方向slab；第二扫描复用本动作已验证的静态目标条件，先计算X区间并作保守分离剪枝，分离时不执行Y/Z表达式；Y同理；只有未分离候选才执行余轴商。保留实际执行算术finite检查，首次扫描不得这样跳过全目标几何验证；最终扫描只验证实际local+parent候选的严格内部相交。r4发布时同一次受检leaf提交更新原fields及同一worlds行（actual local+parent），不建第二缓存；v5 derive在无后来setter且该leaf已同步时消费dirty而不全图重算。group/visibility或已dirty轴setter仍走原轴dirty路径；v5普通leaf setter仅在该轴clean时也同步同一world行，保持原维度校验且不影响其他轴dirty，parent本轴已dirty则禁止捷径。该局部实现需专门覆盖前后group/leaf交错、native非finite及直接world字段，不能仅依赖motion样例。原whole-frame验证仍最终发布边界，错误VM整体销毁。零方向轴保留闭接触判定，不能用开区间剪掉外部切点。优先消除每target重复actor数组索引、全局读取和重复表达式，不改变f32顺序。仅使用局部固定标量/原数组借用，禁止候选数组/第二图/持久缓存；不改SDK/VM计步/源码/Descriptor，不借native Rust求解。

必须实际通过：2D/3D各128总节点，宽树含多个parent交替、深树合法最大深度，全部三轴正负/不等方向、late-axis miss、nearest随ordinal递减、单墙、两接触角落；正常SourceSnapshot/Session及Host路径，一motion24+八结果getter=32。至少两次合法group scalar写覆盖动作前/后，动作后写可作用于别的group；直接读消费者字段验证最终world，不让事后ag_world lazy repair掩盖。原v3/4控制与源码身份不退化。

动作最多两次扫掠候选扫描与一次最终准入扫描；无无限重试/递归。100000steps/120resumes/250ms callback、30s init/2s tick/8ms pump、源256KiB/managed64KiB/combined320KiB不变。精确测managed长度、yield/CPU/固定状态；不把yield当精确steps或32credits当CPU保证。

r3以d157d2872永久反例及r2扩展12组合矩阵为红控制，先实施不变量提升，再按实测热点完成至多三轮有理由的成本修正（r2三轮已保留，r3基于新12组合成本诊断重新绑定逐轴延迟方案）；同一未变代码/输入不盲重跑。关闭仍须完整V-07/Host/managed门，无论简单反例是否转绿。若仍无法达到原预算，保留成本诊断并needs_replan；不得减少128/两接触、提预算、删困难方向、改SDK或把小场景绿当全支持。

### 5.2 公共动作与支持输入

拟`ag_move_and_slide(node int, visible_only int, dx float, dy float, dz float, clearance float) int`，D为world位移，非velocity；支持固定AABB叶子，2D dz精确0，visible_only0/1，clearance须finite且>0（world单位，用户显式指定，不加隐式默认）。移动叶子当前各有效轴world box及静态offset宽度须>0；退化移动体具名InvalidMotionShape错误，target退化box仍沿原代理支持。node0合法，−1/group/越界/错维参数、非finite按受检错误拒绝。

目标同维其他叶子，自动忽略自身/group/其他维度；visible_only只过滤目标effective visibility，不看alpha；隐藏actor仍可移动。使用原Descriptor全部mesh/旋转后AABB，不用scale/2猜GLB中心。目标在调用瞬间静止，当前callback先前parent/leaf写入可见，动作中无用户callback/重入/其他写者。

动作开始保存该叶子的local及其实际world；只在正常候选关闭后写一次该叶子local并标dirty，不能修改父组、目标、visibility或静态字段。内部可派生最新world，不新建场景/VM/Descriptor。动作后普通setter仍可改变位置或移动父组：碰撞保证只覆盖本次请求，不对后来显式用户写入作隐形物理约束。

### 5.3 接触、滑动与f32规则

基础构形区间/表达式沿正式849：移动静态offset[mlo,mhi]、目标当前world[tlo,thi]，每轴L=tlo-mhi/H=thi-mlo，slab在O+t*D、t∈[0,1]求最近f32进入，tie最低Run ordinal、同候选normal tie X/Y/Z。不归一化、不用epsilon扩boolean，公开cast/sweep本身保持原闭区间语义。

**motion自己的接触政策**：

- 初始O严格在任一目标构形box所有有效轴开区间内部，返回InitialOverlap，不移动、不解穿插；最低ordinal为报告node，normal0、contacts0。无论D方向/是否0，都先校验全部匹配目标几何，坏几何不能被该状态遮蔽。
- 初始位于box边界时，如果存在一个边界轴满足低面D<=0或高面D>=0，本段不会进入该box内部，该候选不阻挡；允许沿面/离开。角点同时向所有相关面内运动才t0阻挡，取X/Y/Z进入面。此方向性特判只在motion私有核，v4及v5公开ag_sweep_aabb仍初始闭接触hit/normal0。
- 从box外到达的闭接触含t1，保守阻挡；不把切点命中改成穿过。零位移在非InitialOverlap时Free/actual0，仍完成几何检查。

每个阶段：无hit则接收该阶段完整位移；有hit先计一次接触，normal为轴±1。令a=abs(D的normal轴分量)，按f32先算t*a；clearance>=t*a时safe_t=0，否则safe_t=t-clearance/a。这个分支避免先算可能溢出的clearance/a再clamp。advance=D*safe_t；剩余R=D*(1-safe_t)，只将朝阻挡面内的normal分量置0，保留其余分量。Clearance只是沿该阻挡法向的停止留隙，不扩大query形状/改变原sweep hit，不承诺一次处理已有skin穿插。

最多两个阶段/两个接触；第一接触后的切向R进入第二扫掠，不能直接无检测应用。第二hit按同样留隙推进后终止，未检剩余丢弃，状态ContactLimit；不再第三扫掠/滑行，不把剩余放队列跨tick。第一hit后R全0可提前结束Clipped。每tick由用户重新给D，重复沿墙输入不积累上一tick的未应用量。外部先前setter的位移不算本次动作delta。

局部虚拟推进必须重建实际f32 `candidate_local` 与原有`local+parent_world`顺序得到world，再继续扫掠；不能直接假定parent为0或数学world增量=实际发布增量。先前被阻挡的轴分量保持禁止，不用dot/normalize引入其他精度语义。候选动作和方向/商/advance/剩余/local/派生world算术都finite，出现非finite具名错误并整callback拒绝。

最终有限候选必须经一次完整准入扫描：按实际待写local+parent产生的world，重新核全部匹配目标world几何与同一构形空间的严格内部相交；不能只看已选中两面。若因f32量化无可接受进展、有限区间坍缩，或最终candidate进入目标内部，则返回PrecisionStop：不写本次动作local，保动作开始位置、delta0/unapplied=D，contacts0/node−1/normal0。该受控状态不是API Fault、不撤销callback中动作之前的合法setter；不得用它吞掉非finite、不可用几何、错误参数或原CPU/额度错误。初始严格穿插走InitialOverlap，不能混进PrecisionStop。该fallback不递归缩步寻找答案。

“无可接受进展”指本应有非零advance的有限请求在实际local+parent量化后无法改变world，并且后续投影也不能产生可接受位移；clearance按规则令safe_t=0的普通阻挡不是精度失败。零D的正常Free与可表示的沿墙滑动不得误报PrecisionStop。外部起点在box之外但进入t下溢0，仍为方向性进入normal，不当作初始穿插或边界离开。

布尔/排序/状态/node/normal精确，f32观测容差沿848/849，只比较输出不改变接触/tie。大坐标/小位移的合法PrecisionStop、普通clearance内0进展Clipped与NaN/overflow Fault要有可区分控制，不允许把所有困难输入返回PrecisionStop冒称正确运动。

### 5.4 结果、预算和生命周期

正常命令返回正token；拟getter：`ag_motion_status/contacts/node(token) int`、`ag_motion_origin/delta/unapplied/normal(token,axis) float`。axis0/1/2、2D z0。delta=实际最终world−动作开始world；unapplied=请求D−delta（未应用量，包含阻挡部分，不是下一tick待执行命令）。origin为实际最终world，非接触点；node/normal为最后一次阻挡，Free为−1/0；InitialOverlap/PrecisionStop按§5.3定义。后续setter不改该快照。

| status | 值 | 行为 |
|---|---|---|
| Free | 0 | 全部请求可接受；无接触，实际world位移可能有f32舍入 |
| Clipped | 1 | 一次接触后已安全推进/滑动或被阻挡；可能delta0，不是Fault |
| ContactLimit | 2 | 处理到第二个阻挡后终止，contacts2；剩余不再执行，即使投影后已0 |
| InitialOverlap | 3 | 起始严格穿插，contacts0，位置不动，node为最低ordinal初始穿插者 |
| PrecisionStop | 4 | 有限精度候选不能接受，本次不动、contacts0/node−1/normal0 |

v5扩既有单槽kind为motion，cast/sweep/motion任一尝试先失效旧槽，共用一条Run内正i32递增序列；内部扫掠不签发用户token/不覆盖外层结果/不重复收费。对应kind getter才可读；错/旧/无token或跨callback具名InvalidResult，耗尽先Fault不wrap，普通AABB不失效。begin/end/Stop统一失效；不是持久ID或安全引用。

每callback32credits，motion每尝试24、既有cast8/sweep16、所有结果getter/AABB1；第一超额在几何/结果前QueryBudget。例motion24+status1+contacts1+delta三轴3+normal三轴3=32；第二motion不是“排队下次”，会使当前callback整帧Fault（含第一次动作不发布）。不重复按内部两sweep收16，不说明无限内部操作免费，内部扫描上限受§5.1约束。旧v1..v4收费、源/初始接触规则原样。

v5总query结果/序列/计数/kind加四dirty标量≤40个32-bit逻辑标量（160B）；一套固定结果槽，不留接触历史/候选数组/每query新对象/第二图缓存。实际NanoValue、键String结构/容量、共享map边界及compiled program/stack分开实测，不称160B为SDK总硬字节上限。64轮重复动作对象/存储不随次数增，20轮实际owner另测。

任何API/配置/CPU/数值错误sticky第一错、关闭VM/整callback拒绝/保last-good；内部候选不能提前产生外部像素或污染下一Run。Starting完整配置/取消、Pause/Step/Resume、Stop原编辑内容与新Run重建沿既有合同；动作不是绕过原whole-frame验证的Host命令。

### 5.5 版本、项目及资源接线

MotionV5/profile11贯通SourceSnapshot read_for/from_bytes_for、实际AST准入/reserved、managed regions/完整hash、Play typed配置/profile/header/consumer、Host::Panel与`--behavior-api v5`。v1..v4 managed/API/ENTRY/bridge hash、合法旧ag_motion_*/ag_move_and_slide私有helper保持；新名只在显式v5保留。不得用户关键字猜版本、自动升级/fallback或改raw源码。未知值/缺值/孤立选项和--inspect坏组合exit2，合法默认/原v1..v4保留。

scene4结构不变，精确新增Binding.api /5；same-raw可共享原Arc/blob，准入/编译凭证按版本和managed身份隔离。Bind/Open只编译不执行动作。切版本是内容事务；Undo/Redo/失败保旧/dirty/Save/SaveAs/Verify/Open/旧epoch/新root/memory或确认blob来源沿845/849合同。只存接受源码/角色/场景，不存滑动结果、runtime位置或待执行剩余。旧reader遇/5明确unsupported，不丢绑定。

原8raw/2MiB、history/诊断/report预算保持。复用同一Run Descriptor/实际ModelData借用、GPU Arc/targets/pipelines；无warm重扫/上传/逐帧读源或readback。Input和业务由用户Auto，无Rust按样例名预设动作。计量compile/query/full tick CPU、工作集/资源和排除项，不把60Hz逻辑dt当实时保证。

### 5.6 两个真实Auto项目与后继

新增拟`samples/motion_projects/maze2d/`：2D叶子用真实输入给D并调用motion；演示斜向撞长墙后切向前进、贴边继续/反向离开、墙角两接触截停、高速薄墙无穿越、初始穿插明确反馈。独立手算例：移动offset[-1,1]，竖墙x[4,4.2]足够长，O=(0,0)、D=(10,4)、clearance=.25，则第一hit t=.3，safe_t=.275，最终约(2.75,4)，而简单停止为(2.75,1.1)。不得用只停住也判通过的oracle。

新增拟`samples/motion_projects/corridor3d/`：实际偏心多part GLB作actor，非零parent/静态yaw/pitch/scale，XZ墙面和角落由全mesh代理决定；真实Auto碰撞响应改变实际渲染，单独测Y面/三轴方向与三面夹角第二接触上限。不是把2D常量换space3。原冻结asset/许可来源不变，不加材质/动画/角色美术工具链。

两项目同tick先写后动、动作结果随后setter仍稳定、末尾发布world/Pause不重复/Step一次/Resume不重置app_start/Stop恢复编辑内容。正常沿墙及自由移动不得落PrecisionStop；初始穿插/数值精度状态可见但不冒称可解穿插。

正式UI选v5/Bind/SaveAs产物复制仓外不同cwd，原root/source/导入路径不可用，新进程同一runner字节SHA --open再显式Run；核真实Auto运动/node/normal/状态、独立f32/reference、全mesh最终像素。只改用户D/clearance/分支，Update/Save/reopen改变实际滑动，无Rust重编。透明/漏part/错asset/source/API/partial对应拒绝。原双游戏、Move/Patrol、Zone、Beam/Sight、Dash样例/owner分母并列保留。

完成851后为“查询→遇障停止→有限运动响应”的可用原型阶段，仍非完整角色/物理引擎。重力/地面/跳跃、动态场景/实例生命周期、动画/材质及完整AutoUI/Schema/生产融合按滚动优先级另立计划，本轮不取第二编号。

### 规范增量

全部拟议；swept-aabb-profile先等849沉淀，再描述v5边界，原v4闭接触政策不改。merge前不写canonical。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/bounded-motion-profile.md | 查询/脚本停止→两接触动作、方向性初始接触/留隙/终值准入/状态、24credits | 滑动行为可验收 | AC-02, AC-04, AC-05, AC-08 |
| SD-02 | modify | docs/specs/auto-graphics/swept-aabb-profile.md; docs/specs/auto-graphics/segment-query-profile.md; docs/specs/auto-graphics/spatial-query-profile.md; docs/specs/auto-graphics/behavior-api.md; docs/specs/auto-graphics/behavior-project.md | 原v1..v4保持→显式v5/profile11、单槽motion kind/scene4绑定 | 兼容与不串结果 | AC-03, AC-04, AC-06 |
| SD-03 | modify | docs/specs/auto-graphics/scene-play-session.md | 查询不写→有界候选一次local提交/dirty、Fault/Stop、原生命周期 | 用户动作不能旁路准入 | AC-01, AC-04, AC-05, AC-07 |
| SD-04 | modify | docs/specs/auto-graphics/auto-source-driver.md; docs/specs/auto-graphics/prototype-profile.md; docs/specs/auto-graphics/project.md; docs/specs/auto-graphics/integration-contract.md | 原查询项目→两滑动项目与数值/owner/原生边界，I2/I3仍pending | 交付含义准确 | AC-06, AC-07, AC-08, AC-10 |

## 6. 测试设计

用户明确授权修订后继续全部work，当前r4；按阶段执行定向Cargo/原生/GPU与必需验收，禁全档及docs_gen；未建立fixture仍由对应work任务建立，未来预期不是已执行证据。未授权新agent/chat或指定模型。实施/internal/final同树顺序；final独立上下文/停止实现写入，自查不能代独立最终pass。

未来851 driver目录先用正式849来源/SDK核验：`python tools/prepare_sdk.py --repository D:/autostack/auto-lang --verify-only`、`python tools/verify_source.py`和849正式source门（T-01核入口）。开发`cargo check --locked --all-targets`、`cargo fmt -p autographics-auto-driver -- --check`（覆盖可写Rust，沿848/849避开只读SDK嵌套workspace边界）；新定向`cargo test --locked --test motion_<对应target> -- --nocapture --test-threads=1`及触及旧target。0selected/skip/缺工具/GPU不算通过。

| case_id | AC / work owner | 真实输入和入口（new由任务建立） | 可区分oracle / 反例 | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01；T-01/08 | 正式849/SDK/三锁/source/producer/fresh consumer | 正式交付+错pin/缺源/资源漂移拒绝，旧分母不缩；WIP不当基面 | 固定正式闭包 |
| V-02 | AC-02；T-04 | new motion_geometry、actual Auto单墙斜向/薄墙/空场景 | (2.75,4)对只停(2.75,1.1)，独立hand/f32；去掉第二扫掠或切向投影变异红 | 固定AABB静态目标 |
| V-03 | AC-02；T-04 | 两墙角/3D三面/ordinal和XYZ tie/±0/退化target/初始穿插/边界向内沿面离开/0D | 两接触截停、closed sweep仍hit但motion沿面可走；InitialOverlap不动，正常不误PrecisionStop | 无第三次/解穿插/坡地 |
| V-04 | AC-02, AC-05；T-04/05 | 偏心多mesh/parent+静态旋转/父写后动；大坐标小D/有限坍缩/ULP；native NaN/Inf参数local-return-container | 真offset/实际local+parent world、精度停止可辨，非finite仍Fault；坏matching几何不能被正常状态遮盖 | f32代理非triangle实数精确 |
| V-05 | AC-03；T-03 | SourceSnapshot/Parser/Play v1..v5、新名旧合法helper、伪source/API/base/profile | 旧managed/hash/reserved原样，显式v5全身份、错误不dispatch；保默认和旧版本 | 无猜版本/fallback |
| V-06 | AC-04；T-04/05 | motion→setter/读旧→cast/sweep/错kind/旧/无token/上限；外部先前setters/两次motion | 单槽无ABA/快照稳定/边界失效；内部不签发token；第二动作超额整帧拒绝，原图不受部分提交污染 | token仅该callback |
| V-07 | AC-05；T-02/05 | 128宽深/多parent/难方向/late miss/递减nearest/2contacts，motion24+8get；原v3/4控制 | 原CPU门+64KiB下真实完整Host通过，首越额先拒；直接字段检world，不lazy repair；0selected不绿 | 不承诺任意脚本32组合 |
| V-08 | AC-04, AC-05；T-05 | 初始/终值准入、缺配置/Starting取消/Step/Pause/Fault后修源/Stop | 不提前写local/像素，初始化完整才callback，first-error/last-good/closedVM/Stop恢复；precision状态不代错误 | 无VM rollback继续 |
| V-09 | AC-06；T-03/06 | UI/CLI v5/坏选项及inspect、same-raw版本事务/Undo、SaveAs/Verify/Open/新root/旧epoch/失败 | 准入凭证隔离、raw可共享、切版本非noop/失败保旧；Open不执行动作；runtime不入Save | scene4结构不变 |
| V-10 | AC-06, AC-08；T-06/07 | Maze2D/Corridor3D真实Auto/OS输入、UI绑定SaveAs、搬迁新进程/改源/全part像素 | 确有切向移动和墙角限制、独立位置/状态、同runner；真实透明/漏part/错asset/partial非零 | counter不代内容 |
| V-11 | AC-07；T-08 | 64轮motion存储、20轮混合版本/Undo/Open/Run/Fault/Stop、合法pin及重扫/上传/保留变异 | 固定逻辑+实测值/键/map边界、VM/Descriptor/source最后owner与GPU守恒、warm无重扫重传/读源 | 任意用户heap/驱动/Iced内部排除 |
| V-12 | AC-08；T-08 | API/状态/预算/初始方向性/clearance/f32/Spec delta/source与准确merge矩阵 | 实际代码/文档/原分母/反控一致，raw可复跑，support排除不藏缺项 | 静态非运行验收 |

每case绑定命令/退出码/选择断言数、code/SDK/三锁/fixture/实际用户源/managed/算法/runner SHA与raw/像素。独立参考不调用被测motion核、不以被测结果构造expected。反控实际改目标机制并有恢复正控，setup/工具早退不能冒充命中目标错误；固定CPU超额不能仅换counter让测试绿。

T-08列准确merge矩阵/Spec proposal；T-09独立final pass后T-10冻结完整locked driver/AGP tests/build、原双游戏/合成counter AND content/835..849原生/owner/source/package与fresh own-SDK/own-target consumer，再加motion正反/128/搬迁。正式848定位分母216driver、AGP163/16native/原87graphics；849新增分母未知，T-01/08以正式实际清单和命令重核，不照抄数量当结果。复用先核代码/依赖/配置/输入未变；代码变化回受影响review再冻结。

本851限定driver-only不触根t/tv/tt/tb/tu/taa/docs_gen；开发/internal/final只定向/check/fmt及必需验收。批量到期由merge按现行850编排与regress流程在主检出单实例处理（现行`cargo tf --batch`串行语言/UI及tt/tb），不可退回只测迁前单仓，不在851树并跑全量。不能继承历史847红当当前事实或冒称批量绿。固定SDK预存warning按正式849清单记，driver新增warning清零。

## 7. 验收标准

| ID | owner_stage | 可观察标准 |
|---|---|---|
| AC-01 | work | G849/GSOURCE、正式基面/唯一树/只读边界和旧完整分母冻结，V-01 |
| AC-02 | work | actual Auto两接触移动/滑动、初始方向性/留隙/终值准入/状态与独立参考一致，V-02/03/04 |
| AC-03 | work | 显式v5/profile11全身份准入、旧v1..v4字节/语义/合法helper兼容，V-05 |
| AC-04 | work | 单次目标local提交/实际dirty/world、单槽token/快照/错误与原生命周期成立，V-06/08 |
| AC-05 | work | 128难例/两接触/32credits、源/CPU原门、非finite/配置/whole-frame闭合，V-04/07/08 |
| AC-06 | work | UI/CLI、版本事务/来源/闭包和两个真实Auto滑动项目，V-09/10 |
| AC-07 | work | 固定状态/64查询/20轮真实owner与合法pin、原GPU资源/扫描上传不退化，V-11 |
| AC-08 | work | 同runner仓外new-process/OS/GPU正反无必需gap、API/状态/SD/准确merge矩阵齐，V-10/12 |
| AC-09 | review | 独立核AC-01..08/SD/遗漏/延后/workaround/健康、revision+code绑定pass，完整merge门明确pending |
| AC-10 | merge | 冻结完整门过才落autographics-dev，Spec/ledger/归档/guard/到期收据齐，master仅知识 |

## 8. 执行步骤

当前execution_done8/10；G849/GSTART/T-01..08已关闭，代码冻结75452a3e1。未来T-01..08全部work无必需gap才execution_done8/10，T-09独立pass为reviewed9/10且完整门pending，T-10真实收尾才10/10。原入口T-01→T-02已关闭；下一负责角色为独立review，禁止把work自查当final pass。

| task_id | owner_stage | 依赖 | 实际触点/拟新增路径与动作 | 关闭条件；AC |
|---|---|---|---|---|
| T-01 | work | G849/GSTART | [x] 正式849/Spec/收据、唯一851树、SDK/三锁/source/原分母| path/branch/base/HEAD/正式闭包及未闭债务冻结；AC-01 |
| T-02 | work | T-01 | [x] behavior.rs/Play/Descriptor；new api/behavior_motion_v5.at初核、research/bounded-motion-feasibility.md、tests/motion_feasibility.rs| r2局部标量提升/三扫描方案，永久128反例及完整128难例/2contacts/24+8/终world/managed与反控通过；AC-02, AC-04, AC-05 |
| T-03 | work | T-02 | [x] behavior.rs/behavior_project/Host typed选择；new tests/motion_api.rs、motion_project.rs| v5/11全身份/准入/版本隔离/旧helper与scene4事务、V-05/09；AC-03, AC-06 |
| T-04 | work | T-03 | [x] motion Auto核；new tests/motion_geometry.rs| V-02/03/04/06几何/方向接触/状态/留隙/精度/实际local-world、独立参考/反控；AC-02, AC-04 |
| T-05 | work | T-04 | [x] Play/callback/最终候选/whole-frame/consumer；new tests/motion_host.rs| V-04/06/07/08、CPU/原生命周期/错误/最后world无gap；AC-04, AC-05 |
| T-06 | work | T-05 | [x] Host::Panel/bin；new tests/motion_cli.rs、samples/motion_projects/maze2d和corridor3d| V-09/10正式UI/CLI/绑定和两维实际滑动/角落用户逻辑；AC-06 |
| T-07 | work | T-06 | [x] new tools/check_bounded_motion.py、OS/GPU/仓外reopen| V-10同runner/新进程/全mesh/改源/状态/正反/partial无gap；AC-08 |
| T-08 | work | T-07 | [x] new tests/motion_owners.rs、tools/check_motion_owners.py、851来源闭包/verifier、research/bounded-motion-handoff.md/Spec proposal| V-01/11/12、64存储/20owner、原受影响定向/check/fmt、准确merge矩阵；AC-01, AC-07, AC-08 |
| T-09 | review | T-08 | 同树冻结code/Plan/Spec/raw；主检出独立review附件 | AC-01..08/SD/遗漏/延后/workaround/健康逐项及revision+code结论；AC-09 |
| T-10 | merge | T-09 pass | 冻结完整门→autographics-dev；主检出Spec/ledger/archive/guard/到期 | AC-10全部真实关闭，改代码先受影响复审，不提前标交付 |

## 9. 复审记录

2026-10-26：stage:new；PLAN-851 revision1；outcome:blocked（G849正式交付pending，草案可评审）；next:849独立验收/冻结完整门/合入和知识收尾后，按后续work指令从851 T-01冻结正式基面、T-02验证有界方案。changed:T-01..10、AC-01..10、V-01..12、SD-01..04；drafting0/10。

静态检查见[preflight](attachments/851/planning/preflight.md)。本轮未改849合同/进度或其实现树、未改canonical/ledger/依赖、未创建worktree/执行Cargo或原生/GPU、未分派agent/chat。保留他人主检出WIP。新API/fixture及期望均为未来执行合同，不作已实现/通过声明。

### 2026-10-26 work T-02 A/B 有界实现与反例

stage: work | plan_id: 851 | plan_revision: 1 | outcome: needs_replan | worktree_path: D:/autostack/.wt/lang-851/auto-lang | branch: plan-851-dev | base_commit: 6bb85f354e718f6541256f4b3d75ca97af75b8c4 | code_commit: d157d287268f7dcf1c08fbc28d4a6d8a1c263230 | dependency_revisions: T-01 SDK/三锁不变 | task_ids: T-02（未关闭） | next_responsible_role: new（有界算法/执行成本修订）

已实现候选动作/24credits/结果快照/虚拟local两接触/终值准入和t0法线修复；正式持久反例与诊断在851树 `experimental/autographics-auto-driver/research/bounded-motion-feasibility.md`，raw四份已提交。方案A独立初检在128节点2D耗尽100000steps；方案B融合初检/展开终检后2D通过（99slices非精确steps），同输入3D仍耗尽原CPU门。`cargo test --locked --test motion_scan -- --nocapture --test-threads=1` 实际3passed/1failed/0ignored；通过项含手算沿墙最终(2.75,4)、12个2D/3D边界方向控制和私有扫描不发token/收费。`cargo check --locked --all-targets`、driver fmt check、diff check通过；只读SDK预存146warnings，未改SDK/锁/根代码，未跑全档。

依据§5.1“两候选仍失败…同计划needs_replan”路由。T-01完成，T-02..08仍开放；current_step=1/10、status=executing。未达到execution_done，不能交独立最终复审。所有后续任务依赖T-02，现无独立可继续的交付单元。必须保128/两接触/原CPU与SDK边界修订局部方案；若需改变验收，先具体用户裁定。完整Host/深宽树/精度/旧版兼容/项目/native/GPU/owner仍未完成，不能用局部绿遮盖。代码已提交且唯一树clean，保留供修订与后续实施；review/merge门pending。

### 2026-10-26 r2 算法方案修订

stage: new | plan_id: 851 | plan_revision: 2 | outcome: pass | next: work T-02 | changed: §5.1算法/§6执行时序/T-02；AC-01..10/V-01..12/SD-01..04保留。用户本轮明确授权修订后继续全部work，无需重复确认。

基线d157d287268f7dcf1c08fbc28d4a6d8a1c263230；永久反例motion_scan::bounded_128_two_contacts，实际2D99slices/3D100000steps红。观察重复actor offset索引、全局数组读取、每轴边界判断与第二扫描重复验证；r2固定局部标量提升及验证复用为成本假设，不把它写成已通过事实。实际路径SourceSnapshot→Auto callback→两扫描→实际local+parent终检→原whole-frame/Host，不改producer/consumer或所有权。SDK/三锁/128/两接触/100000steps/120resumes/250ms/64KiB全部保持。第一扫描必须检查全部匹配几何，不能靠nearest/初始穿插掩盖错误；第二阶段不能拿数学world代替实际f32。

三轮有理由的实现成本修正是有界研究，不是三个review轮；每轮回放原反例及受影响控制，T-02关闭需完整V-07。先前r1局部证据仅历史，不支持r2最终通过。已有T-01不重做；T-03..08依赖仍保留；无必需用户语义决策。Canonical swept-aabb-profile在主检出已正式849沉淀，851树基面知识副本较旧，查主检出canonical而不改SDK/实现基面。

### 2026-10-26 r3 第二阶段逐轴延迟修订

stage:new | plan_id:851 | plan_revision:3 | outcome:pass | next:work T-02 | changed:§5.1第二扫描表达式执行顺序；所有AC/V/预算/SDK边界不变。r2三轮改善原反例到78/98slices且4tests绿，扩展motion_feasibility实际12组合5过7CPU红，不能关闭T-02。首次几何全检查保留；第二扫描只有动作内已经验证几何，无外部回调/改图，允许按分离轴延迟剩余算术；这与canonical“Skipped later arithmetic does not create an error”一致。当前代码尚在同树修改中，提交前保持不送审。用户已授权基于反例修订并继续，未扩范围。

### 2026-10-26 r4 叶子发布成本修订

stage:new | plan_id:851 | plan_revision:4 | outcome:pass | next:work T-02 | changed:§5.1 v5 leaf-only同图world同步。r3完整矩阵2D6过/3D6CPU红；独立首扫描诊断44/54/61slices未超限，故首扫描本身可承受，剩余成本包含动作后全图3轴derive和group写后derive。r4只同步已验证无子节点leaf的同一world行并记录一个固定标量，最终derive可消费该同步；后续group/visibility/已dirty轴setter保留完整传播；第三成本修正将同图clean-axis leaf同步复用到动作前的leaf setter，避免原例为设置单leaf的Y/Z重算全图。需新增该路径代表性交错控制。firstscan单独诊断不是整体CPU通过证据，不增预算。所有AC与原producer/whole-frame/资源边界保持。此为本次第三次有证据的方案修订，若完整成本门仍失败则保留诊断交回，不继续无界改方案。


### 2026-10-26 work 启动核验

用户明确授权实施，GSTART关闭。849归档最终pass、正式source 6bb85f354e718f6541256f4b3d75ca97af75b8c4、canonical 8087a6bbd6bc81518e469e0a65592a856b4caf3d、ledger fa7711d0222c9e9852f56eb32fe986fa16dab45e及cleaned收据已核，G849关闭。唯一树 D:/autostack/.wt/lang-851/auto-lang，branch plan-851-dev，base/current为上述正式source；已断言路径和分支。主检出既有文档及未跟踪crates/test锁和fixture保留，未纳入实现。T-01冻结尚未全部完成，T-02..08未实现，current_step保持0/10；不得视为ready-for-review。next_responsible_role: work。

### work T-01 基线冻结

T-01 [x]：正式849/归档cleaned、唯一树/分支已核；三锁/source-lock逐项SHA256与849冻结清单一致。SDK恢复681files/17local；source验证402hashes，metadata479packages/29edges/forbidden0。基线记录位于851树experimental/autographics-auto-driver/research/motion-baseline.json.txt，保留247 producer+247 fresh consumer、81组、AGP23slots/163tests/16native和87graphics语义分母及R849-NATIVE-CLOCK风险。T-02..08仍pending，未交复审。

### work T-02 实现中：版本身份基础

在唯一851树实际修改src/authoring/behavior.rs，新增显式MotionV5 /5/profile11、沿既有segment/sweep transport接线与独立region名字；tests/motion_api.rs两项通过，含实际Session main与v1..v5 profile/version断言、raw字节保持/同raw身份隔离/未知版本拒绝。cargo fmt -p autographics-auto-driver -- --check及git diff --check通过。SDK预存146warnings，未修改SDK。日志在该树driver target/motion-api.log。该实现仅版本基础；motion动作/结果/几何/预算/Host/项目/native/owner尚未实现，T-02未关闭，不交复审，current_step仍1/10。

### 2026-10-26 r4 work 完成并冻结

stage: work | plan_id:851 | plan_revision:4 | outcome:pass | worktree_path:D:/autostack/.wt/lang-851/auto-lang | branch:plan-851-dev | base_commit:6bb85f354e718f6541256f4b3d75ca97af75b8c4 | code_commit:75452a3e1daaba74c5557e78b83eeab2cc1e0763 | dependency_revisions:SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467，三锁/source-lock逐字节不变 | task_ids:T-01..08 | blockers:none | next_responsible_role:independent review T-09（同树只读验收，停止实现写入）

T-02原100000steps反例已关闭；r4原128两接触实际2D78/3D88slices，12组合+混合正负不等三轴128递减nearest全部通过，最重98slices（不是精确steps）。完整Host128宽/深125groups+3leaves、前后group写入、两接触及直接actor/target最终world通过。独立手算沿墙(2.75,4)，删第二扫描实际得到(2.75,1.1)红；大parent相消下local候选量化world8穿墙，删最终准入确实穿墙，原检查PrecisionStop保world0，恢复正控绿。外部切点初始边界区分已有永久红/绿证据。

新增33个唯一Rust定向测试全部通过，旧受影响19个通过；64callbacks对象不增长、20轮真实混合owner及6篡改红/恢复绿通过。29固定标量/116逻辑B，实际NanoValue232B、String结构696B/键容量408B、共享map另计；managed49223B<64KiB。check --locked --all-targets、driver fmt、diff检查通过；固定SDK146预存warning，driver新增0。

两真实Auto项目及同runner SHA256 13a6d0cb5da88d73fb7d77148337ff5a62201a810b302a9c4d1d1110e25dd375：仓外D:/autostack/.verify/lang-851-native-r4-1六正向case+四额外新进程reopen全部phase14通过；八源码/资源/透明/漏part/错asset/partial反控命中预期拒绝；旧Dash2D/3D同runner通过。Source430hashes、SDK681files/17local/29edges/479packages/forbidden0及四来源反控/恢复通过，157份raw按Git暂存实际字节校验一致。代码仅driver；工作树clean，master他人WIP保留。

当前证据/全部AC闭合与Spec proposal/准确merge矩阵见851树 `experimental/autographics-auto-driver/research/bounded-motion-handoff.md`、`bounded-motion-spec-delta.md`、`motion-evidence/receipt.json`，共享收据见[work r4](attachments/851/work-r4/handoff.json)。无work-owned findings/必需gap/待用户语义决策；execution_done8/10。执行自查不等于独立final pass；T-09待独立上下文绑定r4+上述commit审查AC/SD/遗漏/延后/workaround/健康，T-10在pass后跑冻结完整locked driver/AGP及原全部native/owner/source/package/fresh own-SDK own-target consumer+motion门。原247分母+33新增=预期280，merge必须实数重核，不能当作280已跑绿。原AGP23slots/163tests/16native及87graphics语义分母不缩。根全档/docs_gen未跑；到期batch仅merge按850主检出单实例。未合入、未改canonical/ledger、未归档/移树。

## 10. 待澄清事项

无必须另问用户的产品问题。两接触/静态AABB/有限方向性初始接触、24credits/40逻辑标量、显式v5与错误/正常停止边界已明确；实现可行性由T-02有界验证，849正式交付与启动授权已核；r4 work已完成，当前待独立review。

若849验收改变正式接口/几何/f32政策，T-01先审影响，实质变化回new修订。增加接触数/滑行范围、重力/解穿插、改节点/CPU/源码上限、SDK/AGP/生产写面或Schema/ACE/HMR需新合同，不通过PrecisionStop、减测试分母/隐藏反例绕过原验收。

