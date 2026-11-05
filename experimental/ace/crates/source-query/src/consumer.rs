//! PLAN-934 T-08: the shared ordinary-source consumer entry.
//!
//! Every consumer end (AV execution, native source worker, A2X store and
//! the Rust/C emitters) needs the same thing from the frontend: one
//! verified `CheckedModule` whose types came from the query facts. This
//! entry is that single door:
//!
//! ```text
//! parse → query facts per body → fact-supplied bind → emit_resolved_parts
//!       → verify (real HIR verification) → CheckedModule
//! ```
//!
//! No consumer invents its own compile path, and the HIR is the SAME
//! artifact the legacy entry produces (byte-identical for the finite
//! surface, which is what lets every downstream end share it).

use crate::apply::{bind_with_facts, AppliedFn, ApplyError, BindFactsError, BodyTypeSupply};
use crate::facts::FactsError;
use crate::session::QueryError;
use ace_core::atom_text::{Diagnostic, Span, Stage};
use ace_core::source::{
    emit_resolved_parts, parse_program_with, CompiledSource, SourceParts, SourceProfile,
};
use ace_core::verify::{verify_scalar, CheckedModule};

/// What the shared entry produced.
pub struct FactCompiled {
    pub profile: &'static str,
    pub input_hash: String,
    pub atom_text: String,
    pub checked: CheckedModule,
    pub applied: Vec<AppliedFn>,
    pub supply: BodyTypeSupply,
}

/// Why the shared entry failed. Every variant is a core/facts verdict, not
/// a consumer workaround.
#[derive(Debug)]
pub enum ConsumerError {
    Parse(String),
    Bind(BindFactsError),
    Emit(Vec<Diagnostic>),
    Verify(Vec<Diagnostic>),
}

impl std::fmt::Display for ConsumerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsumerError::Parse(m) => write!(f, "parse: {m}"),
            ConsumerError::Bind(e) => write!(f, "bind: {e}"),
            ConsumerError::Emit(ds) => write!(f, "emit: {}", first(ds)),
            ConsumerError::Verify(ds) => write!(f, "verify: {}", first(ds)),
        }
    }
}

fn first(ds: &[Diagnostic]) -> String {
    ds.first()
        .map(|d| format!("{}: {}", d.code, d.message))
        .unwrap_or_else(|| "unknown".into())
}

/// Compile one ordinary scalar source through the fact layer.
pub fn compile_via_facts(
    src: &str,
    module_path: &str,
    capabilities: &[String],
    package: &str,
    module: &str,
    session: &mut crate::session::SourceQuerySession,
    profile: SourceProfile,
) -> Result<FactCompiled, ConsumerError> {
    let program = parse_program_with(src, profile).map_err(|d| ConsumerError::Parse(d.message))?;
    session.set_capabilities(capabilities.iter().cloned());
    let (resolved, applied, supply) =
        bind_with_facts(&program, capabilities, package, module, session, profile)
            .map_err(ConsumerError::Bind)?;
    let parts =
        emit_resolved_parts(&resolved, src, module_path, profile).map_err(ConsumerError::Emit)?;
    let SourceParts {
        profile: prof,
        input_hash,
        atom_text,
        bundle,
        ..
    } = parts;
    let checked = verify_scalar(bundle).map_err(ConsumerError::Verify)?;
    Ok(FactCompiled {
        profile: prof,
        input_hash,
        atom_text,
        checked,
        applied,
        supply,
    })
}

// -- PLAN-934 T-08: the product entry every ACE consumer calls ----------

/// Which lane answered one product compile.
///
/// `Facts` is the only lane that produces a [`CompiledSource`]. `Diagnostic`
/// is a REJECTION: the query layer reported a language diagnostic (a page
/// carrying solver diagnostics or incomplete coverage), and the canonical
/// language diagnostics for that rejection come from the core frontend's
/// own diagnostic path. It produces no HIR, installs no facts and caches
/// nothing, so it can never become a hidden type-production fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompileLane {
    Facts,
    /// PLAN-935: the instances lane answered — the module carried templates,
    /// they were concretized into demanded instances, and the fact path ran
    /// on the desugared concrete program.
    Instances,
    Diagnostic,
}

