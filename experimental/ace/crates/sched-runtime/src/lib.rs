//! PLAN-1067 — the native task-scheduler runtime (callee side of the
//! `mn.sched.runtime/1` ABI).
//!
//! The M2-B native lane executes the SAME scheduler surface PLAN-968 froze
//! for AV / A2R: `task` actor drivers, `Task.spawn`, `send`/`ask`/`reply`,
//! `await`, `go`, `sleep` and `stop`. This crate is the real machine side of
//! that surface for the MIR-native backend: one OS thread per task chain,
//! bounded FIFO mailboxes with sender backpressure (a full mailbox parks the
//! sender), single-consumer ask/reply ports, and an orderly exit drain
//! (stop flags → buffered messages still deliver → stop hooks run → every
//! thread joins → every allocation is released).
//!
//! Identity/constraints:
//! * `no_std` and kernel32-only, exactly like the other mn support libraries
//!   (`ace-text-runtime`, `ace-collections-runtime`): the produced PE keeps
//!   its CRT-free import set. The panic handler exits the process, so no
//!   unwinding can cross the ABI.
//! * Synchronization is `SRWLOCK` + `CONDITION_VARIABLE`, both valid when
//!   zero-initialized, so the whole registry is const-static and needs no
//!   runtime initialization.
//! * Every heap block is charged to `LIVE_ALLOCS`; `mn_sched_drain` releases
//!   the whole registry and returns the residue. A non-zero residue is the
//!   named `sched.leak` exit the lane wrapper reports — it is never silently
//!   swallowed.
//! * Language errors are NOT success sentinels: `mn_sched_fail(code)` writes
//!   the frozen message to stderr and exits with the native trap code (70),
//!   the same classification `fail(...)`/`.?` use on this lane.

#![no_std]
#![allow(non_snake_case)]

use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};

// The lane links exactly ONE Rust support staticlib, so this archive must
// also carry the freestanding mem* intrinsics core lowers to (memcpy /
// memmove / memset / memcmp). They are owned by the embedded owned-text
// runtime; referencing them here keeps their objects in the archive — a
// second Rust staticlib in the same link would collide on the panic/unwind
// runtime symbols.
type CopyFn = unsafe extern "C" fn(*mut u8, *const u8, usize) -> *mut u8;
type SetFn = unsafe extern "C" fn(*mut u8, i32, usize) -> *mut u8;
type CmpFn = unsafe extern "C" fn(*const u8, *const u8, usize) -> i32;

#[used]
static KEEP_MEMCPY: [CopyFn; 1] = [ac_text_runtime::memcpy];
#[used]
static KEEP_MEMMOVE: [CopyFn; 1] = [ac_text_runtime::memmove];
#[used]
static KEEP_MEMSET: [SetFn; 1] = [ac_text_runtime::memset];
#[used]
static KEEP_MEMCMP: [CmpFn; 1] = [ac_text_runtime::memcmp];

// No local #[panic_handler]: this runtime IS the lane's single Rust staticlib
// and the embedded owned-text runtime owns the one panic_impl. Its handler
// spins (documented upstream: the shims never panic by construction); a
// runaway panic is therefore contained by the executor's run budget instead
// of unwinding across the ABI — never a silent success code.

// ---------------------------------------------------------------------------
// kernel32 surface (the only OS dependency; same set the other runtimes use)
// ---------------------------------------------------------------------------

const HEAP_ZERO_MEMORY: u32 = 0x0000_0008;
const INFINITE: u32 = 0xFFFF_FFFF;
const STD_ERROR_HANDLE: i32 = -12;
const EXIT_TRAP: u32 = 70;

#[repr(C)]
pub struct SrwLock {
    ptr: usize,
}

#[repr(C)]
pub struct CondVar {
    ptr: usize,
}

extern "system" {
    fn GetStdHandle(id: i32) -> isize;
    fn WriteFile(
        file: isize,
        buffer: *const u8,
        bytes: u32,
        written: *mut u32,
        overlapped: *mut c_void,
    ) -> i32;
    fn GetProcessHeap() -> isize;
    fn HeapAlloc(heap: isize, flags: u32, bytes: usize) -> *mut c_void;
    fn HeapFree(heap: isize, flags: u32, ptr: *mut c_void) -> i32;
    fn CreateThread(
        attributes: *mut c_void,
        stack_size: usize,
        start: extern "system" fn(*mut c_void) -> u32,
        parameter: *mut c_void,
        flags: u32,
        thread_id: *mut u32,
    ) -> *mut c_void;
    fn WaitForSingleObject(handle: *mut c_void, ms: u32) -> u32;
    fn CloseHandle(handle: *mut c_void) -> i32;
    fn Sleep(ms: u32);
    fn TlsAlloc() -> u32;
    fn TlsSetValue(index: u32, value: *mut c_void) -> i32;
    fn TlsGetValue(index: u32) -> *mut c_void;
    fn AcquireSRWLockExclusive(lock: *mut SrwLock);
    fn ReleaseSRWLockExclusive(lock: *mut SrwLock);
    fn SleepConditionVariableSRW(cv: *mut CondVar, lock: *mut SrwLock, ms: u32, flags: u32) -> i32;
    fn WakeConditionVariable(cv: *mut CondVar);
    fn WakeAllConditionVariable(cv: *mut CondVar);
    fn ExitProcess(code: u32) -> !;
}

