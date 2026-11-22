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
    assemble_collections_file_set, assemble_file_set, assemble_language_file_set,
    assemble_record_text_file_set, canonical_execution_config, lower_collections_program,
    lower_language_program, lower_program, LANG_RECIPE,
};
use ace_core::source::{
    compile_collections_source, compile_language_source, compile_record_source, compile_source,
    compile_text_record_source, compile_text_source,
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
/// PLAN-1115: the ordinary-language RUN lane — the corpus rows' execution
/// oracle axis (source -> fileset -> real rustc -> run; the parent judges
/// exit/stdout/stderr). Distinct identity from the emit lane above.
const LANG_RUN_RECIPE: &str = "ac-a2r/hir-language-run-v1";
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
/// PLAN-976: the TypeScript/JavaScript channels (one shared provider crate;
/// each manifest carries its concrete `backend`). Dispatched by the
/// `daily-emit-ts`/`run-ts` ops with `target_lang` selecting the leg.
const TS_RECIPE: &str = ace_a2x_ts::TS_RECIPE;
const TS_RT_RECIPE: &str = ace_a2x_ts::TS_RT_RECIPE;
const TS_COLL_RECIPE: &str = ace_a2x_ts::TS_COLL_RECIPE;
const JS_RECIPE: &str = ace_a2x_ts::JS_RECIPE;
const JS_RT_RECIPE: &str = ace_a2x_ts::JS_RT_RECIPE;
const JS_COLL_RECIPE: &str = ace_a2x_ts::JS_COLL_RECIPE;

fn main() {
    // PLAN-1034 IPC-03 负控缝（TEST-ONLY，镜像 sema worker 的
    // SEMA_TEST_* 先例；生产路径永不设置）。具名模式由父测试经子进程
    // env 显式提供——stub 身份即本预构建 worker（hash 可钉），case
    // 文件永远不能凭空指定可执行文件。
    run_ipc1034_fault_seams();
    let req = read_request();
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
        "daily-emit-ts" => daily_emit_ts(&v),
        "run-ts" => run_ts(&v),
        // PLAN-1115 T-05: the language RUN lane (corpus rows' oracle axis).
        "run-language" => run_language(&v),
        other => fail(&format!("unknown op `{}`", other)),
    };
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}

/// PLAN-1034 §5 r4 DEC-IPC-LIMIT：一次请求的原始 JSON UTF-8 字节上限
/// （argv 与 stdin 两入口同门；含空白/尾LF，不含传输EOF；超限在 serde
/// 解析前拒绝）。8MiB=8388608；等于上限可进入下一步，超 1 字节即资源
/// 拒绝。数值参照 TestCase 文档硬上限，但对象是 worker 私有请求信封，
/// 不是 8MiB 源码上限。
const REQUEST_MAX_BYTES: usize = 8 << 20;
/// 选中文件源（source_path / certify reference_path / certify-c
/// golden_path）的原始内容上限 64MiB=67108864：一次 cap+1 有界快照转
/// UTF-8 并 hash/compile（metadata 不预读；实际以 cap+1 读防文件增长）。
const SOURCE_FILE_MAX_BYTES: usize = 64 << 20;

/// 读 one-shot 请求（argv 或 stdin）。argv 先按 OsStr 借用长度准入再
/// 创建 String（非 UTF-8 具名拒绝）；stdin 以 cap+1 有界读取——必须读
/// 完整个 one-shot 文档，不能把第一行当请求。
fn read_request() -> String {
    if let Some(arg) = std::env::args_os().nth(1) {
        if arg.len() > REQUEST_MAX_BYTES {
            fail(&format!(
                "a2x.request-limit: argv request {} bytes exceeds the 8MiB cap ({REQUEST_MAX_BYTES}); rejected before parsing",
                arg.len()
            ));
        }
        match arg.into_string() {
            Ok(s) => s,
            Err(_) => fail("a2x.request-utf8: argv request is not valid unicode"),
        }
    } else {
        read_request_bounded(std::io::stdin(), REQUEST_MAX_BYTES)
    }
}

fn read_request_bounded(mut r: impl std::io::Read, cap: usize) -> String {
    match read_request_bounded_result(&mut r, cap) {
        Ok(s) => s,
        Err(m) => fail(&m),
    }
}

/// 有界 one-shot 读取的 Result 核心（`a2x.request-*` 具名拒绝；测试直
/// 调本函数断言拒绝文本，不触发进程退出）。
fn read_request_bounded_result(r: &mut dyn std::io::Read, cap: usize) -> Result<String, String> {
    let mut buf = Vec::new();
    let mut limited = r.take((cap + 1) as u64);
    limited
        .read_to_end(&mut buf)
        .map_err(|e| format!("a2x.request-io: read stdin request: {e}"))?;
    if buf.len() > cap {
        return Err(format!(
            "a2x.request-limit: request {} bytes exceeds the cap ({cap}); rejected before parsing",
            buf.len()
        ));
    }
    String::from_utf8(buf).map_err(|e| format!("a2x.request-utf8: {e}"))
}

