
fn main() {
    let p = ace_syntax_grammar::compile_profile(include_str!("../../../grammars/auto-source.at"), "entry-unit").unwrap();
    let cs = ace_syntax_runtime::lexer::CompiledScanner::compile(&p).unwrap();
    let store = std::rc::Rc::new(ace_syntax_runtime::snapshot::PageStore::new());
    let src = "let total = 0
total = total + 1
total + 41
";
    let snap = ace_syntax_runtime::snapshot::Snapshot::from_text(&store, src);
    let out = ace_syntax_runtime::parse::parse_snapshot(&p, &cs, &snap, &ace_syntax_runtime::program::Budget::default()).unwrap();
    println!("{}", out.tree.canonical());
    let rec = ace_syntax_runtime::cst::reconstruct(&out.tree, &snap).unwrap();
    let want = snap.read_bytes(0, snap.len()).unwrap();
    println!("RECON_OK={}", rec == want);
    for (i, (a, b)) in rec.iter().zip(want.iter()).enumerate() {
        if a != b { println!("first diff at {i}: rec={:?} want={:?}", &rec[i.saturating_sub(6)..i+6], &want[i.saturating_sub(6)..i+6]); break; }
    }
}
