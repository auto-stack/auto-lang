# Auto-Shell: Enhanced `ls` Command with Coreutils Flags

## Objective

Enhance the `ls` command in auto-shell to support common coreutils flags: `-l` (long format), `-a` (all files), `-h` (human-readable), `-t` (time sort), `-r` (reverse), and `-R` (recursive).

## Current State

### Existing Implementation
- **File**: [auto-shell/src/cmd/commands/ls.rs](auto-shell/src/cmd/commands/ls.rs)
- **Functionality**: Basic directory listing with table output
- **Output**: Name, Size, Modified columns
- **Limitations**: No flag support, always hides hidden files, fixed table format

### Supporting Infrastructure Already in Place
- ✅ `ParsedArgs` supports flags via `has_flag()` method
- ✅ `Signature` supports `.flag()` for defining flags
- ✅ `fs::ls_command()` has table formatting infrastructure
- ✅ `FileEntry` struct with metadata support

## Design Approach

### Flag Implementations

#### 1. **`-a` / `--all`** (Show All Files)
- **Implementation**: Filter out entries starting with `.` by default
- **When flag set**: Show all entries including `.` and `..`
- **Complexity**: Low - simple filter change

#### 2. **`-l` / `--long`** (Long Format)
- **Implementation**: Add columns to table output
- **New columns**: Permissions, Owner, Group, Size, Modified
- **Permissions format**: `rwxrwxrwx` (Unix-style)
- **Cross-platform**: Use `std::os::unix::fs::PermissionsExt` on Unix, simulate on Windows
- **Complexity**: Medium - requires platform-specific permission handling

#### 3. **`-h` / `--human-readable`** (Human-Readable Sizes)
- **Current behavior**: Already implemented in `FileEntry::format_size()`
- **Implementation**: Add flag to control format (bytes vs human-readable)
- **Formats**:
  - Human: `1.2K`, `3.4M`, `1.5G`
  - Bytes: `1234`, `3565158`, `1610612736`
- **Complexity**: Low - just toggling existing formatter

#### 4. **`-t` / `--time`** (Sort by Time)
- **Current sort**: Directories first, then alphabetically by name
- **New sort**: By modification time (newest first)
- **Implementation**: Change comparison key in `files.sort_by()`
- **Complexity**: Low - simple sort change

#### 5. **`-r` / `--reverse`** (Reverse Order)
- **Implementation**: Reverse the sorted array
- **Interaction**: Combines with any sort method (alphabetical or time)
- **Complexity**: Low - single `.reverse()` call

#### 6. **`-R` / `--recursive`** (Recursive Listing)
- **Implementation**: DFS traversal of subdirectories
- **Output format**:
  ```
  ./src:
  file1.rs  file2.rs  subdir/

  ./src/subdir:
  nested.rs

  ./tests:
  test.rs
  ```
- **Complexity**: Medium - requires recursive function with path tracking

### Cross-Platform Considerations

#### Unix/Linux/macOS
- Full permission support via `std::os::unix::fs::PermissionsExt`
- Owner/group via `std::os::unix::fs::MetadataExt`
- Symbolic links via `fs::symlink_metadata()`

#### Windows
- Simulated permissions (read-only attribute)
- No owner/group (show `-`/`N/A`)
- Junctions/symlinks via `fs::symlink_metadata()`

### Short Flag Aliases

Support both short and long forms:
- `-a` == `--all`
- `-l` == `--long`
- `-h` == `--human-readable`
- `-t` == `--time`
- `-r` == `--reverse`
- `-R` == `--recursive`

## Implementation Plan

### Phase 1: Update Command Signature (ls.rs)

**File**: `auto-shell/src/cmd/commands/ls.rs`

Add flags to signature:
```rust
fn signature(&self) -> Signature {
    Signature::new("ls", "List directory contents")
        .optional("path", "Path to list")
        .flag("all", "Show all files including hidden (starts with .)")
        .flag("long", "Long listing format (permissions, owner, size, time)")
        .flag("human-readable", "Human-readable file sizes (1K, 234M, 2G)")
        .flag("time", "Sort by modification time (newest first)")
        .flag("reverse", "Reverse sort order")
        .flag("recursive", "List subdirectories recursively")
}
```

