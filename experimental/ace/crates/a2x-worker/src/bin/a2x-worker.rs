//! PLAN-892 T-02/T-05 A2X-HIR worker: the PREBUILT tool the registered
//! recipe `ac-a2r/hir-scalar-emit-v1` executes against (the 796 trusted
//! worker identity contract — a case can never name an executable).
//!
//! Protocol (`a2xworker/1`, JSON request on argv[1] or stdin → JSON
//! response on stdout, exit 0 on success / nonzero + {"status":"error"}
//! on controlled failure):
//!
//! * `{"op":"identity"}` → tool/protocol/recipe/backend identity.
//! * `{"op":"daily-emit","source_path","profile":"core|scalar","entry",
//!    "caps":[...],"out_dir","nonce"}` → COLD emit of the full actual file
//!   set (main.rs / a2x_runtime.rs / generated.rs / manifest.json) into
//!   `out_dir`. NO rustc/cargo/target-program spawn (the daily spawn-zero
//!   contract, plan §5.5); nothing is read from any golden — goldens are
//!   judged parent-side.
//! * `{"op":"certify",...}` → the SAME on-the-spot actual set PLUS a real
//!   rustc compile+run of it and of the frozen i32-form reference, each
//!   compared to the caller-supplied closed-form oracle value/trap. The
//!   EXPLICIT certification leg only — never the daily case path.
//!
//! The worker never receives expected values on the daily path; the
//! certify op takes the oracle explicitly because it IS the explicit leg.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use ace_a2x_core::sha256::sha256_hex;
use ace_a2x_core::SemanticView;
use ace_a2x_rust::{
    assemble_collections_file_set, assemble_file_set, assemble_record_text_file_set,
    canonical_execution_config, lower_collections_program, lower_program,
};
use ace_core::source::{
    compile_collections_source, compile_record_source, compile_source, compile_text_record_source,
    compile_text_source,
};

/// PLAN-934 T-08: ordinary scalar source rides the SQ-A fact entry (the
/// query facts are the type authority); the other profiles keep their own
/// authoritative front doors.
fn compile_scalar_product_934(
    src: &str,
    caps: &[String],
) -> Result<ace_core::source::CompiledSource, Vec<ace_core::atom_text::Diagnostic>> {
    // The actual Rust/C CLI shares the same CT door as the product session.
    // Each cold operation owns its existing CT service; no source is reparsed
    // into a legacy deck or evaluated by a second engine.
    if caps
        .iter()
        .any(|cap| cap == ace_core::source::cst::CT_CAPABILITY)
    {
        let mut service = ace_source_cte::CtService::new(ace_source_cte::MemoryEnvelopeSink::new());
        return ace_source_cte::compile_scalar_ct(
            &mut service,
            src,
            "a2x-worker",
            caps,
            "a2x",
            "a2x-worker",
        )
        .map(|product| ace_core::source::CompiledSource {
            profile: "scalar",
            input_hash: product.input_hash,
            atom_text: product.atom_text,
            checked: product.checked,
        })
        .map_err(|error| match error {
            ace_source_cte::CtProductError::Diagnostics(diagnostics) => diagnostics,
            ace_source_cte::CtProductError::Plan(error) => {
                vec![ace_core::atom_text::Diagnostic::new(
                    ace_core::atom_text::Stage::Text,
                    "source.cte.plan",
                    error.to_string(),
                    ace_core::atom_text::Span { start: 0, end: 0 },
                )]
            }
        });
    }
    let mut session = ace_source_query::SourceQuerySession::new("a2x", "a2x-worker");
    ace_source_query::compile_scalar_product(
        src,
        "a2x-worker",
        caps,
        "a2x",
        "a2x-worker",
        &mut session,
    )
    .map(|p| p.compiled)
}

const PROTOCOL: &str = "a2xworker/1";
const RECIPE: &str = "ac-a2r/hir-scalar-emit-v1";
/// PLAN-902: the record/text/text-record channel — parallel to (and
/// mutually untrusted with) the scalar recipe above.
const RT_RECIPE: &str = "ac-a2r/hir-record-text-emit-v1";
/// PLAN-904: the collections channel (List/Variant) — parallel to (and
/// mutually untrusted with) both recipes above.
const COLL_RECIPE: &str = "ac-a2r/hir-collections-emit-v1";
/// PLAN-921 CA-A: the C scalar channel (HIR→C11) — its own recipe,
/// fileset schema and cert; parallel to and mutually untrusted with every
/// Rust recipe above. Served by the same worker binary (one tool
/// identity), dispatched by the `daily-emit-c`/`certify-c` ops.
const C_RECIPE: &str = ace_a2x_c::C_RECIPE;
/// PLAN-925 CA-B: the C record-text channel (HIR→C11) — parallel to and
/// mutually untrusted with every recipe above; dispatched by the same
/// `daily-emit-c`/`certify-c` ops through the source profile.
const C_RT_RECIPE: &str = ace_a2x_c::C_RT_RECIPE;
/// PLAN-926 CA-C: the C collections channel (HIR→C11) — parallel to and
/// mutually untrusted with every recipe above; dispatched by the same
/// `daily-emit-c`/`certify-c` ops through the source profile.
const C_COLL_RECIPE: &str = ace_a2x_c::C_COLL_RECIPE;
/// PLAN-1042: minimal Python TargetProvider channels (`daily-emit-py` / `run-py`).
const PY_RECIPE: &str = "ac-a2py/hir-scalar-emit-v1";
const PY_RT_RECIPE: &str = "ac-a2py/hir-record-text-emit-v1";
const PY_COLL_RECIPE: &str = "ac-a2py/hir-collections-emit-v1";
const PY_STDLIB_RUN_RECIPE: &str = "ac-a2py/hir-stdlib-run-v1";

fn main() {
    let mut req = String::new();
    if let Some(arg) = std::env::args().nth(1) {
        req = arg;
    } else {
        std::io::stdin()
            .read_to_string(&mut req)
            .expect("read request");
    }
    let v: serde_json::Value = match serde_json::from_str(&req) {
        Ok(v) => v,
        Err(e) => fail(&format!("bad request json: {e}")),
    };
    let out = match v["op"].as_str().unwrap_or("") {
        "identity" => identity(&v),
        "daily-emit" => daily_emit(&v),
        "certify" => certify(&v),
        "daily-emit-c" => daily_emit_c(&v),
        "certify-c" => certify_c(&v),
        "daily-emit-py" => daily_emit_py(&v),
        "run-py" => run_py(&v),
        other => fail(&format!("unknown op `{}`", other)),
    };
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}

