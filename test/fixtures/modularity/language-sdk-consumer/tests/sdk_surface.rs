//! PLAN-814 M814-A V-01：纯 SDK 独立消费面（Core parse→compile→VM + trans）。
//!
//! 闭包正控：本 fixture 唯一依赖 `auto-lang-core`（默认特性）——测试进程
//! 的成功编译与执行即"无 UI checkout 可消费"的结构性证明；行为负控见
//! `ui_grammar_explicitly_refused`。

use auto_lang_core::parser::Parser;
use auto_lang_core::session::CompilerSession;

/// Core 单文件程序经 SDK 公开 run 入口（parse→code→AutoVM）执行并捕获
/// stdout——非 UI 程序全管线真实执行（非字符串检查）。
#[test]
fn core_program_runs_on_vm_with_output() {
    let (result, stdout) = auto_lang_core::run_with_capture(
        r#"
fn add(a int, b int) int {
    return a + b
}
fn main() {
    print(add(2, 40))
}
"#,
    )
    .expect("core program must run");
    assert!(result.is_empty() || result.contains("ok") || !stdout.is_empty());
    assert!(stdout.contains("42"), "stdout={stdout:?} result={result:?}");
}

/// 手工低层管线（auto-vm 同款公开面）：Parser→Codegen→链接→AutoVM
/// spawn+run——证明 bin 级消费者（auto-vm）所依赖的细粒度 API 面。
#[tokio::test]
async fn manual_vm_bytecode_pipeline_is_public() {
    use auto_lang_core::vm::codegen::Codegen;
    use auto_lang_core::vm::engine::AutoVM;
    use auto_lang_core::vm::native_registry::register_builtin_natives;
    use auto_lang_core::vm::virt_memory::VirtualFlash;

    let source = r#"
fn main() {
    print(6 * 7)
}
"#;
    let mut parser = Parser::from(source);
    let code = parser.parse().expect("parse");

    let mut codegen = Codegen::new();
    for stmt in code.stmts.iter() {
        codegen.compile_stmt(&stmt).expect("compile_stmt");
    }
    for reloc in codegen.relocs.iter() {
        if matches!(
            reloc.reloc_type,
            auto_lang_core::vm::loader::RelocType::FuncCall
        ) {
            if let Some(&addr) = codegen.exports.get(&reloc.symbol_name) {
                let bytes = addr.to_le_bytes();
                let offset = reloc.offset as usize;
                for (j, b) in bytes.iter().enumerate() {
                    codegen.code[offset + j] = *b;
                }
            }
        }
    }

    let flash = VirtualFlash::from_vec_with_metadata(
        codegen.code,
        codegen.exports.clone(),
        codegen.object_keys,
        codegen.object_types,
    );
    let mut vm = AutoVM::new(flash, 1024 * 1024);
    vm.load_strings(codegen.strings);
    register_builtin_natives();
    let entry = codegen.exports.get("main").copied().unwrap_or(0) as usize;
    vm.spawn_task(entry, 1024 * 1024);
    vm.run_task_loop().await;
}

/// trans 出口：真实 .at 文件 → Rust 源（含函数签名/打印语句）。
#[test]
fn trans_rust_exit_produces_source() {
    let dir = std::env::temp_dir().join("p814-sdk-consumer-trans");
    std::fs::create_dir_all(&dir).unwrap();
    let at = dir.join("probe.at");
    std::fs::write(&at, "fn main() {\n    print(\"sdk-trans\")\n}\n").unwrap();
    let rust = auto_lang_core::trans_rust(at.to_str().unwrap()).expect("trans_rust");
    // trans 写 `<path>.a2r.rs` 并返回摘要串（Plan 204）——读真实产物断言。
    let out_path = at.with_extension("a2r.rs");
    let mut generated = String::new();
    for _ in 0..50 {
        if out_path.exists() {
            generated = std::fs::read_to_string(&out_path).unwrap_or_default();
            if !generated.is_empty() {
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        generated.contains("fn main"),
        "summary={rust:?} generated={generated:?}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// 纯 SDK 没有隐式 UI 文法；UI 声明只能由上层经显式领域端口组合。
#[test]
fn ui_grammar_explicitly_refused() {
    let mut parser = Parser::from("msg M { A }");
    let err = parser.parse().expect_err("pure SDK must refuse UI grammar");
    assert!(!format!("{err:?}").is_empty());
}

/// Core 场景显式装配：非 UI session 解析普通语言程序（SDK 的主场景），
/// Scenario 核心语义不受 UI 组合影响。
#[test]
fn core_session_parses_plain_program() {
    let mut parser = Parser::from("fn main() {\n    var x int = 1\n}\n").with_session(
        CompilerSession::new(auto_lang_core::session::Scenario::Core),
    );
    let ast = parser.parse().expect("core parse");
    assert!(!ast.stmts.is_empty());
}
