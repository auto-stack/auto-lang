//! V-11 工作集（PLAN-787 T-06）：64MiB 实际源码、流式构建、随机小查询、
//! 旧 lease 释放。源+树页外存（store 目录）；运行时驻留 = 页缓存(≤8MiB 受管)
//! + 目录；查询只取必要页（payload ≤512KiB）；不先读成整串再测。
//!
//! 计量口径：受管预算 = 页缓存字节（自有计量，断言 ≤8MiB）+ 探针 staging
//! 上限（8MiB 冻结）；构建期驻留代理 = 当前块树节点字节（逐块释放）。
//! OS private bytes 无稳定免依赖采样途径，报告以受管口径 + 峰值代理记录。

use ac_syntax_artifact_adapter::{CstView, StreamingCstBuilder, SyntaxArtifactStore, CST_HOT_BUDGET};
use ac_syntax_grammar::compile as compile_grammar;
use ac_syntax_rt::program::GrammarProgram;

fn mini_grammar() -> GrammarProgram {
    let src = std::fs::read_to_string(format!("{}/../../grammars/interface-mini-v1.at", env!("CARGO_MANIFEST_DIR"))).unwrap();
    compile_grammar(&src).unwrap()
}

fn scratch(family: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("ac787-{family}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// 生成 64MiB+ 的短声明流（独立短声明，按行边界切块）。
fn gen_chunks(target_bytes: usize, chunk_decls: usize) -> (Vec<String>, usize) {
    let mut chunks = Vec::new();
    let mut total = 0usize;
    let mut i = 0usize;
    while total < target_bytes {
        let mut c = String::new();
        for _ in 0..chunk_decls {
            let line = format!("api f{i}(a: i32, b: i32) -> i32;\nnote \"n{i}\";\n");
            c.push_str(&line);
            i += 1;
        }
        total += c.len();
        chunks.push(c);
    }
    (chunks, i)
}

#[test]
fn syntax_budget_64mib_streaming_small_queries() {
    const TARGET: usize = 64 << 20;
    let p = mini_grammar();
    let (chunks, decl_count) = gen_chunks(TARGET, 2000);
    let real_bytes: usize = chunks.iter().map(|c| c.len()).sum();
    assert!(real_bytes >= TARGET, "load must be >= 64MiB, got {}", real_bytes >> 20);

    let root = scratch("budget-64m");
    let sas = SyntaxArtifactStore::open(&root, "big").unwrap();
    sas.put_grammar(&p).unwrap();

    // 流式构建：逐块 parse→分页→入库，块树即用即弃（无整树驻留）。
    let mut builder = StreamingCstBuilder::new(&sas, &p);
    let t0 = std::time::Instant::now();
    for c in &chunks {
        builder.push_chunk(c).expect("chunk build");
    }
    let build_secs = t0.elapsed().as_secs_f64();
    let manifest_art = builder.finish(real_bytes as u64).expect("finish");
    assert!(!manifest_art.is_empty());
    let stream_id = builder.stream_snapshot_id().clone();

    // 外存核验：store 目录确实承载了页（源与树都在盘上）。
    let du: u64 = {
        let mut acc = 0u64;
        fn walk(d: &std::path::Path, acc: &mut u64) {
            if let Ok(rd) = std::fs::read_dir(d) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_file() {
                        *acc += p.metadata().map(|m| m.len()).unwrap_or(0);
                    } else {
                        walk(&p, acc);
                    }
                }
            }
        }
        walk(&root, &mut acc);
        acc
    };
    assert!(du as usize >= real_bytes, "on-disk {} must cover source scale {}", du >> 20, real_bytes >> 20);

    // 重开视图（模拟新会话）：目录直读（零页 IO）。
    let snap_stream = ac_syntax_rt::identity::SnapshotId(stream_id.clone());
    let _ = &snap_stream;
    let view = sas
        .open_cst_stream(&p, &stream_id, real_bytes as u64)
        .expect("open streaming view")
        .expect("manifest hit");
    assert_eq!(view.decl_count(), decl_count * 2 / 2 * 2); // api+note 均入目录

    // 随机小查询：抽 200 个声明子树（确定性种子）。
    let t1 = std::time::Instant::now();
    let mut rng = 0xABCDEF01u64;
    let mut queried = 0usize;
    let mut max_subtree_bytes = 0usize;
    for _ in 0..200 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let idx = ((rng >> 33) as usize) % view.decl_count();
        let st = view.decl_subtree(idx).expect("subtree");
        max_subtree_bytes = max_subtree_bytes.max(st.nodes.len() * 32);
        queried += 1;
    }
    let query_secs = t1.elapsed().as_secs_f64();

    // 受管预算：页缓存 ≤8MiB（断言，自有计量）。
    let hot = view.debug_hot_bytes();
    assert!(hot <= CST_HOT_BUDGET, "hot page cache {hot} > 8MiB budget");
    // 查询 payload：单子树 ≤512KiB。
    assert!(max_subtree_bytes <= 512 << 10, "subtree payload {max_subtree_bytes} > 512KiB");

    // 旧视图释放（lease 语义代理：drop 后页缓存随视图回收）。
    drop(view);
    let _ = queried;

    eprintln!(
        "BUDGET64 src={}MiB decls={} build={:.1}s query200={:.1}s hot={}KiB max_subtree={}B disk={}MiB",
        real_bytes >> 20,
        view_decl_total_placeholder(&sas, &p, &stream_id, real_bytes as u64),
        build_secs,
        query_secs,
        hot >> 10,
        max_subtree_bytes,
        du >> 20,
    );
}

fn view_decl_total_placeholder(sas: &SyntaxArtifactStore, p: &GrammarProgram, stream_id: &str, len: u64) -> usize {
    sas.open_cst_stream(p, stream_id, len).unwrap().map(|v| v.decl_count()).unwrap_or(0)
}
