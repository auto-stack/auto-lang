---
plan_id: PLAN-833
status: archived
feature_name: 真实Auto源码驱动AutoGraphics的最小闭环
plan_revision: 2
author: [Codex]
created_at: 2026-10-22
updated_at: 2026-10-23
current_step: 10
total_steps: 10
completion_kind: delivered
depends_on: [PLAN-830]
supersedes_spec_components:
  - docs/specs/auto-graphics/project.md
  - docs/specs/auto-graphics/prototype-profile.md
  - docs/specs/auto-graphics/integration-contract.md
new_spec_components:
  - docs/specs/auto-graphics/auto-source-driver.md
touched_goals: [GOAL-007, GOAL-008]
affects: [auto-graphics, language-sdk-consumer]
---

# [PLAN-833] 真实 Auto 源码驱动 AutoGraphics 的最小闭环

## 0. 变更摘要

**计划833已交付归档：`archived / 10 of 10`，merge outcome: pass。** 独立R2绑定77302d855；完整driver17/17、AGP162/162、16项release探针与当前源码隔离搬迁门均通过。源码fast-forward至`autographics-dev@540c1e70f`；master仅沉淀规范/ledger/计划证据，ledger v68→69。fresh wt-guard clean后，唯一工作树、plan分支和空组目录已移除。独立Windows包运行须保留原受审32MB栈profile，完整命令见README与profile。详[门禁汇总](../attachments/833/merge-r2/gates-summary.json)与[清理收据](../attachments/833/merge-r2/cleaned.json.txt)。

承接829的游戏基础与830的组合/输入，新增独立语言消费桥：真实 `.at` 经已交付 `auto-lang-core` 编译成AutoVM程序，持续接收窗口动作/固定tick，输出图形状态与事件。先做小场景，再使Breakout的运动、碰撞、消砖和计分在Auto中执行；Rust保留宿主、renderer、资源和音频。

本Plan是**实验I1的首个切片**。原I1双游戏分母保持；Auto Squash、生产AutoUI/RQHost、Forge/发行、AutoShader/CAP继续pending，不因“真实Auto已画图”认领完整引擎/新版ACE。

## 1. 目标

1. 独立消费者仅通过纯SDK公开接口编译/调用真实源码，形成可持续实例，参数/结果/状态/错误有可验证语义。
2. Auto创建/更新/销毁基础对象，接收真实输入；至少一个2D小场景和一个已有3D基础对象控制场景可观察。
3. 冻结Breakout的布局、玩法状态、移动、碰撞、消砖/计分由Auto执行；既有Rust Driver只作对照，完成游玩、宿主reset和退出。
4. 共享挂载不重复tick，输出准入/错误/取消/资源寿命有证据；合法修改Auto源码确实改变图形输出。

profile：Windows x64、单原生窗口、键鼠；沿用830最终wgpu/Iced/Portal与既有资源版本。Core场景Auto host计算通过已交付SDK/AutoVM执行，GPU仍用既有WGSL。

非目标：改语言语法/编译器/VM语义、修高层解释器call桩、AC/AV/AJ/AAC接入、AutoShader/CAP、全局native图形目录、通用脚本安全沙箱、异步/网络/文件IO回调、热重载、Forge编辑器、移仓/生产入口/发行。SDK缺口具名回交，不能把玩法留在Rust或改golden代替闭合。

## 2. 架构方案

```text
.at → auto-lang-core解析/编译 → 实例私有AutoVM/状态
                                  ↑          ↓
窗口 → 830实际路由 → 标准动作/tick → 窄adapter → 受检输出
                                                ↓
                       AGP Scene/资源 → 830内容/组合 → wgpu
```

- 拟新增独立workspace `experimental/autographics-auto-driver/`，依赖AGP及显式受管纯SDK来源，自有manifest/lock/target。AGP core不反向依赖SDK、AST、VM Value或Man。
- 优先由宿主调用Auto并拉取纯数据，不用自定义native。高层 `AutoInterpreter::call()` 最终为Nil桩，不能作真实回调；首选 `AutoVM::call_fn_by_name_cpu_slice` / `resume_fn_by_name_cpu_slice`，由T-02实证其初始化/参数/状态路径。
- VM留创建线程，每脚本实例一个状态owner；GPU归host。边界只传值、带session/代际的桥句柄、动作/事件，不传GPU引用，不跨线程发送Rc VM。

### 2.1 来源门与冻结时机

| 门 | 事实与责任 | 当前状态 |
|---|---|---|
| G830 | 830最终review pass、冻结merge-owned门通过、实际合入autographics-dev，Spec/交接对应源码 | 已满足：R5 pass、162/162全测与release/六surface/兼容/重定位门，archived delivered 10/10；冻结764e87559cff，原树已清理 |
| GSDK | 明确已交付SDK SHA/公开符号、manifest/build/include输入、依赖/资源锁与无UI/Man闭包 | 已核实并冻结：master@9576b4dad5ce；递归normal/build/dev及optional本地路径闭包17包、29边，禁入包0；根manifest/锁和来源hash随附件保存 |
| GSTART | 用户要求核源、冻结并交给另一agent；实施agent收到执行833指令后消费本r2 | 已消费：本会话收到实施833授权并完成work；无须再次请求来源放行 |

**当前唯一实施来源（2026-10-22冻结）**：

