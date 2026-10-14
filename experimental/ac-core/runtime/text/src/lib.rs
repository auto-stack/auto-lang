//! PLAN-770 text runtime verification harness — host side.
//!
//! Builds Cranelift callers (WindowsFastcall, same stack as ac-core), links
//! them with the no_std text runtime into a real PE via the REAL ac-core
//! link/executor entries, runs the PE under the contained 60s executor and
//! returns its exit code + stdout/stderr for oracle assertions. This
//! exercises the runtime's OWN contract (counters/ledger/budget/traces/eq)
//! through real machine calls; the language-level text lowering faces
//! (V-02/V-03/V-07 language legs) live in the ac-core test suite (T-05).

#![cfg(windows)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use cranelift_codegen::ir::types;
use cranelift_codegen::ir::{AbiParam, FuncRef, InstBuilder, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::settings::{builder as flags_builder, Flags};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use target_lexicon::Triple;

/// Frozen status codes (probe-cases.md §0; mirrored in the shim).
pub mod status {
    pub const ERR_LEN_OVERFLOW: i32 = -1;
    pub const ERR_LEN_NEGATIVE: i32 = -2;
    pub const ERR_LEN_TOO_LARGE: i32 = -3;
    pub const ERR_BUDGET: i32 = -4;
    pub const ERR_ENV_OOM: i32 = -5;
    pub const ERR_SHIM: i32 = -6;
    pub const ERR_INVALID_UTF8: i32 = -7;
    pub const ERR_INVALID_SCALAR: i32 = -8;
    pub const ERR_RANGE_REVERSED: i32 = -9;
    pub const ERR_RANGE_OOB: i32 = -10;
    pub const ERR_INDEX_OOB: i32 = -11;
}

const WINCALL: CallConv = CallConv::WindowsFastcall;
const ENTRY_SYMBOL: &str = "ac_text_entry";
const DEADLINE: Duration = Duration::from_secs(60);

const I: types::Type = types::I32;
const P: types::Type = types::I64;

type SigSpec = (&'static str, &'static [types::Type], &'static [types::Type]);

/// The frozen shim import surface with exact machine signatures
/// (probe-cases.md §0; interface-contract.md carries the doc table).
const SHIM_SIGS: &[SigSpec] = &[
    ("ac_rt_own_make", &[I, I, P], &[P]),
    ("ac_rt_own_free", &[P], &[]),
    ("ac_rt_own_clone", &[P, I, P], &[P]),
    ("ac_rt_own_concat", &[P, I, P, I, P], &[P]),
    ("ac_rt_own_from_lit", &[P, I, P], &[P]),
    ("ac_rt_text_scalars", &[P, I], &[I]),
    ("ac_rt_text_bytes", &[P, I], &[I]),
    ("ac_rt_char_at", &[P, I, I], &[I]),
    ("ac_rt_text_slice", &[P, I, I, I, P], &[P]),
    ("ac_rt_text_eq", &[P, I, P, I], &[I]),
    ("ac_rt_text_ne", &[P, I, P, I], &[I]),
    ("ac_rt_char_from_codepoint", &[I], &[I]),
    ("ac_rt_char_to_codepoint", &[I], &[I]),
    ("ac_rt_len_from_i32", &[I], &[I]),
    ("ac_rt_len_to_i32", &[P], &[I]),
    ("ac_rt_set_budget", &[P], &[]),
    ("ac_rt_fail_nth_alloc", &[I], &[]),
    ("ac_rt_fail_ledger_nth", &[I], &[]),
    ("ac_rt_shim_error", &[], &[I]),
    ("ac_rt_summary", &[], &[]),
    ("ac_rt_check_invariant", &[], &[I]),
    ("ac_rt_trap_after_alloc", &[I], &[]),
    ("ac_rt_hang_after_alloc", &[I], &[]),
];

/// Everything a caller body can reference: imported shim FuncRefs and data
/// GlobalValues (declared into the function being built).
pub struct Fx {
    pub funcs: BTreeMap<&'static str, FuncRef>,
    pub data: BTreeMap<&'static str, cranelift_codegen::ir::GlobalValue>,
}

pub type Value = cranelift_codegen::ir::Value;
pub type MainBody = Box<dyn FnOnce(&mut FunctionBuilder, &Fx) -> Value>;
pub type AuxBody = Box<dyn FnOnce(&mut FunctionBuilder, &Fx, &[Value]) -> Value>;

struct AuxDef {
    params: Vec<types::Type>,
    body: AuxBody,
}

/// One probe program: imported shim surface + test-owned data + functions +
/// the main function whose i32 result becomes the PE exit code (0 = pass;
/// accumulate `expect_*` violation flags in it).
pub struct ProbeCx {
    om: ObjectModule,
    shim_ids: BTreeMap<&'static str, FuncId>,
    exit_id: FuncId,
    data_defs: Vec<(&'static str, cranelift_module::DataId)>,
    aux: Vec<(&'static str, AuxDef)>,
    aux_ids: BTreeMap<&'static str, FuncId>,
    main: Option<MainBody>,
}

impl ProbeCx {
    fn new() -> Result<ProbeCx, String> {
        let triple = Triple::host();
        let mut fb = flags_builder();
        fb.set("opt_level", "none").map_err(|e| e.to_string())?;
        fb.set("is_pic", "false").map_err(|e| e.to_string())?;
        let flags = Flags::new(fb);
        let isa = cranelift_codegen::isa::lookup(triple)
            .map_err(|e| format!("isa lookup: {e}"))?
            .finish(flags)
            .map_err(|e| format!("isa finish: {e}"))?;
        let obj_builder =
            ObjectBuilder::new(isa, "ac770_text", cranelift_module::default_libcall_names())
                .map_err(|e| format!("object builder: {e}"))?;
        let mut om = ObjectModule::new(obj_builder);

        let mut shim_ids = BTreeMap::new();
        for &(name, params, rets) in SHIM_SIGS {
            let sig = Signature {
                call_conv: WINCALL,
                params: params.iter().map(|t| AbiParam::new(*t)).collect(),
                returns: rets.iter().map(|t| AbiParam::new(*t)).collect(),
            };
            let id = om
                .declare_function(name, Linkage::Import, &sig)
                .map_err(|e| format!("declare {name}: {e}"))?;
            shim_ids.insert(name, id);
        }
        let exit_sig = Signature {
            call_conv: WINCALL,
            params: vec![AbiParam::new(I)],
            returns: vec![],
        };
        let exit_id = om
            .declare_function("ExitProcess", Linkage::Import, &exit_sig)
            .map_err(|e| format!("declare ExitProcess: {e}"))?;

        Ok(ProbeCx {
            om,
            shim_ids,
            exit_id,
            data_defs: Vec::new(),
            aux: Vec::new(),
            aux_ids: BTreeMap::new(),
            main: None,
        })
    }

    /// Declare a data blob (writable or read-only) addressable from caller
    /// code by name; byte content is frozen by the test.
    pub fn data(&mut self, name: &'static str, bytes: &[u8], writable: bool) {
        let id = self
            .om
            .declare_data(name, Linkage::Export, writable, false)
            .expect("declare data");
        let mut desc = DataDescription::new();
        desc.define(bytes.to_vec().into_boxed_slice());
        desc.set_align(4);
        self.om.define_data(id, &desc).expect("define data");
        self.data_defs.push((name, id));
    }

    /// Define an additional caller-side function returning i32 (for
    /// nested-call cases). Must be called before `set_main` so the main body
    /// can reference it by name.
    pub fn func<F>(&mut self, name: &'static str, params: &[types::Type], body: F)
    where
        F: FnOnce(&mut FunctionBuilder, &Fx, &[Value]) -> Value + 'static,
    {
        self.aux.push((
            name,
            AuxDef {
                params: params.to_vec(),
                body: Box::new(body),
            },
        ));
    }

    /// Set the main function body; its returned i32 becomes the exit code.
    pub fn set_main<F>(&mut self, body: F)
    where
        F: FnOnce(&mut FunctionBuilder, &Fx) -> Value + 'static,
    {
        self.main = Some(Box::new(body));
    }

    fn fx_for(&mut self, func: &mut cranelift_codegen::ir::Function) -> Fx {
        let mut funcs = BTreeMap::new();
        for (name, id) in &self.shim_ids {
            funcs.insert(*name, self.om.declare_func_in_func(*id, func));
        }
        for (name, id) in &self.aux_ids {
            funcs.insert(*name, self.om.declare_func_in_func(*id, func));
        }
        let mut data = BTreeMap::new();
        for (name, id) in &self.data_defs {
            data.insert(*name, self.om.declare_data_in_func(*id, func));
        }
        Fx { funcs, data }
    }

    fn define_aux(&mut self, name: &'static str, def: AuxDef) -> Result<(), String> {
        let fid = self
            .aux_ids
            .get(name)
            .copied()
            .ok_or_else(|| format!("aux {name} not pre-declared"))?;
        let sig = Signature {
            call_conv: WINCALL,
            params: def.params.iter().map(|t| AbiParam::new(*t)).collect(),
            returns: vec![AbiParam::new(I)],
        };
        let mut ctx = self.om.make_context();
        ctx.func.signature = sig;
        {
            let mut fb_ctx = FunctionBuilderContext::new();
            let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
            let fx = self.fx_for(&mut fb.func);
            let entry = fb.create_block();
            fb.append_block_params_for_function_params(entry);
            fb.switch_to_block(entry);
            fb.seal_block(entry);
            let params: Vec<Value> = fb.block_params(entry).to_vec();
            let result = (def.body)(&mut fb, &fx, &params);
            fb.ins().return_(&[result]);
            fb.finalize();
        }
        self.om
            .define_function(fid, &mut ctx)
            .map_err(|e| format!("define aux {name}: {e}"))?;
        Ok(())
    }

    fn define_main(&mut self, body: MainBody) -> Result<(), String> {
        let main_sig = Signature {
            call_conv: WINCALL,
            params: vec![],
            returns: vec![AbiParam::new(I)],
        };
        let main_id = self
            .om
            .declare_function("ac_text_main", Linkage::Export, &main_sig)
            .map_err(|e| format!("declare p774_main: {e}"))?;
        let mut ctx = self.om.make_context();
        ctx.func.signature = main_sig;
        {
            let mut fb_ctx = FunctionBuilderContext::new();
            let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
            let fx = self.fx_for(&mut fb.func);
            let entry = fb.create_block();
            fb.switch_to_block(entry);
            fb.seal_block(entry);
            let result = body(&mut fb, &fx);
            fb.ins().return_(&[result]);
            fb.finalize();
        }
        self.om
            .define_function(main_id, &mut ctx)
            .map_err(|e| format!("define p774_main: {e}"))?;

        // Generic entry wrapper: p774_main() -> ExitProcess(result). The
        // Auto function is never the OS entry itself (ac-core convention).
        let entry_sig = Signature {
            call_conv: WINCALL,
            params: vec![],
            returns: vec![],
        };
        let entry_id = self
            .om
            .declare_function(ENTRY_SYMBOL, Linkage::Export, &entry_sig)
            .map_err(|e| format!("declare {ENTRY_SYMBOL}: {e}"))?;
        let mut ctx = self.om.make_context();
        ctx.func.signature = entry_sig;
        {
            let mut fb_ctx = FunctionBuilderContext::new();
            let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
            let main_ref = self.om.declare_func_in_func(main_id, &mut fb.func);
            let exit_ref = self.om.declare_func_in_func(self.exit_id, &mut fb.func);
            let blk = fb.create_block();
            fb.switch_to_block(blk);
            fb.seal_block(blk);
            let call = fb.ins().call(main_ref, &[]);
            let result = fb.inst_results(call)[0];
            fb.ins().call(exit_ref, &[result]);
            fb.ins().return_(&[]);
            fb.finalize();
        }
        self.om
            .define_function(entry_id, &mut ctx)
            .map_err(|e| format!("define {ENTRY_SYMBOL}: {e}"))?;
        Ok(())
    }

    fn finish(mut self) -> Result<Vec<u8>, String> {
        let main = self
            .main
            .take()
            .ok_or_else(|| "probe program without set_main".to_string())?;
        // Pre-declare every aux so aux↔aux and main→aux references resolve
        // without duplication (recursion stays legal: ids precede bodies).
        let deferred: Vec<(&'static str, AuxDef)> = std::mem::take(&mut self.aux);
        for (name, def) in &deferred {
            let sig = Signature {
                call_conv: WINCALL,
                params: def.params.iter().map(|t| AbiParam::new(*t)).collect(),
                returns: vec![AbiParam::new(I)],
            };
            let fid = self
                .om
                .declare_function(name, Linkage::Export, &sig)
                .map_err(|e| format!("declare aux {name}: {e}"))?;
            self.aux_ids.insert(name, fid);
        }
        for (name, def) in deferred {
            self.define_aux(name, def)?;
        }
        self.define_main(main)?;
        self.om
            .finish()
            .emit()
            .map_err(|e| format!("emit object: {e}"))
    }
}

// ---------------------------------------------------------------------------
// machine-side assertion helpers (violation flags accumulated into exit code)
// ---------------------------------------------------------------------------

/// Violation flag: 0 when `got == want` (i32), else 1.
pub fn expect_eq_i32(fb: &mut FunctionBuilder, got: Value, want: i32) -> Value {
    let w = fb.ins().iconst(I, want as i64);
    let ne = fb
        .ins()
        .icmp(cranelift_codegen::ir::condcodes::IntCC::NotEqual, got, w);
    fb.ins().uextend(I, ne)
}

/// Violation flag: 0 when `ptr != null`, else 1.
pub fn expect_non_null(fb: &mut FunctionBuilder, ptr: Value) -> Value {
    let null = fb.ins().iconst(P, 0);
    let eq = fb
        .ins()
        .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, ptr, null);
    fb.ins().uextend(I, eq)
}

/// Violation flag: 0 when `ptr == null`, else 1.
pub fn expect_null(fb: &mut FunctionBuilder, ptr: Value) -> Value {
    let null = fb.ins().iconst(P, 0);
    let ne = fb
        .ins()
        .icmp(cranelift_codegen::ir::condcodes::IntCC::NotEqual, ptr, null);
    fb.ins().uextend(I, ne)
}

/// Accumulate two i32 flags (sum; both are 0/1).
pub fn acc(fb: &mut FunctionBuilder, a: Value, b: Value) -> Value {
    fb.ins().iadd(a, b)
}

/// Address of a declared data blob (I64 value).
pub fn data_addr(fb: &mut FunctionBuilder, fx: &Fx, name: &str) -> Value {
    let gv = *fx
        .data
        .get(name)
        .unwrap_or_else(|| panic!("unknown data blob {name}"));
    fb.ins().global_value(P, gv)
}

/// Load i32 at ptr+offset.
pub fn load_i32(fb: &mut FunctionBuilder, ptr: Value, offset: i32) -> Value {
    fb.ins()
        .load(I, cranelift_codegen::ir::MemFlags::new(), ptr, offset)
}

/// Store i32 at ptr+offset.
pub fn store_i32(fb: &mut FunctionBuilder, ptr: Value, offset: i32, val: Value) {
    fb.ins()
        .store(cranelift_codegen::ir::MemFlags::new(), val, ptr, offset);
}

/// Load i64 at ptr+offset.
pub fn load_i64(fb: &mut FunctionBuilder, ptr: Value, offset: i32) -> Value {
    fb.ins()
        .load(P, cranelift_codegen::ir::MemFlags::new(), ptr, offset)
}

/// Store i64 at ptr+offset.
pub fn store_i64(fb: &mut FunctionBuilder, ptr: Value, offset: i32, val: Value) {
    fb.ins()
        .store(cranelift_codegen::ir::MemFlags::new(), val, ptr, offset);
}

/// Call a shim function by name with args; returns the result Values.
pub fn call_shim(fb: &mut FunctionBuilder, fx: &Fx, name: &str, args: &[Value]) -> Vec<Value> {
    let f = *fx
        .funcs
        .get(name)
        .unwrap_or_else(|| panic!("unknown shim function {name}"));
    let inst = fb.ins().call(f, args);
    fb.inst_results(inst).to_vec()
}

pub fn const_i32(fb: &mut FunctionBuilder, v: i64) -> Value {
    fb.ins().iconst(I, v)
}

pub fn const_ptr(fb: &mut FunctionBuilder, v: i64) -> Value {
    fb.ins().iconst(P, v)
}

// ---------------------------------------------------------------------------
// link + run (reusing the REAL ac-core entries)
// ---------------------------------------------------------------------------

pub struct ProbeOutcome {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Path to the built shim staticlib; builds it once per test process via a
/// contained cargo invocation (60s deadline, --locked per plan §6 evidence
/// contract — review r1 P774-R1-06).
fn shim_lib() -> Result<PathBuf, String> {
    static CACHE: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let mut cmd = Command::new("cargo");
            cmd.args([
                "build",
                "-p",
                "ac-text-runtime-shim",
                "--release",
                "--locked",
            ])
            .current_dir(&root);
            let (code, out, err) =
                auto_ac_prototype::link::run_deadlined_split(&mut cmd, Duration::from_secs(60))
                    .map_err(|e| format!("cargo build shim: {e:?}"))?;
            if code != Some(0) {
                return Err(format!(
                    "cargo build shim exit {code:?}: {out}{}{err}",
                    if out.is_empty() { "" } else { "\n" }
                ));
            }
            let lib = root.join("target/release/ac_text_runtime.lib");
            if lib.is_file() {
                Ok(lib)
            } else {
                Err(format!("shim staticlib missing at {}", lib.display()))
            }
        })
        .clone()
}

/// Build the object, link a real PE (rust-lld + kernel32 + shim) and run it
/// under the contained deadline executor. `case` names the scratch dir.
pub fn run_probe(case: &str, program: impl FnOnce(&mut ProbeCx)) -> ProbeOutcome {
    let mut cx = ProbeCx::new().expect("probe module setup");
    program(&mut cx);
    let obj = cx.finish().expect("probe object emission");

    let dir = std::env::temp_dir().join("ac770-text").join(case);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("probe scratch dir");
    let obj_path = dir.join("probe.obj");
    let exe_path = dir.join("probe.exe");
    std::fs::write(&obj_path, &obj).expect("write probe object");

    let lld = auto_ac_prototype::link::find_rust_lld().expect("locate rust-lld");
    let sdk = auto_ac_prototype::link::find_sdk_um_dir().expect("locate SDK um/x64");
    let shim = shim_lib().expect("shim staticlib");

    let mut link_cmd = Command::new(&lld);
    link_cmd
        .args([
            "-flavor",
            "link",
            "/nologo",
            &format!("/entry:{ENTRY_SYMBOL}"),
            "/subsystem:console",
            &format!("/out:{}", exe_path.display()),
            &format!("{}", obj_path.display()),
            &format!("/libpath:{}", sdk.display()),
            "kernel32.lib",
        ])
        .arg(&shim);
    let (code, out, err) = auto_ac_prototype::link::run_deadlined_split(&mut link_cmd, DEADLINE)
        .unwrap_or_else(|e| panic!("{case}: link executor: {e:?}"));
    assert_eq!(
        code,
        Some(0),
        "{case}: rust-lld failed: {}{}{}",
        out,
        if out.is_empty() { "" } else { "\n" },
        err
    );

    let mut run_cmd = Command::new(&exe_path);
    let (code, stdout, stderr) =
        auto_ac_prototype::link::run_deadlined_split(&mut run_cmd, DEADLINE)
            .unwrap_or_else(|e| panic!("{case}: run executor: {e:?}"));
    ProbeOutcome {
        code,
        stdout,
        stderr,
    }
}

/// Run an exe that is EXPECTED to hit the executor deadline (V-06 timeout
/// classification control) with a short explicit deadline, and OBSERVE the
/// predecessor trace: the child's stderr is redirected (via a `cmd /c`
/// wrapper INSIDE the same contained process tree) to a file that survives
/// the deadline kill — the deadline branch of the contained executor drops
/// piped output by design, so the file is the observable oracle
/// (review r1 P774-R1-06).
pub fn run_expect_deadline_traced(
    case: &str,
    program: impl FnOnce(&mut ProbeCx),
) -> (&'static str, String) {
    let mut cx = ProbeCx::new().expect("probe module setup");
    program(&mut cx);
    let obj = cx.finish().expect("probe object emission");

    let dir = std::env::temp_dir().join("ac770-text").join(case);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("probe scratch dir");
    let obj_path = dir.join("probe.obj");
    let exe_path = dir.join("probe.exe");
    let trace_path = dir.join("trace.txt");
    std::fs::write(&obj_path, &obj).expect("write probe object");

    let lld = auto_ac_prototype::link::find_rust_lld().expect("locate rust-lld");
    let sdk = auto_ac_prototype::link::find_sdk_um_dir().expect("locate SDK um/x64");
    let shim = shim_lib().expect("shim staticlib");

    let mut link_cmd = Command::new(&lld);
    link_cmd
        .args([
            "-flavor",
            "link",
            "/nologo",
            &format!("/entry:{ENTRY_SYMBOL}"),
            "/subsystem:console",
            &format!("/out:{}", exe_path.display()),
            &format!("{}", obj_path.display()),
            &format!("/libpath:{}", sdk.display()),
            "kernel32.lib",
        ])
        .arg(&shim);
    let (code, out, err) = auto_ac_prototype::link::run_deadlined_split(&mut link_cmd, DEADLINE)
        .unwrap_or_else(|e| panic!("{case}: link executor: {e:?}"));
    assert_eq!(code, Some(0), "{case}: rust-lld failed: {out}{err}");

    // cmd.exe redirects the hang PE's stderr into trace.txt; the wrapper and
    // the PE both live inside the contained job (single process tree). Bare
    // relative names + a working directory avoid every quoting layer.
    let mut run_cmd = Command::new("cmd");
    run_cmd
        .args(["/C", "probe.exe 2>trace.txt"])
        .current_dir(&dir);
    let kind =
        match auto_ac_prototype::link::run_deadlined_split(&mut run_cmd, Duration::from_secs(5)) {
            Err(auto_ac_prototype::link::SubprocError::Deadline) => "deadline",
            Err(auto_ac_prototype::link::SubprocError::DeadlineCollect) => "deadline-collect",
            Err(other) => panic!("{case}: unexpected executor error: {other:?}"),
            Ok((code, out, err)) => {
                panic!("{case}: expected deadline, process exited {code:?} (out={out} err={err})")
            }
        };
    let trace = std::fs::read_to_string(&trace_path)
        .unwrap_or_else(|e| panic!("{case}: read trace file: {e}"));
    (kind, trace)
}

/// Assert a PE outcome: exit code 0 and every listed trace line present in
/// stderr (in order).
pub fn assert_pass(outcome: &ProbeOutcome, expected_lines: &[&str]) {
    assert_eq!(
        outcome.code,
        Some(0),
        "probe PE failed; stderr:\n{}",
        outcome.stderr
    );
    let mut search_from = 0usize;
    for line in expected_lines {
        let pos = outcome.stderr[search_from..]
            .find(line)
            .unwrap_or_else(|| panic!("trace line missing: {line}\nstderr:\n{}", outcome.stderr));
        search_from += pos + line.len();
    }
}

/// Assert a trace line is ABSENT (e.g. no SUMMARY after a fatal trap).
pub fn assert_line_absent(outcome: &ProbeOutcome, line: &str) {
    assert!(
        !outcome.stderr.lines().any(|l| l == line),
        "trace line unexpectedly present: {line}\nstderr:\n{}",
        outcome.stderr
    );
}

/// Extract the FREESTAT line, parsed as (free_attempts, frees_ok, invalid).
pub fn freestat_of(outcome: &ProbeOutcome) -> (u32, u32, u32) {
    let line = outcome
        .stderr
        .lines()
        .rev()
        .find(|l| l.starts_with("FREESTAT "))
        .unwrap_or_else(|| panic!("no FREESTAT line; stderr:\n{}", outcome.stderr));
    let nums: Vec<u32> = line
        .split_whitespace()
        .filter_map(|tok| tok.split('=').nth(1).and_then(|v| v.parse::<u32>().ok()))
        .collect();
    assert_eq!(nums.len(), 3, "malformed FREESTAT line: {line}");
    (nums[0], nums[1], nums[2])
}

/// Extract the SUMMARY counters line, parsed as (attempts, allocs, frees,
/// live, live_bytes, peak). live is an object COUNT (u32); live_bytes/peak
/// are byte totals carried at full u64 precision — no field is narrowed on
/// the way out (r5 P774-R5-01).
pub fn summary_of(outcome: &ProbeOutcome) -> (u32, u32, u32, u32, u64, u64) {
    let line = outcome
        .stderr
        .lines()
        .rev()
        .find(|l| l.starts_with("SUMMARY "))
        .unwrap_or_else(|| panic!("no SUMMARY line; stderr:\n{}", outcome.stderr));
    let nums: Vec<u64> = line
        .split_whitespace()
        .filter_map(|tok| tok.split('=').nth(1).and_then(|v| v.parse::<u64>().ok()))
        .collect();
    assert_eq!(nums.len(), 6, "malformed SUMMARY line: {line}");
    (
        nums[0] as u32,
        nums[1] as u32,
        nums[2] as u32,
        nums[3] as u32,
        nums[4],
        nums[5],
    )
}
