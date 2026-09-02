---
plan_id: PLAN-518
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-visual-phase2
author: [zhaopuming]
created_at: 2026-09-01
updated_at: 2026-09-01

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
total_steps: 9
---

# [PLAN-518] 桌面视觉二期——stella 对齐（双主题 / 壁纸资产 / dock 精致化 / 透明度设置）

## 变更摘要

503 的 P2/P3 承接 + 差距矩阵（`scratch/visual-gap/GAP-MATRIX.md` 修订版）
转正式计划。**权威参照** = `scratch/visual-gap/stella/AUTHORITATIVE.png`
（用户提供，含深浅双主题）。503 的教训写进验收：**逐条对表权威图，不再
"计划说刷新了、眼睛说没变"**。

六块（用户三裁定已内化）：

1. **双主题 token 全组**：暖纸 light（米白 #f5f1e8 系 + 墨色前景 + 玫瑰粉
   accent 点睛）+ 精修 dark（深蓝黑面板系），**可切换**（458 机制 + 487
   设置面板 Appearance 分区）；stella dark 实证**壁纸与主题解耦**（深色
   模式下暖纸壁纸保持浅色不变）。
2. **壁纸默认资产**：水墨/宣纸质感默认壁纸（dark/light 各一或共用浅色——
   按 stella dark 实证壁纸独立于主题），经 496 壁纸层生效。
3. **dock 底栏精致化**：去常驻 accent 底格（中性结构色 + hover 才出格）、
   accent 只留激活指示；位置设置可选（487 已有，先沿底栏）。
4. **shell 图标资产核验**：lucide 管线已在（renderer.rs:4686
   `lucide_svg_doc`）——逐点核验 dock/launcher/桌面图标的名称覆盖与
   容器样式（色块感的根因是容器 accent 底非图标缺失）。
5. **窗口 chrome**：标题居中化（stella 形态）、三色圆点保留（用户裁定）、
   阴影/描边按双主题重校。
6. **Transparency 透明度分级设置**：**503-P2 blur parity 决策落点**——
   iced 无 backdrop-blur，以窗口 alpha 分级（关/低/高）设置化实现
   （stella Settings 的 Transparency 开关同位）。

**明确排除**：桌面小组件（时钟卡/日历卡，用户裁定独立计划）；动效系统
（503-P3 维持独立，待形态稳定）；dock 换左竖栏（先沿底栏，形态可选已备）；
app 内部排版密度（458/示例线）。

## 目标

- **G1 双主题**：light 暖纸 / dark 精修蓝黑两套语义色全组
  （`ui/style/theme.rs` 语义分支），设置面板 Appearance 分区切换即时生效
  （`set_dark_mode` 既有链路）。
- **G2 壁纸资产**：默认壁纸（浅色水墨系）随包分发，空配置缺省生效；
  深浅主题下均协调（壁纸独立于主题切换不被重置）。
- **G3 dock**：常驻 accent 底格→中性 + hover 出格；激活指示保留可感知
  （加宽/贴底条形态二选一，实机定）。
- **G4 图标覆盖**：dock/launcher/桌面图标全部 lucide 命名命中（无占位
  色块），容器去 accent 底。
- **G5 窗口 chrome**：标题居中 + 双主题下阴影/描边协调；圆点保留。
- **G6 透明度设置**：设置面板 Transparency 三档（关/低/高）即时生效于
  虚拟窗底色 alpha。
- **G7 验收对表**：双主题全屏截图 vs `AUTHORITATIVE.png` 并排逐条核对
  本矩阵条目，留痕归档。
- **非目标**：桌面小组件（独立计划）；动效（503-P3）；dock 换位；app 内
  密度；vue 轨同步（token 源在 ui/style 共享，vue 侧验证单列任务，深度
  对齐挂 516 后续）。

## 架构方案

- **token**：`crates/auto-lang/src/ui/style/theme.rs` 语义分支按 stella
  双主题重校（Background/Surface/Border/OnBackground/OnSurface/Primary
  密度）——**动的是值不是结构**，458/448 的对齐注释链保留追加注记。
- **壁纸**：资产入 `crates/auto-lang/assets/wallpapers/`（或 examples 资产
  路径，执行期按 496 消费路径定）；缺省链 = storage 键缺席 → 主题配对
  默认图。
- **透明度**：虚拟窗容器背景 alpha 分级（`virtual_window.rs` win_box
  style，503 M5 已有 alpha 基建）——storage 键 `shell.desktop.transparency`
  三档，Appearance 分区消费。
