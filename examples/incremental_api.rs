// =============================================================================
// Incremental Compilation API Example (Plan 065)
// =============================================================================
//
// This example demonstrates how to use the incremental compilation API
// programmatically in Rust code.
//
// Run with: cargo run --example incremental_api
//
// =============================================================================

use auto_lang::{CompileSession, run_with_session};

fn main() -> auto_lang::AutoResult<()> {
    println!("=== AutoLang Incremental Compilation Example ===\n");

    // =========================================================================
    // Step 1: Create a persistent compilation session
    // =========================================================================
    // The CompileSession manages the Database (compile-time data)
    // and persists across multiple compilations.
    let mut session = CompileSession::new();

    println!("Step 1: Created CompileSession");

    // =========================================================================
    // Step 2: Compile first function
    // =========================================================================
    // The function is compiled and stored in the Database.
    let code1 = r#"
        fn add(a int, b int) int {
            a + b
        }
    "#;

    let result1 = run_with_session(&mut session, code1)?;
    println!("Step 2: Compiled 'add' function");
    println!("Result: {}\n", result1);

    // =========================================================================
    // Step 3: Call the function (second input)
    // =========================================================================
    // The Database still contains the `add` function from Step 2,
    // so we can call it without recompiling.
    let code2 = r#"
        add(1, 2)
    "#;

    let result2 = run_with_session(&mut session, code2)?;
    println!("Step 3: Called 'add' function");
    println!("Result: {}\n", result2);

    // =========================================================================
    // Step 4: Define a function that uses the first function
    // =========================================================================
    // The `add` function is already in the Database, so we can
    // reference it in new code without recompiling it.
    let code3 = r#"
        fn calculate(x int) int {
            add(x, 10) * 2
        }
    "#;

    let result3 = run_with_session(&mut session, code3)?;
    println!("Step 4: Compiled 'calculate' function (uses 'add')");
    println!("Result: {}\n", result3);

    // =========================================================================
    // Step 5: Call the new function
    // =========================================================================
    let code4 = r#"
        calculate(5)
    "#;

    let result4 = run_with_session(&mut session, code4)?;
    println!("Step 5: Called 'calculate' function");
    println!("Result: {}\n", result4);

    // =========================================================================
    // Step 6: Get compilation statistics
    // =========================================================================
    // The session tracks what's been compiled.
    let stats = session.stats();
    println!("Step 6: Compilation Statistics");
    println!("  Total Files: {}", stats.total_files);
    println!("  Total Fragments: {}", stats.total_frags);
    println!("  Total Functions: {}", stats.total_functions);
    println!("  Total Specs: {}\n", stats.total_specs);

    // =========================================================================
    // Step 7: Modify a function (incremental update)
    // =========================================================================
    // When we redefine `add`, only that function is recompiled.
    // The `calculate` function is NOT recompiled (it references `add` by name).
    let code5 = r#"
        fn add(a int, b int) int {
            a + b + 100  # Changed: now adds 100
        }
    "#;

    let result5 = run_with_session(&mut session, code5)?;
    println!("Step 7: Redefined 'add' function (incremental update)");
    println!("Result: {}\n", result5);

    // =========================================================================
    // Step 8: Call both functions (see the updated behavior)
    // =========================================================================
    let code6 = r#"
        add(1, 2)  # Now returns 103 (1 + 2 + 100)
    "#;

    let result6 = run_with_session(&mut session, code6)?;
    println!("Step 8: Called updated 'add' function");
    println!("Result: {}\n", result6);

    let code7 = r#"
        calculate(5)  # Now uses the new add function!
    "#;

    let result7 = run_with_session(&mut session, code7)?;
    println!("Step 8: Called 'calculate' (uses updated 'add')");
    println!("Result: {}\n", result7);

    // =========================================================================
    // Step 9: Final statistics
    // =========================================================================
    let stats_final = session.stats();
    println!("Step 9: Final Compilation Statistics");
    println!("  Total Files: {}", stats_final.total_files);
    println!("  Total Fragments: {}", stats_final.total_frags);
    println!("  Total Functions: {}", stats_final.total_functions);
    println!("  Total Specs: {}", stats_final.total_specs);

    println!("\n=== Benefits of Incremental Compilation ===");
    println!("✓ Functions persist in the Database across inputs");
    println!("✓ Only modified code is recompiled");
    println!("✓ Fast iterative development");
    println!("✓ Clean separation: compile-time (Database) vs runtime (ExecutionEngine)");

    Ok(())
}

// =============================================================================
// Output Example:
// =============================================================================
//
// Step 1: Created CompileSession
// Step 2: Compiled 'add' function
// Result: ()
//
// Step 3: Called 'add' function
// Result: 3
//
// Step 4: Compiled 'calculate' function (uses 'add')
// Result: ()
//
// Step 5: Called 'calculate' function
// Result: 30
//
// Step 6: Compilation Statistics
//   Total Files: 1
//   Total Fragments: 2
//   Total Functions: 2
//   Total Specs: 0
//
// Step 7: Redefined 'add' function (incremental update)
// Result: ()
//
// Step 8: Called updated 'add' function
// Result: 103
//
// Step 8: Called 'calculate' (uses updated 'add')
// Result: 230
//
// Step 9: Final Compilation Statistics
//   Total Files: 1
//   Total Fragments: 2
//   Total Functions: 2
//   Total Specs: 0
//
// === Benefits of Incremental Compilation ===
// ✓ Functions persist in the Database across inputs
// ✓ Only modified code is recompiled
// ✓ Fast iterative development
// ✓ Clean separation: compile-time (Database) vs runtime (ExecutionEngine)
//
// =============================================================================
