# Implementation Plan - Shell Shortcuts

## Goal
Implement three shortcut commands to enhance shell productivity: `l` (ls), `u` (up), and `b` (bookmarks).

## Proposed Changes

### Shared Modules
#### [NEW] [src/bookmarks.rs](file:///d:/autostack/auto-lang/auto-shell/src/bookmarks.rs)
- Struct `BookmarkManager` with methods: `load`, `save`, `add`, `del`, `get`, `list`.
- Persistence: `~/.auto-shell-bookmarks` (simple text format: `name=path`).
- Needs to be accessible from both `Shell` and `Completer`.

#### [src/lib.rs](file:///d:/autostack/auto-lang/auto-shell/src/lib.rs)
- Export `mod bookmarks;`.

### Shell Logic
#### [src/shell.rs](file:///d:/autostack/auto-lang/auto-shell/src/shell.rs)
- Integrate `bookmarks::BookmarkManager` into `Shell` struct.
- In `execute()`:
    - Handle `u` command: `cd ..` or `cd ../..` based on argument.
    - Handle `b` command: execution logic (`add`, `del`, jump).

### Builtin Commands
#### [src/cmd/builtin.rs](file:///d:/autostack/auto-lang/auto-shell/src/cmd/builtin.rs)
- Add `l` case mapping to `ls` logic.

### Completions
#### [src/completions.rs](file:///d:/autostack/auto-lang/auto-shell/src/completions.rs)
- Detect `b` command context.
- Load `BookmarkManager` and suggest bookmark names.

## Verification Plan
### Manual Verification
- `l`: `l src` should list src directory.
- `u`: `u` -> parent. `u 2` -> grandparent.
- `b`:
    - `b add foo`: Adds bookmark.
    - `b<TAB>`: Lists `foo`.
    - `b foo`: Jumps to dir.
    - `b del foo`: Removes bookmark.