- 图形/建树基面：`autographics-dev@764e87559cff016d9e0d6b2141e5f1f9903398c5`；AGP subtree `fdfdf351e91f52ff1848bff51134eb85a5f8928f`，其锁SHA256 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`。tip与830 R5受审/交付提交完全相同。
- SDK只读来源：本仓已交付master提交 `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`，只提取纯SDK闭包及输入，不整体merge该提交。SDK目录最近变更为 `9c7e44855c88b936835d57111c06e629e30de0bf`；本次确认它及闭包源码与固定源无差异。
- SDK根Cargo.lock未跟踪于Git，不能靠checkout复原；已保存[精确锁字节](../attachments/833/baseline-r2/sdk-Cargo.lock.txt)，SHA256 `5192d08d502a8f17e277ad8912f5884175659bef642b614e25b394243d1ff091`，274244字节。原始[根manifest](../attachments/833/baseline-r2/sdk-Cargo.toml.txt)及[完整基线清单](../attachments/833/baseline-r2/baseline.json.txt)一并保存。

冻结的是**上游来源**，不是未来driver最终编译锁/实现结果。T-01仅重核和准备这些固定输入，不能自动追随最新master/autographics-dev；T-02探针和T-03可搬动SDK载体/独立消费锁仍为未执行任务。r1观察来源和旧pending结论保留为历史，不再作为当前开工指示，见[r2交接](../attachments/833/planning/handoff-r2.md)。

拟定唯一实施树 `D:/autostack/.wt/lang-833/auto-lang`、分支 `plan-833-dev`，从上列764e87559完整SHA创建，不能使用默认从master起步的建组捷径；work/internal check/final review共用同树。**开工必须读取主检出本r2及baseline附件**（历史图形代码基面未必含当前Plan），不得用旧树计划覆盖主检出。审毕代码合回autographics-dev，master仅沉淀文档；生产迁移门保持。r2核源会话未创建树；当前work树/分支及冻结HEAD见§9。

## 3. 技术栈

复用GSDK的 `auto-lang-core`、单源 `auto-val` 与公开VM接口，沿用G830的wgpu/Iced/音频版本和资产，不升级依赖。新driver自有锁，AGP原manifest/锁原则上不改；必要消费出口只含纯图形数据。

`VMConvertible`、Task/VirtualRAM的使用封装在桥内，错误保留编译/调用/协议准入/图形消费阶段；stdout只作诊断。新CLI（拟名 `auto-graphics-driver`）、tests/fixtures为未来新增，由T-02/03/04落实，草案不声称现已可运行。

## 4. 需求分析与背景调查

### 4.1 授权与依据

本节前段记述r2核源/new会话历史；当前用户已明确授权本会话实施，work完成与冻结交接以§0、§8、§9为准。

2026-10-22 r1授权为SDK调查与草案、等待830交付。当前用户已告知830验收合入，明确要求核对前置、冻结833实施基线并确认可否交另一agent；本r2据此完成来源与交接放行。未指定实施/审核人或模型，本会话不代开聊天/agent或实施，不扩大生产集成/上游写面。

从[overview](../../specs/overview.md)、[SDK Spec](../../specs/language-sdk/project.md)、[图形project](../../specs/auto-graphics/project.md)/[profile](../../specs/auto-graphics/prototype-profile.md)/[交接](../../specs/auto-graphics/integration-contract.md)、Design45（规划时的主检出只读背景文件，未受Git跟踪；不作为本次规范发布物）取材。r1的[接口/缺口调查](../attachments/833/planning/sdk-interface-investigation.md)与[观察指纹](../attachments/833/planning/source-observation.json.txt)保留为历史。本次再核[830归档/收据](830-autographics-surface-portal-composition-and-input.md)、当前surface/nested-input规范与SDK代码，证据见[r2交接](../attachments/833/planning/handoff-r2.md)/[冻结清单](../attachments/833/baseline-r2/baseline.json.txt)。仅复用既有冻结门证据，没有重跑实现验收。设计章和部分SD标题仍带旧draft字样，以已发布版本/hash、R5和merge终态为当前来源，不修改其原生产迁移门。

### 4.2 调查结论

| 现状 | 对本Plan的约束 |
|---|---|
| 814已交付纯SDK，独立fixture有source→VM等5项既有证据 | 不再等完整语言重构才能规划；但这5项不证明持续图形会话 |
| 根 `create_vm_from_source` 返回可保留VM，分片驱动/转换公开 | 有限窄adapter可评估，先whole-path而非先增语言语法 |
| `interpreter/vm_interpreter.rs::call` TODO、恒返Nil | 禁用该便利路径，833不修SDK全局API；如需SDK变更必须具名回owner |
| `SegmentOutcome`有Completed/Runnable/Parked；deadline只明确覆盖task-loop sweep | 宿主需有界pump/终态，耗尽不能假成功，deadline不能代替直接分片预算 |
| 图形分支基面早于SDK，SDK有workspace/build输入 | 固定两个来源并显式准备；不能整体合master或隐式读主检出 |
| `core/scene.rs::Scene::submit`逐条处理 | 桥整包原子保证由桥暂存/校验实现，不能直接套用旧接口名宣称成立 |
| Breakout有冻结f32顺序/步长与golden | 先实证Auto数值/状态，不能为通过放宽既有容差 |
| I1分母为双游戏 | 本次仅首片，Squash真实Auto及I2/I3仍pending |

## 5. 详细设计

### 5.1 首个依赖就绪任务与有限探针

**T-01..08已完成；R1曾重开T-07/08，修复后独立R2通过，见§9。以下是r2冻结的开工与探针合同。** 先读取主检出本r2及冻结附件，确认实际树/分支/固定来源和锁/hash，准备来源而不重新挑选最新版本；G830/GSDK不再待上游发布。等效路径修正记来源；若需改SDK/语言语义、精度或迁移写面，先提出具体缺口和重规划，不自行扩大。

T-01从9576b4dad5ce固定Git对象提取只读SDK源及输入，并使用本r2保存的根锁字节，供T-02最小外部fixture；不得借用原830树（已退役）、上游WIP或活体主检出。T-02先建立该fixture必需的临时独立manifest/锁，不依赖T-03尚未实现的载体工具；其路径/hash明确。T-03再将选定接法整理为正式workspace/受管载体并完成A→B验证。

T-02最多比较两种公开接法：(a)带类型的Auto状态/输出经公开VM值转换传递和保留；(b)同一Auto实例以有限标量回调/查询导出等价状态/命令，计算仍在Auto。不得用stdout、另造解释器或Rust玩法替代。

新增 `research/sdk-driver-decision.md`：记录真实.at、解析/编译/初始化/回调/结果消费符号、参数顺序、类型/状态owner/RC、失败/退出、执行预算和cold/warm调用开销。探针覆盖非交换异质实参、两次状态更新、真实f32路径/对象输出与合法源码变更，并有失败对照。复合返回若不可靠可选标量封送，但行为、玩法和数值分母不变。

两接法都不能满足公开调用、有状态驱动、原数值要求和有界退出时：T-02记SDK owner缺口，T-03之后停止依赖实现，保持未完成并回new；不改VM、砍验收或把未解问题写成已支持。

### 5.2 显式来源与写面

新增workspace内的manifest/锁、脚本、bridge、CLI、tests/research/source-lock/README；AGP按必要增加无语言类型的消费出口。**禁止修改crates/**、根Cargo/锁、生产UI/Man/CLI、其他仓与上游在途树。

T-03从冻结GSDK源生成只读纯SDK镜像，保留相对布局、实际消费的workspace继承值、根锁和build/include输入；记录文件hash/投影规则。原根manifest的iced_widget patch未被纯SDK消费，须登记为不适用并从投影workspace排除，不能因此携入生产UI patch；AGP继续stock iced_widget 0.14.2。投影不改变SDK源码/依赖语义，根原件/hash与投影结果分开记录，复用814无UI镜像模式。不假称当前已有通用exporter：局部工具由T-03在新workspace实现，不重写792/815/831生产发行设施。替换已冻结SDK版本必须记录等价证据和计划修订，不能自行改用最新包。

生成源码不作为第二份可修改实现；生成目录gitignored，清单/准备工具/重建步骤跟踪。消费包A→B不带target/rlib、不可访问producer；源锁包括本地根Cargo.lock实际hash（可能不在Git）。闭合optional path与build扫描/include_str输入，不靠禁feature/删测试伪装纯SDK。所有准备在833唯一树内，不新增第二个同仓实施worktree。

### 5.3 数据、调度与所有权

下列为待T-02落实的概念操作，不宣称SDK已有图形模块。

| 规则 | producer→consumer、状态和关闭条件 |
|---|---|
| init/action/fixed_update/read_output/shutdown | .at→SDK→保留VM/Task→桥；每实例编译一次，Auto-owned状态跨回调保持；返回类型/错误受检，Nil桩不得假成功 |
| 实际输入 | 830返回surface/local/焦点→桥动作→Auto；保session/序号，UI已消费的键不进游戏；不能命中后改用预置坐标投递 |
| 单次推进 | host固定tick→Auto；Breakout每1/60s推进一次，共享两挂载/渲染多帧不加速；每实例仅一个运行中callback，队列有界不重入 |
| 输出与句柄 | Auto状态→桥解码/准入→SceneChange/DrawCmd/音频→原renderer；桥只封送/呈现，不计算玩法；session/代际映射不暴露指针，旧代/跨实例/未知句柄拒绝 |
| 输出发布 | 桥先验证版本、操作、有限数值、范围、资源/对象身份，再整体发布该次输出；坏包不留下前半scene/score/audio；旧Scene::submit语义保持 |
| 长计算/关闭 | CPU slice dispatch/resume→有界pump→完成提交；Runnable不算Completed，仅完整成功发布；错误/超预算保last-good，close后不执行/提交；清Task、VM、队列、映射及最后owner |

首批仅同步纯计算callbacks，显式IO/async等待不在图形脚本API。Parked为受控unsupported-wait终止并清理，不忙等。**不是安全沙箱**，不承诺恶意native/FFI或阻塞系统调用抢占。T-02按样本冻结对象/输出命令/队列/每轮预算及超限诊断；不得用任意语法深度上限掩盖实现缺陷，不承诺无限资源。

### 5.4 固定场景与玩法

| Case | Auto负责 | host及观察 |
|---|---|---|
| A-01 小场景2D/3D | 创建/改变位置颜色/销毁，输入驱动状态和输出 | 复用quad/现有mesh与830 Surface；状态/像素同步；改Auto参数可区分，不称新3D游戏 |
| A-02 Auto Breakout | 冻结56砖布局、球/板移动、边界/逐碰撞顺序、消砖/计分/音效事件 | 原renderer/字体/音频；原canonical replay/golden按原容差，事件精确；Auto模式不能调用Rust BreakoutDriver推进 |
| A-03 实例/挂载/输入 | 两脚本实例隔离；同实例两Portal只tick一次；reset清状态 | 窗口→830真路由→Auto→图形；文本持焦点时游戏不响应，游戏获焦点后键有效；重开拒绝旧事件 |

Breakout底墙反弹、无生命/掉球/game-over，reset为宿主扩展，沿用829规则。f32精度/运算顺序在T-02先证；不得改金样或改成近似游戏。Squash现有Rust路径只做兼容，不算Auto交付。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/auto-source-driver.md | Rust驱动→有限AutoVM host profile、真实入口/数据/预算/错误/寿命/样例 | 首个语言桥 | AC-02,03,04,05,06 |
| SD-02 | modify | docs/specs/auto-graphics/integration-contract.md | I1 pending→仅新增小场景/Auto Breakout证据；双游戏整体及生产I2/I3保持pending | 保实际分母/owner | AC-07 |
| SD-03 | modify | docs/specs/auto-graphics/project.md, docs/specs/auto-graphics/prototype-profile.md | 增实际Auto入口/有限平台/测量，保830事实及未支持面 | 来源绑定 | AC-01,06,07 |

均为provisional。830已交付，其surface-composition/nested-input及profile/交接增量为现行输入，833须保留，不用r1的829快照覆盖；SDK不修改，本Plan不把高层call写成已修。模块plans.md与ledger/index为merge常规关联簿记。

## 6. 测试设计

**V-01..08及独立R2已通过；当前执行/复用边界以[R2复审报告](../attachments/833/review-r2/review.md)为准。** 下表保留r2合同；V-01..08 owner_stage=work，fixtures由对应task建立。最终review由独立于实施的上下文在同一冻结树核验。6项图形/57兼容本轮独立执行，11项不变验收及native/SDK-only来源证据有界复用；完整merge门仍pending。

| ID | AC / invariant | 实际入口/输入与使其可运行的task | 可观察oracle/反控与覆盖边界 |
|---|---|---|---|
| V-01 | AC-01来源/闭包 | T-01/03准备入口→固定载体→外部consumer；隔离producer、A→B无target | fresh+locked执行成功、metadata本地来源在包内、无旧auto-lang/UI/Man；缺文件/错hash/错锁非零；仅实验消费 |
| V-02 | AC-02真实调用/值 | T-02/04，.at→同一VM→非交换异质参数/重复更新→结果 | 数值、参数顺序/类型、状态与栈平衡；错函数/参型/返回拒绝，Nil不得成功；合法改源码改变结果；所用f32/容器RC必须实证 |
| V-03 | AC-03图形消费 | T-05 A-01脚本输出→Scene/renderer→像素 | 位置/颜色/销毁状态和像素同变；丢弃/篡改真实脚本输出使正式门红；不扩材质/动画 |
| V-04 | AC-04 Auto玩法 | T-06 A-02原replay→Auto update→事件/像素/声音请求 | 原轨迹容差、碰撞/计分事件精确；改脚本速度/屏蔽碰撞被正式门识别，调用图无Rust sim代跑；不计Squash I1 |
| V-05 | AC-05真输入/单tick | T-07窗口→830实际surface/local/焦点→Auto；两实例/共享挂载 | 目标改变/他实例不串，文本消费阻止游戏，共享不加速；错误坐标/焦点/重复投递变异使正式断言红；单窗原Portal profile |
| V-06 | AC-06失败/寿命 | T-04/07合法后坏句柄/NaN/错版本/混合好坏包、长循环、reset/close/reopen | 拒绝不半提交且last-good可用；循环实际进Runnable后有界错误/取消，UI继续响应；旧session迟到输出拒绝，最后VM/task/桥owner归零；不承诺强杀native |
| V-07 | AC-06稳态/兼容 | T-07固定设备/场景/动作，cold/warm、单/双视图与20次reset；原829双游戏/830适用门 | CPU帧时P50/P95、编译/调用/上传/目标计数与内存/关闭实测；每实例编译一次、warm不重建pipeline；无GPU timestamp标不可用；不预写成绩 |
| V-08 | AC-07交接 | T-08从.at、SDK符号、bridge追到renderer/SD | 来源/profile/错误/限制与实现一致，I1仅首片、I2/I3/Shader/CAP pending；自报计数不代替行为证据 |

工作门命令：`cargo fmt --manifest-path experimental/autographics-auto-driver/Cargo.toml --check`、`cargo check --manifest-path experimental/autographics-auto-driver/Cargo.toml --locked --all-targets`；定向test的实际target名由T-04建立后写README。全量独立driver test/build和适用AGP兼容档由merge在最终review通过后的冻结提交执行；定向验收与反控不能后置。

仅experimental Rust，不触根cargo t/tv/tt/tb/tu/taa/docs_gen；如需写crates须先修订范围/门禁。tf仍主检出到期单实例，不在本树私跑重型全档。

## 7. 验收标准

| ID | owner_stage | 完成条件与验证 |
|---|---|---|
| AC-01 | work | G830/GSDK绑定真实交付/来源，V-01独立闭包/锁/A→B通过，零根生产代码/manifest/锁改动 |
| AC-02 | work | 真.at编译/初始化/持续调用，V-02参数/结果/状态/RC/预算成立，不依赖高层Nil桩，失败具名 |
| AC-03 | work | A-01两维度基础对象真由Auto输出创建/更新/销毁，V-03状态/像素/反控通过 |
| AC-04 | work | A-02完整冻结玩法计算在Auto，V-04原golden/事件/画面/音效请求闭合，识别脚本错误/Rust代跑 |
| AC-05 | work | A-03真实路由、独立状态/共享单tick/焦点/reset正确，V-05消费真实返回并对反控失败 |
| AC-06 | work | V-06输出准入/last-good/公平性/取消关闭重开、V-07兼容/20次reset通过；计量真实，最后owner释放、无迟到提交 |
| AC-07 | work | SD-01..03全文可审、来源/符号/命令与现况一致，I1分母/生产门保持，V-08通过 |
| AC-08 | review | 独立逐AC/SD/调用图与正反控复审，遗漏/未授权延后/workaround扫描，无open合同finding；merge门仍pending |
| AC-09 | merge | 最终review冻结提交的driver完整test/build和AGP适用真窗口/兼容/资源/反控门通过，合autographics-dev、沉淀/归档/guard清理，实际10/10 |

## 8. 执行步骤

T-01..10已全部完成（R1两finding经R2确认关闭；T-10完整门禁、源码合入、知识沉淀、实际归档与guard清理均完成）。唯一树与实现证据在§9及R2报告汇总。task路径以新workspace为根，已存在AGP/SDK符号见调查及r2交接；新路径均标“新”。

| 完成 | Task | owner_stage | 依赖 | 路径/产出与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 已满足的G830/GSDK；收到执行833的交接 | 读主检出r2与baseline附件；从764e87559建唯一树，从9576b4dad固定对象和所存根锁准备只读SDK源/build输入；重核而不追新来源；新research/source-lock.json、输入/禁入图与V-01入口 | AC-01 |
| [x] | T-02 | work | T-01 | 新最小外部fixture的manifest/锁/probe，核根create_vm_from_source、engine分片与ffi/convert；最多两接法；新research/sdk-driver-decision.md与V-02 whole-path；失败停止依赖任务 | AC-02 |
| [x] | T-03 | work | T-02 pass | 新manifest/独立锁/纯SDK载体准备校验/AGP消费；必要AGP纯数据出口；fresh/locked+A→B V-01，不复制修改语言实现 | AC-01 |
| [x] | T-04 | work | T-03 | 新src/bridge/实现实例、值/句柄、输出准入、有界pump/终态；建立定向test targets，V-02/06合法/反例 | AC-02,06 |
| [x] | T-05 | work | T-04 | 新samples/scene-control/*.at与host；接AGP core/renderer和830最终SurfaceRuntime，A-01/V-03状态像素正反控 | AC-03 |
| [x] | T-06 | work | T-05 | 新samples/breakout/*.at；参考AGP games/breakout/sim.rs与冻结manifest/testdata，不改Rust对照/golden；A-02/V-04 | AC-04 |
| [x] | T-07 | work | T-06 | 接830最终composition_input/内容出口，A-03/V-05；V-06/07失败/20次reset/兼容/测量与永久反例；R1重开：F-833-R1-01叠层/quad全资源归属，F-833-R1-02文字真实上传计量与反控；迭代只定向test/check | AC-05,06 |
| [x] | T-08 | work | T-07 | 新README/research handoff/SD草稿；实际符号/命令/测量/限制/债务归属，V-08与AC全映射；R1依赖重开：随两finding刷新计量/交接/SD证据后提交冻结→独立review | AC-07 |
| [x] | T-09 | review | T-08 | 同树独立核AC/SD/冻结SHA/调用图/反控及遗漏；R2 pass，两finding独立关闭；pass只表示审查通过，门禁仍pending | AC-08 |
| [x] | T-10 | merge | T-09 pass | 冻结提交统一driver全测/build+AGP适用兼容/真窗口/反控/资源门；合autographics-dev，Spec/module plans/ledger/index沉淀、归档，wt-guard clean后清树 | AC-09 |

## 9. 复审记录

### 2026-10-23 merge PLAN-833:r2：pass / cleaned / delivered 10 of10

- stage:merge | outcome:pass | completion_kind:delivered | plan_revision:2 | status:archived | current_step:10/10 | reviewed_commit:77302d8552f074ebedd53e757acfcf3db0bde230 | delivery_commit:540c1e70f0a4682af245191ffb36fa0d8a636665。
- landed/ledger/archived检查点已完成：master规范与归档提交8e41a0911，历史引用边界修正提交c51c852fe；只含docs/.autoos，无图形源码。actual archive gate重新核验所有本计划Markdown链接目标存在且已跟踪、module plans指向833真实归档、10个稳定task以及canonical/ledger读回均通过。下方9/10是清理前历史checkpoint，不代表当前状态。
- **cleaned**：移除前同树clean、所有提交均为autographics-dev祖先；使用Git Bash/MSYS执行fresh `wt-guard.sh`，报告clean、无reparse。`git worktree remove`成功；plan-833-dev按已验证的本地upstream autographics-dev用`branch -d`安全删除（master不承接源码，故“merged upstream but not HEAD”是预期提醒）；确认worktree/branch/空组D:/autostack/.wt/lang-833均不存在。[守护](../attachments/833/merge-r2/final-wt-guard.txt)与[清理读回](../attachments/833/merge-r2/cleaned.json.txt)。其他工作树/主检出预存docs WIP均保留。
- **batch_due_check**：[实际收据](../attachments/833/merge-r2/batch-due-check.json.txt)：833非5倍数，最近批量收据仅4.56h，covered commit后master无生产代码变化；未到期，不触发tf，不改批量收据。无剩余merge gate/cleanup blocker；下一责任none（本计划完成），生产迁移仍另行合同。


### 2026-10-23 merge PLAN-833:r2：landed / ledger_refreshed / archive checkpoint，cleanup pending

- stage:merge | plan_id:PLAN-833 | plan_revision:2 | completion_kind:delivered | reviewed_commit:`77302d8552f074ebedd53e757acfcf3db0bde230` | delivery_commit:`540c1e70f0a4682af245191ffb36fa0d8a636665`。受审实现/依赖/测试输入不变；后继仅规范投影与独立Windows构建profile说明。未改合同、未改SDK、未引入根生产代码。
- **prepared/gates**：[汇总](../attachments/833/merge-r2/gates-summary.json)；完整driver17/17、AGP162/162、locked release两workspace通过；16项AGP release探针全过：六surface6/17/18/15/23/14门，含真实OS CJK `爱彼此你好你好`、资源/计量/故障负控；829两游戏及同设备兼容全过。AGP A→B源码构建后29字段零差异。旧consumer围栏按当前受审commit显式设AGP_BASE_COMMIT；原764e87559→受审提交的新增driver/文档合法范围另以prepared写面守护核对，根生产diff=0。
- **当前源码隔离**：新包30实现hash一致，初始无target/rlib；producer SDK/AGP原目录隐藏时，冷build+完整17测试通过；scene原生30帧及source verify通过，finally恢复源目录。首轮外部窗口默认栈溢出：没有继承受审profile的32MB PE栈设置，保留原红；仅按原profile重链入口（`cargo rustc --locked --offline --bin autographics-auto-driver -- -C link-arg=/STACK:33554432`），PE从1048576B到33554432B后独立运行通过（以PE实测日志为准）。新增README/profile说明，非改源码或降低验收。release首次会话中断损坏sysinfo缓存E0786，仅清理对应release缓存后原代码构建通过；日志均保留。SDK既有warning146(dev)/149(release)，authored无新增warning。
- **native**：release焦点正控exit0，故意旁路负控exit1且命中x=-34/泄漏key=2；Breakout160frames/161ticks，6 requests=6实际Kira plays、零errors；外部同profile30帧scene通过。原V-07上传150720B仍有效，旧89280B作废；完整运行非GPU时间/驻留测量。
- **landed**：autographics-dev `764e87559cff→540c1e70f` fast-forward（非force的本地fetch ref更新；分支无工作树占用），tip=delivery；按833/829/830已裁定模式不把图形源码rebase/merge进master。无hash重写，无需range-diff映射。master知识落盘与归档在同次限定路径提交；其他会话docs WIP保留。
- **ledger_refreshed**：guarded offline projection，主store8080不可达；byte-exact备份[ledger-before](../attachments/833/merge-r2/ledger-before.json.txt)，v68→69，7条（designs4/tests1/reviews1/reports1）file/tags齐全；三个现有current ID复用，1130无关item原文与goals/architecture section字节不变，六section/schema/readback通过。[投影收据](../attachments/833/merge-r2/ledger-projection.json.txt)。历史条目缺timestamp被schema守护发现后从备份恢复，补足受影响条目字段再投影；未触其他条目。
- **archive checkpoint**：计划移至docs/plans/archive，status archived、当前9/10（仅T-10的guard清理余项未完成）；冻结SD四文件去provisional标签、保留829/830前缀，module overview/plans、GOAL-007/008与全局overview/INDEX同步。实际归档路径/状态/引用/计数门通过并提交后才允许清树。
- **production artifacts**：实验release driver/graphics-probe已在冻结源码构建并实跑；生产backend/daemon/vue bundle不消费该分支、零生产源码改动，无此次重启/重建对象。生产迁移/I1双游戏分母、Auto Squash/I2/I3/Shader/CAP继续pending。
- **next checkpoint**：只余actual-state归档提交与fresh wt-guard clean、worktree/branch/空组移除和最终10/10收据；不将待清理说成已完成。批量due判定在cleaned后记录；根t/tv/tt/tb/tu/taa/docs_gen不适用此次实验写面。


### 2026-10-22 merge T-10：prepared，门禁执行中

- receipt_key: PLAN-833:r2 | stage: merge | reviewed_commit:77302d8552f074ebedd53e757acfcf3db0bde230 | worktree:D:/autostack/.wt/lang-833/auto-lang | branch:plan-833-dev | destination:autographics-dev（master仅知识）。30实现hash与三锁核对一致、树clean；主检出其他WIP均为docs，保持原样。
- prepared: [冻结身份/写面/规范前缀](../attachments/833/merge-r2/prepared.json.txt)，R2独立pass且当前Spec前缀保持；T-10所有门通过前不落分支。driver完整test17/17已通过；release首轮会话中断，恢复遇sysinfo缓存损坏E0786，保留日志并仅清理该依赖release缓存重建；不作产品缺陷或豁免。
- canonical/ledger目标准备：SD四文件+模块索引/overview/goals，ledger当前v68、8080不可达，使用guarded offline projection（byte-exact备份与范围/schema/readback校验）；当前仅备份/规划，尚未发布。下一检查点:冻结完整driver/build、AGP full test/release/六surface/兼容/资源/反控/搬迁门。


### 2026-10-22 final review R2：pass，merge-owned门禁pending

- stage: review | review_scope: **final** | reviewer_identity: **当前Codex审核会话，独立于实施，沿用R1审核上下文**（具体模型/会话ID未暴露） | plan_id: PLAN-833 | plan_revision: 2 | outcome: **pass** | status: **reviewed** | current_step: **9/10**。
- worktree_path: `D:/autostack/.wt/lang-833/auto-lang` | branch: `plan-833-dev` | reviewed_commit: `77302d8552f074ebedd53e757acfcf3db0bde230` | implementation_commit: `6c47b79b13352eef44bf7e65084d2a8827a0b8a5` | base_commit: `764e87559cff016d9e0d6b2141e5f1f9903398c5`。进入/结论前同HEAD、clean；30实现/配置/输入hash全匹配。
- dependency_revisions: SDK `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`；SDK原锁 `5192d08d502a8f17e277ad8912f5884175659bef642b614e25b394243d1ff091`；AGP基面同base、原锁 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`；driver锁 `f01c231218b2f1534f971df128b9d188fa19648e5a4399dbe81ce37e5386063d`。依赖/测试配置/SDK/host/script/玩法/来源工具未变。
- findings: **F-833-R1-01/P1 CLOSED / F-833-R1-02/P2 CLOSED**。原同hash输入重放：双向foreign与replacement提前DeviceMismatch，帧计数/target保持；5glyph=3写/368B、相对无overlay多2写/304B。永久16状态归属矩阵、空glyph/clean/warm/共享单挂载精确计量与合法渲染均通过。未发现新修复回归或未授权延期/workaround。
- acceptance_results: **AC-01..08 pass / AC-09 pending**；T-09勾选。6图形+57指定兼容本轮独立重跑；11项不变验收及R1 native焦点/音效、SDK-only隔离/工具负控有界复用，不冒称全部重跑。计量省略负控复用已识别的work-r3实际mutation失败101并核当前恢复hash/正式oracle；审核未改实现源码制造变异。
- V-07本轮：compile26881us；shared P50/P95 2255/2758us，single2128/2859us；5targets/9109600B；upload150720B，warm额外pipeline0；工作集20reset前510513152B/后507691008B。旧89280B失效，GPU timestamp/residency不可测边界保持。两fmt、all-target check、diff/source检查通过，SDK既有146warning归属不变。
- spec_inputs / evidence: [逐finding/AC/SD与复用边界](../attachments/833/review-r2/review.md)、[冻结身份/输入和四份Spec增量hash](../attachments/833/review-r2/baseline.json)。SD-01..03通过，三份现行829/830前缀保留，spec-impact metadata不变；未发布canonical/ledger。
- pending_closeout_gates: **merge-owned完整driver test/build及适用AGP六surface/native/content/反控/resource/full compatibility、当前源码消费/重定位收口仍pending**。旧外部包不作为修复图形交付证据；根Cargo整档/docs_gen不适用，tf不在本树执行。review pass不是交付或免门合入。
- next: **T-10 / merge**在上述冻结SHA执行门禁，通过后仅落autographics-dev、master知识沉淀/ledger/index、归档及guard清理；若需改代码，冻结后复审受影响内容。本次未合入/归档/清树，保留实施树。

