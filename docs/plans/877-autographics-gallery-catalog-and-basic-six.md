---
plan_id: PLAN-877
status: execution_done
feature_name: AutoGraphics Gallery目录与基础六项示例（GY-A）
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-28

plan_revision: 1
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-graphics/gallery-basic-six.md
touched_goals: [GOAL-008]
affects: [auto-graphics, ag-examples]
depends_on: [PLAN-861]
current_step: 6
total_steps: 10
---

# [PLAN-877] Gallery目录与基础六项示例（GY-A）

## 0. 变更摘要

在独立本地ag-examples仓建设轻量目录和G01..G06六项示例，以861已交付AG版本为固定外部消费者；可与874/875、[878边界审计](878-autographics-platform-boundary-and-dependency-audit.md)并行。仅消费原公开入口，不修改引擎、不依赖未来3D角色/运行相机，不在本轮实施。

## 1. 目标

- 六个逻辑ID、十个必需子入口均可独立启动，真实展示已有图形/脚本或编辑能力。
- 目录可按类别/维度/能力检索，给出源码、操作、限制、平台与固定引擎版本；未验收条目不能标Ready。
- 公共CLI启动、资源/来源校验、单项完整ZIP、离线搬迁重开成立，用户不需下载语言/引擎全仓或后续游戏。

非目标：G07..12、874/875新API、正式游戏、浏览器在线引擎、Vue/DOM/移动适配、引擎迁仓、ACE/HIR、UI融合、公开远端建仓/推送/网站部署、资源商店/下载器/包管理器。静态catalog仅为本地浏览/预览/索引，不添加新的产品GUI或专用Rust宿主。

## 2. 架构方案

ag-examples拥有示例Auto/scene/资源/说明、元数据与通用目录/校验/启动/打包工具；AG仅为外部固定运行依赖，示例正常玩法由Auto或公共编辑操作产生。工具传已校验参数给公共CLI，不包含角色/碰撞/拾取算法，不调用私有Host或探针命令实现正常示例。

测试可观测公开SDK/inspect输出和真实OS/GPU结果，但“引擎测试通过/截图存在”不替代仓外项目可用性。已有研究/编辑能力按类别展示，不能都标为公共Auto脚本能力。

## 3. 技术栈

小型Auto/scene4项目、JSON元数据、Python标准库工具/测试与静态HTML索引；无需Node/Vue开发服务器或新增Rust组件。拟新增ag-examples路径：`catalog/index.json`、`engine-lock.json`、`examples/g01-shapes/`..`examples/g06-character2d/`、`tools/gallery.py`、`tests/test_gallery.py`、`licenses/`、`previews/`。具体slug与清单在T-02冻结，均为新路径。

既有引擎路径（autographics-dev）：`experimental/autographics-auto-driver/{README.md,samples/,tools/package_consumer.py,tools/verify_source.py,tools/prepare_sdk.py,src/bin/authoring/cli.rs}`；二进制`autographics-authoring`的`--open`/`--behavior`/`--behavior-api`与`--inspect`已观察，参数细节由T-02再核。不得照抄不存在的命令。

## 4. 需求分析与背景调查

授权：用户要求将GY-A正式规划、暂不实施。未来收到work指令后可在本计划定义的本地仓/树实施；本轮不初始化仓库/建树/构建/制作内容。没有远端创建/推送/部署授权。

依据[全局overview](../specs/overview.md)、[AG overview](../specs/auto-graphics/overview.md)、[scene](../specs/auto-graphics/scene-document.md)、[hierarchy](../specs/auto-graphics/scene-hierarchy.md)、[assets](../specs/auto-graphics/scene-assets.md)、[behavior](../specs/auto-graphics/behavior-api.md)、[segment](../specs/auto-graphics/segment-query-profile.md)、[sweep](../specs/auto-graphics/swept-aabb-profile.md)、[2D角色](../specs/auto-graphics/game-actions-and-character2d.md)与[Gallery设计](../design/autographics/m2-gallery-and-capability-matrix.md)。

前置861 archived 9/9，AG冻结提交`8b12632d7b5b1b0df72fbb4eef84297a054de31d`；874/875未交付不是本计划阻塞。SDK逻辑pin`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`，历史对象不可达时沿用861/860已验证的681文件/17本地包SDK副本；已观察保留路径`D:/autostack/plan-artifacts/plan-860/consumer-package/experimental/autographics-auto-driver/.sdk-source`，入场再次内容验证。不能改用当前语言SDK伪称同基线。

