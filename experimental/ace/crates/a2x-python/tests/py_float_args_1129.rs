//! PLAN-1129: the A2PY lane hands Python the values Python must receive —
//! a runtime float as the number Auto computed (an `f32` at f32 precision),
//! a literal as its own bits, an Auto text value as a Python `str` — and the
//! independent native reference agrees with both corpus oracles.
#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::process::Command;

use ace_a2x_core::{Family, FilesetRequest, FragmentText, SemanticView, TargetProvider};
use ace_a2x_python::py_contract::py_declaration_contract;
use ace_a2x_python::PythonTargetProvider;
use ace_core::runtime_contract::sha256_hex;
use ace_foreign_contract::manual::SOURCE_CAP;
use ace_foreign_contract::resource::RESOURCE_CAP;

const CORPUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../../crates/auto-lang/test/cases/language/script/python-float-args.case.ad"
));
const REFERENCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../../crates/auto-lang/test/cases/ace-m2/python-1129/float_args_ref.py"
);

fn fence<'a>(section: &'a str, language: &str) -> &'a str {
    let marker = format!("```{language}\n");
    let (_, body) = section.split_once(&marker).expect("opening fence");
    body.split_once("```").expect("closing fence").0
}

fn case_of(name: &str) -> (&'static str, &'static str, &'static str) {
    let heading = format!("\n## Case: {name}\n");
    let (_, tail) = CORPUS.split_once(&heading).expect("named case");
    let case = tail.split("\n## Case: ").next().expect("case body");
    assert!(case.contains("\npath: \"main.as\"\n"));
    let source = fence(case, "auto");
    let (_, checks) = case.split_once("### Probe av\n").expect("AV probe");
    let (av, a2py) = checks.split_once("### Probe a2py\n").expect("A2PY probe");
    (source, av, a2py)
}

fn declared_python() -> std::ffi::OsString {
    std::env::var_os("A2PY_PYTHON").unwrap_or_else(|| "python".into())
}

/// The native reference's stdout and stderr for one case, run by the
/// declared interpreter with no compiler output in sight.
fn reference(name: &str) -> (String, String, Option<i32>) {
    let output = Command::new(declared_python())
        .args(["-I", "-B"])
        .arg(Path::new(REFERENCE))
        .arg(name)
        .output()
        .expect("run the declared A2PY interpreter");
    let text = |bytes: &[u8]| String::from_utf8(bytes.to_vec()).expect("utf-8").replace("\r\n", "\n");
    (
        text(&output.stdout),
        text(&output.stderr),
        output.status.code(),
    )
}

fn check_reference(name: &str, hash: &str, failing: Option<&str>) {
    let (source, av, a2py) = case_of(name);
    assert_eq!(sha256_hex(source.as_bytes()), hash, "{name}: author source changed");

    // a program that fails instead of catching ends the run on both ends
    let uncaught = a2py.lines().any(|line| line == "exit: 1");
    assert_eq!(uncaught, av.contains("code: \"lang.error\""), "{name}: both ends agree");
    if let Some(expression) = failing {
        // the failing call expression, as a half-open UTF-8 byte range
        let start = source.find(expression).expect("failing expression");
        let expected = format!(
            "error-span: \"{}..{}\"",
            start,
            start + expression.len()
        );
        assert!(
            av.contains(&expected) && a2py.contains(&expected),
            "{name}: {expected}"
        );
        for section in [av, a2py] {
            assert!(section.contains("error-kind: \"python.5\""), "{name}: ValueError class");
        }
        assert!(a2py.contains("error-type: \"ValueError\""), "{name}");
    }
    let (stdout, stderr, code) = reference(name);
    assert_eq!(code, Some(i32::from(uncaught)), "{name}: {stderr}");
    assert_eq!(stdout, fence(av, "text"), "{name}: AV oracle");
    if uncaught {
        assert!(av.contains(&format!("message: \"{}\"", stderr.trim_end())), "{name}: the AV failure text");
        assert_eq!(fence(a2py, "text"), "", "{name}: no A2PY stdout on failure");
    } else {
        assert_eq!(
            fence(a2py, "text"),
            format!("{stdout}value:0\n"),
            "{name}: A2PY oracle"
        );
    }
    let (_, a2py_stderr) = a2py.split_once("#### stderr\n").expect("A2PY stderr");
    assert_eq!(stderr, fence(a2py_stderr, "text"), "{name}: reference stderr");
}

