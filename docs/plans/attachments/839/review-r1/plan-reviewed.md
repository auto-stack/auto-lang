---
plan_id: PLAN-839
status: execution_done
feature_name: AutoGraphics场景运行模式与Auto最小行为回路
plan_revision: 1
author: [Codex]
created_at: 2026-10-23
updated_at: 2026-10-24
current_step: 8
total_steps: 10
depends_on: [PLAN-838]
supersedes_spec_components:
  - docs/specs/auto-graphics/authoring-session.md
  - docs/specs/auto-graphics/scene-document.md
  - docs/specs/auto-graphics/scene-hierarchy.md
  - docs/specs/auto-graphics/scene-assets.md
  - docs/specs/auto-graphics/viewport-interaction.md
  - docs/specs/auto-graphics/auto-source-driver.md
  - docs/specs/auto-graphics/prototype-profile.md
  - docs/specs/auto-graphics/project.md
  - docs/specs/auto-graphics/integration-contract.md
new_spec_components:
  - docs/specs/auto-graphics/scene-play-session.md
touched_goals: [GOAL-007, GOAL-008]
affects: [auto-graphics, authoring-experimental, scene-play-session]
---

# [PLAN-839] AutoGraphics 场景运行模式与 Auto 最小行为回路

## 0. 变更摘要

2026-10-24实施完成并交接：`e58828ca64b7e26ce984faf850fb3b1d0887517f`，execution_done8/10；同树保留且clean，实施写入暂停。工作AC01..08/V01..11通过，独立复审与完整merge门仍pending，未合入/沉淀/归档。见[work readiness](attachments/839/work-r1/readiness.md)与[结构收据](attachments/839/work-r1/receipt.json.txt)。以下规划观察保留为历史。

838提供自有静态GLB和可携带场景后，下一切片把场景从编辑状态推进到可运行：**编辑／导入场景 → Run → 真实Auto按固定步长响应键盘并移动对象 → Pause／单步 → Stop恢复编辑内容**。

首版采用随程序交付的一个明确Auto行为样例和新实验profile6。用户在UI中选择2D/3D操作对象，实际场景/导入资源直接进入运行副本；不新建一个与当前文档无关的固定游戏窗口，不用Rust动画冒充Auto。自定义行为编辑器、任意脚本文件加载/绑定与可携带脚本包不在本Plan。

规划阶段历史：本轮只规划。观察时838为**executing2/10**，T-01/T-02完成、T-03在途；正式集成基面为837交付`cf2e99f50ee2a70b70ca2cfc23a5af56b405638b`，尚无838正式交付。源码调查只读已提交基面，838合同/探针为拟交接依据，不把WIP或将来接口当成已交付。G838成立前不建839树、不准备/实施或跑探针。

## 1. 目标

1. 在既有authoring窗口提供Run、Pause/Resume、Single Step、Stop及可观察状态/实际Auto tick。Run以当前accepted Document和资源快照起步，支持未保存但已接受的编辑/导入资源，不强制先保存。
2. 运行保持一个独立真实Auto Session，按1/60逻辑秒推进；至少真实2D叶子和838导入的3D Model能在各自canvas焦点下受键盘控制，层级/资源/画面来自当前场景。
3. accepted文档、选择、revision、dirty、undo/redo及磁盘字节在运行中保持原值。Stop恢复运行前编辑视图，再Run从原编辑快照开始，不能把末帧写回文档。
4. Pause/Step在完整tick边界工作；帧率变化不改给定tick/action trace结果，过载不无限补帧。错输出/预算/资源失败关闭失效VM，保last-good运行帧；Stop和新Run仍可恢复。
5. 输入只送实际焦点canvas，Iced文本/运行控件与游戏输入隔离，失焦/取消/暂停清除held和边沿；最后owner/资源复用有真实证据。

支持面沿用838的Windows单窗stock Iced/wgpu、固定视口/相机、128组+叶子、受限GLB、512潜在mesh实例及全部资源/历史/文件预算。不扩资源格式、渲染器、group平移继承或语言SDK。

非目标：自定义脚本编辑/动态加载/脚本随场景保存、通用组件/事件系统、热重载、物理/碰撞/音频扩面、动态创建删除节点/换父/换资源、运行改变旋转/缩放/材质、运行结果Apply到编辑文档、多运行世界/多窗口/录制产品、生产AutoUI/RQHost/正式Forge、ACE/AutoShader/CAP或新平台。原Breakout/Squash继续按原分母验收。

## 2. 架构方案

```text
accepted Editor Document + 838已接受资产 + 编辑呈现/选择快照
  → Idle准入 → Starting（私有RunId、base epoch/revision、资源/源指纹）
  → 完整局部图配置 → 新真实Auto Session / profile6
  → Running：固定tick + 实际焦点动作快照
  → Auto修改运行局部translation/visible → Auto重算world/effective
  → 全帧身份/数值/静态资源/层级准入 → prepare → 发布运行帧
  → Pause/Step在完整帧边界；故障保last-good并关闭运行VM
  → Stop撤销运行owner/输入/时钟 → 恢复原编辑呈现（Editor未变）
```

