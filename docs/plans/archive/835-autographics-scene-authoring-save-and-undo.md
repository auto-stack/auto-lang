---
plan_id: PLAN-835
completion_kind: delivered
status: archived
feature_name: AutoGraphics场景创作保存与撤销原型
plan_revision: 1
author: [Codex]
created_at: 2026-10-23
updated_at: 2026-10-23
current_step: 10
total_steps: 10
depends_on: [PLAN-834]
supersedes_spec_components:
  - docs/specs/auto-graphics/auto-source-driver.md
  - docs/specs/auto-graphics/integration-contract.md
  - docs/specs/auto-graphics/prototype-profile.md
  - docs/specs/auto-graphics/project.md
new_spec_components:
  - docs/specs/auto-graphics/scene-document.md
  - docs/specs/auto-graphics/authoring-session.md
touched_goals: [GOAL-007, GOAL-008]
affects: [auto-graphics, authoring-experimental]
---

# [PLAN-835] AutoGraphics 场景创作：保存与撤销原型

## 0. 变更摘要

**实施及final R2、完整冻结合入门已通过，已交付独立实验。** 834已归档交付，835固定其交付基线实施；冻结SHA与实际验收见§9及work-r1收据。T-09 final R2及T-10完整冻结门已通过，源码已合autographics-dev；清理与批量回归收据见末尾。

双游戏之后，下一步把既有Auto/图形能力变成一个可操作的创作闭环：打开场景→列表选择对象→修改属性/创建删除→真实Auto预览→保存→撤销/重做→重开一致。它是**AutoForge前置的独立场景创作实验**，不是生产Forge产品/仓库、生产AutoUI或RQHost接线，不抵扣I2/I3。

实现限制在现有experimental driver/AGP，首批只编辑受限2D/3D对象场景；不增加第三款游戏、通用引擎能力或编辑运行中游戏玩法。保留原Breakout/Squash的Auto执行、协议、golden、音画和资源/调度行为。

## 1. 目标

1. 建立与VM/GPU无关、可校验/序列化的场景文档，区分持久对象身份、文档修订与运行时session/代际。
2. 单窗口真实Iced对象列表/属性面板，选择后修改正确对象，创建/删除、撤销/重做和错误恢复都改变真实Auto驱动预览。
3. 保存明确版本的完整文档；失败保旧文件/当前文档，成功后重开还原同一对象/属性/资源/画面，不能只保存截图或临时GPU ID。
4. 正确处理dirty、历史分支、失败命令、迟到UI/预览结果、打开坏文件和未保存修改；同一文件只有本会话的有界保存操作，不以全局最好努力锁绕过冲突。
5. 给未来生产AutoUI/Forge提供可消费的文档/编辑/保存合同与实测；当前仅独立stock Iced/AutoGraphics，不认领生产里程碑。

profile：Windows x64、单原生窗口、键鼠；128个对象总额，64步撤销历史及16MiB历史保留字节上限，单文件1MiB。2D为circle/quad，3D为已有cylinder/player/mob固定资源，两个固定内容空间；沿用834的SDK/renderer/Iced版本。只做平面对象表，不编辑任意UI/VTree/Portal层级。

非目标：生产AutoUI/迁移/RQHost、正式Forge仓、AI agent自动改码/源映射、Auto源码生成或热重载、viewport射线拾取/gizmo、层级/reparent、自由材质/资源导入、实时编辑Breakout/Squash玩法、时间轴/骨骼、多窗口/多人协作/云盘/全局文件事务、Shader/CAP/ACE扩面。撤销历史是会话内能力，文件保存场景内容，不承诺重开后恢复历史栈。

## 2. 架构方案

```text
场景文件 → 完整校验 → SceneDocument（稳定ID、schema、内容）
                           ↓
真实Iced列表/属性 → EditCommand(expected revision) → 暂存候选文档
                           ↓
       现有Session + 新authoring预览profile（真实Auto）
                           ↓
           受检输出/资源准备 → 提交文档、历史与实际预览
                           ↓
        有界save snapshot → 同目录准备/校验/替换 → 读回重开
```

在 `experimental/autographics-auto-driver/` 新增 `src/authoring/`、专用bin（拟名 `autographics-authoring`）、Auto小场景和测试。复用该workspace的SDK载体/锁/打包机制和AGP渲染；不建新仓，不复制语言实现。文档/命令模型不包含Iced控件、VM Value/AST、wgpu对象或运行时地址；这些只留adapter/host。

### 2.1 前置、来源与唯一树

| 门 | 成立条件 | 当前状态与责任 |
|---|---|---|
| G834 | 834最终review通过、冻结merge-owned门完成、实际合autographics-dev、Spec/交接与版本对应 | 已满足：834 archived10/10，正式交付后创建835唯一树 |
| GSOURCE | G834后的driver/AGP/SDK完整SHA、锁/源锁/build输入/资源及authoring所需公开符号重核冻结 | 已满足：交付基面19adeebf1与SDK9576b4d；216hash现源码身份/680SDK文件见收据 |
| GSTART | 用户后续启动835；本轮仅提前规划 | 已满足：用户明确启动并要求完成全部work；未创建agent/额外聊天 |

