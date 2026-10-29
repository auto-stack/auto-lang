//! Entry resolution shared by the `ac-native` CLI (PLAN-871 T-04) and the
//! ac-case worker's mn backend arm.
//!
//! The retiring CLI had two spellings on purpose (T-02 F4 frozen contract):
//! raw-HIR mode took the entry DEF ID verbatim, source mode took the
//! declared function NAME, which maps to `d_<name>` (the adapter's
//! identity id). The HIR `name` field is the native symbol (`sf_<name>` /
//! `sm_<Rec>_<name>`), so it is NEVER the lookup key for source-derived
//! modules — but hand-authored HIR documents do carry author-chosen names,
//! so a unique declared-name match is still honored.
//!
//! Ambiguity is an error: entry resolution never picks one of several
//! matches.

use auto_ac_prototype::atom_text::{Diagnostic, Span, Stage};
use auto_ac_prototype::hir::DefKind;
use auto_ac_prototype::verify::CheckedModule;

/// Resolve `--entry`/request entry to `(def id text, declared name)`.
pub fn resolve(checked: &CheckedModule, entry: &str) -> Result<(String, String), Vec<Diagnostic>> {
    let by_name = format!("d_{entry}");
    let mut hits: Vec<(String, String)> = Vec::new();
    for m in &checked.bundle().modules {
        for d in &m.defs {
            let name = match &d.kind {
                DefKind::Function { name, .. }
                    if d.id_text == entry || d.id_text == by_name || name == entry =>
                {
                    name.clone()
                }
                _ => continue,
            };
            hits.push((d.id_text.clone(), name));
        }
    }
    match hits.as_slice() {
        [one] => Ok(one.clone()),
        [] => Err(vec![Diagnostic::new(
            Stage::Backend,
            "entry.not-found",
            format!(
                "no function declaration matches entry `{entry}` (def id, `d_<name>` or declared name)"
            ),
            Span { start: 0, end: 0 },
        )]),
        _ => Err(vec![Diagnostic::new(
            Stage::Backend,
            "entry.ambiguous",
            format!(
                "entry `{entry}` matches {} declarations: {}",
                hits.len(),
                hits.iter()
                    .map(|(id, _)| id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Span { start: 0, end: 0 },
        )]),
    }
}