L1 driver-local模块扩展，写面仅`experimental/autographics-auto-driver/`。新增拟议`src/authoring/play.rs`、`samples/authoring_play/main.at`及定向tests/probe；AGP、838资源接缝、SDK、root workspace/lock及生产crates只读。复用实际Auto Session/renderer/输入，不引入另一套VM、世界/ECS或游戏编辑器框架。需要核心协议/渲染架构/生产写面时回new并按L2先补正式设计。

| 门 | 成立条件 / 当前状态 | owner_stage |
|---|---|---|
| G838 | 838最终review、冻结完整门、实际合autographics-dev、Spec/归档/guard清理及到期检查全部完成；已核838 archived10/10、完整门/实际交付37fd1a9cf/台账75/guard移树/batch not-due全部完成 | work T-01已核 |
| GSOURCE | 已固定正式37fd1a9cfe22d3d3471202ad4a8dbfe2103f9a5c、SDK9576b4d/680、两锁、source264/closure479/17local/29edges；profile5/schema3/Registry/staged资源接缝及原生CLI已核 | work T-01 |
| GSTART | 2026-10-24用户明确“计划839…实施”，G838满足后启动work；未自动授权review/merge或新代理 | work启动前 |

唯一未来树`D:/autostack/.wt/lang-839/auto-lang`、分支`plan-839-dev`，显式从G838后的autographics-dev完整SHA创建，不用默认master基面捷径。实施/internal/final review共用同树、顺序执行；Plan/附件在主检出。代码审毕合autographics-dev，master仅知识；禁止junction/symlink，移树前wt-guard。

## 3. 技术栈

沿用838最终冻结版本的Rust/stock Iced/wgpu/winit、公开SDK/Session、CPU slice及资源/模型接口；当前观察SDK为`9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`。不更新SDK以便利新模式。

已有Script/Publication/Game host提供保留VM、固定tick、完整输出准入、输入焦点与故障生命周期的证据；只复用适用机制，不复用旧游戏slot/score/event含义来装层级文档。profile6显式区别于1/2原游戏、3/4旧authoring、838拟profile5资源authoring。文档仍采用838正式交付schema（拟v3），本Plan不加持久字段或新schema版本。

## 4. 需求分析与背景调查

### 4.1 授权和来源

2026-10-23用户要求“请继续规划下一个计划”，承接838提前规划。授权只调查/下一Plan草案；无839实施、合838、新聊天/agent/模型/预算授权。不在本轮修改838的在途合同、代码、canonical或其他会话WIP。

已读取[全局overview](../specs/overview.md)、[图形overview](../specs/auto-graphics/overview.md)、[project](../specs/auto-graphics/project.md)、[Auto入口](../specs/auto-graphics/auto-source-driver.md)、[3D游戏profile](../specs/auto-graphics/auto-3d-game-profile.md)、[authoring-session](../specs/auto-graphics/authoring-session.md)、[scene-document](../specs/auto-graphics/scene-document.md)、[scene-hierarchy](../specs/auto-graphics/scene-hierarchy.md)、[集成边界](../specs/auto-graphics/integration-contract.md)、[创作策略](../design/strategy/autographics-forge-v06.md)及[838交付Plan](archive/838-autographics-glb-import-and-portable-scene-assets.md)。新scene-assets在838交付前仍是proposed，不能用本草案沉淀为current。

来源定位/工作区文档指纹见[观察清单](attachments/839/planning/source-observation.json.txt)。源码观察绑定837已交付Git对象，不读改838在途Rust来编造稳定接口；838资源registry/prepare/profile5的实际符号须T-01/02重核。

### 4.2 已观察事实与实际缺口

| 触点（观察SHA cf2e99f50） | 已观察行为 | 本Plan约束/新增 |
|---|---|---|
| driver src/lib.rs::Session::{new,start,pump,close} | 保留线程内VM/task，真实CPU slice，返回int/f32检查；每callback100000 steps/120 resume/250ms，slice1000steps/2ms/64轮询 | 保留一个runtime VM，不每tick重新编译；事件循环不调用同步call忙等 |
| script.rs::Script::{tick,enqueue,pump,close} | 原游戏保留VM、tick去重、完整输出/失败关闭、8ms pump；语义含旧game events | 复用调度/错误经验，建立明确层级运行profile，不冒用旧game协议 |
| authoring/preview.rs::Building | 每编辑候选新VM，静态全图派生+output准入 | 编辑preview不能直接当持续运行session；新增Play owner但复用实际renderer |
| authoring/session.rs::Editor | 文档/history/saved hash、私有preview并原子提交 | runtime不得调用Editor.begin/commit把每tick变成编辑历史 |
| authoring/host.rs::Panel/Host::frame/key/pointer_* | 编辑gesture/树/Iced焦点及固定场景呈现 | 加模式准入/焦点路由，Run时阻止编辑命令，不同时推进编辑和运行world |
| 838合同 | 静态资产/可携带几何/同设备资源复用，尚在实施 | runtime只读钉住正式资源快照，不自行导入/换资源/写项目目录 |

