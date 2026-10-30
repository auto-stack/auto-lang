//! PLAN-864 full-chain feasibility: real `.at` source through the
//! collections pipeline — frontend → checked HIR → collections lowering →
//! verify_mir → ABC → AV execution. The T-03 probe cases: L01 (scalar
//! List), L04 (owned-element List with field views), V01 (Option + `is`).

use ace_av::collections_864_support::compile_and_execute;

const L01: &str = r#"
fn main() int {
    var xs = List<int>.new()
    xs.push(20)
    xs.push(22)
    return xs.get(0) + xs.get(1)
}
"#;

const L04: &str = r#"
type Item { name Str; code int }
fn main() int {
    var xs = List<Item>.new()
    let x Item = Item("hello", 37)
    xs.push(x.move)
    return xs.get(0).name.len() + xs.get(0).code
}
"#;

const V01: &str = r#"
fn maybe(x int) ?int {
    if x > 0 { return Some(x) }
    return None
}
fn main() int {
    let x = maybe(42)
    is x {
        Some(v) -> { return v }
        None -> { return 1 }
    }
}
"#;

const L06: &str = r#"
fn main() int {
    var xs = List<int>.new()
    xs.push(42)
    return xs.get(1)
}
"#;

fn run(name: &str, src: &str) -> (i32, usize, usize) {
    match compile_and_execute(src, "main") {
        Ok(out) => {
            println!(
                "{name}: value={:?} live={} allocs={} frees={}",
                out.value, out.live_after, out.allocs, out.frees
            );
            (out.value, out.live_after, out.allocs + out.frees)
        }
        Err(e) => {
            panic!("{name}: {e}");
        }
    }
}

#[test]
fn collections_l01_executes_42() {
    let (v, live, _) = run("L01", L01);
    assert_eq!(v, 42);
    assert_eq!(live, 0);
}

#[test]
fn collections_l04_executes_42() {
    let (v, live, _) = run("L04", L04);
    assert_eq!(v, 42);
    assert_eq!(live, 0);
}

#[test]
fn collections_v01_executes_42() {
    let (v, live, _) = run("V01", V01);
    assert_eq!(v, 42);
    assert_eq!(live, 0);
}

#[test]
fn collections_l06_traps_list_index() {
    match compile_and_execute(L06, "main") {
        Ok(_) => panic!("L06 must trap"),
        Err(e) => {
            let msg = format!("{e}");
            println!("L06: {msg}");
            assert!(msg.contains("list-index"), "trap must be list-index: {msg}");
        }
    }
}
