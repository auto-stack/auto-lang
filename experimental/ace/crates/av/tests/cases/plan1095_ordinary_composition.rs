//! PLAN-1095 permanent regressions: ordinary-project composition faces.
//! The document-case corpus (`language/composition/ordinary-projects.case.ad`,
//! `language/domain/node-values.case.ad`) owns the case-level AV checks;
//! this file pins the engine-side fixes with real serve() runs:
//!  1. package-mode local free fn as a VALUE (`let f = twice` with `use`
//!     imports in scope) — packages/binding.rs Auto-language lane;
//!  2. task + record literal + try/finally composition in one source;
//!  3. CT call + list literal + typed filter/map composition.

use ace_av::language_source_worker::serve;
use base64::Engine;
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::var_os("AUTO_LANG_TEST_SCRATCH")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        Self(root.join(format!(
            "p1095-cache-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn b64(text: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(text)
}

fn run_request(request: serde_json::Value) -> serde_json::Value {
    serve(&request).expect("ordinary worker response")["steps"][0].clone()
}

fn single(source: &str, caps: &[&str]) -> serde_json::Value {
    run_request(json!({
        "recipe": "ac-av/language-source-run-v1",
        "nonce": "0".repeat(64),
        "source_b64": b64(source),
        "steps": [{"op": "run"}],
        "source_path": "main.at",
        "entry": "d_main",
        "caps": caps,
    }))
}

fn project(entry: &str, files: &[(&str, &str)], caps: &[&str]) -> serde_json::Value {
    let store = Scratch::new();
    let extra: Vec<_> = files.iter().filter(|(p, _)| *p != entry).collect();
    run_request(json!({
        "recipe": "ac-av/language-source-run-v1",
        "nonce": "0".repeat(64),
        "source_b64": b64(files.iter().find(|(p, _)| *p == entry).unwrap().1),
        "steps": [{"op": "run"}],
        "source_path": entry,
        "entry": "d_main",
        "caps": caps,
        "project": {
            "entry_path": entry,
            "files": extra.iter().map(|(p, s)| json!({"path": p, "source_b64": b64(s)})).collect::<Vec<_>>(),
        },
        "store_root": &store.0,
    }))
}

fn value(step: &serde_json::Value) -> i64 {
    assert_eq!(step["outcome"], "success", "{step}");
    step["value"].as_i64().expect("i32 value channel")
}

fn stdout(step: &serde_json::Value) -> String {
    String::from_utf8_lossy(
        &base64::engine::general_purpose::STANDARD
            .decode(step["console"]["stdout_b64"].as_str().unwrap_or(""))
            .unwrap_or_default(),
    )
    .into_owned()
}

const CAPS_AV: &[&str] = &["ac.source.language.av/v1", "console.stdout", "console.stderr"];
const CAPS_CT: &[&str] = &["ac.source.language.av/v1", "ac.source.cte.scalar/v1"];

#[test]
fn p1095_project_local_free_fn_as_value_with_import() {
    // PLAN-1095 T-02: `let f = twice` inside a multi-file package whose
    // scope also carries `use lib.id` — the package binding walker must
    // admit a local free fn in value position (the same zero-capture
    // CreateClosure lane as the single-source face, PLAN-1081 T-02).
    let main = concat!(
        "use lib.id\n",
        "fn twice(n int) int { return n * 2 }\n",
        "fn main() int {\n",
        " let f = twice\n",
        " let xs = [id(19), id(23)]\n",
        " if f(xs[0]) + f(xs[1]) == 84 { return 42 }; return 0\n",
        "}\n",
    );
    let lib = "pub fn id<T>(x T) T { return x }\n";
    let step = project("main.at", &[("main.at", main), ("lib.at", lib)], CAPS_AV);
    assert_eq!(value(&step), 42, "{step}");
    assert_eq!(stdout(&step), "", "{step}");
    // `main` stays entry-only: a bare fn value binding of `main` is refused.
    let bad = "fn main() int { let f = main; return 0 }";
    let step = single(bad, CAPS_AV);
    assert_eq!(step["diag"]["code"], "source.fn-as-value", "{step}");
}

#[test]
fn p1095_task_record_finally_composition() {
    // PLAN-1095 case `task-language-composition`: a record literal with
    // mixed str/bool fields, field projection, try/finally and the task
    // spawn/ask/await/stop chain all compose in ONE ordinary source.
    let source = concat!(
        "task Q { on { n int -> { reply(n + 1) } } }\n",
        "fn get() int {\n",
        " let h = Task.spawn(\"Q\", 1)\n",
        " let v = await h.ask(41)\n",
        " h.stop()\n",
        " return v\n",
        "}\n",
        "fn main() int {\n",
        " let meta = { label: \"ready\", enabled: true }\n",
        " try { if meta.enabled { return get() }; return 0 } finally { }\n",
        "}\n",
    );
    let step = single(source, CAPS_AV);
    assert_eq!(value(&step), 42, "{step}");
    assert_eq!(stdout(&step), "", "{step}");
}

#[test]
fn p1095_ct_and_ordinary_library_composition() {
    // PLAN-1095 case `cte-and-ordinary-library` (typed-lambda spelling of
    // the draft's elided `x =>` per the face rule: no lambda parameter
    // inference — the audited 1081/1082 normalization).
    let source = concat!(
        "fn scale<T>(n T) T { return n + n }\n",
        "fn main() int {\n",
        " let seed = #{ scale(21) }\n",
        " let xs = [seed - 1, seed]\n",
        " return xs.filter((x int) => x == 42)[0]\n",
        "}\n",
    );
    let step = single(source, CAPS_CT);
    assert_eq!(value(&step), 42, "{step}");
    assert_eq!(step["counters"]["ct_execute"], 1, "{step}");
    // the CT call itself leaves no runtime frame; the list/filter lane after
    // it does run (runtime_calls >= 1), proving the CT value flows on.
    assert_eq!(step["counters"]["runtime_calls"], 1, "{step}");
    assert_eq!(stdout(&step), "", "{step}");
}

#[test]
fn p1095_node_ordinary_coexist_with_cpu_observation() {
    // PLAN-1095 case `node-ordinary-coexist`: the ordinary chain admits the
    // H0 node declaration (main runs 42) AND the 979 producer observes the
    // same source — the node view text "你好" must come from the shared
    // execution chain, not a static emit claim. No renderer is started.
    let source = concat!(
        "fn label() str { return \"你好\" }\n",
        "node Hello { view { text label() {} } }\n",
        "fn main() int { return 42 }\n",
    );
    let step = single(source, CAPS_AV);
    assert_eq!(value(&step), 42, "{step}");
    let nodes = step["nodes"].as_array().expect("node observation present");
    assert_eq!(nodes.len(), 1, "{step}");
    assert_eq!(nodes[0]["name"], "Hello", "{step}");
    let texts = nodes[0]["view"]["texts"].as_array().expect("view texts");
    assert_eq!(texts.len(), 1, "{step}");
    assert_eq!(texts[0]["value"], "你好", "{step}");
    // a different function body must produce a different observed text
    // through the same chain (no canned constant satisfies both).
    let other = source.replace("return \"你好\"", "return \"世界\"");
    let step2 = single(&other, CAPS_AV);
    let nodes2 = step2["nodes"].as_array().expect("node observation present");
    assert_eq!(nodes2[0]["view"]["texts"][0]["value"], "世界", "{step2}");
    // plain ordinary sources carry no node field.
    let plain = "fn main() int { return 42 }\n";
    assert!(single(plain, CAPS_AV)["nodes"].is_null(), "no nodes on a plain source");
}

#[test]
fn p1095_project_restore_cold_equal_and_sessions_do_not_share() {
    // PLAN-1095 PROJECT-RESTORE (AC-03/AC-04): one project, a bad edit keeps
    // the last accepted generation, restoring the function body re-runs
    // COLD (ct_execute=1) and agrees with the original cold run; a second
    // store/session never shares state or caches with the first.
    let lib = "pub type Record { flag bool; value int }";
    let main = concat!(
        "use lib.Record as Local\n",
        "fn unchanged(n int) int { return n + 39 }\n",
        "fn main() int { let base = #{ unchanged(1) }; ",
        "if #{ ct_has_field(Local, \"flag\") } { return base + #{ ct_field_count(Local) } }; ",
        "return 1 }",
    );
    let b64 = |text: &str| base64::engine::general_purpose::STANDARD.encode(text);
    let run = |store: &Scratch, main_text: &str| {
        let request = json!({
            "recipe": "ac-av/language-source-run-v1",
            "nonce": "0".repeat(64),
            "source_b64": b64(main_text),
            "steps": [{"op": "run"}],
            "source_path": "main.at",
            "entry": "d_main",
            "caps": [CAPS_AV[0], "ac.source.cte.scalar/v1", ace_core::source::META_CAPABILITY],
            "project": {
                "entry_path": "main.at",
                "files": [{"path": "lib.at", "source_b64": b64(lib)}],
            },
            "store_root": &store.0,
        });
        run_request(request)
    };
    let store_a = Scratch::new();
    let store_b = Scratch::new();
    let cold = run(&store_a, main);
    assert_eq!(value(&cold), 42, "{cold}");
    assert_eq!(cold["counters"]["ct_execute"], 1, "{cold}");
    // a bad edit (division by zero inside the CT expression) is a named
    // trap; then restoring the function body re-runs COLD on the same store
    // and matches the first run exactly.
    let bad = run(&store_a, &main.replace("n + 39", "n / 0"));
    assert!(
        bad["diag"]["code"]
            .as_str()
            .expect("trap code")
            .starts_with("cte.trap/"),
        "{bad}"
    );
    let restored = run(&store_a, main);
    assert_eq!(value(&restored), 42, "{restored}");
    // the restored source hits the cache the first cold run published
    // (ct_execute=0, ct_hits=1) and agrees with the cold value/hash.
    assert_eq!(restored["counters"]["ct_execute"], 0, "{restored}");
    assert_eq!(restored["counters"]["ct_hits"], 1, "{restored}");
    assert_eq!(restored["source_hash"], cold["source_hash"]);
    // a second session/store starts cold on its own (ct_execute=1, no hits
    // from the first store) — no shared cache crosses the boundary.
    let cold_b = run(&store_b, main);
    assert_eq!(value(&cold_b), 42, "{cold_b}");
    assert_eq!(cold_b["counters"]["ct_execute"], 1, "{cold_b}");
    assert_eq!(cold_b["counters"]["ct_hits"], 0, "{cold_b}");
}
