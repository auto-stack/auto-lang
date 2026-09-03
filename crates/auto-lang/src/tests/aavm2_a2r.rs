// Plan 447 部分② Phase 7 / AA2R-is 闸门:a2r.at 自身的 is-match/or-臂/
// 枚举载荷发射与主 a2r(transpile_rust,基线 = 已修 H4/H5)逐字符一致。
//
// 语料:test/vm/aavm2/corpus_a2r/*.at(枚举载荷 is/标量 or-臂/字面量与
// 卫语句/字符串模式)。
// 判据:Rust 侧 transpile_rust(name, src).done() 与 AAVM 侧 auto/lib
// 全七文件前置后 ar_run(src, 0) 的输出逐字符相等(M2 式 live 对齐,
// 无落盘 golden——主 a2r 行为即唯一基准)。

use crate::error::AutoResult;
use crate::run_with_capture;
use std::path::PathBuf;

fn escape_for_at_literal(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/vm/aavm2/corpus_a2r")
}

fn test_a2r_corpus_file(path: &std::path::Path) -> AutoResult<()> {
    let code = std::fs::read_to_string(path)?;
    let stem = path.file_stem().unwrap().to_string_lossy().to_string();
    let name = stem.splitn(2, '_').nth(1).unwrap_or(&stem).to_string();
    let mut sink = crate::trans::rust::transpile_rust(&name, &code)?;
    let expected = String::from_utf8_lossy(sink.done()?).to_string();

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let lib_code = crate::aavm2_lib_source(&root)?;
    let program = format!(
        "{}\nfn main() {{\n    print(ar_run(\"{}\", 0))\n}}\n",
        lib_code,
        escape_for_at_literal(&code)
    );
    let (_r, stdout) = run_with_capture(&program)?;
    if std::env::var("AA2R_DUMP").is_ok() {
        eprintln!("DUMP-FILE {}
DUMP-HOST<<<{}>>>
DUMP-AA2R<<<{}>>>", path.display(), expected, stdout);
    }
    assert_eq!(
        stdout.trim_end(),
        expected.trim_end(),
        "AA2R mismatch for {}\n--- main a2r ---\n{}\n--- aa2r ---\n{}",
        path.display(),
        expected,
        stdout
    );
    Ok(())
}

/// Plan 523 W1:语料收集——平铺 `gNN_name.at` + per-case dir
/// `gNN_name/gNN_name.at`(三件套金样格式,新件一律此形态)。
fn collect_corpus(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out: Vec<std::path::PathBuf> = Vec::new();
    for e in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("corpus dir {}: {e}", dir.display())) {
        let p = e.expect("read_dir entry").path();
        if p.is_dir() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let inner = p.join(format!("{}.at", name));
            if inner.is_file() {
                out.push(inner);
            }
        } else if p.extension().map(|x| x == "at").unwrap_or(false) {
            out.push(p);
        }
    }
    out.sort();
    out
}

#[test]
fn test_aavm2_a2r_is_corpus() {
    let dir = corpus_dir();
    let entries = collect_corpus(&dir);
    assert!(!entries.is_empty(), "no corpus files under {}", dir.display());
    let mut checked = 0;
    for p in entries {
        test_a2r_corpus_file(&p).unwrap();
        checked += 1;
    }
    eprintln!("AA2R is corpus: {checked} files, transpiled text identical to main a2r");
}

/// 诊断用:打印主 a2r 对语料的转译产物(--nocapture)。
#[test]
fn test_aavm2_a2r_main_dump_print() {
    let dir = corpus_dir();
    let entries = collect_corpus(&dir);
    for p in entries {
        let code = std::fs::read_to_string(&p).unwrap();
        let stem = p.file_stem().unwrap().to_string_lossy().to_string();
        let name = stem.splitn(2, '_').nth(1).unwrap_or(&stem).to_string();
        match crate::trans::rust::transpile_rust(&name, &code) {
            Ok(mut sink) => {
                let rs = String::from_utf8_lossy(sink.done().unwrap()).to_string();
                eprintln!("=== {} ===\n{}\n", p.display(), rs)
            }
            Err(e) => eprintln!("=== {} === TRANSPILE ERROR: {}\n", p.display(), e),
        }
    }
}

