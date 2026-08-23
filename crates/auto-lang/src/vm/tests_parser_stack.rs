//! Plan 419 §8.3 回归:解析器巨帧栈溢出(曾误判的两个"编译器栈溢出"bug)。
//! 详见 parser.rs parse() 的治理注释。
#![cfg(test)]

fn run_on_stack<F: FnOnce() + Send + 'static>(stack_bytes: usize, f: F) {
    let h = std::thread::Builder::new()
        .stack_size(stack_bytes)
        .spawn(f)
        .expect("spawn small-stack thread");
    let joined = h.join();
    assert!(joined.is_ok(), "parser must not overflow the stack (plan 419 §8.3)");
}

#[test]
fn bug1_struct_literal_overwrite_in_loop_parses() {
    let code = r#"
type Note { id int }

fn main() int {
    var keep int = 0
    var a Note = Note { id: 1 }
    for i in 0..3 {
        a = Note { id: i }
        keep = keep + a.id - i
    }
    keep
}
"#;
    run_on_stack(2 * 1024 * 1024, move || {
        let mut session = crate::compile::CompileSession::new();
        let mut parser = crate::parser::Parser::new_with_type_store(code, session.type_store());
        let _ = parser.parse().expect("parse must succeed");
    });
}

#[test]
fn bug1_full_pipeline_on_default_thread() {
    // 完整 编译+运行 在普通 libtest 工作线程上(端到端)。
    let code = r#"
type Note { id int }

fn main() int {
    var keep int = 0
    var a Note = Note { id: 1 }
    for i in 0..3 {
        a = Note { id: i }
        keep = keep + a.id - i
    }
    keep
}
"#;
    let (vm, stdout, entry, _rt) = crate::create_vm_from_source(code).expect("compile");
    let _ = (vm, stdout, entry);
}

#[test]
fn bug1_if_branch_variant_parses() {
    let code = r#"
type Note { id int }

fn main() int {
    var a Note = Note { id: 1 }
    if true {
        a = Note { id: 2 }
    }
    a.id
}
"#;
    run_on_stack(2 * 1024 * 1024, move || {
        let mut session = crate::compile::CompileSession::new();
        let mut parser = crate::parser::Parser::new_with_type_store(code, session.type_store());
        let _ = parser.parse().expect("parse must succeed");
    });
}

#[test]
fn bug2_nested_block_in_loop_no_overflow() {
    let code = r#"
type Note { id int }

fn main() int {
    var keep int = 0
    for i in 0..3 {
        {
            var c Note = Note { id: i }
            keep = c.id
        }
    }
    keep
}
"#;
    run_on_stack(2 * 1024 * 1024, move || {
        let mut session = crate::compile::CompileSession::new();
        let mut parser = crate::parser::Parser::new_with_type_store(code, session.type_store());
        // 解析成功或干净报错均可;崩溃才是 bug。
        let _ = parser.parse();
    });
}

#[test]
fn deep_expression_nesting_bounded() {
    let mut code = String::from("fn main() int {\n    ");
    for _ in 0..2000 {
        code.push('(');
    }
    code.push('1');
    for _ in 0..2000 {
        code.push(')');
    }
    code.push_str("\n}\n");
    run_on_stack(2 * 1024 * 1024, move || {
        let mut session = crate::compile::CompileSession::new();
        let mut parser = crate::parser::Parser::new_with_type_store(&code, session.type_store());
        let _ = parser.parse();
    });
}

#[test]
fn body_nesting_guard_clean_error() {
    // 略超护栏(257 > 256):护栏转干净错误,不崩溃也不挂死。
    let mut code = String::from("fn main() {\n");
    for _ in 0..257 {
        code.push_str("{\n");
    }
    for _ in 0..257 {
        code.push_str("}\n");
    }
    code.push_str("}\n");
    run_on_stack(4 * 1024 * 1024, move || {
        let mut session = crate::compile::CompileSession::new();
        let mut parser = crate::parser::Parser::new_with_type_store(&code, session.type_store());
        match parser.parse() {
            Ok(_) => panic!("257-deep nesting must hit the guard (limit 256)"),
            Err(_) => {}
        }
    });
}
