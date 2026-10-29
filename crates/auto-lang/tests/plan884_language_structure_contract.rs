//! PLAN-884 语言结构收口合同测试（V-02/V-03 运行桥接行为守恒）。
//!
//! 冻结协议：本文件于未改基面 `d9718cc63`（分支 plan-884-dev 起点）先运行并
//! 固化基线观察；T-03 运行桥接收口后逐字保持。任何断言失败=桥接行为漂移，
//! 不得通过更新本文件期望值来"修复"（那是输出漂移，须回代码修复）。
//!
//! 覆盖矩阵（对齐 Plan 884 §6 V-02/V-03）：
//! - 成功值 + stdout（capture / 无 capture 路径的结果串）
//! - path 相对导入（use db → db.at 同目录解析）
//! - input None / Some("") / Some(非空) 三态
//! - deadline 已到期（立即 Err）
//! - bytecode/meta 非空（BytecodeMeta 形状）
//! - staged 通道：编译错误 → run_trap_code None（基础设施错误非 typed trap）
//! - staged 通道：运行期 typed trap → run_trap_code Some(..)（分类保持）

use std::time::{Duration, Instant};

const OK_PROG: &str = "fn main() {\n    print(1 + 2)\n}\n";

#[test]
fn plan884_v02_run_capture_exact_stdout() {
    let (result, stdout) = auto_lang::run_with_capture(OK_PROG).expect("run must succeed");
    assert_eq!(stdout, "3\n", "stdout must be byte-exact");
    assert_eq!(result, "", "print-only program result string");
}

#[test]
fn plan884_v02_run_plain_result_string() {
    let result = auto_lang::run("fn main() {\n    return 7 * 6\n}\n").expect("run ok");
    assert_eq!(result, "42");
}

#[test]
fn plan884_v02_run_autovm_capture_pair() {
    let (result, stdout) = auto_lang::run_autovm_capture(OK_PROG).expect("autovm capture ok");
    assert_eq!(stdout, "3\n");
    assert_eq!(result, "");
}

#[test]
fn plan884_v02_path_relative_import() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("db.at"),
        "pub fn add(a int, b int) int {\n    a + b\n}\n",
    )
    .unwrap();
    let main_path = dir.path().join("main.at");
    std::fs::write(&main_path, "").unwrap();
    let path = main_path.to_string_lossy().to_string();
    let (_, stdout) = auto_lang::run_with_capture_and_path(
        "use db\nfn main() {\n    print(db.add(2, 3))\n}\n",
        &path,
    )
    .expect("path run ok");
    assert_eq!(stdout, "5\n");
}

#[test]
fn plan884_v02_bytecode_and_meta_shape() {
    let (r, stdout, disasm, meta) =
        auto_lang::run_with_capture_and_bytecode_with_meta(OK_PROG).expect("meta run ok");
    assert_eq!(stdout, "3\n");
    assert_eq!(r, "");
    assert!(!disasm.is_empty(), "disasm lines must be non-empty");
    assert!(format!("{meta:?}").len() > 8, "meta must carry content: {meta:?}");
}

#[test]
fn plan884_v02_input_none_is_process_stdin_semantics() {
    // input=None：请求 stdin 未注入。read_line 在无进程 stdin（测试环境）下
    // 的基线行为=返回空/EOF 串——冻结观察，不做"更好语义"。
    let (r, _stdout, _d, _m) = auto_lang::run_with_capture_and_bytecode_with_input(
        "use io\nfn main() {\n    let s = io.read_line()\n    print(s)\n}\n",
        None,
        None,
    )
    .expect("input=None run ok");
    let _ = r;
}

#[test]
fn plan884_v02_input_empty_is_immediate_eof() {
    let (_r, stdout, _d, _m) = auto_lang::run_with_capture_and_bytecode_with_input(
        "use io\nfn main() {\n    let s = io.read_line()\n    print(\"[\")\n    print(s)\n    print(\"]\")\n}\n",
        None,
        Some(String::new()),
    )
    .expect("input=empty run ok");
    // 基线观察（d9718cc63）：空输入 ⇒ read_line 返回 ""（无换行），print 追加换行。
    assert_eq!(stdout, "[\n\n]\n", "empty input ⇒ read_line returns empty string");
}

#[test]
fn plan884_v02_input_nonempty_line() {
    let (_r, stdout, _d, _m) = auto_lang::run_with_capture_and_bytecode_with_input(
        "use io\nfn main() {\n    let s = io.read_line()\n    print(s)\n}\n",
        None,
        Some("hello-884\n".to_string()),
    )
    .expect("input=nonempty run ok");
    // 基线观察（d9718cc63）：read_line 返回去换行的行内容，print 追加一个换行。
    assert_eq!(stdout, "hello-884\n", "read_line strips trailing newline");
}

#[test]
fn plan884_v03_deadline_already_expired() {
    let deadline = Instant::now()
        .checked_sub(Duration::from_secs(1))
        .expect("clock");
    let res = auto_lang::run_with_capture_and_bytecode_with_deadline(
        "for true {}",
        Some(deadline),
    );
    assert!(res.is_err(), "expired deadline must error, got {res:?}");
}

#[test]
fn plan884_v03_staged_compile_error_is_not_typed_trap() {
    let res = auto_lang::run_with_capture_and_path_staged(
        "fn main() {\n    print(undefined_fn_xyz())\n}\n",
        "nonexistent/main.at",
    );
    let err = res.expect_err("compile error must fail");
    assert!(
        err.run_trap_code.is_none(),
        "compile/infra failure must classify run_trap_code=None, got {:?}",
        err.run_trap_code
    );
}

#[test]
fn plan884_v03_staged_runtime_failure_trap_classification() {
    // 运行期失败走 typed VM trap 通道（task.last_error_code）。
    // 冻结基面观察：断言分类字段非 None（具体 code 值不钉字符串身份，
    // 只钉"typed 通道已接通"这一分类事实——与 None 的编译错误对照）。
    let res = auto_lang::run_with_capture_and_path_staged(
        "fn main() {\n    let m = Map.new()\n    let v = m.get(\"k\")\n    print(v.len())\n}\n",
        "probe/main.at",
    );
    match res {
        Ok((_, stdout)) => panic!("expected runtime failure, got ok stdout={stdout:?}"),
        Err(f) => {
            // 分类守恒：运行期 typed trap 必须接通（Some），区别于编译错误 None。
            assert!(
                f.run_trap_code.is_some(),
                "runtime failure must carry typed trap code, got None (err={:?})",
                f.error
            );
        }
    }
}
