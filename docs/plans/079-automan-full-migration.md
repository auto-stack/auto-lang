# Plan 079: Full AutoMan Migration Strategy

**Date**: 2026-02-06
**Status**: ✅ **COMPLETE**
**Completion**: 2026-02-06
**Time**: ~4 hours
**Related**: Plan 078 (ModuleResolver Foundation), ../auto-man (Source Project)

---

## 🎯 Overview

**Objective**: Smoothly migrate the comprehensive `../auto-man` build system and package manager into the auto-lang monorepo, building upon the ModuleResolver foundation established in Plan 078.

**Source Analysis**:
- **Location**: `../auto-man` (sibling to `auto-lang/`)
- **Size**: 6,402 lines of Rust code across 41 files
- **Scope**: Complete build system and package manager
- **Dependencies**: auto-lang, auto-val, auto-gen, CLI utilities

**Current State (Plan 078)**:
- ✅ ModuleResolver trait defined in auto-lang
- ✅ Basic AutoManResolver implementation (417 lines)
- ✅ Workspace configured
- ❌ Missing: Build system integration, dependency management, CLI tooling

---

## 📊 Gap Analysis

### What Plan 078 Implemented
```
crates/auto-man/
├── src/
│   ├── lib.rs          (60 lines) - Basic exports
│   ├── error.rs        (60 lines) - Simple errors
│   └── resolver.rs     (417 lines) - ModuleResolver impl
└── Cargo.toml
```

### What ../auto-man Has
```
../auto-man/crates/auto-man/src/
├── lib.rs              (40 lines) - Full exports
├── main.rs             (340 lines) - CLI entry point
├── automan.rs          (580 lines) - Main orchestrator
├── pac.rs              (1,470 lines) - Package management
├── target.rs           (1,130 lines) - Build targets
├── builder.rs          (150 lines) - Builder trait
├── builder/
│   ├── cmake.rs        (280 lines)
│   ├── iar.rs          (290 lines)
│   ├── ghs.rs          (280 lines)
│   └── ninja/          (5 sub-files, ~1,500 lines)
├── index.rs            (160 lines) - Package index
├── cache.rs            (210 lines) - Build caching
├── git.rs              (95 lines) - Git operations
├── scanner.rs          (160 lines) - Source scanning
├── lock.rs             (250 lines) - Lock files
├── pull.rs             (20 lines) - Package pulling
├── up.rs               (300 lines) - Self-update
├── asset.rs            (140 lines) - Embedded templates
├── dir.rs              (490 lines) - Directory operations
├── file_types.rs       (250 lines) - File type handling
├── group.rs            (75 lines) - Target grouping
├── node_ext.rs         (60 lines) - Node extensions
├── port.rs             (45 lines) - Port definitions
├── util.rs             (70 lines) - Utilities
├── version.rs          (340 lines) - Version handling
└── fs.rs               (20 lines) - FS utilities
```

**Total**: ~6,402 lines vs ~537 lines (Plan 078)

---

## 🚀 Migration Strategy

### Approach: **Phased Incremental Migration**

**Rationale**:
1. **Preserve Plan 078 work** - Keep ModuleResolver trait implementation
2. **Maintain compilation** - Each phase compiles successfully
3. **Test incrementally** - Verify functionality at each step
4. **Minimize disruption** - AutoLang continues working during migration

**Migration Path**:
```
Phase 0: Foundation (✅ COMPLETE - Plan 078)
  └─ ModuleResolver trait + basic AutoManResolver

Phase 1: Core Integration (🔄 NEXT)
  ├─ Merge lib.rs exports
  ├─ Add error handling
  └─ Verify compilation

Phase 2: Package Management
  ├─ Migrate pac.rs (core package logic)
  ├─ Migrate index.rs (package index)
  ├─ Migrate lock.rs (lock files)
  └─ Migrate git.rs (git operations)

Phase 3: Build System
  ├─ Migrate builder.rs (trait)
  ├─ Migrate builder/cmake.rs
  ├─ Migrate builder/ninja/*
  ├─ Migrate builder/iar.rs
  └─ Migrate builder/ghs.rs

Phase 4: Target & Scanner
  ├─ Migrate target.rs
  ├─ Migrate scanner.rs
  ├─ Migrate cache.rs
  └─ Migrate dir.rs

Phase 5: CLI & Utilities
  ├─ Migrate main.rs
  ├─ Migrate util.rs
  ├─ Migrate asset.rs
  ├─ Migrate version.rs
  └─ Add `[[bin]]` section to Cargo.toml

Phase 6: Testing & Polish
  ├─ Integration tests
  ├─ CLI smoke tests
  ├─ Documentation
  └─ Remove ../auto-man
```

