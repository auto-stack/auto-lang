//! Plan 121: Task/Msg AST structures
//! Plan 125: Phase 3 - Polymorphic routing with implicit unions
//!
//! Task definition and message handling structures for the Actor model.

use crate::ast::{Body, Expr, Fn, Name, Type};
use crate::ast::{AtomWriter, ToAtomStr};
use crate::token::Pos;
use auto_val::{AutoStr, Node as AutoNode, Value};
use std::{fmt, io as stdio};

/// Task annotation attributes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskAttr {
    /// `#[single]` - singleton task (only one instance)
    Single,
}

impl fmt::Display for TaskAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskAttr::Single => write!(f, "single"),
        }
    }
}

/// Task definition
///
/// ```auto
/// #[single]
/// task CounterTask {
///     count mut = 0
///
///     fn start() ! { self.count = 0 }
///     fn stop() ! { print("stopping") }
///
///     on {
///         Add(val) => { self.count += val }
///         Reset => { self.count = 0 }
///         else => { }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TaskDef {
    /// Task name (e.g., "CounterTask")
    pub name: Name,
    /// Annotations (e.g., #[single])
    pub attrs: Vec<TaskAttr>,
    /// Private state fields (name -> (mutable, initial_value))
    pub state: Vec<(Name, bool, Expr)>,
    /// Lifecycle hook: start()
    pub start_hook: Option<Fn>,
    /// Lifecycle hook: stop()
    pub stop_hook: Option<Fn>,
    /// Message handler block
    pub on_block: TaskOnBlock,
    /// Source position
    pub pos: Pos,
}

impl PartialEq for TaskDef {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.attrs == other.attrs
            && self.state.len() == other.state.len()
            && self.start_hook == other.start_hook
            && self.stop_hook == other.stop_hook
            && self.on_block == other.on_block
        // Note: pos not compared for equality
    }
}

impl TaskDef {
    /// Create a new TaskDef
    pub fn new(name: Name, attrs: Vec<TaskAttr>, pos: Pos) -> Self {
        Self {
            name,
            attrs,
            state: Vec::new(),
            start_hook: None,
            stop_hook: None,
            on_block: TaskOnBlock::new(pos),
            pos,
        }
    }

    /// Check if this is a singleton task
    pub fn is_single(&self) -> bool {
        self.attrs.contains(&TaskAttr::Single)
    }

    /// Add a state field
    pub fn add_state(&mut self, name: Name, mutable: bool, initial: Expr) {
        self.state.push((name, mutable, initial));
    }

    /// Set the start hook
    pub fn set_start_hook(&mut self, hook: Fn) {
        self.start_hook = Some(hook);
    }

    /// Set the stop hook
    pub fn set_stop_hook(&mut self, hook: Fn) {
        self.stop_hook = Some(hook);
    }
}

/// Task on block - message handlers
///
/// Similar to OnEvents but specific to task message handling.
/// Includes pattern matching on message variants.
///
/// ## Phase 3 Extensions (Plan 125)
///
/// Supports optional context parameter for reply capability:
///
/// ```auto
/// on(ctx) {                    // context_param: Some("ctx")
///     "ping" => { ctx.reply("pong") }
/// }
/// ```
///
/// Also supports guard expressions:
///
/// ```auto
/// on {
///     amount int if amount > 10000 => { approve() }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TaskOnBlock {
    /// Context parameter name (e.g., "ctx", "req", "origin")
    /// None means no context parameter (Phase 1/2 style)
    pub context_param: Option<Name>,

    /// Message handlers: (pattern, guard_expression, body)
    /// Guard expression is optional (None means always match)
    pub handlers: Vec<(TaskMsgPattern, Option<Expr>, Body)>,

    /// Fallback handler (else => { ... })
    pub else_handler: Option<Body>,

    /// Source position
    pub pos: Pos,
}