观察代码为834受审提交 `ebeab12d2888b9c9fb7e00e3c24479fb3c028011`，原集成分支为833交付 `540c1e70f0a4682af245191ffb36fa0d8a636665`。SDK调查来源仍为 `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`。均为来源定位，**不是835实施冻结版本**；[观察清单](../attachments/835/planning/source-observation.json.txt)保留不可变Git对象和文档指纹。

唯一未来树 `D:/autostack/.wt/lang-835/auto-lang`、分支 `plan-835-dev`，从通过G834的autographics-dev完整SHA明确创建；不可用默认master基面捷径。work/internal check/final review顺序共用同树，主检出维护Plan。启动时读取主检出当前合同/附件，旧基面未必包含本Plan。审毕实验代码仍合autographics-dev，master仅沉淀文档，生产迁移门保持。

## 3. 技术栈

复用 `Session/Scalar/Progress` 的公开SDK CPU-slice执行、AGP Scene/CompositionGraph、SurfaceRuntime/IcedSurface及834原模型资源/mesh工具。现有Profile1/2不改义；创作预览使用另一个显式有界profile，不把Squash游戏slot/身份规则偷改成通用编辑协议。

序列化为UTF-8 JSON场景文件（拟扩展名 `.agscene`、schema身份 `autographics.scene/1`），属于实验文档交换，不是新Auto语言语法。使用已有serde/serde_json等依赖；不升级后端。已有SDK `state_file::atomic_write`可调查复用，但必须由T-02证明满足本次保存合同；`with_lock`超时后降级仍执行闭包的行为**不能**用来声称独占保存已成立。不修SDK，不新造通用文件管理系统。

## 4. 需求分析与背景调查

### 4.1 授权与依据

2026-10-23用户说明834实施完成、正在merge，要求提前规划下一计划。授权仅调查/草案；不合834、不改其代码/规范、不提前执行835，不创建agent/聊天或指定模型/审核人。

从[overview](../../specs/overview.md)、[图形project](../../specs/auto-graphics/project.md)/[profile](../../specs/auto-graphics/prototype-profile.md)/[交接](../../specs/auto-graphics/integration-contract.md)/[Auto入口](../../specs/auto-graphics/auto-source-driver.md)、[组合](../../specs/auto-graphics/surface-composition.md)/[输入](../../specs/auto-graphics/nested-input.md)、Design45（主检出既有未跟踪背景材料，不纳入835交付）、[v0.6创作目标](../../design/strategy/autographics-forge-v06.md)取材。既有目标要求可选择/修改/实时预览/保存/撤销/重开，不要求先做完整编辑器。

[834合同](834-autographics-auto-squash-and-dual-game-sdk.md)及[独立R1](../attachments/834/review-r1/review.md)说明两款Auto游戏已过代码/验收审查，完整门禁/落分支仍pending。候选源码仅只读定位，不替代G834。

### 4.2 为什么先做独立创作实验

| 现状 | 对下一步的约束 |
|---|---|
| 已有真实Auto场景/两款游戏、同设备UI/2D/3D组合与有界语言桥 | 可以转向真实创作闭环，不继续增加游戏数量 |
| 无canonical场景文件、编辑命令/历史/dirty/保存重开合同 | 本Plan补这一局部实验能力，不将Runtime snapshot直接当持久文档 |
| Scene/ObjectId含session/代际，game Packet有特定slot/事件限制 | 持久对象ID与运行时ID必须分开，创作另设显式profile |
| 原Scene::submit逐条处理，CompositionGraph批次有其事务 | 新编辑事务必须自己暂存/验证/发布，不能据接口名宣称文档/预览已原子一致 |
| Session可真实调用标量函数，Iced SurfaceUi公开view/update | 可增加受限Auto预览与真实属性控件，仍需T-02证明完整数据链 |
| SDK atomic_write存在，但with_lock会降级执行 | 保存准入/结果/所有权需实证，不能套旧锁返回值当无冲突保证 |
| 816仍在途，831/832是带启动锁的迁仓草案 | 本Plan不进入这些仓的准备/实施，不改生产UI以便利原型；生产I2/I3保持pending |

当前canonical尚无scene-document/authoring-session；新增Spec为provisional。834拟议增量也需等其正式交付，不能将候选SD提前复制成current。历史设计的旧Plan状态不覆盖现行收据。

## 5. 详细设计

### 5.1 执行入口与有限whole-path

G834/GSOURCE/GSTART满足后，T-01读取主检出当前修订，核唯一树/源/锁/已发布SD，复用已交付prepare/verify设施，不重选最新SDK。确认前序开放项为空、merge实际完成后再实施。

T-02在835树内先建立最小fixture与 `research/authoring-feasibility.md`（新）：一个2D对象和一个原3D模型，真实Iced控件修改→候选值经实际Auto Session→受检场景输出→GPU画面→保存→重新读取重建。只比较最多两种公开预览/提交接法，记录producer/consumer、状态owner、故障/恢复、必要符号与实际时延。

