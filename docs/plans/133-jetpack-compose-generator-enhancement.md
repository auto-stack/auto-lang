# Jetpack Compose 生成器增强设计

> **目标**: 让 Jet 生成器支持完整 AURA 语法，使同一份 Auto 代码可生成 Vue/Tauri/Jet 三端

## 1. 背景与问题

### 当前状态

| 平台 | 生成器状态 | 语法支持 |
|------|-----------|----------|
| Vue | ✅ 成熟 | 完整 AURA (model, computed, view, on, msg, slot) |
| Tauri | ✅ 可用 | 复用 Vue + IPC bridge |
| Jet | ⚠️ 早期 | 简化语法，属性风格不一致 |

### 核心问题

1. **语法不一致**: jetdemo 使用 `padding: 16` 而 component-gallery 使用 `class: "p-4"`
2. **组件覆盖不足**: Jet 只支持 ~22 个组件，component-gallery 需要 50 个
3. **工具链未集成**: Jet 没有集成到 `auto build` / `auto run` 流程

## 2. 设计决策

| 决策点 | 选择 | 理由 |
|--------|------|------|
| 语法统一 | Tailwind CSS 优先 | 一套代码三端生成 |
| 适配策略 | Jet 适配完整 AURA | 保持 AURA 平台无关性 |
| 实施策略 | 分阶段实现 | 快速迭代，降低风险 |
| 第一阶段范围 | 10 个高频组件 | 覆盖 80% 使用场景 |
| 工具链 | Gradle CLI | 自动化，CI 友好 |
| 输出目录 | `jet/` | 与 `vue/` 保持一致 |

## 3. 统一 AURA 语法

### 语法示例

```auto
widget Card {
    msg Msg { Click }

    model {
        title str = "Card Title"
        variant str = "default"  // default, outlined, elevated
    }

    computed {
        cardClass => f"rounded-lg border p-4 ${.variant == "elevated" ? "shadow-lg" : ""}"
    }

    view {
        div (class: .cardClass) {
            h3 (class: "text-lg font-semibold", text: .title) {}
            slot
        }
    }

    on {
        Click => {
            // 事件处理
        }
    }
}
```

### 生成对比

| AURA 特性 | Vue 输出 | Jet 输出 |
|-----------|----------|----------|
| `class: "flex gap-4"` | `<div class="flex gap-4">` | `Row(horizontalArrangement = Arrangement.spacedBy(16.dp))` |
| `class: "p-4"` | CSS class | `Modifier.padding(16.dp)` |
| `class: "text-lg"` | CSS class | `TextStyle(fontSize = 18.sp)` |
| `class: "rounded-lg"` | CSS class | `Modifier.clip(RoundedCornerShape(8.dp))` |
| `slot` | `<slot />` | `content: @Composable () -> Unit` |
| `msg Msg { Click }` | `emit('click')` | `onClick: () -> Unit` |
| `computed { ... }` | `computed(() => ...)` | `derivedStateOf { ... }` |

## 4. 架构设计

### 模块结构

```
┌─────────────────────────────────────────────────────────────┐
│                     AURA AST (统一抽象)                      │
│   widget, model, computed, view, on, msg, slot              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Shared Generator Core                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ Tailwind    │  │ Component   │  │ State/Event         │  │
│  │ Parser      │  │ Registry    │  │ Analyzer            │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  Vue Generator  │ │ Tauri Generator │ │  Jet Generator  │
│                 │ │                 │ │                 │
│ Tailwind → CSS  │ │ Tailwind → CSS  │ │ Tailwind →      │
│ class="..."     │ │ + IPC bridge    │ │ Modifier chain  │
└─────────────────┘ └─────────────────┘ └─────────────────┘
```

### 新增文件

