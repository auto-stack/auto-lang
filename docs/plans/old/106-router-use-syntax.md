# Plan 106: Router `use` 语法改进

## 目标

改进 Plan 105 的 router 语法，使用 `use` 关键字配合模块路径约定，实现更简洁、严谨的路由定义。

## 背景

### 当前问题

Plan 105 的 routes 语法使用组件名：

```auto
routes {
    "/" => IndexPage {}
    "/button" => ButtonPage {}
}
```

问题：
1. 编译器不知道 `IndexPage` 从哪里导入
2. 生成的 router 假设路径为 `@/pages/IndexPage.vue`，是猜测而非确定
3. 需要额外的 `use` 语句来声明模块依赖

### 新方案

使用 `use` 关键字 + 模块名约定：

```auto
routes {
    "/" => use index
    "/button" => use button
    "/user/:id" => use user
}
```

## 设计

### 语法规范

```ebnf
route ::= path "=>" "use" module_name props?
path ::= string_literal
module_name ::= identifier
props ::= "(" prop_list ")"
```

**示例：**

```auto
widget App {
    routes {
        "/" => use index
        "/button" => use button
        "/card" => use card
        "/user/:id" => use user
        "/settings" => use settings (name: "settings")
    }

    view {
        col {
            nav {
                link (to: "/") { text "Home" }
                link (to: "/button") { text "Button" }
            }
            main {
                outlet
            }
        }
    }
}
```

### 约定

| `use` 语句 | 导入模块 | Vue 文件 | 组件 |
|------------|----------|----------|------|
| `use index` | `pages/index` | `@/pages/index.vue` | 模块导出的组件 |
| `use button` | `pages/button` | `@/pages/button.vue` | 模块导出的组件 |
| `use user` | `pages/user` | `@/pages/user.vue` | 模块导出的组件 |

**规则：**
1. `use xxx` 映射到 `@/pages/xxx.vue`
2. 每个页面模块文件只导出一个组件（widget）
3. 文件名使用小写（`index.at`、`button.at`）
4. 路由名称默认为模块名（`index`、`button`）

### 生成的 Vue Router

```typescript
// src/router/index.ts
import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'index',
    component: () => import('@/pages/index.vue')
  },
  {
    path: '/button',
    name: 'button',
    component: () => import('@/pages/button.vue')
  },
  {
    path: '/card',
    name: 'card',
    component: () => import('@/pages/card.vue')
  },
  {
    path: '/user/:id',
    name: 'user',
    component: () => import('@/pages/user.vue'),
    props: true
  }
]

const router = createRouter({
  history: createWebHistory(import.meta.url),
  routes,
})

export default router
```

## 与 Plan 105 的对比

| 特性 | Plan 105 | Plan 106 |
|------|----------|----------|
| 语法 | `"/" => IndexPage {}` | `"/" => use index` |
| 导入路径 | 猜测 `@/pages/IndexPage.vue` | 确定 `@/pages/index.vue` |
| 懒加载 | 否 | 是（默认） |
| 需要 `use` 语句 | 理论上需要 | 集成在 routes 中 |
| 文件命名 | PascalCase | lowercase |

## 实现任务

### Task 1: 扩展 Token 类型

**文件:** `crates/auto-lang/src/token.rs`

添加 `Use` 关键字（如果尚未存在）。

### Task 2: 修改 AST 节点

**文件:** `crates/auto-lang/src/ast/route.rs`

修改 `RouteDef` 结构体：

```rust
pub struct RouteDef {
    pub path: String,
    pub module: String,      // 改名：component -> module
    pub params: Vec<String>,
    pub props: HashMap<String, Expr>,  // 新增：可选 props
}
```

### Task 3: 修改 Parser

**文件:** `crates/auto-lang/src/parser.rs`

修改 `parse_routes_block_inner()` 解析 `use` 语法：

```rust
// 解析: "/path" => use module_name
let path = self.expect_string()?;
self.expect(TokenKind::DoubleArrow)?;
self.expect_ident("use")?;
let module = self.expect_ident_str()?;  // 模块名
```

### Task 4: 修改 AURA Types

**文件:** `crates/auto-lang/src/aura/types.rs`

修改 `AuraRoute`：

```rust
pub struct AuraRoute {
    pub path: String,
    pub module: String,      // 模块名（如 "index", "button"）
    pub params: Vec<String>,
}
```

### Task 5: 修改 Vue Generator

**文件:** `crates/auto-lang/src/ui_gen/vue.rs`

修改 `generate_router_file()` 生成懒加载格式：

```rust
fn generate_router_file(routes: &[AuraRoute]) -> String {
    let route_defs: Vec<String> = routes.iter().map(|route| {
        let props = if route.params.is_empty() {
            String::new()
        } else {
            ", props: true".to_string()
        };

        format!(
            "  {{ path: '{}', name: '{}', component: () => import('@/pages/{}.vue'){} }}",
            route.path,
            route.module,
            route.module,
            props
        )
    }).collect();

    // ...
}
```

### Task 6: 修改 cmd_vue.rs

**文件:** `crates/auto/src/cmd_vue.rs`

修改 Vue 文件生成，使用小写文件名：

```rust
// 原来：widget_name.vue (如 IndexPage.vue)
// 改为：file_stem.vue (如 index.vue)
let component_file = output_subdir.join(format!("{}.vue", file_stem));
```

### Task 7: 更新测试

**文件:** `crates/auto-lang/test/router/000_basic/routes.at`

更新测试用例使用新语法。

### Task 8: 更新文档

**文件:** `docs/router.md`

更新文档说明新的 `use` 语法。

### Task 9: 更新示例

**文件:** `examples/component-gallery/source/front/app.at`

更新示例使用新的 routes 语法。

## 向后兼容

Plan 106 可以与 Plan 105 语法共存：

```auto
routes {
    "/" => use index           // Plan 106 语法（推荐）
    "/legacy" => LegacyPage {} // Plan 105 语法（兼容）
}
```

Parser 检测 `=>` 后面是 `use` 还是标识符来区分两种语法。

## 验收标准

1. `use index` 语法正确解析
2. 生成的 router 使用懒加载 `() => import('@/pages/index.vue')`
3. Vue 文件名使用小写（`index.vue` 而非 `IndexPage.vue`）
4. 测试全部通过
5. 文档更新完整

## 风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 破坏现有代码 | 中 | 高 | 保持向后兼容 |
| 文件名约定冲突 | 低 | 中 | 清晰的命名规范文档 |

## 时间估计

- Task 1-2: 0.5 天
- Task 3-4: 1 天
- Task 5-6: 1 天
- Task 7-9: 0.5 天

**总计: 3 天**
