use crate::{
    error::AutoResult,
    trans::rust::transpile_rust,
};
use std::fs::read_to_string;
use std::path::PathBuf;

fn test_a2r(case: &str) -> AutoResult<()> {
    // Parse test case name: "000_hello" -> "hello"
    let parts: Vec<&str> = case.split("_").collect();
    let name = parts[1..].join("_");

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_path = format!("test/a2r/{}/{}.at", case, name);
    let src_path = d.join(src_path);
    let src = read_to_string(src_path.as_path())?;

    let exp_path = format!("test/a2r/{}/{}.expected.rs", case, name);
    let exp_path = d.join(exp_path);
    let expected = if !exp_path.is_file() {
        "".to_string()
    } else {
        read_to_string(exp_path.as_path())?
    };

    let mut rcode = transpile_rust(&name, &src)?;
    let rs_code = rcode.done()?;

    if rs_code != expected.as_bytes() {
        // Generate .wrong.rs for comparison
        let gen_path = format!("test/a2r/{}/{}.wrong.rs", case, name);
        let gen_path = d.join(gen_path);
        std::fs::write(&gen_path, rs_code)?;
    }

    assert_eq!(String::from_utf8_lossy(rs_code), expected);
    Ok(())
}

#[test]
fn test_000_hello() {
    test_a2r("000_hello").unwrap();
}

#[test]
fn test_001_sqrt() {
    test_a2r("001_sqrt").unwrap();
}

#[test]
fn test_002_array() {
    test_a2r("002_array").unwrap();
}

#[test]
fn test_003_func() {
    test_a2r("003_func").unwrap();
}

#[test]
fn test_005_pointer() {
    test_a2r("005_pointer").unwrap();
}

#[test]
fn test_006_struct() {
    test_a2r("006_struct").unwrap();
}

#[test]
fn test_007_enum() {
    test_a2r("007_enum").unwrap();
}

#[test]
fn test_008_method() {
    test_a2r("008_method").unwrap();
}

#[test]
fn test_010_if() {
    test_a2r("010_if").unwrap();
}

#[test]
fn test_011_for() {
    test_a2r("011_for").unwrap();
}

#[test]
fn test_012_is() {
    test_a2r("012_is").unwrap();
}

#[test]
fn test_013_while() {
    test_a2r("013_while").unwrap();
}

#[test]
fn test_014_closure() {
    test_a2r("014_closure").unwrap();
}

#[test]
fn test_015_nested_if() {
    test_a2r("015_nested_if").unwrap();
}

#[test]
fn test_016_complex() {
    test_a2r("016_complex").unwrap();
}

#[test]
fn test_017_struct_methods() {
    test_a2r("017_struct_methods").unwrap();
}

#[test]
fn test_018_enum_pattern() {
    test_a2r("018_enum_pattern").unwrap();
}

#[test]
fn test_019_blocks() {
    test_a2r("019_blocks").unwrap();
}

#[test]
fn test_020_comprehensive() {
    test_a2r("020_comprehensive").unwrap();
}

#[test]
fn test_021_indexing() {
    test_a2r("021_indexing").unwrap();
}

#[test]
fn test_022_unary() {
    test_a2r("022_unary").unwrap();
}

#[test]
fn test_023_arithmetic() {
    test_a2r("023_arithmetic").unwrap();
}

#[test]
fn test_024_fstring() {
    test_a2r("024_fstring").unwrap();
}

#[test]
fn test_025_fstring_edge() {
    test_a2r("025_fstring_edge").unwrap();
}

#[test]
fn test_026_ref_expr() {
    test_a2r("026_ref_expr").unwrap();
}

#[test]
fn test_027_range_expr() {
    test_a2r("027_range_expr").unwrap();
}

#[test]
fn test_028_object() {
    test_a2r("028_object").unwrap();
}

#[test]
fn test_029_composition() {
    test_a2r("029_composition").unwrap();
}

#[test]
fn test_030_field_composition() {
    test_a2r("030_field_composition").unwrap();
}

#[test]
fn test_031_spec() {
    test_a2r("031_spec").unwrap();
}

#[test]
fn test_032_delegation() {
    test_a2r("032_delegation").unwrap();
}

#[test]
fn test_033_multi_delegation() {
    test_a2r("033_multi_delegation").unwrap();
}

#[test]
fn test_034_delegation_params() {
    test_a2r("034_delegation_params").unwrap();
}

#[test]
fn test_111_generic_alias() {
    test_a2r("111_generic_alias").unwrap();
}

#[test]
fn test_126_generic_field() {
    test_a2r("126_generic_field").unwrap();
}

