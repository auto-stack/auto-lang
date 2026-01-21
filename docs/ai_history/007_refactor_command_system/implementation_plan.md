# Implementation Plan - auto-shell Command System Refactor

## Goal
Refactor the current ad-hoc command execution (hardcoded `match` in `builtin.rs`) to a structured, extensible **Plugin/Command Architecture** inspired by Nushell.

## User Review Required
> [!IMPORTANT]
> This is a major refactor. The shell implementation will change significantly.
> - `src/cmd.rs` will be expanded to define the core traits.
> - `src/cmd/builtin.rs` will be broken down into individual command structs.
> - `Signature` based argument parsing will replace manual `parse_args` in many places.

## Proposed Changes

### 1. Core Architecture (`src/cmd.rs`)
Define the fundamental traits and structs:

```rust
pub struct Signature {
    pub name: String,
    pub description: String,
    pub arguments: Vec<Argument>,
    // ... flags etc.
}

pub trait Command {
    fn name(&self) -> &str;
    fn signature(&self) -> Signature;
    fn run(&self, args: &[String], shell: &mut Shell) -> Result<Option<String>>;
}
```

### 2. Command Registry (`src/cmd/registry.rs`) - [NEW]
A struct to map command names to `Command` trait objects.

```rust
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
}
```

### 3. Refactor Built-ins
[x] Move commands from `src/cmd/builtin.rs` to specific submodules or new files, implementing the `Command` trait.
- `pwd`
- `cd`
- `ls`
- `echo`
- `help`

### 4. Updates to `Shell` struct (`src/shell.rs`)
[x] Add `command_registry: CommandRegistry` to `Shell`.
[x] Initialize it in `Shell::new()` with all built-ins.
[x] Update `execute()` to look up commands in the registry instead of calling internal helpers.

## Verification Plan

### Automated Tests
- Unit tests for `CommandRegistry`.
- Unit tests for new individual command structs.

### Manual Verification
- Run `help` -> should auto-generate from Signatures.
- Run `ls`, `cd`, `pwd`, `echo` -> should behave exactly as before.
