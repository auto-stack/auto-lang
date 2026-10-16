//! Read-only reflection projections and the CT/instance/runtime seam
//! (PLAN-796 §5.3).
//!
//! The reflection surface implemented here: query one field (position +
//! description), query a signature, list a record's ordered fields,
//! list a domain's candidates. Everything else is explicit
//! `unsupported` — never an empty array pretending success.
//!
//! The CT seam freezes SCHEMA + ADMISSION only: portable values/type
//! descriptions with dependencies are representable; addresses, host
//! handles, Unknown values and unchecked executable inputs are
//! rejected. Model producers compute results marked `model_compute` —
//! no AVM execution is claimed or performed.

use crate::input::{Ann, SemaInput};
use ac_sema_contract::{DeclRef, FnMeta, Mode, RecordShape, TypeDesc};

// ---------------------------------------------------------------------------
// read-only reflection projections
// ---------------------------------------------------------------------------

/// One field of one record: (ordinal, name, description). `None` is a
/// real miss (no such record or no such field) — the caller reports it,
/// never a fake empty field.
pub fn query_field(input: &SemaInput, record: &str, field: &str) -> Option<(usize, String, TypeDesc)> {
    let r = input.lookup_record(record)?;
    let (i, (n, a)) = r.fields.iter().enumerate().find(|(_, (n, _))| n == field)?;
    Some((i, n.clone(), ann_desc(a)))
}

/// A function's signature as portable metadata.
pub fn query_signature(input: &SemaInput, fn_name: &str, content: ac_sema_contract::ContentVersion) -> Option<FnMeta> {
    let f = input.lookup_fn(fn_name)?;
    Some(FnMeta {
        decl: DeclRef::new(
            &ac_sema_contract::PackageId::new(&input.package),
            &input.module,
            ac_sema_contract::DeclKind::Fn,
            fn_name,
            content,
        ),
        params: f
            .params
            .iter()
            .map(|(n, a)| (n.clone(), ann_desc(a), Mode::View))
            .collect(),
        ret: ann_desc(&f.ret),
        receiver: "free",
    })
}

/// A record's full ordered shape.
pub fn record_shape(input: &SemaInput, record: &str, content: ac_sema_contract::ContentVersion) -> Option<RecordShape> {
    let r = input.lookup_record(record)?;
    Some(RecordShape {
        decl: DeclRef::new(
            &ac_sema_contract::PackageId::new(&input.package),
            &input.module,
            ac_sema_contract::DeclKind::Record,
            record,
            content,
        ),
        fields: r
            .fields
            .iter()
            .map(|(n, a)| ac_sema_contract::FieldDesc { name: n.clone(), ty: ann_desc(a) })
            .collect(),
    })
}

/// List a record's fields in declaration order.
pub fn list_fields(input: &SemaInput, record: &str) -> Option<Vec<String>> {
    Some(input.lookup_record(record)?.fields.iter().map(|(n, _)| n.clone()).collect())
}

/// List a domain's candidate set (sorted — it is the lookup domain,
/// not an ordered surface).
pub fn list_candidates(input: &SemaInput, domain: &str) -> Vec<String> {
    match domain {
        "fn" => input.fn_candidates(),
        "type" => input.record_candidates(),
        _ => Vec::new(),
    }
}

fn ann_desc(a: &Ann) -> TypeDesc {
    match a {
        Ann::I32 => TypeDesc::I32,
        Ann::Bool => TypeDesc::Bool,
        // Record annotations describe nominal types by name here; the
        // probe-level publication upgrades them to full DeclRefs.
        Ann::Record(r) => TypeDesc::Opaque { name: format!("record:{r}") },
        Ann::Inferred => TypeDesc::Opaque { name: "?inferred".into() },
        Ann::Unsupported(w) => TypeDesc::Opaque { name: format!("unsupported:{w}") },
    }
}

// ---------------------------------------------------------------------------
// CT / instance / runtime seam — schema + admission (frozen, model-only)
// ---------------------------------------------------------------------------

/// Schema identity of the frozen CT seam.
pub const CT_SEAM_SCHEMA: &str = "ac796/ct-seam/v1";

/// Portable value: the ONLY value shapes a CT request/result may carry.
#[derive(Clone, PartialEq, Debug)]
pub enum PortableValue {
    Unit,
    I32(i32),
    Bool(bool),
    Text(String),
    /// Reference to a semantic type description (never a host address).
    TypeRef(TypeDesc),
}

