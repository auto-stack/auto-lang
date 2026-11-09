//! Read original typed test contracts without parsing Rust or running guests.
use ace_a2x_core::sha256::sha256_hex;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn write_inventory(document: serde_json::Value) {
    let output = PathBuf::from(
        std::env::var_os("ACE_M1_CONTRACT_INVENTORY_OUT")
            .expect("explicit contract inventory output required"),
    );
    assert!(output.is_absolute());
    let parent = output.parent().unwrap();
    // Check lexical ancestors before canonicalizing: a link inside the scope
    // must not authorize its target or hide its original path.
    for ancestor in parent.ancestors() {
        let metadata = std::fs::symlink_metadata(ancestor).unwrap();
        assert!(!metadata.file_type().is_symlink());
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            assert_eq!(
                metadata.file_attributes() & 0x400,
                0,
                "linked output parent"
            );
        }
    }
    let root = PathBuf::from("D:/autostack/plan-artifacts/plan-941")
        .canonicalize()
        .unwrap();
    assert!(parent.canonicalize().unwrap().starts_with(root));
    let raw = serde_json::to_vec_pretty(&document).unwrap();
    assert!(raw.len() <= 4 << 20);
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .unwrap();
    file.write_all(&raw).unwrap();
}

#[test]
#[ignore = "explicit read-only contract export; not a semantics execution gate"]
fn read_scalar_contract_inventory_941() {
    let mut names = BTreeSet::new();
    let rows: Vec<_> = super::scalar_semantics::cases()
        .into_iter()
        .map(|case| {
            assert!(
                names.insert(case.name.clone()),
                "duplicate original scalar case"
            );
            let oracle = match case.expect {
                super::scalar_semantics::Expect::Value(value) => {
                    serde_json::json!({"kind":"value", "value":value})
                }
                super::scalar_semantics::Expect::Trap(trap) => {
                    serde_json::json!({"kind":"trap", "trap":trap})
                }
            };
            serde_json::json!({
                "case":case.name, "profile":case.profile, "trace":case.trace,
                "source_sha256":sha256_hex(case.src.as_bytes()), "source":case.src,
                "oracle":oracle, "marks":case.marks, "release_too":case.release_too,
            })
        })
        .collect();
    assert!(!rows.is_empty());
    let document = serde_json::json!({
        "schema":"ace.m1.scalar-contract-inventory/1", "scope":"discovery", "executed":0,
        "producer":"ace-a2x-worker::conformance::scalar_semantics::cases",
        "table_sha256":sha256_hex(include_bytes!("scalar_semantics.rs")),
        "cases":rows,
    });
    write_inventory(document);
}

#[test]
#[ignore = "explicit read-only contract export; not a semantics execution gate"]
fn read_composite_contract_inventory_941() {
    let mut rows = Vec::new();
    for case in super::a2c_rt_cert::mode_contracts_941()
        .into_iter()
        .chain(super::a2c_rt_cert::trap_contracts_941())
    {
        let oracle = match case.oracle_kind {
            "value" => serde_json::json!({"kind":"value", "value":case.oracle_value}),
            "trap" => serde_json::json!({"kind":"trap", "trap":case.oracle_trap}),
            other => panic!("unknown original oracle {other}"),
        };
        rows.push(
            serde_json::json!({"family":"record-text-cert-925", "case":case.id,
            "source":case.src, "source_sha256":sha256_hex(case.src.as_bytes()),
            "profile":case.profile, "caps":case.caps, "oracle":oracle,
            "reference":case.reference, "reference_kind":"independent-handwritten-c"}),
        );
    }
    for (name, source, kind, expected) in super::a2c_rt::text_char_contracts_941() {
        let oracle = match kind {
            "value" => {
                serde_json::json!({"kind":"value", "value":expected.parse::<i64>().unwrap()})
            }
            "trap" => serde_json::json!({"kind":"trap", "trap":expected}),
            other => panic!("unknown original oracle {other}"),
        };
        rows.push(serde_json::json!({"family":"text-char-925", "case":name,
            "source":source, "source_sha256":sha256_hex(source.as_bytes()),
            "profile":"text", "caps":[], "oracle":oracle,
            "reference":null, "reference_kind":"not-declared-by-original-test"}));
    }
    let suite = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../crates/auto-lang/test/cases/a2c-hir-collections-926");
    for (name, kind, expected) in super::a2c_coll_cert::CASES {
        let source = std::fs::read_to_string(suite.join(format!("src/{name}.at"))).unwrap();
        let reference = std::fs::read_to_string(suite.join(format!("reference-{name}.c"))).unwrap();
        let oracle = match *kind {
            "value" => {
                serde_json::json!({"kind":"value", "value":expected.parse::<i64>().unwrap()})
            }
            "trap" => serde_json::json!({"kind":"trap", "trap":expected}),
            other => panic!("unknown original oracle {other}"),
        };
        rows.push(serde_json::json!({"family":"collections-926", "case":name,
            "source_sha256":sha256_hex(source.as_bytes()), "source":source,
            "profile":"collections", "caps":[], "oracle":oracle,
            "reference":reference, "reference_kind":"independent-handwritten-c"}));
    }
    let ids: BTreeSet<_> = rows
        .iter()
        .map(|r| (r["family"].as_str().unwrap(), r["case"].as_str().unwrap()))
        .collect();
    assert_eq!(ids.len(), rows.len());
    assert_eq!(rows.len(), 41); // original modes8 + traps3 + text/char10 + collections20
    write_inventory(
        serde_json::json!({"schema":"ace.m1.composite-contract-inventory/1",
        "scope":"discovery", "executed":0,
        "sources":[
            {"path":"a2c_rt_cert.rs", "sha256":sha256_hex(include_bytes!("a2c_rt_cert.rs"))},
            {"path":"a2c_rt.rs", "sha256":sha256_hex(include_bytes!("a2c_rt.rs"))},
            {"path":"a2c_coll_cert.rs", "sha256":sha256_hex(include_bytes!("a2c_coll_cert.rs"))}
        ], "cases":rows}),
    );
}
