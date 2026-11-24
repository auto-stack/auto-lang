// ---------------------------------------------------------------------------
// a2ts runtime body (PLAN-976) -- plain ESM JavaScript, no dependencies.
//
// This module is the target runtime for AutoLang programs compiled to ES
// modules by the Checked-HIR TypeScript / JavaScript provider. It carries every
// Auto semantic that the host language would otherwise change:
//
//   * checked i32 arithmetic (traps instead of wrapping / becoming float);
//   * i64/u64 as JS `bigint` end to end (never through `Number`);
//   * C/Rust truncation-toward-zero `/` and `%`;
//   * UTF-8 byte text (`Uint8Array`) with Unicode-scalar `len`/`char_at`/slice
//     and embedded-NUL fidelity, owned `Str` with an explicit free ledger;
//   * bit-exact f32/f64 materialisation (NaN payloads, signed zero);
//   * property-style port operations (`a2x_port_call`) with contract checks;
//   * a deterministic ledger + snapshot for conformance harnesses.
//
// Determinism rules: no `console.log`, no wall clock, no randomness, no
// floating-point formatting of host-specific shape. Output always flows
// through `_RT.emit_stdout` / `_RT.emit_stderr`.
//
// NOTE on the identity constants: this body is the single source of truth for
// `A2X_RUNTIME_VERSION` (and therefore for `A2X_PORT_DEP`). The emitter injects
// the identity of the *emitted* file through globals read at module evaluation:
//
//   globalThis.__a2ts_runtime_version      (default "a2ts-runtime/1")
//   globalThis.__a2ts_runtime_body_sha256  (default "", bare import)
//   globalThis.__a2ts_target_lang          (default "javascript")
//
// A bare import of this file therefore reports the frozen version tag, an empty
// body hash and the JavaScript leg, which keeps `A2X_PORT_DEP` self-consistent
// there. The emitter must NOT re-declare these names, nor `a2x_mark_a` /
// `a2x_mark_b` (both are exported here and only need view authorisation).
// ---------------------------------------------------------------------------

const A2X_I32_MIN = -2147483648;
const A2X_I32_MAX = 2147483647;
const A2X_DEFAULT_BUDGET = 1 << 24;
const A2X_EMPTY_BYTES = new Uint8Array(0);
const A2X_VERSION_FALLBACK = "a2ts-runtime/1";

function _injectedString(key, fallback) {
  try {
    const g = globalThis;
    if (g === undefined || g === null) return fallback;
    const v = g[key];
    return typeof v === "string" ? v : fallback;
  } catch (_e) {
    return fallback;
  }
}

/** Version tag of this runtime (participates in every port contract). */
export const A2X_RUNTIME_VERSION = _injectedString(
  "__a2ts_runtime_version",
  A2X_VERSION_FALLBACK,
);

/**
 * Deterministic SHA-256 of the emitted runtime body. Injected by the emitter
 * (`globalThis.__a2ts_runtime_body_sha256`); empty for a bare import of this
 * file, which keeps `A2X_PORT_DEP` self-consistent there too.
 */
export const A2X_RUNTIME_BODY_SHA256 = _injectedString(
  "__a2ts_runtime_body_sha256",
  "",
);

/** The dependency token every generated port call must pass back. */
export const A2X_PORT_DEP =
  A2X_RUNTIME_VERSION + "@" + A2X_RUNTIME_BODY_SHA256;

/**
 * Script language of the emitted program (`"typescript"` / `"javascript"`).
 * Injected by the emitter (`globalThis.__a2ts_target_lang`); a bare import of
 * this file reports the JavaScript leg, because the body is plain ESM.
 */
export const A2X_TARGET_LANG = _injectedString(
  "__a2ts_target_lang",
  "javascript",
);

// ---------------------------------------------------------------------------
// Traps
// ---------------------------------------------------------------------------

/** A deterministic Auto-semantics trap (never a host TypeError). */
export class A2xTrap extends Error {
  constructor(code, detail = "", port_id = null) {
    super(detail || code);
    this.name = "A2xTrap";
    this.code = code;
    this.detail = detail;
    this.port_id = port_id === undefined ? null : port_id;
  }
}

function _trap(code, detail, port_id) {
  throw new A2xTrap(code, detail === undefined ? "" : detail, port_id);
}

/** Recover the owned message of a checked language failure. */
export function a2x_catch_failure(error) {
  if (!(error instanceof A2xTrap) || error.code !== "Fail") throw error;
  return new A2xOwnedStr(_encoder.encode(error.detail));
}

function _show(v) {
  if (v === null) return "null";
  if (v === undefined) return "undefined";
  if (typeof v === "bigint") return `${v}n`;
  if (typeof v === "string") return JSON.stringify(v);
  if (typeof v === "number" || typeof v === "boolean") return String(v);
  return Object.prototype.toString.call(v);
}

// ---------------------------------------------------------------------------
// Ledger + output sinks
// ---------------------------------------------------------------------------

function _nodeProcess() {
  try {
    if (typeof process !== "undefined" && process !== null) return process;
  } catch (_e) {
    /* not node */
  }
  return null;
}

function _nodeStdout() {
  const p = _nodeProcess();
  if (p === null) return null;
  const out = p.stdout;
  if (out !== null && out !== undefined && typeof out.write === "function") {
    return out;
  }
  return null;
}

function _nodeStderr() {
  const p = _nodeProcess();
  if (p === null) return null;
  const err = p.stderr;
  if (err !== null && err !== undefined && typeof err.write === "function") {
    return err;
  }
  return null;
}

function _eventsEnabled() {
  const p = _nodeProcess();
  try {
    if (p !== null && p.env && p.env.A2X_RT_EVENTS === "1") return true;
    if (globalThis.A2X_RT_EVENTS === "1") return true;
  } catch (_e) {
    /* ignore */
  }
  return false;
}

function _u8(value) {
  if (value instanceof Uint8Array) return value;
  if (value instanceof ArrayBuffer) return new Uint8Array(value);
  if (ArrayBuffer.isView(value)) {
    return new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
  }
  return null;
}

/** Concatenate UTF-8 byte buffers into one fresh `Uint8Array`. */
function _concatBytes(chunks) {
  let total = 0;
  for (const c of chunks) total += c.length;
  const out = new Uint8Array(total);
  let at = 0;
  for (const c of chunks) {
    out.set(c, at);
    at += c.length;
  }
  return out;
}

