# Native collections runtime and product contract (PLAN-889)

The implementation consumes the delivered864 collections source domain through
one shared source/HIR/function verifier and `auto-mir-lower` collections MIR.
`collections_lower` emits real scalar/control/call machine code and direct
container primitive calls, COFF, then PE. The runtime contains no MIR/ABC
reader, opcode dispatcher, interpreter or AV dependency. Final Plan delivery
still requires revision-bound review and merge; measurements live in the Plan.

## Domain and identities

Explicit selector: `--source-profile collections`; dedicated SDK recipe
`ac/mir-native-collections-v1`, process `mn-collections-worker`. The old five
native domains, recipe/default, exact receipt shape and legacy rejection remain.
Entry is zero-argument i32/bool. Source includes local List<int/bool/Str/acyclic
record/Variant>, Option/Result/finite enums, acyclic nested variants, owned
payloads/results, read-only projections, for-in and Str.clone. Source List.clone/
Variant.clone remain rejected; independent typed Clone witnesses do not widen
source methods. List parameters/results/record fields, nested lists, Variant
parameters/record fields, wide typed entry, closures/Dyn/Unit entry stay excluded.
AV profile4/opcodes/wire and runtime semantics are unchanged.

The Win64 ABI is `mn.collections.cell/2`: Cell16B (scalar/node address plus
64-bit destructor-right), Frame32B. Runtime identity `mn.collections.runtime/2`,
layout `mn.collections.layout/1`, codegen `mn.collections.codegen/1`, source
`mn.collections.source/1`. Canonical BLAKE3 nominal descriptors bind owners,
ordered fields/arms and transitive payload shapes, independently of table index
or source span. Signatures bind borrow/own modes. Every generated resource use
checks registry membership, kind and descriptor before consuming addresses.

## Ownership, physical resources and exits

Nodes own headers, list/record Cell arrays, active payloads or text-shim buffers.
View reads share without allocation; explicit clone has independent buffers.
Push/insert/set/switch prepare before committing, preserving the prior owner and
uncommitted input responsibility on failure. Clone uses a charged explicit work
stack; destruction uses an intrusive, nonallocating, nonpolling stack. The4096
small-stack construction/clone/drop test is a witness, never a language cutoff.

Generated frames allocate actual Cell storage via frame_open and register the
32B header; bounded low-level ABI probes may use frame_enter stack arrays. All
resource-bearing caller frames are swept on a fatal exit. Return70 is an ordinary
value; trap evidence needs exact raw reason plus exit70 and balanced resource/
text/physical observations. Reasons distinguish index21, budget22, invalid23,
injected allocation25 and cooperative cancellation26. An externally killed PE
is an infrastructure failure and does not claim cooperative cleanup.

Guest logical data8MiB and MAX_LIST_ELEMS65536 remain independent limits.
The physical32MiB domain combines actual committed arena pages and registered
frame storage/headers; allocation metadata and text-ledger backing use the same
arena. Registered storage is reserved before allocation and undone on failure.
Arena fragmentation is coalesced/reused. Successful final cleanup requires live,
committed and retained metadata bytes0. PE private commit delta is separately
capped128MiB, including stack/image/allocator overhead. OS observations cannot be
missing or zero. Runtime globals have one execution thread.

The text shim rlib supplies the same functions/single panic handler; its old
staticlib still exists independently. Additive allocator hooks default to the
old process heap. Hook takeover refuses outstanding buffers/metadata; idle
metadata can be released with its original allocator. Last-frame native cleanup
releases idle metadata. Do not link both whole staticlibs or ignore duplicates.
Clone polls every node/child and4096-byte text chunk. Cancellation rolls back the
partial copy and preserves old source/destination. List growth/shift work is
admitted proportionally before mutation. Cleanup itself never polls/allocates.

Native probes cover exact8MiB,32MiB combined physical and65536-element boundaries
and plus-one refusal; fragmented reuse, Nth allocation/text-ledger failures,
clone cancellation, small-stack depth, caller traps, invalid descriptor/arm and
actual omitted-free sensitivity. Typed MIR Clone -> machine PE independently
proves List/Variant copies survive set/remove/switch/drop while the original
remains readable (84, text allocations/frees3/3). L04/L05 each use one original
text buffer with no hidden clone. These are distinct source/typed/runtime starts.

