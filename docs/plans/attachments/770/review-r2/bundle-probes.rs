use auto_ac_prototype::{hir, native, source, verify};
use std::path::Path;
fn main() {
    let root = std::env::args().nth(1).expect("evidence directory");
    let root = Path::new(&root);
    for entry in std::fs::read_dir(root).unwrap() {
        let p = entry.unwrap().path();
        if p.extension().and_then(|s| s.to_str()) != Some("at") { continue; }
        let src = std::fs::read_to_string(&p).unwrap();
        let caps = if src.contains("mark_") { vec!["hir.test.trace.v1".to_string()] } else { vec![] };
        let parts = source::compile_parts_with(&src, "review", &caps, source::SourceProfile::Text).unwrap();
        std::fs::write(p.with_extension("atom"), &parts.atom_text).unwrap();
        println!("bundle-unmutated {} verify={}", p.file_name().unwrap().to_string_lossy(), verify::verify_text(parts.bundle.clone()).is_ok());
    }
    let src = "fn main() int { let x Str = \"abc\"; return x.len() }";
    let b = source::compile_parts_with(src, "mutation", &[], source::SourceProfile::Text).unwrap().bundle;
    for domain in [hir::HirDomain::Core, hir::HirDomain::Scalar] {
        let mut mutant = b.clone();
        assert_ne!(mutant.domain, domain);
        mutant.domain = domain;
        assert_eq!(mutant.domain, domain);
        match verify::verify_text(mutant) {
            Err(ds) => println!("M01-domain {} rejected {:?}", domain.as_str(), ds.iter().map(|d| &d.code).collect::<Vec<_>>()),
            Ok(c) => {
                println!("M01-domain {} ACCEPTED checked.domain={}", domain.as_str(), c.domain().as_str());
                let main_id = &c.bundle().modules[0].defs[0].id_text;
                let sel = native::find_entry(&c, main_id).unwrap();
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| native::lower_object(&c, &sel)));
                println!("M01-domain lower outcome={:?}", result.map(|r| r.map(|v| v.len()).map_err(|ds| ds.iter().map(|d| d.code.clone()).collect::<Vec<_>>())));
            }
        }
    }
    let mut bad_type = b.clone();
    bad_type.modules[0].bodies[0].locals[0].ty = hir::TypeUse::Ref(hir::TypeId(u32::MAX));
    println!("M02-invalid-type REJECT={:?}", verify::verify_text(bad_type).unwrap_err().iter().map(|d| &d.code).collect::<Vec<_>>());
    let mut bad_ref = b.clone();
    bad_ref.modules[0].bodies[0].exprs[1].kind = hir::ExprKind::ReadLocal { local: hir::LocalId(u32::MAX) };
    println!("M03-invalid-local REJECT={:?}", verify::verify_text(bad_ref).unwrap_err().iter().map(|d| &d.code).collect::<Vec<_>>());
    let mut bad_mode = b.clone();
    bad_mode.modules[0].bodies[0].locals[0].param_mode = hir::ParamMode::MoveOwn;
    println!("M04-mode-on-binding ACCEPT={}", verify::verify_text(bad_mode).is_ok());
    let borrowed = "fn main() int { let v str = \"abc\".to(str); return v.len() }";
    let mut literal_mutant = source::compile_parts_with(borrowed, "literal-mutant", &[], source::SourceProfile::Text).unwrap().bundle;
    let literal = literal_mutant.modules[0].bodies[0].exprs.iter_mut().find(|e| matches!(e.kind, hir::ExprKind::StrLit { .. })).unwrap();
    assert_ne!(literal.ty, hir::TypeUse::Builtin(hir::BuiltinType::StrOwned));
    literal.ty = hir::TypeUse::Builtin(hir::BuiltinType::StrOwned);
    assert_eq!(literal.ty, hir::TypeUse::Builtin(hir::BuiltinType::StrOwned));
    println!("M05-owned-literal-borrow ACCEPT={}", verify::verify_text(literal_mutant.clone()).is_ok());
    std::fs::write(root.join("mut01-owned-literal-view.atom"), auto_ac_prototype::descriptor::write_canonical(&literal_mutant, auto_ac_prototype::text_descriptor()).unwrap()).unwrap();
    let mut no_requires = b.clone();
    assert!(no_requires.requires.iter().any(|r| r == "ac.runtime.text.v1"));
    no_requires.requires.clear();
    assert!(no_requires.requires.is_empty());
    let checked = verify::verify_text(no_requires.clone()).unwrap();
    println!("M06-remove-runtime-requires ACCEPT=true cap-check-empty={}", native::capabilities_check(&checked, &[]).is_ok());
    std::fs::write(root.join("mut02-no-runtime-requires.atom"), auto_ac_prototype::descriptor::write_canonical(&no_requires, auto_ac_prototype::text_descriptor()).unwrap()).unwrap();
}
