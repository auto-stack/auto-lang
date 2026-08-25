//! Plan 442 C2 parser regression: method chains on use.rust-imported fn
//! calls in ARGUMENT position (the "Bug A" family).
//!
//! Root cause chain: `resolve_uses` registers every `use.rust` item via
//! `register_rust_type` — which inserts a synthetic TypeDecl — so an
//! imported FUNCTION like `to_string` looks like a TYPE to the parser.
//! `args()` routes Ident-led arguments through `node_or_call_expr`, whose
//! `is_constructor` heuristic then routed `get(h1).put(h2)` (musk backend's
//! axum route builders) into node-instance parsing: the `(h1)` was consumed
//! as node args and the chain dot dangled ("expected argument separator,
//! found Dot"). Fix: a paren-arg call followed by `.` is never a node
//! instance — nodes don't chain — so it falls through to the plain-call
//! path, which continues the chain.

#[cfg(test)]
mod plan442_parser_regress {
    /// `use.rust` fn import + chained call inside another call's args.
    /// Failed pre-fix with `UnexpectedToken { expected: "argument separator
    /// (comma, newline, or ))", found: "Dot" }`.
    #[test]
    fn chained_call_on_rust_import_in_args_parses() {
        let code = "use.rust serde_json::{to_string}\n\
                    \n\
                    fn wrap(s str, v int) int {\n    return v\n}\n\
                    \n\
                    fn main() {\n\
                    \x20   let x = wrap(\"a\", to_string(1).len())\n\
                    \x20   print(\"ok\")\n\
                    }\n";
        // Full pipeline-style setup: resolve_deps is NOT needed (builtin
        // crate), but resolve_uses IS — it performs the register_rust_type
        // side effect that made `to_string` look like a type.
        let mut session = crate::compile::CompileSession::new();
        let _ = session.resolve_uses(code);
        let mut parser = crate::Parser::new_with_type_store(code, session.type_store());
        let ast = parser
            .parse()
            .unwrap_or_else(|e| panic!("parse failed (Bug A regression): {e:?}"));
        // main must contain the wrap(...) Store; the chained arg survived.
        let main = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::Fn(f) if f.name.as_str() == "main" => Some(f),
                _ => None,
            })
            .expect("fn main");
        assert_eq!(main.body.stmts.len(), 2, "var + print must both survive");
    }

    /// Statement-position chain on the same import (rhs path) — guarded
    /// against regressions in the pratt continuation as well.
    #[test]
    fn chained_call_on_rust_import_in_stmt_parses() {
        let code = "use.rust serde_json::{to_string}\n\
                    \n\
                    fn main() {\n\
                    \x20   let x = to_string(1).len()\n\
                    \x20   print(\"ok\")\n\
                    }\n";
        let mut session = crate::compile::CompileSession::new();
        let _ = session.resolve_uses(code);
        let mut parser = crate::Parser::new_with_type_store(code, session.type_store());
        let ast = parser
            .parse()
            .unwrap_or_else(|e| panic!("parse failed (Bug A stmt variant): {e:?}"));
        let main = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::Fn(f) if f.name.as_str() == "main" => Some(f),
                _ => None,
            })
            .expect("fn main");
        assert_eq!(main.body.stmts.len(), 2);
    }
}
