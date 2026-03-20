# Plan 136: Jet 后端增量适配

> **Status:** ✅ **COMPLETED** (2025-03-20)

**Goal:** 在 `unified-demo` 项目中逐步扩展 Jet 后端支持，最终能展示核心组件的 demo 页面。采用"从小到大，逐步扩展"的策略。

**Tech Stack:** Rust, Kotlin, Jetpack Compose, Material3, NavHost

---

## Implementation Summary

**Completed Tasks:**
1. ✅ 创建 `pages/` 和 `components/` 目录结构
2. ✅ 拆分 Counter 组件到 `components/counter.at`
3. ✅ 创建首页 `pages/index.at`（带导航链接）
4. ✅ 创建 Column demo 页面 `pages/column.at`
5. ✅ 创建 Row demo 页面 `pages/row.at`
6. ✅ 改造 `app.at` 支持 routes
7. ✅ 实现 Jet Generator 的 routes 处理（NavHost 生成）
8. ✅ 实现 Link 组件转换（navController.navigate）
9. ✅ **Card 组件支持**（Material3 Card）
10. ✅ **Tailwind class 样式支持**（与 Vue/Tauri 一致）
11. ✅ **Grid 静态子元素支持**（FlowRow 响应式布局）
12. ✅ **Link 支持元素子节点**（Card 包裹在 Link 中）

**Files Created:**
- `examples/unified-demo/front/pages/index.at`
- `examples/unified-demo/front/pages/counter.at`
- `examples/unified-demo/front/pages/column.at`
- `examples/unified-demo/front/pages/row.at`
- `examples/unified-demo/front/components/counter.at`
- `examples/unified-demo/front/components/DemoCard.at`

**Files Modified:**
- `examples/unified-demo/front/app.at` - 添加 routes 块
- `crates/auto-lang/src/ui_gen/jet/generator.rs` - routes, Link, Card 处理
- `crates/auto-lang/src/ui_gen/jet/layout.rs` - Card 组件, class_to_modifier 修复
- `crates/auto-lang/src/ui_gen/jet/list.rs` - 静态 Grid (FlowRow) 支持
- `crates/auto-lang/src/ui_gen/jet/modifier.rs` - rounded → clip(RoundedCornerShape)

---

## Tailwind Class 样式支持 (2025-03-20 更新)

### 概述

为了与 Vue/Tauri 后端保持一致的样式体验，Jet 后端现在支持 Tailwind class 属性到 Compose Modifier 的转换。

### 实现方式

**TextStyle vs Modifier 分离：**
- 字体样式（fontSize, fontWeight, color）→ `TextStyle(...)`
- 布局样式（padding, background, rounded）→ `Modifier.xxx()`

**关键代码位置：**
- `modifier.rs` 第 43-60 行：`dimension_to_dp()` 转换
- `modifier.rs` 第 219-226 行：`rounded` → `clip(RoundedCornerShape)`
- `generator.rs` 第 977-987 行：添加样式相关 imports
- `generator.rs` 第 1006-1032 行：TextStyle 生成

### Tailwind → Compose 映射表

| Tailwind Class | Compose Modifier |
|----------------|------------------|
| `p-4`, `px-4`, `py-4` | `padding(16.dp)`, `padding(horizontal = 16.dp)` |
| `gap-4` | `Arrangement.spacedBy(16.dp)` |
| `rounded-lg` | `clip(RoundedCornerShape(8.dp))` |
| `rounded-full` | `clip(CircleShape)` |
| `bg-blue-500` | `background(Color(0xFF3B82F6))` |
| `text-lg` | `TextStyle(fontSize = 18.sp)` |
| `font-bold` | `TextStyle(fontWeight = FontWeight.Bold)` |
| `text-gray-600` | `TextStyle(color = Color(0x4B5563))` |

### 单位换算

Tailwind 单位 → Compose Dp：`1 Tailwind unit = 4 dp`

