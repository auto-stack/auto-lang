---
plan_id: PLAN-880
status: reviewed
feature_name: AutoGraphics Gallery公共3D角色与两维运行相机（GY-B）
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 1
supersedes_spec_components:
  - docs/specs/auto-graphics/gallery-basic-six.md
new_spec_components:
  - docs/specs/auto-graphics/gallery-character-and-cameras.md
touched_goals: [GOAL-008]
affects: [auto-graphics, ag-examples]
depends_on: [PLAN-874, PLAN-875, PLAN-877]
current_step: 5
total_steps: 10
---

# [PLAN-880] Gallery公共3D角色与两维运行相机（GY-B）

## 0. 变更摘要

在既有ag-examples仓增加G07三维角色和G08两维运行相机，共两逻辑项、四个必需子入口。正常启动只消费875已交付公共CLI/API；工具升级为按示例选择固定引擎，保持GY-A的861引擎与旧证据有效。与[879纯迁仓](879-autographics-source-move-to-auto-ui.md)并行，不等待或追随迁仓分支。

## 1. 目标

- G07/layout-a、G07/layout-b：固定方向基的公共3D角色，两布局真实Auto跑跳/归一/有限碰撞/支撑，以及源码speed4→5对照。
- G08/2d：公开跟随/夹界/等比内容区；G08/3d：环绕/缩放、同tick相机相对运动与共同发布；各有已交付第二配置对照。
- 目录共8个逻辑项/14个必需子入口，按项锁版本、公开launch、资源/身份校验、两个新增单项完整离线ZIP；旧六项/十入口保持各自原pin。

非目标：G09..12、正式游戏、模型视觉朝向/纹理/动画/声音/HUD、补引擎功能/测试钩子、迁仓/更新SDK、浏览器在线运行/移动、远端发布/网站部署。G07仍用静态占位外形，不认领完整G3-01；G08仅已交付的单角色单相机有限服务。

## 2. 架构方案

ag-examples持有小型Auto/scene项目、说明/元数据、通用工具和验收适配；AG作为固定只读依赖。正常玩法由Auto，目录工具只校验、传CLI参数，不嵌角色/碰撞/相机算法。

现有根`engine-lock.json`保持861身份；新增G07/G08显式指向875锁。按项锁解析同时用于validate/launch/package/acceptance/catalog，避免出现“显示875、实际用861”或旧六项被无声升级。最终M2统一版本仍归GY-C，不在本Plan拼接成单一发行声明。

## 3. 技术栈

既有Python标准库工具`tools/{gallery.py,verify_examples.py,verify_packages.py}`、`tests/test_gallery.py`、JSON元数据/静态HTML、小型scene4/Auto源码。新路径拟为`engine-locks/plan875.json`、`examples/g07-character3d/`、`examples/g08-runtime-cameras/`、`checks/gyb/`（仅验收适配，独立workspace，不是正常游戏Host）。精确schema/CLI/命名在T-02冻结。

875公开选择值已核：`--behavior-api character3d|camera2d|camera3d`（旧值保持），`--open`、`--behavior`、`--inspect`。正常`--open`不自动Run；`--frames/--report`无输入不证明玩法执行，`--inspect`只证明加载/静态渲染，不证明运行相机。不得为验收调用OS自动化或用私有生产接口补正常启动。

## 4. 需求分析与背景调查

授权：用户2026-10-29同意规划“纯迁仓＋GY-B”，本轮只规划，不在ag-examples实施/建树、构建、OS/GPU运行、新建agent或发布。依赖874/875/877均已交付，879不是前置。

权威输入：`docs/specs/overview.md`、`auto-graphics/{overview,game-character3d,runtime-game-cameras,gallery-basic-six,scene-document,behavior-project,scene-play-session}.md`；[Gallery矩阵](../design/autographics/m2-gallery-and-capability-matrix.md)、[875合同](attachments/875/work/runtime-camera-contract.md)与[877合同](attachments/877/work/gallery-contract.md)作来源补充，已交付Spec优先。

