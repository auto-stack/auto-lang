//! V-10 数据拒绝（PLAN-787 T-06）：有效 frame 替换错 chunk、坏 offset/ref、
//! 坏 manifest 身份、缺页 —— 真读取入口受控拒绝/重算，无 panic/错树成功。

use ace_syntax_artifacts::{AdapterError, SyntaxArtifactStore};
use ace_syntax_grammar::compile as compile_grammar;
use ace_syntax_runtime::lexer::CompiledScanner;
use ace_syntax_runtime::parse::parse_snapshot;
use ace_syntax_runtime::program::{Budget, GrammarProgram};
use ace_syntax_runtime::snapshot::{Edit, EditRequest, PageStore, Snapshot};
use std::rc::Rc;

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

/// CAS 块文件为只读（不可变契约）；破坏性测试先去只读再改写。
fn make_writable(p: &std::path::Path) {
    let mut perm = std::fs::metadata(p).unwrap().permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    perm.set_readonly(false);
    std::fs::set_permissions(p, perm).unwrap();
}

/// manifest 页 0 的 chunk 文件路径（blocks/blobs/<shard>/<chunk>）。
fn page_chunk_path(
    sas: &SyntaxArtifactStore,
    root: &std::path::Path,
    p: &GrammarProgram,
    snap: &Snapshot,
) -> std::path::PathBuf {
    let view = sas.open_cst(p, snap).unwrap().expect("manifest");
    let chunk = view.debug_page_chunk(0).expect("page 0");
    // shard = 完整 chunk id 的前两字符（含 chk 前缀）。
    root.join("blocks/blobs").join(&chunk[..2]).join(&chunk)
}

fn setup(
    root: &std::path::Path,
) -> (
    GrammarProgram,
    Snapshot,
    ace_syntax_runtime::parse::ParseOutput,
) {
    let p = mini_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    let text = "api a(x: i32) -> i32;\napi b(y: i32) -> i32;\napi c(z: i32) -> i32;\n";
    let snap = Snapshot::from_text(&Rc::new(PageStore::new()), text);
    let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();
    let sas = SyntaxArtifactStore::open(root, "adm").unwrap();
    sas.put_grammar(&p).unwrap();
    let t = sas.put_tokens(&p, &snap, &out.tokens).unwrap();
    sas.put_cst(&p, &snap, &out.tree, &t).unwrap();
    (p, snap, out)
}

#[test]
fn syntax_admission_page_chunk_corruption_controlled() {
    let root = scratch("adm-corrupt");
    let (p, snap, _out) = setup(&root);
    // 定位一个页 chunk 文件并破坏（写坏字节）——CAS 读取校验拒绝。
    let sas0 = SyntaxArtifactStore::open(&root, "adm").unwrap();
    let victim = page_chunk_path(&sas0, &root, &p, &snap);
    assert!(victim.is_file(), "page chunk {victim:?}");
    make_writable(&victim);
    let mut data = std::fs::read(&victim).unwrap();
    let n = data.len();
    data[n - 3] ^= 0xFF;
    data[n - 5] ^= 0xFF;
    std::fs::write(victim, data).unwrap();

    let sas = SyntaxArtifactStore::open(&root, "adm").unwrap();
    let view = sas.open_cst(&p, &snap).unwrap().expect("manifest readable");
    // 某页读取必然受控失败（manifest 页 0 指向受害 chunk 或遍历到时）。
    let mut saw_controlled = false;
    for i in 0..view.decl_count() {
        if let Err(e) = view.decl_subtree(i) {
            assert!(
                matches!(e, AdapterError::Store(_) | AdapterError::Admissions(_)),
                "panic-free controlled: {e}"
            );
            saw_controlled = true;
        }
    }
    assert!(
        saw_controlled,
        "corrupted page must surface as controlled error"
    );
}

#[test]
fn syntax_admission_missing_page_controlled_miss() {
    let root = scratch("adm-missing");
    let (p, snap, _out) = setup(&root);
    // 删除一个页 chunk（manifest 引用悬空）。
    let sas0 = SyntaxArtifactStore::open(&root, "adm").unwrap();
    let victim = page_chunk_path(&sas0, &root, &p, &snap);
    assert!(victim.is_file(), "page chunk {victim:?}");
    make_writable(&victim);
    std::fs::remove_file(&victim).unwrap();

    let sas = SyntaxArtifactStore::open(&root, "adm").unwrap();
    let view = sas.open_cst(&p, &snap).unwrap().expect("manifest readable");
    let mut saw_miss = false;
    for i in 0..view.decl_count() {
        if let Err(e) = view.decl_subtree(i) {
            assert!(
                matches!(e, AdapterError::Admissions(_)),
                "controlled miss: {e}"
            );
            saw_miss = true;
        }
    }
    assert!(saw_miss, "missing page must be a controlled miss/error");
}