Extract flags in `run()` method:
```rust
fn run(&self, args: &ParsedArgs, _input: Option<&str>, shell: &mut Shell) -> Result<Option<String>> {
    let path_arg = args.positionals.get(0).map(|s| s.as_str()).unwrap_or(".");
    let path = Path::new(path_arg);

    let all = args.has_flag("all");
    let long = args.has_flag("long");
    let human = args.has_flag("human-readable");
    let time = args.has_flag("time");
    let reverse = args.has_flag("reverse");
    let recursive = args.has_flag("recursive");

    let output = fs::ls_command(
        path,
        &shell.pwd(),
        all,
        long,
        human,
        time,
        reverse,
        recursive,
    )?;
    Ok(Some(output))
}
```

### Phase 2: Update fs::ls_command Function

**File**: `auto-shell/src/cmd/fs.rs`

#### 2.1 Update Function Signature
```rust
pub fn ls_command(
    path: &Path,
    current_dir: &Path,
    all: bool,
    long: bool,
    human: bool,
    time_sort: bool,
    reverse: bool,
    recursive: bool,
) -> Result<String>
```

#### 2.2 Implement `-a` (All Files)
```rust
// Around line 43, after getting name
if !all && file.name.starts_with('.') {
    continue;  // Skip hidden files
}
```

#### 2.3 Implement Sort Logic
```rust
// Replace lines 71-78 (current sort)
files.sort_by(|a, b| {
    let cmp = if time_sort {
        // Sort by modification time (newest first)
        b.modified.as_ref().unwrap_or(&String::new())
            .cmp(a.modified.as_ref().unwrap_or(&String::new()))
    } else {
        // Sort alphabetically
        a.name.cmp(&b.name)
    };

    // Directories first
    if a.is_dir != b.is_dir {
        b.is_dir.cmp(&a.is_dir)
    } else {
        cmp
    }
});

if reverse {
    files.reverse();
}
```

#### 2.4 Implement Recursive Listing
```rust
// New helper function
fn list_recursive(path: &Path, all: bool, long: bool, human: bool, time_sort: bool, reverse: bool) -> Result<String> {
    let mut output = String::new();
    output.push_str(&format!("{}:\n", path.display()));

    // List current directory (non-recursive call)
    output.push_str(&ls_command(path, path, all, long, human, time_sort, reverse, false)?);

    // Find subdirectories and recurse
    let entries = fs::read_dir(path).into_diagnostic()?;
    for entry in entries {
        let entry = entry.into_diagnostic()?;
        if entry.path().is_dir() {
            let name = entry.file_name().into_string().unwrap_or_default();
            if !all && name.starts_with('.') {
                continue;
            }
            output.push_str("\n");
            output.push_str(&list_recursive(&entry.path(), all, long, human, time_sort, reverse)?);
        }
    }

    Ok(output)
}

// In ls_command, around line 101
if recursive {
    return list_recursive(&target, all, long, human, time_sort, reverse);
}
```

#### 2.5 Implement Long Format (-l)
```rust
// Conditional table creation
let mut table = if long {
    Table::new()
        .add_column(Column::new("Permissions").align(Align::Left))
        .add_column(Column::new("Owner").align(Align::Left))
        .add_column(Column::new("Size").align(Align::Right))
        .add_column(Column::new("Modified").align(Align::Left))
        .add_column(Column::new("Name").align(Align::Left))
} else {
    Table::new()
        .add_column(Column::new("Name").align(Align::Left))
        .add_column(Column::new("Size").align(Align::Right))
        .add_column(Column::new("Modified").align(Align::Left))
};

// In row addition (lines 87-98)
for file in &files {
    if long {
        let perms = format_permissions(&metadata);
        let owner = get_owner(&metadata);
        let size_str = if human {
            file.format_size()
        } else {
            file.size.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())
        };
        let name_with_indicator = if file.is_dir {
            format!("{}/", file.name)
        } else {
            file.name.clone()
        };

        table = table.add_row(vec![
            perms,
            owner,
            size_str,
            file.modified.clone().unwrap_or_else(|| "-".to_string()),
            name_with_indicator,
        ]);
    } else {
        // Original logic
        // ...
    }
}
```

