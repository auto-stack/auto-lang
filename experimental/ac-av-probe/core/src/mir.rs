//! Typed MIR data model for the `av.core-i32/1` profile (PLAN-798 §5.2).
//!
//! Shape: explicit basic blocks with a UNIQUE terminator each, typed mutable
//! local/temp slots (deliberately NOT SSA), sequential instructions, and a
//! per-function source map. Module-level tables carry types, constants and
//! logical imports (test trace intrinsics).
//!
//! Trust model: [`MirModule`] is UNCHECKED. The only constructor of
//! [`crate::mir_verify::CheckedMir`] is `verify_mir`; mutating checked
//! content is not possible through the public API (the inner module is
//! private), so any rewrite invalidates the credential by construction.
//!
//! Block/slot ids are positional identities within one function — they are
//! not stable cross-revision ids. Persistence goes through the ABC wire
//! codec plus the probe's unit/manifest envelope, never an enum dump.

use crate::contract::{SpanInfo, AV_PROFILE};

/// Index into a module/unit type table.
pub type TypeIdx = u32;
/// Index into a module/unit const table.
pub const NO_CONST: u32 = u32::MAX;
/// Index into a module/unit function table.
pub type FuncIdx = u32;
/// Index into a module/unit import table.
pub type ImportIdx = u32;
/// Slot id within one function.
pub type SlotId = u32;
/// Block id within one function.
pub type BlockId = u32;

/// Logical owner scope of a module or function unit: schema + logical module
/// name. Absolute filesystem paths never enter this identity (PLAN-798
/// §5.3); content-identical units from different projects share cache keys.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OwnerScope {
    pub schema: String,
    pub module: String,
}

/// A MIR type. The core-i32 profile has exactly two value types plus
/// function signatures for call admission; the data profile (PLAN-820)
/// adds the wide scalars and nominal pure-value records; the owned profile
/// (PLAN-821) adds `char`, owned `Str`, the borrowed `StrView` and the
/// `MutRef` write token.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MirType {
    I32,
    Bool,
    Fn(FnSig),
    I64,
    U64,
    F32,
    F64,
    /// Nominal record type (PLAN-820): identity is (owner_module, name,
    /// full field list with declaration order and per-field type indices).
    /// Two records of equal width but different owner/name never compare
    /// equal — `MirType`'s derived `PartialEq` IS the frozen type-equivalence
    /// rule applied at every consumption point. In the owned profile a
    /// record whose transitive field closure reaches `Str` is a RESOURCE
    /// record (derived, never input-claimed); copyability follows the same
    /// derivation (Design44 §4.2).
    Record(RecordTypeRef),
    /// Read-only borrow of a record value (PLAN-821 R1-F08, 782 ABI-D
    /// `*const` borrowed record params): the frozen language surface's
    /// plain `p Note` parameter mode. Identity mirrors the record's
    /// nominal identity; NOT owned (no kill at the call, the callee
    /// reads fields as externally-rooted views, writes are rejected).
    RecordView(RecordTypeRef),
    /// Unicode scalar (PLAN-774/821): domain-checked, Copy, never int.
    Char,
    /// Owned UTF-8 buffer (PLAN-774/821): single owner, real allocation.
    Str,
    /// Borrowed read-only view (PLAN-774/821): a value of places/temporaries
    /// and borrow returns; NEVER a record field (`mir.field-borrow`).
    StrView,
    /// Exclusive write-borrow token (PLAN-821 §5.1.3).
    MutRef,
    /// Owned typed list (PLAN-822, `av.collections/1`): `elem` is the
    /// element type index. Identity is the element type alone (lists are
    /// structural, not nominal). The admissible element closure is frozen
    /// by the collections contract-decision §2 (borrowed views, `Fn`, and
    /// List-of-List are rejected at verification).
    List(ListTypeRef),
    /// Nominal variant (PLAN-822): Option/Result/finite enums all lower to
    /// this one shape. `arms` in declaration order carry optional payload
    /// type indices; the arm INDEX is the runtime discriminant.
    Variant(VariantTypeRef),
    /// Index Place/token (PLAN-822 §5.1.3, wire tag 15): a retained
    /// element reference type PARAMETERIZED by its referent list
    /// (F822-R3-01: the element type is bound at typing — LIST_PLACE
    /// mints only over a matching list, PLACE_GET/SET type against the
    /// bound element). The runtime additionally re-validates buffer
    /// identity, structural epoch and element generation at every use —
    /// a stale place traps before any access.
    Place(ListTypeRef),
    /// Owned closure type (PLAN-823, `av.closures/1`): identity is the
    /// callee SIGNATURE (structural, like `Fn`) — but a `Closure` slot is a
    /// DISTINCT type from `Fn`: an old `Fn`-typed slot never accepts an
    /// arbitrary closure (§5.1.1), and only `CALL_CLOSURE` consumes a
    /// `Closure` slot. The captured env layout is bound per callee (the
    /// unique CREATE_CLOSURE site, contract-decision r2 §5).
    Closure(FnSig),
    /// The type-erased dynamic type (PLAN-824, wire tag 16): a slot of
    /// this type holds a `Value::Dyn` — a checked DynValue share whose
    /// runtime type is the erased source type. Only the dynamic profile
    /// admits this type; DYN_PACK mints it, DYN_IS/DYN_TAKE consume it.
    Dyn,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnSig {
    pub params: Vec<TypeIdx>,
    pub result: TypeIdx,
}