#[test]
fn syntax_admission_bad_manifest_identity_rejected() {
    let root = scratch("adm-id");
    let (p, _snap, out) = setup(&root);
    // 用同一 store 但**不同 snapshot**（内容不同 → key miss）——open 返回 None。
    let other_text = "api other(w: i32) -> i32;\n";
    let other_snap = Snapshot::from_text(&Rc::new(PageStore::new()), other_text);
    let sas = SyntaxArtifactStore::open(&root, "adm").unwrap();
    assert!(
        sas.open_cst(&p, &other_snap).unwrap().is_none(),
        "different snapshot must key-miss"
    );
    // 不同 grammar（同快照）→ key miss。
    let src2 = std::fs::read_to_string(format!(
        "{}/../../grammars/interface-mini-v1.at",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replace(r#"{ literal: "api" }"#, r#"{ literal: "endpoint" }"#);
    let p2 = compile_grammar(&src2).unwrap();
    let _ = out;
    let snap = Snapshot::from_text(
        &Rc::new(PageStore::new()),
        "api a(x: i32) -> i32;\napi b(y: i32) -> i32;\napi c(z: i32) -> i32;\n",
    );
    assert!(
        sas.open_cst(&p2, &snap).unwrap().is_none(),
        "different grammar must key-miss"
    );
}

#[test]
fn syntax_admission_truncated_page_rejected() {
    // 直接构造坏 payload 的页：绕过 store，用 decode 路径的视图级校验——
    // 通过破坏 chunk 为**有效 frame 但内容截断**（校验和重算）。
    let root = scratch("adm-trunc");
    let (p, snap, _out) = setup(&root);
    let sas0 = SyntaxArtifactStore::open(&root, "adm").unwrap();
    let victim = page_chunk_path(&sas0, &root, &p, &snap);
    assert!(victim.is_file(), "page chunk {victim:?}");
    make_writable(&victim);
    let framed = std::fs::read(&victim).unwrap();
    // payload = framed[48..]，截 10 字节后重新 frame（有效 frame + 坏内容）。
    let payload = &framed[48..];
    let payload = &payload[..payload.len().saturating_sub(10)];
    let reframed =
        ace_artifacts::codec::frame_chunk(ace_artifacts::codec::ChunkKind::CstPage, payload);
    std::fs::write(victim, reframed).unwrap();

    let sas = SyntaxArtifactStore::open(&root, "adm").unwrap();
    let view = sas.open_cst(&p, &snap).unwrap().expect("manifest readable");
    let mut saw_bad = false;
    for i in 0..view.decl_count() {
        if let Err(e) = view.decl_subtree(i) {
            // 有效 frame + 坏内容：unframe/decode 层受控拒绝（Admissions 或 Store 校验）。
            assert!(
                matches!(e, AdapterError::Admissions(_) | AdapterError::Store(_)),
                "{e}"
            );
            saw_bad = true;
        }
    }
    assert!(saw_bad, "truncated page payload must be rejected");
}

#[test]
fn syntax_admission_valid_framed_bad_manifest_ref_is_controlled() {
    // F787-R1-07 反例：有效 frame/hash 的 manifest，但 decl 引用越界页号——
    // 真读取入口必须受控错误，绝不 panic。
    use ace_artifacts::{codec::ChunkKind, identity::QueryKey, schema, store::PutRequest};
    let root = scratch("adm-badref");
    let p = mini_grammar();
    let snap = Snapshot::from_text(&Rc::new(PageStore::new()), "api a() -> i32;");
    let sas = SyntaxArtifactStore::open(&root, "adm").unwrap();
    let manifest = serde_json::json!({
        "snapshot": snap.id().as_str(),
        "grammar": p.grammar_id().as_str(),
        "total_len": snap.len(),
        "diags": [],
        "root_edges": [],
        "pages": [],
        "decl_index": [],
        "decl_count": 1,
        "decls": [{"origin": 0, "node": 0, "node_lo": 0, "page": 999}],
        "source_pages": [],
    });
    let key = QueryKey::new(
        schema::layer("A03").unwrap(),
        vec![
            ("source-snapshot".into(), snap.id().as_str().into()),
            ("grammar-id".into(), p.grammar_id().as_str().into()),
            (
                "parser-version".into(),
                ace_syntax_runtime::PARSER_ABI_VERSION.to_string(),
            ),
            ("page".into(), "manifest".into()),
        ],
    );
    sas.inner()
        .put_artifact(PutRequest {
            layer: "A03",
            key,
            kind: ChunkKind::CstManifest,
            payload: serde_json::to_vec(&manifest).unwrap(),
            edges: vec![],
            project: None,
        })
        .unwrap();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // validate_manifest 拒绝页号越界（页 999 >= 0 页）→ Err；断言不 panic。
        matches!(sas.open_cst(&p, &snap), Err(_) | Ok(None) | Ok(Some(_)))
    }));
    assert!(outcome.is_ok(), "bad manifest ref must not panic");
    match sas.open_cst(&p, &snap) {
        Err(e) => assert!(
            matches!(e, AdapterError::Admissions(_)),
            "controlled admission: {e}"
        ),
        Ok(None) => panic!("planted manifest must be found (identity matches)"),
        Ok(Some(v)) => {
            assert!(
                matches!(v.decl_subtree(0), Err(AdapterError::Admissions(_))),
                "decl ref must be rejected"
            );
        }
    }
}