```kotlin
// p-4 → padding(16.dp)  (4 * 4 = 16)
// gap-2 → spacedBy(8.dp)  (2 * 4 = 8)
// rounded-lg → RoundedCornerShape(8.dp)  (lg = 8 Tailwind units)
```

---

## Card 组件支持 (2025-03-20 更新)

### 实现方式

Card 组件映射到 Material3 的 `Card` composable：

```kotlin
Card(
    modifier = Modifier.padding(16.dp).clip(RoundedCornerShape(8.dp))
) {
    // children
}
```

### AURA 语法

```auto
card(class: "p-4 rounded-lg hover:shadow-lg") {
    col(class: "gap-2") {
        h3(text: "Title", class: "text-lg font-semibold")
        text(text: "Description", class: "text-sm text-gray-600")
    }
}
```

### 代码位置

- `layout.rs` 第 323-353 行：`generate_card()` 方法

---

## Link 支持元素子节点 (2025-03-20 更新)

### 实现方式

当 Link 包含非文本子元素（如 Card）时，生成 `Box` + `clickable`：

```kotlin
Box(
    modifier = Modifier.clickable { navController.navigate("/counter") }
) {
    Card(...) { ... }
}
```

### AURA 语法

```auto
link (to: "/counter") {
    card(class: "p-4 rounded-lg") {
        text("Counter Demo")
    }
}
```

### 代码位置

- `generator.rs` 第 846-904 行：`link_to_compose()` 方法

---

## 静态 Grid 支持 (2025-03-20 更新)

### 实现方式

当 `grid` 没有动态数据源（`items` prop）但有静态子元素时，使用 `FlowRow` 生成响应式布局：

```kotlin
FlowRow(
    modifier = Modifier,
    horizontalArrangement = Arrangement.spacedBy(16.dp),
    verticalArrangement = Arrangement.spacedBy(16.dp)
) {
    // 静态子元素（如 Card）
}
```

### AURA 语法

```auto
grid(class: "gap-4") {
    link (to: "/counter") { card { ... } }
    link (to: "/column") { card { ... } }
    link (to: "/row") { card { ... } }
}
```

### 代码位置

- `list.rs` 第 254-295 行：`generate_static_grid()` 方法
- `generator.rs` 第 906-942 行：`list_element_to_compose()` 静态 grid 检测

---

## 生成的首页示例

```kotlin
// IndexPage.kt
@Composable
fun IndexPage(navController: NavHostController, modifier: Modifier = Modifier) {
    Column(modifier = Modifier.padding(16.dp)) {
        Text("Component Gallery", style = TextStyle(
            MaterialTheme.typography.headlineLarge,
            fontSize = 24.sp,
            fontWeight = FontWeight.Bold
        ))
        Text("Jetpack Compose Demo - Phase 1", style = TextStyle(
            color = Color(0x4B5563)
        ))

        FlowRow(
            modifier = Modifier,
            horizontalArrangement = Arrangement.spacedBy(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            // Counter Demo Card
            Box(modifier = Modifier.clickable { navController.navigate("/counter") }) {
                Card(modifier = Modifier.padding(16.dp).clip(RoundedCornerShape(8.dp))) {
                    Column(modifier = Modifier) {
                        Text("Counter Demo", style = TextStyle(
                            MaterialTheme.typography.headlineSmall,
                            fontSize = 18.sp,
                            fontWeight = FontWeight.SemiBold
                        ))
                        Text("A simple counter...", style = TextStyle(
                            color = Color(0x4B5563),
                            fontSize = 14.sp
                        ))
                    }
                }
            }
            // ... more cards
        }
    }
}
```

---

# Design

## 背景

- `component-gallery` 有 50+ 组件和 50+ 页面，一次性迁移成本太高
- `unified-demo` 已有一个简单可工作的 Counter widget（已验证 jet 后端可用）
- jet generator 目前只能处理基础的 widget 结构

## 策略：在 unified-demo 中逐步扩展

直接在 unified-demo 上扩展到"最小可用集合"，而不是在 gallery 里做裁剪。

## 第一阶段范围

