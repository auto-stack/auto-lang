---
plan_id: PLAN-883
status: drafting
feature_name: AutoGraphics有限触发事件与新局隔离及Gallery G11
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 1
supersedes_spec_components:
  - docs/specs/auto-graphics/scene-play-session.md
  - docs/specs/auto-graphics/auto-source-driver.md
new_spec_components:
  - docs/specs/auto-graphics/runtime-triggers-and-rounds.md
  - docs/specs/auto-graphics/gallery-runtime-objects.md
touched_goals: [GOAL-008]
affects: [auto-graphics, ag-examples]
depends_on: [PLAN-882, PLAN-880]
current_step: 0
total_steps: 11
---

# [PLAN-883] AutoGraphics有限触发事件与新局隔离及Gallery G11

## 0. 变更摘要

消费[882对象生命周期](882-autographics-runtime-object-lifecycle.md)实际交付，建立有界几何触发/一次事件读取及受检新Run请求。两维公开小工程验证三物品收集、危险/终点与胜负后新局，玩法仍由用户Auto负责。承接Gallery G11的两维仓外示例；内容阶段等880交付，不接管其在途树。

本稿是有前置的语义合同：882尚未实施，不能预填其最终版本/计费/接口。T-01/02在实际交付基线上冻结本计划；语义变化递增revision。不是完整G2/G3游戏或HUD/声音交付。

## 1. 目标

- 每Run最多8对显式触发，分别有enter/stay/exit、确定顺序和Run/tick/对象代次；同tick只评估一次，事件有界、读取不重发。
- 三个不同物品各首次收集一次；危险/敌人或越界失败、集齐才可终点获胜由Auto状态机决定。销毁物品是真生命周期，旧代不因槽复用继续触发。
- 用户Auto请求新局后进入新Run/VM，重置位置/物品/计数/输入与事件，拒绝旧Run迟到结果；新初始帧失败保完整last-good。
- G11/2d、G11/3d在ag-examples作为独立公开项目/离线包，元数据与真实源/引擎/记录绑定。

非目标：完整两款游戏/最终关卡、动态solid/移动平台/刚体、无限碰撞世界/自动全对扫描、连续碰撞或高速穿越触发保证、动画/纹理/声音/HUD、通用事件总线/跨线程消息、任意多场景切换/存档/联网、880内容/工具重写、ACE/UI融合/跨端/发布。G11占位形状可表现身份变化，不认领最终美术。

## 2. 架构方案

受信Auto按本tick接受前的实际世界位置计算有限形状相交、比较前态、提供callback内有界事件结果；用户Auto负责计数、巡逻、胜负和请求restart。Rust只做配置/身份/几何来源准入、新Run调度与GPU事务，不识别金币/终点名称执行玩法。

复用一个Host/Play/Session调度栈；以新显式组合身份隔离882及所有旧profile。候选触发结果、实例生命周期、角色与相机共同准入/prepare/accept/publish；restart是候选中显式请求，在已接受边界启动全新Run，不在回调中递归建VM或复用旧globals。

## 3. 技术栈

auto-ui driver已有`src/authoring/{play.rs,behavior.rs,behavior/assembly.rs,game_input.rs,runtime_camera.rs,spatial.rs}`、`host/{play_adapter.rs,viewport.rs}`，882拟新增runtime_objects模块。当前实际路径与基线见[本批基线](attachments/882/planning/baseline.md)。无新增SDK/依赖。

本计划新路径拟为`api/runtime_triggers_v1.at`、`src/authoring/runtime_round.rs`、`tests/{runtime_triggers_api,runtime_round_host,runtime_trigger_samples}.rs`、`samples/runtime-triggers/{two,three}/`、新identity/checker；ag-examples新增`examples/g11-runtime-objects/{2d,3d}/`与README/元数据/锁。具体命名/签名/内部编号在882交付后T-02冻结，不虚构已存在API。

## 4. 需求分析与背景调查

授权：用户2026-10-29要求879迁仓后规划下一阶段，880已开始开发。本轮只规划，无实现/新树/agent/880修改/远端动作；执行等待后续work指令。882是引擎硬前置，880是Gallery接线阶段前置；880进行期间可审本稿，882交付后也可先做本计划引擎T-01..04，不提前T-05共改内容工具。

