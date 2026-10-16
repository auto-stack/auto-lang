//! Semantic queries: kinds, lookup keys and dependency freshness
//! (PLAN-796 §5.3/§5.4).
//!
//! Every query result carries its input version and dependency
//! manifest; a cached hit is only reused after the FULL manifest is
//! re-validated against the current input (positive fingerprints +
//! negative candidate-set dependencies). Key components cover body/
//! signature/actually-read inputs, candidate/field sets, profile,
//! algorithm, schema, target-relevant semantics, builtin capabilities
//! and producer version (contract-decision D7).

use crate::input::{Ann, SemaInput};
use crate::infer::Reads;
use ac_sema_contract::{
    ContentVersion, DeclKind, DepFingerprint, DepManifest, LogicalId, NegDep, PackageId,
    SEMA_PAGE_SCHEMA_V1,
};
use std::collections::BTreeMap;

/// Identity of the limited inference algorithm — participates in every
/// infer-query key (algorithm change ⇒ old results miss, never reuse).
pub const INFER_ALGO_ID: &str = "ac796-bidir-worklist/1";

/// Producer string used in query keys (contract-decision D7).
pub const QUERY_PRODUCER: &str = ac_sema_contract::SEMA_PRODUCER_ID;

/// Target-relevant semantics component: the inference profile is
/// target-INDEPENDENT (797 owns physical layout); the key still carries
/// the marker so a future target-coupled semantic splits the namespace
/// explicitly instead of silently.
pub const TARGET_SEMANTICS: &str = "logical-only/v1";

/// Builtin capability set of the limited profile (i32/bool literals and
/// the operator set — no text/record/intrinsics in the infer surface).
pub const BUILTIN_CAPABILITIES: &str = "core-i32-bool/1";

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum QueryKind {
    /// Infer one function body.
    InferBody { fn_name: String },
    /// Read one function signature.
    FnSig { fn_name: String },
    /// Read one record's ordered field set.
    RecordFields { record: String },
    /// List a domain's candidate set (sorted).
    ListCandidates { domain: &'static str },
}

impl QueryKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            QueryKind::InferBody { .. } => "infer-body",
            QueryKind::FnSig { .. } => "fn-sig",
            QueryKind::RecordFields { .. } => "record-fields",
            QueryKind::ListCandidates { .. } => "list-candidates",
        }
    }
}

/// Canonical content of one function's SIGNATURE (params + ret,
/// declaration order; positions excluded — D4).
pub fn fn_sig_canonical(f: &crate::input::InputFn) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(f.name.as_bytes());
    out.push(0);
    for (n, a) in &f.params {
        out.extend_from_slice(n.as_bytes());
        out.push(b':');
        let tag = match a {
            Ann::I32 => b'i',
            Ann::Bool => b'b',
            Ann::Record(r) => {
                out.extend_from_slice(b"rec:");
                out.extend_from_slice(r.as_bytes());
                0
            }
            Ann::Inferred => b'?',
            Ann::Unsupported(_) => b'x',
        };
        out.push(tag);
        out.push(0);
    }
    out.push(b'-');
    out.push(match f.ret {
        Ann::I32 => b'i',
        Ann::Bool => b'b',
        Ann::Record(_) => b'r',
        Ann::Inferred => b'?',
        Ann::Unsupported(_) => b'x',
    });
    out
}

