//! Dependency fingerprints for the frozen manifest (contract-decision
//! §6). The first profile is conservative: every body of the callee's
//! transitive call closure participates, plus the negative-dependency
//! candidate sets — and the manifest declares that granularity.

use ac_sema_contract::identity::domain_hash;
use ac_sema_core::{query, SemaInput};
use auto_cte_contract::{CteDepsGranularity, CteDepsManifestV1, CTE_QUERY_ALGO_ID};

/// Build the manifest for one callee over the current input.
pub fn build_manifest(
    input: &SemaInput,
    callee: &str,
    caps: &[String],
    meta_records: &[String],
    meta_probes: &[(String, String)],
) -> Result<CteDepsManifestV1, String> {
    let Some(start) = input.fns.iter().find(|f| f.name == callee) else {
        return Err(format!("callee `{callee}` is not in the projected input"));
    };
    // Transitive closure over direct calls (796 `called_names` per fn).
    // Intrinsic/undeclared names (e.g. trace marks) are NOT closure
    // members — the capability audit owns their refusal.
    let mut closure: Vec<String> = vec![callee.to_string()];
    let mut i = 0usize;
    while i < closure.len() {
        let name = closure[i].clone();
        let Some(f) = input.fns.iter().find(|f| f.name.as_str() == name.as_str()) else {
            return Err(format!("closure member `{name}` disappeared mid-walk"));
        };
        for called in query::called_names(f) {
            if input.fns.iter().any(|g| g.name == called) && !closure.contains(&called) {
                closure.push(called);
            }
        }
        i += 1;
        if closure.len() > 4096 {
            return Err("call closure exceeds the manifest bound (4096 fns)".into());
        }
    }
    let start_name = start.name.clone();
    let mut fn_fingerprints = Vec::new();
    for name in &closure {
        let Some(f) = input.fns.iter().find(|f| f.name.as_str() == name.as_str()) else {
            return Err(format!("closure member `{name}` not found"));
        };
        let sig = hex_fp(&query::fn_sig_canonical(f));
        let body = hex_fp(&query::fn_body_canonical(f));
        fn_fingerprints.push((name.clone(), sig, body));
    }
    closure.sort();
    let fn_candidates_fp = hex_fp(
        &input
            .fn_candidates()
            .iter()
            .flat_map(|s| s.as_bytes().to_vec())
            .collect::<Vec<u8>>(),
    );
    let record_candidates_fp = hex_fp(
        &input
            .record_candidates()
            .iter()
            .flat_map(|s| s.as_bytes().to_vec())
            .collect::<Vec<u8>>(),
    );
    let meta_facts = derive_meta_facts(input, meta_records, meta_probes)?;
    let mut meta_records = meta_records.to_vec();
    meta_records.sort();
    let mut meta_probes = meta_probes.to_vec();
    meta_probes.sort();
    Ok(CteDepsManifestV1 {
        granularity: CteDepsGranularity::ClosureBodies,
        callee: start_name,
        closure,
        fn_fingerprints,
        meta_records,
        meta_probes,
        meta_facts,
        fn_candidates_fp,
        record_candidates_fp,
        caps: caps.to_vec(),
    })
}

/// Re-derive the canonical meta facts for the probed sets against the
/// CURRENT input ("Pair#2", "Pair.zzz=false").
pub fn derive_meta_facts(
    input: &SemaInput,
    meta_records: &[String],
    meta_probes: &[(String, String)],
) -> Result<Vec<String>, String> {
    let mut facts = Vec::new();
    for record in meta_records {
        let fields = ac_sema_core::reflect::list_fields(input, record)
            .ok_or_else(|| format!("meta record `{record}` no longer exists"))?;
        facts.push(format!("{record}#{}", fields.len()));
        for (r, f) in meta_probes {
            if r == record {
                let has = fields.iter().any(|x| x == f);
                facts.push(format!("{r}.{f}={has}"));
            }
        }
    }
    facts.sort();
    Ok(facts)
}

/// Stable short fingerprint of canonical bytes.
pub fn hex_fp(bytes: &[u8]) -> String {
    domain_hash(CTE_QUERY_ALGO_ID, &[bytes])
        .chars()
        .take(32)
        .collect()
}

/// Manifest fingerprint used inside the request identity.
pub fn manifest_fp(m: &CteDepsManifestV1) -> String {
    let canonical = serde_json_fmt(m);
    hex_fp(canonical.as_bytes())
}

fn serde_json_fmt(m: &CteDepsManifestV1) -> String {
    // CteDepsManifestV1 is Serialize; a stable textual form feeds the fp.
    serde_json::to_string(m).unwrap_or_default()
}

/// Freshness: does the recorded manifest still describe `input`?
/// A helper body/signature change, a candidate-set change (negative
/// deps) or a caps change all invalidate.
pub fn manifest_fresh(m: &CteDepsManifestV1, input: &SemaInput, caps: &[String]) -> bool {
    match build_manifest(input, &m.callee, caps, &m.meta_records, &m.meta_probes) {
        Ok(current) => current == *m,
        Err(_) => false,
    }
}
