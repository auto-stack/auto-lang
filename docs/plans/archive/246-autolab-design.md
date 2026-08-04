# AutoLab — AI-Native Interactive Notebook Design

## Status: ✅ 已完成 — All 5 phases implemented (Playground extraction, Frontend skeleton, Cell types + AI, Polish + Deploy, Quality + AI experience)

## Overview

AutoLab is an AI-native interactive notebook development environment for the Auto programming language. It draws inspiration from Jupyter Notebook's cell-based interactivity, Claude Code / Codex's AI-assisted workflow, and is purpose-built for the AI era. The entire product is built with Auto's own ecosystem: Vue frontend (future: Auto-generated via a2r), Rust backend calling the Auto compiler and VM.

**Codename**: AutoLab

**File extension**: `.ad` (AutoDown format)

---

## 1. Architecture Overview

### Three-Layer Architecture

```
┌─────────────────────────────────────────────────────────┐
│  auto-lab-ui (Vue 3 SPA)                                │
│  ┌───────────┐ ┌──────────────┐ ┌────────────────────┐  │
│  │ Cell      │ │ Variable     │ │ AI Chat            │  │
│  │ Canvas    │ │ Inspector    │ │ Input Bar          │  │
│  └─────┬─────┘ └──────┬───────┘ └─────────┬──────────┘  │
│        │ shared composable layer           │             │
│  ┌─────┴───────────────────────────────────┴──────────┐  │
│  │  auto-playground-vue (standalone Vue package)      │  │
│  │  CodeMirror 6 Editor, Output Panel, Code Preview   │  │
│  └────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────┤
│  auto-lab Backend (Rust / Axum)                          │
│  ┌──────────────┐ ┌──────────────┐ ┌─────────────────┐  │
│  │ Notebook     │ │ VM           │ │ AI              │  │
│  │ Session Mgr  │ │ Runner       │ │ Provider        │  │
│  └──────┬───────┘ └──────┬───────┘ └────────┬────────┘  │
│  ┌──────┴─────────────────┴─────────────────┴────────┐  │
│  │  auto-lang (compiler, AutoVM, transpilers,         │  │
│  │             autodown parser)                       │  │
│  └────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────┤
│  File System (.ad AutoDown files)                        │
└─────────────────────────────────────────────────────────┘
```

### Module Responsibilities

| Layer | Module | Responsibility |
|-------|--------|----------------|
| Frontend | `auto-playground-vue` | Standalone Vue component package: CodeMirror editor, Auto language mode, output rendering, transpile preview |
| Frontend | Cell Canvas | Multi-cell list: add/delete/reorder, collapse/expand, execution status indicators |
| Frontend | AI Chat Input | Persistent bottom input bar, submit → AI Cell → append to cell stream |
| Frontend | Variable Inspector | Side panel showing live variable tree from current session |
| Backend | Notebook Session Manager | Manage multiple active notebook sessions, each holding a long-lived VM |
| Backend | VM Runner | Stateful execution: cells share VM state (variables, function definitions) |
| Backend | AI Provider | Claude/OpenAI API integration for code generation, explanation |
| File | AutoDown (.ad) | Human-readable, Git-friendly notebook persistence format |

---

## 2. Frontend Component Architecture

### Component Tree

```
<AutoLabApp>                         ← Top-level, routing + global state
├── <NotebookToolbar>                ← File operations, run-all, settings
│   ├── <FileMenu>                   ← New / Open / Save .ad
│   └── <RunAllButton>              ← Sequential execution of all cells
│
├── <CellCanvas>                     ← Scrollable cell container, drag-sort
│   └── <CellItem v-for="cell">     ← Single cell, reusable core
│       ├── <CellToolbar>           ← Type icon, run button, collapse, delete
│       ├── <CellEditor>            ← Embeds auto-playground-vue editor
│       │   └── <CodeMirrorEditor>  ← Atomic editor component
│       ├── <CellOutput>            ← Execution result render area
│       │   ├── <OutputText>        ← stdout / text results
│       │   ├── <OutputChart>       ← Extensible: chart rendering (registered)
│       │   ├── <OutputTable>       ← Extensible: table rendering (registered)
│       │   └── <OutputError>       ← Compile / runtime errors
│       └── <CellTypeBadge>         ← Auto / Markdown / AI / Chart
│
├── <AIChatBar>                      ← Persistent bottom input bar
│   ├── <ChatInput>                 ← Text input + context preview
│   └── <ContextChips>              ← Selected context cell tags
│
└── <SidePanel>                      ← Right-side variable panel
    ├── <VariableInspector>         ← Current session variable tree
    ├── <FileExplorer>              ← .ad file browser
    └── <CellTypeRegistry>          ← Registered cell type list
```

