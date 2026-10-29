//! Frozen contract identities, budgets and the outcome taxonomy
//! (PLAN-798 §3/§5.1, frozen by the T-01 `contract-decision.md` gate).
//!
//! These constants are EXPERIMENTAL probe policy for this plan, not a
//! permanent language-syntax ceiling: they bound the limited
//! `av.core-i32/1` profile and its verification/execution admissions.
//! Exceeding a table budget is a *resource* rejection, never an illegal-
//! syntax or unknown-opcode classification.

use std::fmt;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Identities
// ---------------------------------------------------------------------------

/// Logical schema identity of the Typed MIR produced by this probe.
pub const MIR_SCHEMA: &str = "auto.mir.av-probe/1";
/// Wire schema identity of the ABC format emitted/consumed by this probe.
/// /2 (F798-R12-03): the wire carries the unit/module KIND bit
/// (`FLAG_IS_UNIT`) as load-bearing semantics — kind-less /1 artifacts are
/// a different format generation and can never be read as /2 payloads.
pub const ABC_SCHEMA: &str = "auto.abc.av-probe/2";
/// The one execution profile this probe implements.
pub const AV_PROFILE: &str = "av.core-i32/1";
/// Producer identity used in store envelopes and keys (PLAN-798 §5.4).
pub const PRODUCER_ID: &str = "plan798/av-probe";

// ---------------------------------------------------------------------------
// PLAN-820 `av.data/1` identities (contract-decision.md §1, frozen T-01)
// ---------------------------------------------------------------------------

/// Logical schema identity of the data-profile Typed MIR (independent MIR
/// schema: the data type table carries tags the core-i32 schema never had).
pub const MIR_DATA_SCHEMA: &str = "auto.mir.av-data/1";
/// Wire schema identity of the data-profile ABC format. A SEPARATE magic
/// family from `AVAB`: neither generation can be read as the other.
pub const ABC_DATA_SCHEMA: &str = "auto.abc.av-data/1";
/// The data execution profile: wide scalars (i64/u64/f32/f64, data ops only)
/// plus pure-value non-recursive nested records with field places.
pub const AV_DATA_PROFILE: &str = "av.data/1";
/// Producer identity for data-profile units in store envelopes and keys.
pub const DATA_PRODUCER_ID: &str = "plan820/av-data";
/// Wire magic of the data-profile ABC format (`AVAD`).
pub const ABC_DATA_MAGIC: [u8; 4] = *b"AVAD";
/// Wire version of the data-profile ABC format (v1; profile id 2).
pub const ABC_DATA_VERSION: u16 = 1;
/// Numeric profile id carried in the data wire header.
pub const ABC_PROFILE_DATA: u16 = 2;

// ---------------------------------------------------------------------------
// PLAN-821 `av.owned/1` identities (contract-decision.md §1, frozen T-01)
// ---------------------------------------------------------------------------

/// Logical schema identity of the owned-profile Typed MIR.
pub const MIR_OWNED_SCHEMA: &str = "auto.mir.av-owned/1";
/// Wire schema identity of the owned-profile ABC format. Same `AVAD` magic
/// FAMILY as the data profile, but wire VERSION 2 — a different format
/// generation: v1 (data) and v2 (owned) are mutually unreadable (798 r6
/// generation-isolation precedent); a v1 payload in the v2 decoder is a
/// controlled `abc.decode.version` rejection / deterministic stale miss.
pub const ABC_OWNED_SCHEMA: &str = "auto.abc.av-owned/1";
/// The owned execution profile (PLAN-821): char/Str/StrView/MutRef values,
/// owned records, place borrows, move-kill, replace transactions and
/// all-exit cleanup (Design44 §5–7).
pub const AV_OWNED_PROFILE: &str = "av.owned/1";
/// Producer identity for owned-profile units in store envelopes and keys.
pub const OWNED_PRODUCER_ID: &str = "plan821/av-owned";
/// Wire version of the owned-profile ABC format (v2; profile id 3).
pub const ABC_OWNED_VERSION: u16 = 2;
/// Numeric profile id carried in the owned wire header.
pub const ABC_PROFILE_OWNED: u16 = 3;

/// PLAN-821 owned-profile semantic versions — an INDEPENDENT axis from the
/// core-i32 and data tuples (all three profiles' artifacts key separately).
pub const MIR_OWNED_SEMANTICS: MirSemanticsVersion = MirSemanticsVersion(1);
pub const ABC_OWNED_SEMANTICS: AbcSemanticsVersion = AbcSemanticsVersion(1);
pub const AV_OWNED_SEMANTICS: AvSemanticsVersion = AvSemanticsVersion(1);

// ---------------------------------------------------------------------------
// PLAN-822 `av.collections/1` identities (contract-decision.md §1)
// ---------------------------------------------------------------------------

/// Logical schema identity of the collections-profile Typed MIR.
pub const MIR_COLLECTIONS_SCHEMA: &str = "auto.mir.av-collections/1";
/// Wire schema identity of the collections-profile ABC format. Same `AVAD`
/// magic family as data/owned but wire VERSION 3 — a new generation:
/// v1 (data), v2 (owned) and v3 (collections) are mutually unreadable
/// (798 r6 generation-isolation precedent).
pub const ABC_COLLECTIONS_SCHEMA: &str = "auto.abc.av-collections/1";
/// The collections execution profile (PLAN-822): the owned surface
/// (records/Str/places/move-replace) plus typed `List<T>` and nominal
/// variants (Option/Result/finite enums).
pub const AV_COLLECTIONS_PROFILE: &str = "av.collections/1";
/// Producer identity for collections-profile units in store envelopes/keys.
pub const COLLECTIONS_PRODUCER_ID: &str = "plan822/av-collections";
/// Wire version of the collections-profile ABC format (v3; profile id 4).
pub const ABC_COLLECTIONS_VERSION: u16 = 3;
/// Numeric profile id carried in the collections wire header.
pub const ABC_PROFILE_COLLECTIONS: u16 = 4;

