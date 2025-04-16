use crate::eval_config;
use crate::interp;
use crate::AutoResult;
use auto_val::Obj;
use auto_val::{AutoStr, Node, Value};
use std::path::Path;
pub struct AutoConfig {
    pub code: String,
    pub root: Node,
    pub interpreter: interp::Interpreter,
}

impl AutoConfig {
    pub fn read(path: &Path) -> AutoResult<Self> {
        Self::from_file(path, &Obj::default())
    }

    pub fn from_file(path: &Path, args: &Obj) -> AutoResult<Self> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        Self::from_code(content, args)
    }

    pub fn from_code(code: impl Into<String>, args: &Obj) -> AutoResult<Self> {
        let code = code.into();
        let mut interpreter = eval_config(&code, args)?;
        let result = interpreter.result;
        interpreter.result = Value::Nil;
        if let Value::Node(root) = result {
            Ok(Self {
                code: code,
                root: root,
                interpreter: interpreter,
            })
        } else {
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
        self.root.nodes.iter().map(|n| n.title()).collect()
    }
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

        let config = AutoConfig::from_code(code, &Obj::default())?;
        assert_eq!(config.name(), "hello");
        assert_eq!(config.list_target_names(), vec!["lib(\"alib\")"]);

        Ok(())
    }
}