探针使用最小局部数据/临时文件和scratch adapter，不依赖T-03文档模型或T-05正式存储API已存在；只关闭方案可行性，不冒认正式任务完成。它同时验证原始文件替换和失败出口，确定固定schema字段、编解码/原子保存复用方式以及preview提交点。现有SDK锁降级路径不得参与写盘放行；若原helper不满足，只能局部包装/owned staging或本模块有界实现，不改SDK或扩大生产写面。

Auto调用/资源准备/文件替换不能满足后文合同，或需要接触生产UI/编译器时，记录确切owner/缺口并回new，相关后续任务不开始；不能降为只改Rust画面、仅写截图/缓存或砍保存/撤销。

### 5.2 文档、身份与首版编辑面

文档为固定schema/version、document ID、有序对象表及固定资源清单引用；对象有独立稳定ID、名称、空间类型/种类及受检属性。不是保存任意Auto代码、GPU地址、VM堆引用、当前游戏分数/随机状态或用户编辑器布局。

| 支持对象 | 可编辑字段 | 首版边界 |
|---|---|---|
| 2D circle/quad | 名称、translation xy、正scale xy、RGBA、可见性 | 固定2D内容空间，不做任意路径/纹理导入 |
| 3D cylinder/player/mob | 名称、translation xyz、yaw/pitch、正scale xyz、可见性 | 固定已有资源key与原模型材质，不新增自由材质/骨骼 |

总对象≤128、编码后≤1MiB；名称≤128 Unicode字符；数值有限、绝对值≤100000，scale>0、RGBA在0..1。超限是具名资源/支持面错误，不静默截断；未知schema/字段、重复ID/键、错误类型/资源、坏数字须在使用前拒绝。资源key为固定登记身份，不能携任意文件路径/VM/GPU引用。

首版**从对象列表选择**，预览高亮同一对象，属性控件绑定该稳定ID；无需viewport拾取或gizmo。支持创建/删除对象，先不做parent/reparent或编辑Portal拓扑；两个固定2D/3D预览与已有UI组合能力保持。示例至少两种空间各两个对象，便于证明修改A不会影响同名B。

文档稳定ID持久化；运行时Session/Scene/Surface/ObjectId由当前打开实例分配。删除后undo可恢复相同文档ID，但不得复活过期runtime代际/捕获；重新打开即新document epoch，旧消息/输出不能修改新会话。

### 5.3 命令、历史、dirty与发布

| 规则 | 数据流/owner与关闭条件 |
|---|---|
| 编辑消息 | 真实widget→命令(document epoch、expected revision、目标ID、字段/动作)；先校验目标/类型/范围/当前修订，拒绝迟到A字段写到新选中的B |
| 候选与预览 | 暂存文档→新authoring profile→真实Auto输出/资源准备→接受后同时发布文档、预览和历史；失败保持last-good文档/画面/历史/selection，不能仅回退输入框 |
| Auto语义 | 新Auto小场景从完整受检配置产生实际2D/3D输出，沿现有公开Session/renderer消费；Rust负责文档命令/封送/呈现，不在Auto失败时改走硬编码draw。候选配置的VM副作用必须隔离或用完整配置可验证恢复，下一次合法编辑实际恢复 |
| 修订 | 成功变更递增当前会话修订；noop/拒绝不增，不创建历史；undo/redo也检查epoch/revision并产生新修订，不能借旧revision绕过旧消息准入 |
| 历史 | 首版一次明确应用属性/创建/删除=一步；输入草稿/无效数值不进历史，取消恢复原值。undo恢复before，redo恢复after；undo后新改动清redo分支；重开后历史清空 |
| 历史预算 | 最多64步且最多16MiB保留字节；接受新命令后可按顺序淘汰最老历史直到预算满足，撤销到头按钮不可用；单次候选本身超预算则拒绝，不砍成半步，不误改dirty保存点 |
| dirty | 当前规范化内容hash与最后确认保存内容hash比较，不用“revision相同”或history游标代替。undo回已保存内容为clean；另存后的路径/保存点只在确认成功后切换 |
| 输入/调度 | 文本控件焦点不向预览投递游戏按键；失焦/选中对象删除清草稿/捕获。一个文档只发布一个编辑事务，pending操作可见且不可重入；复用有界Auto pump，关闭后丢弃迟到结果 |

预览是受限静态创作场景，未保存游戏运行状态；不在本轮给Breakout/Squash增加编辑接口。保存的是当前**已接受文档**；有未完成编辑时Save应等待该事务完成/拒绝或保持不可用，不能把尚未接受的输入草稿当已保存。

### 5.4 打开、保存与失败恢复

场景文件使用一个有限版本的规范化编码，只保存受支持的对象/属性/资源引用；不保存selection/focus/undo栈或GPU/VM身份。读回语义相等与实际画面均要验证，不要求不同JSON空白格式字节相同。

