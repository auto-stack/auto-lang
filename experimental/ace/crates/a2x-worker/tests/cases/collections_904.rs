//! PLAN-904 T-02 minimal true chain (S+P): the five required first cases
//! through the NEW collections Rust channel — `compile_collections_source`
//! → `verify_collections` (with the collections body facts) →
//! `SemanticView::from_checked` → `lower_collections_program` →
//! `assemble_collections_file_set` — certified by REAL rustc compile+run
//! against the independent hand-computed oracles (plan §4.3): L04=42,
//! L05=42, L06=`trap:ListIndex` (the ONLY cause — never value 70, a panic
//! or any other nonzero), V03=3, N889-nested-variant=42.
//!
//! Sources are the frozen ace-m1 originals, byte-exact.

use std::path::PathBuf;
use std::process::Command;

use ace_a2x_rust::coll::{assemble_collections_file_set, COLL_FILESET_SCHEMA};
use ace_a2x_rust::lower_coll::lower_collections_program;
use ace_a2x_core::sha256::sha256_hex;
use ace_a2x_core::SemanticView;
use ace_core::source::compile_collections_source;

fn out_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out/collections-904")
}

fn scratch(name: &str) -> PathBuf {
    let dir = out_base().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Compile + run a file set with real rustc; returns stdout.
fn run_fileset(name: &str, files: &[(String, String)]) -> String {
    let dir = scratch(name);
    for (p, c) in files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    let exe = dir.join("case.exe");
    let compile = auto_tool_execution::run_command_capture(
        {
            let mut c = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()));
            c.args(["--edition", "2021", "-o"])
                .arg(&exe)
                .arg("main.rs")
                .current_dir(&dir);
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("rustc spawn");
    assert_eq!(
        compile.kind,
        auto_tool_execution::ExitKind::Success,
        "rustc failed for {name}: {}",
        String::from_utf8_lossy(compile.stderr.as_ref().unwrap())
    );
    let run = auto_tool_execution::run_command_capture(
        {
            let mut c = Command::new(&exe);
            c.current_dir(&dir);
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run spawn");
    String::from_utf8_lossy(run.stdout.as_ref().expect("captured stdout")).to_string()
}

fn emit_files(src: &str) -> Vec<(String, String)> {
    let compiled = compile_collections_source(src, "m.at", &[]).expect("compile collections source");
    let view = SemanticView::from_checked(&compiled.checked).expect("view");
    let program = lower_collections_program(&view, "main").expect("lower");
    let config = "default";
    let _ = config;
    let set = assemble_collections_file_set(
        &view,
        &program,
        "input-hash",
        &sha256_hex(src.as_bytes()),
        "{\"budget_work_units\":16777216,\"cancel\":false,\"events\":false,\"fault_alloc\":null,\"mode\":\"debug\"}",
        "main",
    );
    assert!(set.manifest.contains(COLL_FILESET_SCHEMA));
    set.files
}

const L04: &str = "type Item { name Str; code int }\nfn main() int {\n    var xs = List<Item>.new()\n    let x Item = Item(\"hello\", 37)\n    xs.push(x.move)\n    return xs.get(0).name.len() + xs.get(0).code\n}";

const L05: &str = "type Item { name Str; code int }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"hello\", 37))\n    let x Item = xs.remove(0)\n    return x.name.len() + x.code + xs.len()\n}";

const L06: &str = "fn main() int {\n    var xs = List<int>.new()\n    xs.push(42)\n    return xs.get(1)\n}";

const V03: &str = "fn fail() Result<int, Str> { return Err(\"bad\") }\nfn main() int {\n    let r = fail()\n    is r {\n        Ok(v) -> { return v }\n        Err(e) -> { return e.len() }\n    }\n}";

const N889_NESTED: &str = "enum Outer { Wrap(Inner)\nEmpty }\nenum Inner { Data(Item)\nEnd }\ntype Item { name Str; code int }\nfn make() Outer { return Outer.Wrap(Inner.Data(Item(\"中\", 41))) }\nfn main() int {\n    var xs = List<Outer>.new()\n    let created = make()\n    xs.push(created.move)\n    let o Outer = xs.remove(0)\n    is o {\n        Outer.Wrap(inner) -> {\n            is inner {\n                Inner.Data(v) -> { return v.name.len() + v.code }\n                Inner.End -> { return 0 }\n            }\n        }\n        Outer.Empty -> { return 0 }\n    }\n}";

const LSTR864: &str = "fn main() int {\n    var xs = List<Str>.new()\n    xs.push(\"hello\")\n    xs.push(\"中\")\n    let removed Str = xs.remove(0)\n    return removed.len() * 10 + xs.get(0).len()\n}";

#[test]
fn coll_904_min_true_chain_values() {
    assert_eq!(run_fileset("L04", &emit_files(L04)), "value:42\n");
    assert_eq!(run_fileset("L05", &emit_files(L05)), "value:42\n");
    assert_eq!(run_fileset("V03", &emit_files(V03)), "value:3\n");
    // LSTR864 = 51 (independent hand computation: "hello".len()*10 = 50
    // + "中".len() = 1 — Str.len counts Unicode scalars)
    assert_eq!(run_fileset("LSTR864", &emit_files(LSTR864)), "value:51\n");
}

#[test]
fn coll_904_min_true_chain_trap() {
    // L06's ONLY outcome is the named list-index trap: never value 70, a
    // panic, or any other nonzero/exit shape.
    assert_eq!(run_fileset("L06", &emit_files(L06)), "trap:ListIndex\n");
}

#[test]
fn coll_904_nested_owned_variant() {
    assert_eq!(
        run_fileset("N889-nested", &emit_files(N889_NESTED)),
        "value:42\n"
    );
}

/// T-06 green: the STORE channel carries the Collections family —
/// cold assembly over AXS3/AXR3 with the collections fileset, certified
/// by a real rustc run (the trap case keeps its named identity through
/// the whole store round-trip).
#[test]
fn coll_904_store_channel_green() {
    let root = scratch("store-green");
    let mut s = ace_a2x_store::A2xSession::open(&root, "p904", "owner", 8 << 20).unwrap();
    let asm = s
        .cold_assemble_profile(L06, ace_core::source::SourceProfile::Collections, &[], "main", false)
        .expect("collections store family (T-06)");
    assert!(asm.manifest.contains("auto.a2x.rust.collections.fileset/1"));
    let dir = out_base().join("store-green-L06");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (p, c) in &asm.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &asm.manifest).unwrap();
    let exe = dir.join("case.exe");
    let compile = auto_tool_execution::run_command_capture(
        {
            let mut c = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()));
            c.args(["--edition", "2021", "-o"]).arg(&exe).arg("main.rs").current_dir(&dir);
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("rustc");
    assert_eq!(compile.kind, auto_tool_execution::ExitKind::Success);
    let run = auto_tool_execution::run_command_capture(
        { let mut c = Command::new(&exe); c.current_dir(&dir); c },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run");
    assert_eq!(
        String::from_utf8_lossy(run.stdout.as_ref().unwrap()),
        "trap:ListIndex
"
    );
}

// ---- V-13: credential binding (same owner/context, different body) ----

const TYPED_A: &str = "type Item { name Str; code int }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"hello\", 37))\n    return xs.get(0).name.len() + xs.get(0).code\n}";

const TYPED_B: &str = "type Item { name Str; code int }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"hello\", 38))\n    return xs.get(0).name.len() + xs.get(0).code\n}";

#[test]
fn coll_904_v13_same_context_different_body_rejected() {
    use ace_core::source::{
        emit_function_body, env_fn_names, parse_program_with, resolve_env, SourceProfile,
    };
    let caps: Vec<String> = Vec::new();
    let prog_a = parse_program_with(TYPED_A, SourceProfile::Collections).expect("parse A");
    let prog_b = parse_program_with(TYPED_B, SourceProfile::Collections).expect("parse B");
    let env_a = resolve_env(&prog_a, &caps, SourceProfile::Collections).expect("resolve A");
    let env_b = resolve_env(&prog_b, &caps, SourceProfile::Collections).expect("resolve B");
    let main_idx_b = env_fn_names(&env_b).iter().position(|n| *n == "main").expect("main in B");
    let main_idx_a = env_fn_names(&env_a).iter().position(|n| *n == "main").expect("main in A");
    let (parts_b, _receipt) =
        emit_function_body(&env_b, main_idx_b, TYPED_B, "m.at").expect("emit B fragment");
    let (parts_a, _receipt_a) =
        emit_function_body(&env_a, main_idx_a, TYPED_A, "m.at").expect("emit A fragment");
    // A's legitimate credential (same declaration context by construction)
    let def_a = parts_a
        .bundle
        .modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == "d_main")
        .expect("def main A");
    let cred_a =
        ace_core::verify::verify_function_ref(&parts_a.bundle, def_a).expect("credential A");
    // A's credential + B's parts: same owner, same declaration context,
    // DIFFERENT body — must be a NAMED rejection before any emit (V-13)
    let err = SemanticView::from_parts(&parts_b, &cred_a).expect_err("mixed credential");
    assert!(
        err.to_string().contains("body digest"),
        "expected body-digest rejection, got: {err}"
    );
    // B's own legitimate credential still admits (control)
    let def_b = parts_b
        .bundle
        .modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == "d_main")
        .expect("def main B");
    let cred_b =
        ace_core::verify::verify_function_ref(&parts_b.bundle, def_b).expect("credential B");
    let view = SemanticView::from_parts(&parts_b, &cred_b).expect("B's own credential admits");
    assert_eq!(view.functions().len(), 1);
}

// ---- typed-record-family warm fragment: the T-03 S-window red case ----
// (902's documented boundary: record-family `type` declarations' retained
// spans cannot re-anchor into a fragment document; this flips green when
// the shared S repair lands.)

// The type declaration sits LATE in the file so its retained span exceeds
// a short main-only fragment — the shape that actually trips today's
// re-anchor bound check (a span that merely fits inside the fragment
// length passes with silently wrong coordinates — the §5.2 full-source
// identity gap the shared T-03 repair closes).
const TR_TYPED_FULL: &str = "fn pad() int {\n    // a deliberately long padding body so the retained type span lands\n    // far beyond the length of a main-only fragment document\n    return 123456789\n}\ntype W { text Str }\nfn main() int {\n    let w W = W(\"ab\")\n    return w.text.len() * 10 + 2\n}";

const TR_TYPED_FRAG_B: &str = "fn main() int {\n    let w W = W(\"ab\")\n    return w.text.len()\n}";

/// T-03 S repair (green): the record/text-record family's TYPED sources
/// take the real warm FRAGMENT path again — the retained type declaration
/// is inert scaffold metadata in the fragment document, the edited
/// fragment re-produces exactly once, and the assembly value changes.
#[test]
fn coll_904_typed_record_warm_fragment_green() {
    let root = scratch("tr-warm-green");
    let mut s = ace_a2x_store::A2xSession::open(&root, "p904", "owner", 8 << 20).unwrap();
    let a = s
        .cold_assemble_profile(
            TR_TYPED_FULL,
            ace_core::source::SourceProfile::TextRecord,
            &["ac.runtime.text.v1".to_string()],
            "main",
            false,
        )
        .expect("cold typed text-record");
    // control: cold A value (w.text.len()*10+2 = "ab"→4)
    let dir = out_base().join("tr-warm-A");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (p, c) in &a.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &a.manifest).unwrap();
    let prints_a = s.stats.print;
    // the REAL fragment edit (single-declaration parse path)
    let b = s
        .warm_edit("main", TR_TYPED_FRAG_B, "main", false)
        .expect("typed warm fragment edit works after the T-03 repair");
    assert_eq!(s.stats.print, prints_a + 1, "exactly the edited fragment");
    let dir = out_base().join("tr-warm-B");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (p, c) in &b.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &b.manifest).unwrap();
}
