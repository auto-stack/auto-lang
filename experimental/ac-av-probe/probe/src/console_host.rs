//! PLAN-863: bounded console provider and portable, re-admitted whole-module artifacts.
//! This is a probe seam only. Product CLI/SDK defaults do not call it.
use std::path::Path;

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::schema::{LayerCoverage, LayerSpec, Regeneration};
use ac_artifact_probe::store::{ManagedStore, PutRequest};
use auto_av_core::av::{execute_console, CancelCheck, CancelFlag, ExecLimits, ExecTrace};
use auto_av_core::contract::*;
use auto_av_core::host::{self, ConsoleHost, HostError, Stream};
use auto_av_core::mir::OwnerScope;
use auto_av_core::{verify_abc_bytes, verify_mir, LoadedModule};
use serde::{Deserialize, Serialize};

use crate::cache::{envelope_unwrap, envelope_wrap, EnvelopeIdentity, EnvelopeKind};
use crate::driver::StageDiag;

#[derive(Debug, Default)]
pub struct ConsoleCapture {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub dispatches: u64,
}

impl ConsoleHost for ConsoleCapture {
    fn retained_bytes(&self) -> usize {
        self.stdout.len().saturating_add(self.stderr.len())
    }
    fn write(
        &mut self,
        stream: Stream,
        bytes: &[u8],
        cancel: &CancelFlag,
    ) -> Result<(), HostError> {
        self.dispatches += 1;
        if CancelCheck::poll(cancel) {
            return Err(HostError::Cancelled);
        }
        let total = self
            .retained_bytes()
            .checked_add(bytes.len())
            .ok_or(HostError::Resource)?;
        if bytes.len() > host::MAX_CALL_BYTES || total > host::MAX_RUN_BYTES {
            return Err(HostError::Resource);
        }
        let out = match stream {
            Stream::Stdout => &mut self.stdout,
            Stream::Stderr => &mut self.stderr,
        };
        out.try_reserve_exact(bytes.len())
            .map_err(|_| HostError::Resource)?;
        // No cancellation or fallible operation between this poll and the atomic commit.
        if CancelCheck::poll(cancel) {
            return Err(HostError::Cancelled);
        }
        out.extend_from_slice(bytes);
        Ok(())
    }
}

pub const CONSOLE_ABC_LAYER: LayerSpec = LayerSpec {
    id: "A863",
    name: "host-console whole-module ABC",
    schema_id: "ac863/host-console-abc/v1",
    producer: HOST_CONSOLE_PRODUCER_ID,
    consumers: &["AV console probe (restore always re-admits)"],
    key_inputs: &[
        "producer",
        "owner",
        "entry",
        "profile",
        "wire-fp",
        "requirements-fp",
        "snapshot",
        "mir-sem",
        "abc-sem",
        "av-sem",
    ],
    ref_kinds: &["provenance"],
    storage: "whole-module AVHC/1, declarations and requirements bound in envelope/key",
    residency: "no native pointer, provider or grant credential is restored",
    regeneration: Regeneration::FromUserInputs,
    coverage: LayerCoverage::Codec,
};

/// These are expected identity facts supplied by the host, never an execution credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleArtifactRef {
    pub owner: String,
    pub entry: String,
    pub snapshot: u64,
    pub wire_fp: String,
    pub requirements_fp: String,
}

pub struct ConsoleProgram {
    pub reference: ConsoleArtifactRef,
    wire: Vec<u8>,
}

fn diag(stage: &str, code: &str, message: impl ToString) -> StageDiag {
    StageDiag {
        stage: stage.into(),
        code: code.into(),
        message: message.to_string(),
    }
}

fn requirements_fp(m: &auto_av_core::mir::MirModule) -> String {
    let mut fields: Vec<String> = m
        .host_imports
        .iter()
        .map(|h| {
            // The complete stable symbol fixes the exact input/result signature;
            // structural admission has already checked it against the type table.
            format!(
                "{}|{}|{}|{}",
                h.category, h.symbol, h.capability, h.max_call_bytes
            )
        })
        .collect();
    fields.sort();
    fields.extend(
        m.capabilities
            .iter()
            .cloned()
            .map(|c| format!("requires:{c}")),
    );
    format!("{:016x}", fnv1a64(fields.join("\0").as_bytes()))
}

fn envelope(reference: &ConsoleArtifactRef) -> EnvelopeIdentity<'_> {
    EnvelopeIdentity {
        kind: EnvelopeKind::AbcUnit,
        producer: HOST_CONSOLE_PRODUCER_ID,
        schema: ABC_HOST_CONSOLE_SCHEMA,
        owner_module: &reference.owner,
        stage: 2,
        mir_sem: MIR_HOST_CONSOLE_SEMANTICS.0,
        abc_sem: ABC_HOST_CONSOLE_SEMANTICS.0,
        av_sem: AV_HOST_CONSOLE_SEMANTICS.0,
        snapshot: reference.snapshot,
        unit: &reference.entry,
        body_fp: &reference.requirements_fp,
    }
}

