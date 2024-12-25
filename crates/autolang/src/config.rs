use autoval::{Node, Value};
use crate::eval::EvalMode;
use crate::interp;
use std::path::Path;
use crate::eval_config;
pub struct AutoConfig {
    pub code: String,
    pub root: Node,
    pub interpreter: interp::Interpreter,
}

impl AutoConfig {
    pub fn new(code: String, root: Node) -> Self {
        Self {
            code,
            root,
            interpreter: interp::Interpreter::new().wit_eval_mode(EvalMode::CONFIG),
        }
    }

    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        let mut interpreter = eval_config(&content)?;
        let result = interpreter.result;
        interpreter.result = Value::Nil;
        if let Value::Node(root) = result {
            Ok(Self {
                code: content,
                root: root.clone(),
                interpreter: interpreter,
            })
        } else {
            Err(format!("Invalid config result: {}", result.repr()))
        }
    }

    pub fn name(&self) -> String {
        self.root.get_prop("name").to_string()
    }

    pub fn version(&self) -> String {
        self.root.get_prop("version").to_string()
    }

    pub fn list_target_names(&self) -> Vec<String> {
        self.root.nodes.iter().map(|n| n.title()).collect()
    }
}
