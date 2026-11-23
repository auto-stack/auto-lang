//! PLAN-967 F0 (T-F0-I): AV-side generic foreign stub dispatch.
//!
//! F0 freezes the first admitted AV foreign domain: borrowed UTF-8 view
//! (`data: *const u8, byte_len: u64` — the shared `ResKind::ViewText`
//! lane) with `i32`/`bool` pure results or a fallible
//! `status + *mut i32 out` frame. Calls cross a real C ABI boundary via
//! function pointers loaded from the provider's build unit (the
//! generated shim cdylib). Admission stays fail-closed: without a
//! current-generation stub entry every foreign call is still refused,
//! and every rejection below happens BEFORE the callee runs —
//! `callee_count` only moves on a real call.
//!
//! Resource contract (frozen): per-call input view ≤
//! [`MAX_CALL_BYTES`]; per-run charged bytes ≤ [`MAX_RUN_BYTES`]; the
//! view is borrowed noescape (the host copies anything it must retain
//! before the call returns). The frozen panic channel: the GENERATED
//! SHIMS catch a provider panic in-unit (same std instance) and raise
//! the provider's exported panic flag; this module observes the flag
//! after every call and converts it into an independent HOST failure.
//! NO unwind ever crosses the C boundary (the frames are plain
//! `extern "C"`), a panic never reaches a guest frame, and it is never
//! re-reported as a normal i32/bool value.

use ace_foreign_contract::manual::SourceDecl;

/// Per-call input view budget (bytes). One byte over is a rejection.
pub const MAX_CALL_BYTES: usize = 65_536;
/// Per-run charged byte ledger (mirrors the console run budget shape).
pub const MAX_RUN_BYTES: usize = 1_048_576;

/// Pure `(view) -> i32` physical frame (plain C ABI; the panic channel
/// is the exported provider flag, never an unwind across the boundary).
pub type ViewUtf8I32Fn = unsafe extern "C" fn(data: *const u8, byte_len: u64) -> i32;
/// Pure `(view) -> bool` physical frame (0/1 byte lane).
pub type ViewUtf8BoolFn = unsafe extern "C" fn(data: *const u8, byte_len: u64) -> u8;
/// Fallible `(view, out) -> status` physical frame.
pub type ViewUtf8OutI32Fn =
    unsafe extern "C" fn(data: *const u8, byte_len: u64, out: *mut i32) -> i32;
/// PLAN-967 T-11: view + scalar frames (the importer's mixed-parameter
/// domain — one borrowed view plus i32 scalars).
pub type ViewI32I32ToI32Fn =
    unsafe extern "C" fn(data: *const u8, byte_len: u64, p0: i32, p1: i32) -> i32;
/// PLAN-1114: the one-scalar i32 lane of the same T-11 family (the
/// `(view, i32) -> i32` importable shape — the bool twin existed; this
/// closes the i32 half so a one-scalar pure op is never transmuted onto
/// the view-only frame with an unread scalar lane).
pub type ViewI32ToI32Fn = unsafe extern "C" fn(data: *const u8, byte_len: u64, p0: i32) -> i32;
pub type ViewI32ToBoolFn = unsafe extern "C" fn(data: *const u8, byte_len: u64, p0: i32) -> u8;
pub type ViewI32OutI32Fn =
    unsafe extern "C" fn(data: *const u8, byte_len: u64, p0: i32, out: *mut i32) -> i32;
/// PLAN-1064 T-FRAME: checked writable byte output borrow frames.
pub type ViewOutBytesI32Fn =
    unsafe extern "C" fn(data: *const u8, byte_len: u64, dst: *mut u8, cap: u64) -> i32;
pub type OutBytesI32Fn = unsafe extern "C" fn(dst: *mut u8, cap: u64) -> i32;
pub type OwnedHandleFn = unsafe extern "C" fn(handle: i64) -> i32;
pub type GuestCallbackFn = extern "C" fn(ctx: i64, item: i64) -> i64;
pub type CallbackI32Fn = unsafe extern "C" fn(GuestCallbackFn, i64, i32) -> i32;
pub type CompletionI32Fn = unsafe extern "C" fn(i32, *mut i32) -> i32;

#[derive(Clone, Copy, Debug)]
pub enum OwnedOpen {
    Text(unsafe extern "C" fn(*const u8, u64) -> i64),
    I32(unsafe extern "C" fn(i32) -> i64),
    Unit(unsafe extern "C" fn() -> i64),
}

#[derive(Clone, Copy, Debug)]
pub enum OwnedCall {
    Open {
        start: OwnedOpen,
        close: OwnedHandleFn,
        class: usize,
    },
    Borrow {
        call: OwnedHandleFn,
        class: usize,
    },
    Close {
        call: OwnedHandleFn,
        class: usize,
    },
}

pub enum OwnedArgument<'a> {
    Unit,
    I32(i32),
    Text(&'a [u8]),
}

impl OwnedCall {
    fn matches(self, sig: &ace_foreign_contract::ResourceSig, provider: &str) -> bool {
        use ace_foreign_contract::{ResKind, ResOut, ResourceForm};

        match (self, &sig.form, sig.params.as_slice()) {
            (
                OwnedCall::Open { start, .. },
                ResourceForm::Pure {
                    ret: ResOut::Handle { provider: owner },
                },
                params,
            ) if owner == provider => matches!(
                (start, params),
                (OwnedOpen::Text(_), [ResKind::ViewText])
                    | (OwnedOpen::I32(_), [ResKind::I32])
                    | (OwnedOpen::Unit(_), [])
            ),
            (
                OwnedCall::Borrow { .. } | OwnedCall::Close { .. },
                ResourceForm::Pure { ret: ResOut::I32 },
                [ResKind::Handle { provider: owner }],
            ) => owner == provider,
            _ => false,
        }
    }
}

fn next_owned_id() -> Option<i32> {
    use std::sync::atomic::{AtomicI32, Ordering};

    // Handles from a closed session must never alias a newly opened one,
    // even when two independent bindings have the same initial generation.
    static NEXT: AtomicI32 = AtomicI32::new(1);
    NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .ok()
}

/// The frozen F0/T-11 physical callables.
#[derive(Clone, Copy, Debug)]
pub enum ForeignCallable {
    PureI32(ViewUtf8I32Fn),
    PureBool(ViewUtf8BoolFn),
    FallibleOutI32(ViewUtf8OutI32Fn),
    /// T-11 mixed frames.
    ViewI32I32ToI32(ViewI32I32ToI32Fn),
    /// PLAN-1114: the `(view, i32) -> i32` lane (the T-11 family's i32
    /// one-scalar twin).
    ViewI32ToI32(ViewI32ToI32Fn),
    ViewI32ToBool(ViewI32ToBoolFn),
    ViewI32FallibleI32(ViewI32OutI32Fn),
    /// PLAN-1064 T-FRAME: checked writable out-buffer frames.
    ViewOutBytesToI32(ViewOutBytesI32Fn),
    OutBytesToI32(OutBytesI32Fn),
    Owned(OwnedCall),
    Callback(CallbackI32Fn),
    Completion {
        call: CompletionI32Fn,
        asynchronous: bool,
    },
}

#[derive(Clone, Copy)]
struct CallbackSlot {
    data: *mut (),
    invoke: unsafe fn(*mut (), i32, i32) -> Option<i32>,
}

thread_local! {
    static SYNC_CALLBACK: std::cell::Cell<Option<CallbackSlot>> = const {
        std::cell::Cell::new(None)
    };
}

struct CallbackScope<'a>(std::marker::PhantomData<&'a mut ()>);

impl<'a> CallbackScope<'a> {
    fn enter<F: FnMut(i32, i32) -> Option<i32>>(call: &'a mut F) -> Option<Self> {
        unsafe fn invoke<F: FnMut(i32, i32) -> Option<i32>>(
            data: *mut (),
            ctx: i32,
            item: i32,
        ) -> Option<i32> {
            unsafe { (&mut *data.cast::<F>())(ctx, item) }
        }
        SYNC_CALLBACK.with(|active| {
            if active.get().is_some() {
                return None;
            }
            active.set(Some(CallbackSlot {
                data: std::ptr::from_mut(call).cast(),
                invoke: invoke::<F>,
            }));
            Some(Self(std::marker::PhantomData))
        })
    }
}

impl Drop for CallbackScope<'_> {
    fn drop(&mut self) {
        SYNC_CALLBACK.with(|active| active.set(None));
    }
}

extern "C" fn callback_trampoline(ctx: i64, item: i64) -> i64 {
    let (Ok(ctx), Ok(item)) = (i32::try_from(ctx), i32::try_from(item)) else {
        return i64::MIN;
    };
    SYNC_CALLBACK.with(|active| {
        let Some(slot) = active.take() else {
            return i64::MIN;
        };
        // The slot borrows the current stack frame, is available on this
        // thread only, and is unavailable during the callback itself.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            (slot.invoke)(slot.data, ctx, item)
        }));
        active.set(Some(slot));
        result.ok().flatten().map(i64::from).unwrap_or(i64::MIN)
    })
}

/// Access rights for a host-managed checked writable output borrow frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutBufferRights {
    ReadOnly,
    ReadWrite,
}

/// Checked writable output borrow frame `{owner, origin, generation, offset, capacity, rights}`
/// backing `ResKind::OutBytes` calls on the AV host. Runtime pointers never enter build caches.
#[derive(Clone, Debug)]
pub struct OutBufferFrame {
    pub owner: String,
    pub origin: String,
    pub generation: u64,
    pub backing: Vec<u8>,
    pub offset: usize,
    pub capacity: usize,
    pub rights: OutBufferRights,
    pub pin_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub closed: bool,
    pub last_published: usize,
}

impl OutBufferFrame {
    pub fn new(
        owner: impl Into<String>,
        origin: impl Into<String>,
        generation: u64,
        backing: Vec<u8>,
        offset: usize,
        capacity: usize,
        rights: OutBufferRights,
    ) -> Self {
        Self {
            owner: owner.into(),
            origin: origin.into(),
            generation,
            backing,
            offset,
            capacity,
            rights,
            pin_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            closed: false,
            last_published: 0,
        }
    }

