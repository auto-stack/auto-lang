//! PLAN-787 V-09 跨进程恢复探针（F787-R1-09：真新进程 + parser 禁调）：
//! 只读 store——grammar 程序从 store 反序列化（**不编译 .at 源**），CST
//! 查询走 manifest/页路径（无 parse_snapshot 调用），源文本从源页重建。
//! 用法：ac787-query <store-dir> <grammar-id> <grammar-revision> <stream-id>
//!       <total-len> <decl-idx>
//! 输出（stdout 末行）：`QUERY decls=<d> nodes=<n> src_len=<s> src_head=<h>`

use ac_syntax_artifact_adapter::SyntaxArtifactStore;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 7 {
        eprintln!("usage: ac787-query <store-dir> <grammar-id> <grammar-revision> <stream-id> <total-len> <decl-idx>");
        std::process::exit(2);
    }
    let store_dir = &args[1];
    let grammar_id = &args[2];
    let grammar_rev: u64 = args[3].parse().expect("grammar-revision");
    let stream_id = &args[4];
    let total_len: u64 = args[5].parse().expect("total-len");
    let decl_idx: usize = args[6].parse().expect("decl-idx");

    let sas = SyntaxArtifactStore::open(std::path::Path::new(store_dir), "query-proc")
        .expect("open store");
    // grammar 只从 store 装载（canonical 反序列化 + 身份重验）——不触碰 .at。
    let prog = sas
        .get_grammar_by_id(grammar_id, grammar_rev)
        .expect("store read")
        .unwrap_or_else(|| panic!("grammar {grammar_id}@{grammar_rev} missing from store"));
    let view = sas
        .open_cst_stream(&prog, stream_id, total_len)
        .expect("open view")
        .unwrap_or_else(|| panic!("stream manifest {stream_id} missing"));
    let subtree = view.decl_subtree(decl_idx).expect("decl subtree");
    let src = view.source_bytes().expect("rebuild source");
    let head = src
        .iter()
        .take(16)
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    println!(
        "QUERY decls={} nodes={} src_len={} src_head={}",
        view.decl_count(),
        subtree.node_total(),
        src.len(),
        head
    );
}
