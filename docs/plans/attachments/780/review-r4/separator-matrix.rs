use auto_man::project_model_adapter::{old_normalized_from_code, new_normalized_from_code};
use auto_project_model::SourceInfo;
use std::io::Write;
fn main() {
    let i: usize=std::env::args().nth(1).unwrap().parse().unwrap();
    let cases=[
        "back: { project: \"b\" /*multi\nline*/, extra: 1 }\n",
        "back: { project: \"b\" /*multi\nline*/ extra: 1 }\n",
        "members: [\"a\" /*multi\nline*/ \"b\"]\n",
        "cfg: { a: true # line\nb: false }\n",
        "members: [\"a\" # line\n, \"b\"]\n",
        "{ name: \"ok\" /*comment*/, back: {project: \"b\"} }\n",
        "members: [\"a\", /*c*/, \"b\"]\n",
        "cfg: { a: true, /*c*/ b: false }\n",
        "cfg: { a: true /*c*/ false }\n",
        "cfg: { a: true\n, b: false }\n",
        "cfg: [\n, true]\n",
        "members: [\"a\", #c\n \"b\"]\n",
        "members: [\"a\"\r\n\"b\"]\n",
        "back: { project: \"b\",\r\n extra: 1 }\n",
        "cfg: { a: true /*c*/, b: [false\ntrue] }\n",
        "members: [\"a\" /*c*/ \"b\"]\n",
    ];
    let code=cases[i];let old=old_normalized_from_code(code);
    println!("case={i} input={code:?}\nold={old:?}");std::io::stdout().flush().unwrap();
    let new=new_normalized_from_code(code,&SourceInfo::inline());
    println!("new={new:?}");
    match old {
        Ok(want)=>{let (d,got)=new.expect("literal/trivia input accepted by oracle must parse");assert!(d.is_static_equivalent(),"legal frozen syntax must stay static");assert_eq!(want,got);},
        Err(_)=>{if let Ok((d,_))=new{assert!(!d.is_static_equivalent(),"oracle rejects: no static claim");}}
    }
}