ag-examples master `6a223d9b3637ee1ce9a00335c8ee4f46d3832b26`，clean；现有gallery.py全局`load_engine_lock`只读根文件，VALID_APIS不含3D/相机，validate_acceptance只比全局runner，package复制单根锁；必须明确升级这些共同消费者，不只增加文件。G01..06固定AG `8b12632d7b5b1b0df72fbb4eef84297a054de31d`，runner SHA `498ce81437d35526f964f264c466ed767dfe3ebfa4c33275e4e39ff1f0338181`；禁止改旧锁/acceptance为新runner来凑绿。

新增内容固定消费`1cc68aba012f76951af2556763a5fb21be6a6c8b`（875最终交付，含874），SDK逻辑pin `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`、681文件/17包及两AG锁保持。旧SDK对象不可达，沿用受检保留副本；消费包自产生并有逐文件来源/补齐收据。875清理曾重建exe，哈希与旧测量exe不同；新包实测runner SHA在T-02冻结，不能抄旧报告sha冒充本次runner。875 Git树缺plan875-identity.json，保留consumer有原件，导出须先证明身份可复原（见[本轮基线](attachments/879/planning/baseline.md)）。

可用的无窗口验收路径已观察：875 `tests/character3d_samples.rs`通过公开Storage/SourceSnapshot/Session对实际布局运行Auto；`tests/runtime_camera_host.rs`通过公开Host/Play及SurfaceContent验证位置/矩阵/letterbox/Stop。这些是验收适配参考，不是新的普通用户启动入口。实际仓外项目无窗口运行/捕获可行性尚未执行，由T-02整路径先验，失败不得改私有引擎或自动恢复OS探针。

## 5. 详细设计

### 5.1 唯一树与固定输入

仅建`D:/autostack/.wt/lang-880/ag-examples`，分支`codex/plan-880-dev`，从核实后的ag-examples已提交master创建并记录SHA；不重新初始化仓库。语言主检出只Plan簿记。AG从875固定对象导出到`D:/autostack/plan-artifacts/plan-880/consumer-package`，旧861消费包另置独立目录；不建第二可写引擎树、不读取879迁移WIP、不共用target/SDK准备/报告。共享引擎/SDK/auto-ui只读。

T-01复核owner/当前base/旧六项与锁；T-02先冻结`attachments/880/work/gallery-contract.md`：四入口/源hash/每项锁schema/真实CLI/runner身份/离线构建栈与补齐件/期限/无窗口oracle及证据复用清单。whole-path探针至少用G07实际布局与G08两维源码贯通公共loader→真实Auto→Host发布→无窗口GPU捕获；支持不了则保留反例回new，不能改分母/硬界或Rust算玩法。

### 5.2 按项锁的共同合同

旧G01..06未声明新lock时继续解析根`engine-lock.json`，历史schema/1兼容。G07/G08明确选择`engine-locks/plan875.json`；锁路径只能受控root内相对常规文件，拒绝绝对/逃逸/链接/未知schema/错身份。具体版本字段T-02冻结，不把尚未实现schema冒称已可用。旧根锁内容保持逐字节不变。

每条实际launch/验收记录绑定所选repo/完整commit/两锁/SDK锁/runner SHA/项目源码与场景hash；目录显示每项实际engine/platform和证据层次，不能全站统一写latest或875。engine实参需匹配本项锁：用861 runner启动G08、用875 runner冒充旧G06均应拒绝，再换正确runner恢复。

validate_acceptance必须从本项锁和实际源集合判定，空oracles/漏必需oracle/0selected/错误子入口或旧hash不算Ready；新项目改源后原记录失效。旧六项采用原schema/记录，不伪补历史字段；为其引入严格新字段需显式schema升级和受影响验证，禁止不经核对重写旧证据。共同工具的旧输入兼容采用既有临时fixture正反例，零外部runner也能验证路由正确。

package只包含本逻辑项的全部子入口、说明/许可/资源、所需锁与通用工具/验收记录（按实际布局），新包即使不含旧六项/根旧锁也能解析/validate/launch；若保留旧根锁作兼容，也不得错误选择它。选中锁缺失/错runner/篡改清单拒绝，不启动错误引擎。catalog筛选、launch、package三者选择一致。

