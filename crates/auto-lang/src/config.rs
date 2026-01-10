use crate::ast;
use crate::eval::EvalMode;
use crate::eval_config_with_scope;
use crate::interp::Interpreter;
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
    pub interp: Interpreter,
    pub univ: Shared<Universe>,
}

impl AutoConfigReader {
    pub fn new() -> Self {
        let univ = shared(Universe::new());
        let interp = Interpreter::with_univ(univ.clone()).with_eval_mode(EvalMode::CONFIG);
        Self { interp, univ }
    }

    pub fn skip_check(mut self) -> Self {
        self.interp.skip_check();
        self
    }

    pub fn args(self, args: &Obj) -> Self {
        self.univ
            .borrow_mut()
            .define_global("root", Rc::new(Meta::Node(ast::Node::new("root"))));
        self.univ.borrow_mut().set_args(args);
        self
    }

    pub fn parse(&mut self, code: impl Into<AutoStr>) -> AutoResult<AutoConfig> {
        let code = code.into();
        self.interp.interpret(code.as_str())?;
        let result = std::mem::replace(&mut self.interp.result, Value::Nil);

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
        let mut interpreter = eval_config_with_scope(&code, args, univ)?;
        let result = interpreter.result;
        interpreter.result = Value::Nil;
        let args = interpreter.scope.borrow_mut().args.clone();
        if let Value::Node(root) = result {
            Ok(Self {
                code: code.clone(),
                args,
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

        let interp = &mut reader.interp;
        let res = interp.eval("1 + 2");
        println!("{}", res);
        assert_eq!("3", res.to_string());

        Ok(())
    }
}
