//! Independent review counterexamples for PLAN-767 r2 at 033642a27.
//! This is a review artifact, not a product or permanent test change.
//! Compile against the reviewed ac-core rlib; invoke with the worktree root.
use auto_ac_prototype::{descriptor, hir, verify};
use std::path::Path;

fn main() {
    let root = std::env::args().nth(1).expect("worktree root");
    let root = Path::new(&root);
    let core_src = std::fs::read_to_string(
        root.join("docs/design/strategy/hir-examples/01-add.atom"),
    ).unwrap();
    let scalar_src = std::fs::read_to_string(root.join(
        "experimental/ac-core/fixtures/scalar-hir/valid/s02-two-bodies.atom",
    )).unwrap();
    let core = auto_ac_prototype::bind_source(&core_src).unwrap();
    let scalar = auto_ac_prototype::bind_document(&scalar_src).unwrap();
    let mut failures = 0;
    let mut check = |id: &str, expected: &str, ok: bool, actual: String| {
        println!("{id}\texpected={expected}\tactual={actual}\t{}", if ok { "PASS" } else { "FAIL" });
        if !ok { failures += 1; }
    };

    check("CONTROL-CORE", "valid Checked", verify::verify(core.clone()).is_ok(), "valid core".into());
    check("CONTROL-SCALAR", "valid Checked", verify::verify_scalar(scalar.clone()).is_ok(), "valid scalar".into());
    check("CONTROL-DOMAIN", "reject scalar through core entry", verify::verify(scalar.clone()).is_err(), "explicit verify entry".into());
    check("CONTROL-WRITER", "reject core with scalar descriptor", descriptor::write_canonical(&core, auto_ac_prototype::scalar_descriptor()).is_err(), "known domain mismatch".into());

    for which in ["schema", "revision", "profile"] {
        let mut desc = auto_ac_prototype::descriptor().clone();
        let altered = match which {
            "schema" => { desc.schema = "auto.hir.unregistered".into(); core_src.replace("auto.hir.core.draft", &desc.schema) }
            "revision" => { desc.revision = 99; core_src.replace("revision: 1u32", "revision: 99u32") }
            "profile" => { desc.profile = "unregistered-profile".into(); core_src.replace("core-i32-draft", &desc.profile) }
            _ => unreachable!(),
        };
        let written = descriptor::write_canonical(&core, &desc);
        let actual = match &written {
            Ok(text) => text.lines().next().unwrap_or("").to_owned(),
            Err(d) => d.code.clone(),
        };
        check(&format!("WRITER-{which}"), "controlled identity rejection", written.is_err(), actual);

        let bound = descriptor::bind(&desc, &altered);
        let (rejected, actual) = match bound {
            Err(ds) => (true, format!("bind rejected {:?}", ds.iter().map(|d| &d.code).collect::<Vec<_>>())),
            Ok(bundle) => {
                let domain = bundle.domain;
                let verified = verify::verify(bundle);
                (verified.is_err(), format!("bound domain={domain:?}; core verify accepted={}", verified.is_ok()))
            }
        };
        check(&format!("UNKNOWN-{which}"), "unregistered identity must not yield Checked", rejected, actual);
        check(&format!("CONTROL-CLI-REGISTRY-{which}"), "registry rejects unknown identity", auto_ac_prototype::bind_document(&altered).is_err(), "bind_document registry".into());
    }

    // All body/expr refs stay valid. Only a function signature's TypeUse is mutated.
    for which in ["parameter", "result"] {
        let mut b = scalar.clone();
        let hir::TypeKind::Function(sig) = &mut b.modules[0].types[1].kind else { panic!("fixture function type") };
        if which == "parameter" { sig.params[0] = hir::TypeUse::Ref(hir::TypeId(9999)); }
        else { sig.result = hir::TypeUse::Ref(hir::TypeId(9999)); }
        let outcome = verify::verify_scalar(b);
        let actual = match &outcome {
            Ok(c) => {
                let native = std::panic::catch_unwind(|| {
                    let entry = auto_ac_prototype::native::find_entry(&c, "d_main").unwrap();
                    auto_ac_prototype::native::lower_object(&c, &entry)
                });
                format!("invalid signature yielded Checked domain={:?}; native_panicked={}", c.domain(), native.is_err())
            }
            Err(ds) => format!("rejected {:?}", ds.iter().map(|d| &d.code).collect::<Vec<_>>()),
        };
        check(&format!("SIGNATURE-{which}"), "invalid TypeId rejected before Checked", outcome.is_err(), actual);
    }

    // Meaningful invalid controls prove that the harness does exercise the actual verifier.
    let mut b = scalar;
    let hir::ExprKind::Call { eval_args, .. } = &mut b.modules[0].bodies[1].exprs[2].kind else { panic!("fixture call") };
    eval_args[0] = hir::ExprId(9999);
    check("CONTROL-BODY-BOUNDS", "invalid ExprId rejected", verify::verify_scalar(b).is_err(), "known body bounds check".into());
    println!("CONTRACT_FAILURES={failures}");
    if failures != 0 { std::process::exit(1); }
}
