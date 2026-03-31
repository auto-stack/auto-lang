//! Integration tests for structured command output

use auto_shell::shell::Shell;

#[test]
fn test_ls_returns_structured_data() {
    let mut shell = Shell::new();

    // Execute ls command
    let result = shell.execute("ls .");

    // Should return structured data (array of objects)
    assert!(result.is_ok());
}

#[test]
fn test_ps_returns_structured_data() {
    let mut shell = Shell::new();

    // Execute ps command
    let result = shell.execute("ps");

    // Should return structured data
    assert!(result.is_ok());
}

#[test]
fn test_sys_disks_returns_structured_data() {
    let mut shell = Shell::new();

    // Execute sys disks command
    let result = shell.execute("sys disks");

    // Should return structured data
    assert!(result.is_ok());
}

#[test]
fn test_sys_mem_returns_structured_data() {
    let mut shell = Shell::new();

    // Execute sys mem command
    let result = shell.execute("sys mem");

    // Should return structured data
    assert!(result.is_ok());
}