### Component Layering Principle

Each layer maps to a future Auto AURA construct, enabling incremental a2r transpilation:

| Layer | Package | Naming | Responsibility | Maps to AURA |
|-------|---------|--------|----------------|--------------|
| **Atoms** | `@auto-lab/ui-atoms` | `CodeMirrorEditor`, `OutputText`, `ChatInput` | No business logic, pure UI primitives | AURA Element (single widget) |
| **Composites** | `@auto-lab/ui-cells` | `CellEditor`, `CellOutput`, `VariableInspector` | Combine atoms + single business capability | AURA Component (widget composition) |
| **Layout** | `@auto-lab/ui-layout` | `CellCanvas`, `SidePanel`, `NotebookToolbar` | Layout containers orchestrating composites | AURA Layout (container widget) |
| **App Shell** | App-level | `AutoLabApp`, `AIChatBar` | Top-level state + routing | App-level |

### Vue ↔ Auto AURA Mapping

```
Vue atomic component   →  AURA Element  (single widget)
Vue composite component →  AURA Component (widget composition)
Vue composable         →  AURA Hook      (state logic)
Vue layout component   →  AURA Layout    (container widget)
```

This fine-grained decomposition ensures that when a2r matures, each layer can be independently reverse-compiled — atoms first as the easiest targets, composites and layouts progressively.

---

## 3. Backend Session & VM Lifecycle

### The Core Problem

The existing `auto-playground` is **stateless**: every run creates a fresh VM, compiles, executes, and discards. AutoLab requires **stateful** execution — cells share VM state, and later cells access variables defined by earlier cells.

### Session Model

```
Notebook Session (per .ad file)
├── session_id: uuid
├── vm: long-lived AutoVM
├── cells: Vec<CellState>
│   ├── cell_id, source, type, status, output
│   └── compiled_code: Option<ABT>
├── var_snapshot: HashMap<String, VarInfo>
└── created_at, last_active
```

### Cell Execution Strategy

**Single VM, incremental compilation and execution**:

```
Receive "Execute Cell 3" request
  → Frontend sends session info + Cell 3 source
  → Backend re-compiles Cells 1, 2, 3 in order (VM already has 1, 2 state)
     → Optimized path: if Cells 1, 2 are unchanged and cached, skip recompilation, only run
  → Execute Cell 3, capture stdout + return value
  → Update var_snapshot (new/modified variables)
  → Return CellOutput to frontend
```

### Cross-Cell State Sharing

The VM is NOT destroyed between cells. All top-level variables and function definitions remain in the VM's global scope:

```
Cell 1: var x = 42           → VM globals: x = 42
Cell 2: var y = x + 1        → VM globals: x = 42, y = 43
Cell 3: fn add(a, b) { a+b } → VM globals: x = 42, y = 43, add = fn
```

### Dirty Cell Auto-Reexecution

When a user modifies Cell N and then executes Cell M (M > N), the backend must handle stale state.

**Chosen strategy: dependency-chain incremental re-execution (Strategy 1)**

- Track cell dependency graph
- When Cell N is modified, mark all downstream cells as "dirty"
- On next execution request, re-execute from the first dirty cell through the target cell
- Unmodified upstream cells are NOT re-executed

### Session Lifecycle

| State | Trigger | Behavior |
|-------|---------|----------|
| `Created` | Open / new .ad file | Empty session, wait for first execution |
| `Active` | After any cell executes | VM stays alive, wait for more executions |
| `Idle` | 5 min inactivity | VM suspended (variable snapshot retained), release compute resources |
| `Closed` | File closed / timeout | VM destroyed, var_snapshot discarded |

### Backend API Endpoints