### 5.3 两项四入口与内容分母

| ID | 固定公共profile/内容来源 | 必需判别与边界 |
|---|---|---|
| G07/layout-a | character3d/1；875树内samples/character3d/layout_a+main/main-fast按字节消费 | 同输入四向/对角归一、z墙/最终支撑、跳边沿；speed4→5实际位移改变；固定水平基，非模型朝向动画 |
| G07/layout-b | character3d/1；对应layout_b与同main | 同Auto/输入跨平台与缺口布局路径实际分歧，明确有限接触包络，不把不同scene hash当玩法差异 |
| G08/2d | runtime_camera2d/1；samples/runtime-camera/two/layout_a+main/main-alt | x2/15/28夹界为6/15/24，800×380内容区样板570×380、两侧115；第二源码配置改变投影；实际角色/地标图像变化 |
| G08/3d | runtime_camera3d/1；samples/runtime-camera/three/layout_a+main/main-alt | 候选yaw90°同dispatch的W世界方向转90°、zoom/pitch限域、角色/相机同tick、第二投影配置可区分；暂停/Stop恢复与原位置一致 |

新项目优先直接消费已交付Auto/scene字节，路径/文档/元数据由内容仓拥有。binding角色/solid、四向speed4/重力20/jump8/clearance.002及相机域保持已交付语义；Enter只动作，不宣称重开游戏。样板≤128节点（宽/稀疏）、密集深度64，32credits、原SDK源/步数/时限不变。本Plan不做新128负载性能承诺，也不重跑五进程整矩阵。

正常用户入口：`python tools/gallery.py launch g07/layout-a --engine <正确runner>`等，公开`--open`并绑定准确API，用户通过Run及键鼠交互；README列实际操作/支持拒绝/源码修改对照/恢复/平台、引擎获取方式。正常launch不加任何probe参数，不依赖checks/适配。单项包收到后可离线运行、编辑保存、搬迁重开，固定引擎单独准备。

### 5.4 验收适配与OS证据边界

仅验收的新`checks/gyb/`可用独立Rust workspace依赖本Plan只读消费包的公开driver/AGP，测试时读**实际G07/G08文件**，从公开Storage/SourceSnapshot/Session或Host驱动固定逻辑输入；不改引擎/tests、不访问private模块、不发OS事件。build配置/依赖锁/源码hash在记录中绑定，防不同程序报告混用。它是测试oracle和公共消费者，不进入正常玩法、不包含碰撞/相机业务实现；独立公式只用于expected。

必须新鲜观测实际Auto位置、Host发布camera、无窗口GPU内容；复用已有capture/离屏组合入口，不自行写第二个renderer。有限输入序列至少覆盖正常/相抵/对角/跳与90°基/两个相机配置；paused/neutral/Stop在同Host路径核实。控制可直接喂公开逻辑输入，这不等于重新证明OS输入路由。

OS自动化继续暂停。既有874/875真OS证据仅用于未改引擎/router及字节相同样板的能力来源，注明旧runner/项目身份；本次仓外包/路径/元数据/新runner必须新验，不能把历史窗口截图贴新项目当新验收。README与acceptance分别列fresh headless consumer、历史OS复用、未测平台及全量里程碑pending。若必需的仓外OS环节无法合理复用，则该环节保持pending并回new请用户决定，不能私自执行OS探针或将pending写Ready。

负控至少改真实输入/复制项目：冻结移动输入导致G07不动、改speed后位移应变、替成世界轴源使90°门红、错误锁/partial/旧报告/0selected拒绝、漏相机投影或固定捕获由图像/独立投影门检出。只改expected不算产品敏感性；对不可安全变异的私有发布机制不强行改引擎，已有原子发布反例按未变证据复用。

### 5.5 离线包与并行交付

两个新增ZIP分别包含G07两布局或G08两维，不依赖仓外资源/共享cache/链接。独立引擎包从875固定输入构建，仅其第三方registry缓存是明确前置；SDK与必要夹具完整。解到A→validate/无窗口实际运行/保存→搬B→重开语义和hash一致；错hash/缺件/坏lock拒绝，修复后绿。真实交互说明与自动验收能力分开记录。

