# Plan 409: Widgets Gallery — 三模式一致性 + link 子组件 VM 缺口 + 主题色

> **状态**: §1–§9 ✅；§6（`link` 子组件 VM 渲染缺口）与 §8（主题色支持）均已按方案实施并通过回归测试。§9 为本会话 vue 模式审查修复批次（codegen bug + 内容一致性 + 交互演示 + 样式），CodeBlock/PreviewCard 改纯 Auto widget 暂缓。**§10 进行中**（2026-08-12 新一轮 VM 模式审查，6 个残留差距，worktree `plan-409`）。
> **仓库**: **auto-lang**（`crates/auto-lang/src/ui/{iced/renderer.rs, aura_view_builder.rs, widget_registry.rs, style/iced_adapter.rs}` + `ui_gen/{vue.rs, rust.rs}` + `token.rs` + `lib.rs` + `aura/types.rs`）；gallery 产物在 `examples/widgets-gallery/`。
> **背景**: `examples/widgets-gallery` 是覆盖全部 ~50 个 AutoUI widget 的组件画廊，同时作为 vue/vm/rust 三模式一致性与 codegen 缺口的试金石。本计划把 gallery 推进过程中暴露并修复的 **VM 一致性 + Vue codegen** 问题统一登记，并把仍开放的 **`link` 子组件 VM 渲染缺口** 作为唯一待办立项。
> **说明**: §1–§5 的修复在历史上多挂在 Plan 408（VM/gallery track）名下提交，此处按"按计划修复"的视角统一归档为 Plan 409 的已完成项；引用的 commit hash 可追溯。

---

## 0. 背景与现状

### 0.1 gallery 的三模式目标

`examples/widgets-gallery` 用同一份 Auto 源码（`src/front/app.at` + 49 个 `pages/*.at`）在三种渲染模式下呈现同一组件文档站：

| 模式 | 渲染器 | 说明 |
|---|---|---|
| `vue` | 浏览器 + shadcn-vue（`ui_gen/vue.rs`） | 基线，最完整 |
| `vm` | AutoVM 解释器 + iced 原生窗口（GPU/wgpu） | 本计划一致性差距的主战场 |
| `rust` | GPUI/iced 编译产物（`ui_gen/rust.rs`） | 静态转译，覆盖基本元素 |

### 0.2 VM 一致性差距的由来

vue 模式依赖 **CSS 变量继承**（`body { color: var(--text-foreground) }`、`<html class="dark">`），而 vm 模式的 iced 渲染器**没有继承机制**——每个 widget 必须显式拿到颜色/样式。这导致 vm 模式在以下场景与 vue 产生可见差距：路由 `<outlet>`、导航点击、图标、深色模式文字。本计划逐项消除这些差距。

---

## 1. ✅ VM 路由与导航一致性

| 子项 | 现象 | 修复 | 提交 |
|---|---|---|---|
| 1.1 `<outlet>` 不渲染 | 路由页加载后内容区空白 | `WidgetRegistry` 新增 `route_aliases: HashMap<String,String>`（模块名→widget 名），`lib.rs` 注册路由时调 `register_route_alias`，`render_outlet` 改用 `get_by_route_module` | `8cdf47b1` |
| 1.2 nav-link 不可点击/不渲染 | 被当普通 Element 落到空 fallback | `aura_view_builder.rs` 的 `convert_element` fallback 增加 `nav-link`/`nav_link` 识别 → 渲染为带 `to` 的 Button | `2b2db8a0` `f17d28ba` |
| 1.3 VM 栈溢出 | 深层 view 树递归炸栈 | `.cargo/config.toml` 加 `rustflags = ["-C", "link-arg=/STACK:33554432"]`（32MB） | `8cdf47b1` |
| 1.4 `grid` 保留字冲突 | `grid` 无法做路由/widget 名 | 从 `token.rs` 的 `keyword_kind` 移除 `grid` | `accd65d3` |

**用户原始诉求**: "vm模式现在最大的问题是 `<outlet>` 不支持"；"左侧导航栏的导航项无法点击"；"左侧导航栏的列表数量就差了很多"。

---

## 2. ✅ VM 图标支持

vue 侧 nav-link 的 `icon` 属性走 lucide-vue-next，vm 侧原先无任何 SVG/图标能力，导致导航项无图标。

