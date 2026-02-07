# Migration Guide: Feature Flags to Mode Selection

This guide helps you migrate from the old feature flag system to the new mode selection approach.

## What Changed?

### Old System (Deprecated)

**Feature Flags** (`Cargo.toml`):
```toml
[features]
default = []
use-bigvm = []  # Required to enable AutoVM
```

**Build Commands**:
```bash
# With AutoVM (requires feature flag)
cargo build --features use-bigvm
cargo run --features use-bigvm

# Without AutoVM (default, uses Evaluator)
cargo build
cargo run
```

### New System (Current)

**Mode Selection** (`pac.at`):
```auto
mode: "autovm"  # AutoVM is now the DEFAULT
```

**Build Commands**:
```bash
# AutoVM is default (no feature flag needed)
cargo build
cargo run

# Override at runtime with environment variable
export AUTO_EXECUTION_ENGINE=evaluator
cargo run
```

## Migration Steps

### Step 1: Remove Feature Flags

**Before**:
```toml
# Cargo.toml
[features]
default = []
use-bigvm = []
```

**After**:
```toml
# Cargo.toml
[features]
default = []

# The use-bigvm feature is kept for compatibility but is a no-op
# You can remove it entirely or keep it for backward compatibility
use-bigvm = []  # Deprecated: has no effect
```

### Step 2: Update Build Scripts

**Before**:
```bash
#!/bin/bash
# build.sh

# Build with AutoVM
cargo build --release --features use-bigvm
```

**After**:
```bash
#!/bin/bash
# build.sh

# Build with AutoVM (now default)
cargo build --release

# Or explicitly specify mode in pac.at instead
```

### Step 3: Update CI/CD Pipelines

**GitHub Actions - Before**:
```yaml
build:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v2
    - name: Build with AutoVM
      run: cargo build --features use-bigvm --release
```

**GitHub Actions - After**:
```yaml
build:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v2
    - name: Build (AutoVM is now default)
      run: cargo build --release

    # Or specify mode in pac.at:
    # name: Build with C transpilation
    # run: auto trans_c src/main.at
```

### Step 4: Update Documentation

**Before**:
```markdown
# Building

To build with AutoVM:

\`\`\`bash
cargo build --features use-bigvm
\`\`\`
```

**After**:
```markdown
# Building

AutoVM is now the default execution engine:

\`\`\`bash
cargo build
\`\`\`

To override execution mode, use environment variable:

\`\`\`bash
export AUTO_EXECUTION_ENGINE=evaluator
cargo build
\`\`\`

Or specify mode in `pac.at`:

\`\`\`auto
mode: "c"  # Transpile to C
\`\`\`
```

## Project-by-Project Migration

### Pure AutoVM Projects (Simplest)

**Before**:
```bash
# Always built with feature flag
cargo run --features use-bigvm
```

**After**:
```bash
# AutoVM is default
cargo run

# Optional: Add pac.at for clarity
echo 'mode: "autovm"' > pac.at
```

### C Transpilation Projects

**Before**:
```bash
# Used separate a2c test framework
cargo test -p auto-lang -- trans
```

**After**:
```auto
// pac.at
name: "embedded_app"
version: "1.0.0"
mode: "c"  # Transpile to C

app("embedded_app") {
    // All code transpiled to C
}
```

```bash
# Now part of standard build process
auto build  # Generates .c/.h files
gcc -o firmware main.c
```

### Mixed-Mode Projects

**Before**: Not possible (all code used same mode)

**After**:
```auto
// pac.at
name: "mixed_app"
version: "1.0.0"
mode: "autovm"

app("mixed_app") {
    dependencies: [
        ("hal", mode: "c"),       # Hardware in C
        ("crypto", mode: "rust"), # Crypto in Rust
        "std:core",               # Stdlib in AutoVM
    ]
}
```

## Breaking Changes

### 1. Feature Flag Removed

**Impact**: `--features use-bigvm` is now a no-op

**Migration**: Remove the flag from all build scripts

### 2. Default Execution Mode Changed

**Impact**: Projects that relied on Evaluator by default now use AutoVM

**Migration**:
- **Option A**: Explicitly set mode in pac.at:
  ```auto
  mode: "evaluator"  # Use old Evaluator
  ```
- **Option B**: Use environment variable:
  ```bash
  export AUTO_EXECUTION_ENGINE=evaluator
  ```
- **Option C** (Recommended): Migrate to AutoVM

### 3. Runtime Engine Selection

**Before**: Compile-time selection via feature flags

**After**: Runtime selection via environment variable

