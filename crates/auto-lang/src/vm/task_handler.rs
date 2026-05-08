//! Plan 127: TaskHandlerTable - Handler Metadata for Task Message Routing
//!
//! This module provides structures for storing and looking up task message handlers.
//!
//! ## Overview
//!
//! When a task is compiled, each `on` block is converted to a TaskHandler entry
//! containing:
//! - Pattern index for matching
//! - Bytecode offset for the handler body
//! - Whether the handler has a context parameter (on(ctx))
//!
//! The TaskHandlerTable stores all handlers for a task type and provides
//! efficient lookup during message routing.

use crate::ast::TaskMsgPattern;
use std::collections::HashMap;

/// Task handler metadata for message routing
#[derive(Debug, Clone)]
pub struct TaskHandler {
    /// Pattern index for matching (index into TaskHandlerTable::patterns)
    pub pattern_idx: u32,
    /// Bytecode offset for handler body
    pub body_offset: u32,
    /// Whether this handler has a context parameter (on(ctx))
    pub has_context: bool,
}

impl TaskHandler {
    /// Create a new handler entry
    pub fn new(pattern_idx: u32, body_offset: u32, has_context: bool) -> Self {
        Self {
            pattern_idx,
            body_offset,
            has_context,
        }
    }
}

/// Pattern types for serialization
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PatternType {
    Literal = 0x01,
    TypeBinding = 0x02,
    Simple = 0x03,
    WithBindings = 0x04,
}

/// Serialized pattern data for runtime matching
#[derive(Debug, Clone)]
pub struct SerializedPattern {
    /// Pattern type
    pub pattern_type: PatternType,
    /// Raw pattern bytes (format depends on pattern_type)
    pub data: Vec<u8>,
}

impl SerializedPattern {
    /// Create from TaskMsgPattern AST node
    pub fn from_ast(pattern: &TaskMsgPattern, string_pool: &mut Vec<String>) -> Self {
        match pattern {
            TaskMsgPattern::Literal(lit) => {
                let mut data = Vec::new();
                match lit {
                    crate::ast::LiteralValue::String(s) => {
                        data.push(0x01); // String literal type
                        let idx = get_or_add_string(string_pool, s);
                        data.extend_from_slice(&idx.to_le_bytes());
                    }
                    crate::ast::LiteralValue::Int(n) => {
                        data.push(0x02); // Int literal type
                        data.extend_from_slice(&n.to_le_bytes());
                    }
                    crate::ast::LiteralValue::Uint(n) => {
                        data.push(0x03); // Uint literal type
                        data.extend_from_slice(&n.to_le_bytes());
                    }
                    crate::ast::LiteralValue::Bool(b) => {
                        data.push(0x04); // Bool literal type
                        data.push(if *b { 1 } else { 0 });
                    }
                    crate::ast::LiteralValue::Char(c) => {
                        data.push(0x05); // Char literal type
                        data.extend_from_slice(&(*c as u32).to_le_bytes());
                    }
                    crate::ast::LiteralValue::Float(integral, fractional) => {
                        data.push(0x06); // Float literal type
                        data.extend_from_slice(&integral.to_le_bytes());
                        data.extend_from_slice(&fractional.to_le_bytes());
                    }
                }
                SerializedPattern {
                    pattern_type: PatternType::Literal,
                    data,
                }
            }
            TaskMsgPattern::TypeBinding { name, type_expr } => {
                let mut data = Vec::new();
                let name_idx = get_or_add_string(string_pool, name);
                data.extend_from_slice(&name_idx.to_le_bytes());

                // Serialize type tag
                let type_tag = type_expr_to_tag(type_expr);
                data.push(type_tag);

                SerializedPattern {
                    pattern_type: PatternType::TypeBinding,
                    data,
                }
            }
            TaskMsgPattern::Simple(name) => {
                let mut data = Vec::new();
                let name_idx = get_or_add_string(string_pool, name);
                data.extend_from_slice(&name_idx.to_le_bytes());

                SerializedPattern {
                    pattern_type: PatternType::Simple,
                    data,
                }
            }
            TaskMsgPattern::WithBindings { variant, bindings } => {
                let mut data = Vec::new();
                let variant_idx = get_or_add_string(string_pool, variant);
                data.extend_from_slice(&variant_idx.to_le_bytes());

                // Number of bindings
                data.push(bindings.len() as u8);

                // Each binding name
                for binding in bindings {
                    let binding_idx = get_or_add_string(string_pool, binding.as_str());
                    data.extend_from_slice(&binding_idx.to_le_bytes());
                }

                SerializedPattern {
                    pattern_type: PatternType::WithBindings,
                    data,
                }
            }
        }
    }
}

