//! PLAN-960 minimal packages store: one root record per published package
//! graph — identity (binding version, grammar, capabilities, input hash),
//! per-unit sources with hashes, and consumer receipts as required edges.
//! The root NEVER carries module state values or host addresses (V-08:
//! recovery re-initializes a fresh generation from re-compiled units).
//!
//! First-cut restore re-parses and re-compiles through the same authorities
//! (parse gate → bind → resolve/typecheck/emit/verify) and then checks the
//! recomputed identity against the root; the no-reparse fast path follows
//! the fact-page work (see D-960-1).
use crate::modules::packages::{compile_bound_packages, PackageProduct};
use ace_artifacts::{
    codec::ChunkKind,
    identity::{QueryKey, RefKind},
    store::{ManagedStore, PutRequest},
};
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    bind_package_syntax, BoundPackages, PackageModuleInput, PACKAGES_BINDING_VERSION,
    PACKAGES_CAPABILITY,
};
use serde::{Deserialize, Serialize};

pub const PACKAGES_ROOT_SCHEMA: &str = "source-packages-root/1";
const MAX_ROOT_BYTES: usize = 512 * 1024;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitRecord {
    package: String,
    version: String,
    module: String,
    source_sha256: String,
    source: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetRecord {
    artifact: String,
    digest: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Root {
    schema: String,
    binding: String,
    grammar: String,
    entry: String,
    capabilities: Vec<String>,
    input_hash: String,
    units: Vec<UnitRecord>,
    targets: Vec<(String, TargetRecord)>,
}

#[derive(Clone, Debug)]
pub struct PackageStoreReceipt {
    pub artifact: String,
    pub digest: String,
}

pub struct PackageStore;

fn sha256_hex(bytes: &[u8]) -> String {
    ace_core::runtime_contract::sha256_hex(bytes)
}

impl PackageStore {
    /// Persist the published graph's root. The previous current root is
    /// never touched; a failure here leaves it intact.
    pub fn prepare(
        path: &std::path::Path,
        product: &PackageProduct,
        inputs: &[PackageModuleInput],
        targets: &[(String, PackageStoreReceipt)],
    ) -> Result<PackageStoreReceipt, String> {
        let bound = &product.bound;
        let mut units = Vec::new();
        for seg in &bound.segments {
            // the lock version is part of the unit identity (D1)
            let version = inputs
                .iter()
                .find(|i| i.package == seg.package && i.id == seg.module)
                .map(|i| i.version.clone())
                .ok_or_else(|| format!("input for {}::{} absent", seg.package, seg.module))?;
            units.push(UnitRecord {
                package: seg.package.clone(),
                version,
                module: seg.module.clone(),
                source_sha256: sha256_hex(seg.source.as_bytes()),
                source: seg.source.clone(),
            });
        }
        let root = Root {
            schema: PACKAGES_ROOT_SCHEMA.into(),
            binding: PACKAGES_BINDING_VERSION.into(),
            grammar: bound.grammar_id.clone(),
            entry: bound.entry.clone(),
            capabilities: bound.capabilities.clone(),
            input_hash: bound.input_hash.clone(),
            units,
            targets: targets
                .iter()
                .map(|(k, r)| {
                    (
                        k.clone(),
                        TargetRecord {
                            artifact: r.artifact.clone(),
                            digest: r.digest.clone(),
                        },
                    )
                })
                .collect(),
        };
        let payload = serde_json::to_vec(&root).map_err(|e| format!("{e:?}"))?;
        if payload.len() > MAX_ROOT_BYTES {
            return Err("packages root resource budget".into());
        }
        let digest = sha256_hex(&payload);
        let key = QueryKey {
            layer: "SPKG960",
            components: vec![
                ("schema".into(), PACKAGES_ROOT_SCHEMA.into()),
                ("digest".into(), digest.clone()),
            ],
        };
        let store = ManagedStore::open(path).map_err(|e| e.to_string())?;
        let artifact = store
            .put_artifact(PutRequest {
                layer: "SPKG960",
                key,
                kind: ChunkKind::SnapshotRecord,
                payload,
                edges: root
                    .targets
                    .iter()
                    .map(|(_, t)| (t.artifact.clone(), RefKind::RequiredData))
                    .collect(),
                project: None,
            })
            .map_err(|e| format!("{e:?}"))?;
        Ok(PackageStoreReceipt { artifact, digest })
    }

    /// Verify the receipt, re-project and re-compile every unit through the
    /// same authorities, and re-admit the identity: the recomputed
    /// `input_hash` must equal the root's. Returns the product (fresh,
    /// uninitialized generation) and the recorded target receipts.
    pub fn restore(
        path: &std::path::Path,
        receipt: &PackageStoreReceipt,
    ) -> Result<(PackageProduct, Vec<(String, PackageStoreReceipt)>), String> {
        let store = ManagedStore::open(path).map_err(|e| e.to_string())?;
        let a = store
            .get_artifact_by_id_bounded(&receipt.artifact, MAX_ROOT_BYTES as u64)
            .map_err(|e| format!("{e:?}"))?
            .ok_or("packages root absent")?;
        if a.kind != ChunkKind::SnapshotRecord || sha256_hex(&a.payload) != receipt.digest {
            return Err("packages root receipt mismatch".into());
        }
        let root: Root = serde_json::from_slice(&a.payload).map_err(|e| format!("{e:?}"))?;
        if root.schema != PACKAGES_ROOT_SCHEMA || root.binding != PACKAGES_BINDING_VERSION {
            return Err("packages root schema/binding mismatch".into());
        }
        let mut inputs = Vec::new();
        let mut units = std::collections::BTreeMap::new();
        for u in &root.units {
            if sha256_hex(u.source.as_bytes()) != u.source_sha256 {
                return Err("packages unit source hash mismatch".into());
            }
            inputs.push(PackageModuleInput {
                package: u.package.clone(),
                version: u.version.clone(),
                id: u.module.clone(),
                source: u.source.clone(),
            });
            let syntax = ace_core::source::packages::parse_package_source(
                &u.source,
                &root.capabilities,
                FrontBudget::WORK_OWNED_64MIB,
            )
            .map_err(|ds| format!("restore re-parse: {:?}", ds.first().map(|d| &d.code)))?;
            units.insert(
                ace_core::source::packages::unit_key(&u.package, &u.module),
                syntax,
            );
        }
        let bound = bind_package_syntax(
            &inputs,
            &root.entry,
            &root.capabilities,
            FrontBudget::WORK_OWNED_64MIB,
            units,
        )
        .map_err(|e| format!("restore re-bind: {} {}", e.module, e.diagnostic.code))?;
        if bound.input_hash != root.input_hash || bound.grammar_id != root.grammar {
            return Err("packages restore identity mismatch".into());
        }
        let caps = root.capabilities.clone();
        let product = compile_bound_packages(bound, &caps).map_err(|ds| {
            format!(
                "restore re-compile: {} {:?}",
                ds[0].module, ds[0].diagnostic.code
            )
        })?;
        let targets = root
            .targets
            .iter()
            .map(|(k, t)| {
                (
                    k.clone(),
                    PackageStoreReceipt {
                        artifact: t.artifact.clone(),
                        digest: t.digest.clone(),
                    },
                )
            })
            .collect();
        Ok((product, targets))
    }

    /// The raw root payload for evidence assertions (schema keys, absence
    /// of state values/host addresses).
    pub fn root_payload(
        path: &std::path::Path,
        receipt: &PackageStoreReceipt,
    ) -> Result<Vec<u8>, String> {
        let store = ManagedStore::open(path).map_err(|e| e.to_string())?;
        let a = store
            .get_artifact_by_id_bounded(&receipt.artifact, MAX_ROOT_BYTES as u64)
            .map_err(|e| format!("{e:?}"))?
            .ok_or("packages root absent")?;
        Ok(a.payload)
    }
}

/// The packages capability set for lanes that need the exact list.
pub fn packages_caps() -> Vec<String> {
    vec![PACKAGES_CAPABILITY.to_string()]
}

/// Bound package graph re-export for store consumers.
pub type BoundPackageGraph = BoundPackages;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::packages::compile_package_product;

    fn input(package: &str, id: &str, source: &str) -> PackageModuleInput {
        PackageModuleInput {
            package: package.into(),
            version: "1.0.0".into(),
            id: id.into(),
            source: source.into(),
        }
    }
    fn dir() -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!(
            "ace-pkg-store-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn packages_root_restores_identity_and_carries_no_state() {
        let inputs = [
            input("lib", "std", "pub const BASE int = 2\npub fn greet() str { return \"m2\" }"),
            input(
                "app",
                "main",
                "use lib.std.greet\nuse lib.std.BASE\nvar count int = 1\nfn main() int { count = count + 1\n return BASE + count + greet().len() }",
            ),
        ];
        let product = compile_package_product(
            &inputs,
            "app::main",
            &packages_caps(),
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap();
        let d = dir();
        let receipt = PackageStore::prepare(&d, &product, &inputs, &[]).unwrap();
        let (restored, targets) = PackageStore::restore(&d, &receipt).unwrap();
        assert!(targets.is_empty());
        assert_eq!(restored.bound.input_hash, product.bound.input_hash);
        // V-08.5: the root carries identity + sources only — no cell values,
        // no addresses.
        let payload = PackageStore::root_payload(&d, &receipt).unwrap();
        let text = String::from_utf8(payload).unwrap();
        assert!(text.contains("source-packages-root/1"));
        for forbidden in ["\"cells\"", "\"globals\"", "0x", "addr"] {
            assert!(!text.contains(forbidden), "{forbidden} leaked into root");
        }
        // tampered digest rejected
        let bad = PackageStoreReceipt {
            artifact: receipt.artifact.clone(),
            digest: "0".repeat(64),
        };
        assert!(PackageStore::restore(&d, &bad).is_err());
        std::fs::remove_dir_all(&d).ok();
    }
}