---

## 📝 Phase 1: Core Integration (Immediate Next Steps)

### Goal
Merge the foundational modules from ../auto-man while preserving Plan 078's resolver work.

### Tasks

#### 1.1 Update lib.rs Exports
**File**: `crates/auto-man/src/lib.rs`

**Current** (Plan 078):
```rust
pub mod error;
pub mod resolver;

pub use error::*;
pub use resolver::AutoManResolver;

pub const AUTOMAN_VERSION: &str = env!("CARGO_PKG_VERSION");
```

**Target** (merge from ../auto-man):
```rust
// Plan 078: ModuleResolver
pub mod error;
pub mod resolver;
pub use resolver::AutoManResolver;

// Phase 1: Core modules
pub mod asset;
mod automan;
mod cache;
mod builder;
mod dir;
pub mod file_types;
pub mod fs;
pub mod git;
pub mod group;
mod index;
mod lock;
mod node_ext;
mod pac;
mod port;
pub mod pull;
mod scanner;
mod target;
pub mod up;
pub mod util;
pub mod version;

// Re-exports
pub use automan::*;
pub use builder::*;
pub use cache::*;
pub use dir::*;
pub use error::*;
pub use file_types::*;
pub use index::*;
pub use lock::*;
pub use pac::*;
pub use port::*;
pub use target::*;
pub use version::*;

// AutoVal re-exports
pub use auto_val::AutoError;
pub use auto_val::AutoResult;

pub const AUTOMAN_VERSION: &str = env!("CARGO_PKG_VERSION");
```

#### 1.2 Merge Error Types
**File**: `crates/auto-man/src/error.rs`

**Current**: Simple thiserror-based errors
**Target**: Merge with ../auto-man's error.rs (add AutoMan-specific variants)

#### 1.3 Verify Dependencies
**File**: `crates/auto-man/Cargo.toml`

**Add missing dependencies** from ../auto-man:
```toml
[dependencies]
# Existing (Plan 078)
auto-lang = { path = "../auto-lang" }
auto-val = { path = "../auto-val" }
auto-gen = { path = "../auto-gen" }
thiserror = "2.0"
clap = { version = "4.5", features = ["derive"] }
rust-embed = { version = "8.5", features = ["compression"] }

# Add from ../auto-man
tabled = "0.18"
simplelog = "0.12"
log = "0.4"
normalize-path = "0.2"
dialoguer = "0.11"
glob = "0.3"
dirs = "6.0"
reqwest = { version = "0.12", features = ["blocking"] }
version-compare = "0.0"
colored = "3"
zip = "3.0"
glob-match = "0.2"
encoding_rs = "0.8"
is_executable = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
indicatif = "0.17"

[dev-dependencies]
tempfile = "3.14"
```

#### 1.4 Copy Core Modules
Execute in order:
```bash
# Copy core modules (no internal dependencies)
cp ../auto-man/crates/auto-man/src/node_ext.rs crates/auto-man/src/
cp ../auto-man/crates/auto-man/src/port.rs crates/auto-man/src/
cp ../auto-man/crates/auto-man/src/util.rs crates/auto-man/src/
cp ../auto-man/crates/auto-man/src/fs.rs crates/auto-man/src/
cp ../auto-man/crates/auto-man/src/group.rs crates/auto-man/src/
cp ../auto-man/crates/auto-man/src/pull.rs crates/auto-man/src/
cp ../auto-man/crates/auto-man/src/up.rs crates/auto-man/src/

# Copy asset module
cp ../auto-man/crates/auto-man/src/asset.rs crates/auto-man/src/
cp -r ../auto-man/crates/auto-man/assets crates/auto-man/

# Test compilation
cargo check -p auto-man
```

