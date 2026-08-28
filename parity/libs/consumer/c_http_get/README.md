# c_http_get (Plan 368 FU-4 — consumer parity, Layer 2)

HTTP GET consumer app: uses `http.get_sync` to fetch a URL and verifies both
the response body and the status code. Compared three-way (AutoVM vs
a2r-transpiled Rust vs a native Rust oracle on `ureq`).

## Layout

- `auto/c_http_get.at` — Auto replication
- `tests/auto/` — Auto test cases (TAP output)
- `tests/rust/` — native Rust oracle (standalone crate, own `[workspace]`)
- `mock-server/` — minimal standalone HTTP server (127.0.0.1:18080); the
  parity runner spawns it automatically for the duration of the run

## API

- `fetch_body(url) str` — GET request, returns the response body
- `fetch_status(url) int` — GET request, returns the HTTP status code
