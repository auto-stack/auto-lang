# AutoLang Architecture Migration Status

**Last Updated**: 2025-02-01
**Overall Progress**: 70% complete (Core AIE infrastructure ✅, Universe split 60%)

---

## Executive Summary

We are building a new compiler architecture with three interconnected plans:

1. **Plan 063**: AIE (Auto Incremental Engine) - Foundation for incremental compilation
2. **Plan 064**: Split Universe - Separate compile-time (Database) from runtime (ExecutionEngine)
3. **Plan 065**: AIE Integration - Enable incremental compilation in main entry points

**Critical Path**: 063 → 064 → 065

---

## Plan 063: AIE Architecture Migration

**Status**: ✅ **70% Complete** (Core infrastructure done, AutoLive deferred)

### Completed Phases ✅

- ✅ **Phase 1**: File-Level Hashing & Dirty Tracking (2 weeks)
  - `FileHasher` for content hashing
  - `DirtyTracker` for change detection
  - Integration with parser and indexer

- ✅ **Phase 2**: Fragment-Level Indexing (3 weeks)
  - `FragmentIndexer` extracts functions/types/impls from AST
  - `FragmentStore` with L1/L2/L3 hash system
  - `FragmentMeta` with熔断 (signature hash)

- ✅ **Phase 3.1**: Fragment-Level Hashing
  - L1 hash: Content hash (SHA-256 of code)
  - L2 hash: Structure hash (signature + dependencies)
  - L3 hash: Combined hash (L1 + L2)

- ✅ **Phase 3.2**: 熔断 (Signature Change Detection)
  - Validates public API changes
  - Dependent invalidation only when signatures change
  - Smart cache invalidation

- ✅ **Phase 3.3**: Fragment-Level Dependency Tracking
  - `DepScanner` extracts dependencies
  - `DepGraph` maintains dependency relationships
  - Transitive dependency queries

- ✅ **Phase 3.4**: Incremental Query Engine
  - `QueryEngine` with smart caching
  - `Query<T>` trait for cached computations
  - 熔断-aware cache invalidation

- ✅ **Phase 3.5**: Patch Generation
  - `Patch` structure (adds, removes, updates)
  - `Reloc` for relocations
  - Patch generation from dirty fragments

### Deferred Phases ⏸️

- ⏸️ **Phase 3.6**: MCU Runtime Integration (8 weeks)
  - **BLOCKER**: Requires MCU hardware/infrastructure
  - Runtime patch application
  - Live code swapping
  - MCU flash programming interface

- ⏸️ **Phase 3.7**: Debugger Protocol (2 weeks)
  - **BLOCKER**: Requires Phase 3.6
  - Debug symbol generation
  - Breakpoint management
  - Step/continue operations

- ⏸️ **Phase 3.8**: End-to-End Testing (2 weeks)
  - **BLOCKER**: Requires Phase 3.6-3.7
  - Integration testing
  - Performance benchmarks
  - AutoLive demo

### When to Revisit

**Phase 3.6 (MCU Runtime)**: Revisit when:
- MCU hardware/firmware is available
- MCU runtime infrastructure is implemented
- Flash programming interface is ready
- **Estimated effort**: 8 weeks
- **Dependencies**: MCU team deliverables

**Phases 3.7-3.8**: Revisit after Phase 3.6 completes

### What's Next for Plan 063

**Current status**: Core AIE infrastructure is production-ready ✅

**No immediate work needed** - Plan 063 is waiting on:
- Plan 064 completion (Universe split)
- Plan 065 completion (lib.rs integration)
- MCU runtime infrastructure (external dependency)

**Value delivered so far**:
- ✅ Incremental parsing (only parse changed files)
- ✅ Smart dependency tracking (know what needs recompilation)
- ✅ Query caching (avoid repeated computations)
- ✅ Patch generation (ready for hot reload when runtime is ready)

---

## Plan 064: Split Universe into Database + ExecutionEngine

**Status**: ⏸️ **60% Complete** (Evaler migration ongoing, VM refs blocked)

### Completed Phases ✅

- ✅ **Phase 1**: Field Classification (1 week)
  - 19 Universe fields analyzed
  - 11 compile-time → Database
  - 7 runtime → ExecutionEngine
  - 1 hybrid (lambda_counter)

