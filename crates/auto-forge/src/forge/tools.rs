//! AutoSmith Tool System
//!
//! Implements the core tools that the Forge agent can use to interact with
//! the codebase: read_file, write_file, edit_file, shell, and search.

use serde::Serialize;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

// ─── Tool Context (injected by forge_stream handler) ─────────────────────────

thread_local! {
    static CURRENT_PROJECT: RefCell<String> = RefCell::new(String::new());
    static CURRENT_SESSION_ID: RefCell<String> = RefCell::new(String::new());
}

/// Set the project and session context for Jades tools.
pub fn set_tool_context(project: &str, session_id: &str) {
    CURRENT_PROJECT.with(|p| *p.borrow_mut() = project.to_string());
    CURRENT_SESSION_ID.with(|s| *s.borrow_mut() = session_id.to_string());
}

// ─── Tool Definition ─────────────────────────────────────────────────────────

/// A tool that the AI agent can invoke.
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn input_schema(&self) -> Value;
    fn execute(&self, args: Value) -> Result<String, String>;
}

/// Definition of a tool for the Claude API
#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

impl ToolDefinition {
    pub fn from_tool(tool: &dyn Tool) -> Self {
        Self {
            name: tool.name().to_string(),
            description: tool.description().to_string(),
            input_schema: tool.input_schema(),
        }
    }
}

// ─── Tool Registry ───────────────────────────────────────────────────────────

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        registry.register(Box::new(ReadFileTool));
        registry.register(Box::new(WriteFileTool));
        registry.register(Box::new(EditFileTool));
        registry.register(Box::new(ShellTool));
        registry.register(Box::new(SearchTool));
        registry.register(Box::new(ReadJadeTool));
        registry.register(Box::new(WriteJadeTool));
        registry.register(Box::new(ListJadesTool));
        registry
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|t| ToolDefinition::from_tool(t.as_ref()))
            .collect()
    }

    pub fn names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Individual Tools ────────────────────────────────────────────────────────

/// Read the contents of a file.
struct ReadFileTool;

impl Tool for ReadFileTool {
    fn name(&self) -> &'static str {
        "read_file"
    }

    fn description(&self) -> &'static str {
        "Read the full contents of a file at the given path. \
         Returns the file contents as a string. \
         Use this to examine source code, configuration files, or documentation."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The relative path to the file to read"
                }
            },
            "required": ["path"]
        })
    }

    fn execute(&self, args: Value) -> Result<String, String> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'path' argument")?;

        // Security: restrict to project directory
        let path = Path::new(path);
        if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err("Path cannot contain '..'".to_string());
        }

        std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))
    }
}

/// Write content to a file (creates or overwrites).
struct WriteFileTool;

impl Tool for WriteFileTool {
    fn name(&self) -> &'static str {
        "write_file"
    }

    fn description(&self) -> &'static str {
        "Write content to a file at the given path. \
         Creates the file if it doesn't exist, overwrites if it does. \
         Use this to create new source files or completely rewrite existing ones."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The relative path to the file"
                },
                "content": {
                    "type": "string",
                    "description": "The full content to write"
                }
            },
            "required": ["path", "content"]
        })
    }

    fn execute(&self, args: Value) -> Result<String, String> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'path' argument")?;
        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'content' argument")?;

        let path = Path::new(path);
        if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err("Path cannot contain '..'".to_string());
        }

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directories: {}", e))?;
        }

        std::fs::write(path, content)
            .map(|_| format!("Successfully wrote {} bytes to {}", content.len(), path.display()))
            .map_err(|e| format!("Failed to write file '{}': {}", path.display(), e))
    }
}

/// Edit a file by replacing old text with new text.
struct EditFileTool;

impl Tool for EditFileTool {
    fn name(&self) -> &'static str {
        "edit_file"
    }

    fn description(&self) -> &'static str {
        "Replace a specific string in a file with another string. \
         Use this for surgical edits when you only need to change a small part of a file. \
         The old_string must match exactly (including whitespace)."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The relative path to the file"
                },
                "old_string": {
                    "type": "string",
                    "description": "The exact text to replace"
                },
                "new_string": {
                    "type": "string",
                    "description": "The replacement text"
                }
            },
            "required": ["path", "old_string", "new_string"]
        })
    }

    fn execute(&self, args: Value) -> Result<String, String> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'path' argument")?;
        let old_str = args
            .get("old_string")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'old_string' argument")?;
        let new_str = args
            .get("new_string")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'new_string' argument")?;

        let path = Path::new(path);
        if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err("Path cannot contain '..'".to_string());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))?;

        if !content.contains(old_str) {
            return Err(format!(
                "old_string not found in file '{}'. \
                 The text must match exactly (including whitespace and newlines).",
                path.display()
            ));
        }

        let new_content = content.replacen(old_str, new_str, 1);
        std::fs::write(path, new_content)
            .map(|_| format!("Successfully edited {}", path.display()))
            .map_err(|e| format!("Failed to write file '{}': {}", path.display(), e))
    }
}

