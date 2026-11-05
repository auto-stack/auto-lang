//! PLAN-938 public physical signature projection at the actual native call site.
use ace_foreign_contract::{
    manual::{AdmittedForeign, SourceDecl},
    FnForm, LaneKind,
};
use cranelift_codegen::{
    ir::{types, AbiParam, Signature},
    isa::CallConv,
};
pub fn emitter_identity() -> String {
    format!(
        "ace-native-foreign/1+ace-native/{}+cranelift/{}",
        env!("CARGO_PKG_VERSION"),
        cranelift_codegen::VERSION
    )
}

/// Source host composition uses the ordinary fact-driven product entry and the
/// ONE shared HIR→MIR lowering. There is no wrapper-level substitute call.
pub fn compile_source_units(
    source: &str,
    module: &str,
    capabilities: &[String],
    environment: &AdmittedForeign,
    session: &mut ace_source_query::SourceQuerySession,
) -> Result<
    (
        ace_source_query::ProductCompile,
        Vec<ace_av_core::mir::MirModule>,
    ),
    String,
> {
    session.set_foreign(Some(environment.clone()));
    let package = session.package().to_string();
    let owner = session.module().to_string();
    let product = ace_source_query::compile_scalar_product(
        source,
        module,
        capabilities,
        &package,
        &owner,
        session,
    )
    .map_err(|ds| format!("source: {ds:?}"))?;
    let mir = ace_lowering::hir_adapter::lower_checked(
        &product.compiled.checked,
        "d_main",
        ace_av_core::contract::fnv1a64(source.as_bytes()),
    )
    .map_err(|e| format!("shared lowering: {e:?}"))?;
    ace_av_core::mir_verify::verify_mir(&mir).map_err(|e| format!("shared MIR: {e:?}"))?;
    let units = ace_av::cache::split_units(&mir)?;
    for unit in &units {
        admit(unit, environment)?;
    }
    Ok((product, units))
}

pub fn signature(declaration: &SourceDecl) -> Result<Signature, String> {
    declaration.validate()?;
    let physical = |lane| {
        if lane == LaneKind::Bool {
            types::I8
        } else {
            types::I32
        }
    };
    let mut signature = Signature::new(CallConv::WindowsFastcall);
    if matches!(
        declaration.function.form,
        FnForm::Fallible { ctx: Some(_), .. }
    ) {
        signature.params.push(AbiParam::new(types::I64));
    }
    signature.params.extend(
        declaration
            .params()
            .iter()
            .map(|k| AbiParam::new(physical(*k))),
    );
    if matches!(declaration.function.form, FnForm::Fallible { .. }) {
        signature.params.push(AbiParam::new(types::I64));
        signature.returns.push(AbiParam::new(types::I32));
    } else {
        signature
            .returns
            .push(AbiParam::new(physical(declaration.result())));
    }
    Ok(signature)
}

pub fn admit(
    unit: &ace_av_core::mir::MirModule,
    environment: &AdmittedForeign,
) -> Result<(), String> {
    ace_av_core::mir_verify::verify_mir(unit).map_err(|e| format!("foreign.mir: {e:?}"))?;
    for f in &unit.funcs {
        for block in &f.blocks {
            for instruction in &block.instrs {
                if let ace_av_core::mir::MirInstr::ForeignCall { declaration, .. } = instruction {
                    if !environment.admits_declaration(declaration) {
                        return Err("foreign.authority: native declaration differs from the current admitted binding".into());
                    }
                }
            }
        }
    }
    Ok(())
}
