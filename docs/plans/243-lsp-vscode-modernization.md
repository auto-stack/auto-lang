# Auto LSP & VSCode Plugin Modernization Plan

## Executive Summary

The Auto LSP (`crates/auto-lsp`) is currently **broken and disabled** from the workspace. It uses APIs removed in Plan 091 (`Universe::new()`, `Parser::new(code, scope)`), depends on the unmaintained `tower-lsp 0.20`, and implements features via regex fallbacks instead of the rich compiler infrastructure (`Database`, `QueryEngine`, `Indexer`, `TypeStore`, `ModuleResolver`) that already exists in `auto-lang`.

The VSCode extension (`../auto-vscode`) is functional at v0.2.2 but has not been updated to match the language's v0.3+ features (generics, AutoUI, comptime, async/await, borrow operators, closures, etc.).

**Recommended approach:** Rewrite the LSP as a thin, compiler-native wrapper over `auto-lang`'s `Database` + `QueryEngine`. This makes the LSP resilient to language evolution because new features are automatically available if the compiler team maintains the query APIs. Combine this with CI gates, integration tests, and automated VSCode packaging to ensure continuous alignment.

---

## Current State Analysis

### `auto-lsp` — Broken & Outdated
| Issue | Detail |
|-------|--------|
| **Does not compile** | Commented out in workspace `Cargo.toml` with note "Temporarily disabled due to Cargo.toml issues" |
| **Uses removed APIs** | `Universe::new()`, `Parser::new(content, navigator)` were removed in Plan 091 |
| **Unmaintained framework** | `tower-lsp 0.20` is dead; community fork `tower-lsp-server 0.23` is actively maintained |
| **Regex fallbacks** | Document symbols, variable completion, function extraction all use regex instead of AST |
| **No incremental parsing** | Re-parses entire document on every keystroke despite `Indexer::reindex_file()` existing |
| **No cross-file support** | `ModuleResolver`, `CompileSession`, and `FilesystemResolver` exist but are unused |
| **Stubs advertised as supported** | Find References, Rename, Code Actions, Workspace Symbols are all `Ok(None)` but advertised in capabilities |
| **Character-level diagnostics unused** | Parser produces `miette` spans with byte offsets; LSP manually walks lines with regex fallback |

### `auto-vscode` — Functional but Stagnant
| Issue | Detail |
|-------|--------|
| **v0.2.2** | Has not been updated since v0.2 language features |
| **Single JS file** | `extension.js` is plain JS (~309 lines); no TypeScript, no structured src/ tree |
| **Manual binary bundling** | `build-lsp.ps1` / `.sh` copy binaries manually; no CI automation |
| **Missing modern features** | No semantic highlighting, no inlay hints, no signature help, no code lens |
| **Grammar gaps** | TextMate grammar covers many keywords but misses newer constructs like `#[with(...)]`, `#{}`, `~T`, `.?` |

### `auto-lang` — Rich, Underutilized Infrastructure
Modern compiler features already implemented but ignored by the LSP:
- `Database`: symbol locations, fragment metadata, dependency graph, file hashes
- `QueryEngine`: cached queries for `GetCompletionsQuery`, `GetSymbolLocationQuery`, `FindReferencesQuery`, `InferExprTypeQuery`
- `Indexer`: fragment-based incremental re-indexing with BLAKE3 hash comparison
- `TypeStore`: unified registry of `fn_decls`, `type_decls`, `spec_decls`, `enum_decls`
- `ModuleResolver` / `FilesystemResolver`: cross-file `use` / `pac.` / `super.` resolution
- `InferenceContext`: scoped variable bindings with full type inference

---

## Recommended Approach: Compiler-Native LSP + Modern VSCode Extension

### Philosophy
**The LSP should be a thin I/O adapter between the LSP protocol and the compiler's internal APIs.** When the compiler gains a new feature (e.g., a new AST node or type variant), the LSP should require minimal or zero changes to support it in completions, hover, and goto-definition — provided the compiler exposes it through `TypeStore`, `Database`, or `QueryEngine`.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  VSCode Extension (TypeScript)                              │
│  ├─ Thin LSP client (vscode-languageclient)                 │
│  ├─ Build/Run commands (auto build / auto run)              │
│  ├─ TextMate grammar + semantic highlighting fallback       │
│  └─ Snippets, templates, status bar                         │
└────────────────────────┬────────────────────────────────────┘
                         │ stdio JSON-RPC
┌────────────────────────▼────────────────────────────────────┐
│  auto-lsp (Rust) — Thin Protocol Adapter                    │
│  ├─ tower-lsp-server 0.23 (community fork)                  │
│  ├─ Document cache + incremental sync                       │
│  ├─ Per-document Database + QueryEngine                     │
│  └─ Per-workspace ModuleResolver + TypeStore                │
└────────────────────────┬────────────────────────────────────┘
                         │ direct API calls
