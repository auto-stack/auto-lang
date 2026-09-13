---
plan_id: PLAN-619
status: reviewed              # drafting → executing → execution_done → reviewed → archived
feature_name: 015-notes-vm-vue-parity
author: [zhaopuming]
created_at: 2026-09-12
updated_at: 2026-09-12
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md（593/601 段）中「VM 语义色查 stella 单源」的**隐式缺省适用范围**——未声明 theme{} 的应用也按 stella 取色这一条规则退役，改为「权威色板 = 与生成端 index.css 同源的表（scaffold），桌面宿主显式声明 stella」"
new_spec_components:
  - "docs/specs/auto-lang/ui/overview.md：新增「双端一致门禁」与两条语言层已知坑——(a) 容器 padding 覆盖次序（单侧 > 轴 > 统一 > legacy）与 margin 整族（m-*/mx-*/my-*）折算；(b) icon 尺寸权威 = `.at` 的 `size:`（优先级 = 显式 w-*/h-*/size-* 类 > size: > 缺省 20px；VM `with_icon_size` 折 Width/Height、Vue 生成器折内联 style=\"width:Npx;height:Npx\"——合并 PLAN-617 后的双端口径），以及「lucide 文档构造必须单层包装 fragment（嵌套完整文档会按 16/24 二次缩放）」的实现约束"
touched_goals: [GOAL-007]

affects: [auto-lang/ui, auto-man]
current_step: 5
total_steps: 5
---

# [PLAN-619] 015-notes-vm-vue-parity

## 0. 变更摘要

PLAN-616 交付 015-notes 清爽化重做后，**同一份 `.at` 源码在 Vue 与 VM(iced) 两端肉眼可见地不一致**。
本计划把这些差异**量化、定位到引擎侧根因并修掉**，让「双端一致」从口号变成有像素预算的门禁。

用户实测指出的 4 项（均已在 `docs/plans/attachments/616/` 的前后对比截图上复核）：

| # | 现象 | 量化证据（逻辑坐标，`tools/parity_shot_diff.py` 可复跑） |
|---|---|---|
| P1 | 所有面板/表面背景色两端不同 | 页面/编辑区 vue `(9,14,26)` vs vm `(20,26,41)`；侧栏 vue `(14,21,37)` vs vm `(26,34,53)` |
| P2 | 图标明显偏小、未行内居中 | 同一 glyph 同一窗口 ink：vue `17.5×19.5` vs vm `8.2×9.8`（≈47%/50%） |
| P3 | 搜索行左右内边距缺失（VM） | 行左边界 12 → 图标 ink 起始：vue `24` vs vm `16.5`（少 ≈8px） |
| P4 | 侧栏列表左侧内边距两端不同 | 分组标签左起 vue `16` vs vm `8`；笔记行文本左起 vue `24` vs vm `16.5`（少 8px） |

另：**纵向节奏两端一致（地标带 68/112 vs 69/113，差 1px）**——初判的「VM 行更紧」不成立，不列入本计划。

## 1. 目标

- G1：语义 token 在**两端解析为同一色值**（P1）——消除「同一 token 两套色板」。
- G2：`icon` 元素的尺寸在两端**按 `size` 生效**且**光学居中**（P2）。
- G3：容器（row/col 包装件）的 `px-*`/`py-*` 在两端**同等生效**（P3/P4）。
- G4：把「双端一致性」变成**可复跑的门禁**：`tools/parity_shot_diff.py` + 采样点/像素预算写进 acceptance。

**非目标**：不重做 015-notes 的视觉设计（PLAN-616 已定稿）；不放宽 acceptance.atd 既有验收项；
不改后端/数据模型。

## 2. 架构方案

三个根因分别落在不同层，修法不同：

| 现象 | 根因层 | 处置方向 |
|---|---|---|
| P1 色板 | **VM 语义解析单源** vs **Vue index.css 生成源** | 二选一并让另一端跟随：①（推荐）VM 默认活动主题与「scaffold 类应用」的 Vue CSS 同源；②或 app 用 `theme{}` 显式钉住（PLAN-601 已双端消费）。需先定「谁是权威」——见 §10 待澄清① |
| P2 图标 | **`size` 未贯通**（Vue 走 lucide 默认 24；VM 实际 ≈12） | Vue 生成器补 `:size`；VM 侧查清 12px 来源（`icon` 臂设了 Width/Height=size，实测却 ≈12 → 疑落 PUA/字号路径）并使其按 size 绘制 + 盒内居中 |
| P3/P4 内边距 | **容器 padding 解析**（`IcedStyle` 的 per-axis 与 uniform 覆盖次序 / sidebar 契约类的合并） | 先做**有界调查**（T-01）定位到具体函数再改，避免盲改 |

## 3. 技术栈

Auto 语言 `.at`（示例侧，仅作验证载体）；Rust 引擎侧：`crates/auto-lang/src/ui/style/{class.rs,iced_adapter.rs}`、
`crates/auto-lang/src/ui/iced/renderer.rs`、`crates/auto-lang/src/ui/aura_view_builder.rs`、
`crates/auto-lang/src/ui_gen/{vue.rs,sidebar_contract.rs}`；生成侧：`crates/auto-man/src/`（index.css 模板/scaffold）。
验证：`tools/parity_shot_diff.py`（本计划新增，git 跟踪）+ 双端截图 + `cargo tv`/`cargo t`。

## 4. 需求分析与背景调查

### 4.1 证据链（可复跑）

```bash
python tools/parity_shot_diff.py docs/plans/attachments/616/after_vue_initial.png \
                                 docs/plans/attachments/616/after_vm_initial.png
# → 语义面色差 + 地标带 + 图标 ink 三组数字（P1/P2/P4 的机器可读版）
```

截图：`docs/plans/attachments/616/{before,after}_{vue,vm}_initial.png`（PLAN-616 归档，git 跟踪）。

### 4.2 根因定位（已核实部分）

**P1（色板）——已定案**：
- VM：`crates/auto-lang/src/ui/style/theme/mod.rs` 的 `resolve_semantic_rgb()` → `active_theme().resolve(token, is_dark)`
  （注释明确「PLAN-593 V2：静态语义色查 registry（stella 单源）」）；
  `crates/auto-lang/src/design_tokens/registry.rs` 的 `STELLA.dark` 表：`Background = Rgb(20,26,41)`、
  `Card = Rgb(26,34,53)` —— **与 VM 截图像素完全相等**。
- Vue：`gen/front/vue/src/assets/index.css` 的 `:root/.dark`（auto-man scaffold 模板）：
  `--background: 222.2 47% 7%` = `(9,14,26)`、`--card: 222.2 47% 10%` = `(14,21,37)` —— **与 Vue 截图完全相等**。
- 结论：**同一语义 token 在两端来自不同色板**（stella vs scaffold/shadcn），非渲染误差。

**P2（图标）——半定案**：
- `aura_view_builder.rs`：`icon` 臂 `size` 默认 16，并注入 `Width(Pixels(size))`/`Height(Pixels(size))` 类；
- `ui/iced/renderer.rs`（`lucide:` 臂）：svg widget 宽高取 `IcedStyle.width/height`，**缺省 16**；
- `ui_gen/vue.rs:8431`：`icon` 追加 `w-5 h-5`（20px），但 `size` 未出现在发射里 → Vue 实际按 lucide 默认 **24** 渲染
  （实测 ink 17.5×19.5 ≈ 24 盒的 73%/81%）；
