//! PLAN-1064 T-MODULE-I — the MODULE seam's permanent probes (round 1).
//!
//! Seam diagnosed and closed this round: the packages-modules grammar
//! parsed for/if conditions with the greedy `expr`, so `x < ident {`
//! committed the block brace as a record ctor (`ident {`) and the
//! recovery misattributed "`fn` is not part of this source profile" to
//! the enclosing item — json.at/redis.at (loop-driven stdlib authors)
//! were unparseable as package modules. Fix: the record family's
//! `condition-expr` (restricted condition grammar, no ctor commit) now
//! also serves packages-modules for if-cond and for-cond (grammar rule
//! + pratt table + condition-unary/-postfix/-ctor/-atom widened).
//!
//! Verified here: json.at parses as a package module (19 pub exports,
//! 8 private helpers declared but NOT exported), the app::jsond module
//! compiles through compile_package_product with a cross-module pub
//! consumption, and the private-visibility negative — importing a
//! private helper is the NAMED `source.module-private` rejection.
//! The module-scope `use demo` + `demo.answer()` qualified form of the
//! §5.2 demo is the named remainder (resolve has no module-namespace
//! bindings yet); the item-import spelling is the current-Auto face.
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    parse_package_source, PackageModuleInput, PACKAGES_CAPABILITY,
};
use ace_source_query::modules::packages::compile_package_product;

const JSON_AT: &str = include_str!("../../../../../stdlib/ace/std/json.at");
const REDIS_AT: &str = include_str!("../../../../../stdlib/ace/std/redis.at");

fn caps() -> [String; 1] {
    [PACKAGES_CAPABILITY.to_string()]
}

