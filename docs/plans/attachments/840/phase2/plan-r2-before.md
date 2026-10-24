---
plan_id: PLAN-840
status: archived
completion_kind: delivered
feature_name: AutoGraphics最小公共行为API与用户Auto脚本
plan_revision: 2
author: [Codex]
created_at: 2026-10-24
updated_at: 2026-10-24
current_step: 11
total_steps: 12
depends_on: [PLAN-839]
supersedes_spec_components:
  - docs/specs/language-sdk/project.md
  - docs/specs/auto-graphics/scene-play-session.md
  - docs/specs/auto-graphics/authoring-session.md
  - docs/specs/auto-graphics/auto-source-driver.md
  - docs/specs/auto-graphics/prototype-profile.md
  - docs/specs/auto-graphics/project.md
  - docs/specs/auto-graphics/integration-contract.md
new_spec_components:
  - docs/specs/auto-graphics/behavior-api.md
touched_goals: [GOAL-007, GOAL-008]
affects: [auto-graphics, authoring-experimental, user-behavior-api]
---

# [PLAN-840] AutoGraphics 最小公共行为 API 与用户 Auto 脚本

## 0. 变更摘要

当前archived11/12：完整63门、root5332/163及最终delivery身份门通过，source/SDK/canonical/ledger79/归档均已提交；840工作树、开发分支和只读auto-down树已fresh guard安全移除。当前仅编号840触发的主检出单实例批量回归及最终收据待完成；未冒称12/12。详[合入收据](../attachments/840/merge-r1/receipt.md)。

839建立“编辑→运行→停止恢复”，但产品只运行内置Auto行为。下一步让开发者**编写自己的单文件`.at`行为，仅调用最小公共API，选择现有场景运行；修改脚本后Stop/Run看到新行为，无需改Rust或复制profile内部协议**。

本Plan冻结一个有限实验接口`autographics.behavior/1`，首版面向既有节点的局部平移、可见性、层级查询及输入；不同时扩物理/资源/动态结构。提供两个仓外可运行的不同用户行为示例，验证同一runner二进制可复用。

当前r2：用户明确授权先修复SDK依赖扫描对注释与字符串的误判，更新依赖冻结后继续840，原支持面/AC/V不缩减。r1历史已保存在attachments/840/revision-r2/plan-r1.md及work-r1证据。R1三项finding已在1f19395254d7e676bd188e43e3d8f734d57aaa12完成工作修复，T-03/T-05/T-08重新关闭，当前reviewed11/12，受影响复审通过、完整merge门pending；SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21保持。T-09独立review、T-10冻结合入仍各自负责，不能把SDK修复提交冒称完整交付。

## 1. 目标

1. 用户脚本只定义`app_start()`、`app_update(dt float)`和私有辅助/状态，通过公共API访问当前场景/输入并修改允许字段，不实现reset/profile/ordinal/resource输出等transport函数。
2. Run可显式选择内置样例或用户文件；每次新Run读取有界UTF-8源码快照，固定API/bridge/用户源码身份，再通过同一公开语言SDK编译和真实Auto执行。
3. 用户API实际改变当前场景和838导入模型，沿用839全帧准入、Pause/Step/Stop、故障/last-good及编辑/磁盘隔离；禁用脚本失败后的内置行为fallback。
4. 语法/回调签名/保留符号/API版本/路径/运行期错误可定位到实际用户文件与阶段，失败后修正源文件即可新Run恢复。
5. 两个具有不同逻辑的用户示例，使用同一个未重编译runner，在仓外/不同cwd/搬迁场景目录后可跑；源码改变确实影响Auto输出/最终GPU画面。

支持面继承839/838 Windows单窗、固定相机/同维group平移、128节点、受限GLB及资源/文件/历史/CPU预算。公共API只许改变local translation和local visible，读取group/parent/world/effective等元数据，不能换父/资源或创建销毁节点。

非目标：完整稳定AutoGraphics SDK/新独立仓、core AST语法/VM改造及超出依赖扫描的SDK改造、ACE/native接线、多文件use/include/包解析、任意插件/安全沙箱、图形脚本编辑器/热重载、脚本自动随Save/Save As保存/manifest发布、动态结构/碰撞/材质/动画扩面、生产AutoUI/RQHost/正式Forge。此版是公共API的首个纵向切片，接口版本明确，后续扩展另立范围。

## 2. 架构方案

```text
内置来源 → 原profile6行为（保留原路径/分母）
用户来源 → 路径/UTF-8/大小准入 → 实际Parser AST校验回调/符号/profile边界
  → 已审核公共Auto API + managed bridge + 用户完整编译单元
  → SourceSnapshot（API/bridge/原用户字节/组合身份+来源区间）
  → 显式UserBehaviorV1 / profile7 → 实际Session编译
  → 完整场景配置 → app_start一次 → 受检初始帧
  → 每tick app_update(dt)一次 → 实际Auto local/world/effective输出
  → 839同一consumer/资源/renderer → Pause/Step/Stop恢复
```

L1模块特性，r2增加SDK词法依赖扫描缺陷的有界修复（crates/auto-lang-core/src/{use_scanner,dep_scanner,dependency_scan,lib}.rs及对应定向测试），不改语法/VM/协议/依赖清单。driver实现写`experimental/autographics-auto-driver/`，包括其Auto API文本、加载/装配/Host/Play适配和测试/文档。AGP、其余生产crates、根workspace/锁和迁移树只读。SDK修复在同一840树完成，以独立提交隔离供后续master按范围落地；graphics代码仅autographics-dev。固定SDK来源生成基于旧pin且仅承载已核扫描补丁的独立Git提交，不整仓升级、不直接改.sdk-source。SDK提交/快照均由T09审查，完整合入门仍T10。不造第二parser/VM、通用package loader或世界/ECS；需要核心协议/架构写面则回new，按L2先设计。

| 门 | 成立条件 / 当前状态 | owner_stage |
|---|---|---|
| G839 | 839最终review、冻结完整门、实际合autographics-dev、Spec/归档/guard清理/到期检查完成；当前839 archived10/10、正式落地/归档/guard/到期收据核实，G839满足；旧pending为规划历史 | work T-01前 |
| GSOURCE | 已核正式完整SHA a239653365a0c3451c85aa9921189fcf011bd8cf、SDK/两锁/source278与479包闭包/原反控及canonical；实施冻结见work-r1/baseline.json.txt | work T-01 |
| GSTART | 用户2026-10-24已明确授权840 work；G839满足后已启动 | work启动前 |

