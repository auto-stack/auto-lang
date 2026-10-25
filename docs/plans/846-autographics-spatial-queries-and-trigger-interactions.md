---
plan_id: PLAN-846
status: reviewed
feature_name: AutoGraphics空间查询与触发交互原型
plan_revision: 3
author: [Codex]
created_at: 2026-10-25
updated_at: 2026-10-25
current_step: 10
total_steps: 11
depends_on: [PLAN-845]
supersedes_spec_components:
  - docs/specs/auto-val/project.md
  - docs/specs/auto-lang/vm/architecture.md
  - docs/specs/auto-graphics/behavior-api.md
  - docs/specs/auto-graphics/behavior-project.md
  - docs/specs/auto-graphics/scene-play-session.md
  - docs/specs/auto-graphics/auto-source-driver.md
  - docs/specs/auto-graphics/prototype-profile.md
  - docs/specs/auto-graphics/project.md
  - docs/specs/auto-graphics/integration-contract.md
new_spec_components:
  - docs/specs/auto-graphics/spatial-query-profile.md
touched_goals: [GOAL-007, GOAL-008]
affects: [auto-graphics, authoring-experimental, spatial-query]
---

# [PLAN-846] AutoGraphics 空间查询与触发交互原型

## 0. 变更摘要

r3（2026-10-25）用户明确调整AutoGraphics当前验收：增量编译只验功能正确性，不以延迟/P95/吞吐阻断交付；功能完成，尤其与真正ACE引擎合并后再定义性能优化验收。历史r2性能红保留，不改记性能达标。数值、支持范围、实际编译/缓存失效/链接运行/回收及运行时安全预算保持。

845把用户Auto行为、稳定actor绑定与GLB组成可保存/搬迁项目；下一步从“移动/显示节点”推进到“查询场景几何并产生交互”。本Plan交付**有限2D/3D AABB边界、点包含和成对重叠查询，以及两个由真实Auto私有状态实现enter/stay/exit的触发示例**。逻辑不写死在Rust样例名中，项目继续通过845绑定/保存/重开。

新增显式`autographics.behavior/2`与UserSpatialV2/profile8。旧behavior/1、profile1..7及其用户源码保持原路径；不能把新公共函数名直接加入v1 reserved集，拒绝以前合法的用户helper。scene/4结构保持，新增绑定API版本的可接受值；不另造scene/5或事件存储协议。

r2（2026-10-25）用户明确授权修复上游SDK NaN编码并更新固定版本，继续实施，保持原数值合同和支持范围。G845已正式交付10/10；T01完成，T02曾因native负NaN别名为0而needs_replan，原证据保留§9。新增T-02-SDK修复标量生产边界并固定最小Git对象，再继续T02及全部原验收。唯一实施树仍为 `D:/autostack/.wt/lang-846/auto-lang`。不重新申请已给授权，不削减任何旧/新分母。

## 1. 目标

1. 以实际已接受资源和静态局部变换生成有限AABB代理，将只读几何描述绑定到当次Run的节点/资源/版本；脚本不复制GLB顶点或Rust常量表。
2. 真实Auto中提供AABB min/max、point contains、pair overlaps；同callback先移动父组/叶子再查询立即看到最新world，不靠上一帧输出或GPU回读。
3. 行为v2显式可选，v1旧源码/私有helper、原类型/准入/错误/默认外部文件入口、845已存项目和原Play语义保持。
4. 两个不同维度的用户样例只通过公共API和Auto状态实现进入/停留/离开区域及可观察反馈；同runner/仓外/重开/搬迁可复跑，Stop恢复编辑状态。
5. 数值、身份、错误、查询/CPU预算、源码/VM/资源最后owner与GPU复用均有定向反例和原生最终像素证据。

非目标：精确ellipse/OBB/triangle碰撞、raycast/扫掠/连续碰撞、刚体/重力/碰撞响应/导航、动态增删节点或碰撞形状/层mask、新事件回调或事件队列、空间索引/全场pair矩阵、独立通用物理库、新UI组件/样式/Schema声明、ACE增量或热重载、生产AutoUI/RQHost/正式Forge/发行。新接口是有限代理查询，不宣称完整碰撞引擎。节点的scale/yaw/pitch/topology/resource仍只读；不迁移或改写已冻结Breakout/Squash的玩法/碰撞。

## 2. 架构方案

```text
845已接受Document + Registry/实际ModelData
  → 有界spatial描述：每资源local box、每叶子TRyRxS零平移后的8角包围盒
  → 带节点顺序/资源/静态字段/算法版本身份的Run快照
  → v2私有配置（最多128条，每条availability+6 floats）
  → actual Auto world translation + immutable box offsets
  → ag_aabb_min/max/contains/overlaps → Auto用户enter/stay/exit状态/可见性或位移
  → 原整帧准入 → 实际Surface/GPU；Pause/Step/Stop与编辑隔离保持

v1源 → 原v1/API/bridge/profile7，不注入新查询
v2源 → 共用v1能力的受控managed单元 + v2扩展/profile8
```

L1有限实验driver特性，r2额外授权上游SDK单点缺陷修复：实现主要写`experimental/autographics-auto-driver/`，仅NaN标量生产边界可写`crates/auto-val/src/nano_value.rs`及其必要定向回归。新增拟`src/authoring/spatial.rs`与`api/behavior_spatial_v2.at`，局部适配behavior/behavior_project/Play/Host/bin及测试/文档。AGP游戏/资产/renderer、其他生产crates、根manifest与三锁保持只读；SDK仅允许上述NaN编码纠正、最小Git artifact和来源清单/pin刷新，生成.sdk-source仍只从Git对象导出、绝不手改；不改核心AST/语言语义/公共ABI或迁移树。需改变上述写面/既有语义则回new，架构级扩面先按L2设计。

| 门 | 成立条件 / 当前状态 | owner_stage |
|---|---|---|
| G845 | 已核845正式交付10/10、canonical/ledger/归档/guard与到期收尾；来源entry见§9 | work T-01前 |
| GSOURCE | T-01核最终845完整SHA、SDK/三锁、v1 source/AST准入、项目版本/字节来源和旧分母；候选不等于实施冻结 | work T-01 |
| GSTART | 用户已启动work，r2明确授权SDK修复/pin更新，原数值合同及支持不变 | work启动前 |

唯一未来树`D:/autostack/.wt/lang-846/auto-lang`、分支`plan-846-dev`，显式基于最终845正式autographics-dev完整SHA，不能默认从master开树。work/internal/final review同树顺序；主检出记计划/附件/知识。图形代码只落autographics-dev；上游纯标量修复单独提交，T10最终复审/根门禁通过后可仅以该提交diff落master，图形文件绝不夹带；master图形合入门、I2/I3保持。无junction/symlink，移树前wt-guard。

## 3. 技术栈

复用最终845的Rust/stock Iced/wgpu、公开语言SDK/Parser/Session、Arc资源、v1行为准入/来源映射、scene4/Storage、实际Auto callback与consumer。候选产品源码`e518f5fc778cfa6827f7de9eb99b4c0910ef294d`，845测试定位修复tip`a0e2a9115a0b48bab44e3acaef17485dca472420`；规划时观察集成`8b62db5cba54b9bfe7f759f4920b22ccb9b75e88`、SDK`932ad4877e5968f1aa7227d9dcd7fb69b4c69a21`；T-01现已核定正式基面`c1c7cfea5c9cc84987091ddeaf6cde59cefd5d4f`，r2 SDK固定为`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`，详见§9。

AABB是轻量数学查询，先复用现有`interaction::bounds/transform`的几何意义及原glam，不加物理依赖。源码公共API/签名/版本描述单源；v2共用v1实现单元，不长期维护两份图派生或错误逻辑。复用必须保持v1原managed源码身份/行为与允许的用户符号集合，具体装配由T-02取证。

## 4. 需求分析与背景调查

### 4.1 授权、前置与知识

原规划授权见r1历史。用户随后启动846 work，并在2026-10-25明确要求“允许修复上游SDK的NaN编码问题并更新固定版本，保持原数值合同、不缩减支持范围，然后继续实施”。本轮据此修订和继续work，不创建agent/chat/模型角色/预算；不代办无关计划。客户端日期本轮跨至2026-10-25，按此记录新草案日期。

已读[全局overview](../specs/overview.md)、[图形overview](../specs/auto-graphics/overview.md)、[v1行为API](../specs/auto-graphics/behavior-api.md)、[运行会话](../specs/auto-graphics/scene-play-session.md)、[静态资源](../specs/auto-graphics/scene-assets.md)、[视口几何](../specs/auto-graphics/viewport-interaction.md)、[集成边界](../specs/auto-graphics/integration-contract.md)、[845](845-autographics-bound-behavior-and-portable-projects.md)、[其独立R2](attachments/845/review-r2/review.md)、[受影响独立R3](attachments/845/review-r3-gate/review.md)及[merge checkpoint](attachments/845/merge-r1/receipt.md)。behavior-project.md仍待845沉淀，先交付再作为modify基面；不把845树内proposal当canonical。

