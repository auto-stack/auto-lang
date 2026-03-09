# Plan 115: a2jet Phase 4 - Layout & Navigation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现布局组件生成和导航支持。

**Architecture:** 在 Phase 1-3 基础上，添加 `LayoutGenerator` 模块处理布局组件和导航路由。

**Tech Stack:** Rust, Kotlin, Jetpack Compose, Material3

---

## 1. 布局组件映射

### 1.1 AURA → Compose 布局

| AURA Tag | Compose Component | 说明 |
|----------|-------------------|------|
| `col` | `Column` | 垂直布局 |
| `row` | `Row` | 水平布局 |
| `box` | `Box` | 叠加布局 |
| `card` | `Card` | Material3 卡片 |
| `scroll` | `Column + verticalScroll` | 可滚动容器 |
| `grid` | `LazyVerticalGrid` | 网格布局 |
| `container` | `Box` | 通用容器 |

### 1.2 布局属性映射

| AURA 属性 | Compose 属性 | 说明 |
|-----------|-------------|------|
| `gap: N` | `Arrangement.spacedBy(N.dp)` | 间距 |
| `align: "center"` | `Alignment.Center` | 对齐 |
| `justify: "between"` | `Arrangement.SpaceBetween` | 分布 |
| `padding: N` | `Modifier.padding(N.dp)` | 内边距 |
| `class: "..."` | Modifier DSL | Tailwind 类 |

---

## 2. 导航组件

### 2.1 路由定义

**AURA:**
```auto
widget App {
    routes: {
        home: HomeScreen,
        settings: SettingsScreen,
    }
}
```

**Kotlin:**
```kotlin
@Composable
fun AppNavHost(
    navController: NavHostController,
    modifier: Modifier = Modifier
) {
    NavHost(
        navController = navController,
        startDestination = "home",
        modifier = modifier
    ) {
        composable("home") { HomeScreen(navController) }
        composable("settings") { SettingsScreen(navController) }
    }
}
```

### 2.2 导航调用

**AURA:**
```auto
button { text: "Go to Settings", onclick: .Navigate("settings") }
```

**Kotlin:**
```kotlin
Button(onClick = { navController.navigate("settings") }) {
    Text("Go to Settings")
}
```

---

## 3. 文件结构

```
crates/auto-lang/src/ui_gen/jet/
├── mod.rs              # 更新导出
├── generator.rs        # 添加布局生成方法
├── layout.rs           # 新增：布局组件生成器
└── navigation.rs       # 新增：导航路由生成器
```

---

## 4. 实现任务

| Task | 内容 | 预计时间 |
|------|------|---------|
| Task 1 | 创建 layout.rs 模块 | 15 min |
| Task 2 | 实现 Column/Row/Box 生成 | 30 min |
| Task 3 | 实现 Card/Scroll 生成 | 20 min |
| Task 4 | 创建 navigation.rs 模块 | 15 min |
| Task 5 | 实现路由生成 | 25 min |
| Task 6 | 集成到 JetGenerator | 20 min |
| Task 7 | 添加单元测试 | 20 min |
| Task 8 | 最终验证和提交 | 10 min |

---

## 5. 生成代码示例

### 5.1 Column with gap

**AURA:**
```auto
col {
    gap: 8,
    class: "px-4 py-2",

    text { text: "Hello" }
    button { text: "Click" }
}
```

**Kotlin:**
```kotlin
Column(
    modifier = Modifier
        .padding(horizontal = 16.dp)
        .padding(vertical = 8.dp),
    verticalArrangement = Arrangement.spacedBy(8.dp)
) {
    Text("Hello")
    Button(onClick = { }) {
        Text("Click")
    }
}
```

### 5.2 Row with alignment

**AURA:**
```auto
row {
    align: "center",
    justify: "between",
    class: "w-full",

    text { text: "Left" }
    text { text: "Right" }
}
```

**Kotlin:**
```kotlin
Row(
    modifier = Modifier.fillMaxWidth(),
    verticalAlignment = Alignment.CenterVertically,
    horizontalArrangement = Arrangement.SpaceBetween
) {
    Text("Left")
    Text("Right")
}
```

### 5.3 Card

**AURA:**
```auto
card {
    class: "rounded-lg p-4",

    text { text: "Card content" }
}
```

**Kotlin:**
```kotlin
Card(
    modifier = Modifier
        .rounded(8.dp)
        .padding(16.dp)
) {
    Text("Card content")
}
```

---

## 6. 成功标准

- [ ] Column 组件支持 gap, align, class 属性
- [ ] Row 组件支持 gap, align, justify, class 属性
- [ ] Box 组件支持 align, class 属性
- [ ] Card 组件支持 class 属性
- [ ] 导航支持路由定义和 navigate 调用
- [ ] 单元测试覆盖率 > 80%
- [ ] 所有测试通过
