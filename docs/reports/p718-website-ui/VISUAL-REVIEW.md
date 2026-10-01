# PLAN-718 T-07 视觉复核记录

复核人：视觉验收代理 ×2（阅读矩阵 / 原矩阵关键子集），2026-10-01。
截图：本目录 `final-*`（原矩阵 96 张）与 `reading/final-*`（阅读矩阵 32 张），
manifest 为各自目录的 `final-report.json`（含 URL/locale/正文断言与几何测量）。

## 阅读矩阵（reading/final-*.png，32 张）

**32/32 pass**。要点：

- 四个 hub（docs/books × EN/ZH）学习卡组完整，主书卡片紫色高亮置顶，深浅主题对比正常。
- 390px 顶部为统一导航条，其下 Menu/目录条与"此页内容"同行并列，无叠压（T-03 修正生效）。
- zh UI chrome 全中文（目录/此页内容/上一页/下一页/编辑此页/运行），en 全英文；无 404。
- 观察项（不构成 fail）：代码块长行按 overflow-x 容器内滚动（VitePress 标准行为）；
  1440 dark 长图有全页截图滚动条残影；zh 侧栏残留英文条目属翻译完备度（内容计划范畴）；
  TOC 长条目省略号为设计内截断。

## 原矩阵关键子集（final-*.png，32 张）

**30/32 pass**。要点：

- **715 中文证据缺口未复发**：20 张中文页全部落在真实 `/zh/` 界面（715 的 zh 截图因
  `/zh/zh/` 双前缀全部为 404，已在本计划 T-01 修正脚本并以本批图为首次有效中文视觉基线）。
- docs/books hub 学习入口卡组在深浅/双视口下清晰可见。
- **fail（4 张，同一既存缺陷）**：`final-playground-{zh,en}-1440-{dark,light}.png` 的
  hero 大标题 "Playground" 字形顶部被水平裁切。**已核对 715 已入库的
  `docs/reports/p715-website-ui/final-playground-en-1440-dark.png`——同样裁切，属
  715 前既存缺陷，非本计划引入**；本计划未触及 /playground 页面样式，按"不静默扩大
  计划"纪律记入 KNOWN-DEBT-AND-RISKS，修复建议：hero 标题 line-height/overflow
  裁剪（一处 CSS），留待独立修复。
- 次要观察（未计 fail）：en 版 Playground 语料树等组件文本为中文（组件级 i18n 缺口，
  既存）；1440 长图底部留白属全页截图正常现象。

## 历史图片失效范围（715）

失效：`docs/reports/p715-website-ui/final-*-zh-*.png`（zh 页 404，共 24 页 × 2 视口 ×
2 主题中的 zh 半边 = 48 张）——被本计划 `baseline/` 与 `final-*` 的对应 zh 图取代。
有效：715 的 EN 图与其结论不受影响；本计划不覆写 715 目录，失效范围以此说明为准。