package_consumer按身份清单复制；861已披露导出包可能遗漏栈配置及非identity夹具。T-02必须以已冻结Git blob/SDK清单逐文件补足消费闭包，登记额外文件的原路径/hash/理由，不修改引擎工具/旧identity，也不借生产者target。主机auto-ui已拆分与本计划无依赖；未来迁仓不改变本次固定依赖。规划指纹见[本批基线](attachments/877/planning/baseline.md)。

## 5. 详细设计

### 5.1 仓库/工作树与并行写面

目标本地仓`D:/autostack/ag-examples`（规划时不存在）。work T-01先核不覆盖已有目录/不接管其他任务；若仍不存在，建立本地Git锚点/空初始提交，再在`D:/autostack/.wt/lang-877/ag-examples`创建唯一实现树、分支`codex/plan-877-dev`，内容/工具都在该树写。锚点只做Git初始化，不在主检出堆WIP。未来发现已存在仓库，核归属/基线后记录，冲突不覆盖。

Plan簿记在auto-lang主检出；AG源码通过冻结Git导出/保留SDK组成只读消费包置于该Plan本地artifacts，**不创建第二个可写引擎实现树**。实现、自查、独立最终review同一ag-examples树顺序进行，review时停止写入；不得junction/symlink。合入仅本地ag-examples默认分支，auto-lang只沉淀知识/归档，无agp/driver/auto-ui/SDK/Cargo.lock写面。

可与874/875及878并行，不能共用其target、SDK准备目录、报告目录或窗口焦点；native/GPU/重型验收按机器单实例排程。不追AG最新分支、不把未来迁仓后的包无声替换进engine-lock。若冻结引擎缺少必要功能，记录上游缺口并回new，不在本Plan修引擎或删示例验收。

### 5.2 引擎锁与公开启动

T-02先冻结`attachments/877/work/gallery-contract.md`：十个子入口/源hash、精确CLI、清单schema、资源来源、补齐闭包、runner构建/身份与内容oracle。`engine-lock.json`描述repo/完整commit、SDK内容锁、两workspace锁、平台/profile、受控构建配置/runner SHA；只记录可搬迁身份，用户用显式参数指定引擎路径，不存生产者绝对路径。

若需构建固定消费包，仅在其自身目录`cargo build --locked --offline --bin autographics-authoring`并保留原32MiB主线程/16MiB worker配置。SDK用`prepare_sdk.py --verify-only`和source verifier重核；额外夹具记录补齐清单，无法证明来源时阻塞，不能更新引擎源码/SDK/依赖来过门。构建成功不是示例通过。

新CLI合同（T-03建立）：`python tools/gallery.py list`、`validate --all`、`catalog --output <new-directory>`、`launch <Gxx/subentry> --engine <path>`、`package <Gxx> --output <new-zip>`；参数可等价局部实现，但T-02/README必须一致。正常launch只用已观察公共入口，如`autographics-authoring --open <scene>`（绑定源码）或显式`--behavior ... --behavior-api ...`，用户通过公开Run/编辑按钮交互；`--inspect`仅作校验，不用自检专用游戏探针代替运行。

工具拒绝重复ID/子入口、未知版本/字段、越界/绝对资源路径、链接、缺文件/hash错、错误引擎身份/未知profile；正例可独立运行。包装用参数数组而非shell拼接，场景名/路径不执行命令。catalog明确Ready/Blocked/Research与“未测平台”；Ready依赖完成的对应子入口验收记录、不是手填布尔。浏览器预览不标WebGPU在线运行。

验收运行只控制自己启动的PID/窗口，T-02冻结启动/操作/关闭期限；超时/提前退出/未完成oracle均失败，不用`--frames`观察退出或工具自报绿替代完成。关闭/异常时回收自己的子进程和输出读取者，不按全局窗口标题杀其他agent程序；保留exit/实际完成阶段与源码/runner身份，恢复后正例再通过。

### 5.3 六项分母（十个必需子入口）

