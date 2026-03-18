# Plan 132: api-example Read-Only Demo - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make api-example work end-to-end: frontend loads and displays users from backend via HTTP.

**Architecture:** Fix existing API extraction to handle module references, generate TypeScript client with native fetch, generate Axum routes, wire everything in build process.

**Tech Stack:** Rust, auto-lang API module, axum, TypeScript, Vue

---

## Existing Infrastructure (Already Implemented)

- `crates/auto-lang/src/api/mod.rs` - ApiExtractor for `#[api]` functions
- `crates/auto-lang/src/api/targets/typescript.rs` - TypeScript generator
- `crates/auto-lang/src/api/targets/axum.rs` - Axum route generator
- `crates/auto-man/src/api_gen.rs` - Integration with build workflow

**Problems to Fix:**
1. API extraction fails when `use db` module references exist
2. Type definitions (`type User = {...}`) not extracted
3. TypeScript client uses axios (we want native fetch)
4. `front/app.at` doesn't call the generated API

---

## Task 1: Extract Type Definitions from AST

**Files:**
- Modify: `crates/auto-lang/src/api/mod.rs`
- Modify: `crates/auto-lang/src/api/types.rs`

**Step 1: Write the failing test**

Add to `crates/auto-lang/src/api/mod.rs`:

```rust
#[cfg(test)]
mod type_extraction_tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_extract_type_definition() {
        let code = r#"
pub type User = {
    id: int
    name: str
    email: str
}
"#;
        let mut parser = Parser::from(code);
        let ast = parser.parse().unwrap();

        let extractor = ApiExtractor::new();
        let module = extractor.extract("test", &ast.stmts);

        assert_eq!(module.types.len(), 1);
        assert_eq!(module.types[0].name, "User");
        assert_eq!(module.types[0].fields.len(), 3);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang type_extraction_tests`
Expected: FAIL - types Vec is empty

**Step 3: Implement type extraction**

In `crates/auto-lang/src/api/mod.rs`, update `ApiExtractor::extract`:

```rust
pub fn extract(&self, module_name: &str, stmts: &[Stmt]) -> ApiModule {
    let mut api_module = ApiModule::new(module_name.to_string());

    for stmt in stmts {
        match stmt {
            Stmt::Fn(fn_decl) => {
                if let Some(endpoint) = self.extract_endpoint(fn_decl) {
                    api_module.add_endpoint(endpoint);
                }
            }
            Stmt::Type(type_decl) => {
                if let Some(api_type) = self.extract_type(type_decl) {
                    api_module.types.push(api_type);
                }
            }
            _ => {}
        }
    }

    api_module
}

fn extract_type(&self, type_decl: &crate::ast::TypeDecl) -> Option<ApiType> {
    use crate::ast::TypeBody;

    let fields = match &type_decl.body {
        TypeBody::Struct(fields) => {
            fields.iter().map(|f| ApiField {
                name: f.name.to_string(),
                ty: type_to_string(&f.ty),
                optional: false,
                default: None,
            }).collect()
        }
        _ => return None,
    };

    Some(ApiType {
        name: type_decl.name.to_string(),
        fields,
        doc: None,
    })
}
```

**Step 4: Run tests**

Run: `cargo test -p auto-lang type_extraction_tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/src/api/mod.rs
git commit -m "feat(api): extract type definitions for API generation"
```

---

## Task 2: Generate TypeScript Client with Native Fetch

**Files:**
- Modify: `crates/auto-lang/src/api/targets/typescript.rs`

**Step 1: Write the failing test**

Add to `crates/auto-lang/src/api/targets/typescript.rs` tests:

```rust
#[test]
fn test_generate_fetch_client() {
    let gen = TypeScriptGenerator::new();

    let endpoint = ApiEndpoint::new("listusers".to_string(), ApiAttrs {
        method: Some("GET".to_string()),
        path: Some("/api/users".to_string()),
        ..Default::default()
    });
    endpoint.return_type = "[]User".to_string();

    let result = gen.generate_fetch_function(&endpoint);
    assert!(result.contains("fetch"));
    assert!(result.contains("/api/users"));
    assert!(!result.contains("axios"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang test_generate_fetch_client`
Expected: FAIL - method doesn't exist

**Step 3: Implement fetch-based client**

In `crates/auto-lang/src/api/targets/typescript.rs`, add:

