# Plan 181: a2vscode — Auto UI to VSCode Extension Generator

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add `"vscode"` as a UI backend that generates a complete VSCode extension project from Auto UI widgets. The extension renders the app widget in a sidebar webview panel using a2vue-generated Vue 3 content.

**Architecture:** Pipeline transpiler — reads AURA widgets, delegates UI rendering to the existing a2vue generator, and wraps the output in a VSCode extension scaffold (package.json, extension.ts, webview panel). IPC messaging uses the same AURA messages model that a2tauri will use, with `postMessage` as the transport.

**Tech Stack:** Rust, AURA pipeline, existing VueGenerator, VSCode Extension API (raw Webview).

**Source patterns:** `crates/auto-man/src/vue.rs` (backend module), `crates/auto-lang/src/ui_gen/vue.rs` (Vue generator).

---

## Design

### pac.at Configuration

The `vscode` backend is added alongside existing backends. A `vscode {}` config block controls extension properties:

```auto
name: "my-tool"
version: "1.0.0"
scene: "ui"
backend: ["vue", "vscode"]

vscode {
    panel: sidebar              // sidebar | editor
    command: "myTool.open"      // VSCode command ID
    title: "My Tool"            // Panel display title
    icon: "icon.png"            // Panel icon (optional, relative to project root)
}
```

All fields are optional with sensible defaults:
- `panel`: defaults to `sidebar`
- `command`: defaults to `<name>.open` (derived from project name, kebab-cased)
- `title`: defaults to project `name`
- `icon`: defaults to no icon

### Generated Project Structure

```
<project>/vscode/
├── package.json                # Extension manifest
├── tsconfig.json               # TypeScript config
├── webpack.config.js           # Bundling (extension + webview)
├── src/
│   ├── extension.ts            # Entry: activate, register command
│   └── panels/
│       └── AppPanel.ts         # Webview panel: HTML, resource URIs, messaging
├── webview-ui/                 # a2vue output
│   ├── index.html              # Shell HTML with VSCode CSP headers
│   ├── src/
│   │   ├── App.vue             # Generated from app widget
│   │   ├── main.ts             # Vue bootstrap
│   │   └── components/         # Sub-widget components
│   └── package.json            # Webview dependencies (vue, tailwind)
├── media/
│   └── icon.png                # Extension icon (copied from config)
└── .vscodeignore
```

### Transpiler Architecture

```
pac.at config ──→ vscode.rs ──→ package.json, extension.ts, AppPanel.ts
                                   │
AURA widgets ──→ VueGenerator ──→ webview-ui/ (Vue app)
```

`vscode.rs` does NOT reimplement UI rendering. It:
1. Parses `pac.at` for the `vscode {}` config block
2. Extracts AURA widgets from `.at` files
3. Calls `VueGenerator` to produce the webview Vue app
4. Generates the VSCode extension scaffold around that output
5. Injects VSCode CSP headers and resource URI rewriting into the webview HTML

### IPC Messaging (Shared with a2tauri Model)

AURA widgets already define `messages` and `handlers`. a2vscode generates the IPC bridge:

**Auto side (already exists in AURA):**
```auto
widget App {
    messages {
        FileOpened(path str)
        SaveRequested(content str)
    }
}
```

**VSCode generated code:**
- Webview: `vscode.postMessage({ type: "FileOpened", data: { path } })` wrapper + listener
- Extension: message dispatcher in `AppPanel.ts` that routes to handlers

**Tauri (future a2tauri) equivalent:**
- Frontend: `invoke("file_opened", { path })` wrapper
- Backend: `#[tauri::command]` handler

Same AURA message definitions, different IPC transport. The message format is platform-independent; only the glue code varies.

### Scope

| What | Status |
|---|---|
| pac.at `vscode {}` config block | Now |
| Sidebar panel with full app | Now |
| a2vue reuse for webview content | Now |
| Full extension project generation (package.json, extension.ts, AppPanel.ts, webpack) | Now |
| IPC messaging bridge (AURA messages → postMessage) | Now |
| Per-widget placement via annotations | Future |
| Multi-panel views | Future |

---

## Implementation Plan

### Phase 1: VSCode Config Parsing

#### Task 1: Add `vscode` to `BackendType` enum

