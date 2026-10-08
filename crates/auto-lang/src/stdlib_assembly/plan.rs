//! Resolver output shared by batch, persistent and emission entry points.
use std::path::{Path, PathBuf};

use miette::Diagnostic;

use crate::error::{AutoError, AutoResult, MsgWithSource};

use super::model::{fnv1a64, AssemblyContext, AssemblyTarget, LayerKind, ParseStatus};

#[derive(Debug, Clone)]
pub struct AssemblyPlan {
    pub module: String,
    pub context: AssemblyContext,
    pub public_path: PathBuf,
    pub target_path: Option<PathBuf>,
    pub boundary: Option<usize>,
    pub source: String,
    pub fingerprint: u64,
}

impl AssemblyPlan {
    pub fn from_resolved(
        module: &str,
        public: &Path,
        context: AssemblyContext,
    ) -> AutoResult<Self> {
        let public_path = public.canonicalize()?;
        let public_source = std::fs::read_to_string(&public_path)?;
        let stem = public_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| AutoError::Msg("invalid module filename".into()))?
            .strip_suffix(".at")
            .ok_or_else(|| AutoError::Msg("expected public .at source".into()))?;
        let selected =
            public_path.with_file_name(format!("{stem}{}", context.target.context_extension()));
        let mut source = public_source.clone();
        let mut target_path = None;
        let mut boundary = None;
        // VM consumes Auto bodies. Host emitters consume their host provider;
        // side-layer files remain candidates until an emitter implements them.
        if context.target == AssemblyTarget::Vm && selected.is_file() {
            let layer_source = std::fs::read_to_string(&selected)?;
            reject_conflicts(
                module,
                &public_path,
                &public_source,
                &selected,
                &layer_source,
            )?;
            source.push('\n');
            boundary = Some(source.len());
            source.push_str(&layer_source);
            target_path = Some(selected);
        }
        let fingerprint = fnv1a64(&format!(
            "{:?}|{:?}|{}|{}|{}|{}",
            context.target,
            context.environment,
            public_path.display(),
            target_path
                .as_ref()
                .map(|p| p.to_string_lossy())
                .unwrap_or_default(),
            super::providers::catalog_content_fingerprint(),
            source
        ));
        Ok(Self {
            module: module.into(),
            context,
            public_path,
            target_path,
            boundary,
            source,
            fingerprint,
        })
    }

    /// Translate parser labels against the concatenation to the owning source.
    /// No second parse or mutation of the live TypeStore occurs on this path.
    pub fn map_error(&self, error: AutoError) -> AutoError {
        if let AutoError::MultipleErrors {
            count,
            plural,
            errors,
        } = error
        {
            return AutoError::MultipleErrors {
                count,
                plural,
                errors: errors.into_iter().map(|e| self.map_error(e)).collect(),
            };
        }
        let label = error.labels().and_then(|mut labels| labels.next());
        let (offset, length) = label.map(|l| (l.offset(), l.len())).unwrap_or((0, 0));
        let (path, source, offset) = match (self.boundary, &self.target_path) {
            (Some(boundary), Some(path)) if offset >= boundary => {
                (path, &self.source[boundary..], offset - boundary)
            }
            (Some(boundary), _) => (&self.public_path, &self.source[..boundary - 1], offset),
            _ => (&self.public_path, self.source.as_str(), offset),
        };
        AutoError::MsgWithSource(MsgWithSource {
            message: format!(
                "{}: {}",
                error
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "STDASSEMBLY.PARSE_FAIL".into()),
                error
            ),
            source: miette::NamedSource::new(path.to_string_lossy(), source.to_string()),
            span: (
                offset.min(source.len()),
                length.min(source.len().saturating_sub(offset)),
            )
                .into(),
            help: Some(format!(
                "module {} / {:?} / {:?}",
                self.module, self.context.target, self.context.environment
            )),
        })
    }
}