// ---------------------------------------------------------------------------
// frozen value/status surface (mn.sched.runtime/1)
// ---------------------------------------------------------------------------

/// One scheduler value. `tag`: 0 = i32, 1 = bool, 2 = owned text (ptr/len in
/// `a`/`b`; the native lane materializes the bytes into an owned-text-runtime
/// buffer at the handoff, so the RECEIVER frees it with `ac_rt_own_free`).
/// The three-word shape keeps the ABI open for the typed/payload surface
/// without a second entry set.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SchedVal {
    pub tag: i64,
    pub a: i64,
    pub b: i64,
}

impl SchedVal {
    const fn zero() -> Self {
        SchedVal { tag: 0, a: 0, b: 0 }
    }
}

// ---------------------------------------------------------------------------
// value materialization (PLAN-1067 T-03)
// ---------------------------------------------------------------------------

/// The scheduler handoff follows the frozen AV/A2R rule: text crossing a
/// task boundary materializes into an OWNED buffer (copy semantics — the
/// sender keeps its own value). Materializing goes through the embedded
/// owned-text runtime's allocator, which is exactly the allocator the guest
/// frees with (`ac_rt_own_free`); scalars pass through untouched.
unsafe fn materialize(val: *const SchedVal) -> Option<SchedVal> {
    let value = core::ptr::read_volatile(val);
    match value.tag {
        0 | 1 => Some(value),
        2 => {
            if value.b < 0 || (value.b > 0 && value.a == 0) {
                return None;
            }
            let mut out_len: i32 = 0;
            let ptr = ac_text_runtime::ac_rt_own_clone_quiet(
                value.a as *const u8,
                value.b as i32,
                &mut out_len,
            );
            if ptr.is_null() && out_len != 0 {
                // Allocation/cancel failure: the negative status is already
                // in `out_len`; the lane reports the named resource error.
                return None;
            }
            Some(SchedVal {
                tag: 2,
                a: ptr as i64,
                b: out_len as i64,
            })
        }
        _ => None,
    }
}

/// Release every materialized initializer plus its array block (failure
/// paths only — on success the driver owns the values and frees them).
unsafe fn release_args(argc: i32, copied: *mut SchedVal) {
    if copied.is_null() {
        return;
    }
    for i in 0..argc.max(0) as usize {
        release_value(core::ptr::read_volatile(copied.add(i)));
    }
    free(copied as *mut u8);
}
/// Release one materialized value that never reached a consumer (failed
/// enqueue, dropped late reply, undelivered drain residue).
unsafe fn release_value(value: SchedVal) {
    if value.tag == 2 && value.a != 0 {
        ac_text_runtime::ac_rt_own_free(value.a as *mut u8);
    }
}

/// Frozen status codes (negative = named failure the lane turns into a
/// language error / trap; never a silent success).
pub const E_HANDLE: i32 = -1;
/// A task that is stopped/closed refuses new deliveries (the AV/A2R text).
pub const E_CLOSED: i32 = -2;
pub const E_CAPACITY: i32 = -3;
pub const E_RESOURCE: i32 = -4;
pub const E_REPLY: i32 = -5;
pub const E_AWAIT_CONSUMED: i32 = -6;
pub const E_DRAIN: i32 = -7;
pub const E_VALUE: i32 = -8;
pub const E_TASK_ONLY: i32 = -9;
/// PLAN-1067 T-05: the driver closed without publishing the reply its
/// ask future is waiting for — a named terminal outcome, never a hang.
pub const E_REPLY_CLOSED: i32 = -10;

/// The named message of one failure code (byte-identical in the test
/// corpus assertions; `E_CLOSED` keeps the AV wording).
fn fail_message(code: i32) -> &'static [u8] {
    match code {
        E_HANDLE => b"sched: unknown task handle\n",
        E_CLOSED => b"sched: send on a closed task\n",
        E_CAPACITY => b"sched: task mailbox capacity must be at least 1\n",
        E_RESOURCE => b"sched: scheduler resource limit exceeded\n",
        E_REPLY => b"sched: reply is only legal while handling an ask message\n",
        E_AWAIT_CONSUMED => b"sched: await on an already-consumed future\n",
        E_DRAIN => b"sched: await cannot complete during the orderly drain\n",
        E_VALUE => b"sched: unsupported scheduler message value\n",
        E_TASK_ONLY => b"sched: scheduler operation outside a task driver\n",
        E_REPLY_CLOSED => b"sched: task closed without replying\n",
        _ => b"sched: scheduler runtime failure\n",
    }
}

// ---------------------------------------------------------------------------
// registry
// ---------------------------------------------------------------------------

const MAX_TASKS: usize = 256;
const MAX_PORTS: usize = 1024;
const MAX_GO: usize = 256;

#[repr(C)]
struct Mail {
    val: SchedVal,
    port: i32,
}

#[repr(C)]
struct Task {
    cap: u32,
    head: u32,
    len: u32,
    /// 1 while the driver thread is alive.
    running: i32,
    stopped: i32,
    closed: i32,
    /// Set by recv on a REAL delivery; masks `stopping()` for exactly one
    /// check (the a2r handoff order: the message dispatches first).
    recv_real: i32,
    /// The reply port of the message most recently received by this driver.
    reply_port: i32,
    /// PLAN-1067 review R-1067-1: the drain joined this chain (one join per
    /// chain; the drain loop re-snapshots until every chain carries it).
    joined: i32,
    /// PLAN-1067 review2 N-1067-1: the drain already delivered this chain its
    /// stop signal. Set in the SAME cycle that joins it, so a chain created
    /// while the drain is running is never joined while still parked (the old
    /// `thread.is_null()` guard was never true for a live task).
    stopped_marked: i32,
    queue: *mut Mail,
    thread: *mut c_void,
    lock: SrwLock,
    cv_not_empty: CondVar,
    cv_not_full: CondVar,
}

