---
plan_id: PLAN-720
status: reviewed
feature_name: 网站虚拟桌面实景展示
author: [agent]
created_at: 2026-10-01
updated_at: 2026-10-01

plan_revision: 1
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [website]
current_step: 3
total_steps: 3
---

# [PLAN-720] 网站虚拟桌面实景展示

## 0. 变更摘要

将用户提供的 auto-os/desktop-shots/showcase 六张原生截图加入 v0.5、AutoOS 虚拟桌面和 OS 总览中英页面。以深浅色无应用窗口桌面开头，随后 Launcher、小游戏、多应用工作布局。

## 1. 目标

真实截图循序介绍桌面日常使用；说明小组件来自已启动并最小化的应用。保留原图、放大、VitePress 与双语结构。不重拍、不改桌面代码、不实施 PLAN-719 架构文稿，不部署公开站点。

## 2. 架构方案

新增作者数据 website/.vitepress/theme/data/desktop-showcase.ts 与共享 DesktopShowcase.vue；四步介绍配一套六图选择器（前两帧纯桌面）；总览模式仅双主题桌面与专题链接。复用 ScreenshotGallery/EvidenceImage，不新增运行依赖。

## 3. 技术栈

现有 VitePress 1.6.x、Vue 3、CSS、Playwright。六 PNG 复制到新目录 website/public/desktop-showcase/，原始 SHA256、2560×1600 和画面内容不变。

## 4. 需求分析与背景调查

- 用户本条明确授权“在 v0.5 宣传页面和 OS 的虚拟桌面部分的页面里酌情加上这些截图”，指定循序描述及两张桌面图开头。具体顺序与页面范围已在会话中呈现，继承明确实施授权，不重新索要相同范围许可。
- 六图已人工查看：01/02 桌面与小组件，03/04 Launcher 网格，05 俄罗斯方块/纸牌接龙/扫雷，06 编辑器左半屏、Todo 右上、日历右下；捕获日 2026-10-01。画面证明捕获时的布局，不扩展为所有功能或跨端一致性证明。
- 取材 docs/specs/overview.md、website/project.md、design/ui-presentation.md；调查主 HEAD=9fc43d2d1。715 发布数据/实图规则适用；718/719 工作区与稿件保留。ui-desktop.md 为独立后端占位页，本轮不扩展到重写该页。
- Category A，仅网站构建/内容/行为，不跑 Cargo/docs_gen。工作树 D:/autostack/.wt/lang-720/auto-lang，plan-720-dev；计划簿记在主检出。auto-os 源图只读。

## 5. 详细设计

- v05 hero 换 02 深色纯桌面，维持 eager/尺寸占位/放大；桌面段说明保留，六图与四步介绍共用作者数据。前两帧浅色/深色桌面，后两帧浅色/深色 Launcher，最后游戏与工作。
- autoos 完整共享展示放介绍后、统计前；原桌面外壳区旧三图换新工作场景，保留原说明。os 总览在介绍后放双主题桌面，并链接 /autoos/#desktop-showcase（ZH 同构）。
- 小组件说明应用已打开并最小化；Launcher 图标网格由 Tab 切换；游戏说明窗口拖动并排；工作场景说明编辑器占左半屏，待办和日历在右。音乐/日历展示状态只描述画面，不称实时播放或日历为当天。
- 图片保留全幅、非首图 lazy、尺寸占位；选择器自动换行且可键盘使用，窄屏无横向溢出。保留原发布统计、路由与其他章节。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/website/project.md | 分散桌面图 → 六图同源四步说明、总览链接 | 可追溯、双语一致 | AC-01/02/03 |
| SD-02 | modify | docs/specs/website/design/ui-presentation.md | 实图主图 → 本批首推无窗口桌面、小组件前提、循序原图保全 | 说明与证据相符 | AC-04/05/06 |

## 6. 测试设计

原/入库图逐个 SHA256 与尺寸核对；prepare/build；专属端口、CI=1 跑既有网站 e2e 与新增展示/链接/键盘/几何用例。六路由×五宽度×双主题检查无溢出；390/1440×双语×双主题×三页面，共24张展示截图与 manifest，并人工查看关键样例。

## 7. 验收标准

