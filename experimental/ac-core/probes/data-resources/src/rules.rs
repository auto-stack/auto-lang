//! D-04 restricted borrow/own rule model — the probe's FRONT DOOR
//! (probe-cases.md §V-04; decisions.md §D-04).
//!
//! Review r1 (P774-R1-02): the model tracks each slot's PROVENANCE —
//! locally-owned, borrowed-in param, or move-in owned param — because the
//! legality of returns, frees, and mode requirements depends on where the
//! resource came from, not on the operation's label. This is a model of the
//! verify-layer rules 770/771 will enforce, used by the V-04 cases to prove
//! illegal states are rejected BEFORE any native dispatch (ledger stays 0
//! for the rejected op) while legal siblings run as real PEs. It is NOT the
//! production verifier and claims no production borrow-checking support.

use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Debug)]
enum SlotState {
    Dead,
    /// Owned by the current frame (let Str local / own construction).
    LocalOwned,
    /// Borrowed-in `str` param: the owner is the CALLER and outlives us.
    ParamBorrowed,
    /// Move-in `move Str` param: this frame owns it and must clean it up.
    ParamOwned,
    /// Active read-only views over an owned slot; the base says where the
    /// slot returns to when the last view ends.
    Viewed(u32, ViewBase),
    /// Active exclusive mut borrow; the base says where it returns to.
    MutBorrowed(MutBase),
    MovedOut,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum MutBase {
    Local,
    ParamOwned,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ViewBase {
    Local,
    ParamOwned,
}

/// Modeled resource operations (slot indices are abstract places).
/// The variant names used by the r1 review counterexamples (Make/View/
/// EndView/Mut/EndMut/MoveOut/Free/Clone/ReturnViewLocal/ReturnViewParam)
/// keep their exact shapes; provenance registrations and the storage/mode
/// rejection entries are additive.
#[derive(Clone, Copy, Debug)]
pub enum Op {
    Make(usize),
    /// Register the slot as a borrowed-in `str` param (owner: caller).
    ParamBorrowed(usize),
    /// Register the slot as a move-in `move Str` param (owner: this frame).
    ParamOwned(usize),
    View(usize),
    EndView(usize),
    Mut(usize),
    EndMut(usize),
    MoveOut(usize),
    Free(usize),
    Clone(usize),
    /// `return <view of a locally owned slot>` — always an escape.
    ReturnViewLocal(usize),
    /// `return <view claimed to be a param>` — legal ONLY for a live
    /// borrowed-in param; freed locals, moved-out slots and move-in owned
    /// params all reject.
    ReturnViewParam(usize),
    /// Store a VIEW of the slot into a record field — the first batch
    /// forbids borrowed fields (`verify.field-borrow`).
    StoreFieldView(usize),
    /// Store a VIEW of the slot as a container element — the existing
    /// container-ownership ban (`verify.container-borrow`).
    StoreContainerView(usize),
    /// Store an owned CLONE into a field/container — the legal sibling.
    StoreOwnedClone(usize),
    /// Pass the slot to a `move Str` param WITHOUT the call-site `.move` —
    /// rejected (`verify.ownership-required`).
    CallMoveParamImplicit(usize),
    /// Pass with the explicit `.move` — ownership transfers; the callee
    /// takes over cleanup.
    CallMoveParamExplicit(usize),
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

fn op_slot(op: Op) -> usize {
    match op {
        Op::Make(s)
        | Op::ParamBorrowed(s)
        | Op::ParamOwned(s)
        | Op::View(s)
        | Op::EndView(s)
        | Op::Mut(s)
        | Op::EndMut(s)
        | Op::MoveOut(s)
        | Op::Free(s)
        | Op::Clone(s)
        | Op::ReturnViewLocal(s)
        | Op::ReturnViewParam(s)
        | Op::StoreFieldView(s)
        | Op::StoreContainerView(s)
        | Op::StoreOwnedClone(s)
        | Op::CallMoveParamImplicit(s)
        | Op::CallMoveParamExplicit(s) => s,
    }
}

/// Run a script through the front door. On the first illegal op the model
/// rejects with the frozen code and NOTHING further dispatches natively.
pub fn run_script(ops: &[Op]) -> RuleOutcome {
    let mut slots = [SlotState::Dead; SLOTS];
    let mut ledger: BTreeMap<&'static str, u32> = BTreeMap::new();

    macro_rules! reject {
        ($n:expr, $code:expr) => {
            return RuleOutcome {
                rejection: Some(Rejection {
                    index: $n,
                    code: $code,
                }),
                ledger,
            }
        };
    }
    macro_rules! dispatch {
        ($name:expr) => {
            *ledger.entry($name).or_insert(0) += 1
        };
    }

    for (i, op) in ops.iter().enumerate() {
        let n = i + 1;
        let st = slots.get(op_slot(*op)).copied().unwrap_or(SlotState::Dead);
        // Read-state preconditions shared by value uses of a slot: dead and
        // moved-out sources reject regardless of the use kind.
        macro_rules! require_live {
            () => {
                match st {
                    SlotState::Dead => reject!(n, "verify.use-after-free"),
                    SlotState::MovedOut => reject!(n, "verify.use-after-move"),
                    SlotState::MutBorrowed(_) => reject!(n, "verify.mut-conflict"),
                    _ => {}
                }
            };
        }
        match *op {
            Op::Make(s) => match st {
                SlotState::Dead | SlotState::MovedOut => {
                    slots[s] = SlotState::LocalOwned;
                    dispatch!("own_make");
                }
                _ => reject!(n, "verify.ownership-required"),
            },
            Op::ParamBorrowed(s) => match st {
                SlotState::Dead | SlotState::MovedOut => slots[s] = SlotState::ParamBorrowed,
                _ => reject!(n, "verify.ownership-required"),
            },
            Op::ParamOwned(s) => match st {
                SlotState::Dead | SlotState::MovedOut => slots[s] = SlotState::ParamOwned,
                _ => reject!(n, "verify.ownership-required"),
            },
            Op::View(s) => match st {
                SlotState::LocalOwned => {
                    slots[s] = SlotState::Viewed(1, ViewBase::Local);
                    dispatch!("borrow_read");
                }
                SlotState::Viewed(k, base) => {
                    slots[s] = SlotState::Viewed(k + 1, base);
                    dispatch!("borrow_read");
                }
                // a view over a move-in owned param returns to ParamOwned
                SlotState::ParamOwned => {
                    slots[s] = SlotState::Viewed(1, ViewBase::ParamOwned);
                    dispatch!("borrow_read");
                }
                // a view of an already-borrowed param stays a plain read
                SlotState::ParamBorrowed => dispatch!("borrow_read"),
                SlotState::Dead => reject!(n, "verify.use-after-free"),
                SlotState::MovedOut => reject!(n, "verify.use-after-move"),
                SlotState::MutBorrowed(_) => reject!(n, "verify.mut-conflict"),
            },
            Op::EndView(s) => match slots[s] {
                // the last view ending returns the slot to its owner kind
                SlotState::Viewed(1, ViewBase::Local) => slots[s] = SlotState::LocalOwned,
                SlotState::Viewed(1, ViewBase::ParamOwned) => slots[s] = SlotState::ParamOwned,
                SlotState::Viewed(k, base) if k > 1 => slots[s] = SlotState::Viewed(k - 1, base),
                // model construction error in the test, not a language rule
                _ => panic!("EndView without an active view"),
            },
            Op::Mut(s) => match st {
                SlotState::LocalOwned => {
                    slots[s] = SlotState::MutBorrowed(MutBase::Local);
                    dispatch!("rec_mut");
                }
                SlotState::ParamOwned => {
                    slots[s] = SlotState::MutBorrowed(MutBase::ParamOwned);
                    dispatch!("rec_mut");
                }
                // explicit `mut str` params are a declared legal form in
                // D-03 but are NOT modeled here; the modeled surface stays
                // sound by rejecting (documented model gap, not a rule).
                SlotState::Dead => reject!(n, "verify.use-after-free"),
                SlotState::MovedOut => reject!(n, "verify.use-after-move"),
                _ => reject!(n, "verify.mut-conflict"),
            },
            Op::EndMut(s) => match slots[s] {
                SlotState::MutBorrowed(MutBase::Local) => slots[s] = SlotState::LocalOwned,
                SlotState::MutBorrowed(MutBase::ParamOwned) => slots[s] = SlotState::ParamOwned,
                _ => panic!("EndMut without an active mut borrow"),
            },
            Op::MoveOut(s) => match st {
                SlotState::LocalOwned | SlotState::ParamOwned => slots[s] = SlotState::MovedOut,
                SlotState::Dead => reject!(n, "verify.use-after-free"),
                SlotState::MovedOut => reject!(n, "verify.double-move"),
                SlotState::Viewed(_, _) | SlotState::MutBorrowed(_) | SlotState::ParamBorrowed => {
                    reject!(n, "verify.mut-conflict")
                }
            },
            Op::Free(s) => match st {
                SlotState::LocalOwned | SlotState::ParamOwned => {
                    slots[s] = SlotState::Dead;
                    dispatch!("own_free");
                }
                // freeing a borrowed view param requires ownership
                SlotState::ParamBorrowed => reject!(n, "verify.ownership-required"),
                SlotState::Dead => reject!(n, "verify.use-after-free"),
                SlotState::MovedOut => reject!(n, "verify.use-after-move"),
                SlotState::Viewed(_, _) | SlotState::MutBorrowed(_) => {
                    reject!(n, "verify.mut-conflict")
                }
            },
            Op::Clone(_) => {
                require_live!();
                dispatch!("own_clone");
            }
            Op::ReturnViewLocal(_) => {
                reject!(n, "verify.borrow-escape");
            }
            Op::ReturnViewParam(_) => match st {
                // the owner is the caller's borrowed param: outlives us
                SlotState::ParamBorrowed => dispatch!("borrow_return"),
                // freed local passed off as a "param" view (r1 counterexample)
                SlotState::Dead => reject!(n, "verify.use-after-free"),
                SlotState::MovedOut => reject!(n, "verify.use-after-move"),
                // local owned, or a move-in param whose cleanup boundary is
                // this very frame: the view would escape its owner
                _ => reject!(n, "verify.borrow-escape"),
            },
            Op::StoreFieldView(_) => {
                require_live!();
                reject!(n, "verify.field-borrow");
            }
            Op::StoreContainerView(_) => {
                require_live!();
                reject!(n, "verify.container-borrow");
            }
            Op::StoreOwnedClone(_) => {
                require_live!();
                dispatch!("own_clone");
            }
            Op::CallMoveParamImplicit(_) => {
                require_live!();
                reject!(n, "verify.ownership-required");
            }
            Op::CallMoveParamExplicit(s) => match st {
                SlotState::LocalOwned | SlotState::ParamOwned => {
                    slots[s] = SlotState::MovedOut;
                    // the callee takes over cleanup
                    dispatch!("own_free");
                }
                SlotState::Dead => reject!(n, "verify.use-after-free"),
                SlotState::MovedOut => reject!(n, "verify.double-move"),
                _ => reject!(n, "verify.mut-conflict"),
            },
        }
    }
    RuleOutcome {
        rejection: None,
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