#[repr(C)]
struct Port {
    /// 0 pending, 1 published, 2 consumed, 3 closed-without-reply.
    state: u32,
    /// The drain already retired this never-published future (keeps the
    /// re-snapshot loop monotone).
    closed_marked: i32,
    val: SchedVal,
    lock: SrwLock,
    cv: CondVar,
}

#[repr(C)]
struct ThreadArg {
    task: *mut Task,
    thunk: i64,
    args: *mut SchedVal,
    argc: i32,
}

static mut REG_LOCK: SrwLock = SrwLock { ptr: 0 };
static mut TASKS: [*mut Task; MAX_TASKS] = [null_mut(); MAX_TASKS];
static mut TASK_COUNT: u32 = 0;
static mut PORTS: [*mut Port; MAX_PORTS] = [null_mut(); MAX_PORTS];
static mut PORT_COUNT: u32 = 0;
static mut GO_HANDLES: [*mut c_void; MAX_GO] = [null_mut(); MAX_GO];
/// PLAN-1067 review R-1067-1: which `go` slots the drain already joined.
static mut GO_JOINED: [bool; MAX_GO] = [false; MAX_GO];
static mut GO_COUNT: u32 = 0;
/// Heap blocks currently owned by the runtime (tasks, queues, ports, thread
/// args, arg copies). Zero after an orderly drain; anything else is a leak.
static LIVE_ALLOCS: AtomicI32 = AtomicI32::new(0);
static DRAINING: AtomicI32 = AtomicI32::new(0);
static TLS_INDEX: AtomicU32 = AtomicU32::new(u32::MAX);

unsafe fn alloc(bytes: usize) -> *mut u8 {
    if bytes == 0 {
        return null_mut();
    }
    let p = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, bytes) as *mut u8;
    if !p.is_null() {
        LIVE_ALLOCS.fetch_add(1, Ordering::SeqCst);
    }
    p
}

unsafe fn free(ptr: *mut u8) {
    if !ptr.is_null() {
        HeapFree(GetProcessHeap(), 0, ptr as *mut c_void);
        LIVE_ALLOCS.fetch_sub(1, Ordering::SeqCst);
    }
}

unsafe fn reg_lock() {
    AcquireSRWLockExclusive(addr_of_mut!(REG_LOCK));
}

unsafe fn reg_unlock() {
    ReleaseSRWLockExclusive(addr_of_mut!(REG_LOCK));
}

unsafe fn tls_index() -> u32 {
    let current = TLS_INDEX.load(Ordering::SeqCst);
    if current != u32::MAX {
        return current;
    }
    let allocated = TlsAlloc();
    match TLS_INDEX.compare_exchange(u32::MAX, allocated, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(_) => allocated,
        Err(winner) => winner,
    }
}

unsafe fn set_current_task(id: i32) {
    let index = tls_index();
    if index != u32::MAX {
        TlsSetValue(index, (id as isize) as *mut c_void);
    }
}

unsafe fn current_task() -> *mut Task {
    let index = TLS_INDEX.load(Ordering::SeqCst);
    if index == u32::MAX {
        return null_mut();
    }
    let raw = TlsGetValue(index) as isize;
    if raw <= 0 {
        return null_mut();
    }
    let slot = (raw - 1) as usize;
    let tasks = &*addr_of_mut!(TASKS);
    if slot < MAX_TASKS {
        tasks[slot]
    } else {
        null_mut()
    }
}

unsafe fn task_of(handle: i32) -> *mut Task {
    if handle <= 0 {
        return null_mut();
    }
    let slot = (handle - 1) as usize;
    if slot >= MAX_TASKS {
        return null_mut();
    }
    let tasks = &*addr_of_mut!(TASKS);
    tasks[slot]
}

unsafe fn port_of(id: i32) -> *mut Port {
    if id <= 0 {
        return null_mut();
    }
    let slot = (id - 1) as usize;
    if slot >= MAX_PORTS {
        return null_mut();
    }
    let ports = &*addr_of_mut!(PORTS);
    ports[slot]
}

unsafe fn wake_all(cv: *mut CondVar) {
    WakeAllConditionVariable(cv);
}

unsafe fn wake_one(cv: *mut CondVar) {
    WakeConditionVariable(cv);
}

