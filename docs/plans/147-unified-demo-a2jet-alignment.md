# Plan 147: unified-demo 与 a2jet 对齐改进

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 双轨改进 unified-demo 项目和 a2jet generator，使其逐步接近 jet-gallery 参考项目的质量和功能覆盖。

**Reference:** `examples/jet-gallery/` - 完整的 51 widget demos Android 项目

---

## 背景分析

### jet-gallery 参考项目

jet-gallery 是一个完整的 Jetpack Compose 项目，包含：

- **51 个 widget demos**，分为 7 个类别
- **详情页 UI 组件**：
  - `PanelCard` - 带标题的卡片容器（基于 ElevatedCard）
  - `MetaChip` - 圆角标签（基于 Surface）
  - `FlowTagRow` - 自动换行的标签行（基于 FlowRow）
  - `WidgetListCard` - 列表项卡片（基于 OutlinedCard）
- **自适应布局**：手机（底部导航 + 列表详情切换）、平板（侧边 Rail + 主从布局）
- **Tabs 导航**：详情页内的 Tab 切换

### unified-demo 当前状态

- 仅 5 个基础页面：index, counter, column, row, button, input
- 简单的 AURA widget 定义
- 缺少详情页所需的通用组件

### a2jet generator 当前状态

**已支持：**
- Layout: Col → Column, Row → Row, Box → Box, Card → Card, Scroll → Column + verticalScroll
- Form: Button, Input (OutlinedTextField), Checkbox, Switch, Slider
- Text: Text, H1-H6
- List: LazyColumn, LazyRow, LazyVerticalGrid

**缺失（详情页必需）：**
- Card 变体：ElevatedCard, OutlinedCard
- Chip 组件：AssistChip, FilterChip 等
- FlowRow 布局
- Tabs 组件族：TabRow, Tab, TabsContent

---

## Phase 1: 基础组件扩展

### Task 1.1: Card Variant 支持

**目标：** 支持 `variant` prop 生成不同类型的 Card

**AURA 定义：**
```auto
Card (variant: "elevated") { ... }   // → ElevatedCard
Card (variant: "outlined") { ... }   // → OutlinedCard
Card (variant: "filled") { ... }     // → Card (default)
```

**Kotlin 输出：**
```kotlin
// variant: "elevated"
ElevatedCard(modifier = ...) { ... }

// variant: "outlined"
OutlinedCard(modifier = ...) { ... }

// variant: "filled" (default)
Card(modifier = ...) { ... }
```

**修改文件：**
- `crates/auto-lang/src/ui_gen/jet/layout.rs`
  - 修改 `generate_card()` 函数
  - 添加 variant prop 解析
  - 根据 variant 选择不同的 Compose 组件

**测试：**
- 添加 `test_card_variants()` 测试用例

---

### Task 1.2: Chip 组件

**目标：** 添加 Chip 组件支持（区别于 Badge）

**背景：** jet-gallery 的 `MetaChip` 使用 Surface + 圆角实现，但 Material3 有专门的 Chip 组件。

**AURA 定义：**
```auto
Chip "Filter" {}
Chip (variant: "assist", icon: "add") "Add Item" {}
Chip (variant: "filter", selected: .active == "filter") "Filter" {}
Chip (variant: "input", onDismiss: .RemoveChip) "Tag" {}
```

**Kotlin 输出：**
```kotlin
// variant: "assist" (default)
AssistChip(onClick = {}, label = { Text("Add Item") }, leadingIcon = { Icon(...) })

// variant: "filter"
FilterChip(selected = active == "filter", onClick = { ... }, label = { Text("Filter") })

// variant: "input"
InputChip(selected = false, onDismissRequest = { ... }, label = { Text("Tag") })

// variant: "suggestion"
SuggestionChip(onClick = {}, label = { Text("Suggestion") })
```

**新建/修改文件：**
- `crates/auto-lang/src/ui_gen/jet/form.rs` 或新建 `display.rs`
  - 添加 `generate_chip()` 函数
  - 支持 4 种 Chip 变体
