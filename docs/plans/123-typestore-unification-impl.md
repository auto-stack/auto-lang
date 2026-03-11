# TypeStore Unification Implementation Plan

> **Status**: ✅ **COMPLETE** (2026-03-12)

**Goal:** Consolidate all type registries into TypeStore as the single source of truth, fixing enum type reference in codegen.

**Architecture:** Add EnumDecl to TypeStore, then merge infer/registry.rs, type_registry.rs, and Database.type_info_store into TypeStore. Use Rc<T> for shared immutable references.

**Tech Stack:** Rust, Arc<RwLock<...>>, Rc<T>, existing TypeStore in types.rs

---

## Completion Summary

| Task | Description | Status | Commit |
|------|-------------|--------|--------|
| 1 | Add EnumDecl to TypeStore | ✅ | `45d5d02` |
| 2 | Register enums in Parser | ✅ | `7790f25` |
| 3 | Handle enum variant access in Codegen | ✅ | `d3cd33b` |
| 4 | Enable tests | ✅ | `f846e89` |
| 5 | Add is_type helper | ✅ | `b50c3fb` |
| 6 | Use Rc<TypeDecl> | ✅ | `860c399` |
| 7 | Deprecate type_registry.rs | ✅ | `b9778ad` |
| 8 | Deprecate infer/registry.rs | ✅ | `b9778ad` |
| 9 | Verify all tests | ✅ | `b2f7e60` |

### Tests Fixed
- `test_enum_decl_compiles` - Now passes
- `test_combined_type_enum_spec_compiles` - Now passes
- `test_int_enums` - Now passes

---

## Task 1: Add EnumDecl Storage to TypeStore

**Files:**
- Modify: `crates/auto-lang/src/types.rs:132-147` (TypeStore struct)
- Modify: `crates/auto-lang/src/types.rs:149-250` (TypeStore impl)
- Test: `crates/auto-lang/src/types.rs` (inline tests)

**Step 1: Add enum_decls field to TypeStore struct**

In `crates/auto-lang/src/types.rs`, modify the TypeStore struct (around line 132):

```rust
use std::rc::Rc;
use crate::ast::EnumDecl;

#[derive(Debug, Clone)]
pub struct TypeStore {
    /// 类型声明：类型名 -> 完整的类型声明
    type_decls: HashMap<AutoStr, TypeDecl>,

    /// Enum 声明：enum 名 -> EnumDecl (NEW)
    enum_decls: HashMap<AutoStr, Rc<EnumDecl>>,

    /// 函数声明：函数名 -> 函数声明
    fn_decls: HashMap<Name, Fn>,

    /// Spec 声明：spec 名 -> spec 声明
    spec_decls: HashMap<AutoStr, SpecDecl>,

    /// 泛型模板：类型名 -> 泛型模板（用于类型参数替换）
    generic_templates: HashMap<String, GenericTemplate>,

    /// 类型别名：别名 -> 目标类型名（Plan 090）
    type_aliases: HashMap<AutoStr, AutoStr>,
}
```

**Step 2: Update TypeStore::new() to initialize enum_decls**

```rust
impl TypeStore {
    pub fn new() -> Self {
        Self {
            type_decls: HashMap::new(),
            enum_decls: HashMap::new(),  // NEW
            fn_decls: HashMap::new(),
            spec_decls: HashMap::new(),
            generic_templates: HashMap::new(),
            type_aliases: HashMap::new(),
        }
    }
}
```

**Step 3: Add enum registration and lookup methods**

Add these methods to TypeStore impl (after existing methods):

```rust
    /// 注册 Enum 声明
    pub fn register_enum_decl(&mut self, decl: EnumDecl) {
        let name = decl.name.clone();
        self.enum_decls.insert(name, Rc::new(decl));
    }

    /// 查找 Enum 声明
    pub fn lookup_enum_decl(&self, name: &AutoStr) -> Option<Rc<EnumDecl>> {
        self.enum_decls.get(name).cloned()
    }

    /// 查找 Enum 声明（字符串参数）
    pub fn lookup_enum_decl_str(&self, name: &str) -> Option<Rc<EnumDecl>> {
        self.enum_decls.get(&AutoStr::from(name)).cloned()
    }

    /// 检查名称是否为 Enum 类型
    pub fn is_enum(&self, name: &str) -> bool {
        self.enum_decls.contains_key(&AutoStr::from(name))
    }

    /// 获取 Enum 变体的值
    pub fn get_enum_variant_value(&self, enum_name: &str, variant_name: &str) -> Option<i32> {
        self.enum_decls.get(&AutoStr::from(enum_name))
            .and_then(|decl| decl.items.iter()
                .find(|item| item.name.as_ref() == variant_name)
                .map(|item| item.value))
    }

    /// 统一的类型检查（包含 type、enum、spec）
    pub fn is_type(&self, name: &str) -> bool {
        let key = AutoStr::from(name);
        self.type_decls.contains_key(&key)
            || self.enum_decls.contains_key(&key)
            || self.spec_decls.contains_key(&key)
    }
```