/// One field of a nominal record type, in declaration order.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MirFieldDef {
    pub name: String,
    pub ty: TypeIdx,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RecordTypeRef {
    pub owner_module: String,
    pub name: String,
    pub fields: Vec<MirFieldDef>,
}

/// PLAN-822: element type reference of a `List<T>` type.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ListTypeRef {
    pub elem: TypeIdx,
}

/// PLAN-822: one arm of a nominal variant type, in declaration order.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VariantArm {
    pub name: String,
    pub payload: Option<TypeIdx>,
}

/// PLAN-822: nominal variant type identity (owner_module, name, arms).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VariantTypeRef {
    pub owner_module: String,
    pub name: String,
    pub arms: Vec<VariantArm>,
}

/// Module constants. Bool constants are `true`/`false` only. The wide
/// scalars store their EXACT BIT PATTERN (`to_bits`) — NaN payloads and
/// signed zeros survive const/copy/call/return round trips; the profile
/// has no float arithmetic that could canonicalize them. The owned profile
/// (PLAN-821) adds `char` (one Unicode scalar) and `Str` (UTF-8 bytes);
/// the SAME Str const serves an owned `Str` slot (materializes a fresh
/// allocation) and a `StrView` slot (materializes an immortal static view
/// over the const bytes — PLAN-770's static borrowing source).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MirConst {
    I32(i32),
    Bool(bool),
    I64(i64),
    U64(u64),
    /// f32 payload as raw bits.
    F32Bits(u32),
    /// f64 payload as raw bits.
    F64Bits(u64),
    /// One Unicode scalar (domain-checked: no surrogates, ≤ 0x10FFFF).
    Char(char),
    /// UTF-8 bytes of a string literal (invariant: valid UTF-8).
    Str(Box<[u8]>),
}

/// Effect classification of an import (mirrors the HIR intrinsic effect).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Effect {
    Observable,
    Pure,
}

/// A logical import: test trace intrinsics in v1. The symbol is the HIR
/// intrinsic symbol (`hir.test.mark_a`), never a host function pointer.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MirImport {
    pub symbol: String,
    pub effect: Effect,
    pub sig: TypeIdx,
    /// r8 env contract: the declared env layout when this import binds a
    /// closure body (`None` for ordinary function imports); link-time
    /// equivalence checks against the target unit's `MirFunction::env`.
    pub env: Option<Vec<TypeIdx>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotKind {
    /// Parameter slot — initialized on frame entry.
    Param,
    /// Named local (`let`/`var`) — initialized by instructions.
    Local,
    /// Compiler temp — initialized by instructions.
    Temp,
}

#[derive(Clone, Debug)]
pub struct MirSlot {
    pub kind: SlotKind,
    pub ty: TypeIdx,
    pub name: Option<String>,
}

/// Binary operations. Arithmetic traps on overflow/div-by-zero/MIN%-1 per
/// the scalar contract; the trap policy is implicit in the profile (frozen
/// by T-01), so it is not carried per instruction.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    AddI32,
    SubI32,
    MulI32,
    DivI32,
    RemI32,
    LtI32,
    LeI32,
    GtI32,
    GeI32,
    EqI32,
    NeI32,
}