此前两款固定游戏已能真实Auto运行，但用户编辑/导入的场景还没有Run/Pause/Stop生命周期。本切片验证从实际编辑对象到实际运行对象的接口和隔离，为后续行为产品化提供接缝；不再堆一组纯编辑控件，也不把旧游戏窗口冒充当前场景运行。

生产AutoUI/RQHost、831/832迁移及Linux/Shader/CAP仍按各自交付门；不因独立运行原型开启这些写面。

## 5. 详细设计

### 5.1 入口与有界whole-path可行性

下一任务为G838后的T-01，然后T-02；当前T-01..06已完成；规划时839未开始/838在途的描述是历史。T-01从主检出最新合同核正式落点/SDK/锁/资产API，记录唯一树和完整来源，不在838候选上提前准备。

T-02建立新`research/scene-play-feasibility.md`，最多比较两种driver-local运行帧/资源快照接法：一个已有2D叶子、一个838真正导入的多part GLB，均来自accepted场景且至少一个在非零父组下；真实Run→OS焦点键盘→Auto实际位移/全mesh画面→Pause/Step→Stop恢复→再Run。另测pending tick Stop、错误world/resource输出、CPU预算和启动失败后的下一次Run。

先用scratch状态/协议闭合snapshot/VM/输入/prepare/恢复，不要求后续正式Play模块已存在。记录start/stop/compile/帧/128节点事务耗时和实际owner，固定profile6查询布局、状态机和预算。若无法消费838公开资源快照、必须改SDK/AGP或无法不改Editor完成恢复，回new列确切缺口，不降为Rust动画、另开固定游戏或停止后丢编辑内容。

### 5.2 运行快照与状态机

状态为Editing、Starting、Running、Pausing、Paused（含单步inflight标识）、Stopping、Faulted。启动仅Editing且无编辑gesture/事务、未应用属性draft、导入或未确认保存结果；有draft则提示先Apply/Cancel，不静默丢用户输入。允许dirty和已接受但未保存的资产，因为运行取内存accepted快照而非上次磁盘文件。

Run保存base epoch/revision、canonical content hash、选择、history/redo摘要、saved hash/dirty、资源指纹/Arc及可直接恢复的编辑呈现；生成不复用的RunId/操作token。运行图及Auto局部变量与Editor独立；不把runtime Object地址、ordinal或VM塞入Document。活动运行快照固定资源/对象ID/父关系和静态属性，运行期间不能选择新actor/改图/导入/保存。

Starting完整配置/初始输出/prepare通过才显示运行帧；失败仍在原编辑图，释放私有owner并显示原因，可再次Run。停止不需重新编译编辑preview或从源路径重读资源：丢运行候选/VM，清输入与时钟，恢复保留的accepted呈现和选择/普通编辑控件，再变Editing。运行开始前的未提交文本draft不被本Plan承诺恢复，因为它被启动准入阻止。

运行期间真实Auto/消费者故障进入Faulted，冻结last-good运行图/错误，关闭受污染VM及所有pending任务；Stop恢复Editor，后续Run使用新RunId和新VM从accepted内容初始化。真实设备丢失等既有renderer故障不编造恢复成功，按原profile报告；恢复语义不依赖坏VM继续执行。

### 5.3 真实Auto profile6及全帧准入

新`samples/authoring_play/main.at`（拟）是随程序交付的真实Auto样例，不是Rust行为代替。输入完整局部图、稳定ID的事务ordinal、parent/kind/dimension/resource ordinal及固定属性，调用显式init后保留同一Session。每个逻辑tick传正整数tick ID、固定f32 dt=1/60、实际space/轴状态/可见性toggle；Auto在自己的状态中更新local translation/visible，重新派生world/effective。tick达到SDK正i32上限时在下一派发前受控停机并提示，不溢出复用旧ID。

mutable输出仅组/叶子local translation与local visible；kind/ID/count/parent/resource/yaw/pitch/scale/rgba固定为启动快照。所有节点（含hidden）按128节点/数值/维度及838实例/资源预算完整准入；2D z仍0。新导入Model沿启动资源映射消费其全部原mesh。不能动态创建/删除/重挂/更换资源或在越界时clamp/丢节点，错误整tick拒绝。

每个输出含profile、初始化身份/当前tick、节点元数据、local及world/effective；host将RunId/base hash/source指纹与输出序号绑定，准入时先拒绝旧run/重复/跳号/foreign资源。运行内scene只是derived runtime数据，不调用Editor事务。CPU独立参考可核world与输出locals关系；**renderer/OBB/组边界必须消费实际Auto输出**，不能渲染Rust参考值。

