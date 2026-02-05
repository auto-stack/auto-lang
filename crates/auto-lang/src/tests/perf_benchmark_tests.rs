// Plan 073 Phase 9.1: Performance Benchmarking
// Compares BigVM vs Evaluator performance

use crate::run;
use std::time::Instant;

/// Benchmark result for a single test case
#[derive(Debug)]
struct BenchmarkResult {
    name: String,
    evaluator_time_us: u128,
    bigvm_time_us: u128,
    speedup: f64,
    evaluator_result: String,
    bigvm_result: String,
}

/// Run a benchmark comparing evaluator vs BigVM
fn run_benchmark(name: &str, source: &str) -> BenchmarkResult {
    // Benchmark Evaluator
    let eval_start = Instant::now();
    let eval_result = run(source).unwrap_or_else(|e| format!("ERROR: {}", e));
    let eval_duration = eval_start.elapsed().as_micros();

    // Benchmark BigVM (using compile mode script)
    let vm_start = Instant::now();
    let vm_result = crate::run_with_mode(source, crate::CompileMode::Script)
        .unwrap_or_else(|e| format!("ERROR: {}", e));
    let vm_duration = vm_start.elapsed().as_micros();

    // Calculate speedup
    let speedup = if vm_duration > 0 {
        eval_duration as f64 / vm_duration as f64
    } else {
        0.0
    };

    BenchmarkResult {
        name: name.to_string(),
        evaluator_time_us: eval_duration,
        bigvm_time_us: vm_duration,
        speedup,
        evaluator_result: eval_result,
        bigvm_result: vm_result,
    }
}

#[test]
fn benchmark_arithmetic_operations() {
    let source = r#"
fn main() -> int {
    let mut sum = 0
    for i in 0..1000 {
        sum = sum + i
    }
    sum
}
"#;

    let result = run_benchmark("arithmetic_loop_1000", source);

    println!("\n=== Benchmark: {} ===", result.name);
    println!("Evaluator:  {} μs", result.evaluator_time_us);
    println!("BigVM:      {} μs", result.bigvm_time_us);
    println!("Speedup:    {:.2}x", result.speedup);

    // BigVM should be at least as fast or faster
    // (This is a soft assertion - actual performance may vary)
    if result.speedup < 0.5 {
        println!("WARNING: BigVM is significantly slower than evaluator");
    }
}

#[test]
fn benchmark_function_calls() {
    let source = r#"
fn add(a int, b int) int {
    a + b
}

fn main() -> int {
    let mut sum = 0
    for i in 0..100 {
        sum = add(sum, i)
    }
    sum
}
"#;

    let result = run_benchmark("function_calls_100", source);

    println!("\n=== Benchmark: {} ===", result.name);
    println!("Evaluator:  {} μs", result.evaluator_time_us);
    println!("BigVM:      {} μs", result.bigvm_time_us);
    println!("Speedup:    {:.2}x", result.speedup);
}

