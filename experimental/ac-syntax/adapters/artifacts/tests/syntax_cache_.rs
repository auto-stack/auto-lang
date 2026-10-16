//! V-09 持久化（PLAN-787 T-06）：fresh→退出→新 store 实例（跨进程恢复语义）
//! 小查询；内容共享而来源隔离；parser 禁调（从页反序列化，不重 parse）。
//! 修复轮（F787-R1-05/06/09）：A02 checkpoint 完整往返；源页可重建原文；
//! 双工程空间共享断言（非丢弃 space report）；真子进程查询（grammar 从
//! store 反序列化，CST 走 manifest/页路径）。

use ac_syntax_artifact_adapter::SyntaxArtifactStore;
use ac_syntax_grammar::compile as compile_grammar;
use ac_syntax_rt::lexer::CompiledScanner;
use ac_syntax_rt::parse::parse_snapshot;
use ac_syntax_rt::program::{Budget, GrammarProgram};
use ac_syntax_rt::snapshot::{PageStore, Snapshot};
use std::rc::Rc;

fn mini_grammar() -> GrammarProgram {
    let src = std::fs::read_to_string(format!(
        "{}/../../grammars/interface-mini-v1.at",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    compile_grammar(&src).unwrap()
}

fn mini_grammar_variant() -> GrammarProgram {
    let src = std::fs::read_to_string(format!(
        "{}/../../grammars/interface-mini-v1.at",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replace(r#"{ literal: "api" }"#, r#"{ literal: "endpoint" }"#);
    compile_grammar(&src).unwrap()
}

/// 子进程受截止执行（F787-R2-10：裸 Command.output 无截止——复用探针的
/// run_deadlined：wall-clock 上限 + job-object 树杀 + 管道排空）。
fn run_deadlined(cmd: std::process::Command, secs: u64) -> ac_artifact_probe::subproc::RunResult {
    let r = ac_artifact_probe::subproc::run_deadlined(cmd, std::time::Duration::from_secs(secs))
        .expect("spawn child process");
    assert!(!r.timed_out, "child process timed out (deadline {secs}s)");
    assert!(r.code == Some(0), "child exited {:?}: {}", r.code, r.stderr);
    r
}

fn scratch(family: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("ac787-{family}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
fn syntax_cache_fresh_exit_reopen_small_query() {
    let p = mini_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    let text = "api add(a: i32, b: i32) -> i32;\nnote \"x\";\napi mul(m: i32, n: i32) -> i32;\n";
    let store_rc = Rc::new(PageStore::new());
    let snap = Snapshot::from_text(&store_rc, text);
    let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();

    let root = scratch("cache-reopen");
    {
        let sas = SyntaxArtifactStore::open(&root, "proj-a").unwrap();
        let g = sas.put_grammar(&p).unwrap();
        let t = sas.put_tokens(&p, &snap, &out.tokens).unwrap();
        let _m = sas.put_cst(&p, &snap, &out.tree, &t).unwrap();
        let _ = g;
    } // "进程退出"：drop store 句柄。

    // 新实例恢复：小查询（decl 目录/范围/子树），不调 parser、不反序列化整树。
    let sas2 = SyntaxArtifactStore::open(&root, "proj-a").unwrap();
    let view = sas2.open_cst(&p, &snap).unwrap().expect("manifest hit");
    assert_eq!(
        view.decl_count(),
        3,
        "decls: {:?}",
        view.debug_decl_entries()
    );
    assert_eq!(view.total_len(), text.len() as u64);
    let (s0, e0) = view.decl_range(0).unwrap();
    assert_eq!(
        &text[s0 as usize..e0 as usize],
        "api add(a: i32, b: i32) -> i32;"
    );
    let st = view.decl_subtree(2).unwrap();
    // 子树 outline 候选（页面重建 + 字段读取，无 parse）。
    let snap2 = Snapshot::from_text(&store_rc, text);
    let items = ac_syntax_rt::outline::outline(&p, &st, &snap2).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "mul");
    // 源重建（F787-R1-06）：从 store 源页逐字节还原原文。
    let rebuilt = view.source_bytes().unwrap();
    assert_eq!(rebuilt, text.as_bytes(), "source rebuild from store pages");
}

#[test]
fn syntax_cache_a01_a02_roundtrip_identity_binding() {
    let p = mini_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    let text = "api q(a: i32) -> i32;\n";
    let snap = Snapshot::from_text(&Rc::new(PageStore::new()), text);
    let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();

    let root = scratch("cache-a0102");
    let sas = SyntaxArtifactStore::open(&root, "proj-a").unwrap();
    sas.put_tokens(&p, &snap, &out.tokens).unwrap();

    let g = sas.get_grammar(&p).unwrap().expect("grammar hit");
    assert_eq!(g.identity, p.identity, "identity revalidation must match");
    let t = sas.get_tokens(&p, &snap).unwrap().expect("tokens hit");
    assert_eq!(t, out.tokens);

    // A02 checkpoint 完整往返（F787-R1-05）：token 流 + EOF 终态
    // （mode/未闭合状态/行）可再装载。
    let (t2, cp) = sas
        .get_tokens_with_checkpoint(&p, &snap)
        .unwrap()
        .expect("tokens+cp hit");
    assert_eq!(t2, out.tokens);
    assert_eq!(cp.offset, snap.len(), "checkpoint offset = EOF");
    assert_eq!(cp.mode, 0);
    assert_eq!(
        cp.line, 2,
        "EOF line counts the trailing newline in the last (trivia) token"
    );
    assert!(cp.mid.is_none());

    // 变体语法（同名不同内容）隔离：A01/A02 键含 GrammarId → miss 而非错拿。
    let p2 = mini_grammar_variant();
    assert_ne!(p.grammar_id(), p2.grammar_id());
    assert!(
        sas.get_grammar(&p2).unwrap().is_none(),
        "variant grammar must miss"
    );
    assert!(
        sas.get_tokens(&p2, &snap).unwrap().is_none(),
        "variant grammar tokens must miss (F787-R1-05)"
    );
}

#[test]
fn syntax_cache_two_projects_share_content_isolate_roots() {
    let p = mini_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    let text = "api shared() -> i32;\n";
    let snap = Snapshot::from_text(&Rc::new(PageStore::new()), text);
    let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();

    let root = scratch("cache-share");
    let store_rc = Rc::new(PageStore::new());
    let snap_b = Snapshot::from_text(&store_rc, text); // 相同文本 → 相同 snapshot id
    for proj in ["proj-x", "proj-y"] {
        let sas = SyntaxArtifactStore::open(&root, proj).unwrap();
        let t = sas.put_tokens(&p, &snap, &out.tokens).unwrap();
        let m = sas
            .put_cst(
                &p,
                if proj == "proj-x" { &snap } else { &snap_b },
                &out.tree,
                &t,
            )
            .unwrap();
        let _ = m;
    }
    // 两工程各自可查（manifest 根独立）。
    let sas = SyntaxArtifactStore::open(&root, "proj-x").unwrap();
    let vx = sas.open_cst(&p, &snap).unwrap().expect("proj-x view");
    let sas_y = SyntaxArtifactStore::open(&root, "proj-y").unwrap();
    let vy = sas_y.open_cst(&p, &snap_b).unwrap().expect("proj-y view");
    // 内容共享断言（F787-R1-09：不丢弃 space report）：
    // (a) 同内容跨工程 = 幂等复用（键内容寻址，零新块）；
    let r1 = sas.inner().space();
    {
        let sas_z = SyntaxArtifactStore::open(&root, "proj-z").unwrap();
        let t = sas_z.put_tokens(&p, &snap, &out.tokens).unwrap();
        sas_z.put_cst(&p, &snap_b, &out.tree, &t).unwrap();
    }
    let r2 = sas.inner().space();
    assert_eq!(
        r2.blocks_count, r1.blocks_count,
        "identical content across projects must add zero blocks"
    );
    // (b) 前缀共享快照：新快照只为其**新增**源页产生块（内容寻址共享）。
    let text2 = format!("{text}{}", "tail-content\n");
    let snap2 = Snapshot::from_text(&store_rc, &text2);
    let before = sas.inner().space().blocks_count;
    sas.put_source_pages(&snap2).unwrap();
    let after = sas.inner().space().blocks_count;
    assert!(
        after - before <= 2,
        "prefix-shared snapshot must reuse existing source pages (delta {} for a {}-page snapshot)",
        after - before,
        snap2.page_count()
    );
    // 诊断隔离：各工程视图读自己的 manifest 诊断投影。
    assert_eq!(vx.diag_count(), vy.diag_count());
    drop(vx);
    drop(vy);
}

/// V-09 跨进程恢复（F787-R1-09/R2-09/10）：**真新进程**（ac787-query bin，
/// 受截止）——grammar 程序从 store 反序列化（不编译 .at），CST 查询走
/// manifest/页路径（bin 内无 parse_snapshot 调用面）；源按页流式 hash 校验；
/// 精确断言声明 span 与 outline 名（可区分负控：错 decl 不得给出同名同位）。
#[test]
fn syntax_cache_cross_process_recovery_query() {
    let p = mini_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    let mut text = String::new();
    for i in 0..500 {
        text.push_str(&format!(
            "api f{i}(a: i32) -> i32;
"
        ));
    }
    let snap = Snapshot::from_text(&Rc::new(PageStore::new()), &text);
    let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();

    let root = scratch("cache-xproc");
    let sas = SyntaxArtifactStore::open(&root, "proj-xp").unwrap();
    sas.put_grammar(&p).unwrap();
    let t = sas.put_tokens(&p, &snap, &out.tokens).unwrap();
    sas.put_cst(&p, &snap, &out.tree, &t).unwrap();

    // 小流式树驱动跨进程查询。
    let mut b = ac_syntax_artifact_adapter::StreamingCstBuilder::new(&sas, &p);
    b.push_chunk(&text).unwrap();
    let total = text.len() as u64;
    b.finish(total).unwrap();
    let stream_id = b.stream_snapshot_id();
    let decl_total = b.decl_count() as usize;

    // 期望值（进程内独立计算，跨进程对照）：
    let idx = 7usize;
    let expected_origin: u64 = text.lines().take(idx).map(|l| l.len() as u64 + 1).sum();
    let expected_end = expected_origin + format!("api f{idx}(a: i32) -> i32;").len() as u64;
    let expected_hash = {
        let mut h = blake3::Hasher::new();
        h.update(text.as_bytes());
        h.finalize().to_hex().to_string()
    };

    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_ac787-query"));
    cmd.arg(&root)
        .arg(p.grammar_id().as_str())
        .arg(p.revision.to_string())
        .arg(&stream_id)
        .arg(total.to_string())
        .arg(idx.to_string());
    let r = run_deadlined(cmd, 120);
    let last = r.stdout.lines().last().unwrap_or("");
    assert!(
        last.starts_with("QUERY "),
        "unexpected query output: {}",
        r.stdout
    );
    let fields: std::collections::HashMap<&str, &str> = last
        .split_whitespace()
        .skip(1) // "QUERY"
        .map(|kv| kv.split_once('=').expect("k=v field"))
        .collect();
    let decls: usize = fields["decls"].parse().unwrap();
    let src_len: u64 = fields["src_len"].parse().unwrap();
    let nodes: u32 = fields["nodes"].parse().unwrap();
    assert_eq!(decls, decl_total, "cross-process decl count");
    assert_eq!(src_len, total, "cross-process streamed source length");
    assert_eq!(
        fields["src_hash"], expected_hash,
        "cross-process streamed source hash"
    );
    assert!(nodes > 0, "cross-process subtree rebuild");
    // 精确 span/name（可区分断言）。
    assert_eq!(
        fields["span"],
        format!("{expected_origin},{expected_end}"),
        "cross-process decl span"
    );
    assert_eq!(
        fields["name"],
        format!("f{idx}"),
        "cross-process outline name"
    );

    // 负控：错 decl 索引必须给出**不同**精确值（不是同名同位）。
    let mut cmd2 = std::process::Command::new(env!("CARGO_BIN_EXE_ac787-query"));
    cmd2.arg(&root)
        .arg(p.grammar_id().as_str())
        .arg(p.revision.to_string())
        .arg(&stream_id)
        .arg(total.to_string())
        .arg("8");
    let r2 = run_deadlined(cmd2, 120);
    let last2 = r2.stdout.lines().last().unwrap_or("");
    let f2: std::collections::HashMap<&str, &str> = last2
        .split_whitespace()
        .skip(1)
        .map(|kv| kv.split_once('=').expect("k=v"))
        .collect();
    assert_ne!(
        f2["span"], fields["span"],
        "distinct decl must have distinct span"
    );
    assert_ne!(
        f2["name"], fields["name"],
        "distinct decl must have distinct name"
    );
}

/// A02 checkpoint 恢复对照（F787-R2-06）：持久化终态在**续写文本**上恢复
/// 扫描，与 fresh 全量词法逐 token 全等（真实 mid 状态——嵌套深度/转义）。
#[test]
fn syntax_cache_checkpoint_resume_matches_fresh_extension() {
    let p = mini_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    // 未闭合嵌套块注释 + 尾转义字符串两个场景分别验证（真实 mid 重放）。
    for (base, ext) in [
        (
            "api a() -> i32;
/* outer /* inner",
            " still open */ api b() -> i32;
",
        ),
        (
            "note \"half\", api c() -> i32;
note \"tail\\",
            "closed\"; api d() -> i32;
",
        ),
    ] {
        let store = Rc::new(PageStore::new());
        let snap = Snapshot::from_text(&store, base);
        let lx = ac_syntax_rt::lexer::lex_all(&p, &cs, &snap).unwrap();
        let root = scratch("cache-cpresume");
        let sas = SyntaxArtifactStore::open(&root, "proj-cp").unwrap();
        sas.put_tokens(&p, &snap, &lx.tokens).unwrap();
        let (tokens2, cp) = sas
            .get_tokens_with_checkpoint(&p, &snap)
            .unwrap()
            .expect("tokens+cp hit");
        assert_eq!(tokens2, lx.tokens);
        // 续写文档：从持久化 checkpoint 恢复扫描 → 与 fresh 全量对照。
        let full_text = format!("{base}{ext}");
        let full_snap = Snapshot::from_text(&store, &full_text);
        let fresh = ac_syntax_rt::lexer::lex_all(&p, &cs, &full_snap).unwrap();
        let mut sc = ac_syntax_rt::lexer::Scanner::start(&p, &cs, &full_snap, &cp).unwrap();
        let mut resumed = Vec::new();
        while let Some(t) = sc.next_token().unwrap() {
            resumed.push(t);
        }
        // EOF 终态恢复从最后一个未闭合 token 起重发——与 fresh 全量词法的
        // 对应尾部逐 token 全等（含跨文本边界的 token 拼接/行号/错误位）。
        let tail: Vec<ac_syntax_rt::lexer::Token> =
            fresh.tokens[fresh.tokens.len() - resumed.len()..].to_vec();
        assert_eq!(
            resumed, tail,
            "checkpoint resume must equal the fresh lex tail of the extended text"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
