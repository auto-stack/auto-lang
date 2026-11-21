//! Prepare the official 967 F0 or PLAN-975 SQLite fixture before testing.
//! Output is a caller-owned scratch directory, such as plan-artifacts.

use ace_aac_bridge::{Domains, ForeignConfig, InputIdentity, Mode, Options, Request};
use ace_foreign_contract::{
    manual::{AdmittedForeign, SOURCE_CAP},
    resource::RESOURCE_CAP,
};
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let output = PathBuf::from(args.next().ok_or("expected scratch path")?);
    std::fs::create_dir_all(&output)?;
    let mut source_file: Option<PathBuf> = None;
    let mut c_provider: Option<(PathBuf, String)> = None;
    while let Some(arg) = args.next() {
        let arg = arg.to_string_lossy().into_owned();
        if arg == "--source-file" {
            source_file = Some(PathBuf::from(
                args.next().ok_or("--source-file needs a path")?,
            ));
        } else if arg == "--c-provider" {
            let dir = PathBuf::from(args.next().ok_or("--c-provider needs a project dir")?);
            let name = args
                .next()
                .ok_or("--c-provider needs a provider name")?
                .to_string_lossy()
                .into_owned();
            c_provider = Some((dir, name));
        } else if arg == "sqlite" {
            return prepare_sqlite(&output);
        } else {
            return Err(format!("unknown foreign fixture argument: {arg}").into());
        }
    }
    if let Some(file) = source_file {
        return prepare_f0_source(&output, &file);
    }
    if let Some((dir, name)) = c_provider {
        return prepare_c_provider(&output, &dir, &name);
    }
    prepare_f0(&output)
}

/// Build a C provider project (annotated `<name>.h` + `<name>.c`) into
/// the prepared layout the ordinary worker loads: the DLL copied as
/// `<output>/<name>.dll` plus a serialized manual manifest and the
/// header digest (provider identity). An optional `<name>_impl.c` unit
/// (compiled with the FULL SDK include closure) links real CRT/library
/// bodies beside the freestanding provider object — the explicit
/// external-library pattern (real `<string.h>` etc.) without widening
/// the frozen 967 provider compile.
fn prepare_c_provider(output: &Path, project: &Path, name: &str) -> Result<()> {
    let provider = ace_abi::rust_provider::open_c_provider(project, name)?;
    let mut extra_units = Vec::new();
    let mut extra_includes = Vec::new();
    let impl_source = project.join(format!("{name}_impl.c"));
    if impl_source.is_file() {
        let obj = ace_abi::rust_provider::compile_c_unit(output, &impl_source)?;
        extra_units.push(obj);
        extra_includes.extend(ace_abi::rust_provider::sdk_include_dirs());
    }
    let dll = ace_abi::rust_provider::build_c_dll_with(&provider, output, &extra_units, &extra_includes)?;
    let staged = output.join(format!("{name}.dll"));
    std::fs::copy(&dll, &staged)?;
    let authority = ace_abi::rust_provider::c_authority(&provider, 1);
    let admitted = provider.manifest.clone().admit(&authority)?;
    let manifest_bytes = serde_json::to_vec(&provider.manifest)?;
    std::fs::write(output.join("manifest.json"), &manifest_bytes)?;
    let manifest_text = String::from_utf8(manifest_bytes.clone())?;
    let header_text = std::fs::read_to_string(project.join(format!("{name}.h")))?;
    let unit_text = std::fs::read_to_string(project.join(format!("{name}.c")))?;
    write_provider_descriptor(
        output,
        "c",
        name,
        &sha256_hex(&staged),
        Some(&b64(&header_text)),
        Some(&b64(&unit_text)),
        Some(&b64(&manifest_text)),
        None,
        Some(&provider.provider_digest),
        Some(&format!("{name}.dll")),
    )?;
    println!(
        "C provider {name}: dll {} sha256={}",
        staged.display(),
        sha256_hex(&staged)
    );
    println!("C provider {name}: provider_digest={}", provider.provider_digest);
    println!(
        "C provider {name}: exports={:?}",
        admitted
            .source_declarations_ref()
            .iter()
            .map(|d| d.function.symbol.clone())
            .collect::<Vec<_>>()
    );
    println!("manifest.json sha256={}", sha256_hex(&output.join("manifest.json")));
    Ok(())
}

fn sha256_hex(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap_or_default();
    ace_core::runtime_contract::sha256_hex(&bytes)
}

