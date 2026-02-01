// Performance tests for incremental transpilation (Phase 066)
//
// This test measures the performance improvement of incremental transpilation
// compared to full transpilation.

use auto_lang::{trans_c, trans_c_with_session, trans_rust, trans_rust_with_session, compile::CompileSession};
use std::fs;
use std::io::Write;
use tempfile::TempDir;

/// Prepare test files for benchmarking
fn setup_test_files() -> (String, String) {
    let small_file = r#"
fn add(a int, b int) int {
    a + b
}

fn multiply(x int, y int) int {
    x * y
}

fn main() int {
    let result = add(10, 20)
    multiply(result, 2)
}
"#;

    let large_file = r#"
fn abs(x int) int { if x < 0 { -x } else { x } }

fn min(a int, b int) int { if a < b { a } else { b } }

fn max(a int, b int) int { if a > b { a } else { b } }

fn clamp(x int, low int, high int) int {
    min(max(x, low), high)
}

fn factorial(n int) int {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}

fn fibonacci(n int) int {
    if n <= 1 { n } else { fibonacci(n - 1) + fibonacci(n - 2) }
}

fn gcd(a int, b int) int {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn main() int {
    let x = 42
    let y = 100
    let z = add(x, y)
    let w = multiply(z, 2)
    w
}
"#;

    (small_file.to_string(), large_file.to_string())
}

fn create_test_file(dir: &TempDir, name: &str, content: &str) -> String {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path.to_string_lossy().to_string()
}

#[test]
fn bench_full_vs_incremental_c() {
    let dir = TempDir::new().unwrap();
    let (_small, large) = setup_test_files();

    // Create large test file
    let path = create_test_file(&dir, "bench_test.at", &large);

    // Benchmark: Full transpilation
    let start = std::time::Instant::now();
    let _result1 = trans_c(&path).unwrap();
    let full_duration = start.elapsed();

    // Benchmark: Incremental transpilation (no changes)
    let mut session = CompileSession::new();
    let start = std::time::Instant::now();
    let result2 = trans_c_with_session(&mut session, &path).unwrap();
    let incremental_duration = start.elapsed();

    println!("\n=== C Transpilation Benchmark ===");
    println!("Full transpilation: {:?}", full_duration);
    println!("Incremental (no changes): {:?}", incremental_duration);

    if full_duration > incremental_duration {
        let speedup = full_duration.as_nanos() as f64 / incremental_duration.as_nanos() as f64;
        println!("Speedup: {:.2}x", speedup);
    }

    // Verify output is generated
    assert!(result2.contains("[trans]"));
}

#[test]
fn bench_full_vs_incremental_rust() {
    let dir = TempDir::new().unwrap();
    let (_small, large) = setup_test_files();

    // Create large test file
    let path = create_test_file(&dir, "bench_test.at", &large);

    // Benchmark: Full transpilation
    let start = std::time::Instant::now();
    let _result1 = trans_rust(&path).unwrap();
    let full_duration = start.elapsed();

    // Benchmark: Incremental transpilation (no changes)
    let mut session = CompileSession::new();
    let start = std::time::Instant::now();
    let result2 = trans_rust_with_session(&mut session, &path).unwrap();
    let incremental_duration = start.elapsed();

    println!("\n=== Rust Transpilation Benchmark ===");
    println!("Full transpilation: {:?}", full_duration);
    println!("Incremental (no changes): {:?}", incremental_duration);

    if full_duration > incremental_duration {
        let speedup = full_duration.as_nanos() as f64 / incremental_duration.as_nanos() as f64;
        println!("Speedup: {:.2}x", speedup);
    }

    // Verify output is generated
    assert!(result2.contains("[trans]"));
}

#[test]
fn test_cache_hit_rate() {
    let dir = TempDir::new().unwrap();
    let (small, _large) = setup_test_files();

    // Create test file
    let path = create_test_file(&dir, "cache_test.at", &small);

    let mut session = CompileSession::new();

    // First transpilation (cold cache)
    let result1 = trans_c_with_session(&mut session, &path).unwrap();
    let db = session.db();
    let total_frags = db.read().unwrap().all_fragment_ids().len();

    // Hash the file to mark it as clean
    let file_id = db.read().unwrap().get_file_id_by_path(&path).unwrap();
    db.write().unwrap().hash_file(file_id);

    // Second transpilation (warm cache)
    let result2 = trans_c_with_session(&mut session, &path).unwrap();

    println!("\n=== Cache Hit Rate Test ===");
    println!("Total fragments: {}", total_frags);
    println!("First transpilation: {}", result1);
    println!("Second transpilation: {}", result2);

    // Second transpilation should use cached results (dirty = 0)
    assert!(result2.contains("0 dirty"), "Second run should have 0 dirty fragments after hashing");
}