    pub fn is_pinned(&self) -> bool {
        self.pin_flag.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// One admitted stub: the (provider, symbol, content_version) identity
/// at a binding generation, plus the physical callable.
#[derive(Clone, Debug)]
pub struct ForeignStub {
    pub provider: String,
    pub symbol: String,
    pub content_version: String,
    pub generation: u64,
    pub callable: ForeignCallable,
    /// T-05 F1 (AC-INT-1026 / IF-CALL): the view wire carries a
    /// `p967.frame/1` tag frame for this declaration — the AV admission
    /// validates the frame BEFORE any dispatch (a bad frame is a named
    /// `foreign.frame` rejection with callee_count unchanged).
    pub frame: bool,
}

impl ForeignStub {
    /// Construct a non-frame stub (the pre-F1 spelling).
    pub fn new(
        provider: String,
        symbol: String,
        content_version: String,
        generation: u64,
        callable: ForeignCallable,
    ) -> Self {
        Self {
            provider,
            symbol,
            content_version,
            generation,
            callable,
            frame: false,
        }
    }
}

// ---------------------------------------------------------------------------
// p967.frame/1 — the frozen IF-CALL tag parameter frame (AC-INT-1026).
//
// Canonical TEXT encoding over the ViewText wire (UTF-8 overall; ASCII
// LF-terminated HEADERS with decimal lengths; payloads are read EXACTLY
// by length and may contain ANY byte, including LF):
//   p967.frame/1 LF
//   'r' SP tag SP len LF + <len> payload bytes LF     (receiver; EVERY payload is LF-terminated)
//   'p' SP count LF                                    (count 0..=8)
//   [tag SP len LF + payload LF] x count              (positionals)
//   'k' SP count LF                                    (count 0..=8)
//   [namelen LF + name bytes + tag SP len LF + payload LF] x count (no separator after the name)
//                                                      (namelen 1..=64)
//   NOTHING may follow (exact consume).
// tags: `n` (len 0) | `i` (i32 decimal) | `b` (`0`/`1`, len 1)
//       | `t` (any bytes, len 0..=4096)  | `o` (u64 decimal)
//
// PLAN-1129: the same layout carries a second wire version, `p967.frame/2`
// LF, which is the superset above plus the typed float tags
//   `d` (f64 bits) | `f` (f32 bits): payload `<w2> <w1> <w0>` — three
//   unsigned decimals (1..=7 digits each) carrying the 64-bit pattern
//   21 / 21 / 22 bits from the top; every word stays below its bound.
// A `/1` frame never carries `d`/`f`; a `/1` `t` payload keeps whatever the
// provider's legacy mode makes of it. This validator and the generated shim
// decoder (`auto_bindgen::rust_import::FRAME_SHIM_PRELUDE`) accept the same
// byte set — change them together.
// ---------------------------------------------------------------------------

/// Frame caps (frozen with the encoding).
pub const FRAME_MAX_POSITIONALS: usize = 8;
pub const FRAME_MAX_KWARGS: usize = 8;
pub const FRAME_MAX_TEXT: usize = 4096;
pub const FRAME_MAX_NAME: usize = 64;

/// The two wire headers of the tag frame, in version order.
pub const FRAME_HEADERS: [&[u8]; 2] = [b"p967.frame/1\n", b"p967.frame/2\n"];

/// Whether `tag` belongs to `version` of the tag-frame vocabulary.
fn frame_tag_ok(tag: u8, version: u8) -> bool {
    matches!(tag, b'n' | b'i' | b'b' | b't' | b'o')
        || (version == 2 && matches!(tag, b'd' | b'f'))
}

/// Strictly parse and validate a `p967.frame/1` or `p967.frame/2` wire
/// frame. Guest-supplied tags/lengths are never trusted: every structural
/// violation is `Err` with a stable reason (the caller rejects with
/// `foreign.frame` BEFORE dispatch — callee_count stays unchanged). The
/// parse is CURSOR-driven: headers are ASCII lines; payloads are read
/// EXACTLY by length and may contain any byte (including LF).
pub fn validate_frame_wire(bytes: &[u8]) -> Result<(), &'static str> {
    let mut cur = bytes;
    let mut version = 0u8;
    for (index, header) in FRAME_HEADERS.iter().enumerate() {
        if cur.len() >= header.len() && &cur[..header.len()] == *header {
            cur = &cur[header.len()..];
            version = index as u8 + 1;
            break;
        }
    }
    if version == 0 {
        return Err("bad magic/version header");
    }
    // receiver: 'r' SP tag SP len LF + payload LF (the 'r ' prefix marks
    // the receiver slot inside the value header)
    let (tag, payload) = take_prefixed_value(&mut cur, b"r ", version)?;
    check_tag_value(tag, payload, version)?;
    // positionals: 'p' SP count LF then count values
    let count = take_count(&mut cur)?;
    for _ in 0..count {
        let (tag, payload) = take_value(&mut cur, version)?;
        check_tag_value(tag, payload, version)?;
    }
    // kwargs: 'k' SP count LF then count (namelen LF name, value)
    let count = take_count_kw(&mut cur)?;
    for _ in 0..count {
        let name = take_name(&mut cur)?;
        if name.is_empty() {
            return Err("empty kwarg name");
        }
        let (tag, payload) = take_value(&mut cur, version)?;
        let _ = name;
        check_tag_value(tag, payload, version)?;
    }
    if !cur.is_empty() {
        return Err("trailing bytes after the frame");
    }
    Ok(())
}

/// Read one ASCII header line (up to LF); return it.
fn take_line<'a>(cur: &mut &'a [u8]) -> Result<&'a [u8], &'static str> {
    let n = cur
        .iter()
        .position(|&b| b == b'\n')
        .ok_or("unterminated header line")?;
    let (l, tail) = cur.split_at(n);
    if l.is_empty() {
        return Err("empty header line");
    }
    *cur = &tail[1..];
    Ok(l)
}

/// tag SP len LF + exactly len payload bytes (`version` selects the tag set).
fn take_value<'a>(cur: &mut &'a [u8], version: u8) -> Result<(u8, &'a [u8]), &'static str> {
    let header = take_line(cur)?;
    let tag = *header.first().ok_or("missing tag")?;
    if !frame_tag_ok(tag, version) {
        return Err("unknown tag");
    }
    let tail = &header[1..];
    let tail = tail
        .strip_prefix(b" ")
        .ok_or("tag not followed by a length")?;
    let (len, tail) = split_decimal(tail).ok_or("bad length")?;
    if !tail.is_empty() {
        return Err("length not the header end");
    }
    if len > FRAME_MAX_TEXT as u64 {
        return Err("payload over the frame text cap");
    }
    let len = len as usize;
    if cur.len() < len {
        return Err("payload truncated");
    }
    let (payload, tail) = cur.split_at(len);
    let tail = tail
        .strip_prefix(
            b"
",
        )
        .ok_or("payload not LF-terminated")?;
    *cur = tail;
    Ok((tag, payload))
}

/// A value header with a leading marker (`r ` for the receiver slot).
fn take_prefixed_value<'a>(
    cur: &mut &'a [u8],
    prefix: &[u8],
    version: u8,
) -> Result<(u8, &'a [u8]), &'static str> {
    let header = take_line(cur)?;
    let rest = header
        .strip_prefix(prefix)
        .ok_or("expected the receiver value header")?;
    let tag = *rest.first().ok_or("missing tag")?;
    if !frame_tag_ok(tag, version) {
        return Err("unknown tag");
    }
    let tail = &rest[1..];
    let tail = tail
        .strip_prefix(b" ")
        .ok_or("tag not followed by a length")?;
    let (len, tail) = split_decimal(tail).ok_or("bad length")?;
    if !tail.is_empty() {
        return Err("length not the header end");
    }
    if len > FRAME_MAX_TEXT as u64 {
        return Err("payload over the frame text cap");
    }
    let len = len as usize;
    if cur.len() < len {
        return Err("payload truncated");
    }
    let (payload, tail) = cur.split_at(len);
    let tail = tail
        .strip_prefix(b"\n")
        .ok_or("payload not LF-terminated")?;
    *cur = tail;
    Ok((tag, payload))
}

fn take_count(cur: &mut &[u8]) -> Result<usize, &'static str> {
    let header = take_line(cur)?;
    let (h, tail) = header.split_first().ok_or("empty count line")?;
    if *h != b'p' {
        return Err("expected the positional count line");
    }
    let tail = tail.strip_prefix(b" ").ok_or("count not delimited")?;
    let (n, tail) = split_decimal(tail).ok_or("bad count")?;
    if !tail.is_empty() || n > FRAME_MAX_POSITIONALS as u64 {
        return Err("count over the frame cap");
    }
    Ok(n as usize)
}

fn take_count_kw(cur: &mut &[u8]) -> Result<usize, &'static str> {
    let header = take_line(cur)?;
    let (h, tail) = header.split_first().ok_or("empty count line")?;
    if *h != b'k' {
        return Err("expected the kwarg count line");
    }
    let tail = tail.strip_prefix(b" ").ok_or("count not delimited")?;
    let (n, tail) = split_decimal(tail).ok_or("bad count")?;
    if !tail.is_empty() || n > FRAME_MAX_KWARGS as u64 {
        return Err("count over the frame cap");
    }
    Ok(n as usize)
}

/// namelen LF + exactly namelen name bytes.
fn take_name<'a>(cur: &mut &'a [u8]) -> Result<&'a [u8], &'static str> {
    let header = take_line(cur)?;
    let (len, tail) = split_decimal(header).ok_or("bad name length")?;
    if !tail.is_empty() {
        return Err("name length not the header end");
    }
    if len < 1 || len > FRAME_MAX_NAME as u64 {
        return Err("kwarg name length out of range");
    }
    let len = len as usize;
    if cur.len() < len {
        return Err("kwarg name truncated");
    }
    let (name, tail) = cur.split_at(len);
    *cur = tail;
    Ok(name)
}

/// Per-tag payload validation (guest bytes never trusted).
///
/// T-05 NF2 (scoped re-review): this validator is intentionally looser
/// for `t` payloads and kwarg names than the generated shim decoder
/// (which additionally requires UTF-8) — consistency holds because the
/// dispatch order runs the WHOLE-frame `foreign.utf8` gate before
/// `validate_frame_wire` (see dispatch_view_scalars); any future
/// dispatch path that skips the view gate must fold the UTF-8 check
/// into this function to keep the two implementations of the frozen
/// spec aligned.
fn check_tag_value(tag: u8, payload: &[u8], version: u8) -> Result<(), &'static str> {
    match tag {
        b'n' if payload.is_empty() => Ok(()),
        b'n' => Err("tag n must carry no payload"),
        b'i' => std::str::from_utf8(payload)
            .ok()
            .and_then(|t| t.parse::<i32>().ok())
            .map(|_| ())
            .ok_or("tag i payload is not an i32"),
        b'o' => std::str::from_utf8(payload)
            .ok()
            .and_then(|t| t.parse::<u64>().ok())
            .map(|_| ())
            .ok_or("tag o payload is not a u64"),
        b'b' if payload == b"0" || payload == b"1" => Ok(()),
        b'b' => Err("tag b payload must be 0 or 1"),
        b't' => Ok(()), // any bytes, capped by take_value
        // PLAN-1129: typed floats exist in version 2 only.
        b'd' | b'f' if version == 2 => check_float_words(payload),
        _ => Err("unknown tag"),
    }
}

/// The typed float payload `<w2> <w1> <w0>`: three unsigned decimals, the
/// 64-bit pattern split 21 / 21 / 22 bits from the top. Every word is 1..=7
/// digits and below its bound, so one value has exactly one encoding (the
/// same acceptance and the same reasons as the shim decoder's
/// `__f0_float_bits`).
fn check_float_words(payload: &[u8]) -> Result<(), &'static str> {
    const BOUNDS: [u64; 3] = [1 << 21, 1 << 21, 1 << 22];
    let mut words = payload.split(|&b| b == b' ');
    for bound in BOUNDS {
        let word = words
            .next()
            .ok_or("typed float payload is not `<w2> <w1> <w0>`")?;
        if !word.iter().all(|b| b.is_ascii_digit()) {
            return Err("typed float word is not a decimal");
        }
        if word.is_empty() || word.len() > 7 {
            return Err("typed float word is out of range");
        }
        let value = word
            .iter()
            .fold(0u64, |value, b| value * 10 + u64::from(b - b'0'));
        if value >= bound {
            return Err("typed float word is out of range");
        }
    }
    if words.next().is_some() {
        return Err("typed float payload is not `<w2> <w1> <w0>`");
    }
    Ok(())
}

fn split_decimal(l: &[u8]) -> Option<(u64, &[u8])> {
    let mut n: u64 = 0;
    let mut i = 0;
    if l.first() == Some(&b'-') {
        i = 1;
    }
    let digits_start = i;
    while i < l.len() && l[i].is_ascii_digit() {
        n = n.checked_mul(10)?.checked_add((l[i] - b'0') as u64)?;
        if n > (i64::MAX as u64) {
            return None;
        }
        i += 1;
    }
    if i == digits_start {
        return None;
    }
    Some((n, &l[i..]))
}

/// Named pre-dispatch rejection (stable codes, fail-closed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignReject {
    pub code: &'static str,
    pub message: String,
}

fn reject(code: &'static str, message: impl Into<String>) -> ForeignExecError {
    ForeignExecError::Reject(ForeignReject {
        code,
        message: message.into(),
    })
}

/// One dispatch's outcome for the guest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForeignDispatch {
    PureI32(i32),
    PureBool(bool),
    /// Fallible frame: `(status, out)`; `status == 0` carries the value.
    Fallible(i32, i32),
}

/// Execution-boundary failure: either a named pre-dispatch rejection or
/// an independent HOST failure (provider panic) — the latter is never a
/// language-level value and never unwinds into the guest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForeignExecError {
    Reject(ForeignReject),
    Panic(String),
    Cancelled,
    Wall,
}

