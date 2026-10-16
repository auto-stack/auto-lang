use ac_syntax_rt::{snapshot::{Snapshot,PageStore},lexer::CompiledScanner,parse::parse_snapshot,program::Budget};
use ac_syntax_core_adapter::{CoreKinds,project_strict};
use std::rc::Rc;
fn main() {
    let args:Vec<String>=std::env::args().collect();let n:usize=args[1].parse().unwrap();let shape=args.get(2).cloned().unwrap_or("chain".into());
    std::thread::Builder::new().stack_size(2<<20).spawn(move||{
        let src=std::fs::read_to_string("D:/autostack/.wt/lang-787/auto-lang/experimental/ac-syntax/grammars/auto-core.at").unwrap();
        let p=ac_syntax_grammar::compile(&src).unwrap();let cs=CompiledScanner::compile(&p).unwrap();
        let text=if shape=="chain" {format!("fn f() int {{ return 1{} }}"," + 1".repeat(n))} else {format!("fn f() int {{\n{}return 1\n{}return 0\n}}\n","if true {\n".repeat(n),"}\n".repeat(n))};
        assert!(auto_ac_prototype::source::parse_program(&text).is_ok());
        let s=Snapshot::from_text(&Rc::new(PageStore::new()),&text);let out=parse_snapshot(&p,&cs,&s,&Budget::default()).unwrap();
        assert!(!out.tree.has_error_nodes());eprintln!("legacy accepted; CST parsed; entering project_strict shape={shape} depth={n} stack=2MiB");
        let r=project_strict(&p,&CoreKinds::from_program(&p),&out.tree,&s);assert!(r.is_ok(),"{:?}",r.err());
        drop(r);println!("projected and dropped");
    }).unwrap().join().unwrap();
}