impl BinOp {
    pub fn result_is_bool(&self) -> bool {
        matches!(
            self,
            BinOp::LtI32 | BinOp::LeI32 | BinOp::GtI32 | BinOp::GeI32 | BinOp::EqI32 | BinOp::NeI32
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnOp {
    NegI32,
    NotBool,
}

/// Test trace events (PLAN-798 §5.1): observation only, no I/O capability.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TraceEvent {
    MarkA,
    MarkB,
}

#[derive(Clone, Debug)]
pub enum CalleeRef {
    Func(FuncIdx),
    Import(ImportIdx),
}

#[derive(Clone, Debug)]
pub enum MirInstr {
    Const {
        dst: SlotId,
        value: u32,
    },
    Copy {
        dst: SlotId,
        src: SlotId,
    },
    Bin {
        op: BinOp,
        dst: SlotId,
        lhs: SlotId,
        rhs: SlotId,
    },
    Un {
        op: UnOp,
        dst: SlotId,
        operand: SlotId,
    },
    Call {
        callee: CalleeRef,
        dst: SlotId,
        args: Vec<SlotId>,
    },
    Trace {
        event: TraceEvent,
    },
    /// Field-path read (data profile): copies the addressed field value out
    /// of `base` (an rvalue copy — the record itself is untouched). Each
    /// path segment is a declaration-order field index within the record
    /// type of the preceding segment; `verify_mir` checks every segment
    /// against its actual owner and the destination slot against the final
    /// field's type.
    FieldGet {
        dst: SlotId,
        base: SlotId,
        path: Vec<u16>,
    },
    /// Field-path write (data profile): rewrites the addressed field of the
    /// ROOT slot's record value. Pure values are immutable, so the write is
    /// a structural rebuild along the path into a fresh value that is
    /// written back to the root slot. `value` is an already-materialized
    /// slot — the RHS-first materialization of the overlap invariant.
    FieldSet {
        root: SlotId,
        path: Vec<u16>,
        value: SlotId,
    },
    /// Record construction (data profile): builds a fresh record value of
    /// type `record_ty` from `operands` given in DECLARATION-field order.
    /// The adapter/fixture has already evaluated the operands in SOURCE
    /// order; this instruction consumes the mapped slots.
    Aggregate {
        dst: SlotId,
        record_ty: TypeIdx,
        operands: Vec<SlotId>,
    },
    // -- PLAN-821 owned surface (av.owned/1; contract-decision §4) -------
    /// Explicit deep clone (782 `r.clone()` / 770 `s.clone()`): fresh
    /// independent buffers per owned field, scalars by value; empty Str
    /// allocates nothing. `dst` receives the new owner; `src` is unchanged.
    Clone {
        dst: SlotId,
        src: SlotId,
    },
    /// Whole-root ownership transfer (782 `x.move`): `src` must be an
    /// owned value (Str / resource record) that is initialized, not
    /// already moved and free of live borrows; the commit kills `src`
    /// (move-kill). Copyable sources are rejected (`mir.ownership-required`).
    Move {
        dst: SlotId,
        src: SlotId,
    },
    /// Read-only borrow of an owned Str slot (770 `s.to(str)`): `dst`
    /// receives a `StrView` rooted at `src`'s slot. Views have no drop
    /// right; conflicts on the root are conservative (root granularity).
    View {
        dst: SlotId,
        src: SlotId,
    },
    /// Exclusive write-borrow token over a Str/resource-record root slot.
    /// Root-conservative exclusivity: no other token and no root
    /// move/replace while live.
    MutRef {
        dst: SlotId,
        src: SlotId,
    },
    /// End a VIEW or MUTREF token (kind from the token slot's type).
    /// Released roots accept conflicting access again.
    BorrowEnd {
        token: SlotId,
    },
    /// Explicit drop: an initialized owned value cleans up exactly once
    /// (reverse-creation order for its buffers) and the slot becomes
    /// Dropped. Dropping an uninitialized/moved/dropped slot is
    /// `mir.drop-uninit`.
    Drop {
        slot: SlotId,
    },
    /// `s.len()` — Unicode-scalar count of a view (result i32).
    StrLen {
        dst: SlotId,
        view: SlotId,
    },
    /// `s.len_bytes()` — UTF-8 byte length of a view (result i32).
    StrLenBytes {
        dst: SlotId,
        view: SlotId,
    },
    /// `s.char_at(i)` — scalar-indexed access (result char); out of range
    /// traps `av.trap.str-index` at runtime.
    StrCharAt {
        dst: SlotId,
        view: SlotId,
        index: SlotId,
    },
    /// `s.slice(start, end)` — borrowed `[start, end)` scalar subslice of a
    /// view (result StrView sharing the source). Invalid ranges trap.
    StrSlice {
        dst: SlotId,
        view: SlotId,
        start: SlotId,
        end: SlotId,
    },
    /// `a + b` on text — concatenation into a fresh OWNED buffer.
    StrConcat {
        dst: SlotId,
        lhs: SlotId,
        rhs: SlotId,
    },
    /// `s.to(Str)` — owned copy of the viewed range (fresh allocation).
    StrToOwned {
        dst: SlotId,
        view: SlotId,
    },
    /// View equality/inequality (782 eq/ne): byte-wise comparison of the
    /// two views' ranges (result Bool).
    StrCmp {
        eq: bool,
        dst: SlotId,
        lhs: SlotId,
        rhs: SlotId,
    },
    /// `c.to_codepoint()` — char to i32 (always succeeds).
    CharToCodepoint {
        dst: SlotId,
        ch: SlotId,
    },
    /// R2-F08' (mut-record write-back machinery): DESTRUCTIVE whole-field
    /// move-out — `dst` receives the addressed field's value (an Rc share
    /// for records); the ROOT slot dies (single-extraction, use-once).
    /// Adapter-internal: lowers 782 `mut` record params' write-back
    /// wrapper. Owned profile only.
    FieldMove {
        dst: SlotId,
        root: SlotId,
        path: Vec<u16>,
    },
    /// `char.from_codepoint(n)` — i32 to char; invalid scalars trap
    /// `av.trap.char-invalid` at runtime (never a verify error, 770).
    CharFromCodepoint {
        dst: SlotId,
        code: SlotId,
    },
    // -- PLAN-822 collections surface (av.collections/1; contract-decision
    // §3; opcodes 0x1D..0x2B) -------------------------------------------
    /// LIST_NEW: an empty list of type `list_ty` with spare capacity `cap`
    /// (checked against `MAX_LIST_ELEMS`; allocation is ledgered).
    ListNew {
        dst: SlotId,
        list_ty: TypeIdx,
        cap: u32,
    },
    /// LIST_LEN: element count as i32.
    ListLen {
        dst: SlotId,
        src: SlotId,
    },
    /// LIST_CAP: spare capacity as i32.
    ListCap {
        dst: SlotId,
        src: SlotId,
    },
    /// LIST_GET: copy element `index` out (Copy element types only; owned
    /// elements are read via LIST_REMOVE / LIST_FIELD_GET). Out-of-range
    /// indices trap `av.trap.list-index` BEFORE any buffer access.
    ListGet {
        dst: SlotId,
        base: SlotId,
        index: SlotId,
    },
    /// LIST_PUSH: append `value` (may realloc — epoch bump). Prepare →
    /// capacity/borrow check → commit transfers the source; on failure the
    /// old list and the source slot are unchanged.
    ListPush {
        root: SlotId,
        value: SlotId,
    },
    /// LIST_INSERT: insert `value` at `index ∈ [0, len]`, shifting the tail
    /// (epoch bump; shift is billed per element).
    ListInsert {
        root: SlotId,
        index: SlotId,
        value: SlotId,
    },
    /// LIST_SET: in-place replace of element `index`: the old element is
    /// dropped exactly once and `value` is transferred at commit (LHS
    /// reservation semantics per PLAN-821; index and RHS evaluated once).
    ListSet {
        root: SlotId,
        index: SlotId,
        value: SlotId,
    },
    /// LIST_REMOVE: move element `index` OUT into `dst` (owned transfer),
    /// shifting the tail left (epoch bump). Stale places die with the bump;
    /// recomputed in-range indices read the shifted values.
    ListRemove {
        dst: SlotId,
        root: SlotId,
        index: SlotId,
    },
    /// LIST_FIELD_GET: copy a scalar field of element `index`'s record out
    /// along `path` (declaration-order field indices).
    ListFieldGet {
        dst: SlotId,
        base: SlotId,
        index: SlotId,
        path: Vec<u16>,
    },
    /// LIST_FIELD_SET: write `value` into a field of element `index`'s
    /// record along `path` (the first-case core: `list[i].inner.text`
    /// replace). RHS is pre-materialized; the old field value is dropped
    /// and the source transferred at commit; the element's generation
    /// bumps (the buffer epoch does NOT).
    ListFieldSet {
        root: SlotId,
        index: SlotId,
        path: Vec<u16>,
        value: SlotId,
    },
    /// VARIANT_PACK: construct arm `disc` of `variant_ty`; a payload arm
    /// requires an initialized source slot (transferred at commit), a
    /// no-payload arm requires `None`.
    VariantPack {
        dst: SlotId,
        variant_ty: TypeIdx,
        disc: u32,
        payload: Option<SlotId>,
    },
    /// VARIANT_DISC: current arm index as i32.
    VariantDisc {
        dst: SlotId,
        src: SlotId,
    },
    /// VARIANT_GUARD: `dst` = Bool (current arm == `disc`). Records the
    /// guard FACT in the verifier's lattice; never traps at runtime.
    VariantGuard {
        dst: SlotId,
        src: SlotId,
        disc: u32,
    },
    /// VARIANT_PROJECT: move the live arm's payload out into `dst` (owned
    /// transfer; the arm enters payload-moved state) — or, for Copy
    /// payloads, read the value leaving the arm intact. Requires a live
    /// guard fact for `disc` on `root`; re-projection of a moved arm is
    /// `mir.variant.payload-moved` (verify) / `av.trap.variant-branch`
    /// (runtime defense).
    VariantProject {
        dst: SlotId,
        root: SlotId,
        disc: u32,
    },
    /// VARIANT_SWITCH: switch `root` to arm `disc`: the pre-materialized
    /// `payload` transfers in at commit, then the previous LIVE arm (if
    /// not payload-moved) is dropped exactly once. All guard facts on the
    /// root die with the switch.
    VariantSwitch {
        root: SlotId,
        disc: u32,
        payload: Option<SlotId>,
    },
    /// LIST_PLACE (PLAN-822 §5.1.3, opcode 0x2C): mint `dst` (a `Place`
    /// slot) as a retained reference to `base[index]`, snapshotting the
    /// buffer id, the STRUCTURAL epoch and the element's per-element
    /// generation at mint time. Out-of-range indices trap
    /// `av.trap.list-index` before any access.
    ListPlace {
        dst: SlotId,
        base: SlotId,
        index: SlotId,
    },
    /// PLACE_GET (opcode 0x2D): read the Copy element value a place
    /// refers to. A place whose buffer id, structural epoch or element
    /// generation no longer matches traps `av.trap.stale-place` BEFORE
    /// any buffer access (structural change after mint, or an in-place
    /// replacement of that element).
    PlaceGet {
        dst: SlotId,
        src: SlotId,
    },
    /// PLACE_SET (opcode 0x2E): replace the element a place refers to
    /// (the same precheck/commit transaction as LIST_SET; the element's
    /// generation bumps at commit, retiring THIS place and any sibling
    /// place minted before the replacement). Stale validation as
    /// PLACE_GET.
    PlaceSet {
        place: SlotId,
        value: SlotId,
    },
    // -- PLAN-823 closure surface (av.closures/1; contract-decision r2 §3) --
    /// CREATE_CLOSURE (opcode 0x32): build the owned closure env from
    /// `captures` in operand order (prepare/commit two-phase — all
    /// fallible clone allocations happen before any source transfer), bind
    /// `callee` + its signature, and publish the `Closure`-typed value to
    /// `dst`. r8 env contract: the callee is a `CalleeRef` (Func in a
    /// whole module, Import in a unit); its DECLARED env layout must equal
    /// the captures' slot types position-by-position; the site-unique
    /// rule stays (whole modules).
    CreateClosure {
        dst: SlotId,
        callee: CalleeRef,
        captures: Vec<CaptureOperand>,
    },
    /// CALL_CLOSURE (opcode 0x33): indirect call through the `Closure`
    /// slot — runtime re-checks the env-store handle (stale handles trap
    /// `av.trap.invalid-handle`), then pushes a fresh frame bound to the
    /// env. `args` must match the closure signature's params.
    CallClosure {
        dst: SlotId,
        closure: SlotId,
        args: Vec<SlotId>,
    },
    /// ENV_GET (opcode 0x34): read ONE field of the CURRENT frame's bound
    /// closure environment (capture-order index). The read is a handle
    /// share — List/Str/Record/Variant values share (interior mutability
    /// of lists is the mutable-env-field path); the env retains ownership.
    /// Legal only inside the unique closure body.
    EnvGet {
        dst: SlotId,
        field: u16,
    },
    /// DYN_PACK (PLAN-824 `av.dynamic/1`, opcode 0x2F): erase the typed
    /// slot `src` into the Dyn slot `dst` — an explicit, checked type
    /// erasure. Verification admits only the whitelisted type closure
    /// (scalars/char/Str/record/List/Variant; Fn, borrows and Places are
    /// rejected at ANY depth); the executor re-checks via the DynValue
    /// pack admission. CLONE semantics (v1 wire): the source stays valid —
    /// ownership of shared payloads follows the last handle drop (the
    /// frozen "pack 须 move 或显式 clone" rule's explicit-clone form; the
    /// guest's CLONE/MOVE remain the value-level controls).
    DynPack {
        dst: SlotId,
        src: SlotId,
    },
    /// DYN_IS (opcode 0x30): runtime type test — `dst: Bool` is true iff
    /// the Dyn in `src` holds a value of the static type `ty` under the
    /// frozen type equivalence. Never traps; never consumes the source.
    DynIs {
        dst: SlotId,
        src: SlotId,
        ty: TypeIdx,
    },
    /// DYN_TAKE (opcode 0x31): checked recovery — on match, `dst` (typed
    /// `ty`) receives the value; on mismatch the instruction TRAPS
    /// `av.trap.dyn-type` (a runtime checked failure, not bad input): the
    /// source Dyn stays intact in its slot, nothing dropped or moved.
    DynTake {
        dst: SlotId,
        src: SlotId,
        ty: TypeIdx,
    },
}

/// One CREATE_CLOSURE capture operand's mode (PLAN-823 T-03, wire tag
/// 0=move / 1=clone / 2=borrow inside the v5 closure opcode).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureKind {
    /// Whole-root MOVE transfer (kills the source slot).
    Move,
    /// Explicit deep CLONE (fresh independent buffers; source untouched).
    Clone,
    /// BORROW capture (§5.1.3): statically-provable non-escaping
    /// synchronous use only — the env field is a SHARE view minted over
    /// the creator's slot; the source is PINNED until the closure drops;
    /// the closure value may never escape its creating function.
    Borrow,
    /// MOVE-FROM-ENV capture (r8/R1-F01, §5.1.4/§5.5): `src` is a FIELD
    /// index of the CURRENT frame's bound env — the DEDICATED transfer
    /// op. The parent field is taken out (moved-out state; re-reading or
    /// re-transferring it in the same body rejects at verify) and lands
    /// in the child env, which alone owns it from commit on.
    MoveFromEnv,
}

/// One CREATE_CLOSURE capture operand: the mode plus the source slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureOperand {
    pub kind: CaptureKind,
    pub src: SlotId,
}

