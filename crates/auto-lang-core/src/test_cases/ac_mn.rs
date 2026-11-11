//! PLAN-818: the `ac/mir-native-scalar-v1` probe recipe — formal AutoDown
//! execution of the MIR-native backend through the unified probe/run seam
//! (PLAN-796, riding the PLAN-788 TestCase v2 runner).
//!
//! Same contract shape as `ac_av.rs`: the case declares a recipe + steps +
//! an `mn` binding (entry/caps); this adapter builds/spawns the plan's
//! worker (`mn-worker case-run`, protocol `mnworker/1`) under Job
//! containment, admits the schema/nonce/request-hash/recipe/version echoes,
//! and judges the observed outcome PARENT-SIDE. The worker never sees the
//! expected outcomes.
//!
//! Step semantics (T-06): `restart` splits the sequence into worker
//! SESSIONS — each session is a REAL new worker process (the store on disk
//! is the only carried state); a session's leading `edit-source` steps
//! materialize PARENT-SIDE (exactly-once literal replacement) before the
//! request, so the worker protocol `mnworker/1` stays unchanged (the T-02
//! single-run channel decodes and runs identically). The judged step is
//! the sequence's LAST run; every session's run is structurally admitted
//! (it really linked and really executed) and its manifest identity is
//! admitted parent-side — sessions fed identical sources must echo
//! IDENTICAL manifest fingerprints.
//!
//! Judgement faces:
//! - `completion: "success"` + `exit: <i32>` — the real PE exit code
//!   (a language trap IS exit 70, the frozen experiment contract);
//! - `completion: { stage: "parse"|"typecheck", code }` — a diagnosed
//!   pipeline rejection.

use crate::test_cases::sema::containment;
use auto_test_case::compare::{compare_bytes, Mismatch};
use auto_test_case::contract::{CheckSpec, Completion, Stage};
use auto_test_case::ir::{ResolvedCase, ResolvedOracle};
use auto_test_case::report::{NativeRecord, Status};
use auto_test_case::sha256::{hex, sha256};
use base64::Engine as _;

use super::avm::Verdict;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const READ_CAP: usize = (4 << 20) + (64 << 10);
const STDERR_CAP: usize = 1 << 20;
const SESSION_WALL: Duration = Duration::from_secs(120);
const BUILD_WALL: Duration = Duration::from_secs(300);

pub(crate) fn contract_inventory() -> serde_json::Value {
    serde_json::json!({
        "source":"crates/auto-lang-core/src/test_cases/ac_mn.rs", "protocol":"mnworker/1",
        "limits": {
            "response_bytes":READ_CAP, "stderr_bytes":STDERR_CAP,
            "default_session_wall_ms":SESSION_WALL.as_millis() as u64,
            "default_build_wall_ms":BUILD_WALL.as_millis() as u64,
            "effective_rounds_wall_ms":rounds_wall().as_millis() as u64,
            "rounds_override":std::env::var("AC_MN_ROUNDS_WALL_SECS").ok(),
        },
    })
}

/// PLAN-914: decode a worker-reported hex stream (`stdout_hex`/`stderr_hex`).
/// Odd length or a non-hex byte is a real admission mismatch on the stream
/// face (never a silently skipped comparison).
fn hex_stream(raw: &str) -> Result<Vec<u8>, String> {
    let bytes = raw.as_bytes();
    if bytes.len() % 2 != 0 {
        return Err(format!("odd-length hex stream ({} nibbles)", bytes.len()));
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i < bytes.len() {
        let hi = (bytes[i] as char)
            .to_digit(16)
            .ok_or_else(|| format!("non-hex nibble at byte {i}"))? as u8;
        let lo = (bytes[i + 1] as char)
            .to_digit(16)
            .ok_or_else(|| format!("non-hex nibble at byte {}", i + 1))? as u8;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

pub const MN_RECIPE: &str = "ac/mir-native-scalar-v1";
pub const COLLECTIONS_RECIPE: &str = "ac/mir-native-collections-v1";
/// PLAN-891: the native typed-entry recipe (real PE bitwise frame channel).

/// PLAN-914 (I09) / PLAN-932: read the typed refusal of an incremental
/// round. The kind comes from the worker's typed face (top-level `refusal`,
/// or the last step record's `refusal` in the strict source-v1 shape); the
/// reason is the rendered text. A refusal is a POLICY VERDICT only when its
/// typed kind is present and is a declared engine-policy kind — a bare
/// `refused` string (or a missing/unknown/untyped/resource kind) is a
/// typed-fact breach the caller reports as an infra ERROR, never a verdict
/// guessed from the text and never a plain FAIL that masks the missing
/// facts.
fn refusal_verdict(ok: &serde_json::Value) -> Option<Result<(String, String), String>> {
    let refusal = ok.get("refusal").or_else(|| {
        ok.get("steps")
            .and_then(|s| s.as_array())
            .and_then(|a| a.last())
            .and_then(|last| last.get("refusal"))
    });
    let refused = ok.get("refused").and_then(|v| v.as_str());
    if refusal.is_none() && refused.is_none() {
        return None;
    }
    let reason = refusal
        .and_then(|r| r.get("reason"))
        .and_then(|v| v.as_str())
        .or(refused)
        .unwrap_or("refused by the retained environment")
        .to_string();
    let breach = |detail: &str| {
        Some(Err(format!(
            "refused warm edit without a valid typed refusal fact: {detail} (reason: {reason})"
        )))
    };
    let Some(refusal) = refusal.filter(|r| r.is_object()) else {
        return breach("no `refusal` object on the wire (untyped `refused` only)");
    };
    let kind = match refusal.get("kind").and_then(|v| v.as_str()) {
        Some(k) => k,
        None => return breach("`refusal.kind` is missing or not a string"),
    };
    // The declared engine-policy kinds (mirrors native `sd::RefusalKind`).
    // `language-diagnostics` must arrive as an admitted `source_error` fact
    // (never through this face) and `syntax-not-converged` through the
    // fallback rounds — seeing either here is a typed-fact contradiction.
    // `untyped`/`resource` are not policy verdicts (missing facts / a
    // budget breach while capturing the fact → infra).
    match kind {
        "signature-or-context-drift" | "invalid-edit" | "session-or-candidate" | "lower-admit" => {}
        "language-diagnostics" => {
            return breach(
                "kind `language-diagnostics` must be admitted as a source_error fact, not a refusal verdict",
            )
        }
        "syntax-not-converged" => {
            return breach(
                "kind `syntax-not-converged` must arrive through the fallback rounds, not a refusal verdict",
            )
        }
        "untyped" => return breach("kind `untyped` carries no typed facts"),
        "resource" => return breach("kind `resource` is a fact-capture budget breach"),
        other => return breach(&format!("unknown `refusal.kind` {other:?}")),
    }
    Some(Ok((format!("mn.incremental.refused.{kind}"), reason)))
}

pub const TYPED_RECIPE: &str = "ac/mir-native-typed-v1";
/// PLAN-919: the per-owner lifecycle recipe (scenario R01/R04/R06 ac
/// cells) — real collections-profile sources with the bounded `MN-EVT`
/// observation stream on stderr and the R06 exit knobs driven by the
/// generated entry wrapper; parent-side judgement is independent.
pub const RESOURCE_EVENTS_RECIPE: &str = "ac/mir-native-resource-events-v1";
/// PLAN-923: the native resource-guards recipe (scenario R03 ac cell) —
/// per-seam fault injection (seam/nth/budget requests only) plus the
/// checked List-place machine chain on the same collections PE.
pub const RESOURCE_GUARDS_RECIPE: &str = "ac/mir-native-resource-guards-v1";
/// PLAN-924: the native closures-resource recipe (scenario R02/R05 ac
/// cells) — the limited native closure domain on the same runtime library,
/// with the env-node seam (S9) added to the injection domain.
pub const CLOSURES_RESOURCE_RECIPE: &str = "ac/mir-native-closures-resource-v1";
mod collections;
mod native_closures;
mod resource_events;
mod resource_guards;
mod typed;
pub use collections::campaign::{
    run as run_collections_campaign, run_cold_repair as run_collections_cold_repair_campaign,
};
pub const MN_WORKER_PROTOCOL: &str = "mnworker/1";
/// Mirrored worker identity expectations — admission constants; a worker
/// bump without a mirror update fails loudly.
pub const EXPECTED_MN_WORKER_VERSION: &str = "mnworker/1.0";
/// PLAN-911: parent-side STRICT admission of a `mn.source-diagnostic/1`
/// fact — everything here runs BEFORE any oracle comparison, so a
/// protocol/fact failure is an `Error`, never a language FAIL. The
/// admission is taken on the RAW response line (duplicate keys inside
/// `source_error` are scanned on the wire bytes — serde's Map would
/// already have collapsed them) and on the parsed value (exact field
/// sets, types, caps, span bounds, source/operation binding, and the
/// legacy `diagnostic` projection agreement).
mod source_diag {
    use auto_test_case::contract::Stage;
    use serde_json::Value;

    pub const SNAPSHOT_SCHEMA: &str = "mn.source-diagnostic/1";
    pub const MODE: &str = "source-v1";
    const MAX_CAUSES: usize = 64;
    const MAX_MESSAGE_UTF8: usize = 4096;
    const MAX_PAYLOAD_UTF8: usize = 64 * 1024;
    const SNAPSHOT_FIELDS: &[&str] = &[
        "schema",
        "mode",
        "producer",
        "profile",
        "attempted",
        "surface_rejections",
        "path",
        "source_sha256",
        "source_len",
        "span_basis",
        "causes",
        "primary",
        "operation",
    ];
    const CAUSE_FIELDS: &[&str] = &["stage", "code", "message", "span"];
    const OPERATION_FIELDS: &[&str] = &[
        "kind",
        "step",
        "base_sha256",
        "candidate_sha256",
        "session_revision",
        "edit",
    ];
    const SURFACE_FIELDS: &[&str] = &["profile", "codes"];
    const SPAN_FIELDS: &[&str] = &["start", "end"];
    /// PLAN-961 added the numeric source surface as a terminating rung of
    /// the native escalation ladder; PLAN-1000 T-09 (911 red) admits it here.
    const PROFILES: &[&str] = &["scalar", "record", "text-record", "numeric"];
    /// PLAN-999 (T-03) reception: the native ladder attempts a Core side face
    /// between the monotone rungs, so "core" may appear in `attempted`. It is
    /// never the terminating profile (side faces cannot terminate escalation)
    /// and never lists a surface rejection.
    const ATTEMPTED_PROFILES: &[&str] = &["scalar", "record", "text-record", "numeric", "core"];
    const OPERATIONS: &[&str] = &["cold", "warm-edit", "fallback-cold"];
    /// The raw stage vocabulary (Stage::as_str()); only these three can
    /// carry a language-rejection family — anything else is admitted as a
    /// fact ERROR, never defaulted into a family.
    const LANGUAGE_STAGES: &[&str] = &["text", "bind", "verify"];

    /// The admitted fact: what the parent uses for judgement and the
    /// evidence artifact.
    #[derive(Debug)]
    pub struct AdmittedFact {
        /// The validated snapshot, verbatim (for the evidence artifact).
        pub snapshot: Value,
        pub raw_stage: String,
        pub mapped_stage: Stage,
        pub primary_code: String,
        pub primary_message: String,
        pub source_sha256: String,
        pub candidate_sha256: String,
        /// The exact legacy `mn.unsupported.frontend` message projection.
        pub legacy_message: String,
    }

    /// What the operation block must be bound to (the parent's own view of
    /// the request chain — never the payload's self-report). The parent
    /// supplies the candidate/base BYTES: the SHAs are computed here and the
    /// bytes also back the utf8-byte span boundary check.
    pub enum OperationExpectation<'a> {
        /// `step`: `None` for a request-level cold operation (the snapshot
        /// must leave it null); `Some(i)` when a rounds round answered with
        /// a cold whole-request (the snapshot must name exactly that index).
        Cold {
            candidate: &'a [u8],
            step: Option<u64>,
        },
        /// `step`: the parent's own 0-based step index of the failing round.
        WarmEdit {
            base: &'a [u8],
            candidate: &'a [u8],
            step: u64,
        },
        FallbackCold {
            base: &'a [u8],
            candidate: &'a [u8],
            step: u64,
        },
    }

    impl OperationExpectation<'_> {
        fn candidate_bytes(&self) -> &[u8] {
            match self {
                OperationExpectation::Cold { candidate, .. }
                | OperationExpectation::WarmEdit { candidate, .. }
                | OperationExpectation::FallbackCold { candidate, .. } => candidate,
            }
        }
    }

    fn sha_hex(bytes: &[u8]) -> String {
        auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(bytes))
    }

    fn obj_keys(v: &Value, path: &str, known: &[&str]) -> Result<(), String> {
        let Some(m) = v.as_object() else {
            return Err(format!("{path} must be an object"));
        };
        for k in m.keys() {
            if !known.contains(&k.as_str()) {
                return Err(format!("{path} has unknown field {k:?}"));
            }
        }
        if m.len() != known.len() {
            let missing: Vec<&str> = known
                .iter()
                .filter(|f| !m.contains_key(**f))
                .copied()
                .collect();
            return Err(format!("{path} is missing fields {missing:?}"));
        }
        Ok(())
    }

    fn req_str(v: &Value, path: &str) -> Result<String, String> {
        v.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| format!("{path} must be a string"))
    }

    fn req_u64(v: &Value, path: &str) -> Result<u64, String> {
        v.as_u64()
            .ok_or_else(|| format!("{path} must be an unsigned integer (no strings/floats/bools)"))
    }

    fn stage_debug_name(stage: &str) -> String {
        let mut c = stage.chars();
        match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => String::new(),
        }
    }

    /// RAW-BYTE duplicate-key scan inside the `source_error` object of the
    /// line the worker actually wrote (V-18): a key appearing twice in the
    /// same object — in either order — is rejected on the wire bytes, not
    /// on the (already collapsed) parsed Map.
    pub fn scan_raw_duplicate_keys(line: &str) -> Result<(), String> {
        const ANCHOR: &str = "\"source_error\"";
        let Some(pos) = line.find(ANCHOR) else {
            return Ok(());
        };
        let bytes = line.as_bytes();
        let mut i = pos + ANCHOR.len();
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b':' {
            i += 1;
        }
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'{' {
            return Err("source_error is not an object".into());
        }
        let mut stack: Vec<std::collections::BTreeSet<String>> = Vec::new();
        let mut in_str = false;
        let mut esc = false;
        // After `{` or `,` the next string token is an object KEY; after
        // `:` it is a value.
        let mut expect_key = false;
        let mut key = String::new();
        let mut started = false;
        while i < bytes.len() {
            let c = bytes[i] as char;
            if in_str {
                if esc {
                    esc = false;
                } else if c == '\\' {
                    esc = true;
                } else if c == '"' {
                    in_str = false;
                    if expect_key {
                        expect_key = false;
                        let Some(top) = stack.last_mut() else {
                            return Err("source_error key outside any object".into());
                        };
                        if !top.insert(key.clone()) {
                            return Err(format!(
                                "duplicate object key {key:?} inside source_error (raw wire bytes)"
                            ));
                        }
                        key.clear();
                    }
                } else if expect_key {
                    key.push(c);
                }
            } else {
                match c {
                    '{' => {
                        stack.push(Default::default());
                        expect_key = true;
                        started = true;
                    }
                    '}' => {
                        if stack.pop().is_none() {
                            return Err("source_error braces are unbalanced".into());
                        }
                        if started && stack.is_empty() {
                            return Ok(());
                        }
                        expect_key = false;
                    }
                    ',' => expect_key = true,
                    ':' => expect_key = false,
                    '"' => in_str = true,
                    _ => {}
                }
            }
            i += 1;
        }
        Err("source_error object is truncated".into())
    }

    /// Full admission of a source-v1 response carrying a source rejection.
    /// `raw_line` is the worker's response line as read; `resp` its parsed
    /// value; `expected_path` the path the producer must have received
    /// ("mn.at" cold / "probe.at" warm); `op` the parent's own operation
    /// binding.
    pub fn admit(
        raw_line: &str,
        resp: &Value,
        expected_path: &str,
        op: &OperationExpectation<'_>,
    ) -> Result<AdmittedFact, String> {
        scan_raw_duplicate_keys(raw_line)?;
        let snap = resp
            .get("source_error")
            .ok_or_else(|| "source-v1 response without source_error".to_string())?;
        obj_keys(snap, "source_error", SNAPSHOT_FIELDS)?;
        if req_str(&snap["schema"], "source_error.schema")? != SNAPSHOT_SCHEMA {
            return Err("source_error.schema mismatch".into());
        }
        if req_str(&snap["mode"], "source_error.mode")? != MODE {
            return Err("source_error.mode mismatch".into());
        }
        if req_str(&snap["span_basis"], "source_error.span_basis")? != "utf8-byte" {
            return Err("source_error.span_basis must be utf8-byte".into());
        }
        if req_str(&snap["producer"], "source_error.producer")?.is_empty() {
            return Err("source_error.producer must be named".into());
        }
        let profile = req_str(&snap["profile"], "source_error.profile")?;
        if !PROFILES.contains(&profile.as_str()) {
            return Err(format!("source_error.profile {profile:?} is unknown"));
        }
        // The escalation chain: attempted is non-empty, ends at the
        // terminating profile, and every entry is a known profile.
        let attempted = snap["attempted"]
            .as_array()
            .ok_or("source_error.attempted must be an array")?;
        if attempted.is_empty() {
            return Err("source_error.attempted must name at least the terminating profile".into());
        }
        for a in attempted {
            let a = req_str(a, "source_error.attempted[..]")?;
            if !ATTEMPTED_PROFILES.contains(&a.as_str()) {
                return Err(format!("source_error.attempted entry {a:?} is unknown"));
            }
        }
        let last = req_str(
            attempted.last().expect("non-empty"),
            "source_error.attempted[last]",
        )?;
        if last != profile {
            return Err(format!(
                "source_error.attempted ends at {last:?} but the terminating profile is {profile:?}"
            ));
        }
        let surfaces = snap["surface_rejections"]
            .as_array()
            .ok_or("source_error.surface_rejections must be an array")?;
        for s in surfaces {
            obj_keys(s, "source_error.surface_rejections[..]", SURFACE_FIELDS)?;
            let p = req_str(&s["profile"], "surface_rejections[..].profile")?;
            if !PROFILES.contains(&p.as_str()) {
                return Err(format!("surface rejection profile {p:?} is unknown"));
            }
            let codes = s["codes"]
                .as_array()
                .ok_or("surface_rejections[..].codes must be an array")?;
            for c in codes {
                req_str(c, "surface_rejections[..].codes[..]")?;
            }
        }
        if req_str(&snap["path"], "source_error.path")? != expected_path {
            return Err(format!(
                "source_error.path must be the producer's real path {expected_path:?}"
            ));
        }
        let source_sha = req_str(&snap["source_sha256"], "source_error.source_sha256")?;
        if source_sha.len() != 64 || !source_sha.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("source_error.source_sha256 must be a hex sha256".into());
        }
        let source_len = req_u64(&snap["source_len"], "source_error.source_len")?;
        let candidate_bytes = op.candidate_bytes();
        if source_len as usize != candidate_bytes.len() {
            return Err(format!(
                "source_error.source_len {source_len} does not match the parent's candidate ({} bytes)",
                candidate_bytes.len()
            ));
        }
        // Causes: non-empty, capped, exact fields, in-bounds spans.
        let causes = snap["causes"]
            .as_array()
            .ok_or("source_error.causes must be an array")?;
        if causes.is_empty() {
            return Err("source_error.causes must be non-empty".into());
        }
        if causes.len() > MAX_CAUSES {
            return Err(format!(
                "source_error.causes is {} (cap {MAX_CAUSES}) — over-cap is an error, never truncation",
                causes.len()
            ));
        }
        for (i, c) in causes.iter().enumerate() {
            let path = format!("source_error.causes[{i}]");
            obj_keys(c, &path, CAUSE_FIELDS)?;
            let stage = req_str(&c["stage"], &format!("{path}.stage"))?;
            if ![
                "text",
                "bind",
                "verify",
                "capability",
                "backend",
                "link",
                "run",
            ]
            .contains(&stage.as_str())
            {
                return Err(format!("{path}.stage {stage:?} is not a raw stage name"));
            }
            if !LANGUAGE_STAGES.contains(&stage.as_str()) {
                return Err(format!(
                    "{path}.stage {stage:?} cannot carry a language-rejection family"
                ));
            }
            req_str(&c["code"], &format!("{path}.code"))?;
            let message = req_str(&c["message"], &format!("{path}.message"))?;
            if message.as_bytes().len() > MAX_MESSAGE_UTF8 {
                return Err(format!(
                    "{path}.message is {} UTF-8 bytes (cap {MAX_MESSAGE_UTF8})",
                    message.as_bytes().len()
                ));
            }
            obj_keys(&c["span"], &format!("{path}.span"), SPAN_FIELDS)?;
            let start = req_u64(&c["span"]["start"], &format!("{path}.span.start"))?;
            let end = req_u64(&c["span"]["end"], &format!("{path}.span.end"))?;
            if start > end || end > source_len {
                return Err(format!(
                    "{path}.span [{start},{end}] is out of bounds (len {source_len})"
                ));
            }
            // utf8-byte basis: both ends must sit on char boundaries of the
            // candidate the compiler saw (a half-UTF-8 offset is a forged or
            // mis-based span, never a displayable position).
            if let Ok(text) = std::str::from_utf8(candidate_bytes) {
                if !text.is_char_boundary(start as usize) || !text.is_char_boundary(end as usize) {
                    return Err(format!(
                        "{path}.span [{start},{end}] does not sit on UTF-8 char boundaries of the candidate"
                    ));
                }
            }
        }
        // primary: the JSON INTEGER 0 — as_u64 rejects "0"/0.0/false.
        if req_u64(&snap["primary"], "source_error.primary")? != 0 {
            return Err("source_error.primary must be the integer 0".into());
        }
        // Payload cap on the admitted object.
        let payload = serde_json::to_string(snap).map_err(|e| format!("reserialize: {e}"))?;
        if payload.len() > MAX_PAYLOAD_UTF8 {
            return Err(format!(
                "source_error payload is {} UTF-8 bytes (cap {MAX_PAYLOAD_UTF8})",
                payload.len()
            ));
        }
        // Operation binding — the parent's own view decides.
        let operation = &snap["operation"];
        obj_keys(operation, "source_error.operation", OPERATION_FIELDS)?;
        let kind = req_str(&operation["kind"], "source_error.operation.kind")?;
        if !OPERATIONS.contains(&kind.as_str()) {
            return Err(format!("source_error.operation.kind {kind:?} is unknown"));
        }
        let candidate = req_str(
            &operation["candidate_sha256"],
            "source_error.operation.candidate_sha256",
        )?;
        match op {
            OperationExpectation::Cold {
                candidate: cbytes,
                step: want_step,
            } => {
                if kind != "cold" {
                    return Err(format!("expected a cold operation, got {kind:?}"));
                }
                if candidate != sha_hex(cbytes) {
                    return Err(
                        "source_error candidate SHA does not bind the parent's source".into(),
                    );
                }
                // A cold operation may name the step it answered (the rounds
                // protocol carries an internal index); the warm-session
                // binding stays null — there is no accepted prefix.
                for (k, v) in [
                    ("base_sha256", &operation["base_sha256"]),
                    ("session_revision", &operation["session_revision"]),
                    ("edit", &operation["edit"]),
                ] {
                    if !v.is_null() {
                        return Err(format!("cold operation must leave {k} null"));
                    }
                }
                // The step axis binds to the PARENT's view: null for a
                // request-level cold operation, the parent's own index for
                // a cold whole-request answered inside a rounds round.
                if operation["step"].as_u64() != *want_step
                    || (want_step.is_none() && !operation["step"].is_null())
                {
                    return Err(format!(
                        "cold operation step {} does not bind the parent's step {:?}",
                        operation["step"], want_step
                    ));
                }
            }
            OperationExpectation::WarmEdit {
                base: bbytes,
                candidate: cbytes,
                step: want_step,
            } => {
                if kind != "warm-edit" {
                    return Err(format!("expected a warm-edit operation, got {kind:?}"));
                }
                if candidate != sha_hex(cbytes) {
                    return Err(
                        "source_error candidate SHA does not bind the failing candidate".into(),
                    );
                }
                let base = req_str(
                    &operation["base_sha256"],
                    "source_error.operation.base_sha256",
                )?;
                if base != sha_hex(bbytes) {
                    return Err("source_error base SHA does not bind the accepted prefix".into());
                }
                if operation["step"].as_u64() != Some(*want_step) {
                    return Err(format!(
                        "warm-edit operation step {} does not bind the parent's step {want_step}",
                        operation["step"]
                    ));
                }
            }
            OperationExpectation::FallbackCold {
                base: bbytes,
                candidate: cbytes,
                step: want_step,
            } => {
                if kind != "fallback-cold" {
                    return Err(format!("expected a fallback-cold operation, got {kind:?}"));
                }
                if operation["step"].as_u64() != Some(*want_step) {
                    return Err(format!(
                        "fallback-cold operation step {} does not bind the parent's step {want_step}",
                        operation["step"]
                    ));
                }
                if candidate != sha_hex(cbytes) {
                    return Err(
                        "source_error candidate SHA does not bind the gate candidate".into(),
                    );
                }
                let base = req_str(
                    &operation["base_sha256"],
                    "source_error.operation.base_sha256",
                )?;
                if base != sha_hex(bbytes) {
                    return Err("source_error base SHA does not bind the accepted prefix".into());
                }
            }
        }
        // The snapshot's own source identity must equal the candidate the
        // compiler saw.
        if source_sha != candidate {
            return Err("source_error.source_sha256 must equal the operation candidate".into());
        }
        // Mutual exclusion (shape 2): no execution RESULT anywhere. Step
        // records carry the result keys structurally (null / empty on a
        // refusal), so the test is value-based: a non-null exit, non-empty
        // streams or a manifest fingerprint next to a source_error is a
        // forged mixed shape.
        if let Some(ok) = resp.get("ok").and_then(|v| v.as_object()) {
            if ok.get("exit_code").map(|v| !v.is_null()).unwrap_or(false) {
                return Err("a source_error response must not carry an execution exit_code".into());
            }
            for k in ["stdout_hex", "stderr_hex", "manifest_fp"] {
                if ok
                    .get(k)
                    .and_then(|v| v.as_str())
                    .map(|s| !s.is_empty())
                    .unwrap_or(false)
                {
                    return Err(format!(
                        "a source_error response must not carry the execution result {k}"
                    ));
                }
            }
        }
        // The legacy projection must agree with the primary cause.
        let cause0 = &causes[0];
        let stage0 = req_str(&cause0["stage"], "source_error.causes[0].stage")?;
        let code0 = req_str(&cause0["code"], "source_error.causes[0].code")?;
        let msg0 = req_str(&cause0["message"], "source_error.causes[0].message")?;
        let legacy = format!("{}/{}: {}", stage_debug_name(&stage0), code0, msg0);
        let diag_code = resp
            .get("diagnostic")
            .and_then(|d| d.get("code"))
            .and_then(|v| v.as_str())
            .ok_or("source_error response without the legacy diagnostic")?;
        if diag_code != "mn.unsupported.frontend" {
            return Err(format!(
                "a source_error response's legacy diagnostic must be mn.unsupported.frontend, got {diag_code}"
            ));
        }
        let diag_msg = resp
            .get("diagnostic")
            .and_then(|d| d.get("message"))
            .and_then(|v| v.as_str())
            .ok_or("legacy diagnostic without a message")?;
        if diag_msg != legacy {
            return Err("legacy diagnostic message is not the primary cause's projection".into());
        }
        let mapped_stage = match stage0.as_str() {
            "text" => Stage::Parse,
            "bind" => Stage::Resolve,
            _ => Stage::Typecheck,
        };
        Ok(AdmittedFact {
            snapshot: snap.clone(),
            raw_stage: stage0,
            mapped_stage,
            primary_code: code0,
            primary_message: msg0,
            source_sha256: source_sha,
            candidate_sha256: candidate,
            legacy_message: legacy,
        })
    }

    /// Write the parent's evidence artifact: the admitted fact (snapshot
    /// verbatim) plus the admission block bound to the check's execution
    /// key. A write failure is an error, never a green comparison.
    pub fn write_fact_artifact(
        artifact_dir: &std::path::Path,
        execution_key: &str,
        recipe: &str,
        worker_version: &str,
        fact: &AdmittedFact,
    ) -> Result<auto_test_case::report::ArtifactRef, String> {
        let doc = serde_json::json!({
            "schema": "mn-source-diagnostic-fact/1",
            "execution_key": execution_key,
            "admitted": true,
            "mode": MODE,
            "snapshot_schema": SNAPSHOT_SCHEMA,
            "recipe": recipe,
            "worker_version": worker_version,
            "source_sha256": fact.source_sha256,
            "candidate_sha256": fact.candidate_sha256,
            "raw_stage": fact.raw_stage,
            "observed_stage": format!("{:?}", fact.mapped_stage).to_lowercase(),
            "observed_code": fact.primary_code,
            "primary_message": fact.primary_message,
            "primary_index": 0,
            "snapshot": fact.snapshot,
        });
        let bytes = serde_json::to_vec_pretty(&doc).map_err(|e| format!("fact serialize: {e}"))?;
        let path = artifact_dir.join("mn-source-diagnostic.json");
        std::fs::write(&path, &bytes).map_err(|e| format!("write fact artifact: {e}"))?;
        // Re-read: the durable artifact must parse back.
        let reread: Value = serde_json::from_slice(
            &std::fs::read(&path).map_err(|e| format!("re-read fact: {e}"))?,
        )
        .map_err(|e| format!("re-read fact parse: {e}"))?;
        if reread["execution_key"] != execution_key || reread["snapshot"] != fact.snapshot {
            return Err("fact artifact re-read mismatch".into());
        }
        use auto_test_case::sha256::{hex, sha256};
        Ok(auto_test_case::report::ArtifactRef {
            logical_path: "mn-source-diagnostic.json".to_string(),
            stored_path: path.display().to_string(),
            sha256: hex(&sha256(&bytes)),
            bytes: Some(bytes.len()),
        })
    }
}

