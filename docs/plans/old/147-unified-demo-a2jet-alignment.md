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
| Phase 3 | 📋 待做 | 测试与验证 |
| Phase 4.1 | ✅ 完成 | 高优先级 Native Widgets (Progress, Image, Badge, Radio, ListItem) |
| Phase 4.2 | 📋 下一步 | 更多 Native Widgets |
| Phase 4.3 | 📋 计划中 | Overlay Widgets |
| Phase 5 | 📋 计划中 | Composite Widgets |
| Phase 6 | 📋 计划中 | 完整对齐 jet-gallery |

---

## 已完成内容

### Phase 4.1: 高优先级 Native Widgets ✅ (2025-03)

- [x] **Task 4.1.1: Progress 组件**
  - `Progress` → `CircularProgressIndicator` / `LinearProgressIndicator`
  - 支持 `type: "linear"`, `value`, `color` props
  - Demo: `progress.at`

- [x] **Task 4.1.2: Image 组件**
  - `Image` → `AsyncImage` (Coil)
  - 支持 `src`, `contentDescription` props
  - 添加 INTERNET 权限和 Coil 依赖
  - Demo: `image.at`

- [x] **Task 4.1.3: Badge 组件**
  - `Badge` → `Badge`
  - 支持 `count`, `variant: "dot"` props
  - Demo: `badge.at`

- [x] **Task 4.1.5: Radio 组件** (跳过 RadioGroup，直接支持单个 RadioButton)
  - `Radio` / `RadioButton` → `RadioButton`
  - 支持 `selected`, `text`, `disabled` props
  - Demo: `radio.at`

- [x] **Task 4.1.6: ListItem 组件**
  - `ListItem` → `ListItem`
  - 支持 `headline`, `supporting`, `leading`, `trailing` props
  - Demo: `listitem.at` (路由: `/list-item`)

**Bug 修复：**
- 修复 `HorizontalDivider` 不接受字符串参数
- 修复 `path_to_screen_name` 处理连字符路径 (`list-item` → `ListItemPage`)
- 使用 `DateRange` 图标替代 `Calendar`（不在默认图标集中）

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
| RadioGroup | 🔵 中 | Native | ✅ 完成 | RadioButton (单个) |
| Textarea | 🔵 中 | Native | ❌ 待做 | OutlinedTextField multi-line |
| Form | ⚪ 低 | Composite | ❌ 待做 | 高级模式 |

### 4.3 Display Section

| Widget | 优先级 | SupportTier | a2jet 状态 | 备注 |
|--------|--------|-------------|------------|------|
| Text | ✅ 已有 | Native | ✅ 完成 | Text + typography |
| Image | 🟡 高 | Native | ✅ 完成 | AsyncImage (Coil) |
| Badge | 🟡 高 | Native | ✅ 完成 | Badge |
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
| Progress | 🟡 高 | Native | ✅ 完成 | CircularProgressIndicator / LinearProgressIndicator |
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
| ListItem | 🔵 中 | Native | ✅ 完成 | ListItem with headline/supporting/leading/trailing |

---

## 下一步任务 (Phase 4.2): 更多 Native Widgets

### Task 4.2.1: Dialog/AlertDialog 组件 (推迟)

**优先级：** 🔴 复杂 (延后到最后)

---

### Task 4.2.2: Textarea 组件 ✅ 已完成

---

### Task 4.2.3: DropdownMenu 组件 (推迟)

**优先级：** 🔴 复杂 (延后到最后)

---

### Task 4.2.4: AspectRatio 组件

**目标：** 保持宽高比

**AURA 定义：**
```auto
AspectRatio (ratio: 16/9) {
    Image (src: "thumbnail.png")
}
```

**Kotlin 输出：**
```kotlin
Box(modifier = Modifier.aspectRatio(16f / 9f)) {
    Image(painter = ..., contentDescription = ...)
}
```

**优先级：** 🟢 简单

---

## Phase 4.3: Overlay Widgets (推迟)

- Sheet (ModalBottomSheet) - 🔴 复杂
- Tooltip (TooltipBox) - 🔴 复杂
- Drawer (NavigationDrawer) - 🔴 复杂
- Dialog/AlertDialog - 🔴 复杂
- DropdownMenu - 🔴 复杂

---

## Phase 5: Composite Widgets 📋

Composite widgets 需要更复杂的生成策略，但部分静态组件相对简单：

### 🟢 简单 Composite (静态，无复杂状态)
- **Table** - Column + Row + Divider 组合 (静态表格很简单！)
- **Alert** - Card + icon + text pattern
- **Avatar** - AsyncImage + CircleShape + fallback

### 🔴 复杂 Composite (需要状态管理)
- **Select** - ExposedDropdownMenu
- **Collapsible** - AnimatedVisibility
- **Accordion** - 多个 Collapsible
- **Sheet** - ModalBottomSheet
- **Tooltip** - TooltipBox

---

## Phase 6: 完整对齐 jet-gallery 📋

最终目标：
- 51 个 widget demos 全部覆盖
- 自适应布局 (手机/平板)
- NavigationBar 底部导航
- Master-Detail 布局

---

## Widget Demo 页面完整分析

### 已有 Demo 页面 (17个)
`badge`, `button`, `card`, `chip`, `column`, `counter`, `image`, `index`, `input`, `listitem`, `progress`, `radio`, `row`, `tabs`, `textarea`

### 缺失 Demo 页面分组

