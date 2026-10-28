//! PLAN-862 T-03a (V-01, frontend half): the entry matrix, the Unit result
//! type and the single-file script form through the REAL frontend
//! (`parse_program_with` → `resolve_with` → `typecheck_with`).
//!
//! Frozen contract: `docs/plans/attachments/862/work/entry-contract.md` §2.
//! This file exercises the ACCEPT/REJECT set only — Atom emission, HIR
//! binding, MIR lowering and AV execution for this profile belong to
//! T-03b/T-04 and are not wired yet (`compile_entry_unit_source` is gated).

use auto_ac_prototype::source::{
    compile_entry_unit_source, parse_program_with, resolve_with, typecheck_with, SourceProfile, VT,
};

fn front(src: &str) -> Result<auto_ac_prototype::source::ResolvedProgram, Vec<String>> {
    let program = parse_program_with(src, SourceProfile::EntryUnit)
        .map_err(|d| vec![format!("{}: {}", d.code, d.message)])?;
    let resolved = resolve_with(&program, &[], SourceProfile::EntryUnit)
        .map_err(|ds| ds.iter().map(|d| d.code.clone()).collect::<Vec<_>>())?;
    typecheck_with(&resolved, SourceProfile::EntryUnit)
        .map_err(|ds| ds.iter().map(|d| d.code.clone()).collect::<Vec<_>>())?;
    Ok(resolved)
}

fn resolve_codes(src: &str) -> Vec<String> {
    let program = match parse_program_with(src, SourceProfile::EntryUnit) {
        Ok(p) => p,
        Err(d) => return vec![d.code],
    };
    match resolve_with(&program, &[], SourceProfile::EntryUnit) {
        Ok(_) => Vec::new(),
        Err(ds) => ds.into_iter().map(|d| d.code).collect(),
    }
}

// -- the three legal forms ------------------------------------------------

#[test]
fn int_main_is_the_value_entry() {
    let p = front("fn main() int {\n    return 0\n}\n").expect("int main compiles");
    assert_eq!(p.fns.len(), 1);
    assert_eq!(p.fns[0].result, VT::I32);
    assert!(
        !p.fns[0].body.stmts.is_empty(),
        "the return statement resolved"
    );
}

#[test]
fn unit_main_is_the_unit_entry() {
    let p = front("fn main() void {\n    let a = 1\n}\n").expect("unit main compiles");
    assert_eq!(p.fns[0].result, VT::Unit);
}

#[test]
fn script_with_i32_tail_is_a_value_entry() {
    // a script is the top-level statement sequence; its TAIL is the result
    let p = front("fn helper() int {\n    return 7\n}\n\nlet a = 1\na + helper()\n")
        .expect("script compiles");
    assert_eq!(p.fns.len(), 2, "helper + the synthetic script main");
    let main = p
        .fns
        .iter()
        .find(|f| f.name == "main")
        .expect("script main");
    assert_eq!(main.result, VT::I32, "an i32 tail types the script as int");
    assert!(main.params.is_empty());
}

#[test]
fn script_without_tail_is_a_unit_entry() {
    let p = front("let a = 1\nlet b = a + 1\n").expect("script compiles");
    assert_eq!(p.fns.len(), 1);
    assert_eq!(p.fns[0].result, VT::Unit);
}

#[test]
fn void_helpers_are_legal_beside_an_int_entry() {
    let p = front("fn note() void {\n    let a = 1\n}\n\nfn main() int {\n    return 0\n}\n")
        .expect("void helper compiles");
    assert_eq!(p.fns.len(), 2);
    assert_eq!(
        p.fns.iter().find(|f| f.name == "note").unwrap().result,
        VT::Unit
    );
}

// -- uniqueness, form exclusivity, parameters -----------------------------

#[test]
fn missing_entry_is_rejected() {
    let got = resolve_codes("fn helper() int {\n    return 1\n}\n");
    assert_eq!(got, vec!["source.entry-missing".to_string()]);
}

#[test]
fn duplicate_main_is_rejected() {
    let got =
        resolve_codes("fn main() int {\n    return 0\n}\n\nfn main() int {\n    return 1\n}\n");
    assert!(
        got.contains(&"source.entry-duplicate".to_string()),
        "duplicate main must be named, got {got:?}"
    );
}

#[test]
fn declared_main_plus_script_is_ambiguous() {
    let got = resolve_codes("fn main() int {\n    return 0\n}\n\nlet a = 1\n");
    assert!(
        got.contains(&"source.entry-ambiguous".to_string()),
        "a script beside a declared main is ambiguous, got {got:?}"
    );
}

