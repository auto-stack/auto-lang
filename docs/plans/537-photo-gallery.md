---
plan_id: PLAN-537
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: photo-gallery
author: [zhaopuming]
created_at: 2026-09-04
updated_at: 2026-09-04

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/examples]  # 纯示例层新增，不动 crates/
current_step: 0
total_steps: 9
---

# [PLAN-537] photo-gallery — 029 图库应用（examples/ui 填洞 029）

## 变更摘要

examples/ui 新增 **029-photo-gallery**（图库 / Photos）：macOS 相册风图片浏览器
demo——侧边栏相册导航 + 搜索/排序/网格密度工具栏 + 响应式缩略图网格 + 大图
查看器（上一张/下一张/收藏）。**全部基于既有 `image` widget 真实加载网络图片**
（picsum.photos 固定 seed），是该 widget 首个应用级双端示范（现有 019/020 媒体
demo 均为渐变色块模拟封面）。要求四形态可运行：Vue 独立（`auto run`）、VM 独立
（`auto run -r vm`）、虚拟桌面宿主（`ui_desktop` 自动扫描，零登记）。编号按
README 约定回填空洞 **029**（025/026/027/028 同模式回填先例）。

## 目标

1. **G1 应用形态**：单 App 组件图库（网格 + 查看器两视图状态切换），AutoOS
   Dark 默认主题 + 语义 token 配色，交互完整可演示。
2. **G2 双端一致**：Vue 与 VM(Iced) 同一 `.at` 源、同一外观与交互语义；遵守
   025/027/028 实测沉淀的"单组件约束"（详见需求分析③）。
3. **G3 桌面入驻**：`ui_desktop` 全屏桌面出现图库图标并可 LaunchApp 打开
   （app_registry 自动扫描，零登记成本）。
4. **G4 image widget 应用级示范**：24 张真实网络图片（缩略图/大图两档 URL），
   离线时优雅降级（VM 首字母色块兜底 / Vue alt 文本）。

## 架构方案

**形态选型——单文件单组件（025/027/028 形态）**：全部状态内聚 `src/front/app.at`
的 App 根 widget，不使用 routes/outlet/pages、不使用 store 子组件、不使用模块级
fn。理由：routes 的 VM 支持虽存在（`aura_view_builder.rs:121` Plan 401/VM-routing），
但 routes+pages+store 的组合形态从未在双端验证过（018/019/022 路由用户全部
vue-only）；而 024/027/028 三个 vue+vm 双端实测 demo 全是单组件形态。查看器用
`var mode str = "grid" | "view"` 全页条件切换（028 overlay 门控同款），不走路由。

**图片源选型——网络 URL（picsum.photos 固定 seed）**：四种运行形态中唯一全
成立的方案（Vue 浏览器原生加载；VM reqwest 阻塞下载+缓存 `renderer.rs:4763`
load_image_bytes）。备选方案排除理由：base64 data URL VM 不支持；本地相对路径
按进程 cwd 解析（VM 独立=cwd demo 目录 ✓，桌面宿主=cwd 仓库根 ✗，Vue Vite
不伺服应用目录静态文件 ✗）。004-profile-card 网络头像 URL（cravatar）为先例。
URL 规则：缩略图 `https://picsum.photos/seed/gal-{NN}/400/300`，大图
`https://picsum.photos/seed/gal-{NN}/1600/1200`——同 seed 同图、确定性、无 API key。

**数据形态——平行源列表 + handler 构建视图列表（027 实测形态）**：model 内
24 张照片以平行列表种子数据（p_seeds/p_titles/p_albums/p_dates）声明；Init
handler 构建完整 struct 列表 `photos`，ApplyFilter handler 产出过滤排序后的
视图列表 `view`（模板 `for item in .view` 字段访问双端验证，027 `files_view`
同款）。模板文本插值禁止 `.len()`/方法调用——所有计数由 handler 预计算为
str 状态（028 T1 盘点结论）。

## 需求分析与背景调查