```
POST   /api/notebook/session           → Create session, return session_id
POST   /api/notebook/{sid}/execute     → Execute cell, return output + var_snapshot diff
GET    /api/notebook/{sid}/variables   → Get current variable list
POST   /api/notebook/{sid}/transpile   → Reuse existing transpile pipeline
DELETE /api/notebook/{sid}             → Destroy session
POST   /api/notebook/{sid}/ai          → AI request (code generation / explanation)
POST   /api/notebook/{sid}/ai-stream   → AI streaming request (SSE, real-time token output)
```

---

## 4. AutoDown Notebook File Format

### Background: Existing AutoDown Implementation

AutoDown is already implemented in `crates/auto-lang/src/autodown/`:

```
Lexer (mode-aware: Text/Code/Math)
  → Parser (Flip mechanism)
    → ADOC AST (AdocDocument → AdocSection → AdocBlock → AdocInline)
      → Typst transpiler (implemented)
      → HTML transpiler (implemented)
      → DOCX transpiler (planned)
```

**Core syntax — "Three Symbol Domains":**

| Symbol | Domain | Example |
|--------|--------|---------|
| `#` | Heading domain | `# Title`, `## Section` |
| `$` | Logic domain (Auto code takeover via Flip) | `$var x = 42`, `${expr}` interpolation |
| `%{...}` | Math domain (AutoMath) | `%{ E = m * c^2 }` |

### Notebook Extension: Cell Directive Convention

To add notebook cell boundaries without breaking AutoDown compatibility, introduce a lightweight `/// cell:` comment directive. Standard AutoDown toolchain ignores these as comments; AutoLab parses them to build the cell model.

```autodown
/// cell:c1 type:code
# Data Loading

$var data = load_csv("data.csv")
$print(f"Loaded ${data.len} rows")

/// cell:c2 type:code depends_on:c1
# Data Analysis

$var result = data |> filter(x -> x > 0)
$print(f"Result: ${result}")

/// cell:c3 type:chart depends_on:c2
# Visualization

$Chart(type: "bar", data: result, title: "Results") {
    Analysis results visualization
}
```

### Cell Metadata Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `cell` | Yes | Unique ID (`c1`, `c2`...) for dependency tracking |
| `type` | Yes | `code` / `markdown` / `ai` / `chart` / `table` / custom |
| `depends_on` | No | Comma-separated dependent cell IDs. If omitted, depends on all preceding code cells |
| `output` | No | Cached output (populated at runtime, never hand-written in source) |

### Design Principles

- **Humans are the first readers**: Pure AutoDown, readable in any Markdown viewer or GitHub
- **AutoDown native**: Standard AutoDown toolchain (lexer → parser → Typst/HTML) works unchanged
- **Progressive enhancement**: Without `/// cell:` directives, it's just a regular `.ad` file
- **Git diff friendly**: Each cell is isolated; editing one doesn't affect others
- **AI friendly**: Plain text format, Claude/GPT can read and write directly

### Output Caching

The `.ad` source file stores only source code. Execution outputs are cached in a sibling `.autolab/` directory, excluded from version control via `.gitignore`. The `output` attribute in cell directives is auto-populated at runtime and serialized to the cache, never committed to the source file.

---

## 5. AI Integration Design

### Interaction Flow

```
User → [AIChatBar] inputs "Write bubble sort in Auto"
  → Backend POST /api/notebook/{sid}/ai
    → Assemble context: current session variables + all preceding cell sources
    → Send to AI Provider (Claude / OpenAI)
    → AI returns code + explanation
  → Frontend receives, appends two new cells to CellCanvas:
    ├── AI Request Cell (type: ai, role: user)
    │   └── Content: user's original question
    └── AI Response Cell (type: ai, role: assistant)
        └── Content: AI-generated code + explanation
```

### AI Cell Behaviors

- **AI-generated code is NOT auto-executed** — user must review first, then click "Run" on the cell
- **Code extraction**: user can right-click an AI response cell and "Extract as code cell" to promote generated code into an independent, executable code cell
- **Configurable context**: user selects which cells to include as AI conversation context (via `ContextChips` tags). Default: all preceding cells

### AI Provider Abstraction

Minimal backend abstraction, initially only Claude API:

