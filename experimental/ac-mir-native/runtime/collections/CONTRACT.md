# Native collections runtime: T-02 feasibility contract

The T-02 source/machine/resource first chain is implemented and verified; this
is not a claim of full PLAN-889 delivery. The private `collections_lower` consumes shared verified
MIR, emits native arithmetic/control/calls and direct container primitive calls,
then COFF/PE. The original17 source faces, owned combinations and typed clone
probes have actual native results. Product driver/cache/worker/recipe integration
is still pending; do not close formal M1 bindings based only on this private entry.

The private feasibility ABI is `mn.collections.runtime/2`, with generated call ABI
`mn.collections.cell/2`, Win64. A machine Cell is
16 bytes: scalar bits or a node address, then a 64-bit destructor-right flag.
Native generated arithmetic/control flow will operate directly on scalar bits;
the runtime only supplies buffer/record/list/active-payload/owner primitives.
It does not contain a MIR/ABC reader, interpreter, or AV dependency.

Nodes own headers plus list/record Cell arrays, an active variant payload, or
a buffer from the delivered text shim. References share a buffer without an
allocation; explicit clone produces independent buffers. The payload graph is
required to have been statically verified acyclic. Clone uses an allocated,
charged explicit work stack; destruction uses an intrusive work stack without
allocation. Registry membership precedes claimed node dereferences.

Generated stack frames register their actual Cell storage. The MIR emitter
allocates its slot storage under the runtime budget (`frame_open`), avoiding
an unchecked large automatic array on the native stack. The frame header is
32 bytes; its private count high bit records heap-owned slots. Low-level ABI
probes can still register bounded stack Cell arrays (`frame_enter`). Fatal exits sweep
the whole registered call chain before exit70, while return70 remains an
ordinary value. Runtime-only PE probes already distinguish those outcomes,
failure-before-commit, text clone failure, partial clone rollback, a deliberate
omitted release, and construction/clone/drop at depth4096 on a 64KiB **reserved**
native stack. This depth is a witness, not an admission limit.

The collections staticlib depends on the delivered text shim's **rlib** output
and embeds its functions and single panic handler. The shim still produces its
old staticlib separately. Its additive host hooks default to the original
process heap when not explicitly installed. Linking both whole
libraries is unnecessary. Do not permit duplicate symbols with linker flags.

Remaining delivery work, owned by T-03..06 and not closed by the first chain:

- The production/cache/worker use of the source-produced closure. The emitter re-verifies units and
  has an explicit instruction/type whitelist; its verified-whole-module split
  proves original L/V and supplemental paths but is not arbitrary typed support.
- Remaining source-produced closure controls beyond original17 and named
  supplementary witnesses, including all ownership/boundary paths. Current generic
  unsafe helper callers are verified generated code or explicit native probe
  code, not an exposed language FFI. Copyable records use immutable retained
  nodes; resource reads share without buffer allocation; explicit clone deep-copies.
- Formal compiler partition/worker samples and product integration. A cold whole
  frontend probe of the frozen345483-byte/116-function load has a release
  OS peak commit214061056B, despite its post-frontend live private23490560B.
  This rejected prototype remains a control. The bounded producer below resolves
  this first-case peak without shrinking the load; T-04 must select it, with no
  silent whole-module collections fallback.
- Linker/object metadata and formal16/24/8MiB compiler partition observations;
  the worker OS peak alone cannot prove each partition.
- Remaining helper deadline/cancellation integration and full mutation controls,
  runtime content/build/dependency and parent observation integration in the driver.
- All formal performance/resource measurements and formal driver integration.

Implemented layout candidate `mn.collections.layout/1` uses canonical BLAKE3
identities for nominal owner/name, ordered named fields/arms and transitive
element/payload shapes, independent of type-table indices/spans. Iterative
dependency ordering rejects cycles/bad references and handles4096 variants on
a64KiB host stack. Borrowed views share the physical value descriptor; closure
and function-signature identities additionally bind borrow/own modes.

Every generated resource use validates registry membership/kind/layout binding;
constructors bind their verified descriptor words (never dereference an input
descriptor pointer). Clone copies the binding; function entry/result and each
field-path step validate it. A legitimate unit returning a different nominal
record to an independently legitimate caller traps23 before consumption and
still releases every caller owner. This is additional defense; persisted
closure signature/type admission remains required at T-04 before execution.

The new codegen/profile/recipe candidates are `mn.collections.codegen/1`,
`mn.collections.source/1`, `ac/mir-native-collections-v1`. Old five-domain
identities stay unchanged. T-04 must bind these axes, layout closure and exact
runtime library content/build/dependencies in A09'/A10'; no current product
cache uses this entry yet. A later891 package receives this final delivered
contract; it does not acquire N writes while889 owns it.