### 2026-10-22 R1 repair work：pass，待独立复审

- stage: work | plan_id: PLAN-833 | plan_revision: 2 | outcome: pass | task_ids: T-07,T-08 | status: execution_done | current_step:8/10。
- worktree_path:`D:/autostack/.wt/lang-833/auto-lang` | branch:`plan-833-dev` | base_commit:`764e87559cff016d9e0d6b2141e5f1f9903398c5` | implementation_commit:`6c47b79b13352eef44bf7e65084d2a8827a0b8a5` | code_commit/HEAD:`77302d8552f074ebedd53e757acfcf3db0bde230`（后者仅刷新30份hash/修复身份）。SDK/三锁未变，树clean。
- 两finding原始输入在受审b136129fe先复现；修复后同脚本通过拒绝/计量期望。永久设备矩阵16路径状态、精确上传/共享/空glyph/warm回归，6GPU+57兼容通过；省略文字计量反控101。定向check、fmt、来源验证通过。最新V-07上传150720B，原89280B无效；详[当前readiness与逐finding闭合](../attachments/833/work-r3/readiness.md)。11项不受影响非GPU验收和原native/SDK隔离证据有界复用，未冒充本轮全重跑。
- blockers/open in-contract findings/required work gaps: none。README/交接及SD-01/03草稿已刷新；未发布canonical Specs/ledger。最终复审失败计数1，本轮是首次repair而非final review。
- next_responsible_role: independent review，同树冻结提交。T-09/10及所有merge-owned完整门禁pending；未合入/归档/清树，实施停止写入待复审。


