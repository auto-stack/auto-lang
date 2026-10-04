//! Typed HIR data model for the core-i32 profile (PLAN-741).
//!
//! `Unchecked*` structures are produced by the descriptor binder
//! (`crate::descriptor`): every textual reference has been resolved to a
//! typed, category-correct ID scoped to its owning table. They carry source
//! spans for downstream diagnostics. The semantic verifier (`crate::verify`)
//! consumes them and produces `CheckedModule` (added in T-03).
//!
//! Equality on these structures is *semantic*: spans and the
//! `mutable_explicit` presence bit are excluded, so author-form vs
//! explicit-form vs canonical-roundtrip binds compare equal.

use crate::atom_text::Span;
use std::fmt;

macro_rules! typed_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

impl_id_num!(TypeId, DefId, BodyId, LocalId, ExprId, PlaceId, BlockId, LoopId);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuiltinType {
    I32,
    Bool,
}

impl BuiltinType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BuiltinType::I32 => "i32",
            BuiltinType::Bool => "bool",
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

#[derive(Clone, Debug)]
pub enum TypeKind {
    I32,
    Bool,
    Function(FnSig),
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

#[derive(Clone, Debug)]
pub enum DefKind {
    Function {
        name: String,
        ty: TypeId,
        body: BodyId,
    },
    Intrinsic {
        symbol: String,
        effect: Effect,
        ty: TypeId,
    },
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
            DefKind::Intrinsic { symbol, .. } => symbol,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    AddI32,
    MulI32,
    LtI32,
}

impl BinOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinOp::AddI32 => "add_i32",
            BinOp::MulI32 => "mul_i32",
            BinOp::LtI32 => "lt_i32",
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
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExprKind {
    ReadLocal {
        local: LocalId,
    },
    Constant {
        value: i32,
    },
    Binary {
        op: BinOp,
        overflow: OverflowPolicy,
        lhs: ExprId,
        rhs: ExprId,
    },
    Call {
        callee: DefId,
        eval_args: Vec<ExprId>,
        bindings: Vec<CallBinding>,
    },
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub id_text: String,
    pub ty: TypeUse,
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlaceKind {
    Local,
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

/// Root of one parsed document: document-level `requires` capabilities plus
/// one or more modules.
#[derive(Clone, Debug)]
pub struct Bundle {
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
        self.requires == other.requires && self.modules == other.modules
    }
}
impl Eq for Bundle {}