权威输入：`docs/specs/overview.md`、`auto-graphics/{source-repository-and-sdk,behavior-api,scene-play-session,spatial-query-profile,game-actions-and-character2d,game-character3d,runtime-game-cameras,gallery-basic-six}.md`；[Design45§13](../design/45-autographics-wgpu-engine-and-game-prototypes.md#13-m2运行对象触发与新局事务2026-10-29拟议合同)、[游戏分母](../design/autographics/m2-game-sample-matrix.md)、[Gallery矩阵](../design/autographics/m2-gallery-and-capability-matrix.md)及882/880合同。本轮不改canonical Specs。

当前只读auto-ui基线`4adf2d037a1804741adc9c5a012f08cfa566439d`（879），真实执行base必须换为882交付SHA并复核差异/SDK/锁/动态对象合同；AGSDK旧pin7b987ca8/681快照不变。ag-examples规划观察master6a223d9，880唯一树在lang-880；不能据drafting/clean断言其无owner或抢树。G11内容实际base必须为880交付，按其最终按项锁schema，不用本稿猜未来格式。

源码已核：旧v2 `ag_aabb_overlaps`是闭区间、单对查询，没有引擎事件队列；用户例子自己存前态。Play新Run会重建VM，Stop保编辑Prepared；缺少公开用户restart请求与保旧到新tick0的专用边界。已有游戏研究要求三个独立收集物、半径0.5触发区域、胜负后Enter新局、旧句柄不可影响新局。本Plan不能把触发AABB包围盒误当圆/球，或把Stop/Run命令发出算新局完成。

## 5. 详细设计

### 5.1 唯一树与前置冻结

引擎唯一树`D:/autostack/.wt/lang-883/auto-ui`，分支`codex/plan-883-dev`，882实际交付后从该auto-ui源码SHA创建；源码只合auto-ui/master。G11唯一内容树`D:/autostack/.wt/lang-883/ag-examples`、同名分支，在880交付且其树退出共享工具写入后创建。语言主检出只Plan/SD/ledger簿记。每仓一树，同树顺序work/独立review/merge；无链接，清理先guard。

T-01复核882实际接口/Run域/模板与活对象表、原SDK/两锁/current identity；T-02冻结`attachments/883/work/triggers-round-contract.md`：profile/签名、8对资源表、事件字段/顺序/读取/重绑规则、几何边界、restart状态图/失败路径、全部消费者和实际计费。

先证明整路径：真实Auto单角色运动/跳/归一＋相机＋8对检测＋三物品同tick相交/消费/真实销毁＋结果输出。保持共享32credits、1000steps/slice、100000steps/callback、120resumes/250ms、256/64/320KiB、30s初始化/2s整tick/8ms目标。若需紧凑输入或有界批量几何，必须先冻结明确成本/正反例，不以回调拆分或免费无界扫描绕过限额；不足回new。第一次query成功不能替代该组合门。

### 5.2 有界触发与几何

最多8对，用户显式注册/重绑，不自动扫描全世界所有对象对；每对一个同维subject（静态角色/受检叶）和一个静态或882活实例的触发源，稳定pair序号按注册表顺序。实例身份必须完整绑定Run/slot/generation，不能只存slot并自动追同槽新生；重绑新代是显式操作，prev态清零，旧事件不跨绑定代。非法/错维/组/未知/非有限/数量超限在使用前具名拒绝。

支持两种形状：受检实际资源AABB间闭区间相交（沿用旧边界），以及2D圆/3D球与subject实际AABB的最近点平方距离比较（<=radius²，radius>0有限；2D z=0）。默认样板radius=0.5；相切命中、角点在包围盒内却在圆/球外不命中，±1ULP分离须有反例。球半径由明确配置定义，不隐式取GLB包围盒当半径；静态/动态触发源center跟其当前接受前世界平移。隐藏不取消触发（显隐与生命周期分离），非solid触发不阻挡角色；销毁撤销资格。

每次app_update所有本tick运动完成后调用一次evaluate，读取同一当步位置；第二次evaluate、evaluate后再改参与对象位置/重绑配置应受控拒绝，不能把旧几何事件配新位置。evaluate后允许按结果销毁物品；本tick事件保留完整旧身份的只读快照，但不复活其写权限。新spawn/重绑参与本轮检测的时序在T-02冻结，依“评估后无新的几何状态混入本轮”原则实现。

app_start只建立注册表、无enter事件；首次update与false前态比较（初始重叠在首次update enter）。false→true=enter，true→true=stay，true→false=exit，false→false无事件；每对每tick至多1条，按pair序号输出，总≤8。一次evaluate后结果冻结，公开next操作每事件只出队一次、无新事件时返回显式absence，getters只读当前token且callback过期失效。OS重绘/重复getter不推进状态，不能因Pause/重复read再计数。

对象在enter后被销毁：该身份不在后续tick继续stay/合成exit，pair进入失效未绑定态；同槽新代不被自动纳入，显式重绑后可重新enter。user Auto决定“首次收集即销毁并计1”，不是Host看到名字就加分。已接受之前的任何事件都不得给外部声音/HUD/日志业务消费者当作正式提交结果（声音/HUD本计划不实现）。

### 5.3 候选/事件/生命周期共同准入

事件记录至少有profile/Run/tick/pair/binding-generation/phase、参与对象完整身份、有效几何来源；事件唯一键=(Run,tick,pair,binding-generation,phase)。Host核对顺序/唯一/容量/活性或当步合法销毁的墓碑快照、浮点域/几何来源/合法前态转移，拒绝旧Run/旧tick/重复/错对象/缺帧/假enter结果；不能只验证事件count或让Rust代算用户胜负。

Auto内事件前态/游标属于callback候选，外部可见对象/事件/角色/camera一起prepare/accept/publish；若source Fault、预算超限、事件身份错、prepare失败或资源失败，保last-good全体，闭VM，不能出现“币消失但计数未提交”或“新计数配旧对象”。Pause/neutral Step依固定tick语义：Pause不检测不推进，Step只中立输入执行一个逻辑步，可因真实几何变化产生该步结果，不合成新的OS按键。

### 5.4 新局请求与失败隔离

用户Auto拥有Playing/Won/Failed；样板只在Won/Failed读取一次Enter边沿后请求restart。公开请求是本callback有界输出标记，最多一个待处理请求，多次同tick请求幂等合并为1，不建队列；携带当前Run/tick、原接受source/config身份，在当前完整候选接受后才处理。

Host沿原单一调度栈终止旧局：清held/pending/camera capture，关闭旧VM/对象表/事件前态，创建新Run/新VM（app_start再执行一次、用户globals重置）、同一已接受源字节和固定配置，固定初值。临时外部source也使用本次Run已接受的快照，不restart时重新读磁盘；代码编辑/热重载另议。新初始candidate包含新实例代与camera初值，全部准备成功才替换显示。期间保旧last-good Prepared；不闪编辑帧、不混新物品与旧角色、无双调度器。

新app_start/资源prepare失败：具名Fault、关闭新VM，保原最后完整呈现，不假装完成新局或自动无界重试；Stop可恢复原编辑Prepared。迟到旧Run结果/旧事件/旧input永远拒绝，旧对象写权限撤销。新Run成功后位置/三物品/敌人巡逻起点/计数0/相机初值/Playing皆由Auto app_start恢复；held Enter不连续restart，恢复需新按下；Starting/Paused不处理新的restart，Stop取消尚未完成的新局。Run identity耗尽受控拒绝，不回绕复用。

20轮成功新局、失败初始化、Pause/Step/Stop/关闭检查最后VM/对象/事件owner和静态资源复用；保留旧画面所需合法Prepared引用与旧VM/事件owner区别记录。用户外部pin合法保留时只在最后释放后检查，不把计数清零当释放。

### 5.5 两维公开工程与G11内容交付

引擎samples两维各最小试验场，含3个不同实例物品、一个危险/非solid巡逻物体和终点，角色使用已交付公共控制器/相机。Auto内状态机验证：未集齐不胜利、重复stay不加分、同tick三物品各一次、危险/越界失败、集齐触终点胜利、两种结束后Enter新Run。不是最终G2/G3关卡，不要求在此制作纹理/模型动画或音频HUD；可读状态由验收接口/已有形状表达，不能另造每游戏Rust宿主。

G11/2d、G11/3d两子入口消费同Plan冻结引擎候选；正常launch只公开CLI，无probe或checks依赖。G11 README、源码/资源、触发几何、生命周期/旧代拒绝/新局恢复、限制/平台与确切引擎锁齐备。Gallery工具按880交付schema做最小新API/锁接线，不改旧G01..08的固定pin/acceptance；总9项16入口（以880实际8项14入口交付为前提），G09/G10仍未交付，不占满12项。

G11单项ZIP包含两维、说明/许可/必要小资源/选中锁/通用工具，固定引擎独立取得；新目录A实际运行/保存，搬B重开，不访问生产者目录/链接/网络；篡改/缺件拒绝，修复同入口绿。内容树只在880交付后写，T-05前若880未完明确等待、不接管或暗自推迟G11到后继。两仓review绑定引擎/内容冻结commit；合入先auto-ui引擎后ag-examples内容，主语言知识最后沉淀，无远端发布。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/runtime-triggers-and-rounds.md | 手写overlap前态/无restart→有限事件、整数代次与新Run事务 | 公共运行服务 | AC-02/03/04 |
| SD-02 | modify | docs/specs/auto-graphics/scene-play-session.md; docs/specs/auto-graphics/auto-source-driver.md | 对象候选→事件/新局同事务、公开选择/来源/consumer，旧保持 | 完整入口与恢复 | AC-01..05 |
| SD-03 | add | docs/specs/auto-graphics/gallery-runtime-objects.md | G11候选→两维真实生命周期/触发/重开、单项包与固定来源 | 仓外消费知识 | AC-05 |

## 6. 测试设计

未指定模型/agent，work自查后同树独立最终review，pass后merge冻结受影响门。遵守变更相关∩约1/10可复现抽样，seed/全集/选择数记录；新AC下列直接正反例必须关闭，完整AG/全Gallery回归留里程碑。OS输入自动化暂停，不恢复抢焦点/置顶；用公开逻辑输入/无窗口Host/GPU和既有未变路由证据，历史证据不冒充新renderer/新项目实测。

| Case | AC/不变量 | 实际入口/代表输入 | 区别错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 几何/时序 | 新API：AABB闭接触、圆/球角外点/切线/±ULP、同tick双物体移动、8对/9对、重复evaluate | 独立几何oracle与真实Autoenter/stay/exit一致；包围盒替球/旧tick位置真实变异红；超限/阶段错拒绝 | T-02/03；离散采样、无连续穿越保证 |
| V-02 | AC-03 一次/旧代 | 3物品同tick、反复stay/next/getter、销毁后复用同槽/显式重绑、旧eventtoken | 3个不同ID只各+1；老身份无新stay/exit，next不可重发；旧token错代拒绝，合法新代可enter | T-03/04；用户Auto计数非Host业务 |
| V-03 | AC-04 新局/失败 | Won/Failed Enter、held Enter、Pause/Step、旧Run迟到、坏init/prepare/Stop取消 | 新Run/source快照/计数0/3新身份/角色camera初值一致；失败保完整旧呈现、闭新VM；不连续重启/不重读磁盘源 | T-04；无场景切换/存档 |
| V-04 | AC-04 事务/owner | 事件+销毁+orbit同tick、错event身份/重复记录/坏序/超预算，20轮 | 全旧或全新，未接受事件不外泄；weak owner最后释放、静态上传有界；冻结reset/泄漏事件变异红还原绿 | T-04；合法外部pin区分 |
| V-05 | AC-01/05 仓外/预算 | 两维源码对照/fresh SDK consumer、新身份controls、G11两入口包A→B/错锁/旧报告 | 实际Auto状态与GPU内容敏感、共享32credits和原限额；0selected/partial/错runner拒；原G01..08选锁不变、坏包修复可用 | T-02/05；880实际交付后 |

拟命令（新入口由T-03/04建立）：driver `cargo check --locked`、`cargo fmt -- --check`、`cargo test --locked --test runtime_triggers_api --test runtime_round_host --test runtime_trigger_samples -- --test-threads=1`；受影响882对象/Play/相机命名case，current verifier/checker `--controls`及fresh own-SDK离线消费。G11工具命令沿880实际交付`gallery list/validate/launch/package`合同，定向Python结构测试与新两入口无窗口数值/内容/包验收，禁止默认重跑旧窗口probe。

根语言/默认UI零代码diff，不跑根cargo t/tf/tv/tu/docs_gen或完整UI/AG档。GPU按机器单实例；新局需实测真实接受后下一次读取/运行，不以request计数代替闭环。原始证据本地`D:/autostack/plan-artifacts/plan-883/`，attachments只md摘要/契约，记录命令/exit/计数/路径/hash；产品小预览与验收索引放内容仓，非原始大日志。两仓冻结commit/源/runner/资源hash绑定，复用证明代码/依赖/配置/输入未变。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 882实际交付/唯一新源树/旧SDK锁及fresh包核实；内容T-05须880交付 |
| AC-02 | work | 新profile/签名/几何/事件/新局/资源先冻结，完整组合预算成立，旧版本保持 |
| AC-03 | work | 两维有界触发/确定时序/一次读取/真销毁与代次绑定，V-01/02 |
| AC-04 | work | 新Run完整复原/迟到隔离/失败last-good/20轮最后owner，V-03/04 |
| AC-05 | work | 两维公开项目/G11两入口、真实Auto+内容+负控/消费/包/旧锁兼容闭合，V-05 |
| AC-06 | review | 同两树独立核AC/V/SD/资源/遗漏/公开路径/fmt/warnings，merge门pending |
| AC-07 | merge | 两冻结提交/输入身份、受影响抽样/必要新对照与复用审计通过 |
| AC-08 | merge | auto-ui→ag-examples按序实际落地、语言知识/ledger/索引/归档、guard清理收据 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与产出 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 882实际交付 | 唯一auto-ui树、实际SHA/签名/身份/价格/SDK/锁与closed/open核对 | AC-01 | [ ] |
| T-02 | work | T-01 | triggers-round-contract先冻结几何/事件/Run状态图/价格与完整组合probe，不足回new | AC-02 | [ ] |
| T-03 | work | T-02通过 | 新Auto/API/准入/触发时序与代次结果、V-01/02 | AC-02/03 | [ ] |
| T-04 | work | T-03 | 同Host新Run事务/失败/owner、两维公开工程/真实GPU负控，V-03/04 | AC-03/04 | [ ] |
| T-05 | work | T-04且880实际交付 | 唯一ag-examples树接G11两入口/锁/离线包；fresh consumer/资源/来源/fmt/两树提交/SD，V-05 | AC-01..05 | [ ] |
| T-06 | review | T-05 | 同两树独立最终review绑定两commit；merge门pending | AC-06 | [ ] |
| T-07 | merge | T-06 pass | 冻结受影响门/必要新对照与复用审计，失败回work复审 | AC-07 | [ ] |
| T-08 | merge | T-07 | 先auto-ui/master引擎实际落地，固定源码与runner身份 | AC-08 | [ ] |
| T-09 | merge | T-08 | 再ag-examples/master落G11，不动旧pin，无远端发布 | AC-08 | [ ] |
| T-10 | merge | T-09 | 语言主检出SD/overview/plans/ledger/索引沉淀与归档 | AC-08 | [ ] |
| T-11 | merge | T-10 | 两仓wt-guard clean后清树/分支/空组目录；保留必要本地产物收据 | AC-08 | [ ] |

## 9. 复审记录

2026-10-29 new r1：`stage: new | plan_id: PLAN-883 | plan_revision: 1 | outcome: blocked | next: 等882实际交付后work T-01/02冻结；G11内容T-05另等880实际交付`。blocked是硬前置未满足，语义草案可审；不虚构882未来SHA/价格或880 schema。全部AC/V/T/SD未执行，无新树/实现/SDK准备/构建/OS/GPU/agent/远端操作，本轮不改880合同/进度/源码。

2026-10-29 new合同预检：0..10节、唯一AC-01..08/V-01..05、分stage任务与SD覆盖已核；本批两Plan及五份路线文档109个本地链接均可解析、编号各唯一，880跟踪文件相对HEAD零diff。仅结构/路径/合同自查，不是实现或独立final review。

## 10. 待澄清事项

当前无新增产品决策。882交付后如触发/收集/新局组合在原限额不可行，或实际公开消费/880工具合同不能承接G11，带反例回new，不删同tick三物品/新局隔离/任一维或静默抬限。880若尚未完成，T-05明确等待，不建第二占用树、不在其WIP上接线。882/883仍不关闭动画/声音/HUD及完整M2分母。
