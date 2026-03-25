# Plan 147: unified-demo 与 a2jet 对齐改进

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 双轨改进 unified-demo 项目和 a2jet generator，使其逐步接近 jet-gallery 参考项目的质量和功能覆盖。

**Reference:** `examples/jet-gallery/` - 完整的 51 widget demos Android 项目

---

## 进度总览

| Phase | 状态 | 描述 |
|-------|------|------|
| Phase 1 | ✅ 完成 | 基础组件扩展 (Card, Chip, FlowRow, Tabs) |
| Phase 2 | ✅ 完成 | unified-demo 页面扩展 |
| Phase 3 | 🔄 进行中 | 测试与验证 |
| Phase 4 | 📋 计划中 | 更多 Widget Demos |
| Phase 5 | 📋 计划中 | 高级组件与 Composite widgets |
| Phase 6 | 📋 计划中 | 完整对齐 jet-gallery |

---

## 已完成内容

### Phase 1: 基础组件扩展 ✅

- [x] **Task 1.1: Card Variant 支持**
  - `variant: "elevated"` → `ElevatedCard`
  - `variant: "outlined"` → `OutlinedCard`
  - `variant: "filled"` (default) → `Card`

- [x] **Task 1.2: Chip 组件**
  - `Chip` / `variant: "assist"` → `AssistChip`
  - `variant: "filter"` → `FilterChip`
  - `variant: "input"` → `InputChip`
  - `variant: "suggestion"` → `SuggestionChip`

- [x] **Task 1.3: FlowRow 支持**
  - `FlowRow` → `FlowRow` with `ExperimentalLayoutApi`
  - 支持静态子元素和动态 `items` 数据源

- [x] **Task 1.4: Tabs 组件族**
  - `Tabs`, `TabRow`, `Tab`, `TabsContent`
  - 状态管理：`activeTab` with `mutableStateOf`
  - `when` 表达式生成内容切换

### Phase 2: unified-demo 页面扩展 ✅

- [x] **Task 2.1: 创建详情页通用组件**
  - `PanelCard.at` - 带标题的卡片容器
  - `MetaChip.at` - 圆角标签
  - `WidgetListCard.at` - 列表项卡片

- [x] **Task 2.2: 添加 Widget Demo Pages**
  - `card.at` - Card 变体演示
  - `tabs.at` - Tabs 组件演示
  - `chip.at` - Chip 变体演示

- [x] **Task 2.3: 更新 App 路由**
  - 添加 `/card`, `/tabs`, `/chip` 路由
  - 更新首页导航链接

---

## Phase 3: 测试与验证 🔄

### Task 3.1: a2jet 单元测试

**目标：** 为新组件添加测试用例

**修改文件：**
- `crates/auto-lang/src/ui_gen/jet/layout.rs` - Card variant 测试
- `crates/auto-lang/src/ui_gen/jet/form.rs` - Chip 测试
- `crates/auto-lang/src/ui_gen/jet/navigation.rs` - Tabs 测试

---

## Phase 4: 更多 Widget Demos 📋

基于 jet-gallery 的 51 个 widgets，按优先级规划：

### 4.1 Foundation Section (Layout 类别)

| Widget | 优先级 | SupportTier | a2jet 状态 | 备注 |
|--------|--------|-------------|------------|------|
| Col | ✅ 已有 | Native | ✅ 完成 | Column |
| Row | ✅ 已有 | Native | ✅ 完成 | Row |
| Center | 🟡 高 | Native | ✅ 完成 | Box + Center |
| Card | ✅ 已有 | Native | ✅ 完成 | 支持变体 |
| ScrollArea | 🟡 高 | Native | ✅ 完成 | verticalScroll |
| AspectRatio | 🔵 中 | Native | ❌ 待做 | Modifier.aspectRatio() |
| Collapsible | 🔵 中 | Composite | ❌ 待做 | AnimatedVisibility |
| Accordion | ⚪ 低 | Composite | ❌ 待做 | 多个 Collapsible |