/// Execute a shell command.
struct ShellTool;

impl Tool for ShellTool {
    fn name(&self) -> &'static str {
        "shell"
    }

    fn description(&self) -> &'static str {
        "Execute a shell command in the project directory. \
         Use this to run tests, check git status, list files, install dependencies, etc. \
         Be careful with destructive commands."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                }
            },
            "required": ["command"]
        })
    }

    fn execute(&self, args: Value) -> Result<String, String> {
        let cmd = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'command' argument")?;

        // Security: block dangerous commands
        let blocked = ["rm -rf /", "> /dev/", ":(){ :|:& };:", "mkfs"];
        for b in &blocked {
            if cmd.contains(b) {
                return Err(format!("Command blocked for safety: contains '{}'", b));
            }
        }

        let output = std::process::Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Err(format!(
                "Command exited with code {}\nSTDOUT:\n{}\nSTDERR:\n{}",
                output.status.code().unwrap_or(-1),
                stdout,
                stderr
            ));
        }

        let mut result = String::new();
        if !stdout.is_empty() {
            result.push_str(&format!("STDOUT:\n{}\n", stdout));
        }
        if !stderr.is_empty() {
            result.push_str(&format!("STDERR:\n{}\n", stderr));
        }

        Ok(if result.is_empty() {
            "Command executed successfully (no output)".to_string()
        } else {
            result
        })
    }
}

/// Search for text in files using grep-like functionality.
struct SearchTool;

impl Tool for SearchTool {
    fn name(&self) -> &'static str {
        "search"
    }

    fn description(&self) -> &'static str {
        "Search for a pattern in files under the project directory. \
         Returns matching file paths with line numbers and snippets. \
         Use this to find where functions, types, or patterns are defined or used."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The text or regex pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in (default: current directory)"
                }
            },
            "required": ["pattern"]
        })
    }

    fn execute(&self, args: Value) -> Result<String, String> {
        let pattern = args
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'pattern' argument")?;
        let search_path = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let search_path = Path::new(search_path);
        if search_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err("Path cannot contain '..'".to_string());
        }

        let mut results = Vec::new();
        walk_dir(search_path, pattern, &mut results)
            .map_err(|e| format!("Search error: {}", e))?;

        if results.is_empty() {
            Ok(format!("No matches found for '{}' in {}", pattern, search_path.display()))
        } else {
            Ok(results.join("\n"))
        }
    }
}

fn walk_dir(
    dir: &Path,
    pattern: &str,
    results: &mut Vec<String>,
) -> Result<(), std::io::Error> {
    if !dir.is_dir() {
        search_file(dir, pattern, results)?;
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip hidden dirs and common non-source directories
        if path.is_dir() {
            if name_str.starts_with('.')
                || name_str == "target"
                || name_str == "node_modules"
                || name_str == "dist"
                || name_str == "build"
            {
                continue;
            }
            walk_dir(&path, pattern, results)?;
        } else if path.is_file() {
            // Skip binary files
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if matches!(ext, "jpg" | "jpeg" | "png" | "gif" | "ico" | "woff" | "woff2" | "ttf" | "eot" | "wasm") {
                continue;
            }
            search_file(&path, pattern, results)?;
        }
    }

    Ok(())
}

fn search_file(path: &Path, pattern: &str, results: &mut Vec<String>) -> Result<(), std::io::Error> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Ok(()), // Skip unreadable files (binary, etc.)
    };

    for (line_num, line) in content.lines().enumerate() {
        if line.contains(pattern) {
            results.push(format!(
                "{}:{}: {}",
                path.display(),
                line_num + 1,
                line.trim()
            ));
            if results.len() >= 50 {
                results.push("... (truncated at 50 matches)".to_string());
                return Ok(());
            }
        }
    }

    Ok(())
}

