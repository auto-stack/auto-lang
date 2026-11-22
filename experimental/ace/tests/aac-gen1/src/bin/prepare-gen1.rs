use std::path::PathBuf;

use ace_aac_gen1_conformance::{prepare_gen1, GEN1_EXE};

/// Build/integration preparation lane (PLAN-1053 T-02): compile the whole
/// AAC subject tree S with the bound Gen0 toolchain and link the Gen1
/// native executable with the pinned collections runtime.
///
/// Usage:
///   cargo run -p ace-aac-gen1-conformance --bin prepare-gen1 -- <abs-dir>
///
/// Requires ACE_AAC_COLLECTIONS_LIB (absolute path of the prepared
/// ace-collections-runtime library, profile runtime-dev). Writes
/// `<dir>/aac1053-gen1.exe`, `<dir>/driver.manifest`. Daily tests consume
/// the result via AAC1053_GEN1_DIR and never rebuild it.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let root = PathBuf::from(args.next().ok_or("pass a new absolute preparation directory")?);
    if !root.is_absolute() || args.next().is_some() {
        return Err("expected exactly one absolute preparation directory".into());
    }
    let runtime = PathBuf::from(
        std::env::var_os("ACE_AAC_COLLECTIONS_LIB")
            .ok_or("set ACE_AAC_COLLECTIONS_LIB to the absolute path of the prepared runtime")?,
    );
    let exe = prepare_gen1(&root, &runtime)?;
    println!("{} {}", exe.display(), GEN1_EXE);
    Ok(())
}
