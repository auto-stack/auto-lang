# PLAN-720 — 网站桌面截图展示

## 来源与保全

用户提供 `D:/autostack/auto-os/desktop-shots/showcase`，捕获日 2026-10-01。六张原图均人工查看；本仓复制到 `website/public/desktop-showcase/`，逐一 SHA256 与来源一致，无裁切、重绘或重新编码。画面只证明捕获时的布局，不证明所有应用功能、跨端等价、音乐实际播放或日历为当天。

| 原文件与入库文件名 | 尺寸 | SHA256 |
|---|---|---|
| 01-desktop-light.png | 2560×1600 | 644b503f395bfc2d49e3b187815d3162660f76bfa709cb9bfe3d16862526d7e6 |
| 02-desktop-dark.png | 2560×1600 | 77ef82ea50beaa99a29dc1ca9492ea11cdd47535b0866c10b537f4bdca3dc042 |
| 03-launcher-light.png | 2560×1600 | 20696b4380e10bb933b169ff4b941f225d52980db083141527a7cf63259b9ce3 |
| 04-launcher-dark.png | 2560×1600 | d2409be62d526261a3dffeca3c0af7b0b5ab72c8b7efff9e350357ad2af9cce3 |
| 05-games-dark.png | 2560×1600 | 40cd4c89d87b7e23398771d9eda1d5781c6b9a3779551cbd7fab3f90fc6cd480 |
| 06-productivity-dark.png | 2560×1600 | d488fb14350f9db3d9cd83cc95eee38ebc1e3bd6c1059eaaf3bdd8b7b3dbe63b |

## 展示设计

`desktop-showcase.ts` 共用六图的双语标签、alt、caption、尺寸及四步说明。`DesktopShowcase.vue` 复用原 ScreenshotGallery/EvidenceImage：前两帧纯桌面，然后 Launcher、三游戏、工作场景；图片可放大、查看原图、键盘选择。网站主题切换不改写截图，深浅两帧作为实际画面对照。

- v05/zh/v05：首图更新为新深色纯桌面；桌面段为四步说明与六图选择器。保留其他章节、统计与路由；现有“真实 Launcher、非占位”回归更新为新的实际网格图，未删除该用例。
- autoos/zh/autoos：完整介绍放在 OSHero 后、统计前；旧三图替换为新工作场景图，原桌面外壳说明保留。
- os/zh/os：双主题桌面预览和对应语言的完整专题链接。架构文稿保持 PLAN-719 范围，不在截图任务中重写。
- 小组件的说明明确对应应用已启动并最小化；不称桌面上没有运行应用，只称没有展开的应用窗口。游戏为俄罗斯方块/纸牌接龙/扫雷；工作场景编辑器在左半屏，Todo 和日历在右侧上下。

## 调查基线

base commit：ccf1b9b78e43b49ec6913234df7d0d88dbfa45e7；book 只读依赖 d7a71a7fb1fa42ddd26d6cec859715ed98161f7a。Spec 输入 SHA256：project=ae221cbbadcaac30daef10976c002fcd08b169fbe9717a5bb33cfe6a4064b8cf，ui-presentation=b5bbbeee215fb26b856f33ed12fdf2479cd7bd215f3aa3d740fe2bd79580ae8d。

## 首批验证证据（2026-10-01，历史资产）

网站实现基线：`d2eb9f8935ab0109cadba8779a2baade933c0c92`（在 `333a2042ccc5dfbd128caf4b9423ddea3e249a1e` 展示实现上补专题锚点偏移）。仅网站/文档改动，Category A，未运行 Cargo 或 docs_gen。

| 检查 | 可复现命令/方法 | 结果 |
|---|---|---|
| 入库保全 | 对每张源图与 `website/public/desktop-showcase/` 做 SHA256 比较，读取 PNG IHDR 尺寸 | 6/6 相同，均 2560×1600；上表冻结来源 hash |
| 构建 | 在 website 执行 `npm run build` | exit 0，146.75s；既有 auto 高亮回退和 bundle 大小提示，无新增构建警告 |
| 语料预处理 | 根目录 `node scripts/build-playground-notes.mjs --check` | exit 0；vm 471、aavm 158、demo 28（项目 4）、book 634、parity 52 |
| 专项复测 | `AUTO_WEBSITE_TEST_PORT=4215 CI=1 npx playwright test tests/desktop-showcase.spec.ts tests/site-ui.spec.ts --grep 'real desktop\|six pages\|no new console errors on /v05/' --workers=2 --reporter=line` | 8 passed，52.9s；含 v05 console hygiene |
| 完整网站回归 | `AUTO_WEBSITE_TEST_PORT=4218 CI=1 npx playwright test --workers=4 --reporter=line` | 77 passed，3.9m；CI 独占 preview 服务，期间未改写 dist |
| 响应式 | 六路由 × 360/390/768/1024/1440 × light/dark | 60 组合无横向溢出、404、错误 locale/theme；图片自然宽 2560 |
| 视觉包 | `p720-desktop-showcase/manifest.json` 与 24 PNG；390/1440 × 六路由 × 两主题 | 24/24 路由、正文与文件有效；人工查看窄屏发布页、宽屏虚拟桌面与窄屏总览，图片/标题/正文无遮挡 |
| 规范索引/格式 | `python scripts/spec-index.py`、`git diff --check` | 26 项目，索引无语义变化；diff check 通过 |