#[test]
fn syntax_admission_root_replacement_lease_release_gc_lifecycle() {
    // F787-R1-06：当前根替换 + 显式 lease 生命周期——GC 保活（lease 内）与
    // 释放后回收（lease 释放 + 根已被替换 → 旧版本可淘汰）。
    let p = mini_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    let root = scratch("adm-lifecycle");
    let sas = SyntaxArtifactStore::open(&root, "proj").unwrap();
    sas.put_grammar(&p).unwrap();

    let store_rc = Rc::new(PageStore::new());
    let text1 = "api v1(a: i32) -> i32;\n";
    let snap1 = Snapshot::from_text(&store_rc, text1);
    let out1 = parse_snapshot(&p, &cs, &snap1, &Budget::default()).unwrap();
    let t1 = sas.put_tokens(&p, &snap1, &out1.tokens).unwrap();
    sas.put_cst(&p, &snap1, &out1.tree, &t1).unwrap();

    // v2 发布 → 当前根替换（v1 失去 cst-manifest 硬根）。
    let text2 = "api v2(b: i32) -> i32;\napi v2c(c: i32) -> i32;\n";
    let snap2 = Snapshot::from_text(&store_rc, text2);
    let out2 = parse_snapshot(&p, &cs, &snap2, &Budget::default()).unwrap();
    let t2 = sas.put_tokens(&p, &snap2, &out2.tokens).unwrap();
    sas.put_cst(&p, &snap2, &out2.tree, &t2).unwrap();

    // v1 经视图 lease 读：强制 GC 后仍可查（lease → manifest → required 闭包）。
    let v1 = sas
        .open_cst(&p, &snap1)
        .unwrap()
        .expect("v1 view under lease");
    ace_artifacts::store::gc::run_gc(
        sas.inner(),
        ace_artifacts::store::gc::GcConfig { high: 0, low: 0 },
    )
    .unwrap();
    assert!(
        v1.decl_subtree(0).is_ok(),
        "leased old version must survive forced GC"
    );
    assert!(
        sas.open_cst(&p, &snap2).unwrap().is_some(),
        "current root must survive GC"
    );

    // lease 释放（drop 视图）→ 旧版本可回收：GC 后 v1 miss、v2 仍在。
    drop(v1);
    let receipt = ace_artifacts::store::gc::run_gc(
        sas.inner(),
        ace_artifacts::store::gc::GcConfig { high: 0, low: 0 },
    )
    .unwrap();
    assert!(
        receipt.deleted_chunks > 0,
        "released old version must be physically reclaimed"
    );
    assert!(
        sas.open_cst(&p, &snap1).unwrap().is_none(),
        "v1 manifest must be gone after reclaim"
    );
    assert!(
        sas.open_cst(&p, &snap2).unwrap().is_some(),
        "current root v2 must remain queryable"
    );
}

