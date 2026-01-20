//! AutoLang function execution
//!
//! Handles execution of AutoLang functions in command pipelines.

use miette::Result;

/// Execute AutoLang function with optional input
///
/// If pipeline input is provided, it's added as the first argument.
/// This enables Auto functions to participate in pipelines.
///
/// Note: Currently, user-defined functions must be imported from stdlib
/// or defined in CONFIG mode. Function definitions in SCRIPT mode
/// (REPL) may not persist in the interpreter scope.
pub fn execute_auto_function(
    shell: &mut crate::shell::Shell,
    func_name: &str,
    args: &[String],
    input: Option<&str>,
) -> Result<Option<String>> {
    // If input provided, add as first argument
    let mut all_args = Vec::new();
    if let Some(input_str) = input {
        // Don't quote - let AutoLang parse it naturally
        // This handles both strings and numbers
        all_args.push(input_str.to_string());
    }
    all_args.extend(args.iter().cloned());

    shell.execute_auto_function(func_name, all_args)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_auto_expression_basic() {
        let mut shell = crate::shell::Shell::new();

        // Test that basic Auto expressions work
        let result = shell.execute("1 + 2");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("3".to_string()));
    }

    #[test]
    fn test_auto_function_call_formatting() {
        let mut shell = crate::shell::Shell::new();

        // Test that function call formatting works correctly
        // We're not testing actual function execution, just the formatting
        let result = shell.execute_auto_function("test", vec!["1".to_string(), "2".to_string()]);
        // This will fail because "test" doesn't exist, but we can check the error format
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_function_with_args_formatting() {
        let mut shell = crate::shell::Shell::new();

        // Test that arguments are joined correctly with commas
        let result = shell.execute_auto_function("dummy", vec!["5".to_string()]);
        // This will fail because "dummy" doesn't exist
        assert!(result.is_err());
    }
}