一tick的callback、所有queries、资源/帧准备完成后才一次发布runtime frame及accepted runtime tick ID；未完成不是可见新帧，不能发布半个子树。VM内部先执行而包失败时直接Faulted/关闭，不宣称回滚VM或继续下一tick。已经发出的每个合法完成tick须完整消费，不能因显示帧合并丢行为；bad tick不计accepted tick。

### 5.4 样例行为与输入分母

UI明确显示2D和3D操作对象，各选一个现存同维叶子（含导入Model），以稳定ID绑定；默认取文档顺序首个eligible叶子，缺该维则None，空场景也可运行空tick。Run时锁定选择，不按名字/当前树选中对象临时换target。

固定Auto样例行为：canvas焦点下方向键控制对应actor，2D左右→world±X/上下→world±Y，3D左右→world±X/上→−Z/下→+Z；速度分别60个2D单位/秒、3个3D单位/秒，各轴独立，斜向不归一。父组只继承平移，改actor local产生对应world位移；其他节点local保持。V键非repeat按下对当前space actor可见性取反，Auto处理；显示/隐藏不删除对象。坐标超既有合法范围则Faulted，不暗自更改速度/截断。

host仅记录实际CompositionInput焦点surface、键down/up/非repeat边沿；不在Rust执行位移/visibility逻辑。每tick派发时快照轴−1/0/1及space；toggle按space保存一位parity，两次tick前切换抵消，不建无限按键队列。该采样规则公开，不承诺还原tick之间的全部物理路径。

Iced文本/按钮focus不产生行为输入，点击Run不能把旧held方向带入；canvas点击用于运行focus，不启动编辑选择/gizmo。切换canvas先清旧held/toggle；窗口失焦/leave/resize/DPI/minimize及Pause/Stop/Fault清输入，非repeat新按下后才恢复，不能靠OS自动重复重新激活。Esc绑定明确Stop返回编辑，Iced内文本Esc按widget/模式顺序不重复投递。按键up即使失去旧焦点也清host状态，不给新run复活旧动作。

### 5.5 调度、暂停、单步与有界性

逻辑dt固定1/60，并非承诺所有128节点/任意负载都实时60FPS。每事件循环轮最多dispatch4个tick、最多1个inflight tick和4个未派发tick额度（计数，不排整图/VM）；新input在实际dispatch时采样。一轮所有runtime工作共享8ms pump目标，加当前SDK slice/OS调度，不能对每个callback各给8ms再累加无界。

沿用Session的1000steps/2ms slice、64指令轮询、每callback100000steps/120 resume/250ms。新增整tick事务含callback+queries+prepare总期限2s，初始化含编译后检测及配置的期限30s；同步编译/原生调用本身不可抢占，分别报告耗时，不把deadline伪称硬安全沙箱。待查询固定节点数推导且有界，最近报告最多4096条标量摘要，不保留4096份文档/帧，不逐帧积累history/trace。

显示gap>250ms、最小化/失焦，或未派发额度将超过4时进入有原因的auto-Pause，清墙钟债务和held。已派发tick按完整事务完成再Paused，超预算则Faulted；不跳accepted tick ID、不截断Auto输出，也不无限追赶丢失的墙钟时间。恢复从新wall-clock基点开始。

Pause请求使Running→Pausing，拒绝新tick、清输入；若有inflight仅等这一笔完成/失败，Paused后tick/pixels保持。Resume从Paused重建clock基点，不补暂停时间。Single Step仅Paused且无inflight，派发恰好一个**中性输入**tick，成功后留Paused；忙时重复请求拒绝/不排队，同操作token去重。Auto返回的tick计数必须+1，不能只有host HUD自增。Step不承诺静止输入必有物体位移；当前样例中计数推进但物体可不动。

Stop在Starting/Running/Pausing/Paused/Faulted均可请求，在下一个事件循环边界丢候选并关闭VM/所有任务，不等待脚本destroy回调；旧完成/按键/Pause/Step操作token无效。停止期间不派新tick，不将坏候选/未发布帧写回编辑器。再Run只能新初始化，不接着旧tick/源状态继续。

### 5.6 编辑、文件与资源隔离

Run/Pause/Step/Fault不改变accepted Document/hash/revision/saved hash/dirty/history/redo或磁盘entry/blob。期间Create/Delete/Apply/Reparent/Import/Open/New/Save/Save As和actor重选均明确disabled并在命令准入再次拒绝，不排到Stop后补做；手势和运行互斥。Close先Stop恢复，再对accepted dirty/unverified执行原Save/Discard/Cancel；取消关闭留下可编辑的原内容，不恢复运行或吞旧编辑。

不新增持久字段，沿用838 schema与完整目录资源闭包；运行状态/actor设置/tick/输入/代码绑定不保存。本轮固定Auto样例随binary/source consumer交付，场景搬迁后可运行同一内置行为；这不等于任意用户脚本已随场景打包。只有显式Stop回Editing后才能保存编辑版本，末帧不成为新的saved内容。