（取材 docs/specs/overview + examples/ui README + 024/025/027/028 SPEC/源码实测盘点，2026-09-04）

**① 虚拟桌面接入（零登记）**：`crates/auto-lang/src/ui/app_registry.rs` 的
`scan_apps`（:65）扫描 `examples/ui` 一级子目录，入口探测 `probe_entry`（:275）
认 `app.at` → `src/front/app.at`；`parse_pac_fields`（:289）读平铺 pac.at 字段
（name/title/icon/category/render/daemon/back/window）。**建目录 + pac.at +
src/front/app.at 即被桌面收录**，无需改任何 Rust 代码。桌面宿主启动：
`cargo run -p auto-lang --features ui-iced --example ui_desktop -- --fullscreen
--apps-dir examples/ui`（ui_desktop.rs 文件头）。

**② image widget 能力面**：schema `crates/auto-lang/src/aura/schema.rs:772`
（props：src 必填/alt/class/fit∈{cover,contain,fill,none}）。Vue 侧发原生
`<img>`（vue.rs:7281 tag 映射，:12856 动态 `src:` 表达式发 `:src` 绑定）。VM
侧真实加载（renderer.rs:4472-4757）：http/https reqwest 阻塞下载、本地文件、
`builtin:` 内嵌壁纸；Handle 缓存防闪烁（get_or_create_image_handle）；失败兜底
首字母色块（:4734-4753）；**base64 data: URL 不支持**。`.svg` 走 svg widget。

**③ 双端"单组件约束"（028 app.at 文件头 T1 盘点，硬约束）**：
- store 子组件 vue TS 生成损坏（013/038 实测）→ 状态全内聚 App；
- 模块级 fn 不进 vue SFC（024 先例）→ 逻辑写状态法 handler；
- 模板文本插值不能调 `.len()` → 计数在 handler 算成 str/数值状态；
- lucide 图标 Vue 侧只收集**静态名字**（028 SPEC.md:71）→ `icon (name:)`
  一律静态字面量；动态需求（收藏实心/空心）用文本符号 ♡/❤ 或 if 切换两枚
  静态图标。侧边栏相册项手写展开（6 项，不用 for），保静态图标名。

**④ VM 布局能力**：CSS grid 像素级精确仿真（renderer.rs:2644，equal tracks
默认 + col-span）；`grid-cols-N` 双端安全。滚动容器 `overflow-y-auto`（027
:356 主区实测）。`hover:` 前缀类 Vue 增强、VM 忽略——样式不能只靠 hover 表达。

**⑤ 编号/端口/主题约定**：README「编号说明」新示例优先填空洞（029-037 空），
填洞加历史注记；front_port 唯一（4024-4028/4038/4041/4043 已占，取 **4029**）；
主题 Dark+indigo 默认，运行时切换契约 = 根组件声明 `dark_mode`(bool)/
`accent_color`(str) 状态变量（变量名即契约，双端识别）。

**⑥ 现状差距**：README 示例总览无图片浏览器类应用；019-video-app / 020-
music-player 封面均为渐变色块+emoji 模拟，全生态无一个真实图片加载的应用级
demo——本计划补位，同时为 image widget 提供应用级回归载体。

## 详细设计

### 目录与登记

```
examples/ui/029-photo-gallery/
├── pac.at              # name/version/scene/render/front_port/icon/title/category
├── SPEC.md             # 数据形状 + 排序/过滤规则 + 双端差异注记（028 惯例）
└── src/front/app.at    # 单组件全应用
```

pac.at 字段：`name: "photo-gallery"`、`render: "vue"`（开发目标端声明；VM 兼容
由实机验收兜底，041/027 注释先例）、`front_port: 4029`、`icon: "images"`、
`title: "Photo Gallery"`、`category: "media"`。

README 示例总览表加 029 行 + 编号历史注记（029 原能力样板已迁 capability-tests，
本计划回填）。

### 数据模型（model 块）

