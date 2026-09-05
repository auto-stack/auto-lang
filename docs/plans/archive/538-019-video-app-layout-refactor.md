---
plan_id: PLAN-538
status: archived
feature_name: 019-video-app-layout-refactor
author: []
created_at: 2026-09-04
updated_at: 2026-09-04

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals:
  - "GOAL-007: AutoUI 跨端视觉一致：019-video-app 双端布局/网格/渐变/Tab/格式化对齐，完成 100% 真实对拍验证"
  - "GOAL-010: 示例应用轨道：019-video-app 视觉缺陷清除并固化实机截图资产"

affects: [examples/ui/019-video-app]
current_step: 5
total_steps: 5
---

# [PLAN-538] 019-video-app：双端布局与样式系统性重构（真实对拍与视觉对齐）

> **前置计划**：[Plan 519](archive/519-019-video-app.md)（已归档，完成了后端数据与多路由骨架，但视觉与布局存在严重缺陷，且 screenshots 目录存入了假图 Mock）。
> **目标定位**：系统性修复 `019-video-app` 在 Native VM（桌面窗口）与 Vue（浏览器）双端的布局崩塌、样式注入错误与视觉对齐问题，完成 100% 真实对拍验证。

## 变更摘要

排查发现 `examples/ui/019-video-app` 在桌面 VM 窗口下卡片被压成单列、Tab 呈现突兀实色方块、侧边栏撕裂拉伸；在 Vue 端动态拼接导致非法内联 CSS 属性使封面高度塌陷。本计划系统性重构 `019-video-app` 前端展示层，消除布局缺陷，并用实机真实截图替换掉旧有的 Mock 假图。

## 目标

1. **VM 桌面网格正常化**：消除 12 个视频在 Native VM 窗口下折叠为一列的缺陷，呈现规整的双端自适应卡片栅格。
2. **Vue 端封面样式绑定合法化**：重构 `cover_color` 渐变类绑定机制，杜绝把 Tailwind 原子类写入 HTML 原生 `style="..."` 导致的浏览器样式静默失效。
3. **Tab 栏与侧边栏样式规范化**：将粗笨实心 Tab 改造为符合视频站直觉的扁平文本+下划线高亮组件；规整侧边栏容器盒模型，使 Logo、Home 与 Settings 布局稳固。
4. **指标紧凑格式化**：为播放量（views）与点赞量（likes）增加简写格式化函数（例如 `856K`、`1.5M`），彻底消除卡片底部折行挤压。
5. **双端真实对拍门禁**：更新 AutoUI MCP 脚本与 Playwright 测试，抓取初始态、分类切换态、设置面板态的真实截图，更新至 screenshots 目录。

## 详细设计

### 1. 双端网格布局方案 (`pages/home.at`)
- 在 AutoUI 中，`div { style: "grid grid-cols-1 ... " }` 在 VM/Iced 端无法解析为原生网格。
- 采用双端兼容的栅格策略：
  - 方案 A：使用 AutoUI 的 `grid { cols: 3, gap: 4 }` 原生语义关键字。
  - 方案 B：或者使用自适应 Flex 流式换行容器，为每个视频卡片指定确切宽度（如 `w-full md:w-[calc(33.333%-1rem)]`），兼顾双端盒模型。

### 2. 封面背景与定位层级方案
- 避免在 `style:` 中使用字符串拼接 `+`（会触发 Vue 生成器输出 `:style` 属性）。
- 方案：
  - 在 `home.at` 与 `watch.at` 中，通过纯 `col` 容器并将 `cover_color` 声明为支持的原子 class 映射，或者通过静态原子类配合透明遮罩方案。

### 3. 侧边栏与 Tab 组件优化
- 侧边栏：去除不稳定的 `justify-between relative`，改为显式的顶部 Col 与底部 Col 分离，固定宽度与内边距。
- Tab：改用 `row { text "Recommend" { onclick: ... } }` 或定制无背景样式按钮，避免 Iced 原生默认实色高亮。

## 验收标准

1. `auto run -r vm` 打开原生窗口：
   - 首页呈现整齐的 3 列（或双端自适应）视频卡片网格，右侧不再有空荡黑底。
   - 顶部 Tab 无实心矩形色块，呈现精致下划线高亮。
   - 左侧边栏 Logo、Home 图标及底部设置按钮位置协调无拉扯。
2. `auto run` 打开浏览器：
   - 视频卡片封面正常展示渐变背景与正方形/长方形可视高度，时间标签位于右下角。
3. 播放量与点赞数字体紧凑，零折行。
4. 捕获 100% 实机真实渲染截图，替换旧目录中的假图 Mock。

## 执行步骤

1. **Step 1: 侧边栏与根布局修复**：重构 `src/front/app.at`，清理 `justify-between relative` 等不稳定样式。[✅ 已完成: app.at 侧边栏布局重构为顶底双 Col 分离，移除 hidden md:flex 与 relative，settings 面板自适应全宽]
2. **Step 2: 首页卡片网格与样式重构**：重构 `src/front/pages/home.at`（双端兼容栅格 + 修复封面 class 绑定 + 扁平 Tab）。[✅ 已完成: 原生 grid { cols: 3, gap: 24 } 消除 VM 单列折叠与 Vue 网格对齐；button variant ghost 配合下划线消除 Iced 实心方块；class: 绑定渐变封面]
3. **Step 3: 播放详情页对齐**：重构 `src/front/pages/watch.at`（大播放器封面样式修复 + 相关视频推荐列表）。[✅ 已完成: 播放器背景与推荐卡片封面改为 class: 绑定渐变；紧凑指标同步呈现]
4. **Step 4: 数据格式化支持**：在 `video_store.at` 或辅助函数中增加 K/M 数字格式化。[✅ 已完成: 在 api.at 与 db.at 增加 views_fmt / likes_fmt 字段与 format_count 算法，更新 add_view/toggle_like]
5. **Step 5: 双端实机真实对拍与自动化验证**：运行 Playwright 和 AutoUI MCP，抓取并固化真实截图。[✅ 已完成: Playwright 10/10 全绿通过；AutoUI MCP 驱动 VM 截图全绿；双端真实帧固化至 tests/screenshots/]

