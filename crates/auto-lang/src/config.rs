use crate::eval_config_with_vm;
use crate::AutoResult;
use auto_val::Obj;
use auto_val::{AutoPath, AutoStr, Node, Value};
use std::path::Path;
use std::path::PathBuf;

pub struct AutoConfigReader {
    args: Obj,
}

impl AutoConfigReader {
    pub fn new() -> Self {
        Self { args: Obj::new() }
    }

    pub fn skip_check(self) -> Self {
        // No-op in AutoVM mode (skip_check was an Interpreter feature)
        self
    }

    pub fn args(mut self, args: &Obj) -> Self {
        self.args = args.clone();
        self
    }

    pub fn parse(&mut self, code: impl Into<AutoStr>) -> AutoResult<AutoConfig> {
        let code = code.into();

        // Plan 091: Use AutoVM without Universe
        let result = eval_config_with_vm(code.as_str(), &self.args)?;

        Ok(AutoConfig {
            code: code.to_string(),
            root: result.to_node(),
            args: self.args.clone(),
        })
    }

    pub fn read(&mut self, path: impl Into<PathBuf>) -> AutoResult<AutoConfig> {
        let path = path.into();
        let content = std::fs::read_to_string(&path).map_err(|e| {
            format!(
                "Failed to read config file {}: {}",
                path.to_str().unwrap(),
                e
            )
        })?;
        self.parse(content)
    }
}

pub struct AutoConfig {
    pub code: String,
    pub root: Node,
    pub args: Obj,
}

impl AutoConfig {
    pub fn read(path: &Path) -> AutoResult<Self> {
        Self::from_file(path, &Obj::default())
    }

    pub fn from_file(path: &Path, args: &Obj) -> AutoResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            format!(
                "Failed to read config file {}: {}",
                path.to_str().unwrap(),
                e
            )
        })?;
        Self::from_code(content, args)
    }

    pub fn new(code: impl Into<String>) -> AutoResult<Self> {
        Self::from_code(code, &Obj::new())
    }

    pub fn save(&mut self, path: &AutoPath) -> AutoResult<()> {
        let contents = self.root.contents();
        std::fs::write(path.path(), contents.join("\n"))
            .map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    }

    pub fn from_code(code: impl Into<String>, args: &Obj) -> AutoResult<Self> {
        let code = code.into();

        // Plan 091: Use AutoVM without Universe
        let result = eval_config_with_vm(&code, args)?;

        if let Value::Node(root) = result {
            Ok(Self {
                code: code.clone(),
                args: args.clone(),
                root: root,
            })
        } else {
            // For empty config files (Nil result), return an empty root Node instead of error
            if matches!(result, Value::Nil) {
                return Ok(Self {
                    code: code.clone(),
                    args: args.clone(),
                    root: auto_val::Node::new("root"),
                });
            }

            Err(format!("Invalid config result: {}", result.repr()).into())
        }
    }

    pub fn name(&self) -> AutoStr {
        self.root.get_prop("name").to_astr()
    }

    pub fn version(&self) -> AutoStr {
        self.root.get_prop("version").to_astr()
    }

    pub fn list_target_names(&self) -> Vec<AutoStr> {
        self.root
            .kids_iter()
            .filter(|(_, kid)| matches!(kid, auto_val::Kid::Node(_)))
            .map(|(_, kid)| {
                if let auto_val::Kid::Node(n) = kid {
                    n.title()
                } else {
                    unreachable!()
                }
            })
            .collect()
    }

    pub fn to_xml(&self) -> AutoStr {
        AutoStr::new()
    }

    // pub fn eval(&mut self, code: impl Into<AutoStr>) -> Value {
    // self.interpreter.eval(code.into().as_str())
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_with_if() -> AutoResult<()> {
        let code = r#"
            name: "hello"

            var a = true

            if a {
                lib("alib") {}
            }
        "#;

        let mut reader = AutoConfigReader::new();
        let config = reader.parse(code)?;
        assert_eq!(config.name(), "hello");
        assert_eq!(config.list_target_names(), vec!["lib(\"alib\")"]);

        // Note: eval() on the reader is no longer supported with AutoVM
        // Use run() or run_autovm() for script execution

        Ok(())
    }
}
