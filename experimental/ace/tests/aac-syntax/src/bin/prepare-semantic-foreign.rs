use ace_abi::rust_provider::{
    build_source, c_authority, f0_authority, f0_c_header, f0_c_source, f0_rich_provider_source,
    open_c_provider,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn write_page(root: &Path, page: &[u8]) -> Result<(), std::io::Error> {
    fs::write(root.join("source-declarations.json"), page)?;
    fs::write(
        root.join("authority.blake3"),
        format!("{}\n", blake3::hash(page)),
    )
}

fn prepare_c(
    root: &Path,
    name: &str,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    fs::create_dir(root)?;
    let fixture = |source: &str| {
        if name == "f0_c" {
            source.to_owned()
        } else {
            source.replace("f0_c", name).replace("f0c_", "alternative_")
        }
    };
    fs::write(root.join(format!("{name}.h")), fixture(f0_c_header()))?;
    fs::write(root.join(format!("{name}.c")), fixture(f0_c_source()))?;
    let provider = open_c_provider(root, name)?;
    let authority = c_authority(&provider, 1);
    let admitted = provider.manifest.admit(&authority)?;
    let serde_json::Value::Array(entries) =
        serde_json::to_value(admitted.source_declarations_ref())?
    else {
        return Err("expected an array of normalized declarations".into());
    };
    println!("{} provider={}", root.display(), provider.provider_digest);
    Ok(entries)
}

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
    write_page(&root, &page)?;
    println!("{} provider={}", root.display(), build.provider_digest);
    let c_root = root.join("c-provider");
    let mut c_entries = prepare_c(&c_root, "f0_c")?;
    write_page(&c_root, &serde_json::to_vec(&c_entries)?)?;
    let alternate_root = root.join("c-alternate");
    c_entries.extend(prepare_c(&alternate_root, "f0_alternate")?);
    write_page(&alternate_root, &serde_json::to_vec(&c_entries)?)?;
    Ok(())
}
