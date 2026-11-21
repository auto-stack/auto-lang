//! PLAN-1094: prepare the ordinary AV view's registered provider closures
//! into a single shared provider root (the directory of this executable —
//! the harness forwards it as `ACE_AV_PROVIDER_ROOT`).
//!
//! Registered providers (the parent table `REGISTERED_FOREIGN_PROVIDERS`):
//!   - `rust f0_text` — the frozen 967 F0 text provider (repo source).
//!   - `rust py`      — the 1026 CPython adapter (pinned pyo3 closure,
//!                      built once and cached under the OS temp dir).
//!   - `c parity1093_c`— prepared only when the annotated C provider project
//!                      is supplied (`ACE_AV_C_PROVIDER_DIR`); the 1093
//!                      provider sources were never canonized in the repo
//!                      (known debt), so an absent project is a named skip,
//!                      not a silent green.
//!
//! The run phase never compiles: every closure is fully materialized here,
//! and the worker verifies stamp + DLL SHA-256 at load time.

use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn sha256_hex(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap_or_default();
    ace_core::runtime_contract::sha256_hex(&bytes)
}

fn b64(text: &str) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(text.as_bytes())
}

fn write_descriptor(
    root: &Path,
    kind: &str,
    provider: &str,
    artifact_sha256: &str,
    source_b64: Option<&str>,
    rustc_identity: Option<&str>,
    dll: Option<&str>,
    lib: Option<&str>,
    stamp: Option<&str>,
) -> Result<()> {
    let mut descriptor = serde_json::json!({
        "schema": "plan1093.provider-descriptor/1",
        "kind": kind,
        "provider": provider,
        "artifact_sha256": artifact_sha256,
    });
    for (key, value) in [
        ("source_b64", source_b64),
        ("rustc_identity", rustc_identity),
        ("dll", dll),
        ("lib", lib),
        ("stamp", stamp),
    ] {
        if let Some(value) = value {
            descriptor[key] = serde_json::json!(value);
        }
    }
    std::fs::write(
        root.join(format!("{provider}.descriptor.json")),
        serde_json::to_vec(&descriptor)?,
    )?;
    Ok(())
}

/// The frozen 967 F0 provider (`abi/rustside/f0_text.rs`): the repo-source
/// rust closure with the standard `out/f0-build-stamp.txt` layout.
fn prepare_f0(root: &Path) -> Result<()> {
    let out = root.join("out");
    std::fs::create_dir_all(&out)?;
    let build = ace_abi::rust_provider::build_source(root, ace_abi::rust_provider::f0_provider_source())?;
    if !build.cdylib.is_file() {
        return Err("f0_text cdylib missing after build".into());
    }
    let rustc_identity = build.rustc_identity.clone();
    write_descriptor(
        root,
        "rust",
        "f0_text",
        &sha256_hex(&build.cdylib),
        Some(&b64(ace_abi::rust_provider::f0_provider_source())),
        Some(&rustc_identity),
        None,
        None,
        None,
    )?;
    println!("f0_text: dll {} sha256={}", build.cdylib.display(), sha256_hex(&build.cdylib));
    Ok(())
}

/// The 1026 py-bridge provider: the shared cached pyo3 build is copied
/// into the shared root under the provider's own stamp/dll names so it can
/// coexist with f0_text in ONE provider root.
fn prepare_py(root: &Path) -> Result<()> {
    use ace_abi::rust_provider::{py_bridge_provider_source, py_bridge_shared_build};
    let out = root.join("out");
    std::fs::create_dir_all(&out)?;
    let build = py_bridge_shared_build()?;
    let dll = out.join("py_bridge.dll");
    let lib = out.join("py_bridge.lib");
    std::fs::copy(&build.cdylib, &dll)?;
    std::fs::copy(&build.staticlib, &lib)?;
    let shim_digest = auto_bindgen::rust_import::shim_digest(&build.shim_source);
    let stamp = format!(
        "{}\n{shim_digest}\n{}\n",
        build.provider_digest, build.rustc_identity
    );
    std::fs::write(out.join("py-build-stamp.txt"), stamp)?;
    write_descriptor(
        root,
        "rust",
        "py",
        &sha256_hex(&dll),
        Some(&b64(py_bridge_provider_source())),
        Some(&build.rustc_identity.clone()),
        Some("py_bridge.dll"),
        Some("py_bridge.lib"),
        Some("py-build-stamp.txt"),
    )?;
    println!("py: dll {} sha256={}", dll.display(), sha256_hex(&dll));
    Ok(())
}

/// The 1093 C provider: prepared ONLY when the annotated provider project
/// is supplied (sources were never canonized in the repo — known debt).
fn prepare_c(root: &Path) -> Result<()> {
    use ace_abi::rust_provider::{
        build_c_dll_with, c_authority, compile_c_unit, open_c_provider, sdk_include_dirs,
    };
    let Some(dir) = std::env::var_os("ACE_AV_C_PROVIDER_DIR") else {
        println!("parity1093_c: skipped (ACE_AV_C_PROVIDER_DIR unset; provider sources are 1093-owned debt)");
        return Ok(());
    };
    let project = PathBuf::from(dir);
    let name = "parity1093_c";
    if !project.join(format!("{name}.h")).is_file() {
        println!("parity1093_c: skipped (annotated header {name}.h not found in {})", project.display());
        return Ok(());
    }
    let provider = open_c_provider(&project, name)?;
    let mut extra_units = Vec::new();
    let mut extra_includes = Vec::new();
    let impl_source = project.join(format!("{name}_impl.c"));
    if impl_source.is_file() {
        let obj = compile_c_unit(root, &impl_source)?;
        extra_units.push(obj);
        extra_includes.extend(sdk_include_dirs());
    }
    let dll = build_c_dll_with(&provider, root, &extra_units, &extra_includes)?;
    let staged = root.join(format!("{name}.dll"));
    std::fs::copy(&dll, &staged)?;
    let authority = c_authority(&provider, 1);
    let _ = provider.manifest.clone().admit(&authority)?;
    let manifest_bytes = serde_json::to_vec(&provider.manifest)?;
    std::fs::write(root.join("manifest.json"), &manifest_bytes)?;
    let header_text = std::fs::read_to_string(project.join(format!("{name}.h")))?;
    let unit_text = std::fs::read_to_string(project.join(format!("{name}.c")))?;
    let descriptor = serde_json::json!({
        "schema": "plan1093.provider-descriptor/1",
        "kind": "c",
        "provider": name,
        "artifact_sha256": sha256_hex(&staged),
        "source_b64": b64(&header_text),
        "unit_b64": b64(&unit_text),
        "manifest_b64": b64(&String::from_utf8(manifest_bytes)?),
        "provider_digest": provider.provider_digest,
        "dll": format!("{name}.dll"),
    });
    std::fs::write(
        root.join(format!("{name}.descriptor.json")),
        serde_json::to_vec(&descriptor)?,
    )?;
    println!(
        "parity1093_c: dll {} sha256={} provider_digest={}",
        staged.display(),
        sha256_hex(&staged),
        provider.provider_digest
    );
    Ok(())
}

fn main() -> Result<()> {
    let exe = std::env::current_exe()?;
    let root = exe
        .parent()
        .ok_or("prepare-av-providers exe has no parent dir")?;
    std::fs::create_dir_all(root)?;
    let started = std::time::Instant::now();
    prepare_f0(root)?;
    prepare_py(root)?;
    prepare_c(root)?;
    println!(
        "av provider root ready at {} ({} ms)",
        root.display(),
        started.elapsed().as_millis()
    );
    Ok(())
}