唯一未来树`D:/autostack/.wt/lang-840/auto-lang`、分支`plan-840-dev`，显式基于G839后autographics-dev完整SHA。work/internal/final review同树顺序，主检出记Plan/附件。代码合autographics-dev、master仅知识；无junction/symlink，移树前wt-guard。

## 3. 技术栈

沿用最终839的Rust/stock Iced/wgpu及固定SDK（观察`9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`），公开Parser/AST、`Session::new/start/pump`、839 Play/Frame/时钟/输入/资源。用户行为依然是正常Auto函数和表达式，不是Rust识别行为关键词后模拟执行。

拟新增`api/behavior_v1.at`及API文档、`src/authoring/behavior.rs`与定向测试；共享必要managed bridge原语，避免复制一个独立runtime。装配可通过固定、具来源区间的完整顶层单元连接已有单源SDK入口，或现有公开module接口；具体由T-02最多两方案取证。用户模块支持面首版仍只有一个文件，不读取任何未纳入快照的use/include依赖。

## 4. 需求分析与背景调查

### 4.1 授权、知识和来源

r2授权：2026-10-24用户明确要求“先修复SDK依赖扫描对注释和字符串的误判，更新依赖冻结后继续实施，不要缩减原定支持面”。因此在同一840树增加有界SDK写面；不另建agent/聊天，不要求重复批准。扫描正反、源编译与模块fresh/cache共享原因回放为T01a；旧pin精确补丁/完整依赖身份核验为T01b。

2026-10-24用户说“计划839基本实施完毕，请规划下一个计划”。承接前次优先公共API/用户脚本的方向；授权只读调查和新草案，没有实施/合839、新聊天/agent/模型/预算授权。只细化下一切片，不同时分配多个后继编号。

已读取[全局overview](../../specs/overview.md)、[图形overview](../../specs/auto-graphics/overview.md)、[语言SDK](../../specs/language-sdk/project.md)、[Auto入口](../../specs/auto-graphics/auto-source-driver.md)、[场景/资产](../../specs/auto-graphics/scene-assets.md)、[集成边界](../../specs/auto-graphics/integration-contract.md)、[839 Plan](839-autographics-scene-play-mode-and-auto-runtime.md)、[work-r1 readiness](../attachments/839/work-r1/readiness.md)及[internal R1 needs_fix](../attachments/839/review-r1/review.md)。旧readiness并未覆盖Starting取消缺陷，不作完成证明；scene-play-session尚未正式沉淀，先交付再作为840 modify基面。

源码只读候选Git对象`e58828ca64b7e26ce984faf850fb3b1d0887517f`及固定SDK对象；文件/版本hash在[观察清单](../attachments/840/planning/source-observation.json.txt)。源码/正式知识和在途提案分开，不把代码存在或work测试总数当作839已交付。

### 4.2 已观察接口与缺口

| 实际触点 | 已观察事实 | 本Plan对应 |
|---|---|---|
| play.rs::Play::new(Editor,actors,source) | 内部已可接源码并记source_hash，产品仍选固定SOURCE；配置/派生/查询期望profile6 | 不另建运行器，加明确Builtin6/UserV1类型及相应初始化/准入；不能仅把文件塞进source参数就称公共API |
| samples/authoring_play/main.at | reset/identity/append_field/bind_resource/derive和tick动作混在同一内置Auto单元 | 将transport与用户回调分开，用户不操作内部数组/ordinal协议 |
| Host::start_play/dispatch_play | 正式Run取内置SOURCE；另有play_source_for_probe注入 | 增正式UI/CLI来源选择，probe注入不能冒认用户产品入口 |
| Session::new | 已用真实Parser提取root scalar签名，再create_vm_from_source；重复名collect可能覆盖，复杂返回仅None | 在装配前用实际AST明确验证回调/重复/保留符号，不能靠最后一个同名函数覆盖 |
| 839 Frame/consumer | 固定ID/kind/parent/resource/静态属性，只可local translation/visible；actual输出+world/effective检查 | 用户API严格沿同一写面，不借回调放开任意节点/资源修改 |
| SDK module/source接口 | 项目SDK含模块服务，但尚无本消费者已验证的用户模块装配/诊断链 | T-02用固定SDK公开接口证明whole-path，不从主仓新版或私有模块抄加载器 |
| 831/832状态 | 831 executing2/12，832仍drafting并等待831整交付；生产迁移未完 | 不进入迁移准备/实现，I2/I3/master图形源码合入门保持 |

当前差距是“开发者不能仅写业务Auto源码就复用现有场景运行器”。API先从839允许的行为面做实，不宣称这一步已交付通用引擎API、完整脚本系统或生产Forge。

## 5. 详细设计

### 5.1 下一入口与bounded feasibility

下一任务T-01a：同840树、基面4fb2fda5a848478cd2f427c3798a70aef026dc35，修复use/dep共享的注释/字符串状态识别，保留真实声明/参数/顺序/去重规则。T-01b将精确补丁转为基于旧SDK pin的Git来源提交，再重建只读SDK快照及source-lock/baseline/来源清单，核17本地包/29边/锁不漂移，回放r1三个红例与原GPU正例。不得改用户字节、删用例或以自造pipeline替代。随后T-02继续完整可行性，不把解除scanner缺陷当T-02全部完成。

T-02在840树建立新`research/user-behavior-feasibility.md`。最多比较既有公开module装配与固定单元装配两种接法：用户文件只写两个callback和公共API，通过真实Parser/SDK→Session→839 Play→已导入多part模型/2D图形→实际画面→Stop恢复；改用户业务逻辑但runner二进制不变，再Run读到新行为。另做回调签名错/同名覆盖/用户行号、API越界与预算故障/恢复。

探针用scratch API/源快照和最小适配，关闭回调执行次序、用户全局初始化、world读后写一致性、签名/保留符号准入、source区间及错误可定位、profile选择、owner和现有CPU预算可行性，不要求后续正式模块已完成。r2须先T-01a/b核实更新后的固定SDK；除已授权依赖扫描修复外不能靠修改语言核心、代码字符串替换业务逻辑、Rust执行callback或读未快照模块解决。关键契约无法成立则回new，后续相关任务不启动。

### 5.2 公共Auto行为接口 v1

用户必需入口：`fn app_start()`、`fn app_update(dt float)`，返回Unit（省略返回类型或明确void按固定SDK实际语法）；每个入口恰一个、参数精确匹配。允许普通Core Auto私有函数、全局状态和控制流；顶层仅声明，初始化使用有限字面量/纯常量表达式，不允许顶层可执行调用/循环，相关逻辑放app_start。use/include/FFI/异步等依赖装配不属本profile；以真实AST诊断，不用文本/regex误判注释或字符串。

