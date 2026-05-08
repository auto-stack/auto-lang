use regex::Regex;

fn main() {
    let re = Regex::new(r"\d+").unwrap();
    let text = "abc 123 def 456";
    let replaced = re.replace_all(text, "NUM");
    println!("Replaced: {}", replaced);
}