Parent `collections_observe::decode` caps raw bytes, requires an exact single
known trap reason plus exit70 and complete balanced runtime/text/OS observations.
Return70 without a reason stays Return70. Forged prefixes, missing/duplicate/
truncated fields, unknown reason, non70 trap exit, over-budget and failed OS
observations reject. The existing raw stderr and process status remain evidence,
not an authenticated PASS protocol for arbitrary external programs.

The PE queries OS private baseline/end and historical peak process commit.
It reports `process_commit_delta=peak_commit-private_baseline`, separately from
arena/requested bytes and the compiler worker. The8MiB boundary witness returns
to private557056B after peak8966144B (delta8409088B); no missing/zero OS sample
passes. The native stack and image are included in the PE process observation.

Formal workload: preserve the original115-function, ≥256KiB scalar arithmetic
and loop counts (use equivalent loop/break on the collections source surface),
plus one owned List<Variant<Item>>/Str.clone/return helper and main consumption.
The frozen source has116 functions and345483 bytes, base result1786 from the
old closed-form mirror plus42. Only collections_helper changes for revision n,
yielding1786+n; main and old115 functions stay byte-identical across edits.
Source hash at n0:9fbf68139857bf473e291240df8ac48f6fc2bb493ffe4e9c2df8cc6830ba70de.
Minimum30 samples each fresh/hit/restart/body-edit/historical-return path,
512 distinct real churn revisions, P90 sorted one-based N-floor(N/10)≤2000ms,
P95/P99 reported. Release worker; record machine/tools/library hashes and all
failures. Start at parent request/edit enqueue; finish after result/trap/resource
validation. Runtime preparation/build/content checks, compile/link/run are inside
that window; unchanged helper reuse must be demonstrated, not rebuilt off-clock.
Compiler managed16/24/8MiB (total48), worker delta128, PE delta128 and guest8/
execution32MiB remain separate gates; cold results cannot be replaced by warm
baselines. Current single feasibility timings are not formal samples or PASS.

Bounded cold producer `collections_frontend::cold` parses/resolves/typechecks
once into the existing CheckedEnv, then uses the existing emit_function_body,
Collections function verification and shared per-function lowerer. Exactly one
real body is planned/bound at a time; other declarations are body-less scaffolds.
The whole-module credential still rejects such a document. A collection-specific
context identity binds the complete nominal declaration graph without depth
cutoffs; scalar/core context keys stay unchanged. Body-less scaffolds are not
new executable semantics. Current116-function release feasibility sample has
private baseline1372160B, sampled peak36524032B/delta35151872B, real PE1786,
and total wall2017ms including library preparation. This closes the cold peak
counterexample, not formal P90/partition gates. T-05 still must implement and
verify real edits/atomic candidates; cold per-body production alone is not an
incremental session. Units import only actual callees; recursive self calls use
a real self import, not the old invalid sentinel. An unrelated signature edit
leaves main's actual COFF bytes identical.

The runtime now installs an exclusive allocator/poll hook before its first
allocation. A VirtualAlloc arena reserves32MiB of address space and commits at
most32MiB of private pages. Allocator headers, alignment, free fragments, text
ledger capacity and buffers, nodes, arrays, heap slots and clone work all live
in these pages. Tail release decommits pages. A failed allocation cannot grow
commit beyond the limit. Arena live/commit/peak/invalid and text metadata are
reported separately from requested-byte counters. The32B automatic Frame
headers and registered stack Cell storage share the same32MiB ceiling with
actual arena pages; `registered_bytes` and `managed_commit_peak` report this
combined domain. Heap-cell frames reserve their header before allocation; failed
allocation releases the reservation. A real32MiB page request that fits requested
bytes but would exceed the combined domain is rejected before commit, while
32MiB-4096 pages with80B of real registered storage succeeds and releases to0.
Native code spills, stack slack and PE image remain in the separate OS process
observation. Runtime global state has one execution
thread; the small-stack probe transfers execution to its sole worker thread.

Hook installation refuses live text buffers or retained metadata from a prior
allocator. Idle metadata can be explicitly released through its current
allocator before takeover. Old consumers do not install hooks or change their
retention behavior. Native last-frame cleanup releases the idle ledger; zero
container and text balances additionally require arena live/commit/metadata0.

Clone polls at every node/child and each4096-byte text copy chunk. Cancellation
returns -26, preserving source and destination and dropping the partial clone.
List shift/growth work is admitted proportionally before mutation. This is a
private cooperative seam, not a claim that an externally killed PE runs cleanup;
parent timeout/crash remains infrastructure failure. Destruction never polls
or allocates, so a cancelled request can finish its release sweep.

Real PE probes cover initial/growing text-ledger allocation failure, physical
quota refusal with fragment reuse/coalescing, exact8MiB guest and65536 list
boundaries plus one, text/list clone cancellation rollback and allocator
takeover refusal. Existing partial-clone/deep-stack/caller-trap/leak controls
remain in place. These first-case mechanisms do not close the remaining formal
delivery work listed above.