拟议API名称由T-02验证后T-03固定，不得删除以下能力来过门：

| 能力 | v1表面（均int/float/Unit） | 语义 |
|---|---|---|
| 节点发现 | ag_node_count()、ag_node_at(index)、ag_valid(node)、ag_actor(space) | 遍历全部叶子/组，或获取UI绑定的2D/3D actor；缺actor为−1，handle0是合法节点 |
| 元数据 | ag_space(node)、ag_is_group(node)、ag_parent(node) | 同维空间/是否group/父handle，root父−1；不暴露持久u64缩窄或GPU地址 |
| 局部与世界读 | ag_local(node,axis)、ag_world(node,axis)、ag_local_visible(node)、ag_visible(node) | 返回当前回调内已写入后最新的局部/派生值；Auto拓扑计算可用dirty缓存，不读旧frame冒充最新world |
| 有限写 | ag_set_local(node,axis,value)、ag_move_local(node,dx,dy,dz)、ag_set_visible(node,flag) | 仅local translation/0或1 local visible；不改父/资源/类型/scale/yaw/pitch/rgba，2D z规则保持 |
| 输入与时序 | ag_input_space()、ag_input_x()、ag_input_y()、ag_toggle()、ag_tick() | 839当次tick动作快照与逻辑计数，Step中性输入；不读真实墙钟代替固定dt |

Node handle为当次Run的有界ordinal，生命周期随Run结束，不是稳定存盘ID或不可伪造安全句柄。合法parent/actor由启动快照映射；用户不能把名字当身份，同名对象隔离。无效handle/axis/flag在索引前受控API错误；managed bridge保存本callback的首个错误及其节点/参数上下文，后续合法调用不得清掉它，host在app_start/update后先核该状态，非零则整次初始化/tick拒绝并关闭受污染VM。getter的安全占位返回不是成功结果；ag_valid(-1)则为合法false控制。非有限/越界输出最终同样由host整帧拒绝。

业务回调在Auto中执行，API读写及world/effective派生也由Auto实现；Rust仅输入/桥接/检查/渲染。多个set/move的read-your-writes及group→child world/visibility效果必须在同一callback内可观察。不得把ag_move_local实现成Rust帧后补矩阵或只在固定样例名下生效。

接口名和`autographics.behavior/1`有可分发文档/API源/摘要，版本是实验承诺，不宣称完整稳定SDK或永不变更。内部transport/private命名空间与用户声明分离，不能让用户重定义profile/reset/tick等后绕过consumer。

### 5.3 源准入、装配与诊断

正式入口是单个本地`.at`文件，严格UTF-8（不接受BOM/坏字节）、≤256KiB；有界读取后以**该份原始字节**算SHA并解析/编译，不在签名检查、编译、Run之间重新读路径。CRLF/LF均可，原始hash各自如实；API/bridge源≤64KiB，组合单元≤320KiB。沿838本地路径/ancestor reparse规则，拒绝network/device路径，不擅自读取依赖目录。

真实Parser AST须检查必需callback签名、重复全局/函数/类型符号、保留transport/private声明及引用、unsupported use/include与顶层执行，不靠Session的BTreeMap最后写覆盖作为准入。须区分顶层绑定与局部作用域：普通局部值变量可与transport名称同名，不能误当重定义root；保留transport函数调用及private前缀访问仍拒绝，按实际AST绑定/用途判断。注释/字符串内出现保留词不能当声明/调用。保留前缀/具体符号集来自固定API描述，不散落多个不一致正则表。

装配只连接经过准入的完整顶层单元或既有公开模块，固定换行/区域映射；不拼接用户片段到字符串字面量/函数体，不改用户标识符、字面量或函数逻辑，不另写Auto parser。API/bridge逻辑单源，原profile6和新profile7共享可共享原语，避免长期复制两份图/错误处理；旧profile6改动限等价共用，原回归分母保留。

SourceSnapshot含逻辑用户path、原始字节/hash、API版本/hash、bridge hash、组合身份、来源区间、base场景/资源身份；每次新Run固定一份。源文件运行中变化不热载，当前Run继续该快照；Stop后下一Run重新有界读取。API版本不符或组成源漂移明确拒绝，不静默升级/回退；本版不新增用户app manifest。

诊断至少包括实际用户文件、阶段（read/parse/admission/compile/app_start/app_update/output/budget）和callback；SDK提供结构化span时映射到正确用户行/列或具名API文件，不能把prelude偏移行冒当用户行。没有可用span的runtime/native错误保留SDK原文并标明未提供行号，不编造精确位置。缺callback/错误签名等由AST可定位的错误要指到相应声明/缺失入口，失败可修正后新Run恢复。

继承可信同步脚本/native-call边界，不声称安全沙箱、同步编译可抢占或任意用户全局容器有硬内存配额。源码/host/API资源/队列及报告有界，用户私有heap计量边界如实披露；不能把heap对象计数等同所有容器字节。生命周期用合法有界状态示例与真实VM弱owner检验。

### 5.4 适配839运行、正式入口和恢复

内部来源明确为Builtin6或UserBehaviorV1；不能根据返回payload形状猜profile。Builtin维持839内置SOURCE及原语义，新用户来源期望profile7和API version1。profile7 transport由managed bridge提供，user无需定义这些root函数；错误版本/旧Run/资源/不允许字段同839整帧拒绝。

启动顺序：创建私有Session→配置完整局部图/父/资源和neutral输入→app_start恰一次→Auto派生/查询→完整准备和publish tick0。Starting期间窗口取消/失焦/resize等auto-Pause意图必须保存，在完整初始化边界兑现为Paused，不能无条件Running；只有显式Resume恢复，继承F-839-R1-01闭合后的合同。tick时设置当前tick/dt/action→app_update(dt)恰一次→派生和查询/准入→发布；Pause不重入callback，Step推进一次中性tick，Resume不重调app_start。app_start允许改运行副本，但不改Editor；初始化失败留原编辑帧。callback/输出失败后Faulted关闭VM，Stop恢复839保留呈现；源修复后新Run新VM新全局状态，不复用污染VM。

UI Editing中可选“内置／用户脚本”及路径，明确显示实际解析来源/API版本及最后Run源摘要；Starting/Running/Paused/Faulted等活动模式禁止改来源/路径，不排队。Run在读取/解析前执行原busy/draft/存储未确认准入，路径错误不改变选择/文档/history/dirty/文件。原probe专用注入不作为正式用户入口，仍可保留用于反控。

