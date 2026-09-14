//! PLAN-622 red corpus: named-store facade consumption gaps (jade PLAN-064
//! T-05 evidence, DEBTS 064 row ⑥).
//!
//! Empirical map on master @ plan-622-dev base (2026-09-14 iteration):
//!
//! **RED (gap reproduced):**
//! - (c2) an `#[api]` fn called from INSIDE the store module does NOT reach
//!   the backend in split mode (`plan622_c2_...`): full production-shaped
//!   build with `api_over_http = true` + AUTO_BACKEND mock — no HTTP request
//!   arrives (the call executes the in-module stub body instead).
//! - (d) widget-model array `.splice` is a silent no-op (`plan622_d_...`).
//! - (e1) lambda capturing a handler-LOCAL var resolves to nothing — the
//!   find silently misses (`plan622_e1_...`; jade saw the sharper compile-
//!   error shape, same capture semantics).
//! - (e2) lambda comparing against a self field silently misses
//!   (`plan622_e2_...`).
//!
//! **GUARDS (pass on master — kept as regression fence):**
//! - (a/a2/a3/b) bare handler reads of merged store fields, nested
//!   reference-writes read back, map-arg store-msg dispatch, and view
//!   bindings following store-msg mutation all work in the minimal corpus.
//!   The jade sentinel/freeze family did not reproduce at this layer; if the
//!   jade facade switch (cross-repo acceptance) re-surfaces it, these arms
//!   extend into red tests there.
//!
//! Corpus: `test/ui/plan622_store_facade/` — app.at (facade consumption),
//! counter_store.at (named StoreDecl with #[api] call), back/api.at
//! (contract fn), lambda_app.at (isolated e arms so compile-level poisoning
//! cannot mask the runtime arms).

#[cfg(test)]
mod plan622_store_facade_gap_tests {
    use crate::plan370_test_support::build_component_from_app;
    use auto_val::Value;