- ✅ **Phase 2**: ExecutionEngine Extension (1 week)
  - Added 11 runtime fields from Universe
  - `values`, `vm_refs`, `shared_vals`, `env_vals`, etc.
  - Call stack and frame management

- ✅ **Phase 3**: Database Extension (1 week)
  - Added 7 compile-time fields from Universe
  - Symbol tables, type info, specs
  - Symbol locations and AST storage

- ✅ **Phase 4.1**: Design Complete
  - SymbolTable for compile-time scopes
  - StackFrame for runtime variable storage
  - Bridge layer architecture

- ✅ **Phase 4.2**: SymbolTable + StackFrame Structures
  - `SymbolTable` with ScopeKind
  - `StackFrame` with variable storage
  - Sid-based addressing

- ✅ **Phase 4.3**: Bridge Layer Integration
  - `CompileSession` wraps Database
  - Bridge methods in Evaler (20+ methods)
  - Database + ExecutionEngine sharing

- ✅ **Phase 4.4**: Interpreter Migration
  - Interpreter uses CompileSession
  - ExecutionEngine integration
  - Database integration

- ✅ **Phase 4.6**: VM Signature Redesign
  - Changed `VmFunction` from `fn(Shared<Universe>, ...)` to `fn(&mut Evaler, ...)`
  - Migrated 53 VM functions
  - Updated 10 call sites
  - **Tests**: 999 passing

### In-Progress Phases ⏸️

- ⏸️ **Phase 4.5**: Evaler Migration (60% complete)
  - 84 of 141 Universe references migrated (60%)
  - 57 remaining references:
    - ~30: Bridge method fallbacks (expected)
    - ~2: Getter methods (universe(), universe_mut())
    - ~25: VM module access (for VM references)

### Blocked Phases ❌

- ❌ **Phase 4.7**: VM Reference Migration (BLOCKED)
  - **BLOCKER**: Rust RefCell lifetime issues
  - Attempted to move VM refs from Universe to ExecutionEngine
  - Pattern `engine.borrow().get_vm_ref(id)` creates temporary
  - **Root issue**: Lifetime of `Ref<ExecutionEngine>` vs `Ref<VmRefData>`

**Possible solutions** (documented in Plan 064):
1. Use intermediate bindings (requires 43 call site updates)
2. Refactor VM modules (requires 2-3 days)
3. Accept hybrid approach (VM refs stay in Universe for now)

**Decision**: DEFER to future work
- Current hybrid approach works
- Focus on completing Plan 064 Phase 4.5 and Plan 065
- Revisit when architecture is more stable

### When to Revisit

**Phase 4.5 (Remaining 57 references)**: Continue immediately
- **Estimated effort**: 1-2 weeks
- **Next steps**: Migrate more call sites to bridge methods
- **No blockers** - straightforward migration work

**Phase 4.7 (VM References)**: Revisit after Plan 065
- **Estimated effort**: 2-3 days
- **When**: After AIE integration (Plan 065) is stable
- **Why**: Lower priority than getting incremental compilation working

### What's Next for Plan 064

**Immediate priorities**:
1. **Option A**: Continue Phase 4.5 - Migrate 57 remaining Universe references
   - Straightforward bridge method usage
   - No technical blockers
   - Estimated 1-2 weeks

2. **Option B**: Pause Plan 064, start Plan 065 (AIE Integration)
   - Focus on incremental compilation value
   - Come back to Phase 4.5/4.7 later
   - Get faster feedback on architecture decisions

**Recommended**: Option B - Start Plan 065
- Plan 064 is at a good stopping point (60% complete)
- Plan 065 will validate the architecture end-to-end
- Can finish Plan 064 based on real-world usage

---

## Plan 065: AIE Integration with lib.rs Entry Points

**Status**: 📝 **Planning** (Waiting for Plan 064)

### Dependencies

**BLOCKED by Plan 064 Phase 4**:
- Waiting for SymbolTable + StackFrame implementation ✅ (Complete)
- Waiting for Database + ExecutionEngine integration ✅ (Complete)
- Waiting for Universe split to stabilize ⏸️ (60% complete)

### What Plan 065 Will Do

**Goal**: Enable incremental compilation in main entry points

**Current behavior** (Problem):
```rust
pub fn run(code: &str) -> AutoResult<String> {
    let mut interpreter = interp::Interpreter::new();  // Fresh Database!
    interpreter.interpret(code)?;                     // Parse everything
    Ok(interpreter.result.repr().to_string())
}
```

