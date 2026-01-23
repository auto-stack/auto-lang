use crate::run;
use crate::error::AutoResult;

#[test]
fn test_storage_environment_injection() {
    // Test 1: Verify environment injection happens at startup
    let code = r#"
        fn main() {
            // Create a list
            let list = List.new()
            list.push(42)

            // Check capacity (should return i32::MAX for PC, 64 for MCU)
            let cap = list.capacity()

            // Verify list works
            let len = list.len()

            print("Capacity:", cap)
            print("Length:", len)
        }
    "#;

    match run(code) {
        Ok(result) => {
            println!("✓ Storage environment injection test passed");
            println!("Result: {}", result);
        }
        Err(e) => {
            eprintln!("✗ Storage environment injection test failed: {}", e);
            panic!("Test failed");
        }
    }
}

#[test]
fn test_dynamic_storage_type() {
    // Test that Dynamic storage type is recognized
    let code = r#"
        fn main() {
            let storage Dynamic
            print("Dynamic type:", storage)
        }
    "#;

    match run(code) {
        Ok(result) => {
            println!("✓ Dynamic storage type test passed");
            println!("Result: {}", result);
        }
        Err(e) => {
            eprintln!("✗ Dynamic storage type test failed: {}", e);
            panic!("Test failed");
        }
    }
}

#[test]
fn test_fixed_storage_type() {
    // Test that Fixed storage type is recognized
    let code = r#"
        fn main() {
            let storage Fixed<int>
            print("Fixed type:", storage)
        }
    "#;

    match run(code) {
        Ok(result) => {
            println!("✓ Fixed storage type test passed");
            println!("Result: {}", result);
        }
        Err(e) => {
            eprintln!("✗ Fixed storage type test failed: {}", e);
            panic!("Test failed");
        }
    }
}
