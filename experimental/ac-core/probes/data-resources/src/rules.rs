//! D-04 restricted borrow/own rule model — the probe's FRONT DOOR
//! (probe-cases.md §V-04; decisions.md §D-04).
//!
//! This is a model of the verify-layer rules 770/771 will enforce, used by
//! the V-04 cases to prove illegal states are rejected BEFORE any native
//! dispatch (ledger stays 0 for the rejected op) while legal siblings run as
//! real PEs. It is NOT the production verifier and claims no production
//! borrow-checking support.

use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Debug)]
enum SlotState {
    Dead,
    Owned,
    Viewed(u32),
    MutBorrowed,
    MovedOut,
}

/// Modeled resource operations (slot indices are abstract places).
#[derive(Clone, Copy, Debug)]
pub enum Op {
    Make(usize),
    View(usize),
    EndView(usize),
    Mut(usize),
    EndMut(usize),
    MoveOut(usize),
    Free(usize),
    Clone(usize),
    /// `return <view of a locally owned slot>` — always an escape.
    ReturnViewLocal(usize),
    /// `return <view of a borrowed param>` — owner outlives the frame.
    ReturnViewParam(usize),
}

/// A front-door rejection: 1-based op index plus the frozen verify code.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rejection {
    pub index: usize,
    pub code: &'static str,
}

pub struct RuleOutcome {
    pub rejection: Option<Rejection>,
    /// Native dispatch counts per native counterpart. Populated only for
    /// ops that RAN; the rejected op (and everything after) never dispatches.
    pub ledger: BTreeMap<&'static str, u32>,
}

const SLOTS: usize = 4;

/// Run a script through the front door. On the first illegal op the model
/// rejects with the frozen code and NOTHING further dispatches natively.
pub fn run_script(ops: &[Op]) -> RuleOutcome {
    let mut slots = [SlotState::Dead; SLOTS];
    let mut ledger: BTreeMap<&'static str, u32> = BTreeMap::new();

    let reject = |index: usize, code: &'static str| Rejection { index, code };

    for (i, op) in ops.iter().enumerate() {
        let n = i + 1;
        let slot =
            |slots: &[SlotState; SLOTS], i: usize| slots.get(i).copied().unwrap_or(SlotState::Dead);
        match *op {
            Op::Make(s) => match slot(&slots, s) {
                SlotState::Dead | SlotState::MovedOut => {
                    slots[s] = SlotState::Owned;
                    *ledger.entry("own_make").or_insert(0) += 1;
                }
                SlotState::Owned => {
                    return reject_end(ledger, reject(n, "verify.ownership-required"))
                }
                _ => return reject_end(ledger, reject(n, "verify.mut-conflict")),
            },
            Op::View(s) => match slot(&slots, s) {
                SlotState::Owned => {
                    let v = match slots[s] {
                        SlotState::Viewed(k) => k,
                        _ => 0,
                    };
                    slots[s] = SlotState::Viewed(v + 1);
                    *ledger.entry("borrow_read").or_insert(0) += 1;
                }
                SlotState::Viewed(k) => {
                    slots[s] = SlotState::Viewed(k + 1);
                    *ledger.entry("borrow_read").or_insert(0) += 1;
                }
                SlotState::Dead => return reject_end(ledger, reject(n, "verify.use-after-free")),
                SlotState::MovedOut => {
                    return reject_end(ledger, reject(n, "verify.use-after-move"))
                }
                SlotState::MutBorrowed => {
                    return reject_end(ledger, reject(n, "verify.mut-conflict"))
                }
            },
            Op::EndView(s) => match slots[s] {
                // the last view ending returns the slot to plain ownership
                SlotState::Viewed(1) => slots[s] = SlotState::Owned,
                SlotState::Viewed(k) if k > 1 => slots[s] = SlotState::Viewed(k - 1),
                // model construction error in the test, not a language rule
                _ => panic!("EndView without an active view"),
            },
            Op::Mut(s) => match slot(&slots, s) {
                SlotState::Owned => {
                    slots[s] = SlotState::MutBorrowed;
                    *ledger.entry("rec_mut").or_insert(0) += 1;
                }
                SlotState::Dead => return reject_end(ledger, reject(n, "verify.use-after-free")),
                SlotState::MovedOut => {
                    return reject_end(ledger, reject(n, "verify.use-after-move"))
                }
                _ => return reject_end(ledger, reject(n, "verify.mut-conflict")),
            },
            Op::EndMut(s) => match slots[s] {
                SlotState::MutBorrowed => slots[s] = SlotState::Owned,
                _ => panic!("EndMut without an active mut borrow"),
            },
            Op::MoveOut(s) => match slot(&slots, s) {
                SlotState::Owned => slots[s] = SlotState::MovedOut,
                SlotState::MovedOut => return reject_end(ledger, reject(n, "verify.double-move")),
                SlotState::Dead => return reject_end(ledger, reject(n, "verify.use-after-free")),
                _ => return reject_end(ledger, reject(n, "verify.mut-conflict")),
            },
            Op::Free(s) => match slot(&slots, s) {
                SlotState::Owned => {
                    slots[s] = SlotState::Dead;
                    *ledger.entry("own_free").or_insert(0) += 1;
                }
                SlotState::Dead => return reject_end(ledger, reject(n, "verify.use-after-free")),
                SlotState::MovedOut => {
                    return reject_end(ledger, reject(n, "verify.use-after-move"))
                }
                _ => return reject_end(ledger, reject(n, "verify.mut-conflict")),
            },
            Op::Clone(s) => match slot(&slots, s) {
                SlotState::Owned | SlotState::Viewed(_) => {
                    *ledger.entry("own_clone").or_insert(0) += 1;
                }
                SlotState::Dead => return reject_end(ledger, reject(n, "verify.use-after-free")),
                SlotState::MovedOut => {
                    return reject_end(ledger, reject(n, "verify.use-after-move"))
                }
                SlotState::MutBorrowed => {
                    return reject_end(ledger, reject(n, "verify.mut-conflict"))
                }
            },
            Op::ReturnViewLocal(_) => return reject_end(ledger, reject(n, "verify.borrow-escape")),
            Op::ReturnViewParam(_) => {
                *ledger.entry("borrow_return").or_insert(0) += 0;
            }
        }
    }
    RuleOutcome {
        rejection: None,
        ledger,
    }
}

fn reject_end(ledger: BTreeMap<&'static str, u32>, r: Rejection) -> RuleOutcome {
    RuleOutcome {
        rejection: Some(r),
        ledger,
    }
}

/// The full set of model rejection codes (no depth-based limit exists —
/// V-08's audit assertion).
pub const REJECTION_CODES: &[&str] = &[
    "verify.use-after-free",
    "verify.double-move",
    "verify.use-after-move",
    "verify.mut-conflict",
    "verify.borrow-escape",
    "verify.field-borrow",
    "verify.container-borrow",
    "verify.ownership-required",
];
