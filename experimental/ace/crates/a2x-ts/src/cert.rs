//! PLAN-976: strict script (TypeScript / JavaScript) certification identity,
//! expectation/outcome structs, and the dependency-free fileset-manifest
//! integrity verifier.
//!
//! This is the TypeScript/JavaScript counterpart of the PLAN-1042 Python leg
//! (`a2x-python::cert`): the verification SEMANTICS are carried over unchanged
//! (hand-rolled manifest scanning with no JSON dependency, exact schema and
//! producer match, exact `(path, sha256)` fileset with no extra/missing entry),
//! while every identity string moves to the `a2ts` namespace and foreign or
//! legacy script generators are refused by name.
//!
//! Certificate schemas (documented, frozen):
//!
//! * [`TS_CERT_SCHEMA`] — `ace.a2ts-hir.scalar.cert/1` (scalar runs);
//! * [`TS_RT_CERT_SCHEMA`] — `ace.a2ts-hir.record-text.cert/1` (RecordText runs);
//! * [`TS_COLL_CERT_SCHEMA`] — `ace.a2ts-hir.collections.cert/1` (Collections runs).
//!
//! They mirror the per-family fileset schemas of [`crate::target`]
//! (`ace.a2ts-hir.<family>.fileset/1`) so a certificate and the fileset it
//! certifies always share one family name.

use ace_a2x_core::A2xReject;
use ace_core::runtime_contract::sha256_hex;

use crate::target::TS_PRODUCER;

/// Reject code for every certification/identity failure of this target.
const CERT_REJECT_CODE: &str = "a2ts.cert";

/// Certificate schema for scalar TypeScript / JavaScript runs.
pub const TS_CERT_SCHEMA: &str = "ace.a2ts-hir.scalar.cert/1";
/// Certificate schema for RecordText TypeScript / JavaScript runs.
pub const TS_RT_CERT_SCHEMA: &str = "ace.a2ts-hir.record-text.cert/1";
/// Certificate schema for Collections TypeScript / JavaScript runs.
pub const TS_COLL_CERT_SCHEMA: &str = "ace.a2ts-hir.collections.cert/1";

/// The only manifest schemas this target admits: the three certificate
/// schemas plus the three fileset schemas they mirror (a fileset manifest is
/// verified with its own family schema; a certificate receipt uses the
/// `.cert` spelling). No foreign-family schema is admitted even if a manifest
/// happens to carry it verbatim.
const ADMITTED_CERT_SCHEMAS: &[&str] = &[
    TS_CERT_SCHEMA,
    TS_RT_CERT_SCHEMA,
    TS_COLL_CERT_SCHEMA,
    crate::target::TS_SCALAR_FILESET_SCHEMA,
    crate::target::TS_RT_FILESET_SCHEMA,
    crate::target::TS_COLL_FILESET_SCHEMA,
];

/// Legacy generator origins that must never appear in a certified manifest.
const FORBIDDEN_ORIGINS: &[&str] = &[
    "TypeScriptTrans",
    "JavaScriptTrans",
    "crates/auto-lang-core/src/trans/typescript",
    "crates/auto-lang-core/src/trans/javascript",
];

fn cert_reject(detail: impl Into<String>) -> A2xReject {
    A2xReject::new(CERT_REJECT_CODE, detail)
}

/// Concrete script runtime identity bound into run artifacts and cache keys.
///
/// Same shape as the Python leg's identity with one addition: the target
/// certifies both the engine executable and the emitted runtime module, so the
/// runtime body VERSION is part of the fingerprint as well as its body SHA-256.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptRuntimeIdentity {
    /// Runtime engine identity (`node` today; the field exists so a future
    /// certified engine cannot collide with it).
    pub engine: String,
    /// Engine version as reported by the engine itself (`node --version`).
    pub version: String,
    /// SHA-256 of the engine executable that executed the artifacts.
    pub executable_sha256: String,
    /// Version tag of the emitted runtime module (`A2TS_RUNTIME_VERSION`).
    pub runtime_version: String,
    /// SHA-256 of the emitted runtime module body (`a2x_rt.mjs`).
    pub runtime_sha256: String,
    /// Engine target description (`<platform>-<arch>`, e.g. `win32-x64`).
    pub target_triple: String,
}