- VM 实测 ink 8.2×9.8 ≈ **12px 盒**，既不是请求的 18 也不是缺省 16 → 疑走「按钮内嵌 PUA/字号」路径或 size 未达 svg 臂，**需 T-02 钉死**。

**P3/P4（内边距）——待定位**：
- `renderer.rs:1824` `apply_row_style(..., padding, style, ...)` → `iced_padding(padding, style)`（`:1168`）
  → `iced_padding_from_is()`（`:1145`）：**uniform `padding` 命中即 early-return，per-axis 覆盖被丢弃**；
  该 early-return 与「契约类 `p-2` + 用户 `px-2 pt-2` 合并」组合是 P3/P4 的头号嫌疑；
- 侧栏族在 VM 端由 `ui_gen/sidebar_contract.rs` 的契约常量（如
  `CONTENT_BASE = flex min-h-0 flex-1 flex-col gap-2 overflow-auto`）+ 用户类合并生成；
  `015-notes/src/front/sidebar.at` 的 `sidebar_content (style: "px-2 pb-2")`、搜索行 `px-2.5`
  是实测少掉的两处 → **T-01 有界调查**给出确切函数与覆盖次序。

### 4.3 影响面（为何值得单独立项）

P1 影响**所有** gallery/示例应用（任何未声明 `theme{}` 的 scaffold 类应用在 VM 上都会偏 stella），
是 GOAL-007「跨端视觉一致」的正面违例；P2/P3 影响所有用 `icon` 与容器 padding 的页面。
属 **L2 级引擎改动**，需独立计划承载门禁与回归对拍。

### 4.4 授权与约束

- 用户授权（2026-09-12）：指出两图差异（面板底色 / search bar 左右 padding / Notes 列表左 padding /
  图标尺寸与居中）并指示「继续洗」→ 即继续收敛两端差异。
- 约束：不引入新的视觉设计变更；不得以「已知差异」名义放宽既有 parity 契约；
  P1 的「谁是权威色板」若涉及跨示例统一，须在 §10 记录裁决依据。

## 5. 详细设计

### 5.1 分任务设计要点

- **T-01（有界调查，产出决策件）**：定位 P3/P4 的确切丢失点。方法：最小 `.at` 夹具
  （row/col + `p-2`⊕`px-2`、`bg-card` + `px-2.5`）+ VM dump 或单元级断言 `iced_padding_from_is`，
  给出「uniform 覆盖 per-axis」还是「契约类合并丢类」的定论与覆盖次序规则。
- **T-02**：P2 尺寸贯通。确定 VM 端 12px 来源（走查 `icon` 臂 → svg 臂 / PUA 路径），
  使 VM 按 `size` 绘制并**盒内居中**；Vue 生成器为 `icon` 发射 `:size`。
- **T-03**：P1 色板单源。按 §10① 裁决落地（VM 默认主题对齐 scaffold 类应用 / 或示例侧 `theme{}` 钉住 +
  规范写明适用范围），并补**跨端色值对拍断言**。
- **T-04**：把 `tools/parity_shot_diff.py` 纳入门禁（登记到 `autoui-verifier` 技能的双端检查清单），
  为 015-notes 加「两端语义面/图标 ink 在预算内」的 acceptance 条目。
- **T-05**：015-notes 实机复验 + 归档新截图（补充/替换 attachments 基线）。

### 5.2 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-lang/ui/overview.md`（593/601 主题/token 段） | **before**：语义 token 的 VM 解析「静态语义色查 registry（stella 单源）」，未声明 `theme{}` 的应用**隐式**按 stella 取色。**after**：权威色板 = **与生成端 `index.css` 同源的那张表**（`registry::SCAFFOLD`，auto-man 生成 Vue CSS 时逐 token 渲染的就是它）；未声明 `theme{}` 的应用两端取同一表；**桌面宿主必须显式声明 stella**（`DesktopConfig::default().theme_name = Some("stella")`），使规则成文为「宿主 = stella、pac 应用 = scaffold」 | 同一语义 token 两端两套色板 = GOAL-007 正面违例；用户裁决 scaffold/shadcn 为权威（2026-09-12） | AC-01 |
| SD-02 | add | `docs/specs/auto-lang/ui/overview.md`（已知坑节） | 新增三条语言层规则：①**容器 padding 覆盖次序 = 单侧 > 轴 > 统一 > legacy**（对齐 CSS/Tailwind 源码序），margin 的 `m-*`/`mx-*`/`my-*` 整族必须折算（iced 无 margin，折外部 padding）；②**icon 尺寸权威 = `.at` 的 `size:`**（VM 折 Width/Height，含显式 `w-*` 可覆盖；Vue 生成器发 `:size` 并把缺省 `w-5 h-5` 让位），shadcn 资产内置的 `[&>svg]:size-*` 会盖过 `:size` 属已知残余；③**lucide 图标表项是完整 SVG 文档**（`width/height=16` + `viewBox 0 0 24 24`），消费方必须取内层 markup 按目标尺寸重包，直接嵌套会按 16/24 二次缩放 | PLAN-619 实证的四个根因 + 一个文档写法坑，均属「不做就继续踩」的规则 | AC-02, AC-03 |

## 6. 测试设计

- 单元：`iced_adapter`/`class.rs` 的 padding 与 icon 尺寸解析用例（覆盖 `p-2`+`px-2`、`size` 缺省/显式）。
- 对拍：`tools/parity_shot_diff.py` 采集两端语义面色值 + 图标 ink；**预算**：语义面色值**逐通道 ≤2**
  （同一 token 同值），图标 ink 长宽比 **≥0.9**（VM/Vue），列表/搜索行左内边距差 **≤1px**。
- 端到端：`cargo tv`（语料 golden 不回归）+ `cargo t`；015-notes 19 条 MCP 场景与 18 条 Vue 场景**保持全绿**。
- 回归锚：把上述预算写成 015-notes acceptance 的 T14（两端对拍）条目。

## 7. 验收标准

- **AC-01**：015-notes 双端截图中，页面/编辑区/侧栏三处语义面色**逐通道差 ≤2**（当前 P1 差 (11,12,15)）。
- **AC-02**：同一 `icon (size: N)` 在两端 ink 尺寸比 ≥0.9，且图标盒在行内**光学居中**（目验 + ink 中心与文本对齐）。
- **AC-03**：搜索行左内边距与侧栏列表左缩进两端差 **≤1px**（当前差 8px）。
- **AC-04**：`tools/parity_shot_diff.py` 入库并在 acceptance / `autoui-verifier` 清单中登记为常驻检查。
- **AC-05**：`cargo tv` + `cargo t` 零新增红；015-notes 19 MCP 场景 + 18 Vue 场景保持全绿。

## 8. 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 `[✅ 已完成]` 一行证据）

- **T-01 P3/P4 有界调查 → 决策件**：最小夹具 + 单元断言定位 padding 丢失点（uniform 覆盖 per-axis？契约类合并丢类？），
  产出覆盖次序规则与修复点。验证：夹具上新旧行为对比（VM dump / 单元测试）。
  **[x]** 决策件见 §8.1（根因 R1–R4）；取证用 `ui/iced/layout_tests.rs` headless 夹具，
  `cargo nextest run -p auto-lang --lib --features ui-iced,iced-layout-tests -E 'test(plan619)'` 可复跑。