const _RT = {
  // Str ownership ledger.
  next_id: 1,
  live_strs: new Map(),
  freed_strs: [],
  double_free_strs: [],
  alloc_count: 0,
  free_count: 0,
  double_free_count: 0,
  live_bytes: 0,
  // Diagnostics.
  marks: [],
  events: [],
  budget_left: A2X_DEFAULT_BUDGET,
  events_enabled: false,
  // Output sinks (browser / no-stdout hosts).
  stdout_chunks: [],
  stderr_chunks: [],
  // Resource handles (`io.*` buffer readers).
  live_handles: new Set(),
  closed_handles: new Set(),
  handle_trace: [],
  // Lazily loaded stdin (node); harnesses may preload `stdin_lines`.
  stdin_lines: null,
  stdin_loaded: false,
  stdin_bytes: A2X_EMPTY_BYTES,
  stdin_pos: 0,

  reset() {
    this.next_id = 1;
    this.live_strs = new Map();
    this.freed_strs = [];
    this.double_free_strs = [];
    this.alloc_count = 0;
    this.free_count = 0;
    this.double_free_count = 0;
    this.live_bytes = 0;
    this.marks = [];
    this.events = [];
    this.budget_left = A2X_DEFAULT_BUDGET;
    this.events_enabled = _eventsEnabled();
    this.stdout_chunks = [];
    this.stderr_chunks = [];
    this.live_handles = new Set();
    this.closed_handles = new Set();
    this.handle_trace = [];
    this.stdin_lines = null;
    this.stdin_loaded = false;
    this.stdin_bytes = A2X_EMPTY_BYTES;
    this.stdin_pos = 0;
  },

  /** Write raw UTF-8 bytes to stdout (node) or buffer them (other hosts). */
  emit_stdout(bytes) {
    const chunk = _u8(bytes);
    if (chunk === null) {
      _trap("InvalidScalar", "stdout payload is not UTF-8 bytes");
    }
    const out = _nodeStdout();
    if (out !== null) {
      out.write(chunk);
      return;
    }
    _RT.stdout_chunks.push(chunk.slice());
  },

  /** Write diagnostic text/bytes to stderr (node) or buffer them. */
  emit_stderr(text) {
    const u8 = typeof text === "string" ? _encoder.encode(text) : _u8(text);
    if (u8 === null) return;
    const err = _nodeStderr();
    if (err !== null) {
      err.write(u8);
      return;
    }
    _RT.stderr_chunks.push(u8.slice());
  },

  /**
   * Emit buffered stdout on hosts without `process.stdout`: a sink installed at
   * `globalThis.__a2ts_stdout_sink` (a function or an object with `write`) gets
   * the concatenated bytes; otherwise the chunks stay readable from
   * `__a2ts_rt.stdout_chunks` for the embedding harness.
   */
  flush_stdout() {
    if (_RT.stdout_chunks.length === 0) return;
    let sink = null;
    try {
      sink = globalThis.__a2ts_stdout_sink;
    } catch (_e) {
      sink = null;
    }
    if (sink === null || sink === undefined) return;
    const blob = _concatBytes(_RT.stdout_chunks);
    _RT.stdout_chunks = [];
    if (typeof sink === "function") {
      sink(blob);
      return;
    }
    if (typeof sink.write === "function") {
      sink.write(blob);
    }
  },

  flush_stderr() {
    if (_RT.stderr_chunks.length === 0) return;
    let sink = null;
    try {
      sink = globalThis.__a2ts_stderr_sink;
    } catch (_e) {
      sink = null;
    }
    if (sink === null || sink === undefined) return;
    const blob = _concatBytes(_RT.stderr_chunks);
    _RT.stderr_chunks = [];
    if (typeof sink === "function") {
      sink(blob);
      return;
    }
    if (typeof sink.write === "function") {
      sink.write(blob);
    }
  },
};

/**
 * Node's `process.stdout.write` accepts `Uint8Array` directly, so output needs
 * no Buffer conversion (which would also break non-node hosts).
 */

_RT.reset();
globalThis.__a2ts_rt = _RT;

/** Reset the whole runtime ledger (fresh program run). */
export function a2x_rt_reset() {
  _RT.reset();
}

/** Plain, JSON-friendly snapshot of the runtime ledger. */
export function a2x_rt_snapshot() {
  const live = Array.from(_RT.live_strs.keys());
  live.sort((a, b) => a - b);
  return {
    runtime_version: A2X_RUNTIME_VERSION,
    alloc_count: _RT.alloc_count,
    free_count: _RT.free_count,
    double_free_count: _RT.double_free_count,
    live_bytes: _RT.live_bytes,
    live_strs: live,
    marks: _RT.marks.slice(),
    budget_left: _RT.budget_left,
  };
}

/** Charge `units` against the execution budget (trap `Budget` when negative). */
export function a2x_bill(units) {
  const n = _intArg(units, "budget units");
  _RT.budget_left -= n;
  if (_RT.budget_left < 0) {
    _trap("Budget", "execution budget exhausted");
  }
}

// ---------------------------------------------------------------------------
// Numeric helpers: i32 family (JS number, every op explicit and checked)
// ---------------------------------------------------------------------------

function _intArg(v, what) {
  if (typeof v !== "number" || !Number.isInteger(v)) {
    _trap("Overflow", `${what} is not an integer: ${_show(v)}`);
  }
  return v;
}

function _checkedI32(v, detail) {
  if (typeof v !== "number" || !Number.isInteger(v)) {
    _trap("Overflow", `${detail}: ${_show(v)}`);
  }
  if (v < A2X_I32_MIN || v > A2X_I32_MAX) {
    _trap("Overflow", `${detail}: ${String(v)}`);
  }
  return v;
}

function _normZero(n) {
  return n === 0 ? 0 : n;
}

/** Validate/return a checked i32. */
export function a2x_check_i32(v) {
  return _checkedI32(v, "i32 range exceeded");
}

/** Validate/return an integer inside `[lo, hi]` (non-integers trap). */
export function a2x_check_range(v, lo, hi) {
  if (typeof v !== "number" || !Number.isInteger(v)) {
    _trap("Overflow", `integer range check on non-integer: ${_show(v)}`);
  }
  if (v < lo || v > hi) {
    _trap("Overflow", `integer range [${String(lo)}, ${String(hi)}] exceeded: ${String(v)}`);
  }
  return v;
}

/** Checked i32 addition. */
export function a2x_add(a, b) {
  return _checkedI32(_intArg(a, "addend") + _intArg(b, "addend"), "i32 add overflow");
}

/** Checked i32 subtraction. */
export function a2x_sub(a, b) {
  return _checkedI32(_intArg(a, "minuend") - _intArg(b, "subtrahend"), "i32 sub overflow");
}

/** Checked i32 multiplication. */
export function a2x_mul(a, b) {
  return _checkedI32(_intArg(a, "factor") * _intArg(b, "factor"), "i32 mul overflow");
}

