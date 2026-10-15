extern crate auto_ac_prototype as ac;
use ac::hir::*;
fn baseline() -> Bundle {
    let p = ac::source::compile_parts_with(
        "type T { s Str }\nfn consume(move t T) int { return t.s.len() }\nfn main() int {\n let a T = T { s: \"abc\" }\n return consume(a.move)\n}\n",
        "api.at", &[], ac::source::SourceProfile::TextRecord).unwrap();
    ac::descriptor::bind(ac::text_record_descriptor(), &p.atom_text).unwrap()
}
fn main() {
    for name in ["valid", "call_arg", "call_param", "ctor_arg", "field_id", "field_type", "move_inner", "self_cycle", "requires"] {
        let mut b = baseline();
        let before = b.clone();
        if name == "requires" { b.requires.clear(); }
        let m = &mut b.modules[0];
        if name == "field_type" || name == "self_cycle" {
            let idx = m.types.iter().position(|t| matches!(t.kind, TypeKind::Record(_))).unwrap();
            if let TypeKind::Record(s) = &mut m.types[idx].kind {
                s.fields[0].ty = TypeUse::Ref(TypeId(if name == "field_type" { u32::MAX } else { idx as u32 }));
            }
        }
        let mut hit = false;
        for body in &mut m.bodies { for e in &mut body.exprs {
            if hit { continue; }
            match (&mut e.kind, name) {
                (ExprKind::Call { bindings, .. }, "call_arg") => { bindings[0].arg = u32::MAX; hit = true; }
                (ExprKind::Call { bindings, .. }, "call_param") => { bindings[0].param = u32::MAX; hit = true; }
                (ExprKind::RecordCtor { bindings, .. }, "ctor_arg") => { bindings[0].arg = u32::MAX; hit = true; }
                (ExprKind::FieldGet { path, .. }, "field_id") => { path[0].0 = u32::MAX; hit = true; }
                (ExprKind::Move { inner }, "move_inner") => { inner.0 = u32::MAX; hit = true; }
                _ => {}
            }
        }}
        assert!(name == "valid" || b != before, "mutation did not occur: {}", name);
        let r = std::panic::catch_unwind(|| ac::verify::verify_text_record(b));
        match r {
            Ok(Ok(_)) => println!("{name}: CHECKED"),
            Ok(Err(ds)) => println!("{name}: REJECT {:?}", ds.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()),
            Err(_) => println!("{name}: PANIC"),
        }
    }
}
