---
plan_id: PLAN-848
status: reviewed
feature_name: AutoGraphics有限线段查询与最近命中交互
plan_revision: 3
author: [Codex]
created_at: 2026-10-25
updated_at: 2026-10-25
current_step: 9
total_steps: 10
depends_on: [PLAN-846]
supersedes_spec_components:
  - docs/specs/auto-graphics/spatial-query-profile.md
  - docs/specs/auto-graphics/behavior-api.md
  - docs/specs/auto-graphics/behavior-project.md
  - docs/specs/auto-graphics/scene-play-session.md
  - docs/specs/auto-graphics/auto-source-driver.md
  - docs/specs/auto-graphics/prototype-profile.md
  - docs/specs/auto-graphics/project.md
  - docs/specs/auto-graphics/integration-contract.md
new_spec_components:
  - docs/specs/auto-graphics/segment-query-profile.md
touched_goals: [GOAL-007, GOAL-008]
affects: [auto-graphics, authoring-experimental, segment-query]
---

# [PLAN-848] AutoGraphics 有限线段查询与最近命中交互

## 0. 变更摘要

当前状态：用户授权的独立子代理 `/root/review_848_r3` 最终复审 **pass**，F-848-R1-01 CLOSED，T09完成，reviewed9/10。冻结提交6bc5abc60，128节点/32credits/原CPU合同保持。T10完整合入门pending，暂不合入。详见[独立最终报告](attachments/848/review-r3-final/review.md)及§9 review:r3；下方授权待答复、独立pending及OPEN仅为历史记录。

846提供有限2D/3D AABB包含/重叠和用户触发逻辑；下一步增加**有限world线段扫描、确定性最近命中及有界结果读取**，让真实Auto脚本实现光束选目标、视线遮挡/开门。沿用现有几何代理，不把无穷射线、triangle精确拾取或完整物理并入本版。

新增显式`autographics.behavior/3`、profile9，复用846受检spatial描述与Auto算法基础；v1/v2/profiles1..8的managed身份、合法源码/保留名集合和原查询预算保持。scene4结构不变，新增可接受的Binding.api值。两个样例经正式UI/CLI绑定、保存、搬迁、重开与实际GPU验证。

用户允许规划一或两份；本轮选择**一份可执行粒度的下一草案**。扫掠AABB/防穿透运动是未取号后继方向，待本查询结果与数值/资源合同稳定后再细化，不提前冻结运动响应设计。

846 r3已正式交付归档11/11，G846/GSTART/T-01已关闭，正式图形基面bf22db3465203f3babd649494f4b37aa2a012027、固定SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467。848 r2的128预算缺口已按用户顺序先验证新方案、再修订r3并实施；当前冻结6bc5abc6080b2261a43f55a54f127cd779239edb，60组Session/6组Host及原始反例、三轴斜向控制通过，工作完成8/10待独立复审，尚未合入。历史失败均保留；847不是前置，批量due不冒称全绿。

## 1. 目标

1. 对选择维度、忽略actor和可见性政策下最多128节点的实际AABB代理做有限线段查询，返回最近命中的当次Run handle、t、world点及进入面法线；命中/未命中/查询错误分别可辨。
2. 实际相交/排序与结果生成由Auto执行，同callback父组/叶子移动后查询看到最新world；查询结果保存查询瞬间的数据，不伪装成持续更新的view。
3. 提供单槽、token绑定、callback内有界生命周期的结果读取，避免静默读取上次查询或旧callback结果；维护sticky错误、整帧准入与Stop隔离。
4. 显式v3来源/项目绑定与旧版兼容、原始源码/验证身份正确；两个不同维度的用户项目无需改Rust即可复跑并改变交互逻辑。
5. 在原CPU/runtime/资源范围内验证128节点、数值边界、确定性、错误恢复、原生最终像素及真实owner/缓存/上传不退化。

非目标：无穷ray、三角形/OBB/椭圆精确命中、sweep/连续碰撞/物理解算、move-and-slide/重力/刚体、动态创建/删除/材质或层mask、相机控制/屏幕鼠标unproject API、自动引擎事件队列、完整编辑器拾取替换、UI组件/默认设计/统一Schema、ACE接线/增量/HMR、生产AutoUI/RQHost与发行。本Plan不改已冻结Breakout/Squash碰撞或现有pick2/pick3精度语义。

## 2. 架构方案

```text
接受Document/资源 + 846 Host原始ModelData借用
  → 同一受检spatial Descriptor / 原算法与base身份
  → 显式behavior/3/profile9完整初始化
  → actual Auto：derive最新world → filter候选 → closed segment/AABB slab
  → 按t再按Run ordinal选最近 → 单槽不可变查询快照/正token
  → 同callback getters → 用户Auto反馈/开门逻辑
  → 原完整frame检查/Surface/GPU → Pause/Step/Stop
```

L1局部实验driver特性，仅写`experimental/autographics-auto-driver/`：拟新增`api/behavior_segment_v3.at`、tests/segment_*、研究/样例/自检；局部修改behavior/behavior_project/Play/Host/bin。只消费846正式SDK（观察7b987）和spatial输入，不修改SDK、auto-val、AGP、生产crates、根manifest/三锁或迁移树。不造第二parser、通用query框架/物理引擎或native Rust求交路径。超边界先回new，重大架构扩面按L2设计。

| 门 | 成立条件 / 当前状态 | owner_stage |
|---|---|---|
| G846 | 已成立：846 r3 archived/delivered11/11；正式graphics bf22db346、root d369dd760、ledger fcea7d8dc，guard清理完毕；批量due由847承接，详work:r1 | work T-01前 |
| GSOURCE | T-01完成，正式SHA/SDK/三锁及source反控冻结于work-r1/frozen-source.json.txt | work T-01 |
| GSTART | 已成立：用户明确要求按work实施848 | work启动前 |

唯一实施树`D:/autostack/.wt/lang-848/auto-lang`、分支`plan-848-dev`，显式基于最终846正式autographics-dev完整SHA，不默认从master建树。实施/internal/final同树顺序，主检出记计划/附件/知识；源码只落autographics-dev。禁止junction/symlink、移树前wt-guard；原I2/I3与master图形合入迁移门保持。

## 3. 技术栈

复用846 Rust/stock Iced/wgpu、实际Auto Parser/Session/CPU切片、Api枚举/SourceSnapshot、spatial::Descriptor和静态ModelData借用、scene4/CodePayload/Storage。T-01正式基面与三锁/原分母见work-r1/frozen-source.json.txt。r2前现有代码提交5efe91a8a0cead18288215d74aa2d339af2e26f9；新方案探针源与日志见work-r1/selected-candidate.at.txt及candidate-parent-cache.log.txt，不把探针当公开v3/GPU交付。

slab计算用受检有限f32及原Auto运算/控制流；不需要新数学库或归一化/sqrt。v3私有单元复用v1/v2的图/错误/box元数据函数，未修改的v1/v2 API/ENTRY组合字节身份必须维持。结果是固定少量标量，不把候选数组/mesh或每次查询对象塞进VM heap。

## 4. 需求分析与背景调查

### 4.1 授权、知识与现状

2026-10-25用户先授权实施848，随后明确“先找到新方案，再更新计划，再实施”，直接修订848，不另建计划。允许在唯一848树调查、修订等价计算路径并继续实施；不扩大SDK/AGP/生产写面、节点/CPU/数值合同或新agent/chat权限。历史846观察留在§9和原来源附件，当前消费正式交付。

