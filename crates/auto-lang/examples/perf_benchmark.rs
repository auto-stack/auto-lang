// Plan 073 Phase 9.1: Performance Benchmarking
// Compares BigVM vs Evaluator performance
//
// Run with: cargo run --example perf_benchmark

use std::time::Instant;

/// Benchmark result for a single test case
struct BenchmarkResult {
    name: String,
    evaluator_time_us: u128,
    bigvm_time_us: u128,
    speedup: f64,
}

/// Run a benchmark comparing evaluator vs BigVM
fn run_benchmark(name: &str, source: &str) -> BenchmarkResult {
    use auto_lang::{run, CompileMode, run_with_mode};

    // Warm-up runs
    let _ = run(source);
    let _ = run_with_mode(source, CompileMode::Script);

    // Benchmark Evaluator (average of 5 runs)
    let mut eval_times = Vec::new();
    for _ in 0..5 {
        let start = Instant::now();
        let _ = run(source);
        eval_times.push(start.elapsed().as_micros());
    }
    let eval_avg = eval_times.iter().sum::<u128>() / eval_times.len() as u128;

    // Benchmark BigVM (average of 5 runs)
    let mut vm_times = Vec::new();
    for _ in 0..5 {
        let start = Instant::now();
        let _ = run_with_mode(source, CompileMode::Script);
        vm_times.push(start.elapsed().as_micros());
    }
    let vm_avg = vm_times.iter().sum::<u128>() / vm_times.len() as u128;

    // Calculate speedup
    let speedup = if vm_avg > 0 {
        eval_avg as f64 / vm_avg as f64
    } else {
        0.0
    };

    BenchmarkResult {
        name: name.to_string(),
        evaluator_time_us: eval_avg,
        bigvm_time_us: vm_avg,
        speedup,
    }
}

fn main() {
    println!();
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║         Plan 073 Phase 9.1: Performance Benchmarking          ║");
    println!("║              BigVM vs Evaluator Performance                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    let benchmarks = vec![
        ("Simple arithmetic", "1 + 2 * 3 - 4 / 2"),
        ("Complex arithmetic", "fn main() -> int { (100 + 50) * 2 - 100 / 4 }"),
        ("Loop 100", r#"
fn main() -> int {
    let mut sum = 0
    for i in 0..100 {
        sum = sum + i
    }
    sum
}
"#),
        ("Loop 1000", r#"
fn main() -> int {
    let mut sum = 0
    for i in 0..1000 {
        sum = sum + i
    }
    sum
}
"#),
        ("Function calls 100", r#"
fn add(a int, b int) int { a + b }
fn main() -> int {
    let mut sum = 0
    for i in 0..100 {
        sum = add(sum, i)
    }
    sum
}
"#),
        ("Recursion factorial(8)", r#"
fn factorial(n int) int {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}
fn main() -> int { factorial(8) }
"#),
        ("Nested loops 10x10", r#"
fn main() -> int {
    let mut count = 0
    for i in 0..10 {
        for j in 0..10 {
            count = count + 1
        }
    }
    count
}
"#),
        ("List operations 50", r#"
fn main() -> int {
    let list = List.new()
    for i in 0..50 { list.push(i) }
    let mut sum = 0
    for i in 0..50 { sum = sum + list.get(i) }
    sum
}
"#),
        ("Object creation 100", r#"
type Point { x int, y int }
fn main() -> int {
    let mut sum = 0
    for i in 0..100 {
        let p = Point(i, i * 2)
        sum = sum + p.x + p.y
    }
    sum
}
"#),
        ("F-string formatting", r#"
fn main() -> int {
    let name = "World"
    let s = f"Hello, {name}!"
    s.len()
}
"#),
    ];

    let mut results = Vec::new();

    for (name, source) in benchmarks {
        println!("Running: {}...", name);
        let result = run_benchmark(name, source);
        results.push(result);
    }

    println!();
    println!("┌────────────────────────────────┬──────────────┬──────────────┬──────────┐");
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

    // Calculate median speedup
    let mut sorted_speedups: Vec<f64> = results.iter()
        .map(|r| r.speedup)
        .filter(|&s| s.is_finite() && s > 0.0)
        .collect();
    sorted_speedups.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_speedup = if sorted_speedups.len() % 2 == 0 {
        (sorted_speedups[sorted_speedups.len() / 2 - 1] + sorted_speedups[sorted_speedups.len() / 2]) / 2.0
    } else {
        sorted_speedups[sorted_speedups.len() / 2]
    };

    // Find best and worst cases
    let best_result = results.iter()
        .max_by(|a, b| a.speedup.partial_cmp(&b.speedup).unwrap())
        .unwrap();
    let worst_result = results.iter()
        .min_by(|a, b| a.speedup.partial_cmp(&b.speedup).unwrap())
        .unwrap();

    println!();
    println!("📊 Performance Summary:");
    println!("   Average Speedup: {:.2}x", avg_speedup);
    println!("   Median Speedup:  {:.2}x", median_speedup);
    println!();
    println!("   Best Case:  {} ({:.2}x)", best_result.name, best_result.speedup);
    println!("   Worst Case: {} ({:.2}x)", worst_result.name, worst_result.speedup);
    println!();

    if avg_speedup >= 1.0 {
        println!("✅ RESULT: BigVM is faster than or equal to Evaluator");
        println!("   The bytecode VM is performing well!");
    } else if avg_speedup >= 0.5 {
        println!("⚠️  RESULT: BigVM is moderately slower than Evaluator");
        println!("   Consider optimization opportunities.");
    } else {
        println!("❌ RESULT: BigVM is significantly slower than Evaluator");
        println!("   Optimization is needed before deprecating evaluator.");
    }

    println!();
    println!("💡 Memory Usage:");
    println!("   Evaluator: TreeWalker interpreter (high memory overhead)");
    println!("   BigVM:     Bytecode VM (lower memory footprint)");
    println!("   Note: Actual memory profiling requires external tools");
    println!();
}