**Expected Result**: Compiles with warnings about unused modules (normal at this stage)

---

## 📝 Phase 2: Package Management

### Goal
Migrate the core package management logic (pac, index, lock, git).

### Tasks

#### 2.1 Copy Package Modules
```bash
# Copy in dependency order
cp ../auto-man/crates/auto-man/src/git.rs crates/auto-man/src/
cp ../auto-man/crates/auto-man/src/index.rs crates/auto-man/src/
cp ../auto-man/crates/auto-man/src/lock.rs crates/auto-man/src/
cp ../auto-man/crates/auto-man/src/pac.rs crates/auto-man/src/
cp ../auto-man/crates/auto-man/src/automan.rs crates/auto-man/src/

# Test compilation
cargo check -p auto-man
```

#### 2.2 Fix Import Paths
**Issue**: ../auto-man imports from `auto_lang::*`, but we need to ensure paths are correct.

**Check these imports in copied files**:
- `auto_lang::config::AutoConfig` → Should work (already exists)
- `auto_lang::Atom` → Should work (exported from lib.rs)
- `auto_lang::Universe` → Deprecated, but pac.rs uses it

**Fix if needed**: Update imports to match current auto-lang exports.

#### 2.3 Integration with Resolver
**Opportunity**: Enhance Plan 078's AutoManResolver to use pac.rs logic.

**Current**: Simple pac.at parsing
**Target**: Use Pac struct from pac.rs for full configuration

```rust
// In resolver.rs, add method:
impl AutoManResolver {
    pub fn from_pac(pac: &Pac) -> Self {
        // Extract dependencies from Pac object
        // Build resolver with full configuration
    }
}
```

---

## 📝 Phase 3: Build System

### Goal
Migrate builder system (CMake, Ninja, IAR, GHS).

### Tasks

#### 3.1 Copy Builder Infrastructure
```bash
# Copy builder trait and main module
cp ../auto-man/crates/auto-man/src/builder.rs crates/auto-man/src/

# Create builder directory
mkdir -p crates/auto-man/src/builder

# Copy builder implementations
cp ../auto-man/crates/auto-man/src/builder/cmake.rs crates/auto-man/src/builder/
cp ../auto-man/crates/auto-man/src/builder/iar.rs crates/auto-man/src/builder/
cp ../auto-man/crates/auto-man/src/builder/ghs.rs crates/auto-man/src/builder/
cp -r ../auto-man/crates/auto-man/src/builder/ninja crates/auto-man/src/builder/
cp ../auto-man/crates/auto-man/src/builder/tool.rs crates/auto-man/src/builder/

# Test compilation
cargo check -p auto-man
```

#### 3.2 Copy Builder Assets
```bash
# Builder templates (IAR, GHS project files)
cp -r ../auto-man/crates/auto-man/assets/builders crates/auto-man/assets/
```

---

## 📝 Phase 4: Target & Scanner

### Goal
Migrate target discovery and source scanning logic.

### Tasks

#### 4.1 Copy Target & Scanner Modules
```bash
cp ../auto-man/crates/auto-man/src/target.rs crates/auto-man/src/
cp ../auto-man/crates/auto-man/src/scanner.rs crates/auto-man/src/
cp ../auto-man/crates/auto-man/src/cache.rs crates/auto-man/src/
cp ../auto-man/crates/auto-man/src/dir.rs crates/auto-man/src/
cp ../auto-man/crates/auto-man/src/file_types.rs crates/auto-man/src/

# Test compilation
cargo check -p auto-man
```

#### 4.2 Copy Version Module
```bash
cp ../auto-man/crates/auto-man/src/version.rs crates/auto-man/src/
```

---

## 📝 Phase 5: CLI & Utilities

### Goal
Migrate CLI entry point and enable binary compilation.

### Tasks

#### 5.1 Copy CLI Entry Point
```bash
cp ../auto-man/crates/auto-man/src/main.rs crates/auto-man/src/

# Update Cargo.toml to enable binary
```

