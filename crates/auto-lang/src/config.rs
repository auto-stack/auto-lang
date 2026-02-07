use crate::ast;
use crate::eval_config_with_vm;
use crate::scope::Meta;
use crate::AutoResult;
use crate::Universe;
use auto_val::shared;
use auto_val::Obj;
use auto_val::Shared;
use auto_val::{AutoPath, AutoStr, Node, Value};
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

pub struct AutoConfigReader {
    pub univ: Shared<Universe>,
}

impl AutoConfigReader {
    pub fn new() -> Self {
        let univ = shared(Universe::new());
        Self { univ }
    }

    pub fn skip_check(self) -> Self {
        // No-op in AutoVM mode (skip_check was an Interpreter feature)
        self
    }

    pub fn args(mut self, args: &Obj) -> Self {
        self.univ
            .borrow_mut()
            .define_global("root", Rc::new(Meta::Node(ast::Node::new("root"))));
        self.univ.borrow_mut().set_args(args);
        self
    }

    pub fn parse(&mut self, code: impl Into<AutoStr>) -> AutoResult<AutoConfig> {
        let code = code.into();

        // Plan 081 Phase 2: Use AutoVM instead of deprecated Interpreter
        // Note: AutoVM doesn't use Universe directly, so we pass a default one
        let result = eval_config_with_vm(code.as_str(), &Obj::new(), Universe::new())?;

        Ok(AutoConfig {
            code: code.to_string(),
            root: result.to_node(),
            args: Obj::new(),
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
        Self::from_file(path, &Obj::default(), Universe::default())
    }

    pub fn from_file(path: &Path, args: &Obj, univ: Universe) -> AutoResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            format!(
                "Failed to read config file {}: {}",
                path.to_str().unwrap(),
                e
            )
        })?;
        Self::from_code(content, args, univ)
    }

    pub fn new(code: impl Into<String>) -> AutoResult<Self> {
        let univ = Universe::default();
        Self::from_code(code, &Obj::new(), univ)
    }

    pub fn save(&mut self, path: &AutoPath) -> AutoResult<()> {
        let contents = self.root.contents();
        std::fs::write(path.path(), contents.join("\n"))
            .map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    }

    pub fn from_code(code: impl Into<String>, args: &Obj, univ: Universe) -> AutoResult<Self> {
        let code = code.into();

        // Plan 081 Phase 2: Use AutoVM instead of deprecated Interpreter
        let result = eval_config_with_vm(&code, args, univ)?;

        if let Value::Node(root) = result {
            Ok(Self {
                code: code.clone(),
                args: args.clone(),
                root: root,
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