### 组件清单（8个）

| 组件 | 类型 | 状态 | 说明 |
|------|------|------|------|
| `Button` | 基础 | ✅ 已有 | 已在 Counter 中使用 |
| `Text` | 基础 | ✅ 已有 | 已在 Counter 中使用 |
| `Column` | 布局 | ✅ 完成 | 垂直排列子元素 |
| `Row` | 布局 | ✅ 完成 | 水平排列子元素 |
| `Link` | 导航 | ✅ 完成 | 页面跳转，支持包裹元素 |
| `Card` | 容器 | ✅ 新增 | Material3 卡片组件 |
| `Grid` | 布局 | ✅ 新增 | FlowRow 响应式网格 |
| `Tailwind class` | 样式 | ✅ 新增 | class 属性样式支持 |

### 页面清单（4个）

| 页面 | 内容 | 状态 |
|------|------|------|
| `index.at` | 首页，Card 展示 demo 列表 | ✅ 完成 |
| `counter.at` | Counter demo（保留现有） | ✅ 完成 |
| `column.at` | Column demo（示例 + 说明） | ✅ 完成 |
| `row.at` | Row demo（示例 + 说明） | ✅ 完成 |

### Jet Generator 新增功能

| 功能 | 对应 AURA 语法 | 生成目标 |
|------|----------------|----------|
| `routes` 块处理 | `routes { "/" => use index ... }` | `Routes` sealed class + `NavHost` |
| `link` 组件 | `link (to: "/button") { ... }` | `navController.navigate(Routes.XXX)` |

## 文件结构

```
unified-demo/
├── app.at              # 入口，定义 routes
├── pages/
│   ├── index.at        # 首页
│   ├── counter.at      # Counter demo
│   ├── column.at       # Column demo
│   └── row.at          # Row demo
└── components/
    └── counter.at      # Counter 组件定义
```

## AURA → NavHost 映射

| AURA 语法 | Jetpack Compose |
|-----------|-----------------|
| `routes { "/" => use index ... }` | `NavHost(navController, startDestination = Routes.Index)` |
| `"/counter" => use counter` | `composable<Routes.Counter> { CounterPage() }` |
| `link (to: "/counter")` | `navController.navigate(Routes.Counter)` |
| `outlet` | NavHost 内部自动渲染匹配的页面 |

### 生成的 Kotlin 代码结构

```kotlin
// 1. 定义路由对象
@Serializable
object Routes {
    @Serializable data object Index
    @Serializable data object Counter
    @Serializable data object Column
    @Serializable data object Row
}

// 2. NavHost 结构
val navController = rememberNavController()
NavHost(navController, startDestination = Routes.Index) {
    composable<Routes.Index> { IndexPage(navController) }
    composable<Routes.Counter> { CounterPage() }
    composable<Routes.Column> { ColumnPage() }
    composable<Routes.Row> { RowPage() }
}

// 3. Link 组件
@Composable
fun Link(to: Any, navController: NavController, content: @Composable () -> Unit) {
    Text(
        modifier = Modifier.clickable { navController.navigate(to) },
        // ...
    )
}
```

---

# Implementation Plan

## Task 1: 创建 pages/ 目录结构

**Files:**
- Create: `examples/unified-demo/pages/` directory
- Create: `examples/unified-demo/components/` directory

**Steps:**

1. 创建目录结构：
```bash
mkdir -p examples/unified-demo/pages
mkdir -p examples/unified-demo/components
```

2. 验证目录创建成功

---

## Task 2: 拆分 Counter 组件

**Files:**
- Create: `examples/unified-demo/components/counter.at`
- Create: `examples/unified-demo/pages/counter.at`

**Steps:**

1. 将 `app.at` 中的 Counter widget 移到 `components/counter.at`

2. 创建 `pages/counter.at` 作为 Counter demo 页面：
```auto
widget CounterPage {
    view {
        col {
            h2 "Counter Demo"
            text "A simple counter component demonstrating state management."
            Counter {}
        }
    }
}
```

