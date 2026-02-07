# AutoCache CLI Reference

**Version**: 1.0
**Component**: AutoMan CLI (Plan 082)

## Overview

AutoCache CLI commands are accessed via the `auto cache` subcommand. All cache operations require AutoCache to be enabled via the `AUTO_CACHE_ENABLED` environment variable.

## Table of Contents

1. [Quick Reference](#quick-reference)
2. [Commands](#commands)
3. [Environment Variables](#environment-variables)
4. [Exit Codes](#exit-codes)
5. [Examples](#examples)

---

## Quick Reference

| Command | Description | Usage |
|---------|-------------|-------|
| `cache stats` | Show cache statistics | `auto cache stats` |
| `cache list` | List cached artifacts | `auto cache list [--type TYPE] [--limit N]` |
| `cache inspect` | Inspect specific entry | `auto cache inspect <hash or name>` |
| `cache verify` | Verify cache integrity | `auto cache verify` |
| `cache prune` | Run garbage collection | `auto cache prune` |
| `cache clear` | Delete all cached artifacts | `auto cache clear` |

---

## Commands

### cache stats

Display cache statistics including size, hit rate, and artifact count.

#### Usage

```bash
auto cache stats
```

#### Output

```
=== AutoCache Statistics ===

Total Artifacts:       156
Cache Size:            2.3 GB / 10.0 GB
Hit Rate (7 days):     78.5%

Last GC:               2 days ago
Cache Location:        C:\Users\user\.auto\cache
```

#### Fields

- **Total Artifacts**: Number of cached artifacts
- **Cache Size**: Current / maximum size
- **Hit Rate (7 days)**: Percentage of cache hits in last 7 days
- **Last GC**: Time since last garbage collection
- **Cache Location**: Path to cache directory

#### Exit Codes

- **0**: Success
- **1**: Error (cache not enabled, permission denied, etc.)

---

### cache list

List all cached artifacts with optional filtering by type and configurable limit.

#### Usage

```bash
auto cache list [OPTIONS]
```

#### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--type <TYPE>` | `-t` | Filter by artifact type | All types |
| `--limit <N>` | `-l` | Maximum number of artifacts to show | 50 |

#### Artifact Types

| Type | Description |
|------|-------------|
| `c` | C source files (.c) |
| `h` | C header files (.h) |
| `rust` | Rust source files (.rs) |
| `bytecode` | AutoVM bytecode files (.bc) |
| `object` | Compiled object files (.o, .obj) |

#### Output

```
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

#### Fields

- **Module**: Module name (format: `project:module` or sanitized)
- **Type**: Artifact type (C, C Header, Rust, Bytecode, Object)
- **Size**: File size (B/KB/MB/GB)
- **Last Used**: Time since last access (e.g., "2m ago", "1h ago", "3d ago")
- **Access**: Number of times artifact was accessed

#### Examples

**List all C artifacts**:
```bash
auto cache list --type c
```

**List up to 100 Rust artifacts**:
```bash
auto cache list --type rust --limit 100
```

**List all bytecode files**:
```bash
auto cache list --type bytecode
```

#### Exit Codes

- **0**: Success
- **1**: Error (cache not enabled, invalid type, etc.)

---

### cache inspect

Inspect a specific cache entry by hash key or module name. Shows full metadata including file paths and timestamps.

#### Usage

```bash
auto cache inspect <HASH_OR_NAME>
```

#### Arguments

| Argument | Description |
|----------|-------------|
| `HASH_OR_NAME` | Full hash key or module name (fuzzy search supported) |

#### Search Behavior

1. **Exact hash match**: Shows full metadata if hash key is found
2. **Module name search**: Shows list of matching artifacts if fuzzy match
3. **No match**: Error message with suggestions

#### Output (Single Match)

```
=== Cache Entry: std:io ===

Hash Key:         a1b2c3d4e5f6789abc123def4567890...
Module:           std:io
Type:             C
Size:             12.3 KB
Source Hash:      abc123def4567890...
Project:          my_project
Created:          2025-01-15 10:30:00 UTC
Last Used:        2025-01-15 14:25:00 UTC
Access Count:     15
Blob Path:        C:\Users\user\.auto\cache\blobs\a1\a1b2c3d4e5f6...
```

#### Output (Multiple Matches)

```
Found 3 cache entries matching 'io':

  [a1b2c3d4...] std:io - C (12.3 KB)
  [b2c3d4e5...] io:file - Rust (15.2 KB)
  [c3d4e5f6...] myapp:io_utils - Bytecode (8.1 KB)

Use specific hash key for full details.
```

#### Fields

- **Hash Key**: Full cache key (BLAKE3 hex)
- **Module**: Module name
- **Type**: Artifact type
- **Size**: File size
- **Source Hash**: AIE interface hash (hex)
- **Project**: Origin project name
- **Created**: Creation timestamp (UTC)
- **Last Used**: Last access timestamp (UTC)
- **Access Count**: Number of accesses
- **Blob Path**: Path to cached blob file

#### Examples

**Inspect by exact hash**:
```bash
auto cache inspect a1b2c3d4e5f6789abc123def4567890...
```

**Inspect by module name**:
```bash
auto cache inspect std:io
```

**Fuzzy search**:
```bash
auto cache inspect io  # Matches std:io, io:file, etc.
```

#### Exit Codes

- **0**: Success (entry found)
- **1**: Error (cache not enabled, entry not found)

---

### cache verify

Verify cache integrity by checking metadata entries against blob files. Detects corrupted entries (metadata without blob) and orphaned files (blob without metadata).

#### Usage

```bash
auto cache verify
```

#### Output (Valid Cache)

```
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

#### Output (Issues Detected)

```
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
  - Orphaned files will be cleaned up by garbage collection
```

#### Fields

- **metadata entries**: Number of SQLite records
- **blob files**: Number of blob files in filesystem
- **orphaned files**: Blob files without metadata (cleaned by GC)
- **corrupted entries**: Metadata without blob files (needs cleanup)

#### Recommendations

**Corrupted entries detected**:
```
Recommendations:
  - Run `auto cache clear` to remove corrupted entries
```

**Orphaned files detected**:
```
Recommendations:
  - Orphaned files will be cleaned up by garbage collection
  - Run `auto cache prune` to force GC now
```

#### Examples

```bash
auto cache verify
```

#### Exit Codes

- **0**: Success (verification completed, may have issues)
- **1**: Error (cache not enabled, permission denied)

---

### cache prune

Manually run garbage collection to free disk space. Removes least recently used (LRU) artifacts until cache is 80% of maximum size.

#### Usage

```bash
auto cache prune
```

#### Output

```
Running cache garbage collection...
GC: Freed 45 artifacts (234 MB)
Cache GC: freed 234 MB
```

#### How GC Works

1. **Trigger**: Cache size > 10GB (or manual via `cache prune`)
2. **Target**: Reduce to 8GB (80% of max)
3. **Selection**: Least recently used (LRU) artifacts first
4. **Deletion**: Metadata + blob files deleted in transaction
5. **Protection**: Artifacts accessed in last 7 days are not deleted

#### GC Policy

| Parameter | Value |
|-----------|-------|
| Maximum size | 10 GB |
| Watermark | 8 GB (80%) |
| Min access age | 7 days (protected) |
| Selection | LRU (oldest last_used_at first) |

#### Examples

```bash
auto cache prune
```

**Check if GC is needed** (automatic):
```bash
auto cache stats
# Look for: "Cache Size: X GB / 10.0 GB"
# If X > 10, GC runs automatically after next build
```

#### Exit Codes

- **0**: Success (GC completed)
- **1**: Error (cache not enabled, permission denied)

---

### cache clear

Delete all cached artifacts from the cache. Requires confirmation before proceeding.

#### Usage

```bash
auto cache clear
```

#### Output

```
This will delete all 156 cached artifacts (2.3 GB).
Proceed? [y/N]: y

Clearing all cache...
Cache cleared successfully.
```

#### Confirmation Prompt

The command requires manual confirmation:
- **`y`** or **`Y`**: Proceed with deletion
- **`n`** or **`N`** (default): Cancel operation
- **Ctrl+C**: Cancel operation

#### What Gets Deleted

1. **All metadata** from SQLite database
2. **All blob files** from filesystem
3. **Directory structure** remains (blobs/, index.db)

#### Examples

```bash
auto cache clear
```

**Auto-confirm** (for scripts):
```bash
echo "y" | auto cache clear
```

#### Exit Codes

- **0**: Success (cache cleared)
- **1**: Error (cache not enabled, permission denied, user cancelled)

---

## Environment Variables

### AUTO_CACHE_ENABLED

**Description**: Enable or disable AutoCache

**Values**:
- `true` or `1`: Enable AutoCache
- `false`, `0`, or unset: Disable AutoCache

**Default**: `false` (disabled)

**Examples**:

```bash
# Enable for single command
AUTO_CACHE_ENABLED=true auto build

# Enable for session
export AUTO_CACHE_ENABLED=true
auto build

# Disable
unset AUTO_CACHE_ENABLED
```

**Check current value**:
```bash
echo $AUTO_CACHE_ENABLED
```

### AUTO_CACHE_DIR (Future)

**Description**: Custom cache directory location

**Note**: Not yet implemented. Currently hardcoded to `~/.auto/cache`.

**Planned Usage**:
```bash
export AUTO_CACHE_DIR=/mnt/ssd/autocache
auto build
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (cache not enabled, permission denied, etc.) |
| 2 | User cancelled (e.g., `auto cache clear` prompt) |

---

## Examples

### Basic Workflow

```bash
# 1. Enable AutoCache
export AUTO_CACHE_ENABLED=true

# 2. Build project (cache miss on first build)
auto build
# Output: [Cache Miss] std:io (C)
#         [Cache Miss] std:fs (C)

# 3. Check cache statistics
auto cache stats
# Output: Total Artifacts: 2
#         Cache Size: 21.0 KB / 10.0 GB

# 4. List cached artifacts
auto cache list
# Output: std:io    C    12.3 KB   Just now    1
#         std:fs    C    8.7 KB    Just now    1

# 5. Rebuild (cache hit)
auto build
# Output: [Cache Hit] std:io (C)
#         [Cache Hit] std:fs (C)
```

### Cache Management

```bash
# Verify cache integrity
auto cache verify
# Output: ✓ Cache integrity verified

# List only C artifacts
auto cache list --type c

# Inspect specific artifact
auto cache inspect std:io
# Output: Full metadata display

# Run garbage collection
auto cache prune
# Output: GC: Freed 0 artifacts (0 B)
#         (No GC needed yet)
```

### Debugging

```bash
# Check if AutoCache is enabled
echo $AUTO_CACHE_ENABLED
# Output: true

# Check cache location
auto cache stats
# Output: Cache Location: C:\Users\user\.auto\cache

# Verify cache integrity
auto cache verify

# Clear and rebuild cache
auto cache clear
echo "y" | auto cache clear
auto build
```

### Cross-Project Build

```bash
# Build stdlib project
cd /path/to/stdlib
export AUTO_CACHE_ENABLED=true
auto build

# Build app project (reuses stdlib from cache)
cd /path/to/app
auto build
# stdlib:io and stdlib:fs will be cache hits!
```

### CI/CD Integration

```yaml
# .github/workflows/build.yml
name: Build with AutoCache

env:
  AUTO_CACHE_ENABLED: "true"

steps:
  - name: Checkout code
    - uses: actions/checkout@v2

  - name: Build with cache
    - run: |
        auto cache stats
        auto build
        auto cache stats

  - name: Verify cache
    - run: auto cache verify
```

---

## Troubleshooting

### Command Not Found

**Error**: `auto: command not found`

**Solution**: Install AutoMan (Plan 081)
```bash
cargo install auto-man
```

### Cache Not Enabled

**Error**: `AutoCache is not enabled. Set AUTO_CACHE_ENABLED=true to enable.`

**Solution**: Enable environment variable
```bash
export AUTO_CACHE_ENABLED=true
```

### Permission Denied

**Error**: `Permission denied: ~/.auto/cache`

**Solution**: Fix permissions
```bash
chmod 755 ~/.auto/cache
```

### Cache Directory Missing

**Error**: `Cache not found at: ~/.auto/cache`

**Solution**: Create directory
```bash
mkdir -p ~/.auto/cache
```

---

## Related Documentation

- [User Guide](../guides/autocache-guide.md)
- [Architecture Documentation](../architecture/autocache.md)
- [Plan 082: Implementation Summary](../../plan-082.md)

---

**Last Updated**: 2025
**AutoCache Version**: 1.0 (Plan 082)
