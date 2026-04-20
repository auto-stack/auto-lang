在 Vue.js (Vue 3) 中，实现“左侧导航栏 + 右侧主内容栏”的标准架构是基于 **SPA（单页应用）模式**，核心完全依赖于官方提供的路由库：**Vue Router**。

这个架构在业界通常被称为 **“App Shell（应用外壳）”** 或 **“嵌套路由（Nested Routing）”** 布局。

其核心原理非常简单：**左侧导航栏是静态的（不动），右侧主内容区是一个“插槽（Outlet）”，Vue Router 会根据当前浏览器的 URL 地址，动态地把对应的组件“塞”进这个插槽里。**

以下是标准的架构实现步骤和具体代码：

### 1. 核心架构要素

* **`Layout.vue` (布局组件)**：定义页面的骨架（左边菜单，右边内容区）。
* **`<router-view>` (路由出口)**：Vue Router 提供的内置组件，充当右侧内容区的“占位符”。
* **`<router-link>` (路由链接)**：Vue Router 提供的内置组件，用于左侧菜单的点击跳转（它会被渲染成 `<a>` 标签，但会拦截默认的页面刷新行为）。
* **路由配置 (`router.js`)**：定义 URL 路径与组件的映射关系。

---

### 2. 具体实现代码

#### 第一步：配置路由表 (Router Configuration)

这是最关键的一步，你需要使用**嵌套路由（`children`）**的特性。让 `Layout` 充当父路由，各种具体的页面充当子路由。

```javascript
// src/router/index.js
import { createRouter, createWebHistory } from 'vue-router'
import Layout from '../components/Layout.vue'
import Dashboard from '../views/Dashboard.vue'
import UserList from '../views/UserList.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      // 根路径
      path: '/',
      component: Layout, // 父组件：外壳
      redirect: '/dashboard', // 默认重定向到控制台
      children: [
        {
          // 当 URL 为 /dashboard 时，Dashboard 组件会被渲染在 Layout 的 <router-view> 中
          path: 'dashboard', 
          component: Dashboard 
        },
        {
          // 当 URL 为 /users 时，UserList 组件会被渲染
          path: 'users', 
          component: UserList 
        }
      ]
    }
  ]
})

export default router

```

#### 第二步：编写布局组件 (`Layout.vue`)

在这里划定左侧和右侧的区域。右侧核心区域必须放一个 `<router-view>`。

```vue
<template>
  <div class="app-layout">
    <aside class="sidebar">
      <nav>
        <router-link to="/dashboard" active-class="active-menu">控制台</router-link>
        <router-link to="/users" active-class="active-menu">用户管理</router-link>
      </nav>
    </aside>

    <main class="main-content">
      <router-view></router-view> 
    </main>
  </div>
</template>

<style scoped>
/* 简单的 CSS 布局实现 */
.app-layout {
  display: flex;
  height: 100vh;
}
.sidebar {
  width: 200px;
  background-color: #f4f4f4;
  padding: 20px;
}
.sidebar a {
  display: block;
  margin-bottom: 10px;
  text-decoration: none;
  color: #333;
}
.sidebar a.active-menu {
  color: #42b983; /* Vue 的主题绿 */
  font-weight: bold;
}
.main-content {
  flex: 1; /* 占据剩余全部空间 */
  padding: 20px;
  overflow-y: auto;
}
</style>

```

#### 第三步：编程式跳转（如果不使用 router-link）

有时候导航栏的点击不是直接用 `<a>` 标签，而是绑在某个自定义的 UI 库的菜单项（Menu Item）上，比如 Element Plus 或 Ant Design Vue。此时可以通过代码触发跳转：

```vue
<script setup>
import { useRouter } from 'vue-router'

const router = useRouter()

const handleMenuClick = (path) => {
  // 编程式导航，推动 URL 变化，从而触发 <router-view> 的内容替换
  router.push(path) 
}
</script>

<template>
  <div @click="handleMenuClick('/users')">点击跳转用户管理</div>
</template>

```

### 3. 为什么在 AutoUI 中了解这个很重要？

如果你的 `auto-react-transpiler` 或未来的 `auto-vue-transpiler` 需要支持多页面应用（而不是只有一个单纯的组件），你就需要在 Auto 语言层面抽象出**“路由（Routing）”**的概念。

在 Auto 语言里，你可能不会让用户直接写 `<router-view>`，而是设计一套类似页面级别的声明：

```auto
// Auto 语言可能的路由抽象
app AdminApp {
    layout {
        // 左侧
        Menu {
            Item("控制台", route: .Dashboard)
            Item("用户", route: .Users)
        }
        // 右侧出口
        Outlet() 
    }
}

```

然后你的转译器负责把这个 `Outlet()` 翻译成 Vue 的 `<router-view>` 或 React 的 `<Outlet />` (React Router v6)。
