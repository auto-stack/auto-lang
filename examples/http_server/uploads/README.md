# HTTP Server Upload Ingress (PLAN-730)

The same `api.at` serves uploads on the VM default HTTP transport and the
generated Rust/Axum service — identical receive/commit semantics from one
shared host implementation.

## Run

Prepare two sibling directories (staging must exist, live on the same volume,
and stay OUTSIDE the public root):

```
mkdir -p /tmp/uploads-demo/private-staging /tmp/uploads-demo/public-files
```

VM mode:

```
cd examples/http_server/uploads
UPLOADS_PUBLIC_ROOT=/tmp/uploads-demo/public-files \
UPLOADS_STAGING_ROOT=/tmp/uploads-demo/private-staging \
auto run -r vm
```

Generated Rust mode (same api.at):

```
UPLOADS_PUBLIC_ROOT=... UPLOADS_STAGING_ROOT=... auto run
```

## Probes

```
# multipart upload (201 + receipt JSON)
curl -s -F note=hello -F file=@some.bin http://127.0.0.1:8080/api/uploads

# raw upload (PUT)
curl -s -X PUT --data-binary @some.bin -H 'Content-Type: application/octet-stream' \
  http://127.0.0.1:8080/api/uploads/raw

# early reject (401 receipt — no file I/O; works with Expect: 100-continue)
curl -s -X POST -H 'Expect: 100-continue' http://127.0.0.1:8080/api/early

# download the committed file back (729 route)
curl -s http://127.0.0.1:8080/api/files/blob.bin -o back.bin
```

## Notes

- Upload endpoints are `#[api]` handlers with an `UploadRequest` parameter and
  an `UploadReceipt` / `~UploadReceipt` return (POST/PUT only). Routing and
  auth run before any byte of the body is parsed or written.
- Receiving is bounded: single file (default 64 MiB), ordered text fields,
  wire/parts/headers budgets; oversize → 413, malformed multipart → 400,
  unsupported encoding → 415, idle/total → 408, queue full → 503.
- Commit is atomic create-only (`hard_link` publish): an existing target is a
  409 receipt and the original file's bytes stay untouched; the client
  filename never becomes the storage path.
- Business checks read `http.upload_metadata(session)` between receive and
  commit; sessions that neither commit nor reject expire (30s lease) and the
  staging file is cleaned up.
- Tauri IPC and the in-process back-proxy reject upload endpoints with a
  clear diagnostic (use the HTTP URL).
