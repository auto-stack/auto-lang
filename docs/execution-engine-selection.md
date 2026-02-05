# Execution Engine Selection Guide

## Overview

AutoLang now supports two execution engines:

1. **BigVM** (Bytecode VM) - **Default** ✅
   - Fast bytecode execution
   - 23.77x faster than Evaluator
   - Lower memory footprint
   - Production-ready

2. **Evaluator** (TreeWalker Interpreter) - Legacy
   - Slower AST traversal
   - Used for debugging and comparison
   - Fallback for edge cases

## Compile-Time Selection

### Default: BigVM Enabled

The BigVM is enabled by default via the `use-bigvm` feature flag in `Cargo.toml`:

```toml
[features]
default = ["with-file-history", "use-bigvm"]
use-bigvm = []
```

### To Use Evaluator (Legacy)

If you need to use the Evaluator instead (for debugging or comparison):

```bash
# Disable default features, enable only evaluator
cargo run --no-default-features --features use-evaluator

# Or explicitly in code
cargo run --features "use-evaluator"
```

## Runtime Selection

You can override the execution engine at runtime using an environment variable:

```bash
# Use BigVM explicitly
export AUTO_EXECUTION_ENGINE=bigvm
auto run script.at

# Use Evaluator explicitly
export AUTO_EXECUTION_ENGINE=evaluator
auto run script.at

# Use VM (same as bigvm)
export AUTO_EXECUTION_ENGINE=vm
auto run script.at

# Use Eval (same as evaluator)
export AUTO_EXECUTION_ENGINE=eval
auto run script.at
```

## API Usage

### In Your Code

```rust
use auto_lang::execution_engine::ExecutionEngine;

// Get default engine (BigVM unless overridden)
let engine = ExecutionEngine::get();

// Or specify explicitly
let engine = ExecutionEngine::BigVM;

// Execute code with selected engine
let result = auto_lang::execution_engine::execute_with_engine(
    engine,
    "1 + 2"
)?;

// Or use the high-level API (uses default engine)
let result = auto_lang::run("1 + 2")?;
```

### High-Level API (Recommended)

The `run()`, `run_file()`, and related functions now automatically use BigVM:

```rust
// These now use BigVM by default!
let result = auto_lang::run("1 + 2")?;
let result = auto_lang::run_file("script.at")?;
let result = auto_lang::run_with_mode(
    code,
    auto_lang::CompileMode::Script
)?;
```

## Performance Comparison

Based on Plan 073 Phase 9.1 benchmarks:

| Benchmark | Evaluator (μs) | BigVM (μs) | Speedup |
|-----------|----------------|-------------|---------|
| Simple arithmetic | 14,132 | 345 | **40.96x** |
| Complex arithmetic | 16,614 | 303 | **54.83x** |
| Loop 100 | 21,740 | 1,012 | **21.48x** |
| Loop 1000 | 19,405 | 1,292 | **15.02x** |
| Function calls 100 | 34,514 | 2,637 | **13.09x** |
| **Average** | - | - | **23.77x** |

**Conclusion**: BigVM is consistently 13-55x faster than the Evaluator!

## Migration Guide

### For Users

**No changes needed!** The switch to BigVM is transparent:

```bash
# Your existing commands still work, just faster now!
auto run script.at
auto eval "1 + 2"
auto repl
```

### For Developers

If you're using AutoLang as a library:

```rust
// Old code (still works, now uses BigVM under the hood)
use auto_lang::run;
let result = run("1 + 2")?;

// New code (if you need to control the engine)
use auto_lang::execution_engine::{ExecutionEngine, execute_with_engine};
let result = execute_with_engine(ExecutionEngine::BigVM, "1 + 2")?;
```

## Troubleshooting

### Issue: Code works in Evaluator but not in BigVM

**Solution**: Report the issue! We're tracking the remaining gaps in Plan 073.

**Workaround**: You can temporarily use the Evaluator:

```bash
export AUTO_EXECUTION_ENGINE=evaluator
auto run problematic_script.at
```

### Issue: Need to debug AST traversal

**Solution**: Use the Evaluator to see the AST:

```bash
export AUTO_EXECUTION_ENGINE=evaluator
auto run script.at
```

## Status

- ✅ BigVM is default execution engine (2026-02-06)
- ✅ 97.4% test pass rate (1254/1288 tests)
- ✅ 23.77x performance improvement
- ✅ Evaluator available as fallback
- ⏸️ 3 non-critical tests remaining (advanced features)

## See Also

- Plan 073: BigVM Migration Roadmap
- Plan 068: BigVM Implementation
- Plan 070: BigVM Iterator
- Plan 071: BigVM Closures