fn reject_conflicts(
    module: &str,
    public_path: &Path,
    public_source: &str,
    target_path: &Path,
    target_source: &str,
) -> AutoResult<()> {
    let public = super::loader::parse_layer(
        public_source,
        LayerKind::Public,
        public_path.display().to_string(),
    );
    let target = super::loader::parse_layer(
        target_source,
        LayerKind::Vm,
        target_path.display().to_string(),
    );
    // Dependent type declarations may require the combined parser; it remains
    // authoritative for parse errors. Only compare successfully parsed symbols.
    if !matches!(public.parse, ParseStatus::Parsed { .. })
        || !matches!(target.parse, ParseStatus::Parsed { .. })
    {
        return Ok(());
    }
    for declaration in &public.symbols {
        for implementation in target
            .symbols
            .iter()
            .filter(|s| s.name == declaration.name && s.kind == declaration.kind)
        {
            if declaration.has_body && (implementation.has_body || implementation.is_vm_decl) {
                let errors = [
                    (&public_path, public_source, declaration.source_span),
                    (&target_path, target_source, implementation.source_span),
                ]
                .into_iter()
                .map(|(path, source, span)| {
                    let (start, length) = span.unwrap_or((0, 0));
                    AutoError::MsgWithSource(MsgWithSource {
                        message: format!(
                            "STDASSEMBLY.PROVIDER_CONFLICT: {module}.{} has multiple bodies",
                            declaration.name
                        ),
                        source: miette::NamedSource::new(
                            path.to_string_lossy(),
                            source.to_string(),
                        ),
                        span: (start, length).into(),
                        help: None,
                    })
                })
                .collect();
                return Err(AutoError::MultipleErrors {
                    count: 2,
                    plural: "s".into(),
                    errors,
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod plan738_repairs {
    use super::*;
    use crate::compile::CompileSession;

    #[test]
    fn duplicate_body_rejected_before_registration() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("m.at"), "pub fn f() int { return 1 }\n").unwrap();
        std::fs::write(dir.path().join("m.vm.at"), "#[vm]\npub fn f() int;\n").unwrap();
        let mut session = CompileSession::new();
        session.add_source_dir(dir.path().to_path_buf());
        let error = session.resolve_uses("use m: *").unwrap_err();
        assert!(error.to_string().contains("PROVIDER_CONFLICT"), "{error:?}");
        assert!(session.layer_selections.is_empty());
    }

    #[test]
    fn same_session_changed_layer_requires_new_epoch() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("m.at"), "pub fn f() int { return 1 }\n").unwrap();
        std::fs::write(
            dir.path().join("m.vm.at"),
            "#[vm]\npub fn old_layer() int;\n",
        )
        .unwrap();
        let mut session = CompileSession::new();
        session.add_source_dir(dir.path().to_path_buf());
        session.resolve_uses("use m: *").unwrap();
        session.resolve_uses("use m: *").unwrap();
        std::fs::write(
            dir.path().join("m.vm.at"),
            "#[vm]\npub fn new_layer() int;\n",
        )
        .unwrap();
        assert!(session
            .resolve_uses("use m: *")
            .unwrap_err()
            .to_string()
            .contains("session_target_mismatch"));
        let mut rebuilt = session.clone();
        rebuilt.resolve_uses("use m: *").unwrap();
        let store = rebuilt.type_store();
        let store = store.read().unwrap();
        let names = store.lookup_module("m").unwrap().store.pub_fn_names();
        assert!(names.contains(&"new_layer".into()));
        assert!(!names.contains(&"old_layer".into()));
    }

    #[test]
    fn renamed_layer_changes_fingerprint() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("m.at"), "pub fn f() int;\n").unwrap();
        std::fs::write(dir.path().join("m.vm.at"), "#[vm]\npub fn f() int;\n").unwrap();
        let before =
            super::super::loader::stdlib_assembly_fingerprint(dir.path(), AssemblyTarget::Vm)
                .unwrap();
        std::fs::rename(dir.path().join("m.vm.at"), dir.path().join("m.rs.at")).unwrap();
        assert_ne!(
            before,
            super::super::loader::stdlib_assembly_fingerprint(dir.path(), AssemblyTarget::Vm)
                .unwrap()
        );
    }

    #[test]
    fn unreadable_text_remains_a_layer_and_refuses_receipt() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("bad.at"), [0xff, 0xfe]).unwrap();
        let inventory = super::super::loader::scan_inventory(dir.path());
        assert_eq!(inventory.files_total, 1);
        assert_eq!(inventory.module("bad").unwrap().layers.len(), 1);
        assert!(matches!(
            inventory.module("bad").unwrap().layers[0].parse,
            ParseStatus::Failed { .. }
        ));
        assert!(
            super::super::loader::stdlib_assembly_fingerprint(dir.path(), AssemblyTarget::Vm)
                .is_err()
        );
    }

    #[test]
    fn cloned_cache_resolves_new_project_before_lookup() {
        struct RestoreCwd(PathBuf);
        impl Drop for RestoreCwd {
            fn drop(&mut self) {
                std::env::set_current_dir(&self.0).unwrap();
            }
        }
        let _restore = RestoreCwd(std::env::current_dir().unwrap());
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        std::fs::write(
            first.path().join("m.at"),
            "pub fn from_first() int { return 1 }\n",
        )
        .unwrap();
        std::fs::write(
            second.path().join("m.at"),
            "pub fn from_second() int { return 2 }\n",
        )
        .unwrap();
        std::env::set_current_dir(first.path()).unwrap();
        let mut session = CompileSession::new();
        session.resolve_uses("use m: *").unwrap();
        let mut clone = session.clone();
        std::env::set_current_dir(second.path()).unwrap();
        clone.resolve_uses("use m: *").unwrap();
        let selected = Path::new(&clone.layer_selections[0].public_file)
            .canonicalize()
            .unwrap();
        assert_eq!(selected, second.path().join("m.at").canonicalize().unwrap());
        let store = clone.type_store();
        let store = store.read().unwrap();
        let names = store.lookup_module("m").unwrap().store.pub_fn_names();
        assert!(names.contains(&"from_second".into()));
        assert!(!names.contains(&"from_first".into()));
    }

    #[test]
    fn transitive_leaf_change_invalidates_grandparent_cache() {
        use crate::module_cache::{AutoCache, ModuleCache, SourceSegment};
        let dir = tempfile::tempdir().unwrap();
        let context = AssemblyContext::default();
        let schema = super::super::providers::catalog_schema_version();
        let mut cache = AutoCache::new();
        let mut prior = None;
        for name in ["leaf", "mid", "top"] {
            let file = dir.path().join(format!("{name}.at"));
            std::fs::write(&file, "pub fn f() int;\n").unwrap();
            let entry = ModuleCache::with_assembly(
                name,
                crate::types::TypeStore::new(),
                context,
                vec![SourceSegment::present(
                    file.to_string_lossy(),
                    "pub fn f() int;\n",
                )],
                Vec::new(),
                schema,
                prior.clone().into_iter().collect(),
            );
            prior = Some((name.into(), entry.combined_fingerprint()));
            cache.store_assembled(entry);
        }
        assert!(cache.get_valid("top", &context, schema).is_some());
        std::fs::write(dir.path().join("leaf.at"), "pub fn changed() int;\n").unwrap();
        assert!(cache.get_valid("top", &context, schema).is_none());
    }

    #[test]
    fn bound_name_with_return_category_is_not_signature_proof() {
        use super::super::model::{ModuleInventory, StdlibInventory};
        let layer = super::super::loader::parse_layer(
            "#[vm]\npub fn mystery(value int) int;\n",
            LayerKind::Vm,
            "stdlib/auto/net.vm.at".into(),
        );
        let inventory = StdlibInventory {
            schema_version: 2,
            root: "stdlib/auto".into(),
            files_total: 1,
            modules: vec![ModuleInventory {
                module: "net".into(),
                layers: vec![layer],
            }],
            diagnostics: Vec::new(),
        };
        let mut registry = crate::vm::native_registry::AutoVMNativeRegistry::new();
        registry.register_with_id_and_type(
            "auto.net.mystery",
            7777,
            crate::vm::native_registry::NativeRetType::Int,
        );
        let mut shims = crate::vm::native::NativeInterface::new();
        shims.register_static(7777, |_, _| Ok(()));
        let results = super::super::validate::validate_core_vm_bindings(
            &inventory,
            &registry,
            &shims,
            Default::default(),
        );
        assert_eq!(
            results[0].verification,
            super::super::model::VerificationLevel::Bound
        );
        assert_eq!(
            results[0].status,
            super::super::validate::CoreSymbolStatus::Unverified
        );
    }

    #[test]
    fn source_error_keeps_layer_relative_label() {
        let dir = tempfile::tempdir().unwrap();
        let public = "pub fn f() int;\n";
        let target = "pub fn broken(\n";
        std::fs::write(dir.path().join("m.at"), public).unwrap();
        std::fs::write(dir.path().join("m.vm.at"), target).unwrap();
        let plan =
            AssemblyPlan::from_resolved("m", &dir.path().join("m.at"), Default::default()).unwrap();
        let error = AutoError::MsgWithSource(MsgWithSource {
            message: "probe".into(),
            source: miette::NamedSource::new("combined", plan.source.clone()),
            span: (plan.boundary.unwrap() + 7, 3).into(),
            help: None,
        });
        let mapped = plan.map_error(error);
        assert_eq!(mapped.labels().unwrap().next().unwrap().offset(), 7);
        assert!(crate::error::format_error(&mapped).contains("m.vm.at"));
    }

    #[test]
    fn producer_arity_and_active_callee_are_independent_evidence() {
        use super::super::model::{ModuleInventory, StdlibInventory, VerificationLevel};
        let layer = super::super::loader::parse_layer(
            "pub fn tcp_bind(addr str) TcpListener?;\ntype TcpListener\n",
            LayerKind::Public,
            "stdlib/auto/net.at".into(),
        );
        let mut inventory = StdlibInventory {
            schema_version: 2,
            root: "stdlib/auto".into(),
            files_total: 1,
            modules: vec![ModuleInventory {
                module: "net".into(),
                layers: vec![layer],
            }],
            diagnostics: Vec::new(),
        };
        let mut registry = crate::vm::native_registry::AutoVMNativeRegistry::new();
        registry.register_with_id_and_type(
            "auto.net.tcp_bind",
            7777,
            crate::vm::native_registry::NativeRetType::Int,
        );
        let mut shims = crate::vm::native::NativeInterface::new();
        let id = shims.register_typed_shim_by_name(
            "auto.net.tcp_bind",
            |_, _| Ok(()),
            &["str"],
            "TcpListener?",
        );
        registry.register_with_id_and_type(
            "auto.net.tcp_bind",
            id,
            crate::vm::native_registry::NativeRetType::Int,
        );
        let validate = |inventory: &StdlibInventory, shims: &crate::vm::native::NativeInterface| {
            super::super::validate::validate_core_vm_bindings(
                inventory,
                &registry,
                shims,
                Default::default(),
            )
        };
        assert_eq!(
            validate(&inventory, &shims)[0].verification,
            VerificationLevel::SignatureChecked
        );
        let symbol = inventory.modules[0].layers[0]
            .symbols
            .iter_mut()
            .find(|s| s.name == "tcp_bind")
            .unwrap();
        symbol.arity = 999;
        let result = validate(&inventory, &shims);
        assert_eq!(result[0].verification, VerificationLevel::Bound);
        assert!(result[0].reason.as_ref().unwrap().contains("arity drift"));
        inventory.modules[0].layers[0]
            .symbols
            .iter_mut()
            .find(|s| s.name == "tcp_bind")
            .unwrap()
            .arity = 1;
        shims.register_static(id, |_, _| Ok(()));
        assert_eq!(
            validate(&inventory, &shims)[0].verification,
            VerificationLevel::Bound
        );
    }
}
