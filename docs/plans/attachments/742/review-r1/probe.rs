use std::{fs, path::Path};
fn main() {
    let root = std::env::args().nth(1).expect("output root");
    for (name, text) in [
        ("control", "ordinary message"),
        ("list_literal", "List<int>.new([])"),
        ("module_literal", "repository.filename"),
        ("brace_literal", "} return secret"),
    ] {
        let project = Path::new(&root).join(name);
        let back = project.join("src/back");
        fs::create_dir_all(&back).unwrap();
        let api = format!(r#"pub type Probe = {{ text: str }}
#[api(method = "GET", path = "/api/echo")]
pub fn echo() Probe {{
    var msg str = "{text}"
    if false {{ return Probe {{ text: msg }} }}
    return Probe {{ text: msg }}
}}
"#);
        fs::write(back.join("api.at"), &api).unwrap();
        fs::write(back.join("repository.at"), "pub fn ping() int { return 1 }\n").unwrap();
        match auto_man::api_gen::generate_api(&project, "rust") {
            Ok(()) => println!("CASE {name}: generation Ok; expected text={text:?}"),
            Err(e) => println!("CASE {name}: generation Err: {e}"),
        }
        if let Ok(entries) = fs::read_dir(project.join("rust-workspace")) {
            for e in entries.flatten() {
                let p = e.path().join("src/api.rs");
                if let Ok(s) = fs::read_to_string(&p) {
                    println!("ARTIFACT {}", p.display());
                    if let Some(i) = s.find("pub async fn echo") {
                        println!("{}", &s[i..]);
                    }
                }
            }
        }
    }
}
