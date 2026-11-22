//! PLAN-1126: the independent native-Python references for the shared
//! `python-libraries` corpus agree with the SAME frozen oracle the AV and A2PY
//! case views are judged against.
//!
//! The corpus (`language/script/python-libraries.case.ad`) is the ONE author of
//! the Auto sources and of the expected streams; the two real lanes run through
//! `case-av-language-parity` / `case-a2py-language-parity`. These tests add the
//! third party: handwritten Python that calls NumPy and `json` directly (never
//! ACE output), executed by the OS interpreter and compared by VALUE — integers
//! and text exactly, the mean within the plan's `rtol=atol=1e-6`.
#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::process::Command;

use ace_core::runtime_contract::sha256_hex;
use serde_json::Value;

/// Embedded, so any edit to the corpus rebuilds and re-judges these tests.
const CORPUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../../crates/auto-lang/test/cases/language/script/python-libraries.case.ad"
));

const POSITIVE: &str = "numpy-json";
const SHAPE_ERROR: &str = "numpy-shape-error-caught";
const DECODE_ERROR: &str = "json-decode-error-uncaught";

/// The sha256 of each case's `### Auto` bytes (the acceptance contract's
/// source fingerprint; a drifted source must be re-pinned deliberately).
const POSITIVE_SOURCE_SHA256: &str =
    "10f1afcfe24554b5801d972e4c77bcbf05d405b4034e7f636099d96c63fd7cd6";
const SHAPE_ERROR_SOURCE_SHA256: &str =
    "78e343f1c2ebb276295c040ff9fb194829b6597147787be3d8cdb6d0c8b6d854";
const DECODE_ERROR_SOURCE_SHA256: &str =
    "e824ea8e110edcbca05bc7cd8fd5f4c74ba62aaff3c1a042df635ef58e487e07";

fn reference_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../crates/auto-lang/test/cases/ace-m2/python-1126")
}

fn find_python() -> String {
    for cand in [
        std::env::var("AUTO_PYTHON_HOME")
            .map(|h| format!("{h}\\python.exe"))
            .unwrap_or_default(),
        r"C:\Python314\python.exe".to_string(),
        "python".to_string(),
    ] {
        if cand.is_empty() {
            continue;
        }
        if let Ok(out) = Command::new(&cand).arg("--version").output() {
            if out.status.success() {
                return cand;
            }
        }
    }
    panic!("no usable Python interpreter (set AUTO_PYTHON_HOME)");
}

/// Run one reference script with the OS interpreter (a named failure when the
/// interpreter or a package is missing — never a skip).
fn run_reference(script: &str) -> (Option<i32>, Vec<u8>, Vec<u8>) {
    let out = Command::new(find_python())
        .arg("-B")
        .arg(reference_dir().join(script))
        .output()
        .expect("run the independent python reference");
    (out.status.code(), out.stdout, out.stderr)
}

// ---- the case document ----------------------------------------------------

/// `## Case: <name>` up to the next case (or the end of the document).
fn case(name: &str) -> &'static str {
    let start = CORPUS
        .find(&format!("## Case: {name}\n"))
        .unwrap_or_else(|| panic!("case `{name}` is missing from the corpus"));
    let rest = &CORPUS[start..];
    let end = rest[1..].find("\n## Case: ").map_or(rest.len(), |i| i + 1);
    &rest[..end]
}

/// The `### Auto` source: the first ```auto fence of the case, its bytes as
/// authored (the runner stages exactly these bytes as `main.as`).
fn source(name: &str) -> &'static str {
    let body = case(name);
    let start = body
        .find("```auto\n")
        .unwrap_or_else(|| panic!("case `{name}` has no ```auto source"));
    let rest = &body[start + "```auto\n".len()..];
    let end = rest.find("```").expect("the source fence closes");
    &rest[..end]
}

