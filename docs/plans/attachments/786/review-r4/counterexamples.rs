#[path = "D:/autostack/.wt/lang-786/auto-lang/crates/auto-man/src/project_display_reader.rs"]
mod project_display_reader;
fn main() {
 let a: Vec<_> = std::env::args().collect();
 let n: usize = a[2].parse().unwrap();
 let nested = format!("{}0{}", "[".repeat(n), "]".repeat(n));
 let body = match a[1].as_str() {
  "plain" => format!("deep: {nested}\n"),
  "quoted-block" => format!("extra: \"/*\"\ndeep: {nested}\n"),
  "quoted-line" => format!("extra: [\"//\", {nested}]\n"),
  "quoted-hash" => format!("extra: [\"#\", {nested}]\n"),
  "cancel-depth" => format!("deep: {}0{}\n", "[\"]\",".repeat(n), "]".repeat(n)),
  _ => panic!("unknown case")
 };
 let input = format!("name: \"n\"\ntitle: \"ok\"\n{body}");
 std::fs::write(&a[3], &input).unwrap();
 println!("{}-{n} legacy name=n title=ok (frozen first-match line scan)",a[1]);
 let r=project_display_reader::read(&input);
 assert_eq!(r.name.as_deref(),Some("n"));
 assert_eq!(r.title.as_deref(),Some("ok"));
 assert_eq!(r.title_zh,None);
 println!("{}-{n} PASS {:?}", a[1],r.title_path);
}


