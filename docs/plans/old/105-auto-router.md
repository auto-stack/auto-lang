# Auto Router Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement URL-driven routing in Auto language for Vue + shadcn-vue applications, enabling left-sidebar navigation with right-content-area page switching.

**Architecture:** Add `routes` block to existing `widget` syntax (Phase 1), introduce `outlet` and `link` elements, generate Vue Router configuration. Future phases will add `app` keyword for application-level configuration separation.

**Tech Stack:** Rust (Lexer/Parser/AURA), Vue 3 + Vue Router 4, shadcn-vue

---

## Background

Currently, the Component Gallery example (`examples/component-gallery/`) uses a state-driven approach (`pageIndex int`) with conditional rendering (`if .pageIndex == 0`) to switch between pages. This doesn't align with standard SPA routing patterns and doesn't support browser back/forward, bookmarks, or SEO.

**Target Architecture (Vue Router pattern):**
- Left sidebar: Static navigation links (`<router-link>`)
- Right content area: Dynamic route outlet (`<router-view>`)
- Browser URL drives which page component renders

---

## Phase 1: MVP - Routing in Widget (Current Phase)

### Overview

Add routing support within existing `widget` syntax without introducing new `app` keyword.

**Target Syntax:**
```auto
widget App {
    routes {
        "/" => IndexPage {}
        "/button" => ButtonPage {}
        "/card" => CardPage {}
        "/user/:id" => UserPage {}
    }

    view {
        col {
            Sidebar {}
            main {
                outlet
            }
        }
    }
}

// Navigation: declarative
link (to: "/button") { text "Button" }

// Navigation: programmatic
fn handleLogin() {
    nav("/user", id: "123")
}

// Route parameter access
widget UserPage {
    model {
        userId str = route.id
    }
}
```

---

## Task 1: Add Token Types for Router Keywords

**Files:**
- Modify: `crates/auto-lang/src/token.rs`

**Step 1: Add new token kinds**

Add the following token kinds to the `TokenKind` enum:

```rust
// In token.rs, find the TokenKind enum and add:
    // Router keywords (Plan 105)
    Routes,
    Outlet,
    Link,
    Route,
    Nav,
```

**Step 2: Add keywords to keyword lookup**

Find the `KEYWORDS` constant or keyword matching function and add:

```rust
// Add to keyword lookup
("routes", TokenKind::Routes),
("outlet", TokenKind::Outlet),
("link", TokenKind::Link),
("route", TokenKind::Route),
("nav", TokenKind::Nav),
```

**Step 3: Build and verify**

Run: `cargo build -p auto-lang`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add crates/auto-lang/src/token.rs
git commit -m "feat(token): add router keywords (Routes, Outlet, Link, Route, Nav)"
```

---

## Task 2: Extend Lexer for Router Keywords

**Files:**
- Modify: `crates/auto-lang/src/lexer.rs`

**Step 1: Verify keyword recognition**

The lexer should already recognize the new keywords if you added them to the keyword lookup in Task 1. Verify by finding the keyword matching logic.

**Step 2: Add test for new keywords**

Add a test in the lexer tests section:

```rust
#[test]
fn test_router_keywords() {
    let input = "routes outlet link route nav";
    let mut lexer = Lexer::new(input);

    assert_eq!(lexer.next_token().kind, TokenKind::Routes);
    assert_eq!(lexer.next_token().kind, TokenKind::Outlet);
    assert_eq!(lexer.next_token().kind, TokenKind::Link);
    assert_eq!(lexer.next_token().kind, TokenKind::Route);
    assert_eq!(lexer.next_token().kind, TokenKind::Nav);
}
```

**Step 3: Run test**

Run: `cargo test -p auto-lang test_router_keywords`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/auto-lang/src/lexer.rs
git commit -m "test(lexer): add router keywords test"
```

---

## Task 3: Add AST Nodes for Routes

**Files:**
- Modify: `crates/auto-lang/src/ast.rs`

**Step 1: Add RouteDef struct**

Add the following structs to `ast.rs`:

```rust
/// Route definition: "/path" => ComponentName {}
#[derive(Debug, Clone, PartialEq)]
pub struct RouteDef {
    pub path: String,           // "/button" or "/user/:id"
    pub component: String,      // "ButtonPage"
    pub params: Vec<String>,    // ["id"] extracted from path
}

/// Routes block containing multiple route definitions
#[derive(Debug, Clone, PartialEq)]
pub struct RoutesBlock {
    pub routes: Vec<RouteDef>,
}
```

**Step 2: Add helper method to extract params**

```rust
impl RouteDef {
    pub fn new(path: String, component: String) -> Self {
        // Extract :param from path like "/user/:id" -> ["id"]
        let params = extract_route_params(&path);
        Self { path, component, params }
    }
}

fn extract_route_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|segment| segment.starts_with(':'))
        .map(|segment| segment[1..].to_string())
        .collect()
}
```

**Step 3: Add test for param extraction**

```rust
#[test]
fn test_extract_route_params() {
    assert_eq!(extract_route_params("/button"), vec![]);
    assert_eq!(extract_route_params("/user/:id"), vec!["id"]);
    assert_eq!(extract_route_params("/post/:category/:slug"), vec!["category", "slug"]);
}
```

**Step 4: Run test**

Run: `cargo test -p auto-lang test_extract_route_params`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ast.rs
git commit -m "feat(ast): add RouteDef and RoutesBlock for router support"
```

---

## Task 4: Extend Parser for Routes Block

**Files:**
- Modify: `crates/auto-lang/src/parser.rs`

**Step 1: Add routes block parsing in widget**

Find where `widget` is parsed (likely in `parse_widget` function). Add support for `routes` block:

```rust
// In the widget parsing section, add handling for Routes token
if self.check(TokenKind::Routes) {
    let routes = self.parse_routes_block()?;
    widget.routes = Some(routes);
}
```

**Step 2: Implement parse_routes_block**

```rust
fn parse_routes_block(&mut self) -> ParseResult<RoutesBlock> {
    self.expect(TokenKind::Routes)?;
    self.expect(TokenKind::LeftBrace)?;

    let mut routes = Vec::new();

    while !self.check(TokenKind::RightBrace) {
        // Parse: "/path" => ComponentName {}
        let path = self.parse_string_literal()?;
        self.expect(TokenKind::FatArrow)?;
        let component = self.expect_identifier()?;
        self.expect(TokenKind::LeftBrace)?;
        self.expect(TokenKind::RightBrace)?;

        routes.push(RouteDef::new(path, component));

        // Optional comma
        if self.check(TokenKind::Comma) {
            self.advance();
        }
    }

    self.expect(TokenKind::RightBrace)?;
    Ok(RoutesBlock { routes })
}
```

**Step 3: Add outlet parsing in view**

In the view parsing section, add handling for `outlet`:

```rust
// In parse_view_element or similar
if self.check(TokenKind::Outlet) {
    self.advance();
    return Ok(AuraNode::Outlet);
}
```

**Step 4: Add link parsing in view**

```rust
// Parse link element: link (to: "/path") { children }
if self.check(TokenKind::Link) {
    return self.parse_link_element();
}

fn parse_link_element(&mut self) -> ParseResult<AuraNode> {
    self.expect(TokenKind::Link)?;

    // Parse props, looking for "to"
    let mut props = HashMap::new();
    if self.check(TokenKind::LeftParen) {
        self.advance();
        while !self.check(TokenKind::RightParen) {
            let key = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;
            let value = self.parse_expr()?;
            props.insert(key, AuraPropValue::Expr(value));

            if self.check(TokenKind::Comma) {
                self.advance();
            }
        }
        self.expect(TokenKind::RightParen)?;
    }

    // Parse children
    let children = if self.check(TokenKind::LeftBrace) {
        self.parse_block_children()?
    } else {
        vec![]
    };

    Ok(AuraNode::Link { props, children })
}
```

**Step 5: Add nav() function recognition**

In expression parsing, recognize `nav()` as a special builtin:

```rust
// In parse_call or similar
if name == "nav" {
    return self.parse_nav_call();
}