#[derive(Clone, Debug)]
pub enum Terminator {
    Jump {
        target: BlockId,
    },
    Branch {
        cond: SlotId,
        then: BlockId,
        else_: BlockId,
    },
    Return {
        value: SlotId,
    },
}

#[derive(Clone, Debug)]
pub struct MirBlock {
    pub instrs: Vec<MirInstr>,
    pub term: Terminator,
}

/// One function body in MIR. `spans` maps FLAT instruction indices
/// (block-major order: block 0 instrs, block 1 instrs, …) to source info;
/// entries are sorted by `instr` and are diagnostics-only.
#[derive(Clone, Debug)]
pub struct MirFunction {
    pub owner: OwnerScope,
    pub name: String,
    pub sig: TypeIdx,
    pub slots: Vec<MirSlot>,
    pub blocks: Vec<MirBlock>,
    pub entry: BlockId,
    /// Source snapshot fingerprint this function was lowered from.
    pub source_snapshot: u64,
    pub spans: Vec<(u32, SpanInfo)>,
    /// r8 env contract: `Some(layout)` marks a CLOSURE BODY and declares
    /// its env field types in capture order (ENV_GET types against THIS,
    /// so a body UNIT verifies standalone); `None` = ordinary function.
    pub env: Option<Vec<TypeIdx>>,
}