| 子项 | 修复 | 提交 |
|---|---|---|
| 2.1 Icon widget | 新增 `stdlib/aura/widgets/display/Icon.at`（`#[primary] name str`），`mod.at` 注册 | `83a0f0c5` |
| 2.2 `lucide:` 协议 | view builder 把 `icon (name:"bell")` → `View::Image { src: "lucide:bell" }`；renderer 检测 `lucide:` 前缀 → `lucide_svg()` 取 SVG → `iced::widget::svg` 渲染 | `83a0f0c5` |
| 2.3 nav-link 内嵌图标 | nav-link 的 `icon` 属性用 PUA 标记 `\u{EE01}name\u{EE02}` 嵌入 Button label（Button 只接受 String label，不支持子元素，PUA 标记是 button 内嵌图标的合理方案） | `f3f54629` `2381e771` |
| 2.4 图标表 | `lucide_svg()` 覆盖 gallery 用到的图标（home/layers/bell/check/chevron-*/mouse-pointer-click/…） | `83a0f0c5` `2381e771` |

**用户原始诉求**: "侧边栏的导航项没有显示图标…icon 属性 vue 版实现了而 VM 版应该是没有实现的"；"网页上用的是 emoji 吗？为什么不统一用 lucide icon 呢？"。

---

## 3. ✅ VM 深色模式文字可见性

vm 窗口硬编码 `Theme::Dark`，但 `DARK_MODE` 默认 `false`，导致语义色按浅色解析、文字偏黑，与深色背景融合不可见。

| 子项 | 修复 | 提交 |
|---|---|---|
| 3.1 默认对齐深色 | `iced_adapter.rs` `DARK_MODE` 默认改 `true`（与硬编码 `Theme::Dark` 一致）；`resolve_semantic_rgb` 由 `fn` 改 `pub fn` 供 renderer 调用 | `3dd05153` |
| 3.2 Text 默认色 | 无显式 `text_color` 时，Text 用 `resolve_semantic_rgb(Color::OnBackground)` 作默认色（等价 vue 的 `body { text-foreground }` 继承） | `3dd05153` |
| 3.3 有样式 Button | `build_button_style` 的 `text_color` 缺省值改用 `OnBackground`（深色感知） | `3dd05153` |
| 3.4 **无样式 Button** | nav-link 渲染为 `style: None` 的 Button，走 chromeless `else` 分支，原先 `..Default::default()` 导致 `text_color` 仍是 iced 默认黑色。补 `text_color: default_text`（`OnBackground`，回退 `WHITE`） | `38e4d5fc` |

**用户原始诉求**: "现在 widget gallery 模式是深色模式，启动后左侧导航栏的字体也是黑色的，导致两者融合在一起看不清楚"；"参考截图，侧边导航栏仍然是深色字体啊？"（3.4 正是对这轮反馈的补漏）。

---

## 4. ✅ Vue codegen 缺口修复（gallery §9 暴露的 7 + 标题样式）

gallery 作为 codegen 试金石暴露的一批 vue 代码生成缺陷，集中修复。

| 子项 | 现象 | 修复 | 提交 |
|---|---|---|---|
| 4.1 Slider 类型 | `v-model` 与 `number[]` 类型冲突（TS2322） | 改 `:default-value="[val]"` | `accd65d3` |
| 4.2 Drawer 依赖 | `vaul-vue` 模块未找到（TS2307） | `auto-man/src/vue.rs` package.json 加 `vaul-vue` 依赖 | `accd65d3` |
| 4.3 Toast 组件缺失 | toast/toast-title/toast-description 未生成 | `vue.rs` 移除 `map_tag` 早返回 | `accd65d3` `6bd78d03` |
| 4.4 NavLink href | href 未透传给 router-link `to` | `props.get("to").or_else(\|\| props.get("href"))` | `accd65d3` |
| 4.5 Pagination 组件名 | shadcn-vue 导出名错 | 改正 PaginationContent/Item/Previous | `accd65d3` |
| 4.6 rust outlet/link | `View::outlet()` 不存在 | Outlet→`View::empty()`，Link→`View::text_styled(...)` | `accd65d3` |
| 4.7 h1 默认样式 | 标题无字号/字重 | h1=`text-3xl font-bold tracking-tight`，h2/h3 同步 | `926d7b8d` |
| 4.8 标题下方留白 | 标题与正文贴在一起 | h1/h2=`mb-4`，h3=`mb-3` | `8f0f9024` |
| 4.9 组件介绍用 H2 | 介绍首段标题用了普通字体 | 统一组件介绍的标题为 H2 | （随 4.8 批次） |

**用户原始诉求**: "逐个修复 §9 的 7 个缺口"；"h1 和 h2 下方最好都默认留一点空白"；"vue 版本的每个 widget 内容里的第一段说明，开头的标题都没有用 H2 字体…请统一都用 H2 做标题"。

---

## 5. ✅ Gallery 内容与结构

