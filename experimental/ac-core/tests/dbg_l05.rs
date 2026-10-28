use auto_ac_prototype::source::{compile_parts_with, SourceProfile};
#[test]
fn dbg_l05() {
    let src = "type Item { name Str; code int }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"hello\", 37))\n    let x Item = xs.remove(0)\n    return x.name.len() + x.code + xs.len()\n}\n";
    let parts = compile_parts_with(src, "probe.at", &[], SourceProfile::Collections).expect("parts");
    for (i, line) in parts.atom_text.lines().enumerate() {
        if line.contains("expr") { println!("{}: {}", i + 1, line); }
    }
}
