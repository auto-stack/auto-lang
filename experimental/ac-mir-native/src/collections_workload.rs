//! Frozen PLAN-889 representative load: keep the original ≥256KiB/115
//! function scalar load and add an owned collection/variant helper (116 total).
//! The body edit is in that helper, not in a scalar-only substitute or main.
pub const FORMAL_SAMPLES: usize = 30;
pub const CHURN_REVISIONS: usize = 512;
pub const P90_MS: u64 = 2000;
pub const FUNCTIONS: usize = 116;
pub struct Workload {
    pub source: String,
    pub expected: i32,
}
pub fn generate(revision: u32) -> Workload {
    let base = crate::workload::generate(0);
    let declarations = "type Item { name Str; code int }\nenum Token { Data(Item)\nEnd }\n";
    let helper = format!("fn collections_helper(seed int) Token {{\n var xs = List<Token>.new()\n let label Str = \"中\"\n let named = label.clone()\n xs.push(Token.Data(Item(named.move, seed + 41 + {revision})))\n return xs.remove(0)\n}}\n");
    // Collections reserves `for` for iteration and excludes `while`. Its
    // existing loop/break spelling preserves every condition and iteration.
    let scalar = base
        .src
        .lines()
        .map(|line| {
            if let Some(limit) = line
                .strip_prefix("    for acc > ")
                .and_then(|s| s.strip_suffix(" {"))
            {
                format!("    loop {{\n        if acc <= {limit} {{ break }}\n")
            } else {
                format!("{line}\n")
            }
        })
        .collect::<String>();
    let source = scalar.replace("fn main() int {\n", "fn main() int {\n let token = collections_helper(0)\n var owned = 0\n is token { Data(p) -> { owned = p.name.len() + p.code } End -> { owned = 0 } }\n")
        .replace("    return total\n}","    return total + owned\n}");
    Workload {
        source: format!("{declarations}{helper}{source}"),
        expected: i32::try_from(base.expected + 42 + i64::from(revision))
            .expect("frozen revision range fits i32"),
    }
}
