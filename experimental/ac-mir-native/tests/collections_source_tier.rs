//! Original byte-identical sources, independent hand-computed values, actual PE.
mod common;
use common::collections::{assert_balanced, run_source};

fn original(id: &str) -> String {
    std::fs::read_to_string(
        common::repo_root().join(format!("crates/auto-lang/test/cases/ace-m1/src/{id}.at")),
    )
    .unwrap()
}

#[test]
fn collections_original_owned_first_cases() {
    for (id, value) in [
        ("L04", 42),
        ("L05", 42),
        ("LSTR864", 51),
        ("V02", 42),
        ("V03", 3),
    ] {
        let run = run_source(&original(id), id);
        assert_eq!(
            run.exit_code,
            Some(value),
            "{id}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_balanced(&run.stderr);
    }
}

#[test]
fn collections_original_remaining_cases() {
    for (id, value) in [
        ("L01", 42),
        ("L02", 42),
        ("L03", 0),
        ("L07", 42),
        ("V01", 42),
        ("V04", 42),
        ("V05", 42),
        ("V06", 42),
        ("V07", 42),
        ("FIN864A", 42),
        ("FIN864B", 53),
    ] {
        let run = run_source(&original(id), id);
        assert_eq!(
            run.exit_code,
            Some(value),
            "{id}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_balanced(&run.stderr);
    }
}

#[test]
fn collections_list_variant_and_nested_cross_function_sources() {
    for (tag, source) in [
        (
            "list-variant",
            include_str!("../../ac-core/tests/fixtures/collections_889/list_variant.at"),
        ),
        (
            "nested-variant",
            include_str!("../../ac-core/tests/fixtures/collections_889/nested_variant.at"),
        ),
        (
            "optional-variant",
            include_str!("../../ac-core/tests/fixtures/collections_889/optional_variant.at"),
        ),
    ] {
        let run = run_source(source, tag);
        assert_eq!(
            run.exit_code,
            Some(42),
            "{tag}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_balanced(&run.stderr);
    }
}

#[test]
fn collections_list_index_trap_and_normal70_are_distinct() {
    let trap = run_source(&original("L06"), "L06");
    assert_eq!(trap.exit_code, Some(70));
    assert!(String::from_utf8_lossy(&trap.stderr).contains("MN-COLL-TRAP code=21\n"));
    assert_balanced(&trap.stderr);
    let normal = run_source("fn main() int { return 70 }", "normal70");
    assert_eq!(normal.exit_code, Some(70));
    assert!(!String::from_utf8_lossy(&normal.stderr).contains("MN-COLL-TRAP"));
    assert_balanced(&normal.stderr);
}

#[test]
fn collections_source_callee_trap_releases_both_frames() {
    let source = r#"
type Item { name Str; code int }
enum Token { Data(Item)
End }
fn fail() int { var xs = List<int>.new()
    xs.push(42)
    return xs.get(1) }
fn main() int { let t = Token.Data(Item("caller", 41))
    var xs = List<Str>.new()
    xs.push("still-live")
    return fail() }
"#;
    let run = run_source(source, "caller-callee-trap");
    assert_eq!(run.exit_code, Some(70));
    assert!(String::from_utf8_lossy(&run.stderr).contains("MN-COLL-TRAP code=21\n"));
    assert_balanced(&run.stderr);
}

#[test]
fn collections_direct_text_variant_caller_trap_has_no_lost_owner() {
    let source = r#"
enum Token { Data(Str)
End }
fn fail() int { var xs = List<int>.new()
    xs.push(42)
    return xs.get(1) }
fn main() int { let t = Token.Data("caller")
    var xs = List<Str>.new()
    xs.push("still-live")
    return fail() }
"#;
    let run = run_source(source, "direct-text-caller-trap");
    assert_eq!(run.exit_code, Some(70));
    assert!(String::from_utf8_lossy(&run.stderr).contains("MN-COLL-TRAP code=21\n"));
    assert_balanced(&run.stderr);
}

#[test]
fn collections_scalar_machine_checks_and_copyable_record_calls() {
    for (tag, source) in [
        ("text-clone", "fn main() int { let a Str = \"中\"\n let b = a.clone()\n return a.len() * 40 + b.len() * 2 }"),
        ("record-copy-call", "type Pair { x int; y int }\nfn bump(p Pair) Pair { return Pair(p.x + 1, p.y) }\nfn main() int { let a = Pair(19, 22)\n let b = bump(a)\n return a.x + b.y + 1 }"),
        ("record-copy-list", "type Pair { x int; y int }\nfn main() int { var xs = List<Pair>.new()\n let p = Pair(20, 22)\n xs.push(p)\n let q = xs.get(0)\n return p.x + q.y }"),
        ("owned-record-view-call", "type Item { name Str; code int }\nfn read(p Item) int { return p.name.len() }\nfn main() int { let p = Item(\"hello\", 37)\n return read(p) + p.code }"),
        ("bool-list", "fn main() int { var xs = List<bool>.new()\n xs.push(true)\n if xs.get(0) { return 42 }\n return 0 }"),
        ("empty-text-clone", "fn main() int { let s Str = \"\"\n let cloned = s.clone()\n return s.len() + cloned.len() + 42 }"),
        ("owned-list-clear", "fn main() int { var xs = List<Str>.new()\n xs.push(\"hello\")\n xs.clear()\n return xs.len() + 42 }"),
    ] {
        let run = run_source(source, tag);
        assert_eq!(run.exit_code, Some(42), "{tag}: {}", String::from_utf8_lossy(&run.stderr));
        assert_balanced(&run.stderr);
    }
    for (tag, source, reason) in [
        (
            "rem-overflow",
            "fn main() int { let m = -2147483647 - 1\n return m % -1 }",
            30,
        ),
        ("division-zero", "fn main() int { return 42 / 0 }", 31),
        (
            "negative-index",
            "fn main() int { var xs = List<int>.new()\n xs.push(42)\n return xs.get(-1) }",
            21,
        ),
    ] {
        let run = run_source(source, tag);
        assert_eq!(run.exit_code, Some(70));
        assert!(
            String::from_utf8_lossy(&run.stderr).contains(&format!("MN-COLL-TRAP code={reason}\n"))
        );
        assert_balanced(&run.stderr);
    }
}