fn parse_nav_call(&mut self) -> ParseResult<AuraExpr> {
    self.expect(TokenKind::LeftParen)?;

    // First arg: path string
    let path = self.parse_expr()?;

    // Additional named args: id: "123"
    let mut params = HashMap::new();
    while self.check(TokenKind::Comma) {
        self.advance();
        let key = self.expect_identifier()?;
        self.expect(TokenKind::Colon)?;
        let value = self.parse_expr()?;
        params.insert(key, value);
    }

    self.expect(TokenKind::RightParen)?;

    Ok(AuraExpr::NavCall { path, params })
}
```

**Step 6: Add to AuraExpr enum in ast.rs**

```rust
// In AuraExpr enum
NavCall {
    path: Box<AuraExpr>,
    params: HashMap<String, AuraExpr>,
},
```

**Step 7: Build and test**

Run: `cargo build -p auto-lang`
Expected: Compiles without errors

**Step 8: Commit**

```bash
git add crates/auto-lang/src/parser.rs crates/auto-lang/src/ast.rs
git commit -m "feat(parser): add routes block, outlet, link, and nav() parsing"
```

---

## Task 5: Extend AURA Types for Router

**Files:**
- Modify: `crates/auto-lang/src/aura/mod.rs`

**Step 1: Add AuraRoute struct**

```rust
/// AURA route definition
#[derive(Debug, Clone, PartialEq)]
pub struct AuraRoute {
    pub path: String,
    pub component: String,
    pub params: Vec<String>,
}

/// AURA routes configuration
#[derive(Debug, Clone, PartialEq)]
pub struct AuraRoutes {
    pub routes: Vec<AuraRoute>,
}
```

**Step 2: Add AuraNode variants**

```rust
// In AuraNode enum, add:
pub enum AuraNode {
    // ... existing variants ...

    /// Router outlet - where route components render
    Outlet,

    /// Navigation link
    Link {
        to: String,
        children: Vec<AuraNode>,
    },
}
```

**Step 3: Add AuraExpr variant for nav()**

```rust
// In AuraExpr enum, add:
pub enum AuraExpr {
    // ... existing variants ...

    /// Programmatic navigation
    NavCall {
        path: String,
        params: IndexMap<String, AuraExpr>,
    },
}
```

**Step 4: Add routes field to AuraWidget**

```rust
// In AuraWidget struct
pub struct AuraWidget {
    pub name: String,
    pub state_vars: Vec<AuraStateDef>,
    pub computed: Vec<AuraComputed>,
    pub view_tree: AuraNode,
    pub handlers: Vec<(String, AuraEvent)>,
    pub styles: Option<AuraStyle>,

    // New: routes configuration
    pub routes: Option<AuraRoutes>,
}
```

**Step 5: Build**

Run: `cargo build -p auto-lang`
Expected: Compiles without errors

**Step 6: Commit**

```bash
git add crates/auto-lang/src/aura/mod.rs
git commit -m "feat(aura): add AuraRoute, AuraRoutes, Outlet, Link, NavCall types"
```

---

## Task 6: Update Vue Generator for Router

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs`

**Step 1: Add router imports detection**

In `VueGenerator`, add fields to track router usage:

```rust
pub struct VueGenerator {
    // ... existing fields ...

    /// Whether router is needed (has outlet or routes)
    needs_router: bool,

    /// Routes configuration
    routes: Vec<AuraRoute>,
}
```

**Step 2: Handle Outlet in node_to_html**

In the `node_to_html` method, add handling for `AuraNode::Outlet`:

```rust
match node {
    // ... existing cases ...

    AuraNode::Outlet => {
        self.needs_router = true;
        Ok(format!("{}<router-view></router-view>\n", ind))
    }

    AuraNode::Link { to, children } => {
        self.needs_router = true;
        let mut children_html = String::new();
        for child in children {
            children_html.push_str(&self.node_to_html(child, indent + 1)?);
        }
        Ok(format!("{}<router-link to=\"{}\">{}{}</router-link>\n",
            ind, to, children_html, ind))
    }

    // ... rest of existing cases ...
}
```

