# AutoCache User Guide

**Version**: 1.0
**Plan**: 082
**Status**: Production Ready

## Table of Contents

1. [Introduction](#introduction)
2. [Quick Start](#quick-start)
3. [How AutoCache Works](#how-autocache-works)
4. [CLI Commands](#cli-commands)
5. [Cache Management](#cache-management)
6. [Troubleshooting](#troubleshooting)
7. [Best Practices](#best-practices)
8. [Performance Tips](#performance-tips)

---

## Introduction

**AutoCache** is a global build cache for AutoLang projects that dramatically reduces build times by reusing compiled artifacts across projects, sessions, and versions.

### Key Benefits

- **Cross-Project Sharing**: Same module compiled once, used everywhere
- **Automatic Caching**: No configuration needed for basic usage
- **Zero-Copy Hits**: Hard links for instant cache retrieval
- **Intelligent GC**: Automatic cleanup when cache grows too large
- **Integrity Verification**: Detect and repair cache corruption

### What Gets Cached?

AutoCache stores compiled artifacts from:
- **a2c transpilation**: `.c` and `.h` files
- **a2r transpilation**: `.rs` files
- **AutoVM**: Bytecode (`.bc`) files
- **C compilation**: Compiled object files (`.o`, `.obj`)

### Requirements

- AutoLang compiler with AutoMan (Plan 082 integration)
- ~100MB minimum disk space for cache
- Home directory with write permissions

---

## Quick Start

### Enabling AutoCache

AutoCache is **disabled by default**. Enable it by setting an environment variable:

**Windows (Command Prompt)**:
```cmd
set AUTO_CACHE_ENABLED=true
auto build
```

**Windows (PowerShell)**:
```powershell
$env:AUTO_CACHE_ENABLED="true"
auto build
```

**Linux/macOS**:
```bash
export AUTO_CACHE_ENABLED=true
auto build
```

### First Build

The first time you build with AutoCache enabled:

```bash
$ export AUTO_CACHE_ENABLED=true
$ auto build

[Cache Miss] std:io (C)
[Cache Miss] std:fs (C)
[Cache Miss] myapp:main (C)
# ... transpilation happens ...

[Cache Store] std:io (C, 12345 bytes)
[Cache Store] std:fs (C, 8765 bytes)
[Cache Store] myapp:main (C, 15234 bytes)

Build completed in 15.3s
```

### Subsequent Builds

On the next build (or in a different project using the same modules):

```bash
$ auto build

[Cache Hit] std:io (C)
[Cache Hit] std:fs (C)
[Cache Miss] myapp:main (C)  # You modified this file

Build completed in 2.1s  # Much faster!
```

### Typical Performance Gains

- **Clean build**: No speedup (cache miss)
- **Incremental build**: 50-80% faster
- **Cross-project reuse**: Up to 90% faster

---

## How AutoCache Works

### Cache Storage Location

AutoCache stores data in your home directory:

**Windows**:
```
C:\Users\<username>\.auto\cache\
├── index.db         # SQLite metadata database
├── blobs\           # Binary artifacts
│   ├── a1\          # Hash prefix shards
│   │   └── a1b2c3...
│   └── f9\
└── locks\           # Process locks
```

**Linux/macOS**:
```
/home/<username>/.auto/cache/
├── index.db
├── blobs/
│   ├── a1/
│   └── f9/
└── locks/
```

### Cache Key Computation

Each cached artifact has a unique key based on:

1. **Content Hash**: BLAKE3 hash of source code
2. **Context Hash**: Target platform, optimization level, compiler flags
3. **Dependency Hash**: Merkle root of module dependencies

```rust
cache_key = format!("{}_{}_{}", module_name, artifact_type, fingerprint.target_hash())
```

This ensures:
- Same source → same cache key
- Different targets → different cache entries
- Modified dependencies → cache invalidated

### Cache Lifecycle

```
┌─────────────┐
│  Build      │
│  Requested  │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│ Query Cache     │──► Hit? ──► Return cached artifact
└──────┬──────────┘
       │ Miss
       ▼
┌─────────────────┐
│ Transpile/      │
│ Compile         │
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│ Store in Cache  │
└─────────────────┘
```

---

## CLI Commands

AutoCache provides management commands through the `auto cache` CLI.

### cache stats

Display cache statistics and hit rate.

```bash
$ auto cache stats

=== AutoCache Statistics ===

Total Artifacts:       156
Cache Size:            2.3 GB / 10.0 GB
Hit Rate (7 days):     78.5%

Last GC:               2 days ago
Cache Location:        C:\Users\user\.auto\cache
```

### cache list

List all cached artifacts with optional filtering.

**List all artifacts (default limit: 50)**:
```bash
$ auto cache list

=== Cached Artifacts (showing 50 of 156) ===

Module                           Type        Size        Last Used    Access
--------------------------------  --------    --------    -----------   ------
std:io                           C           12.3 KB     2m ago       15
std:fs                           C           8.7 KB      5m ago       8
myapp:main                       Rust        15.2 KB     1h ago       3
stdlib:math                      C           45.6 KB     2d ago       12
mylib:utils                      Bytecode    8.9 KB      3d ago       1

(Top 50 artifacts shown)
```

**Filter by type**:
```bash
$ auto cache list --type c         # Only C files
$ auto cache list --type rust      # Only Rust files
$ auto cache list --type bytecode  # Only bytecode files
```

**Custom limit**:
```bash
$ auto cache list --limit 100      # Show up to 100 artifacts
```

### cache inspect

Inspect a specific cache entry by hash key or module name.

**Inspect by hash key** (most precise):
```bash
$ auto cache inspect a1b2c3d4e5f6...

=== Cache Entry: std:io ===

Hash Key:         a1b2c3d4e5f6789...
Module:           std:io
Type:             C
Size:             12.3 KB
Source Hash:      abc123def456...
Project:          my_project
Created:          2025-01-15 10:30:00 UTC
Last Used:        2025-01-15 14:25:00 UTC
Access Count:     15
Blob Path:        C:\Users\user\.auto\cache\blobs\a1\a1b2c3d4...
```

**Inspect by module name** (fuzzy search):
```bash
$ auto cache inspect std:io
$ auto cache inspect io           # Finds std:io, io:file, etc.
```

**Multiple matches**:
```bash
$ auto cache inspect io

Found 3 cache entries matching 'io':

  [a1b2c3d4...] std:io - C (12.3 KB)
  [b2c3d4e5...] io:file - Rust (15.2 KB)
  [c3d4e5f6...] myapp:io_utils - Bytecode (8.1 KB)

Use specific hash key for full details.
```

### cache verify

Verify cache integrity and detect corruption.

```bash
$ auto cache verify

=== Verifying Cache Integrity ===

Checking metadata entries...
Checking blob files...
Verifying file integrity...

✓ Cache integrity verified
  - 156 metadata entries
  - 156 blob files
  - 0 orphaned files
  - 0 corrupted entries
```

**With issues**:
```bash
$ auto cache verify

=== Verifying Cache Integrity ===

Checking metadata entries...
Checking blob files...
Verifying file integrity...

⚠ Cache integrity issues detected
  - 156 metadata entries
  - 154 blob files
  - 0 orphaned files
  - 2 corrupted entries

Recommendations:
  - Run `auto cache clear` to remove corrupted entries
```

### cache prune

Manually run garbage collection to free disk space.

```bash
$ auto cache prune

Running cache garbage collection...
GC: Freed 45 artifacts (234 MB)
Cache GC: freed 234 MB
```

**When GC runs automatically**:
- After each build (if cache > 10GB)
- When cache size exceeds 80% of limit (8GB)

**GC policy**:
- Removes least recently used (LRU) artifacts first
- Stops when cache is 80% of max size (8GB)
- Never deletes artifacts accessed in the last 7 days

### cache clear

Clear all cached artifacts.

```bash
$ auto cache clear

This will delete all 156 cached artifacts (2.3 GB).
Proceed? [y/N]: y

Clearing all cache...
Cache cleared successfully.
```

**Warning**: This is irreversible. All cached artifacts are deleted.

---

## Cache Management

### Checking Cache Status

**Is AutoCache enabled?**
```bash
$ echo $AUTO_CACHE_ENABLED
true    # Enabled
(empty) # Disabled
```

**Where is the cache located?**
```bash
$ auto cache stats
# Look for "Cache Location:" field
```

**How big is the cache?**
```bash
$ auto cache stats
# Look for "Cache Size:" field
```

### Disabling AutoCache

**Temporary disable** (single command):
```bash
AUTO_CACHE_ENABLED=false auto build
```

**Disable for session**:
```bash
# Unset the environment variable
unset AUTO_CACHE_ENABLED  # Linux/macOS
set AUTO_CACHE_ENABLED=   # Windows
```

### Cache Size Limits

**Default limits**:
- Maximum size: 10 GB
- GC watermark: 8 GB (80%)
- Minimum artifact size: No minimum

**Changing limits** (requires code modification):
Edit `crates/auto-cache/src/lib.rs`:
```rust
let gc = GarbageCollector::new(10);  // Change 10 to desired GB limit
```

---

## Troubleshooting

### Cache Not Working

**Problem**: Builds are not faster, cache always misses.

**Solutions**:
1. Check if AutoCache is enabled:
   ```bash
   echo $AUTO_CACHE_ENABLED  # Should be "true"
   ```

2. Check cache location exists:
   ```bash
   ls ~/.auto/cache  # Linux/macOS
   dir C:\Users\%USERNAME%\.auto\cache  # Windows
   ```

3. Check cache has artifacts:
   ```bash
   auto cache list
   ```

### Corruption Detected

**Problem**: `auto cache verify` reports corrupted entries.

**Solutions**:
1. Run integrity check:
   ```bash
   auto cache verify
   ```

2. Clear corrupted cache:
   ```bash
   auto cache clear
   ```

3. Rebuild to repopulate cache:
   ```bash
   AUTO_CACHE_ENABLED=true auto build
   ```

### Out of Space

**Problem**: Disk full, cache too large.

**Solutions**:
1. Run garbage collection:
   ```bash
   auto cache prune
   ```

2. Clear entire cache:
   ```bash
   auto cache clear
   ```

3. Change cache location (requires code modification):
   Edit `crates/auto-cache/src/lib.rs`:
   ```rust
   let cache_dir = dirs::home_dir()?
       .join(".auto")
       .join("cache");
   // Change to different path
   ```

### Permission Errors

**Problem**: "Permission denied" when accessing cache.

**Solutions**:
1. Check cache directory permissions:
   ```bash
   ls -la ~/.auto/cache  # Linux/macOS
   ```

2. Fix permissions:
   ```bash
   chmod 755 ~/.auto/cache  # Linux/macOS
   ```

3. Ensure home directory is writable:
   ```bash
   # You should own ~/.auto
   chown $USER:$USER ~/.auto  # Linux
   ```

---

## Best Practices

### When to Enable AutoCache

✅ **Enable for**:
- Development builds
- CI/CD pipelines with multiple projects
- Projects sharing common dependencies
- Large codebases with long build times

❌ **Disable for**:
- Production builds (need clean build)
- One-off builds
- When disk space is very limited

### Cache Maintenance

**Regular tasks**:
1. **Weekly**: Check cache size
   ```bash
   auto cache stats
   ```

2. **Monthly**: Run integrity check
   ```bash
   auto cache verify
   ```

3. **Quarterly**: Clear and rebuild cache
   ```bash
   auto cache clear
   AUTO_CACHE_ENABLED=true auto build
   ```

### Multi-Project Setups

**Shared libraries**:
```bash
# Build stdlib first
cd /path/to/stdlib
export AUTO_CACHE_ENABLED=true
auto build

# Build app (reuses stdlib from cache)
cd /path/to/app
auto build  # stdlib:io, stdlib:fs will be cache hits!
```

**CI/CD integration**:
```yaml
# .github/workflows/build.yml
env:
  AUTO_CACHE_ENABLED: "true"
  CACHE_DIR: "~/.auto/cache"

steps:
  - name: Build with cache
    run: auto build

  - name: Cache stats
    run: auto cache stats
```

---

## Performance Tips

### Maximizing Cache Hit Rate

1. **Keep dependencies stable**: Avoid frequent changes to shared modules
2. **Use consistent compiler flags**: Changes invalidate cache
3. **Batch builds**: Build multiple projects in sequence to warm cache
4. **Avoid cleaning**: Don't run `auto cache clear` unnecessarily

### Measuring Performance

**Before AutoCache**:
```bash
time auto build  # Clean build
# 15.3s
```

**After AutoCache** (warm cache):
```bash
time auto build  # Cached build
# 2.1s (7.3x faster!)
```

**Track hit rate**:
```bash
auto cache stats
# Look for "Hit Rate (7 days): XX%"
```

### Cache Warming

**For important builds**:
```bash
# Build all dependencies first
cd dependencies/
export AUTO_CACHE_ENABLED=true
auto build

# Now build main project (fast!)
cd ../main-project
auto build  # Most dependencies will be cache hits
```

---

## Advanced Usage

### Custom Cache Location

Set custom cache location via environment variable (future feature):
```bash
export AUTO_CACHE_DIR=/mnt/ssd/cache  # Not yet implemented
```

For now, modify `crates/auto-cache/src/lib.rs` to change the default path.

### Programmatic Usage

```rust
use auto_cache::{AutoManCache, ArtifactType, CompilationTarget};

// Create cache manager
let cache = AutoManCache::in_home_dir("my_project".to_string())?;

// Query before transpilation
if let Some((c_path, h_path)) = cache.query("std:io", source_code) {
    // Cache hit - use cached files
    return Ok((c_path, h_path));
}

// Cache miss - transpile normally
let (c_code, h_code) = transpile_c(source_code)?;

// Store in cache
cache.store("std:io", source_code, &c_path, Some(&h_path))?;
```

See [AutoCache API Documentation](../architecture/autocache.md) for full API reference.

---

## FAQ

**Q: Does AutoCache work across different compiler versions?**

A: No. Cache keys include compiler flags and toolchain version. Different compiler versions produce different cache keys.

**Q: Can I share cache between users?**

A: Not currently. Cache is stored per-user in `~/.auto/cache`. Future versions may support shared network caches.

**Q: How much disk space does AutoCache use?**

A: Typically 1-5 GB for active development. Maximum is 10 GB (configurable).

**Q: Does AutoCache cache intermediate files?**

A: No. Only final artifacts (.c, .h, .rs, .bc, .o) are cached. Intermediate compilation products are not cached.

**Q: Can AutoCache be used with other build systems?**

A: AutoCache is designed for AutoMan (Plan 082). Integration with other build systems (Make, CMake) would require additional tooling.

**Q: Is AutoCache safe to use on network drives?**

A: Not recommended. Hard link optimization fails on network drives, causing cache to fall back to slower copy operations.

**Q: How do I completely uninstall AutoCache?**

A:
1. Disable: `unset AUTO_CACHE_ENABLED`
2. Remove cache directory: `rm -rf ~/.auto/cache`

---

## Getting Help

- **Documentation**: [AutoCache Architecture](../architecture/autocache.md)
- **CLI Reference**: `auto cache --help`
- **Issues**: Report bugs at [AutoLang GitHub Issues](https://github.com/anthropics/autolang/issues)

---

**Last Updated**: 2025
**AutoCache Version**: 1.0 (Plan 082)