/// A `### <heading>` section up to the next `### `/`## ` heading (its `####`
/// stream blocks included).
fn section<'a>(text: &'a str, heading: &str) -> &'a str {
    let marker = format!("### {heading}\n");
    let start = text
        .find(&marker)
        .unwrap_or_else(|| panic!("section `### {heading}` is missing"));
    let rest = &text[start + marker.len()..];
    let end = rest.find("\n### ").map_or(rest.len(), |i| i + 1);
    &rest[..end]
}

/// One `key: value` attribute line of a section (quotes trimmed).
fn attr<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("\n{key}: ");
    let start = text.find(&prefix)? + prefix.len();
    let end = text[start..].find('\n').map_or(text.len(), |i| i + start);
    Some(text[start..end].trim().trim_matches('"'))
}

/// One `#### <name>` stream block: its declared `compare` mode and its text
/// (an empty fence is the empty string; a missing block is `None`).
fn stream<'a>(text: &'a str, name: &str) -> Option<(Option<&'a str>, &'a str)> {
    let heading = format!("#### {name}\n");
    let start = text.find(&heading)? + heading.len();
    let rest = &text[start..];
    let fence = rest.find("```text\n")?;
    let attrs = &rest[..fence];
    let compare = attrs
        .lines()
        .find_map(|line| line.strip_prefix("compare: "))
        .map(|value| value.trim().trim_matches('"'));
    let body = &rest[fence + "```text\n".len()..];
    let end = if body.starts_with("```") {
        0
    } else {
        body.find("\n```").expect("the stream fence closes") + 1
    };
    Some((compare, &body[..end]))
}

// ---- the value comparison -------------------------------------------------

/// What the author source extracts on each line of the positive output.
#[derive(Clone, Copy, Debug, PartialEq)]
enum LineKind {
    Int,
    Float,
    Bool,
    Text,
}

use LineKind::{Bool, Float, Int, Text};

/// `.to(int)` ×5, `.to(float)`, `.to(int)`, `.to(bool)`, `.to(str)` ×3.
const POSITIVE_SHAPE: [LineKind; 11] =
    [Int, Int, Int, Int, Int, Float, Int, Bool, Text, Text, Text];

/// `rtol = atol = 1e-6` (PLAN-1126 V-01: the mean is compared by value).
const MEAN_RTOL: f64 = 1e-6;
const MEAN_ATOL: f64 = 1e-6;

/// One native-reference JSON value against the Auto oracle line of `kind`.
fn agree(kind: LineKind, reference: &Value, oracle: &str) -> Result<(), String> {
    match kind {
        Int => {
            let want = reference
                .as_i64()
                .ok_or_else(|| format!("reference {reference} is not an integer"))?;
            let got = oracle
                .parse::<i64>()
                .map_err(|_| format!("oracle line {oracle:?} is not an integer"))?;
            if got == want {
                Ok(())
            } else {
                Err(format!("int {got} != reference {want}"))
            }
        }
        Float => {
            let want = reference
                .as_f64()
                .ok_or_else(|| format!("reference {reference} is not a number"))?;
            let got = oracle
                .parse::<f64>()
                .map_err(|_| format!("oracle line {oracle:?} is not a number"))?;
            if (got - want).abs() <= MEAN_ATOL + MEAN_RTOL * want.abs() {
                Ok(())
            } else {
                Err(format!(
                    "float {got} is outside rtol={MEAN_RTOL} atol={MEAN_ATOL} of {want}"
                ))
            }
        }
        Bool => {
            let want = reference
                .as_bool()
                .ok_or_else(|| format!("reference {reference} is not a bool"))?;
            let got = match oracle {
                "true" => true,
                "false" => false,
                other => return Err(format!("oracle line {other:?} is not an Auto bool")),
            };
            if got == want {
                Ok(())
            } else {
                Err(format!("bool {got} != reference {want}"))
            }
        }
        Text => {
            let want = reference
                .as_str()
                .ok_or_else(|| format!("reference {reference} is not text"))?;
            if oracle == want {
                Ok(())
            } else {
                Err(format!("text {oracle:?} != reference {want:?}"))
            }
        }
    }
}