CLI拟支持`autographics-authoring --open <scene.agscene> --behavior <main.at>`；相对behavior路径按scene entry的root解析，未保存新场景只能用绝对本地脚本路径；不临时改变全局cwd。没有behavior则仍为内置样例。现有--play-check/window/其他入口保持；新用户自检命令由T-07给出。

场景schema仍为838正式schema3，不将源码路径/版本/actor选择写进Document；几何Save/Save As不自动复制/保存脚本。Save As到新root后相对behavior按新root解析，UI须显示实际路径，缺文件不回落旧root。开发者可把scene+assets+scripts一起复制，再显式给相对behavior运行；这验证可复跑单文件项目，不冒认自动发布/脚本包manifest。自动源码交付/多文件依赖另立后续Plan。

### 5.5 两个用户示例与同binary验证

新增拟议`samples/user_behavior/move/`和`.../patrol/`，各含用户main.at、说明及所需scene/资产（复用已登记许可的自有GLB，原游戏素材不改）。

- Move示例：通过API获取actor/输入、按dt移动2D和实际导入3D Model/切可见性，使用非零父组，含handle0与缺actor合法控制。
- Patrol示例：用用户私有计数/方向/条件分支实现往返巡逻或定时group行为，输入只影响部分规则；与Move不是只换预设名或Rust速度常量。至少一个示例在同callback group写入后读取child world/effective并据此改变另一个对象，验证API而非固定行为模板。

将两个完整示例复制到仓外普通目录，以**同一runner字节hash**执行；期间不cargo build、不修改Rust/桥/renderer。只改用户Auto逻辑，下一Run的实际轨迹/全mesh GPU像素按独立参考改变；Stop仍恢复原编辑内容。不同cwd、scene+assets+script搬迁且原目录不可用时显式相对behavior可重建。缺脚本/错误版本必须失败，不走内置行为掩盖缺口。

### 5.6 预算、资源和交付边界

沿用839单runtime VM、1inflight/4 tick额度、8ms共享pump目标、callback100000steps/120 resume/250ms、整tick2s/初始化30s、固定1/60 dt、4096标量摘要与Stop/Pause/失焦输入规则。用户API getter/setter/派生和callback都计在同一事务预算内，不逐API调用重置总deadline或再给一份额外8ms。源准入/compile同步成本单独报告；128节点合法控制须在支持预算内实测，失败回new不暗减节点数。

不新增资源格式、GPU model/target/pipeline；warm共用838 immutable Arc。源码选择/校验失败不得留VM；20轮A/B脚本新Run/故障/Stop后实际runtime VM owner释放，Editor/合法外部pin按839保留到最后owner。SourceSnapshot/诊断/用户代码字节不进undo/history或逐tick积累；报告与code范围计量明确，实例重写和模型重传分开。

生产831执行中，832仍待831整交付；I2/I3、RQHost、发行/Shader/CAP未完成。源码仅autographics-dev。本Plan交付可称“用户仅写Auto行为即可复用有限场景运行器”，不能称完整公共引擎/任意脚本产品化。

### 规范增量

全拟议，当前不写canonical；scene-play-session须839先交付。review冻结实际增量，merge再更新overview/plans/goals/ledger。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/behavior-api.md | 无用户API→v1 callback/节点句柄/读写/输入、源profile及版本/诊断/预算/局限 | 业务源码能独立复用 | AC-02, AC-03, AC-04, AC-08 |
| SD-02 | modify | docs/specs/auto-graphics/scene-play-session.md | 仅Builtin6→并存UserV1/profile7、配置后start/每tick update、源快照及原Stop/全帧准入保持 | 保住839生命周期 | AC-04, AC-05, AC-07, AC-08 |
| SD-03 | modify | docs/specs/auto-graphics/authoring-session.md | 无正式来源选择→Editing UI/CLI选择/解析/恢复，schema3与Save不自动含源码 | 明确开发/交付边界 | AC-05, AC-06, AC-08 |
| SD-04 | modify | docs/specs/auto-graphics/auto-source-driver.md; docs/specs/auto-graphics/prototype-profile.md; docs/specs/auto-graphics/project.md; docs/specs/auto-graphics/integration-contract.md | 固定实验行为→v1公开API/两个仓外源码示例及限制，旧profiles/生产门保留 | 不冒认通用SDK/I2/I3 | AC-01, AC-06, AC-07, AC-08, AC-10 |

| SD-05 | modify | docs/specs/language-sdk/project.md | 逐行预扫描误解注释/字符串→共享词法状态识别，仅真实代码行声明参与use/dep解析；真实导入兼容保留 | 修复AST准入与SDK预扫描语义分裂 | AC-01/02/03/07，T-01a/b |

## 6. 测试设计

以下新入口/fixture由work任务建立，不是本轮已跑。未指定实现/internal/final审核模型/新聊天，不创建agent；final review同一840树独立上下文，实施暂停写入，修复回同树。实现会话自查披露上下文局限，不冒充独立final。

未来840树driver目录先按既有prepare_sdk/verify_source核固定来源；定向拟`cargo test --locked --test authoring_behavior -- --nocapture --test-threads=1`和`--test authoring_behavior_host`（T-03..07），加旧authoring_play/play_host/probe_exit与受影响原测试。check/fmt沿用locked all-targets/现有fmt，不跑整档。

新自检拟`autographics-authoring --behavior-check --behavior <file> --report <path>`及`tools/check_user_behavior.py`，T-07建立；--open/--behavior为正式运行入口，不靠自检参数获得额外业务逻辑。命名可等价调整，T-08给准确命令/选面，0selected/skip不能pass。