26个固定Git对象、19份文档输入hash、观察ref/状态与来源事实见[清单](attachments/846/planning/source-observation.json.txt)。831 execution_done9/12、832/841仍drafting；生产组件/Schema、ACE接线/热重载继续[Design45 §11](../design/45-autographics-wgpu-engine-and-game-prototypes.md#11-目标态schema-统一-ui2d-与-3d-声明语义2026-10-24)后继范围，本Plan不依赖这些在途接口，也不宣称绕过其生产门。

### 4.2 实际触点与缺口

| 触点 | 已观察事实 | 本Plan对应 |
|---|---|---|
| interaction.rs::bounds/transform | bounds遍历实际ModelData所有mesh.positions；transform为T*Ry*Rx*S，pick3使用变换后local box的OBB | 新查询明确为该全模型local box变换后的world AABB；不改原精确2D/OBB picking |
| behavior.rs::admit/private | `_ag_`私有；reserved来自所选managed根声明；未定义的ag_aabb_*原可作用户helper | 显式v2而非往v1添加保留名；保留旧合法源对照 |
| behavior.rs::SourceSnapshot/ENTRY | v1/version1/profile7/managed hash与原始用户字节绑定，base_scene附文档/资源/actor/stamp | 版本化构造/诊断/身份，spatial描述只能从接受快照形成，不能由用户拼transport |
| play.rs::new_user/new_program | 重建SourceSnapshot拒绝漂移，完整图/资源及fields配置后才start；一VM/整帧准入 | v2先完整配置图和spatial元数据再app_start，profile8显式consumer；v1/Builtin6无新配置 |
| host.rs::models/start_program | Host持有实际builtin Cylinder/Player/Mob的ModelData，Editor Registry只持有导入资产；原Play入口只收Editor/actors/source | v2描述必须借用Host原models与接受Registry，再走具名受检构造；不能在Play复制builtin模型或重读原GLB |
| behavior_project.rs::Binding/CodePayload | Binding.api当前只许VERSION1，CodePayload编译验证/保raw bytes，source身份按SHA | scene4可显式选择v1/v2；raw共享而准入/编译身份按API区分，不能错用旧验证 |
| 私有游戏collision | Breakout与Squash已有各自冻结游戏算法，不是公共场景查询API | 保原样例分母，不拿旧游戏pass声称任意节点碰撞支持 |

## 5. 详细设计

### 5.1 下一入口与有界whole-path可行性

T01已完成；当前下一任务为T-02-SDK，然后重放T02受影响数值及whole-path。T-01核845完整收尾与实际API/项目/来源收据、唯一树和写面；上游若修复改变假设先重核，不追逐在途WIP。

T-02新增`research/spatial-query-feasibility.md`：在同树用正式v1项目和最小v2装配，证明两个版本可并存；实际ModelData→spatial描述→Auto callback先改group后查AABB→触发另一个节点变化→全mesh GPU→Stop/保存重开。对照v1合法同名helper、错API/profile/描述身份、边界接触及不可用几何、32次/33次查询、错误后恢复。验证128节点两种合法结构（大量叶子、深层group+叶子）下原初始化/CPU预算。

最多比较“v1原单元复用+v2显式扩展”和“同一受控模板按版本生成”的两种局部装配；不改用户源码；SDK仅T-02-SDK授权的上游标量修复，不写第二parser/私有native执行器、不另造通用插件框架。v1语义/源码身份、类型化版本选择或支持预算无法保持，回new明确证据，后续任务不启动。

### 5.1.1 r2 上游SDK NaN编码与固定版本

producer为`auto-val::encode_f64`（native math/VM f64写槽共用）→NaN-box NanoValue→`VirtualRAM::pop_f32`/公开Session/实际Auto查询。负NaN不能落入负NaN-box的整数/对象等tag空间。仅对NaN编码规范化到非tag的正NaN空间，保留NaN数值语义；有限值、±0、±Inf及既有非浮点tag位型保持。NaN符号/载荷非公共数值承诺，不改变函数ABI或把非有限值折0。上游修复后原`v - v != 0.0`判定必须在真实callback拒绝native NaN；禁止改用户源/禁止sqrt/改预算/把false算错误。

在同一worktree编辑真正上游文件，单独提交纯SDK修复。固定artifact以原SDK932ad完整Git树为父，仅应用已验证的标量文件差异；不得从当前graphics/master导出夹入无关WIP。新的artifact commit、tree、父、路径diff、文件SHA、17包/29边/SDK681输入清单和三锁原字节都要记录。生成SDK先保留旧目录，再从新Git对象fresh prepare；禁止手改.sdk-source。更新baseline/source-lock及所有当前验证/consumer来源入口，旧845清单保留为历史，新846清单成为明确当前入口，过期pin/省略新文件负控必须拒绝。

验证V13：直接encode/decode覆盖两符号/quiet-signaling/跨tag载荷NaN、有限/±0/±Inf/非浮点tag；真实native sqrt(-1)、f32 NaN、有限sqrt0/4通过参数/局部/返回/容器可适用路径，最终Auto API不命中且错误7。原frozen反例在新SDK上转绿，恢复旧编码必须红；不能仅单测往返声称consumer闭合。原v1用户helper/常量准入/行为与128预算均复验受影响路径。work仅相关counterexample/module和必要check；根完整`t`+`tv`归T10，绝不在修复提交时提前跑；tf仍主检出单实例到期批量。

### 5.2 精度与几何代理合同

查询对象仅为已有2D/3D叶子；groups本身不参与，用户可遍历其叶子。2D Circle/Quad的代理是世界轴对齐矩形，局部范围`[-scale.x/2,+scale.x/2]×[-scale.y/2,+scale.y/2]`，z固定0；Circle不是椭圆精确测试。3D Cylinder/Player/Mob/Model先取**实际完整资源所有mesh顶点联合local box**，再对其8角应用静态Ry*Rx*S，取分量min/max为相对节点原点的box offsets；加实际Auto world translation得到world AABB。包含空白区域、凹模型空洞及旋转后的额外空间，均属于显式代理语义。

父组只有平移/可见性；不新增祖先旋转缩放或相机变换。查询坐标使用场景world，不含画布屏幕中心、DPI或投影。渲染/交互原路径不改，AABB也不替代原pick2椭圆/矩形和pick3 OBB。

min/max是有限有序f32端点；contains及overlaps都用**闭区间**（边/角/面接触算命中），不添加epsilon扩张；退化但有限的3D平面/线/点box仍按闭区间处理。overlaps要求两叶子同维，2D只比较x/y；自身与自身为1。合法不相交返回0。可见性/alpha与几何相交独立：隐藏节点仍可查询/重叠，示例是否忽略隐藏目标由用户显式ag_visible判断，不暗设物理激活规则。

host派生offset算术、Auto world加法及独立参考的计算顺序在T-02/T-04记录；proxy端点与实际变换顶点包络/画面对应允许沿原位置检验的`max(1e-3,8*f32EPSILON*max(1,abs(expected),abs(actual)))`浮点误差。**该比较容差不能用于把几何查询的0/1判定放宽**；精确可表示的接触/分离及±1ULP控制必须区分。无需GPU像素逐点等于box，因为代理本就含空白。

旧合法场景继续可Run。若资源无可用有限local box或变换结果溢出，只给该叶子标记unavailable，不能使未调用新API的旧路径新增失败；v2请求该叶子查询时明确GeometryUnavailable并拒绝整callback，无虚构零box。原合法有限范围正控必须完整支持，不能用unavailable隐藏漏mesh/算法错误。T-02确认当前受限资产族实际支持边界；若出现新的合法族无法支持则回new，不删旧场景分母。

### 5.3 behavior/2 公共API、错误与预算

v2保留v1所有既有能力及签名，新增以下拟议名称（等价命名可在T-02取证/T-03冻结，不可删能力）：

| API | 返回 | 语义与错误 |
|---|---|---|
| ag_aabb_min(node,axis) / ag_aabb_max(node,axis) | float | world端点；axis0/1/2，2D z合法且返回0 |
| ag_aabb_contains(node,x,y,z) | int0/1 | world点；坐标须有限，2D z须0，否则参数错误 |
| ag_aabb_overlaps(a,b) | int0/1 | 同维叶子AABB闭区间相交；跨维不是合法false，而是明确SpaceMismatch |

node为当次Run ordinal，0合法；无效handle（含−1）、group、错axis/非有限参数、跨维、GeometryUnavailable分别具名错误。沿v1 sticky-first错误，不在索引前使用未检节点，不允许后续合法调用清除已有错误；安全占位结果不代表成功。新错误纳入现有source/stage/node/argument诊断，整callback拒绝、关闭污染VM、保last-good；修源/Stop/新Run可恢复。

新几何操作的实际world派生、端点相加/contains/overlap和触发业务逻辑由Auto执行。host只为**静态已接受几何**提供有限描述及校验，不在Rust按样例名/用户source哈希计算命中或驱动门/收集物。允许Auto dirty派生缓存；同callback写父组/叶子后任何新查询立即见最新world，不能用旧frame/旧world。

每个app_start/app_update最多**32次新公共查询尝试**，每个公共入口计一次，内部helper不重复计费；第33次在访问几何前记QueryBudget错误，整callback失败。计数只在callback边界清零，不能每getter/derive/Step重置。原CPU100000steps/120resumes/250ms、1inflight/四tick额度/8ms共享pump、初始化30s/整tick2s、dt1/60及4096摘要仍覆盖API/派生/用户代码的总成本，不另给一份预算。

合法128节点正控含32次混合查询（8pair overlaps及24次bounds/contains，最多两次group先写后查）必须在原预算通过；宽/深两类结构都验。没有自动O(N²)配对/事件队列，调用次数不等于无限吞吐承诺；失败须回new研究算法或明确支持变更，不能抬deadline/少选节点过门。

### 5.4 Run几何快照与资源归属

新增spatial模块以Document顺序/稳定ID→Run ordinal、kind/dimension、resource原始SHA/内置身份及静态scale/yaw/pitch构建描述，和accepted场景/Registry/actor/API/算法版本共同形成身份。实际producer包括Host原models的builtin资源和Editor Registry的导入ModelData；Host借用它们构造受检描述，v2经具名构造端口消费，旧v1 Play公开入口保持。不能在Play内复制builtin生成器/重读GLB拼另一套模型。

只从实际完整ModelData借用顶点求界；按资源身份缓存local box，一次Run内每种资源最多扫描一次，不clone顶点数组；每节点8角只做一次。后续查询、warm tick、父组移动不重扫/重解析/上传模型。

描述最多128条，逻辑内容availability+6float共≤7标量/节点（3584B原值）；队列/身份编码有界，额外配置调用数/耗时由T-02/T-08如实记录，不把JSON/容器开销算成原值字节。groups无可查询box但仍保持同一ordinal映射。所有描述配置完成后才能app_start；Starting取消、Stop、初始化Fault及错序/缺条/重复配置均不能发布半初始化帧。

v2快照含geometry descriptor hash及base场景/资源/静态字段/算法版本；new_user重核来源及实际base，不能接受foreign/stale描述、仅同名资源或漏掉第二mesh的伪描述。host始终从当前接受内容重建/核验，用户源码不可调用私有配置符号。现有immutable字段/full-frame world/resource检查仍逐项保留，profile8不是放宽写面。

描述是Run内数据，不写scene/history/源码blob/每帧报告；新Run更换资源/Undo/绑定/Save As再Open都按新接受内容重建，不能复用旧ordinal/旧resource的box。只持有必要原资源Arc，不新增GPU target/pipeline或逐tick readback。实际弱owner/mesh Arc/写入计数区分正常Editor/历史/外部pin与泄漏。

### 5.5 显式版本选择与845项目兼容

保留v1 API/bridge/ENTRY/profile7与其声明reserved集合；新增`behavior_spatial_v2.at`，复用v1逻辑单源后加v2私有数组/配置/查询及version2/profile8。不将ag_aabb_*加入v1解析/保留名/全局注册；v1用户原来合法定义同名helper的程序仍能原样编译执行，v2则按其公共声明拒绝重定义。原Builtin6/profiles1..5完全不增加配置或dispatch。

SourceSnapshot增加具名版本选择并绑定对应API/bridge/combined hash、region与base身份；默认现有from_bytes/read入口仍为v1，v2使用明确类型/参数，不能猜source字符串或payload形状。版本化构造/运行consumer拒绝错版本/hash/profile/非法声明，source区域映射/常量初始化/依赖扫描等840合法控制继续通过；不修改用户identifier/literal，不复制parser。

正式UI在Editing选择临时脚本或项目绑定时可明确选v1/v2；CLI拟`--behavior-api v2`配合原`--behavior`，缺省保持v1。`--open`有绑定时用Binding.api；CLI显式版本只适用于临时源，孤立版本参数应报用法错误，不覆盖项目。活动模式修改版本/绑定拒绝且不排队。

scene/4字段结构不变，Binding.api接受精确behavior/1或behavior/2；旧绑定不自动升级，未知值受控拒绝。旧reader不识别v2时明确unsupported API，不能按v1运行或丢binding。Attach/Update/改API仍走同一接受/history/dirty事务；同字节切换API也不是noop，Undo/Redo恢复正确版本/源码准入。v1↔v2切换失败保原binding/源码/dirty，不静默改saved hash。

原始CodeRef/文件路径仍按用户raw SHA、8身份/2MiB quota；同字节在v1/v2下可共享raw Arc，但语义准入/编译凭证必须按API+managed身份区分，不因raw hash相同复用错误版本结论。CodePayload/Registry/History/Storage收据保持真实owner；可采用共raw payload+每次具名准入或typed view共用raw两种局部方式，由T-02决定，不能另设隐藏compiled-cache或保留多VM绕过预算。

Save/Open/Verify/Save As继续校验完整GLB+code闭包和版本，原始代码字节/来源身份、immutable blob、临时override、API诊断和old epoch规则保留。Save与重开不执行app_start/update，模型不新增碰撞参数/动态形状schema。

### 5.6 两个触发示例与原生验收

新增拟`samples/spatial_projects/zone2d/`和`zone3d/`，均为845可绑定/保存的项目、明确behavior/2；用户源只写公共API和私有previous/current计数/状态，无transport或硬编码Rust结果。

- Zone2D：输入驱动actor进入/停留/离开矩形区域；私有Auto状态区分enter/stay/exit，控制已有提示节点可见性/位置，并展示相同circle代理矩形的边角命中语义。是否忽略隐藏触发对象由脚本显式判断。
- Zone3D：实际导入多part GLB、非零父组、非均匀scale及yaw/pitch；用world box重叠实现触发区开门/指示。至少一次callback写group后立刻查询并改变另一节点；不会仅换2D样例维度常量。

事件是用户对相邻完整tick查询结果的比较，不新增on_enter等引擎回调/注册表。app_start初始inside只设基线与提示，不伪造enter；Step每次只产生一次状态更新，Pause无重复事件，Resume不重新start；Stop/新Run清除用户全局状态。提示节点/门仍由Auto local/visible写入原全帧consumer；不实现碰撞解算/阻挡。

同runner SHA复制两个项目到仓外/不同cwd，删原root/导入path后新进程只--open、明确Run；记录outside→enter→stay→exit及边界接触的独立轨迹、事件反馈状态和全部相关GLB mesh最终像素。只改用户条件/区域选择→Update/Save/新进程重开能改变真实行为，runner/Rust/bridge无重编译。v1 Move/Patrol及845临时覆盖、持久绑定/Undo/来源控制并列保留。

### 规范增量

r3增量：SD-06见[冻结提案](attachments/846/revision-r3/spec-delta.md)，只调整增量编译性能验收归属，不改变功能/支持。

全部拟议，当前不写canonical；behavior-project.md先由845完整交付再modify。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-06 | modify | docs/specs/auto-graphics/integration-contract.md | 当前增量速度/P95为阻断→AutoGraphics仅验正确增量编译，耗时观测；功能完成及真实ACE整合后再定性能标准 | 用户本轮明确授权暂缓性能验收 | AC-08, AC-09, AC-10 |
| SD-05 | modify | docs/specs/auto-val/project.md; docs/specs/auto-lang/vm/architecture.md | f64直存有负NaN tag碰撞→NaN编码不占tag空间，有限/±0/±Inf/tag位型不变；SDK pin具名更新 | 保留非有限拒绝合同、修producer | AC-04, AC-05, AC-08 |
| SD-01 | add | docs/specs/auto-graphics/spatial-query-profile.md | 无公共空间查询→有限AABB代理/坐标/闭区间/错误/32次预算/实际Auto与资源来源 | 几何查询有可验语义 | AC-02, AC-04, AC-05, AC-07 |
| SD-02 | modify | docs/specs/auto-graphics/behavior-api.md; docs/specs/auto-graphics/behavior-project.md | 仅v1→v1原样并存v2/profile8、显式选择与API化source准入/绑定/历史/闭包 | 防保留名/同hash跨版本错误复用 | AC-03, AC-06, AC-08 |
| SD-03 | modify | docs/specs/auto-graphics/scene-play-session.md | 仅既有图配置→v2有限spatial描述初始化/身份/瞬时world查询，整帧/CPU/Stop合同保持 | 不冒充Rust玩法或旧帧查询 | AC-04, AC-05, AC-07 |
| SD-04 | modify | docs/specs/auto-graphics/auto-source-driver.md; docs/specs/auto-graphics/prototype-profile.md; docs/specs/auto-graphics/project.md; docs/specs/auto-graphics/integration-contract.md | 移动/显示与可搬迁项目→2D/3D用户触发示例/支持与局限，I2/I3/Schema/ACE仍后继 | 交付含义准确且原分母保留 | AC-01, AC-06, AC-08, AC-10 |

## 6. 测试设计

r1可行性原型与反例详§9。r2先执行V13/T02-SDK，再继续T02；既有通过证据仅在代码/依赖/config/输入等价时复用，不将旧SDK全绿转记新SDK。用户未指定实施/internal/final模型/创建agent权限；实施/internal/final同树顺序，final独立上下文并暂停实现写入，作者自查不能冒充最终独立验收。

未来846树driver目录沿现有prepare/verify核固定依赖。拟新`tests/spatial_geometry.rs`、`spatial_api.rs`、`spatial_host.rs`、`spatial_project.rs`及`tools/check_spatial_queries.py`、`check_spatial_owners.py`，由T-03..08建立。开发用`cargo test --locked --test <对应单一target> -- --nocapture --test-threads=1`、实际受影响原用例、locked all-targets check/fmt；0selected/skip不得算pass。

| case_id | AC / work owner | 输入/实际入口 | 可区分的observable oracle | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01；T-01/08 | 正式845/SDK/三锁/API族/源闭包及独立consumer、原source反控 | 正式版本/来源和原分母不漂移，新增API/源码归入fence，缺件/漂移拒绝 | 候选不是交付冻结 |
| V-02 | AC-02；T-04 | 真实2D Circle/Quad、3D全mesh/偏心/重复mesh名字/scale/yaw/pitch、深组及更换资源 | 用手审输入/独立8角参考求proxy；全mesh顶点在容差内包围，遗漏mesh/错变换/旧资源负控红；原picker不变 | AABB代理非triangle/ellipse/OBB精确碰撞 |
| V-03 | AC-03；T-03 | v1含合法ag_aabb_*私有helper、v2合法新API/重定义、错API/profile/hash，原常量/注释/作用域控制 | 实际Parser/SourceSnapshot/Session两版本各自合法/拒绝，v1集合与managed身份原样；v2错误不回落 | 不复制parser、不默升v1 |
| V-04 | AC-04；T-05 | actual Auto在同callback移动parent/leaf后query；轴0..2、2D world z、非零group和隐藏/alpha0 | 端点/contains/overlap反映最新Auto world，与独立参考/后续Auto输出一致；故意读旧帧被抓 | 查询不依赖屏幕/DPI，不自动忽略隐藏节点 |
| V-05 | AC-04, AC-05；T-05 | 精确接触/±1ULP间隙/点线面退化、自身、不同维、group/−1/越界axis/NaN、unavailable后合法调用 | 闭区间1/0准确；非法区分API错误且sticky-first、整callback拒绝保last-good，安全占位不冒成功；修源新Run恢复 | 容差不放宽bool判定 |
| V-06 | AC-05；T-02/05 | 128节点宽/深结构、32混合请求与2次group写；第33次、callback超步/超时、Step/Pause/新Run | 原100k/250ms等总预算内合法控制过；33受控QueryBudget，边界重置一次；超时错误与命中false分开 | 无自动全pair/无限吞吐保证 |
| V-07 | AC-06；T-03/06 | 相同raw v1/v2切换/Undo/Redo、Attach失败、scene4旧项目、孤立CLI版本、临时覆盖与SaveAs/Verify | 精确Binding.api/实际编译版本，raw可共但凭证不串；旧合法helper仍运行，旧saved hash/history/来源按原合同，坏版本不clean/不fallback | 不新scene5/文件格式 |
| V-08 | AC-04, AC-07；T-04/05 | foreign/stale描述、错resource/静态字段/算法、漏/重/错序配置、Starting取消/Stop | 同一base身份、全图+spatial配置先于app_start；部分初始化不发布，取消仍tick0 Paused，初始失败恢复编辑 | 不新增写权限 |
| V-09 | AC-06, AC-08；T-06/07 | 两Auto触发项目outside/enter/stay/exit/新Run、暂停/单步/恢复、用户条件变异 | 私有状态由真实Auto执行，事件反馈可由场景状态/独立轨迹核对；无Rust样例preset/重复事件 | 触发事件是用户逻辑，不是引擎队列 |
| V-10 | AC-08；T-07 | 正式OS→Iced/API版本/绑定/Run，仓外同binary两项目/原目录不可用，GLB最终像素/GPU故障/partial | 原生真实入口、所有相关part与指示状态正确；改源改变结果无Rust重编，native/aggregate不完整非零 | 真实GPU反例不能以setup失败充数 |
| V-11 | AC-07；T-04/08 | 实际20轮v1/v2/取消/Fault/绑定/Undo/Open/Stop、资源pin、重复geometry扫描/重复上传负控 | 源/VM/spatial weak owner最后释放；同资源local box每Run最多扫一次，warm不扫/不上传，不新target/pipeline/每帧读文件 | 外部合法pin及Iced/驱动计量排除项单列 |
| V-13 | AC-04/05/08；T-02-SDK | 上游encode_f64→native sqrt/参数/局部/返回/可适用容器→Auto查询、旧编码mutation和实际SDK导出/消费 | NaN不别名tag/0；真实nativeNaN错误7、无命中；有限/±0/±Inf/既有tag不漂移，source/pin单路径diff可复核 | 不改ABI/预算/旧语法，不拒绝原支持函数 |
| V-12 | AC-08；T-08 | API表/CLI/源映射/精度预算/SD01..04/准确merge矩阵 | 支持/错误/版本/样例/限制可复跑，与源码和case一一对应；原v1/845/game/合成分母清楚 | 文档静态检查不代实现验收 |

每case保存code/SDK/锁/用户源码/API/bridge/geometry算法/fixture身份、精确命令/退出码/断言数、原始报告与关键像素。独立参考不调用被测spatial helper或读取其输出生成expected。反控至少覆盖漏mesh/旧world、v1误保留名、同raw跨API错误复用、跳过query quota、实际结果/最终渲染破坏与owner/扫描/上传退化。反例在宣称路径实际失败，原同binary控制与复原绿并列。

r3根门策略：显式设置`AUTO_LANG_AUTOGRAPHICS_INCREMENTAL_PERF=observe`，MN正文编辑/签名重建/冲突/实际增量序列保留全部原功能断言及30样本，耗时仅观测；裸根t/tv仍完整执行，不排除测试、不增设ignore、不升高执行deadline。AutoGraphics原生/owner/GPU完整门保留。

最终review前work只定向/check/fmt；T-08登记准确旧/新完整矩阵。**T-09最终独立pass后**T-10才在冻结提交跑完整locked driver/AGP test+build、原16 AGP/双游戏、835..845相关原生/owner/reopen/source及独立own-SDK/own-target consumer、原counter AND content两道门和新spatial正反/项目搬迁/预算。证据复用须证明代码/依赖/配置/输入未变，不能只因旧Plan总绿略过新v2入口。r2含上游标量Rust修复，T09独立pass后T10须根`t`+`tv`（实际依赖组/可构建快照）及完整driver/AGP/consumer门；工作阶段只定向auto-val/实际native与API反例/check。无trans/book/UI生成/aavm触面则不跑相应档，docs_gen不触发；AGP/其他生产扩面仍回new。tf仍主检出单实例批量，846收尾做实际到期判定，不自动在worktree并跑。

## 7. 验收标准

| ID | owner_stage | 可观察标准 |
|---|---|---|
| AC-01 | work | G845/GSOURCE、唯一树/依赖/source/原分母与只读边界成立，V-01 |
| AC-02 | work | 原实际全模型及2D几何/变换→有限代理正确，不改原picking，V-02 |
| AC-03 | work | 显式v1/v2/API/profile/源码准入隔离，旧合法源不回退，V-03 |
| AC-04 | work | actual Auto即时bounds/contains/overlap、坐标/精度/描述身份正确，V-04/05/08 |
| AC-05 | work | sticky错误/32次与原CPU预算、初始化/故障/Step/Stop恢复闭合，V-05/06/08 |
| AC-06 | work | 正式UI/CLI与Binding.api/同raw版本/历史/保存/来源，两个真实Auto触发示例可运行，V-07/09 |
| AC-07 | work | 20轮source/VM/spatial/资源最后owner与扫描/上传/targets复用，V-08/11 |
| AC-08 | work | 仓外同binary/OS/GPU正反/partial及精度/预算/SD/API/README交接齐备，V-09/10/12 |
| AC-09 | review | 独立核AC-01..08/Spec、遗漏/延后/workaround/健康，revision/code绑定pass，完整merge门明确pending |
| AC-10 | merge | 冻结完整门通过才落autographics-dev；canonical/ledger/归档/guard清理及到期收尾齐，graphics源码不入master |

## 8. 执行步骤

当前T-01、T-02-SDK、T-02..09完成，r3受影响复审与完整功能门已通过，reviewed10/11；T10落地/规范/账本/归档/清理收尾pending。T-01..08关闭且无必需gap才execution_done9/11；T-09最终独立pass为reviewed10/11，完整merge门pending；T-10实际收尾才交付11/11。

| task_id | owner_stage | 依赖 | 实际触点／拟新增路径与动作 | 关闭条件；AC |
|---|---|---|---|---|
| T-01 [x] | work | G845/GSTART | 845正式收据/Spec、唯一846树、SDK/三锁/verify_source与所有原分母 | 受影响review/合入/清理齐、正式完整SHA与来源可消费，重新冻结；AC-01 |
| T-02-SDK [x] | work | T-01 | 上游auto-val/nano_value.rs NaN生产边界、定向回归、最小SDK artifact、新pin/source-lock/current verify入口 | V13原NaN反例转绿/旧编码mutation红，有限/±0/±Inf/tag对照、真实callback/SDK来源齐；AC04/05/08 |
| T-02 [x] | work | T-02-SDK | 正式behavior/Play/项目接口、interaction::bounds/transform；新research/spatial-query-feasibility.md | §5.1 whole-path、v1 helper/版本/预算/三维GLB像素/Stop与失败控制通过，最多两方案；否则回new；AC-03, AC-04, AC-05 |
| T-03 [x] | work | T-02 | behavior.rs/behavior_project.rs版本化构造/准入/身份/绑定、CLI/UI版本数据通路；新spatial_api/project tests | V-03/07版本/source/同raw角色/历史隔离定向过，v1源码身份/合法集合不变；AC-03, AC-06 |
| T-04 [x] | work | T-03 | 新authoring/spatial.rs、原interaction几何复用和实际资源只读借用；新spatial_geometry tests | V-02/08元数据与V-11扫描/资源部分过，全mesh/8角/有限状态/身份正确；AC-02, AC-04, AC-07 |
| T-05 [x] | work | T-04 | 新api/behavior_spatial_v2.at、Play profile8/配置/consumer/错误预算 | V-04/05/06/08实际Auto查询、同callback更新、32/33/原预算与Starting/Stop通过；AC-04, AC-05 |
| T-06 [x] | work | T-05 | Host/Panel/bin版本选择闭合、samples/spatial_projects/zone2d、zone3d | V-07/09正式默认/临时源与项目/源定位，用户Auto触发语义和原v1/845定向控制过；AC-06 |
| T-07 [x] | work | T-06 | 新check_spatial_queries.py/正式OS路径/真实GPU故障及新进程项目回放 | V-09/10两项目仓外同binary/搬迁/条件变异/partial/Stop画面完整，非环境gap；AC-08 |
| T-08 [x] | work | T-07 | 新check_spatial_owners.py、source/fresh consumer/README、新research/spatial-query-handoff.md/Spec proposal | V-01/11/12、20轮owner/扫描上传、全部work定向/check/fmt与准确完整merge矩阵齐；AC-01, AC-07, AC-08 |
| T-09 [x] | review | T-08 | 同树冻结code/Plan/Spec及原始两项目报告，主检出独立review附件 | 逐AC-01..08/SD/遗漏/延后/workaround/健康绑定结论；AC-09 |
| T-10 | merge | T-09 pass | 冻结完整门→autographics-dev；主检出Spec/ledger/archive/guard/到期 | AC-10实际落地/清理，改代码先受影响复审再新冻完整门，不提前宣称交付 |

## 9. 复审记录

2026-10-25：stage:new；PLAN-846 revision1；outcome:blocked（G845完整交付pending，草案可评审）；next:845已完成测试定位修复的独立R3，继续其新冻结完整门与合入收尾；后续用户启动846 work，T-01重核正式基面，再T-02。当前drafting0/10；所有T/AC/V/SD是拟议合同，未实施/建树/prepare/build/Cargo/原生/GPU验证，未创建agent/chat。

静态检查见[preflight](attachments/846/planning/preflight.md)。本轮未改845或841/840/迁移/ACE/Schema合同；后续生产目标继续已有设计。若上游合入期间状态改变，T-01消费最新正式收据，观察清单不冒认最新执行冻结。复审/完整门仍由各阶段负责，重复同因先诊断漏测/兄弟路径，不减分母/隐藏支持或调fixture把红当绿。

### 2026-10-25 work:r1 — needs_replan / executing1/10

stage: work | plan_id: PLAN-846 | plan_revision:1 | outcome:needs_replan | worktree_path:`D:/autostack/.wt/lang-846/auto-lang` | branch:`plan-846-dev` | base_commit:`c1c7cfea5c9cc84987091ddeaf6cde59cefd5d4f` | code_commit:`118530ee5fa14959535a0cdb48d6d0cc0c6aaab5` | dependency_revisions:SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21；AGP/三锁未变 | task_ids:T-01完成、T-02未关闭 | evidence:[来源冻结](attachments/846/work-r1/entry.json.txt)、[当前结果/哈希](attachments/846/work-r1/readiness.json.txt)、[冻结NaN反例](attachments/846/work-r1/spatial-frozen-nan-gate.txt)、[实际源码](attachments/846/work-r1/actual-auto-sources/) | blockers:F-846-T02-NAN | next_responsible_role:new，有限数值支持/SDK边界修订后回同树work。

G845/GSTART/GSOURCE已成立，845最终交付/清理/到期收尾可消费；来源681SDK文件/479依赖/17本地包/29路径边/845闭包321hash已实核，master代码无WIP。T01按实际交付完成，不再被草案时期的G845状态阻塞。

T02原型证明显式v2/profile8与v1 helper并存，128节点宽/深结构32混合请求+2次父组写后查在原预算通过、第33次QueryBudget；真实Host/Registry多mesh GLB→Auto即时查询→另一节点→实际两part GPU像素、scene4绑定保存/重开、Stop像素恢复与VM owner释放均有定向通过。`cargo check --locked --all-targets`、fmt/diff检查通过，固定SDK原146warnings保留。以上是机制/定向证据，非T03..08验收完成；旧分母及完整矩阵不因此被免除。

| finding → invariant | affected paths/states | actual evidence / remaining gap |
|---|---|---|
| F-846-T02-NAN → §5.3/AC04/05/V05非有限坐标须具名错误，不能成功命中 | native math sqrt返回f64负NaN→公开float参数→Auto contains→业务分支；min/max端点数值及其他复用路径后续同因审查 | `sqrt(-1)`实际raw=`fff8000000000000`；两种managed guard（`v - v != 0.0`、`v != v`）都status0/hit1，合同要求status7/无成功命中；每种的正常0、native0、native有限sqrt4和真正f32NaN控制成立。最终冻结测试exit101，1failed/0ignored/3filtered；不称部分pass。 |

已定位SDK `auto-val/nano_value.rs::encode_f64/is_f64` 与 `vm/virt_memory.rs::pop_f32`：负NaN位形被视作nanbox非f64，fallback低32位解为0。原生producer独立探针与实际API分支同证；不把所有潜在局部算法宣称数学上不可能，但有界候选未建立符合合同的局部方案。不能擅改只读SDK、重写用户表达式、拒绝原允许数值helper或把NaN报false来过门。建议new明确上游标量tag修复/独立SDK交付pin，或由用户决定数值支持合同变更，再重开受影响T02验收；仍沿用本worktree。

先前探针失效尝试完整保存：main后声明未初始化、返回值直接累加的浮点推断、科学计数literal解析及除零在到达NaN前失败，均未计作本finding；正常对照及两个真实反例独立列明。可行性原型已以research提交冻结，WT clean，不是可交付v2实现。T02未完、T03..08未启动验收，current_step=1/10，保持executing；本轮没有final review、合入、canonical/ledger沉淀、归档/清理或root整档测试。完整统一门仍review通过后由merge负责，当前不具备送final review条件。

### 2026-10-25 new:r2 — pass / next work

stage:new | plan_id:PLAN-846 | plan_revision:2 | outcome:pass | authorization:用户明确允许修上游SDK NaN编码及更新pin、保持原数值合同与支持 | changed:T-02-SDK（新增）、T02依赖、V13、SD05、SDK来源边界及T10根门 | preserved:T01实际完成、全部原AC/V/分母/预算/旧v1语义、r1反例与提交118530ee5 | next:work，同树顺序；当前executing1/11。无需重复批准。本次仅修订合同，代码修复归紧接的work。

### 2026-10-25 work:r2 进展 — SDK修复、T02与T03闭合

用户授权已消费，原数值合同/支持/预算未变。上游纯修复提交`4cef80c1b`，SDK artifact `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`（父932ad、仅nano_value.rs差异），全部17本地包/681输入/479依赖/29边与三锁原字节保持。[artifact](attachments/846/work-r2/sdk-artifact-final.json.txt)。NaN编码规范化符号位；相同异常空间还覆盖raw -Inf的tag识别，已将`is_nanboxed`排除零尾数负Inf、位型不变，避免其也折0。

T-02-SDK [x]：纯模块真实源码直接rustc测试17/17（SDK workspace含既有nested-workspace，故其Cargo单测入口失败后保留raw，并用无crate依赖的原模块直接编译等价验证）；[旧编码实际mutation](attachments/846/work-r2/sdk-old-encoding-mutation.txt)同测试1red、原文件未改。新SDK原NaN双guard/8控制+2反例全部正确；参数/局部/返回/容器及±Inf相关API错误保持。[replay](attachments/846/work-r2/spatial-r2-sdk-replay-final.txt)、[扩展路径](attachments/846/work-r2/spatial-nan-paths-r2.txt)。原严格Session.float对native f64 NaN输出报positive-NaN type mismatch（非错误tag或成功零值），API内部仍必须错误7，测试未放宽查询合同。来源当前入口新增846版本，T08继续最终冻结与独立consumer。

T-02 [x]：上述SDK上重放spatial_feasibility3/3，含宽/深128节点32请求两父写、33错误、scene4绑定保存/重开、实际全mesh GPU/Stop；原型gap F-846-T02-NAN关闭。[geometry/whole-callback Fault/Stop](attachments/846/work-r2/spatial-geometry-host-r2.txt)同时验证last-good、VM关闭、Starting取消与修源新Run，不将安全占位算成功。

T-03 [x]：同raw API切换不no-op、Undo/Redo、raw Arc复用但按API准入、不合法切换原状态保留、v2保存重开；显式临时文件v2与v1编译失败不回落、伪API/profile/base拒绝；孤立CLI版本/未知版本实际exit2。[project](attachments/846/work-r2/spatial-project-r2.txt)、[Host版本](attachments/846/work-r2/spatial-version-host-r2.txt)、CLI raw。原v1定向模块12/12、project session4/4、storage5/5通过。credential仅记录已编译API种类，不保VM/cache；跨API每次具名校验，v1原临时入口保持一次源码读/一次Run编译。

当前executing4/11，T04..08未宣称完成；新源/验证清单随开发变化最终由T08冻结。未启动最终review/完整根档/合入；所有历史red保留，后续继续原合同。

### 2026-10-25 work:r2 最终 — pass / execution_done9/11

stage:work | plan_id:PLAN-846 | plan_revision:2 | outcome:pass | worktree_path:`D:/autostack/.wt/lang-846/auto-lang` | branch:plan-846-dev | base_commit:`c1c7cfea5c9cc84987091ddeaf6cde59cefd5d4f` | code_commit:`655c366a3a4c53160c259a0b91a50c02d1cb020a` | dependency_revisions:SDK`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`；auto-down`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；三锁字节保持 | task_ids:T-01/T-02-SDK/T-02..08全部关闭 | evidence:[完整readiness及AC映射](attachments/846/work-r2/readiness-final.json.txt)、[提交后来源](attachments/846/work-r2/committed-source-final.txt)、[native14次](attachments/846/work-r2/native-final/results.json)、[owner变异控制](attachments/846/work-r2/owner-controls/results.json) | blockers:无 | next_responsible_role:review。

T04：真实Host资源借用、全mesh局部边界/8角变换、缓存与有限/退化/不可用状态和身份定向通过。T05：实际Auto闭区间查询、原数值合同、32/33限制、同callback父写查询、sticky error、last-good/Fault/Stop、配置未准入不得运行app_start全部通过。配置startup采用真实计数反例先红后绿，失效setup尝试保留未计证据。T06：正式UI/CLI/Binding.api与同raw历史/保存/重开闭合，两个真实Auto状态enter/stay/exit示例含Circle矩形代理corner像素，原v1 Move/Patrol实际OS27阶段及既有定向模块通过。

T07：同冻结binary的12种case+2次新进程narrow重开共14次通过，四正向完成14阶段；真实透明/缺part/错asset GPU控制均在指定像素门拒绝，partial非零，不把初始化错误当目标反例。T08：20轮真实owner与资源/扫描复用、5项真实mutation红和恢复通过；新API/旧SDK pin/source-lock/缺GLB四来源控制通过。独立仓外consumer使用自有SDK及target，来源344hash通过，原NaN双guard实际1测试与多mesh GPU/Stop实际1测试通过；producer最终all-targets check、包内fmt、几何2测试通过。SDK17定向、根auto-val check通过；既有SDK146warnings不属新增。所有work必需AC已关闭，无剩余合同内finding/覆盖gap/待决事项，未缩减支持。

SDK artifact由永久tag `autographics-sdk-846-nan`保留，不随scratch/Plan清理丢失。拟议Spec delta留在实施树，未在主检出发布canonical或ledger。历史setup/探针失败与r1反例全部保留，来源工具新增api目录纳入闭包后才算最终冻结。

T09独立复审未执行、T10未执行。完整统一门明确merge-owned pending：root裸cargo t+tv、producer/fresh consumer完整预计175、AGP原163/16 native、845原83有效调用及原分母、新spatial每侧14与owner/source围栏；不得以本work定向替代。完整清单与复用条件见实施树research/spatial-query-handoff.md。通过review后才在冻结提交执行，随后分离根SDK修复与autographics-dev图形落地、canonical/ledger/归档/guard清理及到期收尾。本次无merge/交付完成声明。

### 2026-10-25 review:R1 final — needs_fix / executing6/11

stage:review | review_scope:final | reviewer_identity:Codex /root；同实施会话、无独立上下文，已明示限制并从合同/冻结代码/原始证据重建结论 | plan_id:PLAN-846 | plan_revision:2 | outcome:needs_fix | worktree_path:`D:/autostack/.wt/lang-846/auto-lang` | branch:plan-846-dev | reviewed_commit:`655c366a3a4c53160c259a0b91a50c02d1cb020a` | base_commit:`c1c7cfea5c9cc84987091ddeaf6cde59cefd5d4f` | dependency_revisions:SDK7b987ca8、auto-downc39ac7f9、三锁保持 | spec_inputs:[当前/拟议hash与冻结patch](attachments/846/review-r1/baseline.json.txt) | acceptance_results:AC01/02/04/05/07 pass；AC03/06/08仅CLI子项partial；AC09未过、AC10 merge pending | findings:F-846-R1-01 P2 | evidence:[完整复审/复现/作用域](attachments/846/review-r1/review.md) | next:work，同树只修CLI缺值验证及对应永久反例/来源刷新后提交，再受影响复审。

唯一finding：`--inspect <合法scene> --behavior-api`末尾缺值时exit0且closure_verified:true；相同入口带v2但无--behavior正确exit2、不带选项正确exit0。实际生产CLI将“未提供选项”和“提供但无值”同映射None，违反§5.5/README孤立版本须usage error。32MB正式profile重复同证，精确命令/输出/二进制hash见附件；不是新支持需求。重开T03/T06及依赖其完整验收的T08，保留其他6项完成，current_step6/11，revision2不变。

本轮重新执行全部17个spatial定向测试、独立holdout2（native ln NaN三坐标、pair第二参数错误/单次计费/即时world）、SDK纯模块17均通过；actual3D native正例14阶段、真实透明GPU反例及partial传播门通过；独立consumer四来源变异拒绝与恢复通过。原14native调用、owner实际5mutation及v1未变证据逐项核对/具名复用。初始手工holdout链接/线程栈及native cursor-left环境尝试原样保留，不归为产品finding；后续有效验证与其区分。SDK固定artifact单路径/root同blob、canonical基面无漂移、SD01..05及原分母保持。未改实现/发布canonical/live ledger/合入/清树，树末核clean且HEAD未变。

完整统一门仍merge-owned pending：未来最终pass后root裸t+tv、driver producer/consumer175、AGP163/16native、原83及新spatial每侧14/owner/source矩阵、独立根SDK/图形落地与知识归档guard到期收尾。本次是首轮final needs_fix，不触发两轮失败诊断要求；修复不要求重跑无关整档或已闭native/GC全矩阵。

### 2026-10-25 work:r3 — F-846-R1-01闭合 / execution_done9/11

stage:work | plan_id:PLAN-846 | plan_revision:2 | outcome:pass | worktree_path:`D:/autostack/.wt/lang-846/auto-lang` | branch:plan-846-dev | base_commit:`655c366a3a4c53160c259a0b91a50c02d1cb020a`（修复基线；Plan原base仍c1c7cfea） | code_commit:`bf22db3465203f3babd649494f4b37aa2a012027` | dependency_revisions:SDK7b987ca8/auto-downc39ac7f9/三锁未变 | task_ids:T03/T06/T08重新关闭 | evidence:[finding不变量/输入/关闭表](attachments/846/work-r3/readiness.json.txt) | blockers:无 | next_responsible_role:review，用户已授权继续受影响复审。

F-846-R1-01：生产CLI在版本解析前区分option存在与值缺席，终端缺值及下一option误当值均exit2且不执行inspect；合法absent/default/v1/v2保持绑定。永久spatial_cli 3测试实际10子进程（3缺值、3孤立/未知、4合法）原parser先2pass/1red，修复后3pass；实际Host文件版本/伪造不回落定向1pass，all-targets check/fmt/diff通过。未扩大语言/API/SDK/几何/预算/原生范围，语义revision2不变。

最终来源闭包345hash（新增CLI回归），SDK681及三锁不变；fresh consumer846-r3-final自有SDK/源码导出与四实际来源变异/恢复通过，未跑consumer整档。新增3永久测试使merge预计完整driver分母178=原158+本Plan20；README/handoff/拟议integration Spec只刷新计数。R1未变AC/原证据具名保留；全局t/tv/整包/native全矩阵未重跑，最终pass后仍归merge。实施树末核clean。

### 2026-10-25 review:R2 final — pass / reviewed10/11

stage:review | review_scope:final | reviewer_identity:Codex /root；同实施会话、独立上下文限制保留，从R1/冻结diff/实际命令重建受影响结论 | plan_id:PLAN-846 | plan_revision:2 | outcome:pass | worktree_path:`D:/autostack/.wt/lang-846/auto-lang` | branch:plan-846-dev | reviewed_commit:`bf22db3465203f3babd649494f4b37aa2a012027` | base_commit:`655c366a3a4c53160c259a0b91a50c02d1cb020a`（修复diff；原基面c1c7cfea） | dependency_revisions:SDK7b987ca8/auto-downc39ac7f9/三锁未变 | spec_inputs:[十目标hash/冻结patch](attachments/846/review-r2/baseline.json.txt) | acceptance_results:AC01..09 pass，AC10 merge pending | findings:F-846-R1-01 CLOSED，无剩余finding/gap/决策 | evidence:[受影响复审与复用边界](attachments/846/review-r2/review.md) | next:merge，未获本轮合入指令，不代执行。

R1原三组实际命令逐个重放，另选缺值在inspect之前/带真实source/空值/坏entry前缺值/合法v1与重排v2六组，共9个真实进程符合精确退出码和诊断，合法项目绑定保持。永久CLI三测试（10子进程）重新通过；生产/独立final consumer来源345hash验证通过。实际Host版本定向、check/fmt复用work同冻结结果。AC03/06/08缺口关闭；其他R1已闭几何/SDK/配置/资源/native证据经diff/依赖/配置/输入未变核验后保留，未重跑无关整档。

SD01..05语义与canonical基面未漂移，拟议delta仅完整测试计数增加3至178（原158+本Plan20）；revision2不变。树末核clean/HEAD一致，未改实现、canonical/live ledger、合入或清树。最终pass只表示代码/验收/规范通过；T10 merge-owned裸root t+tv、producer/fresh consumer178、AGP163/16native、原83和新spatial每侧14/owner/source完整矩阵及独立SDK/图形落地、知识归档guard到期收尾全部pending，通过前不得合入或宣称交付。

### 2026-10-25 merge:r1 — prepared / 完整门执行中

stage:merge | plan_id:PLAN-846:r2 | checkpoint:prepared | reviewed_commit:`bf22db3465203f3babd649494f4b37aa2a012027` | worktree:`D:/autostack/.wt/lang-846/auto-lang` | branch:plan-846-dev | root SDK:4cef80c1b/artifact7b987ca8 | evidence:[门准备](attachments/846/merge-r1/prepared.json.txt)、[精确AGP未变复用](attachments/846/merge-r1/agp-reuse.json.txt) | landed/ledger/archived/cleaned:false。

根裸cargo t首轮实际选择5332，运行1150：1131pass/19timeout，4182因fail-fast未跑，不计完整绿。超时实际位于AC/AV/ABI worker冷构建路径，保留原raw及进程快照；同时检测到831另有根t。正在不改代码/选择面/断言/时限的前提下顺序预编译同树release worker，再以定向反例/原正控验证归因，只有前置条件改变且有效才重新执行冻结完整门。producer完整driver/build正在执行，其余consumer/native/root tv待序列门。任何必需门未通过前不合入/归档/清树。

### 2026-10-25 merge:r1 根接收基面对齐（未合入）

图形独立workspaces继续冻结bf22db346。旧图形分支携带的较早根仓快照在全档中出现cold worker超时、census墙钟比较、chart CPU continuation与future_all相对时长失败；各原raw/定向重放保留，未拼凑完整绿。实际master已包含后续chart测试completion修复及根代码更新，不能用旧图形根仓快照替代实际根接收者门。

按独立SDK落地主路径，在**同一唯一实施worktree顺序切换**临时根合入分支`codex/plan-846-sdk-merge`，原Plan分支保持bf22未动；将已审纯SDK4cef80c1b重放到实际master基面9a6c2dace33e65de8db68e7655118bc823eab30a，形成44480cf6aff20f3b1e2a80df783976b866f5920f。[range-diff完全等价与单路径/同blob](attachments/846/merge-r1/sdk-root-equivalence.json.txt)。无冲突、无新增特性/修改根UI代码；不会把图形源码带入master。此为技能允许的旧→新落地hash映射，不更改Plan语义revision。

该实际根基面另有CTE/MN两类新worker，首次全档的3超时如实保留；已补齐6类release前置构建，原CTE/MN3路径重放验证。实际接收者冻结44480的完整t+tv仍pending，旧根失败不改记通过。producer完整178（日志含1个嵌套child，top-level须去重）/build/source已通过；fresh own-SDK/own-target consumer的66定向+4deadline通过、完整178运行中。SDK纯模块17在同blob通过。全部代码/依赖/输入/实际测试分母和根/图形两条落地身份由最终收据核对；源WT/gfx/根SDK分支保留，尚无landing/ledger/archive/cleanup。

### 2026-10-25 merge:r1 — blocked / 未合入

F-846-M1-ROOT-MN-P95：实际根接收冻结44480（纯SDK映射，master父9a6c2d）完整cargo t在6类worker预编译后仍失败：879pass/1fail、4488未跑，signature-rebuild 30样本P95=1032.8293ms≥1000ms；归因待查，不判定为846实现缺陷。实际根tv163/163、producer/fresh consumer完整driver各178及build、SDK模块17通过；AGP23槽具名未变复用。其余原native/owner/project/Step和新spatial完整矩阵pending，不拼凑完整绿。旧根/cold历史原样保留。

[合入阻断与恢复收据](attachments/846/merge-r1/receipt.md)及30样本已保存。唯一WT已恢复plan-846-dev/bf22，原始来源字节恢复并核345/SDK681，Git clean；未落地SDK映射分支和永久SDK tag保留。status仍reviewed10/11；landed/ledger_refreshed/archived/cleaned全部false，未改canonical/live ledger/清树。根性能门调查及必要修复/受影响复审后，重新核对冻结并完成完整门才可继续合入；本轮不再无条件重试或放宽门限。

### 2026-10-25 work:r4 — 根门禁修复授权

用户明确授权排查及修复F-846-M1-ROOT-MN-P95，不放宽1000ms/30样本、不缩减测试范围。 本次授权扩展根接收路径至ac-core的Atom reader/Schema binder与ac-mir-native缓存消费（含定向回归），仅消除重复分配/读取、保持全部语义/诊断/准入/数值与支持合同；修复单独落master，图形仍只落autographics-dev。T09受影响复审重开，current_step9/11，revision2保持。唯一WT顺序进入codex/plan-846-sdk-merge/44480根角色；原图形bf22及其未受影响证据保持。受影响根门证据失效，修复后先定向验证/受影响复审，再进入merge完整门。原样本阶段观测已定位frontend/link_run及缓存重复读取，尚未宣称根因关闭。

### 2026-10-25 work:r4 — 根性能修复冻结 / execution_done9/11

根修复0a12b60d6经range-diff完全等价重放到实际master00973e1d1，冻结根2fffb7e3a775e18175cbab5a392a8cf827f5f21b；图形冻结仍bf22。仅去除Atom token文本复制、Schema字段/规则表克隆和COFF hit二次读取；词法位置/重复与非法字段诊断、Schema绑定/验证、实际链接运行/缓存身份与准入保留。原门限1000ms、30样本、原全档选择面/配置均未改；最终定向原用例30样本P95=758.0851ms（p50=721.1186、max=788.925），101共享binder/source定向及cache7、cargo check与格式通过。首中间修复仍红P951625.7（期间存在本会话剖析负载）原样留存，不冒认pass；最终采样串行无本会话额外负载。

[work冻结/hash/等价/样本](attachments/846/work-r4/readiness.json.txt)。根门finding等待受影响复审确认；原图形/SDK合同及review-r2未变面证据保留。必要root修复为用户本轮明确授权范围，不合入任何额外生产支持。完整根t/tv与图形未完门仍merge-owned pending；未落地/规范发布/归档/清树。

### 2026-10-25 review:R3 affected final — pass / reviewed10/11

根冻结2fffb7e3a（pure SDK72e61c508+根性能修复，range-diff等价），图形仍bf22；F-846-M1-ROOT-MN-P95必要实现修复闭合，原30样本P95758.0851<1000。字段拒绝矩阵/COFF身份准入/双函数局部更新/原签名冲突30样本复审过，原R2未变AC/十目标SD具名保留。reviewer为同实施会话Codex /root，上下文独立性限制明确保留。[受影响复审](attachments/846/review-r3/review.md)。

实际master含831 UI迁仓，旧根44480门不能代替当前接收基面；新增只读组内auto-ui@aee499e87（正式交付pin），auto-down@c39ac7f9保持。第一次签名冲突因缺兄弟依赖在加载时exit102，已补齐后原命令通过，不当作产品红/静默跳过。根性能修复不改Schema/支持/数值/1000ms/30样本及门禁配置，零新规范合同；T10须当前根完整t+tv和迁走UI日常面、不缩减原覆盖，并完成剩余图形完整矩阵。pass不代表完整门或交付通过；未落地/规范发布/归档/清树。

### 2026-10-25 merge:r2 — 当前根完整门通过 / 图形矩阵执行中

根冻结2fffb7e3a、固定UIaee499e/downc39ac7f9：语言裸t3161/3161、tv163/163通过。831迁仓前根门明确只选compiler/dialect/runtime三个UI包；迁前真实测试二进制与当前命令nextest清单逐ID相等247/86/1874共2207，原ignored与过滤均未改；[三包完整2207/2207通过及逐ID映射](attachments/846/merge-r2/ui-original-scope-equivalence.json.txt)。因此原5368全覆盖=3161+2207，skip1552=668+884，没有缩减原测试范围。完整档中的签名重建30样本P95887.1887ms<原1000ms。

首完整档FFI字段超时120s原样保留；原例1.548s与整族5定向通过后，在同冻结完整重跑全3161通过，未把定向pass替代整档、未改时限或过滤，超时原因未确认。新UI裸workspace额外选择原根门未含的189项tools/shell测试，扩展诊断命中已登记P693-D3/P751-D2的merged_api旧期望失败；不将其混作原5368门，也未冒称新UI全库绿或清偿该工具债。完整命令/原始失败与归属保留[收据数据](attachments/846/merge-r2/root-gates.json.txt)。

唯一WT已恢复图形bf22，来源345/SDK681/metadata479闭包及Git clean通过；producer/fresh consumer各178及AGP23未变复用仍有效，根性能代码不在其固定SDK依赖闭包内。剩余原生/owner/source/项目搬迁/Step及新spatial每侧14与owner完整矩阵正在同树串行执行。未落地/规范发布/归档/清树。

### 2026-10-25 review:R4 — Plan820接收基面对齐 / affected final pass

实际master新增820共享MIR扩展，原2ff/00973根整档证据不作为最终接收证据。唯一WT顺序重放至2090e9094，根新冻结204bc0781；SDK210fc8410与性能204bc0781两补丁range-diff完全等价。原binder拒绝、整个cache_tier准入/身份及双函数局部更新在新基面通过；检查820新增类型/指令受控拒绝与windows-sys锁边，不改846五文件/SDK数值/支持/30样本1000ms合同。图形bf22未变，R2十目标SD保留。[R4受影响复审](attachments/846/review-r4/review.md)，同实施会话独立性限制如实记录。新根t/tv和原迁仓三UI包整档仍merge-owned pending；其通过前不合入。

### 2026-10-25 merge:r3 → work:r5 — 新接收基面P95再次失败

图形87有效门全部完成（83原门+4空间），每侧14空间调用、4Step、15项目与2界面搬迁均通过；交互干扰失败历史保留，静默输入重放闭合。根冻结4df468b54/master d14983ae9，实际新增820后全档3172运行888：887pass/1fail，2284未跑；原签名重建30样本P951249.3752ms≥1000，P50955.8096。该真实失败不豁免、不缩范围，R4实现审查不足以证明新基面性能门通过。开启第二轮必要修复work:r5，T09受影响重开，executing9/11；保留图形/SDK与未变SD，根性能关闭待新证据。尚无landing/ledger/archive/cleanup。

### 2026-10-25 work:r5 — 链接工具身份有界读取 / 性能仍未闭合

用户必要修复授权下，MN链接工具身份从整文件临时Vec改为64KiB缓冲完整BLAKE3散列；原字节覆盖/算法/身份轴/准入/支持不变，不引入跨进程hash捷径。607ed28c6已提交，8项cache整族及格式通过；空/64KiB边界/短尾部/重复读取/缺文件对照补齐。实际原30样本定向P951107.9436ms仍≥1000，未送final pass、未伪称finding关闭。检测到其他仓大型编译，正在记录负载并等待30秒无其他cargo/rustc/nextest窗口，再决定原门复验；竞争影响尚未确认为唯一根因。与图形固定SDK闭包无交集，87门原始报告与失败历史已提交。所有规范/ledger/landing/archive/cleanup仍未执行。

### 2026-10-25 work:r5 等待窗口结束 — needs_fix / executing9/11

只读观察10分钟未取得连续30秒无其他构建窗口，故未再触发性能采样，无后台重试。最新有效原门仍P951107.9436≥1000，F-846-M1-ROOT-MN-P95未闭合；不能把竞争观察当作已确认唯一根因。原生重测87门完成证据保留，必要有界工具散列607ed28c6及8项测试已提交。唯一WT根角色clean、图形bf22/SDK永久tag/两个固定只读兄弟均保留；未合入/发布规范/写ledger/归档/清树。[当前恢复收据](attachments/846/merge-r3/receipt.md)、[当前未就绪记录](attachments/846/work-r5/readiness-pending.json.txt)。

### 2026-10-25 new:r3 — 用户授权暂缓AutoGraphics增量性能验收

用户明确“暂时先把性能要求放松；功能完成尤其与真正ACE引擎合并后再优化；AutoGraphics暂只需支持增量编译，不考虑性能检查”。revision2→3，执行中9/11，AC/任务ID和已完功能保留，T09须受影响重审。F-846-M1-ROOT-MN-P95按明确合同变更转为非阻断观测/后续性能议题，历史1249.3752/1107.9436真实红原样保留，绝不宣称性能已修到1000以下。根完整t/tv及原UI迁仓面仍执行；新增命名策略只令MN增量编译延迟为观测，功能断言/30样本/支持/错误/实际链接运行/数值/运行时预算不变，默认其他根验证保持原性能策略。拟SD-06仅落现有integration-contract，r2其他十目标SD保留。[r3冻结提案](attachments/846/revision-r3/spec-delta.md)。本次授权足够，不另索确认。

### 2026-10-25 review:R6 — r3受影响final pass / reviewed10/11

根33ed071a6，图形bf22、SDK7b987ca8、UIaee499e/downc39ac7f固定。用户授权的AutoGraphics增量功能优先策略已实现，默认根性能门仍严格；显式observe只撤销四处增量延迟判定，原功能/缓存失效/实际链接运行/30样本/数值/支持/安全deadline不变。策略正反+实际序列+签名重建3/3，最终元数据修订后策略1/1；签名P951428.581ms明确观测非门禁，不冒称优化达标。有界工具散列及整cache8证据保留。SD06新提案与十目标r2SD经hash核冻结。AC01..08具名复用未变证据，AC09受影响通过；同实施会话审查上下文限制如实保留。[R6复审](attachments/846/review-r6/review.md)。F-846-M1-ROOT-MN-P95按用户合同变更转D-846-INCR-PERF后续议题，非“性能修复关闭”。T10仍须新冻结完整t/tv及原UI2207功能面，再完成全部落地/知识/归档/清理。

### 2026-10-25 merge:r4 — r3冻结完整功能门通过

根冻结ae5616472（R6根33ed071a6四补丁完全等价重放到mastera45a00678），显式AUTO_LANG_AUTOGRAPHICS_INCREMENTAL_PERF=observe，原30样本/全部功能断言/执行deadline不变。根裸t3173/3173（skip668）、tv163/163（skip2006）及原迁仓UI三包2207/2207（skip884）全部通过，原5368范围保留且纳入820与策略新增用例。所有原始结果绑定[本轮冻结](attachments/846/merge-r4/frozen-root.json.txt)/[本轮完整命令](attachments/846/merge-r4/root-gates.json.txt)。P95按r3仅观测，非优化达标声明；旧额外UI工具诊断红与847批量阻断不改记绿。

图形bf22的87有效门（83原门+4空间）、producer/fresh consumer各178与AGP163/16原分母经实际断言审计；仅具名23 AGP未变复用。每侧14空间调用/4Step/15项目/2实际界面搬迁过，历史输入干扰失败原样保存；source345/SDK681/metadata479闭包再次通过且WT clean。[完整审计](attachments/846/merge-r1/validated.json.txt)。当前仅完整门已过，尚未记录landed/ledger/archive/cleanup；下一步规范准备及两条真实ff-only落地。

## 10. 待澄清事项

无待用户决定事项。r3已明确暂缓AutoGraphics增量编译性能检查，历史P95红不再阻断本线路；功能/数值/支持/运行时预算与原完整功能门保持。策略实现、受影响复审及新冻结完整功能门已通过，待合入/规范沉淀/账本/归档/清树。
