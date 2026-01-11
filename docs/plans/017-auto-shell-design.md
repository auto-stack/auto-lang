# AutoShell Design Plan

**Status**: Draft
**Created**: 2025-01-11
**Priority**: High

## Objective

Design and implement a cross-platform shell environment (`auto-shell`) that uses AutoLang as its scripting language. Inspired by modern shells like [nu-shell](https://www.nushell.sh/), AutoShell will provide a powerful, composable command-line interface with structured data support.

## Inspiration from nu-shell

**What makes nu-shell special**:
- **Structured data**: Commands pass structured data (tables, records) not just text
- **Powerful pipelines**: Compose commands with clean pipe syntax
- **Modern UX**: Auto-completion, syntax highlighting, friendly error messages
- **Cross-platform**: Works on Linux, macOS, Windows
- **Type system**: Built-in types for numbers, strings, booleans, dates, files, etc.

**AutoShell's unique advantages**:
- AutoLang scripting (simpler than nu's Rust-based custom language)
- Direct integration with AutoLang ecosystem
- Can leverage AutoLang's C transpilation for embedded use
- Existing standard library (`auto.io`, `auto.sys`, etc.)

## Project Vision

```
A modern shell that speaks AutoLang
+ Structured data pipelines like nu-shell
+ AutoLang's simplicity and C transpilation
= AutoShell: Embedded-friendly, cross-platform shell
```

## Core Architecture

### Directory Structure

```
auto-shell/
├── Cargo.toml                    # Rust implementation (primary)
├── src/
│   ├── main.rs                   # REPL entry point
│   ├── repl.rs                   # Read-Eval-Print Loop
│   ├── parser/
│   │   ├── mod.rs                # Command parser
│   │   ├── pipeline.rs           # Pipeline parsing (|)
│   │   └── redirect.rs           # I/O redirection (>, >>, <)
│   ├── cmd/
│   │   ├── mod.rs                # Command registry
│   │   ├── builtin.rs            # Built-in commands (cd, exit, etc.)
│   │   ├── external.rs           # External process execution
│   │   └── auto.rs               # AutoLang integration
│   ├── shell/
│   │   ├── mod.rs                # Shell state & environment
│   │   ├── vars.rs               # Shell variables ($PATH, etc.)
│   │   ├── dirs.rs               # Directory stack management
│   │   └── jobs.rs               # Background job control
│   ├── data/
│   │   ├── mod.rs                # Structured data types
│   │   ├── value.rs              # Shell value (table, record, primitive)
│   │   └── convert.rs            # Auto Value ↔ Shell Value conversion
│   ├── completions/
│   │   ├── mod.rs                # Auto-completion engine
│   │   ├── command.rs            # Command name completion
│   │   ├── file.rs               # File path completion
│   │   └── auto.rs               # AutoLang variable completion
│   ├── term/
│   │   ├── mod.rs                # Terminal interface
│   │   ├── prompt.rs             # Prompt rendering
│   │   └── highlight.rs          # Syntax highlighting
│   └── platform/
│       ├── mod.rs                # Platform abstraction
│       ├── unix.rs               # Unix-specific (Linux, macOS)
│       └── windows.rs            # Windows-specific
└── stdlib/
    ├── shell.at                  # AutoLang shell stdlib
    ├── prompt.at                 # Prompt customization
    └── aliases.at                # Alias management

auto-shell/autoc/                 # C implementation (portable)
├── CMakeLists.txt
├── shell.c                       # Core shell logic
├── repl.c                        # REPL
└── ...

auto-shell/tests/
├── integration/
│   ├── pipelines.test            # Pipeline tests
│   ├── builtins.test             # Built-in command tests
│   └── scripts.at                # AutoLang shell script tests
└── fixtures/
    └── ...
```

## Language Design

### Command Syntax

AutoShell commands will look like traditional shell commands but with AutoLang expressions:

```bash
# Traditional shell
ls -la | grep ".at" | wc -l

# AutoShell (similar syntax)
ls -la | grep ".at" | count

# But with AutoLang for complex logic
ls | where { |file| file.size > 1024 } | select name, size

# Variables and interpolation
let files = ls
let large = $files | where { |f| f.size > 1024 }
echo f"Found {len($large)} large files"
```

### Pipeline Model

**nu-shell approach**: Commands are functions that take structured input and produce structured output.

```
input → command1 → data1 → command2 → data2 → command3 → output
```

**AutoShell approach**: Use AutoLang's Value system directly:

```rust
// External command: ls returns Value::Array of objects
let files = ls()

// Pipe operator passes Value to next command
let large = $files | grep("pattern")  // grep takes Value::Array, returns filtered array

// Auto functions can be used in pipelines
let result = $files | filter(|f| f.size > 1024) | map(|f| f.name)
```

### Command Types

1. **Built-in commands** (compiled into shell):
   - `cd`, `pwd`, `exit`, `export`, `alias`
   - `where`, `select`, `group`, `sort`
   - `count`, `first`, `last`, `take`

2. **External commands** (system executables):
   - Execute via platform APIs (CreateProcess, execve)
   - Wrap stdin/stdout/stderr as Values

3. **Auto functions** (AutoLang code):
   - Define functions in `.at` files
   - Use in pipelines like built-in commands

## Structured Data System

### Shell Value Types

Map AutoLang `Value` types to shell-specific types:

```rust
enum ShellValue {
    // Primitive types (map to Auto Value)
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),

    // Shell-specific structured types
    Array(Vec<ShellValue>),           // Ordered list
    Object(IndexMap<String, ShellValue>),  // Key-value pairs
    Table {
        columns: Vec<String>,
        rows: Vec<Vec<ShellValue>>,
    },

    // System types
    Path(PathBuf),                    // File path
    Command(String),                  // Command name
    Duration(Duration),               // Time duration
    DateTime(chrono::DateTime<...>),  // Timestamp
}

impl From<Value> for ShellValue { /* ... */ }
impl From<ShellValue> for Value { /* ... */ }
```

### Table Display

Tables should render beautifully in the terminal:

```
> ls | where name =~ "\.at$" | select name, size

───┬─────────────┬──────
 # │ name        │ size
───┼─────────────┼──────
 0 │ io.at       │ 1024
 1 │ sys.at      │  512
 2 │ math.at     │ 2048
───┴─────────────┴──────
```

## Key Features

### 1. Pipeline System

```bash
# Basic pipeline
ls | grep "test" | count

# Pipeline with Auto lambda
ls | filter { |file| file.size > 1024 } | map { |file| file.name }

# Pipeline to external command (text mode)
ps | where name == "auto" | awk '{print $2}'
```

### 2. AutoLang Integration

```bash
# Define Auto function in shell
fn sum_large(dir string) int {
    let files = ls($dir)
    let large = $files | filter { |f| f.size > 1024 }
    return $large | reduce { |acc, f| acc + f.size } 0
}

# Use it
sum_large("./src")  # Returns total size of large files
```

### 3. Auto-completion

```rust
// Command completion
git <TAB>  # Shows: add, commit, push, pull, status...

// File completion
ls ./src/<TAB>  # Shows files in src/

// Variable completion
echo $fil<TAB>  # Shows: $files, $file_count...

// Flag completion
ls --<TAB>  # Shows: --all, --long, --human-readable...
```

### 4. History & Scripts

```bash
# History (up-arrow, Ctrl+R)
history | grep "cargo build"

# Save command to history
# (automatic)

# Script files (.at files)
# script.at
#!/usr/bin/auto-shell
use auto.io: say

let files = ls()
for file in $files {
    say($file.name)
}

# Run script
auto-shell script.at
# or make executable
chmod +x script.at
./script.at
```

### 5. Configuration

```auto
# ~/.config/auto-shell/config.at
# AutoLang configuration file

# Prompt customization
fn prompt() string {
    let dir = pwd()
    let git = git_branch()
    if $git != "" {
        return f"[$dir] ($git) > "
    } else {
        return f"[$dir] > "
    }
}

# Aliases
let aliases = {
    "ll": "ls --long",
    "la": "ls --all",
    "gs": "git status",
}

# Environment
export PATH = $PATH + ":/usr/local/bin"
export EDITOR = "vim"
```

## Platform Support

### Unix (Linux, macOS)

- Use `libc` for process execution
- `posix_spawn` or `execve`
- Signal handling (SIGINT, SIGTSTP)
- Terminal control with termios/ncurses

### Windows

- Use `CreateProcess` for process execution
- Windows console API or `windows-rs`
- ANSI escape sequence support (Windows 10+)
- Path handling (`\` vs `/`)

## Implementation Phases

### Phase 1: Core REPL (Week 1-2)

**Goals**:
- Basic REPL loop (read, parse, eval, print)
- Simple command execution (external processes)
- AutoLang expression evaluation

**Deliverables**:
- [ ] REPL that accepts commands and Auto code
- [ ] Execute external commands (ls, echo, etc.)
- [ ] Evaluate Auto expressions (1 + 2, f"hello")
- [ ] Basic error handling

**Files**:
- `src/main.rs`: REPL entry point
- `src/repl.rs`: Read-Eval-Print Loop
- `src/cmd/external.rs`: External command execution
- `src/parser/mod.rs`: Command parser

### Phase 2: Pipeline System (Week 3)

**Goals**:
- Parse pipeline syntax (`|`)
- Pass data between commands
- Auto Value ↔ Shell Value conversion

**Deliverables**:
- [ ] Pipeline parser
- [ ] Command chaining with `|` operator
- [ ] Value type system (primitives, arrays, objects)
- [ ] Basic built-in commands (`count`, `first`, `last`)

**Files**:
- `src/parser/pipeline.rs`: Pipeline parsing
- `src/data/value.rs`: Shell value types
- `src/data/convert.rs`: Auto ↔ Shell conversion
- `src/cmd/builtin.rs`: Built-in commands

### Phase 3: Built-in Commands (Week 4-5)

**Goals**:
- Core shell built-ins (cd, pwd, exit, export)
- Data manipulation commands (where, select, sort, group)
- File system commands (ls, mv, cp, rm, mkdir)

**Deliverables**:
- [ ] `cd`, `pwd`, `exit`, `export`
- [ ] `where`, `select`, `group`, `sort`
- [ ] `ls`, `mv`, `cp`, `rm`, `mkdir`
- [ ] Command registry system

**Files**:
- `src/cmd/builtin.rs`: All built-in commands
- `src/cmd/registry.rs`: Command registration and lookup
- `src/shell/dirs.rs`: Directory stack (pushd, popd)
- `src/shell/vars.rs`: Shell variables

### Phase 4: AutoLang Integration (Week 6)

**Goals**:
- Use `auto-lang` crate for evaluation
- Define Auto functions in shell
- Import stdlib in shell context

**Deliverables**:
- [ ] AutoLang expression evaluation
- [ ] `fn` definition in shell
- [ ] Import/use statements
- [ ] Access shell variables from Auto

**Files**:
- `src/cmd/auto.rs`: AutoLang integration
- `src/shell/context.rs`: Shared context between shell and Auto
- `stdlib/shell.at`: Shell stdlib

### Phase 5: Table Display & Formatting (Week 7)

**Goals**:
- Beautiful table rendering
- Column alignment and wrapping
- Color output

**Deliverables**:
- [ ] Table data structure
- [ ] Table renderer
- [ ] Auto-formatting based on terminal width
- [ ] Color syntax highlighting

**Files**:
- `src/data/table.rs`: Table type and rendering
- `src/term/highlight.rs`: Syntax highlighting
- `src/term/width.rs`: Terminal width detection

### Phase 6: Auto-completion (Week 8)

**Goals**:
- Command name completion
- File path completion
- Variable completion
- Flag completion

**Deliverables**:
- [ ] Command name completer
- [ ] File path completer
- [ ] Auto variable completer
- [ ] Integration with readline library (rustyline or reedline)

**Files**:
- `src/completions/mod.rs`: Completion engine
- `src/completions/command.rs`: Command completion
- `src/completions/file.rs`: File completion
- `src/completions/auto.rs`: Auto variable completion

### Phase 7: Configuration & Prompt (Week 9)

**Goals**:
- Load config from `~/.config/auto-shell/`
- Customizable prompt (Auto function)
- Alias system
- Environment variables

**Deliverables**:
- [ ] Config file loading
- [ ] Auto function for prompt
- [ ] Alias management
- [ ] Environment variable handling

**Files**:
- `src/shell/config.rs`: Config loading
- `src/term/prompt.rs`: Prompt rendering
- `stdlib/prompt.at`: Prompt customization
- `stdlib/aliases.at`: Alias management

### Phase 8: Platform Support (Week 10)

**Goals**:
- Unix platform support (Linux, macOS)
- Windows platform support
- Platform abstraction layer

**Deliverables**:
- [ ] Unix signal handling
- [ ] Windows console API
- [ ] Cross-platform path handling
- [ ] Platform-specific tests

**Files**:
- `src/platform/mod.rs`: Platform abstraction
- `src/platform/unix.rs`: Unix implementation
- `src/platform/windows.rs`: Windows implementation

### Phase 9: History & Scripting (Week 11)

**Goals**:
- Command history (up-arrow, Ctrl+R)
- History file persistence
- Script file execution (.at files)
- Shebang support

**Deliverables**:
- [ ] In-memory command history
- [ ] History file (`.auto-shell-history`)
- [ ] Execute .at files as scripts
- [ ] Shebang line parsing

**Files**:
- `src/shell/history.rs`: Command history
- `src/repl.rs`: Script execution mode
- Tests for script execution

### Phase 10: Polish & Documentation (Week 12)

**Goals**:
- Comprehensive tests
- Documentation website
- Example scripts
- Performance optimization

**Deliverables**:
- [ ] Test suite (>80% coverage)
- [ ] User guide
- [ ] API documentation
- [ ] Example scripts

**Files**:
- `docs/guide.md`: User guide
- `examples/`: Example scripts
- `README.md`: Project overview
- Test suite

## Dependencies

### Rust Crates

```toml
[dependencies]
# AutoLang integration
auto-lang = { path = "../crates/auto-lang" }
auto-val = { path = "../crates/auto-val" }

# Terminal and REPL
reedline = "0.33"           # Readline library (used by nu-shell)
crossterm = "0.27"          # Cross-platform terminal
nu-ansi-term = "0.49"       # ANSI colors

# Data structures
indexmap = "2.0"            # Ordered maps (consistent with Auto)

# Platform support
dirs = "5.0"                # Config directory resolution
sysinfo = "0.30"            # System information

# Parsing
nom = "7.1"                 # Parser combinator (optional)

# Utilities
chrono = "0.4"              # Date/time
regex = "1.10"              # Regular expressions
unicode-segmentation = "1.10"  # Unicode handling
```

### C Dependencies (for autoc/ version)

- POSIX: `libc`, `libreadline`
- Windows: `windows.h`, `readline` port

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_execution() {
        let mut shell = Shell::new();
        let result = shell.execute("ls | count");
        assert!(result.is_ok());
    }

    #[test]
    fn test_builtin_commands() {
        let mut shell = Shell::new();
        shell.execute("cd /tmp");
        assert_eq!(shell.pwd(), PathBuf::from("/tmp"));
    }

    #[test]
    fn test_auto_integration() {
        let mut shell = Shell::new();
        let result = shell.execute("1 + 2");
        assert_eq!(result, Ok(ShellValue::Int(3)));
    }
}
```

### Integration Tests

```bash
# Test script execution
auto-shell tests/integration/pipelines.test

# Test REPL interactions
# (using expect or similar)

# Benchmark pipelines
./benchmarks/pipeline_bench.sh
```

### Test Fixtures

```
tests/fixtures/
├── sample_files/           # Test file structure
├── scripts/
│   ├── simple.at          # Simple script
│   ├── pipeline.at        # Pipeline test
│   └── error.at           # Error handling test
└── outputs/
    ├── ls.expected        # Expected ls output
    └── ...
```

## Success Criteria

**Must Have** (MVP):
- ✅ REPL that evaluates Auto code and executes commands
- ✅ Pipeline operator (`|`) for chaining commands
- ✅ Basic built-ins (cd, pwd, ls, exit)
- ✅ Table display for structured data
- ✅ Cross-platform (Linux, macOS, Windows)

**Should Have** (v0.2):
- ✅ Auto-completion for commands, files, variables
- ✅ History (up-arrow, Ctrl+R)
- ✅ Configuration via Auto files
- ✅ Customizable prompt
- ✅ Script execution (.at files)

**Nice to Have** (v0.3+):
- 🔄 Background jobs (&, jobs, fg, bg)
- 🔄 Shell variables and environment
- 🔄 Alias system
- 🔄 Syntax highlighting in prompt
- 🔄 Debugger for Auto scripts

## Open Questions

1. **REPL Library**: Use `reedline` (nu-shell's choice) or `rustyline`?
   - **Recommendation**: Start with `reedline` for modern features

2. **Command Parsing**: Custom parser or leverage AutoLang parser?
   - **Recommendation**: Custom parser for shell-specific syntax (pipelines, redirects)

3. **Data Model**: Extend Auto Value or separate ShellValue?
   - **Recommendation**: Start with separate ShellValue, unify later if needed

4. **C Implementation**: Port from Rust or implement independently?
   - **Recommendation**: Implement Rust version first, port to C later

5. **Shell Standard**: POSIX compliance or break compatibility?
   - **Recommendation**: POSIX-adjacent (familiar syntax, modern semantics)

## Related Projects

- [nu-shell](https://github.com/nushell/nushell): Primary inspiration
- [fish-shell](https://fishshell.com/): UX and completion inspiration
- [PowerShell](https://docs.microsoft.com/powershell/): Structured data approach
- [ion-shell](https://github.com/redox-os/ion): Rust-based shell

## Next Steps

1. **Review this plan** with stakeholders
2. **Create GitHub repo** for auto-shell
3. **Set up project structure** (Cargo.toml, src/)
4. **Implement Phase 1** (Core REPL)
5. **Write first tests** (smoke test for REPL)

## References

- [nu-shell Architecture](https://www.nushell.sh/book/book/creating_a_custom_command.html)
- [POSIX Shell Standard](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html)
- [AutoLang Documentation](../README.md)
- [reedline Documentation](https://docs.rs/reedline/)