| 文件 | 职责 |
|------|------|
| `ui_gen/shared/mod.rs` | 共享模块入口 |
| `ui_gen/shared/tailwind.rs` | Tailwind 类名解析器 |
| `ui_gen/shared/registry.rs` | 组件映射注册表 |
| `ui_gen/shared/state.rs` | model/computed/msg 分析 |
| `ui_gen/shared/style.rs` | ComputedStyle 数据结构 |
| `ui_gen/jet/tailwind_to_mod.rs` | Tailwind → Modifier 转换 |

## 5. Tailwind 解析器

### 数据结构

```rust
pub struct ComputedStyle {
    // Layout
    pub display: Display,           // flex, grid, block, none
    pub direction: FlexDirection,   // row, col
    pub gap: Option<Dimension>,     // gap-4 → 16.dp

    // Spacing
    pub padding: Spacing,           // p-4 → 16.dp all sides
    pub margin: Spacing,            // m-4 → 16.dp all sides

    // Size
    pub width: Size,                // w-full → fillMaxWidth()
    pub height: Size,               // h-screen → fillMaxHeight()

    // Typography
    pub font_size: Option<Dimension>,  // text-lg → 18.sp
    pub font_weight: Option<FontWeight>, // font-bold → Bold
    pub text_align: Option<TextAlign>,  // text-center → Center

    // Background
    pub bg_color: Option<Color>,    // bg-blue-500 → Color(0xFF3B82F6)

    // Border
    pub border_radius: Option<Dimension>, // rounded-lg → 8.dp
    pub border_width: Option<Dimension>,  // border-2 → 2.dp
    pub border_color: Option<Color>,      // border-gray-300

    // Effects
    pub shadow: Option<Shadow>,     // shadow-lg → 8.dp elevation
    pub opacity: Option<f32>,       // opacity-50 → 0.5f
}
```

### Tailwind → Jet 转换表

| Tailwind | Jetpack Compose Modifier |
|----------|-------------------------|
| `p-4` | `.padding(16.dp)` |
| `px-2` | `.padding(horizontal = 8.dp)` |
| `py-4` | `.padding(vertical = 16.dp)` |
| `gap-4` | `Arrangement.spacedBy(16.dp)` |
| `flex` | `Row` / `Column` |
| `flex-col` | `Column` |
| `flex-row` | `Row` |
| `w-full` | `.fillMaxWidth()` |
| `h-screen` | `.fillMaxHeight()` |
| `bg-blue-500` | `.background(Color(0xFF3B82F6))` |
| `text-center` | `textAlign = TextAlign.Center` |
| `text-lg` | `fontSize = 18.sp` |
| `font-bold` | `fontWeight = FontWeight.Bold` |
| `rounded-lg` | `.clip(RoundedCornerShape(8.dp))` |
| `shadow-lg` | `.shadow(8.dp)` |
| `border` | `.border(1.dp, Color.Gray)` |

## 6. 组件注册表

### 数据结构

```rust
pub struct ComponentRegistry {
    mappings: HashMap<String, ComponentMapping>,
}

pub struct ComponentMapping {
    pub tag: String,
    pub vue: VueMapping,
    pub jet: JetMapping,
}

pub struct VueMapping {
    pub import: Option<String>,
    pub component: String,
    pub props: HashMap<String, String>,
}

pub struct JetMapping {
    pub import: String,
    pub composable: String,
    pub props: HashMap<String, String>,
    pub modifier_props: Vec<String>,
}
```

### 第一阶段组件映射 (10个)

| AURA Tag | Vue (shadcn-vue) | Jet (Material3) |
|----------|------------------|-----------------|
| `button` | `Button` | `Button(onClick)` |
| `input` | `Input` + `v-model` | `OutlinedTextField(value, onValueChange)` |
| `card` | `Card` | `Card` |
| `dialog` | `Dialog` | `AlertDialog(onDismissRequest)` |
| `form` | `<form>` | `Column` + validation |
| `toast` | `Sonner` | `Snackbar(hostState)` |
| `table` | `Table` | `LazyColumn` + rows |
| `tabs` | `Tabs` | `TabRow` + `Tab` |
| `avatar` | `Avatar` | `Image` + `Modifier.size(40.dp).clip(CircleShape)` |
| `badge` | `Badge` | `Badge` |