| 子项 | 内容 | 提交 |
|---|---|---|
| 5.1 全量组件迁移 | Phase 2 B1–B7 把剩余 ~40 个 widget 迁入，达 50 路由 / 49 文档页 | `87398dec` `f3b7c49b` |
| 5.2 介绍文字丰富 | 49 个组件页面首段介绍由过简改为有意义的说明 | `decc4a4d` |
| 5.3 Toast 可交互 | 用 vue-sonner escape hatch 重写 toast 页（vm 静态降级版兼容） | `6bd78d03` |
| 5.4 目录结构 | `source/front → src/front` + vm 模式兼容 | `12188c28` |
| 5.5 纯前端确认 | 确认 gallery 为纯前端 app（无后端），可跑 `render: "vm"` | （决策记录） |
| 5.6 Home 项改 nav-link | 侧边栏 Home 原用手写 `link { row { icon + text } }`，VM 下只渲染 `to` 值 `/`；改为一致的 `nav-link (icon:"home", label:"Home")` | `e4d5a201` |

**用户原始诉求**: "Phase 2 按 B1–B7 分批迁移"；"每一个组件的页面中，第一段是该组件的介绍…大多都太简略了"；"OK，按你的建议去实现 toast 组件"；"侧边栏第一个链接…VM 里…展示成了 `/`…改用 VM 已经支持的 nav-link"。

---

## 6. ✅ 已修复：`link` 子组件 VM 渲染缺口（本计划核心）

### 6.1 现象与证据

`link` 是通用导航原语，vue 侧支持任意子组件（`link (to:) { text / row / icon ... }`）。vm 侧的 view builder 把 `link` 当**叶子**处理——只取 `to:` 值作为 Button 文本，**丢弃全部子节点**。

**探针**（`autoui_snapshot`，gallery 顶部导航栏）：

```
button "/"            ← link (to: "/") { text "Docs" }      → 只剩 to 值
button "/button"      ← link (to: "/button") { text "Components" } → 只剩 to 值
```

vue 侧正常渲染 "Docs"/"Components" 文字并具备点击导航；vm 侧文字丢失（只剩路由路径）。

### 6.2 影响范围

- **顶部导航栏**（`app.at` 的 Desktop nav：Docs / Components）——当前唯一受影响的可见场景。
- **通用原语正确性**：`link` 语义上应与 vue 对齐（可包裹任意子节点），否则任何 `link { children }` 在 vm 下都会降级为纯路由字符串，限制 vm 模式的可表达性。
- §5.6 的 Home 项是用 `nav-link` 绕开此缺口的一例；顶部导航栏因语义不同（header bar 链接，非侧边栏 nav-link）不适合同样绕开，应正面修复 `link`。

### 6.3 根因

`aura_view_builder.rs` 中 `link` 的转换路径：
- 提取 `to:` → 构造一个 onClick 导航消息的 Button；
- Button 的 label **直接用 `to` 字符串**，未调用子节点转换（`convert_children` / `convert_element` 递归）。
- `View::Button` 只接受 `String` label（不支持子元素树），这是结构性的约束——需要把 `link` 的子节点转成一个容器（row/column）作为 Button 的 content，或引入一个新的"可点击容器"视图。

### 6.4 修复方案（已按方案 A 实施）

**方案 A（推荐）：`link` 子节点 → 可点击容器** —— ✅ 采用

1. ✅ `View::Button` 新增 `content: Option<Box<View<M>>>` 字段（`view.rs`）。`Some` 时 renderer 以该子树作为按钮内容渲染；`None` 时保持 label-only 行为。
2. ✅ `aura_view_builder.rs` `render_link_button_with_icon`：若有子节点，递归 `convert_node_with` 得到子 `View` 列表，过滤 `Empty`；单个子节点直接用该节点，多个包成 `View::Row`；渲染为带 onClick 导航的透明 Button，其 `content` = 该容器（而非 `to` 字符串）。
3. ✅ 无子节点时保持现状（用 `to` 或 label 作文本），且 icon PUA 标记只在无 content 时嵌入 label（nav-link 走此路径）。
4. ✅ label 提取升级为 `extract_children_text`（递归解析 `text (text: "Docs")` 元素子节点），使 snapshot 显示 `button "Docs"` 而非 `button "/"`。
5. ✅ `iced/renderer.rs` Button 渲染：`content` 存在时优先 `into_iced()` 渲染容器；`convert_view_messages` 递归映射 content；snapshot builder 继续用 label（无改动）。
6. ✅ gpui renderer 两处 `View::Button` 模式补 `on_right_click`/`content`（顺带修复了 master 上既有的 gpui feature 编译错误）。
7. ✅ 回归测试 `plan409_tests.rs`：构建真实 gallery app，断言顶部导航 `button "Docs"` / `button "Components"` 带 content 子树。

**方案 B（降级，未采用）**：`link` 子节点只取首个文本节点——会丢 `row { icon + text }` 的图标，与 §2 图标能力背道而驰。

### 6.5 验证