### 2026-10-22 final review R1：needs_fix，重开T-07/08

- stage: review | review_scope: **final** | reviewer_identity: **当前Codex独立审核会话**（未继承实施会话；具体模型/会话ID未暴露，不杜撰） | plan_id: PLAN-833 | plan_revision: 2 | outcome: **needs_fix** | status: **executing** | current_step: **6/10**。
- worktree_path: `D:/autostack/.wt/lang-833/auto-lang` | branch: `plan-833-dev` | reviewed_commit: `b136129fed9a018f07ea49445868066449eaf5e9` | base_commit: `764e87559cff016d9e0d6b2141e5f1f9903398c5`。进入/结论前同HEAD且clean；30实现/配置/输入hash一致，678份SDK未投影源文件与固定Git对象逐字节相同。
- dependency_revisions: SDK `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`；SDK原锁 `5192d08d502a8f17e277ad8912f5884175659bef642b614e25b394243d1ff091`；AGP基面同base、原锁 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`；driver锁 `f01c231218b2f1534f971df128b9d188fa19648e5a4399dbe81ce37e5386063d`。
- findings: **F-833-R1-01/P1**：`SurfaceContent::device_ordinal()`在quad/text不同时返回text ordinal，foreign quad+local text被初次准入及逐帧检查错误放行，真实GPU路径可panic。**F-833-R1-02/P2**：新增文字camera/glyph写入没有计入content/runtime上传数，5glyph实际额外304B但报表无增量；V-07和SD相关证据须更新。
- acceptance_results: **AC-01..05 pass / AC-06 fail / AC-07 partial / AC-08 fail / AC-09 pending**。实际重跑16项具名验收、57项指定兼容均通过；原10800tick/210contacts与容差、真实GPU反控、OS输入正反控、160帧Breakout/6请求=6实际音效均通过，不能抵消新增反例。两workspace fmt及driver all-target check通过；146 SDK既有warning归属保持。
- spec_inputs / evidence: [完整报告、逐AC/SD与重放](../attachments/833/review-r1/review.md)、[冻结身份/原规范和四份拟增量hash](../attachments/833/review-r1/baseline.json)。现行829/830规范前缀保留；SD-02边界口径通过，SD-01/03的归属/计量待实现闭合。spec-impact metadata保持，不发布canonical/ledger。
- 遗漏/延后/workaround扫描已完成；两项finding登记KNOWN-DEBT仅为追踪，不是延期或豁免。T-07、依赖证据T-08重开，保留T-01..06及历史work收据；不增plan_revision（合同语义未改）。
- pending_closeout_gates: 完整driver test/build与适用AGP六surface/native/content/resource/full compatibility等仍由merge持有；本轮未运行根Cargo整档/docs_gen/tf，未合入、归档或清树。
- next: **work在同树修复两finding、补永久定向反例并刷新T-08后提交冻结，再独立复审**。本次审核未修改实现，保留树；最终pass之后才执行merge-owned门禁。

### 2026-10-22 work r2：完成并移交独立复审

- stage: work | plan_id: PLAN-833 | plan_revision: 2 | outcome: **pass** | status: **execution_done** | current_step: **8/10**。
- worktree_path: `D:/autostack/.wt/lang-833/auto-lang` | repository: auto-lang | branch: `plan-833-dev` | base_commit: `764e87559cff016d9e0d6b2141e5f1f9903398c5` | implementation_commit: `eb383933ea84798998387883d4163eb5d062843e` | code_commit/current_HEAD: `b136129fed9a018f07ea49445868066449eaf5e9`（后者仅追加交接/身份清单，30份实现/配置/输入文件hash一致）。
- dependency_revisions: SDK `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`；SDK保存根锁 `5192d08d502a8f17e277ad8912f5884175659bef642b614e25b394243d1ff091`；AGP冻结基面同base，原锁 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`；driver最终锁 `f01c231218b2f1534f971df128b9d188fa19648e5a4399dbe81ce37e5386063d`。
- task_ids: T-01..08；evidence: [逐AC/V/SD与关闭条件readiness](../attachments/833/work-r2/readiness.md)、[同树交接](../attachments/833/work-r2/handoff.md)、[代码/锁身份](../attachments/833/work-r2/code-identity.json)。
- 当前结果：提交上16项具名验收通过；原10,800 tick/210碰撞及原容差、真实消费端回放/速度碰撞反控、2D/3D像素与真实输入/焦点正反控、20次reset/owner、外设备text拒绝均通过；既有AGP定向兼容57/57。最终真实Breakout窗口160帧/每实例163固定tick，6次请求=6次Kira成功播放，零错误。
- V-01最终source包fresh/locked：无target/rlib输入，原SDK/AGP路径不可用期间4项SDK/数值测试、实际30帧搬迁窗口与运行后source/hash校验通过；目录已恢复，无junction/symlink。两workspace fmt、driver all-target check通过；新增代码无新warning，冻结SDK的146项预存warning已归属记录。
- readiness: work-owned AC-01..07已闭合；open in-contract findings / required work coverage gaps / unresolved decisions 均为空。SDK通用数学转换仅为具名上游债务候选，不替换本Plan验收；Parked策略测试为构造SDK outcome，async/IO不在profile。内部自检不代替最终独立复审。
- blockers: none | next_responsible_role: **独立review（T-09，同树冻结基线）**。实施写入暂停；保留树。T-09/10与AC-08/09未勾选。merge-owned完整driver test/build、适用AGP完整/真窗口/资源兼容门仍pending，未合入autographics-dev/master、未发布ledger、未归档/清树。
- 验证时序：只运行具名必要验收/反控与57项触面兼容；未运行根t/tv/tt/tb/tu/taa/docs_gen/tf。

