use ace_abi::rust_provider::{build_source, f0_authority, f0_rich_provider_source};
use std::{fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let root = PathBuf::from(args.next().ok_or("expected new fixture directory")?);
    if !root.is_absolute() || args.next().is_some() {
        return Err("expected one absolute fixture directory".into());
    }
    fs::create_dir(&root)?;
    let build = build_source(&root, f0_rich_provider_source())?;
    let authority = f0_authority(&build, 1);
    let admitted = build.manual.admit(&authority)?;
    let page = serde_json::to_vec(admitted.source_declarations_ref())?;
    fs::write(root.join("source-declarations.json"), &page)?;
    fs::write(
        root.join("authority.blake3"),
        format!("{}\n", blake3::hash(&page)),
    )?;
    println!("{} provider={}", root.display(), build.provider_digest);
    Ok(())
}