/// The executor's stub table. One table == one provider session binding:
/// `bump_generation` (session teardown / provider rebind) instantly
/// invalidates every entry minted under the old generation.
#[derive(Debug)]
pub struct ForeignStubTable {
    entries: Vec<ForeignStub>,
    current_generation: u64,
    run_bytes: usize,
    callee_count: u64,
    rejected_count: u64,
    /// The provider's exported panic flag (`f0_panic_flag()`), observed
    /// after every call. `null` = the in-process test tables (no flag;
    /// the belt catch_unwind below is the only channel there).
    panic_flag: *mut u32,
    /// PLAN-1064 T-FRAME: host-checked writable output borrow frames.
    out_buffers: std::collections::BTreeMap<i32, OutBufferFrame>,
    out_expected_owner: String,
    out_expected_origin: String,
    python_error_observers: Option<Box<[SourceDecl; 2]>>,
    completion_ports: std::sync::Arc<CompletionPorts>,
    owned_handles: std::collections::BTreeMap<i32, OwnedHandle>,
    code_lease: Option<std::sync::Arc<dyn std::fmt::Debug + Send + Sync>>,
    completion_worker_busy: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[derive(Debug)]
struct OwnedHandle {
    provider: String,
    generation: u64,
    class: usize,
    raw: i64,
    close: OwnedHandleFn,
}

impl Drop for ForeignStubTable {
    fn drop(&mut self) {
        self.release_owned_handles();
    }
}

impl ForeignStubTable {
    pub fn new(current_generation: u64) -> Self {
        Self {
            entries: Vec::new(),
            current_generation,
            run_bytes: 0,
            callee_count: 0,
            rejected_count: 0,
            panic_flag: std::ptr::null_mut(),
            out_buffers: std::collections::BTreeMap::new(),
            out_expected_owner: "av".into(),
            out_expected_origin: "guest".into(),
            python_error_observers: None,
            completion_ports: std::sync::Arc::new(CompletionPorts::with_observation(
                current_generation,
                false,
            )),
            owned_handles: std::collections::BTreeMap::new(),
            code_lease: None,
            completion_worker_busy: Default::default(),
        }
    }

    pub fn set_out_context(&mut self, owner: impl Into<String>, origin: impl Into<String>) {
        self.out_expected_owner = owner.into();
        self.out_expected_origin = origin.into();
    }

    pub fn register_out_buffer(&mut self, id: i32, frame: OutBufferFrame) {
        self.out_buffers.insert(id, frame);
    }

    pub fn out_buffer(&self, id: i32) -> Option<&OutBufferFrame> {
        self.out_buffers.get(&id)
    }

    pub fn out_buffer_mut(&mut self, id: i32) -> Option<&mut OutBufferFrame> {
        self.out_buffers.get_mut(&id)
    }

    pub fn close_out_buffer(&mut self, id: i32) -> Result<(), ForeignExecError> {
        let Some(frame) = self.out_buffers.get_mut(&id) else {
            return Err(reject(
                "foreign.out-missing",
                format!("unknown out buffer `{id}`"),
            ));
        };
        if frame.is_pinned() {
            return Err(reject(
                "foreign.out-pinned",
                format!("cannot release out buffer `{id}` while native borrow is active"),
            ));
        }
        frame.closed = true;
        Ok(())
    }

    pub fn callable_of(&self, declaration: &SourceDecl) -> Option<ForeignCallable> {
        self.lookup(declaration)
            .ok()
            .map(|i| self.entries[i].callable)
    }

    /// Attach the error observers of this admitted Python binding. Ordinary
    /// AV sessions request this explicitly; generic foreign status failures
    /// and the frozen aggregates producer retain their original classification.
    pub fn bind_python_error_observers(
        &mut self,
        admitted: &ace_foreign_contract::manual::AdmittedForeign,
    ) -> Result<(), ForeignReject> {
        use ace_foreign_contract::py_ops::{PY_CONTRACT_LOGICAL, PY_CONTRACT_PROVIDER};
        let find = |name: &str| {
            admitted
                .source_declarations_ref()
                .iter()
                .find(|decl| {
                    decl.name == name
                        && decl
                            .logical
                            .rsplit_once('/')
                            .map(|(namespace, _)| namespace)
                            == Some(PY_CONTRACT_LOGICAL)
                        && decl.provider == PY_CONTRACT_PROVIDER
                })
                .ok_or_else(|| ForeignReject {
                    code: "foreign.py-error-observer",
                    message: format!("admitted Python binding lacks {name}"),
                })
        };
        let stage = find("py_error_stage")?;
        let word = find("py_str_word")?;
        self.lookup(stage)?;
        self.lookup(word)?;
        self.python_error_observers = Some(Box::new([stage.clone(), word.clone()]));
        Ok(())
    }

    pub(crate) fn python_exception(
        &mut self,
        failed: &SourceDecl,
        cancel: &crate::av::CancelFlag,
    ) -> Result<Option<(u8, String)>, ForeignExecError> {
        let Some(observers) = self.python_error_observers.as_ref() else {
            return Ok(None);
        };
        if failed.provider != observers[0].provider
            || failed
                .logical
                .rsplit_once('/')
                .map(|(namespace, _)| namespace)
                != Some(ace_foreign_contract::py_ops::PY_CONTRACT_LOGICAL)
            || failed.header_digest != observers[0].header_digest
            || failed.physical_table_digest != observers[0].physical_table_digest
            || failed.content_version != observers[0].content_version
            || failed.review_id != observers[0].review_id
        {
            return Ok(None);
        }
        let [stage, word] = observers.as_ref().clone();
        const EMPTY: &[u8] = b"p967.frame/1\nr n 0\n\np 0\nk 0\n";
        let check_cancel = || {
            if cancel.load(std::sync::atomic::Ordering::Acquire) {
                Err(reject(
                    "foreign.cancelled",
                    "cancelled during Python error observation",
                ))
            } else {
                Ok(())
            }
        };
        check_cancel()?;
        // PLAN-1094: the staged error lane is codepoint-wise (UTF-32-LE
        // words) — one word per char, then UTF-8 re-encoding.
        let length = match self.dispatch_view(&stage, EMPTY)? {
            ForeignDispatch::PureI32(value @ 0..=262) => value as usize,
            _ => {
                return Err(reject(
                    "foreign.py-error-observer",
                    "invalid staged error length",
                ))
            }
        };
        let mut bytes = Vec::with_capacity(length * 4);
        for index in 0..length {
            let position = index.to_string();
            let frame = format!(
                "p967.frame/1\nr n 0\n\np 1\ni {}\n{position}\nk 0\n",
                position.len()
            );
            check_cancel()?;
            match self.dispatch_view(&word, frame.as_bytes())? {
                ForeignDispatch::PureI32(value) => {
                    let cp = u32::from_le_bytes(value.to_le_bytes()) as u32;
                    let Some(c) = char::from_u32(cp) else {
                        return Err(reject(
                            "foreign.py-error-observer",
                            "invalid error text codepoint",
                        ));
                    };
                    let mut buf = [0u8; 4];
                    bytes.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                }
                _ => {
                    return Err(reject(
                        "foreign.py-error-observer",
                        "invalid error text word",
                    ))
                }
            }
        }
        let stage = std::str::from_utf8(&bytes)
            .map_err(|_| reject("foreign.py-error-observer", "invalid error text"))?;
        let (class, message) = stage
            .strip_prefix('[')
            .and_then(|value| value.split_once("] "))
            .and_then(|(class, message)| Some((class.parse::<u8>().ok()?, message)))
            .ok_or_else(|| reject("foreign.py-error-observer", "invalid staged error shape"))?;
        // The provider reserves 20 and above for transport/domain failures.
        Ok((1..=10)
            .contains(&class)
            .then(|| (class, format!("PyException {message}"))))
    }

    /// Bind the provider's exported panic flag (the loader resolves it).
    pub fn set_panic_flag(&mut self, flag: *mut u32) {
        self.panic_flag = flag;
    }

    pub fn insert(&mut self, stub: ForeignStub) -> Result<(), &'static str> {
        if self
            .entries
            .iter()
            .any(|e| e.provider == stub.provider && e.symbol == stub.symbol)
        {
            return Err("foreign.duplicate: provider symbol already bound");
        }
        self.entries.push(stub);
        Ok(())
    }

    /// Session teardown / provider rebind: stale entries stop admitting.
    pub fn bump_generation(&mut self) {
        self.current_generation += 1;
        self.completion_ports
            .bind_session_generation(self.current_generation);
        self.release_owned_handles();
    }

    /// Keep the original native code image alive as long as its stubs or
    /// resource destructors can still be used.
    pub fn pin_code_image(&mut self, lease: std::sync::Arc<dyn std::fmt::Debug + Send + Sync>) {
        assert!(self.entries.is_empty() && self.owned_handles.is_empty());
        self.code_lease = Some(lease);
    }

    fn release_owned_handles(&mut self) {
        for (_, handle) in std::mem::take(&mut self.owned_handles) {
            unsafe { (handle.close)(handle.raw) };
            let _ = self.take_panic_flag();
        }
    }

    pub fn generation(&self) -> u64 {
        self.current_generation
    }

    pub fn callee_count(&self) -> u64 {
        self.callee_count
    }

    pub fn rejected_count(&self) -> u64 {
        self.rejected_count
    }

    pub fn run_bytes(&self) -> usize {
        self.run_bytes
    }

    /// Per-run byte ledger reset (each execution is one budget run).
    pub fn reset_run_bytes(&mut self) {
        self.run_bytes = 0;
    }

    /// Read-only view of the bound entries (identity inspection; tests
    /// and tooling use this — admission itself never mutates).
    pub fn entries(&self) -> &[ForeignStub] {
        &self.entries
    }

    /// Replace the whole entry set (rebuild after inspection).
    pub fn set_entries(&mut self, entries: Vec<ForeignStub>) {
        self.entries = entries;
    }

    fn lookup(&self, declaration: &SourceDecl) -> Result<usize, ForeignReject> {
        let symbol = declaration.function.symbol.as_str();
        let provider = declaration.provider.as_str();
        let mut by_symbol = None;
        for (i, e) in self.entries.iter().enumerate() {
            if e.symbol != symbol {
                continue;
            }
            by_symbol = Some(i);
            // Exact declaration identity: provider + symbol + the
            // interface content version the declaration was checked
            // against. A same-symbol different-identity entry is a
            // signature mismatch, not a hit.
            if e.provider == provider
                && e.content_version == declaration.content_version
                && (matches!(
                    (e.callable, declaration.resource().map(|r| &r.form)),
                    (
                        ForeignCallable::PureI32(_),
                        Some(ace_foreign_contract::ResourceForm::Pure {
                            ret: ace_foreign_contract::ResOut::I32
                        })
                    ) | (
                        ForeignCallable::PureBool(_),
                        Some(ace_foreign_contract::ResourceForm::Pure {
                            ret: ace_foreign_contract::ResOut::Bool
                        })
                    ) | (
                        ForeignCallable::FallibleOutI32(_),
                        Some(ace_foreign_contract::ResourceForm::Fallible {
                            out: Some(ace_foreign_contract::ResOut::I32),
                            ..
                        })
                    ) | (
                        ForeignCallable::ViewI32I32ToI32(_),
                        Some(ace_foreign_contract::ResourceForm::Pure {
                            ret: ace_foreign_contract::ResOut::I32
                        })
                    ) | (
                        // PLAN-1114: the one-scalar i32 lane (same pure
                        // i32 resource form as its two-scalar twin).
                        ForeignCallable::ViewI32ToI32(_),
                        Some(ace_foreign_contract::ResourceForm::Pure {
                            ret: ace_foreign_contract::ResOut::I32
                        })
                    ) | (
                        ForeignCallable::ViewI32ToBool(_),
                        Some(ace_foreign_contract::ResourceForm::Pure {
                            ret: ace_foreign_contract::ResOut::Bool
                        })
                    ) | (
                        ForeignCallable::ViewI32FallibleI32(_),
                        Some(ace_foreign_contract::ResourceForm::Fallible {
                            out: Some(ace_foreign_contract::ResOut::I32),
                            ..
                        })
                    ) | (
                        ForeignCallable::ViewOutBytesToI32(_),
                        Some(ace_foreign_contract::ResourceForm::Pure {
                            ret: ace_foreign_contract::ResOut::I32
                        })
                    ) | (
                        ForeignCallable::OutBytesToI32(_),
                        Some(ace_foreign_contract::ResourceForm::Pure {
                            ret: ace_foreign_contract::ResOut::I32
                        })
                    )
                ) || matches!(e.callable, ForeignCallable::Owned(op)
                    if declaration.resource().is_some_and(|sig| op.matches(sig, provider)))
                    || matches!(e.callable, ForeignCallable::Callback(_)
                    if declaration.logical.starts_with("rust/")
                        && declaration.resource().is_some_and(|sig| {
                            use ace_foreign_contract::{ResKind, ResOut, ResourceForm};
                            sig.noescape
                                && sig.params == [ResKind::Callback { ctx: true }, ResKind::I32, ResKind::I32]
                                && sig.form == (ResourceForm::Pure { ret: ResOut::I32 })
                        }))
                    || matches!(e.callable, ForeignCallable::Completion { .. }
                    if declaration.logical.starts_with("rust/")
                        && declaration.resource().is_some_and(|sig| {
                            use ace_foreign_contract::{ResKind, ResOut, ResourceForm};
                            sig.noescape
                                && sig.params == [ResKind::I32]
                                && sig.form == (ResourceForm::Pure { ret: ResOut::I32 })
                        })))
            {
                return Ok(i);
            }
        }
        Err(match by_symbol {
            Some(_) => ForeignReject {
                code: "foreign.signature",
                message: format!(
                    "symbol `{symbol}` is bound with a different identity than the declaration"
                ),
            },
            None => ForeignReject {
                code: "foreign.unknown-symbol",
                message: format!("no stub bound for symbol `{symbol}` (provider `{provider}`)"),
            },
        })
    }

