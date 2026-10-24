---
plan_id: PLAN-845
status: reviewed
feature_name: AutoGraphics行为绑定与可搬迁项目闭环
plan_revision: 2
author: [Codex]
created_at: 2026-10-24
updated_at: 2026-10-25
current_step: 9
total_steps: 10
depends_on: [PLAN-840]
supersedes_spec_components:
  - docs/specs/auto-graphics/scene-document.md
  - docs/specs/auto-graphics/scene-assets.md
  - docs/specs/auto-graphics/authoring-session.md
  - docs/specs/auto-graphics/scene-play-session.md
  - docs/specs/auto-graphics/behavior-api.md
  - docs/specs/auto-graphics/auto-source-driver.md
  - docs/specs/auto-graphics/project.md
  - docs/specs/auto-graphics/integration-contract.md
new_spec_components:
  - docs/specs/auto-graphics/behavior-project.md
touched_goals: [GOAL-007, GOAL-008]
affects: [auto-graphics, authoring-experimental, behavior-project]
---

# [PLAN-845] AutoGraphics 行为绑定与可搬迁项目闭环

## 0. 变更摘要

840让用户仅写单文件Auto行为，通过正式来源入口运行现有场景，但刻意不保存脚本/角色绑定，也不自动随场景交付源码。下一步让用户**绑定脚本和角色→保存→关闭→搬迁整个项目目录→新进程只打开场景、点击Run即可重现行为**。运行仍由840真实公共API/AutoVM驱动，无需再找原脚本或补--behavior参数。

这是独立实验driver内的有限项目持久化切片，保留已有受限GLB、几何/层级、API和Play语义；不增加游戏引擎行为面或完整包管理。采用scene/4内的单一可空行为绑定、内容寻址Auto源码blob及原场景入口的单提交点，整合现有事务、历史和存储恢复。

r1规划调查时840为executing r2 3/12，SDK扫描修复和精确重冻已记录，当时只读规划。r2实施时840已archived26/26；用户明确接受其披露的复审限制，G840按本页r2门表成立；实现沿最终正式来源开展。841–844已属于其他路线，中央分配本计划为845；不是跳过图形计划或提前占用多份后继。

## 1. 目标

1. 将一个符合840 behavior/1的用户源码快照及可空2D/3D actor稳定ID作为已接受场景内容绑定；绑定/更新/解绑/改角色可撤销、重做，dirty随实际内容计算。
2. 场景保存完整声明闭包：现有GLB与绑定源码；Save As/复制目录后不依赖原导入路径、cwd或开发仓，源码保持原始字节及身份。
3. 重开先校验完整闭包及来源/绑定，再一次替换Editor；Run消费已接受源码，沿840编译/真实Auto执行/全帧准入和Stop恢复，保留临时--behavior独立使用方式。
4. 保存前故障保旧入口；提交后验证失败保明确unverified收据与恢复流程；代码/GLB/entry冲突及晚到旧epoch结果不能把旧内容标成新内容已保存。
5. 沿用840两个不同逻辑示例，在同一个未重编译runner上，独立新进程/仓外搬迁实际Auto轨迹及最终GPU像素可验证。