#[test]
fn native_python_runtime_values() {
    check_reference(
        "runtime-values",
        "a97e276e7c8645f1a924371ee7070dd06f2bc27ff7c56ebd90bbe14627b79fde",
        None,
    );
}

#[test]
fn native_python_types_and_text() {
    check_reference(
        "types-and-text",
        "534b96da737e6ae578e3c1d2cc5688e449126d539634658340dbcc2ef834c8ba",
        None,
    );
}

#[test]
fn native_python_order_and_once() {
    check_reference(
        "order-and-once",
        "7f0a8020acfbabcda2b993233c3142a0695fcf6281dba7158cead614232b3135",
        Some("sqrt(mark(\"neg\", -4.0f))"),
    );
}

#[test]
fn native_python_rounds_first() {
    check_reference(
        "rounds-first",
        "6f00d482e1a5fbc2ad0412082e30a6f5cabc46500e84a61884a11eb8e78bafae",
        None,
    );
}

#[test]
fn native_python_boundaries() {
    check_reference(
        "boundaries",
        "98a57e11f6c743c47bfbecbea9f8f637d42cda8d2a76457fd066971cca710b0f",
        None,
    );
}

/// The sign of a zero the program computed, on the f32 lane: a product that
/// makes `-0.0f`, the same zero widened, and the positive zero, each read
/// back as its exact bits.
#[test]
fn native_python_signed_zero_f32() {
    check_reference(
        "signed-zero-f32",
        "e8ebe7ac2b853da647f58b401ec438cb4a79b32ceb889bf341339e9b9ecec0f2",
        None,
    );
}

/// A float on the right of a Python object's operator (`py_binop`): the
/// standard library's exact rational receives the f32 the program computed.
#[test]
fn native_python_binop_float() {
    check_reference(
        "binop-float",
        "7a409039a9ef397867984c17632a236a17e594643951975b4c1cf95efd01aa81",
        None,
    );
}

#[test]
fn native_python_containers_and_binop() {
    check_reference(
        "containers-and-binop",
        "09b166f2ad8d29f385083e40334ce1ef366a91ce9c049f4ab89bbab0a41ca79f",
        None,
    );
}

/// CPU PyTorch: the optimizer's own `param_groups[0]["lr"]` equals the rate
/// each run computed from its input, and the loss path and final prediction
/// of the two runs match the native training value for value.
#[test]
fn native_python_torch_computed_learning_rate() {
    check_reference(
        "torch-lr-computed",
        "8362f511ee12b30953182f5f0f73a74d59a4ecf40d4496d588dd8423c1c550a4",
        None,
    );
}

#[test]
fn native_python_sqrt_runtime_negative() {
    check_reference(
        "sqrt-runtime-negative",
        "6927153424ed25121ebe7f477db0c067031093b8c000230b8d32067e1531477c",
        Some("sqrt(value)"),
    );
}

#[test]
fn native_python_sqrt_uncaught() {
    check_reference(
        "sqrt-uncaught",
        "11baf93c6fecaa497a2477ccc233f1dad6bd3412536dd0619b57a09bfceba1cc",
        Some("sqrt(value)"),
    );
}

#[test]
fn native_python_sqrt_nested_fails_first() {
    check_reference(
        "sqrt-nested-first",
        "c7e365534a0dc59fd23e209ca656ebda1ceed4ee2b2673302e7713fca175599d",
        Some("sqrt(negate(4.0f))"),
    );
}

/// The same failure behind a multibyte source prefix: the reported range
/// moves by the prefix's BYTE length.
#[test]
fn native_python_sqrt_nested_shifted() {
    check_reference(
        "sqrt-nested-shifted",
        "37f0eb8b8a65071d79b25b95c8aac79aca846a5c81170df78163b16fd1d503e4",
        Some("sqrt(negate(4.0f))"),
    );
}

