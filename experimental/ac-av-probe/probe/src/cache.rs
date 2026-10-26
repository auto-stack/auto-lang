//! Managed cache units for the ac-av probe (PLAN-798 §5.4).
//!
//! Reuses the PLAN-783 `ManagedStore` through its public API ONLY, as an
//! isolated producer namespace: this plan's OWN `LayerSpec`s describe the
//! A08 (MIR) / A09 (ABC + manifest) seams it actually implements, payloads
//! ride the generic `ChunkKind::LayoutData` carrier (never tagged Hir or
//! Object), and every write/read validates the inner envelope's producer,
//! schema, owner and stage. Nothing here modifies 783's registry, ChunkKind
//! set or GC internals.
//!
//! Unit model: MIR/ABC persist PER FUNCTION; a module link manifest maps
//! unit names to artifact ids. Callee bodies are addressed through unit
//! imports (`fn:<name>`), so a body edit re-keys only its own unit while
//! callers' code units stay reusable; a SIGNATURE edit changes every
//! caller's fingerprint (the caller's canonical HIR dump embeds callee
//! signature content). Cache keys always include owner/profile, body+sig
//! content, algorithm/schema and MIR/ABC/AV semantic versions plus the
//! capability-affecting configuration; the source-map sidecar is keyed
//! separately (position changes share code content, sidecar re-binds).
//!
//! Restore is never trusted: every restored payload re-decodes, re-enters
//! the real verifiers, and re-checks the envelope against the caller's
//! expected identity. A hit still executes AV.

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::schema::{LayerCoverage, LayerSpec, Regeneration};
use ac_artifact_probe::store::{LoadedArtifact, ManagedStore, PutRequest, StoreError};
use auto_av_core::contract::{
    fnv1a64_parts, ABC_SCHEMA, ABC_SEMANTICS, AV_PROFILE, AV_SEMANTICS, MIR_SCHEMA, MIR_SEMANTICS,
    PRODUCER_ID,
};
use auto_av_core::mir::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

// ---------------------------------------------------------------------------
// This plan's layer seams (independent of 783's registry)
// ---------------------------------------------------------------------------

pub const MIR_LAYER: LayerSpec = LayerSpec {
    id: "A08",
    name: "semantic-lowering-passes (plan798 MIR units)",
    schema_id: "ac798/a08-mir/v1",
    producer: PRODUCER_ID,
    consumers: &["av probe ABC emission", "diagnostics"],
    key_inputs: &[
        "producer",
        "owner-module",
        "unit",
        "profile",
        "body-sig-content",
        "callee-sigs",
        "mir-schema",
        "mir-semantics",
        "av-semantics",
        // F798-R12-03: MIR units ride the ABC wire codec — the codec
        // generation is a key axis so kind-less /1 payloads miss.
        "abc-codec",
        "capabilities-config",
    ],
    ref_kinds: &["provenance"],
    storage: "one binary unit per function inside the shared managed store (LayoutData carrier)",
    residency: "per-function units; module manifest references units by artifact id",
    regeneration: Regeneration::FromUserInputs,
    coverage: LayerCoverage::Codec,
};

pub const ABC_LAYER: LayerSpec = LayerSpec {
    id: "A09",
    name: "backend-object (plan798 ABC units + module manifest)",
    schema_id: "ac798/a09-abc/v2",
    producer: PRODUCER_ID,
    consumers: &["av executor closure assembly"],
    key_inputs: &[
        "producer",
        "owner-module",
        "unit",
        "profile",
        "body-sig-content",
        "callee-sigs",
        "abc-codec",
        "abc-semantics",
        "av-semantics",
        "capabilities-config",
    ],
    ref_kinds: &["required-data", "provenance"],
    storage: "one ABC unit per function + module link manifest (LayoutData carrier)",
    residency: "restored units re-verify before execution; never trusted flags",
    regeneration: Regeneration::FromUpstreamRecipe,
    coverage: LayerCoverage::Codec,
};

// ---------------------------------------------------------------------------
// Source sidecar codec + merge
// ---------------------------------------------------------------------------

/// Encode one function's span table: count, then (instr, func, start, end).
pub fn encode_sidecar(spans: &[(u32, auto_av_core::contract::SpanInfo)]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + spans.len() * 16);
    out.extend_from_slice(&(spans.len() as u32).to_le_bytes());
    for (instr, sp) in spans {
        out.extend_from_slice(&instr.to_le_bytes());
        out.extend_from_slice(&sp.func.to_le_bytes());
        out.extend_from_slice(&sp.start.to_le_bytes());
        out.extend_from_slice(&sp.end.to_le_bytes());
    }
    out
}

pub fn decode_sidecar(
    bytes: &[u8],
) -> Result<Vec<(u32, auto_av_core::contract::SpanInfo)>, CacheError> {
    if bytes.len() < 4 {
        return Err(CacheError::Codec("sidecar truncated".into()));
    }
    let n = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    if bytes.len() != 4 + n * 16 {
        return Err(CacheError::Codec(format!(
            "sidecar length {} does not match {} entries",
            bytes.len(),
            n
        )));
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let b = &bytes[4 + i * 16..4 + (i + 1) * 16];
        out.push((
            u32::from_le_bytes(b[0..4].try_into().unwrap()),
            auto_av_core::contract::SpanInfo {
                func: u32::from_le_bytes(b[4..8].try_into().unwrap()),
                start: u32::from_le_bytes(b[8..12].try_into().unwrap()),
                end: u32::from_le_bytes(b[12..16].try_into().unwrap()),
            },
        ));
    }
    Ok(out)
}

/// Merge a decoded sidecar into a span-less unit (diagnostics track only).
pub fn merge_spans(unit: &mut MirModule, spans: &[(u32, auto_av_core::contract::SpanInfo)]) {
    if let Some(f) = unit.funcs.first_mut() {
        f.spans = spans.to_vec();
    }
}

pub fn sidecar_key(owner: &str, unit: &str, snapshot: u64) -> QueryKey {
    QueryKey::new(
        &MIR_LAYER,
        vec![
            ("producer".into(), PRODUCER_ID.into()),
            ("owner".into(), owner.into()),
            ("kind".into(), "source-sidecar".into()),
            ("unit".into(), unit.into()),
            ("snapshot".into(), format!("{:016x}", snapshot)),
        ],
    )
}

/// PLAN-828 F-10 unit-span sidecar key: the MINT span table of one
/// persisted ABC unit. Keyed on the unit's own content axes (owner,
/// unit, body fingerprint) so it travels with the unit row — unlike the
/// 798 [`sidecar_key`], which re-binds per source snapshot.
pub fn unit_span_sidecar_key(owner: &str, unit: &str, body_fp: &str) -> QueryKey {
    QueryKey::new(
        &ABC_LAYER,
        vec![
            ("producer".into(), PRODUCER_ID.into()),
            ("owner".into(), owner.into()),
            ("kind".into(), "unit-span-sidecar".into()),
            ("unit".into(), unit.into()),
            ("body-sig".into(), body_fp.into()),
            ("sidecar-format".into(), "1".into()),
        ],
    )
}

/// PLAN-828 F10-S02 repair row: when the primary sidecar key is held by
/// an INVALID survivor (the store has no per-artifact rewrite), the
/// re-derived payload publishes under this NEW identity and restore
/// probes primary → repair ("publish a new identity and switch the
/// restore reference to a valid artifact").
pub fn unit_span_sidecar_repair_key(owner: &str, unit: &str, body_fp: &str) -> QueryKey {
    QueryKey::new(
        &ABC_LAYER,
        vec![
            ("producer".into(), PRODUCER_ID.into()),
            ("owner".into(), owner.into()),
            ("kind".into(), "unit-span-sidecar-repair".into()),
            ("unit".into(), unit.into()),
            ("body-sig".into(), body_fp.into()),
            ("sidecar-format".into(), "1".into()),
        ],
    )
}

