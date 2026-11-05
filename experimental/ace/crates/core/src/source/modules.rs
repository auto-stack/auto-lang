//! PLAN-950 finite module syntax. This layer records source facts only;
//! visibility admission and Checked HIR remain separate authorities.
use super::ast::Program;
use super::cst::{parse_module_front, FrontBudget, INSTANCES_CAPABILITY};
use crate::atom_text::{Diagnostic, Span, Stage};
use std::collections::BTreeSet;

pub const MODULES_CAPABILITY: &str = "ac.source.modules.scalar/v1";
pub const MODULE_BINDING_VERSION: &str = "source-module-binding/1";

#[derive(Clone, Debug)]
pub struct Import {
    pub module: String,
    pub name: String,
    pub span: Span,
}

/// An independent source unit, including the visibility/import facts that
/// cannot be represented by a flattened function-only Program.
#[derive(Clone, Debug)]
pub struct ModuleSyntax {
    pub program: Program,
    pub exports: BTreeSet<String>,
    pub imports: Vec<Import>,
    pub grammar_id: String,
}

/// The explicit module entry. Old scalar entrypoints never discard metadata
/// from this result or silently widen their language surface.
pub fn parse_module_source(
    src: &str,
    capabilities: &[String],
    budget: FrontBudget,
) -> Result<ModuleSyntax, Vec<Diagnostic>> {
    let reason = if !capabilities.iter().any(|c| c == MODULES_CAPABILITY) {
        Some("finite modules require explicit ac.source.modules.scalar/v1")
    } else if capabilities.iter().any(|c| c == INSTANCES_CAPABILITY) {
        Some("module templates are outside the finite scalar module capability")
    } else {
        None
    };
    if let Some(reason) = reason {
        return Err(vec![Diagnostic::new(
            Stage::Text,
            "source.module-capability",
            reason,
            Span { start: 0, end: 0 },
        )]);
    }
    parse_module_front(src, budget)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{
        ast::{ExprKind, Item, Stmt},
        parse_program_with, SourceProfile,
    };
    fn parse(src: &str) -> Result<ModuleSyntax, Vec<Diagnostic>> {
        parse_module_source(
            src,
            &[MODULES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
        )
    }
    #[test]
    fn module_syntax_preserves_visibility_imports_and_qualified_spans() {
        let src = "use std.answer\npub fn main() int { return std.answer() }";
        let m = parse(src).unwrap();
        assert_eq!(m.exports, BTreeSet::from(["main".into()]));
        assert_eq!(m.imports.len(), 1);
        assert_eq!(m.imports[0].module, "std");
        assert_eq!(m.imports[0].name, "answer");
        let Item::Fn(f) = &m.program.items[0] else {
            panic!("function")
        };
        let Stmt::Return { value, .. } = &f.body.stmts[0] else {
            panic!("return")
        };
        let ExprKind::Call { callee, .. } = &value.kind else {
            panic!("call: {:?}", value.kind)
        };
        assert_eq!(callee.name, "std.answer");
        assert_eq!(&src[callee.span.start..callee.span.end], "std.answer");
    }
    #[test]
    fn module_syntax_keeps_old_scalar_gate_and_explicit_capability() {
        for src in [
            "pub fn f() int { return 42 }",
            "use std.answer\nfn f() int { return answer() }",
            "fn f() int { return std.answer() }",
        ] {
            assert!(parse(src).is_ok(), "{src}: {:?}", parse(src));
            assert!(parse_program_with(src, SourceProfile::Scalar).is_err());
            assert_eq!(
                parse_module_source(src, &[], FrontBudget::WORK_OWNED_64MIB).unwrap_err()[0].code,
                "source.module-capability"
            );
        }
        assert_eq!(
            parse("fn f() int { return 1 }\nuse std.answer").unwrap_err()[0].code,
            "source.module-import-order"
        );
        assert_eq!(
            parse("use a.f\nuse b.f\nfn main() int { return f() }").unwrap_err()[0].code,
            "source.module-duplicate-import"
        );
    }
    #[test]
    fn module_syntax_rejects_unsupported_forms_and_preserves_private_functions() {
        let m = parse("pub fn visible() int { return hidden() }\nfn hidden() int { return 42 }")
            .unwrap();
        assert_eq!(m.program.items.len(), 2);
        assert_eq!(m.exports, BTreeSet::from(["visible".into()]));
        for src in [
            "use std.*\nfn main() int { return 1 }",
            "use std.answer as f\nfn main() int { return f() }",
            "pub type X { x int }",
            "pub fn f<T>(x T) T { return x }",
        ] {
            assert!(parse(src).is_err(), "unexpected accepted: {src}");
        }
        assert!(parse_module_source(
            "fn f() int { return 1 }",
            &[MODULES_CAPABILITY.into(), INSTANCES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB
        )
        .is_err());
    }
}

mod binding;
pub use binding::{bind_modules, BoundModules, ModuleDiagnostic, ModuleInput, SourceSegment};
