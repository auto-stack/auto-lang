//! Quick test to check if alloc_array is registered

use crate::run;

#[test]
fn test_alloc_array_exists() {
    let code = r#"
        alloc_array
    "#;

    let result = run(code);
    eprintln!("alloc_array alone result: {:?}", result);

    // Try calling it
    let code_call = r#"
        alloc_array(5)
    "#;

    let result_call = run(code_call);
    eprintln!("alloc_array(5) result: {:?}", result_call);
}