- `crates/auto-lang/src/ui_gen/jet/generator.rs`
  - 添加 `is_display_tag("chip")` 检查
  - 添加 `display_element_to_compose()` 分发

**测试：**
- 添加 `test_chip_assist()`, `test_chip_filter()`, `test_chip_input()` 测试用例

---

### Task 1.3: FlowRow 支持

**目标：** 支持 FlowRow 布局（自动换行的 Row）

**AURA 定义：**
```auto
FlowRow {
    style: "gap-2 flex-wrap"
    Chip "Tag 1" {}
    Chip "Tag 2" {}
    Chip "Tag 3" {}
    // ... 自动换行
}
```

**Kotlin 输出：**
```kotlin
@OptIn(ExperimentalLayoutApi::class)
FlowRow(
    horizontalArrangement = Arrangement.spacedBy(8.dp),
    verticalArrangement = Arrangement.spacedBy(8.dp),
) {
    Chip(...)
    Chip(...)
    Chip(...)
}
```

**修改文件：**
- `crates/auto-lang/src/ui_gen/jet/layout.rs`
  - 添加 `generate_flow_row()` 函数
  - 添加 `ExperimentalLayoutApi` opt-in 注解
- `crates/auto-lang/src/ui_gen/jet/generator.rs`
  - 添加 `is_layout_tag("flow-row" | "flowrow")` 检查

**测试：**
- 添加 `test_flow_row_basic()`, `test_flow_row_with_gap()` 测试用例

---

### Task 1.4: Tabs 组件族

**目标：** 添加完整的 Tabs 组件支持

**AURA 定义：**
```auto
Tabs (activeTab: .activeTab) {
    TabsList {
        TabsTrigger preview (label: "Preview", active: .activeTab == "preview") {}
        TabsTrigger code (label: "Code", active: .activeTab == "code") {}
        TabsTrigger notes (label: "Notes", active: .activeTab == "notes") {}
    }
    TabsContent preview (active: .activeTab == "preview") {
        Text "Preview content"
    }
    TabsContent code (active: .activeTab == "code") {
        Text "Code content"
    }
    TabsContent notes (active: .activeTab == "notes") {
        Text "Notes content"
    }
}
```

**Kotlin 输出：**
```kotlin
var activeTab by remember { mutableStateOf(0) }
val tabs = listOf("Preview", "Code", "Notes")

Column {
    TabRow(selectedTabIndex = activeTab) {
        tabs.forEachIndexed { index, title ->
            Tab(
                selected = activeTab == index,
                onClick = { activeTab = index },
                text = { Text(title) }
            )
        }
    }
    when (activeTab) {
        0 -> { /* Preview content */ }
        1 -> { /* Code content */ }
        2 -> { /* Notes content */ }
    }
}
```

**新建/修改文件：**
- `crates/auto-lang/src/ui_gen/jet/navigation.rs`
  - 添加 `generate_tabs()` 函数
  - 添加 `generate_tab_row()` 函数
  - 添加 `generate_tab_content()` 函数
  - 处理状态管理和条件渲染
- `crates/auto-lang/src/ui_gen/jet/generator.rs`
  - 添加 `is_navigation_tag("tabs" | "tabrow")` 检查

**状态管理策略：**
- `activeTab` 使用 `mutableStateOf<Int>` (索引) 或 `mutableStateOf<String>` (ID)
- `TabsTrigger` 的 `onClick` 更新 `activeTab`
- `TabsContent` 根据 `active` 条件渲染

**测试：**
- 添加 `test_tabs_basic()`, `test_tabs_with_state()` 测试用例

---

## Phase 2: unified-demo 页面扩展

### Task 2.1: 创建详情页通用组件

**目标：** 创建可复用的详情页 UI 组件

