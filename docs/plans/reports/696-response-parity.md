# PLAN-696 T-07: `api.at` response parity evidence

## Setup

- Source revision: Plan 696 worktree, based on `051e7b54023f420c3955c480d986e83197eb5899`.
- VM cases load the checked-in `examples/ui/{015-notes,017-chat,023-realworld}/src/back/api.at` through the `AutoVM` HTTP owner path.
- Rust cases use `auto-man` to generate the matching Axum backends into the isolated `.plan696-axum-workspace/`, then build and exercise the servers over localhost. The generated workspace and helper scripts are temporary and are not product outputs.

## Results

| Example | VM HTTP | Generated Axum | Compared behavior |
|---|---|---|---|
| `015-notes` | `e2e_plan696_real_015_notes_crud_parity` passes | `run015.ps1` passes against the live generated server | Seeded `Welcome` note appears; create returns 200 JSON and the new note is present on the next list request. |
| `017-chat` | `e2e_plan696_real_017_chat_crud_parity` passes | `run017.ps1` passes against the live generated server | Seeded Alice contact; message and typing endpoints return 200; generated SSE emits `NewMessage` for `Message` and `Typing` for typing events. |
| `023-realworld` | `e2e_plan696_real_023_auth_parity` passes | `run023.ps1` passes against the live generated server | Successful Sarah login returns a token without a password field; bearer `/api/user` resolves Sarah; authenticated article creation sets Sarah as author; invalid login remains 200 with empty user id 0; anonymous article creation remains 200 with an empty slug. |

VM command:

```text
cargo nextest run -p auto-lang --lib --features test-http-e2e -E 'test(/e2e_plan696_real_/)' --no-fail-fast
```

Result: 3 passed. The raw TCP HTTP and SSE regressions are recorded in the T-02/T-06 evidence and re-run as part of T-08.

Axum runtime results:

- 015: initial list includes `Welcome`; create returns HTTP 200 `application/json`; follow-up list contains `Plan 696 parity`.
- 017: `text/event-stream`; POST message and typing each return 200; frames contain `"event":"NewMessage"` and `"event":"Typing"`.
- 023: login/current-user/article/invalid-login/anonymous-article assertions all pass with the values in the table above.

Generator regression command:

```text
cargo nextest run -p auto-man --lib -E 'test(/api_gen::tests::test_sse_handler_generation|api_gen::tests::test_meta_param_header_injection|api_gen::tests::test_inline_meta_body_binds_extractors_and_wraps_early_returns|api_gen::tests::test_sse_broadcast_event_name_not_hardcoded/)' --no-fail-fast
```

Result: 4 passed. The endpoint return type now selects the SSE discriminator when it names a declared API type, so `send_message() -> Message` emits `NewMessage` even though the primary API type is `Contact`. Inline metadata handlers also bind JSON/query input, preserve early JSON returns, bind the metadata header before Axum's body-consuming `Json` extractor, and keep nested calls such as `db.current_user(bearer_token(meta))` on the body transpiler path.

## Corrections discovered while comparing backends

- The generated Axum 023 handler initially exposed source-level translation defects: nested metadata calls, unbound inline body parameters, non-JSON early returns, missing borrows for `&str` database functions, and Axum extractor ordering. These were corrected in `crates/auto-man/src/api_gen.rs` and covered by generator tests plus a live generated server.
- The generated 023 database login re-entered its `CREDS` mutex by calling `mint_token()` inside the credential iteration. `examples/ui/023-realworld/src/back/db.at` now records the match while iterating and mints the token after the loop; direct VM and generated Axum login both complete with the same response contract.
- VM `auto.bus.subscribe()` remains a compile seam: `crates/auto-lang/src/vm/ffi/stdlib.rs::shim_bus_subscribe_stub` returns `-1` and explicitly directs execution to a host implementation. Therefore the direct VM 017 endpoint cannot establish publisher-event parity for `/api/stream`; Axum's live publisher SSE and VM's generator-backed SSE are tested as separate paths. This is a pre-existing VM bus limitation, not a claim of full pubsub parity.