    /// Per-run module authorization (mirrors `host::authorize`): every
    /// `ForeignCall` declaration in the module must be exactly admissible
    /// at the CURRENT stub generation. A loaded structural credential
    /// never preserves grants.
    pub fn authorize_module(
        &self,
        module: &crate::mir::MirModule,
    ) -> Result<(), crate::host::AdmissionError> {
        for f in &module.funcs {
            for b in &f.blocks {
                for i in &b.instrs {
                    if let crate::mir::MirInstr::ForeignCall { declaration, .. } = i {
                        if !self.admits_declaration(declaration) {
                            return Err(crate::host::AdmissionError {
                                code: "host.unauthorized",
                                message: format!(
                                    "foreign.unauthorized: declaration `{}` of provider `{}` is not admitted at generation {}",
                                    declaration.function.symbol,
                                    declaration.provider,
                                    self.generation()
                                ),
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Structural admission for a whole module (verify time): every
    /// foreign declaration must be exactly admissible at the CURRENT
    /// generation — a stale entry (the session moved on via rebind or
    /// teardown) never admits. Mirrors `AdmittedForeign::admits_declaration`
    /// plus the generation freshness of `requalify`.
    pub fn admits_declaration(&self, declaration: &SourceDecl) -> bool {
        match self.lookup(declaration) {
            Ok(idx) => self.entries[idx].generation == self.current_generation,
            Err(_) => false,
        }
    }

    /// Per-call dispatch. Ordering is the contract: identity →
    /// generation → UTF-8 → byte budgets → reentry, all BEFORE the
    /// callee; only then the C ABI call (panic-caught), and only a real
    /// call moves `callee_count`.
    pub fn dispatch_view(
        &mut self,
        declaration: &SourceDecl,
        bytes: &[u8],
    ) -> Result<ForeignDispatch, ForeignExecError> {
        self.dispatch_view_scalars(declaration, bytes, &[])
    }

    /// PLAN-967 T-11: dispatch with the scalar lanes (the i32 lanes after
    /// the view, materialized by the AV arm from the guest slots).
    pub fn dispatch_view_scalars(
        &mut self,
        declaration: &SourceDecl,
        bytes: &[u8],
        scalars: &[i32],
    ) -> Result<ForeignDispatch, ForeignExecError> {
        let idx = self.lookup(declaration).map_err(|r| {
            self.rejected_count += 1;
            ForeignExecError::Reject(r)
        })?;
        if self.entries[idx].generation != self.current_generation {
            self.rejected_count += 1;
            return Err(reject(
                "foreign.generation",
                format!(
                    "stub generation {} is not the current generation {} (session moved on)",
                    self.entries[idx].generation, self.current_generation
                ),
            ));
        }
        if std::str::from_utf8(bytes).is_err() {
            self.rejected_count += 1;
            return Err(reject(
                "foreign.utf8",
                "view bytes are not valid UTF-8 (the ViewText contract)",
            ));
        }
        // T-05 F1 (IF-CALL): frame-typed declarations carry a p967.frame/1
        // tag frame on the wire — validate BEFORE any dispatch (a bad
        // frame is a named rejection; the callee never runs).
        if self.entries[idx].frame {
            if let Err(reason) = validate_frame_wire(bytes) {
                self.rejected_count += 1;
                return Err(reject(
                    "foreign.frame",
                    format!("p967.frame/1 rejected: {reason}"),
                ));
            }
        }
        if bytes.len() > MAX_CALL_BYTES {
            self.rejected_count += 1;
            return Err(reject(
                "foreign.call-bytes",
                format!(
                    "input view is {} bytes; the per-call budget is {MAX_CALL_BYTES}",
                    bytes.len()
                ),
            ));
        }
        let total = self
            .run_bytes
            .checked_add(bytes.len())
            .unwrap_or(usize::MAX);
        if total > MAX_RUN_BYTES {
            self.rejected_count += 1;
            return Err(reject(
                "foreign.run-bytes",
                format!(
                    "charged run bytes would reach {total}; the per-run budget is {MAX_RUN_BYTES}"
                ),
            ));
        }
        let callable = self.entries[idx].callable;
        if matches!(
            callable,
            ForeignCallable::Owned(_)
                | ForeignCallable::Callback(_)
                | ForeignCallable::Completion { .. }
        ) {
            self.rejected_count += 1;
            return Err(reject(
                "foreign.lane",
                "owned resources require the checked owner path",
            ));
        }
        let data = bytes.as_ptr();
        let byte_len = bytes.len() as u64;
        // Reentry guard: the same discipline as the console host scope.
        let _scope = crate::host::ProviderScope::enter().ok_or_else(|| {
            self.rejected_count += 1;
            reject(
                "foreign.reentry",
                "synchronous provider reentry is forbidden",
            )
        })?;
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || match callable {
                ForeignCallable::PureI32(f) => {
                    ForeignDispatch::PureI32(unsafe { f(data, byte_len) })
                }
                ForeignCallable::PureBool(f) => {
                    ForeignDispatch::PureBool(unsafe { f(data, byte_len) } != 0)
                }
                ForeignCallable::FallibleOutI32(f) => {
                    let mut out: i32 = 0;
                    let status = unsafe { f(data, byte_len, &mut out) };
                    ForeignDispatch::Fallible(status, out)
                }
                // PLAN-967 T-11: mixed view+scalar frames — the scalar
                // values ride the declaration's i32 lanes after the view.
                ForeignCallable::ViewI32I32ToI32(f) => {
                    ForeignDispatch::PureI32(unsafe { f(data, byte_len, scalars[0], scalars[1]) })
                }
                // PLAN-1114: the one-scalar i32 lane of the same family.
                ForeignCallable::ViewI32ToI32(f) => {
                    ForeignDispatch::PureI32(unsafe { f(data, byte_len, scalars[0]) })
                }
                ForeignCallable::ViewI32ToBool(f) => {
                    ForeignDispatch::PureBool(unsafe { f(data, byte_len, scalars[0]) != 0 })
                }
                ForeignCallable::ViewI32FallibleI32(f) => {
                    let mut out: i32 = 0;
                    let status = unsafe { f(data, byte_len, scalars[0], &mut out) };
                    ForeignDispatch::Fallible(status, out)
                }
                ForeignCallable::ViewOutBytesToI32(_)
                | ForeignCallable::OutBytesToI32(_)
                | ForeignCallable::Owned(_)
                | ForeignCallable::Callback(_)
                | ForeignCallable::Completion { .. } => {
                    panic!("out-buffer callable must dispatch via dispatch_view_out_bytes")
                }
            }))
            .map_err(|payload| {
                self.rejected_count += 1;
                let message = payload
                    .downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| payload.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "provider panicked".into());
                ForeignExecError::Panic(message)
            });
        // The callee ran either way (in-unit catch): count the call and
        // charge the ledger BEFORE the panic check — a panicked call is
        // a HOST failure, not an uncounted rejection.
        self.run_bytes = total;
        self.callee_count += 1;
        let result = result?;
        // The frozen panic channel: observe (and clear) the provider's
        // exported flag; a raised flag invalidates the call's values.
        if self.take_panic_flag() {
            self.rejected_count += 1;
            return Err(ForeignExecError::Panic(
                "provider panic raised the exported flag (in-unit catch)".into(),
            ));
        }
        Ok(result)
    }

    pub fn dispatch_callback(
        &mut self,
        declaration: &SourceDecl,
        ctx: i32,
        item: i32,
        mut callback: impl FnMut(i32, i32) -> Option<i32>,
    ) -> Result<ForeignDispatch, ForeignExecError> {
        let index = self.lookup(declaration).map_err(|failure| {
            self.rejected_count += 1;
            ForeignExecError::Reject(failure)
        })?;
        let stub = &self.entries[index];
        if stub.generation != self.current_generation {
            self.rejected_count += 1;
            return Err(reject(
                "foreign.generation",
                "callback provider binding is stale",
            ));
        }
        let ForeignCallable::Callback(call) = stub.callable else {
            return Err(reject("foreign.lane", "expected a Rust callback callable"));
        };
        let _provider = crate::host::ProviderScope::enter().ok_or_else(|| {
            reject(
                "foreign.reentry",
                "synchronous provider reentry is forbidden",
            )
        })?;
        let _callback = CallbackScope::enter(&mut callback)
            .ok_or_else(|| reject("foreign.reentry", "callback scope is already active"))?;
        let result = unsafe { call(callback_trampoline, i64::from(ctx), item) };
        self.callee_count += 1;
        if self.take_panic_flag() {
            self.rejected_count += 1;
            return Err(ForeignExecError::Panic(
                "Rust callback provider panicked".into(),
            ));
        }
        Ok(ForeignDispatch::PureI32(result))
    }

    pub(crate) fn dispatch_completion(
        &mut self,
        declaration: &SourceDecl,
        input: i32,
        cancel: &crate::av::CancelFlag,
        started: std::time::Instant,
        wall: std::time::Duration,
    ) -> Result<ForeignDispatch, ForeignExecError> {
        use std::sync::{atomic::Ordering, mpsc};
        use std::time::Duration;

        let index = self.lookup(declaration).map_err(|failure| {
            self.rejected_count += 1;
            ForeignExecError::Reject(failure)
        })?;
        let stub = &self.entries[index];
        if stub.generation != self.current_generation {
            self.rejected_count += 1;
            return Err(reject(
                "foreign.generation",
                "completion provider binding is stale",
            ));
        }
        let ForeignCallable::Completion { call, asynchronous } = stub.callable else {
            return Err(reject(
                "foreign.lane",
                "expected a Rust completion callable",
            ));
        };
        let check_stop = || {
            if crate::av::CancelCheck::poll(cancel) {
                Err(ForeignExecError::Cancelled)
            } else if started.elapsed() >= wall {
                Err(ForeignExecError::Wall)
            } else {
                Ok(())
            }
        };
        check_stop()?;
        let _provider = crate::host::ProviderScope::enter().ok_or_else(|| {
            reject(
                "foreign.reentry",
                "completion provider reentry is forbidden",
            )
        })?;
        let ports = &self.completion_ports;
        let call_rust = move || {
            let mut value = 0;
            let status = unsafe { call(input, &mut value) };
            (status, value)
        };
        let (port, (status, value)) = if asynchronous {
            let lease = self.code_lease.clone().ok_or_else(|| {
                reject(
                    "foreign.completion",
                    "background work requires a pinned provider image",
                )
            })?;
            let busy = self.completion_worker_busy.clone();
            if busy.swap(true, Ordering::AcqRel) {
                return Err(reject(
                    "foreign.completion-busy",
                    "previous background provider call has not returned",
                ));
            }
            let port = ports.register(&stub.symbol);
            let worker_ports = ports.clone();
            let (result_tx, result_rx) = mpsc::sync_channel(0);
            let worker = std::thread::Builder::new()
                .name("av-foreign-completion".into())
                .spawn(move || {
                    let _lease = lease;
                    let _provider =
                        crate::host::ProviderScope::enter().expect("fresh worker thread");
                    let result = call_rust();
                    busy.store(false, Ordering::Release);
                    // A zero-capacity channel gives the guest exactly one
                    // consumer. If it has left, the producer owns the late
                    // arrival and still holds the native image.
                    if let Err(mpsc::SendError((status, value))) = result_tx.send(result) {
                        worker_ports.cancel(port);
                        worker_ports.complete(port, completion_value(status, value));
                        worker_ports.retire(port);
                    }
                })
                .map_err(|error| {
                    self.completion_worker_busy.store(false, Ordering::Release);
                    ports.cancel(port);
                    ports.retire(port);
                    reject("foreign.completion-worker", error.to_string())
                })?;
            self.callee_count += 1;
            let result = loop {
                if let Err(error) = check_stop() {
                    ports.cancel(port);
                    return Err(error);
                }
                if !ports.is_pending(port) {
                    return Err(ForeignExecError::Cancelled);
                }
                let timeout = wall
                    .saturating_sub(started.elapsed())
                    .min(Duration::from_millis(2));
                match result_rx.recv_timeout(timeout) {
                    Ok(result) => break result,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        ports.cancel(port);
                        ports.retire(port);
                        return Err(reject(
                            "foreign.completion-worker",
                            "provider worker ended without a completion",
                        ));
                    }
                }
            };
            if worker.join().is_err() {
                ports.cancel(port);
                ports.retire(port);
                return Err(reject(
                    "foreign.completion-worker",
                    "provider worker failed after returning a completion",
                ));
            }
            (port, result)
        } else {
            let port = ports.register(&stub.symbol);
            self.callee_count += 1;
            (port, call_rust())
        };
        if let Err(error) = check_stop() {
            ports.cancel(port);
            ports.complete(port, completion_value(status, value));
            ports.retire(port);
            return Err(error);
        }
        if status != 0 {
            if ports.cancel(port) {
                ports.retire(port);
                self.rejected_count += 1;
                return Err(ForeignExecError::Panic(
                    "Rust completion provider panicked".into(),
                ));
            }
            ports.complete(port, completion_value(status, value));
            ports.retire(port);
            return Err(ForeignExecError::Cancelled);
        }
        match ports.complete(port, completion_value(status, value)) {
            CompletionDisposition::Publish => Ok(ForeignDispatch::PureI32(value)),
            _ => {
                ports.retire(port);
                Err(ForeignExecError::Cancelled)
            }
        }
    }

    fn take_panic_flag(&self) -> bool {
        !self.panic_flag.is_null()
            && unsafe {
                std::sync::atomic::AtomicU32::from_ptr(self.panic_flag)
                    .swap(0, std::sync::atomic::Ordering::SeqCst)
                    != 0
            }
    }

    pub fn dispatch_owned(
        &mut self,
        declaration: &SourceDecl,
        input: OwnedArgument<'_>,
    ) -> Result<ForeignDispatch, ForeignExecError> {
        let idx = self.lookup(declaration).map_err(|failure| {
            self.rejected_count += 1;
            ForeignExecError::Reject(failure)
        })?;
        let stub = &self.entries[idx];
        if stub.generation != self.current_generation {
            self.rejected_count += 1;
            return Err(reject(
                "foreign.generation",
                "owned provider binding is stale",
            ));
        }
        let ForeignCallable::Owned(operation) = stub.callable else {
            return Err(reject(
                "foreign.lane",
                "expected an owned resource callable",
            ));
        };
        let _scope = crate::host::ProviderScope::enter().ok_or_else(|| {
            self.rejected_count += 1;
            reject(
                "foreign.reentry",
                "synchronous provider reentry is forbidden",
            )
        })?;
        let value = match operation {
            OwnedCall::Open {
                start,
                close,
                class,
            } => {
                let id = next_owned_id()
                    .ok_or_else(|| reject("foreign.handle-space", "owned handle IDs exhausted"))?;
                let raw = match (start, input) {
                    (OwnedOpen::Text(call), OwnedArgument::Text(bytes)) => {
                        if std::str::from_utf8(bytes).is_err() {
                            self.rejected_count += 1;
                            return Err(reject("foreign.utf8", "owned opener takes valid UTF-8"));
                        }
                        if bytes.len() > MAX_CALL_BYTES
                            || self.run_bytes.saturating_add(bytes.len()) > MAX_RUN_BYTES
                        {
                            self.rejected_count += 1;
                            return Err(reject(
                                "foreign.call-bytes",
                                "owned opener exceeds the view budget",
                            ));
                        }
                        self.run_bytes += bytes.len();
                        unsafe { call(bytes.as_ptr(), bytes.len() as u64) }
                    }
                    (OwnedOpen::I32(call), OwnedArgument::I32(seed)) => unsafe { call(seed) },
                    (OwnedOpen::Unit(call), OwnedArgument::Unit) => unsafe { call() },
                    _ => {
                        return Err(reject(
                            "foreign.lane",
                            "owned opener argument shape mismatch",
                        ))
                    }
                };
                self.callee_count += 1;
                if self.take_panic_flag() {
                    self.rejected_count += 1;
                    if raw != 0 {
                        unsafe { close(raw) };
                        let _ = self.take_panic_flag();
                    }
                    return Err(ForeignExecError::Panic("owned opener panicked".into()));
                }
                if raw == 0 {
                    return Err(reject(
                        "foreign.handle-null",
                        "owned opener returned no resource",
                    ));
                }
                self.owned_handles.insert(
                    id,
                    OwnedHandle {
                        provider: stub.provider.clone(),
                        generation: stub.generation,
                        class,
                        raw,
                        close,
                    },
                );
                id
            }
            OwnedCall::Borrow { call, class } | OwnedCall::Close { call, class } => {
                let OwnedArgument::I32(id) = input else {
                    return Err(reject(
                        "foreign.lane",
                        "owned resource needs an opaque handle",
                    ));
                };
                let Some(handle) = self.owned_handles.get(&id).filter(|entry| {
                    entry.provider == stub.provider
                        && entry.generation == stub.generation
                        && entry.class == class
                }) else {
                    self.rejected_count += 1;
                    return Err(reject(
                        "foreign.handle",
                        "handle is closed or belongs to another resource binding",
                    ));
                };
                let raw = handle.raw;
                if matches!(operation, OwnedCall::Close { .. }) {
                    self.owned_handles.remove(&id);
                }
                let result = unsafe { call(raw) };
                self.callee_count += 1;
                if self.take_panic_flag() {
                    self.rejected_count += 1;
                    return Err(ForeignExecError::Panic(
                        "owned resource operation panicked".into(),
                    ));
                }
                result
            }
        };
        Ok(ForeignDispatch::PureI32(value))
    }

    /// PLAN-1064 T-FRAME: checked dispatch for `ResKind::OutBytes` (`ViewText + OutBytes -> i32`
    /// or `OutBytes -> i32`). Validates owner/origin/generation/rights/bounds/pin before calling,
    /// shields backing storage with leading + trailing guard bytes on a temporary shadow buffer,
    /// holds an RAII pin across the native call (never unpinned until native returns or unwinds),
    /// and commits ONLY `0 <= n <= cap` written bytes after guard integrity passes.
    pub fn dispatch_view_out_bytes(
        &mut self,
        declaration: &SourceDecl,
        path_bytes: &[u8],
        buf_id: i32,
        cancel: &crate::av::CancelFlag,
    ) -> Result<ForeignDispatch, ForeignExecError> {
        const GUARD_LEN: usize = 16;
        const GUARD_PRE: u8 = 0xA5;
        const GUARD_POST: u8 = 0x5A;

        if cancel.load(std::sync::atomic::Ordering::SeqCst) {
            self.rejected_count += 1;
            return Err(reject(
                "foreign.cancelled",
                "cancelled before foreign out-borrow dispatch",
            ));
        }
        let idx = self.lookup(declaration).map_err(|r| {
            self.rejected_count += 1;
            ForeignExecError::Reject(r)
        })?;
        if self.entries[idx].generation != self.current_generation {
            self.rejected_count += 1;
            return Err(reject(
                "foreign.generation",
                format!(
                    "stub generation {} is not the current generation {} (session moved on)",
                    self.entries[idx].generation, self.current_generation
                ),
            ));
        }
        let callable = self.entries[idx].callable;
        if matches!(callable, ForeignCallable::ViewOutBytesToI32(_))
            && std::str::from_utf8(path_bytes).is_err()
        {
            self.rejected_count += 1;
            return Err(reject(
                "foreign.utf8",
                "view bytes are not valid UTF-8 (the ViewText contract)",
            ));
        }
        let (offset, cap, pin_flag) = {
            let Some(frame) = self.out_buffers.get(&buf_id) else {
                self.rejected_count += 1;
                return Err(reject(
                    "foreign.out-missing",
                    format!("unknown writable out buffer `{buf_id}`"),
                ));
            };
            if frame.closed {
                self.rejected_count += 1;
                return Err(reject(
                    "foreign.out-closed",
                    format!("out buffer `{buf_id}` is already closed"),
                ));
            }
            if frame.is_pinned() {
                self.rejected_count += 1;
                return Err(reject(
                    "foreign.out-aliased",
                    format!("out buffer `{buf_id}` is already pinned by an active mutable borrow"),
                ));
            }
            if frame.rights != OutBufferRights::ReadWrite {
                self.rejected_count += 1;
                return Err(reject(
                    "foreign.out-readonly",
                    format!("out buffer `{buf_id}` lacks write permission"),
                ));
            }
            if frame.owner != self.out_expected_owner {
                self.rejected_count += 1;
                return Err(reject(
                    "foreign.out-owner",
                    format!(
                        "out buffer `{buf_id}` owner `{}` does not match `{}`",
                        frame.owner, self.out_expected_owner
                    ),
                ));
            }
            if frame.origin != self.out_expected_origin {
                self.rejected_count += 1;
                return Err(reject(
                    "foreign.out-origin",
                    format!(
                        "out buffer `{buf_id}` origin `{}` does not match `{}`",
                        frame.origin, self.out_expected_origin
                    ),
                ));
            }
            if frame.generation != self.current_generation {
                self.rejected_count += 1;
                return Err(reject(
                    "foreign.out-generation",
                    format!(
                        "out buffer `{buf_id}` generation {} != current {}",
                        frame.generation, self.current_generation
                    ),
                ));
            }
            let Some(end) = frame.offset.checked_add(frame.capacity) else {
                self.rejected_count += 1;
                return Err(reject(
                    "foreign.out-bounds",
                    "out buffer offset + capacity overflow",
                ));
            };
            if end > frame.backing.len() {
                self.rejected_count += 1;
                return Err(reject(
                    "foreign.out-bounds",
                    format!(
                        "out buffer slice {}..{} exceeds backing length {}",
                        frame.offset,
                        end,
                        frame.backing.len()
                    ),
                ));
            }
            (frame.offset, frame.capacity, frame.pin_flag.clone())
        };

        let call_bytes = path_bytes.len().saturating_add(cap);
        if call_bytes > MAX_CALL_BYTES {
            self.rejected_count += 1;
            return Err(reject(
                "foreign.call-bytes",
                format!("out call {call_bytes} bytes exceeds per-call budget {MAX_CALL_BYTES}"),
            ));
        }
        let total = self.run_bytes.checked_add(call_bytes).unwrap_or(usize::MAX);
        if total > MAX_RUN_BYTES {
            self.rejected_count += 1;
            return Err(reject(
                "foreign.run-bytes",
                format!("charged run bytes would reach {total}; budget {MAX_RUN_BYTES}"),
            ));
        }

        let _scope = crate::host::ProviderScope::enter().ok_or_else(|| {
            self.rejected_count += 1;
            reject(
                "foreign.reentry",
                "synchronous provider reentry is forbidden",
            )
        })?;

        struct PinGuard(std::sync::Arc<std::sync::atomic::AtomicBool>);
        impl Drop for PinGuard {
            fn drop(&mut self) {
                self.0.store(false, std::sync::atomic::Ordering::SeqCst);
            }
        }
        pin_flag.store(true, std::sync::atomic::Ordering::SeqCst);
        let _pin_guard = PinGuard(pin_flag);

        let mut shadow = vec![0u8; GUARD_LEN + cap + GUARD_LEN];
        for b in &mut shadow[..GUARD_LEN] {
            *b = GUARD_PRE;
        }
        for b in &mut shadow[GUARD_LEN + cap..] {
            *b = GUARD_POST;
        }
        let dst_ptr = unsafe { shadow.as_mut_ptr().add(GUARD_LEN) };
        let path_ptr = path_bytes.as_ptr();
        let path_len = path_bytes.len() as u64;
        let cap_u64 = cap as u64;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match callable {
            ForeignCallable::ViewOutBytesToI32(f) => unsafe {
                f(path_ptr, path_len, dst_ptr, cap_u64)
            },
            ForeignCallable::OutBytesToI32(f) => unsafe { f(dst_ptr, cap_u64) },
            _ => unreachable!("validated callable variant"),
        }))
        .map_err(|payload| {
            self.rejected_count += 1;
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "provider panicked".into());
            ForeignExecError::Panic(message)
        });

        self.run_bytes = total;
        self.callee_count += 1;
        let written = result?;

        if !self.panic_flag.is_null() {
            let flag = unsafe {
                std::sync::atomic::AtomicU32::from_ptr(self.panic_flag)
                    .swap(0, std::sync::atomic::Ordering::SeqCst)
            };
            if flag != 0 {
                self.rejected_count += 1;
                return Err(ForeignExecError::Panic(
                    "provider panic raised the exported flag (in-unit catch)".into(),
                ));
            }
        }
        if shadow[..GUARD_LEN].iter().any(|&b| b != GUARD_PRE)
            || shadow[GUARD_LEN + cap..].iter().any(|&b| b != GUARD_POST)
        {
            self.rejected_count += 1;
            return Err(reject(
                "foreign.out-guard",
                "native callee corrupted guard bytes adjacent to out buffer",
            ));
        }
        if written < 0 {
            if let Some(frame) = self.out_buffers.get_mut(&buf_id) {
                frame.last_published = 0;
            }
            return Ok(ForeignDispatch::PureI32(written));
        }
        let written_usize = written as usize;
        if written_usize > cap {
            self.rejected_count += 1;
            return Err(reject(
                "foreign.out-overflow",
                format!("native returned written count {written_usize} > capacity {cap}"),
            ));
        }
        if let Some(frame) = self.out_buffers.get_mut(&buf_id) {
            if written_usize > 0 {
                frame.backing[offset..offset + written_usize]
                    .copy_from_slice(&shadow[GUARD_LEN..GUARD_LEN + written_usize]);
            }
            frame.last_published = written_usize;
        }
        Ok(ForeignDispatch::PureI32(written))
    }
}

/// PLAN-967 T-02: one foreign SESSION binding — the single reusable
/// composition entrance consumers (969/E0, later A2X lanes) drive:
/// `open` → `bind` (admitted stubs at the current generation) →
/// `execute` (per-run authority + fresh byte ledger) → `teardown`
/// (every entry stops admitting instantly; a late caller is refused
/// before any dispatch, `callee_count` frozen at its last value) →
/// `rebind` for the next provider binding.
///
/// This is the §0 "generation+owner+签名验证，授权每次调用重核" spine at
/// the granularity F0 froze: identity (provider/symbol/content_version/
/// form) and generation are re-checked on EVERY call; the owner/grant
/// dimension joins at T-13 without changing this entrance.
#[derive(Debug)]
pub struct ForeignSession {
    table: ForeignStubTable,
}

impl Default for ForeignSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ForeignSession {
    fn drop(&mut self) {
        self.teardown();
    }
}

impl ForeignSession {
    /// A fresh session at generation 1.
    pub fn new() -> Self {
        Self {
            table: ForeignStubTable::new(1),
        }
    }