/// PLAN-822 collections-profile semantic versions — an INDEPENDENT axis from
/// the three earlier profiles (artifacts key separately; all start at 1).
pub const MIR_COLLECTIONS_SEMANTICS: MirSemanticsVersion = MirSemanticsVersion(1);
pub const ABC_COLLECTIONS_SEMANTICS: AbcSemanticsVersion = AbcSemanticsVersion(1);
pub const AV_COLLECTIONS_SEMANTICS: AvSemanticsVersion = AvSemanticsVersion(1);

// ---------------------------------------------------------------------------
// PLAN-823 `av.closures/1` identities (contract-decision.md r2 §2.2)
// ---------------------------------------------------------------------------

/// Logical schema identity of the closures-profile Typed MIR.
pub const MIR_CLOSURES_SCHEMA: &str = "auto.mir.av-closures/1";
/// Wire schema identity of the closures-profile ABC format. Same `AVAD`
/// magic FAMILY, wire VERSION 5 — v4 (dynamic) and v5 (closures) are
/// mutually unreadable format generations (the 798 r6 isolation precedent).
pub const ABC_CLOSURES_SCHEMA: &str = "auto.abc.av-closures/1";
/// The closures execution profile (PLAN-823): typed owned closure
/// environments, checked indirect calls and move / explicit-clone escape
/// captures; subsumes the collections surface.
pub const AV_CLOSURES_PROFILE: &str = "av.closures/1";
/// Producer identity for closures-profile units in store envelopes/keys.
pub const CLOSURES_PRODUCER_ID: &str = "plan823/av-closures";
/// Wire version of the closures-profile ABC format (v5; profile id 6).
pub const ABC_CLOSURES_VERSION: u16 = 5;
/// Numeric profile id carried in the closures wire header.
pub const ABC_PROFILE_CLOSURES: u16 = 6;

/// PLAN-823 closures-profile semantic versions — an INDEPENDENT axis (all
/// start at 1; the five profiles' artifacts key separately).
pub const MIR_CLOSURES_SEMANTICS: MirSemanticsVersion = MirSemanticsVersion(1);
pub const ABC_CLOSURES_SEMANTICS: AbcSemanticsVersion = AbcSemanticsVersion(1);
pub const AV_CLOSURES_SEMANTICS: AvSemanticsVersion = AvSemanticsVersion(1);

/// Per-env field-count guard (module table family, aligned with
/// `MAX_RECORD_FIELDS`; contract-decision r2 §4).
pub const MAX_ENV_FIELDS: usize = 1024;
// PLAN-824 `av.dynamic/1` identities (contract-decision.md §6, frozen r2:
// profile id 5 / AVAD v4 — id 4 and v3 were taken by `av.collections/1`)
// ---------------------------------------------------------------------------

/// Logical schema identity of the dynamic-profile Typed MIR.
pub const MIR_DYNAMIC_SCHEMA: &str = "auto.mir.av-dynamic/1";
/// Wire schema identity of the dynamic-profile ABC format (`AVAD` family,
/// wire VERSION 4 — v1/v2/v3/v4 are mutually unreadable generations).
pub const ABC_DYNAMIC_SCHEMA: &str = "auto.abc.av-dynamic/1";
/// The dynamic execution profile (PLAN-824): the collections surface plus
/// the checked type-erasure Dyn container (DYN_PACK/DYN_IS/DYN_TAKE over
/// the whitelisted value closure).
pub const AV_DYNAMIC_PROFILE: &str = "av.dynamic/1";
/// Producer identity for dynamic-profile units in store envelopes/keys.
pub const DYNAMIC_PRODUCER_ID: &str = "plan824/av-dynamic";
/// Wire version of the dynamic-profile ABC format (v4; profile id 5).
pub const ABC_DYNAMIC_VERSION: u16 = 4;
/// Numeric profile id carried in the dynamic wire header.
pub const ABC_PROFILE_DYNAMIC: u16 = 5;

/// PLAN-824 dynamic-profile semantic versions — an INDEPENDENT axis (all
/// profiles' artifacts key separately; start at 1).
pub const MIR_DYNAMIC_SEMANTICS: MirSemanticsVersion = MirSemanticsVersion(1);
pub const ABC_DYNAMIC_SEMANTICS: AbcSemanticsVersion = AbcSemanticsVersion(1);
pub const AV_DYNAMIC_SEMANTICS: AvSemanticsVersion = AvSemanticsVersion(1);

/// Single-list element-count guard (collections contract-decision §5).
/// len/cap arithmetic is checked u32→u64 throughout; a breach is a resource
/// rejection (`av.resource.list-elems`), never a syntax classification.
pub const MAX_LIST_ELEMS: usize = 65_536;

// ---------------------------------------------------------------------------
// PLAN-862 `av.entry-unit/1` identities (entry-contract.md §1, frozen T-02)
// ---------------------------------------------------------------------------