**新建文件：**
- `examples/unified-demo/front/components/PanelCard.at`
- `examples/unified-demo/front/components/MetaChip.at`
- `examples/unified-demo/front/components/WidgetListCard.at`

**PanelCard.at：**
```auto
/// Panel card container for widget detail sections.
widget PanelCard {
    model {
        title str = ""
    }

    view {
        Card (variant: "elevated", style: "rounded-3xl") {
            Col (style: "w-full p-5 gap-3") {
                H3 .title
                // children via slot
            }
        }
    }
}
```

**MetaChip.at：**
```auto
/// Small rounded chip for tags and labels.
widget MetaChip {
    model {
        #[primary]
        text str = ""
        variant str = "default"  // default, primary, secondary
    }

    computed {
        chipStyle => f"px-3 py-1.5 rounded-full text-sm ${.variantStyle}"
    }

    view {
        Surface (style: .chipStyle) {
            Text .text
        }
    }
}
```

**WidgetListCard.at：**
```auto
/// Card for displaying widget info in a list.
widget WidgetListCard {
    model {
        title str = ""
        description str = ""
        supportTier str = ""
        selected bool = false
    }

    view {
        Card (variant: "outlined", style: .cardStyle) {
            Col (style: "w-full p-4 gap-2") {
                Row (style: "w-full justify-between items-center") {
                    H4 .title
                    MetaChip .supportTier
                }
                Text (variant: "muted") .description
            }
        }
    }
}
```

---

### Task 2.2: 添加第一批 Widget Demo Pages

**目标：** 添加 Card, Tabs, Chip 的 demo 页面

**新建文件：**
- `examples/unified-demo/front/pages/card.at`
- `examples/unified-demo/front/pages/tabs.at`
- `examples/unified-demo/front/pages/chip.at`

**更新文件：**
- `examples/unified-demo/front/app.at` - 添加新路由

**card.at 示例：**
```auto
widget CardPage {
    view {
        Col (style: "p-8 gap-8") {
            // Header
            Col (style: "items-center") {
                H1 "Card Demo"
                Text (variant: "muted") "Container with multiple variants"
            }

            // Variants
            PanelCard (title: "Variants") {
                Col (style: "gap-4") {
                    Card (variant: "filled") { Text "Filled Card" }
                    Card (variant: "elevated") { Text "Elevated Card" }
                    Card (variant: "outlined") { Text "Outlined Card" }
                }
            }

            // Usage
            PanelCard (title: "Usage in Details") {
                Text "PanelCard uses ElevatedCard variant for sections"
            }
        }
    }
}
```

---

### Task 2.3: 更新 App 路由

**更新 `app.at`：**
```auto
widget App {
    routes {
        "/" => use index
        "/counter" => use counter
        "/column" => use column
        "/row" => use row
        "/button" => use button
        "/input" => use input
        "/card" => use card       // 新增
        "/tabs" => use tabs       // 新增
        "/chip" => use chip       // 新增
    }

    view {
        Col {
            style: "w-full h-full"
            Col (style: "p-4 bg-blue-500") {
                H1 "Auto UI Demo"
            }
            outlet
        }
    }
}
```

---

## Phase 3: 测试与验证

### Task 3.1: a2jet 单元测试

**目标：** 为新组件添加测试用例

**修改文件：**
- `crates/auto-lang/src/ui_gen/jet/layout.rs` - 添加 Card variant 测试
- `crates/auto-lang/src/ui_gen/jet/form.rs` - 添加 Chip 测试
- `crates/auto-lang/src/ui_gen/jet/navigation.rs` - 添加 Tabs 测试

**测试用例：**
```rust
#[test]
fn test_card_variant_elevated() {
    let mut gen = LayoutGenerator::new();
    let mut props = HashMap::new();
    props.insert("variant".to_string(), AuraPropValue::Expr(AuraExpr::Literal("elevated".to_string())));

    let result = gen.generate_card(&props, "Text(\"Content\")");
    assert!(result.unwrap().contains("ElevatedCard"));
}

#[test]
fn test_card_variant_outlined() {
    // ...
}

#[test]
fn test_chip_assist() {
    // ...
}

#[test]
fn test_flow_row() {
    // ...
}

#[test]
fn test_tabs_generation() {
    // ...
}
```

