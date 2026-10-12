//! Independent repair audit at 4cc464502; frozen original probes stay unchanged.
use auto_ac_prototype::{descriptor, hir, verify};
use std::path::Path;

fn checked(b: hir::Bundle) -> Result<verify::CheckedModule, Vec<auto_ac_prototype::atom_text::Diagnostic>> {
    match b.domain {
        hir::HirDomain::Core => verify::verify(b),
        hir::HirDomain::Scalar => verify::verify_scalar(b),
    }
}

fn main() {
    let root = std::env::args().nth(1).unwrap();
    let root = Path::new(&root);
    let mut count = 0;
    let mut check = |name: String, ok: bool| {
        assert!(ok, "FAIL {name}");
        count += 1;
        println!("PASS {name}");
    };
    for (name, source, known) in [
        ("core", "docs/design/strategy/hir-examples/01-add.atom", auto_ac_prototype::descriptor()),
        ("scalar", "experimental/ac-core/fixtures/scalar-hir/valid/s02-two-bodies.atom", auto_ac_prototype::scalar_descriptor()),
    ] {
        let src = std::fs::read_to_string(root.join(source)).unwrap();
        let bundle = descriptor::bind(known, &src).unwrap();
        check(format!("{name}/valid-reference-control"), checked(bundle.clone()).is_ok());
        for mask in 1..8 {
            let mut unknown = known.clone();
            let mut altered = src.clone();
            if mask & 1 != 0 {
                altered = altered.replace(&unknown.schema, "auto.hir.unknown");
                unknown.schema = "auto.hir.unknown".into();
            }
            if mask & 2 != 0 {
                altered = altered.replace("revision: 1u32", "revision: 0u32");
                unknown.revision = 0;
            }
            if mask & 4 != 0 {
                altered = altered.replace(&unknown.profile, "unregistered");
                unknown.profile = "unregistered".into();
            }
            let bind_reject = descriptor::bind(&unknown, &altered)
                .is_err_and(|ds| ds.iter().any(|d| d.code == "bind.descriptor-unregistered"));
            let writer_reject = descriptor::write_canonical(&bundle, &unknown)
                .is_err_and(|d| d.code == "canonical.identity-conflict");
            check(format!("{name}/identity-mask-{mask}"), unknown.domain().is_none() && bind_reject && writer_reject);
        }
        // Exact table-length boundary, maximum TypeId, every param position,
        // every function result, including a signature not used by any def.
        let mut extended = bundle.clone();
        let mut unused = extended.modules[0].types.iter()
            .find(|t| matches!(t.kind, hir::TypeKind::Function(_))).unwrap().clone();
        unused.id_text = "unused_signature".into();
        extended.modules[0].types.push(unused);
        check(format!("{name}/unused-valid-control"), checked(extended.clone()).is_ok());
        for (ti, t) in extended.modules[0].types.iter().enumerate() {
            let hir::TypeKind::Function(sig) = &t.kind else { continue };
            for slot in 0..=sig.params.len() {
                for bad in [extended.modules[0].types.len() as u32, u32::MAX] {
                    let mut b = extended.clone();
                    let hir::TypeKind::Function(s) = &mut b.modules[0].types[ti].kind else { unreachable!() };
                    if slot == s.params.len() { s.result = hir::TypeUse::Ref(hir::TypeId(bad)); }
                    else { s.params[slot] = hir::TypeUse::Ref(hir::TypeId(bad)); }
                    check(format!("{name}/signature-{ti}-slot-{slot}-id-{bad}"),
                        checked(b).is_err_and(|ds| ds.iter().any(|d| d.code == "verify.bad-reference")));
                }
            }
        }
        for role in ["local", "expr"] {
            let mut b = bundle.clone();
            let ty = hir::TypeUse::Ref(hir::TypeId(u32::MAX));
            if role == "local" { b.modules[0].bodies[0].locals[0].ty = ty; }
            else { b.modules[0].bodies[0].exprs[0].ty = ty; }
            check(format!("{name}/{role}-type-bound"), checked(b)
                .is_err_and(|ds| ds.iter().any(|d| d.code == "verify.bad-reference")));
        }
    }
    println!("REPAIR_AUDIT_PASSES={count}");
}
