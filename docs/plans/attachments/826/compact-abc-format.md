# auto-av/compact-abc-format (proposed, PLAN-826 SD-01)

Status: proposed for independent review and merge-owned deposit. Work has not
edited any canonical Spec or the ledger. Frozen by T-01
(`attachments/826/contract-decision.md`), production-integrated by T-02..T-05
(commits 99776f37f, 65ea26582, bd1518fc8, 5d99e566e), measured by T-06
(`bench-r1/`, `sd-draft.md`).

## 1. Wire identity and version routing

- Magic `AVAC`, version `1`, profile id `1` = the SAME `av.core-i32/1`
  semantics profile as the legacy wire (only the wire generation differs).
  Logical schema: `auto.abc.av-compact-core-i32/1`.
- AVAC/1 never reinterprets legacy `AVAB/2`, `AVAD/1|2` bytes, and legacy
  decoders reject AVAC by magic. No `DecodedAbc` back-door: a compact payload
  must enter through the compact admission entry, not be relabeled as an old
  version.
- Producer/query keys carry an `abc-codec` dimension
  (`auto.abc.av-compact-core-i32/1`; `abc_unit_key_with_codec`). Manifests gain
  an `abc_codec` field (serde default = legacy AVAB schema; not part of the
  fingerprint — the codec difference is already carried by `payload_fp`).
  The two generations occupy disjoint key spaces; cross-generation lookups are
  deterministic miss/rebuild gated on magic+version.
- Compact wire carries no function names; unit names rebind from the A09
  envelope / manifest link (`rebind_unit_name`). Legacy payloads keep their
  payload-name-vs-declared-name check.

## 2. Container layout

- Fixed header 12 B: `magic[4]`, `version:u16le`, `profile:u16le`,
  `section_count:u16le`, `reserved:u16le = 0`.
- Section directory: 9 B per entry — `id:u8`, `offset:u32le`, `length:u32le`.
  Payload after the directory must contiguously cover through EOF; section
  order follows offsets. Checked offset+length; gaps, overlap, out-of-range
  and trailing bytes are rejected.
- Required sections, exactly once each: `0x01 MODULE` (kind, entry,
  capabilities), `0x02 TYPES`, `0x03 CONSTS`, `0x04 IMPORTS`, `0x05 FUNCS`.
  Unknown ids `< 0x80` are rejected.
- Optional debug: `0x7f SOURCE` (per-function snapshot + span tables), at most
  once. Absent ⇒ `snapshot = 0`, `spans = []`. Ids `0x80..=0xff` are reserved
  for future optional debug extensions; unknown ids in that range are skipped
  by their validated length and never change semantics.
- `FUNCS` = count + per-function `body_len:u32le + body`; a body consumes
  exactly its slice (sig, slots, entry block, blocks/instrs/terminator).
  Owner/module identity stays with the D798 envelope responsibility: the
  decoder rebuilds the compact schema with an empty module owner and stamps
  every function with the same owner; the formal adapter re-stamps the real
  owner before verification.

## 3. Operand encoding and density

- Canonical ULEB128 over `u32` for indices, counts, entries and source
  positions; at most 5 bytes; the 5th byte may carry only the low 4 bits with
  continuation = 0. Redundant zero groups, high bits or continuation on the
  5th byte, and truncation are rejected. This shortest-form policy is AV's
  own canonical rule (not inherited from Wasm).
- Section offsets/lengths and function body lengths stay explicit little-endian
  `u32`. i32 constants keep a tagged exact 4-byte bit pattern (no zigzag);
  bool constants are 0/1 only.
- Instruction tags: `0x01 CONST`, `0x02 COPY`, `0x03 BIN` (sub-op 1..=11),
  `0x04 UN` (sub-op 1..=2), `0x05 CALL` (callee-kind 0=Func/1=Import),
  `0x06 TRACE` (event 1=MarkA/2=MarkB), `0x10 JUMP`, `0x11 BRANCH`,
  `0x12 RETURN`. Type tags 0=I32/1=Bool/2=Fn; slot kinds 0=Param/1=Local/2=Temp.
  Everything else (type/const/opcode/sub-op/kind/profile) is an
  unsupported/format rejection. `data`/`owned` profiles are never silently
  decoded as scalar.
