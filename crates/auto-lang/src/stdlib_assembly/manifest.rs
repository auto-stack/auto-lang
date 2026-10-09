//! Immutable evidence captured at assembly time. Local paths are deliberately
//! excluded from the serialized identity; they remain in LayerSelection.
use std::path::Path;

use super::model::{fnv1a64, AssemblyContext, AssemblyTarget, Environment, LayerSelection};
use super::reference::ReferenceProof;
include!(concat!(env!("OUT_DIR"), "/assembly_build_inputs.rs"));

#[derive(Debug, Clone, serde::Serialize)]
pub struct SourceInput {
    pub module: String,
    pub source_id: String,
    pub role: String,
    pub content_hash: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProviderInput {
    pub source_id: String,
    pub content_hash: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
struct Payload {
    schema_version: u32,
    target: AssemblyTarget,
    environment: Environment,
    platform: String,
    consumer: String,
    features: Vec<String>,
    sources: Vec<SourceInput>,
    providers: Vec<ProviderInput>,
    references: Vec<ReferenceProof>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AssemblyManifest {
    #[serde(flatten)]
    payload: Payload,
    fingerprint: String,
}

impl AssemblyManifest {
    pub fn freeze(
        context: AssemblyContext,
        consumer: &str,
        project_root: &Path,
        stdlib_root: &Path,
        selections: &[LayerSelection],
        references: Vec<ReferenceProof>,
    ) -> Self {
        let mut sources = Vec::new();
        for selected in selections {
            let public = portable_source_id(
                Path::new(&selected.public_file),
                project_root,
                stdlib_root,
                &selected.module,
            );
            sources.push(SourceInput {
                module: selected.module.clone(),
                source_id: public,
                role: "public".into(),
                content_hash: selected.public_hash,
            });
            if let (Some(path), Some(hash)) = (&selected.context_file, selected.context_hash) {
                sources.push(SourceInput {
                    module: selected.module.clone(),
                    source_id: portable_source_id(
                        Path::new(path),
                        project_root,
                        stdlib_root,
                        &selected.module,
                    ),
                    role: "selected_target".into(),
                    content_hash: hash,
                });
            }
        }
        for reference in &references {
            if !sources.iter().any(|s| s.source_id == reference.declaration) {
                sources.push(SourceInput {
                    module: format!("auto.{}", reference.module),
                    source_id: reference.declaration.clone(),
                    role: "native_declaration".into(),
                    content_hash: reference.declaration_hash,
                });
            }
        }
        sources.sort_by(|a, b| {
            (&a.module, &a.role, &a.source_id).cmp(&(&b.module, &b.role, &b.source_id))
        });
        sources.dedup_by(|a, b| a.source_id == b.source_id && a.role == b.role);
        let mut references = references;
        references.sort_by(|a, b| {
            (&a.module, &a.symbol, a.native_id, &a.callee, &a.producer).cmp(&(
                &b.module,
                &b.symbol,
                b.native_id,
                &b.callee,
                &b.producer,
            ))
        });
        references.dedup_by(|a, b| {
            a.module == b.module
                && a.symbol == b.symbol
                && a.native_id == b.native_id
                && a.callee == b.callee
                && a.producer == b.producer
        });
        let payload = Payload {
            schema_version: 3,
            target: context.target,
            environment: context.environment,
            platform: env!("AUTO_ASSEMBLY_TARGET").into(),
            consumer: consumer.into(),
            features: enabled_features(),
            sources,
            providers: provider_inputs(context.target),
            references,
        };
        let fingerprint = format!(
            "{:016x}",
            fnv1a64(&serde_json::to_string(&payload).unwrap())
        );
        Self {
            payload,
            fingerprint,
        }
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }
    pub fn sources(&self) -> &[SourceInput] {
        &self.payload.sources
    }
    pub fn references(&self) -> &[ReferenceProof] {
        &self.payload.references
    }

    /// Consumer-owned inputs are frozen by value before artifact creation.
    /// The builder consumes the old snapshot, preserving immutable receipts.
    pub fn with_consumer_input(mut self, source_id: &str, text: &str) -> Self {
        self.payload.sources.push(SourceInput {
            module: self.payload.consumer.clone(),
            source_id: source_id.into(),
            role: "consumer_input".into(),
            content_hash: fnv1a64(text),
        });
        self.payload.sources.sort_by(|a, b| {
            (&a.module, &a.role, &a.source_id).cmp(&(&b.module, &b.role, &b.source_id))
        });
        self.fingerprint = format!(
            "{:016x}",
            fnv1a64(&serde_json::to_string(&self.payload).unwrap())
        );
        self
    }
}

pub fn enabled_features() -> Vec<String> {
    env!("AUTO_ASSEMBLY_FEATURES")
        .split(',')
        .filter(|f| !f.is_empty())
        .map(str::to_string)
        .collect()
}

/// These are compiled producer inputs, not claims read from the provider catalog.
/// Public .at files enter only through the actual selection/reference closure.
pub fn provider_inputs(target: AssemblyTarget) -> Vec<ProviderInput> {
    let common = [
        (
            "stdlib/assembly-providers.json",
            include_str!("../../../../stdlib/assembly-providers.json"),
        ),
        ("Cargo.lock", include_str!("../../../../Cargo.lock")),
        (
            "crates/auto-macros/src/lib.rs",
            include_str!("../../../auto-macros/src/lib.rs"),
        ),
    ];
    let mut inputs: Vec<_> = common
        .into_iter()
        .map(|(name, text)| ProviderInput {
            source_id: name.into(),
            content_hash: fnv1a64(text),
        })
        .collect();
    let selected: &[(&str, &str)] = match target {
        AssemblyTarget::Vm => &[
            ("vm/native.rs", include_str!("../vm/native.rs")),
            (
                "vm/native_catalog.rs",
                include_str!("../vm/native_catalog.rs"),
            ),
            ("vm/io.rs", include_str!("../vm/io.rs")),
            ("vm/ffi/stdlib.rs", include_str!("../vm/ffi/stdlib.rs")),
            (
                "vm/ffi/http_stream.rs",
                include_str!("../vm/ffi/http_stream.rs"),
            ),
            (
                "vm/ffi/http_server.rs",
                include_str!("../vm/ffi/http_server.rs"),
            ),
            (
                "vm/ffi/async_http.rs",
                include_str!("../vm/ffi/async_http.rs"),
            ),
        ],
        AssemblyTarget::Rust => &[
            ("a2r_std.rs", include_str!("../a2r_std.rs")),
            ("trans/rust.rs", include_str!("../trans/rust.rs")),
        ],
        AssemblyTarget::C => &[("trans/c.rs", include_str!("../trans/c.rs"))],
    };
    inputs.extend(selected.iter().map(|(name, text)| ProviderInput {
        source_id: format!("crates/auto-lang/src/{name}"),
        content_hash: fnv1a64(text),
    }));
    if target == AssemblyTarget::Rust {
        for (name, source) in [
            ("Cargo.toml", include_str!("../../../a2r-std/Cargo.toml")),
            ("src/lib.rs", include_str!("../../../a2r-std/src/lib.rs")),
            ("src/json.rs", include_str!("../../../a2r-std/src/json.rs")),
            ("src/http.rs", include_str!("../../../a2r-std/src/http.rs")),
            ("src/sse.rs", include_str!("../../../a2r-std/src/sse.rs")),
        ] {
            inputs.push(ProviderInput {
                source_id: format!("crates/a2r-std/{name}"),
                content_hash: fnv1a64(source),
            });
        }
    }
    inputs.sort_by(|a, b| a.source_id.cmp(&b.source_id));
    for &(source_id, content_hash) in BUILD_INPUTS {
        if !inputs.iter().any(|input| input.source_id == source_id) {
            inputs.push(ProviderInput {
                source_id: source_id.into(),
                content_hash,
            });
        }
    }
    inputs.sort_by(|a, b| a.source_id.cmp(&b.source_id));
    inputs
}

fn portable_source_id(path: &Path, project: &Path, stdlib: &Path, module: &str) -> String {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if let Ok(relative) = canonical.strip_prefix(
        stdlib
            .canonicalize()
            .unwrap_or_else(|_| stdlib.to_path_buf()),
    ) {
        return format!(
            "stdlib/auto/{}",
            relative.to_string_lossy().replace('\\', "/")
        );
    }
    if let Ok(relative) = canonical.strip_prefix(
        project
            .canonicalize()
            .unwrap_or_else(|_| project.to_path_buf()),
    ) {
        return format!("project/{}", relative.to_string_lossy().replace('\\', "/"));
    }
    format!(
        "dependency/{module}/{}",
        path.file_name().unwrap_or_default().to_string_lossy()
    )
}

#[cfg(test)]
mod plan738_manifest {
    use super::*;
    #[test]
    fn captured_sources_are_immutable_and_portable() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("m.at");
        std::fs::write(&source, "pub fn value() int { return 1 }").unwrap();
        let mut session = crate::compile::CompileSession::new();
        session.add_source_dir(root.path().to_path_buf());
        session.resolve_uses("use m").unwrap();
        let freeze = || {
            AssemblyManifest::freeze(
                Default::default(),
                "compiler",
                root.path(),
                &root.path().join("stdlib"),
                &session.layer_selections,
                Vec::new(),
            )
        };
        let before = freeze();
        std::fs::write(&source, "pub fn value() int { return 2 }").unwrap();
        assert_eq!(before.fingerprint(), freeze().fingerprint());
        let json = serde_json::to_string(&before).unwrap();
        assert!(!json.contains(&root.path().to_string_lossy().to_string()));
        assert!(json.contains("project/m.at"));
        assert_eq!(
            before.sources()[0].content_hash,
            fnv1a64("pub fn value() int { return 1 }")
        );
    }
}