#### 2.6 Platform-Specific Permission Helper
```rust
// New function at bottom of fs.rs
#[cfg(unix)]
fn format_permissions(metadata: &fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    let file_type = if metadata.is_dir() {
        'd'
    } else if metadata.file_type().is_symlink() {
        'l'
    } else {
        '-'
    };

    let user = format_mode_bits(mode & 0o700);
    let group = format_mode_bits(mode & 0o070);
    let other = format_mode_bits(mode & 0o007);

    format!("{}{}{}{}", file_type, user, group, other)
}

#[cfg(unix)]
fn format_mode_bits(bits: u32) -> String {
    format!(
        "{}{}{}",
        if bits & 0o400 != 0 { 'r' } else { '-' },
        if bits & 0o200 != 0 { 'w' } else { '-' },
        if bits & 0o100 != 0 { 'x' } else { '-' }
    )
}

#[cfg(unix)]
fn get_owner(metadata: &fs::Metadata) -> String {
    use std::os::unix::fs::MetadataExt;
    metadata.uid().to_string()
}

#[cfg(windows)]
fn format_permissions(_metadata: &fs::Metadata) -> String {
    // Windows: show read-only attribute
    "-r--r--r--".to_string()  // Simplified
}

#[cfg(windows)]
fn get_owner(_metadata: &fs::Metadata) -> String {
    "-".to_string()
}
```

#### 2.7 Update FileEntry for Metadata
```rust
// Extend FileEntry to store full metadata
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<String>,
    pub metadata: Option<fs::Metadata>,  // Add this field
}

// In ls_command, store metadata
files.push(FileEntry {
    name,
    is_dir,
    size,
    modified,
    metadata: Some(metadata),  // Store it
});
```

### Phase 3: Tests

**File**: `auto-shell/src/cmd/fs.rs` (bottom of file)

Add comprehensive tests:
```rust
#[test]
fn test_ls_with_hidden_files() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path();
    fs::write(path.join(".hidden"), "test").unwrap();
    fs::write(path.join("visible"), "test").unwrap();

    let result = ls_command(path, path, false, false, false, false, false, false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.contains(".hidden"));
    assert!(output.contains("visible"));
}

#[test]
fn test_ls_all_flag() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path();
    fs::write(path.join(".hidden"), "test").unwrap();

    let result = ls_command(path, path, true, false, false, false, false, false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains(".hidden"));
}

#[test]
fn test_ls_time_sort() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path();
    let f1 = path.join("a.txt");
    let f2 = path.join("b.txt");
    fs::write(&f1, "first").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    fs::write(&f2, "second").unwrap();

    let result = ls_command(path, path, true, false, false, true, false, false);
    let output = result.unwrap();
    // b.txt should appear before a.txt (newer first)
    let pos_b = output.find("b.txt").unwrap();
    let pos_a = output.find("a.txt").unwrap();
    assert!(pos_b < pos_a);
}

#[test]
fn test_ls_reverse() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path();
    fs::write(path.join("a.txt"), "").unwrap();
    fs::write(path.join("b.txt"), "").unwrap();

    let result = ls_command(path, path, true, false, false, false, true, false);
    let output = result.unwrap();
    let pos_b = output.find("b.txt").unwrap();
    let pos_a = output.find("a.txt").unwrap();
    assert!(pos_b < pos_a);  // Reverse alphabetical
}
```

### Phase 4: Documentation

Update help text in signature descriptions with usage examples.

## Critical Files Summary

### Files to Modify
1. **[auto-shell/src/cmd/commands/ls.rs](auto-shell/src/cmd/commands/ls.rs)**
   - Update `signature()` to add 6 flags
   - Update `run()` to extract and pass flags to fs function

2. **[auto-shell/src/cmd/fs.rs](auto-shell/src/cmd/fs.rs)**
   - Update `ls_command()` signature to accept 6 boolean parameters
   - Implement `-a` filter logic (line ~43)
   - Implement sort logic for `-t` and `-r` (line ~71)
   - Implement recursive listing for `-R` (new helper function)
   - Implement long format for `-l` (conditional table creation)
   - Add platform-specific permission helpers (new functions)
   - Update `FileEntry` struct to include metadata
   - Add comprehensive tests

### Files to Reference
- **[auto-shell/src/cmd/parser.rs](auto-shell/src/cmd/parser.rs)** - Already supports flag parsing
- **[auto-shell/src/data/table.rs](auto-shell/src/data/table.rs)** - Table rendering system
- **[auto-shell/src/cmd.rs](auto-shell/src/cmd.rs)** - Signature and Argument structs