/// Plan 447 7.5 探针冒烟:99_idiom_probe 的 p01/p02b/p04/p05/p12 经 AA2R
/// (a2r.at 自身)转译后 rustc 零错。`#[ignore]`:需 rustc,按需跑:
/// cargo test -p auto-lang --lib --features test-vm-files a2r_probe_smoke -- --ignored
#[test]
#[ignore = "shells out to rustc; on-demand AA2R compile-level guard (Plan 447)"]
fn test_aavm2_a2r_probe_smoke() {
    let probes = [
        "p01_is_string",
        "p02b_enum_or_arm",
        "p04_runtime_concat_payload",
        "p05_double_match",
        "p12_is_binding_types",
    ];
    let lib_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let lib_code = crate::aavm2_lib_source(&lib_dir).unwrap();
    let probe_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/vm/99_idiom_probe");
    for p in probes {
        let case_dir = probe_root.join(p);
        let stem = p.splitn(2, '_').nth(1).unwrap_or(p);
        let src = std::fs::read_to_string(case_dir.join(format!("{}.at", stem))).unwrap();
        let program = format!(
            "{}
fn main() {{
    print(ar_run(\"{}\", 0))
}}
",
            lib_code,
            escape_for_at_literal(&src)
        );
        let (_r, stdout) = run_with_capture(&program)
            .unwrap_or_else(|e| panic!("AA2R transpile failed for {}: {}", p, e));
        let rs = stdout.trim_end().to_string();
        assert!(!rs.starts_with("TRANSPILE-ERROR") && !rs.is_empty(), "AA2R error for {}: {}", p, rs);
        let out = std::env::temp_dir().join(format!("aa2r_probe_{}.rmeta", stem));
        let status = std::process::Command::new("rustc")
            .args(["--crate-type=bin", "--edition", "2021", "--emit=metadata", "-o"])
            .arg(&out)
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.as_mut().unwrap().write_all(rs.as_bytes()).unwrap();
                child.wait_with_output()
            })
            .expect("rustc spawn");
        assert!(
            status.status.success(),
            "AA2R product for {} failed rustc:
{}
--- product ---
{}",
            p,
            String::from_utf8_lossy(&status.stderr),
            rs
        );
    }
}

/// Plan 523 W1 发射闸:中阶语料(g19–g25)主 a2r 产物独立 rustc 零错——
/// 三面闸的第三面(发射面)落地形态:live 对拍(字符级)+ 本闸(编译级)。
/// 考古五洞 H1–H5 的转绿载体(H1 字段名撞方法/H2 嵌套 place mut/
/// H3 返回推断/H4 str 下标/H5 全局 once_cell)。
/// `#[ignore]`: shells out to rustc,验收/折叠点按需跑:
/// cargo test -p auto-lang --lib --features test-vm-files a2r_corpus_rustc -- --ignored
#[test]
#[ignore = "shells out to rustc; on-demand emission-level guard (Plan 523)"]
fn test_aavm2_a2r_corpus_rustc() {
    let dir = corpus_dir();
    for name in [
        "g19_struct_decl",
        "g20_struct_ctor",
        "g21_field_rw",
        "g22_for_in_arr",
        "g23_str_index",
        "g24_neg",
        "g25_globals",
        // Plan 525 W1:VBool 件入发射编译闸
        "g26_bool_print",
        // Plan 525 W2:方法族/is-struct 件入发射编译闸
        "g27_methods_family",
        "g28_is_struct",
    ] {
        let case = dir.join(name).join(format!("{}.at", name));
        let code = std::fs::read_to_string(&case)
            .unwrap_or_else(|e| panic!("read {}: {}", case.display(), e));
        let stem = name.splitn(2, '_').nth(1).unwrap_or(name);
        let mut sink = crate::trans::rust::transpile_rust(stem, &code)
            .unwrap_or_else(|e| panic!("main a2r failed for {}: {}", name, e));
        let rs = String::from_utf8_lossy(sink.done().unwrap()).to_string();
        let out = std::env::temp_dir().join(format!("p523_rustc_{}.rmeta", name));
        let status = std::process::Command::new("rustc")
            .args(["--crate-type=bin", "--edition", "2021", "--emit=metadata", "-o"])
            .arg(&out)
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.as_mut().unwrap().write_all(rs.as_bytes()).unwrap();
                child.wait_with_output()
            })
            .expect("rustc spawn");
        assert!(
            status.status.success(),
            "main a2r product for {} failed rustc:\n{}\n--- product ---\n{}",
            name,
            String::from_utf8_lossy(&status.stderr),
            rs
        );
    }
}