/// Copy + materialize one initializer list into a runtime-owned `SchedVal`
/// array (owned text is copied into the owned-text allocator). `None` is the
/// named value/resource rejection; every partially materialized value is
/// released before returning.
unsafe fn materialize_args(argc: i32, args: *const SchedVal) -> Option<*mut SchedVal> {
    if argc < 0 || (argc > 0 && args.is_null()) {
        return None;
    }
    let bytes = (argc as usize) * core::mem::size_of::<SchedVal>();
    if bytes == 0 {
        return Some(null_mut());
    }
    let copied = alloc(bytes) as *mut SchedVal;
    if copied.is_null() {
        return None;
    }
    for i in 0..argc as usize {
        match materialize(args.add(i)) {
            Some(value) => core::ptr::write_volatile(copied.add(i), value),
            None => {
                for j in 0..i {
                    release_value(core::ptr::read_volatile(copied.add(j)));
                }
                free(copied as *mut u8);
                return None;
            }
        }
    }
    Some(copied)
}
/// Start one task chain: register the mailbox + handle, then run the driver
/// on a real OS thread. `handle` is 1-based (0 is the "no handle" sentinel).
unsafe fn start_task(capacity: i32, thunk: i64, argc: i32, args: *const SchedVal) -> i32 {
    // PLAN-1067 review R-1067-3: capacity 0 used to park every sender
    // forever (the `0 < 0` comparison can never free space). One mailbox slot
    // is the minimum the frozen "bounded mailbox" contract can offer.
    if capacity < 1 {
        return E_CAPACITY;
    }
    if argc < 0 || (argc > 0 && args.is_null()) {
        return E_VALUE;
    }
    if thunk == 0 {
        return E_RESOURCE;
    }
    let copied = match materialize_args(argc, args) {
        Some(copied) => copied,
        None => return E_VALUE,
    };
    let mail_bytes = (capacity as usize) * core::mem::size_of::<Mail>();
    let queue = alloc(mail_bytes) as *mut Mail;
    if capacity > 0 && queue.is_null() {
        free(copied as *mut u8);
        return E_RESOURCE;
    }
    let task = alloc(core::mem::size_of::<Task>()) as *mut Task;
    if task.is_null() {
        free(copied as *mut u8);
        free(queue as *mut u8);
        return E_RESOURCE;
    }
    (*task).cap = capacity as u32;
    (*task).queue = queue;
    (*task).running = 1;
    let ctx = alloc(core::mem::size_of::<ThreadArg>()) as *mut ThreadArg;
    if ctx.is_null() {
        free(task as *mut u8);
        free(queue as *mut u8);
        free(copied as *mut u8);
        return E_RESOURCE;
    }
    (*ctx).task = task;
    (*ctx).thunk = thunk;
    (*ctx).args = copied;
    (*ctx).argc = argc;

    reg_lock();
    let tasks = &mut *addr_of_mut!(TASKS);
    let mut slot = usize::MAX;
    for i in 0..MAX_TASKS {
        if tasks[i].is_null() {
            slot = i;
            break;
        }
    }
    if slot == usize::MAX {
        reg_unlock();
        free(ctx as *mut u8);
        free(task as *mut u8);
        free(queue as *mut u8);
        release_args(argc, copied);
        return E_RESOURCE;
    }
    // PLAN-1067 review R-1067-1: the task record AND its thread handle are
    // published while the registry lock is still held. `mn_sched_drain`
    // snapshots the same table under that lock, so a chain can never be
    // registered-but-unjoined (which used to leak the handle and report a
    // spurious drain residue / exit 72).
    tasks[slot] = task;
    let mut thread_id: u32 = 0;
    let handle = CreateThread(
        null_mut(),
        0,
        thread_main,
        ctx as *mut c_void,
        0,
        &mut thread_id,
    );
    if handle.is_null() {
        tasks[slot] = null_mut();
        reg_unlock();
        free(ctx as *mut u8);
        free(task as *mut u8);
        free(queue as *mut u8);
        release_args(argc, copied);
        return E_RESOURCE;
    }
    (*task).thread = handle;
    TASK_COUNT += 1;
    reg_unlock();
    (slot as i32) + 1
}

/// Start a fire-and-forget chain (`go`): no mailbox, no handle; joined by
/// the exit drain like every other chain.
unsafe fn start_go(thunk: i64, argc: i32, args: *const SchedVal) -> i32 {
    if argc < 0 || (argc > 0 && args.is_null()) || thunk == 0 {
        return E_VALUE;
    }
    let copied = match materialize_args(argc, args) {
        Some(copied) => copied,
        None => return E_VALUE,
    };
    let ctx = alloc(core::mem::size_of::<ThreadArg>()) as *mut ThreadArg;
    if ctx.is_null() {
        free(copied as *mut u8);
        return E_RESOURCE;
    }
    (*ctx).task = null_mut();
    (*ctx).thunk = thunk;
    (*ctx).args = copied;
    (*ctx).argc = argc;
    // PLAN-1067 review R-1067-1: same single-lock publication as `start_task`.
    // A late `go` (issued by a task handler just as the program exits) is
    // either registered before the drain snapshots the table — and therefore
    // joined — or refused with the named draining error. It can never be
    // created outside the drain's view.
    reg_lock();
    let gos = &mut *addr_of_mut!(GO_HANDLES);
    let mut slot = usize::MAX;
    for i in 0..MAX_GO {
        if gos[i].is_null() {
            slot = i;
            break;
        }
    }
    if slot == usize::MAX {
        reg_unlock();
        free(ctx as *mut u8);
        release_args(argc, copied);
        return E_RESOURCE;
    }
    let mut thread_id: u32 = 0;
    let handle = CreateThread(
        null_mut(),
        0,
        thread_main,
        ctx as *mut c_void,
        0,
        &mut thread_id,
    );
    if handle.is_null() {
        reg_unlock();
        free(ctx as *mut u8);
        release_args(argc, copied);
        return E_RESOURCE;
    }
    gos[slot] = handle;
    GO_COUNT += 1;
    reg_unlock();
    0
}

