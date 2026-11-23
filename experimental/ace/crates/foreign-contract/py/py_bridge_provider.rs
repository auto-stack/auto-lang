//! auto-ffi provider py/1 target=x86_64-pc-windows-msvc profile=py-bridge.v1
//!
//! PLAN-1026 T-03: the CPython adapter — a concrete Rust provider in the
//! 967 INT-1026 shape (a real pinned pyo3 dependency inside the B-01
//! source build closure). Every op rides the checked `p967.frame/1` tag
//! frame; Python objects live ONLY here (a u64 registry, the single
//! owner) while AV holds i32 tokens — never a PyO3 handle, never a
//! second resource ledger. Floats cross as an exact f64 bits pair
//! (T-01-frozen marshaling: non-finite/subnormal values are named
//! rejections, never truncated); display text crosses word-wise (4 ASCII
//! bytes per i32 lane, cap 256). v1 supported boundary: one serial
//! session per process, GIL-enabled CPython 3.14, trusted host.

// auto-ffi dep pyo3 0.29.3

use pyo3::prelude::*;
use pyo3::types::{PyDict, PySlice, PyTuple};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

// ---- frozen error classes (the `py_error_class` lane) -------------------
// 0 none | 1 ImportError | 2 AttributeError | 3 TypeError | 4 IndexError
// 5 ValueError | 6 OverflowError | 7 KeyError | 8 ZeroDivisionError
// 9 RuntimeError | 10 other
const CLASS_NONE: u8 = 0;
const CLASS_IMPORT: u8 = 1;
const CLASS_ATTRIBUTE: u8 = 2;
const CLASS_TYPE: u8 = 3;
const CLASS_INDEX: u8 = 4;
const CLASS_VALUE: u8 = 5;
const CLASS_OVERFLOW: u8 = 6;
const CLASS_KEY: u8 = 7;
const CLASS_ZERO_DIV: u8 = 8;
const CLASS_RUNTIME: u8 = 9;
const CLASS_OTHER: u8 = 10;
// domain rejections (not Python exceptions): 20 frame/domain misuse,
// 21 dead token, 22 text channel overflow/encoding, 23 float boundary
const CLASS_DOMAIN: u8 = 20;
const CLASS_DEAD: u8 = 21;
const CLASS_TEXT: u8 = 22;
const CLASS_FLOAT_BOUNDARY: u8 = 23;

/// Display/text channel cap in CODEPOINTS (PLAN-1094: the staged lane is
/// codepoint-wise UTF-32-LE words; the v1 ASCII-byte boundary was narrower).
const TEXT_CAP: usize = 256;

fn registry() -> &'static Mutex<BTreeMap<u64, Py<PyAny>>> {
    static R: OnceLock<Mutex<BTreeMap<u64, Py<PyAny>>>> = OnceLock::new();
    R.get_or_init(|| Mutex::new(BTreeMap::new()))
}
fn next_token() -> u64 {
    static N: AtomicU64 = AtomicU64::new(1);
    let t = N.fetch_add(1, Ordering::SeqCst);
    assert!(t <= i32::MAX as u64, "py-bridge v1: token space exhausted");
    t
}
fn str_cell() -> &'static Mutex<Vec<u8>> {
    static C: OnceLock<Mutex<Vec<u8>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(Vec::new()))
}
fn f64_cell() -> &'static Mutex<f64> {
    static C: OnceLock<Mutex<f64>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(0.0))
}
fn last_error() -> &'static Mutex<(u8, String)> {
    static E: OnceLock<Mutex<(u8, String)>> = OnceLock::new();
    E.get_or_init(|| Mutex::new((CLASS_NONE, String::new())))
}

fn ensure_interpreter() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| Python::initialize());
}