┌────────────────────────▼────────────────────────────────────┐
│  auto-lang (Rust) — Compiler as a Library                   │
│  ├─ Parser::from() / Parser::with_session()                 │
│  ├─ Database (symbols, fragments, deps)                     │
│  ├─ QueryEngine (completions, hover, refs, goto-def)        │
│  ├─ Indexer (incremental re-indexing)                       │
│  ├─ TypeStore (type/function/spec registry)                 │
│  └─ ModuleResolver (cross-file use/pac/super)               │
└─────────────────────────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Foundation — Make It Compile (Week 1)
**Goal:** Restore `auto-lsp` to the workspace and get a working binary.

1. **Update dependencies**
   - Replace `tower-lsp = "0.20"` with `tower-lsp-server = "0.23"`
   - Update `lsp-types` to match (0.97+)
   - Remove `async_trait` usage (tower-lsp-server uses native `impl Trait` in trait)

2. **Fix API breakage**
   - Replace all `auto_lang::Universe::new()` with `auto_lang::Parser::from(code)`
   - Replace `Parser::new(content, navigator)` with `Parser::from(content)`
   - Use `parser.type_store` and `parser.infer_ctx` instead of `navigator.borrow().lookup_meta()`
   - Update `miette` diagnostic conversion to use `error.labels()` directly for byte-accurate spans

3. **Re-enable workspace membership**
   - Uncomment `"crates/auto-lsp"` in root `Cargo.toml`
   - Ensure `cargo check -p auto-lsp` passes in CI

4. **Smoke-test VSCode connection**
   - Build binary, copy to `../auto-vscode/vscode-extension/bin/`, verify stdio transport still works

### Phase 2: Core Features via QueryEngine (Week 2–3)
**Goal:** Replace regex fallbacks with compiler-native queries.

1. **Diagnostics**
   - Use `parser.errors` and `parser.warnings` vectors directly (already have `miette` spans)
   - Remove `extract_line_number` regex fallback
   - Convert byte offsets to LSP `Position` using a proper UTF-16-aware function

2. **Document Symbols**
   - Replace regex extraction with `Indexer::index_ast()` + `Database.get_fragments_in_file()`
   - Produce hierarchical `DocumentSymbol` trees (e.g., `widget` → `model` → fields)

3. **Completions**
   - Replace static keyword/type/function lists with `GetCompletionsQuery` from `QueryEngine`
   - Use `TypeStore.lookup_fn_decl_str()` / `lookup_type_decl_str()` for user-defined symbols
   - Use `InferenceContext.lookup_type()` for scoped variable completion
   - Keep keyword snippets as a thin overlay (generated from a single source-of-truth list)

4. **Hover**
   - Replace hardcoded docs with `InferExprTypeQuery` + `TypeStore` metadata
   - For stdlib symbols, generate docs from `stdlib/` source files at build time (or embed generated JSON)

5. **Go to Definition**
   - Replace manual AST traversal with `GetSymbolLocationQuery`
   - Support cross-file navigation via `Database.symbol_locations` with file IDs

### Phase 3: Cross-File & Workspace Awareness (Week 4)
**Goal:** Support multi-file projects with `use`, `pac.`, and `super.` resolution.

1. **Workspace initialization**
   - On `initialize`, detect workspace root and scan for `pac.at`
   - Create a workspace-level `ModuleResolver` + `TypeStore` shared across documents

2. **Multi-document Database**
   - Maintain a single `Database` for the workspace
   - Index each opened file with `Indexer::index_ast()`
   - On `did_change`, use `Indexer::reindex_file()` for incremental updates

3. **Cross-file features**
   - `FindReferencesQuery` → workspace-wide references
   - `WorkspaceSymbol` query → `Database.get_all_symbol_locations()`
   - Resolve `use` statements via `FilesystemResolver` to populate completions with imported symbols

4. **Auto-import**
   - When completing a symbol not in scope, search workspace `TypeStore` and offer `use` insertion

### Phase 4: Modern LSP Features (Week 5)
**Goal:** Implement advertised but stubbed features.

1. **Rename Symbol**
   - Use `Database.symbol_locations` + AST rewrite to produce `WorkspaceEdit`
   - Start with single-file rename; extend to workspace via fragment metadata

2. **Code Actions**
   - Auto-import for unresolved names
   - Quick fix: add missing `mut` keyword
   - Quick fix: convert `let` to `var` for mutation

3. **Signature Help**
   - Query `TypeStore` for function signatures while typing inside `(...)`
   - Track parameter index via AST position

4. **Inlay Hints** (optional but high-value)
   - Show inferred types after `let` bindings: `let x /*: int */ = 5`
   - Show parameter names at call sites: `foo(/*x:*/ 1, /*y:*/ 2)`

