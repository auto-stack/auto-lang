# Plan 219: Playground Source Map — Click Auto Line to Highlight Generated Code

> **Status: ✅ COMPLETE** — SourceMapEntry structs implemented in auto-playground, plumbed through all transpilation targets

## Goal
Enable the auto-playground to highlight corresponding generated code (Rust/C/JS/TS/Python) when the user clicks a line in the left Auto source editor. This requires bottom-up source map support in the transpiler pipeline.

## Background Research

After surveying mainstream compilers, the consensus is:
- **Rust (rustc)**: Every AST node embeds a `Span` (byte offsets into a global `CodeMap`). Spans were kept inside the AST despite memory cost (~12%) because side-tables go out of sync during AST transforms.
- **Python (`ast`)**: Each node stores `lineno/col_offset/end_lineno/end_col_offset` directly.
- **TypeScript**: Base `Node` has `pos/end` (absolute character offsets). Line numbers are computed on demand via `SourceFile.lineStarts[]`. Source maps are generated during codegen by the emitter.
- **Vue 3 Compiler**: AST nodes carry `loc: SourceLocation`. Codegen tracks `line/column/offset` and calls `addMapping` on every `push(code, node)`.

The key insight for AutoLang: **statement-level mapping is sufficient** for the playground use case. We do not need expression-level granularity.

## Current State
- `Stmt` enum has **no position information**.
- `Parser` has position info via `Token.pos` (`Pos { line, at, pos, len }`).
- `Sink` is a raw byte buffer with **no source map tracking**.
- Playground `/api/trans` returns only `TransResponse { code, target }`.
- Playground frontend shows raw `<pre><code>` with no line numbers or interactivity.

## Design Decisions

### Decision 1: Statement-level mapping via `Code.source_lines` / `Body.source_lines`

Instead of refactoring the entire `Stmt` enum (which would touch hundreds of match sites), we add a parallel `source_lines: Vec<usize>` array to `Code` and `Body`. `source_lines[i]` records the source line where `stmts[i]` begins.

**Trade-offs:**
- ✅ Minimal AST disruption. `Stmt` enum unchanged.
- ✅ Parser change is localized to `parse()` and `parse_body()`.
- ⚠️ CTEE (`#if`/`#for`/`#is` expansion) must propagate lines when expanding. This is manageable because expansion is statement-level.
- ⚠️ Less granular than expression-level. Acceptable for playground.

### Decision 2: Sink generates a simple line-mapping table

`Sink` grows a `source_map: Vec<SourceMapEntry>` where:
```rust
pub struct SourceMapEntry {
    pub source_line: usize,
    pub output_line: usize,
}
```

The transpiler calls `sink.set_source_line(n)` before emitting a statement. `Sink` records a mapping whenever a newline is written while a source line is "active".

**Why not Source Map v3?**
- Playground only needs "click source line → highlight generated lines".
- A simple `(source_line, output_line)` array is smaller, faster to generate, and trivial to consume in the frontend.
- Standard source maps are designed for browser debuggers; we are building an IDE-like playground feature.

### Decision 3: Frontend uses line-numbered preview + CodeMirror gutter events

- Replace `<pre><code>` with a line-numbered display (native CSS counters or lightweight component).
- CodeMirror 6 already renders line numbers. We can listen to gutter clicks via a small extension.
- The mapping is inverted on the frontend: `source_line → Vec<output_line>` for O(1) lookup.

## Implementation Phases

### Phase 1: AST — Add `source_lines` to `Code` and `Body`

**Files to modify:**
- `crates/auto-lang/src/ast.rs` — add `source_lines: Vec<usize>` to `Code`
- `crates/auto-lang/src/ast/body.rs` — add `source_lines: Vec<usize>` to `Body`
- Update all `Code::new()`, `Body::new()`, `Body::single_expr()`, `Body::default()` constructors.

### Phase 2: Parser — Record source lines when building `Code` and `Body`

**Files to modify:**
- `crates/auto-lang/src/parser.rs`

**Changes:**
1. In `Parser::parse()`, before `self.parse_stmt()`, capture `self.cur.pos.line` and push it into `source_lines` alongside `stmts`.
2. In `Parser::parse_body()`, do the same.
3. For `Stmt::EmptyLine(n)` inserted by the parser, assign the current line number.

### Phase 3: CTEE — Propagate `source_lines` during comptime expansion

**Files to modify:**
- `crates/auto-lang/src/comptime/transformer.rs`

**Changes:**
1. Modify `transform(&mut self, code: &mut Code)` to drain and zip both `stmts` and `source_lines`.
2. Introduce `transform_stmt_with_line(stmt, line) -> Vec<(Stmt, usize)>`.
   - `HashIf`/`HashFor`/`HashIs`/`HashBrace` expansion: all generated statements inherit the original `line`.
   - `other`: returns `vec![(other, line)]`.