/// Logical schema identity of the entry-unit-profile Typed MIR.
pub const MIR_ENTRY_UNIT_SCHEMA: &str = "auto.mir.av-entry-unit/1";
/// Wire schema identity of the entry-unit-profile ABC format. A NEW magic
/// family (`AVAU`, wire version 1): neither the `AVAB` core family nor the
/// `AVAD` v1..v5 generations can be read as it, and vice versa.
pub const ABC_ENTRY_UNIT_SCHEMA: &str = "auto.abc.av-entry-unit/1";
/// The entry-unit execution profile (PLAN-862): the scalar compute surface
/// plus the Unit RESULT channel and the entry-kind contract. No text/record
/// surface (those stay in their own profiles).
pub const AV_ENTRY_UNIT_PROFILE: &str = "av.entry-unit/1";
/// Producer identity for entry-unit-profile units in store envelopes/keys.
pub const ENTRY_UNIT_PRODUCER_ID: &str = "plan862/av-entry-unit";
/// Wire magic of the entry-unit-profile ABC format (`AVAU`).
pub const ABC_ENTRY_UNIT_MAGIC: [u8; 4] = *b"AVAU";
/// Wire version of the entry-unit-profile ABC format (v1 within the new
/// family; profile id 7).
pub const ABC_ENTRY_UNIT_VERSION: u16 = 1;
/// Numeric profile id carried in the entry-unit wire header (1..6 are the
/// core/data/owned/collections/dynamic/closures ids).
pub const ABC_PROFILE_ENTRY_UNIT: u16 = 7;

/// PLAN-862 entry-unit-profile semantic versions — an INDEPENDENT axis (all
/// profiles' artifacts key separately; start at 1).
pub const MIR_ENTRY_UNIT_SEMANTICS: MirSemanticsVersion = MirSemanticsVersion(1);
pub const ABC_ENTRY_UNIT_SEMANTICS: AbcSemanticsVersion = AbcSemanticsVersion(1);
pub const AV_ENTRY_UNIT_SEMANTICS: AvSemanticsVersion = AvSemanticsVersion(1);

/// Per-record field-count guard (module table family; contract-decision §8).
pub const MAX_RECORD_FIELDS: usize = 1024;
/// Field-path depth guard for FIELD_GET/FIELD_SET (u16 domain, explicit cap).
pub const MAX_FIELD_PATH_LEN: usize = 64;

/// Semantic-version tuple carriers. A change in any of these values must
/// invalidate previously persisted artifacts of that stage (probe cache key
/// component; the numeric values are frozen by T-01).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MirSemanticsVersion(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AbcSemanticsVersion(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AvSemanticsVersion(pub u32);

pub const MIR_SEMANTICS: MirSemanticsVersion = MirSemanticsVersion(1);
/// 2 (F798-R12-03): ABC semantics distinguish unit vs complete-module
/// kind; 1 = the kind-less generation whose artifacts must miss/reject.
pub const ABC_SEMANTICS: AbcSemanticsVersion = AbcSemanticsVersion(2);
pub const AV_SEMANTICS: AvSemanticsVersion = AvSemanticsVersion(1);

/// PLAN-820 data-profile semantic versions — an INDEPENDENT axis from the
/// core-i32 tuple above (the two profiles' artifacts key separately; a bump
/// on one axis never invalidates the other). All start at 1.
pub const MIR_DATA_SEMANTICS: MirSemanticsVersion = MirSemanticsVersion(1);
pub const ABC_DATA_SEMANTICS: AbcSemanticsVersion = AbcSemanticsVersion(1);
pub const AV_DATA_SEMANTICS: AvSemanticsVersion = AvSemanticsVersion(1);

/// Capability that admits the closed test-only trace intrinsic set
/// (same string as the HIR front door, `hir.test.trace.v1`).
pub const TRACE_CAPABILITY: &str = "hir.test.trace.v1";

/// PLAN-821: the text runtime capability the text-record source domain
/// derives for resource modules (782: 组合模块派生 `ac.runtime.text.v1`).
/// Admitted ONLY on owned-profile modules at MIR verification.
pub const TEXT_RUNTIME_CAPABILITY: &str = "ac.runtime.text.v1";

// ---------------------------------------------------------------------------
// Frozen budgets (PLAN-798 §3; resource rejection on breach)
// ---------------------------------------------------------------------------

/// Maximum size of one source / HIR / ABC *input* presented to the probe.
pub const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024;
/// Maximum bytes in one codec page (encode flush / decode feed granularity).
pub const MAX_CODEC_PAGE_BYTES: usize = 256 * 1024;
/// Total table entries (types + consts + imports + functions + blocks +
/// slots + instructions) allowed in one MIR/ABC unit before allocation.
pub const MAX_TABLE_ENTRIES: usize = 65_536;
/// Nesting budget enforced at the OLD source/HIR text front door, before the
/// legacy recursive parser runs (probe admission, not a language rule).
pub const MAX_FRONTEND_NESTING: u32 = 128;
/// MIR/ABC verification work-step ceiling per request.
pub const MAX_VERIFY_WORK_STEPS: u64 = 2_000_000;
/// MIR/ABC verification wall ceiling per request.
pub const MAX_VERIFY_WALL: Duration = Duration::from_millis(5_000);
/// AV execution instruction-step ceiling per request.
pub const MAX_AV_INSTR_STEPS: u64 = 1_000_000;
/// AV guest frame ceiling (guest recursion never rides the host stack).
pub const MAX_AV_FRAMES: usize = 1024;
/// AV live slot-data ceiling across all frames.
pub const MAX_AV_SLOT_BYTES: usize = 8 * 1024 * 1024;
/// AV execution wall ceiling per request.
pub const MAX_AV_WALL: Duration = Duration::from_millis(2_000);
/// Instructions between cancellation-flag checks.
pub const CANCEL_CHECK_INTERVAL: u64 = 1024;
/// Test-trace byte ceiling for one execution.
pub const MAX_TRACE_BYTES: usize = 64 * 1024;

/// Slot cell size used for the live-slot-bytes budget accounting.
pub const SLOT_CELL_BYTES: usize = std::mem::size_of::<Value>();