/** Checked i32 negation (INT_MIN traps). */
export function a2x_neg(a) {
  const ia = _intArg(a, "operand");
  if (ia === A2X_I32_MIN) {
    _trap("Overflow", "negation of i32 MIN");
  }
  return _checkedI32(-ia, "i32 negation overflow");
}

/** C truncating-toward-zero i32 division, INT_MIN / -1 traps. */
export function a2x_div(a, b) {
  const ia = _intArg(a, "dividend");
  const ib = _intArg(b, "divisor");
  if (ib === 0) {
    _trap("DivByZero", "division by zero");
  }
  if (ia === A2X_I32_MIN && ib === -1) {
    _trap("Overflow", "i32 MIN / -1 overflow");
  }
  return _checkedI32(_normZero(Math.trunc(ia / ib)), "i32 division overflow");
}

/** C truncating-toward-zero i32 remainder (sign follows the dividend). */
export function a2x_rem(a, b) {
  const ia = _intArg(a, "dividend");
  const ib = _intArg(b, "divisor");
  if (ib === 0) {
    _trap("RemZero", "remainder by zero");
  }
  if (ia === A2X_I32_MIN && ib === -1) {
    _trap("Overflow", "i32 MIN % -1 overflow");
  }
  const q = _normZero(Math.trunc(ia / ib));
  return _checkedI32(_normZero(ia - q * ib), "i32 remainder overflow");
}

/** Unchecked C truncating division (caller range-checks the result). */
export function a2x_c_trunc_div(a, b) {
  const ia = _intArg(a, "dividend");
  const ib = _intArg(b, "divisor");
  if (ib === 0) {
    _trap("DivByZero", "division by zero");
  }
  return _normZero(Math.trunc(ia / ib));
}

/** Unchecked C truncating remainder (caller range-checks the result). */
export function a2x_c_trunc_rem(a, b) {
  const ia = _intArg(a, "dividend");
  const ib = _intArg(b, "divisor");
  if (ib === 0) {
    _trap("RemZero", "remainder by zero");
  }
  const q = _normZero(Math.trunc(ia / ib));
  return _normZero(ia - q * ib);
}

function _shiftAmount(shift, width) {
  let s;
  if (typeof shift === "bigint") {
    s = Number(shift);
  } else if (typeof shift === "number") {
    s = shift;
  } else {
    _trap("Overflow", `shift amount is not an integer: ${_show(shift)}`);
  }
  if (!Number.isInteger(s)) {
    _trap("Overflow", `shift amount is not an integer: ${_show(shift)}`);
  }
  const w =
    width === 8 || width === 16 || width === 32 || width === 64 ? width : 64;
  return ((s % w) + w) % w;
}

/** Shift-left inside a `width`-bit lane (amount masked, caller range-checks). */
export function a2x_shl(a, b, width) {
  const ia = _intArg(a, "value");
  const s = _shiftAmount(b, width);
  return ia * 2 ** s;
}

/** Arithmetic (floor) shift-right inside a `width`-bit lane. */
export function a2x_shr(a, b, width) {
  const ia = _intArg(a, "value");
  const s = _shiftAmount(b, width);
  return Math.floor(ia / 2 ** s);
}

// ---------------------------------------------------------------------------
// Numeric helpers: i64 / u64 (JS bigint end to end)
// ---------------------------------------------------------------------------

function _mustBig(v, what) {
  if (typeof v !== "bigint") {
    _trap("Overflow", `${what} is not a bigint: ${_show(v)}`);
  }
  return v;
}

function _bound(v) {
  if (typeof v === "bigint") return v;
  if (typeof v === "number" && Number.isInteger(v)) return BigInt(Math.trunc(v));
  _trap("Overflow", `integer bound is not an integer: ${_show(v)}`);
}

/** Validate/return a bigint inside `[lo, hi]` (non-bigints trap). */
export function a2x_bcheck(v, lo, hi) {
  const bv = _mustBig(v, "value");
  const l = _bound(lo);
  const h = _bound(hi);
  if (bv < l || bv > h) {
    _trap("Overflow", `integer range [${l}n, ${h}n] exceeded: ${bv}n`);
  }
  return bv;
}

/** Checked bigint addition. */
export function a2x_badd(v, w, lo, hi) {
  return a2x_bcheck(_mustBig(v, "addend") + _mustBig(w, "addend"), lo, hi);
}

/** Checked bigint subtraction. */
export function a2x_bsub(v, w, lo, hi) {
  return a2x_bcheck(_mustBig(v, "minuend") - _mustBig(w, "subtrahend"), lo, hi);
}

/** Checked bigint multiplication. */
export function a2x_bmul(v, w, lo, hi) {
  return a2x_bcheck(_mustBig(v, "factor") * _mustBig(w, "factor"), lo, hi);
}

/** C truncating bigint division (zero traps, result range-checked). */
export function a2x_bdiv(v, w, lo, hi) {
  const a = _mustBig(v, "dividend");
  const b = _mustBig(w, "divisor");
  if (b === 0n) {
    _trap("DivByZero", "division by zero");
  }
  return a2x_bcheck(a / b, lo, hi);
}

/** C truncating bigint remainder (sign follows the dividend). */
export function a2x_brem(v, w, lo, hi) {
  const a = _mustBig(v, "dividend");
  const b = _mustBig(w, "divisor");
  if (b === 0n) {
    _trap("RemZero", "remainder by zero");
  }
  return a2x_bcheck(a % b, lo, hi);
}

/** Checked bigint negation. */
export function a2x_bneg(v, lo, hi) {
  return a2x_bcheck(-_mustBig(v, "operand"), lo, hi);
}

/** Shift-left of a bigint lane (amount masked to 0..width-1, then checked). */
export function a2x_bshl(v, shift, width, lo, hi) {
  const a = _mustBig(v, "value");
  const s = _shiftAmount(shift, width);
  return a2x_bcheck(a << BigInt(s), lo, hi);
}

/**
 * Shift-right of a bigint lane: arithmetic for signed lanes (`lo < 0`) and
 * logical 64-bit for unsigned lanes (matching the Auto `u64` contract).
 */
export function a2x_bshr(v, shift, width, lo, hi) {
  const a = _mustBig(v, "value");
  const s = _shiftAmount(shift, width);
  const l = _bound(lo);
  const h = _bound(hi);
  const shifted =
    l < 0n ? a >> BigInt(s) : BigInt.asUintN(64, a) >> BigInt(s);
  return a2x_bcheck(shifted, l, h);
}

// ---------------------------------------------------------------------------
// Numeric helpers: casts and bit-exact float materialisation
// ---------------------------------------------------------------------------

function _toNum(v, what) {
  if (typeof v === "number") return v;
  if (typeof v === "bigint") return Number(v);
  if (typeof v === "boolean") return v ? 1 : 0;
  _trap("Overflow", `${what} is not numeric: ${_show(v)}`);
}

