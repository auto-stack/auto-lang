# c_wget (Plan 368 FU-4 — consumer parity, Layer 2)

Simple downloader consumer app: combines `http.get_sync` + `fs.write_text` +
`fs.read_text` for a GET → save → read-back verification. Compared three-way
(AutoVM vs a2r-transpiled Rust vs a native Rust oracle on `ureq` + `std::fs`).

## Layout

- `auto/c_wget.at` — Auto replication
- `tests/auto/` — Auto test cases (TAP output)
- `tests/rust/` — native Rust oracle (standalone crate, own `[workspace]`)
- `mock-server/` — minimal standalone HTTP server (127.0.0.1:18080); the
  parity runner spawns it automatically for the duration of the run

## API

- `download(url, filepath) str` — download URL content to a file, return content
- `fetch(url) str` — download URL content, return it directly
- `fetch_status(url) int` — download, return the HTTP status code