### 4.2 Input Section (Form 类别)

| Widget | 优先级 | SupportTier | a2jet 状态 | 备注 |
|--------|--------|-------------|------------|------|
| Button | ✅ 已有 | Native | ✅ 完成 | Button variants |
| Input | ✅ 已有 | Native | ✅ 完成 | OutlinedTextField |
| Checkbox | 🟡 高 | Native | ✅ 完成 | Checkbox |
| Switch | 🟡 高 | Native | ✅ 完成 | Switch |
| Select | 🔵 中 | Composite | ❌ 待做 | ExposedDropdownMenu |
| Slider | 🟡 高 | Native | ✅ 完成 | Slider |
| RadioGroup | 🔵 中 | Native | ❌ 待做 | RadioButton list |
| Textarea | 🔵 中 | Native | ❌ 待做 | OutlinedTextField multi-line |
| Form | ⚪ 低 | Composite | ❌ 待做 | 高级模式 |

### 4.3 Display Section

| Widget | 优先级 | SupportTier | a2jet 状态 | 备注 |
|--------|--------|-------------|------------|------|
| Text | ✅ 已有 | Native | ✅ 完成 | Text + typography |
| Image | 🟡 高 | Native | ❌ 待做 | Image / AsyncImage |
| Badge | 🟡 高 | Native | ❌ 待做 | Badge / BadgedBox |
| Avatar | 🔵 中 | Composite | ❌ 待做 | AsyncImage + CircleShape |
| Separator | 🟡 高 | Native | ✅ 完成 | HorizontalDivider |
| Skeleton | ⚪ 低 | Composite | ❌ 待做 | 加载占位符 |
| Swiper | ⚪ 低 | Composite | ❌ 待做 | HorizontalPager |

### 4.4 Navigation Section

| Widget | 优先级 | SupportTier | a2jet 状态 | 备注 |
|--------|--------|-------------|------------|------|
| Tabs | ✅ 已有 | Native | ✅ 完成 | TabRow + Tab |
| Breadcrumb | ⚪ 低 | Composite | ❌ 待做 | 路径导航 |
| NavigationMenu | ⚪ 低 | Composite | ❌ 待做 | 导航模式 |
| Pagination | ⚪ 低 | Composite | ❌ 待做 | 分页控件 |
| Sidebar | ⚪ 低 | Composite | ❌ 待做 | NavigationRail |
| MenuBar | ⚪ 低 | Composite | ❌ 待做 | 桌面菜单栏 |
| DropdownMenu | 🔵 中 | Native | ❌ 待做 | DropdownMenu |
| NavLink | 🟡 高 | Composite | ✅ 部分完成 | Link 组件 |

### 4.5 Overlay Section

| Widget | 优先级 | SupportTier | a2jet 状态 | 备注 |
|--------|--------|-------------|------------|------|
| Dialog | 🟡 高 | Native | ❌ 待做 | AlertDialog / Dialog |
| AlertDialog | 🟡 高 | Native | ❌ 待做 | AlertDialog |
| Sheet | 🔵 中 | Composite | ❌ 待做 | ModalBottomSheet |
| Drawer | ⚪ 低 | Composite | ❌ 待做 | NavigationDrawer |
| Popover | ⚪ 低 | Composite | ❌ 待做 | Popup |
| Tooltip | 🔵 中 | Composite | ❌ 待做 | TooltipBox |
| HoverCard | ⚪ 低 | Composite | ❌ 待做 | 预览卡片 |
| ContextMenu | ⚪ 低 | Composite | ❌ 待做 | 右键菜单 |

### 4.6 Feedback Section