### 2026-10-22 work r2：T-01/02来源与接法闭合

- T-01：固定图形/SDK完整SHA及两锁hash一致；唯一树/分支已核。SDK Git对象纯投影680文件、17本地包/29路径边；default实际metadata闭包479包，禁入包0，所有本地路径在载体内；AGP锁全部registry身份/checksum保留。错文件hash/缺文件/SDK锁/consumer锁4项反控均非零，恢复精确原字节。
- T-02：选(b)公开标量callbacks/查询，真实Auto-owned globals/list持续状态；非交换异质参数/源码改参/返回tag与栈、f32初速度位型、实际Runnable预算/关闭owner反控通过。未用高层Nil桩；compound-return方案不需要继续（最多两方案）。归一化在Auto用有限f32算术，未改SDK/原golden/容差。
- 10,800 tick与210碰撞明细对拍通过，另有真实AGP消费端回放/源码速度与碰撞反控。完整driver输出是protocol1的有界增量槽查询、整包快照发布与事件通道；具体公共符号/预算/输入门见树内README与research/sdk-driver-decision.md。
- 真实窗口调度曾暴露slice逐帧续跑/全量查询积压，已修为同一8ms轮内续片与Auto changed(slot)，原失败回执保留。当前160显示帧/每实例212固定tick、10请求=10实际音效播放、零错误；仍在补齐最终冻结/搬迁回执，未认领execution_done。
- T-03：无target外部源码包已生成；临时隐藏本树SDK/AGP固定消费目录进行B fresh+locked编译，finally恢复，验证进行中。没有合入/最终review或跑根Cargo全档。

