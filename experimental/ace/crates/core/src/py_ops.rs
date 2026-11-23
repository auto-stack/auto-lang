//! PLAN-1107 T-02 — the shared, sourced Python-operation projection.
//!
//! The `use.py` Python surface is resolved **once** in the shared front door
//! (`source/py.rs` import hoisting + the resolver's py-op sugar). PLAN-1026's
//! AV lane encodes each operation into a frozen `p967.frame/1` wire frame and
//! calls the admitted provider op; that frame is an AV/ABI artifact, not a
//! language fact, so a second target cannot legitimately recover the operation
//! from it.
//!
//! This module records the operation **where the resolver performs it**: each
//! py-op fact names the operation kind, its source span, the static member /
//! module label, and the source-order operand slots (role, span, value class).
//! The facts are attached to the verified module (`CheckedModule::py_ops`,
//! set by the py-lane compile entry) and travel to every target view through
//! `SemanticView::py_ops`. Targets consume the facts; they never re-derive
//! Python operations from frame text, function names or variable names.
//!
//! Identity rules:
//!
//! * A fact exists only because the shared resolver admitted that operation
//!   under an admitted `py` foreign declaration (`SOURCE_CAP` /
//!   `RESOURCE_CAP` admission). There is no second admission path.
//! * Spans are source-global (`Span` offsets in the compiled source), so a
//!   consumer maps a fact to exactly one checked expression by span. Facts
//!   for one source are unique per (kind, span) except the deliberate
//!   `py_f64_lo` continuation lane, which is not recorded as its own fact.
//! * Recording is opt-in per compile (the py lane arms it); every other
//!   compile path leaves the recorder idle and the module carries no plan.

use crate::atom_text::Span;

/// The admitted foreign provider identity of the frozen Python op set.
pub const PY_PROVIDER: &str = "py";

/// The 17 frozen `p967.frame/1` provider ops of PLAN-1026 (declaration
/// identity). Consumers use this list to recognise the provider's ops, not to
/// guess from source text.
pub const PY_OP_NAMES: &[&str] = &[
    "py_admission",
    "py_alive",
    "py_binop",
    "py_call",
    "py_error_class",
    "py_error_stage",
    "py_f64_lo",
    "py_getattr",
    "py_getitem",
    "py_import",
    "py_release",
    "py_session_close",
    "py_stage_str",
    "py_str_word",
    "py_to_bool",
    "py_to_f64",
    "py_to_i32",
];

/// One language-level Python operation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PyOpKind {
    /// `use.py <module>: <member>` — the hoisted, entry-order import.
    Import,
    /// A call or bound method call (`f(x, k: v)`, `m.f(x)`); the member label
    /// is the empty string for a direct call of the receiver object.
    Call,
    /// Attribute read (`obj.name`).
    GetAttr,
    /// Index read with a single key or a constructed slice.
    GetItem,
    /// Arithmetic on a Python object; `member` carries the frozen op word
    /// (`add`/`sub`/`mul`/`truediv`).
    BinOp,
    /// `.to(float)` / `.to(double)` extraction.
    ToFloat,
    /// `.to(int)` extraction.
    ToInt,
    /// `.to(bool)` extraction.
    ToBool,
    /// `.to(str)` display extraction.
    ToText,
}

impl PyOpKind {
    /// Classify an admitted `py` declaration name. Only the operation lanes are
    /// operations here: `py_f64_lo`/`py_str_word` are the continuation lanes of
    /// [`PyOpKind::ToFloat`]/[`PyOpKind::ToText`], and
    /// `py_admission`/`py_alive`/`py_release`/`py_session_close`/`py_error_*`
    /// are provider bookkeeping, not source-level operations.
    pub fn from_decl_name(name: &str) -> Option<Self> {
        match name {
            "py_import" => Some(Self::Import),
            "py_call" => Some(Self::Call),
            "py_getattr" => Some(Self::GetAttr),
            "py_getitem" => Some(Self::GetItem),
            "py_binop" => Some(Self::BinOp),
            "py_to_f64" => Some(Self::ToFloat),
            "py_to_i32" => Some(Self::ToInt),
            "py_to_bool" => Some(Self::ToBool),
            "py_stage_str" => Some(Self::ToText),
            _ => None,
        }
    }