非目标：多脚本节点组件/动态创建销毁、多文件use/include/源码包解析、脚本编辑器/热重载、可执行文件打包/安装/发布器、网络协作/自动GC、源码安全沙箱、SDK/AGP改造、生产AutoUI/RQHost/完整Forge/新默认设计包。完整组件接入、主题继承与跨端样式研究继续沿[Design47 §16](../design/47-autoui-state-design-and-feature-architecture.md#16-autographics-后续完整组件接入与默认设计体系2026-10-24)后续规划；不并入本Plan，也不改840范围。

## 2. 架构方案

```text
840用户文件 → 一次有界读取 → 同一来源准入/编译检查 → immutable CodePayload
  → 带stamp/绑定generation的编辑候选（scene/4 behavior + actor IDs）
  → 原编辑Auto preview/资源prepare → 一次接受 + history/dirty
  → SaveSnapshot：entry + 声明GLB + 当前声明源码
  → root lock → immutable GLB/code准备和回读 → 原单entry rename
  → entry+完整闭包回读 → Confirmed；否则VerificationPending
  → 新进程Open：全部校验/prepare后替换 → Run默认消费绑定字节
  → 840公共API/Play → 实际画面；Stop恢复编辑场景和绑定
```

L1：只写`experimental/autographics-auto-driver/`，局部扩展现有Document/Editor/Storage/Host及840已交付behavior模块。新模块拟`src/authoring/behavior_project.rs`，新测试/样例/工具见T表；不是引擎核心ABI、语言协议或跨仓架构改造。SDK/AGP/production crates/根manifest和两锁只读；需突破这些边界则回new，架构扩面按L2先设计。

| 门 | 成立条件与当前状态 | owner_stage |
|---|---|---|
| G840 | 接受840已披露同实现会话限制的最终复审收据；冻结完整门、正式autographics-dev落地、SDK定向来源归属/Spec/ledger/归档/guard清理与到期检查齐备；2026-10-24用户明确接受限制，已满足 | work T-01前 |
| GSOURCE | T-01已核最终840完整SHA8b62db5c、SDK932/两锁、真实API/SourceSnapshot::from_bytes/来源选择及当前Spec；已满足 | work T-01 |
| GSTART | 用户已明确启动845 work，并接受840复审限制；已满足 | work启动前 |

唯一未来树`D:/autostack/.wt/lang-845/auto-lang`，分支`plan-845-dev`，**显式基于最终840正式autographics-dev SHA**；不能使用默认从master建树的便捷命令。实施/internal/final review同树顺序；主检出只记录计划/知识。源码只落autographics-dev；master图形合入仍守原迁移门。禁止junction/symlink，清理前wt-guard。

## 3. 技术栈

复用最终840的Rust、stock Iced/wgpu、公开语言SDK/Parser/Session、UserBehaviorV1/profile7与原Builtin6；schema严格JSON/serde、原始SHA256、Arc有界payload、现有本地单写者锁/同卷rename和OwnedFile恢复机制。新增项目绑定不引入第二个parser、VM、运行时协议或存储引擎。

本轮候选driver为`e98e210c769019f153c6b9dc949cdb62e61ddb4e`，正式集成为839交付`a239653365a0c3451c85aa9921189fcf011bd8cf`；840 SDK候选`932ad4877e5968f1aa7227d9dcd7fb69b4c69a21`。这些均不代替T-01冻结。840正式behavior模块尚未交付，以下涉及它的调用是拟议消费点，不能当已存在的方法。

## 4. 需求分析与背景调查

### 4.1 授权与来源

2026-10-24用户“840正在实施中，请继续规划下一个计划”。授权只读调查及一份下一Plan草案；本轮不执行840/845、不创建agent/聊天、未指定模型/预算。前次“完整AutoUI/默认设计体系补充到后续规划、840不扩面”的约束继续适用；本计划消费840而不改写其合同。

已读[全局overview](../specs/overview.md)、[图形overview](../specs/auto-graphics/overview.md)、[场景文档](../specs/auto-graphics/scene-document.md)、[静态资源](../specs/auto-graphics/scene-assets.md)、[编辑会话](../specs/auto-graphics/authoring-session.md)、[运行会话](../specs/auto-graphics/scene-play-session.md)、[集成边界](../specs/auto-graphics/integration-contract.md)、[SDK](../specs/language-sdk/project.md)和[840 r2](840-autographics-public-behavior-api-and-user-scripts.md)。固定对象及文档hash见[来源清单](attachments/845/planning/source-observation.json.txt)。behavior-api.md待840沉淀，当前不编造其canonical内容。

### 4.2 已观察事实与缺口

| 源码触点（候选SHA） | 已观察事实 | 下一步 |
|---|---|---|
| document.rs::Document/decode/unique_json | 当前schema3仅几何/groups/assets；严格键与旧schema映射，1MiB/128节点 | 新scene/4显式behavior:null或单绑定，旧schema不改原文件 |
| assets.rs::Registry/regular_path | 有界原始SHA GLB Arc、实际ancestor/reparse校验 | 源码用独立payload及预算，复用路径原则，不冒充GLB或另造资源格式 |
| session.rs::Edit/Step/Pending/Editor | history存Document before/after，dirty基于hash，assets在Editor/pending持有 | 绑定也走候选事务，原history只存metadata不足以保证旧源码可恢复，须补真实owner闭包 |
| storage.rs::Snapshot/load_assets/prepare_closure/verify_locked | 单entry为提交点；GLB先装配，回读整个声明闭包后confirm；unverified持有资产快照 | 加源码闭包，不能只多写一文件后提前clean；Save As从内存快照重建新root |
| host.rs::start_play/dispatch/Panel | 真实Iced命令和840来源适配仍在演进，839绑定是临时actor选择 | 默认来源消费已接受绑定；原外部文件模式为明确临时覆盖，不偷偷持久化 |
| tests/user_behavior_feasibility.rs及research | 840 scratch验证过两段实际源码/GLB/Stop机制；正式profile7/UI尚未结束 | 等840交付后复用正式实现，scratch绿不关闭845验收 |

831当前executing r2 8/12，832仍drafting0/12；完整生产AutoUI/主题线暂未具备最终迁移交接。此处继续独立实验项目闭环，不利用迁移中的UI/SDK写面。未来状态变化由T-01重核，不以历史进度当实施事实。

## 5. 详细设计

### 5.1 下一入口与有限可行性

下一可执行任务是G840/GSTART成立后的T-01，随后T-02。T-01核正式代码/Spec/来源和旧测试分母，若840最终API与本草案假设冲突先修订，不直接在候选上实施。

T-02在唯一树建立新`research/behavior-project-feasibility.md`：以最小scene/4候选绑定一个原始源码payload，经840同一准入/编译与Editor候选、Save As、删除原导入文件、新进程Open/实际Run/GLB GPU/Stop，证明完整路径。另测源码安装前失败、entry提交后源码回读失败/Verify，以及Undo恢复旧payload。只比较现有闭包函数局部扩展或一层受限资源描述统一这两种实现；不预建通用包/事务框架。证明“绑定/Open不执行用户app_start/update”，仅共用静态准入/编译检查并释放检查VM；实际回调只在明确Run发生。若不能复用840正式字节准入而必须改SDK/复制compiler pipeline，则停止依赖任务回new。

### 5.2 scene/4：单一行为与稳定角色绑定

新Document使用`autographics.scene/4`，保留所有既有字段与约束，新增**必需**`behavior`：null，或`{api,source:{format,sha256,byte_length},actors:{two_d,three_d}}`。api精确为最终840的behavior/1版本身份；source format固定`auto-source`，sha为小写64hex、byte_length为正且≤256KiB。actors各为null或合法同维叶子的持久u64 ID，禁止group/不存在/跨维/缩窄映射；null明确无actor，不自动选第一对象。运行时ordinal仍由840当次Run生成，不写盘。

同名对象不共享绑定。绑定actor删除、含其祖先的子树删除或其他使引用失效的候选，均拒绝并提示先解绑/改角色；不静默清空或替换actor。可见性改变不使合法actor失效，父组/结构改动仍按原几何约束。绑定/角色与引用约束对UI、Undo/Redo、decode/Open、Run均适用。

v1/v2/v3各按原规则严格校验后迁移到v4且behavior:null，ID/order/局部值/GLB身份与原语义保留；Open只在内存迁移，不重写来源。Save显式写v4，旧reader不读v4。未知/缺字段/重复键、错API/摘要/字节长度、坏actor在消费前受控拒绝，不能丢binding后按无脚本场景打开。API/bridge随runner版本受检，用户项目不携带可覆盖transport的桥接文件。

项目root仍为scene入口的父目录。源码唯一物理布局是`scripts/<sha256>.at`，不持久化原绝对/相对导入path或自选依赖路径；原GLB仍为assets/<sha>.glb。只声明一个源码blob，无module/包/附加清单/路径搜索。源诊断显示实际项目blob路径；尚未保存的payload以明确内存来源/摘要标识，不能假造仍可读取的原路径。

### 5.3 CodePayload、事务与历史

新增`CodePayload`/有界registry（具体Rust名可等价）：保存一次有界读取的原始UTF-8字节/hash和840准入身份，CRLF/LF按原字节保持，BOM/坏字节/超界沿840拒绝。API+bridge/组合上限和CPU/runtime/native边界沿最终840。需要共享一个从受控字节构造SourceSnapshot的入口，文件、绑定与Open都走同一Validator；不得写临时文件再读取来绕过快照或另写parser。

编辑操作为“绑定/更新脚本”“更改绑定角色”“解绑”。读取外部文件、准入/编译、准备候选后，一次接受Document+源码owner+原预览/历史；确认前不能改dirty/当前来源。携带epoch/revision和绑定请求generation，取消/换文档/忙碌后的晚到结果不得覆盖新绑定。逐角色修改也形成一个明确Apply事务；相同元数据/原字节为noop，无额外history/revision。

源码更新是显式重新导入快照；绑定后原文件被编辑/删除不影响已接受Run/Undo/Save，也不热重载。绑定源码读回语法/签名/编译不合法则拒绝整个绑定；合法但运行时Fault的脚本仍可保存，打开项目不擅自执行callback，Run后仍按840Fault/修源更新恢复。

Step/current/pending/save-unverified各自拥有所需Arc，Undo/Redo必须恢复正确原始源码，无需原导入文件。64步/16MiB的原canonical历史预算保持；另加源码原始payload预算：所有当前/undo/redo/pending/未验证保存收据所需的不同源码身份合计**≤8、原始字节≤2MiB**，相同原始hash只计一次；确认hash/长度及字节一致再复用。候选超额在接受前拒绝，不静默删历史/截断源码；无人持有的payload释放后可重用额度。外部合法测试pin单独报告、不作泄漏，不能用registry清空当真实释放证据。

dirty hash包含binding的API/原始SHA/长度/actor；源码字节已由SHA绑定，不把整个文本复制进每步Document。Undo回已保存元数据+payload才clean。绑定读取/运行快照、源码UI路径、诊断、编译对象不逐帧进入历史。New/Open/Close/历史淘汰和取消都验证最后owner释放。

### 5.4 保存、源码闭包与失败恢复

保存Snapshot一次固定entry编码、声明GLB、当前唯一声明CodePayload及stamp；历史源码不随本次保存导出。沿原root lock→entry lock顺序，所有GLB/code先有界准备/flush/回读/不可覆盖安装，再对唯一scene入口执行原同卷rename。保留已有锁名/单写者互斥；新保存必须仍与旧GLB路径共用root锁，不能创建另一把互不互斥的脚本锁。

源码盘额独立于原GLB8文件/32MiB：scripts目录**≤8文件/2MiB**，现存blob、残留及本次临时预留计入；非普通文件、非法/未知/不完整文件不覆盖或自动删除，具名报告。合法已存在源码必须逐字节/hash/长度验证后复用，错内容不可覆盖。目的路径及每个existing ancestor先检查reparse，再解析实际路径；普通relative/case/..别名不能绕过保护，network/device路径保持不支持。

保留entry冲突字节检查与owned文件精确清理；**scene entry不得替换scripts目录、任何规范源码blob、原asset blob、root/entry锁或本协议临时保留路径**，按Windows实际解析目标分类。无效Save As在写盘/确认任何旧收据前拒绝。原旧分母继续，不借源码扩展放宽GLB规则。

entry提交前故障：旧entry不变，保存path/saved hash/dirty不确认；已正确安装但未引用的code/GLB作为有界残留报告，后续重试验证后复用。entry提交后未验证：VerificationPending保留完整snapshot源码/GLB，不能标clean/关闭丢文档；Verify同时核entry字节和全部声明blob，再确认对应epoch/hash。旧epoch永不写入/确认新文档；同epoch旧revision可确认旧snapshot，但较新修改继续dirty。任一坏/缺blob不得只靠entry相同治愈收据。

Save As从当前接受的内存源码/GLB写新root，即使旧磁盘root/导入文件失效也可保存；原有未验证收据的处理先遵守原Verify合同。不得从原导入path重读“最新源码”偷换快照；若新root同hash路径有坏内容则保留并失败。不自动GC/修复外部篡改，不声称跨卷/断电/恶意并发路径替换的事务保证。

### 5.5 Open、Run与正式UI/CLI

Open有界加载并验证entry、所有GLB与源码hash/长度/UTF-8/API/AST/编译、actor引用和实际编辑preview/GPU prepare，全部成功后一次替换Editor/Storage/来源。失败保旧文档、binding/dirty/history/selection/视图；候选检查VM/源码owner释放。Open不自动执行app_start/update；新进程仍由用户明确Run。新epoch清空历史/临时覆盖，恢复持久binding；--inspect需输出schema/binding身份/角色/闭包验证状态及clean/no-history，不冒充运行证据。

正式运行来源区分“项目默认”和“临时覆盖”。默认在behavior存在时用已接受CodePayload+绑定actor，否则原Builtin6与原默认actor。Run前再次核payload原始hash/长度与当前binding、actor ID映射及受检API/bridge身份一致，不可从同名条目或上次临时源取代。原840显式外部文件/--behavior方式继续每次Run有界重读，属于临时覆盖；显式内置选择也可临时试跑。临时来源和其actor选择不改binding、不进dirty/Save；保存或关闭不能暗中把临时覆盖变成绑定。UI清楚显示当前运行的是绑定摘要/角色还是临时来源；持久角色通过专门编辑事务提交，原临时actor选择不冒充已保存。

`autographics-authoring --open <scene.agscene>`打开scene/4后默认Run不再需要--behavior。旧v1..3/无binding入口与显式--behavior保原语义。运行中禁止绑定/解绑/源/actor持久修改及文件操作，Host重新准入、拒绝不排队；Stop仍恢复编辑场景和绑定。Run使用已接受字节，即使磁盘blob后来变动也不静默替换；后续Open/Save/Verify对磁盘闭包如实检测差异。无源码/错API/解析/运行失败不得fallback到内置或旧源码。

### 5.6 仓外项目与资源验证

复用840 Move/Patrol两个实际Auto样例，各增加可保存的scene/4项目夹（拟`samples/behavior_projects/move/`与`patrol/`）。同runner二进制SHA下，经正式UI绑定/Save As后移走原脚本和原root，以不同cwd新进程只--open迁移后的entry，再明确Run/Pause/Step/Stop；核两种逻辑、非零父组、两维actor及全部导入mesh的独立轨迹/像素、原编辑恢复与新进程clean。

另只改用户源码、显式Update→dirty/Undo/Redo→Save→新进程重开，实际行为随绑定hash改变，runner/bridge/Rust未重编译。不把复制旧截图或只比较文档hash当运行验证。删/改源码blob、缺GLB、坏actor/API、绑定失败和保存阶段故障均有具名反例与恢复。临时override的画面改变不应改变磁盘绑定/重开默认行为。

沿840运行/渲染预算、128节点和受限模型/目标/管线复用；新增只计源码owner/文件闭包开销。20轮绑定A/B/取消/失败/Undo/Run/Stop/Open，实际weak payload/VM、GPU Arc/targets/write_buffer及合法外部pin对照；无逐Run模型重传、无每帧读源码/累积报告。sync compile/native/GPU耗时单列，用户heap与驱动驻留保持原计量边界。

### 规范增量

全部为拟议；本轮不改canonical，behavior-api.md先由840交付后才modify。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/behavior-project.md | 无项目源码合同→单blob绑定/持久角色、owner预算、原始快照、临时override与可搬迁闭包 | 最小自包含可运行项目 | AC-02, AC-03, AC-04, AC-06, AC-08 |
| SD-02 | modify | docs/specs/auto-graphics/scene-document.md; docs/specs/auto-graphics/scene-assets.md | scene/3无代码→scene/4/旧版迁移、GLB+code单entry提交/完整回读、源码盘额与保留路径 | 保存与恢复闭包完整 | AC-02, AC-05, AC-06, AC-07 |
| SD-03 | modify | docs/specs/auto-graphics/authoring-session.md; docs/specs/auto-graphics/scene-play-session.md; docs/specs/auto-graphics/behavior-api.md | 临时文件来源→受检绑定编辑/Undo/dirty、默认已接受源码Run、原外部来源语义保持 | 不丢原脚本或污染编辑/运行隔离 | AC-03, AC-04, AC-06, AC-07 |
| SD-04 | modify | docs/specs/auto-graphics/auto-source-driver.md; docs/specs/auto-graphics/project.md; docs/specs/auto-graphics/integration-contract.md | 用户脚本运行→有限仓外行为项目交付/诊断/支持局限，I2/I3仍pending | 交付含义与生产边界准确 | AC-01, AC-06, AC-08, AC-10 |

## 6. 测试设计

本轮只静态调查，不运行Cargo/build/原生/GPU测试。未来work建立下列新fixture/入口，不能把拟议用例当已执行。用户未分配实现/internal/final模型或创建agent权限；同树顺序验收，final review须独立上下文，作者自查不得冒认最终独立审查。

未来845树driver目录先按最终840 prepare/verify工具核固定来源。拟新`tests/behavior_project_document.rs`、`behavior_project_session.rs`、`behavior_project_storage.rs`、`behavior_project_host.rs`及`tools/check_behavior_project.py`、`check_behavior_project_owners.py`；由T-03..08建立真实入口和精确命令，0selected/skip不可当pass。开发用`cargo test --locked --test <对应单一target> -- --nocapture --test-threads=1`、受影响旧定向用例、locked all-targets check及fmt。

| case_id | AC / work owner | 输入/状态与实际入口 | 可区分的observable oracle | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01；T-01/08 | 正式840/SDK/两锁/源闭包/新文件，verify_source+独立consumer及原反控 | 来源/ABI/旧测试分母准确，新源码完整纳入fence；源漂移/缺blob/依赖漂移失败 | 不以work候选当交付 |
| V-02 | AC-02；T-03 | 实际decode/validate/encode；v1..4/重复键/缺字段/错误API/hash/actor；u64大ID/同名/无actor | 旧语义/顺序和blob身份保持，原文件不重写；scene4完整往返，非法具名且消费前拒绝，不缩窄/首对象替代 | 1MiB/128与原静态支持面保持 |
| V-03 | AC-03；T-02/03/04 | 字节来源→840准入/编译→绑定；CRLF/LF/坏字节/BOM/256KiB±1/回调错/注释字符串合法控制 | 固定原字节，绑定/Open不执行callbacks，错误保旧内容/owner；共享840 validator，无文件重读或第二parser | 单文件、行为API不扩展 |
| V-04 | AC-03, AC-04；T-04 | Attach/Update/retarget/Detach/Undo/Redo、删除actor/祖先、noop/取消/stale epoch/revision/generation | 一次接受一次history，dirty与saved hash一致；撤销无原文件仍恢复旧源码，失败不污染；绑定引用失效明确拒绝 | 临时override不进history |
| V-05 | AC-05；T-05 | Storage源码写/flush/install/entry前失败、GLB已安装再code失败、entry后回读失败、Verify/retry、同epoch旧rev/旧epoch | entry前旧bytes不变/残留具名可复用；entry后unverified阻止clean/离开，Verify核全部blob；旧snapshot不clean新修改/新文档 | 本地单写者同卷，不承诺断电事务 |
| V-06 | AC-05；T-05 | scripts8/2MiB边界含temps、坏同hash/未知文件/占锁/保留目标/case/..、缺源/GLB/Save As新root | 原有及未知文件不覆盖删除，坏闭包拒绝；GLB+code用同root锁；Save As只用固定payload，保留路径在旧收据验证前拒绝 | 不使用实际junction测试破坏worktree红线；路径层可构造metadata反控 |
| V-07 | AC-06；T-06 | 实际Host/CLI默认bound/无binding/显式--behavior/显式builtin、busy/run/fault/Stop，Open坏源/编译错误/prepare失败 | 来源/actor清楚且正确；临时覆盖不写binding，坏Open保旧会话；runtime真实Auto/预算/last-good/Stop原隔离，无fallback | 保留840合法源码与外部文件新Run重读语义 |
| V-08 | AC-06, AC-08；T-07 | 两个仓外项目、同runner、删除原root/脚本、不同cwd/new process；Update保存重开及临时覆盖对照 | 只--open+明确Run得正确独立轨迹/各mesh像素；变源码变行为且无Rust重编；运行后磁盘/editing恢复正确 | 开发项目搬迁，不是exe安装包 |
| V-09 | AC-07；T-04/08 | 8源码/2MiB、history淘汰/新edit丢redo、pending/cancel/unverified收据/合法pin、20轮Run/Stop/Open | raw按hash仅一次计量、超额受控拒绝；实际weak owner/VM/GPU最后owner，故意保留/重复上传负控红 | 合法外部pin与计量排除项单列 |
| V-10 | AC-08；T-07/08 | 真实OS→Iced绑定/保存/Run/Stop，actual GPU故障/漏脚本/错误actor/partial；SD/API/CLI说明 | 真实正式入口、最终画面与错误恢复，native/report/aggregate不完整非零；文档逐项对应实际支持与来源 | 静态metadata不代OS/GPU；新cases由work建立 |

每case记录code/SDK/锁/用户源/API/bridge/fixture身份、command/退出码/实际断言数、原始报告及图像。改用户源而不改runner的对照须保二进制hash；失败必须发生在宣称的实际阶段，setup失败不冒充抓住GPU或entry后故障。已有证据只在代码、依赖、配置和输入未变时复用。

T-08列准确完整合入矩阵；**最终独立review pass后**T-10才在冻结提交执行完整locked driver/AGP test+build、原16 AGP与原双游戏、835..840相关原生正反/partial/owner/reopen、counter+content两道门、新项目迁移/持久化正反和独立source consumer。开发/internal/final review只定向/check/fmt，不提前跑整档；本Plan不改根Rust/Schema生成器，不跑root t/tv/tt/tb/tu/taa/docs_gen。若新增SDK/生产写面先回new按触面重新定义门，不能借此默认豁免根门。tf到期仍主检出单实例批量，收尾记录到期判定。

## 7. 验收标准

| ID | owner_stage | 可观察标准 |
|---|---|---|
| AC-01 | work | G840/GSOURCE、唯一树/闭包/只读边界与最终依赖成立，V-01 |
| AC-02 | work | scene4绑定/大ID/nullable角色与旧版迁移/严格decode闭合，V-02 |
| AC-03 | work | 原始源码单份准入/编译和有界payload，无绑定/Open callback执行，V-03/04 |
| AC-04 | work | 绑定编辑/角色/删除约束、Undo/Redo/dirty/取消与stale隔离完整，V-04 |
| AC-05 | work | GLB+code+entry单提交/完整验证/Save As、路径/预算/冲突和失败恢复成立，V-05/06 |
| AC-06 | work | 正式默认绑定Run/临时覆盖/坏Open保护及Stop真实Auto恢复，仓外只open无需原路径，V-07/08 |
| AC-07 | work | 源码quota/真实owner/20轮VM及GPU复用/原分母无回退，V-09及原相关cases |
| AC-08 | work | 两种源码同binary、OS/GPU正反/partial与SD/API/CLI/handoff准确齐备，V-08/10 |
| AC-09 | review | 独立复核AC-01..08与Spec delta、遗漏/延后/workaround/健康，绑定revision/code pass，merge-owned完整门明确pending |
| AC-10 | merge | 冻结完整门通过才落autographics-dev；canonical/ledger/归档/guard清理/到期收据齐备，graphics源码不入master |

## 8. 执行步骤

r2第5轮修复冻结ae1a87e15经R7全新独立上下文受影响最终复审pass，F-845-M1-05旧window-probe输入前提与五处滚动错误传播不变量关闭；R6及以前有效关闭保留。当前reviewed9/10，T07/08/09关闭；全部原失败历史保留，自动修复计数5/用户上限6。完整合入矩阵保留原81项并增加两端Step回归为83项，T10及全部merge-owned完整门仍pending；门通过前不能合入或宣称交付。

| task_id | owner_stage | 依赖 | 真实触点／拟新增路径及动作 | 关闭条件；AC |
|---|---|---|---|---|
| T-01 [x] | work | G840/GSTART | 840正式收据/Spec、唯一845树、SDK/两锁/verify_source | 正式完整SHA/来源及原分母可消费，修订假设后冻结；AC-01 |
| T-02 [x] | work | T-01 | Document/Editor/Storage/840正式behavior接口；新research/behavior-project-feasibility.md | §5.1 whole-path、前后提交故障/Undo源码/不执行callbacks可行性取证，最多两方案，不能闭合先回new；AC-03, AC-05, AC-06 |
| T-03 [x] | work | T-02 | document.rs、新authoring/behavior_project.rs及behavior_project_document tests | scene4/旧版/严格绑定/CodePayload原字节及840统一准入定向通过，V-02/03；AC-02, AC-03 |
| T-04 [x] | work | T-03 | session.rs::Edit/Step/Pending/Editor、history/源码owners；新behavior_project_session | 候选绑定/更新/改角色/解绑/Undo/引用失效/预算/stale关闭，V-04/09主体；AC-03, AC-04, AC-07 |
| T-05 [x] | work | T-04 | storage.rs::Snapshot/load_assets/prepare_closure/verify_locked、源码闭包与新storage tests | V-05/06及原storage/assets故障定向通过、单entry/同root锁/完整Verify；AC-05 |
| T-06 [x] | work | T-05 | Host/Panel/bin、840正式来源选择与Run、Open/inspect；新host tests | F-845-R1-01工作修复：已存场景新绑定未保存时Run/诊断明确memory/hash；来源生命周期断言及原反例经R2独立验闭；其余V-07证据保留；AC-06 |
| T-07 [x] | work | T-06 | 新samples/behavior_projects/{move,patrol}、check_behavior_project.py/真实GPU反控 | V-08/10 OS/仓外/同binary/搬迁/源码更新/错误恢复/partial全证据；AC-06, AC-08 |
| T-08 [x] | work | T-07 | 新check_behavior_project_owners.py、source/独立consumer、README/new research/behavior-project-handoff.md/Spec proposal | F-845-R1-01来源状态永久断言/知识说明经R2独立复审关闭；其余V-01/09/10、owner/source/consumer/check/fmt与merge矩阵证据保留；AC-01, AC-07, AC-08 |
| T-09 [x] | review | T-08 | 同树冻结code/Plan/Spec与原始报告，主检出独立review附件 | AC-01..08逐项独立审查、遗漏/延后/workaround与健康；AC-09；R6异步Step/物理可达性受影响独立复审通过，完整门仍merge-owned pending |
| T-10 | merge | T-09 pass | 冻结完整门→autographics-dev；主检出知识/ledger/archive/guard | AC-10实际落地/清理/到期，改代码先受影响复审再重新冻结 |

## 9. 复审记录

2026-10-24：stage:new；PLAN-845 revision1；outcome:blocked（G840完整交付pending，草案可评审）；next:840完成独立最终审查及冻结合入交付，再由用户启动845 work，T-01冻结正式基面后做T-02。当前drafting0/10；T/AC/V均拟议未通过，本轮无源码实现、建树、依赖准备、构建/运行或新agent/chat。

静态合同检查及来源校验见[preflight](attachments/845/planning/preflight.md)。本轮未改840或AutoUI后续设计范围，不提前把场景项目等同完整引擎/生产Forge。review/merge仍各自负责，不以计划检查替代运行验收；连续同因不收敛先诊断原因与兄弟路径，不暗减分母/改fixture。


### 2026-10-24 work 启动核查（T-01未关闭）

stage: work | plan_id: PLAN-845 | plan_revision: 1 | outcome: blocked |
worktree_path: D:/autostack/.wt/lang-845/auto-lang | branch: plan-845-dev |
base_commit: 8b62db5cba54b9bfe7f759f4920b22ccb9b75e88 |
code_commit: 同base，未修改实现 | dependency_revisions: SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21 |
task_ids: T-01（未完成） | next_responsible_role: prerequisite review / contract decision

用户本次明确授权845 work，GSTART成立。主检出crates/test/scripts无WIP；840已archived26/26，完整门、batch-r3及guard清理收据已存在，正式图形分支为上述base。
已创建唯一845树并断言toplevel/branch；原源码299、SDK681、479packages/17locals/29edges校验通过，verify_plan840 --controls全部原反控及恢复合法控制通过。两锁SHA256分别为2b131d3dd33a968b43d53664cb8649d6c24225fa0f2c28c1add603be98a358db与5192d08d502a8f17e277ad8912f5884175659bef642b614e25b394243d1ff091；SDK仅从固定Git对象重建，无链接，无根构建/整档测试。

进一步核验发现G840的独立复审证据缺口：840 review-r2/review.md明确“current Codex implementation session, without independent context/model isolation”；phase4/review.md亦明确同实现会话。独立consumer和反例不等同独立上下文的最终复审。不能仅据archived字段把本合同的G840标通过。建树/SDK准备早于此项核实，属于本次入口顺序偏差；无实现编辑、提交或运行发生。保留当前clean树，不创建第二树，不把T-01/AC-01标绿。

T-02..08均依赖该门，未开始；计数保持0/10。需先补齐840独立最终复审证据，或明确修订845的G840以接受已披露限制的840收据，再恢复T-01。未缩减实现或验收，未改840归档状态。现有源码/SDK验证仅为入口证据，不能替代未来845闭包/运行测试。

### r2 有限合同修订与恢复执行

用户明确“接受840已披露的复审限制，修订845的G840要求，继续实施计划845”。仅G840改为接受已披露限制的840最终收据；保留840原始限制/历史，不伪称其有独立上下文。845自己的T-09/AC-09仍要求独立复审，所有功能/验证/来源边界不变。
G840/GSTART成立；T-01关闭，正式base8b62db5cba54b9bfe7f759f4920b22ccb9b75e88、SDK932、两锁及299/681/479/17/29来源证据沿上次入口核验，实际clean同树/branch复核通过；原source drift反控输入未变。current_step1/10，T-02执行中，旧blocked记录为历史，当前无产品/合同待决。下一入口：原SourceSnapshot::from_bytes → Session::new_with_diagnostics（只编译，尚不调app callbacks）→ Arc payload → Document/Editor/Storage。采用现有闭包的局部扩展；T-02须验证真实跨保存/Open/Run及前后提交故障，不能以静态分析关闭。


### r2 work 中间关闭：T-02..04

T-02现有闭包局部扩展可行：同一SourceSnapshot::from_bytes/Session编译入口，不执行callbacks；原始payload以Arc进入候选/当前/历史/保存快照。behavior_project_storage whole_path测试实际前后提交故障、坏code Verify拒绝/恢复、删除原root后Save As/Open/真实Run；behavior_project_host实际双mesh GPU/非零父组、Undo/Redo及20轮真实VM弱owner/mesh Arc检查。research/behavior-project-feasibility.md随实现树交付。
T-03：behavior_project_document2/2；session raw identity/bounds/坏字节/BOM/错callback/编译错误及LF/CRLF、256KiB边界通过；scene4严格可空绑定/大u64/旧v1..3迁移，旧文件不写回。T-04：session四例覆盖候选/noop/stale/cancel/Undo/Redo/8源码quota、redo丢弃/新epoch/历史淘汰/祖先删除拒绝；Host两个定向测试覆盖cancel generation/prepare失败/retarget nullable/临时源与显式内置/默认源隔离。旧模块受影响测试有实测证据；未运行整档。
旧hierarchy测试构造v1时遗留v4 behavior字段已修正，原128/u64断言不变；mixed closure断言的Windows canonical长路径前缀不一致已修正，原错误报告保留。owner受控retain-source/retain-VM/fresh-upload均在各自真实weak/Arc断言翻红，restored通过；raw byte外部pin单列，CodePayload最后owner释放仍通过。T-05..08继续收口，当前4/10 executing；尚无最终独立review或完整门pass。


### r2 T-05..07关闭，T-08独立消费核查中

实现提交ce28bdc070afcb2612d27b04cd103602a6fc8f35；冻结/交付元数据后继75277a7b7（runtime/测试/两锁/SDK未变）。树仍为lang-845/plan-845-dev；当前7/10 executing。仅experimental/autographics-auto-driver下35文件；无根源码、SDK、AGP或锁变更。

T-05：五个永久storage用例覆盖单entry/同root锁、GLB先安装后code故障、code前后提交/残留复用、entry+code+GLB完整Verify、旧revision/旧epoch收据owner、盘额/未知文件/占锁/保留目标、允许普通非保留文件名、原始CRLF及runtime-Fault源码Save/Open不执行callbacks。Producer先前路径断言修正后定向通过；consumer将以自身SDK/target复核该五例。
T-06：Host两例通过；compile/Open整候选与默认已接受code/roles、临时外部/显式Builtin、busy/cancel generation/prepare失败/坏Open保护、nullable角色和Stop隔离；inspect新输出closure_verified:true，schema/binding/角色在document中、dirty false/history0明确。
T-07：native-project-final15/15（7正+8负）通过，含真实OS绑定/Undo/Redo/Save As、Move/Patrol/速度更新、搬迁、临时override、source/GLB/actor坏闭包、partial/三GPU反控。之后仅增加inspect显式closure_verified字段及运输元数据（不改原生Run/负控路径）；native-inspect-final7/7在最终binary SHA2f5198a264c2a7200265d361d679b3fdb0baa7a3eecb45f5f2a2265c8f403588重新通过所有正向、source路径身份/新进程inspect。未受影响八负沿用原记录；最终完整15-case仍merge-owned重新执行。samebinary下两个不同逻辑、源码更新/实际轨迹、六像素阶段各六checks及重开clean全部实测，不以截图hash替代像素。

T-08已完成source320/SDK681/479packages/17locals/29edges、原反控/原始项目blob身份、check all-targets/fmt、真实owner负控与20轮目标/mesh上传/CodePayload最后owner/合法raw byte pin；完整矩阵和SD01..04在driver research handoff/proposal。剩余独立consumer自带SDK/target核查中；首次未设置840既有16MiB测试线程/32MiB主线程栈，实际test stack overflow保留，未计pass。已按原公开要求显式配置并从新冻结元数据导出fresh consumer，未改输入/断言/SDK来回避失败。Git提交换行影响的.gitattributes哈希已通过冻结文本规则修正，待Git对象逐项回读确认。


### r2 work 最终交接：pass / execution_done 8/10

stage: work | plan_id: PLAN-845 | plan_revision: 2 | outcome: pass |
worktree_path: D:/autostack/.wt/lang-845/auto-lang | branch: plan-845-dev |
base_commit: 8b62db5cba54b9bfe7f759f4920b22ccb9b75e88 |
code_commit: 75277a7b7b8d1b43f840c8993721c02833e5657a（runtime ce28bdc070afcb2612d27b04cd103602a6fc8f35未变） |
dependency_revisions: SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21，driver/AGP两锁+SDKroot锁完整SHA见evidence |
task_ids: T-01..08 | evidence: [readiness](attachments/845/work-r2/readiness.md)、[原始证据及身份](attachments/845/work-r2/evidence.json.txt) |
blockers: none | next_responsible_role: independent review

T-08已关闭：fresh consumer845-verified持有自己的SDK/target，源码/依赖反控、五storage用例及仅--open新进程真实GPU正控/缺源码负控全部通过，构建后681源文件再次读回通过。所有320输入与最终Git提交对象逐项匹配。原错误默认栈尝试保留；采用840既有16MiB测试线程和32MiB可执行主栈要求，未改SDK/测试语义或阈值。

补充实际UI产物闭环：直接复制native-inspect-final真实OS Bind/Undo/Redo/Save As生成的两个export项目，将各自原导入根（含main.at和原saved根）移至具名保留目录，确认旧路径不存在，再由同SHA runner在另一cwd/新进程只--open搬迁entry运行，两种真实Auto逻辑、六阶段全mesh像素及clean/no-history inspect全部通过。回放脚本/原始报告均已附，不仅对手工sample做搬迁。合入完整矩阵还须在fresh输出中重放该脚本；没有删减原15-case或63 invocation分母。

AC-01..08当前工作证据逐项pass；13新增永久定向case、47旧触面case、owner三反控+restored、native15（未受inspect改动的八负复用）+final正向7+实际UI产物搬迁2、consumer storage5/native2，各自断言/身份/边界见readiness；不把多轮重复相加。不再有work-owned finding/coverage gap/未决合同；无新的work自查债务候选，独立审查仍须自行核验。

T-09/AC-09独立上下文最终review未执行；T-10/AC-10全部完整锁定门、原63+新增项目/消费矩阵、Spec/ledger/归档/autographics-dev落地/guard清理/845到期批量仍pending。未合入、未归档、未清理树、未发布canonical，不宣称最终交付。实现写入冻结，下一角色在同一树复审。

### 2026-10-24 r2 independent final review R1 — needs_fix

stage: review | review_scope: final | reviewer_identity: `/root/review_845`（获用户授权的新独立上下文；父代理后提供来源疑点，复审者独立构造并实跑反例与控制） | plan_id: PLAN-845 | plan_revision: 2 | outcome: needs_fix |
worktree_path: D:/autostack/.wt/lang-845/auto-lang | branch: plan-845-dev |
reviewed_commit: 75277a7b7b8d1b43f840c8993721c02833e5657a | base_commit: 8b62db5cba54b9bfe7f759f4920b22ccb9b75e88 |
dependency_revisions: SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21、三锁/配置与Spec输入完整hash见[audit](attachments/845/review-r1/audit.json.txt) |
evidence: [完整收据/逐AC与SD](attachments/845/review-r1/review.md)、[独立反例](attachments/845/review-r1/source-origin.rs)、[raw](attachments/845/review-r1/source-origin-run.log.txt)、[重放入口](attachments/845/review-r1/replay.ps1) |
findings: F-845-R1-01 P2（原合同§5.2，AC-03/06/08，T-06/08） |
next: 同树work修复、永久反例/控制、提交后仅复审受影响来源状态；不改语义合同，不重跑完整矩阵。

已覆盖全部AC-01..08与规范增量：AC-01/02/04/05/07 pass，AC-03/06 fail，AC-08 partial。已有保存路径的场景绑定全新未保存源码后直接Run，SourceSnapshot.path/Last Run/真实Auto app_start错误均指向尚不存在的scripts/<hash>.at，而非明确memory/hash来源；新无路径、已保存旧binding、新binding保存后三个合法控制通过。反例已进入实际回调（API error node999），exit101为来源断言失败，不是setup。源码执行字节正确，未发现fallback/数据丢失。

其余独立H01..06正反通过：旧收据前保留路径拒绝、真实坏code Verify/Open拒绝及恢复、伪造snapshot拒绝、nullable/跨维严格decode、删旧blob SaveAs/Open原CRLF/不执行callback、8源码与第9拒绝/最后owner/合法raw pin。实际Git对象320/SDK681/479packages/17locals/29edges及280原始附件SHA独立校验；定向library/单一authoring binary重建与fmt通过（仅固定SDK146预存warning）。新binary4c975e855ff1344819be16a0c6463f23598811c5765e236303326d9b8ebc3d49上正式move通过phase27/36像素断言，bad-source与partial按阶段失败；旧原生/owner/consumer证据经代码/输入/依赖/config身份比对复用。

T-06/T-08重开，计数executing6/10；T-01..05/T-07与无关pass证据保留。F01已登记KNOWN-DEBT候选，但属于阻断合同缺陷，不授权延期/降级。冻结SD01..04 hash7985c0feed0c105f5bbea0f2ef29843311804f07c3fa2f331846d665a8e04e9a，canonical/ledger未写；最终知识文本须保留未保存来源规则。T-09未pass，全部merge-owned完整门、autographics-dev落地/知识归档/guard清理/到期回归仍pending；未合入/归档/清树，不宣称交付。

### r2 F-845-R1-01 修复入口（work R3）

用户授权同树修复、永久回归、提交后独立上下文复审。75277a7b7 clean，主检出无代码WIP。实际路径为Storage load/verify确认闭包→来源身份记录→Host默认Run→SourceSnapshot/Last Run/实际callback诊断；不得以Editor整体dirty或磁盘exists代替源码来源。修复保留当前项目根内已确认的有限源码身份（不持有payload），仅load/成功Verify更新；绑定/Undo/Redo按身份选来源，SaveAs换根后不继承旧根证明，pending/失败不能提前确认为新根来源。Run不读盘。定向关闭范围：原持久反例+三控制、无路径/已存空场景首次绑定、几何dirty、源码Undo/Redo、Save/SaveAs、pending与旧revision Verify、Open。旧整矩阵与owner证据按未改内容复用；完整门merge-owned pending。基线反例实跑exit101，raw见attachments/845/work-r3/baseline-replay.log.txt。

### r2 work R3 — F-845-R1-01 修复交接

stage: work | plan_id: PLAN-845 | plan_revision: 2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-845/auto-lang | branch: plan-845-dev | base_commit: 75277a7b7b8d1b43f840c8993721c02833e5657a | code_commit: e518f5fc778cfa6827f7de9eb99b4c0910ef294d | dependency_revisions: SDK932/三锁未变，完整hash见[identity](attachments/845/work-r3/identity.json.txt) | task_ids: T-06/T-08 | evidence: [关闭矩阵/检查/限制](attachments/845/work-r3/readiness.md)、[raw身份](attachments/845/work-r3/manifest.json.txt) | blockers: none | next_responsible_role: fresh-context independent review

F01工作关闭：Load/完整Verify确认当前root的有限源码身份；Run按身份标记真实已确认项目blob或memory/hash，不以整体dirty/exists判断、不读盘、不保留额外payload。两组永久来源生命周期断言通过，原R1反例同三正控exit0，旧Host逻辑受控回退两例准确翻红并恢复绿；几何dirty/首次绑定/Undo/Redo/Open/SaveAs换root/旧revision与pending完整覆盖。两项受影响旧storage检查、lib check/fmt与source320含原反控通过，仅SDK146固定预存warning。首轮fixture路径/selection/pending错误及原红日志保留，不作产品失败或绿证据。

AC03/06/08工作证据恢复pass；其余R1验收和工作证据依代码/配置/输入未变保留，详readiness。T06/T08关闭、execution_done8/10，F01需独立复审核闭；原合同/plan_revision不变，SD01/03补明确来源规则，canonical/ledger未写。T09未pass，全部merge-owned完整门/T10仍pending，未合入/归档/清树。实现已提交冻结，现转移Plan写权给独立复审者。

### 2026-10-24 r2 independent final review R2 — pass / reviewed9/10

stage: review | review_scope: final | reviewer_identity: `/root/review_845_r2`（获授权的新独立上下文，非实现/R1上下文；无进一步委派） | plan_id: PLAN-845 | plan_revision: 2 | outcome: pass |
worktree_path: D:/autostack/.wt/lang-845/auto-lang | branch: plan-845-dev |
reviewed_commit: e518f5fc778cfa6827f7de9eb99b4c0910ef294d | repair_parent: 75277a7b7b8d1b43f840c8993721c02833e5657a | base_commit: 8b62db5cba54b9bfe7f759f4920b22ccb9b75e88 |
dependency_revisions: SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21，三锁/config/Spec输入与R1一致，完整hash见[audit](attachments/845/review-r2/audit.json.txt) |
acceptance_results: AC-01..09 pass、AC-10 pending | findings: F-845-R1-01 independently closed，无新增finding |
evidence: [完整独立收据/逐AC/SD](attachments/845/review-r2/review.md)、[原反例实跑](attachments/845/review-r2/replay-source-origin.log.txt)、[永久2/2](attachments/845/review-r2/permanent-origin.log.txt)、[自选兄弟/失败状态](attachments/845/review-r2/holdout.rs)、[raw](attachments/845/review-r2/holdout.log.txt) |
next: merge owner在本冻结提交执行全部完整门；通过后才落autographics-dev及知识/归档/guard清理。

R1原反例同三个合法控制exit0；未保存B的SourceSnapshot.path/Last Run/真实Auto app_start node999错误均明确memory/hash。永久两例2/2覆盖首次绑定、几何dirty、Undo/Redo、Open、删除导入/已确认blob仍用接受字节、SaveAs失败/pending/旧revision与换根。独立H1同根更名、H2跨根实际坏blob Verify失败/修复/坏Open保旧、H3解绑SaveAs再Undo不继承旧根证明全部通过。Storage仅在load/完整Verify确认有限CodeRef，不保payload，不读盘判来源；真实callback及中间来源观察已复核。

320冻结Git对象、work-r2 280与work-r3 18原始附件SHA逐项无差异；SDK681只读核验、三锁/config/全部Spec输入与R1一致，故其余未受影响AC与原生/owner/consumer证据保留，不因新HEAD重跑完整矩阵。定向library build/fmt/diff健康检查通过，仅SDK146固定预存warning。SD01..04及来源澄清与合同/代码一致；冻结proposal SHA256 `01eaa3e632a062c50bbcd5860df79d5a28e4d56390c881711754fb6416a351c5`，canonical/ledger未写。

T-09关闭、reviewed9/10；T-10开放。pending_closeout_gates: 原63完整调用、driver/AGP全测/构建、原16 AGP/双游戏、authoring/hierarchy/assets/Play/source/owner/counter AND content、producer/fresh own-SDK/own-target consumer、15新project native及实际UI产物搬迁，全部merge-owned pending；精确矩阵沿冻结handoff。root档/docs_gen本driver-only diff不触发，tf仍主检出单实例批量并在845落地判到期。G840豁免仅840历史，845本轮确有独立上下文。未合入/归档/清树，不宣称交付。

### merge R1 → bounded work：旧控件门禁定位失败

冻结e518完整driver门发现authoring_host::actual_create_delete_widgets_and_bad_open create4≠5；raw见attachments/845/merge-r1/driver-test.txt，未合入。独立真实Host探针create-visible.rs/.txt证明action-create y889.6超800px viewport；相同UI/原真实点击路径滚动后y709.6，创建/选中/删除4→5→4通过。属于新增绑定控件后旧测试缺实际scroll定位，不是授权削减旧断言。现回work仅调整该用例的可见控件定位、重冻source清单；T08/T09重开executing7/10，F01保持已闭。原失败/探针保留；相关永久断言/独立复审后在新冻结提交恢复完整门，不改产品或合同。其他已有source/SDK/输入证据按未变部分保留，完整门仍pending。

### gate定位修复交接：F-845-M1-01 / work pass

code_commit a0e2a9115a0b48bab44e3acaef17485dca472420（parent e518）；仅authoring_host测试与source清单两文件。实际click前通过原Host.scroll+frame将控件滚到800px视口，30次有界、不可达明确失败，不直接dispatch、不删原create/select/delete/badOpen/pixel断言。原失败与独立create-visible实际探针为red/control；永久原入口actual_create_delete_widgets_and_bad_open定向1/1绿，fmt与source320绿，SDK/锁/产品代码完全未变。证据位于attachments/845/merge-r1/create-visible*.txt/.rs；本轮不跑完整套件以替代受影响复审。T08工作关闭，execution_done8/10，T09受影响独立review待定；上一F01独立关闭结论不变，所有merge完整门待新冻结复审后执行。实现clean，写入暂停，转交独立review。

### 2026-10-25 r2 independent affected final review R3 — pass / reviewed9/10

stage: review | review_scope: final（merge失败后的受影响复审） | reviewer_identity: `/root/review_845_gate`（获授权的新独立上下文；非实现/R2复审上下文，无进一步委派） | plan_id: PLAN-845 | plan_revision: 2 | outcome: pass |
worktree_path: D:/autostack/.wt/lang-845/auto-lang | branch: plan-845-dev |
reviewed_commit: a0e2a9115a0b48bab44e3acaef17485dca472420 | repair_parent: e518f5fc778cfa6827f7de9eb99b4c0910ef294d | base_commit: 8b62db5cba54b9bfe7f759f4920b22ccb9b75e88 |
dependency_revisions: SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21，三锁/config/16项Spec输入/提案与R2相同，见[audit](attachments/845/review-r3-gate/audit.json.txt) |
acceptance_results: AC-01..09 pass；AC-10 pending | findings: F-845-M1-01独立关闭，F-845-R1-01保持已闭，无新增finding |
evidence: [受影响独立收据/复用边界](attachments/845/review-r3-gate/review.md)、[永久原入口1/1](attachments/845/review-r3-gate/permanent.log.txt)、[真实Host反例及可见输入控制](attachments/845/review-r3-gate/visible.log.txt)、[Spec冻结](attachments/845/review-r3-gate/spec-delta-frozen.md) |
next: merge owner在本冻结提交恢复全部完整门；通过后才autographics-dev落地/知识归档/guard清理。

独立重编实际Host探针：action-create未滚动y889.5998，原真实click保4；scroll/frame后y709.5998真click创建5，node5真选中、delete恢复4并清selection。永久actual_create_delete_widgets_and_bad_open原入口1/1通过。修复仅30次有界可见定位+source清单单hash更新；完整diff核实原create/select/delete/badOpen/document/history/pixel断言均保留，无直接dispatch/忽略/阈值放宽。source320 Git对象及文件验证、SDK681只读验证、fmt/diff健康通过，只有SDK146固定预存warning。

AC01身份刷新及AC07/08旧门保留受影响重新验收；生产/SDK/AGP/锁/输入/配置/Spec proposal未变，其余R2 AC证据与F01关闭按明确身份复用，不重跑已闭来源/GC/原生/consumer家族。SD01..04提案冻结SHA256 `01eaa3e632a062c50bbcd5860df79d5a28e4d56390c881711754fb6416a351c5` 与R2一致，canonical/ledger未写，无合同修订或新债务。

T09关闭、reviewed9/10；T10及原63完整调用、driver/AGP全测/构建、原16 AGP/双游戏、所有counter AND content/owner/source/producer/fresh consumer、新15 project native及实际UI产物搬迁完整门仍merge-owned pending。旧完整driver失败不计pass；本轮仅受影响定向复审，无完整套件/root档/docs_gen/批量回归。未合入/归档/清树，不宣称交付。最终WT仍a0e2 clean。

### merge R1 → F-845-M1-02 工作定位

a0e2生产/独立消费完整driver155及AGP163各通过，旧owner三入口通过。新project-owners前两反控正确，fresh-upload在真实reupload后先触发stable bound resource mesh_uploads（18/137208→22/137464），故脚本等待后置Arc消息失败；raw保留merge-r1/project-owners。产品无默认重传，restored完整用例此前绿。本轮不放宽runner字符串/不跳过断言：仅把同一永久用例的真实Arc身份断言移至计数断言之前，让原fresh-upload负控与定义的Arc oracle一致，全部资源计数/目标/owner与restored20轮仍保留。原始red/control已经触达实际GPU替换；回work定向四反控/恢复后独立复审，T08/T09重开executing7/10。无关既有门仅在输入/配置/代码未变时按精确边界复用；受影响完整driver/新owner/source/fmt将于新冻结提交补跑，不用旧全套掩盖改动。

### F-845-M1-02 work 修复交接

冻结152bc7fa6c6aa5158ae97868cf2e3b28acf136fd，parent a0e2；仅永久behavior_project_host用例移动原Arc身份检查块到上传计数检查之前，另重冻manifest。没有删除/修改任何断言、生产或脚本；同一强制真实reupload现命中warm model must retain actual GPU Arc，原计数/target/owner断言在restored20轮全部保留。check_behavior_project_owners.py原聚合4控制实跑exit0，逐项101/101/101/0及原消息见merge-r1/project-owners-repair/results.json、对应txt日志；fmt/source320通过。工作关闭T08、execution_done8/10，T09待受影响独立review。原R2 F01及R3可见点击修复保持，未动相应代码/断言。此前已绿的AGP/旧owner/未改模块只按确切未变输入复用；修改所在完整driver档及消费者/manifest检查须在新review后重跑，后续native尚未开始。实现clean且冻结，移交独立review，无产品/合同缩面。

### 2026-10-25 r2 independent affected final review R4 — pass / reviewed9/10

stage: review | review_scope: final（merge失败后的受影响复审） | reviewer_identity: `/root/review_845_owner_gate`（获授权的新独立上下文；非实现或前序复审上下文，无进一步委派） | plan_id: PLAN-845 | plan_revision: 2 | outcome: pass |
worktree_path: D:/autostack/.wt/lang-845/auto-lang | branch: plan-845-dev |
reviewed_commit: 152bc7fa6c6aa5158ae97868cf2e3b28acf136fd | repair_parent: a0e2a9115a0b48bab44e3acaef17485dca472420 | base_commit: 8b62db5cba54b9bfe7f759f4920b22ccb9b75e88 |
dependency_revisions: SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21，三锁/config/16项Spec输入/提案与R2/R3一致，见[audit](attachments/845/review-r4-owner/audit.json.txt) |
acceptance_results: AC-01..09 pass；AC-10 pending | findings: F-845-M1-02独立关闭，F-845-R1-01/F-845-M1-01保持已闭，无新增finding |
evidence: [独立收据/复用边界](attachments/845/review-r4-owner/review.md)、[原工具四控制实跑](attachments/845/review-r4-owner/controls-run.txt)、[真实Arc反控](attachments/845/review-r4-owner/controls/fresh-upload.stderr.txt)、[restored原入口](attachments/845/review-r4-owner/controls/restored.stdout.txt)、[Spec冻结](attachments/845/review-r4-owner/spec-delta-frozen.md) |
next: merge owner在本冻结提交重跑受影响完整driver/consumer/source/fmt并补齐未执行native等完整门，再落autographics-dev及知识归档/guard清理。

完整差异仅原八行pin/Arc断言块移至资源计数前、单manifest hash刷新；所有原计数/target/VM/源码owner/真实像素和20轮断言原样保留，pin为真实store非变更读取。原工具新输出独立实跑exit0，三反控101/101/101分别命中源码最后owner、VM Stop、真实GPU Arc，restored0且1passed/0ignored、20轮完成。旧fresh-upload红确由同一次真实reupload先命中mesh_uploads 18/137208→22/137464，现准确命中原Arc oracle；未放宽工具字符串/断言或改产品行为。source320 Git对象/文件、SDK681、fmt/diff健康均通过，仅原SDK146预存warning。

AC01身份刷新、AC07/08 owner/原门保留新验；生产/SDK/AGP/锁/配置/fixtures/原生输入/Spec均未变，其余AC与已闭findings按R2/R3身份明确复用，未重跑来源/GC/native或全档。SD01..04冻结SHA256 `01eaa3e632a062c50bbcd5860df79d5a28e4d56390c881711754fb6416a351c5`保持，canonical/ledger未写，无合同修订/新增债务。

T09关闭、reviewed9/10；T10开放。pending_closeout_gates: 修改所在producer与fresh own-SDK/own-target consumer完整driver及相应source/manifest/fmt，尚未执行native矩阵、实际UI产物搬迁及原63/project其余完整门。a0e2已绿AGP163/旧owner等只有输入/依赖/config/代码不变部分可复用，不称R4新跑；旧155全driver不覆盖本次修改文件。root档/docs_gen不触发，tf仍主检出单实例批量且845收尾判到期。最终WT仍152bc7fa clean；未合入/归档/清树，不宣称交付。

### merge R1 → F-845-M1-03：原生输入夹具坐标一致性

原16AGP/合成两反控及旧双游戏/创作窗口已过。AGP首轮物理click未收到（presses0），未冒称通过；同冻结exe/PID窗口放置在屏幕内并验证foreground/hit PID后原3图像/counter断言通过，非产品更改。继而interaction positive phase2 temporary pixels unchanged。raw输入明确down690,380→synthetic move730,400→OS真实cursor回传690,380；真实mouse仍在起点，最新样本回零，故不能拿该次伪造窗口事件流当产品反例。公共probe::post目前只维护posted坐标，phase0物理预置不能覆盖后续drag/scroll/click；同helper被hierarchy/assets/play/user/project共享。

本轮第三次有界自动work/review修复：仅原生验证helper的WM_MOUSEMOVE路径在同PID HWND ClientToScreen后同步真实SetCursorPos，再保留原PostMessage路径。不得抑制CursorLeft/取消/blur，不改Host/VM/输入生产逻辑、时序预算或任何原断言。相关old interaction正反/partial、代表hierarchy/assets拖动及user move/wrong-focus需定向验证，明确共享路径后独立review；完整门受影响项在新冻结后执行。T07/T08/T09重开executing6/10；M1/02及F01原关闭保留。失败raw merge-r1/interaction.json/.txt保留。后续若仍有代码性阻断，不再盲目第4轮，须诊断并交回具体缺口。

### F-845-M1-03 work 修复交接

冻结da43267d4dffe9064d9d95585b7b56416b730d5e（parent152bc7fa），仅Windows原生probe::post在WM_MOUSEMOVE时以已验证本PID HWND做ClientToScreen/SetCursorPos，同步真实指针后保留原PostMessage；另重冻source320。未改Host/VM/应用输入、取消/blur、期限或任何验收断言。bin定向测试、PE32MiB构建、fmt/source/diff检查通过。原interaction完成phase27；wrong-target/drop-release/partial52/truncated1四反控均按原oracle失败，hierarchy/assets/user Move三正及user wrong-focus反控通过，九项raw见attachments/845/merge-r1/pointer-work及pointer-work.py。父调用者首次误写完成phase26，实际native已exit0/27；保留原调用日志并按源协议修正为27，不削弱native断言。

T07/T08工作关闭，execution_done8/10，T09待全新上下文受影响独立复审；现有R2/R3/R4未变内容与关闭结论保留。Spec proposal及依赖/锁/SDK未变，完整driver/消费者及共享helper所有native仍须新冻结提交的merge门。未合入/沉淀/归档/清理；第三次有界修复之后不盲目开启第四轮。实现冻结clean，移交Plan写权给独立review。

### 2026-10-25 r2 independent affected final review R5 — pass / reviewed9/10

stage: review | review_scope: final（merge失败后的受影响复审） | reviewer_identity: `/root/review_845_pointer_gate`（用户授权全新独立上下文，非实现及前序review，无进一步委派） | plan_id: PLAN-845 | plan_revision: 2 | outcome: pass |
worktree_path: D:/autostack/.wt/lang-845/auto-lang | branch: plan-845-dev |
reviewed_commit: da43267d4dffe9064d9d95585b7b56416b730d5e | repair_parent: 152bc7fa6c6aa5158ae97868cf2e3b28acf136fd | base_commit: 8b62db5cba54b9bfe7f759f4920b22ccb9b75e88 |
dependency_revisions: SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21，三锁/config/16项Spec/提案未变，见[audit](attachments/845/review-r5-pointer/audit.json.txt) |
acceptance_results: AC-01..09 pass；AC-10 pending | findings: F-845-M1-03独立关闭，F-845-R1-01及F-845-M1-01/02保持已闭，无新增finding |
evidence: [独立收据/共享影响及复用边界](attachments/845/review-r5-pointer/review.md)、[独立原生调用](attachments/845/review-r5-pointer/native-run.txt)、[真实PID/物理坐标](attachments/845/review-r5-pointer/interaction.pointer.json.txt)、[patrol/wrong-focus holdout](attachments/845/review-r5-pointer/user-holdout/results.json.txt)、[真实raw manifest](attachments/845/review-r5-pointer/manifest.json.txt) |
next: merge owner在本冻结提交执行受影响完整门及所有共享原生、实际UI保存搬迁；通过后才autographics-dev落地/知识归档/guard清理。

独立检查原helper保留FindWindow/本PID拒绝、原PostMessage/wheel/key及所有取消/blur/resize/期限/断言；仅WM_MOUSEMOVE先ClientToScreen/SetCursorPos，失败明确返回。重建PE32MiB二进制b1465fe2aac515541bb891d44d76bcbbc66d5436f762615a8188a7511dd35e44；真实runner/window PID29708且系统物理client345,190→365,200→375,205。interaction27正控通过；wrong-target1、drop-release4原期限超时、partial52 phase3均正确失败。独立hierarchy16七组真实像素绿；自选Patrol27六阶段×六像素与wrong-focus phase5反控绿，Stop/保存重开由原工具核实。work九项raw与asset九组×四像素断言逐项复核，单帧phase0保留；未冒称本review重跑asset或全矩阵。定向completion4/4、source320/SDK681/479packages/17locals/29edges、fmt/diff健康通过，仅SDK146固定预存warning。

本轮共享影响包括interaction/hierarchy/assets/Play builtin及user/project、旧authoring的scroll；这些完整native均须新冻结的merge-owned门。纯库/owner、AGP/独立游戏、SDK/锁/配置/fixtures/Spec未变，55项R2/R3/R4原始manifest hash逐项核实，未变AC与既有finding关闭按明确边界复用。SD01..04冻结SHA256 `01eaa3e632a062c50bbcd5860df79d5a28e4d56390c881711754fb6416a351c5`保持，canonical/ledger未写，无新债务/合同缩面。

T07/08保持关闭、T09关闭，reviewed9/10；T10与全部merge-owned完整门pending，包括producer/fresh own-SDK own-target consumer/driver与build/source、原63/project尚余门、所有共享native、counter AND content、owner/source及新15 project native和实际UI产物搬迁；仅未变旧门可精确复用。root档/docs_gen本driver-only不触发；tf仍主检出单实例批量及845收尾到期判定。最终WT仍da43267d clean，所有本review原生进程/构建结束后移交Plan写权；未合入/归档/清树，不宣称交付。

### merge R1 — F-845-M1-04 / needs_fix（自动修复上限）

stage: merge | plan_id: PLAN-845 | plan_revision: 2 | outcome: needs_fix | frozen_commit: da43267d4dffe9064d9d95585b7b56416b730d5e | evidence: [完整收据/状态](attachments/845/merge-r1/receipt.md)、[诊断](attachments/845/merge-r1/failure-m1-04.json.txt)、[消费端raw](attachments/845/merge-r1/consumer-user/move/report.json.txt)、[实际截图](attachments/845/merge-r1/consumer-user/move/report.png)。

新冻结producer/consumer完整driver各155/0/0，AGP各163/0/0（未变来源明确复用）；原16AGP/两合成反控/旧双游戏、producer interaction/反控/reopen/hierarchy/assets/Play/user13均通过。消费端assets/hierarchy/两pixel-fault/Play通过；consumer-user第一Move正例phase6失败。phase5暂停tick54并投递实际Step click，phase6pending false、仍Paused/tick54而非55、runtime error null。生产端相同冻结源正例phase27通过；是否原生点击未被接收还是consumer单步分发失败尚未证实，不把它当环境flake或允许重跑到绿。共享路径是play::Probe phases5/6→native click/post→Host play-step→PlaySession single_step，并覆盖后续user/project native。

T07/T08/T09重开、executing6/10，AC08当前red，受影响AC06/07完整门待补；R5和其他关闭历史保留，无新源码改动。consumer其余12user、producer/consumer各15project与实际UI输出搬迁各2、完整81-call oracle审核均未跑；此前通过原始证据按实际身份保留，不称全部完整门绿。需先观察真实OS/widget接收、pending/tick过渡并区分fixture/产品原因，再做同树永久回归/控制与修复提交、受影响独立复审、冻结后补跑所需完整门。

三次自动work/review修复M1-01/02/03已分别独立关闭，达到auto-plan-work默认cap three；本轮不盲目开始第四次修复。按技能交回具体阻断，用户提高自动修复上限后可继续同一树；无需重建树或重跑未变绿色。source HEAD clean da43267d，autographics-dev仍8b62db5c；canonical/ledger未动，所有准备稿未执行；未合入/归档/清理，postlanding批量到期门也未启动。原source/SDK/manifest/锁配置后置核验两端仍通过。故障候选登记KNOWN-DEBT但不授权延期/豁免。

### r2 work 第4轮入口 — F-845-M1-04，上限6

2026-10-25用户将自动work/review修复上限从3提高至6，并授权修复→提交→独立上下文复审→完整合入/沉淀/归档。沿唯一WT plan-845-dev clean da43267d继续，前3轮计数不重置。实际路径是native Step click→OS/Iced widget接收→Host play-step→PlaySession single_step/pending→Auto tick候选→Host GPU准入/accept→Paused新tick；phase5/6目前仅发点击后等固定12frame，不能证明命令接收。先用同消费exe/耐久输入并启用现有--interaction-probe观测真实mouse/widget/error/pending分界，区分错坐标/丢事件与运行时问题；不先放宽tick+1断言或增加盲重试。共享user/builtin/project Play；需原失败/合法控制、恰好一次与拒绝/未接收控制、代表项目/Patrol及必要bin/module检查。完整driver/全部原生矩阵仍merge-owned，定向关闭后提交和全新上下文复审。Spec合同不变，主检出代码WIP预检为空，canonical/ledger保持不动。

### F-845-M1-04 work 第4轮修复交接（上限6）

stage: work | plan_id: PLAN-845 | plan_revision: 2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-845/auto-lang | branch: plan-845-dev | base_commit: da43267d4dffe9064d9d95585b7b56416b730d5e | code_commit: 944c1c551b524d7d8c7f69519e465831417d21a8 | task_ids: T07/T08 | evidence: [逐不变量/边界/原始失败分类](attachments/845/work-r4-step/readiness.md)、[全部raw哈希](attachments/845/work-r4-step/manifest.json.txt) | blockers: none | next_responsible_role: fresh-context independent final review。

Step只投一次实际OS click；phase6等实际Paused/tick+1而非把固定12帧当接收证明，原360帧截止不改，少tick最终超时、多tick即时拒绝。永久normal/delayed/drop/duplicate实跑0/0/1/1，并断言真实Step矩形内down/up、原六阶段36GPU checks、丢失永无新tick、重复实际+2；受控恢复旧early-check使delayed正例按原phase6消息翻红，恢复绿。原生probe专用整窗工作区放置与每点实际坐标/WindowFromPoint同PID验证，正常App不改；受控裁切证明任务栏命中和真实CursorLeft导致后续动作丢失，修复后同类设置明确fixture拒绝。原故障缺历史鼠标事件，不能唯一追溯是延迟还是遮挡；不编造VM缺陷或把单次重跑绿当关闭。两项实际夹具缺口及可区分控制均见readiness。

Patrol/wrong-focus、bound-project Move、legacy interaction27/wrong-target1、completion4/4、fmt/source321/SDK681均通过，锁/产品/Spec proposal不变，仅SDK146旧warning。所有观察/脚本初始错误和无效设置日志保留且不计绿；原耐久反例scene已按Git802bytes恢复，诊断Save的818bytes副本保留。T07/T08关闭、execution_done8/10；T09待独立验闭，T10完整driver/AGP/83项两端原生/项目/实际UI搬迁等门pending。无canonical/ledger/合入/归档/清理。实现clean且冻结，Plan写权移交独立review者。

### 2026-10-25 r2 independent affected final review R6 — pass / reviewed9/10

stage: review | review_scope: final（merge失败后的受影响复审） | reviewer_identity: `/root/review_845_step_gate`（用户授权全新独立上下文，非实现及R1–R5 review，无进一步委派） | plan_id: PLAN-845 | plan_revision: 2 | outcome: pass |
worktree_path: D:/autostack/.wt/lang-845/auto-lang | branch: plan-845-dev |
reviewed_commit: 944c1c551b524d7d8c7f69519e465831417d21a8 | repair_parent: da43267d4dffe9064d9d95585b7b56416b730d5e | base_commit: 8b62db5cba54b9bfe7f759f4920b22ccb9b75e88 |
dependency_revisions: SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21，三锁/config/16项main canonical Spec/提案未变，见[audit](attachments/845/review-r6-step/audit.json.txt) |
acceptance_results: AC01..09 pass；AC10 pending | findings: F-845-M1-04在已证fixture不变量边界独立关闭，F-R1-01/M1-01/02/03保持关闭，无新增finding |
evidence: [独立收据/AC映射/复用边界](attachments/845/review-r6-step/review.md)、[永久四控制实跑](attachments/845/review-r6-step/step-controls-run.txt)、[实际tick/事件](attachments/845/review-r6-step/step-summary.json.txt)、[自选Patrol120/超期限361及legacy交互](attachments/845/review-r6-step/holdout-results.json.txt)、[实际taskbar PID护栏](attachments/845/review-r6-step/occluded-hitpid/observer.json.txt)、[Spec冻结](attachments/845/review-r6-step/spec-delta-frozen.md) |
next: merge owner在冻结944c1c55执行受影响完整门及83项两端原生/项目/实际UI输出搬迁，通过后才autographics-dev落地/知识归档/guard清理。

独立重建PE32MiB二进制173183e31cd3341c4db645c84e759e92e09b02058218ded5ee0ada6138a8155c；永久normal/delayed/drop/duplicate实际退出0/0/1/1，前两phase27/36GPU checks，缺失无新tick且phase6原期限红，重复实际+2按原oracle红。自选Patrol延迟120帧正例27/36像素，延迟361帧在原360预算下phase6拒绝且零Step事件；无重试/重置期限。真实裁切先证requested2562/actual2559不可达（首次caller仅预期occlusion字符串错误保留），再以可达2518/taskbar PID5416≠runner18048准确触发foreign PID拒绝。legacy interaction27与drop-release phase4红均保原断言，completion4/4、source321 Git对象与文件/SDK681/479packages/17locals/29edges、fmt通过，只有SDK146固定预存warnings。

完整diff仅原生probe/永久回归/README/handoff/manifest七文件；正常App、Host/VM、取消/blur/原GPU/期限断言未改。206项work raw hash逐项核实，实际旧early-check回退使永久delayed正例按原消息翻红；旧裁切/真实CursorLeft及Patrol/wrong-focus/bound Move证据按确切身份复用。原历史54→54缺mouse receipt，不能唯一追溯延迟vs遮挡，不宣称已证VM缺陷/环境flake；关闭的是实际已证的异步确认与物理可达性夹具缺口。生产/owner/storage/SDK/AGP/锁/输入/config未变，其余AC按R2..R5证据边界保留。SD01..04冻结SHA256 `01eaa3e632a062c50bbcd5860df79d5a28e4d56390c881711754fb6416a351c5`保持；无合同缩面/新债务/canonical或ledger发布。

T09关闭、reviewed9/10；T10和全部merge-owned完整门pending：producer/fresh own-SDK own-target consumer完整driver/AGP/build/source/owner、原81调用+两端Step回归=83调用、共享native/user/project/counter AND content与实际UI产物搬迁。未变旧证据仅可身份一致复用，不冒称本轮全包绿。driver-only不触发root整档/docs_gen；tf仍845落地清理后主检出单实例due批量。自动修复计数4of6不重置。最终WT clean944c1c55，本review全部子进程结束且无存活authoring进程，移交Plan写权给merge owner；未合入/沉淀/归档/清树，不宣称交付。

### merge → work第5轮：F-845-M1-05 旧创作窗口phase1

944c1c55两端full driver155、两端Step四控制及旧双游戏已过。authoring-original phase1首次属性输入定位超时，raw merge-r1/step-final/authoring-original.json；观测baseline证明真实node1 down/up已经进入且selection1，故不是节点选择未收到，而是随后field0定位/scroll停住。旧入口使用另一套PowerShell posted click/key；scroll则调用新共同可达性helper但忽略其Result。现按用户上限6进入第5轮，主树源码944 clean，T08/T09重开executing7/10；先轻量记录field bounds/scroll结果，区分缺布局、视口与未处理的原生输入前提错误，再修同类分支。须保留全部原字段/像素/Save/Undo/Redo/resize/负控与全局1400frame预算，原Step修复及不相关pass保留。无合入/沉淀/归档/清树。


### F-845-M1-05 work第5轮交接（上限6）

stage: work | plan_id: PLAN-845 | plan_revision: 2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-845/auto-lang | branch: plan-845-dev | base_commit: 944c1c551b524d7d8c7f69519e465831417d21a8 | code_commit: ae1a87e15b857329efc1110beb9c76e9c0299baa | evidence: [不变量/调试边界](attachments/845/work-r5-legacy/diagnosis.md)、[ready/复用/待门](attachments/845/work-r5-legacy/readiness.md)、[原始哈希](attachments/845/work-r5-legacy/manifest.json.txt) | blockers: none | next_responsible_role: fresh-context independent final review。

旧入口焦点准备、所有click/key统一验证原生通路，五处scroll Result传播，错误携phase即时退出；保留原target/GPU/Save/Undo/Redo/resize与1400frame预算。原完整正例phase9/280/accepted3；错目标与截断负例保持红。新永久CLI反例真实field裁切路径即报错，恢复旧五处丢弃Result使断言按原phase1超时消息翻红，恢复绿；completion4/fmt/source321/SDK681通过，仅SDK146旧warnings。共享helper/生产Host/VM/SDK/锁/Spec proposal不改，M1-04等未变证据按身份复用。无未完work-owned项，execution_done8/10，T09待独立上下文；完整driver156/AGP163两端与83项仍merge-owned pending。所有早期无效设置与修正记录保留，不计绿或豁免。未合入/沉淀/归档/清理。

## 10. 待澄清事项

无阻塞草案的产品问题。完整项目为单scene入口+声明GLB+一个内容寻址用户源码/角色绑定，保留840临时源路径与可信源码局限；G840已按用户r2决定成立，正式API/SDK来源已由T-01核实。T-02可选等价局部接法，无须重复请求常规选择；若需多脚本/多文件包、SDK/AGP/生产写面、跨进程/安装发行或改变原行为支持与资源保证，回new具体修订后再开展依赖工作。

2026-10-24 历史work blocker（已由r2用户决策解除）：G840独立最终复审证据缺口，详§9首次核查。用户启动授权已具备，无需再次确认实施845本身。

2026-10-24 r2：上述G840 blocker经用户明确决策已解除；不再要求重复批准。

### 2026-10-25 r2 independent affected final review R7 — pass / reviewed9/10

stage: review | review_scope: final | reviewer_identity: `/root/review_845_legacy_gate`，用户授权的新鲜独立上下文，非实现与R1–R6会话；未委派/改实现/另建树 | plan_id: PLAN-845 | plan_revision:2 | outcome:pass | worktree_path:`D:/autostack/.wt/lang-845/auto-lang` | branch:`plan-845-dev` | reviewed_commit:`ae1a87e15b857329efc1110beb9c76e9c0299baa` | base_commit:`944c1c551b524d7d8c7f69519e465831417d21a8`，原正式base8b62db5cba54b9bfe7f759f4920b22ccb9b75e88 | dependency_revisions: SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21，三锁/config/16项main canonical Spec未变 | spec_inputs: [audit](attachments/845/review-r7-legacy/audit.json.txt)，[冻结SD01..04](attachments/845/review-r7-legacy/spec-delta-frozen.md)，proposal SHA256 `01eaa3e632a062c50bbcd5860df79d5a28e4d56390c881711754fb6416a351c5`。

acceptance_results: AC01来源身份刷新pass；AC02..05按R2–R6精确未变代码/依赖/config/输入复用；AC06..08受影响旧原生门独立验闭pass，R6 Step/common helper字节相同复用；AC09 pass，AC10 pending。完整五文件diff核实五处scroll均传播Result、所有旧click/key走共同验证通路、原selection/target/GPU/Save/Undo/Redo/resize/1400帧预算保留。永久实际native反例1/1、completion4/4通过；work真实五点忽略Result mutation红/恢复绿已独立审计，未冒认review改实现重跑mutation。

独立同binary完整旧window正例phase9/280帧/accepted3/-551与真实End/字符1事件通过，物理2000×1400除scale2等于原逻辑1000×700；wrong-target保持phase1/1401红，partial1帧保持incomplete，scroll fault第61帧携原phase1错误红；自选相邻interaction27正例与drop-release phase4反控通过。[原始运行](attachments/845/review-r7-legacy/native-results.json.txt)、[收据/AC映射](attachments/845/review-r7-legacy/review.md)。source321 Git对象与文件/SDK681/479packages/17locals/29edges、39原始附件hash、fmt/clean HEAD通过，仅SDK146固定旧warnings。review caller的DPI和CRLF哈希比较错误保留为探针纠正，不计产品finding。

findings: F-845-M1-05关闭，无未完成work-owned验收/非法延期/workaround；T08表格漏勾已按实际证据纠正，T09关闭；自动修复5/6，未重置。pending_closeout_gates: 两端完整driver156/AGP163+build/source/owner、83调用native/user/project/Step与实际UI项目搬迁/counter AND content等统一门仍merge-owned pending；root t/tv/tt/tb/tu/taa/docs_gen本driver-only触面不触发，tf到期/批量仍主检出单实例。未合入/知识发布/归档/清树，图形源码只落autographics-dev。review子进程全部退出/reaped；Plan写权移交parent merge owner，下一步完成冻结完整门后按merge收尾，不宣称门前交付。
