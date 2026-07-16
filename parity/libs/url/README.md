# url Replication

**Upstream:** url crate v2.5.0 (`url::Url::parse`)
**Scope:** URL parsing — split a URL string into its components
(scheme, host, port, path, query, fragment).

Format parsed:
```
scheme://[host[:port]][/path][?query][#fragment]
```

## API

The Auto replication exposes URL parsing as **free functions that take the raw
URL string** and return a primitive component (str / int). This mirrors the
proven base64 replication shape and keeps the module boundary safe (see
"Implementation notes" below).

- `parse(input str)` — validate a URL; returns `Ok(input)` when the URL has a
  `://` separator with a non-empty scheme, `Err(msg)` otherwise.
- `scheme(input str) str` — lower-cased scheme, e.g. `"http"` (`""` if none)
- `host(input str) str` — host without port, e.g. `"example.com"`
- `port(input str) int` — explicit port, or `-1` when not specified (sentinel)
- `path(input str) str` — path, normalised to `"/"` when empty (matches the
  url crate)
- `query(input str) str` — query without the leading `?`, `""` when absent
- `fragment(input str) str` — fragment without the leading `#`, `""` when
  absent

## Representation notes / divergences from the url crate

This is a *simplified* parser built to exercise the same input → output
mapping as `url::Url::parse` for the component-extraction cases the tests
cover. It is not a full WHATWG URL parser. Key, deliberate differences:

- **Optional values as sentinels, not `Option`.** `port` is `int` with `-1`
  meaning "absent"; `query`/`fragment` are empty strings when absent. The url
  crate uses `Option<u16>` / `Option<&str>`. The parity tests are written so
  the same cases pass on both backends (presence is tested with explicit
  values; absence is tested separately).
- **No default-port stripping.** The url crate drops `:80` from `http://` and
  `:443` from `https://` URLs; this parser keeps explicit ports. Tests avoid
  default-port cases (use non-default ports when a port is expected).
- **No percent-decoding / lower-casing of host.** Components are returned
  verbatim from the input (apart from scheme lower-casing and path
  normalisation to `/`).
- **No validation of allowed characters / IPv6 / credentials.** Such inputs
  are out of scope.

## Implementation notes (Auto VM workarounds)

- **Free functions, not a `Url` struct.** User-defined struct values do not
  reliably cross the Auto module boundary through a `Result` Ok payload in the
  current VM (the struct value is corrupted when destructured by the caller),
  so the API returns only `str` / `int` primitives. Each accessor re-slices
  the raw URL string.
- `parse` returns `Ok(input)` / `Err(msg)`; only primitives cross the module
  boundary.
- `str.char_at(i)` returns the code point as `int`, so all character
  comparisons use integer codes; strings are built with `StringBuilder`.
- **Reading the `Err` string payload is broken in the Auto VM** (the value
  comes back as a small negative integer rather than the message). Error test
  cases therefore only assert *that* parsing failed (`is_err`), not the
  message content — mirroring the base64 `check_err` pattern.

## a2r (transpiler) note

The a2r transpiler was extended (Plan 355) to infer `Result<T, String>` return
types for un-annotated functions returning `Ok(...)`, where `T` is derived
from the Ok payload (previously it always inferred `Result<String, String>`).
This fix is exercised by the `parse` function and keeps the transpiled Rust
compiling.

## Known divergences

See `parity/docs/known-divergences.md`.
