# AutoShell

A modern cross-platform shell environment that uses AutoLang as its scripting language. Inspired by [nu-shell](https://www.nushell.sh/), AutoShell provides structured data pipelines with the simplicity of AutoLang.

## Features

- **REPL**: Interactive read-eval-print loop
- **AutoLang Integration**: Execute Auto code directly in the shell
- **Pipeline System**: Chain commands with `|` operator
- **File System Commands**: ls, cd, mkdir, rm, mv, cp (coreutils-style)
- **Data Manipulation**: sort, uniq, head, tail, wc, grep
- **External Commands**: Run any command in your PATH
- **Cross-Platform**: Works on Linux, macOS, and Windows

## Installation

```bash
# From the auto-lang repository
cd auto-shell
cargo build --release
```

## Usage

Start the REPL:

```bash
cargo run
```

### Examples

```bash
# File system commands
ls                  # List current directory
ls /path           # List specific directory
cd /path           # Change directory
mkdir new_dir      # Create directory
mkdir -p a/b/c     # Create nested directories
rm file.txt       # Remove file
rm -r directory   # Remove directory (recursive)
mv old new        # Move/rename
cp src dst        # Copy file
cp -r src dst     # Copy directory

# Data manipulation
echo -e "c\na\nb" | sort           # Sort lines: a, b, c
echo -e "a\na\nb" | uniq           # Remove duplicates: a, b
echo -e "a\nb\nc" | head -n 2      # First 2 lines: a, b
echo -e "a\nb\nc" | tail -n 2      # Last 2 lines: b, c
echo "hello world test" | wc       # Count: 1 line, 3 words
echo -e "hello\nworld" | grep hello   # Match lines

# AutoLang expressions
1 + 2
let x = 42
x * 2

# Chain commands with pipelines
ls | sort
echo "hello\nworld" | grep hello | sort

# Get help
help

# Clear screen
clear

# Exit the shell
exit
```

## Development Status

**Phase 1 ✅**: Core REPL
- ✅ Basic REPL loop
- ✅ External command execution
- ✅ AutoLang expression evaluation
- ✅ Built-in commands (pwd, echo, help, exit)

**Phase 2 ✅**: Pipeline System
- ✅ Pipeline parser (handles `|` operator)
- ✅ Pipeline execution with command chaining
- ✅ Quote-aware parsing (single, double quotes)
- ✅ Parenthesis support in pipelines

**Phase 3 ✅**: Built-in Commands
- ✅ File system: ls, cd, mkdir, rm, mv, cp
- ✅ Data manipulation: sort, uniq, head, tail, wc, grep
- ✅ Flag parsing (-r, -n, -p, -c, etc.)
- ✅ Cross-platform path handling
- ✅ 64 passing tests

See [docs/plans/017-auto-shell-design.md](../docs/plans/017-auto-shell-design.md) for the full implementation plan.

## Testing

```bash
cargo test
```

## License

MIT