/// One decoded unit-span sidecar payload. Generation 2 carries the
/// unit's mint declaration range and text (PLAN-828 F11-S01: a restore
/// re-anchors the mint spans to the CURRENT source through
/// `SpanSegments::from_texts`); the legacy spans-only generation leaves
/// `mint` empty and is taken as CURRENT-source coordinates.
#[derive(Clone, Debug)]
pub struct UnitSidecar {
    pub spans: Vec<(u32, auto_av_core::contract::SpanInfo)>,
    pub mint: Option<(u64, u64, Vec<u8>)>,
}

pub fn encode_unit_sidecar(sc: &UnitSidecar) -> Vec<u8> {
    let blob = encode_sidecar(&sc.spans);
    let mut out =
        Vec::with_capacity(8 + blob.len() + 16 + sc.mint.as_ref().map_or(0, |m| m.2.len()));
    out.extend_from_slice(b"8SS2");
    out.extend_from_slice(&(blob.len() as u32).to_le_bytes());
    out.extend_from_slice(&blob);
    if let Some((d0, d1, text)) = &sc.mint {
        out.extend_from_slice(&d0.to_le_bytes());
        out.extend_from_slice(&d1.to_le_bytes());
        out.extend_from_slice(text);
    }
    out
}

pub fn decode_unit_sidecar(bytes: &[u8]) -> Result<UnitSidecar, CacheError> {
    if bytes.len() >= 8 && &bytes[0..4] == b"8SS2" {
        let blob_len = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        if bytes.len() < 8 + blob_len + 16 {
            return Err(CacheError::Codec("unit sidecar truncated".into()));
        }
        let spans = decode_sidecar(&bytes[8..8 + blob_len])?;
        let d0 = u64::from_le_bytes(bytes[8 + blob_len..8 + blob_len + 8].try_into().unwrap());
        let d1 = u64::from_le_bytes(
            bytes[8 + blob_len + 8..8 + blob_len + 16]
                .try_into()
                .unwrap(),
        );
        let text = bytes[8 + blob_len + 16..].to_vec();
        if d1 < d0 || (d1 - d0) as usize != text.len() {
            return Err(CacheError::Codec("unit sidecar mint range mismatch".into()));
        }
        return Ok(UnitSidecar {
            spans,
            mint: Some((d0, d1, text)),
        });
    }
    // Legacy spans-only generation (the 798 codec).
    Ok(UnitSidecar {
        spans: decode_sidecar(bytes)?,
        mint: None,
    })
}

/// F10-S02/F11-S01 FULL sidecar applicability — the ONE standard both
/// restore candidate selection and the persist survivor/repair
/// write-back apply. A stored unit-sidecar row serves a restore into
/// the CURRENT source only when:
/// * it decodes (either payload generation) and its merged span table
///   is APPLICABLE — coverage of every trap-capable instruction, no
///   inverted ranges (F798-R15-03);
/// * generation 2: the mint text is valid UTF-8, the declared mint
///   FRAME covers every actual span (a plausible-but-unrelated frame
///   must not silently keep old absolute coordinates), and the mint
///   text re-anchors onto the current decl slice (`from_texts`);
/// * generation 1 (spans-only legacy): NO mint basis is recorded, so
///   provenance is proven by exact EVIDENCE — the row's span table
///   must equal the current-source compile's own table (`cur.spans`),
///   never an identity assumption.
pub fn unit_sidecar_usable_for_current(
    bytes: &[u8],
    f: &MirFunction,
    cur: &MirFunction,
    cur_decl_origin: u64,
    cur_decl_text: &[u8],
) -> bool {
    let sc = match decode_unit_sidecar(bytes) {
        Ok(sc) => sc,
        Err(_) => return false,
    };
    let mut probe = f.clone();
    probe.spans = sc.spans.clone();
    if sidecar_applicable(&probe).is_err() {
        return false;
    }
    match sc.mint {
        Some((m_d0, m_d1, m_text)) => {
            if std::str::from_utf8(&m_text).is_err() {
                return false;
            }
            if !sc
                .spans
                .iter()
                .all(|(_, sp)| m_d0 <= sp.start as u64 && sp.end as u64 <= m_d1)
            {
                return false;
            }
            let Some(map) = auto_av_core::span_map::SpanSegments::from_texts(
                m_d0,
                m_d1,
                cur_decl_origin,
                &m_text,
                cur_decl_text,
            ) else {
                return false;
            };
            // F11-S01/r19a-check mapped-source evidence: every stored
            // span, re-anchored into CURRENT coordinates, must equal the
            // current-source compile's own table (same content version ⇒
            // same instruction stream). Frame membership alone is not
            // provenance — a structurally valid span naming the wrong
            // expression must not be admitted or shadow a valid repair.
            // r20a-check: the proof binds EVERY consumed field — the
            // INSTRUCTION index the executor looks the span up by (a
            // duplicated/shifted table like [1,1,2] is not the same
            // provenance) and the FUNCTION index that selects the
            // snapshot's per-code source map at run time (a foreign func
            // number would bypass that map and leak old absolute
            // coordinates), not just the byte endpoints.
            sc.spans.len() == cur.spans.len()
                && sc
                    .spans
                    .iter()
                    .zip(cur.spans.iter())
                    .all(|((i, sp), (ci, c))| {
                        i == ci && sp.func == c.func && {
                            let (s, e) = map.map_span(sp.start as u64, sp.end as u64);
                            s == c.start as u64 && e == c.end as u64
                        }
                    })
        }
        None => sc.spans == cur.spans,
    }
}

// ---------------------------------------------------------------------------
// Inner envelope
// ---------------------------------------------------------------------------

pub const ENVELOPE_MAGIC: &[u8; 4] = b"AVEN";
/// v2 (F798-R6-01): unit/body-fp identity fields joined the envelope —
/// code-content artifacts bind the exact (owner, unit, body fingerprint)
/// they claim to be, so a valid payload stored under ANOTHER body's key
/// fails identity admission instead of hitting.
pub const ENVELOPE_VERSION: u16 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnvelopeKind {
    MirUnit = 1,
    AbcUnit = 2,
    ModuleManifest = 3,
    /// Diagnostics-only source-map track: spans + snapshot per unit. The
    /// SEPARATE mapping track — code-content artifacts never carry spans, so
    /// position changes share code units and only re-bind the sidecar
    /// (PLAN-798 §5.4).
    SourceSidecar = 4,
    /// PLAN-828 T-03: function-level HIR artifacts (A07 function units) —
    /// a separate track from code-content artifacts; restored only as
    /// untrusted descriptions that re-verify before adoption.
    FnHirUnit = 5,
}

#[derive(Clone, Debug)]
pub struct EnvelopeIdentity<'a> {
    pub kind: EnvelopeKind,
    pub producer: &'a str,
    pub schema: &'a str,
    pub owner_module: &'a str,
    /// 1 = MIR stage, 2 = ABC stage (manifests live at the ABC stage).
    pub stage: u8,
    pub mir_sem: u32,
    pub abc_sem: u32,
    pub av_sem: u32,
    pub snapshot: u64,
    /// v2 unit binding (F798-R6-01): the unit name and body fingerprint
    /// this code-content artifact claims. EMPTY on the legacy constructors
    /// (self-check paths); every PRODUCTION write/read of a MirUnit or
    /// AbcUnit uses the `_bound` constructors so a good payload under a
    /// wrong body key fails admission.
    pub unit: &'a str,
    pub body_fp: &'a str,
}