/// The worker-side provider closure (schema-strict). `source` rides the
/// closure so the daily run derives the manifest purely; `artifact_sha256`
/// is the REAL SHA-256 of the built DLL the run binds.
fn write_provider_descriptor(
    output: &Path,
    kind: &str,
    provider: &str,
    artifact_sha256: &str,
    source_b64: Option<&str>,
    unit_b64: Option<&str>,
    manifest_b64: Option<&str>,
    rustc_identity: Option<&str>,
    provider_digest: Option<&str>,
    dll: Option<&str>,
) -> Result<()> {
    let mut descriptor = serde_json::json!({
        "schema": "plan1093.provider-descriptor/1",
        "kind": kind,
        "provider": provider,
        "artifact_sha256": artifact_sha256,
    });
    for (key, value) in [
        ("source_b64", source_b64),
        ("unit_b64", unit_b64),
        ("manifest_b64", manifest_b64),
        ("rustc_identity", rustc_identity),
        ("provider_digest", provider_digest),
        ("dll", dll),
    ] {
        if let Some(value) = value {
            descriptor[key] = serde_json::json!(value);
        }
    }
    std::fs::write(output.join(format!("{provider}.descriptor.json")), serde_json::to_vec(&descriptor)?)?;
    println!(
        "{provider}.descriptor.json sha256={}",
        sha256_hex(&output.join(format!("{provider}.descriptor.json")))
    );
    Ok(())
}

fn b64(text: &str) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(text.as_bytes())
}

fn rust_build_stamp(output: &Path) -> Result<(String, String)> {
    // provider digest + shim digest + rustc identity (the worker's stamp axis).
    let stamp = std::fs::read_to_string(output.join("out/f0-build-stamp.txt"))?;
    let lines: Vec<&str> = stamp.lines().collect();
    let provider_digest = lines
        .first()
        .copied()
        .unwrap_or_default()
        .trim()
        .to_string();
    let rustc_identity = lines
        .get(2)
        .copied()
        .unwrap_or_default()
        .trim()
        .to_string();
    Ok((provider_digest, rustc_identity))
}

/// Build the F0 provider from a CALLER-provided annotated source (the
/// identity/rebind leg — a provider body edit must change the value and
/// the artifact identity together).
fn prepare_f0_source(output: &Path, source_file: &Path) -> Result<()> {
    let source = std::fs::read_to_string(source_file)?;
    let build = ace_abi::rust_provider::build_source(output, &source)?;
    let authority = ace_abi::rust_provider::f0_authority(&build, 1);
    let admitted = build.manual.clone().admit(&authority)?;
    let declaration = admitted
        .source_declarations_ref()
        .iter()
        .find(|decl| decl.name == "text_byte_len")
        .ok_or("F0 length export")?;
    println!(
        "F0 declaration effects={:?} symbol={}",
        declaration.effects, declaration.function.symbol
    );
    let (_, rustc_identity) = rust_build_stamp(output)?;
    write_provider_descriptor(
        output,
        "rust",
        "f0_text",
        &sha256_hex(&build.cdylib),
        Some(&b64(&source)),
        None,
        None,
        Some(&rustc_identity),
        None,
        None,
    )?;
    let lib_hash = blake3::hash(&std::fs::read(&build.cdylib)?);
    let config = ForeignConfig {
        library: build.cdylib.clone(),
        library_hash: *lib_hash.as_bytes(),
        contract_digest: authority.contract_digest,
        provider_digest: authority.provider_digest,
        exports: [declaration.function.symbol.clone()].into_iter().collect(),
        manifest: build.manual,
    };
    let source = "use.rs f0_text\nfn main() int { return text_byte_len(\"A中😀\\0\") }";
    write_fixture(
        output,
        "module",
        source,
        Mode::CompileTime,
        &config,
        admitted,
    )
}

fn prepare_f0(output: &Path) -> Result<()> {
    let build = ace_abi::rust_provider::build_f0(output)?;
    let authority = ace_abi::rust_provider::f0_authority(&build, 1);
    let admitted = build.manual.clone().admit(&authority)?;
    let declaration = admitted
        .source_declarations_ref()
        .iter()
        .find(|decl| decl.name == "text_byte_len")
        .ok_or("F0 length export")?;
    println!(
        "F0 declaration effects={:?} symbol={}",
        declaration.effects, declaration.function.symbol
    );
    let (_, rustc_identity) = rust_build_stamp(output)?;
    let frozen = ace_abi::rust_provider::f0_provider_source();
    write_provider_descriptor(
        output,
        "rust",
        "f0_text",
        &sha256_hex(&build.cdylib),
        Some(&b64(frozen)),
        None,
        None,
        Some(&rustc_identity),
        None,
        None,
    )?;
    let lib_hash = blake3::hash(&std::fs::read(&build.cdylib)?);
    let config = ForeignConfig {
        library: build.cdylib.clone(),
        library_hash: *lib_hash.as_bytes(),
        contract_digest: authority.contract_digest,
        provider_digest: authority.provider_digest,
        exports: [declaration.function.symbol.clone()].into_iter().collect(),
        manifest: build.manual,
    };
    let source = "use.rs f0_text\nfn main() int { return text_byte_len(\"A中😀\\0\") }";
    write_fixture(
        output,
        "module",
        source,
        Mode::CompileTime,
        &config,
        admitted,
    )
}