**Step 3: Add router file generation**

Add a new method to generate `router/index.ts`:

```rust
impl VueGenerator {
    /// Generate Vue Router configuration file
    pub fn generate_router_file(routes: &[AuraRoute]) -> String {
        let mut imports = Vec::new();
        let mut route_defs = Vec::new();

        for route in routes {
            imports.push(format!(
                "import {} from '@/views/{}.vue'",
                route.component, route.component
            ));

            let children_json = if route.params.is_empty() {
                "}".to_string()
            } else {
                // Add props: true for routes with params
                ",\n        props: true\n      }".to_string()
            };

            route_defs.push(format!(
                "    {{\n      path: '{}',\n      name: '{}',\n      component: {}{}\n    }}",
                route.path,
                route.component,
                route.component,
                children_json
            ));
        }

        format!(
r#"import {{ createRouter, createWebHistory }} from 'vue-router'
{}

const router = createRouter({{
  history: createWebHistory(import.meta.url),
  routes: [
{}
  ]
}})

export default router
"#,
            imports.join("\n"),
            route_defs.join(",\n")
        )
    }
}
```

**Step 4: Handle nav() in expression conversion**

In `expr_to_js`, add handling for `NavCall`:

```rust
AuraExpr::NavCall { path, params } => {
    // Build the target path with params interpolated
    if params.is_empty() {
        format!("router.push('{}')", path)
    } else {
        // Build query params or path interpolation
        let query_params: Vec<String> = params.iter()
            .map(|(k, v)| format!("{}: {}", k, self.expr_to_js(v)?))
            .collect();
        format!("router.push({{ path: '{}', query: {{ {} }} }})", path, query_params.join(", "))
    }
}
```

**Step 5: Add useRouter import when nav() is used**

In `generate_script`, add router import if nav() is detected:

```rust
// If any handler uses nav(), add router import
if self.needs_router {
    script.push_str("import { useRouter } from 'vue-router'\n");
    script.push_str("const router = useRouter()\n\n");
}
```

**Step 6: Build**

Run: `cargo build -p auto-lang`
Expected: Compiles without errors

**Step 7: Commit**

```bash
git add crates/auto-lang/src/ui_gen/vue.rs
git commit -m "feat(vue): add router-view, router-link, and router file generation"
```

---

## Task 7: Update cmd_vue.rs for Router Support

**Files:**
- Modify: `crates/auto/src/cmd_vue.rs`

**Step 1: Detect routes in widget**

In the vue command processing, detect if a widget has routes:

```rust
// After parsing widgets, check for routes
let has_routes = widgets.iter().any(|w| w.routes.is_some());

if has_routes {
    // Collect all routes from all widgets
    let all_routes: Vec<AuraRoute> = widgets.iter()
        .filter_map(|w| w.routes.as_ref())
        .flat_map(|r| r.routes.clone())
        .collect();

    // Generate router/index.ts
    let router_content = VueGenerator::generate_router_file(&all_routes);
    fs::write(output_dir.join("src/router/index.ts"), router_content)?;

    println!("  Generated src/router/index.ts");
}
```

**Step 2: Add vue-router dependency**

When generating `package.json`, add vue-router:

```rust
// In package.json dependencies
"vue-router": "^4.2.0",
```

**Step 3: Update main.ts to use router**

Modify `main.ts` template to include router:

```typescript
import {{ createApp }} from 'vue'
import App from './App.vue'
import router from './router'
import './assets/main.css'

const app = createApp(App)
app.use(router)
app.mount('#app')
```

**Step 4: Build**