function _u32bits(v) {
  if (typeof v === "bigint") return Number(BigInt.asUintN(32, v));
  if (typeof v === "number" && Number.isInteger(v)) {
    return Number(BigInt.asUintN(32, BigInt(Math.trunc(v))));
  }
  _trap("Overflow", `f32 bit pattern is not an integer: ${_show(v)}`);
}

function _u64bits(v) {
  if (typeof v === "bigint") return BigInt.asUintN(64, v);
  if (typeof v === "number" && Number.isInteger(v)) {
    return BigInt.asUintN(64, BigInt(Math.trunc(v)));
  }
  _trap("Overflow", `f64 bit pattern is not an integer: ${_show(v)}`);
}

const _f32Buffer = new ArrayBuffer(4);
const _f32Value = new Float32Array(_f32Buffer);
const _f32Pattern = new Uint32Array(_f32Buffer);
const _f64Buffer = new ArrayBuffer(8);
const _f64View = new DataView(_f64Buffer);

function _toNumArg(v, what) { return _toNum(v, what); }

/** Materialise the f32 value of a u32 bit pattern (NaN/signed-zero exact). */
export function a2x_f32_bits(bits) {
  _f32Pattern[0] = _u32bits(bits);
  return _f32Value[0];
}

/** Materialise the f64 value of a u64 bit pattern (NaN/signed-zero exact). */
export function a2x_f64_bits(bits) {
  _f64View.setBigUint64(0, _u64bits(bits), true);
  return _f64View.getFloat64(0, true);
}

/** Round to f32 precision. */
export function a2x_f32_round(x) {
  return Math.fround(_toNumArg(x, "f32 operand"));
}

/** Cast to f32 (rounding, host NaN canonicalised). */
export function a2x_cast_f32(v) {
  return Math.fround(_toNumArg(v, "f32 cast operand"));
}

/** Cast to f64. */
export function a2x_cast_f64(v) {
  return _toNumArg(v, "f64 cast operand");
}

/** IEEE float division: never traps (0/0 -> NaN, x/0 -> +-Infinity). */
export function a2x_fdiv(a, b) {
  return _toNumArg(a, "dividend") / _toNumArg(b, "divisor");
}

/** number -> bigint, truncating toward zero (NaN/Infinity trap). */
export function a2x_cast_big(v) {
  if (typeof v === "bigint") return v;
  if (typeof v === "number" && Number.isFinite(v)) {
    return BigInt(Math.trunc(v));
  }
  _trap("Overflow", `cannot cast to bigint: ${_show(v)}`);
}

/** bigint -> number after an explicit range check. */
export function a2x_cast_num_from_big(v, lo, hi) {
  return Number(a2x_bcheck(v, lo, hi));
}

/** number narrowing with an explicit range check. */
export function a2x_cast_num(v, lo, hi) {
  return a2x_check_range(v, lo, hi);
}

// ---------------------------------------------------------------------------
// Text: borrowed `Uint8Array` / owned `A2xOwnedStr`
// ---------------------------------------------------------------------------

const _encoder = new TextEncoder();
const _decoder = new TextDecoder("utf-8", { fatal: true });
const _validatedBorrowed = new WeakSet();
const _literalCache = new Map();

function _decodeUtf8Strict(bytes) {
  try {
    return _decoder.decode(bytes);
  } catch (_e) {
    _trap("InvalidScalar", "invalid UTF-8 sequence");
  }
}

function _asU8(value, what) {
  const u8 = _u8(value);
  if (u8 === null) {
    _trap("InvalidScalar", `${what} is not UTF-8 bytes: ${_show(value)}`);
  }
  return u8;
}

/** Private adapter: accept both text representations as borrowed bytes. */
function _asBytes(value) {
  if (value instanceof A2xOwnedStr) return value.as_bytes();
  const u8 = _asU8(value, "text value");
  if (!_validatedBorrowed.has(u8)) {
    _decodeUtf8Strict(u8);
    _validatedBorrowed.add(u8);
  }
  return u8;
}

/** Byte offsets of every Unicode scalar boundary (input must be valid UTF-8). */
function _scalarOffsets(bytes) {
  const offsets = [0];
  let i = 0;
  while (i < bytes.length) {
    const b = bytes[i];
    let n;
    if (b < 0x80) {
      n = 1;
    } else if ((b & 0xe0) === 0xc0) {
      n = 2;
    } else if ((b & 0xf0) === 0xe0) {
      n = 3;
    } else if ((b & 0xf8) === 0xf0) {
      n = 4;
    } else {
      _trap("InvalidScalar", "invalid UTF-8 lead byte");
    }
    if (i + n > bytes.length) {
      _trap("InvalidScalar", "truncated UTF-8 sequence");
    }
    i += n;
    offsets.push(i);
  }
  return offsets;
}

/** Owned `Str`: a ledger-registered UTF-8 buffer with explicit release. */
export class A2xOwnedStr {
  constructor(bytes) {
    const src = _asU8(bytes, "owned text backing");
    _decodeUtf8Strict(src);
    this.bytes = src.slice();
    this.id = _RT.next_id++;
    this.freed = false;
    _RT.alloc_count += 1;
    _RT.live_bytes += this.bytes.length;
    _RT.live_strs.set(this.id, this);
    if (_RT.events_enabled) {
      _RT.events.push(`str:alloc#${this.id}:${this.bytes.length}`);
    }
  }

  /** Byte length (0 once released). */
  get length() {
    return this.freed ? 0 : this.bytes.length;
  }

  /** Ownership brand: distinguishes owned `Str` from a borrowed view. */
  isOwnedStr() {
    return true;
  }

  /** Borrow the owned bytes (traps after release). */
  as_bytes() {
    if (this.freed) {
      _trap("ResourceClosed", `use-after-free on Str#${this.id}`);
    }
    return this.bytes;
  }

  /** Release the buffer (double release traps `DoubleClose`). */
  explicit_free() {
    if (this.freed) {
      _RT.double_free_count += 1;
      _RT.double_free_strs.push(this.id);
      _trap("DoubleClose", `double free on Str#${this.id}`);
    }
    const n = this.bytes.length;
    this.freed = true;
    this.bytes = A2X_EMPTY_BYTES;
    _RT.live_strs.delete(this.id);
    _RT.freed_strs.push(this.id);
    _RT.free_count += 1;
    _RT.live_bytes = Math.max(0, _RT.live_bytes - n);
    if (_RT.events_enabled) {
      _RT.events.push(`str:free#${this.id}:${n}`);
    }
  }
}

