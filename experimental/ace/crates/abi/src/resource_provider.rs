//! PLAN-939: the finite, independently reviewed RESOURCE provider fixture
//! policy for resource-import certification. Candidate manifests never
//! supply their own authorization; every deck references the ONE reviewed
//! manifest below (resource surface v1).
//!
//! The provider bytes (`cside/resource939.{h,c}`) and the review table
//! (`cside/resource939.contract`) are hand-audited; their digests bind at
//! the host composition root exactly like the 938 policy.
use ace_foreign_contract::manual::*;
use ace_foreign_contract::resource::{
    FieldKind, ResKind, ResOut, ResourceForm, ResourceSig, ViewRole, RESOURCE_CAP,
    RESOURCE_SIG_FORMAT,
};
use ace_foreign_contract::{ErrorModel, FactsSource, FnFact, FnForm, InterfaceFacts, LaneKind};

pub const REVIEW: &str = "plan939-resource-provider-v1";
pub const HEADER: &[u8] = include_bytes!("../cside/resource939.h");
pub const TABLE: &[u8] = include_bytes!("../cside/resource939.contract");
pub const PROVIDER: &[u8] = include_bytes!("../cside/resource939.c");
/// The C-side independent oracle caller (reference leg).
pub const REFERENCE: &[u8] = include_bytes!("../cside/resource939_reference.c");

pub const EVENT_RUNTIME: &str = "#include <stdio.h>\nvoid source939_event(const char *name) { fprintf(stderr, \"source939.call=%s\\n\", name); fflush(stderr); }\n";

fn hash(b: &[u8]) -> String {
    blake3::hash(b).to_hex().to_string()
}

fn counter_rec() -> (String, Vec<(String, FieldKind)>) {
    (
        "Counter".into(),
        vec![("value".into(), FieldKind::I32)],
    )
}

fn txn_rec() -> (String, Vec<(String, FieldKind)>) {
    (
        "TxnResult".into(),
        vec![("value".into(), FieldKind::I32), ("handle".into(), FieldKind::I32)],
    )
}

