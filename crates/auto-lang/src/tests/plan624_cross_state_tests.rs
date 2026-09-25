//! PLAN-624 red corpus: merged single-state cross-state field resolution
//! (jade facade-switch live findings, DEBTS 064 row ⑥ refinement).
//!
//! Arms (each maps to a Plan face):
//! - P1 `plan624_p1_...`: widget handler reads store fields after a store
//!   msg dispatch (map-literal dispatch + `.tabs.find` + mirror writes) —
//!   jade crashed with `Field 'active_path' not found on App_State`.
//!   Run in BOTH modes: split (jade's crashing configuration) and merged
//!   (matrix data point for the T-01 divergence conclusion).
//! - P2 `plan624_p2_...`: `?str` store field read inside a map-literal
//!   argument — jade crashed with `Invalid object ID` (0x80000001
//!   sign-extension).
//! - P3 `plan624_p3_...`: `&&`/`||` chains on struct operands must be a
//!   compile-time error (agreed ruling; currently they evaluate as boolean
//!   logic and silently write `true` into string fields). Asserted at the
//!   synthesize layer: the corpus must FAIL synthesis with the diagnostic.
//! - P4 `plan624_p4_...`: `find_index` on a widget-model list must return
//!   the first matching index (currently silently no-ops — no native).
//!
//! Corpus: `test/ui/plan624_cross_state/` — app.at + tab_store.at (str
//! sentinel, P1), p2_app.at + p2_tab_store.at (?str, isolated root),
//! p3_chain_app.at (isolated P3 root), p4_app.at (P4), back/api.at
//! (contract stubs).
//!
//! Red = P1/P2/P4 assertions fail on master (crash / wrong value); P3 red =
//! synthesis succeeds without the diagnostic. Green = T-02..T-05 fixes.

#[cfg(test)]
mod plan624_cross_state_tests {
    use crate::plan370_test_support::{build_component_from_app, build_component_from_app_mode};

