//! Query scheduling (contract-decision §3/§7). T-03 shape: admission of
//! the query set (reference completeness, node/depth ceilings, meta
//! capability gates, cycle chains that name the CLOSING edge),
//! resolution of literal and Meta nodes through the landed 796
//! reflection surface, and an ordered AV call plan whose arguments stay
//! RAW — the driver concretizes them progressively as earlier calls
//! execute (a `QueryRef` to an earlier call is a legal DAG dependency,
//! never a cycle). Single-flight dedups identical (callee, args) calls.

use ac_sema_core::{reflect, SemaInput};
use auto_cte_contract::{
    CteAdmissionError, CteCode, CteRequestV1, CteValue, QueryNodeKind, CTE_META_CAP,
    MAX_QUERY_NODES_PER_REQUEST, MAX_QUERY_REF_DEPTH,
};
use std::collections::BTreeMap;

/// One deduplicated real AV call to execute (arguments RAW until the
/// driver concretizes them at execution time).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AvCall {
    /// Query ids that requested this call (single-flight group).
    pub query_ids: Vec<String>,
    pub callee: String,
    /// RAW argument nodes (literals, refs to literals/meta, or refs to
    /// EARLIER calls — legal by execution order).
    pub args: Vec<QueryNodeKind>,
}

/// The resolution plan of one request's query set.
#[derive(Clone, Debug, Default)]
pub struct QueryResolution {
    /// Literals by query id.
    pub literals: BTreeMap<String, CteValue>,
    /// Meta query results by id (each also counted by the driver).
    pub meta: BTreeMap<String, CteValue>,
    /// Records actually read by meta nodes (manifest input).
    pub meta_records: Vec<String>,
    /// HasField probes (record, field) — the negative-dependency set.
    pub meta_probes: Vec<(String, String)>,
    /// Canonical meta facts ("Pair#2", "Pair.zzz=false") for the frozen
    /// manifest — freshness re-derives and compares these.
    pub meta_facts: Vec<String>,
    /// Deduplicated AV calls in declaration order.
    pub av_calls: Vec<AvCall>,
    /// Values known after planning (literals + meta).
    pub values: BTreeMap<String, CteValue>,
}

/// Walk the reference edges backwards from `from` through the declared
/// nodes, naming the full cycle chain including the closing edge.
fn cycle_chain(req: &CteRequestV1, from: &str, first_target: &str) -> Vec<String> {
    let mut chain = vec![from.to_string()];
    let mut cursor = first_target.to_string();
    loop {
        if chain.contains(&cursor) {
            chain.push(cursor);
            return chain;
        }
        chain.push(cursor.clone());
        let next = req
            .queries
            .iter()
            .find(|q| q.id == cursor)
            .and_then(|q| match &q.kind {
                QueryNodeKind::QueryRef(t) => Some(t.clone()),
                QueryNodeKind::Call { args, .. } => args.iter().find_map(|a| match a {
                    QueryNodeKind::QueryRef(t) => Some(t.clone()),
                    _ => None,
                }),
                _ => None,
            });
        match next {
            Some(t) => cursor = t,
            None => return chain,
        }
    }
}

