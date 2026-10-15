use auto_man::project_model_adapter::old_normalized_from_code;
use auto_project_model::{parse_document, SourceInfo};
use std::io::Write;
fn main() {
    let i: usize = std::env::args().nth(1).unwrap().parse().unwrap();
    let cases = [
        "members: [\"a\" // trailing comment\n, \"b\"]\n",
        "members: [\"a\" /* block */ , \"b\"]\n",
        "cfg: { a: true\nb: false }\n",
        "cfg: [true\nfalse]\n",
        "{ name: \"ok\", back: { project: \"b\" /* comment */ } }\n",
        "members: [\"a\",\n\"b\",]\n",
        "cfg: { a: 1\nb: 2 }\n",
        "cfg: { a: /*comment*/ 1, b: 2 }\n",
        "cfg: { a: true, b: [false, nil] }\n",
    ];
    let code=cases[i];
    let old=old_normalized_from_code(code);
    println!("case={i} input={code:?}\nold={old:?}");
    std::io::stdout().flush().unwrap();
    let new=parse_document(code,SourceInfo::inline());
    println!("new={new:?}");
    assert!(old.is_ok(), "positive oracle control must succeed");
    assert!(new.unwrap().is_static_equivalent(), "frozen literal/comments support must remain static");
}
