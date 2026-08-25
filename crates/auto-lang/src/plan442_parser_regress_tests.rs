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

    /// relay_store tail (the 30/32→32/32 blocker): TWO consecutive
    /// corpus-style `#` comment lines inside a `pub type` body. Pre-fix,
    /// parse_fn_annotations skipped exactly one `#` line and returned; the
    /// type-body loop then fell through to type_member with cur on the second
    /// `#`, eating it as a field name and cascading "Expected end of
    /// statement, got Ident<编译期拦截>" + 19 RBrace errors. One `#` line
    /// before a field worked by accident — the accident is what made
    /// minimal repros pass. Real-file bisection (probe
    /// musk_backend_relay_store_bisect) isolated the 14-line RunMetadata
    /// block; this test pins the parser-side fix without the corpus.
    #[test]
    fn consecutive_hash_comment_lines_in_type_body_parse() {
        let code = "pub type RunMetadata {\n\
                    \x20   title Option<str>\n\
                    \x20   # PLAN-032 手补 report Option<ReportMeta>(hw store 同款;.at 无法表达外部\n\
                    \x20   # 类型,重生成后需手工补回——parity 编译期拦截)\n\
                    \x20   task_plan_id Option<str>\n\
                    \x20   phase_index Option<uint>\n\
                    }\n\
                    \n\
                    fn main() {\n\
                    \x20   print(\"ok\")\n\
                    }\n";
        let mut parser = crate::Parser::new(code);
        let ast = parser
            .parse()
            .unwrap_or_else(|e| panic!("parse failed (relay_store # comment run): {e:?}"));
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::TypeDecl(d) if d.name.as_str() == "RunMetadata" => Some(d),
                _ => None,
            })
            .expect("RunMetadata decl");
        assert_eq!(
            decl.members.len(),
            3,
            "title/task_plan_id/phase_index must parse; # lines skipped"
        );
        assert_eq!(decl.members[0].name.as_str(), "title");
        assert_eq!(decl.members[1].name.as_str(), "task_plan_id");
        assert_eq!(decl.members[2].name.as_str(), "phase_index");
    }

    /// Same shape inside an `ext` body — that loop shares the
    /// parse_fn_annotations fall-through, so a comment run before a method
    /// must not leak into the method parse.
    #[test]
    fn hash_comment_run_in_ext_body_parses() {
        let code = "pub type Box2 {\n\
                    \x20   v int\n\
                    }\n\
                    \n\
                    ext Box2 {\n\
                    \x20   # 笔记一——散文注释\n\
                    \x20   # 笔记二,含逗号与括号(注)\n\
                    \x20   pub fn get() int {\n\
                    \x20       return self.v\n\
                    \x20   }\n\
                    }\n\
                    \n\
                    fn main() {\n\
                    \x20   print(\"ok\")\n\
                    }\n";
        let mut parser = crate::Parser::new(code);
        let ast = parser
            .parse()
            .unwrap_or_else(|e| panic!("parse failed (ext # comment run): {e:?}"));
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::TypeDecl(d) if d.name.as_str() == "Box2" => Some(d),
                _ => None,
            })
            .expect("Box2 decl with merged ext");
        assert_eq!(decl.methods.len(), 1, "the one ext method must survive");
        assert_eq!(decl.methods[0].name.as_str(), "get");
    }
}