    /// The provider declaration name this fact lowers through in the AV lane.
    pub fn decl_name(&self) -> &'static str {
        match self {
            Self::Import => "py_import",
            Self::Call => "py_call",
            Self::GetAttr => "py_getattr",
            Self::GetItem => "py_getitem",
            Self::BinOp => "py_binop",
            Self::ToFloat => "py_to_f64",
            Self::ToInt => "py_to_i32",
            Self::ToBool => "py_to_bool",
            Self::ToText => "py_stage_str",
        }
    }

    /// The stable fact-kind word (serialisation/evidence).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::Call => "call",
            Self::GetAttr => "getattr",
            Self::GetItem => "getitem",
            Self::BinOp => "binop",
            Self::ToFloat => "to-float",
            Self::ToInt => "to-int",
            Self::ToBool => "to-bool",
            Self::ToText => "to-text",
        }
    }
}

/// The admitted value class of one operand (what the frame encoder admits).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PyValClass {
    /// A Python object (opaque handle in the AV lane; a native object in a
    /// source-generating target).
    Object,
    /// An exact `i32` literal or dynamic `i32` value.
    Int,
    /// An exact `bool` literal.
    Bool,
    /// A text literal.
    Text,
    /// An exact float literal (compile-time `F:` encoding for AV).
    Float,
    /// Python None for an omitted slice bound.
    None,
    /// A value class outside the admitted frame classes (never produced by an
    /// admitted frame; kept so a recorder never has to guess a class).
    Other,
}

impl PyValClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Object => "object",
            Self::Int => "int",
            Self::Bool => "bool",
            Self::Text => "text",
            Self::Float => "float",
            Self::None => "none",
            Self::Other => "other",
        }
    }
}

/// The source-order role of an operand.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PyRole {
    /// The receiver (callee object, attribute base, index base, left operand,
    /// extraction operand).
    Receiver,
    /// The `n`-th positional argument after the receiver (for [`PyOpKind::Call`]
    /// the member label is NOT an operand — it is the fact's `member`).
    Positional(u32),
    /// One of the start, stop and step slots of a slice key, numbered 0 to 2.
    SliceSlot(u32),
    /// A named argument (`name: value`).
    Kwarg(String),
}

impl PyRole {
    pub fn as_str(&self) -> String {
        match self {
            Self::Receiver => "receiver".to_string(),
            Self::Positional(i) => format!("pos:{i}"),
            Self::SliceSlot(i) => format!("slice:{i}"),
            Self::Kwarg(name) => format!("kwarg:{name}"),
        }
    }
}

/// The exact literal payload of an operand that the AV frame encoder bakes
/// into the frame TEXT (compile-time `i`/`b`/`t`/`F:` encodings). A
/// source-generating target needs the VALUE, not its frame spelling, so the
/// fact carries it: `None` means the operand is a runtime expression (found
/// by its span).
#[derive(Clone, PartialEq, Debug)]
pub enum PyLiteral {
    Int(i32),
    Bool(bool),
    Text(String),
    Float(f64),
    None,
}

/// One operand slot of an operation.
#[derive(Clone, PartialEq, Debug)]
pub struct PyOperand {
    pub role: PyRole,
    /// Source span of the operand expression (the consumer maps it to the
    /// checked expression of that span).
    pub span: (u32, u32),
    pub class: PyValClass,
    /// The compile-time literal payload, when the operand IS a literal.
    pub literal: Option<PyLiteral>,
}

