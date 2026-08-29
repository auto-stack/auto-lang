---
plan_id: PLAN-482
status: archived                # drafting → executing → execution_done → reviewed → archived
feature_name: autoui-nav-item
author: [zcode]
created_at: 2026-08-29
updated_at: 2026-08-29

# /auto-plan:review 结束时填写：
supersedes_spec_components: [auto-lang/ui: nav-link 组件(schema 标 deprecated,实现保留兼容), auto-lang/spec-schema: nav 元素(纯容器→search 集成容器)]
new_spec_components: [auto-lang/ui: nav-item/nav-group 契约组件(ui_gen/nav_contract.rs 单源+NavItem/NavGroup 脚手架), auto-lang/ui: VM 路由历史栈(__route_history/navigate_back/__nav_toggle/__nav_group_states), auto-lang/ui: lucide_svg 文档包装(lucide_svg_doc,修复全 VM lucide 图标空渲染), auto-man: nav/separator 依赖检测+autodown engine 链接推导]
touched_goals: [GOAL-007, GOAL-010]

affects: [auto-lang/ui, auto-lang/spec-schema, auto-man]   # 受影响的 specs 路径
current_step: 24
total_steps: 24
---

# [PLAN-482] AutoUI nav-item 导航组件族：双端一致的通用 Nav 组件 + 三应用替换

## 变更摘要

AutoUI 目前没有真正的导航项组件：`nav` 是纯容器（Vue→`<nav>`、VM→Column），`nav-link`
仅在 widgets-gallery 使用（Vue→router-link、VM→链接样式 Button），三个真实应用
（examples/ui/015-notes、auto-musk、auto-os-config）的左侧导航全部用 `col/aside` +
`button` 手搓，由此产生大量已知痛点：shadcn Button 基类（justify-center/h-9）强迫每项
重声明左对齐；选中态样式以巨型 if 字符串或 store 预计算 class 串的形式五处复制
（os-config modules_store 5×~45 行投影块）；VM 侧按钮→文字颜色继承断裂需要显式
逐 text 补色；hover 展示（如删除按钮浮现）逃逸出 utility class 体系。

本计划参考 shadcn-vue Sidebar（SidebarGroup/SidebarMenuButton isActive + as-child 多态）
与 Material 3 NavigationDrawer/NavigationRail（selected 是 item 的一等属性、indicator
药丸、badge 槽位）的共性，设计 **nav-item / nav-group 两个新组件 + nav 容器的 search
集成**，双端（Vue codegen + VM/iced view builder）共享同一份 class token 契约
（`nav_contract.rs` 常量 ↔ `NavItem.vue`），目标是像素级一致：

- `nav-item`：不是 Button 的变体，而是自带布局类的独立导航项。`to:`（路由模式，
  Vue→RouterLink 改 hash URL、VM→`__navigate`）或 `onclick:`（状态模式）二选一；
  hover 高亮 + cursor、选中态主色调（`bg-primary/10 text-primary`）内建；icon 左槽
  （lucide svg / emoji 双通道）、label/desc 双行、badge 右槽、size sm/md/lg。
- `nav-group`：分组标签 + 可折叠（chevron 头），`open` 可绑定可内置；支持 indent
  树状层级。
- `nav (search: true)`：搜索框以属性集成进容器（search_value/onsearch 绑定），
  不再单独声明。
- VM 路由补历史栈：`__navigate` push `__route_history`，新增 `router.back()`
  （Vue 侧 ts_adapter 同步支持），VM 无 URL 也有可返回的"页面地址"语义。
- 替换三个应用左侧导航：015-notes（仓内）、auto-musk（外部仓，rail）、
  auto-os-config（外部仓，分组树 + 搜索集成 + 投影块瘦身）。

## 目标

1. **G1 组件**：`nav-item` / `nav-group` 在 schema/aura.at 声明、两端 full 支持，
   `nav` 容器支持 search 属性集成。
2. **G2 UX 三态**：hover（独立高亮 + cursor）/ active（主色调选中）/ disabled 双端
   行为一致；active 由 `to` 自动判定（精确或前缀段匹配，`exact:` 可收紧）或由
   `active:` 显式给定。
3. **G3 路由**：`to:` 模式下 Vue 改 hash URL + 浏览器返回可用；VM 改
   `__current_route` 且 `router.back()` 可沿历史栈返回。
4. **G4 复杂 UI**：nav-item 至少支持左侧 icon（lucide svg + emoji 兜底）、
   label+desc 双行、右侧 badge；nav-group 支持可折叠分组与缩进树。
5. **G5 三应用替换**：015-notes / auto-musk / auto-os-config 左侧导航全部换用新组件，
   各自门禁通过；os-config 五处复制的 nav_class 投影块收敛为 `active` 表达式。
6. **G6 契约锁定**：class token 契约双端单一来源 + 单元测试锁（VM `Style::parse`
   全可解析 + 与 NavItem.vue 常量一致性校验）。

## 架构方案