/// IPC-03 负控具名模式（见 main 注释）：
/// - `hang`：读请求前永不退出（父 exec deadline 终止）；
/// - `flood-stdout` / `flood-stderr`：持续超量双流（父流 cap 终止）；
/// - `bad-json`：返回畸形 JSON 后退 0（父协议拒绝，非 crash 掩盖）；
/// - `orphan-holds-pipe`：派生继承 stdout 的后代（re-exec `hang`）后
///   根退 0——父退出而孙进程持管道的树终止反例；PID 见
///   `A2X_TEST_PID_FILE`（若提供，两行 root/descendant）。
fn run_ipc1034_fault_seams() {
    let mode = std::env::var("A2X_TEST_FAULT").unwrap_or_default();
    let note_pid = |label: &str, pid: u32| {
        if let Ok(path) = std::env::var("A2X_TEST_PID_FILE") {
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            {
                let _ = writeln!(f, "{label} {pid}");
            }
        }
    };
    match mode.as_str() {
        "hang" => {
            note_pid("root", std::process::id());
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
            }
        }
        "flood-stdout" => {
            note_pid("root", std::process::id());
            let chunk = [b'x'; 4096];
            use std::io::Write;
            let mut out = std::io::stdout();
            loop {
                if out.write_all(&chunk).is_err() {
                    std::process::exit(0);
                }
            }
        }
        "flood-stderr" => {
            note_pid("root", std::process::id());
            let chunk = [b'e'; 4096];
            use std::io::Write;
            let mut out = std::io::stderr();
            loop {
                if out.write_all(&chunk).is_err() {
                    std::process::exit(0);
                }
            }
        }
        "bad-json" => {
            note_pid("root", std::process::id());
            println!("{{not-json-at-all");
            std::process::exit(0);
        }
        // PLAN-1034: 合法 JSON 响应但回显错误 recipe（父侧
        // trust-but-verify 身份门的负控）。
        "wrong-identity" => {
            note_pid("root", std::process::id());
            println!(
                "{{\"status\":\"ok\",\"tool\":\"a2x-worker\",\"protocol\":\"{PROTOCOL}\",                 \"recipe\":\"ac-a2r/hir-record-text-emit-v1\"}}"
            );
            std::process::exit(0);
        }
        "orphan-holds-pipe" => {
            note_pid("root", std::process::id());
            let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("a2x-worker"));
            let _child = std::process::Command::new(&exe)
                .env("A2X_TEST_FAULT", "hang")
                .env_remove("A2X_TEST_PID_FILE")
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::null())
                .spawn();
            if let Ok(child) = &_child {
                note_pid("descendant", child.id());
            }
            std::process::exit(0);
        }
        _ => {}
    }
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
        "recipes": [RECIPE, RT_RECIPE, COLL_RECIPE, C_RECIPE, C_RT_RECIPE, PY_RECIPE, PY_RT_RECIPE, PY_COLL_RECIPE, PY_STDLIB_RUN_RECIPE, TS_RECIPE, TS_RT_RECIPE, TS_COLL_RECIPE, JS_RECIPE, JS_RT_RECIPE, JS_COLL_RECIPE],
        "backend": t.backend,
        "backend_version": t.backend_version,
        "edition": t.edition,
        "targets": ["rust", "c", "python", "typescript", "javascript"],
        "c_backend": "auto-a2x-c",
        "c_standard": ace_a2x_c::C_STANDARD,
        "python_backend": "auto-a2x-python",
        "python_producer": ace_a2x_python::PY_PRODUCER,
        "ts_backend": ace_a2x_ts::TS_BACKEND,
        "ts_standard": ace_a2x_ts::TS_STANDARD,
        "ts_producer": ace_a2x_ts::TS_PRODUCER,
        "js_backend": ace_a2x_ts::JS_BACKEND,
        "js_standard": ace_a2x_ts::JS_STANDARD,
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
            // PLAN-976: the script engine identity (node) for the TS/JS legs.
            Some("typescript") | Some("javascript") => {
                let rt_sha = ace_a2x_ts::runtime_body_sha256();
                let node = ace_a2x_worker::tsrun::resolve_node(&rt_sha)
                    .unwrap_or_else(|e| fail(&format!("node identity: {e}")));
                serde_json::json!({
                    "kind": "node",
                    "engine": node.identity.engine,
                    "exe": node.program.to_string_lossy(),
                    "version": node.identity.version,
                    "version_sha256": node.identity.executable_sha256,
                    "runtime_version": node.identity.runtime_version,
                    "runtime_sha256": node.identity.runtime_sha256,
                    "os": node.identity.target_triple,
                })
            }
            _ => fail("certification_target must be rust, c, python, typescript, or javascript"),
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
    /// PLAN-1116 (T-03): a prepared foreign provider rides the request —
    /// (provider out dir, DLL file name) for the link/run closure.
    foreign_lib: Option<(String, String)>,
}

/// PLAN-1034 IPC-01：一次 cap+1 有界快照——text/hash/compile 消费同一
/// bytes。旧实现 `read_to_string` + `fs::read` 双读存在 TOCTOU（文件在
/// 两读之间变化时报告的 SHA 与实际编译文本脱钩）且无界；本函数消除第
/// 二次读取：IO/超限/坏编码具名拒绝，绝不以 hash(empty) 或截断文本成功。
/// `source_path` 优先级保持；未选中的 `source_text` 不作前置内容门。
fn read_source(v: &serde_json::Value) -> (String, String) {
    if let Some(p) = v["source_path"].as_str() {
        read_bounded_utf8(p, SOURCE_FILE_MAX_BYTES, "source")
    } else if let Some(t) = v["source_text"].as_str() {
        // 选中内联文本：64MiB 内容门由整请求 8MiB 信封先行约束（实际
        // 可内联大小取交集；r4 R-1034-01 删除了额外 1MiB 字段门）。
        let h = sha256_hex(t.as_bytes());
        (t.to_string(), h)
    } else {
        fail("request needs source_path or source_text");
    }
}

/// 一次有界快照读取：open → take(cap+1) → read_to_end → cap 门 → 同
/// bytes hash → 严格 UTF-8 解码（解码失败具名拒绝，不 lossy）。
fn read_bounded_utf8(path: &str, cap: usize, what: &str) -> (String, String) {
    match read_bounded_utf8_result(path, cap, what) {
        Ok(pair) => pair,
        Err(m) => fail(&m),
    }
}

/// 有界快照的 Result 核心（测试直调断言具名拒绝文本）。
fn read_bounded_utf8_result(
    path: &str,
    cap: usize,
    what: &str,
) -> Result<(String, String), String> {
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("a2x.{what}-io: open {path}: {e}"))?;
    let mut bytes = Vec::new();
    let mut limited = (&mut file).take((cap + 1) as u64);
    limited
        .read_to_end(&mut bytes)
        .map_err(|e| format!("a2x.{what}-io: read {path}: {e}"))?;
    if bytes.len() > cap {
        return Err(format!(
            "a2x.{what}-limit: {path} is {} bytes; exceeds the {}-byte cap; rejected before decode",
            bytes.len(),
            cap
        ));
    }
    let hash = sha256_hex(&bytes);
    String::from_utf8(bytes)
        .map(|s| (s, hash))
        .map_err(|e| format!("a2x.{what}-utf8: {path}: {e}"))
}

