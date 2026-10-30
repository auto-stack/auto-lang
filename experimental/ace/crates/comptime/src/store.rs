//! Frozen-envelope persistence over the PLAN-783 `ManagedStore`
//! (contract-decision §6): 817's own `LayerSpec` (`ac817/a05-cte-exec/v1`,
//! coverage=Codec — the 783 registry itself is untouched), the generic
//! `ChunkKind::LayoutData` carrier, JSON envelope payload, restore =
//! full field comparison + freshness, never deserialized trust.

use ace_artifacts::codec::ChunkKind;
use ace_artifacts::identity::QueryKey;
use ace_artifacts::schema::{LayerCoverage, LayerSpec, Regeneration};
use ace_artifacts::store::{LoadedArtifact, ManagedStore, PutRequest};
use ace_comptime_contract::{
    CteAdmissionError, CteCode, CteEngineId, CteExecutedV1, CteRequestIdentityV1, CTE_FROZEN_LAYER,
    CTE_PRODUCER,
};

/// 817's own layer spec — the A05-limited implementation. The 783
/// registry's A05 row stays `Future` (SD-04): this spec lives beside it
/// as the plan's declared producer namespace, exactly like the 787/798
/// precedent layers.
pub const CTE_LAYER_SPEC: LayerSpec = LayerSpec {
    id: "A05",
    name: "macro-comptime (plan817 CTE executed values)",
    schema_id: "ac817/a05-cte-exec/v1",
    producer: CTE_PRODUCER,
    consumers: &["cte consumer driver", "cte probe cache"],
    key_inputs: &[
        "producer",
        "owner-module",
        "callee",
        "args",
        "query-algo",
        "caps",
        "engine",
        "manifest-fp",
    ],
    ref_kinds: &["provenance"],
    storage: "one frozen value envelope per request identity (LayoutData carrier)",
    residency: "cold — literal materialization needs no live AV heap",
    regeneration: Regeneration::FromUserInputs,
    coverage: LayerCoverage::Codec,
};

/// Store adapter: one root, one project string.
pub struct CteStore {
    store: ManagedStore,
    project: String,
}

/// The per-(store, request) cache verdict.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CacheVerdict {
    Miss,
    Hit,
}

impl CteStore {
    pub fn open(root: &std::path::Path, project: &str) -> Result<Self, String> {
        let store = ManagedStore::open(root).map_err(|e| format!("cte store open: {e}"))?;
        Ok(CteStore {
            store,
            project: project.to_string(),
        })
    }

    pub fn project(&self) -> &str {
        &self.project
    }

    /// The underlying 783 store (lifecycle tests drive GC/roots/leases).
    pub fn raw(&self) -> &ManagedStore {
        &self.store
    }
}

/// Build the store key for one request identity.
pub fn envelope_key(id: &CteRequestIdentityV1) -> QueryKey {
    QueryKey::new(
        &CTE_LAYER_SPEC,
        vec![
            ("producer".into(), CTE_PRODUCER.into()),
            ("owner".into(), id.module.clone()),
            ("callee".into(), id.callee.clone()),
            ("args".into(), id.args_canonical.clone()),
            ("algo".into(), id.query_algo.clone()),
            ("caps".into(), id.caps.join(",")),
            (
                "engine".into(),
                format!(
                    "{}/{}/{}/{}-{}-{}/{}",
                    id.engine.av_profile,
                    id.engine.mir_schema,
                    id.engine.abc_schema,
                    id.engine.mir_semantics,
                    id.engine.abc_semantics,
                    id.engine.av_semantics,
                    id.engine.exec
                ),
            ),
            ("manifest-fp".into(), id.manifest_fp.clone()),
        ],
    )
}

/// Canonical argument rendering for the identity.
pub fn args_canonical(args: &[ace_comptime_contract::CteValue]) -> String {
    args.iter()
        .map(ace_comptime_contract::CteValue::render)
        .collect::<Vec<_>>()
        .join(",")
}