启动资源按accepted snapshot固定并共享合法GPU Arc；runtime只改实例/覆盖层，warm不重新上传vertex/index、不新建pipeline/target。最多额外一个runtime VM、一个正在准备的完整frame和一个last-good runtime frame，原编辑preview VM/资源为合法保留owner；Stop后runtime VM弱owner消失，mesh因Editor仍在使用可以继续存活，不能错误要求它此时全部释放。Editor关闭且最后外部pin drop后资源才最终释放。

20轮Run/Pause/Step/Fault/Stop/再Run验证实际VM弱owner、source资源/mesh Arc、buffer writes与target/pipeline身份。实例每tick重写属于合法开销；model上传/创建与内容/合成写分开统计，不用总计不变伪装运行无写。报告编译/启动/停止/帧CPU、工作集，GPU timestamp/Iced内部/驱动驻留不冒认。

### 规范增量

均拟议，当前不写canonical；scene-assets及schema/profile5先由838正式沉淀，review再冻结839实际增量，merge更新overview/plans/goals/ledger。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/scene-play-session.md | 无用户场景运行模式→snapshot/状态机/profile6/固定tick/输入/单步/故障/Stop恢复 | 可消费运行生命周期 | AC-02, AC-03, AC-04, AC-05, AC-06, AC-08 |
| SD-02 | modify | docs/specs/auto-graphics/authoring-session.md; docs/specs/auto-graphics/scene-document.md; docs/specs/auto-graphics/scene-hierarchy.md; docs/specs/auto-graphics/scene-assets.md | accepted编辑→独立runtime副本/命令互斥/资源pin，schema不变/不保存运行状态/Stop复原 | 不污染编辑与持久化 | AC-02, AC-06, AC-07, AC-08 |
| SD-03 | modify | docs/specs/auto-graphics/viewport-interaction.md | 编辑focus/gizmo→运行focus/action隔离，切换/取消清held，Stop回原编辑呈现 | 实际输入只到目标 | AC-05, AC-06, AC-08 |
| SD-04 | modify | docs/specs/auto-graphics/auto-source-driver.md; docs/specs/auto-graphics/prototype-profile.md; docs/specs/auto-graphics/project.md; docs/specs/auto-graphics/integration-contract.md | profile1..5有限实验→6行为样例/运行模式与旧profile并存，披露预算/可携带边界，I2/I3仍pending | 不冒认完整游戏编辑器/生产集成 | AC-01, AC-03, AC-07, AC-08, AC-10 |

## 6. 测试设计

新增入口/fixtures由work任务建立，以下全是拟验收，不是本轮已跑。未指定实施/internal/final reviewer模型/新聊天，不创建agent；final review同一树独立上下文，暂停实施写入，修复回同树，实现会话自查披露局限，不冒充最终独立review。

未来839树driver目录先用既有`python tools/prepare_sdk.py --repository D:/autostack/auto-lang`和`python tools/verify_source.py`核固定来源。新增拟`cargo test --locked --test authoring_play -- --nocapture --test-threads=1`及`--test authoring_play_host`（T-03..07建立）；旧scheduler_lifecycle、authoring_session/storage/hierarchy/assets/interaction/probe_exit按触面定向。check/fmt沿用locked all-targets和既有fmt入口，不跑整档。

新原生模式拟`autographics-authoring --play-check --report <path>`及`tools/check_scene_play.py`（T-07）；保留原window/interaction/hierarchy/asset-check及inspect完成性。命名可等价调整，T-08给准确命令/实际数量，0selected/skip不算pass。