- **设置面板**：`crates/auto-lang/assets/settings.at` 增 Appearance 分区
  （Dark Mode 切换[458/os-config 既有链] + Transparency 三档 + 壁纸入口
  [496 既有]）。

## 技术栈

既有栈（theme 语义色 / 496 壁纸层 / 487 设置面板 / lucide svg）。壁纸资产
为新增资源文件（来源裁定见待澄清③）。零新 crate 依赖。

## 需求分析与背景调查

（矩阵 `scratch/visual-gap/GAP-MATRIX.md` 修订版 + 现场核验 2026-09-01）

- **503 教训**：P0/P1 交付了 token 级微调（后经 505 合并压缩为七项），
  blur/动效/壁纸内容层被裁或未立项——观感感知弱。本期验收 = 对表权威图。
- **主题基建**：`theme.rs` 语义色 `is_dark` 分支（Background=(9,14,26)
  dark / white light 等，448 对齐注释链）；`set_dark_mode`/`set_accent_name`
  thread-local；458 预设 + `AUTO_UI_THEME` env + os-config per-app 模块
  （504/506 迁移）——**切换链路全通，缺的是 stella 调色组本身**。
- **图标基建**：`lucide_svg_doc`（renderer.rs:4686）svg 渲染既有；shell.at
  `button (icon: p.icon)` 图标名来自宿主注册表注入——G4 是核验+补名，
  非新管线。
- **壁纸/设置基建**：496 壁纸层（storage `shell.desktop.wallpaper`）+
  515 G3 vue 壁纸层；487 设置面板三分区（增 Appearance 的落点）；
  505 C 族实机验收通道（视觉留痕工具）。
- **排程**：515/516 刚立项待领（renderer/session 交叠面小且分散，后合者
  rebase）；509/513 无交叠。可并行领取。

## 详细设计

### 1. 双主题 token（theme.rs）

- light（暖纸）：Background #f5f1e8 系、Surface #fbf8f2（卡片微浮）、
  OnBackground 墨色 #2a2723、Border 暖灰、Primary→玫瑰粉族（与 458 coral
  预设对表或新增 `rose` 预设）；
- dark（精修）：Background 深蓝黑 #141a29 系（现值微调偏暖）、Surface
  #1a2235、Border 低对比；
- accent 密度规则成文：结构色用中性，accent 仅 hover/激活/点睛。

### 2. 壁纸资产与缺省链

- 资产 2–3 张（宣纸质感/水墨系；来源见待澄清③）；dark/light 缺省各一张
  （或共用浅色——stella dark 实证）；
- 缺省链：storage 缺席 → 按 496 层消费默认资产；用户设过则尊重。

### 3. dock 精致化（shell.at）

- 图标格：常驻底色 → transparent；hover/active 才出底（503 七项中的
  rounded-xl 保留）；运行指示改"贴底宽条"形态（可感知性实测）；
- accent 只留激活竖条/指示。

### 4. 图标覆盖核验与大型图标规格（G4）

- 调研结论（2026-09-01）：lucide 管线已支持**画时染色**（`lucide_svg_doc`
  生成 `stroke=currentColor`，渲染臂按 text_color/OnBackground/hover:text-*
  注入 `svg::Style.color`——主题切换自动换色）；"色块感"根因 = 容器 accent
  底而非图标缺失。本期四件事：
  1. **独立 `icon` 组件臂核验**：aura.at icon element 标 `iced:"unknown"`——
     已验证的染色路径是按钮 label 内嵌（PUA 标记）路径；dock/桌面用独立
     形态，缺臂则补（染色语义与内嵌路径一致）；
  2. **stroke-width 参数**：`lucide_svg_doc` 支持描边宽度（默认 2，大尺寸
     48–64px 用 1.5 细线，对齐 stella 线性观感）；
  3. **per-app 容器色**：registry/pac.at 增 accent 色配置（或按名哈希从
     主题色板分配）——dock/launcher 格 = 该色圆角底 + 白色 glyph（stella
     dock 徽标同款；单色 lucide 的"带颜色"正解是容器底色，非多彩 SVG）；
  4. **命名覆盖表**：shell.at/registry icon 名 → `lucide_svg_doc` 命中
     全绿（未命中补名或登记）。

### 5. 窗口 chrome（virtual_window.rs）

- 标题 `container(...).width(Fill).center_x()` 居中化（圆点绝对定位左侧
  或三列 row——实现取不破坏拖拽把手语义者）；