impl PartialEq for TaskOnBlock {
    fn eq(&self, other: &Self) -> bool {
        // Compare context_param
        if self.context_param != other.context_param {
            return false;
        }
        // Compare handlers by pattern only (Body doesn't implement PartialEq)
        if self.handlers.len() != other.handlers.len() {
            return false;
        }
        for ((p1, g1, _), (p2, g2, _)) in self.handlers.iter().zip(other.handlers.iter()) {
            if p1 != p2 {
                return false;
            }
            // Compare guard presence only (Expr comparison is complex)
            if g1.is_some() != g2.is_some() {
                return false;
            }
        }
        // Compare else_handler presence only
        self.else_handler.is_some() == other.else_handler.is_some()
        // Note: pos not compared for equality
    }
}

impl TaskOnBlock {
    /// Create a new TaskOnBlock (Phase 1/2 style, no context parameter)
    pub fn new(pos: Pos) -> Self {
        Self {
            context_param: None,
            handlers: Vec::new(),
            else_handler: None,
            pos,
        }
    }

    /// Create a TaskOnBlock with context parameter (Phase 3 style)
    pub fn with_context(context_param: Name, pos: Pos) -> Self {
        Self {
            context_param: Some(context_param),
            handlers: Vec::new(),
            else_handler: None,
            pos,
        }
    }

    /// Create a TaskOnBlock with handlers and optional else handler
    pub fn with_handlers(
        handlers: Vec<(TaskMsgPattern, Option<Expr>, Body)>,
        else_handler: Option<Body>,
        pos: Pos,
    ) -> Self {
        Self {
            context_param: None,
            handlers,
            else_handler,
            pos,
        }
    }

    /// Create a TaskOnBlock with context parameter and handlers (Phase 3 full)
    pub fn with_context_and_handlers(
        context_param: Option<Name>,
        handlers: Vec<(TaskMsgPattern, Option<Expr>, Body)>,
        else_handler: Option<Body>,
        pos: Pos,
    ) -> Self {
        Self {
            context_param,
            handlers,
            else_handler,
            pos,
        }
    }

    /// Add a message handler (Phase 1/2 style, no guard)
    pub fn add_handler(&mut self, pattern: TaskMsgPattern, body: Body) {
        self.handlers.push((pattern, None, body));
    }

    /// Add a message handler with guard expression (Phase 3 style)
    pub fn add_handler_with_guard(
        &mut self,
        pattern: TaskMsgPattern,
        guard: Option<Expr>,
        body: Body,
    ) {
        self.handlers.push((pattern, guard, body));
    }

    /// Set the context parameter name
    pub fn set_context_param(&mut self, name: Name) {
        self.context_param = Some(name);
    }

    /// Check if this on block has a context parameter
    pub fn has_context(&self) -> bool {
        self.context_param.is_some()
    }

    /// Set the else handler
    pub fn set_else(&mut self, body: Body) {
        self.else_handler = Some(body);
    }
}

/// Message pattern for task on block
///
/// ## Phase 1/2 Patterns
///
/// - `Simple` - Simple variant without data: `Reset`, `Print`
/// - `WithBindings` - Variant with bindings: `Add(val)`, `Log(msg)`
///
/// ## Phase 3 Patterns (Plan 125)
///
/// - `Literal` - Literal exact match: `"start"`, `404`, `true`
/// - `TypeBinding` - Type capture binding: `msg string`, `u User`
#[derive(Debug, Clone)]
pub enum TaskMsgPattern {
    // === Phase 1/2 Patterns ===
    /// Simple variant without data: Reset, Print
    Simple(Name),
    /// Variant with bindings: Add(val), Log(msg)
    WithBindings {
        variant: Name,
        bindings: Vec<Name>,
    },

    // === Phase 3 Patterns (Plan 125) ===
    /// Literal exact match: "start", 404, true
    Literal(LiteralValue),
    /// Type capture binding: msg string, u User
    TypeBinding {
        /// Binding variable name
        name: Name,
        /// Type expression (Box because Type doesn't implement Eq)
        type_expr: Box<Type>,
    },
}