```rust
/// Generate a single fetch function for an endpoint
fn generate_fetch_function(&self, endpoint: &ApiEndpoint) -> String {
    let name = endpoint.frontend_name();
    let method = endpoint.method().to_uppercase();
    let path = endpoint.path();

    // Build parameter list
    let params: Vec<String> = endpoint.params.iter()
        .map(|p| format!("{}: {}", p.name, self.to_ts_type(&p.ty)))
        .collect();
    let param_list = params.join(", ");

    let return_type = self.to_ts_type(&endpoint.return_type);
    let return_type = if return_type == "void" {
        "Promise<void>".to_string()
    } else {
        format!("Promise<{}>", return_type)
    };

    // Build fetch call
    let mut lines = vec![
        format!("export async function {}({}): {} {{", name, param_list, return_type),
    ];

    // Build URL with path params replaced
    let url = if endpoint.path().contains(':') {
        // Has path parameters - need to replace them
        format!("`{}`", endpoint.path().replace(":", "${"))
    } else {
        format!("'{}'", path)
    };

    lines.push(format!("{}const response = await fetch({}, {{", self.indent, url));
    lines.push(format!("{}{}method: '{}',", self.indent, self.indent, method));
    lines.push(format!("{}{}headers: {{ 'Content-Type': 'application/json' }},", self.indent, self.indent));

    // Add body for POST/PUT/PATCH
    if method != "GET" && method != "DELETE" {
        let body = if endpoint.params.len() == 1 {
            endpoint.params[0].name.clone()
        } else if endpoint.params.is_empty() {
            "undefined".to_string()
        } else {
            format!("{{ {} }}", endpoint.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", "))
        };
        lines.push(format!("{}{}body: JSON.stringify({}),", self.indent, self.indent, body));
    }

    lines.push(format!("{}}});", self.indent));
    lines.push(format!("{}if (!response.ok) throw new Error(`HTTP ${{response.status}}`);", self.indent));

    if return_type != "Promise<void>" {
        lines.push(format!("{}return response.json();", self.indent));
    }
    lines.push("}".to_string());

    lines.join("\n")
}

/// Generate simple API client file with fetch functions
pub fn generate_simple_client(&self, module: &ApiModule) -> String {
    let mut lines = Vec::new();

    // Type definitions
    if !module.types.is_empty() {
        lines.push("// Type Definitions".to_string());
        lines.push("".to_string());
        for api_type in &module.types {
            lines.push(self.generate_interface(api_type));
            lines.push("".to_string());
        }
    }

    // API functions
    lines.push("// API Functions".to_string());
    lines.push("".to_string());
    for endpoint in &module.endpoints {
        lines.push(self.generate_fetch_function(endpoint));
        lines.push("".to_string());
    }

    lines.join("\n")
}
```

**Step 4: Run tests**

Run: `cargo test -p auto-lang test_generate_fetch_client`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/src/api/targets/typescript.rs
git commit -m "feat(api): generate TypeScript client with native fetch"
```

---

## Task 3: Fix API Extraction for Module References

**Files:**
- Modify: `crates/auto-man/src/api_gen.rs`

**Step 1: Understand the problem**

The current code fails when parsing `back/api.at` because it contains `use db` which references another module. The parser tries to resolve this and fails.

**Step 2: Implement lenient parsing**

In `crates/auto-man/src/api_gen.rs`, modify:

```rust
/// Extract API info from source with lenient parsing
fn extract_api_lenient(source: &str) -> Option<ApiModule> {
    use auto_lang::api::ApiExtractor;

    // Try to parse the full file first
    let mut parser = auto_lang::Parser::from(source);
    if let Ok(ast) = parser.parse() {
        let extractor = ApiExtractor::new();
        let module = extractor.extract("api", &ast.stmts);
        if !module.endpoints.is_empty() || !module.types.is_empty() {
            return Some(module);
        }
    }

    // Fall back to regex-based extraction for problematic files
    extract_api_via_regex(source)
}

