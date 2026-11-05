//! Declaration-origin relocation and single-function module edits.
use super::*;
use crate::source::ast::{Item, Program};
use std::collections::BTreeMap;
fn error(module: &str, message: impl Into<String>, span: Span) -> ModuleDiagnostic {
    ModuleDiagnostic {
        module: module.into(),
        diagnostic: Diagnostic::new(Stage::Bind, "source.module-edit", message, span),
    }
}
fn shifted(span: Span, delta: i64) -> Span {
    Span {
        start: (span.start as i64 + delta) as usize,
        end: (span.end as i64 + delta) as usize,
    }
}

/// Assemble declarations re-projected from authenticated immutable origins.
/// Current source slices must be exactly the mint slices; arbitrary relocation
/// cannot manufacture an unparsed declaration or change its visibility.
pub fn assemble_relocated_module(
    source: &str,
    items: Vec<(ModuleSyntax, Vec<DeclarationPlacement>)>,
    pieces: Vec<SyntaxPiece>,
    grammar_id: String,
) -> Result<ModuleSyntax, String> {
    let mut functions = Vec::new();
    let mut imports = Vec::new();
    let mut exports = BTreeSet::new();
    let mut declarations = BTreeMap::new();
    for (syntax, placements) in items {
        for item in syntax.program.items {
            let Item::Fn(mut f) = item else {
                return Err("non-function stored module item".into());
            };
            let span = syntax
                .declarations
                .get(&f.name.name)
                .ok_or("missing declaration extent")?;
            let p = placements
                .iter()
                .find(|p| p.mint.start <= span.start && p.mint.end >= span.end)
                .ok_or("unplaced function declaration")?;
            let delta = p.current.start as i64 - p.mint.start as i64;
            crate::source::cst::shift_fn_item_signed(&mut f, delta);
            if declarations
                .insert(f.name.name.clone(), shifted(*span, delta))
                .is_some()
            {
                return Err("duplicate restored module function".into());
            }
            functions.push(Item::Fn(f));
        }
        for mut import in syntax.imports {
            let p = placements
                .iter()
                .find(|p| p.mint.start <= import.span.start && p.mint.end >= import.span.end)
                .ok_or("unplaced import")?;
            import.span = shifted(import.span, p.current.start as i64 - p.mint.start as i64);
            imports.push(import);
        }
        exports.extend(syntax.exports);
    }
    functions.sort_by_key(|f| match f {
        Item::Fn(f) => f.span.start,
        _ => usize::MAX,
    });
    imports.sort_by_key(|i| i.span.start);
    let first_fn = declarations
        .values()
        .map(|s| s.start)
        .min()
        .unwrap_or(usize::MAX);
    let mut names = BTreeSet::new();
    for i in &imports {
        if i.span.start >= first_fn || !names.insert(i.name.clone()) {
            return Err("restored import order/duplicate mismatch".into());
        }
    }
    let mut ranges: Vec<_> = pieces
        .iter()
        .flat_map(|p| p.placements.iter().map(move |d| (p, d)))
        .collect();
    ranges.sort_by_key(|(_, d)| d.current.start);
    let mut end = 0;
    for (piece, d) in ranges {
        if d.current.start < end
            || piece.source.get(d.mint.start..d.mint.end)
                != source.get(d.current.start..d.current.end)
        {
            return Err("restored declaration slice/overlap mismatch".into());
        }
        end = d.current.end;
    }
    Ok(ModuleSyntax {
        program: Program { items: functions },
        exports,
        imports,
        declarations,
        pieces,
        grammar_id,
        source_hash: crate::runtime_contract::sha256_hex(source.as_bytes()),
    })
}

/// Parse only the replacement's real declaration. Unchanged ASTs and syntax
/// origins are retained; all source slicing is an authorized edit to ONE input,
/// never concatenation of library sources into a substitute compiler input.
pub fn edit_module_function(
    base: &BoundModules,
    module: &str,
    name: &str,
    fragment: &str,
    budget: FrontBudget,
) -> Result<BoundModules, ModuleDiagnostic> {
    let zero = Span { start: 0, end: 0 };
    let unit = base
        .units
        .get(module)
        .ok_or_else(|| error(module, "module absent", zero))?;
    let range = *unit
        .declarations
        .get(name)
        .ok_or_else(|| error(module, "function absent", zero))?;
    let parsed =
        crate::source::cst::parse_module_fragment(fragment, budget).map_err(|mut ds| {
            ModuleDiagnostic {
                module: module.into(),
                diagnostic: ds.remove(0),
            }
        })?;
    if parsed.program.items.len() != 1
        || !parsed.imports.is_empty()
        || parsed.declarations.keys().next().map(String::as_str) != Some(name)
    {
        return Err(error(
            module,
            "replacement must contain exactly the same named free function",
            range,
        ));
    }
    let old_source = &base
        .segments
        .iter()
        .find(|s| s.module == module)
        .expect("bound input source")
        .source;
    let mut source = old_source.clone();
    source.replace_range(range.start..range.end, fragment);
    let delta = fragment.len() as i64 - (range.end - range.start) as i64;
    let mut next = unit.clone();
    let mut replacement = match parsed.program.items[0].clone() {
        Item::Fn(f) => f,
        _ => unreachable!(),
    };
    crate::source::cst::shift_fn_item(&mut replacement, range.start);
    for item in &mut next.program.items {
        if let Item::Fn(f) = item {
            if f.name.name == name {
                *f = replacement.clone();
            } else if f.span.start >= range.end {
                crate::source::cst::shift_fn_item_signed(f, delta);
            }
        }
    }
    for (n, span) in &mut next.declarations {
        if n == name {
            *span = shifted(parsed.declarations[name], range.start as i64);
        } else if span.start >= range.end {
            *span = shifted(*span, delta);
        }
    }
    next.exports.remove(name);
    if parsed.exports.contains(name) {
        next.exports.insert(name.into());
    }
    for piece in &mut next.pieces {
        piece.placements.retain(|p| p.current != range);
        for p in &mut piece.placements {
            if p.current.start >= range.end {
                p.current = shifted(p.current, delta);
            }
        }
    }
    next.pieces.retain(|p| !p.placements.is_empty());
    for mut piece in parsed.pieces {
        for p in &mut piece.placements {
            p.current = shifted(p.current, range.start as i64);
        }
        next.pieces.push(piece);
    }
    next.source_hash = crate::runtime_contract::sha256_hex(source.as_bytes());
    let mut units = base.units.clone();
    units.insert(module.into(), next);
    let inputs: Vec<_> = base
        .segments
        .iter()
        .map(|s| binding::ModuleInput {
            id: s.module.clone(),
            source: if s.module == module {
                source.clone()
            } else {
                s.source.clone()
            },
        })
        .collect();
    binding::bind_module_syntax_reusing(
        &inputs,
        &base.entry,
        &base.capabilities,
        budget,
        units,
        Some(base),
    )
}