/// Look up one envelope (test/seed convenience — composes the staged
/// API below). `None` = clean miss. A found row that fails identity
/// validation (wrong schema/producer/engine/request identity) is a
/// CONTROLLED REJECTION — never a miss, never a rebuild-in-place.
pub fn lookup_frozen(
    st: &CteStore,
    identity: &CteRequestIdentityV1,
) -> Result<Option<CteExecutedV1>, CteAdmissionError> {
    let Some(payload) = load_frozen_payload(st, identity)? else {
        return Ok(None);
    };
    decode_envelope(&payload, identity).map(Some)
}

/// Stage 0 of the reserved restore (F-817-R7-01): the ZERO-ALLOCATION
/// size probe. Reads the store's OWN chunk metadata (`chunks.size`,
/// recorded at put time as the FRAMED byte length = 48B header +
/// payload) through the public `ManagedStore::db()` handle — a single
/// indexed SQL lookup, no blob read, no reassembly. Returns
/// `(framed_len, payload_len)`; `None` = clean miss. The caller bounds
/// the load at these REAL numbers BEFORE `get_artifact` allocates its
/// framed Vec + unframed payload copy.
pub fn probe_frozen_size(
    st: &CteStore,
    identity: &CteRequestIdentityV1,
) -> Result<Option<(u64, u64)>, CteAdmissionError> {
    use rusqlite::OptionalExtension as _;
    let conn = st.raw().db();
    // F-817-R8-02: ONLY a genuinely absent row is a miss. SQL execution
    // errors (busy, I/O, schema damage, type conversion) propagate as
    // controlled infra rejections — never silently treated as a miss
    // that would recompute and publish.
    let framed: Option<u64> = conn
        .query_row(
            "SELECT c.size FROM artifacts a JOIN chunks c ON c.chunk_id = a.payload_chunk              WHERE a.layer = ?1 AND a.key_hash = ?2",
            rusqlite::params![
                CTE_FROZEN_LAYER,
                envelope_key(identity).hash()
            ],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| {
            CteAdmissionError::new(
                CteCode::Infra("store".into()),
                format!("frozen size probe failed: {e}"),
            )
        })?;
    Ok(framed.map(|f| {
        let payload = f.saturating_sub(ace_artifacts::codec::CHUNK_HEADER_LEN as u64);
        (f, payload)
    }))
}

/// Stage 1 of the reserved restore (F-817-R8-01): the HARD-BOUNDED
/// load. The store checks the blob file's REAL length against
/// `max_framed_bytes` BEFORE any content is read (zero content
/// allocation on rejection), and the read itself runs through
/// `Read::take(max_framed_bytes)` — the returned Vec can never exceed
/// the caller's granted budget even under file growth, replacement or
/// metadata/file drift. `None` = clean miss.
pub fn load_frozen_payload_bounded(
    st: &CteStore,
    identity: &CteRequestIdentityV1,
    max_framed_bytes: u64,
) -> Result<Option<Vec<u8>>, CteAdmissionError> {
    let loaded: Option<LoadedArtifact> = st
        .store
        .get_artifact_bounded(CTE_FROZEN_LAYER, &envelope_key(identity), max_framed_bytes)
        .map_err(|e| {
            CteAdmissionError::new(
                CteCode::Infra("store".into()),
                format!("frozen bounded lookup failed: {e}"),
            )
        })?;
    let Some(loaded) = loaded else {
        return Ok(None);
    };
    if loaded.kind != ChunkKind::LayoutData {
        return Err(CteAdmissionError::new(
            CteCode::Infra("store".into()),
            format!(
                "frozen artifact {} carries kind {:?}, expected the LayoutData carrier",
                loaded.artifact_id, loaded.kind
            ),
        ));
    }
    Ok(Some(loaded.payload))
}