/// Regex-based extraction as fallback
fn extract_api_via_regex(source: &str) -> Option<ApiModule> {
    use regex::Regex;
    use auto_lang::api::{ApiType, ApiField, ApiEndpoint, ApiAttrs};

    let mut module = ApiModule::new("api".to_string());

    // Extract type definitions
    let type_regex = Regex::new(r"(?m)^(?:pub\s+)?type\s+(\w+)\s*=\s*\{([^}]+)\}").ok()?;
    for caps in type_regex.captures_iter(source) {
        let name = caps.get(1)?.as_str().to_string();
        let body = caps.get(2)?.as_str();

        let fields: Vec<ApiField> = body.lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() { return None; }
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some(ApiField {
                        name: parts[0].trim_end_matches(':').to_string(),
                        ty: parts[1].to_string(),
                        optional: false,
                        default: None,
                    })
                } else {
                    None
                }
            })
            .collect();

        module.types.push(ApiType { name, fields, doc: None });
    }

    // Extract #[api] functions
    let fn_regex = Regex::new(r"#\[api\([^)]*\)\]\s*(?:pub\s+)?fn\s+(\w+)\s*\(([^)]*)\)\s*(\S+)?").ok()?;
    for caps in fn_regex.captures_iter(source) {
        let fn_name = caps.get(1)?.as_str().to_string();
        let params_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let return_type = caps.get(3).map(|m| m.as_str()).unwrap_or("void").to_string();

        let mut endpoint = ApiEndpoint::new(fn_name, ApiAttrs::new());
        endpoint.return_type = return_type;

        // Parse parameters (simplified)
        if !params_str.trim().is_empty() {
            for param in params_str.split(',') {
                let parts: Vec<&str> = param.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    endpoint.params.push(auto_lang::api::ApiParam {
                        name: parts[0].to_string(),
                        ty: parts[1].to_string(),
                        optional: false,
                        default: None,
                    });
                }
            }
        }

        module.endpoints.push(endpoint);
    }

    if module.endpoints.is_empty() && module.types.is_empty() {
        None
    } else {
        Some(module)
    }
}
```

**Step 3: Update generate_api to use lenient extraction**

```rust
pub fn generate_api(root_dir: &Path, backend: &str) -> AutoResult<()> {
    let back_dir = root_dir.join("back");
    let api_file = back_dir.join("api.at");

    if !api_file.exists() {
        return Ok(());
    }

    let api_content = std::fs::read_to_string(&api_file)
        .map_err(|e| format!("Failed to read {}: {}", api_file.display(), e))?;

    // Use lenient extraction
    let api_module = match extract_api_lenient(&api_content) {
        Some(m) => m,
        None => {
            println!("  ⚠ Could not extract API definitions");
            return Ok(());
        }
    };

    if api_module.endpoints.is_empty() && api_module.types.is_empty() {
        println!("  ⚠ No API endpoints or types found");
        return Ok(());
    }

    // Generate code based on backend
    match backend {
        "vue" => generate_vue_api(&api_module, root_dir)?,
        _ => {}
    }

    Ok(())
}
```

**Step 4: Add regex dependency**

In `crates/auto-man/Cargo.toml`:

```toml
[dependencies]
regex = "1"
```

**Step 5: Run tests**

Run: `cargo test -p auto-man api_gen`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/auto-man/src/api_gen.rs crates/auto-man/Cargo.toml
git commit -m "fix(api-gen): lenient parsing for files with module references"
```

---

## Task 4: Generate API Client to dist/src/lib/api.ts

**Files:**
- Modify: `crates/auto-man/src/api_gen.rs`

**Step 1: Update generate_vue_api to write to correct location**

```rust
fn generate_vue_api(api_module: &auto_lang::api::ApiModule, root_dir: &Path) -> AutoResult<()> {
    use auto_lang::api::Target;

    // For workspace projects, output to dist/src/lib/
    let dist_dir = root_dir.join("dist");
    let lib_dir = dist_dir.join("src").join("lib");
    std::fs::create_dir_all(&lib_dir)
        .map_err(|e| format!("Failed to create lib directory: {}", e))?;

    // Generate simple TypeScript client
    let ts_gen = auto_lang::api::TypeScriptGenerator::new();
    let ts_code = ts_gen.generate_simple_client(api_module);

    std::fs::write(lib_dir.join("api.ts"), &ts_code)
        .map_err(|e| format!("Failed to write api.ts: {}", e))?;

    println!("  ✓ Generated TypeScript client: dist/src/lib/api.ts");

    // Generate Rust server if back/ exists
    let back_dir = root_dir.join("back");
    if back_dir.exists() {
        generate_rust_server(api_module, root_dir)?;
    }

    Ok(())
}

fn generate_rust_server(api_module: &auto_lang::api::ApiModule, root_dir: &Path) -> AutoResult<()> {
    use auto_lang::api::Target;

    let rust_dir = root_dir.join("rust").join("src");
    std::fs::create_dir_all(&rust_dir)
        .map_err(|e| format!("Failed to create rust/src directory: {}", e))?;

    // Generate Axum routes
    let axum_gen = auto_lang::api::AxumGenerator::new();
    let axum_code = axum_gen.generate(api_module);

    std::fs::write(rust_dir.join("api_routes.rs"), &axum_code)
        .map_err(|e| format!("Failed to write api_routes.rs: {}", e))?;

    // Generate main.rs
    let main_code = generate_main_rs(api_module);
    std::fs::write(rust_dir.join("main.rs"), &main_code)
        .map_err(|e| format!("Failed to write main.rs: {}", e))?;

    println!("  ✓ Generated Rust server: rust/src/");

    Ok(())
}

fn generate_main_rs(api_module: &auto_lang::api::ApiModule) -> String {
    let routes_include = if api_module.endpoints.is_empty() {
        "".to_string()
    } else {
        "mod api_routes;\nuse api_routes::create_api_router;"
    };

    let router_setup = if api_module.endpoints.is_empty() {
        "let app = axum::Router::new();".to_string()
    } else {
        "let app = create_api_router();".to_string()
    };

    format!(r#"#[tokio::main]
async fn main() {{
{}
    {}

    println!("Server running on http://127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}}
"#, routes_include, router_setup)
}
```