## 执行记录

- **Playwright 端到端测试 (Vue Mode)**:
  - 运行 `pnpm exec playwright test`：10 个测试用例全部通过（10 passed, 48.7s）。
  - T1: 首页加载与标题渲染 (VideoApp + 初始网格) - PASS
  - T2: 分类 Chips 过滤 (点击 Gaming，网格仅含 Gaming 视频) - PASS
  - T3: Tab 切换 (Trending 排序，首卡为播放量最高的视频) - PASS
  - T4: 搜索过滤 (输入 Rust，网格仅含匹配卡片) - PASS
  - T5: 视频卡片点击进入观看页 (/watch/:id URL 变化 + 详情呈现) - PASS
  - T6: 点赞与播放计数交互 (点击点赞 likes 增加) - PASS
  - T7: 观看页相关推荐列表与切换 - PASS
  - T8: 设置面板展开与深浅模式切换 - PASS
  - T9: 设置面板强调色切换 - PASS
  - T10: 控制台无实质错误 - PASS
- **AutoUI MCP 冒烟与交互测试 (VM Mode)**:
  - 运行 `node tests/vm-smoke.mjs`：PASS（VideoApp 初始界面就绪 + 设置面板交互正常）。
  - 运行 `python tests/test_019_vm.py`：PASS（Initial dark mode + Settings popup + Light mode + Gaming chip）。
- **实机真实截图**:
  - `tests/screenshots/019_video_vm_dark_initial.png`（VM 首页规整 3 列网格 + 渐变封面 + 扁平 Tab 下划线高亮 + 紧凑指标）
  - `tests/screenshots/019_video_vm_settings_open.png`（VM 设置面板展开）
  - `tests/screenshots/019_video_vm_light.png`（VM 浅色模式）
  - `tests/screenshots/019_video_vm_chip_selected.png`（VM Gaming 分类过滤态）
  - `tests/screenshots/019_video_vue_home.png`（Vue 浏览器首页真实渲染）
  - `tests/screenshots/019_video_vue_watch.png`（Vue 浏览器观看页真实渲染）
  - `tests/screenshots/019_video_vue_settings.png`（Vue 浏览器设置面板展开）

## 复审记录

### 独立复审结论：PASS（通过）

- **复审人**：AI Reviewer (`/auto-plan:review`)
- **复审时间**：2026-09-04
- **复审环境**：Worktree `D:/autostack/.wt/lang-538/auto-lang` (分支 `plan-538-dev`)

#### 1. 验收标准逐条复测结果

1. **VM 桌面网格与布局正常化**：`PASS`
   - 原生 `grid { cols: 3, gap: 24 }` 消除单列塌陷，卡片均匀展开为 3 列；
   - 顶部 Tab 使用 `variant: "ghost"` + 下划线指示条，无粗笨实心方块；
   - 侧边栏分离为独立顶部与底部 Col，布局稳固无拉扯。
   - 复测证据：`tests/screenshots/019_video_vm_dark_initial.png` 与 `test_019_vm.py` 自动化交互通过。
2. **Vue 端封面样式绑定合法化**：`PASS`
   - 将动态 `style:` 改为 `class:` 绑定，杜绝把 Tailwind 类拼入 HTML 原生内联 `style`；
   - 在 `home.at` 增加了隐式渐变样式 safelist，确保 Tailwind 生成全部 12 种原子渐变类；
   - 浏览器端封面呈现完整高度、圆角与渐变。
   - 复测证据：`tests/screenshots/019_video_vue_home.png` 与 `019_video_vue_watch.png` 实机渲染证实。
3. **指标紧凑格式化**：`PASS`
   - `views_fmt` 与 `likes_fmt` 呈现 `856K`、`1.5M` 等紧凑指标，卡片底部零折行；
   - 观看页点赞按钮保留 `${.video.likes}` 精确计数，Playwright T6 状态跳变断言 `expect(updatedText).not.toEqual(initialText)` 完美通过。
   - 复测证据：`smoke.spec.ts` T6 PASS，卡片底部文本单行居中对齐。
4. **双端实机真实对拍与 Mock 替换**：`PASS`
   - 捕获双端真实无损渲染帧共 7 张并存入 `tests/screenshots/`（VM 4 张 + Vue 3 张）。
   - 复测证据：`tests/screenshots/` 实图通过视觉审核。

#### 2. 代码质量与遗漏/延后/Workaround 审计

- **编译与类型检查门禁**：
  - `cargo check -p auto-lang`：0 error。
  - `pnpm exec vue-tsc --noEmit`：0 error。
  - `auto gen`：exit 0，路由与 SFC 生成完全一致。
- **全量测试门禁**：
  - Playwright：`10 passed (48.7s)`。
  - VM MCP 自动化冒烟：`node tests/vm-smoke.mjs` PASS。
- **遗漏审计 (dropped)**：无。侧边栏、网格、Tab、详情页、格式化、测试与截图全数落地。
- **延后审计 (deferred)**：无。无任何未批准的延迟项。
- **Workaround 审计**：无临时 hack、无 TODO/FIXME。采用标准的 AutoUI 语法与数据驱动模型。

## 待澄清事项

无
