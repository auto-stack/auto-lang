# 14 - Developer Tools

## Status

**Designed:**
- Compiler-native LSP (Language Server Protocol) implementation
- Web Playground (Vue 3 + axum backend)
- AutoLab AI Notebook
- AutoVM MCP Server for AI agents

**Partial:**
- LSP basic implementation exists in `crates/auto-lsp/`
- VSCode extension exists in `editors/vscode/`

**Planned:**
- Full LSP feature parity (hover, goto definition, rename, code actions)
- Web playground deployment
- AutoLab notebook implementation
- MCP server for AI agent integration

## Design

### LSP (Language Server Protocol)

Auto implements a compiler-native LSP server that shares the same parser, type checker, and AST as the main compiler — no duplicate parsing logic.

**Architecture:**

```
┌──────────────┐     LSP Protocol     ┌──────────────────┐
│  VSCode /    │ ←──────────────────→ │  auto-lsp        │
│  Neovim /    │    stdin/stdout       │  (Rust binary)   │
│  Any editor  │                       │                  │
└──────────────┘                       │  ┌────────────┐  │
                                       │  │ Parser     │  │
                                       │  │ TypeStore  │  │
                                       │  │ Inferencer │  │
                                       │  │ Database   │  │
                                       │  └────────────┘  │
                                       └──────────────────┘
```

**Key Design Decisions:**

1. **Shared Parser**: The LSP uses the exact same `lexer.rs` and `parser.rs` as the compiler. No separate "lenient" parser — parse errors are reported as diagnostics, and the last valid AST is cached for partial analysis.

2. **Incremental Analysis**: Leveraging the AIE (Auto Incremental Engine) architecture:
   - `Database` stores parsed fragments, symbol tables, and dependency graphs
   - `CompileSession` manages incremental recompilation on file changes
   - Content hash-based invalidation ensures only changed files are re-analyzed

3. **UTF-16 Handling**: LSP protocol uses UTF-16 offsets internally. The server maintains a UTF-16 offset table for each open document to convert between Rust's UTF-8 `str` offsets and LSP's UTF-16 positions.

4. **Workspace Caching**: On initialization, the LSP scans the workspace, parses all `.at` files, and populates the `TypeStore` with cross-file type information. Subsequent operations (hover, goto definition) read from this cache.

**Feature Matrix:**

| Feature | Status | Notes |
|---------|--------|-------|
| Syntax highlighting (semantic tokens) | ✅ | Via semantic token provider |
| Diagnostics (parse errors) | ✅ | Real-time on file change |
| Diagnostics (type errors) | ⚠️ | Partial — inference integration pending |
| Hover (type info) | ⚠️ | Works for simple expressions |
| Go to definition | ⚠️ | Works within single file |
| Cross-file references | ❌ | Requires Database integration |
| Rename | ❌ | Requires cross-file reference resolution |
| Code actions | ❌ | Requires AST transformation framework |
| Completion | ⚠️ | Basic keyword completion |
| Signature help | ❌ | Requires function parameter tracking |

### Web Playground

A browser-based Auto code execution environment.

**Architecture:**
```
┌──────────────────┐     HTTP/WS      ┌──────────────────┐
│  Vue 3 Frontend  │ ←──────────────→ │  axum Backend    │
│                  │                   │                  │
│  Monaco Editor   │                   │  AutoVM sandbox  │
│  Output panel    │                   │  Execution limit │
│  Example gallery │                   │  File system     │
└──────────────────┘                   └──────────────────┘
```

**Frontend (Vue 3):**
- Monaco editor with Auto language support (syntax highlighting, basic completion)
- Real-time output panel (stdout, stderr, return value)
- Example gallery with pre-loaded Auto programs
- Share button (encodes program in URL hash)

**Backend (axum):**
- Sandboxed AutoVM execution with resource limits:
  - Max execution time: 5 seconds
  - Max memory: 64 MB
  - No file system access (virtual FS)
  - No network access
- REST API: `POST /execute` with `{ code: string }` → `{ output: string, error?: string }`
- WebSocket API for streaming output (REPL mode)

### AutoLab AI Notebook

A Jupyter-like notebook environment optimized for AI-assisted Auto programming.

**Concept:**
- Cells contain Auto code, markdown documentation, or AI prompts
- Each cell executes in a shared AutoVM session (state persists across cells)
- AI cells send prompts to an LLM and receive Auto code suggestions
- Output cells display structured data (tables, charts, JSON trees)

**Cell Types:**

| Type | Syntax | Behavior |
|------|--------|----------|
| Auto | ````auto` | Execute in shared VM session |
| Markdown | ````md` | Render documentation |
| AI | ````ai` | Send prompt to LLM, insert generated code |
| Shell | ````ash` | Execute in AutoShell |