879即使先交付也不改变此Plan输入；以后GY-C再统一新归属的最终发行基线。主语言检出SD/ledger/归档与879串行，内容仅合ag-examples本地master，无远端动作。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/gallery-character-and-cameras.md | 六项→新增G07/G08四入口、固定875/证据层次/两包与限制 | 仓外公开消费 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-graphics/gallery-basic-six.md | 全局锁→旧六项保原pin，新项显式选锁；旧验收语义保持 | 共用工具兼容 | AC-02/05 |

## 6. 测试设计

无指定模型/新agent；work自查后同内容树独立最终review，再merge冻结门。所有下列case本轮未执行。遵守变更相关∩约1/10可复现抽样（seed/全集/样本在案）；四个新增入口的直接验收分母不因随机抽样删减，重复/完整回归留里程碑。OS自动化暂停；非侵入无窗口GPU按机器单实例，进程只回收自启PID，超时/无报告/提前退出必失败，不杀全局窗口。

| Case | AC/不变量 | 实际入口/输入 | 区别错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01/02 来源/锁路由 | gallery validate/launch/package/catalog，861/875双锁+错runner/逃逸/未知API | G01..06旧pin不变，G07/08三API正确路由；旧/新版本互冒拒绝；源码变更使新Ready失效 | T-02/03；非统一发行 |
| V-02 | AC-03 G07 | 实际两布局Auto→公开loader/Session→Host捕获 | speed4→5位移敏感、同输入布局落地/坠坑分歧、归一speed不增、z墙/支撑；冻结输入负控红 | T-04；固定方向基，无动画 |
| V-03 | AC-04 G08 | 实际两维源码/alt→公开Host→无窗口捕获 | 独立clamp/letterbox值与像素；yaw90同tick位置/地标投影，世界轴源/固定图像负控红；Pause/Stop保持 | T-04；OS路由只复用未变证据 |
| V-04 | AC-02/05 证据真实性 | 新acceptance原始记录/source/config/runner，缺oracle/partial/0selected/错hash | 真运行全required green才Ready；记录新旧来源/未测项，旧报告不可冒新项目 | T-03/05；不是重做引擎协议 |
| V-05 | AC-05 包闭环 | 两ZIP，独立A→B+坏资源/锁/缺件，正确runner | 解包后无需旧六项/root仓即可选对锁和实际运行；受控拒绝/修复/重开一致 | T-05；引擎单独固定包 |

基础命令：`python -m unittest discover -s tests -p test_gallery.py`（受影响小型结构面）、`python tools/gallery.py validate --all`、`list/catalog/launch/package`新参数在T-02冻结；验收适配`cargo test --locked --offline --manifest-path checks/gyb/Cargo.toml --test gallery_runtime -- --test-threads=1`（新T-04建立，锁/显式engine-root路径配置T-02定，禁止临时修改引擎）。无窗口`autographics-authoring --inspect <项目>`只作加载/保存重开佐证。原有工具仅扩展共同功能/选择范围，不默认启动g04/05/06探针；既有OS包装器不进入本计划自动验证。