遵循既有分层（spec ledger `auto-lang/ui`）：schema/aura.at 是唯一声明源（Plan 435）
→ `ui_gen/widget/registry.rs` codegen 规格 → Vue 侧 `ui_gen/vue.rs` SFC codegen +
auto-man shadcn 资产脚手架；VM 侧 `ui/aura_view_builder.rs`（Aura IR→View 树）→
`ui/iced/renderer.rs`（View→iced）。本计划在每一层的 nav 通道上扩展，不新增层：

```
.at 源码                nav-item / nav-group / nav(search:)
   │
   ├─ Vue:  vue.rs map_tag→NavItem/NavGroup 脚手架组件(auto-man assets/shadcn-ui/nav/)
   │        NavItem.vue 内部 <component :is="to ? RouterLink : 'button'"> 多态
   │        契约 class 与 nav_contract.rs 镜像(测试锁定)
   │
   └─ VM:   aura_view_builder.rs → View::Button{content: Row[icon, texts(Fill), badge],
            onclick: __navigate|用户msg, style: 契约类+hover_classes}
            iced renderer 现有 Button content/hover 机制(2930-2934 行)直接复用
            nav-group → 头部行 + open 条件子列(绑定或 __nav_group_open:<key> 内置态)
            dynamic.rs __navigate 加历史栈 push;新增 __navigate_back / __nav_toggle 拦截
```

关键决策：
- **不新造 View 变体**：VM 侧 nav-item 复用 `View::Button`（带 content 子树 +
  hover_classes），iced 已支持内容子树、hover 态样式、padding/width/height 解析——
  避免 renderer 新臂；选中/悬停样式由构建期 class 串决定，与 Vue 同源。
- **onclick 状态模式为一等公民**：三个 app 现有切换逻辑均为 store 状态，不强推
  routes（015-notes 是数据过滤非页面导航；musk 有 pushState 语义桥；os-config 有
  hash 深链）。`to:` 路由模式由 018-book-reader 作为规范示例承载。
- **契约交集原则**：共享 class 串只取双端都可解析的 token（整步间距或 `[]` 任意值、
  无 uppercase/tracking 等纯 web 特效），保证像素输出由同一串类决定。

## 技术栈

Rust（auto-lang/ui、ui_gen、auto-man）+ Vue3 SFC 脚手架资产 + iced 0.13 renderer +
既有测试别名（`cargo t` / `cargo tf` / `cargo test --test docs_gen`）+ autoui-verifier
技能脚本（`test_vue_playwright.mjs` / `test_vm_mcp.py`）。外部仓沿用各自门禁
（musk：vite build + web e2e；os-config：`scripts/e2e.sh` + `e2e-vm.mjs` +
`capture.mjs` parity 台账）。

## 需求分析与背景调查

（取材 docs/specs/overview.md、GOAL-007/010、schema/aura.at、三 app 源码与既有 plan 记录）

**框架现状**：
- `nav`（schema 518-527 行）：web native / iced unknown，无 props；vue.rs:6111 映射
  `<nav>`，默认类 `flex items-center gap-4`（横向，适合顶栏）；VM 侧
  render_support.rs:154 与 aside/header 同为容器 full。
- `nav-link`（529-542 行）：Vue→router-link（vue.rs:6020-6023，icon+label 内联
  div），VM→`render_link_button_with_icon`（aura_view_builder.rs:2890-2963）=
  `View::Button` + `__navigate` 消息 + `__current_route` 精确命中高亮（Plan 411
  P1-A）。**仅 widgets-gallery 使用**。
- `nav_item`（1554-1562 行）：schema 已注册但 `backends: none`——本计划将其转正。
- 路由：DSL `routes {}` + `outlet`；Vue 生成 hash router（createWebHashHistory）；
  VM 用 `__current_route` 状态 + `__navigate` 拦截（dynamic.rs:590-607）+
  `render_outlet`（2965-3032 行）段匹配渲染页面；**VM 无历史栈、无 back**。
- VM hover：iced renderer.rs:2900-2934 已实现 `hover:` 类合并的 Hovered/Pressed
  原生反馈（os-config README"hover 类被丢弃"的说法已过时，仅指非 Button 元素）。
- 图标：Vue lucide-vue-next 组件（codegen 收集 import）；VM `lucide_svg` 内嵌
  路径集（~60 个）+ emoji 兜底；button PUA 标记 `\u{EE01}icon\u{EE02}` 通道。
- `auto` CLI 指向本仓 target/debug 构建——外部仓直接吃到编译器改动，无需发布。

**三 app 现状与痛点**（详见各自源码注释/plan 记录）：

| 维度 | 015-notes | auto-musk | auto-os-config |
|---|---|---|---|
| 导航单元 | col + button（view fn NoteItem） | col rail + NavSidebar div + NavListItem | aside + nav + button（store 预计算 nav_class） |
| 结构 | 文件夹伪树（indent prop） | 平铺 rail×4 + 二级列表 + wiki/specs 树 | 概览置顶 + 独立模块 + 可折叠分组（▾/▸） |
| 选中/悬停 | if 字面量样式串 | if 串 + 显式 text 补色（VM 继承断裂）+ web-only :hover CSS | 5×~45 行 nav_class 投影复制 |
| 切换 | store active_id/folder/tag | current_view + pushState/popstate 桥（querySelector 脆弱选择器） | store active_id/kind + #hash 深链 |
| 搜索 | 手搓 row🔍+input | wiki 树过滤/chats 头部搜索 | sidebar 内 input，过滤沉入 store |
| 图标 | emoji 📁 | lucide 组件（use.web 引入） | emoji（模块注册表） |