## 7. 目录结构约定

### 输入 (AURA 源码)

```
my-project/
├── pac.at                    # scene: "ui", backend: ["vue", "tauri", "jet"]
└── source/front/
    ├── app.at
    └── components/
        ├── button.at
        └── card.at
```

### 输出 (生成目录)

```
my-project/
├── vue/                      # Vue 项目
│   ├── src/
│   ├── package.json
│   └── vite.config.ts
│
├── tauri/                    # Tauri 项目 (继承 vue/)
│   ├── src-tauri/
│   └── (共享 vue/ 配置)
│
└── jet/                      # Jetpack Compose 项目
    ├── app/
    │   ├── src/main/java/com/example/myproject/
    │   │   ├── MainActivity.kt
    │   │   ├── ui/
    │   │   │   ├── theme/
    │   │   │   └── widgets/
    │   │   └── build.gradle.kts
    │   └── build.gradle.kts
    ├── gradle/
    ├── settings.gradle.kts
    └── build.gradle.kts
```

### 构建命令

```bash
auto build    # 生成 vue/ + tauri/ + jet/ (根据 backend 配置)
auto run      # vue: npm run dev
              # tauri: npm run tauri dev
              # jet: 提示 "Open jet/ in Android Studio"
```

## 8. 实施计划

### Phase 1: 基础设施 (1周)

| 任务 | 文件 | 说明 |
|------|------|------|
| 1.1 | `ui_gen/shared/mod.rs` | 创建共享模块入口 |
| 1.2 | `ui_gen/shared/tailwind.rs` | Tailwind 类名解析器 |
| 1.3 | `ui_gen/shared/registry.rs` | 组件映射注册表 |
| 1.4 | `ui_gen/shared/state.rs` | model/computed/msg 分析 |
| 1.5 | `ui_gen/shared/style.rs` | ComputedStyle 数据结构 |

### Phase 2: Jet 生成器重构 (1周)

| 任务 | 文件 | 说明 |
|------|------|------|
| 2.1 | `jet/generator.rs` | 重构主生成器，使用统一 AST |
| 2.2 | `jet/tailwind_to_mod.rs` | 样式 → Modifier 转换 |
| 2.3 | `jet/components/*.rs` | 10 个组件实现 |
| 2.4 | `jet/project.rs` | Android 项目模板 |

### Phase 3: Vue 生成器对齐 (0.5周)

| 任务 | 文件 | 说明 |
|------|------|------|
| 3.1 | `vue.rs` | 使用统一 Tailwind 解析 |
| 3.2 | 清理 | 删除重复代码 |

### Phase 4: 集成测试 (0.5周)

| 任务 | 说明 |
|------|------|
| 4.1 | component-gallery 验证三端生成 |
| 4.2 | pac.at backend 配置测试 |
| 4.3 | 文档更新 |

## 9. 成功标准

1. **语法统一**: component-gallery 的 50 个组件 AURA 代码无需修改即可生成 Vue/Jet
2. **组件覆盖**: 第一阶段 10 个组件在 Jet 端完整可用
3. **工具链集成**: `auto build` 正确生成 `jet/` 目录
4. **样式一致性**: Tailwind 类名在三端渲染效果一致

## 10. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Tailwind → Modifier 转换不完整 | 样式差异 | 建立完整转换表，优先高频类名 |
| Material3 组件 API 变化 | 生成代码失效 | 锁定 Material3 版本 |
| Kotlin 语法更新 | 兼容性问题 | 目标 Kotlin 1.9+ |
| Android Studio 版本 | 项目结构变化 | 使用 Gradle CLI 优先 |

## 11. 后续阶段

- **Phase 5-8**: 扩展剩余 40 个组件
- **Phase 9**: iOS (SwiftUI) 生成器
- **Phase 10**: 增量编译支持