两引擎构建仅各自consumer中，32MiB PE/16MiB worker栈保持，准备/verify先行。引擎源码零改动，不跑根cargo t/tf/tv/tu/docs_gen，不全量AG/旧Gallery窗口套件；必要旧六项无窗口抽样要记录所选项与未变复用依据。原始报告/图像/验收临时件本地`D:/autostack/plan-artifacts/plan-880/`，attachments仅md摘要/契约，产品所需小预览与机器验收索引放ag-examples，绝不搬大量原始证据入库。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 唯一内容树/875 fixed package/SDK与新runner来源闭合，无引擎/879写入 |
| AC-02 | work | 按项锁贯穿所有消费者，旧六项保持、坏身份/证据拒绝，V-01/04 |
| AC-03 | work | G07两布局真实Auto/无窗口内容/输入源码负控与历史OS复用边界成立，V-02 |
| AC-04 | work | G08两维真实跟随/投影/相对运动/共同发布与恢复、内容负控成立，V-03 |
| AC-05 | work | 四新入口/两包/目录总8项14入口、离线A→B坏包修复，平台和证据层次诚实，V-04/05 |
| AC-06 | review | 独立核四入口/共用工具/旧pin/AC/V/SD、证据复用与无OS越界，merge门pending |
| AC-07 | merge | 冻结内容/输入与受影响抽样/复用审计通过，旧新选择一致，不冒称全量通过 |
| AC-08 | merge | 仅本地ag-examples落内容，语言Spec/ledger/索引/归档与guard清理实绩 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际操作与产出 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 用户启动；874/875/877已交付 | 核既有repo/owner/唯一树、旧十入口/根锁指纹、新875输入/SDK准备边界 | AC-01/02 | [x] |
| T-02 | work | T-01 | gallery-contract先冻结四入口/锁/CLI/补齐包/新runner/无窗口整路径可行性/OS证据边界；不足回new | AC-01..05 | [x] |
| T-03 | work | T-02通过 | 按项锁/三API、validate/launch/package/catalog/acceptance兼容与结构反例V-01/04 | AC-02/05 | [x] |
| T-04 | work | T-03 | G07/G08工程/说明、仅验收public适配与真实Auto/无窗口图像、V-02/03 | AC-03/04 | [x] |
| T-05 | work | T-04 | 两包离线A→B/坏包修复、旧项无窗口抽样、Ready依据/SD/提交/摘要 | AC-01..05 | [x] |
| T-06 | review | T-05 | 独立同树核四新入口/共同工具兼容/来源/SD，不接管实现 | AC-06 | [ ] |
| T-07 | merge | T-06 pass | 冻结受影响抽样/未变证据复用审计，失败回work受影响复审 | AC-07 | [ ] |
| T-08 | merge | T-07 | 内容本地合ag-examples master，engine-lock两代并列、无远端发布 | AC-08 | [ ] |
| T-09 | merge | T-08 | 主语言检出SD/overview/plans/ledger/索引与归档，同879串行 | AC-08 | [ ] |
| T-10 | merge | T-09 | wt-guard clean后清内容树/临时分支/空组目录，保必要本地产物/收据 | AC-08 | [ ] |

## 9. 复审记录

2026-10-29 new r1：`stage: new | plan_id: PLAN-880 | plan_revision: 1 | outcome: pass | next: 用户启动work后T-01→T-02`。874/875/877前置已交付，879非前置；T-02首门核实当前包缺identity的补齐、按项锁细节与新仓外无窗口验收的整路径可行性。旧OS证据是否适用于原字节样板须逐项映射；不能用未测覆盖换Ready。仅规划，无ag-examples写入/建树/编译/窗口/GPU/新agent/远端操作，全部AC/V/T/SD未执行。

2026-10-29 new文档预检：0..10节、10个唯一T、8个唯一AC、5个V、stage/依赖与SD覆盖已核；两Plan及四份路线文档共90个本地链接可解析、879/880编号各唯一、中央next-id=881。这是合同结构/来源/并行写面自查，不是独立final review或实现验收。旧SDK681文件与875固定source-lock逐项hash一致，仅作规划输入；无Cargo/OS/GPU运行。

