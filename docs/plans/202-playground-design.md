# Plan 202: Auto Playground

## Status: 🔧 PARTIAL

Verified 2026-04-23:
- ✅ crates/auto-playground/ with axum backend (Cargo.toml lists reqwest, axum)
- ✅ crates/auto-playground/frontend/ with Vue 3 (App.vue, PlaygroundLayout.vue, usePlayground.ts)
- ✅ Website VitePress component (website/.vitepress/theme/components/AutoPlayground.vue)
- ❌ V2 deferred features (SourceMap, live preview, shareable URLs, multi-file) not yet done

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

## SourceMap Integration (V2, API预留)

Playground 将在 V2 支持 Auto ↔ 目标代码的行级映射（参考 TypeScript Source Map v3）。
V1 的 API 预留 SourceMap 字段，前端先忽略。

### API 扩展

```json
// POST /api/trans — V1 返回
{ "code": "fn main() { ... }", "target": "rust" }

// POST /api/trans — V2 扩展（向后兼容）
{ "code": "fn main() { ... }", "target": "rust", "source_map": {
  "version": 3,
  "sources": ["input.at"],
  "mappings": "AAAA;AACA,SAAS..."
}}
```

### 前端预留

- CodeEditor 和 CodePreview 组件预留 `highlightLine(n)` 接口
- V2 实现：点击 Auto 代码某行 → 高亮对应目标代码行（双向联动）
- 运行错误映射回 Auto 源码行并高亮

### 依赖

- 需要 Plan 032（Source Mapping System）先实现 transpiler 位置追踪
- C 目标额外支持 `#line` 指令（Plan 032 已设计）

## V2 Scope (Deferred)

- **SourceMap 行级映射**（上述）
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
