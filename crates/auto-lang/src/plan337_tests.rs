//! Plan 337: VM-level test for List<Struct>.push (reproduces create_note
//! infinite recursion without UI). Run with:
//!   cargo test -p auto-lang --lib plan337_tests -- --nocapture

#[cfg(test)]
mod plan337_tests {
    use crate::run_with_capture;

    #[test]
    fn test_basic_print() {
        let (val, stdout) = run_with_capture("print(\"hello world\")").unwrap();
        eprintln!("val=[{}] stdout=[{}]", val, stdout);
        assert!(stdout.contains("hello world"), "got: [{}]", stdout);
    }

    #[test]
    fn test_basic_struct() {
        let code = r#"
type Note {
    id int
    title str
}

let note = Note { id: 1, title: "hello" }
print(note.title)
"#;
        let (val, stdout) = run_with_capture(code).unwrap();
        eprintln!("val=[{}] stdout=[{}]", val, stdout);
        assert!(stdout.contains("hello"), "got: [{}]", stdout);
    }

    #[test]
    fn test_list_struct_push() {
        let code = r#"
type Note {
    id int
    title str
}

var notes = List<Note>.new([])

pub fn add_note(title str) Note {
    let note = Note {
        id: 1,
        title: title,
    }
    notes.push(note)
    return note
}

let result = add_note("hello")
print(result.title)
"#;
        let result = run_with_capture(code);
        eprintln!("RESULT: {:?}", result);
        assert!(result.is_ok(), "add_note should not loop: {:?}", result.err());
        let (val, stdout) = result.unwrap();
        eprintln!("val=[{}] stdout=[{}]", val, stdout);
        assert!(stdout.contains("hello"), "got: [{}]", stdout);
    }

    #[test]
    fn test_list_struct_len_toplevel() {
        // Test len at top level (not in a function)
        let code = r#"
type Item { name str }
var items = List<Item>.new([])
print(items.len())
"#;
        let result = run_with_capture(code);
        eprintln!("RESULT: {:?}", result);
        assert!(result.is_ok());
        let (_, stdout) = result.unwrap();
        eprintln!("stdout=[{}]", stdout);
        assert!(stdout.contains("0"), "empty list len: [{}]", stdout);
    }

    #[test]
    fn test_list_struct_push_toplevel() {
        // Push at top level, then check len
        let code = r#"
type Item { name str }
var items = List<Item>.new([])
items.push(Item { name: "a" })
print(items.len())
"#;
        let result = run_with_capture(code);
        eprintln!("RESULT: {:?}", result);
        assert!(result.is_ok());
        let (_, stdout) = result.unwrap();
        eprintln!("stdout=[{}]", stdout);
        assert!(stdout.contains("1"), "len after push: [{}]", stdout);
    }

    #[test]
    fn test_list_struct_push_then_len() {
        // Push inside a function — the real repro
        let code = r#"
type Item { name str }
var items = List<Item>.new([])

pub fn add_and_count(name str) int {
    let item = Item { name: name }
    items.push(item)
    return items.len()
}

print(add_and_count("first"))
"#;
        let result = run_with_capture(code);
        eprintln!("RESULT: {:?}", result);
        assert!(result.is_ok());
        let (_, stdout) = result.unwrap();
        eprintln!("stdout=[{}]", stdout);
        assert!(stdout.contains("1"), "first push: [{}]", stdout);
    }

    #[test]
    fn test_list_push_int_basic() {
        // Simplest possible: int list push + len
        let code = r#"
var nums List<int> = List<int>.new([])
nums.push(42)
print(nums.len())
"#;
        let result = run_with_capture(code);
        eprintln!("RESULT int: {:?}", result);
        assert!(result.is_ok());
        let (_, stdout) = result.unwrap();
        eprintln!("stdout=[{}]", stdout);
        assert!(stdout.contains("1"), "len after 1 push: [{}]", stdout);
    }
}