2026-10-29 work T-01/T-02：`stage: work | plan_id: PLAN-880 | plan_revision: 1 | outcome: pass(T-01/T-02) | worktree_path: D:/autostack/.wt/lang-880/ag-examples | branch: codex/plan-880-dev | base_commit: 6a223d9b3637ee1ce9a00335c8ee4f46d3832b26（ag-examples master，clean） | dependency_revisions: AG=git 1cc68aba012f76951af2556763a5fb21be6a6c8b（autographics-dev 历史可达）→ 导出 consumer-package 1384文件（内容703+SDK681）+ plan880-consumer-identity.json 逐文件来源清单 ALL CHECKS PASS；SDK .sdk-source=plan-875保留包副本 prepare_sdk --verify-only PASS（681文件/17包/UI-Man旧包不存在）；两Cargo.lock blob=47176cc0/bc7287b5 与根锁一致 | task_ids: T-01,T-02 | evidence: (T-01)根锁指纹与计划§4逐字节一致；**身份偏差裁定**：875保留包12文件（api/runtime_camera2d_v1.at、authoring host/cli/probes族、tests/runtime_camera_api.rs、tools/{check_runtime_camera,package_consumer,verify_source}.py）为无Git来源第三态（≠身份≠git@1cc68aba0≠auto-ui@4adf2d0，保留包=R1前工作树快照）→ 按"无来源件不可借用"880全量消费git@1cc68aba0态，12件不消费、27个motion-evidence取git新态，代价由T-02探针+T-04全验收新鲜闭合。(T-02)[合同冻结](attachments/880/work/gallery-contract.md)：四入口源hash（10文件sha256）、每项锁schema（engine-locks/plan875.json + example.json engine.lock按项指向；根861锁逐字节不动）、真实CLI、runner=df0a8077eebf03bc29729cd786af0829fea20679a2a0d8b6a94fe1ac28af0130（debug 86,127,616字节，--locked --offline实测exit 0）、离线栈（registry缓存=明确前置）、负控矩阵、证据复用清单。**无窗口整路径先验VIABLE**：wholepath-probe 13/13 PASS exit 0——公共loader→真实Auto→Host发布→rt.capture无窗口GPU PNG全通；G07两布局真跑（a贴地z=-4.0、b坠坑y=-5.5、speed4→5位移比1.25）、G08/2d夹界(6,1)/follow∈[6,24]/双letterbox bar 115×380/PNG 800×380、G08/3d orbit dist8/pitch45°/perspective/相机相对位移；栈护栏=64MiB线程（875既有）、camera pitch单位=度。探针源码/输出本地：plan-artifacts/plan-880/wholepath-probe*/ | blockers: 无work内阻塞 | next_responsible_role: T-03（按项锁/三API工具升级+V-01/04结构反例）。

2026-10-29 work T-03/T-04/T-05（execution_done 交接）：`stage: work | plan_id: PLAN-880 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-880/ag-examples | branch: codex/plan-880-dev | base_commit: 6a223d9b3637ee1ce9a00335c8ee4f46d3832b26 | code_commit: ba4cb7d（T-03 工具）+ 21ae99b（T-04 工程/适配）+ c36c412（T-05 身份修正） | dependency_revisions: AG=1cc68aba0（consumer-package 1385 文件含 .cargo/config.toml blob 992c9e1e）、runner=6c461c35125d75f957a79421ce53fb37ac2c6eef47e7d107f6fa791c350d6b59、identity manifest=6f100c8831fec918127b12a801a27042c445ca57e20c096fbc95132d4c0f44a6 | task_ids: T-03,T-04,T-05 | evidence: (T-03)gallery.py 按项锁路由/VALID_APIS+character3d-camera2d-camera3d/严格 acceptance 源绑定；结构测试 47 OK（新增 15 项：双锁路由、跨锁拒绝、strict 源漂移/空 oracle/陈记录、锁路径逃逸/绝对/未知 schema/坏身份、launch 跨锁引擎拒绝、独立包无根锁 validate、catalog 双引擎）；真仓 validate 回归 6/10/1 零破坏。(T-04)G07/G08 四子入口字节消费+README+catalog 8ID/14 入口/C07-C08；checks/gyb 无窗口适配（独立 workspace 由 verify_gyb.py 物化、显式 engine-root、64MiB 线程护栏）；**新鲜验收 32/32 oracle 绿**（G07：导入/真跑/贴地/冻结输入不动/speed4→5 敏感/对角 0.7071 归一/x 墙 5.498 接触包络/跳边沿一次；G08/2d：tick0 夹界(6,1)/follow∈[6,24]/同 tick 共发布/letterbox 115×380+alt 146.67/暂停不动/PNG 800×380；G08/3d：orbit 8-45-persp 源内读/独立 glam VP 恒等/同 tick 共发/yaw90 世界转 +x/像素随 orbit 变/暂停缩放丢弃/zoom 封顶/Stop 文档复原/重启原位/alt ortho 配置+VP/世界轴负控可区分）；PNG 落 out/gyb/artifacts（本地）。(T-05)**身份修正**：首构建 df0a8077 缺根 .cargo/config.toml（32MiB PE 栈）主线程 --inspect 溢出 → 补 blob 992c9e1e 重建 6c461c35、清单 1385 文件 ALL CHECKS PASS、验收重钉；8/8 单项包离线 A→B 重定位+tamper/repair 全绿（g07/g08 加零引擎源漂移负控）；引擎包重定位 A/B inspect 双绿；旧六项无窗口抽样=8 包门全 scene --inspect（861 runner 498ce814 原样）+内容字节未变复用 877 验收记录。**分母**：validate --all PASS 8 逻辑项/14 子入口/2 锁；未跑根 cargo t/tf/tv/tu/docs_gen（引擎源码零改动）；OS 自动化零调用；AG 全量套件=里程碑档 pending 归 merge 记录。门禁 pending（merge-owned）：受影响抽样/复用审计（T-07）。 | blockers: 无 | next_responsible_role: review（T-06 独立同树终审）。

