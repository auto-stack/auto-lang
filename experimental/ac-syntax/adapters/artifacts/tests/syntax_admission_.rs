//! V-10 数据拒绝（PLAN-787 T-06）：有效 frame 替换错 chunk、坏 offset/ref、
//! 坏 manifest 身份、缺页 —— 真读取入口受控拒绝/重算，无 panic/错树成功。

use ac_syntax_artifact_adapter::{AdapterError, SyntaxArtifactStore};
use ac_syntax_grammar::compile as compile_grammar;
use ac_syntax_rt::lexer::CompiledScanner;
use ac_syntax_rt::parse::parse_snapshot;
use ac_syntax_rt::program::{Budget, GrammarProgram};
use ac_syntax_rt::snapshot::{PageStore, Snapshot};
use std::rc::Rc;

fn mini_grammar() -> GrammarProgram {
    let src = std::fs::read_to_string(format!("{}/../../grammars/interface-mini-v1.at", env!("CARGO_MANIFEST_DIR"))).unwrap();
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


/// 递归收集 CAS chunk 文件（blocks/blobs/<shard>/<chunk>）。
fn chunk_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(root.join("blocks/blobs")) {
        for shard in rd.flatten() {
            if let Ok(inner) = std::fs::read_dir(shard.path()) {
                for f in inner.flatten() {
                    let p = f.path();
                    if p.is_file() {
                        out.push(p);
                    }
                }
            }
        }
    }
    out.sort();
    out
}


/// manifest 页 0 的 chunk 文件路径（blocks/blobs/<shard>/<chunk>）。
fn page_chunk_path(sas: &SyntaxArtifactStore, root: &std::path::Path, p: &GrammarProgram, snap: &Snapshot) -> std::path::PathBuf {
    let view = sas.open_cst(p, snap).unwrap().expect("manifest");
    let chunk = view.debug_page_chunk(0).expect("page 0");
    // shard = 完整 chunk id 的前两字符（含 chk 前缀）。
    root.join("blocks/blobs").join(&chunk[..2]).join(&chunk)
}

fn setup(root: &std::path::Path) -> (GrammarProgram, Snapshot, ac_syntax_rt::parse::ParseOutput) {
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
            assert!(matches!(e, AdapterError::Store(_) | AdapterError::Admissions(_)), "panic-free controlled: {e}");
            saw_controlled = true;
        }
    }
    assert!(saw_controlled, "corrupted page must surface as controlled error");
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
            assert!(matches!(e, AdapterError::Admissions(_)), "controlled miss: {e}");
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
    assert!(sas.open_cst(&p, &other_snap).unwrap().is_none(), "different snapshot must key-miss");
    // 不同 grammar（同快照）→ key miss。
    let src2 = std::fs::read_to_string(format!("{}/../../grammars/interface-mini-v1.at", env!("CARGO_MANIFEST_DIR"))).unwrap()
        .replace(r#"{ literal: "api" }"#, r#"{ literal: "endpoint" }"#);
    let p2 = compile_grammar(&src2).unwrap();
    let _ = out;
    let snap = Snapshot::from_text(&Rc::new(PageStore::new()), "api a(x: i32) -> i32;\napi b(y: i32) -> i32;\napi c(z: i32) -> i32;\n");
    assert!(sas.open_cst(&p2, &snap).unwrap().is_none(), "different grammar must key-miss");
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
    let reframed = ac_artifact_probe::codec::frame_chunk(
        ac_artifact_probe::codec::ChunkKind::CstPage,
        payload,
    );
    std::fs::write(victim, reframed).unwrap();

    let sas = SyntaxArtifactStore::open(&root, "adm").unwrap();
    let view = sas.open_cst(&p, &snap).unwrap().expect("manifest readable");
    let mut saw_bad = false;
    for i in 0..view.decl_count() {
        if let Err(e) = view.decl_subtree(i) {
            // 有效 frame + 坏内容：unframe/decode 层受控拒绝（Admissions 或 Store 校验）。
            assert!(matches!(e, AdapterError::Admissions(_) | AdapterError::Store(_)), "{e}");
            saw_bad = true;
        }
    }
    assert!(saw_bad, "truncated page payload must be rejected");
}
