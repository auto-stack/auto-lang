//! The frozen T-05 reference load: 256 KiB of MEANINGFUL scalar source —
//! exactly 100 caller functions + 14 leaves + main, ~13 mixed
//! if/loop/arithmetic blocks per body (no blank padding). The generator
//! derives the INDEPENDENT expected entry result by mirroring the same
//! templates in closed form — no backend run is involved in producing it.
//! Load size, function count, the local-edit position (fn_037's first
//! multiplier) and the signature-conflict edit are FROZEN by
//! contract-decision.md; they must never shrink to make a gate pass.
//! `generate_with_sized` exists ONLY for scaled test smokes (still 100
//! functions — the per-body block count scales); the release-grade
//! turnaround gate always uses the frozen 256 KiB form.

/// A deterministic small-prng so constants vary per function but are
/// reproducible run to run (output must be byte-stable for cache keys).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) & 0xffff) as u32
    }
    fn between(&mut self, lo: u32, hi: u32) -> i32 {
        lo as i32 + (self.next() % (hi - lo + 1)) as i32
    }
}

pub struct Load256 {
    pub source: String,
    pub expected_main: i32,
}

/// Generate the frozen 256 KiB load. `body_edit` flips fn_037's first
/// multiplier (+1), producing a different but still-correct expected.
pub fn generate_with(body_edit: bool) -> Load256 {
    generate_with_sized(body_edit, 256 * 1024)
}

pub fn generate() -> Load256 {
    generate_with(false)
}

/// Size-parameterized core: ALWAYS 100 caller functions (fn_037 exists at
/// every scale); the per-body block count shrinks for small targets.
pub fn generate_with_sized(body_edit: bool, target: usize) -> Load256 {
    let target = target.max(16 * 1024);
    let mut src = String::with_capacity(target + 8192);
    let mut expected: i32 = 0;

    // 14 leaves: fn_leaf_N(v) = v < 0 ? -v : v + N.
    for n in 1..=14usize {
        src.push_str(&format!(
            "fn fn_leaf_{n}(v int) int {{\n    if v < 0 {{\n        return 0 - v\n    }}\n    return v + {n}\n}}\n\n"
        ));
    }
    let leaf = |v: i32, n: i32| -> i32 {
        if v < 0 {
            -v
        } else {
            v + n
        }
    };

    // ~200 bytes per mixed block at full scale; scale the block count down
    // for small targets but keep all 100 functions.
    let blocks_per_fn = ((target.saturating_sub(src.len() + 6144)) / 100 / 148).max(1);
    let mut rng = Lcg(0x798_5eed);
    let mut main_calls = String::new();
    for i in 0..100usize {
        let x = (i as i32 % 7) + 1;
        let a0 = rng.between(3, 40) + if body_edit && i == 37 { 1 } else { 0 };
        src.push_str(&format!("fn fn_{i:03}(x int) int {{\n    var acc = x * {a0}\n"));
        let mut acc = x * a0;
        for _ in 0..blocks_per_fn {
            let b = rng.between(2, 60);
            let c = rng.between(1, 20);
            let d = rng.between(1, 25);
            let e = rng.between(1, 30);
            let f = rng.between(1, 12);
            src.push_str(&format!(
                "    if acc > {b} {{\n        acc = acc - {b}\n    }} else {{\n        acc = acc + {c}\n    }}\n    for acc > {e} {{\n        acc = acc - {f}\n    }}\n    acc = acc + {d} - {d}\n"
            ));
            if acc > b {
                acc -= b;
            } else {
                acc += c;
            }
            while acc > e {
                acc -= f;
            }
        }
        let leaf_n = (i % 14) as i32 + 1;
        src.push_str(&format!(
            "    let extra = fn_leaf_{leaf_n}(acc)\n    return acc + extra\n}}\n\n"
        ));
        let extra = leaf(acc, leaf_n);
        expected = expected.wrapping_add(acc + extra);
        main_calls.push_str(&format!("    total = total + fn_{i:03}({})\n", (i % 7) + 1));
    }
    let mut source = src;
    source.push_str("fn main() int {\n    var total = 0\n");
    source.push_str(&main_calls);
    source.push_str("    return total\n}\n");
    Load256 { source, expected_main: expected }
}

/// The frozen signature-conflict edit: fn_leaf_1 gains a parameter, turning
/// every caller into a bind-time diagnostic request.
pub fn edit_signature_conflict(base: &str) -> String {
    let edited = base.replacen(
        "fn fn_leaf_1(v int) int {",
        "fn fn_leaf_1(v int, w int) int {",
        1,
    );
    assert!(edited != base, "load256: fn_leaf_1 marker missing — frozen load drifted");
    edited
}
