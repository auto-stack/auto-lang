//! Bounded feasibility process for a genuine no-warmup restart. Formal
//! TestCase recipe/certificate routing belongs to the worker adapter.
use ace_abi::source_foreign::{
    read_pointer_identity, CurrentAuthority, Edit as PublicationEdit, Session, Stored, Target,
};
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
#[serde(untagged)]
enum Edit {
    Body(BodyEdit),
    Foreign(ForeignEdit),
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BodyEdit {
    owner: String,
    declaration: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ForeignEdit {
    foreign_manifest: PathBuf,
    foreign_provider: PathBuf,
    source: Option<String>,
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
    if ![8, 16].contains(&args.len())
        || args[0] != "--request"
        || args[2] != "--contract"
        || args[4] != "--provider"
        || args[6] != "--root"
    {
        return Err(
            "usage: --request FILE --contract DIGEST --provider DIGEST --root DIGEST".into(),
        );
    }
    if args.len() == 16
        && (args[8] != "--candidate-contract"
            || args[10] != "--candidate-header"
            || args[12] != "--candidate-table"
            || args[14] != "--candidate-provider")
    {
        return Err("source938.candidate-host-arguments".into());
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
    let store = ace_artifacts::store::ManagedStore::open_existing(&request.store)
        .map_err(|e| e.to_string())?;
    let mut current = || {
        Ok(ace_abi::source_foreign::CurrentAuthority {
            foreign: host.clone(),
            runtime_digest: ace_abi::source_foreign::certification_runtime_policy(request.target),
            emitter: request.target.identity(),
        })
    };
    let observation = ace_source_query::work::Ledger::default();
    let (restored_session, interface_work) = observation.observe_foreign(|| {
        Session::restore_stored_current(
            &store,
            &request.stored,
            request.target,
            &[SOURCE_CAP.into()],
            &mut current,
        )
    });
    let mut session = restored_session?;
    if session.frontend().foreign().manifest() != &manifest {
        return Err("source938.current-contract".into());
    }
    let r = session.meter();
    let f = session.frontend().meter();
    let restored = serde_json::json!({"interface_production":interface_work.len(),"target_production":r.mir_attempts+r.native_attempts+r.c_attempts+r.page_encodes,
        "whole_parse":f.whole_parse_attempts,"body_parse":f.body_parse_attempts,"hir":f.hir_attempts,
        "page_encode":f.page_encode_attempts,"solver":session.frontend().query_counters().solver_runs});
    let mut active = host.clone();
    let pointer = request.output.join("current.json");
    std::fs::create_dir_all(&request.output).map_err(|e| e.to_string())?;
    let mut prior = request.stored.clone();
    let mut steps = Vec::new();
    for (index, edit) in request.edits.into_iter().enumerate() {
        let before = session.snapshot()?.digest;
        let prepared: Result<_, String> = (|| {
            let (edit, authority) = match edit {
                Edit::Body(edit) => (
                    PublicationEdit::Body(session.prepare(&edit.owner, &edit.declaration)?),
                    active.clone(),
                ),
                Edit::Foreign(edit) => {
                    if args.len() != 16 {
                        return Err("source938.missing-candidate-host-authority".into());
                    }
                    let manifest = ManualManifest::decode(&read(&edit.foreign_manifest, 1 << 20)?)?;
                    let provider = read(&edit.foreign_provider, 64 << 20)?;
                    if hash(&provider) != args[15] {
                        return Err("source938.candidate-provider".into());
                    }
                    let mut authority = host.clone();
                    authority.contract_digest = args[9].clone();
                    authority.header_digest = args[11].clone();
                    authority.physical_table_digest = args[13].clone();
                    authority.provider_digest = args[15].clone();
                    authority.binding_generation = 2;
                    let environment = manifest.admit(&authority)?;
                    let candidate = session.prepare_foreign_source(
                        environment,
                        std::sync::Arc::new(provider),
                        edit.source.as_deref(),
                    )?;
                    (PublicationEdit::Foreign(candidate), authority)
                }
            };
            let mut current = || {
                Ok(CurrentAuthority {
                    foreign: authority.clone(),
                    runtime_digest: ace_abi::source_foreign::certification_runtime_policy(
                        request.target,
                    ),
                    emitter: request.target.identity(),
                })
            };
            let ready = session.prepare_publication(
                edit,
                &request.output.join(format!("edit-{index}")),
                &mut current,
                None,
            )?;
            let expected = read_pointer_identity(&pointer)?;
            let stored = session.publish_prepared(
                ready,
                &store,
                &pointer,
                expected.as_deref(),
                Some(&prior),
                |s| serde_json::to_vec(s).map_err(|e| e.to_string()),
                &mut current,
                None,
            )?;
            Ok((stored, authority))
        })();
        let (accepted, error) = match prepared {
            Ok((stored, authority)) => {
                prior = stored;
                active = authority;
                (true, None)
            }
            Err(error) => (false, Some(error)),
        };
        let after = session.snapshot()?.digest;
        steps.push(
            serde_json::json!({"accepted":accepted,"error":error,"before":before,"after":after}),
        );
    }
    drop(provider);
    let mut current = || {
        Ok(CurrentAuthority {
            foreign: active.clone(),
            runtime_digest: ace_abi::source_foreign::certification_runtime_policy(request.target),
            emitter: request.target.identity(),
        })
    };
    let run = session.run_authorized(&request.output.join("accepted"), &mut current, None)?;
    Ok(
        serde_json::json!({"schema":"source938/restart-probe-result/1","target":request.target,"restored":restored,
        "steps":steps,"exit":run.exit_code,"stderr":String::from_utf8_lossy(&run.stderr),
        "solver_runs":session.frontend().query_counters().solver_runs,
        "main_hir_produced":session.frontend().function_meter("main").hir_produced,
        "independent_hir_produced":session.frontend().function_meter("independent").hir_produced}),
    )
}
