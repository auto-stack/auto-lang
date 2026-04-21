# 185: VSCode Extension Reuses Vue Build Output

## Context

The VSCode extension generator (`a2vscode`) currently creates a complete duplicate Vue project inside `gen/vscode/webview-ui/`. This means a second `npm install` and a second Vite build for what is essentially the same frontend code that `gen/vue/` already produces. The user must wait for two full frontend builds when targeting both backends.

## Goal

Eliminate the duplicate webview project. The VSCode extension loads its webview UI from `gen/vue/dist/` instead of maintaining its own `webview-ui/` subfolder.

## Current Structure

```
gen/
  vue/                   # Full Vue project (npm install, shadcn, vite build)
    dist/                # Built output
  vscode/
    src/                 # Extension host code
      extension.ts
      panels/AppPanel.ts
    webview-ui/          # DUPLICATE — separate npm install, separate build
      src/App.vue
      src/main.ts
      package.json
      vite.config.ts
      ...12 more files
    package.json         # Extension manifest
    webpack.config.js
```

## Target Structure

```
gen/
  vue/                   # Full Vue project (single npm install + build)
    dist/                # Built output — shared with VSCode
  vscode/                # Extension scaffold only
    src/
      extension.ts       # Extension activation
      panels/AppPanel.ts # Webview host — loads from ../vue/dist/
    package.json         # Extension manifest
    webpack.config.js
    tsconfig.json
    .vscodeignore
    .vscode/
```

## Changes

### 1. AppPanel.ts resource paths

Change webview asset resolution from `webview-ui/dist/` to `../vue/dist/`:

```ts
// Before
vscode.Uri.joinPath(this._extensionUri, 'webview-ui', 'dist', 'assets', 'index.js')
// After
vscode.Uri.joinPath(this._extensionUri, '..', 'vue', 'dist', 'assets', 'index.js')
```

Same change for the CSS URI.

### 2. localResourceRoots

Allow the extension to load resources from the sibling `vue/dist/` directory:

```ts
// Before
localResourceRoots: [
    vscode.Uri.joinPath(extensionUri, 'webview-ui'),
    vscode.Uri.joinPath(extensionUri, 'media'),
]
// After
localResourceRoots: [
    vscode.Uri.joinPath(extensionUri, '..', 'vue', 'dist'),
    vscode.Uri.joinPath(extensionUri, 'media'),
]
```

### 3. Build flow in build_vscode_project

New build sequence:

1. **Ensure gen/vue/ is built**: Call the Vue generator and run `npm install && npm run build` in `gen/vue/` to produce `gen/vue/dist/`.
2. **Generate extension scaffold**: Only generate extension-specific files into `gen/vscode/` (no webview-ui/).
3. **Install extension deps**: Run `npm install` in `gen/vscode/` (for the extension's own TypeScript compilation deps).
4. **Compile extension**: Run `npm run compile` (webpack) to bundle extension.ts and AppPanel.ts.

### 4. Remove from VSCode generator

Delete all `webview-ui/*` file generation:

- `webview-ui/src/App.vue`
- `webview-ui/src/main.ts`
- `webview-ui/src/env.d.ts`
- `webview-ui/src/assets/index.css`
- `webview-ui/package.json`
- `webview-ui/vite.config.ts`
- `webview-ui/tsconfig.json`
- `webview-ui/tailwind.config.js`
- `webview-ui/postcss.config.js`
- `webview-ui/index.html`

Remove from `package.json` scripts:
- `webview:install`
- `webview:build`

Remove from `.vscodeignore`:
- `webview-ui/src/**`
- `webview-ui/node_modules/**`
- `webview-ui/index.html**`

### 5. compile_at_to_vue removal

The `compile_at_to_vue()` function in vscode.rs is no longer needed. The VSCode generator no longer generates Vue code — it reuses `gen/vue/dist/`.

## Files Modified

- `crates/auto-man/src/vscode.rs` — remove webview-ui generation, update build flow, update template strings for AppPanel.ts

## Verification

1. `AUTO_BACKEND=vscode auto run` from a UI example directory
2. Verify `gen/vue/dist/` is built first
3. Verify `gen/vscode/` contains only extension files (no webview-ui/)
4. Verify VSCode opens and the webview renders correctly
5. Verify `auto gen` for vue backend still works independently

---

## Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Eliminate the duplicate webview-ui/ in the VSCode generator by reusing gen/vue/dist/.

**Architecture:** The VSCode extension becomes a thin wrapper that loads its webview from the sibling gen/vue/dist/ directory. The build flow ensures gen/vue/ is built before gen/vscode/ is generated.

**Tech Stack:** Rust (auto-man crate), VSCode Extension API, Vue 3 + Vite

**Single file modified:** `crates/auto-man/src/vscode.rs`

---

### Task 1: Update AppPanel.ts template — resource paths and localResourceRoots

**Files:**
- Modify: `crates/auto-man/src/vscode.rs` — `generate_app_panel_ts()` function (~line 910)

**Step 1: Update webview asset URIs**

In `generate_app_panel_ts()`, find the `_getHtmlForWebview` method template and change:

```rust
// Before (line ~994-999):
const scriptUri = webview.asWebviewUri(
    vscode.Uri.joinPath(this._extensionUri, 'webview-ui', 'dist', 'assets', 'index.js')
);
const styleUri = webview.asWebviewUri(
    vscode.Uri.joinPath(this._extensionUri, 'webview-ui', 'dist', 'assets', 'index.css')
);
```

To:
```rust
const scriptUri = webview.asWebviewUri(
    vscode.Uri.joinPath(this._extensionUri, '..', 'vue', 'dist', 'assets', 'index.js')
);
const styleUri = webview.asWebviewUri(
    vscode.Uri.joinPath(this._extensionUri, '..', 'vue', 'dist', 'assets', 'index.css')
);
```

**Step 2: Update localResourceRoots**

In the same function, change:

```rust
// Before (line ~1049-1051):
localResourceRoots: [
    vscode.Uri.joinPath(extensionUri, 'webview-ui'),
    vscode.Uri.joinPath(extensionUri, 'media'),
],
```

To:
```rust
localResourceRoots: [
    vscode.Uri.joinPath(extensionUri, '..', 'vue', 'dist'),
    vscode.Uri.joinPath(extensionUri, 'media'),
],
```

**Step 3: Build and verify**

Run: `cargo build -p auto`
Expected: Compiles without errors

**Step 4: Commit**

```
feat(vscode): point webview to ../vue/dist/ instead of webview-ui/dist/
```

---

### Task 2: Update package.json template — remove webview scripts

**Files:**
- Modify: `crates/auto-man/src/vscode.rs` — `generate_package_json()` function (~line 810)

**Step 1: Simplify build scripts**

In `generate_package_json()`, change the scripts section:

```rust
// Before (line ~853-859):
"scripts": {{
    "vscode:prepublish": "npm run compile",
    "build": "npm run webview:install && npm run webview:build && npm install && npm run compile",
    "compile": "webpack --mode production",
    "watch": "webpack --mode development --watch",
    "webview:install": "cd webview-ui && npm install",
    "webview:build": "cd webview-ui && npm run build"
}}
```

To:
```rust
"scripts": {{
    "vscode:prepublish": "npm run compile",
    "build": "npm install && npm run compile",
    "compile": "webpack --mode production",
    "watch": "webpack --mode development --watch"
}}
```

**Step 2: Build and verify**

Run: `cargo build -p auto`
Expected: Compiles without errors

**Step 3: Commit**

```
refactor(vscode): remove webview:install and webview:build scripts
```

---

### Task 3: Remove webview-ui file generation from VscodeProject::generate()

**Files:**
- Modify: `crates/auto-man/src/vscode.rs` — `generate()` method (~line 287) and all `write_webview_*` methods

**Step 1: Remove webview-ui directory creation**

In `generate()` (~line 308-318), remove:
```rust
let webview_src_dir = self.output_dir.join("webview-ui").join("src");
let webview_components_dir = webview_src_dir.join("components");
// ...
fs::create_dir_all(&webview_components_dir)
    .map_err(|e| format!("Failed to create webview-ui/src/components: {}", e))?;
```

**Step 2: Remove webview write calls from generate()**

Remove these lines from `generate()` (~line 330-339):
```rust
self.write_webview_index_html()?;
self.write_webview_main_ts()?;
self.write_webview_app_vue()?;
self.write_webview_package_json()?;
self.write_webview_vite_config()?;
self.write_webview_tsconfig()?;
self.write_webview_env_dts()?;
self.write_tailwind_config()?;
self.write_postcss_config()?;
self.write_base_css()?;
```

**Step 3: Remove sub-widget component writing**

Remove the block that writes sub-widget components to webview_components_dir (~line 347-366):
```rust
for (relative_dir, name, code, widget_name) in &self.components {
    // ... entire block
}
```

**Step 4: Delete the write_webview_* method bodies**

Delete or empty these methods (keep stubs that return `Ok(())` to avoid breaking the impl block):
- `write_webview_index_html()` (~line 420)
- `write_webview_main_ts()` (~line 429)
- `write_webview_app_vue()` (~line 436)
- `write_webview_package_json()` (~line 446)
- `write_webview_vite_config()` (~line 482)
- `write_webview_tsconfig()` (~line 491)
- `write_webview_env_dts()` (~line 500)
- `write_tailwind_config()` (~line 509)
- `write_postcss_config()` (~line 518)
- `write_base_css()` (~line 527)

**Step 5: Update .vscodeignore template**

In the `.vscodeignore` generation (~line 1200-1209), remove the webview-ui lines:
```
webview-ui/src/**
webview-ui/node_modules/**
webview-ui/index.html**
```

And update the tsconfig exclude (~line 1153):
```rust
// Before:
"exclude": ["node_modules", ".vscode-test", "webview-ui"]
// After:
"exclude": ["node_modules", ".vscode-test"]
```

**Step 6: Build and verify**

Run: `cargo build -p auto`
Expected: Compiles without errors

**Step 7: Commit**

```
refactor(vscode): remove webview-ui file generation
```

---

### Task 4: Update build_vscode_project — add Vue build prerequisite

**Files:**
- Modify: `crates/auto-man/src/vscode.rs` — `build_vscode_project()` function (~line 592)

**Step 1: Add Vue build step before extension generation**

In `build_vscode_project()`, after Step 1 (generate_vscode_project), add a new step to ensure gen/vue/ is built. Insert before the existing Step 2:

```rust
// Step 1.5: Ensure gen/vue is built (prerequisite for webview assets)
let vue_dir = root_dir.join("gen").join("vue");
let vue_dist = vue_dir.join("dist");

if !vue_dist.join("assets").join("index.js").exists() {
    println!();
    println!(
        "{}",
        "  Building Vue project (webview dependency)...".bright_cyan()
    );

    // Generate Vue project first
    let vue_output = vue_dir.clone();
    crate::vue::build_vue_project(&root_dir)?;

    // Install and build Vue
    #[cfg(windows)]
    let npm_install = std::process::Command::new("cmd")
        .args(&["/C", "npm", "install"])
        .current_dir(&vue_dir)
        .status();

    #[cfg(not(windows))]
    let npm_install = std::process::Command::new("npm")
        .args(&["install"])
        .current_dir(&vue_dir)
        .status();

    match npm_install {
        Ok(status) if status.success() => {
            println!("  {} Vue dependencies installed", "OK".bright_green());
        }
        _ => {
            println!(
                "  {} Failed to install Vue dependencies",
                "Warning:".bright_yellow()
            );
        }
    }

    #[cfg(windows)]
    let npm_build = std::process::Command::new("cmd")
        .args(&["/C", "npm", "run", "build"])
        .current_dir(&vue_dir)
        .status();

    #[cfg(not(windows))]
    let npm_build = std::process::Command::new("npm")
        .args(&["run", "build"])
        .current_dir(&vue_dir)
        .status();

    match npm_build {
        Ok(status) if status.success() => {
            println!("  {} Vue project built", "OK".bright_green());
        }
        _ => {
            return Err("Vue build failed — cannot generate VSCode extension without webview assets".into());
        }
    }
}
```

**Step 2: Remove webview-ui install from build flow**

In `build_vscode_project()`, remove the entire webview-ui install block (~line 649-676):
```rust
let webview_dir = vscode_dir.join("webview-ui");
// Install webview-ui dependencies if needed
if webview_dir.exists() && !webview_dir.join("node_modules").exists() {
    // ... entire block
}
```

**Step 3: Build and verify**

Run: `cargo build -p auto`
Expected: Compiles without errors

**Step 4: Commit**

```
feat(vscode): add Vue build prerequisite, remove webview-ui install
```

---

### Task 5: Remove compile_at_to_vue and components from VscodeProject

**Files:**
- Modify: `crates/auto-man/src/vscode.rs` — `VscodeProject` struct, `from_workspace()`, `compile_at_to_vue()`

**Step 1: Remove components field and app_vue_code from VscodeProject**

In `VscodeProject` struct (~line 104-119), remove:
```rust
pub app_vue_code: String,
pub components: Vec<(String, String, String, String)>,
```

**Step 2: Remove component compilation from from_workspace()**

In `from_workspace()` (~line 152-275), remove all the `.at` file compilation code that calls `compile_at_to_vue()` and populates `all_components`. The function should only parse the project name, config, and front directory — no Vue code generation.

**Step 3: Delete compile_at_to_vue() function entirely**

Delete `fn compile_at_to_vue()` (~line 1338-1375). Also remove the `VueGenerator` import if it becomes unused.

**Step 4: Build and verify**

Run: `cargo build -p auto`
Expected: Compiles without errors. May need to remove unused import warnings.

**Step 5: Commit**

```
refactor(vscode): remove Vue code generation — now reuses gen/vue/dist/
```

---

### Task 6: End-to-end verification

**Step 1: Build the auto binary**

Run: `cargo build -p auto`

**Step 2: Test Vue generation still works independently**

```bash
cd examples/ui/002-counter
rm -rf gen/
cargo run -p auto -- gen
ls gen/vue/dist/assets/index.js  # Should exist
```

Expected: Vue project generates and builds successfully.

**Step 3: Test VSCode extension generation and run**

```bash
cd examples/ui/002-counter
rm -rf gen/
AUTO_BACKEND=vscode cargo run -p auto -- run
```

Expected output should show:
1. "Building Vue project (webview dependency)..."
2. Vue npm install + build
3. "Generating VSCode extension code..."
4. Extension scaffold generated (no webview-ui/)
5. "VSCode extension built successfully!"
6. VSCode opens

**Step 4: Verify no webview-ui/ in generated output**

```bash
ls gen/vscode/  # Should NOT contain webview-ui/
ls gen/vscode/src/panels/AppPanel.ts  # Should exist
```

**Step 5: Commit**

```
test: verify VSCode extension reuses Vue build output
```

---

### Task 7: Update existing tests

**Files:**
- Modify: `crates/auto-man/src/vscode.rs` — test functions at bottom of file

**Step 1: Update test assertions**

Find and update any tests that assert webview-ui files exist or check for `webview:build` in package.json. The test at ~line 1523 asserts:
```rust
assert!(json.contains(r#"webview:build"#));
```

Change to verify the new structure:
```rust
assert!(json.contains(r#"npm install && npm run compile"#));
```

Remove any assertions about webview-ui/ directory contents.

**Step 2: Run tests**

Run: `cargo test -p auto-man -- vscode`
Expected: All tests pass

**Step 3: Commit and push**

```
test: update VSCode generator tests for shared vue dist
git push
```
