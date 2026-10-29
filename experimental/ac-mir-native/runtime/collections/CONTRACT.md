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
old staticlib separately; its source/exports are unchanged. Linking both whole
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
- Full metadata accounting: the current `managed` counter charges node/array/
  registered-slot/clone-work **requested bytes** and text-buffer bytes, but
  does not include the text shim's retained ledger capacity, object/linker
  descriptor storage, heap overhead or observed process private bytes. Thus
  zero managed/node balance is a release observation for these allocations,
  not proof of the Plan's physical32MiB/128MiB gates. Keep those gates pending.
- Data-proportional work/cancellation polling, all requested boundary controls,
  runtime content/build/dependency binding and bounded parent trap admission.
- Performance workload/sample/timing freeze and all formal measurements.

The code has prechecks for guest8MiB, registered requested managed32MiB,
list65536 and injected allocation failures. These partial checks must not be
reported as the complete resource contract. Separate source/typed/runtime,
logical events/requested bytes and physical observations in every report.
