# -*- coding: utf-8 -*-
# R5-F07: SCC group publication in store.rs
p = 'experimental/ac-semantics/probe/src/store.rs'
s = open(p, encoding='utf8').read()

# 1) lookup: RecordFields accepts RecordMeta OR RecGroup (member match)
old = '''    let expect_target = ac_sema_core::query::target_logical(input, query);
    match &page.body {
        PageBody::CachedResult(c) => admit_cached(c, query, &expect_target, input)?,
        PageBody::FnMeta(m) => {'''
new = '''    let expect_target = ac_sema_core::query::target_logical(input, query);
    // F-796-R5 (F07): a RecordFields query may be served by the
    // member's SINGLE page or by its recursive GROUP page — the group
    // admits iff the queried declaration is a member
    if page.header.kind == PageKind::RecGroup {
        if let QueryKind::RecordFields { record } = query {
            if let PageBody::RecGroup(members) = &page.body {
                let member_target = ac_sema_contract::logical_id(
                    &ac_sema_contract::PackageId::new(&input.package),
                    &input.module,
                    ac_sema_contract::DeclKind::Record,
                    record,
                );
                if members.iter().any(|m| m.decl.logical == member_target) {
                    return Ok(FetchOutcome::Hit { page, artifact_id: loaded.artifact_id });
                }
                return Err(ProbeError::TargetMismatch {
                    found: "a recursive group not containing the queried record".into(),
                    expected: member_target.as_str().to_string(),
                });
            }
        }
        return Err(ProbeError::KindMismatch {
            expected: "RecordMeta or RecGroup".into(),
            found: "RecGroup".into(),
        });
    }
    match &page.body {
        PageBody::CachedResult(c) => admit_cached(c, query, &expect_target, input)?,
        PageBody::FnMeta(m) => {'''
assert old in s, 'lookup anchor'
s = s.replace(old, new)

# expected_kind for RecordFields stays RecordMeta; the group path above
# short-circuits before the kind check? No — the kind check runs FIRST.
# Adjust: move the group branch before the kind check.
old = '''    let page = decode_page(&loaded.payload)?;
    let expected = expected_kind(query);
    if page.header.kind != expected {
        return Err(ProbeError::KindMismatch {
            expected: format!("{expected:?}"),
            found: format!("{:?}", page.header.kind),
        });
    }'''
new = '''    let page = decode_page(&loaded.payload)?;
    let expected = expected_kind(query);
    // a RecordFields query is servable by a group page (checked below
    // with member admission); every other kind must match exactly
    let group_ok_for_fields = page.header.kind == PageKind::RecGroup
        && matches!(query, QueryKind::RecordFields { .. });
    if page.header.kind != expected && !group_ok_for_fields {
        return Err(ProbeError::KindMismatch {
            expected: format!("{expected:?}"),
            found: format!("{:?}", page.header.kind),
        });
    }'''
assert old in s, 'kind check anchor'
s = s.replace(old, new)

# 2) shape builder shared by single/group publication + SCC partition
old = '''/// Publish (or reuse) one record's page whose referenced pages ALL
/// exist already. Non-recursive: the caller resolves the DAG order.'''
new = '''/// Build one record's shape with REAL indirect refs; `unit` is the set
/// of records being published in the SAME unit (group members or the
/// singleton itself) — references inside the unit stay plain DeclRefs
/// (they are part of the same page); references outside must already
/// be published and become required-data edges.
#[allow(clippy::too_many_arguments)]
fn build_shape(
    store: &ManagedStore,
    input: &SemaInput,
    name: &str,
    unit: &std::collections::BTreeSet<String>,
    required_edges: &mut Vec<String>,
) -> Result<ac_sema_contract::RecordShape, ProbeError> {
    let Some(r) = input.lookup_record(name) else {
        return Err(ProbeError::UnknownTarget { what: "record", name: name.to_string() });
    };
    let pkg = ac_sema_contract::PackageId::new(&input.package);
    let mut fields: Vec<ac_sema_contract::FieldDesc> = Vec::new();
    for (n, a) in &r.fields {
        let ty = match a {
            ac_sema_core::Ann::I32 => TypeDesc::I32,
            ac_sema_core::Ann::Bool => TypeDesc::Bool,
            ac_sema_core::Ann::Record(rec) => {
                let (inner, artifact) = record_typedesc(store, input, rec)?;
                if let (TypeDesc::Record { .. }, Some(art)) = (&inner, artifact) {
                    if !unit.contains(rec.as_ref()) {
                        required_edges.push(art);
                    }
                }
                inner
            }
            ac_sema_core::Ann::Inferred => TypeDesc::Opaque { name: "?inferred".into() },
            ac_sema_core::Ann::Unsupported(w) => {
                TypeDesc::Opaque { name: format!("unsupported:{w}") }
            }
        };
        fields.push(ac_sema_contract::FieldDesc { name: n.clone(), ty });
    }
    Ok(ac_sema_contract::RecordShape {
        decl: ac_sema_contract::DeclRef::new(
            &pkg,
            &input.module,
            ac_sema_contract::DeclKind::Record,
            name,
            ac_sema_contract::ContentVersion::of_canonical(
                &ac_sema_core::query::record_canonical(r),
            ),
        ),
        fields,
    })
}

/// SCC partition (iterative Kosaraju) of the record reference graph.
/// Returns units in DEPENDENCY ORDER (referenced groups first); a unit
/// is either a singleton without a self-reference or a recursive group.
fn scc_units(input: &SemaInput) -> Vec<Vec<String>> {
    use std::collections::{BTreeMap, BTreeSet};
    let names: Vec<String> = input.records.iter().map(|r| r.name.clone()).collect();
    let name_set: BTreeSet<&str> = names.iter().map(|s| s.as_str()).collect();
    let adj = |n: &str| -> Vec<String> {
        record_deps(input, n)
            .into_iter()
            .filter(|d| name_set.contains(d.as_str()))
            .collect()
    };
    // 1st pass: order by finish time (iterative DFS)
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
    // 2nd pass on the REVERSE graph in reverse finish order
    let radj = |n: &str| -> Vec<String> {
        let mut out = Vec::new();
        for other in &names {
            if adj(other).iter().any(|d| d == n) {
                out.push(other.clone());
            }
        }
        out
    };
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
                    stack.push(next);
                }
            }
        }
        units.push(comp);
    }
    // Kosaraju emits SCCs in reverse topological order of the condensation:
    // a unit may only depend on EARLIER units — dependencies first.
    units.reverse();
    // verify dependency order; a violation would mean a bug in the sort
    let pos: BTreeMap<String, usize> = units
        .iter()
        .enumerate()
        .flat_map(|(i, u)| u.iter().map(move |n| (n.clone(), i)))
        .collect();
    for (i, u) in units.iter().enumerate() {
        for n in u {
            for d in adj(n) {
                if let Some(&dp) = pos.get(&d) {
                    debug_assert!(dp <= i || u.iter().any(|m| m == &d), "unit order violated");
                }
            }
        }
    }
    units
}

/// Publish (or reuse) one record's page whose referenced pages ALL
/// exist already. Non-recursive: the caller resolves the DAG order.'''
assert old in s, 'shape builder anchor'
s = s.replace(old, new)

