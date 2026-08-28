// Test Plan 151: Global Variables in a2r transpiler
use auto_lang::trans::rust::transpile_rust;
use auto_val::AutoStr;

fn main() {
    let code = r#"
var users List<int> = List<int>.new([1, 2, 3])
var nextid int = 4

fn get_next_id() int {
    return nextid
}

fn add_user(id int) {
    users.push(id)
    nextid = nextid + 1
}

fn get_users() []int {
    return users.to_array()
}

fn main() {
    let id = get_next_id()
    add_user(10)
    let all = get_users()
}
"#;

    let mut sink = transpile_rust(AutoStr::from("test151"), code).unwrap();
    let rust_code = String::from_utf8(sink.done().unwrap().to_vec()).unwrap();

    println!("=== Generated Rust Code ===");
    println!("{}", rust_code);
    println!("=== End ===");

    // Verify key patterns
    assert!(rust_code.contains("static USERS: Lazy<Mutex<Vec<i32>>>"),
            "Missing static USERS with Lazy<Mutex<>>");
    assert!(rust_code.contains("static NEXT_ID: Lazy<Mutex<i32>>"),
            "Missing static NEXT_ID with Lazy<Mutex<>>");
    assert!(rust_code.contains("USERS.lock().unwrap()"),
            "Missing USERS.lock().unwrap() pattern");
    assert!(rust_code.contains("NEXT_ID.lock().unwrap()"),
            "Missing NEXT_ID.lock().unwrap() pattern");
    assert!(rust_code.contains("*NEXT_ID.lock().unwrap() +="),
            "Missing *NEXT_ID.lock().unwrap() += pattern");

    println!("\n✅ All Plan 151 global variable tests passed!");
}