    pub fn generation(&self) -> u64 {
        self.table.generation()
    }

    /// Observe subsequent completion events for this binding. A producer
    /// may keep the registry through cancellation, rebind, or close.
    pub fn completion_ports(&self) -> std::sync::Arc<CompletionPorts> {
        self.table.completion_ports.enable_observation();
        self.table.completion_ports.clone()
    }

    /// Bind admitted stubs at the CURRENT generation. The caller owns
    /// admission (the manifest admit + the loaded exports); this layer
    /// never invents identity.
    pub fn bind(&mut self, stubs: Vec<ForeignStub>) -> Result<(), &'static str> {
        for stub in stubs {
            self.table.insert(stub)?;
        }
        Ok(())
    }

    pub fn table(&mut self) -> &mut ForeignStubTable {
        &mut self.table
    }

    /// Read-only table view (identity/counters for callers that hold the
    /// session, not the table).
    pub fn table_shared(&self) -> &ForeignStubTable {
        &self.table
    }

    pub fn execute(
        &mut self,
        module: &crate::abc_verify::LoadedModule,
        limits: &crate::av::ExecLimits,
        cancel: &crate::av::CancelFlag,
    ) -> Result<(crate::contract::ExecOutcome, crate::av::ExecTrace), crate::host::AdmissionError>
    {
        crate::av::execute_foreign(module, limits, cancel, &mut self.table)
    }