// ─── Jades Tools ─────────────────────────────────────────────────────────────

/// Read a Jades (Specs) section.
struct ReadJadeTool;

impl Tool for ReadJadeTool {
    fn name(&self) -> &'static str {
        "read_jade"
    }

    fn description(&self) -> &'static str {
        "Read the content and status of a Jades (Specs) section. \
         Use this to examine the current project specification during Intake or SpecDraft."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "section_id": {
                    "type": "string",
                    "description": "The section ID to read (e.g., 'goals', 'architecture', 'plans', 'tests')"
                }
            },
            "required": ["section_id"]
        })
    }

    fn execute(&self, args: Value) -> Result<String, String> {
        let project = CURRENT_PROJECT.with(|p| p.borrow().clone());
        let sid = CURRENT_SESSION_ID.with(|s| s.borrow().clone());
        let section_id = args
            .get("section_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'section_id' argument")?;

        if project.is_empty() {
            return Err("No project context set".to_string());
        }

        // Overlay pending spec changes if any
        let pending = if !sid.is_empty() {
            super::forge_sessions()
                .lock()
                .unwrap()
                .get(&sid)
                .and_then(|session| {
                    session.pending_spec_changes.iter()
                        .find(|c| c.section_id == section_id)
                        .map(|c| (c.new_content.clone(), c.new_status.clone()))
                })
        } else {
            None
        };

        let (content, status) = if let Some((c, s)) = pending {
            (c, s)
        } else {
            let store = super::specs().lock().unwrap();
            match store.get(&project)
                .and_then(|doc| doc.sections.iter().find(|s| s.id == section_id))
            {
                Some(sec) => (sec.content.clone(), sec.status.as_str().to_string()),
                None => return Err(format!("Section '{}' not found in project '{}'", section_id, project)),
            }
        };

        Ok(format!(
            "Section: {}\nStatus: {}\n---\n{}",
            section_id, status, content
        ))
    }
}

/// List all Jades sections.
struct ListJadesTool;