**Migration**:
```bash
# Old way (compile-time)
cargo build --features use-bigvm

# New way (runtime)
export AUTO_EXECUTION_ENGINE=autovm
cargo build
```

## Compatibility Matrix

| Old Command | New Equivalent | Notes |
|-------------|----------------|-------|
| `cargo build` | `cargo build` | Now uses AutoVM by default |
| `cargo build --features use-bigvm` | `cargo build` | Feature flag is no-op |
| `cargo test -p auto-lang -- trans` | `auto trans_c file.at` | Now part of main CLI |
| N/A | `export AUTO_EXECUTION_ENGINE=evaluator` | New runtime override |

## Rollback Plan

If you encounter issues after migration:

### Temporary Rollback

You can temporarily use the old behavior:

```bash
# Force Evaluator mode
export AUTO_EXECUTION_ENGINE=evaluator
cargo run
```

### Permanent Rollback

Add to your `pac.at`:

```auto
mode: "evaluator"
```

**Note**: Evaluator mode is deprecated. Use this only for migration purposes.

## Verification

After migration, verify your build works:

```bash
# 1. Clean build
cargo clean

# 2. Build without feature flags
cargo build

# 3. Run tests
cargo test

# 4. Run application
cargo run

# 5. Verify execution mode
cargo run -- --version
```

## Common Issues and Solutions

### Issue: Build Fails After Migration

**Symptom**: Compilation errors after removing `--features use-bigvm`

**Cause**: Code relied on AutoVM-specific features

**Solution**:
1. Check that `pac.at` has correct mode specified
2. Verify all dependencies support selected mode
3. Check for missing FFI declarations

### Issue: Performance Changed

**Symptom**: Application slower/faster after migration

**Cause**: Different execution mode being used

**Solution**:
1. Verify which mode is active: Check build output
2. Explicitly set desired mode in `pac.at`
3. Use environment variable override for testing

### Issue: Tests Failing

**Symptom**: Tests pass with old system, fail with new

**Cause**: Tests relied on specific execution mode

**Solution**:
```rust
// In tests, explicitly set mode
#[test]
fn test_my_feature() {
    // Force AutoVM mode for this test
    std::env::set_var("AUTO_EXECUTION_ENGINE", "autovm");

    // Test code here
}
```

## Best Practices After Migration

### 1. Always Specify Mode in pac.at

```auto
// Recommended
mode: "autovm"  // Explicit is better than implicit
```

### 2. Use Environment Variables for CI

```yaml
# .github/workflows/test.yml
env:
  AUTO_EXECUTION_ENGINE: autovm  # Explicit for CI

steps:
  - run: cargo test
```

### 3. Document Mode Requirements

```auto
// pac.at
// This project requires AutoVM for performance
mode: "autovm"
```

### 4. Test Multiple Modes

```bash
# Test with AutoVM
cargo test

# Test with Evaluator (for compatibility)
export AUTO_EXECUTION_ENGINE=evaluator
cargo test
```

## Timeline

- **✅ Phase 1** (Complete): AutoVM made default, feature flags deprecated
- **✅ Phase 2-5** (Complete): Mode selection infrastructure implemented
- **🔄 Current**: Migration and documentation
- **🔮 Future**: Feature flags may be removed entirely (backward compatibility period)

## Get Help

If you encounter issues during migration:

1. **Check the guides**:
   - [Mode Selection Guide](mode-selection-guide.md)
   - [FFI Usage Guide](ffi-usage-guide.md)

2. **Verify your setup**:
   ```bash
   # Check which engine is being used
   auto --version

   # Check pac.at syntax
   auto validate pac.at
   ```

3. **Report issues**:
   - GitHub: https://github.com/your-repo/issues
   - Include: `pac.at`, error messages, build output

## Summary

| Aspect | Old System | New System |
|--------|-----------|-----------|
| **Configuration** | Feature flags in Cargo.toml | Mode in pac.at |
| **Default** | Evaluator | AutoVM |
| **Override** | Compile-time (feature flags) | Runtime (env var) |
| **Mixed Modes** | Not supported | Fully supported |
| **C Transpilation** | Separate test framework | Integrated in pac.at |

## Next Steps

1. ✅ Read this guide
2. ✅ Update your `pac.at` with desired mode
3. ✅ Remove `--features use-bigvm` from build scripts
4. ✅ Test your application
5. ✅ Update documentation

**Welcome to the new AutoVM-first world! 🚀**

---

**See Also**:
- [Plan 081: AutoVM as Default](../plans/081-autovm-default-mode.md)
- [Mode Selection Guide](mode-selection-guide.md)
- [Phase 2 Completion](../plans/081-phase2-complete.md)
