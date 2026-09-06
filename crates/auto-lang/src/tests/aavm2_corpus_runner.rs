// Plan 565 L1: aavm2 语料闸门 once-compiled runner。
//
// 现状（564 实测）：每语料文件拼 `lib + main{dump("语料字面量")}` 后
// run_with_capture —— 程序串逐文件不同 → 每文件全链重编译 ~500KB 自举链
// （m2 = 35 文件 × ~2.2s ≈ 147s，单次编译 alloc churn 7.4GiB/峰值 ~755MiB）。
//
// 本 runner：语料经 File.read_text（nat#1000，P532 W1 G15；T3 探针验证
// dump 与字面量通道逐字一致）从临时文件注入 → 程序串闸门内恒定 →
// create_vm_from_source 只编译一次 → 每语料仅换文件内容 + 在同一 VM 上
// 重跑 main（spawn_task + run_task_loop，autovm daemon 同款重入形态）。
// 判据断言原样留在各闸门（本模块只回传逐语料 stdout，不改判据）。
//
// VM 为 !Send（Rc 内部）：编译与全部重跑固定在同一专用大栈线程
// （16MB = P574 T6 vm_thread_stack_size 缺省形态）。

use crate::error::AutoResult;
use std::path::PathBuf;

/// 一条语料用例：源文件路径 + 内容（由闸门收集排序后传入）。
#[derive(Clone)]
pub struct CorpusCase {
    pub path: PathBuf,
    pub code: String,
}

/// .at 字符串字面量中的路径转义（复审补丁）：.at lexer 对已识别转义序列
/// （\n \t \r \0 \\ \"）做变换、未知序列原样透传——Windows 路径含
/// `\t`/`\n` 等段首（如 TEMP=C:\tmp、用户名 tom）时未转义路径会被改写、
/// File.read_text 读错文件。反斜杠全量双写即可令所有 `\x` 序列惰性化
/// （同 aavm2_m4 use 腿 escape_for_at_literal(main_path) 先例）。
fn escape_at_path(s: &str) -> String {
    s.replace('\\', "\\\\")
}

/// 单闸门语料批跑：编译一次，逐语料换注入文件重跑 main。
///
/// * `gate` —— 闸门标识（临时文件名去重 + 日志前缀，如 "m1"/"m2"）。
/// * `dump_fn` —— lib 侧 dump 入口（lex_dump/parse_dump/typecheck_dump/
///   codegen_dump）。
/// 返回与 `cases` 同序的 stdout 列表；线程 panic 上抛为 AutoError。
pub fn run_corpus_once_compiled(
    gate: &str,
    dump_fn: &str,
    cases: &[CorpusCase],
) -> AutoResult<Vec<String>> {
    // 临时注入文件：闸门 + 进程 pid 唯一（并行测试进程/闸门互不串台）。
    let tmp = std::env::temp_dir().join(format!(
        "aavm2_corpus_565_{}_{}.at",
        gate,
        std::process::id()
    ));
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let lib_code = crate::aavm2_lib_source(&root)?;
    let program = format!(
        "{}\nfn main() {{\n    print({}(File.read_text(\"{}\")))\n}}\n",
        lib_code,
        dump_fn,
        escape_at_path(&tmp.display().to_string())
    );

    let cases = cases.to_vec();
    let tmp_in_thread = tmp.clone();
    let ran = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || -> AutoResult<Vec<String>> {
            let (vm, stdout_buf, entry, _result_type) =
                crate::create_vm_from_source(&program)?;
            let rt = crate::get_global_runtime();
            let mut outs = Vec::with_capacity(cases.len());
            for case in &cases {
                std::fs::write(&tmp_in_thread, &case.code)?;
                // 清空捕获缓冲：上一语料的 stdout 不串台
                *stdout_buf.write().unwrap() = String::new();
                vm.spawn_task(entry, 65536);
                rt.block_on(async {
                    vm.run_task_loop().await;
                });
                outs.push(stdout_buf.read().unwrap().clone());
            }
            let _ = std::fs::remove_file(&tmp_in_thread);
            Ok(outs)
        })
        .expect("spawn corpus runner thread");
    let result = ran
        .join()
        .map_err(|_| crate::error::AutoError::Msg(format!("corpus runner thread panicked (gate={gate})")))?;
    result
}

/// L1 回归（测试设计节）：同一 once-compiled 批内多语料重跑输出正确
/// （缓存 VM 重入无状态串台）+ 两次独立批跑输出一致（缓存正确性）。
#[test]
#[cfg_attr(windows, ignore = "avm+aavm/avm+aa2r 双重解释器路径关闭(572 待澄清②裁定 2026-09-06):run_autovm_capture 硬编码 4MB 执行线程被 516KB lib 解释栈需求越过(探针 4MB 爆/5MB 过,与用例规模无关;T6 已修栈,路径维持关闭);重型对拍走⑤腿/at_mode/gen2(a2r 转译+编译+运行);Linux/CI 保留全量")]
fn test_aavm2_corpus_runner_rerun_consistency() {
    if !crate::tests::heavy_gate::heavy_gate("test_aavm2_corpus_runner_rerun_consistency") {
        return;
    }
    let cases: Vec<CorpusCase> = [
        "fn add(a int, b int) int {\n    return a + b\n}\n",
        "let s = \"hello\"\nlet n = s.len()\n",
        "type P { x int, y int }\nfn mk(x int) P {\n    return P { x: x, y: 0 }\n}\n",
    ]
    .iter()
    .enumerate()
    .map(|(i, code)| CorpusCase {
        path: PathBuf::from(format!("<rerun-consistency-{i}.at>")),
        code: code.to_string(),
    })
    .collect();

    let outs = run_corpus_once_compiled("runner_reg", "parse_dump", &cases)
        .expect("runner batch 1");
    // 逐语料：重跑输出与 Rust 参考侧逐字一致（缓存 VM 重入无串台）
    for (case, stdout) in cases.iter().zip(&outs) {
        let mut parser = crate::parser::Parser::from(&case.code);
        let ast = parser.parse().expect("rust parse");
        let expected = format!("{}", ast);
        assert_eq!(
            stdout.trim_end(),
            expected.trim_end(),
            "rerun drift for {}\n--- rust ---\n{}\n--- runner ---\n{}",
            case.path.display(),
            expected,
            stdout
        );
    }
    // 两次独立批跑输出一致
    let outs2 = run_corpus_once_compiled("runner_reg", "parse_dump", &cases)
        .expect("runner batch 2");
    assert_eq!(outs, outs2, "two runner batches diverged");
}