extern "system" fn thread_main(ctx: *mut c_void) -> u32 {
    unsafe {
        let arg = &*(ctx as *mut ThreadArg);
        if arg.task.is_null() {
            set_current_task(0);
        } else {
            let tasks = &*addr_of_mut!(TASKS);
            let mut id = 0i32;
            for i in 0..MAX_TASKS {
                if tasks[i] == arg.task {
                    id = (i as i32) + 1;
                    break;
                }
            }
            set_current_task(id);
        }
        let thunk: extern "system" fn(*mut SchedVal) -> i32 = core::mem::transmute(arg.thunk);
        let _ = thunk(arg.args);
        if !arg.task.is_null() {
            let task = arg.task;
            close_latched_reply(task);
            // Ask messages this driver never received can never be answered
            // either: their futures close now (otherwise the waiter parks
            // forever), and the materialized values are released here.
            AcquireSRWLockExclusive(addr_of_mut!((*task).lock));
            while (*task).len > 0 {
                let mail = pop_mail(task);
                release_value(mail.val);
                if mail.port > 0 {
                    close_port_without_reply(mail.port);
                }
            }
            (*task).closed = 1;
            (*task).stopped = 1;
            (*task).running = 0;
            ReleaseSRWLockExclusive(addr_of_mut!((*task).lock));
            wake_all(addr_of_mut!((*task).cv_not_empty));
            wake_all(addr_of_mut!((*task).cv_not_full));
        }
        free(arg.args as *mut u8);
        free(ctx as *mut u8);
        set_current_task(0);
        0
    }
}

/// Allocate one reply port (registry lock held by the caller).
unsafe fn alloc_port() -> i32 {
    reg_lock();
    let ports = &mut *addr_of_mut!(PORTS);
    let mut slot = usize::MAX;
    for i in 0..MAX_PORTS {
        if ports[i].is_null() {
            slot = i;
            break;
        }
    }
    if slot == usize::MAX {
        reg_unlock();
        return E_RESOURCE;
    }
    let port = alloc(core::mem::size_of::<Port>()) as *mut Port;
    if port.is_null() {
        reg_unlock();
        return E_RESOURCE;
    }
    ports[slot] = port;
    PORT_COUNT += 1;
    reg_unlock();
    (slot as i32) + 1
}

/// PLAN-1067 T-05: a driver finishing without answering the ask it latched
/// closes that future (terminal state 3). The waiter then reports the named
/// `E_REPLY_CLOSED` error instead of parking until the process drains.
/// Close one future that can no longer be published (the producing driver is
/// gone). Terminal state 3 makes the waiter report the named error.
unsafe fn close_port_without_reply(port_id: i32) {
    let port = port_of(port_id);
    if port.is_null() {
        return;
    }
    AcquireSRWLockExclusive(addr_of_mut!((*port).lock));
    if (*port).state == 0 {
        (*port).state = 3;
    }
    ReleaseSRWLockExclusive(addr_of_mut!((*port).lock));
    wake_all(addr_of_mut!((*port).cv));
}
unsafe fn close_latched_reply(task: *mut Task) {
    let port_id = (*task).reply_port;
    if port_id <= 0 {
        return;
    }
    let port = port_of(port_id);
    if port.is_null() {
        return;
    }
    AcquireSRWLockExclusive(addr_of_mut!((*port).lock));
    if (*port).state == 0 {
        (*port).state = 3;
    }
    ReleaseSRWLockExclusive(addr_of_mut!((*port).lock));
    wake_all(addr_of_mut!((*port).cv));
}
unsafe fn release_port(id: i32) {
    let port = port_of(id);
    if port.is_null() {
        return;
    }
    AcquireSRWLockExclusive(addr_of_mut!((*port).lock));
    // A port that never published stays consumed so a pending await fails
    // with the named "closed" class instead of hanging.
    (*port).state = 2;
    ReleaseSRWLockExclusive(addr_of_mut!((*port).lock));
    wake_all(addr_of_mut!((*port).cv));
}

unsafe fn push_mail(task: *mut Task, mail: Mail) {
    let head = (*task).head as usize;
    let len = (*task).len as usize;
    let cap = (*task).cap as usize;
    let slot = (head + len) % cap;
    core::ptr::write_volatile((*task).queue.add(slot), mail);
    (*task).len += 1;
}

unsafe fn pop_mail(task: *mut Task) -> Mail {
    let head = (*task).head as usize;
    let cap = (*task).cap as usize;
    let mail = core::ptr::read_volatile((*task).queue.add(head));
    (*task).head = ((head + 1) % cap) as u32;
    (*task).len -= 1;
    mail
}

// ---------------------------------------------------------------------------
// exported ABI
// ---------------------------------------------------------------------------

/// Spawn one task chain: `cap` mailbox slots, driver `thunk`, `argc`
/// initializer values. Returns the 1-based handle or a negative status.
#[no_mangle]
pub extern "system" fn mn_sched_spawn(
    cap: i32,
    thunk: i64,
    argc: i32,
    args: *const SchedVal,
) -> i32 {
    unsafe { start_task(cap, thunk, argc, args) }
}

/// Fire-and-forget spawn of a plain function chain.
#[no_mangle]
pub extern "system" fn mn_sched_go(thunk: i64, argc: i32, args: *const SchedVal) -> i32 {
    unsafe { start_go(thunk, argc, args) }
}