impl ScriptRuntimeIdentity {
    /// Deterministic fingerprint string used for artifact invalidation keys.
    /// Fields are `\0`-separated in declaration order; like the Python leg, the
    /// fingerprint is a SHA-256 over the exact identity bytes.
    pub fn identity_key(&self) -> String {
        sha256_hex(
            format!(
                "{}\0{}\0{}\0{}\0{}\0{}",
                self.engine,
                self.version,
                self.executable_sha256,
                self.runtime_version,
                self.runtime_sha256,
                self.target_triple
            )
            .as_bytes(),
        )
    }
}

/// Independent expectation oracle for a script test execution (never
/// self-reported by target output).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptExecExpectation {
    pub expected_value: Option<i32>,
    pub expected_trap: Option<String>,
    pub expected_stdout_sha256: Option<String>,
    pub expected_stdout_exact: Option<Vec<u8>>,
    pub expected_effect_count: Option<usize>,
}

/// Captured real script execution outcome from a contained OS process.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptExecOutcome {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub value: Option<i32>,
    pub trap: Option<String>,
    pub events_json: Option<String>,
}

/// Verify that `manifest` binds the expected certificate `schema`, the trusted
/// producer [`TS_PRODUCER`], and exactly the `(path, sha256)` pairs of `files`.
///
/// Semantics (carried over from the Python leg, plus the "no extra files" half
/// of the fileset contract):
///
/// * `producer` must be [`TS_PRODUCER`] exactly — a caller cannot certify
///   artifacts on behalf of another target (e.g. `legacy-TypeScriptTrans`);
/// * `schema` must be one of [`ADMITTED_CERT_SCHEMAS`];
/// * no legacy generator origin may appear anywhere in the manifest;
/// * the manifest's top-level `schema` / `producer` strings must match exactly;
/// * every provided file must have a `files[]` entry with the matching SHA-256,
///   and every `files[]` entry must correspond to a provided file.
///
/// The manifest is scanned by hand (no JSON dependency); anything the scanner
/// cannot read exactly — escapes, duplicate keys, truncated text, non-canonical
/// digests — is a named rejection, never a silent pass.
pub fn verify_script_fileset_manifest(
    manifest: &str,
    files: &[(String, String)],
    producer: &str,
    schema: &str,
) -> Result<(), A2xReject> {
    if producer != TS_PRODUCER {
        return Err(cert_reject(format!(
            "untrusted script producer `{producer}` (expected `{TS_PRODUCER}`)"
        )));
    }
    if !ADMITTED_CERT_SCHEMAS.contains(&schema) {
        return Err(cert_reject(format!(
            "foreign certificate schema `{schema}` (expected one of {ADMITTED_CERT_SCHEMAS:?})"
        )));
    }
    for origin in FORBIDDEN_ORIGINS {
        if manifest.contains(origin) {
            return Err(cert_reject(format!(
                "legacy script generator origin `{origin}` forbidden in an a2ts manifest"
            )));
        }
    }

    let parsed = parse_manifest(manifest)?;
    let declared_schema = parsed
        .schema
        .ok_or_else(|| cert_reject(format!("manifest missing the schema `{schema}`")))?;
    if declared_schema != schema {
        return Err(cert_reject(format!(
            "manifest schema `{declared_schema}` does not match the expected `{schema}`"
        )));
    }
    let declared_producer = parsed
        .producer
        .ok_or_else(|| cert_reject(format!("manifest missing the producer `{producer}`")))?;
    if declared_producer != producer {
        return Err(cert_reject(format!(
            "manifest producer `{declared_producer}` does not match the expected `{producer}`"
        )));
    }

    // Provided files must be unique by path: two different byte streams for one
    // path would make the digest check order-dependent.
    let mut provided: Vec<(&str, &str)> = Vec::with_capacity(files.len());
    for (path, content) in files {
        if provided.iter().any(|(seen, _)| *seen == path.as_str()) {
            return Err(cert_reject(format!("duplicate provided file `{path}`")));
        }
        provided.push((path.as_str(), content.as_str()));
    }

    let entries = parsed.files.unwrap_or_default();

    for (path, content) in &provided {
        let path: &str = *path;
        let digest = sha256_hex(content.as_bytes());
        match entries
            .iter()
            .find(|(entry_path, _)| entry_path.as_str() == path)
        {
            None => {
                return Err(cert_reject(format!(
                    "manifest has no files[] entry for provided file `{path}`"
                )))
            }
            Some((_, declared)) if *declared != digest => {
                return Err(cert_reject(format!(
                    "manifest digest mismatch for `{path}` (the provided bytes are not the certified ones)"
                )))
            }
            Some(_) => {}
        }
    }
    for (path, _) in &entries {
        if !provided
            .iter()
            .any(|(provided_path, _)| *provided_path == path.as_str())
        {
            return Err(cert_reject(format!(
                "manifest certifies extra file `{path}` that was not provided"
            )));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Hand-rolled canonical manifest scanner (no JSON dependency)
// ---------------------------------------------------------------------------

/// Top-level string fields of one fileset manifest.
struct ParsedManifest {
    schema: Option<String>,
    producer: Option<String>,
    files: Option<Vec<(String, String)>>,
}

/// Minimal scanner for the canonical fileset-manifest shape emitted by this
/// provider. It understands exactly: one root object, string values, nested
/// objects/arrays skipped verbatim, and a `files` array of
/// `{ "path": ..., "sha256": ... }` entries. Escape sequences are refused (a
/// canonical manifest has none), so a manifest that needs JSON unescaping can
/// never be mistaken for a certified one.
struct ManifestScanner<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> ManifestScanner<'a> {
    fn new(text: &'a str) -> Self {
        ManifestScanner {
            bytes: text.as_bytes(),
            pos: 0,
        }
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.bytes.get(self.pos) {
            if c.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn at_end(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn expect_byte(&mut self, want: u8, what: &str) -> Result<(), A2xReject> {
        self.skip_ws();
        if self.peek() == Some(want) {
            self.pos += 1;
            Ok(())
        } else {
            Err(cert_reject(format!(
                "malformed manifest: expected `{}` {what}",
                want as char
            )))
        }
    }

    fn string(&mut self, what: &str) -> Result<String, A2xReject> {
        self.skip_ws();
        if self.peek() != Some(b'"') {
            return Err(cert_reject(format!(
                "malformed manifest: expected a string {what}"
            )));
        }
        self.pos += 1;
        let start = self.pos;
        loop {
            match self.peek() {
                None => return Err(cert_reject(format!("truncated manifest string {what}"))),
                Some(b'"') => {
                    let raw = std::str::from_utf8(&self.bytes[start..self.pos])
                        .map_err(|_| cert_reject(format!("non-UTF-8 manifest string {what}")))?;
                    let out = raw.to_string();
                    self.pos += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    return Err(cert_reject(format!(
                        "JSON escapes are not admitted in a canonical manifest ({what})"
                    )))
                }
                Some(_) => self.pos += 1,
            }
        }
    }

    /// Skip a non-string value verbatim (nested object/array/number/bool/null).
    fn skip_value(&mut self) -> Result<(), A2xReject> {
        self.skip_ws();
        match self.peek() {
            Some(b'"') => {
                self.string("as a value")?;
                Ok(())
            }
            Some(b'{') | Some(b'[') => {
                let mut depth = 0usize;
                loop {
                    self.skip_ws();
                    match self.peek() {
                        None => return Err(cert_reject("truncated manifest value")),
                        Some(b'"') => {
                            self.string("inside a nested value")?;
                        }
                        Some(b'{') | Some(b'[') => {
                            depth += 1;
                            self.pos += 1;
                        }
                        Some(b'}') | Some(b']') => {
                            if depth == 0 {
                                return Err(cert_reject(
                                    "malformed manifest: unbalanced nested value",
                                ));
                            }
                            depth -= 1;
                            self.pos += 1;
                            if depth == 0 {
                                return Ok(());
                            }
                        }
                        Some(_) => self.pos += 1,
                    }
                }
            }
            Some(_) => {
                // Bare token: number / true / false / null.
                while let Some(c) = self.peek() {
                    if c == b',' || c == b'}' || c == b']' || c.is_ascii_whitespace() {
                        break;
                    }
                    self.pos += 1;
                }
                Ok(())
            }
            None => Err(cert_reject("truncated manifest value")),
        }
    }

    /// Parse the `files` array: `[ { "path": ..., "sha256": ... }, ... ]`.
    fn files_array(&mut self) -> Result<Vec<(String, String)>, A2xReject> {
        self.expect_byte(b'[', "to open `files`")?;
        let mut entries: Vec<(String, String)> = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(entries);
        }
        loop {
            let (path, digest) = self.files_entry()?;
            if entries.iter().any(|(seen, _)| *seen == path) {
                return Err(cert_reject(format!("duplicate files[] entry for `{path}`")));
            }
            entries.push((path, digest));
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(entries);
                }
                _ => {
                    return Err(cert_reject(
                        "malformed manifest: expected `,` or `]` in `files`",
                    ))
                }
            }
        }
    }

    fn files_entry(&mut self) -> Result<(String, String), A2xReject> {
        self.expect_byte(b'{', "to open a files[] entry")?;
        let mut path: Option<String> = None;
        let mut digest: Option<String> = None;
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
        } else {
            loop {
                let key = self.string("as a files[] entry key")?;
                self.expect_byte(b':', "after a files[] entry key")?;
                self.skip_ws();
                match key.as_str() {
                    "path" => {
                        if path.is_some() {
                            return Err(cert_reject("duplicate `path` in a files[] entry"));
                        }
                        path = Some(self.string("as a files[] path")?);
                    }
                    "sha256" => {
                        if digest.is_some() {
                            return Err(cert_reject("duplicate `sha256` in a files[] entry"));
                        }
                        digest = Some(self.string("as a files[] sha256")?);
                    }
                    _ => self.skip_value()?,
                }
                self.skip_ws();
                match self.peek() {
                    Some(b',') => self.pos += 1,
                    Some(b'}') => {
                        self.pos += 1;
                        break;
                    }
                    _ => {
                        return Err(cert_reject(
                            "malformed manifest: expected `,` or `}` in a files[] entry",
                        ))
                    }
                }
            }
        }
        let path = path.ok_or_else(|| cert_reject("files[] entry without a `path`"))?;
        let digest = digest
            .ok_or_else(|| cert_reject(format!("files[] entry for `{path}` without a `sha256`")))?;
        validate_canonical_sha256(&path, &digest)?;
        Ok((path, digest))
    }
}

/// A canonical digest is exactly 64 lowercase hex characters (what
/// [`sha256_hex`] emits); anything else is rejected rather than compared.
fn validate_canonical_sha256(path: &str, digest: &str) -> Result<(), A2xReject> {
    let canonical = digest.len() == 64
        && digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    if canonical {
        Ok(())
    } else {
        Err(cert_reject(format!(
            "files[] entry for `{path}` has a non-canonical sha256 `{digest}`"
        )))
    }
}

/// Parse the root object of a canonical fileset manifest. Only top-level
/// `schema` / `producer` string fields and the `files` array are interpreted;
/// every other value is skipped, including nested objects that happen to carry
/// their own `schema` key (e.g. a port catalogue).
fn parse_manifest(manifest: &str) -> Result<ParsedManifest, A2xReject> {
    let mut sc = ManifestScanner::new(manifest);
    sc.expect_byte(b'{', "at the start of the manifest")?;
    let mut parsed = ParsedManifest {
        schema: None,
        producer: None,
        files: None,
    };
    sc.skip_ws();
    if sc.peek() == Some(b'}') {
        sc.pos += 1;
    } else {
        loop {
            let key = sc.string("as a top-level key")?;
            sc.expect_byte(b':', "after a top-level key")?;
            sc.skip_ws();
            match key.as_str() {
                "files" => {
                    if parsed.files.is_some() {
                        return Err(cert_reject("duplicate top-level `files` key"));
                    }
                    parsed.files = Some(sc.files_array()?);
                }
                "schema" => {
                    if parsed.schema.is_some() {
                        return Err(cert_reject("duplicate top-level `schema` key"));
                    }
                    parsed.schema = Some(sc.string("as the schema value")?);
                }
                "producer" => {
                    if parsed.producer.is_some() {
                        return Err(cert_reject("duplicate top-level `producer` key"));
                    }
                    parsed.producer = Some(sc.string("as the producer value")?);
                }
                _ => sc.skip_value()?,
            }
            sc.skip_ws();
            match sc.peek() {
                Some(b',') => sc.pos += 1,
                Some(b'}') => {
                    sc.pos += 1;
                    break;
                }
                _ => {
                    return Err(cert_reject(
                        "malformed manifest: expected `,` or `}` after a top-level field",
                    ))
                }
            }
        }
    }
    sc.skip_ws();
    if !sc.at_end() {
        return Err(cert_reject("trailing bytes after the manifest object"));
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GENERATED: &str = "export function a2x_f() {\n  return 1;\n}\n";
    const MAIN: &str = "import { a2x_f } from \"./generated.mjs\";\n";

    fn files() -> Vec<(&'static str, &'static str)> {
        vec![("generated.mjs", GENERATED), ("a2x_main.mjs", MAIN)]
    }

    fn provided(files: &[(&str, &str)]) -> Vec<(String, String)> {
        files
            .iter()
            .map(|(p, c)| ((*p).to_string(), (*c).to_string()))
            .collect()
    }

    /// A manifest shaped exactly like `provider::assemble_fileset` emits it:
    /// top-level string fields, a `functions` array of objects (including an
    /// `origin` pair), and the `files` array last. The nested `trace` object
    /// carries its own `schema` key on purpose: the top-level scanner must not
    /// confuse it with the certificate schema.
    fn build_manifest(schema: &str, producer: &str, files: &[(&str, &str)]) -> String {
        let mut body = String::new();
        for (i, (path, content)) in files.iter().enumerate() {
            let comma = if i + 1 < files.len() { "," } else { "" };
            body.push_str(&format!(
                "    {{ \"path\": \"{path}\", \"sha256\": \"{}\" }}{comma}\n",
                sha256_hex(content.as_bytes())
            ));
        }
        format!(
            "{{\n  \"schema\": \"{schema}\",\n  \"producer\": \"{producer}\",\n  \
             \"backend\": \"auto-a2x-typescript\",\n  \"backend_version\": \"0.1.0\",\n  \
             \"target_lang\": \"typescript\",\n  \"standard\": \"typescript5.9-es2022-erasable\",\n  \
             \"runtime_version\": \"a2ts-runtime/1\",\n  \"runtime_sha256\": \"{runtime_sha}\",\n  \
             \"print_version\": \"a2ts-print/1\",\n  \"stdlib_port_catalog\": \"{catalog}\",\n  \
             \"input_hash\": \"abc\",\n  \"source_sha256\": \"def\",\n  \"entry\": \"main\",\n  \
             \"trace\": {{ \"schema\": \"ace.a2ts-trace/9\", \"marks\": [\"mark_a\"] }},\n  \
             \"functions\": [\n    {{ \"decl\": \"main\", \"sem_fp\": \"fp1\", \"origin\": [0, 12] }}\n  ],\n  \
             \"files\": [\n{body}  ]\n}}\n",
            runtime_sha = "1".repeat(64),
            catalog = "2".repeat(64),
        )
    }

    #[test]
    fn valid_manifest_passes() {
        let files = files();
        let manifest = build_manifest(TS_CERT_SCHEMA, TS_PRODUCER, &files);
        verify_script_fileset_manifest(&manifest, &provided(&files), TS_PRODUCER, TS_CERT_SCHEMA)
            .expect("a canonical manifest must verify");
        // All three admitted schemas verify, and an empty fileset is legal.
        for schema in ADMITTED_CERT_SCHEMAS {
            let m = build_manifest(schema, TS_PRODUCER, &files);
            verify_script_fileset_manifest(&m, &provided(&files), TS_PRODUCER, schema)
                .expect("admitted schema must verify");
        }
        let empty = build_manifest(TS_COLL_CERT_SCHEMA, TS_PRODUCER, &[]);
        verify_script_fileset_manifest(&empty, &[], TS_PRODUCER, TS_COLL_CERT_SCHEMA)
            .expect("empty fileset must verify");
    }

    #[test]
    fn foreign_producer_rejected() {
        let files = files();
        let manifest = build_manifest(TS_CERT_SCHEMA, TS_PRODUCER, &files);
        let err = verify_script_fileset_manifest(
            &manifest,
            &provided(&files),
            "legacy-TypeScriptTrans",
            TS_CERT_SCHEMA,
        )
        .expect_err("a foreign producer argument must be rejected");
        assert_eq!(err.code, "a2ts.cert");
        assert!(
            err.detail.contains("untrusted script producer"),
            "{}",
            err.detail
        );

        // A manifest that itself declares a foreign (but non-legacy) producer.
        let foreign = build_manifest(TS_CERT_SCHEMA, "ace-a2x-python/modules/1", &files);
        let err = verify_script_fileset_manifest(
            &foreign,
            &provided(&files),
            TS_PRODUCER,
            TS_CERT_SCHEMA,
        )
        .expect_err("a foreign manifest producer must be rejected");
        assert_eq!(err.code, "a2ts.cert");
        assert!(
            err.detail.contains("does not match the expected"),
            "{}",
            err.detail
        );

        // The legacy generator token anywhere in the manifest is refused too.
        let legacy = build_manifest(TS_CERT_SCHEMA, TS_PRODUCER, &files)
            .replace("auto-a2x-typescript", "legacy-TypeScriptTrans");
        let err =
            verify_script_fileset_manifest(&legacy, &provided(&files), TS_PRODUCER, TS_CERT_SCHEMA)
                .expect_err("a legacy generator origin must be rejected");
        assert_eq!(err.code, "a2ts.cert");
        assert!(
            err.detail.contains("legacy script generator origin"),
            "{}",
            err.detail
        );
    }

    #[test]
    fn wrong_schema_rejected() {
        let files = files();
        // Caller asks for the RecordText schema while the manifest certifies scalar.
        let manifest = build_manifest(TS_CERT_SCHEMA, TS_PRODUCER, &files);
        let err = verify_script_fileset_manifest(
            &manifest,
            &provided(&files),
            TS_PRODUCER,
            TS_RT_CERT_SCHEMA,
        )
        .expect_err("schema mismatch must be rejected");
        assert_eq!(err.code, "a2ts.cert");
        assert!(
            err.detail.contains("does not match the expected"),
            "{}",
            err.detail
        );

        // A Python-leg schema is not in this target's allowlist at all.
        let py = build_manifest("ace.a2py-hir.scalar.cert/1", TS_PRODUCER, &files);
        let err = verify_script_fileset_manifest(
            &py,
            &provided(&files),
            TS_PRODUCER,
            "ace.a2py-hir.scalar.cert/1",
        )
        .expect_err("a foreign certificate schema must be rejected");
        assert_eq!(err.code, "a2ts.cert");
        assert!(
            err.detail.contains("foreign certificate schema"),
            "{}",
            err.detail
        );

        // Version drift inside the namespace is refused as well.
        let drifted = build_manifest("ace.a2ts-hir.scalar.cert/2", TS_PRODUCER, &files);
        let err = verify_script_fileset_manifest(
            &drifted,
            &provided(&files),
            TS_PRODUCER,
            TS_CERT_SCHEMA,
        )
        .expect_err("schema version drift must be rejected");
        assert_eq!(err.code, "a2ts.cert");
    }

    #[test]
    fn tampered_file_bytes_rejected() {
        let files = files();
        let manifest = build_manifest(TS_CERT_SCHEMA, TS_PRODUCER, &files);
        let mut tampered = provided(&files);
        tampered[0]
            .1
            .push_str("// extra byte injected after certification\n");
        let err = verify_script_fileset_manifest(&manifest, &tampered, TS_PRODUCER, TS_CERT_SCHEMA)
            .expect_err("tampered bytes must be rejected");
        assert_eq!(err.code, "a2ts.cert");
        assert!(err.detail.contains("digest mismatch"), "{}", err.detail);
    }

    #[test]
    fn missing_file_entry_rejected() {
        let files = files();
        // Manifest certifies only the first file, caller provides both.
        let partial = build_manifest(TS_CERT_SCHEMA, TS_PRODUCER, &files[..1]);
        let err = verify_script_fileset_manifest(
            &partial,
            &provided(&files),
            TS_PRODUCER,
            TS_CERT_SCHEMA,
        )
        .expect_err("a missing files[] entry must be rejected");
        assert_eq!(err.code, "a2ts.cert");
        assert!(err.detail.contains("no files[] entry"), "{}", err.detail);

        // Manifest certifies an extra file the caller did not provide.
        let full = build_manifest(TS_CERT_SCHEMA, TS_PRODUCER, &files);
        let err = verify_script_fileset_manifest(
            &full,
            &provided(&files[..1]),
            TS_PRODUCER,
            TS_CERT_SCHEMA,
        )
        .expect_err("an extra certified file must be rejected");
        assert_eq!(err.code, "a2ts.cert");
        assert!(
            err.detail.contains("certifies extra file"),
            "{}",
            err.detail
        );
    }

    #[test]
    fn malformed_manifest_rejected() {
        let files = files();
        let good = build_manifest(TS_CERT_SCHEMA, TS_PRODUCER, &files);
        for broken in [
            String::new(),
            "{ not json".to_string(),
            good.replace("\"sha256\"", "\"sha256\\u0000\""),
            good.replace("\"files\"", "\"filez\""),
            format!("{good}}}"),
        ] {
            let err = verify_script_fileset_manifest(
                &broken,
                &provided(&files),
                TS_PRODUCER,
                TS_CERT_SCHEMA,
            )
            .expect_err("malformed manifests must be rejected");
            assert_eq!(err.code, "a2ts.cert", "detail: {}", err.detail);
        }
        // Duplicate provided path is an error even though the digests match one.
        let mut dup = provided(&files);
        dup.push(dup[0].clone());
        let err = verify_script_fileset_manifest(&good, &dup, TS_PRODUCER, TS_CERT_SCHEMA)
            .expect_err("duplicate provided file must be rejected");
        assert!(
            err.detail.contains("duplicate provided file"),
            "{}",
            err.detail
        );
    }

    #[test]
    fn runtime_identity_key_is_stable_and_field_sensitive() {
        let identity = ScriptRuntimeIdentity {
            engine: "node".to_string(),
            version: "v25.2.1".to_string(),
            executable_sha256: "0".repeat(64),
            runtime_version: "a2ts-runtime/1".to_string(),
            runtime_sha256: "1".repeat(64),
            target_triple: "win32-x64".to_string(),
        };
        let key = identity.identity_key();
        assert_eq!(key.len(), 64);
        assert_eq!(key, identity.clone().identity_key());
        assert!(key.bytes().all(|b| b.is_ascii_hexdigit()));

        let mut other = identity.clone();
        other.runtime_version = "a2ts-runtime/2".to_string();
        assert_ne!(key, other.identity_key());
        let mut other = identity.clone();
        other.runtime_sha256 = "2".repeat(64);
        assert_ne!(key, other.identity_key());
        let mut other = identity;
        other.version = "v26.0.0".to_string();
        assert_ne!(key, other.identity_key());
    }
}
