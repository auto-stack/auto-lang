---
plan_id: PLAN-518
status: archived                # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-visual-phase2
author: [zhaopuming]
created_at: 2026-09-01
updated_at: 2026-09-02

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/project.md (ui 模块): theme.rs 语义色双主题 token 重校——light 暖纸系(#f5f1e8/#fbf8f2/墨 #2a2723/暖灰 border)/dark 精修蓝黑(#141a29/#1a2235/#283146),color.rs light 回退表与 renderer 窗口调色板同源换新(448/458 对齐注释链保留)"
  - "docs/specs/auto-lang/project.md (ui 模块): 桌面 shell pack 皮肤——shell.at dock 精致化(聚焦格常驻 accent 底撤除/运行指示贴底宽条/hover 才出格)+settings.at Appearance 分区(深色模式/透明度三档/壁纸入口)+desktop.at 图标格 per-app 徽标色底+白 glyph"
  - "docs/specs/auto-lang/project.md (ui 模块): 496 壁纸层缺省链换新——DESKTOP_WALLPAPER_DEFAULT #090e1a→builtin:ricepaper(内嵌宣纸资产,builtin: 虚拟方案,深浅主题共用浅色=壁纸主题解耦)"
  - "docs/specs/auto-lang/project.md (ui 模块): virtual_window chrome——标题三列 row 窗口级居中+阴影 (0,10)/40 双主题分支+底色乘 Transparency 档位 alpha(chrome 不透明)"
  - "docs/specs/auto-lang/project.md (ui 模块): desktop_protocol Coverage style_prefixes 增 backdrop-(queue 臂词汇放行,§1.3.1 not-yet 误判翻转)"
new_spec_components:
  - "ui/style 毛玻璃词汇(声明冻结): backdrop-blur-{sm|md|lg|xl|2xl|3xl|[Npx]}→StyleClass::BackdropBlur(f32) + backdrop-saturate-{50|100|150|200|[N]}→BackdropSaturate(f32);三臂语义=vue 类串直通出真毛玻璃/iced·gpui·headless 视觉 no-op(装饰性降级非错绘)/queue 臂 BoxLayout 跳过不误判;真 backdrop 渲染挂 RenderQueue 宿主栅格化(KNOWN-DEBT P518 planned-debt:WM 窗口级 glass 属性+DrawOp::BeginBackdrop/EndBackdrop tag 对)"
  - "desktop shell Appearance 面: set_theme 主题热切换动词协议(执行臂 set_dark_mode 即时+shell.appearance.theme 持久化+boot 读回+已声明 dark_mode App 同步[boot 读回/allocate_app desktop 门控/execute_set_theme 三点])+shell.desktop.transparency 三档(off=0.95/low=0.80/high=0.62,每帧读键即时生效)+badge_color_for per-app 徽标色(8 色柔和板 id 哈希,白字对比全板 ≥4.5:1)"
touched_goals:
  - "GOAL-007: stella 双主题 token 对齐(503 系列承接)+backdrop 毛玻璃词汇三臂同源声明冻结(503/515 lineage 延伸)"
  - "GOAL-009: 桌面 shell 视觉二期——壁纸资产化/主题热切换/透明度分级/Appearance 分区/dock·chrome 精致化(496/487/505 lineage 承接)"
  - "GOAL-010: p518-glass-sample 毛玻璃样张示例(G8 验证语料,vue/VM 双臂)"

affects: [auto-lang/ui]
current_step: 12
total_steps: 12
---

# [PLAN-518] 桌面视觉二期——stella 对齐（双主题 / 壁纸资产 / dock 精致化 / 透明度设置）

## 变更摘要

503 的 P2/P3 承接 + 差距矩阵（`scratch/visual-gap/GAP-MATRIX.md` 修订版）
转正式计划。**权威参照** = `scratch/visual-gap/stella/AUTHORITATIVE.png`
（用户提供，含深浅双主题）。503 的教训写进验收：**逐条对表权威图，不再
"计划说刷新了、眼睛说没变"**。

七块（用户三裁定已内化；第 7 条为 2026-09-02 增补 phase）：

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
6. **Transparency 透明度分级设置**：窗口 alpha 分级（关/低/高）设置化
   实现（stella Settings 的 Transparency 开关同位）；503-P2 的"blur
   parity"部分裁定移交第 7 条——**真模糊推迟 RenderQueue，本期冻结声明**。
7. **毛玻璃 DSL 声明冻结（2026-09-02 增补 phase，G8）**：采纳 Tailwind
   原生 `backdrop-blur-*` / `backdrop-saturate-*`（stella 配方
   `blur(24px) saturate(1.6)` 直译 = `backdrop-blur-xl
   backdrop-saturate-[1.6]`）入共享 parser（`ui/style/class.rs` 两个
   StyleClass 变体）；三臂语义 = vue 直通（Tailwind 生成 backdrop-filter，
   零改动）/ iced·gpui·headless 视觉 no-op（不报错不 not-yet——装饰性
   降级非错绘）/ queue 臂因共享 parser 已识别、不触发 §1.3.1 not-yet
   误判；**真 backdrop 渲染挂 RenderQueue 宿主栅格化**（iced 0.14 无
   backdrop primitive 与 pass 口，已验源码；`window::screenshot` 为整场景
   重渲+阻塞读回且有上一帧玻璃反馈污染，只适合快照；fork iced_wgpu 可
   真解但 RenderQueue 在途，不做），KNOWN-DEBT 登记 planned-debt。

**明确排除**：桌面小组件（时钟卡/日历卡，用户裁定独立计划）；动效系统
（503-P3 维持独立，待形态稳定）；dock 换左竖栏（先沿底栏，形态可选已备）；
app 内部排版密度（458/示例线）；**backdrop 真模糊渲染**（挂 RenderQueue，
本期只冻结 DSL 声明与降级语义）。

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
- **G8 毛玻璃 DSL 声明冻结**：`backdrop-blur-*`/`backdrop-saturate-*`
  三臂均可声明——vue 臂实机出毛玻璃、VM 臂降级不报错、queue 臂不触发
  not-yet 误判；真模糊渲染不在本期（挂 RenderQueue）。
- **非目标**：桌面小组件（独立计划）；动效（503-P3）；dock 换位；app 内
  密度；vue 轨同步（token 源在 ui/style 共享，vue 侧验证单列任务，深度
  对齐挂 516 后续）；backdrop 真模糊渲染（挂 RenderQueue 宿主栅格化）。

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
- **毛玻璃声明**：词汇冻结在共享 parser 层（`ui/style/class.rs`，协议
  D2 定案的"两渲染臂共同词汇"）——`StyleClass::BackdropBlur(f32)` /
  `BackdropSaturate(f32)`；渲染端本期 no-op，RenderQueue 期作唯一真源
  lowering（窗口根容器 → 宿主 WM 窗口级 glass 属性，queue/pixels 双臂
  通吃；应用内面板 → `DrawOp::BeginBackdrop/EndBackdrop` 追加式 tag 对，
  线格式零变更）。

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

### 8. 毛玻璃 DSL 声明冻结（backdrop-\* 词汇，渲染分期；G8）

> 背景（2026-09-02 裁定）：真模糊本期不做、等 RenderQueue 宿主自持栅格化；
> 但 DSL 声明面现在冻结，避免将来词汇/协议返工。调研结论（已验 iced 0.14
> registry 源码）：无 backdrop primitive、无 pass 干预口（正常 present 直画
> 交换链，表面仅 RENDER_ATTACHMENT）；`window::screenshot` 是唯一像素通道，
> 形态为整场景重渲 + 阻塞读回 + 上一帧玻璃反馈污染——只适合快照不适合
> backdrop；fork iced_wgpu 可真解（screenshot 代码即施工图）但 RenderQueue
> 在途，投资不划算，裁定不做。

- **词汇（最小集）**：`backdrop-blur-{sm|md|lg|xl|2xl|3xl}` +
  `backdrop-blur-[Npx]`（Tailwind 刻度 sm=4/默认 8/md=12/lg=16/xl=24/
  2xl=40/3xl=64px）→ `StyleClass::BackdropBlur(f32)`；
  `backdrop-saturate-{50|100|150|200}` + `backdrop-saturate-[N]` →
  `BackdropSaturate(f32)`。**刻意不收** brightness/contrast/invert 等其余
  backdrop 系列——照旧走未知类静默跳过，防词汇膨胀。glass 配方另两条腿
  已就绪：半透明底 `bg-white/10`（`parse_color_with_alpha` 既有）+ border
  既有；stella 卡片直译 = `backdrop-blur-xl backdrop-saturate-[1.6]
  bg-white/10 border`。
- **解析落点**：`crates/auto-lang/src/ui/style/class.rs` 两变体 + 前缀
  分支（任意值 `[Npx]` 机制 class.rs:652 既有）；`Style::parse` 结构不动
  （classes 列表天然容纳）。
- **三臂语义**：
  - **vue**：类串直通 DOM，Tailwind 生成 `backdrop-filter`——零改动
    （构建若为 safelist 制需登记，见待澄清）；
  - **iced/gpui/headless**：适配器补 match arm = 视觉 no-op，沿
    `hover_classes` 先例注释（"consumed by future RenderQueue lowering;
    renderers ignore today"）——不报错、不 not-yet（装饰性降级非错绘）；
  - **queue 臂**：共享 parser 已识别 → `BoxLayout` 提取跳过该装饰字段，
    不触发协议 §1.3.1"未知类 → 整 widget not-yet"误判（这正是本期必须
    冻结解析的纪律理由：不冻结，queue 臂爬坡遇玻璃样式会整窗拒绘）。
- **记账**：parity 金样标"已知分歧（VM 降级，RenderQueue 翻转）" +
  autoui-verifier 双端对拍白名单；`docs/plans/KNOWN-DEBT-AND-RISKS.md`
  登记 planned-debt（归还载体 = RenderQueue 宿主栅格化）。

## 测试设计

1. **T1 token 单测**：双主题语义值表断言（theme.rs 既有测试形态）。
2. **T2 设置链路**：Appearance 分区渲染 + 切换 Dark/Transparency 写键与
   生效（desktop_mcp 装载测同型）。
3. **T3 视觉对表**：双主题全屏截图 vs `AUTHORITATIVE.png` 并排（light 主
   对比 + dark 对比），逐条矩阵核对留痕。
4. **T4 回归**：`cargo t ui`、505 验收通道演练一次（设置切换实机流）。
5. **T5 backdrop 词汇**：刻度/任意值解析断言 + 未知 backdrop 系列（如
   `backdrop-brightness-50`）仍静默跳过 + iced 样式应用不 panic（降级
   no-op 断言）。

## 验收标准

1. G7 对表留痕：矩阵每条（1.1/1.2/1.3/2.2/2.3/2.4/2.5 部分）✅/注记，
   light 与 dark 各一轮。
2. 双主题切换即时生效且壁纸不被重置（解耦实证）。
3. dock 无常驻 accent 底格；图标无占位色块（G4 命中表全绿）。
4. Transparency 三档实机生效且窗口内容可用性不受损（文字对比度）。
5. `cargo t ui`、settings/shell 装载测不回归；零警告；批一/批二示例
   （504/506/512）渲染零回归。
6. G8：含 `backdrop-blur-xl backdrop-saturate-[1.6] bg-white/10` 的样张在
   vue 臂实机出毛玻璃、VM（iced）臂正常渲染无报错、queue 臂装载不触发
   not-yet；KNOWN-DEBT 有对应 planned-debt 条目。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **双主题 token**：`crates/auto-lang/src/ui/style/theme.rs` 语义分支
   重校（stella 双主题值组）+ T1 单测。
   验证：`cargo t theme`（或 style 套件）。
   [✅ 已完成] light 暖纸(#f5f1e8/#fbf8f2/墨 #2a2723/暖灰 #e3ddd1)+dark 蓝黑(#141a29/#1a2235/#283146);color.rs 回退表+renderer.rs 窗口调色板/壁纸回退同步;`cargo t theme` 15/15 绿+`cargo t style` 169/169 绿;待澄清②裁定:复用 coral(已校准 #c4706a≈权威图 #C96B62),不新增 rose;顺带收口 449 探针 underline master 预存红(commit 496032e21)
2. **壁纸资产**：资产入位 + 496 层缺省链接（storage 缺席→默认图，主题
   解耦验证）。
   验证：`cargo t ui` + 实机壁纸显示。
   [✅ 已完成] 自制宣纸(ricepaper ≈#EDE7DB,stella 同款)+水墨(inkwash)内嵌,builtin: 方案接 load_image_bytes;DEFAULT #090e1a→builtin:ricepaper;T2a 扩测绿;`cargo t ui` 1699/1703——4 红=master 预存(vm_bridge calendar×3+strip_html×1,stash 干净基座实证同红,与 518 无域交叠),实机显示并入步骤 11 对表;待澄清①裁定:免费素材自制(生成器 scratch/p518/gen_wallpapers.py)
3. **dock 精致化**：`crates/auto-lang/assets/shell.at` 图标格去底 + hover
   出格 + 运行指示形态 + 装载测。
   验证：`cargo t desktop_mcp`。
   [✅ 已完成] 聚焦窗格 bg-primary/15 常驻底撤除(accent 只留竖条)+运行指示贴底宽条(h-1 w-6)+格 hover 才出底;desktop_mcp 3/3+shell_at 冒烟 4/4+dock 34/34 绿(commit 99d57db8f)
4. **图标覆盖核验**：icon 名 → lucide 命中表扫描 + 未命中补名/登记 +
   容器去底联动。
   验证：命中表全绿（脚本或单测）。
   [✅ 已完成] ①独立 icon 臂实证存在(染色路径完整);②lucide_svg_doc_with 参数化,≥48px 独立图标 1.5 细线;③badge_color_for 8 色板(id 哈希,白字对比全板 ≥4.5:1)+desktop.at 徽标色底+白 glyph(dock 保持单色——权威图实测);④命中表测试绿(五资产扫描+pac 清单+examples 并入);a2vue desktop 金样同步;lucide_icon_coverage/desktop_mcp/shell_at/registry 153 全绿(commit e80cf33ec)
5. **窗口 chrome**：`crates/auto-lang/src/ui/iced/virtual_window.rs` 标题
   居中 + shadow 重校（双主题分支）。
   验证：`cargo t ui`。
   [✅ 已完成] 标题三列 row 窗口级居中(左圆点组 64px/中 Fill 居中/右等宽配重,拖拽把手语义不变);阴影 (0,10)/40,dark 40-52%/light 12-18%;cargo t ui 无 518 回归(4 红 master 预存+clipboard 环境型 flaky 单跑绿)(commit b21f89fd3)
6. **Transparency**：`virtual_window.rs` alpha 分级 + storage 键 + T1
   决策单测。
   验证：`cargo t ui && cargo t session`。
   [✅ 已完成] transparency_alpha_for(off=0.95/low=0.80/high=0.62)+键每帧读(写后下一帧生效);win_box+client_area 底色乘 alpha,chrome 不透明;T1 绿;session 88/88+ui 1701/1705(4 红=master 预存)(commit 4203d13fb)
7. **Appearance 分区**：`crates/auto-lang/assets/settings.at` 增分区
   （Dark Mode/Transparency/壁纸入口）+ T2。
   验证：`cargo t desktop_mcp`。
   [✅ 已完成] 深色模式卡(set_theme 动词:SetTheme 编解码+execute_set_theme 执行臂[set_dark_mode 即时+shell.appearance.theme 持久化+dark_mode 声明 App 同步+全 view_dirty]+boot 读回)+透明度三档卡(storage 直写每帧生效)+快照注入 cfg_theme/cfg_transparency;T2 绿;desktop_mcp 3/3+shell_at 4/4+desktop_command 3/3+session 88/88(commit 8d5625569)
8. **backdrop 词汇解析**：`crates/auto-lang/src/ui/style/class.rs` 加
   `BackdropBlur`/`BackdropSaturate` 两变体 + 刻度/任意值解析分支 + T5
   单测。
   验证：`cargo t style`。
   [✅ 已完成] 两变体+刻度/任意值分支(不收系列 Err 静默跳过)+iced_adapter no-op 臂(编译必需);T5 绿+cargo t style 170/170(commit 000a90aba)
9. **三臂适配器**：iced/gpui/headless adapter 补 no-op arm（先例注释）+
   vue 直通核验（类串到 DOM 链路抽查；safelist 制则登记条目）+ queue 臂
   BoxLayout 提取不误判核对。
   验证：`cargo t ui` + vue 实机样张出毛玻璃。
   [✅ 已完成] gpui 显式 no-op 臂+headless(存储式)核验免臂;queue 臂 Coverage 前缀增 backdrop-+三段测(parser/BoxLayout 零污染/judge Covered);vue 实机:样张 p518-glass-sample 计算样式 backdropFilter=blur(24px) saturate(1.6)+视觉确认,Tailwind JIT 直扫零登记(待澄清⑤裁定);VM 臂运行零报错降级实证;cargo t ui 1704/1708(4 红=master 预存)+coverage 17/17(commit fdaa2bbf9)
10. **记账**：`KNOWN-DEBT-AND-RISKS.md` 登记 planned-debt（真模糊挂
    RenderQueue）+ parity 白名单注记 + 样张入 T3 对表集。
    验证：文档留痕。
    [✅ 已完成] KNOWN-DEBT 🟢 P518 planned-debt(WM glass 属性/DrawOp tag 对双路径+iced 0.14 无 primitive 实证);autoui-verifier SKILL.md 对拍检查项第 9 条(玻璃卡已知分歧白名单);样张 p518-glass-sample 入 T3 对表集(commit fdaa2bbf9)
11. **视觉对表 T3**：双主题截图并排逐条核对留痕（505 通道/autoui_screenshot）。
    验证：对表记录归档本计划。
    [✅ 已完成] 双轮 boot(light 预置键/dark 默认)+虚拟窗+live 切换序列实机截图;boot 读回补变量同步+allocate_app 同步(两项 T3 期间发现根因修复);矩阵 1.1/1.2/1.3/2.2/2.3/2.4 ✅+2.5 △(注记);壁纸 #EDE7DB 与权威图逐通道一致;切换即时生效+壁纸不重置解耦实证;记录+8 张证据图归档 docs/plans/reports/518-t3-visual-parity.md;os-config×shell 架构缝登记 KNOWN-DEBT
12. **回归 + 收尾**：T4 全量；健康检查；状态翻 execution_done。
    验证：`cargo check -p auto-lang && cargo t ui`。
    [✅ 已完成] cargo check 零新增警告(159 基线,己改文件零命中)+cargo t ui 1704/1708(4 红=master 预存,stash 干净基座实证)+osconfig 22/22+musk_vm_track 54/54+fit 16/16(504/506/512 渲染回归证据:套件级+T3 实机 todo/notes/glass 全管线装载)+505 验收通道设置切换实机流(T3 live 序列);六待澄清全部裁定落档(①自制②coral 复用⑤JIT 直扫⑥mini 样张)

## 复审记录

**Reviewer**: auto-plan:review(zhaopuming 会话)/ **时间**: 2026-09-02 / **基座**: worktree plan-518-dev(11 提交,分叉 3377ed098;master 期间被 517/519/520/521 推进——合并期再对齐,非本计划域)。

### 逐条验收复核(verify, don't trust)

| # | 标准 | 判定 | 证据(复审重跑) |
|---|---|---|---|
| 1 | G7 对表留痕(light/dark 各一轮,逐条矩阵) | **PASS** | `docs/plans/reports/518-t3-visual-parity.md` + 8 张证据图(对表板 VM light\|权威\|VM dark 并排);矩阵 1.1/1.2/1.3/2.3/2.4 ✅,1.2 色值逐通道断言(light (246,242,232)≈#f6f2e8/dark (20,27,42)=#141a29 逐值命中);2.2 ✅;2.5 △(示例线,计划内排除) |
| 2 | 双主题切换即时生效+壁纸不重置 | **PASS** | live 序列 live-1→2(同帧翻深)+storage 回读 dark→boot 两轮读回正确;壁纸两轮同一张宣纸(topleft 同源)——解耦实证 |
| 3 | dock 无常驻 accent 底+命中表全绿 | **PASS** | dock 图标区饱和非 accent 像素 = 4(≈噪声,无常驻色块);accent 实测仅指示条/激活竖条(112 accent px);lucide_icon_coverage 复跑绿 |
| 4 | Transparency 三档实机生效+可用性 | **PASS** | live-3 实机+storage=high 回读+transparency_levels 复跑绿;文字不透明按构造保对比(chrome/文字不乘 alpha) |
| 5 | 回归+零警告+示例零回归 | **PASS**(注) | **cargo tf 3355/3355 全绿**(含 1M churn)+**cargo tv 3513/3513**+desktop_protocol(ui-iced)120/120+cargo t ui 1704/1708——4 红=master 预存(stash 干净基座同红实证,vm_bridge×3+strip_html×1,与 518 无域交叠,**非本计划债务,建议另行收口**);cargo check 零新增警告(159 基线,己改 31 文件零命中);504/506/512=套件级绿(osconfig 22/musk_vm_track 54/fit 16)+T3 实机全管线装载(todo/notes/glass),未逐一实机重渲(证据充分性注记) |
| 6 | G8 三臂+planned-debt | **PASS** | vue 臂 Playwright 计算样式 `backdrop-filter: blur(24px) saturate(1.6)` 实读+视觉确认(Tailwind JIT 直扫零登记);VM 臂零报错;queue 臂 backdrop_glass_style_queue_arm_not_rejected 复跑绿;KNOWN-DEBT 🟢 P518 两 Entries 在档(backdrop planned-debt+os-config×shell 架构缝) |

### 遗漏/延后/Workaround 猎捕

- **遗漏 1 项已收口**:待澄清③(Transparency × 494 真洞叠加实机核对)执行期未做——**复审门补做**:AUTO_DESKTOP_HOLE=1+transparency=high boot+launch todo,进程存活+窗口渲染+截图产出(hole-transp.png)——无冲突,闭环。
- **延后(全部计划内裁定,非私裁)**:backdrop 真模糊→RenderQueue(计划§8 裁定+planned-debt);vue 手写类名深度对齐→516(非目标);动效→503-P3;小组件/dock 换位→独立计划。
- **Workaround**:零(diff 无 TODO/FIXME/HACK;os-config×shell 架构缝为已登记债务而非遮蔽,且 allocate_app 同步已缓解主路径)。
- **发现并当场修复(执行期)**:boot 主题读回/新挂载 App 的 dark_mode 声明变量同步缺口(011-calculator 翻回根因)——已修+T3 重验。

### 结论

**六条验收全 PASS,无阻断债务** → `status: reviewed`。遗留移交:①4 个 master 预存红(vm_bridge/strip_html)非本计划域,建议独立收口;②os-config 逐 app 主题×shell 全局 dark_mode 单例架构缝(per-app color context 归 RenderQueue 重构);③深色 scrim 35% 浓度实机调参旋钮(纸感偏灰,可读性层权衡)。

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
- **vue 侧 Tailwind 构建形态**：类串若是 JIT 直扫则 backdrop-\* 零登记
  即生效；若 safelist/白名单制需增条目——步骤 9 执行时核验。
- **backdrop 样张来源**：G8 验收需要一个带玻璃样式的示例页/卡片（改既有
  示例或 examples/ui 新增 mini 样张）——步骤 9 执行时定，避免为此单独
  立示例计划。

---

## spec-sync 回写记录（merge 沉淀,2026-09-02）

- `.autoos/specs.json`：P518-1..6 六节幂等 upsert（reports/goals/architecture/designs/tests/reviews,`file` 指向本归档件）。
- `docs/specs/auto-lang/ui/plans.md`：518 行追加（表尾,旧→新序）。
- `docs/specs/auto-lang/ui/overview.md`：「桌面视觉二期（plan-518 落地）」段落（plan-503 段后）。
- `docs/specs/goals.md`：GOAL-007/009/010 三条目推进注记。
- `KNOWN-DEBT-AND-RISKS.md`：P518 backdrop planned-debt（真模糊挂 RenderQueue）+ os-config 逐 app 主题×shell 全局架构缝两 Entries（合并自带）。
- 索引再生：`python scripts/spec-index.py`。