impl Tool for ListJadesTool {
    fn name(&self) -> &'static str {
        "list_jades"
    }

    fn description(&self) -> &'static str {
        "List all Jades (Specs) sections with their titles and statuses. \
         Use this to get an overview of the project specification."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    fn execute(&self, _args: Value) -> Result<String, String> {
        let project = CURRENT_PROJECT.with(|p| p.borrow().clone());
        if project.is_empty() {
            return Err("No project context set".to_string());
        }

        let sid = CURRENT_SESSION_ID.with(|s| s.borrow().clone());
        let pending: HashMap<String, (String, String)> = if !sid.is_empty() {
            super::forge_sessions()
                .lock()
                .unwrap()
                .get(&sid)
                .map(|session| {
                    session.pending_spec_changes.iter()
                        .map(|c| (c.section_id.clone(), (c.new_content.clone(), c.new_status.clone())))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            HashMap::new()
        };

        let store = super::specs().lock().unwrap();
        let doc = store.get(&project)
            .ok_or_else(|| format!("No specs found for project '{}'", project))?;

        let mut lines = vec![format!("Project: {}", project)];
        for section in &doc.sections {
            let has_pending = pending.contains_key(&section.id);
            let status = if has_pending {
                pending.get(&section.id).unwrap().1.clone()
            } else {
                section.status.as_str().to_string()
            };
            let marker = if has_pending { " [pending changes]" } else { "" };
            lines.push(format!(
                "- {}: {} [{}]{}",
                section.id, section.title, status, marker
            ));
        }

        Ok(lines.join("\n"))
    }
}

/// Draft a Jades section update (stored in pending_spec_changes until approved).
struct WriteJadeTool;

impl Tool for WriteJadeTool {
    fn name(&self) -> &'static str {
        "write_jade"
    }

    fn description(&self) -> &'static str {
        "Draft an update to a Jades (Specs) section. \
         The change is queued in pending_spec_changes and applied to the Specs only after human approval. \
         Use this during SpecDraft phase to propose updates to goals, architecture, designs, plans, or tests."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "section_id": {
                    "type": "string",
                    "description": "The section ID to update (e.g., 'goals', 'architecture', 'plans', 'tests')"
                },
                "content": {
                    "type": "string",
                    "description": "The new content for the section"
                },
                "status": {
                    "type": "string",
                    "description": "The status to set (default: 'draft')",
                    "enum": ["draft", "in_progress", "approved", "verified", "drift"]
                }
            },
            "required": ["section_id", "content"]
        })
    }

    fn execute(&self, args: Value) -> Result<String, String> {
        let project = CURRENT_PROJECT.with(|p| p.borrow().clone());
        let sid = CURRENT_SESSION_ID.with(|s| s.borrow().clone());

        if project.is_empty() || sid.is_empty() {
            return Err("No project or session context set".to_string());
        }

        let section_id = args
            .get("section_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'section_id' argument")?;
        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'content' argument")?;
        let status = args
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("draft");

        // Capture old content from specs (or from an earlier pending change)
        let (old_content, old_status) = {
            let sessions = super::forge_sessions().lock().unwrap();
            let session = sessions.get(&sid).ok_or("Session not found")?;
            if let Some(existing) = session.pending_spec_changes.iter().find(|c| c.section_id == section_id) {
                (existing.old_content.clone(), existing.old_status.clone())
            } else {
                let specs = super::specs().lock().unwrap();
                specs.get(&project)
                    .and_then(|doc| doc.sections.iter().find(|s| s.id == section_id))
                    .map(|s| (s.content.clone(), s.status.as_str().to_string()))
                    .unwrap_or_default()
            }
        };

        // Queue pending change
        {
            let mut sessions = super::forge_sessions().lock().unwrap();
            let session = sessions.get_mut(&sid).ok_or("Session not found")?;

            if let Some(existing) = session.pending_spec_changes.iter_mut().find(|c| c.section_id == section_id) {
                existing.new_content = content.to_string();
                existing.new_status = status.to_string();
            } else {
                session.pending_spec_changes.push(super::SpecChange {
                    section_id: section_id.to_string(),
                    item_id: None,
                    old_content,
                    new_content: content.to_string(),
                    old_status,
                    new_status: status.to_string(),
                });
            }

            let clone = session.clone();
            sessions.save(&clone);
        }

        Ok(format!(
            "Drafted update to section '{}'. Status: {}. Awaiting approval.",
            section_id, status
        ))
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_file_tool() {
        let tool = ReadFileTool;
        // Try to read Cargo.toml (should exist in project root)
        let result = tool.execute(serde_json::json!({"path": "Cargo.toml"}));
        assert!(result.is_ok(), "Failed to read Cargo.toml: {:?}", result.err());
        assert!(result.unwrap().contains("[package]"));
    }

    #[test]
    fn test_read_file_not_found() {
        let tool = ReadFileTool;
        let result = tool.execute(serde_json::json!({"path": "does_not_exist.txt"}));
        assert!(result.is_err());
    }

    #[test]
    fn test_write_and_edit_file() {
        let write_tool = WriteFileTool;
        let edit_tool = EditFileTool;
        let read_tool = ReadFileTool;
        let test_path = "/tmp/autosmith_test_file.txt";

        // Write
        let result = write_tool.execute(serde_json::json!({
            "path": test_path,
            "content": "hello world\nfoo bar\n"
        }));
        assert!(result.is_ok(), "{:?}", result);

        // Edit
        let result = edit_tool.execute(serde_json::json!({
            "path": test_path,
            "old_string": "foo bar",
            "new_string": "baz qux"
        }));
        assert!(result.is_ok(), "{:?}", result);

        // Read back
        let result = read_tool.execute(serde_json::json!({"path": test_path}));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("baz qux"));

        // Cleanup
        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn test_shell_tool() {
        let tool = ShellTool;
        let result = tool.execute(serde_json::json!({"command": "echo hello"}));
        assert!(result.is_ok(), "{:?}", result);
        assert!(result.unwrap().contains("hello"));
    }

    #[test]
    fn test_search_tool() {
        let tool = SearchTool;
        let result = tool.execute(serde_json::json!({
            "pattern": "fn main",
            "path": "."
        }));
        assert!(result.is_ok(), "{:?}", result);
        // Should find at least one main function in the project
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_tool_registry() {
        let registry = ToolRegistry::new();
        let defs = registry.definitions();
        assert_eq!(defs.len(), 8);
        assert!(registry.get("read_file").is_some());
        assert!(registry.get("write_file").is_some());
        assert!(registry.get("edit_file").is_some());
        assert!(registry.get("shell").is_some());
        assert!(registry.get("search").is_some());
        assert!(registry.get("read_jade").is_some());
        assert!(registry.get("write_jade").is_some());
        assert!(registry.get("list_jades").is_some());
    }
}
