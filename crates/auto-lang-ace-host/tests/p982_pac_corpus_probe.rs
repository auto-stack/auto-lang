//! PLAN-982（B-CONFIG-I，E7 探针）：**真实 pac.at 语料的 ACE 求值覆盖面**。
//!
//! 不是能力声明：这是把 auto-man 仓的真实配置文档喂给 `AutoConfig` 的显式 ACE
//! 引擎，逐份记录"过/不过 + 首个诊断"，用来决定 E7 的切换面与缺口顺序。
//! 通过 `P982_PAC_CORPUS=<auto-man 仓库根>` 指定语料根；缺省时跳过。

use std::path::{Path, PathBuf};

use auto_lang_core::ace_config::ConfigEngineChoice;
use auto_lang_core::config::AutoConfig;
use auto_val::Obj;

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().and_then(|n| n.to_str()) == Some("target") {
                continue;
            }
            collect(&p, out);
        } else if p.file_name().and_then(|n| n.to_str()) == Some("pac.at") {
            out.push(p);
        }
    }
}

#[test]
fn p982_pac_corpus_ace_coverage() {
    let Ok(root) = std::env::var("P982_PAC_CORPUS") else {
        eprintln!("P982_PAC_CORPUS not set; corpus probe skipped");
        return;
    };
    let _ = auto_lang_ace_host::install();
    let mut files = Vec::new();
    collect(Path::new(&root), &mut files);
    files.sort();
    if let Ok(one) = std::env::var("P982_PAC_ONE") {
        files.retain(|f| f.display().to_string().contains(&one));
    }
    let mut ok = 0usize;
    let mut fail: Vec<(String, String)> = Vec::new();
    let mut diff: Vec<(String, String)> = Vec::new();
    let mut skipped_bad = 0usize;
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        let rel = path
            .strip_prefix(&root)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| path.display().to_string());
        // 严格口径：**不注入**上下文，ACE 与 legacy 的投影必须逐字相等。
        // （模板仓文档的 `${name}` 在无注入时 ACE 具名拒、legacy 留字面文本，
        //  这是已知且故意的分歧，另计在 `templates` 计数里。）
        let ace =
            AutoConfig::from_code_with_engine(src.as_str(), &Obj::new(), ConfigEngineChoice::AceAv);
        let legacy = AutoConfig::from_code_with_engine(
            src.as_str(),
            &Obj::new(),
            ConfigEngineChoice::LegacyVm,
        );
        match (ace, legacy) {
            (Ok(_), Err(_)) => skipped_bad += 1,
            (Ok(ace_cfg), Ok(legacy_cfg)) => {
                ok += 1;
                if ace_cfg.root != legacy_cfg.root {
                    diff.push((
                        rel.clone(),
                        format!(
                            "ace={:?}\n         legacy={:?}",
                            ace_cfg.root, legacy_cfg.root
                        ),
                    ));
                }
            }
            (Err(e), _) => fail.push((rel, format!("{e}"))),
        }
    }
    eprintln!(
        "pac corpus: {} ok ({} disagree with legacy), {} failed, {} ace-only (legacy rejects them), {} total",
        ok,
        diff.len(),
        fail.len(),
        skipped_bad,
        files.len()
    );
    for (file, err) in fail.iter().take(40) {
        eprintln!("FAIL {file}: {err}");
    }
    for (file, d) in diff.iter().take(6) {
        eprintln!("DIFF {file}: {d}");
    }
    assert!(!files.is_empty(), "corpus must not be empty");
    // 探针也要能红（B-CONFIG-R F-982B-06）：语料里 140 份以上必须与 legacy 逐字
    // 相等；任何一份出现结构分歧即失败（不只是打印数字）。
    assert!(
        diff.is_empty(),
        "{} corpus documents disagree with the legacy engine; first: {:?}",
        diff.len(),
        diff.first()
    );
    assert!(
        ok >= 140,
        "the managed corpus must stay at 140+ structurally equal documents, got {ok}"
    );
}
