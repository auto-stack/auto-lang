#[test]
fn rust_midexpr_eof_diags() {
    let src = include_str!("../../../grammars/auto-source.at");
    let p = ace_syntax_grammar::compile_profile(src, "scalar").unwrap();
    let cs = ace_syntax_runtime::lexer::CompiledScanner::compile(&p).unwrap();
    let pages = std::rc::Rc::new(ace_syntax_runtime::snapshot::PageStore::new());
    for sample in [
        "fn b(y int) int { y + ",
        "fn b(y int) int { return y + ",
        "fn a(x int) int { return x }\nfn b(y int) int { return y + ",
    ] {
        let snap = ace_syntax_runtime::snapshot::Snapshot::from_text(&pages, sample);
        let out = ace_syntax_runtime::parse::parse_snapshot(
            &p,
            &cs,
            &snap,
            &ace_syntax_runtime::program::Budget::default(),
        )
        .unwrap();
        println!("SAMPLE {sample:?}");
        for d in &out.tree.diags {
            println!("  DIAG {}..{} {}", d.start, d.end, d.message);
        }
        println!("  decls={}", out.tree.decls.len());
    }
}