/** Encode a JS string literal to a borrowed (immortal, cached) byte view. */
export function a2x_str_lit(s) {
  if (typeof s === "string") {
    let lit = _literalCache.get(s);
    if (lit === undefined) {
      lit = _encoder.encode(s);
      _literalCache.set(s, lit);
    }
    return lit;
  }
  const u8 = _asU8(s, "text literal");
  _decodeUtf8Strict(u8);
  return u8;
}

/** Decode a text value to a JS string (diagnostics / `fail` details). */
export function a2x_str_utf8(x) {
  return _decodeUtf8Strict(_asBytes(x));
}

/** Unicode lowercase of text, including contextual case mappings. */
export function a2x_str_lower(x) {
  const bytes = _asBytes(x);
  a2x_bill(bytes.length);
  const result = _encoder.encode(_decodeUtf8Strict(bytes).toLowerCase());
  a2x_bill(result.length);
  return new A2xOwnedStr(result);
}

/** Find a text sequence from a Unicode scalar offset. */
export function a2x_str_find(text, pattern, start) {
  const haystack = Array.from(a2x_str_utf8(text));
  const needle = a2x_str_utf8(pattern);
  const offset = _intArg(start, "text search start");
  if (offset < 0 || offset > haystack.length) {
    return -1;
  }
  const tail = haystack.slice(offset).join("");
  const utf16Index = tail.indexOf(needle);
  return utf16Index < 0
    ? -1
    : _checkedI32(offset + Array.from(tail.slice(0, utf16Index)).length, "text search index overflow");
}

/** Unicode scalar (code point) count. */
export function a2x_str_len(x) {
  const s = _decodeUtf8Strict(_asBytes(x));
  return _checkedI32(Array.from(s).length, "text scalar length overflow");
}

/** UTF-8 byte count. */
export function a2x_str_len_bytes(x) {
  return _checkedI32(_asBytes(x).length, "text byte length overflow");
}

/** Unicode scalar at scalar index `i`. */
export function a2x_str_char_at(x, i) {
  const s = _decodeUtf8Strict(_asBytes(x));
  const cps = Array.from(s);
  const idx = _intArg(i, "char index");
  if (idx < 0 || idx >= cps.length) {
    _trap("TextIndex", `char index ${idx} out of bounds ${cps.length}`);
  }
  return cps[idx].codePointAt(0);
}

/** Borrowed byte slice of the scalar range `[start, end)` (copied bytes). */
export function a2x_str_slice(x, start, end) {
  const bytes = _asBytes(x);
  const offsets = _scalarOffsets(bytes);
  const count = offsets.length - 1;
  const st = _intArg(start, "slice start");
  const en = _intArg(end, "slice end");
  if (st < 0 || en < st || en > count) {
    _trap("TextIndex", `slice ${st}..${en} out of bounds ${count}`);
  }
  return bytes.slice(offsets[st], offsets[en]);
}

/** Concatenate two text values into a fresh owned `Str`. */
export function a2x_str_concat(l, r) {
  const left = _asBytes(l);
  const right = _asBytes(r);
  const out = new Uint8Array(left.length + right.length);
  out.set(left, 0);
  out.set(right, left.length);
  return new A2xOwnedStr(out);
}

/** Fresh owned copy of any text value. */
export function a2x_str_clone(x) {
  return new A2xOwnedStr(_asBytes(x));
}

/** Fresh owned copy of any text value (ownership conversion). */
export function a2x_str_to_owned(x) {
  return new A2xOwnedStr(_asBytes(x));
}

/** Borrow an owned or borrowed text value without copying or ledger change. */
export function a2x_str_to_borrowed(x) {
  return _asBytes(x);
}

/** Owned `Str` holding exactly one Unicode scalar. */
export function a2x_str_from_char(c) {
  const cp = _charValue(c);
  return new A2xOwnedStr(_encoder.encode(String.fromCodePoint(cp)));
}

/** Release an owned `Str`; `null`/`undefined` are a no-op. */
export function a2x_str_free(x) {
  if (x === null || x === undefined) return;
  if (x instanceof A2xOwnedStr) {
    x.explicit_free();
    return;
  }
  if (_u8(x) !== null) {
    _trap("DoubleClose", "free of a borrowed text view (ownership misuse)");
  }
  _trap("InvalidScalar", `cannot free a non-text value: ${_show(x)}`);
}

/** Byte equality of two text values. */
export function a2x_str_eq(l, r) {
  const left = _asBytes(l);
  const right = _asBytes(r);
  if (left.length !== right.length) return false;
  for (let i = 0; i < left.length; i += 1) {
    if (left[i] !== right[i]) return false;
  }
  return true;
}

/** Byte inequality of two text values. */
export function a2x_str_ne(l, r) {
  return !a2x_str_eq(l, r);
}

/** Owned `Str` of the decimal representation of a checked i32. */
export function a2x_str_display_i32(v) {
  return new A2xOwnedStr(_encoder.encode(String(_checkedI32(v, "i32 display"))));
}

function _displayF32(value) {
  if (Number.isNaN(value)) return "NaN";
  if (value === Infinity) return "inf";
  if (value === -Infinity) return "-inf";

  const sign = value < 0 || Object.is(value, -0) ? "-" : "";
  const magnitude = Math.abs(value);
  // JavaScript prints an f64 shortest decimal. Keep only the precision
  // needed to round-trip to the checked f32, then write fixed notation.
  let candidate;
  for (let precision = 1; precision <= 9; precision += 1) {
    candidate = magnitude.toExponential(precision - 1);
    const rounded = Number(candidate);
    if (Math.fround(rounded) === magnitude) break;

    // At a binary power the rounding interval can be wider on one
    // side. Check the adjacent decimal towards the original value too.
    const [coefficient, exponent] = candidate.split("e");
    const digits = Number(coefficient.replace(".", ""));
    const place = Number(exponent) - precision + 1;
    const neighbor = rounded < magnitude
      ? `${digits + 1}e${place}`
      : digits === 10 ** (precision - 1)
        ? `${digits * 10 - 1}e${place - 1}`
        : `${digits - 1}e${place}`;
    if (Math.fround(Number(neighbor)) === magnitude) {
      candidate = neighbor;
      break;
    }
  }
  const [mantissa, exponent] = candidate.split("e");
  const separator = mantissa.indexOf(".");
  const position = (separator < 0 ? mantissa.length : separator) + Number(exponent ?? 0);
  const digits = mantissa.replace(".", "").replace(/0+$/, "") || "0";
  if (position <= 0) return sign + "0." + "0".repeat(-position) + digits;
  if (position >= digits.length) return sign + digits + "0".repeat(position - digits.length);
  return sign + digits.slice(0, position) + "." + digits.slice(position);
}