/// Deliver one message (parks the sender while the mailbox is full).
#[no_mangle]
pub extern "system" fn mn_sched_send(handle: i32, val: *const SchedVal) -> i32 {
    unsafe {
        if val.is_null() {
            return E_VALUE;
        }
        let task = task_of(handle);
        if task.is_null() {
            return E_HANDLE;
        }
        let materialized = match materialize(val) {
            Some(value) => value,
            None => return E_VALUE,
        };
        let mail = Mail {
            val: materialized,
            port: 0,
        };
        AcquireSRWLockExclusive(addr_of_mut!((*task).lock));
        loop {
            if (*task).closed != 0 || (*task).stopped != 0 {
                ReleaseSRWLockExclusive(addr_of_mut!((*task).lock));
                release_value(materialized);
                return E_CLOSED;
            }
            if (*task).len < (*task).cap {
                push_mail(task, mail);
                ReleaseSRWLockExclusive(addr_of_mut!((*task).lock));
                wake_one(addr_of_mut!((*task).cv_not_empty));
                return 0;
            }
            // Bounded backpressure: the sender parks until space frees.
            SleepConditionVariableSRW(
                addr_of_mut!((*task).cv_not_full),
                addr_of_mut!((*task).lock),
                INFINITE,
                0,
            );
        }
    }
}

/// Deliver one message and open its 1-based reply port (the future id).
#[no_mangle]
pub extern "system" fn mn_sched_ask(handle: i32, val: *const SchedVal) -> i32 {
    unsafe {
        if val.is_null() {
            return E_VALUE;
        }
        let task = task_of(handle);
        if task.is_null() {
            return E_HANDLE;
        }
        let materialized = match materialize(val) {
            Some(value) => value,
            None => return E_VALUE,
        };
        let port = alloc_port();
        if port < 0 {
            release_value(materialized);
            return port;
        }
        let mail = Mail {
            val: materialized,
            port,
        };
        AcquireSRWLockExclusive(addr_of_mut!((*task).lock));
        loop {
            if (*task).closed != 0 || (*task).stopped != 0 {
                ReleaseSRWLockExclusive(addr_of_mut!((*task).lock));
                release_port(port);
                release_value(materialized);
                return E_CLOSED;
            }
            if (*task).len < (*task).cap {
                push_mail(task, mail);
                ReleaseSRWLockExclusive(addr_of_mut!((*task).lock));
                wake_one(addr_of_mut!((*task).cv_not_empty));
                return port;
            }
            SleepConditionVariableSRW(
                addr_of_mut!((*task).cv_not_full),
                addr_of_mut!((*task).lock),
                INFINITE,
                0,
            );
        }
    }
}

/// Publish the reply for the current message's ask port. A second reply (or
/// a reply after the consumer took the value) is dropped: the terminal state
/// commits exactly once.
#[no_mangle]
pub extern "system" fn mn_sched_reply(val: *const SchedVal) -> i32 {
    unsafe {
        if val.is_null() {
            return E_VALUE;
        }
        let task = current_task();
        if task.is_null() {
            return E_REPLY;
        }
        let port_id = (*task).reply_port;
        if port_id <= 0 {
            return E_REPLY;
        }
        let port = port_of(port_id);
        if port.is_null() {
            return E_REPLY;
        }
        let materialized = match materialize(val) {
            Some(value) => value,
            None => return E_VALUE,
        };
        let mut published = false;
        AcquireSRWLockExclusive(addr_of_mut!((*port).lock));
        if (*port).state == 0 {
            (*port).val = materialized;
            (*port).state = 1;
            published = true;
        }
        ReleaseSRWLockExclusive(addr_of_mut!((*port).lock));
        if !published {
            // Single terminal commit: a late/duplicate reply is dropped (and
            // its materialized buffer released), never resurrecting the port.
            release_value(materialized);
        }
        wake_all(addr_of_mut!((*port).cv));
        0
    }
}

/// Park until the ask future is published; the value moves to `out`.
#[no_mangle]
pub extern "system" fn mn_sched_await(port_id: i32, out: *mut SchedVal) -> i32 {
    unsafe {
        if out.is_null() {
            return E_VALUE;
        }
        let port = port_of(port_id);
        if port.is_null() {
            return E_HANDLE;
        }
        AcquireSRWLockExclusive(addr_of_mut!((*port).lock));
        loop {
            match (*port).state {
                1 => {
                    let value = (*port).val;
                    (*port).state = 2;
                    ReleaseSRWLockExclusive(addr_of_mut!((*port).lock));
                    core::ptr::write_volatile(out, value);
                    return 0;
                }
                2 => {
                    ReleaseSRWLockExclusive(addr_of_mut!((*port).lock));
                    return E_AWAIT_CONSUMED;
                }
                // The producer closed without publishing: terminate the
                // wait with a named error instead of parking forever.
                3 => {
                    ReleaseSRWLockExclusive(addr_of_mut!((*port).lock));
                    return E_REPLY_CLOSED;
                }
                _ => {
                    if DRAINING.load(Ordering::SeqCst) != 0 {
                        ReleaseSRWLockExclusive(addr_of_mut!((*port).lock));
                        return E_DRAIN;
                    }
                    SleepConditionVariableSRW(
                        addr_of_mut!((*port).cv),
                        addr_of_mut!((*port).lock),
                        INFINITE,
                        0,
                    );
                }
            }
        }
    }
}