- **T-02 P2 图标尺寸与居中**：`aura_view_builder.rs` / `ui/iced/renderer.rs` / `ui_gen/vue.rs` 三处按 T-01 结论统一 `size` 权威。
  验证：`tools/parity_shot_diff.py` 的 ink 比 ≥0.9。
  **[x]** VM 侧 `size:` 已消费（`convert_image_or_icon` 折 Width/Height，显式 `w-*` 仍可覆盖）；
  Vue 侧发射 `:size` 并在显式 size 时撤掉缺省 `w-5 h-5`；三处回归测试
  （`plan619_icon_size_prop_sets_explicit_box` / `plan619_icon_size_prop_emits_bound_size` /
  `plan619_icon_box_follows_style_size`）。
  **[!]** **AC-02 未达成**：实测同 glyph ink 仍偏小（§8.2 ②），根因为 iced 侧 SVG 栅格化缩放
  与 Vue 侧 shadcn 资产 `[&>svg]:size-4` 钉死 16px 两处，需裁决后续走向（§10③）。
- **T-03 P1 色板单源**：按 §10① 裁决落地 + 跨端色值断言。验证：语义面逐通道差 ≤2。
  **[x]** 轨缺省主题 → `scaffold`（`ui/style/theme/mod.rs:41`），桌面宿主显式 `Some("stella")`
  （`desktop_config.rs`）；实机对拍：页面/编辑区逐通道差 **0**、侧栏差 **1**；
  单测 `default_theme_is_scaffold_and_matches_vue_css_tokens` + `theme_name_roundtrip`。
- **T-04 门禁化**：`tools/parity_shot_diff.py` 登记到 `.agents/skills/autoui-verifier/` 清单与 015-notes acceptance T14。
  **[x]** 探针升级为预算门禁（色 ≤2/通道、ink 比 ∈[0.9,1.1]、左缩进 ≤1px，违规退出 1）；
  acceptance 新增 **T14 + C-PARITY-1**；技能新增「步骤 3.5 像素预算对拍」+ 检查项 9。
- **T-05 实机复验与归档**：双端新截图 + `cargo tv`/`cargo t` 对拍 + 更新 attachments。
  **[x]** 双端实机采集 + 探针预算全绿（§8.2）、门禁对拍零新增红（§8.3）、
  **两套功能场景全绿**（19 MCP 场景 19/19 + 18 Vue 场景 18/18，见 §8.4；F-1 收口 `6aad8ba1f`）。

**依赖**：T-01 → T-02/T-03 → T-04 → T-05。

### 8.1 T-01 决策件：P3/P4 的四个根因（headless 实测，非推断）

| ID | 落点 | 症状 → 量化证据（夹具断言值 = 修前 → 修后） |
|---|---|---|
| **R1** | `ui/aura_view_builder.rs` `with_class_prop` | 只读 `class:` 键、缺 `.or_else(style)`（同文件另 8 处样式取值点都有该回落）→ `sidebar_content (style: "px-2 pb-2")`、`sidebar_menu_button { style: "py-1.5 …" }` 的**用户类整串丢失**。链夹具文字 x：16 → **24**（Vue 同链 DOM = 24）。 |
| **R2** | `ui/iced/renderer.rs` `iced_padding_from_is` | uniform `padding` 命中即 early-return，per-axis/per-side 覆盖被丢（与该文件 `debug_style_insets` 注释记载的「单侧>轴>统一」自相矛盾）。`p-2 py-1.5` 垂直 8 → **6**；`p-2 px-2.5` 水平 8 → **10**（与类串顺序无关）。 |
| **R3** | `ui/iced/renderer.rs` `apply_row_style` / `apply_column_style` | 只读 `margin_left/right/top/bottom`，**`mx-*`/`my-*`/`m-*` 整族不折算**（`MarginX/MarginY` 只写 `margin_x/y`，无消费者）。搜索行内容左起：0 → **12**（`mx-3`），图标 ink 起点 11.5 → 24。 |
| **R4** | `ui/iced/renderer.rs` Text 臂 | 盒样式容器**完全不消费 `height`** → `sidebar_group_label` 契约 `h-8` 不落盒（16 vs 32），整条列表上移 8px。label 文本 y：8 → **16.2**（= group p-2 8 + 盒内居中），便签标题 y 46（= 盒底 40 + `py-1.5`）。 |

修复口径（对齐 CSS/Tailwind 语义）：`ui/style/iced_adapter.rs` 新增
`IcedStyle::effective_padding` / `effective_margin` / `has_margin`（**单侧 > 轴 > 统一 > legacy**），
四个消费点统一走它——把分散的「谁覆盖谁」收敛成一个可断言的事实源。

### 8.2 T-05 双端实机对拍（`tools/parity_shot_diff.py`，1280x800）

| 维度 | Vue | VM | 判定 |
|---|---|---|---|
| 语义面 页面/编辑区 | (9,14,26) | (9,14,26) | Δ0 ✅ AC-01 |
| 语义面 侧栏 | (14,21,37) | (13,20,37) | Δ1 ✅ AC-01 |
| 搜索行左边界 | 12 | 12 | Δ0 ✅ AC-03 |
| 分组标签文字左起 | 24 | 25 | Δ1 ✅ AC-03 |
| 便签标题文字左起 | 25 | 25 | Δ0 ✅ AC-03 |
| **搜索图标** ink 比 vm/vue（同字形判据） | — | **1.03 / 1.03** | ✅ AC-02（预算 [0.9,1.1]；修前 0.68） |
| 顶栏 notebook ink 比（参考，两端字形数据不同） | — | 0.89 / 0.96 | 参考值（修前 0.57/0.62） |

① **AC-01 / AC-03 达成**（实机截图，非夹具推断）。
② **AC-02 已达成**（修后 1.03/1.03）。根因是**我们自己造的**，与 iced 无关，见 §8.5：
`lucide_svg()` 表里每条已是完整 SVG 文档（`width/height=16`），`lucide_svg_doc_with` 又把它套进
第二层 24×24 的 `<svg>` → 嵌套 viewport 使 glyph 按 **16/24 = 0.667** 缩放。这正是 P2 记录的
「声明 18px、实测 ≈12px 盒」（18 × 0.667 ≈ 12）。两端消费同一份文档，故此前矢量臂与
「自栅格化」臂产物逐像素相同。

③ 采集环境注记：worktree 与主检出**共享 `CARGO_TARGET_DIR`**，期间有其它 agent 重建
`auto.exe`，首轮 Vue 生成物因此混用旧二进制（缺 `:size`）；**结论一律以私有拷贝的 `auto.exe`
复跑为准**（`NavTree.vue` 的 `:size` 由 0 → 8 处即该污染的直接证据）。

### 8.5 P2 图标尺寸的根因定案（A 裁决后的定点实验，含一次被证伪的假设）

**结论：不是 iced 的绘制路径，是我们自己的 SVG 文档双重嵌套。**