Run: `cargo build -p auto`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add crates/auto/src/cmd_vue.rs
git commit -m "feat(cmd_vue): generate router files when routes detected"
```

---

## Task 8: Update Component Gallery Example

**Files:**
- Modify: `examples/component-gallery/source/front/app.at`
- Create: `examples/component-gallery/source/front/views/*.at`

**Step 1: Refactor app.at to use routes**

Replace the current `pageIndex` approach with routes:

```auto
// examples/component-gallery/source/front/app.at
widget App {
    routes {
        "/" => IndexPage {}
        "/button" => ButtonPage {}
        "/input" => InputPage {}
        "/card" => CardPage {}
        "/badge" => BadgePage {}
        "/checkbox" => CheckboxPage {}
    }

    view {
        col (class: "min-h-screen") {
            // Header
            header (class: "sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur") {
                row (class: "h-14 items-center px-6 w-full") {
                    row (class: "items-center gap-2 mr-8") {
                        text (text: "Auto UI", class: "text-lg font-bold") {}
                    }
                    row (class: "items-center gap-6 text-sm") {
                        text (text: "Docs", class: "text-muted-foreground") {}
                        text (text: "Components", class: "text-muted-foreground") {}
                    }
                }
            }

            // Main content
            row (class: "flex-1") {
                // Sidebar
                aside (class: "w-64 border-r bg-background p-4 flex flex-col gap-1") {
                    text (text: "Components", class: "text-sm font-semibold text-muted-foreground mb-2 px-3") {}

                    link (to: "/button") {
                        text (text: "Button", class: "block px-3 py-2 rounded-md hover:bg-muted cursor-pointer text-sm") {}
                    }
                    link (to: "/input") {
                        text (text: "Input", class: "block px-3 py-2 rounded-md hover:bg-muted cursor-pointer text-sm") {}
                    }
                    link (to: "/card") {
                        text (text: "Card", class: "block px-3 py-2 rounded-md hover:bg-muted cursor-pointer text-sm") {}
                    }
                    link (to: "/badge") {
                        text (text: "Badge", class: "block px-3 py-2 rounded-md hover:bg-muted cursor-pointer text-sm") {}
                    }
                    link (to: "/checkbox") {
                        text (text: "Checkbox", class: "block px-3 py-2 rounded-md hover:bg-muted cursor-pointer text-sm") {}
                    }
                }

                // Route outlet
                main (class: "flex-1 p-8") {
                    outlet
                }
            }
        }
    }
}
```

**Step 2: Create view files for each page**

Create `examples/component-gallery/source/front/views/button.at`:

```auto
// views/button.at
widget ButtonPage {
    view {
        col {
            row (class: "gap-2 text-sm text-muted-foreground") {
                text "Components"
                text "/"
                text "Button"
            }

            h1 (text: "Button") {}
            text "Displays a button or a component that looks like a button."

            h2 (text: "Installation") {}
            text "npx shadcn-vue@latest add button"

            h2 (text: "Preview") {}
            row {
                button (text: "Default") {}
                button (text: "Secondary", variant: "secondary") {}
                button (text: "Destructive", variant: "destructive") {}
                button (text: "Outline", variant: "outline") {}
            }

            h2 (text: "API Reference") {}
            text "Props: text, variant, size, disabled"
        }
    }
}
```

**Step 3: Commit**

```bash
git add examples/component-gallery/source/front/app.at
git add examples/component-gallery/source/front/views/
git commit -m "refactor(component-gallery): use routes instead of pageIndex"
```

---

## Task 9: Integration Test

**Files:**
- Create: `crates/auto-lang/test/router/`

**Step 1: Create test case**

Create `crates/auto-lang/test/router/000_basic/button.at`:

```auto
widget App {
    routes {
        "/" => HomePage {}
        "/about" => AboutPage {}
    }

    view {
        col {
            nav {
                link (to: "/") { text "Home" }
                link (to: "/about") { text "About" }
            }
            main {
                outlet
            }
        }
    }
}

widget HomePage {
    view {
        h1 "Home"
    }
}

widget AboutPage {
    view {
        h1 "About"
    }
}
```

**Step 2: Create expected router output**

Create `crates/auto-lang/test/router/000_basic/expected_router.ts`:

```typescript
import { createRouter, createWebHistory } from 'vue-router'
import HomePage from '@/views/HomePage.vue'
import AboutPage from '@/views/AboutPage.vue'

const router = createRouter({
  history: createWebHistory(import.meta.url),
  routes: [
    {
      path: '/',
      name: 'HomePage',
      component: HomePage
    },
    {
      path: '/about',
      name: 'AboutPage',
      component: AboutPage
    }
  ]
})

export default router
```

**Step 3: Add test function**

In `ui_gen/vue.rs`, add test:

```rust
#[test]
fn test_router_generation() {
    let routes = vec![
        AuraRoute {
            path: "/".to_string(),
            component: "HomePage".to_string(),
            params: vec![],
        },
        AuraRoute {
            path: "/about".to_string(),
            component: "AboutPage".to_string(),
            params: vec![],
        },
    ];

    let output = VueGenerator::generate_router_file(&routes);

    assert!(output.contains("import HomePage"));
    assert!(output.contains("import AboutPage"));
    assert!(output.contains("path: '/'"));
    assert!(output.contains("path: '/about'"));
}
```

**Step 4: Run test**

Run: `cargo test -p auto-lang test_router_generation`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/test/router/
git add crates/auto-lang/src/ui_gen/vue.rs
git commit -m "test(router): add router generation integration test"
```

---

## Task 10: Documentation

**Files:**
- Create: `docs/router.md`

**Step 1: Create router documentation**

```markdown
# Auto Router

Auto provides built-in routing support for single-page applications (SPA).

## Basic Usage

### Define Routes

Add a `routes` block to your root widget:

```auto
widget App {
    routes {
        "/" => HomePage {}
        "/about" => AboutPage {}
        "/user/:id" => UserPage {}
    }

    view {
        col {
            Sidebar {}
            main {
                outlet
            }
        }
    }
}
```

### Navigation

**Declarative (link element):**

```auto
link (to: "/about") {
    text "About Us"
}
```

**Programmatic (nav function):**

```auto
fn handleLogin() {
    nav("/dashboard")
}

fn viewProfile(userId: str) {
    nav("/user", id: userId)
}
```

### Route Parameters

Access route parameters in page widgets:

```auto
widget UserPage {
    model {
        userId str = route.id
    }

    computed {
        apiUrl => f"/api/users/{.userId}"
    }
}
```

## Vue Router Mapping

| Auto | Vue |
|------|-----|
| `routes {}` | `routes: [...]` in router config |
| `outlet` | `<router-view>` |
| `link (to: "/path")` | `<router-link to="/path">` |
| `nav("/path")` | `router.push("/path")` |
| `route.id` | `route.params.id` |
```

**Step 2: Commit**

```bash
git add docs/router.md
git commit -m "docs: add router documentation"
```

---

## Verification

After completing all tasks:

1. **Build entire project:**
   ```bash
   cargo build --release
   ```

2. **Run all tests:**
   ```bash
   cargo test --workspace
   ```

3. **Generate Component Gallery:**
   ```bash
   cargo run --release -- vue examples/component-gallery/source/front -o tmp/gallery
   ```

4. **Verify router file generated:**
   ```bash
   cat tmp/gallery/src/router/index.ts
   ```

5. **Run the generated Vue app:**
   ```bash
   cd tmp/gallery
   npm install
   npm run dev
   ```

6. **Test in browser:**
   - Navigate to different pages using sidebar
   - Verify URL changes
   - Test browser back/forward buttons

---

## Future Phases

### Phase 2: `app` Keyword

- Add `app` as first-class keyword (parallel to `widget`)
- Move `routes` from widget to app
- Support app-level configuration (theme, i18n)

```auto
app ComponentGallery {
    routes {
        "/" => IndexPage {}
        "/button" => ButtonPage {}
    }
}

widget App {
    view {
        col {
            Sidebar {}
            outlet
        }
    }
}
```

### Phase 3: AURA Architecture Refactoring

- Add `AuraApp` struct to AURA
- Separate app-level and widget-level concerns
- Support multiple backends (Vue, Compose, GPUI)

### Phase 4: Advanced Routing

- Nested routes (`children`)
- Route guards (`beforeEnter`)
- Lazy loading
- Redirects and aliases

---

## Changelog

- **2026-03-03**: Initial plan created for Plan 105