// ── Plan 523 W3:三件套金样(at+expected.rs+expected.out)+ A2R_BLESS ──
//
// ③ 锚:主 a2r transpile_rust 产物 vs `<name>.expected.rs`(live 对拍盲区
//     闭合——主 a2r 自身漂移可检);
// ② 锚:Rust 参考执行输出 vs `<name>.expected.out`(与 m5/compile oracle
//     同源)。
// 布局:corpus_a2r per-case dir(g19–g25 新件,at 在 dir 内)+
// corpus_m4 平铺旁挂(b07/b13/b32/b33/b34/b36/b42 抽验集,at 不动、
// 金样同名旁挂——两 walker 零影响,W3-10 布局裁定)。
// 再生:A2R_BLESS=1 → live 覆写金样并打印 BLESSED;diff 走 git 评审。

/// 三件套金样用例收集:corpus_a2r per-case dir + corpus_m4 平铺旁挂。
fn golden_cases() -> Vec<(std::path::PathBuf, String)> {
    let mut out = Vec::new();
    // per-case dir(新件)
    let a2r_dir = corpus_dir();
    let mut dirs: Vec<_> = std::fs::read_dir(&a2r_dir)
        .expect("corpus_a2r dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    for d in dirs {
        let name = d.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        if d.join(format!("{}.at", name)).is_file() {
            out.push((d, name));
        }
    }
    // 平铺旁挂(抽验集,Plan 523 W3-10 裁定名单;at 不动、金样同名旁挂,
    // m5/compile 两 walker 零影响)
    const M4_SAMPLE: &[&str] = &[
        "b07_fib", "b13_is_enum", "b32_is_break_continue", "b33_fstr_eval",
        "b34_struct_basic", "b36_struct_nested", "b42_globals",
        // Plan 525 W1:裸 bool print 恢复件入金样集(P474 期望翻转评审位)
        "b13_eval_print_true", "b14_eval_print_false", "b19_eval_print_bools",
        // Plan 525 W2:方法族/is-struct 代表件入金样集(四路判定面)
        "b44_methods_basic", "b45_is_struct_basic",
        // Plan 525 W3:容器族代表件入金样集
        "b46_list_basic",
    ];
    let m4 = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/vm/aavm2/corpus_m4");
    for name in M4_SAMPLE {
        let at = m4.join(format!("{}.at", name));
        assert!(at.is_file(), "sample case missing: {}", at.display());
        out.push((m4.clone(), name.to_string()));
    }
    out
}