3. Recursively transform inner `Body` nodes (e.g., `HashIf.then_block`, `HashFor.body`) the same way.

### Phase 4: Sink — Add source map tracking

**Files to modify:**
- `crates/auto-lang/src/trans.rs`

**Changes:**
1. Add `SourceMapEntry` struct.
2. Add to `Sink`:
   - `source_map: Vec<SourceMapEntry>`
   - `current_source_line: Option<usize>`
   - `current_output_line: usize` (starts at 1)
3. Add `pub fn set_source_line(&mut self, line: usize)`.
4. Modify `print`/`println`/`done` to track newlines and push entries when `current_source_line` is set.
   - On every `\n` written to `body`, if `current_source_line.is_some()`, push `(source_line, current_output_line)` and increment `current_output_line`.
   - `current_output_line` starts at 1 and counts `\n` in `body` (and `source` during `done()` if needed).

### Phase 5: Transpilers — Wire source line tracking

**Files to modify:**
- `crates/auto-lang/src/trans/rust.rs`
- `crates/auto-lang/src/trans/c.rs`
- `crates/auto-lang/src/trans/javascript.rs`
- `crates/auto-lang/src/trans/typescript.rs`
- `crates/auto-lang/src/trans/python.rs`

**Changes:**
For each transpiler, in the `Trans::trans()` entry point and anywhere that iterates over `Body.stmts`:
1. Before emitting the i-th top-level statement, call `sink.set_source_line(code.source_lines[i])`.
2. For function bodies, `if` bodies, `for` bodies, etc., call `sink.set_source_line(body.source_lines[i])` before emitting the i-th inner statement.
3. After all transpilation, `sink.set_source_line(None)` or simply stop setting lines.

**Note:** Some transpilers buffer into local `Vec<u8>` before writing to `Sink` (e.g., TypeScript uses `body_buf`). For those, either write directly to `Sink` (preferred) or accept that mappings for those sections will be approximate.

### Phase 6: Playground Backend — Return source map in API

**Files to modify:**
- `crates/auto-playground/src/routes/trans.rs`
- `crates/auto-playground/frontend/src/types.ts`

**Changes:**
1. `TransResponse` adds `source_map: Vec<SourceMapEntry>` (serialized as JSON array of `[source_line, output_line]` pairs).
2. Each `transpile_*` function in `trans.rs` extracts `sink.source_map` and returns it alongside the code.
3. Update frontend `TransResponse` interface.

### Phase 7: Playground Frontend — Interactive line highlighting

**Files to modify:**
- `crates/auto-playground/frontend/src/components/CodeEditor.vue`
- `crates/auto-playground/frontend/src/components/CodePreview.vue`
- `crates/auto-playground/frontend/src/composables/usePlayground.ts`
- `crates/auto-playground/frontend/src/App.vue`

**Changes:**
1. **CodeEditor.vue**: Add a CodeMirror gutter click extension. When a gutter line is clicked, emit `line-click` event with the 1-based line number.
2. **CodePreview.vue**: Refactor from `<pre><code>` to a line-based renderer:
   - Use CSS counters or a simple `v-for` over `code.split('\n')` to show line numbers.
   - Accept `highlightLines: number[]` prop and highlight those rows (e.g., yellow background).
3. **usePlayground.ts**:
   - Store `sourceMap` as `Record<number, number[]>` (inverted index: source line → output lines).
   - Provide `highlightSourceLine(line: number)` that looks up output lines and emits them to the preview.
4. **App.vue / PlaygroundLayout.vue**: Wire `CodeEditor` `@line-click` to `usePlayground.highlightSourceLine`, and pass resulting lines to `CodePreview`.

### Phase 8: Build & Test

1. `cargo test -p auto-lang` — ensure parser/CTEE/transpiler tests pass.
2. `cargo test -p auto-playground` — backend tests.
3. Build frontend and verify the click-to-highlight loop works for Rust, C, JS, TS, and Python targets.

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| CTEE transform desyncs `source_lines` | Unit-test CTEE expansion with `source_lines`; assert `stmts.len() == source_lines.len()` after every transform. |
| Transpilers that buffer to local `Vec<u8>` (TypeScript) lose mapping accuracy | For MVP, map those sections to the statement's start line. Future work: route through `Sink` directly. |
| Empty lines / comments skew line alignment | Map by statement start line; empty lines inherit the previous active source line or are unmapped. |
| Frontend line numbers are 1-based vs 0-based | Define all line numbers as **1-based** (matching human-visible editor line numbers) in both backend and frontend. |

## Success Criteria
- [ ] Clicking line 5 in the Auto editor highlights the corresponding line(s) in the Rust preview.
- [ ] Same behavior works for C, JavaScript, TypeScript, and Python tabs.
- [ ] `cargo test -p auto-lang` passes.
- [ ] No visible performance regression in playground transpile speed.