| case_id | AC / work owner | 输入/状态 | 实际入口与observable oracle | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01/07；T-01/08 | 正式838 source/SDK/两锁/资源，源漂移/缺资源 | 原verify_source/独立consumer闭包及反控，原profile/game/文件/资源分母不漂移 | 不用在途WIP冻结 |
| V-02 | AC-02；T-03/05 | dirty/有redo/已导入未保存的场景，非零父组/actor，旧文本draft和pending事务 | Run入口实际snapshot；dirty合法启动，draft/pending拒绝；运行前后文档bytes/hash/revision/dirty/history/redo不变，runtime空间/资源是实际场景，不是固定游戏 | 原编辑器内容保持，不Apply运行状态 |
| V-03 | AC-03；T-03/04 | 真实Auto多tick；错ID/count/parent/resource/local-world/effective、重复/跳tick、旧RunId | Session/profile6→全帧consumer→实际GPU；合法输出精确映射，坏包保last-good/tick并Fault关闭VM，无半树或Rust fallback；改变Auto速度/可见性实际输出影响画面，错误变异由独立expected拒绝 | 仅local translation/visible可变 |
| V-04 | AC-04；T-04 | 固定600tick/action trace，显示间隔不同，连续Run重放，128节点/超时/过载/gap | actual retained Session+host clock；每tick trace对独立f32参考，小场景位置1e-3、事件/计数精确，分批显示结果相同且不丢完成输出；clock超额autoPause/期限Fault，无无限catch-up | 60Hz逻辑dt，不承诺所有输入实时60FPS |
| V-05 | AC-04/06；T-04/05 | pending时Pause/Stop，Paused长等/Resume/单步/忙重入、旧Step/完成后Stop再Run | 正式状态机/Session owner；Paused不增tick，Step实际Auto tick恰+1；Stop关闭而非继续坏VM，恢复accepted，再Run初始tick/位置正确，旧结果无效 | 同步compile不可抢占，Stop在事件轮边界 |
| V-06 | AC-05；T-06 | 实际2D/3Dcanvas、UI文本/按钮focus、down/up/OS repeat/V parity、切空间/blur/leave/DPI | CompositionInput/真实Host key→tick action→Auto；方向只改当前actor，UI输入不移动，两次toggle抵消，暂停失焦后不sticky，新非repeat合法按下可恢复 | 单窗键盘，采样动作非全物理轨迹 |
| V-07 | AC-06；T-04/05 | init/运行Auto类型/无限循环预算/prepare失败、数值越界、合法Stop→再Run | 实际Session故障/资源注入；init失败编辑帧不变，run失败保最后完整运行帧并关owner；Stop不用重编译或重导入即恢复，下一合法Run成功 | trusted源码/native-call范围沿用 |
| V-08 | AC-02/06；T-05/06 | Run/Paused/Faulted时Save/Open/Import/结构编辑及Close Cancel/Save决策 | 真命令准入+实际文件；busy命令无队列/无文件写，Close先恢复原accepted再原dirty流程；Stop后Save+新进程inspect只有编辑内容，无runtime字段/末帧 | 不新增schema/持久script绑定 |
| V-09 | AC-07；T-03/08 | 20轮Start/Pause/Step/Fault/Stop，导入Model共用资源、合法pin、Editor最终Close | 真VM weak owner/GPU Arc/创建及实际writes；Stop关runtime VM但保Editor mesh，最终owner/pin drop才释放；warm无模型重传/target/pipeline创建；保留VM或重传负控红 | 实例重写合法，排除未测内存/GPU时间 |
| V-10 | AC-05/06/08；T-07 | OS点击Run/两canvas键盘/Pause/Step/Stop，实际导入多part模型/组，wrong-focus/drop-tick/frames1 | 新play-check真实OS→Iced/router→Auto→top capture；具体actor端点/原位置/全部mesh/Stop原编辑帧有独立像素oracle，透明/错资源GPU故障须同门红，partial/漏帧/错焦点非零且report一致；完整正门exit0/passed=true | 可交互桌面必须可用，不能只比state/两个错hash |
| V-11 | AC-08；T-08 | SD-01..04/README/实际CLI/支持面/预算/生产边界 | 拟议Spec与可复跑handoff对应；schema未变、固定示例而非用户脚本产品化、I2/I3未完成如实 | 静态不替代运行/像素/owner门 |

每case保留完整command、退出码、actual断言/数量、冻结code/SDK/锁/输入hash及原始证据。V-04的输入按dispatch tick记录，跨显示频率比较同一tick动作trace，不用不同墙钟输入冒称确定性。V-10必须证明实际Auto→GPU与恢复，host HUD tick/命令/hash不能单独作oracle；原counter/content双合成门继续并列。复用前序证据须证明实现/依赖/配置/输入未变，新profile/模式不能借旧游戏绿外推。

最终review后，merge统一冻结提交跑完整locked driver/AGP test+build、原16 AGP cases、双游戏/835..838原生正反与partial/reopen、新play完整正反/实际GPU反控/Stop恢复/保存重开、counter/content双门、source/assets/独立consumer。准确矩阵由T-08登记。开发/review不跑root整档t/tv/tt/tb/tu/taa/docs_gen或上述完整档，tf仅主检出到期批量。若需改SDK/AGP/生产Rust，先回new界定写面/适用门，不能自动扩大。

## 7. 验收标准