/// Upper bound on the verifier's composite analysis allocations (the
/// definite-initialization bitsets over blocks × slot words, per function),
/// checked BEFORE those structures are allocated (F798-R3-08: a dense
/// 16384-slot/8192-block unit previously allocated ~51.7 MiB of matrices
/// before the work-step budget rejected it). Part of the 16 MiB staging
/// partition of the managed 32 MiB envelope.
pub const MAX_VERIFY_ANALYSIS_BYTES: usize = 16 * 1024 * 1024;
/// Upper bound on one decoded module's structural heap estimate, checked
/// at admission before deep verification (wire length is not a working-set
/// proxy; a tiny wire can declare large tables).
pub const MAX_DECODED_MODULE_BYTES: usize = 16 * 1024 * 1024;

// ---------------------------------------------------------------------------
// Values and traps
// ---------------------------------------------------------------------------

/// A value of the limited profiles. `bool` is not an arbitrary integer: it
/// keeps its own kind end to end. The data profile (PLAN-820) adds the wide
/// scalars `i64`/`u64`/`f32`/`f64` — floats keep their exact bit pattern
/// end to end (no arithmetic exists in this profile, so NaN payloads and
/// signed zeros are never canonicalized) — and immutable pure-value records
/// (shared by `Rc`; structural immutability makes a shallow clone
/// observationally equal to a deep copy).
///
/// The owned profile (PLAN-821, Design44 §4.1/§7) adds `char`, owned `Str`
/// buffers (`StrHandle` — an owning handle, NEVER `Copy`; cloning
/// duplicates the handle = one more owner), read-only `StrView` borrows
/// (frame/slot-rooted or static-const-rooted; a view never owns) and
/// `MutRef` exclusive write tokens. `Rust Copy` is not derived for owned
/// resources by construction (the handle's Drop reports the final release
/// to the arena).
///
/// `PartialEq`/`Eq` are BIT-EXACT for floats (`to_bits`): two NaNs with the
/// same payload are equal, `0.0 != -0.0`. Str compares by BYTES (the
/// allocation id is not identity). The type is `Clone` but not `Copy`
/// since the record/str variants own shared data.
#[derive(Clone, Debug)]
pub enum Value {
    I32(i32),
    /// PLAN-862: the unit value — a zero-width result with no payload. It is
    /// NOT `I32(0)`: the two are different payloads and never compare equal,
    /// never mix in operators and never substitute for each other.
    Unit,
    Bool(bool),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    Record(std::rc::Rc<RecordValue>),
    /// Unicode scalar (PLAN-774): domain-checked (no surrogates, ≤0x10FFFF)
    /// end to end; never implicitly an integer.
    Char(char),
    /// Owned UTF-8 buffer behind an owning handle.
    Str(StrHandle),
    /// Borrowed read-only view: a (frame, slot, generation) root plus a
    /// Unicode-scalar range, or a static view over a const's bytes. A view
    /// has NO drop right and never owns.
    StrView(StrView),
    /// Exclusive write-borrow token over a root slot (PLAN-821 §5.1.3).
    /// Not copyable (exclusivity); released by BORROW_END or frame exit.
    MutRef(MutRefV),
    /// Owned typed list (PLAN-822, `av.collections/1`): an owning handle to
    /// a [`ListBuf`]. NEVER `Copy`; cloning duplicates the handle = one more
    /// owner (shared buffer, same epoch). Structural changes (realloc /
    /// element shift) bump the buffer epoch; stale places must not touch the
    /// new buffer.
    List(ListHandle),
    /// Nominal variant value (PLAN-822): Option/Result/finite enums. `disc`
    /// is the arm index; `payload` is the live arm's payload (no-payload
    /// arms carry `None`). A payload-moved arm (after VARIANT_PROJECT of an
    /// owned payload) also carries `None` — verification keeps those states
    /// apart; the runtime re-check is defense in depth.
    Variant(VariantValue),
    /// Index Place/token (PLAN-822, §5.1.3): a retained reference to ONE
    /// list element — its root (frame, slot), the buffer identity and
    /// STRUCTURAL epoch at mint time, the element index, and the element's
    /// per-element generation at mint time. Structural changes (realloc /
    /// element shift) bump the epoch; in-place element replacement bumps
    /// the element generation — a stale place read/write traps
    /// `av.trap.stale-place` BEFORE any buffer access (defense in depth;
    /// verification gates the admitted shapes).
    Place(ListPlaceV),
    /// Runtime closure reference (PLAN-823, `av.closures/1`): binds the
    /// concrete callee, its signature type-table entry, the environment-
    /// store handle and the binding generation. Copyable METADATA — the
    /// env payload lives in the executor's per-execution env store, so
    /// dropping the reference value itself never frees an env (release is
    /// the executor's Drop/frame-exit path; the store outlives frames).
    Closure(ClosureRefV),
    /// Type-erased dynamic value (PLAN-824, `av.dynamic/1`): an owning
    /// share of a checked [`crate::dynamic::DynValue`] — the whitelist-
    /// clean (value, DynType) pair whose only constructors are
    /// `DynValue::pack` and the portable decoder. Cloning bumps the share
    /// (like every handle); the DYN instructions are the only in-guest
    /// minting path (verification gates the erased closure).
    Dyn(std::rc::Rc<crate::dynamic::DynValue>),
}

/// The `Value::Closure` payload (PLAN-823 §5.1.1): the closure identity —
/// callee function, signature TYPE-table entry, env-store handle and the
/// binding generation (V-03/V-04 handle domains; handles are monotonic
/// per execution, never reused).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClosureRefV {
    pub func: u32,
    pub sig: u32,
    pub env: u64,
    pub gen: u64,
}

/// The `Value::Place` payload: (frame, slot) root + buffer identity +
/// structural epoch + element index + element generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListPlaceV {
    pub frame: u32,
    pub slot: u32,
    pub buf_id: u64,
    pub epoch: u64,
    pub index: u32,
    pub elem_gen: u32,
}

