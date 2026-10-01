---
plan_id: PLAN-720
status: drafting
feature_name: 网站虚拟桌面实景展示
author: [agent]
created_at: 2026-10-01
updated_at: 2026-10-01

plan_revision: 1
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [website]
current_step: 0
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

- [ ] AC-01 六图原样入库，有源路径、捕获日、尺寸、SHA256。
- [ ] AC-02 v05/autoos 双语顺序为桌面双主题→Launcher→游戏→工作；OS 预览进入正确语言专题。
- [ ] AC-03 单一作者数据、六路由无死图/裸组件源文；原 URL、统计与其他章节保留。
- [ ] AC-04 小组件前提、游戏名称、工作布局描述与画面一致，不虚构音乐播放/日历当前日期。
- [ ] AC-05 五宽度×双主题无横向溢出，图片键盘放大/原图/Esc 回焦、尺寸占位与延迟加载正常。
- [ ] AC-06 build、既有与新增网站 e2e 通过；24截图落地/正文有效、看图无遮挡，无新警告或范围外修改。

## 8. 执行步骤

- [ ] T-01 入库与共享展示：复制六 PNG，新增 data/desktop-showcase.ts、components/DesktopShowcase.vue，报告 docs/reports/p720-desktop-showcase.md 记录源证据；核对 hash/尺寸。AC-01/04/05，SD-01/02。
- [ ] T-02 页面接线：修改 ReleaseLanding.vue/release-v05.ts、autoos/index.md、zh/autoos/index.md、os.md、zh/os.md，双语接线与首图/旧图替换；prepare/build/源码核查。AC-02/03/04，SD-01/02。
- [ ] T-03 验证交付：新增 tests/desktop-showcase.spec.ts，24图入 docs/reports/p720-desktop-showcase/；网站 e2e、逐项 AC/SD 复核，review 后按已授权网站合入范围落 master。store writer 不可用时 ledger pending、保持 reviewed，不提前归档。AC-01..06。

## 9. 复审记录

stage: new | plan_id: PLAN-720 | plan_revision: 1 | outcome: pass (contract) | next: work。T-01..03 覆盖 AC-01..06、SD-01/02，用户已明确授权本次截图更新，不代表实现完成。

## 10. 待澄清事项

无阻塞偏好；采用站点现有视觉与用户截图顺序，保留其他会话 WIP。