fn key(reference: &ConsoleArtifactRef) -> QueryKey {
    QueryKey::new(
        &CONSOLE_ABC_LAYER,
        vec![
            ("producer".into(), HOST_CONSOLE_PRODUCER_ID.into()),
            ("owner".into(), reference.owner.clone()),
            ("entry".into(), reference.entry.clone()),
            ("profile".into(), AV_HOST_CONSOLE_PROFILE.into()),
            ("wire-fp".into(), reference.wire_fp.clone()),
            ("requirements-fp".into(), reference.requirements_fp.clone()),
            ("snapshot".into(), reference.snapshot.to_string()),
            ("mir-sem".into(), MIR_HOST_CONSOLE_SEMANTICS.0.to_string()),
            ("abc-sem".into(), ABC_HOST_CONSOLE_SEMANTICS.0.to_string()),
            ("av-sem".into(), AV_HOST_CONSOLE_SEMANTICS.0.to_string()),
        ],
    )
}

pub fn compile(src: &str, entry: &str) -> Result<ConsoleProgram, StageDiag> {
    crate::driver::front_door_admit(src)?;
    let compiled = auto_ac_prototype::source::compile_host_console_source(src, "console.at", &[])
        .map_err(|ds| {
        let d = &ds[0];
        diag(d.stage.as_str(), &d.code, &d.message)
    })?;
    let mir =
        auto_mir_lower::lower_checked_console(&compiled.checked, entry, fnv1a64(src.as_bytes()))
            .map_err(|e| diag("lower", &e.code(), format!("{e:?}")))?;
    verify_mir(&mir).map_err(|e| diag("verify", &e.code(), format!("{e:?}")))?;
    let wire = auto_av_core::encode_abc(&mir)
        .map_err(|e| diag("encode", &e.code(), format!("{e:?}")))?
        .to_flat();
    let reference = ConsoleArtifactRef {
        owner: mir.owner.module.clone(),
        entry: entry.into(),
        snapshot: fnv1a64(src.as_bytes()),
        wire_fp: format!("{:016x}", fnv1a64(&wire)),
        requirements_fp: requirements_fp(&mir),
    };
    Ok(ConsoleProgram { reference, wire })
}

impl ConsoleProgram {
    pub fn wire(&self) -> &[u8] {
        &self.wire
    }

    pub fn load(&self) -> Result<LoadedModule, StageDiag> {
        if self.reference.owner.len() > u16::MAX as usize
            || self.reference.entry.len() > u16::MAX as usize
        {
            return Err(diag(
                "admission",
                "host.identity",
                "console envelope identity exceeds its wire width",
            ));
        }
        if format!("{:016x}", fnv1a64(&self.wire)) != self.reference.wire_fp {
            return Err(diag(
                "admission",
                "host.identity",
                "console wire fingerprint mismatch",
            ));
        }
        let loaded = verify_abc_bytes(
            &self.wire,
            &OwnerScope {
                schema: MIR_HOST_CONSOLE_SCHEMA.into(),
                module: self.reference.owner.clone(),
            },
        )
        .map_err(|e| diag("admission", "host.artifact", e))?;
        if !loaded.module().is_host_console_profile()
            || requirements_fp(loaded.module()) != self.reference.requirements_fp
            || loaded
                .module()
                .funcs
                .get(loaded.module().entry as usize)
                .map(|f| f.name.as_str())
                != Some(self.reference.entry.as_str())
        {
            return Err(diag(
                "admission",
                "host.identity",
                "console profile/requirements/entry mismatch",
            ));
        }
        Ok(loaded)
    }

    pub fn run(
        &self,
        grants: &[String],
        limits: &ExecLimits,
        cancel: &CancelFlag,
        host: &mut dyn ConsoleHost,
    ) -> Result<(ExecOutcome, ExecTrace), StageDiag> {
        let loaded = self.load()?;
        execute_console(&loaded, limits, cancel, grants, host)
            .map_err(|e| diag("admission", e.code, e.message))
    }

    pub fn persist(&self, root: &Path, project: &str) -> Result<String, StageDiag> {
        self.load()?;
        let store = ManagedStore::open(root).map_err(|e| diag("infra", "host.store", e))?;
        store
            .put_artifact(PutRequest {
                layer: CONSOLE_ABC_LAYER.id,
                key: key(&self.reference),
                kind: ChunkKind::LayoutData,
                payload: envelope_wrap(&self.wire, &envelope(&self.reference)),
                edges: vec![],
                project: Some(project.into()),
            })
            .map_err(|e| diag("infra", "host.store", e))
    }
}

pub fn restore(root: &Path, reference: &ConsoleArtifactRef) -> Result<ConsoleProgram, StageDiag> {
    let store = ManagedStore::open(root).map_err(|e| diag("infra", "host.store", e))?;
    let artifact = store
        .get_artifact(CONSOLE_ABC_LAYER.id, &key(reference))
        .map_err(|e| diag("infra", "host.store", e))?
        .ok_or_else(|| diag("admission", "host.artifact-miss", "console artifact absent"))?;
    if artifact.kind != ChunkKind::LayoutData {
        return Err(diag("admission", "host.artifact", "wrong artifact kind"));
    }
    let wire = envelope_unwrap(&artifact.payload, &envelope(reference))
        .map_err(|e| diag("admission", "host.envelope", e))?;
    let program = ConsoleProgram {
        reference: reference.clone(),
        wire,
    };
    program.load()?;
    Ok(program)
}
