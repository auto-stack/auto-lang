//! V-11 工作集（PLAN-787 T-06，F787-R1-08 修复轮）：64MiB 实际源码、
//! **生成器独立进程**（`ac787-gen` bin，峰值分列）、runtime 从文件流式读取
//! （不整源驻留）、声明目录**在线落盘**（builder 不累积百万级目录）、随机
//! 小查询、旧 lease 释放。源+树页外存（store 目录）；运行时驻留 =
//! 页缓存(≤8MiB 受管) + manifest 目录（计入 16MiB 受管总预算）；查询只取
//! 必要页（payload ≤512KiB）。
//!
//! 计量口径（受管 = 自有计量）：
//! - builder 工作集峰值（目录缓冲/页表/源页表）≤ 8MiB；
//! - 视图受管 = hot 页缓存 + manifest 估算 ≤ 16MiB；
//! - OS private bytes：Windows 实测（探针 sysinfo），生成器与 runtime 分列，
//!   runtime 增量目标 ≤64MiB；非 Windows 宿主 sysinfo 返回 0——显式记录
//!   证据缺口（不作断言、不冒充通过）。

use ac_syntax_artifact_adapter::{StreamingCstBuilder, SyntaxArtifactStore, CST_HOT_BUDGET};
use ac_syntax_grammar::compile as compile_grammar;
use ac_syntax_rt::program::{Budget, GrammarProgram, RtError};