### 2026-10-22 work r2：执行入口

- 用户本会话已授权实施833；沿用r2范围/固定来源，不再适用new会话“不启动实现”的历史限制。主检出预检无代码WIP；现存文档/附件改动保留。
- 唯一实施树 `D:/autostack/.wt/lang-833/auto-lang`，分支 `plan-833-dev`，base `764e87559cff016d9e0d6b2141e5f1f9903398c5`；已核实际top-level/branch。SDK只读源固定 `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`。
- T-01进行中：AGP锁与SDK保存锁hash均匹配；17包/stdlib从固定Git对象提取（680文件），投影保留workspace.dependencies、排除未消费UI patch。T-02外部fixture编译/调用探针进行中，尚不认领V-01/02通过。
- work-owned T-01..08；T-09独立review与T-10 merge门pending；本次仅work授权，完成后交review，不合入。

### 2026-10-22 new r2：前置核验与实施基线冻结

- stage: new；plan_id: PLAN-833；plan_revision: 2；status: drafting；current_step: 0/10；**outcome: pass；next: work（T-01）**。
- G830满足：830 archived delivered 10/10，R5及merge门绑定764e87559cff；tip相同/AGP子树无差异/祖先核验通过。六surface、829兼容、重定位与已归因输入注入重试证据齐全，原树/分支已退役。
- GSDK冻结9576b4dad5ce：17个本地路径包/29边，normal/build/dev及optional递归零旧包/UI/Man；源码与固定源无差异；SDK root锁与manifest实际字节已存，关键输入hash/每包tree绑定。公开分片/转换及高层Nil桩边界未变，不把T-02能力探针当来源门已验功能。
- r2仅更新来源/授权/建树入口、T-01与T-03来源消费说明及当前Spec事实；10任务、9AC、8V-case、3SD的功能/精度/平台/写面保持。[完整交接](../attachments/833/planning/handoff-r2.md)、[预检](../attachments/833/planning/preflight-r2.md)、[r1全文](../attachments/833/planning/plan-r1-before-freeze.md)保留。
- 本轮没有实现/测试/开窗或新建agent/树；不改830终态/现行Spec。由用户另开的实施agent收到执行任务后进入work，无须再次请求来源许可。

