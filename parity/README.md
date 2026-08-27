# Parity Verification

The `parity/` workspace verifies that Auto programs behave identically across
execution backends. The `auto-parity` tool (`crates/auto-parity/`) runs
multi-way consistency checks per library:

- **AutoVM** — the `auto` interpreter (native Auto implementation, the oracle)
- **a2r** — Auto transpiled to Rust, compiled and run
- **native oracle** — hand-written Rust (or Python for `libs/python/`) calling
  the same upstream crate / stdlib directly

The `aavm` subcommand additionally runs the AAVM self-hosting matrix
(`auto run` vs merged-lib VM vs golden) over the v2 execution-layer corpus.

## Directory layout

```
parity/
├── crates/auto-parity/   the checker tool (this workspace's only crate)
├── docs/                 parity-guide.md, known-divergences.md, dashboards
└── libs/                 the test corpus, categorized (Plan 460)
    └── <category>/<name>/   one directory per library under test
```

**Library identity = leaf directory name** (`py_math`, `c_wget`, `base64`).
It is what you pass on the CLI (`run py_math`), what appears in the phase
table and TAP results, and the module name under each lib's `auto/` directory
(`auto/py_math.at`, imported as `use auto.py_math`). The category level is
organisation only — adding a new category requires no tool changes. Each lib
directory is self-contained: `auto/<name>.at` (Auto replication),
`tests/auto/` (Auto TAP cases), `tests/rust/` (native Rust oracle crate),
optionally `tests/python/` (Python oracle) and `mock-server/` (live HTTP
fixture), plus a per-lib `README.md` with scope, upstream version and known
divergences.

## Categories

### `libs/rust/` — Rust crate replication (8)

Replicates a published Rust crate's API surface in Auto and verifies the
three-way result matches the real crate. The original Plan 347 p1–p4 rollout.

| case | upstream | intent |
|---|---|---|
| `base64` | base64 v0.22 | encode/decode |
| `regex` | regex v1.10 | `is_match` / `find` |
| `rusqlite` | rusqlite v0.31 | `FromSql`/`ToSql` query layer |
| `serde_json` | serde_json v1.0 | JSON parse/serialize subset |
| `sha2` | sha2 v0.10 | Sha256 digest |
| `tokio` | tokio v1.0 | async/await runtime subset |
| `tokio_stream` | async-stream pattern | `~Stream<T>` generator path (Plan 321/364; for-over-Stream consumption documented as open) |
| `url` | url v2.5 | `Url::parse` |

### `libs/python/` — Python stdlib parity (13) + sci-compute parity (4)

Three-way: AutoVM vs **a2py** (Auto transpiled to Python) vs native Python.
Parity mode is auto-detected from the lib's `tests/python/` directory
(Plan 369). The `py_*` stdlib libs each mirror one stdlib module:

| case | stdlib module | intent |
|---|---|---|
| `py_configparser` | `configparser` | INI config read/write |
| `py_datetime` | `datetime` | date/time construction & formatting |
| `py_hashlib` | `hashlib` | digest algorithms |
| `py_json` | `json` | JSON encode/decode |
| `py_list` | list builtins | list method semantics |
| `py_math` | `math` | math functions |
| `py_os` | `os` | os module subset (path/env) |
| `py_random` | `random` | seeded random (deterministic) |
| `py_re` | `re` | regular expressions |
| `py_string` | `string`/str methods | string method semantics |
| `py_struct` | `struct` | binary pack/unpack |
| `py_sys` | `sys` | sys module subset |
| `py_uuid` | `uuid` | UUID construction/formatting |

**Sci-compute extension** (Plan 461, phase p8): numpy / pandas / matplotlib /
torch called through `use.py` + the embedded-CPython bridge. The suites also
fix the calling conventions: data is created Python-side (`arange` etc.) and
stays on the Python side of the boundary as opaque `PyObjectHandle`s; only
scalars/deterministic strings marshal out; object members go through
`py_call(obj, "method", ...)` / `py_getattr(obj, "attr")` (a2py lowers them to
`obj.method(...)` / `obj.attr`). Auto list/dict arguments into Python are not
used — see known-divergence DIV-PY-AUTOLIST-1.

| case | upstream | intent |
|---|---|---|
| `py_numpy` | numpy | ufuncs, reductions, reshape/shape, dot, dtype/array string forms |
| `py_pandas` | pandas | DataFrame from numpy handle, shape/len, column sums, iloc row selection |
| `py_matplotlib` | matplotlib | headless plot + savefig file artifacts |
| `py_torch` | torch (CPU) | tensor creation, sum, relu/abs, runtime type strings |

### `libs/consumer/` — consumer apps (9)

Whole-application scenarios in the style of C systems programs, exercising
`auto.<module>` stdlib composition (http/fs/env/process/json). Each is
compared three-way against a native Rust oracle calling the same underlying
crate directly (Plans 367/368, phases d5/d6). The HTTP consumers
(`c_crawler`, `c_http_get`, `c_wget`, `http_client_sync`) get a live mock
server spawned by the runner (`mock-server/`, 127.0.0.1:18080).

| case | intent |
|---|---|
| `c_crawler` | multi-page crawl: `http.get_sync` + `Json.parse/get` |
| `c_env_app` | environment variable reading |
| `c_fs_app` | file read/write |
| `c_http_get` | HTTP GET body + status vs `ureq` |
| `c_json_app` | JSON parse/manipulate |
| `c_process_app` | child process spawn/management |
| `c_text_app` | text processing |
| `c_wget` | GET → save → read-back downloader (`http` + `fs`) |
| `http_client_sync` | sync HTTP POST — **skeleton, blocked** by DIV-HTTP-LANG-1 |

### `libs/lang/` — language feature demos (4)

Pure-computation libraries (no IO) whose purpose is to exercise specific Auto
language features end-to-end through all backends (Plan 358).

| case | feature focus |
|---|---|
| `cli_app` | string indexing (`char_at`), loops, boolean ops — wc-style text stats |
| `generators` | `fn f() ~Iter<T> { yield x; }` generator syntax (Plans 359/417) |
| `string_utils` | hand-rolled ASCII string ops — a2r pure-Rust output coverage |
| `trait_advanced` | `spec` traits: default methods, associated types, generics |

### `libs/framework/` — runner smoke (1)

| case | intent |
|---|---|
| `_dummy` | framework smoke test — verifies the runner pipeline itself, not a real library. Excluded from `list`/`all`; run explicitly via `phase p0` |

## Running

Full usage lives in [`docs/parity-guide.md`](docs/parity-guide.md). Quick
start from `parity/`:

```
cargo run -- --root . --auto-binary <path-to-auto> list        # 34 libs (no _dummy)
cargo run -- --root . --auto-binary <path-to-auto> run py_math # single lib
cargo run -- --root . --auto-binary <path-to-auto> phase p1    # by phase
cargo run -- --root . --auto-binary <path-to-auto> all         # everything
```

Phases (p0–p7, d1–d6) are rollout batches defined in
`crates/auto-parity/src/main.rs` (`discover_libraries_by_phase`) — that table
is the source of truth for phase membership.

## Adding a new library

1. Pick the category matching the lib's parity target (or create a new
   category directory — no code changes needed). Create
   `libs/<category>/<name>/` following the self-contained layout above; the
   leaf directory name is the library identity.
2. Give it a `README.md` (scope, upstream version, known divergences).
3. Optionally register it in a phase in `crates/auto-parity/src/main.rs`.
4. Detailed conventions (TAP output format, test-crate workspace rules):
   [`docs/parity-guide.md`](docs/parity-guide.md).
