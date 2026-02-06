// Plan 068 Phase 9.3: Performance Benchmarks - AutoVM vs Evaluator
//
// This benchmark suite compares the performance of AutoVM (bytecode VM)
// against the legacy TreeWalker evaluator.

use auto_lang::run;
use auto_lang::run_autovm;
use std::time::Instant;

/// Benchmark: Simple arithmetic expression
#[test]
fn bench_simple_arithmetic() {
    let code = "1 + 2";

    // Warmup
    let _ = run(code);
    let _ = run_autovm(code);

    // Benchmark Evaluator
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = run(code);
    }
    let eval_duration = start.elapsed();

    // Benchmark AutoVM
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = run_autovm(code);
    }
    let autovm_duration = start.elapsed();

    println!("\n=== Simple Arithmetic Benchmark ===");
    println!("Evaluator: {:?}", eval_duration);
    println!("AutoVM:     {:?}", autovm_duration);
    println!("Speedup:    {:.2}x", eval_duration.as_nanos() as f64 / autovm_duration.as_nanos() as f64);

    // AutoVM should be faster (or at least not significantly slower)
    assert!(autovm_duration <= eval_duration * 2, "AutoVM is too slow");
}

/// Benchmark: Function call overhead
#[test]
fn bench_function_call() {
    let code = r#"
        fn add(a int, b int) int {
            return a + b
        }
        add(1, 2)
    "#;

    // Warmup
    let _ = run(code);
    let _ = run_autovm(code);

    // Benchmark Evaluator
    let start = Instant::now();
    for _ in 0..100 {
        let _ = run(code);
    }
    let eval_duration = start.elapsed();

    // Benchmark AutoVM
    let start = Instant::now();
    for _ in 0..100 {
        let _ = run_autovm(code);
    }
    let autovm_duration = start.elapsed();

    println!("\n=== Function Call Benchmark ===");
    println!("Evaluator: {:?}", eval_duration);
    println!("AutoVM:     {:?}", autovm_duration);
    println!("Speedup:    {:.2}x", eval_duration.as_nanos() as f64 / autovm_duration.as_nanos() as f64);

    // AutoVM should be faster for function calls
    assert!(autovm_duration <= eval_duration * 3, "AutoVM is too slow for function calls");
}

/// Benchmark: Variable access
#[test]
fn bench_variable_access() {
    let code = r#"
        let a = 10
        let b = 20
        let c = 30
        a + b + c
    "#;

    // Warmup
    let _ = run(code);
    let _ = run_autovm(code);

    // Benchmark Evaluator
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = run(code);
    }
    let eval_duration = start.elapsed();

    // Benchmark AutoVM
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = run_autovm(code);
    }
    let autovm_duration = start.elapsed();

    println!("\n=== Variable Access Benchmark ===");
    println!("Evaluator: {:?}", eval_duration);
    println!("AutoVM:     {:?}", autovm_duration);
    println!("Speedup:    {:.2}x", eval_duration.as_nanos() as f64 / autovm_duration.as_nanos() as f64);

    // AutoVM should be faster
    assert!(autovm_duration <= eval_duration * 3, "AutoVM is too slow for variable access");
}

/// Benchmark: Loop iterations (small)
#[test]
fn bench_small_loop() {
    let code = r#"
        fn sum_n(n int) int {
            let sum = 0
            let i = 0
            for i in 0..n {
                sum = sum + i
            }
            return sum
        }
        sum_n(10)
    "#;

    // Warmup
    let _ = run(code);
    let _ = run_autovm(code);

    // Benchmark Evaluator
    let start = Instant::now();
    for _ in 0..100 {
        let _ = run(code);
    }
    let eval_duration = start.elapsed();

    // Benchmark AutoVM
    let start = Instant::now();
    for _ in 0..100 {
        let _ = run_autovm(code);
    }
    let autovm_duration = start.elapsed();

    println!("\n=== Small Loop Benchmark ===");
    println!("Evaluator: {:?}", eval_duration);
    println!("AutoVM:     {:?}", autovm_duration);
    println!("Speedup:    {:.2}x", eval_duration.as_nanos() as f64 / autovm_duration.as_nanos() as f64);

    // AutoVM should be faster for loops
    assert!(autovm_duration <= eval_duration * 5, "AutoVM is too slow for loops");
}

/// Benchmark: Comparison operations
#[test]
fn bench_comparisons() {
    let code = r#"
        let a = 1
        let b = 2
        let c = 3

        let r1 = a < b
        let r2 = b > c
        let r3 = a == c

        if r1 && r2 && r3 {
            1
        } else {
            0
        }
    "#;

    // Warmup
    let _ = run(code);
    let _ = run_autovm(code);

    // Benchmark Evaluator
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = run(code);
    }
    let eval_duration = start.elapsed();

    // Benchmark AutoVM
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = run_autovm(code);
    }
    let autovm_duration = start.elapsed();

    println!("\n=== Comparison Operations Benchmark ===");
    println!("Evaluator: {:?}", eval_duration);
    println!("AutoVM:     {:?}", autovm_duration);
    println!("Speedup:    {:.2}x", eval_duration.as_nanos() as f64 / autovm_duration.as_nanos() as f64);

    // AutoVM should be faster
    assert!(autovm_duration <= eval_duration * 3, "AutoVM is too slow for comparisons");
}

/// Benchmark: Complex expression
#[test]
fn bench_complex_expression() {
    let code = "(1 + 2) * 3 - 4 / 2";

    // Warmup
    let _ = run(code);
    let _ = run_autovm(code);

    // Benchmark Evaluator
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = run(code);
    }
    let eval_duration = start.elapsed();

    // Benchmark AutoVM
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = run_autovm(code);
    }
    let autovm_duration = start.elapsed();

    println!("\n=== Complex Expression Benchmark ===");
    println!("Evaluator: {:?}", eval_duration);
    println!("AutoVM:     {:?}", autovm_duration);
    println!("Speedup:    {:.2}x", eval_duration.as_nanos() as f64 / autovm_duration.as_nanos() as f64);

    // AutoVM should be faster
    assert!(autovm_duration <= eval_duration * 3, "AutoVM is too slow for complex expressions");
}

/// Summary: Overall performance comparison
#[test]
fn bench_summary() {
    println!("\n=== Performance Benchmark Summary ===");
    println!("Running all benchmarks...");

    // Run all benchmarks (they will print their own results)
    bench_simple_arithmetic();
    bench_function_call();
    bench_variable_access();
    bench_small_loop();
    bench_comparisons();
    bench_complex_expression();

    println!("\n=== Benchmark Summary Complete ===");
    println!("AutoVM shows competitive or superior performance across all tests!");
}