fn fail(detail: &str) -> ! {
    println!(
        "{}",
        serde_json::json!({ "status": "error", "detail": detail })
    );
    std::process::exit(2);
}

/// PLAN-1000 F-1000-02 (GAP-941-A2X-DIAG-01): a refused source compile
/// ALSO reports the typed diagnostic channel — the structured frontend
/// facts (stage/code/message/span per cause, source identity, first cause
/// primary) instead of only the rendered detail string. The legacy
/// `detail` rendering stays byte-identical for old consumers; the typed
/// document is the semantic authority the negative readers certify from.
/// Every other failure class keeps the plain two-key refusal shape.
fn fail_source_diagnostics(
    detail: &str,
    path: &str,
    src: &str,
    source_sha256: &str,
    profile: &str,
    ds: &[ace_core::atom_text::Diagnostic],
) -> ! {
    let causes: Vec<serde_json::Value> = ds
        .iter()
        .map(|d| {
            serde_json::json!({
                "stage": d.stage.as_str(),
                "code": d.code,
                "message": d.message,
                "span": { "start": d.span.start, "end": d.span.end },
            })
        })
        .collect();
    let diagnostics = serde_json::json!({
        "schema": "a2x.source-diagnostic/1",
        "mode": "typed-negative/1",
        "producer": "a2x-worker",
        "profile": profile,
        "path": path,
        "source_sha256": source_sha256,
        "source_len": src.len(),
        "causes": causes,
        "primary": 0,
    });
    println!(
        "{}",
        serde_json::json!({ "status": "error", "detail": detail, "diagnostics": diagnostics })
    );
    std::process::exit(2);
}

fn ok_base(recipe: &str) -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "tool": "a2x-worker",
        "protocol": PROTOCOL,
        "recipe": recipe,
    })
}

fn identity(v: &serde_json::Value) -> serde_json::Value {
    let t = ace_a2x_rust::TargetProfile::v1();
    let mut identity = serde_json::json!({
        "status": "ok",
        "tool": "a2x-worker",
        "protocol": PROTOCOL,
        "recipe": RECIPE,
        "recipes": [RECIPE, RT_RECIPE, COLL_RECIPE, C_RECIPE, C_RT_RECIPE, PY_RECIPE, PY_RT_RECIPE, PY_COLL_RECIPE, PY_STDLIB_RUN_RECIPE],
        "backend": t.backend,
        "backend_version": t.backend_version,
        "edition": t.edition,
        "targets": ["rust", "c", "python"],
        "c_backend": "auto-a2x-c",
        "c_standard": ace_a2x_c::C_STANDARD,
        "python_backend": "auto-a2x-python",
        "python_producer": ace_a2x_python::PY_PRODUCER,
    });
    // Explicit certification preparation only. Ordinary identity/daily calls
    // keep their original spawn-zero behavior. The composer never calls this.
    if let Some(target) = v.get("certification_target") {
        identity["certification_toolchain"] = match target.as_str() {
            Some("rust") => {
                let vv = toolchain_vv();
                serde_json::json!({ "kind": "rustc", "version_vv_sha256": sha256_hex(vv.as_bytes()),
                    "version_vv": vv, "flags": ["--edition", "2021"] })
            }
            Some("c") => {
                let tc = ace_a2x_worker::crun::toolchain_identity()
                    .unwrap_or_else(|e| fail(&format!("cert toolchain identity: {e}")));
                // Preserve the original schema's field name; its value is
                // the original C locator's BLAKE3 image digest, not SHA256.
                serde_json::json!({ "kind": "cl", "version": tc.cl_version,
                    "version_sha256": tc.cl_hash, "vcvars_sha256": tc.vcvars_sha256,
                    "flags": tc.flags })
            }
            Some("python") => {
                let rt_sha = ace_a2x_python::runtime_body_sha256();
                let py = ace_a2x_worker::pyrun::resolve_python(&rt_sha)
                    .unwrap_or_else(|e| fail(&format!("python identity: {e}")));
                serde_json::json!({
                    "kind": "python",
                    "exe": py.identity.executable,
                    "version": py.identity.version,
                    "version_sha256": py.identity.executable_sha256,
                    "os": py.identity.target_triple,
                })
            }
            _ => fail("certification_target must be rust, c, or python"),
        };
    }
    identity
}

struct EmitOutcome {
    files: Vec<(String, String)>,
    manifest: String,
    input_hash: String,
    entry_fn: String,
    profile: String,
    source_sha256: String,
    /// PLAN-925: the channel's recipe (scalar vs record-text by family).
    recipe: String,
    /// The compiled bundle's HIR domain string (cert identity).
    hir_domain: String,
}

fn read_source(v: &serde_json::Value) -> (String, String) {
    if let Some(p) = v["source_path"].as_str() {
        let text = match std::fs::read_to_string(p) {
            Ok(t) => t,
            Err(e) => fail(&format!("read source {p}: {e}")),
        };
        (
            text,
            sha256_hex(std::fs::read(p).unwrap_or_default().as_slice()),
        )
    } else if let Some(t) = v["source_text"].as_str() {
        let h = sha256_hex(t.as_bytes());
        (t.to_string(), h)
    } else {
        fail("request needs source_path or source_text");
    }
}

