//! Compare the native Python reference with both fixed case oracles.
#![cfg(windows)]

use std::path::Path;
use std::process::Command;

use ace_core::runtime_contract::sha256_hex;

const CORPUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../../crates/auto-lang/test/cases/language/script/python-slicing.case.ad"
));
const REFERENCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../../crates/auto-lang/test/cases/ace-m2/python-1127/slicing_ref.py"
);

fn fence<'a>(section: &'a str, language: &str) -> &'a str {
    let marker = format!("```{language}\n");
    let (_, body) = section.split_once(&marker).expect("opening fence");
    body.split_once("```").expect("closing fence").0
}

fn check_reference(name: &str, hash: &str, failure_site: Option<&str>) {
    let heading = format!("\n## Case: {name}\n");
    let (_, tail) = CORPUS.split_once(&heading).expect("named case");
    let case = tail.split("\n## Case: ").next().expect("case body");
    assert!(case.contains("\npath: \"main.as\"\n"));
    let source = fence(case, "auto");
    assert_eq!(
        sha256_hex(source.as_bytes()),
        hash,
        "{name}: author source changed"
    );

    let (_, checks) = case.split_once("### Probe av\n").expect("AV probe");
    let (av, a2py) = checks.split_once("### Probe a2py\n").expect("A2PY probe");
    if let Some(expression) = failure_site {
        let start = source.find(expression).expect("failure expression");
        let expected = format!("error-span: \"{}..{}\"", start, start + expression.len());
        assert!(
            av.contains(&expected) && a2py.contains(&expected),
            "{name}: {expected}"
        );
    }
    let failed = a2py.lines().any(|line| line == "exit: 1");
    let python = std::env::var_os("A2PY_PYTHON").unwrap_or_else(|| "python".into());
    let output = Command::new(python)
        .args(["-I", "-B"])
        .arg(Path::new(REFERENCE))
        .arg(name)
        .output()
        .expect("run the declared interpreter with NumPy and PyTorch");
    let stderr = String::from_utf8(output.stderr)
        .expect("reference stderr")
        .replace("\r\n", "\n");
    assert_eq!(
        output.status.code(),
        Some(if failed { 1 } else { 0 }),
        "{stderr}"
    );
    let stdout = String::from_utf8(output.stdout)
        .expect("reference stdout")
        .replace("\r\n", "\n");
    assert_eq!(stdout, fence(av, "text"), "{name}: AV oracle");
    let a2py_stdout = fence(a2py, "text");
    assert_eq!(
        a2py_stdout,
        if failed { stdout } else { stdout + "value:0\n" },
        "{name}: A2PY oracle",
    );
    let (_, a2py_stderr) = a2py.split_once("#### stderr\n").expect("A2PY stderr");
    assert_eq!(
        stderr,
        fence(a2py_stderr, "text"),
        "{name}: reference error"
    );
}

#[test]
fn native_python_numpy_torch_values() {
    check_reference(
        "numpy-torch",
        "e78755f68cc1feff75d24c0033f4bb13107e85dc7d0c5b9f7a80c989c8d805fd",
        None,
    );
}

#[test]
fn native_python_numpy_zero() {
    check_reference(
        "numpy-zero",
        "ec4ff74605a3834efb3ffb6bab727e2e7895be0fbc88a3e7c335b7dfa4adc108",
        Some("row[::zero]"),
    );
}

#[test]
fn native_python_torch_zero() {
    check_reference(
        "torch-zero",
        "35696dc58c4829b15496ad4cb9961076e002ce2be5b19bac3595468b817c0757",
        Some("row[::0]"),
    );
}

#[test]
fn native_python_torch_negative() {
    check_reference(
        "torch-negative",
        "5272ae419c3dffa47885833ea6b4e311eebbe512cd4075bcf772a8471b20ea7f",
        Some("row[::-1]"),
    );
}

#[test]
fn native_python_bound_error() {
    check_reference(
        "bound-error",
        "a85cab8f33d5c04ae6db55b6ce7b8260233fd5d3041afae843c72852d6f69db4",
        Some(r#"loads("{")"#),
    );
}

#[test]
fn native_python_bound_error_shifted() {
    check_reference(
        "bound-error-shifted",
        "0b9b15f08c4dcb987ade2c4f2d1a607ea8f499fdca441bd6e30fda77848cbb8b",
        Some(r#"loads("{")"#),
    );
}

#[test]
fn native_python_source_order_and_compatible_indexing() {
    check_reference(
        "order-compatibility",
        "51208edd3572a87cd336c8451e830d1c10ee2e7b6b77b6b5551c49ee7eff54d3",
        None,
    );
}