    fn locate(rel: &str) -> Option<std::path::PathBuf> {
        let full = format!("test/ui/plan624_cross_state/{}", rel);
        [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(&full)),
            Some(std::path::PathBuf::from(&full)),
            Some(std::path::PathBuf::from(format!("../../{}", full))),
        ]
        .into_iter()
        .flatten()
        .find(|p| p.exists())
    }

    fn state_raw(dc: &crate::ui::dynamic::DynamicComponent, field: &str) -> String {
        match dc.read_state(field) {
            Ok(v) => format!("{:?}", v),
            Err(e) => format!("READ_ERR({})", e),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // (P1) widget handler reads store fields after store-msg dispatch
    // ─────────────────────────────────────────────────────────────────────

    fn p1_assert(dc: &mut crate::ui::dynamic::DynamicComponent, mode_name: &str) {
        dc.on_with_input("OpenPage", Some("Hello World.ad".to_string()));
        // PLAN-702 段驱动适配：OpenPage 内含 api 调用即 park——驱动恢复泵
        // 至落账（生产 = __parked_resume_tick 泵）。
        crate::plan370_test_support::drive_parked_segments(dc, |dc| {
            let t = state_raw(dc, "view_title");
            let s = state_raw(dc, "status");
            t.contains("Hello World") && s.contains("opened")
        }, "plan624 P1 cross-state read");
        let title = state_raw(dc, "view_title");
        let status = state_raw(dc, "status");
        eprintln!("plan624(P1/{}) view_title={} status={}", mode_name, title, status);
        assert!(
            title.contains("Hello World") && status.contains("opened"),
            "(P1/{}) widget handler must read store state after dispatch: \
             view_title should be the strip_ext title and status opened; \
             got view_title={} status={} (Field-not-found crash means the \
             GET_FIELD resolved against App_State instead of the merged \
             store state)",
            mode_name,
            title,
            status
        );
    }

    /// P1 in split mode — jade vm-smoke's configuration (the crashing one):
    /// live backend (local mock) so the interleaved #[api] call after the
    /// store reads succeeds, mirroring the jade OpenFile shape.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan624_p1_split_mode_cross_state_read() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::{Arc, Mutex};

        static ENV_LOCK: Mutex<()> = Mutex::new(());

        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        let listener = TcpListener::bind("127.0.0.1:0").expect("mock bind");
        let port = listener.local_addr().unwrap().port();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();
        let server = std::thread::spawn(move || {
            let _ = listener.set_nonblocking(false);
            while !stop_flag.load(Ordering::SeqCst) {
                let (mut stream, _) = match listener.accept() {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}";
                let _ = stream.write_all(resp.as_bytes());
            }
        });

        let Some(manifest) = locate("src/front/app.at") else {
            stop.store(true, Ordering::SeqCst);
            let _ = std::net::TcpStream::connect(("127.0.0.1", port));
            let _ = server.join();
            eprintln!("plan624: SKIPPED — corpus app.at not found");
            return;
        };
        let old_backend = std::env::var("AUTO_BACKEND").ok();
        std::env::set_var("AUTO_BACKEND", format!("http://127.0.0.1:{}", port));
        let build = build_component_from_app_mode(&manifest, true);
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
                panic!("(P1/split) corpus app.at must build");
            }
        };
        p1_assert(&mut dc, "split");
        if let Some(v) = old_backend {
            std::env::set_var("AUTO_BACKEND", v);
        } else {
            std::env::remove_var("AUTO_BACKEND");
        }
        stop.store(true, Ordering::SeqCst);
        let _ = std::net::TcpStream::connect(("127.0.0.1", port));
        server.join().ok();
    }

    /// P1 matrix data point: merged mode (api_over_http = false).
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan624_p1_merged_mode_cross_state_read() {
        let Some(manifest) = locate("src/front/app.at") else {
            eprintln!("plan624: SKIPPED — corpus app.at not found");
            return;
        };
        let mut dc = build_component_from_app_mode(&manifest, false)
            .expect("(P1/merged) corpus app.at must build");
        p1_assert(&mut dc, "merged");
    }

    // ─────────────────────────────────────────────────────────────────────
    // (P2) ?str store field read inside a map-literal argument
    // ─────────────────────────────────────────────────────────────────────

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan624_p2_opt_str_cross_state_read() {
        let Some(manifest) = locate("src/front/p2_app.at") else {
            eprintln!("plan624: SKIPPED — corpus p2_app.at not found");
            return;
        };
        let mut dc = build_component_from_app(&manifest)
            .expect("(P2) corpus p2_app.at must build");

        // F-01: None-state ?str cross-state read — dispatched BEFORE any
        // Open, so .active_path is still None. The read must not crash and
        // the value must land in the ?str model field (nil form).
        dc.on_with_input("ProbeNone", None);
        let probe_none = state_raw(&dc, "probe_none");
        let status_none = state_raw(&dc, "status");
        eprintln!("plan624(P2/F-01) probe_none={} status={}", probe_none, status_none);
        assert!(
            status_none.contains("probed-none")
                && probe_none.contains("Nil")
                && !probe_none.contains("READ_ERR"),
            "(P2/F-01) reading the store's ?str field in the None state must \
             not crash and must land nil in the ?str model field; got \
             probe_none={} status={} (Invalid-object-ID crash or a READ_ERR \
             means the None-state cross-state read broke)",
            probe_none,
            status_none
        );

        dc.on_with_input("OpenPage", Some("Hello World.ad".to_string()));
        dc.on_with_input("Edit", None);
        let dirty = state_raw(&dc, "view_dirty");
        let status = state_raw(&dc, "status");
        eprintln!("plan624(P2) view_dirty={} status={}", dirty, status);
        assert!(
            dirty.contains("true") && status.contains("edited"),
            "(P2) ?str store field read in a map-literal arg must not crash \
             and the SetBody dirty computation must land; got dirty={} \
             status={} (Invalid-object-ID crash means the ?str encoding \
             broke across the state boundary)",
            dirty,
            status
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // (P3) &&/|| on struct operands — compile-time error ruling
    // ─────────────────────────────────────────────────────────────────────

    /// P3 (rev 2 裁定②): JS value semantics — the &&/|| chain returns the
    /// OPERAND VALUE (short-circuit pass-through), so the web idiom
    /// `fm && fm.title || "fallback"` resolves to the fallback STRING on a
    /// falsy map field instead of the boolean corruption (`true`) the old
    /// bool-normalizing emission produced.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan624_p3_nonbool_chain_value_semantics() {
        let Some(manifest) = locate("src/front/p3_chain_app.at") else {
            eprintln!("plan624: SKIPPED — corpus p3_chain_app.at not found");
            return;
        };
        let mut dc = build_component_from_app(&manifest)
            .expect("(P3) corpus p3_chain_app.at must build");
        dc.on_with_input("Probe", None);
        let status = state_raw(&dc, "status");
        for f in ["hop_a", "hop_b", "hop_c"] {
            eprintln!("plan624(P3) {} = {}", f, state_raw(&dc, f));
        }
        eprintln!("plan624(P3) status = {}", status);
        assert!(
            status.contains("fallback") && !status.contains("Bool"),
            "(P3) the &&/|| chain must pass the fallback STRING through on a              falsy map field; got {} (Bool = the old bool-normalizing              corruption)",
            status
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // (P4) find_index returns the first matching index
    // ─────────────────────────────────────────────────────────────────────

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan624_p4_find_index_returns_first_match() {
        let Some(manifest) = locate("src/front/p4_app.at") else {
            eprintln!("plan624: SKIPPED — corpus p4_app.at not found");
            return;
        };
        let mut dc = build_component_from_app(&manifest)
            .expect("(P4) corpus p4_app.at must build");
        dc.on_with_input("Probe", None);
        let idx = state_raw(&dc, "idx");
        let idx_miss = state_raw(&dc, "idx_miss");
        let status = state_raw(&dc, "status");
        eprintln!("plan624(P4) idx={} idx_miss={} status={}", idx, idx_miss, status);
        assert!(
            idx.contains("1") && status.contains("probed"),
            "(P4) find_index(n => n == 8) on [7, 8, 9] must return 1; \
             got idx={} (sentinel means the call silently no-ops — no \
             native)",
            idx
        );
        // F-02: miss path — 42 is absent, must land -1 (the -99 sentinel
        // would mean the call silently no-op'd; "-99" does not contain
        // "-1", so this assertion also discriminates the no-op case).
        assert!(
            idx_miss.contains("-1"),
            "(P4/F-02) find_index(n => n == 42) on [7, 8, 9] must return -1 \
             on miss; got idx_miss={} (sentinel -99 means the call silently \
             no-op'd)",
            idx_miss
        );
    }
}
