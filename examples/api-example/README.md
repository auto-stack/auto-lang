# API Example - Frontend & Backend Infrastructure

This example demonstrates the **Plan 130 workspace pattern** with frontend/backend separation.

## Project Structure

```
api-example/
├── pac.at                        # Workspace root (scene: "workspace")
│
├── front/                        # AURA Frontend
│   ├── pac.at                    # Frontend config (scene: "ui")
│   └── app.at                    # Main app component
│
└── back/                         # Auto Backend
    ├── pac.at                    # Backend config (scene: "default")
    ├── api.at                    # API interface definitions
    ├── db.at                     # Database operations
    └── service.at                # Common utilities
```

## Config Files (Plan 130 Format)

### Workspace Root (`pac.at`)
```auto
name: "api-example"
version: "1.0.0"
scene: "workspace"

members: ["front", "back"]
```

### Frontend (`front/pac.at`)
```auto
name: "api-example-front"
version: "1.0.0"
scene: "ui"

// Multiple backend support:
// - "vue"  -> Vue SPA with HTTP backend
// - "tauri" -> Tauri desktop app with IPC backend
backend: ["vue", "tauri"]

// API types import (generates TypeScript client)
api: "../back/api.at"
```

### Backend (`back/pac.at`)
```auto
name: "api-example-back"
version: "1.0.0"
scene: "default"   // Default scene = native code (Rust backend)

backend: "rust"
entry: "api.at"
```

## Commands

### `auto build`
Builds the project based on selected backend:
- **Vue**: Generates Vue SPA + Rust HTTP server
- **Tauri**: Generates Tauri desktop app

### `auto run`
Runs development server:
- **Vue**: Starts Vite dev server + Rust backend
- **Tauri**: Starts Tauri dev (Vite + IPC backend)

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      front/ (AURA UI)                           │
│                                                                 │
│  app.at, userlist.at                                            │
│  └── import { api } from "../back/api.at"                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      back/ (Auto Rust)                          │
│                                                                 │
│  api.at   → #[api] annotations (interface)                      │
│  db.at    → Database operations (business logic)                │
│  service.at → Utilities (shared code)                           │
└─────────────────────────────────────────────────────────────────┘
                              │
            ┌─────────────────┴─────────────────┐
            ▼                                   ▼
┌─────────────────────┐              ┌─────────────────────┐
│  Vue + HTTP Mode    │              │  Tauri + IPC Mode   │
│                     │              │                     │
│  ┌─────────────┐    │              │  ┌─────────────┐    │
│  │ Vue SPA     │    │              │  │ Vue SPA     │    │
│  │ (Vite)      │    │              │  │ (Vite)      │    │
│  └─────────────┘    │              │  └─────────────┘    │
│         │           │              │         │           │
│         │ HTTP      │              │         │ IPC       │
│         ▼           │              │         ▼           │
│  ┌─────────────┐    │              │  ┌─────────────┐    │
│  │ Rust Server │    │              │  │ Tauri Core  │    │
│  │ (axum)      │    │              │  │ (#[command])│    │
│  └─────────────┘    │              │  └─────────────┘    │
│                     │              │                     │
│  Browser Access     │              │  Desktop App        │
└─────────────────────┘              └─────────────────────┘
```

## API Annotation

The `api.at` file defines the interface that works for both HTTP and IPC:

```auto
// Shared types
pub type User = {
    id: int
    name: str
    email: str
}

// API endpoint - generates both HTTP route and Tauri command
#[api(method = "GET", path = "/api/users/:id")]
pub fn get_user(id int) User? {
    use db
    return db.find_user(id)
}
```

### Generated Code

**HTTP Mode (axum):**
```rust
#[axum::debug_handler]
async fn get_user(
    Path(id): Path<i64>,
) -> Result<Json<User>, StatusCode> {
    // ...
}
```

**IPC Mode (tauri):**
```rust
#[tauri::command]
async fn get_user(id: i64) -> Result<Option<User>, String> {
    // ...
}
```

**Frontend Client (TypeScript):**
```typescript
// Works for both HTTP and IPC
export async function getUser(id: number): Promise<User | null> {
    // HTTP: fetch('/api/users/' + id)
    // IPC:  invoke('get_user', { id })
}
```

## Benefits

1. **Single Source of Truth**: Write business logic once in Auto
2. **Type Safety**: End-to-end type checking (Auto → Rust → TypeScript)
3. **Flexible Deployment**: Same code works as desktop app or web app
4. **Easy Maintenance**: Change backend, regenerate frontend types automatically
5. **Convention over Configuration**: Standard project structure with `pac.at`
