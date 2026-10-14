use auto_test_case::{admission::Limits, discovery::load_suite, report::RunReport};

fn main() {
    let suite = std::env::args().nth(1).expect("suite path");
    let mut limits = Limits::defaults();
    limits.max_expanded_checks_per_run = 1;
    let loaded = load_suite(std::path::Path::new(&suite), &limits).unwrap();
    let checks: usize = loaded.cases.iter().flat_map(|f| &f.specs).map(|c| c.checks.len()).sum();
    println!("expanded_limit={} discovered_checks={} errors={}", limits.max_expanded_checks_per_run, checks, loaded.errors.len());
    let empty = RunReport::new("empty", "hash", &[]);
    println!("empty_exit={}", empty.exit_code());
    println!("{}", empty.to_json());
}
