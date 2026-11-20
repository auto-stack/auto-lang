//! Prepare the official 967 F0 fixture outside the test process.
//! Output is a caller-owned scratch directory, such as plan-artifacts.

use ace_aac_bridge::{Domains, ForeignConfig, InputIdentity, Mode, Options, Request};
use ace_foreign_contract::{manual::SOURCE_CAP, resource::RESOURCE_CAP};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(std::env::args_os().nth(1).ok_or("expected scratch path")?);
    std::fs::create_dir_all(&output)?;
    let build = ace_abi::rust_provider::build_f0(&output)?;
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
    let lib_hash = blake3::hash(&std::fs::read(&build.cdylib)?);
    let config = ForeignConfig {
        library: build.cdylib.clone(),
        library_hash: *lib_hash.as_bytes(),
        contract_digest: authority.contract_digest,
        provider_digest: authority.provider_digest,
        exports: [declaration.function.symbol.clone()].into_iter().collect(),
        manifest: build.manual,
    };
    let config_bytes = serde_json::to_vec(&config)?;
    let config_hash = blake3::hash(&config_bytes);

    let source = "use.rs f0_text\nfn main() int { return text_byte_len(\"A中😀\\0\") }";
    let mut query = ace_source_query::SourceQuerySession::new("p1051", "m1051");
    query.set_foreign(Some(admitted));
    let product = ace_source_query::compile_text_record_product(
        source,
        "f0-at-1051",
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
        mode: Mode::CompileTime,
        domains: Domains::windows(Mode::CompileTime),
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
    std::fs::write(output.join("request.json"), serde_json::to_vec(&request)?)?;
    std::fs::write(output.join("module.abc"), abc)?;
    std::fs::write(output.join("input.at"), source)?;
    println!("DLL {} BLAKE3={lib_hash}", build.cdylib.display());
    println!("provider.json BLAKE3={config_hash}");
    Ok(())
}