#[test]
fn entry_parameters_are_rejected() {
    let got = resolve_codes("fn main(x int) int {\n    return x\n}\n");
    assert!(
        got.contains(&"source.entry-params".to_string()),
        "the entry takes no parameters, got {got:?}"
    );
}

// -- return / tail rules --------------------------------------------------

#[test]
fn int_entry_tail_is_not_an_implicit_return() {
    let got = resolve_codes("fn main() int {\n    1\n}\n");
    assert!(
        got.contains(&"source.entry-tail".to_string()),
        "an int entry never gets a tail return, got {got:?}"
    );
}

#[test]
fn unit_entry_cannot_return_a_value() {
    let got = resolve_codes("fn main() void {\n    return 1\n}\n");
    assert!(
        got.contains(&"source.unit-return-value".to_string()),
        "a Unit entry has no value to return, got {got:?}"
    );
}

#[test]
fn unit_entry_return_inside_a_nested_block_is_rejected_too() {
    let got = resolve_codes("fn main() void {\n    if true {\n        return 1\n    }\n}\n");
    assert!(
        got.contains(&"source.unit-return-value".to_string()),
        "nested returns are the same rule, got {got:?}"
    );
}

#[test]
fn script_tail_must_be_i32() {
    let got = resolve_codes("true\n");
    assert!(
        got.contains(&"source.entry-tail-type".to_string()),
        "a bool tail is not an i32 result, got {got:?}"
    );
}

#[test]
fn a_non_trailing_expression_statement_is_rejected() {
    let got = resolve_codes("let a = 1\na + 1\na + 2\n");
    assert!(
        got.contains(&"source.entry-tail".to_string()),
        "only the trailing statement may be an expression, got {got:?}"
    );
}

#[test]
fn an_expression_statement_in_a_declared_function_is_rejected() {
    let got = resolve_codes("fn main() int {\n    1\n    return 0\n}\n");
    assert!(
        got.contains(&"source.entry-tail".to_string()),
        "expression statements are a script-tail form only, got {got:?}"
    );
}

#[test]
fn nested_expression_statement_is_never_the_tail() {
    let got = resolve_codes("if true {\n    1\n}\n3\n");
    assert!(
        got.contains(&"source.entry-tail".to_string()),
        "an expression inside an arm is not the script tail, got {got:?}"
    );
}

#[test]
fn top_level_return_is_rejected_by_the_parser() {
    let err = parse_program_with("return 1\n", SourceProfile::EntryUnit)
        .expect_err("a script has no top-level `return`");
    assert_eq!(err.code, "source.entry-top-return");
    assert!(
        err.message.contains("trailing expression"),
        "the rejection explains the script form: {}",
        err.message
    );
}

// -- profile isolation ----------------------------------------------------

#[test]
fn void_is_not_part_of_the_other_profiles() {
    // the scalar profile keeps rejecting the word (zero drift for旧域)
    let err = parse_program_with("fn main() void {\n}\n", SourceProfile::Scalar)
        .expect_err("scalar profile has no `void` result");
    assert_eq!(err.code, "source.unsupported");
}

#[test]
fn expression_statements_stay_rejected_on_the_scalar_surface() {
    let err = parse_program_with(
        "fn f() int {\n    g()\n    return 0\n}\n",
        SourceProfile::Scalar,
    )
    .expect_err("scalar profile has no expression statements");
    assert!(
        err.message
            .contains("expression statements are not part of"),
        "unexpected diagnostic: {} {}",
        err.code,
        err.message
    );
}

#[test]
fn entry_unit_profile_identity_and_domain_are_distinct() {
    assert_eq!(
        SourceProfile::EntryUnit.as_str(),
        "auto.source.entry-unit.draft"
    );
    assert_eq!(
        SourceProfile::EntryUnit.hir_domain(),
        auto_ac_prototype::hir::HirDomain::EntryUnit
    );
    assert_eq!(
        SourceProfile::of("auto.source.entry-unit.draft"),
        Some(SourceProfile::EntryUnit)
    );
    // the new profile is not the scalar profile under another name
    assert_ne!(SourceProfile::EntryUnit, SourceProfile::Scalar);
}

// -- the emit gate is explicit until T-03b --------------------------------

#[test]
fn emission_is_a_controlled_rejection_not_a_silent_fallback() {
    let err = compile_entry_unit_source(
        "fn main() int {\n    return 0\n}\n",
        "entry-unit-862.at",
        &[],
    )
    .expect_err("the entry-unit emitter is not landed yet");
    assert_eq!(err[0].code, "source.unsupported");
    assert!(
        err[0].message.contains("T-03b"),
        "the rejection names the remaining work: {}",
        err[0].message
    );
}
