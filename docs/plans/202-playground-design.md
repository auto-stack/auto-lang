# Plan 202: Auto Playground

## Overview

A web-based playground for AutoLang — edit Auto code, run it via VM, and view transpiled output in Rust/C/TypeScript. Uses Vue 3 + shadcn-vue frontend with a Rust (axum) backend that directly reuses the existing auto-lang crate.

## Architecture

```
┌─────────────────────────────────────────────────┐
│                 Vue 3 Frontend                   │
│  ┌──────────────────┬──────────────────────────┐ │
│  │   CodeMirror     │   Output Panel           │ │
│  │   Auto Editor    │  ┌─────────────────────┐ │ │
│  │   (syntax hl)    │  │ [Run] [Rust][C][TS] │ │ │
│  │                  │  ├─────────────────────┤ │ │
│  │                  │  │ Output / Code View  │ │ │
│  │                  │  │                     │ │ │
│  └──────────────────┘  └─────────────────────┘ │
└──────────────────────┬──────────────────────────┘
                       │ HTTP API
                       ▼
┌─────────────────────────────────────────────────┐
│            Rust API Server (axum)                │
│  POST /api/run      → VM execute, return stdout  │
│  POST /api/trans    → transpile to target lang   │
│  GET  /api/examples → example code list          │
└─────────────────────────────────────────────────┘
```

## Project Structure

```
crates/auto-playground/
├── Cargo.toml              # deps: axum, tower, cors, auto-lang
├── src/                    # Rust backend
│   ├── main.rs             # axum server, CORS config
│   ├── routes/
│   │   ├── run.rs          # /api/run — VM engine execution
│   │   ├── trans.rs        # /api/trans — transpiler calls
│   │   └── examples.rs     # /api/examples — static example list
│   └── error.rs            # unified error handling
└── frontend/               # Vue 3 frontend
    ├── package.json
    ├── vite.config.ts
    └── src/
        ├── App.vue
        ├── components/
        │   ├── PlaygroundLayout.vue
        │   ├── CodeEditor.vue
        │   ├── OutputPanel.vue
        │   ├── OutputTabs.vue
        │   ├── ConsoleOutput.vue
        │   ├── CodePreview.vue
        │   └── ExampleSelector.vue
        ├── composables/
        │   └── usePlayground.ts
        └── types.ts
```

## API Design

### POST /api/run

Execute Auto code via VM.

```json
// Request
{ "source": "fn main() { print(\"hello\") }" }

// Response
{ "stdout": "hello\n", "stderr": "", "exit_code": 0, "time_ms": 12 }
```

### POST /api/trans

Transpile Auto code to a target language.

```json
// Request
{ "source": "...", "target": "rust" }   // target: "rust" | "c" | "typescript"

// Response
{ "code": "fn main() { ... }", "target": "rust" }
```

### GET /api/examples

List built-in example programs.

```json
// Response
{ "examples": [{ "name": "Hello World", "source": "print(\"hello\")" }, ...] }
```

## Backend Implementation

- **Crate**: `auto-playground` depends on `auto-lang`
- **Framework**: axum with tower CORS middleware (allow localhost for dev)
- **Core call chain**:
  - `/api/run` → `auto_lang::vm::engine` execute source, capture stdout
  - `/api/trans` → `auto_lang::trans` transpiler for target language
  - `/api/examples` → hardcoded list, future: scan `examples/` directory
- **Error handling**: Auto compile/runtime errors returned as JSON with line/column info when available

## Frontend Components

### PlaygroundLayout.vue
- Left/right split container using CSS flexbox
- Responsive: side-by-side on wide screens, stacked on narrow (<768px)
- Top toolbar: ExampleSelector (left) + Run button (right)

### CodeEditor.vue
- CodeMirror 6 with custom Auto language mode
- Dark theme (one dark)
- Syntax highlighting for Auto keywords: fn, let, var, const, for, if, else, is, type, enum, loop, break, use, pub, mut
- Shortcut: Ctrl+Enter triggers run

### OutputPanel.vue
- Container with OutputTabs and content area
- Manages active tab state

### OutputTabs.vue
- shadcn-vue Tabs: Console | Rust | C | TypeScript
- Console tab shows VM output
- Language tabs show transpiled code

### ConsoleOutput.vue
- Displays stdout/stderr from VM execution
- Monospace font, error lines in red
- Shows execution time

### CodePreview.vue
- Read-only code display with syntax highlighting
- Target-appropriate highlighting (Rust, C, TypeScript)
- Copy button (top-right corner)

### ExampleSelector.vue
- shadcn-vue Select dropdown
- Populates editor with selected example code

### usePlayground.ts composable
- Manages: source code, active tab, output state, loading state
- API calls via fetch
- Debounced transpile on tab switch (cache results)

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend framework | Vue 3 + TypeScript |
| UI components | shadcn-vue |
| Code editor | CodeMirror 6 |
| Code highlight (output) | highlight.js or CodeMirror readonly |
| Build tool | Vite |
| Backend framework | axum |
| CORS | tower HTTP CORS |
| Auto integration | auto-lang crate (direct dependency) |

## V1 Scope (Included)

- Code editing with Auto syntax highlighting
- Run code via VM, display console output
- Transpile and view Rust/C/TypeScript output
- Built-in example programs
- Keyboard shortcut (Ctrl+Enter to run)
- Copy code button on output panels

## V2 Scope (Deferred)

- UI live preview (render Auto UI code as web components)
- Shareable URLs (codepen-style)
- Multiple file support
- Auto-complete / IntelliSense
- Error annotations in editor (inline error markers)

## Implementation Strategy

1. Build Vue + shadcn-vue frontend first, validate UX
2. Build Rust API server, wire up to auto-lang crate
3. Integrate frontend with backend API
4. Iterate and polish
5. Once stable, translate Vue components back to AutoUI via a2vue understanding
