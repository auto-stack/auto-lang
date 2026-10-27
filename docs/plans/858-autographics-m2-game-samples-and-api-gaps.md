---
plan_id: PLAN-858
status: executing
feature_name: AutoGraphics M2双游戏样本冻结与公共接口缺口调查
author: [Codex]
created_at: 2026-10-27
updated_at: 2026-10-27
plan_revision: 3
supersedes_spec_components: [docs/specs/auto-ac/mir-native-cache.md, docs/specs/auto-av/cache-and-conformance.md, docs/specs/auto-av/resident-function-install.md, docs/specs/auto-cte/frozen-cache.md, docs/specs/auto-semantics/query-cache.md, docs/specs/auto-abi/conformance-cache.md, docs/specs/auto-test-case/profile-v2-native.md, docs/specs/auto-test-case/architecture.md, docs/specs/auto-lang/project.md]
new_spec_components: []
touched_goals: []
affects: [auto-graphics, auto-test-case, auto-lang, auto-ac, auto-av, auto-cte, auto-semantics, auto-abi]
current_step: 11
total_steps: 14
---

# [PLAN-858] AutoGraphics M2双游戏样本冻结与公共接口缺口调查

## 0. 变更摘要

**r2 用户裁定（2026-10-27）**：不使用独立 PLAN-859；将其 AV 语料可选依赖修复转为本计划 Phase 2。此前 Phase 1 已完成/交付/归档 7/7，受审及合入证据保持有效；[原归档快照](archive/858-autographics-m2-game-samples-and-api-gaps.md) 保留历史链接。用户明确要求同一计划增加阶段，故本计划由 archived 返回 executing（对原终态规则的本次明确例外），r2 工作从 7/11 开始；新的代码、审核及合入证据仅适用 Phase 2，不能倒写 Phase 1 已交付事实。本轮仅修订计划，未实施修复或运行测试。

**r3 用户裁定（2026-10-27）**：ACE 开发初期放弃拍脑袋的 P95<1000ms 周转优化目标，改为 P90≤2000ms；10 次有效样本最多 1 次超过 2000ms。架构雏形完成后，以真实测试重新确定性能目标。按此前“不新建 859、在 858 加阶段”的授权，保留本计划并追加 Phase 3；Phase 2 的代码正确性复审记录仍作为历史证据，本次新增修改与规范增量须独立复审。已有合入尝试失败事实不改写。

M2/S2的前置调查：固定2D横版平台与3D第三人称平台探索两款**自有小关卡**的功能分母、参考提交/许可/资产闭包、下载组织，以及现有公共API→需求缺口矩阵。它产出后继可执行合同的输入，不开发游戏/引擎/下载器，也不把研究完成当M2游戏完成。

## 1. 目标

1. 两款样本具备固定最小玩法、状态/输入流程和可检查的完成定义，明确参考实现与完整复刻差别。
2. 参考/资源绑定不可变来源、真实字节SHA/体积、代码与各资产许可；实测决定小资源随Git/大包按需/制作源文件可选下载。
3. 基于已交付AG公共行为接口与实际源码识别能力，区分已有通用、专用游戏协议、尚缺和不可确认，形成有优先级的后继工作建议。
4. 冻结独立游戏仓库/固定引擎/资源清单/完整离线包的交付要求，暂不建仓或发布。

Phase 1 非目标：角色/物理/动态对象/精灵/动画/音频/HUD/游戏相机实现、native游戏验收、完整Godot/Bevy复刻、工程下载器/资产商店、迁移M1资源或引擎/AutoUI/ACE集成。不得凭调查报告宣称新公共能力已经存在。

## 2. 架构方案

研究使用三份来源：Godot官方2D Platformer（2D主参考）、Kenney Starter Kit 3D Platformer（3D首选）、Godot官方3D Platformer（3D比较/替代参考）。最多这三份，读取对应项目子树及其依赖闭包，不下载整个Godot示例集合或引入第四款交付游戏。

报告分“来源事实→M2自有规则/资源选择→现有能力证据→后继需求/验收”四层。独立工程消费同一公共引擎，Auto拥有玩法；不依赖私有Rust Host/transport、专用游戏名称分支或原Godot引擎。所有未来接口/manifest只是需求草案，明确标注未实现。

## 3. 技术栈

Phase 1：只读Git/官方源码与许可、文件哈希/资产元数据分析、Markdown/JSON研究产物。Phase 2：Rust 测试注册/feature 门控与 Cargo 定向检查。可用一次性只读小脚本解析GLB/场景依赖/大小；不新增引擎代码/依赖/锁、Cargo任务或长期通用爬取框架。大文件/参考快照保留仓外，仓内只存小型来源/目录/校验索引和研究文档。

## 4. 需求分析与背景调查

用户2026-10-27授权建立本批三份计划；本轮仅new，后续work需启动，无新agent/远端仓库/发布/推送授权。M2已确认两款自有小关卡与独立示例/按需资源方向；具体commit、资源选择与公共能力缺口正是本Plan任务，不假定已经冻结。

