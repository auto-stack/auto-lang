# API Example - Frontend & Backend Infrastructure

This example demonstrates the **single source, dual deployment** pattern from Plan 102 Phase 5.

## Project Structure

```
examples/api-example/
├── pac.at                        # Workspace root config
│
└── source/
    ├── front/                    # AURA Frontend
    │   ├── pac.at                # Frontend config (scenario: "ui")
    │   ├── app.at                # Main app component
    │   └── userlist.at           # User list component
    │
    └── back/                     # Auto Backend
        ├── pac.at                # Backend config
        ├── api.at                # API interface definitions
        ├── db.at                 # Database service
        └── service.at            # Common utilities
```

## Config Files

### Workspace Root (`pac.at`)
```auto
name: "api-example"
workspace: {
    front: "./source/front"
    back: "./source/back"
}
```

### Frontend (`source/front/pac.at`)
```auto
name: "api-example-ui"
scenario: "ui"          // IMPORTANT: marks this as AURA package
entry: "app.at"
api: "../back/api.at"   // API import path
```

### Backend (`source/back/pac.at`)
```auto
name: "api-example-api"
entry: "api.at"
```

## Commands

### `auto.exe vue`
Compiles the entire project:
1. Reads `source/front/pac.at` (detects `scenario: "ui"`)
2. Compiles `source/front/*.at` as AURA → generates `.vue` files
3. Compiles `source/back/*.at` as Auto → generates `.rs` files
4. Generates workspace-level files (`vite.config.js`, `index.html`, etc.)
5. Runs interactive commands:
   - `npm install`
   - `npx shadcn-vue@latest add button input card table`

### `auto.exe run`
Runs the development server:
- Executes `npm run dev`

### `auto.exe tauri`
For Tauri desktop apps:
- Generates Tauri-specific Rust code
- Runs `tauri dev`

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    source/front/ (AURA)                         │
│                                                                 │
│  app.at, userlist.at                                            │
│  └── import { api } from "../back/api.at"                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    source/back/ (Auto)                          │
│                                                                 │
│  api.at   → #[api(method = "GET", path = "/users/:id")]        │
│  db.at    → Database operations                                 │
│  service.at → Utilities                                         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    auto.exe vue                                 │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Frontend Generation                         │   │
│  │                                                         │   │
│  │  source/front/app.at     → dist/app.vue                 │   │
│  │  source/front/userlist.at → dist/userlist.vue           │   │
│  │  source/back/api.at      → dist/api.ts (types + client) │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Backend Generation                          │   │
│  │                                                         │   │
│  │  source/back/*.at → src-tauri/src/*.rs (Tauri)          │   │
│  │                   or src-server/*.rs (axum)              │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Workspace Files                             │   │
│  │                                                         │   │
│  │  vite.config.js, index.html, package.json               │   │
│  │  tsconfig.json, src-tauri/                              │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
         ┌────────────────────┴────────────────────┐
         │                                         │
         ▼                                         ▼
┌─────────────────────┐                  ┌─────────────────────┐
│  Tauri Desktop App  │                  │  Web Application    │
│                     │                  │                     │
│  WebView (Vue)      │                  │  Browser (Vue)      │
│      │              │                  │      │              │
│      │ IPC          │                  │      │ HTTP         │
│      ▼              │                  │      ▼              │
│  Rust Backend       │                  │  Rust Server        │
│  (tauri command)    │                  │  (axum routes)      │
└─────────────────────┘                  └─────────────────────┘
```

## API Annotation Attributes

| Attribute | Description | Example |
|-----------|-------------|---------|
| `method` | HTTP method | `#[api(method = "POST")]` |
| `path` | Custom path | `#[api(path = "/users/:id")]` |
| `name` | Custom function name | `#[api(name = "getUserById")]` |
| `auth` | Requires authentication | `#[api(auth = true)]` |
| `cache` | Cache duration (seconds) | `#[api(cache = 60)]` |

## Benefits

1. **Single Source of Truth**: Write business logic once in Auto
2. **Type Safety**: End-to-end type checking (Auto → Rust → TypeScript)
3. **Flexible Deployment**: Same code works as desktop app or web app
4. **Easy Maintenance**: Change backend, regenerate frontend types automatically
5. **Convention over Configuration**: Standard project structure with `pac.at`
