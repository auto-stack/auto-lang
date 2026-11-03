//! PLAN-889 private native container primitives. No MIR/ABC, opcode dispatch,
//! source frontend, or AV executor lives here. Control flow and arithmetic
//! belong to generated machine code. All callers are verified generated code.
//! Text buffers use the delivered text shim; these nodes only retain owners.
#![no_std]
#![no_builtins]

// Embed the delivered text implementation and its single panic handler in
// this static library. Linking two independent no_std panic handlers conflicts.
extern crate ac_text_runtime;
mod arena;
mod physical;

use core::ptr::{null_mut, read, write};

extern "system" {
    fn GetStdHandle(which: i32) -> isize;
    fn WriteFile(h: isize, p: *const u8, n: u32, done: *mut u32, over: *mut u8) -> i32;
    fn ExitProcess(code: u32) -> !;
    fn ac_rt_own_from_lit(p: *const u8, len: i32, out: *mut i32) -> *mut u8;
    fn ac_rt_own_clone(p: *const u8, len: i32, out: *mut i32) -> *mut u8;
    fn ac_rt_own_free(p: *mut u8);
    fn ac_rt_text_scalars(p: *const u8, len: i32) -> i32;
    fn ac_rt_summary();
}

pub const ABI: &str = "mn.collections.runtime/3";
pub const GUEST_CAP: usize = 8 << 20;
pub const MANAGED_CAP: usize = 32 << 20;
pub const LIST_CAP: usize = 65536;
pub const INDEX: i32 = -21;
pub const RESOURCE: i32 = -22;
pub const INVALID: i32 = -23;
pub const VARIANT: i32 = -24;
pub const INJECTED: i32 = -25;
pub const CANCELLED: i32 = -26;
/// PLAN-923: a place token's list/epoch/element-generation no longer matches
/// (structural change after mint, or the target element was replaced).
pub const STALE: i32 = -27;
/// PLAN-923: a frame slot holding a place token (Cell.bits = token address).
/// Distinct from owned(0/1) so the frame sweep can retire the token.
pub const PLACE_MARK: u64 = 2;
// PLAN-923 seam ids (t02-design.md §2). 0 = legacy global seam (all
// allocations share one counter); S1..S8 count and inject per seam.
pub const SEAM_GLOBAL: u32 = 0;
pub const SEAM_NODE: u32 = 1;
pub const SEAM_DATA: u32 = 2;
pub const SEAM_GROW: u32 = 3;
pub const SEAM_FRAME: u32 = 4;
pub const SEAM_CLONEWORK: u32 = 5;
pub const SEAM_TEXT: u32 = 6;
pub const SEAM_TEXTMETA: u32 = 7;
pub const SEAM_TOKEN: u32 = 8;
pub const SEAM_COUNT: usize = 9;
static mut INSTALLED: bool = false;
static mut WORK: u64 = 0;
static mut CANCEL_AFTER: u64 = u64::MAX;

unsafe extern "system" fn text_alloc(bytes: usize) -> *mut core::ffi::c_void {
    arena::allocate(bytes).cast()
}
unsafe extern "system" fn text_free(p: *mut core::ffi::c_void) -> i32 {
    arena::release(p.cast()) as i32
}
unsafe extern "system" fn poll(work: u64) -> i32 {
    WORK = WORK.saturating_add(work);
    if WORK > CANCEL_AFTER {
        CANCELLED
    } else {
        0
    }
}
fn ensure_allocator() -> Result<(), i32> {
    physical::begin();
    unsafe {
        if !INSTALLED {
            if ac_text_runtime::ac_rt_install_allocator(
                Some(text_alloc),
                Some(text_free),
                Some(poll),
            ) != 0
            {
                return Err(INVALID);
            }
            INSTALLED = true;
        }
    }
    Ok(())
}
/// Deterministic private cancellation seam; cleanup itself never polls.
#[no_mangle]
pub unsafe extern "system" fn mn_coll_cancel_after_work(work: u64) {
    WORK = 0;
    CANCEL_AFTER = work;
}
#[no_mangle]
pub extern "system" fn mn_coll_arena_limit(bytes: u64) -> i32 {
    if arena::tighten_limit(bytes as usize) {
        0
    } else {
        INVALID
    }
}
#[no_mangle]
pub extern "system" fn mn_coll_arena_committed() -> u64 {
    arena::committed() as u64
}

/// Fixed machine cell: native scalar bits or an owning/borrowed node address.
/// `owned` is a destructor right, not a language type tag or an instruction.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Cell {
    pub bits: u64,
    pub owned: u64,
}
const EMPTY: Cell = Cell { bits: 0, owned: 0 };

/// The generated stack frame registers its real slot storage. Fatal cleanup
/// follows this chain including resource-bearing callers, without unwinding.
#[repr(C)]
pub struct Frame {
    prev: *mut Frame,
    cells: *mut Cell,
    count: usize,
    charge: usize,
    /// PLAN-919: activation identity (frame nesting events).
    seq: u64,
}

const TEXT: u32 = 1;
const RECORD: u32 = 2;
const LIST: u32 = 3;
const TAGGED: u32 = 4;
#[repr(C)]
struct Node {
    type_fp: [u64; 4],
    type_bound: bool,
    prev: *mut Node,
    next: *mut Node,
    drop_next: *mut Node,
    refs: usize,
    kind: u32,
    disc: u32,
    moved: bool,
    len: usize,
    cap: usize,
    data: *mut u8,
    payload: Cell,
    bytes: usize,
    guest: usize,
    /// PLAN-919: creation-order owner identity (the LIFO order key).
    id: u64,
    /// PLAN-923: structural epoch — push/insert/remove bump it; every place
    /// minted against an older epoch is stale before any buffer access.
    epoch: u64,
    /// PLAN-923: per-element generation array (cap entries, allocated with
    /// the data buffer). An in-place set/PlaceSet bumps the target element.
    gens: *mut u64,
}

/// PLAN-923: a minted place token lives in the arena registry (S8 seam).
/// It stores the list handle and the mint-time buffer/epoch/index/generation
/// snapshot; the buffer pointer is only dereferenced AFTER the registry
/// re-validation below (never a dangling frame/list/buffer pointer).
#[repr(C)]
struct PlaceToken {
    next: *mut PlaceToken,
    id: u64,
    list: *mut Node,
    data: *mut u8,
    epoch: u64,
    index: usize,
    elem_gen: u64,
}

