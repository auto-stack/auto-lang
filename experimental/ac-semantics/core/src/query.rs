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

use crate::infer::Reads;
use crate::input::{Ann, SemaInput};
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
/// the body fingerprint). Expression encoding is ITERATIVE (explicit
/// stack): legal operator chains are unbounded in depth, so no native
/// recursion on the canonical walk.
pub fn fn_body_canonical(f: &crate::input::InputFn) -> Vec<u8> {
    fn enc_expr(out: &mut Vec<u8>, root: &crate::input::InputExpr) {
        use crate::input::InputExprKind;
        let mut stack: Vec<&crate::input::InputExpr> = vec![root];
        while let Some(e) = stack.pop() {
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
                    for a in args.iter().rev() {
                        stack.push(a);
                    }
                }
                InputExprKind::Binary { op, lhs, rhs } => {
                    out.push(b'(');
                    out.extend_from_slice(op.as_str().as_bytes());
                    stack.push(rhs);
                    stack.push(lhs);
                }
                InputExprKind::Unary { neg, operand } => {
                    out.push(b'U');
                    out.push(*neg as u8);
                    stack.push(operand);
                }
                InputExprKind::Unsupported { what } => {
                    out.push(b'X');
                    out.extend_from_slice(what.as_bytes());
                    out.push(0);
                }
            }
        }
    }
    fn enc_stmt(out: &mut Vec<u8>, s: &crate::input::InputStmt) {
        use crate::input::InputStmt;
        match s {
            InputStmt::Let {
                name,
                ann,
                mutable,
                value,
                ..
            } => {
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
            InputStmt::Assign {
                target,
                path,
                value,
                ..
            } => {
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
            InputStmt::If {
                cond, then, els, ..
            } => {
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

/// Direct referenced record names of one record, sorted and deduped.
/// These are the declarations whose CONTENT VERSION is embedded in the
/// record's published shape (F-796-R6-01: A.next: B carries B's content
/// version; a B change must change A's key, never serve the old ref).
pub fn record_dep_names(input: &SemaInput, name: &str) -> Vec<String> {
    let Some(r) = input.lookup_record(name) else {
        return Vec::new();
    };
    let mut out: Vec<String> = Vec::new();
    for (_, a) in &r.fields {
        if let Ann::Record(rec) = a {
            if input.lookup_record(rec).is_some() && !out.iter().any(|d| d == rec) {
                out.push((*rec).to_string());
            }
        }
    }
    out.sort();
    out
}

/// Effective dependency environment for record keys/versions
/// (F-796-R7-01): the unit partition (singleton / recursive SCC) plus a
/// TRANSITIVE effective version per unit. A unit's effective version is
/// its members' declaration canonicals plus the effective versions of
/// its EXTERNAL dependencies — so a leaf edit propagates through every
/// reader, and a topology change (singleton ↔ group) produces a
/// different key and carrier.
pub struct KeyEnv {
    units: Vec<Vec<String>>,
    unit_of: std::collections::BTreeMap<String, usize>,
    unit_eff: Vec<String>,
}

impl KeyEnv {
    pub fn new(input: &SemaInput) -> Self {
        let units = record_units(input);
        let mut unit_of: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();
        for (i, u) in units.iter().enumerate() {
            for n in u {
                unit_of.insert(n.clone(), i);
            }
        }
        let mut unit_eff: Vec<String> = vec![String::new(); units.len()];
        for (i, unit) in units.iter().enumerate() {
            let mut members: Vec<&str> = unit.iter().map(|s| s.as_str()).collect();
            members.sort();
            let mut parts: Vec<String> = Vec::new();
            let mut external: Vec<(String, String)> = Vec::new();
            for name in &members {
                let canonical = input
                    .lookup_record(name)
                    .map(record_canonical)
                    .unwrap_or_default();
                let content = ContentVersion::of_canonical(&canonical);
                parts.push(format!("{name}@{}", content.as_str()));
                for dep in record_dep_names(input, name) {
                    let dunit = match unit_of.get(&dep) {
                        Some(u) => *u,
                        None => continue,
                    };
                    if dunit != i {
                        // dependency order: the external unit is already
                        // fully versioned
                        let eff = &unit_eff[dunit];
                        external.push((dep, eff.clone()));
                    }
                }
            }
            external.sort();
            external.dedup();
            for (dep, eff) in external {
                parts.push(format!("dep:{dep}={eff}"));
            }
            unit_eff[i] = fp(parts.join("\u{1}").as_bytes());
        }
        KeyEnv {
            units,
            unit_of,
            unit_eff,
        }
    }

    pub fn units(&self) -> &[Vec<String>] {
        &self.units
    }

    fn unit_index(&self, name: &str) -> Option<usize> {
        self.unit_of.get(name).copied()
    }

    pub fn members(&self, name: &str) -> Option<&[String]> {
        self.unit_index(name).map(|i| self.units[i].as_slice())
    }

    /// The unit's transitive effective version (page-content identity).
    pub fn effective(&self, name: &str) -> Option<&str> {
        self.unit_index(name).map(|i| self.unit_eff[i].as_str())
    }

    /// A record is served by a GROUP page iff its unit is a real SCC
    /// (more than one member) or it references itself.
    pub fn is_group(&self, input: &SemaInput, name: &str) -> bool {
        match self.unit_index(name) {
            Some(i) => {
                self.units[i].len() > 1 || record_dep_names(input, name).iter().any(|d| d == name)
            }
            None => false,
        }
    }

    /// Topology identity of the record's carrier (singleton vs group and
    /// the exact member set): part of every RecordFields key, so a
    /// singleton page can never be served for the grouped form (or the
    /// reverse).
    pub fn topology_fp(&self, name: &str) -> String {
        match self.unit_index(name) {
            None => fp(b"absent"),
            Some(i) => {
                let mut members = self.units[i].clone();
                members.sort();
                if members.len() > 1 {
                    fp(format!("group:{}", members.join("+")).as_bytes())
                } else {
                    fp(format!("single:{}", members[0]).as_bytes())
                }
            }
        }
    }

    /// The group/unit effective version (the published page content).
    pub fn group_eff(&self, members: &[String]) -> String {
        members
            .iter()
            .find_map(|m| self.effective(m))
            .unwrap_or_default()
            .to_string()
    }
}

/// Dependency fingerprint with the effective unit versions (key
/// component for RecordFields).
pub fn record_refs_fp(input: &SemaInput, env: &KeyEnv, name: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    for dep in record_dep_names(input, name) {
        if let Some(eff) = env.effective(&dep) {
            parts.push(format!("{dep}={eff}"));
        }
    }
    parts.sort();
    fp(parts.join("\u{1}").as_bytes())
}

/// SCC partition (iterative Kosaraju) of the record reference graph, in
/// DEPENDENCY ORDER (referenced units first). A unit is a singleton
/// without self-reference or a recursive group.
pub fn record_units(input: &SemaInput) -> Vec<Vec<String>> {
    use std::collections::BTreeSet;
    let names: Vec<String> = input.records.iter().map(|r| r.name.clone()).collect();
    let name_set: BTreeSet<&str> = names.iter().map(|s| s.as_str()).collect();
    let adj = |n: &str| -> Vec<String> {
        record_dep_names(input, n)
            .into_iter()
            .filter(|d| name_set.contains(d.as_str()))
            .collect()
    };
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut order: Vec<String> = Vec::new();
    for start in &names {
        if visited.contains(start) {
            continue;
        }
        let mut stack: Vec<(String, usize)> = vec![(start.clone(), 0)];
        visited.insert(start.clone());
        while let Some((node, idx)) = stack.pop() {
            let succ = adj(&node);
            if idx < succ.len() {
                stack.push((node.clone(), idx + 1));
                let next = succ[idx].clone();
                if visited.insert(next.clone()) {
                    stack.push((next, 0));
                }
            } else {
                order.push(node.clone());
            }
        }
    }
    let mut reverse: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for n in &names {
        reverse.insert(n.clone(), Vec::new());
    }
    for n in &names {
        for d in adj(n) {
            reverse.entry(d).or_default().push(n.clone());
        }
    }
    let radj = |n: &str| -> &[String] { reverse.get(n).map(|v| v.as_slice()).unwrap_or(&[]) };
    let mut assigned: BTreeSet<String> = BTreeSet::new();
    let mut units: Vec<Vec<String>> = Vec::new();
    for start in order.iter().rev() {
        if assigned.contains(start) {
            continue;
        }
        let mut comp: Vec<String> = Vec::new();
        let mut stack = vec![start.clone()];
        assigned.insert(start.clone());
        while let Some(node) = stack.pop() {
            comp.push(node.clone());
            for next in radj(&node) {
                if assigned.insert(next.clone()) {
                    stack.push(next.clone());
                }
            }
        }
        units.push(comp);
    }
    units.reverse();
    units
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

/// Syntactic callee set of one function body (names in Call nodes,
/// collected ITERATIVELY — bodies are unbounded in depth).
pub fn called_names(f: &crate::input::InputFn) -> std::collections::BTreeSet<String> {
    use crate::input::{InputExpr, InputExprKind, InputStmt};
    let mut names = std::collections::BTreeSet::new();
    let mut exprs: Vec<&InputExpr> = Vec::new();
    let mut blocks: Vec<&crate::input::InputBlock> = vec![&f.body];
    while let Some(b) = blocks.pop() {
        for s in &b.stmts {
            match s {
                InputStmt::Let { value, .. }
                | InputStmt::Assign { value, .. }
                | InputStmt::Return { value, .. } => exprs.push(value),
                InputStmt::If {
                    cond, then, els, ..
                } => {
                    exprs.push(cond);
                    blocks.push(then);
                    if let Some(e) = els {
                        blocks.push(e);
                    }
                }
                InputStmt::While { cond, body, .. } => {
                    exprs.push(cond);
                    blocks.push(body);
                }
                InputStmt::Loop { body, .. } => blocks.push(body),
                InputStmt::Break { .. } | InputStmt::Continue { .. } => {}
            }
        }
    }
    while let Some(e) = exprs.pop() {
        match &e.kind {
            InputExprKind::Call { callee, args } => {
                names.insert(callee.clone());
                for a in args {
                    exprs.push(a);
                }
            }
            InputExprKind::Binary { lhs, rhs, .. } => {
                exprs.push(lhs);
                exprs.push(rhs);
            }
            InputExprKind::Unary { operand, .. } => exprs.push(operand),
            _ => {}
        }
    }
    names
}

/// Fingerprint of the SIGNATURE SET the target syntactically calls:
/// part of the InferBody key so a read-callee signature change produces
/// a different key (structurally impossible to conflict one key with
/// two contents; the manifest still validates body/negative/source
/// dependencies at hit time).
pub fn called_sigs_fp(input: &SemaInput, fn_name: &str) -> String {
    match input.lookup_fn(fn_name) {
        None => fp(b"absent"),
        Some(f) => {
            let names = called_names(f);
            let mut parts: Vec<String> = Vec::new();
            for n in names {
                let sig = match input.lookup_fn(&n) {
                    Some(callee) => fp(&fn_sig_canonical(callee)),
                    None => "absent".to_string(),
                };
                parts.push(format!("{n}={sig}"));
            }
            let joined = parts.join("\u{1}");
            fp(joined.as_bytes())
        }
    }
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
        Self::build_with_env(query, input, None)
    }

    /// Key build with a PRECOMPUTED effective-dependency environment
    /// (the publication loop builds one per module instead of one per
    /// record — the 4096-chain cost control, F-796-R6 V-06).
    pub fn build_with_env(query: &QueryKind, input: &SemaInput, env: Option<&KeyEnv>) -> Self {
        let owned: Option<KeyEnv> =
            if env.is_none() && matches!(query, QueryKind::RecordFields { .. }) {
                Some(KeyEnv::new(input))
            } else {
                None
            };
        let env = env.or(owned.as_ref());
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
                    f.map(|f| fp(&fn_sig_canonical(f)))
                        .unwrap_or_else(|| "absent".into()),
                ));
                c.push((
                    "fn-body-fp".into(),
                    f.map(|f| fp(&fn_body_canonical(f)))
                        .unwrap_or_else(|| "absent".into()),
                ));
                // the candidate sets the lookup domain ran against
                c.push((
                    "fn-cand-fp".into(),
                    fp(input.fn_candidates().join("\u{1}").as_bytes()),
                ));
                c.push((
                    "type-cand-fp".into(),
                    fp(input.record_candidates().join("\u{1}").as_bytes()),
                ));
                // the callee SIGNATURE SET the target reads (D7: a read
                // signature change is a key change — one key can never
                // hold two contents)
                c.push(("called-sigs-fp".into(), called_sigs_fp(input, fn_name)));
                // F-796-R4-05 (D7): the SOURCE REVISION of the
                // position-bearing projection is part of the key — a
                // position-drifted source gets a fresh key (fresh
                // compute, CURRENT spans); old positions are never
                // served for a new revision, and one key can never
                // hold two source revisions
                c.push((
                    "src-fp".into(),
                    if input.source_fp.is_empty() {
                        "none".to_string()
                    } else {
                        input.source_fp.clone()
                    },
                ));
            }
            QueryKind::FnSig { fn_name } => {
                let f = input.lookup_fn(fn_name);
                c.push((
                    "fn-sig-fp".into(),
                    f.map(|f| fp(&fn_sig_canonical(f)))
                        .unwrap_or_else(|| "absent".into()),
                ));
            }
            QueryKind::RecordFields { record } => {
                let r = input.lookup_record(record);
                c.push((
                    "field-set-fp".into(),
                    r.map(|r| fp(&record_canonical(r)))
                        .unwrap_or_else(|| "absent".into()),
                ));
                // F-796-R7-01: the key carries the DIRECT dependencies'
                // EFFECTIVE unit versions (transitive) and the carrier
                // topology — a leaf edit, an SCC formation/split or an
                // external group dependency all produce a different key,
                // never a hit onto an old required-data closure.
                let env = env.expect("record key environment");
                c.push(("ref-versions-fp".into(), record_refs_fp(input, env, record)));
                c.push(("unit-topology-fp".into(), env.topology_fp(record)));
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
    let mut source_maps = BTreeMap::new();
    if !input.source_fp.is_empty() {
        source_maps.insert("module-source".to_string(), input.source_fp.clone());
    }
    DependencyFingerprints {
        fn_sigs,
        fn_bodies,
        field_sets,
        candidate_sets,
        source_maps,
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
    // F-796-R4-05: the REAL producer records the module source
    // fingerprint — position-bearing facts are validated against it on
    // every hit (fail-closed), separating semantic-content reuse from
    // source-projection freshness
    if !input.source_fp.is_empty() {
        positives.push(DepFingerprint {
            kind: "source-map",
            origin: "module-source".into(),
            fp: input.source_fp.clone(),
        });
    }
    if let Some(f) = input.lookup_fn(target) {
        positives.push(DepFingerprint {
            kind: "fn-body",
            origin: target.into(),
            fp: fp(&fn_body_canonical(f)),
        });
    }
    for s in &reads.sigs {
        if s != target {
            positives.push(DepFingerprint {
                kind: "fn-sig",
                origin: s.clone(),
                fp: String::new(),
            });
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
            .map(|(d, n, set)| NegDep {
                domain: *d,
                name: n.clone(),
                set_fp: set.clone(),
            })
            .collect(),
    }
}

/// Outcome of validating a cached manifest against the current input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    /// Every dependency is fresh EXCEPT source provenance
    /// (F-796-R4-05): the semantic CONTENT is reusable, position
    /// projections belong to the old source and must be refreshed
    /// before serving.
    ProvenanceStale {
        reason: String,
    },
    /// A semantic dependency drifted — what and why (audit trail).
    Stale {
        reason: String,
    },
}

/// Validate a cached result's manifest against the current input:
/// EVERY positive fingerprint must match, and every negative dependency
/// must still hold (the candidate set unchanged AND the name still
/// absent). Drift is CLASSIFIED: only-source-map drift is
/// [`Freshness::ProvenanceStale`] (content reusable, positions stale);
/// any other drift is [`Freshness::Stale`].
pub fn freshness_class(manifest: &DepManifest, current: &DependencyFingerprints) -> Freshness {
    let mut drifted: Vec<&DepFingerprint> = Vec::new();
    for p in &manifest.positives {
        let ok = match p.kind {
            "fn-sig" => current.fn_sigs.get(&p.origin) == Some(&p.fp),
            "fn-body" => current.fn_bodies.get(&p.origin) == Some(&p.fp),
            "field-set" => current.field_sets.get(&p.origin) == Some(&p.fp),
            "candidate-set" => current.candidate_sets.values().any(|v| v == &p.fp),
            "config" => current.config == p.fp,
            "group-content" => true,
            // fail-closed: an unregistered source origin cannot be
            // validated, so the result is stale rather than trusted
            "source-map" => current.source_maps.get(&p.origin) == Some(&p.fp),
            _ => true,
        };
        if !ok {
            drifted.push(p);
        }
    }
    if !drifted.is_empty() {
        let all_source = drifted.iter().all(|p| p.kind == "source-map");
        let reasons: Vec<String> = drifted
            .iter()
            .map(|p| format!("positive dep {}:{} changed", p.kind, p.origin))
            .collect();
        let reason = reasons.join("; ");
        return if all_source {
            Freshness::ProvenanceStale { reason }
        } else {
            Freshness::Stale { reason }
        };
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
                reason: format!("negative dep candidate set for domain {} changed", n.domain),
            };
        }
    }
    Freshness::Fresh
}

/// Back-compat alias: the undifferentiated staleness view.
pub fn freshness_of(manifest: &DepManifest, current: &DependencyFingerprints) -> Freshness {
    match freshness_class(manifest, current) {
        Freshness::ProvenanceStale { reason } => Freshness::Stale { reason },
        other => other,
    }
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
    ac_sema_contract::logical_id(&PackageId::new(&input.package), &input.module, kind, &name)
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
        QueryKind::RecordFields { record } => input
            .lookup_record(record)
            .map(record_canonical)
            .unwrap_or_default(),
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