### Phase 5: VSCode Extension Modernization (Week 5–6, parallel)
**Goal:** Update the client to match modern VSCode extension best practices.

1. **TypeScript migration**
   - Convert `extension.js` to `src/extension.ts` with proper types
   - Use `@types/vscode` and `vscode-languageclient` TypeScript bindings

2. **Semantic highlighting**
   - Register a `DocumentSemanticTokensProvider` that delegates to the LSP
   - Or implement a lightweight provider in the extension that uses AST queries

3. **Grammar updates**
   - Add missing v0.3+ tokens to `auto.tmLanguage.json`:
     - `#[with(...)]`, `#[derive(...)]` attributes
     - `#{}` comptime expressions, `#if`/`#for`/`#is` blocks
     - `~T` future types, `.await`, `.go`, `.?` postfix operators
     - `.view`, `.mut`, `.move`, `.hold` borrow operators
     - `widget`, `model`, `view`, `on`, `routes`, `computed` UI keywords

4. **CI/CD for extension**
   - GitHub Actions workflow to:
     - Build `auto-lsp` for Windows/macOS/Linux
     - Copy binaries into extension `bin/`
     - Run `vsce package` and publish to marketplace (optional)

### Phase 6: Continuous Maintenance Infrastructure (Ongoing)
**Goal:** Prevent future drift between language and LSP.

1. **CI gate on every PR**
   - `cargo check -p auto-lsp` must pass
   - `cargo test -p auto-lsp` must pass (add integration tests)

2. **LSP integration test suite**
   - Create `crates/auto-lsp/tests/` with sample `.at` files covering all language features
   - Use `lsp-types` + JSON-RPC in-process testing to assert:
     - Completions contain expected items at given positions
     - Hover returns expected markdown for given symbols
     - Goto-definition resolves to correct file/line/column
     - Diagnostics have precise spans for known errors
   - Add a test that fails if `auto-lang` exports a new AST node type that the LSP doesn't handle

3. **Generated keyword/type lists**
   - At build time, scan `auto-lang` AST enums and `stdlib/` to generate:
     - `keywords.rs` — all language keywords
     - `stdlib_types.rs` — all primitive and stdlib types
     - `stdlib_fns.rs` — all stdlib function signatures
   - This ensures completions never drift from the actual language/stdlib

4. **Automated VSCode packaging**
   - Nightly GitHub Action that:
     - Builds `auto-lsp` for all platforms
     - Packages VSCode extension with bundled binaries
     - Creates a draft GitHub release with `.vsix` artifact

5. **LSP ↔ compiler API contract**
   - Document in `docs/implementation/lsp-api-contract.md`:
     - Which `auto-lang` APIs the LSP depends on
     - How to update the LSP when breaking compiler changes occur
   - Require compiler PRs that break these APIs to also update `auto-lsp`

---

## Alternative Approach: Minimal Band-Aid (Not Recommended)

Fix only the compilation errors by replacing removed APIs with their nearest modern equivalents, but keep the regex-based architecture. This would take ~2 days but:
- Leaves the LSP fragile and manually maintained
- Does not solve cross-file support
- Will break again on the next compiler refactor
- Wastes the existing `QueryEngine` / `Database` infrastructure

**Only consider this if immediate unblocking is critical and a full rewrite is scheduled within 1 month.**

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| `tower-lsp-server` API changes | Pin to a specific minor version; migration from 0.20 → 0.23 is the hard part; future updates are minor |
| `QueryEngine` APIs are incomplete | Phase 2 falls back to direct `Database` / `TypeStore` access where queries don't exist yet |
| Cross-file resolution is complex | Phase 3 starts with same-directory files only; expands to full workspace incrementally |
| Performance on large files | Use `Indexer::reindex_file()` + debouncing; benchmark with `benches/` corpus |
| Windows-specific LSP issues | The original had Windows stdout suppression issues; use `tower-lsp-server` which handles this better |

---

## Success Criteria

1. `cargo check -p auto-lsp` passes in CI on every commit
2. All v0.2 features (completion, hover, goto-def, diagnostics) work with v0.3+ syntax
3. New v0.3 features (generics, AutoUI widgets, comptime, async) are at least parseable without false-positive diagnostics
4. Cross-file goto-definition works for `use` and `pac.` imports within the same workspace
5. VSCode extension bundles the LSP binary automatically via CI
6. Integration tests cover >80% of LSP handlers with sample Auto code

---

## Immediate Next Steps (Post-Approval)

1. Read `tower-lsp-server` 0.23 migration guide and update `Cargo.toml`
2. Replace `Universe` usage in `completion.rs`, `hover_info.rs`, `goto_def.rs`
3. Re-enable `auto-lsp` in workspace `Cargo.toml`
4. Add a single integration test: open a `.at` file, assert diagnostics are empty
5. Update VSCode extension grammar with missing v0.3 tokens