/// The whole positive output: reference JSON lines against the Auto lines.
fn positive_agrees(reference: &[Value], oracle: &str) -> Result<(), String> {
    let lines: Vec<&str> = oracle.lines().collect();
    if lines.len() != POSITIVE_SHAPE.len() || reference.len() != POSITIVE_SHAPE.len() {
        return Err(format!(
            "line count: oracle {} / reference {} / contract {}",
            lines.len(),
            reference.len(),
            POSITIVE_SHAPE.len()
        ));
    }
    for (index, ((kind, want), got)) in POSITIVE_SHAPE
        .iter()
        .zip(reference.iter())
        .zip(lines.iter())
        .enumerate()
    {
        agree(*kind, want, got).map_err(|e| format!("line {index}: {e}"))?;
    }
    Ok(())
}

fn json_lines(stdout: &[u8]) -> Vec<Value> {
    String::from_utf8_lossy(stdout)
        .lines()
        .map(|line| {
            serde_json::from_str(line)
                .unwrap_or_else(|_| panic!("reference line `{line}` is not JSON"))
        })
        .collect()
}

fn last_line(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    text.trim_end()
        .rsplit('\n')
        .next()
        .unwrap_or("")
        .trim_end_matches('\r')
        .to_string()
}

// ---- V-01 / V-02 ----------------------------------------------------------

#[test]
fn p1126_v01_v02_native_reference_matches_the_frozen_oracle() {
    let (code, stdout, stderr) = run_reference("numpy_json_ref.py");
    assert_eq!(
        code,
        Some(0),
        "the reference failed: {}",
        String::from_utf8_lossy(&stderr)
    );
    assert!(stderr.is_empty(), "{}", String::from_utf8_lossy(&stderr));
    let reference = json_lines(&stdout);

    let body = case(POSITIVE);
    let av = section(body, "Probe av");
    let a2py = section(body, "Probe a2py");
    let (av_compare, av_stdout) = stream(av, "stdout").expect("the AV stdout oracle");
    assert_eq!(av_compare, None, "the AV stream oracle stays byte-exact");
    assert!(stream(av, "stderr").unwrap().1.is_empty());
    let (_, a2py_stdout) = stream(a2py, "stdout").expect("the A2PY stdout oracle");

    // The third party: NumPy's and `json`'s own values equal the frozen lines
    // BOTH lanes are judged against.
    positive_agrees(&reference, av_stdout)
        .unwrap_or_else(|e| panic!("native reference vs the frozen oracle: {e}"));

    // Both lanes freeze the same program output; the A2PY leg only appends
    // its run-observation line.
    assert_eq!(
        a2py_stdout,
        format!("{av_stdout}value:0\n"),
        "the A2PY oracle is the AV oracle plus the run-observation line"
    );
}

// ---- V-03 -----------------------------------------------------------------

#[test]
fn p1126_v03_native_reference_confirms_the_numpy_valueerror() {
    let (code, stdout, stderr) = run_reference("numpy_shape_error_ref.py");
    assert_eq!(
        code,
        Some(0),
        "the reference failed: {}",
        String::from_utf8_lossy(&stderr)
    );
    let reference = json_lines(&stdout);
    assert_eq!(reference.len(), 3, "{reference:?}");

    let body = case(SHAPE_ERROR);
    let av = section(body, "Probe av");
    let a2py = section(body, "Probe a2py");
    let av_lines: Vec<&str> = stream(av, "stdout").unwrap().1.lines().collect();
    let a2py_lines: Vec<&str> = stream(a2py, "stdout").unwrap().1.lines().collect();
    assert_eq!(av_lines.len(), 3, "{av_lines:?}");
    let tail: [&str; 1] = ["value:0"];
    assert_eq!(a2py_lines, [av_lines.as_slice(), tail.as_slice()].concat());

    // The class Auto reports IS the class NumPy raised, with NumPy's message.
    let native = reference[0]
        .as_str()
        .expect("the reference reports the exception text");
    let native_class = native
        .split_once(": ")
        .map(|(class, _)| class)
        .expect("the reference names the exception class");
    assert_eq!(av_lines[0], format!("caught: PyException {native}"));
    assert_eq!(attr(a2py, "error-type"), Some(native_class));
    for probe in [av, a2py] {
        assert_eq!(attr(probe, "error-kind"), Some("python.5"));
        assert_eq!(attr(probe, "error-span"), Some("76..99"));
    }

    // The handler ran exactly once and execution continued.
    assert_eq!(reference[1], Value::from(1), "caught exactly once");
    agree(Int, &reference[1], av_lines[1]).unwrap();
    agree(Int, &reference[2], av_lines[2]).unwrap();
}