---

## Task 3: 创建首页 index.at

**Files:**
- Create: `examples/unified-demo/pages/index.at`

**Content:**
```auto
widget IndexPage {
    view {
        col {
            h1 "Component Gallery"
            text "Jetpack Compose Demo - Phase 1"

            link (to: "/counter") {
                text "Counter Demo"
            }
            link (to: "/column") {
                text "Column Demo"
            }
            link (to: "/row") {
                text "Row Demo"
            }
        }
    }
}
```

---

## Task 4: 创建 Column demo 页面

**Files:**
- Create: `examples/unified-demo/pages/column.at`

**Content:**
```auto
widget ColumnPage {
    view {
        col {
            h2 "Column Demo"
            text "Column arranges children vertically."

            col (gap: "8") {
                text "Item 1"
                text "Item 2"
                text "Item 3"
            }
        }
    }
}
```

---

## Task 5: 创建 Row demo 页面

**Files:**
- Create: `examples/unified-demo/pages/row.at`

**Content:**
```auto
widget RowPage {
    view {
        col {
            h2 "Row Demo"
            text "Row arranges children horizontally."

            row (gap: "8") {
                text "Left"
                text "Center"
                text "Right"
            }
        }
    }
}
```

---

## Task 6: 改造 app.at 支持 routes

**Files:**
- Modify: `examples/unified-demo/app.at`

**Content:**
```auto
widget App {
    routes {
        "/" => use index
        "/counter" => use counter
        "/column" => use column
        "/row" => use row
    }

    view {
        col {
            header {
                text "Auto UI Demo"
            }
            outlet
        }
    }
}
```

---

## Task 7: 实现 Jet Generator 的 routes 处理

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/jet/generator.rs`

**Steps:**

1. 添加 `generate_routes_sealed()` 方法：
```rust
/// Generate Routes sealed class from widget.routes
fn generate_routes_sealed(&self, widget: &AuraWidget) -> String {
    if widget.routes.is_empty() {
        return String::new();
    }

    let mut variants = Vec::new();
    for route in &widget.routes {
        let name = pascal_case(&route.name);
        variants.push(format!("    @Serializable data object {} : Routes()", name));
    }

    format!("@Serializable\nobject Routes {{\n{}\n}}", variants.join("\n"))
}
```

2. 添加 `generate_nav_host()` 方法：
```rust
/// Generate NavHost from widget.routes
fn generate_nav_host(&self, widget: &AuraWidget) -> GenResult<String> {
    let mut composables = Vec::new();
    for route in &widget.routes {
        let name = pascal_case(&route.name);
        let page_fn = format!("{}Page", name);
        composables.push(format!(
            "        composable<Routes.{}> {{\n            {}()\n        }}",
            name, page_fn
        ));
    }

    Ok(format!(
        r#"val navController = rememberNavController()
NavHost(navController, startDestination = Routes.Index) {{
{}
}}"#,
        composables.join("\n")
    ))
}
```

3. 在 `generate()` 方法中集成 routes 生成

---

## Task 8: 实现 Link 组件转换

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/jet/generator.rs`

**Steps:**

1. 添加 `link_to_compose()` 方法：
```rust
/// Convert link to Compose clickable text
fn link_to_compose(
    &mut self,
    props: &HashMap<String, AuraPropValue>,
    children: &[AuraNode],
    indent: usize,
) -> GenResult<String> {
    let ind = "    ".repeat(indent);
    let to = props.get("to")
        .and_then(|v| match v {
            AuraPropValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_default();

    // Convert "/counter" to "Routes.Counter"
    let route_name = to.trim_start_matches('/')
        .split('/')
        .next()
        .unwrap_or("");
    let route_ref = format!("Routes.{}", pascal_case(route_name));

    let content = self.children_to_compose(children, indent + 1)?;

    Ok(format!(
        "{}Text(\n{}    text = \"{}\",\n{}    modifier = Modifier.clickable {{ navController.navigate({}) }}\n{})",
        ind, ind, content.trim(), ind, route_ref, ind
    ))
}
```