/// Stage 1 of the reserved restore (F-817-R6-02): load the payload
/// bytes. The caller must have reserved a covering bound BEFORE this
/// call — the `Vec` allocation happens here. `None` = clean miss.
/// (Test/seed convenience — the production driver path uses the
/// hard-bounded variant above.)
pub fn load_frozen_payload(
    st: &CteStore,
    identity: &CteRequestIdentityV1,
) -> Result<Option<Vec<u8>>, CteAdmissionError> {
    let loaded: Option<LoadedArtifact> = st
        .store
        .get_artifact(CTE_FROZEN_LAYER, &envelope_key(identity))
        .map_err(|e| {
            CteAdmissionError::new(
                CteCode::Infra("store".into()),
                format!("frozen lookup failed: {e}"),
            )
        })?;
    let Some(loaded) = loaded else {
        return Ok(None);
    };
    if loaded.kind != ChunkKind::LayoutData {
        return Err(CteAdmissionError::new(
            CteCode::Infra("store".into()),
            format!(
                "frozen artifact {} carries kind {:?}, expected the LayoutData carrier",
                loaded.artifact_id, loaded.kind
            ),
        ));
    }
    Ok(Some(loaded.payload))
}

/// The DECODE budget auditor (F-817-R10-01, model repaired in
/// F-817-R11-01): a ZERO-ALLOCATION JSON structural scan of the payload
/// that derives the Hot grant BEFORE `serde_json::from_slice` runs.
/// serde's container growth is not anchored by the payload length (a
/// short-string array decoded to a 12MiB single allocation against a
/// "2x payload" grant), so the grant is derived from the ACTUAL token
/// and container population:
///
///   grant = S                    string CONTENT bytes (the audit counts
///            each string's ENCODED span; the decoded copy is never
///            longer). serde_core's StringVisitor `to_owned`s every
///            parse result into the envelope being built, so ALL owned
///            copies coexist in the final structure — one sum covers
///            them (plain strings take the Borrowed zero-copy path and
///            cost exactly their owned copy).
///         + 2*T                    THE one serde_json scratch Vec, at
///            its peak capacity. `Deserializer::scratch` (serde_json
///            1.0.154 de.rs) is a single shared buffer — `clear()`ed
///            (capacity KEPT) per token, grown by Vec doubling — so
///            its capacity tops out at next_pow2(content) <= 2x the
///            LARGEST single token it ever held. Tokens that use it:
///            strings WITH escapes (SliceRead::parse_str_bytes extends
///            scratch and returns Reference::Copied) and number
///            literals long enough to reach parse_long_integer /
///            parse_decimal (the digits accumulate in the same
///            buffer). T = max over ALL strings and number literals
///            because the buffer is shared — one term covers both
///            branches, and charging plain strings too is a bound, not
///            a bill. The scratch outlives every token (dropped only
///            with the Deserializer), so the steady worst state is
///            all-owned + full scratch = S + 2T — exactly the grant.
///            While a token's scratch GROWS, the realloc'd old+new
///            blocks briefly coexist: this observer nets reallocs, but
///            even a strict alloc/copy/dealloc view stays inside the
///            grant — the transient is prev-owned + 1.5*cap <=
///            (S - T) + 3T <= S + 2T because the growing token's own
///            owned copy does not exist yet.
///         + A * (3*24*2 + 32)      array ELEMENTS: the widest slot the
///            envelope carries is Vec<(String,String,String)> =
///            3x24B; Vec doubling keeps <=2x slots live; +32B per
///            element covers the allocator's minimum granularity for
///            tiny items (empty strings etc.)
///         + 4096                   fixed top-level fields, tiny-token
///            scratch floors (a 1-char escaped string still allocates
///            the 8B minimum Vec slot), and small error-path
///            formatting.
///
/// Every factor is anchored in the serde/serde_json code paths and Rust
/// Vec growth — not an arbitrary multiplier on the payload size (the
/// old "2x S" string term under-covered the escaped-string tier, where
/// scratch-capacity + owned coexist at ~3x the string). Nesting deeper
/// than 64 (a legal envelope is <= 5 deep) and malformed JSON are
/// controlled rejections at the audit itself, BEFORE any decode
/// allocation. The scan reads the payload slice only and allocates
/// nothing.
pub fn audit_envelope_budget(payload: &[u8]) -> Result<u64, CteAdmissionError> {
    const MAX_DEPTH: usize = 64;
    let mut i = 0usize;
    let mut string_bytes: u64 = 0;
    let mut string_max: u64 = 0;
    let mut number_max: u64 = 0;
    let mut array_items: u64 = 0;

    fn skip_ws(p: &[u8], i: &mut usize) {
        while *i < p.len() && matches!(p[*i], b' ' | b'\t' | b'\n' | b'\r') {
            *i += 1;
        }
    }
    fn scan_string(
        p: &[u8],
        i: &mut usize,
        string_bytes: &mut u64,
        string_max: &mut u64,
    ) -> Result<(), &'static str> {
        if p.get(*i) != Some(&b'"') {
            return Err("expected string");
        }
        *i += 1;
        let start = *i;
        while let Some(&c) = p.get(*i) {
            match c {
                b'"' => {
                    let len = (*i - start) as u64;
                    *string_bytes += len;
                    *string_max = (*string_max).max(len);
                    *i += 1;
                    return Ok(());
                }
                b'\\' => {
                    *i += 2;
                }
                _ => *i += 1,
            }
        }
        Err("unterminated json string")
    }
    fn scan_value(
        p: &[u8],
        i: &mut usize,
        d: usize,
        max_depth: usize,
        string_bytes: &mut u64,
        string_max: &mut u64,
        number_max: &mut u64,
        array_items: &mut u64,
    ) -> Result<(), &'static str> {
        if d > max_depth {
            return Err("json nesting deeper than the envelope audit bound");
        }
        skip_ws(p, i);
        let Some(&c) = p.get(*i) else {
            return Err("unexpected end of json");
        };
        match c {
            b'{' => {
                *i += 1;
                skip_ws(p, i);
                if p.get(*i) == Some(&b'}') {
                    *i += 1;
                    return Ok(());
                }
                loop {
                    skip_ws(p, i);
                    scan_string(p, i, string_bytes, string_max)?;
                    skip_ws(p, i);
                    if p.get(*i) != Some(&b':') {
                        return Err("expected ':' in object");
                    }
                    *i += 1;
                    scan_value(
                        p,
                        i,
                        d + 1,
                        max_depth,
                        string_bytes,
                        string_max,
                        number_max,
                        array_items,
                    )?;
                    skip_ws(p, i);
                    match p.get(*i) {
                        Some(&b',') => *i += 1,
                        Some(&b'}') => {
                            *i += 1;
                            return Ok(());
                        }
                        _ => return Err("expected ',' or '}' in object"),
                    }
                }
            }
            b'[' => {
                *i += 1;
                skip_ws(p, i);
                if p.get(*i) == Some(&b']') {
                    *i += 1;
                    return Ok(());
                }
                loop {
                    scan_value(
                        p,
                        i,
                        d + 1,
                        max_depth,
                        string_bytes,
                        string_max,
                        number_max,
                        array_items,
                    )?;
                    *array_items += 1;
                    skip_ws(p, i);
                    match p.get(*i) {
                        Some(&b',') => *i += 1,
                        Some(&b']') => {
                            *i += 1;
                            return Ok(());
                        }
                        _ => return Err("expected ',' or ']' in array"),
                    }
                }
            }
            b'"' => scan_string(p, i, string_bytes, string_max),
            b't' | b'f' | b'n' => {
                for lit in ["true", "false", "null"] {
                    if p[*i..].starts_with(lit.as_bytes()) {
                        *i += lit.len();
                        return Ok(());
                    }
                }
                Err("bad json literal")
            }
            b'-' | b'0'..=b'9' => {
                let start = *i;
                if p.get(*i) == Some(&b'-') {
                    *i += 1;
                }
                let digits_start = *i;
                while matches!(p.get(*i), Some(b'0'..=b'9')) {
                    *i += 1;
                }
                if *i == digits_start {
                    return Err("bad json number");
                }
                if p.get(*i) == Some(&b'.') {
                    *i += 1;
                    while matches!(p.get(*i), Some(b'0'..=b'9')) {
                        *i += 1;
                    }
                }
                // The literal's full span feeds parse_long_integer /
                // parse_decimal's shared scratch when it overflows the
                // u64 fast path, so its length is a scratch token too.
                *number_max = (*number_max).max((*i - start) as u64);
                Ok(())
            }
            _ => Err("unexpected json byte"),
        }
    }

    scan_value(
        payload,
        &mut i,
        0,
        MAX_DEPTH,
        &mut string_bytes,
        &mut string_max,
        &mut number_max,
        &mut array_items,
    )
    .map_err(|m| {
        CteAdmissionError::new(
            CteCode::Infra("restore".into()),
            format!("envelope audit: {m}"),
        )
    })?;
    skip_ws(payload, &mut i);
    if i != payload.len() {
        return Err(CteAdmissionError::new(
            CteCode::Infra("restore".into()),
            "envelope audit: trailing bytes after the json value",
        ));
    }
    let scratch_token = string_max.max(number_max);
    Ok(string_bytes
        .saturating_add(scratch_token.saturating_mul(2))
        .saturating_add(array_items.saturating_mul(3 * 24 * 2 + 32))
        .saturating_add(4096))
}

