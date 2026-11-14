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
pub type ViewI32ToBoolFn = unsafe extern "C" fn(data: *const u8, byte_len: u64, p0: i32) -> u8;
pub type ViewI32OutI32Fn =
    unsafe extern "C" fn(data: *const u8, byte_len: u64, p0: i32, out: *mut i32) -> i32;

/// The frozen F0/T-11 physical callables.
#[derive(Clone, Copy, Debug)]
pub enum ForeignCallable {
    PureI32(ViewUtf8I32Fn),
    PureBool(ViewUtf8BoolFn),
    FallibleOutI32(ViewUtf8OutI32Fn),
    /// T-11 mixed frames.
    ViewI32I32ToI32(ViewI32I32ToI32Fn),
    ViewI32ToBool(ViewI32ToBoolFn),
    ViewI32FallibleI32(ViewI32OutI32Fn),
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
        }
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
                && matches!(
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
                    )
                )
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
                ForeignCallable::ViewI32ToBool(f) => {
                    ForeignDispatch::PureBool(unsafe { f(data, byte_len, scalars[0]) != 0 })
                }
                ForeignCallable::ViewI32FallibleI32(f) => {
                    let mut out: i32 = 0;
                    let status = unsafe { f(data, byte_len, scalars[0], &mut out) };
                    ForeignDispatch::Fallible(status, out)
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
        Ok(result)
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
    generation: u64,
    next_port: u64,
    inner: std::sync::Mutex<PortRegistryInner>,
}

#[derive(Debug)]
struct PortRegistryInner {
    open: std::collections::BTreeMap<u64, OpenPort>,
    events: Vec<ace_foreign_contract::PortEvent>,
}

#[derive(Debug)]
struct OpenPort {
    symbol: String,
    cancelled: bool,
    pinned_generation: u64,
}

impl CompletionPorts {
    pub fn new(generation: u64) -> Self {
        Self {
            generation,
            next_port: 1,
            inner: std::sync::Mutex::new(PortRegistryInner {
                open: std::collections::BTreeMap::new(),
                events: Vec::new(),
            }),
        }
    }

    /// Register a port for `symbol` at the CURRENT generation (the port
    /// pins the generation; a later teardown/rebind drops its lateness).
    pub fn register(&mut self, symbol: &str) -> u64 {
        let port = self.next_port;
        self.next_port += 1;
        let mut inner = self.inner.lock().expect("port registry");
        inner.open.insert(
            port,
            OpenPort {
                symbol: symbol.to_string(),
                cancelled: false,
                pinned_generation: self.generation,
            },
        );
        inner
            .events
            .push(ace_foreign_contract::PortEvent::Registered {
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
        let cancelled = open.cancelled;
        let disposition =
            ace_foreign_contract::completion_disposition(true, cancelled, pinned, self.generation);
        match disposition {
            CompletionDisposition::Publish => {
                inner
                    .events
                    .push(ace_foreign_contract::PortEvent::Published { port, value });
                inner.open.remove(&port);
                inner
                    .events
                    .push(ace_foreign_contract::PortEvent::Released { port });
            }
            CompletionDisposition::DropLate => {
                inner
                    .events
                    .push(ace_foreign_contract::PortEvent::LateDropped { port, value });
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
        if let Some(open) = inner.open.get_mut(&port) {
            open.cancelled = true;
            inner
                .events
                .push(ace_foreign_contract::PortEvent::Cancelled { port });
            inner
                .events
                .push(ace_foreign_contract::PortEvent::Released { port });
            true
        } else {
            false
        }
    }

    /// Session teardown/rebind: every open port dies — late completions
    /// observe the moved generation and drop.
    pub fn teardown(&mut self) {
        self.generation += 1;
    }

    /// PLAN-967 T-13: FUTURE-PORT WIRING — bind the port registry to a
    /// ForeignSession's generation so a session teardown/rebind kills
    /// every in-flight port (the late-completion drop) and a provider
    /// rebind starts from a fresh generation. The consumer (968's await)
    /// drives register/complete/cancel; the SESSION owns the generation
    /// axis.
    pub fn bind_session_generation(&mut self, session_generation: u64) {
        self.generation = session_generation;
    }

    /// PLAN-967 T-13: the current binding generation (the session's).
    pub fn session_generation(&self) -> u64 {
        self.generation
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Drain the observable event stream (the consumer's only oracle).
    pub fn take_events(&self) -> Vec<ace_foreign_contract::PortEvent> {
        let mut inner = self.inner.lock().expect("port registry");
        std::mem::take(&mut inner.events)
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

    unsafe extern "C" fn len_i32(data: *const u8, byte_len: u64) -> i32 {
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
        assert_eq!(
            ports.complete(b, PortValue::I32(7)),
            ace_foreign_contract::CompletionDisposition::DropLate
        );

        // teardown: a completion racing in after the session moved drops
        let c = ports.register("f0.async");
        ports.teardown();
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
                "late:3:status:-1",
            ],
            "causal event order is the consumer's only oracle"
        );
        // cancel of an unknown/settled port is a named false
        assert!(!ports.cancel(a));
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
            }
        }
    }
}
