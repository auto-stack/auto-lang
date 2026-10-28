//! Typed HIR data model for the core-i32 and scalar-i32 profiles
//! (PLAN-741 / PLAN-767).
//!
//! `Unchecked*` structures are produced by the descriptor binder
//! (`crate::descriptor`): every textual reference has been resolved to a
//! typed, category-correct ID scoped to its owning table. They carry source
//! spans for downstream diagnostics. The semantic verifier (`crate::verify`)
//! consumes them and produces `CheckedModule` (added in T-03).
//!
//! Equality on these structures is *semantic*: spans and the
//! `mutable_explicit` presence bit are excluded, so author-form vs
//! explicit-form vs canonical-roundtrip binds compare equal. The bundle's
//! [`HirDomain`] identity DOES participate: bundles bound under different
//! descriptor identities never compare equal.

use crate::atom_text::Span;
use std::fmt;

macro_rules! typed_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub struct $name(pub u32);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

typed_id!(/// Index into `Module::types`.
TypeId);
typed_id!(/// Index into `Module::defs`.
DefId);
typed_id!(/// Index into `Module::bodies`.
BodyId);
typed_id!(/// Index into `Body::locals`.
LocalId);
typed_id!(/// Index into `Body::exprs`.
ExprId);
typed_id!(/// Index into `Body::places`.
PlaceId);
typed_id!(/// Index into `Body::blocks`.
BlockId);
typed_id!(/// Index into `Body::loops` (id texts in registration order).
LoopId);
typed_id!(/// Field declaration index within the owner record type
/// (PLAN-771). The owner is derived per path segment from the preceding
/// segment's record type — the id alone is never a global identity.
FieldId);

/// Numeric projection of a typed ID, used by the binder's shared lookup and
/// diagnostic helpers.
pub trait IdNum {
    fn num(&self) -> u32;
}

macro_rules! impl_id_num {
    ($($t:ty),*) => {
        $(impl IdNum for $t {
            fn num(&self) -> u32 {
                self.0
            }
        })*
    };
}

impl_id_num!(TypeId, DefId, BodyId, LocalId, ExprId, PlaceId, BlockId, LoopId, FieldId);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuiltinType {
    I32,
    Bool,
    /// Borrowed UTF-8 slice (`str`, PLAN-770). A view, never owns.
    StrSlice,
    /// Owned UTF-8 buffer (`Str`, PLAN-770). Single owner, LIFO cleanup.
    StrOwned,
    /// Independent Unicode scalar type (`char`, PLAN-770). u32 scalar value
    /// carried on the I32 ABI channel; never implicitly int.
    Char,
    /// Unit (`unit`, PLAN-862): the no-value RESULT type. It has no runtime
    /// width and no operators; only a function result may name it, and only
    /// the canonical [`ExprKind::Unit`] expression carries it.
    Unit,
}

impl BuiltinType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BuiltinType::I32 => "i32",
            BuiltinType::Bool => "bool",
            BuiltinType::StrSlice => "str",
            BuiltinType::StrOwned => "Str",
            BuiltinType::Char => "char",
            BuiltinType::Unit => "unit",
        }
    }
}

/// A type as written at a use site: a builtin enum shorthand or a reference
/// into the module type table. Builtin-vs-Ref unification is a verifier-level
/// rule; the binder keeps both forms distinct.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TypeUse {
    Builtin(BuiltinType),
    Ref(TypeId),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnSig {
    pub params: Vec<TypeUse>,
    pub result: TypeUse,
}

/// One field of a record type, in declaration order (PLAN-771 §5.1). The
/// declaration order is the layout order (774 D-05) and participates in
/// semantic equality; field names are unique per record (verifier rule).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RecordField {
    pub name: String,
    pub ty: TypeUse,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RecordShape {
    pub fields: Vec<RecordField>,
}

