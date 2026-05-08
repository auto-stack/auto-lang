// Plan 073 Phase 9.1: Performance Benchmarking
// Compares AutoVM vs Evaluator performance

use crate::run;
use std::time::Instant;

/// Benchmark result for a single test case
#[derive(Debug)]
struct BenchmarkResult {
    name: String,
    evaluator_time_us: u128,
    bigvm_time_us: u128,
    speedup: f64,
    _evaluator_result: String,
    _bigvm_result: String,
}

/// Run a benchmark comparing evaluator vs AutoVM
fn run_benchmark(name: &str, source: &str) -> BenchmarkResult {
    // Benchmark Evaluator
    let eval_start = Instant::now();
    let eval_result = run(source).unwrap_or_else(|e| format!("ERROR: {}", e));
    let eval_duration = eval_start.elapsed().as_micros();

    // Benchmark AutoVM (using compile mode script)
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
        _evaluator_result: eval_result,
        _bigvm_result: vm_result,
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
    println!("AutoVM:      {} μs", result.bigvm_time_us);
    println!("Speedup:    {:.2}x", result.speedup);

    // AutoVM should be at least as fast or faster
    // (This is a soft assertion - actual performance may vary)
    if result.speedup < 0.5 {
        println!("WARNING: AutoVM is significantly slower than evaluator");
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
    println!("AutoVM:      {} μs", result.bigvm_time_us);
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
    println!("AutoVM:      {} μs", result.bigvm_time_us);
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
    println!("AutoVM:      {} μs", result.bigvm_time_us);
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
    println!("AutoVM:      {} μs", result.bigvm_time_us);
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
    println!("AutoVM:      {} μs", result.bigvm_time_us);
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
    println!("AutoVM:      {} μs", result.bigvm_time_us);
    println!("Speedup:    {:.2}x", result.speedup);
}

#[test]
fn benchmark_comprehensive() {
    println!("\n");
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║         Plan 073 Phase 9.1: Performance Benchmarking          ║");
    println!("║              AutoVM vs Evaluator Performance                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    let benchmarks = vec![
        ("Simple arithmetic", "1 + 2 * 3 - 4 / 2"),
        (
            "Loop 1000",
            r#"
fn main() -> int {
    let mut sum = 0
    for i in 0..1000 {
        sum = sum + i
    }
    sum
}
"#,
        ),
        (
            "Function calls",
            r#"
fn add(a int, b int) int { a + b }
fn main() -> int {
    let mut sum = 0
    for i in 0..100 {
        sum = add(sum, i)
    }
    sum
}
"#,
        ),
        (
            "Recursion factorial(10)",
            r#"
fn factorial(n int) int {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}
fn main() -> int { factorial(10) }
"#,
        ),
        (
            "List operations",
            r#"
fn main() -> int {
    let list = List.new()
    for i in 0..50 { list.push(i) }
    let mut sum = 0
    for i in 0..50 { sum = sum + list.get(i) }
    sum
}
"#,
        ),
    ];

    let mut results = Vec::new();

    for (name, source) in benchmarks {
        let result = run_benchmark(name, source);
        results.push(result);
    }

    println!("\n┌────────────────────────────────┬──────────────┬──────────────┬──────────┐");
    println!("│ Benchmark                     │ Evaluator    │ AutoVM        │ Speedup  │");
    println!("│                               │ (μs)         │ (μs)         │ (x)      │");
    println!("├────────────────────────────────┼──────────────┼──────────────┼──────────┤");

    for result in &results {
        println!(
            "│ {:30}│ {:12} │ {:12} │ {:8.2} │",
            result.name, result.evaluator_time_us, result.bigvm_time_us, result.speedup
        );
    }

    println!("└────────────────────────────────┴──────────────┴──────────────┴──────────┘");

    // Calculate average speedup
    let avg_speedup: f64 = results
        .iter()
        .map(|r| r.speedup)
        .filter(|&s| s.is_finite() && s > 0.0)
        .sum::<f64>()
        / results.len() as f64;

    println!("\n📊 Average Speedup: {:.2}x", avg_speedup);

    if avg_speedup >= 1.0 {
        println!("✅ AutoVM is faster than or equal to Evaluator");
    } else {
        println!("⚠️  AutoVM is slower than Evaluator (needs optimization)");
    }
}

// ============================================================================
// Plan 077 Phase 7: Downcast Performance Benchmarks
// ============================================================================

#[test]
fn benchmark_downcast_performance() {
    use crate::vm::heap_object::{try_downcast_checked, TypeTag};
    use crate::vm::types::ListData;
    use std::time::Instant;

    println!("\n=== Plan 077 Phase 7: Downcast Performance Benchmark ===\n");

    // Create test list
    let list: ListData<i32> = ListData::new();
    let list_obj: &dyn crate::vm::heap_object::HeapObject = &list;

    // Benchmark 1: Type tag check only (baseline)
    let iterations = 1_000_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _tag = list_obj.type_tag();
    }
    let baseline_ns = start.elapsed().as_nanos() / iterations as u128;
    println!("Type tag check:      {} ns/op", baseline_ns);

    // Benchmark 2: Optimized downcast (with type check)
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = try_downcast_checked::<ListData<i32>>(list_obj, TypeTag::ListInt);
    }
    let optimized_ns = start.elapsed().as_nanos() / iterations as u128;
    println!("Optimized downcast:  {} ns/op", optimized_ns);

    // Benchmark 3: Direct downcast (without type check)
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = list_obj.as_any().downcast_ref::<ListData<i32>>();
    }
    let direct_ns = start.elapsed().as_nanos() / iterations as u128;
    println!("Direct downcast:     {} ns/op", direct_ns);

    // Analysis
    println!("\n📊 Analysis:");
    println!(
        "  Optimized overhead: {} ns",
        optimized_ns.saturating_sub(direct_ns)
    );
    println!("  Type check overhead: {} ns", baseline_ns);

    // Verify target met
    if optimized_ns < 10 {
        println!(
            "  ✅ TARGET MET: Optimized downcast < 10ns (actual: {}ns)",
            optimized_ns
        );
    } else {
        println!(
            "  ⚠️  TARGET NOT MET: Optimized downcast > 10ns (actual: {}ns)",
            optimized_ns
        );
    }

    // Assert optimized is not slower than direct by more than 2x (or both are 0)
    if direct_ns > 0 {
        assert!(
            optimized_ns < direct_ns * 2,
            "Optimized downcast is too slow"
        );
    } else {
        // In release mode, both might be 0 due to optimization
        println!(
            "  ℹ️  Note: Results are 0ns due to compiler optimization (expected in release mode)"
        );
    }
}