| 步骤 | 实测 | 判读 |
|---|---|---|
| 夹具：同 glyph 声明 12/24/36/48 放一行 | ink 5.0/10.0/14.5/19.5（= 0.42×盒），居中 | 恒定比例画小 |
| 同一 32×32 PNG 走 raster（image）臂 | 画出来正好 32.0 逻辑 px | raster 臂 1:1 可信 |
| 换成「自栅格化 + raster 臂」（resvg/tiny-skia，零新增 crate） | 与矢量臂**逐像素相同**（夹具四档 + 双端截图都一致，108 次绘制 `hit=true`） | ⇒ 差距不在绘制路径（该候选已 revert，见 `a6897a61f`/`b693c828a`） |
| 纯 CPU 栅格化同一 fragment | ink 只占 viewBox 的 0.583（几何应 0.833） | 差距在**文档本身** |
| 单独渲染 `circle` / `path` | 4..39 / 32..42（都正确） | 起初误判为「usvg 形状组合坑」 |
| 打印 `lucide_svg("search")` 原文 | 它是**完整 `<svg width="16" height="16" viewBox="0 0 24 24">…</svg>` 文档**，而 `lucide_svg_doc_with` 又把它套进一层 24×24 的 `<svg>` | **根因：嵌套 viewport → 16/24 = 0.667 缩放** |

修法（3 行）：`lucide_svg_doc_with` 只取表项内层 markup（`find('>')` → `rfind("</svg>")`）后
按目标尺寸重包一层；不动 glyph 数据。回归锚 `plan619_lucide_doc_renders_geometric_ink`
（resvg/tiny-skia 仅作 dev-dependencies，两者已在依赖图内）断言 `search` 的 ink 充满度
落在几何值 0.833 邻域——修前 0.583 **必红**。

**剩余（不阻塞本计划 AC）**：P619-D2（内嵌 fragment 的**字形数据**与
`lucide-vue-next@0.312` 有分歧：`notebook` 等，后者带 4 条装订刻度而本仓是 book 形；
`search` 已核对**逐字相同**）；shadcn 资产 `[&>svg]:size-4` 会把组件内图标钉死 16px、
盖过 `:size`（只影响 Button/sidebar 组件内的图标，普通 div 内的图标不受影响——AC-02 的
搜索图标判据即属后者，故已达标）。

### 8.3 门禁对拍（`cargo tv` / `cargo t`，与基线逐条归因）

| 档 | 基线 `3972645ec` | 本分支 | 结论 |
|---|---|---|---|
| `tv`（`--features test-vm-files`） | 5 红 | 5 红 | 同一集合（`ffi_dep_parity_*` / `ffi_dual_019`：oracle 二进制缺失「build step missing」）= 环境预存，**零新增** ✅ |
| `t`（daily，`--features ui-iced`，`--no-fail-fast`） | 27 红 | 28 红 → 修掉新增 2 条后 27 红 | 唯一新增两条是本轮新测试自身缺陷（`set_theme` 返回「是否变化」语义用错 + scaffold Card 字节值应为 (13,20,37)），已修正；其余 27 条与基线**同名同因**（含 `ui::layout::tests` 28 条任务栏高度相关环境预存红）✅ |

**AC-05 判定**：零新增红 ✅（见 §8.4 的功能场景结论）。

### 8.4 015-notes 功能场景（AC-05 第二半，**两套场景均已全绿**）

| 运行器 | 结果 | 归因/纠正 |
|---|---|---|
| `tests/run_autotest.py 015-notes.autotest --mode vm`（19 MCP 场景） | **19 passed / 0 failed / 0 skipped** | 收口前为 3 passed/16 failed，**真因是就绪竞态**（驱动只等 MCP 端口就开跑 → app 未初始化：T0 的 `autoui_state` 返回空、`autoui_find` 在空树上找不到元素，前 16 场景连锁失败，跑到文件末尾 T11-T12 时 app 早已就绪故那三条通过）。**复审期把它归因成「runner 元素解析链路失配」是错的，此处订正**：`autoui_find` 与 runner 的正则解析本身都正常（`_find("button","All")` 手工复核返回正确 vnode）。修法 = `McpAdapter::wait_ready()` 三判据闸门 + `run_suite` 首步调用（commit `6aad8ba1f`） |
| `tests/` playwright（18 场景：smoke 13 + accent-dark 5） | **18 passed（58.4s）** | 需 `@playwright/test`（示例 `tests/` 原缺，已 `pnpm install --offline` 装 1.63.0）+ Vue 端与 back（`auto run --front-port 6111`，API 8080 就绪后跑 `NOTES_URL=http://localhost:6111 pnpm test`） |
| `tests/desktop_mcp.py`（Plan 370 官方驱动） | 12 passed / 1 failed / 1 skipped | 唯一红为 `Snapshot shows New button label` —— PLAN-616 把顶栏按钮 `New`→`New note` 后该断言未同步（测试漂移，与本计划无关；skip 为文件头声明的旧 NavTree 限制，616 后实际正常渲染） |

## 9. 复审记录

### 9.1 /auto-plan:new 起草 + 裁决回填（2026-09-12）

- stage: new；Plan ID: PLAN-619；plan_revision: 1。
- 依据：用户实测 4 项现象 + 本计划 §4.1/§4.2 的像素级复核（`tools/parity_shot_diff.py` 可复跑）。
- **用户裁决（2026-09-12）**：
  - §10① 色板 → **选项 A：scaffold/shadcn 为权威**（VM 的语义解析对未声明 `theme{}` 的 scaffold 类应用
    与 Vue CSS 同源；桌面 shell/stella 类应用今后显式声明 `theme{}` 或由宿主显式 `set_theme("stella")`）。
  - §10② icon 尺寸 → **以 `.at` 的 `size:` 为准**（Vue 生成器补 `:size`；VM 按 size 绘制并盒内居中；
    生成器注入的 `w-5 h-5` 仅作缺省、让位于显式 size）。
- **关键前置核实（让选项 A 变成小改动）**：`registry::SCAFFOLD.dark` 与 Vue 侧生成的
  `gen/front/vue/src/assets/index.css` `.dark` 块**逐 token 完全同值**（background `222.2 47% 7%`、
  card `222.2 47% 10%`、muted `217.2 32.6% 15%`、accent/border `217.2 32.6% 17.5%` …；sidebar 8 键同）。
  → 修法 = 让「未声明 `theme{}` 的应用」在 VM 端走 `scaffold` 而非 stella。
- **落点（已定位）**：缺省活动主题硬编码在 `crates/auto-lang/src/ui/style/theme/mod.rs:44`
  （`registry::builtin("stella")`，注释「缺省 "stella" = VM 轨现行」）；桌面宿主不受影响的前提是
  `desktop_config.theme_name` 缺省（`desktop_config.rs:83` = `None`）→ 宿主依赖该进程缺省，
  **因此必须为宿主补显式 stella 激活（或把宿主 config 缺省写为 `Some("stella")`），否则桌面视觉会整体改色**（blast radius 见 §4.3）。
- outcome: **pass**（裁决已回填，可进入 work）。
- next: work —— T-01（P3/P4 有界调查，先出决策件）→ T-03（P1，落点已明确）→ T-02（P2）→ T-04/T-05。

### 9.2 /auto-plan:work 执行记录（2026-09-12）

- stage: work；plan_id: PLAN-619；plan_revision: 1；code_commit: `plan-619-dev`（worktree
  `D:/autostack/.wt/lang-619/auto-lang`）。
- 完成任务：T-01 ✅（决策件 §8.1，R1–R4 四根因 + headless 夹具）、T-02 ◑（`size` 两端贯通 ✅／
  AC-02 ink 预算未达 ❌）、T-03 ✅、T-04 ✅、T-05 ✅（实机双端 + 门禁对拍）。