/// PLAN-868: admission mirror of `ace_lowering::MIR_LOWERING_VERSION` —
/// the mn pipeline lowers through the shared crate; a lowering contract
/// bump without a mirror update fails the identity axis loudly.
pub const EXPECTED_MIR_LOWERING_VERSION: &str = "4";

pub fn current_toolchain_identity() -> std::collections::BTreeMap<String, String> {
    let mut m = std::collections::BTreeMap::new();
    if let Ok(binary) = worker_binary() {
        if let Ok(sha) = worker_file_sha256(&binary) {
            m.insert("mn-worker-exe-sha256".to_string(), sha);
        }
    }
    m.insert("recipe".to_string(), MN_RECIPE.to_string());
    m.insert(
        "mir-lowering".to_string(),
        EXPECTED_MIR_LOWERING_VERSION.to_string(),
    );
    m
}

fn runtime_toolchain_axes(runtime: &serde_json::Value) -> Vec<(String, String)> {
    [
        ("collections-runtime-abi", "abi"),
        ("collections-runtime-build", "build_key"),
        ("collections-runtime-content", "content_hash"),
    ]
    .into_iter()
    .filter_map(|(axis, key)| {
        runtime["runtime"]
            .get(key)
            .and_then(|value| value.as_str())
            .map(|value| (axis.into(), value.into()))
    })
    .collect()
}
fn link_toolchain_axes(link: &serde_json::Value) -> Vec<(String, String)> {
    [
        "collections-lld-content",
        "collections-sdk-version",
        "collections-kernel32-content",
    ]
    .into_iter()
    .enumerate()
    .filter_map(|(index, key)| {
        link.get(index)
            .and_then(|value| value.as_str())
            .map(|value| (key.into(), value.into()))
    })
    .collect()
}
fn native_toolchain(
    recipe: &str,
    exe_sha: String,
    runtime: Option<&serde_json::Value>,
) -> Vec<(String, String)> {
    let mut axes = vec![
        ("recipe".into(), recipe.into()),
        ("mir-lowering".into(), EXPECTED_MIR_LOWERING_VERSION.into()),
        ("mn-worker-exe-sha256".into(), exe_sha),
    ];
    if recipe == COLLECTIONS_RECIPE {
        if let Some(runtime) = runtime {
            axes.extend(runtime_toolchain_axes(runtime));
            axes.extend(link_toolchain_axes(&runtime["link"]));
        }
    }
    axes
}
fn bound_worker_sha(recipe: &str) -> Result<String, String> {
    let binary = worker_binary_for(recipe)?;
    if recipe == COLLECTIONS_RECIPE {
        Ok(hex(&sha256(
            &std::fs::read(binary).map_err(|e| e.to_string())?,
        )))
    } else {
        worker_file_sha256(&binary)
    }
}
/// PLAN-891: the typed recipe's current toolchain identity axes (report
/// staleness binds these per row).
pub fn current_typed_toolchain_identity() -> std::collections::BTreeMap<String, String> {
    let mut axes = std::collections::BTreeMap::from([
        ("recipe".into(), TYPED_RECIPE.into()),
        ("mir-lowering".into(), EXPECTED_MIR_LOWERING_VERSION.into()),
        ("mn-typed-abi".into(), typed::EXPECTED_TYPED_ABI.into()),
    ]);
    if let Ok(hash) = bound_worker_sha(TYPED_RECIPE) {
        axes.insert("mn-worker-exe-sha256".into(), hash);
    }
    axes
}

pub fn current_collections_toolchain_identity() -> std::collections::BTreeMap<String, String> {
    let mut axes = std::collections::BTreeMap::from([
        ("recipe".into(), COLLECTIONS_RECIPE.into()),
        ("mir-lowering".into(), EXPECTED_MIR_LOWERING_VERSION.into()),
    ]);
    if let Ok(hash) = bound_worker_sha(COLLECTIONS_RECIPE) {
        axes.insert("mn-worker-exe-sha256".into(), hash);
    }
    let mut request = serde_json::json!({"recipe":COLLECTIONS_RECIPE,"nonce":"collections-current-identity","mode":"identity"});
    let hash = hex(&sha256(request.to_string().as_bytes()));
    request["request_hash"] = serde_json::json!(hash);
    if let Ok(line) = call_worker(&request.to_string()) {
        if let Ok(response) = serde_json::from_str::<serde_json::Value>(&line) {
            if response["nonce"] == request["nonce"]
                && response["request_hash"] == request["request_hash"]
                && response["schema"] == MN_WORKER_PROTOCOL
                && response["ok"]["recipe"] == COLLECTIONS_RECIPE
            {
                axes.extend(runtime_toolchain_axes(
                    &serde_json::json!({"runtime":response["ok"]["runtime_identity"]}),
                ));
                axes.extend(link_toolchain_axes(&response["ok"]["link_identity"]));
            }
        }
    }
    axes
}

fn worker_file_sha256(exe: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    // Memoized per (path, len): the turnaround path hashes the worker exe
    // per receipt; a same-process repeat is the same evidence.
    use std::sync::{Mutex, OnceLock};
    static MEMO: OnceLock<Mutex<std::collections::HashMap<(PathBuf, u64), String>>> =
        OnceLock::new();
    let meta = std::fs::metadata(exe).map_err(|e| format!("stat worker exe: {e}"))?;
    let key = (exe.to_path_buf(), meta.len());
    let memo = MEMO.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
    if let Some(h) = memo.lock().unwrap().get(&key) {
        return Ok(h.clone());
    }
    let bytes = std::fs::read(exe).map_err(|e| format!("read worker exe: {e}"))?;
    let h = hex(&Sha256::digest(&bytes));
    memo.lock().unwrap().insert(key, h.clone());
    Ok(h)
}

/// Locate (and on first use build) the mn worker binary. Env override
/// first (read on EVERY call — the fault-injection test seam must stay
/// injectable after the real binary was memoized); otherwise find
/// `experimental/ace/crates/native` above this crate's source root and build
/// its worker release under the contained, bounded build discipline (same
/// shape as the av worker build).
pub fn worker_binary() -> Result<PathBuf, String> {
    if std::env::var("AUTO_TEST_DOMAIN").as_deref() == Ok("legacy") {
        return Err(
            "ACE worker requested by a legacy test group; explicitly select the ACE group".into(),
        );
    }
    if let Ok(env_path) = std::env::var("AC_MN_CASE_WORKER") {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!(
            "AC_MN_CASE_WORKER points at a missing file: {}",
            env_path
        ));
    }
    static CACHE: std::sync::OnceLock<Result<PathBuf, String>> = std::sync::OnceLock::new();
    CACHE
        .get_or_init(|| locate_worker_exn().map_err(|e| e))
        .clone()
}

