//! Bounded feasibility process for a genuine no-warmup restart. Formal
//! TestCase recipe/certificate routing belongs to the worker adapter.
use ace_abi::source_foreign::{Session, Stored, Target};
use ace_foreign_contract::manual::{HostAuthority, ManualManifest, SOURCE_CAP};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema: String,
    store: PathBuf,
    stored: Stored,
    target: Target,
    manifest: PathBuf,
    provider: PathBuf,
    output: PathBuf,
    edits: Vec<Edit>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Edit {
    owner: String,
    declaration: String,
}
fn main() {
    let result = run();
    match result {
        Ok(value) => println!("{value}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
}
fn read(path: &std::path::Path, limit: u64) -> Result<Vec<u8>, String> {
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > limit {
        return Err("source938.probe-input-budget".into());
    }
    std::fs::read(path).map_err(|e| e.to_string())
}
fn run() -> Result<serde_json::Value, String> {
    ace_core::link::enter_self_containment()?;
    let args: Vec<_> = std::env::args().skip(1).collect();
    // These three current host authorizations are SEPARATE from the untrusted
    // manifest/request. A file's self-declared safe marker has no role.
    if args.len() != 8
        || args[0] != "--request"
        || args[2] != "--contract"
        || args[4] != "--provider"
        || args[6] != "--root"
    {
        return Err(
            "usage: --request FILE --contract DIGEST --provider DIGEST --root DIGEST".into(),
        );
    }
    let request: Request =
        serde_json::from_slice(&read(PathBuf::from(&args[1]).as_path(), 4 << 20)?)
            .map_err(|e| e.to_string())?;
    if request.schema != "source938/restart-probe/1"
        || request.edits.is_empty()
        || request.edits.len() > 8
        || request.stored.digest != args[7]
    {
        return Err("source938.probe-envelope".into());
    }
    let manifest = ManualManifest::decode(&read(&request.manifest, 1 << 20)?)?;
    let provider = read(&request.provider, 64 << 20)?;
    let hash = |b: &[u8]| blake3::hash(b).to_hex().to_string();
    if hash(&provider) != args[5] {
        return Err("source938.current-provider".into());
    }
    let host = HostAuthority {
        review_id: "plan938-independent-provider-v1".into(),
        contract_digest: args[3].clone(),
        provider: "independent-c938".into(),
        provider_digest: hash(&provider),
        header_digest: hash(include_bytes!("../../cside/source_foreign938.h")),
        physical_table_digest: hash(include_bytes!("../../cside/source_foreign938.contract")),
        target: ace_foreign_contract::plan::TARGET.into(),
        capabilities: [SOURCE_CAP.into()].into_iter().collect(),
        available_symbols: [
            "provider_add",
            "provider_not",
            "provider_div",
            "provider_tick",
            "provider_trace",
            "provider_calls",
            "provider_mix",
            "provider_ctx",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        binding_generation: 1,
    };
    let environment = manifest.admit(&host)?;
    let store = ace_artifacts::store::ManagedStore::open_existing(&request.store)
        .map_err(|e| e.to_string())?;
    let mut session = Session::restore_stored(
        &store,
        &request.stored,
        request.target,
        &[SOURCE_CAP.into()],
        environment,
    )?;
    let r = session.meter();
    let f = session.frontend().meter();
    let restored = serde_json::json!({"target_production":r.mir_attempts+r.native_attempts+r.c_attempts+r.page_encodes,
        "whole_parse":f.whole_parse_attempts,"body_parse":f.body_parse_attempts,"hir":f.hir_attempts,
        "page_encode":f.page_encode_attempts,"solver":session.frontend().query_counters().solver_runs});
    let mut steps = Vec::new();
    for edit in request.edits {
        let before = session.snapshot()?.digest;
        let result = session
            .prepare(&edit.owner, &edit.declaration)
            .and_then(|c| session.commit(c));
        let accepted = result.is_ok();
        let error = result.err();
        let after = session.snapshot()?.digest;
        steps.push(
            serde_json::json!({"accepted":accepted,"error":error,"before":before,"after":after}),
        );
    }
    drop(provider);
    let run = session.run(&request.output)?;
    Ok(
        serde_json::json!({"schema":"source938/restart-probe-result/1","target":request.target,"restored":restored,
        "steps":steps,"exit":run.exit_code,"stderr":String::from_utf8_lossy(&run.stderr),
        "solver_runs":session.frontend().query_counters().solver_runs}),
    )
}