| case_id | AC / work owner | 代表输入/状态 | 实际入口与observable oracle | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01/07；T-01/08 | 正式839/SDK/锁/资源/新API源、源漂移/缺文件 | verify_source/独立consumer实闭包与原反控；固定SDK/AGP和profiles1..5不漂移，6等价回归 | 不以候选作交付冻结 |
| V-02 | AC-02；T-03 | UTF-8/CRLF/LF、0/256KiB/超界/缺文件/BOM/坏编码、API/bridge版本/摘要变化 | 正式SourceSnapshot/read/assembly；有界读、原字节身份、实际source区间正确，错误不造VM/不改Editor；运行中改文件不热载、下一Run用新字节 | 单文件本地可信源，不含包/网络 |
| V-03 | AC-03；T-03/04 | 正确callback/私有辅助/状态；缺/重名/参数/返回错、保留名/私有引用/use/顶层调用；注释字符串合法控制 | 真SDK Parser/AST→准入→compile；具名错误/用户path/可用span准确，不被collect覆盖；同名局部合法控制不过拒绝，修正后Run成功 | 不自写parser、不声称安全沙箱 |
| V-04 | AC-04；T-04 | handle0/−1缺actor/无效handle/axis/flag、非零父group写后child world/visibility读、普通私有函数 | actual Auto API/callback→Frame→GPU；读后写一致及正确对象/全mesh，非法API整callback Fault保last-good，无Rust位移fallback；真实API/source速度/分支变异在独立轨迹/像素门被区分 | 只local translation/visible可写 |
| V-05 | AC-05；T-05 | app_start一次、Starting窗口取消/正常完成/Stop/初始化失败，600tick/不同显示间隔、Pause/Step/Resume、再Run全局重置，wrong-profile/resource/旧Run、超时/prepare失败 | actual typed Program/Play/consumer；Starting取消后完成合法初始化仍Paused且不派发，Resume才恢复；callback/tick/单步准确，immutable/资源拒绝，失败关VM且Stop/修源新Run恢复，Editor hash/history/dirty/文件不变 | 继承839取消修复/预算/输入/last-good合同 |
| V-06 | AC-05/06；T-06 | UI/CLI source选择、relative行为path、Save As换root、busy/draft/源码变更、文本焦点/键盘 | 正式Host+文件；Editing选择/实际路径正确，活动模式拒绝且不排队，缺源不回落builtin；普通内置无behavior仍原行为，Save不偷偷写源码/配置 | 无script绑定持久化/热重载 |
| V-07 | AC-06；T-06/07 | Move/Patrol仓外目录、同runner SHA，改用户脚本/不同cwd/搬迁原目录不可用 | 正式--open/--behavior与新进程；不同逻辑真实输出/画面匹配独立参考，runner/桥/Rust无重编译或代码变化；源只写callbacks/API无transport | 可复跑单文件项目，不自动发包 |
| V-08 | AC-07；T-04/08 | 20轮A/B/错误源/API越界/CPU Fault/Stop，128节点，外部pin | 真VM/source弱owner、GPU Arc/target/pipeline/实际writes；runtime释放/Editor合法保留、无每Run模型重传；故意retained VM/重复上传负控红；source/compile和用户heap计量边界明确 | 不冒称任意容器硬内存配额 |
| V-09 | AC-05/06/08；T-07 | 原生source路径/Run/不同canvas输入/Pause/Stop，错脚本/漏callback/frames1和实际GPU透明/错资源故障 | 新自检经OS→Iced/实际源装配→Auto→top pixels；具体对象端点/原位置/多part/恢复为独立像素oracle；native/report/aggregate错误非零且partial拒绝，原counter/content并列 | 真实桌面必须可用，非state/hash自洽即可 |
| V-10 | AC-08；T-08 | SD-01..04/公共API签名表/实际CLI/示例/源快照/诊断与支持面 | proposed Spec+可复跑README/handoff，源/版本/预算/生产边界逐项对应case； API名/表/bridge/consumer不漂移 | 静态检查不代VM/原生/像素门 |

每case记command/退出码、实际断言/数量、code/SDK/锁/用户源/API/bridge/fixture身份及原始报告。源码载入不是截图变化的充分证据；必须有用户字节改变→真实Auto输出→具体GPU结果，同一个runner的仓外对照。运行/帧输出故障要在实际对应路径触发，setup/编译失败不冒充成功捕获GPU反控。复用前序证据须证明代码/依赖/配置/输入未变，新user源装配/API不能借旧builtin绿外推。

final review后merge冻结提交统一跑完整locked driver/AGP test+build、原16 AGP cases、旧双游戏/835..839原生正反/partial/reopen/owner、原counter/content两道门、新user来源/公共API/两个仓外示例/实际GPU正反/独立source consumer。T-08登记准确矩阵；开发/review只定向/check/fmt，不跑root整档t/tv/tt/tb/tu/taa/docs_gen或上述完整档。tf只主检出到期批量；如需SDK/AGP/生产写面先回new界定，不能自动扩门。

## 7. 验收标准

| ID | owner_stage | 可观察标准 |
|---|---|---|
| AC-01 | work | G839/GSOURCE、唯一树/固定依赖/源闭包与写面成立；V-01 |
| AC-02 | work | 有界真实用户SourceSnapshot/版本/来源区间/新Run重读准确，坏源不造污染；V-02 |
| AC-03 | work | 真Parser校验callback/保留符号/重复/支持面及可定位诊断，失败修正恢复；V-03 |
| AC-04 | work | 公共API实际Auto读写/句柄/group派生与全帧准入成立，用户不写transport；V-04 |
| AC-05 | work | profile7与Builtin6明确并存，callback次数/运行预算/暂停停止/故障及Editor隔离正确；V-05/06/09 |
| AC-06 | work | 正式UI/CLI与仓外双示例同runner可用、源码改动无需Rust重编译、相对路径搬迁/Save边界正确；V-06/07/09 |
| AC-07 | work | 旧profiles/资源保持，20轮owner/计量/128节点/实际GPU复用成立；V-01/08 |
| AC-08 | work | 真OS/GPU正反/partial、API表/SD-01..04和可复跑诊断/示例/版本/支持交接完整；V-09/10 |
| AC-09 | review | 独立核AC-01..08/Spec及遗漏/延后/workaround/fmt/warnings/debug，版本绑定pass且完整merge门pending |
| AC-10 | merge | 冻结完整门过才落autographics-dev，canonical/ledger/归档/guard清理/到期收据齐备，源码不入master |

## 8. 执行步骤

当前archived11/12；T01/01a/01b/T02..09关闭。T10完整门/实际源码与SDK合入/规范台账/归档提交/fresh guard移树与删分支已完成；仅840到期主检出批量回归及最终跟踪提交pending。历史状态不覆盖本段。

