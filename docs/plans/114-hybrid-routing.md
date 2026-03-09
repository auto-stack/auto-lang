# Plan 119: Hybrid Routing (Convention + Config)

## Objective

Implement a hybrid routing system where:
1. **Convention-based routes** are auto-discovered from `routes/` folder
2. **Config-based routes** are defined in `routes {}` block
3. **Merge strategy**: Config routes override convention routes

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Route Resolution                      │
├─────────────────────────────────────────────────────────┤
│  1. Scan routes/ folder → discovered_routes             │
│  2. Parse routes {} block → config_routes               │
│  3. Merge: config_routes override discovered_routes     │
│  4. Generate platform-specific navigation               │
└─────────────────────────────────────────────────────────┘
```

## File Structure Convention

```
myapp/
├── app.at                    # Main app with optional routes {}
├── routes/                   # Convention-based routes
│   ├── index.at              # "/" route
│   ├── about.at              # "/about" route
│   ├── user/
│   │   └── [id].at           # "/user/:id" route (dynamic)
│   └── admin.at              # "/admin" route (can be overridden)
└── widgets/                  # Reusable components
```

## Implementation Steps

### Step 1: Create Route Discovery Module

**File**: `crates/auto-lang/src/route/discovery.rs`

```rust
pub struct RouteDiscovery {
    routes_dir: PathBuf,
}

impl RouteDiscovery {
    pub fn new(routes_dir: PathBuf) -> Self;
    pub fn discover(&self) -> AutoResult<Vec<RouteDef>>;

    // File name to route path conversion
    // index.at → "/"
    // about.at → "/about"
    // user/[id].at → "/user/:id"
    fn file_to_route(&self, file: &Path) -> Option<RouteDef>;
}
```

### Step 2: Create Route Merger

**File**: `crates/auto-lang/src/route/merger.rs`

```rust
pub struct RouteMerger;

impl RouteMerger {
    pub fn merge(
        discovered: Vec<RouteDef>,
        config: Vec<RouteDef>,
    ) -> Vec<RouteDef>;
}
```

### Step 3: Update a2vue (cmd_vue.rs)

**File**: `crates/auto/src/cmd_vue.rs`

Current flow:
```rust
// Extract routes from widgets
for widget in &widgets {
    if let Some(ref routes) = widget.routes {
        all_routes.extend(routes.routes.clone());
    }
}
```

New flow:
```rust
// Phase 1: Discover convention-based routes
let discovery = RouteDiscovery::new(front_dir.join("routes"));
let discovered_routes = discovery.discover()?;

// Phase 2: Parse config routes from app.at
let config_routes = extract_config_routes(&widgets);

// Phase 3: Merge (config overrides convention)
let all_routes = RouteMerger::merge(discovered_routes, config_routes);
```

### Step 4: Update a2jet (ProjectGenerator)

**File**: `crates/auto-lang/src/ui_gen/jet/project.rs`

Add route discovery to project generation:

```rust
impl ProjectGenerator {
    pub fn generate(&mut self) -> HashMap<String, String> {
        // ... existing code ...

        // NEW: Discover routes if routes/ exists
        if let Ok(discovered) = self.discover_routes() {
            self.merge_routes(discovered);
        }

        // Generate NavHost from merged routes
        self.generate_navigation();
    }
}
```

## File Naming Conventions

| File Pattern | Route Path | Platform Output |
|--------------|------------|-----------------|
| `routes/index.at` | `/` | `IndexScreen.kt`, `index.vue` |
| `routes/about.at` | `/about` | `AboutScreen.kt`, `about.vue` |
| `routes/user/[id].at` | `/user/:id` | `UserScreen.kt`, `user/[id].vue` |
| `routes/admin/settings.at` | `/admin/settings` | `AdminSettingsScreen.kt`, `admin/settings.vue` |

## Merge Behavior

| Scenario | Result |
|----------|--------|
| File only | Use file-based route |
| Config only | Use config route |
| Both file + config | Config wins, merges extra props |
| Duplicate paths | Config wins |

## Files to Modify

1. **NEW**: `crates/auto-lang/src/route/mod.rs` - Module exports
2. **NEW**: `crates/auto-lang/src/route/discovery.rs` - Convention-based discovery
3. **NEW**: `crates/auto-lang/src/route/merger.rs` - Merge logic
4. **MODIFY**: `crates/auto-lang/src/lib.rs` - Export route module
5. **MODIFY**: `crates/auto/src/cmd_vue.rs` - Integrate discovery for a2vue
6. **MODIFY**: `crates/auto-lang/src/ui_gen/jet/project.rs` - Integrate for a2jet
7. **MODIFY**: `crates/auto-lang/src/ui_gen/jet/navigation.rs` - Support merged routes

## Testing

1. Unit tests for `RouteDiscovery::file_to_route()`
2. Unit tests for `RouteMerger::merge()`
3. Integration test: a2vue with hybrid routes
4. Integration test: a2jet with hybrid routes

## Backward Compatibility

- Projects without `routes/` folder: Works as before (config-only)
- Projects with `routes/` but no `routes {}`: Works with auto-discovery
- Projects with both: Hybrid mode (config overrides)

## Example Usage

### Simple App (Convention Only)

```
myapp/
├── app.at              # No routes {} block
└── routes/
    ├── index.at
    └── about.at
```

All routes auto-discovered from `routes/`.

### Complex App (Hybrid)

```
myapp/
├── app.at              # Contains routes {} for special cases
└── routes/
    ├── index.at
    ├── about.at
    └── admin.at        # Overridden by config
```

```auto
// app.at
routes {
    "/admin" => use admin {
        layout: "admin"
        auth: true
    }
}
```

Convention routes for `/` and `/about`, config route for `/admin` with extra props.