| ID/子入口 | 内容/难度与来源 | 公开通过结果与边界 |
|---|---|---|
| G01/main | 基本quad/circle、alpha/层序、显隐；低；现有scene/behavior-v1 | 改颜色/位置实际改变合成内容；不新增纹理/旋转writer |
| G02/main | 一个已有许可小型平色GLB；低；现有assets/move样例 | 实际模型各可见部分、坏hash拒绝、搬迁重开；固定取景，不冒称运行orbit |
| G03/2d、G03/3d | 同维父子组平移/显隐；低中；authoring_hierarchy/behavior | 两维分别验证父变化→后代实际变化、显隐继承、Stop恢复；不混维父子/通用TRS |
| G04/2d、G04/3d | 编辑导航/grid/选中/平移吸附/保存重开；中；856/857 | 真实OS不同view下选到同一对象，保存位置重开一致；明确编辑教程，不称Auto pointer API |
| G05/2d、G05/3d | 有限segment/sweep最近命中/无命中/接触对照；中；beam2d/sight3d/dash2d/dash3d | v4公开脚本可使用两类查询，各维真实几何/最终显示与独立slab/扫掠结果相符；非无限ray/mesh精确拾取/完整物理 |
| G06/layout-a、G06/layout-b | 861单2D角色两布局＋speed4→5源码对照；中 | 真实OS动作、松手/跳边沿/空中不能二跳、地墙顶/最终支撑与Stop；不加相机/完整玩法 |

六逻辑项不按十个子入口充数；十个子入口均通过才GY-A完成。G01/G02先提供素材，G03复用；G04/G05可独立编写；G06只消费861、不等874/875。每项README含目的、能力ID、操作、支持/拒绝、实际平台/profile、默认/修改对照与恢复方式。元数据与source/scene/所选API一致；正常用户代码不含transport、保留名或每游戏Rust分支。

### 5.4 资源、目录与独立包

小必需资源随Git，优先已有明确许可的小模型/自制简单几何，不引入高清素材或外部下载服务。内容hash/长度/来源/许可逐文件记录，原始输入与派生结果分别记身份；两游戏只列规划状态/链接，不复制大工程或制造已发布URL。资源/源码单文件遵守5MB钩子，未盘点前不承诺总包大小或提高引擎预算。

每个Gxx单项ZIP包含该项所有子入口、Auto/scene/小资源/README/许可/engine-lock引用；共享资源打包时实体化到项目支持布局，不依赖仓外路径/缓存/链接。引擎固定包可另行取得，离线验收先显式准备该包及单项ZIP，再在新目录A运行/编辑/保存，搬到B后重开；禁止再访问生产者目录/网络。源/资源篡改拒绝，修复后同入口恢复可用。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/gallery-basic-six.md | Gallery仅设计→仓外目录/六项十入口、固定依赖、资源/公开启动/包及实测限制 | 公共消费知识 | AC-01..05 |

本轮不改canonical Specs，work产出SD草案，review核实后merge发布；ag-examples README/锁/契约是内容交付，不能替代引擎权威Spec或扩大旧profile。

## 6. 测试设计

用户未指定模型/新agent。work自查后同树独立最终review，pass后merge做冻结完整内容门；不因测试内容需要另开实现树。新工具/测试T-03建立。

| Case ID | AC/不变量 | 实际入口/输入 | 区别错误实现的oracle | work任务/覆盖 |
|---|---|---|---|---|
| V-01 | AC-01/02 身份/目录 | gallery validate/launch；正确与错engine/SDK/源hash、重复/漏入口、Ready伪造 | 合法项目通过；坏身份/partial/0selected/未知平台通过声明拒绝；目录实际消费同一元数据 | T-02/03；当前Windows固定版本 |
| V-02 | AC-03 G01..03 | 公共CLI/inspect、真实Auto/scene编辑、实际合成截图 | 颜色/层序/完整模型/父子显隐位置对照；漏mesh/冻结源/错误alpha能由选定内容oracle判红 | T-04；三项4子入口 |
| V-03 | AC-04 G04 | 公开编辑窗口OS操作、导航/DPI/吸附/取消/保存A→B重开 | 命中对象ID及接受位置、实际像素/持久内容一致；只画grid不完成操作不能绿 | T-05；两维编辑教程 |
| V-04 | AC-04 G05/06 | 真实Auto公开API/SDK数值观察＋OS窗口 | 独立最近体/接触、无命中、角色运动学/地墙顶/无二跳；更改源speed与布局有真实差异，冻结输入对照能被识别 | T-05/06；四子入口，非私有测试玩法 |
| V-05 | AC-05 资源/包 | package/validate解包、错hash/缺资源/越界路径/链接、离线A→B及修复 | 拒绝前不写越界/不启动错误包；合法完整包独立运行、编辑保存、搬迁重开；修复后再用成功 | T-03/06；六单项包，非自动下载器 |