/// Plan the query set: validate references and ceilings, resolve
/// literals/meta, and emit the ordered AV call plan. Declaration order
/// is the topological order: a `QueryRef` may only name an
/// ALREADY-declared node; self/later references are cycles.
pub fn resolve_queries(
    req: &CteRequestV1,
    input: &SemaInput,
) -> Result<QueryResolution, CteAdmissionError> {
    if req.queries.len() > MAX_QUERY_NODES_PER_REQUEST {
        return Err(CteAdmissionError::new(
            CteCode::TooManyQueries,
            format!(
                "{} query nodes exceed the per-request ceiling {MAX_QUERY_NODES_PER_REQUEST}",
                req.queries.len()
            ),
        ));
    }
    let mut res = QueryResolution::default();
    // QueryRef-chain depth per node (the frozen <=128 ceiling, checked at
    // resolution — independent of the node-count cap).
    let mut depth: BTreeMap<String, u32> = BTreeMap::new();
    let meta_granted = req.caps.iter().any(|c| c == CTE_META_CAP);
    for (idx, q) in req.queries.iter().enumerate() {
        match &q.kind {
            QueryNodeKind::Literal(v) => {
                depth.insert(q.id.clone(), 0);
                if res.values.insert(q.id.clone(), *v).is_some() {
                    return Err(CteAdmissionError::new(
                        CteCode::UnknownQueryRef,
                        format!("duplicate query id `{}`", q.id),
                    ));
                }
                res.literals.insert(q.id.clone(), *v);
            }
            QueryNodeKind::QueryRef(target) => {
                // Only EARLIER declarations are addressable.
                if !req.queries[..idx].iter().any(|p| p.id == *target) {
                    return Err(CteAdmissionError::new(
                        CteCode::QueryCycle,
                        format!(
                            "query `{}` references `{}` which is not an earlier completed node",
                            q.id, target
                        ),
                    )
                    .with_chain(cycle_chain(req, &q.id, target)));
                }
                let d = depth
                    .get(target)
                    .copied()
                    .unwrap_or(u32::MAX)
                    .saturating_add(1);
                if d > MAX_QUERY_REF_DEPTH as u32 {
                    return Err(CteAdmissionError::new(
                        CteCode::QueryDepth,
                        format!(
                            "query `{}` sits on a reference chain of depth {d} (ceiling {MAX_QUERY_REF_DEPTH})",
                            q.id
                        ),
                    ));
                }
                depth.insert(q.id.clone(), d);
                let v = res.values.get(target).cloned().ok_or_else(|| {
                    // Earlier-declared but not yet valued: its value
                    // depends on an AV call this node feeds — a cycle.
                    CteAdmissionError::new(
                        CteCode::QueryCycle,
                        format!(
                            "query `{}` references `{}` whose value is not yet available (call-dependency cycle)",
                            q.id, target
                        ),
                    )
                    .with_chain(cycle_chain(req, &q.id, target))
                })?;
                res.values.insert(q.id.clone(), v);
            }
            QueryNodeKind::Call { callee, args } => {
                if req.profile == auto_cte_contract::CteSourceProfile::Record {
                    // Pure-meta restricted mode. The CONTRACTED Meta→AV
                    // argument path runs on CORE decks carrying a `meta`
                    // block (F-817-R1-05): record declarations live in
                    // req.meta_source, the executable source stays core.
                    return Err(CteAdmissionError::new(
                        CteCode::CapabilityDenied,
                        "record-profile decks are Meta-only; Meta-as-argument calls run on core decks with a `meta << ... >>` block",
                    ));
                }
                let d = args
                    .iter()
                    .map(|a| match a {
                        QueryNodeKind::QueryRef(t) => depth.get(t).copied().unwrap_or(u32::MAX),
                        _ => 0,
                    })
                    .max()
                    .unwrap_or(0);
                depth.insert(q.id.clone(), d);
                // RAW args: refs to earlier CALLS stay symbolic (the
                // driver concretizes them in execution order); refs to
                // anything later- or self-declared are cycles.
                for a in args {
                    if let QueryNodeKind::QueryRef(t) = a {
                        let pos = req.queries[..idx].iter().position(|p| p.id == *t);
                        match pos {
                            None => {
                                return Err(CteAdmissionError::new(
                                    CteCode::QueryCycle,
                                    format!(
                                        "argument of `{}` references `{}` which is not an earlier node",
                                        q.id, t
                                    ),
                                )
                                .with_chain(cycle_chain(req, &q.id, t)));
                            }
                            Some(p) => {
                                let known = res.values.contains_key(t)
                                    || matches!(req.queries[p].kind, QueryNodeKind::Call { .. });
                                if !known {
                                    return Err(CteAdmissionError::new(
                                        CteCode::QueryCycle,
                                        format!(
                                            "argument of `{}` references `{}` whose value is not yet available",
                                            q.id, t
                                        ),
                                    )
                                    .with_chain(cycle_chain(req, &q.id, t)));
                                }
                            }
                        }
                    }
                }
                // Single-flight: identical (callee, raw args) groups.
                if let Some(existing) = res
                    .av_calls
                    .iter_mut()
                    .find(|c| c.callee == *callee && c.args == *args)
                {
                    existing.query_ids.push(q.id.clone());
                } else {
                    res.av_calls.push(AvCall {
                        query_ids: vec![q.id.clone()],
                        callee: callee.clone(),
                        args: args.clone(),
                    });
                }
            }
            QueryNodeKind::FieldCount { record } => {
                if !meta_granted {
                    return Err(CteAdmissionError::new(
                        CteCode::MetaNotGranted,
                        format!("FieldCount(`{record}`) requires capability `{CTE_META_CAP}`"),
                    ));
                }
                let n = reflect::list_fields(input, record)
                    .ok_or_else(|| {
                        CteAdmissionError::new(
                            CteCode::MetaRecordNotFound,
                            format!("record `{record}` does not exist in the current input"),
                        )
                    })?
                    .len() as i32;
                if !res.meta_records.contains(record) {
                    res.meta_records.push(record.clone());
                }
                let fact = format!("{record}#{n}");
                if !res.meta_facts.contains(&fact) {
                    res.meta_facts.push(fact);
                }
                depth.insert(q.id.clone(), 0);
                res.meta.insert(q.id.clone(), CteValue::I32(n));
                res.values.insert(q.id.clone(), CteValue::I32(n));
            }
            QueryNodeKind::HasField { record, field } => {
                if !meta_granted {
                    return Err(CteAdmissionError::new(
                        CteCode::MetaNotGranted,
                        format!(
                            "HasField(`{record}`,`{field}`) requires capability `{CTE_META_CAP}`"
                        ),
                    ));
                }
                let has = reflect::list_fields(input, record)
                    .ok_or_else(|| {
                        CteAdmissionError::new(
                            CteCode::MetaRecordNotFound,
                            format!("record `{record}` does not exist in the current input"),
                        )
                    })?
                    .iter()
                    .any(|f| f == field);
                if !res.meta_records.contains(record) {
                    res.meta_records.push(record.clone());
                }
                let probe = (record.clone(), field.clone());
                if !res.meta_probes.contains(&probe) {
                    res.meta_probes.push(probe);
                }
                let fact = format!("{record}.{field}={has}");
                if !res.meta_facts.contains(&fact) {
                    res.meta_facts.push(fact);
                }
                depth.insert(q.id.clone(), 0);
                res.meta.insert(q.id.clone(), CteValue::Bool(has));
                res.values.insert(q.id.clone(), CteValue::Bool(has));
            }
        }
    }
    Ok(res)
}

/// Resolve one RAW argument against the values known so far (the driver
/// calls this per call right before execution — every earlier call's
/// value is present by ordering).
pub fn concretize(
    a: &QueryNodeKind,
    values: &BTreeMap<String, CteValue>,
    from: &str,
) -> Result<CteValue, CteAdmissionError> {
    match a {
        QueryNodeKind::Literal(v) => Ok(*v),
        QueryNodeKind::QueryRef(target) => values.get(target).cloned().ok_or_else(|| {
            CteAdmissionError::new(
                CteCode::QueryCycle,
                format!(
                    "argument of `{from}` references `{target}` whose value is not yet available"
                ),
            )
            .with_chain(vec![from.to_string(), target.clone()])
        }),
        other => Err(CteAdmissionError::new(
            CteCode::UnknownQueryRef,
            format!("nested non-literal/non-ref argument is not addressable: {other:?}"),
        )),
    }
}
