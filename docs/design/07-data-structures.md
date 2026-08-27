# 07 - Data Structures

## Status

Core data structures are fully implemented in `crates/auto-val/src/` and `crates/auto-lang/src/`:

- **Node** (`auto-val/src/node.rs`, 1618 lines): Complete with IndexMap-based properties and children, text content, positional/named arguments, and query methods.
- **Obj** (`auto-val/src/obj.rs`): IndexMap-based ordered key-value store with typed accessors.
- **Value** (`auto-val/src/value.rs`, 1420 lines): Universal value enum covering all Auto types (Int, Float, Str, Bool, Array, Obj, Node, Nil, etc.) with arithmetic operators and conversions.
- **Array** (`auto-val/src/array.rs`): Dynamic array with push/pop/insert/remove.
- **Atom** (`auto-lang/src/atom/`): Parser, schema loader, validator, and type system for the Atom format. 7 modules.
- **ListData** and storage-based lists: VM-integrated dynamic lists with Heap and InlineInt64 storage strategies.

The Atom builder API chain methods (Phase 1) are implemented. Builder pattern (Phase 2) and macro DSL (Phase 3) are planned. A serde `Deserializer` bridge (Plan 381, `auto-val/src/de.rs`) lets plain Rust structs deserialize from Atom with zero boilerplate; the symmetric `Serializer` direction landed as `ser.rs` (Plan 332 S1, 2026-08-27; auto-ai dogfood pending).

## Design

### Node and NodeBody

Node is the central tree-building data structure, used for AST representation, configuration trees, UI widget trees, and Atom serialization.

