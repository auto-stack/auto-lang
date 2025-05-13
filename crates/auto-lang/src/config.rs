use crate::AutoResult;
use crate::Universe;
use crate::{eval_config_with_scope, interp};
use auto_val::Obj;
use auto_val::{AutoPath, AutoStr, Node, Value};
use std::path::Path;
pub struct AutoConfig {
    pub code: String,
    pub root: Node,
    pub args: Obj,
    pub interpreter: interp::Interpreter,
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
        Self::from_code(code, &Obj::EMPTY, univ)
    }

    pub fn save(&mut self, path: &AutoPath) -> AutoResult<()> {
        let contents = self.root.contents();
        std::fs::write(path.path(), contents.join("\n"))
            .map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    }

    pub fn from_code(code: impl Into<String>, args: &Obj, univ: Universe) -> AutoResult<Self> {
        let code = code.into();
        let mut interpreter = eval_config_with_scope(&code, args, univ)?;
        let result = interpreter.result;
        interpreter.result = Value::Nil;
        let args = interpreter.scope.borrow_mut().args.clone();
        if let Value::Node(root) = result {
            Ok(Self {
                code: code.clone(),
                args,
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

    pub fn to_xml(&self) -> AutoStr {
        AutoStr::new()
    }

    pub fn eval(&mut self, code: impl Into<AutoStr>) -> Value {
        self.interpreter.eval(code.into().as_str())
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

        let mut config = AutoConfig::new(code)?;
        assert_eq!(config.name(), "hello");
        assert_eq!(config.list_target_names(), vec!["lib(\"alib\")"]);

        let interp = &mut config.interpreter;
        let res = interp.eval("1 + 2");
        println!("{}", res);
        assert_eq!("3", res.to_string());

        Ok(())
    }
}
