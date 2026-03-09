# Plan 116: a2jet Phase 5 - Lists & Data

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现列表和数据绑定组件生成。

**Architecture:** 在 Phase 1-4 基础上，添加 `ListGenerator` 模块处理 LazyColumn、LazyRow、Grid 等列表组件。

**Tech Stack:** Rust, Kotlin, Jetpack Compose, Material3

---

## 1. 列表组件映射

### 1.1 AURA → Compose 列表

| AURA Tag | Compose Component | 说明 |
|----------|-------------------|------|
| `list` | `LazyColumn` | 垂直滚动列表 |
| `list-row` | `LazyRow` | 水平滚动列表 |
| `grid` | `LazyVerticalGrid` | 网格列表 |
| `flow-row` | `FlowRow` | 流式布局 |
| `flow-col` | `FlowColumn` | 流式布局 |

### 1.2 列表属性

| AURA 属性 | Compose 属性 | 说明 |
|-----------|-------------|------|
| `items: .listRef` | `items(listRef)` | 数据源绑定 |
| `key: {item.id}` | `key { item.id }` | 列表项键 |
| `contentType: "item"` | `contentType("item")` | 内容类型 |
| `columns: N` | `GridCells.Fixed(N)` | 网格列数 |

---

## 2. 数据绑定

### 2.1 列表迭代

**AURA:**
```auto
list {
    items: .users,
    key: {item.id},

    UserItem { user: item }
}
```

**Kotlin:**
```kotlin
LazyColumn {
    items(
        items = users,
        key = { user -> user.id }
    ) { user ->
        UserItem(user = user)
    }
}
```

### 2.2 Grid 布局

**AURA:**
```auto
grid {
    columns: 2,
    items: .products,

    ProductCard { product: item }
}
```

**Kotlin:**
```kotlin
LazyVerticalGrid(
    columns = GridCells.Fixed(2),
    modifier = Modifier.fillMaxSize()
) {
    items(
        items = products,
        key = { product -> product.id }
    ) { product ->
        ProductCard(product = product)
    }
}
```

---

## 3. 文件结构

```
crates/auto-lang/src/ui_gen/jet/
├── mod.rs              # 更新导出
├── generator.rs        # 添加列表生成方法
└── list.rs             # 新增：列表组件生成器
```

---

## 4. 实现任务

| Task | 内容 | 预计时间 |
|------|------|---------|
| Task 1 | 创建 list.rs 模块 | 15 min |
| Task 2 | 实现 LazyColumn/LazyRow 生成 | 25 min |
| Task 3 | 实现 LazyVerticalGrid 生成 | 20 min |
| Task 4 | 实现数据绑定 (items, key) | 20 min |
| Task 5 | 集成到 JetGenerator | 15 min |
| Task 6 | 添加单元测试 | 15 min |
| Task 7 | 最终验证和提交 | 10 min |

---

## 5. 生成代码示例

### 5.1 Simple List

**AURA:**
```auto
list {
    class: "p-4",

    items: .todos,
    key: {item.id},

    TodoItem { todo: item }
}
```

**Kotlin:**
```kotlin
LazyColumn(
    modifier = Modifier
        .padding(16.dp)
        .fillMaxSize()
) {
    items(
        items = todos,
        key = { todo -> todo.id }
    ) { todo ->
        TodoItem(todo = todo)
    }
}
```

### 5.2 Horizontal List

**AURA:**
```auto
list-row {
    gap: 2,

    items: .categories,
    key: {item.id},

    CategoryChip { category: item }
}
```

**Kotlin:**
```kotlin
LazyRow(
    horizontalArrangement = Arrangement.spacedBy(8.dp)
) {
    items(
        items = categories,
        key = { category -> category.id }
    ) { category ->
        CategoryChip(category = category)
    }
}
```

### 5.3 Grid

**AURA:**
```auto
grid {
    columns: 3,
    gap: 2,

    items: .photos,
    key: {item.id},

    PhotoItem { photo: item }
}
```

**Kotlin:**
```kotlin
LazyVerticalGrid(
    columns = GridCells.Fixed(3),
    modifier = Modifier.fillMaxSize(),
    verticalArrangement = Arrangement.spacedBy(8.dp),
    horizontalArrangement = Arrangement.spacedBy(8.dp)
) {
    items(
        items = photos,
        key = { photo -> photo.id }
    ) { photo ->
        PhotoItem(photo = photo)
    }
}
```

---

## 6. 成功标准

- [ ] LazyColumn 支持 items, key, class 属性
- [ ] LazyRow 支持 items, key, gap 属性
- [ ] LazyVerticalGrid 支持 columns, gap, items 属性
- [ ] 数据绑定支持 item 变量引用
- [ ] 单元测试覆盖率 > 80%
- [ ] 所有测试通过