#[test]
fn test_127_generic_ptr_field() {
    test_a2r("127_generic_ptr_field").unwrap();
}

#[test]
fn test_128_map_type() {
    test_a2r("128_map_type").unwrap();
}

#[test]
fn test_129_map_func() {
    test_a2r("129_map_func").unwrap();
}

#[test]
fn test_130_option_construct() {
    test_a2r("130_option_construct").unwrap();
}

#[test]
fn test_143_empty_variant_match() {
    test_a2r("143_empty_variant_match").unwrap();
}

#[test]
fn test_131_method_chain() {
    test_a2r("131_method_chain").unwrap();
}

#[test]
fn test_110_const_generics() {
    test_a2r("110_const_generics").unwrap();
}

#[test]
fn test_109_generic_hetero_enum() {
    test_a2r("109_generic_hetero_enum").unwrap();
}

#[test]
fn test_035_inheritance() {
    test_a2r("035_inheritance").unwrap();
}

#[test]
fn test_055_union() {
    test_a2r("055_union").unwrap();
}

#[test]
fn test_014_hetero_enum() {
    test_a2r("014_hetero_enum").unwrap();
}

#[test]
fn test_004_cstr() {
    test_a2r("004_cstr").unwrap();
}

#[test]
fn test_023_borrow_view() {
    test_a2r("023_borrow_view").unwrap();
}

#[test]
fn test_024_borrow_mut() {
    test_a2r("024_borrow_mut").unwrap();
}

#[test]
fn test_025_borrow_move() {
    test_a2r("025_borrow_move").unwrap();
}

#[test]
fn test_026_borrow_conflicts() {
    test_a2r("026_borrow_conflicts").unwrap();
}

#[test]
fn test_016_basic_spec() {
    test_a2r("016_basic_spec").unwrap();
}

#[test]
fn test_017_spec() {
    test_a2r("017_spec").unwrap();
}

#[test]
fn test_117_list_storage() {
    test_a2r("117_list_storage").unwrap();
}

// Plan 120: Option and Result type tests
#[test]
fn test_120_option() {
    test_a2r("120_option").unwrap();
}

// Plan 159 Phase 6B-1: is statement with multi-statement match arms
#[test]
fn test_132_is_multi_stmt() {
    test_a2r("132_is_multi_stmt").unwrap();
}

// Plan 159 Phase 6B-2.6: External crate use statement
#[test]
fn test_133_rust_use() {
    test_a2r("133_rust_use").unwrap();
}

// Plan 159 Phase 6B-2.1: async fn transpilation
#[test]
fn test_134_async_fn() {
    test_a2r("134_async_fn").unwrap();
}

// Plan 159 Phase 6B-2.3: derive attribute passthrough
#[test]
fn test_135_derive_attr() {
    test_a2r("135_derive_attr").unwrap();
}

// Plan 162: Type cast — expr.as(Type)
#[test]
fn test_136_type_cast() {
    test_a2r("136_type_cast").unwrap();
}

// Plan 162: Pointer methods — is_null, is_not_null, add, read, write, .ptr
#[test]
fn test_137_ptr_methods() {
    test_a2r("137_ptr_methods").unwrap();
}

// Plan 162: List-style methods with .as() type casts and or operator
#[test]
fn test_138_list_as_cast() {
    test_a2r("138_list_as_cast").unwrap();
}

#[test]
fn test_139_if_multistmt() {
    test_a2r("139_if_multistmt").unwrap();
}

#[test]
fn test_140_if_return() {
    test_a2r("140_if_return").unwrap();
}

#[test]
fn test_141_func_literal_return() {
    test_a2r("141_func_literal_return").unwrap();
}

// Plan 162: .to(Type) explicit type conversion
#[test]
fn test_142_to_convert() {
    test_a2r("142_to_convert").unwrap();
}

// Plan 163: static fn support
#[test]
fn test_148_static_fn() {
    test_a2r("148_static_fn").unwrap();
}

// Plan 163: pub visibility
#[test]
fn test_149_pub_visibility() {
    test_a2r("149_pub_visibility").unwrap();
}

// Plan 163: #[tokio::main] + async main
#[test]
fn test_150_tokio_main() {
    test_a2r("150_tokio_main").unwrap();
}

// Plan 163: &mut self methods
#[test]
fn test_151_mut_self() {
    test_a2r("151_mut_self").unwrap();
}

// Plan 163: per-field serde attributes
#[test]
fn test_152_field_attrs() {
    test_a2r("152_field_attrs").unwrap();
}

// Plan 164: ext Type for Trait { } — external trait impl
#[test]
fn test_153_ext_for() {
    test_a2r("153_ext_for").unwrap();
}