/** Owned Str of the decimal representation of a checked f32. */
export function a2x_str_display_f32(v) {
  return new A2xOwnedStr(_encoder.encode(_displayF32(a2x_f32_round(v))));
}

function _charValue(c) {
  if (typeof c === "string") {
    if (c.length === 0) _trap("InvalidScalar", "empty character literal");
    const cp = c.codePointAt(0);
    return _validateScalar(cp);
  }
  return _validateScalar(_intArg(c, "character"));
}

function _validateScalar(cp) {
  if (typeof cp !== "number" || !Number.isInteger(cp)) {
    _trap("InvalidScalar", `invalid Unicode scalar: ${_show(cp)}`);
  }
  if (cp < 0 || cp > 0x10ffff || (cp >= 0xd800 && cp <= 0xdfff)) {
    _trap("InvalidScalar", `invalid Unicode scalar ${String(cp)}`);
  }
  return cp;
}

/** Character -> Unicode scalar (validated). */
export function a2x_char_to_codepoint(c) {
  return _charValue(c);
}

/** Unicode scalar -> character (validated, no surrogates). */
export function a2x_char_from_codepoint(n) {
  return _validateScalar(_intArg(n, "code point"));
}

// ---------------------------------------------------------------------------
// Collections: List / Variant / cooperative poll latch
// ---------------------------------------------------------------------------

/** Growable list with a checked element kind tag. */
export class A2xList {
  constructor(elemKind = "any") {
    this.items = [];
    this.elemKind = elemKind;
  }
}

/** Structural variant value (one live arm at a time). */
export class A2xVariant {
  constructor(disc, payload = null) {
    this.disc = disc;
    this.payload = payload === undefined ? null : payload;
  }

  get tag() {
    return this.disc;
  }
}

function _list(value) {
  if (value instanceof A2xList) return value;
  _trap("Invariant", `expected a List, got ${_show(value)}`);
}

function _listIndex(i, len, what) {
  const idx = _intArg(i, "list index");
  if (idx < 0 || idx >= len) {
    _trap("Index", `${what} index ${idx} out of bounds ${len}`);
  }
  return idx;
}

/** New list (`elemKind` defaults to `"any"`). */
export function a2x_list_new(elemKind = "any") {
  return new A2xList(elemKind);
}

/** Current list length. */
export function a2x_list_len(l) {
  const list = _list(l);
  return _checkedI32(list.items.length, "list length overflow");
}

/** Append one element. */
export function a2x_list_push(l, v) {
  const list = _list(l);
  // PLAN-118（T-06）：文本是值语义——owned Str 入表存**独立副本**。生成代码
  // 的体清理会对入表用的局部变量发 `a2x_str_free`（`push(it)` 后 `it` 仍被
  // 计入 owned 集），共享同一对象会让表里留下悬垂句柄（实测 paint 视图读
  // 场景列表即 `use-after-free on Str#…`）。
  list.items.push(v instanceof A2xOwnedStr ? new A2xOwnedStr(_asBytes(v)) : v);
}

/** Insert one element at `i` (`0..len`). */
export function a2x_list_insert(l, i, v) {
  const list = _list(l);
  const idx = _intArg(i, "insert index");
  if (idx < 0 || idx > list.items.length) {
    _trap("Index", `insert index ${idx} out of bounds ${list.items.length}`);
  }
  list.items.splice(idx, 0, v);
}

/** Copy one element out at `i`. */
export function a2x_list_get(l, i) {
  const list = _list(l);
  const idx = _listIndex(i, list.items.length, "get");
  return list.items[idx];
}

/** Overwrite one element at `i`. */
export function a2x_list_set(l, i, v) {
  const list = _list(l);
  const idx = _listIndex(i, list.items.length, "set");
  list.items[idx] = v;
}

/** Remove and return one element at `i`. */
export function a2x_list_remove(l, i) {
  const list = _list(l);
  const idx = _listIndex(i, list.items.length, "remove");
  const removed = list.items.splice(idx, 1);
  return removed[0];
}

/** Drop every element (no return value). */
export function a2x_list_clear(l) {
  const list = _list(l);
  list.items.length = 0;
}

/** Move the payload of `disc` out of a variant (destructive projection). */
export function a2x_variant_project(v, disc) {
  if (!(v instanceof A2xVariant)) {
    _trap("Variant", `project on a non-variant value: ${_show(v)}`);
  }
  if (v.disc !== disc) {
    _trap("Variant", `expected arm ${String(disc)}, got ${String(v.disc)}`);
  }
  const payload = v.payload;
  v.disc = -1;
  v.payload = null;
  return payload;
}

/** Cooperative poll latch: no scheduler in this target, so this is a no-op. */
export function a2x_coll_poll() {
  return undefined;
}

// ---------------------------------------------------------------------------
// Trace marks (the emitter may also append its own, byte-identical twins)
// ---------------------------------------------------------------------------

function _mark(suffix, value) {
  _RT.marks.push(`mark_${suffix}`);
  _RT.emit_stderr(`mark_${suffix}\n`);
  return value;
}

/** Trace mark A (pushes `mark_a`, writes it to stderr, returns 1). */
export function a2x_mark_a() {
  return _mark("a", 1);
}

/** Trace mark B (pushes `mark_b`, writes it to stderr, returns 2). */
export function a2x_mark_b() {
  return _mark("b", 2);
}

// ---------------------------------------------------------------------------
// Ports (`io.*` buffer handles + the standard operation catalogue)
// ---------------------------------------------------------------------------

class A2xResourceHandle {
  constructor(initial) {
    this.id = _RT.next_id++;
    this.bytes = _asBytes(initial).slice();
    this.pos = 0;
    this.closed = false;
    this.pending = [];
    _RT.live_handles.add(this.id);
    _RT.handle_trace.push(`open#${this.id}:io.open_buffer`);
  }

  read_line(op_id) {
    if (this.closed) {
      _trap("ResourceClosed", `read on closed handle#${this.id}`, op_id);
    }
    _RT.handle_trace.push(`read#${this.id}:${op_id}`);
    if (this.pos >= this.bytes.length) return undefined;
    let nl = -1;
    for (let i = this.pos; i < this.bytes.length; i += 1) {
      if (this.bytes[i] === 0x0a) {
        nl = i;
        break;
      }
    }
    if (nl < 0) {
      const line = this.bytes.slice(this.pos);
      this.pos = this.bytes.length;
      return line;
    }
    let line = this.bytes.slice(this.pos, nl);
    this.pos = nl + 1;
    if (line.length > 0 && line[line.length - 1] === 0x0d) {
      line = line.slice(0, line.length - 1);
    }
    return line;
  }

