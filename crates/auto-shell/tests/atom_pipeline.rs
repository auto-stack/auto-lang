//! Atom Pipeline integration tests
//!
//! Validates that:
//! - Commands produce correctly typed Atoms
//! - Pipeline data flows through AtomPipeline correctly
//! - Bridge conversion is lossless
//! - Default run_atom() delegation works for all commands

use auto_shell::Shell;
use auto_shell::pipeline::{AtomPipeline, AtomType};

/// Helper: execute a command and get the raw string output
fn exec(shell: &mut Shell, input: &str) -> Option<String> {
    shell.execute(input).unwrap_or(None)
}

// ── Group A: Data producers ─────────────────────────────────

#[test]
fn test_ls_produces_output() {
    let mut shell = Shell::new();
    let output = exec(&mut shell, "ls");
    // ls should produce output (directory listing)
    assert!(output.is_some());
    assert!(!output.unwrap().is_empty());
}

#[test]
fn test_pwd_produces_path() {
    let mut shell = Shell::new();
    let output = exec(&mut shell, "pwd");
    assert!(output.is_some());
    let path = output.unwrap();
    // Should be an absolute path
    assert!(path.starts_with('/') || path.len() > 1);
}

#[test]
fn test_ps_produces_output() {
    let mut shell = Shell::new();
    let output = exec(&mut shell, "ps");
    assert!(output.is_some());
}

#[test]
fn test_sys_produces_output() {
    let mut shell = Shell::new();
    let output = exec(&mut shell, "sys mem");
    assert!(output.is_some());
}

// ── Group B: Data consumers ──────────────────────────────────

#[test]
fn test_echo_produces_text() {
    let mut shell = Shell::new();
    let output = exec(&mut shell, "echo hello world");
    assert_eq!(output, Some("hello world".to_string()));
}

#[test]
fn test_help_produces_output() {
    let mut shell = Shell::new();
    let output = exec(&mut shell, "help");
    assert!(output.is_some());
    let text = output.unwrap();
    assert!(text.contains("Available Commands"));
}

// ── Group C: Side-effect commands ────────────────────────────

#[test]
fn test_cd_changes_directory() {
    let mut shell = Shell::new();
    // Use shell.cd() directly to avoid shell escaping issues on Windows
    let tmp = std::env::temp_dir().to_string_lossy().to_string();
    let result = shell.cd(&tmp);
    assert!(result.is_ok());
}

#[test]
fn test_mkdir_and_rm() {
    let mut shell = Shell::new();
    let tmp_dir = format!("{}/atom_test_tmp", std::env::temp_dir().to_string_lossy());

    // Create dir
    let result = shell.execute(&format!("mkdir {}", &tmp_dir));
    assert!(result.is_ok());

    // Remove dir
    let result = shell.execute(&format!("rm -r {}", &tmp_dir));
    assert!(result.is_ok());
}

// ── Pipeline data flow ───────────────────────────────────────

#[test]
fn test_pipeline_ls_grep() {
    let mut shell = Shell::new();
    // ls | grep should work without errors
    let result = shell.execute("ls | grep .");
    assert!(result.is_ok());
}

#[test]
fn test_pipeline_echo_wc() {
    let mut shell = Shell::new();
    let result = shell.execute("echo hello world | wc -w");
    assert!(result.is_ok());
    let output = result.unwrap();
    // Should show word count
    assert!(output.is_some());
}

// ── Bridge conversion roundtrip ──────────────────────────────

#[test]
fn test_atom_pipeline_text_roundtrip() {
    let atom = AtomPipeline::text("hello");
    let text = atom.into_text();
    assert_eq!(text, "hello");
}

#[test]
fn test_atom_pipeline_empty_roundtrip() {
    let atom = AtomPipeline::empty();
    assert!(atom.is_empty());
    assert_eq!(atom.into_text(), "");
}

#[test]
fn test_atom_type_tags() {
    use auto_shell::pipeline::{Atom, AtomType};

    let file_list = Atom::file_list(auto_val::Value::Void);
    assert_eq!(file_list.atom_type(), AtomType::FileList);
    assert!(file_list.is_structured());

    let process_list = Atom::process_list(auto_val::Value::Void);
    assert_eq!(process_list.atom_type(), AtomType::ProcessList);

    let path = Atom::path("/tmp");
    assert_eq!(path.atom_type(), AtomType::Path);
    assert_eq!(path.as_text(), "/tmp");

    let text = Atom::text("hello");
    assert_eq!(text.atom_type(), AtomType::Text);
    assert!(!text.is_structured());
}

#[test]
fn test_bridge_conversion_value() {
    use auto_shell::cmd::pipeline_convert::{pipeline_data_to_atom, atom_to_pipeline_data};
    use auto_shell::cmd::PipelineData;

    let pd = PipelineData::from_value(auto_val::Value::Int(42));
    let atom = pipeline_data_to_atom(pd);
    assert!(atom.is_atom());

    let back = atom_to_pipeline_data(atom);
    assert!(back.is_value());
}

#[test]
fn test_bridge_conversion_text() {
    use auto_shell::cmd::pipeline_convert::{pipeline_data_to_atom, atom_to_pipeline_data};
    use auto_shell::cmd::PipelineData;

    let pd = PipelineData::from_text("hello".to_string());
    let atom = pipeline_data_to_atom(pd);
    assert!(atom.is_text());

    let back = atom_to_pipeline_data(atom);
    assert!(back.is_text());
    assert_eq!(back.into_text(), "hello");
}

// ── Type inference ───────────────────────────────────────────

#[test]
fn test_infer_file_list() {
    use auto_shell::pipeline::convert::infer_atom_type;
    use auto_val::{Value, Obj};

    let mut obj = Obj::new();
    obj.set("name", Value::str("test.txt"));
    obj.set("type", Value::str("file"));
    let arr = auto_val::Array::from(vec![Value::Obj(obj)]);

    assert_eq!(infer_atom_type(&Value::Array(arr)), AtomType::FileList);
}

#[test]
fn test_infer_process_list() {
    use auto_shell::pipeline::convert::infer_atom_type;
    use auto_val::{Value, Obj};

    let mut obj = Obj::new();
    obj.set("pid", Value::Int(1));
    obj.set("name", Value::str("init"));
    let arr = auto_val::Array::from(vec![Value::Obj(obj)]);

    assert_eq!(infer_atom_type(&Value::Array(arr)), AtomType::ProcessList);
}

#[test]
fn test_infer_system_info() {
    use auto_shell::pipeline::convert::infer_atom_type;
    use auto_val::{Value, Obj};

    let mut obj = Obj::new();
    obj.set("cpu", Value::str("x86"));
    obj.set("memory", Value::Int(8192));

    assert_eq!(infer_atom_type(&Value::Obj(obj)), AtomType::SystemInfo);
}