fn locate_worker_exn() -> Result<PathBuf, String> {
    let mut dir: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    let manifest = loop {
        let Some(d) = dir else {
            return Err(
                "mn worker not located: no experimental/ace/crates/native above the crate and no AC_MN_CASE_WORKER override".to_string(),
            );
        };
        let m = d.join("experimental/ace/crates/native").join("Cargo.toml");
        if m.is_file() {
            break m;
        }
        dir = d.parent();
    };
    build_worker(&manifest)
}

fn build_worker(manifest: &Path) -> Result<PathBuf, String> {
    build_worker_named(manifest, "mn-worker")
}
fn build_worker_named(manifest: &Path, bin: &str) -> Result<PathBuf, String> {
    let ws_root = manifest.parent().expect("workspace root");
    let exe = ws_root
        .join("../../target")
        .join("release")
        .join(format!("{bin}.exe"));
    // ALWAYS build (no-op incremental builds are sub-second) under the
    // bounded wall — a stale binary silently unbinds evidence from source.
    let cargo = std::env::var("AC_MN_BUILD_CARGO").unwrap_or_else(|_| env!("CARGO").to_string());
    let mut cmd = Command::new(&cargo);
    cmd.args(["build", "--release", "-p", "ace-native", "--bin", bin])
        .current_dir(ws_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(containment::CREATE_SUSPENDED);
    }
    let build_started = Instant::now();
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn cargo for the mn worker: {e}"))?;
    #[cfg(windows)]
    let mut job = match containment::JobGuard::for_child(&child) {
        Some(j) => Some(j),
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(
                "containment unavailable: refusing an uncontained mn worker build tree".to_string(),
            );
        }
    };
    #[cfg(windows)]
    if let Err(e) = containment::resume_suspended_process(&child) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("resume cargo build: {e}"));
    }
    #[cfg(not(windows))]
    let mut job: Option<containment::JobGuard> = None;
    // Drain both pipes on reader threads so a full pipe surfaces instead
    // of deadlocking the build (capped; the tail is diagnostics only).
    fn drain_thread<R: Read + Send + 'static>(
        pipe: Option<R>,
    ) -> Option<std::thread::JoinHandle<()>> {
        pipe.map(|mut s| {
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 8192];
                loop {
                    match s.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            buf.extend_from_slice(&chunk[..n]);
                            if buf.len() > STDERR_CAP {
                                break;
                            }
                        }
                    }
                }
            })
        })
    }
    let mut out_t = drain_thread(child.stdout.take());
    let mut err_t = drain_thread(child.stderr.take());
    let _ = &mut job;
    loop {
        if build_started.elapsed() > BUILD_WALL {
            let _ = child.kill();
            drop(job.take());
            let _ = child.wait();
            return Err("mn worker build exceeded the bounded wall".to_string());
        }
        match child.try_wait() {
            Ok(Some(st)) => {
                drop(job.take());
                if let Some(t) = out_t.take() {
                    let _ = t.join();
                }
                if let Some(t) = err_t.take() {
                    let _ = t.join();
                }
                if !st.success() {
                    return Err(format!("mn worker build exited {st:?}"));
                }
                break;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(e) => return Err(format!("poll mn worker build: {e}")),
        }
    }
    if !exe.is_file() {
        return Err(format!(
            "mn worker binary missing after build: {}",
            exe.display()
        ));
    }
    Ok(exe)
}

fn request_recipe(request: &str) -> &'static str {
    // The collections WORKER serves the collections recipes (the lifecycle
    // and resource-guards recipes ride the same runtime/ABI/observe
    // channel — PLAN-919/923).
    if serde_json::from_str::<serde_json::Value>(request)
        .ok()
        .and_then(|v| {
            v.get("recipe").and_then(|r| r.as_str()).map(|r| {
                r == COLLECTIONS_RECIPE
                    || r == RESOURCE_EVENTS_RECIPE
                    || r == RESOURCE_GUARDS_RECIPE
                    || r == CLOSURES_RESOURCE_RECIPE
            })
        })
        .unwrap_or(false)
    {
        COLLECTIONS_RECIPE
    } else {
        MN_RECIPE
    }
}
fn worker_binary_for(recipe: &str) -> Result<PathBuf, String> {
    if std::env::var("AUTO_TEST_DOMAIN").as_deref() == Ok("legacy") {
        return Err(
            "ACE worker requested by a legacy fixed test group; select an explicit ACE group"
                .into(),
        );
    }
    // PLAN-919/923: the lifecycle and resource-guards recipes ride the
    // collections worker (same runtime/ABI/observe channel).
    if recipe != COLLECTIONS_RECIPE
        && recipe != RESOURCE_EVENTS_RECIPE
        && recipe != RESOURCE_GUARDS_RECIPE
        && recipe != CLOSURES_RESOURCE_RECIPE
    {
        return worker_binary();
    }
    if let Ok(path) = std::env::var("AC_MN_COLLECTIONS_CASE_WORKER") {
        let path = PathBuf::from(path);
        return if path.is_file() {
            Ok(path)
        } else {
            Err("collections worker override is missing".into())
        };
    }
    static CACHE: std::sync::OnceLock<Result<PathBuf, String>> = std::sync::OnceLock::new();
    CACHE
        .get_or_init(|| {
            let mut dir = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
            while let Some(root) = dir {
                let manifest = root.join("experimental/ace/crates/native/Cargo.toml");
                if manifest.is_file() {
                    return build_worker_named(&manifest, "mn-collections-worker");
                }
                dir = root.parent();
            }
            Err("collections worker manifest not located".into())
        })
        .clone()
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// One contained worker call: spawn suspended → Job → resume → write the
/// request line → read to EOF (exactly one protocol line) → exit; a
/// single absolute deadline over the whole session, tree-killed on
/// breach. Returns the response line.
fn call_worker(request: &str) -> Result<String, String> {
    call_worker_observed(request, None)
}

fn call_worker_observed(request: &str, artifact_dir: Option<&Path>) -> Result<String, String> {
    let exe = worker_binary_for(request_recipe(request))?;
    let exchange = match artifact_dir {
        Some(directory) if super::execution_trace::WorkerExchange::enabled()? => {
            // Worker routing coalesces several recipes onto one binary; the
            // evidence identity must retain the exact original request recipe.
            let request_value: serde_json::Value = serde_json::from_str(request)
                .map_err(|e| format!("native evidence request: {e}"))?;
            let evidence_recipe = request_value
                .get("recipe")
                .and_then(|v| v.as_str())
                .ok_or("native evidence request has no recipe")?;
            super::execution_trace::WorkerExchange::start(
                directory,
                MN_WORKER_PROTOCOL,
                evidence_recipe,
                &worker_file_sha256(&exe)?,
                request.as_bytes(),
            )?
        }
        _ => None,
    };
    let mut cmd = Command::new(&exe);
    cmd.arg("case-run")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(containment::CREATE_SUSPENDED);
    }
    let started = Instant::now();
    let mut child = cmd.spawn().map_err(|e| format!("spawn mn worker: {e}"))?;
    #[cfg(windows)]
    let mut job = match containment::JobGuard::for_child(&child) {
        Some(j) => Some(j),
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err("containment unavailable: refusing an uncontained mn worker".into());
        }
    };
    #[cfg(windows)]
    if let Err(e) = containment::resume_suspended_process(&child) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("resume mn worker: {e}"));
    }
    #[cfg(not(windows))]
    let mut job: Option<containment::JobGuard> = None;
    // Request writer + capped stdout/stderr readers.
    let mut stdin = child.stdin.take();
    let req = request.to_string();
    let write_t = std::thread::spawn(move || {
        if let Some(s) = stdin.as_mut() {
            let _ = s.write_all(req.as_bytes());
            let _ = s.write_all(b"\n");
            let _ = s.flush();
        }
    });
    fn cap_reader(
        r: impl Read + Send + 'static,
        cap: usize,
    ) -> std::thread::JoinHandle<(bool, Vec<u8>)> {
        std::thread::spawn(move || {
            let mut r = r;
            let mut buf = Vec::new();
            let mut exceeded = false;
            let mut chunk = [0u8; 8192];
            loop {
                match r.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        buf.extend_from_slice(&chunk[..n]);
                        if buf.len() > cap {
                            exceeded = true;
                            break;
                        }
                    }
                }
            }
            (exceeded, buf)
        })
    }
    let out_pipe = child.stdout.take().ok_or("worker stdout pipe")?;
    let err_pipe = child.stderr.take().ok_or("worker stderr pipe")?;
    let out_t = cap_reader(out_pipe, READ_CAP);
    let err_t = cap_reader(err_pipe, STDERR_CAP);

    let deadline = started + SESSION_WALL;
    let mut timed_out = false;
    let mut exit_status = None;
    loop {
        if Instant::now() > deadline {
            timed_out = true;
            break;
        }
        match child.try_wait() {
            Ok(st @ Some(_)) => {
                exit_status = st;
                break;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(2)),
            Err(e) => return Err(format!("poll mn worker: {e}")),
        }
    }
    if timed_out {
        let _ = child.kill();
        drop(job.take());
        let _ = child.wait();
        let _ = write_t.join();
        let _ = out_t.join();
        let _ = err_t.join();
        return Err("mn worker exceeded the session wall (single absolute deadline)".into());
    }
    drop(job.take());
    let _ = write_t.join();
    let (out_over, out_buf) = out_t.join().map_err(|_| "stdout reader panicked")?;
    let (err_over, err_buf) = err_t.join().map_err(|_| "stderr reader panicked")?;
    let status = exit_status.unwrap_or_else(|| child.wait().unwrap_or_default());
    if let Some(exchange) = exchange {
        exchange.finish(&out_buf, &err_buf, status.code(), None)?;
    }
    if out_over {
        return Err(format!("mn worker stdout exceeded the {READ_CAP}-byte cap"));
    }
    if err_over {
        return Err(format!(
            "mn worker stderr exceeded the {STDERR_CAP}-byte cap; stderr: {:.300}",
            String::from_utf8_lossy(&err_buf)
        ));
    }
    if !status.success() {
        return Err(format!(
            "mn worker exited {:?} (non-zero worker exits are infra, never a verdict); stderr: {:.300}",
            status.code(),
            String::from_utf8_lossy(&err_buf)
        ));
    }
    let text = String::from_utf8_lossy(&out_buf).to_string();
    let non_empty: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if non_empty.len() != 1 {
        return Err(format!(
            "mn worker response must be exactly one non-empty protocol line, saw {} ({:.200})",
            non_empty.len(),
            text
        ));
    }
    Ok(non_empty[0].to_string())
}

/// Exactly-once literal replacement for an `edit-source` step (the
/// discriminating-edit oracle demands a unique match — ambiguity is a
/// contract error, never a silent first-hit edit).
fn replace_once(src: &[u8], from: &str, to: &str) -> Result<Vec<u8>, String> {
    let s = std::str::from_utf8(src).map_err(|e| format!("source not utf-8: {e}"))?;
    let hits = s.matches(from).count();
    if hits != 1 {
        return Err(format!(
            "edit-source literal matched {hits} times (expected exactly once)"
        ));
    }
    Ok(s.replacen(from, to, 1).into_bytes())
}

/// One line-read outcome from a ROUNDS worker's stdout.
enum RoundRead {
    Line(String),
    Closed,
    ReadErr(String),
}

/// The ROUNDS session's absolute wall: the frozen default, overridable to
/// a short budget by the liveness fixtures (AC_MN_ROUNDS_WALL_SECS).
fn rounds_wall() -> Duration {
    std::env::var("AC_MN_ROUNDS_WALL_SECS")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .filter(|s| *s >= 1)
        .map(Duration::from_secs)
        .unwrap_or(SESSION_WALL)
}

/// A persistent ROUNDS mn-worker session (F-818-R1-01): the parent writes
/// ONE request line per step and reads ONE response line per step, so the
/// parent observes the CONTINUOUS per-edit interval — its own request
/// preparation (source copy + literal search/insert + payload
/// serialization), IPC both ways, the worker's locate/edit/link/run,
/// response serialization, and its own admission of (and independent
/// comparison for) the round's result.
///
/// Failure liveness and IPC bounds (F-818-R3-01/R3-02): every blocking
/// operation lives on a session-owned thread; the parent only ever
/// (a) posts a line to the writer's mailbox (bounded by the step count —
/// a stuck pipe can never block the parent), (b) receives a response
/// under the ONE absolute session deadline, and (c) reaps via a
/// deadline-governed poll — never a bare blocking `wait`. Stdout is
/// capped DURING accumulation (a newline-free or oversized stream trips
/// the cap before unbounded allocation) and the response queue is
/// bounded (an unsolicited flood trips the overflow flag). A stderr cap
/// breach sets a flag checked on every receive and at finish, so a
/// cap-violating worker can never produce an apparently-valid verdict.
/// Contained (suspended spawn → Job → resume; the guard is held for the
/// whole session); teardown kills the tree first, then joins every
/// session-owned thread — pipes close with the tree, so no handle leaks.
struct RoundsWorker {
    child: std::process::Child,
    job: Option<containment::JobGuard>,
    tx_lines: Option<std::sync::mpsc::Sender<String>>,
    writer: Option<std::thread::JoinHandle<()>>,
    reader: Option<std::thread::JoinHandle<()>>,
    stderr_t: Option<std::thread::JoinHandle<()>>,
    stderr_done: std::sync::mpsc::Receiver<(bool, Vec<u8>)>,
    stdout_over: std::sync::Arc<std::sync::atomic::AtomicBool>,
    stderr_over: std::sync::Arc<std::sync::atomic::AtomicBool>,
    rx: std::sync::mpsc::Receiver<RoundRead>,
    started: Instant,
    deadline: Instant,
}

impl RoundsWorker {
    /// Spawn the contained worker and return the session (RAII held from
    /// BEFORE the first byte is written) plus the warmup response line.
    fn start(request: &str) -> Result<(RoundsWorker, String), String> {
        let exe = worker_binary_for(request_recipe(request))?;
        let mut cmd = Command::new(&exe);
        cmd.arg("case-run")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(containment::CREATE_SUSPENDED);
        }
        let started = Instant::now();
        let mut child = cmd.spawn().map_err(|e| format!("spawn mn worker: {e}"))?;
        #[cfg(windows)]
        let job = match containment::JobGuard::for_child(&child) {
            Some(j) => Some(j),
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("containment unavailable: refusing an uncontained mn worker".into());
            }
        };
        #[cfg(windows)]
        if let Err(e) = containment::resume_suspended_process(&child) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("resume mn worker: {e}"));
        }
        #[cfg(not(windows))]
        let job: Option<containment::JobGuard> = None;
        // Session-owned threads (joined at teardown — never leaked):
        // - writer: owns stdin; the parent only posts to its mailbox, so a
        //   full pipe can never block the parent.
        // - reader: owns stdout; caps DURING accumulation and posts into a
        //   BOUNDED queue (unsolicited floods trip the overflow flag).
        // - stderr: caps at STDERR_CAP and reports the breach on a flag the
        //   parent checks on every receive and at finish.
        let mut stdin = child.stdin.take().ok_or("worker stdin pipe")?;
        let (tx_lines, rx_lines) = std::sync::mpsc::channel::<String>();
        let writer = std::thread::spawn(move || {
            while let Ok(payload) = rx_lines.recv() {
                let flushed = stdin
                    .write_all(payload.as_bytes())
                    .and_then(|_| stdin.flush());
                if flushed.is_err() {
                    break;
                }
            }
        });
        let out_pipe = child.stdout.take().ok_or("worker stdout pipe")?;
        let (tx, rx) = std::sync::mpsc::sync_channel::<RoundRead>(8);
        let stdout_over = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stdout_flag = std::sync::Arc::clone(&stdout_over);
        let reader = std::thread::spawn(move || {
            let mut r = out_pipe;
            let mut pending: Vec<u8> = Vec::new();
            // The newline scan resumes here (a multi-megabyte valid line
            // must not be rescanned from zero per chunk).
            let mut scanned = 0usize;
            let mut chunk = [0u8; 8192];
            loop {
                match r.read(&mut chunk) {
                    Ok(0) => {
                        let _ = tx.send(RoundRead::Closed);
                        break;
                    }
                    Err(e) => {
                        let _ = tx.send(RoundRead::ReadErr(format!("{e}")));
                        break;
                    }
                    Ok(n) => {
                        pending.extend_from_slice(&chunk[..n]);
                        // The cap applies DURING accumulation: a
                        // newline-free or oversized stream trips it before
                        // the pending buffer can grow past the bound.
                        if pending.len() > READ_CAP {
                            stdout_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                            let _ = tx.try_send(RoundRead::Closed);
                            break;
                        }
                        let mut stop = false;
                        while let Some(rel) = pending[scanned..].iter().position(|&b| b == b'\n') {
                            let pos = scanned + rel;
                            let line = String::from_utf8_lossy(&pending[..pos]).into_owned();
                            pending.drain(..=pos);
                            scanned = 0;
                            if line.trim().is_empty() {
                                continue;
                            }
                            // Bounded queue: an unsolicited flood fills the
                            // channel and trips the flag instead of
                            // allocating without bound.
                            if tx.try_send(RoundRead::Line(line)).is_err() {
                                stdout_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                                stop = true;
                                break;
                            }
                        }
                        if stop {
                            break;
                        }
                    }
                }
            }
        });
        let err_pipe = child.stderr.take().ok_or("worker stderr pipe")?;
        let stderr_over = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stderr_flag = std::sync::Arc::clone(&stderr_over);
        let (tx_err, stderr_done) = std::sync::mpsc::channel::<(bool, Vec<u8>)>();
        let stderr_t = std::thread::spawn(move || {
            let mut s = err_pipe;
            let mut buf = Vec::new();
            let mut chunk = [0u8; 8192];
            let mut exceeded = false;
            loop {
                match s.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        buf.extend_from_slice(&chunk[..n]);
                        if buf.len() > STDERR_CAP {
                            exceeded = true;
                            stderr_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                            break;
                        }
                    }
                }
            }
            let _ = tx_err.send((exceeded, buf));
        });
        let mut worker = RoundsWorker {
            child,
            job,
            tx_lines: Some(tx_lines),
            writer: Some(writer),
            reader: Some(reader),
            stderr_t: Some(stderr_t),
            stderr_done,
            stdout_over,
            stderr_over,
            rx,
            started,
            deadline: started + rounds_wall(),
        };
        // RAII is held from here on; the initial write goes through the
        // session-owned writer (a blocked write cannot block the parent —
        // the warmup receive under the absolute wall governs it).
        worker.send_line(request)?;
        let warm = worker.read_line()?;
        Ok((worker, warm))
    }

    /// Post one line to the writer's mailbox (never blocks on the pipe;
    /// a stuck writer is caught by the response deadline, not by us).
    fn send_line(&mut self, line: &str) -> Result<(), String> {
        if Instant::now() > self.deadline {
            return Err(
                "mn rounds session exceeded the session wall (single absolute deadline)".into(),
            );
        }
        let Some(tx) = self.tx_lines.as_ref() else {
            return Err("mn rounds writer already stopped".into());
        };
        let mut payload = String::with_capacity(line.len() + 1);
        payload.push_str(line);
        payload.push('\n');
        tx.send(payload)
            .map_err(|_| "mn rounds writer already stopped".to_string())
    }

    fn read_line(&self) -> Result<String, String> {
        let Some(timeout) = self.deadline.checked_duration_since(Instant::now()) else {
            return Err(
                "mn rounds session exceeded the session wall (single absolute deadline)".into(),
            );
        };
        let got = self.rx.recv_timeout(timeout);
        // Cap propagation first: a cap-violating worker never yields an
        // apparently-valid line, even when stdout itself stays well-formed.
        if self.stdout_over.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(format!(
                "mn rounds worker stdout exceeded the {READ_CAP}-byte cap"
            ));
        }
        if self.stderr_over.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(format!(
                "mn rounds worker stderr exceeded the {STDERR_CAP}-byte cap"
            ));
        }
        match got {
            Ok(RoundRead::Line(l)) => Ok(l),
            Ok(RoundRead::Closed) => Err("mn rounds worker closed its output early".into()),
            Ok(RoundRead::ReadErr(e)) => Err(format!("mn rounds worker stdout read: {e}")),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                Err("mn rounds session exceeded the session wall (single absolute deadline)".into())
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                Err("mn rounds worker output ended without a response".into())
            }
        }
    }

    /// One round: post the step line, receive the response line.
    fn round(&mut self, line: &str) -> Result<String, String> {
        self.send_line(line)?;
        self.read_line()
    }

    /// Deadline-governed reap — never a bare blocking `wait` (a worker
    /// that answers every round but refuses to exit still hits the wall).
    fn reap_until_deadline(&mut self) -> Result<std::process::ExitStatus, String> {
        loop {
            if Instant::now() > self.deadline {
                return Err(
                    "mn rounds worker did not exit within the session wall (single absolute deadline)"
                        .into(),
                );
            }
            match self.child.try_wait() {
                Ok(Some(st)) => return Ok(st),
                Ok(None) => std::thread::sleep(Duration::from_millis(5)),
                Err(e) => return Err(format!("poll mn rounds worker: {e}")),
            }
        }
    }

    /// The graceful close handshake under the absolute wall: close line →
    /// close response → deadline-governed reap → stderr cap verdict.
    fn finish(&mut self) -> Result<(), String> {
        let handshake = self
            .send_line("{\"op\":\"close\"}")
            .and_then(|_| self.read_line().map(|_| ()));
        let reap = self.reap_until_deadline().and_then(|st| {
            if st.success() {
                Ok(())
            } else {
                Err(format!("mn rounds worker exited {st:?}"))
            }
        });
        // The tree is gone (reaped, or teardown below kills it): the
        // pipes are closed, so the stderr report arrives promptly.
        let (err_exceeded, err_tail) = self
            .stderr_done
            .recv_timeout(Duration::from_secs(2))
            .unwrap_or((false, Vec::new()));
        let mut failures = Vec::new();
        if let Err(e) = &handshake {
            failures.push(format!("close handshake: {e}"));
        }
        if let Err(e) = &reap {
            failures.push(format!("reap: {e}"));
        }
        if err_exceeded {
            failures.push(format!(
                "worker stderr exceeded the {STDERR_CAP}-byte cap; tail: {:.200}",
                String::from_utf8_lossy(&err_tail)
            ));
        }
        self.teardown();
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures.join("; "))
        }
    }

    /// Kill the tree FIRST (closing every pipe), then join all
    /// session-owned threads — bounded because the closed pipes end their
    /// blocking reads/writes. No orphan readers or writers.
    fn teardown(&mut self) {
        self.tx_lines.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        drop(self.job.take());
        if let Some(t) = self.writer.take() {
            let _ = t.join();
        }
        if let Some(t) = self.reader.take() {
            let _ = t.join();
        }
        if let Some(t) = self.stderr_t.take() {
            let _ = t.join();
        }
    }

    fn worker_us(&self) -> u64 {
        self.started.elapsed().as_micros() as u64
    }
}

