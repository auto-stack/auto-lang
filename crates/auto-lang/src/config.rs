use auto_val::{Node, Value, AutoStr};
use crate::interp;
use std::path::Path;
use crate::eval_config;
use auto_val::Obj;

pub struct AutoConfig {
    pub code: String,
    pub root: Node,
    pub interpreter: interp::Interpreter,
}

impl AutoConfig {
    pub fn from_file(path: &Path, args: &Obj) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        Self::from_code(content, args)
    }
    
    pub fn from_code(code: impl Into<String>, args: &Obj) -> Result<Self, String> {
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
            Err(format!("Invalid config result: {}", result.repr()))
        }
    }

    pub fn name(&self) -> AutoStr {
        self.root.get_prop("name").auto_str()
    }

    pub fn version(&self) -> AutoStr {
        self.root.get_prop("version").auto_str()
    }

    pub fn list_target_names(&self) -> Vec<String> {
        self.root.nodes.iter().map(|n| n.title()).collect()
    }
}