#[derive(Clone, Debug)]
pub enum TypeKind {
    I32,
    Bool,
    /// `str` (PLAN-770): borrowed slice kind for named type entries.
    Str,
    /// `Str` (PLAN-770): owned buffer kind for named type entries.
    StrOwned,
    /// `char` (PLAN-770): Unicode scalar kind for named type entries.
    Char,
    Function(FnSig),
    Record(RecordShape),
    /// Owned typed list `List<T>` (PLAN-864). `elem` is the element type;
    /// the admissible element closure is the 822 element domain (int, bool,
    /// Str, non-container record, nominal variant). Identity is structural
    /// via the element type — lists are never nominal.
    List {
        elem: TypeUse,
    },
    /// Nominal variant (PLAN-864): `?T`/`Result<T,E>` and declared finite
    /// `enum`s all bind to this one shape, mirroring `MirType::Variant`.
    /// `id_text` carries the variant name; arms are declaration order and
    /// the arm INDEX is the runtime discriminant. A payload `None` arm is
    /// the no-payload case. Non-recursive by verifier rule.
    Variant {
        arms: Vec<VariantArmDecl>,
    },
}

/// One arm of a [`TypeKind::Variant`], in declaration order (PLAN-864).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VariantArmDecl {
    pub name: String,
    pub payload: Option<TypeUse>,
}

#[derive(Clone, Debug)]
pub struct TypeDef {
    pub id_text: String,
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LocalRole {
    Param,
    Binding,
}

/// Ownership mode of a parameter local (PLAN-770, 774 D-03/R1-04.1/R2-02.4).
/// Defaults to [`ParamMode::View`] for every text param; `mut` and `move`
/// are explicit declarations only. The mode carries ownership semantics and
/// therefore participates in semantic equality.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParamMode {
    /// Read-only borrow (default for `str` and `Str` parameters).
    View,
    /// Exclusive borrow (`mut str` = borrowed-and-mutable,
    /// `mut Str` = exclusive borrow of an owned value).
    MutBorrow,
    /// Ownership transfer in (`move Str`): the call site must mark `.move`,
    /// the callee owns the value and cleans it up at its exits.
    MoveOwn,
}

impl ParamMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ParamMode::View => "view",
            ParamMode::MutBorrow => "mut",
            ParamMode::MoveOwn => "move",
        }
    }
}

impl LocalRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            LocalRole::Param => "param",
            LocalRole::Binding => "binding",
        }
    }
}

#[derive(Clone, Debug)]
pub struct LocalDef {
    pub id_text: String,
    pub ty: TypeUse,
    pub name: String,
    pub role: LocalRole,
    pub index: Option<u32>,
    pub mutable: bool,
    /// Whether `mutable:` was written explicitly (recorded per the Atom text
    /// contract; excluded from semantic equality).
    pub mutable_explicit: bool,
    /// Parameter passing/ownership mode (PLAN-770 + PLAN-771 unified per
    /// 774 D-03/04). Defaults to `View`; `MutBorrow` is the explicit
    /// exclusive borrow (text `mut str`/`mut Str`, record `mut` param,
    /// ABI-E `*mut`), `MoveOwn` the text ownership transfer (`move Str`).
    /// Only meaningful for `param` locals; bindings always carry `View`.
    /// Participates in semantic equality (different modes are different
    /// contracts).
    pub param_mode: ParamMode,
    pub span: Span,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Effect {
    Observable,
    Pure,
}

impl Effect {
    pub fn as_str(&self) -> &'static str {
        match self {
            Effect::Observable => "observable",
            Effect::Pure => "pure",
        }
    }
}

/// Receiver shape of a type-bound method (PLAN-771 §5.3). `Static` methods
/// have no receiver; `View` methods take the receiver as their first
/// signature param (a read-only view borrow, ABI-D) — the "receiver is
/// evaluated exactly once" invariant falls out of the call structure.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReceiverKind {
    Static,
    View,
}

impl ReceiverKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReceiverKind::Static => "static",
            ReceiverKind::View => "view",
        }
    }
}

#[derive(Clone, Debug)]
pub enum DefKind {
    Function {
        name: String,
        ty: TypeId,
        body: BodyId,
    },
    /// Type-bound method inside a record type's declaration (PLAN-771).
    /// The signature (`ty`) carries the receiver as `params[0]` for
    /// `View` receivers; `record` is the owning nominal type.
    Method {
        name: String,
        ty: TypeId,
        body: BodyId,
        record: TypeId,
        receiver: ReceiverKind,
    },
    Intrinsic {
        symbol: String,
        effect: Effect,
        ty: TypeId,
    },
}

