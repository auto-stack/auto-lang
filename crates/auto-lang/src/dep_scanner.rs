// Dep Statement Scanner
//
// Plan 092: Fast scanning of `dep` statements from source code
//
// Scans source code for dependency declarations without full parsing.
// Used during preprocessing to register Rust crate dependencies.
//
// # Example
//
// ```
// use auto_lang::dep_scanner::{scan_dep_statements, DepStatement};
//
// let source = r#"
// dep serde(version: "1.0", features: ["derive"])
// dep my_lib(path: "../my_lib")
// "#;
//
// let deps = scan_dep_statements(source);
// assert_eq!(deps.len(), 2);
// ```

/// Dep statement information
#[derive(Debug, Clone, PartialEq)]
pub struct DepStatement {
    /// Crate name (e.g., "serde", "serde_json")
    pub name: String,

    /// Version specification (optional)
    pub version: Option<String>,

    /// Feature flags
    pub features: Vec<String>,

    /// Local path for local crates
    pub path: Option<String>,

    /// Git repository URL
    pub git: Option<String>,

    /// Git reference (branch/tag/commit)
    pub git_ref: Option<String>,

    /// Whether this is a Rust import
    pub is_rust: bool,
}

impl DepStatement {
    /// Create a new dep statement with just the crate name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
            features: Vec::new(),
            path: None,
            git: None,
            git_ref: None,
            is_rust: true,
        }
    }

    /// Check if this is a local dependency
    pub fn is_local(&self) -> bool {
        self.path.is_some()
    }

    /// Check if this is a git dependency
    pub fn is_git(&self) -> bool {
        self.git.is_some()
    }

    /// Check if this is a crates.io dependency
    pub fn is_crates_io(&self) -> bool {
        !self.is_local() && !self.is_git()
    }
}

/// Scan source code for all `dep` statements
///
/// Uses simple string matching without full parsing.
/// Suitable for preprocessing phase to collect dependencies quickly.
///
/// # Supported Syntax
///
/// - `dep serde` - Latest version
/// - `dep serde(version: "1.0")` - Specific version
/// - `dep serde(version: "1.0", features: ["derive"])` - With features
/// - `dep my_lib(path: "../my_lib")` - Local crate
/// - `dep tokio(git: "https://...", branch: "main")` - Git source
pub fn scan_dep_statements(source: &str) -> Vec<DepStatement> {
    let mut deps = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") {
            continue;
        }

        // Look for `dep` keyword at start of line
        if let Some(rest) = trimmed.strip_prefix("dep ") {
            if let Some(dep) = parse_dep_line(rest.trim()) {
                deps.push(dep);
            }
        }
    }

    deps
}

/// Parse a single dep statement line
///
/// Expected format: `crate_name` or `crate_name(props)`
fn parse_dep_line(line: &str) -> Option<DepStatement> {
    // Find crate name (up to '(' or end)
    let name_end = line.find('(').unwrap_or(line.len());
    let name = line[..name_end].trim();

    if name.is_empty() || !is_valid_crate_name(name) {
        return None;
    }

    let mut dep = DepStatement::new(name);

    // Parse properties if present
    if let Some(props_start) = line.find('(') {
        if let Some(props_end) = line.find(')') {
            let props_str = &line[props_start + 1..props_end];
            parse_dep_properties(props_str, &mut dep);
        }
    }

    Some(dep)
}

/// Check if a string is a valid crate name
fn is_valid_crate_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Must start with letter or underscore
    let first = s.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    // Rest can be alphanumeric, underscore, or hyphen
    s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

/// Parse dep properties: `version: "1.0", features: ["derive"]`
fn parse_dep_properties(props_str: &str, dep: &mut DepStatement) {
    // Simple property parsing
    let mut i = 0;
    let chars: Vec<char> = props_str.chars().collect();

    while i < chars.len() {
        // Skip whitespace
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        if i >= chars.len() {
            break;
        }

        // Find property name
        let name_start = i;
        while i < chars.len() && (chars[i].is_alphabetic() || chars[i] == '_') {
            i += 1;
        }
        let prop_name: String = chars[name_start..i].iter().collect();

        // Skip whitespace and colon
        while i < chars.len() && (chars[i].is_whitespace() || chars[i] == ':') {
            i += 1;
        }

        // Parse value
        if i >= chars.len() {
            break;
        }

        match prop_name.as_str() {
            "version" => {
                if let Some(v) = parse_string_value(&chars, &mut i) {
                    dep.version = Some(v);
                }
            }
            "features" => {
                dep.features = parse_array_value(&chars, &mut i);
            }
            "path" => {
                if let Some(v) = parse_string_value(&chars, &mut i) {
                    dep.path = Some(v);
                }
            }
            "git" => {
                if let Some(v) = parse_string_value(&chars, &mut i) {
                    dep.git = Some(v);
                }
            }
            "branch" | "tag" | "rev" => {
                if let Some(v) = parse_string_value(&chars, &mut i) {
                    dep.git_ref = Some(v);
                }
            }
            _ => {
                // Unknown property, skip to next
                skip_value(&chars, &mut i);
            }
        }

        // Skip comma
        while i < chars.len() && (chars[i].is_whitespace() || chars[i] == ',') {
            i += 1;
        }
    }
}

