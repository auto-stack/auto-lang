//! 临时调试：转储 auto-source CST（PLAN-927 开发期；完成后删除）。
use ace_syntax_grammar::compile_profile;
use ace_syntax_runtime::cst::{FLAG_ERROR, FLAG_MISSING, FLAG_TRIVIA};
use ace_syntax_runtime::lexer::CompiledScanner;
use ace_syntax_runtime::parse::parse_snapshot;
use ace_syntax_runtime::program::Budget;
use ace_syntax_runtime::snapshot::{PageStore, Snapshot};

const SOURCE: &str = include_str!("../../../../grammars/auto-source.at");

#[test]
fn dump_scalar_b01() {
    let src = std::env::var("DUMP_SRC").unwrap_or_else(|_| "fn main() int { return 2 + 3 * 4 }".to_string());
    let prof = std::env::var("DUMP_PROFILE").unwrap_or_else(|_| "scalar".to_string());
    let p = compile_profile(SOURCE, &prof).expect("grammar");
    let cs = CompiledScanner::compile(&p).expect("scanner");
    let store = std::rc::Rc::new(PageStore::new());
    let snap = Snapshot::from_text(&store, &src);
    let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).expect("parse");
    let tree = &out.tree;
    eprintln!("diags: {:?}", tree.diags);
    let mut stack = vec![(tree.root, 0usize)];
    while let Some((n, depth)) = stack.pop() {
        let nd = tree.node(n);
        let (s, e) = tree.abs_range(n);
        let name = ace_syntax_runtime::cst::kind_name(&p, nd.kind);
        let fl = format!(
            "{}{}{}",
            if nd.flags & FLAG_ERROR != 0 { "E" } else { "" },
            if nd.flags & FLAG_MISSING != 0 { "M" } else { "" },
            if nd.flags & FLAG_TRIVIA != 0 { "t" } else { "" }
        );
        if nd.flags & FLAG_TRIVIA == 0 {
            let text = if nd.child_count == 0 { snap.read_str(s, e).unwrap_or_default() } else { String::new() };
            eprintln!("{}{} [{}..{}] {} {:?}", "  ".repeat(depth), name, s, e, fl, text);
        }
        let kids = tree.children(n);
        for c in kids.into_iter().rev() {
            stack.push((c, depth + 1));
        }
    }
}
