//! PLAN-787 V-09 跨进程恢复探针（F787-R1-09/R2-09/R3-06/R5-02）：只读 store——
//! grammar 程序从 store 反序列化（**不编译 .at 源**），CST 查询走 manifest/
//! 页路径（无 parse_snapshot 调用面）。源校验/命名**按范围取源**：
//! - 源完整性：源页流式 hash（零驻留）；
//! - outline 名：仅物化 name 字段绝对范围覆盖的源页（F787-R5-02：不把
//!   前导 trivia 算进窗口——合法多 MiB 空白不得逼出静默空名），按真实
//!   token 范围读取——**无整源驻留、无坐标饱和、无空 outline 冒充成功**。
//! - 窗口驻留超限 → 受控 ResourceExhausted（exit 4）。
//! 用法：ac787-query <store-dir> <grammar-id> <grammar-revision> <stream-id>
//!       <total-len> <decl-idx>
//! 输出（stdout 末行，k=v 空白分隔）：
//! `QUERY decls=<d> nodes=<n> src_len=<s> src_hash=<h> span=<o>,<e> name=<m>`

use ac_syntax_artifact_adapter::SyntaxArtifactStore;
use ac_syntax_rt::cst::CstTree;
use ac_syntax_rt::program::GrammarProgram;

/// 在声明子树内找 outline name 字段的绝对字节范围（与 outline::outline 同
/// 搜索序：先序第一个命中字段的孩子）。
fn name_field_range(p: &GrammarProgram, tree: &CstTree, decl_node: u32) -> Option<(u64, u64)> {
    let spec = p.outline.as_ref()?;
    let name_field = p.field_id(&spec.name_field)?;
    let mut stack = vec![decl_node];
    while let Some(n) = stack.pop() {
        if let Some(c) = tree.field_child(n, name_field) {
            let (s, e) = tree.abs_range(c);
            if e > s {
                return Some((s, e));
            }
        }
        let kids: Vec<u32> = tree.children(n).to_vec();
        for &c in kids.iter().rev() {
            stack.push(c);
        }
    }
    None
}

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

    // name 字段绝对范围——真正需要的 token/源范围（F787-R5-02）。前导 trivia
    // 不在 name 字段内，不得物化进查询窗口。
    let name_span = name_field_range(&prog, &subtree, subtree.root);

    // 窗口驻留上限（默认 2MiB；AC787_QUERY_WINDOW_LIMIT 可覆盖供阈值两侧
    // 永久 oracle）。F787-R6-02：限额在 **append 之前**判定——窗口已超限
    // （允许单源页粒度超前：store 源页 ≤256KiB）即受控 ResourceExhausted
    // （exit 4），不为超限名称继续大规模分配；循环后兜底覆盖"越限页恰为
    // 最后一个相交页"的情形。
    let query_window_limit: usize = std::env::var("AC787_QUERY_WINDOW_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2 << 20);
    fn window_reject(materialized: usize, limit: usize, lo: u64, hi: u64) -> ! {
        eprintln!(
            "query: name-field window {} bytes exceeds limit {} (span {},{}); ResourceExhausted",
            materialized, limit, lo, hi
        );
        std::process::exit(4);
    }

    // 源完整性：流式 hash（零驻留）+ 仅物化 name 字段覆盖页（有界）。
    // name 为空（无 outline 规格/字段）时窗口为空——精确结果，不是截断。
    let (need_lo, need_hi) = match name_span {
        Some((s, e)) => (s, e),
        None => (u64::MAX, u64::MAX),
    };
    let mut hasher = blake3::Hasher::new();
    let mut src_len: u64 = 0;
    let mut slice_start: u64 = u64::MAX;
    let mut window_pages: Vec<u8> = Vec::new();
    view.for_each_source_page(|page| {
        hasher.update(page);
        let page_lo = src_len;
        src_len += page.len() as u64;
        let page_hi = src_len;
        if need_lo != u64::MAX && page_hi > need_lo && page_lo < need_hi {
            if slice_start == u64::MAX {
                slice_start = page_lo;
            }
            // R6-02：append 前判限额——已越限即拒绝，驻留上界 = limit + 一页。
            if window_pages.len() > query_window_limit {
                window_reject(window_pages.len(), query_window_limit, need_lo, need_hi);
            }
            window_pages.extend_from_slice(page);
        }
        Ok(())
    })
    .expect("stream source pages");
    let src_hash = hasher.finalize().to_hex().to_string();
    if window_pages.len() > query_window_limit {
        window_reject(window_pages.len(), query_window_limit, need_lo, need_hi);
    }

    let name = match (name_span, slice_start) {
        (Some((s, e)), base) if base != u64::MAX => {
            let lo = (s - base) as usize;
            let hi = (e - base) as usize;
            if hi > window_pages.len() {
                eprintln!(
                    "query: name field [{s},{e}) not covered by materialized window (base {base}, len {})",
                    window_pages.len()
                );
                std::process::exit(3);
            }
            std::str::from_utf8(&window_pages[lo..hi])
                .expect("utf8 name slice")
                .to_string()
        }
        // 无 name 字段 / 字段为空：精确空名，不是被 guard 截断。
        _ => String::new(),
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
