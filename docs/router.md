# Auto Router

Auto provides built-in routing support for single-page applications (SPA). This feature implements URL-driven navigation using Vue Router as the backend target.

## Overview

The Auto Router enables:
- Declarative route definitions in `routes` blocks
- Navigation via `link` elements (declarative) or `nav()` function (programmatic)
- Route outlets for rendering matched components
- Route parameter extraction (e.g., `/user/:id`)

## Basic Usage

### Define Routes

Add a `routes` block to your root widget:

```auto
widget App {
    routes {
        "/" => HomePage {}
        "/about" => AboutPage {}
        "/user/:id" => UserPage {}
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

```typescript
import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'
import HomePage from '@/pages/HomePage.vue'
import AboutPage from '@/pages/AboutPage.vue'
import UserPage from '@/pages/UserPage.vue'

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'HomePage', component: HomePage },
  { path: '/about', name: 'AboutPage', component: AboutPage },
  { path: '/user/:id', name: 'UserPage', component: UserPage, props: true }
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
| `outlet` | `<router-view>` |
| `link (to: "/path")` | `<router-link to="/path">` |
| `nav("/path")` | `router.push("/path")` |
| `route.id` | `route.params.id` |

## Syntax Reference

### Routes Block

```auto
routes {
    PATH => ComponentName {}
    ...
}
```

- `PATH`: String literal (e.g., `"/"`, `"/about"`, `"/user/:id"`)
- `ComponentName`: Widget name to render for this route
- Parameters are automatically extracted from paths like `/user/:id`

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

## CLI Usage

Generate a Vue project with router:

```bash
auto vue source/front -o output/app
```

If any widget contains a `routes` block, the generated project will include:
- `src/router/index.ts` - Router configuration
- `vue-router` dependency in `package.json`
- Router setup in `src/main.ts`

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
- Lazy loading
- Redirects and aliases
