---
plan_id: PLAN-833
status: drafting
feature_name: 真实Auto源码驱动AutoGraphics的最小闭环
plan_revision: 1
author: [Codex]
created_at: 2026-10-22
updated_at: 2026-10-22
current_step: 0
total_steps: 10
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

**预规划草案，不启动实施，不冻结代码基线。** 用户授权调查现有SDK并制定下一计划，明确等830验收合入后再冻结实施来源。当前 `drafting / 0/10`。

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
| G830 | 830最终review pass、冻结merge-owned门通过、实际合入autographics-dev，Spec/交接对应源码；由830 owner收口 | pending；本Plan不吸收/后移其findings |
| GSDK | 明确已交付SDK SHA/公开符号、manifest/build/include输入、依赖/资源锁与无UI/Man闭包 | 814已有交付可调查；最终版本在T-01重核 |
| GSTART | 用户后续明确启动833，取得来源后在本Plan记录实施基线 | 未授权实施；本轮只起草 |

观察主HEAD `d89e1dff36c13839d20ad35ae9f76088ab7c05d7`，SDK最近变更 `9c7e44855c88b936835d57111c06e629e30de0bf`，图形分支为829落点 `ab56c32ec4cb28e01fbcca74e9861164ffad43f0`，830 R2受审候选 `dbaea29e122d2fa2a18d18ce3f09444583098efd`。**均不是833实施冻结版本**，不消费830工作树WIP。

拟定唯一实施树 `D:/autostack/.wt/lang-833/auto-lang`、分支 `plan-833-dev`，从满足G830的autographics-dev固定提交创建；work/internal check/final review共用同树，主检出维护Plan。审毕代码合回autographics-dev，master仅沉淀文档；生产迁移门保持。本轮不创建树/分支。

## 3. 技术栈

复用GSDK的 `auto-lang-core`、单源 `auto-val` 与公开VM接口，沿用G830的wgpu/Iced/音频版本和资产，不升级依赖。新driver自有锁，AGP原manifest/锁原则上不改；必要消费出口只含纯图形数据。

`VMConvertible`、Task/VirtualRAM的使用封装在桥内，错误保留编译/调用/协议准入/图形消费阶段；stdout只作诊断。新CLI（拟名 `auto-graphics-driver`）、tests/fixtures为未来新增，由T-02/03/04落实，草案不声称现已可运行。

## 4. 需求分析与背景调查

### 4.1 授权与依据

2026-10-22用户明确要求先调查SDK并为真实Auto图形闭环制定草案，“暂不实施，等830验收合入后再冻结实施基线”。未指定实施/审核人或模型，未授权新建聊天/并行agent、生产集成或改上游计划。

从[overview](../specs/overview.md)、[SDK Spec](../specs/language-sdk/project.md)、[图形project](../specs/auto-graphics/project.md)/[profile](../specs/auto-graphics/prototype-profile.md)/[交接](../specs/auto-graphics/integration-contract.md)、[Design45](../design/45-autographics-wgpu-engine-and-game-prototypes.md)取材。[接口/缺口调查](attachments/833/planning/sdk-interface-investigation.md)与[逐文件指纹](attachments/833/planning/source-observation.json.txt)是源码级观察，未运行实现验证。设计章中旧“829 reviewed/830 drafting”仅历史快照，当前状态以Plan/收据为准，原生产迁移门仍有效。

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

**仅在GSTART后执行。** T-01先读取主检出本Plan当前修订，核G830/GSDK，记录实际树、基线/锁/输入hash、canonical Spec及开放依赖。等效路径修正记来源；若需改SDK/语言语义、精度或迁移写面，先提出具体缺口和重规划，不自行扩大。

T-01同时准备可显式读取的固定SDK源（只读归档/已发布包，含当次锁和build输入），供T-02的最小外部fixture使用。T-02先建立该fixture必需的临时独立manifest/锁，不依赖T-03尚未实现的可搬动载体工具；其路径和hash必须明确，禁止隐式主检出回退。T-03再将已选接法整理成正式workspace/受管载体并完成A→B验证。

T-02最多比较两种公开接法：(a)带类型的Auto状态/输出经公开VM值转换传递和保留；(b)同一Auto实例以有限标量回调/查询导出等价状态/命令，计算仍在Auto。不得用stdout、另造解释器或Rust玩法替代。

新增 `research/sdk-driver-decision.md`：记录真实.at、解析/编译/初始化/回调/结果消费符号、参数顺序、类型/状态owner/RC、失败/退出、执行预算和cold/warm调用开销。探针覆盖非交换异质实参、两次状态更新、真实f32路径/对象输出与合法源码变更，并有失败对照。复合返回若不可靠可选标量封送，但行为、玩法和数值分母不变。

两接法都不能满足公开调用、有状态驱动、原数值要求和有界退出时：T-02记SDK owner缺口，T-03之后停止依赖实现，保持未完成并回new；不改VM、砍验收或把未解问题写成已支持。

### 5.2 显式来源与写面

