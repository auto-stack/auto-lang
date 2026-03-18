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

## Implementation Phases

### Phase 1: Module Path Resolution
- [ ] Add `pac` keyword to lexer
- [ ] Parse module paths with `pac.` prefix
- [ ] Parse `super.` prefix
- [ ] Resolve dependency names from `pac.at`

### Phase 2: Module File Discovery
- [ ] Implement module resolver
- [ ] Search source directories
- [ ] Detect ambiguous modules (both `.at` and `/mod.at`)
- [ ] Error reporting for unresolved modules

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
