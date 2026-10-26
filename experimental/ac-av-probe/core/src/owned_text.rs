//! Owned-text arena and UTF-8 text operations (PLAN-821, Design44 §4.1/§7).
//!
//! The arena is the OBSERVATION ledger of owned Str buffers: every
//! allocation takes an id (monotonic, creation order), and the final
//! release of a buffer (last `StrHandle` drop) appends a free event with
//! the buffer's id — the V-04 "逆创建序唯一 drop" and V-01 "宿主 release
//! 后 live=0" oracles read these sequences; nothing is reaped.
//!
//! Execution-side helpers here are pure byte/scalar logic (774's UTF-8 /
//! Unicode-scalar-index rules); frame-rooted view resolution lives with
//! the executor (`av.rs`) which owns the frame table.

use crate::contract::{StrHandle, StrView, TrapKind};

/// One allocation-ledger event (pure data; the arena owns the sequences).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocEvent {
    pub id: u64,
    pub bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FreeEvent {
    pub id: u64,
    pub bytes: usize,
}

/// The shared ledger core. One instance per execution (created by the
/// executor); handles carry a clone of the `Rc` so host-side releases
/// after `execute` returns still report here.
#[derive(Debug, Default)]
pub struct ArenaCore {
    next_id: u64,
    pub allocs: Vec<AllocEvent>,
    pub frees: Vec<FreeEvent>,
    /// Ids of buffers currently alive (inserted on alloc, removed on the
    /// free report) — `live` is its length.
    live_ids: std::collections::BTreeSet<u64>,
    /// PLAN-821 V-03/V-07 accounting: every ATTEMPT counts (successful or
    /// not); `fault_at` (test-only) fails the Nth attempt BEFORE any
    /// buffer exists — attempt recorded, successful_alloc unchanged.
    pub attempts: u64,
    pub fault_at: Option<u64>,
}

impl ArenaCore {
    /// Report the final release of buffer `id` (called from
    /// `StrHandle::drop` while the payload is still readable).
    pub fn report_free(core: &std::rc::Rc<std::cell::RefCell<Self>>, id: u64, bytes: usize) {
        let mut g = core.borrow_mut();
        g.frees.push(FreeEvent { id, bytes });
        g.live_ids.remove(&id);
    }
}

/// The per-execution owned-text arena handle.
#[derive(Clone, Debug, Default)]
pub struct StrArena {
    core: std::rc::Rc<std::cell::RefCell<ArenaCore>>,
}

impl StrArena {
    pub fn new() -> Self {
        StrArena {
            core: std::rc::Rc::new(std::cell::RefCell::new(ArenaCore::default())),
        }
    }

    /// Allocate one owned buffer; the ledger records the attempt AND the
    /// success (V-07: 预算前拒的申请 attempt=0 在预算层; allocator 故障
    /// 注入 attempt 存在但 successful_alloc=0 由调用层区分).
    pub fn alloc(&self, bytes: &[u8]) -> StrHandle {
        self.try_alloc(bytes)
            .expect("the fault seam is test-only; production arenas never configure it")
    }

    /// Number of owned buffers currently alive.
    pub fn live(&self) -> usize {
        self.core.borrow().live_ids.len()
    }

    /// Test-only fault seam (V-03/V-07): fail the Nth allocation ATTEMPT.
    pub fn set_fault_at(&self, n: u64) {
        self.core.borrow_mut().fault_at = Some(n);
    }

    /// Allocation attempts so far (successful or faulted).
    pub fn attempts(&self) -> u64 {
        self.core.borrow().attempts
    }

    /// Successful allocations so far.
    pub fn successful_allocs(&self) -> usize {
        self.core.borrow().allocs.len()
    }

    /// Fallible allocation: the Nth attempt (when configured) fails with
    /// the attempt number — no buffer, no ledger entry, nothing
    /// half-committed (V-03: attempt exists, successful_alloc unchanged).
    pub fn try_alloc(&self, bytes: &[u8]) -> Result<StrHandle, u64> {
        let mut g = self.core.borrow_mut();
        g.attempts += 1;
        if g.fault_at == Some(g.attempts) {
            return Err(g.attempts);
        }
        let id = g.next_id;
        g.next_id += 1;
        g.allocs.push(AllocEvent {
            id,
            bytes: bytes.len(),
        });
        g.live_ids.insert(id);
        let core = std::rc::Rc::clone(&self.core);
        drop(g);
        Ok(StrHandle::attached(
            id,
            bytes.to_vec().into_boxed_slice(),
            core,
        ))
    }

    /// PLAN-822: fallible LIST buffer allocation over the same ledger
    /// (the attempt/fault/success accounting is shared with Str; `bytes`
    /// records the element count).
    /// F822-R13-01: split open — `list_alloc_attempt` reserves the
    /// fault seam BEFORE any elements change hands (on an injected fault
    /// the caller keeps ownership, so the collected elements release
    /// through the explicit cleanup worklist instead of the recursive
    /// drop glue inside this module's call frame); `alloc_list_after_
    /// attempt` is the post-attempt mint.
    pub fn list_alloc_attempt(&self) -> Result<(), u64> {
        let mut g = self.core.borrow_mut();
        g.attempts += 1;
        if g.fault_at == Some(g.attempts) {
            return Err(g.attempts);
        }
        Ok(())
    }

