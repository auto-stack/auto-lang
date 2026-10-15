#[path = "D:/autostack/.wt/lang-786/auto-lang/crates/auto-man/src/project_display_reader.rs"]
mod project_display_reader;
fn old(s: &str, key: &str, skip: bool) -> Option<String> {
 for line in s.lines() { let line = line.trim(); if line.starts_with(&format!("{key}:")) { let v = line.split_once(':').unwrap().1.trim().trim_end_matches(',').trim_matches('"').trim_matches('\''); if !skip || !v.is_empty() { return Some(v.into()); } } } None
}
fn main() {
 let args: Vec<_> = std::env::args().collect();
 if args.len() > 1 {
   let n: usize = args[1].parse().unwrap();
   let s = format!("name: \"n\"\ntitle: \"ok\"\nextra: {}0{}\n", "[".repeat(n), "]".repeat(n));
   assert_eq!(old(&s, "title", true).as_deref(), Some("ok"));
   let r = project_display_reader::read(&s); assert_eq!(r.title.as_deref(), Some("ok"));
   println!("depth={n} PASS {:?}", r.title_path); return;
 }
 let vals = ["\"x\"", "\"\"", "''", "'x'", "nil", "true", "123", "\"  \"", "\"${x}\"", "env(\"T\")", "\"a\\nb\"", "[1]", "{a: 1}"];
 let prefixes = ["", "/*\ntitle: \"hidden\"\n*/\n", "dep x {\nname: \"nested\"\n}\n", "title: \"\"\n", "title : \"phantom\"\n"];
 let mut count=0;
 for p in prefixes { for v in vals { for end in ["\n", ",\n", " // c\n", " /* c */\n"] {
 let s=format!("{p}name: {v}{end}title: {v}{end}title_zh: {v}{end}");
 let r=project_display_reader::read(&s);
 assert_eq!(r.name,old(&s,"name",false),"{s:?}"); assert_eq!(r.title,old(&s,"title",true),"{s:?}"); assert_eq!(r.title_zh,old(&s,"title_zh",true),"{s:?}"); count+=1;
 } } }
 println!("independent combinations={count} PASS");
}