impl MirFunction {
    /// Flat instruction count (spans key domain).
    pub fn flat_instr_count(&self) -> u32 {
        self.blocks.iter().map(|b| b.instrs.len() as u32).sum()
    }

    /// Flat index of the first instruction of `block`.
    pub fn block_base(&self, block: BlockId) -> u32 {
        self.blocks[..block as usize]
            .iter()
            .map(|b| b.instrs.len() as u32)
            .sum()
    }
}

/// An UNCHECKED MIR module (or single-function unit when `funcs.len() == 1`
/// and `is_unit` is set — units address callees through imports only).
#[derive(Clone, Debug)]
pub struct MirModule {
    pub profile: String,
    pub owner: OwnerScope,
    pub is_unit: bool,
    pub types: Vec<MirType>,
    pub consts: Vec<MirConst>,
    pub imports: Vec<MirImport>,
    pub funcs: Vec<MirFunction>,
    pub entry: FuncIdx,
    /// Document-level capabilities this unit requires (e.g. trace).
    pub capabilities: Vec<String>,
}

impl MirModule {
    pub fn new(owner: OwnerScope) -> Self {
        MirModule {
            profile: AV_PROFILE.to_string(),
            owner,
            is_unit: false,
            types: Vec::new(),
            consts: Vec::new(),
            imports: Vec::new(),
            funcs: Vec::new(),
            entry: 0,
            capabilities: Vec::new(),
        }
    }

