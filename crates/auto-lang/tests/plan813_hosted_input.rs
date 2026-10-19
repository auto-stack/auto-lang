use std::time::{Duration, Instant};

const INPUT_SOURCE: &str = r#"
use auto.io

fn main() {
    let first str = next_line()
    let second str = next_line()
    let third str = next_line()
    let fourth str = next_line()
    print(first + "|" + second + "|" + third + "|" + fourth)
}

fn next_line() str {
    let value = io.read_line_opt()
    is value {
        Some(line) -> line
        None -> "EOF"
    }
}
"#;

fn run_with_input(input: String) -> String {
    let (_, stdout, _, _) = auto_lang::run_with_capture_and_bytecode_with_input(
        INPUT_SOURCE,
        Some(Instant::now() + Duration::from_secs(2)),
        Some(input),
    )
    .expect("input program compiles and runs");
    stdout
}

#[test]
fn request_input_distinguishes_blank_line_from_eof_and_handles_crlf() {
    let stdout = run_with_input("alpha\r\n\nlast".to_string());
    assert!(
        stdout.contains("alpha||last|EOF"),
        "unexpected stdout: {stdout:?}"
    );
}

#[test]
fn simultaneous_requests_keep_input_separate() {
    let left = std::thread::spawn(|| run_with_input("left\n".to_string()));
    let right = std::thread::spawn(|| run_with_input("right\n".to_string()));
    let left = left.join().expect("left execution thread");
    let right = right.join().expect("right execution thread");
    assert!(left.contains("left|EOF|EOF|EOF"), "left stdout: {left:?}");
    assert!(
        right.contains("right|EOF|EOF|EOF"),
        "right stdout: {right:?}"
    );
}

#[test]
fn hashset_contains_is_printed_as_a_boolean() {
    let source = r#"
fn main() {
    var unique = HashSet.new()
    unique.insert(1)
    unique.insert(2)
    unique.insert(3)
    print(unique.contains(3))
    print(unique.contains(4))
    unique.drop()
}
"#;
    let (_, stdout) = auto_lang::run_with_capture(source).expect("HashSet program runs");
    assert_eq!(stdout, "true\nfalse\n");
}