/// Convert Type to a tag byte for serialization
fn type_expr_to_tag(ty: &crate::ast::Type) -> u8 {
    use crate::ast::Type;
    match ty {
        Type::Int => 0x01,
        Type::I64 => 0x02,
        Type::Uint => 0x03,
        Type::U64 => 0x04,
        Type::Float => 0x05,
        Type::Double => 0x06,
        Type::Bool => 0x07,
        Type::Char => 0x08,
        Type::StrFixed(_) | Type::StrOwned => 0x09,
        Type::StrSlice => 0x0A,
        Type::CStrLit => 0x0B,
        Type::Void => 0x0C,
        Type::Byte => 0x0D,
        Type::Unknown => 0xFF,
        _ => 0xFF, // Default to unknown for complex types
    }
}

/// Get or add a string to the pool, returning its index
fn get_or_add_string(pool: &mut Vec<String>, s: &str) -> u32 {
    // Check if string already exists
    for (i, existing) in pool.iter().enumerate() {
        if existing == s {
            return i as u32;
        }
    }
    // Add new string
    let idx = pool.len() as u32;
    pool.push(s.to_string());
    idx
}

/// Table of handlers for a task type
#[derive(Debug, Clone)]
pub struct TaskHandlerTable {
    /// Task type name (e.g., "CounterTask")
    pub task_type: String,
    /// Handler entries
    pub handlers: Vec<TaskHandler>,
    /// Serialized patterns for matching
    pub patterns: Vec<SerializedPattern>,
    /// String pool for pattern data
    pub string_pool: Vec<String>,
    /// Start hook bytecode offset (if present)
    pub start_hook_offset: Option<u32>,
    /// Stop hook bytecode offset (if present)
    pub stop_hook_offset: Option<u32>,
    /// Else handler bytecode offset (if present)
    pub else_handler_offset: Option<u32>,
}

impl TaskHandlerTable {
    /// Create a new empty handler table
    pub fn new(task_type: String) -> Self {
        Self {
            task_type,
            handlers: Vec::new(),
            patterns: Vec::new(),
            string_pool: Vec::new(),
            start_hook_offset: None,
            stop_hook_offset: None,
            else_handler_offset: None,
        }
    }

    /// Add a handler to the table
    pub fn add_handler(
        &mut self,
        pattern: &TaskMsgPattern,
        body_offset: u32,
        has_context: bool,
    ) -> u32 {
        // Serialize pattern
        let serialized = SerializedPattern::from_ast(pattern, &mut self.string_pool);
        let pattern_idx = self.patterns.len() as u32;
        self.patterns.push(serialized);

        // Add handler entry
        let handler = TaskHandler::new(pattern_idx, body_offset, has_context);
        self.handlers.push(handler);

        pattern_idx
    }

    /// Get handler by pattern index
    pub fn get_handler(&self, pattern_idx: u32) -> Option<&TaskHandler> {
        self.handlers.iter().find(|h| h.pattern_idx == pattern_idx)
    }

    /// Get handler by bytecode offset
    pub fn get_handler_by_offset(&self, offset: u32) -> Option<&TaskHandler> {
        self.handlers.iter().find(|h| h.body_offset == offset)
    }