struct State {
    head: *mut Node,
    frame: *mut Frame,
    managed: usize,
    guest: usize,
    peak: usize,
    nodes: usize,
    allocs: u64,
    frees: u64,
    invalid: u64,
    attempts: u64,
    fail_nth: u64,
    budget: usize,
    /// PLAN-923: per-seam injection. `fail_seam` 0 keeps the legacy global
    /// counter; S1..S8 use their own counters so a fault in one seam never
    /// consumes another seam's attempts (t02-design.md §2).
    fail_seam: u32,
    seam_attempts: [u64; SEAM_COUNT],
    /// PLAN-919: the bounded per-owner observation channel (off by
    /// default — zero behavior change until `mn_coll_observe_enable`).
    observe: bool,
    events: u64,
    ev_capped: bool,
    next_node_id: u64,
    frame_seq: u64,
    /// PLAN-923: the place-token registry (arena-backed, S8 seam).
    place_head: *mut PlaceToken,
    next_place_id: u64,
}
static mut STATE: State = State {
    head: null_mut(),
    frame: null_mut(),
    managed: 0,
    guest: 0,
    peak: 0,
    nodes: 0,
    allocs: 0,
    frees: 0,
    invalid: 0,
    attempts: 0,
    fail_nth: 0,
    budget: GUEST_CAP,
    fail_seam: 0,
    seam_attempts: [0; SEAM_COUNT],
    observe: false,
    events: 0,
    ev_capped: false,
    next_node_id: 0,
    frame_seq: 0,
    place_head: null_mut(),
    next_place_id: 0,
};
fn state() -> &'static mut State {
    unsafe { &mut *core::ptr::addr_of_mut!(STATE) }
}

fn precheck(bytes: usize, guest: usize) -> Result<(), i32> {
    let s = state();
    if s.managed
        .checked_add(bytes)
        .filter(|n| *n <= MANAGED_CAP)
        .is_none()
        || s.guest
            .checked_add(guest)
            .filter(|n| *n <= s.budget.min(GUEST_CAP))
            .is_none()
    {
        return Err(RESOURCE);
    }
    Ok(())
}
fn charge(bytes: usize, guest: usize) {
    let s = state();
    s.managed += bytes;
    s.guest += guest;
    s.peak = s.peak.max(s.managed);
}
fn uncharge(bytes: usize, guest: usize) {
    let s = state();
    s.managed -= bytes;
    s.guest -= guest;
}
fn allocate(bytes: usize, seam: u32) -> Result<*mut u8, i32> {
    ensure_allocator()?;
    let s = state();
    s.attempts += 1;
    let injected = if s.fail_seam == SEAM_GLOBAL {
        s.fail_nth != 0 && s.attempts == s.fail_nth
    } else {
        let counter = &mut s.seam_attempts[seam as usize];
        *counter += 1;
        s.fail_seam == seam && *counter == s.fail_nth
    };
    if injected {
        // PLAN-923: the injection is a named fact, not a silent OOM — even
        // where the caller collapses into a generic null path (frame-open),
        // the parent can still distinguish injection from a real budget
        // refusal (t02-design.md §2 error taxonomy).
        line(b"MN-COLL-INJECT seam=", seam as u64);
        line(b"MN-COLL-INJECT nth=", s.fail_nth);
        return Err(INJECTED);
    }
    let p = unsafe { arena::allocate(bytes) };
    if p.is_null() {
        Err(RESOURCE)
    } else {
        Ok(p)
    }
}
unsafe fn free(p: *mut u8) -> bool {
    if !p.is_null() {
        if !arena::release(p) {
            state().invalid += 1;
            return false;
        }
    }
    true
}

// ---- PLAN-919: the bounded per-owner observation channel ------------------
//
// Pure data on stderr: one `MN-EVT ...` line per event, a `MN-EVTEND
// total=` marker at the summary, and a HARD cap (`MN-COLL-EVT cap=` +
// abort 22) — a full log is a named refusal, never silent truncation.
// The channel is OFF until `mn_coll_observe_enable`; with it off no line
// is written and the execution is byte-identical.

const EVT_CAP: u64 = 8192;

#[no_mangle]
pub extern "system" fn mn_coll_observe_enable() {
    state().observe = true;
}