/// F787-R3-02：同文本跨修订幂等复用——修订不进内容工件，第二次 put 不
/// Conflict（正常编辑回到既有文本/跨文档同源共享的适用身份）。
#[test]
fn syntax_admission_identical_content_at_new_revision_is_reusable() {
    let p = mini_grammar();
    let text = "api f() -> i32;
";
    let s0 = Snapshot::from_text(&Rc::new(PageStore::new()), text);
    let s1 = s0
        .edit(&EditRequest {
            base_revision: 0,
            edits: vec![Edit {
                start: 0,
                end: s0.len(),
                insert: text.as_bytes().to_vec(),
            }],
        })
        .unwrap();
    assert_eq!(s0.id(), s1.id());
    assert_eq!(s1.revision(), 1);
    let root = scratch("adm-same-content");
    let sas = SyntaxArtifactStore::open(&root, "project").unwrap();
    put_all(&sas, &p, &s0).unwrap();
    put_all(&sas, &p, &s1).expect("same source at revision1 must reuse content, not conflict");
}

/// F787-R3-02：跨会话/重启的 revision=0 是新文档发布，不是旧会话迟到任务。
#[test]
fn syntax_admission_fresh_session_revision_zero_can_be_current() {
    let p = mini_grammar();
    let root = scratch("adm-restart");
    {
        let s0 = Snapshot::from_text(
            &Rc::new(PageStore::new()),
            "api a() -> i32;
",
        );
        let s1 = s0
            .edit(&EditRequest {
                base_revision: 0,
                edits: vec![Edit {
                    start: 4,
                    end: 5,
                    insert: b"aa".to_vec(),
                }],
            })
            .unwrap();
        let sas = SyntaxArtifactStore::open(&root, "project").unwrap();
        put_all(&sas, &p, &s1).unwrap();
    } // 会话关闭
    let new_doc = Snapshot::from_text(
        &Rc::new(PageStore::new()),
        "api new() -> i32;
",
    );
    assert_eq!(new_doc.revision(), 0);
    let sas = SyntaxArtifactStore::open(&root, "project").unwrap();
    let a = put_all(&sas, &p, &new_doc).unwrap();
    assert!(
        sas.inner()
            .root_artifacts("project", "cst-manifest")
            .unwrap()
            .contains(&a),
        "revision0 in a new session is not a late task from the old session"
    );
}

/// F787-R3-03：注册失败的原子性——先注册后删除的顺序保证旧当前根保留
/// （trigger 注入确定性失败于本测试自有临时 store，无实现改动）。
#[test]
fn syntax_admission_failed_root_registration_preserves_previous_current_root() {
    let p = mini_grammar();
    let s0 = Snapshot::from_text(
        &Rc::new(PageStore::new()),
        "api old() -> i32;
",
    );
    let s1 = s0
        .edit(&EditRequest {
            base_revision: 0,
            edits: vec![Edit {
                start: 4,
                end: 7,
                insert: b"new".to_vec(),
            }],
        })
        .unwrap();
    let root = scratch("adm-pubfail");
    let sas = SyntaxArtifactStore::open(&root, "project").unwrap();
    let first = put_all(&sas, &p, &s0).unwrap();
    sas.inner()
        .db()
        .execute_batch(
            "CREATE TRIGGER fail_root BEFORE INSERT ON roots WHEN NEW.kind = 'cst-manifest'              BEGIN SELECT RAISE(ABORT, 'injected root registration failure'); END;",
        )
        .unwrap();
    let r = put_all(&sas, &p, &s1);
    assert!(r.is_err(), "failure control must fire");
    let roots = sas
        .inner()
        .root_artifacts("project", "cst-manifest")
        .unwrap();
    assert_eq!(
        roots,
        vec![first],
        "publication error must preserve the previous current root"
    );
}