2. 在 `element_to_compose()` 中添加 "link" 分支

---

## Task 9: 测试完整流程

**Steps:**

1. 构建项目：
```bash
cargo build --release
```

2. 运行 jet 生成：
```bash
cd examples/unified-demo
../../target/release/auto.exe gen jet
```

3. 检查生成的 Kotlin 代码：
```bash
cat jet/app/src/main/java/com/example/unified_demo/App.kt
```

4. 构建 Android 项目（需要在 Android Studio 中打开）

---

## Success Criteria

1. ✅ `unified-demo` 有清晰的 pages/ 和 components/ 目录结构
2. ✅ `app.at` 定义了 routes 块
3. ✅ Jet Generator 处理 widget.routes 生成 NavHost
4. ✅ Jet Generator 生成 NavHost 结构
5. ✅ Link 组件生成 `navController.navigate()` 调用
6. ✅ 首页显示所有 demo 链接
7. ✅ 点击链接可以跳转到对应 demo 页面
8. ✅ **Card 组件正确生成 Material3 Card**
9. ✅ **Tailwind class 属性转换为 Compose Modifier/TextStyle**
10. ✅ **静态 Grid 使用 FlowRow 生成响应式布局**
11. ✅ **Link 可以包裹 Card 等元素子节点**

## Actual Implementation Notes

**routes 处理实现位置：**
- `generator.rs` 第 1232-1260 行
- 检测 `widget.routes.is_some()` 判断是否为路由 widget
- 使用 `NavigationGenerator::add_routes_from_aura()` 添加路由
- 调用 `NavigationGenerator::generate_nav_host("/")` 生成 NavHost

**Link 组件实现位置：**
- `generator.rs` 第 846-904 行
- 通过 `AuraNode::Link` 处理
- 支持文本子节点（生成 Text + clickable）
- 支持元素子节点（生成 Box + clickable）

**Card 组件实现位置：**
- `layout.rs` 第 323-353 行
- 生成 Material3 Card composable

**Tailwind class 处理实现位置：**
- `modifier.rs` 第 43-60 行：dimension_to_dp() 单位换算
- `modifier.rs` 第 219-226 行：rounded → clip(RoundedCornerShape)
- `layout.rs` 第 373-387 行：class_to_modifier() 返回空字符串处理
- `list.rs` 第 413-463 行：class_to_modifier() 简化版实现

**静态 Grid 实现位置：**
- `list.rs` 第 254-295 行：generate_static_grid() 方法
- `generator.rs` 第 905-942 行：静态 grid 检测和调用

**已验证的文件结构：**
```
examples/unified-demo/front/
├── app.at              # 入口，定义 routes
├── components/
│   └── counter.at      # Counter 组件定义
│   └── DemoCard.at     # Demo card 组件
└── pages/
    ├── column.at       # Column demo
    ├── counter.at      # Counter demo 页面
    ├── index.at        # 首页（Card 导航链接）
    └── row.at          # Row demo
```

**生成的 Kotlin 文件：**
```
examples/unified-demo/jet/app/src/main/java/com/example/unified_demo/ui/widgets/
├── App.kt              # 入口 + NavHost
├── IndexPage.kt        # 首页（FlowRow + Card + Link）
├── CounterPage.kt      # Counter demo 页面
├── ColumnPage.kt       # Column demo 页面
├── RowPage.kt          # Row demo 页面
├── Counter.kt          # Counter 组件
└── DemoCard.kt         # Demo card 组件
```

---

## Future Phases

### Phase 2: 扩展更多组件

- 表单组件：Input, Checkbox, Switch, Slider
- 反馈组件：Toast, Dialog, Alert, Progress

### Phase 3: 完善页面内容

- 添加代码示例展示
- 添加 API 属性表格
- 添加交互式 demo

### Phase 4: 迁移到 component-gallery

- 将验证过的组件迁移到 gallery
- 逐步扩展 gallery 的 jet 后端支持
