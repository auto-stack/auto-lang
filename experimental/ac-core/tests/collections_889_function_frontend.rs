use auto_ac_prototype::{hir, source as ac, verify};
fn env(source: &str) -> ac::CheckedEnv {
    let parsed = ac::parse_program_with(source, ac::SourceProfile::Collections).unwrap();
    let mut env = ac::resolve_env(&parsed, &[], ac::SourceProfile::Collections).unwrap();
    env.typecheck_all().unwrap();
    env
}
#[test]
fn collection_single_body_matches_whole_and_does_not_claim_whole_credential() {
    let source="type Item { name Str; code int }\nenum Token { Data(Item)\nEnd }\nfn make() Token { return Token.Data(Item(\"中\",41)) }\nfn main() int { let t=make()\n is t { Data(p) -> { return p.name.len()+p.code } End -> { return 0 } } }";
    let env = env(source);
    let ((parts, receipt), work) = auto_ac_prototype::body_work::capture_body_work(|| {
        ac::emit_function_body(&env, 1, source, "p889.at").unwrap()
    });
    assert_eq!(receipt.real_body_plans, 1);
    assert_eq!(receipt.scaffold_fns, 1);
    assert_eq!(
        work.len(),
        1,
        "scaffolds must not plan or bind another body"
    );
    let m = &parts.bundle.modules[0];
    let def = m.defs.iter().position(|d| d.id_text == "d_main").unwrap();
    let function = verify::verify_function(parts.bundle.clone(), def).unwrap();
    assert_eq!(function.domain(), hir::HirDomain::Collections);
    let whole = ac::compile_collections_source(source, "p889.at", &[]).unwrap();
    let wm = &whole.checked.bundle().modules[0];
    let whole_body = wm
        .defs
        .iter()
        .find_map(|d| {
            if d.id_text == "d_main" {
                if let hir::DefKind::Function { body, .. } = d.kind {
                    Some(&wm.bodies[body.0 as usize])
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(function.body(), whole_body);
    let failures = verify::verify_collections(parts.bundle).unwrap_err();
    assert!(failures.iter().any(|d| d.code == "verify.missing-body"));
}
#[test]
fn collection_function_keeps_view_escape_and_reference_bounds_checks() {
    let source =
        "fn main() int { var xs=List<Str>.new()\n xs.push(\"hello\")\n return xs.get(0).len() }";
    let env = env(source);
    let (parts, _) = ac::emit_function_body(&env, 0, source, "p889.at").unwrap();
    verify::verify_function(parts.bundle.clone(), 0).unwrap();
    let mut bad = parts.bundle.clone();
    let body = &mut bad.modules[0].bodies[0];
    let index = body
        .exprs
        .iter()
        .position(|e| matches!(e.kind, hir::ExprKind::TextLen { .. }))
        .unwrap();
    let hir::ExprKind::TextLen { operand } = body.exprs[index].kind else {
        unreachable!()
    };
    let mut terminal = body.exprs[index].clone();
    terminal.id_text = "forbidden_clone_len".into();
    terminal.kind = hir::ExprKind::TextLen {
        operand: hir::ExprId(index as u32),
    };
    body.exprs[index].kind = hir::ExprKind::TextClone { operand };
    body.exprs[index].ty = body.exprs[operand.0 as usize].ty;
    let last = hir::ExprId(body.exprs.len() as u32);
    body.exprs.push(terminal);
    for b in &mut body.blocks {
        for statement in &mut b.stmts {
            if let hir::Stmt::Return { value } = statement {
                *value = last;
            }
        }
    }
    let errors = verify::verify_function(bad, 0).unwrap_err();
    assert!(
        errors.iter().any(|d| d.code == "verify.view-escape"),
        "{errors:?}"
    );
    let mut bad = parts.bundle;
    if let hir::TypeKind::Function(s) = &mut bad.modules[0]
        .types
        .iter_mut()
        .find(|t| matches!(t.kind, hir::TypeKind::Function(_)))
        .unwrap()
        .kind
    {
        s.result = hir::TypeUse::Ref(hir::TypeId(999));
    }
    assert!(verify::verify_function(bad, 0)
        .unwrap_err()
        .iter()
        .any(|d| d.code == "verify.bad-reference"));
}
#[test]
fn collection_context_binds_unread_record_and_variant_declarations() {
    let source =
        "type Item { code int }\nenum Token { Data(Item)\nEnd }\nfn main() int { return 42 }";
    let env = env(source);
    let (parts, _) = ac::emit_function_body(&env, 0, source, "p889.at").unwrap();
    let old = verify::verify_function(parts.bundle.clone(), 0).unwrap();
    for variant in [false, true] {
        let mut bad = parts.bundle.clone();
        let m = &mut bad.modules[0];
        if variant {
            let t = m
                .types
                .iter_mut()
                .find(|t| matches!(t.kind, hir::TypeKind::Variant { .. }))
                .unwrap();
            if let hir::TypeKind::Variant { arms } = &mut t.kind {
                arms.swap(0, 1);
            }
        } else {
            let t = m
                .types
                .iter_mut()
                .find(|t| matches!(t.kind, hir::TypeKind::Record(_)))
                .unwrap();
            if let hir::TypeKind::Record(r) = &mut t.kind {
                r.fields[0].name.push('X');
            }
        }
        let new = verify::verify_function(bad, 0).unwrap();
        assert_ne!(old.context_revision(), new.context_revision());
    }
}
#[test]
fn retained_collection_cold_typecheck_rejects_bad_source_and_close() {
    let source = "fn main() int { return true }";
    let p = ac::parse_program_with(source, ac::SourceProfile::Collections).unwrap();
    let mut e = ac::resolve_env(&p, &[], ac::SourceProfile::Collections).unwrap();
    assert!(e
        .typecheck_all()
        .unwrap_err()
        .iter()
        .any(|d| d.code == "source.type-mismatch"));
    e.close();
    assert!(e
        .typecheck_all()
        .unwrap_err()
        .iter()
        .any(|d| d.code == "source.closed"));
}