/// 金样三件套校验/再生(默认校验;A2R_BLESS=1 再生)。
#[test]
fn test_aavm2_goldens_check() {
    let bless = std::env::var("A2R_BLESS").is_ok();
    let cases = golden_cases();
    assert!(!cases.is_empty(), "no golden cases");
    let mut mismatches: Vec<String> = Vec::new();
    let mut blessed = 0usize;
    for (dir, name) in &cases {
        let at = dir.join(format!("{}.at", name));
        let code = std::fs::read_to_string(&at).expect("read case at");
        let stem = name.splitn(2, '_').nth(1).unwrap_or(name);

        // ③ 主 a2r 产物
        let mut sink = crate::trans::rust::transpile_rust(stem, &code).expect("main a2r");
        let rs = String::from_utf8_lossy(sink.done().unwrap()).to_string();
        let rs_path = dir.join(format!("{}.expected.rs", name));
        // ② 参考输出
        let (_r, out) = crate::run_with_capture(&code).expect("rust reference run");
        let out_path = dir.join(format!("{}.expected.out", name));

        if bless {
            std::fs::write(&rs_path, &rs).unwrap();
            std::fs::write(&out_path, out.trim_end().to_string() + "\n").unwrap();
            eprintln!("BLESSED {} / {}", rs_path.display(), out_path.display());
            blessed += 1;
            continue;
        }
        let exp_rs = std::fs::read_to_string(&rs_path)
            .unwrap_or_else(|e| panic!("read {}: {}", rs_path.display(), e));
        if rs.trim_end() != exp_rs.trim_end() {
            mismatches.push(format!("{}: expected.rs (③ 主 a2r 产物漂移)", name));
        }
        let exp_out = std::fs::read_to_string(&out_path)
            .unwrap_or_else(|e| panic!("read {}: {}", out_path.display(), e));
        if out.trim_end() != exp_out.trim_end() {
            mismatches.push(format!("{}: expected.out (② 参考输出漂移)", name));
        }
    }
    assert!(
        mismatches.is_empty(),
        "golden mismatches ({} of {}):\n{}",
        mismatches.len(),
        cases.len(),
        mismatches.join("\n")
    );
    eprintln!(
        "Goldens: {}/{} cases checked{}",
        cases.len() - blessed,
        cases.len(),
        if blessed > 0 { format!(" (+{} blessed)", blessed) } else { String::new() }
    );
}