/// Stage 2 of the reserved restore (F-817-R6-02): decode + full-field
/// admission. The caller must have reserved a covering Hot bound BEFORE
/// this call — the decoded envelope's allocations happen here. The
/// serialized payload length is a sound upper bound for the decoded
/// structure (JSON text ≥ decoded content).
pub fn decode_envelope(
    payload: &[u8],
    identity: &CteRequestIdentityV1,
) -> Result<CteExecutedV1, CteAdmissionError> {
    let env: CteExecutedV1 = serde_json::from_slice(payload).map_err(|e| {
        CteAdmissionError::new(
            CteCode::Infra("store".into()),
            format!("frozen envelope does not decode: {e}"),
        )
    })?;

    // Full-field restore admission (§6): wrong producer/schema/engine/
    // request identity are controlled rejections.
    if env.schema != ace_comptime_contract::CTE_EXECUTED_SCHEMA {
        return Err(CteAdmissionError::new(
            CteCode::Infra("restore".into()),
            format!(
                "frozen envelope schema `{}` is not {}",
                env.schema,
                ace_comptime_contract::CTE_EXECUTED_SCHEMA
            ),
        ));
    }
    if env.producer != CTE_PRODUCER {
        return Err(CteAdmissionError::new(
            CteCode::Infra("restore".into()),
            format!(
                "frozen envelope producer `{}` is not the real-execution producer `{CTE_PRODUCER}` — model_compute output cannot mint execution",
                env.producer
            ),
        ));
    }
    if env.request_identity != *identity {
        return Err(CteAdmissionError::new(
            CteCode::Infra("restore".into()),
            "frozen envelope request identity does not match this request (wrong owner/key/address)",
        ));
    }
    if env.engine != identity.engine {
        return Err(CteAdmissionError::new(
            CteCode::Infra("restore".into()),
            "frozen envelope engine identity does not match the current engine",
        ));
    }
    Ok(env)
}