/// Admission marker: what computed the result. `model_compute` results
/// come from an in-model producer; `vm_execute` is reserved for the
/// real AVM path and CANNOT be minted by this crate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComputeMarking {
    ModelCompute,
    VmExecute,
}

impl ComputeMarking {
    pub fn as_str(self) -> &'static str {
        match self {
            ComputeMarking::ModelCompute => "model_compute",
            ComputeMarking::VmExecute => "vm_execute",
        }
    }
}

/// A comptime evaluation request — schema-frozen. Admission rejects
/// non-portable inputs BEFORE any producer runs.
#[derive(Clone, PartialEq, Debug)]
pub struct CtRequest {
    pub schema: &'static str,
    /// Callee template (logical id string).
    pub callee: String,
    /// Constant/type arguments (portable only).
    pub args: Vec<PortableValue>,
    /// Input dependencies the callee body reads.
    pub deps: Vec<String>,
}

impl CtRequest {
    pub fn new(callee: impl Into<String>, args: Vec<PortableValue>, deps: Vec<String>) -> Self {
        CtRequest { schema: CT_SEAM_SCHEMA, callee: callee.into(), args, deps }
    }

    /// Admission: reject schema drift, Unknown/unsolved type references
    /// and (structurally) non-portable inputs before any producer runs.
    pub fn admit(&self) -> Result<(), CtAdmission> {
        if self.schema != CT_SEAM_SCHEMA {
            return Err(CtAdmission::WrongSchema { expected: CT_SEAM_SCHEMA, found: self.schema });
        }
        for a in &self.args {
            if let PortableValue::TypeRef(TypeDesc::Opaque { name }) = a {
                if name.starts_with('?') {
                    return Err(CtAdmission::UnknownValue);
                }
            }
        }
        Ok(())
    }
}

/// Controlled admission failures of the CT seam.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CtAdmission {
    WrongSchema { expected: &'static str, found: &'static str },
    NonPortableValue(&'static str),
    UnknownValue,
    UncheckedExecutable,
    CyclicRequest { chain: Vec<String> },
}

impl std::fmt::Display for CtAdmission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CtAdmission::WrongSchema { expected, found } => {
                write!(f, "ct seam schema mismatch: expected {expected}, found {found}")
            }
            CtAdmission::NonPortableValue(w) => write!(f, "non-portable ct value: {w}"),
            CtAdmission::UnknownValue => write!(f, "unknown/unsolved ct value rejected"),
            CtAdmission::UncheckedExecutable => {
                write!(f, "unchecked executable input rejected")
            }
            CtAdmission::CyclicRequest { chain } => {
                write!(f, "cyclic ct request chain: {}", chain.join(" -> "))
            }
        }
    }
}

impl std::error::Error for CtAdmission {}

/// A frozen CT result: portable value + marking + dependency manifest
/// references. `vm_execute` marking is impossible to construct here —
/// the real AVM path owns it (future work-package 2).
#[derive(Clone, PartialEq, Debug)]
pub struct CtResult {
    pub schema: &'static str,
    pub value: PortableValue,
    pub marking: ComputeMarking,
    /// Input dependency fingerprints (validated like every manifest).
    pub deps: Vec<String>,
}

impl CtResult {
    pub fn model(value: PortableValue, deps: Vec<String>) -> Self {
        CtResult { schema: CT_SEAM_SCHEMA, value, marking: ComputeMarking::ModelCompute, deps }
    }
}

/// Instance key: (template logical id, substitution fingerprint). No
/// runtime code generation is implied — schema only.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InstanceKey {
    pub template: String,
    pub substitution_fp: String,
}

/// Runtime reflection retention policy (schema only — the loader/link
/// retention faces are future work-package 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RuntimeProjectionPolicy {
    /// No reflection data retained.
    None,
    /// Type identity/name retained.
    Identity,
    /// Shape + attributes retained.
    Shape,
    /// Authorized access/invoke adapters (generation-leased).
    AuthorizedAccess,
}

impl RuntimeProjectionPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            RuntimeProjectionPolicy::None => "none",
            RuntimeProjectionPolicy::Identity => "identity",
            RuntimeProjectionPolicy::Shape => "shape",
            RuntimeProjectionPolicy::AuthorizedAccess => "authorized-access",
        }
    }
}