    pub fn unit(owner: OwnerScope) -> Self {
        let mut m = MirModule::new(owner);
        m.is_unit = true;
        m
    }

    /// A data-profile module (`av.data/1`, PLAN-820). The profile string —
    /// not the owner — is what `verify_mir` gates feature sets on.
    pub fn new_data(owner: OwnerScope) -> Self {
        let mut m = MirModule::new(owner);
        m.profile = crate::contract::AV_DATA_PROFILE.to_string();
        m
    }

    /// A data-profile single-function unit.
    pub fn data_unit(owner: OwnerScope) -> Self {
        let mut m = MirModule::new_data(owner);
        m.is_unit = true;
        m
    }

    /// An owned-profile module (`av.owned/1`, PLAN-821).
    pub fn new_owned(owner: OwnerScope) -> Self {
        let mut m = MirModule::new(owner);
        m.profile = crate::contract::AV_OWNED_PROFILE.to_string();
        m
    }

    /// An owned-profile single-function unit.
    pub fn owned_unit(owner: OwnerScope) -> Self {
        let mut m = MirModule::new_owned(owner);
        m.is_unit = true;
        m
    }

    /// A collections-profile module (`av.collections/1`, PLAN-822). The
    /// collections profile SUBSUMES the owned feature set and adds the
    /// List/Variant surface.
    pub fn new_collections(owner: OwnerScope) -> Self {
        let mut m = MirModule::new(owner);
        m.profile = crate::contract::AV_COLLECTIONS_PROFILE.to_string();
        m
    }

    /// A collections-profile single-function unit.
    pub fn collections_unit(owner: OwnerScope) -> Self {
        let mut m = MirModule::new_collections(owner);
        m.is_unit = true;
        m
    }

    /// A dynamic-profile module (`av.dynamic/1`, PLAN-824). The dynamic
    /// profile SUBSUMES the collections feature set and adds the
    /// checked-erasure Dyn surface.
    pub fn new_dynamic(owner: OwnerScope) -> Self {
        let mut m = MirModule::new(owner);
        m.profile = crate::contract::AV_DYNAMIC_PROFILE.to_string();
        m
    }

    /// A dynamic-profile single-function unit.
    pub fn dynamic_unit(owner: OwnerScope) -> Self {
        let mut m = MirModule::new_dynamic(owner);
        m.is_unit = true;
        m
    }

    /// Whether this module declares the data profile.
    pub fn is_data_profile(&self) -> bool {
        self.profile == crate::contract::AV_DATA_PROFILE
    }

    /// Whether this module declares the owned profile (PLAN-821). The
    /// owned profile SUBSUMES the data feature set (wide-scalar data ops
    /// and FIELD_*/AGGREGATE) and adds the owned surface.
    pub fn is_owned_profile(&self) -> bool {
        self.profile == crate::contract::AV_OWNED_PROFILE
    }

    /// Whether this module declares the collections profile (PLAN-822).
    /// The collections profile SUBSUMES the owned feature set.
    pub fn is_collections_profile(&self) -> bool {
        self.profile == crate::contract::AV_COLLECTIONS_PROFILE
    }

    /// A closures-profile module (`av.closures/1`, PLAN-823). The
    /// closures profile SUBSUMES the collections feature set and adds the
    /// closure surface (owned envs, checked indirect calls, capture
    /// prepare/commit).
    pub fn new_closures(owner: OwnerScope) -> Self {
        let mut m = MirModule::new(owner);
        m.profile = crate::contract::AV_CLOSURES_PROFILE.to_string();
        m
    }

    /// A closures-profile single-function unit.
    pub fn closures_unit(owner: OwnerScope) -> Self {
        let mut m = MirModule::new_closures(owner);
        m.is_unit = true;
        m
    }

    /// Whether this module declares the closures profile (PLAN-823). The
    /// closures profile SUBSUMES the collections feature set.
    pub fn is_closures_profile(&self) -> bool {
        self.profile == crate::contract::AV_CLOSURES_PROFILE
    }

