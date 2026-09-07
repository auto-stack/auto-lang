// Plan 565 P0/T2: mem-profile 归因报告测试（#[ignore] 诊断，独占进程）。
//
// 跑法（单测独占进程，计数即该进程全程，见 D1）：
//   cargo test -p auto-lang --lib --features mem-profile -- \
//     --ignored test_mem_profile_report --nocapture
//
// 归因探针取 m2 语料闸门同形态程序（aavm2 lib + main{parse_dump(语料)}），
// 分界落在 create_vm_from_source（编译侧：session/parse/CTEE/codegen/link/
// VM 构造）与 spawn_task+run_task_loop（执行侧，主管线同形态，见 lib.rs
// "5. Execute" 段）。VM 为 !Send（Rc 内部），编译与执行必须在同一线程——
// 与 run_with_capture 一致走专用大栈线程（aavm 516KB lib 解释栈需求，
// P574 T6 后缺省 16MB），快照经全局原子在任意线程读取，归因不受影响。

use crate::error::AutoResult;
use crate::mem_profile::{report_delta_line, report_total_line, Snapshot};

/// m2 闸门同款转义（aavm2_m2.rs escape_for_at_literal 的本地副本）。
fn escape_for_at_literal(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// 探针语料：小而有代表性（fn 定义 + 调用 + let 绑定）。
const PROBE_CORPUS: &str = "fn add(a int, b int) int {\n    return a + b\n}\nlet x = add(1, 2)\n";

/// m2 闸门同形态程序：aavm2 lib + main{parse_dump(语料字面量)}。
fn build_probe_program() -> AutoResult<String> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let lib_code = crate::aavm2_lib_source(&root)?;
    Ok(format!(
        "{}\nfn main() {{\n    print(parse_dump(\"{}\"))\n}}\n",
        lib_code,
        escape_for_at_literal(PROBE_CORPUS)
    ))
}

/// Rust 参考侧 AST dump（m2 判据同款：Parser::from + Code 的 Display）。
fn rust_parse_dump(code: &str) -> AutoResult<String> {
    let mut parser = crate::parser::Parser::from(code);
    let ast = parser.parse()?;
    Ok(format!("{}", ast))
}

#[test]
#[ignore = "mem-profile 归因诊断：需 --features mem-profile 显式独占进程跑（见文件头跑法）"]
fn test_mem_profile_report() {
    // —— 阶段 0：harness 前置（lib 源拼装 + 程序串构造） ——
    let s0 = Snapshot::take();
    let program = build_probe_program().expect("build probe program");
    let expected = rust_parse_dump(PROBE_CORPUS).expect("rust parse dump");
    let s1 = Snapshot::take();

    // —— 阶段 1/2：编译侧与执行侧（同一线程，VM !Send；16MB 栈=P574 T6 形态） ——
    let program_for_thread = program.clone();
    let (s2, s3, stdout) = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let (vm, stdout_buf, entry, _result_type) =
                crate::create_vm_from_source(&program_for_thread).expect("compile probe program");
            let s2 = Snapshot::take();

            let rt = crate::get_global_runtime();
            vm.spawn_task(entry, 65536);
            rt.block_on(async {
                vm.run_task_loop().await;
            });
            let stdout = stdout_buf.read().unwrap().clone();
            let s3 = Snapshot::take();
            (s2, s3, stdout)
        })
        .expect("spawn probe thread")
        .join()
        .expect("probe thread panicked");

    // 判据不降级：探针 dump 与 Rust 参考侧逐字一致（m2 判据同款）。
    assert_eq!(
        stdout.trim_end(),
        expected.trim_end(),
        "probe AST-dump mismatch\n--- rust ---\n{}\n--- probe ---\n{}",
        expected,
        stdout
    );

    // —— 阶段 3：端到端交叉核对（run_with_capture，m2 闸门同入口） ——
    let (_r, stdout_e2e) = crate::run_with_capture(&program).expect("run_with_capture probe");
    assert_eq!(
        stdout_e2e.trim_end(),
        expected.trim_end(),
        "end-to-end AST-dump mismatch"
    );
    let s4 = Snapshot::take();

    eprintln!("{}", report_delta_line("0_harness_libload", &s1.delta_since(&s0)));
    eprintln!("{}", report_delta_line("1_compile", &s2.delta_since(&s1)));
    eprintln!("{}", report_delta_line("2_exec", &s3.delta_since(&s2)));
    eprintln!("{}", report_delta_line("3_end_to_end_rerun", &s4.delta_since(&s3)));
    eprintln!("{}", report_total_line());
}