impl EnvelopeIdentity<'_> {
    pub fn mir(owner: &str, snapshot: u64) -> EnvelopeIdentity<'_> {
        EnvelopeIdentity {
            kind: EnvelopeKind::MirUnit,
            producer: PRODUCER_ID,
            schema: MIR_SCHEMA,
            owner_module: owner,
            stage: 1,
            mir_sem: 1,
            abc_sem: 1,
            av_sem: 1,
            snapshot,
            unit: "",
            body_fp: "",
        }
    }

    /// Production MIR-unit identity: bound to the exact unit + body
    /// fingerprint (F798-R6-01).
    pub fn mir_bound<'a>(owner: &'a str, unit: &'a str, body_fp: &'a str) -> EnvelopeIdentity<'a> {
        EnvelopeIdentity {
            unit,
            body_fp,
            ..Self::mir(owner, 0)
        }
    }

    pub fn abc_raw(owner: &str, snapshot: u64) -> EnvelopeIdentity<'_> {
        EnvelopeIdentity {
            kind: EnvelopeKind::AbcUnit,
            producer: PRODUCER_ID,
            schema: ABC_SCHEMA,
            owner_module: owner,
            stage: 2,
            mir_sem: 1,
            abc_sem: 1,
            av_sem: 1,
            snapshot,
            unit: "",
            body_fp: "",
        }
    }

    /// Production ABC-unit identity: bound to the exact unit + body
    /// fingerprint (F798-R6-01).
    /// PLAN-828 T-03: function-HIR artifact identity, bound to the unit
    /// (def id) and the span-free content version it claims.
    pub fn fn_hir_bound<'a>(
        owner: &'a str,
        unit: &'a str,
        body_cv: &'a str,
    ) -> EnvelopeIdentity<'a> {
        EnvelopeIdentity {
            kind: EnvelopeKind::FnHirUnit,
            producer: PRODUCER_ID,
            schema: "ac828/a07-fnhir/v1",
            owner_module: owner,
            stage: 0,
            mir_sem: 1,
            abc_sem: 1,
            av_sem: 1,
            snapshot: 0,
            unit,
            body_fp: body_cv,
        }
    }

    pub fn abc_bound<'a>(owner: &'a str, unit: &'a str, body_fp: &'a str) -> EnvelopeIdentity<'a> {
        EnvelopeIdentity {
            unit,
            body_fp,
            ..Self::abc_raw(owner, 0)
        }
    }

    /// Code-content identities carry NO snapshot: position changes share
    /// code units (only the sidecar re-binds). Callers pass 0.
    pub fn abc(owner: &str) -> EnvelopeIdentity<'_> {
        Self::abc_raw(owner, 0)
    }

    pub fn manifest(owner: &str, snapshot: u64) -> EnvelopeIdentity<'_> {
        EnvelopeIdentity {
            kind: EnvelopeKind::ModuleManifest,
            producer: PRODUCER_ID,
            schema: ABC_SCHEMA,
            owner_module: owner,
            stage: 2,
            mir_sem: 1,
            abc_sem: 1,
            av_sem: 1,
            snapshot,
            unit: "",
            body_fp: "",
        }
    }

    /// The diagnostics-only source-map track identity.
    pub fn sidecar(owner: &str, snapshot: u64) -> EnvelopeIdentity<'_> {
        EnvelopeIdentity {
            kind: EnvelopeKind::SourceSidecar,
            producer: PRODUCER_ID,
            schema: MIR_SCHEMA,
            owner_module: owner,
            stage: 1,
            mir_sem: 1,
            abc_sem: 1,
            av_sem: 1,
            snapshot,
            unit: "",
            body_fp: "",
        }
    }

    /// PLAN-828 F-10 unit-span sidecar identity: the diagnostics-only
    /// span track OF one ABC unit, bound to the exact (owner, unit, body
    /// fingerprint) of the unit row it belongs to — a sidecar stored
    /// under another body's key fails admission.
    pub fn unit_span_sidecar<'a>(
        owner: &'a str,
        unit: &'a str,
        body_fp: &'a str,
    ) -> EnvelopeIdentity<'a> {
        EnvelopeIdentity {
            kind: EnvelopeKind::SourceSidecar,
            schema: ABC_SCHEMA,
            stage: 2,
            unit,
            body_fp,
            ..Self::sidecar(owner, 0)
        }
    }
}

#[derive(Debug)]
pub enum CacheError {
    Store(StoreError),
    Envelope(String),
    Codec(String),
    Verify(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::Store(e) => write!(f, "store: {}", e),
            CacheError::Envelope(m) => write!(f, "envelope: {}", m),
            CacheError::Codec(m) => write!(f, "codec: {}", m),
            CacheError::Verify(m) => write!(f, "verify: {}", m),
        }
    }
}

impl From<StoreError> for CacheError {
    fn from(e: StoreError) -> Self {
        CacheError::Store(e)
    }
}

pub fn envelope_wrap(payload: &[u8], id: &EnvelopeIdentity) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 96);
    out.extend_from_slice(ENVELOPE_MAGIC);
    out.extend_from_slice(&ENVELOPE_VERSION.to_le_bytes());
    out.push(id.kind as u8);
    out.push(id.stage);
    out.extend_from_slice(&id.mir_sem.to_le_bytes());
    out.extend_from_slice(&id.abc_sem.to_le_bytes());
    out.extend_from_slice(&id.av_sem.to_le_bytes());
    out.extend_from_slice(&id.snapshot.to_le_bytes());
    for s in [id.producer, id.schema, id.owner_module, id.unit, id.body_fp] {
        out.extend_from_slice(&(s.len() as u16).to_le_bytes());
        out.extend_from_slice(s.as_bytes());
    }
    out.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    out.extend_from_slice(payload);
    let sum = fnv1a64_parts(&[&out]);
    out.extend_from_slice(&sum.to_le_bytes());
    out
}

