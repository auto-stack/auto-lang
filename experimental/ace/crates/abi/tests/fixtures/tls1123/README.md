# PLAN-1123 TLS test fixtures (loopback only)

TEST-ONLY identities for `127.0.0.1` peers. Every private key here is
public on purpose; never use any of them for anything but a loopback test.

| File | Role |
| --- | --- |
| `ca-test.cert.pem` / `.der` | the LOCAL TEST TRUST — the CA a test tells the transport to trust |
| `ca-untrust.cert.pem` | an issuer no test trusts |
| `leaf-ok.server.pem` | key + certificate valid for `localhost` and `127.0.0.1`, signed by `ca-test` (must pass) |
| `leaf-ok.cert.pem` | the same certificate alone |
| `leaf-name.server.pem` | key + certificate valid for `other.invalid` only, signed by `ca-test` (hostname failure) |
| `leaf-ca.server.pem` | key + certificate valid for `localhost` and `127.0.0.1`, signed by `ca-untrust` (trust failure) |
| `https_app.at` | the ordinary project's Auto source: HTTP request/response over the secure transport (one author for the provider lane and the project lane) |
| `peers.rs` | the loopback peers both lanes share: a rustls TLS server, a plaintext origin and an HTTP proxy (CONNECT tunnel, absolute-form forward, reject, close, stall) |
| `gen-test-pki.sh` | regenerates the PKI above |

## Regenerating the PKI

```sh
bash experimental/ace/crates/abi/tests/fixtures/tls1123/gen-test-pki.sh <empty-dir>
```

The script needs OpenSSL 3 (`req -addext`, EC P-256). It writes new keys, so
regenerating changes every certificate; replace the files above together and
re-run the `avrs_tls_1123` / `tls_project_1123` cases. The certificates are
valid for 30 years so the daily lane never expires them.