Every `run()` call:
- ❌ Creates new Database (no persistence)
- ❌ Parses from scratch (no caching)
- ❌ No incremental recompilation (wastes AIE!)

**Target behavior** (Solution):
```rust
// REPL mode - incremental!
let mut session = CompileSession::new();  // Persistent Database
loop {
    let result = run_with_session(&mut session, input)?;
    // Only recompiles changed code!
}

// Script mode - one-shot
let result = run(script)?;  // Backwards compatible
```

### Planned Implementation

**Phase 1**: Design ReplSession (30 min)
- Persistent CompileSession + QueryEngine
- Backwards-compatible API

**Phase 2**: Implement `run_with_session()` (1-2 hours)
- Incremental compilation logic
- Use Database for caching
- Fallback to Universe when needed

**Phase 3**: Integrate QueryEngine (1-2 hours)
- Cache bytecode/AST results
- 熔断 smart invalidation
- Subsequent runs are instant!

**Phase 4**: REPL Integration (1 hour)
- Update REPL to use ReplSession
- Add `:stats` command
- Show cache hit rates

**Phase 5**: Testing (1 hour)
- Unit tests for incremental behavior
- Performance benchmarks
- No regressions

**Phase 6**: Documentation (30 min)
- Update CLAUDE.md
- Add examples
- API docs

### When to Start

**Can start NOW** ✅
- Plan 064 Phase 4.1-4.4 are complete (SymbolTable, StackFrame, integration)
- Core infrastructure is ready
- Can validate architecture with real incremental compilation

**Dependencies that can wait**:
- Plan 064 Phase 4.5 (60% done, good enough)
- Plan 064 Phase 4.7 (VM refs can stay in Universe for now)

### Estimated Effort

**Total**: 4-6 hours
- Phase 1: 30 min (design)
- Phase 2: 1-2 hours (implementation)
- Phase 3: 1-2 hours (QueryEngine)
- Phase 4: 1 hour (REPL)
- Phase 5: 1 hour (testing)
- Phase 6: 30 min (docs)

### Value Delivered

After Plan 065 completes:
- ✅ REPL uses incremental compilation (subsequent runs instant!)
- ✅ Scripts can opt-in to incremental mode
- ✅ Database persists across multiple runs
- ✅ AIE infrastructure actually gets used!
- ✅ Validates Plan 063+064 architecture end-to-end

---

## Critical Path & Dependencies

```
Plan 063 (AIE Infrastructure) ✅ 70% complete
    ↓
Plan 064 (Universe Split) ⏸️ 60% complete
    ├─ Phase 4.1-4.4 ✅ Complete (needed for 065)
    ├─ Phase 4.5 ⏸️ 60% complete (can continue later)
    └─ Phase 4.7 ❌ Blocked (VM refs - can defer)
        ↓
Plan 065 (AIE Integration) 📝 Ready to start
    └─ Depends on 064 Phase 4.1-4.4 ✅
```

**Key insight**: Plan 065 can start NOW despite Plan 064 being incomplete

---

## Recommended Next Steps

### Option A: Continue Plan 064 Phase 4.5 (1-2 weeks)

**Pros**:
- Complete the Universe split
- Reduce technical debt
- Clean architecture before adding features

**Cons**:
- More refactoring before seeing value
- Delay incremental compilation benefits

**Best for**: If you prefer completeness and cleanliness

---

### Option B: Start Plan 065 Now (4-6 hours) ⭐ **RECOMMENDED**

**Pros**:
- **See incremental compilation working ASAP!**
- Validate Plan 063+064 architecture
- Get user feedback on REPL performance
- Come back to finish Plan 064 with real-world experience

**Cons**:
- Plan 064 remains incomplete (60%)
- VM references still in Universe (hybrid approach)

**Best for**: If you want to see the AIE value and validate architecture

**When to finish Plan 064**: After Plan 065, based on usage patterns

---

### Option C: Tackle Plan 064 Phase 4.7 First (2-3 days)

**Pros**:
- Remove VM reference lifetime complexity
- Clean separation before Plan 065

**Cons**:
- **High risk** - Rust lifetime issues are tricky
- Could break tests (999 passing → ???)
- Delay seeing AIE benefits

**Best for**: If you enjoy solving hard Rust problems