新增workspace内的manifest/锁、脚本、bridge、CLI、tests/research/source-lock/README；AGP按必要增加无语言类型的消费出口。**禁止修改crates/**、根Cargo/锁、生产UI/Man/CLI、其他仓与上游在途树。

T-03从GSDK已交付源生成只读纯SDK镜像，保留相对布局、workspace继承/patch、根锁、build/include输入；记录文件hash和投影规则，复用814无UI镜像模式。不假称当前已有通用exporter：局部准备/校验工具由T-03在新workspace实现，不重写792/815/831生产发行设施。若owner已有等价可搬SDK包，可用固定版替代，提供同等证据。

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

均为provisional。830未合入，不提前覆盖其Spec；SDK不修改，本Plan不把高层call写成已修。模块plans.md与ledger/index为merge常规关联簿记。

## 6. 测试设计

**以下全部是未来验证，未运行。** V-01..08 owner_stage=work，fixtures由对应task建立。最终review由独立上下文在同一冻结树核验，实施自检不代替最终review；未指定人/模型或建agent。

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

未来命令：`cargo fmt --manifest-path experimental/autographics-auto-driver/Cargo.toml --check`、`cargo check --manifest-path experimental/autographics-auto-driver/Cargo.toml --locked --all-targets`；定向test的实际target名由T-04建立后写README。全量独立driver test/build和适用AGP兼容档由merge在最终review通过后的冻结提交执行；定向验收与反控不能后置。

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

全部未实施，GSTART与G830/GSDK满足才进入。task路径以新workspace为根，已存在AGP/SDK符号见调查；新路径均标“新”。

| 完成 | Task | owner_stage | 依赖 | 路径/产出与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | G830/GSDK/GSTART | 读主检出Plan与SDK/图形交接、830终态；固定双来源/唯一树及供probe显式消费的只读SDK源/锁/build输入；新research/source-lock.json、输入/禁入图与V-01入口 | AC-01 |
| [ ] | T-02 | work | T-01 | 新最小外部fixture的manifest/锁/probe，核根create_vm_from_source、engine分片与ffi/convert；最多两接法；新research/sdk-driver-decision.md与V-02 whole-path；失败停止依赖任务 | AC-02 |
| [ ] | T-03 | work | T-02 pass | 新manifest/独立锁/纯SDK载体准备校验/AGP消费；必要AGP纯数据出口；fresh/locked+A→B V-01，不复制修改语言实现 | AC-01 |
| [ ] | T-04 | work | T-03 | 新src/bridge/实现实例、值/句柄、输出准入、有界pump/终态；建立定向test targets，V-02/06合法/反例 | AC-02,06 |
| [ ] | T-05 | work | T-04 | 新samples/scene-control/*.at与host；接AGP core/renderer和830最终SurfaceRuntime，A-01/V-03状态像素正反控 | AC-03 |
| [ ] | T-06 | work | T-05 | 新samples/breakout/*.at；参考AGP games/breakout/sim.rs与冻结manifest/testdata，不改Rust对照/golden；A-02/V-04 | AC-04 |
| [ ] | T-07 | work | T-06 | 接830最终composition_input/内容出口，A-03/V-05；V-06/07失败/20次reset/兼容/测量与永久反例；迭代只定向test/check | AC-05,06 |
| [ ] | T-08 | work | T-07 | 新README/research handoff/SD草稿；实际符号/命令/测量/限制/债务归属，V-08与AC全映射，自检提交冻结→独立review | AC-07 |
| [ ] | T-09 | review | T-08 | 同树独立核AC/SD/冻结SHA/调用图/反控及遗漏；pass只表示审查通过，门禁仍pending | AC-08 |
| [ ] | T-10 | merge | T-09 pass | 冻结提交统一driver全测/build+AGP适用兼容/真窗口/反控/资源门；合autographics-dev，Spec/module plans/ledger/index沉淀、归档，wt-guard clean后清树 | AC-09 |

## 9. 复审记录

### 2026-10-22 new r1：调查与草案交接

- stage: new；plan_id: PLAN-833；plan_revision: 1；status: drafting；current_step: 0/10。
- 已完成源码级调查：纯SDK已交付，高层call为桩，底层公开分片/转换可评估，需补来源载体、回调/数值/状态/RC/预算whole-path及图形桥。[调查](attachments/833/planning/sdk-interface-investigation.md)/[指纹](attachments/833/planning/source-observation.json.txt)不冒称运行验证。
- outcome: **blocked（仅实施交接）**：G830未交付、基线未冻结，且用户明确暂不实施；调查/草案编制已交付。next: 830 owner完成最终验收合入；用户另行启动833后，work在T-01重核/冻结双来源。本轮不索取实施确认，不改830状态/findings。
- 新合同T-01..10、AC-01..09、SD-01..03均待实施。静态预检见[preflight](attachments/833/planning/preflight.md)。未创建agent/聊天/分支/worktree，未运行Cargo/窗口/游戏。

## 10. 待澄清事项

1. 来源待交付：830最终SHA与SDK选用SHA/锁/输入由GSTART后的T-01冻结；候选修复不提前成为稳定来源。
2. 实现待实证：T-02负责状态/参数/返回与f32/RC、pump/资源预算的有界决定，形成明确可通过/拒绝结果后才做依赖任务；两接法失败具名回SDK owner，不默认已支持。
3. 范围已明确：首个Auto游戏Breakout；Squash真实Auto/生产I2/I3另选后继；本次SDK/AutoVM host入口不自动证明新版ACE或Shader/CAP具备。
4. 没有需要用户立即补充的决定。调查/草案已授权，实施未授权；不以时间经过或草案写完推定放行。
