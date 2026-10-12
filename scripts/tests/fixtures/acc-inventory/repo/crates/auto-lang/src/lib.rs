// fixture stub of the registry host: only the block the scanner reads matters.
pub(crate) const AUTO_LIB_FILES: &[&str] = &[
    "auto/lib-legacy/token.at",
];

pub(crate) const AUTO_LIB_FILES_V2: &[&str] = &[
    "auto/lib/token.at",
    "auto/lib/lexer.at",
    "auto/lib/parser.at",
    "auto/lib/typeinfo.at",
    "auto/lib/codegen.at",
    "auto/lib/engine.at",
    "auto/lib/a2r.at",
];

pub(crate) fn aavm2_lib_source(project_root: &std::path::Path) -> AutoResult<String> {
    let mut out = String::new();
    for file in AUTO_LIB_FILES_V2 {
        let content = std::fs::read_to_string(project_root.join(file))?;
        for line in content.lines() {
            if line.trim_start().starts_with("use auto.lib.") {
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
    }
    Ok(out)
}