  close(op_id) {
    if (this.closed) {
      _trap("DoubleClose", `double close on handle#${this.id}`, op_id);
    }
    this.closed = true;
    this.bytes = A2X_EMPTY_BYTES;
    this.pending = [];
    _RT.live_handles.delete(this.id);
    _RT.closed_handles.add(this.id);
    _RT.handle_trace.push(`close#${this.id}:${op_id}`);
  }
}

const _TEXT_OPS = new Set([
  "text.byte_length",
  "text.equal",
  "text.clone",
  "text.owned",
]);

/**
 * The port catalogue. `version`/`modes` are the frozen contract; an operation
 * that is listed but not implemented traps `PortUnsupported`, and an unknown
 * id or a version/mode/arity mismatch traps `PortContract`.
 */
const _PORT_SPECS = new Map([
  ["console.say", { version: 1, modes: ["View"] }],
  ["text.byte_length", { version: 1, modes: ["View"] }],
  ["text.equal", { version: 1, modes: ["View", "View"] }],
  ["text.clone", { version: 1, modes: ["View"] }],
  ["text.owned", { version: 1, modes: ["View"] }],
  ["math.private_square", { version: 1, modes: ["View"] }],
  ["math.private_cube", { version: 1, modes: ["View"] }],
  ["math.abs", { version: 1, modes: ["View"] }],
  ["math.min", { version: 1, modes: ["View", "View"] }],
  ["math.max", { version: 1, modes: ["View", "View"] }],
  ["math.sqrt", { version: 1, modes: ["View"] }],
  ["math.pow", { version: 1, modes: ["View", "View"] }],
  ["math.floor", { version: 1, modes: ["View"] }],
  ["math.ceil", { version: 1, modes: ["View"] }],
  ["math.trunc", { version: 1, modes: ["View"] }],
  ["math.mod", { version: 1, modes: ["View", "View"] }],
  ["io.stdout", { version: 1, modes: [] }],
  ["io.stderr", { version: 1, modes: [] }],
  ["io.stdin", { version: 1, modes: [] }],
  ["io.console_kind", { version: 1, modes: ["View"] }],
  ["io.write_some_slice", { version: 1, modes: ["View", "View", "View", "View"] }],
  ["io.console_write16", { version: 1, modes: ["View", "View", "View", "View"] }],
  ["io.last_error", { version: 1, modes: [] }],
  ["io.read_line", { version: 1, modes: [] }],
  ["io.read_line_opt", { version: 1, modes: [] }],
  ["io.open_buffer", { version: 1, modes: ["View"] }],
  ["io.read_line_handle", { version: 1, modes: ["Mut"] }],
  ["io.close_handle", { version: 1, modes: ["Move"] }],
  ["io.write_handle", { version: 1, modes: ["Mut", "View"] }],
  ["io.flush_handle", { version: 1, modes: ["Mut"] }],
  ["mem.alloc", { version: 1, modes: ["View"] }],
  ["mem.write_u8", { version: 1, modes: ["View", "View", "View"] }],
  ["mem.read_u8", { version: 1, modes: ["View", "View"] }],
  ["mem.free", { version: 1, modes: ["View"] }],
]);

function _nodeFs() {
  const p = _nodeProcess();
  if (p === null) return null;
  try {
    if (typeof p.getBuiltinModule === "function") {
      return p.getBuiltinModule("node:fs");
    }
  } catch (_e) {
    /* older node or restricted host */
  }
  return null;
}

function _loadStdin() {
  if (_RT.stdin_loaded) return;
  _RT.stdin_loaded = true;
  const fs = _nodeFs();
  if (fs === null) return;
  const chunks = [];
  const scratch = new Uint8Array(65536);
  try {
    for (;;) {
      const n = fs.readSync(0, scratch, 0, scratch.length, null);
      if (n === 0) break;
      chunks.push(scratch.slice(0, n));
    }
  } catch (_e) {
    /* EOF / non-readable stdin */
  }
  if (chunks.length > 0) {
    _RT.stdin_bytes = _concatBytes(chunks);
  }
}

function _nextStdinLine() {
  if (_RT.stdin_lines !== null) {
    if (_RT.stdin_lines.length === 0) return undefined;
    const raw = _RT.stdin_lines.shift();
    return _u8(raw) !== null ? _u8(raw).slice() : _encoder.encode(String(raw));
  }
  _loadStdin();
  if (_RT.stdin_pos >= _RT.stdin_bytes.length) return undefined;
  let nl = -1;
  for (let i = _RT.stdin_pos; i < _RT.stdin_bytes.length; i += 1) {
    if (_RT.stdin_bytes[i] === 0x0a) {
      nl = i;
      break;
    }
  }
  let line;
  if (nl < 0) {
    line = _RT.stdin_bytes.slice(_RT.stdin_pos);
    _RT.stdin_pos = _RT.stdin_bytes.length;
  } else {
    line = _RT.stdin_bytes.slice(_RT.stdin_pos, nl);
    _RT.stdin_pos = nl + 1;
  }
  if (line.length > 0 && line[line.length - 1] === 0x0d) {
    line = line.slice(0, line.length - 1);
  }
  return line;
}

function _normalizeModes(modes) {
  if (typeof modes === "string") return [modes];
  if (modes === null || modes === undefined) return null;
  if (typeof modes.length === "number") {
    const out = [];
    for (let i = 0; i < modes.length; i += 1) out.push(String(modes[i]));
    return out;
  }
  return null;
}

/**
 * Dispatch a standard operation. `args` are the operation parameters in
 * declaration order; the dep/semver/modes contract is validated first.
 */