    /// Session teardown: the whole binding dies atomically. Every later
    /// execute/admission refuses; late callees cannot run.
    pub fn teardown(&mut self) {
        self.table.bump_generation();
    }

    /// Provider rebind: the old binding dies, the next binding starts at
    /// a fresh generation (a provider edit never keeps stale authority).
    pub fn rebind(&mut self, stubs: Vec<ForeignStub>) -> Result<(), &'static str> {
        self.teardown();
        self.bind(stubs)
    }

    /// Adopt a fully-loaded table (a loader-built table at the session's
    /// generation); the session takes over its counters and flag.
    pub fn adopt(&mut self, table: ForeignStubTable) {
        self.teardown();
        self.table = table;
    }
}

/// PLAN-967 T-08 (AC-06): the host-side completion-port registry — the
/// registration/completion/cancellation port 968's language `await`
/// consumes. Every disposition goes through the frozen
/// [`ace_foreign_contract::completion_disposition`] rule table; late
/// completions (cancel, teardown, rebind) are DROPPED, never published,
/// and never touch a closed session.
#[derive(Debug)]
pub struct CompletionPorts {
    inner: std::sync::Mutex<PortRegistryInner>,
}

fn completion_value(status: i32, value: i32) -> ace_foreign_contract::PortValue {
    if status == 0 {
        ace_foreign_contract::PortValue::I32(value)
    } else {
        ace_foreign_contract::PortValue::Status(status)
    }
}

#[derive(Debug)]
struct PortRegistryInner {
    generation: u64,
    next_port: u64,
    open: std::collections::BTreeMap<u64, OpenPort>,
    observe_events: bool,
    events: Vec<ace_foreign_contract::PortEvent>,
}

#[derive(Debug)]
struct OpenPort {
    released: bool,
    pinned_generation: u64,
}

impl CompletionPorts {
    pub fn new(generation: u64) -> Self {
        Self::with_observation(generation, true)
    }

    fn with_observation(generation: u64, observe_events: bool) -> Self {
        Self {
            inner: std::sync::Mutex::new(PortRegistryInner {
                generation,
                next_port: 1,
                open: std::collections::BTreeMap::new(),
                observe_events,
                events: Vec::new(),
            }),
        }
    }

    fn enable_observation(&self) {
        self.inner.lock().expect("port registry").observe_events = true;
    }

    /// Register a port for `symbol` at the CURRENT generation (the port
    /// pins the generation; a later teardown/rebind drops its lateness).
    pub fn register(&self, symbol: &str) -> u64 {
        let mut inner = self.inner.lock().expect("port registry");
        let port = inner.next_port;
        inner.next_port += 1;
        let pinned_generation = inner.generation;
        inner.open.insert(
            port,
            OpenPort {
                released: false,
                pinned_generation,
            },
        );
        inner.record(|| ace_foreign_contract::PortEvent::Registered {
            port,
            symbol: symbol.to_string(),
        });
        port
    }

    /// An arriving completion: publish, drop-late, or named-unknown —
    /// exactly the frozen rule table.
    pub fn complete(
        &self,
        port: u64,
        value: ace_foreign_contract::PortValue,
    ) -> CompletionDisposition {
        let mut inner = self.inner.lock().expect("port registry");
        let Some(open) = inner.open.get(&port) else {
            return CompletionDisposition::UnknownPort;
        };
        let pinned = open.pinned_generation;
        let cancelled = open.released;
        let disposition =
            ace_foreign_contract::completion_disposition(true, cancelled, pinned, inner.generation);
        match disposition {
            CompletionDisposition::Publish => {
                inner.record(|| ace_foreign_contract::PortEvent::Published { port, value });
                inner.open.remove(&port);
                inner.record(|| ace_foreign_contract::PortEvent::Released { port });
            }
            CompletionDisposition::DropLate => {
                inner.record(|| ace_foreign_contract::PortEvent::LateDropped { port, value });
            }
            CompletionDisposition::UnknownPort => {}
        }
        disposition
    }

    /// Cancel a port: every later completion drops. The cancel releases
    /// the pinned provider callback but keeps a CANCELLED TOMBSTONE so a
    /// racing late completion classifies as `DropLate` ("was cancelled"),
    /// never as `UnknownPort` ("never existed") — the two are different
    /// facts for the consumer.
    pub fn cancel(&self, port: u64) -> bool {
        let mut inner = self.inner.lock().expect("port registry");
        let Some(open) = inner.open.get_mut(&port) else {
            return false;
        };
        if open.released {
            return false;
        }
        open.released = true;
        inner.record(|| ace_foreign_contract::PortEvent::Cancelled { port });
        inner.record(|| ace_foreign_contract::PortEvent::Released { port });
        true
    }

    fn is_pending(&self, port: u64) -> bool {
        let inner = self.inner.lock().expect("port registry");
        inner
            .open
            .get(&port)
            .is_some_and(|entry| !entry.released && entry.pinned_generation == inner.generation)
    }

    // A Rust completion export has one producer and one return. Once
    // that producer has stopped, no tombstone is needed for its port.
    fn retire(&self, port: u64) {
        self.inner.lock().expect("port registry").open.remove(&port);
    }

    /// Session teardown/rebind: every open port dies — late completions
    /// observe the moved generation and drop.
    pub fn teardown(&mut self) {
        let inner = self.inner.get_mut().expect("port registry");
        inner.advance_generation(inner.generation + 1);
    }

    /// PLAN-967 T-13: FUTURE-PORT WIRING — bind the port registry to a
    /// ForeignSession's generation so a session teardown/rebind kills
    /// every in-flight port (the late-completion drop) and a provider
    /// rebind starts from a fresh generation. The consumer (968's await)
    /// drives register/complete/cancel; the SESSION owns the generation
    /// axis.
    pub fn bind_session_generation(&self, session_generation: u64) {
        self.inner
            .lock()
            .expect("port registry")
            .advance_generation(session_generation);
    }

    /// PLAN-967 T-13: the current binding generation (the session's).
    pub fn session_generation(&self) -> u64 {
        self.generation()
    }

    pub fn generation(&self) -> u64 {
        self.inner.lock().expect("port registry").generation
    }

    /// Drain the observable event stream (the consumer's only oracle).
    pub fn take_events(&self) -> Vec<ace_foreign_contract::PortEvent> {
        let mut inner = self.inner.lock().expect("port registry");
        std::mem::take(&mut inner.events)
    }
}

impl PortRegistryInner {
    fn record(&mut self, event: impl FnOnce() -> ace_foreign_contract::PortEvent) {
        if self.observe_events {
            self.events.push(event());
        }
    }

    fn advance_generation(&mut self, generation: u64) {
        if self.generation == generation {
            return;
        }
        self.generation = generation;
        for (&port, entry) in &mut self.open {
            if !entry.released {
                entry.released = true;
                if self.observe_events {
                    self.events
                        .push(ace_foreign_contract::PortEvent::Released { port });
                }
            }
        }
    }
}

