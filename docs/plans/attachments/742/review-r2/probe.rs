use std::{fs, path::Path};
fn generate(root: &str, name: &str, api: &str, repo: &str) {
    let project = Path::new(root).join(name);
    let back = project.join("src/back");
    fs::create_dir_all(&back).unwrap();
    fs::write(back.join("api.at"), api).unwrap();
    fs::write(back.join("repository.at"), repo).unwrap();
    match auto_man::api_gen::generate_api(&project, "rust") {
        Ok(()) => println!("CASE {name}: generation Ok"),
        Err(e) => println!("CASE {name}: generation Err: {e}"),
    }
    if let Ok(entries) = fs::read_dir(project.join("rust-workspace")) {
        for e in entries.flatten() {
            if let Ok(s) = fs::read_to_string(e.path().join("src/api.rs")) {
                if let Some(i) = s.find("pub async fn echo") { println!("{}", &s[i..]); }
            }
        }
    }
}
fn main() {
    let root = std::env::args().nth(1).expect("output root");
    let old = Path::new("D:/autostack/auto-lang/docs/plans/attachments/742/review-r1/original-fixture");
    generate(&root, "original_fixture", &fs::read_to_string(old.join("api.at")).unwrap(), &fs::read_to_string(old.join("repository.at")).unwrap());
    for (name, text) in [
        ("control", "ordinary message"),
        ("list_literal", "List<int>.new([])"),
        ("module_literal", "repository.filename"),
        ("brace_literal", "} return secret"),
        ("borrow_literal", "crate::repository::take(value)"),
        ("escape_literal", r#"引号\" List<int>.new([]) repository.ping() } return secret"#),
    ] {
        let api = format!(r#"pub type Probe = {{ text: str }}
#[api(method = "GET", path = "/api/echo")]
pub fn echo() Probe {{
    var msg str = "{text}"
    if false {{ return Probe {{ text: msg }} }}
    return Probe {{ text: msg }}
}}
"#);
        let repo = if matches!(name, "control" | "list_literal" | "module_literal" | "brace_literal") {
            "pub fn ping() int { return 1 }\n"
        } else {
            "pub fn ping() int { return 7 }\npub fn take(value str) int { return value.len() }\n"
        };
        generate(&root, name, &api, repo);
    }
    generate(&root, "module_control", r#"pub type Probe = { text: str }
#[api(method = "GET", path = "/api/echo")]
pub fn echo() Probe {
    var n int = repository.ping()
    if n == 7 { return Probe { text: "ok" } }
    return Probe { text: "bad" }
}
"#, "pub fn ping() int { return 7 }\n");
    generate(&root, "scope_shadow", r#"pub type Probe = { text: str }
#[api(method = "GET", path = "/api/echo")]
pub fn echo() Probe {
    var n int = repository.ping()
    if false {
        var repository str = "local"
        var size int = repository.len()
    }
    if n == 7 { return Probe { text: "ok" } }
    return Probe { text: "bad" }
}
"#, "pub fn ping() int { return 7 }\n");
    generate(&root, "void_control", r#"pub type Probe = { text: str }
#[api(method = "POST", path = "/api/echo")]
pub fn echo() {
    if false { var n int = 0 }
    var x int = 1
}
"#, "pub fn ping() int { return 7 }\n");
    generate(&root, "void_branch", r#"pub type Probe = { text: str }
#[api(method = "POST", path = "/api/echo")]
pub fn echo() {
    if false { return }
    var x int = 1
}
"#, "pub fn ping() int { return 7 }\n");
    generate(&root, "option_variable", r#"pub type Probe = { text: str }
#[api(method = "GET", path = "/api/echo/:id")]
pub fn echo(id int) ?Probe {
    var p ?Probe = Some(Probe { text: "ok" })
    if id > 0 { return p }
    return None
}
"#, "pub fn ping() int { return 7 }\n");
    generate(&root, "file_control", r#"use auto.http
pub type Probe = { text: str }
#[api(method = "GET", path = "/api/echo")]
pub fn echo() FileResponse {
    var path str = "readme.txt"
    return http.file_response("D:/files", path, "{}")
}
"#, "pub fn path() str { return \"readme.txt\" }\n");
    generate(&root, "file_module", r#"use auto.http
pub type Probe = { text: str }
#[api(method = "GET", path = "/api/echo")]
pub fn echo() FileResponse {
    var path str = repository.path()
    return http.file_response("D:/files", path, "{}")
}
"#, "pub fn path() str { return \"readme.txt\" }\n");
}