/// The product-facing result: the compiled source plus the fact-layer
/// provenance a host needs (lane, per-function application, and the type
/// supply for a retained environment built from the same facts).
pub struct ProductCompile {
    pub compiled: CompiledSource,
    pub lane: CompileLane,
    pub applied: Vec<AppliedFn>,
    /// The admitted fact supply of every body of this module. A host that
    /// retains a checked environment (AV/native/A2X warm sessions) passes
    /// it to `ace_core::source::resolve_env_with_supply`, so the retained
    /// environment is seeded from the same facts as the emitted HIR.
    pub supply: BodyTypeSupply,
    /// PLAN-935: the instances this compile produced (empty outside the
    /// instances lane). Hosts that pin per-instance implementation versions
    /// read their identity from here — never from the generated fn name.
    pub instances: Vec<crate::instances::InstanceRecord>,
    /// PLAN-935: the program the fact path actually ran on — the DESUGARED
    /// concrete program under the instances lane (templates removed,
    /// instances placed, calls rebound), the parsed program otherwise.
    /// Hosts that retain an environment (A2X) build it from THIS program,
    /// never from a second caps-blind parse.
    pub program: ace_core::source::ast::Program,
}

impl ProductCompile {
    /// Translate the product result into the legacy shape hosts already
    /// consume (`CompiledSource`), keeping the provenance available.
    pub fn into_compiled(self) -> CompiledSource {
        self.compiled
    }
}

/// PLAN-934 T-08: compile one ordinary scalar source for a PRODUCT
/// consumer (AV driver/session, native source driver, A2X cold assemble).
///
/// This is the single entry those hosts call instead of
/// `ace_core::source::compile_scalar_source`. On success every type comes
/// from the query facts; on a language rejection the canonical diagnostics
/// are reported unchanged (code/stage/message/span), so consumer-visible
/// behavior does not drift while the type production authority moves.
///
/// PLAN-935（T-02）：capabilities 携带 `ac.source.instances.scalar/v1` 时走
/// 实例 lane——统一前门以 `scalar-instances` 能力面解析，模板按调用点实
/// 参实例化，事实路径跑在**脱糖具体程序**上（模板绝不进 resolve/emit）。
/// plain 模块（无模板）经实例 lane 与 plain 面逐字节同产物。
pub fn compile_scalar_product(
    src: &str,
    module_path: &str,
    capabilities: &[String],
    package: &str,
    module: &str,
    session: &mut crate::session::SourceQuerySession,
) -> Result<ProductCompile, Vec<Diagnostic>> {
    use ace_core::source::cst::{
        parse_front_with_caps, FrontBudget, FrontKind, FrontOutcome, FrontRequest,
        INSTANCES_CAPABILITY,
    };
    let instances_lane = capabilities.iter().any(|c| c == INSTANCES_CAPABILITY);
    let (program, instance_records) = if instances_lane {
        match parse_front_with_caps(
            FrontRequest {
                src,
                profile: SourceProfile::Scalar,
                kind: FrontKind::WholeModule,
                budget: FrontBudget::WORK_OWNED_64MIB,
            },
            capabilities,
        ) {
            FrontOutcome::Ok(ok) => {
                match crate::instances::instantiate_program(
                    &ok.program,
                    package,
                    module,
                    capabilities,
                ) {
                    Ok(out) => (out.program, out.instances),
                    Err(ds) => return Err(ds),
                }
            }
            FrontOutcome::Diag(ds) => return Err(ds),
            FrontOutcome::ResourceExhausted { what, limit } => {
                return Err(vec![Diagnostic::new(
                    Stage::Text,
                    "source.resource",
                    format!("resource exhausted: {what} (limit {limit})"),
                    Span { start: 0, end: 0 },
                )])
            }
            FrontOutcome::StaleForeign(m) | FrontOutcome::Infra(m) => {
                return Err(vec![Diagnostic::new(
                    Stage::Text,
                    "source.infra",
                    m,
                    Span { start: 0, end: 0 },
                )])
            }
        }
    } else {
        (
            parse_program_with(src, SourceProfile::Scalar).map_err(|d| vec![d])?,
            Vec::new(),
        )
    };
    // PLAN-935: the program the facts ran on travels with the product
    // (desugared under the instances lane) so env-retaining hosts never
    // re-parse caps-blind.
    let program_out = program.clone();
    let lane = if instance_records.is_empty() {
        CompileLane::Facts
    } else {
        CompileLane::Instances
    };
    // The capability set is part of every query key AND decides which
    // capability intrinsics the neutral input carries, so the session is
    // aligned with the compile request before anything is queried.
    session.set_capabilities(capabilities.iter().cloned());
    match bind_with_facts(
        &program,
        capabilities,
        package,
        module,
        session,
        SourceProfile::Scalar,
    ) {
        Ok((resolved, applied, supply)) => {
            let parts = emit_resolved_parts(&resolved, src, module_path, SourceProfile::Scalar)?;
            let SourceParts {
                profile,
                input_hash,
                atom_text,
                bundle,
                ..
            } = parts;
            let checked = verify_scalar(bundle)?;
            Ok(ProductCompile {
                compiled: CompiledSource {
                    profile,
                    input_hash,
                    atom_text,
                    checked,
                },
                lane,
                applied,
                supply,
                instances: instance_records,
                program: program_out,
            })
        }
        Err(e) => Err(rejection_verdict(
            src,
            module_path,
            capabilities,
            session,
            e,
            instances_lane,
        )),
    }
}