/// Unwrap + validate an envelope against the EXPECTED identity. Every field
/// must match: a self-consistent envelope from another producer, schema,
/// owner or stage is a controlled rejection, not a hit.
pub fn envelope_unwrap(bytes: &[u8], expect: &EnvelopeIdentity) -> Result<Vec<u8>, CacheError> {
    let bad = |m: String| Err(CacheError::Envelope(m));
    if bytes.len() < 4 + 2 + 1 + 1 + 12 + 8 + 8 {
        return bad("envelope truncated".to_string());
    }
    if &bytes[0..4] != ENVELOPE_MAGIC {
        return bad("not an AVEN envelope".to_string());
    }
    let ver = u16::from_le_bytes([bytes[4], bytes[5]]);
    if ver != ENVELOPE_VERSION {
        return bad(format!("unknown envelope version {}", ver));
    }
    let body_end = bytes.len() - 8;
    let want = u64::from_le_bytes(bytes[body_end..].try_into().unwrap());
    let got = fnv1a64_parts(&[&bytes[..body_end]]);
    if want != got {
        return bad(format!(
            "envelope checksum mismatch: stored {:016x}, computed {:016x}",
            want, got
        ));
    }
    let mut p = 6usize;
    let kind = bytes[p];
    p += 1;
    let stage = bytes[p];
    p += 1;
    let mir_sem = u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap());
    p += 4;
    let abc_sem = u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap());
    p += 4;
    let av_sem = u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap());
    p += 4;
    let snapshot = u64::from_le_bytes(bytes[p..p + 8].try_into().unwrap());
    p += 8;
    let mut strs = Vec::with_capacity(5);
    for _ in 0..5 {
        let n = u16::from_le_bytes(bytes[p..p + 2].try_into().unwrap()) as usize;
        p += 2;
        if p + n > body_end {
            return bad("envelope string overruns body".to_string());
        }
        strs.push(String::from_utf8_lossy(&bytes[p..p + n]).to_string());
        p += n;
    }
    let plen = u64::from_le_bytes(bytes[p..p + 8].try_into().unwrap()) as usize;
    p += 8;
    if p + plen != body_end {
        return bad(format!(
            "envelope payload length {} does not close the body at {}",
            plen, body_end
        ));
    }
    if kind != expect.kind as u8 {
        return bad(format!(
            "envelope kind {} is not the expected {:?}",
            kind, expect.kind
        ));
    }
    if stage != expect.stage {
        return bad(format!(
            "envelope stage {} is not the expected {}",
            stage, expect.stage
        ));
    }
    if strs[0] != expect.producer {
        return bad(format!(
            "envelope producer `{}` is not `{}`",
            strs[0], expect.producer
        ));
    }
    if strs[1] != expect.schema {
        return bad(format!(
            "envelope schema `{}` is not `{}`",
            strs[1], expect.schema
        ));
    }
    // v2 unit binding (F798-R6-01): the artifact must declare the exact
    // unit + body fingerprint the consumer expects — a good payload under
    // another body's key fails here, at the real consumption entry.
    if strs[3] != expect.unit || strs[4] != expect.body_fp {
        return bad(format!(
            "envelope unit binding `{}`/`{}` is not the expected `{}`/`{}`",
            strs[3], strs[4], expect.unit, expect.body_fp
        ));
    }
    if strs[2] != expect.owner_module {
        return bad(format!(
            "envelope owner `{}` is not `{}`",
            strs[2], expect.owner_module
        ));
    }
    if (mir_sem, abc_sem, av_sem) != (expect.mir_sem, expect.abc_sem, expect.av_sem) {
        return bad(format!(
            "envelope semantics ({},{},{}) do not match expected ({},{},{})",
            mir_sem, abc_sem, av_sem, expect.mir_sem, expect.abc_sem, expect.av_sem
        ));
    }
    if snapshot != expect.snapshot {
        return bad(format!(
            "envelope snapshot {} is not {}",
            snapshot, expect.snapshot
        ));
    }
    Ok(bytes[p..p + plen].to_vec())
}

// ---------------------------------------------------------------------------
// Canonical HIR body fingerprint (pre-MIR, per unit)
// ---------------------------------------------------------------------------

use auto_ac_prototype::hir;

/// Deterministic canonical dump of one function's verified HIR content —
/// the CONTENT-IDENTITY basis (history keys, store content versions).
///
/// Field policy (B1-R2-01/R3-01): every field that participates in HIR
/// semantic equality (`hir` PartialEq — the same rule the verifier
/// admits) is rendered; spans and cosmetic fields (`mutable_explicit`)
/// are not. Locals render id/name/role/index/mutability/param-mode/type;
/// places render id/kind/`local`/type/access; call bindings render
/// param/arg/`moved`; binaries render their overflow policy; exprs and
/// blocks render their ids; the body renders its id, owner (by the owner
/// def's stable id TEXT — never a table slot) and entry block.
/// Cross-table references resolve to stable content (names/shapes), so a
/// dump is table-independent. The `can4` format tag re-keys every content
/// version once: all store products (HIR artifacts, ABC units, sidecars)
/// written under the pre-R3 dump are unreachable under their old keys —
/// never silently reused or overwritten in place.
pub fn canonical_hir_function(module: &hir::Module, body_id: hir::BodyId) -> Vec<u8> {
    let body = &module.bodies[body_id.0 as usize];
    let mut c = CanonBuf {
        out: Vec::with_capacity(1024),
    };
    // Format tag + body identity/entry (semantic equality fields).
    c.w("can4[id ");
    c.w(&body.id_text);
    c.w(" o");
    c.w(&module
        .defs
        .get(body.owner.0 as usize)
        .map(|d| d.id_text.as_str())
        .unwrap_or("?"));
    c.w(" e");
    c.w(&body.entry.0.to_string());
    c.w("]");
    // Locals.
    c.w("[loc");
    for l in &body.locals {
        c.w(" ");
        c.w(&l.id_text);
        c.w(" ");
        c.w(&l.name);
        c.w(" ");
        c.w(match l.role {
            hir::LocalRole::Param => "p",
            hir::LocalRole::Binding => "b",
        });
        c.b(l.mutable);
        match l.index {
            Some(i) => {
                c.w("@");
                c.w(&i.to_string());
            }
            None => c.w("@-"),
        }
        c.w(" ");
        c.w(l.param_mode.as_str());
        c.w(":");
        c.w(&shape_of_use(&l.ty, module));
    }
    c.w("]");
    // Places.
    c.w("[pl");
    for p in &body.places {
        c.w(" ");
        c.w(&p.id_text);
        c.w(" ");
        c.w(match &p.kind {
            hir::PlaceKind::Local => "l",
            hir::PlaceKind::FieldPath(_) => "f",
        });
        c.w(&p.local.0.to_string());
        c.w(" ");
        c.w(&shape_of_use(&p.ty, module));
        c.b(matches!(p.access, hir::Access::Mutable));
    }
    c.w("]");
    // Exprs (table order; graph shape is verify-guaranteed).
    c.w("[ex");
    for e in &body.exprs {
        c.w(" (");
        c.w(&e.id_text);
        c.w(" ");
        c.w(&shape_of_use(&e.ty, module));
        c.w(" ");
        match &e.kind {
            hir::ExprKind::ReadLocal { local } => {
                c.w("rl ");
                c.w(&local.0.to_string());
            }
            hir::ExprKind::Constant { value } => {
                c.w("k ");
                c.w(&value.to_string());
            }
            hir::ExprKind::Binary {
                op,
                overflow,
                lhs,
                rhs,
            } => {
                c.w("b ");
                c.w(op.as_str());
                // Overflow policy is a semantic field — rendered even
                // though Trap is currently its only variant, so a future
                // variant cannot enter content identity through a gap.
                c.w(" ");
                c.w(match overflow {
                    hir::OverflowPolicy::Trap => "trap",
                });
                c.w(" ");
                c.w(&lhs.0.to_string());
                c.w(" ");
                c.w(&rhs.0.to_string());
            }
            hir::ExprKind::Unary { op, operand } => {
                c.w("u ");
                c.w(op.as_str());
                c.w(" ");
                c.w(&operand.0.to_string());
            }
            hir::ExprKind::Logical { op, lhs, rhs } => {
                c.w("l ");
                c.w(op.as_str());
                c.w(" ");
                c.w(&lhs.0.to_string());
                c.w(" ");
                c.w(&rhs.0.to_string());
            }
            hir::ExprKind::Call {
                callee,
                eval_args,
                bindings,
            } => {
                c.w("c ");
                let callee_name = module
                    .defs
                    .get(callee.0 as usize)
                    .map(|d| d.name_for_diagnostic().to_string())
                    .unwrap_or_else(|| "?".to_string());
                c.w(&callee_name);
                // Embed the callee SIGNATURE content (dependency edge).
                if let Some(d) = module.defs.get(callee.0 as usize) {
                    if let Some(ty) = d.kind.fn_type() {
                        c.w("<");
                        c.w(&sig_content(module, ty));
                        c.w(">");
                    }
                }
                for a in eval_args {
                    c.w(" ");
                    c.w(&a.0.to_string());
                }
                c.w(" |");
                let mut bs: Vec<_> = bindings.iter().collect();
                bs.sort_by_key(|b| b.param);
                for b in bs {
                    c.w(" ");
                    c.w(&b.param.to_string());
                    c.w(":");
                    c.w(&b.arg.to_string());
                    // The `.move` ownership marker is SEMANTIC (hir
                    // PartialEq; B1-R3-01) — it enters content identity.
                    if b.moved {
                        c.w("!");
                    }
                }
            }
            other => {
                c.w("?unsupported:");
                c.w(&format!("{:?}", std::mem::discriminant(other)));
            }
        }
        c.w(")");
    }
    c.w("]");
    // Blocks/stmts.
    c.w("[bl");
    for b in &body.blocks {
        c.w(" ");
        c.w(&b.id_text);
        c.w(" {");
        for s in &b.stmts {
            match s {
                hir::Stmt::Let { local, value } => {
                    c.w(" let ");
                    c.w(&local.0.to_string());
                    c.w(" ");
                    c.w(&value.0.to_string());
                }
                hir::Stmt::Return { value } => {
                    c.w(" ret ");
                    c.w(&value.0.to_string());
                }
                hir::Stmt::If { cond, then, els } => {
                    c.w(" if ");
                    c.w(&cond.0.to_string());
                    c.w(" ");
                    c.w(&then.0.to_string());
                    if let Some(e) = els {
                        c.w(" ");
                        c.w(&e.0.to_string());
                    }
                }
                hir::Stmt::Loop { id, body } => {
                    c.w(" loop ");
                    c.w(&id.0.to_string());
                    c.w(" ");
                    c.w(&body.0.to_string());
                }
                hir::Stmt::Assign { place, value } => {
                    c.w(" as ");
                    c.w(&place.0.to_string());
                    c.w(" ");
                    c.w(&value.0.to_string());
                }
                hir::Stmt::Break { target } => {
                    c.w(" brk ");
                    c.w(&target.0.to_string());
                }
                hir::Stmt::Continue { target } => {
                    c.w(" cont ");
                    c.w(&target.0.to_string());
                }
                hir::Stmt::Drop { local } => {
                    c.w(" drop ");
                    c.w(&local.0.to_string());
                }
            }
        }
        c.w(" }");
    }
    c.w("]");
    // Loop registration order.
    c.w("[lp");
    for l in &body.loops {
        c.w(" ");
        c.w(l);
    }
    c.w("]");
    c.out
}

