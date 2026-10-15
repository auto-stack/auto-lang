#[path = "D:/autostack/.wt/lang-786/auto-lang/crates/auto-man/src/project_display_reader.rs"]
mod project_display_reader;
fn main() {
    let a: Vec<_> = std::env::args().collect();
    let n: usize = a[2].parse().unwrap();
    let nested = format!("{}0{}", "[".repeat(n), "]".repeat(n));
    let prefix = "name: \"n\"\ntitle: \"ok\"\n";
    let body = match a[1].as_str() {
        "plain" => format!("deep: {nested}\n"),
        "multiline" => format!("extra: [\"hello\nworld\", {nested}]\n"),
        "single-unclosed" => format!("extra: 'abc\nignored: 0\ndeep: {nested}\n"),
        "single-closed-later" => format!("extra: 'abc\nignored: 0\ndeep: {nested}\ntail: 'z'\n"),
        "f-unclosed" => format!("extra: f\"abc\nignored: 0\ndeep: {nested}\n"),
        "call-unclosed" => format!("extra: call('abc\n)\ndeep: {nested}\n"),
        _ => panic!("unknown case"),
    };
    let input = format!("{prefix}{body}");
    std::fs::write(&a[3], &input).unwrap();
    assert_eq!(input.lines().find(|l| l.starts_with("title:")).unwrap(), "title: \"ok\"");
    let r = project_display_reader::read(&input);
    assert_eq!(r.name.as_deref(), Some("n"));
    assert_eq!(r.title.as_deref(), Some("ok"));
    println!("{}-{n} PASS {:?}", a[1], r.title_path);
}