**参照设计**：shadcn-vue Sidebar（SidebarGroup(label)+SidebarMenuButton(isActive,
tooltip, size sm/default/lg)——本设计取 isActive/size/lg 双行思想，弃 Provider/
Sheet 移动端等重型机制）；M3 NavigationDrawerItem（selected 一等属性 + 药丸
indicator + badge 槽——取 selected/badge 思想，药丸改为 bg-primary/10 全宽块，
与 musk 现行选中样式一致）。两者共性即本契约：**选中是 item 的属性而非按钮
变体；布局（icon|文本|badge）内建；hover 是容器级反馈**。

## 详细设计

### D1. DSL API（完整）

```
nav (style: "...",                          # 容器类，侧栏典型 "flex flex-col gap-1 p-2"
     search: true|false,                    # 集成搜索行（默认 false）
     search_placeholder: "Search...",
     search_value: .store.search,           # 绑定表达式（search:true 时必填）
     onsearch: .SearchChanged) {            # 消息（携带输入串，同 input oninput）

    nav-group (label: " GENERAL ",          # 分组标签
               collapsible: true,           # 可折叠（默认 false；true 时头部可点）
               open: .store.groupOpen,      # 开合态绑定（缺省 true；可绑可省）
               ontoggle: .ToggleGroup,      # 折叠消息（缺省用内置态）
               indent: true) {              # 子项缩进 pl-3（默认 false）

        nav-item (to: "/chats",             # 路由模式：Vue RouterLink / VM __navigate
                  onclick: .Select(m.id),   # 状态模式（与 to 互斥，同给时 to 优先+警告）
                  active: .cur == "chats",  # 选中覆写（to 模式缺省自动判定；onclick 模式必填才亮）
                  exact: false,             # to 自动判定用精确匹配（默认前缀段匹配）
                  icon: "message-square",   # 左图标：lucide 名或 emoji 字面量
                  label: "会话",            # 主文本
                  desc: "最近对话",          # 次行文本（有则双行布局）
                  badge: "3",               # 右侧徽标
                  disabled: false,
                  size: "sm"|"md"|"lg")     # md=h-9 单行(默认)；lg=py-[10px] 双行；sm=h-7
    }
}
```

nav-group 直接子节点外的用法（无分组的平铺 nav-item）同样合法。

### D2. class token 契约（双端单一来源）

新模块 `crates/auto-lang/src/ui/nav_contract.rs` 导出常量；`NavItem.vue`/`NavGroup.vue`
内嵌同一串（单测读资产文件比对锁死）。全部 token 须通过 VM `Style::parse`
（整步间距或 `[]` 任意值；select-none/cursor-pointer/transition-colors 为 web 增强
项，VM 忽略且无视觉差；其余如 uppercase/tracking 不入契约）：

```text
ITEM_BASE_MD   = "nav-item flex w-full items-center gap-2 rounded-md px-3 h-9 text-sm text-left text-foreground select-none cursor-pointer transition-colors"
ITEM_BASE_LG   = "nav-item flex w-full items-start gap-3 rounded-md px-3 py-[10px] text-sm text-left text-foreground select-none cursor-pointer transition-colors"
ITEM_BASE_SM   = "nav-item flex w-full items-center gap-2 rounded-md px-2 h-7 text-xs text-left text-foreground select-none cursor-pointer transition-colors"
ITEM_HOVER     = "hover:bg-accent hover:text-accent-foreground"      # 仅未选中时挂
ITEM_ACTIVE    = "bg-primary/10 text-primary font-medium"            # 选中态（挂载后 hover 不再挂）
ITEM_DISABLED  = "opacity-60 cursor-default"                          # web；VM 走 disabled 灰文路径
BADGE_PILL     = "ml-auto inline-flex items-center justify-center rounded-full bg-primary/15 text-primary px-2 py-[2px] text-xs font-medium shrink-0"
ICON_MD = "h-4 w-4 shrink-0" / ICON_LG = "h-5 w-5 shrink-0"
TEXTS_FILL     = "flex-1 min-w-0"                                    # 有 badge/desc 时挂，双端文本列 Fill
GROUP_LABEL    = "nav-group-label px-3 pt-2 pb-1 text-xs font-medium text-muted-foreground"
GROUP_TOGGLE   = "nav-group-toggle flex w-full items-center gap-2 px-3 py-2 rounded-md text-sm font-medium text-foreground cursor-pointer select-none"
GROUP_TOGGLE_HOVER = "hover:bg-accent"
GROUP_CONTENT  = "nav-group-content flex flex-col gap-1"             # indent 时追加 " pl-3"
SEARCH_ROW     = "nav-search flex items-center gap-2 mx-3 mb-2 px-3 h-9 rounded-md border border-input bg-muted/50 text-sm shrink-0"
SEARCH_INPUT   = "w-full bg-transparent border-0 outline-none placeholder:text-muted-foreground text-foreground text-sm"
```

