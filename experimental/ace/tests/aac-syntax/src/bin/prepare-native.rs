use std::path::PathBuf;
use std::process::Command;

use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_aac_syntax_conformance::{
    native_fingerprint, syntax_av_modules, BUDGET, DRIVER_EXE, DRIVER_MANIFEST,
    NATIVE_DRIVER_SOURCE,
};
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, TargetOs,
};
use ace_source_query::modules::packages::{
    assemble_stdlib_sys_inputs, compile_assembled_aggregates_inputs,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let root = PathBuf::from(args.next().ok_or("pass a new preparation directory")?);
    if !root.is_absolute() || args.next().is_some() {
        return Err("expected one absolute preparation directory".into());
    }
    std::fs::create_dir(&root)?;
    let generated = root.join("generated");
    std::fs::create_dir(&generated)?;
    let mut files = syntax_av_modules();
    files.push(("main.at".to_owned(), NATIVE_DRIVER_SOURCE.to_owned()));
    let entry = "app::main";
    let mut context = PackageAssemblyContext::new(
        AssemblyBackend::A2R,
        ProviderChoice::Auto,
        TargetOs::Windows,
    );
    let inputs = assemble_stdlib_sys_inputs(&files, entry, &mut context)?;
    let product = compile_assembled_aggregates_inputs(&inputs, entry, BUDGET, &context)
        .map_err(|diagnostics| format!("compile AAC driver: {diagnostics:?}"))?;
    let target = PackageA2xTarget::cold_aggregates(
        &root.join("store"),
        "aac1046-native",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &product,
    )
    .map_err(|err| format!("emit AAC driver: {err:?}"))?;
    for (path, source) in &target.current.files {
        std::fs::write(generated.join(path), source)?;
    }

    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let exe = root.join(DRIVER_EXE);
    let output = Command::new(&rustc)
        .args(["--edition", "2021", "-C", "link-arg=/STACK:16777216"])
        .arg("main.rs")
        .arg("-o")
        .arg(&exe)
        .current_dir(&generated)
        .output()?;
    std::fs::write(root.join("rustc.log"), &output.stderr)?;
    if !output.status.success() {
        return Err(format!("driver rustc failed: {}", output.status).into());
    }
    let tool = Command::new(&rustc).arg("-vV").output()?;
    if !tool.status.success() {
        return Err(format!("read rustc identity: {}", tool.status).into());
    }
    let manifest = format!(
        "{}\n{}\n{}",
        native_fingerprint(),
        blake3::hash(&std::fs::read(&exe)?),
        String::from_utf8(tool.stdout)?,
    );
    std::fs::write(root.join(DRIVER_MANIFEST), manifest)?;
    println!("{}", exe.display());
    Ok(())
}