1. ✅ **vm 模式**：`plan409_tests::link_children_render_as_button_content` 断言顶部导航渲染为 `button "Docs"` / `button "Components"` 且带 content 子树（`cargo test -p auto-lang --features ui-iced --lib plan409` 全绿）。
2. ✅ **回归**：`cargo test -p auto-lang` 通过（21 个失败均为既有的 dstr/ark/codegen 遗留，与本次改动无关，clean tree 上同样失败）；`ui-iced`/`ui-gpui`/`ui-headless` feature 编译通过；nav-link/无子节点 link 的 label 路径不变（`plain_link_without_children_keeps_to_label`）。
3. ✅ **vue 不变**：`ui_gen/vue.rs` 未改动，vue codegen 路径无回归。

---

## 7. 验收与已知遗留

### 7.1 已完成验收（§1–§6）

- vm 模式 gallery 可启动、路由可切换、侧边栏可点击导航（MCP `autoui_action` 点击验证）。
- 深色模式：侧边栏文字可见——截图像素采样，左侧栏浅色像素占比 ~13.4%（修复前 ~0%），背景 `(43,45,49)`。
- Home 项视图树：`button "homeHome"`（PUA 图标 + 标签），与其余 nav-link 一致。
- vue 模式：49 组件页标题统一 H2、h1/h2 有下方留白、Slider/Drawer/Toast/Pagination/NavLink 均正常。
- §6 `link` 子组件：顶部导航渲染为 `button "Docs"` / `button "Components"`（带 content 子树），`plan409_tests` 回归全绿。

### 7.2 已知遗留

| 项 | 状态 | 说明 |
|---|---|---|
| 顶部标题栏视觉差距 | ⚪ 接受 | vue/vm 渲染器本质差异（CSS vs GPU），视觉差距属预期，非缺陷 |
| `render: "rust"` 模式 | ⚪ 部分 | rust codegen 覆盖基本元素，复杂组件仍有缺口（独立于本计划）；gpui renderer 的 `content` 子树按 label 降级渲染 |
| `cargo test` 既有失败 | ⚪ 既有 | dstr/ark/codegen 等 21 项失败在 clean tree 同样存在，与 §6 无关 |
| ui-iced 测试栈溢出 | ⚪ 既有 | 深层 view 树在 test 线程小栈下溢出（clean tree 同样），非 §6 引入 |

### 7.3 文件清单（本计划触及）

```
crates/auto-lang/src/
  ui/iced/renderer.rs              # lucide: 渲染 / 深色 Text+Button 默认色 / 无样式 Button chromeless / Button content 优先渲染
  ui/aura_view_builder.rs          # nav-link fallback / icon→lucide / outlet route alias / link 子节点 → content 容器
  ui/widget_registry.rs            # route_aliases HashMap
  ui/style/iced_adapter.rs         # DARK_MODE 默认 true / resolve_semantic_rgb pub
  ui/view.rs                       # View::Button 新增 content 字段（§6）
  ui/node_converter.rs             # Button content: None（§6 适配）
  ui/vnode_converter.rs            # Button content: None（§6 适配）
  ui/gpui/auto_render.rs           # Button 模式补 on_right_click/content（修复既有编译错误）
  ui/gpui/renderer.rs              # 同上
  ui_gen/vue.rs                    # §4 全部 vue codegen 修复
  ui_gen/rust.rs                   # outlet→empty / link→text_styled
  token.rs                         # 移除 grid 保留字
  lib.rs                           # 注册 route alias + plan409_tests 模块声明
  plan409_tests.rs                 # §6/§8 回归测试（新增）
  aura/types.rs                    # §8: aura_events_get_base 大小写不敏感（onClick）
stdlib/aura/widgets/display/Icon.at  # 新增 Icon widget
examples/widgets-gallery/          # app.at + 49 pages + pac.at
.cargo/config.toml                 # 32MB 栈
```

---

## 8. ✅ 主题色支持（Widgets Gallery）

> 给 gallery 加主题色能力：主操作/显眼内容（primary Button / H1-H3 / link / input 边框）
> 用主题色；顶栏最右加主题色选择器；Home 页 "Auto UI" 大字用主题色。
> 主题色 5 色（indigo/coral/ocean/sage/amber）取自 **auto-forge** 的
> `useAccentColor.ts` / `theme.css`（`--primary` HSL 约定），两产品共享视觉语言。

### 8.1 现状与机制（探索结论）

- **VM/iced 侧主题色机制已存在**：`renderer.rs:4695` 每帧读 `accent_color` state →
  `set_accent_name` → `Color::Primary` 走 5 色 HSL；`text-primary`/`bg-primary`/
  `border-primary`/`bg-primary/10`/`from-primary`/`to-primary` 类已解析并 accent-aware。
- **Vue 侧**：`applyAccent` 注入只在 **store composable 路径**（Plan 360，015-notes 用
  store）；gallery 的 `App` 是 **widget**（无 store）→ 需扩展 widget script 路径。