已读[全局overview](../specs/overview.md)、[图形overview](../specs/auto-graphics/overview.md)、[行为API](../specs/auto-graphics/behavior-api.md)、[项目绑定](../specs/auto-graphics/behavior-project.md)、[运行会话](../specs/auto-graphics/scene-play-session.md)、[视口几何](../specs/auto-graphics/viewport-interaction.md)、[集成边界](../specs/auto-graphics/integration-contract.md)、[846](archive/846-autographics-spatial-queries-and-trigger-interactions.md)及[其R1 needs_fix](attachments/846/review-r1/review.md)。spatial-query-profile.md已由846正式沉淀，作为modify基面；历史候选观察不再作为当前前置。

26个固定Git对象/16份文档hash与状态见[来源清单](attachments/848/planning/source-observation.json.txt)。831 reviewed10/12、832/841 drafting，完整生产迁移未交接；完整组件/设计体系、Schema统一声明与ACE/HMR仍按[Design45 §11](../design/45-autographics-wgpu-engine-and-game-prototypes.md#11-目标态schema-统一-ui2d-与-3d-声明语义2026-10-24)后续分期，本Plan不依赖未交付接口，也不冒认其成果。

### 4.2 已观察触点与缺口

| 实际触点（846候选） | 事实 | 本Plan消费点 |
|---|---|---|
| spatial.rs::Descriptor | 实际资源全mesh/静态8角offset、base hash、rows/资源身份/local_scans；最大128节点 | 原样消费同一代理及身份，不重扫顶点/新碰撞shape |
| behavior_spatial_v2.at | `_ag_spatial_edge`读Auto world+offset；32次attempt、ready/finite/sticky错误，closed contains/overlap | v3复用私有edge/检查/派生，用独立segment数学及有界结果槽；不能循环调用收费公共getter扫描128个候选 |
| behavior.rs::Api/from_bytes_for | V1/SpatialV2到版本/profile7/8，组合/region/准入及reserved集按选定managed单位形成 | 增SegmentV3/profile9，v1/v2保持原managed hash和合法helper |
| Play::new_user_with_spatial及配置入口 | 源码+Descriptor与完整图初始化后才能callback；profile/header检查和整帧不可变字段 | v3显式构造/consumer复用完整初始化，不能只改返回profile绕过准入 |
| CodePayload/Binding/Host | 版本化准入、raw按SHA共用、已接受来源与临时覆盖、真实UI/CLI | v3绑定/切换/Undo/闭包/来源保持同合同，旧项目不自动升级 |
| 原interaction::pick2/pick3 | CPU屏幕拾取含可见性/投影，2D精确椭圆、3D OBB | 新API是world有限线段/AABB，不复用其屏幕坐标或宣称同精度 |

## 5. 详细设计

### 5.1 r3：按脏坐标轴推导与受信热点源码布局

T-01已冻结正式来源；r2在R1暴露128晚轴miss/逐个变近预算缺口，work-r3探索均未采用。用户再次明确沿848寻找新方案、先验证完整控制后修订实施，128/32credits及CPU均不变。r3候选已通过[实际60组Session与6组Host控制](attachments/848/work-r4/full-controls-fixed.log.txt)，正式实施前已保存候选源码快照并恢复原产品文件；据此更新当前合同，不另建Plan、不把预研当最终交付。仍不引入额外Descriptor/全场cache/结果list、native求交或SDK写面。

- Auto仍X/Y/Z展开、ordinal升序、每匹配box全部finite验证在任何剪枝之前；按方向预选offset索引，实际f32商及严格nearest/tie不变。扫描只保存最终t/target，命中点和法线在最终胜者上物化一次；法线用相同f32商按XYZ优先恢复，outside正t下溢0仍保进入面，inside/boundary仍全0。数值/错误/token合同不变，执行的非finite算术仍拒绝；GeometryUnavailable不得被剪枝隐藏。
- 仅v3给原共享setters派生四个有界dirty标量X/Y/Z/visibility：set_local标记指定轴，move_local保守标记三轴（即使某delta为0），set_visible标记visibility。原dirty布尔与callback收口不变。每次derive只更新实际标记的轴/visibility；初始化或原数组形状不完整时预填同一world/effective数组并全量重算；无标记却显式derive时保守全量。原数组原位更新，不创建第二套世界图或几何cache。
- 从同一shared derive提取并校验原轴内公式及visibility片段，只旋转循环到“每脏轴、parent-first节点”，每轴每节点仍严格local+parent一次f32加法；每轴局部缓存最后parent，切换时重读，不跨derive/callback/Run保留。visibility仍parent-first原规则。仅v3替换受信代码，边界漂移失败关闭，不手抄第二图算法。
- 固定SDK将SOURCE_LINE也计入指令预算。仅对v3受信数值API/derive生成紧凑行布局，去除冗余调试行切换；仍交原Parser/Codegen/VM执行，每条实际指令照原预算收费，无字节码补丁/计数绕过。用户原始字节/编译单元/源码哈希不改，v1/v2源码布局与managed身份不变；v3生成区按真实布局报告，不伪称原行映射。
- 结果槽仍11标量44B；额外四个dirty标量16B单独计量，两者合计15标量60B≤16/64B。无新增候选/结果/几何数组。callback末尾仍仅dirty时derive，token仍callback begin/end失效。

预研60组=2D/3D×宽/深×6分布（等深、晚轴miss、逐个变近、正/负/非等比例三轴斜向）×各有效写入轴，另6组正式Host使用真实Cylinder/完整配置/独立几何结果及最终tick1。每callback双group单轴写、2cast+10getter+6AABB精确32；沿原Session/Host预算。yield只是调度次数，不是精确步数或余量。正式实现须重放，并验证多轴move、混合dirty、visibility、parent切换、最终发布、数值/生命周期及变异敏感性；不得把受限X探针当通用完成。

T-02关闭数值/身份/资源方案和CPU可行性并将probe敏感性绑定；原“整链必须完成才能T-03”与T-03首次创建正式入口的依赖循环予以消除：公开v3/consumer/token/GLB/GPU/Stop/搬迁分别仍由T-03..07负责并保持全部AC/V不删减，T-08检查整链无gap后才可execution_done。新建tests/segment_api.rs、segment_geometry.rs等永久断言覆盖早退前坏几何、非首父组/切换父组/负坐标/隐藏、写后查与查后写最终发布、端点/tie/平行/溢出及cached-parent错误变异。若完整控制因正式入口开销再次超额，继续同一计算方案的有证据局部修正，重复失败先诊断，不以再取号或减验收代替。

### 5.2 公共入口、候选与坐标

拟`ag_cast_segment(space, ignore, visible_only, ax, ay, az, bx, by, bz) int`，前三项int，端点float。space精确2/3；ignore为−1或同维合法叶子handle，0合法，group/外维/越界明确参数错误。visible_only为0/1，0包含隐藏，1只取effective visible叶子；alpha不是该过滤条件。group本身自动排除，不新增group aggregate相交。过滤后的候选至多128，按原Run ordinal遍历；ignore只是忽略一个节点，不代表忽略其子树。

坐标是world端点A/B，t范围[0,1]；不归一化方向、不用屏幕/DPI/相机，也不是无穷ray。2D两端z必须0，3D用三轴；端点及必须执行的方向/商/命中点运算须finite。有限输入在中间算术溢出时具名ArithmeticNonFinite，不伪装为miss。±0平行分量不做除零，退化但finite的box沿闭区间处理。

匹配候选必须先核有效finite world box；GeometryUnavailable不能因前方已有命中而被静默跳过。被ignore/不匹配维度/visible_only过滤排除者不参加几何校验；没有候选是合法miss。同callback前置的group/叶子写入经Auto dirty派生被看见，查询过程中没有其他写者/重入callback。

### 5.3 相交、最近命中与数值政策

使用closed segment/AABB slab：对每个有效维度求进入/离开t区间，平行且在范围外为miss，所有轴闭区间与[0,1]有交集则命中。起点在box内或其边界时t=0、法线全0；零长线段退化为点，包含时hit0，否则miss。t=1终点接触算命中，不能用epsilon扩张或排除端点。

最近按**实际f32 t升序，再按最小Run ordinal**确定；不按显示名或HashMap顺序、不用epsilon合并不同t。同值浮点tie不声称无限实数精度；有重叠/相同深度时规则可重复。进入面法线为某一轴的±1，其余0；同t的多轴进入按X/Y/Z顺序选一轴。只有起点不在box闭区间内时返回进入面法线，起点inside/boundary为0。

hit点为`A+t*(B-A)`，不是模型三角形上的精确接触点。代理继承846的Circle矩形/旋转3D全模型box8角world AABB，包含空白区域；新查询不改旧pick2/pick3或输入捕获。

独立oracle用手审可精确表示的轴向/斜向/角边/±1ULP样例及另一实现，不能调用被测Auto helper生成expected。fraction比较容差`max(1e-6,8*f32EPSILON)`，point比较沿原世界位置`max(1e-3,8*f32EPSILON*max(1,abs(expected),abs(actual)))`；正常向量精确0/±1。**容差只用于数值测量，不改变hit/miss、t顺序或边界规则**；近分离/inside和非finite必须有区分性控制。

### 5.4 单槽结果与token寿命

cast成功（hit或miss）返回一个新的正int token；失败返回0安全值并设置sticky API错误，不形成可用新结果。每次cast尝试先使上一结果失效；旧token不能读新查询。token当次Run单调且在i32正上限前受控ResultIdExhausted，不能回绕。不是持久ID/安全凭证，不跨Run保存。

拟getter：`ag_cast_hit(token)`为0/1，`ag_cast_node(token)`为handle或−1，`ag_cast_t(token)`为fraction，`ag_cast_point(token,axis)`、`ag_cast_normal(token,axis)`为float。结果在同一callback内有效，后续cast或callback结束后失效；未查询/错token/旧callback/坏axis具名错误并沿sticky-first拒绝完整callback。旧v2 AABB查询不会覆盖结果槽。

结果是**查询瞬间快照**。查询后同callback移动物体再读同token，仍读原t/point/normal；要最新命中必须显式新cast。不能偷偷更新旧槽以假装live view，也不需要重写v1/v2 setters来追踪几何epoch。Pause/Resume不调用getter/重新查询；Step一个callback；新Run重置VM、序列与结果。

合法miss同样有可读token：hit0、node−1、t1、point=B、normal全0；因此终点hit1/t1与miss0/t1可区分，不能拿−1当无效getter错误。固定槽/计数/标志逻辑原值合计≤16个标量（≤64B），无每次query对象/无限结果list；实际容器/SDK开销单独计量。

### 5.5 预算、错误与旧版兼容

v3共用每callback**32 query credits**：846 AABB公开尝试计1；segment尝试计8（一次最多128候选扫描）；每次result getter尝试计1，内部box/slab helper不重复收费。第一个超额调用在读几何/结果前QueryBudget，完整callback失败。边界由v3 callback入口只重置一次；v1/v2仍为原32次公共AABB尝试，不受新权重、token或reserved影响。

支持控制为128节点宽/深两类场景，每callback2次全场cast＋10次结果读取＋6次AABB查询=32 credits，含两次group写后查询；须在原100000steps/120resumes/250ms和30s初始化/2s全tick、8ms共享pump内通过。按过滤后实际候选计扫描，报告不把cast call数当候选数。没有自动O(N²)查询或引擎事件队列。

原v2错误码/语义保留；v3新增错误需单源描述，区分参数/unsupported box、ArithmeticNonFinite、QueryBudget、InvalidResult/ResultIdExhausted。错误不可被后续合法调用清掉；safe getter placeholder不能成为成功业务输出。query/numeric fault关闭污染VM、保最后完整帧；Stop恢复接受编辑内容，修源新Run恢复。调用新的native math producer产生NaN/±Inf、跨参数/局部/返回/容器的原846合同继续成立，不假设字符串解析失败已覆盖运行数值路径。

显式behavior/3/profile9：v1/v2 public表/managed source/ENTRY hash与reserved集合保持，新ag_cast_*不得回灌旧版本，让旧私有同名helper变非法。v3复用原v2单元及单一private图/box实现，新增受控segment源单元和callback begin/end；不复制原graph或parser，不改用户标识符/literal。

Api枚举/SourceSnapshot::read_for/from_bytes_for/Play typed入口/头部consumer增加v3，API/bridge/combined/region/base/spatial身份核验不减。正式Editing UI可显式选v3，CLI沿`--behavior-api v3`；缺省仍原v1临时文件政策，绑定默认用Binding.api，不按source关键词猜版本。继承846修复后的显式选项存在性/缺值/未知值/孤立版本用法检查，覆盖普通运行和--inspect入口，不能把缺值当未提供。场景schema4字段结构不变，精确接受新增版本；旧reader未知v3明确unsupported，不能删除绑定或fallback。

同raw脚本在v1/v2/v3可共用原始Arc/文件，准入/编译凭证/结果API按version及managed身份区分；切版本是内容事务，dirty/Undo/Redo/Save/Open/Verify/SaveAs与来源memory/hash/已确认blob合同保持，失败保旧binding和源码。不改source256KiB/managed64KiB/combined320KiB、8raw/2MiB等预算，不保留三份运行VM或额外私有cache。

### 5.6 两个真实用户项目

新增拟`samples/segment_projects/beam2d/`与`sight3d/`，各以scene4/behavior3绑定正式源码和资产；没有Rust预设命中对象或业务分支。

- Beam2D：Auto从actor world位置及用户输入/私有方向构造有限线段，区分近/远目标、遮挡、miss和终点命中；以预置提示节点位置/可见性反馈选择。反馈节点位于测试线段外，不让结果指示物成为意外候选。Circle边角按AABB语义验收。
- Sight3D：实际多part GLB、非零parent、静态yaw/pitch/scale，Auto最近命中判断视线开关/门或指示；至少一次同callback移动group后cast，再移动另一对象并读取旧token证实快照语义，第二次cast才得到新结果。不是只换维度常量的2D复制。

origin/方向来自当前Auto API/用户状态；本版不提供鼠标屏幕射线或新线条渲染primitive。响应由Auto local/visible写回原consumer，不实现碰撞阻挡。Pause无重复动作、Step一次、Resume不重置app_start；Stop/新Run状态复原。

两项目经正式UI绑定/Save As产物复制到仓外不同cwd，原root/导入path不可用，新进程只--open并明确Run；同一个runner字节SHA验证命中身份、独立t/point/normal参考、反馈节点及全部相关GLB mesh最终像素。只改用户端点/过滤/条件后Update/Save/reopen改变实际行为，Rust/API/bridge未重编译。原v1 Move/Patrol、v2 Zone2D/Zone3D及845来源/临时覆盖分母并列保留。

### 5.7 owner、扫描与交接

消费846同一Descriptor/几何算法身份，不新增几何格式或每frame重新扫描/上传。每次cast从实际Auto world+offset查候选，单槽固定scratch；不得在Rust Host/pick函数按预期结果补命中。未使用segment的v1/v2不创建额外descriptor/result/runtime。

实际20轮v1/v2/v3绑定/Undo/Open/Run/Fault/Stop后source/VM/Descriptor最后owner释放；合法Editor/历史/外部pin保留分别计量。targets/pipelines/mesh Arc与上传计数不变，warm无模型重扫/逐tick读源或GPU readback，报告≤4096。有限全场扫描开销/配置/Auto CPU单列，Iced/驱动驻留与任意用户heap硬配额保持原排除项。

### 规范增量

全部拟议，不写canonical；spatial-query-profile.md先等846沉淀再modify。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/segment-query-profile.md | 无有限cast→候选/闭区间/nearest/normal/数值/单槽token与credits合同 | 定向感知可验收 | AC-02, AC-04, AC-05, AC-07 |
| SD-02 | modify | docs/specs/auto-graphics/spatial-query-profile.md; docs/specs/auto-graphics/behavior-api.md; docs/specs/auto-graphics/behavior-project.md | v1/v2原样→显式v3/profile9/项目绑定，v3 credits与same-raw身份、旧版不变 | 避免reserved与错误凭证复用 | AC-03, AC-05, AC-06 |
| SD-03 | modify | docs/specs/auto-graphics/scene-play-session.md | 既有空间初始化→v3结果槽生命周期/consumer/错误及按脏轴/visibility的共享derive，受信紧凑生成区、四标量dirty元数据（合计60B），原CPU/Stop/数值顺序保持 | 不泄露旧结果或漏派生，不依赖nearest剪枝满足控制 | AC-04, AC-05, AC-07 |
| SD-04 | modify | docs/specs/auto-graphics/auto-source-driver.md; docs/specs/auto-graphics/prototype-profile.md; docs/specs/auto-graphics/project.md; docs/specs/auto-graphics/integration-contract.md | 重叠触发→两个有限视线/光束项目及精度/局限，I2/I3仍后继 | 交付含义准确 | AC-01, AC-06, AC-08, AC-10 |

## 6. 测试设计

当前r3实施已完成指定定向验证与原生/独立消费重放，原始日志见work-r4；不以预研绿替代正式修复验收。实施/internal/final同树顺序，final独立上下文并暂停实现写入，作者自查不冒认最终独立复审。用户未指定模型；本会话申请只读独立复审子代理的明确授权，授权前不创建代理/新聊天。

未来848树driver目录沿现有prepare/verify核最终SDK/锁。拟`tests/segment_api.rs`、`segment_geometry.rs`、`segment_host.rs`、`segment_project.rs`与`tools/check_segment_queries.py`、`check_segment_owners.py`；T-03..08建立真实入口。开发用`cargo test --locked --test <对应单一target> -- --nocapture --test-threads=1`及受影响旧测试、locked all-targets check/fmt；0selected/skip不是通过。

| case_id | AC / work owner | 输入与真实入口 | 可区分observable oracle | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01；T-01/08 | 正式846/SDK/三锁/原闭包与新源，verify/source consumer及原反控 | 正式交付及版本/源fence正确，缺源/pin/资源漂移拒绝，原分母不缩 | 候选不作执行冻结 |
| V-02 | AC-02；T-04 | actual Auto：轴向/斜向、parallel±0、零长、inside/boundary、t1、退化box、±1ULP | hand/f32参考hit/miss/t/normal/point符合闭区间，非epsilon混淆；漏轴/错方向/tie负控红 | AABB非triangle/OBB精确命中 |
| V-03 | AC-02, AC-04；T-04/05 | 多候选不同t、相同t不同ordinal、隐藏/alpha0/parent隐藏、ignore0/−1、unavailable | 遍历顺序/nearest/过滤先后正确；坏匹配候选错误而非miss，合法排除不报错 | 不保证持久ID排序/层mask |
| V-04 | AC-03；T-03 | v1/v2合法ag_cast_*私有helper、v3重定义/未知版本/profile/伪source/base | 实际Parser/SourceSnapshot/Play接受合法/具名拒绝，旧managed身份原样，无fallback | 新名称只在显式v3保留 |
| V-05 | AC-04；T-04/05 | 同callback父/叶写后cast、cast后再移动/读token、第二cast、callback切换/新Run、未查询/坏axis/ID上限 | 最新world参与query，结果快照不漂；旧/错token/过期具名sticky Fault，token不wrap；miss正常可读 | 单槽/当次Run，不是持久安全句柄 |
| V-06 | AC-05；T-02/05 | 128节点宽/深结构，2cast+10get+6AABB；越额/超steps/超时、NaN/±Inf实际producer各流转 | 原预算内控制过，按权重首个超额失败；非finite不能成功hit/miss，错误不被后续调用清掉 | 不把解析失败当运行数值控制 |
| V-07 | AC-05, AC-07；T-05 | 未完成/错序spatial配置、Starting取消/Stop、partial/Fault/修源、Pause/Step/Resume | 配置全好才callback，tick0 Paused/原生命周期正确，保last-good/Stop编辑恢复，旧结果不泄漏 | 无热更新/跨callback查询句柄 |
| V-08 | AC-06；T-03/06 | 同raw v1/v2/v3 Apply/失败/Undo/Redo、temporary source、scene4 Save/Verify/Open/新root/old epoch；--behavior-api缺值/未知/孤立及--inspect组合 | 原始Arc可共但API准入不串；版本切换非noop、坏Open/失败保旧，来源与闭包完整性正确；显式坏选项exit2，合法缺省仍成功 | 无scene5或安装包；继承F-846-R1-01修复 |
| V-09 | AC-06, AC-08；T-06/07 | Beam2D/Sight3D实际Auto输入/方向/遮挡/条件变异，独立预期 | 用户Auto产出正确选择/门/提示，无Rust名称preset；同callback写后查与快照同时成立 | 不新增屏幕unproject/相机控制 |
| V-10 | AC-08；T-07 | 真实OS→Iced/绑定/版本/Run，same binary仓外两项目、new process、GLB实际像素/透明漏part/partial | 正式入口/命中反馈/全mesh/GPU正反与Stop成立；改源改行为无Rust重编，partial非零 | GPU反例必须在实际目标阶段失败 |
| V-11 | AC-07；T-08 | 20轮绑定/Undo/Open/错token Fault/Run/Stop、pin、扫描/上传/槽累积mutations | weak VM/Descriptor/source及GPU最后owner符合；一槽有界，warm不重扫/重传/逐帧读文件，原counter AND content保持 | 外部合法pin与计量排除单列 |
| V-12 | AC-08；T-08 | 精确API/结果/credit表、SD01..04、README/CLI、合入矩阵 | 实际源码/版本/错误/用例与handoff逐项一致，可复跑；原v1/v2分母完整 | 静态文档不代运行验收 |

每case记录精确命令/退出码/断言数、code/SDK/锁/API/bridge/Descriptor算法/用户源/fixture身份及raw报告/像素；独立参考不调用被测slab/读取被测输出生成expected。反控覆盖错nearest/tie/parallel/inside、旧world/旧token、漏预算、v2误保留新名、same-raw错误凭证、source/owner/实际GPU；setup失败不能冒充捕获目标反例。

T-08登记准确完整矩阵；T-09最终独立pass后，T-10在冻结提交执行完整locked driver/AGP test+build、原16 AGP/双游戏、835..846既有原生/owner/reopen/source/producer与fresh own-SDK/own-target consumer、原counter AND content、新segment正反/预算/项目搬迁。work/internal/final只定向/check/fmt，证据复用核代码/依赖/配置/输入未变。848 driver-only不触root t/tv/tt/tb/tu/taa/docs_gen；846根SDK门不能被本计划替代。tf仍主检出单实例到期批量，848收尾记录实际判定。

## 7. 验收标准

| ID | owner_stage | 可观察标准 |
|---|---|---|
| AC-01 | work | G846/GSOURCE、唯一树/固定依赖/原分母与只读边界成立，V-01 |
| AC-02 | work | actual Auto closed slab/nearest/tie/过滤/数值与独立参考一致，V-02/03 |
| AC-03 | work | 显式v3/profile9来源/准入/新reserved、v1/v2语义与managed身份不漂，V-04 |
| AC-04 | work | 最新world查询、瞬时结果/token正确及有界过期/溢出错误，V-03/05 |
| AC-05 | work | 128节点/32credits与原CPU、非finite/sticky/whole-frame/Fault/Starting/Stop闭合，V-06/07 |
| AC-06 | work | 正式UI/CLI及版本化绑定/历史/闭包/来源、两真实Auto用户交互项目成立，V-08/09 |
| AC-07 | work | 单槽/Descriptor/source/VM/GPU最后owner与扫描上传/原生命周期保持，V-07/11 |
| AC-08 | work | 同binary仓外/真实OS/GPU正反/partial及SD/API/精度/credit文档交接齐备，V-09/10/12 |
| AC-09 | review | 独立核AC-01..08/Spec、遗漏/延后/workaround/健康，revision/code绑定pass，完整merge门明确pending |
| AC-10 | merge | 冻结完整门过才落autographics-dev，Spec/ledger/归档/guard清理及到期齐，图形源码不入master |

## 8. 执行步骤

当前r3新方案已先通过完整预研控制，再更新本计划并正式实施；F-848-R1-01工作修复通过，待独立复审确认。T-01..08完成，execution_done8/10；T-09最终pass为reviewed9/10且完整门pending；T-10真实收尾才交付10/10。

| task_id | owner_stage | 依赖 | 触点／拟新增路径与动作 | 关闭条件；AC |
|---|---|---|---|---|
| T-01 | work | G846/GSTART | 正式846收据/Specs、唯一848树、SDK/三锁/原source闭包和分母 | [x] 完整846及纯修复来源可消费，重核正式SHA/身份冻结；AC-01|
| T-02 | work | T-01 | Api/SourceSnapshot/Descriptor/Play，new research/segment-query-feasibility.md | [x] r3按脏轴/shared derive及受信布局方案先验证成立；60组2D/3D宽深、各单轴写与原反例/三轴斜向32credits控制通过，补调试行冗余变异失败对照；AC-02, AC-03, AC-04, AC-05|
| T-03 | work | T-02 | behavior.rs/behavior_project.rs/正式版本选择，新segment_api/project tests | [x] v3构造/准入/Binding/同raw凭证/Undo，V-04/08；旧v1/v2 source/reserved/身份保持；AC-03, AC-06|
| T-04 | work | T-03 | 新api/behavior_segment_v3.at、new segment_geometry.rs；只读复用spatial描述 | [x] r3最终胜者物化/方向预选与显式float有限性通过API8/geometry6及预算oracle，保留9/12原错误分类、underflow/XYZ/nearest/token与独立参考/变异；AC-02, AC-04|
| T-05 | work | T-04 | Play profile9/callback begin-end/result token/weighted errors/consumer，新segment_host tests | [x] 原R1完整输入正式Host3分布均Paused/tick1，另6组实际Cylinder独立结果oracle，原Host4/宽深/配置/Stop恢复通过；新mixed dirty/visibility/move/final-field及缺标记变异闭合；AC-04, AC-05, AC-07|
| T-06 | work | T-05 | Host/Panel/bin正式v3选择、samples/segment_projects/beam2d与sight3d | [x] V-08/09与旧v1/v2/项目分母定向通过，callback操作与实际Auto业务完整；AC-06|
| T-07 | work | T-06 | 新check_segment_queries.py/OS/GPU/仓外new-process回放 | [x] V-09/10同binary/全部mesh反馈/条件变异/Stop/partial正反无gap；AC-08|
| T-08 | work | T-07 | 新check_segment_owners.py/source/fresh consumer/README/new research/segment-query-handoff.md/Spec proposal | [x] 373源/固定SDK/锁闭包与四反控、独立SDK/target消费12相关测试、4原生+2重开，slot+dirty60B及实际SDK存储、heap/owner未变边界、提案/完整门216分母及readiness齐；AC-01, AC-07, AC-08|
| T-09 | review | T-08 | 同树冻结code/Plan/Spec与raw，主检出独立review附件 | AC-01..08逐项/遗漏/延后/workaround/健康、版本绑定结论；AC-09 |
| T-10 | merge | T-09 pass | 新冻结完整门→autographics-dev；主检出知识/ledger/archive/guard/到期 | AC-10真实交付/清理，改代码先受影响复审再冻结，不提前冒领 |

## 9. 复审记录

2026-10-25：stage:new；PLAN-848 revision1；outcome:blocked（G846完整交付pending，草案可评审）；next:846先修复F-846-R1-01并完成有效最终独立review及冻结完整门/落地/知识与清理，再由后续work指令启动848 T-01冻结正式基面，T-02做有界可行性。drafting0/10；本轮无实现/建树/prepare/build/Cargo/原生/GPU执行，无agent/chat。

静态合同/source检查见[preflight](attachments/848/planning/preflight.md)。所有AC/V/SD是拟议未执行，不修改846/845/840/841或Schema/ACE/HMR原范围；一份近程草案，不预设下一份扫掠的编号/月份。若上游收尾状态变动，T-01以正式收据刷新。重复同因不收敛先诊断邻接路径，不通过改fixture/减分母/扩大容差隐藏支持缺口。


### 2026-10-25 work:r1 — 正式启动与 T-01 冻结

用户明确授权实施848，GSTART成立。846 r3已archived/delivered11/11，正式graphics bf22db3465203f3babd649494f4b37aa2a012027、root d369dd760、ledger fcea7d8dc，三个WT guard clean后已清理；历史草案中的needs_fix/pending观察由本记录更新。846按用户r3只对增量编译性能暂缓验收，功能/数值/运行时安全不变；批量due/B845-FEATURE-01由847承接，不宣称全局批量绿。

[x] T-01：唯一工作树 D:/autostack/.wt/lang-848/auto-lang，plan-848-dev，显式以正式846 graphics完整SHA建树；根路径/分支已断言。原681 SDK文件/17 local/29 edges/479 metadata闭包与345 source hashes通过，四个source/pin/lock/asset拒绝反控和恢复正控通过。三锁及准确旧分母见[冻结记录](attachments/848/work-r1/frozen-source.json.txt)。未升级SDK/锁或借用他人WIP；主检出既有文档及未跟踪fixture锁保留。

stage:work | plan_id:PLAN-848 | plan_revision:1 | outcome:executing | worktree_path:D:/autostack/.wt/lang-848/auto-lang | branch:plan-848-dev | base_commit/code_commit:bf22db3465203f3babd649494f4b37aa2a012027 | dependency_revisions:SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 | task_ids:T-01 closed,T-02 next | blockers:none | next_responsible_role:work。当前1/10，T-02..08未完成，最终review及merge完整门pending。


### 2026-10-25 work:r1 — T-02 needs_replan

stage:work | plan_id:PLAN-848 | plan_revision:1 | outcome:needs_replan | worktree_path:D:/autostack/.wt/lang-848/auto-lang | branch:plan-848-dev | base_commit:bf22db3465203f3babd649494f4b37aa2a012027 | code_commit:5efe91a8a0cead18288215d74aa2d339af2e26f9 | dependency_revisions:SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、三锁保持T-01 | task_ids:T-01 closed,T-02 incomplete,T-03..10 pending | next_responsible_role:new（有界修订T-02，随后work沿原树续行）。status executing、1/10，不宣称work pass。

两个允许候选：A轴循环/共享helper；B内联finite与box edge降低调用开销。两者实际Auto128叶子、2cast+10get+6AABB均在_ag_tick触发原100000累计步上限；不是解析/初始化失败。最终同文件3测=2pass/1fail，exit101；2节点nearest/tie/t=.25/point=-1/normal=-1/移动后快照与16节点同负载正控通过。测试负载比完整128宽/深双parent写更轻，已足以否决当前方案；不推断所有Auto实现均不可行。保留源码候选A及[原始失败](attachments/848/work-r1/candidate-a.log.txt)、[B失败](attachments/848/work-r1/candidate-b.log.txt)、[最终三测](attachments/848/work-r1/final-scoped.log.txt)、[16节点正控](attachments/848/work-r1/candidate-b-small-control.log.txt)。初始探针setup错误已修正后才计运行失败。

固定SDK原有146 warnings保留，不改依赖；rustfmt单文件与diff whitespace通过。已提交三份本Plan探针/研究材料，树clean；未实现公开v3准入、GPU/GLB/Stop、深层及数值/token完整控制，未执行候选tie变异，不将它们记pass。无整档、无canonical/live ledger更新、无合入/清树。具体风险及下步在工作树research/segment-query-feasibility.md。按§5.1最多两方案仍失败，不能擅自加预算/减128/写SDK，先回new有界修订计算路径；原支持与安全合同保持。

### 2026-10-25 new:r2 — 有界修订通过，next work

stage:new | plan_id:PLAN-848 | plan_revision:2 | outcome:pass | next:work，沿原唯一树plan-848-dev继续T-02..08。用户授权直接修订本计划，先预研再更新后实施；未申请新编号、SDK/AGP写面或改节点/预算/数值支持。修改§0/3/4当前来源、§5.1局部方案及阶段依赖、T-02关闭条件；AC-01..10、V-01..12和SD-01..04交付责任保留，SD-03覆盖v3 dirty末尾与共享derive等价特化。T-01源码/SDK/三锁证据保留；r1失败仍在历史附件。公开v3/consumer/GPU等未执行证据不能沿用或宣称通过。

新方案预研[6测结果](attachments/848/work-r1/candidate-parent-cache.log.txt)、[选定Auto候选](attachments/848/work-r1/selected-candidate.at.txt)，其余探索和诊断日志按原始失败保留；仅证明下一实施入口可行，不是最终work/review pass。current_step保持1/10。下一步将探针转为版本隔离的真实v3实现，固化cache/dirty/几何/剪枝正反，并完成其余既定工作。

### 2026-10-25 work:r2 — pass / execution_done8/10

用户要求的顺序已完成：先在原848树找出并验证新方案，再将848修订为r2，之后实施。未另建Plan。实现提交83d313212、数值收口aaab538a6、实际consumer字段反控d3032163e及SDK存储计量cef25b5c4均留在plan-848-dev。当前冻结代码 **cef25b5c41dbbfbec6b7bfd4b13507d8d5f1e34d**；工作树clean，实施写入暂停，next review。

[x] T-02：r2轴展开/完整几何先验/安全nearest剪枝及单次derive父坐标cache成立，dirty末尾不重复推导；实际128宽/深、两次parent写、2cast+10get+6AABB守原100000steps/120resumes/250ms和32credits。SourceSnapshot/Host/原生整链补齐，不以预研替代正式v3验证。

[x] T-03：显式behavior/3/profile9接SourceSnapshot、Binding、CodePayload、Host、正式UI/CLI；旧v1/v2 API/ENTRY/transport输入和保留名语义保持，同raw版本事务与坏快照/未知版本/缺值/孤立选项均有实际反例与正控。

[x] T-04：实际Auto closed slab、nearest/ordinal/XYZ、过滤、miss/inside/endpoint/parallel/退化/ULP/all-axis normals通过独立预期。有限geometry在nearest剪枝前验证，实际NaN/±Inf经参数/局部/返回/容器流转拒绝。数值收口发现正fraction下溢0时丢进入面；保留[pre-fix red](attachments/848/work-r1/underflow-red.log.txt)，已修2D/3D并保留boundary正常0法线控制，不加epsilon或降支持。

[x] T-05：单槽token callback begin/end与下一cast尝试失效、查询后移动的旧快照/新查询、sticky-first/32credits/序列上限、原完整配置门和Starting Stop/last-good/Fault/修源恢复均通过。最终world读取使用Play实际消费的非派生`_ag_field`，移除final derive的变异会被抓住，不由ag_world自修复掩盖。

[x] T-06：Beam2D与Sight3D由用户Auto实现有限光束/视线最近命中、反馈及同callback组移动/快照/第二次查询；3D消费非零parent/静态旋转缩放的真实多part GLB。正式绑定/版本/Save/Save As/Undo/Redo/Open来源与角色合同保留；只改用户reach即可Update/保存/重开改变选择/结果，Rust不注入业务。

[x] T-07：同一个已保存的实际runner，14具名native cases=6正控/8预期拒绝，另4个new-process reopens；两项目均从正式UI Bind/Save As后搬迁，旧路径移走再重开。实际OS输入/Pause/Step/Resume/Stop、独立几何和每part最终像素通过；partial、透明、漏part、错资产、缺/坏source、坏API、缺GLB均在预期阶段拒绝。最终权威为[固定产物重放](attachments/848/work-r1/native-preserved-replay/results.json)，runner SHA `075bab84dee36195e0ecd05becdc46325faf46a9bb675cb7792eb6d264bbc7dd`，保留于工作树target/segment-verified-runner。一次真实鼠标移出触发auto-Pause的中断如实留档，安静输入重放完整通过；未改Pause政策。旧v2 Zone2D/Zone3D原生正控也通过。

[x] T-08：370 source/asset hashes、681 SDK文件/17本地包/29 edges/479 metadata闭包及三锁保持；新源/陈旧pin/锁表/GLB四反控+恢复正控通过，源包用独立SDK与target实际消费。39个去重定向检查（30新增+8原v2控制+1共享配置门）与相应反控通过；新consumer14例及最终字段/SDK存储定向重放通过。20轮v1/v2/v3历史/Open/Run/Fault/Stop释放VM/Descriptor/source最后owner，GPU Arc/target/上传/扫描不退化，5个保留/重扫/上传错误变异被抓住后恢复正控通过。64次查询heap对象数不增长。11标量结果逻辑44B，实测NanoValue88B/String结构264B/键容量138B；共享map/分配器等边界单列，未伪称总硬额度。fmt、类型检查和diff whitespace通过；固定只读SDK原有146 warnings保持、driver无新增警告。

早期naive预算失败、探针setup/float-abs便利调用错误、合法远目标遮挡、初次空白行诊断及临时debug产物哈希更换均保留为历史，不计作通过。Sight3D远目标沿原查询方向改至x6防合法遮挡，全部part/pixel判据和业务/数值支持未弱化。最终代码/证据及逐AC映射见[readiness](attachments/848/work-r1/readiness.json.txt)；研究/规范提案位于工作树research/segment-query-{feasibility,handoff,spec-delta}.md。

AC-01..08 work-owned结果pass；open in-contract findings、required coverage gaps、unresolved decisions均为空。测试复用按相关代码/SDK/配置/输入未变确认；下溢、非派生consumer字段和SDK存储断言均在当前相关producer/consumer代码上重放。T-09独立复审尚未执行，作者自查不冒认独立pass。T-10完整门明确merge-owned pending：预计完整driver208（原178+新增30，须merge实际选择清单核数）、AGP163/16 native、原87图形及原spatial/Step/project/source/owner矩阵、producer/fresh consumer与新segment矩阵。通过后才可落autographics-dev/沉淀Spec与ledger/归档/guard清理/判定批量到期。未跑根t/tv/tt/tb/tu/taa/docs_gen/tf、未合入或发布规范、未清树。主检出他人文档簿记及fixture锁WIP保留。

stage:work | plan_id:PLAN-848 | plan_revision:2 | outcome:pass | worktree_path:D:/autostack/.wt/lang-848/auto-lang | branch:plan-848-dev | base_commit:bf22db3465203f3babd649494f4b37aa2a012027 | code_commit:cef25b5c41dbbfbec6b7bfd4b13507d8d5f1e34d | dependency_revisions:SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、T-01三锁/AGP未变 | task_ids:T-01..08 closed | evidence:readiness及具名原始结果 | blockers:none | next_responsible_role:review。

### 2026-10-25 review:r1 — needs_fix / executing5/10

stage:review | review_scope:final | reviewer_identity:当前Codex会话（含实施上下文，非全新独立会话；停止实现写入、从合同/代码/raw重建判断，不冒认最终独立pass） | plan_id:PLAN-848 | plan_revision:2 | outcome:needs_fix | worktree_path:D:/autostack/.wt/lang-848/auto-lang | branch:plan-848-dev | reviewed_commit:cef25b5c41dbbfbec6b7bfd4b13507d8d5f1e34d | base_commit:bf22db3465203f3babd649494f4b37aa2a012027 | dependency_revisions:SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467，AGP/三锁未变 | spec_inputs:review-r1/baseline.json.txt及冻结spec-delta.md | acceptance_results:AC01/02数值/03/04/06/07保留通过，AC05失败，AC08部分，AC09未通过，AC10 pending | findings:F-848-R1-01 P1 OPEN | next:work同树修复后重新送审。

[逐项报告、可复用反例与关闭范围](attachments/848/review-r1/review.md)。actual Session和正式Host均验证：1group+127合法3D叶子、双group写、2cast+10getter+6AABB，在晚Z轴miss或ordinal逐个更近时触发100000累计步上限；等深对照成功tick1。正式Host两反例均Faulted/tick0，非probe-only/初始化错误或计时噪声。当前nearest剪枝收益依赖候选分布，原均匀box绿不能证明r2完整128支持。T02/T05/T08重开，保留其余已验证任务与原始历史；readiness旧无gap声明由本记录替代，不弱化数值/预算/节点合同。

本轮API8+geometry6定向重放通过，source370/SDK681/17local/29edges/479metadata、固定runner075bab…、14原生raw报告/4重开/5owner反控与SD01..04逐项复核，fmt及diff whitespace通过；原生/GPU/20轮owner按未变相关代码/输入复用，未再跑整档。完整driver/AGP/原图形与spatial/native/owner/source及producer/fresh consumer门仍merge-owned pending，最终审查通过前不启动。未改产品/SDK/锁/canonical/live ledger、未合入/归档/清树。临时review探针归档后删除，实施树HEAD及clean复核无漂移。

### 2026-10-25 work:r3 — 方案调查 needs_replan，未采用修复

用户授权修复F-848-R1-01，明确先找到能守原预算的新方案，再更新当前计划并实施，完成后复审、先不合入。原唯一树/分支及冻结HEAD cef25b5c41dbbfbec6b7bfd4b13507d8d5f1e34d核对clean；主检出tracked代码路径无WIP，其他会话文档簿记保留。

本轮先重放R1 actual Session/正式Host原例，失败及等深正控保持。随后在临时managed-source探针比较结果延后物化、局部别名/原位数组/循环、推导融合及受限按轴更新等方案；[完整诊断和原始源码/日志](attachments/848/work-r3/diagnosis.md)。B等普通局部简化仍不能通过逐个变近；H/I/J/K的X特化在原轴向反例上可过，但追加的正/负三轴斜向同负载仍100000步耗尽，且尚未实现通用dirty轴/visibility。它们不构成合格方案，不能只取通过的原例关闭finding，也不能宣称r3产品设计成立。

因此仅更新调查/交接证据，**没有改语义合同或提升plan_revision，没有产品修复提交**；工作树仍cef25b5c4/clean。临时探针已删除，实际Auto编译输入及所有探索保存到附件；370源/681SDK/17local/29edges/479metadata再次核验通过。没有运行完整测试档或新最终复审，未合入/归档/清树、未改SDK/AGP/三锁/规范台账。没有证明固定SDK内所有方案都不可能；下一设计入口必须先解决无法剪枝的完整扫描与图推导合计预算，而不是继续依赖均匀box。保持原128/32credits/CPU与数值/资源要求，后续若需要扩SDK等边界，须具体回new决策。

stage:work | plan_id:PLAN-848 | plan_revision:2 | outcome:needs_replan | worktree_path:D:/autostack/.wt/lang-848/auto-lang | branch:plan-848-dev | base_commit:bf22db3465203f3babd649494f4b37aa2a012027 | code_commit:cef25b5c41dbbfbec6b7bfd4b13507d8d5f1e34d | dependency_revisions:SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、三锁/AGP未变 | task_ids:T02/T05/T08仍未关闭，executing5/10 | evidence:work-r3/diagnosis.md | blockers:F-848-R1-01尚无满足完整控制的可采用方案 | next_responsible_role:new（同一848有界设计调查，不另取号、不擅自减验收）。

### 2026-10-25 new:r3 — 先验证方案后修订，next work

stage:new | plan_id:PLAN-848 | plan_revision:3 | outcome:pass | base/code:bf22db346/cef25b5c4 | dependency:固定SDK7b987、三锁/AGP未变 | next:work沿原唯一树实施r3，不另建Plan。用户授权范围与128节点/32credits/CPU、数值/来源/生命周期/owner等AC保持；仅修订§5.1有界计算设计和受影响任务。预研候选已实际通过60组Session+6组正式Host；[验证日志](attachments/848/work-r4/full-controls-fixed.log.txt)、[受信组合候选](attachments/848/work-r4/behavior-validated.rs.txt)、[Auto候选](attachments/848/work-r4/api-validated.at.txt)保存。原产品文件已恢复后才更新计划，尚未宣称正式修复/独立review pass。T02/T04/T05/T08待正式验证，current4/10，merge全门pending。

### 2026-10-25 work:r4 — pass / execution_done8/10，独立复审待执行

按用户顺序完成：原树找到方案、先实际验证60组Session+6组正式Host控制，保存/恢复候选产品文件后修订当前Plan为r3，再正式实施。冻结提交 **6bc5abc6080b2261a43f55a54f127cd779239edb**，树clean；四个dirty标量+原结果槽合计15/60B，SDK/AGP/三锁与原CPU/128/32credits均不变。SDK原SourceLine也收费；只压紧受信v3生成代码，原计数/编译器不改。按轴推导复用shared原公式，原位更新原数组，不引入第二图/cache。

F-848-R1-01 work-fixed：原始R1 source/setup完整重放（只把预期Fault断言改为成功）均通过；60组控制覆盖2D/3D、宽/深128、每个写入轴、等深/晚轴miss/逐个变近/正负及非等比例斜向，精确32credits并断言hit/node/t/point.x/normal.x；正式Host6组真实Cylinder也成功tick1。宽3D斜向观察97yield只是调度次数，不宣称精确步数或普遍余量。恢复冗余调试行的实际变异重现100000预算失败，合法控制成功，证明布局变化不是计数绕过。混合dirty/visibility/三轴move、父切换、最终非派生字段、f32顺序、用户原始字节、缺标记/错误parent/nearest变异均有区分断言。

本轮合计33个去重定向检查通过（不把60子例/6Host当新增测试数），独立own-SDK/own-target消费12测试通过；现有消费源按新373闭包刷新，固定681 SDK/17local/29edges/479metadata与独立target再次核实，复用其不变依赖缓存，不冒称重新空缓存全编。API8/geometry6保留数值、9/12错误、token、underflow/XYZ边界；一次候选finite比较推断导致错误12而非9已由原断言抓住，显式float边界/商修正后原断言未弱化。初次storage精确滤串0selected和临时Rust分号错误均留原始日志、不计pass；最终storage实测flags16B逻辑/32B值/96B键结构/56B键容量，与原44B结果槽分开；64次查询heap无增长、原V1/V2没有dirty/result增量。空白清理前的diff错误已修正；生成API逐行trim不受其影响，最终fmt/diff/check/source通过、无新增driver warning（SDK146预存）。

固定runner **7ba32aa93a71433e1a1774b58fc704b4d9e5417ed353cc94c7cdfa826cce972a** 保存在target/segment-r3-verified-runner；本轮4原生正控（两项目及短源）+2新进程重开成功，实际OS/Pause/Step/Resume/Stop、每part像素与用户改源行为均由原严格oracle验证。未变CLI/Binding/SaveAs、GPU负控与owner机制等按R1代码/配置/输入映射复用；不重复完整旧矩阵。完整driver预期216（178原+38新增）须merge实际核数，不冒领全包已过；完整driver/AGP/原图形native-owner-source/consumer及批量到期等仍merge-owned pending。

证据：[readiness](attachments/848/work-r4/readiness.json.txt)、work-r4具名原始日志、[冻结Spec提案](attachments/848/work-r4/spec-delta.md)。所有work-owned AC通过，开放work finding/gap/产品决策为空；F01待独立review确认关闭，未宣称最终reviewed。已申请用户明确授权只读独立子代理；未授权前不建代理/新chat。未合入/规范沉淀/归档/清树。

stage:work | plan_id:PLAN-848 | plan_revision:3 | outcome:pass | worktree_path:D:/autostack/.wt/lang-848/auto-lang | branch:plan-848-dev | base_commit:bf22db3465203f3babd649494f4b37aa2a012027 | code_commit:6bc5abc6080b2261a43f55a54f127cd779239edb | dependency_revisions:SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、原三锁/AGP | task_ids:T01..08 closed,T09/T10 pending | evidence:work-r4/readiness | blockers:none in work | next_responsible_role:review。

### 2026-10-25 review:r2 internal — pass，独立final仍pending

stage:review | review_scope:internal | reviewer_identity:当前实施会话，明确上下文限制 | plan_id:PLAN-848 | plan_revision:3 | outcome:pass | worktree_path:D:/autostack/.wt/lang-848/auto-lang | branch:plan-848-dev | reviewed_commit:6bc5abc6080b2261a43f55a54f127cd779239edb | base_commit:bf22db3465203f3babd649494f4b37aa2a012027 | dependency_revisions:固定SDK7b987/三锁/AGP | acceptance_results:AC01..08内部通过，AC09独立final pending，AC10 merge pending | findings:F-848-R1-01内部确认修复，待独立关闭 | next:获明确授权后独立只读复审或新独立会话。

[受影响复核报告](attachments/848/review-r2-internal/review.md)逐项核原R1输入、60/6预算oracle、shared derive/dirty/布局/数值/生命周期/资源及Spec增量，复用未变兼容/项目/负控证据有明确边界；实现提交/clean/最终源四反控再核无漂移。无新work finding，不跑全包/整档。当前含实施上下文，内部pass不等于最终独立pass，保持execution_done8/10，未合入。

### 2026-10-25 review:r3 final — pass / reviewed9/10

stage:review | review_scope:final | reviewer_identity:用户明确授权的独立子代理/root/review_848_r3，全新上下文、未参与实施 | plan_id:PLAN-848 | plan_revision:3 | outcome:pass | worktree_path:D:/autostack/.wt/lang-848/auto-lang | branch:plan-848-dev | reviewed_commit:6bc5abc6080b2261a43f55a54f127cd779239edb | base_commit:bf22db3465203f3babd649494f4b37aa2a012027 | dependency_revisions:SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、原三锁/AGP未变 | spec_inputs:review-r3-final/baseline.json.txt，冻结spec-delta SHA256 a07b8bff14af5359d9eef139da1d4204d3e0a9bca907a541b57005800529d89a | acceptance_results:AC01..09 pass，AC10 pending | findings:F-848-R1-01 CLOSED，无新阻塞 | pending_closeout_gates:T10完整driver/AGP/原native-owner-source/producer-fresh consumer等统一门 | next:merge阶段，本轮按用户要求不合入。

[独立最终报告](attachments/848/review-r3-final/review.md)逐项绑定AC/任务/代码及证据，18个具名测试通过，60预算子例/6Host不虚增测试数；另选非零起点、正负混合斜向、两查询切换X→Z脏轴、nearest127/Y法线与point.y的128节点独立oracle通过。原R1 exact Session/Host、dirty/parent/f32/finite/token/旧身份定向控制均通过，98yield仅为切片观察，不伪称精确步数或普遍余量。原128节点、32credits和CPU合同保持。

T09关闭，Plan状态reviewed9/10。原生/GPU/owner未变机制按报告明确边界复用原始证据，当前r3正控用当前runner补齐；未跑整档/全包/native完整矩阵。父代理复核报告、实施HEAD/clean后单写计划簿记；实施保持冻结，canonical Specs/live ledger未发布，T10及全部merge-owned门仍pending。最终pass不等于交付完成，任何后续代码/依赖/配置/输入漂移须受影响复审，未合入/归档/清树。本记录替代上方历史的授权待答复、独立final pending及F01待确认状态。

## 10. 待澄清事项

当前无新的产品选择待用户决定；用户授权的独立最终复审已pass，F-848-R1-01关闭，reviewed9/10。有限world线段/AABB最近命中、single-slot callback token、显式v3、128节点/weighted32credits及原CPU合同保持。T10完整统一门与实际交付仍pending，本轮暂不合入。若实际仍需要SDK/AGP/生产写面、提高CPU/降低节点保证或扩其他非目标，回new具体修订，不默认扩面实施。
