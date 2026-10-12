//! Fidelity guard for the vendored parser snapshot (Plan 768 feasibility §1).
//!
//! The three generated files must stay byte-identical to the pinned upstream
//! revision; any drift fails here and requires explicit re-vendoring with an
//! updated PROVENANCE.md.

use auto_test_case::sha256::{hex, sha256};

const EXPECTED: &[(&str, &str)] = &[
    (
        "markdown_parser.rs",
        "4c41853517d09f4740ed32f3a6645f3fedd8bd7ad69ac0fd3ece16f21427ffc9",
    ),
    (
        "block_model.rs",
        "8e99920ff0f078706ea7d8e8b2f351e3fe90c5220d57c791d6ca0e1000b04c36",
    ),
    (
        "ial.rs",
        "085091486a36aac33079984401575dc0e9331377d87114a2ae44c5fcd8a46bfa",
    ),
];

#[test]
fn vendored_files_match_pinned_fingerprints() {
    for (name, want) in EXPECTED {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../vendor/autodown-test-parser/src")
            .join(name);
        let bytes =
            std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
        let got = hex(&sha256(&bytes));
        assert_eq!(
            &got, want,
            "{} drifted from the pinned snapshot — re-vendor explicitly and update PROVENANCE.md",
            name
        );
    }
}
