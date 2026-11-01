fn main() {
    // Actual Cargo-selected source, consumed by SDK assembly fingerprints.
    println!(
        "cargo:root={}",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    );
}