- [x] AC-01 六图原样入库，有源路径、捕获日、尺寸、SHA256。
- [x] AC-02 v05/autoos 双语顺序为桌面双主题→Launcher→游戏→工作；OS 预览进入正确语言专题。
- [x] AC-03 单一作者数据、六路由无死图/裸组件源文；原 URL、统计与其他章节保留。
- [x] AC-04 小组件前提、游戏名称、工作布局描述与画面一致，不虚构音乐播放/日历当前日期。
- [x] AC-05 五宽度×双主题无横向溢出，图片键盘放大/原图/Esc 回焦、尺寸占位与延迟加载正常。
- [x] AC-06 build、既有与新增网站 e2e 通过；24截图落地/正文有效、看图无遮挡，无新警告或范围外修改。

## 8. 执行步骤

- [x] T-01 入库与共享展示：复制六 PNG，新增 data/desktop-showcase.ts、components/DesktopShowcase.vue，报告 docs/reports/p720-desktop-showcase.md 记录源证据；核对 hash/尺寸。AC-01/04/05，SD-01/02。（重拍六图同步，4张实际变化；最新 hash/尺寸与画面已复验。）
- [x] T-02 页面接线：修改 ReleaseLanding.vue/release-v05.ts、autoos/index.md、zh/autoos/index.md、os.md、zh/os.md，双语接线与首图/旧图替换；prepare/build/源码核查。AC-02/03/04，SD-01/02。
- [x] T-03 验证交付：新增 tests/desktop-showcase.spec.ts，24图入 docs/reports/p720-desktop-showcase/；网站 e2e、逐项 AC/SD 复核，review 后按已授权网站合入范围落 master。store writer 不可用时 ledger pending、保持 reviewed，不提前归档。AC-01..06。（重拍修正的图片、构建与视觉证据已刷新；保留旧历史记录。）

## 9. 复审记录

stage: new | plan_id: PLAN-720 | plan_revision: 1 | outcome: pass (contract) | next: work。T-01..03 覆盖 AC-01..06、SD-01/02，用户已明确授权本次截图更新，不代表实现完成。

### 2026-10-01 execution_done → review

stage: work | plan_id: PLAN-720 | plan_revision: 1 | outcome: execution_done | implementation_commit: d2eb9f8935ab0109cadba8779a2baade933c0c92 | evidence_commit: ecdfcc7655a5409cac42449c674958ae216b1815 | next: review。

stage: review | plan_id: PLAN-720 | plan_revision: 1 | outcome: pass | reviewed_commit: ecdfcc7655a5409cac42449c674958ae216b1815 | base_commit: ccf1b9b78e43b49ec6913234df7d0d88dbfa45e7 | dependency_revisions: book=d7a71a7fb1fa42ddd26d6cec859715ed98161f7a（只读） | evidence: docs/reports/p720-desktop-showcase.md 与同名目录 manifest/24 PNG | next: merge。

复审上下文限制：在实施会话中按 review 技能重建证据，未创建独立会话/agent，不声称人员独立。已提交实现与 diff、六张原图、最终生产构建、专项及完整回归、视觉包逐项复核；未以勾选框或执行摘要代替证据。

| acceptance_results | tasks | evidence | verdict |
|---|---|---|---|
| AC-01 | T-01 | 源/入库 6 SHA256 一致，PNG IHDR 均 2560×1600，报告首表 | pass |
| AC-02 | T-02/03 | 六路由逐帧检查，四步顺序，OS 专题当前语言 href/实际跳转/标题避顶栏 | pass |
| AC-03 | T-01/02/03 | 作者单源、六路由无裸组件/404/死图；既有统计、导航和应用回归 | pass |
| AC-04 | T-01/02 | 六源图与双语文案逐一核查；小组件最小化、三游戏与半屏工作布局准确 | pass |
| AC-05 | T-01/03 | 六路由×五宽×双主题共60组合；键盘打开/关闭/回焦、原图、lazy 与占位 | pass |
| AC-06 | T-03 | build 146.75s exit0；最终专项8/8、完整77/77（3.9m），24截图可追溯且无遮挡 | pass |