- Open先读/完整校验/准备预览，再替换当前文档；坏版本/损坏/缺资源/超限失败时当前文档、dirty/历史及可用预览保持。
- Open/Close遇dirty时提供Save/Discard/Cancel三种明确用户动作；Cancel零副作用。保存失败或结果未核实时不能假装已保存并关掉文档。
- 保存抓取(document epoch、revision、内容hash、目标路径、原文件预期hash)的不可变快照；只写用户选定当前文件或明确新路径。保存前已发生的外部字节修改/同文件占用要拒绝并保留当前编辑；单写者profile不宣称能排斥不合作程序在最后校验后的任意并发写入。
- 在目标同目录以唯一owned临时文件准备完整字节并校验/flush，再执行支持平台的替换；准备/写入/替换前失败保持原目标字节。不能先删旧文件再写，也不覆盖/清理无owner证明的临时文件或陌生目录。
- 替换为提交点：成功后读回并核内容/hash，才确认保存点。若替换已发生、后续读回失败，报告“结果待核验”，不假称旧文件未动或直接清dirty；再次读取/重试必须识别已提交的本次内容并恢复正常，不陷入永久冲突/误覆盖。
- 保存期间若当前内容已变，文件可确认保存较早快照，但dirty仍按当前内容与该保存hash计算；旧epoch完成不能清新文档dirty/换路径。可选同步实现同样必须证明不产生错误的迟到提交。
- 程序崩溃/断电后的全设备持久性、任意网络盘/多写者协作不是本profile承诺；本地同卷保存及受控进程失败点必须实测，残留只按owner证据处理。

### 5.5 固定验收流程与规范增量

| 场景 | 实际动作 | 核心oracle |
|---|---|---|
| A-01 创建与编辑 | 新文档→2D/3D各两对象→列表选A→真控件改属性→创建/删除 | 正确ID/文档值/Auto输出/最终GPU画面同变，B不动；共享预览多挂载不重复应用命令 |
| A-02 保存/历史 | 编辑→Save(S1)→undo(S0,dirty)→redo(S1,clean)→undo后新编辑S2 | 历史/dirty正确，redo失效，Save S2后独立重开恢复S2及原资源/画面；重开历史为空 |
| A-03 拒绝/恢复 | 坏属性/旧消息/Auto坏输出/资源失败、坏文件Open、各save失败点 | 文档/历史/画面/原文件按提交阶段保持；下一次合法编辑/保存/重开实际成功，不只看错误码 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/scene-document.md | 无场景文件→有限schema/持久ID/属性/资源/规范化编码/准入/保存重开 | 创作数据与运行时分离 | AC-02,05 |
| SD-02 | add | docs/specs/auto-graphics/authoring-session.md | 无编辑事务→命令/epoch修订/历史dirty/预览提交/失败恢复 | 真实创作闭环 | AC-03,04,05,06 |
| SD-03 | modify | docs/specs/auto-graphics/auto-source-driver.md | 双游戏桥→保旧profile、新有界authoring profile与真实配置/输出路径 | 复用SDK而非Rust假预览 | AC-03,06 |
| SD-04 | modify | docs/specs/auto-graphics/project.md, docs/specs/auto-graphics/prototype-profile.md, docs/specs/auto-graphics/integration-contract.md | 增独立创作实验/入口/实测/限制；I2/I3与生产Forge仍pending | 不把前置原型冒认生产里程碑 | AC-01,07 |

均为provisional；以834实际交付后的canonical为修改基面，保留其I1事实/双游戏/组合输入和生产迁移门。835只增加独立创作证据，不因保存/撤销通过把生产I3改done。module plans.md与ledger/index为merge常规簿记。

## 6. 测试设计

**V-01..09的work验收已执行，见§9及work-r1/readiness.md。** 下列owner_stage=work，fixtures/CLI/target已按任务建立并在实施README公开。独立final review在同一冻结树重新核AC/SD/实际路径；未分配人或模型，未创建agent。

| ID | AC / invariant | 实际入口/输入与启用task | 可观察oracle/反例与覆盖边界 |
|---|---|---|---|
| V-01 | AC-01来源/兼容 | T-01/07，G834来源/锁/资源→prepare/verify/package；旧两游戏/830组合 | fresh+locked独立包A→B，无producer/target；旧profile/golden/音画不退化，来源漂移/漏资产拒绝；不宣称生产发行 |
| V-02 | AC-02文档 | T-02/03，新decoder/validator→canonical model/encoder | 合法2D/3D往返语义相同；错schema/重复key/ID/未知字段/资源/NaN/超限拒绝，无runtime ID入文件；边界合法对照通过 |
| V-03 | AC-03编辑与Auto预览 | T-02/04，真实Iced消息→命令→Auto配置/输出→Scene→实际像素 | A字段/名称/创建删除正确，B不动；改Auto输出/丢配置/错目标变异令同一正式门红，不能只断言文档数字 |
| V-04 | AC-04历史/dirty | T-04/06，A-02经正式命令路径，noop/invalid/历史预算/删除undo/新编辑分支 | before/after精确恢复、保存点hash正确、undo回保存点clean、64步/字节边界真实；旧runtime代际不复活、错误历史不入栈 |
| V-05 | AC-05打开/保存 | T-05，A-02/A-03通过真实文件与新进程Open/预览 | 重开同稳定ID/资源/变换/颜色和实际画面；dirty取消不丢内容，坏Open保原文档；关闭旧进程后不借其内存恢复 |
| V-06 | AC-05保存失败/读回 | T-05准备/写/flush/替换前与提交后读回失败、外部预改字节、owned/foreign临时文件 | 提交前失败旧字节不变；提交后未知明确不假绿，重试/读回恢复可用；foreign文件不动；旧保存结果不能清新内容dirty |
| V-07 | AC-06发布/身份/寿命 | T-04/06，pending编辑/切选/旧rev、Auto拒绝/资源失败、Open后旧回调、20次open/edit/close | 文档/历史/preview一起提交或保last-good，下一合法操作实际恢复；session/target/VM/资源最后owner释放，合法pin区分；计数清零不能替代drop |
| V-08 | AC-06真实输入/计量 | T-06/07，真窗口列表/文本/属性/undo/save、DPI/resize，单/共享挂载 | 正确目标被实际消息消费，焦点不串；重复投递/错ID反控红；UI有界pump，冷暖帧时P50/P95/编译/资源上传/工作集真实；无法测的GPU时间注明 |
| V-09 | AC-07合同/交接 | T-07/08，从文件→document→命令→Auto→renderer→保存读回全链 | SD/CLI/符号/资源/限制与实际一致，原I1与生产I2/I3边界保持；只提供图片/JSON自报计数不能算通过 |