/// Receive the next message of the current task. Status 0 = real delivery
/// (the reply port is latched), 1 = closed/stop handoff (the destination
/// gets the zero value and the driver's stopping() check leaves the loop).
#[no_mangle]
pub extern "system" fn mn_sched_recv(out: *mut SchedVal) -> i32 {
    unsafe {
        if out.is_null() {
            return E_VALUE;
        }
        let task = current_task();
        if task.is_null() {
            return E_TASK_ONLY;
        }
        AcquireSRWLockExclusive(addr_of_mut!((*task).lock));
        loop {
            if (*task).len > 0 {
                let mail = pop_mail(task);
                (*task).reply_port = mail.port;
                (*task).recv_real = 1;
                ReleaseSRWLockExclusive(addr_of_mut!((*task).lock));
                core::ptr::write_volatile(out, mail.val);
                wake_one(addr_of_mut!((*task).cv_not_full));
                return 0;
            }
            if (*task).stopped != 0 || (*task).closed != 0 {
                (*task).reply_port = 0;
                (*task).recv_real = 0;
                ReleaseSRWLockExclusive(addr_of_mut!((*task).lock));
                core::ptr::write_volatile(out, SchedVal::zero());
                return 1;
            }
            SleepConditionVariableSRW(
                addr_of_mut!((*task).cv_not_empty),
                addr_of_mut!((*task).lock),
                INFINITE,
                0,
            );
        }
    }
}

/// The reply port of the most recently received message (0 = send mode).
#[no_mangle]
pub extern "system" fn mn_sched_port() -> i32 {
    unsafe {
        let task = current_task();
        if task.is_null() {
            0
        } else {
            (*task).reply_port
        }
    }
}

/// Whether the current task is stopping (stopped AND no message was just
/// delivered — a real delivery masks this for exactly one check).
#[no_mangle]
pub extern "system" fn mn_sched_stopping() -> i32 {
    unsafe {
        let task = current_task();
        if task.is_null() {
            return 0;
        }
        if (*task).recv_real != 0 {
            (*task).recv_real = 0;
            return 0;
        }
        if (*task).stopped == 0 && (*task).closed == 0 {
            return 0;
        }
        // The closing handoff reports stopping only on an EMPTY mailbox:
        // buffered messages deliver first (a stop request never discards
        // queued deliveries). No send can enqueue after the stop is set, so
        // an empty mailbox here is final.
        AcquireSRWLockExclusive(addr_of_mut!((*task).lock));
        let pending = (*task).len > 0;
        ReleaseSRWLockExclusive(addr_of_mut!((*task).lock));
        if pending {
            0
        } else {
            1
        }
    }
}

/// Request an orderly stop of one task (buffered messages still deliver).
#[no_mangle]
pub extern "system" fn mn_sched_stop(handle: i32) -> i32 {
    unsafe {
        let task = task_of(handle);
        if task.is_null() {
            return E_HANDLE;
        }
        AcquireSRWLockExclusive(addr_of_mut!((*task).lock));
        (*task).stopped = 1;
        ReleaseSRWLockExclusive(addr_of_mut!((*task).lock));
        wake_all(addr_of_mut!((*task).cv_not_empty));
        wake_all(addr_of_mut!((*task).cv_not_full));
        0
    }
}

/// Sleep `ms` in cooperative slices: the orderly drain completes sleeps
/// immediately (the AV drain rule), so an exiting program never waits for a
/// timer that can no longer be observed.
#[no_mangle]
pub extern "system" fn mn_sched_sleep(ms: i32) -> i32 {
    unsafe {
        let mut remaining = if ms < 0 { 0 } else { ms };
        let task = current_task();
        loop {
            if DRAINING.load(Ordering::SeqCst) != 0 {
                return 0;
            }
            if !task.is_null() && ((*task).stopped != 0 || (*task).closed != 0) {
                return 0;
            }
            if remaining <= 0 {
                return 0;
            }
            let chunk = if remaining > 10 { 10 } else { remaining };
            Sleep(chunk as u32);
            remaining -= chunk;
        }
    }
}