/// Record a Python failure for the `py_error_stage`/`py_error_class`
/// lanes: class code + "PyClassName: message" text (ASCII, capped).
fn record_py_err(py: Python<'_>, e: &PyErr) {
    // match on the type NAME (cross-version stable enough for the
    // frozen v1 table; unknown names fall to "other")
    let name = e
        .get_type(py)
        .qualname()
        .map(|s| s.to_string())
        .unwrap_or_default();
    let class = match name.as_str() {
        "ImportError" | "ModuleNotFoundError" => CLASS_IMPORT,
        "AttributeError" => CLASS_ATTRIBUTE,
        "TypeError" => CLASS_TYPE,
        "IndexError" => CLASS_INDEX,
        "ValueError" => CLASS_VALUE,
        "OverflowError" => CLASS_OVERFLOW,
        "KeyError" => CLASS_KEY,
        "ZeroDivisionError" => CLASS_ZERO_DIV,
        "RuntimeError" => CLASS_RUNTIME,
        _ => CLASS_OTHER,
    };
    let mut text = format!("{e}");
    if !text.is_ascii() {
        text = text
            .chars()
            .map(|c| if c.is_ascii() { c } else { '?' })
            .collect();
    }
    if text.len() > TEXT_CAP {
        text.truncate(TEXT_CAP);
    }
    *last_error().lock().unwrap() = (class, text);
}

fn record_domain(class: u8, msg: &str) {
    *last_error().lock().unwrap() = (class, msg.to_string());
}

fn store(py: Python<'_>, obj: Bound<'_, PyAny>) -> u64 {
    let t = next_token();
    registry().lock().unwrap().insert(t, obj.unbind());
    let _ = py;
    t
}

/// Resolve a frame token to the live object (dead tokens are the named
/// CLASS_DEAD rejection — a bumped/cleared registry is never trusted).
fn resolve(py: Python<'_>, token: u64) -> Option<Bound<'_, PyAny>> {
    registry()
        .lock()
        .unwrap()
        .get(&token)
        .map(|o| o.bind(py).clone())
}

/// One frame value -> one Python object (the frozen v1 domain: exact
/// i32/bool scalars, text (with the `F:` float-literal encoding used by
/// the compiler for compile-time-frozen kwargs), opaque tokens).
fn to_python(py: Python<'_>, v: &F0Val) -> PyResult<Py<PyAny>> {
    match v {
        F0Val::None => Ok(py.None()),
        F0Val::I32(n) => Ok(n.into_pyobject(py)?.unbind().into_any()),
        F0Val::Bool(b) => Ok(pyo3::types::PyBool::new(py, *b).to_owned().into_any().unbind()),
        F0Val::Text(t) => {
            if let Some(dec) = t.strip_prefix("F:") {
                let f: f64 = dec
                    .parse()
                    .map_err(|_| pyo3::exceptions::PyValueError::new_err("bad F: float literal"))?;
                Ok(f.into_pyobject(py)?.unbind().into_any())
            } else {
                Ok(t.into_pyobject(py)?.unbind().into_any())
            }
        }
        F0Val::Opaque(o) => registry()
            .lock()
            .unwrap()
            .get(o)
            .map(|h| h.clone_ref(py))
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("py.dead-object")),
    }
}

fn run_guarded<T>(f: impl FnOnce(Python<'_>) -> PyResult<T>) -> Option<T> {
    ensure_interpreter();
    Python::attach(|py| match f(py) {
        Ok(v) => Some(v),
        Err(e) => {
            record_py_err(py, &e);
            None
        }
    })
}

// The ops -------------------------------------------------------------------
//
// Frame schemas (frozen, T-01): the receiver is the target object token
// (`n` for py_import/admission); positionals[0] is a text member/label
// where noted; kwargs carry named scalars. All fallible ops return None
// on the first failure (the shim maps it to the op's single frozen
// status); the class+text lanes recover the specifics.

// auto-ffi fn py_admission symbol=py.admission.v1 form=fallible errors=py.init:-100 frame=p967.frame/1
pub fn py_admission(_frame: &F0Frame) -> Option<i32> {
    ensure_interpreter();
    Some(1)
}

// auto-ffi fn py_import symbol=py.import.v1 form=fallible errors=py.import:-101 frame=p967.frame/1
pub fn py_import(frame: &F0Frame) -> Option<i32> {
    let module = frame
        .kwargs
        .iter()
        .find(|k| k.name == "mod")
        .and_then(|k| match &k.value {
            F0Val::Text(t) => Some(t.clone()),
            _ => None,
        })?;
    let member = frame
        .kwargs
        .iter()
        .find(|k| k.name == "member")
        .and_then(|k| match &k.value {
            F0Val::Text(t) => Some(t.clone()),
            _ => None,
        })?;
    if module.is_empty() || member.is_empty() {
        record_domain(CLASS_DOMAIN, "py.import: mod/member required");
        return None;
    }
    run_guarded(|py| {
        let m = py.import(module.as_str())?;
        let attr = m.getattr(member.as_str())?;
        Ok(store(py, attr) as i32)
    })
}

// auto-ffi fn py_getattr symbol=py.getattr.v1 form=fallible errors=py.attr:-102 frame=p967.frame/1
pub fn py_getattr(frame: &F0Frame) -> Option<i32> {
    let token = match &frame.receiver {
        F0Val::Opaque(o) => *o,
        _ => {
            record_domain(CLASS_DOMAIN, "py.getattr: receiver token required");
            return None;
        }
    };
    let name = match frame.positionals.first() {
        Some(F0Val::Text(t)) => t.clone(),
        _ => {
            record_domain(CLASS_DOMAIN, "py.getattr: member name required");
            return None;
        }
    };
    run_guarded(|py| {
        let obj = resolve(py, token).ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("py.dead-object")
        })?;
        let attr = obj.getattr(name.as_str())?;
        Ok(store(py, attr) as i32)
    })
}