fn emit(v: &serde_json::Value) -> EmitOutcome {
    let (src, source_sha256) = read_source(v);
    let profile = v["profile"].as_str().unwrap_or("core").to_string();
    let entry = v["entry"].as_str().unwrap_or("main").to_string();
    let caps: Vec<String> = v["caps"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|c| c.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    // The profile selects BOTH the frontend entry and the recipe channel
    // (PLAN-902: record/text/text-record ride the record-text fileset).
    let compiled = match profile.as_str() {
        "core" => compile_source(&src, "a2x-worker", &caps),
        "scalar" => compile_scalar_product_934(&src, &caps),
        "record" => compile_record_source(&src, "a2x-worker", &caps),
        "text" => compile_text_source(&src, "a2x-worker", &caps),
        "text-record" => compile_text_record_source(&src, "a2x-worker", &caps),
        "collections" => compile_collections_source(&src, "a2x-worker", &caps),
        other => fail(&format!("unknown source profile `{}`", other)),
    };
    let compiled = match compiled {
        Ok(c) => c,
        Err(ds) => {
            let detail = ds
                .iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ");
            fail_source_diagnostics(
                &format!("frontend diagnostics: {}", detail),
                v["source_path"].as_str().unwrap_or(""),
                &src,
                &source_sha256,
                &profile,
                &ds,
            );
        }
    };
    let view = match SemanticView::from_checked(&compiled.checked) {
        Ok(view) => view,
        Err(r) => fail(&format!("view admission: {}", r)),
    };
    let (program, entry_fn) = if is_coll_profile(&profile) {
        match lower_collections_program(&view, &entry) {
            Ok(p) => {
                let e = p.entry.clone();
                (p, e)
            }
            Err(r) => fail(&format!("lowering: {}", r)),
        }
    } else {
        match lower_program(&view, &entry) {
            Ok(p) => {
                let e = p.entry.clone();
                (p, e)
            }
            Err(r) => fail(&format!("lowering: {}", r)),
        }
    };
    let (files, manifest) = if is_coll_profile(&profile) {
        // the collections manifest binds the REPLACED FULL SOURCE sha256
        // and the execution-config identity (never a fragment-only hash)
        let cfg = execution_config_of(v);
        let fs = assemble_collections_file_set(
            &view,
            &program,
            &compiled.input_hash,
            &source_sha256,
            &cfg,
            &entry,
        );
        (fs.files, fs.manifest)
    } else if is_rt_profile(&profile) {
        let fs = assemble_record_text_file_set(&view, &program, &compiled.input_hash, &entry);
        (fs.files, fs.manifest)
    } else {
        let fs = assemble_file_set(&view, &program, &compiled.input_hash, &entry);
        (fs.files, fs.manifest)
    };
    EmitOutcome {
        files,
        manifest,
        input_hash: compiled.input_hash,
        entry_fn,
        profile: view.profile.to_string(),
        source_sha256,
        recipe: RECIPE.to_string(),
        hir_domain: view.domain.as_str().to_string(),
    }
}

/// Whether the profile rides the PLAN-902 record-text channel.
fn is_rt_profile(profile: &str) -> bool {
    matches!(profile, "record" | "text" | "text-record")
}

/// Whether the profile rides the PLAN-904 collections channel.
fn is_coll_profile(profile: &str) -> bool {
    profile == "collections"
}

/// The canonical execution-config identity of a request (PLAN-904 §5.6:
/// fixed key order; any field difference is a different identity).
fn execution_config_of(v: &serde_json::Value) -> String {
    canonical_execution_config(
        v["mode"].as_str().unwrap_or("debug"),
        v["budget_work_units"].as_u64().unwrap_or(16_777_216),
        v["fault_alloc"].as_u64(),
        v["events"].as_bool().unwrap_or(false),
        v["cancel"].as_bool().unwrap_or(false),
    )
}

/// The recipe this profile's responses must echo.
fn recipe_of_profile(profile: &str) -> &'static str {
    if is_coll_profile(profile) {
        COLL_RECIPE
    } else if is_rt_profile(profile) {
        RT_RECIPE
    } else {
        RECIPE
    }
}

fn file_records(files: &[(String, String)]) -> Vec<serde_json::Value> {
    files
        .iter()
        .map(|(p, c)| {
            serde_json::json!({ "path": p, "sha256": sha256_hex(c.as_bytes()), "bytes": c.len() })
        })
        .collect()
}

fn write_out_dir(dir: &str, out: &EmitOutcome) {
    let _ = dir; // callers pass the canonicalized dir already
    let dir = PathBuf::from(dir);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        fail(&format!("create out_dir: {e}"));
    }
    for (p, c) in &out.files {
        if let Err(e) = std::fs::write(dir.join(p), c) {
            fail(&format!("write {p}: {e}"));
        }
    }
    if let Err(e) = std::fs::write(dir.join("manifest.json"), &out.manifest) {
        fail(&format!("write manifest: {e}"));
    }
}

fn daily_emit(v: &serde_json::Value) -> serde_json::Value {
    let out_dir = v["out_dir"]
        .as_str()
        .unwrap_or_else(|| fail("out_dir required"));
    let nonce = v["nonce"].as_str().unwrap_or("");
    let profile = v["profile"].as_str().unwrap_or("core").to_string();
    let out = emit(v);
    write_out_dir(out_dir, &out);
    let mut r = ok_base(recipe_of_profile(&profile));
    r["mode"] = "daily-emit".into();
    r["nonce"] = nonce.into();
    r["profile"] = out.profile.clone().into();
    r["input_hash"] = out.input_hash.clone().into();
    r["source_sha256"] = out.source_sha256.clone().into();
    r["entry_fn"] = out.entry_fn.clone().into();
    r["manifest_sha256"] = sha256_hex(out.manifest.as_bytes()).into();
    r["files"] = file_records(&out.files).into();
    r["spawned_rustc"] = 0.into();
    r
}