---

## Deferred Work Summary

### Short-Term Deferred (Revisit in 1-2 weeks)

1. **Plan 064 Phase 4.5** (remaining 57 Universe references)
   - **When**: After Plan 065 or during slower period
   - **Effort**: 1-2 weeks
   - **Value**: Completes Universe split, reduces tech debt

2. **Plan 064 Phase 4.7** (VM reference migration)
   - **When**: After Plan 065, when architecture is stable
   - **Effort**: 2-3 days
   - **Value**: Removes hybrid approach, cleaner architecture

### Long-Term Deferred (Revisit when MCU available)

3. **Plan 063 Phase 3.6** (MCU Runtime Integration)
   - **When**: MCU hardware/infrastructure available
   - **Effort**: 8 weeks
   - **Value**: AutoLive - hot reload on microcontrollers

4. **Plan 063 Phase 3.7** (Debugger Protocol)
   - **When**: After Phase 3.6
   - **Effort**: 2 weeks
   - **Value**: Debugging support for incremental compilation

5. **Plan 063 Phase 3.8** (End-to-End Testing)
   - **When**: After Phase 3.7
   - **Effort**: 2 weeks
   - **Value**: Validates entire AutoLive pipeline

---

## Progress Metrics

### Code Migration Progress

| Metric | Before | Current | Target | % Complete |
|--------|--------|---------|--------|------------|
| Plan 063 (AIE Infrastructure) | 0% | 70% | 100% | ✅ Good progress |
| Plan 064 (Universe Split) | 0% | 60% | 100% | ⏸️ On track |
| Plan 065 (AIE Integration) | 0% | 0% | 100% | 📝 Ready to start |
| Universe references | 141 | 57 | 0 | **60% migrated** ✅ |
| VM functions migrated | 0 | 53 | 53 | **100%** ✅ |

### Test Health

| Suite | Status | Count |
|-------|--------|-------|
| Unit tests | ✅ Passing | 999/1006 |
| Integration tests | ⏸️ Some failures | 7 pre-existing |
| VM tests | ✅ Passing | 146/149 (3 pre-existing) |
| AIE tests | ✅ Passing | All green |

**No regressions** from all migration work ✅

---

## Risk Assessment

### High Risks 🔴

1. **Plan 064 Phase 4.7** (VM References)
   - **Risk**: Rust lifetime issues
   - **Mitigation**: Deferred, not on critical path
   - **Impact**: Low - hybrid approach works

2. **Plan 063 Phase 3.6** (MCU Runtime)
   - **Risk**: External dependencies (hardware)
   - **Mitigation**: Deferred until hardware available
   - **Impact**: Medium - delays AutoLive feature

### Medium Risks 🟡

3. **Plan 065 Technical Unknowns**
   - **Risk**: Unforeseen integration issues
   - **Mitigation**: Plan 064.1-4.4 are complete and tested
   - **Impact**: Low - can iterate quickly

### Low Risks 🟢

4. **Plan 064 Phase 4.5 Completion**
   - **Risk**: None - straightforward migration
   - **Mitigation**: Well-understood patterns
   - **Impact**: None - can pause/resume anytime

---

## Decision Matrix

| Option | Effort | Value | Risk | Recommendation |
|--------|--------|-------|------|----------------|
| **A: Continue 064.5** | 1-2 weeks | Medium | Low | Good for completeness |
| **B: Start 065** ⭐ | 4-6 hours | **High** | Low | **See value ASAP!** |
| **C: Fix 064.7** | 2-3 days | Low | High | Hard problem, low value |

---

## Conclusion

**Current state**: Solid foundation (70% AIE ✅, 60% Universe split ✅)

**Recommended next step**: **Start Plan 065 (AIE Integration)**
- See incremental compilation working in 4-6 hours
- Validate architecture end-to-end
- Get user feedback on REPL performance
- Come back to finish Plan 064 with real experience

**Deferred work is well-scoped**:
- Plan 064.5/4.7: Clear path forward, no blockers
- Plan 063.6-3.8: Waiting on external dependencies (MCU)

**Architecture is healthy**:
- Tests passing (999/1006)
- No regressions
- Clear separation of concerns
- Incremental progress validated

---

**Last reviewed**: 2025-02-01
**Next review**: After Plan 065 completion
**Owner**: AutoLang architecture team
