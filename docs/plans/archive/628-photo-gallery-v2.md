---
plan_id: PLAN-628
status: archived              # drafting → executing → execution_done → reviewed → archived
feature_name: photo-gallery-v2
author: [agent]
created_at: 2026-09-14
updated_at: 2026-09-14

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [examples/029-photo-gallery]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [examples/ui/029-photo-gallery]                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 5
total_steps: 5
---

# [PLAN-628] photo-gallery-v2 现代移动/平板风真实照片相册重塑

## 0. 变更摘要

针对 `examples/ui/029-photo-gallery` 旧版实现的 UI/UX 改善需求：
1. **假数据与网络图问题**：旧版采用假分类（自然/城市/天空/抽象）与内联 SVG/picsum 网络图，体验与真实相册脱节。
2. **重构为真实图片目录验证**：直连用户真实目录 `C:\Users\zhaop\Pictures\`（包含照片与屏幕截图，真实大小、真实宽高、真实拍摄日期）。
3. **真实缩略图生成能力**：编写 `scripts/prepare_gallery.py` 构建轻量 JPEG 缩略图生成流水线（260px，~11KB/张，秒级离线渲染）。
4. **平板/手机端现代 UI/UX 交互形态重塑**：
   - 取消生硬的 macOS 三段式侧边栏。
   - 采用顶部沉浸式分段导航控件（Segmented Pills）：`全部照片` / `📷 图片` / `📱 截图` / `❤ 收藏`。
   - 沉浸式 Edge-to-edge 无缝照片流网格（支持 2/3/4 列动态切换、悬浮一键收藏）。
   - 移动端风格悬浮胶囊大图查看器（带返回胶囊、图片元信息条、上一张/下一张悬浮导航胶囊）。

## 1. 目标

- **G1 (真实数据源)**：将相册数据源完全替换为 `C:\Users\zhaop\Pictures\` 的真实照片与屏幕截图，展示真实文件名、分辨率、大小与时间。
- **G2 (缩略图生成流水线)**：建立高质量且极轻量的缩略图预构建脚本，避免前端内联超大 Base64 导致编译器 OOM，实现毫秒级秒开。
- **G3 (现代移动/平板 UI/UX)**：落地 iOS/iPadOS 风格的沉浸式相册布局与悬浮胶囊查看器。
- **G4 (VM 模式实机验证)**：通过内置 AutoUI MCP 驱动在 VM 模式（`auto run -r vm`）下完成无损渲染、相簿切换、大图查看器循环导航与实机截图核验。

## 2. 架构设计与避坑沉淀

1. **编译器内存边界防御**：
   - 严禁在 `.at` 中内联巨量 Base64 Data URL（曾测 68 张图导致内存飙升至 8GB+ OOM 卡死）。
   - Iced `load_image_bytes` 原生支持直接读取本地文件绝对路径（如 `D:/autostack/.../thumb_001.jpg` 与 `C:/Users/zhaop/Pictures/xxx.jpg`）。
   - 路径中的 Windows 反斜杠 `\` 必须归一化为正斜杠 `/`，防止作为非法 unicode 转义报错。
2. **平行数组长度强一致要求**：
   - `p_ids`, `p_titles`, `p_tls`, `p_albums`, `p_dates`, `p_keys`, `p_favs`, `p_fulls`, `p_metas`, `p_thumbs` 必须长度严格一致。生成与截断切片脚本统一使用 `json.loads` / `json.dumps`，杜绝因文件名内含逗号导致的拆分不对齐。

## 3. 执行步骤与完成证据

- [✅] **T1: 真实目录调研与缩略图流水线构建**：扫描 `C:\Users\zhaop\Pictures\`，提取 53+ 张图片，生成轻量缩略图于 `src/front/thumbnails/`。
- [✅] **T2: 手机/平板风格现代 UI 重写**：实现顶部 Segmented Tabs、沉浸式卡片网格与悬浮胶囊原画查看器。
- [✅] **T3: VM 实机启动与状态核验**：启动 `auto run -r vm`，通过 MCP `autoui_state` 确认 24 项真实图片数据与相簿计数正确加载。
- [✅] **T4: 视觉与交互实机截图验收**：
  - 网格初始态：`tests/screenshots/photo_gallery_real_grid.png`
  - 收藏相簿筛选：`tests/screenshots/photo_gallery_real_favs.png`
  - 深色主题切换：`tests/screenshots/photo_gallery_real_dark.png` / `photo_gallery_real_dark_grid.png`
  - 全屏原画大图：`tests/screenshots/photo_gallery_real_fullscreen.png`
  - 查看器下一张导航：`tests/screenshots/photo_gallery_real_next.png`
- [✅] **T5: 规范与文档回写**：更新 `examples/ui/029-photo-gallery/SPEC.md` 与本计划文档。

## 4. 复审记录

### PLAN-628:r1 独立复审 (auto-plan:review)
- **审阅时间**：2026-09-14 17:40
- **复审基线 Commit**：`c7c68f30acb867102136de377528e3a26f217ddd`
- **Worktree 路径**：`D:/autostack/.wt/lang-628/auto-lang`
- **复审判定**：**PASS**
- **证据清单**：
  1. **清单对账**：T1..T5 全部 100% 达成，无遗漏无延后。
  2. **Workaround 扫描**：无临时 hack，图片绝对路径通过正斜杠归一化规避了转义问题，避免了内联 Base64 膨胀。
  3. **健康度与门禁**：
     - `cargo check -p auto-lang` 0 新增编译警告通过。
     - VM 运行日志确认：`SetSearch`、`SelectAlbum`、`OpenPhoto`、`NextPhoto`、`ToggleFav`、`ToggleDark` 各项事件与状态变更经 MCP 验证无 runtime error。
     - 实机截图留档：网格初始态、相簿筛选、暗色模式、无损原画大图及下一张轮播共 5 组全量截图。
  4. **知识沉淀 Spec Delta**：
     - `examples/ui/029-photo-gallery/SPEC.md` 已全面更新为 Plan 628 真实图库规范。