```rust
trait AiProvider {
    fn chat(&self, messages: Vec<Message>, context: NotebookContext) -> String;
}
```

- Default provider: Claude API (via `anthropic` SDK)
- User-configurable API key (environment variable or settings panel)
- Prompt includes Auto language syntax rules extracted from CLAUDE.md to reduce syntax errors in generated code

### The AIChatBar Model

A persistent input bar at the bottom of the notebook — similar to Claude Code / Codex's chat input. On submission:
1. The input content becomes an AI Request cell in the cell stream
2. The input bar clears, ready for the next input
3. The AI response arrives as a new AI Response cell
4. The entire conversation becomes part of the notebook's "human-AI collaboration narrative"

---

## 6. Cell Type Registration System

### Design Goal

Users can register new cell types (Chart, Table, Diagram, 3D View...), each with its own editor UI and output renderer. Registration is primarily frontend behavior (listed in SidePanel), but new types can pair with backend handlers.

### Cell Type Interface

```typescript
interface CellType {
  id: string                    // "chart", "table", "mermaid", ...
  label: string                 // Display name
  icon: string                  // lucide icon name
  defaultSource: string         // Default content template for new cells
  editorComponent?: Component   // Optional: dedicated editor (defaults to CodeMirror)
  outputComponent?: Component   // Optional: dedicated output renderer (defaults to text)
  languageMode?: string         // CodeMirror language mode (defaults to "auto")
  onExecute?: (source: string, session: Session) => Output  // Backend execution hook
}
```

### Built-in Types

| Type | Editor | Output Rendered | Description |
|------|--------|-----------------|-------------|
| `code` | CodeMirror (auto mode) | Text / stdout | Standard Auto code execution |
| `markdown` | Rich text / Markdown editor | Rendered HTML | Via AutoDown renderer |
| `ai` | Read-only chat bubble | Formatted conversation | AI dialogue record |
| `chart` | CodeMirror (auto mode, Chart DSL) | Canvas / SVG chart | Via `$Chart(...)` call |
| `table` | CodeMirror + table editor | Sortable HTML table | Data display |

### Registration Flow

```
SidePanel → CellTypeRegistry → shows registered types → user clicks "+Custom Type"
  → Form: id, label, defaultSource, outputComponent (optional)
  → Save to user local config
  → New type appears in dropdown when creating cells
```

### Backend Relationship

Most cell types are purely frontend rendered (Chart via ECharts/Chart.js, Table via HTML table). Special types can specify `onExecute` to invoke backend handlers — the backend `/api/notebook/{sid}/execute` dispatches to different executors based on `cell.type` (VM execution / AI request / data analysis, etc.).

### Future Extension

When a2r matures, cell type definitions themselves can be written as `.at` files, compiled via Auto into Vue components for registration. This closes the loop of "using Auto to extend AutoLab."

---

## 7. Project Structure

