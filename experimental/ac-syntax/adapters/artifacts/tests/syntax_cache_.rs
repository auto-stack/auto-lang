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
use ac_syntax_rt::snapshot::{Edit, EditRequest, PageStore, Snapshot};
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
        // F787-R3-04：跨行未闭合字符串——恢复后的 token 必须保持**原始起始行**
        // （checkpoint.line 只描述 EOF 位置）。
        (
            "note \"first line
second line",
            " closes\";
api f() -> i32;
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

/// F787-R4-01 永久 oracle：同一 snapshot 的 fresh 与增量树入库共存。
/// fresh arena 与增量共享块的布局不同，但规范形（`canonicalize`）相同——
/// 两次 put 必须幂等复用（修复前：A03 页键含块序号且内容为内存块原样
/// 编码，第二次 put 在 page=0 返回 Store::Conflict）。覆盖：fresh→增量、
/// 增量→fresh（反向序）、重复编辑、跨 store 实例重开查询；control：不同
/// 文本的 manifest id 不同。
#[test]
fn syntax_cache_fresh_and_incremental_same_snapshot_coexist() {
    let p = mini_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    let text = (0..40)
        .map(|i| format!("api f{i}() -> i32;\n"))
        .collect::<String>();
    let store_rc = Rc::new(PageStore::new());
    let s = Snapshot::from_text(&store_rc, &text);
    let old = parse_snapshot(&p, &cs, &s, &Budget::default()).unwrap();
    let pos = text.find("f20").unwrap() as u64;
    let ed = Edit {
        start: pos,
        end: pos,
        insert: b"new_".to_vec(),
    };
    let ns = s
        .edit(&EditRequest {
            base_revision: 0,
            edits: vec![ed.clone()],
        })
        .unwrap();
    let inc =
        ac_syntax_rt::edit::incremental_reparse(&p, &cs, &old, &s, &ns, &[ed], &Budget::default())
            .unwrap();
    let fresh = parse_snapshot(&p, &cs, &ns, &Budget::default()).unwrap();
    // 前提：逻辑等价（结构/跨度/字段/标记）且确有增量复用。
    assert_eq!(inc.tree.canonical(), fresh.tree.canonical());
    assert!(inc.reused_decls > 0, "incremental path must reuse decls");

    let root = scratch("cache-r4-coexist");
    let (tok, m_fresh_first) = {
        let sas = SyntaxArtifactStore::open(&root, "proj").unwrap();
        let t = sas.put_tokens(&p, &ns, &fresh.tokens).unwrap();
        let m_fresh = sas.put_cst(&p, &ns, &fresh.tree, &t).unwrap();
        // 同 snapshot 的增量树 put：幂等复用（同键同内容），不再 Conflict。
        let m_inc = sas
            .put_cst(&p, &ns, &inc.tree, &t)
            .expect("incremental put of the same snapshot must be idempotent");
        assert_eq!(
            m_fresh, m_inc,
            "canonical pages must give the same manifest artifact"
        );
        (t, m_fresh)
    };

    // 反向序（独立 store）：增量先入、fresh 后入——同为幂等。
    {
        let root2 = scratch("cache-r4-coexist-rev");
        let sas = SyntaxArtifactStore::open(&root2, "proj").unwrap();
        let t = sas.put_tokens(&p, &ns, &inc.tokens).unwrap();
        let m_inc = sas.put_cst(&p, &ns, &inc.tree, &t).unwrap();
        let m_fresh = sas
            .put_cst(&p, &ns, &fresh.tree, &t)
            .expect("fresh put after incremental must be idempotent");
        assert_eq!(m_inc, m_fresh);
        let _ = std::fs::remove_dir_all(&root2);
    }

    // 重开（新 store 实例 = 跨进程恢复语义）：编辑后声明可查、名字正确。
    {
        let sas = SyntaxArtifactStore::open(&root, "proj").unwrap();
        let view = sas.open_cst(&p, &ns).unwrap().expect("manifest hit");
        assert_eq!(view.decl_count(), 40);
        let st = view.decl_subtree(20).unwrap();
        let items = ac_syntax_rt::outline::outline(&p, &st, &ns).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "new_f20");
    }

    // 重复编辑：在编辑后的 snapshot 上再编辑 → 新 snapshot 的两棵树仍幂等。
    {
        let sas = SyntaxArtifactStore::open(&root, "proj").unwrap();
        let pos2 = ns.read_str(0, ns.len()).unwrap().find("f30").unwrap() as u64;
        let ed2 = Edit {
            start: pos2,
            end: pos2,
            insert: b"v2_".to_vec(),
        };
        let ns2 = ns
            .edit(&EditRequest {
                base_revision: ns.revision(),
                edits: vec![ed2.clone()],
            })
            .unwrap();
        let inc_out = ac_syntax_rt::parse::ParseOutput {
            tree: inc.tree.clone(),
            tokens: inc.tokens.clone(),
            lex_diags: inc.lex_diags.clone(),
            grafted_decls: inc.reused_decls,
        };
        let inc2 = ac_syntax_rt::edit::incremental_reparse(
            &p,
            &cs,
            &inc_out,
            &ns,
            &ns2,
            &[ed2],
            &Budget::default(),
        )
        .unwrap();
        let fresh2 = parse_snapshot(&p, &cs, &ns2, &Budget::default()).unwrap();
        assert_eq!(inc2.tree.canonical(), fresh2.tree.canonical());
        let t2 = sas.put_tokens(&p, &ns2, &fresh2.tokens).unwrap();
        let m1 = sas.put_cst(&p, &ns2, &fresh2.tree, &t2).unwrap();
        let m2 = sas
            .put_cst(&p, &ns2, &inc2.tree, &t2)
            .expect("second-edit incremental put must be idempotent");
        assert_eq!(m1, m2);

        // control：不同文本（不同 snapshot id）的 manifest id 必然不同。
        assert_ne!(
            m1, m_fresh_first,
            "different snapshots must not share a manifest artifact"
        );
        assert!(
            sas.open_cst(&p, &ns).is_ok(),
            "old snapshot manifest still hit"
        );
        assert!(sas.open_cst(&p, &ns2).unwrap().is_some());
    }
    let _ = tok;
    let _ = std::fs::remove_dir_all(&root);
}

