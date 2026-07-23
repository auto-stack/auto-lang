# Router（SPA 路由）

## 范围

Auto 内置的单页应用路由：`routes` 块声明式定义、`link`/`nav()` 导航、`outlet` 渲染出口、路由参数提取。当前唯一后端目标是 Vue Router（`ui_gen/vue.rs:VueGenerator::generate_router_file`）。

## 语法（双形态并存）

**Plan 106（推荐）**：`use` + 模块名约定，小写文件名，懒加载。

```auto
widget App {
    routes {
        "/" => use index
        "/user/:id" => use user
    }
    view { col { main { outlet } } }
}
```

约定：`use index` → `@/pages/index.vue`；源文件 `source/front/pages/index.at` 小写命名。

**Plan 105（兼容保留）**：`"/" => HomePage {}`，组件名转小写模块名（`HomePage` → `homepage`），静态 import、PascalCase 文件名。

## 导航与参数

- 声明式：`link (to: "/about") { ... }` → `<router-link>`。
- 编程式：`nav("/dashboard")`、`nav("/user", id: userId)` → `router.push`。
- 参数：`route.id` → `route.params.id`，可在 `model`/`computed` 中直接引用。

## Vue 映射表

| Auto | Vue |
|---|---|
| `routes {}` | router config `routes: [...]` |
| `"/" => use index` | `{ path, component: () => import('@/pages/index.vue') }` |
| `outlet` | `<router-view>` |
| `link (to: ...)` | `<router-link to=...>` |
| `nav(...)` | `router.push(...)` |
| `route.id` | `route.params.id` |

任一 widget 含 `routes` 块时，生成工程自动附带 `src/router/index.ts`（懒加载）、`vue-router` 依赖与 `main.ts` 的 `app.use(router)`。IR 侧对应 `aura/types.rs:AuraRoutes`/`AuraRoute`。

## 不变量

- 路由表同时进入 AURA IR（`AuraWidget.routes`），codegen 与 VM 渲染共用。
- 生成器探测 router 需求（outlet/link/`nav()` 任一出现）才注入依赖——见 `vue.rs` 中 `needs_router`（Plan 105 注释）。

## 显式非目标

- Phase 2-4 未做：`app` 一等关键字（routes 移出 widget）、app 级 theme/i18n；`AuraApp` 分离 app/widget 关注（`AuraApp` 结构体已在 types.rs 预留）；嵌套路由 `children`、`beforeEnter` 守卫、redirect/alias。
- 非 Vue 后端的路由映射未定。

> 来源: docs/router.md；docs/plans/old/{105-auto-router,106-router-use-syntax}.md；crates/auto-lang/src/ui_gen/vue.rs（generate_router_file）；crates/auto-lang/src/aura/types.rs