/// One recorded Python operation.
#[derive(Clone, PartialEq, Debug)]
pub struct PyOpFact {
    pub op: PyOpKind,
    /// Source span of the whole operation expression.
    pub span: (u32, u32),
    /// Static member label (call member, attribute/index/binary word,
    /// import member, extraction target word). Empty when not applicable.
    pub member: String,
    /// Static module path (imports only).
    pub module: String,
    /// Operands in source order.
    pub operands: Vec<PyOperand>,
}

impl PyOpFact {
    pub fn new(op: PyOpKind, span: Span) -> Self {
        Self {
            op,
            span: (span.start as u32, span.end as u32),
            member: String::new(),
            module: String::new(),
            operands: Vec::new(),
        }
    }

    pub fn with_member(mut self, member: impl Into<String>) -> Self {
        self.member = member.into();
        self
    }

    pub fn with_module(mut self, module: impl Into<String>) -> Self {
        self.module = module.into();
        self
    }

    pub fn operand(mut self, role: PyRole, span: Span, class: PyValClass) -> Self {
        self.operands.push(PyOperand {
            role,
            span: (span.start as u32, span.end as u32),
            class,
            literal: None,
        });
        self
    }

    /// The same, carrying the compile-time literal payload.
    pub fn operand_lit(
        mut self,
        role: PyRole,
        span: Span,
        class: PyValClass,
        literal: Option<PyLiteral>,
    ) -> Self {
        self.operands.push(PyOperand {
            role,
            span: (span.start as u32, span.end as u32),
            class,
            literal,
        });
        self
    }

    /// The receiver slot, when the operation has one.
    pub fn receiver(&self) -> Option<&PyOperand> {
        self.operands
            .iter()
            .find(|o| matches!(o.role, PyRole::Receiver))
    }

    /// Positional operands in slot order.
    pub fn positionals(&self) -> Vec<&PyOperand> {
        self.operands
            .iter()
            .filter(|o| matches!(o.role, PyRole::Positional(_)))
            .collect()
    }

