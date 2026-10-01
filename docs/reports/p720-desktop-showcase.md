# PLAN-720 — 网站桌面截图展示

## 来源与保全

用户提供 `D:/autostack/auto-os/desktop-shots/showcase`，捕获日 2026-10-01。六张原图均人工查看；本仓复制到 `website/public/desktop-showcase/`，逐一 SHA256 与来源一致，无裁切、重绘或重新编码。画面只证明捕获时的布局，不证明所有应用功能、跨端等价、音乐实际播放或日历为当天。

| 原文件与入库文件名 | 尺寸 | SHA256 |
|---|---|---|
| 01-desktop-light.png | 2560×1600 | 3126c9fd652d05442dce2f61c71bbb1d152b0d5eeea861324e34b42521eed04b |
| 02-desktop-dark.png | 2560×1600 | 552120488abc87b3ea471114505e07054d7b128053415ae137b9f2f443ce5d40 |
| 03-launcher-light.png | 2560×1600 | a143c62d07c47270a9e0f33cb3390befd71a81f2911e17b5760a4389b2e0f166 |
| 04-launcher-dark.png | 2560×1600 | 6e5974124f3b337a58d46e6d6d749e229593c3fce57fe4699c4504101e660622 |
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

验证与复审记录在完成后追加，不将源码检查充当视觉或交互验证。