/// F787-R3-05：有效 frame/hash 下越源 span 在读取入口受控拒绝（不能等
/// outline 读源时才失败）。
#[test]
fn syntax_admission_out_of_source_span_is_rejected() {
    use ace_artifacts::{
        codec::{framed_chunk_id, ChunkKind},
        identity::QueryKey,
        schema,
        store::PutRequest,
    };
    let p = mini_grammar();
    let s = Snapshot::from_text(&Rc::new(PageStore::new()), "x");
    let sas = SyntaxArtifactStore::open(&scratch("adm-span"), "project").unwrap();
    let kind = ace_syntax_runtime::program::RULE_KIND_BASE
        + p.rules.iter().position(|r| r.name == "api-decl").unwrap() as u32;
    let mut payload = Vec::new();
    payload.extend_from_slice(&1u32.to_le_bytes());
    payload.extend_from_slice(&0u32.to_le_bytes());
    for v in [kind, 9u32, 1, 0, 0, 0, 1, 0] {
        payload.extend_from_slice(&v.to_le_bytes());
    }
    let cid = framed_chunk_id(ChunkKind::CstPage, &payload).to_string();
    sas.inner()
        .put_artifact(PutRequest {
            layer: "A03",
            key: QueryKey::new(
                schema::layer("A03").unwrap(),
                vec![("page".into(), "bad-span".into())],
            ),
            kind: ChunkKind::CstPage,
            payload,
            edges: vec![],
            project: None,
        })
        .unwrap();
    let m = serde_json::json!({
        "snapshot": s.id().as_str(),
        "grammar": p.grammar_id().as_str(),
        "total_len": 1,
        "diags": [],
        "root_edges": [],
        "pages": [{"chunk": cid, "first_node": 0, "node_count": 1}],
        "decl_index": [],
        "decl_count": 1,
        "decls": [{"origin": 0, "node": 0, "node_lo": 0, "page": 0}],
        "source_pages": [],
    });
    let key = QueryKey::new(
        schema::layer("A03").unwrap(),
        vec![
            ("source-snapshot".into(), s.id().as_str().into()),
            ("grammar-id".into(), p.grammar_id().as_str().into()),
            (
                "parser-version".into(),
                ace_syntax_runtime::PARSER_ABI_VERSION.to_string(),
            ),
            ("page".into(), "manifest".into()),
        ],
    );
    sas.inner()
        .put_artifact(PutRequest {
            layer: "A03",
            key,
            kind: ChunkKind::CstManifest,
            payload: serde_json::to_vec(&m).unwrap(),
            edges: vec![],
            project: None,
        })
        .unwrap();
    match sas.open_cst(&p, &s) {
        Err(_) | Ok(None) => {}
        Ok(Some(v)) => {
            assert!(
                v.decl_range(0).is_err(),
                "one-byte source must not expose a successful [9,10) declaration span"
            );
        }
    }
}

fn put_all(
    sas: &SyntaxArtifactStore,
    p: &GrammarProgram,
    s: &Snapshot,
) -> Result<String, ace_syntax_artifacts::AdapterError> {
    let cs = CompiledScanner::compile(p).unwrap();
    let out = parse_snapshot(p, &cs, s, &Budget::default()).unwrap();
    let t = sas.put_tokens(p, s, &out.tokens)?;
    sas.put_cst(p, s, &out.tree, &t)
}

// ---- F787-R4-02：子树读取完整 admission（永久 oracle） ----
//
// 反例形态：经正常 put_artifact 构造**有效 frame/hash** 的页与 manifest——
// 根节点 span 合法（decl_range 通过 = accepted control），但可达子树内含
// 越源/越父/异归属/坏字段的 child。decl_subtree 必须在实际读取入口拒绝；
// 修复前只有根级检查（F787-R3-05），坏 child 直接进入返回树。