    /// The post-attempt mint — `list_alloc_attempt` already said Ok.
    pub fn alloc_list_after_attempt(
        &self,
        elems: Vec<crate::contract::Value>,
        reserve_cap: usize,
    ) -> crate::contract::ListHandle {
        let mut g = self.core.borrow_mut();
        let id = g.next_id;
        g.next_id += 1;
        g.allocs.push(AllocEvent {
            id,
            bytes: elems.len(),
        });
        g.live_ids.insert(id);
        let core = std::rc::Rc::clone(&self.core);
        drop(g);
        let handle = crate::contract::ListHandle::attached(id, elems, core);
        {
            let mut buf = handle.rc.borrow_mut();
            buf.cap = buf.cap.max(reserve_cap as u32);
        }
        handle
    }

    pub fn try_alloc_list(
        &self,
        elems: Vec<crate::contract::Value>,
        reserve_cap: usize,
    ) -> Result<crate::contract::ListHandle, u64> {
        self.list_alloc_attempt()?;
        Ok(self.alloc_list_after_attempt(elems, reserve_cap))
    }

    /// The full allocation event sequence (creation order).
    pub fn alloc_events(&self) -> Vec<AllocEvent> {
        self.core.borrow().allocs.clone()
    }

    /// The full free event sequence (release order — the LIFO oracle).
    pub fn free_events(&self) -> Vec<FreeEvent> {
        self.core.borrow().frees.clone()
    }
}

// ---------------------------------------------------------------------------
// UTF-8 / Unicode-scalar rules (774 frozen semantics)
// ---------------------------------------------------------------------------

/// Decode the scalar values of a UTF-8 byte slice (`char_at`/`len` domain
/// is Unicode scalars, never bytes/graphemes). Iterative, no host recursion.
pub fn scalars(bytes: &[u8]) -> Vec<char> {
    std::str::from_utf8(bytes)
        .map(|s| s.chars().collect())
        .unwrap_or_default()
}

/// Resolve `[start, end)` scalar indices onto `bytes` to a byte range.
/// `None` = out-of-domain (negative-shaped values are rejected by
/// verification as i32 args; runtime bounds are checked here): `end > len`
/// or `start > end` is a runtime `av.trap.str-index`.
pub fn scalar_range_to_bytes(
    bytes: &[u8],
    start: i32,
    end: i32,
) -> Result<(usize, usize), TrapKind> {
    let chars = scalars(bytes);
    let n = chars.len() as i32;
    if start < 0 || end < 0 || end > n || start > end {
        return Err(TrapKind::StrIndex);
    }
    let mut byte_start = 0usize;
    for (i, c) in chars.iter().enumerate() {
        if i as i32 == start {
            break;
        }
        byte_start += c.len_utf8();
    }
    let mut byte_end = byte_start;
    for (i, c) in chars.iter().enumerate().skip(start as usize) {
        if i as i32 == end {
            break;
        }
        byte_end += c.len_utf8();
    }
    Ok((byte_start, byte_end))
}

/// The scalar at index `i` (Unicode-scalar indexing, 774: `A中😀` = 0/1/2).
pub fn char_at(bytes: &[u8], index: i32) -> Result<char, TrapKind> {
    let chars = scalars(bytes);
    if index < 0 || index as usize >= chars.len() {
        return Err(TrapKind::StrIndex);
    }
    Ok(chars[index as usize])
}

/// Materialize the byte slice a view denotes over `bytes`. The scalar
/// range is re-projected here so a stored view is just (source, range).
pub fn view_bytes<'a>(bytes: &'a [u8], view: &StrView) -> Result<&'a [u8], TrapKind> {
    let (s, e) = scalar_range_to_bytes(bytes, view.start as i32, view.end as i32)?;
    Ok(&bytes[s..e])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_records_lifo_release() {
        let a = StrArena::new();
        let h1 = a.alloc(b"auto");
        let h2 = a.alloc(b"lang");
        assert_eq!(a.live(), 2);
        drop(h2);
        drop(h1);
        assert_eq!(a.live(), 0);
        assert_eq!(
            a.free_events(),
            vec![FreeEvent { id: 1, bytes: 4 }, FreeEvent { id: 0, bytes: 4 },]
        );
    }

    #[test]
    fn shared_handle_releases_once_at_last_owner() {
        let a = StrArena::new();
        let h = a.alloc(b"x");
        let shared = h.clone();
        drop(h);
        assert_eq!(a.live(), 1, "one owner still alive");
        drop(shared);
        assert_eq!(a.live(), 0);
    }

    #[test]
    fn scalar_rules_follow_774() {
        // "A中😀": len=3 scalars, len_bytes=8; slice [1,3) = "中😀".
        let bytes = "A中😀".as_bytes();
        assert_eq!(scalars(bytes).len(), 3);
        assert_eq!(bytes.len(), 8);
        assert_eq!(char_at(bytes, 1), Ok('中'));
        let (s, e) = scalar_range_to_bytes(bytes, 1, 3).unwrap();
        assert_eq!(&bytes[s..e], "中😀".as_bytes());
        assert!(scalar_range_to_bytes(bytes, 0, 4).is_err());
        assert!(scalar_range_to_bytes(bytes, 2, 1).is_err());
        // Composed scalar: e + U+0301 = 2 scalars (not a grapheme merge).
        assert_eq!(scalars("e\u{0301}".as_bytes()).len(), 2);
    }
}
