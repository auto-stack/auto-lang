//! Plan 442 B-support regression tests: web-platform globals bridged on the
//! VM render target (musk corpus runs single-source in both render modes).
//!
//! - `localStorage.getItem/setItem/removeItem` → the Plan 401 session KV
//!   store, with the browser's None-on-miss semantics (musk's AuthStore
//!   tests `saved != None`).
//! - `encodeURIComponent` (bare JS global) → the VM's percent-encoding
//!   native (also completing the ID-map-only `auto.url.encode` entry with
//!   a real shim binding).
//!
//! Corpus: `test/ui/plan442_webcompat/` mirrors the musk call shapes.

#[cfg(test)]
mod plan442_webcompat_tests {
    fn locate_corpus() -> Option<std::path::PathBuf> {
        let rel = "test/ui/plan442_webcompat/src/front/app.at";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ];
        candidates.into_iter().flatten().find(|p| p.exists())
    }

    fn build() -> Option<crate::ui::dynamic::DynamicComponent> {
        crate::plan370_test_support::build_component_from_app(&locate_corpus()?)
    }

    fn state_str(dc: &crate::ui::dynamic::DynamicComponent, field: &str) -> String {
        match dc.read_state(field) {
            Ok(auto_val::Value::Str(s)) => s.as_str().to_string(),
            Ok(auto_val::Value::Bool(b)) => b.to_string(),
            Ok(other) => format!("{:?}", other),
            Err(e) => panic!("read_state('{}') failed: {}", field, e),
        }
    }

    /// REGRESSION: localStorage round-trip with None-on-miss semantics —
    /// the exact shapes musk's AuthStore uses.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn localstorage_bridge_roundtrip() {
        let mut dc = match build() {
            Some(c) => c,
            None => {
                eprintln!("plan442: SKIPPED — corpus app.at not found");
                return;
            }
        };
        // Init: key absent → getItem returns None → restored stays false.
        assert_eq!(state_str(&dc, "restored"), "false");
        // Save then Load: value round-trips through the bridge.
        dc.on_with_input("Save", None);
        dc.on_with_input("Load", None);
        assert_eq!(state_str(&dc, "token"), "t-ok");
        assert_eq!(state_str(&dc, "restored"), "false", "Init ran before Save");
        // Drop then Load: missing again → token unchanged.
        dc.on_with_input("Drop", None);
        dc.on_with_input("Load", None);
        assert_eq!(
            state_str(&dc, "token"),
            "t-ok",
            "after removeItem, getItem must return None (token untouched)"
        );
    }

    /// REGRESSION: the bare JS global encodeURIComponent percent-encodes
    /// (musk's forge_store builds SSE URLs with it — an unresolved symbol
    /// here was the link-fatal blocker of the first musk VM probe).
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn encodeuricomponent_bridge() {
        let mut dc = match build() {
            Some(c) => c,
            None => {
                eprintln!("plan442: SKIPPED — corpus app.at not found");
                return;
            }
        };
        dc.on_with_input("Url", None);
        let url = state_str(&dc, "url");
        assert_eq!(
            url, "/api/chats/session/id%2042%2F%2B/stream",
            "encodeURIComponent must percent-encode spaces, slashes and plus (JS parity)"
        );
    }
}
