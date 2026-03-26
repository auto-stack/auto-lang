# Plan 150: AI Mode (--ai flag)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add `--ai` flag to `auto.exe` for AI-friendly JSON output with suppressed human-readable info.

**Architecture:** Add a global `--ai` boolean flag that sets `ai_mode` variable. When enabled: (1) suppress logo/info output, (2) format errors as JSON, (3) wrap success results in JSON structure.

**Tech Stack:** Rust, clap (CLI parsing), serde_json (JSON output), miette (diagnostics)

---

## Status: ✅ COMPLETED (2026-03-26)

---

## Design

### Core Principles

1. **Human info → Silent**: Logo, progress messages, and other auxiliary info should not be output
2. **AI info → JSON structured**: Errors and results should be in JSON format
3. **Non-AI mode unchanged**: Existing behavior must remain exactly the same when `--ai` is not specified

### Behavior Comparison

| Output Type | Normal Mode | AI Mode (`--ai`) |
|-------------|-------------|------------------|
| Logo (`println_logo`) | ✅ Shown | ❌ Silent |
| Info logs (`info!`) | ✅ Shown | ❌ Silent |
| Success result | Plain text | `{"status": "success", "result": ...}` |
| Error | Miette formatted | `{"message":...,"code":...,"severity":"error"}` |

### CLI Flag

```rust
/// AI-friendly output mode: JSON structured output, suppress human-readable info
/// Equivalent to --format json with additional output suppression
#[arg(long = "ai", global = true)]
ai: bool,
```

### JSON Formats

#### Error Output (reuse existing `format_error_json()`)

```json
{
  "message": "unexpected token",
  "code": "auto_syntax_E0001",
  "severity": "error",
  "spans": [{"offset": 10, "len": 5, "label": "here"}],
  "help": "Expected 'identifier', but found '+'"
}
```

#### Success Output (pure commands)

```json
{
  "status": "success",
  "result": "42"
}
```

#### Success Output (file operation commands)

```json
{
  "status": "success",
  "result": {
    "message": "Project created",
    "files_created": ["myapp/am.at"],
    "dirs_created": ["myapp/src"]
  }
}
```

---

## Implementation

### File Modified
- `crates/auto/src/main.rs`

### Helper Functions Added

```rust
/// Format success result as JSON for AI consumption
fn format_success_json<T: serde::Serialize>(result: T) -> String {
    json!({
        "status": "success",
        "result": result
    }).to_string()
}

/// Output success result in appropriate format based on AI mode
fn output_success(ai_mode: bool, result: &str) {
    if ai_mode {
        println!("{}", format_success_json(result));
    } else {
        println!("{}", result);
    }
}
```

### Commands Updated

All CLI commands now support AI mode:
- `new`, `init` - Project creation
- `build`, `run`, `clean` - Build lifecycle
- `fetch`, `deps` - Dependencies
- `device`, `export` - Hardware/Export
- `info`, `open` - Project utils
- `upgrade`, `env` - Environment
- `gen` - Code generation
- `parse`, `eval`, `config` - Direct execution
- `c`, `rust`, `python`, `javascript` - Transpilation

### Pattern Applied

```rust
Some(Commands::Build { dir, port }) => {
    if !ai_mode {
        init_logger();
        println_logo();
    }
    // ... execute command ...
    // On error:
    if ai_mode {
        eprintln!("{}", format_error_json(&AutoError::Msg(e.to_string())));
        std::process::exit(1);
    }
    // On success:
    if ai_mode {
        println!("{}", format_success_json(json!({"message": "Build completed"})));
    }
}
```

---

## Testing

### Manual Tests

```bash
# Normal mode (unchanged behavior)
auto build
auto run hello.at

# AI mode - error output
echo 'fn main() { let x = 1 + }' > tmp/test_error.at
auto --ai tmp/test_error.at 2>&1
# Expected: {"message":"...","code":"...","severity":"error",...}

# AI mode - logo suppressed
auto --ai --help | grep -c "AutoNexus"
# Expected: 0 (logo suppressed)

# AI mode - flag recognized
auto --help | grep -A2 "ai"
# Expected: shows --ai flag description
```

### Test Script

```bash
#!/bin/bash
set -e

echo "=== Test 1: --ai flag recognized ==="
./target/debug/auto --help | grep -q "ai" && echo "PASS" || echo "FAIL"

echo "=== Test 2: Error JSON format ==="
echo 'fn main() { let x = 1 + }' > tmp/test_error.at
OUTPUT=$(./target/debug/auto --ai tmp/test_error.at 2>&1 || true)
echo "$OUTPUT" | grep -q '"severity":"error"' && echo "PASS" || echo "FAIL"

echo "=== Test 3: Logo suppressed in AI mode ==="
OUTPUT=$(./target/debug/auto --ai --help 2>&1)
echo "$OUTPUT" | grep -q "AutoNexus" && echo "FAIL (logo shown)" || echo "PASS (logo suppressed)"

echo "=== All tests completed ==="
```

---

## Success Criteria

1. ✅ `--ai` flag is recognized globally
2. ✅ AI mode outputs JSON for errors
3. ✅ AI mode outputs JSON for success results
4. ✅ AI mode suppresses logo and info logs
5. ✅ Non-AI mode behavior is completely unchanged
6. ✅ No breaking changes to existing CLI usage

---

## Future Enhancements (Phase 2)

- [ ] Track file system changes for detailed `result` field
- [ ] Refactor all commands to use unified output system
- [ ] Add more structured output for commands like `deps`, `info`
- [ ] Progress JSON for long-running operations
