# Design 19: AutoUI 统一的深浅色模式与主题色配置方案

> **状态**: 设计完成，待实施
> **日期**: 2026-07-17
> **影响**: 所有 UI 后端（Vue / Rust / Jet / Ark）
> **依赖**: shadcn CSS 变量体系（已内置）、Tailwind `darkMode: "class"`

---

## 1. 问题

AutoUI 的 view DSL 里目前硬编码具体颜色（如 `bg-blue-500`、`text-gray-800`）。这导致：

1. **深色模式**：需要为每个元素写两套 class（`bg-white dark:bg-gray-900`），冗长且易遗漏。
2. **主题色切换**：无法切换主色调（蓝→绿→紫），因为颜色写死在 `.at` 源码里。
3. **跨后端不一致**：`bg-blue-500` 在 Vue 里是 Tailwind class，在 Rust/GPUI 里没有对应物，Jet/Ark 也没有。
4. **各后端各自的主题体系不同**：Vue 用 CSS 变量，Rust 用 `Color` 枚举，Jet 用 MaterialTheme，Ark 用资源引用。

## 2. 设计目标

- `.at` 源码用**语义化 token**（`bg-primary`、`text-foreground`），不硬编码颜色。
- 各后端生成器各自把 token dispatch 为底层实现。
- 深浅色模式和主题色切换是**正交的两个维度**，可独立或组合使用。
- pac.at 可选声明主题色表覆盖默认值。

## 3. Token 体系

### 3.1 语义化 token 列表

采用 shadcn 的语义化颜色命名（已经是事实标准），每个 token 有 light/dark 两套值：

| Token | 含义 | Light 默认 | Dark 默认 |
|---|---|---|---|
| `background` | 页面背景 | `#ffffff` | `#0f172a` |
| `foreground` | 页面文字 | `#0f172a` | `#f1f5f9` |
| `primary` | 主色（按钮、高亮） | `#3b82f6` | `#60a5fa` |
| `primary-foreground` | 主色上的文字 | `#ffffff` | `#0f172a` |
| `secondary` | 次要按钮 | `#f1f5f9` | `#1e293b` |
| `secondary-foreground` | 次要按钮文字 | `#0f172a` | `#f1f5f9` |
| `muted` | 静默背景（标签栏、禁用态） | `#f1f5f9` | `#1e293b` |
| `muted-foreground` | 静默文字（时间戳、说明） | `#64748b` | `#94a3b8` |
| `accent` | 强调背景（选中行、hover） | `#dbeafe` | `#1e3a5f` |
| `accent-foreground` | 强调背景上的文字 | `#1d4ed8` | `#93c5fd` |
| `destructive` | 危险色（删除按钮） | `#ef4444` | `#dc2626` |
| `destructive-foreground` | 危险色上的文字 | `#ffffff` | `#ffffff` |
| `border` | 边框 | `#e2e8f0` | `#334155` |
| `card` | 卡片背景 | `#ffffff` | `#1e293b` |
| `card-foreground` | 卡片文字 | `#0f172a` | `#f1f5f9` |

### 3.2 Tailwind class 映射

| 硬编码 class | 语义化 class |
|---|---|
| `bg-white` / `bg-gray-900` | `bg-background` |
| `text-gray-800` / `text-gray-100` | `text-foreground` |
| `bg-blue-500` | `bg-primary` |
| `text-white`（按钮上） | `text-primary-foreground` |
| `bg-gray-100` / `bg-blue-100` | `bg-accent`（选中态）或 `bg-muted`（静默态） |
| `text-gray-400` / `text-gray-500` | `text-muted-foreground` |
| `border-gray-200` / `border-gray-700` | `border-border` |
| `bg-red-500` | `bg-destructive` |

### 3.3 主题色预设

| 预设名 | primary | primary-foreground | 适用场景 |
|---|---|---|---|
| `blue`（默认） | `#3b82f6` | `#ffffff` | 通用 |
| `green` | `#22c55e` | `#ffffff` | 健康/环保 |
| `purple` | `#8b5cf6` | `#ffffff` | 创意/社交 |
| `orange` | `#f97316` | `#ffffff` | 餐饮/活力 |
| `rose` | `#f43f5e` | `#ffffff` | 时尚/情感 |