## Verification Strategy

### Manual Testing
```bash
# Start auto-shell
cd auto-shell
cargo run

# Test each flag
ls                  # Default behavior
ls -a               # Show hidden files
ls -l               # Long format
ls -lh              # Long with human-readable sizes
ls -lt              # Long, sorted by time
ls -ltr             # Long, time sort, reverse
ls -R               # Recursive
ls -aR              # All files, recursive

# Test path handling
ls /etc             # Absolute path
ls ..               # Parent directory
ls nonexistent      # Error handling
```

### Expected Output Examples

#### Default (`ls`)
```
Name       Size  Modified
src/       -     2025-01-11 21:00
main.rs    2.5K  2025-01-11 21:00
Cargo.toml 1.2K  2025-01-11 20:00
```

#### Long format (`ls -l`)
```
Permissions  Owner  Size    Modified          Name
drwxr-xr-x   1000   -       2025-01-11 21:00  src/
-rw-r--r--   1000   2560    2025-01-11 21:00  main.rs
-rw-r--r--   1000   1234    2025-01-11 20:00  Cargo.toml
```

#### Human-readable (`ls -lh`)
```
Name       Size  Modified
main.rs    2.5K  2025-01-11 21:00
Cargo.toml 1.2K  2025-01-11 20:00
```

#### All files (`ls -a`)
```
Name       Size  Modified
.          -     -
..         -     -
.git/      -     2025-01-10 10:00
src/       -     2025-01-11 21:00
main.rs    2.5K  2025-01-11 21:00
```

#### Recursive (`ls -R`)
```
./:
src/  tests/  main.rs

./src:
cmd/  data/  lib.rs

./src/cmd:
commands/  fs.rs  parser.rs

./tests:
integration.rs
```

### Automated Tests
```bash
cargo test -p auto-shell ls
```

Run all ls-related unit tests.

## Risks & Mitigations

### R1: Platform Differences
**Risk**: Permissions and owner info differ significantly between Unix and Windows
**Mitigation**: Use `#[cfg(unix)]` and `#[cfg(windows)]` to provide reasonable defaults per platform

### R2: Recursive Performance
**Risk**: `-R` on large directory trees could be slow
**Mitigation**: Keep it simple for now, consider depth limits in future iterations

### R3: Sort Complexity
**Risk**: Combined flags like `-ltr` require careful ordering
**Mitigation**: Document sort precedence clearly in code comments

### R4: Metadata Storage
**Risk**: Storing full `fs::Metadata` in `FileEntry` increases memory
**Mitigation**: Only needed for `-l` flag, could be lazy-loaded if memory becomes issue

## Success Criteria

1. ✅ All 6 flags work independently (`-a`, `-l`, `-h`, `-t`, `-r`, `-R`)
2. ✅ Flags work in combination (`-alh`, `-ltr`, etc.)
3. ✅ Cross-platform (Windows, Linux, macOS)
4. ✅ Error handling maintained (nonexistent paths, permissions)
5. ✅ Table formatting works for both default and long formats
6. ✅ Comprehensive test coverage
7. ✅ Backwards compatible (default behavior unchanged)

## Timeline Estimate

- **Phase 1** (Command signature): 30 minutes
- **Phase 2** (fs.rs implementation): 2-3 hours
  - Simple flags (-a, -h, -t, -r): 1 hour
  - Long format (-l): 1 hour
  - Recursive (-R): 1 hour
- **Phase 3** (Tests): 1 hour
- **Phase 4** (Documentation): 30 minutes

**Total**: 4-5 hours

## Next Steps

1. ✅ User approved feature selection
2. ✅ Implementation plan complete
3. ✅ Phase 1: Update ls.rs command signature
4. ✅ Phase 2: Implement all 6 flags (-a, -l, -h, -t, -r, -R)
5. ✅ Phase 3: Add short flag aliases (a, l, h, t, r, R)
6. ✅ Phase 4: Implement combined short flags (e.g., -al, -ltr)
7. ✅ Manual verification completed

## Implementation Status: ✅ COMPLETE

All phases completed successfully:
- All 6 coreutils flags implemented and tested
- Short flag aliases working
- Combined short flags working (POSIX-style)
- Cross-platform support (Windows permissions simulated)
- Backwards compatible with default behavior