impl PartialEq for TaskMsgPattern {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TaskMsgPattern::Simple(a), TaskMsgPattern::Simple(b)) => a == b,
            (
                TaskMsgPattern::WithBindings { variant: v1, bindings: b1 },
                TaskMsgPattern::WithBindings { variant: v2, bindings: b2 },
            ) => v1 == v2 && b1 == b2,
            (TaskMsgPattern::Literal(a), TaskMsgPattern::Literal(b)) => a == b,
            (
                TaskMsgPattern::TypeBinding { name: n1, type_expr: t1 },
                TaskMsgPattern::TypeBinding { name: n2, type_expr: t2 },
            ) => {
                // Compare names only for TypeBinding (Type doesn't implement PartialEq)
                n1 == n2 && std::mem::discriminant(t1.as_ref()) == std::mem::discriminant(t2.as_ref())
            }
            _ => false,
        }
    }
}

impl Eq for TaskMsgPattern {}

/// Literal value for Phase 3 pattern matching
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralValue {
    /// String literal: "start", "ping"
    String(AutoStr),
    /// Integer literal: 404, 200
    Int(i64),
    /// Unsigned integer literal
    Uint(u64),
    /// Float literal (stored as integer and fractional parts)
    Float(i64, i64),
    /// Boolean literal: true, false
    Bool(bool),
    /// Character literal: 'a', 'Z'
    Char(char),
}

impl fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralValue::String(s) => write!(f, "\"{}\"", s),
            LiteralValue::Int(n) => write!(f, "{}", n),
            LiteralValue::Uint(n) => write!(f, "{}u", n),
            LiteralValue::Float(i, frac) => write!(f, "{}.{}", i, frac),
            LiteralValue::Bool(b) => write!(f, "{}", b),
            LiteralValue::Char(c) => write!(f, "'{}'", c),
        }
    }
}

impl fmt::Display for TaskMsgPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskMsgPattern::Simple(name) => write!(f, "{}", name),
            TaskMsgPattern::WithBindings { variant, bindings } => {
                write!(f, "{}(", variant)?;
                for (i, binding) in bindings.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", binding)?;
                }
                write!(f, ")")
            }
            TaskMsgPattern::Literal(v) => {
                match v {
                    LiteralValue::String(s) => write!(f, "\"{}\"", s),
                    LiteralValue::Int(n) => write!(f, "{}", n),
                    LiteralValue::Uint(n) => write!(f, "{}u", n),
                    LiteralValue::Float(i, frac) => write!(f, "{}.{}", i, frac),
                    LiteralValue::Bool(b) => write!(f, "{}", b),
                    LiteralValue::Char(c) => write!(f, "'{}'", c),
                }
            }
            TaskMsgPattern::TypeBinding { name, type_expr } => {
                write!(f, "{} ", name)?;
                write!(f, "{}", type_expr.unique_name())
            }
        }
    }
}

impl TaskMsgPattern {
    /// Create a simple pattern (no bindings)
    pub fn simple(name: Name) -> Self {
        TaskMsgPattern::Simple(name)
    }

    /// Create a pattern with bindings
    pub fn with_bindings(variant: Name, bindings: Vec<Name>) -> Self {
        TaskMsgPattern::WithBindings { variant, bindings }
    }

    /// Get the variant name (Phase 1/2 only, returns None for Literal/TypeBinding)
    pub fn variant_name(&self) -> Option<&Name> {
        match self {
            TaskMsgPattern::Simple(name) => Some(name),
            TaskMsgPattern::WithBindings { variant, .. } => Some(variant),
            TaskMsgPattern::Literal(_) => None,
            TaskMsgPattern::TypeBinding { .. } => None,
        }
    }

    /// Check if this pattern has bindings
    pub fn has_bindings(&self) -> bool {
        matches!(self, TaskMsgPattern::WithBindings { .. })
    }

    /// Check if this is a Phase 3 literal pattern
    pub fn is_literal(&self) -> bool {
        matches!(self, TaskMsgPattern::Literal(_))
    }

