//! Query scheduling (contract-decision §3/§7). T-02 scope: admission of
//! the query set (reference completeness, node/depth ceilings, meta
//! capability gates), resolution of literal and Meta nodes through the
//! landed 796 reflection surface, and an ordered AV call plan with
//! single-flight dedup. The worklist/cancel/late-snapshot machinery is
//! T-03; the declarations here are the seams it extends.

use ac_sema_core::{reflect, SemaInput};
use auto_cte_contract::{
    CteAdmissionError, CteCode, CteRequestV1, CteValue, QueryNodeKind, CTE_META_CAP,
    MAX_QUERY_NODES_PER_REQUEST,
};
use std::collections::BTreeMap;

/// One deduplicated real AV call to execute.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AvCall {
    /// Query ids that requested this call (single-flight group).
    pub query_ids: Vec<String>,
    pub callee: String,
    /// Concrete literal arguments (refs already resolved by ordering).
    pub args: Vec<CteValue>,
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
    /// Deduplicated AV calls in declaration order.
    pub av_calls: Vec<AvCall>,
    /// Values known after planning (literals + meta).
    pub values: BTreeMap<String, CteValue>,
}

/// Plan the query set: validate references and ceilings, resolve
/// literals/meta, and emit the ordered AV call plan. Declaration order is
/// the topological order: a `QueryRef` may only name an ALREADY-declared
/// node; self/later references surface as cycle/unknown-ref admissions.
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
    let meta_granted = req.caps.iter().any(|c| c == CTE_META_CAP);
    for (idx, q) in req.queries.iter().enumerate() {
        match &q.kind {
            QueryNodeKind::Literal(v) => {
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
                    let chain = vec![q.id.clone(), target.clone(), q.id.clone()];
                    return Err(CteAdmissionError::new(
                        CteCode::QueryCycle,
                        format!(
                            "query `{}` references `{}` which is not an earlier completed node",
                            q.id, target
                        ),
                    )
                    .with_chain(chain));
                }
                let v = res
                    .values
                    .get(target)
                    .cloned()
                    .ok_or_else(|| {
                        // Earlier-declared but not yet valued: its value
                        // depends on an AV call this node feeds — a cycle.
                        CteAdmissionError::new(
                            CteCode::QueryCycle,
                            format!(
                                "query `{}` references `{}` whose value is not yet available (call-dependency cycle)",
                                q.id, target
                            ),
                        )
                        .with_chain(vec![q.id.clone(), target.clone()])
                    })?;
                res.values.insert(q.id.clone(), v);
            }
            QueryNodeKind::Call { callee, args } => {
                let mut concrete = Vec::with_capacity(args.len());
                for a in args {
                    concrete.push(concretize(a, &res.values, &q.id)?);
                }
                // Single-flight: identical (callee, args) groups together.
                if let Some(existing) = res
                    .av_calls
                    .iter_mut()
                    .find(|c| c.callee == *callee && c.args == concrete)
                {
                    existing.query_ids.push(q.id.clone());
                } else {
                    res.av_calls.push(AvCall {
                        query_ids: vec![q.id.clone()],
                        callee: callee.clone(),
                        args: concrete,
                    });
                }
                // The VALUE is filled by the driver after execution; a
                // later QueryRef against an unexecuted call is the cycle
                // guard above.
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
                res.meta.insert(q.id.clone(), CteValue::Bool(has));
                res.values.insert(q.id.clone(), CteValue::Bool(has));
            }
        }
    }
    Ok(res)
}

/// Resolve one argument kind against the values known so far.
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