| ID | owner_stage | 可观察标准与方法 |
|---|---|---|
| AC-01 | work | G838/GSOURCE、唯一树/SDK/写面与闭包成立；V-01 |
| AC-02 | work | actual accepted快照启动，运行不改文档/hash/history/dirty/资源文件，不读旧磁盘替代未保存内容；V-02/08 |
| AC-03 | work | 保留真实Auto Session/profile6，完整mutable/immutable/层级/资源/tick准入和actual输出消费；V-03 |
| AC-04 | work | 固定tick/有界调度、Pause/Step/Resume/过载和whole-frame期限正确；V-04/05 |
| AC-05 | work | UI/2D/3D实际输入隔离，既有canvas下实际actor/全部导入mesh可响应，无sticky/错focus；V-06/10 |
| AC-06 | work | Start/故障/Stop/旧token/Close/busy命令及恢复/新Run闭环，无末帧保存；V-05/07/08/10 |
| AC-07 | work | 原profiles/游戏/资产保持，20轮运行VM释放、Editor合法pin/mesh复用/上传计量有界；V-01/09 |
| AC-08 | work | 真实OS正反/partial/GPU反控、源指纹、SD-01..04与可复跑支持/预算交接完整；V-10/11 |
| AC-09 | review | 独立核AC-01..08/Spec及遗漏/延后/workaround/fmt/warnings/debug，版本绑定pass，完整merge门显式pending |
| AC-10 | merge | 冻结完整门通过后实际合autographics-dev，canonical/ledger/归档/guard清理与到期检查完成，源码不入master |

## 8. 执行步骤

当前execution_done8/10，T-01..08完成；T-09独立复审与T-10完整合入门pending。T-01..08完成且无in-contract finding/必需gap才execution_done8/10；T-09 pass为reviewed9/10（完整merge门pending），T-10实际完成才交付10/10。

| task_id | owner_stage | 依赖 | 实际触点／新路径与动作 | 关闭条件；AC |
|---|---|---|---|---|
| T-01 | [✅] work | GSTART/G838 | 主合同/838完整交付收据、839唯一树；prepare_sdk/verify_source/锁/正式资源API | [x] 838 archived10/10、R2/55入口/guard移树/台账75/not-due齐备，正式基面37fd1a9cfe22d3d3471202ad4a8dbfe2103f9a5c；唯一树/分支断言、SDK680/closure479/17local/29edges/source264核验通过；profile5/schema3/Registry/同设备staged接缝已核。来源反控沿用verify_plan838 --controls，详work-r1；AC-01 |
| T-02 | [✅] work | T-01 | Session/Building/Editor/Host/正式838资源；新research/scene-play-feasibility.md | [x] scratch真实Auto61 tick、多part/非零父、实际GPU恢复/type/budget/owner重启通过；随后同设备分离Frame/保留Prepared接法的native Run→键盘→Pause/Step→Stop→再Run正门通过。原型提升及接口/布局/预算决策见树内research/scene-play-feasibility.md；正式覆盖归后续任务。 AC-02, AC-03, AC-06；原始结果归work-r1收据。 |
| T-03 | [✅] work | T-02 | 新src/authoring/play.rs、samples/authoring_play/main.at、tests/authoring_play.rs；实际profile consumer | [x] profile6/Play独立snapshot、int身份全帧准入、immutable/resource/world/effective反控、dirty/redo/未存资产隔离与VM释放通过；authoring_play 6/6、play_host 5/5。 AC-02, AC-03, AC-07；原始结果归work-r1收据。 |
| T-04 | [✅] work | T-03 | play时钟/CPU pump/Pause/Step/Fault/关闭及trace；原Session只消费 | [x] 两次600 trace独立f32参考、两种显示间隔600完整tick、128节点、Pause/Step/Stop stale/type/预算/数值/启动30s/整tick2s/prepare后期限/tick上限全部定向通过。 AC-03, AC-04, AC-06；原始结果归work-r1收据。 |
| T-05 | [✅] work | T-04 | Panel/Host::dispatch/frame/prepare/publish、Editor read-only snapshot/资源pin及Close准入 | [x] Host dirty+redo+未存GLB、运行/暂停/Faulted忙命令/文件sentinel、Close Cancel、Stop不用prepare/compile恢复、Save+异cwd真实新进程重开及20轮VM/mesh最后owner通过。 AC-02, AC-04, AC-06；原始结果归work-r1收据。 |
| T-06 | [✅] work | T-05 | Host key/pointer/focus、bin::window_event、actor选择/运行控件 | [x] 原生27阶段正门及wrong-focus/drop-tick/实际GPU三反控/partial门通过；实际2D/3D箭头/UI隔离/V parity+hide/show/OS blur清held/Stop+再Run/保存搬迁重开。旧Host7/interaction7/session4/scheduler1定向通过；新增面板布局/scroll witness保护实际控件点击。 AC-05, AC-06；原始结果归work-r1收据。 |
| T-07 | [✅] work | T-06 | 新play-check/aggregate/原inspect/完成性、导入fixture/实际像素反控 | [x] 当前producer/独立consumer各7原生正反/partial门全部符合预期；正例27phase、六种真实top像素状态、实际Auto Step+1、Iced文本隔离/V/blur、Stop复原/新Run/保存搬迁异cwd新进程重开闭合，无环境gap；旧window9/hierarchy16/assets20正例当前通过；AC-05, AC-06, AC-08。 |
| T-08 | [✅] work | T-07 | README/new research/scene-play-handoff.md、source/consumer/计量及树内Spec proposal | [x] source278/SDK680/closure479/17local/29edge与四来源反控、独立consumer check/play11/native7通过；producer36独立永久case/20owner循环/保留VM及重传实际反控、fmt/check无新增warning；README/SD01..04/scene-play-handoff准确矩阵已冻结，完整merge档未跑；AC-01, AC-07, AC-08。 |
| T-09 | review | T-08 | 同树冻结code/Plan r1/Spec，主检出独立review记录 | 核AC-01..08及必要定向/反控/遗漏/延期/workaround/健康；版本绑定结论；AC-09 |
| T-10 | merge | T-09 pass | 冻结完整门→autographics-dev；主检出知识/ledger/archive/guard | AC-10实际门/落点/沉淀/清理/到期收据，代码变更先受影响复审再重冻，不提前交付 |