要点：
- 选中项**不再挂 hover 类**（构建期二选一），消除 Tailwind 类表顺序不定导致的
  hover 覆盖选中问题——两端同策略。
- 用户 `style:` 追加在契约串之后（codegen append），作为逃逸通道（如 rail 中
  `w-auto`、覆盖宽度/圆角）。
- VM badge 右对齐：文本列挂 `flex-1`（Fill），badge 落行尾；Vue 同类等效。
- 图标：字面量 lucide 名 → Vue 收集 import 渲染 lucide 组件 / VM `lucide_svg(name)`
  渲染 svg（16/20px）；非 lucide 名（emoji 串如 "🔌"）→ 双端按文本渲染。
  绑定表达式 icon 由 Vue 侧 NavItem 运行时判别（lucide 组件集兜底 emoji 文本），
  VM 侧构建期求值后同规则。

### D3. Vue 端实现

1. **脚手架组件** `crates/auto-man/assets/shadcn-ui/nav/`：
   - `NavItem.vue`：`<component :is="to ? RouterLink : 'button'" :to :class="cls"
     :disabled :aria-current="active ? 'page' : undefined">`；插槽：icon（默认按
     icon prop 解析 lucide/emoji）、texts（label+desc）、badge；`cls` 按契约常量+
     active/disabled/size 组合。onclick 模式透传原生 @click（codegen 挂在组件标签上）。
   - `NavGroup.vue`：label 头（collapsible 时为 button + chevron svg 旋转，open 取
     prop（v-model 式）或内部 ref 默认 true）；content 列按 GROUP_CONTENT/indent。
   - `index.ts` 导出二者。
2. **codegen**（vue.rs）：
   - map_tag：`nav-item`→`NavItem`、`nav-group`→`NavGroup`（component_refs 脚手架
     路径，参照 CodeEditor 6035-6038 行模式）；import 路径 `@/components/ui/nav`
     由 widget registry spec 提供。
   - prop 臂（参照 nav_link 8886-8907 行扩展）：to/onclick/active/exact/icon/label/
     desc/badge/disabled/size → 绑定 attr + 事件 + slot；label/desc 缺省走子节点
     转译（保留复杂子内容作默认 slot）。
   - `nav` 容器：search:true 时在容器首子位内联发射搜索行（icon search + input，
     input 绑定 search_value + oninput:onsearch + placeholder）。
   - ts_adapter：`router.back()` → `useRouter().back()`（现仅 push）。
3. **auto-man**：资产拷贝清单加 `nav/`（参照 sidebar 资产接线）。

### D4. VM 端实现

1. **aura_view_builder.rs**：
   - `nav-item`（Element/Component 各分发点，参照 nav-link 2209-2213 等四处）：
     构建 `View::Button{ label(快照/无障碍用), content: Some(Row[icon, texts, badge]),
     onclick, style, disabled }`。content 由子节点转译或 label/desc/icon/badge 合成；
     texts 有 badge/desc 时列宽 Fill。onclick：to 模式→`__navigate`(args=[to])，
     onclick 模式→用户消息。active：to 模式自动判定（`__current_route` == to 或
     前缀段 `to + "/"`；exact 收紧）或 active 表达式求值（bindings 求值，参照
     现有 prop 求值通道）。style 按契约拼装（未选中挂 hover_classes——Style 已有
     hover_classes 通道，renderer 原生 Hovered 反馈）。icon：lucide_svg 命中→
     `View::Image{src:"lucide:<name>"}`，否则文本（emoji 直接可用）。
   - `nav-group`：头行（label 或可点 Button+chevron）+ open 时 content 列（GROUP_
     CONTENT + indent）。open：绑定表达式优先；未绑定时读内置态
     `__nav_group_open:<label>#<序号>`（构建期确定 key）。折叠点击：绑定了
     ontoggle→用户消息，否则→内置 Typed 消息 `__nav_toggle`(args=[key])。
   - `nav` search 行：SEARCH_ROW 容器 + icon + `View::Input`（value/oninput/
     placeholder 接既有 input 转译）。
2. **dynamic.rs**：`__navigate` 拦截处（590-607 行）扩展——改写 `__current_route`
   前将旧值 push 进 `__route_history`（Value::Array，上限 50 截尾）；新增
   `__navigate_back`（弹出栈顶写入 `__current_route`，空栈 no-op）与
   `__nav_toggle`（翻转 `__nav_group_open:<key>` 布尔）拦截。
3. **handler_codegen.rs**：`router.back()` → 派发 `__navigate_back`（与
   router.push 改写 121-152 行同模式）。
4. **iced renderer**：预期零/极少改动（Button content 子树、hover 合并、padding、
   Fill 子项均已支持）；仅补契约 token 缺口（如 `bg-primary/15`、`rounded-full`
   徽标底、font-medium 权重解析）——以 cargo t iced + 双端截图核对驱动。
5. **lucide_svg 扩容**：补 musk/os 常用图标（message-square、list-todo、
   scroll-text、book-open、panel-left、users、bot、terminal…以实际采用清单为准）。

