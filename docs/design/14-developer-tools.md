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
- Plan 299: AutoUI MCP V2 protocol