/// Parse a string value: `"value"`
fn parse_string_value(chars: &[char], i: &mut usize) -> Option<String> {
    // Skip whitespace
    while *i < chars.len() && chars[*i].is_whitespace() {
        *i += 1;
    }

    // Expect opening quote
    if *i >= chars.len() || chars[*i] != '"' {
        return None;
    }
    *i += 1;

    // Find closing quote
    let start = *i;
    while *i < chars.len() && chars[*i] != '"' {
        *i += 1;
    }

    let value: String = chars[start..*i].iter().collect();

    // Skip closing quote
    if *i < chars.len() {
        *i += 1;
    }

    Some(value)
}

/// Parse an array value: `["a", "b"]`
fn parse_array_value(chars: &[char], i: &mut usize) -> Vec<String> {
    let mut items = Vec::new();

    // Skip whitespace
    while *i < chars.len() && chars[*i].is_whitespace() {
        *i += 1;
    }

    // Expect opening bracket
    if *i >= chars.len() || chars[*i] != '[' {
        return items;
    }
    *i += 1;

    // Parse items
    while *i < chars.len() {
        // Skip whitespace
        while *i < chars.len() && chars[*i].is_whitespace() {
            *i += 1;
        }

        if *i >= chars.len() || chars[*i] == ']' {
            break;
        }

        // Skip comma
        if chars[*i] == ',' {
            *i += 1;
            continue;
        }

        // Parse string
        if let Some(v) = parse_string_value(chars, i) {
            items.push(v);
        } else {
            break;
        }
    }

    // Skip closing bracket
    if *i < chars.len() && chars[*i] == ']' {
        *i += 1;
    }

    items
}

/// Skip a value we don't recognize
fn skip_value(chars: &[char], i: &mut usize) {
    // Skip until comma or end
    while *i < chars.len() && chars[*i] != ',' {
        *i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_simple_dep() {
        let source = r#"
dep serde
dep tokio
"#;
        let deps = scan_dep_statements(source);
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].name, "serde");
        assert_eq!(deps[1].name, "tokio");
    }

    #[test]
    fn test_scan_dep_with_version() {
        let source = r#"dep serde(version: "1.0")"#;
        let deps = scan_dep_statements(source);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "serde");
        assert_eq!(deps[0].version, Some("1.0".to_string()));
    }

    #[test]
    fn test_scan_dep_with_features() {
        let source = r#"dep serde(version: "1.0", features: ["derive", "rc"])"#;
        let deps = scan_dep_statements(source);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "serde");
        assert_eq!(deps[0].features, vec!["derive", "rc"]);
    }

    #[test]
    fn test_scan_dep_with_path() {
        let source = r#"dep my_lib(path: "../my_lib")"#;
        let deps = scan_dep_statements(source);
        assert_eq!(deps.len(), 1);
        assert!(deps[0].is_local());
        assert_eq!(deps[0].path, Some("../my_lib".to_string()));
    }

    #[test]
    fn test_scan_dep_with_git() {
        let source = r#"dep tokio(git: "https://github.com/tokio-rs/tokio", branch: "main")"#;
        let deps = scan_dep_statements(source);
        assert_eq!(deps.len(), 1);
        assert!(deps[0].is_git());
        assert_eq!(deps[0].git, Some("https://github.com/tokio-rs/tokio".to_string()));
        assert_eq!(deps[0].git_ref, Some("main".to_string()));
    }

    #[test]
    fn test_scan_dep_skip_comments() {
        let source = r#"
// This is a comment
dep serde
// dep should_ignore_this
dep tokio
"#;
        let deps = scan_dep_statements(source);
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn test_dep_types() {
        let local = DepStatement::new("test");
        assert!(local.is_crates_io());

        let mut with_path = DepStatement::new("test");
        with_path.path = Some("../lib".to_string());
        assert!(with_path.is_local());

        let mut with_git = DepStatement::new("test");
        with_git.git = Some("https://...".to_string());
        assert!(with_git.is_git());
    }
}