// ---- the generated Python itself ------------------------------------------------

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("p1129_a2py_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Lower one `use.py` source to a native Python fileset (no worker, no
/// interpreter) and return the directory it was written to.
fn emit(dir: &Path, src: &str) {
    let contract = py_declaration_contract().expect("declaration contract");
    let caps = vec![SOURCE_CAP.to_string(), RESOURCE_CAP.to_string()];
    let compiled = ace_core::source::compile_language_source_with_foreign(src, "main.as", &caps, &contract)
        .unwrap_or_else(|d| panic!("compile: {d:?}"));
    let view = SemanticView::from_checked(&compiled.checked).expect("view");
    let provider = PythonTargetProvider;
    provider
        .validate_view(&view)
        .expect("py declarations are language operations");
    let runtime = provider.runtime_source(&view, Family::Collections).expect("runtime");
    assert!(runtime.contains("def a2x_f32_round("), "the f32 rounding helper");
    assert!(runtime.contains("def a2x_py_text("), "the text boundary helper");
    let fns: Vec<&ace_a2x_core::FnFacts> = view
        .functions()
        .iter()
        // the AV frame helpers are never lowered for a source target
        .filter(|f| !ace_a2x_python::provider::is_py_lane_helper(f.name))
        .collect();
    let texts = provider
        .produce_fragments(&view, &fns, Family::Collections)
        .unwrap_or_else(|e| panic!("lower: {e:?}"));
    let defs: Vec<FragmentText> = fns
        .iter()
        .zip(texts.iter())
        .map(|(f, t)| FragmentText {
            def_index: f.def_index,
            source_name: f.source_name,
            sem_fp: "",
            text: t.as_str(),
        })
        .collect();
    let fileset = provider
        .assemble_fileset(FilesetRequest {
            entry: "main",
            input_hash: "p1129",
            family: Family::Collections,
            fragments: &defs,
            view: Some(&view),
            runtime: &runtime,
            prelude_seed: None,
            origins: None,
            source_sha256: None,
        })
        .unwrap_or_else(|e| panic!("assemble: {e:?}"));
    for (name, content) in &fileset.files {
        std::fs::write(dir.join(name), content).unwrap();
    }
}

fn run_generated(dir: &Path) -> String {
    let output = Command::new(declared_python())
        .args(["-I", "-B"])
        .arg(dir.join("a2x_main.py"))
        .output()
        .expect("run the generated program");
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout.to_vec())
        .expect("utf-8")
        .replace("\r\n", "\n")
}

/// The VALUE argument of every `a2x_py_call(.., l0_pack, None, [<fmt>, <value>], {})`
/// line, in source order.
fn pack_arguments(generated: &str) -> Vec<String> {
    generated
        .lines()
        .filter(|line| line.contains("a2x_py_call") && line.contains("l0_pack, None, ["))
        .filter_map(|line| line.split("l0_pack, None, [").nth(1))
        .filter_map(|rest| rest.split("], {})").next())
        .filter_map(|args| args.splitn(2, ", ").nth(1))
        .map(|arg| arg.trim().to_string())
        .collect()
}

/// The variable a py-call argument names, once its boundary conversion is
/// peeled off.
fn variable(arg: &str) -> &str {
    arg.strip_prefix("a2x_f32_round(")
        .and_then(|rest| rest.strip_suffix(')'))
        .unwrap_or(arg)
}

