use ace_av_core::mir::{CaptureKind, MirInstr, MirType};
use ace_av_core::mir_verify::verify_mir;
use ace_core::source::compile_closures_source;
use ace_lowering::lower_checked_closures;

#[test]
fn source_closure_lowers_to_env_get_create_and_indirect_call() {
    let source =
        "fn main() int { let base = 40; let add = (n int) => { return base + n }; return add(2) }";
    let compiled =
        compile_closures_source(source, "closures-lower.at", &[]).expect("source closure compiles");
    let mir =
        lower_checked_closures(&compiled.checked, "main", 1).expect("checked closure HIR lowers");

    assert!(mir.is_closures_profile());
    assert!(mir.funcs.iter().any(|function| {
        function.env.as_ref().is_some_and(|env| env.len() == 1)
            && function.blocks.iter().any(|block| {
                block
                    .instrs
                    .iter()
                    .any(|instr| matches!(instr, MirInstr::EnvGet { field: 0, .. }))
            })
    }));
    assert!(mir.funcs.iter().any(|function| {
        function.blocks.iter().any(|block| {
            block.instrs.iter().any(|instr| {
                matches!(instr, MirInstr::CreateClosure { captures, .. }
                    if captures.len() == 1 && captures[0].kind == CaptureKind::Move)
            })
        })
    }));
    assert!(mir.funcs.iter().any(|function| {
        function.blocks.iter().any(|block| {
            block
                .instrs
                .iter()
                .any(|instr| matches!(instr, MirInstr::CallClosure { .. }))
        })
    }));
    verify_mir(&mir).expect("lowered closure MIR verifies");
}

#[test]
fn borrow_capture_lowers_as_borrow_and_declares_str_view_env() {
    let source = "fn main() int { let s Str = \"ok\"; let size = () => s.len(); return size() }";
    let compiled = compile_closures_source(source, "closures-borrow-lower.at", &[])
        .expect("source borrow closure compiles");
    let mir = lower_checked_closures(&compiled.checked, "main", 2).expect("borrow capture lowers");
    let closure = mir
        .funcs
        .iter()
        .find(|function| function.env.is_some())
        .expect("closure body has an environment declaration");
    assert_eq!(closure.env.as_ref().unwrap().len(), 1);
    assert!(matches!(
        mir.types.get(closure.env.as_ref().unwrap()[0] as usize),
        Some(MirType::StrView)
    ));
    assert!(mir.funcs.iter().any(|function| {
        function.blocks.iter().any(|block| {
            block.instrs.iter().any(|instr| {
                matches!(instr, MirInstr::CreateClosure { captures, .. }
                    if captures.len() == 1 && captures[0].kind == CaptureKind::Borrow)
            })
        })
    }));
    verify_mir(&mir).expect("borrow closure MIR verifies");
}

#[test]
fn closure_env_fields_follow_first_source_use_order() {
    let source = r#"fn main() int { let base int = 40; let text Str = "xy"; let f = move (n int) => { return text.len() + base + n }; return f(0) }"#;
    let compiled = compile_closures_source(source, "closures-capture-order.at", &[])
        .expect("multiple owned/copy captures compile");
    let mir = lower_checked_closures(&compiled.checked, "main", 3).expect("capture order lowers");
    let closure = mir
        .funcs
        .iter()
        .find(|function| function.env.is_some())
        .expect("closure body has an environment declaration");
    let env = closure.env.as_ref().unwrap();
    assert_eq!(env.len(), 2);
    assert!(matches!(mir.types.get(env[0] as usize), Some(MirType::Str)));
    assert!(matches!(mir.types.get(env[1] as usize), Some(MirType::I32)));
    assert!(mir.funcs.iter().any(|function| {
        function.blocks.iter().any(|block| {
            block.instrs.iter().any(|instr| {
                matches!(instr, MirInstr::CreateClosure { captures, .. }
                    if captures.len() == 2
                        && captures.iter().all(|capture| capture.kind == CaptureKind::Move))
            })
        })
    }));
    verify_mir(&mir).expect("multi-field closure MIR verifies");
}
