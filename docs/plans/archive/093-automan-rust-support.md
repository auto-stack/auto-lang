# Auto-Man Redesign for Rust Support

## Goal Description
Currently, `auto-man` is heavily optimized for Auto/C mixed projects. It acts as a package manager that retrieves `.at` and `.c`/`.h` files, transpiles `.at` to `.c`, and drives a C build system (CMake, Ninja, IAR, GHS) using a custom `Builder` trait. 

To support Auto/Rust mixed projects, `auto-man` must evolve to act as a transpiler-frontend for Cargo. It needs to read project configuration (`pac.at`), transpile `.at` files to `.rs` code, assemble dynamically generated `Cargo.toml` files mapping the project tree, and invoke `cargo` as the backend builder.

This document outlines the architectural changes required to support both **Auto/C** and **Auto/Rust** mixed projects within `auto-man`.

## Proposed Changes

### 1. Target Language Specification (`pac.rs`, `pac.at`)
Currently, `auto-man` assumes everything transpiles to C. We need to introduce the concept of a "target language" (C vs. Rust) applied to Pacs/Targets.

*   **Action:** Add `lang` (e.g., `"rust"`, `"c"`) to `pac.at` parameters. We can default to `"c"` for backward compatibility during the transition, but eventually default to `"rust"`.
*   **Action:** Expose this up to `Target` so each target knows what it's generating. We also need to add a `TargetKind::RustLib`, `TargetKind::RustApp`, or generalize `TargetKind::App` with a language label.

### 2. File Type Registration (`file_types.rs`)
`file_types.rs` rigidly expects `.c`, `.h`, `.s`, `.gpj`, etc. It has no concept of Rust files.

*   **Action:** Add `RustSource` (`.rs`) to the `FileType` enum.
*   **Action:** Update `FileFilter::for_target` to allow `.rs` files when the target language is Rust.

### 3. Transpilation Abstraction (`target.rs`)
`Target::transpile_auto()` currently hardcodes `transpile_c`. This logic must become polymorpic based on the target language.

*   **Action:** Refactor `Target::transpile_auto()`:
    *   If `lang == "c"`: Output `.c` and `.h` using `transpile_c`. Add them to `self.srcs` and `self.incs`.
    *   If `lang == "rust"`: Output `.rs` using the existing `transpile_rust` function from the `auto-lang` crate. Save them into `src/` inside the build directory to satisfy standard Cargo layout. Add the outputs to the target's source set.

### 4. Builder Trait Extensions (`builder.rs`)
The `Builder` trait abstracts the mechanics of setting up and building a project. We need a `CargoBuilder` implementation.

*   **Action:** Add a `Cargo` variant to `BuilderKind` (`pub enum BuilderKind { CMake, IAR, GHS, Ninja, **Cargo** }`).
*   **Action:** Introduce `mod cargo;` under `src/builder/`.
*   **Action:** Implement `Builder` for `CargoBuilder`:
    *   `setup()`: Generate a valid `Cargo.toml` dynamically. It should inspect the `Target`'s dependencies (from `self.deps`) and map them to local path dependencies in the workspace `Cargo.toml`. Create standard Rust project layouts (`src/main.rs` for `App`, `src/lib.rs` for `Lib` containing `mod` declarations for generated `.rs` files).
    *   `build()`: Invoke `cargo build --manifest-path <out-dir>/Cargo.toml`.
    *   `run()`: Invoke `cargo run --manifest-path ...`.
    *   `clean()`: Invoke `cargo clean`.

### 5. Dependency Management (`target.rs`, `resolver.rs`)
How do `auto-man` targets translate to Rust crates? When `auto-man` downloads another `.at` project, how does Cargo see it?

*   **Action:** Every `Target` parsed out of `pac.at` (that's marked as Rust) becomes a Cargo crate.
*   **Action:** Use a virtual Cargo workspace at the top level of `auto-man`'s build folder (`build/Cargo.toml`). This workspaces encompasses all local targets (`[workspace] members = ["app", "lib_dep1", ...]`).
*   **Action:** When `Target` builds its `Cargo.toml`, it automatically translates `self.deps` into Cargo path dependencies pointing to sibling directories built by `auto-man`.

### 6. Adjusting `Port` defaults (`port.rs`)
The `Port::default()` method returns a Ninja builder. This will need to check if user environment defaults to cargo, or be explicitly dictated by the project configuration.

### Workflow Example for Auto/Rust:
1.  **Parse:** `automan.rs` reads `pac.at` which says `builder: "cargo", lang: "rust"`.
2.  **Transpile:** `Target::transpile_auto` finds `main.at`. Sees `lang: "rust"`, calls `transpile_rust(..., "main.at")` → yields `main.rs`. Writes it to `build/app/src/main.rs`.
3.  **Setup:** `CargoBuilder::setup()` generates `build/app/Cargo.toml` and a workspace `build/Cargo.toml`.
4.  **Build:** `CargoBuilder::build()` executes `cargo build` inside `build/`.

## User Review Required
1.  **Project Layout**: Does generating a full Cargo workspace in the `build/` directory with a dynamic `Cargo.toml` align with your vision, or do you expect the `.at` transpilation to integrate into a pre-existing `Cargo.toml` file checked into Git?
2.  **Backwards Compatibility**: Can we enforce a configuration parameter like `lang: "rust"` in `pac.at`, or should `auto-man` infer the backend from the specified `builder` (e.g., if `builder` == `"cargo"`, then `lang` == `"rust"`)?