定向`python -m unittest discover -s tests -p 'test_gallery.py'`、`python tools/gallery.py validate --all`和每项T-02定义的公开CLI/实际OS内容检查。自动控制可用于验收操作，不成为正常用户启动依赖；原始像素/数值/命令与exe/source绑定，未知不可测项明确报告。负控改真实输入/复制项目，恢复正例绿，不能仅改expected。无必要引擎修改，因此不跑根cargo t/tf/tv/tu/docs_gen，不重复全引擎测试；冻结引擎历史证据只用于其未变部分，不能替代新十入口运行。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 唯一内容树/861固定依赖、SDK/两锁/补齐包来源闭合，无引擎/auto-ui写入 |
| AC-02 | work | 元数据/检索/校验/启动契约先冻结，状态不虚报，V-01正负成立 |
| AC-03 | work | G01..03四子入口的公共源码/场景、真实数值与完整内容成立，V-02 |
| AC-04 | work | G04..06六子入口真实OS/编辑/查询/角色及修改/恢复对照成立，V-03/04 |
| AC-05 | work | 六单项完整包、来源许可/坏包拒绝/修复、离线搬迁，V-05；说明不声称Web或未来能力 |
| AC-06 | review | 独立逐项核十入口/AC/V/SD、实际public路径/内容/遗漏、并行写面；merge门pending |
| AC-07 | merge | 最终提交完整catalog/测试/打包及十入口当前有效结果，复用证据有内容/依赖/配置/输入未变依据 |
| AC-08 | merge | 内容仅落ag-examples本地分支，SD/ledger/索引/归档/guard清理实绩，无远端发布 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 861交付（已满足） | 核新仓归属/建立本地锚点与唯一ag-examples树；核AG/SDK/锁/并行目录边界 | AC-01 | [✅] |
| T-02 | work | T-01 | 核冻结消费包/补齐来源与构建、gallery-contract/十入口/CLI/元数据/资源/内容expected先冻结；不可行回new | AC-01/02 | [✅] |
| T-03 | work | T-02 | 新catalog/lock/gallery.py/结构负控/六包机制与说明，V-01/05 | AC-02/05 | [✅] |
| T-04 | work | T-03 | G01..03四子入口源码/场景/资产/说明及真实内容对照V-02 | AC-03 | [✅] |
| T-05 | work | T-03 | G04..06六子入口、OS/数值/内容/源码编辑与恢复V-03/04 | AC-04 | [✅] |
| T-06 | work | T-04/05 | 六单项包离线A→B/坏包修复、许可/状态/真实负控收口，提交/SD草案/摘要 | AC-01..05 | [✅] |
| T-07 | review | T-06 | 同树独立最终review十入口/工具/包/写面与原始证据，merge门pending | AC-06 | [ ] |
| T-08 | merge | T-07 pass | 冻结完整内容门/必要重放，失败回work后受影响复审 | AC-07 | [ ] |
| T-09 | merge | T-08 | ag-examples本地合入；语言主检出SD/overview/plans/ledger/索引与归档 | AC-08 | [ ] |
| T-10 | merge | T-09 | guard clean后清唯一树/临时分支/组目录，保留必要本地证据与实际10/10收据 | AC-08 | [ ] |

## 9. 复审记录

2026-10-28 new r1：`stage: new | plan_id: PLAN-877 | plan_revision: 1 | outcome: pass | next: 等用户启动work后T-01/02`。861前置已交付；新本地仓由T-01建立，冻结包/公开CLI与资源由T-02先核。全部T/AC/V/SD未执行，本轮仅规划，无新仓/工作树/运行证据/实现/远端动作。874/875/878共享源码零写面。