fn mini_grammar() -> GrammarProgram {
    let src = std::fs::read_to_string(format!(
        "{}/../../grammars/interface-mini-v1.at",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    compile_grammar(&src).unwrap()
}

fn scratch(family: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("ac787-{family}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn os_private() -> u64 {
    ac_artifact_probe::sysinfo::current_process_memory().private_bytes
}

/// 子进程受截止执行（F787-R2-10：复用探针 run_deadlined——裸 output 无截止）。
fn run_deadlined(cmd: std::process::Command, secs: u64) -> ac_artifact_probe::subproc::RunResult {
    let r = ac_artifact_probe::subproc::run_deadlined(cmd, std::time::Duration::from_secs(secs))
        .expect("spawn child process");
    assert!(!r.timed_out, "generator timed out (deadline {secs}s)");
    assert!(
        r.code == Some(0),
        "generator exited {:?}: {}",
        r.code,
        r.stderr
    );
    r
}

#[test]
fn syntax_budget_64mib_streaming_small_queries() {
    const TARGET_MIB: usize = 64;
    const OS_EVIDENCE: bool = cfg!(windows);
    let p = mini_grammar();

    let root = scratch("budget-64m");
    std::fs::create_dir_all(&root).unwrap();
    let src_file = root.join("source.txt");

    // 1) 生成器独立进程（生成器与 runtime 分列）：流式写文件 + 自报峰值。
    let gen_cmd = {
        let mut c = std::process::Command::new(env!("CARGO_BIN_EXE_ac787-gen"));
        c.arg(&src_file).arg(TARGET_MIB.to_string());
        c
    };
    let gen = run_deadlined(gen_cmd, 600);
    let gen_out = gen.stdout.clone();
    let gen_peak: u64 = gen_out
        .lines()
        .last()
        .and_then(|l| l.split("peak_private_bytes=").nth(1))
        .and_then(|v| v.parse().ok())
        .expect("generator peak report");

    let real_bytes = std::fs::metadata(&src_file).unwrap().len() as usize;
    assert!(
        real_bytes >= TARGET_MIB << 20,
        "load must be >= 64MiB, got {}",
        real_bytes >> 20
    );

    // 2) runtime：从文件流式读取（行边界缓冲 ~64KiB，不整源驻留）。
    let sas = SyntaxArtifactStore::open(&root, "big").unwrap();
    sas.put_grammar(&p).unwrap();
    let rt_base = os_private();
    let mut rt_peak = rt_base;
    let mut builder = StreamingCstBuilder::new(&sas, &p);
    let t0 = std::time::Instant::now();
    {
        use std::io::BufRead;
        let f = std::fs::File::open(&src_file).unwrap();
        let mut reader = std::io::BufReader::new(f);
        let mut chunk = String::with_capacity(1 << 16);
        let mut line = String::new();
        let mut chunks = 0usize;
        loop {
            line.clear();
            if reader.read_line(&mut line).unwrap() == 0 {
                break;
            }
            chunk.push_str(&line);
            if chunk.len() >= (1 << 16) {
                builder.push_chunk(&chunk).expect("chunk build");
                chunk.clear();
                chunks += 1;
                if chunks % 16 == 0 {
                    rt_peak = rt_peak.max(os_private());
                }
            }
        }
        if !chunk.is_empty() {
            builder.push_chunk(&chunk).expect("tail chunk build");
        }
    }
    rt_peak = rt_peak.max(os_private());
    rt_peak = rt_peak.max(os_private());
    let manifest_art = builder.finish(real_bytes as u64).expect("finish");
    // 受管峰值在 finish **之后**读取（F787-R3-06：finish 的 manifest JSON/
    // edges 瞬态也计入；此前在 finish 前读取会漏掉）。
    let managed_peak = builder.working_set_peak();
    rt_peak = rt_peak.max(os_private());
    assert!(!manifest_art.is_empty());
    let stream_id = builder.stream_snapshot_id();
    let build_secs = t0.elapsed().as_secs_f64();
    rt_peak = rt_peak.max(os_private());
    let decl_count = builder.decl_count() as usize;

    // builder 受管工作集（常驻目录 + parse 树/页 payload/目录页 JSON 等暂存，
    // F787-R2-09：在暂存量驻留时采样，不只统计常驻容器）≤ 8MiB。
    assert!(
        managed_peak <= 8 << 20,
        "builder managed working set (incl. transients) {managed_peak} > 8MiB"
    );

    // 外存核验：store 目录确实承载了源页与树页。
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
    assert!(
        du as usize >= real_bytes,
        "on-disk {} must cover source scale {}",
        du >> 20,
        real_bytes >> 20
    );

    // 3) 重开视图（新会话语义）：目录直读（零页 IO）+ 随机小查询。
    let view = sas
        .open_cst_stream(&p, &stream_id, real_bytes as u64)
        .expect("open streaming view")
        .expect("manifest hit");
    assert_eq!(view.decl_count(), decl_count, "api+note decls all indexed");
    let t1 = std::time::Instant::now();
    let mut rng = 0xABCDEF01u64;
    let mut max_subtree_bytes = 0usize;
    for _ in 0..200 {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let idx = ((rng >> 33) as usize) % view.decl_count();
        let st = view.decl_subtree(idx).expect("subtree");
        max_subtree_bytes = max_subtree_bytes.max(st.node_total() as usize * 32);
    }
    let max_page_payload = view.debug_max_page_payload();
    let query_secs = t1.elapsed().as_secs_f64();

    // 单查询读取界（F787-R3-06）：manifest 一次性入口成本之外，单次小查询
    // 的增量读取（目录页+数据页）≤512KiB；累计口径含 manifest。
    {
        let q_view = sas
            .open_cst_stream(&p, &stream_id, real_bytes as u64)
            .expect("open fresh recovery view")
            .expect("manifest hit");
        let idx0 = q_view.decl_count() / 3;
        let _st = q_view.decl_subtree(idx0).expect("recovery query");
        let total = q_view.debug_total_chunk_read();
        let manifest_read = q_view.debug_manifest_read();
        assert!(
            total - manifest_read <= 512 << 10,
            "single recovery query incremental reads {} > 512KiB (manifest {})",
            total - manifest_read,
            manifest_read
        );
        drop(q_view);
        rt_peak = rt_peak.max(os_private());
    }

    // 受管预算：hot 页缓存 ≤8MiB；hot + manifest 目录 ≤16MiB（索引计入）。
    let hot = view.debug_hot_bytes();
    let manifest_bytes = view.manifest_bytes();
    assert!(hot <= CST_HOT_BUDGET, "hot page cache {hot} > 8MiB budget");
    assert!(
        hot + manifest_bytes <= 16 << 20,
        "managed total (hot {hot} + manifest {manifest_bytes}) > 16MiB"
    );
    // 查询 payload：单子树 ≤512KiB；单页读取 ≤512KiB。
    assert!(
        max_subtree_bytes <= 512 << 10,
        "subtree payload {max_subtree_bytes} > 512KiB"
    );
    assert!(
        max_page_payload <= 512 << 10,
        "single page payload {max_page_payload} > 512KiB"
    );

    // 4) 源重建（流式逐页比对，零整源驻留）：与生成文件逐字节一致。
    {
        use std::io::Read;
        let mut f = std::fs::File::open(&src_file).unwrap();
        let mut expect: Vec<u8> = Vec::new();
        let mut mismatch: Option<String> = None;
        let mut consumed = 0u64;
        let r = view.for_each_source_page(|page| {
            expect.clear();
            expect.resize(page.len(), 0);
            f.read_exact(&mut expect)
                .expect("source file shorter than store pages");
            if expect != page {
                mismatch = Some(format!("source page mismatch at offset {consumed}"));
                return Err(ac_syntax_artifact_adapter::AdapterError::Admissions(
                    "mismatch".into(),
                ));
            }
            consumed += page.len() as u64;
            Ok(())
        });
        if let Some(m) = mismatch {
            panic!("{m}");
        }
        r.expect("source rebuild compare");
        // 文件无多余字节。
        let mut trailer = [0u8; 1];
        let extra = f.read(&mut trailer).unwrap();
        assert_eq!(
            extra, 0,
            "source file has bytes beyond rebuilt total {consumed}"
        );
        assert_eq!(consumed, real_bytes as u64);
    }

    // OS 终态采样（F787-R2-09：覆盖构建/查询/源重建全阶段后的峰值）+
    // 证据断言（Windows 实测；其它宿主 sysinfo=0 → 显式缺口记录，不作断言）。
    rt_peak = rt_peak.max(os_private());
    let rt_delta = rt_peak.saturating_sub(rt_base);
    if OS_EVIDENCE {
        assert!(
            rt_delta <= 64 << 20,
            "runtime private bytes delta {rt_delta} > 64MiB target (gen peak {gen_peak} separate)"
        );
    } else {
        eprintln!(
            "BUDGET64 OS-EVIDENCE-GAP: sysinfo returns 0 on this host; managed-only accounting"
        );
    }

    // 5) 旧视图释放（真实 lease：drop → lease 释放；页缓存随视图回收）。
    drop(view);

    eprintln!(
        "BUDGET64 src={}MiB decls={} build={:.1}s query200={:.1}s managed_peak={}KiB hot={}KiB manifest={}KiB disk={}MiB rt_private_delta={}KiB gen_peak={}KiB",
        real_bytes >> 20,
        decl_count,
        build_secs,
        query_secs,
        managed_peak >> 10,
        hot >> 10,
        manifest_bytes >> 10,
        du >> 20,
        rt_delta >> 10,
        gen_peak >> 10,
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// 单个巨声明无法在配置预算内完成 → 受控 ResourceExhausted（不是语法
/// 错误、不是 OOM/截断）。
#[test]
fn syntax_budget_giant_decl_resource_exhausted() {
    let p = mini_grammar();
    let root = scratch("budget-giant");
    let sas = SyntaxArtifactStore::open(&root, "big").unwrap();
    let mut b = StreamingCstBuilder::new(&sas, &p).with_budget(Budget {
        max_parse_depth: 1 << 20,
        work_bytes: 1 << 20,
    });
    let mut text = String::from("api f(");
    for i in 0..60_000 {
        text.push_str(&format!("a{i}: i32, "));
    }
    text.push_str("zz: i32) -> i32;");
    let r = b.push_chunk(&text);
    match r {
        Err(ac_syntax_artifact_adapter::AdapterError::Rt(RtError::ResourceExhausted {
            ..
        })) => {}
        other => panic!("giant decl must be controlled ResourceExhausted, got {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&root);
}