**Step 4: Add import for EnumDecl**

At top of `types.rs`, add:

```rust
use crate::ast::EnumDecl;
use std::rc::Rc;
```

**Step 5: Run tests to verify compilation**

Run: `cargo test -p auto-lang types:: --no-run`
Expected: Compiles successfully

**Step 6: Commit**

```bash
git add crates/auto-lang/src/types.rs
git commit -m "feat(types): add EnumDecl storage to TypeStore

- Add enum_decls HashMap with Rc<EnumDecl>
- Add register_enum_decl, lookup_enum_decl methods
- Add get_enum_variant_value for enum member lookup
- Add unified is_type method"
```

---

## Task 2: Register Enums in Parser's TypeStore

**Files:**
- Modify: `crates/auto-lang/src/parser.rs:2957-2959` (enum parsing)

**Step 1: Register enum in type_store when parsing**

In `crates/auto-lang/src/parser.rs`, find the `parse_enum` method (around line 2957). After `self.define()`, add registration to type_store:

```rust
    fn parse_enum(&mut self) -> AutoResult<Stmt> {
        // ... existing parsing code ...

        self.expect(TokenKind::RBrace)?;
        // make enum ast node
        let enum_decl = EnumDecl { name, items };
        self.define(enum_decl.name.as_str(), Meta::Enum(enum_decl.clone()));

        // NEW: Register enum in type_store
        self.type_store.write().unwrap().register_enum_decl(enum_decl.clone());

        Ok(Stmt::EnumDecl(enum_decl))
    }
```

**Step 2: Verify compilation**

Run: `cargo build -p auto-lang`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add crates/auto-lang/src/parser.rs
git commit -m "feat(parser): register EnumDecl in TypeStore when parsing"
```

---

## Task 3: Handle Enum Variant Access in Codegen

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs:1973-1992` (Expr::Dot handling)

**Step 1: Add enum variant handling in compile_expr for Expr::Dot**

In `crates/auto-lang/src/vm/codegen.rs`, find the `Expr::Dot(obj, field)` case (around line 1973). Add enum handling BEFORE the `.type` check:

```rust
            // Plan 073: Dot expression field access (obj.field)
            // Plan 087 Phase 2: Support generic instance field access
            Expr::Dot(obj, field) => {
                // NEW: Check if this is enum variant access (e.g., Color.Red)
                if let Expr::Ident(type_name) = obj.as_ref() {
                    if let Some(value) = self.type_store.read().unwrap()
                        .get_enum_variant_value(type_name.as_ref(), field.as_ref())
                    {
                        // Enum variant access - emit the variant value as integer
                        self.emit(OpCode::PUSH_INT);
                        self.code.extend_from_slice(&value.to_le_bytes());
                        return Ok(());
                    }
                }

                // Check if this is the .type property - returns type name as string
                if field.as_str() == "type" {
                    // ... existing code ...
```

**Step 2: Verify type_store is accessible in Codegen**

Check that Codegen has `type_store` field. If not, add it:

```rust
pub struct Codegen {
    // ... existing fields ...
    pub type_store: Arc<RwLock<types::TypeStore>>,
}
```

If needed, also add the import and initialization.

**Step 3: Run the failing test**

Run: `cargo test -p auto-lang test_enum_decl_compiles -- --ignored`
Expected: Test PASSES

**Step 4: Commit**

```bash
git add crates/auto-lang/src/vm/codegen.rs
git commit -m "feat(codegen): handle enum variant access (Color.Red)

- Check if Dot expr is enum variant access
- Emit variant value as PUSH_INT if matched"
```

---

## Task 4: Remove #[ignore] from Test

**Files:**
- Modify: `crates/auto-lang/src/vm_types_tests.rs:1627` (test_enum_decl_compiles)

**Step 1: Remove ignore attribute**

Change:
```rust
#[test]
#[ignore = "enum type reference not fully implemented in codegen"]
fn test_enum_decl_compiles() {
```

To:
```rust
#[test]
fn test_enum_decl_compiles() {
```

**Step 2: Run test to verify it passes**

Run: `cargo test -p auto-lang test_enum_decl_compiles -v`
Expected: test passes

**Step 3: Also enable the combined test**

Remove `#[ignore]` from `test_combined_type_enum_spec_compiles` as well (line 1681).

**Step 4: Run all vm_types_tests**

Run: `cargo test -p auto-lang -- vm_types_tests`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/auto-lang/src/vm_types_tests.rs
git commit -m "test: enable enum type reference tests (now passing)"
```

---

## Task 5: Add is_type Method to Codegen for Type Lookup

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs` (add is_type helper)

**Step 1: Add is_type helper method to Codegen**