    /// Whether this module declares the dynamic profile (PLAN-824). The
    /// dynamic profile SUBSUMES the collections feature set.
    pub fn is_dynamic_profile(&self) -> bool {
        self.profile == crate::contract::AV_DYNAMIC_PROFILE
    }

    /// Resolve a type to its scalar shape for verification (`None` = Fn,
    /// Record, or the owned profile's Char/Str/StrView/MutRef — the latter
    /// have their own typing arms and never ride ScalarShape).
    pub fn type_shape(&self, idx: TypeIdx) -> Option<ScalarShape> {
        match self.types.get(idx as usize)? {
            MirType::I32 => Some(ScalarShape::I32),
            MirType::Bool => Some(ScalarShape::Bool),
            MirType::I64 => Some(ScalarShape::I64),
            MirType::U64 => Some(ScalarShape::U64),
            MirType::F32 => Some(ScalarShape::F32),
            MirType::F64 => Some(ScalarShape::F64),
            MirType::Fn(_)
            | MirType::Record(_)
            | MirType::RecordView(_)
            | MirType::Char
            | MirType::Str
            | MirType::StrView
            | MirType::MutRef
            | MirType::List(_)
            | MirType::Variant(_)
            | MirType::Place(_)
            | MirType::Closure(_)
            | MirType::Dyn => None,
        }
    }

    /// Walk a field path from a record type index, returning the type the
    /// path ends at (or `None` when a segment's owner record lacks the
    /// field index). Shared by verification and layout derivation.
    pub fn resolve_field_path(&self, mut ty: TypeIdx, path: &[u16]) -> Option<(TypeIdx, TypeIdx)> {
        // (final type index, immediate owner record type index)
        let mut owner = ty;
        for seg in path {
            match self.types.get(ty as usize)? {
                MirType::Record(r) | MirType::RecordView(r) => {
                    // R1-F08: a borrowed record view resolves fields exactly
                    // like its record (reads only; writes are typed away).
                    let f = r.fields.get(*seg as usize)?;
                    owner = ty;
                    ty = f.ty;
                }
                _ => return None,
            }
        }
        Some((ty, owner))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScalarShape {
    I32,
    Bool,
    I64,
    U64,
    F32,
    F64,
}

impl MirInstr {
    /// Whether THIS instruction can raise a language trap at runtime
    /// (F798-R8-02 shared classifier): every Bin (overflow / div / rem
    /// traps) and the unary MIN negation. Diagnostics-provenance
    /// requirements (sidecar coverage on restore) key off this set, so it
    /// must list the full trap-capable instruction family of the frozen
    /// profile — never just one shape.
    pub fn can_trap(&self) -> bool {
        match self {
            MirInstr::Bin { .. } => true,
            MirInstr::Un {
                op: UnOp::NegI32, ..
            } => true,
            // PLAN-822: index traps and the runtime variant defense.
            MirInstr::ListGet { .. }
            | MirInstr::ListSet { .. }
            | MirInstr::ListInsert { .. }
            | MirInstr::ListRemove { .. }
            | MirInstr::ListFieldGet { .. }
            | MirInstr::ListFieldSet { .. }
            | MirInstr::VariantProject { .. }
            | MirInstr::ListPlace { .. }
            | MirInstr::PlaceGet { .. }
            | MirInstr::PlaceSet { .. } => true,
            _ => false,
        }
    }
}

/// Marker consumed by builders to avoid accidentally treating an unchecked
/// module as verified (the Checked credential lives in
/// [`crate::mir_verify::CheckedMir`]).
pub struct MirModuleUncheckedMarker;

impl MirModuleUncheckedMarker {
    pub fn note() -> &'static str {
        "MirModule is unchecked; only verify_mir produces a CheckedMir"
    }
}

/// Interning helpers shared by the HIR adapter and unit assembly.
impl MirModule {
    pub fn intern_type(&mut self, t: MirType) -> TypeIdx {
        if let Some(i) = self.types.iter().position(|x| *x == t) {
            return i as TypeIdx;
        }
        self.types.push(t);
        (self.types.len() - 1) as TypeIdx
    }

    pub fn intern_const(&mut self, c: MirConst) -> u32 {
        if let Some(i) = self.consts.iter().position(|x| *x == c) {
            return i as u32;
        }
        self.consts.push(c);
        self.consts.len() as u32 - 1
    }

    pub fn intern_import(&mut self, imp: MirImport) -> ImportIdx {
        if let Some(i) = self.imports.iter().position(|x| *x == imp) {
            return i as ImportIdx;
        }
        self.imports.push(imp);
        self.imports.len() as ImportIdx - 1
    }

    pub fn sig_of(&self, idx: TypeIdx) -> Option<&FnSig> {
        match self.types.get(idx as usize)? {
            MirType::Fn(s) => Some(s),
            _ => None,
        }
    }

    /// Total table-entry count (reporting helper; the FROZEN budget is
    /// enforced per table family — see mir_verify and contract-decision §6).
    pub fn table_entries(&self) -> usize {
        self.types.len()
            + self.consts.len()
            + self.imports.len()
            + self.funcs.len()
            + self
                .funcs
                .iter()
                .map(|f| f.slots.len() + f.blocks.len() + f.flat_instr_count() as usize)
                .sum::<usize>()
    }

