# AutoGraphics 截至 PLAN-851 的阶段总结与优化评估

日期：2026-10-26。结论：**适合作为“独立、可创作、真实 Auto 驱动的 2D/3D 交互原型”里程碑，当前应收口总结/优化。** 初始独立研究目标及后来逐步批准的编辑、行为、查询和有限运动闭环已有交付证据；完整通用引擎、生产 AutoUI 融合和 HIR/ACE 统一仍未完成，按最新决策后置。

这不是重新执行851最终验收，不重开已归档Plan，也不宣称原定所有长期能力已经完成。本报告同时区分最初研究目标、后续已批准范围和长期目标，避免用里程碑命名掩盖缺口。

## 1. 核查基线与方法

- 实现冻结：`autographics-dev@11911a4a6d809bfd515f745b77d00bc8f04f6ba9`；851 revision4，archived/delivered10/10，独立最终R1及受影响R2、完整合入门与guard清理已有收据。
- 主检出调查基线：`e66e2abdee10a2ad9d8db543ea2b201b37cb9fc6`；main仅知识与报告，实验实现仍在开发分支。后续main并发提交不改变本报告绑定的实现。
- 固定SDK：`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`。实际使用旧AutoVM公开SDK/Session；图形没有迁入新版HIR/ACE。
- 已读15份归档计划：829、830、833–840、845、846、848、849、851，以及Design45、图形战略、当前版本/远期路线和相关模块Specs；对照实际Git对象、API、几何/运行/保存/示例代码与关键断言。阶段判断采用auto-plan-review的证据方法，不以勾选数替代行为证据。
- 本轮**没有运行Cargo、构建、原生窗口/GPU或性能测试**。独立重算源码/SDK身份、测试日志、原始报告断言和产物哈希；运行行为复用绑定冻结代码/依赖/配置/输入的既有独立review与merge证据。源检查不能替代一次新环境运行，本报告不作新环境认证或完整代码无缺陷保证。

可复查材料：[本轮审计](autographics-milestone-851-20261026/audit.json.txt)、[只读审计脚本](autographics-milestone-851-20261026/audit_snapshot.py)、[851独立复审](../plans/attachments/851/review-r4/review.md)、[冻结合入收据](../plans/attachments/851/merge-r4/receipt.md)、[实际归档清理](../plans/attachments/851/merge-r4/cleaned.json.txt)。

## 2. 当初目标与实际交付

