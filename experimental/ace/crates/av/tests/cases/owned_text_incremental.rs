//! PLAN-894 AV tier: the owned-TEXT source session
//! (`OwnedFunctionSession` over text-only modules) — V-06's AV leg
//! (true single-body increments with the published production counters)
//! and V-07's AV leg (a REFUSED edit keeps the prior revision runnable).
//! The native twin lives in
//! `ac-mir-native/tests/text_source_894_incremental.rs`; the frozen
//! original T03/T05 values are covered by the corpus tiers.

use ace_av::owned_session::OwnedFunctionSession;

/// The §5.3 supplement source: text_job = 23 (with the `c = c + "中"`
/// replacement), cmp_job = 42, main = 65; disjoint() is uncalled.
const SUPPLEMENT: &str = "\
fn text_job() int {
    let a Str = \"ab\"
    let b Str = a.clone()
    var c Str = b.move
    c = c + \"中\"
    return a.len() * 10 + c.len()
}
fn cmp_job() int {
    let a str = \"A\\0B\"
    if a == \"A\" { return 1 }
    if a != \"A\\0B\" { return 2 }
    return 42
}
fn disjoint() int { return 100 }
fn main() int { return text_job() + cmp_job() }
";

fn production_sums(s: &OwnedFunctionSession) -> Vec<(String, u32, u32)> {
    s.production()
        .iter()
        .map(|(k, v)| (k.clone(), v.parse, v.lower))
        .collect()
}

#[test]
fn av_owned_session_increment_65_66_65() {
    let mut s = OwnedFunctionSession::warmup(SUPPLEMENT, "d_main", &[])
        .expect("warmup on the text-only supplement");
    assert_eq!(s.run_i32(), 65, "cold value 23 + 42");
    let cold = production_sums(&s);
    for (name, parse, lower) in &cold {
        assert_eq!((parse, lower), (&1, &1), "{name}: one cold round");
    }

    // ---- single-body edit: `中` -> `中x` inside text_job (65 -> 66) ----
    let pos = SUPPLEMENT.find("c + \"中\"").expect("needle");
    let byte_pos = SUPPLEMENT[..pos].len() + "c + \"".len();
    let invalidated = s
        .edit(
            byte_pos as u64,
            (byte_pos + "中".len()) as u64,
            "中x".as_bytes(),
        )
        .expect("single-body edit");
    assert!(
        invalidated.iter().any(|n| n.contains("text_job")),
        "text_job invalidated: {invalidated:?}"
    );
    assert_eq!(s.run_i32(), 66, "24 + 42");
    let after_b = production_sums(&s);
    for (name, parse, lower) in &after_b {
        let was = cold.iter().find(|(n, _, _)| n == name).unwrap();
        if name.contains("text_job") {
            assert_eq!((*parse, *lower), (2, 2), "{name}: one real re-production");
        } else {
            assert_eq!(
                (parse, lower),
                (&was.1, &was.2),
                "{name}: ZERO new production on a single-body edit"
            );
        }
    }

    // ---- back to A: 66 -> 65 (the prior ABC objects stay published).
    // The back-edit REPLACES the just-inserted `中x` — the same start
    // offset (the edit site's prefix is unchanged).
    let inv2 = s
        .edit(
            byte_pos as u64,
            (byte_pos + "中x".len()) as u64,
            "中".as_bytes(),
        )
        .expect("back edit");
    assert!(inv2.iter().any(|n| n.contains("text_job")));
    assert_eq!(s.run_i32(), 65, "value back to 65");
    let after_a = production_sums(&s);
    let tj = after_a
        .iter()
        .find(|(n, _, _)| n.contains("text_job"))
        .unwrap();
    assert_eq!(
        (tj.1, tj.2),
        (3, 3),
        "three real frontend rounds, three unit productions"
    );
}

#[test]
fn av_owned_session_refused_edit_keeps_running() {
    let mut s = OwnedFunctionSession::warmup(SUPPLEMENT, "d_main", &[]).expect("warmup");
    // A type-broken body edit: `return a.len() * 10 + c.len()` -> `return a`.
    let pos = SUPPLEMENT.find("return a.len() * 10 + c.len()").unwrap();
    let byte_pos = SUPPLEMENT[..pos].len() + "return ".len();
    let err = s
        .edit(
            byte_pos as u64,
            (byte_pos + "a.len() * 10 + c.len()".len()) as u64,
            b"a",
        )
        .expect_err("the broken edit is refused");
    assert!(
        err.contains("type-mismatch") || err.contains("mismatch"),
        "named frontend refusal: {err}"
    );
    // the prior revision still runs at 65
    assert_eq!(s.run_i32(), 65, "the prior revision keeps running");
    // and a FOLLOWING legal edit recovers 66 (the refusal left the
    // source untouched — the same offsets apply)
    let pos = SUPPLEMENT.find("c + \"中\"").unwrap();
    let byte_pos = SUPPLEMENT[..pos].len() + "c + \"".len();
    s.edit(
        byte_pos as u64,
        (byte_pos + "中".len()) as u64,
        "中x".as_bytes(),
    )
    .expect("legal edit after refusal");
    assert_eq!(s.run_i32(), 66);
}
