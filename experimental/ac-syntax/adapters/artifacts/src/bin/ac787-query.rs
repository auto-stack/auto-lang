//! PLAN-787 V-09 跨进程恢复探针（F787-R1-09/R2-09/R3-06）：只读 store——
//! grammar 程序从 store 反序列化（**不编译 .at 源**），CST 查询走 manifest/
//! 页路径（无 parse_snapshot 调用面）。源校验/命名**按范围取源**：
//! - 源完整性：源页流式 hash（零驻留）；
//! - outline 名：仅物化目标声明 span 覆盖的源页（≤512KiB 量级），子树
//!   origin 平移到切片坐标系——**无整源驻留、无整源 Snapshot**。
//! 用法：ac787-query <store-dir> <grammar-id> <grammar-revision> <stream-id>
//!       <total-len> <decl-idx>
//! 输出（stdout 末行，k=v 空白分隔）：
//! `QUERY decls=<d> nodes=<n> src_len=<s> src_hash=<h> span=<o>,<e> name=<m>`

use ac_syntax_artifact_adapter::SyntaxArtifactStore;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 7 {
        eprintln!(
            "usage: ac787-query <store-dir> <grammar-id> <grammar-revision> <stream-id> <total-len> <decl-idx>"
        );
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
    let prog = sas
        .get_grammar_by_id(grammar_id, grammar_rev)
        .expect("store read")
        .unwrap_or_else(|| panic!("grammar {grammar_id}@{grammar_rev} missing from store"));
    let view = sas
        .open_cst_stream(&prog, stream_id, total_len)
        .expect("open view")
        .unwrap_or_else(|| panic!("stream manifest {stream_id} missing"));
    let subtree = view.decl_subtree(decl_idx).expect("decl subtree");
    let (span_lo, span_hi) = view.decl_range(decl_idx).expect("decl range");
    // 源覆盖窗口 = 声明**单元坐标域** [origin, span_hi)（F787-R4-03）：声明
    // origin 覆盖前导 trivia，可位于语义 span 之前的源页——窗口从 origin
    // 起算，否则 origin 平移到切片坐标系时无符号下溢/切片缺字节。物化与
    // 窗口相交的源页（页粒度 ≤512KiB 级；按需上界不回退为整源驻留）。
    let origin = subtree.decls[0].origin;

    // 源完整性：流式 hash + 只物化窗口覆盖页（F787-R3-06 + R4-03）。
    let mut hasher = blake3::Hasher::new();
    let mut src_len: u64 = 0;
    let mut slice_start: u64 = u64::MAX;
    let mut window_pages: Vec<u8> = Vec::new();
    view.for_each_source_page(|page| {
        hasher.update(page);
        let page_lo = src_len;
        src_len += page.len() as u64;
        let page_hi = src_len;
        if page_hi > origin && page_lo < span_hi {
            if slice_start == u64::MAX {
                slice_start = page_lo;
            }
            window_pages.extend_from_slice(page);
        }
        Ok(())
    })
    .expect("stream source pages");
    let src_hash = hasher.finalize().to_hex().to_string();

    // outline：子树 origin 受校验平移到切片坐标系 + 仅窗口文本的 Snapshot。
    // 文本切片取 [origin, span_hi)（两端都是 token 边界——页边界可能切开多
    // 字节字符，切片前转整缓冲会误拒；切片坐标系以 origin 为零点，平移基准
    // = origin）。checked_sub——回归即受控退出（不下溢 panic、不饱和错位/
    // 吞错冒充成功）。
    let name = if slice_start != u64::MAX && window_pages.len() <= (2 << 20) {
        let shift = slice_start;
        let lo = (origin - shift) as usize;
        let hi = (span_hi - shift) as usize;
        let mut sub = subtree.clone();
        for d in &mut sub.decls {
            d.origin = d.origin.checked_sub(origin).unwrap_or_else(|| {
                eprintln!(
                    "query: decl origin {} precedes the slice start {origin}",
                    d.origin
                );
                std::process::exit(3);
            });
        }
        let text = std::str::from_utf8(&window_pages[lo..hi]).expect("utf8 source slice");
        let snap = ac_syntax_rt::snapshot::Snapshot::from_text(
            &std::rc::Rc::new(ac_syntax_rt::snapshot::PageStore::new()),
            text,
        );
        ac_syntax_rt::outline::outline(&prog, &sub, &snap)
            .ok()
            .and_then(|items| items.first().map(|i| i.name.clone()))
            .unwrap_or_default()
    } else {
        String::new()
    };
    println!(
        "QUERY decls={} nodes={} src_len={} src_hash={} span={},{} name={}",
        view.decl_count(),
        subtree.node_total(),
        src_len,
        src_hash,
        span_lo,
        span_hi,
        name
    );
}