impl Drop for RoundsWorker {
    fn drop(&mut self) {
        self.teardown();
    }
}

/// One executed session's parent-side record (receipt evidence).
struct SessionRecord {
    steps: Vec<String>,
    source_sha: String,
    exit_code: Option<i32>,
    manifest_fp: Option<String>,
    counters: Vec<(String, u32)>,
    spans: Vec<(String, u64)>,
    worker_us: u64,
    judged: bool,
}

pub fn run_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    _project_dir: &Path,
    artifact_dir: &Path,
    _run_dir: &Path,
    execution_key: &str,
) -> Result<Verdict, String> {
    let recipe = check
        .options
        .probe_recipe
        .as_deref()
        .ok_or("missing native recipe")?;
    if recipe == TYPED_RECIPE {
        // PLAN-891: the typed recipe consumes the file-ized fixture
        // (mn-typed-fixture/1), not .at source text.
        return typed::run_check_typed(case, check, oracle, artifact_dir);
    }
    let started = Instant::now();
    if case.spec.source.syntax != auto_test_case::contract::Syntax::Auto {
        return Err(format!(
            "{recipe} consumes .at sources; syntax {:?} is not accepted by this recipe",
            case.spec.source.syntax.as_str()
        ));
    }
    let mn = check
        .options
        .probe_mn
        .as_ref()
        .ok_or("probe check without an mn binding (contract violation)")?;
    let steps = check
        .options
        .probe_steps
        .as_ref()
        .ok_or("probe check without steps (contract violation)")?;
    if !matches!(steps.last(), Some(auto_test_case::contract::ProbeStep::Run)) {
        return Err(format!(
            "{recipe} step sequences must end with a run step (contract violation)"
        ));
    }
    let input_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or_else(|| "probe adapter needs the resolved entry input".to_string())?;
    // Store root (T-07 turnaround shape, the 798 session form): with an
    // explicit SESSION id the store lives under a deterministic shared
    // temp root keyed by that id, so calls of the same session (warm-up +
    // measured edit) share one warm store across worker processes; the
    // PARENT owns cleanup. Without a session, the check's own artifact
    // dir hosts a private per-request store.
    // PLAN-911: the strict source-diagnostic observation mode (mn.diagnostics
    // = "source-v1", decoder-validated). Everything below admits the typed
    // fact BEFORE any oracle comparison when set.
    let strict = mn.diagnostics.as_deref() == Some(source_diag::MODE);
    let store_root = match &mn.session {
        Some(id) => {
            // The session scratch's PARENT is the adapter host: the
            // default is the system temp; AC_MN_SESSION_ROOT (a directory)
            // relocates session stores (the turnaround host uses the
            // reference machine's build disk — same evidence, faster
            // store I/O).
            match std::env::var("AC_MN_SESSION_ROOT") {
                Ok(base) if std::path::Path::new(&base).is_dir() => {
                    std::path::PathBuf::from(base).join(format!("mn818-session-{id}"))
                }
                _ => std::env::temp_dir().join(format!("mn818-session-{id}")),
            }
        }
        None => artifact_dir.join("mn-store"),
    };
    std::fs::create_dir_all(&store_root).map_err(|e| format!("mn store root: {e}"))?;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("mn artifact dir: {e}"))?;
    let workdir = artifact_dir.join("mn-work");

    // ---- T-06 session split: `restart` ends a session (each session is a
    // REAL new worker process; the on-disk store is the carried state);
    // a session's leading edits materialize parent-side before its run.
    let mut sessions: Vec<Vec<&auto_test_case::contract::ProbeStep>> = Vec::new();
    let mut current: Vec<&auto_test_case::contract::ProbeStep> = Vec::new();
    for st in steps.iter() {
        let is_restart = matches!(st, auto_test_case::contract::ProbeStep::Restart);
        current.push(st);
        if is_restart {
            sessions.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        sessions.push(current);
    }
    // F-818-R1-03: warm edit sequences (edits, no restart boundary) run
    // through the D-828 incremental session INSIDE the worker — single
    // warm-up, single-body edit transactions, per-step real link/run.
    // Restart-mixed sequences stay on the whole-request session model
    // (cross-process = the cold scenario; its warm edits are not
    // incremental and are not labeled so).
    let has_edit_step = steps.iter().any(|s| {
        matches!(
            s,
            auto_test_case::contract::ProbeStep::EditSource { .. }
                | auto_test_case::contract::ProbeStep::DeclarationInvalidate { .. }
        )
    });
    let has_restart = steps
        .iter()
        .any(|s| matches!(s, auto_test_case::contract::ProbeStep::Restart));
    if has_edit_step && !has_restart {
        return run_incremental(case, check, oracle, artifact_dir, execution_key, mn, steps);
    }
    let total_runs = steps
        .iter()
        .filter(|s| matches!(s, auto_test_case::contract::ProbeStep::Run))
        .count();
    let mut run_index = 0usize; // 0-based index of executed RUN steps

    let mut source = input_bytes.clone();
    let mut mismatches: Vec<Mismatch> = Vec::new();
    let mut infra_detail: Option<String> = None;
    let mut diag: Option<(String, String)> = None;
    // PLAN-911: the admitted typed fact of the judged session (if any).
    let mut source_fact: Option<source_diag::AdmittedFact> = None;
    let mut exit_code: Option<i32> = None;
    let mut stderr_hex = String::new();
    let mut session_records: Vec<SessionRecord> = Vec::new();
    // Parent-side manifest identity admission: source-hash → manifest fp.
    let mut collection_runtime: Option<serde_json::Value> = None;
    let mut collection_ids: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    let mut manifest_by_source: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();

    'sessions: for session in sessions.iter() {
        // Materialize this session's leading edits parent-side.
        let mut step_labels = Vec::new();
        for st in session.iter() {
            match st {
                auto_test_case::contract::ProbeStep::EditSource { from, to } => {
                    source = replace_once(&source, from, to)?;
                    step_labels.push("edit-source".to_string());
                }
                auto_test_case::contract::ProbeStep::Restart => {
                    step_labels.push("restart".to_string());
                }
                auto_test_case::contract::ProbeStep::Run => step_labels.push("run".to_string()),
                // PLAN-866: resident ops are ac-av/resident-run-v1 only.
                auto_test_case::contract::ProbeStep::ResidentSlice { .. }
                | auto_test_case::contract::ProbeStep::ResidentApply
                | auto_test_case::contract::ProbeStep::ResidentFinish => {
                    return Err(format!(
                        "{recipe} does not accept resident steps (decode contract violation)"
                    ));
                }
                auto_test_case::contract::ProbeStep::Query => {
                    return Err(format!(
                        "{recipe} does not take query steps (decode contract violation)"
                    ));
                }
                // PLAN-914 (I04): declaration rounds belong to the
                // incremental rounds path, never the restart-session path.
                auto_test_case::contract::ProbeStep::PartialRebuild { .. } => {
                    return Err(format!(
                        "{recipe} does not take partial-rebuild steps in the restart-session path"
                    ));
                }
                auto_test_case::contract::ProbeStep::DeclarationInvalidate { .. } => {
                    return Err(format!(
                        "{recipe} does not take declaration rounds in the restart-session path"
                    ));
                }
            }
        }
        let has_run = session
            .iter()
            .any(|s| matches!(s, auto_test_case::contract::ProbeStep::Run));
        if !has_run {
            // A restart-only prefix: a session boundary without a judged
            // run — nothing to request; the next session opens cold.
            continue;
        }
        let source_sha = hex(&sha256(&source));

        let nonce = hex(&sha256(
            format!("{}|{:?}|mn", std::process::id(), started).as_bytes(),
        ));
        let mut request = serde_json::json!({
            "recipe": recipe,
            "nonce": nonce,
            "source_b64": b64(&source),
            "entry": mn.entry,
            "caps": mn.caps,
            "store_root": store_root.display().to_string().replace('/', "\\"),
            "workdir": workdir.display().to_string().replace('/', "\\"),
        });
        if recipe == COLLECTIONS_RECIPE {
            if let Some(id) = collection_ids.get(&source_sha) {
                request["manifest_id"] = serde_json::json!(id);
            }
        }
        if strict {
            request["diagnostics"] = serde_json::json!(source_diag::MODE);
        }
        // PLAN-919/923: the lifecycle and resource-guards recipes' exit
        // knobs (operations and budgets only; the observation switch is
        // implied by the recipe). Guards adds the per-seam injection knob.
        if recipe == RESOURCE_EVENTS_RECIPE
            || recipe == RESOURCE_GUARDS_RECIPE
            || recipe == CLOSURES_RESOURCE_RECIPE
        {
            let res = mn
                .resource
                .as_ref()
                .ok_or_else(|| "resource recipes require mn.resource".to_string())?;
            let mut o = serde_json::Map::new();
            if let Some(v) = res.budget_bytes {
                o.insert("budget_bytes".into(), serde_json::json!(v));
            }
            if let Some(v) = res.cancel_after_work {
                o.insert("cancel_after_work".into(), serde_json::json!(v));
            }
            if let Some(v) = res.fail_nth {
                o.insert("fail_nth".into(), serde_json::json!(v));
            }
            if let Some((seam, nth)) = res.fail_seam {
                o.insert("fail_seam".into(), serde_json::json!([seam, nth]));
            }
            if let Some(v) = res.fail_text_nth {
                o.insert("fail_text_nth".into(), serde_json::json!(v));
            }
            if let Some(v) = res.fail_ledger_nth {
                o.insert("fail_ledger_nth".into(), serde_json::json!(v));
            }
            request["resource"] = serde_json::Value::Object(o);
        }
        let request_hash = hex(&sha256(request.to_string().as_bytes()));
        request["request_hash"] = serde_json::json!(request_hash);

        let worker_started = Instant::now();
        let resp_result = call_worker_observed(&request.to_string(), Some(artifact_dir));
        let worker_us = worker_started.elapsed().as_micros() as u64;

        let mut counters: Vec<(String, u32)> = Vec::new();
        let mut spans: Vec<(String, u64)> = Vec::new();
        let mut session_exit: Option<i32> = None;
        let mut session_manifest: Option<String> = None;

        'admit: {
            let resp = match &resp_result {
                Ok(r) => serde_json::from_str::<serde_json::Value>(r)
                    .map_err(|e| format!("bad mn worker response ({e}): {r:.200}")),
                Err(e) => Err(e.clone()),
            };
            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    infra_detail = Some(e);
                    break 'admit;
                }
            };
            if resp.get("schema").and_then(|v| v.as_str()) != Some(MN_WORKER_PROTOCOL) {
                infra_detail = Some("mn worker schema mismatch".to_string());
                break 'admit;
            }
            if resp.get("nonce").and_then(|v| v.as_str()) != Some(nonce.as_str())
                || resp.get("request_hash").and_then(|v| v.as_str()) != Some(request_hash.as_str())
            {
                infra_detail = Some("mn worker nonce/request_hash echo mismatch".to_string());
                break 'admit;
            }
            let Some(ok) = resp.get("ok") else {
                let e = resp.get("error");
                infra_detail = Some(format!(
                    "mn worker error: {}",
                    e.map(|v| v.to_string()).unwrap_or_else(|| "missing".into())
                ));
                break 'admit;
            };
            if ok.get("recipe").and_then(|v| v.as_str()) != Some(recipe) {
                infra_detail = Some("mn worker recipe mismatch".to_string());
                break 'admit;
            }
            if ok.get("worker_version").and_then(|v| v.as_str()) != Some(EXPECTED_MN_WORKER_VERSION)
            {
                infra_detail = Some("mn worker version mismatch".to_string());
                break 'admit;
            }
            if ok.get("mn_abi").and_then(|v| v.as_str()) != Some("msvc-x64/priv-i32-bool-v1") {
                infra_detail = Some("mn worker ABI axis mismatch".to_string());
                break 'admit;
            }
            if recipe == COLLECTIONS_RECIPE && ok.get("diagnostic").is_none() {
                if let Err(error) = collections::admit_raw(ok) {
                    infra_detail = Some(error);
                    break 'admit;
                }
            }
            if recipe == COLLECTIONS_RECIPE {
                collection_runtime = Some(
                    serde_json::json!({"runtime":ok["runtime_identity"],"link":ok["link_identity"]}),
                );
            }
            if let Some(c) = ok.get("counters").and_then(|v| v.as_object()) {
                for (k, v) in c {
                    if let Some(n) = v.as_u64() {
                        counters.push((k.clone(), n as u32));
                    }
                }
            }
            if let Some(s) = ok.get("spans").and_then(|v| v.as_object()) {
                for (k, v) in s {
                    if let Some(n) = v.as_u64() {
                        spans.push((k.clone(), n));
                    }
                }
            }
            if strict {
                // The mode echo is mandatory for EVERY strict response —
                // legal programs included (a worker silently ignoring the
                // field is not compatible).
                if ok.get("diagnostics_mode").and_then(|v| v.as_str()) != Some(source_diag::MODE)
                    || ok.get("diagnostics_schema").and_then(|v| v.as_str())
                        != Some(source_diag::SNAPSHOT_SCHEMA)
                {
                    infra_detail =
                        Some("mn worker did not echo the source-v1 diagnostics mode".into());
                    break 'admit;
                }
            }
            if let Some(d) = resp.get("diagnostic").or_else(|| {
                if recipe == COLLECTIONS_RECIPE {
                    ok.get("diagnostic")
                } else {
                    None
                }
            }) {
                // A diagnosed pipeline rejection (mn.* / mir.* code) —
                // only the judged session's diagnosis is a verdict.
                let mut code = d
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .to_string();
                let mut message = d
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if strict {
                    // PLAN-911: a REAL source rejection is the typed fact —
                    // admitted on the raw line BEFORE any oracle comparison;
                    // the judged code/message become the primary cause's
                    // RAW stage/code (never the generic mn.* shell).
                    let raw_line = resp_result.as_deref().unwrap_or("");
                    match source_diag::admit(
                        raw_line,
                        &resp,
                        "mn.at",
                        &source_diag::OperationExpectation::Cold {
                            candidate: &source,
                            step: None,
                        },
                    ) {
                        Ok(fact) => {
                            source_fact = Some(fact);
                            code = source_fact
                                .as_ref()
                                .map(|f| f.primary_code.clone())
                                .unwrap_or(code);
                            message = source_fact
                                .as_ref()
                                .map(|f| f.legacy_message.clone())
                                .unwrap_or(message);
                        }
                        Err(e) => {
                            infra_detail = Some(format!("source-v1 admission: {e}"));
                            break 'admit;
                        }
                    }
                }
                let judged = run_index + 1 == total_runs;
                session_records.push(SessionRecord {
                    steps: step_labels,
                    source_sha,
                    exit_code: None,
                    manifest_fp: None,
                    counters,
                    spans,
                    worker_us,
                    judged,
                });
                if judged {
                    diag = Some((code, message));
                    break 'sessions;
                }
                run_index += 1;
                continue 'sessions;
            }
            // Structural counter admission (successful runs only — a
            // diagnosed rejection never reaches link/run by design): a
            // judged run REALLY linked and REALLY executed; a cached PASS
            // is never a verdict here.
            let link = counters
                .iter()
                .find(|(k, _)| k == "link_invocations")
                .map(|(_, v)| *v)
                .unwrap_or(0);
            let run = counters
                .iter()
                .find(|(k, _)| k == "run_invocations")
                .map(|(_, v)| *v)
                .unwrap_or(0);
            if link < 1 || run < 1 {
                infra_detail = Some(format!(
                    "mn worker counters violate the always-execute contract (link={link}, run={run})"
                ));
                break 'admit;
            }
            session_exit = ok
                .get("exit_code")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32);
            let session_stderr = ok
                .get("stderr_hex")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            // Parent-side MANIFEST identity admission: a successful run
            // must echo its manifest fingerprint, and sessions fed
            // IDENTICAL sources must agree on it.
            let fp = ok
                .get("manifest_fp")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            // PLAN-919/923: a lifecycle/guards run that ENDED in the abort
            // sweep (budget/cancel/trap) never publishes its candidate —
            // the manifest identity chain applies to the published faces.
            let trap_face = (recipe == RESOURCE_EVENTS_RECIPE
                || recipe == RESOURCE_GUARDS_RECIPE
                || recipe == CLOSURES_RESOURCE_RECIPE)
                && hex_stream(&session_stderr)
                    .ok()
                    .and_then(|b| String::from_utf8(b).ok())
                    .map(|t| t.contains("MN-COLL-TRAP"))
                    .unwrap_or(false);
            if fp.is_empty() && !trap_face {
                infra_detail = Some("mn worker success without a manifest fingerprint".to_string());
                break 'admit;
            }
            if let Some(prev) = manifest_by_source.get(&source_sha) {
                if *prev != fp {
                    infra_detail = Some(format!(
                        "mn worker manifest identity unstable for one source: {prev} vs {fp}"
                    ));
                    break 'admit;
                }
            } else {
                manifest_by_source.insert(source_sha.clone(), fp.clone());
            }
            if recipe == COLLECTIONS_RECIPE {
                if let Some(id) = ok.get("manifest_id").and_then(|v| v.as_str()) {
                    collection_ids.insert(source_sha.clone(), id.into());
                }
            }
            session_manifest = Some(fp);
            if run_index + 1 == total_runs {
                // The judged session: its raw PE bytes ride the receipt.
                stderr_hex = session_stderr;
            }
        }
        if infra_detail.is_some() {
            break 'sessions;
        }
        let judged = run_index + 1 == total_runs;
        session_records.push(SessionRecord {
            steps: step_labels,
            source_sha,
            exit_code: session_exit,
            manifest_fp: session_manifest,
            counters,
            spans,
            worker_us,
            judged,
        });
        if judged {
            exit_code = session_exit;
        }
        run_index += 1;
    }

    if let Some(e) = infra_detail {
        return Ok(Verdict {
            status: Status::Error,
            mismatches: vec![Mismatch {
                observation: "worker".into(),
                kind: "infra".into(),
                expected: "a contained worker session".into(),
                actual: e,
                first_diff: None,
            }],
            detail: Some(format!("{recipe}: infra failure")),
            artifacts: vec![],
            native: None,
        });
    }

    if recipe == COLLECTIONS_RECIPE && diag.is_none() {
        if let Some(reason) = collections::trap_from_hex(&stderr_hex)? {
            diag = Some((
                collections::trap_code(reason).into(),
                format!("native collection trap {reason}"),
            ));
        }
    }
    // PLAN-919: the lifecycle recipe's parent-side judge — strict decode
    // of the MN-EVT stream, the trap face mapping, and the frozen per-case
    // table (independent of anything the worker self-reports).
    if recipe == RESOURCE_EVENTS_RECIPE {
        match resource_events::decode_events_hex(&stderr_hex) {
            Ok(Some(facts)) => {
                if diag.is_none() {
                    if let Some(t) = facts.trap {
                        if let Some(code) = resource_events::completion_code(t) {
                            diag = Some((code.to_string(), format!("native collection trap {t}")));
                        }
                    }
                }
                let oracle_key = format!("{}/{}", case.spec.id, check.id);
                match resource_events::oracle_for(&oracle_key) {
                    Some(o) => resource_events::judge_native(o, exit_code, &facts, &mut mismatches),
                    None => mismatches.push(Mismatch {
                        observation: "native-lifecycle".into(),
                        kind: "case-registration".into(),
                        expected: "a frozen native resource-events oracle row".into(),
                        actual: format!("check {oracle_key} is not in the parent table"),
                        first_diff: None,
                    }),
                }
            }
            Ok(None) => {
                mismatches.push(Mismatch {
                    observation: "native-lifecycle".into(),
                    kind: "events-missing".into(),
                    expected: "the MN-EVT stream of an observed run".into(),
                    actual: "no event lines".into(),
                    first_diff: None,
                });
            }
            Err(detail) => {
                mismatches.push(Mismatch {
                    observation: "native-lifecycle".into(),
                    kind: "admission".into(),
                    expected: "a strictly decodable MN-EVT stream".into(),
                    actual: detail,
                    first_diff: None,
                });
            }
        }
    }
    // PLAN-923: the resource-guards recipe's parent-side judge — strict
    // decode of the MN-COLL-INJECT seam/nth facts and the trap/cleanup
    // face against the frozen per-case table (independent of anything the
    // worker self-reports; the request carried only operation/seam/index/
    // budget).
    if recipe == RESOURCE_GUARDS_RECIPE {
        match resource_guards::decode_facts_hex(&stderr_hex) {
            Ok(facts) => {
                if diag.is_none() {
                    if let Some(t) = facts.trap {
                        if let Some(code) = resource_events::completion_code(t) {
                            diag = Some((code.to_string(), format!("native collection trap {t}")));
                        }
                    }
                }
                let oracle_key = format!("{}/{}", case.spec.id, check.id);
                match resource_guards::oracle_for(&oracle_key) {
                    Some(o) => resource_guards::judge_native(o, exit_code, &facts, &mut mismatches),
                    None => mismatches.push(Mismatch {
                        observation: "native-guards".into(),
                        kind: "case-registration".into(),
                        expected: "a frozen native resource-guards oracle row".into(),
                        actual: format!("check {oracle_key} is not in the parent table"),
                        first_diff: None,
                    }),
                }
            }
            Err(detail) => {
                mismatches.push(Mismatch {
                    observation: "native-guards".into(),
                    kind: "admission".into(),
                    expected: "a strictly decodable MN-COLL-INJECT/trap stream".into(),
                    actual: detail,
                    first_diff: None,
                });
            }
        }
    }
    // PLAN-924: the closures-resource recipe's parent-side judge — strict
    // decode of the S1..S9 injection facts, the trap/cleanup face and the
    // R05 env-lifecycle event shape against the frozen per-case table.
    if recipe == CLOSURES_RESOURCE_RECIPE {
        match native_closures::decode_facts_hex(&stderr_hex) {
            Ok(facts) => {
                if diag.is_none() {
                    if let Some(t) = facts.trap {
                        if let Some(code) = resource_events::completion_code(t) {
                            diag = Some((code.to_string(), format!("native closure trap {t}")));
                        }
                    }
                }
                let oracle_key = format!("{}/{}", case.spec.id, check.id);
                match native_closures::oracle_for(&oracle_key) {
                    Some(o) => native_closures::judge_native(o, exit_code, &facts, &mut mismatches),
                    None => mismatches.push(Mismatch {
                        observation: "native-closures".into(),
                        kind: "case-registration".into(),
                        expected: "a frozen native closures oracle row".into(),
                        actual: format!("check {oracle_key} is not in the parent table"),
                        first_diff: None,
                    }),
                }
            }
            Err(detail) => {
                mismatches.push(Mismatch {
                    observation: "native-closures".into(),
                    kind: "admission".into(),
                    expected: "a strictly decodable MN-COLL-INJECT/trap/MN-EVT stream".into(),
                    actual: detail,
                    first_diff: None,
                });
            }
        }
    }
    // ---- parent-side judgement -----------------------------------------
    let observed = if let Some((code, message)) = &diag {
        // Map the mn.* / mir.* diagnostic onto the contract stages:
        // frontend rejections parse; adapter/MIR admissions report at the
        // probe adapter's post-frontend boundary face `typecheck` (the
        // probe stage vocabulary admits parse/resolve/typecheck/run —
        // PLAN-870 T-05 wiring; the underlying pipeline boundary stays
        // named by the mn.* code itself).
        let stage = if let Some(fact) = &source_fact {
            // PLAN-911 strict: the RAW producer stage decides (text→parse,
            // bind→resolve, verify→typecheck) — the admission already
            // rejected every non-language stage, so this never defaults.
            fact.mapped_stage
        } else if recipe == COLLECTIONS_RECIPE && code.starts_with("mn.trap.") {
            Stage::Run
        } else if (recipe == RESOURCE_EVENTS_RECIPE
            || recipe == RESOURCE_GUARDS_RECIPE
            || recipe == CLOSURES_RESOURCE_RECIPE)
            && (code.starts_with("mn.trap.") || code.starts_with("mn.resource."))
        {
            Stage::Run
        } else if code.starts_with("mn.unsupported.frontend")
            || code.starts_with("mn.unsupported.input")
        {
            Stage::Parse
        } else {
            Stage::Typecheck
        };
        match &oracle.completion {
            Completion::Error {
                stage: want_stage,
                code: want_code,
                ..
            } => {
                if want_stage != &stage {
                    mismatches.push(Mismatch {
                        observation: "stage".into(),
                        kind: "stage".into(),
                        expected: format!("{want_stage:?}"),
                        actual: format!("{stage:?}"),
                        first_diff: None,
                    });
                }
                if let Some(want) = want_code {
                    if want != code {
                        mismatches.push(Mismatch {
                            observation: "code".into(),
                            kind: "code".into(),
                            expected: want.clone(),
                            actual: code.clone(),
                            first_diff: None,
                        });
                    }
                }
            }
            other => {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "outcome".into(),
                    expected: format!("{other:?}"),
                    actual: format!("diagnosed {code}: {message}"),
                    first_diff: None,
                });
            }
        }
        "diagnosed".to_string()
    } else {
        match &oracle.completion {
            Completion::Success => {
                if let Some(want) = oracle.exit {
                    if exit_code != Some(want) {
                        mismatches.push(Mismatch {
                            observation: "exit".into(),
                            kind: "exit".into(),
                            expected: format!("{want}"),
                            actual: format!("{exit_code:?}"),
                            first_diff: None,
                        });
                    }
                } else if exit_code.is_none() {
                    mismatches.push(Mismatch {
                        observation: "exit".into(),
                        kind: "missing".into(),
                        expected: "a real PE exit code".into(),
                        actual: "none".into(),
                        first_diff: None,
                    });
                }
                // PLAN-914: the real PE's stderr (the executed mark bytes on
                // the MIR-native scenario path). The worker reports it
                // hex-encoded; the parent decodes and compares it to the
                // declared oracle. A malformed stream is a mismatch, never a
                // silent skip.
                if let Some(want) = &oracle.stderr {
                    match hex_stream(&stderr_hex) {
                        Ok(actual) => {
                            if let Some(m) =
                                compare_bytes(want.text.as_bytes(), &actual, want.compare, "stderr")
                            {
                                mismatches.push(m);
                            }
                        }
                        Err(detail) => mismatches.push(Mismatch {
                            observation: "stderr".into(),
                            kind: "decode".into(),
                            expected: "a hex-encoded PE stderr stream".into(),
                            actual: detail,
                            first_diff: None,
                        }),
                    }
                }
            }
            Completion::Error { stage, .. } => {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "outcome".into(),
                    expected: format!("{stage:?} rejection"),
                    actual: format!("success exit {exit_code:?}"),
                    first_diff: None,
                });
            }
        }
        "success".to_string()
    };

    // ---- receipt: per-session evidence (T-06) -------------------------
    let request_wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    let exe_sha = bound_worker_sha(recipe)?;
    let judged_record = session_records.iter().rev().find(|r| r.judged);
    let sessions_json: Vec<serde_json::Value> = session_records
        .iter()
        .map(|r| {
            serde_json::json!({
                "steps": r.steps,
                "source_sha256": r.source_sha,
                "exit_code": r.exit_code,
                "manifest_fp": r.manifest_fp,
                "judged": r.judged,
                "worker_us": r.worker_us,
                "counters": r.counters,
                "spans": r.spans,
            })
        })
        .collect();
    let mut receipt = serde_json::json!({
        "recipe": recipe,
        "protocol": MN_WORKER_PROTOCOL,
        "exit_code": exit_code,
        "stderr_hex": stderr_hex,
        "sessions": sessions_json,
        "request_wall_ms": request_wall_ms,
        "toolchain": {
            "recipe": recipe,
            "mir-lowering": EXPECTED_MIR_LOWERING_VERSION,
            "mn-worker-exe-sha256": exe_sha,
        },
    });
    if recipe == COLLECTIONS_RECIPE {
        receipt["collections_runtime"] = serde_json::json!(collection_runtime);
    }
    std::fs::write(
        artifact_dir.join("mn-probe-receipt.json"),
        serde_json::to_string_pretty(&receipt).unwrap_or_default(),
    )
    .map_err(|e| format!("write receipt: {e}"))?;

    // Aggregated observability for the record (the judged session's own
    // counters/spans lead, so single-run shapes are byte-compatible in
    // spirit with the T-02 receipt).
    let mut counters: Vec<(String, u32)> = Vec::new();
    let mut spans: Vec<(String, u64)> = Vec::new();
    let mut worker_us_total: u64 = 0;
    for r in &session_records {
        worker_us_total += r.worker_us;
        for (k, v) in &r.counters {
            if let Some(e) = counters.iter_mut().find(|(ek, _)| ek == k) {
                e.1 += v;
            } else {
                counters.push((k.clone(), *v));
            }
        }
        for (k, v) in &r.spans {
            if let Some(e) = spans.iter_mut().find(|(ek, _)| ek == k) {
                e.1 += v;
            } else {
                spans.push((k.clone(), *v));
            }
        }
    }
    let judged_spans: Vec<(String, u64)> =
        judged_record.map(|r| r.spans.clone()).unwrap_or_default();
    let judged_counters: Vec<(String, u32)> = judged_record
        .map(|r| r.counters.clone())
        .unwrap_or_default();

    let native = NativeRecord {
        exit_code,
        stage_spans: {
            let mut s = vec![
                ("worker_us".to_string(), worker_us_total),
                ("request_wall_ms".to_string(), request_wall_ms as u64),
            ];
            s.extend(judged_spans);
            s
        },
        cache_mode: None,
        cache_hits: judged_counters
            .iter()
            .filter(|(k, _)| k == "cache_hit_coff")
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect(),
        counters,
        toolchain: native_toolchain(recipe, exe_sha, collection_runtime.as_ref()),
        gaps: vec![],
    };
    let status = if mismatches.is_empty() {
        Status::Pass
    } else {
        Status::Fail
    };
    // PLAN-911: the admitted typed fact is published as the check's
    // evidence artifact (bound to the execution key; a write failure is
    // an error, never a green comparison).
    let mut artifacts: Vec<auto_test_case::report::ArtifactRef> = Vec::new();
    if let Some(fact) = &source_fact {
        artifacts.push(source_diag::write_fact_artifact(
            artifact_dir,
            execution_key,
            recipe,
            EXPECTED_MN_WORKER_VERSION,
            fact,
        )?);
    }
    Ok(Verdict {
        status,
        mismatches,
        detail: Some(format!(
            "{recipe}: run judged (outcome {observed}); receipt mn-probe-receipt.json"
        )),
        artifacts,
        native: Some(native),
    })
}