    /// Get all handlers
    pub fn get_handlers(&self) -> &[TaskHandler] {
        &self.handlers
    }

    /// Get pattern by index
    pub fn get_pattern(&self, idx: u32) -> Option<&SerializedPattern> {
        self.patterns.get(idx as usize)
    }

    /// Get string from pool
    pub fn get_string(&self, idx: u32) -> Option<&str> {
        self.string_pool.get(idx as usize).map(|s| s.as_str())
    }

    /// Number of handlers
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    /// Plan 128: Create TaskHandlerTable from pre-serialized components
    ///
    /// Used by VMLoader to convert CompiledPackage task definitions
    /// into runtime handler tables.
    pub fn from_components(
        task_type: String,
        handlers: Vec<TaskHandler>,
        patterns: Vec<SerializedPattern>,
        string_pool: Vec<String>,
        start_hook_offset: Option<u32>,
        stop_hook_offset: Option<u32>,
        else_handler_offset: Option<u32>,
    ) -> Self {
        Self {
            task_type,
            handlers,
            patterns,
            string_pool,
            start_hook_offset,
            stop_hook_offset,
            else_handler_offset,
        }
    }
}

/// Registry of all task handler tables
#[derive(Debug, Clone, Default)]
pub struct TaskHandlerRegistry {
    /// Maps task type name to handler table
    tables: HashMap<String, TaskHandlerTable>,
}