```
d:\autostack\auto-lang\
├── crates/
│   ├── auto-lang/                          # Existing: compiler core
│   │   └── src/
│   │       └── autodown/                   # Existing: AutoDown parser + transpilers
│   │           ├── mod.rs
│   │           ├── lexer.rs                # Text/Code/Math mode-aware lexer
│   │           ├── ast.rs                  # AdocDocument, AdocBlock, AdocInline
│   │           ├── parser.rs               # Flip mechanism parser
│   │           ├── math.rs                 # AutoMath parser
│   │           ├── error.rs
│   │           └── trans/
│   │               ├── mod.rs
│   │               ├── typst.rs            # Typst code generator
│   │               └── html.rs             # HTML code generator
│   │
│   ├── auto-playground/                    # Existing: refactored to pure backend
│   │   └── src/
│   │       ├── main.rs                     # Axum server entry
│   │       ├── routes/                     # Existing: run, trans, debug...
│   │       └── notebook/                   # NEW: notebook session management
│   │           ├── mod.rs
│   │           ├── session.rs              # Session lifecycle
│   │           ├── executor.rs             # Incremental cell execution
│   │           └── ai.rs                   # AI Provider integration
│   │
│   └── auto-lab/                           # NEW: AutoLab backend crate
│       └── src/
│           └── main.rs                     # Axum server (reuses playground logic)
│
├── packages/                               # NEW: frontend monorepo
│   ├── auto-playground-vue/                # Extracted standalone Vue component package
│   │   ├── src/
│   │   │   ├── components/
│   │   │   │   ├── CodeMirrorEditor.vue
│   │   │   │   ├── OutputPanel.vue
│   │   │   │   └── CodePreview.vue
│   │   │   ├── composables/
│   │   │   │   ├── usePlayground.ts
│   │   │   │   └── autoLang.ts            # Unified Auto language mode
│   │   │   └── index.ts                   # Public exports
│   │   └── package.json
│   │
│   └── auto-lab-ui/                        # AutoLab frontend SPA
│       ├── src/
│       │   ├── App.vue
│       │   ├── components/
│       │   │   ├── atoms/                  # Atomic components
│       │   │   │   ├── CodeMirrorEditor.vue    (re-export from playground-vue)
│       │   │   │   ├── ChatInput.vue
│       │   │   │   └── OutputText.vue
│       │   │   ├── cells/                  # Cell composite components
│       │   │   │   ├── CellItem.vue
│       │   │   │   ├── CellToolbar.vue
│       │   │   │   ├── CellEditor.vue
│       │   │   │   ├── CellOutput.vue
│       │   │   │   └── CellTypeBadge.vue
│       │   │   ├── layout/                 # Layout components
│       │   │   │   ├── CellCanvas.vue
│       │   │   │   ├── SidePanel.vue
│       │   │   │   └── NotebookToolbar.vue
│       │   │   └── notebook/               # Notebook-specific components
│       │   │       ├── AIChatBar.vue
│       │   │       ├── VariableInspector.vue
│       │   │       ├── FileExplorer.vue
│       │   │       └── CellTypeRegistry.vue
│       │   ├── composables/
│       │   │   ├── useNotebook.ts          # Notebook state management
│       │   │   └── useAI.ts               # AI interaction state
│       │   └── types/
│       │       └── cell.ts                 # CellType interface definitions
│       └── package.json
│
└── docs/
    └── plans/
        └── 246-autolab-design.md           # This document
```

---

## 8. Implementation Phases

### Phase 1: Playground Component Extraction + Notebook Backend Prototype ✅ COMPLETE

| # | Task | Status | Input | Output |
|---|------|--------|-------|--------|
| 1.1 | Extract core editor from `auto-playground/frontend/` into standalone `auto-playground-vue` package | ✅ Done | Existing CodeMirrorEditor, autoLang.ts | Publishable Vue component |
| 1.2 | Merge website and playground's two Auto language mode definitions | ✅ Done | Two versions of autoLang.ts | Unified language mode (`packages/auto-playground-vue/src/lang/auto.ts`) |
| 1.3 | Website switches to package import instead of inline component | ✅ Done | auto-playground-vue | No more iframe |
| 1.4 | Implement Notebook Session backend (single VM, multi-cell sequential execution) | ✅ Done | auto-playground backend | `/api/notebook/*` endpoints (`crates/auto-playground/src/notebook/mod.rs`, `routes/notebook.rs`) |
| 1.5 | AutoDown cell directive parsing (`/// cell:` → AdocAST extension) | ✅ Done | autodown module | Cell metadata extraction (`crates/auto-lang/src/autodown/cell.rs`) |

---

### Phase 2: AutoLab Frontend Skeleton 🔄 IN PROGRESS

### Phase 2: AutoLab Frontend Skeleton ✅ COMPLETE

| # | Task | Status | Input | Output |
|---|------|--------|-------|--------|
| 2.1 | Scaffold `auto-lab-ui` Vite + Vue 3 project, import `auto-playground-vue` | ✅ Done | Phase 1 components | `packages/auto-lab-ui/` with Vite + Vue 3 + TypeScript |
| 2.2 | Implement `CellCanvas` + `CellItem` (add/delete/reorder, collapse, status) | ✅ Done | Design spec | Multi-cell management with toolbar controls |
| 2.3 | Implement `AIChatBar` (bottom input, submit → cell stream) | ✅ Done | AI Provider | Persistent bottom chat bar appending AI cells |
| 2.4 | Implement `VariableInspector` (side panel variable tree) | ✅ Done | Session var_snapshot | Side panel with variable list + cell overview |
| 2.5 | Implement `.ad` notebook file read/write (load/save/autosave) | ✅ Done | AutoDown parser | `loadFromAd` / `serializeToAd` / `saveToFile` / `loadFromFile` in `useNotebook.ts` |