| Widget | 优先级 | SupportTier | a2jet 状态 | 备注 |
|--------|--------|-------------|------------|------|
| Alert | 🔵 中 | Composite | ❌ 待做 | Card + icon |
| Toast | 🔵 中 | Composite | ❌ 待做 | Android Toast |
| Progress | 🟡 高 | Native | ❌ 待做 | LinearProgressIndicator |
| Sonner | ⚪ 低 | Composite | ❌ 待做 | Snackbar |

### 4.7 Data Section

| Widget | 优先级 | SupportTier | a2jet 状态 | 备注 |
|--------|--------|-------------|------------|------|
| Table | ⚪ 低 | Composite | ❌ 待做 | 自定义表格 |
| DataTable | ⚪ 低 | Composite | ❌ 待做 | 高级表格 |
| Calendar | ⚪ 低 | Composite | ❌ 待做 | DatePicker |
| Grid | 🟡 高 | Native | ✅ 完成 | LazyVerticalGrid |
| GridItem | 🟡 高 | Native | ✅ 完成 | Grid cell |
| List | ✅ 已有 | Native | ✅ 完成 | LazyColumn |
| ListItem | 🔵 中 | Native | ❌ 待做 | Material ListItem |

---

## 下一步任务 (Phase 4.1): 高优先级 Native Widgets

### Task 4.1.1: Progress 组件

**目标：** 添加进度指示器支持

**AURA 定义：**
```auto
Progress {}                           // indeterminate circular
Progress (type: "linear") {}          // indeterminate linear
Progress (value: 0.7) {}              // determinate circular (70%)
Progress (type: "linear", value: 0.5) {} // determinate linear
```

**Kotlin 输出：**
```kotlin
// Circular indeterminate
CircularProgressIndicator()

// Linear indeterminate
LinearProgressIndicator()

// Circular determinate
CircularProgressIndicator(progress = 0.7f)

// Linear determinate
LinearProgressIndicator(progress = 0.5f)
```

**修改文件：**
- `crates/auto-lang/src/ui_gen/jet/form.rs` 或新建 `feedback.rs`

---

### Task 4.1.2: Image 组件

**目标：** 添加图片组件支持

**AURA 定义：**
```auto
Image (src: "https://example.com/image.png")
Image (src: .avatarUrl, contentDescription: "User avatar")
```

**Kotlin 输出：**
```kotlin
Image(
    painter = rememberAsyncImagePainter(model = "https://example.com/image.png"),
    contentDescription = "Image",
    modifier = Modifier
)
```

**修改文件：**
- `crates/auto-lang/src/ui_gen/jet/display.rs` (新建)
- 添加 Coil `AsyncImage` 支持

---

### Task 4.1.3: Badge 组件

**目标：** 添加徽章组件支持

**AURA 定义：**
```auto
Badge (count: 5) {}
Badge (variant: "dot") {}  // 小圆点
BadgedBox (badge: { Badge (count: 3) }) {
    Icon "notifications"
}
```

**Kotlin 输出：**
```kotlin
Badge { Text("5") }

BadgedBox(
    badge = { Badge { Text("3") } }
) {
    Icon(Icons.Default.Notifications, contentDescription = null)
}
```

---

### Task 4.1.4: Dialog 组件

**目标：** 添加对话框支持

**AURA 定义：**
```auto
Dialog (open: .showDialog, onDismiss: .CloseDialog) {
    Card {
        Col {
            H2 "Confirm Action"
            Text "Are you sure?"
            Row {
                Button (variant: "text", click: .CloseDialog) "Cancel"
                Button (click: .Confirm) "Confirm"
            }
        }
    }
}
```

**Kotlin 输出：**
```kotlin
if (showDialog) {
    AlertDialog(
        onDismissRequest = { showDialog = false },
        confirmButton = {
            Button(onClick = { /* confirm */ }) { Text("Confirm") }
        },
        dismissButton = {
            TextButton(onClick = { showDialog = false }) { Text("Cancel") }
        },
        title = { Text("Confirm Action") },
        text = { Text("Are you sure?") }
    )
}
```

---

### Task 4.1.5: RadioGroup 组件