Add this method to Codegen impl (near other lookup methods):

```rust
    /// Check if a name is a registered type (type, enum, or spec)
    pub fn is_type(&self, name: &str) -> bool {
        self.type_store.read().unwrap().is_type(name)
    }
```

**Step 2: Run tests**

Run: `cargo test -p auto-lang`
Expected: All tests pass

**Step 3: Commit**

```bash
git add crates/auto-lang/src/vm/codegen.rs
git commit -m "feat(codegen): add is_type helper using TypeStore"
```

---

## Task 6: Update TypeStore to Use Rc<TypeDecl> (Future-Proofing)

**Files:**
- Modify: `crates/auto-lang/src/types.rs:134` (type_decls field)
- Modify: All type_decls usages

**Step 1: Change type_decls to use Rc<TypeDecl>**

```rust
pub struct TypeStore {
    type_decls: HashMap<AutoStr, Rc<TypeDecl>>,  // Changed from TypeDecl to Rc<TypeDecl>
    // ...
}
```

**Step 2: Update register_type_decl**

```rust
pub fn register_type_decl(&mut self, decl: &TypeDecl) {
    let name = decl.name.clone();
    self.type_decls.insert(name, Rc::new(decl.clone()));
    // ... rest unchanged
}
```

**Step 3: Update lookup_type_decl to return Rc**

```rust
pub fn lookup_type_decl(&self, name: &AutoStr) -> Option<Rc<TypeDecl>> {
    self.type_decls.get(name).cloned()
}

pub fn lookup_type_decl_str(&self, name: &str) -> Option<Rc<TypeDecl>> {
    self.type_decls.get(&AutoStr::from(name)).cloned()
}
```

**Step 4: Run tests**

Run: `cargo test -p auto-lang`
Expected: All tests pass (may need to fix call sites)

**Step 5: Commit**

```bash
git add crates/auto-lang/src/types.rs
git commit -m "refactor(types): use Rc<TypeDecl> for shared ownership"
```

---

## Task 7: Deprecate type_registry.rs (Documentation)

**Files:**
- Modify: `crates/auto-lang/src/type_registry.rs:1` (add deprecation notice)

**Step 1: Add deprecation notice**

Add at top of file:

```rust
//! **DEPRECATED**: This module is deprecated.
//!
//! Use `types::TypeStore` instead, which is the single source of truth
//! for type information. This module will be removed in a future version.
//!
//! Migration guide:
//! - `TypeRegistry::is_type(name)` → `TypeStore::is_type(name)`
//! - `TypeRegistry::get_type(name)` → `TypeStore::get_type(name)` (when implemented)
//! - `TypeRegistry::register_type(name, ty)` → Use `TypeStore::register_type_decl()` or `register_enum_decl()`
```

**Step 2: Commit**

```bash
git add crates/auto-lang/src/type_registry.rs
git commit -m "docs: deprecate type_registry.rs in favor of TypeStore"
```

---

## Task 8: Deprecate infer/registry.rs (Documentation)

**Files:**
- Modify: `crates/auto-lang/src/infer/registry.rs:1` (add deprecation notice)

**Step 1: Add deprecation notice**

Add at top of file:

```rust
//! **DEPRECATED**: This module is deprecated.
//!
//! Use `types::TypeStore` instead, which is the single source of truth
//! for type information. This module will be removed in a future version.
//!
//! Migration guide:
//! - `TypeRegistry::register_type_decl()` → `TypeStore::register_type_decl()`
//! - `TypeRegistry::lookup_type_decl()` → `TypeStore::lookup_type_decl()`
//! - `TypeRegistry::get_template()` → `TypeStore::get_class_template()` (when implemented)
```

**Step 2: Commit**

```bash
git add crates/auto-lang/src/infer/registry.rs
git commit -m "docs: deprecate infer/registry.rs in favor of TypeStore"
```

---

## Task 9: Run Full Test Suite

**Files:**
- None (verification only)

**Step 1: Run all tests**

Run: `cargo test -p auto-lang`
Expected: All tests pass

**Step 2: Run with specific focus on type-related tests**

Run: `cargo test -p auto-lang -- types`
Expected: All type tests pass

**Step 3: Final commit if any fixes needed**

```bash
git add -A
git commit -m "fix: ensure all tests pass after TypeStore unification"
```

---

## Summary

| Task | Description | Risk |
|------|-------------|------|
| 1 | Add EnumDecl to TypeStore | Low |
| 2 | Register enums in Parser | Low |
| 3 | Handle enum variant access in Codegen | Medium |
| 4 | Enable tests | Low |
| 5 | Add is_type helper | Low |
| 6 | Use Rc<TypeDecl> | Medium |
| 7 | Deprecate type_registry.rs | Low |
| 8 | Deprecate infer/registry.rs | Low |
| 9 | Verify all tests | Low |

**Total Estimated Time:** 2-3 hours