**Files:**
- Modify: `crates/auto-man/src/lib.rs` (add `Vscode` variant to `BackendType`)

Add `Vscode` to the existing `BackendType` enum alongside `Vue`, `Jet`, `Arkts`, `Rust`.

#### Task 2: Parse `vscode {}` config block from pac.at

**Files:**
- Modify: `crates/auto-man/src/lib.rs` or the pac.at parser module

Add a `VscodeConfig` struct and parsing logic:
```rust
struct VscodeConfig {
    panel: String,      // "sidebar" | "editor", default "sidebar"
    command: String,    // default "<name>.open"
    title: String,      // default from project name
    icon: Option<String>,
}
```

### Phase 2: Extension Scaffold Generator

#### Task 3: Create `crates/auto-man/src/vscode.rs` — Core Generation Module

**Files:**
- Create: `crates/auto-man/src/vscode.rs`
- Modify: `crates/auto-man/src/lib.rs` (add `pub mod vscode;`)

This module follows the `ark.rs` / `vue.rs` pattern. It contains:

1. `generate_vscode(project_dir, output_dir, project)` — main entry point
2. `generate_package_json(config, widgets)` — produce extension manifest
3. `generate_extension_ts(config)` — produce extension entry point
4. `generate_app_panel_ts(config)` — produce webview panel class
5. `generate_index_html(config)` — produce webview shell HTML with CSP
6. `generate_webpack_config(config)` — produce bundling config
7. `generate_tsconfig()` — produce TypeScript config
8. `generate_vscodeignore()` — produce .vscodeignore

The `generate_vscode` function:
- Reads `.at` files from `front/` directory
- Extracts AURA widgets
- Calls `VueGenerator` to produce webview content
- Generates all scaffold files
- Writes to `<project>/vscode/` output directory

#### Task 4: Wire `BackendType::Vscode` into auto-man CLI

**Files:**
- Modify: `crates/auto-man/src/lib.rs` (gen/build/run dispatch)

Add `BackendType::Vscode` cases to the `gen()`, `build()`, and `run()` methods. For `run()`, output instructions for `code --extensionDevelopmentPath=<path>` since VSCode extensions can't be launched directly from CLI.

### Phase 3: Webview Integration

#### Task 5: Generate webview HTML with VSCode CSP

**Files:**
- Modify: `crates/auto-man/src/vscode.rs`

Generate an `index.html` that:
- Sets VSCode Content Security Policy headers
- Loads Vue app from bundled assets
- Includes `vscode.postMessage` bridge stub
- Uses `vscode-webview-resource` URIs for local resources

#### Task 6: Generate AppPanel.ts with messaging bridge

**Files:**
- Modify: `crates/auto-man/src/vscode.rs`

Generate `AppPanel.ts` that:
- Creates a webview panel (sidebar or editor based on config)
- Sets HTML content with proper resource URI rewriting
- Includes `postMessage` listener/dispatcher stub
- Includes message-to-handler routing for AURA messages

### Phase 4: Tests & Examples

#### Task 7: Add a2vscode test cases

**Files:**
- Create: `crates/auto-lang/test/a2vscode/` directory
- Create test cases following existing transpiler test pattern

Test structure per case:
```
a2vscode/
└── 001_sidebar_app/
    ├── input.at           # Auto UI widget
    └── expected/           # Generated files
        ├── package.json
        ├── src/extension.ts
        ├── src/panels/AppPanel.ts
        └── webview-ui/index.html
```

Start with 2-3 tests:
- `001_sidebar_app` — basic sidebar panel with default config
- `002_editor_panel` — editor panel with explicit config
- `003_with_messages` — widget with AURA messages (IPC bridge generated)

#### Task 8: Add example project

**Files:**
- Create: `examples/vscode-demo/` following `examples/unified-demo/` pattern

A minimal example with pac.at, one widget, and vscode backend configured.

### Phase 5: Integration

#### Task 9: Add `"vscode"` to `auto new` template options

**Files:**
- Modify: `crates/auto-man/src/new.rs` or template system

When creating a new project with `auto new --backend vscode`, scaffold includes vscode config.

#### Task 10: Update CLI help and documentation

**Files:**
- Modify relevant CLI help text to mention `vscode` backend
