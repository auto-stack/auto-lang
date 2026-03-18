# Plan 131: Module Path Syntax Design

## Overview

Design a comprehensive module path syntax for AutoLang that supports:
- Relative imports (`super`)
- Package-relative imports (`pac`)
- Dependency imports
- Symbol-level imports

## Design Summary

### File Extensions

| Extension | Type | Wildcard Imports |
|-----------|------|------------------|
| `.at` | Auto module/library | Not allowed |
| `.as` | AutoScript | Allowed |

### Module Path Syntax

| Syntax | Meaning | Example |
|--------|---------|---------|
| `use db` | Same directory | `./db.at` or `./db/mod.at` |
| `use super.db` | Parent directory | `../db.at` or `../db/mod.at` |
| `use pac.db` | Package root | Search in configured source dirs |
| `use pac.api.handlers` | Deep path from root | `src/api/handlers.at` or `src/api/handlers/mod.at` |
| `use dep_pkg.module` | From dependency | Based on `dep` declaration |

### Import Semantics

| Syntax | Effect | Usage |
|--------|--------|-------|
| `use db` | Namespace import | `db.load()` |
| `use db: load` | Symbol import | `load()` |
| `use db: load, save` | Multiple symbols | `load()`, `save()` |
| `use db: *` | Wildcard (script mode only) | All public symbols |

### Dependency Declaration

```auto
// pac.at
name: "myapp"

// Basic dependency
dep database(path: "../database")

// With alias
dep database(path: "../database", as: "db")

// Registry dependency (future)
dep serde(version: "1.0")
```

**From code:**
```auto
use database.connection     // original name
use db.connection           // via alias
```

### Module Resolution Rules

#### Directory Structure Example

```
myapp/
├── pac.at              # name: "myapp", src: ["src"]
├── src/
│   ├── main.at
│   ├── db.at           # module "db"
│   ├── utils.at        # module "utils"
│   └── api/
│       ├── mod.at      # module "api" (directory module)
│       ├── handlers.at # module "api.handlers"
│       └── routes.at   # module "api.routes"
└── scripts/
    └── deploy.as       # AutoScript (wildcards OK)
```

#### Resolution Algorithm

For `use pac.api.handlers`:

1. Search in all configured source directories (`src/`)
2. Try `src/api/handlers.at` → file module
3. Try `src/api/handlers/mod.at` → directory module
4. **Error if BOTH exist** - explicit resolution required

#### Explicit Resolution (Error on Ambiguity)

```
src/
├── api.at
└── api/
    └── mod.at
```

**Error:** `Ambiguous module 'api' - both 'api.at' and 'api/mod.at' exist`

User must rename one to resolve.

### Package Root Import (`pac.`)

The `pac` keyword refers to the current package's configured source directories.

```auto
// From src/api/handlers/user.at

use pac.utils           // → src/utils.at
use pac.db              // → src/db.at
use pac.api.handlers    // → self (this module)
```

### Parent Import (`super`)

Only **one level** of `super` is encouraged. For deeper navigation, use `pac.` instead.

```auto
// From src/api/handlers/user.at

use super.handlers      // → src/api/handlers.at (parent's sibling)
use super.super.utils   // → src/utils.at (DISCOURAGED - use pac.utils instead)
```

**Design Principle:** `super.super`+ is a code smell. Use `pac.` for clarity.

### Dependency Import

Dependencies declared in `pac.at` are imported by package name (or alias):

```auto
// pac.at
dep database(path: "../database", as: "db")
dep serde(version: "1.0")

// In code
use database.connection    // by original name
use db.connection          // by alias
use serde.json.from_str    // nested path
```

## Implementation Status

**Completed (2025-03-18):**

| Component | Status |
|-----------|--------|
| ModulePath AST type | ✅ |
| pac/super lexer keywords | ✅ |
| Use.module_path field | ✅ |
| Parser support | ✅ |
| FilesystemResolver.resolve_with_prefix | ✅ |
| AutoManResolver dependency resolution | ✅ |
| Integration tests | ✅ |
| Error messages | ✅ |

**Total commits: 15**
**Test coverage: 25+ tests**

---

## Implementation Phases

### Phase 1: Module Path Resolution
- [x] Add `pac` keyword to lexer
- [x] Parse module paths with `pac.` prefix
- [x] Parse `super.` prefix
- [x] Resolve dependency names from `pac.at`

### Phase 2: Module File Discovery
- [x] Implement module resolver
- [x] Search source directories
- [x] Detect ambiguous modules (both `.at` and `/mod.at`)
- [x] Error reporting for unresolved modules

### Phase 3: Symbol Import
- [ ] Parse `use module: symbol` syntax
- [ ] Parse `use module: sym1, sym2` syntax
- [ ] Import symbols into current scope
- [ ] Error on duplicate symbol imports

### Phase 4: Visibility Control (Future)
- [ ] Add `pub` keyword
- [ ] Default private, explicit public
- [ ] Re-exports with `pub use`

