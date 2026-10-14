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
        ("qualified_literal", "a2r_std::sqlite"),
        ("marker_literal", "__AUTO_A2R_STD_QUAL__ __AUTO_A2R_STD_SQLITE__ __AUTO_A2R_STD_SYS__"),
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
    generate(&root, "nested_borrow", r#"pub type Probe = { text: str }
#[api(method = "GET", path = "/api/echo")]
pub fn echo() Probe {
    var s str = "hi"
    var n int = repository.outer(repository.measure(s))
    if n == 2 { return Probe { text: "ok" } }
    return Probe { text: "bad" }
}
"#, "pub fn measure(value str) int { return value.len() }\npub fn outer(value int) int { return value }\n");
    generate(&root, "borrow_positive", r#"pub type Probe = { text: str }
#[api(method = "GET", path = "/api/echo")]
pub fn echo() Probe {
    var s str = "hi"
    var n int = repository.measure(s)
    if n == 2 { return Probe { text: "ok" } }
    return Probe { text: "bad" }
}
"#, "pub fn measure(value str) int { return value.len() }\n");
    generate(&root, "pair_scope", r#"pub type Probe = { text: str }
#[api(method = "GET", path = "/api/echo")]
pub fn echo() Probe {
    var names []str = ["a", "bb"]
    var n int = 0
    for i, repository in names { n = n + repository.len() }
    if n == 3 { return Probe { text: "ok" } }
    return Probe { text: "bad" }
}
"#, "pub fn ping() int { return 7 }\n");
    generate(&root, "pair_control", r#"pub type Probe = { text: str }
#[api(method = "GET", path = "/api/echo")]
pub fn echo() Probe {
    var names []str = ["a", "bb"]
    var n int = 0
    for i, entry in names { n = n + entry.len() }
    if n == 3 { return Probe { text: "ok" } }
    return Probe { text: "bad" }
}
"#, "pub fn ping() int { return 7 }\n");
    let extra = Path::new(&root).join("namespace_suffix/src/back");
    fs::create_dir_all(&extra).unwrap();
    fs::write(extra.join("custom_a2r_std.at"), "pub fn ping() int { return 7 }\n").unwrap();
    generate(&root, "namespace_suffix", r#"pub type Probe = { text: str }
#[api(method = "GET", path = "/api/echo")]
pub fn echo() Probe {
    var n int = custom_a2r_std.ping()
    if n == 7 { return Probe { text: "ok" } }
    return Probe { text: "bad" }
}
"#, "pub fn ping() int { return 7 }\n");
    generate(&root, "nested_twice", r#"pub type Probe = { text: str }
#[api(method = "GET", path = "/api/echo")]
pub fn echo() Probe {
    var s str = "hi"
    var n int = repository.outer(repository.outer(repository.measure(s)))
    if n == 2 { return Probe { text: "ok" } }
    return Probe { text: "bad" }
}
"#, "pub fn measure(value str) int { return value.len() }\npub fn outer(value int) int { return value }\n");
    for (name, flag) in [("compare_less", "1 < 2"), ("compare_greater", "2 > 1"), ("compare_control", "true")] {
        let api = format!(r#"pub type Probe = {{ text: str }}
#[api(method = "GET", path = "/api/echo")]
pub fn echo() Probe {{
    var s str = "ok"
    var n int = repository.choose({flag}, s)
    if n == 2 {{ return Probe {{ text: "ok" }} }}
    return Probe {{ text: "bad" }}
}}
"#);
        generate(&root, name, &api, "pub fn choose(flag bool, value str) int {\n    if flag { return value.len() }\n    return 0\n}\n");
    }
    generate(&root, "multiline_literal", r#"pub type Probe = { text: str }
#[api(method = "GET", path = "/api/echo")]
pub fn echo() Probe {
    var s str = """first
a2r_std::sqlite repository.take(value) List<int>.new([])
last"""
    if true { return Probe { text: s } }
    return Probe { text: "bad" }
}
"#, "pub fn take(value str) int { return value.len() }\n");
    let control = Path::new(&root).join("namespace_control/src/back");
    fs::create_dir_all(&control).unwrap();
    fs::write(control.join("custom.at"), "pub fn ping() int { return 7 }\n").unwrap();
    generate(&root, "namespace_control", r#"pub type Probe = { text: str }
#[api(method = "GET", path = "/api/echo")]
pub fn echo() Probe {
    var n int = custom.ping()
    if n == 7 { return Probe { text: "ok" } }
    return Probe { text: "bad" }
}
"#, "pub fn ping() int { return 7 }\n");
}