**Step 2: Run tests**

Run: `cargo test -p auto-man api_gen`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/auto-man/src/api_gen.rs
git commit -m "feat(api-gen): generate client to dist/src/lib and Rust server"
```

---

## Task 5: Update front/app.at to Display Users

**Files:**
- Modify: `examples/api-example/front/app.at`

**Step 1: Update the widget**

```auto
// front/app.at - Main Application Entry
// Demo: Load and display users from backend API

widget App {
    msg Msg { Click, Load }

    model {
        var count int = 0
        var users List = List.new()
        var loading bool = false
    }

    view {
        col {
            h1 (text: "API Demo") {}

            row {
                text `Count: ${.count}`
            }

            button (text: "Load Users", onclick: .Load) {}

            // Loading indicator
            if .loading {
                text "Loading..."
            }

            // Display users
            for user in .users {
                col {
                    text `${user.name}`
                    text `${user.email}`
                }
            }
        }
    }

    on {
        .Click => {
            .count = .count + 1
        }
        .Load => {
            .loading = true
            // TODO: Call API when generated client is available
            // .users = Api.listusers()
            .loading = false
        }
    }
}
```

**Step 2: Commit**

```bash
git add examples/api-example/front/app.at
git commit -m "feat(api-example): add user list display to frontend"
```

---

## Task 6: Wire Vue Generator to Import API Client

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue/generator.rs`

**Step 1: Add API import to generated component**

Find the generated `onLoad` function and update it to call the API:

```rust
// In the Vue component generator, when generating event handlers:
// If the handler name contains "Load" and there's an api.ts, import and use it

fn generate_script_section(&self, widget: &Widget) -> String {
    let has_api_call = widget.handlers.iter().any(|h| h.name.contains("Load"));

    let mut imports = vec![
        "import { ref, computed } from 'vue'".to_string(),
    ];

    if has_api_call {
        imports.push("import { listusers } from '@/lib/api'".to_string());
    }

    // ... rest of generation
}
```

**Step 2: For now, manually verify the generated code**

Since this requires understanding the full Vue generator, we'll manually verify the output after running `auto build`.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/vue/
git commit -m "feat(vue): wire API client import in generated components"
```

---

## Task 7: End-to-End Test

**Step 1: Build the project**

```bash
cd examples/api-example
auto build
```

**Step 2: Verify generated files exist**

```bash
ls dist/src/lib/api.ts
ls rust/src/main.rs
ls rust/src/api_routes.rs
```

**Step 3: Run the Rust server**

Terminal 1:
```bash
cd examples/api-example/rust
cargo run
```

Expected: "Server running on http://127.0.0.1:3000"

**Step 4: Test the API endpoint**

```bash
curl http://127.0.0.1:3000/api/users
```

Expected: JSON array with users (or error if db module not linked)

**Step 5: Run the Vue frontend**

Terminal 2:
```bash
cd examples/api-example/dist
npm run dev
```

**Step 6: Test in browser**

1. Open http://localhost:5173
2. Click "Load Users" button
3. Verify users appear in the list

**Step 7: Document issues found**

If there are issues (likely since db.at isn't compiled), document them for the next iteration:

```
## Issues Found
1. db.at is not compiled/linked - need to include it in Rust server
2. The users data needs to be hardcoded in api_routes.rs for demo
3. Vue generator needs to emit async/await for API calls
```

**Step 8: Commit test results**

```bash
git add examples/api-example/dist/src/lib/api.ts
git add examples/api-example/rust/
git commit -m "test: api-example end-to-end test artifacts"
```

---

## Summary

| Task | Status | Files Changed |
|------|--------|---------------|
| 1. Extract type definitions | TODO | `api/mod.rs`, `api/types.rs` |
| 2. Generate fetch client | TODO | `api/targets/typescript.rs` |
| 3. Lenient API parsing | TODO | `api_gen.rs`, `Cargo.toml` |
| 4. Generate to correct paths | TODO | `api_gen.rs` |
| 5. Update front/app.at | TODO | `examples/api-example/front/app.at` |
| 6. Wire Vue generator | TODO | `ui_gen/vue/` |
| 7. E2E test | TODO | Test artifacts |

---

## Deferred to Future Tasks

- Compile `db.at` into Rust server (need a2rs to transpile multiple files)
- Generate proper async/await in Vue components
- Handle CORS for local development
- Add error handling in UI