#[test]
fn p1126_v03_native_reference_confirms_the_json_decode_error() {
    let (code, stdout, stderr) = run_reference("json_decode_error_ref.py");
    assert_eq!(code, Some(1), "the uncaught reference must exit non-zero");
    assert!(stdout.is_empty(), "{}", String::from_utf8_lossy(&stdout));
    let native = last_line(&stderr);
    let (qualified, native_message) = native
        .split_once(": ")
        .unwrap_or_else(|| panic!("traceback line `{native}`"));
    let native_class = qualified.rsplit('.').next().unwrap();
    assert_eq!(native_class, "JSONDecodeError", "traceback line `{native}`");

    let body = case(DECODE_ERROR);
    let av = section(body, "Probe av");
    let a2py = section(body, "Probe a2py");
    // Both lanes report the original class and the interpreter's own message.
    let auto_text = format!("PyException {native_class}: {native_message}");
    assert_eq!(attr(av, "message"), Some(auto_text.as_str()));
    assert_eq!(stream(a2py, "stderr").unwrap().1, format!("{auto_text}\n"));
    // `error-type` is the A2PY oracle; AV carries the class in its message.
    assert!(attr(av, "error-type").is_none());
    assert_eq!(attr(a2py, "error-type"), Some(native_class));
    for probe in [av, a2py] {
        assert_eq!(attr(probe, "error-kind"), Some("python.10"));
        assert_eq!(attr(probe, "error-span"), Some("50..60"));
    }
    assert_eq!(attr(av, "code"), Some("lang.error"));
    assert_eq!(attr(av, "stage"), Some("run"));
    assert!(
        stream(av, "stdout").unwrap().1.is_empty(),
        "no partial output"
    );
    assert!(
        stream(a2py, "stdout").unwrap().1.is_empty(),
        "no success value"
    );
}

// ---- V-04: source identity and oracle sensitivity -------------------------

#[test]
fn p1126_v04_sources_are_pinned_and_spans_match_the_authored_bytes() {
    for (name, pinned, from, to) in [
        (POSITIVE, POSITIVE_SOURCE_SHA256, "arange(6)", "arange(5)"),
        (
            SHAPE_ERROR,
            SHAPE_ERROR_SOURCE_SHA256,
            "arange(6)",
            "arange(5)",
        ),
        (
            DECODE_ERROR,
            DECODE_ERROR_SOURCE_SHA256,
            "loads(\"{\")",
            "loads(\"{}\")",
        ),
    ] {
        let bytes = source(name).as_bytes();
        assert_eq!(sha256_hex(bytes), pinned, "case `{name}` source drifted");
        // The fingerprint is sensitive to a one-token change of the source.
        let mutated = source(name).replacen(from, to, 1);
        assert_ne!(mutated, source(name), "the `{from}` mutation must apply");
        assert_ne!(sha256_hex(mutated.as_bytes()), pinned, "case `{name}`");
    }
    // The failing-operation ranges are byte ranges in the authored source.
    for (name, probe, needle, span) in [
        (SHAPE_ERROR, "Probe av", "arange(6).reshape(4, 2)", "76..99"),
        (
            SHAPE_ERROR,
            "Probe a2py",
            "arange(6).reshape(4, 2)",
            "76..99",
        ),
        (DECODE_ERROR, "Probe av", "loads(\"{\")", "50..60"),
        (DECODE_ERROR, "Probe a2py", "loads(\"{\")", "50..60"),
    ] {
        let bytes = source(name).as_bytes();
        let mut hits = bytes
            .windows(needle.len())
            .enumerate()
            .filter(|(_, w)| *w == needle.as_bytes())
            .map(|(i, _)| i);
        let start = hits
            .next()
            .unwrap_or_else(|| panic!("`{needle}` is missing from `{name}`"));
        assert!(
            hits.next().is_none(),
            "`{needle}` occurs exactly once in `{name}`"
        );
        let computed = format!("{start}..{}", start + needle.len());
        assert_eq!(
            computed, span,
            "the pinned range is the authored byte range of `{needle}`"
        );
        assert_eq!(
            attr(section(case(name), probe), "error-span"),
            Some(span),
            "`{name}` / `{probe}`"
        );
    }
}