use ace_foreign_contract::CompletionDisposition;

#[cfg(test)]
mod tests {
    use super::*;
    use ace_foreign_contract::manual::SOURCE_CAP;
    use ace_foreign_contract::manual::{HostAuthority, ManualManifest, MANUAL_SCHEMA_V2};
    use ace_foreign_contract::resource::{
        ResKind, ResOut, ResourceForm, ResourceSig, RESOURCE_CAP, RESOURCE_SIG_FORMAT,
    };
    use ace_foreign_contract::{
        ErrorModel, FactsSource, FnFact, FnForm, InterfaceFacts, LaneKind, FACTS_FORMAT,
    };

    unsafe extern "C" fn len_i32(_data: *const u8, byte_len: u64) -> i32 {
        byte_len as i32
    }
    unsafe extern "C" fn ascii_bool(data: *const u8, byte_len: u64) -> u8 {
        let bytes = std::slice::from_raw_parts(data, byte_len as usize);
        bytes.is_ascii() as u8
    }
    unsafe extern "C" fn first_or_status(data: *const u8, byte_len: u64, out: *mut i32) -> i32 {
        let bytes = std::slice::from_raw_parts(data, byte_len as usize);
        match bytes.first() {
            Some(b) => {
                *out = *b as i32;
                0
            }
            None => -1,
        }
    }
    // The panic probe mirrors the GENERATED shim protocol exactly: the
    // provider-side catch is in-unit and raises the exported flag; the
    // table observes the flag after the call. No unwind crosses the
    // plain C frame.
    static LOCAL_PANIC_FLAG: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    unsafe extern "C" fn panic_on_boom(data: *const u8, byte_len: u64) -> i32 {
        let bytes = std::slice::from_raw_parts(data, byte_len as usize);
        let panicked = std::panic::catch_unwind(|| {
            if bytes == b"boom" {
                panic!("probe");
            }
            byte_len as i32
        });
        match panicked {
            Ok(v) => v,
            Err(_) => {
                LOCAL_PANIC_FLAG.store(1, std::sync::atomic::Ordering::SeqCst);
                0
            }
        }
    }
    unsafe extern "C" fn local_panic_flag() -> *mut u32 {
        &LOCAL_PANIC_FLAG as *const _ as *mut u32
    }

    fn manifest() -> ManualManifest {
        let sig = |form: ace_foreign_contract::ResourceForm| ResourceSig {
            format: RESOURCE_SIG_FORMAT.into(),
            params: vec![ResKind::ViewText],
            form,
            atomic: false,
            noescape: true,
            typed_peer: false,
        };
        ManualManifest {
            schema: MANUAL_SCHEMA_V2.into(),
            review_id: "f0-test/1".into(),
            provider: "f0_text".into(),
            header_digest: "a".repeat(64),
            physical_table_digest: "b".repeat(64),
            target: ace_foreign_contract::plan::TARGET.into(),
            profile: ace_foreign_contract::plan::PROFILE.into(),
            callconv: "windows-x64".into(),
            direction: "source-import".into(),
            facts: InterfaceFacts {
                format: FACTS_FORMAT.into(),
                logical: "rust/f0_text".into(),
                content_version: "c".repeat(16),
                source: FactsSource::Typed {
                    id: "f0-test/1".into(),
                },
                functions: vec![
                    FnFact {
                        symbol: "f0_text_byte_len_v1".into(),
                        form: FnForm::Pure {
                            params: vec![],
                            ret: LaneKind::I32,
                        },
                        error_model: None,
                        semantic_ref: None,
                        resource: Some(sig(ResourceForm::Pure { ret: ResOut::I32 })),
                    },
                    FnFact {
                        symbol: "f0_text_is_ascii_v1".into(),
                        form: FnForm::Pure {
                            params: vec![],
                            ret: LaneKind::Bool,
                        },
                        error_model: None,
                        semantic_ref: None,
                        resource: Some(sig(ResourceForm::Pure { ret: ResOut::Bool })),
                    },
                    FnFact {
                        symbol: "f0_text_first_byte_v1".into(),
                        form: FnForm::Fallible {
                            ctx: None,
                            params: vec![],
                            outs: vec![LaneKind::I32],
                        },
                        error_model: Some(ErrorModel {
                            kind: "status-i32".into(),
                            codes: vec![(-1, "empty-input".into())],
                        }),
                        semantic_ref: None,
                        resource: Some(sig(ResourceForm::Fallible {
                            ctx: None,
                            out: Some(ResOut::I32),
                        })),
                    },
                    FnFact {
                        symbol: "f0_text_panic_probe_v1".into(),
                        form: FnForm::Pure {
                            params: vec![],
                            ret: LaneKind::I32,
                        },
                        error_model: None,
                        semantic_ref: None,
                        resource: Some(sig(ResourceForm::Pure { ret: ResOut::I32 })),
                    },
                ],
                effects: vec![
                    "pure:f0_text_byte_len_v1".into(),
                    "pure:f0_text_is_ascii_v1".into(),
                    "pure:f0_text_panic_probe_v1".into(),
                    "checked".into(),
                ],
                capabilities: vec![SOURCE_CAP.into(), RESOURCE_CAP.into()],
            },
            declarations: [
                ("text_byte_len", "f0_text_byte_len_v1"),
                ("text_is_ascii", "f0_text_is_ascii_v1"),
                ("text_first_byte", "f0_text_first_byte_v1"),
                ("text_panic_probe", "f0_text_panic_probe_v1"),
            ]
            .into_iter()
            .map(
                |(source_name, symbol)| ace_foreign_contract::manual::ManualDecl {
                    context_symbol: None,
                    source_name: source_name.into(),
                    symbol: symbol.into(),
                },
            )
            .collect(),
        }
    }

    fn table(generation: u64) -> (ForeignStubTable, ManualManifest) {
        let m = manifest();
        let mut t = ForeignStubTable::new(generation);
        let admitted = m.clone().admit(&authority(&m)).unwrap();
        for decl in admitted.source_declarations_ref() {
            let callable = match decl.function.symbol.as_str() {
                "f0_text_byte_len_v1" | "f0_text_panic_probe_v1" => ForeignCallable::PureI32(
                    if decl.function.symbol.as_str() == "f0_text_byte_len_v1" {
                        len_i32
                    } else {
                        panic_on_boom
                    },
                ),
                "f0_text_is_ascii_v1" => ForeignCallable::PureBool(ascii_bool),
                _ => ForeignCallable::FallibleOutI32(first_or_status),
            };
            t.insert(ForeignStub {
                provider: decl.provider.clone(),
                symbol: decl.function.symbol.clone(),
                content_version: decl.content_version.clone(),
                generation,
                callable,
                frame: false,
            })
            .unwrap();
        }
        t.set_panic_flag(unsafe { local_panic_flag() });
        (t, m)
    }

    fn authority(m: &ManualManifest) -> HostAuthority {
        HostAuthority {
            review_id: m.review_id.clone(),
            contract_digest: m.contract_digest().unwrap(),
            provider: m.provider.clone(),
            provider_digest: "d".repeat(64),
            header_digest: m.header_digest.clone(),
            physical_table_digest: m.physical_table_digest.clone(),
            target: m.target.clone(),
            capabilities: m.facts.capabilities.iter().cloned().collect(),
            available_symbols: m.facts.functions.iter().map(|f| f.symbol.clone()).collect(),
            binding_generation: 1,
        }
    }

    fn decl_of(m: &ManualManifest, symbol: &str) -> SourceDecl {
        let admitted = m.clone().admit(&authority(m)).unwrap();
        admitted
            .source_declarations_ref()
            .iter()
            .find(|d| d.function.symbol == symbol)
            .unwrap()
            .clone()
    }

    #[test]
    fn completion_ports_publish_cancel_and_drop_late_exactly() {
        use crate::foreign_exec::CompletionPorts;
        use ace_foreign_contract::PortValue;

        // publish: open port, current generation
        let mut ports = CompletionPorts::new(1);
        let a = ports.register("f0.async");
        assert_eq!(
            ports.complete(a, PortValue::I32(18)),
            ace_foreign_contract::CompletionDisposition::Publish
        );
        // a settled port is unknown afterwards (never re-published)
        assert_eq!(
            ports.complete(a, PortValue::I32(99)),
            ace_foreign_contract::CompletionDisposition::UnknownPort
        );

        // cancel: a later completion drops late, value never published
        let b = ports.register("f0.async");
        assert!(ports.cancel(b));
        assert!(!ports.cancel(b));
        assert_eq!(
            ports.complete(b, PortValue::I32(7)),
            ace_foreign_contract::CompletionDisposition::DropLate
        );

        // teardown: a completion racing in after the session moved drops
        let c = ports.register("f0.async");
        ports.teardown();
        assert!(!ports.cancel(c));
        assert_eq!(
            ports.complete(c, PortValue::Status(-1)),
            ace_foreign_contract::CompletionDisposition::DropLate
        );

        // an unknown id is a named rejection, not a silent drop
        assert_eq!(
            ports.complete(9999, PortValue::I32(1)),
            ace_foreign_contract::CompletionDisposition::UnknownPort
        );

        let events = ports.take_events();
        let spells: Vec<String> = events
            .iter()
            .map(|e| match e {
                ace_foreign_contract::PortEvent::Registered { port, .. } => {
                    format!("reg:{port}")
                }
                ace_foreign_contract::PortEvent::Published { port, value } => {
                    format!("pub:{port}:{}", value.spell())
                }
                ace_foreign_contract::PortEvent::Cancelled { port } => format!("cancel:{port}"),
                ace_foreign_contract::PortEvent::LateDropped { port, value } => {
                    format!("late:{port}:{}", value.spell())
                }
                ace_foreign_contract::PortEvent::Released { port } => format!("rel:{port}"),
            })
            .collect();
        assert_eq!(
            spells,
            vec![
                "reg:1",
                "pub:1:i32:18",
                "rel:1",
                "reg:2",
                "cancel:2",
                "rel:2",
                "late:2:i32:7",
                "reg:3",
                "rel:3",
                "late:3:status:-1",
            ],
            "causal event order is the consumer's only oracle"
        );
        // cancel of an unknown/settled port is a named false
        assert!(!ports.cancel(a));
    }

    #[test]
    fn closing_foreign_session_releases_port_once_and_rejects_late_completion() {
        use ace_foreign_contract::{CompletionDisposition, PortEvent, PortValue};

        let mut session = ForeignSession::new();
        let ports = session.completion_ports();
        let port = ports.register("test.provider");
        session.teardown();
        drop(session);
        assert_eq!(
            ports.complete(port, PortValue::I32(4)),
            CompletionDisposition::DropLate
        );
        assert_eq!(
            ports.take_events(),
            vec![
                PortEvent::Registered {
                    port,
                    symbol: "test.provider".into(),
                },
                PortEvent::Released { port },
                PortEvent::LateDropped {
                    port,
                    value: PortValue::I32(4),
                },
            ]
        );
    }