/// Publish one envelope (test/seed convenience — no ledger; the
/// production request path reserves first, see driver::publish_env).
pub fn publish_frozen(st: &CteStore, env: &CteExecutedV1) -> Result<String, CteAdmissionError> {
    let payload = serialize_envelope(env)?;
    publish_frozen_payload(st, env, payload)
}

/// Serialize once — callers that must reserve ledger bytes first count
/// with [`count_envelope_bytes`] and reserve BEFORE calling this (the
/// allocation happens here).
pub fn serialize_envelope(env: &CteExecutedV1) -> Result<Vec<u8>, CteAdmissionError> {
    serde_json::to_vec(env).map_err(|e| {
        CteAdmissionError::new(
            CteCode::Infra("encode".into()),
            format!("envelope serialization failed: {e}"),
        )
    })
}

/// The reserved encode (F-817-R6-02): COUNT the bytes without
/// allocating, RESERVE the exact staging amount, then serialize — the
/// allocation provably happens only after the reservation (the budget
/// event log records Reserve(ok) BEFORE Alloc("envelope-encode")). A
/// rejected reservation returns Err with NO payload produced.
pub fn serialize_envelope_reserved(
    env: &CteExecutedV1,
    budget: &mut crate::observe::ManagedBudget,
) -> Result<Vec<u8>, CteAdmissionError> {
    let count = count_envelope_bytes(env)?;
    budget
        .reserve(crate::observe::Partition::Staging, count, "envelope encode")
        .map_err(|m| CteAdmissionError::new(CteCode::Resource("managed-staging".into()), m))?;
    // EXACT capacity (F-817-R10-02): pre-sized to the reserved count,
    // filled by to_writer — the allocation the allocator sees IS the
    // grant (no growth-chain capacity overshoot).
    let mut buf = Vec::with_capacity(count as usize);
    serde_json::to_writer(&mut buf, env).map_err(|e| {
        CteAdmissionError::new(
            CteCode::Infra("encode".into()),
            format!("envelope serialization failed: {e}"),
        )
    })?;
    debug_assert_eq!(buf.len() as u64, count);
    budget.record_alloc("envelope-encode", buf.capacity() as u64);
    Ok(buf)
}