### D5. 三应用替换方案

1. **015-notes**（仓内，Phase B）：sidebar.at 重构——`nav(search:true, search_value:
   .search, onsearch:.SearchChanged)` + 视图页签（nav-item sm 行内样式 w-auto
   flex-1）+ 文件夹 nav-group（collapsible）内 NoteItem→nav-item（label 标题/
   desc 摘要/onclick .SelectNote/active 表达式）；标签 chips 与底部圆点保留。
   状态模式切换不变（数据过滤非页面导航）。顺带核对 VM 侧 view-fn FALLBACK
   旧 bug（tests/desktop_mcp.py:13-15 记录）是否随重构消除。
2. **018-book-reader**（仓内）：侧栏链接→nav-item(to:)，作为路由模式规范示例
   （URL/返回双端演示）。
3. **widgets-gallery**（仓内）：nav-link→nav-item；schema 中 nav-link 标注
   deprecated（指向 nav-item），组件保留不删（向后兼容）。
4. **auto-musk**（外部仓，Phase C，其仓 worktree 法）：app.at rail 4 项→
   nav-item（icon lucide 字面量、onclick .ShowX、active: .current_view=="x"、
   label），删除巨型条件样式串与逐 text 补色 workaround；composables/
   viewstate_router.ts:95-97 的结构选择器改为 `.app-rail .nav-item:nth…`
   （nav-item 基类即稳定锚点）；二级 NavSidebar/NavListItem（会话列表，非导航）
   保持不动——边界内不扩散。
5. **auto-os-config**（外部仓，Phase D）：sidebar.at→nav(search:true 集成搜索行
   替手搓 input) + 概览置顶 nav-item + `for g in .store.view_groups` →
   nav-group(label, collapsible, open: g.open, ontoggle: .ToggleGroup) 内成员
   nav-item(icon emoji/label/desc 双行 size lg/onclick .Select/active 表达式)；
   modules_store.at 五处 nav_class/name_class 投影块删除（active 判定回归视图
   表达式）；搜索扁平逻辑（搜索时隐藏组头）保留在投影。门禁：e2e.sh 28 断言 +
   e2e-vm.mjs 17 断言 + capture.mjs 双轨**重基线**（视觉微变可接受，README
   "vm 轨已知偏差"段落同步更新）。

## 测试设计

1. **单元（Rust）**：
   - nav_contract：常量逐 token `Style::parse` 可解析；与 NavItem.vue/NavGroup.vue
     资产文件内嵌串一致（CARGO_MANIFEST_DIR 相对路径读取比对）。
   - aura_view_builder：nav-item 三态（默认/选中/disabled）×双模式（to/onclick）→
     View 结构与 style 断言；nav-group open 绑定/内置/indent；nav search 行。
   - dynamic：`__navigate` 历史栈 push/上限、`__navigate_back` 弹栈、`__nav_toggle`
     翻转（沿用既有 __navigate 测试样式）。
   - handler_codegen/ts_adapter：router.back() 双端改写产物断言。
   - lucide_svg 新增图标非 None。
2. **docs_gen**：schema 变更后 `cargo test -p auto-lang --test docs_gen`（Category C）。
3. **双端可视化**：autoui-verifier 技能——015-notes/018 用 `test_vue_playwright.mjs`
   + `test_vm_mcp.py` 双端跑；hover/active 三态以 VM MCP 交互 + Vue playwright
   截图核对（需要看图时切 GLM-5.3-Flash 图像模型）。
4. **外部仓门禁**：musk vite build + web e2e；os-config e2e.sh + e2e-vm.mjs +
   capture.mjs 重基线对比。
5. **全量**：`cargo tf`（涉及编译器/schema，合入前一次，Plan 466 门禁）。

## 验收标准

1. schema/aura.at：nav-item/nav-group 双端 full 声明（含 props/aliases），nav 具备
   search 四属性；docs_gen 通过；nav-link 标 deprecated。
2. nav-item 三态双端一致：hover 高亮（VM 为 iced 原生 Hovered 态）+ pointer cursor
   （Vue）；选中 `bg-primary/10 text-primary font-medium`；disabled 灰置不可点。
3. 路由：018 示例 Vue 模式 URL 随导航变化且浏览器返回可用；VM 模式
   `__current_route` 切换且 `router.back()` 沿历史栈返回（单元测试覆盖）。
4. nav-group 可折叠（绑定与内置两种开合来源）、chevron 指示、indent 缩进，
   双端结构一致；nav(search:true) 双端渲染同构搜索行并正确派发 onsearch。
5. 三 app 替换完成且各自门禁全绿：015-notes 双端脚本通过；musk build+e2e 通过、
   viewstate_router 桥选择器更新；os-config 28+17 断言通过、parity 台账重基线、
   modules_store 投影块删除（-5×~45 行）。
6. 契约测试通过（nav_contract 全 token 可解析 + 资产镜像一致）。
7. `cargo tf` 全绿（唯一失败须为 master 既有、与本改动零交集）。

## 执行步骤