- 改动面（7 文件 + 1 探针 + 1 契约）：
  `ui/style/iced_adapter.rs`（+`effective_padding/effective_margin/has_margin`）、
  `ui/iced/renderer.rs`（`iced_padding_from_is` 委托 + 三个 margin 消费点 + Text 臂 height）、
  `ui/aura_view_builder.rs`（`with_class_prop` 补 `style:` 回落；另 4 处同类取值点补齐；icon `size:` 折盒）、
  `ui_gen/vue.rs`（icon 发射 `:size` + 撤缺省 `w-5 h-5`）、
  `ui/style/theme/mod.rs`（缺省主题 scaffold）、`ui/desktop_config.rs`（宿主显式 stella）、
  `tools/parity_shot_diff.py`（预算门禁）、`examples/ui/015-notes/tests/acceptance.atd`（T14/C-PARITY-1）、
  `.agents/skills/autoui-verifier/SKILL.md`（步骤 3.5 + 检查项 9）；
  回归测试 7 条（`plan619_*`）+ 受影响既有测试的钉定/期望更新（theme/plan593/musk/desktop_config）。
- 验证证据：`cargo nextest ... -E 'test(plan619)'` 7/7 绿；双端实机探针（§8.2）；`cargo tv`/`cargo t`
  与基线逐条归因（§8.3，零新增红）。
- **遗留 / 阻塞**：AC-02（图标 ink 比 ≥0.9）未达成。`size:` 的贯通已闭环，剩余差距是
  **两端各自的渲染器/资产面**：(a) iced 侧 SVG 栅格化未按盒缩放，(b) Vue 侧 shadcn 资产
  `[&>svg]:size-4` 覆盖 `:size`，(c) `notebook` 两端字形版本不同。三者都不在「引擎解析」层，
  修法（改 iced svg 用法 / 在生成器侧中和资产 CSS / 对齐内嵌 lucide 版本）需用户裁决 → 见 §10③。
- outcome: **blocked**（AC-02 需裁决后续走向；其余 AC 达成）。保持 `executing`。
- next: 用户裁决 §10③ → 定向修复（或降级 AC-02 为债务并记入 KNOWN-DEBT）→ review。

### 9.3 /auto-plan:work 续记：用户裁决 A → P2 根因定案（2026-09-12）

- 用户裁决：走 **A**（三腿一起修），并要求先做定点实验再落刀。
- 定点实验（§8.5）结论**推翻了 (a) 的原始假设**：不是 iced 矢量绘制路径的 DPI 缩放丢失，
  而是 `lucide_svg()` 表项本身是完整 SVG 文档、`lucide_svg_doc_with` 又套了一层
  → 嵌套 viewport 使 glyph 按 **16/24 = 0.667** 缩放。中途曾按「自栅格化 + raster 臂」
  实现候选（`a6897a61f`），被两组逐像素相同的实测证伪后 revert（`b693c828a`）。
- 修法：`lucide_svg_doc_with` 取内层 markup 重包（3 行）；回归锚
  `plan619_lucide_doc_renders_geometric_ink`（dev-deps resvg/tiny-skia，零新增 crate）。
- 复验（`tools/parity_shot_diff.py`，双端实机 1280x800）：**预算全绿 exit 0** ——
  语义面色 Δ0/Δ1（AC-01 ✅）、搜索图标 ink 比 **1.03/1.03**（AC-02 ✅，修前 0.68）、
  左缩进 Δ0/Δ1/Δ0（AC-03 ✅）。
- **AC 状态：AC-01 ✅ / AC-02 ✅ / AC-03 ✅ / AC-04 ✅ / AC-05 ✅（零新增红，见 §8.3/§8.4）。**
- outcome: **pass** → `execution_done`。code_commit：`d3a44aa6f`（+`94e994bd6`）。
- next: /auto-plan:review（独立复审）。残余项（不阻塞 AC）：P619-D2 字形数据对齐、
  shadcn 资产 `[&>svg]:size-4` 盖 `:size`，见 KNOWN-DEBT。

### 9.4 /auto-plan:review 复审记录（2026-09-12）

- stage: review | plan_id: PLAN-619 | plan_revision: 1 | **outcome: needs_fix**
- reviewed_commit: `d3a44aa6f`（分支 `plan-619-dev`；含 `94e994bd6`/`a6897a61f`/`b693c828a`）
- base_commit: `3972645ec`；dependency_revisions: 无跨仓依赖变更；**新增直接依赖仅 `resvg 0.45`/`tiny-skia 0.11`（两者本已在依赖图中，经 iced_wgpu；且只作 dev-dependencies 供回归锚）**——`Cargo.lock` 无新增 crate
- spec_inputs: `docs/specs/auto-lang/ui/overview.md`（69155 B，693/601 主题段为 SD-01 目标；文件在标）
- acceptance_results:
  | AC | 判定 | 复现方法 / 结果 |
  |---|---|---|
  | AC-01 语义面色逐通道 ≤2 | **pass** | 复审期**重新构建**（`plan619_review_vm.png`，新采）后跑 `python tools/parity_shot_diff.py docs/plans/attachments/619/after_vue_initial.png <新采 vm>`：页面/编辑区 **Δ0**、侧栏 **Δ1** |
  | AC-02 图标 ink 比 ≥0.9 | **pass** | 同上：搜索图标（两端同字形判据）**1.03 / 1.03**（修前 0.68）；顶栏 notebook 参考 0.89/0.96（其字形数据两端不同，见 P619-D2） |
  | AC-03 左缩进 ≤1px | **pass** | 同上：搜索行左边界 **Δ0**、分组标签 **Δ1**、便签标题 **Δ0** |
  | AC-04 探针入常驻门禁 | **pass** | ①探针含预算判定；**正例 exit 0 / 负例（616 基线对）exit 1** 双向实测（此前我误用管道取到 `tail` 退出码，复审已用 `pipefail` 直取复核）；②`examples/ui/015-notes/tests/acceptance.atd` 有 T14 + C-PARITY-1；③`.agents/skills/autoui-verifier/SKILL.md` 有「步骤 3.5 像素预算对拍」+ 检查项 9 |
  | AC-05 `cargo tv`+`cargo t` 零新增红；19 MCP + 18 Vue 场景全绿 | **partial** | 前半 ✅：`cargo t`（nextest +ui-iced，no-fail-fast）本分支 26 红 ⊆ 基线 27 红（差者为基线偶发 `osconfig_daemon`）；`cargo tv` 本分支 6 红 = 基线 5 红 + `dep_parity_018_dep_fields`，**归因=环境**（panic 为 `binary not found ... build step missing?`，且**两处 checkout 的 `test/ffi_dual/*/oracle/target/release/*.exe` 均为 0 个**）。后半 ❌ **未验证**：见 F-1/F-2 |