enum Seg {
    B(&'static [u8]),
    N(u64),
}

fn buf_put(buf: &mut [u8], n: &mut usize, b: &[u8]) {
    for &x in b {
        if *n < buf.len() {
            unsafe {
                write(buf.as_mut_ptr().add(*n), x);
            }
        }
        *n += 1;
    }
}

fn buf_put_num(buf: &mut [u8], n: &mut usize, v: u64) {
    let mut digits = [0u8; 20];
    let mut k = 0;
    let mut v = v;
    loop {
        unsafe {
            write(digits.as_mut_ptr().add(k), b'0' + (v % 10) as u8);
        }
        k += 1;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    while k > 0 {
        k -= 1;
        unsafe {
            write(buf.as_mut_ptr().add(*n), read(digits.as_ptr().add(k)));
        }
        *n += 1;
    }
}

fn write_out(buf: &[u8]) {
    let mut done = 0;
    unsafe {
        WriteFile(
            GetStdHandle(-12),
            buf.as_ptr(),
            buf.len() as u32,
            &mut done,
            null_mut(),
        );
    }
}

fn evt(segs: &[Seg]) -> bool {
    let s = state();
    if !s.observe || s.ev_capped {
        return true;
    }
    if s.events >= EVT_CAP {
        // Bounded observation: refuse, never truncate.
        s.ev_capped = true;
        line(b"MN-COLL-EVT cap=", EVT_CAP);
        return false;
    }
    s.events += 1;
    let seq = s.events;
    let mut buf = [0u8; 256];
    let mut n = 0;
    buf_put(&mut buf, &mut n, b"MN-EVT seq=");
    buf_put_num(&mut buf, &mut n, seq);
    for seg in segs {
        match seg {
            Seg::B(b) => buf_put(&mut buf, &mut n, b),
            Seg::N(v) => buf_put_num(&mut buf, &mut n, *v),
        }
    }
    buf_put(&mut buf, &mut n, b"\n");
    write_out(&buf[..n.min(buf.len())]);
    true
}

/// The frame/slot a cell lives in (NO_SLOT when node-internal).
unsafe fn locate(c: *const Cell) -> (u64, u32) {
    let heap_flag = 1usize << (usize::BITS - 1);
    let mut f = state().frame;
    while !f.is_null() {
        let base = (*f).cells;
        let count = (*f).count & !heap_flag;
        let off = (c as usize).saturating_sub(base as usize) / core::mem::size_of::<Cell>();
        if (c as usize) >= base as usize && off < count {
            return ((*f).seq, off as u32);
        }
        f = (*f).prev;
    }
    (0, u32::MAX)
}

fn kind_word(kind: u32) -> &'static [u8] {
    match kind {
        TEXT => b"text",
        RECORD => b"record",
        LIST => b"list",
        TAGGED => b"tagged",
        _ => b"?",
    }
}

unsafe fn cell_owner_id(cell: Cell) -> Option<(*mut Node, u64)> {
    if cell.owned != 1 || cell.bits == 0 {
        return None;
    }
    node(cell.bits).ok().map(|p| (p, (*p).id))
}

/// own event for the value now held by cell `c`.
unsafe fn ev_own(c: *const Cell, received: bool) {
    let s = state();
    if !s.observe {
        return;
    }
    let Some((p, id)) = cell_owner_id(read(c as *mut Cell)) else {
        return;
    };
    let (fr, slot) = locate(c);
    let via: &'static [u8] = if received { b"received" } else { b"created" };
    if !evt(&[
        Seg::B(b" act=own kind="),
        Seg::B(kind_word((*p).kind)),
        Seg::B(b" id="),
        Seg::N(id),
        Seg::B(b" key="),
        Seg::N(id),
        Seg::B(b" via="),
        Seg::B(via),
        Seg::B(b" frame="),
        Seg::N(fr),
        Seg::B(b" slot="),
        Seg::N(slot as u64),
    ]) {
        mn_coll_abort(RESOURCE);
    }
}

/// disown event for the value leaving cell `c`.
unsafe fn ev_disown(c: *const Cell, via: &'static [u8]) {
    let s = state();
    if !s.observe {
        return;
    }
    let Some((p, id)) = cell_owner_id(read(c as *mut Cell)) else {
        return;
    };
    let (fr, slot) = locate(c);
    if !evt(&[
        Seg::B(b" act=disown kind="),
        Seg::B(kind_word((*p).kind)),
        Seg::B(b" id="),
        Seg::N(id),
        Seg::B(b" key="),
        Seg::N(id),
        Seg::B(b" via="),
        Seg::B(via),
        Seg::B(b" frame="),
        Seg::N(fr),
        Seg::B(b" slot="),
        Seg::N(slot as u64),
    ]) {
        mn_coll_abort(RESOURCE);
    }
}

/// free fact for a destroyed node (the destruction order IS the free order).
unsafe fn ev_free(id: u64, bytes: usize) {
    let s = state();
    if !s.observe {
        return;
    }
    if !evt(&[
        Seg::B(b" act=free id="),
        Seg::N(id),
        Seg::B(b" bytes="),
        Seg::N(bytes as u64),
    ]) {
        mn_coll_abort(RESOURCE);
    }
}

/// Registry lookup precedes any dereference of a claimed node handle.
fn node(bits: u64) -> Result<*mut Node, i32> {
    let mut p = state().head;
    while !p.is_null() {
        if p as u64 == bits {
            return Ok(p);
        }
        p = unsafe { (*p).next };
    }
    Err(INVALID)
}
fn of_kind(bits: u64, kind: u32) -> Result<*mut Node, i32> {
    let p = node(bits)?;
    if unsafe { (*p).kind } != kind {
        Err(INVALID)
    } else {
        Ok(p)
    }
}
/// Descriptor words are constants emitted only after shared verification and
/// native layout admission. Never interpret a caller-provided descriptor pointer.
/// Mode1 binds a newly constructed node; mode0 requires an existing binding.
#[no_mangle]
pub unsafe extern "system" fn mn_coll_type(
    bits: u64,
    kind: u32,
    arity: u32,
    a: u64,
    b: u64,
    c: u64,
    d: u64,
    mode: u32,
) -> i32 {
    let Ok(p) = of_kind(bits, kind) else {
        line(b"MN-COLL-TYPE refusal=", 1);
        return INVALID;
    };
    if mode > 1 || !matches!(kind, TEXT | RECORD | LIST | TAGGED) {
        line(b"MN-COLL-TYPE refusal=", 2);
        return INVALID;
    }
    if (kind == RECORD && (*p).len != arity as usize)
        || (kind == LIST && arity != 1)
        || (kind == TAGGED && (*p).disc >= arity)
        || (kind == TEXT && arity != 0)
    {
        line(b"MN-COLL-TYPE refusal=", 3);
        return INVALID;
    }
    let fp = [a, b, c, d];
    if (*p).type_bound {
        if (*p).type_fp == fp {
            0
        } else {
            line(b"MN-COLL-TYPE refusal=", 4);
            INVALID
        }
    } else if mode == 1 {
        (*p).type_fp = fp;
        (*p).type_bound = true;
        0
    } else {
        line(b"MN-COLL-TYPE refusal=", 5);
        INVALID
    }
}
fn retain(c: Cell) -> Result<(), i32> {
    if c.owned > 1 {
        return Err(INVALID);
    }
    if c.owned == 1 && c.bits != 0 {
        let p = node(c.bits)?;
        unsafe {
            (*p).refs = (*p).refs.checked_add(1).ok_or(RESOURCE)?;
        }
    }
    Ok(())
}

fn validate(c: Cell) -> Result<(), i32> {
    match c.owned {
        0 => Ok(()),
        1 => node(c.bits).map(|_| ()),
        _ => Err(INVALID),
    }
}

/// Intrusive work stack: destruction never recurses or allocates. Nodes are
/// detached once when their last reference disappears, before traversing children.
fn release(c: Cell) {
    if c.owned == 0 || c.bits == 0 {
        return;
    }
    // PLAN-923: a place-token slot retires its token (never a node).
    if c.owned == PLACE_MARK {
        release_place(c.bits as *mut PlaceToken);
        return;
    }
    unsafe fn enqueue(c: Cell, stack: &mut *mut Node) {
        if c.owned == 0 || c.bits == 0 {
            return;
        }
        let Ok(p) = node(c.bits) else {
            state().invalid += 1;
            return;
        };
        if c.owned != 1 || (*p).refs == 0 {
            state().invalid += 1;
            return;
        }
        (*p).refs -= 1;
        if (*p).refs != 0 {
            return;
        }
        if (*p).prev.is_null() {
            state().head = (*p).next;
        } else {
            (*(*p).prev).next = (*p).next;
        }
        if !(*p).next.is_null() {
            (*(*p).next).prev = (*p).prev;
        }
        (*p).drop_next = *stack;
        *stack = p;
    }
    let mut stack = null_mut();
    unsafe {
        enqueue(c, &mut stack);
        while !stack.is_null() {
            let p = stack;
            stack = (*p).drop_next;
            match (*p).kind {
                TEXT => ac_rt_own_free((*p).data),
                RECORD | LIST => {
                    for i in 0..(*p).len {
                        enqueue(read((*p).data.cast::<Cell>().add(i)), &mut stack);
                    }
                    free((*p).data);
                }
                TAGGED => {
                    if !(*p).moved {
                        enqueue((*p).payload, &mut stack);
                    }
                }
                _ => {
                    state().invalid += 1;
                }
            }
            let gone_id = (*p).id;
            let gone_bytes = (*p).bytes;
            uncharge((*p).bytes, (*p).guest);
            free(p.cast());
            state().nodes -= 1;
            state().frees += 1;
            ev_free(gone_id, gone_bytes);
        }
    }
}
unsafe fn clear(c: *mut Cell) {
    clear_via(c, b"dropped");
}
unsafe fn clear_via(c: *mut Cell, via: &'static [u8]) {
    ev_disown(c, via);
    let old = read(c);
    write(c, EMPTY);
    release(old);
}
unsafe fn commit(dst: *mut Cell, c: Cell) {
    clear_via(dst, b"replaced");
    write(dst, c);
}
unsafe fn take(c: *mut Cell) -> Cell {
    let value = read(c);
    if value.owned != 0 {
        ev_disown(c, b"moved-out");
        write(c, EMPTY);
    }
    value
}

fn new_node(kind: u32, cap: usize, guest: usize) -> Result<*mut Node, i32> {
    let cell_bytes = if kind == LIST || kind == RECORD {
        cap.checked_mul(core::mem::size_of::<Cell>())
            .ok_or(RESOURCE)?
    } else {
        0
    };
    // PLAN-923: the per-element generation array rides the data buffer
    // (one allocation, same lifecycle; counted in managed bytes).
    let gens_bytes = if kind == LIST || kind == RECORD {
        cap.checked_mul(core::mem::size_of::<u64>())
            .ok_or(RESOURCE)?
    } else {
        0
    };
    let bytes = core::mem::size_of::<Node>()
        .checked_add(cell_bytes)
        .and_then(|n| n.checked_add(gens_bytes))
        .and_then(|n| n.checked_add(if kind == TEXT { guest } else { 0 }))
        .ok_or(RESOURCE)?;
    precheck(bytes, guest)?;
    let p = allocate(core::mem::size_of::<Node>(), SEAM_NODE)?.cast::<Node>();
    let (data, gens) = if cell_bytes + gens_bytes == 0 {
        (null_mut(), null_mut())
    } else {
        match allocate(cell_bytes + gens_bytes, SEAM_DATA) {
            Ok(buf) => {
                let gens = if gens_bytes == 0 {
                    null_mut()
                } else {
                    unsafe { buf.add(cell_bytes).cast() }
                };
                (buf, gens)
            }
            Err(e) => {
                unsafe {
                    free(p.cast());
                }
                return Err(e);
            }
        }
    };
    let id = {
        let s = state();
        s.next_node_id += 1;
        s.next_node_id
    };
    unsafe {
        write(
            p,
            Node {
                type_fp: [0; 4],
                type_bound: false,
                prev: null_mut(),
                next: state().head,
                drop_next: null_mut(),
                refs: 1,
                kind,
                disc: 0,
                moved: false,
                len: 0,
                cap,
                data,
                payload: EMPTY,
                bytes,
                guest,
                id,
                epoch: 0,
                gens,
            },
        );
        if !state().head.is_null() {
            (*state().head).prev = p;
        }
    }
    state().head = p;
    state().nodes += 1;
    state().allocs += 1;
    charge(bytes, guest);
    Ok(p)
}
fn owner(p: *mut Node) -> Cell {
    Cell {
        bits: p as u64,
        owned: 1,
    }
}
fn status(r: Result<(), i32>) -> i32 {
    r.err().unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "system" fn mn_coll_frame_enter(
    f: *mut Frame,
    cells: *mut Cell,
    count: u64,
) -> i32 {
    physical::begin();
    enter_frame(f, cells, count, false)
}
unsafe fn enter_frame(f: *mut Frame, cells: *mut Cell, count: u64, heap_reserved: bool) -> i32 {
    status((|| {
        let bytes = (count as usize)
            .checked_mul(16)
            .and_then(|n| n.checked_add(core::mem::size_of::<Frame>()))
            .ok_or(RESOURCE)?;
        precheck(bytes, 0)?;
        if !heap_reserved && !arena::hold_external(bytes) {
            return Err(RESOURCE);
        }
        let seq = {
            let s = state();
            s.frame_seq += 1;
            s.frame_seq
        };
        write(
            f,
            Frame {
                prev: state().frame,
                cells,
                count: count as usize
                    | if heap_reserved {
                        1usize << (usize::BITS - 1)
                    } else {
                        0
                    },
                charge: bytes,
                seq,
            },
        );
        state().frame = f;
        charge(bytes, 0);
        if !evt(&[
            Seg::B(b" act=frame-enter frame="),
            Seg::N(seq),
        ]) {
            return Err(RESOURCE);
        }
        Ok(())
    })())
}
#[no_mangle]
pub unsafe extern "system" fn mn_coll_frame_leave(f: *mut Frame) -> i32 {
    if state().frame != f {
        return INVALID;
    }
    let heap_flag = 1usize << (usize::BITS - 1);
    let count = (*f).count & !heap_flag;
    // PLAN-919: the sweep releases each owned slot in REVERSE slot order
    // (the emitted disowns carry the actual order); the frame-leave event
    // closes the activation.
    for i in (0..count).rev() {
        clear_via((*f).cells.add(i), b"swept");
    }
    let left_seq = (*f).seq;
    if !evt(&[
        Seg::B(b" act=frame-leave frame="),
        Seg::N(left_seq),
    ]) {
        mn_coll_abort(RESOURCE);
    }
    if (*f).count & heap_flag != 0 {
        free((*f).cells.cast());
    }
    arena::release_external(if (*f).count & heap_flag != 0 {
        core::mem::size_of::<Frame>()
    } else {
        (*f).charge
    });
    state().frame = (*f).prev;
    uncharge((*f).charge, 0);
    if state().frame.is_null() {
        // Refused while a leaked text owner exists; parent observations catch it.
        ac_text_runtime::ac_rt_release_idle_metadata();
    }
    0
}

/// Generated functions allocate their slot area under the managed budget,
/// rather than overflowing the native stack before a budget check can run.
#[no_mangle]
pub unsafe extern "system" fn mn_coll_frame_open(f: *mut Frame, count: u64) -> *mut Cell {
    let Some(bytes) = (count as usize).checked_mul(16) else {
        return null_mut();
    };
    let Some(total) = bytes.checked_add(core::mem::size_of::<Frame>()) else {
        return null_mut();
    };
    if precheck(total, 0).is_err() {
        return null_mut();
    }
    let header = core::mem::size_of::<Frame>();
    if !arena::hold_external(header) {
        return null_mut();
    }
    // An injected failure returns null here too — the emitted guard traps
    // RESOURCE — but the INJECT fact line above still distinguishes it from
    // a real budget refusal (PLAN-923 error taxonomy).
    let Ok(cells) = allocate(bytes, SEAM_FRAME) else {
        arena::release_external(header);
        return null_mut();
    };
    if enter_frame(f, cells.cast(), count, true) != 0 {
        free(cells);
        arena::release_external(header);
        return null_mut();
    }
    cells.cast()
}
#[no_mangle]
pub unsafe extern "system" fn mn_coll_move(dst: *mut Cell, src: *mut Cell) -> i32 {
    if dst == src {
        return 0;
    }
    if let Err(e) = validate(read(src)) {
        return e;
    }
    commit(dst, take(src));
    ev_own(dst, true);
    0
}
#[no_mangle]
pub unsafe extern "system" fn mn_coll_share(dst: *mut Cell, src: *const Cell) -> i32 {
    if dst.cast_const() == src {
        return 0;
    }
    let c = read(src);
    if let Err(e) = retain(c) {
        return e;
    }
    commit(dst, c);
    ev_own(dst, true);
    0
}
#[no_mangle]
pub unsafe extern "system" fn mn_coll_drop(c: *mut Cell) {
    clear(c);
}

#[no_mangle]
pub unsafe extern "system" fn mn_coll_literal(dst: *mut Cell, bytes: *const u8, len: i32) -> i32 {
    status((|| {
        if len < 0 {
            return Err(INVALID);
        }
        let p = new_node(TEXT, 0, len as usize)?;
        let mut actual = 0;
        let data = ac_rt_own_from_lit(bytes, len, &mut actual);
        if actual < 0 {
            release(owner(p));
            return Err(if actual == ac_text_runtime::ERR_CANCELLED {
                CANCELLED
            } else {
                RESOURCE
            });
        }
        (*p).data = data;
        (*p).len = actual as usize;
        commit(dst, owner(p));
        ev_own(dst, false);
        Ok(())
    })())
}
#[no_mangle]
pub unsafe extern "system" fn mn_coll_str_len(bits: u64) -> i32 {
    match of_kind(bits, TEXT) {
        Ok(p) => ac_rt_text_scalars((*p).data, (*p).len as i32),
        Err(e) => e,
    }
}

#[no_mangle]
pub unsafe extern "system" fn mn_coll_record_new(dst: *mut Cell, count: u64) -> i32 {
    status((|| {
        let p = new_node(
            RECORD,
            count as usize,
            (count as usize).checked_mul(16).ok_or(RESOURCE)?,
        )?;
        (*p).len = count as usize;
        commit(dst, owner(p));
        ev_own(dst, false);
        Ok(())
    })())
}
#[no_mangle]
pub unsafe extern "system" fn mn_coll_record_put(bits: u64, index: u64, src: *mut Cell) -> i32 {
    status((|| {
        let p = of_kind(bits, RECORD)?;
        if index as usize >= (*p).len {
            return Err(INDEX);
        }
        validate(read(src))?;
        commit((*p).data.cast::<Cell>().add(index as usize), take(src));
        Ok(())
    })())
}
#[no_mangle]
pub unsafe extern "system" fn mn_coll_field_get(dst: *mut Cell, bits: u64, index: u64) -> i32 {
    status((|| {
        let p = of_kind(bits, RECORD)?;
        if index as usize >= (*p).len {
            return Err(INDEX);
        }
        let c = read((*p).data.cast::<Cell>().add(index as usize));
        retain(c)?;
        commit(dst, c);
        ev_own(dst, true);
        Ok(())
    })())
}

#[no_mangle]
pub unsafe extern "system" fn mn_coll_list_new(dst: *mut Cell, cap: u64) -> i32 {
    status((|| {
        if cap as usize > LIST_CAP {
            return Err(RESOURCE);
        }
        let p = new_node(
            LIST,
            cap as usize,
            (cap as usize).checked_mul(16).ok_or(RESOURCE)?,
        )?;
        commit(dst, owner(p));
        ev_own(dst, false);
        Ok(())
    })())
}
#[no_mangle]
pub unsafe extern "system" fn mn_coll_list_len(bits: u64) -> i32 {
    match of_kind(bits, LIST) {
        Ok(p) => (*p).len as i32,
        Err(e) => e,
    }
}
#[no_mangle]
pub unsafe extern "system" fn mn_coll_list_insert(bits: u64, index: i32, src: *mut Cell) -> i32 {
    status((|| {
        let p = of_kind(bits, LIST)?;
        if index < 0 || index as usize > (*p).len {
            return Err(INDEX);
        }
        if (*p).len >= LIST_CAP {
            return Err(RESOURCE);
        }
        let input = read(src);
        if input.owned == 1 {
            node(input.bits)?;
        } else if input.owned != 0 {
            return Err(INVALID);
        }
        // Admission of bounded data work precedes mutation, keeping cancellation
        // atomic even when capacity growth and a shift are both required.
        let work = (*p).len - index as usize + 1 + if (*p).len == (*p).cap { (*p).len } else { 0 };
        if poll(work as u64) != 0 {
            return Err(CANCELLED);
        }
        if (*p).len == (*p).cap {
            let cap = (*p)
                .cap
                .max(1)
                .checked_mul(2)
                .ok_or(RESOURCE)?
                .min(LIST_CAP);
            let new_cells = cap.checked_mul(16).ok_or(RESOURCE)?;
            let new_gens = cap.checked_mul(8).ok_or(RESOURCE)?;
            let old_cells = (*p).cap * 16;
            let old_gens = (*p).cap * 8;
            // Charge the real simultaneous old + new allocation, not just its delta.
            precheck(new_cells + new_gens, new_cells + new_gens)?;
            let buffer = allocate(new_cells + new_gens, SEAM_GROW)?;
            charge(new_cells + new_gens, new_cells + new_gens);
            for i in 0..(*p).len {
                write(
                    buffer.cast::<Cell>().add(i),
                    read((*p).data.cast::<Cell>().add(i)),
                );
                write(
                    buffer.add(new_cells).cast::<u64>().add(i),
                    read((*p).gens.add(i)),
                );
            }
            free((*p).data);
            uncharge(old_cells + old_gens, old_cells + old_gens);
            (*p).data = buffer;
            (*p).gens = buffer.add(new_cells).cast();
            (*p).cap = cap;
            (*p).bytes += new_cells + new_gens - old_cells - old_gens;
            (*p).guest += new_cells + new_gens - old_cells - old_gens;
        }
        let data = (*p).data.cast::<Cell>();
        for i in (index as usize..(*p).len).rev() {
            write(data.add(i + 1), read(data.add(i)));
            write((*p).gens.add(i + 1), read((*p).gens.add(i)));
        }
        write(data.add(index as usize), take(src));
        write((*p).gens.add(index as usize), 0);
        (*p).len += 1;
        // PLAN-923: a structural change retires every minted place.
        (*p).epoch = (*p).epoch.wrapping_add(1);
        Ok(())
    })())
}
#[no_mangle]
pub unsafe extern "system" fn mn_coll_list_get(dst: *mut Cell, bits: u64, index: i32) -> i32 {
    status((|| {
        let p = of_kind(bits, LIST)?;
        if index < 0 || index as usize >= (*p).len {
            return Err(INDEX);
        }
        let c = read((*p).data.cast::<Cell>().add(index as usize));
        // The verified caller only requests Copy elements. A pure record is
        // physically an immutable node, so its value copy retains that node.
        retain(c)?;
        commit(dst, c);
        Ok(())
    })())
}

/// Transient projection base, valid only while the verified list root lives.
#[no_mangle]
pub unsafe extern "system" fn mn_coll_list_peek(dst: *mut Cell, bits: u64, index: i32) -> i32 {
    status((|| {
        let p = of_kind(bits, LIST)?;
        if index < 0 || index as usize >= (*p).len {
            return Err(INDEX);
        }
        let c = read((*p).data.cast::<Cell>().add(index as usize));
        commit(
            dst,
            Cell {
                bits: c.bits,
                owned: 0,
            },
        );
        Ok(())
    })())
}
#[no_mangle]
pub unsafe extern "system" fn mn_coll_list_remove(dst: *mut Cell, bits: u64, index: i32) -> i32 {
    status((|| {
        let p = of_kind(bits, LIST)?;
        if index < 0 || index as usize >= (*p).len {
            return Err(INDEX);
        }
        if poll(((*p).len - index as usize) as u64) != 0 {
            return Err(CANCELLED);
        }
        let data = (*p).data.cast::<Cell>();
        let c = read(data.add(index as usize));
        for i in index as usize + 1..(*p).len {
            write(data.add(i - 1), read(data.add(i)));
            write((*p).gens.add(i - 1), read((*p).gens.add(i)));
        }
        (*p).len -= 1;
        write(data.add((*p).len), EMPTY);
        write((*p).gens.add((*p).len), 0);
        commit(dst, c);
        ev_own(dst, true);
        // PLAN-923: a structural change retires every minted place.
        (*p).epoch = (*p).epoch.wrapping_add(1);
        Ok(())
    })())
}
#[no_mangle]
pub unsafe extern "system" fn mn_coll_list_set(bits: u64, index: i32, src: *mut Cell) -> i32 {
    status((|| {
        let p = of_kind(bits, LIST)?;
        if index < 0 || index as usize >= (*p).len {
            return Err(INDEX);
        }
        if (*src).owned == 1 {
            node((*src).bits)?;
        } else if (*src).owned != 0 {
            return Err(INVALID);
        }
        commit((*p).data.cast::<Cell>().add(index as usize), take(src));
        // PLAN-923: an in-place replacement bumps the element generation —
        // the element's own places (and every sibling minted before the
        // replacement) go stale; other elements stay legal.
        (*p).gens.add(index as usize).write(read((*p).gens.add(index as usize)) + 1);
        Ok(())
    })())
}

// ---- PLAN-923: checked List places (V-04/V-05) -----------------------------
//
// A place token is a registered snapshot (list handle + buffer identity +
// structural epoch + element index + element generation). Every access
// re-validates against the live node registry BEFORE touching the buffer:
// a stale token traps `STALE` with zero buffer reads, never a dangling
// dereference.

fn find_place(bits: u64) -> Result<*mut PlaceToken, i32> {
    let mut p = state().place_head;
    while !p.is_null() {
        if p as u64 == bits {
            return Ok(p);
        }
        p = unsafe { (*p).next };
    }
    Err(INVALID)
}

fn release_place(t: *mut PlaceToken) {
    if t.is_null() {
        return;
    }
    let mut p = state().place_head;
    let mut prev: *mut PlaceToken = null_mut();
    while !p.is_null() {
        if p == t {
            if prev.is_null() {
                state().place_head = unsafe { (*p).next };
            } else {
                unsafe {
                    (*prev).next = (*p).next;
                }
            }
            unsafe {
                free(p.cast());
            }
            uncharge(core::mem::size_of::<PlaceToken>(), 0);
            return;
        }
        prev = p;
        p = unsafe { (*p).next };
    }
}

/// Registry re-validation precedes ANY buffer dereference. A dead list, a
/// replaced buffer, a structural epoch change or an element replacement all
/// refuse with STALE before any element read/write.
unsafe fn validate_place(t: *mut PlaceToken) -> Result<(), i32> {
    let list = node((*t).list as u64).map_err(|_| STALE)?;
    if (*list).kind != LIST {
        return Err(INVALID);
    }
    if (*list).data != (*t).data || (*list).epoch != (*t).epoch {
        return Err(STALE);
    }
    if (*t).index >= (*list).len {
        return Err(STALE);
    }
    if read((*list).gens.add((*t).index)) != (*t).elem_gen {
        return Err(STALE);
    }
    Ok(())
}

/// MINT_LIST_PLACE: snapshot `list[index]` into the place slot `dst`.
#[no_mangle]
pub unsafe extern "system" fn mn_coll_place_mint(dst: *mut Cell, list_bits: u64, index: i32) -> i32 {
    status((|| {
        let p = of_kind(list_bits, LIST)?;
        if index < 0 || index as usize >= (*p).len {
            return Err(INDEX);
        }
        let size = core::mem::size_of::<PlaceToken>();
        precheck(size, 0)?;
        let t = allocate(size, SEAM_TOKEN)?.cast::<PlaceToken>();
        charge(size, 0);
        let id = {
            let s = state();
            s.next_place_id += 1;
            s.next_place_id
        };
        write(
            t,
            PlaceToken {
                next: state().place_head,
                id,
                list: p,
                data: (*p).data,
                epoch: (*p).epoch,
                index: index as usize,
                elem_gen: read((*p).gens.add(index as usize)),
            },
        );
        state().place_head = t;
        commit(
            dst,
            Cell {
                bits: t as u64,
                owned: PLACE_MARK,
            },
        );
        Ok(())
    })())
}

/// PLACE_GET: read the Copy element a place refers to (verified caller
/// guarantees the element type is Copy-only).
#[no_mangle]
pub unsafe extern "system" fn mn_coll_place_get(dst: *mut Cell, place_bits: u64) -> i32 {
    status((|| {
        let t = find_place(place_bits)?;
        validate_place(t)?;
        let c = read((*t).list.as_ref().unwrap().data.cast::<Cell>().add((*t).index));
        commit(dst, c);
        Ok(())
    })())
}

/// PLACE_SET: replace the element a place refers to; the replacement bumps
/// the element generation (retiring this place and every sibling minted
/// before it) and retires this token.
#[no_mangle]
pub unsafe extern "system" fn mn_coll_place_set(place: *mut Cell, value: *mut Cell) -> i32 {
    status((|| {
        let t = find_place((*place).bits)?;
        validate_place(t)?;
        let p = (*t).list;
        let v = read(value);
        if v.owned == 1 {
            node(v.bits)?;
        } else if v.owned != 0 {
            return Err(INVALID);
        }
        commit(p.as_ref().unwrap().data.cast::<Cell>().add((*t).index), v);
        let gen = read((*p).gens.add((*t).index)) + 1;
        write((*p).gens.add((*t).index), gen);
        release_place(t);
        write(place, EMPTY);
        Ok(())
    })())
}

#[no_mangle]
pub unsafe extern "system" fn mn_coll_variant_pack(
    dst: *mut Cell,
    disc: u64,
    payload: *mut Cell,
) -> i32 {
    status((|| {
        if !payload.is_null() {
            validate(read(payload))?;
        }
        let p = new_node(TAGGED, 0, 16)?;
        (*p).disc = disc as u32;
        if !payload.is_null() {
            (*p).payload = take(payload);
        }
        commit(dst, owner(p));
        ev_own(dst, false);
        Ok(())
    })())
}

/// Clone only one node's storage. Child owning cells start empty until the
/// explicit work stack attaches their clones; every partial tree is droppable.
unsafe fn clone_shallow(source: *mut Node) -> Result<*mut Node, i32> {
    if poll(1) != 0 {
        return Err(CANCELLED);
    }
    let dest = new_node((*source).kind, (*source).cap, (*source).guest)?;
    (*dest).type_fp = (*source).type_fp;
    (*dest).type_bound = (*source).type_bound;
    (*dest).len = (*source).len;
    (*dest).disc = (*source).disc;
    (*dest).moved = (*source).moved;
    if (*source).kind == TEXT {
        let mut length = 0;
        let data = ac_rt_own_clone((*source).data, (*source).len as i32, &mut length);
        if length < 0 {
            release(owner(dest));
            return Err(if length == ac_text_runtime::ERR_CANCELLED {
                CANCELLED
            } else {
                RESOURCE
            });
        }
        (*dest).data = data;
    }
    Ok(dest)
}

#[repr(C)]
struct CloneWork {
    next: *mut CloneWork,
    source: *mut Node,
    dest: *mut Node,
    index: usize,
}

unsafe fn clone_child(p: *mut Node, index: usize) -> *mut Cell {
    if (*p).kind == TAGGED {
        core::ptr::addr_of_mut!((*p).payload)
    } else {
        (*p).data.cast::<Cell>().add(index)
    }
}
unsafe fn child_count(p: *mut Node) -> usize {
    match (*p).kind {
        LIST | RECORD => (*p).len,
        TAGGED if !(*p).moved => 1,
        _ => 0,
    }
}
unsafe fn push_clone_work(
    stack: &mut *mut CloneWork,
    source: *mut Node,
    dest: *mut Node,
) -> Result<(), i32> {
    if child_count(source) == 0 {
        return Ok(());
    }
    let size = core::mem::size_of::<CloneWork>();
    precheck(size, 0)?;
    let work = allocate(size, SEAM_CLONEWORK)?.cast::<CloneWork>();
    charge(size, 0);
    write(
        work,
        CloneWork {
            next: *stack,
            source,
            dest,
            index: 0,
        },
    );
    *stack = work;
    Ok(())
}
unsafe fn pop_clone_work(stack: &mut *mut CloneWork) {
    let old = *stack;
    *stack = (*old).next;
    free(old.cast());
    uncharge(core::mem::size_of::<CloneWork>(), 0);
}

#[no_mangle]
pub unsafe extern "system" fn mn_coll_clone(dst: *mut Cell, src: *const Cell) -> i32 {
    status((|| {
        let input = read(src);
        validate(input)?;
        if input.owned == 0 {
            commit(dst, input);
            return Ok(());
        }
        let root = node(input.bits)?;
        let output = clone_shallow(root)?;
        let mut stack = null_mut();
        let result = (|| {
            push_clone_work(&mut stack, root, output)?;
            while !stack.is_null() {
                if poll(1) != 0 {
                    return Err(CANCELLED);
                }
                let w = stack;
                if (*w).index == child_count((*w).source) {
                    pop_clone_work(&mut stack);
                    continue;
                }
                let index = (*w).index;
                (*w).index += 1;
                let child = read(clone_child((*w).source, index));
                validate(child)?;
                if child.owned == 0 {
                    write(clone_child((*w).dest, index), child);
                } else {
                    let source = node(child.bits)?;
                    let dest = clone_shallow(source)?;
                    write(clone_child((*w).dest, index), owner(dest));
                    push_clone_work(&mut stack, source, dest)?;
                }
            }
            Ok(())
        })();
        while !stack.is_null() {
            pop_clone_work(&mut stack);
        }
        if let Err(e) = result {
            release(owner(output));
            return Err(e);
        }
        commit(dst, owner(output));
        ev_own(dst, false);
        Ok(())
    })())
}
#[no_mangle]
pub unsafe extern "system" fn mn_coll_variant_disc(bits: u64) -> i32 {
    match of_kind(bits, TAGGED) {
        Ok(p) => (*p).disc as i32,
        Err(e) => e,
    }
}
#[no_mangle]
pub unsafe extern "system" fn mn_coll_variant_project(
    dst: *mut Cell,
    bits: u64,
    disc: u64,
    moving: u64,
) -> i32 {
    status((|| {
        let p = of_kind(bits, TAGGED)?;
        if (*p).disc != disc as u32 || (*p).moved {
            return Err(VARIANT);
        }
        let c = (*p).payload;
        if moving != 0 {
            (*p).payload = EMPTY;
            (*p).moved = true;
        } else {
            retain(c)?;
        }
        commit(dst, c);
        ev_own(dst, true);
        Ok(())
    })())
}

#[no_mangle]
pub extern "system" fn mn_coll_fail_nth(n: u64) {
    let s = state();
    s.attempts = 0;
    s.fail_seam = SEAM_GLOBAL;
    s.fail_nth = n;
    s.seam_attempts = [0; SEAM_COUNT];
}
/// PLAN-923: inject on exactly one seam's own attempt counter. S1..S8 are
/// independent, so a fault on one seam never consumes another's attempts.
#[no_mangle]
pub extern "system" fn mn_coll_fail_nth_seam(seam: u32, n: u64) {
    let s = state();
    s.attempts = 0;
    s.fail_seam = seam;
    s.fail_nth = n;
    s.seam_attempts = [0; SEAM_COUNT];
}
#[no_mangle]
pub extern "system" fn mn_coll_set_budget(bytes: u64) {
    state().budget = (bytes as usize).min(GUEST_CAP);
}
#[no_mangle]
pub extern "system" fn mn_coll_live_nodes() -> u64 {
    state().nodes as u64
}
#[no_mangle]
pub extern "system" fn mn_coll_live_bytes() -> u64 {
    state().managed as u64
}

fn line(prefix: &[u8], value: u64) {
    // All prefixes are private constants. Keep output bounded without the
    // general Rust formatting/panic machinery (the PE links kernel32 only).
    if prefix.len() > 100 {
        return;
    }
    let mut buf = [0u8; 128];
    let mut n = 0;
    for &b in prefix {
        unsafe {
            write(buf.as_mut_ptr().add(n), b);
        }
        n += 1;
    }
    let mut digits = [0u8; 20];
    let mut k = 0;
    let mut v = value;
    loop {
        unsafe {
            write(digits.as_mut_ptr().add(k), b'0' + (v % 10) as u8);
        }
        k += 1;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    while k > 0 {
        k -= 1;
        unsafe {
            write(buf.as_mut_ptr().add(n), read(digits.as_ptr().add(k)));
        }
        n += 1;
    }
    unsafe {
        write(buf.as_mut_ptr().add(n), b'\n');
    }
    n += 1;
    let mut done = 0;
    unsafe {
        WriteFile(
            GetStdHandle(-12),
            buf.as_ptr(),
            n as u32,
            &mut done,
            null_mut(),
        );
    }
}
#[no_mangle]
pub extern "system" fn mn_coll_summary() {
    let process = physical::finish();
    line(b"MN-COLL process_private_base=", process.base);
    line(b"MN-COLL process_private_end=", process.current);
    line(b"MN-COLL process_commit_peak=", process.peak_commit);
    line(b"MN-COLL process_commit_delta=", process.delta);
    line(b"MN-COLL process_sample_ok=", process.ok as u64);
    line(b"MN-COLL live=", state().nodes as u64);
    line(b"MN-COLL bytes=", state().managed as u64);
    line(b"MN-COLL guest=", state().guest as u64);
    line(b"MN-COLL peak=", state().peak as u64);
    line(b"MN-COLL allocs=", state().allocs);
    line(b"MN-COLL frees=", state().frees);
    line(b"MN-COLL invalid=", state().invalid);
    line(b"MN-COLL arena_live=", arena::live() as u64);
    line(b"MN-COLL arena_commit=", arena::committed() as u64);
    line(b"MN-COLL arena_peak=", arena::peak() as u64);
    line(b"MN-COLL arena_invalid=", arena::invalid());
    line(b"MN-COLL registered_bytes=", arena::registered() as u64);
    line(b"MN-COLL managed_commit_peak=", arena::domain_peak() as u64);
    line(
        b"MN-COLL text_metadata=",
        ac_text_runtime::ac_rt_metadata_bytes() as u64,
    );
    if state().observe {
        line(b"MN-EVTEND total=", state().events);
    }
    unsafe {
        ac_rt_summary();
    }
}
#[no_mangle]
pub unsafe extern "system" fn mn_coll_abort(code: i32) -> ! {
    while !state().frame.is_null() {
        mn_coll_frame_leave(state().frame);
    }
    line(b"MN-COLL-TRAP code=", code.unsigned_abs() as u64);
    mn_coll_summary();
    ExitProcess(70)
}
