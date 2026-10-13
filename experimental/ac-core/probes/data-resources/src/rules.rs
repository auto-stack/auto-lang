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
    /// Borrowed-in `str` param (default mode): the owner is the CALLER and
    /// outlives us.
    ParamBorrowed,
    /// Incoming `mut str` param (explicit mode): an exclusive mutable
    /// borrow of a caller buffer, held for the whole body in the first
    /// batch.
    ParamMutBorrowed,
    /// Move-in `move Str` param: this frame owns it and must clean it up.
    ParamOwned,
    /// A static literal: immortal read-only storage, no cleanup risk.
    StaticLit,
    /// Active read-only views over a readable slot; the base says where the
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
    ParamBorrow,
    ParamOwned,
    StaticLit,
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
    /// Register the slot as an incoming `mut str` param: an exclusive
    /// mutable borrow of a caller buffer.
    ParamMutBorrowed(usize),
    /// Register the slot as a move-in `move Str` param (owner: this frame).
    ParamOwned(usize),
    /// Register the slot as a static literal (immortal read-only storage).
    StaticLiteral(usize),
    View(usize),
    EndView(usize),
    Mut(usize),
    EndMut(usize),
    MoveOut(usize),
    Free(usize),
    Clone(usize),
    /// `return <view of a locally owned slot>` — always an escape.
    ReturnViewLocal(usize),
    /// `return <view claimed to be a param>` — legal for a live borrowed-in
    /// param OR a static literal (D-03: 借自参数或常驻字面量); freed locals,
    /// moved-out slots and move-in owned params all reject.
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
        | Op::ParamMutBorrowed(s)
        | Op::ParamOwned(s)
        | Op::StaticLiteral(s)
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
            Op::ParamMutBorrowed(s) => match st {
                SlotState::Dead | SlotState::MovedOut => slots[s] = SlotState::ParamMutBorrowed,
                _ => reject!(n, "verify.ownership-required"),
            },
            Op::ParamOwned(s) => match st {
                SlotState::Dead | SlotState::MovedOut => slots[s] = SlotState::ParamOwned,
                _ => reject!(n, "verify.ownership-required"),
            },
            Op::StaticLiteral(s) => match st {
                SlotState::Dead | SlotState::MovedOut => slots[s] = SlotState::StaticLit,
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
                // a view of an incoming borrowed param is a region over a
                // caller-owned buffer (r2: trackable, ends back at the param)
                SlotState::ParamBorrowed => {
                    slots[s] = SlotState::Viewed(1, ViewBase::ParamBorrow);
                    dispatch!("borrow_read");
                }
                SlotState::StaticLit => {
                    slots[s] = SlotState::Viewed(1, ViewBase::StaticLit);
                    dispatch!("borrow_read");
                }
                SlotState::Dead => reject!(n, "verify.use-after-free"),
                SlotState::MovedOut => reject!(n, "verify.use-after-move"),
                SlotState::MutBorrowed(_) | SlotState::ParamMutBorrowed => {
                    reject!(n, "verify.mut-conflict")
                }
            },
            Op::EndView(s) => match slots[s] {
                // the last view ending returns the slot to its source state
                SlotState::Viewed(1, ViewBase::Local) => slots[s] = SlotState::LocalOwned,
                SlotState::Viewed(1, ViewBase::ParamBorrow) => slots[s] = SlotState::ParamBorrowed,
                SlotState::Viewed(1, ViewBase::ParamOwned) => slots[s] = SlotState::ParamOwned,
                SlotState::Viewed(1, ViewBase::StaticLit) => slots[s] = SlotState::StaticLit,
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
                // mutating THROUGH an incoming `mut str` param is the normal
                // use of that exclusive borrow (r2: modeled; the region is
                // held for the whole body in the first batch)
                SlotState::ParamMutBorrowed => dispatch!("rec_mut"),
                SlotState::Dead => reject!(n, "verify.use-after-free"),
                SlotState::MovedOut => reject!(n, "verify.use-after-move"),
                // readonly borrowed params and immortal literals are not
                // mutable sources
                _ => reject!(n, "verify.mut-conflict"),
            },
            Op::EndMut(s) => match slots[s] {
                SlotState::MutBorrowed(MutBase::Local) => slots[s] = SlotState::LocalOwned,
                SlotState::MutBorrowed(MutBase::ParamOwned) => slots[s] = SlotState::ParamOwned,
                // the incoming `mut str` region spans the whole body; there
                // is no mid-body close form in the first batch — a script
                // that tries one is a test construction error
                _ => panic!("EndMut without an active local mut borrow"),
            },
            Op::MoveOut(s) => match st {
                SlotState::LocalOwned | SlotState::ParamOwned => slots[s] = SlotState::MovedOut,
                SlotState::Dead => reject!(n, "verify.use-after-free"),
                SlotState::MovedOut => reject!(n, "verify.double-move"),
                SlotState::Viewed(_, _) | SlotState::MutBorrowed(_) => {
                    reject!(n, "verify.mut-conflict")
                }
                // borrowed params, incoming mut borrows and immortal
                // literals are not ownable sources (promotion is a copy)
                SlotState::ParamBorrowed | SlotState::ParamMutBorrowed | SlotState::StaticLit => {
                    reject!(n, "verify.ownership-required")
                }
            },
            Op::Free(s) => match st {
                SlotState::LocalOwned | SlotState::ParamOwned => {
                    slots[s] = SlotState::Dead;
                    dispatch!("own_free");
                }
                // freeing borrowed params, incoming mut borrows or
                // immortal literals requires ownership
                SlotState::ParamBorrowed | SlotState::ParamMutBorrowed | SlotState::StaticLit => {
                    reject!(n, "verify.ownership-required")
                }
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
            Op::ReturnViewLocal(_) => match st {
                SlotState::Dead => reject!(n, "verify.use-after-free"),
                SlotState::MovedOut => reject!(n, "verify.use-after-move"),
                _ => reject!(n, "verify.borrow-escape"),
            },
            Op::ReturnViewParam(_) => match st {
                // the owner is the caller's borrowed param: outlives us
                SlotState::ParamBorrowed => dispatch!("borrow_return"),
                // D-03: a view of a static literal may also be returned —
                // immortal storage, no cleanup boundary (r2 restoration)
                SlotState::StaticLit => dispatch!("borrow_return"),
                // views over those two bases are equally returnable
                SlotState::Viewed(_, ViewBase::ParamBorrow)
                | SlotState::Viewed(_, ViewBase::StaticLit) => dispatch!("borrow_return"),
                // freed local passed off as a "param" view (r1 counterexample)
                SlotState::Dead => reject!(n, "verify.use-after-free"),
                SlotState::MovedOut => reject!(n, "verify.use-after-move"),
                // local owned, a view over a local, an incoming mut borrow
                // (no first-batch reborrow downgrade), or a move-in param
                // whose cleanup boundary is this very frame
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
                SlotState::Viewed(_, _) | SlotState::MutBorrowed(_) => {
                    reject!(n, "verify.mut-conflict")
                }
                SlotState::ParamBorrowed | SlotState::ParamMutBorrowed | SlotState::StaticLit => {
                    reject!(n, "verify.ownership-required")
                }
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