// Plan 165: Struct destructuring in is match arms
#[test]
fn test_154_struct_destructure() {
    test_a2r("154_struct_destructure").unwrap();
}

// Plan 166: Generic constraints #[with(T as Trait)]
#[test]
fn test_155_with_constraint() {
    test_a2r("155_with_constraint").unwrap();
}

// Plan 159 Phase 6B-2.7: impl From<A> for B via ext-for
#[test]
fn test_156_ext_from() {
    test_a2r("156_ext_from").unwrap();
}

// Plan 6B-3.4: const declaration
#[test]
fn test_157_const_decl() {
    test_a2r("157_const_decl").unwrap();
}

// Plan 6B-4.14: Box::new() / Arc::new() smart pointer constructors
#[test]
fn test_158_box_arc() {
    test_a2r("158_box_arc").unwrap();
}

// Plan 167 Phase 2: wildcard import (use module: *)
#[test]
fn test_160_wildcard_import() {
    test_a2r("160_wildcard_import").unwrap();
}

// Plan 167 Phase 1: pub use re-export
#[test]
fn test_159_pub_use() {
    test_a2r("159_pub_use").unwrap();
}

// Plan 167 Phase 4: multi-file project transpilation
#[test]
fn test_161_multi_file() {
    use crate::trans::rust::transpile_rust_project;

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let entry = d.join("test/a2r/161_multi_file/main.at");

    let result = transpile_rust_project(entry.to_str().unwrap()).unwrap();

    // Check that all 4 files were generated
    assert!(result.contains_key("main.rs"), "Missing main.rs");
    assert!(result.contains_key("db.rs"), "Missing db.rs");
    assert!(result.contains_key("api/mod.rs"), "Missing api/mod.rs");
    assert!(result.contains_key("api/handlers.rs"), "Missing api/handlers.rs");

    // Validate main.rs
    let main_rs = String::from_utf8_lossy(&result["main.rs"]);
    assert!(main_rs.contains("mod db;"), "main.rs should have 'mod db;'");
    assert!(main_rs.contains("mod api;"), "main.rs should have 'mod api;'");
    assert!(main_rs.contains("fn main()"), "main.rs should have fn main()");

    // Validate api/mod.rs
    let api_mod = String::from_utf8_lossy(&result["api/mod.rs"]);
    assert!(api_mod.contains("pub mod handlers;"), "api/mod.rs should have 'pub mod handlers;'");

    // Validate db.rs
    let db_rs = String::from_utf8_lossy(&result["db.rs"]);
    assert!(db_rs.contains("struct Connection"), "db.rs should have struct Connection");
    assert!(db_rs.contains("fn connect()"), "db.rs should have fn connect()");

    // Validate api/handlers.rs
    let handlers_rs = String::from_utf8_lossy(&result["api/handlers.rs"]);
    assert!(handlers_rs.contains("use super::db;"), "api/handlers.rs should have 'use super::db;'");
    assert!(handlers_rs.contains("fn handle_request"), "api/handlers.rs should have fn handle_request");

    // Validate Cargo.toml
    assert!(result.contains_key("Cargo.toml"), "Missing Cargo.toml");
    let cargo_toml = String::from_utf8_lossy(&result["Cargo.toml"]);
    assert!(cargo_toml.contains("[package]"), "Cargo.toml should have [package]");
    assert!(cargo_toml.contains("name = \"161_multi_file\""), "Cargo.toml should have project name");
    assert!(cargo_toml.contains("edition = \"2021\""), "Cargo.toml should have edition = 2021");
}

// Plan 168: Multi-line strings
#[test]
fn test_163_multi_str() {
    test_a2r("163_multi_str").unwrap();
}

// Plan 6B-4.19: shared variable declaration (static storage)
#[test]
fn test_162_shared_var() {
    test_a2r("162_shared_var").unwrap();
}

#[test]
fn test_999_doc_comments() {
    test_a2r("999_doc_comments").unwrap();
}

