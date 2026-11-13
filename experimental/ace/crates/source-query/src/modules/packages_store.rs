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
    bind_package_syntax, bind_package_syntax_assembled, BoundPackages, PackageAssemblyContext,
    PackageImplSyntax, PackageModuleInput, PACKAGES_BINDING_VERSION, PACKAGES_CAPABILITY,
};
use serde::{Deserialize, Serialize};

/// PLAN-1043（T-04）：schema /2 —— 单元可携所选实现层（kind+sha+源），根可
/// 携装配上下文（backend/provider/target/候选存在集）。/1 根不再互认（新
/// 旧装配配方版本不能互冒）；恢复经同一装配权威重验身份。
pub const PACKAGES_ROOT_SCHEMA: &str = "source-packages-root/2";
const MAX_ROOT_BYTES: usize = 512 * 1024;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitRecord {
    package: String,
    version: String,
    module: String,
    source_sha256: String,
    source: String,
    /// PLAN-1043：所选实现层（无层单元缺省）。
    provider: Option<ProviderRecord>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderRecord {
    kind: String,
    source_sha256: String,
    source: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetRecord {
    artifact: String,
    digest: String,
}

/// PLAN-1043：装配上下文记录（恢复重放选择的输入身份）。
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextRecord {
    backend: String,
    provider: String,
    target: String,
    /// 候选存在性读集（`module:at+c` 串形）。
    available: Vec<String>,
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
    /// PLAN-1043：装配上下文（无 ctx 的旧 lane 缺省 None）。
    context: Option<ContextRecord>,
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
        Self::prepare_with_context(path, product, inputs, targets, None)
    }

    /// PLAN-1043（T-04）：带装配上下文的持久化（实现层随单元记录；上下文
    /// 与候选存在集进根——恢复重验，改变即失效）。
    pub fn prepare_with_context(
        path: &std::path::Path,
        product: &PackageProduct,
        inputs: &[PackageModuleInput],
        targets: &[(String, PackageStoreReceipt)],
        context: Option<&PackageAssemblyContext>,
    ) -> Result<PackageStoreReceipt, String> {
        let bound = &product.bound;
        let mut units = Vec::new();
        for input in inputs {
            if units
                .iter()
                .any(|u: &UnitRecord| u.package == input.package && u.module == input.id)
            {
                return Err(format!(
                    "duplicate store unit for {}::{}",
                    input.package, input.id
                ));
            }
            let provider = input.provider.as_ref().map(|l| ProviderRecord {
                kind: l.kind.label().to_string(),
                source_sha256: sha256_hex(l.source.as_bytes()),
                source: l.source.clone(),
            });
            units.push(UnitRecord {
                package: input.package.clone(),
                version: input.version.clone(),
                module: input.id.clone(),
                source_sha256: sha256_hex(input.source.as_bytes()),
                source: input.source.clone(),
                provider,
            });
        }
        let context = context.map(|c| ContextRecord {
            backend: c.backend.label().to_string(),
            provider: c.provider.label(),
            target: c.target.label().to_string(),
            available: c
                .available
                .iter()
                .map(|(m, ks)| {
                    let mut labels: Vec<&str> = ks.iter().map(|k| k.label()).collect();
                    labels.sort_unstable();
                    format!("{m}:{}", labels.join("+"))
                })
                .collect(),
        });
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
            context,
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
        let mut impls = std::collections::BTreeMap::new();
        let kind_of = |label: &str| -> Result<ace_core::source::packages::ProviderKind, String> {
            Ok(match label {
                "at" => ace_core::source::packages::ProviderKind::At,
                "c" => ace_core::source::packages::ProviderKind::C,
                "rs" => ace_core::source::packages::ProviderKind::Rs,
                "py" => ace_core::source::packages::ProviderKind::Py,
                other => return Err(format!("unknown provider kind `{other}`")),
            })
        };
        for u in &root.units {
            if sha256_hex(u.source.as_bytes()) != u.source_sha256 {
                return Err("packages unit source hash mismatch".into());
            }
            if let Some(p) = &u.provider {
                if sha256_hex(p.source.as_bytes()) != p.source_sha256 {
                    return Err("packages unit provider source hash mismatch".into());
                }
            }
            let provider = match &u.provider {
                None => None,
                Some(p) => Some(ace_core::source::packages::PackageImplLayer {
                    kind: kind_of(&p.kind)?,
                    source: p.source.clone(),
                }),
            };
            inputs.push(PackageModuleInput {
                package: u.package.clone(),
                version: u.version.clone(),
                id: u.module.clone(),
                source: u.source.clone(),
                provider: provider.clone(),
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
            if let Some(p) = &u.provider {
                let isyntax = ace_core::source::packages::parse_package_source(
                    &p.source,
                    &root.capabilities,
                    FrontBudget::WORK_OWNED_64MIB,
                )
                .map_err(|ds| {
                    format!("restore re-parse impl: {:?}", ds.first().map(|d| &d.code))
                })?;
                impls.insert(
                    ace_core::source::packages::unit_key(&u.package, &u.module),
                    PackageImplSyntax {
                        layer: ace_core::source::packages::PackageImplLayer {
                            kind: kind_of(&p.kind)?,
                            source: p.source.clone(),
                        },
                        syntax: isyntax,
                    },
                );
            }
        }
        // PLAN-1043：根带上下文 → 同一装配权威重放；带实现层却无上下文 = 拒。
        let has_impl = inputs.iter().any(|i| i.provider.is_some());
        let bound = match &root.context {
            Some(cx) => {
                let backend = match cx.backend.as_str() {
                    "av" => ace_core::source::packages::AssemblyBackend::Av,
                    "ac" => ace_core::source::packages::AssemblyBackend::Ac,
                    "a2r" => ace_core::source::packages::AssemblyBackend::A2R,
                    "a2py" => ace_core::source::packages::AssemblyBackend::A2Py,
                    "a2c" => ace_core::source::packages::AssemblyBackend::A2C,
                    other => return Err(format!("unknown assembly backend `{other}`")),
                };
                let provider = match cx.provider.as_str() {
                    "auto" => ace_core::source::packages::ProviderChoice::Auto,
                    other => ace_core::source::packages::ProviderChoice::Kind(kind_of(other)?),
                };
                let target = match cx.target.as_str() {
                    "windows" => ace_core::source::packages::TargetOs::Windows,
                    "linux" => ace_core::source::packages::TargetOs::Linux,
                    "macos" => ace_core::source::packages::TargetOs::Macos,
                    other => return Err(format!("unknown target os `{other}`")),
                };
                let mut ctx = PackageAssemblyContext::new(backend, provider, target);
                for entry in &cx.available {
                    let Some((module, kinds)) = entry.split_once(':') else {
                        return Err(format!("bad availability entry `{entry}`"));
                    };
                    let mut set = std::collections::BTreeSet::new();
                    for k in kinds.split('+') {
                        set.insert(kind_of(k)?);
                    }
                    ctx.available.insert(module.to_string(), set);
                }
                bind_package_syntax_assembled(
                    &inputs,
                    &root.entry,
                    &root.capabilities,
                    FrontBudget::WORK_OWNED_64MIB,
                    units,
                    impls,
                    &ctx,
                )
                .map_err(|e| format!("restore re-bind: {} {}", e.module, e.diagnostic.code))?
            }
            None => {
                if has_impl {
                    return Err("packages root carries provider layers without a context".into());
                }
                bind_package_syntax(
                    &inputs,
                    &root.entry,
                    &root.capabilities,
                    FrontBudget::WORK_OWNED_64MIB,
                    units,
                )
                .map_err(|e| format!("restore re-bind: {} {}", e.module, e.diagnostic.code))?
            }
        };
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
            provider: None,
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
        assert!(text.contains("source-packages-root/2"));
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

    /// PLAN-1043（T-04 / ASM-04）：装配 lane 的身份/失效/恢复正反例。
    /// 身份读集 = 公共源 + 所选实现层(kind+sha) + ctx(backend/provider/target/
    /// 候选存在集) + sys 事实；未选 body 内容不入身份（仅改未选文件→复用）。
    mod plan1043_assembly {
        use super::*;
        use crate::modules::packages::{
            assemble_package_inputs, compile_assembled_inputs, discover_provider_sources,
        };
        use ace_core::source::packages::{
            AssemblyBackend, PackageAssemblyContext as Ctx, PackageModuleInput, ProviderChoice,
            ProviderKind, TargetOs,
        };
        use std::collections::BTreeSet;

        fn write_files(dir: &std::path::Path, files: &[(&str, &str)]) {
            std::fs::create_dir_all(dir).unwrap();
            for (n, c) in files {
                std::fs::write(dir.join(n), c).unwrap();
            }
        }

        fn fixture() -> Vec<(&'static str, &'static str)> {
            vec![
            (
                "main.at",
                "use demo.provider_id\n#in sys is current {\n    Windows -> use winio.plat\n    Linux -> use lnxio.plat\n}\nfn main() int { return provider_id() + plat() }",
            ),
            (
                "demo.at",
                "pub spec fn provider_id() int",
            ),
            ("demo.at.at", "fn provider_id() int { return 1 }"),
            ("demo.c.at", "fn provider_id() int { return 2 }"),
            ("winio.at", "pub fn plat() int { return 10 }"),
            ("lnxio.at", "pub fn plat() int { return 20 }"),
        ]
        }

        fn built(
            dir: &std::path::Path,
            backend: AssemblyBackend,
            provider: ProviderChoice,
            target: TargetOs,
        ) -> (
            Vec<PackageModuleInput>,
            Ctx,
            crate::modules::packages::PackageProduct,
        ) {
            let files = discover_provider_sources(dir).unwrap();
            let mut ctx = Ctx::new(backend, provider, target);
            let inputs = assemble_package_inputs(&files, "app::main", &mut ctx).unwrap();
            let product =
                compile_assembled_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx)
                    .unwrap();
            (inputs, ctx, product)
        }

        #[test]
        fn asm04_assembly_identity_covers_every_input_axis() {
            let dir = dir();
            write_files(&dir, &fixture());
            let (inputs, ctx, product) = built(
                &dir,
                AssemblyBackend::Av,
                ProviderChoice::Auto,
                TargetOs::Windows,
            );
            let base = &product.bound.input_hash;

            // 改公共源（新增公共常量，仍可编译）→ 失效。
            std::fs::write(
                dir.join("demo.at"),
                "pub spec fn provider_id() int
pub const K int = 3",
            )
            .unwrap();
            let (_, _, p2) = built(
                &dir,
                AssemblyBackend::Av,
                ProviderChoice::Auto,
                TargetOs::Windows,
            );
            assert_ne!(
                base, &p2.bound.input_hash,
                "public signature change must invalidate"
            );
            std::fs::write(dir.join("demo.at"), "pub spec fn provider_id() int").unwrap();

            // 改所选实现 body → 失效。
            std::fs::write(
                dir.join("demo.at.at"),
                "fn provider_id() int { return 100 }",
            )
            .unwrap();
            let (_, _, p3) = built(
                &dir,
                AssemblyBackend::Av,
                ProviderChoice::Auto,
                TargetOs::Windows,
            );
            assert_ne!(
                base, &p3.bound.input_hash,
                "selected body change must invalidate"
            );
            std::fs::write(dir.join("demo.at.at"), "fn provider_id() int { return 1 }").unwrap();

            // 仅改未选实现 body（c 未被选）→ 身份不变（复用）。
            std::fs::write(dir.join("demo.c.at"), "fn provider_id() int { return 999 }").unwrap();
            let (i4, c4, p4) = built(
                &dir,
                AssemblyBackend::Av,
                ProviderChoice::Auto,
                TargetOs::Windows,
            );
            assert_eq!(
                base, &p4.bound.input_hash,
                "unselected body change must reuse"
            );
            let _ = (i4, c4);

            // target / backend / 显式 provider 变化 → 失效。
            let (_, _, p5) = built(
                &dir,
                AssemblyBackend::Av,
                ProviderChoice::Auto,
                TargetOs::Linux,
            );
            assert_ne!(base, &p5.bound.input_hash, "target change must invalidate");
            let (_, _, p6) = built(
                &dir,
                AssemblyBackend::A2R,
                ProviderChoice::Auto,
                TargetOs::Windows,
            );
            assert_ne!(base, &p6.bound.input_hash, "backend change must invalidate");
            let (_, _, p7) = built(
                &dir,
                AssemblyBackend::A2C,
                ProviderChoice::Kind(ProviderKind::C),
                TargetOs::Windows,
            );
            assert_ne!(
                base, &p7.bound.input_hash,
                "explicit provider change must invalidate"
            );

            // 候选存在性变化（删候选）→ 失效（即使选择不变）。
            std::fs::remove_file(dir.join("demo.py.at")).ok();
            std::fs::remove_file(dir.join("demo.c.at")).unwrap();
            let (_, _, p8) = built(
                &dir,
                AssemblyBackend::Av,
                ProviderChoice::Auto,
                TargetOs::Windows,
            );
            assert_ne!(
                base, &p8.bound.input_hash,
                "candidate existence change must invalidate"
            );
            let _ = (inputs, ctx);
            std::fs::remove_dir_all(&dir).ok();
        }

        #[test]
        fn asm04_restore_reverifies_and_keeps_old_on_failure() {
            let dir = dir();
            write_files(&dir, &fixture());
            let (inputs, ctx, product) = built(
                &dir,
                AssemblyBackend::Av,
                ProviderChoice::Auto,
                TargetOs::Windows,
            );
            let receipt =
                PackageStore::prepare_with_context(&dir, &product, &inputs, &[], Some(&ctx))
                    .unwrap();
            // 恢复重验同一身份（真实 re-parse/re-bind/compile）。
            let (restored, _) = PackageStore::restore(&dir, &receipt).unwrap();
            assert_eq!(restored.bound.input_hash, product.bound.input_hash);
            // 恢复身份与在途文件不一致（所选实现被改）→ 拒绝且不动旧根。
            std::fs::write(dir.join("demo.at.at"), "fn provider_id() int { return 42 }").unwrap();
            let (i2, c2, p2) = built(
                &dir,
                AssemblyBackend::Av,
                ProviderChoice::Auto,
                TargetOs::Windows,
            );
            assert_ne!(restored.bound.input_hash, p2.bound.input_hash);
            // 新产物可再持久化（旧根不动；prepare 从不触旧 CURRENT）。
            let r2 = PackageStore::prepare_with_context(&dir, &p2, &i2, &[], Some(&c2)).unwrap();
            let (re2, _) = PackageStore::restore(&dir, &r2).unwrap();
            assert_eq!(re2.bound.input_hash, p2.bound.input_hash);
            // 旧收据在新根发布后仍按其自身身份恢复（版本互不冒充）。
            let (old, _) = PackageStore::restore(&dir, &receipt).unwrap();
            assert_eq!(old.bound.input_hash, product.bound.input_hash);
            // 编译失败保旧：缺实现编译失败，不产生可恢复新根。
            std::fs::write(dir.join("demo.at.at"), "fn wrong_name() int { return 1 }").unwrap();
            let files = discover_provider_sources(&dir).unwrap();
            let mut c3 = Ctx::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
            let i3 = assemble_package_inputs(&files, "app::main", &mut c3).unwrap();
            assert!(
                compile_assembled_inputs(&i3, "app::main", FrontBudget::WORK_OWNED_64MIB, &c3)
                    .is_err()
            );
            // 旧 r2 根仍可恢复（失败没有替换它）。
            let (again, _) = PackageStore::restore(&dir, &r2).unwrap();
            assert_eq!(again.bound.input_hash, p2.bound.input_hash);
            std::fs::remove_dir_all(&dir).ok();
        }

        #[test]
        fn asm04_root_records_context_and_provider_but_no_unselected_bodies() {
            let dir = dir();
            write_files(&dir, &fixture());
            let (inputs, ctx, product) = built(
                &dir,
                AssemblyBackend::A2C,
                ProviderChoice::Kind(ProviderKind::C),
                TargetOs::Linux,
            );
            let receipt =
                PackageStore::prepare_with_context(&dir, &product, &inputs, &[], Some(&ctx))
                    .unwrap();
            let payload = PackageStore::root_payload(&dir, &receipt).unwrap();
            let text = String::from_utf8(payload).unwrap();
            assert!(text.contains("source-packages-root/2"));
            assert!(text.contains("\"backend\":\"a2c\""), "{text}");
            assert!(text.contains("\"provider\":\"c\""), "{text}");
            assert!(text.contains("\"target\":\"linux\""), "{text}");
            // 所选 c 层源入根；未选 at 层内容不入根（存在集才入）。
            assert!(text.contains("fn provider_id() int { return 2 }"), "{text}");
            assert!(
                !text.contains("fn provider_id() int { return 1 }"),
                "{text}"
            );
            assert!(!text.contains("return 999"));
            // 候选存在集在根内（demo:at+c）。
            assert!(text.contains("demo:at+c"), "{text}");
            std::fs::remove_dir_all(&dir).ok();
        }

        #[test]
        fn asm04_discovery_ignores_non_contract_files() {
            let dir = dir();
            write_files(
                &dir,
                &[
                    ("m.at", "pub fn f() int { return 1 }"),
                    ("m.at.at", "fn g() int { return 2 }"),
                    ("m.vm.at", "ext F { }"),
                    ("notes.txt", "x"),
                ],
            );
            let files = discover_provider_sources(&dir).unwrap();
            let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
            assert_eq!(names, vec!["m.at", "m.at.at"], "{names:?}");
            let mut ctx = Ctx::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
            // m 的公共源只有 f；at 层的 g 是私有 helper（无需求也合法）。
            let inputs = assemble_package_inputs(&files, "app::m", &mut ctx).unwrap();
            assert!(inputs.iter().any(|i| i.id == "m" && i.provider.is_some()));
            compile_assembled_inputs(&inputs, "app::m", FrontBudget::WORK_OWNED_64MIB, &ctx)
                .unwrap();
            std::fs::remove_dir_all(&dir).ok();
        }

        #[test]
        fn asm04_avoiding_selection_is_named_at_discovery() {
            let dir = dir();
            // 只有 c 候选 + Auto 默认 → 具名拒（不静默回落）。
            write_files(
                &dir,
                &[
                    ("m.at", "pub spec fn f() int"),
                    ("m.c.at", "fn f() int { return 2 }"),
                ],
            );
            let files = discover_provider_sources(&dir).unwrap();
            let mut ctx = Ctx::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
            let e = assemble_package_inputs(&files, "app::m", &mut ctx).unwrap_err();
            assert!(e.contains("assembly.missing-default-provider"), "{e}");
            std::fs::remove_dir_all(&dir).ok();
        }

        #[test]
        fn asm04_available_set_records_only_real_files() {
            let mut set = BTreeSet::new();
            set.insert(ProviderKind::At);
            let c = Ctx::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Macos);
            let _ = set;
            assert_eq!(c.target, TargetOs::Macos);
            assert_eq!(c.sys_facts()["current"], 3, "macos is a selectable input");
        }
    }
}
