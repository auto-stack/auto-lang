# c_env_app (Plan 368 F3 — consumer parity)

Environment-variable consumer application. Verifies that Auto consuming the
`env.*` stdlib (backed by std::env) produces the same behavior as a native
Rust app using std::env directly — three-way (AutoVM / a2r / native Rust).

This is **consumer mode**: Auto calls library capabilities (`env.get` /
`env.set` / `env.get_or`) to read/write environment variables. The Rust
oracle calls std::env directly.

## API

| Function | Signature | Calls (Auto) | Calls (Rust oracle) |
|----------|-----------|--------------|---------------------|
| `set_and_get` | `(key, val) -> str` | `env.set` + `env.get` | `env::set_var` + `env::var` |
| `get_missing` | `(key) -> str` | `env.get` | `env::var` (empty on miss) |
| `get_or_value` | `(key, default) -> str` | `env.get_or` | `env::var(..).unwrap_or(default)` |

## Determinism (design doc §F3)

Environment variables are process-global, and the three backends run as
independent processes — so the suite tests the **"within this process,
`set(k,v)` then `get(k) == v`" behavior pattern**, not a cross-process shared
value. Each test uses a UNIQUE key (`C_ENV_APP_TEST_*`) so parallel `#[test]`
threads on the Rust side do not collide.

7 test cases, names mirror the Rust oracle exactly:
`test_set_get_basic`, `test_set_get_unicode`, `test_set_get_empty`,
`test_set_get_overwrite`, `test_get_missing`, `test_get_or_exists`,
`test_get_or_missing`.

## a2r notes (Plan 368 F3)

Same family of pre-existing a2r transpiler quirks as c_fs_app (documented
there). To keep the test source unambiguous across backends, all env keys are
passed as **string literals** (no concatenation, no reused owned variables),
which sidesteps both: (1) owned `str` vars registered as `StrSlice` but
transpiled to `String`, and (2) inline-concat args to user functions not
being auto-`.as_str()`'d.

Two a2r fixes were needed for `env.*` (committed alongside this lib):
* the generic `Map.set -> HashMap::insert` rewrite now **skips stdlib module
  receivers** (it used to capture `env.set` and emit invalid `env.insert(...)`,
  E0423);
* the `env.*` stdlib routing now handles `get`/`set`/`get_or` and borrows
  string args (`expr_as_str`) so an owned key is not moved.
* `a2r-std::env::get` returns `String` (empty on miss) to match the VM
  `auto.env.get` native (`shim_env_get` → `unwrap_or_default()`).
