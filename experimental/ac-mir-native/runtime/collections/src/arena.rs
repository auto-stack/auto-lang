//! A bounded native allocation domain. Every allocator header, alignment gap,
//! free fragment, text ledger and buffer occupies the same committed pages.
//! Reserve is virtual address space; commit/peak are actual private page bytes.
use core::ptr::{null_mut, write, write_volatile};
const PAGE: usize = 4096;
pub const LIMIT: usize = 32 << 20;
extern "system" {
    fn VirtualAlloc(address: *mut u8, bytes: usize, kind: u32, protect: u32) -> *mut u8;
    fn VirtualFree(address: *mut u8, bytes: usize, kind: u32) -> i32;
}
#[repr(C)]
struct Block {
    size: usize,
    previous: usize,
    next_free: *mut Block,
    available: usize,
}
const HEADER: usize = core::mem::size_of::<Block>();
struct Arena {
    base: *mut u8,
    used: usize,
    committed: usize,
    peak: usize,
    free: *mut Block,
    last_size: usize,
    live: usize,
    invalid: u64,
    limit: usize,
    external: usize,
    domain_peak: usize,
}
static mut ARENA: Arena = Arena {
    base: null_mut(),
    used: 0,
    committed: 0,
    peak: 0,
    free: null_mut(),
    last_size: 0,
    live: 0,
    invalid: 0,
    limit: LIMIT,
    external: 0,
    domain_peak: 0,
};
fn arena() -> &'static mut Arena {
    unsafe { &mut *core::ptr::addr_of_mut!(ARENA) }
}
unsafe fn remove_free(block: *mut Block) {
    let mut link = core::ptr::addr_of_mut!(arena().free);
    while !(*link).is_null() {
        if *link == block {
            *link = (*block).next_free;
            return;
        }
        link = core::ptr::addr_of_mut!((**link).next_free);
    }
}
pub unsafe fn allocate(bytes: usize) -> *mut u8 {
    let limit = arena().limit.min(LIMIT - arena().external);
    let Some(size) = bytes
        .max(1)
        .checked_add(HEADER)
        .and_then(|n| n.checked_add(15))
        .map(|n| n & !15)
    else {
        return null_mut();
    };
    if size > limit {
        return null_mut();
    }
    if arena().base.is_null() {
        let base = VirtualAlloc(null_mut(), LIMIT, 0x2000, 4);
        if base.is_null() {
            return null_mut();
        }
        arena().base = base;
    }
    let mut block = arena().free;
    while !block.is_null() && (*block).size < size {
        block = (*block).next_free;
    }
    if !block.is_null() {
        remove_free(block);
        let rest = (*block).size - size;
        if rest >= HEADER + 16 {
            let tail = block.cast::<u8>().add(size).cast::<Block>();
            write(
                tail,
                Block {
                    size: rest,
                    previous: size,
                    next_free: arena().free,
                    available: 1,
                },
            );
            arena().free = tail;
            (*block).size = size;
            let end = tail.cast::<u8>().add(rest);
            if end < arena().base.add(arena().used) {
                (*end.cast::<Block>()).previous = rest;
            } else {
                arena().last_size = rest;
            }
        }
        (*block).available = 0;
        (*block).next_free = null_mut();
    } else {
        let Some(end) = arena().used.checked_add(size).filter(|n| *n <= limit) else {
            return null_mut();
        };
        let commit = (end + PAGE - 1) & !(PAGE - 1);
        if commit > limit {
            return null_mut();
        }
        if commit > arena().committed {
            let fresh = VirtualAlloc(
                arena().base.add(arena().committed),
                commit - arena().committed,
                0x1000,
                4,
            );
            if fresh.is_null() {
                return null_mut();
            }
            arena().committed = commit;
            arena().peak = arena().peak.max(commit);
            arena().domain_peak = arena().domain_peak.max(commit + arena().external);
        }
        block = arena().base.add(arena().used).cast();
        write(
            block,
            Block {
                size,
                previous: arena().last_size,
                next_free: null_mut(),
                available: 0,
            },
        );
        arena().used = end;
        arena().last_size = size;
    }
    arena().live += (*block).size;
    let data = block.cast::<u8>().add(HEADER);
    for i in 0..bytes.max(1) {
        write_volatile(data.add(i), 0);
    }
    data
}
pub unsafe fn release(data: *mut u8) -> bool {
    if data.is_null() {
        return true;
    }
    if arena().base.is_null()
        || (data as usize) & 15 != 0
        || data < arena().base.add(HEADER)
        || data >= arena().base.add(arena().used)
    {
        arena().invalid += 1;
        return false;
    }
    let mut block = data.sub(HEADER).cast::<Block>();
    let size = (*block).size;
    if (*block).available != 0
        || size < HEADER + 16
        || size & 15 != 0
        || block.cast::<u8>().add(size) > arena().base.add(arena().used)
    {
        arena().invalid += 1;
        return false;
    }
    arena().live -= size;
    (*block).available = 1;
    let previous = (*block).previous;
    if previous != 0 {
        let prev = block.cast::<u8>().sub(previous).cast::<Block>();
        if (*prev).available == 1 {
            remove_free(prev);
            (*prev).size += size;
            block = prev;
        }
    }
    let end = block.cast::<u8>().add((*block).size);
    if end < arena().base.add(arena().used) {
        let next = end.cast::<Block>();
        if (*next).available == 1 {
            remove_free(next);
            (*block).size += (*next).size;
        }
    }
    let end = block.cast::<u8>().add((*block).size);
    if end == arena().base.add(arena().used) {
        let used = block as usize - arena().base as usize;
        arena().last_size = (*block).previous;
        arena().used = used;
        let commit = (used + PAGE - 1) & !(PAGE - 1);
        if commit < arena().committed {
            if VirtualFree(arena().base.add(commit), arena().committed - commit, 0x4000) == 0 {
                arena().invalid += 1;
            } else {
                arena().committed = commit;
            }
        }
    } else {
        (*end.cast::<Block>()).previous = (*block).size;
        (*block).next_free = arena().free;
        arena().free = block;
    }
    true
}
pub fn committed() -> usize {
    arena().committed
}
pub fn peak() -> usize {
    arena().peak
}
pub fn live() -> usize {
    arena().live
}
pub fn invalid() -> u64 {
    arena().invalid
}
/// Registered real frame/stack-cell storage shares the production managed
/// ceiling with actual arena pages. Probe arena-only limits remain independent.
pub fn hold_external(bytes: usize) -> bool {
    let Some(next) = arena().external.checked_add(bytes).filter(|n| {
        n.checked_add(arena().committed)
            .is_some_and(|total| total <= LIMIT)
    }) else {
        return false;
    };
    arena().external = next;
    arena().domain_peak = arena().domain_peak.max(next + arena().committed);
    true
}
pub fn release_external(bytes: usize) {
    if bytes > arena().external {
        arena().invalid += 1;
    } else {
        arena().external -= bytes;
    }
}
pub fn registered() -> usize {
    arena().external
}
pub fn domain_peak() -> usize {
    arena().domain_peak
}
/// Probe seam only tightens the fixed production ceiling; an active domain
/// cannot set a ceiling below pages that are already committed.
pub fn tighten_limit(bytes: usize) -> bool {
    let bytes = (bytes.min(LIMIT)) & !(PAGE - 1);
    if bytes < arena().committed || bytes > arena().limit {
        return false;
    }
    arena().limit = bytes;
    true
}