/// F787-R4-03 永久 oracle：跨源页前导 trivia 的范围查询。声明单元 origin
/// 覆盖前导 trivia，可位于语义 span 之前的源页——查询 bin 的源覆盖窗口
/// 从 origin 起算（修复前只物化语义 span 相交页，origin 平移无符号下溢
/// exit=101）。覆盖：跨页空白、CRLF、Unicode 注释 trivia、多页 trivia、
/// 非首声明；断言精确 span 与 outline 名。
#[test]
fn syntax_cache_range_query_leading_trivia_across_pages() {
    let p = mini_grammar();
    let decl = "api f() -> i32;";
    let page = 256 * 1024;
    // (名字, 前导 trivia 文本, 查询的声明序号, 期望名)
    let cases: &[(&str, String, usize, &str)] = &[
        // 跨页空白前导（origin=0 在页 0，语义 span 在页 1）。
        ("space-cross", " ".repeat(page + 10), 0, "f"),
        // CRLF 前导（跨页）。
        ("crlf-cross", "\r\n".repeat(page / 2 + 5), 0, "f"),
        // Unicode 注释 trivia（非 ASCII 多字节字符块，>256KiB 跨 store 源页
        // 边界——页边界可切字符；窗口切片在 token 边界，from_utf8 必须安全）。
        (
            "unicode-comment",
            format!("/* {} */{}", "🌙か文".repeat(page / 8), " ".repeat(8)),
            0,
            "f",
        ),
        // 多页 trivia（>2 个源页）。
        ("multi-page", " ".repeat(page * 2 + 100), 0, "f"),
        // 非首声明：decl 0 在前（其后再有跨页 trivia），查询 decl 1。
        (
            "second-decl",
            format!("api g() -> i32;\n{}", " ".repeat(page * 2 + 100)),
            1,
            "f",
        ),
        // F787-R5-02：前导 trivia >2MiB（旧隐藏 guard 会静默空名）。
        ("over-2mib-trivia", " ".repeat((2 << 20) + 10), 0, "f"),
    ];
    for (name, prefix, decl_idx, expect_name) in cases {
        let expect_name = *expect_name;
        let decl_idx = *decl_idx;
        let text = format!("{prefix}{decl}\n");
        let root = scratch(&format!("cache-r4-trivia-{name}"));
        let sas = SyntaxArtifactStore::open(&root, "proj").unwrap();
        let mut b = ac_syntax_artifact_adapter::StreamingCstBuilder::new(&sas, &p);
        b.push_chunk(&text).unwrap();
        b.finish(text.len() as u64).unwrap();
        let id = b.stream_snapshot_id();

        // 进程内期望值（视图直读）：
        let view = sas
            .open_cst_stream(&p, &id, text.len() as u64)
            .unwrap()
            .expect("manifest");
        let (span_lo, span_hi) = view.decl_range(decl_idx).unwrap();
        let st = view.decl_subtree(decl_idx).unwrap();
        assert!(
            st.decls[0].origin < span_lo || prefix.is_empty(),
            "case {name}: leading trivia must place origin before the semantic span"
        );
        drop(view);

        let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_ac787-query"));
        cmd.arg(&root)
            .arg(p.grammar_id().as_str())
            .arg(p.revision.to_string())
            .arg(&id)
            .arg(text.len().to_string())
            .arg(decl_idx.to_string());
        let r = run_deadlined(cmd, 120);
        let last = r.stdout.lines().last().unwrap_or("");
        assert!(
            last.starts_with("QUERY "),
            "case {name}: unexpected output: {}",
            r.stdout
        );
        let fields: std::collections::HashMap<&str, &str> = last
            .split_whitespace()
            .skip(1)
            .map(|kv| kv.split_once('=').expect("k=v field"))
            .collect();
        assert_eq!(
            fields["span"],
            format!("{span_lo},{span_hi}"),
            "case {name}: exact span"
        );
        assert_eq!(
            fields["name"], expect_name,
            "case {name}: outline name (no underflow / no swallowed error)"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}

/// F787-R5-02 阈值两侧永久 oracle：语义 name 窗口超限 → 受控
/// ResourceExhausted（exit 4），绝不用空 name 冒充成功；阈值内精确取名。
/// 用 AC787_QUERY_WINDOW_LIMIT 把阈值压到可测区间（两侧各一例）。
#[test]
fn syntax_cache_range_query_name_window_limit_is_controlled() {
    let p = mini_grammar();
    let text = "api f() -> i32;\n";
    let root = scratch("cache-r5-window-limit");
    let sas = SyntaxArtifactStore::open(&root, "proj").unwrap();
    let mut b = ac_syntax_artifact_adapter::StreamingCstBuilder::new(&sas, &p);
    b.push_chunk(text).unwrap();
    b.finish(text.len() as u64).unwrap();
    let id = b.stream_snapshot_id();

    // 阈值内（name 字段 `f` 很小）：精确 name=f，exit 0。
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_ac787-query"));
    cmd.arg(&root)
        .arg(p.grammar_id().as_str())
        .arg(p.revision.to_string())
        .arg(&id)
        .arg(text.len().to_string())
        .arg("0");
    let r = run_deadlined(cmd, 30);
    assert!(
        r.stdout.contains("name=f"),
        "under-limit query must keep exact name: {}",
        r.stdout
    );

    // 阈值外（limit=1 字节，name 窗口必超）：受控 ResourceExhausted exit 4，
    // 不是 name= 空的假成功。
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_ac787-query"));
    cmd.env("AC787_QUERY_WINDOW_LIMIT", "1")
        .arg(&root)
        .arg(p.grammar_id().as_str())
        .arg(p.revision.to_string())
        .arg(&id)
        .arg(text.len().to_string())
        .arg("0");
    let out = ac_artifact_probe::subproc::run_deadlined(cmd, std::time::Duration::from_secs(30))
        .expect("spawn over-limit query");
    assert!(!out.timed_out, "over-limit query must finish promptly");
    assert_eq!(
        out.code,
        Some(4),
        "over-limit name window must be controlled ResourceExhausted, got {:?} stdout={} stderr={}",
        out.code,
        out.stdout,
        out.stderr
    );
    assert!(
        !out.stdout.contains("name=f"),
        "over-limit must not pretend successful exact name"
    );
    assert!(
        out.stderr.contains("ResourceExhausted"),
        "over-limit must name the controlled resource exit: {}",
        out.stderr
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// F787-R6-02 永久 oracle：超限拒绝发生在**继续大规模分配之前**——真实
/// 大 name 字段（512KiB）+ 小 limit（64KiB）下 exit 4，且 stderr 自报的
/// 已物化窗口 ≤ limit + 一个 store 源页（256KiB）——限额在 append 前判定
/// （修复前：先整窗物化再在循环后检查，8MiB 名称先积累 8MiB 驻留）。
#[test]
fn syntax_cache_range_query_window_limit_bounds_materialization() {
    let p = mini_grammar();
    let name = "x".repeat(512 << 10);
    let text = format!("api {name}() -> i32;\n");
    let root = scratch("cache-r6-window-bound");
    let sas = SyntaxArtifactStore::open(&root, "proj").unwrap();
    let mut b = ac_syntax_artifact_adapter::StreamingCstBuilder::new(&sas, &p);
    b.push_chunk(&text).unwrap();
    b.finish(text.len() as u64).unwrap();
    let id = b.stream_snapshot_id();

    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_ac787-query"));
    cmd.env("AC787_QUERY_WINDOW_LIMIT", (64 << 10).to_string())
        .arg(&root)
        .arg(p.grammar_id().as_str())
        .arg(p.revision.to_string())
        .arg(&id)
        .arg(text.len().to_string())
        .arg("0");
    let out = ac_artifact_probe::subproc::run_deadlined(cmd, std::time::Duration::from_secs(120))
        .expect("spawn bounded query");
    assert!(!out.timed_out, "bounded rejection must finish promptly");
    assert_eq!(
        out.code,
        Some(4),
        "over-limit big name must be controlled ResourceExhausted, stdout={} stderr={}",
        out.stdout,
        out.stderr
    );
    let materialized: usize = out
        .stderr
        .split("window ")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse().ok())
        .expect("stderr must report the materialized window bytes");
    assert!(
        materialized <= (64 << 10) + (256 << 10),
        "rejection must precede further mass allocation: at most one 256KiB source page overshoot over the limit, observed {materialized} bytes"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// F787-R7-01 永久正控：真实 SourcePage 生产 → **新 store 实例（重开）** →
/// 公开产物读入口按 key 与按 artifact id 均可读（kind=SourcePage、payload
/// 逐字节全等）。修复前 store 侧 kind 映射表与 codec 帧层漂移、漏 10——
/// 两入口在读元数据时即 `Internal("bad kind")`（尚未进入 payload 校验）。
/// 拒绝边界不变：未知 kind 字节仍拒绝（probe admission 族既有控制）。
#[test]
fn syntax_cache_source_page_artifact_reads_by_key_and_id() {
    use ac_artifact_probe::codec::ChunkKind;
    use ac_artifact_probe::identity::{domain_hash, QueryKey};
    use ac_artifact_probe::schema;

    let p = mini_grammar();
    let text = "api f() -> i32;\n";
    let snap = Snapshot::from_text(&Rc::new(PageStore::new()), text);
    let root = scratch("cache-r7-srcpage");
    let art_ids = {
        let sas = SyntaxArtifactStore::open(&root, "proj").unwrap();
        let (arts, _chunks, put_peak) = sas.put_source_pages(&snap).unwrap();
        // 三份驻留口径下界自检（R6-01 公式在真实路径上的可见性）。
        assert!(put_peak >= text.len() * 3);
        arts
    };
    assert_eq!(art_ids.len(), 1, "single aggregated source page");

    // 重开（跨实例恢复语义）+ 公开入口读取。
    let sas = SyntaxArtifactStore::open(&root, "proj").unwrap();
    let hash = domain_hash("src-page", &[text.as_bytes()]);
    let key = QueryKey::new(
        schema::layer("A02").unwrap(),
        vec![("page-hash".into(), hash)],
    );
    let by_key = sas
        .inner()
        .get_artifact("A02", &key)
        .unwrap()
        .expect("SourcePage readable by its page-hash key after reopen");
    assert_eq!(by_key.kind, ChunkKind::SourcePage);
    assert_eq!(by_key.payload, text.as_bytes());
    let by_id = sas
        .inner()
        .get_artifact_by_id(&art_ids[0])
        .unwrap()
        .expect("SourcePage readable by artifact id after reopen");
    assert_eq!(by_id.kind, ChunkKind::SourcePage);
    assert_eq!(by_id.payload, text.as_bytes());
    assert_eq!(by_id.artifact_id, art_ids[0]);
    let _ = p;
    let _ = std::fs::remove_dir_all(&root);
}