impl TaskHandlerRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    /// Register a handler table for a task type
    pub fn register(&mut self, table: TaskHandlerTable) {
        self.tables.insert(table.task_type.clone(), table);
    }

    /// Get handler table for a task type
    pub fn get_table(&self, task_type: &str) -> Option<&TaskHandlerTable> {
        self.tables.get(task_type)
    }

    /// Check if a task type has handlers
    pub fn has_handlers(&self, task_type: &str) -> bool {
        self.tables.contains_key(task_type)
    }

    /// Get all registered task types
    pub fn task_types(&self) -> impl Iterator<Item = &String> {
        self.tables.keys()
    }

    /// Plan 128: Export all task definitions for CompiledPackage
    ///
    /// Converts the internal TaskHandlerTable entries into TaskDefinition
    /// structures that can be serialized into CompiledPackage.
    pub fn export_task_definitions(&self) -> HashMap<String, crate::vm::loader::TaskDefinition> {
        self.tables
            .iter()
            .map(|(_key, table)| {
                let def = crate::vm::loader::TaskDefinition {
                    name: table.task_type.clone(),
                    is_single: false, // TODO: Track this in TaskHandlerTable
                    patterns: table.patterns.clone(),
                    handlers: table.handlers.clone(),
                    start_hook_offset: table.start_hook_offset,
                    stop_hook_offset: table.stop_hook_offset,
                    else_handler_offset: table.else_handler_offset,
                    strings: table.string_pool.clone(),
                };
                (table.task_type.clone(), def)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{LiteralValue, TaskMsgPattern, Type};

    #[test]
    fn test_task_handler_new() {
        let handler = TaskHandler::new(0, 100, true);
        assert_eq!(handler.pattern_idx, 0);
        assert_eq!(handler.body_offset, 100);
        assert!(handler.has_context);
    }

    #[test]
    fn test_task_handler_table_new() {
        let table = TaskHandlerTable::new("TestTask".to_string());
        assert_eq!(table.task_type, "TestTask");
        assert_eq!(table.handler_count(), 0);
    }

    #[test]
    fn test_add_literal_pattern() {
        let mut table = TaskHandlerTable::new("TestTask".to_string());
        let pattern = TaskMsgPattern::Literal(LiteralValue::String("ping".into()));

        let idx = table.add_handler(&pattern, 100, false);

        assert_eq!(idx, 0);
        assert_eq!(table.handler_count(), 1);
        assert_eq!(table.string_pool.len(), 1);
        assert_eq!(table.string_pool[0], "ping");
    }

    #[test]
    fn test_add_type_binding_pattern() {
        let mut table = TaskHandlerTable::new("TestTask".to_string());
        let pattern = TaskMsgPattern::TypeBinding {
            name: "msg".into(),
            type_expr: Box::new(Type::StrFixed(0)),
        };

        let idx = table.add_handler(&pattern, 200, true);

        assert_eq!(idx, 0);
        assert_eq!(table.handler_count(), 1);

        let handler = table.get_handler(idx).unwrap();
        assert!(handler.has_context);
    }

    #[test]
    fn test_add_simple_pattern() {
        let mut table = TaskHandlerTable::new("TestTask".to_string());
        let pattern = TaskMsgPattern::Simple("Reset".into());

        let idx = table.add_handler(&pattern, 300, false);

        assert_eq!(idx, 0);
        assert_eq!(table.handler_count(), 1);

        let serialized = table.get_pattern(idx).unwrap();
        assert_eq!(serialized.pattern_type as u8, PatternType::Simple as u8);
    }

    #[test]
    fn test_add_with_bindings_pattern() {
        let mut table = TaskHandlerTable::new("TestTask".to_string());
        let pattern = TaskMsgPattern::WithBindings {
            variant: "Add".into(),
            bindings: vec![auto_val::AutoStr::from("val")],
        };

        let idx = table.add_handler(&pattern, 400, false);

        assert_eq!(idx, 0);
        assert_eq!(table.handler_count(), 1);

        let serialized = table.get_pattern(idx).unwrap();
        assert_eq!(serialized.pattern_type as u8, PatternType::WithBindings as u8);
    }

    #[test]
    fn test_multiple_handlers() {
        let mut table = TaskHandlerTable::new("CounterTask".to_string());

        // Add multiple handlers
        let p1 = TaskMsgPattern::Literal(LiteralValue::String("ping".into()));
        let p2 = TaskMsgPattern::Simple("Reset".into());
        let p3 = TaskMsgPattern::WithBindings {
            variant: "Add".into(),
            bindings: vec![auto_val::AutoStr::from("val")],
        };

        table.add_handler(&p1, 100, true);
        table.add_handler(&p2, 200, false);
        table.add_handler(&p3, 300, false);

        assert_eq!(table.handler_count(), 3);

        // Verify string pool has unique strings
        assert!(table.string_pool.contains(&"ping".to_string()));
        assert!(table.string_pool.contains(&"Reset".to_string()));
        assert!(table.string_pool.contains(&"Add".to_string()));
        assert!(table.string_pool.contains(&"val".to_string()));
    }

    #[test]
    fn test_task_handler_registry() {
        let mut registry = TaskHandlerRegistry::new();

        let mut table1 = TaskHandlerTable::new("Task1".to_string());
        table1.add_handler(
            &TaskMsgPattern::Simple("A".into()),
            100,
            false,
        );

        let mut table2 = TaskHandlerTable::new("Task2".to_string());
        table2.add_handler(
            &TaskMsgPattern::Simple("B".into()),
            200,
            false,
        );

        registry.register(table1);
        registry.register(table2);

        assert!(registry.has_handlers("Task1"));
        assert!(registry.has_handlers("Task2"));
        assert!(!registry.has_handlers("Task3"));

        let t1 = registry.get_table("Task1").unwrap();
        assert_eq!(t1.handler_count(), 1);
    }

    #[test]
    fn test_type_expr_to_tag() {
        assert_eq!(type_expr_to_tag(&Type::Int), 0x01);
        assert_eq!(type_expr_to_tag(&Type::I64), 0x02);
        assert_eq!(type_expr_to_tag(&Type::Uint), 0x03);
        assert_eq!(type_expr_to_tag(&Type::Bool), 0x07);
        assert_eq!(type_expr_to_tag(&Type::StrFixed(0)), 0x09);
        assert_eq!(type_expr_to_tag(&Type::Unknown), 0xFF);
    }
}