// === Temporary: verify auto-code .at files can be parsed and transpiled ===
#[test]
fn test_autocode_types() {
    let src = std::fs::read_to_string("../../../auto-code/src/types.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("types", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_permission() {
    let src = std::fs::read_to_string("../../../auto-code/src/permission.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("permission", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_tools() {
    let src = std::fs::read_to_string("../../../auto-code/src/tools.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("tools", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_sse() {
    let src = std::fs::read_to_string("../../../auto-code/src/sse.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("sse", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_context() {
    let src = std::fs::read_to_string("../../../auto-code/src/context.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("context", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_settings() {
    let src = std::fs::read_to_string("../../../auto-code/src/settings.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("settings", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_agent() {
    let src = std::fs::read_to_string("../../../auto-code/src/agent.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("agent", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_anthropic() {
    let src = std::fs::read_to_string("../../../auto-code/src/anthropic.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("anthropic", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_openai() {
    let src = std::fs::read_to_string("../../../auto-code/src/openai.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("openai", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_session() {
    let src = std::fs::read_to_string("../../../auto-code/src/session.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("session", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_repl() {
    let src = std::fs::read_to_string("../../../auto-code/src/repl.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("repl", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_main() {
    let src = std::fs::read_to_string("../../../auto-code/src/main.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("main", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_mod() {
    let src = std::fs::read_to_string("../../../auto-code/src/mod.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("mod", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_tool_bash() {
    let src = std::fs::read_to_string("../../../auto-code/src/tool_bash.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("tool_bash", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_tool_grep() {
    let src = std::fs::read_to_string("../../../auto-code/src/tool_grep.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("tool_grep", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_tool_file_read() {
    let src = std::fs::read_to_string("../../../auto-code/src/tool_file_read.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("tool_file_read", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_tool_file_write() {
    let src = std::fs::read_to_string("../../../auto-code/src/tool_file_write.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("tool_file_write", &src).unwrap();
    r.done().unwrap();
}

#[test]
fn test_autocode_tool_file_edit() {
    let src = std::fs::read_to_string("../../../auto-code/src/tool_file_edit.at").unwrap();
    let mut r = crate::trans::rust::transpile_rust("tool_file_edit", &src).unwrap();
    r.done().unwrap();
}

// === Language feature tests ===

#[test]
fn test_920_enum_as_fn_param() {
    // enum variants can be passed as function parameters and matched with is
    let src = r#"
pub enum ToolError {
    ExecutionFailed str
    InvalidInput str
}
pub fn handle(err ToolError) str {
    is err {
        ToolError.ExecutionFailed(msg) => { return msg }
        ToolError.InvalidInput(msg) => { return msg }
    }
}
"#;
    crate::trans::rust::transpile_rust("test", src).expect("enum as fn param should work");
}

#[test]
fn test_921_is_match_in_ext() {
    // is pattern matching works inside ext blocks
    let src = r#"
pub enum ToolError {
    ExecutionFailed str
    InvalidInput str
}
ext ToolError {
    pub fn to_string() str {
        is self {
            ToolError.ExecutionFailed(msg) => { return msg }
            ToolError.InvalidInput(msg) => { return msg }
        }
    }
}
"#;
    crate::trans::rust::transpile_rust("test", src).expect("is match in ext should work");
}

#[test]
fn test_922_or_keyword() {
    // 'or' keyword for logical OR (|| is not supported)
    let src = r#"
pub fn check(a int, b int) bool {
    return a > 0 or b > 0
}
"#;
    crate::trans::rust::transpile_rust("test", src).expect("or keyword should work");
}

#[test]
fn test_923_backtick_fstring() {
    // Backtick strings support multi-line and interpolation
    let src = r#"
pub fn test() str {
    let template = `hello
world`
    return template
}
"#;
    crate::trans::rust::transpile_rust("test", src).expect("backtick with newline should work");
}

#[test]
fn test_924_escaped_quotes_in_fstring() {
    // Escaped quotes inside f-strings
    let src = r#"
pub fn test() str {
    let template = "{\"env\": {}}"
    return template
}
"#;
    crate::trans::rust::transpile_rust("test", src).expect("escaped quotes in string should work");
}

#[test]
fn test_925_option_bool_field() {
    // Option<bool> as struct field type
    let src = r#"
pub type Settings {
    env Map<str, str>
    provider Option<str>
}
ext Settings {
    pub static fn default() Settings {
        return Settings({}, nil)
    }
}
"#;
    crate::trans::rust::transpile_rust("test", src).expect("Option type field should work");
}

#[test]
fn test_926_const_declaration() {
    // const declarations before ext blocks
    let src = r#"
pub type Settings {
    env Map<str, str>
}
const SETTINGS_DIR str = ".auto-code-rs"
ext Settings {
    pub static fn default() Settings {
        return Settings({})
    }
}
"#;
    crate::trans::rust::transpile_rust("test", src).expect("const before ext should work");
}

#[test]
fn test_927_empty_body_with_comment() {
    // ext method with empty body (just a comment) is valid
    let src = r#"
pub type Settings {
    env Map<str, str>
}
ext Settings {
    pub static fn default() Settings {
        return Settings({})
    }
    pub static fn inject_env(settings Settings) {
        // Iterate over env map and set each key-value pair
    }
}
"#;
    crate::trans::rust::transpile_rust("test", src).expect("empty body with comment should work");
}

#[test]
fn test_928_self_field_access() {
    // .field syntax for self access in ext methods
    let src = r#"
pub type Counter {
    count int
}
ext Counter {
    pub fn increment() void {
        .count = .count + 1
    }
    pub fn get() int {
        return .count
    }
}
"#;
    crate::trans::rust::transpile_rust("test", src).expect("self field access should work");
}

#[test]
fn test_929_is_non_exhaustive() {
    // is match without else branch is allowed
    let src = r#"
pub fn check(x int) str {
    is x {
        0 => { return "zero" }
        1 => { return "one" }
    }
    return "other"
}
"#;
    crate::trans::rust::transpile_rust("test", src).expect("non-exhaustive is match should work");
}

#[test]
fn test_930_fn_result_enum() {
    // Functions returning Result with enum error type
    let src = r#"
pub enum ToolError {
    ExecutionFailed str
    InvalidInput str
}
pub fn execute(input str) Result<str, ToolError> {
    if input == "" {
        return Err(ToolError.InvalidInput("empty"))
    }
    return Ok(input)
}
"#;
    crate::trans::rust::transpile_rust("test", src).expect("fn with enum Result should work");
}

// === Known limitations (tests that document unsupported features) ===

#[test]
fn test_940_left_shift_not_supported() {
    // << operator is not supported by the parser
    let src = r#"
pub fn test() int {
    var base_secs = 1
    var i = 0
    for i < 3 {
        base_secs = base_secs * 2
        i = i + 1
    }
    return base_secs
}
"#;
    crate::trans::rust::transpile_rust("test", src).expect("multiply-by-2 loop workaround should work");
}

#[test]
fn test_941_tuple_in_generic_not_supported() {
    // Tuple types inside generics fail: List<(str, str)>, Result<(int, str), E>
    // Workaround: define a named type instead
    let src = r#"
pub type Message {
    role str
    content str
}
pub fn load() Result<List<Message>, str> {
    var messages = List.new()
    return Ok(messages)
}
"#;
    crate::trans::rust::transpile_rust("test", src).expect("named type instead of tuple in generic should work");
}

#[test]
fn test_942_ext_is_keyword() {
    // 'ext' is a reserved keyword, cannot be used as variable name
    let src = r#"
pub fn test() int {
    let file_ext = ".rs"
    return 0
}
"#;
    crate::trans::rust::transpile_rust("test", src).expect("ext as keyword not used as var should work");
}

// Test each file with full error details — shows exact byte offset, line, and source
#[test]
fn test_911_detailed_errors() {
    use crate::parser::{Parser, CompileDest};

    let base = "../../../auto-code/src/";
    let files = [
        "tools", "sse", "context", "settings",
        "agent", "anthropic", "openai", "session", "repl", "main",
        "tool_bash", "tool_grep", "tool_file_read", "tool_file_write", "tool_file_edit",
    ];

    for name in &files {
        let path = format!("{}{}.at", base, name);
        let src = std::fs::read_to_string(&path).unwrap();
        let mut parser = Parser::from(&src);
        parser.set_dest(CompileDest::TransRust);
        match parser.parse() {
            Ok(_) => println!("OK: {}", name),
            Err(e) => {
                let err_str = format!("{:?}", e);
                let offset = extract_offset(&err_str);
                let (line, col, source_line) = offset_to_line_col(&src, offset);
                println!("FAIL: {} — byte {} = line {} col {}", name, offset, line, col);
                println!("  | {}", source_line.trim_end());
                println!("  | {:>width$}", "^", width = col);
            }
        }
    }
}

fn extract_offset(s: &str) -> usize {
    if let Some(pos) = s.find("SourceOffset(") {
        let rest = &s[pos + 13..];
        if let Some(end) = rest.find(")") {
            return rest[..end].parse().unwrap_or(0);
        }
    }
    0
}

fn offset_to_line_col(src: &str, offset: usize) -> (usize, usize, String) {
    let mut line = 1;
    let mut last_newline = 0;
    for (i, ch) in src.char_indices() {
        if i == offset {
            return (line, i - last_newline + 1, get_line(src, offset));
        }
        if ch == '\n' {
            line += 1;
            last_newline = i + 1;
        }
    }
    (line, 0, String::new())
}

fn get_line(src: &str, offset: usize) -> String {
    let line_start = src[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = src[offset..].find('\n').map(|i| offset + i).unwrap_or(src.len());
    src[line_start..line_end].to_string()
}