未来835树仅跑定向新document/editor/save/preview测试、实际输入/失败反控、必要driver check和两workspace fmt；完整driver/AGP适用兼容门由final review pass后在冻结提交统一执行，不能后置定向验收。沿用独立Windows构建的32MiB主栈/既定worker栈profile，不因导出包失去配置。

仅experimental Rust，不跑根cargo t/tv/tt/tb/tu/taa/docs_gen；若需生产crates变更先重规划。tf仍主检出到期单实例，不在此树私跑重型档。

## 7. 验收标准

| ID | owner_stage | 完成条件 |
|---|---|---|
| AC-01 | work | G834/GSOURCE固定，V-01来源/独立消费与两游戏/组合兼容成立，零生产写面 |
| AC-02 | work | schema/稳定ID/空间/属性/资源与预算明确，V-02合法往返与拒绝边界通过，运行时身份不持久化 |
| AC-03 | work | A-01真实控件修改/创建删除经实际Auto配置/输出改变正确对象和画面，V-03有区分力反控通过 |
| AC-04 | work | V-04撤销/重做/分支/noop/预算与dirty/保存点正确，删除undo不复活旧runtime事件 |
| AC-05 | work | A-02/03真实文件、新进程重开、失败/结果待核验/重试及未保存处理V-05/06成立，不丢旧文档/外部文件 |
| AC-06 | work | V-07/08发布一致性/恢复、迟到消息、焦点、有界pump/20次寿命与实际计量成立，保旧资源/设备/上传合同 |
| AC-07 | work | V-09/SD-01..04与实际全链一致，公开入口/重建步骤/限制完整，不冒认生产Forge/I2/I3 |
| AC-08 | review | 同树独立逐AC/SD/故障与合法控制、遗漏/未授权延后/workaround检查，无开放合同finding；merge-owned门pending明确 |
| AC-09 | merge | review冻结提交的完整driver/AGP必要门通过，合autographics-dev、规范/索引/ledger沉淀、归档及guard clean清理，实际10/10 |

## 8. 执行步骤

T-01～08已实施、定向验收并提交；T-09独立复审/T-10合入门禁仍pending。路径相对现有driver workspace，AGP/SDK只使用最后交付的公开接缝；新路径标“新”。