impl DefKind {
    /// The function type of this def, if it has one (functions, methods
    /// and intrinsics all do).
    pub fn fn_type(&self) -> Option<TypeId> {
        match self {
            DefKind::Function { ty, .. } => Some(*ty),
            DefKind::Method { ty, .. } => Some(*ty),
            DefKind::Intrinsic { ty, .. } => Some(*ty),
        }
    }

    /// The body of this def, if it has one.
    pub fn fn_body(&self) -> Option<BodyId> {
        match self {
            DefKind::Function { body, .. } | DefKind::Method { body, .. } => Some(*body),
            DefKind::Intrinsic { .. } => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Def {
    pub id_text: String,
    pub kind: DefKind,
    pub span: Span,
}

impl Def {
    pub fn name_for_diagnostic(&self) -> &str {
        match &self.kind {
            DefKind::Function { name, .. } => name,
            DefKind::Method { name, .. } => name,
            DefKind::Intrinsic { symbol, .. } => symbol,
        }
    }
}

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
    Eq,
    Ne,
}

impl BinOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinOp::AddI32 => "add_i32",
            BinOp::SubI32 => "sub_i32",
            BinOp::MulI32 => "mul_i32",
            BinOp::DivI32 => "div_i32",
            BinOp::RemI32 => "rem_i32",
            BinOp::LtI32 => "lt_i32",
            BinOp::LeI32 => "le_i32",
            BinOp::GtI32 => "gt_i32",
            BinOp::GeI32 => "ge_i32",
            BinOp::Eq => "eq",
            BinOp::Ne => "ne",
        }
    }

    /// Arithmetic whose overflow is policy-governed (requires the
    /// `overflow` field in Atom text and traps on the chosen policy).
    /// Division/remainder traps are inherent to those ops, and comparisons
    /// cannot overflow — neither carries the field.
    pub fn has_overflow_field(&self) -> bool {
        matches!(self, BinOp::AddI32 | BinOp::SubI32 | BinOp::MulI32)
    }

    /// True only for the original core-i32-draft operator set; the core
    /// domain's verifier rejects every other op/node (PLAN-767 §5.1: the
    /// widened shared enum must not widen the old profile).
    pub fn in_core_profile(&self) -> bool {
        matches!(self, BinOp::AddI32 | BinOp::MulI32 | BinOp::LtI32)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnaryOp {
    NegI32,
    NotBool,
}

impl UnaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            UnaryOp::NegI32 => "neg_i32",
            UnaryOp::NotBool => "not_bool",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogicOp {
    And,
    Or,
}

impl LogicOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogicOp::And => "and",
            LogicOp::Or => "or",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OverflowPolicy {
    Trap,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CallBinding {
    pub param: u32,
    pub arg: u32,
    /// Whether the call site marked this argument `.move` (PLAN-770):
    /// ownership of an owned `Str` argument transfers to the callee, the
    /// argument local dies at this call. Semantic.
    pub moved: bool,
}

/// Field-binding of one constructor argument (PLAN-771 §5.2): `field` is
/// the declaration index in the record's field table, `arg` the index into
/// the ctor's `eval_args` (source evaluation order). `bindings` are stored
/// sorted by `field`; the eval order lives in `eval_args` alone, so a
/// right-then-left named construction never folds into a declaration-order
/// evaluation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CtorBinding {
    pub field: u32,
    pub arg: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExprKind {
    ReadLocal {
        local: LocalId,
    },
    Constant {
        value: i32,
    },
    /// PLAN-862: the unit value — the canonical operand of a `return` in a
    /// Unit function. It carries no payload, has no runtime width, and is
    /// the ONLY expression whose type is `unit`.
    Unit,
    Binary {
        op: BinOp,
        overflow: OverflowPolicy,
        lhs: ExprId,
        rhs: ExprId,
    },
    /// Eager unary operation (`-i32` / `!bool`), PLAN-767.
    Unary {
        op: UnaryOp,
        operand: ExprId,
    },
    /// Lazy boolean conjunction/disjunction with short-circuit evaluation,
    /// PLAN-767. Distinct from `Binary` on purpose: mapping `&&`/`||` onto
    /// an eager two-operand op would evaluate the RHS unconditionally.
    Logical {
        op: LogicOp,
        lhs: ExprId,
        rhs: ExprId,
    },
    Call {
        callee: DefId,
        eval_args: Vec<ExprId>,
        bindings: Vec<CallBinding>,
    },
    /// String literal (PLAN-770). The value is the decoded UTF-8 text
    /// (escapes resolved at the lexing front door); a static, immortal
    /// borrowing source. Auto-promotes to an owned `Str` in owning
    /// contexts (AP-4 at runtime).
    StrLit {
        value: String,
    },
    /// Character literal (PLAN-770): exactly one Unicode scalar, carried
    /// as its u32 code point on the I32 ABI channel.
    CharLit {
        scalar: u32,
    },
    /// `s.len()` — Unicode scalar count of a text operand (result i32).
    TextLen {
        operand: ExprId,
    },
    /// `s.len_bytes()` — UTF-8 byte length of a text operand (result i32).
    TextLenBytes {
        operand: ExprId,
    },
    /// `s.char_at(i)` — scalar-indexed char access (result char).
    TextCharAt {
        operand: ExprId,
        index: ExprId,
    },
    /// `s.slice(start, end)` — borrowed `[start, end)` scalar slice
    /// (result str; no allocation, views the operand).
    TextSlice {
        operand: ExprId,
        start: ExprId,
        end: ExprId,
    },
    /// `a + b` on text — concatenation into a fresh owned buffer
    /// (result Str; AP-3).
    TextConcat {
        lhs: ExprId,
        rhs: ExprId,
    },
    /// `s.clone()` — independent owned copy (result Str; AP-2).
    TextClone {
        operand: ExprId,
    },
    /// `s.to(Str)` — promotion to an owned copy (result Str; AP-4 form).
    TextToOwned {
        operand: ExprId,
    },
    /// `s.to(str)` — borrowing view of a text operand (result str; the
    /// operand stays the owner, nothing transfers).
    TextToBorrowed {
        operand: ExprId,
    },
    /// `c.to_codepoint()` — char to i32 (always succeeds).
    CharToCodepoint {
        operand: ExprId,
    },
    /// `char.from_codepoint(n)` — i32 to char; invalid scalars are a
    /// runtime rejection (`text.invalid-scalar`), not a verify error.
    CharFromCodepoint {
        operand: ExprId,
    },
    /// Record construction (PLAN-771): every field initialized exactly
    /// once. `eval_args` is the source-order evaluation list (side effects
    /// run left-to-right as written, never re-sorted by field order);
    /// `bindings` maps each field to its argument. The expr's type is the
    /// record type itself.
    RecordCtor {
        record: TypeId,
        eval_args: Vec<ExprId>,
        bindings: Vec<CtorBinding>,
    },
    /// Field read through a (possibly nested) path (PLAN-771). Each
    /// `FieldId` is the declaration index within the record type of the
    /// preceding segment (the first within `base`'s record type); the
    /// verifier checks every segment against its actual owner. Reading is
    /// an rvalue (a copy of the field for records).
    FieldGet {
        base: ExprId,
        path: Vec<FieldId>,
    },
    /// Read-only borrow of a record value (view param / readonly receiver
    /// argument, PLAN-771; ABI-D `*const`). `base` is evaluated exactly
    /// once into a temp when it is not already a place.
    BorrowView {
        base: ExprId,
    },
    /// Exclusive mutable borrow of a record local (explicit `mut` param
    /// argument, PLAN-771; ABI-E `*mut`). The target must be a mutable
    /// local (a `var` binding or a `mut` param); at most one active
    /// `BorrowMut` per local per call (774 D-04 exclusivity).
    BorrowMut {
        local: LocalId,
    },
    /// Explicit deep clone of a record value (PLAN-782): recursively
    /// duplicates every owned `Str` field (one allocation per non-empty
    /// buffer) and copies scalar/copyable fields by value. The source
    /// stays valid; the two owners' buffers are independent. Only the
    /// text-record domain accepts this node.
    RecordClone {
        record: TypeId,
        base: ExprId,
    },
    /// Whole-root ownership transfer `.move` (PLAN-782): a first-class
    /// wrapper expression usable in let/assign/return/call-argument
    /// positions of the text-record domain. `inner` must be a whole-local
    /// read — moving a field or sub-expression is `verify.partial-move`;
    /// the source root dies at the wrapper. The text domain keeps its
    /// `CallBinding.moved` marker; in the text-record domain that marker
    /// is rejected and this node is the canonical form.
    Move {
        inner: ExprId,
    },
    // -- PLAN-864 collections surface (source domain of av.collections/1) --
    /// `List<T>.new()` — an empty list of the named list type. The
    /// lowering fixes the initial capacity (4, the executor's first-grow
    /// step); the ledger shows exactly one buffer allocation per `.new()`.
    ListNew {
        list: TypeId,
    },
    /// `xs.len()` — element count (result i32).
    ListLen {
        list: ExprId,
    },
    /// `xs.push(v)` (result unit, consumed only by `Stmt::Eval`). Copy
    /// elements pass by value; owned elements require a move-source.
    ListPush {
        list: ExprId,
        value: ExprId,
    },
    /// `xs.insert(i, v)` (result unit).
    ListInsert {
        list: ExprId,
        index: ExprId,
        value: ExprId,
    },
    /// `xs.get(i)` — copy OUT for Copy element types only (int/bool);
    /// owned-element reads go through field views (`FieldGet` rooted at a
    /// `ListGet`, fused by the lowering into LIST_FIELD_GET / view
    /// sequences) or LIST_REMOVE.
    ListGet {
        list: ExprId,
        index: ExprId,
    },
    /// `xs.set(i, v)` (result unit); the old element drops exactly once
    /// at commit.
    ListSet {
        list: ExprId,
        index: ExprId,
        value: ExprId,
    },
    /// `xs.remove(i)` — element transfer out (owned) / copy out (Copy).
    ListRemove {
        list: ExprId,
        index: ExprId,
    },
    /// `xs.clear()` (result unit): the lowering expands this into a
    /// remove-from-tail loop (`LIST_LEN` + `LIST_REMOVE` + `DROP` per
    /// owned element) — there is no LIST_CLEAR opcode in profile 4 and
    /// none is added.
    ListClear {
        list: ExprId,
    },
    /// `Some(e)/None/Ok(e)/Err(e)/E.A/E.A(e)` — variant construction
    /// (result = the nominal variant type). `disc` is the arm index;
    /// a payload arm transfers its operand at commit.
    VariantCtor {
        variant: TypeId,
        disc: u32,
        payload: Option<ExprId>,
    },
    /// `is` dispatch test (result bool, PLAN-864): true iff the variant
    /// local read in `root` currently holds arm `disc`. The root must be
    /// a whole-local read of a variant type. The lowering emits
    /// VARIANT_GUARD (a MUST guard fact, never a trap).
    VariantGuardTest {
        root: ExprId,
        disc: u32,
    },
    /// `is` payload projection (result = the arm payload type): move-out
    /// for owned payloads, copy for Copy payloads. Verify requires a live
    /// guard fact for `disc` on the root — the resolve lowering of `is`
    /// only projects inside the positively-guarded arm.
    VariantProject {
        root: ExprId,
        disc: u32,
    },
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub id_text: String,
    pub ty: TypeUse,
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PlaceKind {
    Local,
    /// A field path rooted at a local (PLAN-771): each `FieldId` is the
    /// declaration index within the record type of the preceding segment.
    /// The place's `ty` is the final field's type; writes go through the
    /// root local's storage (value semantics: the path addresses into the
    /// local itself, never into a copy).
    FieldPath(Vec<FieldId>),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Access {
    Mutable,
    ReadOnly,
}

#[derive(Clone, Debug)]
pub struct Place {
    pub id_text: String,
    pub kind: PlaceKind,
    pub local: LocalId,
    pub ty: TypeUse,
    pub access: Access,
    pub span: Span,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Stmt {
    Let {
        local: LocalId,
        value: ExprId,
    },
    Return {
        value: ExprId,
    },
    If {
        cond: ExprId,
        then: BlockId,
        els: Option<BlockId>,
    },
    Loop {
        id: LoopId,
        body: BlockId,
    },
    Assign {
        place: PlaceId,
        value: ExprId,
    },
    Break {
        target: LoopId,
    },
    Continue {
        target: LoopId,
    },
    /// Explicit cleanup of an owned local (PLAN-770). The source frontend
    /// never emits this — cleanup is compiler-scheduled; programmatic
    /// bundles may carry it and the verifier audits it against the same
    /// LIFO / unique-responsibility rules.
    Drop {
        local: LocalId,
    },
    /// PLAN-864: a unit-typed expression evaluated for effect only — the
    /// statement form of `xs.push(v)` / `xs.insert(..)` / `xs.set(..)` /
    /// `xs.clear()`. The result carries no slot (Unit is never a MIR slot
    /// operand in this domain); the lowering evaluates the expression and
    /// discards. Never legal where a value is consumed (verify rule).
    Eval {
        value: ExprId,
    },
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id_text: String,
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

/// One function body: locals, places, expressions and blocks all scope to the
/// body; types and defs scope to the module.
#[derive(Clone, Debug)]
pub struct Body {
    pub id_text: String,
    pub owner: DefId,
    pub entry: BlockId,
    pub locals: Vec<LocalDef>,
    pub places: Vec<Place>,
    pub exprs: Vec<Expr>,
    pub blocks: Vec<Block>,
    /// Loop registration order; `LoopId(i)` indexes here.
    pub loops: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Module {
    pub id_text: String,
    pub path: Option<String>,
    pub types: Vec<TypeDef>,
    pub defs: Vec<Def>,
    pub bodies: Vec<Body>,
    pub span: Span,
}

/// Authoritative HIR identity domain the bundle was bound under
/// (PLAN-767 §5.1). Derived from the descriptor triple — never guessed from
/// the operator set — and carried from binder through verify, `CheckedModule`
/// and the canonical writer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HirDomain {
    /// `auto.hir.core.draft / 1 / core-i32-draft` (PLAN-741).
    Core,
    /// `auto.hir.scalar.draft / 1 / scalar-i32-draft` (PLAN-767).
    Scalar,
    /// `auto.hir.text.draft / 1 / text-i32-draft` (PLAN-770): the scalar
    /// surface plus str/Str/char with the 774 data-resource contract.
    Text,
    /// `auto.hir.record.draft / 1 / record-i32-draft` (PLAN-771; identity
    /// frozen by the 774 interface-contract §7.2 table).
    Record,
    /// `auto.hir.text-record.draft / 1 / text-record-i32-draft`
    /// (PLAN-782): the union surface of the text and record profiles —
    /// owned `Str` fields inside nominal records under the 774
    /// data-resource contract.
    TextRecord,
    /// `auto.hir.entry-unit.draft / 1 / entry-unit-draft` (PLAN-862): the
    /// scalar compute surface plus the ENTRY surface — a Unit (`void`)
    /// function result, the entry matrix and the single-file script form.
    /// Unit is a RESULT-only type in this domain.
    EntryUnit,
    /// `auto.hir.collections.draft / 1 / collections-draft` (PLAN-864): the
    /// source-domain twin of `av.collections/1` — typed `List<T>` locals,
    /// nominal `?T`/`Result<T,E>`/finite `enum` variants with payload
    /// construction and `is` dispatch (resolve lowers to guard/project),
    /// `for-in` iteration (resolve lowers to an index loop) and owned-type
    /// element views under the 822 execution contract (no new MIR
    /// instructions; profile id 4 is consumed unchanged).
    Collections,
}

impl HirDomain {
    pub fn as_str(&self) -> &'static str {
        match self {
            HirDomain::Core => "core-i32-draft",
            HirDomain::Scalar => "scalar-i32-draft",
            HirDomain::Text => "text-i32-draft",
            HirDomain::Record => "record-i32-draft",
            HirDomain::TextRecord => "text-record-i32-draft",
            HirDomain::EntryUnit => "entry-unit-draft",
            HirDomain::Collections => "collections-draft",
        }
    }

    pub fn schema(&self) -> &'static str {
        match self {
            HirDomain::Core => "auto.hir.core.draft",
            HirDomain::Scalar => "auto.hir.scalar.draft",
            HirDomain::Text => "auto.hir.text.draft",
            HirDomain::Record => "auto.hir.record.draft",
            HirDomain::TextRecord => "auto.hir.text-record.draft",
            HirDomain::EntryUnit => "auto.hir.entry-unit.draft",
            HirDomain::Collections => "auto.hir.collections.draft",
        }
    }

    pub fn revision(&self) -> u32 {
        1
    }
}

/// Root of one parsed document: document-level `requires` capabilities plus
/// one or more modules.
#[derive(Clone, Debug)]
pub struct Bundle {
    pub domain: HirDomain,
    pub requires: Vec<String>,
    pub modules: Vec<Module>,
}

// ---------------------------------------------------------------------------
// Semantic equality (spans and presence bits excluded)
// ---------------------------------------------------------------------------

impl PartialEq for TypeDef {
    fn eq(&self, other: &Self) -> bool {
        self.id_text == other.id_text
            && match (&self.kind, &other.kind) {
                (TypeKind::Function(a), TypeKind::Function(b)) => {
                    a.params == b.params && a.result == b.result
                }
                (TypeKind::Record(a), TypeKind::Record(b)) => a == b,
                (TypeKind::List { elem: a }, TypeKind::List { elem: b }) => a == b,
                (TypeKind::Variant { arms: a }, TypeKind::Variant { arms: b }) => a == b,
                (a, b) => discriminant(a) == discriminant(b),
            }
    }
}
impl Eq for TypeDef {}

fn discriminant(k: &TypeKind) -> u8 {
    match k {
        TypeKind::I32 => 0,
        TypeKind::Bool => 1,
        TypeKind::Function(_) => 2,
        TypeKind::Str => 3,
        TypeKind::StrOwned => 4,
        TypeKind::Char => 5,
        TypeKind::Record(_) => 6,
        TypeKind::List { .. } => 7,
        TypeKind::Variant { .. } => 8,
    }
}

impl PartialEq for LocalDef {
    fn eq(&self, other: &Self) -> bool {
        self.id_text == other.id_text
            && self.ty == other.ty
            && self.name == other.name
            && self.role == other.role
            && self.index == other.index
            && self.mutable == other.mutable
            && self.param_mode == other.param_mode
    }
}
impl Eq for LocalDef {}

impl PartialEq for Def {
    fn eq(&self, other: &Self) -> bool {
        self.id_text == other.id_text
            && match (&self.kind, &other.kind) {
                (
                    DefKind::Function {
                        name: n1,
                        ty: t1,
                        body: b1,
                    },
                    DefKind::Function {
                        name: n2,
                        ty: t2,
                        body: b2,
                    },
                ) => n1 == n2 && t1 == t2 && b1 == b2,
                (
                    DefKind::Method {
                        name: n1,
                        ty: t1,
                        body: b1,
                        record: r1,
                        receiver: k1,
                    },
                    DefKind::Method {
                        name: n2,
                        ty: t2,
                        body: b2,
                        record: r2,
                        receiver: k2,
                    },
                ) => n1 == n2 && t1 == t2 && b1 == b2 && r1 == r2 && k1 == k2,
                (
                    DefKind::Intrinsic {
                        symbol: s1,
                        effect: e1,
                        ty: t1,
                    },
                    DefKind::Intrinsic {
                        symbol: s2,
                        effect: e2,
                        ty: t2,
                    },
                ) => s1 == s2 && e1 == e2 && t1 == t2,
                _ => false,
            }
    }
}
impl Eq for Def {}

impl PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        self.id_text == other.id_text
            && self.owner == other.owner
            && self.entry == other.entry
            && self.locals == other.locals
            && self.places == other.places
            && self.exprs == other.exprs
            && self.blocks == other.blocks
            && self.loops == other.loops
    }
}
impl Eq for Body {}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.id_text == other.id_text
            && self.path == other.path
            && self.types == other.types
            && self.defs == other.defs
            && self.bodies == other.bodies
    }
}
impl Eq for Module {}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.id_text == other.id_text && self.ty == other.ty && self.kind == other.kind
    }
}
impl Eq for Expr {}

impl PartialEq for Place {
    fn eq(&self, other: &Self) -> bool {
        self.id_text == other.id_text
            && self.kind == other.kind
            && self.local == other.local
            && self.ty == other.ty
            && self.access == other.access
    }
}
impl Eq for Place {}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.id_text == other.id_text && self.stmts == other.stmts
    }
}
impl Eq for Block {}

impl PartialEq for Bundle {
    fn eq(&self, other: &Self) -> bool {
        self.domain == other.domain
            && self.requires == other.requires
            && self.modules == other.modules
    }
}
impl Eq for Bundle {}