- 种子平行列表（24 项）：`p_ids`(1..24)、`p_seeds`("gal-01".."gal-24")、
  `p_titles`（按相册主题命名的中文标题，如"晨雾山谷"）、`p_albums`（
  "nature"/"city"/"sky"/"abstract" 各 6）、`p_dates`（2026-05..08 错落）、
  `p_favs`（bool，预置 4 张收藏）。
- Init 构建 `photos`（struct：id/title/album/seed/date/fav/thumb/full/url 已拼
  好完整 URL 字符串，避免模板运行时拼接）。
- 相册表（静态 6 项，仅 handler/手写模板用）：all/favorites/nature/city/sky/
  abstract；中文标签 全部照片/收藏/自然/城市/天空/抽象。
- 视图状态：`view`（过滤+排序后的 struct 列表）、`view_label str`（"24 张照片
  · 全部"类组合文案，handler 预拼）、`cnt_* str` ×6（侧边栏计数）。
- 交互状态：`album str`（当前相册 key，默认 "all"）、`search_q str`、
  `sort_dir str`（"desc" 默认=最新在前 /"asc"）、`density str`（"2"/"3"/"4" 列
  → handler 映射 `grid_class str` = "grid-cols-2/3/4"）、`mode str`
  （"grid"/"view"）、`cur_id int`（查看器当前照片）、`cur_title/cur_meta str`
  （元信息预拼）、`prev_id/next_id int`（循环导航预计算）、`fav_label str`
  （查看器收藏按钮文案 ❤/♡）。
- 主题契约变量：`dark_mode bool = true`、`accent_color str = "indigo"`。

### msg / handler（状态变换，无模块级 fn）

`Init, SelectAlbum(str), SetSearch(str), ApplyFilter, ToggleSort, SetDensity(str),
OpenPhoto(int), PrevPhoto, NextPhoto, BackToGrid, ToggleFav(int), ToggleDark,
SetAccent(str)`

- `ApplyFilter`（核心，被 Init/SelectAlbum/SetSearch/ToggleSort/ToggleFav 复用）：
  按 album 过滤（"favorites" → fav==true；其余 → album 相等；"all" → 全量）
  → 按 search_q 子串过滤 title（小写比较用预小写字段，028 规避浮点/宽度差的
  整数/布尔判定同思路）→ 按 date + sort_dir 排序（date 为 "YYYY-MM-DD" 字符串，
  字典序即时间序，避免解析）→ 写 `view` + `view_label` + `cnt_*` 六计数。
- `OpenPhoto(id)`：mode="view"；在 `view` 内定位下标 → 预计算 prev/next（循环，
  单元素时 prev=next=自身）+ `cur_*` 元信息（相册中文标签 + 日期 + "1600×1200"
  + seed）。
- `PrevPhoto/NextPhoto`：`OpenPhoto(prev_id/next_id)` 的薄封装。
- `ToggleFav(id)`：翻转 photos 内该项 fav → ApplyFilter 重算（收藏计数联动）→
  若 mode=="view" 且 cur_id==id 则刷新 fav_label。
- `SetDensity`：density + grid_class 两状态。

### 视图（view 块）

根：`h-screen flex bg-background text-foreground overflow-hidden font-sans`，
按 mode 条件渲染两分支：

**网格分支**（mode=="grid"）：
- 左侧边栏 `w-56 border-r border-border flex-col`：标题行"图库 🖼 / Photos" +
  6 项手写展开（icon 静态名 images/heart/mountain/building-2/cloud-sun/sparkles
  + 中文标签 + `cnt_*` 计数徽标；选中态 `bg-accent/10 text-accent border-l-2`，
  019 分类 pill 的 if 双分支写法）。
- 主区 flex-col：工具栏（`input (placeholder "搜索照片…")` 搜索框、排序 toggle
  按钮"最新 ↓/最早 ↑"、密度按钮组 2/3/4、弹性空隙、dark 切换 icon
  sun/moon（if 切换两枚静态名）、accent 五色点行 coral/ocean/sage/amber/indigo）
  + `view_label` 计数条 + 网格区 `flex-1 overflow-y-auto p-4`。