### Phase 3: Cell Type System + AI Integration ✅ COMPLETE

| # | Task | Status | Input | Output |
|---|------|--------|-------|--------|
| 3.1 | Implement cell type registration system (built-in types + extension mechanism) | ✅ Done | CellType interface | Chart/Table display (`OutputChart.vue`, `OutputTable.vue`), `table` type added |
| 3.2 | Integrate Claude API (backend AI Provider + frontend interaction) | ✅ Done | `reqwest` | `ClaudeProvider` in `notebook/ai.rs`, `/api/notebook/{sid}/ai`, frontend `askAI()` |
| 3.3 | Dependency-chain incremental re-execution (modify cell → mark dirty → rerun downstream) | ✅ Done | `notebook/mod.rs` | `cell_snapshots` + cascade dirty tracking + upstream re-execution queue |

### Phase 4: Polish + Deploy ✅ COMPLETE

| # | Task | Status | Input | Output |
|---|------|--------|-------|--------|
| 4.1 | Error diagnostics enhancement (compile errors mapped to specific cells) | ✅ Done | `CellOutput` | `diagnostics: Diagnostic[]` with line extraction + error line highlight in CodeEditor |
| 4.2 | Session suspend/resume (idle VM → variable snapshot → rebuild) | ✅ Done (simplified) | `session.rs` | `SessionStatus` enum (Active/Idle/Closed), `GET /api/notebook/{sid}/status`, 30s frontend polling |
| 4.3 | Deploy to playground server (add Nginx routing) | ✅ Done | `deploy/` | `/lab` route in nginx + backend `nest_service("/lab", ...)` for auto-lab-ui dist |

---

### Phase 5: Quality + AI Experience ✅ COMPLETE

| # | Task | Status | Input | Output |
|---|------|--------|-------|--------|
| 5.1 | Add test coverage (backend notebook + frontend composables) | ✅ Done | Existing code | `cargo test` (11 tests in `notebook/mod.rs`), Vitest (14 tests in `useNotebook.spec.ts`) |
| 5.2 | AI streaming output (SSE instead of blocking JSON) | ✅ Done | `ClaudeProvider` | `/api/notebook/{sid}/ai-stream` SSE endpoint + frontend `askAIStream()` via EventSource |
| 5.3 | One-click code extraction (AI response → executable code cell) | ✅ Done | AI Cell UI | `extractCodeFromAI()` in `useNotebook.ts` + "Extract code" toolbar button in `CellToolbar.vue` |

---

## 9. Design Decisions Summary

| Decision | Options Considered | Chosen | Rationale |
|----------|-------------------|--------|-----------|
| Frontend approach | A: Direct Vue + CodeMirror / B: a2r-generated Vue | **A** (B as future milestone) | Fast delivery; a2r has too many feature gaps for complex UI |
| Cell type model | A: Fixed types / B: Extensible registry | **B** | Future-proof; users can add Chart, Table, custom types |
| AI interaction | A: Sidebar Copilot / B: Agent mode / C: Chat input → Cell stream | **C** | Persistent input bar + conversation as notebook cells |
| Cell execution strategy | A: Dependency chain rerun / B: Full rerun / C: Manual only | **A** | Smart but conservative — only rerun what's needed |
| Notebook file format | A: JSON (.ipynb-like) / B: AutoDown / C: Standard Auto | **B** (AutoDown with cell directives) | Human-readable, existing AutoDown infrastructure, Git friendly |
| Dirty cell re-execution | A: Dependency chain only / B: Rerun all from modified / C: Manual | **A** | Precise; rerun only dirty cells and their dependents |
| Frontend component strategy | A: Monolithic / B: Layered atoms-composites-layout | **B** | Enables incremental a2r reverse-compilation per layer |
| VM execution model | Stateless (current) → Stateful (required) | **Stateful** | Cells must share variables across a session |
