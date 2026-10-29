# Native collections runtime: T-02 draft

This is a **T-02 feasibility implementation**, not the frozen full PLAN-889
contract. The private `collections_lower` entry now consumes shared verified
MIR, emits native arithmetic/control/calls and direct container primitive calls,
then COFF/PE. The original17 source faces, owned combinations and typed clone
probes have actual native results. Product driver/cache/worker/recipe integration
is still pending; do not close formal M1 bindings based only on this private entry.

The private draft ABI is `mn.collections.runtime/1`, Win64. A machine Cell is
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

Open before T-02 can pass or the layout/ABI can be frozen:

- Complete source-produced closure/signature and nominal descriptor admission,
  and its production/cache/worker use. The new emitter re-verifies units and
  has an explicit instruction/type whitelist; its verified-whole-module split
  proves original L/V and supplemental paths but is not arbitrary typed support.
- Nominal type/layout descriptor binding and the complete source-produced
  instruction/signature closure, including copyable record semantics and
  borrowed projection lifetime across all supported paths. Current generic
  unsafe helper callers are verified generated code or explicit native probe
  code, not an exposed language FFI. Copyable records use immutable retained
  nodes; resource reads share without buffer allocation; explicit clone deep-copies.
- Process private peak/delta, object/linker descriptor storage and compiler
  partitions still need their separate observations; the bounded runtime arena
  below is not evidence of the whole-process128MiB gate.
- Remaining helper deadline/cancellation integration and boundary controls,
  runtime content/build/dependency binding and bounded parent trap admission.
- Performance workload/sample/timing freeze and all formal measurements.

The runtime now installs an exclusive allocator/poll hook before its first
allocation. A VirtualAlloc arena reserves32MiB of address space and commits at
most32MiB of private pages. Allocator headers, alignment, free fragments, text
ledger capacity and buffers, nodes, arrays, heap slots and clone work all live
in these pages. Tail release decommits pages. A failed allocation cannot grow
commit beyond the limit. Arena live/commit/peak/invalid and text metadata are
reported separately from requested-byte counters. The32B automatic Frame
headers, native stack and PE image are outside that arena; they remain part of
the required whole-process observation. Runtime global state has one execution
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
remain in place. Keep full T-02 pending until the other open items above close.