/// F-818-R1-03: the formal INCREMENTAL path — the worker warms one D-828
/// session on the case source and applies the edit steps as single-body
/// transactions. F-818-R1-01: the parent drives the ROUNDS protocol (one
/// persistent worker process; ONE request/response line pair per step)
/// and measures each edit's CONTINUOUS parent-observed window — the
/// §5.5 interval from the parent's own request preparation (source copy,
/// literal search/insert, payload serialization) through IPC, the
/// worker's locate/edit/link/run, the response, and the parent's
/// admission of the round. Parent-side admission beyond the
/// whole-request path: every edit step must be TRULY incremental (no
/// unaffected-function stage work, no whole-module stage, no recorded
/// fallback, no refusal) and every step really links and runs; the
/// judged value is the LAST step's PE exit. A syntax-fallback edit
/// switches the session to whole-request materialization (F-818-R2-02:
/// the accepted prefix is preserved exactly once); its trailing run is
/// admitted under the whole-request contract and the receipt labels the
/// fallback. The worker-side continuous `wall_ms` rides every record.
fn run_incremental(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    artifact_dir: &Path,
    execution_key: &str,
    mn: &auto_test_case::contract::ProbeMnSpec,
    steps: &[auto_test_case::contract::ProbeStep],
) -> Result<Verdict, String> {
    let strict = mn.diagnostics.as_deref() == Some(source_diag::MODE);
    use auto_test_case::contract::{Completion, Stage};
    let recipe = check
        .options
        .probe_recipe
        .as_deref()
        .ok_or("missing native recipe")?;

    let started = Instant::now();
    // PLAN-914 (I04): declaration-oracle mismatches collected inside the
    // rounds window and merged into the final verdict after the loop.
    let mut decl_mismatches: Vec<Mismatch> = Vec::new();
    // PLAN-1000 (I05): the content-revisit oracle, judged after the loop
    // from the summed content-addressed cache hits.
    let mut history_mismatches: Vec<Mismatch> = Vec::new();
    if case.spec.source.syntax != auto_test_case::contract::Syntax::Auto {
        return Err(format!(
            "{recipe} consumes .at sources; syntax {:?} is not accepted",
            case.spec.source.syntax.as_str()
        ));
    }
    let input_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or_else(|| "probe adapter needs the resolved entry input".to_string())?;
    let store_root = match &mn.session {
        Some(id) => {
            let base = std::env::var("AC_MN_SESSION_ROOT")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| std::env::temp_dir());
            std::fs::create_dir_all(&base).map_err(|e| format!("session root: {e}"))?;
            base.join(format!("mn818-inc-{id}"))
        }
        None => artifact_dir.join("mn-inc-store"),
    };
    std::fs::create_dir_all(&store_root).map_err(|e| format!("inc store root: {e}"))?;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("mn artifact dir: {e}"))?;
    let workdir = artifact_dir.join("mn-inc-work");

    // F-818-R1-01 residual: the OBSERVER SEAM for independent per-edit
    // expectations. The worker never sees expected outcomes; when the
    // observer (the formal test, holding the independent oracle) supplies
    // them via the environment, the adapter compares each edit round's
    // real PE result INSIDE the round's continuous parent-observed window
    // (preparation → IPC → worker → admission → comparison) — a mismatch
    // is a recorded verdict failure, never a silent admission.
    let expected_exits: Option<Vec<i64>> = match std::env::var("AC_MN_INCREMENTAL_EXPECT_EXITS") {
        Err(_) => None,
        Ok(raw) if raw.trim().is_empty() => None,
        Ok(raw) => {
            let list = raw
                .split(',')
                .map(|t| t.trim().parse::<i64>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("AC_MN_INCREMENTAL_EXPECT_EXITS: {e}"))?;
            let edits = steps
                .iter()
                .filter(|s| matches!(s, auto_test_case::contract::ProbeStep::EditSource { .. }))
                .count();
            if list.len() != edits {
                return Err(format!(
                    "AC_MN_INCREMENTAL_EXPECT_EXITS carries {} values for {edits} edit steps (observer seam misuse)",
                    list.len()
                ));
            }
            Some(list)
        }
    };

    let nonce = hex(&sha256(
        format!("{}|{:?}|mn-inc", std::process::id(), started).as_bytes(),
    ));
    let mut request = serde_json::json!({
        "recipe": recipe,
        "mode": "rounds",
        "nonce": nonce,
        "source_b64": b64(&input_bytes),
        "entry": mn.entry,
        "caps": mn.caps,
        "store_root": store_root.display().to_string().replace('/', "\\"),
        "workdir": workdir.display().to_string().replace('/', "\\"),
    });
    if strict {
        request["diagnostics"] = serde_json::json!(source_diag::MODE);
    }
    let request_hash = hex(&sha256(request.to_string().as_bytes()));
    request["request_hash"] = serde_json::json!(request_hash);

    let (mut rounds, warm_line) = match RoundsWorker::start(&request.to_string()) {
        Ok(p) => p,
        Err(e) => {
            return Ok(Verdict {
                status: Status::Error,
                mismatches: vec![Mismatch {
                    observation: "worker".into(),
                    kind: "infra".into(),
                    expected: "a contained incremental worker session".into(),
                    actual: e,
                    first_diff: None,
                }],
                detail: Some(format!("{recipe}: infra failure (incremental)")),
                artifacts: vec![],
                native: None,
            })
        }
    };

    let mut collection_runtime: Option<serde_json::Value> = None;
    let mut infra_detail: Option<String> = None;
    let mut diag: Option<(String, String)> = None;
    // PLAN-911: the admitted typed fact of the judged refusal (if any).
    let mut source_fact: Option<source_diag::AdmittedFact> = None;
    let mut exit_code: Option<i32> = None;
    let mut stderr_hex = String::new();
    let mut stdout_hex = String::new();
    let mut step_records: Vec<serde_json::Value> = Vec::new();
    let mut parent_windows_ms: Vec<f64> = Vec::new();
    let mut whole_fallback = false;
    let mut resp_manifest_fp = String::new();
    // Seam-supplied independent comparisons that FAILED (verdict-level).
    let mut expect_mismatches: Vec<(usize, i64, Option<i32>)> = Vec::new();

    'admit: {
        let warm: serde_json::Value = match serde_json::from_str(&warm_line) {
            Ok(v) => v,
            Err(e) => {
                infra_detail = Some(format!("bad warmup line ({e}): {warm_line:.200}"));
                break 'admit;
            }
        };
        if warm.get("schema").and_then(|v| v.as_str()) != Some(MN_WORKER_PROTOCOL)
            || warm.get("nonce").and_then(|v| v.as_str()) != Some(nonce.as_str())
            || warm.get("request_hash").and_then(|v| v.as_str()) != Some(request_hash.as_str())
        {
            infra_detail = Some("mn rounds warmup echo mismatch".to_string());
            break 'admit;
        }
        let warm_ok = warm.get("ok").cloned().unwrap_or(serde_json::Value::Null);
        if warm_ok.get("op").and_then(|v| v.as_str()) != Some("warmup")
            || warm_ok
                .get("exit_code")
                .map(|v| v.is_null())
                .unwrap_or(true)
        {
            infra_detail = Some("mn rounds worker did not answer the warmup round".to_string());
            break 'admit;
        }
        if warm_ok.get("recipe").and_then(|v| v.as_str()) != Some(recipe)
            || warm_ok.get("worker_version").and_then(|v| v.as_str())
                != Some(EXPECTED_MN_WORKER_VERSION)
            || warm_ok.get("mn_abi").and_then(|v| v.as_str()) != Some("msvc-x64/priv-i32-bool-v1")
        {
            infra_detail = Some("mn rounds worker identity mismatch".to_string());
            break 'admit;
        }
        if strict {
            // The warmup round must echo the strict mode too (a legal
            // warmup included); a warmup source rejection arrives as the
            // typed fact and is admitted below like any other round.
            if warm_ok.get("diagnostics_mode").and_then(|v| v.as_str()) != Some(source_diag::MODE)
                || warm_ok.get("diagnostics_schema").and_then(|v| v.as_str())
                    != Some(source_diag::SNAPSHOT_SCHEMA)
            {
                // A warm-up source rejection: shape with source_error and
                // no execution result — the identity checks above already
                // passed on the ok metadata.
                if warm.get("source_error").is_some() {
                    match source_diag::admit(
                        &warm_line,
                        &warm,
                        "mn.at",
                        &source_diag::OperationExpectation::Cold {
                            candidate: &input_bytes,
                            step: None,
                        },
                    ) {
                        Ok(fact) => {
                            diag = Some((fact.primary_code.clone(), fact.legacy_message.clone()));
                            source_fact = Some(fact);
                            step_records.push(warm_ok);
                            break 'admit;
                        }
                        Err(e) => {
                            infra_detail = Some(format!("source-v1 admission (warmup): {e}"));
                            break 'admit;
                        }
                    }
                }
                infra_detail =
                    Some("mn rounds worker did not echo the source-v1 diagnostics mode".into());
                break 'admit;
            }
        }
        if recipe == COLLECTIONS_RECIPE {
            collections::admit_raw(&warm_ok)?;
            collection_runtime = Some(
                serde_json::json!({"runtime":warm_ok["runtime_identity"],"link":warm_ok["link_identity"]}),
            );
        }
        step_records.push(warm_ok);

        let mut source = input_bytes.clone();
        // The observer-seam cursor: one expected exit per edit step,
        // consumed at PREPARATION time (fallback-materialized rounds keep
        // their slot — the comparison there is null, the case oracle owns
        // the judged value).
        let mut expect_idx = 0usize;
        // PLAN-914 (I04): declaration-oracle mismatches collected inside the
        // rounds window and merged into the final verdict after the loop.
        for (i, st) in steps.iter().enumerate() {
            // The parent-observed window opens at the parent's OWN
            // preparation for this round (F-818-R1-01): the exactly-once
            // literal replacement on the parent's source copy plus the
            // round payload serialization happen INSIDE it.
            let round_started = Instant::now();
            let round_nonce = hex(&sha256(format!("{nonce}|step{i}").as_bytes()));
            let (op, round_line, step_expect, round_sources) = match st {
                auto_test_case::contract::ProbeStep::EditSource { from, to } => {
                    // PLAN-911: the parent's own view of the failing
                    // candidate — the accepted prefix (pre-edit source)
                    // and the candidate (post-edit) bind the snapshot.
                    let base_source = source.clone();
                    let next = match replace_once(&source, from, to) {
                        Ok(s) => s,
                        Err(e) => {
                            infra_detail = Some(e);
                            break 'admit;
                        }
                    };
                    source = next;
                    let candidate_source = source.clone();
                    let step_expect = expected_exits.as_ref().map(|xs| xs[expect_idx]);
                    expect_idx += 1;
                    (
                        "edit-source",
                        serde_json::json!({
                            "op": "edit-source",
                            "from": from,
                            "to": to,
                            "nonce": round_nonce,
                        })
                        .to_string(),
                        step_expect,
                        Some((base_source, candidate_source)),
                    )
                }
                auto_test_case::contract::ProbeStep::DeclarationInvalidate { from, to } => {
                    // PLAN-914 (I04): the explicit declaration/layout
                    // invalidation round. The parent materializes the
                    // candidate exactly once (the same discriminating
                    // replacement shape as a body edit); the worker runs the
                    // cold declaration frontend + dependency-precise rebuild.
                    let base_source = source.clone();
                    let next = match replace_once(&source, from, to) {
                        Ok(s) => s,
                        Err(e) => {
                            infra_detail = Some(e);
                            break 'admit;
                        }
                    };
                    source = next;
                    let candidate_source = source.clone();
                    (
                        "declaration-invalidate",
                        serde_json::json!({
                            "op": "declaration-invalidate",
                            "from": from,
                            "to": to,
                            "nonce": round_nonce,
                        })
                        .to_string(),
                        None,
                        Some((base_source, candidate_source)),
                    )
                }
                auto_test_case::contract::ProbeStep::Run => (
                    "run",
                    serde_json::json!({"op": "run", "nonce": round_nonce}).to_string(),
                    None,
                    None,
                ),
                other => {
                    infra_detail = Some(format!(
                        "{recipe} incremental path does not take {other:?} steps"
                    ));
                    break 'admit;
                }
            };
            let line = match rounds.round(&round_line) {
                Ok(l) => l,
                Err(e) => {
                    infra_detail = Some(e);
                    break 'admit;
                }
            };
            let resp: serde_json::Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(e) => {
                    infra_detail = Some(format!("bad round line ({e}): {line:.200}"));
                    break 'admit;
                }
            };
            if resp.get("schema").and_then(|v| v.as_str()) != Some(MN_WORKER_PROTOCOL) {
                infra_detail = Some("mn worker schema mismatch".to_string());
                break 'admit;
            }
            if resp.get("nonce").and_then(|v| v.as_str()) != Some(round_nonce.as_str()) {
                infra_detail = Some("mn worker round nonce echo mismatch".to_string());
                break 'admit;
            }
            if let Some(err) = resp.get("error") {
                infra_detail = Some(format!(
                    "mn worker error: {}",
                    serde_json::to_string(err).unwrap_or_default()
                ));
                break 'admit;
            }
            let Some(ok) = resp.get("ok").cloned() else {
                infra_detail = Some("mn round response without a record".to_string());
                break 'admit;
            };
            if ok.get("op").and_then(|v| v.as_str()) != Some(op) {
                infra_detail = Some(format!(
                    "mn round answered op {:?}, the parent asked for {op}",
                    ok.get("op").and_then(|v| v.as_str())
                ));
                break 'admit;
            }
            if recipe == COLLECTIONS_RECIPE {
                if Some(serde_json::json!({"runtime":ok["runtime_identity"],"link":ok["link_identity"]})).as_ref()!=collection_runtime.as_ref() {infra_detail=Some("collections runtime changed within a retained session".into());break 'admit;}
                if let Err(error) = collections::admit_raw(&ok) {
                    infra_detail = Some(error);
                    break 'admit;
                }
                if op == "edit-source" {
                    if let Err(error) = collections::admit_body(&ok) {
                        infra_detail = Some(error);
                        break 'admit;
                    }
                }
            }
            if op == "edit-source" {
                if strict && resp.get("source_error").is_some() {
                    // PLAN-911 strict: a REAL source rejection of this
                    // round's candidate (fast-path refusal or the strict
                    // materialization gate). Admitted on the raw line; the
                    // sequence STOPS here — later steps are not sent (the
                    // parent's side of first-failure-stop), and the
                    // parent's source rolls back to the accepted prefix.
                    let (base_source, candidate_source) =
                        round_sources.expect("edit-source rounds carry their sources");
                    let expect = if ok.get("warm_edit_fallback").and_then(|v| v.as_bool())
                        == Some(true)
                        && ok.get("materialized").and_then(|v| v.as_bool()) == Some(false)
                    {
                        source_diag::OperationExpectation::FallbackCold {
                            base: &base_source,
                            candidate: &candidate_source,
                            step: i as u64,
                        }
                    } else {
                        source_diag::OperationExpectation::WarmEdit {
                            base: &base_source,
                            candidate: &candidate_source,
                            step: i as u64,
                        }
                    };
                    match source_diag::admit(&line, &resp, "probe.at", &expect) {
                        Ok(fact) => {
                            // The accepted prefix is `base_source` (bound in
                            // the fact); the sequence stops here, so the
                            // parent's cursor is not advanced further.
                            diag = Some((fact.primary_code.clone(), fact.legacy_message.clone()));
                            source_fact = Some(fact);
                            let mut rec = ok;
                            rec["parent_window_ms"] =
                                serde_json::json!(round_started.elapsed().as_secs_f64() * 1000.0);
                            step_records.push(rec);
                            // Stop the sequence at the first failure.
                            break 'admit;
                        }
                        Err(e) => {
                            infra_detail = Some(format!("source-v1 admission (round {i}): {e}"));
                            break 'admit;
                        }
                    }
                }
                if strict {
                    // PLAN-914 (I09): a typed NON-language refusal (the
                    // retained environment's interface/layout gate, an
                    // invalid edit shape, session/candidate drift) is the
                    // engine's DECLARED POLICY decision, not an
                    // infrastructure failure: judge it as the typed refusal
                    // verdict and stop the sequence. LanguageDiagnostics
                    // refusals carry a source_error and are admitted above.
                    // PLAN-932: a refusal whose typed fact is missing or
                    // invalid is an infra ERROR, never a degraded verdict.
                    match refusal_verdict(&ok) {
                        Some(Ok((code, reason))) => {
                            diag = Some((code, reason));
                            let mut rec = ok;
                            rec["parent_window_ms"] =
                                serde_json::json!(round_started.elapsed().as_secs_f64() * 1000.0);
                            step_records.push(rec);
                            break 'admit;
                        }
                        Some(Err(detail)) => {
                            infra_detail = Some(detail);
                            break 'admit;
                        }
                        None => {}
                    }
                }
                if ok.get("warm_edit_fallback").and_then(|v| v.as_bool()) == Some(true) {
                    // F-818-R2-02: the syntax increment could not converge
                    // on this legal edit; the worker materialized it on the
                    // session's CURRENT source (accepted prefix preserved).
                    // Not an incremental sample; the trailing run executes
                    // one whole-request pipeline admitted below. The seam
                    // slot is kept with a NULL comparison — no per-edit PE
                    // result exists here; the case oracle judges the run.
                    whole_fallback = true;
                    let mut rec = ok;
                    if let Some(want) = step_expect {
                        rec["expected_exit"] = serde_json::json!(want);
                        rec["expected_match"] = serde_json::Value::Null;
                    }
                    rec["parent_window_ms"] =
                        serde_json::json!(round_started.elapsed().as_secs_f64() * 1000.0);
                    step_records.push(rec);
                    continue;
                }
                // PLAN-914 (I09): the retained environment REFUSED this warm
                // edit (the non-fast-path interface/declaration-context gate).
                // That refusal IS the live-state policy result the scenario
                // judges — record it as the typed verdict and stop the
                // sequence; the accepted prefix remains the session state.
                // PLAN-932: only a complete valid typed refusal is a
                // verdict; a default-mode untyped `refused` string is a
                // typed-fact breach → infra ERROR (never a degraded FAIL).
                match refusal_verdict(&ok) {
                    Some(Ok((code, reason))) => {
                        diag = Some((code, reason));
                        let mut rec = ok;
                        rec["parent_window_ms"] =
                            serde_json::json!(round_started.elapsed().as_secs_f64() * 1000.0);
                        step_records.push(rec);
                        break 'admit;
                    }
                    Some(Err(detail)) => {
                        infra_detail = Some(detail);
                        break 'admit;
                    }
                    None => {}
                }
                let unaffected = ok
                    .get("unaffected_stage_delta")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let whole = ok
                    .get("whole_module_stage_delta")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let fallback = ok
                    .get("fallback_whole_lower")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if unaffected != 0 || whole != 0 || fallback != 0 {
                    infra_detail = Some(format!(
                        "edit round {i} violates the incremental contract (unaffected={unaffected}, whole={whole}, fallback={fallback})"
                    ));
                    break 'admit;
                }
                let round_exit = ok
                    .get("exit_code")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);
                if round_exit.is_none() {
                    infra_detail = Some(format!("edit round {i} without a real PE exit"));
                    break 'admit;
                }
                exit_code = round_exit;
                stderr_hex = ok
                    .get("stderr_hex")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                stdout_hex = ok
                    .get("stdout_hex")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let mut rec = ok;
                // F-818-R1-01 residual: the INDEPENDENT expected-exit
                // comparison runs INSIDE the window (before it closes) —
                // the round-owned measurement covers preparation through
                // admission AND comparison; a mismatch is recorded and
                // fails the verdict below.
                if let Some(want) = step_expect {
                    let matched = round_exit == Some(want as i32);
                    rec["expected_exit"] = serde_json::json!(want);
                    rec["expected_match"] = serde_json::json!(matched);
                    if !matched {
                        expect_mismatches.push((i, want, round_exit));
                    }
                }
                // The parent-observed window closes here: preparation,
                // IPC, worker work, response, admission, and comparison.
                let window_ms = round_started.elapsed().as_secs_f64() * 1000.0;
                rec["parent_window_ms"] = serde_json::json!(window_ms);
                parent_windows_ms.push(window_ms);
                step_records.push(rec);
            } else if matches!(
                st,
                auto_test_case::contract::ProbeStep::DeclarationInvalidate { .. }
            ) {
                // PLAN-914 (I04, D-05 criterion): the declaration round's
                // engine policy is a cold declaration frontend followed by a
                // dependency-precise rebuild — the body-edit zero-production
                // gate does NOT apply. Judge the declared declaration oracle:
                // the real cold-frontend invalidations, the object-producer
                // bound and the identity carry-over of the unaffected units.
                let Some(spec) = mn.declaration.as_ref() else {
                    infra_detail =
                        Some("declaration round without a declaration oracle".to_string());
                    break 'admit;
                };
                decl_mismatches.extend(judge_declaration_invalidation(&ok, spec));
                let round_exit = ok
                    .get("exit_code")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);
                let Some(round_exit) = round_exit else {
                    infra_detail = Some(format!("declaration round {i} without a real PE exit"));
                    break 'admit;
                };
                exit_code = Some(round_exit);
                stderr_hex = ok
                    .get("stderr_hex")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                stdout_hex = ok
                    .get("stdout_hex")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let mut rec = ok;
                rec["parent_window_ms"] =
                    serde_json::json!(round_started.elapsed().as_secs_f64() * 1000.0);
                step_records.push(rec);
                continue;
            } else {
                // A run round: either the judged incremental re-link/run or
                // the whole-request leg after a syntax fallback.
                if ok.get("warm_edit_fallback").and_then(|v| v.as_bool()) == Some(true) {
                    if strict && resp.get("source_error").is_some() {
                        // PLAN-911 strict: the whole-request leg's source
                        // rejection — the final materialized source is the
                        // candidate. Admitted, judged, sequence stops.
                        match source_diag::admit(
                            &line,
                            &resp,
                            "mn.at",
                            &source_diag::OperationExpectation::Cold {
                                candidate: &source,
                                step: Some(i as u64),
                            },
                        ) {
                            Ok(fact) => {
                                diag =
                                    Some((fact.primary_code.clone(), fact.legacy_message.clone()));
                                source_fact = Some(fact);
                                let mut rec = ok;
                                rec["parent_window_ms"] = serde_json::json!(
                                    round_started.elapsed().as_secs_f64() * 1000.0
                                );
                                step_records.push(rec);
                                break 'admit;
                            }
                            Err(e) => {
                                infra_detail =
                                    Some(format!("source-v1 admission (run round): {e}"));
                                break 'admit;
                            }
                        }
                    }
                    if let Some(d) = ok.get("diagnostic") {
                        // A diagnosed rejection from the whole-request leg
                        // (a diagnosed rejection never reaches link/run by
                        // design — admitted as a verdict, not infra).
                        whole_fallback = true;
                        diag = Some((
                            d.get("code")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?")
                                .to_string(),
                            d.get("message")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                        ));
                        let mut rec = ok;
                        rec["parent_window_ms"] =
                            serde_json::json!(round_started.elapsed().as_secs_f64() * 1000.0);
                        step_records.push(rec);
                        continue;
                    }
                    let link = ok
                        .get("counters")
                        .and_then(|c| c.get("link_invocations"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let run = ok
                        .get("counters")
                        .and_then(|c| c.get("run_invocations"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    if link < 1 || run < 1 {
                        infra_detail =
                            Some("fallback run violates the always-execute contract".to_string());
                        break 'admit;
                    }
                    let fp = ok
                        .get("manifest_fp")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if fp.is_empty() {
                        infra_detail =
                            Some("fallback success without a manifest fingerprint".to_string());
                        break 'admit;
                    }
                    resp_manifest_fp = fp;
                    let round_exit = ok
                        .get("exit_code")
                        .and_then(|v| v.as_i64())
                        .map(|v| v as i32);
                    if round_exit.is_none() {
                        infra_detail = Some("no judged exit in the fallback run round".to_string());
                        break 'admit;
                    }
                    exit_code = round_exit;
                    stderr_hex = ok
                        .get("stderr_hex")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    stdout_hex = ok
                        .get("stdout_hex")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    whole_fallback = true;
                    let mut rec = ok;
                    rec["parent_window_ms"] =
                        serde_json::json!(round_started.elapsed().as_secs_f64() * 1000.0);
                    step_records.push(rec);
                    continue;
                }
                if ok.get("infra").map(|v| !v.is_null()) == Some(true) {
                    infra_detail = Some(format!("run round {i} infra: {:?}", ok.get("infra")));
                    break 'admit;
                }
                let round_exit = ok
                    .get("exit_code")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);
                if round_exit.is_none() {
                    infra_detail = Some(format!("run round {i} without a real PE exit"));
                    break 'admit;
                }
                exit_code = round_exit;
                stderr_hex = ok
                    .get("stderr_hex")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                stdout_hex = ok
                    .get("stdout_hex")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let mut rec = ok;
                rec["parent_window_ms"] =
                    serde_json::json!(round_started.elapsed().as_secs_f64() * 1000.0);
                step_records.push(rec);
            }
        }
        if infra_detail.is_none() {
            if let Err(e) = rounds.finish() {
                infra_detail = Some(e);
            }
        }
    }
    let worker_us = rounds.worker_us();
    drop(rounds);

    if let Some(e) = infra_detail {
        return Ok(Verdict {
            status: Status::Error,
            mismatches: vec![Mismatch {
                observation: "worker".into(),
                kind: "infra".into(),
                expected: "a contained incremental worker session".into(),
                actual: e,
                first_diff: None,
            }],
            detail: Some(format!("{recipe}: infra failure (incremental)")),
            artifacts: vec![],
            native: None,
        });
    }

    // ---- parent-side judgement (the same faces as the whole-request
    // path; the judged value is the LAST step's PE exit).
    let mut mismatches: Vec<Mismatch> = Vec::new();
    mismatches.extend(decl_mismatches);
    if let Some(spec) = mn.history.as_ref() {
        history_mismatches.extend(judge_history_revisit(&step_records, spec));
    }
    mismatches.extend(history_mismatches);
    if let Some(spec) = mn.turnaround.as_ref() {
        mismatches.extend(judge_turnaround(&parent_windows_ms, spec));
    }
    if let Some((code, message)) = &diag {
        let stage = if let Some(fact) = &source_fact {
            // PLAN-911 strict: the RAW producer stage decides.
            fact.mapped_stage
        } else if code.starts_with("mn.unsupported.frontend")
            || code.starts_with("mn.unsupported.input")
        {
            Stage::Parse
        } else {
            Stage::Verify
        };
        match &oracle.completion {
            Completion::Error {
                stage: want_stage,
                code: want_code,
                ..
            } => {
                if want_stage != &stage {
                    mismatches.push(Mismatch {
                        observation: "stage".into(),
                        kind: "stage".into(),
                        expected: format!("{want_stage:?}"),
                        actual: format!("{stage:?}"),
                        first_diff: None,
                    });
                }
                if let Some(want) = want_code {
                    if want != code {
                        mismatches.push(Mismatch {
                            observation: "code".into(),
                            kind: "code".into(),
                            expected: want.clone(),
                            actual: code.clone(),
                            first_diff: None,
                        });
                    }
                }
            }
            other => {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "outcome".into(),
                    expected: format!("{other:?}"),
                    actual: format!("diagnosed {code}: {message}"),
                    first_diff: None,
                });
            }
        }
    } else {
        match &oracle.completion {
            Completion::Success => {
                if let Some(want) = oracle.exit {
                    if exit_code != Some(want) {
                        mismatches.push(Mismatch {
                            observation: "exit".into(),
                            kind: "exit".into(),
                            expected: format!("{want}"),
                            actual: format!("{exit_code:?}"),
                            first_diff: None,
                        });
                    }
                } else if exit_code.is_none() {
                    mismatches.push(Mismatch {
                        observation: "exit".into(),
                        kind: "missing".into(),
                        expected: "a real PE exit code".into(),
                        actual: "none".into(),
                        first_diff: None,
                    });
                }
                // PLAN-914: the real PE's stderr (the executed mark bytes on
                // the MIR-native scenario path). The worker reports it
                // hex-encoded; the parent decodes and compares it to the
                // declared oracle. A malformed stream is a mismatch, never a
                // silent skip.
                if let Some(want) = &oracle.stderr {
                    match hex_stream(&stderr_hex) {
                        Ok(actual) => {
                            if let Some(m) =
                                compare_bytes(want.text.as_bytes(), &actual, want.compare, "stderr")
                            {
                                mismatches.push(m);
                            }
                        }
                        Err(detail) => mismatches.push(Mismatch {
                            observation: "stderr".into(),
                            kind: "decode".into(),
                            expected: "a hex-encoded PE stderr stream".into(),
                            actual: detail,
                            first_diff: None,
                        }),
                    }
                }
            }
            Completion::Error { stage, .. } => {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "outcome".into(),
                    expected: format!("{stage:?} rejection"),
                    actual: format!("success exit {exit_code:?}"),
                    first_diff: None,
                });
            }
        }
    }
    // F-818-R1-01 residual: seam-supplied independent comparisons are
    // verdicts — a mismatched edit round fails the case (never a silent
    // admission); the comparison already ran inside each round's window.
    for (i, want, got) in expect_mismatches.drain(..) {
        mismatches.push(Mismatch {
            observation: format!("edit round {i} independent exit"),
            kind: "exit".into(),
            expected: format!("{want}"),
            actual: format!("{got:?}"),
            first_diff: None,
        });
    }

    let request_wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    // The whole-request fallback's manifest identity rides the receipt
    // when present (the true incremental shape has none by design).
    let manifest_fp = resp_manifest_fp.as_str();
    let exe_sha = bound_worker_sha(recipe)?;
    let mut receipt = serde_json::json!({
        "recipe": recipe,
        "protocol": MN_WORKER_PROTOCOL,
        "incremental": !whole_fallback,
        "warm_edit_fallback": whole_fallback,
        "exit_code": exit_code,
        "stderr_hex": stderr_hex,
        "stdout_hex": stdout_hex,
        "manifest_fp": manifest_fp,
        "steps": step_records,
        "parent_windows_ms": parent_windows_ms,
        "worker_us": worker_us,
        "request_wall_ms": request_wall_ms,
        "toolchain": {
            "recipe": recipe,
            "mir-lowering": EXPECTED_MIR_LOWERING_VERSION,
            "mn-worker-exe-sha256": exe_sha,
        },
    });
    if recipe == COLLECTIONS_RECIPE {
        receipt["collections_runtime"] = serde_json::json!(collection_runtime);
    }
    std::fs::write(
        artifact_dir.join("mn-probe-receipt.json"),
        serde_json::to_string_pretty(&receipt).unwrap_or_default(),
    )
    .map_err(|e| format!("write receipt: {e}"))?;

    let status = if mismatches.is_empty() {
        Status::Pass
    } else {
        Status::Fail
    };
    // PLAN-911: the admitted typed fact of a strict rounds verdict is
    // published exactly like the whole-request path's (bound to the
    // execution key; a write failure is an error).
    let mut artifacts: Vec<auto_test_case::report::ArtifactRef> = Vec::new();
    if let Some(fact) = &source_fact {
        artifacts.push(source_diag::write_fact_artifact(
            artifact_dir,
            execution_key,
            recipe,
            EXPECTED_MN_WORKER_VERSION,
            fact,
        )?);
    }
    Ok(Verdict {
        status,
        mismatches,
        detail: Some(format!(
            "{recipe}: incremental run judged (exit {exit_code:?}); receipt mn-probe-receipt.json"
        )),
        artifacts,
        native: Some(NativeRecord {
            exit_code,
            stage_spans: vec![
                ("worker_us".to_string(), worker_us),
                ("request_wall_ms".to_string(), request_wall_ms as u64),
            ],
            cache_mode: None,
            cache_hits: vec![],
            counters: vec![],
            toolchain: native_toolchain(recipe, exe_sha, collection_runtime.as_ref()),
            gaps: vec![],
        }),
    })
}