- Small-index (each operand 0..=127) body-encoding targets, verified exactly:
  `BIN ≤ 5 B`, `COPY ≤ 3 B`, `RETURN ≤ 2 B` (opcode/sub-op included; module
  header, tables and debug excluded). Large indices use explicit ULEB
  expansion, never truncation. Short immediates, slot reuse, instruction
  fusion and constant-pool rewriting are NOT implemented; the const table is
  carried as-is.
- Encoding domain ≠ admission domain: 16/32-bit/LEB can express 65535/65536
  as integers, but that does not make a same-numbered slot admissible. The
  decoder may produce unchecked MIR referencing out-of-range slots; the real
  `verify_mir` rejects it (`mir.slot-ref`, `mir.jump-target`), and table-family
  over-quota is classified as resource, not format.

## 4. Admission, budget and failure classes

- The compact decoder returns only `DecodedCompactAbc { version, profile,
  module }` — never `CheckedMir`/`LoadedModule` and no execution entry.
  `abc_verify::{verify_compact_abc, verify_compact_abc_bytes}` is the ONLY
  LoadedModule admission for AVAC/1: owner re-stamp +
  `verify_mir_with_deadline` + executable-closure credential (shared
  `executable_closure` helper; profile 1 = the same `av.core-i32/1` semantics).
- Budgets reuse the delivered constants: input ≤ 4 MiB; module
  types+consts+imports+funcs ≤ 65536 (one shared family budget); per function
  slots+blocks+instrs ≤ 65536; decode work ≤ 2,000,000 steps / 5 s; decoded
  structural estimate ≤ 16 MiB. Counts are checked BEFORE allocation
  (Vec/String/span estimates plus accumulated decode budget); capacities start
  small so a count can never trigger a huge pre-allocation. AV execution
  budgets (1M steps / 1024 frames / 2 s / 8 MiB slots) are unchanged by the
  wire: fuel, frames and cancel behave identically over compact-admitted
  modules (verified: `InstrSteps`, `Frames`, `Cancelled` budget rows).
- format (truncation, overlap, non-canonical ULEB, unknown required/opcode),
  resource (input/table/work/wall/decode-memory) and semantic
  invalid/unsupported (kept by the verifier's original classification) stay
  separated. Admitted dispatch of an invalid module = 0.
- Debug stripping equivalence: a real lowered wire includes `SOURCE`; removing
  it leaves admission + execution (result/trace/capabilities) identical.
  A good hash with an invalid inner payload is still rejected; no lazy trust.

## 5. Integration surface (consumed, not redefined here)

- Driver/worker wire policy: `WireCodec` (`LegacyAbc` default / `CompactAbc`
  opt-in via `set_wire_compact`) routes A09 emission/hit/closure admission and
  restore unit/closure decoding; default legacy behavior is byte-identical.
  Worker accepts recipe `ac-av/compact-run-v1`.
- Incremental sessions (SD-07): the compact wire rides the 828 session surface
  (`warmup_with_codec` / `restore_with_codec` / `warmup_persisted_with_codec`);
  unchanged functions keep object/generation identity, their six production
  stages show zero IO, and precise invalidation still fires on real
  signature/dependency changes.
- Size reporting (SD-06): both wire generations are accounted per workload —
  raw, same-algorithm same-parameter gzip, and loaded — from frozen payloads;
  body ratios never stand in for whole-module totals.

## 6. Verification evidence (work-owned, review-consumed)

V-01 first integration (`first` scope), V-02/03 density + integer domain and
V-04/05 mutation/security matrices (`targeted`), V-06/09 cache/restore and
consumer compat (`compat`), V-12 four incremental sub-cases (`incremental`),
V-14 unified 852 validation (`baseline`), V-08/09/10 measurement and parent
side counter-controls (`bench`), all aggregated by `acceptance`
(`scripts/verify-av-826.ps1`). Module-same-source proof (V-13): the production
core and the independent shell compile the same `compact_abc.rs` by path.