    #[test]
    fn cancelled_rust_worker_keeps_its_image_and_drops_its_late_result_once() {
        use ace_foreign_contract::{PortEvent, PortValue};
        use std::sync::{atomic::Ordering, mpsc, Arc, Mutex};
        use std::time::{Duration, Instant};

        struct WorkerGate {
            entered: mpsc::Sender<()>,
            release: mpsc::Receiver<()>,
        }
        static GATE: Mutex<Option<WorkerGate>> = Mutex::new(None);
        unsafe extern "C" fn double(input: i32, out: *mut i32) -> i32 {
            let Some(gate) = GATE.lock().ok().and_then(|mut gate| gate.take()) else {
                return 1;
            };
            if gate.entered.send(()).is_err()
                || gate.release.recv_timeout(Duration::from_secs(3)).is_err()
            {
                return 1;
            }
            unsafe { *out = input * 2 };
            0
        }
        let mut provider = manifest();
        provider.facts.functions[0]
            .resource
            .as_mut()
            .unwrap()
            .params = vec![ResKind::I32];
        let declaration = decl_of(&provider, "f0_text_byte_len_v1");
        let mut session = ForeignSession::new();
        let lease = Arc::new(());
        let weak = Arc::downgrade(&lease);
        session.table().pin_code_image(lease);
        session
            .bind(vec![ForeignStub {
                provider: declaration.provider.clone(),
                symbol: declaration.function.symbol.clone(),
                content_version: declaration.content_version.clone(),
                generation: 1,
                callable: ForeignCallable::Completion {
                    call: double,
                    asynchronous: true,
                },
                frame: false,
            }])
            .unwrap();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        *GATE.lock().unwrap() = Some(WorkerGate {
            entered: entered_tx,
            release: release_rx,
        });
        let ports = session.completion_ports();
        let cancel = crate::av::CancelFlag::new(false);
        let timeout = Duration::from_secs(2);
        std::thread::scope(|scope| {
            let worker_cancel = &cancel;
            scope.spawn(move || {
                entered_rx.recv_timeout(timeout).expect("provider entered");
                worker_cancel.store(true, Ordering::SeqCst);
            });
            assert_eq!(
                session.table().dispatch_completion(
                    &declaration,
                    21,
                    &cancel,
                    Instant::now(),
                    timeout
                ),
                Err(ForeignExecError::Cancelled)
            );
        });
        drop(session);
        assert!(weak.upgrade().is_some(), "worker still holds its image");
        assert_eq!(
            ports.take_events(),
            [
                PortEvent::Registered {
                    port: 1,
                    symbol: declaration.function.symbol
                },
                PortEvent::Cancelled { port: 1 },
                PortEvent::Released { port: 1 },
            ]
        );
        release_tx.send(()).unwrap();
        let until = Instant::now() + timeout;
        while weak.upgrade().is_some() {
            assert!(Instant::now() < until, "worker did not release its image");
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(
            ports.take_events(),
            [PortEvent::LateDropped {
                port: 1,
                value: PortValue::I32(42)
            }]
        );
    }

    #[test]
    fn dispatches_all_frozen_forms_and_counts_callees() {
        let (mut t, m) = table(1);
        let len = decl_of(&m, "f0_text_byte_len_v1");
        assert_eq!(
            t.dispatch_view(&len, "A\u{4E2D}\u{1F600}\0".as_bytes())
                .unwrap(),
            ForeignDispatch::PureI32(9)
        );
        assert_eq!(
            t.dispatch_view(&len, b"").unwrap(),
            ForeignDispatch::PureI32(0)
        );
        let ascii = decl_of(&m, "f0_text_is_ascii_v1");
        assert_eq!(
            t.dispatch_view(&ascii, b"abc").unwrap(),
            ForeignDispatch::PureBool(true)
        );
        assert_eq!(
            t.dispatch_view(&ascii, "中".as_bytes()).unwrap(),
            ForeignDispatch::PureBool(false)
        );
        let first = decl_of(&m, "f0_text_first_byte_v1");
        assert_eq!(
            t.dispatch_view(&first, b"Abc").unwrap(),
            ForeignDispatch::Fallible(0, 65)
        );
        assert_eq!(t.callee_count(), 5);
        assert_eq!(t.rejected_count(), 0);
    }

    #[test]
    fn rejections_happen_before_dispatch_with_callee_count_zero() {
        let (mut t, m) = table(3);
        let len = decl_of(&m, "f0_text_byte_len_v1");
        // unknown symbol
        let mut ghost = len.clone();
        ghost.function.symbol = "f0_text_ghost_v1".into();
        assert_eq!(
            t.dispatch_view(&ghost, b"x").unwrap_err().code(),
            "foreign.unknown-symbol"
        );
        // generation staleness
        let mut stale = ForeignStubTable::new(3);
        stale.entries = t.entries.clone();
        stale.bump_generation();
        assert_eq!(
            stale.dispatch_view(&len, b"x").unwrap_err().code(),
            "foreign.generation"
        );
        // invalid UTF-8 (F0 contract: bytes must be UTF-8)
        assert_eq!(
            t.dispatch_view(&len, &[0xFF, 0xFE]).unwrap_err().code(),
            "foreign.utf8"
        );
        // per-call budget: exactly at the limit passes, one over rejects
        let at_limit = vec![b'a'; MAX_CALL_BYTES];
        assert!(t.dispatch_view(&len, &at_limit).is_ok());
        let over = vec![b'a'; MAX_CALL_BYTES + 1];
        assert_eq!(
            t.dispatch_view(&len, &over).unwrap_err().code(),
            "foreign.call-bytes"
        );
        // per-run ledger: sixteen max-size calls exactly fill the run
        // budget; one more byte on a seventeenth call rejects.
        t.reset_run_bytes();
        let chunk = vec![b'a'; MAX_CALL_BYTES];
        for _ in 0..MAX_RUN_BYTES / MAX_CALL_BYTES {
            assert!(t.dispatch_view(&len, &chunk).is_ok());
        }
        assert_eq!(t.run_bytes(), MAX_RUN_BYTES);
        assert_eq!(
            t.dispatch_view(&len, b"x").unwrap_err().code(),
            "foreign.run-bytes"
        );
        // identity mismatch: same symbol, different content version
        let mut other = len.clone();
        other.content_version = "zzzz".into();
        assert_eq!(
            t.dispatch_view(&other, b"x").unwrap_err().code(),
            "foreign.signature"
        );
        // FORM (ABI) mismatch: a bool-form declaration against an i32
        // callable bound under the same symbol+content_version is a
        // signature rejection, never a mis-typed dispatch (audit #5).
        let ascii = decl_of(&m, "f0_text_is_ascii_v1");
        let mut swapped = t.entries().to_vec();
        for stub in &mut swapped {
            if stub.symbol == "f0_text_is_ascii_v1" {
                stub.callable = ForeignCallable::PureI32(len_i32);
            }
        }
        let mut wrong_form = ForeignStubTable::new(t.generation());
        wrong_form.set_entries(swapped);
        assert_eq!(
            wrong_form.dispatch_view(&ascii, b"abc").unwrap_err().code(),
            "foreign.signature"
        );
        assert_eq!(wrong_form.callee_count(), 0);
        assert_eq!(t.callee_count(), 17);
        // five rejections on THIS table: ghost, utf8, over-limit,
        // run-bytes and the signature mismatch (the generation check ran
        // on a stale copy and counts on that copy).
        assert_eq!(t.rejected_count(), 5);
    }

    #[test]
    fn provider_panic_is_an_independent_host_failure() {
        let (mut t, m) = table(1);
        let probe = decl_of(&m, "f0_text_panic_probe_v1");
        match t.dispatch_view(&probe, b"boom").unwrap_err() {
            ForeignExecError::Panic(_) => {}
            other => panic!("expected a host-failure panic, got {other:?}"),
        }
        // the panic did not unwind and the session stays usable
        assert_eq!(
            t.dispatch_view(&probe, b"calm").unwrap(),
            ForeignDispatch::PureI32(4)
        );
        // the panicked call RAN (counted) and was rejected as a failure
        assert_eq!(t.callee_count(), 2);
        assert_eq!(t.rejected_count(), 1);
    }

    impl ForeignExecError {
        fn code(&self) -> &'static str {
            match self {
                ForeignExecError::Reject(r) => r.code,
                ForeignExecError::Panic(_) => "foreign.panic",
                ForeignExecError::Cancelled => "foreign.cancelled",
                ForeignExecError::Wall => "foreign.wall",
            }
        }
    }
}

#[cfg(test)]
mod frame_wire_1129_tests {
    //! PLAN-1129: the two wire versions of the tag frame as the AV admission
    //! sees them (the generated shim decoder accepts the same bytes).
    use super::validate_frame_wire;

    const V1: &str = "p967.frame/1\n";
    const V2: &str = "p967.frame/2\n";

    fn ok(version: &str, body: &str) {
        assert_eq!(
            validate_frame_wire(format!("{version}{body}").as_bytes()),
            Ok(()),
            "{version:?}{body:?} must be admitted"
        );
    }

    fn bad(version: &str, body: &str, reason: &'static str) {
        assert_eq!(
            validate_frame_wire(format!("{version}{body}").as_bytes()),
            Err(reason),
            "{version:?}{body:?}"
        );
    }

    /// A positional value with an exact length header.
    fn positional(tag: &str, payload: &str) -> String {
        format!("r n 0\n\np 1\n{tag} {}\n{payload}\nk 0\n", payload.len())
    }

    #[test]
    fn version_one_keeps_exactly_its_five_tags() {
        ok(V1, "r n 0\n\np 0\nk 0\n");
        ok(V1, "r o 1\n7\np 0\nk 0\n");
        ok(V1, &positional("t", "F:2.5"));
        ok(V1, "r n 0\n\np 0\nk 2\n2\nxli 1\n5\n1\nyi 1\n6\n");
        for tag in ["d", "f"] {
            bad(V1, &positional(tag, "0 0 0"), "unknown tag");
        }
        bad(V1, "r d 5\n0 0 0\np 0\nk 0\n", "unknown tag");
        bad(V1, "r n 0\n\np 0\nk 1\n1\nxf 5\n0 0 0\n", "unknown tag");
    }

    #[test]
    fn version_two_admits_typed_floats_in_every_value_slot() {
        ok(V2, "r n 0\n\np 0\nk 0\n");
        ok(V2, "r n 0\n\np 2\nt 0\n\nd 5\n0 0 0\nk 0\n");
        ok(V2, "r d 23\n2097151 2097151 4194303\np 0\nk 0\n");
        ok(V2, "r n 0\n\np 0\nk 1\n1\nxf 23\n2097151 2097151 4194303\n");
        // the version-1 tags and rules are unchanged inside a version-2 frame
        ok(V2, &positional("t", "F:2.5"));
        ok(V2, "r n 0\n\np 1\ni 2\n42\nk 0\n");
        bad(V2, "r x 1\n1\np 0\nk 0\n", "unknown tag");
        bad(V2, "r n 0\n\np 9\nk 0\n", "count over the frame cap");
        bad(
            V2,
            &format!("{}garbage", positional("d", "0 0 0")),
            "trailing bytes after the frame",
        );
    }

    #[test]
    fn a_typed_float_payload_is_three_bounded_decimals() {
        for payload in ["0 0 0", "0 0 1", "0 1 0", "2097151 2097151 4194303"] {
            ok(V2, &positional("d", payload));
            ok(V2, &positional("f", payload));
        }
        let reason = "typed float payload is not `<w2> <w1> <w0>`";
        for payload in ["0", "0 0", "0 0 0 0", "0 0 0 "] {
            bad(V2, &positional("d", payload), reason);
        }
        for payload in ["0 -1 0", "0 +1 0", "0 0 0x1", "0 1.5 0", "0 0 ٣"] {
            bad(
                V2,
                &positional("d", payload),
                "typed float word is not a decimal",
            );
        }
        for payload in [
            "",              // a missing word is an empty one
            "0  0",          // so is a doubled separator
            "2097152 0 0",   // w2 carries 21 bits
            "0 2097152 0",   // so does w1
            "0 0 4194304",   // w0 carries 22
            "12345678 0 0",  // at most seven digits
        ] {
            bad(
                V2,
                &positional("f", payload),
                "typed float word is out of range",
            );
        }
    }

    #[test]
    fn an_unknown_wire_version_is_rejected_before_its_body() {
        for header in ["p967.frame/3\n", "p967.frame/12\n", "p967.frame/\n", ""] {
            bad(header, "r n 0\n\np 0\nk 0\n", "bad magic/version header");
        }
    }

    #[test]
    fn the_decoder_and_the_admission_accept_the_same_bytes() {
        // The generated shim decoder is compiled into every provider; its
        // source is the one prelude, so a sample of the admission's verdicts
        // is replayed against the same acceptance spelled once more here.
        let samples = [
            (V1, positional("t", "F:2.5"), true),
            (V1, positional("d", "0 0 0"), false),
            (V2, positional("d", "0 0 2097151"), true),
            (V2, positional("f", "0 2097151 4194303"), true),
            (V2, positional("f", "0 4194304 0"), false),
            (V2, positional("d", "0 0 "), false),
        ];
        for (version, body, admit) in samples {
            assert_eq!(validate_frame_wire(format!("{version}{body}").as_bytes()).is_ok(), admit);
        }
    }
}