| task_id | owner_stage | 依赖 | 实际触点／新路径与动作 | 关闭条件；AC |
|---|---|---|---|---|
| T-01 | [✅] work | GSTART/G839 | 主合同/839正式收据、840唯一树，prepare_sdk/verify_source/实际Play/锁/资源API | [x] 正式839=a239653365a0c3451c85aa9921189fcf011bd8cf已核archive10/10/完整门/归档guard清理/not-due；唯一840树/branch断言、SDK680/479closure/17local/29edge/source278及原四反控/LF合法控制通过，两锁及资源原样。AC-01基面部分关闭；完整AC-01仍依赖T-08。见work-r1/baseline.json.txt |
| T-01a | [✅] work | T-01 | SDK use_scanner/dep_scanner共享词法状态过滤及定向回归；同一840树 | 注释/普通多行/三引号/反引号/转义/CRLF中的伪use/dep不触发；真实use各生态/dep参数及顺序去重保留；定向scanner及源/模块fresh/cache控制通过；AC-02/03/07 |
| T-01b | [✅] work | T-01a | 独立SDK来源Git提交、baseline/source-lock/prepare/verify与原反例回放 | 旧pin+精确补丁来源可再生，680文件数按实际/17local/29edge与两锁核实；r1三红转绿且GPU正例仍过，冻结完整SHA/身份，禁止直接修改.sdk-source；AC-01/02/03/07 |
| T-02 | [✅] work | T-01b | 真实SDK Parser/单源装配及research/user-behavior-feasibility.md | [x] 扫描阻塞解除后三红转绿，真实Auto两源/全局初始化/API/120tick/全mesh/Stop/owner可行；正式profile7/600tick/双示例/原生与独立consumer也通过。r1 needs_replan保留为历史；AC-02/03/04/05 |
| T-03 | [✅] work | T-02 | 新src/authoring/behavior.rs/API描述与SourceSnapshot/真AST准入/映射；新tests/authoring_behavior.rs | [x] F-840-R1-01/02修复：共享Rules AST遍历覆盖回调表达式返回并守住嵌套函数边界；源序不可变常量/FStr准入及动态/私有负控通过；原复审10/10和行为12测试通过。 原关闭条件：V-02/03及源/符号/签名/路径错误定向通过、用户API声明单源；真实API执行由T-04关；AC-02, AC-03 |
| T-04 | [✅] work | T-03 | 新api/behavior_v1.at/managed bridge、原profile6等价共享原语及API文档 | V-04及V-08 source/VM部分通过；callbacks仅业务、读后写/world/句柄/错误和实际Auto行为正确，不复制独立runtime；AC-04, AC-07 |
| T-05 | [✅] work | T-04 | play.rs Program flavor/初始化callback/consumer及原调度/恢复，相关定向tests | [x] F-840-R1-01修复：18嵌套返回反例受控拒绝，合法helper/closure/init+tick/owner、坏源→修源实际Host全mesh像素/Stop恢复通过；Host4与独立consumer恢复通过。 原关闭条件：V-05、受影响旧play/play_host/probe_exit通过；Builtin6保分母、User7全帧/预算/Stop/Editor隔离；AC-05 |
| T-06 | [✅] work | T-05 | Panel/Host正式source UI、bin --behavior/路径准入、双用户示例目录 | V-06及V-07命令/示例部分通过，源仅Editing选择、Save边界/相对root/修源恢复明确；仓外/OS完整证据T-07关；AC-05, AC-06 |
| T-07 | [✅] work | T-06 | 新behavior-check/aggregate/实际GPU反控、仓外Move/Patrol与source文件修改 | V-07/09同runner/无重编译/搬迁/OS全正反/partial/原生恢复关闭，无环境gap；AC-05, AC-06, AC-08 |
| T-08 | [✅] work | T-07 | source/独立consumer/owner/计量，README/new research/user-behavior-handoff.md及树内Spec proposal | [x] F-840-R1-03修复：Spec SD01..05和8个canonical目标版本/hash核对通过；源码299/SDK681/两锁原样及定向独立consumer通过，完整合入档仍pending。 原关闭条件：V-01/08/10、全work case/定向check/fmt完成，精确API/例子/merge矩阵可用，不跑完整merge档；AC-01, AC-07, AC-08 |
| T-09 | [✅] review | T-08 | 同树冻结code/Plan r2/Spec/真实双示例，主检出独立review附件 | [x] review-r2：三项finding关闭，原10case/行为12/实际Host恢复1/独立相邻8及Spec5ID/8目标通过，受影响AC与复用边界核实；AC-09 pass，完整merge门pending |
| T-10 | merge | T-09 pass | 冻结完整门→autographics-dev；主检出知识/ledger/archive/guard | AC-10实际门/落点/沉淀/清理/到期，改代码先受影响复审再重冻，不提前交付 |

## 9. 复审记录

2026-10-24：stage:merge | PLAN-840:r2 | outcome:in_progress | checkpoint:cleaned | source:autographics-dev@8b62db5cb；SDK master0676dc133；archive/knowledge commit e6371606b | final-guard/down-final-guard均clean，lang840两Git树及plan-840-dev开发分支已实际移除（落地祖先验证+expected-SHA CAS）；独立standalone消费者/复现文件保留，组目录非空不删。实际安装入口debug auto.exe已重建并经真实VM注释/backtick假依赖烟测通过；其他production artifacts按路径/构建日期观察，未冒称部署。仅840 modulo5批量回归及最终状态提交待完成，T10仍pending；见[清理](../attachments/840/merge-r1/cleanup-operations.json.txt)与[产物](../attachments/840/merge-r1/production-artifacts.json.txt)。

2026-10-24：stage:merge | PLAN-840:r2 | outcome:in_progress | checkpoint:landed/ledger_refreshed，archive_commit/cleaned/batch pending | reviewed_commit:1f19395254d7e676bd188e43e3d8f734d57aaa12 | delivery_commit:8b62db5cba54b9bfe7f759f4920b22ccb9b75e88（仅canonical/ledger后继，implementation/deps/config未变）| source_branch:autographics-dev fast-forward | SDK master:0676dc13398f27655b4ecc1c34f70a7e95374736，精确四文件patch-id相同 | 完整63有效入口、16AGP/两道合成门、旧双游戏/创作/层级/资产/Play/owner/partial/reopen、producer/consumer各13用户场景、root5332/163全部通过；最终消费者原生产物重冻后全受影响native族重跑。AGP2x采用PID绑定DPIaware真实OS输入夹具，原点击/重绘/resize/计数及内容断言保持，原失败未冒称通过。台账78→79仅11目标且无关raw项/顶层原样，canonical8目标和module/goals/INDEX回写通过。规范与归档提交/清树/到期回归仍待，未宣称最终merge pass。见[收据](../attachments/840/merge-r1/receipt.md)、[逐报告审计](../attachments/840/merge-r1/validated.json.txt)。

