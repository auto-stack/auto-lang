use crate::eval::{Evaler, EvalMode};
use crate::parser::Parser;
use crate::scope::Universe;
use auto_val::{Value, AutoStr};
use std::rc::Rc;
use std::cell::RefCell;

pub struct Importer {
    pub path: AutoStr,
    pub scope: Rc<RefCell<Universe>>,
}

pub struct Interpreter {
    pub evaler: Evaler,
    pub scope: Rc<RefCell<Universe>>,
    pub result: Value,
}

impl Interpreter {
    pub fn new() -> Self {
        let scope = Rc::new(RefCell::new(Universe::new()));
        let interpreter = Self { 
            evaler: Evaler::new(scope.clone()),
            scope, 
            result: Value::Nil 
        };
        interpreter
    }

    pub fn with_scope(scope: Universe) -> Self {
        let scope = Rc::new(RefCell::new(scope));
        let interpreter = Self { 
            evaler: Evaler::new(scope.clone()),
            scope, 
            result: Value::Nil 
        };
        interpreter
    }
    
    pub fn wit_eval_mode(mut self, mode: EvalMode) -> Self {
        self.evaler = self.evaler.with_mode(mode);
        self
    }

    pub fn import(&mut self, path: AutoStr) -> Result<(), String> {
        println!("import: {}", path);
        Ok(())
    }

    pub fn interpret(&mut self, code: &str) -> Result<(), String> {
        let mut parser = Parser::new(code, self.scope.clone());
        let ast = parser.parse()?;
        let result = self.evaler.eval(&ast);
        self.result = result;
        Ok(())
    }

    pub fn load_file(&mut self, filename: &str) -> Result<Value, String> {
        let code = std::fs::read_to_string(filename).map_err(|e| format!("Failed to read file: {}", e))?;
        self.interpret(&code)?;
        Ok(self.result.clone())
    }


    pub fn eval(&mut self, code: &str) -> Value {
        match self.interpret(code) {
            Ok(_) => self.result.clone(),
            Err(e) => Value::Error(e),
        }
    }
}