### Phase 5: Wildcard Support (Future)
- [ ] Parse `use module: *` syntax
- [ ] Restrict to `.as` (AutoScript) files
- [ ] Import all public symbols

## Open Questions

1. **Module caching:** Should imported modules be compiled once and cached?
2. **Circular imports:** How to detect and handle circular dependencies?
3. **Namespace collision:** What happens when two deps have same symbol name?

## Examples

### Basic Import

```auto
// src/main.at
use pac.db

fn main() {
    let conn = db.connect("localhost")
    db.query(conn, "SELECT * FROM users")
}
```

### Symbol Import

```auto
// src/main.at
use pac.db: connect, query

fn main() {
    let conn = connect("localhost")
    query(conn, "SELECT * FROM users")
}
```

### With Dependencies

```auto
// pac.at
name: "myapp"
dep database(path: "../database", as: "db")
dep serde(version: "1.0")

// src/api.at
use db.connection
use serde.json.{from_str, to_str}

fn handle_request(body str) {
    let data = from_str(body)
    // ...
}
```

### AutoScript (Wildcard OK)

```auto
// scripts/deploy.as
use pac.utils: *
use pac.helpers: *

// Wildcards allowed in .as files
quick_deploy()
check_status()
```

## Comparison with Other Languages

| Feature | Auto | Rust | Python | Go |
|---------|------|------|--------|-----|
| Namespace import | `use db` | `use db` | `import db` | `import "db"` |
| Symbol import | `use db: load` | `use db::load` | `from db import load` | - |
| Package root | `pac.` | `crate::` | - | - |
| Parent | `super.` | `super::` | - | - |
| Wildcard | `use db: *` | `use db::*` | `from db import *` | - |

## Design Rationale

### Why `pac.` instead of `crate::`?

- `pac` is shorter and more distinctive
- Matches `pac.at` package file naming
- Clear semantic: "from my package"

### Why only one `super` level encouraged?

- Deep `super` chains are brittle under refactoring
- `pac.` provides absolute, refactor-friendly paths
- Encourages thinking about module structure

### Why `.as` for scripts?

- Clear visual distinction from library code
- Easy to identify at a glance
- Allows different semantics (wildcards, relaxed rules)

---

## Implementation Details

### Key Files Modified

| File | Changes |
|------|---------|
| `crates/auto-lang/src/ast/module_path.rs` | New file: `PathPrefix` enum, `ModulePath` struct |
| `crates/auto-lang/src/ast/use_.rs` | Added `module_path: Option<ModulePath>` field |
| `crates/auto-lang/src/token.rs` | Added `Pac` and `Super` keywords |
| `crates/auto-lang/src/lexer.rs` | Keyword recognition for pac/super |
| `crates/auto-lang/src/parser.rs` | Updated `use_stmt()` to parse prefixes |
| `crates/auto-lang/src/resolver.rs` | Added `resolve_with_prefix()`, `find_module()` |
| `crates/auto-man/src/resolver.rs` | Added dependency resolution via `PathPrefix::Dep` |

### Core Data Structures

```rust
// crates/auto-lang/src/ast/module_path.rs
pub enum PathPrefix {
    None,           // use db
    Super,          // use super.db
    Pac,            // use pac.db
    Dep(AutoStr),   // use database.connection
}

pub struct ModulePath {
    pub prefix: PathPrefix,
    pub segments: Vec<AutoStr>,
    pub items: Vec<AutoStr>,
}
```

### Resolution Algorithm

```
resolve_with_prefix(module_path, current_file):
  match prefix:
    Pac -> search in package source dirs
    Super -> search in parent of current file's dir
    None -> search in current file's dir
    Dep(name) -> lookup in dependencies map, search in dep path

find_module(base_dir, segments):
  path = base_dir + segments.join("/")
  if path.at exists:
    if path/mod.at exists: ERROR ambiguous
    return path.at
  if path/mod.at exists: return path/mod.at
  ERROR not found
```

### Error Messages

| Scenario | Message |
|----------|---------|
| Ambiguous module | "Ambiguous module 'X' - both 'X.at' and 'X/mod.at' exist" |
| Module not found | "Module 'X' not found. Searched locations: ..." |
| Super at root | "Cannot use 'super' at package root level. Use 'pac.' instead" |
| Dep not declared | "Dependency 'X' not declared in pac.at. Add: dep X(path: ...)" |

### Tests Added

- **ModulePath unit tests**: 5 tests for path construction/display
- **Lexer tests**: 4 tests for pac/super keyword recognition
- **Parser tests**: 8 tests for parsing new syntax
- **Resolver tests**: 7 tests for pac/super/local resolution
- **AutoMan tests**: 6 tests for dependency resolution
- **Integration tests**: 5 end-to-end tests

**Total: 35+ new tests**

---

## Deferred to Future Plans

- **Phase 4:** `pub` visibility and `pub use` re-exports
- **Phase 5:** Wildcard imports (`use db: *`) in `.as` scripts
- **Dependency alias support:** `dep database(path: "../database", as: "db")`