#[test]
fn benchmark_recursion() {
    let source = r#"
fn factorial(n int) int {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fn main() -> int {
    factorial(8)
}
"#;

    let result = run_benchmark("recursion_factorial_10", source);

    println!("\n=== Benchmark: {} ===", result.name);
    println!("Evaluator:  {} μs", result.evaluator_time_us);
    println!("BigVM:      {} μs", result.bigvm_time_us);
    println!("Speedup:    {:.2}x", result.speedup);
}

#[test]
fn benchmark_list_operations() {
    let source = r#"
fn main() -> int {
    let list = List.new()
    for i in 0..100 {
        list.push(i)
    }

    let mut sum = 0
    for i in 0..100 {
        let val = list.get(i)
        sum = sum + val
    }
    sum
}
"#;

    let result = run_benchmark("list_operations_100", source);

    println!("\n=== Benchmark: {} ===", result.name);
    println!("Evaluator:  {} μs", result.evaluator_time_us);
    println!("BigVM:      {} μs", result.bigvm_time_us);
    println!("Speedup:    {:.2}x", result.speedup);
}

#[test]
fn benchmark_nested_loops() {
    let source = r#"
fn main() -> int {
    let mut count = 0
    for i in 0..10 {
        for j in 0..10 {
            for k in 0..10 {
                count = count + 1
            }
        }
    }
    count
}
"#;

    let result = run_benchmark("nested_loops_10x10x10", source);

    println!("\n=== Benchmark: {} ===", result.name);
    println!("Evaluator:  {} μs", result.evaluator_time_us);
    println!("BigVM:      {} μs", result.bigvm_time_us);
    println!("Speedup:    {:.2}x", result.speedup);
}

#[test]
fn benchmark_string_operations() {
    let source = r#"
fn main() -> int {
    let s1 = "hello"
    let s2 = "world"
    let s3 = f"{s1} {s2}"
    s3.len()
}
"#;

    let result = run_benchmark("string_operations", source);

    println!("\n=== Benchmark: {} ===", result.name);
    println!("Evaluator:  {} μs", result.evaluator_time_us);
    println!("BigVM:      {} μs", result.bigvm_time_us);
    println!("Speedup:    {:.2}x", result.speedup);
}

#[test]
fn benchmark_object_creation() {
    let source = r#"
type Point {
    x int
    y int
}

fn main() -> int {
    let mut sum = 0
    for i in 0..100 {
        let p = Point(i, i * 2)
        sum = sum + p.x + p.y
    }
    sum
}
"#;

    let result = run_benchmark("object_creation_100", source);

    println!("\n=== Benchmark: {} ===", result.name);
    println!("Evaluator:  {} μs", result.evaluator_time_us);
    println!("BigVM:      {} μs", result.bigvm_time_us);
    println!("Speedup:    {:.2}x", result.speedup);
}

#[test]
fn benchmark_comprehensive() {
    println!("\n");
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║         Plan 073 Phase 9.1: Performance Benchmarking          ║");
    println!("║              BigVM vs Evaluator Performance                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    let benchmarks = vec![
        ("Simple arithmetic", "1 + 2 * 3 - 4 / 2"),
        ("Loop 1000", r#"
fn main() -> int {
    let mut sum = 0
    for i in 0..1000 {
        sum = sum + i
    }
    sum
}
"#),
        ("Function calls", r#"
fn add(a int, b int) int { a + b }
fn main() -> int {
    let mut sum = 0
    for i in 0..100 {
        sum = add(sum, i)
    }
    sum
}
"#),
        ("Recursion factorial(10)", r#"
fn factorial(n int) int {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}
fn main() -> int { factorial(10) }
"#),
        ("List operations", r#"
fn main() -> int {
    let list = List.new()
    for i in 0..50 { list.push(i) }
    let mut sum = 0
    for i in 0..50 { sum = sum + list.get(i) }
    sum
}
"#),
    ];

    let mut results = Vec::new();

    for (name, source) in benchmarks {
        let result = run_benchmark(name, source);
        results.push(result);
    }

    println!("\n┌────────────────────────────────┬──────────────┬──────────────┬──────────┐");
    println!("│ Benchmark                     │ Evaluator    │ BigVM        │ Speedup  │");
    println!("│                               │ (μs)         │ (μs)         │ (x)      │");
    println!("├────────────────────────────────┼──────────────┼──────────────┼──────────┤");

    for result in &results {
        println!("│ {:30}│ {:12} │ {:12} │ {:8.2} │",
            result.name,
            result.evaluator_time_us,
            result.bigvm_time_us,
            result.speedup
        );
    }

    println!("└────────────────────────────────┴──────────────┴──────────────┴──────────┘");

    // Calculate average speedup
    let avg_speedup: f64 = results.iter()
        .map(|r| r.speedup)
        .filter(|&s| s.is_finite() && s > 0.0)
        .sum::<f64>() / results.len() as f64;

    println!("\n📊 Average Speedup: {:.2}x", avg_speedup);

    if avg_speedup >= 1.0 {
        println!("✅ BigVM is faster than or equal to Evaluator");
    } else {
        println!("⚠️  BigVM is slower than Evaluator (needs optimization)");
    }
}