/// Count the serialized envelope bytes WITHOUT allocating (a counting
/// writer; serde's internal scratch is O(1)) — the reservation input
/// for the F-817-R6-02 reserve-before-encode order.
pub fn count_envelope_bytes(env: &CteExecutedV1) -> Result<u64, CteAdmissionError> {
    struct Count(u64);
    impl std::io::Write for Count {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0 += buf.len() as u64;
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut c = Count(0);
    serde_json::to_writer(&mut c, env).map_err(|e| {
        CteAdmissionError::new(
            CteCode::Infra("encode".into()),
            format!("envelope counting serialization failed: {e}"),
        )
    })?;
    Ok(c.0)
}

/// Publish a PRE-SERIALIZED envelope (the caller reserved the ledger
/// bytes before serializing). The payload moves in — the store path
/// adds NO second unreserved copy (F-817-R6-02).
pub fn publish_frozen_payload(
    st: &CteStore,
    env: &CteExecutedV1,
    payload: Vec<u8>,
) -> Result<String, CteAdmissionError> {
    st.store
        .put_artifact(PutRequest {
            layer: CTE_FROZEN_LAYER,
            key: envelope_key(&env.request_identity),
            kind: ChunkKind::LayoutData,
            payload,
            edges: Vec::new(),
            project: Some(st.project.clone()),
        })
        .map_err(|e| {
            CteAdmissionError::new(
                CteCode::Infra("store".into()),
                format!("frozen publish failed: {e}"),
            )
        })
}

/// Freshness: the envelope's dependency manifest must still describe the
/// CURRENT input (helper bodies/signatures, candidate sets, caps).
pub fn frozen_is_fresh(
    env: &CteExecutedV1,
    input: &ace_semantics::SemaInput,
    caps: &[String],
) -> bool {
    ace_comptime_core::dependencies::manifest_fresh(&env.deps, input, caps)
}

/// The engine identity embedded in fresh envelopes.
pub fn expected_engine(current: &CteEngineId) -> CteEngineId {
    current.clone()
}