export function a2x_port_call(op, semver, modes, dep, ...args) {
  const opId = typeof op === "string" ? op : String(op);
  if (dep !== A2X_PORT_DEP) {
    _trap("PortContract", `dependency identity mismatch for ${opId}`, opId);
  }
  const spec = _PORT_SPECS.get(opId);
  if (spec === undefined) {
    _trap("PortContract", `unknown operation ${opId}`, opId);
  }
  const modeList = _normalizeModes(modes);
  if (modeList === null || modeList.length !== spec.modes.length) {
    _trap("PortContract", `contract mismatch for ${opId}`, opId);
  }
  if (semver !== spec.version) {
    _trap("PortContract", `semantic version mismatch for ${opId}`, opId);
  }
  for (let i = 0; i < modeList.length; i += 1) {
    if (modeList[i] !== spec.modes[i]) {
      _trap("PortContract", `contract mismatch for ${opId}`, opId);
    }
  }
  if (args.length !== spec.modes.length) {
    _trap("PortContract", `arity mismatch for ${opId}`, opId);
  }

  if (_TEXT_OPS.has(opId)) {
    if (opId === "text.byte_length") return a2x_str_len_bytes(args[0]);
    if (opId === "text.equal") return a2x_str_eq(args[0], args[1]);
    return a2x_str_clone(args[0]);
  }

  switch (opId) {
    case "console.say": {
      const payload = _asBytes(args[0]);
      const line = new Uint8Array(payload.length + 1);
      line.set(payload, 0);
      line[payload.length] = 0x0a;
      _RT.emit_stdout(line);
      _RT.events.push(`port:console.say:${payload.length}`);
      return undefined;
    }
    case "math.private_square": {
      const x = a2x_check_i32(args[0]);
      return a2x_mul(x, x);
    }
    case "math.private_cube": {
      const x = a2x_check_i32(args[0]);
      return a2x_mul(a2x_mul(x, x), x);
    }
    case "math.abs": {
      const x = args[0];
      if (typeof x === "bigint") return x < 0n ? -x : x;
      const n = _toNumArg(x, "abs operand");
      if (n === A2X_I32_MIN) {
        _trap("Overflow", "abs of i32 MIN", opId);
      }
      return Math.abs(n);
    }
    case "math.min":
    case "math.max": {
      const a = args[0];
      const b = args[1];
      if (typeof a === "bigint" || typeof b === "bigint") {
        const ba = a2x_cast_big(a);
        const bb = a2x_cast_big(b);
        if (opId === "math.min") return ba < bb ? ba : bb;
        return ba > bb ? ba : bb;
      }
      const na = _toNumArg(a, "min/max operand");
      const nb = _toNumArg(b, "min/max operand");
      return opId === "math.min" ? Math.min(na, nb) : Math.max(na, nb);
    }
    case "math.sqrt":
      return Math.sqrt(_toNumArg(args[0], "sqrt operand"));
    case "math.pow":
      return Math.pow(
        _toNumArg(args[0], "pow base"),
        _toNumArg(args[1], "pow exponent"),
      );
    case "math.floor":
      return Math.floor(_toNumArg(args[0], "floor operand"));
    case "math.ceil":
      return Math.ceil(_toNumArg(args[0], "ceil operand"));
    case "math.trunc":
      return Math.trunc(_toNumArg(args[0], "trunc operand"));
    case "math.mod": {
      const a = _toNumArg(args[0], "mod dividend");
      const b = _toNumArg(args[1], "mod divisor");
      if (b === 0) {
        _trap("RemZero", "remainder by zero", opId);
      }
      return _normZero(a - Math.trunc(a / b) * b);
    }
    case "io.stdin":
      return 0;
    case "io.stdout":
      return 1;
    case "io.stderr":
      return 2;
    case "io.last_error":
      return 0;
    case "io.read_line": {
      const line = _nextStdinLine();
      return line === undefined ? A2X_EMPTY_BYTES : line;
    }
    case "io.read_line_opt":
      return _nextStdinLine();
    case "io.open_buffer":
      return new A2xResourceHandle(args[0]);
    case "io.read_line_handle": {
      const handle = args[0];
      if (!(handle instanceof A2xResourceHandle)) {
        _trap("PortContract", "read_line_handle expects a buffer handle", opId);
      }
      return handle.read_line(opId);
    }
    case "io.close_handle": {
      const handle = args[0];
      if (!(handle instanceof A2xResourceHandle)) {
        _trap("PortContract", "close_handle expects a buffer handle", opId);
      }
      handle.close(opId);
      return undefined;
    }
    case "io.write_handle": {
      const handle = args[0];
      const data = _asBytes(args[1]);
      if (handle instanceof A2xResourceHandle) {
        if (handle.closed) {
          _trap("ResourceClosed", `write on closed handle#${handle.id}`, opId);
        }
        handle.pending.push(data.slice());
        return data.length;
      }
      const hn = typeof handle === "number" ? handle : -1;
      if (hn === 1 || hn === 2) {
        if (hn === 1) _RT.emit_stdout(data);
        else _RT.emit_stderr(data);
        return data.length;
      }
      _trap("PortUnsupported", "write_handle needs a buffer handle or 1/2", opId);
      break;
    }
    case "io.flush_handle": {
      const handle = args[0];
      if (handle instanceof A2xResourceHandle) {
        if (handle.closed) {
          _trap("ResourceClosed", `flush on closed handle#${handle.id}`, opId);
        }
        for (const chunk of handle.pending) _RT.emit_stdout(chunk);
        handle.pending = [];
        return undefined;
      }
      const hn = typeof handle === "number" ? handle : -1;
      if (hn === 1 || hn === 2) return undefined;
      _trap("PortUnsupported", "flush_handle needs a buffer handle or 1/2", opId);
      break;
    }
    default:
      break;
  }
  _trap("PortUnsupported", `operation ${opId} is not implemented`, opId);
}

// ---------------------------------------------------------------------------
// Entry driver
// ---------------------------------------------------------------------------

function _valueLine(res) {
  if (typeof res === "boolean") return `value:${res ? 1 : 0}\n`;
  if (typeof res === "bigint") return `value:${res.toString(10)}\n`;
  if (typeof res === "number") {
    if (!Number.isFinite(res) || !Number.isInteger(res)) {
      _trap("Invariant", `entry returned a non-integer number: ${_show(res)}`);
    }
    return `value:${String(res)}\n`;
  }
  if (res === null || res === undefined) return "value:0\n";
  _trap("Invariant", `entry returned an unsupported value: ${_show(res)}`);
}

/**
 * Run the program entry function and emit the harness line:
 *   * `value:<n>\n` on success (boolean -> 0/1, i32 -> decimal, i64 -> decimal);
 *   * `trap:<code>\n` on an `A2xTrap` (plus `port_error:...` on stderr);
 *   * `trap:Internal\n` on any other host exception.
 * Returns 0 on success and 1 on either trap path (never throws).
 */
export function a2x_run_main(entry) {
  a2x_rt_reset();
  if (typeof entry !== "function") {
    _RT.emit_stdout(_encoder.encode("trap:Invariant\n"));
    _RT.flush_stdout();
    return 1;
  }
  try {
    const res = entry();
    _RT.emit_stdout(_encoder.encode(_valueLine(res)));
  } catch (err) {
    if (err instanceof A2xTrap) {
      _RT.emit_stdout(_encoder.encode(`trap:${err.code}\n`));
      if (err.port_id !== null && err.port_id !== undefined) {
        _RT.emit_stderr(`port_error:${err.port_id}:${err.code}:${err.detail}\n`);
      }
    } else {
      const detail = err && err.message ? String(err.message) : String(err);
      _RT.emit_stderr(`internal:${detail}\n`);
      _RT.emit_stdout(_encoder.encode("trap:Internal\n"));
    }
    _RT.flush_stdout();
    _RT.flush_stderr();
    return 1;
  }
  _RT.flush_stdout();
  _RT.flush_stderr();
  return 0;
}