**目标：** 添加单选按钮组支持

**AURA 定义：**
```auto
RadioGroup (selected: .selectedOption, onChange: .SelectOption) {
    RadioButton (value: "option1") "Option 1"
    RadioButton (value: "option2") "Option 2"
    RadioButton (value: "option3") "Option 3"
}
```

**Kotlin 输出：**
```kotlin
Column {
    Row(verticalAlignment = Alignment.CenterVertically) {
        RadioButton(
            selected = selectedOption == "option1",
            onClick = { selectedOption = "option1" }
        )
        Text("Option 1")
    }
    // ...
}
```

---

### Task 4.1.6: ListItem 组件

**目标：** 添加 Material3 ListItem 支持

**AURA 定义：**
```auto
ListItem {
    headline: "Primary text"
    supporting: "Secondary text"
    leading: Icon "person"
    trailing: Icon "chevron_right"
}
```

**Kotlin 输出：**
```kotlin
ListItem(
    headlineContent = { Text("Primary text") },
    supportingContent = { Text("Secondary text") },
    leadingContent = { Icon(Icons.Default.Person, null) },
    trailingContent = { Icon(Icons.Default.ChevronRight, null) }
)
```

---

## Phase 5: Composite Widgets 📋

Composite widgets 需要更复杂的生成策略：

### 高优先级 Composite
- **Avatar** - AsyncImage + CircleShape + fallback
- **Sheet** - ModalBottomSheet
- **Tooltip** - TooltipBox

### 中优先级 Composite
- **Select** - ExposedDropdownMenu
- **Alert** - Card + icon + text pattern
- **Collapsible** - AnimatedVisibility

---

## Phase 6: 完整对齐 jet-gallery 📋

最终目标：
- 51 个 widget demos 全部覆盖
- 自适应布局 (手机/平板)
- NavigationBar 底部导航
- Master-Detail 布局

---

## 实现优先级总结

### 立即执行 (Phase 4.1)
1. **Progress** - Native, 高频使用
2. **Image** - Native, 基础组件
3. **Badge** - Native, 常用
4. **Dialog/AlertDialog** - Native, 核心 UI
5. **RadioGroup** - Native, 表单必需
6. **ListItem** - Native, 列表基础

### 短期规划 (Phase 4.2-4.3)
7. **Textarea** - 表单扩展
8. **Select** - 表单扩展 (Composite)
9. **DropdownMenu** - 导航扩展
10. **Avatar** - 显示扩展 (Composite)

### 中期规划 (Phase 5)
11. **Sheet/BottomSheet** - Overlay
12. **Tooltip** - Overlay
13. **Collapsible/Accordion** - Layout
14. **Alert/Toast** - Feedback

### 长期规划 (Phase 6)
15. NavigationBar + Section 导航
16. Master-Detail 布局
17. 完整 51 widget 覆盖

---

## Success Criteria

### Phase 1-2 ✅
- [x] Card 支持 3 种变体
- [x] Chip 支持 4 种变体
- [x] FlowRow 正确处理自动换行
- [x] Tabs 正确管理状态和内容切换
- [x] unified-demo 新增 3 个 demo 页面

### Phase 3 (进行中)
- [ ] 所有新组件有对应单元测试
- [ ] 生成的 Kotlin 代码可编译运行

### Phase 4 (下一步)
- [ ] Progress 组件支持 circular/linear, determinate/indeterminate
- [ ] Image 组件支持本地和网络图片
- [ ] Badge 组件支持数字和小圆点
- [ ] Dialog/AlertDialog 组件
- [ ] RadioGroup 组件
- [ ] ListItem 组件

---

## Related Plans

- Plan 113-118: a2jet (Jetpack Compose 代码生成)
- Plan 133: Jetpack Compose Generator Enhancement
- Plan 134: Jet Generator View Body
- Plan 140: AURA Widget Library
- Plan 145: jet-gallery Reference Project
