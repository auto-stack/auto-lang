# c_crawler (Plan 368 FU-4 — consumer parity, Layer 2)

Simple crawler consumer app: combines `http.get_sync` + `Json.parse/get` to
crawl multiple pages served by the bundled mock server. Compared three-way
(AutoVM vs a2r-transpiled Rust vs a native Rust oracle on `ureq` + `serde_json`).

## Layout

- `auto/c_crawler.at` — Auto replication
- `tests/auto/` — Auto test cases (TAP output)
- `tests/rust/` — native Rust oracle (standalone crate, own `[workspace]`)
- `mock-server/` — minimal standalone HTTP server (127.0.0.1:18080); the
  parity runner spawns it automatically for the duration of the run

## API

- `crawl_page1_body(url) str` / `crawl_page2_body(url) str` — fetch page body
- `crawl_status(url) int` — fetch and return the HTTP status code
- `crawl_file(url) str` — fetch static file content
