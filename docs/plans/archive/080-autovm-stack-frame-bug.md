# Plan 080: AutoVM Stack Frame Bug Fix

## Problem Discovery

**Date**: 2025-02-06
**Issue**: REPL variable values accumulating (5 → 6 → 7)
**Root Cause**: Stack and local variables share the same memory region

## Root Cause Analysis

### Current Stack Frame Layout

For main task (REPL), `bp = 0`:

```
Memory:
raw[0] = x (local variable 0, STORE_LOC_0 writes here)
raw[1] = (unused or stack)
raw[2] = (unused or stack)
...
```

When executing `x + 1`:

1. **Initial state**: `bp=0, sp=0, raw[0]=5` (variable x)
2. **LOAD_LOC_0**: Read `raw[0]=5`, push to `raw[sp]` = `raw[0]`, `sp=1`
   - `raw[0]` is now BOTH the variable x AND the stack top!
3. **CONST_I32 1**: Push to `raw[1]`, `sp=2`
4. **ADD**:
   - Pop `raw[1]=1`, `sp=1`
   - Pop `raw[0]=5`, `sp=0` ← **Overwrites variable x!**
   - Push `5+1=6` to `raw[0]`, `sp=1`
   - **Result**: `raw[0]=6` (x was modified from 5 to 6!)

### The Bug

**Local variables and stack share the same memory because bp=0 in main task!**

- Local variables: `bp+0`, `bp+1`, `bp+2`, ... (i.e., `raw[0]`, `raw[1]`, `raw[2]`, ...)
- Stack grows from `sp` upward: `raw[sp]`, `raw[sp+1]`, ...
- When `sp == bp == 0`, pushing to stack overwrites local variables!

### Standard Stack Frame Layout

According to [engine.rs:1200-1201](../crates/auto-lang/src/vm/engine.rs#L1200-L1201):

```
Stack layout: [..., old_closure_id, ret_ip, old_bp, args...]
                    bp-2          bp-1    bp
```

For a function call:
- `bp-2`: old_closure_id
- `bp-1`: ret_ip (return address)
- `bp`: old_bp (saved base pointer)
- `bp+1`: args (function arguments)
- `bp+num_args+1`: local variables

**Local variables should be at `bp + num_args + 1`, not at `bp+0`!**

## Proposed Solutions

### Option 1: Reserve Space for Local Variables (Quick Fix)

**Implementation**:
1. When starting main task, set `bp` to a non-zero value (e.g., 100)
2. This reserves `raw[0..100]` for system use
3. Local variables go to `raw[bp+1]`, `raw[bp+2]`, ... (i.e., `raw[101]`, `raw[102]`, ...)
4. Stack starts at `bp + num_locals` (e.g., `sp = 100 + num_locals`)

**Pros**:
- ✅ Simple change
- ✅ Works for REPL (main task)
- ✅ No changes to bytecode generation

**Cons**:
- ❌ Wastes memory (reserves fixed space)
- ❌ Doesn't fix the root cause (stack frame layout)
- ❌ May not work for nested function calls

### Option 2: Proper Stack Frame Layout (Correct Fix)

**Implementation**:
1. Modify codegen to track `num_locals` per function
2. Emit `ENTER` opcode at function start:
   ```rust
   ENTER num_locals:
     // Reserve space for local variables above bp
     task.sp += num_locals
   ```
3. Change `LOAD_LOC_N` to:
   ```rust
   LOAD_LOC_N:
     let val = task.ram.read_i32(task.bp + 1 + N)  // bp+1, bp+2, ...
     task.ram.push_i32(val)
   ```
4. Change `STORE_LOC_N` to:
   ```rust
   STORE_LOC_N:
     let val = task.ram.pop_i32()
     task.ram.write_i32(task.bp + 1 + N, val)  // bp+1, bp+2, ...
   ```

**Stack Layout After Fix**:
```
[... ret_ip, old_bp, arg0, arg1, ..., local0, local1, ..., stack...]
                     bp                           bp+num_args+1
                                                        ↑
                                                  local variables start here
```

**Pros**:
- ✅ Proper stack frame layout
- ✅ No memory waste
- ✅ Works for nested function calls
- ✅ Aligns with standard calling conventions

**Cons**:
- ❌ Requires codegen changes
- ❌ Requires bytecode changes
- ❌ Breaking change (need to recompile all code)

### Option 3: Hybrid Approach (Recommended for REPL)

**Implementation**:
1. For REPL (main task only), use Option 1 (set `bp=100`)
2. For function calls, use Option 2 (proper stack frame layout)
3. Add `ENTER` opcode for function entry

**Pros**:
- ✅ Works immediately for REPL
- ✅ Proper fix for function calls
- ✅ No breaking changes to existing code

**Cons**:
- ❌ More complex (two different layouts)
- ❌ Needs careful documentation

## Recommended Action

**Phase 1**: Implement Option 1 (Quick Fix for REPL)
- Set `bp=100` in main task initialization
- This immediately fixes the variable accumulation bug
- Allows REPL to be usable while proper fix is implemented

**Phase 2**: Implement Option 3 (Proper Fix)
- Add `ENTER` opcode
- Modify codegen to emit `ENTER` at function start
- Update `LOAD_LOC_N` and `STORE_LOC_N` to use `bp+1+N`
- Update function call prologue/epilogue

## Testing Plan

### Phase 1 Tests

```auto
AutoVM> let x = 5
AutoVM> x + 1
6
AutoVM> x + 1
6  ← Should return 6, not 7!
AutoVM> let y = x * 2
AutoVM> y
10
```

### Phase 2 Tests

```auto
// Function with local variables
fn test() int {
  let a = 5
  let b = 10
  return a + b
}

AutoVM> test()
15

// Nested function calls
fn outer(n int) int {
  let x = n + 1
  return inner(x)
}

fn inner(m int) int {
  let y = m * 2
  return y
}

AutoVM> outer(5)
12
```

## Implementation Checklist

### Phase 1: Quick Fix
- [ ] Modify `AutovmReplSession::new()` to set `bp=100`
- [ ] Add test for variable persistence
- [ ] Update documentation

### Phase 2: Proper Fix
- [ ] Design `ENTER` opcode
- [ ] Modify codegen to track `num_locals`
- [ ] Emit `ENTER` at function start
- [ ] Update `LOAD_LOC_N` to use `bp+1+N`
- [ ] Update `STORE_LOC_N` to use `bp+1+N`
- [ ] Update function call prologue/epilogue
- [ ] Add comprehensive tests
- [ ] Update Plan 069 (global variables)

## References

- [engine.rs:1200-1201](../crates/auto-lang/src/vm/engine.rs#L1200-L1201) - Stack frame layout
- [Plan 069](069-autovm-global-vars.md) - AutoVM global variables support
- [autovm_persistent.rs](../crates/auto-lang/src/autovm_persistent.rs) - REPL implementation