| 目标层次 | 对应计划/设计 | 当前实现和证据 | 判断 |
|---|---|---|---|
| wgpu共同基础、2D/3D游戏原型 | [829](../plans/archive/829-autographics-independent-research-and-game-prototypes.md)，Design45首批样本 | 自有AGP renderer及资源/时间/输入基础；冻结Bevy Breakout与Godot Squash由本引擎复刻，3D代码含定向光/阴影，示例动画、原模型/材质与音频有对应验收 | 初始有限Windows研究目标已交付；不是Godot/Bevy引擎兼容 |
| 活体UI/2D/3D组合、嵌套输入 | [830](../plans/archive/830-autographics-surface-portal-composition-and-input.md) | Surface/View/Portal、DAG/循环拒绝、UI→3D→UI与3D→UI→2D；真实stock Iced控件离屏采样，同设备、焦点/捕获/IME；共享内容多挂载不重复tick | 有限单窗/平面挂载实验目标已交付；未推成任意网格、多窗口或生产AutoUI支持 |
| 真实Auto驱动两款冻结游戏 | [833](../plans/archive/833-autographics-auto-source-minimal-loop.md)、[834](../plans/archive/834-autographics-auto-squash-and-dual-game-sdk.md) | Auto拥有玩法/状态；公开SDK/Session执行，受检输出驱动实际场景、HUD/声音。冻结轨迹、事件、像素/音频、输入/重开及owner正反例保留 | 原I1实验双游戏分母已完成；不是运行原引擎或Rust替代业务 |
| 可编辑的场景与资源 | [835](../plans/archive/835-autographics-scene-authoring-save-and-undo.md)–[838](../plans/archive/838-autographics-glb-import-and-portable-scene-assets.md) | 选择/平移、有限同维层级、属性/显隐、撤销/重做、静态多part GLB、内容寻址资源、保存/失败恢复/搬迁重开 | 作者工具研究雏形成立；生产AutoForge未交付 |
| 当前编辑场景运行、用户行为和项目绑定 | [839](../plans/archive/839-autographics-scene-play-mode-and-auto-runtime.md)、[840](../plans/archive/840-autographics-public-behavior-api-and-user-scripts.md)、[845](../plans/archive/845-autographics-bound-behavior-and-portable-projects.md) | Run/Pause/neutral Step/Resume/Stop；真实app_start/app_update和公共图/输入接口；接受源码/角色、历史、scene4源码/资产闭包与新进程搬迁 | 可编辑→绑定源码→运行→保存/重开已闭环；脚本仍是旧VM有限单文件行为profile |
| 位置感知、方向感知与体积路径查询 | [846](../plans/archive/846-autographics-spatial-queries-and-trigger-interactions.md)、[848](../plans/archive/848-autographics-segment-casts-and-nearest-hit-interactions.md)、[849](../plans/archive/849-autographics-swept-aabb-and-stop-before-impact.md) | 实际all-mesh AABB代理、包含/重叠、有限线段最近命中、固定体积扫掠、类型化结果快照；Zone/Beam/Sight/Dash实际Auto项目及OS/GPU正反例 | 有限空间查询目标完成；不是精确mesh碰撞或完整物理 |
| 两接触移动与滑动 | [851](../plans/archive/851-autographics-bounded-move-and-slide.md) | actual Auto最多两次方向性扫描＋最终f32 local+parent准入；留隙/贴墙/墙角、5类正常状态及严格Fault边界；Maze2D/Corridor3D | 当前阶段最后一个功能切片完成；功能与原预算均有冻结证据 |
| 完整AutoUI组件/主题、统一HIR方言、ACE/HMR、生产RQHost/Forge/发行 | Design45 §11/§12、Design47、原较大版本目标 | 当前节点/校验仍由Rust与领域实现定义；无新版HIR/ACE正式接线；生产组件兼容与宿主协议未认领完成 | 尚未达到；2026-10-26明确等ACE开发完成后再讨论，不作本阶段交付 |

“达到当初目标”的准确表述是：**研究/独立原型合同达到，长期完整体系目标尚未达到。** 早期版本路线中较大的生产融合目标确实未交付，后置是用户显式排期决定，不应写成已经完成。原有样本、兼容和验收分母没有因此删减。

## 3. 本轮独立核对的证据强度

1. 15份计划均为archived、实际步骤完成；早期829缺少后来新增的completion_kind字段，由其11/11、冻结源码祖先关系及交付记录判定，不把旧格式当作未交付。核对了10个关键交付源码引用在当前图形冻结提交中的祖先关系。
2. 851来源清单的**430项源码/资产哈希和目录集合完全一致**。从固定Git对象在内存重建**681项SDK文件身份**（含准备阶段生成的workspace manifest和固定锁），无不符；没有prepare SDK、写入SDK或触发构建。
3. 源码中10个绑定项目的scene4声明与实际Auto原始字节长度/SHA相符：Move、Patrol、Zone2D/3D、Beam2D/Sight3D、Dash2D/3D、Maze2D/Corridor3D。项目验证不等于所有任意用户程序已支持。
4. 851合入记录87次尝试归并为**85组最终有效门**，最终exit与expected一致、日志在场。原生产端/独立own-SDK与空target消费者日志各**280 passed / 0 failed / 0 ignored**，本轮按76个Cargo二进制/Doc-test块重算一致。asset_feasibility会启动portable_child，不能把嵌套libtest摘要重复当唯一测试。
5. **AGP 23个证据槽、163 tests/16 native是明确复用**，不是851重新执行。本轮检查正式849→851整个AGP无差异，并核各记录所指证据源提交与当前AGP一致；原87图形语义槽不是与85组命令简单相加的测试数量。
6. **2463项合入原始文件哈希**与记录相符。两端motion各14主例（6正向/8预期拒绝）及各4次额外独立进程重开：重新检查实际report的完成阶段、API/用户源码/runner身份、trace及全部part像素断言；透明/漏part/错资产在phase5内容门拒绝，坏源码/API/缺资源在phase0拒绝，partial未完成不能绿。
7. 两端实际测试过的motion runner仍在仓外保留，SHA分别为生产端`34b83e50a626b76cf93f8abfbd160d803d0045be84b5b144b2a257d52ab9a246`、消费者`337a5162d47f9a2f22478832f84a44c820c9405de4c16ccb9c64278e87ddd71d`，本轮读回校验一致；没有拿owner反控之后重建的其他二进制替换旧证据。
8. 原owner反控区分源码/VM/Descriptor保留、重复顶点扫描/上传和内容错误，6个负例命中具名断言、恢复正例通过；64回调固定存储和20轮混合生命周期是冻结验收的一部分。本轮复查报告与源码，不冒称重新跑了这些回调。