fn prepare_sqlite(output: &Path) -> Result<()> {
    use ace_abi::rust_provider::{
        build_c_dll_with, c_authority, compile_c_unit, locate_pinned_sqlite, open_c_provider,
        p967_sqlite_header, p967_sqlite_source, sdk_include_dirs, stage_pinned_sqlite,
    };

    let sqlite = locate_pinned_sqlite("3.46.0")?;
    let project = output.join("project");
    std::fs::create_dir_all(&project)?;
    std::fs::write(project.join("p967_sqlite.h"), p967_sqlite_header())?;
    std::fs::write(project.join("p967_sqlite.c"), p967_sqlite_source())?;
    stage_pinned_sqlite(&project, &sqlite)?;
    let provider = open_c_provider(&project, "p967_sqlite")?;
    let sqlite_obj = compile_c_unit(output, &project.join("sqlite3.c"))?;
    let dll = build_c_dll_with(
        &provider,
        output,
        &[sqlite_obj, project.join("sqlite3.h")],
        &sdk_include_dirs(),
    )?;
    let authority = c_authority(&provider, 1);
    let admitted = provider.manifest.clone().admit(&authority)?;
    let config = ForeignConfig {
        library_hash: *blake3::hash(&std::fs::read(&dll)?).as_bytes(),
        library: dll,
        contract_digest: authority.contract_digest,
        provider_digest: authority.provider_digest,
        exports: admitted
            .source_declarations_ref()
            .iter()
            .map(|declaration| declaration.function.symbol.clone())
            .collect(),
        manifest: provider.manifest,
    };
    let library = include_str!("../../../../../stdlib/ace/std/sqlite.at");
    let main = r#"
fn main() int {
    if sqlite_open_memory() != 1 { return 101 }
    if sqlite_exec("CREATE TABLE t(a INTEGER); INSERT INTO t VALUES (4); INSERT INTO t VALUES (5);") != 1 { return 102 }
    if sqlite_exec("BEGIN; INSERT INTO t VALUES (100); ROLLBACK;") != 1 { return 103 }
    let sum = sqlite_query_int("SELECT SUM(a) FROM t")
    if sqlite_close() != 1 { return 104 }
    return sum
}
"#;
    let source = format!("use.c p967_sqlite\n{library}\n{main}");
    let rejected = format!(
        "use.c p967_sqlite\n{library}\nfn main() int {{ return sqlite_exec(\"SELECT 1\") }}"
    );
    write_fixture(
        output,
        "reject",
        &rejected,
        Mode::NativeObjects,
        &config,
        admitted.clone(),
    )?;
    write_fixture(
        output,
        "module",
        &source,
        Mode::NativeObjects,
        &config,
        admitted,
    )
}

fn write_fixture(
    output: &Path,
    case: &str,
    source: &str,
    mode: Mode,
    config: &ForeignConfig,
    admitted: AdmittedForeign,
) -> Result<()> {
    let config_bytes = serde_json::to_vec(config)?;
    let config_hash = blake3::hash(&config_bytes);
    let mut query = ace_source_query::SourceQuerySession::new("p1051", "m1051");
    query.set_foreign(Some(admitted));
    let product = ace_source_query::compile_text_record_product(
        source,
        "foreign-fixture-1051",
        &[SOURCE_CAP.into(), RESOURCE_CAP.into()],
        "p1051",
        "m1051",
        &mut query,
    )
    .map_err(|error| format!("{error:?}"))?;
    let mir = ace_lowering::lower_checked_owned(&product.compiled.checked, "d_main", 1051)
        .map_err(|error| format!("{error:?}"))?;
    ace_av_core::verify_mir(&mir).map_err(|error| format!("{error:?}"))?;
    let abc = ace_av_core::encode_abc(&mir)
        .map_err(|error| format!("{error:?}"))?
        .to_flat();
    let request = Request {
        id: [107; 16],
        mode,
        domains: Domains::windows(mode),
        inputs: InputIdentity {
            source: *blake3::hash(source.as_bytes()).as_bytes(),
            grammar: [2; 32],
            passes: [3; 32],
            stdlib: [4; 32],
            dependencies: *config_hash.as_bytes(),
        },
        owner: mir.owner.module,
        mir_schema: mir.owner.schema,
        options: Options { remaining_ms: 5000 },
        capabilities: mir.capabilities,
        external_provider: Some(*config_hash.as_bytes()),
        globals: vec![],
        initializers: vec![],
    };
    std::fs::write(output.join("provider.json"), config_bytes)?;
    let request_name = if case == "module" {
        "request.json".into()
    } else {
        format!("{case}.json")
    };
    std::fs::write(output.join(request_name), serde_json::to_vec(&request)?)?;
    std::fs::write(output.join(format!("{case}.abc")), abc)?;
    std::fs::write(output.join(format!("{case}.at")), source)?;
    println!(
        "DLL {} BLAKE3={}",
        config.library.display(),
        blake3::Hash::from(config.library_hash)
    );
    println!("provider.json BLAKE3={config_hash}");
    Ok(())
}