# 3) publish_module_records: SCC units, groups as RecGroup pages
old = '''pub fn publish_module_records(
    store: &ManagedStore,
    input: &SemaInput,
    project: Option<&str>,
) -> Result<BTreeMap<String, String>, ProbeError> {
    let mut out = BTreeMap::new();
    let mut remaining: Vec<String> = input.records.iter().map(|r| r.name.clone()).collect();
    while !remaining.is_empty() {
        let mut progressed = false;
        let mut next_round: Vec<String> = Vec::new();
        for name in &remaining {
            let deps = record_deps(input, name);
            let ready = deps.iter().all(|d| {
                out.contains_key(d)
                    || matches!(
                        lookup_only(store, input, &QueryKind::RecordFields { record: d.clone() }),
                        Ok(FetchOutcome::Hit { .. })
                    )
            });
            if ready {
                let art = ensure_record_page(store, input, name, project)?;
                out.insert(name.clone(), art);
                progressed = true;
            } else {
                next_round.push(name.clone());
            }
        }
        remaining = next_round;
        if !progressed {
            return Err(ProbeError::CyclicRecordGroup { cycle: remaining.join(" -> ") });
        }
    }
    Ok(out)
}'''
new = '''pub fn publish_module_records(
    store: &ManagedStore,
    input: &SemaInput,
    project: Option<&str>,
) -> Result<BTreeMap<String, String>, ProbeError> {
    let mut out = BTreeMap::new();
    for unit in scc_units(input) {
        let unit_set: std::collections::BTreeSet<String> =
            unit.iter().cloned().collect();
        let is_group = unit.len() > 1
            || unit.iter().any(|n| record_deps(input, n).iter().any(|d| d == n));
        if !is_group {
            // singleton without self-reference: single page (reused if
            // a page for exactly this record already exists)
            let name = &unit[0];
            if let Ok(FetchOutcome::Hit { artifact_id, .. }) =
                lookup_only(store, input, &QueryKind::RecordFields { record: name.clone() })
            {
                out.insert(name.clone(), artifact_id);
                continue;
            }
            let art = ensure_record_page(store, input, name, project)?;
            out.insert(name.clone(), art);
            continue;
        }
        // recursive group: ONE page for the whole SCC (internal refs
        // are plain DeclRefs inside the page; external refs must be
        // published already and become required-data edges)
        let mut required_edges: Vec<String> = Vec::new();
        let mut members: Vec<ac_sema_contract::RecordShape> = Vec::new();
        for name in &unit {
            members.push(build_shape(store, input, name, &unit_set, &mut required_edges)?);
        }
        let group = ac_sema_contract::RecGroup { members: members.clone() };
        let page = SemaPage {
            header: PageHeader {
                schema: SEMA_PAGE_SCHEMA_V1.into(),
                producer: ac_sema_contract::SEMA_PRODUCER_ID.into(),
                codec_version: ac_sema_contract::SEMA_CODEC_VERSION,
                kind: PageKind::RecGroup,
                content: group.group_content(),
            },
            body: PageBody::RecGroup(members),
        };
        // key: the group's canonical identity (sorted member names)
        let mut member_names = unit.clone();
        member_names.sort();
        let key = ac_sema_core::SemaQueryKey {
            components: vec![
                ("query".into(), "record-group".into()),
                ("package".into(), input.package.clone()),
                ("module".into(), input.module.clone()),
                ("members".into(), member_names.join("+")),
            ],
        };
        let payload = encode_page(&page)?;
        let edges: Vec<(String, ac_artifact_probe::identity::RefKind)> = required_edges
            .into_iter()
            .map(|a| (a, ac_artifact_probe::identity::RefKind::RequiredData))
            .collect();
        let req = PutRequest {
            layer: SEMA_LAYER,
            key: QueryKey { layer: SEMA_LAYER, components: key.components.clone() },
            kind: SEMA_CARRIER,
            payload,
            edges,
            project: project.map(|s| s.to_string()),
        };
        let art = store.put_artifact(req)?;
        for name in &unit {
            out.insert(name.clone(), art.clone());
        }
    }
    Ok(out)
}'''
assert old in s, 'module records anchor'
s = s.replace(old, new)
open(p, 'w', encoding='utf8', newline='\n').write(s)
print('store group ok')
