//! Original-source binding shared by the pilot CLI and its regressions.
use super::super::test_batch;
use auto_test_case::admission::Limits;
use auto_test_case::contract::{CheckSpec, PayloadSpec};
use auto_test_case::discovery::{load_suite, LoadedSuite};
use auto_test_case::ir::{resolve_case, resolve_oracle, ResolvedCase, ResolvedOracle};
use auto_test_case::suite::apply_defaults;
use std::path::PathBuf;

pub struct Bound {
    pub case: ResolvedCase,
    pub check: CheckSpec,
    pub oracle: ResolvedOracle,
    pub root: PathBuf,
    pub origin: serde_json::Value,
}

fn bind(
    suite: &LoadedSuite,
    case_id: &str,
    check_id: &str,
    root: PathBuf,
) -> Result<Bound, String> {
    let (file, spec) = suite
        .cases
        .iter()
        .flat_map(|f| f.specs.iter().map(move |s| (f, s)))
        .find(|(_, s)| s.id == case_id)
        .ok_or("missing original case identity")?;
    let case = resolve_case(
        spec.clone(),
        &file.doc,
        &suite.fixture_base,
        &Limits::defaults(),
    )
    .map_err(|e| e.to_string())?;
    let declared = spec
        .checks
        .iter()
        .find(|c| c.id == check_id)
        .ok_or("missing original check identity")?;
    let (check, _) = apply_defaults(declared, &suite.suite.defaults);
    let oracle = resolve_oracle(case_id, &check, &suite.fixture_base, &Limits::defaults())
        .map_err(|e| e.to_string())?;
    let fence = file
        .doc
        .fences()
        .iter()
        .filter(|f| f.language == auto_test_case::discovery::CASE_FENCE_LANGUAGE)
        .nth(
            file.specs
                .iter()
                .position(|s| s.id == case_id)
                .ok_or("case span missing")?,
        )
        .ok_or("case fence missing")?;
    let origin = serde_json::json!({"case_document":file.path,
        "case_open_line":fence.open_line,"source":match &spec.source.payload {
            PayloadSpec::File(rel) => suite.fixture_base.join(rel),
            _ => file.path.clone(),
        }, "oracle_document":file.path,"original_case":case_id,"check":check_id});
    Ok(Bound {
        case,
        check,
        oracle,
        root,
        origin,
    })
}

pub fn load(tag: &str) -> Result<Vec<Bound>, String> {
    if tag.len() > 80 || !tag.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return Err("pilot tag must contain at most 80 ASCII letters, digits or hyphens".into());
    }
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "plan888-batch-{}-{timestamp}-{unique}-{tag}",
        std::process::id()
    ));
    // Never recursively delete a pre-existing, predictable path.
    std::fs::create_dir(&root).map_err(|e| e.to_string())?;
    let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../auto-lang/test/cases/ace-m1");
    let suite = load_suite(&corpus.join("ace-m1-base.suite.ad"), &Limits::defaults())
        .map_err(|e| e.to_string())?;
    if !suite.errors.is_empty() {
        return Err(format!("suite load errors: {:?}", suite.errors));
    }
    let binding = std::fs::read_to_string(corpus.join("binding.tsv")).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for n in 1..=12 {
        let case_id = format!("ace.m1.b{n:02}");
        let mut item = if matches!(n, 8 | 11) {
            // These sources have existing binding-table AV obligations but
            // no green four-path suite row (legacy differs). Create a new
            // AV-only consumer using the ORIGINAL source and binding oracle.
            let mut item = bind(&suite, "ace.m1.b01", "av", root.join(format!("b{n:02}")))?;
            let original_id = format!("B{n:02}");
            let row: Vec<_> = binding
                .lines()
                .find(|l| l.starts_with(&format!("{original_id}\tcorpus\t")))
                .ok_or("missing original binding row")?
                .split('\t')
                .collect();
            if row.len() < 7 || row[6] != "ok" {
                return Err("binding not registered AV success".into());
            }
            let expected = row[4].parse::<i32>().map_err(|e| e.to_string())?;
            let file = &suite.cases[0];
            let mut spec = item.case.spec.clone();
            spec.id = format!("ace.m1.binding.b{n:02}");
            spec.source.payload = PayloadSpec::File(format!("src/{original_id}.at"));
            item.case = resolve_case(spec, &file.doc, &suite.fixture_base, &Limits::defaults())
                .map_err(|e| e.to_string())?;
            item.check.expect.exit = Some(expected);
            item.origin = serde_json::json!({"source":corpus.join(format!("src/{original_id}.at")),
                "source_line":1,"oracle_document":corpus.join("binding.tsv"),
                "oracle_line":binding.lines().position(|l|l.starts_with(&format!("{original_id}\tcorpus\t"))).map(|n|n+1),
                "oracle_column":5,"original_case":original_id,"consumer":"new AV-only binding consumer"});
            item.oracle = resolve_oracle(
                &item.case.spec.id,
                &item.check,
                &suite.fixture_base,
                &Limits::defaults(),
            )
            .map_err(|e| e.to_string())?;
            item
        } else {
            bind(&suite, &case_id, "av", root.join(format!("b{n:02}")))?
        };
        item.case.spec.checks = vec![item.check.clone()];
        result.push(item);
    }
    let suite = load_suite(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../auto-lang/test/cases/governance-887/first-wave.suite.ad"),
        &Limits::defaults(),
    )
    .map_err(|e| e.to_string())?;
    for name in ["arithmetic", "unary", "group"] {
        result.push(bind(
            &suite,
            &format!("gov.d.{name}"),
            "av-scalar-control",
            root.join(name),
        )?);
    }
    for item in &result {
        std::fs::create_dir_all(&item.root).map_err(|e| e.to_string())?;
    }
    Ok(result)
}

pub fn items(values: &[Bound]) -> Vec<test_batch::Item<'_>> {
    values
        .iter()
        .map(|v| test_batch::Item {
            case: &v.case,
            check: &v.check,
            oracle: &v.oracle,
            project_dir: &v.root,
            artifact_dir: &v.root,
            run_dir: &v.root,
        })
        .collect()
}