/// Turn a fact-layer rejection into the diagnostics the product reports.
///
/// * a CORE bind/verify verdict (bad fact, missing fact, real source
///   error) is returned as-is — core owns those diagnostics;
/// * a query page that carries the solver's own language diagnostic (or
///   cannot cover the body) is a LANGUAGE rejection: the backend's
///   canonical diagnostic path answers with the exact code/stage/message/
///   span the product reported before this wiring, and the fact layer
///   records the verdict (see `Counters::diagnostic_verdicts`);
/// * anything else (a construct outside the finite query surface, solver
///   resource state, cancellation) is an INFRASTRUCTURE rejection: it is
///   reported as `source.unsupported`/`source.fact-resource` at the Text
///   stage and never silently re-routed to the legacy type engine.
fn rejection_verdict(
    src: &str,
    module_path: &str,
    capabilities: &[String],
    session: &mut crate::session::SourceQuerySession,
    err: BindFactsError,
    instances_lane: bool,
) -> Vec<Diagnostic> {
    let language_verdict = match &err {
        BindFactsError::Apply(ApplyError::Facts(FactsError::Diagnostics { .. }))
        | BindFactsError::Apply(ApplyError::Facts(FactsError::IncompleteCoverage { .. })) => true,
        _ => false,
    };
    if !language_verdict {
        return match err {
            BindFactsError::Resolve(ds) => ds,
            other => vec![infra_diagnostic(&other)],
        };
    }
    // PLAN-935：实例 lane 不咨询 legacy 诊断 oracle——oracle 前端解析不了
    // 模板拼写（其裁决会是语法错，不是本面的规范诊断）。模板承载模块是
    // 新面：事实层的语言裁决即规范面（source.instances-* / 事实页诊断）。
    if instances_lane {
        return vec![Diagnostic::new(
            Stage::Verify,
            "source.instances-diagnostic",
            format!(
                "the query layer rejected this template module (`{err}`); the instances lane \
                 owns the canonical diagnostics for this surface"
            ),
            Span { start: 0, end: 0 },
        )];
    }
    session.note_diagnostic_verdict();
    // The canonical diagnostic oracle: the core frontend's own diagnostic
    // path. It produces a diagnostic verdict only — no HIR, no facts.
    match ace_core::source::compile_scalar_source(src, module_path, capabilities) {
        Err(ds) => ds,
        Ok(_) => vec![Diagnostic::new(
            Stage::Verify,
            "source.fact-diagnostic",
            format!(
                "the query layer rejected this module (`{err}`) but the frontend's diagnostic \
                 path accepts it; refusing to silently fall back to legacy type production"
            ),
            Span { start: 0, end: 0 },
        )],
    }
}

/// A controlled infrastructure rejection of the finite query surface.
fn infra_diagnostic(err: &BindFactsError) -> Diagnostic {
    match err {
        BindFactsError::Query(QueryError::Produce(crate::produce::ProduceError::Cancelled)) => {
            Diagnostic::new(
                Stage::Text,
                "source.fact-cancelled",
                "the semantic query was cancelled",
                Span { start: 0, end: 0 },
            )
        }
        BindFactsError::Query(QueryError::Produce(crate::produce::ProduceError::Resource(m))) => {
            Diagnostic::new(
                Stage::Text,
                "source.fact-resource",
                format!("semantic query resource state: {m}"),
                Span { start: 0, end: 0 },
            )
        }
        BindFactsError::Query(QueryError::SolverDisabled { query, owner }) => Diagnostic::new(
            Stage::Text,
            "source.fact-resource",
            format!("{query} for `{owner}` misses the query cache and the solver is disabled"),
            Span { start: 0, end: 0 },
        ),
        other => Diagnostic::new(
            Stage::Text,
            "source.unsupported",
            format!("{other}"),
            Span { start: 0, end: 0 },
        ),
    }
}