原merge捕获的旧测试误判“/5未知”已经修为/5接受、/6拒绝并经受影响独立复审；完整两端门随后重跑。中断consumer日志及重建runner身份差异有恢复记录，没有当作成功结果。这里没有新增需要重开851的合同内缺口。

## 4. 里程碑的边界

建议采用描述：**AutoGraphics独立交互原型阶段（截至851）**。它证明了共同GPU基础、真实语言驱动、创作/项目闭包、运行控制、查询与有限运动能在同一研究体系内衔接，足以停止当前功能扩展并进入质量收敛。

必须同时带以下限制：

- Windows x64、单原生窗口、原键鼠/设备profile；现有证据不外推Linux/Web/mobile、多窗口或任意GPU性能。
- Surface/Portal证明有限平面挂载与嵌套机制；不是完整生产AutoUI组件/主题体系，也不是任意深度/任意网格UI。
- 作者场景最多128节点；支持同维平移/显隐层级，运行时公共行为只改有限local/visibility，拓扑、形状、资源、scale/angles不动态变更。
- 静态GLB导入拒绝images/textures/animations/skins、morph及外部URI等不支持内容；当前renderer有定向光/阴影，未因此成为通用PBR/材质/骨骼动画系统。Squash专用示例中的动画与完整通用动画库须分开认领。
- 通用查询是AABB代理，移动最多两接触；重力、地面/跳跃、台阶、解穿插、刚体/完整CCD等仍无通用公共能力。
- 原Breakout/Squash通过专用profile/protocol暴露其玩法、对象生命周期、HUD/声音。**两款完整示例已复刻，不等于公共behavior/5已经能表达任意游戏。** 公共行为API的动态对象、通用音效/HUD/材质/相机等仍需以后按实际需求设计。
- 原100000steps/120resumes/250ms callback、30s初始化/2s tick/8ms pump和32credits是有界支持合同；60Hz是逻辑dt，不是任意128节点/32credits负载的实时性能承诺。同步compile/native/GPU不能硬抢占。
- 旧AutoVM固定SDK仍是当前执行环境。Atom＋Schema定义的HIR、AutoUI/图形方言统一、ACE增量/HMR和生产融合等ACE完成后再议，不挤入本阶段优化。

因此，可以称“原型阶段完成”，不宜称“完整AutoGraphics引擎完成”“通用游戏引擎SDK稳定”或“AutoUI/ACE集成完成”。这些长期差距不应被伪装成851实施遗漏，也不能靠给未来工作取名字就算已解决。

## 5. 优化观察与建议范围

以下是下一轮质量工作的候选，不是已证实的新运行时bug；除文档链接修正外，本轮没有实现这些优化。

