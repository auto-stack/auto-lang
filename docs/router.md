# Auto Router

Auto provides built-in routing support for single-page applications (SPA). This feature implements URL-driven navigation using Vue Router as the backend target.

## Overview

The Auto Router enables:
- Declarative route definitions in `routes` blocks
- Navigation via `link` elements (declarative) or `nav()` function (programmatic)
- Route outlets for rendering matched components
- Route parameter extraction (e.g., `/user/:id`)
- Lazy loading for optimal bundle size (Plan 106)

## Basic Usage

### Define Routes (Plan 106 - Recommended)

Use the `use` keyword to specify page modules:

```auto
widget App {
    routes {
        "/" => use index
        "/about" => use about
        "/user/:id" => use user
    }

    view {
        col {
            Sidebar {}
            main {
                outlet
            }
        }
    }
}
```

**Convention:**
- `use index` maps to `@/pages/index.vue`
- `use about` maps to `@/pages/about.vue`
- `use user` maps to `@/pages/user.vue`

### Define Routes (Plan 105 - Backward Compatible)

The old syntax with component names is still supported:

```auto
widget App {
    routes {
        "/" => HomePage {}
        "/about" => AboutPage {}
        "/user/:id" => UserPage {}
    }

    view {
        col {
            main {
                outlet
            }
        }
    }
}
```

**Note:** This syntax converts component names to lowercase module names (e.g., `HomePage` → `homepage`).

### Navigation

**Declarative (link element):**

```auto
link (to: "/about") {
    text "About Us"
}
```

**Programmatic (nav function):**

```auto
fn handleLogin() {
    nav("/dashboard")
}

fn viewProfile(userId: str) {
    nav("/user", id: userId)
}
```

### Route Parameters

Access route parameters in page widgets:

```auto
widget UserPage {
    model {
        userId str = route.id
    }

    computed {
        apiUrl => f"/api/users/{.userId}"
    }
}
```

## Generated Output

When you use `auto vue` to generate a Vue project:

### Router Configuration (`src/router/index.ts`)

Plan 106 generates lazy-loaded routes:

```typescript
import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'index', component: () => import('@/pages/index.vue') },
  { path: '/about', name: 'about', component: () => import('@/pages/about.vue') },
  { path: '/user/:id', name: 'user', component: () => import('@/pages/user.vue'), props: true }
]

const router = createRouter({
  history: createWebHistory(import.meta.url),
  routes,
})

export default router
```

### Main Entry (`src/main.ts`)

```typescript
import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import './assets/index.css'

const app = createApp(App)
app.use(router)
app.mount('#app')
```

### Package Dependencies

```json
{
  "dependencies": {
    "vue": "^3.4.0",
    "vue-router": "^4.2.0",
    ...
  }
}
```

## Vue Router Mapping

| Auto | Vue |
|------|-----|
| `routes {}` | `routes: [...]` in router config |
| `"/" => use index` | `{ path: '/', component: () => import('@/pages/index.vue') }` |
| `outlet` | `<router-view>` |
| `link (to: "/path")` | `<router-link to="/path">` |
| `nav("/path")` | `router.push("/path")` |
| `route.id` | `route.params.id` |

## Syntax Reference

### Routes Block (Plan 106)

```auto
routes {
    PATH => use MODULE_NAME
    ...
}
```

- `PATH`: String literal (e.g., `"/"`, `"/about"`, `"/user/:id"`)
- `MODULE_NAME`: Lowercase module name (e.g., `index`, `about`, `user`)
- Maps to `@/pages/{MODULE_NAME}.vue`
- Parameters are automatically extracted from paths like `/user/:id`

### Routes Block (Plan 105 - Backward Compatible)

```auto
routes {
    PATH => ComponentName {}
    ...
}
```

- `ComponentName` is converted to lowercase for the module name
- Example: `HomePage {}` → `@/pages/homepage.vue`

### Outlet

```auto
outlet
```

Renders the matched route component at this location.

### Link

```auto
link (to: "/path") {
    // Children
}
```

Creates a navigation link. The `to` prop specifies the target path.

### Nav Function

```auto
nav("/path")
nav("/path", param1: value1, param2: value2)
```

Programmatically navigates to a path with optional parameters.

## File Structure Convention

For Plan 106, use lowercase file names:

```
source/front/
├── app.at           # Main app with routes
└── pages/
    ├── index.at     # "/" route → @/pages/index.vue
    ├── about.at     # "/about" route → @/pages/about.vue
    └── user.at      # "/user/:id" route → @/pages/user.vue
```

## CLI Usage

Generate a Vue project with router:

```bash
auto vue source/front -o output/app
```

If any widget contains a `routes` block, the generated project will include:
- `src/router/index.ts` - Router configuration with lazy loading
- `vue-router` dependency in `package.json`
- Router setup in `src/main.ts`

## Plan History

### Plan 105 (Original)
- Syntax: `"/path" => ComponentName {}`
- Static imports: `import ComponentName from '@/pages/ComponentName.vue'`
- File names: PascalCase (e.g., `HomePage.vue`)

### Plan 106 (Current - Recommended)
- Syntax: `"/path" => use module`
- Lazy loading: `component: () => import('@/pages/module.vue')`
- File names: lowercase (e.g., `index.vue`)
- Backward compatible with Plan 105

## Future Phases

### Phase 2: `app` Keyword

- Add `app` as first-class keyword (parallel to `widget`)
- Move `routes` from widget to app
- Support app-level configuration (theme, i18n)

### Phase 3: AURA Architecture Refactoring

- Add `AuraApp` struct to AURA
- Separate app-level and widget-level concerns
- Support multiple backends (Vue, Compose, GPUI)

### Phase 4: Advanced Routing

- Nested routes (`children`)
- Route guards (`beforeEnter`)
- Redirects and aliases