2026-10-29 review r1（final pass → reviewed）：`stage: review | review_scope: final | reviewer_identity: 独立子代理（未参与实现的独立上下文，同机同树只读；独立性=程序性的——全部门禁新鲜重执行、不信任执行方输出，单机环境因素共享） | plan_id: PLAN-880 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-880/ag-examples | branch: codex/plan-880-dev | reviewed_commit: c36c412 | base_commit: 6a223d9b3637ee1ce9a00335c8ee4f46d3832b26 | dependency_revisions: AG=1cc68aba0、runner=6c461c35125d75f957a79421ce53fb37ac2c6eef47e7d107f6fa791c350d6b59、manifest=6f100c8831fec918127b12a801a27042c445ca57e20c096fbc95132d4c0f44a6 | spec_inputs: docs/specs/auto-graphics/{gallery-basic-six.md 在库, gallery-character-and-cameras.md 待merge新增} + Plan SD-01/SD-02 delta 表核一致（六→八、旧pin字节不动、按项锁、C07/C08） | acceptance_results: AC-01..05 全 pass（独立重执行：manifest ALL CHECKS PASS、双 runner sha 三方匹配、跨锁 launch 双拒 rc=2、结构测试 47 OK/1 skipped、verify_gyb --check-only 32/32 新鲜匹配、12 源文件字节恒等、validate --all 8/14/2、verify_packages 8/8 PASS、OS 自动化 grep 干净、适配器 import 面严格公开子集、无同义反复 oracle）；AC-06 本审通过 | pending_closeout_gates: T-07 冻结抽样/复用审计、AG 全量套件里程碑档、批回归到期判定、语言仓 spec/ledger/归档（merge-owned，显式未跑） | findings: F-R1-01（minor/note-only：计划 T-03 证据写"新增15项"实为19个测试函数，47 OK 实质claim已核，merge 簿记时更正口径）；F-R2-01（minor/note-only：计划§6字面命令引用 checks/gyb/Cargo.toml，实际入口=tools/verify_gyb.py 物化 out/gyb/ws（与冻结合同§7一致），merge 文档引用实际入口）——均不阻断、无需返工 | evidence: 复审代理自跑命令与rc见其报告；worktree HEAD c36c412 状态clean | next: merge（T-07..T-10）。

## 10. 待澄清事项

无未决产品决策。work 已交付（T-01..T-05，execution_done）：唯一内容树消费 git@1cc68aba0 固定输入，runner 6c461c35（含 32MiB PE 栈配置），四入口 32/32 无窗口 oracle 绿，8/8 包 A→B 门绿，validate 8ID/14 入口/2 锁。T-02 的"OS 暂停下公开消费路径不足须回 new"情形未触发——整路径探针一次贯通。merge-owned 门禁 pending：T-07 冻结抽样/复用审计、AG 全量套件里程碑档、批回归到期判定。无需远端地址；本 Plan 不创建远端。既有大录屏/原始证据不迁入产品仓；新增 PNG/报告证据均在本地 plan-artifacts 与包 out/（gitignored），attachments 只入 md 合同/摘要。