#[test]
fn p1126_v04_the_oracle_rejects_wrong_values() {
    // A legitimate reference: the typed values NumPy and `json` produce.
    let reference: Vec<Value> = serde_json::from_str(
        r#"[2, 3, 7, 21, 10, 3.5, 5, true, "雪",
            "{\"label\": \"雪\", \"meta\": {\"enabled\": true}, \"values\": [2, 3, 5]}",
            "{\"label\": \"雪\", \"meta\": {\"enabled\": true}, \"values\": [2, 3, 5]}"]"#,
    )
    .unwrap();
    // The frozen positive lines both lanes are judged against.
    let frozen = "2\n3\n7\n21\n10\n3.5\n5\ntrue\n雪\n\
        {\"label\": \"雪\", \"meta\": {\"enabled\": true}, \"values\": [2, 3, 5]}\n\
        {\"label\": \"雪\", \"meta\": {\"enabled\": true}, \"values\": [2, 3, 5]}\n";
    assert_eq!(reference.len(), POSITIVE_SHAPE.len());
    positive_agrees(&reference, frozen).expect("the legitimate pair agrees");

    // The tolerance is the plan's, not wider: 1e-7 passes, 1e-5 does not.
    let near = frozen.replacen("3.5\n", "3.5000001\n", 1);
    positive_agrees(&reference, &near).expect("within rtol=atol=1e-6");

    // One wrong line at a time; each must disagree.
    let right: Vec<String> = frozen.lines().map(String::from).collect();
    for (index, wrong) in [
        (3usize, "20"), // a wrong total
        (4, "15"),      // the whole row instead of the strided slice
        (4, "8"),       // a dropped element
        (5, "3.50001"), // mean outside the tolerance
        (6, "4"),       // a wrong json leaf
        (7, "false"),   // a wrong json leaf
        (8, "标签"),    // a wrong json field text
    ] {
        let mut lines = right.clone();
        lines[index] = wrong.to_string();
        let joined = lines.join("\n");
        assert!(
            positive_agrees(&reference, &format!("{joined}\n")).is_err(),
            "a wrong line {index} = {wrong:?} must NOT agree"
        );
    }
    // A wrong json field NAME inside the encoded text.
    assert!(positive_agrees(&reference, &frozen.replace("\"label\"", "\"labels\"")).is_err());
    // A missing line and an extra line both disagree.
    assert!(positive_agrees(&reference, &frozen.replacen("2\n3\n", "2\n", 1)).is_err());
    assert!(positive_agrees(&reference, &format!("{frozen}7\n")).is_err());
    // A reference of the wrong type never agrees with a right-looking line.
    assert!(agree(Int, &Value::from(3.5), "3").is_err());
    assert!(agree(Bool, &Value::from(1), "true").is_err());
    assert!(agree(Text, &Value::from(7), "7").is_err());
    assert!(agree(Int, &Value::from(3), "3.0").is_err());
}