/// The orderly exit drain: stop every task chain (buffered messages still
/// deliver), run the stop hooks, join every thread, release every block and
/// report the residue (0 = no leak; the lane wrapper turns a non-zero
/// residue into the named `sched.leak` exit, never a silent success).
#[no_mangle]
pub extern "system" fn mn_sched_drain() -> i32 {
    unsafe {
        // PLAN-1067 review R-1067-1: the drain must not SNAPSHOT the chain
        // tables and hope nothing appears later. A stop hook, or a handler
        // whose buffered message is still being delivered, may legally start
        // another `go`/task chain while the drain is running (the frozen
        // semantics: buffered messages deliver first, the stop hooks then
        // run). The drain therefore stops and joins in a loop, re-snapshotting
        // under the registry lock each cycle until no unjoined chain remains.
        // Registration and thread creation happen under that same lock, so a
        // chain can never be created outside this loop's view.
        DRAINING.store(1, Ordering::SeqCst);

        let mut go_slot = 0usize;
        let mut task_slot = 0usize;
        let mut cycles: u32 = 0;
        let mut unjoined_residue: i32 = 0;
        loop {
            cycles += 1;
            if cycles > 100_000 {
                // A program that keeps starting chains forever cannot be
                // closed; report it instead of spinning (the wrapper turns a
                // non-zero residue into the named leak exit).
                unjoined_residue = 1;
                break;
            }
            let mut handle: *mut c_void = null_mut();
            let mut task_record: *mut Task = null_mut();
            let mut port_to_close: *mut Port = null_mut();
            reg_lock();
            let tasks = &*addr_of_mut!(TASKS);
            // PLAN-1067 review2 N-1067-1: EVERY task record present in this
            // cycle is stopped BEFORE it can be joined — including a task that
            // a chain started while the drain was already running. The mark is
            // per record, so each chain is stopped exactly once and no chain is
            // ever joined while still parked in recv (that omission hung the PE
            // until the host run budget killed it).
            let mut sweep = 0usize;
            while sweep < MAX_TASKS {
                let task = tasks[sweep];
                sweep += 1;
                if task.is_null() || (*task).stopped_marked != 0 {
                    continue;
                }
                (*task).stopped_marked = 1;
                AcquireSRWLockExclusive(addr_of_mut!((*task).lock));
                (*task).stopped = 1;
                ReleaseSRWLockExclusive(addr_of_mut!((*task).lock));
                wake_all(addr_of_mut!((*task).cv_not_empty));
                wake_all(addr_of_mut!((*task).cv_not_full));
            }
            let _ = task_slot;
            // Take ONE handle to join this cycle (task chains first).
            let mut i = 0usize;
            while i < MAX_TASKS {
                let task = tasks[i];
                if !task.is_null() && (*task).joined == 0 {
                    (*task).joined = 1;
                    handle = (*task).thread;
                    task_record = task;
                    break;
                }
                i += 1;
            }
            if handle.is_null() {
                let gos = &mut *addr_of_mut!(GO_HANDLES);
                while go_slot < MAX_GO {
                    let slot = go_slot;
                    go_slot += 1;
                    let g = gos[slot];
                    if !g.is_null() && !((&*addr_of_mut!(GO_JOINED))[slot]) {
                        (&mut *addr_of_mut!(GO_JOINED))[slot] = true;
                        handle = g;
                        break;
                    }
                }
            }
            if handle.is_null() {
                // Also retire any still-pending future so a waiter cannot park.
                let ports = &*addr_of_mut!(PORTS);
                let mut p = 0usize;
                while p < MAX_PORTS {
                    let port = ports[p];
                    if !port.is_null() && (*port).state == 0 && (*port).closed_marked == 0 {
                        (*port).closed_marked = 1;
                        port_to_close = port;
                        break;
                    }
                    p += 1;
                }
            }
            reg_unlock();

            if !port_to_close.is_null() {
                AcquireSRWLockExclusive(addr_of_mut!((*port_to_close).lock));
                if (*port_to_close).state == 0 {
                    (*port_to_close).state = 3;
                }
                ReleaseSRWLockExclusive(addr_of_mut!((*port_to_close).lock));
                wake_all(addr_of_mut!((*port_to_close).cv));
                continue;
            }
            if handle.is_null() {
                let _ = task_record;
                break;
            }
            WaitForSingleObject(handle, INFINITE);
            CloseHandle(handle);
            if !task_record.is_null() {
                (*task_record).thread = null_mut();
            }
        }

        // Release whatever the orderly stop left undelivered (messages whose
        // driver already left, published-but-never-awaited replies) and the
        // registry records themselves.
        reg_lock();
        let tasks = &*addr_of_mut!(TASKS);
        let ports = &*addr_of_mut!(PORTS);
        for i in 0..MAX_TASKS {
            let task = tasks[i];
            if task.is_null() {
                continue;
            }
            while (*task).len > 0 {
                let mail = pop_mail(task);
                release_value(mail.val);
            }
            free((*task).queue as *mut u8);
            free(task as *mut u8);
            (&mut *addr_of_mut!(TASKS))[i] = null_mut();
        }
        for i in 0..MAX_PORTS {
            let port = ports[i];
            if port.is_null() {
                continue;
            }
            if (*port).state == 1 {
                release_value((*port).val);
            }
            free(port as *mut u8);
            (&mut *addr_of_mut!(PORTS))[i] = null_mut();
        }
        for i in 0..MAX_GO {
            (&mut *addr_of_mut!(GO_HANDLES))[i] = null_mut();
            (&mut *addr_of_mut!(GO_JOINED))[i] = false;
        }
        TASK_COUNT = 0;
        PORT_COUNT = 0;
        GO_COUNT = 0;
        reg_unlock();

        // Audit: every block this runtime owned is released, and the embedded
        // owned-text allocator balances too. A residue is reported (never
        // swallowed) by the lane wrapper as the named leak exit.
        let mut residue = LIVE_ALLOCS.load(Ordering::SeqCst);
        if ac_text_runtime::ac_rt_owned_audit() != 0 {
            residue += 1;
        }
        residue + unjoined_residue
    }
}
/// The language-error transport: write the frozen message to stderr and
/// exit with the native trap code. The lane's lowering calls this exactly
/// where AV reports `ExecOutcome::Error`.
#[no_mangle]
pub extern "system" fn mn_sched_fail(code: i32) -> ! {
    unsafe {
        let message = fail_message(code);
        let handle = GetStdHandle(STD_ERROR_HANDLE);
        if handle != 0 && handle != -1 {
            let mut written: u32 = 0;
            WriteFile(
                handle,
                message.as_ptr(),
                message.len() as u32,
                &mut written,
                null_mut(),
            );
        }
        ExitProcess(EXIT_TRAP);
    }
}