/// One pure-value record: the owning type-table index plus the field values
/// in DECLARATION order. Constructed only by AGGREGATE/FIELD_SET rebuilds;
/// never mutated in place. An uninitialized record SLOT (def-before-use is
/// forbidden by the definite-initialization analysis) holds a placeholder
/// with an empty field list — same never-observed guarantee as the i32/bool
/// typed zero, and structurally distinct from any admitted record value.
#[derive(Clone, Debug)]
pub struct RecordValue {
    pub ty: u32,
    pub fields: Vec<Value>,
}

/// The shared payload of one owned Str buffer. `id` is the arena allocation
/// identity (event-ledger key); `bytes` is the UTF-8 content (invariant:
/// valid UTF-8 — construction goes through `from_utf8`/`String` only).
#[derive(Debug)]
pub struct StrBuf {
    pub id: u64,
    pub bytes: Box<[u8]>,
}

/// Owning handle to an [`StrBuf`] (the `Value::Str` payload). Cloning adds
/// one owner; the LAST handle's drop reports the free to the arena when one
/// is attached (`hook` is `None` for placeholders — the never-read typed
/// zeros — and for out-of-execution copies, e.g. test fixtures, where no
/// ledger exists).
#[derive(Debug)]
pub struct StrHandle {
    pub rc: std::rc::Rc<StrBuf>,
    hook: Option<std::rc::Rc<std::cell::RefCell<crate::owned_text::ArenaCore>>>,
}

impl StrHandle {
    /// A handle with NO arena hook (placeholder / out-of-execution value).
    pub fn detached(bytes: Box<[u8]>) -> Self {
        StrHandle {
            rc: std::rc::Rc::new(StrBuf { id: 0, bytes }),
            hook: None,
        }
    }
    /// A LEDGERED handle: the arena mints the id and receives the final
    /// release report (only the arena constructs these).
    pub fn attached(
        id: u64,
        bytes: Box<[u8]>,
        hook: std::rc::Rc<std::cell::RefCell<crate::owned_text::ArenaCore>>,
    ) -> Self {
        StrHandle {
            rc: std::rc::Rc::new(StrBuf { id, bytes }),
            hook: Some(hook),
        }
    }
    pub fn bytes(&self) -> &[u8] {
        &self.rc.bytes
    }
}

impl Clone for StrHandle {
    fn clone(&self) -> Self {
        StrHandle {
            rc: std::rc::Rc::clone(&self.rc),
            hook: self.hook.clone(),
        }
    }
}

impl Drop for StrHandle {
    fn drop(&mut self) {
        // Exactly-one-strong-ref (this handle) ⇒ this drop is the final
        // release: report the free event while the payload is still alive.
        if let Some(hook) = self.hook.take() {
            if std::rc::Rc::strong_count(&self.rc) == 1 {
                crate::owned_text::ArenaCore::report_free(&hook, self.rc.id, self.rc.bytes.len());
            }
        }
    }
}

/// Borrowed view source (774 SNAPSHOT semantics, source-text.md: "旧构
/// 造的 view 读作陈旧"): a view SHARES the buffer it was created from —
/// a later replace of the owning place leaves the view reading the OLD
/// bytes (the share keeps them alive); it never roots at a mutable place.
/// Static views reference a module constant's bytes (immortal literals).
#[derive(Clone, Debug)]
pub enum StrViewSrc {
    /// An Rc share of the buffer this view was minted over. The share is
    /// NOT ownership: the view has no drop right, and the buffer's free
    /// event fires only when the LAST reference (owner or view) goes.
    Share(StrHandle),
    /// Backed by a module constant's bytes (a literal borrowing source:
    /// immortal). F821-R1-03: the view carries the RESOLVED bytes of the
    /// OWNING unit's constant — a bare const INDEX resolves against the
    /// wrong table when the view crosses function-unit boundaries (each
    /// unit has an independent const index domain).
    Static { buf: std::rc::Rc<Box<[u8]>> },
}

/// The `Value::StrView` payload: source + Unicode-scalar range `[start, end)`.
#[derive(Clone, Debug)]
pub struct StrView {
    pub src: StrViewSrc,
    pub start: u32,
    pub end: u32,
}

impl PartialEq for StrView {
    fn eq(&self, other: &Self) -> bool {
        let bytes_eq = match (&self.src, &other.src) {
            (StrViewSrc::Share(a), StrViewSrc::Share(b)) => a.bytes() == b.bytes(),
            (StrViewSrc::Static { buf: a }, StrViewSrc::Static { buf: b }) => a == b,
            _ => false,
        };
        bytes_eq && self.start == other.start && self.end == other.end
    }
}
impl Eq for StrView {}

/// The `Value::MutRef` payload: exclusive token over (frame, slot) at a
/// generation. Invariants are enforced by the verifier's borrow analysis;
/// the runtime re-checks the generation at use (V-05 defense in depth).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MutRefV {
    pub frame: u32,
    pub slot: u32,
    pub gen: u64,
}

/// The shared payload of one owned list buffer (PLAN-822). `id` is the
/// arena allocation identity (event-ledger key); `epoch` bumps on every
/// STRUCTURAL change (realloc, element insert/remove shift) so stale
/// places can be detected; `cap` is the spare capacity in slots;
/// `elems` are the element values in index order.
#[derive(Debug)]
pub struct ListBuf {
    pub id: u64,
    pub epoch: u64,
    pub cap: u32,
    pub elems: Vec<Value>,
    /// Per-element replacement generations (PLAN-822 §4): position i's
    /// generation bumps on every in-place replacement of that element
    /// (LIST_SET / LIST_FIELD_SET / PLACE_SET); structural shifts carry
    /// the generations WITH their elements. A retained place compares
    /// its mint-time generation against the current one.
    pub elem_gens: Vec<u32>,
}