    /// Check if this is a Phase 3 type binding pattern
    pub fn is_type_binding(&self) -> bool {
        matches!(self, TaskMsgPattern::TypeBinding { .. })
    }

    /// Get the type expression (only for TypeBinding)
    pub fn type_expr(&self) -> Option<&Type> {
        match self {
            TaskMsgPattern::TypeBinding { type_expr, .. } => Some(type_expr.as_ref()),
            _ => None,
        }
    }

    /// Get the binding name (only for TypeBinding)
    pub fn binding_name(&self) -> Option<&Name> {
        match self {
            TaskMsgPattern::TypeBinding { name, .. } => Some(name),
            _ => None,
        }
    }
}

// ============================================================================
// Display implementations
// ============================================================================

impl fmt::Display for TaskDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(task {}", self.name)?;

        if !self.attrs.is_empty() {
            write!(f, " (attrs")?;
            for attr in &self.attrs {
                write!(f, " {}", attr)?;
            }
            write!(f, ")")?;
        }

        if !self.state.is_empty() {
            write!(f, " (state")?;
            for (name, mutable, value) in &self.state {
                if *mutable {
                    write!(f, " ({} mut {})", name, value)?;
                } else {
                    write!(f, " ({} {})", name, value)?;
                }
            }
            write!(f, ")")?;
        }

        if let Some(start) = &self.start_hook {
            write!(f, " (start {})", start)?;
        }

        if let Some(stop) = &self.stop_hook {
            write!(f, " (stop {})", stop)?;
        }

        write!(f, " {})", self.on_block)
    }
}

impl fmt::Display for TaskOnBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(on")?;
        // Phase 3: context parameter
        if let Some(ctx) = &self.context_param {
            write!(f, "({})", ctx)?;
        }
        // Phase 3: handlers with optional guard
        for (pattern, guard, body) in &self.handlers {
            write!(f, " ({}", pattern)?;
            if let Some(g) = guard {
                write!(f, " if {}", g)?;
            }
            write!(f, " -> {})", body)?;
        }
        if let Some(else_body) = &self.else_handler {
            write!(f, " (else -> {})", else_body)?;
        }
        write!(f, ")")
    }
}

// ============================================================================
// ToAtom and ToNode implementations
// ============================================================================

use crate::ast::{ToAtom, ToNode};

impl ToAtom for TaskAttr {
    fn to_atom(&self) -> AutoStr {
        self.to_string().into()
    }
}

impl AtomWriter for TaskAttr {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "{}", self)?;
        Ok(())
    }
}

impl ToNode for TaskAttr {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("attr");
        node.set_prop("kind", Value::str(self.to_string().as_str()));
        node
    }
}

impl AtomWriter for TaskDef {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        // Write annotations
        for attr in &self.attrs {
            write!(f, "#[{}] ", attr)?;
        }

        write!(f, "task {} {{", self.name)?;

        // Write state fields
        for (name, mutable, value) in &self.state {
            write!(f, "\n    {}", name)?;
            if *mutable {
                write!(f, " mut")?;
            }
            write!(f, " = {}", value.to_atom_str())?;
        }

        // Write lifecycle hooks
        if let Some(start) = &self.start_hook {
            write!(f, "\n\n    fn start() ! {}", start.body.to_atom_str())?;
        }

        if let Some(stop) = &self.stop_hook {
            write!(f, "\n    fn stop() ! {}", stop.body.to_atom_str())?;
        }

        // Write on block
        write!(f, "\n\n    on {{")?;
        // Phase 3: context parameter
        if let Some(ctx) = &self.on_block.context_param {
            write!(f, "({})", ctx)?;
        }
        // Phase 3: handlers with optional guard
        for (pattern, guard, body) in &self.on_block.handlers {
            write!(f, "\n        {}", pattern)?;
            if let Some(g) = guard {
                write!(f, " if {}", g.to_atom_str())?;
            }
            write!(f, " -> {}", body.to_atom_str())?;
        }
        if let Some(else_body) = &self.on_block.else_handler {
            write!(f, "\n        else -> {}", else_body.to_atom_str())?;
        }
        write!(f, "\n    }}")?;

