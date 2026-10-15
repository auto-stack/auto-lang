#[path = "D:/autostack/.wt/lang-786/auto-lang/crates/auto-man/src/project_display_reader.rs"]
mod project_display_reader;
fn main() {
 let a:Vec<_>=std::env::args().collect(); let n:usize=a[2].parse().unwrap();
 let nested=format!("{}0{}", "[".repeat(n), "]".repeat(n));
 let s=match a[1].as_str() {
 "plain"=>format!("name: \"n\"\ntitle: \"ok\"\nextra: {nested}\n"),
 "multiline"=>format!("name: \"n\"\ntitle: \"ok\"\nextra: [\"hello\nworld\", {nested}]\n"),
 "escaped-newline"=>format!("name: \"n\"\ntitle: \"ok\"\nextra: [\"hello\\\nworld\", {nested}]\n"),
 "nodes"=>format!("name: \"n\"\ntitle: \"ok\"\n{}{}", "node {\n".repeat(n), "}\n".repeat(n)),
 _=>panic!("unknown mode")};
 std::fs::write(&a[3],&s).unwrap();
 // Frozen old algorithm: prefix title line alone already proves old result.
 assert_eq!(s.lines().find(|l|l.starts_with("title:")).unwrap(), "title: \"ok\"");
 let r=project_display_reader::read(&s); assert_eq!(r.title.as_deref(),Some("ok"));
 println!("{} depth={} PASS {:?}",a[1],n,r.title_path);
}
