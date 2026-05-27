# Smart Regeneration for `auto run --backend=rust`

## Context

`run_rust_ui()` in `crates/auto-man/src/rust_ui.rs` currently only checks `Cargo.toml` existence. If it exists, it **always skips** regeneration — even when the `.at` source files have changed, or when the RustGenerator code itself has been updated. This forces manual deletion of `gen/rust/` during development to pick up codegen changes.

The user wants `auto run --backend=rust` (and `auto build`) to automatically detect when regeneration is needed and do the right thing:
- **Source changed** → regenerate `main.rs` only (fast, keeps Cargo.toml/cargo cache)
- **Backend feature changed** (e.g. gpui → iced) → full regeneration including Cargo.toml
- **Nothing changed** → skip regeneration entirely (fastest)

## Current Behavior

`run_rust_ui()` (line 389-396):
```rust
if !rust_dir.join("Cargo.toml").exists() {
    generate_rust_ui(project_dir, None, false)?;
}
// ... cargo run
```

No timestamp/hash comparison at all. Cargo.toml exists = skip.

## Plan

### Change: `run_rust_ui()` and `generate_rust_ui()` in `rust_ui.rs`

Add a `needs_regeneration()` function that checks three things:

1. **Cargo.toml missing** → full regeneration needed
2. **Any `.at` source file newer than `main.rs`** → code regeneration needed (not Cargo.toml)
3. **Cargo.toml `default` feature differs from current backend** → full regeneration needed

Then split the generation into two paths:

- `generate_rust_ui()` — full generation (Cargo.toml + main.rs), same as now
- `regenerate_code_only()` — only rewrite `main.rs`, skip Cargo.toml (new helper)

### Implementation

**Step 1**: Add `needs_regeneration()` function:

```rust
/// Check if the generated Rust project needs to be regenerated.
/// Returns (needs_full_regen: bool, needs_code_regen: bool)
fn needs_regeneration(project_dir: &Path, rust_dir: &Path) -> (bool, bool) {
    let cargo_toml = rust_dir.join("Cargo.toml");
    let main_rs = rust_dir.join("src/main.rs");

    // No Cargo.toml at all → full regeneration
    if !cargo_toml.exists() || !main_rs.exists() {
        return (true, true);
    }

    // Check if any .at source file is newer than main.rs
    let front_dir = find_front_dir(project_dir);
    if let Ok(at_files) = collect_at_files(&front_dir) {
        if let Ok(main_meta) = fs::metadata(&main_rs) {
            if let Ok(main_time) = main_meta.modified() {
                for at_file in &at_files {
                    if let Ok(at_meta) = fs::metadata(at_file) {
                        if let Ok(at_time) = at_meta.modified() {
                            if at_time > main_time {
                                return (false, true); // source changed, code only
                            }
                        }
                    }
                }
            }
        }
    }

    // Check if default feature in Cargo.toml matches current expectation
    // (e.g. was generated with ui-gpui but now we want ui-iced)
    if let Ok(content) = fs::read_to_string(&cargo_toml) {
        let expected_default = "default = [\"ui-iced\"]";
        let has_expected = content.contains(expected_default);
        if !has_expected {
            return (true, true); // feature mismatch → full regen
        }
    }

    (false, false) // everything up to date
}
```

**Step 2**: Update `run_rust_ui()`:

```rust
pub fn run_rust_ui(project_dir: &Path, args: Vec<String>) -> AutoResult<()> {
    let rust_dir = project_dir.join("gen").join("rust");
    let (full, code) = needs_regeneration(project_dir, &rust_dir);

    if full {
        println!("{}", "Generating Rust UI project...".bright_cyan());
        generate_rust_ui(project_dir, None, false)?;
    } else if code {
        println!("{}", "Regenerating Rust UI code (source changed)...".bright_cyan());
        regenerate_code_only(project_dir, &rust_dir)?;
    }

    // ... cargo run (unchanged)
}
```

**Step 3**: Add `regenerate_code_only()` — extracts the code generation part from `generate_rust_ui()` without rewriting Cargo.toml:

```rust
fn regenerate_code_only(project_dir: &Path, rust_dir: &Path) -> AutoResult<()> {
    let front_dir = find_front_dir(project_dir);
    let at_files = collect_at_files(&front_dir)?;
    if at_files.is_empty() { return Ok(()); }

    let pac_path = project_dir.join("pac.at");
    let project_name = if pac_path.exists() {
        parse_pac_name(&pac_path).unwrap_or_else(|| "MyApp".to_string())
    } else {
        "MyApp".to_string()
    };

    // Compile .at files → Rust code (same logic as generate_rust_ui)
    let mut all_components = String::new();
    for at_path in &at_files {
        match compile_at_file(at_path) {
            Ok(code) => { all_components.push_str(&code); all_components.push('\n'); }
            Err(e) => println!("{} Failed to compile: {}", "Warning:".bright_yellow(), e),
        }
    }

    let full_code = wrap_example(&project_name, &all_components);
    let main_rs = rust_dir.join("src").join("main.rs");
    fs::write(&main_rs, &full_code)?;
    Ok(())
}
```

**Step 4**: Extract `find_front_dir()` helper to avoid duplicating the front/ directory resolution logic.

### Files Modified

- `crates/auto-man/src/rust_ui.rs` — all changes here

### Verification

1. `cargo build -p auto-man` — compiles
2. Run `auto run --dir examples/ui/004-profile-card --backend=rust` twice:
   - First run: regenerates if source changed
   - Second run: skips regeneration (source unchanged)
3. Touch `app.at`: `touch examples/ui/004-profile-card/src/front/app.at`, then `auto run` → should detect change and regenerate code only
4. `cargo test -p auto-man` — existing tests pass
5. Manually test the full-regeneration path by changing Cargo.toml's default feature
