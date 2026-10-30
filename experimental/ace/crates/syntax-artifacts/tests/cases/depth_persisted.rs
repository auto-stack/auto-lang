//! V-12 深度·持久化编解码（PLAN-787 T-04/T-06，F787-R1-09 补遗）：≥4096 层
//! 真嵌套声明经 put_cst 分页落盘 → open_cst 按需取页重建子树——构建、查询、
//! 编解码、释放全路径显式栈迭代，无递归爆栈。

use ace_syntax_artifacts::SyntaxArtifactStore;
use ace_syntax_grammar::compile as compile_grammar;
use ace_syntax_runtime::lexer::CompiledScanner;
use ace_syntax_runtime::parse::parse_snapshot;
use ace_syntax_runtime::program::{Budget, GrammarProgram};
use ace_syntax_runtime::snapshot::{PageStore, Snapshot};
use std::rc::Rc;

fn core_grammar() -> GrammarProgram {
    let src = std::fs::read_to_string(format!(
        "{}/../../grammars/auto-core.at",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    compile_grammar(&src).unwrap()
}

const DEPTH: usize = 4096;

#[test]
fn depth_persisted_codec_roundtrip_no_stack_overflow() {
    let p = core_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    let text = format!(
        "fn f() int {{ return {}1{} }}",
        "(".repeat(DEPTH + 64),
        ")".repeat(DEPTH + 64)
    );
    let snap = Snapshot::from_text(&Rc::new(PageStore::new()), &text);
    let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();
    out.tree.assert_lossless(&snap).unwrap();

    let root = std::env::temp_dir().join(format!("ac787-depth-persist-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    {
        let sas = SyntaxArtifactStore::open(&root, "depth").unwrap();
        let tok = sas.put_tokens(&p, &snap, &out.tokens).unwrap();
        sas.put_cst(&p, &snap, &out.tree, &tok).unwrap();
    } // store 句柄"退出"

    let sas = SyntaxArtifactStore::open(&root, "depth").unwrap();
    let view = sas.open_cst(&p, &snap).unwrap().expect("manifest hit");
    assert_eq!(view.decl_count(), 1);
    let st = view.decl_subtree(0).unwrap();
    // 重建子树可遍历（canonical 迭代实现），深度足够。
    let c = st.canonical();
    assert!(
        c.lines().count() > DEPTH,
        "rebuilt deep tree must be traversable, got {} lines",
        c.lines().count()
    );
    // 源重建逐字节一致。
    let rebuilt = view.source_bytes().unwrap();
    assert_eq!(rebuilt, text.as_bytes());
    drop(view);
    drop(st);
    let _ = std::fs::remove_dir_all(&root);
}
