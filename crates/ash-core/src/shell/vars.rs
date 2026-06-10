//! Shell variables and environment management

use std::collections::HashMap;

/// Shell variable storage
#[derive(Debug, Clone)]
pub struct ShellVars {
    /// Local shell variables (not exported to child processes)
    locals: HashMap<String, String>,
    /// Environment variables (exported to child processes)
    env: HashMap<String, String>,
}

impl ShellVars {
    /// Create a new variable store
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            env: HashMap::new(),
        }
    }

    /// Set a local variable
    pub fn set_local(&mut self, name: String, value: String) {
        self.locals.insert(name, value);
    }

    /// Get a local variable
    pub fn get_local(&self, name: &str) -> Option<&String> {
        self.locals.get(name)
    }

    /// Set an environment variable (export)
    pub fn set_env(&mut self, name: String, value: String) {
        // Update our copy
        self.env.insert(name.clone(), value.clone());
        // Update actual process environment
        std::env::set_var(name, value);
    }

    /// Get an environment variable
    pub fn get_env(&self, name: &str) -> Option<String> {
        // Check our copy first, then fall back to process environment
        self.env.get(name)
            .cloned()
            .or_else(|| std::env::var(name).ok())
    }

    /// Unset a local variable
    pub fn unset_local(&mut self, name: &str) {
        self.locals.remove(name);
    }

    /// Unset an environment variable
    pub fn unset_env(&mut self, name: &str) {
        self.env.remove(name);
        std::env::remove_var(name);
    }

    /// List all local variables
    pub fn list_locals(&self) -> Vec<(String, String)> {
        self.locals.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// List all environment variables
    pub fn list_env(&self) -> Vec<(String, String)> {
        self.env.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

impl Default for ShellVars {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get_local() {
        let mut vars = ShellVars::new();
        vars.set_local("test".to_string(), "value".to_string());
        assert_eq!(vars.get_local("test"), Some(&"value".to_string()));
    }

    #[test]
    fn test_set_get_env() {
        let mut vars = ShellVars::new();
        vars.set_env("TEST_VAR".to_string(), "test_value".to_string());
        assert_eq!(vars.get_env("TEST_VAR"), Some("test_value".to_string()));
    }

    #[test]
    fn test_unset_local() {
        let mut vars = ShellVars::new();
        vars.set_local("test".to_string(), "value".to_string());
        vars.unset_local("test");
        assert_eq!(vars.get_local("test"), None);
    }

    #[test]
    fn test_list_locals() {
        let mut vars = ShellVars::new();
        vars.set_local("a".to_string(), "1".to_string());
        vars.set_local("b".to_string(), "2".to_string());
        let list = vars.list_locals();
        assert_eq!(list.len(), 2);
    }
}
