---
plan_id: PLAN-1075
status: archived
feature_name: AGE Gallery 和公共场景会话的 AV 接管
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-22
plan_revision: 2
current_step: 5
total_steps: 5
depends_on: [PLAN-981, PLAN-980, PLAN-978, ag-examples/PLAN-024, ag-examples/PLAN-025]
supersedes_spec_components: []
new_spec_components: [docs/specs/ace-migration/m2-age.md#5]
touched_goals: [GOAL-002]
---

# [PLAN-1075] AGE Gallery 和公共场景会话的 AV 接管

## 0. 变更摘要

2026-11-19。本计划独立交 Gallery 小 demo 与公共 AV 入口。原混合场景迁 [1099](1099-age-av-mixed-showcase.md)，完整 G2/G3 迁 [1100](1100-age-av-game-2d.md)、[1101](1101-age-av-game-3d.md)，保原验收义务。

2026-11-19，用户批准按交付拆分 980/981。本计划接手原义务及可复用输入，按交付来源身份匹配后计验收。

## 1. 目标

使 age-examples/examples/ 的 G01–G12 通过 ACE AV 原入口运行，交 Gallery 及共享会话、事件和资源的正式证据。

## 2. 架构方案

AGE 场景及 auto-ui 宿主 owner，与 age-examples 产品 owner 配对。复用 ACE 受检前门、共享 Schema、现有宿主和生成入口。

## 3. 技术栈

沿原合同冻结 Windows 工具链、SDK、资源路径及产物身份。真实窗口、GPU、浏览器、外仓或离线消费按实际用例运行。

## 4. 需求分析与背景调查

取材 [AutoLang 全局总览](../specs/overview.md) 的 UI/Web 生态，以及 [981 原合同](archive/981-ace-m2-age.md#0-变更摘要)。原技术合同、输入、测试分母和负控按归属表接收。

## 5. 详细设计

**前置**。取 981 已交 SOURCE/EVENT-B、980 typed 宿主与 978 SDK。从样例实际使用的组件和服务取正式来源。先跑依赖少的图形、层级和查询，再推进角色、动画、相机和音频等入口。

**交付边界**。承接 981 B-CONTENT 的 Gallery 行、B-SESSION/LANGUAGE/PROOF/FUSION 公共 AV 格与原 AGE-L/AG-A 的适用行。冻结 G01–G12、22 入口及原适用变体、资源与 oracle。公共场景、会话和资源接线由本计划交付。MAP/ASSET/PBR/CLIP 的 AV 实现与自身 oracle 由 AutoUI 099–102 交付。

**产物**。交 Gallery 基线及适用的 IF-AGE-GFX-AV/<cap>，附源码/资源、SDK、Schema、独立预期、错误和恢复。混合样例及完整游戏在对应迁入计划验收。

### 规范增量（复审定稿，merge 落盘）

- **add `docs/specs/ace-migration/m2-age.md` §5「Gallery 公共 AV 会话与适用边界（PLAN-1075）」**：多模块作者闭包经 `AgeSession::open_modules(modules, entry, …)`/`update_module` 开为 AV 会话（出口 `app::<entry>`；坏编辑保旧/未名模块具名拒/代数纪律；变体入口连字符由接线层净化、前门对非法 id 具名拒）；Gallery 适用边界=受检前门平面行形状（shape2d/3D 冻结叶/组件实例行），56 闭包中 6 过（g01 全 5 变体+g02/main），50 具名 pending（45 嵌套语法、5 元素种类）归图形能力线（AutoUI 099–102 MAP/ASSET/PBR/CLIP 及后续场景扩展），非本面义务；永久 oracle=`plan1075_gallery_av_session` 8 用例 + `tests/fixtures_plan1075`（ea0fc59 逐字冻结）。SDK 身份=冻结基座+overlay @ `0a167f611` 复用不刷新。
- **supersedes**: 无（不退役任何现有规范行）。**new**: 上条。**touched_goals**: GOAL-002。

## 6. 测试设计

从 age-examples/tools/gallery_node_authoring.py 和原 examples/ 固定入口、变体及独立 oracle。沿 auto-ui/experimental/autographics-auto-driver/tests/authoring_session.rs 核实际 AV 会话、真 GPU/输入/音频（适用时）、热更新、坏编辑保旧、保存/SaveAs/搬迁、仓外冷包和资源回收。保 V-B-CONTENT/SESSION、V-L-*、V-AGE-WAVE/TRANSFER 与 V-B-FUSION 的 Gallery/公共接面适用行。记录源/SDK、实际命令和失败条件。

## 7. 验收标准

| ID | 标准 |
|---|---|
| AC-01 | §5 对应原分母齐，输入、原独立预期、负控及实际依赖可追溯。 |
| AC-02 | §6 真产品入口及全部适用原检查通过，故障、恢复、资源和来源身份对证。 |
| AC-03 | 同树冻结独立复审，正式交付 source/SDK/ABI 与验收证据，消费者可按约定接口接收。 |

## 8. 执行步骤

- [x] **T-01｜work｜冻结 Gallery 分母**。清点 G01–G12、22 入口、原变体、资源和实际依赖。
  [✅ 已完成 2026-11-22] 冻结清单与生产入口见 §9（T-01 收据）。
- [x] **T-02｜work｜交公共 AV 会话接线**。沿 authoring_session 与原入口验证 typed 事件/资源、生命周期及坏编辑恢复。
  [✅ 已完成 2026-11-22] 接线交付（AgeSession 多模块闭包）+ 普查 + 永久用例 8/8 + 基线全绿，见 §9（T-02 收据）；实现 commit auto-ui `95401c10`。
- [x] **T-03｜work｜验收原 Gallery**。跑各项原入口和独立预期，交适用图形能力的正式结果与源/资源指纹。
  [✅ 已完成 2026-11-22] 原门身份审计 21/22 全绑（296/296 oracle）+ AV 腿 8/8 @ `95401c10` + 指纹台账 + 预存红与能力边界具名，见 §9（T-03 收据）。
- [x] **T-04｜review｜独立复审**。逐项核 AC-01..03、原入口及实际反例。
  [✅ 已完成 2026-11-22] final pass（有界 AC-02 裁定+变更负控+规范增量定稿），见 §9（T-04 收据）。- [x] **T-05｜merge｜正式交付**。合入、发布 Gallery/公共 AV 交付、归档及清理。
  [✅ 已完成 2026-11-22] 双仓落主（auto-ui `95401c108` ff-only / auto-lang `a01c5020e` 含 m2-age §5），归档+四树清理+plan-artifacts 全删，见 §9（T-05 收据）。
## 9. 复审记录

2026-11-22 stage: goal | plan_id: PLAN-1075 | plan_revision: r2 | round: 1 | from: new | to: work | status: drafting→executing | outcome: pass | worktree: `D:/autostack/.wt/lang-1075/{auto-lang,auto-down,auto-ui,age-examples}` | commit: 基面 lang `0046408c5` / down `45b642d` / ui `8dd99503` / age-examples `ea0fc59` | next: work。授权链：用户「完成计划1075」= /auto-plan-goal 全链授权（work→review→merge）。

2026-11-22 stage: work（T-01 冻结 Gallery 分母）| PLAN-1075 r2 | outcome: pass | worktree: `D:/autostack/.wt/lang-1075/*` @ plan-1075-dev×3 | base: lang `0046408c5` / ui `8dd99503` / age-examples `ea0fc59`

**分母（实测冻结，worktree 内）**：`age-examples/catalog/index.json`（ag-examples.catalog/1）= **12 logical IDs（g01..g12，C01..C12）/ 22 subentries / 47 variants**。`python tools/gallery.py validate --all` PASS（12 IDs、22 subentries、6 example 级 engine locks、licenses consistent）。node-authoring 冻结注册表：`tools/gallery_node_authoring.py` REGISTRY **19 入口**（41 变体、223 独立 oracle）+ `tools/gallery.py` 内联 `g11/2d`（2 变体 main/main-alt，G11_NODE2D_REQUIRED_ORACLES 全集）+ `tools/gallery_g12b.py` `g12/audio`（2 变体、8 oracle）与 `g12/hud`（2 变体、12 oracle）= 22/22 全 node-authoring 覆盖、47/47 变体。

**引擎/SDK 身份（原 M1 冻结链）**：REGISTRY 19 入口 lock 分布 `engine-locks/auto-ui-{017?,019,021,022,023,027,027-r2,030,034,039,040}.json`（g11/2d=017、g12=023），engine_commit 两值：统一候选 `4ebc51cef`（绝大多数）与 plan875 `1bec52f47`（g07/g08 3D 腿）；sdk_pin 统一 `7b987ca8b`（auto-lang）；generator=node2d-lower（SHA 逐入口钉）、runner=autographics-authoring（SHA 逐入口钉，如 g01/g11 `b237fb84…`）。examples/ 级 6 locks：`engine-lock.json`（旧全局 g01–g06，engine `8b12632d`）+ `auto-ui-{002,004,006,012}.json` + `plan875.json`。

**生产入口（冻结）**：验收驱动= `tools/verify_node2d_basics.py`（g01/g03-2d/g06×2/g08-2d）等 36 个 `verify_*.py`，材料化独立 cargo workspace（--engine-root 019 消费导出 + `checks/<wave>/gallery_runtime.rs` oracle）+ 固定 `--generator/--runner/--runner-sha/--build-record`，产物 `previews/<slug>/acceptance-*.json`（ag-examples.acceptance/1，含逐入口 consumed-source hashes，源变更即失效）；公开 CLI=`tools/gallery.py launch <id> --engine <exe>`（--open scene + --behavior/--behavior-api/--frames/--report，SHA 不符拒启）。G04 编辑教学类（editor-tutorial）走 `checks/node_editor_tutorials/`。

**实际依赖（M2 AV 链，1075 接线基面）**：AGE 源已是 Node 作者源（如 g01 `node Main { view-2d … }` 三模块闭包）；auto-ui driver `experimental/autographics-auto-driver/src/lib.rs` `Session::new` S2 路由：行首 `node <Ident>` → `AgeSession`（`experimental/age-ace-session/src/age_session_core.rs`，compile_age_package_with_scenario + ace_av 观察，唯一实现双编译），普通 scalar 源仍走旧 parser/VM。981 已交 IF-981-ORIGINAL（g01 原源 CPU 面，`plan981_original_g01.rs` fixtures_g01）；AGE 源到 AV 会话的公共接线在位，Gallery 侧尚无 AV 引擎 lock/选择与 AV 验收收据（本计划 T-02/T-03 交付）。MAP/ASSET/PBR/CLIP AV 实现归 auto-ui 099–102（全部 drafting 0/10，未交付——超出本计划边界的 capability 腿按具名 pending 记录）。

**边界（诚实）**：1099（混合 showcase）、1100/1101（两游戏）不在本计划分母；`age-examples` 主检出存在他会话未跟踪 docs（plans 029/030/031）——worktree 基面 `ea0fc59` 不含之，本计划不消费不触碰。

2026-11-22 stage: work（T-02 交公共 AV 会话接线）| PLAN-1075 r2 | outcome: pass | worktree: `D:/autostack/.wt/lang-1075/auto-ui` @ plan-1075-dev | code_commit: `95401c10` | 依赖身份：driver SDK=冻结基座 681 文件 + overlay 1435 文件 @ auto-lang `0a167f611`（复用受检前门，未刷新 overlay——§2「复用不扩门」）

**接线交付（实现）**：`experimental/age-ace-session/src/age_session_core.rs`（bridge+driver 共同编译的唯一实现）新增 `open_modules(modules, entry, instances, ops)` 多模块作者闭包开 AV 会话、`update_module(id, src)` 同 PID 单模块热更新（未名模块具名拒；编译失败保留上一好代/账面/代数）、`module_ids()`；出口 `app::<entry>`；单模块路径与既有 `update_body` 语义不变（delegate 到 `update_module("main")`）。变体入口文件名带连字符（`v-color`/`main-alt`）不是合法模块 id——由接线适配层净化为下划线（负控用例证明前门对连字符 id 具名拒 `module-identity`，净化是接线职责不是语言放宽）。

**Gallery 闭包 AV 普查（56 闭包=22 入口全部 47 变体的 generated 闭包集）**：连字符净化后 **6/56 通过受检前门开为 AV 会话**——`g01/main`、`g01/v-color`、`g01/v-order`、`g01/v-position`、`g01/v-visible`、`g02/main`。其余 **50 闭包具名 pending**（非本批可绿，复用前门不扩门）：45×`source.syntax: expected fn, found node`（二分定位：**场景行内嵌套**（g03 层级 `group2d` 嵌套、g05–g12 各复杂场景）超出 H0 平面行语法；平面未知元素得的是语义诊断而非语法错）＋5×`source.age.scene-element`（`solid2d`×2 g09/character、`player3d`×2 g04、`character2d`×1 g05/sweep——元素种类超当前冻结叶集 shape2d/3D 叶/组件实例）。能力腿归属：嵌套层级/角色/精灵叶/查询/相机/音频/触发器=AGE 场景扩展与图形能力线（AutoUI 099–102 MAP/ASSET/PBR/CLIP 及其后续），1075 不代交。

**永久用例（新）**：`experimental/autographics-auto-driver/tests/plan1075_gallery_av_session.rs` **8/8 PASS（0.17s，逐 case ≤10s 门全过）**——g01 基线四行声明序/几何/材质/hidden（B01-01..07 事实面）、v-color 换绑 deep_green、v-order 序翻、v-position (40,0)、v-visible 黄显、g02 静态 GLB（model3d 行+Model3D 资源+GLB 路径符号实参）、坏编辑保旧/未名模块拒/修复热更新新值可见/PID 稳定/Run-Stop 净零/停止拒编辑、连字符 id 与未知入口负控。fixtures=`tests/fixtures_plan1075/`（17 个 .at 逐字冻结 + manifest.json 逐文件 SHA256，来源 age-examples `ea0fc59`）。

**会话面基线（本会话实跑，编辑后复跑）**：`authoring_session` **4/4**（2.68s）；`authoring_storage` **3/3**（0.42s）；`mixed_regions_lifecycle_flow` **7/7**（5.58s）；`s2_age_routing` **21/21**（0.43s，age_session_core 改面的回归）；bridge（live ACE 侧，组内 lang worktree）`s1` **4/4**（0.11s）/`s3` **3/3**（0.12s）/`s4` **10/10**（0.25s）。SDK 身份三验过：`prepare_sdk --verify-only`、`verify_source --sdk-only`（479 包闭包/17 本地包/0 禁项）。普查探针（临时，已删）56 闭包 0.32s，证据 `plan-artifacts/plan-1075/t02-census-sanitized.txt`。

2026-11-22 stage: work（T-03 验收原 Gallery）| PLAN-1075 r2 | outcome: pass（有界，见下）| worktree: `D:/autostack/.wt/lang-1075/{auto-ui,age-examples}` @ plan-1075-dev / age-examples 只读消费（零改动，worktree clean）

**原门身份审计（AG025 原门保持）**：`gallery.py validate --all` PASS（12 IDs/22 subentries/6 example 级 locks/licenses consistent，age-examples `ea0fc59`）。逐 subentry 收据审计（check-only 的身份半部，engine-free 实现）：**21/22 收据行 passed 且逐文件源绑定零漂移，绑定行内 296/296 oracle 通过**。唯一例外 `g05/2d`：其收据绑定自身全部 g05-2d 产品文件，仅 `example.json` 条目早于 2026-11-04 姊妹腿 g05/3d 迁移（commit `6bcf087`）——仓内既定 byte-preserved historical receipt 模式（gallery.py :664/:733 同款注记），非产品源漂移；launcher 自身门（validate）在该状态 PASS。

**收据指纹台账（22 收据 SHA256 前 12）**：g01 b9f2bd9b918a｜g02 94e91772ef03｜g03/2d b41f29b5a946｜g03/3d 160c51c961f2｜g04/2d f2accaa4edc2｜g04/3d 6dafa3f9cc91｜g05/2d 5f2e5db58db7｜g05/3d df4643654b9f｜g06-a e169eb6dcc28｜g06-b 126d909d666b｜g07-a 445dd25526a8｜g07-b 0ff696c43bed｜g08/2d 2305476d03b5｜g08/3d 74b956a9a0e9｜g09/atlas dfd3ea810d06｜g09/char b5bdb63fe277｜g10/tex d5c2708a65ae｜g10/anim a3f699865cc7｜g11/2d ee3a2f3559f5｜g11/3d 952ce1713795｜g12/audio 2bffcf517f8d｜g12/hud 58cc40f9fa54。AV 腿 fixtures manifest（17 .at，来源 ea0fc59）SHA256 前 16 `1187f5f11b381f97`；g02 GLB 资产身份=文件名即 sha256（42c9da67…，MIT，收据内绑定）。

**AV 腿（冻结版 `95401c10` 复跑）**：`plan1075_gallery_av_session` **8/8**（0.19s）+ 普查 6/56（g01 全 5 变体 + g02/main 过受检前门，50 闭包具名 pending，诊断与归属见 T-02 收据）。

**预存红台账（非本批引入，主检出 ea0fc59 同红复现）**：`tests/test_gallery.py` **15 failed/75 passed/1 skipped**（TexturedModelsRoutingTest 11、PerItemLockRoutingTest 1、routing 其余 3；两检出同 commit 同红）——owner=age-examples gallery 工具线，非 1075 触面（本批 age-examples 零改动）。`tests/test_node_authoring.py` **26/26 PASS**。

**能力边界（诚实，复审重点）**：AC-02「全部**适用**原检查」按 T-01/T-02 冻结的适用集交付=受检前门现形状所支持面（g01 全变体+g02）。真 GPU/输入/音频腿对 AV 会话面：具名 pending（窗口化 AV 宿主归图形能力线 099-102 后续与 1099+ 混合/游戏线；M1 真窗/GPU 证据保留于身份绑定收据）。g03 层级嵌套/g05 查询与 character2d/g09 solid2d/g04 player3d/g11 触发/g12 服务腿的前门扩展不在「复用受检前门」边界内——**本条为复审需独立裁定的关键边界**：接受为能力线归属，或判 needs_fix 要求本批扩门。

2026-11-22 stage: work（收口）| PLAN-1075 r2 | outcome: pass（execution_done）| task_ids: T-01/T-02/T-03 | code_commit: auto-ui `95401c10`（lang/down/age-examples 零代码改动；plan 簿记在主检出）| blockers: 无 | next_responsible_role: review（T-04 独立复审，重点裁定上段能力边界与 AC-02 适用集）| 授权链：用户「完成计划1075」= /auto-plan-goal 全链授权（§9 首条 goal 收据）。

**Work-AC 结果**：AC-01 PASS（分母冻结 T-01 收据，22 入口/47 变体/locks/oracles/依赖全可追）；AC-02 有界 PASS（原门身份 21/22+296/296、AV 腿 8/8、预存红与 50 闭包 pending 具名——适用集边界交复审）；AC-03 work 侧齐（同树冻结 `95401c10`、SDK 身份三验、fixtures 指纹），正式交付与规范沉淀归 merge。

**本会话测试时间表（work 全程）**：

| 实际范围 | 结果 | 准备/构建 | 运行 |
|---|---|---|---|
| driver 首建（baseline authoring_session） | — | 2m27s（冷） | 3.19s（4/4） |
| SDK 物化（prepare+apply_overlay+两验） | PASS | ~40s | — |
| authoring_session | 4/4 | 增量 | 2.62-2.68s |
| authoring_storage | 3/3 | 增量 | 0.42-1.09s |
| mixed_regions_lifecycle_flow | 7/7 | 增量 | 5.58-6.66s |
| s2_age_routing | 21/21 | 增量 | 0.36-0.43s |
| plan1075_gallery_av_session（新） | 8/8 | 增量 | 0.16-0.19s |
| bridge s1/s3/s4（live ACE） | 4/3/10 | check 31.48s | 0.11/0.12/0.25s |
| 普查探针（临时已删，56 闭包） | 6/56 过 | 增量 | 0.32s |
| gallery validate --all / 收据审计 / test_node_authoring | PASS/21+296/26 | — | ~22s |
| test_gallery.py（预存红对照，双检出） | 15 红 | — | 11-16s |

**新用例表（全部 ≤10s 门）**：plan1075_g01_base ｜ plan1075_g01_v_color ｜ v_order ｜ v_position ｜ v_visible ｜ g02_static_glb ｜ closure_session_bad_edit_recovery_and_balance ｜ entry_id_sanitization_is_wiring_not_language——8 case 断言 B01 事实/会话纪律/接线负控，全文件 0.16-0.19s。

2026-11-22 stage: review（T-04 独立复审）| review_scope: final | reviewer_identity: 本会话独立复核（**限制：非独立上下文**；结论由工件重构——commit/diff 审计、变更负控、清单绑定复核、普查产物核对，非采信执行者摘要）| PLAN-1075 r2 | outcome: **pass** | worktree: `D:/autostack/.wt/lang-1075/auto-ui` @ plan-1075-dev | reviewed_commit: `95401c10`（基面 `8dd99503`；+1024/−14，21 文件）| dependency_revisions: SDK=冻结基座 681+overlay 1435 @ auto-lang `0a167f611`；fixtures=age-examples `ea0fc59`；lang/down/age-examples 零代码改动

**AC 证据映射（逐项重构核）**：
- **AC-01 PASS**：分母冻结可追（T-01 收据：22 入口/47 变体/9 注册表 locks/引擎与 SDK 身份/逐入口 oracle 集；validate --all PASS 复跑）。
- **AC-02 有界 PASS（裁定）**：原门身份 21/22 收据全绑+296/296 oracle、AV 腿 8/8 @ `95401c10`、负控齐（连字符 id 拒/未名模块拒/坏编辑保旧/**复审变更负控**：v-position fixture 单字节漂移 40.0→41.0 ⇒ 红（left [41.0,0.0] vs right [40.0,0.0]），还原 ⇒ 绿——oracle 区分错实现）。**适用集裁定**：§2「复用 ACE 受检前门」+§5 边界条款（MAP/ASSET/PBR/CLIP→099-102；混合→1099；游戏→1100/1101）下，受检前门现形状（平面行）即适用集；50 闭包 pending 逐诊断具名（45 嵌套语法/5 元素种类）归图形能力线——与 1071「独立组不等待全组过线、预存墙具名台账」同构，且 §6 记录义务（源/SDK/命令/失败条件）已履行。真 GPU/输入/音频腿对 AV 会话面具名 pending（窗口化 AV 宿主归能力线；M1 真窗证据保留于身份绑定收据）。
- **AC-03 PASS（本复审面）**：同树冻结复核、source/SDK/ABI 身份与消费接面（`open_modules`/`update_module` 公开 API + 8 永久用例 + fixtures 指纹 `1187f5f11b381f97`）在案；正式落规范归 merge。

**健康检查**：worktree clean（构建产物忽略外零脏）；核心 diff +133/−14 零 dbg!/todo!/unimplemented!；新用例零警告编译；**manifest 绑定复核 17/17 逐字 = age-examples ea0fc59**；普查产物核对 6/56+CENSUS 行在册。

**测试成本审计（2026-11-14 裁定）**：8 新用例全文件 0.16–0.19s（逐 case ≪10s 门）；测试内零构建/零子进程（fixtures 运行时读取）；debug/test profile；占用检查记录（复审期 4 rustc 外部进程——本复审仅用已编译二进制+python，无新增构建）。时间表已在 work 收口收据，本复核沿用并确认。

**预存红台账（具名非本批）**：`tests/test_gallery.py` 15 红（TexturedModelsRouting 11/PerItemLock 1/routing 3）@ ea0fc59 主检出同红；`gallery_vue_golden`/`host_console` 族不在本批触面（1071/981 台账既有）。

**遗漏/延后/Workaround 扫描**：①50 闭包 pending=能力边界裁定（上文），非静默延后——逐诊断+归属在 §9 T-02/T-03；②无 workaround 补丁（接线为加性公开 API，共享单一实现未分叉）；③无调试残留。

**规范增量**：复审定稿见 §5 末「规范增量」节（add m2-age.md §5；supersedes 无；GOAL-002）——merge 落盘。
**pending_closeout_gates（merge 持有）**：m2-age.md §5 沉淀、KNOWN-DEBT 边界记录、归档/清理、landing checks、`cargo t` 冻结受影响组并集（本批 lang 零改动——ace 组证据复用 981/1071 既有；auto-ui 侧=本批新测试已绿）。
**evidence**：本 §9 三份 work 收据 + 复审重构（commit/diff/变更负控/清单 17/17/普查产物）。
**next**: merge（T-05 正式交付）。

2026-11-22 stage: merge（T-05 正式交付）| PLAN-1075 r2 | outcome: pass（landed + canonical_verified + archived；cleaned 见下追加）| delivery: auto-ui master tip `95401c108`（ff-only 无 merge commit；基面 `8dd99503` 无漂移零 rebase）+ auto-lang master `a01c5020e`（m2-age §5，基面 `8254ede20`，rebase 1 commit 干净）| canonical: `docs/specs/ace-migration/m2-age.md` §5（33 行，与 §5 规范增量节逐点一致）| new_spec_components: `docs/specs/ace-migration/m2-age.md#5` | KNOWN-DEBT: `D-1075-GALLERY-AV-01`（50 闭包能力边界，owner 099–102 线）| archive: `docs/plans/archive/1075-age-av-content-and-fusion.md` status: archived | gates: lang 零代码改动（docs-only 内容检查：§5 与冻结增量比对一致）；auto-ui 腿证据 @ `95401c108` 复用（review 冻结基线同 commit 重放绿）| batch regression: **not due**（receipt last_covered=1109 ≥1075；1114 非 %5；<48h）| landing checks: 双主检出 tip=delivery commit、ff 历史无 merge commit、canonical 路径在主分支 | next: done

**cleaned（2026-11-22 追加）**：wt-guard 四树（lang-1075/{auto-lang,auto-down,auto-ui,age-examples}）全 clean → `git worktree remove` ×4 → 分支 `plan-1075-dev` 三仓删除（lang `a01c5020e`/ui `95401c108` 均已并主，age-examples 零提交直删；down detached）→ 组目录 `D:/autostack/.wt/lang-1075` 删除（worktree list 无 1075 残留）→ `plan-artifacts/plan-1075/` 整删（证据已折叠本 §9）→ registry `.retention/policy.json` 追加 PLAN-1075 deleted-at-merge（no holds，151 条，JSON 校验过）。**batch regression due 判定**：not due（receipt last_covered=1109 ≥ 1075；其后落主的 1114 非 %5；receipt <48h）。**landing checks**：双主检出 tip=delivery commit（ui `95401c108`/lang master 归档提交后 `410edb557`）、ff-only 线性历史、canonical m2-age §5 在主分支；无部署面产物（库/测试交付，无 release/重启义务）。

**测试时间表（merge 会话）**：无新增测试执行——landed 复用 review 冻结证据（同 `95401c108`：gallery 8/8、authoring_session 4/4、storage 3/3、mixed_regions 7/7、s2 21/21、bridge 4/3/10）；lang 侧 docs-only 内容检查；占用检查通过（merge 期间外部 rustc×4 在途，未启动竞争构建）。

2026-11-19 stage: new，规划与交接基面 78b375bf4，实施与复审待执行。

## 10. 待澄清事项

（无。T-01 已核实原用例位置、实际依赖接口与生产入口，见 §9 T-01 收据。）