- findings:
  - **F-1（medium，阻塞 AC-05 后半）**：`015-notes.autotest` 19 场景**整批不可验证**——官方 runner（`run_autotest.py` + `autotest/__init__.py`）的解析链路失配，全部卡在 `Button 'X' not found`；而同一时刻**直连 MCP** 的 `autoui_exists/find` 对同名元素（`New note`/`All`/`Search notes`）**全部命中**（108 次图标绘制打印同证 app 正常）。属**预存工具链问题**（P619-D5 在案），非本计划回归；但 AC-05 明列该套必须全绿 → 未验证即未达成。纠正：或以可用驱动（如 `tests/desktop_mcp.py` 同款直调 `autoui_find`→`autoui_press`）重跑该 19 场景并留痕，或修 runner 后重跑。
  - **F-2（medium，阻塞 AC-05 后半）**：18 条 Vue playwright 场景（`tests/smoke.spec.ts` 13 + `tests/accent-dark.spec.ts` 5）**本轮未运行**——`tests/node_modules` 下缺 `@playwright/test`（browser 二进制在 `~/AppData/Local/ms-playwright` 有）。纠正：`cd examples/ui/015-notes/tests && pnpm i && NOTES_URL=http://localhost:<front-port> pnpm test`（需先起 Vue 端），结果回填本表。
  - **F-3（low，非阻塞）**：桌面宿主 stella 观感的**实机截图复核未做**（计划 §5.1/T-03 明确要求「这条别省」）。已用单测锁住配置语义（`theme_name_roundtrip` 覆盖「存量缺键 → stella」「空串 → stella」），但宿主整体观感未实机留痕。纠正：起 `auto-os` 桌面宿主截图与 616 存档对比。
  - **F-4（info，方法学）**：`lucide_svg()` 表头注释（"SVG wrapper: 16x16"）与实际被消费方式长期不符（消费方误当 fragment 用），已在本计划内同时修正注释与消费点；复审认为属**根因级修复**而非 workaround（改动 3 行 + 一条纯 CPU 回归锚，锚在修前必红）。
  - 猎查：无 dropped 子项、无未批准的范围收缩；`a6897a61f`（被证伪的候选）已 revert 且留档，不构成遗留；新增依赖仅 dev 档且零新 crate。
- evidence（复审期重跑）：`plan619_review_vm.png`（新采 VM 帧）、`tools/parity_shot_diff.py` 正/负例退出码、`cargo t`/`cargo tv` 失败集合与基线差、`plan619_lucide_doc_renders_geometric_ink` 与 5 条 `plan619_*` 全绿、`desktop_mcp.py` 12 passed/1 stale/1 skip。
- spec delta：SD-01/SD-02 已在 §5.2 落为可入册文本；frontmatter `supersedes/new_spec_components/touched_goals` 已填（GOAL-007）。
- next: 回 `/auto-plan:work` 收口 F-1/F-2（AC-05 后半），F-3 由能起桌面宿主的环境补；完成后重新复审（证据需重新绑定）。

### 9.5 复审 findings 收口记录（2026-09-12，/auto-plan:work）

- **F-1 关闭**（`6aad8ba1f`）：19 MCP 场景 **19/19 绿**。**并订正复审期的归因**——真因不是
  runner 解析链路失配，而是**就绪竞态**（驱动只等端口；空树导致前 16 场景连锁失败、尾部
  3 条因 app 已就绪而偶过）。修法 = `McpAdapter::wait_ready()`（tree + state + button 三判据，
  180s 超时显式 FAIL）+ `run_suite` 首步调用 → 官方入口自带稳健性。
- **F-2 关闭**：18 条 Vue playwright 场景 **18/18 绿**（`pnpm install --offline` 装
  `@playwright/test@1.63.0`；`NOTES_URL=http://localhost:6111`）。
- **F-3 未关闭（保留）**：桌面宿主 stella 观感实机截图——需能起 `auto-os` 宿主的环境。
- **AC-05 判定更新：pass**（前半零新增红 + 两套场景全绿）。**五条 AC 全绿**。
- outcome: **pass** → `execution_done`（revised）。code_commit: `6aad8ba1f`
  （实现面仍为 `d3a44aa6f`：本次只改示例测试资产，`crates/**` 零改动——AC-01..04 的
  复审证据因此仍有效，无需重跑）。next: 重新复审以绑定新 revision。

### 9.6 /auto-plan:review 定向复审（2026-09-12，F-1/F-2 收口后重绑证据）

- stage: review | plan_id: PLAN-619 | plan_revision: 1 | **outcome: pass**
- reviewed_commit: `77c366202`（= `6aad8ba1f` 的最终 SHA）；base_commit: `3972645ec`
- dependency_revisions: 无跨仓依赖变更；直接依赖新增仅 dev 档 `resvg 0.45`/`tiny-skia 0.11`（已在依赖图内，`Cargo.lock` 零新 crate）
- spec_inputs: `docs/specs/auto-lang/ui/overview.md`（SD-01 目标节在标）
- **复用上轮证据的理由（显式）**：`git diff d3a44aa6f..HEAD -- crates/ tools/` **为空** —— 实现面与探针零改动，仅新增示例测试资产（`tests/autotest/__init__.py` 的 `wait_ready`）。故 AC-01..AC-04 的复现证据（新采 VM 帧 + 探针正负例退出码 + 门禁差集）仍绑定当前 revision，无需重跑；AC-05 用本轮新证据。
- acceptance_results:
  | AC | 判定 | 证据（本轮） |
  |---|---|---|
  | AC-01 | **pass** | 语义面色 Δ0（页面/编辑区）/Δ1（侧栏），预算 ≤2 —— §9.4 复现证据，实现面未变 |
  | AC-02 | **pass** | 搜索图标 ink 比 **1.03/1.03**（预算 [0.9,1.1]），同上 |
  | AC-03 | **pass** | 左缩进 Δ0/Δ1/Δ0，同上 |
  | AC-04 | **pass** | 探针正例 exit 0 / 负例 exit 1（双向实测）；acceptance T14 + C-PARITY-1 + 技能清单在案；`tools/` 本轮零改动 |
  | AC-05 | **pass** | ①`cargo t` 26 红 ⊆ 基线 27、`cargo tv` 6 = 基线 5 + `dep_parity_018`（oracle 二进制两处 checkout 均 0 个 → 环境类）②**19 MCP 场景 19/19 绿**（F-1 收口）③**18 Vue playwright 场景 18/18 绿**（F-2 收口） |
- findings（本轮）：
  - F-1 **closed**（并订正上轮归因：真因是就绪竞态而非 runner 解析失配；修入 `McpAdapter::wait_ready`，官方入口自带稳健性）。
  - F-2 **closed**（`@playwright/test@1.63.0` 离线装入示例 tests/，18/18 绿）。
  - F-3 **open（非阻塞，计划外环境）**：桌面宿主 stella 观感实机截图——需能起 `auto-os` 宿主的环境；配置语义已有单测锁定（`theme_name_roundtrip`）。
  - F-4（info，上轮）保持：`lucide_svg` 表头注释与消费方式不符属根因级修复。
  - 猎查：无 dropped 子项、无未批准范围收缩、无 workaround；本轮改动未触实现面，未新增债务（P619-D5 已关闭并订正）。
- spec delta：SD-01/SD-02 文本已在 §5.2 定稿（可入册）；frontmatter `supersedes_spec_components`/`new_spec_components`/`touched_goals: [GOAL-007]` 已填。本复审未发布 spec、未改账本。
- 结论：**五条 AC 全绿、无阻塞 finding（F-3 属计划外环境项）→ status: reviewed**，可进 `/auto-plan:merge`。

### 9.7 /auto-plan:merge 整合收据（2026-09-12）——`PLAN-619:r1`

- stage: merge | plan_id: PLAN-619 | plan_revision: 1 | **outcome: blocked（landing 前置被占）**
- 门禁：`status: reviewed` + `pass`（§9.6，reviewed_commit `77c366202`）✅；无未完成必需任务
  （AC-01..05 全绿）✅