fn input(package: &str, id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: package.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

/// json.at parses as a package module; the pub/private split projects
/// exactly (19 exports, private helpers declared but not exported).
#[test]
fn module_1064_json_at_parses_with_pub_private_split() {
    let m = parse_package_source(JSON_AT, &caps(), FrontBudget::WORK_OWNED_64MIB)
        .expect("json.at parses as a package module");
    assert_eq!(m.exports.len(), 19, "the 19 pub fns of json.at");
    for private in [
        "json_cp",
        "json_ws",
        "json_str_end",
        "json_num_end",
        "json_exp_end",
        "json_hex4",
        "json_path_value",
        "json_skip_value",
    ] {
        assert!(
            m.declarations.contains_key(private),
            "{private} is declared in-module"
        );
        assert!(
            !m.exports.contains(private),
            "{private} must NOT be exported"
        );
    }
    assert!(m.exports.contains("json_is_valid"));
    assert!(m.exports.contains("json_parse_str"));
}

/// redis.at parses the same way (loop-driven author source).
#[test]
fn module_1064_redis_at_parses_as_package_module() {
    let m = parse_package_source(REDIS_AT, &caps(), FrontBudget::WORK_OWNED_64MIB)
        .expect("redis.at parses as a package module");
    assert!(m.exports.contains("redis_get"));
    assert!(m.exports.contains("redis_cmd_get"));
}

/// The closed seam end-to-end: the jsond module (json.at VERBATIM) is
/// consumed cross-module by app::main through compile_package_product.
#[test]
fn module_1064_json_module_consumed_cross_module() {
    let inputs = vec![
        input("app", "jsond", JSON_AT),
        input(
            "app",
            "main",
            "use app.jsond.json_is_valid\npub fn main() int {\n    if json_is_valid(\"{\\\"a\\\":1}\") { return 42 }\n    return 1\n}",
        ),
    ];
    let product = compile_package_product(
        &inputs,
        "app::main",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    )
    .expect("the json.at packages seam is closed (condition-expr widening)");
    assert!(
        product
            .compiled
            .checked
            .bundle()
            .modules
            .iter()
            .any(|m| m.defs.iter().any(|d| d.id_text == "d_main")),
        "the consuming app::main reaches checked HIR"
    );
}

/// The §5.2 demo in the current-Auto item-import spelling: the private
/// helper is callable IN-module (pub answer -> helper), the pub item is
/// importable cross-module, and importing the PRIVATE helper is the
/// NAMED visibility rejection.
#[test]
fn module_1064_demo_visibility_item_form() {
    let demo = "fn helper(x int) int { return x + 1 }\npub fn answer() int { return helper(41) }";
    // in-module private use + cross-module pub import: compiles
    let ok = compile_package_product(
        &[
            input("app", "demo", demo),
            input(
                "app",
                "main",
                "use demo.answer\npub fn main() int { return answer() }",
            ),
        ],
        "app::main",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    );
    assert!(
        ok.is_ok(),
        "demo pub consumption compiles: {:?}",
        ok.err()
            .map(|ds| ds.iter().map(|d| d.diagnostic.message.clone()).collect::<Vec<_>>())
    );

    // cross-module PRIVATE import: named rejection
    let err = match compile_package_product(
        &[
            input("app", "demo", demo),
            input(
                "app",
                "main",
                "use demo.helper\npub fn main() int { return helper(1) }",
            ),
        ],
        "app::main",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        Ok(_) => panic!("importing a private helper must be rejected"),
        Err(ds) => ds,
    };
    assert!(
        err.iter().any(|d| d.diagnostic.code == "source.module-private"),
        "named module-private rejection, got {:?}",
        err.iter().map(|d| d.diagnostic.code.clone()).collect::<Vec<_>>()
    );
}

/// The grammar fix's construct matrix (the shapes the stdlib authors
/// use in loop bodies) — all must parse as package modules.
#[test]
fn module_1064_author_constructs_parse() {
    let snippets: &[(&str, &str)] = &[
        ("method_call", "pub fn f(s Str) int {\n    let c = s.char_at(0)\n    return 1\n}"),
        ("escape_lit", "pub fn f() Str {\n    return \"\\\"\"\n}"),
        ("nested_if", "pub fn f(x int) int {\n    if x == 1 { return 1 } else { return 2 }\n}"),
        ("var_init", "pub fn f() int {\n    var out int = 0\n    return out\n}"),
        ("var_infer", "pub fn f() int {\n    var i = 0\n    return i\n}"),
        ("for_empty", "pub fn f(n int) int {\n    for n < 0 { }\n    return n\n}"),
        ("for_lit_brace", "pub fn f(n int) int {\n    var i int = 0\n    for i < 3 {\n        i = i + 1\n    }\n    return i\n}"),
        ("for_ident_brace", "pub fn f(n int) int {\n    var i int = 0\n    for i < n {\n        i = i + 1\n    }\n    return i\n}"),
        ("if_ident_brace", "pub fn f(n int, x int) int {\n    var i int = 0\n    if n > x {\n        i = i + 1\n    }\n    return i\n}"),
        ("assign_stmt", "pub fn f() int {\n    var i int = 0\n    i = i + 1\n    return i\n}"),
        ("private_fn", "fn helper(x int) int { return x + 1 }\npub fn main() int { return helper(1) }"),
    ];
    let mut failures: Vec<String> = Vec::new();
    for (name, src) in snippets {
        if let Err(ds) = parse_package_source(src, &caps(), FrontBudget::WORK_OWNED_64MIB) {
            failures.push(format!("{name}: {}", ds[0].message));
        }
    }
    assert!(
        failures.is_empty(),
        "author constructs rejected:\n{}",
        failures.join("\n")
    );
}

/// The §5.2 demo VERBATIM (module-scope form): `use demo` declares the
/// dependency; `demo.answer()` is the qualified call; the private helper
/// stays callable in-module. Cross-module `demo.helper(41)` is the named
/// visibility rejection (the qualified call's own span).
#[test]
fn module_1064_demo_module_scope_form() {
    let demo = "fn helper(x int) int { return x + 1 }
pub fn answer() int { return helper(41) }";
    // positive: the contract's exact shape compiles
    let ok = compile_package_product(
        &[
            input("app", "demo", demo),
            input(
                "app",
                "main",
                "use demo
pub fn main() int { return demo.answer() }",
            ),
        ],
        "app::main",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    );
    assert!(
        ok.is_ok(),
        "the §5.2 module-scope demo compiles: {:?}",
        ok.err()
            .map(|ds| ds.iter().map(|d| format!("{}: {}", d.diagnostic.code, d.diagnostic.message)).collect::<Vec<_>>())
    );

    // negative: the qualified call to the PRIVATE helper is named-rejected
    let err = match compile_package_product(
        &[
            input("app", "demo", demo),
            input(
                "app",
                "main",
                "use demo
pub fn main() int { return demo.helper(41) }",
            ),
        ],
        "app::main",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        Ok(_) => panic!("demo.helper must be visibility-rejected"),
        Err(ds) => ds,
    };
    assert!(
        err.iter().any(|d| d.diagnostic.code == "source.module-private"),
        "named module-private rejection on the qualified call, got {:?}",
        err.iter().map(|d| d.diagnostic.code.clone()).collect::<Vec<_>>()
    );

    // negative: a module namespace is not directly callable
    let err2 = match compile_package_product(
        &[
            input("app", "demo", demo),
            input(
                "app",
                "main",
                "use demo
pub fn main() int { return demo() }",
            ),
        ],
        "app::main",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        Ok(_) => panic!("calling a module namespace must be rejected"),
        Err(ds) => ds,
    };
    assert!(
        err2.iter().any(|d| d.diagnostic.code == "source.module-symbol"),
        "named module-symbol rejection, got {:?}",
        err2.iter().map(|d| d.diagnostic.code.clone()).collect::<Vec<_>>()
    );

    // negative: whole-module re-export is a named rejection
    let err3 = match compile_package_product(
        &[
            input("app", "demo", demo),
            input(
                "app",
                "main",
                "pub use demo
pub fn main() int { return 0 }",
            ),
        ],
        "app::main",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        Ok(_) => panic!("whole-module re-export must be rejected"),
        Err(ds) => ds,
    };
    assert!(
        err3.iter().any(|d| d.diagnostic.code == "source.module-path"),
        "named module-path rejection for module re-export, got {:?}",
        err3.iter().map(|d| d.diagnostic.code.clone()).collect::<Vec<_>>()
    );

    // negative: `use demo` of an unknown module is named-missing
    let err4 = match compile_package_product(
        &[input(
            "app",
            "main",
            "use nope
pub fn main() int { return 0 }",
        )],
        "app::main",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        Ok(_) => panic!("unknown module import must be rejected"),
        Err(ds) => ds,
    };
    assert!(
        err4.iter().any(|d| d.diagnostic.code == "source.module-missing"),
        "named module-missing rejection, got {:?}",
        err4.iter().map(|d| d.diagnostic.code.clone()).collect::<Vec<_>>()
    );
}