#### 5.2 Enable Binary in Cargo.toml
**File**: `crates/auto-man/Cargo.toml`

```toml
[[bin]]
name = "auto-man"
path = "src/main.rs"
```

#### 5.3 Test CLI
```bash
# Build binary
cargo build -p auto-man

# Test help
./target/debug/auto-man --help

# Test version
./target/debug/auto-man --version
```

---

## 📝 Phase 6: Testing & Cleanup

### Goal
Verify full functionality and remove old project.

### Tasks

#### 6.1 Integration Tests
- Create test project with `pac.at`
- Test `auto-man build` command
- Test dependency resolution
- Test builder integration

#### 6.2 Update Documentation
- Update CLAUDE.md with new structure
- Update README with migration info
- Document AutoManResolver + Pac integration

#### 6.3 Remove Old Project
```bash
# After verification, remove ../auto-man
mv ../auto-man ../auto-man.backup
# Test everything still works
rm -rf ../auto-man.backup
```

---

## 🔧 Technical Considerations

### Dependency Resolution
- **Issue**: ../auto-man uses `auto_lang::*` imports
- **Solution**: Verify all exports exist in `crates/auto-lang/src/lib.rs`

### Workspace Dependencies
- **Issue**: ../auto-man uses workspace dependencies
- **Solution**: Already configured in `crates/Cargo.toml` (Plan 078)

### Asset Embedding
- **Issue**: `rust-embed` needs correct asset path
- **Solution**: Copy `assets/` directory and verify paths in asset.rs

### Deprecated APIs
- **Issue**: pac.rs uses deprecated `Universe`
- **Solution**: Keep for now, refactor in future plan

---

## ✅ Success Criteria

### Phase 1 Success
- ✅ All core modules copied
- ✅ `cargo check -p auto-man` passes
- ✅ ModuleResolver still works
- ✅ No breaking changes to auto-lang

### Phase 2 Success
- ✅ Package management works
- ✅ Can parse pac.at files
- ✅ Integration with AutoManResolver

### Phase 3 Success
- ✅ All builders compile
- ✅ Can generate build files
- ✅ Builder assets embedded

### Phase 4 Success
- ✅ Target scanning works
- ✅ Source discovery works
- ✅ File type detection works

### Phase 5 Success
- ✅ CLI binary builds
- ✅ `auto-man --help` works
- ✅ Can run basic commands

### Phase 6 Success
- ✅ Full integration test passes
- ✅ Documentation updated
- ✅ Old project removed

---

## 📊 Estimated Effort

| Phase | Files | Lines | Complexity | Time |
|-------|-------|-------|------------|------|
| Phase 1 | 10 | ~200 | Low | 1-2 hours |
| Phase 2 | 5 | ~2,500 | Medium | 2-3 hours |
| Phase 3 | 8 | ~2,600 | High | 3-4 hours |
| Phase 4 | 5 | ~2,200 | Medium | 2-3 hours |
| Phase 5 | 3 | ~400 | Low | 1-2 hours |
| Phase 6 | - | - | Medium | 2-3 hours |
| **Total** | **31** | **~7,900** | **High** | **11-17 hours** |

---

## 🚦 Risk Mitigation

### High Risk Areas
1. **Import path mismatches** - Systematic check of all imports
2. **Workspace dependency conflicts** - Verify versions match
3. **Asset embedding** - Test rust-embed paths carefully
4. **Deprecated API usage** - Accept for now, document tech debt

### Rollback Strategy
- Each phase is git commit atomic
- Can revert to Plan 078 state if needed
- Keep ../auto-man until Phase 6 verification complete

---

## 📚 Related Documents

- [Plan 078: ModuleResolver Foundation](078-progress.md)
- [Plan 078: Integration Design](078-automan-integration.md)
- [../auto-man CLAUDE.md](../../auto-man/CLAUDE.md) - Source project docs
- [../auto-man README.md](../../auto-man/README.md) - Usage guide

---

**Status**: 📋 Ready to begin Phase 1
**Next Action**: Execute Phase 1 tasks 1.1-1.4
**Blocking**: None (Plan 078 complete)