**NodeBody** uses `IndexMap<ValueKey, NodeItem>` for storage, which provides:
- O(1) average-case lookups (better than BTreeMap's O(log n))
- Insertion-order preservation (unlike HashMap)
- Single data structure for both properties and children, eliminating index synchronization

**Node structure** contains:
- `name: AutoStr` -- the node type identifier (e.g., "config", "div")
- `body: NodeBody` -- ordered properties and children
- `text: AutoStr` -- optional text content
- `args: Vec<Value>` -- positional arguments
- `main_arg: Option<Value>` -- the primary argument (used in node syntax like `text "Hello"`)

**Access patterns**: Properties are accessed by key name. Children are accessed by node name. All operations preserve insertion order for deterministic serialization.

### Atom Format (Auto Object Markup)

Atom is a data interchange format that serves as a JSON superset with XML-like tree structure capabilities. It is designed for both machine parsing and human readability.

**Relationship to Auto**: Atom is a subset of Auto language. Dynamic Auto code (with variables, functions, loops) compiles down to static Atom data. This makes Atom the natural serialization target for configuration, templates, and UI descriptions.

**Basic types** (JSON-compatible): strings, integers, floats (with `3.0f` suffix and scientific notation), booleans, null.

**Extensions beyond JSON**:
1. Integer/float distinction without decimal ambiguity.
2. Single-line (`//`) and multi-line (`/* */`) comments.
3. Unquoted object keys: `{ a: "1", b: "2" }`.
4. Newlines replace commas as separators.
5. Top-level can be arrays, objects, standalone key-value pairs, or any mix.

**Node syntax** (XML-equivalent power): `parent(attrs) { field: val; child_node; ... }`. This provides tree nesting that JSON cannot express directly. Example:

```
root(id: "123") {
    name("Puming")
    age(41) {
        unit: "years"
    }
}
```

**Three pillars**:

| Concept | Syntax | Purpose |
|---------|--------|---------|
| Object | `{ k: v }` | Pure data carrier (like JSON) |
| Node | `name (attrs) { children }` | Tree topology description (replaces XML/HTML) |
| Link | `@ref(uuid)` | Remote/cross-process handle for distributed references |

### Atom Serialization

Auto types serialize to Atom text automatically through compiler-generated code.

**To Atom**: The compiler generates `to_compact()` and `to_pretty()` methods by AST traversal. For enum/tag types, the serialization writes name labels (e.g., `color: RED`) rather than integer values, leveraging the compiler's String<->ID mapping for readability.

**From Atom**: Deserialization uses a recursive descent parser with context stack, handling the three-level bracket structure (`[]`, `()`, `{}`). When the parser reads a node name, it looks up the corresponding type definition from the symbol table for type alignment.

**Tag serialization**: Enum variants serialize as identifiers (e.g., `status: Active` not `status: 0`) for human readability. The compiler maintains bidirectional string-to-ID mappings.

**Link mechanism**: `link @uuid(protocol://address)` represents cross-process handles. Serialized as `@ref: "uuid_string"`. Deserialization resolves these through a LinkRegistry rather than creating new objects.

**Rust struct ↔ Atom serde bridge** (Plan 381 delivered the deserialize direction; Plan 332's original custom-trait design was ruled out in favor of this serde path):

| Direction | Status | Mechanism |
|-----------|--------|-----------|
| .at → Rust struct (De) | ✅ Plan 381 | serde `Deserializer` adapter (`auto-val/src/de.rs`, ~900 lines); `#[derive(Deserialize)]` + `node.deserialize::<T>()` — zero boilerplate |
| Rust struct → .at (Ser) | 🟡 core landed (Plan 332 S1, 2026-08-27) | serde `Serializer` adapter (`auto-val/src/ser.rs`): `#[derive(Serialize)]` → `to_value` / `Value::serialize_from`, and `node_from_value(name, &T)` for the `name { … }` node shape; downstream `to_at_source()` emission and ser↔de round-trips covered by 17 tests. Remaining: auto-ai dogfood migration (Plan 332 S2). |

Annotation semantics for the ser direction (from Plan 332; to be carried as serde attribute conventions rather than a new trait family):

- Container: `#[atom(node = "name")]` — root node name, required (`.at` is a `name { ... }` structure); `legacy_node = "oldname"` accepts superseded node names on deserialize (e.g. `profession` → `role` compat).
- Fields: `rename = "key"` (field name → .at prop key), `skip`, `default` (missing → `Default::default()` instead of error).
- `Option<T>`: `None` → prop omitted on serialize; missing prop → `None` on deserialize.
- `Vec<T>` → `Array`; empty vec omitted by default.
- Inherit/merge business semantics (e.g. role config `inherit` chain) stay hand-written above the derive layer — the bridge does pure field round-trips only.

Dogfood target: auto-ai's hand-written `serialize_at_role` (~120 lines, `auto-ai-agent/rust/src/role_config.rs:363`, consumed at `roles.rs:220/322` and re-exported to Auto code via `lib.rs`) — replaced by the Serializer bridge plus `to_at_source()` once the ser direction lands.

### Atom Extensions

Atom extensions transform Atom from a pure data format into a quasi-programming language (the foundation of ASTL).

**Simplification rules**: Standard Atom is verbose because it encodes all structural information explicitly. Simplification introduces implicit rules:

1. **Primary attribute promotion**: The first field of a type definition (typically `name`) can appear directly after the node name: `fn main` instead of `fn { name: "main" }`.
2. **Secondary attribute promotion**: The second field (typically a type/kind) appears after the primary: `fn main int` instead of `fn { name: "main", return: int }`.
3. **Parameter list promotion**: Fields annotated as `@args` can be written in parentheses: `fn main(a int, b int)` instead of explicit arg nodes.
4. **Empty brace elision**: Nodes without content can omit `{}`.
5. **Body default expansion**: A field named `body` is the default expansion point for `{ }` content.

These rules reduce verbose Atom to near-source-language readability while remaining parseable.

**Schema system**: Each node type's simplification rules are defined by a schema (currently embedded in the parser, future: explicit `@primary`/`@secondary`/`@args`/`@kids` annotations on `type` definitions).

**Query mechanism**: Atom provides XPath/CSS-selector-inspired access patterns supporting bidirectional traversal (both up and down the tree), type/attribute/guard-based filtering, and function calls within selectors. Design favors built-in syntax blocks over string-based queries.

### Obj Structure

Obj is a lightweight ordered key-value store using `IndexMap<ValueKey, Value>`. It serves as the Auto language's equivalent of JavaScript objects or Python dicts.

Key constraint: `IndexMap` cannot be used in const contexts, so `Obj::new()` is not const. Static instances use `OnceLock` wrappers.

### ListData and Storage-Based Lists

Auto supports dynamic lists with pluggable storage strategies via `List<T, S>`.

**Heap storage** (`List<T, Heap>`): For PC/server environments. Dynamic growth via malloc/realloc. Limited only by available memory.

**Inline storage** (`List<T, InlineInt64>`): For MCU/embedded environments. Zero heap usage, all on stack. Fixed 64-element capacity. Deterministic memory usage with no allocation failures.

**Storage spec**: Any storage strategy implements `Storage<T>` with `data()`, `capacity()`, and `try_grow()` methods. This allows custom storage backends (arena, pool, memory-mapped) without changing list logic.

## Open Questions

- Whether the Atom query syntax should be a dedicated DSL or embedded in Auto language.
- Schema annotation syntax: `@primary`/`@secondary` annotations versus implicit positional rules.
- Performance of IndexMap for very large nodes (hundreds of properties) versus alternatives.

## Source Documents

- [raw/data-structures.md](raw/data-structures.md) -- Node, Obj, ListData implementation details
- [raw/atom.md](raw/atom.md) -- Atom format definition and philosophy
- Plan 381: value serde Deserializer (archived) -- the De-direction bridge
- Plan 332: Atom serialization derive (rewritten 2026-08-27) -- annotation semantics + Ser-direction plan
- [raw/atom-serialize.md](raw/atom-serialize.md) -- Atom serialization and node-centric spec
- [raw/extending_atom.md](raw/extending_atom.md) -- Atom extensions, simplification rules, and query design