        write!(f, "\n}}")?;
        Ok(())
    }
}

impl ToNode for TaskDef {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("task");
        node.set_prop("name", Value::str(self.name.as_str()));

        // Add attrs
        for attr in &self.attrs {
            node.add_kid(attr.to_node());
        }

        // Add state fields as children
        for (name, mutable, value) in &self.state {
            let mut state_node = AutoNode::new("state");
            state_node.set_prop("name", Value::str(name.as_str()));
            state_node.set_prop("mutable", Value::Bool(*mutable));
            state_node.set_prop("initial", Value::str(&*value.to_atom()));
            node.add_kid(state_node);
        }

        // Add lifecycle hooks
        if let Some(start) = &self.start_hook {
            node.add_kid(start.to_node());
        }

        if let Some(stop) = &self.stop_hook {
            node.add_kid(stop.to_node());
        }

        // Add on block
        node.add_kid(self.on_block.to_node());

        node
    }
}

impl ToAtom for TaskDef {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl AtomWriter for TaskOnBlock {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        // Phase 3: context parameter
        if let Some(ctx) = &self.context_param {
            write!(f, "({}) ", ctx)?;
        }

        write!(f, "{{")?;

        // Phase 3: handlers with optional guard
        for (pattern, guard, body) in &self.handlers {
            write!(f, "{} -> ", pattern)?;
            if let Some(g) = guard {
                write!(f, " if {} ", g.to_atom_str())?;
            }
            write!(f, " {}", body.to_atom_str())?;
        }

        if let Some(else_body) = &self.else_handler {
            write!(f, "else -> {}", else_body.to_atom_str())?;
        }

        write!(f, "}}")?;
        Ok(())
    }
}

impl ToNode for TaskOnBlock {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("on");

        // Phase 3: context parameter
        if let Some(ctx) = &self.context_param {
            node.set_prop("context_param", Value::str(ctx.as_str()));
        }

        // Phase 3: handlers with optional guard
        for (pattern, guard, body) in &self.handlers {
            let mut handler_node = AutoNode::new("handler");
            handler_node.set_prop("pattern", Value::str(pattern.to_string().as_str()));
            if let Some(g) = guard {
                handler_node.set_prop("guard", Value::str(g.to_atom().as_str()));
            }
            handler_node.add_kid(body.to_node());
            node.add_kid(handler_node);
        }

        if let Some(else_body) = &self.else_handler {
            let mut else_node = AutoNode::new("else");
            else_node.add_kid(else_body.to_node());
            node.add_kid(else_node);
        }

        node
    }
}

impl ToAtom for TaskOnBlock {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl AtomWriter for TaskMsgPattern {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "{}", self)?;
        Ok(())
    }
}

impl ToNode for TaskMsgPattern {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("pattern");

        match self {
            TaskMsgPattern::Simple(name) => {
                node.set_prop("kind", Value::str("simple"));
                node.set_prop("variant", Value::str(name.as_str()));
            }
            TaskMsgPattern::WithBindings { variant, bindings } => {
                node.set_prop("kind", Value::str("with_bindings"));
                node.set_prop("variant", Value::str(variant.as_str()));
                for binding in bindings {
                    let mut binding_node = AutoNode::new("binding");
                    binding_node.set_prop("name", Value::str(binding.as_str()));
                    node.add_kid(binding_node);
                }
            }
            TaskMsgPattern::Literal(v) => {
                node.set_prop("kind", Value::str("literal"));
                // Store the literal value based on type
                match v {
                    LiteralValue::String(s) => {
                        node.set_prop("literal_type", Value::str("string"));
                        node.set_prop("value", Value::str(s.as_str()));
                    }
                    LiteralValue::Int(n) => {
                        node.set_prop("literal_type", Value::str("int"));
                        node.set_prop("value", Value::Int(*n as i32));
                    }
                    LiteralValue::Uint(n) => {
                        node.set_prop("literal_type", Value::str("uint"));
                        node.set_prop("value", Value::Uint(*n as u32));
                    }
                    LiteralValue::Float(i, f) => {
                        node.set_prop("literal_type", Value::str("float"));
                        node.set_prop("value", Value::Float((*i as f64 + *f as f64) / 1000000000.0));
                    }
                    LiteralValue::Bool(b) => {
                        node.set_prop("literal_type", Value::str("bool"));
                        node.set_prop("value", Value::Bool(*b));
                    }
                    LiteralValue::Char(c) => {
                        node.set_prop("literal_type", Value::str("char"));
                        node.set_prop("value", Value::Char(*c));
                    }
                }
            }
            TaskMsgPattern::TypeBinding { name, type_expr } => {
                node.set_prop("kind", Value::str("type_binding"));
                node.set_prop("binding_name", Value::str(name.as_str()));
                node.set_prop("type", Value::str(type_expr.unique_name().as_str()));
            }
        }