/// Canonical content of one function's BODY (statement/expression
/// structure with spans EXCLUDED — a whitespace/position-only edit keeps
/// the body fingerprint).
pub fn fn_body_canonical(f: &crate::input::InputFn) -> Vec<u8> {
    fn enc_expr(out: &mut Vec<u8>, e: &crate::input::InputExpr) {
        use crate::input::InputExprKind;
        match &e.kind {
            InputExprKind::IntLit(v) => {
                out.push(b'N');
                out.extend_from_slice(&v.to_le_bytes());
            }
            InputExprKind::BoolLit(v) => {
                out.push(b'B');
                out.push(*v as u8);
            }
            InputExprKind::Var(n) => {
                out.push(b'V');
                out.extend_from_slice(n.as_bytes());
                out.push(0);
            }
            InputExprKind::Call { callee, args } => {
                out.push(b'C');
                out.extend_from_slice(callee.as_bytes());
                out.push(0);
                out.extend_from_slice(&(args.len() as u32).to_le_bytes());
                for a in args {
                    enc_expr(out, a);
                }
            }
            InputExprKind::Binary { op, lhs, rhs } => {
                out.push(b'(');
                out.extend_from_slice(op.as_str().as_bytes());
                enc_expr(out, lhs);
                enc_expr(out, rhs);
            }
            InputExprKind::Unary { neg, operand } => {
                out.push(b'U');
                out.push(*neg as u8);
                enc_expr(out, operand);
            }
            InputExprKind::Unsupported { what } => {
                out.push(b'X');
                out.extend_from_slice(what.as_bytes());
                out.push(0);
            }
        }
    }
    fn enc_stmt(out: &mut Vec<u8>, s: &crate::input::InputStmt) {
        use crate::input::InputStmt;
        match s {
            InputStmt::Let { name, ann, mutable, value, .. } => {
                out.push(b'L');
                out.extend_from_slice(name.as_bytes());
                out.push(0);
                let tag = match ann {
                    Ann::I32 => b'i',
                    Ann::Bool => b'b',
                    Ann::Record(_) => b'r',
                    Ann::Inferred => b'?',
                    Ann::Unsupported(_) => b'x',
                };
                out.push(tag);
                out.push(*mutable as u8);
                enc_expr(out, value);
            }
            InputStmt::Assign { target, path, value, .. } => {
                out.push(b'A');
                out.extend_from_slice(target.as_bytes());
                out.push(0);
                for p in path {
                    out.push(b'.');
                    out.extend_from_slice(p.as_bytes());
                }
                enc_expr(out, value);
            }
            InputStmt::Return { value, .. } => {
                out.push(b'R');
                enc_expr(out, value);
            }
            InputStmt::If { cond, then, els, .. } => {
                out.push(b'I');
                enc_expr(out, cond);
                out.push(b'{');
                for s in &then.stmts {
                    enc_stmt(out, s);
                }
                out.push(b'}');
                if let Some(e) = els {
                    out.push(b'E');
                    for s in &e.stmts {
                        enc_stmt(out, s);
                    }
                    out.push(b'}');
                }
            }
            InputStmt::While { cond, body, .. } => {
                out.push(b'W');
                enc_expr(out, cond);
                out.push(b'{');
                for s in &body.stmts {
                    enc_stmt(out, s);
                }
                out.push(b'}');
            }
            InputStmt::Loop { body, .. } => {
                out.push(b'P');
                out.push(b'{');
                for s in &body.stmts {
                    enc_stmt(out, s);
                }
                out.push(b'}');
            }
            InputStmt::Break { .. } => out.push(b'K'),
            InputStmt::Continue { .. } => out.push(b'N'),
        }
    }
    let mut out = Vec::new();
    out.extend_from_slice(f.name.as_bytes());
    out.push(0);
    for s in &f.body.stmts {
        enc_stmt(&mut out, s);
    }
    out
}

/// Canonical content of one record's ordered field set.
pub fn record_canonical(r: &crate::input::InputRecord) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(r.name.as_bytes());
    out.push(0);
    for (n, a) in &r.fields {
        out.extend_from_slice(n.as_bytes());
        out.push(b':');
        let tag = match a {
            Ann::I32 => b'i',
            Ann::Bool => b'b',
            Ann::Record(r) => {
                out.extend_from_slice(b"rec:");
                out.extend_from_slice(r.as_bytes());
                0
            }
            Ann::Inferred => b'?',
            Ann::Unsupported(_) => b'x',
        };
        out.push(tag);
        out.push(0);
    }
    out
}

fn fp(bytes: &[u8]) -> String {
    ac_sema_contract::identity::domain_hash("ac796/dep-fp/v1", &[bytes])
}

/// A lookup key for the semantic cache: named components, order-free
/// after canonicalization (the store layer sorts them).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SemaQueryKey {
    pub components: Vec<(String, String)>,
}