    /// Deterministic structural heap estimate (bytes) of this module as it
    /// lives in memory — the staging/hot partition accounting primitive.
    /// Wire length is NOT a working-set proxy (F798-R3-08): decoded clones,
    /// slot/block tables and span maps are real allocations the ledger must
    /// cover. The estimate uses `len()` (not capacity) so it is stable
    /// across allocator behavior; it deliberately OVER-counts fixed
    /// per-allocation headers, never under-counts the payload bytes.
    pub fn heap_estimate(&self) -> usize {
        const STR: usize = std::mem::size_of::<String>(); // 24
        const VEC: usize = std::mem::size_of::<usize>() * 3; // 24
        let mut b = std::mem::size_of::<MirModule>()
            + self.profile.len()
            + self.owner.schema.len()
            + self.owner.module.len()
            + STR * 3
            + self
                .capabilities
                .iter()
                .map(|c| c.len() + STR)
                .sum::<usize>()
            + VEC * 4;
        b += self
            .types
            .iter()
            .map(|t| match t {
                MirType::I32
                | MirType::Bool
                | MirType::I64
                | MirType::U64
                | MirType::F32
                | MirType::F64
                | MirType::Char
                | MirType::Str
                | MirType::StrView
                | MirType::MutRef => 0, // inline enum, table vec counted below
                MirType::Fn(s) => s.params.len() * std::mem::size_of::<TypeIdx>() + VEC,
                // PLAN-822: List carries one element index inline; Variant
                // owns its arm table (names + payload indices); Place is
                // an inline capability type.
                MirType::List(_) | MirType::Place(_) => 0,
                // PLAN-823: a Closure owns its signature's param vec (like Fn).
                MirType::Closure(s) => s.params.len() * std::mem::size_of::<TypeIdx>() + VEC,
                // PLAN-824: the erased Dyn type is an inline capability
                // type (no table payload).
                MirType::Dyn => 0,
                MirType::Variant(v) => {
                    v.owner_module.len()
                        + v.name.len()
                        + STR * 2
                        + v.arms.len() * std::mem::size_of::<VariantArm>()
                        + v.arms.iter().map(|a| a.name.len() + STR).sum::<usize>()
                        + VEC
                }
                // R4-F09: a RecordView OWNS its RecordTypeRef — the owner/
                // name strings and the field table (names + type indices)
                // are real heap payloads it directly carries, exactly like
                // a Record (the old inline-zero branch under-counted a
                // 3MB field-name payload to zero).
                MirType::Record(r) | MirType::RecordView(r) => {
                    r.owner_module.len()
                        + r.name.len()
                        + STR * 2
                        + r.fields.len()
                            * (std::mem::size_of::<TypeIdx>() + std::mem::size_of::<MirFieldDef>())
                        + r.fields.iter().map(|f| f.name.len() + STR).sum::<usize>()
                        + VEC
                }
            })
            .sum::<usize>()
            + self.types.len() * std::mem::size_of::<MirType>()
            + VEC;
        b += self.consts.len() * std::mem::size_of::<MirConst>()
            + VEC
            + self
                .consts
                .iter()
                .map(|c| match c {
                    MirConst::Str(bytes) => bytes.len(),
                    _ => 0,
                })
                .sum::<usize>();
        b += self
            .imports
            .iter()
            .map(|i| i.symbol.len() + STR)
            .sum::<usize>()
            + self.imports.len() * std::mem::size_of::<MirImport>()
            + VEC;
        b += self.funcs.len() * std::mem::size_of::<MirFunction>() + VEC;
        for f in &self.funcs {
            b += f.name.len() + STR;
            b += f
                .slots
                .iter()
                .map(|s| s.name.as_ref().map_or(0, |n| n.len() + STR))
                .sum::<usize>()
                + f.slots.len() * std::mem::size_of::<MirSlot>()
                + VEC;
            b += f.blocks.len() * std::mem::size_of::<MirBlock>() + VEC;
            for blk in &f.blocks {
                for instr in &blk.instrs {
                    match instr {
                        MirInstr::Call { args, .. } => {
                            b += args.len() * std::mem::size_of::<SlotId>() + VEC
                        }
                        MirInstr::Aggregate { operands, .. } => {
                            b += operands.len() * std::mem::size_of::<SlotId>() + VEC
                        }
                        MirInstr::FieldGet { path, .. }
                        | MirInstr::FieldSet { path, .. }
                        | MirInstr::FieldMove { path, .. }
                        | MirInstr::ListFieldGet { path, .. }
                        | MirInstr::ListFieldSet { path, .. } => {
                            b += path.len() * std::mem::size_of::<u16>() + VEC
                        }
                        MirInstr::CallClosure { args, .. } => {
                            b += args.len() * std::mem::size_of::<SlotId>() + VEC
                        }
                        MirInstr::CreateClosure { captures, .. } => {
                            b += captures.len() * std::mem::size_of::<CaptureOperand>() + VEC
                        }
                        _ => {}
                    }
                }
                b += blk.instrs.len() * std::mem::size_of::<MirInstr>() + VEC;
            }
            b += f.spans.len() * std::mem::size_of::<(u32, SpanInfo)>() + VEC;
            if let Some(env) = &f.env {
                b += env.len() * std::mem::size_of::<TypeIdx>() + VEC;
            }
        }
        b
    }
}
