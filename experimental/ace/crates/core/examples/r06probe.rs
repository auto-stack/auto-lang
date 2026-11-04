use ace_core::source::{parse_program_with_handwritten, SourceProfile};
fn main() {
    for (s, w) in [
        ("type P { }\n", "empty-type"),
        ("type P { a int }\n", "one-field"),
        ("enum E { A }\n", "enum-min"),
    ] {
        for (p, pn) in [(SourceProfile::Collections, "coll"), (SourceProfile::Closures, "clos")] {
            match parse_program_with_handwritten(s, p) {
                Ok(_) => println!("{w}/{pn}: OK"),
                Err(d) => println!("{w}/{pn}: {} {}-{} {}", d.code, d.span.start, d.span.end, d.message),
            }
        }
    }
}