/// Owning handle to a [`ListBuf`] (the `Value::List` payload). Cloning adds
/// one owner; the LAST handle's drop reports the free to the arena when one
/// is attached (`hook` is `None` for placeholders and out-of-execution
/// copies, mirroring `StrHandle`).
#[derive(Debug)]
pub struct ListHandle {
    pub rc: std::rc::Rc<std::cell::RefCell<ListBuf>>,
    hook: Option<std::rc::Rc<std::cell::RefCell<crate::owned_text::ArenaCore>>>,
}

impl ListHandle {
    /// A handle with NO arena hook (placeholder / out-of-execution value).
    pub fn detached(elems: Vec<Value>) -> Self {
        let gens = vec![0u32; elems.len()];
        ListHandle {
            rc: std::rc::Rc::new(std::cell::RefCell::new(ListBuf {
                id: 0,
                epoch: 0,
                cap: elems.len() as u32,
                elems,
                elem_gens: gens,
            })),
            hook: None,
        }
    }
    /// A LEDGERED handle: the arena mints the id and receives the final
    /// release report (only the arena constructs these).
    pub fn attached(
        id: u64,
        elems: Vec<Value>,
        hook: std::rc::Rc<std::cell::RefCell<crate::owned_text::ArenaCore>>,
    ) -> Self {
        let gens = vec![0u32; elems.len()];
        ListHandle {
            rc: std::rc::Rc::new(std::cell::RefCell::new(ListBuf {
                id,
                epoch: 0,
                cap: elems.len() as u32,
                elems,
                elem_gens: gens,
            })),
            hook: Some(hook),
        }
    }
    pub fn len(&self) -> usize {
        self.rc.borrow().elems.len()
    }
}

impl Clone for ListHandle {
    fn clone(&self) -> Self {
        ListHandle {
            rc: std::rc::Rc::clone(&self.rc),
            hook: self.hook.clone(),
        }
    }
}

impl Drop for ListHandle {
    fn drop(&mut self) {
        // Exactly-one-strong-ref (this handle) ⇒ this drop is the final
        // release: report the free event while the payload is still alive.
        if let Some(hook) = self.hook.take() {
            if std::rc::Rc::strong_count(&self.rc) == 1 {
                let (id, elem_count) = {
                    let b = self.rc.borrow();
                    (b.id, b.elems.len())
                };
                crate::owned_text::ArenaCore::report_free(&hook, id, elem_count);
            }
        }
    }
}