/// PLAN-1116 (T-03): the request `foreign` section (declarative: kind +
/// provider; the prepared closure carries every artifact identity) — the
/// same shape the AV lane consumes.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ForeignSpec {
    /// `rust` (`c` keeps the named rejection on this lane for this batch).
    pub kind: String,
    /// Provider id (`f0_text` on the frozen checked-shim domain).
    pub provider: String,
    /// The prepared artifact root (must exist; the run phase never
    /// compiles — missing/stale artifacts are named preparation failures).
    pub artifact_root: String,
}

/// The prepared provider closure (schema-strict), mirroring the AV lane's
/// descriptor — one schema, one identity axis.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderDescriptor {
    pub schema: String,
    pub kind: String,
    pub provider: String,
    /// SHA-256 hex of the provider DLL the run binds.
    pub artifact_sha256: String,
    /// The frozen provider source (rust), base64.
    #[serde(default)]
    pub source_b64: Option<String>,
    /// The rustc identity the prepared Rust build used (stamp axis).
    #[serde(default)]
    pub rustc_identity: Option<String>,
    #[serde(default)]
    pub dll: Option<String>,
    #[serde(default)]
    pub lib: Option<String>,
    #[serde(default)]
    pub stamp: Option<String>,
}

const PROVIDER_DESCRIPTOR_SCHEMA: &str = "plan1093.provider-descriptor/1";

