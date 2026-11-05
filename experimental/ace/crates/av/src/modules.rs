//! Finite source-module consumer: existing unit lowering, ABC envelope,
//! FunctionCode admission and ProgramSnapshot linking, with source/query
//! preparation owned by ace-source-query. No source parser lives here.
use crate::{
    cache::{
        abc_unit_key, caps_fingerprint, envelope_unwrap, put_enveloped, split_units,
        EnvelopeIdentity,
    },
    wire::WireCodec,
};
use ace_artifacts::{
    codec::ChunkKind,
    identity::{QueryKey, RefKind},
    store::{ManagedStore, PutRequest},
};
use ace_av_core::{
    contract::{ExecOutcome, Value},
    function_code::FunctionCode,
    program_snapshot::ProgramSnapshot,
    span_map::SpanSegments,
    CancelFlag, ExecLimits,
};
use ace_core::source::modules::BoundModules;
use ace_source_query::modules::{ModuleProduct, ModuleReceipt, PreparedModuleEdit};
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};
const SCHEMA: &str = "av-source-modules/1";
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Unit {
    name: String,
    artifact: String,
    body_fp: String,
    mint: (usize, usize),
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Root {
    schema: String,
    source: String,
    caps: Vec<String>,
    entry: String,
    owner: String,
    units: Vec<Unit>,
}
pub struct ModuleAvTarget {
    store: ManagedStore,
    current: Arc<ProgramSnapshot>,
    root: Root,
    pub lowered: u64,
}
pub struct ModuleAvEdit {
    base: String,
    snapshot: Arc<ProgramSnapshot>,
    root: Root,
}
fn span(bound: &BoundModules, name: &str) -> Result<(usize, usize), String> {
    bound
        .program
        .items
        .iter()
        .find_map(|i| match i {
            ace_core::source::ast::Item::Fn(f) if format!("d_{}", f.name.name) == name => {
                Some((f.span.start, f.span.end))
            }
            _ => None,
        })
        .ok_or_else(|| format!("module AV owner `{name}` absent"))
}
fn link(
    root: &Root,
    bound: &BoundModules,
    codes: Vec<Arc<FunctionCode>>,
    revision: u64,
) -> Result<Arc<ProgramSnapshot>, String> {
    let maps = root
        .units
        .iter()
        .map(|u| {
            span(bound, &u.name)
                .map(|s| SpanSegments::identity_range(u.mint.0 as u64, u.mint.1 as u64, s.0 as u64))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Arc::new(ProgramSnapshot::link_with_maps(
        codes,
        &root.entry,
        revision,
        maps,
    )?))
}
fn persist_unit(
    store: &ManagedStore,
    unit: &ace_av_core::mir::MirModule,
    mint: (usize, usize),
    caps: &[String],
    revision: u64,
) -> Result<(Unit, Arc<FunctionCode>), String> {
    let wire = WireCodec::LegacyAbc;
    let bytes = wire.encode(unit).map_err(|e| format!("{e:?}"))?;
    let body_fp = format!(
        "module-unit-wire/1:{}",
        ace_core::runtime_contract::sha256_hex(&bytes)
    );
    let name = unit.funcs[0].name.clone();
    let key = abc_unit_key(&unit.owner.module, &name, &body_fp, &caps_fingerprint(caps));
    let envelope = EnvelopeIdentity::abc_bound(&unit.owner.module, &name, &body_fp);
    let artifact = put_enveloped(store, &key, "A09", &envelope, &bytes, Vec::new(), None)
        .map_err(|e| e.to_string())?;
    let loaded = wire
        .admit_bytes(&bytes, &unit.owner)
        .map_err(|e| format!("{e:?}"))?;
    let code = Arc::new(FunctionCode::admit_unit(loaded, revision)?);
    Ok((
        Unit {
            name,
            artifact,
            body_fp,
            mint,
        },
        code,
    ))
}
impl ModuleAvTarget {
    pub fn cold(path: &Path, product: &ModuleProduct) -> Result<Self, String> {
        let store = ManagedStore::open(path).map_err(|e| e.to_string())?;
        let mir = ace_lowering::lower_checked(&product.compiled.checked, "d_main", 0)
            .map_err(|e| format!("{e:?}"))?;
        let mut root = Root {
            schema: SCHEMA.into(),
            source: product.bound.input_hash.clone(),
            caps: product.bound.capabilities.clone(),
            entry: "d_main".into(),
            owner: mir.owner.module.clone(),
            units: Vec::new(),
        };
        let mut codes = Vec::new();
        for unit in split_units(&mir)? {
            let mint = span(&product.bound, &unit.funcs[0].name)?;
            let (record, code) = persist_unit(&store, &unit, mint, &root.caps, 0)?;
            root.units.push(record);
            codes.push(code);
        }
        let lowered = codes.len() as u64;
        let current = link(&root, &product.bound, codes, 0)?;
        Ok(Self {
            store,
            current,
            root,
            lowered,
        })
    }
    pub fn current(&self) -> &Arc<ProgramSnapshot> {
        &self.current
    }
    pub fn run(&self) -> Result<i32, String> {
        let (out, _) = ace_av_core::execute_program(
            &self.current,
            &ExecLimits::default(),
            &CancelFlag::new(false),
        );
        match out {
            ExecOutcome::Success {
                value: Value::I32(n),
                ..
            } => Ok(n),
            other => Err(format!("{other:?}")),
        }
    }
    pub fn prepare(&mut self, edit: &PreparedModuleEdit) -> Result<ModuleAvEdit, String> {
        if self.root.source != edit.base_source() {
            return Err("module AV source base mismatch".into());
        }
        if self.root.caps != edit.bound.capabilities {
            return Err("AV module capabilities changed".into());
        }
        let def = format!("d_{}", edit.owner);
        let parts = edit.candidate.parts();
        let index = parts.bundle.modules[0]
            .defs
            .iter()
            .position(|d| d.id_text == def)
            .ok_or("AV module candidate def absent")?;
        ace_core::verify::verify_function_ref(&parts.bundle, index)
            .map_err(|e| format!("{e:?}"))?;
        self.lowered += 1;
        let unit = ace_lowering::lower_function_unit(
            &parts.bundle,
            &def,
            &parts.bundle.requires,
            self.current.revision() + 1,
        )
        .map_err(|e| format!("{e:?}"))?;
        let (record, code) = persist_unit(
            &self.store,
            &unit,
            span(&edit.bound, &def)?,
            &self.root.caps,
            self.current.revision() + 1,
        )?;
        let mut root = Root {
            schema: SCHEMA.into(),
            source: edit.bound.input_hash.clone(),
            caps: self.root.caps.clone(),
            entry: self.root.entry.clone(),
            owner: self.root.owner.clone(),
            units: self.root.units.clone(),
        };
        let slot = root
            .units
            .iter()
            .position(|u| u.name == def)
            .ok_or("AV module candidate owner absent")?;
        root.units[slot] = record;
        let mut codes = self.current.code().to_vec();
        codes[slot] = code;
        let snapshot = link(&root, &edit.bound, codes, self.current.revision() + 1)?;
        Ok(ModuleAvEdit {
            base: self.root.source.clone(),
            snapshot,
            root,
        })
    }
    pub fn validate(&self, edit: &ModuleAvEdit) -> Result<(), String> {
        if edit.base != self.root.source {
            return Err("stale AV module edit".into());
        }
        Ok(())
    }
    pub fn commit(&mut self, edit: ModuleAvEdit) -> Result<(), String> {
        self.validate(&edit)?;
        self.current = edit.snapshot;
        self.root = edit.root;
        Ok(())
    }
    pub fn receipt(&self) -> Result<ModuleReceipt, String> {
        write_root(&self.store, &self.root)
    }
    pub fn prepared_receipt(&self, edit: &ModuleAvEdit) -> Result<ModuleReceipt, String> {
        if edit.base != self.root.source {
            return Err("stale AV module receipt".into());
        }
        write_root(&self.store, &edit.root)
    }
    pub fn restore(
        path: &Path,
        receipt: &ModuleReceipt,
        bound: &BoundModules,
    ) -> Result<Self, String> {
        let store = ManagedStore::open(path).map_err(|e| e.to_string())?;
        let a = store
            .get_artifact_by_id_bounded(&receipt.artifact, 512 * 1024)
            .map_err(|e| e.to_string())?
            .ok_or("AV module root absent")?;
        if a.kind != ChunkKind::SnapshotRecord
            || ace_core::runtime_contract::sha256_hex(&a.payload) != receipt.digest
        {
            return Err("AV module root receipt mismatch".into());
        }
        let root: Root = serde_json::from_slice(&a.payload).map_err(|e| e.to_string())?;
        if root.schema != SCHEMA
            || root.source != bound.input_hash
            || root.caps != bound.capabilities
            || root.entry != "d_main"
        {
            return Err("AV module source/schema/capability mismatch".into());
        }
        let required: std::collections::BTreeSet<_> =
            root.units.iter().map(|u| u.artifact.clone()).collect();
        if store
            .required_targets(&receipt.artifact)
            .map_err(|e| e.to_string())?
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            != required
        {
            return Err("AV module required edges mismatch".into());
        }
        let mut codes = Vec::new();
        let mut names = std::collections::BTreeSet::new();
        for u in &root.units {
            if !names.insert(u.name.clone()) {
                return Err("duplicate AV module unit".into());
            }
            let a = store
                .get_artifact_by_id_bounded(&u.artifact, 8 * 1024 * 1024)
                .map_err(|e| e.to_string())?
                .ok_or("AV module unit absent")?;
            if a.kind != ChunkKind::LayoutData {
                return Err("AV module unit carrier mismatch".into());
            }
            let bytes = envelope_unwrap(
                &a.payload,
                &EnvelopeIdentity::abc_bound(&root.owner, &u.name, &u.body_fp),
            )
            .map_err(|e| e.to_string())?;
            if format!(
                "module-unit-wire/1:{}",
                ace_core::runtime_contract::sha256_hex(&bytes)
            ) != u.body_fp
            {
                return Err("AV module unit digest mismatch".into());
            }
            let expected = ace_av_core::mir::OwnerScope {
                schema: ace_av_core::MIR_SCHEMA.into(),
                module: root.owner.clone(),
            };
            let loaded = WireCodec::LegacyAbc
                .admit_bytes(&bytes, &expected)
                .map_err(|e| format!("{e:?}"))?;
            let code = Arc::new(FunctionCode::admit_unit(loaded, 0)?);
            if code.name() != u.name {
                return Err("AV module unit name mismatch".into());
            }
            codes.push(code);
        }
        if names.len() != bound.origins.len() {
            return Err("AV module owner set mismatch".into());
        }
        let current = link(&root, bound, codes, 0)?;
        Ok(Self {
            store,
            current,
            root,
            lowered: 0,
        })
    }
}
fn write_root(store: &ManagedStore, root: &Root) -> Result<ModuleReceipt, String> {
    let payload = serde_json::to_vec(root).map_err(|e| e.to_string())?;
    let digest = ace_core::runtime_contract::sha256_hex(&payload);
    let artifact = store
        .put_artifact(PutRequest {
            layer: "AVM950",
            key: QueryKey {
                layer: "AVM950",
                components: vec![
                    ("schema".into(), SCHEMA.into()),
                    ("digest".into(), digest.clone()),
                ],
            },
            kind: ChunkKind::SnapshotRecord,
            payload,
            edges: root
                .units
                .iter()
                .map(|u| (u.artifact.clone(), RefKind::RequiredData))
                .collect(),
            project: None,
        })
        .map_err(|e| e.to_string())?;
    Ok(ModuleReceipt { artifact, digest })
}
