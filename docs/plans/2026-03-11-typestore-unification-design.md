# Design: TypeStore Unification - Single Source of Truth for Type Information

**Date**: 2026-03-11
**Status**: Approved
**Related Plans**: Plan 084 (TypeStore), Plan 087 (Generics), Plan 064 (Database)

## Overview

Consolidate all type registries into a single `TypeStore` as the source of truth for type information across the compiler. This fixes `test_enum_decl_compiles` and completes Plan 084's follow-up work.

## Problem Statement

### Current State - Four Type Storages

| Location | Contents | Owner | Purpose |
|----------|----------|-------|---------|
| `types.rs` (TypeStore) | TypeDecl, Fn, Spec, GenericTemplate | Parser, Codegen | Plan 084 unified storage |
| `type_registry.rs` | `HashMap<String, Type>` | Parser, autovm_persistent | REPL persistence |
| `infer/registry.rs` | TypeDecl, ClassTemplate | InferenceContext | Type inference |
| `Database.type_info_store` | TypeInfo (method names only) | Database | Incremental compilation |

### Issues

1. **Data duplication** - TypeDecl stored in 3 places
2. **Sync complexity** - Changes must propagate to multiple registries
3. **Incomplete data** - `TypeInfo` only has method names, not field types
4. **Missing enum support** - EnumDecl not registered anywhere
5. **Failing test** - `test_enum_decl_compiles` fails because enums aren't registered

## Target Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                 TypeStore - Single Source of Truth                  │
│                 Arc<RwLock<TypeStore>>                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  type_decls: HashMap<AutoStr, Rc<TypeDecl>>                         │
│  enum_decls: HashMap<AutoStr, Rc<EnumDecl>>      ← NEW              │
│  fn_decls: HashMap<Name, Rc<Fn>>                                    │
│  spec_decls: HashMap<AutoStr, Rc<SpecDecl>>                         │
│  generic_templates: HashMap<String, ClassTemplate>                  │
│  type_aliases: HashMap<AutoStr, AutoStr>                            │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                       Key Methods                                    │
│                                                                      │
│  register_type_decl(decl)        // Creates ClassTemplate too       │
│  register_enum_decl(decl)        // NEW                             │
│  lookup_type_decl(name) → Option<Rc<TypeDecl>>                      │
│  lookup_enum_decl(name) → Option<Rc<EnumDecl>>                      │
│  get_class_template(name) → Option<&ClassTemplate>                  │
│  is_type(name) → bool             // Unified type check             │
│  get_type(name) → Option<Type>    // For REPL                       │
│  get_enum_variant_value(enum, variant) → Option<i32>                │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                       Consumers                                      │
│                                                                      │
│  Parser ────────────► type_store.write()  (register types)          │
│  Codegen ───────────► type_store.read()   (lookup types)            │
│  InferenceContext ──► type_store.read()   (type inference)          │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                     Deprecated → Removed                             │
│                                                                      │
│  ✗ type_registry.rs (simple)     → TypeStore                        │
│  ✗ infer/registry.rs             → TypeStore                        │
│  ✗ Database.type_info_store      → TypeStore                        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Key Design Decisions

### 1. Use `Rc<T>` for Shared Immutable References

Type declarations are immutable after definition. Using `Rc<TypeDecl>` allows:
- Cheap cloning (just increment ref count)
- No lifetime issues when returning from behind `RwLock`
- Multiple consumers can hold references simultaneously

```rust
pub struct TypeStore {
    type_decls: HashMap<AutoStr, Rc<TypeDecl>>,
    enum_decls: HashMap<AutoStr, Rc<EnumDecl>>,
    // ...
}

impl TypeStore {
    pub fn lookup_type_decl(&self, name: &AutoStr) -> Option<Rc<TypeDecl>> {
        self.type_decls.get(name).cloned()  // Clones Rc, not TypeDecl
    }
}
```

### 2. Use `ClassTemplate` from `vm/generic_registry.rs`

Reuse existing generic infrastructure instead of custom `GenericTemplate`:
- Already tested and working
- Maintains compatibility with Codegen
- Supports field type substitution

### 3. Separate Storage for EnumDecl

Enums have different semantics from TypeDecl:
- No instantiation (no `Point{x:1}` syntax)
- Variants have associated values
- Pattern matching support needed

## Implementation Steps

### Step 1: Add EnumDecl to TypeStore

**Scope**: Small | **Risk**: Low

**Changes**:
1. Add `enum_decls: HashMap<AutoStr, Rc<EnumDecl>>` to `TypeStore`
2. Add `register_enum_decl()`, `lookup_enum_decl()`, `is_enum()`, `get_enum_variant_value()`
3. Update Parser to register enums in `type_store`
4. Update Codegen to lookup enum variants from `type_store`

**Expected Result**: `test_enum_decl_compiles` passes

### Step 2: Merge `infer/registry.rs` into `TypeStore`

**Scope**: Medium | **Risk**: Medium

**Changes**:
1. Move `ClassTemplate` generation to `TypeStore.register_type_decl()`
2. Update `InferenceContext` to query `type_store` instead of own `type_registry`
3. Mark `infer/registry.rs` as deprecated
4. Update all `lookup_type_decl()` callers

**Challenges**:
- Lifetime issues when returning references from `RwLock`
- Solution: Return `Rc<TypeDecl>` instead of `&TypeDecl`

### Step 3: Merge `type_registry.rs` into `TypeStore`

**Scope**: Small | **Risk**: Low

**Changes**:
1. Add `is_type()` and `get_type()` convenience methods to `TypeStore`
2. Update Parser to use `type_store.is_type()` instead of `type_registry`
3. Update `autovm_persistent.rs` to use `type_store`
4. Mark `type_registry.rs` as deprecated

### Step 4: Deprecate `Database.type_info_store`

**Scope**: Medium | **Risk**: Medium

**Changes**:
1. Add `type_store: Arc<RwLock<TypeStore>>` to Database
2. Update Indexer to register to `type_store`
3. Update consumers to use `type_store` directly
4. Mark `type_info_store` as deprecated
5. Provide backward-compatible `type_info_store()` view method

## Success Criteria

- ✅ `test_enum_decl_compiles` passes
- ✅ `test_combined_type_enum_spec_compiles` passes
- ✅ All existing tests pass (999+ tests)
- ✅ No duplicate type storage
- ✅ Single `TypeStore` API for all type queries
- ✅ REPL type persistence works

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Breaking existing code | Incremental steps, each step tested |
| Performance regression | `Rc` is cheap; benchmark if concerned |
| Missed call sites | Grep for all registry usages before removal |

## Files Changed

| File | Change Type |
|------|-------------|
| `types.rs` | Major enhancement |
| `parser.rs` | Update type registration |
| `codegen.rs` | Update type lookup |
| `infer/context.rs` | Remove local type_registry |
| `infer/registry.rs` | Deprecate, then remove |
| `type_registry.rs` | Deprecate, then remove |
| `database.rs` | Add type_store, deprecate type_info_store |
| `indexer.rs` | Register to type_store |
| `autovm_persistent.rs` | Use type_store |

## Timeline

- Step 1: ~1 hour (enum support)
- Step 2: ~2-3 hours (infer/registry merge)
- Step 3: ~1 hour (type_registry merge)
- Step 4: ~2 hours (Database migration)

**Total**: ~6-7 hours

## References

- [Plan 084: Unified Type Context](./084-unified-type-context.md)
- [Plan 087: AutoVM Generics](./087-autovm-generics-type-erasure-specialization.md)
- [Plan 064: Split Universe](./064-split-universe-compile-runtime.md)