- 网格：`div { style: "grid " + .grid_class + " gap-3"` 之类由 grid_class 状态
  驱动（handler 拼好完整 class 串存状态更稳：`grid_class` 直接存
  "grid grid-cols-3 gap-3"）；`for item in .view` 卡片：`rounded-xl overflow-hidden
  border border-border bg-card hover:border-primary/50 transition-colors
  cursor-pointer`，`image (src: item.thumb, alt: item.title, fit: "cover")` 于
  `h-40 w-full` 容器 + 下方信息行（title 截断 + ♡/❤ 收藏角标按钮，
  onclick stopPropagation 语义按 019/027 卡片点击先例处理——若无 stopPropagation
  原语则收藏按钮放卡片外独立列，以实机为准记入 SPEC）。
- 空态：无结果时 emoji 📷 + "未找到照片" + 清除搜索提示（019 空态同款）。

**查看器分支**（mode=="view"）：
- 顶栏：返回按钮（arrow-left icon + "返回网格"）、居中 `cur_title`、右侧
  `fav_label` 收藏切换按钮。
- 大图区 `flex-1 min-h-0 flex items-center justify-center bg-black/80 p-4`：
  `image (src: item_full, fit: "contain")`——cur 的 full URL 由 cur_full str
  状态承载（handler 预取，模板不做字段链查找）。
- 底栏：`cur_meta` 元信息行 + prev/next 按钮（chevron-left/right 静态 icon +
  "上一张/下一张"，循环导航；单张时禁用态文案）。
- 布局全部常规 flex/网格，**不用 absolute/fixed**（VM 定位支持面窄，019 的
  absolute 角标是 vue-only 用法）。

### 双端差异注记（写入 SPEC.md）

- 图片为网络加载：VM reqwest 阻塞下载（有缓存，首开稍慢）；离线时 VM 显示
  首字母色块兜底、Vue 显示 alt 文本——功能断言不依赖图片字节加载成功。
- `hover:` 类仅 Vue 生效；关键状态（选中/收藏）均有静态样式表达。
- 图片 `fit: cover/contain` 两端语义一致（schema 单一定义）。

## 测试设计

复用 `.agents/skills/autoui-verifier/scripts/` 标准脚本（AGENTS 规定不写临时
脚本）：

1. **Vue 独立**：`cd examples/ui/029-photo-gallery && auto run`（4029）→
   `test_vue_playwright.mjs` 冒烟：标题渲染、侧边栏 6 相册 + 计数、搜索过滤
   （输入后卡片数变化）、排序切换、密度切换列数、打开查看器、prev/next 循环、
   收藏联动计数、返回、dark/accent 切换。
2. **VM 独立**：`auto run -r vm` → `test_vm_mcp.py` 断言同上交互子集
   （网格渲染/相册过滤/查看器导航/收藏/主题）。
3. **桌面宿主**：`cargo run -p auto-lang --features ui-iced --example
   ui_desktop -- --fullscreen --apps-dir examples/ui` → 桌面出现 Photo Gallery
   图标，LaunchApp 打开，窗口内网格/查看器可用。
4. **回归护栏**：`app_registry.rs` 既有测试 `scan_examples_ui_finds_at_least_
   27_apps`（:320 附近）计数自动 +1 通过——若该断言是精确值需同步 bump（执行
   时确认）。

## 验收标准

- [ ] AC1 `examples/ui/029-photo-gallery/{pac.at,SPEC.md,src/front/app.at}` 存在，
      pac 声明含 front_port 4029 / icon images / category media。
- [ ] AC2 Vue 独立模式 `auto run` 启动无错，网格 24 张、相册过滤/搜索/排序/
      密度/收藏/查看器导航全部可用（playwright 冒烟通过）。
- [ ] AC3 VM 独立模式 `auto run -r vm` 启动，同一交互语义可用（desktop_mcp
      断言通过）；离线时图片兜底不崩溃。
- [ ] AC4 `ui_desktop --fullscreen --apps-dir examples/ui` 桌面出现图库并可
      LaunchApp 打开（实机截图在案）。
