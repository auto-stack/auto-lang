# tokio Replication (async/await subset)

**Upstream:** tokio crate v1.0 (async runtime)
**Scope:** Async/await serial future composition — `~T` functions, `.await`, nested async calls.
Does NOT include: spawn/join, channels, timers, select (see known-divergences.md for VM limitations).
**Auto features tested:** async/await (`~T` → `async fn`, `.await` transpilation).

## API

- `double(n int) ~int` — async function that doubles
- `add(a, b int) ~int` — async function that adds
- `compute(a, b int) ~int` — nested async (double then add)
- `delay_value(v int) ~int` — async identity
- `pipeline(n int) ~int` — multi-step async composition

## Known divergences

See `parity/docs/known-divergences.md` for:
- DIV-TOKIO-VM-1: `.go` (spawn) crashes VM with stack underflow
- DIV-TOKIO-VM-2: `Handle[T]` generic syntax not parseable
- DIV-TOKIO-VM-3: channels (`chan_new[T]`) have no Auto-level syntax binding