### AutoVM MCP Server

Exposes AutoVM as a Model Context Protocol (MCP) server, allowing AI agents to execute Auto code, inspect types, and query the compiler.

**MCP Tools:**

| Tool | Description |
|------|-------------|
| `execute` | Run Auto code and return output |
| `type_check` | Check code for type errors without execution |
| `explain` | Explain what a piece of Auto code does |
| `suggest` | Suggest code completions at a cursor position |
| `format` | Format Auto code according to style guide |
| `test` | Run tests and return results |
| `doc` | Generate documentation for a function/type |

**Architecture:**
```
┌──────────────┐     MCP Protocol     ┌──────────────────┐
│  AI Agent    │ ←──────────────────→ │  auto-mcp        │
│  (Claude,    │    stdio / HTTP       │  (Rust binary)   │
│   GPT, etc)  │                       │                  │
└──────────────┘                       │  ┌────────────┐  │
                                       │  │ AutoVM     │  │
                                       │  │ TypeStore  │  │
                                       │  │ Formatter  │  │
                                       │  └────────────┘  │
                                       └──────────────────┘
```

**Self-Description:**
The MCP server provides a `self_describe` tool that returns the Auto language specification, syntax reference, and available stdlib modules — enabling AI agents to learn Auto on-the-fly.

### AutoUI MCP Server

A second MCP server, embedded **inside the iced desktop process** (Plan 278/299), lets AI agents perceive and drive a running AutoUI app over HTTP (`localhost:9247`). Unlike the AutoVM server (which operates on source code), this one operates on the **live, rendered UI**.

**Two complementary channels:**

| Channel | Tool | Form | What it shows |
|---------|------|------|---------------|
| **Structure / layout / style (primary)** | `autoui_vtree` | Atom text, 1:1 with the rendered VTree | The actually-rendered tree (for-loops expanded), per-node box model, computed style, events, source — all measured per-frame |
| **Pixels (secondary)** | `autoui_screenshot` | PNG file path | Pixel-level visual verification |

`autoui_vtree` (Plan 314) is the **primary perceptual channel**. Each Atom node maps 1:1 to a rendered `VNode`: its name is the source widget keyword (`col`/`row`/`button`/`center`/`text`…), its id is the instance-level `vnode_<n>`, and its props carry the full **box model** (`bbox` border-box + `content`/`padding`/`border`/`margin` insets), `style` (computed k/v), `class`, `events`, `source`, and `for_iter`. This is close to the *post-render* result rather than source, so an agent can reason about layout precisely without a screenshot.

```
col vnode_0 {
  bbox: { x: 0; y: 0; w: 1600; h: 900 }
  style: { direction: "column" }
  class: "w-full h-screen bg-white flex-col"
  button vnode_3 {
    label: "+ New"
    bbox: { x: 1480; y: 16; w: 104; h: 36 }
    style: { bg: "#3b82f6"; radius: 8 }
    events: { press: ".NewNote" }
    source: "app.at:18"
  }
}
```

`autoui_vtree` accepts `scope` (subtree by `vnode_<n>`), `depth` (fold deeper children), and `include_box`/`include_style`/`include_events`/`include_source`/`include_props` toggles for token control. Any field not yet measured (e.g. bounds before the first layout pass) is **omitted**, never an error.

The legacy `autoui_snapshot` tool (build-time AURA template + simple `@rect`) is retained for backward compatibility; `autoui_vtree` supersedes it as the runtime/computed channel. Rounding out the toolset: `autoui_inspect`, `autoui_action`, `autoui_check`, `autoui_state`, `autoui_wait`, `autoui_type`, `autoui_keyboard`.

**Data path:** the renderer populates `live_vtree` + `live_cache` every frame whenever DevTools (F12) is open **or** the MCP server is active — so an agent gets the full VTree + box model without anyone opening F12. Measured bounds flow via a per-frame `LayoutCollector` → `backfill_bounds` → `SharedState` snapshot copy.

## Open Questions

- Should the web playground support multi-file projects or remain single-file?
- AutoLab: should it support collaborative editing (multiple users)?
- MCP: should tool results include AST information or only text output?
- LSP: what's the priority order for implementing remaining features?

## Source Documents

- Plan 243: LSP VSCode modernization
- Plan 202: Web playground design
- Plan 246: AutoLab AI notebook
- Plan 265: AutoVM MCP server
- Plan 278: AutoUI MCP desktop (in-process server + SharedState)
- Plan 299: AutoUI MCP V2 protocol
- Plan 314: AutoUI MCP `autoui_vtree` — live styled VTree as Atom
