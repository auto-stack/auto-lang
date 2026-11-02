use crate::ast::{Body, Code, Fn, Name, Stmt, Type};
use crate::error::{AutoError, TypeError};
use crate::infer::InferenceContext;
use crate::types::TypeStore;
use miette::SourceSpan;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

mod walk;

pub(super) fn check(
    ast: &Code,
    sibling_store: Option<Arc<RwLock<TypeStore>>>,
) -> Result<(), AutoError> {
    let type_store = sibling_store.unwrap_or_else(|| Arc::new(RwLock::new(TypeStore::new())));
    let mut functions = HashMap::new();
    {
        let mut store = type_store.write().unwrap();
        for stmt in &ast.stmts {
            match stmt {
                Stmt::Fn(decl) => {
                    functions.insert(decl.name.clone(), decl);
                    store.register_fn_decl(decl);
                }
                Stmt::TypeDecl(decl) => store.register_type_decl(decl),
                Stmt::SpecDecl(decl) => store.register_spec_decl(decl),
                Stmt::EnumDecl(decl) => store.register_enum_decl(decl.clone()),
                Stmt::Ext(decl) => store.register_ext_methods(decl),
                _ => {}
            }
        }
    }

    for stmt in &ast.stmts {
        match stmt {
            Stmt::Fn(decl) => check_fn(decl, ast, &functions, type_store.clone())?,
            Stmt::TypeDecl(decl) => {
                for method in &decl.methods {
                    check_fn(method, ast, &functions, type_store.clone())?;
                }
            }
            Stmt::Ext(decl) => {
                for method in &decl.methods {
                    check_fn(method, ast, &functions, type_store.clone())?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn check_fn(
    decl: &Fn,
    ast: &Code,
    functions: &HashMap<Name, &Fn>,
    type_store: Arc<RwLock<TypeStore>>,
) -> Result<(), AutoError> {
    let known_symbols = type_store.read().unwrap().get_defined_names();
    let mut ctx = InferenceContext::with_type_store(type_store);
    for name in known_symbols {
        ctx.bind_var(
            Name::from(name),
            Type::Fn(Vec::new(), Box::new(Type::Unknown)),
        );
    }
    for stmt in &ast.stmts {
        match stmt {
            Stmt::Store(store) => ctx.bind_var(store.name.clone(), store.ty.clone()),
            Stmt::TypeDecl(decl) => ctx.bind_var(
                decl.name.clone(),
                Type::Fn(Vec::new(), Box::new(Type::Unknown)),
            ),
            Stmt::EnumDecl(decl) => {
                ctx.bind_var(
                    decl.name.clone(),
                    Type::Fn(Vec::new(), Box::new(Type::Unknown)),
                );
                for item in &decl.items {
                    ctx.bind_var(
                        item.name.clone(),
                        Type::Fn(Vec::new(), Box::new(Type::Unknown)),
                    );
                }
            }
            Stmt::Union(decl) => ctx.bind_var(
                decl.name.clone(),
                Type::Fn(Vec::new(), Box::new(Type::Unknown)),
            ),
            Stmt::Tag(decl) => ctx.bind_var(
                decl.name.clone(),
                Type::Fn(Vec::new(), Box::new(Type::Unknown)),
            ),
            Stmt::SpecDecl(decl) => ctx.bind_var(
                decl.name.clone(),
                Type::Fn(Vec::new(), Box::new(Type::Unknown)),
            ),
            _ => {}
        }
    }
    ctx.push_fn_scope();
    for param in &decl.params {
        ctx.bind_var(param.name.clone(), param.ty.clone());
    }
    for stmt in &decl.body.stmts {
        if let Stmt::Store(store) = stmt {
            ctx.bind_var(store.name.clone(), store.ty.clone());
        }
    }
    ctx.current_ret = Some(decl.ret.clone());
    walk::check_fn(decl, ast, functions, ctx)
}

fn check_return_paths(decl: &Fn) -> Result<(), AutoError> {
    if matches!(decl.ret, Type::Void | Type::Unknown | Type::Handle { .. })
        || matches!(&decl.ret, Type::Result(inner) if matches!(inner.as_ref(), Type::Void | Type::Unknown))
        || matches!(&decl.ret, Type::GenericInstance(inst) if matches!(inst.base_name.as_str(), "Future" | "Stream"))
        || body_guarantees_return(&decl.body)
        || body_has_tail_value(&decl.body)
    {
        return Ok(());
    }
    Err(TypeError::Mismatch {
        expected: format!("return {} on every path", decl.ret),
        found: "path without a return value".into(),
        span: fn_span(decl.span),
    }
    .into())
}

fn body_has_tail_value(body: &Body) -> bool {
    match body.stmts.last() {
        Some(Stmt::Expr(_) | Stmt::Is(_) | Stmt::Node(_)) => true,
        Some(Stmt::Block(body)) => body_has_tail_value(body),
        Some(Stmt::If(if_)) => {
            if_.else_.is_some()
                && if_.branches.iter().all(|branch| {
                    body_guarantees_return(&branch.body) || body_has_tail_value(&branch.body)
                })
                && if_
                    .else_
                    .as_ref()
                    .is_some_and(|body| body_guarantees_return(body) || body_has_tail_value(body))
        }
        _ => false,
    }
}

fn body_guarantees_return(body: &Body) -> bool {
    body.stmts.iter().any(stmt_guarantees_return)
}

fn stmt_guarantees_return(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Return(_) => true,
        Stmt::Block(body) => body_guarantees_return(body),
        Stmt::If(if_) => {
            if if_.branches.is_empty() || if_.else_.is_none() {
                return false;
            }
            if_.branches
                .iter()
                .all(|branch| body_guarantees_return(&branch.body))
                && if_
                    .else_
                    .as_ref()
                    .is_some_and(|body| body_guarantees_return(body))
        }
        _ => false,
    }
}

fn fn_span(span: Option<(usize, usize)>) -> SourceSpan {
    span.map(|(offset, len)| SourceSpan::new(offset.into(), len.max(1)))
        .unwrap_or_else(|| SourceSpan::new(0.into(), 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trans::rust::transpile_rust;

    fn rejects(source: &str) -> AutoError {
        match transpile_rust("semantic_admission", source) {
            Ok(_) => panic!("illegal source emitted Rust:\n{}", source),
            Err(error) => error,
        }
    }

    #[test]
    fn original_semantic_negatives_are_typed_rejections() {
        let sources = [
            (
                "B16",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B16.at"),
            ),
            (
                "B17",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B17.at"),
            ),
            (
                "B18",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B18.at"),
            ),
            (
                "B19",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B19.at"),
            ),
            (
                "T06",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/T06.at"),
            ),
            (
                "R09",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/R09.at"),
            ),
            (
                "R10",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/R10.at"),
            ),
        ];
        for (name, source) in sources {
            let error = rejects(source);
            assert!(
                matches!(
                    error,
                    AutoError::Syntax(_)
                        | AutoError::SyntaxWithSource(_)
                        | AutoError::Type(_)
                        | AutoError::Name(_)
                        | AutoError::MultipleErrors { .. }
                ),
                "{} produced a non-language error: {:?}",
                name,
                error
            );
        }
    }

    #[test]
    fn legal_neighbors_and_repair_are_admitted() {
        let sources = [
            include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/NB17.at"),
            include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/NB18.at"),
            include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/NB19.at"),
            include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B04.at"),
            include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B20.at"),
            r#"fn missing() int { return 42 }
fn main() int {
    if false { return missing() }
    return 42
}
"#,
            r#"fn owned() Str {
    let s Str = "ok"
    return s
}
fn main() int { return owned().len() }
"#,
            r#"type A { value int }
fn read(x A) int { return x.value }
fn main() int { return read(A(42)) }
"#,
            r#"type Item { name Str; code int }
fn main() int {
    let a Item = Item("x", 41)
    let b Item = a.move
    return b.code
}
"#,
        ];
        for source in sources {
            transpile_rust("semantic_admission", source)
                .unwrap_or_else(|error| panic!("legal source rejected: {}\n{}", error, source));
        }
    }

    #[test]
    fn branch_move_and_wrong_nominal_type_have_distinguishing_controls() {
        let invalid_branch = r#"type Item { code int }
fn main() int {
    let a Item = Item(41)
    if false { let b Item = a.move }
    return a.code
}
"#;
        assert!(matches!(rejects(invalid_branch), AutoError::Type(_)));
        let valid_branch = r#"type Item { code int }
fn main() int {
    let a Item = Item(41)
    if false {
        let b Item = a.move
        return b.code
    }
    return 0
}
"#;
        transpile_rust("semantic_admission", valid_branch).unwrap();
    }

    #[test]
    fn review_counterexamples_and_legal_scope_controls() {
        let invalid = [
            r#"fn bad() str {
    let s Str = "temporary"
    if false { return s.to(str) }
    return "ok"
}
fn main() int { return bad().len() }
"#,
            r#"fn bad() str {
    let s Str = "temporary"
    return if false { s.to(str) } else { "ok" }
}
fn main() int { return bad().len() }
"#,
            r#"type Item { code int }
fn main() int {
    let a Item = Item(41)
    let b Item = if true { a.move } else { a.move }
    return a.code + b.code
}
"#,
            r#"fn main() int {
    let x int = 1
    let y int = if true { x = 2 } else { 0 }
    return x + y
}
"#,
            r#"fn main() int {
    let f = () => missing()
    return 42
}
"#,
            r#"use.rs std::fs
fn main() int {
    if false { return missing() }
    return 42
}
"#,
            r#"fn main() int {
    let x int = 1
    try { x = 2 } catch { }
    return x
}
"#,
            r#"type Item { code int }
fn main() int {
    let a Item = Item(41)
    is 0 { else -> { let b Item = a.move } }
    return a.code
}
"#,
            r#"type Item { code int }
fn main() int {
    let a Item = Item(41)
    let f = () => a.move
    return a.code
}
"#,
        ];
        for source in invalid {
            let error = rejects(source);
            assert!(
                !matches!(error, AutoError::Msg(_) | AutoError::Io(_)),
                "non-language error: {:?}",
                error
            );
        }

        let legal = [
            r#"fn main() int {
    var x int = 1
    let y int = if true { x = 2 } else { 0 }
    return x + y
}
"#,
            r#"fn helper() int { return 42 }
fn main() int {
    let f = () => helper()
    return f()
}
"#,
            r#"use.rs std::fs
fn missing() int { return 42 }
fn main() int {
    if false { return missing() }
    return 42
}
"#,
            r#"type Item { code int }
fn main() int {
    let a Item = Item(41)
    let b Item = a.move
    if true {
        let a Item = Item(2)
        return a.code
    }
    return b.code
}
"#,
        ];
        for source in legal {
            transpile_rust("semantic_admission", source).unwrap_or_else(|error| {
                panic!("legal scope control rejected: {}\n{}", error, source)
            });
        }
    }

    #[test]
    fn project_entry_rejects_same_negatives_and_recovers_after_repair() {
        let root = std::env::var_os("AUTO_LANG_PLAN916_SCRATCH")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(format!("project-entry-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let sources = [
            (
                "B16",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B16.at"),
            ),
            (
                "B17",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B17.at"),
            ),
            (
                "B18",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B18.at"),
            ),
            (
                "B19",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B19.at"),
            ),
            (
                "T06",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/T06.at"),
            ),
            (
                "R09",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/R09.at"),
            ),
            (
                "R10",
                include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/R10.at"),
            ),
            (
                "nested-expr-borrow",
                r#"fn bad() str {
    let s Str = "temporary"
    return if false { s.to(str) } else { "ok" }
}
fn main() int { return bad().len() }
"#,
            ),
        ];
        for (name, source) in sources {
            let project = root.join(name);
            std::fs::create_dir_all(&project).unwrap();
            let entry = project.join("main.at");
            std::fs::write(&entry, source).unwrap();
            let error = match crate::trans::rust::transpile_rust_project(entry.to_str().unwrap()) {
                Ok(_) => panic!("{} emitted a project artifact", name),
                Err(error) => error,
            };
            assert!(
                !matches!(error, AutoError::Msg(_) | AutoError::Io(_)),
                "{} lost its typed language identity: {:?}",
                name,
                error
            );
        }

        let repair = root.join("repair");
        std::fs::create_dir_all(&repair).unwrap();
        let entry = repair.join("main.at");
        std::fs::write(
            &entry,
            include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B18.at"),
        )
        .unwrap();
        assert!(crate::trans::rust::transpile_rust_project(entry.to_str().unwrap()).is_err());
        std::fs::write(
            &entry,
            include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/NB18.at"),
        )
        .unwrap();
        let artifacts = crate::trans::rust::transpile_rust_project(entry.to_str().unwrap())
            .expect("repaired source did not emit a fresh project artifact");
        assert!(!artifacts.is_empty());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn four_state_emit_turnaround_has_thirty_samples_each() {
        let legal = include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/NB17.at");
        let invalid = include_str!("../../../../../crates/auto-lang/test/cases/ace-m1/src/B17.at");
        for state in ["fresh", "hit", "body-edit", "repair"] {
            let mut samples = Vec::with_capacity(30);
            for _ in 0..30 {
                let started = std::time::Instant::now();
                match state {
                    "fresh" | "hit" | "repair" => {
                        transpile_rust("semantic_admission", legal).unwrap();
                    }
                    _ => {
                        let error = rejects(invalid);
                        assert!(format!("{:?}", error).len() <= 4 * 1024 * 1024);
                    }
                }
                samples.push(started.elapsed().as_micros());
            }
            samples.sort_unstable();
            let p90 = samples[26];
            assert!(p90 <= 2_000_000, "{} emit P90={}us", state, p90);
            eprintln!("a2r916 state={} samples=30 p90={}us", state, p90);
        }
    }
}
