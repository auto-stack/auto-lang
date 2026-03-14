//! Plan 121: Task/Msg AST structures
//!
//! Task definition and message handling structures for the Actor model.

use crate::ast::{Body, Expr, Fn, Name};
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
#[derive(Debug, Clone)]
pub struct TaskOnBlock {
    /// Message handlers: (pattern, body)
    /// Pattern is a message variant pattern like "Add(val)" or "Reset"
    pub handlers: Vec<(TaskMsgPattern, Body)>,
    /// Fallback handler (else => { ... })
    pub else_handler: Option<Body>,
    /// Source position
    pub pos: Pos,
}

impl PartialEq for TaskOnBlock {
    fn eq(&self, other: &Self) -> bool {
        // Compare handlers by pattern only (Body doesn't implement PartialEq)
        if self.handlers.len() != other.handlers.len() {
            return false;
        }
        for ((p1, _), (p2, _)) in self.handlers.iter().zip(other.handlers.iter()) {
            if p1 != p2 {
                return false;
            }
        }
        // Compare else_handler presence only
        self.else_handler.is_some() == other.else_handler.is_some()
        // Note: pos not compared for equality
    }
}

impl TaskOnBlock {
    /// Create a new TaskOnBlock
    pub fn new(pos: Pos) -> Self {
        Self {
            handlers: Vec::new(),
            else_handler: None,
            pos,
        }
    }

    /// Create a TaskOnBlock with handlers and optional else handler
    pub fn with_handlers(
        handlers: Vec<(TaskMsgPattern, Body)>,
        else_handler: Option<Body>,
        pos: Pos,
    ) -> Self {
        Self {
            handlers,
            else_handler,
            pos,
        }
    }

    /// Add a message handler
    pub fn add_handler(&mut self, pattern: TaskMsgPattern, body: Body) {
        self.handlers.push((pattern, body));
    }

    /// Set the else handler
    pub fn set_else(&mut self, body: Body) {
        self.else_handler = Some(body);
    }
}

/// Message pattern for task on block
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskMsgPattern {
    /// Simple variant without data: Reset, Print
    Simple(Name),
    /// Variant with bindings: Add(val), Log(msg)
    WithBindings {
        variant: Name,
        bindings: Vec<Name>,
    },
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

    /// Get the variant name
    pub fn variant_name(&self) -> &Name {
        match self {
            TaskMsgPattern::Simple(name) => name,
            TaskMsgPattern::WithBindings { variant, .. } => variant,
        }
    }

    /// Check if this pattern has bindings
    pub fn has_bindings(&self) -> bool {
        matches!(self, TaskMsgPattern::WithBindings { .. })
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
        for (pattern, body) in &self.handlers {
            write!(f, " ({} => {})", pattern, body)?;
        }
        if let Some(else_body) = &self.else_handler {
            write!(f, " (else => {})", else_body)?;
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
        for (pattern, body) in &self.on_block.handlers {
            write!(f, "\n        {} => {}", pattern, body.to_atom_str())?;
        }
        if let Some(else_body) = &self.on_block.else_handler {
            write!(f, "\n        else => {}", else_body.to_atom_str())?;
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
        write!(f, "on {{")?;
        for (pattern, body) in &self.handlers {
            write!(f, " {} => {}", pattern, body.to_atom_str())?;
        }
        if let Some(else_body) = &self.else_handler {
            write!(f, " else => {}", else_body.to_atom_str())?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}

impl ToNode for TaskOnBlock {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("on");

        for (pattern, body) in &self.handlers {
            let mut handler_node = AutoNode::new("handler");
            handler_node.set_prop("pattern", Value::str(pattern.to_string().as_str()));
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
        node.set_prop("variant", Value::str(self.variant_name().as_str()));

        if let TaskMsgPattern::WithBindings { bindings, .. } = self {
            for binding in bindings {
                let mut binding_node = AutoNode::new("binding");
                binding_node.set_prop("name", Value::str(binding.as_str()));
                node.add_kid(binding_node);
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
        assert_eq!(pattern.variant_name(), &AutoStr::from("Reset"));
        assert!(!pattern.has_bindings());
    }

    #[test]
    fn test_task_msg_pattern_with_bindings() {
        let pattern = TaskMsgPattern::with_bindings(
            "Add".into(),
            vec!["val".into(), "other".into()],
        );
        assert_eq!(pattern.to_string(), "Add(val, other)");
        assert_eq!(pattern.variant_name(), &AutoStr::from("Add"));
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
}