/// The `Value::Variant` payload: nominal type-table index, live arm index,
/// and the live arm's payload (no-payload arms carry `None`). The payload
/// rides an `Rc` (F822-R10-01): a Value clone must be a SHALLOW handle
/// bump for every supported kind — nested Variant snapshot trees are the
/// deep-clone visitor's job, never derived-Clone recursion.
#[derive(Clone, Debug)]
pub struct VariantValue {
    pub ty: u32,
    pub disc: u32,
    pub payload: Option<std::rc::Rc<Value>>,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (I32(a), I32(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (I64(a), I64(b)) => a == b,
            (U64(a), U64(b)) => a == b,
            (F32(a), F32(b)) => a.to_bits() == b.to_bits(),
            (F64(a), F64(b)) => a.to_bits() == b.to_bits(),
            (Record(a), Record(b)) => {
                // Nominal identity first (same type index), then fields.
                a.ty == b.ty && a.fields == b.fields
            }
            // Str compares BY CONTENT (the allocation id is not identity):
            // two independently cloned buffers of equal bytes are equal.
            (Str(a), Str(b)) => a.bytes() == b.bytes(),
            (Char(a), Char(b)) => a == b,
            (StrView(a), StrView(b)) => a == b,
            (MutRef(a), MutRef(b)) => a == b,
            // List compares BY CONTENT element-wise (the allocation id and
            // epoch are not identity), mirroring the Str rule.
            (List(a), List(b)) => {
                let (ra, rb) = (a.rc.borrow(), b.rc.borrow());
                ra.elems == rb.elems
            }
            (Variant(a), Variant(b)) => a.ty == b.ty && a.disc == b.disc && a.payload == b.payload,
            (Place(a), Place(b)) => a == b,
            (Closure(a), Closure(b)) => a == b,
            // Dyn compares by the container's own (value, type) equality —
            // content identity, never the Rc pointer.
            (Dyn(a), Dyn(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for Value {}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::I32(_) => "i32",
            // PLAN-862: the unit value is its own kind — never "i32".
            Value::Unit => "unit",
            Value::Bool(_) => "bool",
            Value::I64(_) => "i64",
            Value::U64(_) => "u64",
            Value::F32(_) => "f32",
            Value::F64(_) => "f64",
            Value::Record(_) => "record",
            Value::Char(_) => "char",
            Value::Str(_) => "Str",
            Value::StrView(_) => "str",
            Value::MutRef(_) => "mut-ref",
            Value::List(_) => "list",
            Value::Variant(_) => "variant",
            Value::Place(_) => "place",
            Value::Closure(_) => "closure",
            Value::Dyn(_) => "dyn",
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Value::I32(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Deterministic deep data-size estimate (bytes) used by the live
    /// slot-data budget: every value cell counts `size_of::<Value>()`;
    /// record levels add their `Vec` header; owned Str buffers count their
    /// byte payload (PLAN-821 §8: 槽+聚合+buffer 合计). Shared `Rc`s are
    /// counted PER REFERENCE — conservative over-counting, never
    /// under-counting (contract-decision §6). `visited` counts every
    /// traversed record node AND field (the frozen "deep_size 按字段计"
    /// fuel rule) — scalar-only programs never enter the record branch and
    /// keep their exact historic step counts.
    /// F822-R12-01: EXPLICIT worklist over OWNED shallow snapshots (Value
    /// clones are Rc/handle bumps for every supported kind) — the frozen
    /// domain admits arbitrarily deep acyclic Variant/Record nesting with
    /// no language depth cap, so the metering walk must not ride the host
    /// stack (same explicit-traversal invariant as cleanup/order-key
    /// discovery). The increment scheme mirrors the historical recursive
    /// one exactly — one node visit per Record/Str/List/Variant item and
    /// one edge visit per pushed field/element/payload — so `visited` and
    /// byte totals are bit-identical to the recursive walk and every
    /// pinned step count is preserved. The dropped snapshots never reach
    /// the last reference (the source still holds each), so no free event
    /// can fire here.
    pub fn deep_size(&self, visited: &mut u64) -> usize {
        let mut total = 0usize;
        let mut stack: Vec<Value> = vec![self.clone()];
        while let Some(x) = stack.pop() {
            let mut b = std::mem::size_of::<Value>();
            match &x {
                Value::Record(r) => {
                    *visited += 1;
                    b += std::mem::size_of::<Vec<Value>>(); // Vec header (len-based)
                    for f in r.fields.iter() {
                        *visited += 1; // the field edge
                        stack.push(f.clone());
                    }
                }
                Value::Str(s) => {
                    *visited += 1;
                    b += s.bytes().len();
                }
                // PLAN-822: every element is counted individually (the frozen
                // per-element fuel rule — a list never hides its data volume
                // behind one cheap instruction).
                Value::List(l) => {
                    *visited += 1;
                    b += std::mem::size_of::<Vec<Value>>();
                    let buf = l.rc.borrow();
                    for e in buf.elems.iter() {
                        *visited += 1; // the element edge
                        stack.push(e.clone());
                    }
                }
                Value::Variant(v) => {
                    *visited += 1;
                    if let Some(p) = v.payload.as_ref() {
                        *visited += 1; // the payload edge
                        stack.push(p.as_ref().clone());
                    }
                }
                // PLAN-824: a Dyn slot's data volume is its inner value's
                // PLUS the carried type-descriptor tree (F-824-R1-03: a
                // resolved descriptor is real slot-resident data — the
                // shared value-data line counts it, megabytes cannot hide
                // behind "bookkeeping").
                Value::Dyn(d) => {
                    *visited += 1;
                    b += dyn_type_bytes(d.ty(), visited);
                    stack.push(d.as_value().clone());
                }
                _ => {}
            }
            total += b;
        }
        total
    }
}

/// Structural byte estimate of one `DynType` tree: a node cell plus its
/// name payloads per node (same conservative cost model as `deep_size`;
/// explicit worklist — the type depth is bounded at construction, but the
/// walk keeps the module's no-host-stack-recursion discipline).
fn dyn_type_bytes(ty: &crate::dynamic::DynType, visited: &mut u64) -> usize {
    use crate::dynamic::{DynArm, DynField, DynType};
    let mut total = 0usize;
    let mut stack: Vec<&DynType> = vec![ty];
    while let Some(t) = stack.pop() {
        *visited += 1;
        total += std::mem::size_of::<DynType>();
        match t {
            DynType::Record {
                owner,
                name,
                fields,
            } => {
                total += owner.len() + name.len();
                for DynField { name, ty } in fields {
                    *visited += 1;
                    // The embedded DynType cell is counted when visited.
                    total = total.saturating_add(name.len()).saturating_add(
                        std::mem::size_of::<DynField>() - std::mem::size_of::<DynType>(),
                    );
                    stack.push(ty);
                }
            }
            DynType::Variant { owner, name, arms } => {
                total += owner.len() + name.len();
                for DynArm { name, payload } in arms {
                    *visited += 1;
                    total = total.saturating_add(name.len()).saturating_add(
                        std::mem::size_of::<DynArm>() - std::mem::size_of::<DynType>(),
                    );
                    if let Some(p) = payload {
                        stack.push(p);
                    } else {
                        // A payload-less arm still occupies its inline option.
                        total = total.saturating_add(std::mem::size_of::<DynType>());
                    }
                }
            }
            DynType::List(elem) => stack.push(elem),
            _ => {}
        }
    }
    total
}

/// Language traps of the scalar contract (i32 arithmetic). These are
/// *runtime* outcomes; a trap on an unexecuted branch must stay latent —
/// it is never folded into a compile-time rejection. PLAN-821 adds the
/// text-domain traps (770/782 runtime-failure semantics as AV traps).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrapKind {
    /// add/sub/mul signed overflow (trap policy is implicit in the profile).
    IntOverflow,
    /// `x / 0`.
    DivByZero,
    /// `MIN / -1`.
    DivOverflow,
    /// `MIN % -1` (traps per the scalar contract, unlike host `%`).
    RemOverflow,
    /// `char_at` index / `slice` range outside the view (782 ABI −10
    /// mirrored as an AV trap; negative/reversed/`end > len` all here).
    StrIndex,
    /// `char.from_codepoint(n)` with an invalid Unicode scalar
    /// (`text.invalid-scalar`, 770).
    CharInvalid,
    /// A frame/slot-rooted view or mut token used after its root's
    /// generation moved (V-05 defense in depth — verification should
    /// prevent this for admitted programs).
    InvalidHandle,
    /// PLAN-824: `DYN_TAKE` of a dynamic value whose runtime type differs
    /// from the instruction's expected type — a RUNTIME CHECKED failure
    /// (not bad input): the source Dyn stays intact in its slot, nothing
    /// is dropped or half-moved.
    DynTypeMismatch,
    /// PLAN-822: list index outside `[0, len)` (or insert outside
    /// `[0, len]`); negative indices land here too. Checked BEFORE any
    /// buffer access.
    ListIndex,
    /// PLAN-822: a place/token minted at an older list epoch used after a
    /// structural change (realloc / element shift) bumped the epoch.
    StalePlace,
    /// PLAN-822: variant payload projected from the wrong arm or from a
    /// payload-moved arm at runtime (defense in depth — static guard
    /// analysis should prevent this for admitted programs).
    VariantBranch,
}

impl TrapKind {
    pub fn code(&self) -> &'static str {
        match self {
            TrapKind::IntOverflow => "av.trap.int-overflow",
            TrapKind::DivByZero => "av.trap.div-by-zero",
            TrapKind::DivOverflow => "av.trap.div-overflow",
            TrapKind::RemOverflow => "av.trap.rem-overflow",
            TrapKind::StrIndex => "av.trap.str-index",
            TrapKind::CharInvalid => "av.trap.char-invalid",
            TrapKind::InvalidHandle => "av.trap.invalid-handle",
            TrapKind::DynTypeMismatch => "av.trap.dyn-type",
            TrapKind::ListIndex => "av.trap.list-index",
            TrapKind::StalePlace => "av.trap.stale-place",
            TrapKind::VariantBranch => "av.trap.variant-branch",
        }
    }
}

/// Resource-limit classification (distinct from language traps and from
/// infrastructure failures). Host stack overflows and process kills are
/// never valid members of this taxonomy.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResourceKind {
    /// PLAN-863: bounded console bytes, never truncated success.
    ConsoleOutput,
    InputTooLarge,
    CodecPage,
    TableEntries,
    VerifySteps,
    VerifyWall,
    InstrSteps,
    Frames,
    SlotBytes,
    ExecutionWall,
    TraceBytes,
    /// Composite analysis allocations (init-dataflow bitsets etc.).
    VerifyMemory,
    /// Decoded module structural heap estimate.
    DecodeMemory,
    /// PLAN-822: single-list element count over `MAX_LIST_ELEMS`.
    ListElems,
    /// PLAN-824: a Dyn type operand's DAG→tree expansion exceeded the
    /// frozen dyn node ceiling (`av.resource.dyn-nodes` — the same stop
    /// family as the value-tree node budgets in `dynamic`/`portable`).
    DynTypeNodes,
}

impl ResourceKind {
    pub fn code(&self) -> &'static str {
        match self {
            ResourceKind::ConsoleOutput => "resource.console-output",
            ResourceKind::InputTooLarge => "av.resource.input-size",
            ResourceKind::CodecPage => "av.resource.codec-page",
            ResourceKind::TableEntries => "av.resource.table-entries",
            ResourceKind::VerifySteps => "av.resource.verify-steps",
            ResourceKind::VerifyWall => "av.resource.verify-wall",
            ResourceKind::InstrSteps => "av.resource.instr-steps",
            ResourceKind::Frames => "av.resource.frames",
            ResourceKind::SlotBytes => "av.resource.slot-bytes",
            ResourceKind::ExecutionWall => "av.resource.execution-wall",
            ResourceKind::TraceBytes => "av.resource.trace-bytes",
            ResourceKind::VerifyMemory => "av.resource.verify-memory",
            ResourceKind::DecodeMemory => "av.resource.decode-memory",
            ResourceKind::ListElems => "av.resource.list-elems",
            ResourceKind::DynTypeNodes => "av.resource.dyn-nodes",
        }
    }
}

/// Outcome taxonomy of one AV execution (success value / language trap /
/// resource / cancellation / infra are distinct states; unsupported and the
/// two admission rejections happen before execution starts).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecOutcome {
    Success {
        value: Value,
        steps: u64,
    },
    Trap {
        kind: TrapKind,
        span: Option<SpanInfo>,
    },
    Resource {
        kind: ResourceKind,
        message: String,
    },
    Cancelled,
    /// Executor-internal failure (budget bookkeeping etc.) — never used to
    /// absorb a language trap or a resource classification.
    Infra(String),
}

