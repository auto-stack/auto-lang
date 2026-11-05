//! Finite namespace and visibility binding over the shared projected AST.
use super::*;
use crate::source::ast::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct ModuleInput {
    pub id: String,
    pub source: String,
}
#[derive(Clone, Debug)]
pub struct ModuleDiagnostic {
    pub module: String,
    pub diagnostic: Diagnostic,
}
#[derive(Clone, Debug)]
pub struct SourceSegment {
    pub module: String,
    pub source: String,
    pub start: usize,
}
#[derive(Clone, Debug)]
pub struct BoundModules {
    pub entry: String,
    pub capabilities: Vec<String>,
    pub program: Program,
    pub units: BTreeMap<String, ModuleSyntax>,
    pub segments: Vec<SourceSegment>,
    /// Internal function symbol -> original module and source function name.
    pub origins: BTreeMap<String, (String, String)>,
    /// Actual direct calls, not a declaration inventory.
    pub references: BTreeMap<String, BTreeSet<String>>,
    pub binding_walks: BTreeSet<String>,
    pub input_hash: String,
    pub grammar_id: String,
}
impl BoundModules {
    pub fn source_len(&self) -> usize {
        self.segments.last().map_or(0, |s| s.start + s.source.len())
    }
    pub fn is_char_boundary(&self, pos: usize) -> bool {
        self.segments
            .iter()
            .any(|s| pos >= s.start && s.source.is_char_boundary(pos - s.start))
    }
    pub fn locate(&self, mut diagnostic: Diagnostic) -> ModuleDiagnostic {
        let segment = self
            .segments
            .iter()
            .rev()
            .find(|s| diagnostic.span.start >= s.start);
        let module = segment.map(|s| s.module.clone()).unwrap_or_default();
        if let Some(s) = segment {
            diagnostic.span.start = diagnostic.span.start.saturating_sub(s.start);
            diagnostic.span.end = diagnostic.span.end.saturating_sub(s.start);
        }
        ModuleDiagnostic { module, diagnostic }
    }
}
fn fail(
    module: &str,
    code: &'static str,
    message: impl Into<String>,
    span: Span,
) -> ModuleDiagnostic {
    ModuleDiagnostic {
        module: module.into(),
        diagnostic: Diagnostic::new(Stage::Bind, code, message, span),
    }
}
fn symbol(module: &str, name: &str, entry: &str) -> String {
    if module == entry && name == "main" {
        "main".into()
    } else {
        format!("m{}_{}_{}", module.len(), module, name)
    }
}
fn valid_id(id: &str) -> bool {
    let mut b = id.bytes();
    matches!(b.next(), Some(b'a'..=b'z' | b'A'..=b'Z' | b'_'))
        && b.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// Bind a host-supplied finite module graph. No filesystem discovery, source
/// concatenation, parser fallback, type synthesis or Checked construction.
pub fn bind_modules(
    inputs: &[ModuleInput],
    entry: &str,
    caps: &[String],
    budget: FrontBudget,
) -> Result<BoundModules, ModuleDiagnostic> {
    let mut units = BTreeMap::new();
    let mut total_bytes = 0usize;
    for input in inputs {
        total_bytes = total_bytes.checked_add(input.source.len()).ok_or_else(|| {
            fail(
                &input.id,
                "source.resource",
                "module input byte budget",
                Span { start: 0, end: 0 },
            )
        })?;
        if total_bytes > budget.work_bytes {
            return Err(fail(
                &input.id,
                "source.resource",
                "module input byte budget",
                Span { start: 0, end: 0 },
            ));
        }
        if !valid_id(&input.id) || units.contains_key(&input.id) {
            return Err(fail(
                &input.id,
                "source.module-identity",
                "duplicate module identity",
                Span { start: 0, end: 0 },
            ));
        }
        let syntax = parse_module_source(&input.source, caps, budget).map_err(|mut ds| {
            ModuleDiagnostic {
                module: input.id.clone(),
                diagnostic: ds.remove(0),
            }
        })?;
        units.insert(input.id.clone(), syntax);
    }
    bind_module_syntax(inputs, entry, caps, budget, units)
}

/// Shared binding of already projected cold, restored or edited source units.
/// The source/grammar/capability identities are re-admitted before names bind.
pub fn bind_module_syntax(
    inputs: &[ModuleInput],
    entry: &str,
    caps: &[String],
    budget: FrontBudget,
    units: BTreeMap<String, ModuleSyntax>,
) -> Result<BoundModules, ModuleDiagnostic> {
    bind_module_syntax_reusing(inputs, entry, caps, budget, units, None)
}
pub(super) fn bind_module_syntax_reusing(
    inputs: &[ModuleInput],
    entry: &str,
    caps: &[String],
    budget: FrontBudget,
    units: BTreeMap<String, ModuleSyntax>,
    previous: Option<&BoundModules>,
) -> Result<BoundModules, ModuleDiagnostic> {
    let zero = Span { start: 0, end: 0 };
    if !caps.iter().any(|c| c == MODULES_CAPABILITY)
        || caps.iter().any(|c| c == super::INSTANCES_CAPABILITY)
    {
        return Err(fail(
            entry,
            "source.module-capability",
            "module capability mismatch",
            zero,
        ));
    }
    let grammar = super::module_grammar().map_err(|e| fail(entry, "source.infra", e, zero))?;
    let grammar_id = format!("{}", grammar.program().identity.grammar_id.0);
    let mut sources = BTreeMap::new();
    let mut bytes = 0usize;
    for input in inputs {
        bytes = bytes.checked_add(input.source.len()).ok_or_else(|| {
            fail(
                &input.id,
                "source.resource",
                "module input byte budget",
                zero,
            )
        })?;
        if bytes > budget.work_bytes {
            return Err(fail(
                &input.id,
                "source.resource",
                "module input byte budget",
                zero,
            ));
        }
        if !valid_id(&input.id) || sources.contains_key(&input.id) {
            return Err(fail(
                &input.id,
                "source.module-identity",
                "invalid or duplicate module identity",
                zero,
            ));
        }
        let u = units.get(&input.id).ok_or_else(|| {
            fail(
                &input.id,
                "source.module-missing",
                "source unit absent",
                zero,
            )
        })?;
        if u.source_hash != crate::runtime_contract::sha256_hex(input.source.as_bytes())
            || u.grammar_id != grammar_id
        {
            return Err(fail(
                &input.id,
                "source.module-stale",
                "source/grammar identity mismatch",
                zero,
            ));
        }
        sources.insert(input.id.clone(), input.source.clone());
    }
    if units.len() != sources.len() {
        return Err(fail(
            entry,
            "source.module-identity",
            "extraneous projected module",
            zero,
        ));
    }
    if !units.contains_key(entry) {
        return Err(fail(
            entry,
            "source.module-missing",
            "entry module absent",
            zero,
        ));
    }
    let mut declarations = BTreeMap::new();
    for (id, unit) in &units {
        let mut names = BTreeMap::new();
        for item in &unit.program.items {
            let Item::Fn(f) = item else {
                return Err(fail(
                    id,
                    "source.module-surface",
                    "only scalar free functions are supported",
                    zero,
                ));
            };
            if names.insert(f.name.name.clone(), f.name.span).is_some() {
                return Err(fail(
                    id,
                    "source.module-duplicate-function",
                    format!("duplicate function `{}`", f.name.name),
                    f.name.span,
                ));
            }
        }
        declarations.insert(id.clone(), names);
    }
    // Visibility is validated for declarations even when a local function
    // shadows the imported bare name. Qualification never bypasses admission.
    let admit =
        |owner: &str, target: &str, name: &str, span: Span| -> Result<String, ModuleDiagnostic> {
            let u = units.get(target).ok_or_else(|| {
                fail(
                    owner,
                    "source.module-missing",
                    format!("unknown module `{target}`"),
                    span,
                )
            })?;
            if !declarations[target].contains_key(name) {
                return Err(fail(
                    owner,
                    "source.module-symbol",
                    format!("unknown function `{target}.{name}`"),
                    span,
                ));
            }
            if owner != target && !u.exports.contains(name) {
                return Err(fail(
                    owner,
                    "source.module-private",
                    format!("private function `{target}.{name}`"),
                    span,
                ));
            }
            Ok(symbol(target, name, entry))
        };
    let scope_same = previous.is_some_and(|old| {
        old.entry == entry
            && old.capabilities
                == caps
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>()
            && old.units.keys().eq(units.keys())
            && units.iter().all(|(id, u)| {
                let o = &old.units[id];
                o.exports == u.exports
                    && o.declarations.keys().eq(u.declarations.keys())
                    && o.imports
                        .iter()
                        .map(|i| (&i.module, &i.name))
                        .eq(u.imports.iter().map(|i| (&i.module, &i.name)))
            })
    });
    let mut result = BoundModules {
        entry: entry.into(),
        capabilities: caps
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        program: Program { items: Vec::new() },
        units: BTreeMap::new(),
        segments: Vec::new(),
        origins: BTreeMap::new(),
        references: BTreeMap::new(),
        binding_walks: BTreeSet::new(),
        input_hash: String::new(),
        grammar_id: String::new(),
    };
    let mut edges: BTreeMap<String, BTreeSet<String>> =
        units.keys().map(|k| (k.clone(), BTreeSet::new())).collect();
    let mut offset = 0;
    let mut identity = format!("{MODULE_BINDING_VERSION}:{}:{entry}\n", entry.len());
    for (id, unit) in &units {
        result.grammar_id = unit.grammar_id.clone();
        let mut imports = BTreeMap::new();
        for i in &unit.imports {
            imports.insert(i.name.clone(), admit(id, &i.module, &i.name, i.span)?);
            edges.get_mut(id).unwrap().insert(i.module.clone());
        }
        identity.push_str(&format!(
            "{}:{}:{}\n",
            id.len(),
            id,
            crate::runtime_contract::sha256_hex(sources[id].as_bytes())
        ));
        for item in &unit.program.items {
            let Item::Fn(original) = item else {
                unreachable!()
            };
            let mut f = original.clone();
            let owner = symbol(id, &f.name.name, entry);
            if scope_same {
                let old = previous.expect("scope comparison needs old graph");
                let old_source = &old
                    .segments
                    .iter()
                    .find(|s| s.module == *id)
                    .expect("old module")
                    .source;
                let old_original = old.units[id].program.items.iter().find_map(|i| match i {
                    Item::Fn(f) if f.name.name == original.name.name => Some(f),
                    _ => None,
                });
                if let Some(old_original) = old_original {
                    if old_source.get(old_original.span.start..old_original.span.end)
                        == sources[id].get(original.span.start..original.span.end)
                    {
                        let old_bound = old
                            .program
                            .items
                            .iter()
                            .find_map(|i| match i {
                                Item::Fn(f) if f.name.name == owner => Some(f),
                                _ => None,
                            })
                            .expect("old bound owner");
                        let mut reused = old_bound.clone();
                        crate::source::cst::shift_fn_item_signed(
                            &mut reused,
                            (offset + original.span.start) as i64 - old_bound.span.start as i64,
                        );
                        let refs = old.references[&owner].clone();
                        for target in &refs {
                            if let Some((target_module, _)) = old.origins.get(target) {
                                if target_module != id {
                                    edges.get_mut(id).unwrap().insert(target_module.clone());
                                }
                            }
                        }
                        result.references.insert(owner.clone(), refs);
                        result
                            .origins
                            .insert(owner, (id.clone(), original.name.name.clone()));
                        result.program.items.push(Item::Fn(reused));
                        continue;
                    }
                }
            }
            result.binding_walks.insert(owner.clone());
            let mut refs = BTreeSet::new();
            // Explicit frames keep namespace binding independent of native
            // stack depth; lexical local sets preserve call-site shadowing.
            enum Task<'a> {
                Statements(&'a mut [Stmt], BTreeSet<String>),
                Expr(&'a mut Expr, BTreeSet<String>),
            }
            let locals = f.params.iter().map(|p| p.name.name.clone()).collect();
            let mut tasks = vec![Task::Statements(&mut f.body.stmts, locals)];
            let mut work = 0usize;
            while let Some(task) = tasks.pop() {
                work += 1;
                if work > (1 << 22) {
                    return Err(fail(
                        id,
                        "source.resource",
                        "module binding work budget",
                        original.span,
                    ));
                }
                match task {
                    Task::Statements(stmts, mut locals) => {
                        if let Some((s, rest)) = stmts.split_first_mut() {
                            let before = locals.clone();
                            if let Stmt::Let { name, .. } = s {
                                locals.insert(name.name.clone());
                            }
                            tasks.push(Task::Statements(rest, locals));
                            match s {
                                Stmt::Let { value, .. }
                                | Stmt::Return { value, .. }
                                | Stmt::ExprStmt { value, .. } => {
                                    tasks.push(Task::Expr(value, before))
                                }
                                Stmt::Assign { target, value, .. } if target.path.is_empty() => {
                                    tasks.push(Task::Expr(value, before))
                                }
                                Stmt::If {
                                    cond, then, els, ..
                                } => {
                                    if let Some(e) = els {
                                        tasks.push(Task::Statements(&mut e.stmts, before.clone()));
                                    }
                                    tasks.push(Task::Statements(&mut then.stmts, before.clone()));
                                    tasks.push(Task::Expr(cond, before));
                                }
                                Stmt::While { cond, body, .. } => {
                                    tasks.push(Task::Statements(&mut body.stmts, before.clone()));
                                    tasks.push(Task::Expr(cond, before));
                                }
                                Stmt::Loop { body, .. } => {
                                    tasks.push(Task::Statements(&mut body.stmts, before))
                                }
                                Stmt::Break { .. } | Stmt::Continue { .. } => (),
                                _ => {
                                    return Err(fail(
                                        id,
                                        "source.module-surface",
                                        "statement outside scalar modules",
                                        original.span,
                                    ))
                                }
                            }
                        }
                    }
                    Task::Expr(e, locals) => match &mut e.kind {
                        ExprKind::Call { callee, args } => {
                            let resolved = if let Some((target, name)) = callee.name.split_once('.')
                            {
                                if locals.contains(target) {
                                    return Err(fail(
                                        id,
                                        "source.module-shadowed",
                                        format!("local `{target}` is not a module"),
                                        callee.span,
                                    ));
                                }
                                let resolved = admit(id, target, name, callee.span)?;
                                if target != id {
                                    edges.get_mut(id).unwrap().insert(target.into());
                                }
                                resolved
                            } else if locals.contains(&callee.name) {
                                // Preserve a non-callable local for the unique resolver.
                                callee.name.clone()
                            } else if declarations[id].contains_key(&callee.name) {
                                symbol(id, &callee.name, entry)
                            } else if let Some(imported) = imports.get(&callee.name) {
                                imported.clone()
                            } else {
                                return Err(fail(
                                    id,
                                    "source.module-symbol",
                                    format!("unknown function `{}`", callee.name),
                                    callee.span,
                                ));
                            };
                            refs.insert(resolved.clone());
                            callee.name = resolved;
                            for arg in args.iter_mut().rev() {
                                match arg {
                                    Arg::Pos(v) | Arg::Named { value: v, .. } => {
                                        tasks.push(Task::Expr(v, locals.clone()))
                                    }
                                    _ => {
                                        return Err(fail(
                                            id,
                                            "source.module-surface",
                                            "move arguments outside scalar modules",
                                            e.span,
                                        ))
                                    }
                                }
                            }
                        }
                        ExprKind::Binary { lhs, rhs, .. } => {
                            tasks.push(Task::Expr(rhs, locals.clone()));
                            tasks.push(Task::Expr(lhs, locals));
                        }
                        ExprKind::Unary { operand, .. } => tasks.push(Task::Expr(operand, locals)),
                        ExprKind::IntLit(_) | ExprKind::BoolLit(_) => (),
                        ExprKind::Var(v) if !v.name.contains('.') => (),
                        _ => {
                            return Err(fail(
                                id,
                                "source.module-surface",
                                "expression outside scalar modules",
                                e.span,
                            ))
                        }
                    },
                }
            }
            result
                .origins
                .insert(owner.clone(), (id.clone(), f.name.name.clone()));
            result.references.insert(owner.clone(), refs);
            f.name.name = owner;
            crate::source::cst::shift_fn_item(&mut f, offset);
            result.program.items.push(Item::Fn(f));
        }
        result.segments.push(SourceSegment {
            module: id.clone(),
            source: sources[id].clone(),
            start: offset,
        });
        offset += sources[id].len() + 1;
    }
    // Kahn's algorithm admits a finite DAG without a recursive graph walk.
    let mut pending = edges;
    while !pending.is_empty() {
        let leaves: BTreeSet<_> = pending
            .iter()
            .filter(|(_, deps)| deps.is_empty())
            .map(|(k, _)| k.clone())
            .collect();
        if leaves.is_empty() {
            return Err(fail(
                entry,
                "source.module-cycle",
                "cyclic module dependency",
                zero,
            ));
        }
        pending.retain(|k, _| !leaves.contains(k));
        for deps in pending.values_mut() {
            deps.retain(|k| !leaves.contains(k));
        }
    }
    result.input_hash = crate::runtime_contract::sha256_hex(identity.as_bytes());
    result.units = units;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn bind(a: &str, b: &str) -> Result<BoundModules, ModuleDiagnostic> {
        bind_modules(
            &[
                ModuleInput {
                    id: "std".into(),
                    source: a.into(),
                },
                ModuleInput {
                    id: "app".into(),
                    source: b.into(),
                },
            ],
            "app",
            &[MODULES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
        )
    }
    #[test]
    fn namespaces_visibility_and_local_precedence_are_bound_in_the_ast() {
        let b=bind("pub fn answer() int { return helper() } fn helper() int { return 42 }",
                   "use std.answer\nfn answer() int { return helper() } fn helper() int { return 7 } fn main() int { return std.answer() + answer() }").unwrap();
        assert_eq!(b.program.items.len(), 5);
        assert_eq!(
            b.references["main"],
            BTreeSet::from(["m3_std_answer".into(), "m3_app_answer".into()])
        );
        assert_eq!(
            b.references["m3_std_answer"],
            BTreeSet::from(["m3_std_helper".into()])
        );
        assert_eq!(
            b.references["m3_app_answer"],
            BTreeSet::from(["m3_app_helper".into()])
        );
        for item in &b.program.items {
            let Item::Fn(f) = item else { panic!("fn") };
            assert!(b.is_char_boundary(f.name.span.start));
            let origin = b.locate(Diagnostic::new(Stage::Bind, "test", "", f.name.span));
            let segment = b
                .segments
                .iter()
                .find(|s| s.module == origin.module)
                .unwrap();
            assert_eq!(
                &segment.source[origin.diagnostic.span.start..origin.diagnostic.span.end],
                b.origins[&f.name.name].1
            );
        }
    }
    #[test]
    fn module_admission_rejects_private_missing_duplicate_and_cycles() {
        for (library, entry, code) in [
            (
                "fn hidden() int { return 42 }",
                "fn main() int { return std.hidden() }",
                "source.module-private",
            ),
            (
                "pub fn answer() int { return 42 }",
                "fn main() int { return other.answer() }",
                "source.module-missing",
            ),
            (
                "pub fn answer() int { return 42 }",
                "fn main() int { return std.absent() }",
                "source.module-symbol",
            ),
            (
                "pub fn answer() int { return 42 }",
                "fn main() int { return 1 } fn main() int { return 2 }",
                "source.module-duplicate-function",
            ),
            (
                "use app.main\npub fn answer() int { return main() }",
                "use std.answer\npub fn main() int { return answer() }",
                "source.module-cycle",
            ),
        ] {
            let e = bind(library, entry).unwrap_err();
            assert_eq!(e.diagnostic.code, code, "{e:?}");
        }
        let duplicate = ModuleInput {
            id: "app".into(),
            source: "fn main() int { return 1 }".into(),
        };
        assert_eq!(
            bind_modules(
                &[duplicate.clone(), duplicate],
                "app",
                &[MODULES_CAPABILITY.into()],
                FrontBudget::WORK_OWNED_64MIB
            )
            .unwrap_err()
            .diagnostic
            .code,
            "source.module-identity"
        );
    }
}