### 2026-10-22 new r1：调查与草案交接

- stage: new；plan_id: PLAN-833；plan_revision: 1；status: drafting；current_step: 0/10。
- 已完成源码级调查：纯SDK已交付，高层call为桩，底层公开分片/转换可评估，需补来源载体、回调/数值/状态/RC/预算whole-path及图形桥。[调查](../attachments/833/planning/sdk-interface-investigation.md)/[指纹](../attachments/833/planning/source-observation.json.txt)不冒称运行验证。
- outcome: **blocked（仅实施交接）**：G830未交付、基线未冻结，且用户明确暂不实施；调查/草案编制已交付。next: 830 owner完成最终验收合入；用户另行启动833后，work在T-01重核/冻结双来源。本轮不索取实施确认，不改830状态/findings。
- 新合同T-01..10、AC-01..09、SD-01..03均待实施。静态预检见[preflight](../attachments/833/planning/preflight.md)。未创建agent/聊天/分支/worktree，未运行Cargo/窗口/游戏。

## 10. 待澄清事项

1. **来源已冻结**：G830/GSDK均已满足，具体完整SHA、锁和输入见§2.1与baseline附件；T-01重核固定版本并准备消费，不重新等待830或整体迁移。
2. 已实证：T-02选择公开标量接法，状态/参数/返回、所用f32/容器RC与pump/预算已通过；通用SDK数学转换为具名后继候选，本次有限f32算法保持原数值分母。
3. 范围已明确：首个Auto游戏Breakout；Squash真实Auto/生产I2/I3另选后继；本次SDK/AutoVM host入口不自动证明新版ACE或Shader/CAP具备。
4. 当前无待澄清/交付阻塞：T-10已完成，计划archived10/10；生产迁移、Auto Squash、I2/I3与Shader/CAP边界保持。


## spec-sync 回写记录

SD-01新增auto-graphics/auto-source-driver.md；SD-02 integration-contract.md；SD-03 project.md/prototype-profile.md。新增模块overview，plans索引与GOAL-007/008/global overview回写，INDEX由spec-index.py从main-current规范快照+本次覆盖再生。ledger仅派生摘要/引用，canonical仍为docs/specs；全部源码只在autographics-dev。


归档实际状态门修正：首次检查发现历史Design45背景链接指向主检出既有未跟踪文档。本次保留其取材身份为文字说明，发布知识只引用已提交的auto-graphics规范；未将其他会话WIP纳入提交。重新核验归档链接、已跟踪证据和任务计数后再清理；此为文档closeout修正，不改受审实现。