2026-10-24：stage:review | review_scope:final | reviewer_identity/limitations:Codex当前实施会话，无独立会话/模型隔离；从冻结代码、原反例、独立相邻输入及原始证据重建结论，未建agent/chat | plan_id:PLAN-840 | plan_revision:2 | outcome:**pass** | status:reviewed11/12 | worktree_path:D:/autostack/.wt/lang-840/auto-lang | branch:plan-840-dev | reviewed_commit:1f19395254d7e676bd188e43e3d8f734d57aaa12 | base_commit:a239653365a0c3451c85aa9921189fcf011bd8cf | dependency_revisions:SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21/681/479/17/29、两锁/AGP不变 | spec_inputs:review-r2/baseline.json.txt及冻结spec-delta-reviewed.md绑定5增量/8目标版本 | acceptance_results:AC01..09pass，AC10pending_merge | findings:F-840-R1-01/02/03均review_closed，无新合同内阻塞/未批准缩面/遗漏/延后/workaround | checks:原规范反例10/10、behavior12、实际Host/GPU修源恢复1、独立相邻8、Spec5ID/8目标、package fmt/diff/clean HEAD通过；SDK146既有warnings无新增。未变路径按代码/依赖/配置/输入及实际guest身份有界复用，旧原生不冒称新exe重跑。 | evidence:[受影响复审与逐AC](../attachments/840/review-r2/review.md)、[冻结收据](../attachments/840/review-r2/receipt.json.txt) | pending_closeout_gates:全部完整driver/AGP/原16case/旧新native/owner/counter+content/sourceconsumer及适用root档仍merge-owned pending，未运行整档/合入/发布canonical或ledger/归档/清树 | next:**merge**须先在冻结提交执行完整门、核对版本；任一代码修复先受影响复审，不以reviewed宣称交付。

2026-10-24：stage:work | plan_id:PLAN-840 | plan_revision:2 | outcome:**pass** | status:execution_done10/12 | worktree_path:D:/autostack/.wt/lang-840/auto-lang | branch:plan-840-dev | base_commit:a239653365a0c3451c85aa9921189fcf011bd8cf | repair_from:a1da1a00b7a239a872702c948cd4722014ee0797 | code_commit:1f19395254d7e676bd188e43e3d8f734d57aaa12 | dependency_revisions:SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21/681/479/17/29，AGP/两锁/运行器/Auto API源及managed装配不变 | task_ids:T03/T05/T08工作重新关闭，T09/T10pending | findings:F-840-R1-01/02/03均work_closed_pending_review；共享AST遍历按词法边界携带Unit规则、immutable const/let及常量FStr/结构值准入且拒动态/可变/赋值/私有/循环依赖、Spec稳定ID/8目标版本对齐 | checks:复审原10case在旧提交6红/新提交10绿；18回调嵌套返回反例、4合法helper/closure/for-in/Unit程序init+tick、4常量真实初始化值及7负控；行为12/Host4/可行性5、all-targetcheck/fmt/source299与8控制均过，SDK146既有warning新增0；独立consumer仅复制精确5变动文件及identity，自有SDK/target行为12/实际GPU修源恢复1通过。原生4正例user/API/bridge/combined/version逐trace与当前实际guest完全一致，未变运行期证据有界复用，不宣称重跑新exe13项。初次for-init正例及consumer漏设既有线程栈失败保留并独立归因；不增SDK写面/不缩支持面。 | evidence:[逐finding关闭/范围与复用](../attachments/840/work-r3-repair/closure.md)、[冻结收据](../attachments/840/work-r3-repair/receipt.json.txt) | blockers/open_work_findings/required_work_gaps/decisions:none | next_responsible_role:**review**同树对新提交做受影响复审/Spec核验；完整driver/AGP/原16case/旧新native/owner/counter+content/sourceconsumer及适用root档仍merge-owned pending，未执行整档/独立再审/合入/沉淀canonical/ledger/归档/清树。

2026-10-24：stage:review | review_scope:final | reviewer_identity/limitations:Codex当前实施会话，无独立会话/模型隔离，不以角色名冒称独立；按冻结代码/真实反例/原始证据重建结论，未创建agent/chat | plan_id:PLAN-840 | plan_revision:2 | outcome:**needs_fix** | worktree_path:D:/autostack/.wt/lang-840/auto-lang | branch:plan-840-dev | reviewed_commit:a1da1a00b7a239a872702c948cd4722014ee0797 | base_commit:a239653365a0c3451c85aa9921189fcf011bd8cf | dependency_revisions:SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21/681/479/17/29，精确4文件补丁及原两锁/AGP核实 | spec_inputs:review-r1/baseline.json.txt含9项canonical来源hash，冻结提案spec-delta-reviewed.md | acceptance_results:AC01/04/07pass，AC02/05/06/08partial，AC03fail；AC09未pass/AC10pending | findings:F-840-R1-01 callback Unit检查漏过Store/一般表达式内带值return，显式void也实际accept；F-840-R1-02有限immutable常量引用和纯常量backtick/FStr误拒，构成未获准支持面裁减；F-840-R1-03 Spec proposal稳定SD01/02/04与Plan目标映射不一致 | checks:38scanner、9behavior+3host+5feasibility、1probe_exit/10进程场景、source299/SDK681和fmt均过；自选实际SDK/Play反例10项规范oracle6红4绿、replay退出1（原有测试绿不能覆盖新反例）；13原生raw reports及独立consumer原证据逐项核验并按未变输入复用。raw标记非法探针/Windows链接路径/PowerShell编码等实验问题另记，未冒认产品缺陷。实现树复审前后clean；未改代码。 | evidence:[完整复审/逐AC与finding移交](../attachments/840/review-r1/review.md)、[冻结收据](../attachments/840/review-r1/receipt.json.txt) | pending_closeout_gates:全部完整driver/AGP/16case/旧新native/owner/counter+content/sourceconsumer及适用root档仍merge-owned pending，未提前执行/合入/发布canonical/归档/清树 | next:**work**同一840树修复T03/T05/T08后提交及受影响复审；当前executing7/12，合同revision仍2，无新增产品范围。

