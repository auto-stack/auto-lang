fn main() {
    let src = std::fs::read_to_string("../../crates/auto-lang/test/cases/ace-m1/src/K01.at").expect("K01");
    let dir = std::env::temp_dir().join(format!("c865-probe-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut s = ace_av::closures_865_support::ClosuresSourceSession::new(&dir);
    let run = s.apply_source(&src, "K01.at").expect("apply");
    println!("hits={} misses={} writes={}", run.production.cache_hits, run.production.cache_misses, run.production.store_writes);
    let _ = std::fs::remove_dir_all(&dir);
}