fn resource_fn(
    symbol: &str,
    params: Vec<ResKind>,
    form: ResourceForm,
    atomic: bool,
    noescape: bool,
    error_codes: Option<Vec<(i32, &'static str)>>,
) -> FnFact {
    FnFact {
        symbol: symbol.into(),
        // The scalar form is inert for resource entries; the resource sig
        // is the single authoritative form.
        form: FnForm::Pure {
            params: vec![],
            ret: LaneKind::I32,
        },
        error_model: error_codes.map(|codes| ErrorModel {
            kind: "status-i32".into(),
            codes: codes.into_iter().map(|(c, r)| (c, r.to_string())).collect(),
        }),
        semantic_ref: None,
        resource: Some(ResourceSig {
            format: RESOURCE_SIG_FORMAT.into(),
            params,
            form,
            atomic,
            noescape,
        }),
    }
}

/// The ONE reviewed resource manifest (surface v1).
pub fn reviewed_resource_manifest() -> ManualManifest {
    let (counter, counter_fields) = counter_rec();
    let (txn, txn_fields) = txn_rec();
    let functions = vec![
        resource_fn(
            "t939_byte_length",
            vec![ResKind::ViewText],
            ResourceForm::Pure { ret: ResOut::U64 },
            false,
            false,
            None,
        ),
        resource_fn(
            "t939_scalar_count",
            vec![ResKind::ViewText],
            ResourceForm::Pure { ret: ResOut::U64 },
            false,
            false,
            None,
        ),
        resource_fn(
            "t939_bump",
            vec![
                ResKind::Rec {
                    rec: counter,
                    fields: counter_fields,
                    role: ViewRole::Mut,
                },
                ResKind::I32,
            ],
            ResourceForm::Fallible {
                ctx: None,
                out: Some(ResOut::I32),
            },
            true,
            false,
            Some(vec![(-2, "seed")]),
        ),
        resource_fn(
            "t939_create",
            vec![ResKind::I32],
            ResourceForm::Pure {
                ret: ResOut::Handle {
                    provider: "r939".into(),
                },
            },
            false,
            false,
            None,
        ),
        resource_fn(
            "t939_get",
            vec![ResKind::Handle {
                provider: "r939".into(),
            }],
            ResourceForm::Fallible {
                ctx: None,
                out: Some(ResOut::I32),
            },
            false,
            false,
            Some(vec![(-1, "handle")]),
        ),
        resource_fn(
            "t939_destroy",
            vec![ResKind::Handle {
                provider: "r939".into(),
            }],
            ResourceForm::Fallible {
                ctx: None,
                out: Some(ResOut::I32),
            },
            false,
            false,
            Some(vec![(-1, "handle")]),
        ),
        resource_fn(
            "t939_txn",
            vec![
                ResKind::I32,
                ResKind::Rec {
                    rec: txn,
                    fields: txn_fields,
                    role: ViewRole::Out,
                },
            ],
            ResourceForm::Fallible {
                ctx: None,
                out: Some(ResOut::I32),
            },
            true,
            false,
            Some(vec![(-3, "alloc"), (-2, "seed")]),
        ),
        resource_fn(
            "t939_fold",
            vec![
                ResKind::Callback { ctx: true },
                ResKind::I32,
                ResKind::I32,
            ],
            ResourceForm::Fallible {
                ctx: None,
                out: Some(ResOut::I32),
            },
            false,
            // §5.1/T-01 freeze: callback lanes are synchronous per-call
            // and NEVER stored (noescape is mandatory, not optional).
            true,
            Some(vec![(-7, "callback"), (-2, "seed")]),
        ),
        resource_fn(
            "t939_live",
            vec![],
            ResourceForm::Pure {
                ret: ResOut::I64,
            },
            false,
            false,
            None,
        ),
    ];
    let declarations = functions
        .iter()
        .map(|f| ManualDecl {
            source_name: f.symbol.clone(),
            symbol: f.symbol.clone(),
            context_symbol: None,
        })
        .collect();
    ManualManifest {
        schema: MANUAL_SCHEMA_V2.into(),
        review_id: REVIEW.into(),
        provider: "r939".into(),
        header_digest: hash(HEADER),
        physical_table_digest: hash(TABLE),
        target: ace_foreign_contract::plan::TARGET.into(),
        profile: ace_foreign_contract::plan::PROFILE.into(),
        callconv: "windows-x64".into(),
        direction: "source-import".into(),
        facts: InterfaceFacts {
            format: ace_foreign_contract::FACTS_FORMAT.into(),
            logical: "r939".into(),
            content_version: "resource-v1".into(),
            source: FactsSource::Typed { id: REVIEW.into() },
            functions,
            effects: vec![
                "pure:t939_byte_length".into(),
                "pure:t939_scalar_count".into(),
                "pure:t939_create".into(),
                "pure:t939_live".into(),
                "checked".into(),
            ],
            capabilities: vec![SOURCE_CAP.into(), RESOURCE_CAP.into()],
        },
        declarations,
    }
}

/// Hand-registry authority for the resource surface.
pub fn resource_authority(
    provider_digest: &str,
    generation: u64,
    caps: &[String],
) -> HostAuthority {
    let manifest = reviewed_resource_manifest();
    let symbols: Vec<String> = manifest
        .facts
        .functions
        .iter()
        .map(|f| f.symbol.clone())
        .collect();
    HostAuthority {
        review_id: REVIEW.into(),
        contract_digest: manifest.contract_digest().expect("reviewed manifest"),
        provider: "r939".into(),
        provider_digest: provider_digest.into(),
        header_digest: hash(HEADER),
        physical_table_digest: hash(TABLE),
        target: ace_foreign_contract::plan::TARGET.into(),
        capabilities: caps.iter().cloned().collect(),
        available_symbols: symbols.into_iter().collect(),
        binding_generation: generation,
    }
}

/// The deck manifest must EQUAL the reviewed resource manifest — no
/// self-supplied authorization, no partial selection.
pub fn qualify_resource(candidate: &ManualManifest, caps: &[String]) -> Result<(), String> {
    candidate.validate()?;
    if *candidate != reviewed_resource_manifest() {
        return Err(
            "foreign.authority: candidate is outside the resource review registry".into(),
        );
    }
    if candidate
        .facts
        .capabilities
        .iter()
        .any(|c| !caps.contains(c))
    {
        return Err("foreign.capability: current host denies resource FFI".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reviewed_resource_manifest_is_valid_and_admits() {
        let manifest = reviewed_resource_manifest();
        manifest.validate().expect("valid /2 resource manifest");
        assert_eq!(manifest.facts.functions.len(), 9);
        let caps = vec![SOURCE_CAP.to_string(), RESOURCE_CAP.to_string()];
        let authority = resource_authority(&"c".repeat(64), 1, &caps);
        let admitted = manifest
            .clone()
            .admit(&authority)
            .expect("resource manifest admits");
        assert!(admitted
            .source_declarations_ref()
            .iter()
            .all(|d| d.resource().is_some()));
    }

    #[test]
    fn qualify_rejects_modified_surface() {
        let caps = vec![SOURCE_CAP.to_string(), RESOURCE_CAP.to_string()];
        let mut manifest = reviewed_resource_manifest();
        assert!(qualify_resource(&manifest, &caps).is_ok());
        manifest.facts.functions.pop();
        assert!(qualify_resource(&manifest, &caps).is_err());
        let mut nocaps = reviewed_resource_manifest();
        nocaps.facts.capabilities = vec![SOURCE_CAP.into()];
        assert!(qualify_resource(&nocaps, &caps).is_err());
    }
}

pub fn hash_pub(b: &[u8]) -> String {
    blake3::hash(b).to_hex().to_string()
}

#[cfg(test)]
mod c_leg_probe {
    use base64::Engine as _;
    #[test]
    fn c_leg_executes_without_panic() {
        let deck = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../../crates/auto-lang/test/cases/ace-ca-d-resources-939/src/utf8-bytes.atomdata"
        ))
        .expect("deck fixture");
        let req = crate::worker::WorkerRequest {
            recipe: "abi/resource-foreign-certify-v1".into(),
            nonce: "n".into(),
            request_hash: "h".into(),
            deck_b64: base64::engine::general_purpose::STANDARD.encode(&deck),
            leg: "c".into(),
            store_root: std::env::temp_dir()
                .join("r939-cleg-probe")
                .to_string_lossy()
                .into_owned(),
            steps: vec![crate::worker::StepIn {
                op: "query".into(),
                from: None,
                to: None,
            }],
        };
        match crate::worker_source::execute(&req) {
            Ok(_) => println!("C-LEG-OK"),
            Err(d) => println!("C-LEG-DIAG: {} {}", d.code, d.message),
        }
    }
}