/// Small write helper keeping the canonical dump allocation-light.
struct CanonBuf {
    out: Vec<u8>,
}

impl CanonBuf {
    fn w(&mut self, s: &str) {
        self.out.extend_from_slice(s.as_bytes());
    }
    fn b(&mut self, v: bool) {
        self.out.push(v as u8);
    }
}

fn shape_of_use(u: &hir::TypeUse, m: &hir::Module) -> String {
    let s = match u {
        hir::TypeUse::Builtin(b) => b.as_str(),
        hir::TypeUse::Ref(tid) => match m.types.get(tid.0 as usize).map(|t| &t.kind) {
            Some(hir::TypeKind::I32) => "i32",
            Some(hir::TypeKind::Bool) => "bool",
            _ => "other",
        },
    };
    s.to_string()
}

fn sig_content(m: &hir::Module, ty: hir::TypeId) -> String {
    match m.types.get(ty.0 as usize).map(|t| &t.kind) {
        Some(hir::TypeKind::Function(sig)) => {
            let mut s = String::from("(");
            for p in &sig.params {
                s.push_str(&shape_of_use(p, m));
                s.push(',');
            }
            s.push_str(")->");
            s.push_str(&shape_of_use(&sig.result, m));
            s
        }
        _ => "?".to_string(),
    }
}

/// RETIRED (T-06): the plan-local FNV body fingerprint was the temporary
/// semantic-ref snapshot; unit keys now ride the landed 796
/// `ContentVersion` domain via [`crate::semrefs::content_version_of`].
/// Kept only for the churn fixture's per-round edit assertions.
pub fn unit_body_fingerprint(module: &hir::Module, body_id: hir::BodyId) -> String {
    let dump = canonical_hir_function(module, body_id);
    crate::semrefs::content_version_of(&dump)
}

/// Capability-affecting configuration fingerprint (sorted, deduped).
pub fn caps_fingerprint(caps: &[String]) -> String {
    let mut v: Vec<&str> = caps.iter().map(|s| s.as_str()).collect();
    v.sort_unstable();
    v.dedup();
    format!("{:016x}", fnv1a64_parts(&[v.join(",").as_bytes()]))
}

// ---------------------------------------------------------------------------
// Unit split / closure assembly
// ---------------------------------------------------------------------------

/// Split a lowered module into per-function self-contained units. Calls to
/// other module functions become unit imports (`fn:<name>`) carrying the
/// callee signature; test-mark imports are copied verbatim.
pub fn split_units(m: &MirModule) -> Result<Vec<MirModule>, String> {
    let mut units = Vec::with_capacity(m.funcs.len());
    for f in m.funcs.iter() {
        let mut unit = MirModule::unit(m.owner.clone());
        unit.capabilities = m.capabilities.clone();
        let mut ty_map: Vec<TypeIdx> = vec![u32::MAX; m.types.len()];
        let mut c_map: Vec<u32> = vec![u32::MAX; m.consts.len()];
        // Needed types: own sig + slots + callee sigs + referenced import sigs.
        let mut needed: Vec<TypeIdx> = vec![f.sig];
        for s in &f.slots {
            needed.push(s.ty);
        }
        for b in &f.blocks {
            for i in &b.instrs {
                if let MirInstr::Call { callee, .. } = i {
                    match callee {
                        CalleeRef::Func(t) => needed.push(m.funcs[*t as usize].sig),
                        CalleeRef::Import(ii) => needed.push(m.imports[*ii as usize].sig),
                    }
                }
            }
        }
        // Transitive closure over Fn-sig param/result types: scalars intern
        // immediately; Fn types intern once their children are remapped
        // (v1 sigs have scalar children only, but keep the fixpoint general).
        loop {
            let mut progress = false;
            let mut scan = 0usize;
            while scan < needed.len() {
                let t = needed[scan];
                scan += 1;
                if ty_map[t as usize] != u32::MAX {
                    continue;
                }
                match &m.types[t as usize] {
                    MirType::Fn(sig) => {
                        let mut children_pending = false;
                        for p in sig.params.iter().chain(std::iter::once(&sig.result)) {
                            if ty_map[*p as usize] == u32::MAX {
                                children_pending = true;
                                if !needed.contains(p) {
                                    needed.push(*p);
                                }
                            }
                        }
                        if !children_pending {
                            let mut ns = FnSig {
                                params: Vec::with_capacity(sig.params.len()),
                                result: 0,
                            };
                            for p in &sig.params {
                                ns.params.push(ty_map[*p as usize]);
                            }
                            ns.result = ty_map[sig.result as usize];
                            let idx = unit.intern_type(MirType::Fn(ns));
                            ty_map[t as usize] = idx;
                            progress = true;
                        }
                    }
                    _ => {
                        let idx = unit.intern_type(m.types[t as usize].clone());
                        ty_map[t as usize] = idx;
                        progress = true;
                    }
                }
            }
            if !progress {
                break;
            }
        }
        if needed.iter().any(|t| ty_map[*t as usize] == u32::MAX) {
            return Err(format!("unit `{}`: unresolved type closure", f.name));
        }

        // Rewrite instructions into unit-local indices.
        let mut blocks = f.blocks.clone();
        for b in blocks.iter_mut() {
            for instr in b.instrs.iter_mut() {
                match instr {
                    MirInstr::Const { value, .. } => {
                        if c_map[*value as usize] == u32::MAX {
                            let idx = unit.intern_const(m.consts[*value as usize].clone());
                            c_map[*value as usize] = idx;
                        }
                        *value = c_map[*value as usize];
                    }
                    MirInstr::Call { callee, .. } => match *callee {
                        CalleeRef::Func(target) => {
                            let tdef = &m.funcs[target as usize];
                            let local_sig = ty_map[tdef.sig as usize];
                            let idx = unit.intern_import(MirImport {
                                symbol: format!("fn:{}", tdef.name),
                                effect: Effect::Pure,
                                sig: local_sig,
                            });
                            *callee = CalleeRef::Import(idx);
                        }
                        CalleeRef::Import(ii) => {
                            let src = &m.imports[ii as usize];
                            let idx = unit.intern_import(MirImport {
                                symbol: src.symbol.clone(),
                                effect: src.effect,
                                sig: ty_map[src.sig as usize],
                            });
                            *callee = CalleeRef::Import(idx);
                        }
                    },
                    _ => {}
                }
            }
        }
        let slots = f
            .slots
            .iter()
            .map(|s| MirSlot {
                kind: s.kind,
                ty: ty_map[s.ty as usize],
                name: s.name.clone(),
            })
            .collect();
        unit.funcs.push(MirFunction {
            owner: f.owner.clone(),
            name: f.name.clone(),
            sig: ty_map[f.sig as usize],
            slots,
            blocks,
            entry: f.entry,
            source_snapshot: f.source_snapshot,
            spans: f.spans.clone(),
        });
        unit.entry = 0;
        units.push(unit);
    }
    Ok(units)
}

