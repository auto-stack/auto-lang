# av-abc-825

PLAN-825 LoadedProgram formal cases. The `ac-av/loaded-run-v1` recipe executes
the real scalar source → HIR → MIR → ABC → AV pipeline. Parent-side judgement
requires the independently declared result plus nonzero loaded-index builds,
PC lookups and parameter-map queries, with zero reference PC scans or owned
parameter-map copies. Rust first-case tests independently pin the `a,b,a`
trace and fresh-worker re-verification/recovery.
