use std::path::PathBuf;
use std::process::Command;

use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_aac_syntax_conformance::{
    native_fingerprint, semantic_fingerprint, semantic_modules, syntax_av_modules, BUDGET,
    DRIVER_EXE, DRIVER_MANIFEST, NATIVE_DRIVER_SOURCE, SEMANTIC_DRIVER_EXE, SEMANTIC_DRIVER_SOURCE,
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
    let mode = args.next();
    if !root.is_absolute() || args.next().is_some() {
        return Err("expected an absolute preparation directory and optional mode".into());
    }
    let semantic = match mode.as_deref().and_then(std::ffi::OsStr::to_str) {
        None if mode.is_none() => false,
        Some("semantic") => true,
        #[cfg(all(windows, feature = "aac-ct-bootstrap"))]
        Some("ct") => {
            let exe = ace_aac_syntax_conformance::prepare_ct_bootstrap(&root)?;
            println!("{}", exe.display());
            return Ok(());
        }
        _ => return Err("expected optional semantic or enabled CT driver mode".into()),
    };
    let (mut files, source, executable, store_key) = if semantic {
        (
            semantic_modules(),
            SEMANTIC_DRIVER_SOURCE,
            SEMANTIC_DRIVER_EXE,
            "aac1047-native",
        )
    } else {
        (
            syntax_av_modules(),
            NATIVE_DRIVER_SOURCE,
            DRIVER_EXE,
            "aac1046-native",
        )
    };
    std::fs::create_dir(&root)?;
    let generated = root.join("generated");
    std::fs::create_dir(&generated)?;
    files.push(("main.at".to_owned(), source.to_owned()));
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
        store_key,
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
    let exe = root.join(executable);
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
        if semantic {
            semantic_fingerprint()
        } else {
            native_fingerprint()
        },
        blake3::hash(&std::fs::read(&exe)?),
        String::from_utf8(tool.stdout)?,
    );
    std::fs::write(root.join(DRIVER_MANIFEST), manifest)?;
    println!("{}", exe.display());
    Ok(())
}