首次整站运行曾得到 76 pass / 1 fail，唯一失败是 `/v05/` 的 hydration warning。当时构建重跑与测试尾段重叠，不能据此断定根因。最终稳定构建上的同项专项检查与完整 77 项回归均通过；保留原 console hygiene 断言，未加白名单、跳过或降低断言。

初次视觉取景时固定栏遮住展示顶部，已扩大截图取景高度（宽度矩阵不变），并给完整专题加入导航栏锚点偏移。新增用例实际点击总览→专题后检查标题在顶栏下方，最终 24 图重新生成。

## AC → 实现与证据

| 验收 | 实现/任务 | 证据与结论 |
|---|---|---|
| AC-01 | T-01，public/desktop-showcase 六 PNG | 来源 hash/尺寸核对及本报告首表；pass |
| AC-02 | T-02，DesktopShowcase、六路由调用 | 六路由逐帧 src 检查、四步正文、OS 当前语言专题点击与无遮挡；pass |
| AC-03 | T-01/02，共享 desktop-showcase.ts 与 release 数据 | 共用作者数据、源码 diff、六路由和完整旧回归（统计/章节/真实 Launcher/无裸组件源）；pass |
| AC-04 | T-01/02，双语 alt/caption/steps | 与六张源图逐一对照：小组件启动最小化、三游戏名、编辑器半屏布局；日历演示状态不称当天，音乐不称正在播放；pass |
| AC-05 | T-01/03，Gallery/EvidenceImage 复用 | 60 组合几何、加载尺寸，Enter/原图/Esc 回焦；既有方向键 gallery 用例保留；pass |
| AC-06 | T-03，tests、24 图、报告 | 最终构建/77 回归/24 图人工看图；无 UI 范围外修改，无新增债务或未授权延期；pass |

## 规范增量冻结

- SD-01：`docs/specs/website/project.md`，原 hash `ae221cbbadcaac30daef10976c002fcd08b169fbe9717a5bb33cfe6a4064b8cf` → `71e2ddeeaf83158593599a5652b8a80abf6f641e266300fcf4c8578e9196f7ba`。
- SD-02：`docs/specs/website/design/ui-presentation.md`，原 hash `b5bbbeee215fb26b856f33ed12fdf2479cd7bd215f3aa3d740fe2bd79580ae8d` → `25d3a9920fd5e60999fc88abc977fe8967bd9260b645c4c577c928f74a7dbdd7`。
- 按现行 merge 附随程序在 `docs/specs/website/plans.md` 加 PLAN-720 活跃链接行（hash `cb47183bc0416aec065a740815fcd5a30f06c914e5d75151703a806b732326ea`）；无新模块/ADR/GOAL，不虚构新增或替代组件。
- Specs 正文只记录已验证现状；未覆盖 PLAN-718 后续学习功能或 PLAN-719 架构文稿。索引再生无语义变化。

复核在实施会话中完成，未创建独立 agent；结论由已提交实现、源码 diff、源图、运行检查和落地视觉包重新建立。完整 revision-bound verdict 与合入检查点写共享 Plan §9。

## 发布与收尾边界

本次只合入网站源代码并提供本地生产构建预览，不部署公开站点。当前会话未注册 store-mediated Spec writer，`127.0.0.1:8080/api/specs` 连接拒绝；ledger 更新/归档/原 worktree 清理保留为后续检查点，不直接改 `.autoos/specs.json`。规范正文及测试证据随代码保留。

## 用户重拍修正（repair-1）

用户报告首批截图缺少桌面快捷方式，并明确要求重拍六图替换、合入 master。2026-10-01 14:18..14:23 更新了 01..04；人工查看四张新原图，桌面恢复多列快捷方式，Launcher 后方桌面同步更新。05 游戏、06 工作图与原批 hash 相同，整组六文件均同步并重新核对；上方来源表现在记录最新批次。旧截图与首批页面视觉包可由 `19bbbdeeab77e6c57aa4d7031c5fc3be14eb23f5` 恢复，不将旧 77 项测试伪称为新图片测试。

| 文件 | 首批 SHA256（仅历史溯源） |
|---|---|
| 01-desktop-light.png | 3126c9fd652d05442dce2f61c71bbb1d152b0d5eeea861324e34b42521eed04b |
| 02-desktop-dark.png | 552120488abc87b3ea471114505e07054d7b128053415ae137b9f2f443ce5d40 |
| 03-launcher-light.png | a143c62d07c47270a9e0f33cb3390befd71a81f2911e17b5760a4389b2e0f166 |
| 04-launcher-dark.png | 6e5974124f3b337a58d46e6d6d749e229593c3fce57fe4699c4504101e660622 |

修正仅资产与证据，Vue/数据/路由/正文/测试断言和 canonical Spec 均不变；无需新语义修订。受影响验收重新验证：六图 source hash/尺寸、新构建、既有展示与发布页图片/console 用例、60 组合/24 图更新及人工看图。未触及网站行为的旧完整回归保留为已识别基线，Category A 不运行 Cargo/docs_gen。