#### 🟢 第一组：简单 Native (已有 a2jet 支持，只需 demo)

| Widget | Compose 组件 | 说明 |
|--------|-------------|------|
| **center** | Box(contentAlignment) | 居中容器 |
| **separator** | HorizontalDivider | 分隔线 ✅ 已支持 |
| **checkbox** | Checkbox | 复选框 ✅ 已支持 |
| **switch** | Switch | 开关 ✅ 已支持 |
| **slider** | Slider | 滑块 ✅ 已支持 |
| **grid** | LazyVerticalGrid | 网格 ✅ 已支持 |
| **list** | LazyColumn | 列表 ✅ 已支持 |
| **text** | Text | 文本样式 |

**预计工作量：** 每个 15-30 分钟，总共 ~2 小时

#### 🟡 第二组：中等难度 (简单 Composite 或需少量状态)

| Widget | Compose 组件 | 说明 |
|--------|-------------|------|
| **Table** | Column + Row | **静态表格很简单！可优先做** |
| **avatar** | AsyncImage + CircleShape | 头像 |
| **aspectratio** | Modifier.aspectRatio | 宽高比 |
| **scrollarea** | verticalScroll | 滚动区域 |
| **alert** | Card + icon | 警告卡片 |

**预计工作量：** 每个 30-60 分钟，总共 ~3 小时

#### 🔴 第三组：复杂 (需要复杂状态管理，延后)

| Widget | 说明 |
|--------|------|
| **dropdownmenu** | 需要 expanded 状态 |
| **dialog/alertdialog** | 需要 open 状态 + 确认/取消逻辑 |
| **radiogroup** | 需要选中状态管理 |
| **sheet** | ModalBottomSheet 状态 |
| **tooltip** | TooltipBox 状态 |
| **drawer** | NavigationDrawer 状态 |
| **collapsible** | AnimatedVisibility 状态 |
| **accordion** | 多个 collapsible 状态 |
| **select** | ExposedDropdownMenu 状态 |
| **swiper** | HorizontalPager 状态 |

**预计工作量：** 每个 1-2 小时，延后处理

---

## 实现优先级总结

### ✅ 已完成 (Phase 4.1)
1. **Progress** - ✅ Native, circular/linear
2. **Image** - ✅ Native, AsyncImage (Coil)
3. **Badge** - ✅ Native, Badge
4. **Radio/RadioButton** - ✅ Native, 单个 RadioButton
5. **ListItem** - ✅ Native, headline/supporting/leading/trailing

### ✅ 已完成 (Phase 4.2)
6. **Textarea** - ✅ Native, multi-line OutlinedTextField

### 📋 下一步建议顺序

**第一批 (🟢 简单，快速完成):**
1. **Table** - 静态表格，可用于展示 API 列表 ⭐ 优先
2. **checkbox** - Demo page
3. **switch** - Demo page
4. **slider** - Demo page
5. **separator** - Demo page
6. **center** - Demo page
7. **grid** - Demo page
8. **list** - Demo page

**第二批 (🟡 中等):**
9. **avatar** - 头像组件
10. **aspectratio** - 宽高比
11. **alert** - 警告卡片

**第三批 (🔴 复杂，延后):**
12. dropdownmenu
13. dialog/alertdialog
14. 其他需要状态管理的组件

### ✅ 已完成 (Phase 4.2)
6. **Textarea** - ✅ Native, multi-line OutlinedTextField

### 📋 下一步建议顺序

**第一批 (🟢 简单，快速完成):**
1. **Table** - 静态表格，可用于展示 API 列表 ⭐ 优先
2. **checkbox** - Demo page
3. **switch** - Demo page
4. **slider** - Demo page
5. **separator** - Demo page
6. **center** - Demo page
7. **grid** - Demo page
8. **list** - Demo page

**第二批 (🟡 中等):**
9. **avatar** - 头像组件
10. **aspectratio** - 宽高比
11. **alert** - 警告卡片

**第三批 (🔴 复杂，延后):**
12. dropdownmenu
13. dialog/alertdialog
14. 其他需要状态管理的组件

---

## Success Criteria

### Phase 1-2 ✅
- [x] Card 支持 3 种变体
- [x] Chip 支持 4 种变体
- [x] FlowRow 正确处理自动换行
- [x] Tabs 正确管理状态和内容切换
- [x] unified-demo 新增 3 个 demo 页面

### Phase 4.1 ✅
- [x] Progress 组件支持 circular/linear, determinate/indeterminate
- [x] Image 组件支持网络图片 (AsyncImage/Coil)
- [x] Badge 组件支持数字
- [x] RadioButton 组件
- [x] ListItem 组件支持 headline/supporting/leading/trailing

### Phase 4.2 ✅
- [x] Textarea 组件 (multi-line)

### Phase 4.3 (下一步)
- [ ] Table 静态表格组件 ⭐ 优先
- [ ] checkbox/switch/slider demo pages
- [ ] separator/center demo pages
- [ ] grid/list demo pages

### Phase 3 (待做)
- [ ] 所有新组件有对应单元测试
- [ ] 生成的 Kotlin 代码可编译运行 (Android Studio)

---

## Related Plans

- Plan 113-118: a2jet (Jetpack Compose 代码生成)
- Plan 133: Jetpack Compose Generator Enhancement
- Plan 134: Jet Generator View Body
- Plan 140: AURA Widget Library
- Plan 145: jet-gallery Reference Project
