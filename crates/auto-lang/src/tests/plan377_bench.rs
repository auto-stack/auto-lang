// Plan 377 §8.6 / §4.3: 单槽化性能验收
//
// 对比 push/pop 与 64 位算术在单槽化前后的吞吐。重点压测受 2-slot→1-slot
// 影响最大的路径：f64/i64/u64 的栈往返与算术。这些是最能暴露单槽化是否
// 引入回退的场景。
//
// 运行：cargo test -p auto-lang --lib plan377_bench -- --nocapture --ignored
// （测试体内部循环重复 N 次取最小值，降低噪声）

use crate::vm::engine::AutoVM;
use crate::vm::task::AutoTask;
use crate::vm::virt_memory::VirtualRAM;
use std::time::Instant;

/// 取多次运行的最小耗时（μs），降低调度噪声。
fn bench_min_us<F: FnMut()>(label: &str, iters: usize, rounds: usize, mut f: F) -> u128 {
    let mut best = u128::MAX;
    for _ in 0..rounds {
        let start = Instant::now();
        for _ in 0..iters {
            f();
        }
        let d = start.elapsed().as_micros();
        if d < best {
            best = d;
        }
    }
    println!("  {:32} {} iters × {} rounds = {} μs (min)", label, iters, rounds, best);
    best
}

#[test]
#[ignore]
fn plan377_bench_push_pop_f64() {
    println!("\n=== Plan 377 bench: f64 push/pop (单槽化主路径) ===");
    let mut ram = VirtualRAM::new(1_000_000);
    // 预热
    for _ in 0..1000 {
        ram.push_f64(3.14);
        let _ = ram.pop_f64();
    }
    let t = bench_min_us("push_f64+pop_f64", 2_000_000, 5, || {
        ram.push_f64(3.14);
        let _ = ram.pop_f64();
    });
    println!("  → f64 单次往返 = {} ns/op", (t as f64 / 2_000_000.0) * 1000.0);
}

#[test]
#[ignore]
fn plan377_bench_push_pop_i64_u64() {
    println!("\n=== Plan 377 bench: i64/u64 push/pop ===");
    let mut ram = VirtualRAM::new(1_000_000);
    for _ in 0..1000 {
        ram.push_i64(42);
        let _ = ram.pop_i64();
        ram.push_u64(99);
        let _ = ram.pop_u64();
    }
    let ti = bench_min_us("push_i64+pop_i64", 2_000_000, 5, || {
        ram.push_i64(42);
        let _ = ram.pop_i64();
    });
    let tu = bench_min_us("push_u64+pop_u64", 2_000_000, 5, || {
        ram.push_u64(99);
        let _ = ram.pop_u64();
    });
    println!("  → i64 往返 = {:.2} ns/op, u64 往返 = {:.2} ns/op",
        (ti as f64 / 2_000_000.0) * 1000.0,
        (tu as f64 / 2_000_000.0) * 1000.0);
}

#[test]
#[ignore]
fn plan377_bench_arith_opcodes() {
    println!("\n=== Plan 377 bench: 算术 opcode（经 engine dispatch，含 ADD_D/ADD_U64）===");
    // 用一段含 f64 + i64 + u64 算术的 Auto 脚本压测 end-to-end 编译+执行
    let source = r#"
fn main() {
    let mut d = 0.0
    let mut i = 0
    for k in 0..1000 {
        d = d + 1.5
        i = i + k
    }
    let s = "1000000000"
    let u = s.to_uint()
    print(d)
    print(i)
    print(u)
}
"#;
    let mut best = u128::MAX;
    for _ in 0..7 {
        let start = Instant::now();
        let _ = crate::run_with_mode(source, crate::CompileMode::Script);
        let d = start.elapsed().as_micros();
        if d < best { best = d; }
    }
    println!("  end-to-end (compile+run, 1000 次循环 f64+i64+u64 算术) min = {} μs", best);
}

#[test]
#[ignore]
fn plan377_bench_bigint_overflow_path() {
    println!("\n=== Plan 377 bench: BigInt 堆装箱路径（>2^48，罕见但需确认不劣化）===");
    // 仅验证正确性 + 粗略计时；此路径现实中几乎不触发
    let flash = crate::vm::virt_memory::VirtualFlash::new(1024);
    let vm = AutoVM::new(flash, 1_000_000);
    let mut task = AutoTask::new(0, 1_000_000, 0);
    use crate::vm::ffi::VMConvertible;
    let big = 0xFFFF_FFFF_FFFF_FFFFu64;
    let start = Instant::now();
    let rounds = 100_000;
    for _ in 0..rounds {
        big.push_to_stack(&mut task, &vm).unwrap();
        let back = u64::pop_from_stack(&mut task, &vm).unwrap();
        debug_assert_eq!(back, big);
    }
    let total = start.elapsed().as_nanos();
    println!("  u64::MAX round-trip (堆装箱) × {} = {} ns, {:.2} ns/op",
        rounds, total, total as f64 / rounds as f64);
}