// auto-ffi fn py_call symbol=py.call.v1 form=fallible errors=py.call:-103 frame=p967.frame/1
pub fn py_call(frame: &F0Frame) -> Option<i32> {
    let token = match &frame.receiver {
        F0Val::Opaque(o) => *o,
        _ => {
            record_domain(CLASS_DOMAIN, "py.call: receiver token required");
            return None;
        }
    };
    // positionals[0] is the member name ("" = call the receiver itself).
    let member = match frame.positionals.first() {
        Some(F0Val::Text(t)) => t.clone(),
        _ => {
            record_domain(CLASS_DOMAIN, "py.call: member label required");
            return None;
        }
    };
    let args = &frame.positionals[1.min(frame.positionals.len())..];
    run_guarded(|py| {
        let target = resolve(py, token).ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("py.dead-object")
        })?;
        let callable = if member.is_empty() {
            target
        } else {
            target.getattr(member.as_str())?
        };
        let py_args: PyResult<Vec<Py<PyAny>>> =
            args.iter().map(|v| to_python(py, v)).collect();
        let args_tuple = PyTuple::new(py, py_args?)?;
        let kwargs = PyDict::new(py);
        for k in &frame.kwargs {
            let v = to_python(py, &k.value)?;
            kwargs.set_item(k.name.as_str(), v)?;
        }
        let result = callable.call(&args_tuple, Some(&kwargs))?;
        Ok(store(py, result) as i32)
    })
}

// auto-ffi fn py_getitem symbol=py.getitem.v1 form=fallible errors=py.index:-105 frame=p967.frame/1
pub fn py_getitem(frame: &F0Frame) -> Option<i32> {
    let token = match &frame.receiver {
        F0Val::Opaque(o) => *o,
        _ => {
            record_domain(CLASS_DOMAIN, "py.getitem: receiver token required");
            return None;
        }
    };
    run_guarded(|py| {
        let obj = resolve(py, token).ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("py.dead-object")
        })?;
        let key = if let Some(key) = frame.positionals.first() {
            to_python(py, key)?.into_bound(py)
        } else if let [start, stop, step] = frame.kwargs.as_slice() {
            if start.name != "start" || stop.name != "stop" || step.name != "step" {
                return Err(pyo3::exceptions::PyTypeError::new_err(
                    "py.getitem: expected start, stop and step",
                ));
            }
            // The CPython type accepts arbitrary Python objects, including None.
            // The key stays local to this call and never gets a session token.
            py.get_type::<PySlice>().call1((
                to_python(py, &start.value)?,
                to_python(py, &stop.value)?,
                to_python(py, &step.value)?,
            ))?
        } else {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "py.getitem: key required",
            ));
        };
        let result = obj.get_item(&key)?;
        Ok(store(py, result) as i32)
    })
}

