//! PLAN-999 T-05 — F-941-13 permanent regressions: the catalog signature
//! digest is a SPAN-FREE structured projection (`signature_of`/`type_digest`).
//!
//! The pre-fix digest formatted `TypeRef` with `{:?}`; `Record`/`TyVar`/
//! `Generic`/`Optional`/`Function` carry `Ident { name, span }`, so a body
//! edit in one owner shifted the annotation coordinates of a LATER owner and
//! a body-only edit there falsely reported `OwnerSignatureChanged`. The
//! distinguishing carrier is an INNER ident: a pure-`int` annotation never
//! reproduces it (§5.4), so every false-positive case here carries a
//! nominal or nested annotation.
//!
//! True signature surfaces (param type / mode / return type / param name /
//! nested nominal argument) must STILL fire the event.

use ace_core::source::SourceProfile;
use ace_core::source::cst::units;

const BUDGET: ace_core::source::cst::FrontBudget =
    ace_core::source::cst::FrontBudget::WORK_OWNED_64MIB;

fn signature_events(cat: &units::UnitCatalog, owner: &str) -> usize {
    cat.events
        .iter()
        .filter(
            |e| matches!(e, units::CatalogEvent::OwnerSignatureChanged { owner: o } if o == owner),
        )
        .count()
}

/// Edit one owner's body only, at its CURRENT (possibly drifted) position.
fn body_edit(cat: &mut units::UnitCatalog, owner: &str, from: &str, to: &str) {
    let span = cat.fns[owner].span;
    let text = cat.src()[span.start as usize..span.end as usize].to_string();
    let edited = text.replacen(from, to, 1);
    assert_ne!(text, edited, "body-only edit must apply");
    cat.edit_owner(owner, &edited, BUDGET)
        .expect("body-only edit");
}

/// The F-941-13 representative sequence: `a` before `b`, `b` carries a
/// nominal record parameter. Lengthening `a`'s body drifts `b`'s spans; the
/// following BODY-ONLY edit of `b` must not report a signature change, and a
/// cold rebuild at the drifted source must produce the SAME digest.
#[test]
fn f13_record_param_span_drift_no_false_event() {
    let src = "type P { a int }\nfn a() int { return 1 }\nfn b(p P) int { return p.a }\n";
    let mut cat = units::UnitCatalog::build(src, SourceProfile::Record, BUDGET).expect("cold");
    let before = cat.signature_snapshot("b").expect("snapshot");

    cat.edit_owner("a", "fn a() int { return 1 + 1 + 1 + 1 + 1 }", BUDGET)
        .expect("edit a (body lengthened)");
    let drift = cat.fns["b"].span.start;
    assert!(drift > 0, "b's position drifted after a's edit");
    assert_eq!(
        signature_events(&cat, "b"),
        0,
        "drift alone reports nothing"
    );

    body_edit(&mut cat, "b", "return p.a", "return p.a + 0");
    assert_eq!(
        signature_events(&cat, "b"),
        0,
        "F-941-13: span drift must not fake a signature change"
    );

    let cold2 =
        units::UnitCatalog::build(cat.src(), SourceProfile::Record, BUDGET).expect("rebuild");
    assert_eq!(
        cold2.signature_snapshot("b").as_deref(),
        Some(before.as_str()),
        "digest is span-free across cold rebuilds"
    );
}

/// True signature surfaces still fire: param type, mode word, return type,
/// param name. None of these may be masked by the span-free projection.
#[test]
fn f13_true_signature_changes_still_fire() {
    for (tag, new_text) in [
        ("param type", "fn b(p int) int { return p }"),
        ("mode word", "fn b(mut p P) int { return p.a }"),
        ("return type", "fn b(p P) bool { return p.a > 0 }"),
        ("param name", "fn b(q P) int { return q.a }"),
    ] {
        let src = "type P { a int }\nfn b(p P) int { return p.a }\n";
        let mut cat = units::UnitCatalog::build(src, SourceProfile::Record, BUDGET).expect("cold");
        cat.edit_owner("b", new_text, BUDGET)
            .unwrap_or_else(|e| panic!("{tag} edit must parse: {e:?}"));
        assert_eq!(
            signature_events(&cat, "b"),
            1,
            "{tag}: a real signature change must fire"
        );
    }
}

/// UTF-8 comment content and CRLF line endings ride the same spans — a drift
/// + body-only edit under them must stay event-free and digest-stable.
#[test]
fn f13_crlf_utf8_comment_stability() {
    let src = "type P { a int }\r\nfn a() int { return 1 }\r\nfn b(p P) int {\r\n    // 中文注释：坐标漂移不应触发签名事件\r\n    return p.a\r\n}\r\n";
    let mut cat = units::UnitCatalog::build(src, SourceProfile::Record, BUDGET).expect("cold");
    let before = cat.signature_snapshot("b").expect("snapshot");
    cat.edit_owner("a", "fn a() int { return 1 + 1 + 1 }", BUDGET)
        .expect("edit a (CRLF source)");
    body_edit(&mut cat, "b", "return p.a", "return p.a + 0");
    assert_eq!(
        signature_events(&cat, "b"),
        0,
        "CRLF/中文注释源上的坐标漂移不得误报"
    );
    let cold2 =
        units::UnitCatalog::build(cat.src(), SourceProfile::Record, BUDGET).expect("rebuild");
    assert_eq!(
        cold2.signature_snapshot("b").as_deref(),
        Some(before.as_str())
    );
}

/// Nested annotations (Generic with a NOMINAL argument) carry inner Idents on
/// every level — the deepest false-positive carrier. Drift + body-only edit
/// stays silent; a real nested-argument change (`List<P>` → `List<int>`)
/// fires; the digest is stable across cold rebuilds.
#[test]
fn f13_nested_generic_annotation() {
    let src = "type P { a int }\nfn a() int { return 1 }\nfn g(x List<P>) int { return x.len() }\n";
    let mut cat = units::UnitCatalog::build(src, SourceProfile::Collections, BUDGET).expect("cold");
    let before = cat.signature_snapshot("g").expect("snapshot");

    cat.edit_owner("a", "fn a() int { return 1 + 1 + 1 + 1 + 1 + 1 }", BUDGET)
        .expect("edit a");
    body_edit(&mut cat, "g", "return x.len()", "return x.len() + 0");
    assert_eq!(
        signature_events(&cat, "g"),
        0,
        "嵌套Generic内层Ident的坐标漂移不得误报"
    );
    let cold2 =
        units::UnitCatalog::build(cat.src(), SourceProfile::Collections, BUDGET).expect("rebuild");
    assert_eq!(
        cold2.signature_snapshot("g").as_deref(),
        Some(before.as_str())
    );

    // The real nested-argument change still fires.
    cat.edit_owner("g", "fn g(x List<int>) int { return x.len() }", BUDGET)
        .expect("real nested change");
    assert_eq!(
        signature_events(&cat, "g"),
        1,
        "List<P> → List<int> 是真签名变化"
    );
}
