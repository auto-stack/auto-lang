# auto-a2r Collections/Variant HIR→Rust Backend (PLAN-904 AX-C)

Identity: recipe `ac-a2r/hir-collections-emit-v1` | fileset `auto.a2x.rust.collections.fileset/1` | cert `ace.a2r-hir.collections.cert/1` | source profile `auto.source.collections.draft` (HIR `collections-draft`) | store layers AXS3 (`a2904/axs-coll-sem/v1`) / AXR3 (`a2904/axr-coll-fn/v1`).

## Supported domain (frozen)

- Entry: zero-param free function with i32 result. Types: int/bool (copy), `Str` (restricted face: literals, `.len()`, clone), records without container fields, nominal `enum`/`?T`/`Result<T,E>` (single payload, acyclic, arm order = discriminant), `List<T>` with T ∈ {int, bool, Str, record, variant}; List is function-local (param/return/field/variant-payload positions and nested Lists are named rejections; `str`/`char` excluded).
- List ops: `new` (logical start capacity 4)/push/insert/get/remove/set/len/clear; LHS/index/RHS each evaluated once in source order; owned-element push/insert/set require a whole-local `.move` or a fresh construction (`source.move-required`).
- Owned-element reads are CHECKED VIEWS (`a2x_coll_list_borrow_elem`): `xs.get(i)` is only a field-projection root or `.len()` receiver (`source.view-escape`); `for x in xs` reads len once, copy-binds copy elements, projects owned elements; structural mutation of the iterated list is rejected (`source.iterate-mutate`), `set` is legal.
- `is` matching is exhaustive, no guards/wildcards, arms unique; wrong-arm/double-move are static rejections with a runtime discriminant defense (`trap:VariantBranch`).
- Single index trap `trap:ListIndex` for negative/≥len/empty-container get/remove/set, insert outside `[0,len]` — trapping BEFORE any buffer access; never a value-70, panic, or arbitrary nonzero.

## Rust representation (T-02 freeze)

| Auto | Rust |
|---|---|
| int/bool | i32/bool |
| Str | `A2xStr` (no Drop; freed ONLY by `a2x_str_free`) |
| record | `Rec<Name>` struct, `f_<field>` fields |
| List<T> | `A2xList<E>` = `{ buf: ManuallyDrop<Vec<E>> }`, no Drop; sole release path `a2x_coll_list_free` (elements reverse index order, buffer last) |
| Variant | nominal `A2xv<Name>` enum; guard `matches!` borrow; projection `a2x_coll_take_<V>_<Arm>` by value (Rust move = static single-move; helper checks discriminant) |

No implicit Clone/Drop; deep clone/free are generated explicit helpers; cleanup follows the verifier-owned `CollectionsBodyFacts` schedules (owned locals, per-exit LIFO frees, replace frees, per-fallible-node error-path frees, loop survivor sets) — the lowering never re-derives ownership.

## Billing / cancellation / fault (frozen table)

2 WU per list element slot (new base 4 + growth), 1 WU per 8 copied Str bytes, 1 WU per ctor, 4 WU per call frame, 1 WU per loop back-edge poll; default budget 2^24 (`A2X_RT_BUDGET` overrides). Cooperative cancellation at polling points only (`A2X_RT_CANCEL=1` → `trap:Cancelled`); exhaustion is `trap:Budget`. Charges precede the allocation/commit they pay for; `A2X_RT_FAULT_ALLOC=<n>` fails the n-th real allocation request BEFORE it happens. Cleanup paths are zero-charge, zero-poll, zero-allocation; list element release is reverse-index, buffer last. Trap names: 902 set + `ListIndex, VariantBranch, Budget, Cancelled`.

## Failure atomicity

Every mutation prechecks bounds/budget BEFORE committing; a failed push/insert/set frees its uncommitted owned input (helper-side) and leaves the old target intact; `set` frees the old element exactly once at commit; deep clone rolls back its partial prefix. Verified by live-counter fallback (nodes/str_bytes/backing → 0), not logs.

## Function-level increment (AXS3/AXR3)

Semantic keys add variant name + ordered arms + payload transit graph, list element types, callee signatures/modes (target-neutral); fragment keys add the collections runtime content hash. Unchanged functions reuse residents/store bytes with zero production. The collections WARM contract is `warm_edit_ranged`: base-bound full-source authorization (current sha binding, UTF-8 range covering exactly the target declaration, same-named single-declaration fragment) — stale base/half-UTF-8/wrong-target/whole-source ranges are named rejections (`a2x.stale-base`/`a2x.range`/`a2x.parse`); a fragment-only edit is a named boundary (`a2x.range`). The manifest binds sha256 of the REPLACED FULL SOURCE plus the canonical `execution_config` — never a fragment-only hash. Restart re-admits by pure store hits; read-only writes zero bytes; close rejects before any lookup.

## Certification

Explicit two-leg: the same-session ACTUAL file set and the case's HAND-WRITTEN std-only reference both really compile and run against the independent oracle; wrong oracle/reference REJECTED. `execution_config` (mode debug/release, budget, fault, events, cancel; canonical fixed-key JSON) is certificate KEY MATERIAL: collections certs require it, the older schemas must not carry it, and configs never cross-trust. Parent-side judging re-hashes the manifest, byte-compares all declared artifacts, rejects undeclared extras, and applies path-domain review (relative, no `..`/rooted forms, unique after ASCII-case-insensitive normalization). Daily emit is spawn-zero over the prebuilt worker.

## Budgets (§5.7)

Managed working set 48MiB along 902; store namespace ≤128MiB; certify scratch ≤512MiB; value data ≤8MiB; single list ≤65536 elements; managed backing ≤32MiB; process private peak − baseline ≤128MiB; compile ≤120s / run ≤30s / certify wall ≤360s; formal sampling fresh/hit/restart/body-edit × 30 samples, P90 ≤ 2000ms with correctness/structure/resource gates passing together; P50/95/99 and prepare/compile/certify timings listed separately. Deep nesting is handled by iterative work structures — the 3072-level/256KiB-stack witness proves the cleanup/clone/fault mechanism, not a language depth constant.

## Known boundaries

- Unit-arm qualified construction (`E.A` without payload in expression position) is not in the frozen surface; unit arms are matched but not constructed qualified.
- Call results are not fresh rvalues for owned pushes — bind to a local and `.move`.
- Fused `.len()` views over `List<Str>` preserve their own indices (resolve task-ordering fix, PLAN-904; counterexample `v02_fused_view_indices_preserved`).