- **缺口**：`variant:"primary"` 预设硬编码 `bg-blue-500`；h1/h2/h3/link/input 默认无
  主题色；`palette` icon 不在 `lucide_svg` 表；顶栏无选择器 UI；Home 大字用渐变
  （`via-*`/`bg-clip-text text-transparent` 在 iced 下不可见）。
- **VM 限制**：iced 不支持 absolute 定位 → 选择框用条件渲染块（顶栏下方内联展开）。

### 8.2 改动

| 子项 | 说明 |
|---|---|
| 8.1 primary preset | `convert_button` primary 变体预设 `bg-blue-500 hover:bg-blue-600 text-white` → `bg-primary text-primary-foreground font-medium rounded`（主题色） |
| 8.2 h1-h3 默认色 | VM `aura_view_builder` 与 vue `vue.rs` 的 h1/h2/h3 默认类加 `text-primary`（页面标题主题色） |
| 8.3 link 主题色 | VM `render_link_button_with_icon` 的 Button `style: Some(text-primary)`；vue router-link 默认类加 `text-primary`。VM renderer 新增 `inherit_text_color`：把按钮 text_color 继承给 content 中无显式颜色的 Text（`link (to:) { text "Docs" }` 子文字主题色） |
| 8.4 input 边框 | VM/vue 的 input/textarea 默认类 `border` → `border-primary`（输入框边框主题色） |
| 8.5 palette icon | `lucide_svg` 表加 `"palette"`（顶栏选择器图标） |
| 8.6 vue widget accent | `vue.rs` widget script 路径检测 `accent_color` state → 注入 `ACCENT_PALETTE_JS`（复用 store 路径常量）+ `onMounted` 恢复 localStorage + `SetAccent`/`ToggleDarkMode` handler 追加 `applyAccent(...)` |
| 8.7 vue index.css | `auto-man generate_index_css` 的 `:root`/`.dark` `--primary` 改 indigo（239 84% 67%/77%，与 VM `accent_hsl` 默认、auto-forge 一致），`--ring` 同步 |
| 8.8 gallery app.at | `model` 加 `themeOpen bool` + `accent_color str = "indigo"`；`on` 加 `.openThemePicker`/`.SetAccent`；顶栏右加 palette 按钮；`if .themeOpen` 条件块渲染色板（5 色块，`bg-{color}-500` + active `ring-primary`，仿 015-notes sidebar） |
| 8.9 Home 大字 | `pages/index.at` 的 "Auto UI" h1 去渐变（`via-*`/`bg-clip-text text-transparent` iced 不可见）→ `text-primary` |
| 8.10 事件大小写 | `aura/types.rs` `aura_events_get_base` 改大小写不敏感——gallery 用 `onClick:`（Vue 约定），VM 查询 `onclick`，此前所有 button onClick 在 VM 下失效（降级为 `click`） |

### 8.3 验证

- `cargo test -p auto-lang --features ui-iced --lib plan409`：4 项通过（新增
  `theme_accent_color_state_and_handlers`、`theme_palette_ui_and_primary_rendering`）。
- vue.rs 单测 `test_widget_accent_color_injects_apply_accent`：widget 含 `accent_color`
  state 时 script 注入 `ACCENT_PALETTES`/`applyAccent`/`getSavedAccent` bootstrap。
- 回归：`cargo test -p auto-lang` 无新增失败（既有 21 项失败与本次无关）；vue 194 项
  全绿；aura_view_builder 28 / iced renderer 15 全绿。
- gallery 生成产物（`auto build --gen-only --render vue`）核对：App.vue 含 palette 按钮、
  `applyAccent` 注入、5 色板条件渲染、router-link `text-primary`；index.css `--primary`
  indigo；IndexPage.vue hero `text-primary`。（gen/ 被 .gitignore 排除，不入库。）

### 8.4 已知遗留

| 项 | 状态 | 说明 |
|---|---|---|
| 色板浮层定位 | ⚪ 接受 | VM 无 absolute → 色板顶栏下方内联展开；vue absolute 浮动弹出 |
| `ring-*` 类 | ⚪ 部分 | VM 不支持 ring（active 高亮在 vue 生效、VM 忽略）；gallery 用 `ring-primary` 仅在 vue 完整 |
| 渐变文字 | ⚪ 移除 | `bg-gradient text-transparent` iced 不可渲染，Home 大字改纯 `text-primary` |

---

## 9. ✅ 本会话修复批次（vue 模式审查 + codegen bug + 交互演示）

> 用户在 vue 模式下逐页审查 gallery，暴露并修复一批 codegen bug、内容缺口、
> 交互演示缺失。改动分布在 `ui_gen/{vue.rs, validators.rs, widget/registry.rs}`
> 与 gallery 的 `.at` / README。提交：`1e59c791`（plan-410 批次）+ `1f3a0440`
> （tooltip 白屏 + toast 动态）。