/// Source-mapping record attached to traps and diagnostics: the owning
/// function plus a byte span into that function's source snapshot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpanInfo {
    pub func: u32,
    pub start: u32,
    pub end: u32,
}

impl fmt::Display for SpanInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "func#{}:{}..{}", self.func, self.start, self.end)
    }
}

/// FNV-1a 64-bit — the content-fingerprint primitive used by the codec
/// trailer, envelopes and unit keys. A change detector, not cryptography.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// FNV-1a 64-bit over several parts (domain-separated by hashing lengths).
pub fn fnv1a64_parts(parts: &[&[u8]]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for p in parts {
        h ^= p.len() as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
        for b in *p {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

// PLAN-863: an independent wire/profile family; AVAD and AVAU remain frozen.
pub const MIR_HOST_CONSOLE_SCHEMA: &str = "auto.mir.av-host-console/1";
pub const ABC_HOST_CONSOLE_SCHEMA: &str = "auto.abc.av-host-console/1";
pub const AV_HOST_CONSOLE_PROFILE: &str = "av.host-console/1";
pub const HOST_CONSOLE_PRODUCER_ID: &str = "plan863/av-host-console";
pub const ABC_HOST_CONSOLE_MAGIC: [u8; 4] = *b"AVHC";
pub const ABC_HOST_CONSOLE_VERSION: u16 = 1;
pub const ABC_PROFILE_HOST_CONSOLE: u16 = 8;
pub const MIR_HOST_CONSOLE_SEMANTICS: MirSemanticsVersion = MirSemanticsVersion(1);
pub const ABC_HOST_CONSOLE_SEMANTICS: AbcSemanticsVersion = AbcSemanticsVersion(1);
pub const AV_HOST_CONSOLE_SEMANTICS: AvSemanticsVersion = AvSemanticsVersion(1);