| 完成 | Task | owner_stage | 依赖 | 路径/产出与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | G834/GSOURCE/GSTART | 主检出读Plan/834终态/SDK与图形Spec，核唯一树/固定锁/资源和准备工具；新research/authoring-source.json，V-01来源图 | AC-01 |
| [x] | T-02 | work | T-01 | 新scratch fixture、最小authoring Auto程序与research/authoring-feasibility.md；Session→真Iced/2D/3D→文件读回whole-path，最多两接法，保存helper/失败边界实证 | AC-02,03,05 |
| [x] | T-03 | work | T-02 pass | 新src/authoring/document.rs及codec/validator、schema说明与fixtures；稳定ID/空间/属性/资源/预算，V-02；不把游戏Packet当场景文件 | AC-02 |
| [x] | T-04 | work | T-03 | 新src/authoring/session.rs/preview adapter与samples/authoring/*.at；正式命令/事务/历史/dirty、真实Auto输出；V-03/04/07，旧profile不改义 | AC-03,04,06 |
| [x] | T-05 | work | T-04 | 新src/authoring/storage.rs及真实文件故障fixtures；打开/保存/另存/所有权/读回/重试/未保存处理；V-05/06，新进程恢复 | AC-05 |
| [x] | T-06 | work | T-05 | 新src/authoring/host.rs/专用bin与manifest target，复用IcedSurface/SurfaceRuntime；真列表/属性/预览/保存/撤销，V-07/08焦点/迟到/20次寿命 | AC-06 |
| [x] | T-07 | work | T-06 | 更新本模块打包/来源身份工具的真实新输入，不改前序冻结收据；V-01/08/09独立A→B/旧两游戏/组合定向兼容、精确资源计量与永久反控 | AC-01,06,07 |
| [x] | T-08 | work | T-07 | README/Auto样例/研究交接与SD-01..04全文；真实CLI/符号/数据/状态/失败及生产边界，自检提交冻结→review | AC-07 |
| [x] | T-09 | review | T-08 | 独立逐AC/SD/冻结SHA/全链与反控，扫描遗漏/延期，pass只指审查、merge门pending | AC-08 |
| [x] | T-10 | merge | T-09 pass | 冻结提交完整driver/AGP适用门+创作保存/失败/真窗口/资源门；合autographics-dev、知识沉淀/归档、wt-guard clean后清树 | AC-09 |

## 9. 复审记录

### 2026-10-23 new r1：下一份场景创作计划

- stage: new；plan_id: PLAN-835；plan_revision: 1；status: drafting；current_step: 0/10。
- 静态核查canonical、834 reviewed候选/独立R1及生产迁移启动锁，选择不阻塞迁移的独立创作前置；代码观察指纹见[附件](../attachments/835/planning/source-observation.json.txt)。未把834 reviewed当delivered，未更新其状态或Spec。
- outcome: **blocked（仅实施交接）**：G834 merge尚未交付、GSOURCE未冻结，用户仅要求提前规划；草案编制完成。next: 834 owner完成冻结门/实际合入/知识交付，再核源冻结835，用户启动后从T-01进入。
- 新合同T-01..10、AC-01..09、V-01..09、SD-01..04全部未实施，[预检](../attachments/835/planning/preflight.md)。无Cargo/游戏/窗口/文件保存/性能测试，无新agent/聊天/树/分支，无生产代码改动。


### 2026-10-23 work：用户启动与唯一树建立

- stage: work | plan_id: PLAN-835 | plan_revision: 1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-835/auto-lang | branch: plan-835-dev | base_commit: 19adeebf1b1031554c71acabaa219731b3a28b96 | code_commit: 同base | task_ids: T-01进行中 | next_responsible_role: work。
- 用户明确授权实施，GSTART满足；834已在archive终态10/10，G834满足。此前仅规划/尚未交付文字保留为历史，不再代表当前启动状态。
- 主检出crates/test/experimental无代码WIP，既有文档改动保留。唯一树top-level及plan-835-dev分支已实核，从实际autographics-dev交付SHA创建，未使用master基面。
- 固定SDK载体prepare成功：680 files、17 local packages，无UI/Man/old package。尚未完成GSOURCE全部输入冻结与T-01来源图，T-02及后续未开始；不勾选任务或声称验收通过。未运行根Cargo整档，review/merge门仍pending。

### 2026-10-23 work：T-01来源冻结与缓存限制处理

- stage: work | plan_id: PLAN-835 | plan_revision: 1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-835/auto-lang | branch: plan-835-dev | base_commit: 19adeebf1b1031554c71acabaa219731b3a28b96 | code_commit: 9b3a5a202 | task_ids: T-01 | next_responsible_role: work。
- 显式清空单进程RUSTC_WRAPPER后verify_source.py exit0：SDK680文件/17本地包/29路径边/479 metadata闭包、forbidden0，834 authored source/assets180hash通过；未改变全局缓存配置、SDK、锁或生产代码。GSOURCE输入SHA256及固定SDK版本已记录并提交experimental/autographics-auto-driver/research/authoring-source.json。
- T-01完成；T-02 whole-path尚未完成，T-03..08未开始。workspace all-target cargo check已启动，结果待回收，不视为通过；work全部完成前不交接复审，不运行merge-owned完整档。
### 2026-10-23 work：T-02～05定向实现

- T-02真实widget→Auto→2D/原mob三mesh GPU变更/恢复与本地同目录替换读回1/1通过；提交895101a37。T-03严格schema/重复key/ID/类型/数值/资源与容量边界1/1通过，提交268d44dbf。
- T-04 profile3完整配置与有界slice pump、候选隔离、历史/dirty/epoch/修订/选择/预算/坏输出反控及下一合法操作恢复2/2通过，提交78cd8f17d。Auto列表push同名导出冲突由局部命名append_field消除，未改SDK。
- T-05真实文件准备/写/flush/replace前故障、提交后待核验恢复、外部字节修改/foreign锁与临时文件、较早快照/旧epoch2/2通过，提交3adb6800e。同步存储不产生异步结果切新文档。
- T-06进行中：新增独立host/bin，GPU/真实列表和属性焦点/失败保last-good/dirty取消/文件重开定向测试1/1通过。尚需真实原生输入/新进程/生命周期/打包及正式反控收口，未交接复审；T-07/08 pending。

### 2026-10-23 work r1：全部work完成，冻结交接8/10

- stage: work | plan_id: PLAN-835 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-835/auto-lang | branch: plan-835-dev | base_commit: 19adeebf1b1031554c71acabaa219731b3a28b96 | code_commit: e3ffdff1a04d26caad8b51396ad816953781b354 | dependency_revisions: SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9、原AGP锁、216hash源码/资产身份 | task_ids: T-01..08 | blockers: none | next_responsible_role: independent review。
- [逐AC/V/SD readiness](../attachments/835/work-r1/readiness.md)、[结构化交接](../attachments/835/work-r1/handoff.json)、[绑定来源清单](../attachments/835/work-r1/plan835-identity.json)、[原始证据hash](../attachments/835/work-r1/evidence-hashes.json)。AC-01..07/V-01..09 work pass；新定向15个唯一测试零fail/ignored，旧游戏golden/反控4及旧图形/实际OGG与所有权7通过。没有根cargo整档或docs_gen。
- 最终独立consumer来源/metadata/216hash验证通过，源码漂移/缺文件永久反控拒识；从无target冷构建并锁定离线验收，全部本地依赖在包内，producer env不可用。首包遗漏历史code-identity.json被门禁拒绝，已按身份清单驱动打包补齐；未修改冻结834收据。最终纯源码包consumer-final4无target且来源/反控全通过，消费者实际Rust/依赖/配置/输入与冻结源码一致。
- 真OS控件列表/属性/Apply/Save/undo/redo/resize正控phase9 exit0；错误目标反控同oracle exit1。原窗口退出后新进程重开文档、2D/3D像素hash相同、历史空/dirty false。实际DPI scale2.0、resize2000×1400、32MiB PE栈已实读；20轮VM/runtime-target/mesh最后owner+合法pin、共享挂载一次发布、upload漏计反控通过。完整测量/排除项见收据。
- 内部收口修复：草稿亦绑定epoch/revision/target；选择资源失败恢复原selection；Discard后坏Open恢复可用控件；相应永久回归与128对象全Auto上界/丢配置/错目标变异通过。遗漏/未授权延期/workaround扫描无开放合同finding、gap或新增债务候选。两workspace fmt/all-target/whitespace通过；固定SDK146条既有警告，无本次源码新warning。实施树tracked/untracked delivery clean，全部实现已提交。
- status execution_done / current_step8 of10。T-09/AC-08尚未执行；T-10/AC-09完整driver/AGP适用门、实际合autographics-dev、知识/ledger沉淀、归档和guard清理均merge-owned pending。保留唯一树并停止实施写入供独立review；未合入、未归档、未冒认生产I2/I3/Forge交付。

### 2026-10-23 review R1：needs_fix

- stage: review | review_scope: final | reviewer_identity: 同实施聊天，独立上下文有限，重新核冻结实现及额外反例 | plan_revision:1 | reviewed_commit:e3ffdff1a04d26caad8b51396ad816953781b354 | outcome: needs_fix。
- F-835-R1-01（AC-02/05/06、T-06）：合法持久ID u64::MAX已被Document::encode接受，但Host::new以max+1初始化分配器而返回identity exhausted，不能打开既有文档；同源Open发布后重设分配器也会在提交后报错。反例research/review835-identity.log，真实test exit101。需将现有文档生命周期与后续ID分配耗尽分离；修复新建Host及Open两个路径、保留创建拒绝及下一合法编辑/保存能力。其他15个定向检查正在重放，无需改变schema或缩小持久ID范围。
- 状态回executing，T-06重开，其余已完成任务保留；全档门仍merge-owned pending。next work，同一树修复后复审。

### 2026-10-23 review R2：final pass，9/10

- stage: review | review_scope: final | reviewer_identity/limitations: 同实施聊天Codex，独立上下文有限，重建代码/断言及独立反例，未创建agent或编造会话身份 | plan_revision:1 | reviewed_commit:0c29fa2a7cfb0dbc08a906542c511bd20f34ebed | outcome:pass | worktree_path:D:/autostack/.wt/lang-835/auto-lang | branch:plan-835-dev。
- [逐AC/SD审查](../attachments/835/review-r2/review.md)、[冻结Spec差异](../attachments/835/review-r2/spec-delta.diff)。R1 max-ID合法边界已复现并在构造/Open同源路径修复，永久host7/7及旧合法控制通过。此前15定向重放通过；无开放finding/gap/未授权延后；旧profile/SDK/资源/config未改的golden/native证据明确复用，merge再跑完整档/原生门。
- T-09/AC-08完成，status reviewed/current_step9。T-10/AC-09完整driver/AGP适用门及实际合入/知识/ledger/归档/guard清理均merge-owned pending，未宣称交付。

### 2026-10-23 merge r1：冻结门进行中，桌面输入前置blocked

- stage: merge | key: PLAN-835:r1 | reviewed_commit:0c29fa2a7cfb0dbc08a906542c511bd20f34ebed | prepared_delivery_commit:2a7b10250245332f46b7867dc140da1a4f9d0969（纯docs/ledger后继，Rust/依赖/配置未变）| outcome:blocked（真实前台输入环境）。
- 完整driver/AGP tests+build均通过，[checkpoint](../attachments/835/merge-r1/checkpoint.json)有实际计数；创作真OS正控phase9/错目标负控/新进程重开通过；AGP8项结构/资源/六surface探针通过。独立消费者host7与来源/漏资产反控通过。
- 原Iced合成正式门3次有界点击未观察到；诊断实测DPI192、SetForegroundWindow=False，未猜测锁屏。需恢复可接收真实鼠标的前台桌面，再重跑同一正控与counter/content、blank/transparent负控。已向用户请求此环境恢复；不修改判定/容差，不盲重试。
- canonical/goals/index及guarded ledger v71→72在唯一树准备且已提交，无关raw item字节保留；尚未向主检出发布、未合autographics-dev、未归档/清树。T-10/AC-09保持pending，status reviewed/current_step9。835后续落地会触发%5批量回归，届时按主检出单实例执行，不提前在树内跑tf。

## 10. 待澄清事项

1. 当前无开放work合同finding、必测缺口或未决用户决定；834已交付且835来源已冻结。831/832原启动锁保持，不以本Plan提前做它们的准备。
2. T-02负责真实Auto预览提交/恢复与本地文件替换/读回失败的有界可行性；遇SDK/生产写面缺口具名回owner，不删保存/历史/真实预览验收。
3. 选择方式、有限对象/字段、容量/历史预算、会话内撤销和生产非目标已明确；此原型不是完整Forge或UI语言编辑器，不自动解除I2/I3迁移门。
4. 用户已明确授权实施，work已完成；下一责任角色为独立review，review/merge-owned任务仍pending，不能以work完成代替审查或正式交付。

Merge前置：请恢复Windows可交互前台桌面；原Iced真实鼠标门未通过前禁止合入。当前无新增语义决定，唯一待外部恢复项为桌面输入环境。

### 2026-10-23 merge：已合入并沉淀，清理pending

- stage:merge | key:PLAN-835:r1 | outcome:in_progress | reviewed_commit:0c29fa2a7cfb0dbc08a906542c511bd20f34ebed | delivery_commit:2a7b10250245332f46b7867dc140da1a4f9d0969 | landed:autographics-dev FF | ledger_refreshed:v71→72 | archived:本文件提交后核验 | cleaned:pending。
- 完整driver58/AGP162 tests+build零fail/ignored；16个AGP正式探针全通过，Iced counter/content正控及blank/transparent反控通过，创作原生正控phase9、错目标负控、独立重开通过。桌面解锁恢复后原门通过；IME仅激活自身测试窗口即观察真实中文组合提交，未改代码/容差。独立consumer初次用了829默认历史基面导致collateral红，实际冷A/B构建/对拍成功；显式绑定当前冻结交付基面重跑全门绿，保留初次红归因。
- 交付仅autographics-dev源码；主检出仅知识/计划投影。canonical6文档+模块/goals/index和guarded ledger9条upsert已读回验证，1142无关raw item字节保持。主检出其余会话WIP未纳入。Spec/ledger纯文档后继经diff证明不改受审实现/依赖/配置。
- 生产AutoUI/RQHost/Forge I2/I3仍pending，根backend/daemon/web不消费本实验入口，无生产部署或重启。T-10待guard清理后计10/10；835%5==0触发的主检出单实例批量回归在清理后执行。

## spec-sync 回写记录

SD-01 scene-document、SD-02 authoring-session新增；SD-03 auto-source-driver及SD-04 project/prototype-profile/integration-contract沉淀，module overview/plans与GOAL-007/008/global overview回写，spec-index再生。派生ledger v71→72，6 designs/1 tests/1 reviews/1 reports，canonical仍为docs/specs；来源/审查/门禁/原始失败与恢复归因见835附件。

### 2026-10-23 merge收尾：delivered10/10，guard清理完成

- 最终delivery2a7b10250245332f46b7867dc140da1a4f9d0969已在autographics-dev；main知识/归档提交b294fefe3、实际归档检查a98950ab2。源/Spec/ledger读回通过，真实archive全部20个本地链接均存在且跟踪。
- 即时Git Bash wt-guard clean后移除唯一835树，已检查其全部提交在autographics-dev；分支设其为upstream再用无force的git branch -d删除，空组删除。9个本任务临时consumer逐一绝对路径校验+guard clean后删除，未触其他会话文件。见cleaned.json/final-guard.txt/consumer-guards.txt。
- T-10/AC-09完成，status archived/completion_kind delivered/current_step10 of10。无剩余835实施/复审/合入/知识/归档/清理任务；生产I2/I3不外推。批量回归%5 due在主检出单实例继续，属于归档后统一回归收据，不在计划树内运行。

### 2026-10-23 postlanding batch：known-reds-only，最终闭合

- /auto-plan:regress 主检出单实例：tf6282/6282；完整tt复跑5702/5703，唯一红为既有D-762-1图表Init helper早退；完整tb复跑5307/5307。初跑3个worker计时红定向及完整tt复跑均通过，未改代码/阈值；原始红/重放/归因全部保留。[批量回执](../attachments/835/batch-r1/receipt.md)。
- .last-batch-regression.json更新覆盖835，verdict known-reds-only/new_reds空；未将剩余已知红记绿，不擅自修改需另行裁定的生产helper。AAVM触发diff为空，不跑taa/t3。835所有实施/复审/冻结全门/合入/知识/ledger/归档/清理与逢5回归任务完成；最终archived/delivered10/10。其他会话WIP保持，生产I2/I3未启动。