// ---------------------------------------------------------------------------
// PLAN-911 T-03: parent-side strict admission unit controls (V-06/V-18).
// ---------------------------------------------------------------------------
#[cfg(test)]
mod source_diag_tests {
    use super::source_diag::{self, OperationExpectation};
    use serde_json::{json, Value};

    const SRC: &str = "fn main() int { return true + 1 }\n";

    fn sha(bytes: &[u8]) -> String {
        auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(bytes))
    }

    /// A valid cold B17 response (the worker's real shape).
    fn valid_response() -> (String, Value) {
        let snap = json!({
            "schema": "mn.source-diagnostic/1",
            "mode": "source-v1",
            "producer": "ace-core.source",
            "profile": "scalar",
            "attempted": ["scalar"],
            "surface_rejections": [],
            "path": "mn.at",
            "source_sha256": sha(SRC.as_bytes()),
            "source_len": SRC.len(),
            "span_basis": "utf8-byte",
            "causes": [
                {
                    "stage": "verify",
                    "code": "source.type-mismatch",
                    "message": "operator `+` expects i32 operands",
                    "span": { "start": 24, "end": 33 },
                }
            ],
            "primary": 0,
            "operation": {
                "kind": "cold",
                "step": null,
                "base_sha256": null,
                "candidate_sha256": sha(SRC.as_bytes()),
                "session_revision": null,
                "edit": null,
            },
        });
        let resp = json!({
            "schema": "mnworker/1",
            "nonce": "n",
            "request_hash": "h",
            "ok": {
                "recipe": "ac/mir-native-scalar-v1",
                "worker_version": "mnworker/1.0",
                "mn_abi": "msvc-x64/priv-i32-bool-v1",
                "mn_target": "x86_64-pc-windows-msvc",
                "diagnostics_mode": "source-v1",
                "diagnostics_schema": "mn.source-diagnostic/1",
                "counters": {},
                "spans": {},
            },
            "source_error": snap,
            "diagnostic": {
                "code": "mn.unsupported.frontend",
                "message": "Verify/source.type-mismatch: operator `+` expects i32 operands",
            },
        });
        let raw = serde_json::to_string(&resp).expect("serializes");
        (raw, resp)
    }

    fn admit_ok(raw: &str, resp: &Value) -> Result<source_diag::AdmittedFact, String> {
        source_diag::admit(
            raw,
            resp,
            "mn.at",
            &OperationExpectation::Cold {
                candidate: SRC.as_bytes(),
                step: None,
            },
        )
    }

    #[test]
    fn mn911_admit_valid_cold_fact() {
        let (raw, resp) = valid_response();
        let fact = admit_ok(&raw, &resp).expect("admits");
        assert_eq!(fact.raw_stage, "verify");
        assert_eq!(fact.primary_code, "source.type-mismatch");
        assert_eq!(
            fact.mapped_stage,
            auto_test_case::contract::Stage::Typecheck
        );
        assert_eq!(
            fact.legacy_message,
            "Verify/source.type-mismatch: operator `+` expects i32 operands"
        );
    }

    #[test]
    fn mn911_admit_rejects_raw_duplicate_keys_both_orders() {
        // Mutate the WIRE string so `"primary"` appears twice — after the
        // causes array and before it. serde's Map would collapse either;
        // the raw scanner must reject both.
        for order in ["after", "before"] {
            let (raw, _) = valid_response();
            let dup = match order {
                "after" => raw.replace(
                    "\"primary\":0",
                    "\"primary\":0,\"stage\":\"verify\",\"primary\":0",
                ),
                _ => raw.replace("\"primary\":0", "\"primary\":0,\"zz\":1,\"primary\":1"),
            };
            let resp: Value = serde_json::from_str(&dup).expect("parses (collapsed)");
            let err = admit_ok(&dup, &resp).expect_err("duplicate keys are wire-rejected");
            assert!(err.contains("duplicate object key"), "{order}: {err}");
        }
    }

    #[test]
    fn mn911_admit_rejects_forged_primary() {
        for forge in [json!("0"), json!(0.0), json!(false), json!(1), json!(null)] {
            let (raw, mut resp) = valid_response();
            resp["source_error"]["primary"] = forge.clone();
            let r = serde_json::to_string(&resp).unwrap();
            let err = admit_ok(&r, &resp).expect_err("forged primary");
            assert!(err.contains("primary"), "{forge}: {err}");
        }
    }

    #[test]
    fn mn911_admit_rejects_unknown_field_and_missing_field() {
        let (raw, mut resp) = valid_response();
        resp["source_error"]["expected_family"] = json!("type");
        let r = serde_json::to_string(&resp).unwrap();
        let err = admit_ok(&r, &resp).expect_err("unknown field");
        assert!(err.contains("unknown field"), "{err}");

        let (raw, mut resp) = valid_response();
        resp["source_error"]
            .as_object_mut()
            .unwrap()
            .remove("span_basis");
        let r = serde_json::to_string(&resp).unwrap();
        let err = admit_ok(&r, &resp).expect_err("missing field");
        assert!(err.contains("missing fields"), "{err}");
        let _ = raw;
    }

    #[test]
    fn mn911_admit_rejects_non_family_stage() {
        for stage in ["capability", "backend", "link", "run", "unknown-stage"] {
            let (_, mut resp) = valid_response();
            resp["source_error"]["causes"][0]["stage"] = json!(stage);
            let r = serde_json::to_string(&resp).unwrap();
            let err = admit_ok(&r, &resp).expect_err("stage gate");
            assert!(
                err.contains("stage") || err.contains("family"),
                "{stage}: {err}"
            );
        }
    }

    #[test]
    fn mn911_admit_rejects_out_of_bounds_and_misordered_span() {
        for (start, end) in [
            (999usize, 1000usize),
            (33usize, 24usize),
            (0usize, 999usize),
        ] {
            let (_, mut resp) = valid_response();
            resp["source_error"]["causes"][0]["span"] = json!({"start": start, "end": end});
            let r = serde_json::to_string(&resp).unwrap();
            let err = admit_ok(&r, &resp).expect_err("span bounds");
            assert!(err.contains("out of bounds"), "({start},{end}): {err}");
        }
    }

    #[test]
    fn mn911_admit_rejects_over_cap_causes_and_message() {
        let (_, mut resp) = valid_response();
        let mut causes = Vec::new();
        for _ in 0..65 {
            causes.push(json!({
                "stage": "verify", "code": "source.type-mismatch",
                "message": "m", "span": {"start": 0, "end": 1},
            }));
        }
        resp["source_error"]["causes"] = json!(causes);
        let r = serde_json::to_string(&resp).unwrap();
        let err = admit_ok(&r, &resp).expect_err("cause cap");
        assert!(err.contains("cap"), "{err}");

        let (_, mut resp) = valid_response();
        resp["source_error"]["causes"][0]["message"] = json!("m".repeat(4097));
        let r = serde_json::to_string(&resp).unwrap();
        let err = admit_ok(&r, &resp).expect_err("message cap");
        assert!(err.contains("cap"), "{err}");
    }

    #[test]
    fn mn911_admit_rejects_execution_result_mixing() {
        // A source_error response that also claims an execution result.
        let (_, mut resp) = valid_response();
        resp["ok"]["exit_code"] = json!(0);
        let r = serde_json::to_string(&resp).unwrap();
        let err = admit_ok(&r, &resp).expect_err("mutual exclusion");
        assert!(err.contains("must not carry"), "{err}");
    }

    #[test]
    fn mn911_admit_rejects_legacy_projection_mismatch() {
        let (_, mut resp) = valid_response();
        resp["diagnostic"]["message"] = json!("something else");
        let r = serde_json::to_string(&resp).unwrap();
        let err = admit_ok(&r, &resp).expect_err("projection");
        assert!(err.contains("projection"), "{err}");
    }

    #[test]
    fn mn911_admit_rejects_unbound_candidate_and_cold_with_base() {
        // The snapshot claims a different source than the parent sent.
        let (_, mut resp) = valid_response();
        let other = sha(b"fn main() int { return 1 }\n");
        resp["source_error"]["operation"]["candidate_sha256"] = json!(other);
        resp["source_error"]["source_sha256"] = json!(other);
        let r = serde_json::to_string(&resp).unwrap();
        let err = admit_ok(&r, &resp).expect_err("binding");
        assert!(err.contains("bind"), "{err}");

        // A cold operation must not carry warm-session binding.
        let (_, mut resp) = valid_response();
        resp["source_error"]["operation"]["base_sha256"] = json!(sha(b"base"));
        let r = serde_json::to_string(&resp).unwrap();
        let err = admit_ok(&r, &resp).expect_err("cold purity");
        assert!(err.contains("must leave"), "{err}");
    }

    #[test]
    fn mn911_admit_rejects_wrong_path_and_escalation_lie() {
        let (_, mut resp) = valid_response();
        resp["source_error"]["path"] = json!("case-logical-path.at");
        let r = serde_json::to_string(&resp).unwrap();
        let err = admit_ok(&r, &resp).expect_err("path");
        assert!(err.contains("path"), "{err}");

        // attempted must end at the terminating profile.
        let (_, mut resp) = valid_response();
        resp["source_error"]["attempted"] = json!(["scalar", "record"]);
        let r = serde_json::to_string(&resp).unwrap();
        let err = admit_ok(&r, &resp).expect_err("escalation");
        assert!(err.contains("terminating"), "{err}");
    }

    #[test]
    fn mn911_admit_rejects_truncated_source_error() {
        // The wire line ends mid-object: the raw scanner must reject.
        let (raw, resp) = valid_response();
        // Cut MID-object (right before the span_basis key) and add no
        // closing braces: the wire line ends inside source_error.
        let cut = raw.find("\"span_basis\"").expect("anchor");
        let truncated = raw[..cut].to_string();
        let err = admit_ok(&truncated, &resp).expect_err("truncated");
        assert!(
            err.contains("truncated") || err.contains("unbalanced"),
            "{err}"
        );
    }

    #[test]
    fn mn911_admit_warm_edit_binding() {
        let (_, mut resp) = valid_response();
        let base = sha(b"base\n");
        let candidate = sha(b"candidate\n");
        resp["source_error"]["path"] = json!("probe.at");
        resp["source_error"]["source_sha256"] = json!(candidate);
        resp["source_error"]["source_len"] = json!(10);
        resp["source_error"]["causes"][0]["span"] = json!({"start": 0, "end": 4});
        resp["source_error"]["operation"] = json!({
            "kind": "warm-edit",
            "step": 2,
            "base_sha256": base,
            "candidate_sha256": candidate,
            "session_revision": 1,
            "edit": {"start": 0, "end": 4, "insert_len": 3},
        });
        let raw = serde_json::to_string(&resp).unwrap();
        let fact = source_diag::admit(
            &raw,
            &resp,
            "probe.at",
            &OperationExpectation::WarmEdit {
                base: b"base\n",
                candidate: b"candidate\n",
                step: 2,
            },
        )
        .expect("warm-edit admits with the right binding");
        assert_eq!(fact.candidate_sha256, candidate);
        // The step axis is bound to the parent's own index (F-911-R1-01).
        let err = source_diag::admit(
            &raw,
            &resp,
            "probe.at",
            &OperationExpectation::WarmEdit {
                base: b"base\n",
                candidate: b"candidate\n",
                step: 1,
            },
        )
        .expect_err("a step that is not the parent's is rejected");
        assert!(err.contains("step"), "{err}");
    }

    #[test]
    fn mn911_admit_rejects_half_utf8_span_and_accepts_multibyte_control() {
        // A candidate with a multibyte comment: the error span sits after
        // the comment on char boundaries (valid control); a span cutting a
        // multibyte char is rejected on the utf8-byte basis.
        let src = "// 类型\nfn main() int { return true + 1 }\n";
        let (_, mut resp) = valid_response();
        resp["source_error"]["source_sha256"] = json!(sha(src.as_bytes()));
        resp["source_error"]["source_len"] = json!(src.len());
        resp["source_error"]["operation"]["candidate_sha256"] = json!(sha(src.as_bytes()));
        let ok_start = src.find("true").unwrap();
        resp["source_error"]["causes"][0]["span"] = json!({"start": ok_start, "end": ok_start + 8});
        let raw = serde_json::to_string(&resp).unwrap();
        source_diag::admit(
            &raw,
            &resp,
            "mn.at",
            &OperationExpectation::Cold {
                candidate: src.as_bytes(),
                step: None,
            },
        )
        .expect("multibyte source with boundary-aligned span admits");
        // Offset 4 is inside the 3-byte `类` (bytes 3..6): a half-UTF-8 span.
        resp["source_error"]["causes"][0]["span"] = json!({"start": 4, "end": 7});
        let raw = serde_json::to_string(&resp).unwrap();
        let err = source_diag::admit(
            &raw,
            &resp,
            "mn.at",
            &OperationExpectation::Cold {
                candidate: src.as_bytes(),
                step: None,
            },
        )
        .expect_err("half-UTF-8 span rejected");
        assert!(err.contains("char boundaries"), "{err}");
        // source_len lying about the candidate is rejected too.
        resp["source_error"]["causes"][0]["span"] = json!({"start": ok_start, "end": ok_start + 8});
        resp["source_error"]["source_len"] = json!(src.len() + 1);
        let raw = serde_json::to_string(&resp).unwrap();
        let err = source_diag::admit(
            &raw,
            &resp,
            "mn.at",
            &OperationExpectation::Cold {
                candidate: src.as_bytes(),
                step: None,
            },
        )
        .expect_err("source_len mismatch rejected");
        assert!(err.contains("source_len"), "{err}");
    }

    #[test]
    fn mn911_fact_artifact_write_failure_is_an_error() {
        let (raw, resp) = valid_response();
        let fact = admit_ok(&raw, &resp).expect("admits");
        // An artifact directory that cannot exist (a FILE in the way).
        let tmp = std::env::temp_dir().join(format!("mn911-noart-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::write(&tmp, b"not a dir").unwrap();
        let err = source_diag::write_fact_artifact(
            &tmp,
            "s:h:c:k",
            "ac/mir-native-scalar-v1",
            "mnworker/1.0",
            &fact,
        )
        .expect_err("write into a non-directory must error");
        assert!(err.contains("write fact artifact"), "{err}");
        let _ = std::fs::remove_file(&tmp);
        // And a good directory round-trips with the execution key bound.
        let dir = std::env::temp_dir().join(format!("mn911-art-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let art = source_diag::write_fact_artifact(
            &dir,
            "s:h:c:k",
            "ac/mir-native-scalar-v1",
            "mnworker/1.0",
            &fact,
        )
        .expect("writes");
        assert_eq!(art.logical_path, "mn-source-diagnostic.json");
        let doc: Value = serde_json::from_slice(&std::fs::read(&art.stored_path).unwrap()).unwrap();
        assert_eq!(doc["execution_key"], "s:h:c:k");
        assert_eq!(doc["observed_stage"], "typecheck");
        assert_eq!(doc["observed_code"], "source.type-mismatch");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// PLAN-914 (I04, D-05 criterion): judge the declaration-invalidation round
/// against the declared oracle — the cold-frontend invalidations actually
/// recorded, the object-producer unit bound, and the resident MIR/COFF
/// identity carry-over of the units the case names as UNAFFECTED.
/// PLAN-1000 (I11): judge the single-function edit turnaround oracle from
/// the REAL parent-observed per-edit windows (F-818-R1-01: preparation ->
/// IPC -> worker -> admission -> comparison). The P90 of those windows
/// must hold the approved bound; fewer representative edits than declared
/// is a verdict failure, never a silent pass.
fn judge_turnaround(
    parent_windows_ms: &[f64],
    spec: &auto_test_case::contract::ProbeMnTurnaroundSpec,
) -> Vec<Mismatch> {
    let mut out = Vec::new();
    if (parent_windows_ms.len() as u64) < spec.min_edits {
        out.push(Mismatch {
            observation: "edit_turnaround_windows".into(),
            kind: "counter".into(),
            expected: format!(">= {} representative edit windows", spec.min_edits),
            actual: format!("{}", parent_windows_ms.len()),
            first_diff: None,
        });
        return out;
    }
    let mut sorted = parent_windows_ms.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    // nearest-rank P90 (ceil(0.90 * n) - 1)
    let rank = ((90 * sorted.len() + 99) / 100).saturating_sub(1);
    let p90 = sorted[rank];
    if p90 > spec.p90_max_ms as f64 {
        out.push(Mismatch {
            observation: "edit_turnaround_p90_ms".into(),
            kind: "budget".into(),
            expected: format!("<= {} ms (approved P90)", spec.p90_max_ms),
            actual: format!("{p90:.1} ms"),
            first_diff: None,
        });
    }
    out
}

/// PLAN-1000 (I05): judge the content-revisit oracle from the session's
/// REAL counters — the content-addressed COFF cache hits summed across all
/// step records. An edit back to previous content that recompiled instead
/// of reusing the history units fails here, never silently passes.
fn judge_history_revisit(
    step_records: &[serde_json::Value],
    spec: &auto_test_case::contract::ProbeMnHistorySpec,
) -> Vec<Mismatch> {
    let hits: u64 = step_records
        .iter()
        .filter_map(|rec| {
            rec.get("counters")
                .and_then(|c| c.get("cache_hit_coff"))
                .and_then(|v| v.as_u64())
        })
        .sum();
    if hits < spec.min_cache_hit_coff {
        vec![Mismatch {
            observation: "cache_hit_coff".into(),
            kind: "counter".into(),
            expected: format!(
                ">= {} (the revisit run reuses the content-addressed history units)",
                spec.min_cache_hit_coff
            ),
            actual: format!("{hits}"),
            first_diff: None,
        }]
    } else {
        Vec::new()
    }
}

fn judge_declaration_invalidation(
    ok: &serde_json::Value,
    spec: &auto_test_case::contract::ProbeMnDeclarationSpec,
) -> Vec<Mismatch> {
    let mut out = Vec::new();
    let invalidations = ok
        .get("counters")
        .and_then(|c| c.get("declaration_frontend_invalidations"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if invalidations < spec.min_frontend_invalidations {
        out.push(Mismatch {
            observation: "declaration_frontend_invalidations".into(),
            kind: "counter".into(),
            expected: format!(">= {}", spec.min_frontend_invalidations),
            actual: format!("{invalidations}"),
            first_diff: None,
        });
    }
    if let Some(max) = spec.max_codegen_units {
        let emitted = ok
            .get("codegen_units")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        if emitted > max {
            out.push(Mismatch {
                observation: "codegen_units".into(),
                kind: "counter".into(),
                expected: format!("<= {max} (only dependency-affected units enter the producer)"),
                actual: format!("{emitted}"),
                first_diff: None,
            });
        }
    }
    // The carried object identity is the resident MIR and COFF allocation
    // (the tuple's last two components); the leading body identity is
    // reminted by the cold declaration frontend and is not an object axis.
    let carried = |v: &serde_json::Value| -> Option<(u64, u64)> {
        let arr = v.as_array()?;
        if arr.len() < 3 {
            return None;
        }
        Some((arr[1].as_u64()?, arr[2].as_u64()?))
    };
    let before = ok.get("identities_before").and_then(|v| v.as_object());
    let after = ok.get("identities_after").and_then(|v| v.as_object());
    for def in &spec.stable_defs {
        let b = before.and_then(|m| m.get(def)).and_then(carried);
        let a = after.and_then(|m| m.get(def)).and_then(carried);
        match (b, a) {
            (Some(b), Some(a)) if b == a => {}
            (Some(b), Some(a)) => out.push(Mismatch {
                observation: format!("identities[{def}]"),
                kind: "identity".into(),
                expected: format!("resident mir/coff {b:?}"),
                actual: format!("resident mir/coff {a:?}"),
                first_diff: None,
            }),
            _ => out.push(Mismatch {
                observation: format!("identities[{def}]"),
                kind: "missing".into(),
                expected: "a carried resident mir/coff identity".into(),
                actual: "absent".into(),
                first_diff: None,
            }),
        }
    }
    out
}

/// PLAN-932 parent-side unit controls for the typed refusal face: a refusal
/// is a POLICY VERDICT only when its typed kind is present and declared;
/// missing/unknown/contradictory facts are breaches the rounds loop reports
/// as infra ERRORs (never a degraded verdict masking the missing facts).
#[cfg(test)]
mod refusal_tests {
    use serde_json::{json, Value};

    fn round(refusal: Option<Value>, refused: Option<&str>) -> Value {
        let mut ok = json!({ "op": "edit-source" });
        if let Some(r) = refusal {
            ok["refusal"] = r;
        }
        if let Some(s) = refused {
            ok["refused"] = json!(s);
        }
        ok
    }

    fn verdict(ok: &Value) -> Option<Result<(String, String), String>> {
        super::refusal_verdict(ok)
    }

    #[test]
    fn nw932_no_refusal_is_not_a_verdict() {
        assert!(verdict(&round(None, None)).is_none());
        // A round without any refusal keys at all.
        assert!(verdict(&json!({ "op": "run", "exit_code": 7 })).is_none());
    }

    #[test]
    fn nw932_declared_policy_kinds_are_verdicts() {
        for kind in [
            "signature-or-context-drift",
            "invalid-edit",
            "session-or-candidate",
            "lower-admit",
        ] {
            let ok = round(
                Some(json!({ "kind": kind, "reason": "non-fast-path" })),
                Some("non-fast-path"),
            );
            match verdict(&ok) {
                Some(Ok((code, reason))) => {
                    assert_eq!(code, format!("mn.incremental.refused.{kind}"));
                    assert_eq!(reason, "non-fast-path");
                }
                other => panic!("kind {kind}: expected a typed verdict, got {other:?}"),
            }
        }
    }

    #[test]
    fn nw932_untyped_refused_string_is_a_fact_breach() {
        // The default-mode shape: only the legacy `refused` string.
        let ok = round(None, Some("non-fast-path: signature change refused"));
        match verdict(&ok) {
            Some(Err(detail)) => {
                assert!(detail.contains("no `refusal` object"), "{detail}");
            }
            other => panic!("expected a typed-fact breach, got {other:?}"),
        }
    }

    #[test]
    fn nw932_missing_or_malformed_kind_is_a_fact_breach() {
        // `refusal` present but `kind` absent / not a string.
        for bad in [
            json!({ "reason": "r" }),
            json!({ "kind": 7, "reason": "r" }),
            json!({ "kind": null, "reason": "r" }),
        ] {
            let ok = round(Some(bad), None);
            match verdict(&ok) {
                Some(Err(detail)) => {
                    assert!(detail.contains("`refusal.kind`"), "{detail}");
                }
                other => panic!("expected a missing-kind breach, got {other:?}"),
            }
        }
    }

    #[test]
    fn nw932_unknown_kind_is_a_fact_breach() {
        let ok = round(
            Some(json!({ "kind": "stale-range-drift", "reason": "r" })),
            None,
        );
        match verdict(&ok) {
            Some(Err(detail)) => {
                assert!(detail.contains("unknown `refusal.kind`"), "{detail}");
            }
            other => panic!("expected an unknown-kind breach, got {other:?}"),
        }
    }

    #[test]
    fn nw932_untyped_and_resource_kinds_are_breaches() {
        for kind in ["untyped", "resource"] {
            let ok = round(Some(json!({ "kind": kind, "reason": "r" })), None);
            match verdict(&ok) {
                Some(Err(_)) => {}
                other => panic!("kind {kind}: expected a breach, got {other:?}"),
            }
        }
    }

    #[test]
    fn nw932_contradictory_kinds_are_breaches() {
        // A language refusal must arrive as an admitted source_error fact,
        // and syntax-not-converged through the fallback rounds — seeing
        // either on the refusal face is a typed-fact contradiction.
        for kind in ["language-diagnostics", "syntax-not-converged"] {
            let ok = round(Some(json!({ "kind": kind, "reason": "r" })), None);
            match verdict(&ok) {
                Some(Err(detail)) => {
                    assert!(detail.contains(kind), "{detail}");
                }
                other => panic!("kind {kind}: expected a contradiction breach, got {other:?}"),
            }
        }
    }

    #[test]
    fn nw932_last_step_refusal_is_read_in_the_strict_shape() {
        // The strict single-request shape carries the refusal on the last
        // step record; the top-level `ok` has only the steps array.
        let ok = json!({
            "op": "edit-source",
            "steps": [
                { "op": "warmup" },
                { "op": "edit-source", "refusal": { "kind": "signature-or-context-drift", "reason": "non-fast-path" } },
            ],
        });
        match verdict(&ok) {
            Some(Ok((code, _))) => {
                assert_eq!(code, "mn.incremental.refused.signature-or-context-drift")
            }
            other => panic!("expected the last-step typed verdict, got {other:?}"),
        }
    }
}
