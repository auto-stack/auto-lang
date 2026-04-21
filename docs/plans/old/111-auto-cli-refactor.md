# 111 - Auto CLI Refactor & Unification

## 1. Overview
The goal of this refactor is to fully merge the `auto-man` CLI (`am.exe`) capabilities into the universal `auto` CLI (`auto.exe`) and standardize the command structure based on the `docs/design/auto-cli.md` specification. By letting `pac.at` control the project backend ("smart configuration"), we can provide a minimal and stable set of verbs ("dumb CLI") that effortlessly transitions between C, Rust, Vue, and Auto project types.

## 2. CLI Restructuring
The target `auto.exe` command structure will act primarily on categories: Execution, Project Creation, Build & Run, Dependencies, Hardware, Project Utils, and Environment.

### Commands to Remove / Replace
Remove the following flat commands from `crates/auto/src/main.rs` and `crates/auto-man/src/main.rs`:
- `App`, `Lib`, `Capp`, `Clib` &rarr; Replaced by `auto new <name> -t <template>`
- `Scan`, `Pull` &rarr; Merged into `auto fetch`
- `Vue`, `Tauri` (as project generators) &rarr; Absorbed into `auto new` templates and `auto run/build`
- `Devices`, `Port` &rarr; Collapsed under `auto device list/select`
- `Reset`, `Install` &rarr; Collapsed under `auto env reset/install`
- `Ui` &rarr; Handled by the generic `auto build/run` mechanism interacting with `pac.at`.

### Target CLI Structure (`auto/src/main.rs`)
```rust
#[derive(Parser)]
struct Cli {
    #[arg(index = 1, help = "Run an Auto script directly via AutoVM")]
    file: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

enum Commands {
    // Project Creation
    New { name: String, #[arg(short, long)] template: Option<String> },
    Init,

    // Build & Run
    Build { #[arg(short, long)] dir: Option<String> },
    Run { args: Vec<String> },
    Clean { #[arg(short, long)] dir: Option<String> },

    // Dependencies
    Add { package: String },
    Fetch, // Replaces scan and pull
    Deps,

    // Hardware & Embedded
    Device { #[command(subcommand)] action: DeviceAction },

    // Project Utils
    Info { #[arg(short, long)] target: Option<String> },
    Open,

    // Environment
    Upgrade,
    Env { #[command(subcommand)] action: EnvAction },
}
```

## 3. Step-by-Step Implementation Plan

### Step 1: Update `auto` CLI Arguments and Parser
- Modify `crates/auto/src/main.rs` to implement the new `Cli` struct.
- Setup positional parameters such that `auto` with no args launches the repl, and `auto script.at` executes the file directly.
- Add implementations for the unified `Commands` enum, stubbing out the new handlers.

### Step 2: Implement `auto new` and `auto fetch` in `auto-man` library
- Update `crates/auto-man/src/lib.rs` (or `automan.rs`) to provide a unified `Automan::new_project(name, template)` function.
- `template` argument will dictate the initialization of `pac.at` (pure auto, `c-app`, `rs-app`, `vue-app`, `gadget`).
- Create `Automan::fetch()` combining the discovery flow of `scan` and the downloading flow of `pull`. 

### Step 3: Implement Smart Build/Run Router
- Modify `Automan::build` and `Automan::run` to read the `backend` field from `pac.at`.
  - **rust**: invoke `cargo build` / `cargo run` internally.
  - **c**: invoke Ninja and C-toolchain compilation.
  - **vue**: invoke AST transformation mapped to standard Vite tooling (`npm run build`).
- `build` should perform an automatic, silent `fetch` step before building.

### Step 4: Implement Subcommands (`device`, `env`)
- Convert old `list_devices()` and `select_port()` functions into handlers for the new `auto device list` and `auto device select <PORT>` subcommands.
- Group cache management and system resets under `auto env`. Keep the `cache` subcommand logic, potentially mapping it under `env cache` or leaving it beside `env`.

### Step 5: Clean up backwards-compatibility leftovers
- Gut the unused specific generators inside `auto/src/main.rs` corresponding to `vue`, `tauri`, etc.
- In `crates/auto-man/src/main.rs` (`am.exe`), replace the entire CLI body with a deprecation warning that instructs users to use `auto.exe` directly, or simply remove `am` completely from `Cargo.toml` if the team is ready for a hard cut-over.

## 4. Risks and Considerations
- Make sure `pac.at` cleanly exposes the `backend` key.
- The implicit "Run script" behaviour mapping to a raw positional file argument might conflict with commands if someone names their file "build". Clap's typical parsing resolves matching Subcommands over positional arguments, so this should naturally behave as intended.