每个预设有 light/dark 两套 primary 值（dark 下调亮饱和度）。

## 4. 架构设计

### 4.1 三层分离

```
Auto .at 源码（统一语义层）
  bg-primary, text-foreground, bg-background
        │
        ▼
  AuraAST / AuraNode（保持 token 为 symbol，不提前展开）
        │
        ├── Vue 生成器 → shadcn CSS 变量（直接输出 bg-primary）
        ├── Rust 生成器 → 查主题表，输出 Color::from_hex("#3b82f6")
        ├── Jet 生成器 → MaterialTheme.colorScheme.primary
        └── Ark 生成器 → $r('app.color.primary')
```

**核心原则**：语义化 token 是 Auto 语言的"虚拟寄存器"，各后端生成器是"指令选择器"——同一个 token 在不同后端展开为不同的底层实现。

### 4.2 各后端的 dispatch 策略

#### Vue（已就绪）

shadcn-vue 已内置完整的 CSS 变量体系：
- `tailwind.config.cjs` 定义 `primary: "hsl(var(--primary))"` 等颜色
- `index.css` 定义 `:root { --primary: ... }` 和 `.dark { --primary: ... }`
- 生成器直接输出 `class="bg-primary"`，浏览器运行时解析 CSS 变量
- 主题切换：改根元素 class（`dark`、`theme-green`）

**零额外工作**——Vue 生成器只需透传语义化 class。

#### Rust (iced/GPUI)

```rust
// 生成器维护 token → Color 映射表
fn resolve_color(token: &str, dark: bool) -> Color {
    match (token, dark) {
        ("primary", false) => Color::from_rgb(0.231, 0.510, 0.965),  // #3b82f6
        ("primary", true)  => Color::from_rgb(0.376, 0.647, 0.980),  // #60a5fa
        ("background", false) => Color::WHITE,
        ("background", true)  => Color::from_rgb(0.059, 0.090, 0.165), // #0f172a
        ...
    }
}

// 生成的代码
container.style = Style {
    background: Some(resolve_color("primary", dark_mode)),
    text_color: Some(resolve_color("primary-foreground", dark_mode)),
};
```

主题切换时：重新渲染（reactive 模式下自动响应 `dark_mode` 状态变化）。

#### Jetpack Compose

```kotlin
// 语义化 token 映射到 MaterialTheme
Button(
    colors = ButtonDefaults.buttonColors(
        containerColor = MaterialTheme.colorScheme.primary,
        contentColor = MaterialTheme.colorScheme.onPrimary
    )
)

// 主题定义在 Theme.kt
val LightColors = lightColorScheme(primary = Color(0xFF3B82F6), ...)
val DarkColors = darkColorScheme(primary = Color(0xFF60A5FA), ...)
```

Jet 生成器输出 `MaterialTheme.colorScheme.primary`，主题切换由 Compose 的 `isSystemInDarkTheme()` 驱动。

#### ArkTS

```typescript
// 资源文件 resources/base/element/color.json
{ "color": [
  { "name": "primary", "value": "#3b82f6" }
]}

// 生成的代码
Button().backgroundColor($r('app.color.primary'))
```

主题切换通过 ArkTS 的 AppStorage + 资源限定词（`dark/element/color.json`）。

### 4.3 主题色表配置（pac.at）

pac.at 可选声明主题色，覆盖默认值：

```auto
name: "notes"
theme: {
  mode: "auto"           // "light" | "dark" | "auto"（跟随系统）
  color: "blue"          // "blue" | "green" | "purple" | "orange" | "rose"
}
```

或自定义色表：

```auto
theme: {
  mode: "dark"
  colors: {
    primary: "#22c55e"
    primary_foreground: "#ffffff"
  }
}
```

生成器读取 `theme` 配置，在各后端生成对应的主题定义：
- Vue：生成 `index.css` 的 CSS 变量初始值
- Rust：生成 `Theme` struct 的颜色值
- Jet：生成 `ColorScheme`
- Ark：生成资源文件

### 4.4 深浅色与主题色的正交组合