// auto-ffi fn py_binop symbol=py.binop.v1 form=fallible errors=py.binop:-104 frame=p967.frame/1
pub fn py_binop(frame: &F0Frame) -> Option<i32> {
    let token = match &frame.receiver {
        F0Val::Opaque(o) => *o,
        _ => {
            record_domain(CLASS_DOMAIN, "py.binop: receiver token required");
            return None;
        }
    };
    let op = frame
        .kwargs
        .iter()
        .find(|k| k.name == "op")
        .and_then(|k| match &k.value {
            F0Val::Text(t) => Some(t.clone()),
            _ => None,
        })?;
    let dunder = match op.as_str() {
        "add" => "__add__",
        "sub" => "__sub__",
        "mul" => "__mul__",
        "truediv" => "__truediv__",
        _ => {
            record_domain(CLASS_DOMAIN, "py.binop: unsupported op (v1: add/sub/mul/truediv)");
            return None;
        }
    };
    let operand = frame.positionals.first()?;
    run_guarded(|py| {
        let obj = resolve(py, token).ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("py.dead-object")
        })?;
        let method = obj.getattr(dunder)?;
        let arg = to_python(py, operand)?;
        let result = method.call1((arg.bind(py),))?;
        Ok(store(py, result) as i32)
    })
}

// auto-ffi fn py_to_f64 symbol=py.to_f64.v1 form=fallible errors=py.to-f64:-106 frame=p967.frame/1
pub fn py_to_f64(frame: &F0Frame) -> Option<i32> {
    let token = match &frame.receiver {
        F0Val::Opaque(o) => *o,
        _ => {
            record_domain(CLASS_DOMAIN, "py.to_f64: receiver token required");
            return None;
        }
    };
    run_guarded(|py| {
        let obj = resolve(py, token).ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("py.dead-object")
        })?;
        let v: f64 = obj.extract()?;
        // T-01 frozen float boundary: exact IEEE normal values only —
        // non-finite and subnormal payloads are NAMED rejections (never
        // a silent truncation); zero is exact and fine.
        if !v.is_finite() || v.is_subnormal() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "py.float-boundary: non-finite/subnormal value rejected (v1)",
            ));
        }
        let bits = v.to_bits();
        *f64_cell().lock().unwrap() = v;
        Ok((bits >> 32) as i32)
    })
}

// auto-ffi fn py_f64_lo symbol=py.f64_lo.v1 form=pure ret=i32 frame=p967.frame/1
pub fn py_f64_lo(_frame: &F0Frame) -> i32 {
    let bits = f64_cell().lock().unwrap().to_bits();
    bits as i32
}

// auto-ffi fn py_to_i32 symbol=py.to_i32.v1 form=fallible errors=py.to-i32:-107 frame=p967.frame/1
pub fn py_to_i32(frame: &F0Frame) -> Option<i32> {
    let token = match &frame.receiver {
        F0Val::Opaque(o) => *o,
        _ => {
            record_domain(CLASS_DOMAIN, "py.to_i32: receiver token required");
            return None;
        }
    };
    run_guarded(|py| {
        let obj = resolve(py, token).ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("py.dead-object")
        })?;
        let wide: i64 = obj.extract()?;
        i32::try_from(wide)
            .map_err(|_| pyo3::exceptions::PyOverflowError::new_err("py.to_i32: out of range"))
    })
}

// auto-ffi fn py_to_bool symbol=py.to_bool.v1 form=fallible errors=py.to-bool:-108 frame=p967.frame/1
pub fn py_to_bool(frame: &F0Frame) -> Option<i32> {
    let token = match &frame.receiver {
        F0Val::Opaque(o) => *o,
        _ => {
            record_domain(CLASS_DOMAIN, "py.to_bool: receiver token required");
            return None;
        }
    };
    run_guarded(|py| {
        let obj = resolve(py, token).ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("py.dead-object")
        })?;
        let b = obj.is_truthy()?;
        Ok(b as i32)
    })
}