        node
    }
}

impl ToAtom for TaskMsgPattern {
    fn to_atom(&self) -> AutoStr {
        self.to_string().into()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_attr_single() {
        let attr = TaskAttr::Single;
        assert_eq!(attr.to_string(), "single");
        assert_eq!(attr.to_atom(), "single");
    }

    #[test]
    fn test_task_msg_pattern_simple() {
        let pattern = TaskMsgPattern::simple("Reset".into());
        assert_eq!(pattern.to_string(), "Reset");
        assert_eq!(pattern.variant_name(), Some(&AutoStr::from("Reset")));
        assert!(!pattern.has_bindings());
    }

    #[test]
    fn test_task_msg_pattern_with_bindings() {
        let pattern = TaskMsgPattern::with_bindings(
            "Add".into(),
            vec!["val".into(), "other".into()],
        );
        assert_eq!(pattern.to_string(), "Add(val, other)");
        assert_eq!(pattern.variant_name(), Some(&AutoStr::from("Add")));
        assert!(pattern.has_bindings());
    }

    #[test]
    fn test_task_on_block_new() {
        let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
        let on_block = TaskOnBlock::new(pos);
        assert!(on_block.handlers.is_empty());
        assert!(on_block.else_handler.is_none());
    }

    #[test]
    fn test_task_def_new() {
        let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
        let task = TaskDef::new("CounterTask".into(), vec![], pos);
        assert_eq!(task.name, "CounterTask");
        assert!(!task.is_single());
        assert!(task.state.is_empty());
        assert!(task.start_hook.is_none());
        assert!(task.stop_hook.is_none());
    }

    #[test]
    fn test_task_def_single() {
        let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
        let task = TaskDef::new("SingletonTask".into(), vec![TaskAttr::Single], pos);
        assert!(task.is_single());
    }

    #[test]
    fn test_task_def_add_state() {
        let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
        let mut task = TaskDef::new("CounterTask".into(), vec![], pos);
        task.add_state("count".into(), true, Expr::Int(0));
        assert_eq!(task.state.len(), 1);
        assert_eq!(task.state[0].0, "count");
        assert!(task.state[0].1); // mutable
    }

    #[test]
    fn test_task_msg_pattern_equality() {
        let p1 = TaskMsgPattern::simple("Reset".into());
        let p2 = TaskMsgPattern::simple("Reset".into());
        let p3 = TaskMsgPattern::simple("Add".into());
        let p4 = TaskMsgPattern::with_bindings("Add".into(), vec!["val".into()]);

        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
        assert_ne!(p3, p4); // Simple vs WithBindings
    }

    #[test]
    fn test_task_on_block_add_handler() {
        let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
        let mut on_block = TaskOnBlock::new(pos);
        let pattern = TaskMsgPattern::simple("Reset".into());
        let body = Body::new();

        on_block.add_handler(pattern, body);
        assert_eq!(on_block.handlers.len(), 1);
    }

    #[test]
    fn test_task_on_block_set_else() {
        let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
        let mut on_block = TaskOnBlock::new(pos);
        let body = Body::new();

        on_block.set_else(body);
        assert!(on_block.else_handler.is_some());
    }

    // ========== Phase 3 Tests (Plan 125) ==========

    #[test]
    fn test_literal_value_string() {
        let lit = LiteralValue::String("ping".into());
        assert_eq!(lit.to_string(), "\"ping\"");
    }

    #[test]
    fn test_literal_value_int() {
        let lit = LiteralValue::Int(404);
        assert_eq!(lit.to_string(), "404");
    }

    #[test]
    fn test_literal_value_uint() {
        let lit = LiteralValue::Uint(200);
        assert_eq!(lit.to_string(), "200u");
    }

    #[test]
    fn test_literal_value_float() {
        let lit = LiteralValue::Float(3, 14);
        assert_eq!(lit.to_string(), "3.14");
    }

    #[test]
    fn test_literal_value_bool() {
        let lit_true = LiteralValue::Bool(true);
        let lit_false = LiteralValue::Bool(false);
        assert_eq!(lit_true.to_string(), "true");
        assert_eq!(lit_false.to_string(), "false");
    }

    #[test]
    fn test_literal_value_char() {
        let lit = LiteralValue::Char('a');
        assert_eq!(lit.to_string(), "'a'");
    }

    #[test]
    fn test_task_msg_pattern_literal() {
        let pattern = TaskMsgPattern::Literal(LiteralValue::String("ping".into()));
        assert_eq!(pattern.to_string(), "\"ping\"");
        assert!(pattern.is_literal());
        assert!(!pattern.has_bindings());
        assert!(pattern.variant_name().is_none());
    }

    #[test]
    fn test_task_msg_pattern_type_binding() {
        let pattern = TaskMsgPattern::TypeBinding {
            name: "msg".into(),
            type_expr: Box::new(Type::Str(0)),
        };
        assert_eq!(pattern.to_string(), "msg str");
        assert!(pattern.is_type_binding());
        assert!(!pattern.has_bindings());
        assert!(pattern.variant_name().is_none());
        assert!(pattern.type_expr().is_some());
        assert!(pattern.binding_name().is_some());
    }

    #[test]
    fn test_task_on_block_with_context() {
        let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
        let on_block = TaskOnBlock::with_context("ctx".into(), pos);
        assert!(on_block.has_context());
        assert_eq!(on_block.context_param, Some("ctx".into()));
    }

    #[test]
    fn test_task_on_block_add_handler_with_guard() {
        use auto_val::Op;

        let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
        let mut on_block = TaskOnBlock::new(pos);
        let pattern = TaskMsgPattern::TypeBinding {
            name: "amount".into(),
            type_expr: Box::new(Type::Int),
        };
        let guard = Some(Expr::Bina(
            Box::new(Expr::Ident("amount".into())),
            Op::Gt,
            Box::new(Expr::Int(10000)),
        ));
        let body = Body::new();

        on_block.add_handler_with_guard(pattern, guard.clone(), body);
        assert_eq!(on_block.handlers.len(), 1);
        let (p, g, _) = &on_block.handlers[0];
        assert!(g.is_some());
    }

    #[test]
    fn test_literal_value_equality() {
        let s1 = LiteralValue::String("a".into());
        let s2 = LiteralValue::String("a".into());
        let s3 = LiteralValue::String("b".into());

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);

        let i1 = LiteralValue::Int(1);
        let i2 = LiteralValue::Int(1);
        assert_eq!(i1, i2);
        assert_ne!(i1, LiteralValue::Int(2));

        let b1 = LiteralValue::Bool(true);
        let b2 = LiteralValue::Bool(true);
        assert_eq!(b1, b2);
        assert_ne!(b1, LiteralValue::Bool(false));
    }

    #[test]
    fn test_task_msg_pattern_literal_equality() {
        let p1 = TaskMsgPattern::Literal(LiteralValue::Int(404));
        let p2 = TaskMsgPattern::Literal(LiteralValue::Int(404));
        let p3 = TaskMsgPattern::Literal(LiteralValue::Int(500));

        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }
}
