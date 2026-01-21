use crate::cmd::Command;
use miette::Result;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry for managing and executing shell commands
pub struct CommandRegistry {
    commands: HashMap<String, Arc<dyn Command>>,
}

impl CommandRegistry {
    /// Create a new command registry
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Register a command
    pub fn register(&mut self, command: Box<dyn Command>) {
        self.commands
            .insert(command.name().to_string(), Arc::from(command));
    }

    /// Get a registered command by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Command>> {
        self.commands.get(name).cloned()
    }

    /// List all registered commands
    pub fn params(&self) -> Vec<crate::cmd::Signature> {
        self.commands.values().map(|c| c.signature()).collect()
    }
}