/// 植入一个单声明 manifest：页 payload 由节点/边值表构造（32B 节点 =
/// kind,start,len,first_child,child_count,expected,decl,flags）。
fn plant_single_decl_manifest(
    sas: &SyntaxArtifactStore,
    p: &GrammarProgram,
    s: &Snapshot,
    total_len: u64,
    nodes: &[[u32; 8]],
    edges: &[(u32, u32)],
) {
    use ace_artifacts::{codec::ChunkKind, identity::QueryKey, schema, store::PutRequest};
    let mut payload = Vec::new();
    payload.extend_from_slice(&(nodes.len() as u32).to_le_bytes());
    payload.extend_from_slice(&(edges.len() as u32).to_le_bytes());
    for nd in nodes {
        for v in nd.iter() {
            payload.extend_from_slice(&v.to_le_bytes());
        }
    }
    for (c, f) in edges {
        payload.extend_from_slice(&c.to_le_bytes());
        payload.extend_from_slice(&f.to_le_bytes());
    }
    let cid = ace_artifacts::codec::framed_chunk_id(ChunkKind::CstPage, &payload).to_string();
    sas.inner()
        .put_artifact(PutRequest {
            layer: "A03",
            key: QueryKey::new(
                schema::layer("A03").unwrap(),
                vec![(
                    "page".into(),
                    format!("r4-{}", s.id().as_str()[..6].to_owned()),
                )],
            ),
            kind: ChunkKind::CstPage,
            payload,
            edges: vec![],
            project: None,
        })
        .unwrap();
    let m = serde_json::json!({
        "snapshot": s.id().as_str(),
        "grammar": p.grammar_id().as_str(),
        "total_len": total_len,
        "diags": [],
        "root_edges": [],
        "pages": [{"chunk": cid, "first_node": 0, "node_count": nodes.len() as u32}],
        "decl_index": [],
        "decl_count": 1,
        "decls": [{"origin": 0, "node": 0, "node_lo": 0, "page": 0}],
        "source_pages": [],
    });
    let key = QueryKey::new(
        schema::layer("A03").unwrap(),
        vec![
            ("source-snapshot".into(), s.id().as_str().into()),
            ("grammar-id".into(), p.grammar_id().as_str().into()),
            (
                "parser-version".into(),
                ace_syntax_runtime::PARSER_ABI_VERSION.to_string(),
            ),
            ("page".into(), "manifest".into()),
        ],
    );
    sas.inner()
        .put_artifact(PutRequest {
            layer: "A03",
            key,
            kind: ChunkKind::CstManifest,
            payload: serde_json::to_vec(&m).unwrap(),
            edges: vec![],
            project: None,
        })
        .unwrap();
}

fn decl_kinds(p: &GrammarProgram) -> (u32, u32) {
    let rule = ace_syntax_runtime::program::RULE_KIND_BASE
        + p.rules.iter().position(|r| r.name == "api-decl").unwrap() as u32;
    let tok = ace_syntax_runtime::program::TOKEN_KIND_BASE
        + p.tokens.iter().position(|t| t.name == "api").unwrap() as u32;
    (rule, tok)
}

/// 反例：根合法（[0,1)，decl_range 通过），child span [9,10) 越出源长 1——
/// 子树入口必须拒绝（修复前 Ok 返回坏树）。区分性 control：prefix 内
/// decl_range(0) == (0,1) 通过（根级单点检查的语义范围）。
#[test]
fn syntax_admission_subtree_child_span_outside_source_rejected() {
    let root = scratch("adm-r4-childsrc");
    let p = mini_grammar();
    let s = Snapshot::from_text(&Rc::new(PageStore::new()), "x");
    let sas = SyntaxArtifactStore::open(&root, "adm").unwrap();
    let (rule, tok) = decl_kinds(&p);
    plant_single_decl_manifest(
        &sas,
        &p,
        &s,
        1,
        &[
            [rule, 0, 1, 0, 1, 0, 1, 0], // 根：合法 rule，span [0,1)
            [tok, 9, 1, 1, 0, 0, 1, 0],  // child：越源 [9,10)
        ],
        &[(1, u32::MAX)],
    );
    match sas.open_cst(&p, &s) {
        Err(_) | Ok(None) => {}
        Ok(Some(v)) => {
            assert_eq!(
                v.decl_range(0).unwrap(),
                (0, 1),
                "root-only check is an accepted control"
            );
            assert!(
                v.decl_subtree(0).is_err(),
                "invalid child span must not escape the actual subtree read entry"
            );
        }
    }
}

/// 反例：child span 在源内（total_len=8）但越出父 span [0,1)——父子约束
/// 拒绝（与越源反例区分：绝对跨度界不触发）。
#[test]
fn syntax_admission_subtree_child_beyond_parent_span_rejected() {
    let root = scratch("adm-r4-childpar");
    let p = mini_grammar();
    let s = Snapshot::from_text(&Rc::new(PageStore::new()), "x");
    let sas = SyntaxArtifactStore::open(&root, "adm").unwrap();
    let (rule, tok) = decl_kinds(&p);
    plant_single_decl_manifest(
        &sas,
        &p,
        &s,
        8,
        &[
            [rule, 0, 1, 0, 1, 0, 1, 0], // 父：[0,1)
            [tok, 2, 2, 1, 0, 0, 1, 0],  // child：[2,4) 在源内、在父外
        ],
        &[(1, u32::MAX)],
    );
    match sas.open_cst(&p, &s) {
        Err(_) | Ok(None) => {}
        Ok(Some(v)) => {
            assert_eq!(v.decl_range(0).unwrap(), (0, 1));
            assert!(
                v.decl_subtree(0).is_err(),
                "child span escaping its parent must be rejected at load admission"
            );
        }
    }
}