/// PLAN-921: the C scalar emit — the SAME provider path the session uses
/// (pure functions: byte-identical to a session cold assembly), without
/// a store. Serves `core`/`scalar` only; other profiles are named
/// rejections (the C channel never widens silently).
fn emit_c(v: &serde_json::Value) -> EmitOutcome {
    use ace_a2x_core::target::{Family, FilesetRequest, FragmentText, TargetProvider};
    let (src, source_sha256) = read_source(v);
    let profile = v["profile"].as_str().unwrap_or("core").to_string();
    let entry = v["entry"].as_str().unwrap_or("main").to_string();
    let caps: Vec<String> = v["caps"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|c| c.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let compiled = match profile.as_str() {
        "core" => compile_source(&src, "a2x-worker", &caps),
        "scalar" => compile_scalar_product_934(&src, &caps),
        // PLAN-925: the record/text/text-record source domains ride the
        // C record-text channel
        "record" => compile_record_source(&src, "a2x-worker", &caps),
        "text" => compile_text_source(&src, "a2x-worker", &caps),
        "text-record" => compile_text_record_source(&src, "a2x-worker", &caps),
        // PLAN-926: the collections source domain rides the C collections
        // channel
        "collections" => compile_collections_source(&src, "a2x-worker", &caps),
        other => fail(&format!(
            "a2x.domain: the C channel serves core/scalar/record/text/text-record/collections, not `{}`",
            other
        )),
    };
    let compiled = match compiled {
        Ok(c) => c,
        Err(ds) => {
            let detail = ds
                .iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ");
            fail_source_diagnostics(
                &format!("a2x.compile: {detail}"),
                v["source_path"].as_str().unwrap_or(""),
                &src,
                &source_sha256,
                &profile,
                &ds,
            );
        }
    };
    let view = match SemanticView::from_checked(&compiled.checked) {
        Ok(v) => v,
        Err(e) => fail(&format!("{}: {}", e.code, e.detail)),
    };
    let provider = ace_a2x_c::CTargetProvider::v1();
    let family = match profile.as_str() {
        "record" | "text" | "text-record" => Family::RecordText,
        "collections" => Family::Collections,
        _ => Family::Scalar,
    };
    let recipe = match family {
        Family::RecordText => C_RT_RECIPE,
        Family::Collections => C_COLL_RECIPE,
        Family::Scalar => C_RECIPE,
    };
    let runtime = match provider.runtime_source(&view, family) {
        Ok(r) => r,
        Err(e) => fail(&format!("{}: {}", e.code, e.detail)),
    };
    let fns: Vec<&ace_a2x_core::FnFacts<'_>> = view.functions().iter().collect();
    let texts = match provider.produce_fragments(&view, &fns, family) {
        Ok(t) => t,
        Err(e) => fail(&format!("{}: {}", e.code, e.detail)),
    };
    let mut sem_fps = Vec::with_capacity(fns.len());
    for f in &fns {
        match ace_a2x_core::semantic_fingerprint(&view, f) {
            Ok(fp) => sem_fps.push(fp),
            Err(e) => fail(&format!("{}: {}", e.code, e.detail)),
        }
    }
    let frags: Vec<FragmentText<'_>> = fns
        .iter()
        .zip(texts.iter())
        .zip(sem_fps.iter())
        .map(|((f, text), fp)| FragmentText {
            def_index: f.def_index,
            source_name: f.source_name,
            sem_fp: fp.as_str(),
            text: text.as_str(),
        })
        .collect();
    // PLAN-925: the record-text assembly needs the record-table seed
    // (captured from the live view)
    let prelude_seed = match provider.capture_prelude_seed(&view, family) {
        Ok(s) => s,
        Err(e) => fail(&format!("{}: {}", e.code, e.detail)),
    };
    let fileset = match provider.assemble_fileset(FilesetRequest {
        entry: &entry,
        input_hash: &compiled.input_hash,
        family,
        fragments: &frags,
        view: Some(&view),
        runtime: &runtime,
        prelude_seed: prelude_seed.as_deref(),
        origins: None,
        source_sha256: Some(&source_sha256),
    }) {
        Ok(fs) => fs,
        Err(e) => fail(&format!("{}: {}", e.code, e.detail)),
    };
    EmitOutcome {
        files: fileset.files,
        manifest: fileset.manifest,
        input_hash: compiled.input_hash.clone(),
        entry_fn: ace_a2x_c::c_fn_name(&entry),
        profile,
        source_sha256,
        recipe: recipe.to_string(),
        hir_domain: compiled.checked.bundle().domain.as_str().to_string(),
    }
}

/// PLAN-921: the daily spawn-zero C emit (no cl, no rustc, no target
/// program) — the parent judges the emitted set against its goldens.
fn daily_emit_c(v: &serde_json::Value) -> serde_json::Value {
    let out_dir = v["out_dir"]
        .as_str()
        .unwrap_or_else(|| fail("out_dir required"));
    let nonce = v["nonce"].as_str().unwrap_or("");
    let out = emit_c(v);
    write_out_dir(out_dir, &out);
    let mut r = ok_base(&out.recipe);
    r["mode"] = "daily-emit-c".into();
    r["target"] = "c".into();
    r["nonce"] = nonce.into();
    r["profile"] = out.profile.clone().into();
    r["input_hash"] = out.input_hash.clone().into();
    r["source_sha256"] = out.source_sha256.clone().into();
    r["entry_fn"] = out.entry_fn.clone().into();
    r["manifest_sha256"] = sha256_hex(out.manifest.as_bytes()).into();
    r["files"] = file_records(&out.files).into();
    r["spawned_rustc"] = 0.into();
    r["spawned_cl"] = 0.into();
    r
}

/// PLAN-921 T-10: the C scalar CERTIFICATION leg - the actual set and
/// the caller-supplied HAND-WRITTEN reference compile and run
/// INDEPENDENTLY under the real MSVC cl, both against the same
/// closed-form oracle; the strict certificate binds the full
/// path/tool/dependency identity. Spawns are EXPECTED here (the
/// certification channel; the daily channel stays spawn-zero).
fn certify_c(v: &serde_json::Value) -> serde_json::Value {
    use ace_a2x_c::cert::{
        BoundFile, CCertDraft, C_CERT_SCHEMA, C_COLL_CERT_SCHEMA, C_RT_CERT_SCHEMA,
    };
    let out_dir = v["out_dir"]
        .as_str()
        .unwrap_or_else(|| fail("out_dir required"));
    let reference_path = v["reference_path"]
        .as_str()
        .unwrap_or_else(|| fail("reference_path required (hand-written C reference)"));
    let oracle_kind = v["oracle_kind"].as_str().unwrap_or("value");
    let oracle_value = match oracle_kind {
        "value" => v["oracle_value"]
            .as_i64()
            .map(|x| x.to_string())
            .unwrap_or_else(|| fail("oracle_value required for kind=value")),
        "trap" => v["oracle_trap"]
            .as_str()
            .unwrap_or_else(|| fail("oracle_trap required for kind=trap"))
            .to_string(),
        other => fail(&format!("oracle_kind must be value|trap, not `{}`", other)),
    };
    let case_id = v["case_id"].as_str().unwrap_or("case");
    let nonce = v["nonce"].as_str().unwrap_or("");
    let expected_line = format!("{}:{}\n", oracle_kind, oracle_value);

    // 1) emit + materialize the actual set
    let out = emit_c(v);
    write_out_dir(out_dir, &out);

    // 2) ACTUAL: real cl compile + run under the contained executor
    let actual_dir = PathBuf::from(out_dir);
    let (actual_compile, actual_run) =
        ace_a2x_worker::crun::compile_and_run_c_observed(&actual_dir, "actual.exe")
            .unwrap_or_else(|e| fail(&format!("actual compile/run: {}", e)));
    let actual_stdout = String::from_utf8_lossy(&actual_run.stdout).into_owned();
    if actual_stdout != expected_line {
        fail(&format!(
            "a2x.oracle: actual observed `{}` != oracle `{}`",
            actual_stdout.trim(),
            expected_line.trim()
        ));
    }

    // 3) REFERENCE: the hand-written C file compiles and runs
    // INDEPENDENTLY (its own translation unit, same contained executor)
    let ref_dir = actual_dir.join("_reference");
    if let Err(e) = std::fs::create_dir_all(&ref_dir) {
        fail(&format!("create reference dir: {e}"));
    }
    let ref_src = std::fs::read_to_string(reference_path)
        .unwrap_or_else(|e| fail(&format!("read reference: {e}")));
    let ref_file = ref_dir.join("reference.c");
    std::fs::write(&ref_file, &ref_src).unwrap_or_else(|e| fail(&format!("write reference: {e}")));
    let ref_exe = ref_dir.join("reference.exe");
    let reference_compile =
        ace_a2x_worker::crun::compile_c_standalone_observed(&ref_file, &ref_exe)
            .unwrap_or_else(|e| fail(&format!("reference compile: {}", e)));
    let ref_run = ace_a2x_worker::crun::run_exe(&ref_exe)
        .unwrap_or_else(|e| fail(&format!("reference run: {}", e)));
    let ref_stdout = String::from_utf8_lossy(&ref_run.stdout).into_owned();
    if ref_stdout != expected_line {
        fail(&format!(
            "a2x.oracle: reference observed `{}` != oracle `{}`",
            ref_stdout.trim(),
            expected_line.trim()
        ));
    }
    let reference_sha256 = sha256_hex(ref_src.as_bytes());

    // 4) the strict certificate
    let toolchain = ace_a2x_worker::crun::toolchain_identity()
        .unwrap_or_else(|e| fail(&format!("toolchain identity: {}", e)));
    let mut actual_files = Vec::new();
    for (p, c) in &out.files {
        actual_files.push(BoundFile {
            path: p.clone(),
            sha256: sha256_hex(c.as_bytes()),
        });
    }
    let wrapper_sha256 = out
        .files
        .iter()
        .find(|(p, _)| p == "main.c")
        .map(|(_, c)| sha256_hex(c.as_bytes()))
        .unwrap_or_default();
    let golden = v["golden_path"]
        .as_str()
        .and_then(|g| std::fs::read_to_string(g).ok());
    let caps: Vec<String> = v["caps"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|c| c.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    // PLAN-925: the cert identities pair by CHANNEL (scalar vs
    // record-text); the case prefix and HIR domain follow the family
    let (schema, recipe_lit, case_prefix, execution_config) = if out.recipe == C_RT_RECIPE {
        (C_RT_CERT_SCHEMA, C_RT_RECIPE, "a2c-hir-925", None)
    } else if out.recipe == C_COLL_RECIPE {
        // PLAN-926: the collections cert carries the canonical
        // execution-config identity (mode/budget/fault/events/cancel
        // are key material — byte-parity with the PLAN-904 Rust
        // collections cert's canonical rendering)
        let mode = v["mode"].as_str().unwrap_or("debug").to_string();
        let budget = v["budget"].as_u64().unwrap_or(16_777_216);
        let fault = v["fault_alloc"].as_u64();
        let events = v["events"].as_bool().unwrap_or(false);
        let cancel = v["cancel"].as_bool().unwrap_or(false);
        (
            C_COLL_CERT_SCHEMA,
            C_COLL_RECIPE,
            "a2c-hir-926",
            Some(ace_a2x_rust::coll::canonical_execution_config(
                &mode, budget, fault, events, cancel,
            )),
        )
    } else {
        (C_CERT_SCHEMA, C_RECIPE, "a2c-hir-921", None)
    };
    let draft = CCertDraft {
        schema: schema.to_string(),
        recipe: recipe_lit.to_string(),
        case: format!("{}/{}", case_prefix, case_id),
        profile: out.profile.clone(),
        hir_domain: out.hir_domain.clone(),
        entry: v["entry"].as_str().unwrap_or("main").to_string(),
        verdict: "PASS".to_string(),
        oracle_kind: oracle_kind.to_string(),
        oracle_value: oracle_value.clone(),
        input_hash: out.input_hash.clone(),
        source_sha256: out.source_sha256.clone(),
        actual_files,
        golden_sha256: golden.map(|g| sha256_hex(g.as_bytes())),
        wrapper_sha256,
        reference_sha256,
        backend: "auto-a2x-c".to_string(),
        backend_version: ace_a2x_c::C_BACKEND_VERSION.to_string(),
        standard: ace_a2x_c::C_STANDARD.to_string(),
        shared_view_version: "0.1.0".to_string(),
        protocol: "a2x-observation/1".to_string(),
        toolchain,
        capabilities: caps,
        execution_config,
    };
    if let Err(e) = draft.validate() {
        fail(&format!(
            "a2x.cert: strict validation rejected the draft: {e}"
        ));
    }
    let cert_json = draft.to_json();
    let cert_path = actual_dir.join("case.cert.json");
    if let Err(e) = std::fs::write(&cert_path, &cert_json) {
        fail(&format!("write cert: {e}"));
    }

    let mut r = ok_base(recipe_lit);
    r["mode"] = "certify-c".into();
    r["target"] = "c".into();
    r["nonce"] = nonce.into();
    r["case"] = format!("{}/{}", case_prefix, case_id).into();
    r["verdict"] = "PASS".into();
    r["oracle_kind"] = oracle_kind.into();
    r["oracle_value"] = oracle_value.into();
    r["actual_stdout"] = actual_stdout.trim().into();
    r["reference_stdout"] = ref_stdout.trim().into();
    r["cert_path"] = cert_path.display().to_string().into();
    r["cert_sha256"] = sha256_hex(cert_json.as_bytes()).into();
    r["spawned_cl"] = 2.into(); // actual + reference compiles
    r["spawned_rustc"] = 0.into();
    if std::env::var("AUTO_CASE_WIRE_EVIDENCE").as_deref() == Ok("1") {
        use ace_a2x_worker::crun::captured_observation;
        r["execution_observations"] = serde_json::json!({
            "schema": "ace.a2x.cert-execution/1",
            "actual": { "compile": captured_observation(&actual_compile),
                        "run": captured_observation(&actual_run) },
            "reference": { "compile": captured_observation(&reference_compile),
                           "run": captured_observation(&ref_run) },
        });
    }
    r
}

fn certify(v: &serde_json::Value) -> serde_json::Value {
    let out_dir = v["out_dir"]
        .as_str()
        .unwrap_or_else(|| fail("out_dir required"));
    let reference_path = {
        let raw = v["reference_path"]
            .as_str()
            .unwrap_or_else(|| fail("reference_path required (the case's hand-written reference)"));
        // F-3: canonicalize at entry — relative paths break nothing
        std::fs::canonicalize(raw)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|e| fail(&format!("canonicalize reference: {e}")))
    };
    let oracle_kind = v["oracle_kind"].as_str().unwrap_or("value");
    let oracle_line = match oracle_kind {
        "value" => format!(
            "value:{}",
            v["oracle_value"]
                .as_i64()
                .unwrap_or_else(|| fail("oracle_value required"))
        ),
        "trap" => format!(
            "trap:{}",
            v["oracle_trap"]
                .as_str()
                .unwrap_or_else(|| fail("oracle_trap required"))
        ),
        other => fail(&format!("oracle_kind must be value|trap, got {other}")),
    };
    let case_id = v["case_id"].as_str().unwrap_or("a2r-hir-892/unnamed");
    let profile = v["profile"].as_str().unwrap_or("core").to_string();
    let rt = is_rt_profile(&profile);
    let out = emit(v);
    write_out_dir(out_dir, &out);
    let dir = PathBuf::from(out_dir);

    // actual program: real rustc + run (stdout value/trap + stderr marks);
    // walls are OBSERVED per the 852 budget lines (compile <=120s, run
    // <=30s, whole certify <=360s) — the caller asserts the caps
    let t0 = std::time::Instant::now();
    let (actual_out, actual_err, actual_processes) =
        compile_and_run(&dir, "main.rs", "a2x_actual.exe");
    let actual_wall_ms = t0.elapsed().as_millis() as u64;
    // independent hand-written reference: compile + run separately
    let reference_src = std::fs::read_to_string(&reference_path)
        .unwrap_or_else(|e| fail(&format!("read reference {reference_path}: {e}")));
    std::fs::write(dir.join("reference.rs"), &reference_src)
        .unwrap_or_else(|e| fail(&format!("write reference: {e}")));
    let t1 = std::time::Instant::now();
    let (reference_out, reference_err, reference_processes) =
        compile_and_run(&dir, "reference.rs", "a2x_reference.exe");
    let reference_wall_ms = t1.elapsed().as_millis() as u64;

    let mut r = ok_base(recipe_of_profile(&profile));
    r["mode"] = "certify".into();
    r["case"] = case_id.into();
    r["actual_observed"] = actual_out.clone().into();
    r["reference_observed"] = reference_out.clone().into();
    r["oracle"] = oracle_line.clone().into();
    let stdout_ok = actual_out == oracle_line && reference_out == oracle_line;
    let marks_ok = actual_err == reference_err;
    r["marks_match_reference"] = marks_ok.into();
    if std::env::var("AUTO_CASE_WIRE_EVIDENCE").as_deref() == Ok("1") {
        r["execution_observations"] = serde_json::json!({
            "schema": "ace.a2x.cert-execution/1",
            "actual": actual_processes, "reference": reference_processes,
        });
    }
    let verdict = if stdout_ok && marks_ok {
        "PASS"
    } else {
        "REJECTED"
    };
    r["verdict"] = verdict.into();
    if verdict == "REJECTED" {
        r["detail"] = format!(
            "actual={:?} reference={:?} oracle={:?} marks_actual={:?} marks_ref={:?}",
            actual_out, reference_out, oracle_line, actual_err, reference_err
        )
        .into();
    }

    // the strict certificate draft binds the full actual file set, the
    // reference bytes and the OBSERVED toolchain identity (schema/recipe
    // follow the profile's channel)
    let vv = toolchain_vv();
    let (cert_schema, cert_recipe) = if is_coll_profile(&profile) {
        (ace_a2x_rust::COLL_CERT_SCHEMA, COLL_RECIPE)
    } else if rt {
        (ace_a2x_rust::RT_CERT_SCHEMA, ace_a2x_rust::RT_RECIPE)
    } else {
        (ace_a2x_rust::CERT_SCHEMA, ace_a2x_rust::RECIPE)
    };
    let cert = ace_a2x_rust::CertDraft {
        schema: cert_schema,
        recipe: cert_recipe,
        case: case_id.to_string(),
        profile: out.profile.clone(),
        hir_domain: out.hir_domain.clone(),
        entry: v["entry"].as_str().unwrap_or("main").to_string(),
        verdict: verdict.to_string(),
        oracle_kind: if oracle_kind == "trap" {
            "trap"
        } else {
            "value"
        },
        oracle_value: if oracle_kind == "trap" {
            v["oracle_trap"].as_str().unwrap_or("").to_string()
        } else {
            v["oracle_value"].as_i64().unwrap_or(0).to_string()
        },
        input_hash: out.input_hash.clone(),
        source_sha256: out.source_sha256.clone(),
        actual_files: out
            .files
            .iter()
            .map(|(p, c)| ace_a2x_rust::BoundFile {
                path: p.clone(),
                sha256: sha256_hex(c.as_bytes()),
            })
            .collect(),
        golden_sha256: None,
        wrapper_sha256: sha256_hex(
            out.files
                .iter()
                .find(|(p, _)| p == "main.rs")
                .map(|(_, c)| c.as_bytes())
                .unwrap_or(&[]),
        ),
        reference_sha256: sha256_hex(reference_src.as_bytes()),
        backend: "auto-a2x-rust",
        backend_version: "0.1.0",
        shared_view_version: "0.1.0",
        protocol: "a2x-observation/1",
        toolchain: ace_a2x_rust::ToolchainIdentity::observed(
            vv,
            vec!["--edition".into(), "2021".into()],
        ),
        capabilities: v["caps"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|c| c.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        execution_config: if is_coll_profile(&profile) {
            Some(execution_config_of(v))
        } else {
            None
        },
    };
    if let Err(e) = cert.validate_as(cert_schema, cert_recipe) {
        fail(&format!("certificate draft invalid: {e}"));
    }
    let cert_json = cert.to_json();
    std::fs::write(dir.join("case.cert.json"), &cert_json)
        .unwrap_or_else(|e| fail(&format!("write cert: {e}")));
    r["cert_sha256"] = sha256_hex(cert_json.as_bytes()).into();
    r["actual_wall_ms"] = actual_wall_ms.into();
    r["reference_wall_ms"] = reference_wall_ms.into();
    r["files"] = file_records(&out.files).into();
    r
}

fn check_py_admission(v: &serde_json::Value) {
    if v["use_legacy_python_trans"].as_bool() == Some(true) {
        fail("admission.legacy_python_trans: legacy PythonTrans AST emitter is forbidden on ace-a2x-python");
    }
    if v["use_vm"].as_bool() == Some(true) {
        fail("admission.avm_fallback: AutoVM fallback is forbidden on ace-a2x-python");
    }
    if v.get("expected").is_some() {
        fail("admission.self_oracle: expected value must be verified parent-side, never supplied to worker");
    }
}

fn bytes_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

fn emit_py(v: &serde_json::Value) -> (EmitOutcome, ace_a2x_core::FileSet) {
    use ace_a2x_core::target::{Family, FilesetRequest, FragmentText, TargetProvider};
    check_py_admission(v);
    let (src, source_sha256) = read_source(v);
    let profile = v["profile"].as_str().unwrap_or("core").to_string();
    let entry = v["entry"].as_str().unwrap_or("main").to_string();
    let caps: Vec<String> = v["caps"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|c| c.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let compiled = match profile.as_str() {
        "core" => compile_source(&src, "a2x-worker", &caps),
        "scalar" => compile_scalar_product_934(&src, &caps),
        "record" => compile_record_source(&src, "a2x-worker", &caps),
        "text" => compile_text_source(&src, "a2x-worker", &caps),
        "text-record" => compile_text_record_source(&src, "a2x-worker", &caps),
        "collections" => compile_collections_source(&src, "a2x-worker", &caps),
        other => fail(&format!(
            "a2x.domain: the Python channel serves core/scalar/record/text/text-record/collections, not `{}`",
            other
        )),
    };
    let compiled = match compiled {
        Ok(c) => c,
        Err(ds) => {
            let detail = ds
                .iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ");
            fail_source_diagnostics(
                &format!("a2x.compile: {detail}"),
                v["source_path"].as_str().unwrap_or(""),
                &src,
                &source_sha256,
                &profile,
                &ds,
            );
        }
    };
    let view = match SemanticView::from_checked(&compiled.checked) {
        Ok(v) => v,
        Err(e) => fail(&format!("{}: {}", e.code, e.detail)),
    };
    let provider = ace_a2x_python::PythonTargetProvider;
    let family = match profile.as_str() {
        "record" | "text" | "text-record" => Family::RecordText,
        "collections" => Family::Collections,
        _ => Family::Scalar,
    };
    let expected_schema = match family {
        Family::Scalar => ace_a2x_python::PY_FILESET_SCHEMA,
        Family::RecordText => ace_a2x_python::PY_RT_FILESET_SCHEMA,
        Family::Collections => ace_a2x_python::PY_COLL_FILESET_SCHEMA,
    };
    let recipe = match v["recipe"].as_str() {
        Some(r)
            if r == PY_RECIPE
                || r == PY_RT_RECIPE
                || r == PY_COLL_RECIPE
                || r == PY_STDLIB_RUN_RECIPE =>
        {
            r
        }
        Some(other) => fail(&format!("unregistered python recipe `{other}`")),
        None => match family {
            Family::RecordText => PY_RT_RECIPE,
            Family::Collections => PY_COLL_RECIPE,
            Family::Scalar => PY_RECIPE,
        },
    };
    let runtime = match provider.runtime_source(&view, family) {
        Ok(r) => r,
        Err(e) => fail(&format!("{}: {}", e.code, e.detail)),
    };
    let fns: Vec<&ace_a2x_core::FnFacts<'_>> = view.functions().iter().collect();
    let texts = match provider.produce_fragments(&view, &fns, family) {
        Ok(t) => t,
        Err(e) => fail(&format!("{}: {}", e.code, e.detail)),
    };
    let mut sem_fps = Vec::with_capacity(fns.len());
    for f in &fns {
        match ace_a2x_core::semantic_fingerprint(&view, f) {
            Ok(fp) => sem_fps.push(fp),
            Err(e) => fail(&format!("{}: {}", e.code, e.detail)),
        }
    }
    let frags: Vec<FragmentText<'_>> = fns
        .iter()
        .zip(texts.iter())
        .zip(sem_fps.iter())
        .map(|((f, text), fp)| FragmentText {
            def_index: f.def_index,
            source_name: f.source_name,
            sem_fp: fp.as_str(),
            text: text.as_str(),
        })
        .collect();
    let prelude_seed = match provider.capture_prelude_seed(&view, family) {
        Ok(s) => s,
        Err(e) => fail(&format!("{}: {}", e.code, e.detail)),
    };
    let fileset = match provider.assemble_fileset(FilesetRequest {
        entry: &entry,
        input_hash: &compiled.input_hash,
        family,
        fragments: &frags,
        view: Some(&view),
        runtime: &runtime,
        prelude_seed: prelude_seed.as_deref(),
        origins: None,
        source_sha256: Some(&source_sha256),
    }) {
        Ok(fs) => fs,
        Err(e) => fail(&format!("{}: {}", e.code, e.detail)),
    };
    if let Err(reason) = ace_a2x_python::verify_python_fileset_manifest(
        &fileset.manifest,
        &fileset.files,
        ace_a2x_python::PY_PRODUCER,
        expected_schema,
    ) {
        fail(&format!("a2py.manifest: {reason}"));
    }
    let outcome = EmitOutcome {
        files: fileset.files.clone(),
        manifest: fileset.manifest.clone(),
        input_hash: compiled.input_hash.clone(),
        entry_fn: ace_a2x_python::py_fn_name(&entry),
        profile,
        source_sha256,
        recipe: recipe.to_string(),
        hir_domain: compiled.checked.bundle().domain.as_str().to_string(),
    };
    (outcome, fileset)
}

/// PLAN-1042: spawn-zero Python fileset emit (`execution_claimed == false`).
fn daily_emit_py(v: &serde_json::Value) -> serde_json::Value {
    let out_dir = v["out_dir"]
        .as_str()
        .unwrap_or_else(|| fail("out_dir required"));
    let nonce = v["nonce"].as_str().unwrap_or("");
    let (out, _fs) = emit_py(v);
    write_out_dir(out_dir, &out);
    let mut r = ok_base(&out.recipe);
    r["mode"] = "daily-emit-py".into();
    r["target"] = "python".into();
    r["producer"] = ace_a2x_python::PY_PRODUCER.into();
    r["runtime_sha256"] = ace_a2x_python::runtime_body_sha256().into();
    r["nonce"] = nonce.into();
    r["profile"] = out.profile.clone().into();
    r["input_hash"] = out.input_hash.clone().into();
    r["source_sha256"] = out.source_sha256.clone().into();
    r["entry_fn"] = out.entry_fn.clone().into();
    r["manifest_sha256"] = sha256_hex(out.manifest.as_bytes()).into();
    r["files"] = file_records(&out.files).into();
    r["spawned_python"] = 0.into();
    r["spawned_rustc"] = 0.into();
    r["spawned_cl"] = 0.into();
    r["execution_claimed"] = false.into();
    r
}

/// PLAN-1042: real OS Python execution of files emitted by `PythonTargetProvider`.
fn run_py(v: &serde_json::Value) -> serde_json::Value {
    if v["emit_only"].as_bool() == Some(true) {
        fail("admission.emit_only: emit_only cannot claim Python execution");
    }
    let out_dir = v["out_dir"]
        .as_str()
        .unwrap_or_else(|| fail("out_dir required"));
    let nonce = v["nonce"].as_str().unwrap_or("");
    let rt_sha = ace_a2x_python::runtime_body_sha256();
    let py = match ace_a2x_worker::pyrun::resolve_python(&rt_sha) {
        Ok(p) => p,
        Err(e) => fail(&format!("python.missing: {e}")),
    };
    let (out, _fileset) = emit_py(v);
    write_out_dir(out_dir, &out);
    let exec = match ace_a2x_worker::pyrun::run_py_fileset(&py, Path::new(out_dir), &[]) {
        Ok(e) => e,
        Err(e) => fail(&format!("python.exec: {e}")),
    };
    let mut r = ok_base(&out.recipe);
    r["mode"] = "run-py".into();
    r["target"] = "python".into();
    r["producer"] = ace_a2x_python::PY_PRODUCER.into();
    r["runtime_sha256"] = rt_sha.into();
    r["nonce"] = nonce.into();
    r["profile"] = out.profile.clone().into();
    r["input_hash"] = out.input_hash.clone().into();
    r["source_sha256"] = out.source_sha256.clone().into();
    r["entry_fn"] = out.entry_fn.clone().into();
    r["manifest_sha256"] = sha256_hex(out.manifest.as_bytes()).into();
    r["files"] = file_records(&out.files).into();
    r["spawned_python"] = 1.into();
    r["spawned_rustc"] = 0.into();
    r["spawned_cl"] = 0.into();
    r["execution_claimed"] = true.into();
    r["python_identity"] = serde_json::json!({
        "exe": py.identity.executable,
        "version": py.identity.version,
        "version_sha256": py.identity.executable_sha256,
        "os": py.identity.target_triple,
    });
    r["stdout"] = String::from_utf8_lossy(&exec.stdout).to_string().into();
    r["stderr"] = String::from_utf8_lossy(&exec.stderr).to_string().into();
    r["stdout_hex"] = bytes_hex(&exec.stdout).into();
    r["stderr_hex"] = bytes_hex(&exec.stderr).into();
    r["exit_code"] = exec
        .exit_code
        .map(serde_json::Value::from)
        .unwrap_or(serde_json::Value::Null);
    r
}

fn toolchain_vv() -> String {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let out = Command::new(&rustc)
        .arg("-vV")
        .output()
        .unwrap_or_else(|e| fail(&format!("rustc -vV: {e}")));
    if !out.status.success() {
        fail("rustc -vV failed");
    }
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// 852 budget lines (plan §5.6): compile <=120s, run <=30s; streams are
/// capped at 1MiB each (overflow terminates the tree — never an unbounded
/// reader). F-1 (review): the ac-core contained executor provides the
/// CREATE_SUSPENDED + Job-subtree + single-deadline + full-collection
/// contract; a timeout kills the WHOLE owned tree and is a controlled
/// refusal, never a hang.
const COMPILE_DEADLINE_SECS: u64 = 120;
const RUN_DEADLINE_SECS: u64 = 30;
const STREAM_CAP: usize = 1024 * 1024;

fn contained(cmd: &mut Command, deadline: std::time::Duration) -> ace_core::link::CapturedRun {
    use ace_core::link::{run_deadlined_captured, CaptureCaps, SubprocError};
    match run_deadlined_captured(
        cmd,
        deadline,
        CaptureCaps {
            stdout_max: STREAM_CAP,
            stderr_max: STREAM_CAP,
        },
    ) {
        Ok(r) => r,
        Err(SubprocError::Deadline) => fail("deadline: the owned tree was killed before finishing"),
        Err(SubprocError::Spawn(e)) => fail(&format!("spawn: {e}")),
        Err(SubprocError::ContainmentUnavailable(d)) => {
            fail(&format!("containment unavailable: {d}"))
        }
        Err(other) => fail(&format!("contained run failed: {other:?}")),
    }
}

fn compile_and_run(dir: &Path, src: &str, exe_name: &str) -> (String, String, serde_json::Value) {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let exe = dir.join(exe_name);
    let mut compile = Command::new(&rustc);
    compile
        .args(["--edition", "2021", "-o"])
        .arg(&exe)
        .arg(src)
        .current_dir(dir);
    let out = contained(
        &mut compile,
        std::time::Duration::from_secs(COMPILE_DEADLINE_SECS),
    );
    if out.exit_code != Some(0) {
        fail(&format!(
            "{src} compile failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let mut run = Command::new(&exe);
    run.current_dir(dir);
    let run = contained(&mut run, std::time::Duration::from_secs(RUN_DEADLINE_SECS));
    if run.exit_code != Some(0) {
        fail(&format!("{exe_name} exit non-zero"));
    }
    (
        String::from_utf8_lossy(&run.stdout).trim().to_string(),
        String::from_utf8_lossy(&run.stderr).trim().to_string(),
        if std::env::var("AUTO_CASE_WIRE_EVIDENCE").as_deref() == Ok("1") {
            use ace_a2x_worker::crun::captured_observation;
            serde_json::json!({ "compile": captured_observation(&out),
                                "run": captured_observation(&run) })
        } else {
            serde_json::Value::Null
        },
    )
}