## 9. 复审记录

2026-10-24：stage: work | plan_id: PLAN-839 | plan_revision: 1 | outcome: **pass** | worktree_path: D:/autostack/.wt/lang-839/auto-lang | branch: plan-839-dev | base_commit: 37fd1a9cfe22d3d3471202ad4a8dbfe2103f9a5c | code_commit: e58828ca64b7e26ce984faf850fb3b1d0887517f | dependency_revisions: SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9/680、两锁不变、metadata479/17local/29edge；source278 LF SHA526f760d6a5e8eb74fd33bc0f3bd222fff76131f83b4da5b9cfc32469da7d0a3 | task_ids: T-01..08 | evidence: [readiness/逐V与AC映射](attachments/839/work-r1/readiness.md)、[版本/命令/原生/owner/计量结构收据](attachments/839/work-r1/receipt.json.txt) | blockers: none；工作AC01..08 pass、open findings/必需gap/未决决策均空 | next_responsible_role: **review**。producer36独立永久case、current Host启动失败owner/下次Run与Resume提示反例、producer/consumer各7原生正反、consumer11当前case与check、source四反控、owner保留/重传两反控及恢复均通过。唯一树clean并保留、实施写入暂停；自查非最终独立复审，T09/AC09与T10/AC10、冻结完整合入矩阵显式pending；未整档/合入/发布canonical-ledger/归档清理/新建agent或聊天。技能为用户指定auto-musk/.agents/skills/auto-plan-work/SKILL.md及仓库commit技能；不触及AGP/SDK/root源码或锁。

2026-10-24：work进度6/10。T02 whole-path可行性已闭合并提升为正式Play；T03..06模型/Host/OS定向通过，T07在当前最终身份重放、T08源闭包/独立consumer/交接收口。期间发现旧固定坐标/未计scroll控件探针不适用新增面板布局，保留原断言改实际widget/OS scroll；V注入补真实scan+WM_CHAR，像素读回触发autoPause后通过实际Resume并重获canvas焦点，生产时钟不放宽。开发失败原日志保留；未发生final review，未跑完整合入档。

2026-10-24：stage:work | PLAN-839:r1 | outcome:in_progress | 用户明确授权839实施；GSTART/G838满足，838正式交付与清理收据已核。worktree_path:D:/autostack/.wt/lang-839/auto-lang | branch:plan-839-dev | base_commit:37fd1a9cfe22d3d3471202ad4a8dbfe2103f9a5c | SDK:9576b4dad5ceb8b201adedd46f9689f25c2dd5f9 | 主检出代码WIP为空，其他文档WIP保留。T-01来源准备/正控已过，T-02核actual Session→局部图→Auto world/effective→Host GPU的接法与恢复owner；未创建agent/聊天，完整merge门pending。技能来源为用户指定auto-musk/.agents/skills/auto-plan-work/SKILL.md。

2026-10-23：stage:new；PLAN-839 revision1；outcome:blocked（G838完整交付未完成，草案内容可交接）；next:838完整交付后，用户启动839 work，从T-01冻结正式基面，再T-02可行性。本轮未实施/建树/准备/build/Cargo或原生测试，未创建agent/新聊天。

规划阶段历史drafting0/10；T-01..10、AC-01..10、V-01..11、SD-01..04均为新合同，非执行pass。静态检查见[preflight](attachments/839/planning/preflight.md)。final review和完整merge门均未发生；连续两轮同因不收敛时保留轮次，先诊断根因/漏测/兄弟路径再修，不改fixture或暗降支持面过门。

## 10. 待澄清事项

当前工作无开放finding、必需覆盖gap、未决合同问题或阻塞；T01..08已完成，execution_done8/10，下一责任角色为独立review。T09/T10与完整merge矩阵是后续owner事项，不冒认为已审/交付。

当前无产品决策阻塞；G838已于2026-10-24核验完整交付，原草案前置已解除。首版是内置真实Auto行为样例的运行模式，schema不变、无自定义脚本产品/运行结果回写已明确。T-02关闭等价局部机制无需重复审批；若要任意脚本导入/可携带代码、组件/物理/音频/动态结构、Apply运行结果、SDK/AGP/生产集成等，回new提出具体范围/支持变更；架构扩面按L2先补正式设计。