// ── Plan 523 W3:四路统一 runner(单用例四途径一致判定 + 译文回链)──
//
//   path1  AutoVM+aavm :lib + print(ev_run(src))   → 输出 ②(执行锚)
//   path3  a2r+aavm    :aavm2_bin <at>             → 输出 ②
//   path2  AutoVM+AA2R :lib + print(ar_run(src,0)) → 译文 ③(发射锚)
//   path4  a2r+AA2R    :aavm2_bin --trans <at>     → 译文 ③
//   回链   译文可运行   :rustc 编译 path4 译文并运行 → 输出对拍 ②
//   同锚   主 a2r 三方  :transpile_rust == path2 == path4(③)
//
// 用例集 = 三件套金样集(g19–g25 per-case dir + corpus_m4 抽验集)。
// `#[ignore]`(shells aavm2_bin[内含 cargo build] + rustc),验收/折叠点
// 按需:cargo test -p auto-lang --lib --features test-vm-files fourpath -- --ignored --nocapture
#[test]
#[ignore = "shells cargo/rustc; on-demand four-path acceptance runner (Plan 523)"]
fn test_aavm2_fourpath_runner() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let lib_code = crate::aavm2_lib_source(&root).unwrap();
    // 内容寻址缓存复用:compile corpus 的 build_aavm_rust_bin
    let bin = crate::tests::vm_file_tests::build_aavm_rust_bin_pub();
    let cases = golden_cases();
    assert!(!cases.is_empty());

    let mut table: Vec<String> = Vec::new();
    let mut fails = 0usize;
    for (dir, name) in &cases {
        let at = dir.join(format!("{}.at", name));
        let code = std::fs::read_to_string(&at).unwrap();
        let stem = name.splitn(2, '_').nth(1).unwrap_or(name);

        // path1 / path2(参考宿主执行 v2 VM)
        let prog_run = format!(
            "{}\nfn main() {{\n    print(ev_run(\"{}\"))\n}}\n",
            lib_code,
            escape_for_at_literal(&code)
        );
        let (_r, out1) = crate::run_with_capture(&prog_run).expect("path1 ev_run");
        let prog_trans = format!(
            "{}\nfn main() {{\n    print(ar_run(\"{}\", 0))\n}}\n",
            lib_code,
            escape_for_at_literal(&code)
        );
        let (_r2, trans2) = crate::run_with_capture(&prog_trans).expect("path2 ar_run");

        // path3 / path4(转译编译 bin)
        let run3 = std::process::Command::new(&bin).arg(&at).output().expect("path3 spawn");
        assert!(run3.status.success(), "path3 failed on {}", name);
        let out3 = String::from_utf8_lossy(&run3.stdout).to_string();
        let run4 = std::process::Command::new(&bin).arg("--trans").arg(&at).output().expect("path4 spawn");
        assert!(run4.status.success(), "path4 failed on {}", name);
        let trans4 = String::from_utf8_lossy(&run4.stdout).to_string();

        // 主 a2r 同锚
        let mut sink = crate::trans::rust::transpile_rust(stem, &code).unwrap();
        let trans_m = String::from_utf8_lossy(sink.done().unwrap()).to_string();

        // 译文回链:rustc 编译 path4 译文并运行
        let link_dir = std::env::temp_dir().join(format!("p523-link-{}", name));
        let _ = std::fs::remove_dir_all(&link_dir);
        std::fs::create_dir_all(&link_dir).unwrap();
        let link_exe = link_dir.join("case_bin.exe");
        let rc = std::process::Command::new("rustc")
            .args(["--edition", "2021", "-O"])
            .arg("-o").arg(&link_exe)
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.as_mut().unwrap().write_all(trans4.as_bytes()).unwrap();
                child.wait_with_output()
            })
            .expect("rustc spawn");
        let link_ok = rc.status.success();
        let out4r = if link_ok {
            let ro = std::process::Command::new(&link_exe).output().expect("run linked");
            assert!(ro.status.success(), "linked bin failed on {}", name);
            String::from_utf8_lossy(&ro.stdout).to_string()
        } else {
            String::new()
        };

        // 判定
        let exec_ok = link_ok
            && out1.trim_end() == out3.trim_end()
            && out1.trim_end() == out4r.trim_end();
        let trans_ok = trans2.trim_end() == trans4.trim_end()
            && trans2.trim_end() == trans_m.trim_end();
        if !(exec_ok && trans_ok) {
            fails += 1;
        }
        table.push(format!(
            "{:<24} exec(②): {:<4} trans(③): {:<4} {}",
            name,
            if out1.trim_end() == out3.trim_end() { "p1=p3" } else { "p1≠p3" },
            if trans2.trim_end() == trans4.trim_end() { "p2=p4" } else { "p2≠p4" },
            if exec_ok && trans_ok { "PASS" } else {
                if link_ok { "FAIL" } else { "FAIL(link)" }
            }
        ));
        if !trans_ok {
            eprintln!(
                "[fourpath:trans diff {}] main-a2r vs path2 identical: {} / path2 vs path4 identical: {}",
                name,
                trans_m.trim_end() == trans2.trim_end(),
                trans2.trim_end() == trans4.trim_end()
            );
        }
        if !exec_ok && link_ok {
            eprintln!(
                "[fourpath:exec diff {}] p1==p3: {} p1==link: {}",
                name,
                out1.trim_end() == out3.trim_end(),
                out1.trim_end() == out4r.trim_end()
            );
        }
    }
    eprintln!("==== 四路 runner 判定表 ====");
    for row in &table {
        eprintln!("{}", row);
    }
    eprintln!("==== {} cases, {} PASS, {} FAIL ====", table.len(), table.len() - fails, fails);
    assert_eq!(fails, 0, "four-path runner failures");
}