### 9.1 Codegen bug 修复

| 子项 | 现象 | 修复 |
|---|---|---|
| 9.1.1 text 字面量重复 | `text (style:"…") { "字面量" }` 渲染两遍（首页 Hero 介绍文案重复） | shadcn path 把子 Text 节点 hoist 到 `slot_content` 时记录被消费的子节点索引（`consumed_text_child_idx`），输出时跳过该子节点 |
| 9.1.2 App.vue 非法 export | `<script setup>` 内主题色辅助函数带 `export function` → vite `Pre-transform error` → 白屏 | codegen 模板去掉 `applyAccent`/`getSavedAccent`/`getAccentNames` 的 `export`（`<script setup>` 不允许 ES exports） |
| 9.1.3 R004 lint 误报 | 170 个「handler 缺失」警告（实际 button 用内联赋值表达式 `@click="X=!X"`） | R004 正则只匹配纯引用/函数调用（标识符后紧跟 `(` 或 `"`），排除赋值表达式；+ 2 防回归测试（`r004_ignores_inline_*_assignment`） |
| 9.1.4 button force_native | 带 `style` 的 button 被降级为原生 `<button>`（丢 shadcn `bg-primary` 主题色，Get Started 按钮） | `force_native_elements` 移除 `"button"`（shadcn Button 支持额外 class 叠加，只有 input/checkbox/textarea 保留原生） |
| 9.1.5 row/col gap 丢失 | `row (gap:"2")` 生成为 `<div class="flex flex-row">`（无 `gap-2`） | shadcn path 的 row/col 加 `gap` prop → `gap-N` class 转换（之前只有 native path 处理 gap） |
| 9.1.6 toast() import 检测 | handler 调 `toast()`/`toast.success()` 但不生成 `import { toast } from 'vue-sonner'` | 新增 `stmts_call_toast`（照 `stmts_call_complete`）扫描 AST；直接检查 `call.name` 的 Expr 结构（Ident 或 `Dot(Ident "toast", _)`）——`get_name_text_safe` 对方法调用返回 None，必须直查 Expr |

### 9.2 Gallery 内容与一致性

| 子项 | 内容 |
|---|---|
| 9.2.1 sidebar 补全 | hovercard/radiogroup/toggle/togglegroup 4 个组件补进 sidebar 导航 + 首页卡片（之前有路由但无导航链接） |
| 9.2.2 首页数量一致 | 标题 46→49；首页补 grid/navlink 卡片；Feedback count 7→5（之前声明数与实际卡片数不符） |
| 9.2.3 Components→Widgets | 所有 UI 文本「Components」改「Widgets」（header/mobile 导航、Hero 标题/按钮、搜索 placeholder、章节标题、表头、面包屑、描述文案，12 处） |
| 9.2.4 搜索框居中 | Home 搜索框 input 加 `w-full`（之前容器 `mx-auto` 居中但 input 未占满，视觉偏左） |
| 9.2.5 slider value 类型 | 核查确认 codegen 已正确转 `:default-value="[50]"`（number[]），无需改 |
| 9.2.6 README | 「已知边界」更新：carousel 移出占位列表（已能渲染 slide 内容），剩 command/combobox/toggle-group 3 族 |

### 9.3 组件交互演示

| 子项 | 现象 | 修复 |
|---|---|---|
| 9.3.1 button onclick toast | gallery 此前无任何 onclick 事件示例 | `button.at` 加 Events 示例（`onClick: .showClickToast` + `toast-provider`），点击弹 toast；依赖 9.1.6 自动 import |
| 9.3.2 tooltip 白屏 | `/tooltip` 整页空白（header/sidebar 在但 main 空） | 三层 codegen 修复：(a) `registry.rs` 加 `TooltipProvider` WidgetSpec（generate_shadcn_imports 才能生成 import）；(b) `map_tag` 识别 `tooltip-provider` tag → `<TooltipProvider>`；(c) `tooltip-trigger` 自动加 `as-child`（reka-ui TooltipTrigger 渲染自身 `<button>`，内嵌 `<Button>` 会嵌套 button 白屏）。`.at` 加 `tooltip-provider` 包裹 |
| 9.3.3 toast 动态演示 | toast 页只有静态卡片预览，无动态触发 | `toast.at` 加 Live Demo（Success/Error/Info 三按钮 `onClick` 触发 `toast.success/error/info` + `toast-provider`），依赖 9.1.6 的方法调用检测 |

### 9.4 样式