---

### Task 3.2: 端到端验证

**目标：** 验证生成的代码与 jet-gallery 对比

**步骤：**
1. 运行 `auto build` 生成 unified-demo 的 Jet 代码
2. 检查生成的 `card.kt`, `tabs.kt`, `chip.kt`
3. 与 jet-gallery 中对应组件对比
4. 确保样式、状态管理、交互行为一致

**验收标准：**
- [ ] Card 支持 3 种变体（filled, elevated, outlined）
- [ ] Chip 支持 4 种变体（assist, filter, input, suggestion）
- [ ] FlowRow 正确处理自动换行
- [ ] Tabs 正确管理状态和内容切换
- [ ] unified-demo 新增 3 个 demo 页面可正常编译运行

---

## 关键设计决策

### 1. Card 变体映射

| AURA variant | Compose Component | 特点 |
|--------------|-------------------|------|
| `"filled"` (default) | `Card` | 平面卡片 |
| `"elevated"` | `ElevatedCard` | 带阴影 |
| `"outlined"` | `OutlinedCard` | 边框无阴影 |

### 2. Chip vs Badge 区分

| 组件 | 用途 | Compose 对应 |
|------|------|-------------|
| **Badge** | 通知数字、状态点 | `Badge`, `BadgedBox` |
| **Chip** | 可交互标签、筛选器 | `AssistChip`, `FilterChip`, `InputChip`, `SuggestionChip` |

jet-gallery 的 `MetaChip` 本质是简化版 Chip，使用 `Surface` + 圆角实现。

### 3. Tabs 状态管理

两种策略：
1. **索引模式**：`activeTab: Int` - 简单，适合静态 tabs
2. **ID 模式**：`activeTab: String` - 灵活，适合动态 tabs

推荐使用 **索引模式**，与 Compose `TabRow` API 一致。

### 4. FlowRow Opt-in

FlowRow 需要 `@OptIn(ExperimentalLayoutApi::class)` 注解。
生成器应在文件顶部添加，或在使用处添加。

---

## 实现顺序

```
Phase 1: 基础组件
├── 1.1 Card variants     (修改 layout.rs)
├── 1.2 Chip 组件         (修改 form.rs)
├── 1.3 FlowRow           (修改 layout.rs)
└── 1.4 Tabs 组件族       (修改 navigation.rs)

Phase 2: 页面扩展
├── 2.1 通用组件          (PanelCard, MetaChip, WidgetListCard)
├── 2.2 Demo 页面         (card.at, tabs.at, chip.at)
└── 2.3 路由更新          (app.at)

Phase 3: 测试验证
├── 3.1 单元测试          (layout, form, navigation tests)
└── 3.2 端到端验证        (auto build → 对比 jet-gallery)
```

---

## Success Criteria

1. **Card 变体**：`variant: "elevated" | "outlined" | "filled"` 正确生成对应组件
2. **Chip 组件**：支持 4 种 Material3 Chip 变体
3. **FlowRow**：支持 `flex-wrap` 样式自动换行
4. **Tabs**：支持 `Tabs`, `TabsList`, `TabsTrigger`, `TabsContent` 完整组件族
5. **unified-demo**：新增 3+ demo 页面，与 jet-gallery 功能对齐
6. **测试覆盖**：所有新组件有对应单元测试
7. **编译通过**：`auto build` 生成的 Kotlin 代码可编译运行

---

## Related Plans

- Plan 113-118: a2jet (Jetpack Compose 代码生成)
- Plan 133: Jetpack Compose Generator Enhancement
- Plan 134: Jet Generator View Body
- Plan 140: AURA Widget Library
- Plan 145: jet-gallery Reference Project