/// PLAN-1116 (T-03): resolve + verify the prepared provider closure (the
/// RUST branch of the AV lane's loader, without the in-process binding —
/// the A2R lane links the checked shim exports at rustc time and loads
/// the DLL as a process dependency). Every identity axis is checked
/// before the compile front door sees the admitted foreign: descriptor
/// schema/kind/provider, the frozen-source-derived digests (re-derived
/// here, never trusted from the descriptor), the prepare stamp, the
/// artifacts' presence, and the DLL's sha256.
fn load_prepared_foreign(
    v: &serde_json::Value,
) -> (
    ace_foreign_contract::manual::AdmittedForeign,
    String,
    String,
) {
    let spec: ForeignSpec = serde_json::from_value(v["foreign"].clone())
        .unwrap_or_else(|e| fail(&format!("a2x.foreign-spec: invalid foreign binding: {e}")));
    if spec.kind != "rust" {
        fail(&format!(
            "a2x.foreign-kind: provider kind `{}` is outside the A2R support domain (rust only on this lane)",
            spec.kind
        ));
    }
    let root = std::path::Path::new(&spec.artifact_root);
    let descriptor_path = root.join(format!("{}.descriptor.json", spec.provider));
    let bytes = std::fs::read(&descriptor_path).unwrap_or_else(|e| {
        fail(&format!(
            "a2x.foreign-prepared: prepared provider closure missing at {}: {e} (run the prepare lane)",
            descriptor_path.display()
        ))
    });
    let descriptor: ProviderDescriptor = serde_json::from_slice(&bytes)
        .unwrap_or_else(|e| fail(&format!("a2x.foreign-prepared: bad provider descriptor: {e}")));
    if descriptor.schema != PROVIDER_DESCRIPTOR_SCHEMA {
        fail(&format!(
            "a2x.foreign-prepared: expected {PROVIDER_DESCRIPTOR_SCHEMA}, got {}",
            descriptor.schema
        ));
    }
    if descriptor.kind != spec.kind || descriptor.provider != spec.provider {
        fail(&format!(
            "a2x.foreign-prepared: descriptor identity mismatch: declared {} {} but closure holds {} {}",
            spec.kind, spec.provider, descriptor.kind, descriptor.provider
        ));
    }
    let source_b64 = descriptor.source_b64.as_deref().unwrap_or_else(|| {
        fail("a2x.foreign-prepared: rust provider closure lacks its source")
    });
    use base64::Engine as _;
    let provider_source = String::from_utf8(
        base64::engine::general_purpose::STANDARD
            .decode(source_b64)
            .unwrap_or_else(|e| fail(&format!("a2x.foreign-source: {e}"))),
    )
    .unwrap_or_else(|e| fail(&format!("a2x.foreign-source: {e}")));
    let manifest = auto_bindgen::rust_import::import(&provider_source)
        .unwrap_or_else(|e| fail(&format!("a2x.foreign-import: {}: {}", e.code, e.message)));
    let shim_source = auto_bindgen::rust_import::generate_shim(&manifest);
    let provider_digest = auto_bindgen::rust_import::provider_digest(&provider_source);
    let shim_digest = auto_bindgen::rust_import::shim_digest(&shim_source);
    let manual = auto_bindgen::rust_import::to_manual_manifest(
        &manifest,
        "f0-rust-provider/1",
        &provider_digest,
        &shim_digest,
    )
    .unwrap_or_else(|e| fail(&format!("a2x.foreign-manifest: {e}")));
    let rustc_identity = descriptor
        .rustc_identity
        .as_deref()
        .unwrap_or_else(|| fail("a2x.foreign-prepared: rust provider closure lacks a rustc identity"));
    let dll_name = descriptor
        .dll
        .clone()
        .unwrap_or_else(|| "f0_text_provider.dll".to_string());
    let lib_name = descriptor
        .lib
        .clone()
        .unwrap_or_else(|| "f0_text_provider.lib".to_string());
    let stamp_name = descriptor
        .stamp
        .clone()
        .unwrap_or_else(|| "f0-build-stamp.txt".to_string());
    let out = root.join("out");
    let stamp = format!("{provider_digest}\n{shim_digest}\n{rustc_identity}\n");
    let stamp_path = out.join(&stamp_name);
    let prepared = stamp_path.is_file()
        && std::fs::read_to_string(&stamp_path)
            .map(|s| s == stamp)
            .unwrap_or(false)
        && out.join(&dll_name).is_file()
        && out.join(&lib_name).is_file();
    if !prepared {
        fail(&format!(
            "a2x.foreign-prepared: prepared Rust provider missing or stale at {} (run the prepare lane)",
            out.display()
        ));
    }
    let dll = out.join(&dll_name);
    let actual = sha256_hex(
        &std::fs::read(&dll)
            .unwrap_or_else(|e| fail(&format!("a2x.foreign-artifact: read {}: {e}", dll.display()))),
    );
    if actual != descriptor.artifact_sha256 {
        fail(&format!(
            "a2x.foreign-artifact: prepared provider dll sha {actual} != declared {}",
            descriptor.artifact_sha256
        ));
    }
    let authority = ace_foreign_contract::manual::HostAuthority {
        review_id: manual.review_id.clone(),
        contract_digest: manual
            .contract_digest()
            .unwrap_or_else(|e| fail(&format!("a2x.foreign-manifest: {e}"))),
        provider: manual.provider.clone(),
        provider_digest,
        header_digest: manual.header_digest.clone(),
        physical_table_digest: manual.physical_table_digest.clone(),
        target: manual.target.clone(),
        capabilities: manual.facts.capabilities.iter().cloned().collect(),
        available_symbols: manifest
            .functions
            .iter()
            .map(|f| f.export.clone())
            .collect(),
        binding_generation: 1,
    };
    let admitted = manual
        .admit(&authority)
        .unwrap_or_else(|e| fail(&format!("a2x.foreign-admit: {e}")));
    (admitted, out.to_string_lossy().to_string(), dll_name)
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
    // PLAN-1116 (T-03): a prepared foreign provider closure rides the
    // request on the language lane only.
    let foreign = if v.get("foreign").is_some() {
        if profile != "language" {
            fail("a2x.domain: a `foreign` binding is only accepted on the `language` profile");
        }
        Some(load_prepared_foreign(v))
    } else {
        None
    };
    // The profile selects BOTH the frontend entry and the recipe channel
    // (PLAN-902: record/text/text-record ride the record-text fileset).
    let compiled = match profile.as_str() {
        "core" => compile_source(&src, "a2x-worker", &caps),
        "scalar" => compile_scalar_product_934(&src, &caps),
        "record" => compile_record_source(&src, "a2x-worker", &caps),
        "text" => compile_text_source(&src, "a2x-worker", &caps),
        "text-record" => compile_text_record_source(&src, "a2x-worker", &caps),
        "collections" => compile_collections_source(&src, "a2x-worker", &caps),
        // PLAN-1115 T-04: the ordinary language face — its own front door
        // (the caps carry the admission capability, e.g. the corpus rows'
        // `ac.source.language.av/v1`). Generic templates concretize through
        // the shared instance pass and `#{...}` compile-time demands are
        // served by the same CT service the scalar lane uses — the same
        // prepare hooks the AV language lane runs. PLAN-1116 T-03: a
        // prepared foreign provider rides the request through the shared
        // admitted-foreign front door (the same checked closure the AV
        // lane binds; the run phase never compiles).
        "language" => {
            match &foreign {
                Some((admitted, _, _)) => ace_core::source::adapter::compile_language_source_with_foreign(
                    &src,
                    "a2x-worker",
                    &caps,
                    admitted,
                ),
                None => ace_core::source::adapter::compile_language_source_prepared(
                    &src,
                    "a2x-worker",
                    &caps,
            |program, projected_source, has_instances| {
                if has_instances {
                    ace_source_query::instantiate_auto_program(program, "a2x-worker", &caps)?;
                }
                if src.contains('#') {
                    let mut service =
                        ace_source_cte::CtService::new(ace_source_cte::MemoryEnvelopeSink::new());
                    let _counters = service.counters();
                    ace_source_cte::product::prepare_auto_ct(
                        &mut service,
                        program,
                        projected_source,
                        "a2x-worker",
                        &caps,
                    )
                } else {
                    Ok(())
                }
            },
            ),
            }
        }
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
    let (program, entry_fn) = if is_language_profile(&profile) {
        match lower_language_program(&view, &entry) {
            Ok(p) => {
                let e = p.entry.clone();
                (p, e)
            }
            Err(r) => fail(&format!("lowering: {}", r)),
        }
    } else if is_coll_profile(&profile) {
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
    let (files, manifest) = if is_language_profile(&profile) {
        // PLAN-1115 T-02: the LANGUAGE channel manifest — same full-source
        // sha256 + execution-config binding discipline as the collections
        // channel, under its own fileset schema.
        let cfg = execution_config_of(v);
        let fs = assemble_language_file_set(
            &view,
            &program,
            &compiled.input_hash,
            &source_sha256,
            &cfg,
            &entry,
        );
        (fs.files, fs.manifest)
    } else if is_coll_profile(&profile) {
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
        foreign_lib: foreign.map(|(_, out, dll)| (out, dll)),
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

/// Whether the profile rides the PLAN-1115 ordinary-language channel.
fn is_language_profile(profile: &str) -> bool {
    profile == "language"
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
    if is_language_profile(profile) {
        LANG_RECIPE
    } else if is_coll_profile(profile) {
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
        foreign_lib: None,
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
    // PLAN-1034: reference 源一次 cap+1 有界快照（r4 R-1034-03：certify-c
    // 的 reference/golden 与 source 同门——IO/超限/坏编码具名 Infra）。
    let (ref_src, reference_sha256) =
        read_bounded_utf8(reference_path, SOURCE_FILE_MAX_BYTES, "reference");
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
    // PLAN-1034 r4 R-1034-03：golden_path 未声明保持证书该项 None；声明
    // 则实际 UTF-8 文件 ≤64MiB 并一次有界读取/hash——缺文件、IO、坏
    // 编码、超量明确 Infra，不再 `.ok()` 把读取失败降级成"未声明"。
    // 该文件仅显式认证的 hash 键材，不参与生产 emit；daily 父侧 golden
    // 仍绝不传 worker。
    let golden: Option<String> = v["golden_path"]
        .as_str()
        .map(|g| read_bounded_utf8(g, SOURCE_FILE_MAX_BYTES, "golden").0);
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

/// PLAN-1115 T-05: the language RUN lane — the ordinary source compiles
/// through the REAL toolchain and executes under the contained executor;
/// the parent-side case adapter judges the observed exit/stdout/stderr
/// against the corpus row's expectations. Frontend rejections surface as
/// the structured `a2x.source-diagnostic/1` envelope (stage/code carried).
fn run_language(v: &serde_json::Value) -> serde_json::Value {
    let out_dir = v["out_dir"]
        .as_str()
        .unwrap_or_else(|| fail("out_dir required"));
    let nonce = v["nonce"].as_str().unwrap_or("");
    match v["profile"].as_str().unwrap_or("language") {
        "language" => {}
        other => fail(&format!(
            "a2x.domain: the language run lane serves `language`, not `{other}`"
        )),
    }
    let out = emit(v);
    write_out_dir(out_dir, &out);
    let dir = PathBuf::from(out_dir);
    let t0 = std::time::Instant::now();
    let fl = out.foreign_lib.clone();
    let (stdout, stderr, processes) = compile_and_run(&dir, "main.rs", "a2x_lang_run.exe", fl.as_ref());
    let wall_ms = t0.elapsed().as_millis() as u64;
    let mut r = ok_base(LANG_RUN_RECIPE);
    r["mode"] = "run-language".into();
    r["nonce"] = nonce.into();
    r["profile"] = out.profile.clone().into();
    r["input_hash"] = out.input_hash.clone().into();
    r["source_sha256"] = out.source_sha256.clone().into();
    r["entry_fn"] = out.entry_fn.clone().into();
    r["manifest_sha256"] = sha256_hex(out.manifest.as_bytes()).into();
    r["stdout"] = stdout.into();
    r["stderr"] = stderr.into();
    r["compile_run_ms"] = wall_ms.into();
    if std::env::var("AUTO_CASE_WIRE_EVIDENCE").as_deref() == Ok("1") {
        r["execution_observations"] = serde_json::json!({
            "schema": "ace.a2x.cert-execution/1",
            "run": processes,
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
        compile_and_run(&dir, "main.rs", "a2x_actual.exe", None);
    let actual_wall_ms = t0.elapsed().as_millis() as u64;
    // independent hand-written reference: compile + run separately.
    // PLAN-1034: reference 源同一次 cap+1 有界快照（读取/hash 同 bytes，
    // IO/超限/坏编码具名拒绝）。
    let (reference_src, reference_sha256) =
        read_bounded_utf8(&reference_path, SOURCE_FILE_MAX_BYTES, "reference");
    std::fs::write(dir.join("reference.rs"), &reference_src)
        .unwrap_or_else(|e| fail(&format!("write reference: {e}")));
    let t1 = std::time::Instant::now();
    let (reference_out, reference_err, reference_processes) =
        compile_and_run(&dir, "reference.rs", "a2x_reference.exe", None);
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
    let (cert_schema, cert_recipe) = if is_language_profile(&profile) {
        (ace_a2x_rust::LANG_CERT_SCHEMA, LANG_RECIPE)
    } else if is_coll_profile(&profile) {
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
        reference_sha256,
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
    let user_path = v["logical_path"].as_str().unwrap_or("a2x-worker");
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
        "python-script" => {
            if v["op"] != "run-py"
                || v["recipe"] != PY_STDLIB_RUN_RECIPE
                || !user_path.ends_with(".as")
            {
                fail("a2py.admission: python-script requires a run recipe and a logical .as source");
            }
            let foreign = ace_a2x_python::py_contract::py_declaration_contract()
                .unwrap_or_else(|e| fail(&format!("a2py.admission: {e}")));
            ace_core::source::compile_language_source_with_foreign(&src, user_path, &caps, &foreign)
        }
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
        "collections" | "python-script" => Family::Collections,
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
    let fns: Vec<&ace_a2x_core::FnFacts<'_>> = view
        .functions()
        .iter()
        .filter(|f| view.py_ops().is_none() || !ace_a2x_python::provider::is_py_lane_helper(f.name))
        .collect();
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
        foreign_lib: None,
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
    let trace_dir = (out.profile == "python-script").then(|| {
        tempfile::Builder::new()
            .prefix(".py-observation-")
            .tempdir_in(out_dir)
            .unwrap_or_else(|e| fail(&format!("python.observation: {e}")))
    });
    let trace_path = trace_dir.as_ref().map(|dir| dir.path().join("error.json"));
    let envs = trace_path.as_ref().map(|path| {
        [(
            "ACE_A2PY_ERROR_OBSERVATION",
            path.to_str()
                .unwrap_or_else(|| fail("python.observation: non-UTF-8 path")),
        )]
    });
    let exec = match ace_a2x_worker::pyrun::run_py_fileset(
        &py,
        Path::new(out_dir),
        envs.as_ref().map_or(&[], |env| &env[..]),
    ) {
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
    if let Some(path) = trace_path {
        r["logical_path"] = v["logical_path"].clone();
        r["source_syntax"] = "auto-script".into();
        match std::fs::File::open(path) {
            Ok(file) => {
                use std::io::Read;
                let mut bytes = Vec::new();
                file.take(4097)
                    .read_to_end(&mut bytes)
                    .unwrap_or_else(|e| fail(&format!("python.observation: {e}")));
                if bytes.len() > 4096 {
                    fail("python.observation: observation exceeds its size limit");
                }
                r["language_error"] = serde_json::from_slice(&bytes)
                    .unwrap_or_else(|e| fail(&format!("python.observation: {e}")));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => fail(&format!("python.observation: {e}")),
        }
    }
    r["exit_code"] = exec
        .exit_code
        .map(serde_json::Value::from)
        .unwrap_or(serde_json::Value::Null);
    r
}

fn check_ts_admission(v: &serde_json::Value) {
    if v["use_legacy_ts_trans"].as_bool() == Some(true) {
        fail("admission.legacy_ts_trans: legacy TypeScriptTrans/JavaScriptTrans AST emitters are forbidden on ace-a2x-ts");
    }
    if v["use_vm"].as_bool() == Some(true) {
        fail("admission.avm_fallback: AutoVM fallback is forbidden on ace-a2x-ts");
    }
    if v.get("expected").is_some() {
        fail("admission.self_oracle: expected value must be verified parent-side, never supplied to worker");
    }
}

/// PLAN-976: emit a TypeScript/JavaScript fileset through the shared
/// `ace-a2x-ts` provider (the SAME `TargetProvider` path the session uses;
/// `target_lang` selects the leg).
fn emit_ts(v: &serde_json::Value) -> (EmitOutcome, ace_a2x_core::FileSet) {
    use ace_a2x_core::target::{Family, FilesetRequest, FragmentText, TargetProvider};
    check_ts_admission(v);
    let (src, source_sha256) = read_source(v);
    let profile = v["profile"].as_str().unwrap_or("core").to_string();
    let entry = v["entry"].as_str().unwrap_or("main").to_string();
    let lang = match v["target_lang"].as_str().unwrap_or("typescript") {
        "typescript" => ace_a2x_ts::ScriptLang::TypeScript,
        "javascript" => ace_a2x_ts::ScriptLang::JavaScript,
        other => fail(&format!(
            "a2ts.target_lang: must be `typescript` or `javascript`, not `{other}`"
        )),
    };
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
            "a2ts.domain: the script channel serves core/scalar/record/text/text-record/collections, not `{}`",
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
    let ts_provider = ace_a2x_ts::TypeScriptTargetProvider;
    let js_provider = ace_a2x_ts::JavaScriptTargetProvider;
    let provider: &dyn TargetProvider = match lang {
        ace_a2x_ts::ScriptLang::TypeScript => &ts_provider,
        ace_a2x_ts::ScriptLang::JavaScript => &js_provider,
    };
    let family = match profile.as_str() {
        "record" | "text" | "text-record" => Family::RecordText,
        "collections" => Family::Collections,
        _ => Family::Scalar,
    };
    let expected_schema = match family {
        Family::Scalar => ace_a2x_ts::TS_SCALAR_FILESET_SCHEMA,
        Family::RecordText => ace_a2x_ts::TS_RT_FILESET_SCHEMA,
        Family::Collections => ace_a2x_ts::TS_COLL_FILESET_SCHEMA,
    };
    let default_recipe = match (lang, family) {
        (ace_a2x_ts::ScriptLang::TypeScript, Family::Scalar) => TS_RECIPE,
        (ace_a2x_ts::ScriptLang::TypeScript, Family::RecordText) => TS_RT_RECIPE,
        (ace_a2x_ts::ScriptLang::TypeScript, Family::Collections) => TS_COLL_RECIPE,
        (_, Family::Scalar) => JS_RECIPE,
        (_, Family::RecordText) => JS_RT_RECIPE,
        (_, Family::Collections) => JS_COLL_RECIPE,
    };
    let recipes: &[&str] = match lang {
        ace_a2x_ts::ScriptLang::TypeScript => &[TS_RECIPE, TS_RT_RECIPE, TS_COLL_RECIPE],
        ace_a2x_ts::ScriptLang::JavaScript => &[JS_RECIPE, JS_RT_RECIPE, JS_COLL_RECIPE],
    };
    let recipe = match v["recipe"].as_str() {
        Some(r) if recipes.contains(&r) => r,
        Some(other) => fail(&format!("unregistered script recipe `{other}`")),
        None => default_recipe,
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
    if let Err(reason) = ace_a2x_ts::verify_script_fileset_manifest(
        &fileset.manifest,
        &fileset.files,
        ace_a2x_ts::TS_PRODUCER,
        expected_schema,
    ) {
        fail(&format!("a2ts.manifest: {reason}"));
    }
    let outcome = EmitOutcome {
        files: fileset.files.clone(),
        manifest: fileset.manifest.clone(),
        input_hash: compiled.input_hash.clone(),
        entry_fn: ace_a2x_ts::ts_fn_name(&entry),
        profile,
        source_sha256,
        recipe: recipe.to_string(),
        hir_domain: compiled.checked.bundle().domain.as_str().to_string(),
        foreign_lib: None,
    };
    (outcome, fileset)
}

/// PLAN-976: spawn-zero script fileset emit (`execution_claimed == false`).
fn daily_emit_ts(v: &serde_json::Value) -> serde_json::Value {
    let out_dir = v["out_dir"]
        .as_str()
        .unwrap_or_else(|| fail("out_dir required"));
    let nonce = v["nonce"].as_str().unwrap_or("");
    let target_lang = v["target_lang"]
        .as_str()
        .unwrap_or("typescript")
        .to_string();
    let (out, _fs) = emit_ts(v);
    write_out_dir(out_dir, &out);
    let mut r = ok_base(&out.recipe);
    r["mode"] = "daily-emit-ts".into();
    r["target_lang"] = target_lang.into();
    r["producer"] = ace_a2x_ts::TS_PRODUCER.into();
    r["runtime_version"] = ace_a2x_ts::A2TS_RUNTIME_VERSION.into();
    r["runtime_sha256"] = ace_a2x_ts::runtime_body_sha256().into();
    r["nonce"] = nonce.into();
    r["profile"] = out.profile.clone().into();
    r["input_hash"] = out.input_hash.clone().into();
    r["source_sha256"] = out.source_sha256.clone().into();
    r["entry_fn"] = out.entry_fn.clone().into();
    r["manifest_sha256"] = sha256_hex(out.manifest.as_bytes()).into();
    r["files"] = file_records(&out.files).into();
    r["spawned_node"] = 0.into();
    r["spawned_rustc"] = 0.into();
    r["spawned_cl"] = 0.into();
    r["spawned_python"] = 0.into();
    r["execution_claimed"] = false.into();
    r
}

/// PLAN-976: real OS node execution of files emitted by the shared
/// TS/JS provider (node type-strips `.ts` natively; `.mjs` is plain ESM).
fn run_ts(v: &serde_json::Value) -> serde_json::Value {
    if v["emit_only"].as_bool() == Some(true) {
        fail("admission.emit_only: emit_only cannot claim script execution");
    }
    let out_dir = v["out_dir"]
        .as_str()
        .unwrap_or_else(|| fail("out_dir required"));
    let nonce = v["nonce"].as_str().unwrap_or("");
    let target_lang = match v["target_lang"].as_str().unwrap_or("typescript") {
        "typescript" => ace_a2x_ts::ScriptLang::TypeScript,
        "javascript" => ace_a2x_ts::ScriptLang::JavaScript,
        other => fail(&format!(
            "a2ts.target_lang: must be `typescript` or `javascript`, not `{other}`"
        )),
    };
    let rt_sha = ace_a2x_ts::runtime_body_sha256();
    let node = match ace_a2x_worker::tsrun::resolve_node(&rt_sha) {
        Ok(n) => n,
        Err(e) => fail(&format!("node.missing: {e}")),
    };
    let (out, _fileset) = emit_ts(v);
    write_out_dir(out_dir, &out);
    let exec = match ace_a2x_worker::tsrun::run_script_fileset(
        &node,
        std::path::Path::new(out_dir),
        target_lang,
        &[],
    ) {
        Ok(e) => e,
        Err(e) => fail(&format!("node.exec: {e}")),
    };
    let mut r = ok_base(&out.recipe);
    r["mode"] = "run-ts".into();
    r["target_lang"] = target_lang.lang_name().into();
    r["producer"] = ace_a2x_ts::TS_PRODUCER.into();
    r["runtime_version"] = ace_a2x_ts::A2TS_RUNTIME_VERSION.into();
    r["runtime_sha256"] = rt_sha.into();
    r["nonce"] = nonce.into();
    r["profile"] = out.profile.clone().into();
    r["input_hash"] = out.input_hash.clone().into();
    r["source_sha256"] = out.source_sha256.clone().into();
    r["entry_fn"] = out.entry_fn.clone().into();
    r["manifest_sha256"] = sha256_hex(out.manifest.as_bytes()).into();
    r["files"] = file_records(&out.files).into();
    r["spawned_node"] = 1.into();
    r["spawned_rustc"] = 0.into();
    r["spawned_cl"] = 0.into();
    r["spawned_python"] = 0.into();
    r["execution_claimed"] = true.into();
    r["node_identity"] = serde_json::json!({
        "engine": node.identity.engine,
        "version": node.identity.version,
        "version_sha256": node.identity.executable_sha256,
        "runtime_version": node.identity.runtime_version,
        "runtime_sha256": node.identity.runtime_sha256,
        "os": node.identity.target_triple,
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

fn compile_and_run(
    dir: &Path,
    src: &str,
    exe_name: &str,
    foreign_lib: Option<&(String, String)>,
) -> (String, String, serde_json::Value) {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let exe = dir.join(exe_name);
    let mut compile = Command::new(&rustc);
    compile
        .args(["--edition", "2021", "-o"])
        .arg(&exe)
        .arg(src)
        .current_dir(dir);
    // PLAN-1116 (T-03): the checked provider's import library directory
    // (the generated bridge's `#[link(name = ...)]` resolves here).
    if let Some((lib_dir, _)) = foreign_lib {
        compile.arg("-L").arg(lib_dir);
    }
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
    // The provider DLL loads as a process dependency — place it next to
    // the produced exe (a DLL search location on Windows).
    if let Some((lib_dir, dll)) = foreign_lib {
        std::fs::copy(std::path::Path::new(lib_dir).join(dll), dir.join(dll))
            .unwrap_or_else(|e| fail(&format!("a2x.foreign-artifact: stage {dll}: {e}")));
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

// ---- PLAN-1034 T-03（IPC-01）：一次有界快照 / 请求门 / golden 有界 ----
#[cfg(test)]
mod audit1034_ipc_01 {
    use super::*;

    /// §6 冻结输入：28 字节固定源 + 恰一个尾 LF（SHA256 与计划一致）。
    const FIXED: &[u8] = b"fn main() int { return 42 }
";
    const FIXED_SHA: &str = "a1458aa7834a132d416cdc044693771f9719ca5febaad7df69fcd8f08534fc5d";

    fn tmp_dir(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!(
            "plan1034_{tag}_{}_{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn err_of(r: Result<(String, String), String>) -> String {
        r.expect_err("expected a named refusal")
    }

    /// FIXED 的文本视图（json! 字面量直写 LF 会被编辑器/传输层改写成真
    /// 换行+缩进——统一从字节常量派生，杜绝该类脏字面量）。
    fn fixed_text() -> String {
        String::from_utf8(FIXED.to_vec()).unwrap()
    }

    /// text/hash/compile 同一快照：source_path 报告的 SHA 恰为该 bytes 的
    /// SHA（真实 oracle）；单读之后文件变化不再影响结果。
    #[test]
    fn source_path_hash_and_text_come_from_one_snapshot() {
        let d = tmp_dir("ipc01a");
        let p = d.join("src.at");
        std::fs::write(&p, FIXED).unwrap();
        let (text, sha) = read_source(&serde_json::json!({ "source_path": p }));
        assert_eq!(text, fixed_text());
        assert_eq!(sha, FIXED_SHA, "hash must be of the exact read bytes");
        // 写坏文件不影响已返回快照（单读语义），再读则按新内容计。
        std::fs::write(&p, b"changed").unwrap();
        let (text2, sha2) = read_source(&serde_json::json!({ "source_path": p }));
        assert_eq!(text2, "changed");
        assert_eq!(sha2, sha256_hex(b"changed"));
    }

    /// source_text 邻例：同 bytes 的内联路径 hash 一致（64MiB 内容门由
    /// 整请求 8MiB 信封先行约束，无额外字段门）。
    #[test]
    fn source_text_neighbor_hashes_the_same_bytes() {
        let (text, sha) = read_source(&serde_json::json!({
            "source_text": fixed_text()
        }));
        assert_eq!(text, fixed_text());
        assert_eq!(sha, FIXED_SHA);
    }

    /// cap 等界接受、超 1 字节拒绝（小 cap 机制等界；生产 64MiB 同门，
    /// 不在单测里分配巨型文件）。
    #[test]
    fn source_file_cap_boundary_accepts_cap_rejects_cap_plus_one() {
        let d = tmp_dir("ipc01b");
        let p = d.join("cap.at");
        std::fs::write(&p, vec![b'a'; 1024]).unwrap();
        let (t, _) = read_bounded_utf8_result(p.to_str().unwrap(), 1024, "source").unwrap();
        assert_eq!(t.len(), 1024, "exactly-cap is admitted");
        std::fs::write(&p, vec![b'a'; 1025]).unwrap();
        let msg = err_of(read_bounded_utf8_result(
            p.to_str().unwrap(),
            1024,
            "source",
        ));
        assert!(
            msg.contains("a2x.source-limit"),
            "named resource refusal, got: {msg}"
        );
    }

    /// 坏 UTF-8 具名拒绝（严格解码，不 lossy）。
    #[test]
    fn source_file_invalid_utf8_is_named_refusal() {
        let d = tmp_dir("ipc01c");
        let p = d.join("bad.at");
        std::fs::write(
            &p,
            vec![0xffu8, 0xfe, b' ', b'i', b'l', b'l', b'e', b'g', b'a', b'l'],
        )
        .unwrap();
        let msg = err_of(read_bounded_utf8_result(p.to_str().unwrap(), 64, "source"));
        assert!(
            msg.contains("a2x.source-utf8"),
            "named utf8 refusal, got: {msg}"
        );
    }

    /// 文件缺失是具名 IO，不是 hash(empty) 成功（IPC-01 关闭断言）。
    #[test]
    fn source_file_missing_is_named_infra_not_empty_hash_success() {
        let d = tmp_dir("ipc01d");
        let missing = d.join("absent.at");
        let msg = err_of(read_bounded_utf8_result(
            missing.to_str().unwrap(),
            SOURCE_FILE_MAX_BYTES,
            "source",
        ));
        assert!(
            msg.contains("a2x.source-io"),
            "named io refusal, got: {msg}"
        );
    }

    /// stdin one-shot 请求门：多行 JSON + 尾 LF 全量读取；cap 拒绝在
    /// serde 之前；首行可解析不提前接受（整文档解析）。
    #[test]
    fn request_stream_gate_reads_whole_document_before_parse() {
        // 小 cap 机制等界（生产 8MiB 同门）。&mut &[u8] 读取会推进切片
        // 视图——每次断言前重新绑定新鲜切片。
        const DOC: &[u8] = b"{
  \"op\": \"identity\"
}
";
        let mut doc = DOC;
        let at_cap = DOC.len();
        let expect = String::from_utf8_lossy(DOC).into_owned();
        assert_eq!(
            read_request_bounded_result(&mut doc, at_cap).unwrap(),
            expect
        );
        let over_cap = DOC.len() - 1;
        let mut doc = DOC;
        let msg =
            read_request_bounded_result(&mut doc, over_cap).expect_err("cap+1 must be rejected");
        assert!(
            msg.contains("a2x.request-limit"),
            "named request-limit, got: {msg}"
        );
        // 坏 UTF-8 信封具名拒绝。
        let mut bad: &[u8] = &[b'{', b'"', b'o', b'p', b'"', b':', 0xffu8, b'}'][..];
        let msg = read_request_bounded_result(&mut bad, 64)
            .expect_err("invalid utf-8 envelope must be rejected");
        assert!(
            msg.contains("a2x.request-utf8"),
            "named request-utf8, got: {msg}"
        );
        // 两行 JSON（首行独立可解析）不是合法 one-shot 文档：整文档解析
        // 拒绝，不接受首行前缀。
        let mut two: &[u8] = b"{\"op\":\"identity\"}
{\"op\":\"identity\"}
";
        let s = read_request_bounded_result(&mut two, 1 << 20).unwrap();
        assert!(
            serde_json::from_str::<serde_json::Value>(&s).is_err(),
            "two concatenated documents must not parse as one"
        );
    }

    /// certify-c golden：未声明 None；声明则同门读取（IO/坏编码具名，
    /// 不 `.ok()` 降级成"未声明"）。
    #[test]
    fn certify_c_golden_declared_reads_bounded_missing_stays_none() {
        let d = tmp_dir("ipc01e");
        let g = d.join("golden.txt");
        std::fs::write(
            &g,
            b"golden bytes
",
        )
        .unwrap();
        let got = golden_text_of(&serde_json::json!({ "golden_path": g })).unwrap();
        assert_eq!(
            got.as_deref(),
            Some(
                "golden bytes
"
            )
        );
        assert_eq!(
            golden_text_of(&serde_json::json!({})).unwrap(),
            None,
            "undeclared golden stays certificate-None"
        );
        let missing = d.join("nope.txt");
        let msg = golden_text_of(&serde_json::json!({ "golden_path": missing }))
            .expect_err("declared-but-missing golden is named infra");
        assert!(
            msg.contains("a2x.golden-io"),
            "declared-but-missing golden is named infra, got: {msg}"
        );
        std::fs::write(&g, vec![0xffu8, b' ', b'b', b'a', b'd']).unwrap();
        let msg = golden_text_of(&serde_json::json!({ "golden_path": g }))
            .expect_err("bad-encoding golden is named infra");
        assert!(
            msg.contains("a2x.golden-utf8"),
            "bad-encoding golden is named infra, got: {msg}"
        );
    }

    /// r4 断言：1MiB+1 ASCII 内联源可进入（无额外字段门，交集由请求
    /// 8MiB 信封先行约束）；等价的 1MiB 边界在 source 上无独立门。
    #[test]
    fn inline_source_over_1mib_is_admitted_no_field_gate() {
        let mut text = String::from("fn main() int { return 42 } //");
        while text.len() < (1 << 20) + 1 {
            text.push('a');
        }
        assert_eq!(text.len(), (1 << 20) + 1);
        let (t, sha) = read_source(&serde_json::json!({ "source_text": text }));
        assert_eq!(t.len(), (1 << 20) + 1, "no 1MiB field gate on inline text");
        assert_eq!(sha.len(), 64);
    }

    /// golden 读取 Result 核心（certify_c 同语义，供测试直调）；
    /// 未声明 → Ok(None)；声明 → 有界读取，IO/超限/坏编码具名 Err。
    fn golden_text_of(v: &serde_json::Value) -> Result<Option<String>, String> {
        v["golden_path"]
            .as_str()
            .map(|g| read_bounded_utf8_result(g, SOURCE_FILE_MAX_BYTES, "golden").map(|(t, _)| t))
            .transpose()
    }
}