    fn locate_corpus(rel: &str) -> Option<std::path::PathBuf> {
        let full = format!("test/ui/plan622_store_facade/{}", rel);
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(&full)),
            Some(std::path::PathBuf::from(&full)),
            Some(std::path::PathBuf::from(format!("../../{}", full))),
        ];
        candidates.into_iter().flatten().find(|p| p.exists())
    }

    fn build_app(rel: &str) -> Option<crate::ui::dynamic::DynamicComponent> {
        build_component_from_app(&locate_corpus(rel)?)
    }

    fn state_raw(dc: &crate::ui::dynamic::DynamicComponent, field: &str) -> String {
        match dc.read_state(field) {
            Ok(v) => format!("{:?}", v),
            Err(e) => panic!("read_state('{}') failed: {}", field, e),
        }
    }

    fn snapshot_of(dc: &crate::ui::dynamic::DynamicComponent) -> String {
        let state = dc.read_all_state_materialized();
        let template = dc.view_template();
        use crate::ui::aura_snapshot_builder::AuraSnapshotBuilder;
        let builder = AuraSnapshotBuilder::new(&state);
        builder.build(dc.widget_name(), template)
    }

    // ─────────────────────────────────────────────────────────────────────
    // (a) handler reads merged store field by bare name
    // ─────────────────────────────────────────────────────────────────────

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan622_a_handler_reads_store_field_bare() {
        let mut dc = match build_app("src/front/app.at") {
            Some(c) => c,
            None => {
                eprintln!("plan622: SKIPPED — corpus app.at not found");
                return;
            }
        };
        dc.on_with_input("ProbeRead", None);
        let raw = state_raw(&dc, "probe_count");
        eprintln!("plan622(a) probe_count = {}", raw);
        assert!(
            raw.contains("41"),
            "(a) handler bare read of store field must see the seeded 41; \
             got {} (sentinel means the handler context does not resolve \
             merged store fields)",
            raw
        );
    }

    /// (a-nested) widget handler reads a store array OBJECT field mutated
    /// through a retrieved reference (the jade tabs_store .Open shape).
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan622_a2_handler_reads_store_nested_field_after_reference_write() {
        let mut dc = match build_app("src/front/app.at") {
            Some(c) => c,
            None => {
                eprintln!("plan622: SKIPPED — corpus app.at not found");
                return;
            }
        };
        dc.on_with_input("Touch", None);
        dc.on_with_input("ProbeNested", None);
        let raw = state_raw(&dc, "probe_body");
        eprintln!("plan622(a2) probe_body = {}", raw);
        assert!(
            raw.contains("mutated"),
            "(a-nested) handler read of a store object field (written through \
             a retrieved reference) must see 'mutated'; got {}",
            raw
        );
    }

    /// (a-arg) cross-facade store msg dispatch WITH a map argument (the jade
    /// `Tabs.Open({ path, title })` shape — msg payloads are single-typed
    /// maps). A silent dispatch failure leaves store state at model defaults,
    /// which is the observed downstream sentinel (0/"") family.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan622_a3_store_msg_dispatch_with_map_arg() {
        let mut dc = match build_app("src/front/app.at") {
            Some(c) => c,
            None => {
                eprintln!("plan622: SKIPPED — corpus app.at not found");
                return;
            }
        };
        dc.on_with_input("GrowFive", None);
        let raw = state_raw(&dc, "count");
        eprintln!("plan622(a3) count after GrowFive = {}", raw);
        assert!(
            raw.contains("46"),
            "(a-arg) store msg dispatch with a map arg must apply the delta \
             (41 + 5 = 46); got {} (stale 41 means the dispatch silently \
             failed and state stayed at defaults)",
            raw
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // (b) view text bound to store field follows store-msg mutation
    // ─────────────────────────────────────────────────────────────────────

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan622_b_view_follows_store_field_update() {
        let mut dc = match build_app("src/front/app.at") {
            Some(c) => c,
            None => {
                eprintln!("plan622: SKIPPED — corpus app.at not found");
                return;
            }
        };
        let initial = snapshot_of(&dc);
        assert!(
            initial.contains("41"),
            "(b) view should render seeded store count 41; got:\n{}",
            initial
        );
        assert!(
            initial.contains("x"),
            "(b) view should render the seeded store page body 'x'; got:\n{}",
            initial
        );
        dc.on_with_input("Bump", None);
        dc.on_with_input("Touch", None);
        let count_after = state_raw(&dc, "count");
        eprintln!("plan622(b) count after Bump = {}", count_after);
        let followed = snapshot_of(&dc);
        eprintln!("plan622(b) snapshot after Bump+Touch:\n{}", followed);
        assert!(
            followed.contains("42"),
            "(b) view must follow the store-field mutation (42 after Bump); \
             frozen at 41 means the view reads the stale load-time merge"
        );
        assert!(
            followed.contains("mutated"),
            "(b) view must follow the store reference-write (page body \
             'mutated' after Touch); still 'x' means the view reads the \
             stale load-time merge"
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // (c) #[api] call inside the store module — split-mode rewrite
     // ─────────────────────────────────────────────────────────────────────

    /// (c-runtime) end-to-end split-mode arm: build the corpus app with
    /// `api_over_http = true`, point AUTO_BACKEND at a local mock server,
    /// dispatch the widget msg that reaches the STORE module's #[api] call,
    /// and assert the HTTP request actually arrives. Env vars are read at
    /// codegen time inside the build, so the test serializes on a mutex.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan622_c2_store_module_api_call_reaches_backend_runtime() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::{Arc, Mutex};

        static ENV_LOCK: Mutex<()> = Mutex::new(());

        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        // 1. Mock backend: capture the first request line, always 200 "{}".
        let listener = TcpListener::bind("127.0.0.1:0").expect("mock bind");
        let port = listener.local_addr().unwrap().port();
        let captured: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));
        let cap = captured.clone();
        let stop_flag = stop.clone();
        let server = std::thread::spawn(move || {
            listener
                .set_nonblocking(false)
                .ok();
            while !stop_flag.load(Ordering::SeqCst) {
                let (mut stream, _) = match listener.accept() {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf).unwrap_or(0);
                if n > 0 {
                    let req = String::from_utf8_lossy(&buf[..n]).to_string();
                    let line = req.lines().next().unwrap_or("").to_string();
                    *cap.lock().unwrap() = Some(line);
                }
                let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}";
                stream.write_all(resp.as_bytes()).ok();
            }
        });

        // 2. Point the 340 rewrite at the mock and build in split mode.
        let old_backend = std::env::var("AUTO_BACKEND").ok();
        std::env::set_var("AUTO_BACKEND", format!("http://127.0.0.1:{}", port));
        let build = build_app("src/front/app.at");
        let mut dc = match build {
            Some(c) => c,
            None => {
                if let Some(v) = old_backend {
                    std::env::set_var("AUTO_BACKEND", v);
                } else {
                    std::env::remove_var("AUTO_BACKEND");
                }
                stop.store(true, Ordering::SeqCst);
                let _ = std::net::TcpStream::connect(("127.0.0.1", port));
                let _ = server.join();
                panic!("(c-runtime) split-mode build must succeed");
            }
        };

        // 3. Dispatch the widget msg that reaches the STORE module's #[api]
        //    call (App.StoreSave → store.Save() → save_note(.count)).
        dc.on_with_input("StoreSave", None);

        // 4. Restore env, stop the mock.
        if let Some(v) = old_backend {
            std::env::set_var("AUTO_BACKEND", v);
        } else {
            std::env::remove_var("AUTO_BACKEND");
        }
        stop.store(true, Ordering::SeqCst);
        let _ = std::net::TcpStream::connect(("127.0.0.1", port));
        server.join().ok();

        let hit = captured.lock().unwrap().clone();
        eprintln!("plan622(c-runtime) captured request = {:?}", hit);
        assert!(
            hit.as_deref().map(|l| l.contains("POST /api/notes/save")).unwrap_or(false),
            "(c-runtime) the store-module #[api] call must reach the backend \
             in split mode; captured = {:?} (None means the call executed the \
             in-module stub without HTTP)",
            hit
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // (d) widget-model array splice writes back
    // ─────────────────────────────────────────────────────────────────────

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan622_d_widget_model_array_splice_writes_back() {
        let mut dc = match build_app("src/front/app.at") {
            Some(c) => c,
            None => {
                eprintln!("plan622: SKIPPED — corpus app.at not found");
                return;
            }
        };
        dc.on_with_input("Splice", None);
        let state = dc.read_all_state_materialized();
        let items_raw = state
            .get("items")
            .map(|v| format!("{:?}", v))
            .unwrap_or_default();
        eprintln!("plan622(d) items after splice = {}", items_raw);
        assert!(
            items_raw.contains("\"b\"") && items_raw.contains("\"c\"") && !items_raw.contains("\"a\""),
            "(d) splice(0, 1) on widget-model array must drop the first item; \
             got {} (unchanged means the mutation never wrote back)",
            items_raw
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // (e1) lambda captures a handler-local var
    // ─────────────────────────────────────────────────────────────────────

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan622_e1_lambda_captures_handler_local_var() {
        let mut dc = match build_app("src/front/lambda_app.at") {
            Some(c) => c,
            None => panic!(
                "(e1) lambda_app.at must BUILD: a lambda capturing a handler \
                 local var currently fails handler codegen with 'undefined \
                 variable' — the corpus must compile"
            ),
        };
        dc.on_with_input("FindLocal", None);
        let status = state_raw(&dc, "status");
        eprintln!("plan622(e1) status = {}", status);
        assert!(
            status.contains("hit-b"),
            "(e1) lambda capturing a handler-local var must find the match; \
             got {}",
            status
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // (e2) lambda compares against a self field
    // ─────────────────────────────────────────────────────────────────────

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan622_e2_lambda_compares_self_field() {
        let mut dc = match build_app("src/front/lambda_app.at") {
            Some(c) => c,
            None => {
                eprintln!("plan622: SKIPPED — lambda_app.at not found");
                return;
            }
        };
        dc.on_with_input("FindSelf", None);
        let status = state_raw(&dc, "status");
        eprintln!("plan622(e2) status = {}", status);
        assert!(
            status.contains("self-b"),
            "(e2) lambda comparing against a self field must find the match; \
             got {} (status unchanged means the self-field read silently \
             failed inside the lambda)",
            status
        );
    }

    // Silence the unused import warning when Value is only used by helpers.
    #[allow(unused)]
    fn _value_witness(_: Value) {}
}