取材[全局概览](../specs/overview.md)、[AG概览](../specs/auto-graphics/overview.md)、[行为API](../specs/auto-graphics/behavior-api.md)、[静态资产](../specs/auto-graphics/scene-assets.md)、[源码/双游戏](../specs/auto-graphics/auto-source-driver.md)、[运动](../specs/auto-graphics/bounded-motion-profile.md)、[M2设计§5–7](../design/autographics/m2-editor-and-public-game-prototypes.md#5-游戏样本与能力牵引)。

当前交付源码854=`523d27b863dbe306c89af0ea546334225a29f8e9`，SDK=`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`；855观察`70f1addf0a12a6af7f3b9bf447428f558a779d26`未交付。公共behavior/1..5拓扑固定、平移/显隐与有限AABB查询/两接触移动；静态GLB不接受纹理/蒙皮/动画等，原完整Squash的专用能力不能直接算成公共API。

2026-10-27重读[Godot 2D README](https://github.com/godotengine/godot-demo-projects/tree/master/2d/platformer)、[Godot 3D README](https://github.com/godotengine/godot-demo-projects/tree/master/3d/platformer)、[Kenney README](https://github.com/kenneyNL/Starter-Kit-3D-Platformer)：前者范围包括斜坡/移动平台/射击，后者列有二段跳/坠落平台/相机及资源许可说明。这只是可变网页调查输入，不能代替work固定commit、逐文件许可和资源字节证据；本轮未下载或运行项目。现行Specs没有上述样本冻结结果，拟新增SD-01，明确其为研究事实/未来要求而非实现支持。

## 5. 详细设计

### 5.1 独立入口与写面

Phase 1 无855/856/857交付硬依赖，能先做只读来源/文档调查；不读取855未提交文件作为能力基线。唯一研究树`D:/autostack/.wt/lang-858/auto-lang`、`plan-858-dev`从主检出当时冻结提交创建，source对象通过Git读取854已交付引用；记录main与graphics两个身份。文档成果最终合main；Phase 1 没有需要落autographics-dev的引擎源码变更；Phase 2 在主线修复测试 feature 边界。

当前dependency-ready任务T-01。研究中855/856/857若交付，保留旧观察与新交付的时间/来源，T-04最终核对矩阵受影响行；不混淆“候选代码存在”和“已交付”。未来游戏工作仍要等所需编辑/运行合同交付，本研究pass不自动放行它们。

Phase 1 写面限本Plan及attachments、`docs/design/autographics/m2-game-sample-matrix.md`（新研究设计产物）、M2战略/专题设计必要交接。canonical新主题在review后merge沉淀；不修改既有Plan合同/完成状态、引擎源码/SDK/锁/原资产/金样，不在本轮创建正式游戏仓。

### 5.2 固定两款小关卡分母

| 样本 | 必需玩法与表现 | 不自动纳入 |
|---|---|---|
| 2D横版平台 | 一个可游玩关卡；左右运动/重力/跳跃/落地、墙顶碰撞、平台/坑洞、可收集物、一种巡逻敌人；失败/胜利/重开；跟随相机/边界、基础精灵动画/音效/HUD | 斜坡、移动平台、刚体子弹/射击、完整TileMap、手柄/联网、复杂战斗 |
| 3D第三人称平台探索 | 一个低多边形立体关卡；相机相关的行走/跳跃/落地、静态平台/危险区、收集与终点、失败/重开；可操作跟随/环绕/缩放相机、所需表现/音效/HUD | 二段跳/坠落平台/复杂斜坡/刚体、完整骨骼动画工具链、手柄/联网/开放世界 |

T-02给每条必需项稳定`G2-xx/G3-xx` ID，明确触发输入、Auto拥有的状态、预期状态/数值/视觉/声音结果、失败/恢复和未来验证入口；附一次失败重开与一次获胜重开的完整可玩流程。自有规则允许定义伤害/胜负，与参考差异逐项记录，不能把Godot原示例玩家无敌等来源事实误写成已批准的M2必需规则。

只在上述三来源选择实现/资源参考。3D优先Kenney；若资源依赖或许可不满足目标，比较Godot官方3D并给证据，在保持必需分母的范围内选定；改变必需范围或引入新游戏类型须用户决定。称“参考实现”，不称原项目完整复刻。没有清楚的素材许可/来源不能认领可发行资源已冻结。

### 5.3 来源、资源与下载盘点

T-02固定仓URL、完整commit、路径/版本、读取日期与hash索引。T-03按实际场景/模型/材质/字体/声音引用追踪**所选小关卡所需资源闭包**，逐文件记录：角色/用途、源commit/路径、原始字节SHA-256/长度、格式/依赖、许可原文/归属/NOTICE、是否需转换/裁剪及派生身份。代码和素材许可分别检查，README总体许可不能覆盖未查的第三方文件。

实测汇总：运行必需资源、可选高清/音乐、美术制作源文件；分别给总大小、最大文件、预计最小下载/完整包大小与测量方法。压缩大小未经打包只能标估算，不伪称实际ZIP字节。只按样本子树取文件，不直接clone全示例库；参考下载/解包放`D:/autostack/plan-artifacts/plan-858/`、无链接/路径越界/覆盖既有目录，保留获取失败与恢复。缺远端文件/LFS真实内容或许可未明记为未闭合，不能用指针hash代替资源hash。

据数据冻结小资源随Git/大资源包/制作源可选的分配与默认下载预算建议；本Plan不发布资源、不实现fetcher、不改原AGP资产。记录原素材到M2拟用资源的映射，未经真实转换的派生文件保持“待制作”，不能凭占位GLB宣称其材质/动画已支持。

未来manifest需求包括固定引擎/资源版本、URL或发布资产身份、长度/SHA、相对目标路径、来源/许可；完整ZIP含源码和运行资源，本地闭包可离线/搬迁。损坏/缺失拒绝与修复后启动、外部无缓存运行是后继验收需求。本次只审查需求与资源事实，未执行这些未来运行场景。

### 5.4 公共能力矩阵与后继步骤

T-04每个G需求映射到现行Spec＋固定代码符号/API/profile，分类`delivered-public / delivered-specialized / missing / unverified`，附来源证据与支持限制。建议领域至少覆盖：动作输入、角色/地面墙顶/跳跃、对象生命周期/过期句柄、触发事件/时序、公共相机、精灵/图层/动画、模型/材质、声音/HUD、Run/暂停/重开/资源释放、工程/资源闭包。

不把现有`ag_move_and_slide`当完整角色控制，不把专用Squash声音/HUD当公共服务，不把AGP有透视矩阵当公共游戏相机；不把生成文件/脚本能读当完整工程消费。每行给最小公共接口需求、失败/资源/生命周期约束与可区分反例，**需求名不等于已承诺新API签名**。

按依赖建议后继有界切片，例如角色运动可行性→公共输入/角色合同，实例化/销毁→事件/游戏服务，表现/声音/HUD→单游戏闭环；根据实际缺口调整，不预取新ID或承诺固定计划数。旧VM CPU/managed/128节点不足的疑问应列为后继真实Auto探针门，不在纯研究报告假造性能通过。若材料不足，补明确缺口/下一动作，不能降分类来凑全绿。

### 5.5 Phase 2：恢复 AV 语料测试的可选依赖边界（r2）

**目标与来源**：承接原 PLAN-859 未实施合同与 [B855 批量失败](attachments/855/batch-r1/receipt.json.txt)：`7f966bfa3d416a746b44cfe7abbde897e450e4da` 的 language listing 以 24 个 E0432/E0433 失败，0 测试执行，UI/tt/tb 未到达；上一完整绿色锚仍为 PLAN-850。855 图形交付不是该失败来源。当前 `crates/auto-lang/src/tests.rs` 中 `av_collections_822_tests` 已有 `#[cfg(feature = "test-cases")]`，`av_owned_821_tests` 和 `av_source_827_tests` 未被该条件门控；`crates/auto-lang/Cargo.toml` 的 `test-cases` 启用可选 `auto-test-case`。`637037506` 引入 821 注册，`735f01242` 插入 822 改变属性归属；827 引入链和所有共享消费者须由 T-08 核实，不能预断单一来源。

**实施范围**：仅在独立 `D:/autostack/.wt/lang-858/auto-lang`、`plan-858-dev`（Phase 1 的树已清理；新阶段建立前验证路径和分支无其他所有者）修改必要的测试注册/特性边界及必要回归，不改 AutoGraphics 源码、公共 API、原语料、基准 ID 或测试阈值。对每个需要可选依赖的消费者应用明确且独立的 feature 边界；不得通过默认开启 feature、删除测试、改过滤器或放宽断言来消红。Phase 2 为 Rust Category B；Phase 1 的 Category A 限制只适用于研究阶段，不限制这次用户新授权的代码修复。

**规范增量**：Phase 1 的 SD-01 已由 `43d1c7766c0c5b9a8f6497862a97f7107d3b817b` 沉淀，不再重复发布。Phase 2 只恢复 [auto-test-case](../specs/auto-test-case/project.md) 已有可选依赖边界，不改变公共协议/行为，故本阶段无新增 canonical Spec；若调查发现合同变化，先按流程修订并复审，不得静默扩大。

**执行身份**：当前主线以 work 启动时实际 HEAD 为准并记录；855 失败日志为历史反例，无法仅靠新 HEAD 抹去。原 `PLAN-859` 草案已移为 [未实施的历史记录](archive/859-restore-av-corpus-feature-boundaries.md)，编号不回收；本阶段只有 PLAN-858 一个执行合同。本轮用户授权计划修订，实施由后续 `/auto-plan:work` 启动。

### 规范增量（Phase 1，已交付历史）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/m2-game-sample-research.md | 无固定样本研究结果→固定来源/功能分母/资产许可与大小/现有API缺口及后继需求；明确未实现 | 为功能合同提供真实输入 | AC-02/03/04/05 |

现有engine行为无变化；overview/plans在merge登记“研究交付”，不把所需API写成已支持。新增文件内容由独立review核实后才canonical化。


### 5.6 Phase 3：ACE 开发初期周转目标统一（r3）

用户新授权 ACE 全线路从 P95<1000ms 调整到 **P90≤2000ms**：对于有效样本数 N，按 nearest-rank 取排序后第 `ceil(0.9*N)` 项（1-based），该值≤2000ms；等价于超过 2000ms 的有效样本数不超过 `floor(N/10)`。10 个有效样本中允许 1 个超限、2 个超限必失败；恰好 2000ms 不算超过。将用户的“10次”作为比例的明确例子，保留各正式测量已有的 N≥30 以免降低证据量；不可把测量次数减到 10 来缩短验证。smoke/反控用例可按其明确的非正式参数保持较小样本，不能冒称正式结果。每个请求的计时起止、真实路径、正确结果/诊断、数据身份、全部场景、原始记录及结构/资源/生命周期等非时延要求均不降级；无效、缺失、NaN/非有限或非正时间、错误结果与未执行样本不得计入绿。

T-12 首先全仓核对 ACE 主线（AC/MIR-native、AV、CTE、语义、ABI、AutoDown test-case 和调用/正式验收路径）的现行合同、实际门禁、测试 ID、样本/数据消费者及在途 823/824/828 等工作，形成逐项映射；其他独立项目（图形帧、HTTP、UI 等）的性能 SLO 不自动改变。P95/P99 可继续作为诊断及历史测量输出，不能作为仍阻断 ACE 开发期交付的隐含旧 1s 硬门。历史计划、历史审查和已有历史测量忠实保留；当前适用规范给出 supersession 与用户裁定，不回写过去的测试结果为新政策通过。后续精确性能目标待整个架构基本成形后另行测量和确定，不预先承诺数值。

当前初筛实际入口包括 `crates/auto-lang/src/tests/{ac_mn_probe_tests.rs,ac_av_probe_tests.rs,probe_adapter_tests.rs,probe_adapter_turnaround_ipc.rs}`、`experimental/{ac-av-probe,ac-cte-probe,ac-semantics,auto-abi-probe}/` 的正式周转 producer/消费者；T-12 还须扩展扫描及逐项证据，不能只修复 858 遇到的单例。先编写并复审 Phase 3 的规范增量草案，canonical Spec 由 merge 按冻结草案沉淀。已有 r2 账本投影为准备态，不构成最终 r3 规范沉淀。

### 规范增量（Phase 3，待复审）

| delta_id | 操作/目标 | 前→后及边界 | 验收 |
|---|---|---|---|
| SD-02 | modify `docs/specs/auto-ac/mir-native-cache.md`、`docs/specs/auto-av/{cache-and-conformance,resident-function-install}.md` | ACE 活跃的编辑/结果周转 P95<1s → P90≤2s，原路径和结构正确性不变；过去结果为历史 | AC-12/13 |
| SD-03 | modify `docs/specs/auto-cte/frozen-cache.md`、`docs/specs/auto-semantics/query-cache.md`、`docs/specs/auto-abi/conformance-cache.md` | 同一用户裁定覆盖 CTE/语义/ABI 正式周转门，保留真实采样和资源限制 | AC-12/13 |
| SD-04 | modify `docs/specs/auto-test-case/{profile-v2-native,architecture}.md`、`docs/specs/auto-lang/project.md`，以及 T-12 核实的其他 current Spec | 保证调用方、报告及主门禁引用与新统计口径一致；仅旧值历史事实不改写 | AC-12/13 |

## 6. 测试设计

Phase 1 文档/资产调查Category A：不运行Cargo整档、docs_gen、原生/GPU或性能基准。实施者完成调查与自查，独立上下文最终review在同一研究树复核来源/数据/覆盖，merge负责实际沉淀/归档/清理；未指定模型/新agent。

| Case ID | AC/不变量 | 验证入口/材料 | 可区别错误结论的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-02 来源/分母 | 固定commit与两款G2/G3表、参考差异/完整游玩流程 | 可从指定对象读回来源；所有必需项有ID/流程/未来oracle，未采用项不被误称完整复刻 | T-02；源阅读不是试玩证明 |
| V-02 | AC-03 资源事实 | 实际文件与scene/model依赖、SHA/size/许可/归属表 | 按字节重算SHA/长度；引用闭包无漏项、无LFS指针冒充；已转换/待制作分开 | T-03；仅选定资源闭包 |
| V-03 | AC-03/05 下载组织 | 分组总量/实测或估算标签、manifest字段、完整ZIP需求 | 汇总可重算，不把制作源打进默认运行包；一项目按需且本地可闭合；未知许可/缺文件不能绿 | T-03/04；未实现下载/运行 |
| V-04 | AC-04 现状分类 | 固定交付代码/Spec逐G行核对，特别是动态节点/纹理/相机/音效 | 至少明确区分一项通用已有、一项专用已有、一项缺失；每行实际入口/边界可查，未来函数不冒称已有 | T-04；不证明新API可行 |
| V-05 | AC-04/05 后继可用 | 依赖图与每个缺口的拟探针/正反oracle、857/856状态 | 无循环/无隐含ACE前置；所有G项被已支持或待补合同覆盖；资源/CPU疑问有下一动作，未虚报实测 | T-04；不预占实施ID |

复核可用小型只读脚本重算manifest与链接/计数，不新增通用测试框架。review逐项检查全部选中运行资产SHA/许可字段与需求映射，抽读原源码证据及第三方许可原文；数量/字段齐全不能替代语义审查。负面材料如缺依赖/未知许可须在报告明确，不能标成“下载暂缺但已发行就绪”。

### Phase 2 验证合同（V-06..08）

| ID | 不变量及实际入口 | 可区分的反例与通过条件 | 责任 |
|---|---|---|---|
| V-06 | `crates/auto-lang/src/tests.rs` 可选依赖消费者 → `auto-lang` test 编译；核对主线失败日志、定位所有共享消费者 | 默认 feature 的 `cargo test -p auto-lang --lib --no-run` 与 `--features test-book --no-run` 能编译且不再有 821/827 的未解析依赖；缺任一 cfg 的受控负例须能暴露编译失败。`cargo check -p auto-lang` 必须通过 | T-08 work |
| V-07 | `test-cases` 开启后 821/822/827 三个已有测试模块与语料 | 比对修复前后测试 ID：当前静态声明分别 3/3/8，运行时列表以冻结源码实际结果为准；具名过滤运行这三族的全部正例、负 oracle、worker fault、admission/inventory 反例，报告实际 passed/ignored/skipped；不能因 cfg 让三族在启用时消失或减少断言 | T-09 work |
| V-08 | 最终审查后的真实主线 batch：`cargo tf --batch --output <新仓外证据目录>`，官方串行入口含 language/UI/tt/tb | 保留原选中 ID、features、profile、非时延限额和报告保护；仅 ACE 周转统计/阈值按 r3 裁定改变；逐腿记录执行数与失败，未执行不记绿。失败归因并按流程处理，只有完整成功才推进批量覆盖锚；此门由 merge/auto-plan-regress 主检出单实例执行 | T-11 merge |

T-08/T-09 只跑反例和受影响测试/必要编译，不运行整档 `t/tv/tt/tb/tu/taa/tf`；`cargo t` 及由实际 diff 触发的档在最终 review pass 后由 merge 在冻结提交运行。V-08 是既有全局批量门，不得被定向通过替代。执行时确认 sibling auto-down/auto-ui 依赖身份、配置与测试数据，避免在主检出代码 WIP 上构建；记录实际命令和提交。


### Phase 3 验证合同（V-09..11）

| ID | 入口/不变量 | 可区分的结果 | 责任 |
|---|---|---|---|
| V-09 | 逐个实际 producer→报告→gate 消费者与规范，含 CTE/语义/ABI/AV/MIR native | 10 条有效值 9×≤2000+1×>2000 必过；8×≤2000+2×>2000 必红；恰好 2000 通过；30 条 27/3 通过、26/4 必红。空值、NaN/Inf、非正、缺样本或错误结果必红；旧 P95 仅诊断；真实连续窗口/结构断言保留 | T-12/13 work |
| V-10 | 原有真实场景与选中测试 ID；各入口的默认参数、CLI/报告/消费者 | 定向测试验证标准使用 P90/2000、场景与正式 N 不减少，输出同时如实记录 P90/P95；负控证明两条超限/篡改正确性/结构退化不通过；涉及的 Spec 草案和真实测试一致 | T-13 work |
| V-11 | 独立对照 T-12 全域矩阵、实际差异、spec delta 与 in-flight 冲突 | 无残存可阻断交付的旧 P95/1s 门，无无关 SLO 被改；复审绑定代码、依赖和 SD；merge 在新冻结版本跑裸 cargo t、适用触面和 V-08 | T-14 review / T-11 merge |

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 研究树/main/已交付graphics身份明确，只读来源与允许文档写面，未改引擎/在途855 |
| AC-02 | work | 两款自有小关卡必需分母、不可变参考、差异与失败/获胜重开流程可审查；V-01 |
| AC-03 | work | 选定资源闭包真实SHA/大小/依赖/许可与来源完整，派生/未知清楚，下载分组可重算；V-02/03 |
| AC-04 | work | 每个G需求有当前公共/专用/缺失/未核分类、代码/Spec证据、后继需求与反例；V-04/05 |
| AC-05 | work | 独立仓/固定引擎/资源包/离线闭包与预算建议、后继有界切片可消费；无新实现/性能/发行宣称；V-03/05 |
| AC-06 | review | 独立核work AC/V/SD、全部资源索引与许可/需求分类，遗漏/未决如实记录；非实现验收 |
| AC-07 | merge | Phase 1 文档/规范/ledger/归档与guard清理实际完成，历史7/7；未修改graphics源码或创建游戏远端 |

| AC-08 | work | 默认/full 所需的无 `test-cases` 编译、book 配置无 821/827 可选依赖错误；同族消费者逐项核对，V-06 正/反控制通过 |
| AC-09 | work | 启用 `test-cases` 时 821/822/827 原始测试 ID、语料与正/反断言完整，实际具名定向运行通过，V-07 |
| AC-10 | review | 独立复核 Phase 2 范围、feature 边界、源数据、V-06/07 和冻结提交；Phase 1 历史复审不替代本阶段复审 |
| AC-11 | merge | 最终审查后的适用 `cargo t`/触面档通过，代码合入及工作树 guard 清理；主检出完整批量 V-08 实际结果与回执如实记录，成功后才移动覆盖锚 |


| AC-12 | work | ACE 全线路现行 P95<1s 门完成逐项归属与引用盘点；正式门使用 P90≤2000ms，10 样本 9/1 与 8/2 的有效/失败边界、2000ms 等值及30样本边界真实可区分；非时延的正确性、结构和资源边界不退化；V-09/10 |
| AC-13 | review | 独立复审 r3 的代码、全域矩阵、用户裁定与规范增量；复审 Phase 2 未受影响证据并确认有效，合入门禁仍 pending；V-11 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 无功能前置 | 从冻结main建唯一研究树，核854交付与855观察身份/边界，记录资料/产物目录 | AC-01 | [x]（研究树 base `7e498e4ccbce5f0a7b7b8008d8303b170d6f373e`，分支 `plan-858-dev`；854 `523d27b863dbe306c89af0ea546334225a29f8e9`、SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`；855 已交付 `05fdc81c546aff735eefdb6c39fbc5fb35dc546a`。仓外材料 `D:/autostack/plan-artifacts/plan-858/`） |
| T-02 | work | T-01 | 固定三来源中的样本commit和所选两小关卡G表/差异/完整流程，产出新sample-matrix设计 | AC-02 | [x]（`8cb7dbcc5`：17 个 G ID、固定几何/输入/状态与失败/获胜重开，2D Godot 与 3D Kenney 来源差异；V-01 静态表/ID 检查，非试玩） |
| T-03 | work | T-02 | 仓外所选资源闭包读取/hash/许可/大小/派生状态；attachments资源manifest与分组/预算建议 | AC-03/05 | [x]（`11d9de467`：13项所选原始输入逐文件SHA/长度与固定Git blob重算；2D 149188、3D 198763、去重203825字节；四GLB色图引用闭包核查，CC0/OFL区分；派生/ZIP待制作，不宣称发行） |
| T-04 | work | T-02/03 | 每G项对应现行Spec/交付代码和缺口/后继探针，核最终交付变化；提交研究产物/SD草案/自查 | AC-01/04/05 | [x]（`a4812981e`：17项现状矩阵、四类能力/限制和后继反例、SD-01草案；854→855 API 无diff，855为已交付；文档/资源索引定向自查通过） |
| T-05 | review | T-04 | 独立来源/资源/范围/API分类与SD复审；不实施服务或代签新游戏验收 | AC-06 | [x]（独立 final R1 pass，受审 `a4812981e72df3d54ecb555dae5f77c3fabbd959`；13/13资源及4GLB闭包、17G分类、V-01..05/SD-01核对，见[复审报告](attachments/858/review-r1/review.md)；T-06/T-07 pending） |
| T-06 | merge | T-05 pass | 文档合main、知识/overview/plans/ledger/索引沉淀并归档；仅文档检查，无Cargo | AC-07 | [x]（快进 `43d1c7766c0c5b9a8f6497862a97f7107d3b817b`；canonical/overview/plans/goals、账本98→99、生成索引无语义变更；归档实际检查及提交见merge收据） |
| T-07 | merge | T-06 | guard clean后清研究树/临时分支，保留必要来源恢复路径/SHA和真实7/7；交后继研究结论 | AC-07 | [x]（归档提交 `90afcede9` 后 fresh wt-guard clean；`D:/autostack/.wt/lang-858/auto-lang`、分支 `plan-858-dev` 和空组已实际移除；固定来源可按索引恢复） |

| T-08 | work | Phase 1 已交付 | 创建并核实本阶段唯一树；追溯 821/822/827 引入链和全部同族消费者，修复 feature cfg；V-06 | AC-08 | [x]（`c20c08a0cb5f45bb64f4feb3e2a1a41d65775fc8`；默认及book no-run、`cargo check -p auto-lang` exit0；分别移除821/827门控均重现E0432/E0433且源码逐字节恢复） |
| T-09 | work | T-08 | 对 V-06/07 做定向编译、测试 ID 及真实正负例检查；提交修复和证据，整档 merge-owned pending | AC-08/09 | [x]（`c20c08a0c` 代码；`f098e8d70` 证据：功能启用清单与修复前源码 14/14 一致，821/822/827 实际 3+3+8 全通过，0 failed/ignored；格式与diff检查通过） |
| T-10 | review | T-09 | 独立复核 Phase 2 全部 AC/V、未删断言/未默认开 feature、冻结身份 | AC-10 | [x]（Phase 2 final R1 pass @f098e8d707c4774e0c5a2f7dc459c9065e515488；见[复审报告](attachments/858/review-phase2-r1/review.md)，合入门禁 pending） |
| T-12 | work | T-10 pass / 用户 r3 裁定 | 在现有唯一工作树核对主线及同组依赖；逐项盘点 ACE 性能门、producer/consumer/CLI/report、样本/测试 ID、当前规范和在途工作冲突，确定完整矩阵与 r3 规范增量草案 | AC-12 | [x]（`771557e4b929e887eb55aa688946e89140e844eb`：[盘点](D:/autostack/.wt/lang-858/auto-lang/docs/plans/attachments/858/phase3-work/inventory.md)和[SD草案](D:/autostack/.wt/lang-858/auto-lang/docs/plans/attachments/858/phase3-work/spec-delta-draft.md)，含16个规范/设计输入哈希；另有主检出六项设计WIP及823/824同文件交叉，沉淀/合入前须协调） |
| T-13 | work | T-12 | 在同一树实施所有 ACE 适用正式门 P90≤2000ms、原采样数和真实路径/结构不变；完成 V-09/10 边界/负控/定向测试及冻结证据，提交代码和规范草案，不跑整档 | AC-12 | [ ] |
| T-14 | review | T-13 | 独立复核 r3 全范围、实际统计/边界、原有正确性/样本/测试 ID、规范草案及 Phase 2 复用有效性，给出受影响复审结论 | AC-13 | [ ] |
| T-11 | merge | T-10/T-14 pass | 在最终冻结提交执行适用合入门并合 main；按 r3 冻结 SD 沉淀当前规范与 ledger；保留 Phase 1 历史快照到 attachments、修复其相对链接后以 r3 归档文件替换原历史快照路径；guard 清树；主检出单实例跑官方完整 batch，成功后更新覆盖锚 | AC-11 | [ ] |

## 9. 复审记录

2026-10-27 T-01：`git rev-parse --show-toplevel` 与分支核对通过，研究树干净；主检出有其他会话的未跟踪代码路径文件，已保留。855 已交付，不再仅是观察对象；856 executing、857 drafting，均非已交付游戏能力。Godot 子树固定 `b761b4cd5718a7118f2c95eb55f181d0a5f004f1`，Kenney 固定 `3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0`；检出仅供只读调查。T-02..T-04 待完成。

2026-10-27 new r1：stage=new；outcome=pass（研究草案可交work，非调查已完成）；next=用户启动后work/T-01。无需等855/856/857交付做只读调查；后续游戏实施仍受所需功能交付约束。本轮仅官方README/本地Spec/源码观察与文档检查，没有固定外部提交、下载资源、实施游戏/引擎或运行Cargo/native/GPU。

2026-10-27 work 续查：stage=work | plan_id=PLAN-858 | plan_revision=1 | outcome=blocked | worktree_path=D:/autostack/.wt/lang-858/auto-lang | branch=plan-858-dev | base_commit=7e498e4ccbce5f0a7b7b8008d8303b170d6f373e | code_commit=f79c1a006 | dependency_revisions=854:523d27b863dbe306c89af0ea546334225a29f8e9,855:05fdc81c546aff735eefdb6c39fbc5fb35dc546a,SDK:7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 | task_ids=T-02/T-03/T-04 进行中 | evidence=研究草稿与16项候选索引已提交；逐文件 SHA-256/长度及四个 GLB 外部图像依赖存在性复算通过；非运行闭包或许可审计；未运行 Cargo/native/GPU | blockers=Godot 2D候选美术及音效的逐文件许可/归属未证实；T-02几何/输入与 T-04逐行分类仍须定稿，T-03闭包/分组未完成 | next_responsible_role=work。

2026-10-27 work 继续：对上述许可障碍，已在允许来源内提出 Kenney CC0 模型/声音/精灵与 OFL 字体的替代候选，2D 人物/敌人帧从带 idle/walk/jump 的模型衍生（尚未制作或证明可辨识），Godot 2D仅作玩法参考；因此不要求用户决策，许可障碍有候选绕行路径但闭包、转换可行性尚未核实。仍保持 executing，T-02/03/04 未完成。

2026-10-27 work T-02：`8cb7dbcc5` 固定 Godot b761b4cd 与 Kenney 3fa8a04b 的参考身份，17 条 G 需求、规则与两类重开流程；静态 ID 检查 17/17，跳跃可达算式只是必要条件，不是试玩。T-03/T-04 仍未完成；下一角色 work，最终复审与 merge 门禁 pending。

2026-10-27 work T-03：原 Godot 素材逐项许可不明确，已在允许来源内改选 Kenney README 声明 CC0 的素材、OFL 字体，保留 Godot 为参考；源闭包证据见 `11d9de467`。模型包含外部色图/动画，原件不被现行静态 GLB profile 接受，2D 帧及未来包待制作；当前不宣称派生或离线包已完成。先前许可阻塞已通过替代选择解除，T-04 和独立 review 仍待完成。

2026-10-27 work 最终交接：stage=work | plan_id=PLAN-858 | plan_revision=1 | outcome=pass | worktree_path=D:/autostack/.wt/lang-858/auto-lang | branch=plan-858-dev | base_commit=7e498e4ccbce5f0a7b7b8008d8303b170d6f373e | code_commit=a4812981e72df3d54ecb555dae5f77c3fabbd959 | dependency_revisions=854:523d27b863dbe306c89af0ea546334225a29f8e9,855:05fdc81c546aff735eefdb6c39fbc5fb35dc546a,SDK:7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467,Godot:b761b4cd5718a7118f2c95eb55f181d0a5f004f1,Kenney:3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0 | task_ids=T-01..T-04 | evidence=17项G表及分类ID/四类能力结构核对；13项所选源文件实际SHA/长度与固定Git blob、GLB图像依赖、分组字节重算通过；854→855 API无diff，856 executing/857 drafting；选中目录reparse扫描clean；git diff --check通过且工作树干净；仅研究无Cargo/native/GPU，实际转换、运行/包及性能未宣称 | blockers=无工作阶段阻塞；未知Godot素材与FLA未入所选闭包 | next_responsible_role=review/T-05。AC-01..05为工作自查结果，须独立语义核对完整来源、各项许可和资源映射；T-06/T-07及其文档沉淀/归档/guard清理由merge负责且pending。

2026-10-27 consolidation PLAN-858:r1：stage=merge | outcome=publication pass, cleanup pending | reviewed_commit=a4812981e72df3d54ecb555dae5f77c3fabbd959 | delivery_commit=43d1c7766c0c5b9a8f6497862a97f7107d3b817b | canonical=docs/specs/auto-graphics/m2-game-sample-research.md | ledger=.autoos/specs.json version 98→99, P858-M2-GAME-RESEARCH/P858-R1/P858-MERGE/GOAL-008, guarded offline projection | archive=docs/plans/archive/858-autographics-m2-game-samples-and-api-gaps.md | completion_kind=delivered | evidence=[准备记录](attachments/858/merge-r1/prepared.md)、[账本校验](attachments/858/merge-r1/ledger-projection.json.txt)；Category A 文档/字节检查通过，不运行Cargo/docs_gen；实际归档检查和清理仍待完成。

2026-10-27 cleaned：stage=merge | plan_id=PLAN-858 | plan_revision=1 | outcome=pass | completion_kind=delivered | delivery_commit=43d1c7766c0c5b9a8f6497862a97f7107d3b817b | canonical=docs/specs/auto-graphics/m2-game-sample-research.md | ledger=.autoos/specs.json v99 | archive=docs/plans/archive/858-autographics-m2-game-samples-and-api-gaps.md | cleanup=guard clean、worktree/branch/空组实际移除 | evidence=[merge收据](attachments/858/merge-r1/receipt.md)、[账本投影](attachments/858/merge-r1/ledger-projection.json.txt) | remaining=批量回归到期判定另记；无新游戏/引擎/发布交付。archived 7/7。

2026-10-27 r2 新阶段：stage=new | plan_id=PLAN-858 | plan_revision=2 | outcome=pass | 用户明确要求取消独立859并在858追加Phase 2；Phase 1 受审/交付/归档及原7项证据保留在历史快照 | current=executing 7/11 | added=T-08..T-11, AC-08..AC-11, V-06..V-08 | next=work/T-08 | 批量仍 due/compile-blocked；本次仅计划修订，不声称修复或测试通过。

2026-10-27 Phase 2 入口：stage=work | plan_id=PLAN-858 | plan_revision=2 | worktree=D:/autostack/.wt/lang-858/auto-lang | branch=plan-858-dev | base=243da2c5ebb0a2f657a89a117885a91b17a696ed | auto-down=c3464d0a0d36e4f8a7988b0c0c26448fc52361b4（组内detached、干净） | auto-ui=b4fe0000e56cddaf7d3cf70e348431f2258aacde（主检出干净） | T-08/T-09进行中。855 的24个未解析错误在 [language-build.log.txt](attachments/855/batch-r1/language-build.log.txt)；失败基线与当前受影响测试源码、Cargo配置和锁无diff，可作为预修复反例。源码追溯：821 `63703750697f95c61897f8e7e64a9fa721f2e2de`、827 `e1b91b4cc3204945d8d48fff5a6c6488541ff6cb`、822 `735f012423c84af55fcef803bc7ac166d37ea2cd`。主要同族直接消费者按模块逐一核对，a2r/vm 的 pilot 调用各自由 test-cases 函数级cfg保护；修复前尚需完成实际默认/book编译、feature启用14项和负控制。仅最终review后由merge跑整档。

2026-10-27 Phase 2 依赖路径补齐：首次 `cargo test -p auto-lang --lib --no-run` 在读取 `auto-ui-compiler` manifest 前因组内 `../auto-ui` 缺失退出101，未执行测试；已在同组补建只读 detached auto-ui `b4fe0000e56cddaf7d3cf70e348431f2258aacde`，与前述主检出身份一致、干净，无链接。原因已变化，后续可重试；保留原日志 `D:/autostack/plan-artifacts/plan-858/phase2-work/default-no-run.log`。

2026-10-27 Phase 2 磁盘保护：补齐 auto-ui 后，`cargo test -p auto-lang --lib --no-run` 默认配置实际 exit0（4m33s；原日志 `D:/autostack/plan-artifacts/plan-858/phase2-work/default-no-run-r2.log`）；遗留依赖编译 warning（auto-lang-core 146、auto-ui-compiler 40、auto-lang lib-test 41）保留日志，不当作新增问题掩盖。多会话并发写盘后 D 盘不足 400MB；本次独立新建的 plan858 target 经路径/reparse核验后移除以恢复空间，源码与日志未动。后续 Cargo 仍从唯一计划工作树调用，`CARGO_TARGET_DIR=C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/target` 为独立 C 盘产物目录，未建链接；`CARGO_INCREMENTAL=0`、dev/test debug=0、jobs=4，构建设置绑定后续证据。

2026-10-27 Phase 2 T-08：修复提交 `c20c08a0cb5f45bb64f4feb3e2a1a41d65775fc8`，仅 `crates/auto-lang/src/tests.rs` 的两个模块新增各自 `test-cases` cfg，修正原共享注释。827 原先具 cfg（`e1b91b4cc`），821 插入 `637037506` 时占用该属性，822 插入 `735f01242` 时又占用它；按模块扫描所有直接 TestCase/AV probe 消费者，a2r/vm 聚合各自有函数级 cfg，其余相关模块已门控。默认 `cargo test -p auto-lang --lib --no-run`、`--features test-book --no-run`、`cargo check -p auto-lang` 均 exit0；分别撤销 821/827 单个 cfg 的 book 编译均 exit101 且对应源文件 E0432/E0433，恢复源码 SHA256 `04b3c25e521ffd731f3b8000f730a295e448bf7e58da606594223b74b9acffb6`。完整日志及负控结果在 `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/`，默认正例原始日志在 `D:/autostack/plan-artifacts/plan-858/phase2-work/default-no-run-r2.log`。既有库 warning 保留原日志；T-09 的启用配置实际测试及 ID 对照尚待完成。

2026-10-27 Phase 2 work 交接：stage=work | plan_id=PLAN-858 | plan_revision=2 | outcome=pass | worktree_path=D:/autostack/.wt/lang-858/auto-lang | branch=plan-858-dev | base_commit=243da2c5ebb0a2f657a89a117885a91b17a696ed | code_commit=c20c08a0cb5f45bb64f4feb3e2a1a41d65775fc8 | handoff_commit=f098e8d707c4774e0c5a2f7dc459c9065e515488（其后仅证据文档） | dependency_revisions=auto-down:c3464d0a0d36e4f8a7988b0c0c26448fc52361b4,auto-ui:b4fe0000e56cddaf7d3cf70e348431f2258aacde,AV-worker-SHA256:2f5c3df14101fff3f5902a7fe5364b3e2113992e5fc765d97dbe76dafd4cc501 | task_ids=T-08/T-09 | evidence=V-06：默认及book测试no-run、cargo check exit0，逐个移除821/827 cfg分别E0432/E0433且恢复；V-07：未启用book二进制无三组ID，启用时实际14/14 ID等于修复前测试源码（3/3/8），逐组真实运行14 passed、0 failed/ignored；rustfmt单文件及git diff --check通过。完整记录和日志哈希见工作树 [phase2-work/summary.md](D:/autostack/.wt/lang-858/auto-lang/docs/plans/attachments/858/phase2-work/summary.md)（交付树中，合入后须改为主检出相对路径），原日志保存在 C 盘项目产物目录（默认正例在 D 盘 plan-artifacts），构建输出使用独立C盘target。代码与依赖/锁/配置未改动；原855历史失败日志作为预修复输入，原Phase1验收不变。 | blockers=无工作阶段阻塞；无已知未解决的合同内finding/覆盖缺口 | pending=独立最终review T-10；合并阶段冻结提交裸cargo t/适用触面档与主检出单实例完整批量 V-08仍待执行，不可宣称回归通过或已合入 | next_responsible_role=review/T-10。


2026-10-27 Phase 2 final R1：stage=review | review_scope=final | reviewer_identity=Codex 当前独立检查上下文，未参与本阶段实施、未另开agent，模型/会话ID未暴露 | plan_id=PLAN-858 | plan_revision=2 | outcome=pass | worktree_path=D:/autostack/.wt/lang-858/auto-lang | branch=plan-858-dev | reviewed_commit=f098e8d707c4774e0c5a2f7dc459c9065e515488 | code_commit=c20c08a0cb5f45bb64f4feb3e2a1a41d65775fc8 | base_commit=243da2c5ebb0a2f657a89a117885a91b17a696ed | dependency_revisions=auto-down:c3464d0a0d36e4f8a7988b0c0c26448fc52361b4,auto-ui:b4fe0000e56cddaf7d3cf70e348431f2258aacde,worker:2f5c3df14101fff3f5902a7fe5364b3e2113992e5fc765d97dbe76dafd4cc501 | spec_inputs=project/architecture SHA 见报告 | acceptance_results=AC-08/09/10 pass,V-06/07 pass；独立book no-run exit0、14/14实际定向测试全过；11项工作日志hash核对、默认/check及独立cfg撤销负控证据有效；源/锁/配置/语料未改 | findings=无阻塞项，无新债务 | evidence=[Phase2 R1](attachments/858/review-phase2-r1/review.md)、[冻结SD](attachments/858/review-phase2-r1/spec-delta-frozen.txt) SHA256=6ce45beea0cc49c26cd729de97312f55d82b674df45c406cfaaab4efda8b9cee | pending_closeout_gates=AC-11/T-11/V-08，merge冻结提交裸cargo t及主检出单实例官方完整batch，历史迁存/链接/归档/guard清理；批量覆盖锚仍850，未合入 | next=merge/T-11。reviewed10/11；Phase1历史不变，不声称全局回归或交付完成。

2026-10-27 Phase 2 merge attempt R1：stage=merge | plan_id=PLAN-858 | plan_revision=2 | outcome=gate_failed，未合入 | reviewed_commit=f098e8d707c4774e0c5a2f7dc459c9065e515488 | rebased_tip=ee91830f9aadb6434af84509d6ae343a75b8f55f（两次 rebase range-diff 均全等） | cargo t 三次均未完成：首次 83 pass/20 timeout（辅助程序冷编译）；第二次 895 pass/1 fail/4 timeout，原生限额失败在预构建后定向复核通过；第三次 904 pass/1 fail，signature-rebuild P95 1245.5ms，门限 <1000ms，其余 2288 未运行。测试中存在其他工作树构建；性能失败与负载的因果尚未确认。完整证据及 hashes 见 [merge attempt](attachments/858/phase2-merge-attempt/receipt.md)。所有辅助程序已按原测试命令预构建，未改测试/阈值。共享机器其他任务仍在运行；待空闲后复核性能项并在最终冻结提交重跑裸 cargo t，确认 main 基线后才可 ff 合入。规范账本投影仅在计划树提交、未发布到 main；官方完整 batch、覆盖锚、归档与 guard 清理均 pending，保留计划树供恢复。

2026-10-27 new r3：stage=new | plan_id=PLAN-858 | plan_revision=3 | outcome=pass | 用户明确授予 ACE 开发初期由 P95<1000ms 改为 P90≤2000ms（10 次最多 1 次超过）的范围；沿用 858、保留现有树和历史阶段，新增 T-12..14、AC-12/13、V-09..11、SD-02..04，T-11 等待 r3 复审。比例解释与 formal 样本数保留见 §5.6；现有正确性、资源、结构门不撤销。r2 复审只覆盖原代码与无 SD，不是 r3 新口径的复审；之前 cargo t 失败记录有效，未经复审不得直接合入。next=work/T-12。

2026-10-27 r3 work/T-12：stage=work | plan_id=PLAN-858 | plan_revision=3 | outcome=inventory-pass，implementation ongoing | worktree=D:/autostack/.wt/lang-858/auto-lang | branch=plan-858-dev | baseline=97ad76691578957b9c35f62dfbc72eb4bda83660 | inventory_commit=771557e4b929e887eb55aa688946e89140e844eb | deps=auto-down:c3464d0a0d36e4f8a7988b0c0c26448fc52361b4,auto-ui:b4fe0000e56cddaf7d3cf70e348431f2258aacde | evidence=ACE producer/consumer、CLI、报告及现行规范逐族盘点；各正式样本数及非时延门保持；同组依赖无旧目标直接命中。主检出 design 39/40/43/44 和两份 strategy 文件带其他未提交修改，823/824 在途树有同文件变更，禁止覆盖；合入前重新比对并协调。 | pending=T-13 实施/完整定向验证，T-14 独立复审，T-11 冻结门禁及沉淀。next=work/T-13。

## 10. 待澄清事项

无阻塞new草案的问题。T-02/03负责在限定来源与既定必需分母内选资源/冻结来源，常规判断无需重复批准。若需要删必需功能、换游戏类型、改变独立仓/离线交付目标，或将未知许可资产算入可发行闭包，须先提出具体选择由用户决定；本Plan不能以“研究”名义静默变更M2目标。

2026-10-27 work 未闭合：官方项目整体 MIT 声明及 Godot 贡献指南允许多种美术许可不足以确定候选文件各自许可。已追溯 2023 美术重制提交 c17d803a1becb2b0fac47fa87d3572486fd77131 及 2020 音效引入提交 e323bead2ecee69c13cd26895042917985532916，但提交作者不等于逐文件权利人/许可。下一步需取得所选文件原始权利人及具体许可证据，或在允许的三来源内选择可核实许可的替代素材，重新核对引用闭包与候选关卡，不得把未知许可标作可发行。已完成的候选 SHA/长度仅供追溯；T-02/03/04 保持未完成。

2026-10-27 交接补充：此前 Godot 素材许可阻塞已通过选择同一允许来源 Kenney 的 CC0 模型/精灵/声音及带 OFL 许可的字体解除；原 Godot 素材与许可未核的制作源不计入发行闭包。派生帧、最终游戏源码与 ZIP 尚未存在，后继工作须制作并验收，本研究不声称其已完成。

2026-10-27 独立 final review R1：stage=review | review_scope=final | reviewer_identity=Codex 当前独立检查上下文（未参与本次实现；环境未暴露模型/会话ID，未另开agent） | plan_id=PLAN-858 | plan_revision=1 | outcome=pass | worktree_path=D:/autostack/.wt/lang-858/auto-lang | branch=plan-858-dev | reviewed_commit=a4812981e72df3d54ecb555dae5f77c3fabbd959 | base_commit=7e498e4ccbce5f0a7b7b8008d8303b170d6f373e | dependency_revisions=854:523d27b863dbe306c89af0ea546334225a29f8e9,855:05fdc81c546aff735eefdb6c39fbc5fb35dc546a,SDK:7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467,Godot:b761b4cd5718a7118f2c95eb55f181d0a5f004f1,Kenney:3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0 | spec_inputs=[baseline.json](attachments/858/review-r1/baseline.json) 中11个实际Spec输入SHA | acceptance_results=AC-01..06 pass,AC-07 pending merge；V-01..05 pass；全部13项源输入SHA/长度/固定blob、4GLB外部闭包/动画、分组总数、17项G语义分类及后继需求独立复核 | findings=无阻塞finding；派生帧/真实Auto预算/正式包仍为后继风险，未核Godot/FLA排除；原始字体两处保留名措辞与MIT/CC0通知来源精度见报告 | evidence=[R1报告](attachments/858/review-r1/review.md)、[逐文件重放](attachments/858/review-r1/source-replay.json)、[可重放检查](attachments/858/review-r1/check_sources.py)、[冻结SD](attachments/858/review-r1/spec-delta-frozen.md) SHA256=9b2ca30398ff0533bde7df4c0f633314e107c4d561cfee0c21edd54a827e8a9a | pending_closeout_gates=T-06文档/链接检查与实际合入/规范-ledger-index/归档、T-07实际guard-clean/清树分支与7/7；Category A不运行Cargo或docs_gen，不存在待补Cargo整档 | next=merge。状态reviewed5/7；仅研究验收通过，未宣称M2游戏/新API/离线包完成，未合入归档或清树。

2026-10-27 batch due-check：上一成功覆盖850，855落地后到期；855批量已在编译/list阶段 exit101（B855-FEATURE-01），当时跟进 [PLAN-859](archive/859-restore-av-corpus-feature-boundaries.md) 为 drafting（历史状态，现已由 Phase 2 承接）。失败基线 `7f966bfa3d416a746b44cfe7abbde897e450e4da` 至当前代码、Cargo配置与锁无diff；无新条件不重复同一失败，Category A不跑Cargo/docs_gen。批量维持due/compile-blocked，待 Phase 2 修复后主检出单实例重跑；不影响858研究交付和归档。