## Compilation, retained edits and persistence

Cold parsing/resolution/typechecking retains the existing CheckedEnv. The same
Emitter/binder batches at most4 real bodies and16KiB summed source body bytes;
large bodies use the original single path. All functions are actually planned,
bound, function-verified, shared-lowered, MIR-admitted and unit-produced. Exact
per-function ABC/coordinates/fingerprint/COFF equivalence covers the frozen load,
originals, owned supplements and recursive/text-return combinations.

Pure body edits parse only the granted UTF8 function fragment and perform one
resolve/typecheck/plan/bind/shared lower/admission/unit operation. Unchanged real
AST/MIR/COFF allocations remain identical. Body-local interned type pools move
atomically with candidates. Current coordinates track length changes; historical
COFF can be reused with freshly minted target MIR/spans. Declaration/configuration
invalidation explicitly performs cold frontend work and precise native reuse.
Failed/cancelled/foreign/stale/bad-link/budget candidates preserve current source,
environment, coordinates, root, manifest and runnable product pair. Runtime/config
is rechecked at prepared-edit commit before link/publication.

A process-local immutable checked MIR/layout credential avoids duplicate live
admission; it is never serialized. Shared MIR ownership dataflow omits provably
all-None borrow/field side-table headers only for all-I32/Bool slot functions.
Rules/worklist/diagnostics and original conservative steps/memory bounds stay
identical to the private dense test control; other function shapes keep n tables.

Manifest `ac889/a10-collections/3` binds source/coordinates/runtime/MIR/COFF and
required-data closure. Cold MIR uses bounded256KiB carrier pages; each slice has
its exact content hash/bounds. Restore treats every byte as untrusted: bounded
read, decode/shared MIR verification, canonical type/interface/coordinate checks,
COFF admission, runtime/build/content and linker/SDK/kernel32 identity, then real
link/run with codegen forbidden. Cache modes retain reuse/fresh/read-only behavior.
New runtime/codegen keys leave old five-domain identities unchanged. Manifest/root
publication follows successful staging link/run and atomic exe/receipt placement;
placement failure restores both prior files. Intentionally omitted frees are
allowed only in isolated ephemeral drivers, never persistent cache production.

Compiler managed caps are MirSymbol16/Staging24/IO8MiB, total48MiB. The dedicated
worker global allocator counts every Rust backing request including its header,
alignment and original partition on free. Frontend/CLIF/store default to staging;
MIR production/restore and IO buffers use explicit scopes. OS private peak delta
is separately128MiB and includes C-side storage/heap slack. Publication refuses
observed cap violations. Closing a retained session frees all MirSymbol bytes.
The earlier whole-module Atom/HIR prototype exceeded128MiB and is a retained
negative control; it is never a fallback or an alternative accepted path.

## Formal measurements and successor boundary

Frozen input116 functions/345483B, SHA256
`df314cc77208bcc115f02b446b8c1012046e96b046784a1a0c6e2e2094a1715c`,
result1786+n. Only owned collections_helper changes;115 functions remain unchanged.
Full campaign is fresh/hit/restart/body/history each30 and512 distinct revisions,
P90 sorted one-based N-floor(N/10)<=2000ms, with P95/P99 and every attempt retained.
Parent request/edit enqueue through independent result/resource admission is the
window, including helper discovery/preparation/build/content checks/link/run.
Shared path hints never replace current content hashes. A scoped repair labels its
covered paths and cannot claim the full matrix or512 gate independently; reuse
requires the Plan's explicit unchanged-input/mechanism/equivalence evidence.

891 is a planning-only typed/wide successor. It inherits the final delivered
source/ABI/runtime/cache boundaries after889 releases N, and must not change
this profile's entry/transport/receipt or old domains as an implicit side effect.
Its own Plan must separately version and verify any typed entry/wide expansion.
