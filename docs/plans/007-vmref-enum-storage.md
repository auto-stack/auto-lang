# Plan: Replace Box<dyn Any> with Enum-Based Storage for VM References

**Status:** Proposed
**Created:** 2025-01-18
**Priority:** HIGH - Critical bug blocking HashMap/HashSet functionality

## Problem

Current implementation uses `Box<dyn Any>` to store VM references (HashMap, HashSet, StringBuilder data). A mysterious TypeId mismatch bug occurs during storage:

- Created TypeId: `0xc3f10180ed89e7127de3c381a3066247` (correct)
- Stored TypeId: `0x5ba9901630e4e97a2ecd1d681e46344f` (wrong)

This causes all downcast operations to fail, making HashMap/HashSet operations non-functional.

The TypeId changes **between the call site and function entry**, suggesting:
- Compiler bug with LTO
- Undefined behavior in trait object passing
- Type confusion at ABI level

## Solution

Replace `Box<dyn Any>` storage with an enum-based approach that doesn't rely on TypeId or runtime downcasting.

## Implementation Plan

### Phase 1: Define Enum Type (5 min)
- [ ] Create `VmRefData` enum in `universe.rs` with variants:
  - `HashMap(HashMapData)`
  - `HashSet(HashSetData)`
  - `StringBuilder(StringBuilderData)`
- [ ] Move `HashMapData`, `HashSetData`, `StringBuilderData` types to `universe.rs` or keep in `collections.rs` and re-export

### Phase 2: Update Universe Storage (5 min)
- [ ] Change `vm_refs` type from `HashMap<usize, RefCell<Box<dyn Any>>>` to `HashMap<usize, RefCell<VmRefData>>`
- [ ] Update `add_vmref()` signature to take `VmRefData` instead of `Box<dyn Any>`
- [ ] Update `get_vmref_ref()` return type
- [ ] Remove `dyn Any` imports if no longer needed

### Phase 3: Update Collections Module (15 min)
- [ ] Update `hash_map_new()` to wrap data in `VmRefData::HashMap()`
- [ ] Update `hash_set_new()` to wrap data in `VmRefData::HashSet()`
- [ ] Replace all `downcast_ref::<HashMapData>()` with pattern matching on `VmRefData`
- [ ] Replace all `downcast_mut::<HashMapData>()` with pattern matching on `VmRefData`
- [ ] Do the same for HashSet functions

### Phase 4: Update StringBuilder (5 min)
- [ ] Update `string_builder_new()` to wrap data in `VmRefData::StringBuilder()`
- [ ] Update all StringBuilder methods to use pattern matching

### Phase 5: Remove Unsafe Code (2 min)
- [ ] Remove the unsafe workaround in `hash_map_insert_str()`
- [ ] Restore safe pattern matching throughout

### Phase 6: Test (5 min)
- [ ] Run `test_hashmap_oop_contains` - should pass
- [ ] Run `test_hashmap_oop_size` - should pass
- [ ] Run `test_hashmap_oop_remove` - should pass
- [ ] Run `test_hashset_oop_contains` - should pass
- [ ] Run all collection tests to ensure no regressions

### Phase 7: Cleanup (3 min)
- [ ] Remove debug logging from test files
- [ ] Remove test files created during debugging (`test_typeids.rs`, `test_find_typeid.rs`)
- [ ] Verify no compiler warnings

## Success Criteria

- ✅ All HashMap/HashSet tests pass
- ✅ No TypeId checks or downcasting in collections code
- ✅ No unsafe code (except where already needed elsewhere)
- ✅ Zero compilation warnings
- ✅ Code is cleaner and more maintainable

## Risks

- **Low Risk**: Enum approach is simpler and more type-safe than trait objects
- **Backward Compatibility**: No external API changes, only internal refactoring
- **Performance**: Slightly better (no vtable lookups, no downcasting)

## Alternatives Considered

1. **Continue debugging TypeId issue**: Rejected - appears to be compiler/LTO bug beyond our control
2. **Use unsafe pointer casting**: Rejected - caused segfault, unsafe
3. **Rebuild without LTO**: Tested - didn't fix the issue
4. **Duplicate type definitions**: Investigated - only one definition exists per type

## Notes

- This is a **pure refactoring** - no behavior changes from user perspective
- The enum approach is actually **more idiomatic Rust** than trait objects for this use case
- Pattern matching will be **faster** than runtime downcasting
- Code will be **more maintainable** with explicit type variants
