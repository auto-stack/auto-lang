# HTTP Server File Responses (PLAN-729)

Serve file downloads from a named `#[api]` GET handler with `http.file_response`:
GET/HEAD, single-range (`Range`), conditional requests (`If-None-Match` /
`If-Match` / `If-Modified-Since` / `If-Range`), restricted-root opening and
bounded streaming. Works identically on the VM default HTTP transport
(`auto run -r vm`) and the generated Rust/Axum backend (`auto run`).

## Run

```bash
# files served from this directory (root is app-trusted config)
mkdir -p files
echo "hello plan729" > files/hello.txt

auto run -r vm   # VM default HTTP transport
# or
auto run         # generated Rust/Axum service (same api.at)
```

Then:

```bash
curl -v http://127.0.0.1:8080/api/files/hello.txt        # GET 200
curl -I http://127.0.0.1:8080/api/files/hello.txt        # HEAD (same metadata)
curl -H "Range: bytes=0-4" http://127.0.0.1:8080/api/files/hello.txt   # 206
curl -H "If-None-Match: v1" -v http://127.0.0.1:8080/api/etagged/hello.txt  # 304
```

## Notes

- The relative path comes from the route param and is untrusted: traversal
  (`..`), absolute/drive/UNC paths and device names are rejected with 403.
- File endpoints support GET/HEAD only; other annotated methods get 405.
- Non-HTTP call forms (Tauri IPC, in-process back-proxy) explicitly reject
  file endpoints — use the HTTP URL.