    /// Named operands (`(name, operand)` in source order).
    pub fn kwargs(&self) -> Vec<(&str, &PyOperand)> {
        self.operands
            .iter()
            .filter_map(|o| match &o.role {
                PyRole::Kwarg(name) => Some((name.as_str(), o)),
                _ => None,
            })
            .collect()
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct PyOpPlan {
    facts: Vec<PyOpFact>,
}

impl PyOpPlan {
    pub fn from_facts(facts: Vec<PyOpFact>) -> Self {
        Self { facts }
    }

    pub fn facts(&self) -> &[PyOpFact] {
        &self.facts
    }

    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    pub fn len(&self) -> usize {
        self.facts.len()
    }

    /// The fact for `kind` whose operation span starts at `start` (the
    /// consumer's span key; see the module identity rules).
    pub fn find(&self, kind: PyOpKind, start: u32) -> Option<&PyOpFact> {
        self.facts
            .iter()
            .find(|f| f.op == kind && f.span.0 == start)
    }

    /// The fact for `kind` whose operation span is EXACTLY `span`. Chained
    /// operations share a start offset (`a.b(c) + d`), so a consumer that has
    /// the whole checked expression must prefer the exact-span fact.
    pub fn find_exact(&self, kind: PyOpKind, span: (u32, u32)) -> Option<&PyOpFact> {
        self.facts.iter().find(|f| f.op == kind && f.span == span)
    }

    /// All facts of one kind, in source order.
    pub fn of_kind(&self, kind: PyOpKind) -> impl Iterator<Item = &PyOpFact> {
        self.facts.iter().filter(move |f| f.op == kind)
    }
}

thread_local! {
    /// The active recorder of the current compile (armed only by the py lane).
    static RECORDER: std::cell::RefCell<Option<Vec<PyOpFact>>> =
        const { std::cell::RefCell::new(None) };
}

/// Arm recording for the current thread's compile (the py-lane entry calls
/// this before parsing; any previous leftovers are discarded).
pub fn begin_recording() {
    RECORDER.with(|r| *r.borrow_mut() = Some(Vec::new()));
}

/// Whether recording is armed on this thread.
pub fn recording_active() -> bool {
    RECORDER.with(|r| r.borrow().is_some())
}

/// Record one operation. A no-op when recording is not armed (all non-py
/// compile paths), so call sites need no mode branch.
pub fn record(fact: PyOpFact) {
    RECORDER.with(|r| {
        if let Some(facts) = r.borrow_mut().as_mut() {
            facts.push(fact);
        }
    });
}

/// Disarm recording and take the plan collected by this compile.
pub fn take_plan() -> PyOpPlan {
    RECORDER.with(|r| PyOpPlan::from_facts(r.borrow_mut().take().unwrap_or_default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fact(op: PyOpKind, member: &str) -> PyOpFact {
        PyOpFact::new(op, Span { start: 10, end: 20 }).with_member(member)
    }

    #[test]
    fn decl_names_are_unique_and_classified() {
        let mut seen = std::collections::BTreeSet::new();
        for name in PY_OP_NAMES {
            assert!(seen.insert(*name), "duplicate py op name `{name}`");
        }
        assert_eq!(seen.len(), 17, "the frozen py op set is 17 provider ops");
        for kind in [
            PyOpKind::Import,
            PyOpKind::Call,
            PyOpKind::GetAttr,
            PyOpKind::GetItem,
            PyOpKind::BinOp,
            PyOpKind::ToFloat,
            PyOpKind::ToInt,
            PyOpKind::ToBool,
            PyOpKind::ToText,
        ] {
            assert!(seen.contains(kind.decl_name()), "{}", kind.decl_name());
            assert_eq!(
                PyOpKind::from_decl_name(kind.decl_name()),
                Some(kind),
                "{} round-trips",
                kind.decl_name()
            );
        }
        // bookkeeping / continuation lanes are provider ops, not source ops
        for not_op in [
            "py_admission",
            "py_alive",
            "py_f64_lo",
            "py_release",
            "py_session_close",
            "py_error_class",
            "py_error_stage",
            "py_str_word",
        ] {
            assert_eq!(PyOpKind::from_decl_name(not_op), None, "{not_op}");
            assert!(seen.contains(not_op), "{not_op} is still a provider op");
        }
    }

    #[test]
    fn plan_lookups_use_kind_and_span_key() {
        let plan = PyOpPlan::from_facts(vec![
            fact(PyOpKind::Import, "sqrt"),
            fact(PyOpKind::Call, "reshape"),
        ]);
        assert_eq!(plan.len(), 2);
        assert!(plan.find(PyOpKind::Call, 10).is_some());
        assert!(plan.find(PyOpKind::Call, 11).is_none(), "span key is exact");
        assert!(
            plan.find(PyOpKind::ToFloat, 10).is_none(),
            "kind key is exact"
        );
        assert_eq!(plan.of_kind(PyOpKind::Import).count(), 1);
    }

    #[test]
    fn operands_keep_roles_and_source_order() {
        let span = Span { start: 1, end: 4 };
        let f = PyOpFact::new(PyOpKind::Call, span)
            .with_member("parameters")
            .operand(PyRole::Receiver, span, PyValClass::Object)
            .operand(PyRole::Positional(0), span, PyValClass::Int)
            .operand(PyRole::Kwarg("lr".to_string()), span, PyValClass::Float);
        assert_eq!(f.receiver().map(|o| o.class), Some(PyValClass::Object));
        assert_eq!(f.positionals().len(), 1);
        assert_eq!(f.kwargs()[0].0, "lr");
        assert_eq!(f.kwargs()[0].1.class, PyValClass::Float);
        assert_eq!(f.operands[0].role.as_str(), "receiver");
        assert_eq!(f.operands[2].role.as_str(), "kwarg:lr");
    }

    #[test]
    fn recorder_is_idle_outside_the_py_lane() {
        assert!(!recording_active());
        record(fact(PyOpKind::Call, "ignored"));
        assert!(take_plan().is_empty(), "an unarmed recorder drops facts");
    }
}