> 阶段 A/B 在 `.worktrees/plan-482-dev`（本仓）；阶段 C/D 在外部仓各自规范下执行
> （musk：`.worktrees/plan-051-dev`；os-config：其 docs/plans/012 流程）。
> 前置：master 上 commit `.next-id` 与本 plan 骨架后再建 worktree。

### 阶段 A：框架（本仓 worktree）

- [x] **T1** `schema/aura.at`：改写 `nav`（category navigation + search/
  [✅ 已完成] cargo check -p auto-lang 通过（schema nav/nav-link/nav-group/nav-item 声明）
  search_placeholder/search_value/onsearch 四 props）、`nav-link`（description 标
  deprecated→nav-item）、新增 `nav-item`/`nav-group` 元素（aliases nav_item/NavItem/
  nav_group/NavGroup，backends web:component+iced:full，vue component NavItem/
  NavGroup import "@/components/ui/nav"，props 按 D1）。验证：`cargo check -p auto-lang`。
- [x] **T2** `crates/auto-lang/src/ui_gen/widget/registry.rs`：注册 NavItem/NavGroup
  [✅ 已完成] cargo check 通过 + registry NavItem/NavGroup spec 注册（schema→registry vue 映射自动同步验证于 T9 测试）
  WidgetSpec（Category Navigation，含别名与 import 路径，参照 NavLink 1270-1274 与
  Sidebar 家族 1168-1202 行）。验证：`cargo check -p auto-lang` + registry 单测。
- [x] **T3** 新建 `crates/auto-lang/src/ui/nav_contract.rs`：D2 全部常量 + 单元测试
  [✅ 已完成] cargo t nav_contract 3 测全绿（token 可解析 + hover 通道 + 资产镜像）
  （逐 token Style::parse 可解析；与 auto-man 资产文件镜像比对——资产 T11 落地后
  启用比对断言）。验证：`cargo t nav_contract`。
- [x] **T4** `crates/auto-lang/src/ui/aura_view_builder.rs`：nav-item 转换（Element/
  [✅ 已完成] cargo t test_nav_item_* 全绿（路由/onclick 双模式 × 三态结构断言）
  Component 分发点 + active 自动/显式判定 + to/onclick 双模式 + icon svg/emoji +
  badge/desc 布局 + disabled）+ 单测（三态×双模式结构断言）。验证：`cargo t aura`。
- [x] **T5** 同文件：nav-group 转换（头行/折叠/内置态 key/indent）+ nav search 行
  [✅ 已完成] cargo t test_nav_group_fold_states / test_nav_search_row 全绿
  发射 + 单测。验证：`cargo t aura`。
- [x] **T6** `crates/auto-lang/src/ui/dynamic.rs`：`__navigate` 历史栈（push 上限
  [✅ 已完成] cargo t test_nav_route_history_push_and_back / test_nav_toggle_group_state_explicit 全绿
  50）+ `__navigate_back` + `__nav_toggle` 拦截 + 单测。验证：`cargo t dynamic`。
- [x] **T7** `crates/auto-lang/src/ui/handler_codegen.rs`：router.back() →
  [✅ 已完成] cargo t rewrites_router_back_to_pending_flag 全绿
  `__navigate_back` 改写 + 单测。验证：`cargo t handler`。
- [x] **T8** `crates/auto-lang/src/ui/render_support.rs`：nav-item/nav-group full 行、
  [✅ 已完成] cargo check 通过（nav-item/nav-group full、nav-link deprecated 措辞）
  nav 行补 search 说明、nav-link 行改 deprecated 措辞。验证：`cargo check -p auto-lang`。
- [x] **T9** `crates/auto-lang/src/ui_gen/vue.rs`：map_tag nav-item/nav-group →
  [✅ 已完成] cargo t test_nav_family_shadcn_sfc / test_nav_family_inline_sfc 全绿（双模式产物断言）
  脚手架组件（component_refs 路径，参照 CodeEditor 模式）+ prop 臂（全部 D1 属性
  → attr/事件/slot）+ nav search 行内联发射 + codegen 单测（产物含 NavItem/
  NavGroup import、:active 绑定、@click、search input）。验证：`cargo t vue_gen`。
- [x] **T10** `crates/auto-lang/src/ui_gen/ts_adapter.rs`：router.back() →
  [✅ 已完成] stmts_have_router_nav 增 back 检测（router.back() 通用转译已有,仅需 import 触发）
  `useRouter().back()` + 单测。验证：`cargo t ts_adapter`。
- [x] **T11** `crates/auto-man/assets/shadcn-ui/nav/`：NavItem.vue/NavGroup.vue/
  [✅ 已完成] assets/shadcn-ui/nav/ 三文件 + COMPONENT_PATTERNS 接线 + SNAPSHOT.md 登记;T3 镜像测试转绿 + plan_457 目录同步测试绿
  index.ts（按 D2 契约与 D3 结构）；`crates/auto-man/src/vue.rs` 资产拷贝清单接
  nav/（参照 sidebar 接线）。验证：T3 镜像比对测试转绿 + `cargo check -p auto-man`。
