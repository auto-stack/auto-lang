# http_client_sync (D3 — skeleton, blocked)

Synchronous HTTP POST parity library. **Status: skeleton committed, blocked
from running three-way by a pre-existing parser bug (DIV-HTTP-LANG-1).**

## Layout

- `mock-server/` — minimal standalone Rust HTTP server. Listens on
  `127.0.0.1:18080`, responds to every POST with `{"echo":"ok"}` (status 200),
  405 for other methods. `cargo run` from this dir.
- `auto/http_client_sync.at` — Auto wrapper: `post_echo(body) -> str` calls
  `auto.http.post_sync("http://127.0.0.1:18080/echo", body, "")`.
- `tests/auto/post_echo.at` — Auto TAP tests (3 cases), names mirror the Rust oracle.
- `tests/rust/` — Rust oracle using `ureq` (matches a2r-std's transport).

## Blocker

Any `use auto.http: ...` fails to parse because the shipped stdlib
`auto/http.at` uses declaration syntax the current parser rejects
(see `parity/docs/known-divergences.md` DIV-HTTP-LANG-1). This is a
language/parser bug independent of a2r and parity.

## Completing this library (once DIV-HTTP-LANG-1 is fixed)

1. Fix the stdlib `auto/http.at` declaration parsing (separate language task).
2. Add a runner setup/teardown hook in `auto-parity` that spawns
   `mock-server/` before the three-way run and kills it after — the mock
   server must outlive all three backend processes (they are independent
   processes with no shared lifecycle).
3. `auto-parity run http_client_sync` should then report 3/3 consistent.

The skeleton is committed so the design and the mock-server artifact are not
lost; the library is **not** in the L1 count and **not** wired into the
dashboard's phase table until the blocker is resolved.