/// Assemble an executable module closure from verified single-function ABC
/// units. Every `fn:<name>` import must resolve to another unit with a
/// structurally equal signature (interned-type equality in the merged
/// table); unresolved names or signature mismatches are controlled
/// rejections. The assembled module still passes `verify_mir`/`verify_abc`
/// before execution — this is the complete-consistency gate, not a trust
/// grant.
pub fn assemble_closure(
    units: &[MirModule],
    owner: &str,
    entry: &str,
) -> Result<MirModule, String> {
    let mut merged = MirModule::new(OwnerScope {
        schema: MIR_SCHEMA.to_string(),
        module: owner.to_string(),
    });
    merged.capabilities = units
        .iter()
        .flat_map(|u| u.capabilities.iter().cloned())
        .collect::<Vec<_>>();
    merged.capabilities.sort();
    merged.capabilities.dedup();

    struct Fused {
        name: String,
        idx: FuncIdx,
        sig: TypeIdx,
    }
    let mut fused: Vec<Fused> = Vec::new();
    // (function index in merged, block, instr, target name, merged-space sig)
    let mut patches: Vec<(usize, usize, usize, String, TypeIdx)> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for u in units {
        let ty_map = merge_types(&mut merged, u);
        let c_map = merge_consts(&mut merged, u);
        let mut blocks = u.funcs[0].blocks.clone();
        for (bi, b) in blocks.iter_mut().enumerate() {
            for (ii, instr) in b.instrs.iter_mut().enumerate() {
                match instr {
                    MirInstr::Const { value, .. } => *value = c_map[*value as usize],
                    MirInstr::Call { callee, .. } => {
                        if let CalleeRef::Import(impo) = *callee {
                            let imp = &u.imports[impo as usize];
                            let sig_merged = ty_map[imp.sig as usize];
                            if let Some(fname) = imp.symbol.strip_prefix("fn:") {
                                // Deferred resolution: unit calls may be
                                // recursive/mutual, so targets merge later.
                                patches.push((
                                    merged.funcs.len(),
                                    bi,
                                    ii,
                                    fname.to_string(),
                                    sig_merged,
                                ));
                                *callee = CalleeRef::Func(u32::MAX);
                            } else {
                                let idx = merged.intern_import(MirImport {
                                    symbol: imp.symbol.clone(),
                                    effect: imp.effect,
                                    sig: sig_merged,
                                });
                                *callee = CalleeRef::Import(idx);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        let slots = u.funcs[0]
            .slots
            .iter()
            .map(|s| MirSlot {
                kind: s.kind,
                ty: ty_map[s.ty as usize],
                name: s.name.clone(),
            })
            .collect();
        let sig = ty_map[u.funcs[0].sig as usize];
        if !matches!(merged.types.get(sig as usize), Some(MirType::Fn(_))) {
            return Err(format!(
                "unit `{}` sig is not a function type",
                u.funcs[0].name
            ));
        }
        fused.push(Fused {
            name: u.funcs[0].name.clone(),
            idx: merged.funcs.len() as FuncIdx,
            sig,
        });
        names.push(u.funcs[0].name.clone());
        merged.funcs.push(MirFunction {
            owner: merged.owner.clone(),
            name: u.funcs[0].name.clone(),
            sig,
            slots,
            blocks,
            entry: u.funcs[0].entry,
            source_snapshot: u.funcs[0].source_snapshot,
            spans: u.funcs[0].spans.clone(),
        });
    }
    // Duplicate unit names are a closure inconsistency.
    {
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        if sorted.len() != names.len() {
            return Err("duplicate unit names in closure".to_string());
        }
    }
    // Patch deferred callees; interned-type equality is content equality.
    for (fi, bi, ii, target, sig_merged) in patches {
        let t = fused.iter().find(|f| f.name == target).ok_or_else(|| {
            format!(
                "closure import `fn:{}` does not resolve to any unit",
                target
            )
        })?;
        if t.sig != sig_merged {
            return Err(format!(
                "closure signature mismatch for `fn:{}`: call site type #{} != unit type #{}",
                target, sig_merged, t.sig
            ));
        }
        merged.funcs[fi].blocks[bi].instrs[ii] = match &merged.funcs[fi].blocks[bi].instrs[ii] {
            MirInstr::Call { dst, args, .. } => MirInstr::Call {
                callee: CalleeRef::Func(t.idx),
                dst: *dst,
                args: args.clone(),
            },
            other => unreachable!("patch site is a call: {:?}", other),
        };
    }
    let entry_idx = merged
        .funcs
        .iter()
        .position(|f| f.name == entry)
        .ok_or_else(|| format!("entry function `{}` not among units", entry))?;
    merged.entry = entry_idx as u32;
    Ok(merged)
}

fn merge_types(merged: &mut MirModule, u: &MirModule) -> Vec<TypeIdx> {
    let mut map = vec![u32::MAX; u.types.len()];
    // Scalars intern immediately; Fn sigs intern once their children are
    // mapped (fixpoint loop, general over nested sigs).
    loop {
        let mut progress = false;
        for (i, t) in u.types.iter().enumerate() {
            if map[i] != u32::MAX {
                continue;
            }
            match t {
                MirType::I32
                | MirType::Bool
                | MirType::I64
                | MirType::U64
                | MirType::F32
                | MirType::F64
                | MirType::Char
                | MirType::Str
                | MirType::StrView
                | MirType::MutRef => {
                    map[i] = merged.intern_type(t.clone());
                    progress = true;
                }
                MirType::List(l) => {
                    if map[l.elem as usize] != u32::MAX {
                        let nl = auto_av_core::mir::ListTypeRef {
                            elem: map[l.elem as usize],
                        };
                        map[i] = merged.intern_type(MirType::List(nl));
                        progress = true;
                    }
                }
                MirType::Variant(v) => {
                    let pending = v
                        .arms
                        .iter()
                        .any(|a| a.payload.map_or(false, |t| map[t as usize] == u32::MAX));
                    if !pending {
                        let nv = auto_av_core::mir::VariantTypeRef {
                            owner_module: v.owner_module.clone(),
                            name: v.name.clone(),
                            arms: v
                                .arms
                                .iter()
                                .map(|a| auto_av_core::mir::VariantArm {
                                    name: a.name.clone(),
                                    payload: a.payload.map(|t| map[t as usize]),
                                })
                                .collect(),
                        };
                        let idx = merged.intern_type(MirType::Variant(nv));
                        map[i] = idx;
                        progress = true;
                    }
                }
                MirType::RecordView(r) => {
                    let pending = r.fields.iter().any(|f| map[f.ty as usize] == u32::MAX);
                    if !pending {
                        let nr = auto_av_core::mir::RecordTypeRef {
                            owner_module: r.owner_module.clone(),
                            name: r.name.clone(),
                            fields: r
                                .fields
                                .iter()
                                .map(|f| auto_av_core::mir::MirFieldDef {
                                    name: f.name.clone(),
                                    ty: map[f.ty as usize],
                                })
                                .collect(),
                        };
                        let idx = merged.intern_type(MirType::RecordView(nr));
                        map[i] = idx;
                        progress = true;
                    }
                }
                MirType::Record(r) => {
                    let pending = r.fields.iter().any(|f| map[f.ty as usize] == u32::MAX);
                    if !pending {
                        let nr = auto_av_core::mir::RecordTypeRef {
                            owner_module: r.owner_module.clone(),
                            name: r.name.clone(),
                            fields: r
                                .fields
                                .iter()
                                .map(|f| auto_av_core::mir::MirFieldDef {
                                    name: f.name.clone(),
                                    ty: map[f.ty as usize],
                                })
                                .collect(),
                        };
                        let idx = merged.intern_type(MirType::Record(nr));
                        map[i] = idx;
                        progress = true;
                    }
                }
                MirType::Fn(sig) => {
                    let pending = sig
                        .params
                        .iter()
                        .chain(std::iter::once(&sig.result))
                        .any(|p| map[*p as usize] == u32::MAX);
                    if !pending {
                        let mut ns = FnSig {
                            params: Vec::with_capacity(sig.params.len()),
                            result: 0,
                        };
                        for p in &sig.params {
                            ns.params.push(map[*p as usize]);
                        }
                        ns.result = map[sig.result as usize];
                        let idx = merged.intern_type(MirType::Fn(ns));
                        map[i] = idx;
                        progress = true;
                    }
                }
            }
        }
        if !progress {
            break;
        }
    }
    map
}

fn merge_consts(merged: &mut MirModule, u: &MirModule) -> Vec<u32> {
    let mut map = vec![0u32; u.consts.len()];
    for (i, c) in u.consts.iter().enumerate() {
        map[i] = merged.intern_const(c.clone());
    }
    map
}

// ---------------------------------------------------------------------------
// Module link manifest
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UnitLink {
    pub name: String,
    pub artifact: String,
    /// Canonical signature content `(t,t)->t` for closure cross-checks.
    pub sig: String,
    /// Shared-domain logical id (PLAN-796 identity; T-06 axis) —
    /// registration-order-independent, so restored manifests re-derive
    /// the same ids across process restarts.
    #[serde(default)]
    pub logical: String,
    /// Body fingerprint (canonical-HIR content version) the link's unit
    /// was keyed under — restore cross-checks it against the artifact's
    /// envelope binding (F798-R6-01). Empty = legacy/unbound (rejected).
    #[serde(default)]
    pub body_fp: String,
    /// Content fingerprint of the EXACT payload bytes this link points at
    /// (F798-R3-02): the persisted sema-ref envelope binds this id, and
    /// restore re-derives it from the loaded bytes before execution — a
    /// manifest rewritten onto a foreign/older payload is rejected here,
    /// not at some later "unresolved" infra branch. Empty = legacy
    /// manifest without the binding (rejected at restore admission).
    #[serde(default)]
    pub payload_fp: String,
    /// The shared-domain sema-ref envelope for this unit's payload
    /// (`{producer, schema, payload_id}`); admitted against the real
    /// ac-sema-contract producer/schema at restore.
    #[serde(default)]
    pub sema: Option<SemaRefSer>,
}

/// Serializable mirror of the 796 `SemaRef` envelope (owner producer,
/// schema, payload id) persisted in the manifest.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SemaRefSer {
    pub producer: String,
    pub schema: String,
    pub payload_id: String,
}

impl SemaRefSer {
    /// Mint the envelope for one unit payload (driver side).
    pub fn for_payload(payload_fp: &str) -> SemaRefSer {
        SemaRefSer {
            producer: ac_sema_contract::SEMA_PRODUCER_ID.to_string(),
            schema: ac_sema_contract::SEMA_REF_SCHEMA_V1.to_string(),
            payload_id: payload_fp.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ModuleManifest {
    /// Recovery metadata generation, separate from function ABC identities.
    pub format: u16,
    /// Required sidecars belong to this source snapshot. Position-only edits
    /// share unit code, but must publish a distinct recovery closure.
    pub diagnostics_snapshot: u64,
    pub owner: String,
    pub entry: String,
    pub profile: String,
    pub av_sem: u32,
    pub units: Vec<UnitLink>,
}

impl ModuleManifest {
    pub fn fingerprint(&self) -> String {
        let mut parts: Vec<String> = self
            .units
            .iter()
            .map(|u| {
                let sema = match &u.sema {
                    Some(s) => format!("{}|{}|{}", s.producer, s.schema, s.payload_id),
                    None => "-".to_string(),
                };
                format!(
                    "{}={}:{}:{}:{}:{}:{}",
                    u.name, u.artifact, u.sig, u.logical, u.body_fp, u.payload_fp, sema
                )
            })
            .collect();
        parts.sort();
        let joined = format!(
            "{}|{}|{}|{}|{}|{}|{}",
            self.format,
            self.diagnostics_snapshot,
            self.owner,
            self.entry,
            self.profile,
            self.av_sem,
            parts.join(";")
        );
        format!("{:016x}", fnv1a64_parts(&[joined.as_bytes()]))
    }
}

// ---------------------------------------------------------------------------
// Cross-layer identity helpers (F798-R3-02)
// ---------------------------------------------------------------------------

/// Content fingerprint of one persisted unit payload — the binding the
/// sema-ref envelope carries and restore re-derives from loaded bytes.
pub fn payload_fingerprint(bytes: &[u8]) -> String {
    format!("{:016x}", auto_av_core::contract::fnv1a64(bytes))
}

/// Render a MIR unit's single function signature in the SAME textual shape
/// the driver derives from HIR (`(i32,bool)->i32`), so a manifest's claimed
/// signature is checkable against the decoded payload at restore — a
/// tampered `sig` field is an identity rejection, not a silent mismatch.
pub fn render_unit_sig(unit: &MirModule) -> String {
    let shape = |m: &MirModule, idx: u32| -> String {
        match m.types.get(idx as usize) {
            Some(MirType::I32) => "i32".to_string(),
            Some(MirType::Bool) => "bool".to_string(),
            Some(MirType::I64) => "i64".to_string(),
            Some(MirType::U64) => "u64".to_string(),
            Some(MirType::F32) => "f32".to_string(),
            Some(MirType::F64) => "f64".to_string(),
            Some(MirType::Char) => "char".to_string(),
            Some(MirType::Str) => "Str".to_string(),
            Some(MirType::StrView) => "str".to_string(),
            Some(MirType::MutRef) => "mut".to_string(),
            Some(MirType::Fn(_)) => "fn".to_string(),
            Some(MirType::Record(r)) => format!("rec:{}", r.name),
            Some(MirType::RecordView(r)) => format!("recview:{}", r.name),
            Some(MirType::List(_)) => "list".to_string(),
            Some(MirType::Variant(v)) => format!("variant:{}", v.name),
            None => "?".to_string(),
        }
    };
    let Some(f) = unit.funcs.first() else {
        return "?".to_string();
    };
    match unit.sig_of(f.sig) {
        Some(sig) => {
            let mut s = String::from("(");
            for p in &sig.params {
                s.push_str(&shape(unit, *p));
                s.push(',');
            }
            s.push_str(")->");
            s.push_str(&shape(unit, sig.result));
            s
        }
        None => "?".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Keys
// ---------------------------------------------------------------------------

pub fn mir_unit_key(owner: &str, unit: &str, body_fp: &str, caps_fp: &str) -> QueryKey {
    QueryKey::new(
        &MIR_LAYER,
        vec![
            ("producer".into(), PRODUCER_ID.into()),
            ("owner".into(), owner.into()),
            ("unit".into(), unit.into()),
            ("profile".into(), AV_PROFILE.into()),
            ("body-sig".into(), body_fp.into()),
            ("mir-schema".into(), MIR_SCHEMA.into()),
            ("mir-sem".into(), MIR_SEMANTICS.0.to_string()),
            ("av-sem".into(), AV_SEMANTICS.0.to_string()),
            // F798-R12-03: the MIR unit payload is ABC-wire-encoded; the
            // codec generation belongs to the key so kind-less /1
            // artifacts can never be restored under a /2 lookup.
            ("abc-codec".into(), ABC_SCHEMA.into()),
            ("caps".into(), caps_fp.into()),
        ],
    )
}

pub fn abc_unit_key(owner: &str, unit: &str, body_fp: &str, caps_fp: &str) -> QueryKey {
    QueryKey::new(
        &ABC_LAYER,
        vec![
            ("producer".into(), PRODUCER_ID.into()),
            ("owner".into(), owner.into()),
            ("unit".into(), unit.into()),
            ("profile".into(), AV_PROFILE.into()),
            ("body-sig".into(), body_fp.into()),
            ("abc-codec".into(), ABC_SCHEMA.into()),
            ("abc-sem".into(), ABC_SEMANTICS.0.to_string()),
            ("av-sem".into(), AV_SEMANTICS.0.to_string()),
            ("caps".into(), caps_fp.into()),
        ],
    )
}

pub fn manifest_key(owner: &str, entry: &str, manifest_fp: &str) -> QueryKey {
    QueryKey::new(
        &ABC_LAYER,
        vec![
            ("producer".into(), PRODUCER_ID.into()),
            ("owner".into(), owner.into()),
            ("kind".into(), "module-manifest".into()),
            ("manifest-format".into(), "2".into()),
            ("entry".into(), entry.into()),
            ("profile".into(), AV_PROFILE.into()),
            ("units-fp".into(), manifest_fp.into()),
            ("av-sem".into(), "1".into()),
        ],
    )
}

// ---------------------------------------------------------------------------
// Store operations
// ---------------------------------------------------------------------------

/// Put one enveloped payload as a LayoutData-carried artifact.
pub fn put_enveloped(
    store: &ManagedStore,
    key: &QueryKey,
    layer: &'static str,
    id: &EnvelopeIdentity,
    payload: &[u8],
    edges: Vec<(String, ac_artifact_probe::identity::RefKind)>,
    project: Option<&str>,
) -> Result<String, CacheError> {
    let env = envelope_wrap(payload, id);
    let art = store.put_artifact(PutRequest {
        layer,
        key: key.clone(),
        kind: ChunkKind::LayoutData, // generic carrier; NEVER Hir/Object
        payload: env,
        edges,
        project: project.map(|s| s.to_string()),
    })?;
    Ok(art)
}

/// Get + validate one enveloped payload; `None` on a clean miss.
pub fn get_enveloped_opt(
    store: &ManagedStore,
    key: &QueryKey,
    layer: &str,
    expect: &EnvelopeIdentity,
) -> Result<Option<Vec<u8>>, CacheError> {
    match store.get_artifact(layer, key)? {
        None => Ok(None),
        Some(loaded) => {
            if loaded.kind != ChunkKind::LayoutData {
                return Err(CacheError::Envelope(format!(
                    "artifact {} carries kind {:?}, expected the LayoutData carrier",
                    loaded.artifact_id, loaded.kind
                )));
            }
            let payload = envelope_unwrap(&loaded.payload, expect)?;
            Ok(Some(payload))
        }
    }
}

/// F798-R15-03: sidecar APPLICABILITY — structurally valid ranges and a
/// span for EVERY trap-capable instruction (`can_trap`: Bin family +
/// NegI32). A decodable-but-empty or non-covering map loses diagnostics
/// provenance exactly like a missing one (F798-R7-03); shared by the
/// driver's hit path (which can re-derive from the current source) and
/// the source-less restore (which refuses).
pub fn sidecar_applicable(f: &MirFunction) -> Result<(), String> {
    for (_, sp) in f.spans.iter() {
        if sp.start > sp.end {
            return Err(format!("inverted span {}..{}", sp.start, sp.end));
        }
    }
    let covered: std::collections::BTreeSet<u32> = f.spans.iter().map(|(i, _)| *i).collect();
    let mut flat = 0u32;
    for b in f.blocks.iter() {
        for (ii, instr) in b.instrs.iter().enumerate() {
            if instr.can_trap() && !covered.contains(&(flat + ii as u32)) {
                return Err(format!(
                    "does not cover trap-capable instruction {}",
                    flat + ii as u32
                ));
            }
        }
        flat += b.instrs.len() as u32;
    }
    Ok(())
}

/// F798-R15-04: get + validate one enveloped payload, ALSO returning the
/// artifact id — the request-consumption lease (and manifest required
/// edges) bind to the concrete artifact, not the key.
pub fn get_enveloped_with_id(
    store: &ManagedStore,
    key: &QueryKey,
    layer: &str,
    expect: &EnvelopeIdentity,
) -> Result<Option<(String, Vec<u8>)>, CacheError> {
    match store.get_artifact(layer, key)? {
        None => Ok(None),
        Some(loaded) => {
            if loaded.kind != ChunkKind::LayoutData {
                return Err(CacheError::Envelope(format!(
                    "artifact {} carries kind {:?}, expected the LayoutData carrier",
                    loaded.artifact_id, loaded.kind
                )));
            }
            let payload = envelope_unwrap(&loaded.payload, expect)?;
            Ok(Some((loaded.artifact_id.clone(), payload)))
        }
    }
}

/// Load an artifact payload by ARTIFACT ID (restore path: the manifest holds
/// unit artifact ids; the store's address-admission still guards the read).
pub fn get_by_artifact_id(
    store: &ManagedStore,
    artifact_id: &str,
    expect: &EnvelopeIdentity,
) -> Result<Vec<u8>, CacheError> {
    #[allow(unused_imports)]
    use rusqlite::OptionalExtension;
    let db = store.db();
    let chunk: String = db
        .query_row(
            "SELECT payload_chunk FROM artifacts WHERE artifact_id = ?1",
            [artifact_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| CacheError::Store(StoreError::Io(e.to_string())))?
        .ok_or_else(|| CacheError::Envelope(format!("artifact {} not present", artifact_id)))?;
    drop(db);
    let framed = store
        .read_chunk_gated(chunk.as_str())?
        .ok_or_else(|| CacheError::Envelope(format!("artifact {} chunk missing", artifact_id)))?;
    let (kind, payload) = ac_artifact_probe::codec::unframe_chunk(&framed)
        .map_err(|e| CacheError::Codec(e.to_string()))?;
    if kind != ChunkKind::LayoutData {
        return Err(CacheError::Envelope(format!(
            "artifact {} carries kind {:?}, expected the LayoutData carrier",
            artifact_id, kind
        )));
    }
    envelope_unwrap(&payload, expect)
}

/// Convenience for `LoadedArtifact` payload access.
pub fn loaded_payload(loaded: &LoadedArtifact) -> &[u8] {
    &loaded.payload
}

/// Driver counter map type.
pub type Counters = BTreeMap<String, u64>;

/// Open (creating when absent) the probe's managed store.
pub fn open_store(root: &Path) -> Result<ManagedStore, CacheError> {
    Ok(ManagedStore::open(root)?)
}
