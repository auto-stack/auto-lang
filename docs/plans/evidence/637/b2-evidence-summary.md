# PLAN-637 B2 批次证据摘要（2026-09-17）

批次：B2 = T-03 W2 波（006/007/008/009/010/012/014，含 B1 已前置 T-01b 的 006/010）。
worktree：`D:/autostack/.wt/lang-637/auto-lang`（plan-637-dev）。
截图命名：`<demo>-<vue|vm>-before-b2.png`（改写前）、`-phaseA.png`（007/008/012
Phase A 中间态）、`-after-b2.png`（终态）。006/010 的 before-b2 复用 B1-after
（T-01b 后状态未变，B2 前无其他改动）。

## 1. Phase A 零漂移对拍（大面三件，Phase A 状态经 git 精确重建后双端实拍）

| demo | Phase A 提取 | vue | vm |
|---|---|---|---|
| 007 | 12 配方 88 位点（含漏盘补提取 panel_card ×2、member_stats_row ×3） | IDENTICAL | IDENTICAL |
| 008 | 4 配方 24 位点 | IDENTICAL | IDENTICAL |
| 012 | 12 配方 48 位点 | IDENTICAL | IDENTICAL |

## 2. 终态对拍与归因

| demo | Phase B 要点 | vue | vm | 归因 |
|---|---|---|---|---|
| 006 | 亮支 bg-gray-50/text-gray-900→bg-background/text-foreground；暗支 text-white→text-primary-foreground（渐变豁免保留） | DIFF 1.20%（bbox=hero 文案区） | IDENTICAL | VM 暗主题下 primary-foreground 即纯白，暗支换白不可分；亮支未在 dark 截图呈现 |
| 007 | gray-900→foreground、gray-500→muted-foreground、bg-white→bg-card、border-gray-100→border-border、页底 bg-gray-50→bg-background；**stylekit 消费：hint_sm→caption_text ×12、stat_value→section_title(text-sm) ×21**；类目色 chip/avatar ×11 豁免 | DIFF 97.97% | DIFF 98.11% | 页底字面量→背景 token 全屏翻转 + 全部文字灰阶随主题，逐条对映射表；结构/布局完好（截图目检） |
| 008 | zinc/gray 明暗臂按角色收敛（zinc-100/gray-900→foreground；zinc-400/gray-500→muted-foreground；zinc-300/gray-700→foreground；卡片→bg-card/border-border；plan3 按钮→secondary 族；页根→bg-background 双臂全同） | DIFF 97.20% | DIFF 97.15% | 全局文字灰阶随主题 + 卡面 token 化；RECOMMENDED 徽章/primary 按钮原已 token 不变 |
| 009 | 三文章卡：同 008 收敛口径 + 缩略图 bg-muted/border-border/50 + hover:border-foreground/20；Phase A 提取 art_tag/read_more/thumb_emoji/content_col 各 ×3 | DIFF 99.88% | DIFF 99.71% | 页根+全部文字随主题；卡面 bg-card |
| 010 | 同收敛口径（含眉标签 ×3→muted-foreground、卡面 ×2→bg-card）；emerald 成功回执簇 ×7 豁免登记；Phase A field_col ×3 | DIFF 91.67% | DIFF 96.75% | 页根+卡面+标签；emerald 区不变（豁免） |
| 012 | 状态按钮：开始 green-500→primary、停止 red-500→destructive、暂停 yellow-500→secondary（text-white→对应 -foreground） | DIFF 0.41%（bbox=开始按钮区，恰为映射点） | 空窗（见 §4） | Vue 侧差异 bbox 与映射按钮逐一吻合 |
| 014 | bg-blue-500/text-white→primary 族；蓝卡上 text-blue-100/white/200→text-primary-foreground（/80・/70）；tab_idle→secondary 族；页底→bg-background；渐变豁免 | DIFF 27.20% | DIFF 26.28% | 蓝卡文字降档+tab 按钮+页底，渐变不变 |

## 3. 机制与门禁

- **stylekit 消费增量**：007 双配方消费（caption_text ×12 位点、section_title ×21
  位点，后者经 `style stat_value = section_title(size: "text-sm")` 声明级派生——
  Phase B 归一后体序全等）。AC-05 累计 3 demo（045/015/007）。
- **title_lg 不消费 section_title 的原因**：源串词序（text-lg 在前）与 recipe 输出
  序不同，字节等价不可达；词序敏感性未验证，不冒险（注记在 recipe 头注）。
- **guard**：manifest completed 增至 15；新增豁免 007 类目色 ×8 键、010 emerald ×7 键。
  过程中发现并修复守卫缺口：行尾注释中的旧字面量曾误报——改用「空白+//」边界剥
  行尾注释（保护 URL 的 ://），自测补 mapped.at/URL 两用例，5/5 绿；正例 15/15 PASS。

## 4. 偏差与勘正记录

1. **fit 窗 demo 的 VM 像素证据退化**：003（B1）与 012（B2）的 VM 截图为
   部分/未绘制帧（fit 窗 hidden 测量态→显示竞态；012 三帧全空窗 400×1140，
   003 半绘 400×720）。双侧同等欠绘 → IDENTICAL 退化为「空对空」，不构成
   风格证据。两 demo 的样式主张由 Vue 像素对（003 零漂移 / 012 差异 bbox=映射
   按钮）+ MCP rendered 结构树承担。verifier 截图时序修复列为已知债（不入本计划）。
2. **006/010 before-b2 复用 B1-after**：两 demo 自 B1 T-01b 后无改动，字节级
   同状态，复用成立。
3. **重复串盘点口径**：单类短串（items-center ×18、gap-0 ×7、w-full 等）不提取
   ——「长串」解释为 ≥2 class；过度抽象禁令（AC-05 r2 注记）优先。
4. **首轮盘点截断漏盘**：007 的 p-5 卡片变体（×2）与 gap-4 mt-3（×3）因 head
   截断漏看，复查全量清单后补提取；其余 demo 均经全量重扫确认无漏。
5. **收敛后条件结构保留**：明暗双臂 token 收敛为同串时保留 if/else（结构简化
   不属本计划授权面）；008 页根/特性行、009 页根等双臂已同串。
6. **012 停止按钮→destructive、暂停→secondary 的语义裁定**：开始=主行动
   （primary）、停止=中止（destructive）、暂停=次行动（secondary），映射表在
   源文件头注。

## 5. AC 进度（B2 后）

- AC-01：Phase A 完成 15/33（+007/008/009/010/012/014，006 无重复串）。
- AC-02：token 化完成 15 demo 中除豁免外清零（006/007/008/009/010/012/014 入账）。
- AC-05：消费 3 demo（045/015/007），位点 26（5+21）。
- AC-06：guard 15/15 PASS（自测 5/5）。
- 剩余：B3（10 demo 轻半）、B4（5 demo 巨型+tree_icon）、B5（重型+收口）。