#[test]
fn benchmark_unified_registry_operations() {
    use crate::vm::heap_object::{try_downcast_checked, try_downcast_checked_mut, TypeTag};
    use crate::vm::types::ListData;
    use std::sync::{Arc, RwLock};
    use std::time::Instant;

    println!("\n=== Plan 077 Phase 7: Unified Registry Operations Benchmark ===\n");

    // Create unified registry (simplified version)
    let mut registry = std::collections::HashMap::new();
    let list_id: u64 = 1;

    // Insert list into registry
    let list: ListData<i32> = ListData::new();
    registry.insert(list_id, Arc::new(RwLock::new(list)));

    // Benchmark list operations
    let iterations = 100_000;

    // Benchmark 1: Read + downcast
    let start = Instant::now();
    for _ in 0..iterations {
        if let Some(obj) = registry.get(&list_id) {
            let guard = obj.read().unwrap();
            let _ = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt);
        }
    }
    let read_ns = start.elapsed().as_nanos() / iterations as u128;
    println!("Read + downcast:          {} ns/op", read_ns);

    // Benchmark 2: Write + downcast
    let start = Instant::now();
    for i in 0..iterations {
        if let Some(obj) = registry.get(&list_id) {
            let mut guard = obj.write().unwrap();
            if let Some(list) =
                try_downcast_checked_mut::<ListData<i32>>(&mut *guard, TypeTag::ListInt)
            {
                list.push(i);
            }
        }
    }
    let write_ns = start.elapsed().as_nanos() / iterations as u128;
    println!("Write + downcast:         {} ns/op", write_ns);

    // Verify list was updated correctly
    if let Some(obj) = registry.get(&list_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = try_downcast_checked::<ListData<i32>>(&*guard, TypeTag::ListInt) {
            println!("  ✅ Final list length: {}", list.len());
        }
    }

    // Performance target: read + downcast < 20ns
    if read_ns < 20 {
        println!(
            "  ✅ TARGET MET: Read + downcast < 20ns (actual: {}ns)",
            read_ns
        );
    } else {
        println!(
            "  ⚠️  TARGET NOT MET: Read + downcast > 20ns (actual: {}ns)",
            read_ns
        );
    }
}