/// 反例：child 归属其它声明槽位（decl=2 ≠ 根 decl=1）——归属约束拒绝
/// （跨声明边不得进入子树）。spans/字段均合法，区分归属维度。
#[test]
fn syntax_admission_subtree_foreign_decl_child_rejected() {
    let root = scratch("adm-r4-childown");
    let p = mini_grammar();
    let s = Snapshot::from_text(&Rc::new(PageStore::new()), "x");
    let sas = SyntaxArtifactStore::open(&root, "adm").unwrap();
    let (rule, tok) = decl_kinds(&p);
    plant_single_decl_manifest(
        &sas,
        &p,
        &s,
        4,
        &[
            [rule, 0, 4, 0, 1, 0, 1, 0], // 根：[0,4) decl=1
            [tok, 0, 4, 1, 0, 0, 2, 0],  // child：[0,4) 合法，但 decl=2
        ],
        &[(1, u32::MAX)],
    );
    match sas.open_cst(&p, &s) {
        Err(_) | Ok(None) => {}
        Ok(Some(v)) => {
            assert!(
                v.decl_subtree(0).is_err(),
                "foreign-decl child must be rejected"
            );
        }
    }
}

/// 反例：边字段 id 越出 grammar 字段名表——字段适用性拒绝（spans/归属
/// 均合法，区分字段维度）。
#[test]
fn syntax_admission_subtree_bad_field_rejected() {
    let root = scratch("adm-r4-field");
    let p = mini_grammar();
    let s = Snapshot::from_text(&Rc::new(PageStore::new()), "x");
    let sas = SyntaxArtifactStore::open(&root, "adm").unwrap();
    let (rule, tok) = decl_kinds(&p);
    assert!(p.field_names.len() < 900);
    plant_single_decl_manifest(
        &sas,
        &p,
        &s,
        4,
        &[[rule, 0, 4, 0, 1, 0, 1, 0], [tok, 0, 4, 1, 0, 0, 1, 0]],
        &[(1, 999)],
    );
    match sas.open_cst(&p, &s) {
        Err(_) | Ok(None) => {}
        Ok(Some(v)) => {
            assert!(
                v.decl_subtree(0).is_err(),
                "edge field outside the grammar field table must be rejected"
            );
        }
    }
}

/// 正控（跨页目录 + 非零 origin + 超预算声明独占页）：真实树走规范形
/// put_cst——超大首声明（> CST_CANON_PAGE_NODES 节点）独占页 0，后续
/// 声明落页 1（非零 origin）；其子树读取必须通过 admission 并与 fresh
/// outline 全等。
#[test]
fn syntax_admission_subtree_cross_page_nonzero_origin_ok() {
    use ace_syntax_artifacts::CST_CANON_PAGE_NODES;
    let p = mini_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    // 每参数 ≈4 节点（param/name/type）——5000 参数 > 4096 节点预算。
    let mut text = String::from("api big(");
    for i in 0..5000 {
        if i > 0 {
            text.push_str(", ");
        }
        text.push_str(&format!("p{i}: i32"));
    }
    text.push_str(") -> i32;\napi small(x: i32) -> i32;\n");
    let snap = Snapshot::from_text(&Rc::new(PageStore::new()), &text);
    let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();
    let root = scratch("adm-r4-crosspage");
    let sas = SyntaxArtifactStore::open(&root, "adm").unwrap();
    let t = sas.put_tokens(&p, &snap, &out.tokens).unwrap();
    sas.put_cst(&p, &snap, &out.tree, &t).unwrap();
    let view = sas.open_cst(&p, &snap).unwrap().expect("manifest");
    assert!(
        view.debug_page_chunk(1).is_some(),
        "oversized first decl must leave the small decl on a later page"
    );
    let _ = CST_CANON_PAGE_NODES;
    let st = view
        .decl_subtree(1)
        .expect("cross-page decl subtree admission");
    let items = ace_syntax_runtime::outline::outline(&p, &st, &snap).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "small");
    assert_eq!(items[0].syntax_ok, true);
}

