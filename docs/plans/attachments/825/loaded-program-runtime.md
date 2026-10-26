# Proposed Spec delta SD-01: LoadedProgram execution representation

Status: proposed for merge, not canonical Spec content.

- MIR and portable ABC remain the semantic and persistence contracts. An
  ABC-admitted complete module or linked `ProgramSnapshot` may build a private
  `LoadedProgram`; indexes are rebuilt after untrusted restore and are never
  serialized as trusted state or host addresses.
- `av.core-i32/1` uses per-function block-base and declaration-order parameter
  tables. Dispatch performs one bounded lookup per logical PC instead of
  summing all preceding blocks. Table construction is O(B+I) for the derived
  portion, separately accounted from complete verification work.
- `execute`/`execute_entry`, ProgramSnapshot execution, and stepped/resumed
  execution consume the same source and the one existing dispatcher.
  `execute_reference`, `execute_entry_reference`, and
  `execute_program_reference` retain the checked reference path. Data, owned,
  and collections profiles do not acquire a scalar optimized fallback.
- Resident indexes retain the original admitted `FunctionCode`. An unchanged
  code `Arc` preserves index identity and generation across revisions and slot
  reordering. Changed code gets a new table while old snapshots keep old code
  and tables alive. A same-content new code object is not an identity hit.
- Index bytes are execution-partition charges, not an additional entitlement.
  The existing execution 8 MiB partition includes them together with slots and
  frame storage. The shared budget ledger spans old and new snapshot
  generations. Table construction and allocation failures happen before the
  affected function can dispatch.
- Execution traces expose loaded/reference PC and parameter-map counts, index
  builds/reuses/bytes, root/call frame commits, and remaining frames. These
  observations are required to reject correct results produced by a silently
  degraded path. All terminal states leave zero frames after cleanup.
- The `ac-av/loaded-run-v1` recipe uses the real source→HIR→MIR→ABC→AV path.
  Its parent judge requires independent results plus nonzero loaded build/PC/
  parameter counts and zero reference PC scans/copies on successful optimized
  runs. Restore re-verifies portable inputs before rebuilding execution tables.
