//! PLAN-927 T-02 探针 + V-01 差分（旧手写 parser 仅作迁移差分之一；冻结
//! 语料的独立 oracle 在 T-09 的文件化 case 里）。
//!
//! 差分合同：对每个 (语料, profile)，`parse_program_with`（旧）与
//! `cst::parse_program_via_cst`（新：共享CST→严格投影）必须产生全等的
//! `Program`（含全部字段/span）或全等的首个 `Diagnostic`
//! (stage/code/message/span)。逐 profile 全跑——不改分母、不挑选。

use ace_core::source::cst;
use ace_core::source::{parse_program_with, SourceProfile};

const SEVEN: [SourceProfile; 7] = [
    SourceProfile::Core,
    SourceProfile::Scalar,
    SourceProfile::Text,
    SourceProfile::Record,
    SourceProfile::TextRecord,
    SourceProfile::EntryUnit,
    SourceProfile::HostConsole,
];

fn assert_same(src: &str, profile: SourceProfile, what: &str) {
    let old = ace_core::source::parse_program_with_handwritten(src, profile);
    let new = cst::parse_program_via_cst(src, profile);
    match (&old, &new) {
        (Ok(a), Ok(b)) => {
            assert_eq!(
                format!("{a:?}"),
                format!("{b:?}"),
                "{what}: AST mismatch under {:#?}\n--old--\n{a:#?}\n--new--\n{b:#?}",
                profile
            );
        }
        (Err(d), Err(e)) => {
            assert_eq!(
                (format!("{}", d.stage.as_str()), &d.code, &d.message, d.span.start, d.span.end),
                (format!("{}", e.stage.as_str()), &e.code, &e.message, e.span.start, e.span.end),
                "{what}: diagnostic mismatch under {:#?}\nsrc={src:?}\nold={d:?}\nnew={e:?}",
                profile
            );
        }
        _ => panic!(
            "{what}: outcome mismatch under {:#?}\nsrc={src:?}\nold={old:?}\nnew={new:?}",
            profile
        ),
    }
}

// ---------------------------------------------------------------- T-02 探针

/// 最小整链：有 owned 文本的 record + 方法 → 共享CST → AST →（与旧前端
/// 全等的）语义检查入口材料。证明无环/无重扫投影/预算可行（AC-02/04/08）。
#[test]
fn probe_whole_path_text_record() {
    let src = "type Note {\n    body Str\n    fn len(selfless int) int { return 0 }\n}\nfn main() int {\n    let n Note = Note { body: \"hi\" }\n    return n.len()\n}\n";
    assert_same(src, SourceProfile::TextRecord, "text-record whole-path");
}

#[test]
fn probe_scalar_basics() {
    assert_same("fn main() int { return 2 + 3 * 4 }", SourceProfile::Scalar, "scalar b01");
    assert_same("fn main() int { return -2147483648 }", SourceProfile::Scalar, "i32 min");
    assert_same(
        "fn main() bool { let x bool = true && false\n return x || !false }",
        SourceProfile::Scalar,
        "scalar bool ops",
    );
    assert_same("fn main() int { for x < 10 { break } return x }", SourceProfile::Scalar, "cond for");
    assert_same("fn main() int { while x { } return 0 }", SourceProfile::Scalar, "scalar while reject");
    assert_same("fn main() int { return 1 - 2 }", SourceProfile::Scalar, "scalar minus");
    assert_same("fn main() int { return 1 - 2 }", SourceProfile::Core, "core minus reject");
}

#[test]
fn probe_typed_budget_outcome() {
    use cst::{parse_front, FrontBudget, FrontKind, FrontOutcome, FrontRequest};
    let src = "fn main() int { return 1 }";
    let out = parse_front(FrontRequest {
        src,
        profile: SourceProfile::Scalar,
        kind: FrontKind::WholeModule,
        budget: FrontBudget::WORK_OWNED_64MIB,
    });
    match out {
        FrontOutcome::Ok(ok) => {
            assert!(!ok.grammar_id.is_empty());
            assert_eq!(ok.caps_version, "ac-source-front/1");
            assert!(!ok.program.items.is_empty());
        }
        other => panic!("expected Ok, got {other:?}"),
    }
    // 缩预算反控：1 字节工作集 → typed 资源态（不是源码诊断）。
    let out = parse_front(FrontRequest {
        src,
        profile: SourceProfile::Scalar,
        kind: FrontKind::WholeModule,
        budget: FrontBudget { work_bytes: 1, max_parse_depth: 1 << 20 },
    });
    match out {
        FrontOutcome::ResourceExhausted { .. } => {}
        other => panic!("expected ResourceExhausted, got {other:?}"),
    }
}