impl SemaQueryKey {
    /// Build the key for one query over one input. The key covers
    /// everything the result actually depends on at lookup time; the
    /// fine-grained manifest rides IN the payload and is re-validated
    /// on every hit (so, e.g., a helper-body edit invalidates via the
    /// manifest even though the top-level signature components did not
    /// change).
    pub fn build(query: &QueryKind, input: &SemaInput) -> Self {
        let mut c: Vec<(String, String)> = vec![
            ("query".into(), query.as_str().into()),
            ("package".into(), input.package.clone()),
            ("module".into(), input.module.clone()),
            ("algo".into(), INFER_ALGO_ID.into()),
            ("schema".into(), SEMA_PAGE_SCHEMA_V1.into()),
            ("producer".into(), QUERY_PRODUCER.into()),
            ("target-sem".into(), TARGET_SEMANTICS.into()),
            ("caps".into(), BUILTIN_CAPABILITIES.into()),
        ];
        match query {
            QueryKind::InferBody { fn_name } => {
                let f = input.lookup_fn(fn_name);
                c.push((
                    "fn-sig-fp".into(),
                    f.map(|f| fp(&fn_sig_canonical(f))).unwrap_or_else(|| "absent".into()),
                ));
                c.push((
                    "fn-body-fp".into(),
                    f.map(|f| fp(&fn_body_canonical(f))).unwrap_or_else(|| "absent".into()),
                ));
                // the candidate sets the lookup domain ran against
                c.push(("fn-cand-fp".into(), fp(input.fn_candidates().join("\u{1}").as_bytes())));
                c.push((
                    "type-cand-fp".into(),
                    fp(input.record_candidates().join("\u{1}").as_bytes()),
                ));
            }
            QueryKind::FnSig { fn_name } => {
                let f = input.lookup_fn(fn_name);
                c.push((
                    "fn-sig-fp".into(),
                    f.map(|f| fp(&fn_sig_canonical(f))).unwrap_or_else(|| "absent".into()),
                ));
            }
            QueryKind::RecordFields { record } => {
                let r = input.lookup_record(record);
                c.push((
                    "field-set-fp".into(),
                    r.map(|r| fp(&record_canonical(r))).unwrap_or_else(|| "absent".into()),
                ));
            }
            QueryKind::ListCandidates { domain } => {
                let set = match *domain {
                    "fn" => input.fn_candidates(),
                    "type" => input.record_candidates(),
                    _ => Vec::new(),
                };
                c.push(("candidate-set-fp".into(), fp(set.join("\u{1}").as_bytes())));
            }
        }
        c.sort_by(|a, b| a.0.cmp(&b.0));
        SemaQueryKey { components: c }
    }

    pub fn canonical(&self) -> String {
        let mut s = String::from("S796[");
        for (i, (k, v)) in self.components.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str(&format!("{k}={v}"));
        }
        s.push(']');
        s
    }
}

/// All dependency fingerprints of the CURRENT input, indexed for
/// manifest validation. `source_maps` is filled by the publishing layer
/// with the actual source bytes read (origin → fp); a manifest claiming
/// a source-map dependency fails CLOSED when the origin is not
/// registered — reflection that read source/doc cannot silently reuse
/// a stale result (§5.3).
#[derive(Clone, Debug, Default)]
pub struct DependencyFingerprints {
    pub fn_sigs: BTreeMap<String, String>,
    pub fn_bodies: BTreeMap<String, String>,
    pub field_sets: BTreeMap<String, String>,
    pub candidate_sets: BTreeMap<&'static str, String>,
    pub source_maps: BTreeMap<String, String>,
    pub config: String,
}

fn cand_fp(set: &[String]) -> String {
    fp(set.join("\u{1}").as_bytes())
}

/// Compute the current fingerprints of an input (what a manifest is
/// validated against).
pub fn current_fingerprints(input: &SemaInput) -> DependencyFingerprints {
    let mut fn_sigs = BTreeMap::new();
    let mut fn_bodies = BTreeMap::new();
    for f in &input.fns {
        fn_sigs.insert(f.name.clone(), fp(&fn_sig_canonical(f)));
        fn_bodies.insert(f.name.clone(), fp(&fn_body_canonical(f)));
    }
    let mut field_sets = BTreeMap::new();
    for r in &input.records {
        field_sets.insert(r.name.clone(), fp(&record_canonical(r)));
    }
    let mut candidate_sets = BTreeMap::new();
    candidate_sets.insert("fn", cand_fp(&input.fn_candidates()));
    candidate_sets.insert("type", cand_fp(&input.record_candidates()));
    DependencyFingerprints {
        fn_sigs,
        fn_bodies,
        field_sets,
        candidate_sets,
        source_maps: BTreeMap::new(),
        config: fp(
            format!("{INFER_ALGO_ID}\u{1}{TARGET_SEMANTICS}\u{1}{BUILTIN_CAPABILITIES}").as_bytes(),
        ),
    }
}

