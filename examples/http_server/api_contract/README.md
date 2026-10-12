# API Contract & Generation Integrity (PLAN-734)

The same `api.at` serves the ordinary-JSON contract on the VM default HTTP
transport and the generated Rust/Axum service — one typed contract
(`api/contract.rs` classification) drives parameter binding, response
marshaling, and every generator.

## Run

```
cd examples/http_server/api_contract
AUTO_HTTP_PORT=8080 auto run -r vm    # VM default HTTP
AUTO_HTTP_PORT=8080 auto run          # generated Rust/Axum (same api.at)
```

## Probes

```
curl -s -X POST -H 'Content-Type: application/json' \
  -d '{"n":42,"tag":"hi"}' http://127.0.0.1:8080/api/echo
# 200 {"ok": true, "n": 42, "tag": "hi"}       — typed body binding

curl -s -X POST -H 'Content-Type: application/json' \
  -d '{"n":"forty-two","tag":"hi"}' http://127.0.0.1:8080/api/echo
# 400 — str-for-int is a bad shape (never silently marshaled)

curl -s http://127.0.0.1:8080/api/big
# 200 5000000000 — i64 full range (no u32 mask / null / truncation)

curl -s http://127.0.0.1:8080/api/bigparam/5000000000
# 200 5000000001 — i64 path param round-trip

curl -s http://127.0.0.1:8080/api/errorfield
# 200 {"error": "business-rejected", "value": 7} — an `error` FIELD is data

curl -s http://127.0.0.1:8080/api/plain
# 200 734734 — plain ints are never hijacked by resource-id tables
```

## Notes

- Parameter source priority is `path > JSON body > query`; the whole-body
  single-param tolerance and the `meta`/`metadata`/`req`/`request` cookie/auth
  aliases remain as explicitly-marked compatibility sources.
- Generation is strict: an api.at parse failure, an untranspilable business
  body, or an HTTP-exclusive kind on a non-HTTP consumer fails the build with
  a structured diagnostic (endpoint + stage + reason) instead of falling back
  to a CRUD template. `AUTO_A2R_BODY=0` opts into an explicitly labeled
  SCAFFOLD mode.
- A `generation.json` ready record (source fingerprints) ships with each
  generated bundle; `AUTO_REUSE_BACKEND=1` refuses to reuse a stale generation.
- File (729) / upload (730) / SSE endpoints keep their own protocols and
  rejection diagnostics — they never degrade to JSON.