2026-10-28 work T-01/T-02：`stage: work | plan_id: PLAN-877 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-877/ag-examples | branch: codex/plan-877-dev | base_commit: 4df37bd（锚点root commit） | code_commit: （T-03起在ag-examples树产生） | dependency_revisions: AG=8b12632d7（12 blob逐项复核一致）、SDK=860保留副本681文件verify PASS、两锁=47176cc0/bc7287b5 | task_ids: T-01,T-02 | evidence: (T-01)D:/autostack/ag-examples原不存在→git init master+空锚点4df37bd，worktree D:/autostack/.wt/lang-877/ag-examples分支codex/plan-877-dev（rev-parse toplevel/branch核过）；主检出WIP仅docs簿记；874(在途WIP)/876在途、878未建树，零写面交叉。(T-02)[gallery-contract r1](attachments/877/work/gallery-contract.md)冻结：十子入口/CLI/--inspect与--report语义/探针仅验收用/元数据schema/状态规则/engine-lock schema/打包与A→B协议/窗口纪律。消费包D:/autostack/plan-artifacts/plan-877/consumer-package（本地证据）：443 identity文件自冻结blob导出+681 SDK副本+5补齐件（plan861-identity.json按在案freeze_plan860规则确定性重生成[861原件未提交已随worktree丢失]、.cargo/config.toml逐字节、3个861先例夹具）；包内prepare_sdk --verify-only PASS 681、verify_source PASS（identity 443+metadata 479包+AGP锁保留）；cargo build --locked --offline runner SHA256=498ce814…，--inspect样例冒烟exit 0。资产=two-parts.glb（sha256 42c9da67…，PLAN-838自制MIT，byte_length 1528实测）。合同关键机制事实（源码核对）：schema/4强制+旧schema升级、探针均于--open后运行本项目、segment/sweep探针独立slab对账但钉marker公式（G05沿用beam2d/dash2d约定）、viewport/hierarchy探针按Document::sample布局校准（G04场景沿用该布局+自有教程内容，理由已在合同登记）、--behavior仅绑定不自动play、无探针有界运行passed=true出报告 | blockers: 无 | next_responsible_role: T-03（ag-examples树实施工具/目录/负控）

## 10. 待澄清事项

远端所有者/URL不影响本地交付，留未来发布计划，本Plan不建远端。本地目标若已被占、必需SDK无法验证或固定引擎不支持本分母，具体记录阻塞后回new，不扩范围修上游。用户当前仅授权规划，禁止立即开始T-01。本计划不决定AG迁仓；未来归属变化由独立迁仓合同处理，固定消费pin保持。

2026-10-28 work T-03..T-06 收口：`stage: work | plan_id: PLAN-877 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-877/ag-examples | branch: codex/plan-877-dev | base_commit: 4df37bd | code_commit: 017b959（6 提交） | dependency_revisions: AG 8b12632d7/SDK 860副本/两锁不变、runner 498ce814 | task_ids: T-03,T-04,T-05,T-06 | evidence: (T-03)catalog/index.json+engine-lock.json+tools/gallery.py+tests/test_gallery.py 28用例（V-01 结构负控：重复ID/未知字段/逃逸路径/绝对路径/symlink/缺场景/资产哈希/缺许可/Ready无证据/坏engine-lock/引擎身份拒绝/argv构造/打包往返与manifest）全绿；示例元数据+README 六例落盘。(T-04/05)十子入口内容与验收全部成立：G01 像素级 oracle（5 捕获两两不同+alpha混色+层序翻转+显隐+位移，12断言）；G02 GLB 双部件像素+inspect重开一致+篡改拒绝（5断言）；G03 inspect world 链式数值(±1e-3)+显隐捕获差异+Stop恢复+层级探针16相位（12断言）；G04 真实OS探针：g04/2d 项目场景 20相位、g04/3d 默认文档 34步（含save→reopen等值+play-restore）+partial负控+双冒烟（9断言）；G05 segment/sweep 探针独立slab对账 2D+3D（spatial_phase 14、segment-trace、changed源哈希差异、partial负控，18断言）；G06 角色探针 10相位×2布局（真实OS键盘、跳峰≈2.002、blur清held、Stop恢复）+布局几何文档差异+speed4→5位移1.333/1.667（10断言）。全套66断言在冻结提交复跑绿。实施发现与合同修订见[gallery-contract §9](attachments/877/work/gallery-contract.md)（2D层序=alpha升序+数组序、G03 Stop恢复=探针play-restore、G04 3D探针默认文档语义、G05探针id6/5参照+update流程、G06对照构成）；验收工具tools/verify_examples.py（--window-helper 只操作自启窗口+环境性竞争≤2次有界重试）。(T-06)六单项ZIP（过滤目录+tools+acceptance+许可+实体化资产）离线A→B搬迁 validate+inspect 哈希恒等、g02/g05 资产字节篡改引擎拒绝、修复后可用（tools/verify_packages.py）全过；六例状态全 ready 且真仓 validate --all PASS。SD草案[attachments/877/work/sd-gallery-basic-six.md](attachments/877/work/sd-gallery-basic-six.md)。ag-examples 树 status clean | blockers: 无work内阻塞 | pending_closeout_gates（merge-owned T-07..T-10，不因本记录视为完成）: 独立最终review（T-07）、冻结完整内容门/必要重放（T-08）、ag-examples本地合入+语言主检出SD/overview/plans/ledger/索引与归档（T-09）、guard clean清树/分支/组目录（T-10） | next_responsible_role: 独立最终review（T-07）