2026-10-24：stage:work | plan_id:PLAN-840 | plan_revision:2 | outcome:**pass** | status:execution_done10/12 | worktree_path:D:/autostack/.wt/lang-840/auto-lang | branch:plan-840-dev | base_commit:a239653365a0c3451c85aa9921189fcf011bd8cf | code_commit:a1da1a00b7a239a872702c948cd4722014ee0797 | dependency_revisions:SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21/681/479closure/17local/29edge，两锁及AGP不变；SDK规范补丁5e2f15806、重冻e98e210c7 | task_ids:T-01/T-01a/b/T-02..08完成；T-09/T-10pending | evidence:[工作就绪与逐AC移交](../attachments/840/work-r2-behavior/handoff.md)、[冻结身份及定向检查收据](../attachments/840/work-r2-behavior/receipt.json.txt) | checks:实际用户API/source/Host、原Play/Host、600tick/128node/20A/B轮、真实VM泄漏与GPU重传反控、299来源及8漂移控制、仓外独立SDK/target/check/18定向测试与2原生正例/透明GPU反例通过；最终同runner13原生case全部满足（4正例各27phase/6像素记录+9预期失败），不缩支持面。38scanner/5feasibility/9behavior/3host/6oldplay/7oldhost/1corruptconsumer/4completion均过；check/fmt/diff过，SDK146既有warning新增0。失焦暂停、Windows占用、侧栏恢复见证修正及独立consumer默认栈失败均保留原日志；最终consumer按既有32MiBPE栈要求通过。 | blockers:none；open_in_contract_findings/required_work_gaps/decisions均空 | next_responsible_role:**review**，须同冻结revision独立复审AC01..08/SD01..05并明确完整merge-owned门pending；未运行root整档/完整driver或AGP合入矩阵，未复审/合入/沉淀canonical/归档/清理，工作树保留。

2026-10-24：stage:work | PLAN-840:r2 | outcome:in_progress | T-01a/b关闭，当前executing3/12 | 同树plan-840-dev | SDK规范修复提交5e2f15806、driver重冻e98e210c7；可再生SDK来源932ad4877e5968f1aa7227d9dcd7fb69b4c69a21（parent旧9576b4d，仅4个扫描文件不同），681文件/17local/29edge/479闭包、两锁原样；独立artifact ref codex/plan-840-sdk仅固定来源，未宣称已合master。源扫描/模块fresh-cache/真实依赖控制通过，原3红转绿与GPU120tick正例通过，38定向scanner通过，locked all-targets check通过/SDK146既有warning。见attachments/840/work-r2-sdk；无支持面裁减，T02继续，T09/T10pending。

2026-10-24：stage:new | PLAN-840:r2 | outcome:pass | 用户明确授权扫描修复/精确重冻/继续work且不降支持面；新增T-01a/b、SD-05，保留T01完成与r1红例历史；next:work，同一840树顺序实施，T09独立审查及T10完整门pending。

2026-10-24：stage:work | plan_id:PLAN-840 | plan_revision:1 | outcome:**needs_replan** | worktree_path:D:/autostack/.wt/lang-840/auto-lang | branch:plan-840-dev | base_commit:a239653365a0c3451c85aa9921189fcf011bd8cf | code_commit:4fb2fda5a848478cd2f427c3798a70aef026dc35 | dependency_revisions:SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9/680，driver/AGP两锁不变 | task_ids:T-01完成/T-02未完成/T-03..10未开始 | evidence:[可行性与逐case移交](../attachments/840/work-r1/handoff.md)、[冻结身份/命令收据](../attachments/840/work-r1/receipt.json.txt) | blockers:B-840-T02-01（真实Parser无Use/Dep的合法块注释/多行字符串仍被SDK Session/公开module预扫描当依赖；两候选均失败，不能在固定SDK/不改用户源码/单源pipeline合同内闭合） | checks:四case1pass3fail0skip；正例同binary两源码/120Auto tick/非零全局初始化/组写后child world读/全部mesh GPU像素/Stop及owner；3个合同反例红未转绿冒充通过。locked all-targets check/fmt/diff/SDK后核通过，继承SDK146 warnings新增0 | current:executing1/10 | next_responsible_role:**new**，需界定SDK依赖扫描修复前置并刷新840冻结，或明确公开dependency-free in-memory接缝；不得静默裁减合法源码/改写用户literal/复制pipeline。840树clean保留，未启动T03/执行完整门/独立final/合入/规范ledger/归档清理/agent/chat。

2026-10-24：stage:work | PLAN-840:r1 | outcome:in_progress | 用户明确授权840实施，GSTART满足；839已archived10/10，最终收据merge-r1/final-state.json.txt核实，正式autographics-dev=a239653365a0c3451c85aa9921189fcf011bd8cf，G839满足（草案中的pending为历史）。唯一树D:/autostack/.wt/lang-840/auto-lang，branch plan-840-dev，base同正式交付SHA；主检出代码WIP为空，其余文档WIP保留。固定SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9/680文件、metadata479/17local/29edges、原source278及LF/源漂移/缺两类GLB/锁漂移反控均通过。T01进行正式基面核实，T02在唯一树做固定SDK完整单元装配scratch，不修改SDK/AGP/root。实施为单会话，未创建agent/聊天；完整回归仍merge-owned pending。

2026-10-24：stage:new；PLAN-840 revision1；outcome:blocked（G839完整交付未完成，草案可交接）；next:839先修F-839-R1-01并完成独立final审查/完整合入交付，再由用户启动840 work，T-01冻结正式基面后T-02可行性。本轮未实施/建树/准备/build/Cargo或原生测试，未创建agent/聊天。

规划历史drafting0/10；当时T-01..10、AC-01..10、V-01..10、SD-01..04均新合同，非执行pass。静态preflight见[检查记录](../attachments/840/planning/preflight.md)。final review和完整merge门未发生；两轮同因不收敛先诊断漏测/兄弟路径并保留轮次，不能改fixture或暗降支持面过门。

## 10. 待澄清事项

当前无待决产品/批准问题。F-840-R1-01..03已完成工作修复及定向验证，T03/T05/T08重新关闭，见work-r3-repair；1f1939525受影响复审已pass，详见review-r2；当前reviewed11/12，完整合入门禁尚未执行，须merge阶段通过后才交付。

用户已授权r2的SDK扫描修复、精确依赖重冻及继续实施，无待决产品问题。原支持面/AC-01..10/V-01..10/SD-01..04保持；新增SDK规范增量SD-05。若发现超出此扫描修复的语法/VM/架构变更，再据具体证据修订，不能静默缩减支持。

## spec-sync 回写记录

PLAN-840:r2：SD01新增behavior-api；SD02/03更新scene-play/authoring，SD04四个实验/集成合同保留全部旧profiles及生产I2/I3边界，SD05沉淀SDK共享词法依赖扫描。auto-graphics与language-sdk的overview/plans、全局GOAL007/008/overview及INDEX已回写；INDEX按当前canonical59项目再生（包含既有auto-cte卡，不改其合同）。RegimeB无store可达，guarded offline projection备份/六段schema/无关raw项/顶层保护/读回通过，ledger78→79共11目标。原有三条无关断链仅记录，未新引入。源码autographics-dev；master代码仅精确SDK扫描四文件补丁，其余知识/归档。详merge-r1/canonical-audit.json.txt与ledger-projection.json.txt。