| 子项 | 内容 |
|---|---|
| 9.4.1 代码框滚动条 | codeblock 的 `<pre>` 横向滚动条改半透明细条（zinc-200/20 thumb、透明 track、无 tracker），与 ScrollArea 视觉一致。在 `generate_style` 给有 codeblock 的页面注入 `pre::-webkit-scrollbar` + Firefox `scrollbar-color` |
| 9.4.2 Tabs 下划线 | shadcn `<Tabs>` 默认「分段控件」风（灰胶囊 + 白按钮）改「下划线」风（TabsList 底线、TabsTrigger active 主题色 `border-b-2 border-primary`），与 CodeBlock 的 Auto/Vue tab 一致。改 shadcn 组件 `TabsList.vue`/`TabsTrigger.vue`（gen 产物，本地修改） |

### 9.5 验证

- **dev server 逐页核查**（vue 模式）：首页 Hero 文案不重复、Get Started 主题色、搜索框居中、标题「49 Widgets」、sidebar 50 项、`/tooltip` 不白屏（Hover me 按钮）、`/toast` Live Demo（3 按钮）、`/button` Events onclick toast、各页「Components」→「Widgets」、row gap 生效。
- **测试**：`cargo test -p auto-lang r004` 5 项全绿（含 2 防回归）；`cargo build --bin auto` 无错误。
- **gen 产物核对**：App.vue 无 `export function`、tooltip.vue 含 `TooltipProvider` import + `<TooltipTrigger as-child>`、toast.vue 含 `import { toast } from 'vue-sonner'`、index.vue Hero 文案单行、button.vue 含 `showClickToast` handler。

### 9.6 暂缓

| 项 | 状态 | 说明 |
|---|---|---|
| CodeBlock/PreviewCard 改纯 Auto widget | ⚪ 暂缓 | 当前是「Auto 声明壳 + codegen 硬编码 UI」混合模式（`.at` 的 view 是占位 `div`，真实 UI 在 `generate_codeblock_html`/`generate_previewcard_html`）。改成纯 Auto 需把 Prism 高亮 / clipboard / setTimeout / Auto-Vue tab 切换等命令式逻辑搬进 `.at` 的 model/on/computed/view，codegen transpile 这些浏览器/API 调用的能力需先验证。作为后续独立任务。 |

### 9.7 提交与文件

- `1e59c791` feat(plan-410): toast() import 检测 + widget gallery 更新 — §9.1.3/9.1.5/9.1.6 + §9.2 全部 + §9.3.1 + §9.4.1（vue.rs 110 行 + validators.rs + 8 个 .at + README）
- `1f3a0440` fix(widget-gallery): tooltip 白屏 + toast 动态演示 — §9.3.2/9.3.3 + §9.1.6 方法调用补丁（vue.rs 后续 + registry.rs TooltipProvider spec + toast.at/tooltip.at）
- §9.1.1/9.1.2/9.1.4 含在 `1e59c791` 的 vue.rs 批次内
- §9.4.2 TabsList/TabsTrigger 是 shadcn add 生成（gen/，gitignore），本地修改未入库

---

## 10. ⏳ 本批次待修：VM 视觉差距 6 项（2026-08-12 审查）

> §1–§9 完成后，对 VM 模式做新一轮逐页审查，发现 6 个残留视觉/交互差距。
> 本节登记并修复，工作在 worktree `plan-409`（分支 `plan-409`）。
> 6 项按根因合并为 4 组（原问题 1+2 渲染部分合一、原问题 4+5 路由合一）。

### 10.1 环境与验证前置

- **Rust toolchain**：见 `rust-toolchain.toml`。
- **冷启动验证序列**（复制即跑）：
  1. `examples/widgets-gallery/pac.at` 设 `render: "vm"`
  2. 改 codegen 前先 `taskkill //F //IM auto.exe` 释放占用（auto.exe 被占会导致 build 失败）
  3. `cargo build --bin auto`（首次较慢）
  4. `D:/autostack/auto-lang/.worktree/plan-409/target/debug/auto.exe run`
  5. VM 启动后 AutoUI MCP 监听 `http://127.0.0.1:9247/mcp`（Streamable HTTP, JSON-RPC 2.0）
  6. 拿 snapshot：
     ```bash
     curl -X POST http://127.0.0.1:9247/mcp -H "Content-Type: application/json" \
       -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"autoui_snapshot","arguments":{}},"id":2}'
     ```
  7. 工具：`autoui_snapshot`（UI 树）/ `autoui_action`（点击，验证导航/state）/ `autoui_inspect`（节点详情）/ `autoui_screenshot`（截图）
- **vue 对照基准**：`pac.at` 设 `render: "vue"` → `auto run` → 浏览器看"正确答案"。

### 10.2 待修问题（合并为 4 组 + 1 独立）

#### 组 A：icon-only button 渲染 "Button" 文本（原问题 1+2 渲染部分）