- [x] **T12** `crates/auto-lang/src/ui/iced/renderer.rs`：按双端截图核对补契约
  [✅ 已完成] nav_contract_tokens_parse_on_vm 全 token 断言（bg-primary/15、rounded-full、py-[2px]、font-medium 等均过解析门）
  token 缺口（badge 药丸底/rounded-full/bg-primary/15/font-medium 等，预计小改）。
  验证：`cargo t iced`。
- [x] **T13** lucide_svg 扩容（musk/os 采用清单内缺失图标，附路径数据来源注释）。
  [✅ 已完成] nav_family_lucide_icons_present 测试锁 musk 四图标+chevron+search 全在内嵌集（84 项无需扩容）
  验证：`cargo t lucide`。

### 阶段 B：仓内示例（同 worktree）

- [x] **T14** `examples/ui/018-book-reader/src/front/app.at`：侧栏链接→nav-item
  [✅ 已完成] 018 sidebar 两 link→nav-item(to:,exact)；auto gen 产物含 NavItem+契约 import
  (to:)，跑双端验证脚本确认 URL/返回/选中三态。验证：autoui-verifier 双端脚本。
- [x] **T15** `examples/ui/015-notes/src/front/sidebar.at`（+app.at 接线）：按 D5.1
  [✅ 已完成] 015 sidebar 重构 nav(search:)+nav-group×4+NoteNav(原 NoteItem)+SearchChanged 接线（原死输入框修复）；auto gen 产物验证（v-model+@input 带文本实参+NavItem :active）
  重构（nav search 集成 + nav-group 文件夹 + NoteItem→nav-item）；核对 VM view-fn
  FALLBACK 旧 bug 消除。验证：autoui-verifier 双端脚本 + tests/desktop_mcp.py。
- [x] **T16** `examples/widgets-gallery/src/front/app.at`：nav-link→nav-item。
  [✅ 已完成] widgets-gallery 66 处 nav-link→nav-item 一对一替换（render:vm 工程,gen 跳 vue 为既有行为;VM 轨验证并入 T14/T15 批次）
  验证：auto gen 产物 diff 检查 + 快速双端跑。

### 阶段 C：auto-musk（外部仓，其 worktree 法 + 其 plan 051 文档）

- [x] **T17** musk 仓建 plan 051 文档（rail nav-item 化 + viewstate_router 选择器
  [✅ 已完成] musk 仓 docs/plans/052-nav-item-rail.md 起草并提交(08d59ce 链)
  更新方案）。验证：文档评审（用户确认）。
- [x] **T18** `src/front/app.at` rail 4 项→nav-item；`composables/viewstate_router.ts`
  [✅ 已完成] musk rail 4 项 nav-item+桥选择器 .app-rail .nav-item;vite build 3.20s 绿;plan-052-dev 折叠 main 并清理
  选择器改 `.app-rail .nav-item`；删除逐 text 补色 workaround；lucide 字面量 icon
  （VM 走 lucide_svg，T13 已扩容）。验证：`auto build --gen-only` + `npx vite
  build` + web e2e + VM 双端抽查截图。

### 阶段 D：auto-os-config（外部仓，其 docs/plans/012 流程）

- [x] **T19** os-config 仓建 plan 012 文档（sidebar 组件化 + 投影瘦身 + parity
  [✅ 已完成] os-config 仓 docs/plans/012-nav-item-sidebar.md 起草并提交(9ca1449 链)
  重基线方案）。验证：文档评审（用户确认）。
- [x] **T20** `auto/src/front/sidebar.at`→nav/nav-group/nav-item（D5.5）；
  [✅ 已完成] sidebar.at nav 组件族化+modules_store -6.6KB;e2e 三套件 PASS+e2e-vm 全断言 PASS;README 更新
  `modules_store.at` 删五处 nav_class/name_class 投影块；`auto/README.md` 偏差段
  更新。验证：`./scripts/e2e.sh`（28 断言）+ `node scripts/e2e-vm.mjs`（17 断言）。
- [x] **T21** `node scripts/track-parity/capture.mjs --track vue|vm` 双轨重基线并
  [✅ 已完成] capture.mjs 双轨捕获(顺修 plan011 既有就绪串缺口);diff sidebar 2.1-5.0% 新基线落 docs/plans/012
  核对 sidebar 差异率。验证：capture 脚本产出 + 台账提交。

### 阶段 E：收口（本仓）

- [x] **T22** `cargo test -p auto-lang --test docs_gen`（schema 文档再生成校验）。
  [✅ 已完成] docs_gen 再生成(core.md/kitchen-sink)+围栏修复后 4/4 绿
- [x] **T23** `cargo tf` 全量门禁（Plan 466）；唯一失败须为 master 既有。
  [✅ 已完成] cargo tf 3258/3258 绿(折叠前终跑);tv 两失败基线裁定 master 既有(3b2b20b05 复现)
- [x] **T24** /auto-plan:review 独立复审（验收标准逐条对代码核验 + 遗漏/延后扫描 +
  [✅ 已完成] 复审记录+spec-impact 填写+P482-1..4 债务登记+status→reviewed(本 merge 前置)
  spec-impact metadata 填写）→ status: reviewed → merge（含 spec ledger 沉淀：
  GOAL-007 契约与组件、GOAL-010 示例演进）。

## 复审记录

