//! PLAN-821 T-02 recipe-side tests (`av_owned_821` family): the frozen
//! owned-fixture table, the canonical owned face and the corpus-seed byte
//! identity. The formal parent-side judge face is exercised by the
//! `av-data-821` corpus in the main workspace; this file pins the
//! WORKER-side truths those runs depend on.

use ace_av_core::contract::Value;

/// The corpus seeds must stay byte-identical to the frozen fixture/test
/// sources — a drifted corpus silently tests different bytes than the
/// frozen table judges (827's seeds-identity pattern).
const CORPUS_FIRST_CASE: &str =
    "../../../../crates/auto-lang/test/cases/av-data/821/src/first-case.at";

fn corpus_path(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

#[test]
fn av_owned_recipe_seeds_identity() {
    let bytes = std::fs::read(corpus_path(CORPUS_FIRST_CASE))
        .expect("corpus seed src/first-case.at exists");
    assert_eq!(
        bytes,
        ace_av::owned_fixture::FIRST_CASE_SRC.as_bytes(),
        "the corpus seed must be byte-identical to the frozen fixture source"
    );
    // The fixture table's ids are unique.
    let ids: Vec<&str> = ace_av::owned_fixture::OWNED_FIXTURES
        .iter()
        .map(|f| f.id)
        .collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), ids.len(), "duplicate owned fixture ids");
}

/// The fixture face on the REAL owned pipeline (store round trip included)
/// matches the hand-audited first-case truths — the worker-side positive
/// the parent's frozen table independently mirrors.
#[test]
fn av_owned_recipe_fixture_face_matches_hand_audit() {
    let dir = std::env::temp_dir().join(format!(
        "av821-recipe-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let f = ace_av::owned_fixture::owned_fixture("first-case").unwrap();
    let face = ace_av::owned_fixture::run_owned_fixture(f, Some(&dir), "recipe-test")
        .expect("fixture runs the owned whole path");
    assert_eq!(face["result"]["kind"], "record");
    assert_eq!(
        face["result"]["name"], "t_rec_Note",
        "the type table's nominal (synthesized) name"
    );
    assert_eq!(face["result"]["fields"][0]["utf8"], "Auto");
    assert_eq!(face["result"]["fields"][1]["bits"], "4");
    assert_eq!(
        face["live_after_release"], 0,
        "host release empties the ledger"
    );
    let allocs = face["allocs"].as_array().unwrap();
    assert_eq!(
        allocs
            .iter()
            .map(|a| a["bytes"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![4, 4],
        "two 4-byte buffers: ctor literal + clone"
    );
    let frees = face["frees"].as_array().unwrap();
    assert_eq!(
        frees
            .iter()
            .map(|e| e["id"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        allocs
            .iter()
            .map(|a| a["id"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        "creation-order LIFO free order"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// The canonical face rules: scalars ride the typed-bits encoding, Str
/// carries the exact UTF-8 bytes, records carry name+declaration-order
/// fields, and a view/token (which may never cross the entry) is refused.
#[test]
fn av_owned_recipe_canonical_face_shapes() {
    let names: std::collections::BTreeMap<u32, String> =
        [(1u32, "Note".to_string())].into_iter().collect();
    // Scalars: lowercase unpadded hex for i32/bool/char, fixed widths else.
    assert_eq!(
        ace_av::owned_fixture::canonical_owned(&Value::I32(404), &names).unwrap()["bits"],
        "194"
    );
    assert_eq!(
        ace_av::owned_fixture::canonical_owned(&Value::Bool(true), &names).unwrap()["bits"],
        "1"
    );
    assert_eq!(
        ace_av::owned_fixture::canonical_owned(&Value::U64(u64::MAX), &names).unwrap()["bits"],
        "ffffffffffffffff"
    );
    // A Str face is the exact bytes; a record face is name + fields.
    let arena = ace_av_core::owned_text::StrArena::new();
    let s = arena.alloc("A中😀".as_bytes());
    let face = ace_av::owned_fixture::canonical_owned(&Value::Str(s), &names).unwrap();
    assert_eq!(face["utf8"], "A中😀");
    let rec = ace_av_core::contract::RecordValue {
        ty: 1,
        fields: vec![Value::I32(7), Value::Bool(false)],
    };
    let face = ace_av::owned_fixture::canonical_owned(&Value::Record(rec.into()), &names).unwrap();
    assert_eq!(face["name"], "Note");
    assert_eq!(face["fields"][0]["bits"], "7");
    assert_eq!(face["fields"][1]["bits"], "0");
    // Views/tokens may never cross the entry boundary — canonicalization
    // refuses them (the worker treats that as a protocol violation).
    let view = ace_av_core::contract::StrView {
        src: ace_av_core::contract::StrViewSrc::Static {
            buf: std::rc::Rc::new(Box::from(&b"abc"[..])),
        },
        start: 0,
        end: 1,
    };
    assert!(ace_av::owned_fixture::canonical_owned(&Value::StrView(view), &names).is_none());
}