spec_inputs / frozen_delta（完整差异固定于 reviewed_commit）：SD-01 project.md hash ae221cbbadcaac30daef10976c002fcd08b169fbe9717a5bb33cfe6a4064b8cf → 71e2ddeeaf83158593599a5652b8a80abf6f641e266300fcf4c8578e9196f7ba；SD-02 design/ui-presentation.md hash b5bbbeee215fb26b856f33ed12fdf2479cd7bd215f3aa3d740fe2bd79580ae8d → 25d3a9920fd5e60999fc88abc977fe8967bd9260b645c4c577c928f74a7dbdd7。两项 modify 与实现/AC 一致，无新增要求；website/plans.md 附随行 hash cb47183bc0416aec065a740815fcd5a30f06c914e5d75151703a806b732326ea，spec-index 再生无语义改动。supersedes/new/touched_goals 均空：更新既有网站展示契约，不新增/替代组件或目标。

findings：遗漏/延后/workaround 扫描无新增债务，无 Rust/后端/桌面应用改动，无 debug/TODO 替身。首次 console 检查曾 transient hydration fail，未降低原断言；在最终稳定 dist 上专项与整站均 pass，历史结果保留报告。既有高亮回退与 bundle 提示未增加。原 worktree guard 已确认 clean、无 junction/symlink。公开部署不属本次范围。

### PLAN-720:r1 合入检查点（2026-10-01）

stage: merge | plan_id: PLAN-720 | plan_revision: 1 | outcome: blocked（仅 ledger 发布/归档收尾；网站代码已交付） | delivery_commit: 19bbbdeeab77e6c57aa4d7031c5fc3be14eb23f5 | next: store writer 恢复后校对既有条目、发布并读回，再归档与 guard 清理。

| checkpoint | evidence |
|---|---|
| prepared | reviewed_commit ecdfcc7655a5409cac42449c674958ae216b1815；SD-01/02 已冻结且 plans.md 附随行复核，工作树 clean；Git Bash wt-guard 显示 clean、无 reparse point。目标投影为 designs→website/project.md、designs→website/design/ui-presentation.md，tests/reviews/reports 引用本报告与 PLAN-720:r1；服务恢复后按 canonical target 复用真实 item ID，未盲目创建重复条目。 |
| landed | master 从 cf38117503c601c2afe1f33561d093c8240ea524 ff-only 到 19bbbdeeab77e6c57aa4d7031c5fc3be14eb23f5，无 merge commit；当时 master HEAD 与 delivery 相同。并发主线仅 718/720/721 簿记，没有 website/Spec 变更。 |
| rebase equivalence | `git range-diff ccf1b9b78e43b49ec6913234df7d0d88dbfa45e7..ecdfcc7655a5409cac42449c674958ae216b1815 cf38117503c601c2afe1f33561d093c8240ea524..19bbbdeeab77e6c57aa4d7031c5fc3be14eb23f5` 三项全等：333a2042ccc5dfbd128caf4b9423ddea3e249a1e→41e8b686c619867c3888969ecc426fbcfc61eb8b；d2eb9f8935ab0109cadba8779a2baade933c0c92→ba66d2ad1ff14f73b9dbcee56a2d46255a906d9c；ecdfcc7655a5409cac42449c674958ae216b1815→19bbbdeeab77e6c57aa4d7031c5fc3be14eb23f5。旧 review 绑定的 website/Spec 内容不变。 |
| integration | 主线三份 Spec SHA256 与冻结值一致；旧 reviewed_commit→delivery 的 website/Spec diff 为空；最终 production preview 的 /zh/v05/、/zh/autoos/、/zh/os 均 HTTP200、包含真实桌面 src 与最小化说明。无代码更改，复用已识别的 77 pass 证据，无额外 Cargo。 |
| ledger_refreshed | pending：会话无 read/list/write/update Spec tools；HTTP 127.0.0.1:8080/api/specs 连接拒绝。没有手写 .autoos/specs.json，没有提前归档。目标 workspace D:/autostack/auto-lang，runtime ledger .autoos/specs.json。 |
| archived / cleaned | pending：Plan 保持 reviewed；保留 D:/autostack/.wt/lang-720/auto-lang 与 plan-720-dev（clean、已合入）用于来源复核和预览。 |
| production artifacts | website/.vitepress/dist 是 2026-10-01 本次稳定构建（146.75s）；本地 http://127.0.0.1:4220/zh/v05/ 与 /zh/autoos/ 使用此产物。公开 http://112.74.45.241 未部署，本请求只加入图片并沿用已授权主分支合入，不声称线上已更新；无后端或依赖二进制变更。 |
| batch handoff | 读取 .last-batch-regression.json：last_covered_plan_id=715、2026-09-30T16:20:00Z，720 为 %5 节点。归档/cleaned 之后的批量回归检查点尚未到达，且本次 Category A 禁止 Cargo；到达该检查点再交 /auto-plan:regress 主检出单实例，不改旧收据、不把网站 77 pass 写成 Rust 批量覆盖。 |

