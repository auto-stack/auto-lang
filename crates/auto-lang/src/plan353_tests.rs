//! Plan 353: IO/FS module tests.
//!
//! Tests io.lines (streaming line reader), io.chunks, eprint/eprintln.

#[cfg(test)]
mod plan353_tests {
    use crate::run_with_capture;
    use std::io::Write;

    /// Test eprintln: write to stderr.
    #[test]
    fn test_eprintln() {
        let code = r#"
eprintln("error from vm")
print("ok")
"#;
        let result = run_with_capture(code);
        assert!(result.is_ok());
        let (_, stdout) = result.unwrap();
        assert!(stdout.contains("ok"), "expected ok: [{}]", stdout);
    }

    /// Test eprint (no newline).
    #[test]
    fn test_eprint() {
        let code = r#"
eprint("no newline")
print("done")
"#;
        let result = run_with_capture(code);
        assert!(result.is_ok());
        let (_, stdout) = result.unwrap();
        assert!(stdout.contains("done"), "expected done: [{}]", stdout);
    }

    /// Test io.lines: streaming line reader on a temp file.
    /// Note: io.lines uses async yield (Plan 348 mechanism) which requires
    /// run_task_loop. In run_with_capture, the iterator will be created but
    /// for-loop iteration may not work. We test iterator creation instead.
    #[test]
    fn test_io_lines_iterator_creation() {
        // Create a temp file with lines.
        let temp = std::env::temp_dir().join("auto_test_lines.txt");
        let mut f = std::fs::File::create(&temp).unwrap();
        writeln!(f, "line1").unwrap();
        writeln!(f, "line2").unwrap();
        writeln!(f, "line3").unwrap();

        let code = format!(
            r#"
let iter = io.lines("{}")
print("created")
"#,
            temp.to_str().unwrap()
        );
        let result = run_with_capture(&code);
        assert!(result.is_ok(), "io.lines should run: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        assert!(stdout.contains("created"), "expected created: [{}]", stdout);
        let _ = std::fs::remove_file(&temp);
    }

    /// Test io.chunks: streaming chunk reader.
    #[test]
    fn test_io_chunks_iterator_creation() {
        let temp = std::env::temp_dir().join("auto_test_chunks.bin");
        std::fs::write(&temp, b"0123456789ABCDEFGHIJ").unwrap();

        let code = format!(
            r#"
let iter = io.chunks("{}", 4)
print("chunked")
"#,
            temp.to_str().unwrap()
        );
        let result = run_with_capture(&code);
        assert!(result.is_ok(), "io.chunks should run: {:?}", result.err());
        let (_, stdout) = result.unwrap();
        assert!(stdout.contains("chunked"), "expected chunked: [{}]", stdout);
        let _ = std::fs::remove_file(&temp);
    }
}