#[test]
fn generated_python_passes_the_runtime_value_and_never_a_remembered_literal() {
    let dir = scratch("values");
    let (source, av, _) = case_of("runtime-values");
    emit(&dir, source);
    let generated = std::fs::read_to_string(dir.join("generated.py")).unwrap();

    let args = pack_arguments(&generated);
    // nine `pack(<fmt>, <value>)` calls in source order: four runtime
    // values, then five literals
    assert_eq!(args.len(), 9, "{args:?}");
    let (runtime, literals) = args.split_at(4);
    // `rate` (reassigned, then computed), `lr` (computed), the result of
    // `scale(..)` and the reassigned `again` cross by the variable that
    // holds them — never by a number the compiler remembered.
    for arg in runtime {
        let name = variable(arg);
        assert!(
            !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_'),
            "{arg} names a variable, not a folded number"
        );
    }
    // the f32 operands take the width conversion at the boundary; `lr`, an
    // f64, needs none
    assert!(runtime[0].starts_with("a2x_f32_round("), "{args:?}");
    assert!(runtime[2].starts_with("a2x_f32_round("), "{args:?}");
    assert!(runtime[3].starts_with("a2x_f32_round("), "{args:?}");
    assert_eq!(variable(&runtime[1]), runtime[1], "{args:?}");
    // the literals are their own bits — the f32 0.1 is not the double 0.1,
    // and the zeros keep their signs
    assert_eq!(
        literals,
        ["0.10000000149011612", "0.1", "-0.0", "-0.0", "0.0"]
    );

    // the reassigned local is assigned its initializer, then its new value,
    // and the call that follows reads the local
    let initializer = generated
        .lines()
        .find(|line| line.contains("a2x_f32_bits(1048576000)"))
        .expect("the initializer 0.25f32");
    let local = initializer.split(" = ").next().unwrap().trim();
    assert!(local.contains("again"), "{initializer}");
    let after = generated.split_once(initializer).expect("after the initializer").1;
    assert!(
        after.contains(&format!("{local} = a2x_f32_bits(1056964608)")),
        "the reassignment to 0.5f32 precedes the call: {after}"
    );
    assert!(
        runtime[3].contains(&format!("{local})")),
        "the call reads the local: {}",
        runtime[3]
    );

    // f32 arithmetic is rounded at every step
    assert!(generated.contains("a2x_f32_round(("), "f32 add / sub / mul round");

    assert_eq!(
        run_generated(&dir),
        format!("{}value:0\n", fence(av, "text"))
    );
    let _ = std::fs::remove_dir_all(&dir);

    // the division case: `third = 1.0f / 3.0f` is rounded at the step it
    // happens, and so is each following f32 multiply; the f64 division
    // `(0.1d / 3.0d) * 3.0d` takes no rounding at all
    let dir = scratch("division");
    let (source, av, _) = case_of("rounds-first");
    emit(&dir, source);
    let generated = std::fs::read_to_string(dir.join("generated.py")).unwrap();
    let third = "a2x_f32_round(a2x_fdiv(a2x_f32_bits(1065353216), a2x_f32_bits(1077936128)))";
    assert!(
        generated.contains(third),
        "an f32 division is rounded at the step it happens: {generated}"
    );
    assert!(
        generated.contains("a2x_f32_round((l1_third * a2x_f32_bits(1077936128)))"),
        "and the next f32 multiply too: {generated}"
    );
    assert!(
        generated.contains("t9 = a2x_fdiv(a2x_f64_bits(4591870180066957722), a2x_f64_bits(4613937818241073152))"),
        "an f64 division is exact as it stands: {generated}"
    );
    assert_eq!(
        run_generated(&dir),
        format!("{}value:0\n", fence(av, "text"))
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn generated_python_hands_python_the_text_value() {
    let dir = scratch("text");
    let (source, av, _) = case_of("types-and-text");
    emit(&dir, source);
    let generated = std::fs::read_to_string(dir.join("generated.py")).unwrap();
    // a text the program computed is an Auto value: it crosses through the
    // boundary conversion, as the `str` it holds
    assert!(
        generated.contains("a2x_py_text("),
        "a computed text argument is converted at the boundary"
    );
    // a text literal crosses as the literal it is
    assert!(
        generated.contains(r#"["F:1.25", "__class__"]"#),
        "a text literal stays a literal"
    );
    assert_eq!(
        run_generated(&dir),
        format!("{}value:0\n", fence(av, "text"))
    );
    let _ = std::fs::remove_dir_all(&dir);
}