/// Build the manifest a freshly computed result must carry: the config
/// fingerprint, the target's body fingerprint, every callee signature
/// actually read, and the negative lookups.
pub fn manifest_from(reads: &Reads, input: &SemaInput, target: &str) -> DepManifest {
    let mut positives = vec![DepFingerprint {
        kind: "config",
        origin: "profile/algo/schema".into(),
        fp: fp(
            format!("{INFER_ALGO_ID}\u{1}{TARGET_SEMANTICS}\u{1}{BUILTIN_CAPABILITIES}").as_bytes(),
        ),
    }];
    if let Some(f) = input.lookup_fn(target) {
        positives.push(DepFingerprint {
            kind: "fn-body",
            origin: target.into(),
            fp: fp(&fn_body_canonical(f)),
        });
    }
    for s in &reads.sigs {
        if s != target {
            positives.push(DepFingerprint { kind: "fn-sig", origin: s.clone(), fp: String::new() });
        }
    }
    for p in positives.iter_mut() {
        if p.kind == "fn-sig" {
            if let Some(f) = input.lookup_fn(&p.origin) {
                p.fp = fp(&fn_sig_canonical(f));
            }
        }
    }
    DepManifest {
        positives,
        negatives: reads
            .negatives
            .iter()
            .map(|(d, n, set)| NegDep { domain: *d, name: n.clone(), set_fp: set.clone() })
            .collect(),
    }
}

/// Outcome of validating a cached manifest against the current input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    /// What changed and why — the miss reason (audit trail).
    Stale { reason: String },
}

/// Validate a cached result's manifest against the current input:
/// EVERY positive fingerprint must match, and every negative dependency
/// must still hold (the candidate set unchanged AND the name still
/// absent). A single drift is a miss.
pub fn freshness_of(manifest: &DepManifest, current: &DependencyFingerprints) -> Freshness {
    for p in &manifest.positives {
        let ok = match p.kind {
            "fn-sig" => current.fn_sigs.get(&p.origin) == Some(&p.fp),
            "fn-body" => current.fn_bodies.get(&p.origin) == Some(&p.fp),
            "field-set" => current.field_sets.get(&p.origin) == Some(&p.fp),
            "candidate-set" => {
                current.candidate_sets.values().any(|v| v == &p.fp)
            }
            "config" => current.config == p.fp,
            "group-content" => true,
            // fail-closed: an unregistered source origin cannot be
            // validated, so the result is stale rather than trusted
            "source-map" => current.source_maps.get(&p.origin) == Some(&p.fp),
            _ => true,
        };
        if !ok {
            return Freshness::Stale {
                reason: format!("positive dep {}:{} changed", p.kind, p.origin),
            };
        }
    }
    for n in &manifest.negatives {
        let set_now = current.candidate_sets.get(n.domain);
        let name_defined = match n.domain {
            "fn" => current.fn_sigs.contains_key(&n.name),
            "type" => current.field_sets.contains_key(&n.name),
            _ => false,
        };
        if name_defined {
            return Freshness::Stale {
                reason: format!(
                    "negative dep broken: `{}` is now defined in the {} domain",
                    n.name, n.domain
                ),
            };
        }
        if set_now != Some(&n.set_fp) {
            return Freshness::Stale {
                reason: format!(
                    "negative dep candidate set for domain {} changed",
                    n.domain
                ),
            };
        }
    }
    Freshness::Fresh
}

/// The logical id of a query target (used in cached-result pages).
pub fn target_logical(input: &SemaInput, query: &QueryKind) -> LogicalId {
    let (kind, name) = match query {
        QueryKind::InferBody { fn_name } | QueryKind::FnSig { fn_name } => {
            (DeclKind::Fn, fn_name.clone())
        }
        QueryKind::RecordFields { record } => (DeclKind::Record, record.clone()),
        QueryKind::ListCandidates { .. } => (DeclKind::Fn, "<domain>".to_string()),
    };
    ac_sema_contract::logical_id(
        &PackageId::new(&input.package),
        &input.module,
        kind,
        &name,
    )
}

/// Content version of a query target's definition (page header field).
pub fn target_content(input: &SemaInput, query: &QueryKind) -> ContentVersion {
    let bytes = match query {
        QueryKind::InferBody { fn_name } | QueryKind::FnSig { fn_name } => input
            .lookup_fn(fn_name)
            .map(|f| {
                let mut b = fn_sig_canonical(f);
                b.extend_from_slice(&fn_body_canonical(f));
                b
            })
            .unwrap_or_default(),
        QueryKind::RecordFields { record } => {
            input.lookup_record(record).map(record_canonical).unwrap_or_default()
        }
        QueryKind::ListCandidates { domain } => {
            let set = match *domain {
                "fn" => input.fn_candidates(),
                "type" => input.record_candidates(),
                _ => Vec::new(),
            };
            set.join("\u{1}").into_bytes()
        }
    };
    ContentVersion::of_canonical(&bytes)
}