### 2026-10-01 用户重拍修正（repair-1）

stage: work | plan_id: PLAN-720 | plan_revision: 1 | outcome: executing | reason: 用户明确报告桌面快捷方式缺失，要求将新六图替换到网站并合入 master | next: 同工作树资产替换、核对、复核后再次 ff-only 合入。原协议与 AC/SD 不变，不新建计划或改写旧 review；旧 pass 仅覆盖旧资产。T-01/T-03、AC-01/04/05/06 重新验证。

取材当前主线 da724ab24；新 01..04 分别在 14:18..14:23 重拍，人工查看确认桌面多列快捷方式恢复，Launcher 背景相应更新；05/06 与原图 hash 相同。只替换资产并更新来源/视觉证据，不改页面 Vue/路由/正文。验证按 Category A：build、现有六路由展示用例/五宽双主题与24图刷新，以及相关发布页图片/console 检查；未触及行为的旧77整站证据明确复用，非伪称重跑。

### repair-1 完成与复审

stage: work | plan_id: PLAN-720 | plan_revision: 1 | outcome: execution_done | code_commit: 699368fd8e79a533c8a594bc197904877a152fe9 | task_ids: T-01/T-03 | evidence: docs/reports/p720-desktop-showcase.md 修正验证结果与刷新24图 | next: review。

stage: review | plan_id: PLAN-720 | plan_revision: 1 | repair: 1 | outcome: pass | reviewed_commit: 699368fd8e79a533c8a594bc197904877a152fe9 | base_commit: 1867dfce452a59bef8c098afb7e6fcf6c2c3fd96 | dependency_revisions: book=d7a71a7fb1fa42ddd26d6cec859715ed98161f7a，继承已合入 PLAN-718 阅读实现 | next: merge。

acceptance_results：AC-01=6张源/入库/构建/HTTP hash 一致且均2560×1600；AC-02/03=六路由逐帧/专题链接、原接线与正文未改；AC-04=4张重拍原图与新视觉包人工对照，快捷方式恢复、小组件/Launcher/游戏/半屏工作布局说明准确；AC-05=键盘/放大/原图/回焦、60组合几何与解码后24图；AC-06=集成 build exit0（197.08s）、相关14项 pass（1.2m），仅受影响矩阵补 img.decode 后重跑1项 pass（35.4s），其他13项实现/依赖/断言不变明确复用，旧77项仅作为历史结果。每项 pass。复审在本实施上下文按提交/diff/原图/HTTP/hash/运行和视觉证据重建，不声称独立人员。

spec_inputs：主线718当前 website/project.md=994113f25acd22617fb92862204fe14a737077d6caa0e66d35c53c4a55a8ebdc；design/ui-presentation.md=07a644d95f92d73ef244fcebafb4a1cd75c4b6ce32cde8825ac25323ec7734c0；plans.md=cb47183bc0416aec065a740815fcd5a30f06c914e5d75151703a806b732326ea。修正无额外 Spec delta；原 SD-01/02 现状契约保留且未覆盖718变更，影响元数据仍为空，无新组件/目标/架构。source SHA256 冻结于报告首表和 reviewed_commit。

findings：早期视觉取景1张异步解码空白，已增加 decode 等待并重新生成24图；旧4220服务元数据按旧尺寸截断响应，已确认进程归属后重启，六张 HTTP bytes hash 全相同。没有修改网页行为、删除断言或延期验收；无新增债务，无 Rust/Cargo/docs_gen。原资产 264c0ec6c→5b0426b7c range-diff 全等；主线并发网站718变更已同步后重建复验。

## 10. 待澄清事项

无阻塞偏好；采用站点现有视觉与用户截图顺序，保留其他会话 WIP。