- [ ] AC5 README 示例总览表新增 029 行 + 编号历史注记；029 编号空洞已回填。
- [ ] AC6 双端"单组件约束"合规：无 store 子组件/无模块级 fn/模板无 .len()
      调用/图标 name 全静态字面量。
- [ ] AC7 未修改 `crates/` 任何 Rust 源码（纯示例层计划；若 AC4 护栏测试需
      bump 计数属例外，须在复审记录说明）。
- [ ] AC8 复审通过：验收逐条对证据、KNOWN-DEBT 无新增未登记项、无遗留调试
      输出。

## 执行步骤

（worktree：`D:/autostack/.wt/lang-537/auto-lang`，分支 `plan-537-dev`；
模板步骤均在 worktree 内执行，plan 文件勾记在主检出）

1. **T1 骨架与登记**：建 `examples/ui/029-photo-gallery/`（pac.at + SPEC.md
   骨架）；README 示例总览表加 029 行 + 编号历史注记。
   验证：`ls examples/ui/029-photo-gallery/` + README diff 自查。
2. **T2 状态与数据层**：`src/front/app.at` 写 App 骨架：msg 块全消息、model
   块（种子平行列表 24 项 + 交互/视图/主题状态全集）+ Init/ApplyFilter handler。
   验证：`auto build`（demo 目录内，零错误）。
3. **T3 网格视图**：view 块网格分支——侧边栏 6 项手写展开 + 工具栏（搜索/
   排序/密度/主题）+ `for item in .view` 卡片网格 + 空态。
   验证：`auto run`（4029）浏览器人工过一遍 + handler 逻辑自查。
4. **T4 查看器视图**：mode=="view" 分支——顶栏/大图 contain/底栏 prev-next/
   收藏切换；OpenPhoto/PrevPhoto/NextPhoto/ToggleFav/BackToGrid 接线。
   验证：`auto run` 人工点击全流程（开图→prev/next 循环→收藏→返回）。
5. **T5 SPEC.md 定稿**：数据形状、过滤/排序规则、双端差异注记（hover/网络图/
   兜底行为）写入。
   验证：SPEC 与 app.at 实际行为逐条对照。
6. **T6 Vue 独立机验**：`auto run` + `node
   .agents/skills/autoui-verifier/scripts/test_vue_playwright.mjs`（或按脚本
   头注释的调用形态）跑冒烟断言，截图在案。
   验证：断言全绿 + 截图存 `docs/plans/` 附件或 scratch（按技能惯例）。
7. **T7 VM 独立机验**：`auto run -r vm` + `python
   .agents/skills/autoui-verifier/scripts/test_vm_mcp.py` 断言同交互子集，
   截图在案。
   验证：断言全绿。
8. **T8 桌面宿主机验**：`cargo run -p auto-lang --features ui-iced --example
   ui_desktop -- --fullscreen --apps-dir examples/ui`（仓库根）→ 确认图库
   图标在桌面 + LaunchApp 打开 + 窗口内可用，截图在案；`cargo t app_registry`
   确认扫描测试（AC4 护栏）通过。
   验证：截图 + `cargo t app_registry` 绿。
9. **T9 收尾**：`/auto-plan:review` 独立复审（验收逐条对证据 + 单组件约束
   合规扫描 + KNOWN-DEBT 检查）→ 回填 spec-impact 元数据 → 状态流转。

## 复审记录

## 待澄清事项

1. （低风险，执行内自决）卡片收藏按钮点击是否冒泡触发开图——若 AURA 无
   stopPropagation 原语，收藏按钮移出卡片点击区（信息行内独立按钮），行为
   记入 SPEC.md。
2. （执行内自决）`app_registry` 扫描测试若为精确计数断言，随新增目录 +1 同步
   bump（属 AC7 例外，复审记录说明）。
3. picsum.photos 在弱网/离线环境不可达属预期降级路径（AC3 兜底条款），不做
   本地镜像；若未来需要离线确定性图源，另立基建（builtin: 扩展），不入本计划。