- **现象**：header 汉堡菜单（`button style:"md:hidden -ml-2" icon:"menu"`）、search/theme 按钮（`button variant:"ghost" style:"h-9 w-9" icon:"search"/"palette"`）——有 icon 无 text 的 button，VM 渲染默认 "Button" 文本。
- **根因**：`convert_button`（`aura_view_builder.rs`）对 icon-only button（有 icon 无 text）未取 icon 作内容，回退默认 "Button" label。
- **修复方向**：`convert_button` 对"有 icon 无 text"的 button，渲染 icon（`View::Image { src: "lucide:xxx" }`）而非 "Button" 文本。
- **验证**：`autoui_snapshot` → header 不再出现 "Button" 字样；汉堡/search/theme 三处显示对应 icon。

#### 组 B：VM onClick handler 执行（原问题 2 交互部分）

- **现象**：search/theme 按钮 `onClick:.openSearch/.openThemePicker`（切 `searchOpen`/`themeOpen` state），VM 点击无效。
- **修复方向**：VM onClick handler 执行机制（参考 §8.10 的 `aura_events_get_base` 大小写修复经验，确认 handler 编译/分发路径）。
- **验证**：`autoui_action` 点 theme 按钮 → snapshot 出现色板块（state `themeOpen` 翻转）。

#### 组 C：header logo icon + Docs/Widgets 主题色（原问题 3）

- **现象**：header 的 logo icon（`row { icon name:"layers" }`）+ Docs/Widgets link 应 `text-primary`（主题色），当前是普通色——被 §8.3 的 nav-link 普通色修复带歪（`render_link_button` 直接调 `render_link_button_with_icon`，header link 也走了 `style:None`）。
- **修复方向**：`render_link_button_with_icon` 加 `themed: bool` 参数——`render_link_button`（header link）传 `themed=true`（`text-primary`），nav-link/component-card 传 `themed=false`（普通色）。
- **⚠️ 易漏点**：header logo icon（`row` 里的 `icon name:"layers"`）也要加 `text-primary`，别只顾 link。
- **验证**：`autoui_snapshot` + `autoui_screenshot` → logo icon 与 Docs/Widgets 文字呈主题色（indigo）。

#### 组 D：路由 `/` 匹配（原问题 4+5）

- **现象**：（a）VM 启动 main 区不显示 Home（IndexPage），要点 sidebar 才出现；（b）点 Home（nav-link `to:"/"`）不切换首页，但点其它组件（`/button` 等）可切换。
- **修复方向**：`render_outlet`（`aura_view_builder.rs`，搜 "render the page widget matching"）——初始路由设 `/` + `/` 根路由匹配 IndexPage。
- **验证**：VM 启动 → snapshot main 区有 Home 内容；`autoui_action` 点 Home → main 切回 Home。

#### 组 E：CodePreview block VM 识别（原问题 6）

- **⚠️ 注意**：此问题与 §9.6 的"CodeBlock/PreviewCard 改纯 Auto widget"不同——§9.6 是 **vue 侧**混合模式改造（暂缓），本项是 **VM 侧** view_builder 不识别 `preview-card`/`codeblock` tag → `View::Empty` 被过滤。
- **现象**：点 Button 等组件页，VM 只渲染标题+描述+表格，CodePreview block（`preview-card`/`codeblock`，含示例预览+代码+Auto/Vue tab+copy）缺失。
- **根因**：`preview-card`/`codeblock` 是 vue codegen 特殊处理（`vue.rs` `generate_previewcard_html` ~4254 / `generate_codeblock_html` ~4414），VM 不识别。
- **修复方向**：VM view_builder 加 `preview-card`/`codeblock` 识别（参考 category-section 的修复模式——untracked `_` fallback + tracked `_` fallback 双保险）：
  - `preview-card` → 容器（col，含 preview 区 + code toggle footer），递归 children
  - `codeblock` → 简化（`View::Text`（code 内容），或 `<pre>` 样式）
- **验证**：`autoui_snapshot` 点 Button 页 → 能找到 preview-card/codeblock 子树（至少代码文本可见）。

### 10.3 关键经验（沿用）

- **VM 主渲染走 untracked 路径**（`convert_node_with`→`convert_element` untracked 的 `_` fallback），不是 tracked。调试时 eprintln 加到 untracked `_` fallback 才命中（组 E 同样适用）。
- **toast 降级**：`handler_codegen.rs` `rewrite_expr`（搜 "vue-only escape hatch"）把 vue-only 调用降级为 `Expr::Bool(false)`，避免 VM link 失败。

### 10.4 文件（预计触及）

```
crates/auto-lang/src/ui/aura_view_builder.rs  # 组 A/C/D/E 主战场
crates/auto-lang/src/ui/handler_codegen.rs    # 组 B（handler 执行，若需）
examples/widgets-gallery/src/front/app.at     # 验证场景（对照）
```
