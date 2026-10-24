//! The frozen reference workload of PLAN-818 T-05 (golden.md §2, frozen at
//! T-01): 14 leaves + 100 callers + main, K = 19 mixed blocks (if/else,
//! guard loop, canceling arithmetic), no blank padding; constants drawn
//! from an LCG (seed `0x818_5eed`) in EMISSION order. The expectation is
//! the generator's closed-form MIRROR — plain Rust arithmetic replaying
//! the same template in lockstep, never a backend. The audit trail
//! (per-caller constants + stepwise acc + final value) satisfies the
//! §2.4 hand-audit obligation; the frozen edit bits are `revision`
//! (fn_041's A0 += n) and the fn_leaf_2 signature conflict.

/// Caller count / leaf count / blocks per caller (frozen).
pub const CALLERS: usize = 100;
pub const LEAVES: usize = 14;
pub const K_BLOCKS: usize = 19;
/// Minimum total source bytes (the frozen ≥256 KiB scale duty).
pub const MIN_SOURCE_BYTES: usize = 262_144;

/// The frozen LCG (T-05 implementation freeze; the draw ORDER and ranges
/// were frozen at T-01 — this is the concrete stream behind them).
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new() -> Lcg {
        Lcg {
            state: 0x818_5eed,
        }
    }

    /// Next draw mapped into `[lo, hi]` (inclusive; frozen ranges).
    fn draw(&mut self, lo: i64, hi: i64) -> i64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let r = (self.state >> 33) as u64;
        lo + (r % ((hi - lo + 1) as u64)) as i64
    }
}

/// Frozen ranges (golden.md §2.1): A0∈[3,40], B∈[2,60], C∈[1,20],
/// D∈[1,25], E∈[1,30], F∈[1,12].
const A0_RANGE: (i64, i64) = (3, 40);
const B_RANGE: (i64, i64) = (2, 60);
const C_RANGE: (i64, i64) = (1, 20);
const D_RANGE: (i64, i64) = (1, 25);
const E_RANGE: (i64, i64) = (1, 30);
const F_RANGE: (i64, i64) = (1, 12);

/// One caller's audit record: the constant list plus the mirrored acc
/// trajectory (per-block result) and the terminal leaf value — the §2.4
/// hand-audit artifact, printed by [`Workload::audit_lines`].
#[derive(Clone, Debug)]
pub struct CallerAudit {
    pub index: usize,
    pub x: i64,
    pub a0: i64,
    /// Per block: [B, C, D, E, F].
    pub blocks: Vec<[i64; 5]>,
    /// acc after each of the 19 blocks.
    pub acc_steps: Vec<i64>,
    pub leaf: usize,
    pub leaf_in: i64,
    pub value: i64,
}

/// The generated reference load: source text, the independent (mirrored)
/// expected value, and the audit trail.
pub struct Workload {
    pub src: String,
    pub expected: i64,
    pub audits: Vec<CallerAudit>,
}

fn leaf_mirror(v: i64, n: usize) -> i64 {
    if v < 0 {
        0 - v
    } else {
        v + n as i64
    }
}

/// Generate the reference load at `revision` (rev 0 = the base load; the
/// frozen body edit bumps fn_041's A0 by `revision`).
pub fn generate(revision: i64) -> Workload {
    let mut lcg = Lcg::new();
    let mut src = String::with_capacity(400 * 1024);
    let mut audits = Vec::with_capacity(CALLERS);

    for n in 1..=LEAVES {
        src.push_str(&format!(
            "fn fn_leaf_{n}(v int) int {{\n    if v < 0 {{\n        return 0 - v\n    }}\n    return v + {n}\n}}\n\n"
        ));
    }

    let mut total: i64 = 0;
    for i in 0..CALLERS {
        let x = (i % 7 + 1) as i64;
        let mut a0 = lcg.draw(A0_RANGE.0, A0_RANGE.1);
        if i == 41 {
            // Frozen body-edit bit: fn_041's A0 += revision.
            a0 += revision;
        }
        src.push_str(&format!(
            "fn fn_{i:03}(x int) int {{\n    var acc = x * {a0}\n"
        ));
        let mut blocks = Vec::with_capacity(K_BLOCKS);
        let mut acc_steps = Vec::with_capacity(K_BLOCKS);
        let mut acc: i64 = x * a0;
        for _ in 0..K_BLOCKS {
            let b = lcg.draw(B_RANGE.0, B_RANGE.1);
            let c = lcg.draw(C_RANGE.0, C_RANGE.1);
            let d = lcg.draw(D_RANGE.0, D_RANGE.1);
            let e = lcg.draw(E_RANGE.0, E_RANGE.1);
            let f = lcg.draw(F_RANGE.0, F_RANGE.1);
            src.push_str(&format!(
                "    if acc > {b} {{\n        acc = acc - {b}\n    }} else {{\n        acc = acc + {c}\n    }}\n    for acc > {e} {{\n        acc = acc - {f}\n    }}\n    acc = acc + {d} - {d}\n"
            ));
            // Mirror (plain i64 replay; the frozen no-trap bound analysis
            // keeps every intermediate far from i32 edges).
            if acc > b {
                acc -= b;
            } else {
                acc += c;
            }
            while acc > e {
                acc -= f;
            }
            acc += d - d;
            blocks.push([b, c, d, e, f]);
            acc_steps.push(acc);
        }
        let leaf = i % LEAVES + 1;
        src.push_str(&format!(
            "    return fn_leaf_{leaf}(acc)\n}}\n\n"
        ));
        let value = leaf_mirror(acc, leaf);
        audits.push(CallerAudit {
            index: i,
            x,
            a0,
            blocks,
            acc_steps,
            leaf,
            leaf_in: acc,
            value,
        });
        total += value;
    }

    src.push_str("fn main() int {\n    var total = 0\n");
    for i in 0..CALLERS {
        let x = i % 7 + 1;
        src.push_str(&format!("    total = total + fn_{i:03}({x})\n"));
    }
    src.push_str("    return total\n}\n");

    Workload {
        src,
        expected: total,
        audits,
    }
}

impl Workload {
    /// The §2.4 audit record for one caller (constant list + stepwise acc
    /// + terminal leaf value), for hand re-derivation against the mirror.
    pub fn audit_lines(&self, i: usize) -> Vec<String> {
        let a = &self.audits[i];
        let mut out = vec![format!(
            "fn_{:03}: x={} A0={} terminal-acc={} (calls fn_leaf_{}(acc))",
            a.index, a.x, a.a0, a.leaf_in, a.leaf
        )];
        for (j, blk) in a.blocks.iter().enumerate() {
            out.push(format!(
                "  block {:02}: B={} C={} D={} E={} F={} -> acc={}",
                j + 1,
                blk[0],
                blk[1],
                blk[2],
                blk[3],
                blk[4],
                a.acc_steps[j]
            ));
        }
        out.push(format!("  value={} (fn_leaf_{})", a.value, a.leaf));
        out
    }
}

/// The frozen signature-conflict edit: fn_leaf_2 gains a parameter — the
/// correct expectation is a FRONTEND diagnosis (every call site breaks),
/// never a crash or a silent pass.
pub fn signature_conflict_source() -> String {
    let base = generate(0).src;
    let old_leaf = "fn fn_leaf_2(v int) int {";
    let new_leaf = "fn fn_leaf_2(v int, w int) int {";
    assert!(
        base.contains(old_leaf),
        "frozen leaf template drifted — golden.md §2.1 is authoritative"
    );
    base.replace(old_leaf, new_leaf)
}