| 编号/优先顺序 | 证据与问题 | 建议成果/关闭条件 |
|---|---|---|
| OPT-01：先收敛行为源生成与版本兼容 | `behavior.rs`1214行，286–389附近按版本组合、压缩并多处replace trusted transport/API/ENTRY；v5又增加leaf同步路径。当前有身份/旧版本测试，未发现已证实的错误，但今后修改受多个版本/字节合同牵制 | 显式整理版本构成和可审计生成边界，保留v1..v5现行managed身份/合法源码/诊断/错误与预算；等价整理若改公共身份需明确版本合同。源码长度/编译CPU与case兼容一起核，不把展开数学机械抽helper导致VM成本回升 |
| OPT-02：建立同条件性能/资源基线，再选热点 | 851曾多轮触100000步，最终难例观察到98次yield，不能推断精确steps或统一余量；v5 managed49223/65536B，约75%，固定29标量116逻辑B，实际NanoValue232B/String结构696B/键容量408B另记 | 用同设备/构建配置/脚本/负载区分compile、Auto计算、输出读取/准入、renderer prepare及完整tick，记录cold/warm P50/P95和峰值/owner、扫描上传。收益基于前后对照；没有GPU timestamp就保持不可测，不拿逻辑额度代替耗时。逐标量桥成本是待测假设，不先认定瓶颈 |
| OPT-03：缩小原型宿主与运行模块的耦合面 | `host.rs`2584行、`play.rs`1126行、bin入口1054行；编辑命令、绑定/运行控制与宿主展示集中。AGP host还包含较多专用case代码，这是实验积累，不直接构成错误 | 沿已有职责拆可写driver模块、整理状态/事件入口和测试支撑，保持一套Editor/Play/Descriptor/资源owner及原状态机，明确实验专用代码；不复制第二运行框架、不借此启动生产AutoUI/ACE迁移 |
| OPT-04：收敛证据与工具的产物身份管理 | 85组最终门/23复用槽与多版本源码清单已形成；851曾出现owner反控重建runner、consumer中断、未知未来版本测试滞后，最终均修复并冻结 | 保留原断言/完整分母，将测试选中ID、来源/runner SHA、正负oracle阶段和复用理由做清晰单源交接；尽早固定不可变native runner，避免工具覆盖已测产物。失败必须传递、0selected/skip不能绿；不新增庞大通用调度框架 |
| DOC-01：文档导航和当前能力表 | 本轮初检发现当前路线5页共8处计划链接未随归档更新；829归档里有37处、830有4处相对链接不通。模块部分卡片按历史段落追加，容易把旧schema/profile或pending读成现状 | 本轮已修8处当前路线链接，未改归档合同。后续在现有Spec/README归并清晰的当前能力/限制、API版本/profile及运行入口表，历史证据保留；旧归档相对链接另按来源审计修复，不以断链说明实现未交付 |

源码行数是该冻结Git树的文本观察，含内联测试/诊断，不等于纯生产体积或独立性能问题。原固定SDK146条预存warning与driver新增0分别记录；当前优化不扩写旧SDK或压缩/删验收来制造瘦身成绩。

## 6. 本轮已完成的文档优化

已把Design45、图形战略及v0.6路线中共8处829/830/851旧活动区链接改为真实归档路径；修改清单见[doc-fixes](autographics-milestone-851-20261026/doc-fixes.json.txt)。发现前状态保存在[audit-before-doc-fix](autographics-milestone-851-20261026/audit-before-doc-fix.json.txt)，修正后再次核当前导航。

未修改归档Plan、canonical Specs/ledger、任何Rust/Auto实现、SDK/依赖/锁、API/预算/原始验收报告；没有创建实施worktree、新Plan或执行功能优化。历史Plan的断链保留为维护候选，不影响本轮已独立校验的实际证据路径和Git对象。

## 7. 推荐下一步

接受851为独立研究里程碑，先冻结本报告的能力/限制和优化候选。下一份优化合同优先选择行为源/版本构成、driver职责和证据工具的有界整理，性能收益按同条件数据认领；原行为、支持矩阵、数字/owner/合成两道门及完整合入门保持。性能基线与热点确认也应由优化合同承担，不在本次文档核查中假造新实测。

完成总结/优化后再讨论独立能力建设，不自动续建“额外3～5份计划”。AutoGraphics留autographics-dev；AutoUI归仓/共同HIR声明、ACE替代适配和生产集成等ACE自身开发完成后重新讨论范围与验收。本报告不提前放行这些工作。
