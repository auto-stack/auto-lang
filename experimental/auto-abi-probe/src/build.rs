//! Producer / consumer recipes (plan §5.2): each producer is an
//! independent command producing its object plus the descriptor of its
//! public boundary; the consumer builds AFTER reading that boundary —
//! admission ([`crate::validate`]) gates the consumer emit, so the
//! consumer never shares signature pointers with the producer in one
//! codegen unit and an incompatible boundary stops the build with zero
//! native invocations.

use crate::decl::FnDecl;
use crate::descriptor::ModuleDescriptor;
use crate::emit::{emit_callee_suite, emit_caller_suite, SuiteCheck};
use crate::link::ProbeError;
use crate::validate::{admit, Admission, ConsumerExpectation, Rejection};

/// A produced unit: the object bytes plus the descriptor describing its
/// public boundary (what a consumer must admit before linking).
pub struct Produced {
    pub obj: Vec<u8>,
    pub descriptor: ModuleDescriptor,
}

/// Derive the module descriptor PURELY from the declaration set —
/// NO emission happens here (F-797-R4-08): admission must not secretly
/// run the emitter on the cache-hit path, and `compute_calls` counts
/// every real producer invocation.
pub fn descriptor_for_defs(defs: &[FnDecl]) -> ModuleDescriptor {
    let mut descriptor = ModuleDescriptor::probe_module();
    descriptor.symbols = defs
        .iter()
        .map(|d| crate::descriptor::SymbolEntry {
            name: d.symbol.to_string(),
            form: decl_form(d),
        })
        .collect();
    descriptor
}

/// Producer recipe: emit the callee-module object and derive its
/// descriptor (the descriptor half is [`descriptor_for_defs`]).
pub fn produce_callee_suite(
    module_name: &str,
    defs: &[FnDecl],
    import_inner: bool,
) -> Result<Produced, ProbeError> {
    let obj = emit_callee_suite(module_name, defs, import_inner).map_err(ProbeError)?;
    Ok(Produced {
        obj,
        descriptor: descriptor_for_defs(defs),
    })
}

fn decl_form(d: &FnDecl) -> crate::descriptor::SymbolForm {
    use crate::descriptor::{ParamKind, RetKind, SymbolForm};
    let params: Vec<ParamKind> = d
        .params
        .iter()
        .map(|p| match p {
            crate::decl::AbiScalar::I32 => ParamKind::I32,
            crate::decl::AbiScalar::I64 => ParamKind::I64,
            crate::decl::AbiScalar::F64 => ParamKind::F64,
            crate::decl::AbiScalar::Ptr => ParamKind::I64, // view/out lane
        })
        .collect();
    match d.body {
        crate::decl::BodyKind::FallibleDivV1 => SymbolForm::Fallible {
            // explicit nullable context first (F-797-R4-02 / §2)
            ctx: Some(true),
            params: params[1..3].to_vec(),
            outs: vec![ParamKind::I64],
        },
        crate::decl::BodyKind::RecSumV1 => SymbolForm::Pure {
            // F-797-R4-02 (R5): record views are DESCRIBED as record
            // views in the production builder — not collapsed to i64.
            params: vec![ParamKind::RecView {
                rec: "RecMix".into(),
                view: crate::descriptor::ViewRole::Const,
            }],
            ret: RetKind::I64,
        },
        _ => SymbolForm::Pure {
            params,
            ret: match d.ret {
                crate::decl::AbiScalar::I32 => RetKind::I32,
                crate::decl::AbiScalar::F64 => RetKind::F64,
                _ => RetKind::I64,
            },
        },
    }
}

/// Consumer recipe: admit the producer boundary FIRST (unknown version /
/// wrong target / packing / symbols / services stop here — native calls
/// 0), then emit the caller object for the admitted surface.
pub fn consume_and_build_caller(
    descriptor_bytes: &[u8],
    expect: &ConsumerExpectation,
    module_name: &str,
    checks: &[SuiteCheck],
) -> Result<(Admission, Vec<u8>), ConsumeError> {
    let admission = admit(descriptor_bytes, expect).map_err(ConsumeError::Rejected)?;
    let obj = emit_caller_suite(module_name, checks)
        .map_err(|e| ConsumeError::Probe(ProbeError(format!("caller emit: {e}"))))?;
    Ok((admission, obj))
}

#[derive(Debug)]
pub enum ConsumeError {
    Rejected(Rejection),
    Probe(ProbeError),
}

impl std::fmt::Display for ConsumeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsumeError::Rejected(r) => write!(f, "{r}"),
            ConsumeError::Probe(p) => write!(f, "{p}"),
        }
    }
}

/// Consumer expectation for a whole suite module: every declared symbol
/// with its exact signature form (the consumer's own view of the
/// boundary it intends to call).
pub fn suite_expectation(defs: &[FnDecl]) -> ConsumerExpectation {
    let mut e = crate::validate::probe_expectation();
    e.required_symbols = defs
        .iter()
        .map(|d| (d.symbol.to_string(), decl_form(d).signature()))
        .collect();
    e
}