```
根元素 class 组合（Vue）：
  ""                 → 浅色 + 蓝色主色
  "dark"             → 深色 + 蓝色主色
  "theme-green"      → 浅色 + 绿色主色
  "dark theme-green" → 深色 + 绿色主色

CSS 变量分层覆盖：
  :root              { --primary: 222 47% 11%; --background: 0 0% 100%; }
  .dark              { --background: 222 47% 11%; --foreground: 210 40% 98%; }
  .theme-green       { --primary: 142 71% 45%; }
  .dark.theme-green  { --primary: 142 71% 35%; }
```

两个维度完全独立，无冲突。

## 5. Auto 语言层改动

### 5.1 view DSL 的 style 属性

**当前**（硬编码）：
```auto
button { style: "bg-blue-500 text-white px-4 py-2 rounded-lg" }
```

**改为**（语义化）：
```auto
button { style: "bg-primary text-primary-foreground px-4 py-2 rounded-lg" }
```

非颜色相关的 class（`px-4`, `py-2`, `rounded-lg`）保持不变。只替换颜色相关的 class。

### 5.2 深色模式 handler（已实现）

```auto
widget App {
    msg Msg { ..., ToggleDarkMode }
    on {
        .ToggleDarkMode -> { store.ToggleDarkMode() }
    }
}
```

Vue 生成器检测到 `ToggleDarkMode` handler → 注入 `:class="{ dark: store.dark_mode }"` 到根元素。

### 5.3 主题切换 handler（新增）

```auto
widget App {
    msg Msg { ..., ToggleTheme }
    on {
        .ToggleTheme -> { store.CycleTheme() }
    }
}
```

Store 维护 `theme_color: str` 状态。Vue 生成器注入 `:class="{ ['theme-' + store.theme_color]: true }"` 到根元素。

## 6. 生成器改动

### 6.1 Vue 生成器（改动最小）

**已就绪**：
- shadcn CSS 变量体系已内置
- `dark` class 注入已实现
- 语义化 class 直接透传（Tailwind 编译器处理）

**需新增**：
- `theme-xxx` class 注入（类似 dark mode 的机制）
- `index.css` 生成时从 pac.at 读取自定义色表覆盖默认值

### 6.2 Rust 生成器

**需新增**：
- `resolve_color(token, dark)` 函数
- 生成器在遇到 `bg-primary` 等 token 时调用 resolve_color
- 生成 `Theme` struct 定义

### 6.3 Jet 生成器

**需新增**：
- `bg-primary` → `MaterialTheme.colorScheme.primary`
- 生成 `ColorScheme` 定义

### 6.4 Ark 生成器

**需新增**：
- `bg-primary` → `$r('app.color.primary')`
- 生成资源文件

## 7. 迁移策略

### Phase 1：Vue 后端（立即可做）

1. 把 015-notes 的 `.at` 文件中的硬编码颜色替换为语义化 token
2. 验证深色模式 + 正常模式都正确渲染
3. 更新 Plan 357 和 Plan 354

改动范围：
- `app.at`：所有 `bg-white` → `bg-background`，`text-gray-800` → `text-foreground` 等
- `editor.at`：同上
- `notes_store.at`：无样式，无需改

### Phase 2：主题色切换（Vue）

1. pac.at 支持 `theme.color` 配置
2. Store 加 `theme_color` 状态 + `CycleTheme` action
3. Vue 生成器注入 `theme-xxx` class
4. `index.css` 生成时读取 pac.at 色表

### Phase 3：其他后端

1. Rust 生成器：`resolve_color` + `Theme` struct
2. Jet 生成器：MaterialTheme 映射
3. Ark 生成器：资源文件映射

## 8. 验收标准

- [ ] `.at` 源码中不出现硬编码颜色（`bg-blue-500`、`text-gray-800` 等）
- [ ] Vue 后端：深色模式自动切换（已实现 `dark` class 注入）
- [ ] Vue 后端：主题色可切换（至少支持 blue/green/rose 三种预设）
- [ ] Rust 后端：语义化 token 正确解析为 Color 值
- [ ] Jet/Ark 后端：映射到各自主题系统
- [ ] pac.at 可选声明 `theme` 配置
- [ ] 015-notes 作为参考实现，完整使用语义化 token