- shadow：dark (0,10)/40 40–52px、light 12–18%（随主题分支，503 M5 基础
  上调参）。

### 6. Transparency 设置（G6）

- `virtual_window.rs` win_box 背景 alpha：off=0.95 / low=0.80 / high=0.62
  （初值，实机调）；storage `shell.desktop.transparency` + Appearance
  分区三档按钮；仅影响虚拟窗底色（chrome 圆点/边框不透明保可用性）。

### 7. 设置面板 Appearance 分区（settings.at）

- Dark Mode 切换（写主题键，既有 os-config/458 链）+ Transparency 三档 +
  壁纸更换入口（跳 496 既有键）。

## 测试设计

1. **T1 token 单测**：双主题语义值表断言（theme.rs 既有测试形态）。
2. **T2 设置链路**：Appearance 分区渲染 + 切换 Dark/Transparency 写键与
   生效（desktop_mcp 装载测同型）。
3. **T3 视觉对表**：双主题全屏截图 vs `AUTHORITATIVE.png` 并排（light 主
   对比 + dark 对比），逐条矩阵核对留痕。
4. **T4 回归**：`cargo t ui`、505 验收通道演练一次（设置切换实机流）。

## 验收标准

1. G7 对表留痕：矩阵每条（1.1/1.2/1.3/2.2/2.3/2.4/2.5 部分）✅/注记，
   light 与 dark 各一轮。
2. 双主题切换即时生效且壁纸不被重置（解耦实证）。
3. dock 无常驻 accent 底格；图标无占位色块（G4 命中表全绿）。
4. Transparency 三档实机生效且窗口内容可用性不受损（文字对比度）。
5. `cargo t ui`、settings/shell 装载测不回归；零警告；批一/批二示例
   （504/506/512）渲染零回归。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **双主题 token**：`crates/auto-lang/src/ui/style/theme.rs` 语义分支
   重校（stella 双主题值组）+ T1 单测。
   验证：`cargo t theme`（或 style 套件）。
2. **壁纸资产**：资产入位 + 496 层缺省链接（storage 缺席→默认图，主题
   解耦验证）。
   验证：`cargo t ui` + 实机壁纸显示。
3. **dock 精致化**：`crates/auto-lang/assets/shell.at` 图标格去底 + hover
   出格 + 运行指示形态 + 装载测。
   验证：`cargo t desktop_mcp`。
4. **图标覆盖核验**：icon 名 → lucide 命中表扫描 + 未命中补名/登记 +
   容器去底联动。
   验证：命中表全绿（脚本或单测）。
5. **窗口 chrome**：`crates/auto-lang/src/ui/iced/virtual_window.rs` 标题
   居中 + shadow 重校（双主题分支）。
   验证：`cargo t ui`。
6. **Transparency**：`virtual_window.rs` alpha 分级 + storage 键 + T1
   决策单测。
   验证：`cargo t ui && cargo t session`。
7. **Appearance 分区**：`crates/auto-lang/assets/settings.at` 增分区
   （Dark Mode/Transparency/壁纸入口）+ T2。
   验证：`cargo t desktop_mcp`。
8. **视觉对表 T3**：双主题截图并排逐条核对留痕（505 通道/autoui_screenshot）。
   验证：对表记录归档本计划。
9. **回归 + 收尾**：T4 全量；健康检查；状态翻 execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **壁纸资产来源**：stella 仓壁纸（`D:/Down/stella-os/wallpapers/`，授权
  状态需确认）vs 自制/免费素材（宣纸纹理 + 水墨元素合成）——默认走免费
  素材自制，T2 定。
- **accent 预设**：玫瑰粉是新增 458 预设（rose）还是复用 coral——T1 对
  表后定（新增则 accent 切换器同步）。
- **透明度与 494 真洞的相互作用**：494 的 hole 模式已翻 z 序，Transparency
  只动虚拟窗底色 alpha——两者叠加的表现 T6 实机核一次（预期无冲突：不同
  层面）。
- **vue 轨 token 同步深度**：a2vue 生成链消费 ui/style 同源 token 的部分
  自动跟随；vue 桌面宿主（515 Wallpaper.vue 等）的手写类名需人工同步——
  本期只列核对任务，深度对齐挂 516 后续。
- **动效（503-P3）**：维持独立，本计划不做——但 T3 对表时若 hover 瞬变
  在对比中显著扎眼，登记优先级建议供 P3 立项引用。