// ---- F787-R5-01 零宽 epsilon 坐标合同（生产者/准入统一）----
// 空产生式（如空参数 args）给出**已消费 token 终点**上的零宽节点；准入
// 父子约束据此放行合法 epsilon，同时保留真实越源/越父/错归属拒绝。

/// 正控：合法空参/非空参调用经真实 parse→put→decl_subtree 全链可读（R5-01）。
/// auto-core 的 `args` 规则在空参时匹配空产生式，产生零宽 epsilon——不得被
/// 父子跨度误拒。
#[test]
fn syntax_admission_epsilon_empty_and_nonempty_args_survive_subtree() {
    let core_src = std::fs::read_to_string(format!(
        "{}/../../grammars/auto-core.at",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let p = compile_grammar(&core_src).unwrap();
    let cs = CompiledScanner::compile(&p).unwrap();
    for (tag, txt) in [
        ("no-args", "fn main() int { return f() }\n"),
        ("one-arg", "fn main() int { return f(1) }\n"),
    ] {
        let snap = Snapshot::from_text(&Rc::new(PageStore::new()), txt);
        let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();
        let root = scratch(&format!("adm-r5-eps-{tag}"));
        let sas = SyntaxArtifactStore::open(&root, "adm").unwrap();
        let t = sas.put_tokens(&p, &snap, &out.tokens).unwrap();
        sas.put_cst(&p, &snap, &out.tree, &t).unwrap();
        let v = sas.open_cst(&p, &snap).unwrap().expect("manifest");
        let st = v.decl_subtree(0).unwrap_or_else(|e| {
            panic!("{tag}: legal empty/nonempty args must survive admission: {e:?}")
        });
        assert!(st.node_total() > 0, "{tag}: subtree must be non-empty");
        let _ = std::fs::remove_dir_all(&root);
    }
}

/// 反控：伪造零宽 child（start=0/len=0 占位形）挂在后位父 span 下——
/// 准入仍须拒绝（区分合法 epsilon 的真实坐标 vs 坏跨度占位）。
#[test]
fn syntax_admission_zero_width_placeholder_outside_parent_rejected() {
    let root = scratch("adm-r5-zw-bad");
    let p = mini_grammar();
    let s = Snapshot::from_text(&Rc::new(PageStore::new()), "x");
    let sas = SyntaxArtifactStore::open(&root, "adm").unwrap();
    let (rule, tok) = decl_kinds(&p);
    // 父 [23,26)；零宽 child start=0/len=0（旧占位形）→ abs [0,0) 在父外。
    plant_single_decl_manifest(
        &sas,
        &p,
        &s,
        32,
        &[
            [rule, 23, 3, 0, 1, 0, 1, 0], // 父
            [tok, 0, 0, 1, 0, 0, 1, 0],   // 零宽占位 child
        ],
        &[(1, u32::MAX)],
    );
    match sas.open_cst(&p, &s) {
        Err(_) | Ok(None) => {}
        Ok(Some(v)) => {
            assert!(
                v.decl_subtree(0).is_err(),
                "zero-width placeholder outside parent span must still be rejected"
            );
        }
    }
}

/// 正控：零宽 epsilon 位于父 span 内（真实坐标）——准入放行。
#[test]
fn syntax_admission_zero_width_epsilon_inside_parent_accepted() {
    let root = scratch("adm-r5-zw-ok");
    let p = mini_grammar();
    let s = Snapshot::from_text(&Rc::new(PageStore::new()), "x");
    let sas = SyntaxArtifactStore::open(&root, "adm").unwrap();
    let (rule, tok) = decl_kinds(&p);
    // 父 [23,26)；零宽 child start=25/len=0（已消费 token 终点形）→ 父内。
    plant_single_decl_manifest(
        &sas,
        &p,
        &s,
        32,
        &[[rule, 23, 3, 0, 1, 0, 1, 0], [tok, 25, 0, 1, 0, 0, 1, 0]],
        &[(1, u32::MAX)],
    );
    match sas.open_cst(&p, &s) {
        Err(_) | Ok(None) => {}
        Ok(Some(v)) => {
            v.decl_subtree(0)
                .expect("zero-width epsilon with real in-parent coordinate must be accepted");
        }
    }
}
