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
    // PLAN-939: a manifest carrying RESOURCE entries compiles in the
    // text-record domain and lowers through the owned-profile shared
    // entry (str views + nominal records live there); scalar manifests
    // keep the exact 938 product and lowering path.
    let resource_manifest = environment
        .source_declarations_ref()
        .iter()
        .any(|d| d.resource().is_some());
    let (product, mir) = if resource_manifest {
        let product = ace_source_query::compile_text_record_product(
            source,
            module,
            capabilities,
            &package,
            &owner,
            session,
        )
        .map_err(|ds| format!("source: {ds:?}"))?;
        let mir = ace_lowering::hir_adapter::lower_checked_owned(
            &product.compiled.checked,
            "d_main",
            ace_av_core::contract::fnv1a64(source.as_bytes()),
        )
        .map_err(|e| format!("shared lowering: {e:?}"))?;
        (product, mir)
    } else {
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
        (product, mir)
    };
    ace_av_core::mir_verify::verify_mir(&mir).map_err(|e| format!("shared MIR: {e:?}"))?;
    let units = ace_av::cache::split_units(&mir)?;
    for unit in &units {
        admit(unit, environment)?;
    }
    Ok((product, units))
}

pub fn signature(declaration: &SourceDecl) -> Result<Signature, String> {
    declaration.validate()?;
    if let Some(res) = declaration.resource() {
        return resource_signature(res);
    }
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

/// PLAN-939 physical resource signature (Windows x64): pointer lanes are
/// I64; the view pair is (ptr, u64 len); the callback lane is an I64 fn
/// pointer whose ctx value is the SIGN-EXTENDED i32 seed param adjacent to
/// it; handles cross as I64; the fallible form appends the trailing out
/// pointer and returns the i32 status.
pub fn resource_signature(res: &ace_foreign_contract::resource::ResourceSig) -> Result<Signature, String> {
    use ace_foreign_contract::resource::{ResKind, ResOut, ResourceForm};
    res.validate()?;
    let mut signature = Signature::new(CallConv::WindowsFastcall);
    if let ResourceForm::Fallible { ctx: Some(_), .. } = res.form {
        signature.params.push(AbiParam::new(types::I64));
    }
    for (i, p) in res.params.iter().enumerate() {
        let after_cb = i > 0 && matches!(res.params[i - 1], ResKind::Callback { ctx: true });
        match p {
            ResKind::I32 if after_cb => signature.params.push(AbiParam::new(types::I64)),
            ResKind::I32 => signature.params.push(AbiParam::new(types::I32)),
            ResKind::Bool => signature.params.push(AbiParam::new(types::I8)),
            ResKind::ViewText => {
                signature.params.push(AbiParam::new(types::I64));
                signature.params.push(AbiParam::new(types::I64));
            }
            ResKind::Rec { .. } | ResKind::Handle { .. } => {
                signature.params.push(AbiParam::new(types::I64))
            }
            // PLAN-939 S07 typed-peer wide lanes (real physical forms:
            // element view = ptr + u64 element count; bare wide i64).
            ResKind::BufViewI64 => {
                signature.params.push(AbiParam::new(types::I64));
                signature.params.push(AbiParam::new(types::I64));
            }
            ResKind::I64 => signature.params.push(AbiParam::new(types::I64)),
            ResKind::Callback { .. } => signature.params.push(AbiParam::new(types::I64)),
        }
    }
    match &res.form {
        ResourceForm::Pure { ret } => {
            signature.returns.push(AbiParam::new(match ret {
                ResOut::I32 => types::I32,
                ResOut::Bool => types::I8,
                ResOut::I64 | ResOut::U64 | ResOut::Handle { .. } => types::I64,
            }));
        }
        ResourceForm::Fallible { out, .. } => {
            // The value out is written through a trailing POINTER lane
            // (pointee width by the out lane); every fallible form also
            // carries the caller status cell the 938 contract spells.
            let _ = out;
            signature.params.push(AbiParam::new(types::I64));
            signature.returns.push(AbiParam::new(types::I32));
        }
    }
    Ok(signature)
}

pub fn admit(
    unit: &ace_av_core::mir::MirModule,
    environment: &AdmittedForeign,
) -> Result<(), String> {
    ace_av_core::mir_verify::validate_mir(unit).map_err(|e| format!("foreign.mir: {e:?}"))?;
    admit_declarations(unit, environment)
}

/// Source scalar composition charges the real verifier's initialization
/// workspace to its shared staging pool, before bases/CFG/matrix allocation.
pub fn admit_reserved(
    unit: &ace_av_core::mir::MirModule,
    environment: &AdmittedForeign,
    resources: &ace_source_query::resources::Resources,
) -> Result<(), crate::MnError> {
    use ace_av_core::{contract::ResourceKind, mir_verify::MirVerifyError};
    use ace_source_query::resources::Charge;
    if unit.profile != ace_av_core::contract::AV_PROFILE
        && unit.profile != ace_av_core::contract::AV_OWNED_PROFILE
    {
        return Err(crate::MnError::Unsupported(
            "foreign.profile",
            "source foreign analysis admission requires the scalar or owned MIR profile".into(),
        ));
    }
    let mut analysis = resources
        .reserve(Charge::staging(0))
        .map_err(|e| crate::MnError::Resource("verify-memory", e.to_string()))?;
    ace_av_core::mir_verify::validate_mir_with_analysis_reservation(unit, |bytes| {
        analysis
            .resize(Charge::staging(bytes))
            .map_err(|e| MirVerifyError::Resource {
                kind: ResourceKind::VerifyMemory,
                message: e.to_string(),
            })
    })
    .map_err(|e| match e {
        MirVerifyError::Resource { .. } => {
            crate::MnError::Resource("verify-memory", format!("foreign.mir: {e:?}"))
        }
        _ => crate::MnError::Unsupported("foreign.authority", format!("foreign.mir: {e:?}")),
    })?;
    admit_declarations(unit, environment)
        .map_err(|e| crate::MnError::Unsupported("foreign.authority", e))
}

fn admit_declarations(
    unit: &ace_av_core::mir::MirModule,
    environment: &AdmittedForeign,
) -> Result<(), String> {
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