// ---------------------------------------------------------------- V-01 差分

/// 冻结语料 × 七 profile 全量差分（ace-m1/src 语料 + 边界用例）。
#[test]
fn differential_corpus_all_profiles() {
    let corpus_dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../crates/auto-lang/test/cases/ace-m1/src"
    );
    let mut checked = 0usize;
    let mut failures: Vec<String> = Vec::new();
    let entries = std::fs::read_dir(corpus_dir).expect("corpus dir");
    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "at").unwrap_or(false))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "corpus empty");
    for f in &files {
        let src = std::fs::read_to_string(f).expect("read corpus file");
        let name = f.file_name().unwrap().to_string_lossy().to_string();
        for profile in SEVEN {
            checked += 1;
            let old = ace_core::source::parse_program_with_handwritten(&src, profile);
            let new = cst::parse_program_via_cst(&src, profile);
            let agree = match (&old, &new) {
                (Ok(a), Ok(b)) => format!("{a:?}") == format!("{b:?}"),
                (Err(d), Err(e)) => {
                    d.stage.as_str() == e.stage.as_str()
                        && d.code == e.code
                        && d.message == e.message
                        && d.span.start == e.span.start
                        && d.span.end == e.span.end
                }
                _ => false,
            };
            if !agree {
                failures.push(format!(
                    "{name} @ {:#?}: old={:?} new={:?}",
                    profile,
                    old.map_err(|d| (d.code.clone(), d.message.clone(), d.span.start, d.span.end)),
                    new.map_err(|d| (d.code.clone(), d.message.clone(), d.span.start, d.span.end)),
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "differential failures ({}/{} disagree):\n{}",
        failures.len(),
        checked,
        failures.join("\n")
    );
}

/// 冻结的拒绝/边界小语料（T-01 清单抽样；逐字节诊断保真）。
#[test]
fn differential_edge_rejects() {
    let cases: &[(&str, SourceProfile)] = &[
        // 词法层
        ("fn main() int { return 1 & 2 }", SourceProfile::Scalar),
        ("fn main() int { return \"abc }", SourceProfile::Scalar),
        ("fn main() int { return \"abc }", SourceProfile::Text),
        ("fn main() int { return 'a }", SourceProfile::Text),
        ("fn main() int { return '\\q' }", SourceProfile::Text),
        ("fn main() int { return '\\u{zz}' }", SourceProfile::Text),
        ("fn main() int { return '' }", SourceProfile::Text),
        ("fn main() int { return 'ab' }", SourceProfile::Text),
        ("fn main() int { return 1i32 }", SourceProfile::Scalar),
        ("fn main() int { return 2147483648 }", SourceProfile::Scalar),
        ("fn main() int { return -2147483649 }", SourceProfile::Scalar),
        ("fn main() int { return /* c }", SourceProfile::Scalar),
        // 结构层
        ("", SourceProfile::Scalar),
        (";", SourceProfile::Scalar),
        ("// only comment", SourceProfile::Scalar),
        ("fn main() int { return 0 } fn", SourceProfile::Scalar),
        ("fn main() int { return 0 } fn b() int", SourceProfile::Scalar),
        ("use x", SourceProfile::Scalar),
        ("struct x", SourceProfile::Scalar),
        ("static x", SourceProfile::Scalar),
        ("type P { x int }", SourceProfile::Scalar),
        ("enum E { A }", SourceProfile::Scalar),
        ("let x = 1", SourceProfile::Scalar),
        ("fn main() int { let x = }", SourceProfile::Scalar),
        ("fn main() int { let x str = \"a\" return 0 }", SourceProfile::Scalar),
        ("fn main() int { let x int = 1 return x }", SourceProfile::Scalar),
        ("fn main() int { return f(1 2) }", SourceProfile::Scalar),
        ("fn main() int { return f(x: ) }", SourceProfile::Scalar),
        ("fn main() int { return (1 }", SourceProfile::Scalar),
        ("fn main() int { return 1 + }", SourceProfile::Scalar),
        ("fn main() int { x = 1 return 0 }", SourceProfile::Scalar),
        ("fn main() int { return x.1 }", SourceProfile::Scalar),
        ("fn main() int { return -x }", SourceProfile::Scalar),
        ("fn main() int { return -x }", SourceProfile::Core),
        ("fn main() int { return !x }", SourceProfile::Core),
        ("fn main() int { return +x }", SourceProfile::Scalar),
        ("fn main() int { for x in xs { } return 0 }", SourceProfile::Scalar),
        ("fn main() int { return 0 }", SourceProfile::Core),
        ("fn main() int { for x { break } return 0 }", SourceProfile::Core),
        ("fn main() int { while x { } return 0 }", SourceProfile::Core),
        // record 面
        ("type P { x int }", SourceProfile::Record),
        ("type P { }", SourceProfile::Record),
        ("type P { x int x bool }", SourceProfile::Record),
        ("type P { self int }", SourceProfile::Record),
        ("type P { x int\n static fn f() int { return 0 } }", SourceProfile::Record),
        ("type P { mut fn f() int { return 0 } }", SourceProfile::Record),
        ("fn main() int { let p P = P { x: 1 } return p.x }", SourceProfile::Record),
        ("fn main() int { let p P = P { } return 0 }", SourceProfile::Record),
        ("fn main() int { return p.mut }", SourceProfile::Record),
        ("fn main() int { return P { x: 1 }.x }", SourceProfile::Record),
        // text 面
        ("fn main() int { let s str = \"a\" return 0 }", SourceProfile::Text),
        ("fn main() int { let s Str = \"a\" return 0 }", SourceProfile::Text),
        ("fn main() int { return \"a\".to(str) }", SourceProfile::Text),
        ("fn main() int { return \"a\".to(bool) }", SourceProfile::Text),
        ("fn main() int { return \"a\".move }", SourceProfile::Text),
        ("fn main() int { return \"a\".len() }", SourceProfile::Text),
        ("fn main() int { return \"a\".len }", SourceProfile::Text),
        ("fn f(mut s str) int { return 0 }", SourceProfile::Text),
        ("fn f(move s Str) int { return 0 }", SourceProfile::Text),
        ("fn f(mut s str) int { return 0 }", SourceProfile::Scalar),
        ("fn main() int { return 1 }", SourceProfile::Text),
        // entry/console 面
        ("let x = 1\nx + 1", SourceProfile::EntryUnit),
        ("fn f() void { } ", SourceProfile::EntryUnit),
        ("return 1", SourceProfile::EntryUnit),
        ("let x = 1\nfn f() int { return 0 }", SourceProfile::EntryUnit),
        ("fn main() int { return 0 }", SourceProfile::HostConsole),
        ("fn print() int { return 0 }", SourceProfile::HostConsole),
        ("let print = 1\n1", SourceProfile::HostConsole),
    ];
    for (i, (src, profile)) in cases.iter().enumerate() {
        let old = ace_core::source::parse_program_with_handwritten(src, *profile);
        let new = cst::parse_program_via_cst(src, *profile);
        let agree = match (&old, &new) {
            (Ok(a), Ok(b)) => format!("{a:?}") == format!("{b:?}"),
            (Err(d), Err(e)) => {
                d.stage.as_str() == e.stage.as_str()
                    && d.code == e.code
                    && d.message == e.message
                    && d.span.start == e.span.start
                    && d.span.end == e.span.end
            }
            _ => false,
        };
        assert!(
            agree,
            "edge#{i} {:#?} src={src:?}\nold={:?}\nnew={:?}",
            profile,
            old.map_err(|d| (d.code.clone(), d.message.clone(), d.span.start, d.span.end)),
            new.map_err(|d| (d.code.clone(), d.message.clone(), d.span.start, d.span.end)),
        );
    }
}

#[test]
fn t07_direct() {
    // T07 代理对：旧/新首诊断全等（stage/code/message/span 逐字段）。
    let src = r#"fn main() int { let c char = '\u{D800}'; return 0 }"#;
    let old = parse_program_with(src, SourceProfile::Text);
    let new = cst::parse_program_via_cst(src, SourceProfile::Text);
    match (&old, &new) {
        (Err(d), Err(e)) => assert_eq!(
            (d.stage.as_str(), &d.code, &d.message, d.span.start, d.span.end),
            (e.stage.as_str(), &e.code, &e.message, e.span.start, e.span.end)
        ),
        _ => panic!("T07 surrogate must diagnose on both chains: old={old:?} new={new:?}"),
    }
}

/// 临时调试：小片段 old/new 对照（ZZ927S=1 启用）。
#[test]
fn zz_snippets() {
    if std::env::var("ZZ927S").unwrap_or_default().is_empty() {
        return;
    }
    let snips: &[(&str, SourceProfile)] = &[
        ("fn fn_leaf_1(v int) int {
    if v < 0 {
        return 0 - v
    }
    return v + 1
}

fn fn_000(x int) int {
    var acc = x * 3
    acc = acc + fn_leaf_1(x)
    return acc
}

fn main() int {
    return fn_000(5)
}
", SourceProfile::Scalar),
    ];
    // ZZ927F=<path>：整文件差分（大负载用）。
    if let Ok(path) = std::env::var("ZZ927F") {
        let src = std::fs::read_to_string(&path).expect("read ZZ927F");
        let profile = SourceProfile::Scalar;
        let old = ace_core::source::parse_program_with_handwritten(&src, profile);
        let new = cst::parse_program_via_cst(&src, profile);
        match (&old, &new) {
            (Ok(a), Ok(b)) => {
                let o = format!("{a:?}");
                let n = format!("{b:?}");
                if o == n {
                    eprintln!("FILE-IDENTICAL len={}", src.len());
                } else {
                    let pos = o.bytes().zip(n.bytes()).position(|(x, y)| x != y).unwrap_or(o.len().min(n.len()));
                    let st = pos.saturating_sub(120);
                    eprintln!("FILE-DIFF@{pos}
--old--
{}
--new--
{}", &o[st..(pos + 160).min(o.len())], &n[st..(pos + 160).min(n.len())]);
                }
            }
            (Err(d), Err(e)) => eprintln!("FILE-BOTH-ERR old={d:?} new={e:?}"),
            _ => eprintln!("FILE-OUTCOME-MISMATCH old={:?} new={:?}", old.is_ok(), new.is_ok()),
        }
        return;
    }
    for (i, (src, profile)) in snips.iter().enumerate() {
        let old = ace_core::source::parse_program_with_handwritten(src, *profile);
        let new = cst::parse_program_via_cst(src, *profile);
        let agree = match (&old, &new) {
            (Ok(a), Ok(b)) => format!("{a:?}") == format!("{b:?}"),
            (Err(d), Err(e)) => {
                d.stage.as_str() == e.stage.as_str() && d.code == e.code && d.message == e.message
                    && d.span.start == e.span.start && d.span.end == e.span.end
            }
            _ => false,
        };
        eprintln!(
            "SNIP#{i} {agree} {:#?} src={src:?}
  old={:?}
  new={:?}",
            profile,
            old.as_ref().map(|_| "..ok..").map_err(|d| (d.stage.as_str(), d.code.clone(), d.message.clone(), d.span.start, d.span.end)),
            new.as_ref().map(|_| "..ok..").map_err(|d| (d.stage.as_str(), d.code.clone(), d.message.clone(), d.span.start, d.span.end)),
        );
        if !agree {
            let o = format!("{old:?}");
            let n = format!("{new:?}");
            let pos = o.bytes().zip(n.bytes()).position(|(a, b)| a != b).unwrap_or(o.len().min(n.len()));
            let s0 = pos.saturating_sub(120);
            eprintln!(
                "FIRSTDIFF@{pos}
--old--
{}
--new--
{}",
                &o[s0..(pos + 160).min(o.len())],
                &n[s0..(pos + 160).min(n.len())]
            );
        }
    }
}

/// 临时调试：对指定 (语料文件, profile) 打印 old/new 首差上下文（ZZ927=文件名,profile）。
#[test]
fn zz_debug_first_diff() {
    let args = std::env::var("ZZ927").unwrap_or_default();
    if args.is_empty() {
        return;
    }
    let mut it = args.split(',');
    let fname = it.next().unwrap();
    let pname = it.next().unwrap();
    let profile = match pname {
        "Core" => SourceProfile::Core,
        "Scalar" => SourceProfile::Scalar,
        "Text" => SourceProfile::Text,
        "Record" => SourceProfile::Record,
        "TextRecord" => SourceProfile::TextRecord,
        "EntryUnit" => SourceProfile::EntryUnit,
        _ => SourceProfile::HostConsole,
    };
    let corpus_dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../crates/auto-lang/test/cases/ace-m1/src"
    );
    let src = std::fs::read_to_string(format!("{corpus_dir}/{fname}")).expect("read");
    let old = parse_program_with(&src, profile);
    let new = cst::parse_program_via_cst(&src, profile);
    let o = format!("{old:?}");
    let n = format!("{new:?}");
    if o == n {
        eprintln!("IDENTICAL");
        return;
    }
    let pos = o.bytes().zip(n.bytes()).position(|(a, b)| a != b).unwrap_or(o.len().min(n.len()));
    let ctx = 160;
    let s = pos.saturating_sub(ctx);
    eprintln!("FIRST DIFF at byte {pos}:
--old--
{}
--new--
{}", &o[s..(pos + ctx).min(o.len())], &n[s..(pos + ctx).min(n.len())]);
}

// ---------------------------------------------------------------- T-07 片段

/// V-04：free fn 片段差分（旧 parse_function_fragment 合同——free-only、
/// 完整消耗、绝对 span）。
#[test]
fn fragment_free_fn_differential() {
    let src = "fn before() int { return 0 }
fn mid(a int, b int) int { return a + b }
fn after() bool { return true }
";
    // mid 的区间（绝对字节坐标）。
    let start = src.find("fn mid").unwrap();
    let end = src.find("fn after").unwrap();
    let old = ace_core::source::parse_function_fragment(src, start..end, SourceProfile::Scalar);
    let new = cst::parse_function_fragment_via_cst(src, start..end, SourceProfile::Scalar);
    match (&old, &new) {
        (Ok(a), Ok(b)) => {
            assert_eq!(format!("{a:?}"), format!("{b:?}"), "fragment AST mismatch");
            // 绝对 span：name 在 mid 位置（非 0 基）。
            assert_eq!(b.name.span.start, src.find("mid").unwrap());
        }
        (Err(d), Err(e)) => {
            assert_eq!(
                (d.code.as_str(), d.message.as_str(), d.span.start, d.span.end),
                (e.code.as_str(), e.message.as_str(), e.span.start, e.span.end)
            );
        }
        _ => panic!("outcome mismatch: old={old:?} new={new:?}"),
    }
    // 反控：区间含两个函数 → 旧"exactly one"拒绝。
    let start2 = src.find("fn mid").unwrap();
    let r = cst::parse_function_fragment_via_cst(src, start2..src.len(), SourceProfile::Scalar);
    let rold = ace_core::source::parse_function_fragment(src, start2..src.len(), SourceProfile::Scalar);
    assert!(rold.is_err());
    assert!(r.is_err());
    // 坏区间：无序/越界 → 同名拒绝。
    assert!(cst::parse_function_fragment_via_cst(src, 10..5, SourceProfile::Scalar).is_err());
    assert!(cst::parse_function_fragment_via_cst(src, 0..9999, SourceProfile::Scalar).is_err());
}

/// V-04：方法片段（owner/receiver 上下文恢复；不拼假 free fn）。
#[test]
fn fragment_method_context() {
    let src = "type P { v int
    fn get(x int) int { return .v + x }
}
";
    let start = src.find("fn get").unwrap();
    let end = src[start..].find("}
}").map(|i| i + start + 1).unwrap();
    let out = cst::parse_front(cst::FrontRequest {
        src,
        profile: SourceProfile::Record,
        kind: cst::FrontKind::MethodFragment {
            range: start..end,
            owner: "P".to_string(),
            receiver: ace_core::source::ast::Receiver::View,
        },
        budget: cst::FrontBudget::WORK_OWNED_64MIB,
    });
    match out {
        cst::FrontOutcome::Ok(ok) => {
            assert_eq!(ok.program.items.len(), 1);
            match &ok.program.items[0] {
                ace_core::source::ast::Item::Fn(f) => {
                    assert_eq!(f.receiver, ace_core::source::ast::Receiver::View);
                    assert_eq!(f.owner.as_ref().unwrap().name, "P");
                    assert_eq!(f.name.name, "get");
                    assert_eq!(f.name.span.start, src.find("get").unwrap());
                }
                other => panic!("expected fn, got {other:?}"),
            }
        }
        other => panic!("expected Ok, got {other:?}"),
    }
}