// auto-ffi fn py_stage_str symbol=py.stage_str.v1 form=fallible errors=py.stage-str:-109 frame=p967.frame/1
pub fn py_stage_str(frame: &F0Frame) -> Option<i32> {
    let token = match &frame.receiver {
        F0Val::Opaque(o) => *o,
        _ => {
            record_domain(CLASS_DOMAIN, "py.stage_str: receiver token required");
            return None;
        }
    };
    let repr = frame
        .kwargs
        .iter()
        .find(|k| k.name == "spec")
        .map(|k| matches!(&k.value, F0Val::Text(t) if t == "repr"));
    run_guarded(|py| {
        let obj = resolve(py, token).ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("py.dead-object")
        })?;
        let s = if repr == Some(true) { obj.repr()? } else { obj.str()? };
        let text = s.to_cow()?.to_string();
        // PLAN-1094: the staged lane is CODEPOINT-wise (UTF-32-LE words),
        // so Unicode text round-trips byte-exactly through the word lane
        // (the v1 ASCII byte-lane was a narrower boundary). Still bounded:
        // the cap is now in codepoints.
        let chars: Vec<u32> = text.chars().map(u32::from).collect();
        if chars.len() > TEXT_CAP {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "py.text-channel: over cap ({}/{TEXT_CAP})",
                chars.len()
            )));
        }
        let mut bytes = Vec::with_capacity(chars.len() * 4);
        for c in chars {
            bytes.extend(c.to_le_bytes());
        }
        *str_cell().lock().unwrap() = bytes;
        Ok((str_cell().lock().unwrap().len() / 4) as i32)
    })
}

// auto-ffi fn py_str_word symbol=py.str_word.v1 form=pure ret=i32 frame=p967.frame/1
pub fn py_str_word(frame: &F0Frame) -> i32 {
    let pos = match frame.positionals.first() {
        Some(F0Val::I32(n)) if *n >= 0 => *n as usize,
        _ => return 0,
    };
    let cell = str_cell().lock().unwrap();
    let mut word: u32 = 0;
    for k in 0..4 {
        let idx = pos * 4 + k;
        let b = cell.get(idx).copied().unwrap_or(0);
        word |= (b as u32) << (8 * k);
    }
    word as i32
}

// auto-ffi fn py_error_stage symbol=py.error_stage.v1 form=pure ret=i32 frame=p967.frame/1
pub fn py_error_stage(_frame: &F0Frame) -> i32 {
    let (class, text) = last_error().lock().unwrap().clone();
    // PLAN-1094: the same codepoint-wise lane as py_stage_str.
    let staged = format!("[{class}] {text}");
    let mut bytes = Vec::with_capacity(staged.chars().count() * 4);
    for c in staged.chars() {
        bytes.extend(u32::from(c).to_le_bytes());
    }
    *str_cell().lock().unwrap() = bytes;
    (str_cell().lock().unwrap().len() / 4) as i32
}

// auto-ffi fn py_error_class symbol=py.error_class.v1 form=pure ret=i32 frame=p967.frame/1
pub fn py_error_class(_frame: &F0Frame) -> i32 {
    last_error().lock().unwrap().0 as i32
}

// auto-ffi fn py_release symbol=py.release.v1 form=fallible errors=py.release:-110 frame=p967.frame/1
pub fn py_release(frame: &F0Frame) -> Option<i32> {
    let token = match &frame.receiver {
        F0Val::Opaque(o) => *o,
        _ => {
            record_domain(CLASS_DOMAIN, "py.release: receiver token required");
            return None;
        }
    };
    match registry().lock().unwrap().remove(&token) {
        Some(_) => Some(0),
        None => {
            record_domain(CLASS_DEAD, "py.release: unknown token");
            None
        }
    }
}

// auto-ffi fn py_alive symbol=py.alive.v1 form=pure ret=i32 frame=p967.frame/1
pub fn py_alive(_frame: &F0Frame) -> i32 {
    registry().lock().unwrap().len() as i32
}

// auto-ffi fn py_session_close symbol=py.session_close.v1 form=fallible errors=py.close:-111 frame=p967.frame/1
pub fn py_session_close(_frame: &F0Frame) -> Option<i32> {
    // Session close releases ONLY this bridge's own references (never
    // Py_Finalize — the interpreter is host infrastructure, R-06).
    let n = registry().lock().unwrap().len();
    registry().lock().unwrap().clear();
    *last_error().lock().unwrap() = (CLASS_NONE, String::new());
    str_cell().lock().unwrap().clear();
    Some(n as i32)
}