> 2026-08-29 一轮复审见下（随 review 提交 8dd4a3167）。以下为用户指令发起的
> **二轮独立事后复审**（归档后复审；对"声称完成"重新核验，非信任一轮结论）。

**2026-08-29 /auto-plan:review 二轮（zcode）— PASS（补 1 处沉淀遗漏，当日清偿）**

核验方法：worktree 已折叠，全部对 master HEAD（b78ad7050）与两外部仓 main 重跑。

关键发现与处置：
1. **补遗 3/4/5 未过全量门禁（本轮主要缺口）**：一轮 review 的 cargo tf 跑在
   补遗 2（ff7f1f0bd）之后，补遗 3/4/5（nav-name/nav-desc 锚、文本 chevron、
   搜索图标分轨）在其后提交而未重跑。本轮补跑：**cargo tf 3258/3258 绿**；
   cargo tv 失败集 = cb_asynchronous_channel + aavm2_m4，与一轮基线裁定
   （3b2b20b05 复现，master 既有）完全一致，无新增回归。✅
2. **遗漏 #1（已当场清偿）**：merge 沉淀缺 ui/plans.md 索引行与 overview
   现状段（479 先例有、482 漏落）——本轮补 482 行 + 导航组件线段落。✅
3. scoped 门禁 master 重跑：nav 单测 13+4 绿（含 router.back 改写/lucide
   存在性/双模式 SFC）、docs_gen 4/4、gallery_golden 1/1、auto-man plan_457
   同步 1/1。✅
4. 外部仓终态核验：musk main 08d59ce（rail nav-item×4 + .app-rail 锚 +
   桥选择器）、os-config main 9ca1449（sidebar 组件族 + nav_class 残留仅
   1 处注释）。✅
5. Workaround/TODO 扫描：Plan 482 diff 范围内零 TBD/FIXME/HACK/临时标记。✅
6. 债务复核：P482-1..4 已在 KNOWN-DEBT 登记（musk 视觉复跑/015 搜索过滤/
   015 置顶标记/title 通道），无未批准延后。✅

结论：维持 archived 终态；沉淀遗漏当日清偿，无阻塞项。



**2026-08-29 /auto-plan:review（zcode）— PASS（附 4 项债务登记）**

验收逐条核验（对实际代码/运行证据）：
1. ✅ schema nav-item/nav-group 双端 full + nav search 四属性；docs_gen 4/4 绿；
   nav-link deprecated；gallery /navitem 演示页（全属性表）+ 金样基线更新。
2. ✅ 三态双端：单测锁（10 nav 测试全绿）+ 015/018 双端截图实测（选中
   bg-primary/10 text-primary、点击切换、hover 由 iced Hovered 类合并承载）。
3. ✅ 路由：018 Vue hash URL 随导航变化（#/ → #/settings 实测）；VM
   __current_route 切换 + Settings 页渲染 + __navigate 处理器实测；历史栈/
   back 单测（dynamic/handler_codegen）+ 渲染器拦截同 __navigate 家族路径。
   ※ router.back() 无 018 UI 入口，VM 实操返回未演示——单测覆盖，D6 登记。
4. ✅ nav-group 折叠双端（os-config e2e-vm 折叠断言 PASS；内置/绑定两种
   open 来源单测）；search 行双端（015 实测+os-config e2e）；indent 单测。
5. ✅ 三 app：015 双端实测；musk rail 3.2KB→776B+vite build 绿+产物核验；
   os-config 双门禁全绿（三套件+e2e-vm 全断言）+投影块 -6.6KB+parity
   重基线（sidebar 全窗 2.1–5.0%，新基线落 docs/plans/012）。
6. ✅ 契约测试 3 项全绿（token VM 可解析+资产镜像+hover 通道）。

全量门禁：cargo tf 3258/3258 绿 ×3（阶段 A 后/阶段 B 后/折叠前）；tv 两失败
（cb_asynchronous_channel/aavm2_m4）经基线 worktree 3b2b20b05 复现裁定
master 既有、零交集；gallery_golden/docs_gen 更新后绿。

遗漏/延后扫描 → KNOWN-DEBT 登记 P482-1..4（见下）；workaround 扫描：无新增
逃逸补丁（os-config regen.sh :title 补偿仍在——desc 截断悬停提示，组件化后
nav-item 同样无 title 通道，属上游组件属性候选，随 D4 记录）。

## 待澄清事项

1. **015-notes 是否引入 routes**：本计划维持状态模式（切换本质是数据过滤）；
   若希望它演示 URL 语义可后续追加（低风险增量）。
2. **musk 是否借机切 hash routes 并删除 viewstate_router 桥**：本计划暂缓（保持
   其真实路径 URL 语义与 popstate 行为，仅换锚点选择器）；切 routes 属独立决策。
3. **os-config parity 台账重基线**：组件化后视觉微变（选中块、搜索行、组头样式
   统一为主色调契约），默认接受重基线；若要求零视觉差需另议契约微调。
4. **nav-link 去留**：保留实现、schema 标 deprecated（widgets-gallery 迁走后无仓内
   消费者）；是否在更早版本物理删除由后续版本策略定。
