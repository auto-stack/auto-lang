//! V-05 core 投影对照（PLAN-787 T-05）：CST 严格投影 vs 旧手写 parser（oracle）。
//! - 旧 parser 接受的源 → 投影必须接受且 AST（结构/顺序/span/字段）逐项一致；
//! - 旧 parser 拒绝的源 → 投影必须拒绝（error/missing 或 literal-range）；
//! - 投影真实读 CST（本包不调用旧 parser 参与投影路径，仅测试层作 oracle）。

use ac_syntax_core_adapter::{project_strict, CoreKinds, ProjectionError};
use ac_syntax_grammar::compile;
use ac_syntax_rt::lexer::CompiledScanner;
use ac_syntax_rt::parse::parse_snapshot;
use ac_syntax_rt::program::{Budget, GrammarProgram};
use ac_syntax_rt::snapshot::{PageStore, Snapshot};
use auto_ac_prototype::source::parse_program;
use std::rc::Rc;

pub fn core_grammar() -> GrammarProgram {
    let src = std::fs::read_to_string(format!(
        "{}/../../grammars/auto-core.at",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    compile(&src).unwrap()
}

fn project(p: &GrammarProgram, cs: &CompiledScanner, src: &str) -> Result<String, ProjectionError> {
    let snap = Snapshot::from_text(&Rc::new(PageStore::new()), src);
    let out = parse_snapshot(p, cs, &snap, &Budget::default()).unwrap();
    let k = CoreKinds::from_program(p);
    let program = project_strict(p, &k, &out.tree, &snap)?;
    Ok(format!("{program:?}"))
}

pub fn fixtures(sub: &str) -> Vec<(String, String)> {
    let dir = format!(
        "{}/../../../ac-core/fixtures/source/{sub}",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut v: Vec<(String, String)> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {dir}: {e}"))
        .filter_map(|e| {
            let p = e.ok()?.path();
            if p.extension()? == "at" {
                Some((
                    p.file_name()?.to_string_lossy().into_owned(),
                    std::fs::read_to_string(&p).ok()?,
                ))
            } else {
                None
            }
        })
        .collect();
    v.sort();
    v
}

#[test]
fn core_projection_valid_corpus_identical() {
    let p = core_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    let mut mismatches = Vec::new();
    let mut passed = 0;
    for (name, src) in fixtures("valid") {
        let oracle = match parse_program(&src) {
            Ok(o) => o,
            Err(e) => {
                mismatches.push(format!("{name}: ORACLE rejected valid fixture?? {e:?}"));
                continue;
            }
        };
        match project(&p, &cs, &src) {
            Ok(got) => {
                let want = format!("{oracle:?}");
                if got != want {
                    mismatches.push(format!("{name}: AST mismatch"));
                } else {
                    passed += 1;
                }
            }
            Err(e) => mismatches.push(format!("{name}: projection refused: {e:?}")),
        }
    }
    assert!(passed > 0, "no valid fixtures compared: {mismatches:?}");
    assert!(
        mismatches.is_empty(),
        "{} mismatches:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

#[test]
fn core_projection_invalid_corpus_accept_reject_parity() {
    let p = core_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    let mut mismatches = Vec::new();
    let mut agree = 0;
    for (name, src) in fixtures("invalid") {
        let oracle_ok = parse_program(&src).is_ok();
        let ours = project(&p, &cs, &src);
        match (oracle_ok, ours) {
            (true, Ok(_)) => agree += 1,
            (true, Err(e)) => mismatches.push(format!(
                "{name}: oracle accepted, projection refused: {e:?}"
            )),
            (false, Err(_)) => agree += 1,
            (false, Ok(_)) => {
                mismatches.push(format!("{name}: oracle rejected, projection accepted"))
            }
        }
    }
    assert!(agree > 0);
    assert!(
        mismatches.is_empty(),
        "{} mismatches:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

#[test]
fn core_projection_literal_range_refuses() {
    let p = core_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    for src in [
        "fn f() int { return 2147483648 }",
        "fn f() int { return -2147483649 }",
    ] {
        assert!(
            matches!(
                project(&p, &cs, src),
                Err(ProjectionError::LiteralRange { .. })
            ),
            "{src}"
        );
    }
    for src in [
        "fn f() int { return 2147483647 }",
        "fn f() int { return -2147483648 }",
    ] {
        assert!(project(&p, &cs, src).is_ok(), "{src} must be in-range");
    }
}

/// F787-R2-08 反控：合法 core 深输入不受固定句法深度上限拒绝——投影以
/// 节点访问预算受控（PROJECTION_NODE_BUDGET）。大显式线程栈仅隔离测试
/// 环境，语义上限 = 预算。平链 / 深嵌套 / 深块 / 深调用实参全路径。
#[test]
fn supported_core_deep_inputs_project_without_fixed_depth_cutoff() {
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(|| {
            let p = core_grammar();
            let kinds = ac_syntax_core_adapter::CoreKinds::from_program(&p);
            let cs = ac_syntax_rt::lexer::CompiledScanner::compile(&p).unwrap();
            // 平链（左脊 2200）。
            let text = format!("fn f() int {{ return 1{} }}", " + 1".repeat(2200));
            assert!(
                auto_ac_prototype::source::parse_program(&text).is_ok(),
                "legacy core accepts"
            );
            let s = ac_syntax_rt::snapshot::Snapshot::from_text(
                &std::rc::Rc::new(ac_syntax_rt::snapshot::PageStore::new()),
                &text,
            );
            let out = ac_syntax_rt::parse::parse_snapshot(
                &p,
                &cs,
                &s,
                &ac_syntax_rt::program::Budget::default(),
            )
            .unwrap();
            assert!(!out.tree.has_error_nodes());
            let r = ac_syntax_core_adapter::project_strict(&p, &kinds, &out.tree, &s);
            assert!(r.is_ok(), "deep chain must project, got {:?}", r.err());
            drop(out);
            // 深嵌套括号（4096+）。
            let text2 = format!(
                "fn g() int {{ return {}1{} }}",
                "(".repeat(4096 + 64),
                ")".repeat(4096 + 64)
            );
            let s2 = ac_syntax_rt::snapshot::Snapshot::from_text(
                &std::rc::Rc::new(ac_syntax_rt::snapshot::PageStore::new()),
                &text2,
            );
            let out2 = ac_syntax_rt::parse::parse_snapshot(
                &p,
                &cs,
                &s2,
                &ac_syntax_rt::program::Budget::default(),
            )
            .unwrap();
            assert!(!out2.tree.has_error_nodes());
            let r2 = ac_syntax_core_adapter::project_strict(&p, &kinds, &out2.tree, &s2);
            assert!(r2.is_ok(), "deep nesting must project, got {:?}", r2.err());
            drop(out2);
            // 深块（2000 语句）+ 深调用实参链。
            let mut text3 = String::from("fn h() int {");
            for i in 0..2000 {
                text3.push_str(&format!(" let v{i} = {i};"));
            }
            text3.push_str(" return v0 }");
            let s3 = ac_syntax_rt::snapshot::Snapshot::from_text(
                &std::rc::Rc::new(ac_syntax_rt::snapshot::PageStore::new()),
                &text3,
            );
            let out3 = ac_syntax_rt::parse::parse_snapshot(
                &p,
                &cs,
                &s3,
                &ac_syntax_rt::program::Budget::default(),
            )
            .unwrap();
            let r3 = ac_syntax_core_adapter::project_strict(&p, &kinds, &out3.tree, &s3);
            assert!(r3.is_ok(), "deep block must project, got {:?}", r3.err());
        })
        .unwrap()
        .join()
        .unwrap();
}