| Checkpoint | 状态 | 证据 |
|---|---|---|
| `prepared` | ✅ 完成 | ①**reconcile**：master 已前进（`3972645ec` → `012b30832`，PLAN-617 折入含真实代码，其中 `crates/auto-lang/src/ui_gen/vue.rs` 与 `ui/mod.rs` 命中我的落地面）→ 在 worktree `git merge master`（合并提交 `ef09a3035`，**零冲突**，我的 `:size` 改动与回归锚均在）；②**刷新受影响验证（合并基线上重跑）**：`plan619_*` 8/8 绿；重建 `auto` 后**新采 VM 帧** → 探针 **exit 0**（面色 Δ0/Δ1、ink 1.03/1.03、缩进 Δ0/Δ1/Δ0）；**19 MCP 场景 19/19**；**18 Vue 场景 18/18**（首轮曾 4 红，定位为驱动侧「数据未就绪」同款竞态——补 `data_ok` 闸门后复现稳定绿）；③**规范增量**：SD-01/SD-02 落 `docs/specs/auto-lang/ui/overview.md`（现状节 + 已知坑节）+ `plans.md` 619 行；`python scripts/spec-index.py` 复跑 26 projects 无断链（INDEX.md 内容零变化）；④**delivery_commit**：`6ecb5cf7d`（reviewed 的**纯文档后代**——`crates/**` 与依赖零改动，符合 merge 技能「文档/投影专属后代」条款）；⑤**spec 冻结**：frontmatter `supersedes/new_spec_components/touched_goals: [GOAL-007]` 已填（§5.2 文本即冻结 delta） |
| `landed` | ⛔ **阻塞** | 主检出（master 唯一检出）有 **37 个在途脏文件**（PLAN-617/618），与我的落地面**交集 3 个**：`crates/auto-lang/src/ui/aura_view_builder.rs`、`crates/auto-lang/src/ui/iced/renderer.rs`、`crates/auto-lang/src/ui/iced/layout_tests.rs`。我的提交会改写这 3 个文件 → 任何落地方式（含 `--ff`，本分支已含 master）都必须更新工作树 → git 会拒绝（避免覆盖他人未提交改动）；按用户既定约束「主检出 crates/** 他人在途改动不要碰、不要提交」，我**未**做 stash/覆盖。改动未落地前不得归档（技能：先落地+账本校验，后归档） |
| `ledger_refreshed` | ⏸ 待落地后 | 待投影目标：`.autoos/specs.json`（本仓为**主检出运行时态**、未跟踪）+ 已完成的 canonical 回写（`ui/overview.md`/`plans.md`/INDEX 再生）。按技能「runtime-only ledger data 在 canonical Specs 落地后发布」→ 随 landing 一并做 |
| `archived` | ⏸ 未做（被 landing 阻塞） | 计划保持 `reviewed`（技能：归档前 publication-only blocker 时维持 reviewed） |
| `cleaned` | ⏸ 未做 | worktree/branch 保留：`D:/autostack/.wt/lang-619/auto-lang` @ `plan-619-dev` |

- 精确解绑动作（三步，缺一不可）：
  1. **由 3 个文件的在途 owner（PLAN-617/618）提交或暂存其改动**（或用户授权我暂存）；
  2. `cd /d/autostack/auto-lang && git merge plan-619-dev`（本分支已含 master `012b30832`，
     预期 fast-forward；落地后 master 应含 `94e994bd6..6ecb5cf7d` 全部提交）；
  3. 归档 + 清理：`git mv docs/plans/619-015-notes-vm-vue-parity.md docs/plans/archive/` →
     `status: archived` + 主检出各自最终簿记提交 → `bash D:/autostack/wt-guard.sh
     D:/autostack/.wt/lang-619/auto-lang`（必须 clean；`examples/**/node_modules` 的 pnpm
     junction 用 `cmd /c rmdir` 逐链清）→ `git worktree remove` + `git branch -d plan-619-dev`。
- 备注：本收据点后若发生对 reviewed 契约或实现的改动，受影响证据失效，须回 review 重绑。

### 9.8 合并收据刷新（2026-09-13，PLAN-618 已落地后）

- **reconcile #2**：master 再前进（`012b30832` → `56dbb1283`，PLAN-618 全量落地）→ 其中
  `crates/auto-lang/src/ui/iced/renderer.rs`、`crates/auto-lang/src/ui_gen/vue.rs`、
  `docs/specs/auto-lang/ui/plans.md` **命中我的落地面** → 工作树 `git merge master`：
  代码文件自动合并，**唯一冲突在 `plans.md` 表格行**（双方各追加一行）→ 按计划号保留两行
  （618 在前、619 在后）并提交（合并提交 `5241b0771`）。我的三处关键改动逐一复核幸存
  （`size_attr` / lucide 内层重包 `{inner}` / `effective_style::effective_padding`）。
- **刷新验证（全部在 reconcile 后的基线上重跑）**：
  | 项 | 结果 |
  |---|---|
  | `plan619_*` 回归锚 | 8/8 绿 |
  | 双端探针（重建 `auto` + 新采 VM 帧 `plan619_merge2_vm.png`） | **exit 0**：面色 Δ0/Δ1、图标 ink 0.89/0.96（notebook 参考）与 **1.03/1.03**（同字形判据）、左缩进 Δ0/Δ1/Δ0 |
  | 19 MCP 场景 | **19/19** |
  | 18 Vue playwright 场景 | **18/18** |
- **构建基建注记（P617-D6 用户转达 + 本机实测）**：本机 `RUSTC_WRAPPER` 指向 sccache，
  其缓存超限会以 `os error 5` 让构建失败 → 本轮所有重建均以 **`RUSTC_WRAPPER=`** 执行，
  且**重建后的探针/场景结果与先前逐项一致**，确认既有证据未被 sccache 污染。
- **过程事故（自因，已定位并修）**：我的 MCP 驱动曾把端口硬编码成 `9247`，而 9247 被一个
  残留 app 占着 → app `failed to bind` → 就绪闸门报 `T-ready FAIL`（**不是** 618 引入的回归）。
  修：驱动改 `pick_free_port()` 随机空闲端口 + 清残留进程 → 19/19 恢复。就绪闸门的价值在此
  得到印证：把「19 场景全灭」变成一句精确诊断。
- **外部事件注记（他方转达）**：合并期间主检出的在途改动被**暂存保护过一次并原样 pop 回**
  （stash 记录 `f51b6501` 已清）—— 与我的落地面判断无关，仅记录以解释主检出可能出现的短时扰动。
- **landing 仍阻塞（原因更新）**：主检出的在途脏改动（PLAN-617，**仍在进行中**）与我的落地面
  交集仍是那 3 个文件（`ui/aura_view_builder.rs`、`ui/iced/renderer.rs`、`ui/iced/layout_tests.rs`）；
  用户已示意**等 PLAN-617 完成后**再落地。解绑三步同 §9.7（`git merge plan-619-dev` 预期可
  fast-forward：本分支已含 master；随后归档 + wt-guard + 清理）。

### 9.9 解绑落地：reconcile #3 + 全量验证刷新（2026-09-14，/auto-plan:merge 续）

- **阻塞解除确认**：PLAN-617/618 均已并入 master（主检出 37 个在途脏文件清零，仅剩根目录
  4 个 p618_*.txt 未跟踪杂物，与落地面零交集）。master 自 reconcile #2（`56dbb1283`）前进
  46 提交（617 代码 `5e6c1005e` + 账本 + 归档收据 + auto-man 修复×3）。
- **reconcile #3**（合并提交 `504319c82`）：`git merge master` 三冲突，裁定原则 =
  「617 对同一契约实现了更强机制，取 617；619 独有根因修复保住」：
  | 冲突 | 裁定 | 依据 |
  |---|---|---|
  | `aura_view_builder.rs` icon size 臂 | 取 617 `with_icon_size`（显式 w/h 类 > size > 缺省 20px） | 619 的「size>0 前置注入」是其子集；617 额外覆盖块式 `style:`、动态 class、缺省 20px 双端对齐（619 版缺省时 VM 16 / Vue 20 仍裂） |
  | `ui_gen/vue.rs` icon 发射 | 取 617 内联 `style="width:Npx;height:Npx"` | 619 的 `:size` 绑定被取代：内联样式特异性压过 shadcn 资产 `[&>svg]:size-*`（顺带缩小 P619-D6 受害面）；生成物实证 `<Search class="text-muted-foreground" style="width:14px;height:14px"/>`；`plan619_icon_size_prop_emits_bound_size` 断言同步改为新契约 |
  | `renderer.rs` lucide 表 | 取 617 全量生成表（1401 项，lucide-vue-next v0.312 同源） | **根治 P619-D2 字形分叉**（构造级「同名 ⇒ 同字形」）；**但 617 换表时双重嵌套坑（619 §8.5 根因）一度复发**——`lucide_svg` 仍包 16×16 完整文档、`doc_with` 再套 24×24。修法 = `lucide_svg_doc_with` 改为直接消费 `lucide_fragment` **单层包装**（619 回归锚守住该契约） |
- **基建注记**：617 新增 workspace 成员 `a2r-actor-tests` 的 `autodown-core` path 依赖要求组内
  兄弟 → 按规约补**真实 worktree** `D:/autostack/.wt/lang-619/auto-down`（detached @ auto-down
  master `ad5b1d4`，非 junction），随 cleaned 步骤一并移除。
- **规范同步**：overview.md SD-02 ②③ 改述合并后机制（现状节 + 已知坑节）；KNOWN-DEBT
  **P619-D2 划掉**（617 根治，本次合并确证）、D1 注记刷新（含「617 换表时复发」事实）、
  D6 受害面缩小（显式 size 走内联样式已可压过资产 CSS）；frontmatter `new_spec_components`
  对齐最终口径。契约本身（size 权威、缺省让位、显式类优先）未变——变的是实现机制描述。
- **验证刷新（全部在 reconcile #3 后基线，私有 `auto619.exe` 防 §8.2③ 二进制污染）**：
  | 项 | 结果 |
  |---|---|
  | `plan619_*` 回归锚 | **8/8 绿**（含改述后的 Vue 断言与几何锚） |
  | icon/lucide 面 | **26/26 绿**（含 617 自己的 `test_icon_size_precedence`、覆盖率、缓存去重） |
  | `cargo tv`（no-fail-fast） | 3686 过 / 5 红 = `ffi_dep_parity_*` oracle 缺失（§8.3 在案环境预存）——**零新增** |
  | `cargo t`（daily，no-fail-fast） | 24 红全部归因：17×`ui::layout::tests`（任务栏高度环境族）+ 5×`ffi_dep_parity` + `c2_param_msg`/`scan_examples`/`strips_tags`/`covered_elements`（master 基线同红，本机复跑实证）+ `external_config_poll_hot_apply_loopsafe`（同窗口 master 连跑 6 次也翻 2 次 = osconfig 环境偶发族，§9.6 在案）——**零新增确定性红** |
  | 双端探针（**双端新采**，`merge3_vue.png`/`merge3_vm.png` 归档 attachments/619） | **exit 0**：面色 Δ0/Δ1（AC-01）、搜索图标 ink **1.03/1.03**（AC-02）、notebook 参考 **0.98/0.96**（升自 0.89/0.96——617 全量表根治字形分叉的直接证据）、左缩进 Δ0/Δ1/Δ0（AC-03） |
  | 19 MCP 场景（`--url` 指向自起实例，随机空闲端口） | **19/19 绿** |
  | 18 Vue playwright 场景（`NOTES_URL` 指向本轮 6112 实例） | **18/18 绿（59.4s）** |
- 过程卫生：本轮起的服务全部关闭；§9.8 遗留的 stale vite（6111，PID 19092，同 worktree gen 目录）一并清除。
- outcome: **pass**（五条 AC 在合并后基线全部复验）。next: 落地（ff）→ 账本投影 → 归档 → 清理。

## 10. 待澄清事项

1. ~~**P1 权威色板**~~ → **已裁决（2026-09-12）：选项 A，scaffold/shadcn 为权威**。
   实施注意（衍生约束，不再征询）：桌面宿主必须显式声明 stella 才能保住现有观感——
   优先做法是把 `desktop_config::DesktopConfig::default().theme_name` 写为 `Some("stella")`
   （`crates/auto-lang/src/ui/desktop_config.rs:83`），使「宿主 = stella、pac 应用 = scaffold」成为显式规则，
   并在 SD-01 成文。
2. ~~**P2 的 `size` 权威**~~ → **已裁决（2026-09-12）：以 `.at` 的 `size:` 为准**；
   `ui_gen/vue.rs` 的 `w-5 h-5` 降级为缺省（显式 `size:` 时让位），写进 SD-02。
   实现已完成（两端贯通，§8.1/§8.2 有证据）。
3. ~~**AC-02 的 ink 预算未达 —— 后续走向？**~~ → **已裁决并解决（2026-09-12）**：
   用户选择 **A**（三腿一起修 + 先定点实验）。实验把真根因定在「我们自己的 SVG 文档双重
   嵌套」（§8.5），修复 3 行即让 AC-02 达标（1.03/1.03）；(a) 原假设（iced 矢量臂 DPI
   缩放丢失）被两组逐像素相同实测证伪，(b)/(c) 降为不阻塞的残余项（P619-D2）。
   原三选项讨论保留如下（历史）：
   现状：`size:` 贯通后 VM 盒宽 = 声明值（可证），但同 glyph 的 ink 仍只有浏览器的 ≈0.7；
   且顶栏 `notebook` 两端字形版本不同。已定位三个独立面（§8.2 ②）：
   (a) iced 侧 `iced_wgpu` SVG 栅格化/quad 缩放未按盒；(b) Vue 侧 shadcn sidebar 资产
   `[&>svg]:size-4` 把图标钉死 16px、覆盖 `:size`；(c) VM 内嵌 lucide fragment 与
   `lucide-vue-next@0.312` 的 `notebook` 字形不同。
   三个候选走向：
   - **A（推荐，收敛真 parity）**：三面一起修 —— (b) 在生成器侧对显式 `size:` 输出
     `!size-[Npx]`（或包裹层）中和资产 CSS；(a) 在 `View::Image` 的 lucide 臂显式
     `content_fit`/按盒声明 SVG 尺寸并加 headless 断言；(c) 按 `lucide-vue-next` 的版本
     对齐内嵌 fragment 集（可加一条「同名同 path」契约测试护住）。
   - **B（拆新计划）**：本轮只交「`size:` 贯通 + 门禁」，把 AC-02 降级为
     `KNOWN-DEBT-AND-RISKS.md` 一条（含本节证据），另立 ICON-PARITY 小计划。
   - **C（重定 AC）**：把 AC-02 改为「盒宽一致 + ink 比 ≥0.6」并成文说明
     （不推荐：等于把当前实现固化为标准，且掩盖 (b) 的 CSS 覆盖问题）。
   用户裁决前保持 `executing`（其余 AC 已达成，见 §8.2/§8.3）。
