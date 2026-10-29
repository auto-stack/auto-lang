//! PLAN-889 private native container primitives. No MIR/ABC, opcode dispatch,
//! source frontend, or AV executor lives here. Control flow and arithmetic
//! belong to generated machine code. All callers are verified generated code.
//! Text buffers use the delivered text shim; these nodes only retain owners.
#![no_std]
#![no_builtins]

// Embed the delivered text implementation and its single panic handler in
// this static library. Linking two independent no_std panic handlers conflicts.
extern crate ac_text_runtime;

use core::ptr::{null_mut, read, write};

extern "system" {
    fn GetProcessHeap() -> isize;
    fn HeapAlloc(heap: isize, flags: u32, bytes: usize) -> *mut u8;
    fn HeapFree(heap: isize, flags: u32, ptr: *mut u8) -> i32;
    fn GetStdHandle(which: i32) -> isize;
    fn WriteFile(h: isize, p: *const u8, n: u32, done: *mut u32, over: *mut u8) -> i32;
    fn ExitProcess(code: u32) -> !;
    fn ac_rt_own_from_lit(p: *const u8, len: i32, out: *mut i32) -> *mut u8;
    fn ac_rt_own_clone(p: *const u8, len: i32, out: *mut i32) -> *mut u8;
    fn ac_rt_own_free(p: *mut u8);
    fn ac_rt_text_scalars(p: *const u8, len: i32) -> i32;
    fn ac_rt_summary();
}

pub const ABI: &str = "mn.collections.runtime/1";
pub const GUEST_CAP: usize = 8 << 20;
pub const MANAGED_CAP: usize = 32 << 20;
pub const LIST_CAP: usize = 65536;
pub const INDEX: i32 = -21;
pub const RESOURCE: i32 = -22;
pub const INVALID: i32 = -23;
pub const VARIANT: i32 = -24;
pub const INJECTED: i32 = -25;

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
}

const TEXT: u32 = 1;
const RECORD: u32 = 2;
const LIST: u32 = 3;
const TAGGED: u32 = 4;
#[repr(C)]
struct Node {
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
fn allocate(bytes: usize) -> Result<*mut u8, i32> {
    state().attempts += 1;
    if state().fail_nth != 0 && state().attempts == state().fail_nth {
        return Err(INJECTED);
    }
    let p = unsafe { HeapAlloc(GetProcessHeap(), 8, bytes.max(1)) };
    if p.is_null() {
        Err(RESOURCE)
    } else {
        Ok(p)
    }
}
unsafe fn free(p: *mut u8) -> bool {
    if !p.is_null() {
        if HeapFree(GetProcessHeap(), 0, p) == 0 {
            state().invalid += 1;
            return false;
        }
    }
    true
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
            uncharge((*p).bytes, (*p).guest);
            free(p.cast());
            state().nodes -= 1;
            state().frees += 1;
        }
    }
}
unsafe fn clear(c: *mut Cell) {
    let old = read(c);
    write(c, EMPTY);
    release(old);
}
unsafe fn commit(dst: *mut Cell, c: Cell) {
    clear(dst);
    write(dst, c);
}
unsafe fn take(c: *mut Cell) -> Cell {
    let value = read(c);
    if value.owned != 0 {
        write(c, EMPTY);
    }
    value
}

