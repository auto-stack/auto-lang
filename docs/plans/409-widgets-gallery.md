# Plan 409: Widgets Gallery — 三模式一致性 + link 子组件 VM 缺口

> **状态**: §1–§5 ✅ **已完成并合并 master**；§6 🟡 **待修复**（`link` 子组件在 VM 模式下不渲染，本计划的核心前瞻项）。
> **仓库**: **auto-lang**（`crates/auto-lang/src/ui/{iced/renderer.rs, aura_view_builder.rs, widget_registry.rs, style/iced_adapter.rs}` + `ui_gen/{vue.rs, rust.rs}` + `token.rs` + `lib.rs`）；gallery 产物在 `examples/widgets-gallery/`。
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

## 6. 🟡 待修复：`link` 子组件 VM 渲染缺口（本计划核心）

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

### 6.4 修复方案

**方案 A（推荐）：`link` 子节点 → 可点击容器**

1. 在 `aura_view_builder.rs` 的 `link` 转换中，若有子节点：
   - 递归 `convert_children` 得到子 `View` 列表；
   - 包成一个 `View::Row`（或按子节点推断 row/column）；
   - 渲染为带 onClick 导航的透明 Button，其 content = 该容器（而非 `to` 字符串）。
2. 无子节点时保持现状（用 `to` 或 `label` 作文本）。
3. `View::Button` 需支持 `content: Element` 而非仅 `String`——若当前 Button 变体只支持 String，则新增一个"chromeless clickable container"路径（复用 §3.4 的 chromeless 样式）。

**方案 B（降级）：`link` 子节点只取首个文本节点**

- 若子节点是单个 `text`，提取其内容作 Button label；其余子节点忽略。
- 实现成本低，但 `row { icon + text }` 这类仍会丢图标——不推荐（与 §2 的图标能力背道而驰）。

### 6.5 验证

1. **vm 模式**：顶部导航栏 Docs/Components 正确渲染文字并可点击导航（`autoui_snapshot` 应见 `button "Docs"` / `button "Components"`，而非 `"/"` `"/button"`）。
2. **回归**：gallery 全部 50 路由 + 侧边栏 nav-link 不受影响（`cargo test -p auto-lang` 全绿；vm 启动 + MCP 截图对照 vue）。
3. **vue 不变**：vue codegen 路径不动，确认无回归。

---

## 7. 验收与已知遗留

### 7.1 已完成验收（§1–§5）

- vm 模式 gallery 可启动、路由可切换、侧边栏可点击导航（MCP `autoui_action` 点击验证）。
- 深色模式：侧边栏文字可见——截图像素采样，左侧栏浅色像素占比 ~13.4%（修复前 ~0%），背景 `(43,45,49)`。
- Home 项视图树：`button "homeHome"`（PUA 图标 + 标签），与其余 nav-link 一致。
- vue 模式：49 组件页标题统一 H2、h1/h2 有下方留白、Slider/Drawer/Toast/Pagination/NavLink 均正常。

### 7.2 已知遗留

| 项 | 状态 | 说明 |
|---|---|---|
| `link` 子组件 VM 缺口 | 🟡 §6 待修复 | 顶部 Docs/Components 仍显示路由字符串 |
| 顶部标题栏视觉差距 | ⚪ 接受 | vue/vm 渲染器本质差异（CSS vs GPU），视觉差距属预期，非缺陷 |
| `render: "rust"` 模式 | ⚪ 部分 | rust codegen 覆盖基本元素，复杂组件仍有缺口（独立于本计划） |

### 7.3 文件清单（本计划触及）

```
crates/auto-lang/src/
  ui/iced/renderer.rs              # lucide: 渲染 / 深色 Text+Button 默认色 / 无样式 Button chromeless
  ui/aura_view_builder.rs          # nav-link fallback / icon→lucide / outlet route alias
  ui/widget_registry.rs            # route_aliases HashMap
  ui/style/iced_adapter.rs         # DARK_MODE 默认 true / resolve_semantic_rgb pub
  ui_gen/vue.rs                    # §4 全部 vue codegen 修复
  ui_gen/rust.rs                   # outlet→empty / link→text_styled
  token.rs                         # 移除 grid 保留字
  lib.rs                           # 注册 route alias
stdlib/aura/widgets/display/Icon.at  # 新增 Icon widget
examples/widgets-gallery/          # app.at + 49 pages + pac.at
.cargo/config.toml                 # 32MB 栈
```
