# rusqlite Replication

**Upstream:** rusqlite crate v0.31.0 (`FromSql` / `ToSql` query layer)
**Scope:** the deterministic, pure-function value-marshalling layer of rusqlite —
`FromSql::column_result` (SQLite Value -> Rust value) and `ToSql` (Rust value ->
SQLite Value) coercions, including integral range checking (`OutOfRange`),
type-mismatch detection (`InvalidType`), and the Integer->Real/bool coercions.
**Auto features tested:** tagged-union emulation via an opaque struct handle,
multi-arm type dispatch, signed-int range comparisons, int->float widening,
module-boundary value marshalling, Result-free error encoding.

## What is NOT replicated (and why)

rusqlite is a *driver* over SQLite, not pure logic. The stateful, opaque parts
cannot be replicated in the current Auto VM and are out of scope:

- **`Connection` / `Statement`** — opaque handles that cannot cross the VM
  boundary. `use.rust` `RustFfiBridge` marshals only primitives via
  `VMConvertible` (i32, u32, bool, i64, u64, f64, String, Vec<...>, Option,
  tuples); `Connection`/`Statement` have no `VMConvertible` impl and no
  `RustStdlibObject` shim exists for them. See `DIV-RUSQLITE-1`.
- **SQL execution / query planning** — that is SQLite's job, not rusqlite's; it
  is non-deterministic w.r.t. storage and not a pure function.

The query-layer coercions (`FromSql`/`ToSql`) ARE pure functions of the value
alone, so they are exactly the right slice for a three-way parity check. The
native oracle runs each value through a real in-memory SQLite `SELECT ?1` so
the Value -> Rust mapping goes through genuine rusqlite 0.31.0.

## API

A SQLite `Value` is modelled as an **opaque** struct `Val { kind, ival, sval,
fval }`:

| kind | variant  | payload field |
|------|----------|---------------|
| 0    | Null     | —             |
| 1    | Integer  | `ival` (i64 bit pattern) |
| 2    | Real     | `fval` (f64)  |
| 3    | Text     | `sval` (str)  |
| 4    | Blob     | `sval` (bytes-as-str) |

Each `FromSql` coercion is exposed as TWO functions returning plain primitives
(avoiding `Result`-crossing-boundary bugs — see Implementation notes):

- `<name>_status(v) int` -> `0` Ok, `1` InvalidType, `2` OutOfRange
- `<name>_value(v) <T>`   -> the coerced payload (valid only when status == 0)

Coercions: `from_i64`, `from_i32`, `from_i16`, `from_i8`, `from_u32`, `from_u16`,
`from_u8`, `from_f64`/`from_f32`, `from_bool`, `from_string`, `from_blob`,
`from_option`. `ToSql`: `to_sql_i64`, `to_sql_i32`, `to_sql_f64`, `to_sql_bool`,
`to_sql_string`, `to_sql_null`. Constructors: `null_`, `integer_`, `real_`,
`text_`, `blob_`. Inspector: `data_type`.

## Implementation notes

### Load-bearing VM workarounds

- **Result payloads corrupt across the module boundary in two ways**, so the
  public API never returns `Result<T>` across it:
  - Err *string* payloads read via `is`/`match` come back as a small negative
    tag marker (DIV-URL-VM-2, also seen in base64/url).
  - `Result`-wrapped *float* payloads crossing the boundary come back with a
    corrupted bit pattern (DIV-RUSQLITE-VM-1, discovered here).
  - Plain (non-Result) int / float / str returns cross cleanly. Hence the
    split `status` + `value` API, each returning a plain primitive.
- **`Val` is an opaque handle.** Callers construct it via the `null_/...`
  constructors and consume it via `from_*`/`data_type` only; they never read
  `Val` fields directly. This sidesteps the user-defined-struct field-corruption
  bug (DIV-URL-VM-1).
- **`tag` is a reserved keyword** in Auto (it tokenises as `Tag`), so the
  discriminator field is named `kind`.
- **The VM `int` is 32-bit signed and silently wraps** — it has no i64. Values
  outside [-2147483648, 2147483647] cannot be represented. This constrains the
  testable range (see Known divergences / DIV-RUSQLITE-VM-2).

### Why not `use.rust` FFI?

The Plan-355 task brief explored calling rusqlite directly via `use.rust
rusqlite::Connection`. This is not viable in the current VM: `use.rust`
`RustFfiBridge` can only marshal `VMConvertible` primitives, and
`Connection`/`Statement` are opaque handles with no marshal path. The proven
parity-library pattern (base64, url, serde_json, regex, sha2) is a pure-Auto
reimplementation compared against a native oracle, so this library follows that
pattern on the deterministic query-layer slice.

## Known divergences

See `parity/docs/known-divergences.md` entries `DIV-RUSQLITE-1`,
`DIV-RUSQLITE-VM-1`, `DIV-RUSQLITE-VM-2`. In summary:

- `DIV-RUSQLITE-1` — `Connection`/`Statement` not replicable (opaque, no FFI
  marshal). Out of scope by design.
- `DIV-RUSQLITE-VM-1` — cross-module `Result`-wrapped float payload corruption.
  Worked around (status+value API).
- `DIV-RUSQLITE-VM-2` — 32-bit VM int cannot represent i32/u32 out-of-range
  boundary values (2147483648, 4294967295, ...). Those specific cases are
  excluded from the suite; i8/i16/u8/u16 exercise the OutOfRange path with
  representable values, and all three backends agree on every included case.