fn new_node(kind: u32, cap: usize, guest: usize) -> Result<*mut Node, i32> {
    let data_bytes = if kind == LIST || kind == RECORD {
        cap.checked_mul(core::mem::size_of::<Cell>())
            .ok_or(RESOURCE)?
    } else {
        0
    };
    let bytes = core::mem::size_of::<Node>()
        .checked_add(data_bytes)
        .and_then(|n| n.checked_add(if kind == TEXT { guest } else { 0 }))
        .ok_or(RESOURCE)?;
    precheck(bytes, guest)?;
    let p = allocate(core::mem::size_of::<Node>())?.cast::<Node>();
    let data = if data_bytes == 0 {
        null_mut()
    } else {
        match allocate(data_bytes) {
            Ok(data) => data,
            Err(e) => {
                unsafe {
                    free(p.cast());
                }
                return Err(e);
            }
        }
    };
    unsafe {
        write(
            p,
            Node {
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
    status((|| {
        let bytes = (count as usize)
            .checked_mul(16)
            .and_then(|n| n.checked_add(core::mem::size_of::<Frame>()))
            .ok_or(RESOURCE)?;
        precheck(bytes, 0)?;
        write(
            f,
            Frame {
                prev: state().frame,
                cells,
                count: count as usize,
                charge: bytes,
            },
        );
        state().frame = f;
        charge(bytes, 0);
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
    for i in (0..count).rev() {
        clear((*f).cells.add(i));
    }
    if (*f).count & heap_flag != 0 {
        free((*f).cells.cast());
    }
    state().frame = (*f).prev;
    uncharge((*f).charge, 0);
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
    let Ok(cells) = allocate(bytes) else {
        return null_mut();
    };
    if mn_coll_frame_enter(f, cells.cast(), count) != 0 {
        free(cells);
        return null_mut();
    }
    (*f).count |= 1usize << (usize::BITS - 1);
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
            return Err(RESOURCE);
        }
        (*p).data = data;
        (*p).len = actual as usize;
        commit(dst, owner(p));
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
        if (*p).len == (*p).cap {
            let cap = (*p)
                .cap
                .max(1)
                .checked_mul(2)
                .ok_or(RESOURCE)?
                .min(LIST_CAP);
            let new_bytes = cap.checked_mul(16).ok_or(RESOURCE)?;
            let old_bytes = (*p).cap * 16;
            // Charge the real simultaneous old + new allocation, not just its delta.
            precheck(new_bytes, new_bytes)?;
            let buffer = allocate(new_bytes)?;
            charge(new_bytes, new_bytes);
            for i in 0..(*p).len {
                write(
                    buffer.cast::<Cell>().add(i),
                    read((*p).data.cast::<Cell>().add(i)),
                );
            }
            free((*p).data);
            uncharge(old_bytes, old_bytes);
            (*p).data = buffer;
            (*p).cap = cap;
            (*p).bytes += new_bytes - old_bytes;
            (*p).guest += new_bytes - old_bytes;
        }
        let data = (*p).data.cast::<Cell>();
        for i in (index as usize..(*p).len).rev() {
            write(data.add(i + 1), read(data.add(i)));
        }
        write(data.add(index as usize), take(src));
        (*p).len += 1;
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
        let data = (*p).data.cast::<Cell>();
        let c = read(data.add(index as usize));
        for i in index as usize + 1..(*p).len {
            write(data.add(i - 1), read(data.add(i)));
        }
        (*p).len -= 1;
        write(data.add((*p).len), EMPTY);
        commit(dst, c);
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
        Ok(())
    })())
}

/// Clone only one node's storage. Child owning cells start empty until the
/// explicit work stack attaches their clones; every partial tree is droppable.
unsafe fn clone_shallow(source: *mut Node) -> Result<*mut Node, i32> {
    let dest = new_node((*source).kind, (*source).cap, (*source).guest)?;
    (*dest).len = (*source).len;
    (*dest).disc = (*source).disc;
    (*dest).moved = (*source).moved;
    if (*source).kind == TEXT {
        let mut length = 0;
        let data = ac_rt_own_clone((*source).data, (*source).len as i32, &mut length);
        if length < 0 {
            release(owner(dest));
            return Err(RESOURCE);
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
    let work = allocate(size)?.cast::<CloneWork>();
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
        Ok(())
    })())
}

#[no_mangle]
pub extern "system" fn mn_coll_fail_nth(n: u64) {
    state().attempts = 0;
    state().fail_nth = n;
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
    line(b"MN-COLL live=", state().nodes as u64);
    line(b"MN-COLL bytes=", state().managed as u64);
    line(b"MN-COLL guest=", state().guest as u64);
    line(b"MN-COLL peak=", state().peak as u64);
    line(b"MN-COLL allocs=", state().allocs);
    line(b"MN-COLL frees=", state().frees);
    line(b"MN-COLL invalid=", state().invalid);
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