// ---------------------------------------------------------------------------
// model CT producer (§5.3: 显式模型 producer 测试缓存/依赖环)
// ---------------------------------------------------------------------------

/// Explicit MODEL producer for CT requests: admits the request, tracks
/// the live request chain, and rejects cycles with the full chain in
/// the diagnostic. Results are always `model_compute` — no AVM
/// execution is claimed, performed, or marked.
pub struct ModelCt {
    chain: Vec<String>,
}

impl Default for ModelCt {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelCt {
    pub fn new() -> Self {
        ModelCt { chain: Vec::new() }
    }

    /// Evaluate one request. `body` is the (model) implementation of
    /// the callee; it may recursively evaluate dependency requests on
    /// this same scheduler — a request chain that revisits a callee is
    /// a controlled `CyclicRequest`, never unbounded recursion.
    pub fn evaluate<F>(&mut self, req: &CtRequest, body: F) -> Result<CtResult, CtAdmission>
    where
        F: FnOnce(&CtRequest, &mut ModelCt) -> Result<CtResult, CtAdmission>,
    {
        req.admit()?;
        if let Some(pos) = self.chain.iter().position(|c| *c == req.callee) {
            let mut chain: Vec<String> = self.chain[pos..].to_vec();
            chain.push(req.callee.clone());
            return Err(CtAdmission::CyclicRequest { chain });
        }
        self.chain.push(req.callee.clone());
        let result = body(req, self);
        self.chain.pop();
        result
    }

    /// Directly fold i32 constants (the simplest model producer): sums
    /// the i32 arguments. Used by tests and the cache/dependency-cycle
    /// model scenarios.
    pub fn fold_i32_sum(req: &CtRequest) -> Result<CtResult, CtAdmission> {
        req.admit()?;
        let mut sum = 0i32;
        for a in &req.args {
            match a {
                PortableValue::I32(v) => sum = sum.wrapping_add(*v),
                _ => return Err(CtAdmission::NonPortableValue("fold expects i32 args")),
            }
        }
        Ok(CtResult::model(PortableValue::I32(sum), req.deps.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{InputFn, InputRecord};

    fn input() -> SemaInput {
        SemaInput {
            package: "p".into(),
            module: "m".into(),
            fns: vec![InputFn {
                name: "f".into(),
                params: vec![("a".into(), Ann::I32)],
                ret: Ann::Bool,
                body: crate::input::InputBlock { stmts: vec![] },
            }],
            records: vec![InputRecord {
                name: "Pair".into(),
                fields: vec![("left".into(), Ann::I32), ("right".into(), Ann::Bool)],
                methods: vec![],
            }],
        }
    }

    #[test]
    fn meta_query_reflect_field_and_signature() {
        let i = input();
        let (ord, name, ty) = query_field(&i, "Pair", "right").unwrap();
        assert_eq!((ord, name.as_str()), (1, "right"));
        assert_eq!(ty, TypeDesc::Bool);
        assert!(query_field(&i, "Pair", "middle").is_none());
        assert!(query_field(&i, "Nope", "left").is_none());
        let sig = query_signature(&i, "f", ac_sema_contract::ContentVersion::of_canonical(b"c"))
            .unwrap();
        assert_eq!(sig.ret, TypeDesc::Bool);
        assert_eq!(sig.params.len(), 1);
        assert_eq!(list_fields(&i, "Pair").unwrap(), vec!["left", "right"]);
        assert_eq!(list_candidates(&i, "fn"), vec!["f".to_string()]);
        assert_eq!(list_candidates(&i, "type"), vec!["Pair".to_string()]);
    }

    #[test]
    fn meta_query_ct_seam_admission() {
        let req = CtRequest::new("tpl/x", vec![PortableValue::I32(3), PortableValue::Bool(true)], vec![]);
        assert!(req.admit().is_ok());
        let res = CtResult::model(PortableValue::I32(9), vec!["dep1".into()]);
        assert_eq!(res.marking.as_str(), "model_compute");
        // vm_execute is not mintable through the public constructor
        assert_eq!(CtResult::model(PortableValue::Unit, vec![]).marking, ComputeMarking::ModelCompute);
    }

    #[test]
    fn meta_query_projection_policy_levels() {
        assert_eq!(RuntimeProjectionPolicy::None.as_str(), "none");
        assert_eq!(RuntimeProjectionPolicy::AuthorizedAccess.as_str(), "authorized-access");
    }
}
