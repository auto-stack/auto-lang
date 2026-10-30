//! PLAN-864: the full 852 L/V corpus (L01–L07, V01–V07) through the real
//! source pipeline, with the frozen oracles — every positive's hand-computed
//! value, L06's trap, and the List<Str> variant: removed="hello" (5
//! scalars) *10 + xs.get(0)="中" (1 scalar) = 51.

use ace_av::collections_864_support::compile_and_execute;

fn exec(src: &str) -> i32 {
    match compile_and_execute(src, "main") {
        Ok(out) => {
            assert_eq!(out.live_after, 0, "once-release: live after must be 0");
            out.value
        }
        Err(e) => panic!("exec failed: {e}"),
    }
}

const L02: &str = r#"
fn main() int {
    var xs = List<int>.new()
    xs.push(10)
    xs.push(30)
    xs.insert(1, 20)
    xs.remove(1)
    return xs.get(0) + xs.get(1) + xs.len()
}
"#;

const L03: &str = r#"
fn main() int {
    var xs = List<int>.new()
    if xs.len() != 0 { return 1 }
    xs.push(42)
    xs.clear()
    return xs.len()
}
"#;

const L05: &str = r#"
type Item { name Str; code int }
fn main() int {
    var xs = List<Item>.new()
    xs.push(Item("hello", 37))
    let x Item = xs.remove(0)
    return x.name.len() + x.code + xs.len()
}
"#;

const L07: &str = r#"
fn main() int {
    var xs = List<int>.new()
    xs.push(20)
    xs.push(1)
    xs.set(1, 22)
    return xs.get(0) + xs.get(1)
}
"#;

const LSTR: &str = r#"
fn main() int {
    var xs = List<Str>.new()
    xs.push("hello")
    xs.push("中")
    let removed Str = xs.remove(0)
    return removed.len() * 10 + xs.get(0).len()
}
"#;

const V02: &str = r#"
fn divide(a int, b int) Result<int, Str> {
    if b == 0 { return Err("bad") }
    return Ok(a / b)
}
fn main() int {
    let r = divide(84, 2)
    is r {
        Ok(v) -> { return v }
        Err(e) -> { return e.len() }
    }
}
"#;

const V03: &str = r#"
fn fail() Result<int, Str> { return Err("bad") }
fn main() int {
    let r = fail()
    is r {
        Ok(v) -> { return v }
        Err(e) -> { return e.len() }
    }
}
"#;

const V04: &str = r#"
enum Token {
    Number(int)
    End
}
fn main() int {
    let t = Token.Number(42)
    is t {
        Token.Number(v) -> { return v }
        Token.End -> { return 1 }
    }
}
"#;

const V05: &str = r#"
type Item { name Str; code int }
enum Token {
    Data(Item)
    End
}
fn main() int {
    let t = Token.Data(Item("hello", 37))
    is t {
        Token.Data(v) -> { return v.name.len() + v.code }
        Token.End -> { return 1 }
    }
}
"#;

const V06: &str = r#"
fn main() int {
    var r Result<int, Str> = Err("old")
    r = Ok(42)
    is r {
        Ok(v) -> { return v }
        Err(e) -> { return e.len() }
    }
}
"#;

const V07: &str = r#"
fn maybe(x int) ?int {
    if x > 0 { return Some(x) }
    return None
}
fn main() int {
    let x = maybe(-1)
    is x {
        Some(v) -> { return v }
        None -> { return 42 }
    }
}
"#;

#[test]
fn corpus_l01() {
    let src = "\nfn main() int {\n    var xs = List<int>.new()\n    xs.push(20)\n    xs.push(22)\n    return xs.get(0) + xs.get(1)\n}\n";
    assert_eq!(exec(src), 42);
}

#[test]
fn corpus_l02_insert_remove() {
    // insert(1, 20) then remove(1) -> [10, 30]; 10+30+len(2) = 42
    assert_eq!(exec(L02), 42);
}

#[test]
fn corpus_l03_empty_clear() {
    assert_eq!(exec(L03), 0);
}

#[test]
fn corpus_l04_owned_type() {
    let src = "type Item { name Str; code int }\nfn main() int {\n    var xs = List<Item>.new()\n    let x Item = Item(\"hello\", 37)\n    xs.push(x.move)\n    return xs.get(0).name.len() + xs.get(0).code\n}\n";
    assert_eq!(exec(src), 42);
}

#[test]
fn corpus_l05_remove_owned() {
    assert_eq!(exec(L05), 5 + 37 + 0);
}

#[test]
fn corpus_l07_set() {
    assert_eq!(exec(L07), 42);
}

/// V-06's List<Str> variant: push hello (5 scalars), push 中 (1 scalar),
/// remove(0) transfers out; removed.len()*10 + xs.get(0).len() = 5*10+1 = 51.
#[test]
fn corpus_list_str_variant_51() {
    assert_eq!(exec(LSTR), 51);
}

#[test]
fn corpus_v01_option_some() {
    let src = "\nfn maybe(x int) ?int {\n    if x > 0 { return Some(x) }\n    return None\n}\nfn main() int {\n    let x = maybe(42)\n    is x {\n        Some(v) -> { return v }\n        None -> { return 1 }\n    }\n}\n";
    assert_eq!(exec(src), 42);
}

#[test]
fn corpus_v02_result_ok() {
    assert_eq!(exec(V02), 42);
}

#[test]
fn corpus_v03_result_err_len() {
    assert_eq!(exec(V03), 3);
}

#[test]
fn corpus_v04_enum_payload() {
    assert_eq!(exec(V04), 42);
}

#[test]
fn corpus_v05_enum_owned_payload() {
    assert_eq!(exec(V05), 42);
}

#[test]
fn corpus_v06_switch_live_branch() {
    assert_eq!(exec(V06), 42);
}

#[test]
fn corpus_v07_option_none() {
    assert_eq!(exec(V07), 42);
}

// L06 keeps its frozen trap (asserted in the av_collections_864 probe).

// ===== for-in iteration (contract C7) =====

const FORIN_COPY: &str = r#"
fn main() int {
    var xs = List<int>.new()
    xs.push(10)
    xs.push(20)
    xs.push(12)
    var total = 0
    for x in xs {
        total = total + x
    }
    return total
}
"#;

const FORIN_OWNED: &str = r#"
type Item { name Str; code int }
fn main() int {
    var xs = List<Item>.new()
    xs.push(Item("ab", 1))
    xs.push(Item("cde", 2))
    var total = 0
    for it in xs {
        total = total + it.name.len() * 10 + it.code
    }
    return total
}
"#;

#[test]
fn corpus_forin_copy_sum() {
    assert_eq!(exec(FORIN_COPY), 42);
}

/// owned-element iteration: field projections per use, no materialization —
/// "ab"(2)*10+1 + "cde"(3)*10+2 = 21 + 32 = 53.
#[test]
fn corpus_forin_owned_projections() {
    assert_eq!(exec(FORIN_OWNED), 53);
}
